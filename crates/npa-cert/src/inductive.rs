use npa_kernel::expr::collect_apps;
use npa_kernel::level::{level_eq, levels_eq};
use npa_kernel::{ConstructorDecl, Expr, InductiveDecl, Level, RecursorDecl, RecursorRules};

use crate::{CertError, Name, Result};

/// Result of the Phase 2 deterministic inductive artifact profile classifier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InductiveArtifactProfileCheckV1 {
    /// The declaration is in the MVP recursor profile.
    SupportedMvpRecursor,
    /// The declaration needs a recursor profile outside the MVP.
    UnsupportedMvpRecursorProfile(UnsupportedMvpRecursorProfileV1),
}

/// Unsupported recursor profile reason returned by the Phase 2 classifier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnsupportedMvpRecursorProfileV1 {
    /// The declaration would require a large-elimination profile.
    LargeEliminationRequired,
    /// The declaration would require mutual or nested recursor generation.
    MutualOrNestedRecursorRequired,
    /// The declaration has an eliminator shape not handled by the MVP generator.
    UnsupportedEliminatorShape,
}

/// Classify whether an inductive declaration is supported by the Phase 2 MVP artifact generator.
pub fn classify_inductive_artifact_profile_v1(
    base: &InductiveDecl,
) -> InductiveArtifactProfileCheckV1 {
    if base.recursor.is_some() || !base.indices.is_empty() {
        return InductiveArtifactProfileCheckV1::UnsupportedMvpRecursorProfile(
            UnsupportedMvpRecursorProfileV1::UnsupportedEliminatorShape,
        );
    }
    for constructor in &base.constructors {
        let (domains, _) = peel_pi_domains(&constructor.ty);
        for (domain_index, domain) in domains.iter().enumerate() {
            if domain_index >= base.params.len()
                && is_direct_recursive_domain(
                    &base.name,
                    &base.universe_params,
                    base.params.len(),
                    base.indices.len(),
                    domain,
                    domain_index,
                )
                .unwrap_or(false)
            {
                continue;
            }
            if contains_const(domain, &base.name) {
                return InductiveArtifactProfileCheckV1::UnsupportedMvpRecursorProfile(
                    UnsupportedMvpRecursorProfileV1::MutualOrNestedRecursorRequired,
                );
            }
        }
    }
    InductiveArtifactProfileCheckV1::SupportedMvpRecursor
}

/// Generate the Phase 2 MVP inductive artifacts for a supported base declaration.
pub fn generate_inductive_artifacts_v1(base: &InductiveDecl) -> Result<InductiveDecl> {
    if classify_inductive_artifact_profile_v1(base)
        != InductiveArtifactProfileCheckV1::SupportedMvpRecursor
    {
        return Err(CertError::InductiveGeneratedArtifactMismatch {
            name: Name::from_dotted(&base.name),
        });
    }
    let rules = RecursorRules::new(
        base.params.len() + 1,
        base.params.len() + 1 + base.constructors.len(),
    );
    let recursor_universe_params = recursor_universe_params(base);
    let recursor_ty = generated_recursor_type(base, &recursor_universe_params, &rules)?;
    let mut final_decl = base.clone();
    final_decl.recursor = Some(RecursorDecl::with_rules(
        format!("{}.rec", base.name),
        recursor_universe_params,
        recursor_ty,
        rules,
    ));
    Ok(final_decl)
}

fn recursor_universe_params(base: &InductiveDecl) -> Vec<String> {
    let mut params = base.universe_params.clone();
    if level_eq(&base.sort, &Level::zero()) {
        return params;
    }
    let mut index = 0usize;
    loop {
        let candidate = if index == 0 {
            "u".to_owned()
        } else {
            format!("u{index}")
        };
        if !params.iter().any(|param| param == &candidate) {
            params.push(candidate);
            return params;
        }
        index += 1;
    }
}

fn generated_recursor_type(
    base: &InductiveDecl,
    recursor_universe_params: &[String],
    rules: &RecursorRules,
) -> Result<Expr> {
    let param_count = base.params.len();
    let index_count = base.indices.len();
    let mut domains = base
        .params
        .iter()
        .map(|param| param.ty.clone())
        .collect::<Vec<_>>();
    let motive_target = inductive_target_expr(
        &base.name,
        &base.universe_params,
        domains.len(),
        param_count,
    )?;
    domains.push(Expr::pi(
        "_",
        motive_target,
        Expr::sort(expected_motive_level(base, recursor_universe_params)),
    ));

    for (constructor_index, constructor) in base.constructors.iter().enumerate() {
        domains.push(expected_minor_type_expr(
            base,
            param_count,
            index_count,
            constructor,
            constructor_index,
        )?);
    }

    let major_domain = inductive_target_expr(
        &base.name,
        &base.universe_params,
        domains.len(),
        param_count,
    )?;
    domains.push(major_domain);
    let body = motive_app(
        domains.len(),
        param_count,
        bvar_for_abs(domains.len(), rules.major_index)?,
    )?;
    Ok(mk_pi_from_domains(domains, body))
}

fn expected_motive_level(base: &InductiveDecl, recursor_universe_params: &[String]) -> Level {
    if level_eq(&base.sort, &Level::zero()) {
        return Level::zero();
    }
    if let Some(param) = recursor_universe_params
        .iter()
        .rev()
        .find(|param| !base.universe_params.contains(*param))
    {
        return Level::param(param.clone());
    }
    recursor_universe_params
        .last()
        .map(|param| Level::param(param.clone()))
        .unwrap_or_else(|| base.sort.clone())
}

fn inductive_target_expr(
    inductive_name: &str,
    universe_params: &[String],
    ctx_len: usize,
    param_count: usize,
) -> Result<Expr> {
    let levels = universe_params
        .iter()
        .map(|param| Level::param(param.clone()))
        .collect();
    let args = (0..param_count)
        .map(|param_abs| bvar_for_abs(ctx_len, param_abs))
        .collect::<Result<Vec<_>>>()?;
    Ok(Expr::apps(
        Expr::konst(inductive_name.to_owned(), levels),
        args,
    ))
}

fn expected_minor_type_expr(
    base: &InductiveDecl,
    param_count: usize,
    index_count: usize,
    constructor: &ConstructorDecl,
    constructor_index: usize,
) -> Result<Expr> {
    let (constructor_domains, _) = peel_pi_domains(&constructor.ty);
    if constructor_domains.len() < param_count {
        return Err(CertError::InductiveGeneratedArtifactMismatch {
            name: Name::from_dotted(&base.name),
        });
    }

    let prefix_len = param_count + 1 + constructor_index;
    let motive_abs = param_count;
    let mut source_to_target: Vec<usize> = (0..param_count).collect();
    let mut target_ctx_len = prefix_len;
    let mut expected_domains = Vec::new();
    let mut field_abs = Vec::new();

    for (field_index, field_domain) in constructor_domains[param_count..].iter().enumerate() {
        let source_ctx_len = param_count + field_index;
        expected_domains.push(remap_bvars(
            field_domain,
            source_ctx_len,
            target_ctx_len,
            &source_to_target,
        )?);

        source_to_target.push(target_ctx_len);
        field_abs.push(target_ctx_len);
        target_ctx_len += 1;

        if is_direct_recursive_domain(
            &base.name,
            &base.universe_params,
            param_count,
            index_count,
            field_domain,
            source_ctx_len,
        )? {
            expected_domains.push(motive_app(target_ctx_len, motive_abs, Expr::bvar(0))?);
            target_ctx_len += 1;
        }
    }

    let mut constructor_args = Vec::with_capacity(param_count + field_abs.len());
    for param_abs in 0..param_count {
        constructor_args.push(bvar_for_abs(target_ctx_len, param_abs)?);
    }
    for field_abs in field_abs {
        constructor_args.push(bvar_for_abs(target_ctx_len, field_abs)?);
    }

    let levels = base
        .universe_params
        .iter()
        .map(|param| Level::param(param.clone()))
        .collect();
    let constructor_value = Expr::apps(
        Expr::konst(constructor.name.clone(), levels),
        constructor_args,
    );
    let result = motive_app(target_ctx_len, motive_abs, constructor_value)?;

    Ok(mk_pi_from_domains(expected_domains, result))
}

fn peel_pi_domains(ty: &Expr) -> (Vec<Expr>, Expr) {
    let mut domains = Vec::new();
    let mut current = ty.clone();
    while let Expr::Pi { ty, body, .. } = current {
        domains.push(*ty);
        current = *body;
    }
    (domains, current)
}

fn motive_app(ctx_len: usize, motive_abs: usize, target: Expr) -> Result<Expr> {
    Ok(Expr::app(bvar_for_abs(ctx_len, motive_abs)?, target))
}

fn bvar_for_abs(ctx_len: usize, abs: usize) -> Result<Expr> {
    if abs >= ctx_len {
        return Err(CertError::InvalidBVar { index: abs as u32 });
    }
    Ok(Expr::bvar((ctx_len - 1 - abs) as u32))
}

fn mk_pi_from_domains(domains: Vec<Expr>, body: Expr) -> Expr {
    domains
        .into_iter()
        .rev()
        .fold(body, |body, domain| Expr::pi("_", domain, body))
}

fn remap_bvars(
    expr: &Expr,
    source_ctx_len: usize,
    target_ctx_len: usize,
    source_to_target: &[usize],
) -> Result<Expr> {
    match expr {
        Expr::Sort(level) => Ok(Expr::sort(level.clone())),
        Expr::BVar(index) => {
            let index = *index as usize;
            if index >= source_ctx_len {
                return Err(CertError::InvalidBVar {
                    index: index as u32,
                });
            }
            let source_abs = source_ctx_len - 1 - index;
            let target_abs =
                source_to_target
                    .get(source_abs)
                    .copied()
                    .ok_or(CertError::InvalidBVar {
                        index: index as u32,
                    })?;
            bvar_for_abs(target_ctx_len, target_abs)
        }
        Expr::Const { name, levels } => Ok(Expr::konst(name.clone(), levels.clone())),
        Expr::App(fun, arg) => Ok(Expr::app(
            remap_bvars(fun, source_ctx_len, target_ctx_len, source_to_target)?,
            remap_bvars(arg, source_ctx_len, target_ctx_len, source_to_target)?,
        )),
        Expr::Lam { binder, ty, body } => {
            let mut body_map = source_to_target.to_vec();
            body_map.push(target_ctx_len);
            Ok(Expr::lam(
                binder.clone(),
                remap_bvars(ty, source_ctx_len, target_ctx_len, source_to_target)?,
                remap_bvars(body, source_ctx_len + 1, target_ctx_len + 1, &body_map)?,
            ))
        }
        Expr::Pi { binder, ty, body } => {
            let mut body_map = source_to_target.to_vec();
            body_map.push(target_ctx_len);
            Ok(Expr::pi(
                binder.clone(),
                remap_bvars(ty, source_ctx_len, target_ctx_len, source_to_target)?,
                remap_bvars(body, source_ctx_len + 1, target_ctx_len + 1, &body_map)?,
            ))
        }
        Expr::Let {
            binder,
            ty,
            value,
            body,
        } => {
            let mut body_map = source_to_target.to_vec();
            body_map.push(target_ctx_len);
            Ok(Expr::let_in(
                binder.clone(),
                remap_bvars(ty, source_ctx_len, target_ctx_len, source_to_target)?,
                remap_bvars(value, source_ctx_len, target_ctx_len, source_to_target)?,
                remap_bvars(body, source_ctx_len + 1, target_ctx_len + 1, &body_map)?,
            ))
        }
    }
}

fn is_direct_recursive_domain(
    inductive_name: &str,
    universe_params: &[String],
    param_count: usize,
    index_count: usize,
    domain: &Expr,
    ctx_len: usize,
) -> Result<bool> {
    let (head, args) = collect_apps(domain);
    let levels = match head {
        Expr::Const { name, levels } if name == inductive_name => levels,
        _ => return Ok(false),
    };

    let expected_levels: Vec<_> = universe_params
        .iter()
        .map(|param| Level::param(param.clone()))
        .collect();
    if !levels_eq(&levels, &expected_levels) || args.len() != param_count + index_count {
        return Ok(false);
    }

    for (param_index, arg) in args.iter().take(param_count).enumerate() {
        let expected = bvar_for_abs(ctx_len, param_index)?;
        if arg != &expected {
            return Ok(false);
        }
    }

    Ok(args.iter().all(|arg| !contains_const(arg, inductive_name)))
}

fn contains_const(expr: &Expr, needle: &str) -> bool {
    match expr {
        Expr::Sort(_) | Expr::BVar(_) => false,
        Expr::Const { name, .. } => name == needle,
        Expr::App(fun, arg) => contains_const(fun, needle) || contains_const(arg, needle),
        Expr::Lam { ty, body, .. } | Expr::Pi { ty, body, .. } => {
            contains_const(ty, needle) || contains_const(body, needle)
        }
        Expr::Let {
            ty, value, body, ..
        } => {
            contains_const(ty, needle)
                || contains_const(value, needle)
                || contains_const(body, needle)
        }
    }
}
