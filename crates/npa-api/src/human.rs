use crate::{
    HumanApiCompileOptions, HumanCompileCertificateOk, HumanCompileCertificateRequest,
    HumanCompileCoreOk, HumanCompileCoreRequest, HumanCompileError, HumanStartProofError,
    HumanStartProofOk, HumanStartProofRequest,
};

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

pub fn human_api_default_compile_options() -> HumanApiCompileOptions {
    HumanApiCompileOptions::default()
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
}
