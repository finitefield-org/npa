use crate::{
    HumanApiCompileOptions, HumanCompileCertificateOk, HumanCompileCertificateRequest,
    HumanCompileCoreOk, HumanCompileCoreRequest, HumanCompileError, HumanExactTacticOk,
    HumanExactTacticRequest, HumanIntroTacticError, HumanIntroTacticOk, HumanIntroTacticRequest,
    HumanStartProofError, HumanStartProofOk, HumanStartProofRequest, HumanTacticScriptError,
    HumanTacticScriptRunOk, HumanTacticScriptRunRequest, HumanTacticTermCheckOk,
    HumanTacticTermCheckRequest, HumanTacticTermError,
};
use npa_kernel::Decl;

pub fn compile_human_source_to_core(
    request: HumanCompileCoreRequest<'_, '_>,
) -> Result<HumanCompileCoreOk, HumanCompileError> {
    let options = npa_frontend::HumanCompileOptions::from(&request.options);
    let output = npa_frontend::compile_human_source_to_core_output_with_source_interfaces(
        request.current_source.file_id,
        request.current_module,
        request.current_source.source,
        request.verified_imports,
        request.imported_source_interfaces,
        &options,
    )?;
    Ok(HumanCompileCoreOk {
        core_module: output.core_module,
        source_interface: output.source_interface,
    })
}

pub fn compile_human_source_to_certificate(
    request: HumanCompileCertificateRequest<'_, '_>,
) -> Result<HumanCompileCertificateOk, HumanCompileError> {
    let options = npa_frontend::HumanCompileOptions::from(&request.options);
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
    let phase4_imports =
        active_human_verified_import_refs(request.verified_modules, &prepared.active_imports)?;
    let mut checked_current_decls = Vec::with_capacity(prepared.proof.prior_declarations.len());
    for (source_index, decl) in prepared
        .proof
        .prior_declarations
        .iter()
        .cloned()
        .enumerate()
    {
        let checked = npa_tactic::check_current_decl_for_machine_tactic_from_verified_imports(
            &phase4_imports,
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
        phase4_imports,
        checked_current_decls,
        npa_tactic::MachineTacticOptions::default(),
    )?;
    npa_tactic::validate_machine_proof_state(&state)?;

    Ok(HumanStartProofOk {
        state,
        source_interface: prepared.source_interface,
    })
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
    let checked = check_human_tactic_term(HumanTacticTermCheckRequest {
        state: request.state,
        goal_id: request.goal_id,
        term: request.term,
        current_source_interface: request.current_source_interface,
        imported_source_interfaces: request.imported_source_interfaces,
        options: request.options,
    })?;
    let (state, delta) = npa_tactic::assign_goal(
        request.state,
        request.goal_id,
        npa_tactic::ProofExpr::Core(checked.expr.clone()),
        Vec::new(),
    )?;
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
    let (state, delta) = npa_tactic::run_machine_tactic_with_budget(
        request.state,
        npa_tactic::MachineTactic::Intro {
            goal_id: request.goal_id,
            name,
        },
        request.budget,
    )
    .map_err(|diagnostic| human_intro_machine_error(diagnostic, request.name.span))?;
    npa_tactic::validate_machine_proof_state(&state)?;

    Ok(HumanIntroTacticOk { state, delta })
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
            other => {
                return Err(
                    human_script_unsupported_tactic_diagnostic(other.span(), other.kind()).into(),
                );
            }
        }
    }

    if !state.open_goals.is_empty() {
        return Err(human_script_unresolved_goal_diagnostic(
            request.script.span,
            state.open_goals.len(),
        )
        .into());
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

fn human_intro_machine_error(
    diagnostic: npa_tactic::MachineTacticDiagnostic,
    span: npa_frontend::Span,
) -> HumanIntroTacticError {
    match diagnostic.kind {
        npa_tactic::MachineTacticDiagnosticKind::TypeMismatch => {
            npa_frontend::HumanDiagnostic::error(
                npa_frontend::HumanDiagnosticKind::ExpectedFunctionType,
                span,
                "`intro` can only be used when the target is a function type or forall.",
            )
            .with_phase(npa_frontend::HumanDiagnosticPhase::Elaborator)
            .into()
        }
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
    .with_phase(npa_frontend::HumanDiagnosticPhase::Elaborator)
}

fn human_script_intro_error(error: HumanIntroTacticError) -> HumanTacticScriptError {
    match error {
        HumanIntroTacticError::Human(error) => HumanTacticScriptError::Human(error),
        HumanIntroTacticError::Machine(diagnostic) => HumanTacticScriptError::Machine(diagnostic),
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
    .with_phase(npa_frontend::HumanDiagnosticPhase::Elaborator)
}

fn human_script_unresolved_goal_diagnostic(
    span: npa_frontend::Span,
    open_goal_count: usize,
) -> npa_frontend::HumanDiagnostic {
    npa_frontend::HumanDiagnostic::error(
        npa_frontend::HumanDiagnosticKind::UnresolvedGoal,
        span,
        format!("Human tactic script finished with {open_goal_count} open goal(s)"),
    )
    .with_phase(npa_frontend::HumanDiagnosticPhase::Elaborator)
}

fn human_script_unsupported_tactic_diagnostic(
    span: npa_frontend::Span,
    kind: npa_frontend::HumanTacticKind,
) -> npa_frontend::HumanDiagnostic {
    npa_frontend::HumanDiagnostic::unsupported_tactic(
        span,
        format!("{kind:?} is not implemented by the Human script executor yet"),
    )
    .with_phase(npa_frontend::HumanDiagnosticPhase::Elaborator)
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
    use crate::{create_machine_session, HumanCurrentModuleSource};

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
            .expect("Human API should compile source to a Phase 2 certificate");
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
    fn human_api_core_request_uses_explicit_verified_imports_and_current_source() {
        let request = HumanCompileCoreRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanCore"),
            current_source: HumanCurrentModuleSource {
                file_id: npa_frontend::FileId(7),
                source: "def id : forall (A : Type), Type := fun A => A",
            },
            verified_imports: &[],
            imported_source_interfaces: &[],
            options: HumanApiCompileOptions {
                max_notation_candidates: 4,
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
            verified_imports: &[import],
            imported_source_interfaces: &[source_interface],
            options: human_api_default_compile_options(),
        })
        .expect("consumer Human API request should use imported source metadata");

        assert_eq!(consumer.core_module.declarations.len(), 2);
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

        let err = run_human_intro_tactic(HumanIntroTacticRequest {
            state: &started.state,
            goal_id: npa_tactic::GoalId(0),
            name: &name,
            budget: npa_tactic::TacticBudget::default(),
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
            Some(npa_frontend::HumanDiagnosticPhase::Elaborator)
        );
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
        assert_eq!(started.state.open_goals, vec![npa_tactic::GoalId(0)]);
    }

    #[test]
    fn human_tactic_script_executor_rejects_unimplemented_tactic_without_machine_batch() {
        let (nat, nat_interface) = verified_nat_human_import();
        let verified_modules = vec![nat];
        let imported_source_interfaces = vec![nat_interface];
        let source = "\
import Std.Nat.Basic
theorem id_nat : forall (n : Nat), Nat := by
  apply Nat.zero";
        let started = start_human_proof(HumanStartProofRequest {
            current_module: npa_cert::Name::from_dotted("Api.HumanScriptUnsupported"),
            theorem_name: npa_cert::Name::from_dotted("Api.HumanScriptUnsupported.id_nat"),
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
        .expect_err("unsupported tactic should be rejected by Human script executor");

        let HumanTacticScriptError::Human(HumanCompileError { diagnostic }) = err else {
            panic!("unsupported tactic should map to a Human diagnostic");
        };
        assert_eq!(
            diagnostic.kind,
            npa_frontend::HumanDiagnosticKind::UnsupportedTactic
        );
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
                    "source": "def human : Type := Type"
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
        let module = npa_frontend::parse_human_module(npa_frontend::FileId(0), source)
            .expect("Human source should parse");
        module
            .items
            .into_iter()
            .find_map(|item| {
                let npa_frontend::HumanItem::Theorem(decl) = item else {
                    return None;
                };
                let npa_frontend::HumanDeclValue::ProofBlock(block) = decl.value else {
                    return None;
                };
                Some(block.script)
            })
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
}
