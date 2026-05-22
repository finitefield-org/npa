use std::collections::BTreeMap;

use crate::{
    HumanApiCompileOptions, HumanApplyTacticError, HumanApplyTacticOk, HumanApplyTacticRequest,
    HumanCompileCertificateOk, HumanCompileCertificateRequest, HumanCompileCoreOk,
    HumanCompileCoreRequest, HumanCompileError, HumanCurrentModuleSource, HumanDocumentSnapshot,
    HumanDocumentUpdateError, HumanDocumentUpdateOk, HumanDocumentUpdateRequest,
    HumanExactTacticOk, HumanExactTacticRequest, HumanInductionTacticError, HumanInductionTacticOk,
    HumanInductionTacticRequest, HumanIntroTacticError, HumanIntroTacticOk,
    HumanIntroTacticRequest, HumanProofSession, HumanProofSessionStatus, HumanProofSessionStore,
    HumanRewriteTacticError, HumanRewriteTacticOk, HumanRewriteTacticRequest,
    HumanSessionCreateError, HumanSessionCreateOk, HumanSessionCreateRequest,
    HumanSimpLiteTacticError, HumanSimpLiteTacticOk, HumanSimpLiteTacticRequest,
    HumanStartProofError, HumanStartProofOk, HumanStartProofRequest, HumanStateRequestError,
    HumanStateRequestHeader, HumanTacticScriptError, HumanTacticScriptRunOk,
    HumanTacticScriptRunRequest, HumanTacticTermCheckOk, HumanTacticTermCheckRequest,
    HumanTacticTermError,
};
use npa_kernel::{subst::instantiate, Ctx, Decl, Expr, Level};

/// Compile Human source through the Human tactic API adapter.
///
/// Unlike the plain `npa_frontend::compile_human_source_to_core*` helpers, this
/// wrapper executes Human `by` proof blocks through the `npa-api` tactic bridge
/// and substitutes the extracted core proof terms before returning the core
/// module. The request keeps the current module/source/imports/options explicit
/// and does not create a Machine API session.
pub fn compile_human_source_to_core(
    request: HumanCompileCoreRequest<'_, '_>,
) -> Result<HumanCompileCoreOk, HumanCompileError> {
    compile_human_source_to_core_with_tactic_proofs(
        request.current_module,
        request.current_source,
        request.verified_modules,
        request.imported_source_interfaces,
        request.options,
    )
    .map(|output| HumanCompileCoreOk {
        core_module: output.core_module,
        source_interface: output.source_interface,
    })
}

/// Compile Human source to a certificate through the Human API adapter.
///
/// Plain frontend certificate compilation remains responsible for already-core
/// Human terms only. This API wrapper is the layer that runs Human `by` proof
/// blocks, verifies the resulting core module, and hashes the returned source
/// interface for downstream Human imports. It does not widen `/machine/*`
/// request grammar or implicitly allocate a Machine session.
pub fn compile_human_source_to_certificate(
    request: HumanCompileCertificateRequest<'_, '_>,
) -> Result<HumanCompileCertificateOk, HumanCompileError> {
    let options = npa_frontend::HumanCompileOptions::from(&request.options);
    let verified_imports: Vec<_> = request
        .verified_modules
        .iter()
        .map(npa_frontend::VerifiedImport::from)
        .collect();
    let by_targets = npa_frontend::collect_human_by_proof_targets_with_source_interfaces(
        request.current_source.file_id,
        request.current_module.clone(),
        request.current_source.source,
        &verified_imports,
        request.imported_source_interfaces,
        &options,
    )?;

    if !by_targets.targets.is_empty() {
        let core = compile_human_source_to_core_with_tactic_proofs(
            request.current_module,
            request.current_source,
            request.verified_modules,
            request.imported_source_interfaces,
            request.options,
        )?;
        let certificate_imports = npa_frontend::certificate_imports_for_human_core_module(
            &core.core_module,
            &core.active_imports,
            request.verified_modules,
            request.current_source.file_id,
        )?;
        let certificate = human_build_and_verify_certificate(
            core.core_module,
            &certificate_imports,
            request.current_source,
        )?;
        let source_interface =
            human_source_interface_with_certificate_hashes(core.source_interface, &certificate);
        return Ok(HumanCompileCertificateOk {
            certificate,
            source_interface,
        });
    }

    let output = npa_frontend::compile_human_source_to_certificate_output_with_source_interfaces(
        request.current_source.file_id,
        request.current_module,
        request.current_source.source,
        request.verified_modules,
        request.imported_source_interfaces,
        &options,
    )?;
    Ok(HumanCompileCertificateOk {
        certificate: output.certificate,
        source_interface: output.source_interface,
    })
}

/// Create a Human IDE proof session from explicit source and imports.
///
/// This is the library equivalent of Phase 5 Human `POST /sessions`. It stores
/// the caller-provided source text, module, verified imports, Human source
/// interfaces, and options in an in-memory `HumanProofSessionStore`. It does
/// not read from the filesystem, perform network lookup, or create a
/// `MachineProofSession`.
pub fn create_human_session(
    store: &mut HumanProofSessionStore,
    request: HumanSessionCreateRequest<'_, '_>,
) -> Result<HumanSessionCreateOk, HumanSessionCreateError> {
    let (session_id, document_id) = store.allocate_session_ids()?;
    let document = HumanDocumentSnapshot {
        document_id: document_id.clone(),
        document_version: crate::HumanDocumentVersion::initial(),
        current_module: request.current_module,
        file_id: request.current_source.file_id,
        source: request.current_source.source.to_owned(),
        verified_modules: request.verified_modules.to_vec(),
        imported_source_interfaces: request.imported_source_interfaces.to_vec(),
        options: request.options,
    };
    let collected = collect_human_session_document(&document);
    let messages = collected.messages.clone();
    let session = HumanProofSession {
        session_id: session_id.clone(),
        status: HumanProofSessionStatus::Open,
        document,
        source_interface: collected.source_interface,
        active_imported_source_interfaces: collected.active_imports,
        messages: collected.messages,
    };
    store.insert_session(session);

    Ok(HumanSessionCreateOk {
        session_id,
        document_id,
        document_version: crate::HumanDocumentVersion::initial(),
        status: HumanProofSessionStatus::Open,
        messages,
    })
}

/// Replace the current Human document snapshot for an open Human session.
///
/// This is the library equivalent of Phase 5 Human `POST /documents/update`.
/// The document id remains stable and the document version increases
/// monotonically. Imports and source interfaces are always supplied explicitly
/// by the request; this function performs no filesystem or network lookup.
pub fn update_human_document(
    store: &mut HumanProofSessionStore,
    request: HumanDocumentUpdateRequest<'_, '_>,
) -> Result<HumanDocumentUpdateOk, HumanDocumentUpdateError> {
    let (document_id, current_version) = {
        let session = store.session(&request.session_id).ok_or_else(|| {
            HumanDocumentUpdateError::UnknownSession {
                session_id: request.session_id.clone(),
            }
        })?;
        (
            session.document.document_id.clone(),
            session.document.document_version,
        )
    };
    let next_version = current_version.next().ok_or_else(|| {
        HumanDocumentUpdateError::DocumentVersionOverflow {
            session_id: request.session_id.clone(),
            document_id: document_id.clone(),
            current: current_version,
        }
    })?;
    let document = HumanDocumentSnapshot {
        document_id: document_id.clone(),
        document_version: next_version,
        current_module: request.current_module,
        file_id: request.current_source.file_id,
        source: request.current_source.source.to_owned(),
        verified_modules: request.verified_modules.to_vec(),
        imported_source_interfaces: request.imported_source_interfaces.to_vec(),
        options: request.options,
    };
    let collected = collect_human_session_document(&document);
    let messages = collected.messages.clone();
    let session = store
        .session_mut(&request.session_id)
        .expect("session was checked before document collection");
    session.status = HumanProofSessionStatus::Open;
    session.document = document;
    session.source_interface = collected.source_interface;
    session.active_imported_source_interfaces = collected.active_imports;
    session.messages = collected.messages;

    Ok(HumanDocumentUpdateOk {
        session_id: request.session_id,
        document_id,
        document_version: next_version,
        status: HumanProofSessionStatus::Open,
        messages,
    })
}

/// Validate the document identity/version portion common to future Human state requests.
///
/// P5H-01 does not materialize proof states yet; it only fixes the stale
/// document-version guard that `/state/*` APIs must apply before returning any
/// session state.
pub fn validate_human_state_request_document(
    store: &HumanProofSessionStore,
    request: HumanStateRequestHeader,
) -> Result<(), HumanStateRequestError> {
    let session = store.session(&request.session_id).ok_or_else(|| {
        HumanStateRequestError::UnknownSession {
            session_id: request.session_id.clone(),
        }
    })?;
    let current_document_id = &session.document.document_id;
    if current_document_id != &request.document_id {
        return Err(HumanStateRequestError::DocumentMismatch {
            session_id: request.session_id,
            requested: request.document_id,
            current: current_document_id.clone(),
        });
    }
    let current_version = session.document.document_version;
    if request.document_version < current_version {
        return Err(HumanStateRequestError::StaleDocumentVersion {
            session_id: request.session_id,
            document_id: request.document_id,
            requested: request.document_version,
            current: current_version,
        });
    }
    if request.document_version > current_version {
        return Err(HumanStateRequestError::FutureDocumentVersion {
            session_id: request.session_id,
            document_id: request.document_id,
            requested: request.document_version,
            current: current_version,
        });
    }
    Ok(())
}

pub fn start_human_proof(
    request: HumanStartProofRequest<'_, '_>,
) -> Result<HumanStartProofOk, HumanStartProofError> {
    let frontend_options = npa_frontend::HumanCompileOptions::from(&request.options);
    let frontend_imports: Vec<_> = request
        .verified_modules
        .iter()
        .map(npa_frontend::VerifiedImport::from)
        .collect();
    let prepared = npa_frontend::prepare_human_proof_start_core_with_source_interfaces(
        request.current_source.file_id,
        request.current_module.clone(),
        request.theorem_name,
        request.current_source.source,
        &frontend_imports,
        request.imported_source_interfaces,
        &frontend_options,
    )?;
    start_human_proof_from_prepared(prepared, request.verified_modules, request.options)
}

#[derive(Clone, Debug)]
struct HumanSessionDocumentCollection {
    source_interface: Option<npa_frontend::HumanSourceInterface>,
    active_imports: Vec<npa_frontend::HumanImportedSourceInterface>,
    messages: Vec<npa_frontend::HumanDiagnostic>,
}

fn collect_human_session_document(
    document: &HumanDocumentSnapshot,
) -> HumanSessionDocumentCollection {
    let frontend_options = npa_frontend::HumanCompileOptions::from(&document.options);
    let verified_imports: Vec<_> = document
        .verified_modules
        .iter()
        .map(npa_frontend::VerifiedImport::from)
        .collect();
    match npa_frontend::collect_human_by_proof_targets_with_source_interfaces(
        document.file_id,
        document.current_module.clone(),
        &document.source,
        &verified_imports,
        &document.imported_source_interfaces,
        &frontend_options,
    ) {
        Ok(output) => HumanSessionDocumentCollection {
            source_interface: Some(output.source_interface),
            active_imports: output.active_imports,
            messages: Vec::new(),
        },
        Err(diagnostic) => HumanSessionDocumentCollection {
            source_interface: None,
            active_imports: Vec::new(),
            messages: vec![diagnostic],
        },
    }
}

#[derive(Clone, Debug)]
struct HumanCompileCoreWithTacticProofsOk {
    core_module: npa_cert::CoreModule,
    source_interface: npa_frontend::HumanSourceInterface,
    active_imports: Vec<npa_frontend::HumanImportedSourceInterface>,
}

fn compile_human_source_to_core_with_tactic_proofs(
    current_module: npa_cert::ModuleName,
    current_source: HumanCurrentModuleSource<'_>,
    verified_modules: &[npa_cert::VerifiedModule],
    imported_source_interfaces: &[npa_frontend::HumanImportedSourceInterface],
    options: HumanApiCompileOptions,
) -> Result<HumanCompileCoreWithTacticProofsOk, HumanCompileError> {
    let frontend_options = npa_frontend::HumanCompileOptions::from(&options);
    let verified_imports: Vec<_> = verified_modules
        .iter()
        .map(npa_frontend::VerifiedImport::from)
        .collect();
    let by_targets = npa_frontend::collect_human_by_proof_targets_with_source_interfaces(
        current_source.file_id,
        current_module.clone(),
        current_source.source,
        &verified_imports,
        imported_source_interfaces,
        &frontend_options,
    )?;

    if by_targets.targets.is_empty() {
        let output = npa_frontend::compile_human_source_to_core_output_with_source_interfaces(
            current_source.file_id,
            current_module,
            current_source.source,
            &verified_imports,
            imported_source_interfaces,
            &frontend_options,
        )?;
        return Ok(HumanCompileCoreWithTacticProofsOk {
            core_module: output.core_module,
            source_interface: output.source_interface,
            active_imports: Vec::new(),
        });
    }

    let mut by_proofs = Vec::with_capacity(by_targets.targets.len());
    for target in &by_targets.targets {
        let prepared =
            npa_frontend::prepare_human_proof_start_core_with_source_interfaces_and_by_proofs(
                npa_frontend::HumanProofStartCoreWithProofsRequest {
                    file_id: current_source.file_id,
                    module_name: current_module.clone(),
                    theorem_name: target.theorem_name.clone(),
                    source: current_source.source,
                    verified_imports: &verified_imports,
                    imported_source_interfaces,
                    prior_by_proofs: &by_proofs,
                    options: &frontend_options,
                },
            )?;
        let started = start_human_proof_from_prepared(prepared, verified_modules, options.clone())
            .map_err(human_compile_start_error)?;
        let run = run_human_tactic_script(HumanTacticScriptRunRequest {
            state: &started.state,
            script: &target.script,
            current_source_interface: &started.source_interface,
            imported_source_interfaces,
            options: options.clone(),
            budget: npa_tactic::TacticBudget::default(),
        })
        .map_err(|error| human_compile_script_error(error, target.script.span))?;
        by_proofs.push(npa_frontend::HumanByProofCore {
            source_index: target.source_index,
            proof: run.proof,
        });
    }

    let output =
        npa_frontend::compile_human_source_to_core_output_with_source_interfaces_and_by_proofs(
            current_source.file_id,
            current_module,
            current_source.source,
            &verified_imports,
            imported_source_interfaces,
            &by_proofs,
            &frontend_options,
        )?;

    Ok(HumanCompileCoreWithTacticProofsOk {
        core_module: output.core_module,
        source_interface: output.source_interface,
        active_imports: by_targets.active_imports,
    })
}

fn start_human_proof_from_prepared(
    prepared: npa_frontend::HumanProofStartCoreOutput,
    verified_modules: &[npa_cert::VerifiedModule],
    options: HumanApiCompileOptions,
) -> Result<HumanStartProofOk, HumanStartProofError> {
    let machine_tactic_imports =
        active_human_verified_import_refs(verified_modules, &prepared.active_imports)?;
    let mut checked_current_decls = Vec::with_capacity(prepared.proof.prior_declarations.len());
    for (source_index, decl) in prepared
        .proof
        .prior_declarations
        .iter()
        .cloned()
        .enumerate()
    {
        let checked = npa_tactic::check_current_decl_for_machine_tactic_from_verified_imports(
            &machine_tactic_imports,
            &checked_current_decls,
            source_index as u64,
            decl,
        )?;
        checked_current_decls.push(checked);
    }

    let state = npa_tactic::start_machine_proof(
        npa_tactic::MachineProofSpec {
            module: prepared.proof.module,
            theorem_name: prepared.proof.theorem_name,
            source_index: prepared.proof.source_index,
            universe_params: prepared.proof.universe_params,
            theorem_type: prepared.proof.theorem_type,
        },
        machine_tactic_imports,
        checked_current_decls,
        options.tactic_options.clone(),
    )?;
    npa_tactic::validate_machine_proof_state(&state)?;

    Ok(HumanStartProofOk {
        state,
        source_interface: prepared.source_interface,
    })
}

fn human_build_and_verify_certificate(
    core_module: npa_cert::CoreModule,
    certificate_imports: &[npa_cert::VerifiedModule],
    source: HumanCurrentModuleSource<'_>,
) -> Result<npa_cert::ModuleCert, HumanCompileError> {
    let certificate =
        npa_cert::build_module_cert(core_module, certificate_imports).map_err(|err| {
            npa_frontend::HumanDiagnostic::error(
                npa_frontend::HumanDiagnosticKind::KernelRejected,
                human_source_span(source),
                format!("certificate certificate handoff rejected Human by proof source: {err:?}"),
            )
            .with_phase(npa_frontend::HumanDiagnosticPhase::CertificateHandoff)
        })?;
    let bytes = npa_cert::encode_module_cert(&certificate).map_err(|err| {
        npa_frontend::HumanDiagnostic::error(
            npa_frontend::HumanDiagnosticKind::KernelRejected,
            human_source_span(source),
            format!("certificate certificate encoding rejected Human by proof source: {err:?}"),
        )
        .with_phase(npa_frontend::HumanDiagnosticPhase::CertificateHandoff)
    })?;
    let mut session = npa_cert::VerifierSession::new();
    for import in certificate_imports {
        session.register_verified_module(import.clone());
    }
    npa_cert::verify_module_cert(&bytes, &mut session, &npa_cert::AxiomPolicy::normal()).map_err(
        |err| {
            npa_frontend::HumanDiagnostic::error(
                npa_frontend::HumanDiagnosticKind::KernelRejected,
                human_source_span(source),
                format!(
                    "certificate certificate verification rejected Human by proof source: {err:?}"
                ),
            )
            .with_phase(npa_frontend::HumanDiagnosticPhase::CertificateHandoff)
        },
    )?;
    Ok(certificate)
}

fn human_compile_start_error(error: HumanStartProofError) -> HumanCompileError {
    match error {
        HumanStartProofError::Human(error) => error,
        HumanStartProofError::Machine(diagnostic) => human_compile_machine_tactic_diagnostic(
            diagnostic,
            npa_frontend::Span::empty(npa_frontend::FileId(0)),
        ),
    }
}

fn human_compile_script_error(
    error: HumanTacticScriptError,
    span: npa_frontend::Span,
) -> HumanCompileError {
    match error {
        HumanTacticScriptError::Human(error) => error,
        HumanTacticScriptError::Machine(diagnostic) => {
            human_compile_machine_tactic_diagnostic(diagnostic, span)
        }
    }
}

fn human_compile_machine_tactic_diagnostic(
    diagnostic: npa_tactic::MachineTacticDiagnostic,
    span: npa_frontend::Span,
) -> HumanCompileError {
    human_tactic_machine_diagnostic(
        &diagnostic,
        span,
        None,
        None,
        format!(
            "Human by proof tactic execution failed before certificate handoff: {:?}: {}",
            &diagnostic.kind, diagnostic.message
        ),
    )
    .into()
}

fn human_source_span(source: HumanCurrentModuleSource<'_>) -> npa_frontend::Span {
    npa_frontend::Span::new(source.file_id, 0, source.source.len() as u32)
}

fn human_source_interface_with_certificate_hashes(
    mut source_interface: npa_frontend::HumanSourceInterface,
    cert: &npa_cert::ModuleCert,
) -> npa_frontend::HumanSourceInterface {
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
            .or_else(|| export_hashes.get(&human_prefixed_current_name(&module_name, &name)))
        {
            decl.decl_interface_hash = Some(*hash);
        }
    }

    for generated in &mut source_interface.generated_declarations {
        let name = npa_cert::Name(generated.name.parts.clone());
        if let Some(hash) = export_hashes
            .get(&name)
            .or_else(|| export_hashes.get(&human_prefixed_current_name(&module_name, &name)))
        {
            generated.decl_interface_hash = Some(*hash);
        }
    }

    source_interface
}

fn human_prefixed_current_name(
    module_name: &npa_cert::ModuleName,
    name: &npa_cert::Name,
) -> npa_cert::Name {
    if name.0.len() > module_name.0.len() && name.0.starts_with(&module_name.0) {
        return name.clone();
    }

    let mut parts = module_name.0.clone();
    parts.extend(name.0.iter().cloned());
    npa_cert::Name(parts)
}

fn human_tactic_goal_payload(
    goal: &npa_tactic::MachineGoal,
    span: npa_frontend::Span,
) -> npa_frontend::HumanDiagnosticPayload {
    npa_frontend::HumanDiagnosticPayload {
        hole_goals: vec![human_tactic_goal_display(goal, span)],
        ..npa_frontend::HumanDiagnosticPayload::default()
    }
}

fn human_tactic_goal_display(
    goal: &npa_tactic::MachineGoal,
    span: npa_frontend::Span,
) -> npa_frontend::HumanHoleGoal {
    let mut local_names = Vec::with_capacity(goal.context.len());
    let mut context = Vec::with_capacity(goal.context.len());
    for local in &goal.context {
        let ty = human_render_core_expr(&local.ty, &local_names);
        let value = local
            .value
            .as_ref()
            .map(|value| human_render_core_expr(value, &local_names));
        context.push(npa_frontend::HumanHoleGoalLocal {
            name: local.name.clone(),
            ty,
            value,
        });
        local_names.push(local.name.clone());
    }

    npa_frontend::HumanHoleGoal {
        hole: Some(format!("g{}", goal.id.0)),
        context,
        target: Some(human_render_core_expr(&goal.target, &local_names)),
        source_span: span,
    }
}

fn human_render_core_expr(expr: &Expr, local_names: &[String]) -> String {
    let mut names = local_names.to_vec();
    human_render_core_expr_with_names(expr, &mut names, 0)
}

fn human_render_core_expr_with_names(
    expr: &Expr,
    local_names: &mut Vec<String>,
    parent_prec: u8,
) -> String {
    const PREC_BINDER: u8 = 10;
    const PREC_APP: u8 = 80;
    const PREC_ATOM: u8 = 100;

    let (rendered, prec) = match expr {
        Expr::Sort(level) => (human_render_sort(level), PREC_ATOM),
        Expr::BVar(index) => {
            let index = *index as usize;
            let name = local_names
                .len()
                .checked_sub(index + 1)
                .and_then(|local_index| local_names.get(local_index))
                .cloned()
                .unwrap_or_else(|| format!("#{index}"));
            (name, PREC_ATOM)
        }
        Expr::Const { name, levels } => {
            let rendered = if levels.is_empty() {
                name.clone()
            } else {
                format!(
                    "{}.{{{}}}",
                    name,
                    levels
                        .iter()
                        .map(human_render_level)
                        .collect::<Vec<_>>()
                        .join(",")
                )
            };
            (rendered, PREC_ATOM)
        }
        Expr::App(_, _) => {
            let mut parts = Vec::new();
            human_collect_app_parts(expr, local_names, &mut parts);
            (parts.join(" "), PREC_APP)
        }
        Expr::Lam { binder, ty, body } => {
            let binder = human_fresh_binder_name(binder, local_names);
            let ty = human_render_core_expr_with_names(ty, local_names, 0);
            local_names.push(binder.clone());
            let body = human_render_core_expr_with_names(body, local_names, 0);
            local_names.pop();
            (format!("fun ({binder} : {ty}) => {body}"), PREC_BINDER)
        }
        Expr::Pi { binder, ty, body } => {
            let binder = human_fresh_binder_name(binder, local_names);
            let ty = human_render_core_expr_with_names(ty, local_names, 0);
            local_names.push(binder.clone());
            let body = human_render_core_expr_with_names(body, local_names, 0);
            local_names.pop();
            (format!("forall ({binder} : {ty}), {body}"), PREC_BINDER)
        }
        Expr::Let {
            binder,
            ty,
            value,
            body,
        } => {
            let binder = human_fresh_binder_name(binder, local_names);
            let ty = human_render_core_expr_with_names(ty, local_names, 0);
            let value = human_render_core_expr_with_names(value, local_names, 0);
            local_names.push(binder.clone());
            let body = human_render_core_expr_with_names(body, local_names, 0);
            local_names.pop();
            (
                format!("let {binder} : {ty} := {value} in {body}"),
                PREC_BINDER,
            )
        }
    };

    if prec < parent_prec {
        format!("({rendered})")
    } else {
        rendered
    }
}

fn human_collect_app_parts(expr: &Expr, local_names: &mut Vec<String>, parts: &mut Vec<String>) {
    match expr {
        Expr::App(func, arg) => {
            human_collect_app_parts(func, local_names, parts);
            parts.push(human_render_core_expr_with_names(arg, local_names, 100));
        }
        _ => parts.push(human_render_core_expr_with_names(expr, local_names, 80)),
    }
}

fn human_fresh_binder_name(base: &str, local_names: &[String]) -> String {
    let candidate = if base.is_empty() || base == "_" {
        "x".to_owned()
    } else {
        base.to_owned()
    };
    if !local_names.iter().any(|name| name == &candidate) {
        return candidate;
    }
    for index in 1.. {
        let fresh = format!("{candidate}{index}");
        if !local_names.iter().any(|name| name == &fresh) {
            return fresh;
        }
    }
    unreachable!("unbounded fresh-name search should return");
}

fn human_render_sort(level: &Level) -> String {
    match level {
        Level::Zero => "Prop".to_owned(),
        Level::Succ(inner) if matches!(inner.as_ref(), Level::Zero) => "Type".to_owned(),
        Level::Succ(inner) => format!("Type {}", human_render_level(inner)),
        _ => format!("Sort {}", human_render_level(level)),
    }
}

fn human_render_level(level: &Level) -> String {
    match level {
        Level::Zero => "0".to_owned(),
        Level::Succ(inner) => format!("succ {}", human_render_level(inner)),
        Level::Max(lhs, rhs) => {
            format!(
                "max {} {}",
                human_render_level(lhs),
                human_render_level(rhs)
            )
        }
        Level::IMax(lhs, rhs) => {
            format!(
                "imax {} {}",
                human_render_level(lhs),
                human_render_level(rhs)
            )
        }
        Level::Param(name) => name.clone(),
    }
}

fn human_tactic_machine_kind(
    diagnostic: &npa_tactic::MachineTacticDiagnostic,
) -> npa_frontend::HumanDiagnosticKind {
    match &diagnostic.kind {
        npa_tactic::MachineTacticDiagnosticKind::AmbiguousApplyArgument
        | npa_tactic::MachineTacticDiagnosticKind::AmbiguousRewriteRule
        | npa_tactic::MachineTacticDiagnosticKind::ExpectedEqTarget
        | npa_tactic::MachineTacticDiagnosticKind::ExpectedFunctionType
        | npa_tactic::MachineTacticDiagnosticKind::ExpectedPiTarget
        | npa_tactic::MachineTacticDiagnosticKind::InvalidMetaDependency
        | npa_tactic::MachineTacticDiagnosticKind::MissingExplicitArgument
        | npa_tactic::MachineTacticDiagnosticKind::ProofExprTypeMismatch
        | npa_tactic::MachineTacticDiagnosticKind::SubgoalDataArgument
        | npa_tactic::MachineTacticDiagnosticKind::TooFewApplyArguments
        | npa_tactic::MachineTacticDiagnosticKind::TooManyApplyArguments
        | npa_tactic::MachineTacticDiagnosticKind::TypeMismatch => {
            npa_frontend::HumanDiagnosticKind::TypeMismatch
        }
        npa_tactic::MachineTacticDiagnosticKind::AmbiguousLocalName
        | npa_tactic::MachineTacticDiagnosticKind::AmbiguousTacticHead => {
            npa_frontend::HumanDiagnosticKind::AmbiguousName
        }
        npa_tactic::MachineTacticDiagnosticKind::UnknownLocalName
        | npa_tactic::MachineTacticDiagnosticKind::UnknownName
        | npa_tactic::MachineTacticDiagnosticKind::UnknownTacticHead => {
            npa_frontend::HumanDiagnosticKind::UnknownIdentifier
        }
        npa_tactic::MachineTacticDiagnosticKind::InvalidInductionTarget
        | npa_tactic::MachineTacticDiagnosticKind::InvalidLocalHead
        | npa_tactic::MachineTacticDiagnosticKind::InvalidMachineTactic
        | npa_tactic::MachineTacticDiagnosticKind::TacticPrimitiveUnavailable
        | npa_tactic::MachineTacticDiagnosticKind::UnsupportedMachineTactic => {
            npa_frontend::HumanDiagnosticKind::UnsupportedTactic
        }
        npa_tactic::MachineTacticDiagnosticKind::UnresolvedGoal => {
            npa_frontend::HumanDiagnosticKind::UnresolvedGoal
        }
        npa_tactic::MachineTacticDiagnosticKind::KernelRejected => {
            npa_frontend::HumanDiagnosticKind::KernelRejected
        }
        _ => npa_frontend::HumanDiagnosticKind::MachineElaborationError,
    }
}

fn human_tactic_machine_diagnostic(
    diagnostic: &npa_tactic::MachineTacticDiagnostic,
    span: npa_frontend::Span,
    goal: Option<&npa_tactic::MachineGoal>,
    kind: Option<npa_frontend::HumanDiagnosticKind>,
    message: impl Into<String>,
) -> npa_frontend::HumanDiagnostic {
    let kind = kind.unwrap_or_else(|| human_tactic_machine_kind(diagnostic));
    let mut human = npa_frontend::HumanDiagnostic::error(kind, span, message)
        .with_phase(npa_frontend::HumanDiagnosticPhase::TacticExecution);
    if let Some(goal) = goal {
        human = human.with_payload(human_tactic_goal_payload(goal, span));
    }
    human
}

fn human_tactic_validation_diagnostic_with_goal(
    mut diagnostic: npa_frontend::HumanDiagnostic,
    goal: &npa_tactic::MachineGoal,
    span: npa_frontend::Span,
) -> npa_frontend::HumanDiagnostic {
    diagnostic = diagnostic.with_phase(npa_frontend::HumanDiagnosticPhase::TacticValidation);
    diagnostic.with_payload(human_tactic_goal_payload(goal, span))
}

pub fn check_human_tactic_term(
    request: HumanTacticTermCheckRequest<'_, '_>,
) -> Result<HumanTacticTermCheckOk, HumanTacticTermError> {
    let frontend_options = npa_frontend::HumanCompileOptions::from(&request.options);
    let goal = request.state.goal(request.goal_id)?;
    let direct_imports = request
        .state
        .env
        .imports
        .iter()
        .filter(|import| import.is_visible())
        .map(frontend_import_from_tactic_ref)
        .collect::<Vec<_>>();
    let available_imports = request
        .state
        .env
        .imports
        .iter()
        .map(|import| npa_frontend::VerifiedImport::from(import.verified_module()))
        .collect::<Vec<_>>();
    let checked_current_decls = request
        .state
        .env
        .checked_current_decls
        .iter()
        .map(|decl| npa_frontend::MachineCheckedCurrentDecl {
            name: decl.signature().name().clone(),
            source_index: decl.source_index(),
            decl_interface_hash: decl.signature().decl_interface_hash(),
            decl: decl.core_decl().clone(),
        })
        .collect::<Vec<_>>();
    let current_generated_decls =
        human_tactic_current_generated_decls(&request.state.env.checked_current_decls);
    let local_context = goal
        .context
        .iter()
        .map(|local| npa_frontend::MachineLocalDecl {
            name: local.name.clone(),
            ty: local.ty.clone(),
            value: local.value.clone(),
        })
        .collect::<Vec<_>>();
    let context = npa_frontend::HumanTacticTermElabContext::from_request(
        npa_frontend::HumanTacticTermElabContextRequest {
            direct_imports: &direct_imports,
            available_imports: &available_imports,
            current_module: request.state.root.module.clone(),
            checked_current_decls: &checked_current_decls,
            current_generated_decls: &current_generated_decls,
            local_context,
            universe_params: request.state.root.universe_params.clone(),
            current_source_interface: Some(request.current_source_interface),
            imported_source_interfaces: request.imported_source_interfaces,
        },
    )?;
    let output = npa_frontend::elaborate_human_tactic_term_check(
        &context,
        request.term,
        &goal.target,
        &frontend_options,
    )?;

    Ok(HumanTacticTermCheckOk {
        expr: output.expr,
        inferred_type: output.inferred_type,
    })
}

pub fn run_human_exact_tactic(
    request: HumanExactTacticRequest<'_, '_>,
) -> Result<HumanExactTacticOk, HumanTacticTermError> {
    let goal = request.state.goal(request.goal_id)?;
    let checked = check_human_tactic_term(HumanTacticTermCheckRequest {
        state: request.state,
        goal_id: request.goal_id,
        term: request.term,
        current_source_interface: request.current_source_interface,
        imported_source_interfaces: request.imported_source_interfaces,
        options: request.options,
    })
    .map_err(|error| human_exact_check_error(error, &goal, request.term.span()))?;
    let (state, delta) = npa_tactic::assign_goal(
        request.state,
        request.goal_id,
        npa_tactic::ProofExpr::Core(checked.expr.clone()),
        Vec::new(),
    )
    .map_err(|diagnostic| {
        human_tactic_machine_diagnostic(
            &diagnostic,
            request.term.span(),
            Some(&goal),
            Some(npa_frontend::HumanDiagnosticKind::TypeMismatch),
            format!(
                "`exact` could not assign the proof term: {}",
                diagnostic.message
            ),
        )
    })?;
    npa_tactic::validate_machine_proof_state(&state)?;

    Ok(HumanExactTacticOk {
        state,
        delta,
        expr: checked.expr,
        inferred_type: checked.inferred_type,
    })
}

pub fn run_human_intro_tactic(
    request: HumanIntroTacticRequest<'_, '_>,
) -> Result<HumanIntroTacticOk, HumanIntroTacticError> {
    let name = human_intro_name(request.name)?;
    let before_goal = request.state.goal(request.goal_id)?;
    let (state, delta) = npa_tactic::run_machine_tactic_with_budget(
        request.state,
        npa_tactic::MachineTactic::Intro {
            goal_id: request.goal_id,
            name,
        },
        request.budget,
    )
    .map_err(|diagnostic| human_intro_machine_error(diagnostic, &before_goal, request.name.span))?;
    npa_tactic::validate_machine_proof_state(&state)?;

    Ok(HumanIntroTacticOk { state, delta })
}

pub fn run_human_apply_tactic(
    request: HumanApplyTacticRequest<'_, '_>,
) -> Result<HumanApplyTacticOk, HumanApplyTacticError> {
    let goal = request.state.goal(request.goal_id)?;
    let resolved = human_apply_resolve(
        request.state,
        &goal,
        request.term,
        request.current_source_interface,
        request.imported_source_interfaces,
    )?;
    let (state, delta) = npa_tactic::run_machine_tactic_with_budget(
        request.state,
        npa_tactic::MachineTactic::Apply {
            goal_id: request.goal_id,
            head: resolved.head.clone(),
            universe_args: resolved.universe_args.clone(),
            args: resolved.args.clone(),
        },
        request.budget,
    )
    .map_err(|diagnostic| human_apply_machine_error(diagnostic, &goal, &resolved))?;
    npa_tactic::validate_machine_proof_state(&state)?;

    Ok(HumanApplyTacticOk { state, delta })
}

pub fn run_human_rewrite_tactic(
    request: HumanRewriteTacticRequest<'_, '_>,
) -> Result<HumanRewriteTacticOk, HumanRewriteTacticError> {
    if request.rules.is_empty() {
        return Err(human_rewrite_unsupported_diagnostic(
            request.span,
            "rw requires at least one rewrite rule",
        )
        .into());
    }

    let mut state = request.state.clone();
    let mut deltas = Vec::new();
    let mut current_goal_id = request.goal_id;

    for rule in request.rules {
        let resolved = human_rewrite_resolve_rule(
            &state,
            current_goal_id,
            rule,
            request.current_source_interface,
            request.imported_source_interfaces,
        )?;
        let mut rule_rewrote = false;
        let mut last_error = None;

        for site in [
            npa_tactic::RewriteSite::EqTargetLeft,
            npa_tactic::RewriteSite::EqTargetRight,
        ] {
            let before_goal = state.goal(current_goal_id)?;
            match npa_tactic::run_machine_tactic_with_budget(
                &state,
                npa_tactic::MachineTactic::Rewrite {
                    goal_id: current_goal_id,
                    rule: npa_tactic::RewriteRuleRef {
                        head: resolved.head.clone(),
                        universe_args: resolved.universe_args.clone(),
                        args: resolved.args.clone(),
                    },
                    direction: resolved.direction,
                    site,
                },
                request.budget,
            ) {
                Ok((next_state, delta)) => {
                    state = next_state;
                    current_goal_id = *delta.added_goals.last().ok_or_else(|| {
                        human_rewrite_unsupported_diagnostic(
                            resolved.span,
                            "Human rw expected Machine rewrite to create a rewritten target goal",
                        )
                    })?;
                    deltas.push(delta);
                    rule_rewrote = true;
                    npa_tactic::validate_machine_proof_state(&state)?;
                }
                Err(diagnostic) => {
                    last_error = Some(human_rewrite_machine_error(
                        diagnostic,
                        &before_goal,
                        &resolved,
                        site,
                    ));
                }
            }
        }

        if !rule_rewrote {
            return Err(last_error.unwrap_or_else(|| {
                human_rewrite_unsupported_diagnostic(
                    resolved.span,
                    format!(
                        "rewrite rule `{}` did not match the target",
                        resolved.head_label
                    ),
                )
                .into()
            }));
        }
    }

    Ok(HumanRewriteTacticOk { state, deltas })
}

pub fn run_human_simp_lite_tactic(
    request: HumanSimpLiteTacticRequest<'_>,
) -> Result<HumanSimpLiteTacticOk, HumanSimpLiteTacticError> {
    let before_goal = request.state.goal(request.goal_id)?;
    let (state, delta) = npa_tactic::run_machine_tactic_with_budget(
        request.state,
        npa_tactic::MachineTactic::SimpLite {
            goal_id: request.goal_id,
            rules: Vec::new(),
        },
        request.budget,
    )
    .map_err(|diagnostic| human_simp_lite_machine_error(diagnostic, &before_goal, request.span))?;

    if !delta.added_goals.is_empty() || state.open_goals.contains(&request.goal_id) {
        let residual_target = delta
            .added_goals
            .last()
            .and_then(|goal_id| state.goal(*goal_id).ok())
            .map(|goal| goal.target);
        return Err(human_simp_lite_not_closed_diagnostic(
            request.span,
            &before_goal,
            residual_target.as_ref(),
        )
        .into());
    }

    npa_tactic::validate_machine_proof_state(&state)?;
    Ok(HumanSimpLiteTacticOk { state, delta })
}

pub fn run_human_induction_tactic(
    request: HumanInductionTacticRequest<'_, '_>,
) -> Result<HumanInductionTacticOk, HumanInductionTacticError> {
    let local_name = human_induction_name(request.name)?;
    let before_goal = request.state.goal(request.goal_id)?;
    let (state, delta) = npa_tactic::run_machine_tactic_with_budget(
        request.state,
        npa_tactic::MachineTactic::InductionNat {
            goal_id: request.goal_id,
            local_name,
        },
        request.budget,
    )
    .map_err(|diagnostic| human_induction_machine_error(diagnostic, &before_goal, request.span))?;
    npa_tactic::validate_machine_proof_state(&state)?;
    Ok(HumanInductionTacticOk { state, delta })
}

pub fn run_human_tactic_script(
    request: HumanTacticScriptRunRequest<'_, '_>,
) -> Result<HumanTacticScriptRunOk, HumanTacticScriptError> {
    let mut state = request.state.clone();
    let mut deltas = Vec::with_capacity(request.script.tactics.len());

    for tactic in &request.script.tactics {
        let Some(goal_id) = state.open_goals.first().copied() else {
            return Err(human_script_no_goals_diagnostic(tactic.span()).into());
        };

        match tactic {
            npa_frontend::HumanTacticSyntax::Intro { name, .. } => {
                let ok = run_human_intro_tactic(HumanIntroTacticRequest {
                    state: &state,
                    goal_id,
                    name,
                    budget: request.budget,
                })
                .map_err(human_script_intro_error)?;
                state = ok.state;
                deltas.push(ok.delta);
            }
            npa_frontend::HumanTacticSyntax::Exact { term, .. } => {
                let ok = run_human_exact_tactic(HumanExactTacticRequest {
                    state: &state,
                    goal_id,
                    term,
                    current_source_interface: request.current_source_interface,
                    imported_source_interfaces: request.imported_source_interfaces,
                    options: request.options.clone(),
                })
                .map_err(human_script_term_error)?;
                state = ok.state;
                deltas.push(ok.delta);
            }
            npa_frontend::HumanTacticSyntax::Apply { term, .. } => {
                let ok = run_human_apply_tactic(HumanApplyTacticRequest {
                    state: &state,
                    goal_id,
                    term,
                    current_source_interface: request.current_source_interface,
                    imported_source_interfaces: request.imported_source_interfaces,
                    budget: request.budget,
                })
                .map_err(human_script_apply_error)?;
                state = ok.state;
                deltas.push(ok.delta);
            }
            npa_frontend::HumanTacticSyntax::Rewrite { rules, span } => {
                let ok = run_human_rewrite_tactic(HumanRewriteTacticRequest {
                    state: &state,
                    goal_id,
                    rules,
                    span: *span,
                    current_source_interface: request.current_source_interface,
                    imported_source_interfaces: request.imported_source_interfaces,
                    budget: request.budget,
                })
                .map_err(human_script_rewrite_error)?;
                state = ok.state;
                deltas.extend(ok.deltas);
            }
            npa_frontend::HumanTacticSyntax::SimpLite { span } => {
                let ok = run_human_simp_lite_tactic(HumanSimpLiteTacticRequest {
                    state: &state,
                    goal_id,
                    span: *span,
                    budget: request.budget,
                })
                .map_err(human_script_simp_lite_error)?;
                state = ok.state;
                deltas.push(ok.delta);
            }
            npa_frontend::HumanTacticSyntax::Induction { name, span } => {
                let ok = run_human_induction_tactic(HumanInductionTacticRequest {
                    state: &state,
                    goal_id,
                    name,
                    span: *span,
                    budget: request.budget,
                })
                .map_err(human_script_induction_error)?;
                state = ok.state;
                deltas.push(ok.delta);
            }
        }
    }

    if !state.open_goals.is_empty() {
        return Err(human_script_unresolved_goal_diagnostic(request.script.span, &state).into());
    }

    let proof = npa_tactic::extract_closed_machine_proof(&state)?;
    Ok(HumanTacticScriptRunOk {
        state,
        deltas,
        proof,
    })
}

pub fn human_api_default_compile_options() -> HumanApiCompileOptions {
    HumanApiCompileOptions::default()
}

fn human_intro_name(name: &npa_frontend::HumanName) -> Result<String, HumanIntroTacticError> {
    if name.parts.len() != 1 {
        return Err(human_intro_invalid_diagnostic(
            name.span,
            format!(
                "intro binder name must be a single identifier, got {}",
                name.as_dotted()
            ),
        )
        .into());
    }
    Ok(name.parts[0].clone())
}

fn human_exact_check_error(
    error: HumanTacticTermError,
    goal: &npa_tactic::MachineGoal,
    span: npa_frontend::Span,
) -> HumanTacticTermError {
    match error {
        HumanTacticTermError::Human(HumanCompileError { diagnostic }) => {
            human_tactic_validation_diagnostic_with_goal(diagnostic, goal, span).into()
        }
        HumanTacticTermError::Machine(diagnostic) => human_tactic_machine_diagnostic(
            &diagnostic,
            span,
            Some(goal),
            Some(npa_frontend::HumanDiagnosticKind::MachineElaborationError),
            format!("`exact` term validation failed: {}", diagnostic.message),
        )
        .with_phase(npa_frontend::HumanDiagnosticPhase::TacticValidation)
        .into(),
    }
}

fn human_intro_machine_error(
    diagnostic: npa_tactic::MachineTacticDiagnostic,
    goal: &npa_tactic::MachineGoal,
    span: npa_frontend::Span,
) -> HumanIntroTacticError {
    match &diagnostic.kind {
        npa_tactic::MachineTacticDiagnosticKind::TypeMismatch => human_tactic_machine_diagnostic(
            &diagnostic,
            span,
            Some(goal),
            Some(npa_frontend::HumanDiagnosticKind::ExpectedFunctionType),
            "`intro` can only be used when the target is a function type or forall.",
        )
        .into(),
        npa_tactic::MachineTacticDiagnosticKind::InvalidMachineTactic => {
            human_intro_invalid_diagnostic(span, diagnostic.message.to_string()).into()
        }
        _ => diagnostic.into(),
    }
}

fn human_intro_invalid_diagnostic(
    span: npa_frontend::Span,
    message: impl Into<String>,
) -> npa_frontend::HumanDiagnostic {
    npa_frontend::HumanDiagnostic::error(
        npa_frontend::HumanDiagnosticKind::UnsupportedTactic,
        span,
        message,
    )
    .with_phase(npa_frontend::HumanDiagnosticPhase::TacticValidation)
}

fn human_induction_name(
    name: &npa_frontend::HumanName,
) -> Result<String, HumanInductionTacticError> {
    if name.parts.len() != 1 {
        return Err(human_induction_unsupported_diagnostic(
            name.span,
            format!(
                "induction target name must be a single local identifier, got {}",
                name.as_dotted()
            ),
        )
        .into());
    }
    Ok(name.parts[0].clone())
}

#[derive(Clone, Debug)]
struct HumanApplyResolved {
    head: npa_tactic::TacticHead,
    universe_args: Vec<Level>,
    args: Vec<npa_tactic::ApplyArg>,
    head_label: String,
    head_type: Expr,
    span: npa_frontend::Span,
}

#[derive(Clone, Debug)]
struct HumanRewriteResolved {
    head: npa_tactic::TacticHead,
    universe_args: Vec<Level>,
    args: Vec<npa_tactic::ApplyArg>,
    direction: npa_tactic::RewriteDirection,
    head_label: String,
    head_type: Expr,
    span: npa_frontend::Span,
}

#[derive(Clone, Debug)]
enum HumanApplyGlobalCandidate {
    Imported {
        module: npa_cert::ModuleName,
        export_hash: npa_cert::Hash,
        certificate_hash: npa_cert::Hash,
        name: npa_cert::Name,
        decl_interface_hash: npa_cert::Hash,
    },
    Current {
        name: npa_cert::Name,
        decl_interface_hash: npa_cert::Hash,
    },
}

impl HumanApplyGlobalCandidate {
    fn name(&self) -> &npa_cert::Name {
        match self {
            Self::Imported { name, .. } | Self::Current { name, .. } => name,
        }
    }

    fn sort_key(&self) -> String {
        match self {
            Self::Imported {
                module,
                export_hash,
                certificate_hash,
                name,
                decl_interface_hash,
            } => format!(
                "imported:{}:{}:{}:{}:{}",
                module.as_dotted(),
                human_apply_hash_hex(export_hash),
                human_apply_hash_hex(certificate_hash),
                name.as_dotted(),
                human_apply_hash_hex(decl_interface_hash)
            ),
            Self::Current {
                name,
                decl_interface_hash,
            } => format!(
                "current:{}:{}",
                name.as_dotted(),
                human_apply_hash_hex(decl_interface_hash)
            ),
        }
    }

    fn tactic_head(&self) -> npa_tactic::TacticHead {
        match self {
            Self::Imported {
                name,
                decl_interface_hash,
                ..
            } => npa_tactic::TacticHead::Imported {
                name: name.clone(),
                decl_interface_hash: *decl_interface_hash,
            },
            Self::Current {
                name,
                decl_interface_hash,
            } => npa_tactic::TacticHead::CurrentModule {
                name: name.clone(),
                decl_interface_hash: *decl_interface_hash,
            },
        }
    }
}

fn human_apply_resolve(
    state: &npa_tactic::MachineProofState,
    goal: &npa_tactic::MachineGoal,
    term: &npa_frontend::HumanExpr,
    current_source_interface: &npa_frontend::HumanSourceInterface,
    imported_source_interfaces: &[npa_frontend::HumanImportedSourceInterface],
) -> Result<HumanApplyResolved, HumanApplyTacticError> {
    let npa_frontend::HumanExpr::Ident {
        name,
        universe_args,
        span,
        ..
    } = term
    else {
        return Err(human_apply_unsupported_head_diagnostic(term.span()).into());
    };

    let explicit_universe_args = human_apply_universe_args(universe_args.as_deref());
    if let Some(local_name) = human_apply_local_head(goal, name, *span)? {
        if !explicit_universe_args.is_empty() {
            return Err(human_apply_unsupported_diagnostic(
                *span,
                "local apply heads do not accept universe arguments",
            )
            .into());
        }
        let local_index = goal
            .context
            .iter()
            .position(|local| local.name == local_name)
            .expect("resolved local apply head should exist");
        let local = &goal.context[local_index];
        let ctx = human_apply_goal_ctx(state, goal, *span)?;
        let head_bvar = Expr::bvar((goal.context.len() - 1 - local_index) as u32);
        let head_type = state
            .env
            .kernel_env()
            .infer(&ctx, &state.root.universe_params, &head_bvar)
            .map_err(|err| {
                human_apply_unsupported_diagnostic(
                    *span,
                    format!("cannot infer local apply head {} type: {err:?}", local.name),
                )
            })?;
        let args = human_apply_args_for_type(
            state,
            goal,
            &head_type,
            &[],
            *span,
            &format!("local {}", local.name),
        )?;
        return Ok(HumanApplyResolved {
            head: npa_tactic::TacticHead::Local {
                name: local.name.clone(),
            },
            universe_args: Vec::new(),
            args,
            head_label: local.name.clone(),
            head_type,
            span: *span,
        });
    }

    let candidate = human_apply_global_head(state, goal, name, *span)?;
    let decl = state
        .env
        .kernel_env()
        .decl(&candidate.name().as_dotted())
        .ok_or_else(|| {
            human_apply_unsupported_diagnostic(
                *span,
                format!(
                    "apply head {} is not present in the kernel environment",
                    candidate.name().as_dotted()
                ),
            )
        })?;
    let universe_params = decl.universe_params();
    let universe_args = if let Some(args) = universe_args {
        let args = human_apply_universe_args(Some(args));
        if args.len() != universe_params.len() {
            return Err(human_apply_unsupported_diagnostic(
                *span,
                format!(
                    "apply head {} expects {} universe argument(s), got {}",
                    candidate.name().as_dotted(),
                    universe_params.len(),
                    args.len()
                ),
            )
            .into());
        }
        args
    } else if universe_params.is_empty() {
        Vec::new()
    } else {
        return Err(human_apply_unsupported_diagnostic(
            *span,
            format!(
                "apply head {} requires explicit universe arguments in the Human apply MVP",
                candidate.name().as_dotted()
            ),
        )
        .into());
    };
    let head_type =
        npa_kernel::subst::subst_levels_expr(decl.ty(), universe_params, &universe_args);
    let implicit_profile = human_apply_global_implicit_profile(
        &candidate,
        current_source_interface,
        imported_source_interfaces,
    );
    let args = human_apply_args_for_type(
        state,
        goal,
        &head_type,
        &implicit_profile,
        *span,
        &candidate.name().as_dotted(),
    )?;
    Ok(HumanApplyResolved {
        head: candidate.tactic_head(),
        universe_args,
        args,
        head_label: candidate.name().as_dotted(),
        head_type,
        span: *span,
    })
}

fn human_rewrite_resolve_rule(
    state: &npa_tactic::MachineProofState,
    goal_id: npa_tactic::GoalId,
    rule: &npa_frontend::HumanRewriteRuleSyntax,
    _current_source_interface: &npa_frontend::HumanSourceInterface,
    _imported_source_interfaces: &[npa_frontend::HumanImportedSourceInterface],
) -> Result<HumanRewriteResolved, HumanRewriteTacticError> {
    let goal = state.goal(goal_id)?;
    let npa_frontend::HumanExpr::Ident {
        name,
        universe_args,
        span,
        ..
    } = &rule.term
    else {
        return Err(human_rewrite_unsupported_head_diagnostic(rule.term.span()).into());
    };

    let explicit_universe_args = human_apply_universe_args(universe_args.as_deref());
    if let Some(local_name) =
        human_apply_local_head(&goal, name, *span).map_err(human_rewrite_from_apply_error)?
    {
        if !explicit_universe_args.is_empty() {
            return Err(human_rewrite_unsupported_diagnostic(
                *span,
                "local rw rule heads do not accept universe arguments",
            )
            .into());
        }
        let local_index = goal
            .context
            .iter()
            .position(|local| local.name == local_name)
            .expect("resolved local rewrite head should exist");
        let local = &goal.context[local_index];
        let ctx =
            human_apply_goal_ctx(state, &goal, *span).map_err(human_rewrite_from_apply_error)?;
        let head_bvar = Expr::bvar((goal.context.len() - 1 - local_index) as u32);
        let head_type = state
            .env
            .kernel_env()
            .infer(&ctx, &state.root.universe_params, &head_bvar)
            .map_err(|err| {
                human_rewrite_unsupported_diagnostic(
                    *span,
                    format!("cannot infer local rw rule {} type: {err:?}", local.name),
                )
            })?;
        let args = human_rewrite_args_for_type(state, &goal, &head_type, *span, &local.name)?;
        return Ok(HumanRewriteResolved {
            head: npa_tactic::TacticHead::Local {
                name: local.name.clone(),
            },
            universe_args: Vec::new(),
            args,
            direction: human_rewrite_direction(rule.direction),
            head_label: local.name.clone(),
            head_type,
            span: *span,
        });
    }

    let candidate = human_apply_global_head(state, &goal, name, *span)
        .map_err(human_rewrite_from_apply_error)?;
    let decl = state
        .env
        .kernel_env()
        .decl(&candidate.name().as_dotted())
        .ok_or_else(|| {
            human_rewrite_unsupported_diagnostic(
                *span,
                format!(
                    "rw rule {} is not present in the kernel environment",
                    candidate.name().as_dotted()
                ),
            )
        })?;
    let universe_params = decl.universe_params();
    let universe_args = if let Some(args) = universe_args {
        let args = human_apply_universe_args(Some(args));
        if args.len() != universe_params.len() {
            return Err(human_rewrite_unsupported_diagnostic(
                *span,
                format!(
                    "rw rule {} expects {} universe argument(s), got {}",
                    candidate.name().as_dotted(),
                    universe_params.len(),
                    args.len()
                ),
            )
            .into());
        }
        args
    } else if universe_params.is_empty() {
        Vec::new()
    } else {
        return Err(human_rewrite_unsupported_diagnostic(
            *span,
            format!(
                "rw rule {} requires explicit universe arguments in the Human rw MVP",
                candidate.name().as_dotted()
            ),
        )
        .into());
    };
    let head_type =
        npa_kernel::subst::subst_levels_expr(decl.ty(), universe_params, &universe_args);
    let args = human_rewrite_args_for_type(
        state,
        &goal,
        &head_type,
        *span,
        &candidate.name().as_dotted(),
    )?;
    Ok(HumanRewriteResolved {
        head: candidate.tactic_head(),
        universe_args,
        args,
        direction: human_rewrite_direction(rule.direction),
        head_label: candidate.name().as_dotted(),
        head_type,
        span: *span,
    })
}

fn human_rewrite_args_for_type(
    state: &npa_tactic::MachineProofState,
    goal: &npa_tactic::MachineGoal,
    head_type: &Expr,
    span: npa_frontend::Span,
    head_label: &str,
) -> Result<Vec<npa_tactic::ApplyArg>, HumanRewriteTacticError> {
    let ctx = human_apply_goal_ctx(state, goal, span).map_err(human_rewrite_from_apply_error)?;
    let env = state.env.kernel_env();
    let delta = &state.root.universe_params;
    let mut current = head_type.clone();
    let mut args = Vec::new();

    loop {
        let whnf = env.whnf(&ctx, delta, &current).map_err(|err| {
            human_rewrite_unsupported_diagnostic(
                span,
                format!("cannot inspect rw rule {head_label} type: {err:?}"),
            )
        })?;
        let Expr::Pi { body, .. } = whnf else {
            break;
        };
        let arg = npa_tactic::ApplyArg::InferFromTarget;
        let placeholder = human_apply_placeholder(&arg, goal);
        current = instantiate(&body, &placeholder).map_err(|err| {
            human_rewrite_unsupported_diagnostic(
                span,
                format!("cannot instantiate rw rule {head_label} type: {err:?}"),
            )
        })?;
        args.push(arg);
    }

    Ok(args)
}

fn human_rewrite_direction(
    direction: npa_frontend::HumanRewriteDirection,
) -> npa_tactic::RewriteDirection {
    match direction {
        npa_frontend::HumanRewriteDirection::Forward => npa_tactic::RewriteDirection::Forward,
        npa_frontend::HumanRewriteDirection::Backward => npa_tactic::RewriteDirection::Backward,
    }
}

fn human_rewrite_from_apply_error(error: HumanApplyTacticError) -> HumanRewriteTacticError {
    match error {
        HumanApplyTacticError::Human(error) => HumanRewriteTacticError::Human(error),
        HumanApplyTacticError::Machine(diagnostic) => HumanRewriteTacticError::Machine(diagnostic),
    }
}

fn human_apply_local_head(
    goal: &npa_tactic::MachineGoal,
    name: &npa_frontend::HumanName,
    span: npa_frontend::Span,
) -> Result<Option<String>, HumanApplyTacticError> {
    if name.parts.len() != 1 {
        return Ok(None);
    }
    let matches = goal
        .context
        .iter()
        .filter(|local| local.name == name.parts[0])
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [] => Ok(None),
        [local] => Ok(Some(local.name.clone())),
        _ => Err(npa_frontend::HumanDiagnostic::error(
            npa_frontend::HumanDiagnosticKind::AmbiguousName,
            span,
            format!("ambiguous local apply head {}", name.as_dotted()),
        )
        .with_phase(npa_frontend::HumanDiagnosticPhase::TacticValidation)
        .with_payload(human_tactic_goal_payload(goal, span))
        .into()),
    }
}

fn human_apply_global_head(
    state: &npa_tactic::MachineProofState,
    goal: &npa_tactic::MachineGoal,
    name: &npa_frontend::HumanName,
    span: npa_frontend::Span,
) -> Result<HumanApplyGlobalCandidate, HumanApplyTacticError> {
    for candidates in human_apply_global_candidate_levels(state, name) {
        let candidates = human_apply_dedupe_candidates(candidates);
        if candidates.is_empty() {
            continue;
        }
        if candidates.len() == 1 {
            return Ok(candidates[0].clone());
        }
        return Err(npa_frontend::HumanDiagnostic::error(
            npa_frontend::HumanDiagnosticKind::AmbiguousName,
            span,
            format!("ambiguous apply head {}", name.as_dotted()),
        )
        .with_payload(npa_frontend::HumanDiagnosticPayload {
            candidates: candidates
                .into_iter()
                .map(|candidate| candidate.sort_key())
                .collect(),
            hole_goals: vec![human_tactic_goal_display(goal, span)],
            ..npa_frontend::HumanDiagnosticPayload::default()
        })
        .with_phase(npa_frontend::HumanDiagnosticPhase::TacticValidation)
        .into());
    }

    Err(npa_frontend::HumanDiagnostic::error(
        npa_frontend::HumanDiagnosticKind::UnknownIdentifier,
        span,
        format!("unknown apply head {}", name.as_dotted()),
    )
    .with_phase(npa_frontend::HumanDiagnosticPhase::TacticValidation)
    .with_payload(human_tactic_goal_payload(goal, span))
    .into())
}

fn human_apply_global_candidate_levels(
    state: &npa_tactic::MachineProofState,
    name: &npa_frontend::HumanName,
) -> Vec<Vec<HumanApplyGlobalCandidate>> {
    let exact = npa_cert::Name(name.parts.clone());
    if name.parts.len() == 1 {
        vec![
            human_apply_exact_candidates(state, &exact),
            human_apply_short_name_candidates(state, &name.parts[0]),
        ]
    } else {
        vec![
            human_apply_exact_candidates(state, &exact),
            human_apply_suffix_candidates(state, &name.parts),
        ]
    }
}

fn human_apply_exact_candidates(
    state: &npa_tactic::MachineProofState,
    name: &npa_cert::Name,
) -> Vec<HumanApplyGlobalCandidate> {
    let current = state
        .env
        .checked_current_decls
        .iter()
        .filter(|decl| decl.signature().name() == name)
        .map(|decl| HumanApplyGlobalCandidate::Current {
            name: decl.signature().name().clone(),
            decl_interface_hash: decl.signature().decl_interface_hash(),
        })
        .collect::<Vec<_>>();
    if !current.is_empty() {
        return current;
    }

    human_apply_imported_candidates(state, |export| &export.name == name)
}

fn human_apply_short_name_candidates(
    state: &npa_tactic::MachineProofState,
    short_name: &str,
) -> Vec<HumanApplyGlobalCandidate> {
    let current = state
        .env
        .checked_current_decls
        .iter()
        .filter(|decl| {
            decl.signature()
                .name()
                .0
                .last()
                .is_some_and(|part| part == short_name)
        })
        .map(|decl| HumanApplyGlobalCandidate::Current {
            name: decl.signature().name().clone(),
            decl_interface_hash: decl.signature().decl_interface_hash(),
        })
        .collect::<Vec<_>>();
    if !current.is_empty() {
        return current;
    }

    human_apply_imported_candidates(state, |export| {
        export.name.0.last().is_some_and(|part| part == short_name)
    })
}

fn human_apply_suffix_candidates(
    state: &npa_tactic::MachineProofState,
    suffix: &[String],
) -> Vec<HumanApplyGlobalCandidate> {
    let current = state
        .env
        .checked_current_decls
        .iter()
        .filter(|decl| human_apply_name_has_suffix(&decl.signature().name().0, suffix))
        .map(|decl| HumanApplyGlobalCandidate::Current {
            name: decl.signature().name().clone(),
            decl_interface_hash: decl.signature().decl_interface_hash(),
        })
        .collect::<Vec<_>>();
    if !current.is_empty() {
        return current;
    }

    human_apply_imported_candidates(state, |export| {
        human_apply_name_has_suffix(&export.name.0, suffix)
    })
}

fn human_apply_imported_candidates(
    state: &npa_tactic::MachineProofState,
    mut is_match: impl FnMut(&npa_tactic::VerifiedExportSignature) -> bool,
) -> Vec<HumanApplyGlobalCandidate> {
    let mut candidates = Vec::new();
    for import in state
        .env
        .imports
        .iter()
        .filter(|import| import.is_visible())
    {
        for export in import.exports().iter().filter(|export| is_match(export)) {
            candidates.push(HumanApplyGlobalCandidate::Imported {
                module: import.module().clone(),
                export_hash: import.export_hash(),
                certificate_hash: import.certificate_hash(),
                name: export.name.clone(),
                decl_interface_hash: export.decl_interface_hash,
            });
        }
    }
    candidates
}

fn human_apply_dedupe_candidates(
    candidates: Vec<HumanApplyGlobalCandidate>,
) -> Vec<HumanApplyGlobalCandidate> {
    let mut deduped = BTreeMap::new();
    for candidate in candidates {
        deduped
            .entry(candidate.sort_key())
            .or_insert_with(|| candidate);
    }
    deduped.into_values().collect()
}

fn human_apply_global_implicit_profile(
    candidate: &HumanApplyGlobalCandidate,
    current_source_interface: &npa_frontend::HumanSourceInterface,
    imported_source_interfaces: &[npa_frontend::HumanImportedSourceInterface],
) -> Vec<npa_frontend::MachineCallableBinderVisibility> {
    match candidate {
        HumanApplyGlobalCandidate::Current {
            name,
            decl_interface_hash,
        } => current_source_interface
            .declarations
            .iter()
            .find(|decl| {
                npa_cert::Name(decl.name.parts.clone()) == *name
                    && decl.decl_interface_hash == Some(*decl_interface_hash)
            })
            .map(|decl| npa_frontend::machine_callable_profile_from_human_binders(&decl.binders))
            .unwrap_or_default(),
        HumanApplyGlobalCandidate::Imported {
            module,
            export_hash,
            certificate_hash,
            name,
            decl_interface_hash,
        } => imported_source_interfaces
            .iter()
            .filter(|interface| {
                interface.module == *module
                    && interface.export_hash == *export_hash
                    && interface.certificate_hash == Some(*certificate_hash)
            })
            .flat_map(|interface| interface.source_interface.declarations.iter())
            .find(|decl| {
                npa_cert::Name(decl.name.parts.clone()) == *name
                    && decl.decl_interface_hash == Some(*decl_interface_hash)
            })
            .map(|decl| npa_frontend::machine_callable_profile_from_human_binders(&decl.binders))
            .or_else(|| {
                if npa_cert::builtin_decl_interface_hash(name) == Some(*decl_interface_hash) {
                    npa_frontend::builtin_machine_callable_profile(name)
                } else {
                    None
                }
            })
            .unwrap_or_default(),
    }
}

fn human_apply_args_for_type(
    state: &npa_tactic::MachineProofState,
    goal: &npa_tactic::MachineGoal,
    head_type: &Expr,
    implicit_profile: &[npa_frontend::MachineCallableBinderVisibility],
    span: npa_frontend::Span,
    head_label: &str,
) -> Result<Vec<npa_tactic::ApplyArg>, HumanApplyTacticError> {
    let ctx = human_apply_goal_ctx(state, goal, span)?;
    let env = state.env.kernel_env();
    let delta = &state.root.universe_params;
    let mut current = head_type.clone();
    let mut args = Vec::new();

    loop {
        let whnf = env.whnf(&ctx, delta, &current).map_err(|err| {
            human_apply_unsupported_diagnostic(
                span,
                format!("cannot inspect apply head {head_label} type: {err:?}"),
            )
        })?;
        if env
            .is_defeq(&ctx, delta, &whnf, &goal.target)
            .map_err(|err| {
                human_apply_unsupported_diagnostic(
                    span,
                    format!("cannot compare apply head {head_label} with target: {err:?}"),
                )
            })?
        {
            break;
        }

        let Expr::Pi { ty, body, .. } = whnf else {
            break;
        };
        let domain = *ty;
        let is_implicit = implicit_profile.get(args.len()).is_some_and(|visibility| {
            *visibility == npa_frontend::MachineCallableBinderVisibility::Implicit
        });
        let is_proof_relevant =
            human_apply_domain_is_proof_relevant(state, goal, &ctx, &domain, span)?;
        let arg = if !is_proof_relevant {
            if human_apply_body_returns_current_binder(body.as_ref()) {
                human_apply_nonproof_arg_from_target(goal)
                    .unwrap_or(npa_tactic::ApplyArg::InferFromTarget)
            } else {
                npa_tactic::ApplyArg::InferFromTarget
            }
        } else if is_implicit {
            npa_tactic::ApplyArg::InferFromTarget
        } else {
            npa_tactic::ApplyArg::Subgoal { name_hint: None }
        };
        let placeholder = human_apply_placeholder(&arg, goal);
        current = instantiate(&body, &placeholder).map_err(|err| {
            human_apply_unsupported_diagnostic(
                span,
                format!("cannot instantiate apply head {head_label} type: {err:?}"),
            )
        })?;
        args.push(arg);
    }

    Ok(args)
}

fn human_apply_goal_ctx(
    state: &npa_tactic::MachineProofState,
    goal: &npa_tactic::MachineGoal,
    span: npa_frontend::Span,
) -> Result<Ctx, HumanApplyTacticError> {
    let mut ctx = Ctx::new();
    let env = state.env.kernel_env();
    for local in &goal.context {
        env.check(
            &ctx,
            &state.root.universe_params,
            &local.ty,
            &Expr::sort(npa_kernel::type0()),
        )
        .or_else(|_| {
            env.infer(&ctx, &state.root.universe_params, &local.ty)
                .map(|_| ())
        })
        .map_err(|err| {
            human_apply_unsupported_diagnostic(
                span,
                format!("cannot inspect local context for apply: {err:?}"),
            )
        })?;
        match &local.value {
            Some(value) => ctx.push_definition(local.name.clone(), local.ty.clone(), value.clone()),
            None => ctx.push_assumption(local.name.clone(), local.ty.clone()),
        }
    }
    Ok(ctx)
}

fn human_apply_domain_is_proof_relevant(
    state: &npa_tactic::MachineProofState,
    goal: &npa_tactic::MachineGoal,
    ctx: &Ctx,
    domain: &Expr,
    span: npa_frontend::Span,
) -> Result<bool, HumanApplyTacticError> {
    let sort = state
        .env
        .kernel_env()
        .infer(ctx, &state.root.universe_params, domain)
        .map_err(|err| {
            human_apply_unsupported_diagnostic(
                span,
                format!(
                    "cannot infer apply premise type for goal {}: {err:?}",
                    goal.id.0
                ),
            )
        })?;
    Ok(matches!(sort, Expr::Sort(level) if level == npa_kernel::prop()))
}

fn human_apply_placeholder(arg: &npa_tactic::ApplyArg, goal: &npa_tactic::MachineGoal) -> Expr {
    match arg {
        npa_tactic::ApplyArg::InferFromTarget => goal.target.clone(),
        npa_tactic::ApplyArg::Term(_) => goal.target.clone(),
        npa_tactic::ApplyArg::Subgoal { .. } => Expr::bvar(0),
    }
}

fn human_apply_body_returns_current_binder(body: &Expr) -> bool {
    let mut depth = 0;
    let mut current = body;
    while let Expr::Pi { body, .. } = current {
        depth += 1;
        current = body;
    }
    matches!(current, Expr::BVar(index) if *index as usize == depth)
}

fn human_apply_nonproof_arg_from_target(
    goal: &npa_tactic::MachineGoal,
) -> Option<npa_tactic::ApplyArg> {
    let source = human_apply_render_target_arg(&goal.target, goal)?;
    Some(npa_tactic::ApplyArg::Term(
        npa_tactic::MachineTermSource::new_checked(source).ok()?,
    ))
}

fn human_apply_render_target_arg(expr: &Expr, goal: &npa_tactic::MachineGoal) -> Option<String> {
    match expr {
        Expr::BVar(index) => {
            let index = *index as usize;
            if index >= goal.context.len() {
                return None;
            }
            Some(goal.context[goal.context.len() - 1 - index].name.clone())
        }
        Expr::Const { name, levels } if levels.is_empty() => Some(name.clone()),
        Expr::Sort(level) if *level == npa_kernel::prop() => Some("Prop".to_owned()),
        Expr::Sort(level) if *level == npa_kernel::type0() => Some("Type".to_owned()),
        _ => None,
    }
}

fn human_apply_machine_error(
    diagnostic: npa_tactic::MachineTacticDiagnostic,
    goal: &npa_tactic::MachineGoal,
    resolved: &HumanApplyResolved,
) -> HumanApplyTacticError {
    match &diagnostic.kind {
        npa_tactic::MachineTacticDiagnosticKind::TypeMismatch
        | npa_tactic::MachineTacticDiagnosticKind::TooFewApplyArguments
        | npa_tactic::MachineTacticDiagnosticKind::AmbiguousApplyArgument
        | npa_tactic::MachineTacticDiagnosticKind::MissingExplicitArgument
        | npa_tactic::MachineTacticDiagnosticKind::SubgoalDataArgument => {
            human_tactic_machine_diagnostic(
                &diagnostic,
                resolved.span,
                Some(goal),
                Some(npa_frontend::HumanDiagnosticKind::TypeMismatch),
                format!(
                    "cannot apply `{}`\n\ntarget:\n  {:?}\n\nhead type:\n  {:?}\n\n{}",
                    resolved.head_label, goal.target, resolved.head_type, diagnostic.message
                ),
            )
            .into()
        }
        _ => diagnostic.into(),
    }
}

fn human_rewrite_machine_error(
    diagnostic: npa_tactic::MachineTacticDiagnostic,
    goal: &npa_tactic::MachineGoal,
    resolved: &HumanRewriteResolved,
    site: npa_tactic::RewriteSite,
) -> HumanRewriteTacticError {
    match &diagnostic.kind {
        npa_tactic::MachineTacticDiagnosticKind::AmbiguousRewriteRule
        | npa_tactic::MachineTacticDiagnosticKind::ExpectedEqTarget
        | npa_tactic::MachineTacticDiagnosticKind::InvalidMetaDependency
        | npa_tactic::MachineTacticDiagnosticKind::MissingExplicitArgument
        | npa_tactic::MachineTacticDiagnosticKind::TacticPrimitiveUnavailable
        | npa_tactic::MachineTacticDiagnosticKind::TooManyApplyArguments
        | npa_tactic::MachineTacticDiagnosticKind::TypeMismatch => {
            human_tactic_machine_diagnostic(
                &diagnostic,
                resolved.span,
                Some(goal),
                Some(npa_frontend::HumanDiagnosticKind::TypeMismatch),
                format!(
                    "cannot rewrite with `{}`\n\ndirection:\n  {:?}\n\nsite:\n  {:?}\n\ntarget:\n  {:?}\n\nrule type:\n  {:?}\n\n{}",
                    resolved.head_label,
                    resolved.direction,
                    site,
                    goal.target,
                    resolved.head_type,
                    diagnostic.message
                ),
            )
            .into()
        }
        _ => diagnostic.into(),
    }
}

fn human_simp_lite_machine_error(
    diagnostic: npa_tactic::MachineTacticDiagnostic,
    goal: &npa_tactic::MachineGoal,
    span: npa_frontend::Span,
) -> HumanSimpLiteTacticError {
    match &diagnostic.kind {
        npa_tactic::MachineTacticDiagnosticKind::AmbiguousRewriteRule
        | npa_tactic::MachineTacticDiagnosticKind::AmbiguousSimpRule
        | npa_tactic::MachineTacticDiagnosticKind::ExpectedEqTarget
        | npa_tactic::MachineTacticDiagnosticKind::InvalidSimpRule
        | npa_tactic::MachineTacticDiagnosticKind::SimpNoProgress
        | npa_tactic::MachineTacticDiagnosticKind::TacticPrimitiveUnavailable
        | npa_tactic::MachineTacticDiagnosticKind::TypeMismatch
        | npa_tactic::MachineTacticDiagnosticKind::UnknownSimpRule => {
            human_tactic_machine_diagnostic(
                &diagnostic,
                span,
                Some(goal),
                Some(npa_frontend::HumanDiagnosticKind::TypeMismatch),
                format!(
                    "simp-lite could not close the target\n\ntarget:\n  {:?}\n\n{}",
                    goal.target, diagnostic.message
                ),
            )
            .into()
        }
        _ => diagnostic.into(),
    }
}

fn human_simp_lite_not_closed_diagnostic(
    span: npa_frontend::Span,
    goal: &npa_tactic::MachineGoal,
    residual_target: Option<&Expr>,
) -> npa_frontend::HumanDiagnostic {
    let mut message = format!(
        "simp-lite simplified the target but did not close it in the Human MVP\n\noriginal target:\n  {:?}",
        goal.target
    );
    if let Some(target) = residual_target {
        message.push_str(&format!("\n\nresidual target:\n  {target:?}"));
    }
    npa_frontend::HumanDiagnostic::error(
        npa_frontend::HumanDiagnosticKind::TypeMismatch,
        span,
        message,
    )
    .with_phase(npa_frontend::HumanDiagnosticPhase::TacticExecution)
    .with_payload(human_tactic_goal_payload(goal, span))
}

fn human_induction_machine_error(
    diagnostic: npa_tactic::MachineTacticDiagnostic,
    goal: &npa_tactic::MachineGoal,
    span: npa_frontend::Span,
) -> HumanInductionTacticError {
    match &diagnostic.kind {
        npa_tactic::MachineTacticDiagnosticKind::AmbiguousLocalName
        | npa_tactic::MachineTacticDiagnosticKind::InvalidInductionTarget
        | npa_tactic::MachineTacticDiagnosticKind::InvalidMachineTactic
        | npa_tactic::MachineTacticDiagnosticKind::TacticPrimitiveUnavailable
        | npa_tactic::MachineTacticDiagnosticKind::TypeMismatch
        | npa_tactic::MachineTacticDiagnosticKind::UnknownLocalName => {
            human_tactic_machine_diagnostic(
                &diagnostic,
                span,
                Some(goal),
                Some(npa_frontend::HumanDiagnosticKind::UnsupportedTactic),
                format!(
                    "cannot perform simple induction in the Human MVP\n\ntarget:\n  {:?}\n\n{}",
                    goal.target, diagnostic.message
                ),
            )
            .into()
        }
        _ => diagnostic.into(),
    }
}

fn human_induction_unsupported_diagnostic(
    span: npa_frontend::Span,
    message: impl Into<String>,
) -> npa_frontend::HumanDiagnostic {
    npa_frontend::HumanDiagnostic::error(
        npa_frontend::HumanDiagnosticKind::UnsupportedTactic,
        span,
        message,
    )
    .with_phase(npa_frontend::HumanDiagnosticPhase::TacticValidation)
}

fn human_rewrite_unsupported_head_diagnostic(
    span: npa_frontend::Span,
) -> npa_frontend::HumanDiagnostic {
    human_rewrite_unsupported_diagnostic(
        span,
        "Human rw MVP only supports a resolved local or global name as the rule head",
    )
}

fn human_rewrite_unsupported_diagnostic(
    span: npa_frontend::Span,
    message: impl Into<String>,
) -> npa_frontend::HumanDiagnostic {
    npa_frontend::HumanDiagnostic::error(
        npa_frontend::HumanDiagnosticKind::UnsupportedTactic,
        span,
        message,
    )
    .with_phase(npa_frontend::HumanDiagnosticPhase::TacticValidation)
}

fn human_apply_unsupported_head_diagnostic(
    span: npa_frontend::Span,
) -> npa_frontend::HumanDiagnostic {
    human_apply_unsupported_diagnostic(
        span,
        "Human apply MVP only supports a resolved local or global name as the head",
    )
}

fn human_apply_unsupported_diagnostic(
    span: npa_frontend::Span,
    message: impl Into<String>,
) -> npa_frontend::HumanDiagnostic {
    npa_frontend::HumanDiagnostic::error(
        npa_frontend::HumanDiagnosticKind::UnsupportedTactic,
        span,
        message,
    )
    .with_phase(npa_frontend::HumanDiagnosticPhase::TacticValidation)
}

fn human_apply_universe_args(levels: Option<&[npa_frontend::HumanLevel]>) -> Vec<Level> {
    levels
        .unwrap_or_default()
        .iter()
        .map(human_apply_level)
        .collect()
}

fn human_apply_level(level: &npa_frontend::HumanLevel) -> Level {
    match level {
        npa_frontend::HumanLevel::Nat { value, .. } => {
            let mut level = Level::zero();
            for _ in 0..*value {
                level = Level::succ(level);
            }
            level
        }
        npa_frontend::HumanLevel::Param { name, .. } => Level::param(name.clone()),
        npa_frontend::HumanLevel::Succ { level, .. } => Level::succ(human_apply_level(level)),
        npa_frontend::HumanLevel::Max { lhs, rhs, .. } => {
            Level::max(human_apply_level(lhs), human_apply_level(rhs))
        }
        npa_frontend::HumanLevel::IMax { lhs, rhs, .. } => {
            Level::imax(human_apply_level(lhs), human_apply_level(rhs))
        }
    }
}

fn human_apply_name_has_suffix(name: &[String], suffix: &[String]) -> bool {
    name.len() >= suffix.len() && &name[(name.len() - suffix.len())..] == suffix
}

fn human_apply_hash_hex(hash: &npa_cert::Hash) -> String {
    hash.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn human_script_intro_error(error: HumanIntroTacticError) -> HumanTacticScriptError {
    match error {
        HumanIntroTacticError::Human(error) => HumanTacticScriptError::Human(error),
        HumanIntroTacticError::Machine(diagnostic) => HumanTacticScriptError::Machine(diagnostic),
    }
}

fn human_script_apply_error(error: HumanApplyTacticError) -> HumanTacticScriptError {
    match error {
        HumanApplyTacticError::Human(error) => HumanTacticScriptError::Human(error),
        HumanApplyTacticError::Machine(diagnostic) => HumanTacticScriptError::Machine(diagnostic),
    }
}

fn human_script_rewrite_error(error: HumanRewriteTacticError) -> HumanTacticScriptError {
    match error {
        HumanRewriteTacticError::Human(error) => HumanTacticScriptError::Human(error),
        HumanRewriteTacticError::Machine(diagnostic) => HumanTacticScriptError::Machine(diagnostic),
    }
}

fn human_script_simp_lite_error(error: HumanSimpLiteTacticError) -> HumanTacticScriptError {
    match error {
        HumanSimpLiteTacticError::Human(error) => HumanTacticScriptError::Human(error),
        HumanSimpLiteTacticError::Machine(diagnostic) => {
            HumanTacticScriptError::Machine(diagnostic)
        }
    }
}

fn human_script_induction_error(error: HumanInductionTacticError) -> HumanTacticScriptError {
    match error {
        HumanInductionTacticError::Human(error) => HumanTacticScriptError::Human(error),
        HumanInductionTacticError::Machine(diagnostic) => {
            HumanTacticScriptError::Machine(diagnostic)
        }
    }
}

fn human_script_term_error(error: HumanTacticTermError) -> HumanTacticScriptError {
    match error {
        HumanTacticTermError::Human(error) => HumanTacticScriptError::Human(error),
        HumanTacticTermError::Machine(diagnostic) => HumanTacticScriptError::Machine(diagnostic),
    }
}

fn human_script_no_goals_diagnostic(span: npa_frontend::Span) -> npa_frontend::HumanDiagnostic {
    npa_frontend::HumanDiagnostic::error(
        npa_frontend::HumanDiagnosticKind::NoGoalsButTacticRemaining,
        span,
        "Human tactic script has a remaining tactic after all goals were closed",
    )
    .with_phase(npa_frontend::HumanDiagnosticPhase::TacticExecution)
}

fn human_script_unresolved_goal_diagnostic(
    span: npa_frontend::Span,
    state: &npa_tactic::MachineProofState,
) -> npa_frontend::HumanDiagnostic {
    let open_goal_count = state.open_goals.len();
    let hole_goals = state
        .open_goals
        .iter()
        .filter_map(|goal_id| state.goal(*goal_id).ok())
        .map(|goal| human_tactic_goal_display(&goal, span))
        .collect::<Vec<_>>();

    npa_frontend::HumanDiagnostic::error(
        npa_frontend::HumanDiagnosticKind::UnresolvedGoal,
        span,
        format!("Human tactic script finished with {open_goal_count} open goal(s)"),
    )
    .with_phase(npa_frontend::HumanDiagnosticPhase::TacticUnresolvedGoal)
    .with_payload(npa_frontend::HumanDiagnosticPayload {
        hole_goals,
        ..npa_frontend::HumanDiagnosticPayload::default()
    })
}

fn frontend_import_from_tactic_ref(
    import: &npa_tactic::VerifiedImportRef,
) -> npa_frontend::VerifiedImport {
    let mut frontend = npa_frontend::VerifiedImport::from(import.verified_module());
    let visible_exports = import
        .exports()
        .iter()
        .map(|export| (export.name.clone(), export.decl_interface_hash))
        .collect::<std::collections::BTreeSet<_>>();
    frontend.exports.retain(|export| {
        visible_exports.contains(&(export.name.clone(), export.decl_interface_hash))
    });
    frontend
}

fn human_tactic_current_generated_decls(
    checked_current_decls: &[npa_tactic::CheckedCurrentDecl],
) -> Vec<npa_frontend::MachineCheckedCurrentGeneratedDecl> {
    let mut generated = Vec::new();
    for decl in checked_current_decls {
        if let Decl::Inductive { data, .. } = decl.core_decl() {
            for constructor in &data.constructors {
                generated.push(npa_frontend::MachineCheckedCurrentGeneratedDecl {
                    name: npa_cert::Name::from_dotted(&constructor.name),
                    parent_source_index: decl.source_index(),
                    decl_interface_hash: decl.signature().decl_interface_hash(),
                });
            }
            if let Some(recursor) = &data.recursor {
                generated.push(npa_frontend::MachineCheckedCurrentGeneratedDecl {
                    name: npa_cert::Name::from_dotted(&recursor.name),
                    parent_source_index: decl.source_index(),
                    decl_interface_hash: decl.signature().decl_interface_hash(),
                });
            }
        }
    }
    generated
}

fn active_human_verified_import_refs(
    verified_modules: &[npa_cert::VerifiedModule],
    active_imports: &[npa_frontend::HumanImportedSourceInterface],
) -> Result<Vec<npa_tactic::VerifiedImportRef>, HumanStartProofError> {
    active_imports
        .iter()
        .map(|active| {
            let verified = verified_modules
                .iter()
                .find(|module| {
                    let import = npa_frontend::VerifiedImport::from(*module);
                    import.module == active.module
                        && import.export_hash == active.export_hash
                        && import.certificate_hash == active.certificate_hash
                })
                .ok_or_else(|| {
                    npa_tactic::MachineTacticDiagnostic::new(
                        npa_tactic::MachineTacticDiagnosticKind::InvalidVerifiedImport,
                        format!(
                            "active Human import {} is not present in verified modules",
                            active.module.as_dotted()
                        ),
                    )
                })?;
            npa_tactic::VerifiedImportRef::from_verified_module(verified)
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(HumanStartProofError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        create_machine_session, HumanCurrentModuleSource, HumanDocumentVersion,
        HumanProofSessionStore, HumanStateRequestHeader,
    };

    fn human_payload(
        diagnostic: &npa_frontend::HumanDiagnostic,
    ) -> &npa_frontend::HumanDiagnosticPayload {
        diagnostic
            .payload
            .as_deref()
            .expect("Human diagnostic should carry a structured payload")
    }

    #[test]
    fn human_api_compiles_source_to_certificate_without_machine_session() {
        let request = HumanCompileCertificateRequest {
            current_module: npa_cert::Name::from_dotted("Api.Human"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source: "axiom P : Prop",
            },
            verified_modules: &[],
            imported_source_interfaces: &[],
            options: human_api_default_compile_options(),
        };

        let ok = compile_human_source_to_certificate(request)
            .expect("Human API should compile source to a certificate certificate");
        assert_eq!(ok.source_interface.declarations.len(), 1);
        let bytes = npa_cert::encode_module_cert(&ok.certificate)
            .expect("Human API certificate should encode");
        let verified = npa_cert::verify_module_cert(
            &bytes,
            &mut npa_cert::VerifierSession::new(),
            &npa_cert::AxiomPolicy::normal(),
        )
        .expect("Human API certificate should verify with normal axiom policy");

        assert_eq!(verified.module(), &npa_cert::Name::from_dotted("Api.Human"));
    }

    #[test]
    fn human_session_create_returns_open_session_with_initial_document_version() {
        let mut store = HumanProofSessionStore::new();

        let ok = create_human_session(
            &mut store,
            HumanSessionCreateRequest {
                current_module: npa_cert::Name::from_dotted("Api.Session"),
                current_source: HumanCurrentModuleSource {
                    file_id: npa_frontend::FileId(11),
                    source: "axiom P : Prop",
                },
                verified_modules: &[],
                imported_source_interfaces: &[],
                options: human_api_default_compile_options(),
            },
        )
        .expect("Human session id allocation should succeed");

        assert_eq!(ok.document_version, HumanDocumentVersion::initial());
        assert_eq!(ok.status, HumanProofSessionStatus::Open);
        assert!(ok.messages.is_empty());
        let session = store
            .session(&ok.session_id)
            .expect("created Human session should be stored");
        assert_eq!(session.session_id, ok.session_id);
        assert_eq!(session.document.document_id, ok.document_id);
        assert_eq!(session.document.document_version, ok.document_version);
        assert_eq!(session.document.file_id, npa_frontend::FileId(11));
        assert_eq!(session.document.source, "axiom P : Prop");
        assert_eq!(
            session.document.current_module,
            npa_cert::Name::from_dotted("Api.Session")
        );
        assert!(session.source_interface.is_some());
        assert!(session.active_imported_source_interfaces.is_empty());
        assert!(session.messages.is_empty());
    }

    #[test]
    fn human_session_create_returns_initial_parse_messages_without_closing_session() {
        let mut store = HumanProofSessionStore::new();

        let ok = create_human_session(
            &mut store,
            HumanSessionCreateRequest {
                current_module: npa_cert::Name::from_dotted("Api.SessionDiagnostics"),
                current_source: HumanCurrentModuleSource {
                    file_id: npa_frontend::FileId(15),
                    source: "def bad : Type :=",
                },
                verified_modules: &[],
                imported_source_interfaces: &[],
                options: human_api_default_compile_options(),
            },
        )
        .expect("Human session should still open with source diagnostics");

        assert_eq!(ok.status, HumanProofSessionStatus::Open);
        assert_eq!(ok.messages.len(), 1);
        assert_eq!(store.session_count(), 1);
        let session = store
            .session(&ok.session_id)
            .expect("diagnostic session should be stored");
        assert_eq!(session.messages, ok.messages);
        assert!(session.source_interface.is_none());
    }

    #[test]
    fn human_session_document_update_increments_version_and_rejects_stale_state_request() {
        let mut store = HumanProofSessionStore::new();
        let created = create_human_session(
            &mut store,
            HumanSessionCreateRequest {
                current_module: npa_cert::Name::from_dotted("Api.Update"),
                current_source: HumanCurrentModuleSource {
                    file_id: npa_frontend::FileId(12),
                    source: "axiom P : Prop",
                },
                verified_modules: &[],
                imported_source_interfaces: &[],
                options: human_api_default_compile_options(),
            },
        )
        .expect("Human session should be created");

        let updated = update_human_document(
            &mut store,
            HumanDocumentUpdateRequest {
                session_id: created.session_id.clone(),
                current_module: npa_cert::Name::from_dotted("Api.Update"),
                current_source: HumanCurrentModuleSource {
                    file_id: npa_frontend::FileId(12),
                    source: "axiom P : Prop\naxiom q : P",
                },
                verified_modules: &[],
                imported_source_interfaces: &[],
                options: human_api_default_compile_options(),
            },
        )
        .expect("Human document update should succeed");

        assert_eq!(updated.document_id, created.document_id);
        assert_eq!(updated.document_version.as_u64(), 2);
        let err = validate_human_state_request_document(
            &store,
            HumanStateRequestHeader {
                session_id: created.session_id.clone(),
                document_id: created.document_id.clone(),
                document_version: created.document_version,
            },
        )
        .expect_err("old document version should be stale after update");
        assert_eq!(
            err,
            HumanStateRequestError::StaleDocumentVersion {
                session_id: created.session_id.clone(),
                document_id: created.document_id.clone(),
                requested: HumanDocumentVersion::initial(),
                current: updated.document_version,
            }
        );

        validate_human_state_request_document(
            &store,
            HumanStateRequestHeader {
                session_id: updated.session_id,
                document_id: updated.document_id,
                document_version: updated.document_version,
            },
        )
        .expect("current document version should validate");
    }

    #[test]
    fn human_session_stores_explicit_imports_without_machine_session_integration() {
        let producer = compile_human_source_to_certificate(HumanCompileCertificateRequest {
            current_module: npa_cert::Name::from_dotted("Api.SessionLib"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(13),
                source: "axiom A : Type",
            },
            verified_modules: &[],
            imported_source_interfaces: &[],
            options: human_api_default_compile_options(),
        })
        .expect("producer Human API request should compile");
        let bytes =
            npa_cert::encode_module_cert(&producer.certificate).expect("producer cert encodes");
        let verified = npa_cert::verify_module_cert(
            &bytes,
            &mut npa_cert::VerifierSession::new(),
            &npa_cert::AxiomPolicy::normal(),
        )
        .expect("producer cert verifies");
        let import = npa_frontend::VerifiedImport::from(&verified);
        let source_interface = npa_frontend::HumanImportedSourceInterface {
            module: import.module.clone(),
            export_hash: import.export_hash,
            certificate_hash: import.certificate_hash,
            source_interface: producer.source_interface,
        };

        let mut store = HumanProofSessionStore::new();
        let ok = create_human_session(
            &mut store,
            HumanSessionCreateRequest {
                current_module: npa_cert::Name::from_dotted("Api.SessionUser"),
                current_source: HumanCurrentModuleSource {
                    file_id: npa_frontend::FileId(14),
                    source: "axiom B : Type",
                },
                verified_modules: std::slice::from_ref(&verified),
                imported_source_interfaces: std::slice::from_ref(&source_interface),
                options: human_api_default_compile_options(),
            },
        )
        .expect("Human session should store explicit import inputs");
        let session = store
            .session(&ok.session_id)
            .expect("created session should be stored");

        assert_eq!(session.document.verified_modules, vec![verified]);
        assert_eq!(
            session.document.imported_source_interfaces,
            vec![source_interface]
        );
        assert_eq!(store.session_count(), 1);
    }

    #[test]
    fn human_api_core_request_uses_explicit_verified_modules_and_current_source() {
        let request = HumanCompileCoreRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanCore"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(7),
                source: "def id : forall (A : Type), Type := fun A => A",
            },
            verified_modules: &[],
            imported_source_interfaces: &[],
            options: HumanApiCompileOptions {
                max_notation_candidates: 4,
                ..human_api_default_compile_options()
            },
        };

        let ok = compile_human_source_to_core(request)
            .expect("Human API should compile explicit current source to core");

        assert_eq!(ok.core_module.declarations.len(), 1);
        assert_eq!(ok.source_interface.declarations.len(), 1);
    }

    #[test]
    fn human_api_returns_source_interface_for_downstream_human_imports() {
        let producer = compile_human_source_to_certificate(HumanCompileCertificateRequest {
            current_module: npa_cert::Name::from_dotted("Api.Lib"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source: "\
axiom A : Type
def choose {B : Type} (x y : B) : B := x
infixl:65 \" ++ \" => choose",
            },
            verified_modules: &[],
            imported_source_interfaces: &[],
            options: human_api_default_compile_options(),
        })
        .expect("producer Human API request should compile");
        assert!(producer
            .source_interface
            .declarations
            .iter()
            .all(|decl| decl.decl_interface_hash.is_some()));
        let bytes =
            npa_cert::encode_module_cert(&producer.certificate).expect("producer cert encodes");
        let verified = npa_cert::verify_module_cert(
            &bytes,
            &mut npa_cert::VerifierSession::new(),
            &npa_cert::AxiomPolicy::normal(),
        )
        .expect("producer cert verifies");
        let import = npa_frontend::VerifiedImport::from(&verified);
        let source_interface = npa_frontend::HumanImportedSourceInterface {
            module: import.module.clone(),
            export_hash: import.export_hash,
            certificate_hash: import.certificate_hash,
            source_interface: producer.source_interface,
        };

        let consumer = compile_human_source_to_core(HumanCompileCoreRequest {
            current_module: npa_cert::Name::from_dotted("Api.Consumer"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(1),
                source: "\
import Api.Lib
axiom a : A
def use : A := a ++ a",
            },
            verified_modules: std::slice::from_ref(&verified),
            imported_source_interfaces: &[source_interface],
            options: human_api_default_compile_options(),
        })
        .expect("consumer Human API request should use imported source metadata");

        assert_eq!(consumer.core_module.declarations.len(), 2);
    }

    #[test]
    fn human_api_compile_core_handoffs_by_proof_expr_before_later_decl() {
        let ok = compile_human_source_to_core(HumanCompileCoreRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanByCore"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source: "\
theorem id_prop : forall (P : Prop), Prop := by
  intro P
  exact P
def use (P : Prop) : Prop := id_prop P",
            },
            verified_modules: &[],
            imported_source_interfaces: &[],
            options: human_api_default_compile_options(),
        })
        .expect("Human API core path should replace by proof with extracted proof Expr");

        assert_eq!(ok.core_module.declarations.len(), 2);
        let Decl::Theorem {
            name, ty, proof, ..
        } = &ok.core_module.declarations[0]
        else {
            panic!("first declaration should be the by theorem");
        };
        assert_eq!(name, "Api.HumanByCore.id_prop");
        assert_eq!(
            ty,
            &Expr::pi(
                "P",
                Expr::sort(npa_kernel::Level::zero()),
                Expr::sort(npa_kernel::Level::zero())
            )
        );
        assert_eq!(
            proof,
            &Expr::lam("P", Expr::sort(npa_kernel::Level::zero()), Expr::bvar(0))
        );
        let Decl::Def { name, value, .. } = &ok.core_module.declarations[1] else {
            panic!("second declaration should use the by theorem");
        };
        assert_eq!(name, "Api.HumanByCore.use");
        assert_eq!(
            value,
            &Expr::lam(
                "P",
                Expr::sort(npa_kernel::Level::zero()),
                Expr::app(
                    Expr::konst("Api.HumanByCore.id_prop", Vec::new()),
                    Expr::bvar(0)
                )
            )
        );
        let cert = npa_cert::build_module_cert(ok.core_module, &[])
            .expect("by core module should certify");
        let bytes = npa_cert::encode_module_cert(&cert).expect("by core cert should encode");
        npa_cert::verify_module_cert(
            &bytes,
            &mut npa_cert::VerifierSession::new(),
            &npa_cert::AxiomPolicy::normal(),
        )
        .expect("by core cert should verify");
    }

    #[test]
    fn human_api_compile_certificate_verifies_by_proof_and_hashes_interface() {
        let ok = compile_human_source_to_certificate(HumanCompileCertificateRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanByCert"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source: "\
theorem id_type : forall (A : Type), Type := by
  intro A
  exact A",
            },
            verified_modules: &[],
            imported_source_interfaces: &[],
            options: human_api_default_compile_options(),
        })
        .expect("Human API certificate path should certify by proof theorem");

        assert!(ok
            .source_interface
            .declarations
            .iter()
            .all(|decl| decl.decl_interface_hash.is_some()));
        let bytes =
            npa_cert::encode_module_cert(&ok.certificate).expect("by proof cert should encode");
        let verified = npa_cert::verify_module_cert(
            &bytes,
            &mut npa_cert::VerifierSession::new(),
            &npa_cert::AxiomPolicy::normal(),
        )
        .expect("by proof certificate should verify");
        assert_eq!(
            verified.module(),
            &npa_cert::Name::from_dotted("Api.HumanByCert")
        );
    }

    #[test]
    fn human_api_by_intro_exact_nat_certificate_verifies() {
        let (nat, nat_interface) = verified_nat_human_import();
        let verified_modules = vec![nat];
        let imported_source_interfaces = vec![nat_interface];

        let ok = compile_human_source_to_certificate(HumanCompileCertificateRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanByIntroExactNat"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source: "\
import Std.Nat.Basic
theorem id_nat : forall (n : Nat), Nat := by
  intro n
  exact n",
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human API should certify by intro n / exact n");

        assert!(ok
            .source_interface
            .declarations
            .iter()
            .all(|decl| decl.decl_interface_hash.is_some()));
        let bytes = npa_cert::encode_module_cert(&ok.certificate)
            .expect("intro/exact by proof cert should encode");
        let mut session = npa_cert::VerifierSession::new();
        session.register_verified_module(verified_modules[0].clone());
        let verified =
            npa_cert::verify_module_cert(&bytes, &mut session, &npa_cert::AxiomPolicy::normal())
                .expect("intro/exact by proof cert should verify with Nat import");
        assert_eq!(
            verified.module(),
            &npa_cert::Name::from_dotted("Api.HumanByIntroExactNat")
        );
    }

    #[test]
    fn human_api_by_exact_eq_refl_uses_imported_source_interfaces() {
        let (nat, nat_interface) = verified_nat_human_import();
        let (eq, eq_interface) = verified_eq_human_import();
        let verified_modules = vec![nat, eq];
        let imported_source_interfaces = vec![nat_interface, eq_interface];

        let ok = compile_human_source_to_certificate(HumanCompileCertificateRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanByExactEqRefl"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source: "\
import Std.Nat.Basic
import Std.Logic.Eq
def n : Nat := Nat.zero
theorem self_eq : Eq.{1} n n := by
  exact Eq.refl n",
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human API should certify by exact Eq.refl n");

        assert!(ok
            .source_interface
            .declarations
            .iter()
            .all(|decl| decl.decl_interface_hash.is_some()));
        let bytes =
            npa_cert::encode_module_cert(&ok.certificate).expect("Eq.refl by cert should encode");
        let mut session = npa_cert::VerifierSession::new();
        for module in verified_modules {
            session.register_verified_module(module);
        }
        let verified =
            npa_cert::verify_module_cert(&bytes, &mut session, &npa_cert::AxiomPolicy::normal())
                .expect("Eq.refl by proof cert should verify with Nat and Eq imports");
        assert_eq!(
            verified.module(),
            &npa_cert::Name::from_dotted("Api.HumanByExactEqRefl")
        );
    }

    #[test]
    fn machine_tactic_human_section13_minimal_certificate_fixtures_compile() {
        let (nat, nat_interface) = verified_nat_human_import();
        let (eq, eq_interface) = verified_eq_human_import();
        let verified_modules = vec![nat, eq];
        let imported_source_interfaces = vec![nat_interface, eq_interface];
        let default_options = human_api_default_compile_options();

        assert_human_fixture_certificate_verifies(
            "Api.HumanTacticIntroExactFixture",
            "\
import Std.Nat.Basic
theorem id_nat : forall (n : Nat), Nat := by
  intro n
  exact n",
            &verified_modules,
            &imported_source_interfaces,
            default_options.clone(),
        );
        assert_human_fixture_certificate_verifies(
            "Api.HumanTacticEqReflFixture",
            "\
import Std.Nat.Basic
import Std.Logic.Eq
def n : Nat := Nat.zero
theorem self_eq : Eq.{1} n n := by
  exact Eq.refl n",
            &verified_modules,
            &imported_source_interfaces,
            default_options.clone(),
        );
        assert_human_fixture_certificate_verifies(
            "Api.HumanTacticApplyFixture",
            "\
theorem id_prop {q : Prop} (hq : q) : q := hq
theorem use_id (q : Prop) (hq : q) : q := by
  intro q
  intro hq
  apply id_prop
  exact hq",
            &[],
            &[],
            default_options.clone(),
        );
        assert_human_fixture_certificate_verifies(
            "Api.HumanTacticSimpLiteFixture",
            "\
import Std.Nat.Basic
import Std.Logic.Eq
theorem self_eq (n : Nat) : Eq.{1} n n := by
  intro n
  simp-lite",
            &verified_modules,
            &imported_source_interfaces,
            default_options,
        );
    }

    #[test]
    fn machine_tactic_human_section13_rw_and_induction_certificate_fixtures_compile() {
        let (nat, nat_interface) = verified_nat_human_import();
        let (eq, eq_interface) = verified_eq_human_import();
        let verified_modules = vec![nat, eq];
        let imported_source_interfaces = vec![nat_interface, eq_interface];

        assert_human_fixture_certificate_verifies(
            "Api.HumanTacticRwFixture",
            "\
import Std.Nat.Basic
import Std.Logic.Eq
theorem rw_local (a b : Nat) (h : Eq.{1} a b) : Eq.{1} a a := by
  intro a
  intro b
  intro h
  rw [h]
  exact Eq.refl b",
            &verified_modules,
            &imported_source_interfaces,
            human_api_default_compile_options(),
        );
        assert_human_fixture_certificate_verifies(
            "Api.HumanTacticPriorRwFixture",
            "\
import Std.Nat.Basic
import Std.Logic.Eq
theorem rw_local (a b : Nat) (h : Eq.{1} a b) : Eq.{1} a a := by
  intro a
  intro b
  intro h
  rw [h]
  exact Eq.refl b
theorem use_rw_local (a b : Nat) (h : Eq.{1} a b) : Eq.{1} a a := by
  intro a
  intro b
  intro h
  exact rw_local a b h",
            &verified_modules,
            &imported_source_interfaces,
            human_api_default_compile_options(),
        );
        assert_human_fixture_certificate_verifies(
            "Api.HumanTacticInductionFixture",
            "\
import Std.Nat.Basic
import Std.Logic.Eq
theorem ind_self (n : Nat) : Eq.{1} n n := by
  intro n
  induction n
  exact Eq.refl Nat.zero
  simp-lite",
            &verified_modules,
            &imported_source_interfaces,
            human_nat_compile_options(&verified_modules[0]),
        );
    }

    #[test]
    fn machine_tactic_human_section14_typeclass_driven_apply_is_rejected_by_diagnostic() {
        let source = "\
theorem no_typeclass_apply (p : Prop) : p := by
  intro p
  apply inferInstance";
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanTacticUnsupportedTypeclassApply"),
            theorem_name: npa_cert::Name::from_dotted(
                "Api.HumanTacticUnsupportedTypeclassApply.no_typeclass_apply",
            ),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source,
            },
            verified_modules: &[],
            imported_source_interfaces: &[],
            options: human_api_default_compile_options(),
        })
        .expect("unsupported typeclass apply fixture should still start proof state");
        let script = first_theorem_script(source);

        let err = run_human_tactic_script(HumanTacticScriptRunRequest {
            state: &started.state,
            script: &script,
            current_source_interface: &started.source_interface,
            imported_source_interfaces: &[],
            options: human_api_default_compile_options(),
            budget: npa_tactic::TacticBudget::default(),
        })
        .expect_err("typeclass-driven apply is outside Human tactic MVP");

        let HumanTacticScriptError::Human(HumanCompileError { diagnostic }) = err else {
            panic!("typeclass-driven apply should map to a Human diagnostic");
        };
        assert_eq!(
            diagnostic.kind,
            npa_frontend::HumanDiagnosticKind::UnknownIdentifier
        );
        assert!(diagnostic.message.contains("unknown apply head"));
    }

    #[test]
    fn human_api_compile_certificate_rejects_unresolved_by_goal_before_certificate() {
        let err = compile_human_source_to_certificate(HumanCompileCertificateRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanByOpenGoal"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source: "\
theorem open_goal : forall (A : Type), Type := by
  intro A",
            },
            verified_modules: &[],
            imported_source_interfaces: &[],
            options: human_api_default_compile_options(),
        })
        .expect_err("unresolved Human by proof goal should stop before certificate construction");

        assert_eq!(
            err.diagnostic.kind,
            npa_frontend::HumanDiagnosticKind::UnresolvedGoal
        );
        assert_eq!(
            err.diagnostic
                .payload
                .as_ref()
                .and_then(|payload| payload.phase),
            Some(npa_frontend::HumanDiagnosticPhase::TacticUnresolvedGoal)
        );
        let payload = human_payload(&err.diagnostic);
        assert_eq!(payload.hole_goals.len(), 1);
        assert_eq!(payload.hole_goals[0].context[0].name, "A");
        assert_eq!(payload.hole_goals[0].target.as_deref(), Some("Type"));
    }

    #[test]
    fn human_api_by_proof_certificate_uses_verified_imports() {
        let producer = compile_human_source_to_certificate(HumanCompileCertificateRequest {
            current_module: npa_cert::Name::from_dotted("Api.ByImportLib"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source: "\
axiom ImportedP : Prop
axiom imported_p : ImportedP",
            },
            verified_modules: &[],
            imported_source_interfaces: &[],
            options: human_api_default_compile_options(),
        })
        .expect("producer Human API request should compile");
        let bytes =
            npa_cert::encode_module_cert(&producer.certificate).expect("producer cert encodes");
        let verified = npa_cert::verify_module_cert(
            &bytes,
            &mut npa_cert::VerifierSession::new(),
            &npa_cert::AxiomPolicy::normal(),
        )
        .expect("producer cert verifies");
        let import = npa_frontend::VerifiedImport::from(&verified);
        let source_interface = npa_frontend::HumanImportedSourceInterface {
            module: import.module.clone(),
            export_hash: import.export_hash,
            certificate_hash: import.certificate_hash,
            source_interface: producer.source_interface,
        };

        let ok = compile_human_source_to_certificate(HumanCompileCertificateRequest {
            current_module: npa_cert::Name::from_dotted("Api.ByImportUser"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(1),
                source: "\
import Api.ByImportLib
theorem target : ImportedP := by
  exact imported_p",
            },
            verified_modules: std::slice::from_ref(&verified),
            imported_source_interfaces: &[source_interface],
            options: human_api_default_compile_options(),
        })
        .expect("by proof certificate path should use verified imports");
        let bytes = npa_cert::encode_module_cert(&ok.certificate).expect("consumer cert encodes");
        let mut session = npa_cert::VerifierSession::new();
        session.register_verified_module(verified);
        npa_cert::verify_module_cert(&bytes, &mut session, &npa_cert::AxiomPolicy::normal())
            .expect("consumer by proof cert verifies with import");
    }

    #[test]
    fn human_proof_bridge_starts_machine_state_for_by_theorem() {
        let ok = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanProof"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanProof.target"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source: "\
def choose {A : Type} (x y : A) : A := x
infixl:65 \" ++ \" => choose
def use (A : Type) (x : A) : A := x ++ x
theorem target : Prop := by simp-lite",
            },
            verified_modules: &[],
            imported_source_interfaces: &[],
            options: human_api_default_compile_options(),
        })
        .expect("Human bridge should start a deterministic Machine proof state");

        assert_eq!(
            ok.state.root.module,
            npa_cert::Name::from_dotted("Api.HumanProof")
        );
        assert_eq!(
            ok.state.root.theorem_name,
            npa_cert::Name::from_dotted("Api.HumanProof.target")
        );
        assert_eq!(ok.state.root.source_index, 2);
        assert_eq!(ok.state.env.checked_current_decls.len(), 2);
        assert_eq!(ok.state.open_goals.len(), 1);
        assert_eq!(
            ok.state.root.theorem_type,
            npa_kernel::Expr::sort(npa_kernel::Level::zero())
        );
        npa_tactic::validate_machine_proof_state(&ok.state)
            .expect("Human-started state must pass Machine state validation");
    }

    #[test]
    fn human_proof_bridge_uses_verified_imports_and_source_interfaces() {
        let producer = compile_human_source_to_certificate(HumanCompileCertificateRequest {
            current_module: npa_cert::Name::from_dotted("Api.ProofLib"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source: "axiom ImportedP : Prop",
            },
            verified_modules: &[],
            imported_source_interfaces: &[],
            options: human_api_default_compile_options(),
        })
        .expect("producer Human API request should compile");
        let bytes =
            npa_cert::encode_module_cert(&producer.certificate).expect("producer cert encodes");
        let verified = npa_cert::verify_module_cert(
            &bytes,
            &mut npa_cert::VerifierSession::new(),
            &npa_cert::AxiomPolicy::normal(),
        )
        .expect("producer cert verifies");
        let import = npa_frontend::VerifiedImport::from(&verified);
        let source_interface = npa_frontend::HumanImportedSourceInterface {
            module: import.module.clone(),
            export_hash: import.export_hash,
            certificate_hash: import.certificate_hash,
            source_interface: producer.source_interface,
        };

        let ok = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanImportProof"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanImportProof.target"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(1),
                source: "\
import Api.ProofLib
theorem target : ImportedP := by simp-lite",
            },
            verified_modules: &[verified],
            imported_source_interfaces: &[source_interface],
            options: human_api_default_compile_options(),
        })
        .expect("Human bridge should start a state with active verified imports");

        assert_eq!(ok.state.env.imports.len(), 1);
        assert_eq!(ok.state.root.source_index, 0);
        npa_tactic::validate_machine_proof_state(&ok.state)
            .expect("import-backed Human-started state must validate");
    }

    #[test]
    fn human_tactic_term_bridge_checks_goal_local_without_machine_hot_path_dependency() {
        let ok = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanTactic"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanTactic.target"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source: "theorem target : forall (A : Type), Type := by simp-lite",
            },
            verified_modules: &[],
            imported_source_interfaces: &[],
            options: human_api_default_compile_options(),
        })
        .expect("Human proof bridge should start a theorem with a Pi target");
        let (state, _) = npa_tactic::run_machine_tactic(
            &ok.state,
            npa_tactic::MachineTactic::Intro {
                goal_id: npa_tactic::GoalId(0),
                name: "A".to_owned(),
            },
        )
        .expect("Machine intro should create a local A goal");
        let term = npa_frontend::parse_human_term(npa_frontend::FileId(0), "A")
            .expect("Human tactic term should parse");
        let checked = check_human_tactic_term(HumanTacticTermCheckRequest {
            state: &state,
            goal_id: npa_tactic::GoalId(1),
            term: &term,
            current_source_interface: &ok.source_interface,
            imported_source_interfaces: &[],
            options: human_api_default_compile_options(),
        })
        .expect("Human tactic bridge should check exact local A");

        assert_eq!(checked.expr, npa_kernel::Expr::bvar(0));
        assert_eq!(
            checked.inferred_type,
            npa_kernel::Expr::sort(npa_kernel::type0())
        );
    }

    #[test]
    fn human_exact_closes_nat_identity_goal_with_local() {
        let (nat, nat_interface) = verified_nat_human_import();
        let verified_modules = vec![nat];
        let imported_source_interfaces = vec![nat_interface];
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanExactNat"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanExactNat.id_nat"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source: "\
import Std.Nat.Basic
theorem id_nat : forall (n : Nat), Nat := by simp-lite",
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human proof bridge should start id_nat");
        let (state, _) = npa_tactic::run_machine_tactic(
            &started.state,
            npa_tactic::MachineTactic::Intro {
                goal_id: npa_tactic::GoalId(0),
                name: "n".to_owned(),
            },
        )
        .expect("intro should expose the Nat local");
        let term = npa_frontend::parse_human_term(npa_frontend::FileId(0), "n")
            .expect("Human exact term should parse");

        let ok = run_human_exact_tactic(HumanExactTacticRequest {
            state: &state,
            goal_id: npa_tactic::GoalId(1),
            term: &term,
            current_source_interface: &started.source_interface,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human exact should check the local and close the goal");

        assert!(ok.state.open_goals.is_empty());
        assert!(ok.delta.added_goals.is_empty());
        assert_eq!(ok.expr, npa_kernel::Expr::bvar(0));
        assert_eq!(ok.inferred_type, npa_kernel::nat());
        let proof = npa_tactic::extract_closed_machine_proof(&ok.state)
            .expect("closed Human exact proof should extract");
        assert_eq!(
            proof,
            npa_kernel::Expr::lam("n", npa_kernel::nat(), npa_kernel::Expr::bvar(0))
        );
    }

    #[test]
    fn human_exact_inserts_eq_refl_implicit_and_closes_goal() {
        let (nat, nat_interface) = verified_nat_human_import();
        let (eq, eq_interface) = verified_eq_human_import();
        let verified_modules = vec![nat, eq];
        let imported_source_interfaces = vec![nat_interface, eq_interface];
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanExactEq"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanExactEq.self_eq"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source: "\
import Std.Nat.Basic
import Std.Logic.Eq
theorem self_eq (n : Nat) : Eq.{1} n n := by simp-lite",
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human proof bridge should start self_eq");
        let (state, _) = npa_tactic::run_machine_tactic(
            &started.state,
            npa_tactic::MachineTactic::Intro {
                goal_id: npa_tactic::GoalId(0),
                name: "n".to_owned(),
            },
        )
        .expect("intro should expose the Nat local");
        let term = npa_frontend::parse_human_term(npa_frontend::FileId(0), "Eq.refl n")
            .expect("Human exact term should parse");
        let expected = npa_kernel::eq(
            npa_kernel::type0(),
            npa_kernel::nat(),
            npa_kernel::Expr::bvar(0),
            npa_kernel::Expr::bvar(0),
        );

        let ok = run_human_exact_tactic(HumanExactTacticRequest {
            state: &state,
            goal_id: npa_tactic::GoalId(1),
            term: &term,
            current_source_interface: &started.source_interface,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human exact should elaborate Eq.refl n and close the goal");

        assert!(ok.state.open_goals.is_empty());
        assert_eq!(
            ok.expr,
            npa_kernel::eq_refl(
                npa_kernel::type0(),
                npa_kernel::nat(),
                npa_kernel::Expr::bvar(0)
            )
        );
        assert_eq!(ok.inferred_type, expected);
        let proof = npa_tactic::extract_closed_machine_proof(&ok.state)
            .expect("closed Human exact proof should extract");
        assert_eq!(
            proof,
            npa_kernel::Expr::lam(
                "n",
                npa_kernel::nat(),
                npa_kernel::eq_refl(
                    npa_kernel::type0(),
                    npa_kernel::nat(),
                    npa_kernel::Expr::bvar(0)
                )
            )
        );
    }

    #[test]
    fn human_exact_rejects_unresolved_hole_without_mutating_state() {
        let (nat, nat_interface) = verified_nat_human_import();
        let verified_modules = vec![nat];
        let imported_source_interfaces = vec![nat_interface];
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanExactHole"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanExactHole.id_nat"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source: "\
import Std.Nat.Basic
theorem id_nat : forall (n : Nat), Nat := by simp-lite",
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human proof bridge should start id_nat");
        let (state, _) = npa_tactic::run_machine_tactic(
            &started.state,
            npa_tactic::MachineTactic::Intro {
                goal_id: npa_tactic::GoalId(0),
                name: "n".to_owned(),
            },
        )
        .expect("intro should expose the Nat local");
        let term = npa_frontend::parse_human_term(npa_frontend::FileId(0), "_")
            .expect("Human hole should parse");

        let err = run_human_exact_tactic(HumanExactTacticRequest {
            state: &state,
            goal_id: npa_tactic::GoalId(1),
            term: &term,
            current_source_interface: &started.source_interface,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect_err("Human exact must reject unresolved holes conservatively");

        assert!(matches!(
            err,
            HumanTacticTermError::Human(HumanCompileError {
                diagnostic: npa_frontend::HumanDiagnostic {
                    kind: npa_frontend::HumanDiagnosticKind::UnsolvedHole,
                    ..
                }
            })
        ));
        assert_eq!(state.open_goals, vec![npa_tactic::GoalId(1)]);
        assert!(
            npa_tactic::extract_closed_machine_proof(&state).is_err(),
            "rejected Human exact must leave the original goal open"
        );
    }

    #[test]
    fn human_exact_type_mismatch_returns_goal_payload() {
        let (nat, nat_interface) = verified_nat_human_import();
        let verified_modules = vec![nat];
        let imported_source_interfaces = vec![nat_interface];
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanExactMismatch"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanExactMismatch.target"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source: "\
import Std.Nat.Basic
theorem target : Nat := by simp-lite",
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human proof bridge should start Nat target");
        let original_fingerprint = started.state.fingerprint;
        let term = npa_frontend::parse_human_term(npa_frontend::FileId(0), "Prop")
            .expect("Prop should parse as a Human term");

        let err = run_human_exact_tactic(HumanExactTacticRequest {
            state: &started.state,
            goal_id: npa_tactic::GoalId(0),
            term: &term,
            current_source_interface: &started.source_interface,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect_err("exact Prop should not prove Nat");

        let HumanTacticTermError::Human(HumanCompileError { diagnostic }) = err else {
            panic!("exact mismatch should map to a Human diagnostic");
        };
        assert_eq!(
            diagnostic.kind,
            npa_frontend::HumanDiagnosticKind::TypeMismatch
        );
        let payload = human_payload(&diagnostic);
        assert_eq!(
            payload.phase,
            Some(npa_frontend::HumanDiagnosticPhase::TacticValidation)
        );
        assert_eq!(payload.hole_goals.len(), 1);
        assert_eq!(payload.hole_goals[0].target.as_deref(), Some("Nat"));
        assert_eq!(started.state.fingerprint, original_fingerprint);
    }

    #[test]
    fn human_intro_creates_nat_body_goal_via_machine_intro() {
        let (nat, nat_interface) = verified_nat_human_import();
        let verified_modules = vec![nat];
        let imported_source_interfaces = vec![nat_interface];
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanIntroNat"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanIntroNat.id_nat"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source: "\
import Std.Nat.Basic
theorem id_nat : forall (n : Nat), Nat := by simp-lite",
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human proof bridge should start id_nat");
        let name = human_name("n", 0, 1);
        let budget = npa_tactic::TacticBudget::default();

        let human = run_human_intro_tactic(HumanIntroTacticRequest {
            state: &started.state,
            goal_id: npa_tactic::GoalId(0),
            name: &name,
            budget,
        })
        .expect("Human intro should create a Nat body goal");
        let direct_machine = npa_tactic::run_machine_tactic_with_budget(
            &started.state,
            npa_tactic::MachineTactic::Intro {
                goal_id: npa_tactic::GoalId(0),
                name: "n".to_owned(),
            },
            budget,
        )
        .expect("direct Machine intro should match Human intro");

        assert_eq!(human.delta, direct_machine.1);
        assert_eq!(human.state.fingerprint, direct_machine.0.fingerprint);
        assert_eq!(human.state.open_goals, vec![npa_tactic::GoalId(1)]);
        let goal = human
            .state
            .goal(npa_tactic::GoalId(1))
            .expect("intro should create goal 1");
        assert_eq!(goal.context.len(), 1);
        assert_eq!(goal.context[0].name, "n");
        assert_eq!(goal.context[0].ty, npa_kernel::nat());
        assert_eq!(goal.target, npa_kernel::nat());
    }

    #[test]
    fn human_intro_non_pi_returns_human_expected_function_diagnostic() {
        let (nat, nat_interface) = verified_nat_human_import();
        let verified_modules = vec![nat];
        let imported_source_interfaces = vec![nat_interface];
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanIntroNonPi"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanIntroNonPi.target"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source: "\
import Std.Nat.Basic
theorem target : Nat := by simp-lite",
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human proof bridge should start a non-Pi theorem");
        let name = human_name("n", 0, 1);
        let budget = npa_tactic::TacticBudget::default();
        let intro_tactic = npa_tactic::MachineTactic::Intro {
            goal_id: npa_tactic::GoalId(0),
            name: "n".to_owned(),
        };
        let cache_key_before = npa_tactic::machine_tactic_cache_key_hash(
            &npa_tactic::machine_tactic_cache_key(&started.state, &intro_tactic, budget),
        );

        let err = run_human_intro_tactic(HumanIntroTacticRequest {
            state: &started.state,
            goal_id: npa_tactic::GoalId(0),
            name: &name,
            budget,
        })
        .expect_err("intro should reject non-Pi targets as a Human diagnostic");

        let HumanIntroTacticError::Human(HumanCompileError { diagnostic }) = err else {
            panic!("non-Pi intro should map to a Human diagnostic");
        };
        assert_eq!(
            diagnostic.kind,
            npa_frontend::HumanDiagnosticKind::ExpectedFunctionType
        );
        assert_eq!(
            diagnostic
                .payload
                .as_ref()
                .and_then(|payload| payload.phase),
            Some(npa_frontend::HumanDiagnosticPhase::TacticExecution)
        );
        let payload = human_payload(&diagnostic);
        assert_eq!(payload.hole_goals.len(), 1);
        assert_eq!(payload.hole_goals[0].hole.as_deref(), Some("g0"));
        assert!(payload.hole_goals[0].context.is_empty());
        assert_eq!(payload.hole_goals[0].target.as_deref(), Some("Nat"));
        let cache_key_after = npa_tactic::machine_tactic_cache_key_hash(
            &npa_tactic::machine_tactic_cache_key(&started.state, &intro_tactic, budget),
        );
        assert_eq!(cache_key_after, cache_key_before);
        assert_eq!(started.state.open_goals, vec![npa_tactic::GoalId(0)]);
    }

    #[test]
    fn human_intro_rejects_shadowing_name_deterministically() {
        let (nat, nat_interface) = verified_nat_human_import();
        let verified_modules = vec![nat];
        let imported_source_interfaces = vec![nat_interface];
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanIntroShadow"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanIntroShadow.target"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source: "\
import Std.Nat.Basic
theorem target : forall (n : Nat), forall (m : Nat), Nat := by simp-lite",
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human proof bridge should start a two-argument theorem");
        let name = human_name("n", 0, 1);
        let first = run_human_intro_tactic(HumanIntroTacticRequest {
            state: &started.state,
            goal_id: npa_tactic::GoalId(0),
            name: &name,
            budget: npa_tactic::TacticBudget::default(),
        })
        .expect("first intro should succeed");

        let err = run_human_intro_tactic(HumanIntroTacticRequest {
            state: &first.state,
            goal_id: npa_tactic::GoalId(1),
            name: &name,
            budget: npa_tactic::TacticBudget::default(),
        })
        .expect_err("second intro should reject local shadowing deterministically");

        let HumanIntroTacticError::Human(HumanCompileError { diagnostic }) = err else {
            panic!("intro shadowing should map to a Human diagnostic");
        };
        assert_eq!(
            diagnostic.kind,
            npa_frontend::HumanDiagnosticKind::UnsupportedTactic
        );
        assert_eq!(first.state.open_goals, vec![npa_tactic::GoalId(1)]);
    }

    #[test]
    fn human_intro_rejects_invalid_binder_name_deterministically() {
        let (nat, nat_interface) = verified_nat_human_import();
        let verified_modules = vec![nat];
        let imported_source_interfaces = vec![nat_interface];
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanIntroInvalidName"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanIntroInvalidName.id_nat"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source: "\
import Std.Nat.Basic
theorem id_nat : forall (n : Nat), Nat := by simp-lite",
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human proof bridge should start id_nat");
        let name = human_name_parts(&["Nat", "x"], 0, 5);

        let err = run_human_intro_tactic(HumanIntroTacticRequest {
            state: &started.state,
            goal_id: npa_tactic::GoalId(0),
            name: &name,
            budget: npa_tactic::TacticBudget::default(),
        })
        .expect_err("intro should reject non-local binder names deterministically");

        let HumanIntroTacticError::Human(HumanCompileError { diagnostic }) = err else {
            panic!("invalid intro binder should map to a Human diagnostic");
        };
        assert_eq!(
            diagnostic.kind,
            npa_frontend::HumanDiagnosticKind::UnsupportedTactic
        );
        assert_eq!(started.state.open_goals, vec![npa_tactic::GoalId(0)]);
    }

    #[test]
    fn human_intro_then_exact_closes_id_nat() {
        let (nat, nat_interface) = verified_nat_human_import();
        let verified_modules = vec![nat];
        let imported_source_interfaces = vec![nat_interface];
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanIntroExact"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanIntroExact.id_nat"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source: "\
import Std.Nat.Basic
theorem id_nat : forall (n : Nat), Nat := by simp-lite",
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human proof bridge should start id_nat");
        let name = human_name("n", 0, 1);
        let intro = run_human_intro_tactic(HumanIntroTacticRequest {
            state: &started.state,
            goal_id: npa_tactic::GoalId(0),
            name: &name,
            budget: npa_tactic::TacticBudget::default(),
        })
        .expect("Human intro should create the body goal");
        let term = npa_frontend::parse_human_term(npa_frontend::FileId(0), "n")
            .expect("Human exact local should parse");
        let exact = run_human_exact_tactic(HumanExactTacticRequest {
            state: &intro.state,
            goal_id: npa_tactic::GoalId(1),
            term: &term,
            current_source_interface: &started.source_interface,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human exact should close the body goal after intro");

        assert!(exact.state.open_goals.is_empty());
        let proof = npa_tactic::extract_closed_machine_proof(&exact.state)
            .expect("intro + exact proof should extract");
        assert_eq!(
            proof,
            npa_kernel::Expr::lam("n", npa_kernel::nat(), npa_kernel::Expr::bvar(0))
        );
    }

    #[test]
    fn human_tactic_script_executor_closes_intro_exact_script() {
        let (nat, nat_interface) = verified_nat_human_import();
        let verified_modules = vec![nat];
        let imported_source_interfaces = vec![nat_interface];
        let source = "\
import Std.Nat.Basic
theorem id_nat : forall (n : Nat), Nat := by
  intro n
  exact n";
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanScriptIntroExact"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanScriptIntroExact.id_nat"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source,
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human proof bridge should start id_nat");
        let script = first_theorem_script(source);

        let ok = run_human_tactic_script(HumanTacticScriptRunRequest {
            state: &started.state,
            script: &script,
            current_source_interface: &started.source_interface,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
            budget: npa_tactic::TacticBudget::default(),
        })
        .expect("Human script executor should close intro + exact");

        assert_eq!(ok.deltas.len(), 2);
        assert!(ok.state.open_goals.is_empty());
        assert_eq!(
            ok.proof,
            npa_kernel::Expr::lam("n", npa_kernel::nat(), npa_kernel::Expr::bvar(0))
        );
        npa_tactic::extract_closed_machine_proof(&ok.state)
            .expect("extracted script proof should pass kernel check");
    }

    #[test]
    fn human_tactic_script_executor_rejects_extra_tactic_after_close() {
        let (nat, nat_interface) = verified_nat_human_import();
        let verified_modules = vec![nat];
        let imported_source_interfaces = vec![nat_interface];
        let source = "\
import Std.Nat.Basic
theorem zero : Nat := by
  exact Nat.zero
  exact Nat.zero";
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanScriptExtra"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanScriptExtra.zero"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source,
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human proof bridge should start zero");
        let script = first_theorem_script(source);

        let err = run_human_tactic_script(HumanTacticScriptRunRequest {
            state: &started.state,
            script: &script,
            current_source_interface: &started.source_interface,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
            budget: npa_tactic::TacticBudget::default(),
        })
        .expect_err("extra tactic after closed goal should be rejected");

        let HumanTacticScriptError::Human(HumanCompileError { diagnostic }) = err else {
            panic!("extra tactic should map to a Human diagnostic");
        };
        assert_eq!(
            diagnostic.kind,
            npa_frontend::HumanDiagnosticKind::NoGoalsButTacticRemaining
        );
        assert_eq!(
            human_payload(&diagnostic).phase,
            Some(npa_frontend::HumanDiagnosticPhase::TacticExecution)
        );
        assert_eq!(started.state.open_goals, vec![npa_tactic::GoalId(0)]);
    }

    #[test]
    fn human_tactic_script_executor_applies_tactic_to_first_open_goal() {
        let verified_modules = Vec::new();
        let imported_source_interfaces = Vec::new();
        let source = "\
theorem target : Prop := by
  exact fun (p : Prop) => p";
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanScriptFirstGoal"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanScriptFirstGoal.target"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source,
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human proof bridge should start target");
        let budget = npa_tactic::TacticBudget::default();
        let (state, _) = npa_tactic::assign_goal(
            &started.state,
            npa_tactic::GoalId(0),
            npa_tactic::ProofExpr::app(
                npa_tactic::ProofExpr::meta(npa_tactic::MetaVarId(1)),
                npa_tactic::ProofExpr::meta(npa_tactic::MetaVarId(2)),
            ),
            vec![
                npa_tactic::MachineNewGoalSpec::new(
                    Vec::new(),
                    npa_kernel::Expr::pi(
                        "p",
                        npa_kernel::Expr::sort(npa_kernel::prop()),
                        npa_kernel::Expr::sort(npa_kernel::prop()),
                    ),
                ),
                npa_tactic::MachineNewGoalSpec::new(
                    Vec::new(),
                    npa_kernel::Expr::sort(npa_kernel::prop()),
                ),
            ],
        )
        .expect("Machine setup should split root into two goals");
        assert_eq!(
            state.open_goals,
            vec![npa_tactic::GoalId(1), npa_tactic::GoalId(2)]
        );
        let script = first_theorem_script(source);

        let err = run_human_tactic_script(HumanTacticScriptRunRequest {
            state: &state,
            script: &script,
            current_source_interface: &started.source_interface,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
            budget,
        })
        .expect_err("one exact should close the first goal and leave the step goal open");

        let HumanTacticScriptError::Human(HumanCompileError { diagnostic }) = err else {
            panic!("remaining second goal should be a Human diagnostic");
        };
        assert_eq!(
            diagnostic.kind,
            npa_frontend::HumanDiagnosticKind::UnresolvedGoal
        );
        let payload = human_payload(&diagnostic);
        assert_eq!(
            payload.phase,
            Some(npa_frontend::HumanDiagnosticPhase::TacticUnresolvedGoal)
        );
        assert_eq!(payload.hole_goals.len(), 1);
        assert_eq!(payload.hole_goals[0].hole.as_deref(), Some("g2"));
        assert_eq!(payload.hole_goals[0].target.as_deref(), Some("Prop"));
    }

    #[test]
    fn human_tactic_script_executor_reports_unresolved_goal_at_end() {
        let (nat, nat_interface) = verified_nat_human_import();
        let verified_modules = vec![nat];
        let imported_source_interfaces = vec![nat_interface];
        let source = "\
import Std.Nat.Basic
theorem id_nat : forall (n : Nat), Nat := by
  intro n";
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanScriptUnresolved"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanScriptUnresolved.id_nat"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source,
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human proof bridge should start id_nat");
        let script = first_theorem_script(source);

        let err = run_human_tactic_script(HumanTacticScriptRunRequest {
            state: &started.state,
            script: &script,
            current_source_interface: &started.source_interface,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
            budget: npa_tactic::TacticBudget::default(),
        })
        .expect_err("script with remaining body goal should be rejected");

        let HumanTacticScriptError::Human(HumanCompileError { diagnostic }) = err else {
            panic!("unresolved goal should map to a Human diagnostic");
        };
        assert_eq!(
            diagnostic.kind,
            npa_frontend::HumanDiagnosticKind::UnresolvedGoal
        );
        assert_ne!(
            diagnostic.kind,
            npa_frontend::HumanDiagnosticKind::NoGoalsButTacticRemaining
        );
        let payload = human_payload(&diagnostic);
        assert_eq!(
            payload.phase,
            Some(npa_frontend::HumanDiagnosticPhase::TacticUnresolvedGoal)
        );
        assert_eq!(payload.hole_goals.len(), 1);
        assert_eq!(payload.hole_goals[0].context[0].name, "n");
        assert_eq!(payload.hole_goals[0].context[0].ty, "Nat");
        assert_eq!(payload.hole_goals[0].target.as_deref(), Some("Nat"));
        assert_eq!(started.state.open_goals, vec![npa_tactic::GoalId(0)]);
    }

    #[test]
    fn human_apply_local_assumption_creates_subgoal_closed_by_exact() {
        let verified_modules = Vec::new();
        let imported_source_interfaces = Vec::new();
        let source = "\
theorem use_local (p q : Prop) (h : forall (hp : p), q) (hp : p) : q := by
  intro p
  intro q
  intro h
  intro hp
  apply h
  exact hp";
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanApplyLocal"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanApplyLocal.use_local"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source,
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human proof bridge should start use_local");
        let script = first_theorem_script(source);

        let ok = run_human_tactic_script(HumanTacticScriptRunRequest {
            state: &started.state,
            script: &script,
            current_source_interface: &started.source_interface,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
            budget: npa_tactic::TacticBudget::default(),
        })
        .expect("Human apply local script should close");

        assert!(ok.state.open_goals.is_empty());
        assert_eq!(ok.deltas.len(), 6);
        assert_eq!(ok.deltas[4].added_goals.len(), 1);
        npa_tactic::extract_closed_machine_proof(&ok.state)
            .expect("Human apply local proof should pass kernel check");
    }

    #[test]
    fn human_apply_checked_current_theorem_creates_subgoal_closed_by_exact() {
        let verified_modules = Vec::new();
        let imported_source_interfaces = Vec::new();
        let source = "\
theorem id_prop {q : Prop} (hq : q) : q := hq
theorem use_id (q : Prop) (hq : q) : q := by
  intro q
  intro hq
  apply id_prop
  exact hq";
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanApplyCurrent"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanApplyCurrent.use_id"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source,
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human proof bridge should start use_id with checked current id_prop");
        let script = first_theorem_script(source);

        let ok = run_human_tactic_script(HumanTacticScriptRunRequest {
            state: &started.state,
            script: &script,
            current_source_interface: &started.source_interface,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
            budget: npa_tactic::TacticBudget::default(),
        })
        .expect("Human apply checked current script should close");

        assert!(ok.state.open_goals.is_empty());
        assert_eq!(ok.deltas.len(), 4);
        assert_eq!(ok.deltas[2].added_goals.len(), 1);
        npa_tactic::extract_closed_machine_proof(&ok.state)
            .expect("Human apply checked current proof should pass kernel check");
    }

    #[test]
    fn human_apply_mismatch_reports_target_and_head_type() {
        let verified_modules = Vec::new();
        let imported_source_interfaces = Vec::new();
        let source = "\
theorem bad_apply (p q : Prop) (hp : p) : q := by
  intro p
  intro q
  intro hp
  apply hp";
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanApplyMismatch"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanApplyMismatch.bad_apply"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source,
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human proof bridge should start bad_apply");
        let script = first_theorem_script(source);
        let original_fingerprint = started.state.fingerprint;

        let err = run_human_tactic_script(HumanTacticScriptRunRequest {
            state: &started.state,
            script: &script,
            current_source_interface: &started.source_interface,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
            budget: npa_tactic::TacticBudget::default(),
        })
        .expect_err("Human apply mismatch should be a Human diagnostic");

        let HumanTacticScriptError::Human(HumanCompileError { diagnostic }) = err else {
            panic!("apply mismatch should map to a Human diagnostic");
        };
        assert_eq!(
            diagnostic.kind,
            npa_frontend::HumanDiagnosticKind::TypeMismatch
        );
        assert!(diagnostic.message.contains("cannot apply `hp`"));
        assert!(diagnostic.message.contains("target:"));
        assert!(diagnostic.message.contains("head type:"));
        let payload = human_payload(&diagnostic);
        assert_eq!(
            payload.phase,
            Some(npa_frontend::HumanDiagnosticPhase::TacticExecution)
        );
        assert_eq!(payload.hole_goals.len(), 1);
        assert_eq!(
            payload.hole_goals[0]
                .context
                .iter()
                .map(|local| local.name.as_str())
                .collect::<Vec<_>>(),
            vec!["p", "q", "hp"]
        );
        assert_eq!(payload.hole_goals[0].target.as_deref(), Some("q"));
        assert_eq!(started.state.fingerprint, original_fingerprint);
    }

    #[test]
    fn human_rw_local_forward_rewrites_eq_sides_and_exact_closes() {
        let (nat, nat_interface) = verified_nat_human_import();
        let (eq, eq_interface) = verified_eq_human_import();
        let verified_modules = vec![nat, eq];
        let imported_source_interfaces = vec![nat_interface, eq_interface];
        let source = "\
import Std.Nat.Basic
import Std.Logic.Eq
theorem rw_local (a b : Nat) (h : Eq.{1} a b) : Eq.{1} a a := by
  intro a
  intro b
  intro h
  rw [h]
  exact Eq.refl b";
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanRewriteLocal"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanRewriteLocal.rw_local"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source,
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human proof bridge should start rw_local");
        let script = first_theorem_script(source);

        let ok = run_human_tactic_script(HumanTacticScriptRunRequest {
            state: &started.state,
            script: &script,
            current_source_interface: &started.source_interface,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
            budget: npa_tactic::TacticBudget::default(),
        })
        .expect("Human rw local script should close");

        assert!(ok.state.open_goals.is_empty());
        assert_eq!(ok.deltas.len(), 6);
        assert_eq!(ok.deltas[3].added_goals.len(), 1);
        assert_eq!(ok.deltas[4].added_goals.len(), 1);
        npa_tactic::extract_closed_machine_proof(&ok.state)
            .expect("Human rw local proof should pass kernel check");
    }

    #[test]
    fn human_rw_local_backward_rewrites_deterministically() {
        let (nat, nat_interface) = verified_nat_human_import();
        let (eq, eq_interface) = verified_eq_human_import();
        let verified_modules = vec![nat, eq];
        let imported_source_interfaces = vec![nat_interface, eq_interface];
        let source = "\
import Std.Nat.Basic
import Std.Logic.Eq
theorem rw_backward (a b : Nat) (h : Eq.{1} a b) : Eq.{1} b b := by
  intro a
  intro b
  intro h
  rw [<- h]
  exact Eq.refl a";
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanRewriteBackward"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanRewriteBackward.rw_backward"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source,
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human proof bridge should start rw_backward");
        let script = first_theorem_script(source);

        let ok = run_human_tactic_script(HumanTacticScriptRunRequest {
            state: &started.state,
            script: &script,
            current_source_interface: &started.source_interface,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
            budget: npa_tactic::TacticBudget::default(),
        })
        .expect("Human reverse rw local script should close");

        assert!(ok.state.open_goals.is_empty());
        assert_eq!(ok.deltas.len(), 6);
        npa_tactic::extract_closed_machine_proof(&ok.state)
            .expect("Human reverse rw proof should pass kernel check");
    }

    #[test]
    fn human_rw_checked_current_theorem_rule_runs_through_machine_rewrite() {
        let (nat, nat_interface) = verified_nat_human_import();
        let (eq, eq_interface) = verified_eq_human_import();
        let verified_modules = vec![nat, eq];
        let imported_source_interfaces = vec![nat_interface, eq_interface];
        let source = "\
import Std.Nat.Basic
import Std.Logic.Eq
theorem refl_rule (x : Nat) : Eq.{1} x x := Eq.refl x
theorem use_refl_rule (a : Nat) : Eq.{1} a a := by
  intro a
  rw [refl_rule]
  exact Eq.refl a";
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanRewriteCurrent"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanRewriteCurrent.use_refl_rule"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source,
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human proof bridge should start use_refl_rule");
        let script = first_theorem_script(source);

        let ok = run_human_tactic_script(HumanTacticScriptRunRequest {
            state: &started.state,
            script: &script,
            current_source_interface: &started.source_interface,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
            budget: npa_tactic::TacticBudget::default(),
        })
        .expect("Human rw checked current theorem script should close");

        assert!(ok.state.open_goals.is_empty());
        assert_eq!(ok.deltas.len(), 4);
        npa_tactic::extract_closed_machine_proof(&ok.state)
            .expect("Human rw checked current proof should pass kernel check");
    }

    #[test]
    fn human_rw_rejects_complex_rule_head_as_unsupported() {
        let (nat, nat_interface) = verified_nat_human_import();
        let (eq, eq_interface) = verified_eq_human_import();
        let verified_modules = vec![nat, eq];
        let imported_source_interfaces = vec![nat_interface, eq_interface];
        let source = "\
import Std.Nat.Basic
import Std.Logic.Eq
theorem bad_rw (a : Nat) : Eq.{1} a a := by
  intro a
  rw [Eq.refl a]";
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanRewriteUnsupported"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanRewriteUnsupported.bad_rw"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source,
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human proof bridge should start bad_rw");
        let script = first_theorem_script(source);

        let err = run_human_tactic_script(HumanTacticScriptRunRequest {
            state: &started.state,
            script: &script,
            current_source_interface: &started.source_interface,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
            budget: npa_tactic::TacticBudget::default(),
        })
        .expect_err("complex rw rule head should be rejected");

        let HumanTacticScriptError::Human(HumanCompileError { diagnostic }) = err else {
            panic!("complex rw rule should map to a Human diagnostic");
        };
        assert_eq!(
            diagnostic.kind,
            npa_frontend::HumanDiagnosticKind::UnsupportedTactic
        );
        assert!(diagnostic.message.contains("Human rw MVP"));
    }

    #[test]
    fn human_rw_conditional_rule_fails_as_human_diagnostic() {
        let (eq, eq_interface) = verified_eq_human_import();
        let verified_modules = vec![eq];
        let imported_source_interfaces = vec![eq_interface];
        let source = "\
import Std.Logic.Eq
theorem bad_conditional_rw (p q : Prop) (h : forall (hp : p), Eq.{1} p q) : Eq.{1} p p := by
  intro p
  intro q
  intro h
  rw [h]";
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanRewriteConditional"),
            theorem_name: npa_cert::Name::from_dotted(
                "Api.HumanRewriteConditional.bad_conditional_rw",
            ),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source,
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human proof bridge should start bad_conditional_rw");
        let script = first_theorem_script(source);

        let err = run_human_tactic_script(HumanTacticScriptRunRequest {
            state: &started.state,
            script: &script,
            current_source_interface: &started.source_interface,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
            budget: npa_tactic::TacticBudget::default(),
        })
        .expect_err("conditional rw should fail deterministically");

        let HumanTacticScriptError::Human(HumanCompileError { diagnostic }) = err else {
            panic!("conditional rw should map to a Human diagnostic");
        };
        assert_eq!(
            diagnostic.kind,
            npa_frontend::HumanDiagnosticKind::TypeMismatch
        );
        assert!(diagnostic.message.contains("cannot rewrite with `h`"));
        assert!(diagnostic.message.contains("rule type:"));
    }

    #[test]
    fn human_simp_lite_closes_reflexive_eq_target() {
        let (nat, nat_interface) = verified_nat_human_import();
        let (eq, eq_interface) = verified_eq_human_import();
        let verified_modules = vec![nat, eq];
        let imported_source_interfaces = vec![nat_interface, eq_interface];
        let source = "\
import Std.Nat.Basic
import Std.Logic.Eq
theorem self_eq (n : Nat) : Eq.{1} n n := by
  intro n
  simp-lite";
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanSimpRefl"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanSimpRefl.self_eq"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source,
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
        })
        .expect("Human proof bridge should start self_eq");
        let script = first_theorem_script(source);

        let ok = run_human_tactic_script(HumanTacticScriptRunRequest {
            state: &started.state,
            script: &script,
            current_source_interface: &started.source_interface,
            imported_source_interfaces: &imported_source_interfaces,
            options: human_api_default_compile_options(),
            budget: npa_tactic::TacticBudget::default(),
        })
        .expect("Human simp-lite should close reflexive Eq target");

        assert!(ok.state.open_goals.is_empty());
        assert_eq!(ok.deltas.len(), 2);
        assert!(ok.deltas[1].added_goals.is_empty());
        npa_tactic::extract_closed_machine_proof(&ok.state)
            .expect("Human simp-lite proof should pass kernel check");
    }

    #[test]
    fn human_simp_lite_uses_registered_rule_and_closes() {
        let import = npa_tactic::VerifiedImportRef::from_verified_module(
            &verified_axiom_simp_close_module(),
        )
        .expect("simp close module should become a tactic import");
        let rule_hash = export_interface_hash(&import, "Lib.succ_zero");
        let state = npa_tactic::start_machine_proof(
            human_simp_machine_spec(eq_nat(nat_succ(nat_zero()), nat_zero())),
            vec![import],
            Vec::new(),
            npa_tactic::MachineTacticOptions {
                simp_rules: vec![npa_tactic::SimpRuleRef {
                    name: npa_cert::Name::from_dotted("Lib.succ_zero"),
                    decl_interface_hash: rule_hash,
                    direction: npa_tactic::RewriteDirection::Forward,
                }],
                ..npa_tactic::MachineTacticOptions::default()
            },
        )
        .expect("Machine proof with registered simp rule should start");

        let ok = run_human_simp_lite_tactic(HumanSimpLiteTacticRequest {
            state: &state,
            goal_id: npa_tactic::GoalId(0),
            span: npa_frontend::Span::empty(npa_frontend::FileId(0)),
            budget: npa_tactic::TacticBudget::default(),
        })
        .expect("Human simp-lite should reuse the registered Machine simp rule");

        assert!(ok.state.open_goals.is_empty());
        assert!(ok.delta.added_goals.is_empty());
        npa_tactic::extract_closed_machine_proof(&ok.state)
            .expect("registered-rule simp-lite proof should pass kernel check");
    }

    #[test]
    fn human_simp_lite_rejects_residual_goal_after_progress() {
        let import =
            npa_tactic::VerifiedImportRef::from_verified_module(&verified_one_unfold_simp_module())
                .expect("one_unfold module should become a tactic import");
        let rule_hash = export_interface_hash(&import, "Lib.one_unfold");
        let state = npa_tactic::start_machine_proof(
            human_simp_machine_spec(eq_nat(
                npa_kernel::Expr::konst("Lib.one", Vec::new()),
                nat_zero(),
            )),
            vec![import],
            Vec::new(),
            npa_tactic::MachineTacticOptions {
                simp_rules: vec![npa_tactic::SimpRuleRef {
                    name: npa_cert::Name::from_dotted("Lib.one_unfold"),
                    decl_interface_hash: rule_hash,
                    direction: npa_tactic::RewriteDirection::Forward,
                }],
                ..npa_tactic::MachineTacticOptions::default()
            },
        )
        .expect("Machine proof with registered simp rule should start");

        let err = run_human_simp_lite_tactic(HumanSimpLiteTacticRequest {
            state: &state,
            goal_id: npa_tactic::GoalId(0),
            span: npa_frontend::Span::empty(npa_frontend::FileId(0)),
            budget: npa_tactic::TacticBudget::default(),
        })
        .expect_err("Human simp-lite MVP should reject residual goals");

        let HumanSimpLiteTacticError::Human(HumanCompileError { diagnostic }) = err else {
            panic!("residual simp-lite goal should map to a Human diagnostic");
        };
        assert_eq!(
            diagnostic.kind,
            npa_frontend::HumanDiagnosticKind::TypeMismatch
        );
        assert!(diagnostic.message.contains("did not close"));
        assert_eq!(state.open_goals, vec![npa_tactic::GoalId(0)]);
    }

    #[test]
    fn human_simp_lite_preserves_machine_step_limit_failure() {
        let import = npa_tactic::VerifiedImportRef::from_verified_module(
            &verified_axiom_simp_chain_module(),
        )
        .expect("simp chain module should become a tactic import");
        let first_rule_hash = export_interface_hash(&import, "Lib.a_two_one");
        let second_rule_hash = export_interface_hash(&import, "Lib.b_one_zero");
        let state = npa_tactic::start_machine_proof(
            human_simp_machine_spec(eq_nat(nat_succ(nat_succ(nat_zero())), nat_zero())),
            vec![import],
            Vec::new(),
            npa_tactic::MachineTacticOptions {
                simp_rules: vec![
                    npa_tactic::SimpRuleRef {
                        name: npa_cert::Name::from_dotted("Lib.a_two_one"),
                        decl_interface_hash: first_rule_hash,
                        direction: npa_tactic::RewriteDirection::Forward,
                    },
                    npa_tactic::SimpRuleRef {
                        name: npa_cert::Name::from_dotted("Lib.b_one_zero"),
                        decl_interface_hash: second_rule_hash,
                        direction: npa_tactic::RewriteDirection::Forward,
                    },
                ],
                max_simp_rewrite_steps: 1,
                ..npa_tactic::MachineTacticOptions::default()
            },
        )
        .expect("Machine proof with registered simp rule should start");

        let err = run_human_simp_lite_tactic(HumanSimpLiteTacticRequest {
            state: &state,
            goal_id: npa_tactic::GoalId(0),
            span: npa_frontend::Span::empty(npa_frontend::FileId(0)),
            budget: npa_tactic::TacticBudget::default(),
        })
        .expect_err("Human simp-lite should preserve Machine step-limit failures");

        let HumanSimpLiteTacticError::Machine(diagnostic) = err else {
            panic!("max_simp_rewrite_steps failure should stay a Machine diagnostic");
        };
        assert_eq!(
            diagnostic.kind,
            npa_tactic::MachineTacticDiagnosticKind::SimpStepLimitExceeded
        );
        assert_eq!(state.open_goals, vec![npa_tactic::GoalId(0)]);
    }

    #[test]
    fn human_induction_nat_creates_base_step_and_closes_with_exact_simp() {
        let (nat, nat_interface) = verified_nat_human_import();
        let (eq, eq_interface) = verified_eq_human_import();
        let options = human_nat_compile_options(&nat);
        let verified_modules = vec![nat, eq];
        let imported_source_interfaces = vec![nat_interface, eq_interface];
        let source = "\
import Std.Nat.Basic
import Std.Logic.Eq
theorem ind_self (n : Nat) : Eq.{1} n n := by
  intro n
  induction n
  exact Eq.refl Nat.zero
  simp-lite";
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanInductionNat"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanInductionNat.ind_self"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source,
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: options.clone(),
        })
        .expect("Human proof bridge should start ind_self");
        let script = first_theorem_script(source);

        let ok = run_human_tactic_script(HumanTacticScriptRunRequest {
            state: &started.state,
            script: &script,
            current_source_interface: &started.source_interface,
            imported_source_interfaces: &imported_source_interfaces,
            options,
            budget: npa_tactic::TacticBudget::default(),
        })
        .expect("Human induction script should close base and step goals");

        assert!(ok.state.open_goals.is_empty());
        assert_eq!(ok.deltas.len(), 4);
        assert_eq!(
            ok.deltas[1].added_goals,
            vec![npa_tactic::GoalId(2), npa_tactic::GoalId(3)]
        );
        npa_tactic::extract_closed_machine_proof(&ok.state)
            .expect("Human induction proof should pass kernel check");
    }

    #[test]
    fn human_induction_rejects_dependent_later_hypothesis_as_human_diagnostic() {
        let (nat, nat_interface) = verified_nat_human_import();
        let (eq, eq_interface) = verified_eq_human_import();
        let options = human_nat_compile_options(&nat);
        let verified_modules = vec![nat, eq];
        let imported_source_interfaces = vec![nat_interface, eq_interface];
        let source = "\
import Std.Nat.Basic
import Std.Logic.Eq
theorem bad_induction (n : Nat) (h : Eq.{1} n n) : Eq.{1} n n := by
  intro n
  intro h
  induction n";
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanInductionBad"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanInductionBad.bad_induction"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source,
            },
            verified_modules: &verified_modules,
            imported_source_interfaces: &imported_source_interfaces,
            options: options.clone(),
        })
        .expect("Human proof bridge should start bad_induction");
        let script = first_theorem_script(source);

        let err = run_human_tactic_script(HumanTacticScriptRunRequest {
            state: &started.state,
            script: &script,
            current_source_interface: &started.source_interface,
            imported_source_interfaces: &imported_source_interfaces,
            options,
            budget: npa_tactic::TacticBudget::default(),
        })
        .expect_err("dependent later hypothesis should be rejected by Human induction");

        let HumanTacticScriptError::Human(HumanCompileError { diagnostic }) = err else {
            panic!("dependent induction rejection should map to a Human diagnostic");
        };
        assert_eq!(
            diagnostic.kind,
            npa_frontend::HumanDiagnosticKind::UnsupportedTactic
        );
        let payload = human_payload(&diagnostic);
        assert_eq!(
            payload.phase,
            Some(npa_frontend::HumanDiagnosticPhase::TacticExecution)
        );
        assert_eq!(payload.hole_goals.len(), 1);
        assert_eq!(
            payload.hole_goals[0]
                .context
                .iter()
                .map(|local| local.name.as_str())
                .collect::<Vec<_>>(),
            vec!["n", "h"]
        );
        assert!(diagnostic
            .message
            .contains("cannot perform simple induction"));
    }

    fn workspace_manifest(crate_name: &str) -> String {
        let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .expect("npa-api should live under crates/");
        let manifest_path = workspace_root
            .join("crates")
            .join(crate_name)
            .join("Cargo.toml");
        std::fs::read_to_string(&manifest_path).unwrap_or_else(|err| {
            panic!("failed to read {}: {err}", manifest_path.display());
        })
    }

    fn manifest_declares_dependency(manifest: &str, dependency: &str) -> bool {
        let prefix = format!("{dependency} = ");
        let dotted_prefix = format!("{dependency}.");
        let quoted_prefix = format!("\"{dependency}\" = ");
        let quoted_dotted_prefix = format!("\"{dependency}\".");
        let dependency_tables = [
            format!("[dependencies.{dependency}]"),
            format!("[dev-dependencies.{dependency}]"),
            format!("[build-dependencies.{dependency}]"),
        ];
        let target_dependency_kinds = [
            ".dependencies.",
            ".dev-dependencies.",
            ".build-dependencies.",
        ];
        let dependency_table_suffix = format!(".{dependency}]");
        manifest.lines().map(str::trim_start).any(|line| {
            line.starts_with(&prefix)
                || line.starts_with(&dotted_prefix)
                || line.starts_with(&quoted_prefix)
                || line.starts_with(&quoted_dotted_prefix)
                || dependency_tables.iter().any(|table| line == table)
                || (line.starts_with("[target.")
                    && target_dependency_kinds
                        .iter()
                        .any(|dependency_kind| line.contains(dependency_kind))
                    && line.ends_with(&dependency_table_suffix))
        })
    }

    #[test]
    fn human_tactic_bridge_boundary_avoids_frontend_tactic_cycle() {
        let frontend_manifest = workspace_manifest("npa-frontend");
        let tactic_manifest = workspace_manifest("npa-tactic");
        let api_manifest = workspace_manifest("npa-api");

        assert!(
            !manifest_declares_dependency(&frontend_manifest, "npa-tactic"),
            "Human tactic bridge must not live in npa-frontend; use npa-api or another adapter crate"
        );
        assert!(
            manifest_declares_dependency(&tactic_manifest, "npa-frontend"),
            "npa-tactic may consume Machine Surface helpers from npa-frontend"
        );
        assert!(
            manifest_declares_dependency(&api_manifest, "npa-frontend")
                && manifest_declares_dependency(&api_manifest, "npa-tactic"),
            "npa-api is the current adapter layer that can bridge Human frontend data to tactic execution"
        );
    }

    #[test]
    fn machine_session_api_stays_machine_surface_only() {
        let body = r#"{
            "protocol_version": "npa.machine-api.v1",
            "root": {
                "module": "Api.Machine",
                "theorem_name": "Api.Machine.thm",
                "source_index": 0,
                "universe_params": [],
                "theorem_type": {
                    "format": "machine_surface_v1",
                    "source": "by exact ("
                }
            },
            "import_closure": [],
            "imports": [],
            "checked_current_decls": [],
            "options": {
                "kernel_check_profile": "npa.kernel.v0.1.builtin-nat-eq-rec",
                "allow_axioms": [],
                "tactic_options": {
                    "simp_rules": [],
                    "eq_family": null,
                    "nat_family": null,
                    "max_simp_rewrite_steps": 100,
                    "max_open_goals": 32,
                    "max_metas": 64
                }
            }
        }"#;

        let err = create_machine_session(body)
            .expect_err("Machine session theorem_type must remain Machine Surface");

        assert_eq!(
            err.diagnostic.kind,
            crate::MachineApiErrorKind::MachineTermParseError
        );
    }

    fn human_name(value: &str, start: u32, end: u32) -> npa_frontend::HumanName {
        human_name_parts(&[value], start, end)
    }

    fn human_name_parts(parts: &[&str], start: u32, end: u32) -> npa_frontend::HumanName {
        npa_frontend::HumanName::new(
            parts.iter().map(|part| (*part).to_owned()).collect(),
            npa_frontend::Span::new(npa_frontend::FileId(0), start, end),
        )
    }

    fn first_theorem_script(source: &str) -> npa_frontend::HumanTacticScript {
        nth_theorem_script(source, 0)
    }

    fn assert_human_fixture_certificate_verifies(
        module: &str,
        source: &str,
        verified_modules: &[npa_cert::VerifiedModule],
        imported_source_interfaces: &[npa_frontend::HumanImportedSourceInterface],
        options: HumanApiCompileOptions,
    ) {
        let ok = compile_human_source_to_certificate(HumanCompileCertificateRequest {
            current_module: npa_cert::Name::from_dotted(module),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source,
            },
            verified_modules,
            imported_source_interfaces,
            options,
        })
        .expect("Human tactic fixture should compile to a certificate");
        assert!(ok
            .source_interface
            .declarations
            .iter()
            .all(|decl| decl.decl_interface_hash.is_some()));
        let bytes =
            npa_cert::encode_module_cert(&ok.certificate).expect("fixture certificate encodes");
        let mut session = npa_cert::VerifierSession::new();
        for verified in verified_modules {
            session.register_verified_module(verified.clone());
        }
        let verified =
            npa_cert::verify_module_cert(&bytes, &mut session, &npa_cert::AxiomPolicy::normal())
                .expect("fixture certificate verifies");
        assert_eq!(verified.module(), &npa_cert::Name::from_dotted(module));
    }

    fn nth_theorem_script(source: &str, theorem_index: usize) -> npa_frontend::HumanTacticScript {
        let module = npa_frontend::parse_human_module(npa_frontend::FileId(0), source)
            .expect("Human source should parse");
        module
            .items
            .into_iter()
            .filter_map(|item| {
                let npa_frontend::HumanItem::Theorem(decl) = item else {
                    return None;
                };
                let npa_frontend::HumanDeclValue::ProofBlock(block) = decl.value else {
                    return None;
                };
                Some(block.script)
            })
            .nth(theorem_index)
            .expect("source should contain a theorem proof block")
    }

    fn verified_human_import(
        module: &str,
        source: &str,
    ) -> (
        npa_cert::VerifiedModule,
        npa_frontend::HumanImportedSourceInterface,
    ) {
        let producer = compile_human_source_to_certificate(HumanCompileCertificateRequest {
            current_module: npa_cert::Name::from_dotted(module),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(0),
                source,
            },
            verified_modules: &[],
            imported_source_interfaces: &[],
            options: human_api_default_compile_options(),
        })
        .expect("producer Human import source should compile");
        let bytes =
            npa_cert::encode_module_cert(&producer.certificate).expect("certificate should encode");
        let verified = npa_cert::verify_module_cert(
            &bytes,
            &mut npa_cert::VerifierSession::new(),
            &npa_cert::AxiomPolicy::normal(),
        )
        .expect("certificate should verify");
        let import = npa_frontend::VerifiedImport::from(&verified);
        let source_interface = npa_frontend::HumanImportedSourceInterface {
            module: import.module,
            export_hash: import.export_hash,
            certificate_hash: import.certificate_hash,
            source_interface: producer.source_interface,
        };

        (verified, source_interface)
    }

    fn verified_nat_human_import() -> (
        npa_cert::VerifiedModule,
        npa_frontend::HumanImportedSourceInterface,
    ) {
        verified_human_import(
            "Std.Nat.Basic",
            "\
inductive Nat : Type where
| zero : Nat
| succ : forall (n : Nat), Nat",
        )
    }

    fn verified_eq_human_import() -> (
        npa_cert::VerifiedModule,
        npa_frontend::HumanImportedSourceInterface,
    ) {
        verified_human_import(
            "Std.Logic.Eq",
            "\
inductive Eq.{u} {A : Sort u} (a : A) : forall (b : A), Prop where
| refl : Eq.{u} a a",
        )
    }

    fn verified_core_module(module: npa_cert::CoreModule) -> npa_cert::VerifiedModule {
        let cert = npa_cert::build_module_cert(module, &[]).expect("core module cert builds");
        let bytes = npa_cert::encode_module_cert(&cert).expect("core module cert encodes");
        npa_cert::verify_module_cert(
            &bytes,
            &mut npa_cert::VerifierSession::new(),
            &npa_cert::AxiomPolicy::normal(),
        )
        .expect("core module cert verifies")
    }

    fn verified_one_unfold_simp_module() -> npa_cert::VerifiedModule {
        let one = npa_kernel::Expr::konst("Lib.one", Vec::new());
        verified_core_module(npa_cert::CoreModule {
            name: npa_cert::Name::from_dotted("Lib.Simp"),
            declarations: vec![
                Decl::Def {
                    name: "Lib.one".to_owned(),
                    universe_params: Vec::new(),
                    ty: nat(),
                    value: nat_succ(nat_zero()),
                    reducibility: npa_kernel::Reducibility::Reducible,
                },
                Decl::Theorem {
                    name: "Lib.one_unfold".to_owned(),
                    universe_params: Vec::new(),
                    ty: eq_nat(one, nat_succ(nat_zero())),
                    proof: eq_refl_nat(nat_succ(nat_zero())),
                },
            ],
        })
    }

    fn verified_axiom_simp_close_module() -> npa_cert::VerifiedModule {
        let ty = eq_nat(nat_succ(nat_zero()), nat_zero());
        verified_core_module(npa_cert::CoreModule {
            name: npa_cert::Name::from_dotted("Lib.SimpClose"),
            declarations: vec![
                Decl::Axiom {
                    name: "Lib.succ_zero_axiom".to_owned(),
                    universe_params: Vec::new(),
                    ty: ty.clone(),
                },
                Decl::Theorem {
                    name: "Lib.succ_zero".to_owned(),
                    universe_params: Vec::new(),
                    ty,
                    proof: npa_kernel::Expr::konst("Lib.succ_zero_axiom", Vec::new()),
                },
            ],
        })
    }

    fn verified_axiom_simp_chain_module() -> npa_cert::VerifiedModule {
        let two = nat_succ(nat_succ(nat_zero()));
        let one = nat_succ(nat_zero());
        let zero = nat_zero();
        let first_ty = eq_nat(two, one.clone());
        let second_ty = eq_nat(one, zero);
        verified_core_module(npa_cert::CoreModule {
            name: npa_cert::Name::from_dotted("Lib.SimpChain"),
            declarations: vec![
                Decl::Axiom {
                    name: "Lib.a_two_one_axiom".to_owned(),
                    universe_params: Vec::new(),
                    ty: first_ty.clone(),
                },
                Decl::Theorem {
                    name: "Lib.a_two_one".to_owned(),
                    universe_params: Vec::new(),
                    ty: first_ty,
                    proof: npa_kernel::Expr::konst("Lib.a_two_one_axiom", Vec::new()),
                },
                Decl::Axiom {
                    name: "Lib.b_one_zero_axiom".to_owned(),
                    universe_params: Vec::new(),
                    ty: second_ty.clone(),
                },
                Decl::Theorem {
                    name: "Lib.b_one_zero".to_owned(),
                    universe_params: Vec::new(),
                    ty: second_ty,
                    proof: npa_kernel::Expr::konst("Lib.b_one_zero_axiom", Vec::new()),
                },
            ],
        })
    }

    fn export_interface_hash(import: &npa_tactic::VerifiedImportRef, name: &str) -> npa_cert::Hash {
        import
            .exports()
            .iter()
            .find(|export| export.name == npa_cert::Name::from_dotted(name))
            .expect("test export should exist")
            .decl_interface_hash
    }

    fn human_nat_compile_options(
        verified_nat: &npa_cert::VerifiedModule,
    ) -> HumanApiCompileOptions {
        let nat_import = npa_tactic::VerifiedImportRef::from_verified_module(verified_nat)
            .expect("verified Nat module should become a tactic import");
        HumanApiCompileOptions {
            tactic_options: npa_tactic::MachineTacticOptions {
                nat_family: Some(nat_family_ref(&nat_import)),
                ..npa_tactic::MachineTacticOptions::default()
            },
            ..human_api_default_compile_options()
        }
    }

    fn nat_family_ref(import: &npa_tactic::VerifiedImportRef) -> npa_tactic::NatFamilyRef {
        npa_tactic::NatFamilyRef {
            nat_name: npa_cert::Name::from_dotted("Nat"),
            nat_interface_hash: export_interface_hash(import, "Nat"),
            zero_name: npa_cert::Name::from_dotted("Nat.zero"),
            zero_interface_hash: export_interface_hash(import, "Nat.zero"),
            succ_name: npa_cert::Name::from_dotted("Nat.succ"),
            succ_interface_hash: export_interface_hash(import, "Nat.succ"),
            rec_name: npa_cert::Name::from_dotted("Nat.rec"),
            rec_interface_hash: export_interface_hash(import, "Nat.rec"),
        }
    }

    fn human_simp_machine_spec(theorem_type: Expr) -> npa_tactic::MachineProofSpec {
        npa_tactic::MachineProofSpec {
            module: npa_cert::Name::from_dotted("Api.HumanSimpMachine"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanSimpMachine.target"),
            source_index: 0,
            universe_params: Vec::new(),
            theorem_type,
        }
    }

    fn nat() -> Expr {
        npa_kernel::nat()
    }

    fn nat_zero() -> Expr {
        npa_kernel::nat_zero()
    }

    fn nat_succ(arg: Expr) -> Expr {
        npa_kernel::nat_succ(arg)
    }

    fn eq_nat(lhs: Expr, rhs: Expr) -> Expr {
        npa_kernel::eq(npa_kernel::type0(), nat(), lhs, rhs)
    }

    fn eq_refl_nat(value: Expr) -> Expr {
        npa_kernel::eq_refl(npa_kernel::type0(), nat(), value)
    }
}
