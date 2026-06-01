use std::collections::BTreeMap;

use npa_cert::{Name, VerifiedModule};
use npa_package::{
    build_package_lock_from_artifacts, format_package_hash, package_file_hash,
    PackageArtifactFileReference, PackageArtifactOrigin, PackageCheckerMode, PackageCheckerSummary,
    PackageHash, PackageLockArtifact, PackageLockEntryOrigin, PackageLockError,
    PackageLockErrorKind, PackageLockErrorReason, PackageLockManifest,
    PackageLockManifestReference, PackagePath, ValidatedPackageManifest,
};

use crate::package_verifier::{
    verify_package_fast_source_free_with_modules, verify_package_reference_source_free,
    PackageCertificateArtifact, PackageModuleVerificationResult, PackageModuleVerificationStatus,
    PackageVerificationError, PackageVerificationErrorKind, PackageVerificationErrorReason,
    PackageVerificationMode, PackageVerificationReport, PackageVerificationResult,
    PackageVerificationStatus, PackageVerificationVerdictSource, PackageVerifiedModuleRecord,
};

/// Whether package artifact extraction should include reference-checker summaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageArtifactReferenceSummaryMode {
    /// Collect only fast-kernel summaries required to obtain verified modules.
    Omit,
    /// Also run the CLR-03 source-free reference checker and project its summary.
    Include,
}

/// Source-free input used to extract package artifact metadata.
#[derive(Clone, Debug)]
pub struct PackageArtifactExtractionInput<'a> {
    /// Validated package manifest.
    pub validated: &'a ValidatedPackageManifest,
    /// Package-relative path to the manifest bytes used by the lock.
    pub manifest_path: PackagePath,
    /// Exact manifest bytes used to check lock freshness.
    pub manifest_bytes: &'a [u8],
    /// Parsed generated package lock.
    pub package_lock: &'a PackageLockManifest,
    /// Certificate artifacts loaded by the caller.
    pub certificates: Vec<PackageCertificateArtifact<'a>>,
    /// Reference-checker summary mode.
    pub reference_summaries: PackageArtifactReferenceSummaryMode,
}

/// Stable identity key for a verified package module.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PackageArtifactVerifiedModuleKey {
    /// Module name.
    pub module: Name,
    /// Verified module export hash.
    pub export_hash: PackageHash,
    /// Verified module certificate hash.
    pub certificate_hash: PackageHash,
}

/// Verified module payload available to later package artifact projections.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageArtifactVerifiedModule {
    /// Stable identity key.
    pub key: PackageArtifactVerifiedModuleKey,
    /// Local or external package origin.
    pub origin: PackageArtifactOrigin,
    /// Certificate file identity.
    pub certificate: PackageArtifactFileReference,
    /// Verified module axiom report hash.
    pub axiom_report_hash: PackageHash,
    /// Kernel-verified module data.
    pub verified_module: VerifiedModule,
}

/// Source-free extraction output shared by CLR-05 artifact generators.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageArtifactExtraction {
    /// Manifest reference checked against the package lock.
    pub manifest: PackageLockManifestReference,
    /// Verified modules keyed by module, export hash, and certificate hash.
    pub verified_modules: BTreeMap<PackageArtifactVerifiedModuleKey, PackageArtifactVerifiedModule>,
    /// Verified module keys in package-lock topological order.
    pub topological_order: Vec<PackageArtifactVerifiedModuleKey>,
    /// Checker summaries with explicit fast/reference mode labels.
    pub checker_summaries: Vec<PackageCheckerSummary>,
    /// Fast source-free verifier report used to collect verified modules.
    pub fast_verification_report: PackageVerificationReport,
    /// Optional CLR-03 source-free reference checker report.
    pub reference_verification_report: Option<PackageVerificationReport>,
}

/// Extract source-free package artifact metadata from manifest, lock, and certificates.
///
/// This adapter does not read files. The caller supplies manifest bytes,
/// package-lock data, and certificate bytes; source, replay, metadata, theorem
/// index, AI traces, registry data, and network state are outside this API.
pub fn extract_package_artifacts_source_free<'a>(
    input: PackageArtifactExtractionInput<'a>,
) -> PackageVerificationResult<PackageArtifactExtraction> {
    let fast = verify_package_fast_source_free_with_modules(
        input.validated,
        input.package_lock,
        input.certificates.clone(),
    )?;
    ensure_report_passed(&fast.report)?;
    ensure_package_lock_current(&input)?;

    let mut checker_summaries = checker_summaries_from_report(
        &fast.report,
        PackageCheckerMode::Fast,
        PackageVerificationVerdictSource::FastKernelCertificateVerifier.as_str(),
        PackageVerificationMode::FastKernel.as_str(),
    );
    let reference_verification_report =
        if input.reference_summaries == PackageArtifactReferenceSummaryMode::Include {
            let report = verify_package_reference_source_free(
                input.validated,
                input.package_lock,
                input.certificates,
            )?;
            ensure_report_passed(&report)?;
            checker_summaries.extend(checker_summaries_from_report(
                &report,
                PackageCheckerMode::Reference,
                PackageVerificationVerdictSource::ReferenceChecker.as_str(),
                &input.validated.manifest().checker_profile,
            ));
            Some(report)
        } else {
            None
        };

    let (verified_modules, topological_order) = verified_module_collection(fast.verified_modules)?;

    Ok(PackageArtifactExtraction {
        manifest: PackageLockManifestReference {
            path: input.manifest_path,
            file_hash: package_file_hash(input.manifest_bytes),
        },
        verified_modules,
        topological_order,
        checker_summaries,
        fast_verification_report: fast.report,
        reference_verification_report,
    })
}

fn ensure_report_passed(report: &PackageVerificationReport) -> PackageVerificationResult<()> {
    if report.status == PackageVerificationStatus::Passed {
        Ok(())
    } else {
        Err(report_failure_error(report))
    }
}

fn report_failure_error(report: &PackageVerificationReport) -> PackageVerificationError {
    report
        .modules
        .iter()
        .find_map(|module| module.error.clone())
        .unwrap_or_else(|| PackageVerificationError {
            kind: match report.mode {
                PackageVerificationMode::FastKernel => PackageVerificationErrorKind::Kernel,
                PackageVerificationMode::Reference => {
                    PackageVerificationErrorKind::ReferenceChecker
                }
            },
            path: "verification.status".to_owned(),
            field: Some("status".to_owned()),
            reason_code: match report.mode {
                PackageVerificationMode::FastKernel => {
                    PackageVerificationErrorReason::KernelVerificationFailed
                }
                PackageVerificationMode::Reference => {
                    PackageVerificationErrorReason::ReferenceCheckerRejected
                }
            },
            expected_value: Some(PackageModuleVerificationStatus::Passed.as_str().to_owned()),
            actual_value: Some(report.status.as_str().to_owned()),
            checker_error: None,
        })
}

fn ensure_package_lock_current(
    input: &PackageArtifactExtractionInput<'_>,
) -> PackageVerificationResult<()> {
    let regenerated = build_package_lock_from_artifacts(
        input.validated,
        input.manifest_path.clone(),
        input.manifest_bytes,
        input
            .certificates
            .iter()
            .map(|artifact| PackageLockArtifact {
                path: artifact.path.clone(),
                bytes: artifact.bytes,
            }),
    )
    .map_err(package_lock_error_to_verification_error)?;
    let expected_json = regenerated
        .canonical_json()
        .map_err(package_lock_error_to_verification_error)?;
    let actual_json = input
        .package_lock
        .canonical_json()
        .map_err(package_lock_error_to_verification_error)?;

    if expected_json == actual_json {
        Ok(())
    } else {
        Err(PackageVerificationError::package_lock_stale(
            "generated/package-lock.json",
            format_package_hash(&package_file_hash(expected_json.as_bytes())),
            format_package_hash(&package_file_hash(actual_json.as_bytes())),
        ))
    }
}

fn package_lock_error_to_verification_error(error: PackageLockError) -> PackageVerificationError {
    let (kind, reason_code) = match error.kind {
        PackageLockErrorKind::ArtifactIo => (
            PackageVerificationErrorKind::Artifact,
            match error.reason_code {
                PackageLockErrorReason::CertificateMissing => {
                    PackageVerificationErrorReason::CertificateArtifactMissing
                }
                _ => PackageVerificationErrorReason::CertificateArtifactMissing,
            },
        ),
        PackageLockErrorKind::CertificateDecode => (
            PackageVerificationErrorKind::CertificateDecode,
            PackageVerificationErrorReason::CertificateDecodeFailed,
        ),
        PackageLockErrorKind::CertificateIdentity => (
            PackageVerificationErrorKind::CertificateIdentity,
            match error.reason_code {
                PackageLockErrorReason::CertificateModuleMismatch => {
                    PackageVerificationErrorReason::CertificateModuleMismatch
                }
                PackageLockErrorReason::CertificateFileHashMismatch => {
                    PackageVerificationErrorReason::CertificateFileHashMismatch
                }
                PackageLockErrorReason::ExportHashMismatch => {
                    PackageVerificationErrorReason::ExportHashMismatch
                }
                PackageLockErrorReason::AxiomReportHashMismatch => {
                    PackageVerificationErrorReason::AxiomReportHashMismatch
                }
                PackageLockErrorReason::CertificateHashMismatch => {
                    PackageVerificationErrorReason::CertificateHashMismatch
                }
                _ => PackageVerificationErrorReason::LockGraphInvalid,
            },
        ),
        PackageLockErrorKind::Graph => (
            PackageVerificationErrorKind::LockGraph,
            PackageVerificationErrorReason::LockGraphInvalid,
        ),
        _ => (
            PackageVerificationErrorKind::Input,
            PackageVerificationErrorReason::LockGraphInvalid,
        ),
    };

    PackageVerificationError {
        kind,
        path: error.path,
        field: error.field,
        reason_code,
        expected_value: error.expected_value,
        actual_value: error.actual_value,
        checker_error: None,
    }
}

fn verified_module_collection(
    records: Vec<PackageVerifiedModuleRecord>,
) -> PackageVerificationResult<(
    BTreeMap<PackageArtifactVerifiedModuleKey, PackageArtifactVerifiedModule>,
    Vec<PackageArtifactVerifiedModuleKey>,
)> {
    let mut verified_modules = BTreeMap::new();
    let mut topological_order = Vec::with_capacity(records.len());
    for record in records {
        let key = PackageArtifactVerifiedModuleKey {
            module: record.module,
            export_hash: record.export_hash,
            certificate_hash: record.certificate_hash,
        };
        let verified = PackageArtifactVerifiedModule {
            key: key.clone(),
            origin: artifact_origin(record.origin),
            certificate: PackageArtifactFileReference {
                path: record.certificate,
                file_hash: record.certificate_file_hash,
            },
            axiom_report_hash: record.axiom_report_hash,
            verified_module: record.verified_module,
        };
        if verified_modules.insert(key.clone(), verified).is_some() {
            return Err(PackageVerificationError {
                kind: PackageVerificationErrorKind::LockGraph,
                path: "verified_modules".to_owned(),
                field: Some("module".to_owned()),
                reason_code: PackageVerificationErrorReason::LockGraphInvalid,
                expected_value: Some("unique module/export/certificate identity".to_owned()),
                actual_value: Some(key.module.as_dotted()),
                checker_error: None,
            });
        }
        topological_order.push(key);
    }
    Ok((verified_modules, topological_order))
}

fn artifact_origin(origin: PackageLockEntryOrigin) -> PackageArtifactOrigin {
    match origin {
        PackageLockEntryOrigin::Local => PackageArtifactOrigin::Local,
        PackageLockEntryOrigin::External => PackageArtifactOrigin::External,
    }
}

fn checker_summaries_from_report(
    report: &PackageVerificationReport,
    mode: PackageCheckerMode,
    checker: &str,
    profile: &str,
) -> Vec<PackageCheckerSummary> {
    report
        .modules
        .iter()
        .map(|module| checker_summary_from_module(module, mode, checker, profile))
        .collect()
}

fn checker_summary_from_module(
    module: &PackageModuleVerificationResult,
    mode: PackageCheckerMode,
    checker: &str,
    profile: &str,
) -> PackageCheckerSummary {
    PackageCheckerSummary {
        module: module.module.clone(),
        checker: checker.to_owned(),
        profile: profile.to_owned(),
        mode,
        status: module.status.as_str().to_owned(),
        export_hash: module.export_hash,
        certificate_hash: module.certificate_hash,
        axiom_report_hash: module.axiom_report_hash,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use npa_package::{
        build_package_lock_from_artifacts, parse_and_validate_manifest_str, PackageLockImport,
    };

    use super::*;

    const BASIC_CERTIFICATE_PATH: &str = "Proofs/Ai/Basic/certificate.npcert";

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("npa-api crate lives under crates/")
            .to_path_buf()
    }

    fn basic_certificate_bytes() -> Vec<u8> {
        fs::read(repo_root().join("proofs").join(BASIC_CERTIFICATE_PATH)).unwrap()
    }

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

    fn basic_lock(
        validated: &ValidatedPackageManifest,
        manifest_source: &str,
        certificate_bytes: &[u8],
    ) -> PackageLockManifest {
        build_package_lock_from_artifacts(
            validated,
            PackagePath::new("npa-package.toml"),
            manifest_source.as_bytes(),
            [PackageLockArtifact {
                path: PackagePath::new(BASIC_CERTIFICATE_PATH),
                bytes: certificate_bytes,
            }],
        )
        .unwrap()
    }

    fn extraction_input<'a>(
        validated: &'a ValidatedPackageManifest,
        lock: &'a PackageLockManifest,
        manifest_source: &'a str,
        certificate_bytes: &'a [u8],
        reference_summaries: PackageArtifactReferenceSummaryMode,
    ) -> PackageArtifactExtractionInput<'a> {
        PackageArtifactExtractionInput {
            validated,
            manifest_path: PackagePath::new("npa-package.toml"),
            manifest_bytes: manifest_source.as_bytes(),
            package_lock: lock,
            certificates: vec![PackageCertificateArtifact {
                path: PackagePath::new(BASIC_CERTIFICATE_PATH),
                bytes: certificate_bytes,
            }],
            reference_summaries,
        }
    }

    #[test]
    fn package_artifact_extraction_collects_verified_modules_and_fast_summaries_source_free() {
        let manifest_source = basic_manifest_source();
        let validated = parse_and_validate_manifest_str(&manifest_source).unwrap();
        let certificate_bytes = basic_certificate_bytes();
        let lock = basic_lock(&validated, &manifest_source, &certificate_bytes);

        let extraction = extract_package_artifacts_source_free(extraction_input(
            &validated,
            &lock,
            &manifest_source,
            &certificate_bytes,
            PackageArtifactReferenceSummaryMode::Omit,
        ))
        .unwrap();

        assert_eq!(extraction.topological_order.len(), 1);
        let key = &extraction.topological_order[0];
        assert_eq!(key.module.as_dotted(), "Proofs.Ai.Basic");
        let module = extraction.verified_modules.get(key).unwrap();
        assert_eq!(module.origin, PackageArtifactOrigin::Local);
        assert_eq!(
            module.verified_module.module().as_dotted(),
            "Proofs.Ai.Basic"
        );
        assert_eq!(
            module.verified_module.export_hash(),
            key.export_hash.into_bytes()
        );
        assert_eq!(
            module.verified_module.certificate_hash(),
            key.certificate_hash.into_bytes()
        );
        assert_eq!(extraction.checker_summaries.len(), 1);
        let summary = &extraction.checker_summaries[0];
        assert_eq!(summary.mode, PackageCheckerMode::Fast);
        assert_eq!(summary.checker, "fast-kernel-certificate-verifier");
        assert_eq!(summary.profile, "fast-kernel");
        assert_ne!(summary.checker, "npa-checker-ref");
        assert_eq!(summary.status, "passed");
        assert!(
            !extraction
                .fast_verification_report
                .reference_checker_verdict
        );
        assert!(extraction.reference_verification_report.is_none());
    }

    #[test]
    fn package_artifact_extraction_reference_summary_uses_source_free_reference_verifier() {
        let manifest_source = basic_manifest_source();
        let validated = parse_and_validate_manifest_str(&manifest_source).unwrap();
        let certificate_bytes = basic_certificate_bytes();
        let lock = basic_lock(&validated, &manifest_source, &certificate_bytes);

        let extraction = extract_package_artifacts_source_free(extraction_input(
            &validated,
            &lock,
            &manifest_source,
            &certificate_bytes,
            PackageArtifactReferenceSummaryMode::Include,
        ))
        .unwrap();

        assert_eq!(extraction.checker_summaries.len(), 2);
        let reference = extraction
            .checker_summaries
            .iter()
            .find(|summary| summary.mode == PackageCheckerMode::Reference)
            .expect("reference summary is included");
        assert_eq!(reference.checker, "npa-checker-ref");
        assert_eq!(reference.profile, "npa.checker.reference.v0.1");
        assert_eq!(reference.status, "passed");
        assert!(
            extraction
                .reference_verification_report
                .as_ref()
                .unwrap()
                .reference_checker_verdict
        );
    }

    #[test]
    fn package_artifact_extraction_rejects_stale_lock_missing_artifacts_and_imports() {
        let manifest_source = basic_manifest_source();
        let validated = parse_and_validate_manifest_str(&manifest_source).unwrap();
        let certificate_bytes = basic_certificate_bytes();
        let lock = basic_lock(&validated, &manifest_source, &certificate_bytes);

        let mut missing_input = extraction_input(
            &validated,
            &lock,
            &manifest_source,
            &certificate_bytes,
            PackageArtifactReferenceSummaryMode::Omit,
        );
        missing_input.certificates.clear();
        let missing = extract_package_artifacts_source_free(missing_input).unwrap_err();
        assert_eq!(
            missing.reason_code,
            PackageVerificationErrorReason::CertificateArtifactMissing
        );

        let mut stale_lock = lock.clone();
        stale_lock.manifest.file_hash = PackageHash::new([0x77; 32]);
        let stale = extract_package_artifacts_source_free(extraction_input(
            &validated,
            &stale_lock,
            &manifest_source,
            &certificate_bytes,
            PackageArtifactReferenceSummaryMode::Omit,
        ))
        .unwrap_err();
        assert_eq!(
            stale.reason_code,
            PackageVerificationErrorReason::PackageLockStale
        );

        let mut stale_certificate = certificate_bytes.clone();
        stale_certificate[0] ^= 0x01;
        let stale_certificate_error = extract_package_artifacts_source_free(extraction_input(
            &validated,
            &lock,
            &manifest_source,
            &stale_certificate,
            PackageArtifactReferenceSummaryMode::Omit,
        ))
        .unwrap_err();
        assert_eq!(
            stale_certificate_error.reason_code,
            PackageVerificationErrorReason::CertificateFileHashMismatch
        );

        let mut missing_import_lock = lock.clone();
        missing_import_lock.entries[0]
            .imports
            .push(PackageLockImport {
                module: Name(vec!["Missing".to_owned(), "Import".to_owned()]),
                export_hash: PackageHash::new([0x11; 32]),
                certificate_hash: PackageHash::new([0x22; 32]),
            });
        let missing_import = extract_package_artifacts_source_free(extraction_input(
            &validated,
            &missing_import_lock,
            &manifest_source,
            &certificate_bytes,
            PackageArtifactReferenceSummaryMode::Omit,
        ))
        .unwrap_err();
        assert_eq!(
            missing_import.reason_code,
            PackageVerificationErrorReason::LockGraphInvalid
        );
    }
}
