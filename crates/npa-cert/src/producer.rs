use npa_kernel::Decl;

use crate::{encode_uvar_to, hash_with_domain, CertError, Hash, VerifiedModule};

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
