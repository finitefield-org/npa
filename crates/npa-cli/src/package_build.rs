//! Implementation of `npa package build-certs`.

use std::{
    collections::BTreeSet,
    fs, io,
    path::{Path, PathBuf},
};

use npa_api::{build_legacy_std_package_module_cert, LEGACY_STD_PACKAGE_PRODUCER_PROFILE};
use npa_cert::{AxiomPolicy, CoreFeature, ModuleCert, Name, VerifiedModule, VerifierSession};
use npa_frontend::{
    compile_human_source_to_certificate_output_with_source_interfaces_and_axiom_policy,
    parse_human_module, FileId, HumanCompileOptions, HumanImportedSourceInterface, HumanItem,
    HumanName, HumanSourceDeclarationKind, HumanSourceDeclarationMetadata, HumanSourceInterface,
    HumanUniverseParam, Span, VerifiedImport,
};
use npa_package::{
    build_package_lock_from_artifacts, format_package_hash, package_file_hash,
    parse_package_lock_json, PackageHash, PackageLockArtifact, PackageModule, PackagePath,
};

use crate::args::{PackageBuildCertsOptions, PackageCommonOptions};
use crate::diagnostic::{CommandDiagnostic, CommandResult, DiagnosticKind};
use crate::fs::{join_package_path, render_package_path};
use crate::package::{load_package_root, LoadedPackageRoot};

const COMMAND: &str = "package build-certs";
const PACKAGE_LOCK_PATH: &str = "generated/package-lock.json";

#[derive(Clone, Debug)]
struct CertificateArtifactBuffer {
    path: PackagePath,
    bytes: Vec<u8>,
}

#[derive(Clone, Debug)]
struct LocalCertificateBuild {
    module_index: usize,
    module: Name,
    path: PackagePath,
    bytes: Vec<u8>,
}

#[derive(Clone, Debug)]
struct PackageCertificateBuild {
    local_certificates: Vec<LocalCertificateBuild>,
    package_lock_json: String,
}

#[derive(Clone, Debug)]
struct PendingWrite {
    path: PackagePath,
    full_path: PathBuf,
    temp_path: PathBuf,
    reason_code: &'static str,
    module: Option<Name>,
}

/// Run `package build-certs`.
pub fn run_package_build_certs(options: PackageBuildCertsOptions) -> CommandResult {
    if options.check {
        return run_package_build_certs_check(options.common);
    }

    run_package_build_certs_write(options.common)
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

    let build = match build_package_certificates(&loaded) {
        Ok(build) => build,
        Err(diagnostic) => {
            return CommandResult::failed(COMMAND, loaded.root_display, vec![*diagnostic]);
        }
    };

    if let Some(diagnostic) = check_local_certificate_files(&loaded, &build.local_certificates) {
        return CommandResult::failed(COMMAND, loaded.root_display, vec![diagnostic]);
    }

    if let Some(diagnostic) = check_package_lock(&loaded, &build.package_lock_json) {
        return CommandResult::failed(COMMAND, loaded.root_display, vec![diagnostic]);
    }

    CommandResult::passed(COMMAND, loaded.root_display)
}

/// Run certificate rebuild write mode.
///
/// This mode uses the same complete in-memory build as `--check`, then writes
/// only command-owned certificate artifacts and the generated package lock. No
/// target file is touched until every module has built successfully.
pub fn run_package_build_certs_write(options: PackageCommonOptions) -> CommandResult {
    let loaded = match load_package_root(&options.root, COMMAND) {
        Ok(loaded) => loaded,
        Err(result) => return result,
    };

    if let Some(diagnostic) = check_write_mode_targets(&loaded) {
        return CommandResult::failed(COMMAND, loaded.root_display, vec![diagnostic]);
    }

    let build = match build_package_certificates(&loaded) {
        Ok(build) => build,
        Err(diagnostic) => {
            return CommandResult::failed(COMMAND, loaded.root_display, vec![*diagnostic]);
        }
    };

    if let Some(diagnostic) = write_package_build(&loaded, &build) {
        return CommandResult::failed(COMMAND, loaded.root_display, vec![diagnostic]);
    }

    CommandResult::passed(COMMAND, loaded.root_display)
}

fn check_write_mode_targets(loaded: &LoadedPackageRoot) -> Option<CommandDiagnostic> {
    for (module_index, module) in loaded.validated.manifest().modules.iter().enumerate() {
        let Some(forbidden_reason) =
            forbidden_local_certificate_write_reason(loaded, &module.certificate)
        else {
            continue;
        };
        return Some(
            CommandDiagnostic::error(
                DiagnosticKind::ArtifactIo,
                "certificate_write_target_forbidden",
            )
            .with_module(module.module.as_dotted())
            .with_path(render_package_path(&module.certificate))
            .with_field(format!("modules[{module_index}].certificate"))
            .with_expected_value("local module .npcert certificate artifact")
            .with_actual_value(forbidden_reason),
        );
    }
    None
}

fn forbidden_local_certificate_write_reason(
    loaded: &LoadedPackageRoot,
    path: &PackagePath,
) -> Option<&'static str> {
    if path == &loaded.manifest_path {
        return Some("package_manifest");
    }
    if path.as_str() == PACKAGE_LOCK_PATH {
        return Some("package_lock");
    }
    if loaded
        .validated
        .manifest()
        .imports
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .any(|import| import.certificate == *path)
    {
        return Some("external_import_certificate");
    }
    for module in &loaded.validated.manifest().modules {
        if module.source == *path {
            return Some("source_file");
        }
        if module.meta.as_ref() == Some(path) || module.replay.as_ref() == Some(path) {
            return Some("untrusted_sidecar");
        }
    }
    if !path.as_str().ends_with(".npcert") {
        return Some("non_npcert_certificate_path");
    }
    None
}

fn build_package_certificates(
    loaded: &LoadedPackageRoot,
) -> Result<PackageCertificateBuild, Box<CommandDiagnostic>> {
    let policy = axiom_policy_for_package(loaded);
    let mut verified_modules = Vec::new();
    let mut source_interfaces = Vec::new();
    let mut artifacts = Vec::new();
    let mut local_certificates = Vec::new();

    if let Some(diagnostic) = load_external_imports(
        loaded,
        &policy,
        &mut verified_modules,
        &mut source_interfaces,
        &mut artifacts,
    ) {
        return Err(Box::new(diagnostic));
    }

    if let Some(diagnostic) = build_local_modules(
        loaded,
        &policy,
        &mut verified_modules,
        &mut source_interfaces,
        &mut artifacts,
        &mut local_certificates,
    ) {
        return Err(Box::new(diagnostic));
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
            return Err(Box::new(CommandDiagnostic::from_package_lock_error(&error)));
        }
    };

    let regenerated_lock_json = match regenerated_lock.canonical_json() {
        Ok(json) => json,
        Err(error) => {
            return Err(Box::new(CommandDiagnostic::from_package_lock_error(&error)));
        }
    };

    Ok(PackageCertificateBuild {
        local_certificates,
        package_lock_json: regenerated_lock_json,
    })
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
    local_certificates: &mut Vec<LocalCertificateBuild>,
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

        let (certificate, generated_bytes, verified, source_interface) = if module
            .producer_profile
            .as_deref()
            == Some(LEGACY_STD_PACKAGE_PRODUCER_PROFILE)
        {
            match build_legacy_std_package_certificate(
                module_index,
                module,
                &source,
                &direct_verified_modules,
                policy,
            ) {
                Ok(output) => output,
                Err(diagnostic) => return Some(*diagnostic),
            }
        } else {
            let output = match compile_human_source_to_certificate_output_with_source_interfaces_and_axiom_policy(
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
            let generated_bytes = match npa_cert::encode_module_cert(&output.certificate) {
                Ok(bytes) => bytes,
                Err(error) => {
                    return Some(
                        CommandDiagnostic::error(
                            DiagnosticKind::Build,
                            "certificate_encode_failed",
                        )
                        .with_module(module.module.as_dotted())
                        .with_path(format!("modules[{module_index}].certificate"))
                        .with_actual_value(format!("{error:?}")),
                    );
                }
            };
            (
                output.certificate,
                generated_bytes,
                output.verified_module,
                output.source_interface,
            )
        };

        if let Some(diagnostic) =
            check_generated_axiom_policy(loaded, module_index, module, &certificate)
        {
            return Some(diagnostic);
        }

        if std::env::var_os("NPA_SKIP_PACKAGE_BUILD_HASH_CHECKS").is_none() {
            if let Some(diagnostic) =
                check_generated_manifest_hashes(module_index, module, &certificate, &generated_bytes)
            {
                return Some(diagnostic);
            }
        }

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
            export_hash: certificate.hashes.export_hash,
            certificate_hash: Some(certificate.hashes.certificate_hash),
            source_interface,
        };
        verified_modules.push(verified);
        source_interfaces.push(imported_source_interface);
        local_certificates.push(LocalCertificateBuild {
            module_index,
            module: module.module.clone(),
            path: module.certificate.clone(),
            bytes: generated_bytes.clone(),
        });
        artifacts.push(CertificateArtifactBuffer {
            path: module.certificate.clone(),
            bytes: generated_bytes,
        });
    }
    None
}

fn check_local_certificate_files(
    loaded: &LoadedPackageRoot,
    certificates: &[LocalCertificateBuild],
) -> Option<CommandDiagnostic> {
    for certificate in certificates {
        let checked_in_bytes = match read_certificate_bytes(
            loaded,
            &certificate.path,
            format!("modules[{}].certificate", certificate.module_index),
        ) {
            Ok(bytes) => bytes,
            Err(diagnostic) => return Some(*diagnostic),
        };
        if checked_in_bytes != certificate.bytes {
            return Some(
                CommandDiagnostic::error(DiagnosticKind::Build, "build_certificate_changed")
                    .with_module(certificate.module.as_dotted())
                    .with_path(render_package_path(&certificate.path))
                    .with_hashes(
                        format_package_hash(&package_file_hash(&checked_in_bytes)),
                        format_package_hash(&package_file_hash(&certificate.bytes)),
                    ),
            );
        }
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

fn build_legacy_std_package_certificate(
    module_index: usize,
    module: &PackageModule,
    source: &str,
    direct_verified_modules: &[VerifiedModule],
    policy: &AxiomPolicy,
) -> Result<(ModuleCert, Vec<u8>, VerifiedModule, HumanSourceInterface), Box<CommandDiagnostic>> {
    validate_legacy_std_source_skeleton(module_index, module, source)?;
    let certificate =
        match build_legacy_std_package_module_cert(&module.module, direct_verified_modules) {
            Some(Ok(certificate)) => certificate,
            Some(Err(error)) => {
                return Err(Box::new(
                    CommandDiagnostic::error(DiagnosticKind::Build, "certificate_build_failed")
                        .with_module(module.module.as_dotted())
                        .with_path(format!("modules[{module_index}].certificate"))
                        .with_actual_value(format!("{error:?}")),
                ));
            }
            None => {
                return Err(Box::new(
                    CommandDiagnostic::error(
                        DiagnosticKind::Build,
                        "unsupported_legacy_std_module",
                    )
                    .with_module(module.module.as_dotted())
                    .with_path(format!("modules[{module_index}].producer_profile"))
                    .with_field("producer_profile")
                    .with_expected_value(LEGACY_STD_PACKAGE_PRODUCER_PROFILE)
                    .with_actual_value(
                        module
                            .producer_profile
                            .as_deref()
                            .unwrap_or("<missing-producer-profile>"),
                    ),
                ));
            }
        };
    let generated_bytes = npa_cert::encode_module_cert(&certificate).map_err(|error| {
        Box::new(
            CommandDiagnostic::error(DiagnosticKind::Build, "certificate_encode_failed")
                .with_module(module.module.as_dotted())
                .with_path(format!("modules[{module_index}].certificate"))
                .with_actual_value(format!("{error:?}")),
        )
    })?;

    let mut session = VerifierSession::new();
    for import in direct_verified_modules {
        session.register_verified_module(import.clone());
    }
    let verified =
        npa_cert::verify_module_cert(&generated_bytes, &mut session, policy).map_err(|error| {
            Box::new(
                CommandDiagnostic::error(DiagnosticKind::Build, "certificate_rejected")
                    .with_module(module.module.as_dotted())
                    .with_path(format!("modules[{module_index}].certificate"))
                    .with_actual_value(format!("{error:?}")),
            )
        })?;
    let source_interface = fallback_imported_source_interface(&verified).source_interface;
    Ok((certificate, generated_bytes, verified, source_interface))
}

fn validate_legacy_std_source_skeleton(
    module_index: usize,
    module: &PackageModule,
    source: &str,
) -> Result<(), Box<CommandDiagnostic>> {
    let file_id = match u32::try_from(module_index) {
        Ok(index) => FileId(index),
        Err(_) => {
            return Err(Box::new(
                CommandDiagnostic::error(DiagnosticKind::Internal, "module_index_out_of_range")
                    .with_module(module.module.as_dotted()),
            ));
        }
    };
    let parsed = parse_human_module(file_id, source).map_err(|error| {
        Box::new(
            CommandDiagnostic::error(DiagnosticKind::Build, "source_skeleton_parse_failed")
                .with_module(module.module.as_dotted())
                .with_path(format!("modules[{module_index}].source"))
                .with_field("source_skeleton")
                .with_actual_value(error.message),
        )
    })?;
    let mut actual_imports = Vec::new();
    for item in parsed.items {
        match item {
            HumanItem::Import { module, .. } => {
                actual_imports.push(Name::from_dotted(module.as_dotted()));
            }
            other => {
                return Err(Box::new(
                    CommandDiagnostic::error(DiagnosticKind::Build, "source_skeleton_has_items")
                        .with_module(module.module.as_dotted())
                        .with_path(format!("modules[{module_index}].source"))
                        .with_field("source_skeleton")
                        .with_expected_value("imports and comments only")
                        .with_actual_value(format!("{:?}", other.span())),
                ));
            }
        }
    }
    if actual_imports != module.imports {
        return Err(Box::new(
            CommandDiagnostic::error(DiagnosticKind::Build, "source_imports_mismatch")
                .with_module(module.module.as_dotted())
                .with_path(format!("modules[{module_index}].source"))
                .with_field("imports")
                .with_expected_value(format!("{:?}", module.imports))
                .with_actual_value(format!("{actual_imports:?}")),
        ));
    }
    Ok(())
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

fn write_package_build(
    loaded: &LoadedPackageRoot,
    build: &PackageCertificateBuild,
) -> Option<CommandDiagnostic> {
    let mut pending = Vec::new();
    for certificate in &build.local_certificates {
        match prepare_pending_write(
            &loaded.root,
            &certificate.path,
            format!("modules[{}].certificate", certificate.module_index),
            &certificate.bytes,
            "certificate_write_failed",
            Some(certificate.module.clone()),
        ) {
            Ok(Some(write)) => pending.push(write),
            Ok(None) => {}
            Err(diagnostic) => {
                cleanup_pending_writes(&pending);
                return Some(*diagnostic);
            }
        }
    }

    let lock_path = PackagePath::new(PACKAGE_LOCK_PATH);
    match prepare_pending_write(
        &loaded.root,
        &lock_path,
        "package_lock.path",
        build.package_lock_json.as_bytes(),
        "package_lock_write_failed",
        None,
    ) {
        Ok(Some(write)) => pending.push(write),
        Ok(None) => {}
        Err(diagnostic) => {
            cleanup_pending_writes(&pending);
            return Some(*diagnostic);
        }
    }

    commit_pending_writes(&pending)
}

fn prepare_pending_write(
    root: &Path,
    package_path: &PackagePath,
    manifest_field_path: impl Into<String>,
    bytes: &[u8],
    reason_code: &'static str,
    module: Option<Name>,
) -> Result<Option<PendingWrite>, Box<CommandDiagnostic>> {
    let full_path = join_package_path(root, package_path, manifest_field_path)?;
    match fs::read(&full_path) {
        Ok(existing) if existing == bytes => return Ok(None),
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(_) => {
            return Err(Box::new(write_artifact_diagnostic(
                reason_code,
                package_path,
                module.as_ref(),
            )));
        }
    }

    if let Some(parent) = full_path.parent() {
        if fs::create_dir_all(parent).is_err() {
            return Err(Box::new(write_artifact_diagnostic(
                reason_code,
                package_path,
                module.as_ref(),
            )));
        }
    }

    let temp_path = temporary_write_path(&full_path);
    if fs::write(&temp_path, bytes).is_err() {
        return Err(Box::new(write_artifact_diagnostic(
            reason_code,
            package_path,
            module.as_ref(),
        )));
    }

    Ok(Some(PendingWrite {
        path: package_path.clone(),
        full_path,
        temp_path,
        reason_code,
        module,
    }))
}

fn commit_pending_writes(pending: &[PendingWrite]) -> Option<CommandDiagnostic> {
    for write in pending {
        if fs::rename(&write.temp_path, &write.full_path).is_err() {
            cleanup_pending_writes(pending);
            return Some(write_artifact_diagnostic(
                write.reason_code,
                &write.path,
                write.module.as_ref(),
            ));
        }
    }
    None
}

fn cleanup_pending_writes(pending: &[PendingWrite]) {
    for write in pending {
        let _ = fs::remove_file(&write.temp_path);
    }
}

fn temporary_write_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("artifact");
    path.with_file_name(format!(".{file_name}.npa-build-certs.tmp"))
}

fn write_artifact_diagnostic(
    reason_code: &'static str,
    path: &PackagePath,
    module: Option<&Name>,
) -> CommandDiagnostic {
    let diagnostic =
        CommandDiagnostic::error(DiagnosticKind::ArtifactIo, reason_code).with_path(path.as_str());
    if let Some(module) = module {
        diagnostic.with_module(module.as_dotted())
    } else {
        diagnostic
    }
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
