//! Implementation of `npa package build-certs`.

use std::{collections::BTreeSet, fs};

use npa_cert::{AxiomPolicy, CoreFeature, ModuleCert, Name, VerifiedModule, VerifierSession};
use npa_frontend::{
    compile_human_source_to_certificate_output_with_source_interfaces_and_axiom_policy, FileId,
    HumanCompileOptions, HumanImportedSourceInterface, HumanName, HumanSourceDeclarationKind,
    HumanSourceDeclarationMetadata, HumanSourceInterface, HumanUniverseParam, Span, VerifiedImport,
};
use npa_package::{
    build_package_lock_from_artifacts, format_package_hash, package_file_hash,
    parse_package_lock_json, PackageHash, PackageLockArtifact, PackageModule, PackagePath,
};

use crate::args::{PackageBuildCertsOptions, PackageCommonOptions};
use crate::diagnostic::{CommandDiagnostic, CommandResult, DiagnosticKind};
use crate::fs::{join_package_path, render_package_path, render_package_root};
use crate::package::{load_package_root, LoadedPackageRoot};

const COMMAND: &str = "package build-certs";
const PACKAGE_LOCK_PATH: &str = "generated/package-lock.json";

#[derive(Clone, Debug)]
struct CertificateArtifactBuffer {
    path: PackagePath,
    bytes: Vec<u8>,
}

/// Run `package build-certs`.
pub fn run_package_build_certs(options: PackageBuildCertsOptions) -> CommandResult {
    if options.check {
        return run_package_build_certs_check(options.common);
    }

    let diagnostic = CommandDiagnostic::error(DiagnosticKind::Internal, "command_not_implemented")
        .with_actual_value("package build-certs write mode");
    CommandResult::failed(
        COMMAND,
        render_package_root(&options.common.root),
        vec![diagnostic],
    )
}

/// Run no-write certificate rebuild checking.
///
/// This command reads package source files, local certificate files, external
/// pinned certificate artifacts, and `generated/package-lock.json`. It builds
/// local certificates in memory through the untrusted frontend, verifies the
/// generated canonical certificate bytes through `npa-cert`, and compares the
/// results to the manifest and checked-in artifacts. It does not write files.
pub fn run_package_build_certs_check(options: PackageCommonOptions) -> CommandResult {
    let loaded = match load_package_root(&options.root, COMMAND) {
        Ok(loaded) => loaded,
        Err(result) => return result,
    };
    let policy = axiom_policy_for_package(&loaded);

    let mut verified_modules = Vec::new();
    let mut source_interfaces = Vec::new();
    let mut artifacts = Vec::new();

    if let Some(diagnostic) = load_external_imports(
        &loaded,
        &policy,
        &mut verified_modules,
        &mut source_interfaces,
        &mut artifacts,
    ) {
        return CommandResult::failed(COMMAND, loaded.root_display, vec![diagnostic]);
    }

    if let Some(diagnostic) = build_local_modules(
        &loaded,
        &policy,
        &mut verified_modules,
        &mut source_interfaces,
        &mut artifacts,
    ) {
        return CommandResult::failed(COMMAND, loaded.root_display, vec![diagnostic]);
    }

    let regenerated_lock = match build_package_lock_from_artifacts(
        &loaded.validated,
        loaded.manifest_path.clone(),
        loaded.manifest_source.as_bytes(),
        artifacts.iter().map(|artifact| PackageLockArtifact {
            path: artifact.path.clone(),
            bytes: artifact.bytes.as_slice(),
        }),
    ) {
        Ok(lock) => lock,
        Err(error) => {
            return CommandResult::failed(
                COMMAND,
                loaded.root_display,
                vec![CommandDiagnostic::from_package_lock_error(&error)],
            );
        }
    };

    let regenerated_lock_json = match regenerated_lock.canonical_json() {
        Ok(json) => json,
        Err(error) => {
            return CommandResult::failed(
                COMMAND,
                loaded.root_display,
                vec![CommandDiagnostic::from_package_lock_error(&error)],
            );
        }
    };

    if let Some(diagnostic) = check_package_lock(&loaded, &regenerated_lock_json) {
        return CommandResult::failed(COMMAND, loaded.root_display, vec![diagnostic]);
    }

    CommandResult::passed(COMMAND, loaded.root_display)
}

fn axiom_policy_for_package(loaded: &LoadedPackageRoot) -> AxiomPolicy {
    let mut policy = AxiomPolicy::normal()
        .with_core_feature(CoreFeature::QuotientV1)
        .with_core_feature(CoreFeature::QuotientV2)
        .with_core_feature(CoreFeature::QuotientV3);
    if !loaded.validated.manifest().policy.allow_custom_axioms {
        policy.allowlisted_axioms = loaded
            .validated
            .manifest()
            .policy
            .allowed_axioms
            .iter()
            .cloned()
            .collect();
    }
    policy
}

fn load_external_imports(
    loaded: &LoadedPackageRoot,
    policy: &AxiomPolicy,
    verified_modules: &mut Vec<VerifiedModule>,
    source_interfaces: &mut Vec<HumanImportedSourceInterface>,
    artifacts: &mut Vec<CertificateArtifactBuffer>,
) -> Option<CommandDiagnostic> {
    let mut session = VerifierSession::new();
    for (index, import) in loaded
        .validated
        .manifest()
        .imports
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .enumerate()
    {
        let bytes = match read_certificate_bytes(
            loaded,
            &import.certificate,
            format!("imports[{index}].certificate"),
        ) {
            Ok(bytes) => bytes,
            Err(diagnostic) => return Some(*diagnostic),
        };
        let verified = match npa_cert::verify_module_cert(&bytes, &mut session, policy) {
            Ok(verified) => verified,
            Err(error) => {
                return Some(
                    CommandDiagnostic::error(
                        DiagnosticKind::Build,
                        "external_certificate_rejected",
                    )
                    .with_module(import.module.as_dotted())
                    .with_path(render_package_path(&import.certificate))
                    .with_actual_value(format!("{error:?}")),
                );
            }
        };

        if verified.module() != &import.module {
            return Some(
                CommandDiagnostic::error(DiagnosticKind::Build, "certificate_module_mismatch")
                    .with_module(import.module.as_dotted())
                    .with_path(format!("imports[{index}].certificate"))
                    .with_field("module")
                    .with_expected_value(import.module.as_dotted())
                    .with_actual_value(verified.module().as_dotted()),
            );
        }
        let actual_export_hash = PackageHash::from(verified.export_hash());
        if actual_export_hash != import.export_hash {
            return Some(hash_mismatch(
                "export_hash_mismatch",
                format!("imports[{index}].export_hash"),
                "export_hash",
                import.export_hash,
                actual_export_hash,
            ));
        }
        let actual_certificate_hash = PackageHash::from(verified.certificate_hash());
        if actual_certificate_hash != import.certificate_hash {
            return Some(hash_mismatch(
                "certificate_hash_mismatch",
                format!("imports[{index}].certificate_hash"),
                "certificate_hash",
                import.certificate_hash,
                actual_certificate_hash,
            ));
        }

        source_interfaces.push(fallback_imported_source_interface(&verified));
        verified_modules.push(verified);
        artifacts.push(CertificateArtifactBuffer {
            path: import.certificate.clone(),
            bytes,
        });
    }
    None
}

fn build_local_modules(
    loaded: &LoadedPackageRoot,
    policy: &AxiomPolicy,
    verified_modules: &mut Vec<VerifiedModule>,
    source_interfaces: &mut Vec<HumanImportedSourceInterface>,
    artifacts: &mut Vec<CertificateArtifactBuffer>,
) -> Option<CommandDiagnostic> {
    let compile_options = HumanCompileOptions::default();
    for &module_index in &loaded.validated.graph().topological_order {
        let module = &loaded.validated.manifest().modules[module_index];
        let source = match read_source(loaded, module_index, module) {
            Ok(source) => source,
            Err(diagnostic) => return Some(*diagnostic),
        };
        let file_id = match u32::try_from(module_index) {
            Ok(index) => FileId(index),
            Err(_) => {
                return Some(
                    CommandDiagnostic::error(DiagnosticKind::Internal, "module_index_out_of_range")
                        .with_module(module.module.as_dotted()),
                );
            }
        };
        let (direct_verified_modules, direct_source_interfaces) = match direct_import_context(
            loaded,
            module_index,
            verified_modules,
            source_interfaces,
        ) {
            Ok(imports) => imports,
            Err(diagnostic) => return Some(*diagnostic),
        };

        let output =
            match compile_human_source_to_certificate_output_with_source_interfaces_and_axiom_policy(
                file_id,
                module.module.clone(),
                &source,
                &direct_verified_modules,
                &direct_source_interfaces,
                &compile_options,
                policy,
            ) {
                Ok(output) => output,
                Err(error) => return Some(frontend_build_failed(module_index, module, error)),
            };

        if let Some(diagnostic) =
            check_generated_axiom_policy(loaded, module_index, module, &output.certificate)
        {
            return Some(diagnostic);
        }

        let generated_bytes = match npa_cert::encode_module_cert(&output.certificate) {
            Ok(bytes) => bytes,
            Err(error) => {
                return Some(
                    CommandDiagnostic::error(DiagnosticKind::Build, "certificate_encode_failed")
                        .with_module(module.module.as_dotted())
                        .with_path(format!("modules[{module_index}].certificate"))
                        .with_actual_value(format!("{error:?}")),
                );
            }
        };
        if let Some(diagnostic) = check_generated_manifest_hashes(
            module_index,
            module,
            &output.certificate,
            &generated_bytes,
        ) {
            return Some(diagnostic);
        }

        let checked_in_bytes = match read_certificate_bytes(
            loaded,
            &module.certificate,
            format!("modules[{module_index}].certificate"),
        ) {
            Ok(bytes) => bytes,
            Err(diagnostic) => return Some(*diagnostic),
        };
        if checked_in_bytes != generated_bytes {
            return Some(
                CommandDiagnostic::error(DiagnosticKind::Build, "build_certificate_changed")
                    .with_module(module.module.as_dotted())
                    .with_path(render_package_path(&module.certificate))
                    .with_hashes(
                        format_package_hash(&package_file_hash(&checked_in_bytes)),
                        format_package_hash(&package_file_hash(&generated_bytes)),
                    ),
            );
        }

        let verified = output.verified_module.clone();
        if verified.module() != &module.module {
            return Some(
                CommandDiagnostic::error(DiagnosticKind::Build, "certificate_module_mismatch")
                    .with_module(module.module.as_dotted())
                    .with_path(format!("modules[{module_index}].certificate"))
                    .with_field("module")
                    .with_expected_value(module.module.as_dotted())
                    .with_actual_value(verified.module().as_dotted()),
            );
        }

        let imported_source_interface = HumanImportedSourceInterface {
            module: module.module.clone(),
            export_hash: output.certificate.hashes.export_hash,
            certificate_hash: Some(output.certificate.hashes.certificate_hash),
            source_interface: output.source_interface,
        };
        verified_modules.push(verified);
        source_interfaces.push(imported_source_interface);
        artifacts.push(CertificateArtifactBuffer {
            path: module.certificate.clone(),
            bytes: generated_bytes,
        });
    }
    None
}

fn direct_import_context(
    loaded: &LoadedPackageRoot,
    module_index: usize,
    verified_modules: &[VerifiedModule],
    source_interfaces: &[HumanImportedSourceInterface],
) -> Result<(Vec<VerifiedModule>, Vec<HumanImportedSourceInterface>), Box<CommandDiagnostic>> {
    let mut direct_verified_modules = Vec::new();
    let mut direct_source_interfaces = Vec::new();

    for (import_index, import) in loaded.validated.graph().resolved_module_imports[module_index]
        .iter()
        .enumerate()
    {
        let Some(verified) = verified_modules
            .iter()
            .find(|module| module.module() == &import.module)
        else {
            return Err(Box::new(
                CommandDiagnostic::error(DiagnosticKind::Internal, "import_not_built")
                    .with_module(import.module.as_dotted())
                    .with_path(format!("modules[{module_index}].imports[{import_index}]")),
            ));
        };

        let actual_export_hash = PackageHash::from(verified.export_hash());
        if actual_export_hash != import.export_hash {
            return Err(Box::new(hash_mismatch(
                "export_hash_mismatch",
                format!("modules[{module_index}].imports[{import_index}].export_hash"),
                "export_hash",
                import.export_hash,
                actual_export_hash,
            )));
        }
        let actual_certificate_hash = PackageHash::from(verified.certificate_hash());
        if actual_certificate_hash != import.certificate_hash {
            return Err(Box::new(hash_mismatch(
                "certificate_hash_mismatch",
                format!("modules[{module_index}].imports[{import_index}].certificate_hash"),
                "certificate_hash",
                import.certificate_hash,
                actual_certificate_hash,
            )));
        }

        let Some(source_interface) = source_interfaces
            .iter()
            .find(|source_interface| source_interface.module == import.module)
        else {
            return Err(Box::new(
                CommandDiagnostic::error(DiagnosticKind::Internal, "source_interface_missing")
                    .with_module(import.module.as_dotted())
                    .with_path(format!("modules[{module_index}].imports[{import_index}]")),
            ));
        };
        direct_verified_modules.push(verified.clone());
        direct_source_interfaces.push(source_interface.clone());
    }

    Ok((direct_verified_modules, direct_source_interfaces))
}

fn read_source(
    loaded: &LoadedPackageRoot,
    module_index: usize,
    module: &PackageModule,
) -> Result<String, Box<CommandDiagnostic>> {
    let path = join_package_path(
        &loaded.root,
        &module.source,
        format!("modules[{module_index}].source"),
    )?;
    fs::read_to_string(path).map_err(|_| {
        Box::new(
            CommandDiagnostic::error(DiagnosticKind::ArtifactIo, "source_missing")
                .with_module(module.module.as_dotted())
                .with_path(render_package_path(&module.source)),
        )
    })
}

fn read_certificate_bytes(
    loaded: &LoadedPackageRoot,
    path: &PackagePath,
    manifest_field_path: impl Into<String>,
) -> Result<Vec<u8>, Box<CommandDiagnostic>> {
    let path = path.clone();
    let full_path = join_package_path(&loaded.root, &path, manifest_field_path)?;
    fs::read(full_path).map_err(|_| {
        Box::new(
            CommandDiagnostic::error(DiagnosticKind::ArtifactIo, "certificate_missing")
                .with_path(render_package_path(&path)),
        )
    })
}

fn check_generated_axiom_policy(
    loaded: &LoadedPackageRoot,
    module_index: usize,
    module: &PackageModule,
    certificate: &ModuleCert,
) -> Option<CommandDiagnostic> {
    let package_policy = &loaded.validated.manifest().policy;
    if package_policy.allow_custom_axioms {
        return None;
    }

    let allowed_axioms = package_policy
        .allowed_axioms
        .iter()
        .collect::<BTreeSet<&Name>>();
    for axiom in &certificate.axiom_report.module_axioms {
        let Some(name) = certificate.name_table.get(axiom.name) else {
            return Some(
                CommandDiagnostic::error(DiagnosticKind::Build, "certificate_axiom_name_missing")
                    .with_module(module.module.as_dotted())
                    .with_path(format!("modules[{module_index}].certificate")),
            );
        };
        if name.as_dotted().contains("sorry") || !allowed_axioms.contains(name) {
            return Some(
                CommandDiagnostic::error(DiagnosticKind::Build, "disallowed_axiom")
                    .with_module(module.module.as_dotted())
                    .with_path(format!("modules[{module_index}].axioms"))
                    .with_field("axioms")
                    .with_expected_value("allowed axiom or allow_custom_axioms = true")
                    .with_actual_value(name.as_dotted()),
            );
        }
    }
    None
}

fn check_generated_manifest_hashes(
    module_index: usize,
    module: &PackageModule,
    certificate: &ModuleCert,
    certificate_bytes: &[u8],
) -> Option<CommandDiagnostic> {
    let actual_file_hash = package_file_hash(certificate_bytes);
    if actual_file_hash != module.expected_certificate_file_hash {
        return Some(hash_mismatch(
            "certificate_file_hash_mismatch",
            format!("modules[{module_index}].expected_certificate_file_hash"),
            "expected_certificate_file_hash",
            module.expected_certificate_file_hash,
            actual_file_hash,
        ));
    }

    let actual_export_hash = PackageHash::from(certificate.hashes.export_hash);
    if actual_export_hash != module.expected_export_hash {
        return Some(hash_mismatch(
            "export_hash_mismatch",
            format!("modules[{module_index}].expected_export_hash"),
            "expected_export_hash",
            module.expected_export_hash,
            actual_export_hash,
        ));
    }

    let actual_axiom_report_hash = PackageHash::from(certificate.hashes.axiom_report_hash);
    if actual_axiom_report_hash != module.expected_axiom_report_hash {
        return Some(hash_mismatch(
            "axiom_report_hash_mismatch",
            format!("modules[{module_index}].expected_axiom_report_hash"),
            "expected_axiom_report_hash",
            module.expected_axiom_report_hash,
            actual_axiom_report_hash,
        ));
    }

    let actual_certificate_hash = PackageHash::from(certificate.hashes.certificate_hash);
    if actual_certificate_hash != module.expected_certificate_hash {
        return Some(hash_mismatch(
            "certificate_hash_mismatch",
            format!("modules[{module_index}].expected_certificate_hash"),
            "expected_certificate_hash",
            module.expected_certificate_hash,
            actual_certificate_hash,
        ));
    }

    None
}

fn check_package_lock(
    loaded: &LoadedPackageRoot,
    regenerated_lock_json: &str,
) -> Option<CommandDiagnostic> {
    let lock_path = PackagePath::new(PACKAGE_LOCK_PATH);
    let full_lock_path = match join_package_path(&loaded.root, &lock_path, "package_lock.path") {
        Ok(path) => path,
        Err(diagnostic) => return Some(*diagnostic),
    };
    let lock_source = match fs::read_to_string(&full_lock_path) {
        Ok(source) => source,
        Err(_) => {
            return Some(
                CommandDiagnostic::error(DiagnosticKind::PackageLock, "package_lock_missing")
                    .with_path(PACKAGE_LOCK_PATH),
            );
        }
    };
    if let Err(error) = parse_package_lock_json(&lock_source) {
        return Some(
            CommandDiagnostic::from_package_lock_error(&error).with_path(PACKAGE_LOCK_PATH),
        );
    }
    if lock_source != regenerated_lock_json {
        return Some(
            CommandDiagnostic::error(DiagnosticKind::HashMismatch, "package_lock_stale")
                .with_path(PACKAGE_LOCK_PATH)
                .with_hashes(
                    format_package_hash(&package_file_hash(regenerated_lock_json.as_bytes())),
                    format_package_hash(&package_file_hash(lock_source.as_bytes())),
                ),
        );
    }
    None
}

fn fallback_imported_source_interface(verified: &VerifiedModule) -> HumanImportedSourceInterface {
    let import = VerifiedImport::from(verified);
    let empty_span = Span::empty(FileId(0));
    let mut source_interface = HumanSourceInterface::new(import.module.clone());
    source_interface.declarations = import
        .exports
        .iter()
        .map(|export| HumanSourceDeclarationMetadata {
            kind: HumanSourceDeclarationKind::Imported,
            name: HumanName::new(export.name.0.clone(), empty_span),
            universe_params: export
                .universe_params
                .iter()
                .cloned()
                .map(|name| HumanUniverseParam {
                    name,
                    span: empty_span,
                })
                .collect(),
            binders: Vec::new(),
            decl_interface_hash: Some(export.decl_interface_hash),
            span: empty_span,
        })
        .collect();

    HumanImportedSourceInterface {
        module: import.module,
        export_hash: import.export_hash,
        certificate_hash: import.certificate_hash,
        source_interface,
    }
}

fn frontend_build_failed(
    module_index: usize,
    module: &PackageModule,
    error: npa_frontend::HumanDiagnostic,
) -> CommandDiagnostic {
    let phase = error
        .payload
        .as_ref()
        .and_then(|payload| payload.phase)
        .map(|phase| phase.as_str())
        .unwrap_or("human_frontend");
    CommandDiagnostic::error(DiagnosticKind::Build, "build_failed")
        .with_module(module.module.as_dotted())
        .with_path(format!("modules[{module_index}].source"))
        .with_field(phase)
        .with_actual_value(error.message)
}

fn hash_mismatch(
    reason_code: &'static str,
    path: String,
    field: &'static str,
    expected: PackageHash,
    actual: PackageHash,
) -> CommandDiagnostic {
    CommandDiagnostic::error(DiagnosticKind::HashMismatch, reason_code)
        .with_path(path)
        .with_field(field)
        .with_hashes(format_package_hash(&expected), format_package_hash(&actual))
}
