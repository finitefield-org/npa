use crate::{
    HumanApiCompileOptions, HumanCompileCertificateOk, HumanCompileCertificateRequest,
    HumanCompileCoreOk, HumanCompileCoreRequest, HumanCompileError,
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

pub fn human_api_default_compile_options() -> HumanApiCompileOptions {
    HumanApiCompileOptions::default()
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
