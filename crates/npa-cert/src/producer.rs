use std::collections::BTreeSet;

use npa_kernel::{Ctx, Decl, Env, Error, Expr, Level};

use crate::{encode_uvar_to, hash_with_domain, CertError, Hash, ProducerLimitKind, VerifiedModule};

/// Sidecar-only producer classification for audit and diagnostics.
///
/// This profile is intentionally not accepted by certificate construction or verification APIs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProducerProfile {
    /// Human-facing surface-language producer.
    HumanSurface,
    /// AI-facing MVP core declaration producer.
    AiCoreMvp,
}

/// Deterministic resource limits for producer candidate checking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProducerLimits {
    /// Maximum declarations accepted in a candidate batch.
    pub max_declarations: u32,
    /// Maximum expression nodes accepted in a candidate declaration.
    pub max_expr_nodes: u32,
    /// Maximum level nodes accepted in a candidate declaration.
    pub max_level_nodes: u32,
    /// Maximum dotted-name components accepted in a candidate declaration.
    pub max_name_components: u32,
    /// Maximum reduction steps available to candidate checking.
    pub max_reduction_steps: u64,
    /// Maximum conversion steps available to candidate checking.
    pub max_conversion_steps: u64,
}

/// Return canonical bytes for a producer limit profile.
///
/// Fields are encoded in [`ProducerLimits`] declaration order, and each numeric field uses the
/// same minimal ULEB128 encoding as certificate canonical binary.
pub fn producer_limits_canonical_bytes(limits: &ProducerLimits) -> Vec<u8> {
    let mut out = Vec::new();
    encode_uvar_to(&mut out, u64::from(limits.max_declarations));
    encode_uvar_to(&mut out, u64::from(limits.max_expr_nodes));
    encode_uvar_to(&mut out, u64::from(limits.max_level_nodes));
    encode_uvar_to(&mut out, u64::from(limits.max_name_components));
    encode_uvar_to(&mut out, limits.max_reduction_steps);
    encode_uvar_to(&mut out, limits.max_conversion_steps);
    out
}

/// Return the canonical hash for a producer limit profile.
pub fn producer_limits_hash(limits: &ProducerLimits) -> Hash {
    hash_with_domain(
        b"NPA-PRODUCER-LIMITS-0.1",
        &producer_limits_canonical_bytes(limits),
    )
}

/// Return whether limit profile `a` is at least as strict as profile `b`.
///
/// A profile is stricter-or-equal only when every field is less than or equal to the corresponding
/// field in `b`.
pub fn stricter_or_equal(a: &ProducerLimits, b: &ProducerLimits) -> bool {
    a.max_declarations <= b.max_declarations
        && a.max_expr_nodes <= b.max_expr_nodes
        && a.max_level_nodes <= b.max_level_nodes
        && a.max_name_components <= b.max_name_components
        && a.max_reduction_steps <= b.max_reduction_steps
        && a.max_conversion_steps <= b.max_conversion_steps
}

/// Precheck a single producer candidate against an existing kernel environment under limits.
///
/// This fast path does not emit `.npcert` bytes or create a verified module. It only performs
/// deterministic schema limit checks and a metered kernel precheck for source declarations that
/// the AI producer MVP is allowed to submit directly.
pub fn precheck_core_decl_candidate(
    env: &Env,
    candidate: &CoreDeclCandidate,
    limits: &ProducerLimits,
) -> Result<(), CertError> {
    ensure_candidate_schema_limits(&candidate.declaration, limits)?;
    let mut whnf_fuel = fuel_to_usize(
        limits.max_reduction_steps,
        ProducerLimitKind::MaxReductionSteps,
    )?;
    let mut conversion_fuel = fuel_to_usize(
        limits.max_conversion_steps,
        ProducerLimitKind::MaxConversionSteps,
    )?;
    precheck_decl_with_fuel(
        env,
        &candidate.declaration,
        &mut whnf_fuel,
        &mut conversion_fuel,
    )
}

/// Untrusted core declaration candidate submitted by a producer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoreDeclCandidate {
    /// Already elaborated kernel declaration proposed by the producer.
    pub declaration: Decl,
}

/// Batch of untrusted core declaration candidates checked against a current environment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateBatch<'a> {
    /// Verified imports available to the candidate batch.
    pub imports: &'a [VerifiedModule],
    /// Previously checked current-module declarations available to later candidates.
    pub prior_current_decls: &'a [CheckedDeclCandidate],
    /// Candidate declarations to check, in input order.
    pub candidates: Vec<CoreDeclCandidate>,
    /// Deterministic resource limits applied to this batch.
    pub limits: ProducerLimits,
}

/// Opaque token for a candidate declaration accepted by producer checking.
///
/// The token has no public constructor and exposes no raw declaration getter. Later producer
/// milestones construct this token only after candidate checking has recomputed its private hashes
/// and fingerprints.
///
/// ```compile_fail
/// use npa_cert::{CandidateHashPreview, CheckedDeclCandidate, ProducerLimits};
/// use npa_kernel::{Decl, Expr, Level};
///
/// let declaration = Decl::Axiom {
///     name: "P".to_owned(),
///     universe_params: vec![],
///     ty: Expr::sort(Level::zero()),
/// };
/// let zero = [0_u8; 32];
/// let limits = ProducerLimits {
///     max_declarations: 1,
///     max_expr_nodes: 1,
///     max_level_nodes: 1,
///     max_name_components: 1,
///     max_reduction_steps: 1,
///     max_conversion_steps: 1,
/// };
///
/// let _token = CheckedDeclCandidate {
///     declaration,
///     preview_hashes: CandidateHashPreview {
///         type_hash: None,
///         body_hash: None,
///         decl_interface_hash: None,
///         decl_certificate_hash: None,
///     },
///     pre_env_fingerprint: zero,
///     post_env_fingerprint: zero,
///     prior_chain_fingerprint: zero,
///     limits,
///     limit_profile_hash: zero,
///     decl_interface_hash: zero,
///     decl_certificate_hash: zero,
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedDeclCandidate {
    declaration: Decl,
    preview_hashes: CandidateHashPreview,
    pre_env_fingerprint: Hash,
    post_env_fingerprint: Hash,
    prior_chain_fingerprint: Hash,
    limits: ProducerLimits,
    limit_profile_hash: Hash,
    decl_interface_hash: Hash,
    decl_certificate_hash: Hash,
}

impl CheckedDeclCandidate {
    /// Return non-authoritative preview hashes captured while checking this token.
    pub fn preview_hashes(&self) -> CandidateHashPreview {
        self.preview_hashes
    }

    /// Return the producer environment fingerprint before this declaration was accepted.
    pub fn pre_env_fingerprint(&self) -> Hash {
        self.pre_env_fingerprint
    }

    /// Return the producer environment fingerprint after this declaration was accepted.
    pub fn post_env_fingerprint(&self) -> Hash {
        self.post_env_fingerprint
    }

    /// Return the prior-chain fingerprint committed by this token.
    pub fn prior_chain_fingerprint(&self) -> Hash {
        self.prior_chain_fingerprint
    }

    /// Return the deterministic limits used when this token was created.
    pub fn limits(&self) -> ProducerLimits {
        self.limits
    }

    /// Return the diagnostic hash of the limits used when this token was created.
    pub fn limit_profile_hash(&self) -> Hash {
        self.limit_profile_hash
    }

    /// Return whether the stored limits match this token's diagnostic limit-profile hash.
    pub fn limit_profile_hash_matches(&self) -> bool {
        producer_limits_hash(&self.limits) == self.limit_profile_hash
    }

    /// Return whether this token's checked limits are reusable under `batch_limits`.
    pub fn limits_are_reusable_under(&self, batch_limits: &ProducerLimits) -> bool {
        stricter_or_equal(&self.limits, batch_limits)
    }

    /// Return the token's diagnostic declaration interface hash.
    pub fn decl_interface_hash(&self) -> Hash {
        self.decl_interface_hash
    }

    /// Return the token's diagnostic declaration certificate hash.
    pub fn decl_certificate_hash(&self) -> Hash {
        self.decl_certificate_hash
    }
}

/// Non-authoritative hash preview computed while checking a producer candidate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CandidateHashPreview {
    /// Preview of the declaration type hash, when available.
    pub type_hash: Option<Hash>,
    /// Preview of the declaration body or proof hash, when available.
    pub body_hash: Option<Hash>,
    /// Preview of the declaration interface hash, when available.
    pub decl_interface_hash: Option<Hash>,
    /// Preview of the declaration certificate hash, when available.
    pub decl_certificate_hash: Option<Hash>,
}

/// Per-candidate status returned by producer batch checking.
#[derive(Clone, Debug, PartialEq, Eq)]
// Phase 2 specifies a by-value accepted token; do not box the public API boundary.
#[allow(clippy::large_enum_variant)]
pub enum CandidateStatus {
    /// Candidate passed producer precheck and became an opaque token.
    Accepted(CheckedDeclCandidate),
    /// Candidate was rejected with a deterministic certificate error.
    Rejected(CertError),
}

/// Result for a producer candidate batch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateBatchResult {
    /// One status per input candidate, in the same order.
    pub statuses: Vec<CandidateStatus>,
}

fn fuel_to_usize(value: u64, limit: ProducerLimitKind) -> Result<usize, CertError> {
    usize::try_from(value).map_err(|_| CertError::ProducerLimitExceeded { limit })
}

fn ensure_candidate_schema_limits(decl: &Decl, limits: &ProducerLimits) -> Result<(), CertError> {
    if limits.max_declarations == 0 {
        return Err(CertError::ProducerLimitExceeded {
            limit: ProducerLimitKind::MaxDeclarations,
        });
    }

    let expr_nodes = decl_expr_node_count(decl);
    if expr_nodes > limits.max_expr_nodes as u64 {
        return Err(CertError::ProducerLimitExceeded {
            limit: ProducerLimitKind::MaxExprNodes,
        });
    }

    let level_nodes = decl_level_node_count(decl);
    if level_nodes > limits.max_level_nodes as u64 {
        return Err(CertError::ProducerLimitExceeded {
            limit: ProducerLimitKind::MaxLevelNodes,
        });
    }

    if decl_max_name_components(decl) > limits.max_name_components as u64 {
        return Err(CertError::ProducerLimitExceeded {
            limit: ProducerLimitKind::MaxNameComponents,
        });
    }

    Ok(())
}

fn precheck_decl_with_fuel(
    env: &Env,
    decl: &Decl,
    whnf_fuel: &mut usize,
    conversion_fuel: &mut usize,
) -> Result<(), CertError> {
    match decl {
        Decl::Axiom {
            name,
            universe_params,
            ty,
        } => {
            ensure_fresh(env, name)?;
            let delta = validate_universe_params(universe_params)?;
            expect_sort_with_fuel(env, &delta, ty, whnf_fuel, conversion_fuel)
        }
        Decl::Def {
            name,
            universe_params,
            ty,
            value,
            ..
        } => {
            ensure_fresh(env, name)?;
            let delta = validate_universe_params(universe_params)?;
            expect_sort_with_fuel(env, &delta, ty, whnf_fuel, conversion_fuel)?;
            env.check_with_fuel_metered(
                &Ctx::new(),
                &delta,
                value,
                ty,
                whnf_fuel,
                conversion_fuel,
            )?;
            Ok(())
        }
        Decl::Theorem {
            name,
            universe_params,
            ty,
            proof,
        } => {
            ensure_fresh(env, name)?;
            let delta = validate_universe_params(universe_params)?;
            expect_sort_with_fuel(env, &delta, ty, whnf_fuel, conversion_fuel)?;
            env.check_with_fuel_metered(
                &Ctx::new(),
                &delta,
                proof,
                ty,
                whnf_fuel,
                conversion_fuel,
            )?;
            Ok(())
        }
        Decl::Inductive { name, .. } => Err(CertError::Kernel(Error::InvalidInductive(format!(
            "{name} inductive candidate precheck is not part of the Phase 2 AI MVP"
        )))),
        Decl::Constructor { .. } | Decl::Recursor { .. } => Err(CertError::UnknownDependency {
            name: crate::Name::from_dotted(decl.name()),
        }),
    }
}

fn ensure_fresh(env: &Env, name: &str) -> Result<(), CertError> {
    if env.decl(name).is_some() {
        Err(CertError::Kernel(Error::DuplicateDecl(name.to_owned())))
    } else {
        Ok(())
    }
}

fn validate_universe_params(params: &[String]) -> Result<Vec<String>, CertError> {
    let mut seen = BTreeSet::new();
    for param in params {
        if !seen.insert(param.clone()) {
            return Err(CertError::Kernel(Error::UnknownUniverseParam(
                param.clone(),
            )));
        }
    }
    Ok(params.to_vec())
}

fn expect_sort_with_fuel(
    env: &Env,
    delta: &[String],
    term: &Expr,
    whnf_fuel: &mut usize,
    conversion_fuel: &mut usize,
) -> Result<(), CertError> {
    let ty = env.infer_with_fuel_metered(&Ctx::new(), delta, term, whnf_fuel, conversion_fuel)?;
    match env.whnf_with_fuel_metered(&Ctx::new(), delta, &ty, whnf_fuel)? {
        Expr::Sort(_) => Ok(()),
        actual => Err(CertError::Kernel(Error::ExpectedSort { actual })),
    }
}

fn decl_expr_node_count(decl: &Decl) -> u64 {
    match decl {
        Decl::Axiom { ty, .. } => expr_node_count(ty),
        Decl::Def { ty, value, .. } => expr_node_count(ty) + expr_node_count(value),
        Decl::Theorem { ty, proof, .. } => expr_node_count(ty) + expr_node_count(proof),
        Decl::Inductive { ty, data, .. } => {
            expr_node_count(ty)
                + data
                    .params
                    .iter()
                    .map(|binder| expr_node_count(&binder.ty))
                    .sum::<u64>()
                + data
                    .indices
                    .iter()
                    .map(|binder| expr_node_count(&binder.ty))
                    .sum::<u64>()
                + data
                    .constructors
                    .iter()
                    .map(|constructor| expr_node_count(&constructor.ty))
                    .sum::<u64>()
                + data
                    .recursor
                    .iter()
                    .map(|recursor| expr_node_count(&recursor.ty))
                    .sum::<u64>()
        }
        Decl::Constructor { ty, .. } | Decl::Recursor { ty, .. } => expr_node_count(ty),
    }
}

fn expr_node_count(expr: &Expr) -> u64 {
    match expr {
        Expr::Sort(_) | Expr::BVar(_) | Expr::Const { .. } => 1,
        Expr::App(fun, arg) => 1 + expr_node_count(fun) + expr_node_count(arg),
        Expr::Lam { ty, body, .. } | Expr::Pi { ty, body, .. } => {
            1 + expr_node_count(ty) + expr_node_count(body)
        }
        Expr::Let {
            ty, value, body, ..
        } => 1 + expr_node_count(ty) + expr_node_count(value) + expr_node_count(body),
    }
}

fn decl_level_node_count(decl: &Decl) -> u64 {
    match decl {
        Decl::Axiom { ty, .. } => expr_level_node_count(ty),
        Decl::Def { ty, value, .. } => expr_level_node_count(ty) + expr_level_node_count(value),
        Decl::Theorem { ty, proof, .. } => expr_level_node_count(ty) + expr_level_node_count(proof),
        Decl::Inductive { ty, data, .. } => {
            expr_level_node_count(ty)
                + level_node_count(&data.sort)
                + data
                    .params
                    .iter()
                    .map(|binder| expr_level_node_count(&binder.ty))
                    .sum::<u64>()
                + data
                    .indices
                    .iter()
                    .map(|binder| expr_level_node_count(&binder.ty))
                    .sum::<u64>()
                + data
                    .constructors
                    .iter()
                    .map(|constructor| expr_level_node_count(&constructor.ty))
                    .sum::<u64>()
                + data
                    .recursor
                    .iter()
                    .map(|recursor| expr_level_node_count(&recursor.ty))
                    .sum::<u64>()
        }
        Decl::Constructor { ty, .. } | Decl::Recursor { ty, .. } => expr_level_node_count(ty),
    }
}

fn expr_level_node_count(expr: &Expr) -> u64 {
    match expr {
        Expr::Sort(level) => level_node_count(level),
        Expr::BVar(_) => 0,
        Expr::Const { levels, .. } => levels.iter().map(level_node_count).sum(),
        Expr::App(fun, arg) => expr_level_node_count(fun) + expr_level_node_count(arg),
        Expr::Lam { ty, body, .. } | Expr::Pi { ty, body, .. } => {
            expr_level_node_count(ty) + expr_level_node_count(body)
        }
        Expr::Let {
            ty, value, body, ..
        } => expr_level_node_count(ty) + expr_level_node_count(value) + expr_level_node_count(body),
    }
}

fn level_node_count(level: &Level) -> u64 {
    match level {
        Level::Zero | Level::Param(_) => 1,
        Level::Succ(level) => 1 + level_node_count(level),
        Level::Max(lhs, rhs) | Level::IMax(lhs, rhs) => {
            1 + level_node_count(lhs) + level_node_count(rhs)
        }
    }
}

fn decl_max_name_components(decl: &Decl) -> u64 {
    match decl {
        Decl::Axiom {
            name,
            universe_params,
            ty,
        } => max_name_components(
            std::iter::once(name.as_str()).chain(universe_params.iter().map(String::as_str)),
            std::iter::once(ty),
        ),
        Decl::Def {
            name,
            universe_params,
            ty,
            value,
            ..
        } => max_name_components(
            std::iter::once(name.as_str()).chain(universe_params.iter().map(String::as_str)),
            [ty, value],
        ),
        Decl::Theorem {
            name,
            universe_params,
            ty,
            proof,
        } => max_name_components(
            std::iter::once(name.as_str()).chain(universe_params.iter().map(String::as_str)),
            [ty, proof],
        ),
        Decl::Inductive {
            name,
            universe_params,
            ty,
            data,
        } => {
            let names = std::iter::once(name.as_str())
                .chain(universe_params.iter().map(String::as_str))
                .chain(std::iter::once(data.name.as_str()))
                .chain(data.universe_params.iter().map(String::as_str))
                .chain(data.params.iter().map(|binder| binder.name.as_str()))
                .chain(data.indices.iter().map(|binder| binder.name.as_str()))
                .chain(
                    data.constructors
                        .iter()
                        .map(|constructor| constructor.name.as_str()),
                )
                .chain(data.recursor.iter().map(|recursor| recursor.name.as_str()))
                .chain(
                    data.recursor
                        .iter()
                        .flat_map(|recursor| recursor.universe_params.iter().map(String::as_str)),
                );
            let exprs = std::iter::once(ty)
                .chain(data.params.iter().map(|binder| &binder.ty))
                .chain(data.indices.iter().map(|binder| &binder.ty))
                .chain(data.constructors.iter().map(|constructor| &constructor.ty))
                .chain(data.recursor.iter().map(|recursor| &recursor.ty));
            max_name_components(names, exprs).max(level_max_name_components(&data.sort))
        }
        Decl::Constructor {
            name,
            universe_params,
            ty,
            inductive,
        } => max_name_components(
            std::iter::once(name.as_str())
                .chain(universe_params.iter().map(String::as_str))
                .chain(std::iter::once(inductive.as_str())),
            std::iter::once(ty),
        ),
        Decl::Recursor {
            name,
            universe_params,
            ty,
            inductive,
            ..
        } => max_name_components(
            std::iter::once(name.as_str())
                .chain(universe_params.iter().map(String::as_str))
                .chain(std::iter::once(inductive.as_str())),
            std::iter::once(ty),
        ),
    }
}

fn max_name_components<'a>(
    names: impl IntoIterator<Item = &'a str>,
    exprs: impl IntoIterator<Item = &'a Expr>,
) -> u64 {
    names
        .into_iter()
        .map(name_component_count)
        .chain(exprs.into_iter().map(expr_max_name_components))
        .max()
        .unwrap_or(0)
}

fn expr_max_name_components(expr: &Expr) -> u64 {
    match expr {
        Expr::Sort(level) => level_max_name_components(level),
        Expr::BVar(_) => 0,
        Expr::Const { name, levels } => levels
            .iter()
            .map(level_max_name_components)
            .chain(std::iter::once(name_component_count(name)))
            .max()
            .unwrap_or(0),
        Expr::App(fun, arg) => expr_max_name_components(fun).max(expr_max_name_components(arg)),
        Expr::Lam {
            binder, ty, body, ..
        }
        | Expr::Pi {
            binder, ty, body, ..
        } => name_component_count(binder)
            .max(expr_max_name_components(ty))
            .max(expr_max_name_components(body)),
        Expr::Let {
            binder,
            ty,
            value,
            body,
        } => name_component_count(binder)
            .max(expr_max_name_components(ty))
            .max(expr_max_name_components(value))
            .max(expr_max_name_components(body)),
    }
}

fn level_max_name_components(level: &Level) -> u64 {
    match level {
        Level::Zero => 0,
        Level::Param(name) => name_component_count(name),
        Level::Succ(level) => level_max_name_components(level),
        Level::Max(lhs, rhs) | Level::IMax(lhs, rhs) => {
            level_max_name_components(lhs).max(level_max_name_components(rhs))
        }
    }
}

fn name_component_count(name: &str) -> u64 {
    name.split('.').count() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_limits() -> ProducerLimits {
        ProducerLimits {
            max_declarations: 1,
            max_expr_nodes: 8,
            max_level_nodes: 2,
            max_name_components: 4,
            max_reduction_steps: 16,
            max_conversion_steps: 16,
        }
    }

    fn test_token(limits: ProducerLimits, limit_profile_hash: Hash) -> CheckedDeclCandidate {
        let zero = [0_u8; 32];
        CheckedDeclCandidate {
            declaration: Decl::Axiom {
                name: "P".to_owned(),
                universe_params: vec![],
                ty: Expr::sort(Level::zero()),
            },
            preview_hashes: CandidateHashPreview {
                type_hash: None,
                body_hash: None,
                decl_interface_hash: None,
                decl_certificate_hash: None,
            },
            pre_env_fingerprint: zero,
            post_env_fingerprint: zero,
            prior_chain_fingerprint: zero,
            limits,
            limit_profile_hash,
            decl_interface_hash: zero,
            decl_certificate_hash: zero,
        }
    }

    #[test]
    fn checked_decl_candidate_limit_helpers_use_private_limits() {
        let limits = test_limits();
        let token = test_token(limits, producer_limits_hash(&limits));

        assert!(token.limit_profile_hash_matches());

        let reusable_batch_limits = ProducerLimits {
            max_declarations: limits.max_declarations + 1,
            max_expr_nodes: limits.max_expr_nodes + 1,
            max_level_nodes: limits.max_level_nodes + 1,
            max_name_components: limits.max_name_components + 1,
            max_reduction_steps: limits.max_reduction_steps + 1,
            max_conversion_steps: limits.max_conversion_steps + 1,
        };
        assert!(token.limits_are_reusable_under(&reusable_batch_limits));

        let too_strict_batch_limits = ProducerLimits {
            max_expr_nodes: limits.max_expr_nodes - 1,
            ..reusable_batch_limits
        };
        assert!(!token.limits_are_reusable_under(&too_strict_batch_limits));

        let mismatched_token = test_token(limits, [0_u8; 32]);
        assert!(!mismatched_token.limit_profile_hash_matches());
    }
}
