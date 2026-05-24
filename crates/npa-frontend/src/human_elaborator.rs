use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{
    builtin_machine_callable_profile, elaborator::certificate_imports_for_module,
    machine_callable_profile_from_human_binders, parse_human_module_with_source_interfaces,
    resolve_human_module_with_source_interfaces, HumanBinder, HumanBinderKind, HumanCompileOptions,
    HumanDeclValue, HumanDiagnostic, HumanDiagnosticKind, HumanDiagnosticPayload,
    HumanDiagnosticPhase, HumanExpr, HumanGeneratedDeclarationKind, HumanGlobalRef,
    HumanGlobalScopeEntry, HumanHoleGoal, HumanHoleGoalLocal, HumanImplicitMode,
    HumanImportedSourceInterface, HumanItem, HumanLevel, HumanName, HumanResolvedName,
    HumanResolvedNameUse, HumanResolvedNotationEntry, HumanResolvedNotationUse, HumanResult,
    HumanSourceDeclarationKind, HumanSourceDeclarationMetadata, HumanSourceInterface,
    HumanTacticScript, HumanUnsolvedMeta, HumanUnsolvedMetaKind, MachineBinder,
    MachineCallableBinderVisibility, MachineCheckedCurrentDecl, MachineCheckedCurrentGeneratedDecl,
    MachineDecl, MachineLevel, MachineLocalDecl, MachineName, MachineTerm, MachineUniverseParam,
    ResolvedHumanModule, Span, VerifiedImport,
};
use npa_kernel::{
    eq_inductive, eq_rec_type, nat_inductive, subst, Binder, ConstructorDecl, Ctx, Decl, Env,
    Error, Expr, InductiveDecl, Level, RecursorDecl, Reducibility,
};

const MAX_HUMAN_IMPLICIT_INSERTION_STEPS: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanCoreCompileOutput {
    pub core_module: npa_cert::CoreModule,
    pub source_interface: HumanSourceInterface,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanCertificateCompileOutput {
    pub certificate: npa_cert::ModuleCert,
    pub source_interface: HumanSourceInterface,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanProofStartCore {
    pub module: npa_cert::ModuleName,
    pub theorem_name: npa_cert::Name,
    pub source_index: u64,
    pub universe_params: Vec<String>,
    pub theorem_type: Expr,
    pub prior_declarations: Vec<Decl>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanProofStartCoreOutput {
    pub proof: HumanProofStartCore,
    pub source_interface: HumanSourceInterface,
    pub active_imports: Vec<HumanImportedSourceInterface>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanByProofCore {
    pub source_index: u64,
    pub proof: Expr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanByProofTarget {
    pub source_index: u64,
    pub theorem_name: npa_cert::Name,
    pub script: HumanTacticScript,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanByProofTargetsOutput {
    pub targets: Vec<HumanByProofTarget>,
    pub source_interface: HumanSourceInterface,
    pub active_imports: Vec<HumanImportedSourceInterface>,
}

#[derive(Clone, Debug)]
pub struct HumanProofStartCoreWithProofsRequest<'a> {
    pub file_id: crate::FileId,
    pub module_name: npa_cert::ModuleName,
    pub theorem_name: npa_cert::Name,
    pub source: &'a str,
    pub verified_imports: &'a [VerifiedImport],
    pub imported_source_interfaces: &'a [HumanImportedSourceInterface],
    pub prior_by_proofs: &'a [HumanByProofCore],
    pub options: &'a HumanCompileOptions,
}

#[derive(Clone, Debug)]
pub struct HumanTacticTermElabContext {
    env: Env,
    global_scope: HumanTacticGlobalScope,
    notation_entries: Vec<HumanResolvedNotationEntry>,
    signatures: BTreeMap<String, HumanCallableSignature>,
    local_context: Vec<MachineLocalDecl>,
    universe_params: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct HumanTacticTermElabContextRequest<'a> {
    pub direct_imports: &'a [VerifiedImport],
    pub available_imports: &'a [VerifiedImport],
    pub current_module: npa_cert::ModuleName,
    pub checked_current_decls: &'a [MachineCheckedCurrentDecl],
    pub current_generated_decls: &'a [MachineCheckedCurrentGeneratedDecl],
    pub local_context: Vec<MachineLocalDecl>,
    pub universe_params: Vec<String>,
    pub current_source_interface: Option<&'a HumanSourceInterface>,
    pub imported_source_interfaces: &'a [HumanImportedSourceInterface],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanTacticTermCheckOutput {
    pub expr: Expr,
    pub inferred_type: Expr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanTacticTermInferOutput {
    pub expr: Expr,
    pub inferred_type: Expr,
}

#[derive(Clone, Debug, Default)]
struct HumanTacticGlobalScope {
    current: Vec<HumanGlobalScopeEntry>,
    imported: Vec<HumanGlobalScopeEntry>,
}

pub fn elaborate_human_module(
    module_name: npa_cert::ModuleName,
    module: ResolvedHumanModule,
    verified_imports: &[VerifiedImport],
    options: &HumanCompileOptions,
) -> HumanResult<npa_cert::CoreModule> {
    let span = module.module.span;
    let plans = notation_candidate_plans(&module, options.max_notation_candidates)
        .map_err(|diagnostic| diagnostic.with_default_phase(HumanDiagnosticPhase::Elaborator))?;
    let mut first_error = None;
    let mut success = None;

    for plan in plans {
        match elaborate_human_module_with_notation_plan(
            module_name.clone(),
            &module,
            verified_imports,
            &plan,
        ) {
            Ok(core) if success.is_none() => success = Some(core),
            Ok(_) => {
                return Err(HumanDiagnostic::error(
                    HumanDiagnosticKind::AmbiguousNotation,
                    span,
                    "multiple notation candidates elaborated successfully",
                )
                .with_default_phase(HumanDiagnosticPhase::Elaborator));
            }
            Err(err) => {
                first_error.get_or_insert(err);
            }
        }
    }

    if let Some(core) = success {
        Ok(core)
    } else if let Some(err) = first_error {
        Err(err.with_default_phase(HumanDiagnosticPhase::Elaborator))
    } else {
        Err(HumanDiagnostic::error(
            HumanDiagnosticKind::AmbiguousNotation,
            span,
            "no notation candidate plan was available",
        )
        .with_default_phase(HumanDiagnosticPhase::Elaborator))
    }
}

fn elaborate_human_proof_start_core(
    module_name: npa_cert::ModuleName,
    theorem_name: npa_cert::Name,
    module: &ResolvedHumanModule,
    verified_imports: &[VerifiedImport],
    options: &HumanCompileOptions,
) -> HumanResult<HumanProofStartCore> {
    let span = module.module.span;
    let notation_use_count =
        human_proof_start_notation_use_count(&module_name, &theorem_name, module)?;
    let plans = notation_candidate_plans_for_count(
        module,
        options.max_notation_candidates,
        notation_use_count,
    )
    .map_err(|diagnostic| diagnostic.with_default_phase(HumanDiagnosticPhase::Elaborator))?;
    let mut first_error = None;
    let mut success = None;

    for plan in plans {
        match prepare_human_proof_start_core_with_notation_plan(
            module_name.clone(),
            &theorem_name,
            module,
            verified_imports,
            &plan,
        ) {
            Ok(core) if success.is_none() => success = Some(core),
            Ok(_) => {
                return Err(HumanDiagnostic::error(
                    HumanDiagnosticKind::AmbiguousNotation,
                    span,
                    "multiple notation candidates elaborated successfully",
                )
                .with_default_phase(HumanDiagnosticPhase::Elaborator));
            }
            Err(err) => {
                first_error.get_or_insert(err);
            }
        }
    }

    if let Some(core) = success {
        Ok(core)
    } else if let Some(err) = first_error {
        Err(err.with_default_phase(HumanDiagnosticPhase::Elaborator))
    } else {
        Err(HumanDiagnostic::error(
            HumanDiagnosticKind::AmbiguousNotation,
            span,
            "no notation candidate plan was available",
        )
        .with_default_phase(HumanDiagnosticPhase::Elaborator))
    }
}

fn elaborate_human_proof_start_core_with_by_proofs(
    module_name: npa_cert::ModuleName,
    theorem_name: npa_cert::Name,
    module: &ResolvedHumanModule,
    verified_imports: &[VerifiedImport],
    by_proofs: &BTreeMap<u64, Expr>,
    options: &HumanCompileOptions,
) -> HumanResult<HumanProofStartCore> {
    let span = module.module.span;
    let notation_use_count =
        human_proof_start_notation_use_count(&module_name, &theorem_name, module)?;
    let plans = notation_candidate_plans_for_count(
        module,
        options.max_notation_candidates,
        notation_use_count,
    )
    .map_err(|diagnostic| diagnostic.with_default_phase(HumanDiagnosticPhase::Elaborator))?;
    let mut first_error = None;
    let mut success = None;

    for plan in plans {
        match prepare_human_proof_start_core_with_notation_plan_and_by_proofs(
            module_name.clone(),
            &theorem_name,
            module,
            verified_imports,
            &plan,
            by_proofs,
        ) {
            Ok(core) if success.is_none() => success = Some(core),
            Ok(_) => {
                return Err(HumanDiagnostic::error(
                    HumanDiagnosticKind::AmbiguousNotation,
                    span,
                    "multiple notation candidates elaborated successfully",
                )
                .with_default_phase(HumanDiagnosticPhase::Elaborator));
            }
            Err(err) => {
                first_error.get_or_insert(err);
            }
        }
    }

    if let Some(core) = success {
        Ok(core)
    } else if let Some(err) = first_error {
        Err(err.with_default_phase(HumanDiagnosticPhase::Elaborator))
    } else {
        Err(HumanDiagnostic::error(
            HumanDiagnosticKind::AmbiguousNotation,
            span,
            "no notation candidate plan was available",
        )
        .with_default_phase(HumanDiagnosticPhase::Elaborator))
    }
}

fn prepare_human_proof_start_core_with_notation_plan(
    module_name: npa_cert::ModuleName,
    theorem_name: &npa_cert::Name,
    module: &ResolvedHumanModule,
    verified_imports: &[VerifiedImport],
    notation_plan: &[usize],
) -> HumanResult<HumanProofStartCore> {
    let mut lowering = HumanToMachineLowering::new(module, verified_imports, notation_plan)?
        .with_current_module_prefix(module_name.clone());
    let lowered = lowering.lower_proof_start(&module_name, theorem_name, module)?;
    let elaborator = HumanBidirectionalElaborator::new(module, verified_imports)?;
    let proof = elaborator.elaborate_proof_start_core(module_name.clone(), lowered)?;
    Ok(prefix_human_current_decl_identities_for_machine_bridge(
        &module_name,
        proof,
    ))
}

fn prepare_human_proof_start_core_with_notation_plan_and_by_proofs(
    module_name: npa_cert::ModuleName,
    theorem_name: &npa_cert::Name,
    module: &ResolvedHumanModule,
    verified_imports: &[VerifiedImport],
    notation_plan: &[usize],
    by_proofs: &BTreeMap<u64, Expr>,
) -> HumanResult<HumanProofStartCore> {
    let mut lowering = HumanToMachineLowering::new(module, verified_imports, notation_plan)?
        .with_current_module_prefix(module_name.clone());
    let lowered = lowering.lower_proof_start_with_core_proofs(
        &module_name,
        theorem_name,
        module,
        by_proofs,
    )?;
    let elaborator = HumanBidirectionalElaborator::new(module, verified_imports)?;
    let proof = elaborator.elaborate_proof_start_core(module_name.clone(), lowered)?;
    Ok(prefix_human_current_decl_identities_for_machine_bridge(
        &module_name,
        proof,
    ))
}

pub fn elaborate_human_tactic_term_check(
    context: &HumanTacticTermElabContext,
    term: &HumanExpr,
    expected: &Expr,
    options: &HumanCompileOptions,
) -> HumanResult<HumanTacticTermCheckOutput> {
    let resolved = resolve_human_tactic_term(context, term, options)?;
    let mut first_error = None;
    let mut success = None;

    for plan in notation_candidate_plans_from_uses(
        &resolved.resolved_notations,
        options.max_notation_candidates,
    )
    .map_err(|diagnostic| diagnostic.with_default_phase(HumanDiagnosticPhase::Elaborator))?
    {
        match elaborate_human_tactic_term_check_with_plan(context, &resolved, &plan, expected) {
            Ok(output) if success.is_none() => success = Some(output),
            Ok(_) => {
                return Err(HumanDiagnostic::error(
                    HumanDiagnosticKind::AmbiguousNotation,
                    term.span(),
                    "multiple Human tactic term notation candidates elaborated successfully",
                )
                .with_default_phase(HumanDiagnosticPhase::Elaborator));
            }
            Err(err) => {
                first_error.get_or_insert(err);
            }
        }
    }

    if let Some(output) = success {
        Ok(output)
    } else if let Some(err) = first_error {
        Err(err.with_default_phase(HumanDiagnosticPhase::Elaborator))
    } else {
        Err(HumanDiagnostic::error(
            HumanDiagnosticKind::AmbiguousNotation,
            term.span(),
            "no Human tactic term notation candidate plan was available",
        )
        .with_default_phase(HumanDiagnosticPhase::Elaborator))
    }
}

pub fn elaborate_human_tactic_term_infer(
    context: &HumanTacticTermElabContext,
    term: &HumanExpr,
    options: &HumanCompileOptions,
) -> HumanResult<HumanTacticTermInferOutput> {
    let resolved = resolve_human_tactic_term(context, term, options)?;
    let mut first_error = None;
    let mut success = None;

    for plan in notation_candidate_plans_from_uses(
        &resolved.resolved_notations,
        options.max_notation_candidates,
    )
    .map_err(|diagnostic| diagnostic.with_default_phase(HumanDiagnosticPhase::Elaborator))?
    {
        match elaborate_human_tactic_term_infer_with_plan(context, &resolved, &plan) {
            Ok(output) if success.is_none() => success = Some(output),
            Ok(_) => {
                return Err(HumanDiagnostic::error(
                    HumanDiagnosticKind::AmbiguousNotation,
                    term.span(),
                    "multiple Human tactic term notation candidates elaborated successfully",
                )
                .with_default_phase(HumanDiagnosticPhase::Elaborator));
            }
            Err(err) => {
                first_error.get_or_insert(err);
            }
        }
    }

    if let Some(output) = success {
        Ok(output)
    } else if let Some(err) = first_error {
        Err(err.with_default_phase(HumanDiagnosticPhase::Elaborator))
    } else {
        Err(HumanDiagnostic::error(
            HumanDiagnosticKind::AmbiguousNotation,
            term.span(),
            "no Human tactic term notation candidate plan was available",
        )
        .with_default_phase(HumanDiagnosticPhase::Elaborator))
    }
}

pub fn compile_human_source_to_core(
    file_id: crate::FileId,
    module_name: npa_cert::ModuleName,
    source: &str,
    verified_imports: &[VerifiedImport],
    options: &HumanCompileOptions,
) -> HumanResult<npa_cert::CoreModule> {
    compile_human_source_to_core_with_source_interfaces(
        file_id,
        module_name,
        source,
        verified_imports,
        &[],
        options,
    )
}

pub fn compile_human_source_to_core_with_source_interfaces(
    file_id: crate::FileId,
    module_name: npa_cert::ModuleName,
    source: &str,
    verified_imports: &[VerifiedImport],
    imported_source_interfaces: &[HumanImportedSourceInterface],
    options: &HumanCompileOptions,
) -> HumanResult<npa_cert::CoreModule> {
    compile_human_source_to_core_output_with_source_interfaces(
        file_id,
        module_name,
        source,
        verified_imports,
        imported_source_interfaces,
        options,
    )
    .map(|output| output.core_module)
}

pub fn compile_human_source_to_core_output_with_source_interfaces(
    file_id: crate::FileId,
    module_name: npa_cert::ModuleName,
    source: &str,
    verified_imports: &[VerifiedImport],
    imported_source_interfaces: &[HumanImportedSourceInterface],
    options: &HumanCompileOptions,
) -> HumanResult<HumanCoreCompileOutput> {
    let module =
        parse_human_module_with_source_interfaces(file_id, source, imported_source_interfaces)?;
    let resolved = resolve_human_module_with_source_interfaces(
        module_name.clone(),
        module,
        verified_imports,
        imported_source_interfaces,
        options,
    )?;
    let source_interface = resolved.state.source_interfaces.current.clone();
    let core_module = elaborate_human_module(module_name, resolved, verified_imports, options)
        .map_err(|diagnostic| diagnostic.with_default_phase(HumanDiagnosticPhase::Elaborator))?;
    Ok(HumanCoreCompileOutput {
        core_module,
        source_interface,
    })
}

pub fn collect_human_by_proof_targets_with_source_interfaces(
    file_id: crate::FileId,
    module_name: npa_cert::ModuleName,
    source: &str,
    verified_imports: &[VerifiedImport],
    imported_source_interfaces: &[HumanImportedSourceInterface],
    options: &HumanCompileOptions,
) -> HumanResult<HumanByProofTargetsOutput> {
    let parsed =
        parse_human_module_with_source_interfaces(file_id, source, imported_source_interfaces)?;
    let resolved = resolve_human_module_with_source_interfaces(
        module_name.clone(),
        parsed,
        verified_imports,
        imported_source_interfaces,
        options,
    )?;
    let source_interface = resolved.state.source_interfaces.current.clone();
    let active_imports = resolved.state.source_interfaces.imports.clone();
    let targets = human_by_proof_targets(&module_name, &resolved)?;

    Ok(HumanByProofTargetsOutput {
        targets,
        source_interface,
        active_imports,
    })
}

pub fn prepare_human_proof_start_core_with_source_interfaces_and_by_proofs(
    request: HumanProofStartCoreWithProofsRequest<'_>,
) -> HumanResult<HumanProofStartCoreOutput> {
    let parsed = parse_human_module_with_source_interfaces(
        request.file_id,
        request.source,
        request.imported_source_interfaces,
    )?;
    let resolved = resolve_human_module_with_source_interfaces(
        request.module_name.clone(),
        parsed,
        request.verified_imports,
        request.imported_source_interfaces,
        request.options,
    )?;
    let source_interface = resolved.state.source_interfaces.current.clone();
    let active_imports = resolved.state.source_interfaces.imports.clone();
    let by_proofs = by_proof_map(request.prior_by_proofs, resolved.module.span)?;
    let by_targets = human_by_proof_targets(&request.module_name, &resolved)?;
    if let Some(target) = by_targets
        .iter()
        .find(|target| target.theorem_name == request.theorem_name)
    {
        let expected_prior = by_targets
            .iter()
            .filter(|prior| prior.source_index < target.source_index)
            .map(|prior| prior.source_index)
            .collect::<BTreeSet<_>>();
        validate_by_proof_map_indices(&by_proofs, &expected_prior, resolved.module.span)?;
    }
    let proof = elaborate_human_proof_start_core_with_by_proofs(
        request.module_name,
        request.theorem_name,
        &resolved,
        request.verified_imports,
        &by_proofs,
        request.options,
    )
    .map_err(|diagnostic| diagnostic.with_default_phase(HumanDiagnosticPhase::Elaborator))?;

    Ok(HumanProofStartCoreOutput {
        proof,
        source_interface,
        active_imports,
    })
}

pub fn compile_human_source_to_core_output_with_source_interfaces_and_by_proofs(
    file_id: crate::FileId,
    module_name: npa_cert::ModuleName,
    source: &str,
    verified_imports: &[VerifiedImport],
    imported_source_interfaces: &[HumanImportedSourceInterface],
    by_proofs: &[HumanByProofCore],
    options: &HumanCompileOptions,
) -> HumanResult<HumanCoreCompileOutput> {
    let module =
        parse_human_module_with_source_interfaces(file_id, source, imported_source_interfaces)?;
    let resolved = resolve_human_module_with_source_interfaces(
        module_name.clone(),
        module,
        verified_imports,
        imported_source_interfaces,
        options,
    )?;
    let source_interface = resolved.state.source_interfaces.current.clone();
    let by_proofs = by_proof_map(by_proofs, resolved.module.span)?;
    let expected_by_proofs = human_by_proof_targets(&module_name, &resolved)?
        .into_iter()
        .map(|target| target.source_index)
        .collect::<BTreeSet<_>>();
    validate_by_proof_map_indices(&by_proofs, &expected_by_proofs, resolved.module.span)?;
    let core_module = elaborate_human_module_with_by_proofs(
        module_name,
        resolved,
        verified_imports,
        &by_proofs,
        options,
    )
    .map_err(|diagnostic| diagnostic.with_default_phase(HumanDiagnosticPhase::Elaborator))?;

    Ok(HumanCoreCompileOutput {
        core_module,
        source_interface,
    })
}

pub fn prepare_human_proof_start_core_with_source_interfaces(
    file_id: crate::FileId,
    module_name: npa_cert::ModuleName,
    theorem_name: npa_cert::Name,
    source: &str,
    verified_imports: &[VerifiedImport],
    imported_source_interfaces: &[HumanImportedSourceInterface],
    options: &HumanCompileOptions,
) -> HumanResult<HumanProofStartCoreOutput> {
    let parsed =
        parse_human_module_with_source_interfaces(file_id, source, imported_source_interfaces)?;
    let resolved = resolve_human_module_with_source_interfaces(
        module_name.clone(),
        parsed,
        verified_imports,
        imported_source_interfaces,
        options,
    )?;
    let source_interface = resolved.state.source_interfaces.current.clone();
    let active_imports = resolved.state.source_interfaces.imports.clone();
    let proof = elaborate_human_proof_start_core(
        module_name,
        theorem_name,
        &resolved,
        verified_imports,
        options,
    )
    .map_err(|diagnostic| diagnostic.with_default_phase(HumanDiagnosticPhase::Elaborator))?;

    Ok(HumanProofStartCoreOutput {
        proof,
        source_interface,
        active_imports,
    })
}

pub fn certificate_imports_for_human_core_module(
    core: &npa_cert::CoreModule,
    active_imports: &[HumanImportedSourceInterface],
    verified_modules: &[npa_cert::VerifiedModule],
    file_id: crate::FileId,
) -> HumanResult<Vec<npa_cert::VerifiedModule>> {
    let active_import_indices = active_human_import_indices_from_source_interfaces(
        active_imports,
        verified_modules,
        file_id,
    )?;
    certificate_imports_for_module(core, &active_import_indices, verified_modules, file_id).map_err(
        |diagnostic| {
            human_certificate_import_diagnostic(Span::empty(file_id), diagnostic)
                .with_phase(HumanDiagnosticPhase::CertificateHandoff)
        },
    )
}

pub fn compile_human_source_to_certificate(
    file_id: crate::FileId,
    module_name: npa_cert::ModuleName,
    source: &str,
    verified_modules: &[npa_cert::VerifiedModule],
    options: &HumanCompileOptions,
) -> HumanResult<npa_cert::ModuleCert> {
    compile_human_source_to_certificate_with_source_interfaces(
        file_id,
        module_name,
        source,
        verified_modules,
        &[],
        options,
    )
}

pub fn compile_human_source_to_certificate_with_source_interfaces(
    file_id: crate::FileId,
    module_name: npa_cert::ModuleName,
    source: &str,
    verified_modules: &[npa_cert::VerifiedModule],
    imported_source_interfaces: &[HumanImportedSourceInterface],
    options: &HumanCompileOptions,
) -> HumanResult<npa_cert::ModuleCert> {
    compile_human_source_to_certificate_output_with_source_interfaces(
        file_id,
        module_name,
        source,
        verified_modules,
        imported_source_interfaces,
        options,
    )
    .map(|output| output.certificate)
}

pub fn compile_human_source_to_certificate_output_with_source_interfaces(
    file_id: crate::FileId,
    module_name: npa_cert::ModuleName,
    source: &str,
    verified_modules: &[npa_cert::VerifiedModule],
    imported_source_interfaces: &[HumanImportedSourceInterface],
    options: &HumanCompileOptions,
) -> HumanResult<HumanCertificateCompileOutput> {
    let verified_imports: Vec<_> = verified_modules.iter().map(VerifiedImport::from).collect();
    let parsed =
        parse_human_module_with_source_interfaces(file_id, source, imported_source_interfaces)?;
    let resolved = resolve_human_module_with_source_interfaces(
        module_name.clone(),
        parsed,
        &verified_imports,
        imported_source_interfaces,
        options,
    )?;
    let source_interface = resolved.state.source_interfaces.current.clone();
    let active_import_indices = active_human_import_indices(&resolved, &verified_imports)
        .map_err(|diagnostic| diagnostic.with_default_phase(HumanDiagnosticPhase::Resolver))?;
    let core = elaborate_human_module(module_name, resolved, &verified_imports, options)
        .map_err(|diagnostic| diagnostic.with_default_phase(HumanDiagnosticPhase::Elaborator))?;
    let certificate_imports =
        certificate_imports_for_module(&core, &active_import_indices, verified_modules, file_id)
            .map_err(|err| {
                human_certificate_import_diagnostic(source_span(file_id, source), err)
                    .with_phase(HumanDiagnosticPhase::CertificateHandoff)
            })?;
    let cert = npa_cert::build_module_cert(core, &certificate_imports).map_err(|err| {
        HumanDiagnostic::error(
            HumanDiagnosticKind::KernelRejected,
            source_span(file_id, source),
            format!("certificate certificate handoff rejected Human source: {err:?}"),
        )
        .with_phase(HumanDiagnosticPhase::CertificateHandoff)
    })?;
    let bytes = npa_cert::encode_module_cert(&cert).map_err(|err| {
        HumanDiagnostic::error(
            HumanDiagnosticKind::KernelRejected,
            source_span(file_id, source),
            format!("certificate certificate encoding rejected Human source: {err:?}"),
        )
        .with_phase(HumanDiagnosticPhase::CertificateHandoff)
    })?;
    let mut session = npa_cert::VerifierSession::new();
    for import in &certificate_imports {
        session.register_verified_module(import.clone());
    }
    npa_cert::verify_module_cert(&bytes, &mut session, &npa_cert::AxiomPolicy::normal()).map_err(
        |err| {
            HumanDiagnostic::error(
                HumanDiagnosticKind::KernelRejected,
                source_span(file_id, source),
                format!("certificate certificate verification rejected Human source: {err:?}"),
            )
            .with_phase(HumanDiagnosticPhase::CertificateHandoff)
        },
    )?;
    let source_interface = source_interface_with_certificate_hashes(source_interface, &cert);
    Ok(HumanCertificateCompileOutput {
        certificate: cert,
        source_interface,
    })
}

fn source_span(file_id: crate::FileId, source: &str) -> Span {
    Span::new(file_id, 0, source.len() as u32)
}

fn source_interface_with_certificate_hashes(
    mut source_interface: HumanSourceInterface,
    cert: &npa_cert::ModuleCert,
) -> HumanSourceInterface {
    let module_name = source_interface.module.clone();
    let export_hashes: BTreeMap<_, _> = cert
        .export_block
        .iter()
        .map(|entry| {
            (
                cert.name_table[entry.name].clone(),
                entry.decl_interface_hash,
            )
        })
        .collect();

    for decl in &mut source_interface.declarations {
        let name = npa_cert::Name(decl.name.parts.clone());
        if let Some(hash) = export_hashes
            .get(&name)
            .or_else(|| export_hashes.get(&prefixed_human_current_name(&module_name, &name)))
        {
            decl.decl_interface_hash = Some(*hash);
        }
    }

    for generated in &mut source_interface.generated_declarations {
        let name = npa_cert::Name(generated.name.parts.clone());
        if let Some(hash) = export_hashes
            .get(&name)
            .or_else(|| export_hashes.get(&prefixed_human_current_name(&module_name, &name)))
        {
            generated.decl_interface_hash = Some(*hash);
        }
    }

    source_interface
}

fn human_current_name_matches_target(
    module_name: &npa_cert::ModuleName,
    current_name: &HumanName,
    target: &npa_cert::Name,
) -> bool {
    let current = npa_cert::Name(current_name.parts.clone());
    &current == target || prefixed_human_current_name(module_name, &current) == *target
}

fn prefixed_human_current_name(
    module_name: &npa_cert::ModuleName,
    name: &npa_cert::Name,
) -> npa_cert::Name {
    if name_has_module_prefix(name, module_name) {
        return name.clone();
    }

    let mut parts = module_name.0.clone();
    parts.extend(name.0.iter().cloned());
    npa_cert::Name(parts)
}

fn name_has_module_prefix(name: &npa_cert::Name, module_name: &npa_cert::ModuleName) -> bool {
    name.0.len() > module_name.0.len() && name.0.starts_with(&module_name.0)
}

fn prefix_human_current_decl_identities_for_machine_bridge(
    module_name: &npa_cert::ModuleName,
    mut proof: HumanProofStartCore,
) -> HumanProofStartCore {
    proof.theorem_name = prefixed_human_current_name(module_name, &proof.theorem_name);
    proof.prior_declarations = proof
        .prior_declarations
        .into_iter()
        .map(|decl| prefix_current_decl_identity(module_name, decl))
        .collect();
    proof
}

fn prefixed_current_name_string(module_name: &npa_cert::ModuleName, name: String) -> String {
    prefixed_human_current_name(module_name, &npa_cert::Name::from_dotted(name)).as_dotted()
}

fn prefix_current_decl_identity(module_name: &npa_cert::ModuleName, decl: Decl) -> Decl {
    match decl {
        Decl::Def {
            name,
            universe_params,
            ty,
            value,
            reducibility,
        } => Decl::Def {
            name: prefixed_current_name_string(module_name, name),
            universe_params,
            ty,
            value,
            reducibility,
        },
        Decl::DefConstrained {
            name,
            universe_params,
            universe_constraints,
            ty,
            value,
            reducibility,
        } => Decl::DefConstrained {
            name: prefixed_current_name_string(module_name, name),
            universe_params,
            universe_constraints,
            ty,
            value,
            reducibility,
        },
        Decl::Theorem {
            name,
            universe_params,
            ty,
            proof,
        } => Decl::Theorem {
            name: prefixed_current_name_string(module_name, name),
            universe_params,
            ty,
            proof,
        },
        Decl::TheoremConstrained {
            name,
            universe_params,
            universe_constraints,
            ty,
            proof,
        } => Decl::TheoremConstrained {
            name: prefixed_current_name_string(module_name, name),
            universe_params,
            universe_constraints,
            ty,
            proof,
        },
        Decl::Axiom {
            name,
            universe_params,
            ty,
        } => Decl::Axiom {
            name: prefixed_current_name_string(module_name, name),
            universe_params,
            ty,
        },
        Decl::AxiomConstrained {
            name,
            universe_params,
            universe_constraints,
            ty,
        } => Decl::AxiomConstrained {
            name: prefixed_current_name_string(module_name, name),
            universe_params,
            universe_constraints,
            ty,
        },
        Decl::Inductive {
            name,
            universe_params,
            ty,
            data,
        } => Decl::Inductive {
            name: prefixed_current_name_string(module_name, name),
            universe_params,
            ty,
            data: Box::new(prefix_current_inductive_identities(module_name, *data)),
        },
        Decl::Constructor {
            name,
            universe_params,
            ty,
            inductive,
        } => Decl::Constructor {
            name: prefixed_current_name_string(module_name, name),
            universe_params,
            ty,
            inductive: prefixed_current_name_string(module_name, inductive),
        },
        Decl::Recursor {
            name,
            universe_params,
            ty,
            inductive,
            rules,
        } => Decl::Recursor {
            name: prefixed_current_name_string(module_name, name),
            universe_params,
            ty,
            inductive: prefixed_current_name_string(module_name, inductive),
            rules,
        },
    }
}

fn prefix_current_inductive_identities(
    module_name: &npa_cert::ModuleName,
    mut data: InductiveDecl,
) -> InductiveDecl {
    data.name = prefixed_current_name_string(module_name, data.name);
    data.constructors = data
        .constructors
        .into_iter()
        .map(|constructor| ConstructorDecl {
            name: prefixed_current_name_string(module_name, constructor.name),
            ty: constructor.ty,
        })
        .collect();
    data.recursor = data.recursor.map(|recursor| RecursorDecl {
        name: prefixed_current_name_string(module_name, recursor.name),
        universe_params: recursor.universe_params,
        ty: recursor.ty,
        rules: recursor.rules,
    });
    data
}

impl HumanTacticTermElabContext {
    pub fn from_request(request: HumanTacticTermElabContextRequest<'_>) -> HumanResult<Self> {
        let span = Span::empty(crate::FileId(0));
        let mut env = Env::new();
        let mut seen_imports = BTreeSet::new();
        let mut imports = Vec::new();
        for import in request
            .direct_imports
            .iter()
            .chain(request.available_imports)
        {
            let key = (
                import.module.clone(),
                import.export_hash,
                import.certificate_hash,
            );
            if seen_imports.insert(key) {
                imports.push(import);
            }
        }
        add_human_kernel_imports_to_env(&mut env, &imports, span)?;

        for decl in request.checked_current_decls {
            add_human_kernel_decl_to_env(
                &mut env,
                decl.decl.clone(),
                span,
                "Human tactic checked current declaration",
            )?;
        }

        let global_scope = human_tactic_global_scope(request.direct_imports, &request);
        let signatures = human_tactic_callable_signatures(&request);
        let notation_entries = human_tactic_notation_entries(
            &global_scope,
            request.current_source_interface,
            request.imported_source_interfaces,
            span,
        )?;

        Ok(Self {
            env,
            global_scope,
            notation_entries,
            signatures,
            local_context: request.local_context,
            universe_params: request.universe_params,
        })
    }

    pub fn local_context(&self) -> &[MachineLocalDecl] {
        &self.local_context
    }

    pub fn universe_params(&self) -> &[String] {
        &self.universe_params
    }
}

fn add_human_kernel_imports_to_env(
    env: &mut Env,
    imports: &[&VerifiedImport],
    span: Span,
) -> HumanResult<()> {
    let mut pending = imports
        .iter()
        .flat_map(|import| kernel_decls_for_human_import(import))
        .collect::<Vec<_>>();

    while !pending.is_empty() {
        let pending_names = pending
            .iter()
            .map(|decl| decl.name().to_owned())
            .collect::<BTreeSet<_>>();
        let mut next = Vec::new();
        let mut progressed = false;

        for decl in pending {
            if human_decl_waits_for_pending_import(env, &decl, &pending_names) {
                next.push(decl);
                continue;
            }
            add_human_kernel_decl_to_env(env, decl, span, "Human tactic import environment")?;
            progressed = true;
        }

        if !progressed {
            if next.is_empty() {
                return Ok(());
            }
            let names = next
                .iter()
                .map(|decl| decl.name().to_owned())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::KernelRejected,
                span,
                format!("Human tactic import environment has cyclic or unresolved dependencies: {names}"),
            )
            .with_phase(HumanDiagnosticPhase::KernelHandoff));
        }
        pending = next;
    }

    Ok(())
}

fn human_decl_waits_for_pending_import(
    env: &Env,
    decl: &Decl,
    pending_names: &BTreeSet<String>,
) -> bool {
    let mut references = BTreeSet::new();
    collect_const_names_from_human_decl(&mut references, decl);
    remove_human_decl_owned_const_names(&mut references, decl);
    references.into_iter().any(|name| {
        let dotted = name.as_dotted();
        pending_import_decl_covers_reference(&dotted, pending_names) && env.decl(&dotted).is_none()
    })
}

fn pending_import_decl_covers_reference(reference: &str, pending_names: &BTreeSet<String>) -> bool {
    pending_names.contains(reference)
        || reference
            .rsplit_once('.')
            .is_some_and(|(parent, _)| pending_names.contains(parent))
}

fn add_human_kernel_decl_to_env(
    env: &mut Env,
    decl: Decl,
    span: Span,
    context: &str,
) -> HumanResult<()> {
    if let Some(existing) = env.decl(decl.name()) {
        if existing == &decl {
            return Ok(());
        }
        return Err(HumanDiagnostic::error(
            HumanDiagnosticKind::KernelRejected,
            span,
            format!(
                "kernel declaration {} conflicts with an existing declaration",
                decl.name()
            ),
        )
        .with_phase(HumanDiagnosticPhase::KernelHandoff));
    }

    add_referenced_builtin_decls_to_human_env(env, &decl, span, context)?;

    match decl {
        Decl::Axiom {
            name,
            universe_params,
            ty,
        } => env.add_axiom(name, universe_params, ty),
        Decl::AxiomConstrained {
            name,
            universe_params,
            universe_constraints,
            ty,
        } => {
            env.add_axiom_with_universe_constraints(name, universe_params, universe_constraints, ty)
        }
        Decl::Def {
            name,
            universe_params,
            ty,
            value,
            reducibility,
        } => env.add_def(name, universe_params, ty, value, reducibility),
        Decl::DefConstrained {
            name,
            universe_params,
            universe_constraints,
            ty,
            value,
            reducibility,
        } => env.add_def_with_universe_constraints(
            name,
            universe_params,
            universe_constraints,
            ty,
            value,
            reducibility,
        ),
        Decl::Theorem {
            name,
            universe_params,
            ty,
            proof,
        } => env.add_theorem(name, universe_params, ty, proof),
        Decl::TheoremConstrained {
            name,
            universe_params,
            universe_constraints,
            ty,
            proof,
        } => env.add_theorem_with_universe_constraints(
            name,
            universe_params,
            universe_constraints,
            ty,
            proof,
        ),
        Decl::Inductive { data, .. } => env.add_inductive(*data),
        Decl::Constructor { .. } | Decl::Recursor { .. } => Ok(()),
    }
    .map_err(|err| {
        human_kernel_decl_diagnostic(span, err, context)
            .with_phase(HumanDiagnosticPhase::KernelHandoff)
    })
}

fn human_tactic_global_scope(
    direct_imports: &[VerifiedImport],
    request: &HumanTacticTermElabContextRequest<'_>,
) -> HumanTacticGlobalScope {
    let span = Span::empty(crate::FileId(0));
    let imported = direct_imports
        .iter()
        .flat_map(|import| {
            import
                .exports
                .iter()
                .map(move |export| HumanGlobalScopeEntry {
                    name: HumanName::new(export.name.0.clone(), span),
                    reference: HumanGlobalRef::Imported {
                        module: import.module.clone(),
                        name: export.name.clone(),
                        decl_interface_hash: export.decl_interface_hash,
                    },
                    span,
                })
        })
        .collect();
    let mut current = request
        .checked_current_decls
        .iter()
        .map(|decl| HumanGlobalScopeEntry {
            name: HumanName::new(
                prefixed_human_current_name(&request.current_module, &decl.name).0,
                span,
            ),
            reference: HumanGlobalRef::Local {
                index: decl.source_index as usize,
                name: prefixed_human_current_name(&request.current_module, &decl.name),
            },
            span,
        })
        .collect::<Vec<_>>();
    current.extend(
        request
            .current_generated_decls
            .iter()
            .map(|decl| HumanGlobalScopeEntry {
                name: HumanName::new(
                    prefixed_human_current_name(&request.current_module, &decl.name).0,
                    span,
                ),
                reference: HumanGlobalRef::LocalGenerated {
                    index: decl.parent_source_index as usize,
                    name: prefixed_human_current_name(&request.current_module, &decl.name),
                },
                span,
            }),
    );
    HumanTacticGlobalScope { current, imported }
}

fn human_tactic_callable_signatures(
    request: &HumanTacticTermElabContextRequest<'_>,
) -> BTreeMap<String, HumanCallableSignature> {
    let mut signatures = BTreeMap::new();
    for import in request.direct_imports {
        for export in &import.exports {
            let implicit_profile = human_imported_source_interface_profile(
                request.imported_source_interfaces,
                import,
                export,
            )
            .unwrap_or_else(|| {
                if npa_cert::builtin_decl_interface_hash(&export.name)
                    == Some(export.decl_interface_hash)
                {
                    builtin_machine_callable_profile(&export.name).unwrap_or_default()
                } else {
                    Vec::new()
                }
            });
            signatures.insert(
                export.name.as_dotted(),
                HumanCallableSignature {
                    universe_params: export.universe_params.clone(),
                    implicit_profile,
                },
            );
        }
    }

    for decl in request.checked_current_decls {
        let implicit_profile = human_current_source_interface_profile(
            request.current_source_interface,
            &decl.name,
            decl.decl_interface_hash,
        )
        .unwrap_or_default();
        signatures.insert(
            prefixed_human_current_name(&request.current_module, &decl.name).as_dotted(),
            HumanCallableSignature {
                universe_params: decl.decl.universe_params().to_vec(),
                implicit_profile: implicit_profile.clone(),
            },
        );
        if let Decl::Inductive { data, .. } = &decl.decl {
            for constructor in &data.constructors {
                signatures.insert(
                    constructor.name.clone(),
                    HumanCallableSignature {
                        universe_params: data.universe_params.clone(),
                        implicit_profile: generated_constructor_profile(
                            &constructor.ty,
                            &implicit_profile,
                        ),
                    },
                );
            }
            if let Some(recursor) = &data.recursor {
                signatures.insert(
                    recursor.name.clone(),
                    HumanCallableSignature {
                        universe_params: recursor.universe_params.clone(),
                        implicit_profile: all_explicit_profile(pi_domain_count(&recursor.ty)),
                    },
                );
            }
        }
    }

    signatures
}

fn human_imported_source_interface_profile(
    imported_source_interfaces: &[HumanImportedSourceInterface],
    import: &VerifiedImport,
    export: &crate::VerifiedExport,
) -> Option<Vec<MachineCallableBinderVisibility>> {
    human_source_interface_profile_for_export(imported_source_interfaces, import, export)
}

fn human_source_interface_profile_for_export(
    imported_source_interfaces: &[HumanImportedSourceInterface],
    import: &VerifiedImport,
    export: &crate::VerifiedExport,
) -> Option<Vec<MachineCallableBinderVisibility>> {
    for interface in imported_source_interfaces.iter().filter(|interface| {
        interface.module == import.module
            && interface.export_hash == import.export_hash
            && interface.certificate_hash == import.certificate_hash
    }) {
        if let Some(decl) = interface.source_interface.declarations.iter().find(|decl| {
            decl.kind != HumanSourceDeclarationKind::Imported
                && npa_cert::Name(decl.name.parts.clone()) == export.name
                && decl.decl_interface_hash == Some(export.decl_interface_hash)
        }) {
            return Some(machine_callable_profile_from_human_binders(&decl.binders));
        }

        let Some(generated) = interface
            .source_interface
            .generated_declarations
            .iter()
            .find(|decl| {
                npa_cert::Name(decl.name.parts.clone()) == export.name
                    && decl.decl_interface_hash == Some(export.decl_interface_hash)
            })
        else {
            continue;
        };
        match generated.kind {
            HumanGeneratedDeclarationKind::Constructor => {
                let parent_profile = interface
                    .source_interface
                    .declarations
                    .iter()
                    .find(|decl| {
                        decl.kind == HumanSourceDeclarationKind::Inductive
                            && decl.name.parts == generated.parent.parts
                    })
                    .map(|decl| machine_callable_profile_from_human_binders(&decl.binders))
                    .unwrap_or_default();
                return Some(generated_constructor_profile(&export.ty, &parent_profile));
            }
            HumanGeneratedDeclarationKind::Recursor => {
                return Some(all_explicit_profile(pi_domain_count(&export.ty)));
            }
        }
    }
    None
}

fn human_current_source_interface_profile(
    current_source_interface: Option<&HumanSourceInterface>,
    name: &npa_cert::Name,
    decl_interface_hash: npa_cert::Hash,
) -> Option<Vec<MachineCallableBinderVisibility>> {
    current_source_interface?
        .declarations
        .iter()
        .find(|decl| {
            npa_cert::Name(decl.name.parts.clone()) == *name
                && decl.decl_interface_hash == Some(decl_interface_hash)
        })
        .map(|decl| machine_callable_profile_from_human_binders(&decl.binders))
}

fn human_tactic_notation_entries(
    scope: &HumanTacticGlobalScope,
    current_source_interface: Option<&HumanSourceInterface>,
    imported_source_interfaces: &[HumanImportedSourceInterface],
    span: Span,
) -> HumanResult<Vec<HumanResolvedNotationEntry>> {
    let mut entries = Vec::new();
    if let Some(source_interface) = current_source_interface {
        append_human_tactic_notation_entries(scope, &mut entries, &source_interface.notations)?;
    }
    for source_interface in imported_source_interfaces {
        append_human_tactic_notation_entries(
            scope,
            &mut entries,
            &source_interface.source_interface.notations,
        )?;
    }
    let _ = span;
    Ok(entries)
}

fn append_human_tactic_notation_entries(
    scope: &HumanTacticGlobalScope,
    entries: &mut Vec<HumanResolvedNotationEntry>,
    notations: &[crate::HumanSourceNotationMetadata],
) -> HumanResult<()> {
    for notation in notations {
        let Ok(Some(target)) = resolve_human_tactic_global_name(scope, &notation.target) else {
            continue;
        };
        entries.push(HumanResolvedNotationEntry {
            kind: notation.kind,
            associativity: notation.associativity,
            precedence: notation.precedence,
            token: notation.token.clone(),
            target,
            namespace: notation.namespace.clone(),
            span: notation.span,
        });
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct HumanResolvedTacticTerm<'a> {
    term: &'a HumanExpr,
    resolved_names: Vec<HumanResolvedNameUse>,
    resolved_notations: Vec<HumanResolvedNotationUse>,
}

#[derive(Clone, Debug, Default)]
struct HumanTacticLocalScope {
    names: Vec<HumanName>,
}

impl HumanTacticLocalScope {
    fn from_machine_locals(locals: &[MachineLocalDecl]) -> Self {
        let span = Span::empty(crate::FileId(0));
        Self {
            names: locals
                .iter()
                .map(|local| HumanName::new(vec![local.name.clone()], span))
                .collect(),
        }
    }

    fn push(&mut self, name: HumanName) {
        self.names.push(name);
    }

    fn lookup(&self, name: &str) -> Option<(HumanName, usize)> {
        self.names
            .iter()
            .rev()
            .enumerate()
            .find(|(_, local)| local.parts.len() == 1 && local.parts[0] == name)
            .map(|(index, local)| (local.clone(), index))
    }
}

struct HumanTacticTermResolver<'a> {
    context: &'a HumanTacticTermElabContext,
    max_notation_candidates: usize,
    resolved_names: Vec<HumanResolvedNameUse>,
    resolved_notations: Vec<HumanResolvedNotationUse>,
}

fn resolve_human_tactic_term<'a>(
    context: &HumanTacticTermElabContext,
    term: &'a HumanExpr,
    options: &HumanCompileOptions,
) -> HumanResult<HumanResolvedTacticTerm<'a>> {
    let mut resolver = HumanTacticTermResolver {
        context,
        max_notation_candidates: options.max_notation_candidates,
        resolved_names: Vec::new(),
        resolved_notations: Vec::new(),
    };
    let mut locals = HumanTacticLocalScope::from_machine_locals(&context.local_context);
    resolver.resolve_expr(term, &mut locals)?;
    Ok(HumanResolvedTacticTerm {
        term,
        resolved_names: resolver.resolved_names,
        resolved_notations: resolver.resolved_notations,
    })
}

impl HumanTacticTermResolver<'_> {
    fn resolve_expr(
        &mut self,
        expr: &HumanExpr,
        locals: &mut HumanTacticLocalScope,
    ) -> HumanResult<()> {
        match expr {
            HumanExpr::Ident { name, span, .. } => {
                let resolved = self.resolve_name(name, locals, *span)?;
                self.resolved_names.push(HumanResolvedNameUse {
                    source: name.clone(),
                    resolved,
                });
            }
            HumanExpr::Sort { .. } | HumanExpr::Hole { .. } => {}
            HumanExpr::App { func, arg, .. } => {
                self.resolve_expr(func, locals)?;
                self.resolve_expr(arg, locals)?;
            }
            HumanExpr::Lam { binders, body, .. } | HumanExpr::Pi { binders, body, .. } => {
                let mut nested = locals.clone();
                self.resolve_binders(binders, &mut nested)?;
                self.resolve_expr(body, &mut nested)?;
            }
            HumanExpr::Let {
                name,
                ty,
                value,
                body,
                ..
            } => {
                if let Some(ty) = ty {
                    self.resolve_expr(ty, locals)?;
                }
                self.resolve_expr(value, locals)?;
                let mut nested = locals.clone();
                nested.push(name.clone());
                self.resolve_expr(body, &mut nested)?;
            }
            HumanExpr::Annot { expr, ty, .. } => {
                self.resolve_expr(expr, locals)?;
                self.resolve_expr(ty, locals)?;
            }
            HumanExpr::Arrow {
                domain, codomain, ..
            } => {
                self.resolve_expr(domain, locals)?;
                self.resolve_expr(codomain, locals)?;
            }
            HumanExpr::NotationApp { head, args, .. } => {
                for arg in args {
                    self.resolve_expr(arg, locals)?;
                }
                let candidates = self.resolve_notation_candidates(head)?;
                self.resolved_notations.push(HumanResolvedNotationUse {
                    head: head.clone(),
                    candidates,
                });
            }
        }

        Ok(())
    }

    fn resolve_binders(
        &mut self,
        binders: &[HumanBinder],
        locals: &mut HumanTacticLocalScope,
    ) -> HumanResult<()> {
        let mut index = 0;
        while index < binders.len() {
            let group_end = human_binder_group_end(binders, index);
            for binder in &binders[index..group_end] {
                if let Some(ty) = &binder.ty {
                    self.resolve_expr(ty, locals)?;
                }
            }
            for binder in &binders[index..group_end] {
                if let HumanBinderKind::Named(name) = &binder.kind {
                    locals.push(name.clone());
                }
            }
            index = group_end;
        }

        Ok(())
    }

    fn resolve_name(
        &self,
        name: &HumanName,
        locals: &HumanTacticLocalScope,
        span: Span,
    ) -> HumanResult<HumanResolvedName> {
        if name.parts.len() == 1 {
            if let Some((local_name, de_bruijn_index)) = locals.lookup(&name.parts[0]) {
                return Ok(HumanResolvedName::Local {
                    name: local_name,
                    de_bruijn_index,
                });
            }
        }

        if let Some(resolved) = resolve_human_tactic_global_name(&self.context.global_scope, name)?
        {
            return Ok(HumanResolvedName::Global(resolved));
        }

        Err(HumanDiagnostic::error(
            HumanDiagnosticKind::UnknownIdentifier,
            span,
            format!("unknown identifier {}", name.as_dotted()),
        ))
    }

    fn resolve_notation_candidates(
        &self,
        head: &crate::HumanNotationHead,
    ) -> HumanResult<Vec<HumanGlobalRef>> {
        let mut candidates = BTreeMap::new();
        for entry in self.context.notation_entries.iter().filter(|entry| {
            entry.token == head.token
                && entry.kind == head.kind
                && entry.precedence == head.precedence
                && entry.associativity == head.associativity
        }) {
            candidates.insert(
                human_tactic_global_ref_sort_key(&entry.target),
                entry.target.clone(),
            );
        }

        if candidates.len() > self.max_notation_candidates {
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::TooManyNotationCandidates,
                head.span,
                format!("notation {} has too many candidates", head.token),
            )
            .with_payload(human_tactic_candidate_payload(
                candidates.keys().cloned().collect(),
            )));
        }

        if candidates.is_empty() {
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::AmbiguousNotation,
                head.span,
                format!("notation {} has no resolved candidates", head.token),
            ));
        }

        Ok(candidates.into_values().collect())
    }
}

fn resolve_human_tactic_global_name(
    scope: &HumanTacticGlobalScope,
    name: &HumanName,
) -> HumanResult<Option<HumanGlobalRef>> {
    for candidates in human_tactic_global_candidate_levels(scope, name) {
        let mut candidates = human_tactic_dedupe_and_sort_candidates(candidates);
        if candidates.is_empty() {
            continue;
        }
        if candidates.len() == 1 {
            return Ok(Some(candidates.remove(0).reference));
        }
        return Err(HumanDiagnostic::error(
            HumanDiagnosticKind::AmbiguousName,
            name.span,
            format!("ambiguous name {}", name.as_dotted()),
        )
        .with_payload(human_tactic_candidate_payload(
            candidates
                .into_iter()
                .map(|candidate| candidate.key)
                .collect(),
        )));
    }
    Ok(None)
}

fn human_tactic_global_candidate_levels(
    scope: &HumanTacticGlobalScope,
    name: &HumanName,
) -> Vec<Vec<HumanTacticNameCandidate>> {
    let exact = npa_cert::Name(name.parts.clone());
    if name.parts.len() == 1 {
        vec![
            human_tactic_lookup_exact_candidates(scope, &exact),
            human_tactic_short_name_candidates(scope, &name.parts[0]),
        ]
    } else {
        vec![
            human_tactic_lookup_exact_candidates(scope, &exact),
            human_tactic_suffix_candidates(scope, &name.parts),
        ]
    }
}

fn human_tactic_lookup_exact_candidates(
    scope: &HumanTacticGlobalScope,
    name: &npa_cert::Name,
) -> Vec<HumanTacticNameCandidate> {
    let current = scope
        .current
        .iter()
        .filter(|entry| npa_cert::Name(entry.name.parts.clone()) == *name)
        .map(human_tactic_candidate_from_entry)
        .collect::<Vec<_>>();
    if !current.is_empty() {
        return current;
    }

    scope
        .imported
        .iter()
        .filter(|entry| npa_cert::Name(entry.name.parts.clone()) == *name)
        .map(human_tactic_candidate_from_entry)
        .collect()
}

fn human_tactic_short_name_candidates(
    scope: &HumanTacticGlobalScope,
    short_name: &str,
) -> Vec<HumanTacticNameCandidate> {
    let current = scope
        .current
        .iter()
        .filter(|entry| {
            entry
                .name
                .parts
                .last()
                .is_some_and(|part| part == short_name)
        })
        .map(human_tactic_candidate_from_entry)
        .collect::<Vec<_>>();
    if !current.is_empty() {
        return current;
    }

    scope
        .imported
        .iter()
        .filter(|entry| {
            entry
                .name
                .parts
                .last()
                .is_some_and(|part| part == short_name)
        })
        .map(human_tactic_candidate_from_entry)
        .collect()
}

fn human_tactic_suffix_candidates(
    scope: &HumanTacticGlobalScope,
    suffix: &[String],
) -> Vec<HumanTacticNameCandidate> {
    let current = scope
        .current
        .iter()
        .filter(|entry| human_tactic_name_has_suffix(&entry.name.parts, suffix))
        .map(human_tactic_candidate_from_entry)
        .collect::<Vec<_>>();
    if !current.is_empty() {
        return current;
    }

    scope
        .imported
        .iter()
        .filter(|entry| human_tactic_name_has_suffix(&entry.name.parts, suffix))
        .map(human_tactic_candidate_from_entry)
        .collect()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HumanTacticNameCandidate {
    key: String,
    reference: HumanGlobalRef,
}

fn human_tactic_candidate_from_entry(entry: &HumanGlobalScopeEntry) -> HumanTacticNameCandidate {
    HumanTacticNameCandidate {
        key: human_tactic_global_ref_sort_key(&entry.reference),
        reference: entry.reference.clone(),
    }
}

fn human_tactic_dedupe_and_sort_candidates(
    candidates: Vec<HumanTacticNameCandidate>,
) -> Vec<HumanTacticNameCandidate> {
    let mut map = BTreeMap::new();
    for candidate in candidates {
        map.entry(candidate.key.clone()).or_insert(candidate);
    }
    map.into_values().collect()
}

fn human_tactic_candidate_payload(mut candidates: Vec<String>) -> HumanDiagnosticPayload {
    candidates.sort();
    candidates.dedup();
    candidates.truncate(32);
    HumanDiagnosticPayload {
        candidates,
        ..HumanDiagnosticPayload::default()
    }
}

fn human_tactic_global_ref_sort_key(reference: &HumanGlobalRef) -> String {
    match reference {
        HumanGlobalRef::Imported {
            module,
            name,
            decl_interface_hash,
        } => format!(
            "imported:{}:{}:{}",
            module.as_dotted(),
            name.as_dotted(),
            human_tactic_hash_hex(decl_interface_hash)
        ),
        HumanGlobalRef::Builtin {
            name,
            decl_interface_hash,
        } => format!(
            "builtin:{}:{}",
            name.as_dotted(),
            human_tactic_hash_hex(decl_interface_hash)
        ),
        HumanGlobalRef::Local { index, name } => {
            format!("local:{index:08}:{}", name.as_dotted())
        }
        HumanGlobalRef::LocalGenerated { index, name } => {
            format!("local-generated:{index:08}:{}", name.as_dotted())
        }
    }
}

fn human_tactic_hash_hex(hash: &npa_cert::Hash) -> String {
    hash.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn human_tactic_name_has_suffix(name: &[String], suffix: &[String]) -> bool {
    name.len() >= suffix.len() && &name[(name.len() - suffix.len())..] == suffix
}

fn human_binder_group_end(binders: &[HumanBinder], start: usize) -> usize {
    let mut end = start + 1;
    while end < binders.len() && same_human_binder_group(&binders[start], &binders[end]) {
        end += 1;
    }
    end
}

fn elaborate_human_tactic_term_check_with_plan(
    context: &HumanTacticTermElabContext,
    resolved: &HumanResolvedTacticTerm<'_>,
    notation_plan: &[usize],
    expected: &Expr,
) -> HumanResult<HumanTacticTermCheckOutput> {
    let span = resolved.term.span();
    let mut lowering = HumanToMachineLowering::for_tactic_term(
        &resolved.resolved_names,
        &resolved.resolved_notations,
        notation_plan,
        HumanImplicitInserter::from_tactic_context(context),
    );
    lowering.meta_store.begin_declaration();
    let mut lowering_locals =
        HumanLoweringLocalContext::from_machine_locals(&context.local_context);
    let mut locals = human_local_context_from_machine(&context.local_context);
    let expected_machine = core_expr_to_machine_term(expected, &locals, span);
    let lowered = lowering.lower_expr(
        resolved.term.clone(),
        &mut lowering_locals,
        expected_machine.as_ref(),
    )?;
    let lowered =
        lowering
            .implicit_inserter
            .insert_term(lowered, &mut locals, &context.universe_params)?;
    lowering.meta_store.reject_unsolved_for_decl(span)?;
    let expr = HumanBidirectionalElaborator::from_tactic_context(context).check_human_expr(
        &lowered,
        expected,
        &locals,
        &context.universe_params,
    )?;

    Ok(HumanTacticTermCheckOutput {
        expr,
        inferred_type: expected.clone(),
    })
}

fn elaborate_human_tactic_term_infer_with_plan(
    context: &HumanTacticTermElabContext,
    resolved: &HumanResolvedTacticTerm<'_>,
    notation_plan: &[usize],
) -> HumanResult<HumanTacticTermInferOutput> {
    let span = resolved.term.span();
    let mut lowering = HumanToMachineLowering::for_tactic_term(
        &resolved.resolved_names,
        &resolved.resolved_notations,
        notation_plan,
        HumanImplicitInserter::from_tactic_context(context),
    );
    lowering.meta_store.begin_declaration();
    let mut lowering_locals =
        HumanLoweringLocalContext::from_machine_locals(&context.local_context);
    let mut locals = human_local_context_from_machine(&context.local_context);
    let lowered = lowering.lower_expr(resolved.term.clone(), &mut lowering_locals, None)?;
    let lowered =
        lowering
            .implicit_inserter
            .insert_term(lowered, &mut locals, &context.universe_params)?;
    lowering.meta_store.reject_unsolved_for_decl(span)?;
    let (expr, inferred_type) = HumanBidirectionalElaborator::from_tactic_context(context)
        .infer_human_expr(&lowered, &locals, &context.universe_params)?;

    Ok(HumanTacticTermInferOutput {
        expr,
        inferred_type,
    })
}

fn elaborate_human_module_with_notation_plan(
    module_name: npa_cert::ModuleName,
    module: &ResolvedHumanModule,
    verified_imports: &[VerifiedImport],
    notation_plan: &[usize],
) -> HumanResult<npa_cert::CoreModule> {
    let span = module.module.span;
    let mut lowering = HumanToMachineLowering::new(module, verified_imports, notation_plan)
        .map_err(|diagnostic| diagnostic.with_default_phase(HumanDiagnosticPhase::Elaborator))?;
    let machine_module = lowering
        .lower_module(module)
        .map_err(|diagnostic| diagnostic.with_default_phase(HumanDiagnosticPhase::Elaborator))?;
    HumanBidirectionalElaborator::new(module, verified_imports)
        .map_err(|diagnostic| diagnostic.with_default_phase(HumanDiagnosticPhase::Elaborator))?
        .elaborate_module(module_name, machine_module)
        .map_err(|diagnostic| {
            if diagnostic.primary_span == Span::empty(crate::FileId(0)) {
                HumanDiagnostic::error(diagnostic.kind, span, diagnostic.message)
                    .with_default_phase(HumanDiagnosticPhase::Elaborator)
            } else {
                diagnostic.with_default_phase(HumanDiagnosticPhase::Elaborator)
            }
        })
}

fn elaborate_human_module_with_by_proofs(
    module_name: npa_cert::ModuleName,
    module: ResolvedHumanModule,
    verified_imports: &[VerifiedImport],
    by_proofs: &BTreeMap<u64, Expr>,
    options: &HumanCompileOptions,
) -> HumanResult<npa_cert::CoreModule> {
    let span = module.module.span;
    let plans = notation_candidate_plans(&module, options.max_notation_candidates)
        .map_err(|diagnostic| diagnostic.with_default_phase(HumanDiagnosticPhase::Elaborator))?;
    let mut first_error = None;
    let mut success = None;

    for plan in plans {
        match elaborate_human_module_with_notation_plan_and_by_proofs(
            module_name.clone(),
            &module,
            verified_imports,
            &plan,
            by_proofs,
        ) {
            Ok(core) if success.is_none() => success = Some(core),
            Ok(_) => {
                return Err(HumanDiagnostic::error(
                    HumanDiagnosticKind::AmbiguousNotation,
                    span,
                    "multiple notation candidates elaborated successfully",
                )
                .with_default_phase(HumanDiagnosticPhase::Elaborator));
            }
            Err(err) => {
                first_error.get_or_insert(err);
            }
        }
    }

    if let Some(core) = success {
        Ok(core)
    } else if let Some(err) = first_error {
        Err(err.with_default_phase(HumanDiagnosticPhase::Elaborator))
    } else {
        Err(HumanDiagnostic::error(
            HumanDiagnosticKind::AmbiguousNotation,
            span,
            "no notation candidate plan was available",
        )
        .with_default_phase(HumanDiagnosticPhase::Elaborator))
    }
}

fn elaborate_human_module_with_notation_plan_and_by_proofs(
    module_name: npa_cert::ModuleName,
    module: &ResolvedHumanModule,
    verified_imports: &[VerifiedImport],
    notation_plan: &[usize],
    by_proofs: &BTreeMap<u64, Expr>,
) -> HumanResult<npa_cert::CoreModule> {
    let span = module.module.span;
    let mut lowering = HumanToMachineLowering::new(module, verified_imports, notation_plan)
        .map_err(|diagnostic| diagnostic.with_default_phase(HumanDiagnosticPhase::Elaborator))?
        .with_current_module_prefix(module_name.clone());
    let machine_module = lowering
        .lower_module_with_core_proofs(module, by_proofs)
        .map_err(|diagnostic| diagnostic.with_default_phase(HumanDiagnosticPhase::Elaborator))?;
    HumanBidirectionalElaborator::new(module, verified_imports)
        .map_err(|diagnostic| diagnostic.with_default_phase(HumanDiagnosticPhase::Elaborator))?
        .elaborate_module(module_name, machine_module)
        .map_err(|diagnostic| {
            if diagnostic.primary_span == Span::empty(crate::FileId(0)) {
                HumanDiagnostic::error(diagnostic.kind, span, diagnostic.message)
                    .with_default_phase(HumanDiagnosticPhase::Elaborator)
            } else {
                diagnostic.with_default_phase(HumanDiagnosticPhase::Elaborator)
            }
        })
}

fn notation_candidate_plans(
    module: &ResolvedHumanModule,
    max_plans: usize,
) -> HumanResult<Vec<Vec<usize>>> {
    notation_candidate_plans_from_uses(&module.resolved_notations, max_plans)
}

fn notation_candidate_plans_for_count(
    module: &ResolvedHumanModule,
    max_plans: usize,
    notation_use_count: usize,
) -> HumanResult<Vec<Vec<usize>>> {
    let notations = module
        .resolved_notations
        .get(..notation_use_count)
        .ok_or_else(|| {
            HumanDiagnostic::not_implemented(
                module.module.span,
                "Human proof start notation cursor",
            )
        })?;
    notation_candidate_plans_from_uses(notations, max_plans)
}

fn notation_candidate_plans_from_uses(
    notations: &[HumanResolvedNotationUse],
    max_plans: usize,
) -> HumanResult<Vec<Vec<usize>>> {
    let mut plans = vec![Vec::new()];

    for notation in notations {
        if notation.candidates.is_empty() {
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::AmbiguousNotation,
                notation.head.span,
                format!("notation {} has no candidates", notation.head.token),
            ));
        }
        let mut next = Vec::new();
        for plan in &plans {
            for candidate_index in 0..notation.candidates.len() {
                let mut plan = plan.clone();
                plan.push(candidate_index);
                next.push(plan);
                if next.len() > max_plans {
                    return Err(HumanDiagnostic::error(
                        HumanDiagnosticKind::TooManyNotationCandidates,
                        notation.head.span,
                        format!(
                            "notation {} exceeds the bounded elaboration candidate budget",
                            notation.head.token
                        ),
                    ));
                }
            }
        }
        plans = next;
    }

    Ok(plans)
}

fn human_proof_start_notation_use_count(
    module_name: &npa_cert::ModuleName,
    theorem_name: &npa_cert::Name,
    module: &ResolvedHumanModule,
) -> HumanResult<usize> {
    let mut count = 0;
    let mut declarations = module.state.source_interfaces.current.declarations.iter();

    for item in &module.module.items {
        match item {
            HumanItem::Import { .. }
            | HumanItem::Open { .. }
            | HumanItem::NamespaceStart { .. }
            | HumanItem::NamespaceEnd { .. }
            | HumanItem::Notation(_) => {}
            HumanItem::Def(decl) => {
                let metadata = declarations.next().ok_or_else(|| {
                    HumanDiagnostic::not_implemented(decl.span, "Human declaration metadata")
                })?;
                if human_current_name_matches_target(module_name, &metadata.name, theorem_name) {
                    return Ok(count);
                }
                count += human_decl_notation_use_count(decl);
            }
            HumanItem::Theorem(decl) => {
                let metadata = declarations.next().ok_or_else(|| {
                    HumanDiagnostic::not_implemented(decl.span, "Human declaration metadata")
                })?;
                if human_current_name_matches_target(module_name, &metadata.name, theorem_name) {
                    if matches!(decl.value, HumanDeclValue::ProofBlock(_)) {
                        count += human_decl_signature_notation_use_count(decl);
                    }
                    return Ok(count);
                }
                count += human_decl_signature_notation_use_count(decl);
                if let HumanDeclValue::Term(value) = &decl.value {
                    count += human_expr_notation_use_count(value);
                }
            }
            HumanItem::Axiom(decl) => {
                let metadata = declarations.next().ok_or_else(|| {
                    HumanDiagnostic::not_implemented(decl.span, "Human declaration metadata")
                })?;
                if human_current_name_matches_target(module_name, &metadata.name, theorem_name) {
                    return Ok(count);
                }
                count += human_binders_notation_use_count(&decl.binders);
                count += human_expr_notation_use_count(&decl.ty);
            }
            HumanItem::Inductive(decl) => {
                let metadata = declarations.next().ok_or_else(|| {
                    HumanDiagnostic::not_implemented(decl.span, "Human declaration metadata")
                })?;
                if human_current_name_matches_target(module_name, &metadata.name, theorem_name) {
                    return Ok(count);
                }
                count += human_binders_notation_use_count(&decl.binders);
                count += human_expr_notation_use_count(&decl.ty);
                count += decl
                    .constructors
                    .iter()
                    .map(|constructor| human_expr_notation_use_count(&constructor.ty))
                    .sum::<usize>();
            }
        }
    }

    Ok(count)
}

fn human_decl_notation_use_count(decl: &crate::HumanDecl) -> usize {
    let mut count = human_decl_signature_notation_use_count(decl);
    if let HumanDeclValue::Term(value) = &decl.value {
        count += human_expr_notation_use_count(value);
    }
    count
}

fn human_decl_signature_notation_use_count(decl: &crate::HumanDecl) -> usize {
    human_binders_notation_use_count(&decl.binders) + human_expr_notation_use_count(&decl.ty)
}

fn human_binders_notation_use_count(binders: &[HumanBinder]) -> usize {
    binders
        .iter()
        .filter_map(|binder| binder.ty.as_deref())
        .map(human_expr_notation_use_count)
        .sum()
}

fn human_expr_notation_use_count(expr: &HumanExpr) -> usize {
    match expr {
        HumanExpr::Ident { .. } | HumanExpr::Sort { .. } | HumanExpr::Hole { .. } => 0,
        HumanExpr::App { func, arg, .. } => {
            human_expr_notation_use_count(func) + human_expr_notation_use_count(arg)
        }
        HumanExpr::Lam { binders, body, .. } | HumanExpr::Pi { binders, body, .. } => {
            human_binders_notation_use_count(binders) + human_expr_notation_use_count(body)
        }
        HumanExpr::Let {
            ty, value, body, ..
        } => {
            ty.as_deref().map_or(0, human_expr_notation_use_count)
                + human_expr_notation_use_count(value)
                + human_expr_notation_use_count(body)
        }
        HumanExpr::Annot { expr, ty, .. } => {
            human_expr_notation_use_count(expr) + human_expr_notation_use_count(ty)
        }
        HumanExpr::Arrow {
            domain, codomain, ..
        } => human_expr_notation_use_count(domain) + human_expr_notation_use_count(codomain),
        HumanExpr::NotationApp { args, .. } => {
            args.iter()
                .map(human_expr_notation_use_count)
                .sum::<usize>()
                + 1
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HumanLoweredDeclKind {
    Def,
    Theorem,
}

#[derive(Clone, Debug)]
struct HumanLoweredModule {
    items: Vec<HumanLoweredItem>,
}

#[derive(Clone, Debug)]
enum HumanLoweredItem {
    Import,
    Def(MachineDecl),
    Theorem(MachineDecl),
    TheoremCoreProof { decl: Decl, span: Span },
    Axiom(HumanLoweredAxiomDecl),
    Inductive(HumanLoweredInductiveDecl),
}

#[derive(Clone, Debug)]
struct HumanLoweredProofStart {
    source_index: u64,
    prior_items: Vec<HumanLoweredItem>,
    target: HumanLoweredDeclSignature,
}

#[derive(Clone, Debug)]
struct HumanLoweredDeclSignature {
    name: MachineName,
    universe_params: Vec<MachineUniverseParam>,
    binders: Vec<MachineBinder>,
    ty: MachineTerm,
}

#[derive(Clone, Debug)]
struct HumanLoweredAxiomDecl {
    name: MachineName,
    universe_params: Vec<MachineUniverseParam>,
    binders: Vec<MachineBinder>,
    ty: MachineTerm,
    span: Span,
}

#[derive(Clone, Debug)]
struct HumanLoweredInductiveDecl {
    name: MachineName,
    universe_params: Vec<MachineUniverseParam>,
    binders: Vec<MachineBinder>,
    ty: MachineTerm,
    constructors: Vec<HumanLoweredConstructorDecl>,
    span: Span,
}

#[derive(Clone, Debug)]
struct HumanLoweredConstructorDecl {
    name: MachineName,
    ty: MachineTerm,
    span: Span,
}

#[derive(Clone, Debug)]
struct HumanCallableSignature {
    universe_params: Vec<String>,
    implicit_profile: Vec<MachineCallableBinderVisibility>,
}

const HUMAN_UNIVERSE_META_PREFIX: &str = "__npa_internal_human_universe_meta#";
const HUMAN_SPINE_IMPLICIT_PREFIX: &str = "__npa_internal_human_spine_implicit#";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct HumanSpineImplicitId(usize);

#[derive(Clone, Debug)]
struct HumanSpineImplicit {
    domain: Expr,
    assignment: Option<Expr>,
    span: Span,
}

#[derive(Clone, Debug)]
enum HumanSpineArg {
    Explicit(MachineTerm),
    Implicit(HumanSpineImplicitId),
}

#[derive(Clone, Debug)]
struct HumanSolvedSpine {
    universe_args: Option<Vec<MachineLevel>>,
    args: Vec<MachineTerm>,
}

struct HumanUniverseSpineSolver<'a> {
    env: &'a Env,
    locals: &'a HumanLocalContext,
    delta: &'a [String],
    universe_params: &'a [String],
    universe_assignments: Vec<Option<Level>>,
    implicit_metas: Vec<HumanSpineImplicit>,
}

impl<'a> HumanUniverseSpineSolver<'a> {
    fn new(
        env: &'a Env,
        locals: &'a HumanLocalContext,
        delta: &'a [String],
        universe_params: &'a [String],
        fixed_universe_args: Option<Vec<Level>>,
    ) -> Self {
        let universe_assignments = fixed_universe_args.map_or_else(
            || vec![None; universe_params.len()],
            |levels| levels.into_iter().map(Some).collect(),
        );
        Self {
            env,
            locals,
            delta,
            universe_params,
            universe_assignments,
            implicit_metas: Vec::new(),
        }
    }

    fn decl_type_with_universe_metas(&self, ty: &Expr, span: Span) -> HumanResult<Expr> {
        let levels = (0..self.universe_params.len())
            .map(|index| {
                self.universe_assignments[index]
                    .clone()
                    .unwrap_or_else(|| human_universe_meta_level(index))
            })
            .collect::<Vec<_>>();
        let _ = span;
        Ok(subst::subst_levels_expr(ty, self.universe_params, &levels))
    }

    fn fresh_implicit(&mut self, domain: Expr, span: Span) -> HumanSpineImplicitId {
        let id = HumanSpineImplicitId(self.implicit_metas.len());
        self.implicit_metas.push(HumanSpineImplicit {
            domain,
            assignment: None,
            span,
        });
        id
    }

    fn implicit_placeholder(id: HumanSpineImplicitId) -> Expr {
        Expr::konst(format!("{HUMAN_SPINE_IMPLICIT_PREFIX}{}", id.0), Vec::new())
    }

    fn implicit_id(expr: &Expr) -> Option<HumanSpineImplicitId> {
        let Expr::Const { name, levels } = expr else {
            return None;
        };
        if !levels.is_empty() {
            return None;
        }
        name.strip_prefix(HUMAN_SPINE_IMPLICIT_PREFIX)
            .and_then(|suffix| suffix.parse::<usize>().ok())
            .map(HumanSpineImplicitId)
    }

    fn resolve_implicit_expr(&self, expr: Expr) -> Expr {
        self.resolve_implicit_expr_at(expr, 0)
    }

    fn resolve_implicit_expr_at(&self, expr: Expr, depth: u32) -> Expr {
        if let Some(id) = Self::implicit_id(&expr) {
            return self.implicit_metas[id.0]
                .assignment
                .clone()
                .map(|assignment| {
                    let assignment = self.resolve_implicit_expr_at(assignment, 0);
                    subst::shift(&assignment, depth as i32, 0)
                        .expect("positive Human implicit lift must preserve de Bruijn indices")
                })
                .unwrap_or(expr);
        }
        match expr {
            Expr::App(fun, arg) => Expr::app(
                self.resolve_implicit_expr_at(*fun, depth),
                self.resolve_implicit_expr_at(*arg, depth),
            ),
            Expr::Lam { binder, ty, body } => Expr::lam(
                binder,
                self.resolve_implicit_expr_at(*ty, depth),
                self.resolve_implicit_expr_at(*body, depth + 1),
            ),
            Expr::Pi { binder, ty, body } => Expr::pi(
                binder,
                self.resolve_implicit_expr_at(*ty, depth),
                self.resolve_implicit_expr_at(*body, depth + 1),
            ),
            Expr::Let {
                binder,
                ty,
                value,
                body,
            } => Expr::let_in(
                binder,
                self.resolve_implicit_expr_at(*ty, depth),
                self.resolve_implicit_expr_at(*value, depth),
                self.resolve_implicit_expr_at(*body, depth + 1),
            ),
            Expr::Sort(level) => Expr::sort(self.resolve_level(level)),
            Expr::Const { name, levels } => Expr::konst(
                name,
                levels
                    .into_iter()
                    .map(|level| self.resolve_level(level))
                    .collect(),
            ),
            Expr::BVar(_) => expr,
        }
    }

    fn resolve_level(&self, level: Level) -> Level {
        if let Some(index) = human_universe_meta_index(&level) {
            return self.universe_assignments[index]
                .clone()
                .map(|assignment| self.resolve_level(assignment))
                .unwrap_or(level);
        }
        match level {
            Level::Succ(inner) => Level::succ(self.resolve_level(*inner)),
            Level::Max(lhs, rhs) => Level::max(self.resolve_level(*lhs), self.resolve_level(*rhs)),
            Level::IMax(lhs, rhs) => {
                Level::imax(self.resolve_level(*lhs), self.resolve_level(*rhs))
            }
            Level::Zero | Level::Param(_) => level,
        }
    }

    fn unify_expr(&mut self, lhs: Expr, rhs: Expr, span: Span) -> HumanResult<()> {
        let lhs = self.resolve_implicit_expr(lhs);
        let rhs = self.resolve_implicit_expr(rhs);
        if lhs == rhs {
            return Ok(());
        }
        if let Some(id) = Self::implicit_id(&lhs) {
            return self.assign_implicit(id, rhs, span);
        }
        if let Some(id) = Self::implicit_id(&rhs) {
            return self.assign_implicit(id, lhs, span);
        }

        match (lhs, rhs) {
            (Expr::Sort(lhs), Expr::Sort(rhs)) => self.unify_level(lhs, rhs, span),
            (
                Expr::Const {
                    name: lhs_name,
                    levels: lhs_levels,
                },
                Expr::Const {
                    name: rhs_name,
                    levels: rhs_levels,
                },
            ) if lhs_name == rhs_name && lhs_levels.len() == rhs_levels.len() => {
                for (lhs, rhs) in lhs_levels.into_iter().zip(rhs_levels) {
                    self.unify_level(lhs, rhs, span)?;
                }
                Ok(())
            }
            (Expr::App(lhs_fun, lhs_arg), Expr::App(rhs_fun, rhs_arg)) => {
                self.unify_expr(*lhs_fun, *rhs_fun, span)?;
                self.unify_expr(*lhs_arg, *rhs_arg, span)
            }
            (
                Expr::Lam {
                    ty: lhs_ty,
                    body: lhs_body,
                    ..
                },
                Expr::Lam {
                    ty: rhs_ty,
                    body: rhs_body,
                    ..
                },
            )
            | (
                Expr::Pi {
                    ty: lhs_ty,
                    body: lhs_body,
                    ..
                },
                Expr::Pi {
                    ty: rhs_ty,
                    body: rhs_body,
                    ..
                },
            ) => {
                self.unify_expr(*lhs_ty, *rhs_ty, span)?;
                self.unify_expr(
                    self.resolve_implicit_expr_at(*lhs_body, 1),
                    self.resolve_implicit_expr_at(*rhs_body, 1),
                    span,
                )
            }
            (
                Expr::Let {
                    ty: lhs_ty,
                    value: lhs_value,
                    body: lhs_body,
                    ..
                },
                Expr::Let {
                    ty: rhs_ty,
                    value: rhs_value,
                    body: rhs_body,
                    ..
                },
            ) => {
                self.unify_expr(*lhs_ty, *rhs_ty, span)?;
                self.unify_expr(*lhs_value, *rhs_value, span)?;
                self.unify_expr(
                    self.resolve_implicit_expr_at(*lhs_body, 1),
                    self.resolve_implicit_expr_at(*rhs_body, 1),
                    span,
                )
            }
            (Expr::BVar(lhs), Expr::BVar(rhs)) if lhs == rhs => Ok(()),
            _ => Err(human_universe_solver_error(
                span,
                "Human universe constraint unsatisfied while inferring implicit arguments",
            )),
        }
    }

    fn assign_implicit(
        &mut self,
        id: HumanSpineImplicitId,
        value: Expr,
        span: Span,
    ) -> HumanResult<()> {
        if expr_contains_spine_implicit(id, &value) {
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::OccursCheckFailed,
                span,
                "Human implicit argument assignment failed the occurs check",
            ));
        }

        if let Some(existing) = self.implicit_metas[id.0].assignment.clone() {
            return self.unify_expr(existing, value, span);
        }

        let value_ty = self
            .env
            .infer(&self.locals.to_kernel_ctx(), self.delta, &value)
            .map_err(|err| {
                human_kernel_expr_diagnostic(span, err, "Human implicit assignment inference")
            })?;
        let domain = self.implicit_metas[id.0].domain.clone();
        self.unify_expr(domain, value_ty, span)?;
        self.implicit_metas[id.0].assignment = Some(value);
        Ok(())
    }

    fn unify_level(&mut self, lhs: Level, rhs: Level, span: Span) -> HumanResult<()> {
        let lhs = npa_kernel::level::normalize_level(self.resolve_level(lhs));
        let rhs = npa_kernel::level::normalize_level(self.resolve_level(rhs));
        if lhs == rhs {
            return Ok(());
        }
        if let Some(index) = human_universe_meta_index(&lhs) {
            return self.assign_universe(index, rhs, span);
        }
        if let Some(index) = human_universe_meta_index(&rhs) {
            return self.assign_universe(index, lhs, span);
        }

        match (lhs, rhs) {
            (Level::Succ(lhs), Level::Succ(rhs)) => self.unify_level(*lhs, *rhs, span),
            (Level::Max(lhs_a, lhs_b), Level::Max(rhs_a, rhs_b))
            | (Level::IMax(lhs_a, lhs_b), Level::IMax(rhs_a, rhs_b)) => {
                self.unify_level(*lhs_a, *rhs_a, span)?;
                self.unify_level(*lhs_b, *rhs_b, span)
            }
            _ => Err(human_universe_solver_error(
                span,
                "Human universe equality constraint is unsatisfied",
            )),
        }
    }

    fn assign_universe(&mut self, index: usize, value: Level, span: Span) -> HumanResult<()> {
        if level_contains_universe_meta(index, &value) {
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::OccursCheckFailed,
                span,
                "Human universe metavariable assignment failed the occurs check",
            ));
        }

        let value = npa_kernel::level::normalize_level(value);
        if let Some(existing) = self.universe_assignments[index].clone() {
            return self.unify_level(existing, value, span);
        }
        self.universe_assignments[index] = Some(value);
        Ok(())
    }

    fn materialize_implicit_arg(
        &self,
        id: HumanSpineImplicitId,
        span: Span,
    ) -> HumanResult<MachineTerm> {
        let Some(value) = self.implicit_metas[id.0].assignment.clone() else {
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::UnsolvedImplicit,
                self.implicit_metas[id.0].span,
                "unsolved synthetic implicit argument",
            )
            .with_payload(HumanDiagnosticPayload {
                unsolved_meta: Some(HumanUnsolvedMeta {
                    kind: HumanUnsolvedMetaKind::SyntheticImplicit,
                    name: None,
                }),
                ..HumanDiagnosticPayload::default()
            }));
        };
        let value = self.resolve_implicit_expr(value);
        core_expr_to_machine_term(&value, self.locals, span).ok_or_else(|| {
            HumanDiagnostic::error(
                HumanDiagnosticKind::UnsolvedImplicit,
                span,
                "cannot materialize inferred implicit argument",
            )
            .with_payload(HumanDiagnosticPayload {
                unsolved_meta: Some(HumanUnsolvedMeta {
                    kind: HumanUnsolvedMetaKind::SyntheticImplicit,
                    name: None,
                }),
                ..HumanDiagnosticPayload::default()
            })
        })
    }

    fn solved_universe_args(&self, span: Span) -> HumanResult<Vec<MachineLevel>> {
        self.universe_assignments
            .iter()
            .enumerate()
            .map(|(index, level)| {
                let Some(level) = level.clone() else {
                    return Err(human_universe_solver_error(
                        span,
                        "ambiguous Human universe metavariable",
                    ));
                };
                if human_level_contains_any_universe_meta(&level) {
                    return Err(human_universe_solver_error(
                        span,
                        "unresolved Human universe metavariable",
                    ));
                }
                let _ = index;
                Ok(core_level_to_machine_level(
                    &npa_kernel::level::normalize_level(level),
                    span,
                ))
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct HumanTermMetaId(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct HumanUniverseMetaId(u32);

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum HumanTermMetaKind {
    UserHole,
    SyntheticImplicit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HumanTermMeta {
    id: HumanTermMetaId,
    name: Option<String>,
    context: HumanMetaContextSnapshot,
    goal_context: Vec<HumanHoleGoalLocal>,
    target: Option<String>,
    assignment: Option<HumanMetaExpr>,
    kind: HumanTermMetaKind,
    span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HumanUniverseMeta {
    id: HumanUniverseMetaId,
    assignment: Option<HumanMetaLevel>,
    span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HumanMetaContextSnapshot {
    locals: Vec<HumanMetaLocalSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HumanMetaLocalSnapshot {
    ty: MachineTerm,
    value: Option<MachineTerm>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum HumanMetaExpr {
    Core(Expr),
    Meta(HumanTermMetaId),
    App(Box<HumanMetaExpr>, Box<HumanMetaExpr>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum HumanMetaLevel {
    Core(Level),
    Meta(HumanUniverseMetaId),
    Succ(Box<HumanMetaLevel>),
    Max(Box<HumanMetaLevel>, Box<HumanMetaLevel>),
    IMax(Box<HumanMetaLevel>, Box<HumanMetaLevel>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HumanTermConstraintKind {
    TypeEq,
    TermEq,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum HumanConstraint {
    TypeEq {
        lhs: HumanMetaExpr,
        rhs: HumanMetaExpr,
        span: Span,
    },
    TermEq {
        lhs: HumanMetaExpr,
        rhs: HumanMetaExpr,
        span: Span,
    },
    LevelEq {
        lhs: HumanMetaLevel,
        rhs: HumanMetaLevel,
        span: Span,
    },
    LevelLe {
        lhs: HumanMetaLevel,
        rhs: HumanMetaLevel,
        span: Span,
    },
}

#[derive(Clone, Debug, Default)]
struct HumanMetaStore {
    term_metas: Vec<HumanTermMeta>,
    universe_metas: Vec<HumanUniverseMeta>,
    constraints: VecDeque<HumanConstraint>,
    named_holes: BTreeMap<String, HumanTermMetaId>,
}

impl HumanMetaStore {
    fn begin_declaration(&mut self) {
        self.term_metas.clear();
        self.universe_metas.clear();
        self.constraints.clear();
        self.named_holes.clear();
    }

    fn fresh_user_hole(
        &mut self,
        name: Option<&HumanName>,
        context: &HumanLoweringLocalContext,
        expected: Option<&MachineTerm>,
        span: Span,
    ) -> HumanResult<HumanTermMetaId> {
        let snapshot = context.meta_snapshot();
        let hole_name = name.map(|name| format!("?{}", name.as_dotted()));

        if let Some(hole_name) = &hole_name {
            if let Some(existing) = self.named_holes.get(hole_name).copied() {
                let existing_meta = self.term_meta(existing);
                if existing_meta.context != snapshot {
                    return Err(HumanDiagnostic::error(
                        HumanDiagnosticKind::NamedHoleContextMismatch,
                        span,
                        format!("named hole {hole_name} was reused with a different context"),
                    )
                    .with_payload(HumanDiagnosticPayload {
                        hole_goals: vec![
                            self.hole_goal(existing_meta),
                            HumanHoleGoal {
                                hole: Some(hole_name.clone()),
                                context: context.hole_goal_context(),
                                target: expected.map(render_machine_term),
                                source_span: span,
                            },
                        ],
                        ..HumanDiagnosticPayload::default()
                    }));
                }
                return Ok(existing);
            }
        }

        let id = HumanTermMetaId(self.term_metas.len() as u32);
        let meta = HumanTermMeta {
            id,
            name: hole_name.clone(),
            context: snapshot,
            goal_context: context.hole_goal_context(),
            target: expected.map(render_machine_term),
            assignment: None,
            kind: HumanTermMetaKind::UserHole,
            span,
        };
        self.term_metas.push(meta);
        if let Some(hole_name) = hole_name {
            self.named_holes.insert(hole_name, id);
        }
        Ok(id)
    }

    #[allow(dead_code)]
    fn fresh_synthetic_implicit(
        &mut self,
        context: &HumanLoweringLocalContext,
        expected: Option<&MachineTerm>,
        span: Span,
    ) -> HumanTermMetaId {
        let id = HumanTermMetaId(self.term_metas.len() as u32);
        self.term_metas.push(HumanTermMeta {
            id,
            name: None,
            context: context.meta_snapshot(),
            goal_context: context.hole_goal_context(),
            target: expected.map(render_machine_term),
            assignment: None,
            kind: HumanTermMetaKind::SyntheticImplicit,
            span,
        });
        id
    }

    #[allow(dead_code)]
    fn fresh_universe_meta(&mut self, span: Span) -> HumanUniverseMetaId {
        let id = HumanUniverseMetaId(self.universe_metas.len() as u32);
        self.universe_metas.push(HumanUniverseMeta {
            id,
            assignment: None,
            span,
        });
        id
    }

    fn add_constraint(&mut self, constraint: HumanConstraint) {
        self.constraints.push_back(constraint);
    }

    fn solve_constraints(&mut self) -> HumanResult<()> {
        while let Some(constraint) = self.constraints.pop_front() {
            match constraint {
                HumanConstraint::TypeEq { lhs, rhs, span } => {
                    self.solve_term_eq(HumanTermConstraintKind::TypeEq, lhs, rhs, span)?;
                }
                HumanConstraint::TermEq { lhs, rhs, span } => {
                    self.solve_term_eq(HumanTermConstraintKind::TermEq, lhs, rhs, span)?;
                }
                HumanConstraint::LevelEq { lhs, rhs, span } => {
                    self.solve_level_eq(lhs, rhs, span)?;
                }
                HumanConstraint::LevelLe { lhs, rhs, span } => {
                    self.solve_level_le(lhs, rhs, span)?;
                }
            }
        }
        Ok(())
    }

    fn reject_unsolved_for_decl(&mut self, span: Span) -> HumanResult<()> {
        self.solve_constraints()?;

        if let Some(meta) = self
            .term_metas
            .iter()
            .find(|meta| meta.assignment.is_none())
        {
            return match meta.kind {
                HumanTermMetaKind::UserHole => Err(HumanDiagnostic::error(
                    HumanDiagnosticKind::UnsolvedHole,
                    meta.span,
                    match &meta.name {
                        Some(name) => format!("unsolved hole {name}"),
                        None => "unsolved anonymous hole".to_owned(),
                    },
                )
                .with_payload(HumanDiagnosticPayload {
                    hole_goals: vec![self.hole_goal(meta)],
                    unsolved_meta: Some(HumanUnsolvedMeta {
                        kind: HumanUnsolvedMetaKind::Hole,
                        name: meta.name.clone(),
                    }),
                    ..HumanDiagnosticPayload::default()
                })),
                HumanTermMetaKind::SyntheticImplicit => Err(HumanDiagnostic::error(
                    HumanDiagnosticKind::UnsolvedImplicit,
                    meta.span,
                    "unsolved synthetic implicit argument",
                )
                .with_payload(HumanDiagnosticPayload {
                    unsolved_meta: Some(HumanUnsolvedMeta {
                        kind: HumanUnsolvedMetaKind::SyntheticImplicit,
                        name: None,
                    }),
                    ..HumanDiagnosticPayload::default()
                })),
            };
        }

        if let Some(meta) = self
            .universe_metas
            .iter()
            .find(|meta| meta.assignment.is_none())
        {
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::UnsolvedUniverseMeta,
                meta.span,
                "unsolved universe metavariable",
            )
            .with_payload(HumanDiagnosticPayload {
                unsolved_meta: Some(HumanUnsolvedMeta {
                    kind: HumanUnsolvedMetaKind::Universe,
                    name: None,
                }),
                ..HumanDiagnosticPayload::default()
            }));
        }

        let _ = span;
        Ok(())
    }

    fn solve_term_eq(
        &mut self,
        kind: HumanTermConstraintKind,
        lhs: HumanMetaExpr,
        rhs: HumanMetaExpr,
        span: Span,
    ) -> HumanResult<()> {
        let lhs = self.resolve_meta_expr(lhs);
        let rhs = self.resolve_meta_expr(rhs);
        match (lhs, rhs) {
            (lhs, rhs) if lhs == rhs => Ok(()),
            (HumanMetaExpr::Meta(id), value) | (value, HumanMetaExpr::Meta(id)) => {
                self.assign_term_meta(id, value, span)
            }
            (HumanMetaExpr::App(lhs_fun, lhs_arg), HumanMetaExpr::App(rhs_fun, rhs_arg)) => {
                self.add_constraint(term_constraint(kind, *lhs_fun, *rhs_fun, span));
                self.add_constraint(term_constraint(kind, *lhs_arg, *rhs_arg, span));
                Ok(())
            }
            (HumanMetaExpr::Core(lhs), HumanMetaExpr::Core(rhs)) if lhs == rhs => Ok(()),
            _ => Err(HumanDiagnostic::error(
                HumanDiagnosticKind::MachineElaborationError,
                span,
                "Human metavariable constraint could not be unified",
            )),
        }
    }

    fn solve_level_eq(
        &mut self,
        lhs: HumanMetaLevel,
        rhs: HumanMetaLevel,
        span: Span,
    ) -> HumanResult<()> {
        let lhs = self.resolve_meta_level(lhs);
        let rhs = self.resolve_meta_level(rhs);
        match (lhs, rhs) {
            (lhs, rhs) if lhs == rhs => Ok(()),
            (HumanMetaLevel::Meta(id), value) | (value, HumanMetaLevel::Meta(id)) => {
                self.assign_universe_meta(id, value, span)
            }
            (HumanMetaLevel::Succ(lhs), HumanMetaLevel::Succ(rhs)) => {
                self.add_constraint(HumanConstraint::LevelEq {
                    lhs: *lhs,
                    rhs: *rhs,
                    span,
                });
                Ok(())
            }
            (HumanMetaLevel::Max(lhs_a, lhs_b), HumanMetaLevel::Max(rhs_a, rhs_b)) => {
                self.add_constraint(HumanConstraint::LevelEq {
                    lhs: *lhs_a,
                    rhs: *rhs_a,
                    span,
                });
                self.add_constraint(HumanConstraint::LevelEq {
                    lhs: *lhs_b,
                    rhs: *rhs_b,
                    span,
                });
                Ok(())
            }
            (HumanMetaLevel::IMax(lhs_a, lhs_b), HumanMetaLevel::IMax(rhs_a, rhs_b)) => {
                self.add_constraint(HumanConstraint::LevelEq {
                    lhs: *lhs_a,
                    rhs: *rhs_a,
                    span,
                });
                self.add_constraint(HumanConstraint::LevelEq {
                    lhs: *lhs_b,
                    rhs: *rhs_b,
                    span,
                });
                Ok(())
            }
            (HumanMetaLevel::Core(lhs), HumanMetaLevel::Core(rhs)) if lhs == rhs => Ok(()),
            _ => Err(HumanDiagnostic::error(
                HumanDiagnosticKind::MachineElaborationError,
                span,
                "Human universe metavariable constraint could not be unified",
            )),
        }
    }

    fn solve_level_le(
        &mut self,
        lhs: HumanMetaLevel,
        rhs: HumanMetaLevel,
        span: Span,
    ) -> HumanResult<()> {
        let lhs = self.resolve_meta_level(lhs);
        let rhs = self.resolve_meta_level(rhs);
        match (lhs, rhs) {
            (lhs, rhs) if lhs == rhs => Ok(()),
            (HumanMetaLevel::Meta(id), value) => self.assign_universe_meta(id, value, span),
            (HumanMetaLevel::Core(lhs), HumanMetaLevel::Core(rhs))
                if human_level_leq(&lhs, &rhs) =>
            {
                Ok(())
            }
            _ => Err(HumanDiagnostic::error(
                HumanDiagnosticKind::MachineElaborationError,
                span,
                "Human universe inequality constraint could not be solved",
            )),
        }
    }

    fn assign_term_meta(
        &mut self,
        id: HumanTermMetaId,
        value: HumanMetaExpr,
        span: Span,
    ) -> HumanResult<()> {
        if meta_expr_occurs(id, &value) {
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::OccursCheckFailed,
                span,
                "Human metavariable assignment failed the occurs check",
            ));
        }

        let index = id.0 as usize;
        if let Some(existing) = self.term_metas[index].assignment.clone() {
            self.add_constraint(HumanConstraint::TermEq {
                lhs: existing,
                rhs: value,
                span,
            });
        } else {
            self.term_metas[index].assignment = Some(value);
        }
        Ok(())
    }

    fn assign_universe_meta(
        &mut self,
        id: HumanUniverseMetaId,
        value: HumanMetaLevel,
        span: Span,
    ) -> HumanResult<()> {
        if meta_level_occurs(id, &value) {
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::OccursCheckFailed,
                span,
                "Human universe metavariable assignment failed the occurs check",
            ));
        }

        let index = id.0 as usize;
        if let Some(existing) = self.universe_metas[index].assignment.clone() {
            self.add_constraint(HumanConstraint::LevelEq {
                lhs: existing,
                rhs: value,
                span,
            });
        } else {
            self.universe_metas[index].assignment = Some(value);
        }
        Ok(())
    }

    fn resolve_meta_expr(&self, value: HumanMetaExpr) -> HumanMetaExpr {
        match value {
            HumanMetaExpr::Meta(id) => self.term_metas[id.0 as usize]
                .assignment
                .clone()
                .map(|assignment| self.resolve_meta_expr(assignment))
                .unwrap_or(HumanMetaExpr::Meta(id)),
            HumanMetaExpr::App(func, arg) => HumanMetaExpr::App(
                Box::new(self.resolve_meta_expr(*func)),
                Box::new(self.resolve_meta_expr(*arg)),
            ),
            HumanMetaExpr::Core(expr) => HumanMetaExpr::Core(expr),
        }
    }

    fn resolve_meta_level(&self, value: HumanMetaLevel) -> HumanMetaLevel {
        match value {
            HumanMetaLevel::Meta(id) => self.universe_metas[id.0 as usize]
                .assignment
                .clone()
                .map(|assignment| self.resolve_meta_level(assignment))
                .unwrap_or(HumanMetaLevel::Meta(id)),
            HumanMetaLevel::Succ(level) => {
                HumanMetaLevel::Succ(Box::new(self.resolve_meta_level(*level)))
            }
            HumanMetaLevel::Max(lhs, rhs) => HumanMetaLevel::Max(
                Box::new(self.resolve_meta_level(*lhs)),
                Box::new(self.resolve_meta_level(*rhs)),
            ),
            HumanMetaLevel::IMax(lhs, rhs) => HumanMetaLevel::IMax(
                Box::new(self.resolve_meta_level(*lhs)),
                Box::new(self.resolve_meta_level(*rhs)),
            ),
            HumanMetaLevel::Core(level) => HumanMetaLevel::Core(level),
        }
    }

    fn term_meta(&self, id: HumanTermMetaId) -> &HumanTermMeta {
        &self.term_metas[id.0 as usize]
    }

    fn hole_goal(&self, meta: &HumanTermMeta) -> HumanHoleGoal {
        HumanHoleGoal {
            hole: meta.name.clone(),
            context: meta.goal_context.clone(),
            target: meta.target.clone(),
            source_span: meta.span,
        }
    }
}

fn term_constraint(
    kind: HumanTermConstraintKind,
    lhs: HumanMetaExpr,
    rhs: HumanMetaExpr,
    span: Span,
) -> HumanConstraint {
    match kind {
        HumanTermConstraintKind::TypeEq => HumanConstraint::TypeEq { lhs, rhs, span },
        HumanTermConstraintKind::TermEq => HumanConstraint::TermEq { lhs, rhs, span },
    }
}

fn meta_expr_occurs(id: HumanTermMetaId, value: &HumanMetaExpr) -> bool {
    match value {
        HumanMetaExpr::Core(_) => false,
        HumanMetaExpr::Meta(value_id) => *value_id == id,
        HumanMetaExpr::App(func, arg) => meta_expr_occurs(id, func) || meta_expr_occurs(id, arg),
    }
}

fn meta_level_occurs(id: HumanUniverseMetaId, value: &HumanMetaLevel) -> bool {
    match value {
        HumanMetaLevel::Core(_) => false,
        HumanMetaLevel::Meta(value_id) => *value_id == id,
        HumanMetaLevel::Succ(level) => meta_level_occurs(id, level),
        HumanMetaLevel::Max(lhs, rhs) | HumanMetaLevel::IMax(lhs, rhs) => {
            meta_level_occurs(id, lhs) || meta_level_occurs(id, rhs)
        }
    }
}

fn human_universe_meta_level(index: usize) -> Level {
    Level::param(format!("{HUMAN_UNIVERSE_META_PREFIX}{index}"))
}

fn human_universe_meta_index(level: &Level) -> Option<usize> {
    let Level::Param(name) = level else {
        return None;
    };
    name.strip_prefix(HUMAN_UNIVERSE_META_PREFIX)
        .and_then(|suffix| suffix.parse::<usize>().ok())
}

fn human_level_contains_any_universe_meta(level: &Level) -> bool {
    match level {
        Level::Zero => false,
        Level::Succ(inner) => human_level_contains_any_universe_meta(inner),
        Level::Max(lhs, rhs) | Level::IMax(lhs, rhs) => {
            human_level_contains_any_universe_meta(lhs)
                || human_level_contains_any_universe_meta(rhs)
        }
        Level::Param(_) => human_universe_meta_index(level).is_some(),
    }
}

fn level_contains_universe_meta(index: usize, level: &Level) -> bool {
    match level {
        Level::Zero => false,
        Level::Succ(inner) => level_contains_universe_meta(index, inner),
        Level::Max(lhs, rhs) | Level::IMax(lhs, rhs) => {
            level_contains_universe_meta(index, lhs) || level_contains_universe_meta(index, rhs)
        }
        Level::Param(_) => human_universe_meta_index(level) == Some(index),
    }
}

fn expr_contains_spine_implicit(id: HumanSpineImplicitId, expr: &Expr) -> bool {
    if HumanUniverseSpineSolver::implicit_id(expr) == Some(id) {
        return true;
    }
    match expr {
        Expr::Sort(_) | Expr::BVar(_) | Expr::Const { .. } => false,
        Expr::App(fun, arg) => {
            expr_contains_spine_implicit(id, fun) || expr_contains_spine_implicit(id, arg)
        }
        Expr::Lam { ty, body, .. } | Expr::Pi { ty, body, .. } => {
            expr_contains_spine_implicit(id, ty) || expr_contains_spine_implicit(id, body)
        }
        Expr::Let {
            ty, value, body, ..
        } => {
            expr_contains_spine_implicit(id, ty)
                || expr_contains_spine_implicit(id, value)
                || expr_contains_spine_implicit(id, body)
        }
    }
}

fn human_universe_solver_error(span: Span, message: impl Into<String>) -> HumanDiagnostic {
    HumanDiagnostic::error(HumanDiagnosticKind::UnsolvedUniverseMeta, span, message).with_payload(
        HumanDiagnosticPayload {
            unsolved_meta: Some(HumanUnsolvedMeta {
                kind: HumanUnsolvedMetaKind::Universe,
                name: None,
            }),
            ..HumanDiagnosticPayload::default()
        },
    )
}

fn human_meta_placeholder(id: HumanTermMetaId, span: Span) -> MachineTerm {
    MachineTerm::Local {
        name: format!("__human_meta_{}", id.0),
        span,
    }
}

fn canonicalize_machine_term_for_meta_context(term: &MachineTerm) -> MachineTerm {
    let span = meta_context_span();
    match term {
        MachineTerm::Ident {
            name,
            universe_args,
            explicit_mode,
            ..
        } => MachineTerm::Ident {
            name: MachineName {
                parts: name.parts.clone(),
                span,
            },
            universe_args: universe_args.as_ref().map(|args| {
                args.iter()
                    .map(canonicalize_machine_level_for_meta_context)
                    .collect()
            }),
            explicit_mode: *explicit_mode,
            span,
        },
        MachineTerm::Local { name, .. } => MachineTerm::Local {
            name: name.clone(),
            span,
        },
        MachineTerm::Prop { .. } => MachineTerm::Prop { span },
        MachineTerm::Type { level, .. } => MachineTerm::Type {
            level: canonicalize_machine_level_for_meta_context(level),
            span,
        },
        MachineTerm::Sort { level, .. } => MachineTerm::Sort {
            level: canonicalize_machine_level_for_meta_context(level),
            span,
        },
        MachineTerm::App { func, arg, .. } => MachineTerm::App {
            func: Box::new(canonicalize_machine_term_for_meta_context(func)),
            arg: Box::new(canonicalize_machine_term_for_meta_context(arg)),
            span,
        },
        MachineTerm::Lam { binders, body, .. } => MachineTerm::Lam {
            binders: canonicalize_machine_binders_for_meta_context(binders),
            body: Box::new(canonicalize_machine_term_for_meta_context(body)),
            span,
        },
        MachineTerm::Pi { binders, body, .. } => MachineTerm::Pi {
            binders: canonicalize_machine_binders_for_meta_context(binders),
            body: Box::new(canonicalize_machine_term_for_meta_context(body)),
            span,
        },
        MachineTerm::Let {
            name,
            ty,
            value,
            body,
            ..
        } => MachineTerm::Let {
            name: name.clone(),
            ty: Box::new(canonicalize_machine_term_for_meta_context(ty)),
            value: Box::new(canonicalize_machine_term_for_meta_context(value)),
            body: Box::new(canonicalize_machine_term_for_meta_context(body)),
            span,
        },
        MachineTerm::Annot { expr, ty, .. } => MachineTerm::Annot {
            expr: Box::new(canonicalize_machine_term_for_meta_context(expr)),
            ty: Box::new(canonicalize_machine_term_for_meta_context(ty)),
            span,
        },
    }
}

fn canonicalize_machine_binders_for_meta_context(binders: &[MachineBinder]) -> Vec<MachineBinder> {
    let span = meta_context_span();
    binders
        .iter()
        .map(|binder| MachineBinder {
            name: binder.name.clone(),
            ty: canonicalize_machine_term_for_meta_context(&binder.ty),
            span,
        })
        .collect()
}

fn canonicalize_machine_level_for_meta_context(level: &MachineLevel) -> MachineLevel {
    let span = meta_context_span();
    match level {
        MachineLevel::Nat { value, .. } => MachineLevel::Nat {
            value: *value,
            span,
        },
        MachineLevel::Param { name, .. } => MachineLevel::Param {
            name: name.clone(),
            span,
        },
        MachineLevel::Succ { level, .. } => MachineLevel::Succ {
            level: Box::new(canonicalize_machine_level_for_meta_context(level)),
            span,
        },
        MachineLevel::Max { lhs, rhs, .. } => MachineLevel::Max {
            lhs: Box::new(canonicalize_machine_level_for_meta_context(lhs)),
            rhs: Box::new(canonicalize_machine_level_for_meta_context(rhs)),
            span,
        },
        MachineLevel::IMax { lhs, rhs, .. } => MachineLevel::IMax {
            lhs: Box::new(canonicalize_machine_level_for_meta_context(lhs)),
            rhs: Box::new(canonicalize_machine_level_for_meta_context(rhs)),
            span,
        },
    }
}

fn meta_context_span() -> Span {
    Span::empty(crate::FileId(0))
}

fn render_machine_term(term: &MachineTerm) -> String {
    match term {
        MachineTerm::Ident {
            name,
            universe_args,
            explicit_mode,
            ..
        } => {
            let mut rendered = if *explicit_mode {
                format!("@{}", name.as_dotted())
            } else {
                name.as_dotted()
            };
            if let Some(args) = universe_args {
                rendered.push_str(".{");
                rendered.push_str(
                    &args
                        .iter()
                        .map(render_machine_level)
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                rendered.push('}');
            }
            rendered
        }
        MachineTerm::Local { name, .. } => name.clone(),
        MachineTerm::Prop { .. } => "Prop".to_owned(),
        MachineTerm::Type { level, .. } => format!("Type {}", render_machine_level(level)),
        MachineTerm::Sort { level, .. } => format!("Sort {}", render_machine_level(level)),
        MachineTerm::App { func, arg, .. } => {
            format!(
                "({} {})",
                render_machine_term(func),
                render_machine_term(arg)
            )
        }
        MachineTerm::Lam { binders, body, .. } => format!(
            "(fun {} => {})",
            render_machine_binders(binders),
            render_machine_term(body)
        ),
        MachineTerm::Pi { binders, body, .. } => format!(
            "(forall {}, {})",
            render_machine_binders(binders),
            render_machine_term(body)
        ),
        MachineTerm::Let {
            name,
            ty,
            value,
            body,
            ..
        } => format!(
            "(let {name} : {} := {} in {})",
            render_machine_term(ty),
            render_machine_term(value),
            render_machine_term(body)
        ),
        MachineTerm::Annot { expr, ty, .. } => {
            format!(
                "({} : {})",
                render_machine_term(expr),
                render_machine_term(ty)
            )
        }
    }
}

fn render_machine_binders(binders: &[MachineBinder]) -> String {
    binders
        .iter()
        .map(|binder| format!("({} : {})", binder.name, render_machine_term(&binder.ty)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_machine_level(level: &MachineLevel) -> String {
    match level {
        MachineLevel::Nat { value, .. } => value.to_string(),
        MachineLevel::Param { name, .. } => name.clone(),
        MachineLevel::Succ { level, .. } => format!("succ {}", render_machine_level(level)),
        MachineLevel::Max { lhs, rhs, .. } => {
            format!(
                "max {} {}",
                render_machine_level(lhs),
                render_machine_level(rhs)
            )
        }
        MachineLevel::IMax { lhs, rhs, .. } => {
            format!(
                "imax {} {}",
                render_machine_level(lhs),
                render_machine_level(rhs)
            )
        }
    }
}

fn human_level_leq(lhs: &Level, rhs: &Level) -> bool {
    match (
        core_level_as_u64(&npa_kernel::level::normalize_level(lhs.clone())),
        core_level_as_u64(&npa_kernel::level::normalize_level(rhs.clone())),
    ) {
        (Some(lhs), Some(rhs)) => lhs <= rhs,
        _ => lhs == rhs,
    }
}

#[derive(Clone, Debug)]
struct HumanElaboratedBinder {
    name: String,
    ty: Expr,
}

#[derive(Clone, Debug)]
struct HumanLocalDecl {
    name: String,
    ty: Expr,
    value: Option<Expr>,
}

#[derive(Clone, Debug, Default)]
struct HumanLocalContext {
    locals: Vec<HumanLocalDecl>,
}

impl HumanLocalContext {
    fn push_assumption(&mut self, name: String, ty: Expr) {
        self.locals.push(HumanLocalDecl {
            name,
            ty,
            value: None,
        });
    }

    fn push_definition(&mut self, name: String, ty: Expr, value: Expr) {
        self.locals.push(HumanLocalDecl {
            name,
            ty,
            value: Some(value),
        });
    }

    fn lookup_bvar(&self, name: &str) -> Option<u32> {
        self.locals
            .iter()
            .rev()
            .position(|local| local.name == name)
            .map(|index| index as u32)
    }

    fn lookup(&self, name: &str, span: Span) -> HumanResult<(u32, Expr)> {
        let Some((index, local)) = self
            .locals
            .iter()
            .rev()
            .enumerate()
            .find(|(_, local)| local.name == name)
        else {
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::UnknownIdentifier,
                span,
                format!("unknown local name {name}"),
            ));
        };
        let index = index as u32;
        let ty = subst::shift(&local.ty, index as i32 + 1, 0)
            .map_err(|err| human_kernel_expr_diagnostic(span, err, "local type lookup"))?;
        Ok((index, ty))
    }

    fn name_for_bvar(&self, index: u32) -> Option<&str> {
        let index = usize::try_from(index).ok()?;
        self.locals
            .len()
            .checked_sub(index + 1)
            .and_then(|local_index| self.locals.get(local_index))
            .map(|local| local.name.as_str())
    }

    fn to_kernel_ctx(&self) -> Ctx {
        let mut ctx = Ctx::new();
        for local in &self.locals {
            match &local.value {
                Some(value) => {
                    ctx.push_definition(local.name.clone(), local.ty.clone(), value.clone())
                }
                None => ctx.push_assumption(local.name.clone(), local.ty.clone()),
            }
        }
        ctx
    }
}

fn human_local_context_from_machine(locals: &[MachineLocalDecl]) -> HumanLocalContext {
    let mut context = HumanLocalContext::default();
    for local in locals {
        match &local.value {
            Some(value) => {
                context.push_definition(local.name.clone(), local.ty.clone(), value.clone())
            }
            None => context.push_assumption(local.name.clone(), local.ty.clone()),
        }
    }
    context
}

struct HumanBidirectionalElaborator {
    env: Env,
}

impl HumanBidirectionalElaborator {
    fn new(module: &ResolvedHumanModule, verified_imports: &[VerifiedImport]) -> HumanResult<Self> {
        let mut elaborator = Self { env: Env::new() };

        let active_imports = active_human_imports(module, verified_imports)?;
        for import in active_imports {
            elaborator.add_import(import, module.module.span)?;
        }
        let builtin_names = human_referenced_builtin_names(module);
        add_human_builtin_decls_for_names(
            &mut elaborator.env,
            &builtin_names,
            module.module.span,
            "Human resolved builtin",
        )?;

        Ok(elaborator)
    }

    fn from_tactic_context(context: &HumanTacticTermElabContext) -> Self {
        Self {
            env: context.env.clone(),
        }
    }

    fn elaborate_module(
        mut self,
        module_name: npa_cert::ModuleName,
        module: HumanLoweredModule,
    ) -> HumanResult<npa_cert::CoreModule> {
        let mut declarations = Vec::new();

        for item in module.items {
            match item {
                HumanLoweredItem::Import => {}
                HumanLoweredItem::Def(decl) => {
                    let span = decl.span;
                    let decl = self.elaborate_decl(decl, HumanLoweredDeclKind::Def)?;
                    self.add_kernel_decl(decl.clone(), span)?;
                    declarations.push(decl);
                }
                HumanLoweredItem::Theorem(decl) => {
                    let span = decl.span;
                    let decl = self.elaborate_decl(decl, HumanLoweredDeclKind::Theorem)?;
                    self.add_kernel_decl(decl.clone(), span)?;
                    declarations.push(decl);
                }
                HumanLoweredItem::TheoremCoreProof { decl, span } => {
                    self.add_kernel_decl(decl.clone(), span)?;
                    declarations.push(decl);
                }
                HumanLoweredItem::Axiom(decl) => {
                    let span = decl.span;
                    let decl = self.elaborate_axiom_decl(decl)?;
                    self.add_kernel_decl(decl.clone(), span)?;
                    declarations.push(decl);
                }
                HumanLoweredItem::Inductive(decl) => {
                    let span = decl.span;
                    let decl = self.elaborate_inductive_decl(decl)?;
                    self.add_kernel_decl(decl.clone(), span)?;
                    declarations.push(decl);
                }
            }
        }

        Ok(npa_cert::CoreModule {
            name: module_name,
            declarations,
        })
    }

    fn elaborate_proof_start_core(
        mut self,
        module_name: npa_cert::ModuleName,
        proof_start: HumanLoweredProofStart,
    ) -> HumanResult<HumanProofStartCore> {
        let mut prior_declarations = Vec::new();

        for item in proof_start.prior_items {
            match item {
                HumanLoweredItem::Import => {}
                HumanLoweredItem::Def(decl) => {
                    let span = decl.span;
                    let decl = self.elaborate_decl(decl, HumanLoweredDeclKind::Def)?;
                    self.add_kernel_decl(decl.clone(), span)?;
                    prior_declarations.push(decl);
                }
                HumanLoweredItem::Theorem(decl) => {
                    let span = decl.span;
                    let decl = self.elaborate_decl(decl, HumanLoweredDeclKind::Theorem)?;
                    self.add_kernel_decl(decl.clone(), span)?;
                    prior_declarations.push(decl);
                }
                HumanLoweredItem::TheoremCoreProof { decl, span } => {
                    self.add_kernel_decl(decl.clone(), span)?;
                    prior_declarations.push(decl);
                }
                HumanLoweredItem::Axiom(decl) => {
                    let span = decl.span;
                    let decl = self.elaborate_axiom_decl(decl)?;
                    self.add_kernel_decl(decl.clone(), span)?;
                    prior_declarations.push(decl);
                }
                HumanLoweredItem::Inductive(decl) => {
                    let span = decl.span;
                    let decl = self.elaborate_inductive_decl(decl)?;
                    self.add_kernel_decl(decl.clone(), span)?;
                    prior_declarations.push(decl);
                }
            }
        }

        let theorem_name = npa_cert::Name(proof_start.target.name.parts.clone());
        let (universe_params, theorem_type) =
            self.elaborate_decl_signature_type(&proof_start.target)?;

        Ok(HumanProofStartCore {
            module: module_name,
            theorem_name,
            source_index: proof_start.source_index,
            universe_params,
            theorem_type,
            prior_declarations,
        })
    }

    fn elaborate_decl_signature_type(
        &self,
        decl: &HumanLoweredDeclSignature,
    ) -> HumanResult<(Vec<String>, Expr)> {
        let delta: Vec<_> = decl
            .universe_params
            .iter()
            .map(|param| param.name.clone())
            .collect();
        let mut locals = HumanLocalContext::default();
        let mut elaborated_binders = Vec::with_capacity(decl.binders.len());

        for binder in &decl.binders {
            let (ty, ty_type) = self.infer_human_expr(&binder.ty, &locals, &delta)?;
            self.expect_human_sort(&ty_type, &locals, &delta, binder.ty.span())?;
            locals.push_assumption(binder.name.clone(), ty.clone());
            elaborated_binders.push(HumanElaboratedBinder {
                name: binder.name.clone(),
                ty,
            });
        }

        let (ty, ty_type) = self.infer_human_expr(&decl.ty, &locals, &delta)?;
        self.expect_human_sort(&ty_type, &locals, &delta, decl.ty.span())?;

        Ok((delta, human_close_pi(&elaborated_binders, ty)))
    }

    fn elaborate_axiom_decl(&self, decl: HumanLoweredAxiomDecl) -> HumanResult<Decl> {
        let delta: Vec<_> = decl
            .universe_params
            .iter()
            .map(|param| param.name.clone())
            .collect();
        let mut locals = HumanLocalContext::default();
        let mut elaborated_binders = Vec::with_capacity(decl.binders.len());

        for binder in &decl.binders {
            let (ty, ty_type) = self.infer_human_expr(&binder.ty, &locals, &delta)?;
            self.expect_human_sort(&ty_type, &locals, &delta, binder.ty.span())?;
            locals.push_assumption(binder.name.clone(), ty.clone());
            elaborated_binders.push(HumanElaboratedBinder {
                name: binder.name.clone(),
                ty,
            });
        }

        let (ty, ty_type) = self.infer_human_expr(&decl.ty, &locals, &delta)?;
        self.expect_human_sort(&ty_type, &locals, &delta, decl.ty.span())?;

        Ok(Decl::Axiom {
            name: decl.name.as_dotted(),
            universe_params: delta,
            ty: human_close_pi(&elaborated_binders, ty),
        })
    }

    fn elaborate_inductive_decl(&self, decl: HumanLoweredInductiveDecl) -> HumanResult<Decl> {
        let delta: Vec<_> = decl
            .universe_params
            .iter()
            .map(|param| param.name.clone())
            .collect();
        let mut locals = HumanLocalContext::default();
        let mut params = Vec::with_capacity(decl.binders.len());

        for binder in &decl.binders {
            let (ty, ty_type) = self.infer_human_expr(&binder.ty, &locals, &delta)?;
            self.expect_human_sort(&ty_type, &locals, &delta, binder.ty.span())?;
            locals.push_assumption(binder.name.clone(), ty.clone());
            params.push(HumanElaboratedBinder {
                name: binder.name.clone(),
                ty,
            });
        }

        let (result_ty, result_ty_type) = self.infer_human_expr(&decl.ty, &locals, &delta)?;
        self.expect_human_sort(&result_ty_type, &locals, &delta, decl.ty.span())?;
        let (indices, sort) =
            split_inductive_result_type(&self.env, result_ty, &locals, &delta, decl.ty.span())?;
        let head_ty = human_inductive_head_type(&params, &indices, sort.clone());
        let name = decl.name.as_dotted();

        let mut temporary = Self {
            env: self.env.clone(),
        };
        temporary.add_kernel_decl(
            Decl::Axiom {
                name: name.clone(),
                universe_params: delta.clone(),
                ty: head_ty.clone(),
            },
            decl.span,
        )?;

        let mut constructors = Vec::with_capacity(decl.constructors.len());
        for constructor in &decl.constructors {
            let (ty, ty_type) = temporary.infer_human_expr(&constructor.ty, &locals, &delta)?;
            temporary.expect_human_sort(&ty_type, &locals, &delta, constructor.ty.span())?;
            constructors.push(ConstructorDecl::new(
                constructor.name.as_dotted(),
                human_close_pi(&params, ty),
            ));
        }

        let data = finalize_human_inductive_data(
            name.clone(),
            delta.clone(),
            params.iter().map(kernel_binder_from_human).collect(),
            indices.iter().map(kernel_binder_from_human).collect(),
            sort,
            constructors,
        );

        Ok(Decl::Inductive {
            name,
            universe_params: delta,
            ty: head_ty,
            data: Box::new(data),
        })
    }

    fn add_import(&mut self, import: &VerifiedImport, span: Span) -> HumanResult<()> {
        for decl in kernel_decls_for_human_import(import) {
            self.add_kernel_decl(decl, span)?;
        }
        Ok(())
    }

    fn elaborate_decl(&self, decl: MachineDecl, kind: HumanLoweredDeclKind) -> HumanResult<Decl> {
        let delta: Vec<_> = decl
            .universe_params
            .iter()
            .map(|param| param.name.clone())
            .collect();
        let mut locals = HumanLocalContext::default();
        let mut elaborated_binders = Vec::with_capacity(decl.binders.len());

        for binder in &decl.binders {
            let (ty, ty_type) = self.infer_human_expr(&binder.ty, &locals, &delta)?;
            self.expect_human_sort(&ty_type, &locals, &delta, binder.ty.span())?;
            locals.push_assumption(binder.name.clone(), ty.clone());
            elaborated_binders.push(HumanElaboratedBinder {
                name: binder.name.clone(),
                ty,
            });
        }

        let (ty, ty_type) = self.infer_human_expr(&decl.ty, &locals, &delta)?;
        self.expect_human_sort(&ty_type, &locals, &delta, decl.ty.span())?;
        let value = self.check_human_expr(&decl.value, &ty, &locals, &delta)?;

        let name = decl.name.as_dotted();
        let closed_ty = human_close_pi(&elaborated_binders, ty);
        let closed_value = human_close_lam(&elaborated_binders, value);
        let universe_params = delta;

        Ok(match kind {
            HumanLoweredDeclKind::Def => Decl::Def {
                name,
                universe_params,
                ty: closed_ty,
                value: closed_value,
                reducibility: Reducibility::Reducible,
            },
            HumanLoweredDeclKind::Theorem => Decl::Theorem {
                name,
                universe_params,
                ty: closed_ty,
                proof: closed_value,
            },
        })
    }

    fn infer_human_expr(
        &self,
        term: &MachineTerm,
        locals: &HumanLocalContext,
        delta: &[String],
    ) -> HumanResult<(Expr, Expr)> {
        Ok(match term {
            MachineTerm::Ident {
                name,
                universe_args,
                span,
                ..
            } => {
                let expr = self.elaborate_human_global(name, universe_args.as_deref(), *span)?;
                let ty = self.infer_core_expr_type(&expr, locals, delta, *span)?;
                (expr, ty)
            }
            MachineTerm::Local { name, span } => {
                let (index, ty) = locals.lookup(name, *span)?;
                (Expr::bvar(index), ty)
            }
            MachineTerm::Prop { .. } => (
                Expr::sort(Level::zero()),
                Expr::sort(Level::succ(Level::zero())),
            ),
            MachineTerm::Type { level, .. } => {
                let level = elaborate_machine_level(level.clone())?;
                let sort = Level::succ(level);
                (Expr::sort(sort.clone()), Expr::sort(Level::succ(sort)))
            }
            MachineTerm::Sort { level, .. } => {
                let level = elaborate_machine_level(level.clone())?;
                (Expr::sort(level.clone()), Expr::sort(Level::succ(level)))
            }
            MachineTerm::App { func, arg, span } => {
                let (func_expr, func_ty) = self.infer_human_expr(func, locals, delta)?;
                let func_ty = self.whnf_human_expr(&func_ty, locals, delta, *span)?;
                let Expr::Pi { ty, body, .. } = func_ty else {
                    return Err(HumanDiagnostic::error(
                        HumanDiagnosticKind::ExpectedFunctionType,
                        *span,
                        format!("application head is not a function: {func_ty:?}"),
                    ));
                };
                let arg_expr = self.check_human_expr(arg, &ty, locals, delta)?;
                let result_ty = subst::instantiate(&body, &arg_expr).map_err(|err| {
                    human_kernel_expr_diagnostic(*span, err, "Human application result type")
                })?;
                (Expr::app(func_expr, arg_expr), result_ty)
            }
            MachineTerm::Lam {
                binders,
                body,
                span: _,
            } => {
                let mut nested = locals.clone();
                let mut elaborated_binders = Vec::with_capacity(binders.len());
                for binder in binders {
                    let (ty, ty_type) = self.infer_human_expr(&binder.ty, &nested, delta)?;
                    self.expect_human_sort(&ty_type, &nested, delta, binder.ty.span())?;
                    nested.push_assumption(binder.name.clone(), ty.clone());
                    elaborated_binders.push(HumanElaboratedBinder {
                        name: binder.name.clone(),
                        ty,
                    });
                }
                let (body, body_ty) = self.infer_human_expr(body, &nested, delta)?;
                (
                    human_close_lam(&elaborated_binders, body),
                    human_close_pi(&elaborated_binders, body_ty),
                )
            }
            MachineTerm::Pi {
                binders,
                body,
                span,
            } => {
                let mut nested = locals.clone();
                let mut elaborated_binders = Vec::with_capacity(binders.len());
                for binder in binders {
                    let (ty, ty_type) = self.infer_human_expr(&binder.ty, &nested, delta)?;
                    self.expect_human_sort(&ty_type, &nested, delta, binder.ty.span())?;
                    nested.push_assumption(binder.name.clone(), ty.clone());
                    elaborated_binders.push(HumanElaboratedBinder {
                        name: binder.name.clone(),
                        ty,
                    });
                }
                let body_span = body.span();
                let (body_expr, body_type) = self.infer_human_expr(body, &nested, delta)?;
                self.expect_human_sort(&body_type, &nested, delta, body_span)?;
                let pi = human_close_pi(&elaborated_binders, body_expr);
                let pi_ty = self.infer_core_expr_type(&pi, locals, delta, *span)?;
                (pi, pi_ty)
            }
            MachineTerm::Let {
                name,
                ty,
                value,
                body,
                span,
            } => {
                let (ty_expr, ty_type) = self.infer_human_expr(ty, locals, delta)?;
                self.expect_human_sort(&ty_type, locals, delta, ty.span())?;
                let value_expr = self.check_human_expr(value, &ty_expr, locals, delta)?;
                let mut nested = locals.clone();
                nested.push_definition(name.clone(), ty_expr.clone(), value_expr.clone());
                let (body_expr, body_ty) = self.infer_human_expr(body, &nested, delta)?;
                let result_ty = subst::instantiate(&body_ty, &value_expr).map_err(|err| {
                    human_kernel_expr_diagnostic(*span, err, "Human let result type")
                })?;
                (
                    Expr::let_in(name.clone(), ty_expr, value_expr, body_expr),
                    result_ty,
                )
            }
            MachineTerm::Annot { expr, ty, span: _ } => {
                let (ty_expr, ty_type) = self.infer_human_expr(ty, locals, delta)?;
                self.expect_human_sort(&ty_type, locals, delta, ty.span())?;
                let expr = self.check_human_expr(expr, &ty_expr, locals, delta)?;
                (expr, ty_expr)
            }
        })
    }

    fn check_human_expr(
        &self,
        term: &MachineTerm,
        expected: &Expr,
        locals: &HumanLocalContext,
        delta: &[String],
    ) -> HumanResult<Expr> {
        if let MachineTerm::Lam {
            binders,
            body,
            span: _,
        } = term
        {
            return self.check_human_lambda(binders, body, expected, locals, delta);
        }

        let (expr, actual) = self.infer_human_expr(term, locals, delta)?;
        if self.is_human_defeq(&actual, expected, locals, delta, term.span())? {
            Ok(expr)
        } else {
            Err(HumanDiagnostic::error(
                HumanDiagnosticKind::TypeMismatch,
                term.span(),
                format!("type mismatch: expected {expected:?}, got {actual:?}"),
            ))
        }
    }

    fn check_human_lambda(
        &self,
        binders: &[MachineBinder],
        body: &MachineTerm,
        expected: &Expr,
        locals: &HumanLocalContext,
        delta: &[String],
    ) -> HumanResult<Expr> {
        let mut nested = locals.clone();
        let mut expected = expected.clone();
        let mut elaborated_binders = Vec::with_capacity(binders.len());

        for binder in binders {
            let expected_whnf = self.whnf_human_expr(&expected, &nested, delta, binder.span)?;
            let Expr::Pi { ty, body, .. } = expected_whnf else {
                return Err(HumanDiagnostic::error(
                    HumanDiagnosticKind::ExpectedFunctionType,
                    binder.span,
                    format!("lambda is checked against a non-function type: {expected_whnf:?}"),
                ));
            };
            let (binder_ty, binder_ty_type) = self.infer_human_expr(&binder.ty, &nested, delta)?;
            self.expect_human_sort(&binder_ty_type, &nested, delta, binder.ty.span())?;
            if !self.is_human_defeq(&binder_ty, &ty, &nested, delta, binder.ty.span())? {
                return Err(HumanDiagnostic::error(
                    HumanDiagnosticKind::TypeMismatch,
                    binder.ty.span(),
                    format!("lambda binder type mismatch: expected {ty:?}, got {binder_ty:?}"),
                ));
            }
            nested.push_assumption(binder.name.clone(), (*ty).clone());
            elaborated_binders.push(HumanElaboratedBinder {
                name: binder.name.clone(),
                ty: (*ty).clone(),
            });
            expected = *body;
        }

        let body = self.check_human_expr(body, &expected, &nested, delta)?;
        Ok(human_close_lam(&elaborated_binders, body))
    }

    fn elaborate_human_global(
        &self,
        name: &MachineName,
        universe_args: Option<&[MachineLevel]>,
        span: Span,
    ) -> HumanResult<Expr> {
        let name = name.as_dotted();
        let Some(decl) = self.env.decl(&name) else {
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::UnknownIdentifier,
                span,
                format!("unknown global name {name}"),
            ));
        };
        let expected = decl.universe_params().len();
        let levels = match universe_args {
            Some(args) if args.len() == expected => args
                .iter()
                .cloned()
                .map(elaborate_machine_level)
                .collect::<HumanResult<Vec<_>>>()?,
            Some(args) => {
                return Err(HumanDiagnostic::error(
                    HumanDiagnosticKind::UnsolvedUniverseMeta,
                    span,
                    format!(
                        "global name {name} expects {expected} universe arguments, got {}",
                        args.len()
                    ),
                )
                .with_payload(HumanDiagnosticPayload {
                    unsolved_meta: Some(HumanUnsolvedMeta {
                        kind: HumanUnsolvedMetaKind::Universe,
                        name: Some(name.clone()),
                    }),
                    ..HumanDiagnosticPayload::default()
                }));
            }
            None if expected == 0 => Vec::new(),
            None => {
                return Err(HumanDiagnostic::error(
                    HumanDiagnosticKind::UnsolvedUniverseMeta,
                    span,
                    format!("global name {name} still has unresolved universe arguments"),
                )
                .with_payload(HumanDiagnosticPayload {
                    unsolved_meta: Some(HumanUnsolvedMeta {
                        kind: HumanUnsolvedMetaKind::Universe,
                        name: Some(name.clone()),
                    }),
                    ..HumanDiagnosticPayload::default()
                }));
            }
        };
        Ok(Expr::konst(name, levels))
    }

    fn infer_core_expr_type(
        &self,
        expr: &Expr,
        locals: &HumanLocalContext,
        delta: &[String],
        span: Span,
    ) -> HumanResult<Expr> {
        self.env
            .infer(&locals.to_kernel_ctx(), delta, expr)
            .map_err(|err| human_kernel_expr_diagnostic(span, err, "Human expression inference"))
    }

    fn expect_human_sort(
        &self,
        inferred_type: &Expr,
        locals: &HumanLocalContext,
        delta: &[String],
        span: Span,
    ) -> HumanResult<()> {
        let whnf = self.whnf_human_expr(inferred_type, locals, delta, span)?;
        if matches!(whnf, Expr::Sort(_)) {
            Ok(())
        } else {
            Err(HumanDiagnostic::error(
                HumanDiagnosticKind::ExpectedSort,
                span,
                format!("expected a type, got {whnf:?}"),
            ))
        }
    }

    fn whnf_human_expr(
        &self,
        expr: &Expr,
        locals: &HumanLocalContext,
        delta: &[String],
        span: Span,
    ) -> HumanResult<Expr> {
        self.env
            .whnf(&locals.to_kernel_ctx(), delta, expr)
            .map_err(|err| human_kernel_expr_diagnostic(span, err, "Human weak-head reduction"))
    }

    fn is_human_defeq(
        &self,
        lhs: &Expr,
        rhs: &Expr,
        locals: &HumanLocalContext,
        delta: &[String],
        span: Span,
    ) -> HumanResult<bool> {
        self.env
            .is_defeq(&locals.to_kernel_ctx(), delta, lhs, rhs)
            .map_err(|err| human_kernel_expr_diagnostic(span, err, "Human definitional equality"))
    }

    fn add_kernel_decl(&mut self, decl: Decl, span: Span) -> HumanResult<()> {
        if let Some(existing) = self.env.decl(decl.name()) {
            if existing == &decl {
                return Ok(());
            }
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::KernelRejected,
                span,
                format!(
                    "kernel declaration {} conflicts with an existing declaration",
                    decl.name()
                ),
            )
            .with_phase(HumanDiagnosticPhase::KernelHandoff));
        }

        add_referenced_builtin_decls_to_human_env(
            &mut self.env,
            &decl,
            span,
            "Human declaration handoff",
        )?;

        match decl {
            Decl::Axiom {
                name,
                universe_params,
                ty,
            } => self.env.add_axiom(name, universe_params, ty),
            Decl::AxiomConstrained {
                name,
                universe_params,
                universe_constraints,
                ty,
            } => self.env.add_axiom_with_universe_constraints(
                name,
                universe_params,
                universe_constraints,
                ty,
            ),
            Decl::Def {
                name,
                universe_params,
                ty,
                value,
                reducibility,
            } => self
                .env
                .add_def(name, universe_params, ty, value, reducibility),
            Decl::DefConstrained {
                name,
                universe_params,
                universe_constraints,
                ty,
                value,
                reducibility,
            } => self.env.add_def_with_universe_constraints(
                name,
                universe_params,
                universe_constraints,
                ty,
                value,
                reducibility,
            ),
            Decl::Theorem {
                name,
                universe_params,
                ty,
                proof,
            } => self.env.add_theorem(name, universe_params, ty, proof),
            Decl::TheoremConstrained {
                name,
                universe_params,
                universe_constraints,
                ty,
                proof,
            } => self.env.add_theorem_with_universe_constraints(
                name,
                universe_params,
                universe_constraints,
                ty,
                proof,
            ),
            Decl::Inductive { data, .. } => self.env.add_inductive(*data),
            Decl::Constructor { .. } | Decl::Recursor { .. } => Ok(()),
        }
        .map_err(|err| {
            human_kernel_decl_diagnostic(span, err, "Human declaration handoff")
                .with_phase(HumanDiagnosticPhase::KernelHandoff)
        })
    }
}

#[derive(Clone, Debug, Default)]
struct HumanLoweringLocalContext {
    locals: Vec<HumanLoweringLocalDecl>,
}

#[derive(Clone, Debug)]
struct HumanLoweringLocalDecl {
    name: String,
    ty: MachineTerm,
    value: Option<MachineTerm>,
}

impl HumanLoweringLocalContext {
    fn push_assumption(&mut self, name: String, ty: MachineTerm) {
        self.locals.push(HumanLoweringLocalDecl {
            name,
            ty,
            value: None,
        });
    }

    fn push_definition(&mut self, name: String, ty: MachineTerm, value: MachineTerm) {
        self.locals.push(HumanLoweringLocalDecl {
            name,
            ty,
            value: Some(value),
        });
    }

    fn from_machine_locals(locals: &[MachineLocalDecl]) -> Self {
        let span = Span::empty(crate::FileId(0));
        let mut lowering = Self::default();
        let mut core_locals = HumanLocalContext::default();
        for local in locals {
            let ty = core_expr_to_machine_term(&local.ty, &core_locals, span)
                .unwrap_or_else(|| human_tactic_meta_fallback_machine_term(span));
            match &local.value {
                Some(value) => {
                    let value_term = core_expr_to_machine_term(value, &core_locals, span)
                        .unwrap_or_else(|| human_tactic_meta_fallback_machine_term(span));
                    lowering.push_definition(local.name.clone(), ty.clone(), value_term);
                    core_locals.push_definition(
                        local.name.clone(),
                        local.ty.clone(),
                        value.clone(),
                    );
                }
                None => {
                    lowering.push_assumption(local.name.clone(), ty);
                    core_locals.push_assumption(local.name.clone(), local.ty.clone());
                }
            }
        }
        lowering
    }

    fn meta_snapshot(&self) -> HumanMetaContextSnapshot {
        HumanMetaContextSnapshot {
            locals: self
                .locals
                .iter()
                .map(|local| HumanMetaLocalSnapshot {
                    ty: canonicalize_machine_term_for_meta_context(&local.ty),
                    value: local
                        .value
                        .as_ref()
                        .map(canonicalize_machine_term_for_meta_context),
                })
                .collect(),
        }
    }

    fn hole_goal_context(&self) -> Vec<HumanHoleGoalLocal> {
        self.locals
            .iter()
            .map(|local| HumanHoleGoalLocal {
                name: local.name.clone(),
                ty: render_machine_term(&local.ty),
                value: local.value.as_ref().map(render_machine_term),
            })
            .collect()
    }
}

#[derive(Clone)]
struct HumanImplicitInserter {
    env: Env,
    signatures: BTreeMap<String, HumanCallableSignature>,
    imported_source_interfaces: Vec<HumanImportedSourceInterface>,
    insertion_steps: usize,
}

impl HumanImplicitInserter {
    fn new(module: &ResolvedHumanModule, verified_imports: &[VerifiedImport]) -> HumanResult<Self> {
        let mut inserter = Self {
            env: Env::new(),
            signatures: BTreeMap::new(),
            imported_source_interfaces: module.state.source_interfaces.imports.clone(),
            insertion_steps: 0,
        };

        let active_imports = active_human_imports(module, verified_imports)?;
        for import in active_imports {
            inserter.add_import(import, module.module.span)?;
        }
        inserter.add_referenced_builtins(module)?;

        Ok(inserter)
    }

    fn from_tactic_context(context: &HumanTacticTermElabContext) -> Self {
        Self {
            env: context.env.clone(),
            signatures: context.signatures.clone(),
            imported_source_interfaces: Vec::new(),
            insertion_steps: 0,
        }
    }

    fn add_import(&mut self, import: &VerifiedImport, span: Span) -> HumanResult<()> {
        for decl in kernel_decls_for_human_import(import) {
            self.add_kernel_decl(decl, span)?;
        }
        for export in &import.exports {
            let implicit_profile = self
                .imported_source_interface_profile(import, export)
                .unwrap_or_else(|| {
                    if npa_cert::builtin_decl_interface_hash(&export.name)
                        == Some(export.decl_interface_hash)
                    {
                        builtin_machine_callable_profile(&export.name).unwrap_or_default()
                    } else {
                        Vec::new()
                    }
                });
            self.signatures.insert(
                export.name.as_dotted(),
                HumanCallableSignature {
                    universe_params: export.universe_params.clone(),
                    implicit_profile,
                },
            );
        }
        Ok(())
    }

    fn add_referenced_builtins(&mut self, module: &ResolvedHumanModule) -> HumanResult<()> {
        let builtin_names = human_referenced_builtin_names(module);
        add_human_builtin_decls_for_names(
            &mut self.env,
            &builtin_names,
            module.module.span,
            "Human implicit builtin",
        )?;
        for name in builtin_names {
            let dotted = name.as_dotted();
            let Some(decl) = self.env.decl(&dotted) else {
                continue;
            };
            self.signatures.insert(
                dotted,
                HumanCallableSignature {
                    universe_params: decl.universe_params().to_vec(),
                    implicit_profile: builtin_machine_callable_profile(&name).unwrap_or_default(),
                },
            );
        }
        Ok(())
    }

    fn imported_source_interface_profile(
        &self,
        import: &VerifiedImport,
        export: &crate::VerifiedExport,
    ) -> Option<Vec<MachineCallableBinderVisibility>> {
        human_source_interface_profile_for_export(&self.imported_source_interfaces, import, export)
    }

    fn insert_decl(
        &mut self,
        mut decl: MachineDecl,
        metadata: &HumanSourceDeclarationMetadata,
        kind: HumanLoweredDeclKind,
    ) -> HumanResult<MachineDecl> {
        let delta: Vec<_> = decl
            .universe_params
            .iter()
            .map(|param| param.name.clone())
            .collect();
        let mut locals = HumanLocalContext::default();
        let mut elaborated_binders = Vec::with_capacity(decl.binders.len());
        let mut transformed_binders = Vec::with_capacity(decl.binders.len());

        for binder in decl.binders {
            let ty = self.insert_term(binder.ty, &mut locals, &delta)?;
            let ty_expr = self.elaborate_machine_term(&ty, &locals, &delta)?;
            locals.push_assumption(binder.name.clone(), ty_expr.clone());
            elaborated_binders.push(HumanElaboratedBinder {
                name: binder.name.clone(),
                ty: ty_expr,
            });
            transformed_binders.push(MachineBinder {
                name: binder.name,
                ty,
                span: binder.span,
            });
        }

        decl.binders = transformed_binders;
        decl.ty = self.insert_term(decl.ty, &mut locals, &delta)?;
        decl.value = self.insert_term(decl.value, &mut locals, &delta)?;

        let ty_expr = self.elaborate_machine_term(&decl.ty, &locals, &delta)?;
        let value_expr = self.elaborate_machine_term(&decl.value, &locals, &delta)?;
        let closed_ty = human_close_pi(&elaborated_binders, ty_expr);
        let closed_value = human_close_lam(&elaborated_binders, value_expr);
        let name = decl.name.as_dotted();
        let universe_params = delta.clone();
        let core_decl = match kind {
            HumanLoweredDeclKind::Def => Decl::Def {
                name: name.clone(),
                universe_params,
                ty: closed_ty,
                value: closed_value,
                reducibility: Reducibility::Reducible,
            },
            HumanLoweredDeclKind::Theorem => Decl::Theorem {
                name: name.clone(),
                universe_params,
                ty: closed_ty,
                proof: closed_value,
            },
        };
        self.add_kernel_decl(core_decl, decl.span)?;
        self.signatures.insert(
            name,
            HumanCallableSignature {
                universe_params: delta,
                implicit_profile: machine_callable_profile_from_human_binders(&metadata.binders),
            },
        );

        Ok(decl)
    }

    fn insert_decl_signature(
        &mut self,
        mut decl: HumanLoweredDeclSignature,
    ) -> HumanResult<HumanLoweredDeclSignature> {
        let delta: Vec<_> = decl
            .universe_params
            .iter()
            .map(|param| param.name.clone())
            .collect();
        let mut locals = HumanLocalContext::default();
        let mut transformed_binders = Vec::with_capacity(decl.binders.len());

        for binder in decl.binders {
            let ty = self.insert_term(binder.ty, &mut locals, &delta)?;
            let ty_expr = self.elaborate_machine_term(&ty, &locals, &delta)?;
            locals.push_assumption(binder.name.clone(), ty_expr);
            transformed_binders.push(MachineBinder {
                name: binder.name,
                ty,
                span: binder.span,
            });
        }

        decl.binders = transformed_binders;
        decl.ty = self.insert_term(decl.ty, &mut locals, &delta)?;
        let _ = self.elaborate_machine_term(&decl.ty, &locals, &delta)?;

        Ok(decl)
    }

    fn insert_core_theorem_decl(
        &mut self,
        decl: HumanLoweredDeclSignature,
        metadata: &HumanSourceDeclarationMetadata,
        proof: Expr,
        span: Span,
    ) -> HumanResult<Decl> {
        let decl = self.insert_decl_signature(decl)?;
        let delta: Vec<_> = decl
            .universe_params
            .iter()
            .map(|param| param.name.clone())
            .collect();
        let mut locals = HumanLocalContext::default();
        let mut elaborated_binders = Vec::with_capacity(decl.binders.len());

        for binder in &decl.binders {
            let ty = self.elaborate_machine_term(&binder.ty, &locals, &delta)?;
            let ty_type = self.infer_core_expr_type(&ty, &locals, &delta, binder.ty.span())?;
            self.expect_core_sort(&ty_type, &locals, &delta, binder.ty.span())?;
            locals.push_assumption(binder.name.clone(), ty.clone());
            elaborated_binders.push(HumanElaboratedBinder {
                name: binder.name.clone(),
                ty,
            });
        }

        let ty = self.elaborate_machine_term(&decl.ty, &locals, &delta)?;
        let ty_type = self.infer_core_expr_type(&ty, &locals, &delta, decl.ty.span())?;
        self.expect_core_sort(&ty_type, &locals, &delta, decl.ty.span())?;
        let core_decl = Decl::Theorem {
            name: decl.name.as_dotted(),
            universe_params: delta.clone(),
            ty: human_close_pi(&elaborated_binders, ty),
            proof,
        };
        self.add_kernel_decl(core_decl.clone(), span)?;
        self.signatures.insert(
            decl.name.as_dotted(),
            HumanCallableSignature {
                universe_params: delta,
                implicit_profile: machine_callable_profile_from_human_binders(&metadata.binders),
            },
        );

        Ok(core_decl)
    }

    fn insert_axiom_decl(
        &mut self,
        mut decl: HumanLoweredAxiomDecl,
        metadata: &HumanSourceDeclarationMetadata,
    ) -> HumanResult<HumanLoweredAxiomDecl> {
        let delta: Vec<_> = decl
            .universe_params
            .iter()
            .map(|param| param.name.clone())
            .collect();
        let mut locals = HumanLocalContext::default();
        let mut elaborated_binders = Vec::with_capacity(decl.binders.len());
        let mut transformed_binders = Vec::with_capacity(decl.binders.len());

        for binder in decl.binders {
            let ty = self.insert_term(binder.ty, &mut locals, &delta)?;
            let ty_expr = self.elaborate_machine_term(&ty, &locals, &delta)?;
            locals.push_assumption(binder.name.clone(), ty_expr.clone());
            elaborated_binders.push(HumanElaboratedBinder {
                name: binder.name.clone(),
                ty: ty_expr,
            });
            transformed_binders.push(MachineBinder {
                name: binder.name,
                ty,
                span: binder.span,
            });
        }

        decl.binders = transformed_binders;
        decl.ty = self.insert_term(decl.ty, &mut locals, &delta)?;
        let ty_expr = self.elaborate_machine_term(&decl.ty, &locals, &delta)?;
        let name = decl.name.as_dotted();
        let core_decl = Decl::Axiom {
            name: name.clone(),
            universe_params: delta.clone(),
            ty: human_close_pi(&elaborated_binders, ty_expr),
        };
        self.add_kernel_decl(core_decl, decl.span)?;
        self.signatures.insert(
            name,
            HumanCallableSignature {
                universe_params: delta,
                implicit_profile: machine_callable_profile_from_human_binders(&metadata.binders),
            },
        );

        Ok(decl)
    }

    fn insert_inductive_decl(
        &mut self,
        mut decl: HumanLoweredInductiveDecl,
        metadata: &HumanSourceDeclarationMetadata,
    ) -> HumanResult<HumanLoweredInductiveDecl> {
        let delta: Vec<_> = decl
            .universe_params
            .iter()
            .map(|param| param.name.clone())
            .collect();
        let mut locals = HumanLocalContext::default();
        let mut params = Vec::with_capacity(decl.binders.len());
        let mut transformed_binders = Vec::with_capacity(decl.binders.len());

        for binder in decl.binders {
            let ty = self.insert_term(binder.ty, &mut locals, &delta)?;
            let ty_expr = self.elaborate_machine_term(&ty, &locals, &delta)?;
            locals.push_assumption(binder.name.clone(), ty_expr.clone());
            params.push(HumanElaboratedBinder {
                name: binder.name.clone(),
                ty: ty_expr,
            });
            transformed_binders.push(MachineBinder {
                name: binder.name,
                ty,
                span: binder.span,
            });
        }
        decl.binders = transformed_binders;

        decl.ty = self.insert_term(decl.ty, &mut locals, &delta)?;
        let result_ty = self.elaborate_machine_term(&decl.ty, &locals, &delta)?;
        let (indices, sort) =
            split_inductive_result_type(&self.env, result_ty, &locals, &delta, decl.ty.span())?;
        let head_ty = human_inductive_head_type(&params, &indices, sort.clone());
        let name = decl.name.as_dotted();
        let head_signature = HumanCallableSignature {
            universe_params: delta.clone(),
            implicit_profile: inductive_head_profile(metadata, indices.len()),
        };

        let mut constructor_inserter = self.clone();
        constructor_inserter.add_kernel_decl(
            Decl::Axiom {
                name: name.clone(),
                universe_params: delta.clone(),
                ty: head_ty.clone(),
            },
            decl.span,
        )?;
        constructor_inserter
            .signatures
            .insert(name.clone(), head_signature.clone());

        let mut constructors = Vec::with_capacity(decl.constructors.len());
        let mut transformed_constructors = Vec::with_capacity(decl.constructors.len());
        for constructor in decl.constructors {
            let mut constructor_locals = locals.clone();
            let ty = constructor_inserter.insert_term(
                constructor.ty,
                &mut constructor_locals,
                &delta,
            )?;
            let ty_expr = constructor_inserter.elaborate_machine_term(&ty, &locals, &delta)?;
            constructors.push(ConstructorDecl::new(
                constructor.name.as_dotted(),
                human_close_pi(&params, ty_expr),
            ));
            transformed_constructors.push(HumanLoweredConstructorDecl {
                name: constructor.name,
                ty,
                span: constructor.span,
            });
        }
        self.insertion_steps = constructor_inserter.insertion_steps;
        decl.constructors = transformed_constructors;

        let data = finalize_human_inductive_data(
            name.clone(),
            delta.clone(),
            params.iter().map(kernel_binder_from_human).collect(),
            indices.iter().map(kernel_binder_from_human).collect(),
            sort,
            constructors,
        );
        self.add_kernel_decl(
            Decl::Inductive {
                name: name.clone(),
                universe_params: delta.clone(),
                ty: head_ty,
                data: Box::new(data.clone()),
            },
            decl.span,
        )?;
        self.signatures.insert(name, head_signature);
        self.add_inductive_generated_signatures(&data, metadata);

        Ok(decl)
    }

    fn add_inductive_generated_signatures(
        &mut self,
        data: &InductiveDecl,
        metadata: &HumanSourceDeclarationMetadata,
    ) {
        let param_profile = machine_callable_profile_from_human_binders(&metadata.binders);
        for constructor in &data.constructors {
            self.signatures.insert(
                constructor.name.clone(),
                HumanCallableSignature {
                    universe_params: data.universe_params.clone(),
                    implicit_profile: generated_constructor_profile(
                        &constructor.ty,
                        param_profile.as_slice(),
                    ),
                },
            );
        }
        if let Some(recursor) = &data.recursor {
            self.signatures.insert(
                recursor.name.clone(),
                HumanCallableSignature {
                    universe_params: recursor.universe_params.clone(),
                    implicit_profile: all_explicit_profile(pi_domain_count(&recursor.ty)),
                },
            );
        }
    }

    fn insert_term(
        &mut self,
        term: MachineTerm,
        locals: &mut HumanLocalContext,
        delta: &[String],
    ) -> HumanResult<MachineTerm> {
        match term {
            MachineTerm::App { .. } => {
                let (head, args, span) = collect_machine_app_spine(term);
                let head = self.insert_app_head(head, locals, delta)?;
                let args = args
                    .into_iter()
                    .map(|arg| self.insert_term(arg, locals, delta))
                    .collect::<HumanResult<Vec<_>>>()?;
                self.insert_implicit_spine(head, args, span, locals, delta)
            }
            MachineTerm::Lam {
                binders,
                body,
                span,
            } => {
                let mut nested_locals = locals.clone();
                let binders = self.insert_binders(binders, &mut nested_locals, delta)?;
                Ok(MachineTerm::Lam {
                    binders,
                    body: Box::new(self.insert_term(*body, &mut nested_locals, delta)?),
                    span,
                })
            }
            MachineTerm::Pi {
                binders,
                body,
                span,
            } => {
                let mut nested_locals = locals.clone();
                let binders = self.insert_binders(binders, &mut nested_locals, delta)?;
                Ok(MachineTerm::Pi {
                    binders,
                    body: Box::new(self.insert_term(*body, &mut nested_locals, delta)?),
                    span,
                })
            }
            MachineTerm::Let {
                name,
                ty,
                value,
                body,
                span,
            } => {
                let ty = self.insert_term(*ty, locals, delta)?;
                let value = self.insert_term(*value, locals, delta)?;
                let ty_expr = self.elaborate_machine_term(&ty, locals, delta)?;
                let value_expr = self.elaborate_machine_term(&value, locals, delta)?;
                let mut nested_locals = locals.clone();
                nested_locals.push_definition(name.clone(), ty_expr, value_expr);
                Ok(MachineTerm::Let {
                    name,
                    ty: Box::new(ty),
                    value: Box::new(value),
                    body: Box::new(self.insert_term(*body, &mut nested_locals, delta)?),
                    span,
                })
            }
            MachineTerm::Annot { expr, ty, span } => Ok(MachineTerm::Annot {
                expr: Box::new(self.insert_term(*expr, locals, delta)?),
                ty: Box::new(self.insert_term(*ty, locals, delta)?),
                span,
            }),
            MachineTerm::Ident {
                name,
                universe_args,
                explicit_mode: false,
                span,
            } if self
                .signatures
                .get(&name.as_dotted())
                .is_some_and(|signature| {
                    signature
                        .implicit_profile
                        .contains(&MachineCallableBinderVisibility::Implicit)
                }) =>
            {
                let _ = universe_args;
                Err(self.unsolved_implicit(
                    span,
                    format!(
                        "global name {} still has unresolved implicit arguments",
                        name.as_dotted()
                    ),
                ))
            }
            term => Ok(term),
        }
    }

    fn insert_app_head(
        &mut self,
        term: MachineTerm,
        locals: &mut HumanLocalContext,
        delta: &[String],
    ) -> HumanResult<MachineTerm> {
        match term {
            MachineTerm::Ident { .. } => Ok(term),
            term => self.insert_term(term, locals, delta),
        }
    }

    fn insert_binders(
        &mut self,
        binders: Vec<MachineBinder>,
        locals: &mut HumanLocalContext,
        delta: &[String],
    ) -> HumanResult<Vec<MachineBinder>> {
        binders
            .into_iter()
            .map(|binder| {
                let ty = self.insert_term(binder.ty, locals, delta)?;
                let ty_expr = self.elaborate_machine_term(&ty, locals, delta)?;
                locals.push_assumption(binder.name.clone(), ty_expr);
                Ok(MachineBinder {
                    name: binder.name,
                    ty,
                    span: binder.span,
                })
            })
            .collect()
    }

    fn insert_implicit_spine(
        &mut self,
        head: MachineTerm,
        args: Vec<MachineTerm>,
        span: Span,
        locals: &HumanLocalContext,
        delta: &[String],
    ) -> HumanResult<MachineTerm> {
        let MachineTerm::Ident {
            name,
            universe_args,
            explicit_mode,
            span: head_span,
        } = head
        else {
            return Ok(rebuild_machine_apps(head, args, span));
        };
        if explicit_mode {
            return Ok(rebuild_machine_apps(
                MachineTerm::Ident {
                    name,
                    universe_args,
                    explicit_mode,
                    span: head_span,
                },
                args,
                span,
            ));
        }

        let Some(signature) = self.signatures.get(&name.as_dotted()).cloned() else {
            return Ok(rebuild_machine_apps(
                MachineTerm::Ident {
                    name,
                    universe_args,
                    explicit_mode,
                    span: head_span,
                },
                args,
                span,
            ));
        };
        if signature.universe_params.is_empty()
            && !signature
                .implicit_profile
                .contains(&MachineCallableBinderVisibility::Implicit)
        {
            return Ok(rebuild_machine_apps(
                MachineTerm::Ident {
                    name,
                    universe_args,
                    explicit_mode,
                    span: head_span,
                },
                args,
                span,
            ));
        }

        let solved =
            self.solve_human_spine(&name, &signature, universe_args, args, locals, delta)?;
        let head = MachineTerm::Ident {
            name,
            universe_args: solved.universe_args,
            explicit_mode: true,
            span: head_span,
        };
        Ok(rebuild_machine_apps(head, solved.args, span))
    }

    fn solve_human_spine(
        &mut self,
        name: &MachineName,
        signature: &HumanCallableSignature,
        universe_args: Option<Vec<MachineLevel>>,
        args: Vec<MachineTerm>,
        locals: &HumanLocalContext,
        delta: &[String],
    ) -> HumanResult<HumanSolvedSpine> {
        let span = name.span;
        let dotted = name.as_dotted();
        let Some(decl) = self.env.decl(&dotted) else {
            return Ok(HumanSolvedSpine {
                universe_args,
                args,
            });
        };
        let decl_ty = decl.ty().clone();

        let fixed_universe_args = match universe_args.as_ref() {
            Some(args) if args.len() == signature.universe_params.len() => Some(
                args.iter()
                    .cloned()
                    .map(elaborate_machine_level)
                    .collect::<HumanResult<Vec<_>>>()?,
            ),
            Some(args) => {
                return Err(human_universe_solver_error(
                    span,
                    format!(
                        "global name {} expects {} universe arguments, got {}",
                        dotted,
                        signature.universe_params.len(),
                        args.len()
                    ),
                ));
            }
            None => None,
        };

        for _ in signature
            .implicit_profile
            .iter()
            .filter(|visibility| **visibility == MachineCallableBinderVisibility::Implicit)
        {
            self.bump_insertion_step(span)?;
        }

        let mut solver = HumanUniverseSpineSolver::new(
            &self.env,
            locals,
            delta,
            &signature.universe_params,
            fixed_universe_args,
        );
        let mut current_ty = solver.decl_type_with_universe_metas(&decl_ty, span)?;
        let mut source_args = args.into_iter();
        let mut expanded_args = Vec::new();

        for visibility in &signature.implicit_profile {
            let (ty, body) = match current_ty.clone() {
                Expr::Pi { ty, body, .. } => (ty, body),
                _ => {
                    return Err(HumanDiagnostic::error(
                        HumanDiagnosticKind::ExpectedFunctionType,
                        span,
                        format!("global name {dotted} has more callable binders than its type"),
                    ));
                }
            };
            match visibility {
                MachineCallableBinderVisibility::Implicit => {
                    let id = solver.fresh_implicit((*ty).clone(), span);
                    let placeholder = HumanUniverseSpineSolver::implicit_placeholder(id);
                    current_ty = subst::instantiate(&body, &placeholder).map_err(|err| {
                        human_kernel_expr_diagnostic(
                            span,
                            err,
                            "Human implicit binder instantiation",
                        )
                    })?;
                    expanded_args.push(HumanSpineArg::Implicit(id));
                }
                MachineCallableBinderVisibility::Explicit => {
                    let Some(arg) = source_args.next() else {
                        break;
                    };
                    let arg_expr = self.elaborate_machine_term(&arg, locals, delta)?;
                    let arg_ty = self.infer_core_expr_type(&arg_expr, locals, delta, arg.span())?;
                    solver.unify_expr((*ty).clone(), arg_ty, arg.span())?;
                    current_ty = subst::instantiate(&body, &arg_expr).map_err(|err| {
                        human_kernel_expr_diagnostic(
                            arg.span(),
                            err,
                            "Human explicit binder instantiation",
                        )
                    })?;
                    expanded_args.push(HumanSpineArg::Explicit(arg));
                }
            }
        }

        for arg in source_args {
            let (ty, body) = match current_ty.clone() {
                Expr::Pi { ty, body, .. } => (ty, body),
                _ => {
                    expanded_args.push(HumanSpineArg::Explicit(arg));
                    continue;
                }
            };
            let arg_expr = self.elaborate_machine_term(&arg, locals, delta)?;
            let arg_ty = self.infer_core_expr_type(&arg_expr, locals, delta, arg.span())?;
            solver.unify_expr((*ty).clone(), arg_ty, arg.span())?;
            current_ty = subst::instantiate(&body, &arg_expr).map_err(|err| {
                human_kernel_expr_diagnostic(arg.span(), err, "Human spine instantiation")
            })?;
            expanded_args.push(HumanSpineArg::Explicit(arg));
        }

        let args = expanded_args
            .into_iter()
            .map(|arg| match arg {
                HumanSpineArg::Explicit(arg) => Ok(arg),
                HumanSpineArg::Implicit(id) => solver.materialize_implicit_arg(id, span),
            })
            .collect::<HumanResult<Vec<_>>>()?;
        let universe_args = match universe_args {
            Some(args) => Some(args),
            None if signature.universe_params.is_empty() => None,
            None => Some(solver.solved_universe_args(span)?),
        };

        Ok(HumanSolvedSpine {
            universe_args,
            args,
        })
    }

    fn elaborate_machine_term(
        &self,
        term: &MachineTerm,
        locals: &HumanLocalContext,
        delta: &[String],
    ) -> HumanResult<Expr> {
        let _universe_context_len = delta.len();
        Ok(match term {
            MachineTerm::Ident {
                name,
                universe_args,
                span,
                ..
            } => {
                let name = name.as_dotted();
                let expected = self
                    .env
                    .decl(&name)
                    .map(|decl| decl.universe_params().len())
                    .unwrap_or(0);
                let levels = match universe_args {
                    Some(args) if args.len() == expected => args
                        .iter()
                        .cloned()
                        .map(elaborate_machine_level)
                        .collect::<HumanResult<Vec<_>>>()?,
                    Some(args) => {
                        return Err(human_universe_solver_error(
                            *span,
                            format!(
                                "global name {name} expects {expected} universe arguments, got {}",
                                args.len()
                            ),
                        ));
                    }
                    None if expected == 0 => Vec::new(),
                    None => {
                        return Err(human_universe_solver_error(
                            *span,
                            format!("global name {name} still has unresolved universe arguments"),
                        ));
                    }
                };
                Expr::konst(name, levels)
            }
            MachineTerm::Local { name, span } => {
                locals.lookup_bvar(name).map(Expr::bvar).ok_or_else(|| {
                    HumanDiagnostic::error(
                        HumanDiagnosticKind::UnknownIdentifier,
                        *span,
                        format!("unknown local name {name}"),
                    )
                })?
            }
            MachineTerm::Prop { .. } => Expr::sort(Level::zero()),
            MachineTerm::Type { level, .. } => {
                Expr::sort(Level::succ(elaborate_machine_level(level.clone())?))
            }
            MachineTerm::Sort { level, .. } => Expr::sort(elaborate_machine_level(level.clone())?),
            MachineTerm::App { func, arg, .. } => Expr::app(
                self.elaborate_machine_term(func, locals, delta)?,
                self.elaborate_machine_term(arg, locals, delta)?,
            ),
            MachineTerm::Lam { binders, body, .. } => {
                let mut nested = locals.clone();
                let mut elaborated_binders = Vec::with_capacity(binders.len());
                for binder in binders {
                    let ty = self.elaborate_machine_term(&binder.ty, &nested, delta)?;
                    nested.push_assumption(binder.name.clone(), ty.clone());
                    elaborated_binders.push(HumanElaboratedBinder {
                        name: binder.name.clone(),
                        ty,
                    });
                }
                let body = self.elaborate_machine_term(body, &nested, delta)?;
                human_close_lam(&elaborated_binders, body)
            }
            MachineTerm::Pi { binders, body, .. } => {
                let mut nested = locals.clone();
                let mut elaborated_binders = Vec::with_capacity(binders.len());
                for binder in binders {
                    let ty = self.elaborate_machine_term(&binder.ty, &nested, delta)?;
                    nested.push_assumption(binder.name.clone(), ty.clone());
                    elaborated_binders.push(HumanElaboratedBinder {
                        name: binder.name.clone(),
                        ty,
                    });
                }
                let body = self.elaborate_machine_term(body, &nested, delta)?;
                human_close_pi(&elaborated_binders, body)
            }
            MachineTerm::Let {
                name,
                ty,
                value,
                body,
                ..
            } => {
                let ty = self.elaborate_machine_term(ty, locals, delta)?;
                let value = self.elaborate_machine_term(value, locals, delta)?;
                let mut nested = locals.clone();
                nested.push_definition(name.clone(), ty, value);
                self.elaborate_machine_term(body, &nested, delta)?
            }
            MachineTerm::Annot { expr, .. } => self.elaborate_machine_term(expr, locals, delta)?,
        })
    }

    fn infer_core_expr_type(
        &self,
        expr: &Expr,
        locals: &HumanLocalContext,
        delta: &[String],
        span: Span,
    ) -> HumanResult<Expr> {
        self.env
            .infer(&locals.to_kernel_ctx(), delta, expr)
            .map_err(|err| human_kernel_expr_diagnostic(span, err, "Human implicit inference"))
    }

    fn expect_core_sort(
        &self,
        inferred_type: &Expr,
        locals: &HumanLocalContext,
        delta: &[String],
        span: Span,
    ) -> HumanResult<()> {
        let whnf = self
            .env
            .whnf(&locals.to_kernel_ctx(), delta, inferred_type)
            .map_err(|err| human_kernel_expr_diagnostic(span, err, "Human implicit type"))?;
        if matches!(whnf, Expr::Sort(_)) {
            Ok(())
        } else {
            Err(HumanDiagnostic::error(
                HumanDiagnosticKind::ExpectedSort,
                span,
                format!("expected a type, got {whnf:?}"),
            ))
        }
    }

    fn add_kernel_decl(&mut self, decl: Decl, span: Span) -> HumanResult<()> {
        if let Some(existing) = self.env.decl(decl.name()) {
            if existing == &decl {
                return Ok(());
            }
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::KernelRejected,
                span,
                format!(
                    "kernel declaration {} conflicts with an existing declaration",
                    decl.name()
                ),
            )
            .with_phase(HumanDiagnosticPhase::KernelHandoff));
        }

        add_referenced_builtin_decls_to_human_env(
            &mut self.env,
            &decl,
            span,
            "Human implicit environment",
        )?;

        match decl {
            Decl::Axiom {
                name,
                universe_params,
                ty,
            } => self.env.add_axiom(name, universe_params, ty),
            Decl::AxiomConstrained {
                name,
                universe_params,
                universe_constraints,
                ty,
            } => self.env.add_axiom_with_universe_constraints(
                name,
                universe_params,
                universe_constraints,
                ty,
            ),
            Decl::Def {
                name,
                universe_params,
                ty,
                value,
                reducibility,
            } => self
                .env
                .add_def(name, universe_params, ty, value, reducibility),
            Decl::DefConstrained {
                name,
                universe_params,
                universe_constraints,
                ty,
                value,
                reducibility,
            } => self.env.add_def_with_universe_constraints(
                name,
                universe_params,
                universe_constraints,
                ty,
                value,
                reducibility,
            ),
            Decl::Theorem {
                name,
                universe_params,
                ty,
                proof,
            } => self.env.add_theorem(name, universe_params, ty, proof),
            Decl::TheoremConstrained {
                name,
                universe_params,
                universe_constraints,
                ty,
                proof,
            } => self.env.add_theorem_with_universe_constraints(
                name,
                universe_params,
                universe_constraints,
                ty,
                proof,
            ),
            Decl::Inductive { data, .. } => self.env.add_inductive(*data),
            Decl::Constructor { .. } | Decl::Recursor { .. } => Ok(()),
        }
        .map_err(|err| {
            human_kernel_decl_diagnostic(span, err, "Human implicit environment")
                .with_phase(HumanDiagnosticPhase::KernelHandoff)
        })
    }

    fn bump_insertion_step(&mut self, span: Span) -> HumanResult<()> {
        self.insertion_steps += 1;
        if self.insertion_steps > MAX_HUMAN_IMPLICIT_INSERTION_STEPS {
            return Err(self.unsolved_implicit(
                span,
                "Human implicit insertion exceeded the bounded search limit".to_owned(),
            ));
        }
        Ok(())
    }

    fn unsolved_implicit(&self, span: Span, message: String) -> HumanDiagnostic {
        HumanDiagnostic::error(HumanDiagnosticKind::UnsolvedImplicit, span, message).with_payload(
            HumanDiagnosticPayload {
                unsolved_meta: Some(HumanUnsolvedMeta {
                    kind: HumanUnsolvedMetaKind::SyntheticImplicit,
                    name: None,
                }),
                ..HumanDiagnosticPayload::default()
            },
        )
    }
}

fn add_referenced_builtin_decls_to_human_env(
    env: &mut Env,
    decl: &Decl,
    span: Span,
    context: &str,
) -> HumanResult<()> {
    let mut names = BTreeSet::new();
    collect_const_names_from_human_decl(&mut names, decl);
    remove_human_decl_owned_const_names(&mut names, decl);
    add_human_builtin_decls_for_names(env, &names, span, context)
}

fn human_referenced_builtin_names(module: &ResolvedHumanModule) -> BTreeSet<npa_cert::Name> {
    let mut names = BTreeSet::new();
    for resolved in &module.resolved_names {
        if let HumanResolvedName::Global(HumanGlobalRef::Builtin { name, .. }) = &resolved.resolved
        {
            names.insert(name.clone());
        }
    }
    for notation in &module.resolved_notations {
        for candidate in &notation.candidates {
            if let HumanGlobalRef::Builtin { name, .. } = candidate {
                names.insert(name.clone());
            }
        }
    }
    names
}

fn collect_const_names_from_human_decl(names: &mut BTreeSet<npa_cert::Name>, decl: &Decl) {
    match decl {
        Decl::Axiom { ty, .. } | Decl::AxiomConstrained { ty, .. } => {
            collect_const_names_from_human_expr(names, ty)
        }
        Decl::Def { ty, value, .. } | Decl::DefConstrained { ty, value, .. } => {
            collect_const_names_from_human_expr(names, ty);
            collect_const_names_from_human_expr(names, value);
        }
        Decl::Theorem { ty, proof, .. } | Decl::TheoremConstrained { ty, proof, .. } => {
            collect_const_names_from_human_expr(names, ty);
            collect_const_names_from_human_expr(names, proof);
        }
        Decl::Inductive { data, .. } => {
            for param in &data.params {
                collect_const_names_from_human_expr(names, &param.ty);
            }
            for index in &data.indices {
                collect_const_names_from_human_expr(names, &index.ty);
            }
            for constructor in &data.constructors {
                collect_const_names_from_human_expr(names, &constructor.ty);
            }
            if let Some(recursor) = &data.recursor {
                collect_const_names_from_human_expr(names, &recursor.ty);
            }
        }
        Decl::Constructor { ty, .. } | Decl::Recursor { ty, .. } => {
            collect_const_names_from_human_expr(names, ty);
        }
    }
}

fn remove_human_decl_owned_const_names(names: &mut BTreeSet<npa_cert::Name>, decl: &Decl) {
    names.remove(&npa_cert::Name::from_dotted(decl.name()));
    if let Decl::Inductive { data, .. } = decl {
        for constructor in &data.constructors {
            names.remove(&npa_cert::Name::from_dotted(&constructor.name));
        }
        if let Some(recursor) = &data.recursor {
            names.remove(&npa_cert::Name::from_dotted(&recursor.name));
        }
    }
}

fn collect_const_names_from_human_expr(names: &mut BTreeSet<npa_cert::Name>, expr: &Expr) {
    match expr {
        Expr::Sort(_) | Expr::BVar(_) => {}
        Expr::Const { name, .. } => {
            names.insert(npa_cert::Name::from_dotted(name));
        }
        Expr::App(func, arg) => {
            collect_const_names_from_human_expr(names, func);
            collect_const_names_from_human_expr(names, arg);
        }
        Expr::Lam { ty, body, .. } | Expr::Pi { ty, body, .. } => {
            collect_const_names_from_human_expr(names, ty);
            collect_const_names_from_human_expr(names, body);
        }
        Expr::Let {
            ty, value, body, ..
        } => {
            collect_const_names_from_human_expr(names, ty);
            collect_const_names_from_human_expr(names, value);
            collect_const_names_from_human_expr(names, body);
        }
    }
}

fn add_human_builtin_decls_for_names(
    env: &mut Env,
    names: &BTreeSet<npa_cert::Name>,
    span: Span,
    context: &str,
) -> HumanResult<()> {
    let needs_nat = names.iter().any(|name| {
        let name = name.as_dotted();
        matches!(name.as_str(), "Nat" | "Nat.zero" | "Nat.succ" | "Nat.rec")
    });
    let needs_eq = names.iter().any(|name| {
        let name = name.as_dotted();
        matches!(name.as_str(), "Eq" | "Eq.refl" | "Eq.rec")
    });
    let needs_eq_rec = names.iter().any(|name| name.as_dotted() == "Eq.rec");

    if needs_nat && env.decl("Nat").is_none() {
        env.add_inductive(nat_inductive()).map_err(|err| {
            human_kernel_decl_diagnostic(span, err, context)
                .with_phase(HumanDiagnosticPhase::KernelHandoff)
        })?;
    }
    if needs_eq && env.decl("Eq").is_none() {
        env.add_inductive(eq_inductive()).map_err(|err| {
            human_kernel_decl_diagnostic(span, err, context)
                .with_phase(HumanDiagnosticPhase::KernelHandoff)
        })?;
    }
    if needs_eq_rec && env.decl("Eq.rec").is_none() {
        env.add_axiom(
            "Eq.rec",
            vec!["u".to_owned(), "v".to_owned()],
            eq_rec_type(Level::param("u"), Level::param("v")),
        )
        .map_err(|err| {
            human_kernel_decl_diagnostic(span, err, context)
                .with_phase(HumanDiagnosticPhase::KernelHandoff)
        })?;
    }
    Ok(())
}

fn human_kernel_expr_diagnostic(span: Span, err: Error, context: &str) -> HumanDiagnostic {
    match err {
        Error::ExpectedPi { actual } => HumanDiagnostic::error(
            HumanDiagnosticKind::ExpectedFunctionType,
            span,
            format!("{context}: expected a function type, got {actual:?}"),
        ),
        Error::ExpectedSort { actual } => HumanDiagnostic::error(
            HumanDiagnosticKind::ExpectedSort,
            span,
            format!("{context}: expected a type, got {actual:?}"),
        ),
        Error::TypeMismatch { expected, actual } => HumanDiagnostic::error(
            HumanDiagnosticKind::TypeMismatch,
            span,
            format!("{context}: expected {expected:?}, got {actual:?}"),
        ),
        Error::UnknownConstant(name) => HumanDiagnostic::error(
            HumanDiagnosticKind::UnknownIdentifier,
            span,
            format!("{context}: unknown global name {name}"),
        ),
        err => HumanDiagnostic::error(
            HumanDiagnosticKind::KernelRejected,
            span,
            format!("{context}: kernel rejected elaborated Human expression: {err:?}"),
        ),
    }
}

fn human_kernel_decl_diagnostic(span: Span, err: Error, context: &str) -> HumanDiagnostic {
    match err {
        Error::ExpectedPi { actual } => HumanDiagnostic::error(
            HumanDiagnosticKind::ExpectedFunctionType,
            span,
            format!("{context}: expected a function type, got {actual:?}"),
        ),
        Error::ExpectedSort { actual } => HumanDiagnostic::error(
            HumanDiagnosticKind::ExpectedSort,
            span,
            format!("{context}: expected a declaration type, got {actual:?}"),
        ),
        Error::TypeMismatch { expected, actual } => HumanDiagnostic::error(
            HumanDiagnosticKind::TypeMismatch,
            span,
            format!("{context}: declaration value has type {actual:?}, expected {expected:?}"),
        ),
        err => HumanDiagnostic::error(
            HumanDiagnosticKind::KernelRejected,
            span,
            format!("{context}: kernel rejected elaborated Human declaration: {err:?}"),
        ),
    }
}

fn active_human_imports<'a>(
    module: &ResolvedHumanModule,
    verified_imports: &'a [VerifiedImport],
) -> HumanResult<Vec<&'a VerifiedImport>> {
    active_human_import_indices(module, verified_imports).map(|indices| {
        indices
            .into_iter()
            .map(|index| &verified_imports[index])
            .collect()
    })
}

fn active_human_import_indices(
    module: &ResolvedHumanModule,
    verified_imports: &[VerifiedImport],
) -> HumanResult<Vec<usize>> {
    let mut seen = BTreeSet::new();
    let mut imports = Vec::new();
    for item in &module.module.items {
        let HumanItem::Import {
            module: import_name,
            span,
        } = item
        else {
            continue;
        };
        let import_module = npa_cert::Name(import_name.parts.clone());
        if !seen.insert(import_module.clone()) {
            continue;
        }
        imports.push(find_human_verified_import_index(
            verified_imports,
            &import_module,
            import_name,
            *span,
        )?);
    }
    Ok(imports)
}

fn active_human_import_indices_from_source_interfaces(
    active_imports: &[HumanImportedSourceInterface],
    verified_modules: &[npa_cert::VerifiedModule],
    file_id: crate::FileId,
) -> HumanResult<Vec<usize>> {
    active_imports
        .iter()
        .map(|active| {
            verified_modules
                .iter()
                .position(|module| {
                    let import = VerifiedImport::from(module);
                    import.module == active.module
                        && import.export_hash == active.export_hash
                        && import.certificate_hash == active.certificate_hash
                })
                .ok_or_else(|| {
                    HumanDiagnostic::error(
                        HumanDiagnosticKind::MissingVerifiedImport,
                        Span::empty(file_id),
                        format!(
                            "missing verified import for active Human import {}",
                            active.module.as_dotted()
                        ),
                    )
                })
        })
        .collect()
}

fn human_by_proof_targets(
    module_name: &npa_cert::ModuleName,
    module: &ResolvedHumanModule,
) -> HumanResult<Vec<HumanByProofTarget>> {
    let mut declarations = module.state.source_interfaces.current.declarations.iter();
    let mut source_index = 0_u64;
    let mut targets = Vec::new();

    for item in &module.module.items {
        match item {
            HumanItem::Def(_) | HumanItem::Axiom(_) | HumanItem::Inductive(_) => {
                declarations.next().ok_or_else(|| {
                    HumanDiagnostic::not_implemented(item.span(), "Human declaration metadata")
                })?;
                source_index += 1;
            }
            HumanItem::Theorem(decl) => {
                let metadata = declarations.next().ok_or_else(|| {
                    HumanDiagnostic::not_implemented(decl.span, "Human declaration metadata")
                })?;
                if let HumanDeclValue::ProofBlock(block) = &decl.value {
                    let current_name = npa_cert::Name(metadata.name.parts.clone());
                    targets.push(HumanByProofTarget {
                        source_index,
                        theorem_name: prefixed_human_current_name(module_name, &current_name),
                        script: block.script.clone(),
                    });
                }
                source_index += 1;
            }
            HumanItem::Import { .. }
            | HumanItem::Open { .. }
            | HumanItem::NamespaceStart { .. }
            | HumanItem::NamespaceEnd { .. }
            | HumanItem::Notation(_) => {}
        }
    }

    Ok(targets)
}

fn by_proof_map(by_proofs: &[HumanByProofCore], span: Span) -> HumanResult<BTreeMap<u64, Expr>> {
    let mut map = BTreeMap::new();
    for by_proof in by_proofs {
        if map
            .insert(by_proof.source_index, by_proof.proof.clone())
            .is_some()
        {
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::UnsupportedTactic,
                span,
                format!(
                    "duplicate Human by proof core for source index {}",
                    by_proof.source_index
                ),
            )
            .with_phase(HumanDiagnosticPhase::Elaborator));
        }
    }
    Ok(map)
}

fn validate_by_proof_map_indices(
    by_proofs: &BTreeMap<u64, Expr>,
    expected: &BTreeSet<u64>,
    span: Span,
) -> HumanResult<()> {
    let actual = by_proofs.keys().copied().collect::<BTreeSet<_>>();
    if &actual == expected {
        return Ok(());
    }

    Err(HumanDiagnostic::error(
        HumanDiagnosticKind::UnsupportedTactic,
        span,
        format!(
            "Human by proof core source indices must match by theorem indices exactly: expected {:?}, got {:?}",
            expected, actual
        ),
    )
    .with_phase(HumanDiagnosticPhase::Elaborator))
}

fn find_human_verified_import_index(
    verified_imports: &[VerifiedImport],
    import_module: &npa_cert::ModuleName,
    import_name: &HumanName,
    span: Span,
) -> HumanResult<usize> {
    let mut matches = verified_imports
        .iter()
        .enumerate()
        .filter(|(_, import)| &import.module == import_module);
    let Some((first_index, first)) = matches.next() else {
        return Err(HumanDiagnostic::error(
            HumanDiagnosticKind::MissingVerifiedImport,
            span,
            format!(
                "missing verified import for module {}",
                import_name.as_dotted()
            ),
        ));
    };

    if matches.any(|(_, import)| import != first) {
        return Err(HumanDiagnostic::error(
            HumanDiagnosticKind::AmbiguousName,
            span,
            format!(
                "ambiguous verified import for module {}",
                import_name.as_dotted()
            ),
        ));
    }

    Ok(first_index)
}

fn human_certificate_import_diagnostic(
    fallback_span: Span,
    err: crate::MachineDiagnostic,
) -> HumanDiagnostic {
    let kind = match err.kind {
        crate::MachineDiagnosticKind::MissingVerifiedImport => {
            HumanDiagnosticKind::MissingVerifiedImport
        }
        crate::MachineDiagnosticKind::ImportResolutionError => {
            HumanDiagnosticKind::ImportResolutionError
        }
        _ => HumanDiagnosticKind::KernelRejected,
    };
    let primary_span = if err.primary_span == Span::empty(fallback_span.file_id) {
        fallback_span
    } else {
        err.primary_span
    };
    HumanDiagnostic::error(
        kind,
        primary_span,
        format!(
            "certificate certificate import closure rejected Human source: {}",
            err.message
        ),
    )
}

fn kernel_decls_for_human_import(import: &VerifiedImport) -> Vec<Decl> {
    if !import.kernel_decls.is_empty() {
        return import.kernel_decls.clone();
    }

    import
        .exports
        .iter()
        .map(|export| Decl::Axiom {
            name: export.name.as_dotted(),
            universe_params: export.universe_params.clone(),
            ty: export.ty.clone(),
        })
        .collect()
}

fn collect_machine_app_spine(term: MachineTerm) -> (MachineTerm, Vec<MachineTerm>, Span) {
    let span = term.span();
    let mut args = Vec::new();
    let mut head = term;
    while let MachineTerm::App { func, arg, .. } = head {
        args.push(*arg);
        head = *func;
    }
    args.reverse();
    (head, args, span)
}

fn rebuild_machine_apps(head: MachineTerm, args: Vec<MachineTerm>, span: Span) -> MachineTerm {
    let mut term = head;
    for arg in args {
        let app_span = term.span().join(arg.span());
        term = MachineTerm::App {
            func: Box::new(term),
            arg: Box::new(arg),
            span: app_span,
        };
    }
    if matches!(term, MachineTerm::App { .. }) {
        term
    } else {
        let _ = span;
        term
    }
}

fn take_expected_pi_binder(expected: MachineTerm) -> Option<(MachineBinder, MachineTerm)> {
    let MachineTerm::Pi {
        mut binders,
        body,
        span,
    } = expected
    else {
        return None;
    };
    if binders.is_empty() {
        return None;
    }
    let binder = binders.remove(0);
    let rest = if binders.is_empty() {
        *body
    } else {
        MachineTerm::Pi {
            binders,
            body,
            span,
        }
    };
    Some((binder, rest))
}

fn rename_machine_local(term: MachineTerm, from: &str, to: &str) -> MachineTerm {
    rename_machine_local_scoped(term, from, to, false)
}

fn rename_machine_local_scoped(
    term: MachineTerm,
    from: &str,
    to: &str,
    shadowed: bool,
) -> MachineTerm {
    match term {
        MachineTerm::Ident {
            name,
            universe_args,
            explicit_mode,
            span,
        } => MachineTerm::Ident {
            name,
            universe_args,
            explicit_mode,
            span,
        },
        MachineTerm::Local { name, span } if !shadowed && name == from => MachineTerm::Local {
            name: to.to_owned(),
            span,
        },
        MachineTerm::Local { name, span } => MachineTerm::Local { name, span },
        MachineTerm::Prop { span } => MachineTerm::Prop { span },
        MachineTerm::Type { level, span } => MachineTerm::Type { level, span },
        MachineTerm::Sort { level, span } => MachineTerm::Sort { level, span },
        MachineTerm::App { func, arg, span } => MachineTerm::App {
            func: Box::new(rename_machine_local_scoped(*func, from, to, shadowed)),
            arg: Box::new(rename_machine_local_scoped(*arg, from, to, shadowed)),
            span,
        },
        MachineTerm::Lam {
            binders,
            body,
            span,
        } => {
            let (binders, body_shadowed) =
                rename_machine_binders_scoped(binders, from, to, shadowed);
            MachineTerm::Lam {
                binders,
                body: Box::new(rename_machine_local_scoped(*body, from, to, body_shadowed)),
                span,
            }
        }
        MachineTerm::Pi {
            binders,
            body,
            span,
        } => {
            let (binders, body_shadowed) =
                rename_machine_binders_scoped(binders, from, to, shadowed);
            MachineTerm::Pi {
                binders,
                body: Box::new(rename_machine_local_scoped(*body, from, to, body_shadowed)),
                span,
            }
        }
        MachineTerm::Let {
            name,
            ty,
            value,
            body,
            span,
        } => MachineTerm::Let {
            body: Box::new(rename_machine_local_scoped(
                *body,
                from,
                to,
                shadowed || name == from,
            )),
            name,
            ty: Box::new(rename_machine_local_scoped(*ty, from, to, shadowed)),
            value: Box::new(rename_machine_local_scoped(*value, from, to, shadowed)),
            span,
        },
        MachineTerm::Annot { expr, ty, span } => MachineTerm::Annot {
            expr: Box::new(rename_machine_local_scoped(*expr, from, to, shadowed)),
            ty: Box::new(rename_machine_local_scoped(*ty, from, to, shadowed)),
            span,
        },
    }
}

fn rename_machine_binders_scoped(
    binders: Vec<MachineBinder>,
    from: &str,
    to: &str,
    mut shadowed: bool,
) -> (Vec<MachineBinder>, bool) {
    let binders = binders
        .into_iter()
        .map(|binder| {
            let ty = rename_machine_local_scoped(binder.ty, from, to, shadowed);
            if binder.name == from {
                shadowed = true;
            }
            MachineBinder {
                name: binder.name,
                ty,
                span: binder.span,
            }
        })
        .collect();
    (binders, shadowed)
}

fn human_close_lam(binders: &[HumanElaboratedBinder], mut body: Expr) -> Expr {
    for binder in binders.iter().rev() {
        body = Expr::lam(binder.name.clone(), binder.ty.clone(), body);
    }
    body
}

fn human_close_pi(binders: &[HumanElaboratedBinder], mut body: Expr) -> Expr {
    for binder in binders.iter().rev() {
        body = Expr::pi(binder.name.clone(), binder.ty.clone(), body);
    }
    body
}

fn human_inductive_head_type(
    params: &[HumanElaboratedBinder],
    indices: &[HumanElaboratedBinder],
    sort: Level,
) -> Expr {
    let mut binders = Vec::with_capacity(params.len() + indices.len());
    binders.extend_from_slice(params);
    binders.extend_from_slice(indices);
    human_close_pi(&binders, Expr::sort(sort))
}

fn split_inductive_result_type(
    env: &Env,
    result_ty: Expr,
    locals: &HumanLocalContext,
    delta: &[String],
    span: Span,
) -> HumanResult<(Vec<HumanElaboratedBinder>, Level)> {
    let mut nested = locals.clone();
    let mut indices = Vec::new();
    let mut current = result_ty;

    loop {
        let whnf = env
            .whnf(&nested.to_kernel_ctx(), delta, &current)
            .map_err(|err| human_kernel_expr_diagnostic(span, err, "Human inductive type"))?;
        match whnf {
            Expr::Pi { binder, ty, body } => {
                let ty = *ty;
                nested.push_assumption(binder.clone(), ty.clone());
                indices.push(HumanElaboratedBinder { name: binder, ty });
                current = *body;
            }
            Expr::Sort(level) => return Ok((indices, level)),
            actual => {
                return Err(HumanDiagnostic::error(
                    HumanDiagnosticKind::ExpectedSort,
                    span,
                    format!(
                        "Human inductive type: expected telescope ending in Sort, got {actual:?}"
                    ),
                ));
            }
        }
    }
}

fn kernel_binder_from_human(binder: &HumanElaboratedBinder) -> Binder {
    Binder::new(binder.name.clone(), binder.ty.clone())
}

fn finalize_human_inductive_data(
    name: String,
    universe_params: Vec<String>,
    params: Vec<Binder>,
    indices: Vec<Binder>,
    sort: Level,
    constructors: Vec<ConstructorDecl>,
) -> InductiveDecl {
    let base = InductiveDecl::new(
        name,
        universe_params,
        params,
        indices,
        sort,
        constructors,
        None,
    );
    npa_cert::generate_inductive_artifacts_v1(&base).unwrap_or(base)
}

fn inductive_head_profile(
    metadata: &HumanSourceDeclarationMetadata,
    index_count: usize,
) -> Vec<MachineCallableBinderVisibility> {
    let mut profile = machine_callable_profile_from_human_binders(&metadata.binders);
    profile.extend(all_explicit_profile(index_count));
    profile
}

fn generated_constructor_profile(
    ty: &Expr,
    param_profile: &[MachineCallableBinderVisibility],
) -> Vec<MachineCallableBinderVisibility> {
    let domain_count = pi_domain_count(ty);
    let mut profile = Vec::with_capacity(domain_count);
    profile.extend(param_profile.iter().copied().take(domain_count));
    if domain_count > profile.len() {
        profile.extend(all_explicit_profile(domain_count - profile.len()));
    }
    profile
}

fn all_explicit_profile(count: usize) -> Vec<MachineCallableBinderVisibility> {
    vec![MachineCallableBinderVisibility::Explicit; count]
}

fn pi_domain_count(ty: &Expr) -> usize {
    let mut count = 0;
    let mut current = ty;
    while let Expr::Pi { body, .. } = current {
        count += 1;
        current = body;
    }
    count
}

fn elaborate_machine_level(level: MachineLevel) -> HumanResult<Level> {
    Ok(match level {
        MachineLevel::Nat { value, .. } => level_from_nat(value),
        MachineLevel::Param { name, .. } => Level::param(name),
        MachineLevel::Succ { level, .. } => Level::succ(elaborate_machine_level(*level)?),
        MachineLevel::Max { lhs, rhs, .. } => Level::max(
            elaborate_machine_level(*lhs)?,
            elaborate_machine_level(*rhs)?,
        ),
        MachineLevel::IMax { lhs, rhs, .. } => Level::imax(
            elaborate_machine_level(*lhs)?,
            elaborate_machine_level(*rhs)?,
        ),
    })
}

fn level_from_nat(value: u64) -> Level {
    let mut level = Level::zero();
    for _ in 0..value {
        level = Level::succ(level);
    }
    level
}

fn core_expr_to_machine_term(
    expr: &Expr,
    locals: &HumanLocalContext,
    span: Span,
) -> Option<MachineTerm> {
    match expr {
        Expr::Sort(level) => Some(MachineTerm::Sort {
            level: core_level_to_machine_level(level, span),
            span,
        }),
        Expr::BVar(index) => Some(MachineTerm::Local {
            name: locals.name_for_bvar(*index)?.to_owned(),
            span,
        }),
        Expr::Const { name, levels } => Some(MachineTerm::Ident {
            name: MachineName {
                parts: name.split('.').map(str::to_owned).collect(),
                span,
            },
            universe_args: (!levels.is_empty()).then(|| {
                levels
                    .iter()
                    .map(|level| core_level_to_machine_level(level, span))
                    .collect()
            }),
            explicit_mode: !levels.is_empty(),
            span,
        }),
        Expr::App(_, _) => {
            let (head, args) = npa_kernel::expr::collect_apps(expr);
            let head = core_expr_to_machine_term(&head, locals, span)?;
            let args = args
                .iter()
                .map(|arg| core_expr_to_machine_term(arg, locals, span))
                .collect::<Option<Vec<_>>>()?;
            Some(rebuild_machine_apps(head, args, span))
        }
        Expr::Lam { binder, ty, body } => {
            let ty_term = core_expr_to_machine_term(ty, locals, span)?;
            let mut nested = locals.clone();
            nested.push_assumption(binder.clone(), (**ty).clone());
            Some(MachineTerm::Lam {
                binders: vec![MachineBinder {
                    name: binder.clone(),
                    ty: ty_term,
                    span,
                }],
                body: Box::new(core_expr_to_machine_term(body, &nested, span)?),
                span,
            })
        }
        Expr::Pi { binder, ty, body } => {
            let ty_term = core_expr_to_machine_term(ty, locals, span)?;
            let mut nested = locals.clone();
            nested.push_assumption(binder.clone(), (**ty).clone());
            Some(MachineTerm::Pi {
                binders: vec![MachineBinder {
                    name: binder.clone(),
                    ty: ty_term,
                    span,
                }],
                body: Box::new(core_expr_to_machine_term(body, &nested, span)?),
                span,
            })
        }
        Expr::Let {
            binder,
            ty,
            value,
            body,
        } => {
            let ty_term = core_expr_to_machine_term(ty, locals, span)?;
            let value_term = core_expr_to_machine_term(value, locals, span)?;
            let mut nested = locals.clone();
            nested.push_definition(binder.clone(), (**ty).clone(), (**value).clone());
            Some(MachineTerm::Let {
                name: binder.clone(),
                ty: Box::new(ty_term),
                value: Box::new(value_term),
                body: Box::new(core_expr_to_machine_term(body, &nested, span)?),
                span,
            })
        }
    }
}

fn human_tactic_meta_fallback_machine_term(span: Span) -> MachineTerm {
    MachineTerm::Sort {
        level: MachineLevel::Nat { value: 0, span },
        span,
    }
}

fn core_level_to_machine_level(level: &Level, span: Span) -> MachineLevel {
    if let Some(value) = core_level_as_u64(level) {
        return MachineLevel::Nat { value, span };
    }

    match level {
        Level::Zero => MachineLevel::Nat { value: 0, span },
        Level::Param(name) => MachineLevel::Param {
            name: name.clone(),
            span,
        },
        Level::Succ(level) => MachineLevel::Succ {
            level: Box::new(core_level_to_machine_level(level, span)),
            span,
        },
        Level::Max(lhs, rhs) => MachineLevel::Max {
            lhs: Box::new(core_level_to_machine_level(lhs, span)),
            rhs: Box::new(core_level_to_machine_level(rhs, span)),
            span,
        },
        Level::IMax(lhs, rhs) => MachineLevel::IMax {
            lhs: Box::new(core_level_to_machine_level(lhs, span)),
            rhs: Box::new(core_level_to_machine_level(rhs, span)),
            span,
        },
    }
}

fn core_level_as_u64(level: &Level) -> Option<u64> {
    match npa_kernel::level::normalize_level(level.clone()) {
        Level::Zero => Some(0),
        Level::Succ(level) => Some(core_level_as_u64(&level)? + 1),
        _ => None,
    }
}

struct HumanToMachineLowering<'a> {
    name_uses: std::slice::Iter<'a, HumanResolvedNameUse>,
    notation_uses: std::slice::Iter<'a, HumanResolvedNotationUse>,
    notation_choices: std::slice::Iter<'a, usize>,
    implicit_inserter: HumanImplicitInserter,
    meta_store: HumanMetaStore,
    current_module_prefix: Option<npa_cert::ModuleName>,
}

impl<'a> HumanToMachineLowering<'a> {
    fn new(
        module: &'a ResolvedHumanModule,
        verified_imports: &[VerifiedImport],
        notation_plan: &'a [usize],
    ) -> HumanResult<Self> {
        Ok(Self {
            name_uses: module.resolved_names.iter(),
            notation_uses: module.resolved_notations.iter(),
            notation_choices: notation_plan.iter(),
            implicit_inserter: HumanImplicitInserter::new(module, verified_imports)?,
            meta_store: HumanMetaStore::default(),
            current_module_prefix: None,
        })
    }

    fn for_tactic_term(
        resolved_names: &'a [HumanResolvedNameUse],
        resolved_notations: &'a [HumanResolvedNotationUse],
        notation_plan: &'a [usize],
        implicit_inserter: HumanImplicitInserter,
    ) -> Self {
        Self {
            name_uses: resolved_names.iter(),
            notation_uses: resolved_notations.iter(),
            notation_choices: notation_plan.iter(),
            implicit_inserter,
            meta_store: HumanMetaStore::default(),
            current_module_prefix: None,
        }
    }

    fn with_current_module_prefix(mut self, module_name: npa_cert::ModuleName) -> Self {
        self.current_module_prefix = Some(module_name);
        self
    }

    fn machine_name_from_current_metadata(&self, name: crate::HumanName) -> MachineName {
        match &self.current_module_prefix {
            Some(module_name) => {
                let span = name.span;
                let prefixed =
                    prefixed_human_current_name(module_name, &npa_cert::Name(name.parts));
                MachineName {
                    parts: prefixed.0,
                    span,
                }
            }
            None => machine_name(name),
        }
    }

    fn lower_module(&mut self, module: &ResolvedHumanModule) -> HumanResult<HumanLoweredModule> {
        self.lower_module_with_core_proofs(module, &BTreeMap::new())
    }

    fn lower_module_with_core_proofs(
        &mut self,
        module: &ResolvedHumanModule,
        by_proofs: &BTreeMap<u64, Expr>,
    ) -> HumanResult<HumanLoweredModule> {
        let mut lowered_items = Vec::new();
        let mut declarations = module.state.source_interfaces.current.declarations.iter();
        let mut source_index = 0_u64;

        for item in &module.module.items {
            match item {
                HumanItem::Import { module, span } => {
                    let _ = (module, span);
                    lowered_items.push(HumanLoweredItem::Import);
                }
                HumanItem::Def(decl) => {
                    let metadata = declarations.next().ok_or_else(|| {
                        HumanDiagnostic::not_implemented(decl.span, "Human declaration metadata")
                    })?;
                    let lowered = self.lower_decl(decl.clone(), metadata)?;
                    let lowered = self.implicit_inserter.insert_decl(
                        lowered,
                        metadata,
                        HumanLoweredDeclKind::Def,
                    )?;
                    lowered_items.push(HumanLoweredItem::Def(lowered));
                    source_index += 1;
                }
                HumanItem::Theorem(decl) => {
                    let metadata = declarations.next().ok_or_else(|| {
                        HumanDiagnostic::not_implemented(decl.span, "Human declaration metadata")
                    })?;
                    if let HumanDeclValue::ProofBlock(block) = &decl.value {
                        let Some(proof) = by_proofs.get(&source_index) else {
                            return Err(HumanDiagnostic::unsupported_tactic(
                                block.span,
                                "by proof block elaboration is reserved for the Human tactic tactic bridge",
                            )
                            .with_phase(HumanDiagnosticPhase::Elaborator));
                        };
                        let lowered = self.lower_decl_signature(decl.clone(), metadata)?;
                        let span = decl.span;
                        let core_decl = self.implicit_inserter.insert_core_theorem_decl(
                            lowered,
                            metadata,
                            proof.clone(),
                            span,
                        )?;
                        lowered_items.push(HumanLoweredItem::TheoremCoreProof {
                            decl: core_decl,
                            span,
                        });
                    } else {
                        let lowered = self.lower_decl(decl.clone(), metadata)?;
                        let lowered = self.implicit_inserter.insert_decl(
                            lowered,
                            metadata,
                            HumanLoweredDeclKind::Theorem,
                        )?;
                        lowered_items.push(HumanLoweredItem::Theorem(lowered));
                    }
                    source_index += 1;
                }
                HumanItem::Axiom(decl) => {
                    let metadata = declarations.next().ok_or_else(|| {
                        HumanDiagnostic::not_implemented(decl.span, "Human declaration metadata")
                    })?;
                    let lowered = self.lower_axiom_decl(decl.clone(), metadata)?;
                    let lowered = self
                        .implicit_inserter
                        .insert_axiom_decl(lowered, metadata)?;
                    lowered_items.push(HumanLoweredItem::Axiom(lowered));
                    source_index += 1;
                }
                HumanItem::Inductive(decl) => {
                    let metadata = declarations.next().ok_or_else(|| {
                        HumanDiagnostic::not_implemented(decl.span, "Human declaration metadata")
                    })?;
                    let lowered = self.lower_inductive_decl(decl.clone(), metadata)?;
                    let lowered = self
                        .implicit_inserter
                        .insert_inductive_decl(lowered, metadata)?;
                    lowered_items.push(HumanLoweredItem::Inductive(lowered));
                    source_index += 1;
                }
                HumanItem::Open { .. }
                | HumanItem::NamespaceStart { .. }
                | HumanItem::NamespaceEnd { .. }
                | HumanItem::Notation(_) => {}
            }
        }

        Ok(HumanLoweredModule {
            items: lowered_items,
        })
    }

    fn lower_proof_start(
        &mut self,
        module_name: &npa_cert::ModuleName,
        theorem_name: &npa_cert::Name,
        module: &ResolvedHumanModule,
    ) -> HumanResult<HumanLoweredProofStart> {
        self.lower_proof_start_with_core_proofs(module_name, theorem_name, module, &BTreeMap::new())
    }

    fn lower_proof_start_with_core_proofs(
        &mut self,
        module_name: &npa_cert::ModuleName,
        theorem_name: &npa_cert::Name,
        module: &ResolvedHumanModule,
        by_proofs: &BTreeMap<u64, Expr>,
    ) -> HumanResult<HumanLoweredProofStart> {
        let mut prior_items = Vec::new();
        let mut declarations = module.state.source_interfaces.current.declarations.iter();
        let mut source_index = 0_u64;

        for item in &module.module.items {
            match item {
                HumanItem::Import { module, span } => {
                    let _ = (module, span);
                    prior_items.push(HumanLoweredItem::Import);
                }
                HumanItem::Def(decl) => {
                    let metadata = declarations.next().ok_or_else(|| {
                        HumanDiagnostic::not_implemented(decl.span, "Human declaration metadata")
                    })?;
                    if human_current_name_matches_target(module_name, &metadata.name, theorem_name)
                    {
                        return Err(HumanDiagnostic::unsupported_tactic(
                            decl.span,
                            "selected Human proof target is a def, not a by theorem",
                        ));
                    }
                    let lowered = self.lower_decl(decl.clone(), metadata)?;
                    let lowered = self.implicit_inserter.insert_decl(
                        lowered,
                        metadata,
                        HumanLoweredDeclKind::Def,
                    )?;
                    prior_items.push(HumanLoweredItem::Def(lowered));
                    source_index += 1;
                }
                HumanItem::Theorem(decl) => {
                    let metadata = declarations.next().ok_or_else(|| {
                        HumanDiagnostic::not_implemented(decl.span, "Human declaration metadata")
                    })?;
                    if human_current_name_matches_target(module_name, &metadata.name, theorem_name)
                    {
                        if !matches!(&decl.value, HumanDeclValue::ProofBlock(_)) {
                            return Err(HumanDiagnostic::unsupported_tactic(
                                decl.span,
                                "selected Human proof target does not use a by proof block",
                            ));
                        }
                        let lowered = self.lower_decl_signature(decl.clone(), metadata)?;
                        let target = self.implicit_inserter.insert_decl_signature(lowered)?;
                        return Ok(HumanLoweredProofStart {
                            source_index,
                            prior_items,
                            target,
                        });
                    }

                    if let HumanDeclValue::ProofBlock(block) = &decl.value {
                        let Some(proof) = by_proofs.get(&source_index) else {
                            return Err(HumanDiagnostic::unsupported_tactic(
                                block.span,
                                "prior Human by proof has not been elaborated yet",
                            )
                            .with_phase(HumanDiagnosticPhase::Elaborator));
                        };
                        let lowered = self.lower_decl_signature(decl.clone(), metadata)?;
                        let span = decl.span;
                        let core_decl = self.implicit_inserter.insert_core_theorem_decl(
                            lowered,
                            metadata,
                            proof.clone(),
                            span,
                        )?;
                        prior_items.push(HumanLoweredItem::TheoremCoreProof {
                            decl: core_decl,
                            span,
                        });
                    } else {
                        let lowered = self.lower_decl(decl.clone(), metadata)?;
                        let lowered = self.implicit_inserter.insert_decl(
                            lowered,
                            metadata,
                            HumanLoweredDeclKind::Theorem,
                        )?;
                        prior_items.push(HumanLoweredItem::Theorem(lowered));
                    }
                    source_index += 1;
                }
                HumanItem::Axiom(decl) => {
                    let metadata = declarations.next().ok_or_else(|| {
                        HumanDiagnostic::not_implemented(decl.span, "Human declaration metadata")
                    })?;
                    if human_current_name_matches_target(module_name, &metadata.name, theorem_name)
                    {
                        return Err(HumanDiagnostic::unsupported_tactic(
                            decl.span,
                            "selected Human proof target is an axiom, not a by theorem",
                        ));
                    }
                    let lowered = self.lower_axiom_decl(decl.clone(), metadata)?;
                    let lowered = self
                        .implicit_inserter
                        .insert_axiom_decl(lowered, metadata)?;
                    prior_items.push(HumanLoweredItem::Axiom(lowered));
                    source_index += 1;
                }
                HumanItem::Inductive(decl) => {
                    let metadata = declarations.next().ok_or_else(|| {
                        HumanDiagnostic::not_implemented(decl.span, "Human declaration metadata")
                    })?;
                    if human_current_name_matches_target(module_name, &metadata.name, theorem_name)
                    {
                        return Err(HumanDiagnostic::unsupported_tactic(
                            decl.span,
                            "selected Human proof target is an inductive declaration, not a by theorem",
                        ));
                    }
                    let lowered = self.lower_inductive_decl(decl.clone(), metadata)?;
                    let lowered = self
                        .implicit_inserter
                        .insert_inductive_decl(lowered, metadata)?;
                    prior_items.push(HumanLoweredItem::Inductive(lowered));
                    source_index += 1;
                }
                HumanItem::Open { .. }
                | HumanItem::NamespaceStart { .. }
                | HumanItem::NamespaceEnd { .. }
                | HumanItem::Notation(_) => {}
            }
        }

        Err(HumanDiagnostic::error(
            HumanDiagnosticKind::UnknownIdentifier,
            module.module.span,
            format!(
                "Human proof target {} was not found in the current source",
                theorem_name.as_dotted()
            ),
        ))
    }

    fn lower_decl(
        &mut self,
        decl: crate::HumanDecl,
        metadata: &HumanSourceDeclarationMetadata,
    ) -> HumanResult<MachineDecl> {
        self.meta_store.begin_declaration();
        let mut local_context = HumanLoweringLocalContext::default();
        let binders = self.lower_binders(decl.binders, &mut local_context)?;
        let ty = self.lower_expr(decl.ty, &mut local_context, None)?;
        let value = match decl.value {
            HumanDeclValue::Term(value) => self.lower_expr(value, &mut local_context, Some(&ty))?,
            HumanDeclValue::ProofBlock(block) => {
                return Err(HumanDiagnostic::unsupported_tactic(
                    block.span,
                    "by proof block elaboration is reserved for the Human tactic tactic bridge",
                )
                .with_phase(HumanDiagnosticPhase::Elaborator));
            }
        };
        self.meta_store.reject_unsolved_for_decl(decl.span)?;

        Ok(MachineDecl {
            name: self.machine_name_from_current_metadata(metadata.name.clone()),
            universe_params: decl
                .universe_params
                .into_iter()
                .map(|param| crate::MachineUniverseParam {
                    name: param.name,
                    span: param.span,
                })
                .collect(),
            binders,
            ty,
            value,
            span: decl.span,
        })
    }

    fn lower_decl_signature(
        &mut self,
        decl: crate::HumanDecl,
        metadata: &HumanSourceDeclarationMetadata,
    ) -> HumanResult<HumanLoweredDeclSignature> {
        self.meta_store.begin_declaration();
        let mut local_context = HumanLoweringLocalContext::default();
        let binders = self.lower_binders(decl.binders, &mut local_context)?;
        let ty = self.lower_expr(decl.ty, &mut local_context, None)?;
        self.meta_store.reject_unsolved_for_decl(decl.span)?;

        Ok(HumanLoweredDeclSignature {
            name: self.machine_name_from_current_metadata(metadata.name.clone()),
            universe_params: decl
                .universe_params
                .into_iter()
                .map(|param| crate::MachineUniverseParam {
                    name: param.name,
                    span: param.span,
                })
                .collect(),
            binders,
            ty,
        })
    }

    fn lower_axiom_decl(
        &mut self,
        decl: crate::HumanAxiomDecl,
        metadata: &HumanSourceDeclarationMetadata,
    ) -> HumanResult<HumanLoweredAxiomDecl> {
        self.meta_store.begin_declaration();
        let mut local_context = HumanLoweringLocalContext::default();
        let binders = self.lower_binders(decl.binders, &mut local_context)?;
        let ty = self.lower_expr(decl.ty, &mut local_context, None)?;
        self.meta_store.reject_unsolved_for_decl(decl.span)?;

        Ok(HumanLoweredAxiomDecl {
            name: self.machine_name_from_current_metadata(metadata.name.clone()),
            universe_params: decl
                .universe_params
                .into_iter()
                .map(|param| MachineUniverseParam {
                    name: param.name,
                    span: param.span,
                })
                .collect(),
            binders,
            ty,
            span: decl.span,
        })
    }

    fn lower_inductive_decl(
        &mut self,
        decl: crate::HumanInductiveDecl,
        metadata: &HumanSourceDeclarationMetadata,
    ) -> HumanResult<HumanLoweredInductiveDecl> {
        self.meta_store.begin_declaration();
        let mut local_context = HumanLoweringLocalContext::default();
        let name = self.machine_name_from_current_metadata(metadata.name.clone());
        let binders = self.lower_binders(decl.binders, &mut local_context)?;
        let ty = self.lower_expr(decl.ty, &mut local_context, None)?;
        let constructors = decl
            .constructors
            .into_iter()
            .map(|constructor| {
                let ty = self.lower_expr(constructor.ty, &mut local_context, None)?;
                Ok(HumanLoweredConstructorDecl {
                    name: machine_child_name_from_machine(&name, constructor.name),
                    ty,
                    span: constructor.span,
                })
            })
            .collect::<HumanResult<Vec<_>>>()?;
        self.meta_store.reject_unsolved_for_decl(decl.span)?;

        Ok(HumanLoweredInductiveDecl {
            name,
            universe_params: decl
                .universe_params
                .into_iter()
                .map(|param| MachineUniverseParam {
                    name: param.name,
                    span: param.span,
                })
                .collect(),
            binders,
            ty,
            constructors,
            span: decl.span,
        })
    }

    fn lower_binders(
        &mut self,
        binders: Vec<HumanBinder>,
        context: &mut HumanLoweringLocalContext,
    ) -> HumanResult<Vec<MachineBinder>> {
        let mut lowered = Vec::with_capacity(binders.len());
        let mut binders = binders.into_iter().peekable();

        while let Some(first) = binders.next() {
            let mut group = vec![first];
            while binders
                .peek()
                .is_some_and(|next| same_human_binder_group(&group[0], next))
            {
                group.push(binders.next().expect("peeked binder must exist"));
            }

            let mut group_lowered = Vec::with_capacity(group.len());
            for binder in group {
                let HumanBinderKind::Named(name) = binder.kind else {
                    return Err(HumanDiagnostic::not_implemented(
                        binder.span,
                        "anonymous Human binder lowering",
                    ));
                };
                let Some(ty) = binder.ty else {
                    return Err(HumanDiagnostic::not_implemented(
                        binder.span,
                        "unannotated Human binder lowering",
                    ));
                };
                let machine_name = name.as_dotted();
                let ty = self.lower_expr(*ty, context, None)?;
                group_lowered.push(MachineBinder {
                    name: machine_name,
                    ty,
                    span: binder.span,
                });
            }

            for binder in &group_lowered {
                context.push_assumption(binder.name.clone(), binder.ty.clone());
            }
            lowered.extend(group_lowered);
        }

        Ok(lowered)
    }

    fn lower_lambda_binders(
        &mut self,
        binders: Vec<HumanBinder>,
        context: &mut HumanLoweringLocalContext,
        expected: Option<&MachineTerm>,
    ) -> HumanResult<(Vec<MachineBinder>, Option<MachineTerm>)> {
        let mut expected = expected.cloned();
        let mut lowered = Vec::with_capacity(binders.len());
        let mut binders = binders.into_iter().peekable();

        while let Some(first) = binders.next() {
            let mut group = vec![first];
            while binders
                .peek()
                .is_some_and(|next| same_human_binder_group(&group[0], next))
            {
                group.push(binders.next().expect("peeked binder must exist"));
            }

            let mut group_lowered = Vec::with_capacity(group.len());
            for binder in group {
                let name = match binder.kind {
                    HumanBinderKind::Named(name) => name.as_dotted(),
                    HumanBinderKind::Anonymous => "_".to_owned(),
                };
                let (expected_binder, expected_body) = match expected.take() {
                    Some(expected_term) => take_expected_pi_binder(expected_term),
                    None => None,
                }
                .map_or((None, None), |(binder, body)| (Some(binder), Some(body)));

                let ty = match binder.ty {
                    Some(ty) => self.lower_expr(*ty, context, None)?,
                    None => {
                        let Some(expected_binder) = &expected_binder else {
                            return Err(HumanDiagnostic::error(
                                HumanDiagnosticKind::ExpectedFunctionType,
                                binder.span,
                                "unannotated Human lambda binder requires an expected function type",
                            ));
                        };
                        expected_binder.ty.clone()
                    }
                };

                expected = match (expected_binder, expected_body) {
                    (Some(expected_binder), Some(body)) => {
                        Some(rename_machine_local(body, &expected_binder.name, &name))
                    }
                    _ => None,
                };

                group_lowered.push(MachineBinder {
                    name,
                    ty,
                    span: binder.span,
                });
            }

            for binder in &group_lowered {
                context.push_assumption(binder.name.clone(), binder.ty.clone());
            }
            lowered.extend(group_lowered);
        }

        Ok((lowered, expected))
    }

    fn machine_name_from_global_ref(&self, reference: &HumanGlobalRef, span: Span) -> MachineName {
        match (&self.current_module_prefix, reference) {
            (
                Some(module_name),
                HumanGlobalRef::Local { name, .. } | HumanGlobalRef::LocalGenerated { name, .. },
            ) => {
                let prefixed = prefixed_human_current_name(module_name, name);
                MachineName {
                    parts: prefixed.0,
                    span,
                }
            }
            _ => machine_name_from_global_ref(reference, span),
        }
    }

    fn lower_expr(
        &mut self,
        expr: HumanExpr,
        context: &mut HumanLoweringLocalContext,
        expected: Option<&MachineTerm>,
    ) -> HumanResult<MachineTerm> {
        Ok(match expr {
            HumanExpr::Ident {
                name,
                universe_args,
                implicit_mode,
                span,
            } => {
                let resolved = self.next_name_use(&name)?;
                match resolved {
                    HumanResolvedName::Local { name, .. } => MachineTerm::Local {
                        name: name.as_dotted(),
                        span,
                    },
                    HumanResolvedName::Global(reference) => MachineTerm::Ident {
                        name: self.machine_name_from_global_ref(&reference, span),
                        universe_args: universe_args.map(|levels| {
                            levels
                                .into_iter()
                                .map(lower_level)
                                .collect::<Vec<MachineLevel>>()
                        }),
                        explicit_mode: implicit_mode == HumanImplicitMode::Explicit,
                        span,
                    },
                }
            }
            HumanExpr::Sort { level, span } => MachineTerm::Sort {
                level: lower_level(level),
                span,
            },
            HumanExpr::App { func, arg, span } => MachineTerm::App {
                func: Box::new(self.lower_expr(*func, context, None)?),
                arg: Box::new(self.lower_expr(*arg, context, None)?),
                span,
            },
            HumanExpr::Lam {
                binders,
                body,
                span,
            } => {
                let mut nested = context.clone();
                let (binders, body_expected) =
                    self.lower_lambda_binders(binders, &mut nested, expected)?;
                MachineTerm::Lam {
                    binders,
                    body: Box::new(self.lower_expr(*body, &mut nested, body_expected.as_ref())?),
                    span,
                }
            }
            HumanExpr::Pi {
                binders,
                body,
                span,
            } => {
                let mut nested = context.clone();
                MachineTerm::Pi {
                    binders: self.lower_binders(binders, &mut nested)?,
                    body: Box::new(self.lower_expr(*body, &mut nested, None)?),
                    span,
                }
            }
            HumanExpr::Let {
                name,
                ty,
                value,
                body,
                span,
            } => {
                let Some(ty) = ty else {
                    return Err(HumanDiagnostic::not_implemented(
                        span,
                        "unannotated Human let lowering",
                    ));
                };
                let ty = self.lower_expr(*ty, context, None)?;
                let value = self.lower_expr(*value, context, Some(&ty))?;
                let mut nested = context.clone();
                nested.push_definition(name.as_dotted(), ty.clone(), value.clone());
                MachineTerm::Let {
                    name: name.as_dotted(),
                    ty: Box::new(ty),
                    value: Box::new(value),
                    body: Box::new(self.lower_expr(*body, &mut nested, expected)?),
                    span,
                }
            }
            HumanExpr::Annot { expr, ty, span } => {
                let ty = self.lower_expr(*ty, context, None)?;
                MachineTerm::Annot {
                    expr: Box::new(self.lower_expr(*expr, context, Some(&ty))?),
                    ty: Box::new(ty),
                    span,
                }
            }
            HumanExpr::Arrow {
                domain,
                codomain,
                span,
            } => MachineTerm::Pi {
                binders: vec![MachineBinder {
                    name: "_".to_owned(),
                    ty: self.lower_expr(*domain, context, None)?,
                    span,
                }],
                body: Box::new(self.lower_expr(*codomain, context, None)?),
                span,
            },
            HumanExpr::Hole { name, span } => {
                let id = self
                    .meta_store
                    .fresh_user_hole(name.as_ref(), context, expected, span)?;
                human_meta_placeholder(id, span)
            }
            HumanExpr::NotationApp { head, args, span } => {
                let lowered_args = args
                    .into_iter()
                    .map(|arg| self.lower_expr(arg, context, None))
                    .collect::<HumanResult<Vec<_>>>()?;
                let notation = self.next_notation_use(&head)?;
                let choice = self.next_notation_choice(&head)?;
                let Some(candidate) = notation.candidates.get(choice) else {
                    return Err(HumanDiagnostic::error(
                        HumanDiagnosticKind::AmbiguousNotation,
                        head.span,
                        format!("notation {} candidate plan is out of range", head.token),
                    ));
                };
                let mut term = MachineTerm::Ident {
                    name: self.machine_name_from_global_ref(candidate, head.span),
                    universe_args: None,
                    explicit_mode: false,
                    span: head.span,
                };
                for arg in lowered_args {
                    let app_span = term.span().join(arg.span());
                    term = MachineTerm::App {
                        func: Box::new(term),
                        arg: Box::new(arg),
                        span: app_span,
                    };
                }
                let _ = span;
                term
            }
        })
    }

    fn next_name_use(&mut self, source: &crate::HumanName) -> HumanResult<HumanResolvedName> {
        let Some(resolved) = self.name_uses.next() else {
            return Err(HumanDiagnostic::not_implemented(
                source.span,
                "Human resolved name cursor",
            ));
        };
        debug_assert_eq!(resolved.source.as_dotted(), source.as_dotted());
        Ok(resolved.resolved.clone())
    }

    fn next_notation_use(
        &mut self,
        source: &crate::HumanNotationHead,
    ) -> HumanResult<HumanResolvedNotationUse> {
        let Some(resolved) = self.notation_uses.next() else {
            return Err(HumanDiagnostic::not_implemented(
                source.span,
                "Human resolved notation cursor",
            ));
        };
        debug_assert_eq!(resolved.head.token, source.token);
        Ok(resolved.clone())
    }

    fn next_notation_choice(&mut self, source: &crate::HumanNotationHead) -> HumanResult<usize> {
        self.notation_choices.next().copied().ok_or_else(|| {
            HumanDiagnostic::not_implemented(source.span, "Human notation choice cursor")
        })
    }
}

fn lower_level(level: HumanLevel) -> MachineLevel {
    match level {
        HumanLevel::Nat { value, span } => MachineLevel::Nat { value, span },
        HumanLevel::Param { name, span } => MachineLevel::Param { name, span },
        HumanLevel::Succ { level, span } => MachineLevel::Succ {
            level: Box::new(lower_level(*level)),
            span,
        },
        HumanLevel::Max { lhs, rhs, span } => MachineLevel::Max {
            lhs: Box::new(lower_level(*lhs)),
            rhs: Box::new(lower_level(*rhs)),
            span,
        },
        HumanLevel::IMax { lhs, rhs, span } => MachineLevel::IMax {
            lhs: Box::new(lower_level(*lhs)),
            rhs: Box::new(lower_level(*rhs)),
            span,
        },
    }
}

fn same_human_binder_group(first: &HumanBinder, next: &HumanBinder) -> bool {
    first.ty.is_some()
        && next.span == first.span
        && next.binder_info == first.binder_info
        && next.ty == first.ty
}

fn machine_name(name: crate::HumanName) -> MachineName {
    MachineName {
        parts: name.parts,
        span: name.span,
    }
}

fn machine_child_name_from_machine(parent: &MachineName, child: HumanName) -> MachineName {
    let span = child.span;
    let mut parts = parent.parts.clone();
    parts.extend(child.parts);
    MachineName { parts, span }
}

fn machine_name_from_global_ref(reference: &HumanGlobalRef, span: Span) -> MachineName {
    match reference {
        HumanGlobalRef::Imported { name, .. }
        | HumanGlobalRef::Builtin { name, .. }
        | HumanGlobalRef::Local { name, .. }
        | HumanGlobalRef::LocalGenerated { name, .. } => MachineName {
            parts: name.0.clone(),
            span,
        },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::{FileId, HumanDiagnosticKind, MachineDiagnosticKind};
    use npa_kernel::{
        eq, eq_refl, eq_refl_type, eq_type, nat, type0, Decl, Expr, Level, Reducibility,
    };

    fn hash(seed: u8) -> npa_cert::Hash {
        [seed; 32]
    }

    fn verified_axiom_module(module: &str, axiom: &str) -> npa_cert::VerifiedModule {
        let cert = npa_cert::build_module_cert(
            npa_cert::CoreModule {
                name: npa_cert::Name::from_dotted(module),
                declarations: vec![Decl::Axiom {
                    name: axiom.to_owned(),
                    universe_params: Vec::new(),
                    ty: Expr::sort(Level::zero()),
                }],
            },
            &[],
        )
        .expect("test axiom module should build a certificate");
        let bytes = npa_cert::encode_module_cert(&cert).expect("test axiom module should encode");
        npa_cert::verify_module_cert(
            &bytes,
            &mut npa_cert::VerifierSession::new(),
            &npa_cert::AxiomPolicy::normal(),
        )
        .expect("test axiom module should verify")
    }

    fn verified_import(module: &str, exports: &[(&str, &[&str])]) -> VerifiedImport {
        let exports: Vec<_> = exports
            .iter()
            .enumerate()
            .map(|(index, (name, universe_params))| {
                let name = npa_cert::Name::from_dotted(name);
                crate::VerifiedExport {
                    universe_params: universe_params
                        .iter()
                        .map(|param| param.to_string())
                        .collect(),
                    ty: export_ty(&name.as_dotted()),
                    decl_interface_hash: npa_cert::builtin_decl_interface_hash(&name)
                        .unwrap_or_else(|| hash(index as u8 + 2)),
                    name,
                }
            })
            .collect();
        let kernel_decls = exports
            .iter()
            .map(|export| Decl::Axiom {
                name: export.name.as_dotted(),
                universe_params: export.universe_params.clone(),
                ty: export.ty.clone(),
            })
            .collect();
        let decl_interface_hashes = exports
            .iter()
            .map(|export| (export.name.clone(), export.decl_interface_hash))
            .collect();

        VerifiedImport {
            module: npa_cert::Name::from_dotted(module),
            export_hash: hash(1),
            certificate_hash: None,
            exports,
            decl_interface_hashes,
            kernel_decls,
            kernel_decl_dependencies: BTreeMap::new(),
        }
    }

    fn nat_import() -> VerifiedImport {
        verified_import("Std.Nat.Basic", &[("Nat", &[])])
    }

    fn eq_import() -> VerifiedImport {
        verified_import("Std.Logic.Eq", &[("Eq", &["u"]), ("Eq.refl", &["u"])])
    }

    fn non_builtin_hash_eq_import() -> VerifiedImport {
        let mut import = eq_import();
        for export in &mut import.exports {
            if export.name == npa_cert::Name::from_dotted("Eq.refl") {
                export.decl_interface_hash = hash(99);
            }
        }
        import.decl_interface_hashes = import
            .exports
            .iter()
            .map(|export| (export.name.clone(), export.decl_interface_hash))
            .collect();
        import
    }

    fn verified_human_import(
        module: &str,
        source: &str,
    ) -> (
        VerifiedImport,
        HumanImportedSourceInterface,
        npa_cert::VerifiedModule,
    ) {
        let module_name = npa_cert::Name::from_dotted(module);
        let output = compile_human_source_to_certificate_output_with_source_interfaces(
            FileId(0),
            module_name.clone(),
            source,
            &[],
            &[],
            &HumanCompileOptions::default(),
        )
        .expect("producer Human source should build a certificate");
        let bytes =
            npa_cert::encode_module_cert(&output.certificate).expect("producer cert should encode");
        let verified = npa_cert::verify_module_cert(
            &bytes,
            &mut npa_cert::VerifierSession::new(),
            &npa_cert::AxiomPolicy::normal(),
        )
        .expect("producer cert should verify");
        let import = VerifiedImport::from(&verified);
        let source_interface = HumanImportedSourceInterface {
            module: import.module.clone(),
            export_hash: import.export_hash,
            certificate_hash: import.certificate_hash,
            source_interface: output.source_interface,
        };

        (import, source_interface, verified)
    }

    fn export_ty(name: &str) -> Expr {
        match name {
            "Nat" => Expr::sort(type0()),
            "Eq" => eq_type(Level::param("u")),
            "Eq.refl" => eq_refl_type(Level::param("u")),
            _ => Expr::sort(Level::zero()),
        }
    }

    fn collect_const_level_args(expr: &Expr, target: &str, out: &mut Vec<Vec<Level>>) {
        match expr {
            Expr::Const { name, levels } => {
                if name == target {
                    out.push(levels.clone());
                }
            }
            Expr::App(fun, arg) => {
                collect_const_level_args(fun, target, out);
                collect_const_level_args(arg, target, out);
            }
            Expr::Lam { ty, body, .. } | Expr::Pi { ty, body, .. } => {
                collect_const_level_args(ty, target, out);
                collect_const_level_args(body, target, out);
            }
            Expr::Let {
                ty, value, body, ..
            } => {
                collect_const_level_args(ty, target, out);
                collect_const_level_args(value, target, out);
                collect_const_level_args(body, target, out);
            }
            Expr::Sort(_) | Expr::BVar(_) => {}
        }
    }

    fn def_value<'a>(module: &'a npa_cert::CoreModule, target: &str) -> &'a Expr {
        module
            .declarations
            .iter()
            .find_map(|decl| match decl {
                Decl::Def { name, value, .. } if name == target => Some(value),
                _ => None,
            })
            .unwrap_or_else(|| panic!("expected def {target}"))
    }

    fn level_has_internal_human_meta(level: &Level) -> bool {
        match level {
            Level::Zero => false,
            Level::Succ(inner) => level_has_internal_human_meta(inner),
            Level::Max(lhs, rhs) | Level::IMax(lhs, rhs) => {
                level_has_internal_human_meta(lhs) || level_has_internal_human_meta(rhs)
            }
            Level::Param(name) => name.starts_with(HUMAN_UNIVERSE_META_PREFIX),
        }
    }

    fn expr_has_internal_human_meta(expr: &Expr) -> bool {
        match expr {
            Expr::Sort(level) => level_has_internal_human_meta(level),
            Expr::Const { name, levels } => {
                name.starts_with(HUMAN_SPINE_IMPLICIT_PREFIX)
                    || levels.iter().any(level_has_internal_human_meta)
            }
            Expr::App(fun, arg) => {
                expr_has_internal_human_meta(fun) || expr_has_internal_human_meta(arg)
            }
            Expr::Lam { ty, body, .. } | Expr::Pi { ty, body, .. } => {
                expr_has_internal_human_meta(ty) || expr_has_internal_human_meta(body)
            }
            Expr::Let {
                ty, value, body, ..
            } => {
                expr_has_internal_human_meta(ty)
                    || expr_has_internal_human_meta(value)
                    || expr_has_internal_human_meta(body)
            }
            Expr::BVar(_) => false,
        }
    }

    fn decl_has_internal_human_meta(decl: &Decl) -> bool {
        if decl
            .universe_params()
            .iter()
            .any(|param| param.starts_with(HUMAN_UNIVERSE_META_PREFIX))
        {
            return true;
        }
        if decl.universe_constraints().iter().any(|constraint| {
            level_has_internal_human_meta(&constraint.lhs)
                || level_has_internal_human_meta(&constraint.rhs)
        }) {
            return true;
        }
        if expr_has_internal_human_meta(decl.ty()) {
            return true;
        }
        match decl {
            Decl::Def { value, .. } | Decl::DefConstrained { value, .. } => {
                expr_has_internal_human_meta(value)
            }
            Decl::Theorem { proof, .. } | Decl::TheoremConstrained { proof, .. } => {
                expr_has_internal_human_meta(proof)
            }
            Decl::Inductive { data, .. } => {
                data.params
                    .iter()
                    .chain(data.indices.iter())
                    .any(|binder| expr_has_internal_human_meta(&binder.ty))
                    || level_has_internal_human_meta(&data.sort)
                    || data
                        .constructors
                        .iter()
                        .any(|ctor| expr_has_internal_human_meta(&ctor.ty))
                    || data
                        .recursor
                        .as_ref()
                        .is_some_and(|recursor| expr_has_internal_human_meta(&recursor.ty))
            }
            Decl::Axiom { .. }
            | Decl::AxiomConstrained { .. }
            | Decl::Constructor { .. }
            | Decl::Recursor { .. } => false,
        }
    }

    fn assert_core_module_has_no_internal_human_metas(module: &npa_cert::CoreModule) {
        assert!(
            !module.declarations.iter().any(decl_has_internal_human_meta),
            "Human elaboration-only metas must not reach the core module"
        );
    }

    fn assert_certificate_has_no_internal_human_metas(cert: &npa_cert::ModuleCert) {
        assert!(
            cert.name_table.iter().all(|name| {
                let dotted = name.as_dotted();
                !dotted.starts_with(HUMAN_UNIVERSE_META_PREFIX)
                    && !dotted.starts_with(HUMAN_SPINE_IMPLICIT_PREFIX)
            }),
            "Human elaboration-only metas must not reach the certificate name table"
        );
    }

    #[test]
    fn compile_human_source_checks_verified_imports_before_elaboration() {
        let err = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Current.Module"),
            "import Std.Nat.Basic",
            &[],
            &HumanCompileOptions::default(),
        )
        .expect_err("missing import should fail during Human resolution");

        assert_eq!(err.kind, HumanDiagnosticKind::MissingVerifiedImport);
    }

    #[test]
    fn human_axiom_elaborates_to_core_axiom_declaration() {
        let module = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "axiom P : Prop",
            &[],
            &HumanCompileOptions::default(),
        )
        .expect("Human source-level axiom should elaborate to a core axiom");

        assert_eq!(
            module.declarations,
            vec![Decl::Axiom {
                name: "P".to_owned(),
                universe_params: Vec::new(),
                ty: Expr::sort(Level::zero()),
            }]
        );
    }

    #[test]
    fn human_axiom_is_available_to_later_declarations() {
        let module = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
axiom P : Prop
axiom p : P
theorem use : P := p",
            &[],
            &HumanCompileOptions::default(),
        )
        .expect("Human axiom should enter the global environment after kernel acceptance");

        assert_eq!(module.declarations.len(), 3);
        let Decl::Theorem { proof, .. } = &module.declarations[2] else {
            panic!("expected theorem");
        };
        assert_eq!(proof, &Expr::konst("p", vec![]));
    }

    #[test]
    fn human_axiom_certificate_reports_axiom_and_high_trust_requires_allowlist() {
        let cert = compile_human_source_to_certificate(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "axiom P : Prop",
            &[],
            &HumanCompileOptions::default(),
        )
        .expect("Human axiom source should build a certificate certificate");

        assert_eq!(cert.axiom_report.module_axioms.len(), 1);
        let axiom_ref = &cert.axiom_report.module_axioms[0];
        assert_eq!(
            cert.name_table[axiom_ref.name],
            npa_cert::Name::from_dotted("P")
        );

        let bytes =
            npa_cert::encode_module_cert(&cert).expect("Human axiom certificate should encode");
        let err = npa_cert::verify_module_cert(
            &bytes,
            &mut npa_cert::VerifierSession::new(),
            &npa_cert::AxiomPolicy::high_trust(),
        )
        .expect_err("high-trust verification should reject an unallowlisted Human axiom");
        assert!(matches!(
            err,
            npa_cert::CertError::ForbiddenAxiom { axiom }
                if axiom == npa_cert::Name::from_dotted("P")
        ));

        let mut policy = npa_cert::AxiomPolicy::high_trust();
        policy
            .allowlisted_axioms
            .insert(npa_cert::Name::from_dotted("P"));
        npa_cert::verify_module_cert(&bytes, &mut npa_cert::VerifierSession::new(), &policy)
            .expect("allowlisted Human axiom should verify in high-trust mode");
    }

    #[test]
    fn human_axiom_certificate_omits_unimported_verified_modules() {
        let unused = verified_axiom_module("Unused", "Unused.P");
        let cert = compile_human_source_to_certificate(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "axiom P : Prop",
            &[unused],
            &HumanCompileOptions::default(),
        )
        .expect("available but unimported verified modules should not enter the certificate");

        assert!(cert.imports.is_empty());
        assert_eq!(cert.axiom_report.module_axioms.len(), 1);
    }

    #[test]
    fn human_simple_inductive_elaborates_to_core_inductive_declaration() {
        let module = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
inductive Nat : Type where
| zero : Nat
| succ : forall (n : Nat), Nat",
            &[],
            &HumanCompileOptions::default(),
        )
        .expect("Human simple inductive should elaborate to a core inductive");

        assert_eq!(module.declarations.len(), 1);
        let Decl::Inductive {
            name,
            universe_params,
            ty,
            data,
        } = &module.declarations[0]
        else {
            panic!("expected inductive declaration");
        };
        assert_eq!(name, "Nat");
        assert_eq!(universe_params, &Vec::<String>::new());
        assert_eq!(ty, &Expr::sort(type0()));
        assert_eq!(data.name, "Nat");
        assert!(data.params.is_empty());
        assert!(data.indices.is_empty());
        assert_eq!(data.sort, type0());
        assert_eq!(
            data.constructors,
            vec![
                ConstructorDecl::new("Nat.zero", Expr::konst("Nat", vec![])),
                ConstructorDecl::new(
                    "Nat.succ",
                    Expr::pi("n", Expr::konst("Nat", vec![]), Expr::konst("Nat", vec![]))
                ),
            ]
        );
        assert_eq!(
            data.recursor
                .as_ref()
                .map(|recursor| recursor.name.as_str()),
            Some("Nat.rec")
        );
    }

    #[test]
    fn human_simple_inductive_generated_constructor_is_available_later() {
        let module = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
inductive Nat : Type where
| zero : Nat
| succ : forall (n : Nat), Nat
def z : Nat := Nat.zero
def one : Nat := Nat.succ Nat.zero",
            &[],
            &HumanCompileOptions::default(),
        )
        .expect("Human generated constructor should enter global scope after kernel acceptance");

        assert_eq!(module.declarations.len(), 3);
        let Decl::Def { value, .. } = &module.declarations[1] else {
            panic!("expected z definition");
        };
        assert_eq!(value, &Expr::konst("Nat.zero", vec![]));
        let Decl::Def { value, .. } = &module.declarations[2] else {
            panic!("expected one definition");
        };
        assert_eq!(
            value,
            &Expr::app(
                Expr::konst("Nat.succ", vec![]),
                Expr::konst("Nat.zero", vec![])
            )
        );
    }

    #[test]
    fn human_simple_inductive_certificate_exports_generated_artifacts() {
        let cert = compile_human_source_to_certificate(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
inductive Nat : Type where
| zero : Nat
| succ : forall (n : Nat), Nat",
            &[],
            &HumanCompileOptions::default(),
        )
        .expect("Human simple inductive should build a certificate certificate");

        assert!(cert.export_block.iter().any(|entry| {
            entry.kind == npa_cert::ExportKind::Constructor
                && cert.name_table[entry.name] == npa_cert::Name::from_dotted("Nat.zero")
        }));
        assert!(cert.export_block.iter().any(|entry| {
            entry.kind == npa_cert::ExportKind::Constructor
                && cert.name_table[entry.name] == npa_cert::Name::from_dotted("Nat.succ")
        }));
        assert!(cert.export_block.iter().any(|entry| {
            entry.kind == npa_cert::ExportKind::Recursor
                && cert.name_table[entry.name] == npa_cert::Name::from_dotted("Nat.rec")
        }));
    }

    #[test]
    fn human_certificate_output_hashes_source_interface_exports() {
        let output = compile_human_source_to_certificate_output_with_source_interfaces(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
inductive Nat : Type where
| zero : Nat
| succ : forall (n : Nat), Nat",
            &[],
            &[],
            &HumanCompileOptions::default(),
        )
        .expect("Human certificate output should include source interface metadata");
        let cert = output.certificate;
        let source_interface = output.source_interface;
        let export_hash = |name: &str| {
            cert.export_block
                .iter()
                .find(|entry| cert.name_table[entry.name] == npa_cert::Name::from_dotted(name))
                .map(|entry| entry.decl_interface_hash)
                .expect("expected source interface name to be exported")
        };

        assert_eq!(
            source_interface.declarations[0].decl_interface_hash,
            Some(export_hash("Nat"))
        );
        assert_eq!(source_interface.generated_declarations.len(), 3);
        assert_eq!(
            source_interface.generated_declarations[0].decl_interface_hash,
            Some(export_hash("Nat.zero"))
        );
        assert_eq!(
            source_interface.generated_declarations[1].decl_interface_hash,
            Some(export_hash("Nat.succ"))
        );
        assert_eq!(
            source_interface.generated_declarations[2].decl_interface_hash,
            Some(export_hash("Nat.rec"))
        );
    }

    #[test]
    fn human_indexed_inductive_constructor_uses_temporary_head_implicit_profile() {
        let module = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
inductive Eq.{u} {A : Sort u} (a : A) : forall (b : A), Prop where
| refl : Eq.{u} a a",
            &[],
            &HumanCompileOptions::default(),
        )
        .expect("Human indexed inductive should insert implicit params for constructor result");

        let Decl::Inductive { data, .. } = &module.declarations[0] else {
            panic!("expected inductive declaration");
        };
        assert_eq!(data.params.len(), 2);
        assert_eq!(data.indices.len(), 1);
        assert_eq!(data.constructors[0].name, "Eq.refl");
        assert_eq!(
            data.constructors[0].ty,
            Expr::pi(
                "A",
                Expr::sort(Level::param("u")),
                Expr::pi(
                    "a",
                    Expr::bvar(0),
                    Expr::apps(
                        Expr::konst("Eq", vec![Level::param("u")]),
                        vec![Expr::bvar(1), Expr::bvar(0), Expr::bvar(0)]
                    )
                )
            )
        );
    }

    #[test]
    fn human_imported_eq_module_allows_builtin_eq_rec_reference() {
        let (_, source_interface, verified) = verified_human_import(
            "Std.Logic.Eq",
            "\
inductive Eq.{u} {A : Sort u} (a : A) : forall (b : A), Prop where
| refl : Eq.{u} a a",
        );
        let output = compile_human_source_to_certificate_output_with_source_interfaces(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Logic.Eq
def copy_eq_rec.{u,v} :
  forall (A : Sort u), forall (a : A), forall (motive : forall (b : A), forall (h : @Eq.{u} A a b), Sort v), forall (minor : motive a (@Eq.refl.{u} A a)), forall (b : A), forall (h : @Eq.{u} A a b), motive b h :=
  @Eq.rec.{u,v}",
            &[verified],
            &[source_interface],
            &HumanCompileOptions::default(),
        )
        .expect("Human imported Eq should allow canonical builtin Eq.rec references");

        let axioms = output
            .certificate
            .axiom_report
            .module_axioms
            .iter()
            .map(|axiom| output.certificate.name_table[axiom.name].as_dotted())
            .collect::<Vec<_>>();
        assert_eq!(axioms, vec!["Eq.rec"]);
    }

    #[test]
    fn human_bad_inductive_constructor_type_is_kernel_rejected() {
        let err = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
inductive Bad : Type where
| bad : Type",
            &[],
            &HumanCompileOptions::default(),
        )
        .expect_err("bad constructor result should be rejected by kernel handoff");

        assert_eq!(err.kind, HumanDiagnosticKind::KernelRejected);
        assert_eq!(
            err.payload.as_ref().and_then(|payload| payload.phase),
            Some(HumanDiagnosticPhase::KernelHandoff)
        );
    }

    #[test]
    fn elaborates_single_candidate_infix_notation_to_machine_application() {
        let module = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
def add (n m : Sort 2) : Sort 2 := n
infixl:65 \" + \" => add
def use (n : Sort 2) : Sort 2 := n + Type",
            &[],
            &HumanCompileOptions::default(),
        )
        .expect("single-candidate notation should elaborate through the Machine path");

        let sort_1 = Expr::sort(Level::succ(Level::zero()));
        let sort_2 = Expr::sort(Level::succ(Level::succ(Level::zero())));
        assert_eq!(module.declarations.len(), 2);
        assert_eq!(
            module.declarations[1],
            Decl::Def {
                name: "use".to_owned(),
                universe_params: Vec::new(),
                ty: Expr::pi("n", sort_2.clone(), sort_2.clone()),
                value: Expr::lam(
                    "n",
                    sort_2,
                    Expr::app(Expr::app(Expr::konst("add", vec![]), Expr::bvar(0)), sort_1)
                ),
                reducibility: Reducibility::Reducible,
            }
        );
    }

    #[test]
    fn notation_elaboration_rolls_back_failed_candidate_and_uses_successful_one() {
        let module = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
def add_bad (n m : Type) : Type := n
def add_ok (n m : Sort 2) : Sort 2 := n
infixl:65 \" + \" => add_bad
infixl:65 \" + \" => add_ok
def use (n : Sort 2) : Sort 2 := n + Type",
            &[],
            &HumanCompileOptions::default(),
        )
        .expect("elaboration should try the second notation candidate after the first fails");

        assert_eq!(module.declarations.len(), 3);
        let Decl::Def { value, .. } = &module.declarations[2] else {
            panic!("expected use definition");
        };
        assert_eq!(
            value,
            &Expr::lam(
                "n",
                Expr::sort(Level::succ(Level::succ(Level::zero()))),
                Expr::app(
                    Expr::app(Expr::konst("add_ok", vec![]), Expr::bvar(0)),
                    Expr::sort(Level::succ(Level::zero()))
                )
            )
        );
    }

    #[test]
    fn human_notation_candidate_count_limit_rejects_before_elaboration() {
        let options = HumanCompileOptions {
            max_notation_candidates: 1,
        };
        let err = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
def add_a (n m : Type) : Type := n
def add_b (n m : Type) : Type := m
infixl:65 \" + \" => add_a
infixl:65 \" + \" => add_b
def use (n : Type) : Type := n + Type",
            &[],
            &options,
        )
        .expect_err("Human notation overloads above the configured limit should fail");

        assert_eq!(err.kind, HumanDiagnosticKind::TooManyNotationCandidates);
        let payload = err
            .payload
            .expect("candidate count limit should carry a bounded payload");
        assert_eq!(payload.phase, Some(HumanDiagnosticPhase::Resolver));
        assert_eq!(payload.candidates.len(), 1);
    }

    #[test]
    fn human_path_inserts_implicit_type_argument_for_eq_refl() {
        let imports = [nat_import(), eq_import()];
        let module = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Nat.Basic
import Std.Logic.Eq
theorem self_eq (n : Nat) : Eq.{1} Nat n n := Eq.refl n",
            &imports,
            &HumanCompileOptions::default(),
        )
        .expect("Human path should insert Eq.refl implicit type argument");

        assert_eq!(module.declarations.len(), 1);
        let Decl::Theorem { proof, .. } = &module.declarations[0] else {
            panic!("expected theorem");
        };
        assert_eq!(
            proof,
            &Expr::lam("n", nat(), eq_refl(type0(), nat(), Expr::bvar(0)))
        );
    }

    #[test]
    fn human_universe_solver_infers_polymorphic_id_const_and_map() {
        let module = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
def id.{u} {A : Sort u} (x : A) : A := x
def const.{u,v} {A : Sort u} {B : Sort v} (x : A) (y : B) : A := x
axiom map.{u} {A : Sort u} (f : forall (a : A), A) (x : A) : A
def use_id.{u} {A : Sort u} (x : A) : A := id x
def use_const.{u,v} {A : Sort u} {B : Sort v} (x : A) (y : B) : A := const x y
theorem use_map.{u} {A : Sort u} (f : forall (a : A), A) (x : A) : A := map f x",
            &[],
            &HumanCompileOptions::default(),
        )
        .expect(
            "Human path should infer implicit universe arguments for common polymorphic spines",
        );

        let mut id_levels = Vec::new();
        collect_const_level_args(def_value(&module, "use_id"), "id", &mut id_levels);
        assert!(id_levels.contains(&vec![Level::param("u")]));

        let mut const_levels = Vec::new();
        collect_const_level_args(def_value(&module, "use_const"), "const", &mut const_levels);
        assert!(const_levels.contains(&vec![Level::param("u"), Level::param("v")]));

        let Decl::Theorem { proof, .. } = &module.declarations[5] else {
            panic!("expected use_map theorem");
        };
        let mut map_levels = Vec::new();
        collect_const_level_args(proof, "map", &mut map_levels);
        assert!(map_levels.contains(&vec![Level::param("u")]));
        assert_core_module_has_no_internal_human_metas(&module);
    }

    #[test]
    fn human_universe_solver_reports_unsatisfied_constraint() {
        let err = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
def id.{u} {A : Sort u} (x : A) : A := x
def bad.{u,v} {A : Sort u} (x : A) : A := id.{v} x",
            &[],
            &HumanCompileOptions::default(),
        )
        .expect_err("fixed explicit universe arguments must reject unsatisfied Human constraints");

        assert_eq!(err.kind, HumanDiagnosticKind::UnsolvedUniverseMeta);
        assert_eq!(
            err.payload
                .as_ref()
                .and_then(|payload| payload.unsolved_meta.as_ref())
                .map(|meta| meta.kind),
            Some(HumanUnsolvedMetaKind::Universe)
        );
    }

    #[test]
    fn human_universe_solver_is_deterministic_and_certificate_meta_free() {
        let source = "\
def id.{u} {A : Sort u} (x : A) : A := x
def use_id.{u} {A : Sort u} (x : A) : A := id x";
        let module = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            source,
            &[],
            &HumanCompileOptions::default(),
        )
        .expect("Human source should elaborate to meta-free core");
        assert_core_module_has_no_internal_human_metas(&module);

        let first = compile_human_source_to_certificate(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            source,
            &[],
            &HumanCompileOptions::default(),
        )
        .expect("first Human certificate should build");
        let second = compile_human_source_to_certificate(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            source,
            &[],
            &HumanCompileOptions::default(),
        )
        .expect("second Human certificate should build");

        assert_eq!(
            first.hashes.certificate_hash,
            second.hashes.certificate_hash
        );
        assert_certificate_has_no_internal_human_metas(&first);
        assert_certificate_has_no_internal_human_metas(&second);
    }

    #[test]
    fn human_ambiguous_universe_reports_structured_diagnostic() {
        let err = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
axiom F.{u} : Sort u
def bad : Type := F",
            &[],
            &HumanCompileOptions::default(),
        )
        .expect_err("ambiguous universe metavariable should be reported before core handoff");

        assert_eq!(err.kind, HumanDiagnosticKind::UnsolvedUniverseMeta);
        assert_eq!(
            err.payload
                .as_ref()
                .and_then(|payload| payload.unsolved_meta.as_ref())
                .map(|meta| meta.kind),
            Some(HumanUnsolvedMetaKind::Universe)
        );
    }

    #[test]
    fn human_explicit_mode_suppresses_implicit_insertion() {
        let imports = [nat_import(), eq_import()];
        let err = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Nat.Basic
import Std.Logic.Eq
theorem bad (n : Nat) : Eq.{1} Nat n n := @Eq.refl.{1} n",
            &imports,
            &HumanCompileOptions::default(),
        )
        .expect_err("@ mode should not insert the implicit type argument");

        assert_eq!(err.kind, HumanDiagnosticKind::TypeMismatch);
    }

    #[test]
    fn human_explicit_mode_accepts_explicit_implicit_argument() {
        let imports = [nat_import(), eq_import()];
        compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Nat.Basic
import Std.Logic.Eq
theorem self_eq (n : Nat) : Eq.{1} Nat n n := @Eq.refl.{1} Nat n",
            &imports,
            &HumanCompileOptions::default(),
        )
        .expect("explicit @ mode should accept an explicitly supplied implicit argument");
    }

    #[test]
    fn human_expected_type_elaborates_unannotated_lambda_to_core_declaration() {
        let imports = [nat_import()];
        let module = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Nat.Basic
def id : forall (x : Nat), Nat := fun x => x",
            &imports,
            &HumanCompileOptions::default(),
        )
        .expect("Human checker should use the expected Pi type for lambda binders");

        assert_eq!(
            module.declarations,
            vec![Decl::Def {
                name: "id".to_owned(),
                universe_params: Vec::new(),
                ty: Expr::pi("x", nat(), nat()),
                value: Expr::lam("x", nat(), Expr::bvar(0)),
                reducibility: Reducibility::Reducible,
            }]
        );
    }

    #[test]
    fn human_ill_typed_term_returns_structured_type_mismatch() {
        let imports = [nat_import()];
        let err = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Nat.Basic
def bad : Nat := Type",
            &imports,
            &HumanCompileOptions::default(),
        )
        .expect_err("ill-typed Human value should be rejected as a structured diagnostic");

        assert_eq!(err.kind, HumanDiagnosticKind::TypeMismatch);
    }

    #[test]
    fn human_unannotated_lambda_requires_expected_function_type() {
        let err = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "def bad : Type := fun x => x",
            &[],
            &HumanCompileOptions::default(),
        )
        .expect_err("unannotated lambda should not trigger open-ended search");

        assert_eq!(err.kind, HumanDiagnosticKind::ExpectedFunctionType);
    }

    #[test]
    fn human_builtin_profile_requires_builtin_interface_hash() {
        let imports = [nat_import(), non_builtin_hash_eq_import()];
        let err = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Nat.Basic
import Std.Logic.Eq
theorem bad (n : Nat) : Eq.{1} Nat n n := Eq.refl n",
            &imports,
            &HumanCompileOptions::default(),
        )
        .expect_err("name-only Eq.refl should not get the builtin implicit profile");

        assert_eq!(err.kind, HumanDiagnosticKind::UnsolvedUniverseMeta);
        assert_eq!(
            err.payload
                .as_ref()
                .and_then(|payload| payload.unsolved_meta.as_ref())
                .map(|meta| meta.kind),
            Some(HumanUnsolvedMetaKind::Universe)
        );
    }

    #[test]
    fn human_tactic_term_check_resolves_goal_local() {
        let imports = [nat_import()];
        let local_context = vec![MachineLocalDecl {
            name: "n".to_owned(),
            ty: nat(),
            value: None,
        }];
        let context = HumanTacticTermElabContext::from_request(HumanTacticTermElabContextRequest {
            direct_imports: &imports,
            available_imports: &imports,
            current_module: npa_cert::Name::from_dotted("Api.Target"),
            checked_current_decls: &[],
            current_generated_decls: &[],
            local_context,
            universe_params: Vec::new(),
            current_source_interface: None,
            imported_source_interfaces: &[],
        })
        .expect("Human tactic context should accept a Nat local");
        let term = crate::parse_human_term(FileId(0), "n").expect("term should parse");
        let output = elaborate_human_tactic_term_check(
            &context,
            &term,
            &nat(),
            &HumanCompileOptions::default(),
        )
        .expect("Human tactic check should resolve the goal local");

        assert_eq!(output.expr, Expr::bvar(0));
        assert_eq!(output.inferred_type, nat());

        let inferred =
            elaborate_human_tactic_term_infer(&context, &term, &HumanCompileOptions::default())
                .expect("Human tactic infer should resolve the same goal local");
        assert_eq!(inferred.expr, Expr::bvar(0));
        assert_eq!(inferred.inferred_type, nat());
    }

    #[test]
    fn human_tactic_term_check_inserts_eq_refl_implicit() {
        let imports = [nat_import(), eq_import()];
        let local_context = vec![MachineLocalDecl {
            name: "n".to_owned(),
            ty: nat(),
            value: None,
        }];
        let context = HumanTacticTermElabContext::from_request(HumanTacticTermElabContextRequest {
            direct_imports: &imports,
            available_imports: &imports,
            current_module: npa_cert::Name::from_dotted("Api.Target"),
            checked_current_decls: &[],
            current_generated_decls: &[],
            local_context,
            universe_params: Vec::new(),
            current_source_interface: None,
            imported_source_interfaces: &[],
        })
        .expect("Human tactic context should accept Nat and Eq imports");
        let term = crate::parse_human_term(FileId(0), "Eq.refl n").expect("term should parse");
        let expected = eq(type0(), nat(), Expr::bvar(0), Expr::bvar(0));
        let output = elaborate_human_tactic_term_check(
            &context,
            &term,
            &expected,
            &HumanCompileOptions::default(),
        )
        .expect("Human tactic check should insert Eq.refl implicit type argument");

        assert_eq!(output.expr, eq_refl(type0(), nat(), Expr::bvar(0)));
        assert_eq!(output.inferred_type, expected);
    }

    #[test]
    fn human_tactic_term_check_rejects_unresolved_hole_before_certificate() {
        let imports = [nat_import()];
        let context = HumanTacticTermElabContext::from_request(HumanTacticTermElabContextRequest {
            direct_imports: &imports,
            available_imports: &imports,
            current_module: npa_cert::Name::from_dotted("Api.Target"),
            checked_current_decls: &[],
            current_generated_decls: &[],
            local_context: Vec::new(),
            universe_params: Vec::new(),
            current_source_interface: None,
            imported_source_interfaces: &[],
        })
        .expect("Human tactic context should build");
        let term = crate::parse_human_term(FileId(0), "_").expect("hole should parse");
        let err = elaborate_human_tactic_term_check(
            &context,
            &term,
            &nat(),
            &HumanCompileOptions::default(),
        )
        .expect_err("unresolved Human tactic hole should stop before certificate handoff");

        assert_eq!(err.kind, HumanDiagnosticKind::UnsolvedHole);
        assert_eq!(
            err.payload
                .as_ref()
                .and_then(|payload| payload.unsolved_meta.as_ref())
                .map(|meta| meta.kind),
            Some(HumanUnsolvedMetaKind::Hole)
        );
    }

    #[test]
    fn human_tactic_term_resolves_checked_current_decl_and_generated_constructor() {
        let imports = [nat_import()];
        let id_name = npa_cert::Name::from_dotted("Api.Target.id_type");
        let id_decl = Decl::Def {
            name: id_name.as_dotted(),
            universe_params: Vec::new(),
            ty: Expr::pi("A", Expr::sort(type0()), Expr::sort(type0())),
            value: Expr::lam("A", Expr::sort(type0()), Expr::bvar(0)),
            reducibility: Reducibility::Reducible,
        };
        let unit_name = npa_cert::Name::from_dotted("Api.Target.Unit");
        let unit_mk_name = npa_cert::Name::from_dotted("Api.Target.Unit.mk");
        let unit_data = InductiveDecl::new(
            unit_name.as_dotted(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            type0(),
            vec![ConstructorDecl::new(
                unit_mk_name.as_dotted(),
                Expr::konst(unit_name.as_dotted(), vec![]),
            )],
            None,
        );
        let unit_decl = Decl::Inductive {
            name: unit_name.as_dotted(),
            universe_params: Vec::new(),
            ty: Expr::sort(type0()),
            data: Box::new(unit_data),
        };
        let checked_current_decls = vec![
            MachineCheckedCurrentDecl {
                name: id_name.clone(),
                source_index: 0,
                decl_interface_hash: hash(42),
                decl: id_decl,
            },
            MachineCheckedCurrentDecl {
                name: unit_name.clone(),
                source_index: 1,
                decl_interface_hash: hash(43),
                decl: unit_decl,
            },
        ];
        let current_generated_decls = vec![MachineCheckedCurrentGeneratedDecl {
            name: unit_mk_name.clone(),
            parent_source_index: 1,
            decl_interface_hash: hash(43),
        }];
        let context = HumanTacticTermElabContext::from_request(HumanTacticTermElabContextRequest {
            direct_imports: &imports,
            available_imports: &imports,
            current_module: npa_cert::Name::from_dotted("Api.Target"),
            checked_current_decls: &checked_current_decls,
            current_generated_decls: &current_generated_decls,
            local_context: Vec::new(),
            universe_params: Vec::new(),
            current_source_interface: None,
            imported_source_interfaces: &[],
        })
        .expect("Human tactic context should include checked current declarations");

        let id_term =
            crate::parse_human_term(FileId(0), "id_type Nat").expect("id_type term should parse");
        let id_output = elaborate_human_tactic_term_check(
            &context,
            &id_term,
            &Expr::sort(type0()),
            &HumanCompileOptions::default(),
        )
        .expect("Human tactic term should resolve checked current declaration by short name");
        assert_eq!(
            id_output.expr,
            Expr::app(Expr::konst(id_name.as_dotted(), vec![]), nat())
        );

        let mk_term =
            crate::parse_human_term(FileId(0), "Unit.mk").expect("Unit.mk term should parse");
        let mk_output = elaborate_human_tactic_term_check(
            &context,
            &mk_term,
            &Expr::konst(unit_name.as_dotted(), vec![]),
            &HumanCompileOptions::default(),
        )
        .expect("Human tactic term should resolve generated current constructor by suffix");
        assert_eq!(
            mk_output.expr,
            Expr::konst(unit_mk_name.as_dotted(), vec![])
        );
    }

    #[test]
    fn human_current_callable_profile_uses_implicit_binder_metadata() {
        let imports = [nat_import()];
        let module = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Nat.Basic
def id {A : Type} (x : A) : A := x
def use (n : Nat) : Nat := id n",
            &imports,
            &HumanCompileOptions::default(),
        )
        .expect("Human path should insert current declaration implicit type argument");

        assert_eq!(module.declarations.len(), 2);
        let Decl::Def { value, .. } = &module.declarations[1] else {
            panic!("expected def");
        };
        assert_eq!(
            value,
            &Expr::lam(
                "n",
                nat(),
                Expr::app(Expr::app(Expr::konst("id", vec![]), nat()), Expr::bvar(0))
            )
        );
    }

    #[test]
    fn human_proof_start_prefixes_only_resolved_current_refs() {
        let imports = [verified_import("Api.Lib", &[("foo", &[])])];
        let prepared = prepare_human_proof_start_core_with_source_interfaces(
            FileId(0),
            npa_cert::Name::from_dotted("Api.Target"),
            npa_cert::Name::from_dotted("Api.Target.target"),
            "\
import Api.Lib
axiom use_import : foo
theorem target : Prop := by simp-lite
def foo : forall (A : Type), Type := fun A => A",
            &imports,
            &[],
            &HumanCompileOptions::default(),
        )
        .expect("proof start preparation should keep imported refs distinct from current names");

        assert_eq!(prepared.proof.prior_declarations.len(), 1);
        let Decl::Axiom { name, ty, .. } = &prepared.proof.prior_declarations[0] else {
            panic!("expected prior axiom");
        };
        assert_eq!(name, "Api.Target.use_import");
        assert_eq!(ty, &Expr::konst("foo", vec![]));
        assert_eq!(
            prepared.proof.theorem_name,
            npa_cert::Name::from_dotted("Api.Target.target")
        );
    }

    #[test]
    fn human_proof_start_ignores_suffix_notation_choices() {
        let prepared = prepare_human_proof_start_core_with_source_interfaces(
            FileId(0),
            npa_cert::Name::from_dotted("Api.Target"),
            npa_cert::Name::from_dotted("Api.Target.target"),
            "\
theorem target : Prop := by simp-lite
def pick_left {A : Type} (x y : A) : A := x
def pick_right {A : Type} (x y : A) : A := y
infixl:65 \" ** \" => pick_right
infixl:65 \" ** \" => pick_left
def later (A : Type) (x : A) : A := x ** x",
            &[],
            &[],
            &HumanCompileOptions::default(),
        )
        .expect("proof start should not enumerate notation choices after the target theorem");

        assert_eq!(prepared.proof.prior_declarations.len(), 0);
        assert_eq!(prepared.proof.source_index, 0);
        assert_eq!(prepared.source_interface.declarations.len(), 4);
    }

    #[test]
    fn human_by_proof_core_indices_must_match_by_theorems() {
        let err = compile_human_source_to_core_output_with_source_interfaces_and_by_proofs(
            FileId(0),
            npa_cert::Name::from_dotted("Api.Target"),
            "def x : Prop := Prop",
            &[],
            &[],
            &[HumanByProofCore {
                source_index: 0,
                proof: Expr::sort(Level::zero()),
            }],
            &HumanCompileOptions::default(),
        )
        .expect_err("unused by proof core must not be silently ignored");

        assert_eq!(err.kind, HumanDiagnosticKind::UnsupportedTactic);
        assert!(err
            .message
            .contains("must match by theorem indices exactly"));
    }

    #[test]
    fn human_imported_source_interface_supplies_user_implicit_binder_metadata() {
        let source = "\
axiom A : Type
def id {B : Type} (x : B) : B := x";
        let (import, source_interface, _) = verified_human_import("Lib", source);
        let consumer = "\
import Lib
def use (a : A) : A := id a";

        compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Consumer"),
            consumer,
            std::slice::from_ref(&import),
            &HumanCompileOptions::default(),
        )
        .expect_err(
            "without Human source metadata, imported user implicit binders are not inferred",
        );

        let module = compile_human_source_to_core_with_source_interfaces(
            FileId(0),
            npa_cert::Name::from_dotted("Consumer"),
            consumer,
            &[import],
            &[source_interface],
            &HumanCompileOptions::default(),
        )
        .expect("Human source metadata should supply imported user implicit binder profiles");

        let Decl::Def { value, .. } = &module.declarations[0] else {
            panic!("expected use definition");
        };
        assert_eq!(
            value,
            &Expr::lam(
                "a",
                Expr::konst("A", vec![]),
                Expr::app(
                    Expr::app(Expr::konst("id", vec![]), Expr::konst("A", vec![])),
                    Expr::bvar(0)
                )
            )
        );
    }

    #[test]
    fn human_imported_source_interface_supplies_imported_notation_to_parser_and_resolver() {
        let source = "\
axiom A : Type
def choose (x y : A) : A := x
infixl:65 \" ++ \" => choose";
        let (import, source_interface, _) = verified_human_import("Lib", source);
        let consumer = "\
import Lib
axiom a : A
def use : A := a ++ a";

        compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Consumer"),
            consumer,
            std::slice::from_ref(&import),
            &HumanCompileOptions::default(),
        )
        .expect_err(
            "without Human source metadata, imported notation is unavailable at parse time",
        );

        let module = compile_human_source_to_core_with_source_interfaces(
            FileId(0),
            npa_cert::Name::from_dotted("Consumer"),
            consumer,
            &[import],
            &[source_interface],
            &HumanCompileOptions::default(),
        )
        .expect("Human source metadata should supply imported notation");

        let Decl::Def { value, .. } = &module.declarations[1] else {
            panic!("expected use definition");
        };
        assert_eq!(
            value,
            &Expr::app(
                Expr::app(Expr::konst("choose", vec![]), Expr::konst("a", vec![])),
                Expr::konst("a", vec![])
            )
        );
    }

    #[test]
    fn grouped_binder_annotation_is_lowered_before_group_locals_enter_scope() {
        let module = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "def ok (A x : Type) : Type := x",
            &[],
            &HumanCompileOptions::default(),
        )
        .expect("grouped binder annotation should be shared from the outer context");

        assert_eq!(
            module.declarations,
            vec![Decl::Def {
                name: "ok".to_owned(),
                universe_params: Vec::new(),
                ty: Expr::pi(
                    "A",
                    Expr::sort(type0()),
                    Expr::pi("x", Expr::sort(type0()), Expr::sort(type0()))
                ),
                value: Expr::lam(
                    "A",
                    Expr::sort(type0()),
                    Expr::lam("x", Expr::sort(type0()), Expr::bvar(0))
                ),
                reducibility: Reducibility::Reducible,
            }]
        );
    }

    #[test]
    fn grouped_binder_annotation_does_not_let_later_group_members_depend_on_earlier_ones() {
        let err = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "def bad (A x : Type) : A := x",
            &[],
            &HumanCompileOptions::default(),
        )
        .expect_err("x should have type Type, not the earlier grouped binder A");

        assert_eq!(err.kind, HumanDiagnosticKind::TypeMismatch);
    }

    #[test]
    fn human_unresolved_implicit_is_rejected_before_certificate_output() {
        let imports = [nat_import(), eq_import()];
        let err = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Nat.Basic
import Std.Logic.Eq
def bad : Type := Eq.refl",
            &imports,
            &HumanCompileOptions::default(),
        )
        .expect_err("unresolved implicit should reject the declaration");

        assert_eq!(err.kind, HumanDiagnosticKind::UnsolvedImplicit);
    }

    #[test]
    fn human_anonymous_hole_returns_goal_diagnostic_payload() {
        let imports = [nat_import()];
        let err = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Nat.Basic
def bad (n : Nat) : Nat := _",
            &imports,
            &HumanCompileOptions::default(),
        )
        .expect_err("unresolved Human hole should stop before Machine elaboration");

        assert_eq!(err.kind, HumanDiagnosticKind::UnsolvedHole);
        let payload = err.payload.expect("hole diagnostic should carry a payload");
        assert_eq!(payload.phase, Some(HumanDiagnosticPhase::Elaborator));
        assert_eq!(
            payload.unsolved_meta.as_ref().map(|meta| meta.kind),
            Some(HumanUnsolvedMetaKind::Hole)
        );
        assert_eq!(payload.hole_goals.len(), 1);
        let goal = &payload.hole_goals[0];
        assert_eq!(goal.hole, None);
        assert_eq!(goal.target.as_deref(), Some("Nat"));
        assert_eq!(goal.context.len(), 1);
        assert_eq!(goal.context[0].name, "n");
        assert_eq!(goal.context[0].ty, "Nat");
    }

    #[test]
    fn human_unresolved_hole_rejects_certificate_path_before_certificate_output() {
        let err = compile_human_source_to_certificate(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "def bad : Type := _",
            &[],
            &HumanCompileOptions::default(),
        )
        .expect_err("unresolved Human hole should not reach certificate construction");

        assert_eq!(err.kind, HumanDiagnosticKind::UnsolvedHole);
    }

    #[test]
    fn human_unresolved_universe_meta_rejects_certificate_path_before_certificate_output() {
        let err = compile_human_source_to_certificate(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
axiom F.{u} : Sort u
def bad : Type := F",
            &[],
            &HumanCompileOptions::default(),
        )
        .expect_err("unresolved Human universe meta should not reach certificate construction");

        assert_eq!(err.kind, HumanDiagnosticKind::UnsolvedUniverseMeta);
        let payload = err
            .payload
            .expect("unresolved universe meta should carry payload");
        assert_eq!(
            payload.unsolved_meta.as_ref().map(|meta| meta.kind),
            Some(HumanUnsolvedMetaKind::Universe)
        );
    }

    #[test]
    fn human_named_hole_reuse_requires_same_context_snapshot() {
        let imports = [nat_import()];
        let err = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Nat.Basic
def bad_named_hole : Nat := let x : Nat := ?m in ?m",
            &imports,
            &HumanCompileOptions::default(),
        )
        .expect_err("same named hole under a different context should be rejected");

        assert_eq!(err.kind, HumanDiagnosticKind::NamedHoleContextMismatch);
        let payload = err
            .payload
            .expect("context mismatch should carry both hole contexts");
        assert_eq!(payload.phase, Some(HumanDiagnosticPhase::Elaborator));
        assert_eq!(payload.hole_goals.len(), 2);
        assert_eq!(payload.hole_goals[0].hole.as_deref(), Some("?m"));
        assert_eq!(payload.hole_goals[0].context.len(), 0);
        assert_eq!(payload.hole_goals[1].hole.as_deref(), Some("?m"));
        assert_eq!(payload.hole_goals[1].context.len(), 1);
        assert_eq!(payload.hole_goals[1].context[0].name, "x");
    }

    #[test]
    fn machine_path_still_rejects_holes_before_ast_elaboration() {
        let err = crate::parse_machine_module(FileId(0), "def Test.bad : Prop := _")
            .expect_err("Machine Surface Complete path should reject holes");

        assert_eq!(err.kind, MachineDiagnosticKind::HoleNotAllowed);
    }

    #[test]
    fn human_meta_store_solves_simple_term_and_universe_constraints() {
        let span = Span::empty(FileId(0));
        let mut store = HumanMetaStore::default();
        let context = HumanLoweringLocalContext::default();
        let term_meta = store
            .fresh_user_hole(None, &context, None, span)
            .expect("hole meta should allocate");
        let universe_meta = store.fresh_universe_meta(span);

        store.add_constraint(HumanConstraint::TermEq {
            lhs: HumanMetaExpr::Meta(term_meta),
            rhs: HumanMetaExpr::Core(nat()),
            span,
        });
        store.add_constraint(HumanConstraint::TypeEq {
            lhs: HumanMetaExpr::App(
                Box::new(HumanMetaExpr::Core(Expr::konst("F", vec![]))),
                Box::new(HumanMetaExpr::Meta(term_meta)),
            ),
            rhs: HumanMetaExpr::App(
                Box::new(HumanMetaExpr::Core(Expr::konst("F", vec![]))),
                Box::new(HumanMetaExpr::Core(nat())),
            ),
            span,
        });
        store.add_constraint(HumanConstraint::LevelEq {
            lhs: HumanMetaLevel::Succ(Box::new(HumanMetaLevel::Meta(universe_meta))),
            rhs: HumanMetaLevel::Succ(Box::new(HumanMetaLevel::Core(Level::zero()))),
            span,
        });
        store.add_constraint(HumanConstraint::LevelEq {
            lhs: HumanMetaLevel::Max(
                Box::new(HumanMetaLevel::Core(Level::zero())),
                Box::new(HumanMetaLevel::Core(Level::zero())),
            ),
            rhs: HumanMetaLevel::Max(
                Box::new(HumanMetaLevel::Core(Level::zero())),
                Box::new(HumanMetaLevel::Core(Level::zero())),
            ),
            span,
        });
        store.add_constraint(HumanConstraint::LevelLe {
            lhs: HumanMetaLevel::Core(Level::zero()),
            rhs: HumanMetaLevel::Core(type0()),
            span,
        });

        store
            .solve_constraints()
            .expect("simple constraints should solve");
        assert_eq!(
            store.term_metas[term_meta.0 as usize].assignment,
            Some(HumanMetaExpr::Core(nat()))
        );
        assert_eq!(
            store.universe_metas[universe_meta.0 as usize].assignment,
            Some(HumanMetaLevel::Core(Level::zero()))
        );
    }

    #[test]
    fn human_meta_store_rejects_occurs_check_cycles() {
        let span = Span::empty(FileId(0));
        let mut store = HumanMetaStore::default();
        let context = HumanLoweringLocalContext::default();
        let term_meta = store
            .fresh_user_hole(None, &context, None, span)
            .expect("hole meta should allocate");
        store.add_constraint(HumanConstraint::TermEq {
            lhs: HumanMetaExpr::Meta(term_meta),
            rhs: HumanMetaExpr::App(
                Box::new(HumanMetaExpr::Meta(term_meta)),
                Box::new(HumanMetaExpr::Core(nat())),
            ),
            span,
        });

        let err = store
            .solve_constraints()
            .expect_err("cyclic assignment should fail occurs check");
        assert_eq!(err.kind, HumanDiagnosticKind::OccursCheckFailed);
    }
}
