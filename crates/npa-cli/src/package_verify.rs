//! Implementation of `npa package verify-certs`.

use std::{collections::BTreeMap, fs, io, thread};

use npa_api::{
    verify_package_fast_source_free, verify_package_reference_source_free,
    PackageCertificateArtifact, PackageModuleVerificationResult, PackageModuleVerificationStatus,
    PackageVerificationError, PackageVerificationErrorKind, PackageVerificationErrorReason,
    PackageVerificationMode, PackageVerificationReport, PackageVerificationStatus,
    PackageVerificationVerdictSource,
};
use npa_cert::Name;
use npa_package::{
    build_package_lock_from_artifacts, format_package_hash, package_file_hash,
    parse_package_lock_json, PackageLockArtifact, PackageLockEntry, PackageLockManifest,
    PackagePath,
};

use crate::args::{PackageChecker, PackageVerifyCertsOptions};
use crate::diagnostic::{CommandDiagnostic, CommandResult, DiagnosticKind};
use crate::fs::{join_package_path, render_package_path, render_package_root};
use crate::package::{load_package_root, LoadedPackageRoot};

const COMMAND: &str = "package verify-certs";
const PACKAGE_LOCK_PATH: &str = "generated/package-lock.json";
const PACKAGE_VERIFY_STACK_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Debug)]
struct CertificateArtifactBuffer {
    path: PackagePath,
    bytes: Vec<u8>,
}

/// Run source-free package certificate verification.
///
/// This command reads the package manifest, `generated/package-lock.json`, and
/// local/external certificate files. It intentionally does not read source,
/// replay, metadata, theorem-index, AI trace, registry, or checker-result
/// sidecars.
pub fn run_package_verify_certs(options: PackageVerifyCertsOptions) -> CommandResult {
    let root_display = render_package_root(&options.common.root);
    match thread::Builder::new()
        .name("npa-cli-package-verify-certs".to_owned())
        .stack_size(PACKAGE_VERIFY_STACK_BYTES)
        .spawn(move || run_package_verify_certs_on_stack(options))
    {
        Ok(handle) => match handle.join() {
            Ok(result) => result,
            Err(_) => CommandResult::failed(
                COMMAND,
                root_display,
                vec![CommandDiagnostic::error(
                    DiagnosticKind::Internal,
                    "verify_thread_panicked",
                )],
            ),
        },
        Err(error) => CommandResult::failed(
            COMMAND,
            root_display,
            vec![
                CommandDiagnostic::error(DiagnosticKind::Internal, "verify_thread_spawn_failed")
                    .with_actual_value(error.to_string()),
            ],
        ),
    }
}

fn run_package_verify_certs_on_stack(options: PackageVerifyCertsOptions) -> CommandResult {
    let checker = options.checker;
    let loaded = match load_package_root(&options.common.root, COMMAND) {
        Ok(loaded) => loaded,
        Err(result) => return result,
    };

    let (lock_source, checked_lock) = match read_package_lock(&loaded) {
        Ok(lock) => lock,
        Err(diagnostic) => {
            return CommandResult::failed(COMMAND, loaded.root_display, vec![*diagnostic]);
        }
    };

    let artifacts = match read_certificate_artifacts(&loaded) {
        Ok(artifacts) => artifacts,
        Err(diagnostic) => {
            return CommandResult::failed(COMMAND, loaded.root_display, vec![*diagnostic]);
        }
    };

    let regenerated_lock_json = match regenerated_package_lock_json(&loaded, &artifacts) {
        Ok(json) => json,
        Err(diagnostic) => {
            return CommandResult::failed(COMMAND, loaded.root_display, vec![*diagnostic]);
        }
    };

    if lock_source != regenerated_lock_json {
        return CommandResult::failed(
            COMMAND,
            loaded.root_display,
            vec![
                CommandDiagnostic::error(DiagnosticKind::HashMismatch, "package_lock_stale")
                    .with_path(PACKAGE_LOCK_PATH)
                    .with_hashes(
                        format_package_hash(&package_file_hash(regenerated_lock_json.as_bytes())),
                        format_package_hash(&package_file_hash(lock_source.as_bytes())),
                    ),
            ],
        );
    }

    let report = match verify_package(checker, &loaded, &checked_lock, &artifacts) {
        Ok(report) => report,
        Err(error) => {
            return CommandResult::failed(
                COMMAND,
                loaded.root_display,
                vec![verification_error_diagnostic(
                    &error,
                    None,
                    checker_diagnostic_kind(checker),
                    checker_label(checker),
                )],
            );
        }
    };

    command_result_from_report(loaded.root_display, &checked_lock, report)
}

fn read_package_lock(
    loaded: &LoadedPackageRoot,
) -> Result<(String, PackageLockManifest), Box<CommandDiagnostic>> {
    let lock_path = PackagePath::new(PACKAGE_LOCK_PATH);
    let full_lock_path = join_package_path(&loaded.root, &lock_path, "package_lock.path")?;
    let lock_source = match fs::read_to_string(&full_lock_path) {
        Ok(source) => source,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Err(Box::new(
                CommandDiagnostic::error(DiagnosticKind::PackageLock, "package_lock_missing")
                    .with_path(PACKAGE_LOCK_PATH),
            ));
        }
        Err(_) => {
            return Err(Box::new(
                CommandDiagnostic::error(DiagnosticKind::ArtifactIo, "package_lock_missing")
                    .with_path(PACKAGE_LOCK_PATH),
            ));
        }
    };
    let lock = parse_package_lock_json(&lock_source).map_err(|error| {
        Box::new(CommandDiagnostic::from_package_lock_error(&error).with_path(PACKAGE_LOCK_PATH))
    })?;
    Ok((lock_source, lock))
}

fn read_certificate_artifacts(
    loaded: &LoadedPackageRoot,
) -> Result<Vec<CertificateArtifactBuffer>, Box<CommandDiagnostic>> {
    let mut artifacts = Vec::new();
    for (index, module) in loaded.validated.manifest().modules.iter().enumerate() {
        let bytes = read_certificate_bytes(
            loaded,
            &module.certificate,
            format!("modules[{index}].certificate"),
            Some(&module.module),
        )?;
        artifacts.push(CertificateArtifactBuffer {
            path: module.certificate.clone(),
            bytes,
        });
    }
    for (index, import) in loaded
        .validated
        .manifest()
        .imports
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .enumerate()
    {
        let bytes = read_certificate_bytes(
            loaded,
            &import.certificate,
            format!("imports[{index}].certificate"),
            Some(&import.module),
        )?;
        artifacts.push(CertificateArtifactBuffer {
            path: import.certificate.clone(),
            bytes,
        });
    }
    Ok(artifacts)
}

fn read_certificate_bytes(
    loaded: &LoadedPackageRoot,
    package_path: &PackagePath,
    manifest_field_path: impl Into<String>,
    module: Option<&Name>,
) -> Result<Vec<u8>, Box<CommandDiagnostic>> {
    let full_path = join_package_path(&loaded.root, package_path, manifest_field_path)?;
    fs::read(full_path).map_err(|_| {
        let mut diagnostic =
            CommandDiagnostic::error(DiagnosticKind::ArtifactIo, "certificate_missing")
                .with_path(render_package_path(package_path));
        if let Some(module) = module {
            diagnostic = diagnostic.with_module(module.as_dotted());
        }
        Box::new(diagnostic)
    })
}

fn regenerated_package_lock_json(
    loaded: &LoadedPackageRoot,
    artifacts: &[CertificateArtifactBuffer],
) -> Result<String, Box<CommandDiagnostic>> {
    let lock = build_package_lock_from_artifacts(
        &loaded.validated,
        loaded.manifest_path.clone(),
        loaded.manifest_source.as_bytes(),
        artifacts.iter().map(|artifact| PackageLockArtifact {
            path: artifact.path.clone(),
            bytes: artifact.bytes.as_slice(),
        }),
    )
    .map_err(|error| Box::new(CommandDiagnostic::from_package_lock_error(&error)))?;
    lock.canonical_json()
        .map_err(|error| Box::new(CommandDiagnostic::from_package_lock_error(&error)))
}

fn verify_package(
    checker: PackageChecker,
    loaded: &LoadedPackageRoot,
    lock: &PackageLockManifest,
    artifacts: &[CertificateArtifactBuffer],
) -> Result<PackageVerificationReport, PackageVerificationError> {
    match checker {
        PackageChecker::Reference => verify_package_reference_source_free(
            &loaded.validated,
            lock,
            package_certificate_artifacts(artifacts),
        ),
        PackageChecker::Fast => verify_package_fast_source_free(
            &loaded.validated,
            lock,
            package_certificate_artifacts(artifacts),
        ),
    }
}

fn package_certificate_artifacts(
    artifacts: &[CertificateArtifactBuffer],
) -> Vec<PackageCertificateArtifact<'_>> {
    artifacts
        .iter()
        .map(|artifact| PackageCertificateArtifact {
            path: artifact.path.clone(),
            bytes: artifact.bytes.as_slice(),
        })
        .collect()
}

fn command_result_from_report(
    root_display: String,
    lock: &PackageLockManifest,
    report: PackageVerificationReport,
) -> CommandResult {
    if report.status == PackageVerificationStatus::Passed {
        let mut result = CommandResult::passed(COMMAND, root_display);
        result.diagnostics = passed_report_diagnostics(lock, &report);
        return result;
    }

    let diagnostics = failed_report_diagnostics(&report);
    CommandResult::failed(COMMAND, root_display, diagnostics)
}

fn passed_report_diagnostics(
    lock: &PackageLockManifest,
    report: &PackageVerificationReport,
) -> Vec<CommandDiagnostic> {
    let entries_by_module = lock_entries_by_module(lock);
    let mut diagnostics = vec![aggregate_report_diagnostic(report)];
    diagnostics.extend(report.modules.iter().map(|module| {
        let path = entries_by_module
            .get(&module.module)
            .map(|entry| entry.certificate.as_str())
            .unwrap_or("<unknown-certificate>");
        CommandDiagnostic::info(
            diagnostic_kind_for_mode(module.checker_mode),
            "module_verified",
        )
        .with_module(module.module.as_dotted())
        .with_path(path)
        .with_field("status")
        .with_expected_value(PackageModuleVerificationStatus::Passed.as_str())
        .with_actual_value(module.status.as_str())
        .with_checker(report.verdict_source.as_str())
    }));
    diagnostics
}

fn aggregate_report_diagnostic(report: &PackageVerificationReport) -> CommandDiagnostic {
    CommandDiagnostic::info(diagnostic_kind_for_mode(report.mode), "package_verified")
        .with_field("verdict_source")
        .with_actual_value(format!(
            "mode={};verdict_source={};reference_checker_verdict={};modules={}",
            report.mode.as_str(),
            report.verdict_source.as_str(),
            report.reference_checker_verdict,
            report.modules.len()
        ))
        .with_checker(report.verdict_source.as_str())
}

fn failed_report_diagnostics(report: &PackageVerificationReport) -> Vec<CommandDiagnostic> {
    let kind = diagnostic_kind_for_mode(report.mode);
    let checker = report.verdict_source.as_str();
    let diagnostics = report
        .modules
        .iter()
        .filter_map(|module| {
            module
                .error
                .as_ref()
                .map(|error| verification_error_diagnostic(error, Some(module), kind, checker))
        })
        .collect::<Vec<_>>();
    if diagnostics.is_empty() {
        vec![CommandDiagnostic::error(
            DiagnosticKind::Internal,
            "verification_failed_without_error",
        )
        .with_checker(checker)]
    } else {
        diagnostics
    }
}

fn verification_error_diagnostic(
    error: &PackageVerificationError,
    module: Option<&PackageModuleVerificationResult>,
    fallback_kind: DiagnosticKind,
    fallback_checker: &str,
) -> CommandDiagnostic {
    let kind = diagnostic_kind_for_error(error).unwrap_or(fallback_kind);
    let mut diagnostic = CommandDiagnostic::error(kind, error.reason_code.as_str())
        .with_path(error.path.clone())
        .with_checker(
            error
                .checker_error
                .as_ref()
                .map(|checker| checker.checker.as_str())
                .unwrap_or(fallback_checker),
        );
    if let Some(field) = &error.field {
        diagnostic = diagnostic.with_field(field.clone());
    }
    if let Some(module) = module {
        diagnostic = diagnostic.with_module(module.module.as_dotted());
    }
    if is_hash_mismatch_reason(error.reason_code) {
        if let (Some(expected), Some(actual)) = (&error.expected_value, &error.actual_value) {
            diagnostic = diagnostic.with_hashes(expected.clone(), actual.clone());
        }
    } else {
        if let Some(expected) = &error.expected_value {
            diagnostic = diagnostic.with_expected_value(expected.clone());
        }
        if let Some(actual) = &error.actual_value {
            diagnostic = diagnostic.with_actual_value(actual.clone());
        }
    }
    diagnostic
}

fn diagnostic_kind_for_error(error: &PackageVerificationError) -> Option<DiagnosticKind> {
    Some(match error.kind {
        PackageVerificationErrorKind::Input => DiagnosticKind::PackageLock,
        PackageVerificationErrorKind::LockGraph => DiagnosticKind::PackageGraph,
        PackageVerificationErrorKind::Artifact => DiagnosticKind::ArtifactIo,
        PackageVerificationErrorKind::CertificateDecode => DiagnosticKind::SourceFreeBoundary,
        PackageVerificationErrorKind::CertificateIdentity => DiagnosticKind::HashMismatch,
        PackageVerificationErrorKind::Kernel => DiagnosticKind::FastVerifier,
        PackageVerificationErrorKind::ReferenceChecker => DiagnosticKind::ReferenceVerifier,
        PackageVerificationErrorKind::Phase8Adapter => DiagnosticKind::SourceFreeBoundary,
        PackageVerificationErrorKind::Dependency => return None,
    })
}

fn is_hash_mismatch_reason(reason: PackageVerificationErrorReason) -> bool {
    matches!(
        reason,
        PackageVerificationErrorReason::CertificateFileHashMismatch
            | PackageVerificationErrorReason::ExportHashMismatch
            | PackageVerificationErrorReason::AxiomReportHashMismatch
            | PackageVerificationErrorReason::CertificateHashMismatch
    )
}

fn diagnostic_kind_for_mode(mode: PackageVerificationMode) -> DiagnosticKind {
    match mode {
        PackageVerificationMode::FastKernel => DiagnosticKind::FastVerifier,
        PackageVerificationMode::Reference => DiagnosticKind::ReferenceVerifier,
    }
}

fn checker_diagnostic_kind(checker: PackageChecker) -> DiagnosticKind {
    match checker {
        PackageChecker::Reference => DiagnosticKind::ReferenceVerifier,
        PackageChecker::Fast => DiagnosticKind::FastVerifier,
    }
}

fn checker_label(checker: PackageChecker) -> &'static str {
    match checker {
        PackageChecker::Reference => PackageVerificationVerdictSource::ReferenceChecker.as_str(),
        PackageChecker::Fast => {
            PackageVerificationVerdictSource::FastKernelCertificateVerifier.as_str()
        }
    }
}

fn lock_entries_by_module(lock: &PackageLockManifest) -> BTreeMap<Name, &PackageLockEntry> {
    lock.entries
        .iter()
        .map(|entry| (entry.module.clone(), entry))
        .collect()
}
