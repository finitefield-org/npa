//! Source-free package artifact extraction loading for CLR-05 commands.

use std::{fs, io, path::Path};

use npa_api::{
    build_package_audit_snapshot_source_free, extract_package_artifacts_source_free,
    PackageArtifactExtraction, PackageArtifactExtractionInput, PackageArtifactReferenceSummaryMode,
    PackageAuditCertificateInput, PackageAuditSnapshot, PackageAuditSnapshotBuildError,
    PackageAuditSnapshotInput, PackageCertificateArtifact, PackageVerificationError,
    PackageVerificationErrorKind, PackageVerificationErrorReason, PackageVerificationVerdictSource,
};
use npa_cert::Name;
use npa_package::{
    package_file_hash, parse_package_lock_json, PackageArtifactError, PackageArtifactFileReference,
    PackageLockManifest, PackagePath, ValidatedPackageManifest,
};

use crate::diagnostic::{CommandDiagnostic, CommandResult, DiagnosticKind};
use crate::fs::{join_package_path, render_package_path};
use crate::package::{load_package_root, LoadedPackageRoot};
use crate::timing::{
    PackageTimingCollector, TIMING_ARTIFACT_COMPARE_MS, TIMING_CHECKER_MS,
    TIMING_DECODE_CERTIFICATES_MS, TIMING_LOAD_LOCK_MS, TIMING_LOAD_ROOT_MS,
};

/// Package-relative path to the generated package lock.
pub const PACKAGE_LOCK_PATH: &str = "generated/package-lock.json";
/// Package-relative path to the generated package axiom report.
pub const PACKAGE_AXIOM_REPORT_PATH: &str = "generated/axiom-report.json";
/// Package-relative path to the generated package theorem index.
pub const PACKAGE_THEOREM_INDEX_PATH: &str = "generated/theorem-index.json";

#[derive(Clone, Debug)]
struct CertificateArtifactBuffer {
    path: PackagePath,
    bytes: Vec<u8>,
}

/// Which checked generated CLR-05 artifacts should be read from disk.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PackageGeneratedArtifactReadMode {
    /// Read `generated/axiom-report.json`.
    pub axiom_report: bool,
    /// Read `generated/theorem-index.json`.
    pub theorem_index: bool,
}

impl PackageGeneratedArtifactReadMode {
    /// Do not read checked generated CLR-05 artifacts.
    pub const fn none() -> Self {
        Self {
            axiom_report: false,
            theorem_index: false,
        }
    }

    /// Read both checked generated CLR-05 artifacts.
    pub const fn all() -> Self {
        Self {
            axiom_report: true,
            theorem_index: true,
        }
    }
}

/// Checked generated artifacts loaded only for check-mode comparisons.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckedGeneratedPackageArtifacts {
    /// Checked-in axiom report JSON, when requested.
    pub axiom_report_json: Option<String>,
    /// Checked-in theorem index JSON, when requested.
    pub theorem_index_json: Option<String>,
}

/// Loaded source-free extraction output and optional checked generated artifacts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadedPackageArtifactExtraction {
    /// Sanitized package root display string for diagnostics.
    pub root_display: String,
    /// Validated package manifest used for extraction.
    pub validated: ValidatedPackageManifest,
    /// Checked package-lock JSON bytes loaded from disk.
    pub package_lock_json: String,
    /// Parsed checked package-lock manifest loaded from disk.
    pub package_lock_manifest: PackageLockManifest,
    /// Exact package-lock file identity used for extraction.
    pub package_lock: PackageArtifactFileReference,
    /// Source-free extraction output for later artifact projection.
    pub extraction: PackageArtifactExtraction,
    /// Checked generated artifacts requested by check mode.
    pub checked_generated: CheckedGeneratedPackageArtifacts,
}

/// Loaded process-local package audit snapshot and optional checked generated artifacts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadedPackageAuditSnapshot {
    /// Sanitized package root display string for diagnostics.
    pub root_display: String,
    /// Checked package-lock JSON bytes loaded from disk.
    pub package_lock_json: String,
    /// Source-free package audit snapshot for later artifact projection.
    pub snapshot: PackageAuditSnapshot,
    /// Checked generated artifacts requested by check mode.
    pub checked_generated: CheckedGeneratedPackageArtifacts,
}

/// Load and verify source-free inputs for CLR-05 package artifact commands.
///
/// This reads `npa-package.toml`, `generated/package-lock.json`, local and
/// external certificate artifacts, and optionally the checked generated CLR-05
/// artifacts. It does not read source, replay, metadata, AI trace, registry, or
/// theorem-search sidecars.
pub fn load_package_artifact_extraction(
    root: impl AsRef<Path>,
    command: impl Into<String>,
    generated_read_mode: PackageGeneratedArtifactReadMode,
    reference_summaries: PackageArtifactReferenceSummaryMode,
) -> Result<LoadedPackageArtifactExtraction, CommandResult> {
    let command = command.into();
    load_package_artifact_extraction_impl(
        root.as_ref(),
        command,
        generated_read_mode,
        reference_summaries,
        None,
    )
}

pub(crate) fn load_package_artifact_extraction_with_timings(
    root: impl AsRef<Path>,
    command: impl Into<String>,
    generated_read_mode: PackageGeneratedArtifactReadMode,
    reference_summaries: PackageArtifactReferenceSummaryMode,
    timings: &mut PackageTimingCollector,
) -> Result<LoadedPackageArtifactExtraction, CommandResult> {
    let command = command.into();
    load_package_artifact_extraction_impl(
        root.as_ref(),
        command,
        generated_read_mode,
        reference_summaries,
        Some(timings),
    )
}

/// Load a reusable package audit snapshot for combined in-process projections.
pub fn load_package_audit_snapshot(
    root: impl AsRef<Path>,
    command: impl Into<String>,
    generated_read_mode: PackageGeneratedArtifactReadMode,
    reference_summaries: PackageArtifactReferenceSummaryMode,
) -> Result<LoadedPackageAuditSnapshot, CommandResult> {
    let command = command.into();
    load_package_audit_snapshot_impl(
        root.as_ref(),
        command,
        generated_read_mode,
        reference_summaries,
        None,
    )
}

pub(crate) fn load_package_audit_snapshot_with_timings(
    root: impl AsRef<Path>,
    command: impl Into<String>,
    generated_read_mode: PackageGeneratedArtifactReadMode,
    reference_summaries: PackageArtifactReferenceSummaryMode,
    timings: &mut PackageTimingCollector,
) -> Result<LoadedPackageAuditSnapshot, CommandResult> {
    let command = command.into();
    load_package_audit_snapshot_impl(
        root.as_ref(),
        command,
        generated_read_mode,
        reference_summaries,
        Some(timings),
    )
}

fn load_package_artifact_extraction_impl(
    root: &Path,
    command: String,
    generated_read_mode: PackageGeneratedArtifactReadMode,
    reference_summaries: PackageArtifactReferenceSummaryMode,
    mut timings: Option<&mut PackageTimingCollector>,
) -> Result<LoadedPackageArtifactExtraction, CommandResult> {
    let loaded = match timings.as_mut() {
        Some(timings) => timings.time_phase(TIMING_LOAD_ROOT_MS, || {
            load_package_root(root, command.clone())
        }),
        None => load_package_root(root, command.clone()),
    }?;
    let lock_result = match timings.as_mut() {
        Some(timings) => timings.time_phase(TIMING_LOAD_LOCK_MS, || read_package_lock(&loaded)),
        None => read_package_lock(&loaded),
    };
    let (lock_source, lock) = match lock_result {
        Ok(lock) => lock,
        Err(diagnostic) => {
            return Err(CommandResult::failed(
                command,
                loaded.root_display,
                vec![*diagnostic],
            ));
        }
    };
    let package_lock = PackageArtifactFileReference {
        path: PackagePath::new(PACKAGE_LOCK_PATH),
        file_hash: package_file_hash(lock_source.as_bytes()),
    };
    let certificates_result = match timings.as_mut() {
        Some(timings) => timings.time_phase(TIMING_DECODE_CERTIFICATES_MS, || {
            read_certificate_artifacts(&loaded)
        }),
        None => read_certificate_artifacts(&loaded),
    };
    let certificates = match certificates_result {
        Ok(certificates) => certificates,
        Err(diagnostic) => {
            return Err(CommandResult::failed(
                command,
                loaded.root_display,
                vec![*diagnostic],
            ));
        }
    };
    let certificate_artifacts = package_certificate_artifacts(&certificates);
    let extraction_result = match timings.as_mut() {
        Some(timings) => timings.time_phase(TIMING_CHECKER_MS, || {
            extract_package_artifacts_source_free(PackageArtifactExtractionInput {
                validated: &loaded.validated,
                manifest_path: loaded.manifest_path.clone(),
                manifest_bytes: loaded.manifest_source.as_bytes(),
                package_lock: &lock,
                certificates: certificate_artifacts,
                reference_summaries,
            })
        }),
        None => extract_package_artifacts_source_free(PackageArtifactExtractionInput {
            validated: &loaded.validated,
            manifest_path: loaded.manifest_path.clone(),
            manifest_bytes: loaded.manifest_source.as_bytes(),
            package_lock: &lock,
            certificates: certificate_artifacts,
            reference_summaries,
        }),
    };
    let extraction = match extraction_result {
        Ok(extraction) => extraction,
        Err(error) => {
            return Err(CommandResult::failed(
                command,
                loaded.root_display,
                vec![extraction_error_diagnostic(&error)],
            ));
        }
    };
    let checked_generated_result = match timings.as_mut() {
        Some(timings) => timings.time_phase(TIMING_ARTIFACT_COMPARE_MS, || {
            read_checked_generated_artifacts(&loaded, generated_read_mode)
        }),
        None => read_checked_generated_artifacts(&loaded, generated_read_mode),
    };
    let checked_generated = match checked_generated_result {
        Ok(artifacts) => artifacts,
        Err(diagnostic) => {
            return Err(CommandResult::failed(
                command,
                loaded.root_display,
                vec![*diagnostic],
            ));
        }
    };

    Ok(LoadedPackageArtifactExtraction {
        root_display: loaded.root_display,
        validated: loaded.validated,
        package_lock_json: lock_source,
        package_lock_manifest: lock,
        package_lock,
        extraction,
        checked_generated,
    })
}

fn load_package_audit_snapshot_impl(
    root: &Path,
    command: String,
    generated_read_mode: PackageGeneratedArtifactReadMode,
    reference_summaries: PackageArtifactReferenceSummaryMode,
    mut timings: Option<&mut PackageTimingCollector>,
) -> Result<LoadedPackageAuditSnapshot, CommandResult> {
    let loaded = match timings.as_mut() {
        Some(timings) => timings.time_phase(TIMING_LOAD_ROOT_MS, || {
            load_package_root(root, command.clone())
        }),
        None => load_package_root(root, command.clone()),
    }?;
    let lock_result = match timings.as_mut() {
        Some(timings) => timings.time_phase(TIMING_LOAD_LOCK_MS, || read_package_lock(&loaded)),
        None => read_package_lock(&loaded),
    };
    let (lock_source, lock) = match lock_result {
        Ok(lock) => lock,
        Err(diagnostic) => {
            return Err(CommandResult::failed(
                command,
                loaded.root_display,
                vec![*diagnostic],
            ));
        }
    };
    let package_lock = PackageArtifactFileReference {
        path: PackagePath::new(PACKAGE_LOCK_PATH),
        file_hash: package_file_hash(lock_source.as_bytes()),
    };
    let certificates_result = match timings.as_mut() {
        Some(timings) => timings.time_phase(TIMING_DECODE_CERTIFICATES_MS, || {
            read_certificate_artifacts(&loaded)
        }),
        None => read_certificate_artifacts(&loaded),
    };
    let certificates = match certificates_result {
        Ok(certificates) => certificates,
        Err(diagnostic) => {
            return Err(CommandResult::failed(
                command,
                loaded.root_display,
                vec![*diagnostic],
            ));
        }
    };
    let certificate_inputs = package_audit_certificate_inputs(certificates);
    let snapshot_result = match timings.as_mut() {
        Some(timings) => timings.time_phase(TIMING_CHECKER_MS, || {
            build_package_audit_snapshot_source_free(PackageAuditSnapshotInput {
                validated: &loaded.validated,
                manifest_path: loaded.manifest_path.clone(),
                manifest_bytes: loaded.manifest_source.as_bytes(),
                package_lock_manifest: &lock,
                package_lock: package_lock.clone(),
                certificates: certificate_inputs,
                reference_summaries,
            })
        }),
        None => build_package_audit_snapshot_source_free(PackageAuditSnapshotInput {
            validated: &loaded.validated,
            manifest_path: loaded.manifest_path.clone(),
            manifest_bytes: loaded.manifest_source.as_bytes(),
            package_lock_manifest: &lock,
            package_lock: package_lock.clone(),
            certificates: certificate_inputs,
            reference_summaries,
        }),
    };
    let snapshot = match snapshot_result {
        Ok(snapshot) => snapshot,
        Err(error) => {
            return Err(CommandResult::failed(
                command,
                loaded.root_display,
                vec![snapshot_build_error_diagnostic(&error)],
            ));
        }
    };
    let checked_generated_result = match timings.as_mut() {
        Some(timings) => timings.time_phase(TIMING_ARTIFACT_COMPARE_MS, || {
            read_checked_generated_artifacts(&loaded, generated_read_mode)
        }),
        None => read_checked_generated_artifacts(&loaded, generated_read_mode),
    };
    let checked_generated = match checked_generated_result {
        Ok(artifacts) => artifacts,
        Err(diagnostic) => {
            return Err(CommandResult::failed(
                command,
                loaded.root_display,
                vec![*diagnostic],
            ));
        }
    };

    Ok(LoadedPackageAuditSnapshot {
        root_display: loaded.root_display,
        package_lock_json: lock_source,
        snapshot,
        checked_generated,
    })
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

fn package_audit_certificate_inputs(
    artifacts: Vec<CertificateArtifactBuffer>,
) -> Vec<PackageAuditCertificateInput> {
    artifacts
        .into_iter()
        .map(|artifact| PackageAuditCertificateInput {
            path: artifact.path,
            bytes: artifact.bytes,
        })
        .collect()
}

fn read_checked_generated_artifacts(
    loaded: &LoadedPackageRoot,
    mode: PackageGeneratedArtifactReadMode,
) -> Result<CheckedGeneratedPackageArtifacts, Box<CommandDiagnostic>> {
    Ok(CheckedGeneratedPackageArtifacts {
        axiom_report_json: if mode.axiom_report {
            Some(read_generated_artifact(
                loaded,
                PACKAGE_AXIOM_REPORT_PATH,
                DiagnosticKind::AxiomReport,
                "axiom_report_missing",
            )?)
        } else {
            None
        },
        theorem_index_json: if mode.theorem_index {
            Some(read_generated_artifact(
                loaded,
                PACKAGE_THEOREM_INDEX_PATH,
                DiagnosticKind::TheoremIndex,
                "theorem_index_missing",
            )?)
        } else {
            None
        },
    })
}

fn read_generated_artifact(
    loaded: &LoadedPackageRoot,
    package_path: &str,
    kind: DiagnosticKind,
    missing_reason: &str,
) -> Result<String, Box<CommandDiagnostic>> {
    let package_path = PackagePath::new(package_path);
    let full_path = join_package_path(&loaded.root, &package_path, "generated_artifact.path")?;
    fs::read_to_string(full_path).map_err(|error| {
        let reason = if error.kind() == io::ErrorKind::NotFound {
            missing_reason
        } else {
            "generated_artifact_read_failed"
        };
        Box::new(CommandDiagnostic::error(kind, reason).with_path(package_path.as_str()))
    })
}

fn snapshot_build_error_diagnostic(error: &PackageAuditSnapshotBuildError) -> CommandDiagnostic {
    match error {
        PackageAuditSnapshotBuildError::Verification(error) => extraction_error_diagnostic(error),
        PackageAuditSnapshotBuildError::Artifact(error) => {
            snapshot_artifact_error_diagnostic(error)
        }
    }
}

fn snapshot_artifact_error_diagnostic(error: &PackageArtifactError) -> CommandDiagnostic {
    let mut diagnostic = CommandDiagnostic::error(
        DiagnosticKind::GeneratedArtifact,
        error.reason_code.as_str(),
    )
    .with_path(error.path.clone());
    if let Some(field) = &error.field {
        diagnostic = diagnostic.with_field(field.clone());
    }
    if let Some(expected) = &error.expected_value {
        diagnostic = diagnostic.with_expected_value(expected.clone());
    }
    if let Some(actual) = &error.actual_value {
        diagnostic = diagnostic.with_actual_value(actual.clone());
    }
    diagnostic
}

fn extraction_error_diagnostic(error: &PackageVerificationError) -> CommandDiagnostic {
    let reason_code = if error.reason_code == PackageVerificationErrorReason::AxiomPolicyRejected {
        "axiom_report_policy_violation"
    } else {
        error.reason_code.as_str()
    };
    let mut diagnostic = CommandDiagnostic::error(diagnostic_kind_for_error(error), reason_code)
        .with_path(error.path.clone());
    if let Some(field) = &error.field {
        diagnostic = diagnostic.with_field(field.clone());
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
    diagnostic.with_checker(
        error
            .checker_error
            .as_ref()
            .map(|checker| checker.checker.as_str())
            .unwrap_or_else(|| fallback_checker(error).as_str()),
    )
}

fn diagnostic_kind_for_error(error: &PackageVerificationError) -> DiagnosticKind {
    if error.reason_code == PackageVerificationErrorReason::AxiomPolicyRejected {
        return DiagnosticKind::PackagePolicy;
    }
    match error.kind {
        PackageVerificationErrorKind::Input => DiagnosticKind::PackageLock,
        PackageVerificationErrorKind::LockGraph => DiagnosticKind::PackageGraph,
        PackageVerificationErrorKind::Artifact => DiagnosticKind::ArtifactIo,
        PackageVerificationErrorKind::CertificateDecode => DiagnosticKind::SourceFreeBoundary,
        PackageVerificationErrorKind::CertificateIdentity => DiagnosticKind::HashMismatch,
        PackageVerificationErrorKind::Kernel => DiagnosticKind::FastVerifier,
        PackageVerificationErrorKind::ReferenceChecker => DiagnosticKind::ReferenceVerifier,
        PackageVerificationErrorKind::Phase8Adapter => DiagnosticKind::SourceFreeBoundary,
        PackageVerificationErrorKind::Dependency => DiagnosticKind::SourceFreeBoundary,
    }
}

fn is_hash_mismatch_reason(reason: PackageVerificationErrorReason) -> bool {
    matches!(
        reason,
        PackageVerificationErrorReason::PackageLockStale
            | PackageVerificationErrorReason::CertificateFileHashMismatch
            | PackageVerificationErrorReason::ExportHashMismatch
            | PackageVerificationErrorReason::AxiomReportHashMismatch
            | PackageVerificationErrorReason::CertificateHashMismatch
    )
}

fn fallback_checker(error: &PackageVerificationError) -> PackageVerificationVerdictSource {
    match error.kind {
        PackageVerificationErrorKind::ReferenceChecker => {
            PackageVerificationVerdictSource::ReferenceChecker
        }
        _ => PackageVerificationVerdictSource::FastKernelCertificateVerifier,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicUsize, Ordering},
    };

    use npa_api::{
        project_package_axiom_report_from_extraction,
        project_package_theorem_index_from_extraction,
        project_package_verified_export_summary_from_extraction,
        PackageArtifactReferenceSummaryMode,
    };
    use npa_package::{
        build_package_lock_from_artifacts, parse_and_validate_manifest_str, PackageLockArtifact,
        PackagePath,
    };

    use super::*;
    use crate::package::PACKAGE_MANIFEST_PATH;

    static TEST_DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

    struct TestDir {
        path: PathBuf,
    }

    impl TestDir {
        fn new(label: &str) -> Self {
            let index = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "npa-cli-package-artifact-boundary-{}-{label}-{index}",
                std::process::id()
            ));
            if path.exists() {
                fs::remove_dir_all(&path).unwrap();
            }
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }

        fn artifact_path(&self, relative: &str) -> PathBuf {
            self.path.join(relative)
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("npa-cli crate lives under crates/")
            .to_path_buf()
    }

    fn source_free_fixture(label: &str) -> TestDir {
        let dir = TestDir::new(label);
        let manifest_source = basic_manifest_source();
        write_file(dir.artifact_path(PACKAGE_MANIFEST_PATH), &manifest_source);
        let certificate_bytes =
            fs::read(repo_root().join("proofs").join(BASIC_CERTIFICATE_PATH)).unwrap();
        write_bytes(
            dir.artifact_path(BASIC_CERTIFICATE_PATH),
            certificate_bytes.as_slice(),
        );
        let validated = parse_and_validate_manifest_str(&manifest_source).unwrap();
        let lock = build_package_lock_from_artifacts(
            &validated,
            PackagePath::new(PACKAGE_MANIFEST_PATH),
            manifest_source.as_bytes(),
            [PackageLockArtifact {
                path: PackagePath::new(BASIC_CERTIFICATE_PATH),
                bytes: certificate_bytes.as_slice(),
            }],
        )
        .unwrap();
        write_file(
            dir.artifact_path(PACKAGE_LOCK_PATH),
            &lock.canonical_json().unwrap(),
        );
        dir
    }

    const BASIC_CERTIFICATE_PATH: &str = "Proofs/Ai/Basic/certificate.npcert";

    fn basic_manifest_source() -> String {
        r#"schema = "npa.package.v0.1"
package = "fixture-package"
version = "0.1.0"
core_spec = "npa.core.v0.1"
kernel_profile = "npa.kernel.v0.1"
certificate_format = "npa.certificate.canonical.v0.1"
checker_profile = "npa.checker.reference.v0.1"

[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]

[[modules]]
module = "Proofs.Ai.Basic"
source = "missing/source/Proofs/Ai/Basic.npa"
certificate = "Proofs/Ai/Basic/certificate.npcert"
meta = "missing/meta/Proofs/Ai/Basic.json"
replay = "missing/replay/Proofs/Ai/Basic.json"
producer_profile = "human-surface-explicit-term"
expected_source_hash = "sha256:28330ae585898b77be110adcdd53fe50e7f141a54113f12e6af9143fa4fcf54e"
expected_certificate_file_hash = "sha256:464a0d224b8667e4870888522454782231cd2cdd9049e6fa930cbefa62c18ffc"
expected_export_hash = "sha256:3341d28e9d1d9dd875138399ab1bd7aa6e2727449cb87fe03c73b220c4b231c0"
expected_axiom_report_hash = "sha256:fed11e73accfbfb0dfc28b4f510e151fa33d8af82d58fdb23b92567e04e59e40"
expected_certificate_hash = "sha256:69cb8c64c6ce722209e27820cd790af6d325c98478b3599ae796ee03df528b13"
imports = []
definitions = []
theorems = ["id"]
axioms = []
"#
        .to_owned()
    }

    fn write_file(path: PathBuf, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }

    fn write_bytes(path: PathBuf, contents: &[u8]) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }

    #[test]
    fn package_projection_snapshot_reuses_one_snapshot_for_all_projection_artifacts() {
        let fixture = source_free_fixture("projection-snapshot");
        let standalone = load_package_artifact_extraction(
            fixture.path(),
            "package projection snapshot standalone",
            PackageGeneratedArtifactReadMode::none(),
            PackageArtifactReferenceSummaryMode::Omit,
        )
        .unwrap();
        let standalone_axiom_report_json = project_package_axiom_report_from_extraction(
            &standalone.validated,
            &standalone.extraction,
            standalone.package_lock.clone(),
        )
        .and_then(|report| report.canonical_json())
        .unwrap();
        let standalone_theorem_index_json = project_package_theorem_index_from_extraction(
            &standalone.validated,
            &standalone.extraction,
            standalone.package_lock.clone(),
        )
        .and_then(|index| index.canonical_json())
        .unwrap();
        let standalone_export_summary_json =
            project_package_verified_export_summary_from_extraction(
                &standalone.validated,
                &standalone.package_lock_manifest,
                standalone.package_lock.clone(),
                &standalone.extraction,
            )
            .and_then(|summary| summary.canonical_json())
            .unwrap();

        write_file(
            fixture.artifact_path(PACKAGE_AXIOM_REPORT_PATH),
            &standalone_axiom_report_json,
        );
        write_file(
            fixture.artifact_path(PACKAGE_THEOREM_INDEX_PATH),
            &standalone_theorem_index_json,
        );

        let shared = load_package_audit_snapshot(
            fixture.path(),
            "package projection snapshot shared",
            PackageGeneratedArtifactReadMode::all(),
            PackageArtifactReferenceSummaryMode::Include,
        )
        .unwrap();
        assert_eq!(shared.snapshot.certificate_artifacts.len(), 1);
        assert!(shared.snapshot.reference_verification_report.is_some());
        assert_eq!(
            shared
                .snapshot
                .projection_input_hashes
                .package_lock_file_hash,
            shared.snapshot.package_lock.file_hash
        );

        let shared_axiom_report_json = shared
            .snapshot
            .project_axiom_report()
            .and_then(|report| report.canonical_json())
            .unwrap();
        let shared_theorem_index_json = shared
            .snapshot
            .project_theorem_index()
            .and_then(|index| index.canonical_json())
            .unwrap();
        let shared_export_summary_json = shared
            .snapshot
            .project_verified_export_summary()
            .and_then(|summary| summary.canonical_json())
            .unwrap();
        assert_eq!(shared_axiom_report_json, standalone_axiom_report_json);
        assert_eq!(shared_theorem_index_json, standalone_theorem_index_json);
        assert_eq!(shared_export_summary_json, standalone_export_summary_json);

        let shared_publish_inputs =
            crate::package_publish::load_package_publish_inputs_from_snapshot(shared).unwrap();
        let shared_publish_plan_json =
            crate::package_publish::project_package_publish_plan_from_inputs(
                &shared_publish_inputs,
            )
            .unwrap()
            .canonical_json()
            .unwrap();

        let standalone_publish_inputs =
            crate::package_publish::load_package_publish_inputs(fixture.path()).unwrap();
        let standalone_publish_plan_json =
            crate::package_publish::project_package_publish_plan_from_inputs(
                &standalone_publish_inputs,
            )
            .unwrap()
            .canonical_json()
            .unwrap();
        assert_eq!(shared_publish_plan_json, standalone_publish_plan_json);
    }

    #[test]
    fn package_artifact_source_free_boundary_ignores_source_replay_meta_and_unrequested_generated()
    {
        let fixture = source_free_fixture("no-source-sidecars");
        assert!(!fixture
            .artifact_path("missing/source/Proofs/Ai/Basic.npa")
            .exists());
        assert!(!fixture
            .artifact_path("missing/meta/Proofs/Ai/Basic.json")
            .exists());
        assert!(!fixture
            .artifact_path("missing/replay/Proofs/Ai/Basic.json")
            .exists());
        assert!(!fixture.artifact_path(PACKAGE_AXIOM_REPORT_PATH).exists());
        assert!(!fixture.artifact_path(PACKAGE_THEOREM_INDEX_PATH).exists());

        let loaded = load_package_artifact_extraction(
            fixture.path(),
            "package axiom-report",
            PackageGeneratedArtifactReadMode::none(),
            PackageArtifactReferenceSummaryMode::Omit,
        )
        .unwrap();

        assert_eq!(loaded.extraction.verified_modules.len(), 1);
        assert!(loaded.checked_generated.axiom_report_json.is_none());
        assert!(loaded.checked_generated.theorem_index_json.is_none());
    }

    #[test]
    fn package_artifact_source_free_boundary_reads_generated_only_when_check_mode_requests_it() {
        let fixture = source_free_fixture("checked-generated");
        write_file(
            fixture.artifact_path(PACKAGE_AXIOM_REPORT_PATH),
            "{\"schema\":\"npa.package.axiom_report.v0.1\"}",
        );
        write_file(
            fixture.artifact_path(PACKAGE_THEOREM_INDEX_PATH),
            "{\"schema\":\"npa.package.theorem_index.v0.1\"}",
        );

        let loaded = load_package_artifact_extraction(
            fixture.path(),
            "package index",
            PackageGeneratedArtifactReadMode::all(),
            PackageArtifactReferenceSummaryMode::Omit,
        )
        .unwrap();

        assert_eq!(
            loaded.checked_generated.axiom_report_json.as_deref(),
            Some("{\"schema\":\"npa.package.axiom_report.v0.1\"}")
        );
        assert_eq!(
            loaded.checked_generated.theorem_index_json.as_deref(),
            Some("{\"schema\":\"npa.package.theorem_index.v0.1\"}")
        );

        let missing = source_free_fixture("missing-generated");
        let result = load_package_artifact_extraction(
            missing.path(),
            "package index",
            PackageGeneratedArtifactReadMode::all(),
            PackageArtifactReferenceSummaryMode::Omit,
        )
        .unwrap_err();
        assert_eq!(result.diagnostics[0].reason_code, "axiom_report_missing");
        assert_eq!(result.diagnostics[0].kind, DiagnosticKind::AxiomReport);
    }
}
