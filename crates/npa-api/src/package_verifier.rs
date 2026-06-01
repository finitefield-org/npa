use std::collections::BTreeMap;

use npa_cert::{
    decode_module_cert, verify_module_cert, AxiomPolicy, CertError, CoreFeature, Name,
    VerifiedModule, VerifierSession,
};
use npa_checker_ref::{
    check_certificate, verify_certificate_hashes, ReferenceCertificateSection, ReferenceCheckError,
    ReferenceCheckErrorKind, ReferenceCheckReason, ReferenceCheckResult, ReferenceCheckedModule,
    ReferenceCheckerPolicy, ReferenceCoreFeature, ReferenceImportStore, ReferenceModuleName,
    ReferenceTrustMode,
};
use npa_package::{
    build_package_lock_graph, format_package_hash, package_file_hash,
    validate_package_lock_against_manifest_graph, PackageHash, PackageLockEntry,
    PackageLockEntryOrigin, PackageLockManifest, PackageLockResolvedImport, PackagePath,
    ValidatedPackageManifest, CHECKER_PROFILE_REFERENCE_V0_1,
};

use crate::independent_checker::{
    independent_checker_file_hash, independent_checker_request_materialize,
    parse_independent_checker_import_lock_manifest, IndependentCheckerCommandError,
    IndependentCheckerImportLockCertificate, IndependentCheckerImportLockEntry,
    IndependentCheckerImportLockManifest, IndependentCheckerMachineCheckRequest,
    IndependentCheckerRequestStoreManifest, IndependentCheckerRunnerPolicy,
};
use crate::types::{machine_api_name_canonical_bytes, parse_module_name_wire};

/// Result type for source-free package verification.
pub type PackageVerificationResult<T> = Result<T, PackageVerificationError>;

/// Certificate artifact bytes supplied by the caller.
#[derive(Clone, Debug)]
pub struct PackageCertificateArtifact<'a> {
    /// Package-relative certificate path.
    pub path: PackagePath,
    /// Exact certificate bytes at [`Self::path`].
    pub bytes: &'a [u8],
}

/// Package verification mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageVerificationMode {
    /// Fast local verifier backed by `npa_cert::verify_module_cert`.
    FastKernel,
    /// Source-free independent reference checker mode backed by `npa-checker-ref`.
    Reference,
}

impl PackageVerificationMode {
    /// Return the stable mode string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FastKernel => "fast-kernel",
            Self::Reference => "reference",
        }
    }
}

/// Source of the package verification verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageVerificationVerdictSource {
    /// Verdict came from the fast certificate verifier, not `npa-checker-ref`.
    FastKernelCertificateVerifier,
    /// Verdict came from `npa-checker-ref`.
    ReferenceChecker,
}

impl PackageVerificationVerdictSource {
    /// Return the stable verdict-source string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FastKernelCertificateVerifier => "fast-kernel-certificate-verifier",
            Self::ReferenceChecker => "npa-checker-ref",
        }
    }

    /// Return whether this verdict came from the independent reference checker.
    pub const fn is_reference_checker_verdict(self) -> bool {
        match self {
            Self::FastKernelCertificateVerifier => false,
            Self::ReferenceChecker => true,
        }
    }
}

/// Overall package verification status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageVerificationStatus {
    /// Every lock entry verified successfully.
    Passed,
    /// At least one lock entry failed or was skipped after an earlier failure.
    Failed,
}

impl PackageVerificationStatus {
    /// Return the stable status string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
        }
    }
}

/// Per-module package verification status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageModuleVerificationStatus {
    /// Certificate bytes verified successfully.
    Passed,
    /// Certificate bytes failed deterministic fast-kernel verification.
    Failed,
    /// Certificate verification was not attempted after an earlier failure.
    Skipped,
}

impl PackageModuleVerificationStatus {
    /// Return the stable status string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }
}

/// Source-free package verification report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageVerificationReport {
    /// Verification mode used for every module in this report.
    pub mode: PackageVerificationMode,
    /// Explicit verdict source, to distinguish fast results from reference checker results.
    pub verdict_source: PackageVerificationVerdictSource,
    /// Convenience field that is true only for independent reference checker verdicts.
    pub reference_checker_verdict: bool,
    /// Overall status.
    pub status: PackageVerificationStatus,
    /// Topological lock-graph verification order.
    pub topological_order: Vec<Name>,
    /// Per-module results in [`Self::topological_order`].
    pub modules: Vec<PackageModuleVerificationResult>,
}

/// Per-module source-free verification result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageModuleVerificationResult {
    /// Module name from the package lock entry.
    pub module: Name,
    /// Verification mode used for this module.
    pub checker_mode: PackageVerificationMode,
    /// Per-module status.
    pub status: PackageModuleVerificationStatus,
    /// Expected export hash from the package lock entry.
    pub export_hash: PackageHash,
    /// Expected axiom report hash from the package lock entry.
    pub axiom_report_hash: PackageHash,
    /// Expected certificate hash from the package lock entry.
    pub certificate_hash: PackageHash,
    /// Deterministic failure details for failed or skipped modules.
    pub error: Option<PackageVerificationError>,
}

/// Verified module payload accepted by the fast source-free package verifier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageVerifiedModuleRecord {
    /// Module name from the package lock entry.
    pub module: Name,
    /// Whether this module is local to the package or an external hash-pinned import.
    pub origin: PackageLockEntryOrigin,
    /// Package-relative certificate path.
    pub certificate: PackagePath,
    /// Exact SHA-256 hash of the certificate file bytes.
    pub certificate_file_hash: PackageHash,
    /// Verified module export hash.
    pub export_hash: PackageHash,
    /// Verified module axiom report hash.
    pub axiom_report_hash: PackageHash,
    /// Verified module certificate hash.
    pub certificate_hash: PackageHash,
    /// Kernel-verified module data used by later certificate-derived projections.
    pub verified_module: VerifiedModule,
}

/// Fast source-free package verification report with collected verified modules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageFastSourceFreeVerification {
    /// Fast verifier summary.
    pub report: PackageVerificationReport,
    /// Verified modules in package-lock topological order.
    pub verified_modules: Vec<PackageVerifiedModuleRecord>,
}

/// Per-module Phase 8 import lock derived from a package lock entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackagePhase8ImportLockMaterialization {
    /// Module this import lock verifies.
    pub module: Name,
    /// Deterministic package-relative path for the generated import lock JSON.
    pub path: String,
    /// Phase 8 import lock manifest containing only direct imports.
    pub manifest: IndependentCheckerImportLockManifest,
    /// Exact file hash of [`Self::manifest`] canonical JSON.
    pub manifest_hash: npa_cert::Hash,
}

/// Per-module Phase 8 machine-check request derived from a package lock entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackagePhase8RequestMaterialization {
    /// Module this request verifies.
    pub module: Name,
    /// Phase 8 checker profile used for this request.
    pub checker_profile: String,
    /// Deterministic package-relative path for the generated import lock JSON.
    pub import_lock_path: String,
    /// Phase 8 import lock manifest containing only direct imports.
    pub import_lock_manifest: IndependentCheckerImportLockManifest,
    /// Exact file hash of [`Self::import_lock_manifest`] canonical JSON.
    pub import_lock_manifest_hash: npa_cert::Hash,
    /// Deterministic package-relative path for the generated request JSON.
    pub request_path: String,
    /// Materialized Phase 8 machine-check request.
    pub request: IndependentCheckerMachineCheckRequest,
    /// Exact file hash of [`Self::request`] canonical JSON.
    pub request_file_hash: npa_cert::Hash,
}

/// Package-level Phase 8 machine-check request materialization result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackagePhase8RequestMaterializationReport {
    /// Per-module requests in package-lock topological order.
    pub modules: Vec<PackagePhase8RequestMaterialization>,
    /// Final request-store manifest after adding every generated request.
    pub request_store: IndependentCheckerRequestStoreManifest,
    /// Exact file hash of [`Self::request_store`] canonical JSON.
    pub request_store_file_hash: npa_cert::Hash,
    /// Whether the request store needs to be written or replaced.
    pub request_store_rewrite_required: bool,
}

/// Structured source-free package verification error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageVerificationError {
    /// Stable error category.
    pub kind: PackageVerificationErrorKind,
    /// Stable artifact-local path, for example `entries[0].certificate`.
    pub path: String,
    /// Field name when the error is attached to one object field.
    pub field: Option<String>,
    /// Stable machine-readable reason code.
    pub reason_code: PackageVerificationErrorReason,
    /// Expected value or type when useful.
    pub expected_value: Option<String>,
    /// Actual value or type when useful.
    pub actual_value: Option<String>,
    /// Checker-local structured rejection details, when the error came from a checker.
    pub checker_error: Option<Box<PackageVerificationCheckerError>>,
}

/// Structured checker-local package verification error details.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageVerificationCheckerError {
    /// Checker implementation that produced the error.
    pub checker: String,
    /// Checker-local stable error kind.
    pub kind: String,
    /// Checker-local certificate section.
    pub section: Option<String>,
    /// Checker-local byte offset, when applicable.
    pub offset: Option<usize>,
    /// Checker-local stable reason code.
    pub reason_code: Option<String>,
}

impl PackageVerificationError {
    pub(crate) fn package_lock_stale(
        path: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self::new(
            PackageVerificationErrorKind::Input,
            path,
            Some("package_lock".to_owned()),
            PackageVerificationErrorReason::PackageLockStale,
            Some(expected.into()),
            Some(actual.into()),
        )
    }

    fn package_identity_mismatch(
        path: impl Into<String>,
        field: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self::new(
            PackageVerificationErrorKind::Input,
            path,
            Some(field.into()),
            PackageVerificationErrorReason::PackageIdentityMismatch,
            Some(expected.into()),
            Some(actual.into()),
        )
    }

    fn lock_graph_invalid(actual: impl Into<String>) -> Self {
        Self::new(
            PackageVerificationErrorKind::LockGraph,
            "lock",
            None,
            PackageVerificationErrorReason::LockGraphInvalid,
            Some("valid package lock graph matching manifest imports".to_owned()),
            Some(actual.into()),
        )
    }

    fn duplicate_certificate_artifact(path: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::new(
            PackageVerificationErrorKind::Artifact,
            path,
            Some("certificate".to_owned()),
            PackageVerificationErrorReason::DuplicateCertificateArtifact,
            Some("unique certificate artifact path".to_owned()),
            Some(actual.into()),
        )
    }

    fn certificate_artifact_missing(path: impl Into<String>, expected: impl Into<String>) -> Self {
        Self::new(
            PackageVerificationErrorKind::Artifact,
            path,
            Some("certificate".to_owned()),
            PackageVerificationErrorReason::CertificateArtifactMissing,
            Some(expected.into()),
            None,
        )
    }

    fn certificate_file_hash_mismatch(
        path: impl Into<String>,
        expected: PackageHash,
        actual: PackageHash,
    ) -> Self {
        Self::hash_mismatch(
            PackageVerificationErrorKind::CertificateIdentity,
            path,
            "certificate_file_hash",
            PackageVerificationErrorReason::CertificateFileHashMismatch,
            expected,
            actual,
        )
    }

    fn certificate_decode_failed(path: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::new(
            PackageVerificationErrorKind::CertificateDecode,
            path,
            Some("certificate".to_owned()),
            PackageVerificationErrorReason::CertificateDecodeFailed,
            Some("decodable npa module certificate".to_owned()),
            Some(actual.into()),
        )
    }

    fn certificate_module_mismatch(
        path: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self::new(
            PackageVerificationErrorKind::CertificateIdentity,
            path,
            Some("module".to_owned()),
            PackageVerificationErrorReason::CertificateModuleMismatch,
            Some(expected.into()),
            Some(actual.into()),
        )
    }

    fn export_hash_mismatch(
        path: impl Into<String>,
        expected: PackageHash,
        actual: PackageHash,
    ) -> Self {
        Self::hash_mismatch(
            PackageVerificationErrorKind::CertificateIdentity,
            path,
            "export_hash",
            PackageVerificationErrorReason::ExportHashMismatch,
            expected,
            actual,
        )
    }

    fn axiom_report_hash_mismatch(
        path: impl Into<String>,
        expected: PackageHash,
        actual: PackageHash,
    ) -> Self {
        Self::hash_mismatch(
            PackageVerificationErrorKind::CertificateIdentity,
            path,
            "axiom_report_hash",
            PackageVerificationErrorReason::AxiomReportHashMismatch,
            expected,
            actual,
        )
    }

    fn certificate_hash_mismatch(
        path: impl Into<String>,
        expected: PackageHash,
        actual: PackageHash,
    ) -> Self {
        Self::hash_mismatch(
            PackageVerificationErrorKind::CertificateIdentity,
            path,
            "certificate_hash",
            PackageVerificationErrorReason::CertificateHashMismatch,
            expected,
            actual,
        )
    }

    fn verify_failed(path: impl Into<String>, source: CertError) -> Self {
        let reason_code = match source {
            CertError::ForbiddenAxiom { .. } | CertError::SorryDenied { .. } => {
                PackageVerificationErrorReason::AxiomPolicyRejected
            }
            CertError::UnsupportedCoreFeature { .. } => {
                PackageVerificationErrorReason::UnsupportedCoreFeature
            }
            _ => PackageVerificationErrorReason::KernelVerificationFailed,
        };
        Self::new_with_checker_error(
            PackageVerificationErrorKind::Kernel,
            path,
            Some("certificate".to_owned()),
            reason_code,
            Some("kernel-verifiable module certificate".to_owned()),
            Some(format!("{source:?}")),
            Some(PackageVerificationCheckerError {
                checker: "npa-cert".to_owned(),
                kind: "certificate_verifier".to_owned(),
                section: None,
                offset: None,
                reason_code: Some(reason_code.as_str().to_owned()),
            }),
        )
    }

    fn reference_checker_rejected(path: impl Into<String>, source: ReferenceCheckError) -> Self {
        let reason_code = package_reference_checker_reason(&source);
        Self::new_with_checker_error(
            PackageVerificationErrorKind::ReferenceChecker,
            path,
            Some("certificate".to_owned()),
            reason_code,
            Some("reference-checker-verifiable module certificate".to_owned()),
            Some(format!("{source:?}")),
            Some(reference_checker_error_details(&source)),
        )
    }

    fn phase8_import_lock_invalid(path: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::new(
            PackageVerificationErrorKind::Phase8Adapter,
            path,
            Some("imports.manifest".to_owned()),
            PackageVerificationErrorReason::Phase8ImportLockMaterializationFailed,
            Some("valid independent checker import lock manifest".to_owned()),
            Some(actual.into()),
        )
    }

    fn phase8_request_materialization_failed(
        path: impl Into<String>,
        source: IndependentCheckerCommandError,
    ) -> Self {
        let expected_value = source
            .expected_value
            .map(|value| value.to_string())
            .or_else(|| {
                source
                    .expected_hash
                    .as_deref()
                    .map(|hash| format_package_hash(&PackageHash::from(*hash)))
            });
        let actual_value = source
            .actual_value
            .map(|value| value.to_string())
            .or_else(|| {
                source
                    .actual_hash
                    .as_deref()
                    .map(|hash| format_package_hash(&PackageHash::from(*hash)))
            });
        Self::new(
            PackageVerificationErrorKind::Phase8Adapter,
            path,
            source.field.as_deref().map(str::to_owned),
            PackageVerificationErrorReason::Phase8RequestMaterializationFailed,
            expected_value,
            actual_value,
        )
    }

    fn earlier_module_failed(path: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::new(
            PackageVerificationErrorKind::Dependency,
            path,
            Some("module".to_owned()),
            PackageVerificationErrorReason::EarlierModuleFailed,
            Some("all prior package lock entries passed".to_owned()),
            Some(actual.into()),
        )
    }

    fn hash_mismatch(
        kind: PackageVerificationErrorKind,
        path: impl Into<String>,
        field: impl Into<String>,
        reason_code: PackageVerificationErrorReason,
        expected: PackageHash,
        actual: PackageHash,
    ) -> Self {
        Self::new(
            kind,
            path,
            Some(field.into()),
            reason_code,
            Some(format_package_hash(&expected)),
            Some(format_package_hash(&actual)),
        )
    }

    fn new(
        kind: PackageVerificationErrorKind,
        path: impl Into<String>,
        field: Option<String>,
        reason_code: PackageVerificationErrorReason,
        expected_value: Option<String>,
        actual_value: Option<String>,
    ) -> Self {
        Self::new_with_checker_error(
            kind,
            path,
            field,
            reason_code,
            expected_value,
            actual_value,
            None,
        )
    }

    fn new_with_checker_error(
        kind: PackageVerificationErrorKind,
        path: impl Into<String>,
        field: Option<String>,
        reason_code: PackageVerificationErrorReason,
        expected_value: Option<String>,
        actual_value: Option<String>,
        checker_error: Option<PackageVerificationCheckerError>,
    ) -> Self {
        Self {
            kind,
            path: path.into(),
            field,
            reason_code,
            expected_value,
            actual_value,
            checker_error: checker_error.map(Box::new),
        }
    }
}

/// Stable package verification error category.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageVerificationErrorKind {
    /// Caller supplied inconsistent manifest or lock identity.
    Input,
    /// Package lock graph validation failed before certificate verification.
    LockGraph,
    /// Required certificate artifact bytes are absent or duplicated.
    Artifact,
    /// Certificate bytes could not be decoded syntactically.
    CertificateDecode,
    /// Certificate identity does not match the package lock entry.
    CertificateIdentity,
    /// Kernel certificate verification failed.
    Kernel,
    /// Independent reference checker verification failed.
    ReferenceChecker,
    /// Phase 8 import-lock or request adapter materialization failed.
    Phase8Adapter,
    /// Verification was skipped because an earlier lock entry failed.
    Dependency,
}

/// Stable package verification error reason code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PackageVerificationErrorReason {
    /// Manifest and lock package identity differ.
    PackageIdentityMismatch,
    /// Checked package lock no longer matches manifest and certificate artifacts.
    PackageLockStale,
    /// Lock graph or manifest import accountability validation failed.
    LockGraphInvalid,
    /// Caller supplied duplicate artifact bytes for one certificate path.
    DuplicateCertificateArtifact,
    /// Certificate artifact bytes are missing.
    CertificateArtifactMissing,
    /// Certificate file hash differs from the lock entry.
    CertificateFileHashMismatch,
    /// Certificate bytes do not decode as an NPA module certificate.
    CertificateDecodeFailed,
    /// Certificate module name differs from the lock entry.
    CertificateModuleMismatch,
    /// Certificate export hash differs from the lock entry.
    ExportHashMismatch,
    /// Certificate axiom report hash differs from the lock entry.
    AxiomReportHashMismatch,
    /// Certificate canonical hash differs from the lock entry.
    CertificateHashMismatch,
    /// Certificate was rejected by package-derived axiom policy.
    AxiomPolicyRejected,
    /// Certificate requires a core feature unsupported by the selected checker profile.
    UnsupportedCoreFeature,
    /// Certificate was rejected by the fast kernel verifier.
    KernelVerificationFailed,
    /// Certificate was rejected by the independent reference checker.
    ReferenceCheckerRejected,
    /// Phase 8 import lock could not be materialized from package data.
    Phase8ImportLockMaterializationFailed,
    /// Phase 8 machine-check request could not be materialized from package data.
    Phase8RequestMaterializationFailed,
    /// Module was skipped because an earlier topological dependency failed.
    EarlierModuleFailed,
}

impl PackageVerificationErrorReason {
    /// Return the stable wire reason code.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PackageIdentityMismatch => "package_identity_mismatch",
            Self::PackageLockStale => "package_lock_stale",
            Self::LockGraphInvalid => "lock_graph_invalid",
            Self::DuplicateCertificateArtifact => "duplicate_certificate_artifact",
            Self::CertificateArtifactMissing => "certificate_artifact_missing",
            Self::CertificateFileHashMismatch => "certificate_file_hash_mismatch",
            Self::CertificateDecodeFailed => "certificate_decode_failed",
            Self::CertificateModuleMismatch => "certificate_module_mismatch",
            Self::ExportHashMismatch => "export_hash_mismatch",
            Self::AxiomReportHashMismatch => "axiom_report_hash_mismatch",
            Self::CertificateHashMismatch => "certificate_hash_mismatch",
            Self::AxiomPolicyRejected => "axiom_policy_rejected",
            Self::UnsupportedCoreFeature => "unsupported_core_feature",
            Self::KernelVerificationFailed => "kernel_verification_failed",
            Self::ReferenceCheckerRejected => "reference_checker_rejected",
            Self::Phase8ImportLockMaterializationFailed => {
                "independent_checker_import_lock_materialization_failed"
            }
            Self::Phase8RequestMaterializationFailed => {
                "independent_checker_request_materialization_failed"
            }
            Self::EarlierModuleFailed => "earlier_module_failed",
        }
    }
}

/// Verify package certificates source-free with the fast kernel verifier.
///
/// The verifier consumes only a validated package manifest, a package lock, and
/// caller-provided certificate bytes. It never reads source, replay, metadata,
/// theorem-index, AI trace, or checker-result files.
pub fn verify_package_fast_source_free<'a>(
    validated: &ValidatedPackageManifest,
    lock: &PackageLockManifest,
    artifacts: impl IntoIterator<Item = PackageCertificateArtifact<'a>>,
) -> PackageVerificationResult<PackageVerificationReport> {
    Ok(verify_package_fast_source_free_with_modules(validated, lock, artifacts)?.report)
}

/// Verify package certificates source-free with the fast kernel verifier and
/// return the verified module collection.
///
/// The returned modules are the `npa_cert::VerifiedModule` values produced by
/// the same source-free fast verifier used for the report. No source, replay,
/// metadata, theorem-index, AI trace, registry, or checker-result files are
/// read by this API.
pub fn verify_package_fast_source_free_with_modules<'a>(
    validated: &ValidatedPackageManifest,
    lock: &PackageLockManifest,
    artifacts: impl IntoIterator<Item = PackageCertificateArtifact<'a>>,
) -> PackageVerificationResult<PackageFastSourceFreeVerification> {
    validate_manifest_lock_identity(validated, lock)?;
    let graph = validate_package_lock_against_manifest_graph(validated, lock)
        .map_err(|error| PackageVerificationError::lock_graph_invalid(format!("{error:?}")))?;
    let artifact_bytes = artifact_byte_map(artifacts)?;
    let entries = canonical_lock_entries(lock);
    let entries_by_module = entries
        .iter()
        .map(|(index, entry)| (entry.module.clone(), (*index, *entry)))
        .collect::<BTreeMap<_, _>>();
    let policy = package_fast_kernel_policy(validated);
    let mut session = VerifierSession::new();
    let mut results = Vec::with_capacity(graph.topological_order.len());
    let mut verified_modules = Vec::with_capacity(graph.topological_order.len());
    let mut failed_module = None::<Name>;

    for module in &graph.topological_order {
        let (entry_index, entry) = entries_by_module
            .get(module)
            .expect("lock graph order only contains lock entries");
        if let Some(failed) = &failed_module {
            results.push(module_result(
                entry,
                PackageModuleVerificationStatus::Skipped,
                Some(PackageVerificationError::earlier_module_failed(
                    format!("entries[{entry_index}].module"),
                    failed.as_dotted(),
                )),
                PackageVerificationMode::FastKernel,
            ));
            continue;
        }

        match verify_lock_entry(*entry_index, entry, &artifact_bytes, &mut session, &policy) {
            Ok(verified_module) => {
                verified_modules.push(PackageVerifiedModuleRecord {
                    module: entry.module.clone(),
                    origin: entry.origin,
                    certificate: entry.certificate.clone(),
                    certificate_file_hash: entry.certificate_file_hash,
                    export_hash: entry.export_hash,
                    axiom_report_hash: entry.axiom_report_hash,
                    certificate_hash: entry.certificate_hash,
                    verified_module,
                });
                results.push(module_result(
                    entry,
                    PackageModuleVerificationStatus::Passed,
                    None,
                    PackageVerificationMode::FastKernel,
                ));
            }
            Err(error) => {
                failed_module = Some(entry.module.clone());
                results.push(module_result(
                    entry,
                    PackageModuleVerificationStatus::Failed,
                    Some(error),
                    PackageVerificationMode::FastKernel,
                ));
            }
        }
    }

    let status = if failed_module.is_some() {
        PackageVerificationStatus::Failed
    } else {
        PackageVerificationStatus::Passed
    };
    let verdict_source = PackageVerificationVerdictSource::FastKernelCertificateVerifier;

    let report = PackageVerificationReport {
        mode: PackageVerificationMode::FastKernel,
        verdict_source,
        reference_checker_verdict: verdict_source.is_reference_checker_verdict(),
        status,
        topological_order: graph.topological_order,
        modules: results,
    };

    Ok(PackageFastSourceFreeVerification {
        report,
        verified_modules,
    })
}

/// Verify package certificates source-free with the independent reference checker.
///
/// This verifier consumes only a validated package manifest, a package lock, and
/// caller-provided certificate bytes. It executes `npa-checker-ref` in-process
/// in package-lock topological order and builds each import store from modules
/// already accepted by the same reference checker.
pub fn verify_package_reference_source_free<'a>(
    validated: &ValidatedPackageManifest,
    lock: &PackageLockManifest,
    artifacts: impl IntoIterator<Item = PackageCertificateArtifact<'a>>,
) -> PackageVerificationResult<PackageVerificationReport> {
    validate_manifest_lock_identity(validated, lock)?;
    let graph = validate_package_lock_against_manifest_graph(validated, lock)
        .map_err(|error| PackageVerificationError::lock_graph_invalid(format!("{error:?}")))?;
    let artifact_bytes = artifact_byte_map(artifacts)?;
    let entries = canonical_lock_entries(lock);
    let entries_by_module = entries
        .iter()
        .map(|(index, entry)| (entry.module.clone(), (*index, *entry)))
        .collect::<BTreeMap<_, _>>();
    let policy = package_reference_checker_policy(validated);
    let mut checked_by_module = BTreeMap::<Name, ReferenceCheckedModule>::new();
    let mut results = Vec::with_capacity(graph.topological_order.len());
    let mut failed_module = None::<Name>;

    for module in &graph.topological_order {
        let (entry_index, entry) = entries_by_module
            .get(module)
            .expect("lock graph order only contains lock entries");
        if let Some(failed) = &failed_module {
            results.push(module_result(
                entry,
                PackageModuleVerificationStatus::Skipped,
                Some(PackageVerificationError::earlier_module_failed(
                    format!("entries[{entry_index}].module"),
                    failed.as_dotted(),
                )),
                PackageVerificationMode::Reference,
            ));
            continue;
        }

        let resolved_imports = &graph.resolved_entry_imports[*entry_index];
        match verify_reference_lock_entry(
            *entry_index,
            entry,
            resolved_imports,
            &artifact_bytes,
            &checked_by_module,
            &policy,
        ) {
            Ok(checked) => {
                checked_by_module.insert(entry.module.clone(), checked);
                results.push(module_result(
                    entry,
                    PackageModuleVerificationStatus::Passed,
                    None,
                    PackageVerificationMode::Reference,
                ));
            }
            Err(error) => {
                failed_module = Some(entry.module.clone());
                results.push(module_result(
                    entry,
                    PackageModuleVerificationStatus::Failed,
                    Some(error),
                    PackageVerificationMode::Reference,
                ));
            }
        }
    }

    let status = if failed_module.is_some() {
        PackageVerificationStatus::Failed
    } else {
        PackageVerificationStatus::Passed
    };
    let verdict_source = PackageVerificationVerdictSource::ReferenceChecker;

    Ok(PackageVerificationReport {
        mode: PackageVerificationMode::Reference,
        verdict_source,
        reference_checker_verdict: verdict_source.is_reference_checker_verdict(),
        status,
        topological_order: graph.topological_order,
        modules: results,
    })
}

/// Materialize one Phase 8 import lock per package-lock entry.
///
/// Each generated import lock contains exactly the module's direct certificate
/// imports from the package lock. No source, replay, metadata, theorem-index,
/// AI trace, registry, or solver data is introduced.
pub fn materialize_package_phase8_import_locks(
    lock: &PackageLockManifest,
    checker_profile: &str,
) -> PackageVerificationResult<Vec<PackagePhase8ImportLockMaterialization>> {
    let graph = build_package_lock_graph(lock)
        .map_err(|error| PackageVerificationError::lock_graph_invalid(format!("{error:?}")))?;
    let entries = canonical_lock_entries(lock);
    let entries_by_module = entries
        .iter()
        .map(|(index, entry)| (entry.module.clone(), (*index, *entry)))
        .collect::<BTreeMap<_, _>>();
    let mut materialized = Vec::with_capacity(graph.topological_order.len());

    for module in &graph.topological_order {
        let (entry_index, entry) = entries_by_module
            .get(module)
            .expect("lock graph order only contains lock entries");
        let import_lock = materialize_phase8_import_lock_for_entry(
            lock,
            *entry_index,
            entry,
            &graph.resolved_entry_imports[*entry_index],
            &entries,
            checker_profile,
        )?;
        materialized.push(import_lock);
    }

    Ok(materialized)
}

/// Materialize Phase 8 machine-check requests for every package-lock entry.
///
/// This derives per-module direct-import locks from the package lock and then
/// delegates request construction to the existing Phase 8 request materializer,
/// preserving request-hash recomputation and request-store behavior.
pub fn materialize_package_phase8_requests<'a>(
    lock: &PackageLockManifest,
    artifacts: impl IntoIterator<Item = PackageCertificateArtifact<'a>>,
    policy: &IndependentCheckerRunnerPolicy,
    checker_profile: &str,
    existing_store: Option<&IndependentCheckerRequestStoreManifest>,
) -> PackageVerificationResult<PackagePhase8RequestMaterializationReport> {
    let graph = build_package_lock_graph(lock)
        .map_err(|error| PackageVerificationError::lock_graph_invalid(format!("{error:?}")))?;
    let artifact_bytes = artifact_byte_map(artifacts)?;
    let entries = canonical_lock_entries(lock);
    let entries_by_module = entries
        .iter()
        .map(|(index, entry)| (entry.module.clone(), (*index, *entry)))
        .collect::<BTreeMap<_, _>>();
    let mut current_store =
        existing_store
            .cloned()
            .unwrap_or(IndependentCheckerRequestStoreManifest {
                requests: Vec::new(),
            });
    let mut request_store_file_hash =
        independent_checker_file_hash(current_store.canonical_json().as_bytes());
    let mut request_store_rewrite_required = false;
    let mut modules = Vec::with_capacity(graph.topological_order.len());

    for module in &graph.topological_order {
        let (entry_index, entry) = entries_by_module
            .get(module)
            .expect("lock graph order only contains lock entries");
        let bytes = artifact_bytes
            .get(&entry.certificate)
            .copied()
            .ok_or_else(|| {
                PackageVerificationError::certificate_artifact_missing(
                    format!("entries[{entry_index}].certificate"),
                    entry.certificate.as_str(),
                )
            })?;
        let actual_file_hash = package_file_hash(bytes);
        if entry.certificate_file_hash != actual_file_hash {
            return Err(PackageVerificationError::certificate_file_hash_mismatch(
                format!("entries[{entry_index}].certificate_file_hash"),
                entry.certificate_file_hash,
                actual_file_hash,
            ));
        }

        let import_lock = materialize_phase8_import_lock_for_entry(
            lock,
            *entry_index,
            entry,
            &graph.resolved_entry_imports[*entry_index],
            &entries,
            checker_profile,
        )?;
        let import_lock_json = import_lock.manifest.canonical_json();
        let request_id = package_phase8_request_id(lock, &entry.module, checker_profile);
        let request_path = package_phase8_request_path(lock, &entry.module, checker_profile);
        let materialized = independent_checker_request_materialize(
            policy,
            entry.module.as_dotted(),
            entry.certificate.as_str(),
            bytes,
            &import_lock.path,
            import_lock_json.as_bytes(),
            import_lock.manifest_hash,
            checker_profile,
            &request_id,
            &request_path,
            Some(&current_store),
        )
        .map_err(|error| {
            PackageVerificationError::phase8_request_materialization_failed(
                format!("entries[{entry_index}].independent_checker_request"),
                error,
            )
        })?;

        let actual_certificate_hash =
            PackageHash::from(materialized.request.certificate.expected_certificate_hash);
        if actual_certificate_hash != entry.certificate_hash {
            return Err(PackageVerificationError::certificate_hash_mismatch(
                format!("entries[{entry_index}].certificate_hash"),
                entry.certificate_hash,
                actual_certificate_hash,
            ));
        }

        request_store_rewrite_required |= materialized.request_store_rewrite_required;
        current_store = materialized.request_store.clone();
        request_store_file_hash = materialized.request_store_file_hash;
        modules.push(PackagePhase8RequestMaterialization {
            module: entry.module.clone(),
            checker_profile: checker_profile.to_owned(),
            import_lock_path: import_lock.path,
            import_lock_manifest: import_lock.manifest,
            import_lock_manifest_hash: import_lock.manifest_hash,
            request_path,
            request: materialized.request,
            request_file_hash: materialized.request_file_hash,
        });
    }

    Ok(PackagePhase8RequestMaterializationReport {
        modules,
        request_store: current_store,
        request_store_file_hash,
        request_store_rewrite_required,
    })
}

fn materialize_phase8_import_lock_for_entry(
    lock: &PackageLockManifest,
    entry_index: usize,
    entry: &PackageLockEntry,
    resolved_imports: &[PackageLockResolvedImport],
    entries: &[(usize, &PackageLockEntry)],
    checker_profile: &str,
) -> PackageVerificationResult<PackagePhase8ImportLockMaterialization> {
    let mut imports = resolved_imports
        .iter()
        .map(|import| {
            let import_entry = entries
                .get(import.entry_index)
                .map(|(_, entry)| *entry)
                .expect("resolved import index points into canonical lock entries");
            IndependentCheckerImportLockEntry {
                module: import.module.as_dotted(),
                export_hash: import.export_hash.into_bytes(),
                certificate: IndependentCheckerImportLockCertificate {
                    path: import_entry.certificate.as_str().to_owned(),
                    file_hash: import_entry.certificate_file_hash.into_bytes(),
                    certificate_hash: import.certificate_hash.into_bytes(),
                },
            }
        })
        .collect::<Vec<_>>();
    imports.sort_by(|left, right| {
        phase8_import_lock_module_sort_key(&left.module)
            .cmp(&phase8_import_lock_module_sort_key(&right.module))
            .then_with(|| left.certificate.path.cmp(&right.certificate.path))
            .then_with(|| {
                left.certificate
                    .certificate_hash
                    .cmp(&right.certificate.certificate_hash)
            })
            .then_with(|| left.certificate.file_hash.cmp(&right.certificate.file_hash))
    });
    let manifest = IndependentCheckerImportLockManifest { imports };
    let manifest_json = manifest.canonical_json();
    parse_independent_checker_import_lock_manifest(&manifest_json).map_err(|error| {
        PackageVerificationError::phase8_import_lock_invalid(
            format!("entries[{entry_index}].independent_checker_import_lock"),
            format!("{error:?}"),
        )
    })?;
    let manifest_hash = independent_checker_file_hash(manifest_json.as_bytes());

    Ok(PackagePhase8ImportLockMaterialization {
        module: entry.module.clone(),
        path: package_phase8_import_lock_path(lock, &entry.module, checker_profile),
        manifest,
        manifest_hash,
    })
}

fn phase8_import_lock_module_sort_key(module: &str) -> Vec<u8> {
    parse_module_name_wire(module)
        .and_then(|name| machine_api_name_canonical_bytes(&name))
        .unwrap_or_else(|_| module.as_bytes().to_vec())
}

fn package_phase8_request_id(
    lock: &PackageLockManifest,
    module: &Name,
    checker_profile: &str,
) -> String {
    format!(
        "package:{}:{}:{}:{}",
        lock.package.as_str(),
        lock.version.as_str(),
        module.as_dotted(),
        checker_profile
    )
}

fn package_phase8_import_lock_path(
    lock: &PackageLockManifest,
    module: &Name,
    checker_profile: &str,
) -> String {
    format!(
        "{}/imports.json",
        package_phase8_module_dir(lock, module, checker_profile)
    )
}

fn package_phase8_request_path(
    lock: &PackageLockManifest,
    module: &Name,
    checker_profile: &str,
) -> String {
    format!(
        "{}/request.json",
        package_phase8_module_dir(lock, module, checker_profile)
    )
}

fn package_phase8_module_dir(
    lock: &PackageLockManifest,
    module: &Name,
    checker_profile: &str,
) -> String {
    format!(
        "generated/checker-requests/{}/{}/{}/{}",
        lock.package.as_str(),
        lock.version.as_str(),
        module.as_dotted(),
        checker_profile
    )
}

fn validate_manifest_lock_identity(
    validated: &ValidatedPackageManifest,
    lock: &PackageLockManifest,
) -> PackageVerificationResult<()> {
    let manifest = validated.manifest();
    if lock.package != manifest.package {
        return Err(PackageVerificationError::package_identity_mismatch(
            "package",
            "package",
            manifest.package.as_str(),
            lock.package.as_str(),
        ));
    }
    if lock.version != manifest.version {
        return Err(PackageVerificationError::package_identity_mismatch(
            "version",
            "version",
            manifest.version.as_str(),
            lock.version.as_str(),
        ));
    }
    Ok(())
}

fn artifact_byte_map<'a>(
    artifacts: impl IntoIterator<Item = PackageCertificateArtifact<'a>>,
) -> PackageVerificationResult<BTreeMap<PackagePath, &'a [u8]>> {
    let mut artifact_bytes = BTreeMap::new();
    for artifact in artifacts {
        if artifact_bytes
            .insert(artifact.path.clone(), artifact.bytes)
            .is_some()
        {
            return Err(PackageVerificationError::duplicate_certificate_artifact(
                "artifacts",
                artifact.path.as_str(),
            ));
        }
    }
    Ok(artifact_bytes)
}

fn canonical_lock_entries(lock: &PackageLockManifest) -> Vec<(usize, &PackageLockEntry)> {
    let mut entries = lock.entries.iter().collect::<Vec<_>>();
    entries.sort_by(|left, right| left.module.cmp(&right.module));
    entries.into_iter().enumerate().collect()
}

fn package_fast_kernel_policy(validated: &ValidatedPackageManifest) -> AxiomPolicy {
    let package_policy = &validated.manifest().policy;
    let mut policy = if package_policy.allow_custom_axioms {
        AxiomPolicy::normal()
    } else {
        let mut policy = AxiomPolicy::high_trust();
        policy
            .allowlisted_axioms
            .extend(package_policy.allowed_axioms.iter().cloned());
        policy
    };
    policy.supported_core_features.extend([
        CoreFeature::QuotientV1,
        CoreFeature::QuotientV2,
        CoreFeature::QuotientV3,
    ]);
    policy
}

fn package_reference_checker_policy(
    validated: &ValidatedPackageManifest,
) -> ReferenceCheckerPolicy {
    let package_policy = &validated.manifest().policy;
    ReferenceCheckerPolicy {
        trust_mode: ReferenceTrustMode::HighTrust,
        allowed_axioms: package_policy
            .allowed_axioms
            .iter()
            .map(Name::as_dotted)
            .collect(),
        deny_sorry: true,
        deny_custom_axioms: !package_policy.allow_custom_axioms,
        supported_core_features: reference_checker_supported_core_features(
            &validated.manifest().checker_profile,
        ),
    }
}

fn reference_checker_supported_core_features(profile: &str) -> Vec<ReferenceCoreFeature> {
    match profile {
        CHECKER_PROFILE_REFERENCE_V0_1 => vec![
            ReferenceCoreFeature::QuotientV1,
            ReferenceCoreFeature::QuotientV2,
            ReferenceCoreFeature::QuotientV3,
        ],
        _ => Vec::new(),
    }
}

fn verify_lock_entry(
    entry_index: usize,
    entry: &PackageLockEntry,
    artifact_bytes: &BTreeMap<PackagePath, &[u8]>,
    session: &mut VerifierSession,
    policy: &AxiomPolicy,
) -> PackageVerificationResult<VerifiedModule> {
    let entry_path = format!("entries[{entry_index}]");
    let bytes = artifact_bytes
        .get(&entry.certificate)
        .copied()
        .ok_or_else(|| {
            PackageVerificationError::certificate_artifact_missing(
                format!("{entry_path}.certificate"),
                entry.certificate.as_str(),
            )
        })?;
    let actual_file_hash = package_file_hash(bytes);
    if entry.certificate_file_hash != actual_file_hash {
        return Err(PackageVerificationError::certificate_file_hash_mismatch(
            format!("{entry_path}.certificate_file_hash"),
            entry.certificate_file_hash,
            actual_file_hash,
        ));
    }

    let cert = decode_module_cert(bytes).map_err(|source| {
        PackageVerificationError::certificate_decode_failed(
            format!("{entry_path}.certificate"),
            format!("{source:?}"),
        )
    })?;
    if cert.header.module != entry.module {
        return Err(PackageVerificationError::certificate_module_mismatch(
            format!("{entry_path}.certificate"),
            entry.module.as_dotted(),
            cert.header.module.as_dotted(),
        ));
    }
    check_entry_hashes(entry_index, entry, &cert)?;

    let verified = verify_module_cert(bytes, session, policy).map_err(|source| {
        PackageVerificationError::verify_failed(format!("{entry_path}.certificate"), source)
    })?;
    if verified.module() != &entry.module {
        return Err(PackageVerificationError::certificate_module_mismatch(
            format!("{entry_path}.certificate"),
            entry.module.as_dotted(),
            verified.module().as_dotted(),
        ));
    }
    let actual_export_hash = PackageHash::from(verified.export_hash());
    if actual_export_hash != entry.export_hash {
        return Err(PackageVerificationError::export_hash_mismatch(
            format!("{entry_path}.export_hash"),
            entry.export_hash,
            actual_export_hash,
        ));
    }
    let actual_certificate_hash = PackageHash::from(verified.certificate_hash());
    if actual_certificate_hash != entry.certificate_hash {
        return Err(PackageVerificationError::certificate_hash_mismatch(
            format!("{entry_path}.certificate_hash"),
            entry.certificate_hash,
            actual_certificate_hash,
        ));
    }

    Ok(verified)
}

fn verify_reference_lock_entry(
    entry_index: usize,
    entry: &PackageLockEntry,
    resolved_imports: &[PackageLockResolvedImport],
    artifact_bytes: &BTreeMap<PackagePath, &[u8]>,
    checked_by_module: &BTreeMap<Name, ReferenceCheckedModule>,
    policy: &ReferenceCheckerPolicy,
) -> PackageVerificationResult<ReferenceCheckedModule> {
    let entry_path = format!("entries[{entry_index}]");
    let bytes = artifact_bytes
        .get(&entry.certificate)
        .copied()
        .ok_or_else(|| {
            PackageVerificationError::certificate_artifact_missing(
                format!("{entry_path}.certificate"),
                entry.certificate.as_str(),
            )
        })?;
    let actual_file_hash = package_file_hash(bytes);
    if entry.certificate_file_hash != actual_file_hash {
        return Err(PackageVerificationError::certificate_file_hash_mismatch(
            format!("{entry_path}.certificate_file_hash"),
            entry.certificate_file_hash,
            actual_file_hash,
        ));
    }

    let decoded = verify_certificate_hashes(bytes).map_err(|source| {
        PackageVerificationError::reference_checker_rejected(
            format!("{entry_path}.certificate"),
            source,
        )
    })?;
    let actual_module = reference_name_to_package_name(&decoded.header().module);
    if actual_module != entry.module {
        return Err(PackageVerificationError::certificate_module_mismatch(
            format!("{entry_path}.certificate"),
            entry.module.as_dotted(),
            actual_module.as_dotted(),
        ));
    }
    check_reference_entry_hashes(entry_index, entry, decoded.hashes())?;

    let import_modules = resolved_imports
        .iter()
        .map(|import| {
            checked_by_module
                .get(&import.module)
                .cloned()
                .ok_or_else(|| {
                    PackageVerificationError::earlier_module_failed(
                        format!("{entry_path}.imports"),
                        import.module.as_dotted(),
                    )
                })
        })
        .collect::<PackageVerificationResult<Vec<_>>>()?;
    let imports = ReferenceImportStore::from_checked_modules(import_modules).map_err(|source| {
        PackageVerificationError::reference_checker_rejected(
            format!("{entry_path}.imports"),
            source,
        )
    })?;
    let checked = match check_certificate(bytes, &imports, policy) {
        ReferenceCheckResult::Checked(checked) => checked,
        ReferenceCheckResult::Rejected(error) => {
            return Err(PackageVerificationError::reference_checker_rejected(
                format!("{entry_path}.certificate"),
                error,
            ));
        }
    };

    let actual_module = reference_name_to_package_name(checked.module());
    if actual_module != entry.module {
        return Err(PackageVerificationError::certificate_module_mismatch(
            format!("{entry_path}.certificate"),
            entry.module.as_dotted(),
            actual_module.as_dotted(),
        ));
    }
    let actual_export_hash = PackageHash::from(*checked.export_hash());
    if actual_export_hash != entry.export_hash {
        return Err(PackageVerificationError::export_hash_mismatch(
            format!("{entry_path}.export_hash"),
            entry.export_hash,
            actual_export_hash,
        ));
    }
    let actual_axiom_report_hash = PackageHash::from(*checked.axiom_report_hash());
    if actual_axiom_report_hash != entry.axiom_report_hash {
        return Err(PackageVerificationError::axiom_report_hash_mismatch(
            format!("{entry_path}.axiom_report_hash"),
            entry.axiom_report_hash,
            actual_axiom_report_hash,
        ));
    }
    let actual_certificate_hash = PackageHash::from(*checked.certificate_hash());
    if actual_certificate_hash != entry.certificate_hash {
        return Err(PackageVerificationError::certificate_hash_mismatch(
            format!("{entry_path}.certificate_hash"),
            entry.certificate_hash,
            actual_certificate_hash,
        ));
    }

    Ok(checked)
}

fn check_entry_hashes(
    entry_index: usize,
    entry: &PackageLockEntry,
    cert: &npa_cert::ModuleCert,
) -> PackageVerificationResult<()> {
    let entry_path = format!("entries[{entry_index}]");
    let actual_export_hash = PackageHash::from(cert.hashes.export_hash);
    if entry.export_hash != actual_export_hash {
        return Err(PackageVerificationError::export_hash_mismatch(
            format!("{entry_path}.export_hash"),
            entry.export_hash,
            actual_export_hash,
        ));
    }
    let actual_axiom_report_hash = PackageHash::from(cert.hashes.axiom_report_hash);
    if entry.axiom_report_hash != actual_axiom_report_hash {
        return Err(PackageVerificationError::axiom_report_hash_mismatch(
            format!("{entry_path}.axiom_report_hash"),
            entry.axiom_report_hash,
            actual_axiom_report_hash,
        ));
    }
    let actual_certificate_hash = PackageHash::from(cert.hashes.certificate_hash);
    if entry.certificate_hash != actual_certificate_hash {
        return Err(PackageVerificationError::certificate_hash_mismatch(
            format!("{entry_path}.certificate_hash"),
            entry.certificate_hash,
            actual_certificate_hash,
        ));
    }

    Ok(())
}

fn check_reference_entry_hashes(
    entry_index: usize,
    entry: &PackageLockEntry,
    hashes: &npa_checker_ref::ReferenceModuleHashes,
) -> PackageVerificationResult<()> {
    let entry_path = format!("entries[{entry_index}]");
    let actual_export_hash = PackageHash::from(hashes.export_hash);
    if entry.export_hash != actual_export_hash {
        return Err(PackageVerificationError::export_hash_mismatch(
            format!("{entry_path}.export_hash"),
            entry.export_hash,
            actual_export_hash,
        ));
    }
    let actual_axiom_report_hash = PackageHash::from(hashes.axiom_report_hash);
    if entry.axiom_report_hash != actual_axiom_report_hash {
        return Err(PackageVerificationError::axiom_report_hash_mismatch(
            format!("{entry_path}.axiom_report_hash"),
            entry.axiom_report_hash,
            actual_axiom_report_hash,
        ));
    }
    let actual_certificate_hash = PackageHash::from(hashes.certificate_hash);
    if entry.certificate_hash != actual_certificate_hash {
        return Err(PackageVerificationError::certificate_hash_mismatch(
            format!("{entry_path}.certificate_hash"),
            entry.certificate_hash,
            actual_certificate_hash,
        ));
    }

    Ok(())
}

fn reference_name_to_package_name(name: &ReferenceModuleName) -> Name {
    Name(name.components().to_vec())
}

fn package_reference_checker_reason(
    source: &ReferenceCheckError,
) -> PackageVerificationErrorReason {
    if source.kind == ReferenceCheckErrorKind::UnsupportedCoreFeature
        || source.reason == Some(ReferenceCheckReason::UnsupportedCoreFeature)
    {
        return PackageVerificationErrorReason::UnsupportedCoreFeature;
    }
    if matches!(
        source.reason,
        Some(ReferenceCheckReason::ForbiddenAxiom | ReferenceCheckReason::SorryDenied)
    ) {
        return PackageVerificationErrorReason::AxiomPolicyRejected;
    }
    if source.kind == ReferenceCheckErrorKind::AxiomPolicy {
        return PackageVerificationErrorReason::AxiomPolicyRejected;
    }
    PackageVerificationErrorReason::ReferenceCheckerRejected
}

fn reference_checker_error_details(
    source: &ReferenceCheckError,
) -> PackageVerificationCheckerError {
    PackageVerificationCheckerError {
        checker: "npa-checker-ref".to_owned(),
        kind: reference_check_error_kind_code(source.kind).to_owned(),
        section: Some(reference_certificate_section_code(source.section).to_owned()),
        offset: Some(source.offset),
        reason_code: source
            .reason
            .map(reference_check_reason_code)
            .map(str::to_owned),
    }
}

fn reference_check_error_kind_code(kind: ReferenceCheckErrorKind) -> &'static str {
    match kind {
        ReferenceCheckErrorKind::EmptyCertificate => "empty_certificate",
        ReferenceCheckErrorKind::MalformedCertificate => "malformed_certificate",
        ReferenceCheckErrorKind::HashMismatch => "hash_mismatch",
        ReferenceCheckErrorKind::ImportResolution => "import_resolution",
        ReferenceCheckErrorKind::AxiomReportMismatch => "axiom_report_mismatch",
        ReferenceCheckErrorKind::AxiomPolicy => "axiom_policy",
        ReferenceCheckErrorKind::TypeCheck => "type_check",
        ReferenceCheckErrorKind::UnsupportedSkeleton => "unsupported_skeleton",
        ReferenceCheckErrorKind::UnsupportedCoreFeature => "unsupported_core_feature",
    }
}

fn reference_certificate_section_code(section: ReferenceCertificateSection) -> &'static str {
    match section {
        ReferenceCertificateSection::HeaderFormat => "header_format",
        ReferenceCertificateSection::HeaderCoreSpec => "header_core_spec",
        ReferenceCertificateSection::HeaderModule => "header_module",
        ReferenceCertificateSection::Imports => "imports",
        ReferenceCertificateSection::NameTable => "name_table",
        ReferenceCertificateSection::LevelTable => "level_table",
        ReferenceCertificateSection::TermTable => "term_table",
        ReferenceCertificateSection::Declarations => "declarations",
        ReferenceCertificateSection::ExportBlock => "export_block",
        ReferenceCertificateSection::AxiomReport => "axiom_report",
        ReferenceCertificateSection::Hashes => "hashes",
        ReferenceCertificateSection::ImportStore => "import_store",
        ReferenceCertificateSection::FullCertificate => "full_certificate",
    }
}

fn reference_check_reason_code(reason: ReferenceCheckReason) -> &'static str {
    match reason {
        ReferenceCheckReason::UnexpectedEof => "unexpected_eof",
        ReferenceCheckReason::NonCanonicalUvar => "non_canonical_uvar",
        ReferenceCheckReason::UvarOverflow => "uvar_overflow",
        ReferenceCheckReason::LengthOverflow => "length_overflow",
        ReferenceCheckReason::UnknownTag { .. } => "unknown_tag",
        ReferenceCheckReason::InvalidUtf8 => "invalid_utf8",
        ReferenceCheckReason::FormatMismatch => "format_mismatch",
        ReferenceCheckReason::CoreSpecMismatch => "core_spec_mismatch",
        ReferenceCheckReason::EmptyModuleName => "empty_module_name",
        ReferenceCheckReason::EmptyModuleNameComponent => "empty_module_name_component",
        ReferenceCheckReason::DottedNameComponent => "dotted_name_component",
        ReferenceCheckReason::DanglingReference => "dangling_reference",
        ReferenceCheckReason::NonCanonicalOrder => "non_canonical_order",
        ReferenceCheckReason::DuplicateName => "duplicate_name",
        ReferenceCheckReason::DuplicateDeclarationName => "duplicate_declaration_name",
        ReferenceCheckReason::ReservedCorePrimitive => "reserved_core_primitive",
        ReferenceCheckReason::DuplicateImport => "duplicate_import",
        ReferenceCheckReason::NonNormalizedLevel => "non_normalized_level",
        ReferenceCheckReason::NonNormalizedTerm => "non_normalized_term",
        ReferenceCheckReason::UnusedTableEntry => "unused_table_entry",
        ReferenceCheckReason::TrailingBytes => "trailing_bytes",
        ReferenceCheckReason::MissingImport => "missing_import",
        ReferenceCheckReason::ImportExportHashMismatch => "import_export_hash_mismatch",
        ReferenceCheckReason::MissingImportCertificateHash => "missing_import_certificate_hash",
        ReferenceCheckReason::ImportCertificateHashMismatch => "import_certificate_hash_mismatch",
        ReferenceCheckReason::UncheckedImport => "unchecked_import",
        ReferenceCheckReason::UnknownReference => "unknown_reference",
        ReferenceCheckReason::UnsupportedCoreFeature => "unsupported_core_feature",
        ReferenceCheckReason::BadUniverseArity => "bad_universe_arity",
        ReferenceCheckReason::DuplicateUniverseParam => "duplicate_universe_param",
        ReferenceCheckReason::UnresolvedMetavariable => "unresolved_metavariable",
        ReferenceCheckReason::InvalidBVar => "invalid_bvar",
        ReferenceCheckReason::ExpectedSort => "expected_sort",
        ReferenceCheckReason::ExpectedFunction => "expected_function",
        ReferenceCheckReason::TypeMismatch => "type_mismatch",
        ReferenceCheckReason::ResourceLimit => "resource_limit",
        ReferenceCheckReason::BadConstructorResult => "bad_constructor_result",
        ReferenceCheckReason::NonPositiveOccurrence => "non_positive_occurrence",
        ReferenceCheckReason::BadRecursorRule => "bad_recursor_rule",
        ReferenceCheckReason::BadRecursorParam => "bad_recursor_param",
        ReferenceCheckReason::BadRecursorMotive => "bad_recursor_motive",
        ReferenceCheckReason::BadRecursorMajor => "bad_recursor_major",
        ReferenceCheckReason::BadRecursorMinor => "bad_recursor_minor",
        ReferenceCheckReason::BadRecursorResult => "bad_recursor_result",
        ReferenceCheckReason::BadRecursorType => "bad_recursor_type",
        ReferenceCheckReason::HashMismatch { .. } => "hash_mismatch",
        ReferenceCheckReason::AxiomReportMismatch => "axiom_report_mismatch",
        ReferenceCheckReason::SorryDenied => "sorry_denied",
        ReferenceCheckReason::ForbiddenAxiom => "forbidden_axiom",
        ReferenceCheckReason::ReferenceCheckerBodyUnimplemented => {
            "reference_checker_body_unimplemented"
        }
    }
}

fn module_result(
    entry: &PackageLockEntry,
    status: PackageModuleVerificationStatus,
    error: Option<PackageVerificationError>,
    checker_mode: PackageVerificationMode,
) -> PackageModuleVerificationResult {
    PackageModuleVerificationResult {
        module: entry.module.clone(),
        checker_mode,
        status,
        export_hash: entry.export_hash,
        axiom_report_hash: entry.axiom_report_hash,
        certificate_hash: entry.certificate_hash,
        error,
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, BTreeSet},
        fs,
        path::{Path, PathBuf},
    };

    use npa_package::{
        parse_and_validate_manifest_str, parse_manifest_str, parse_package_lock_json,
        validate_manifest, PackageLockManifest, PackagePath, ValidatedPackageManifest,
    };

    use crate::independent_checker::{
        independent_checker_machine_check_request_hash,
        parse_independent_checker_import_lock_manifest,
        parse_independent_checker_machine_check_request,
        parse_independent_checker_request_store_manifest, IndependentCheckerAllowlistEntry,
        IndependentCheckerRunnerAxiomPolicy, IndependentCheckerRunnerBudget,
        IndependentCheckerRunnerImportPolicy, IndependentCheckerRunnerPolicy,
        IndependentCheckerTrustMode,
    };

    use super::*;

    const PACKAGE_FAST_VERIFIER_TEST_STACK_BYTES: usize = 64 * 1024 * 1024;

    fn run_on_large_stack(name: &str, test: impl FnOnce() + Send + 'static) {
        std::thread::Builder::new()
            .name(name.to_owned())
            .stack_size(PACKAGE_FAST_VERIFIER_TEST_STACK_BYTES)
            .spawn(test)
            .expect("package fast verifier test thread should spawn")
            .join()
            .expect("package fast verifier test thread should not panic");
    }

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("npa-api crate lives under crates/")
            .to_path_buf()
    }

    fn proofs_root() -> PathBuf {
        repo_root().join("proofs")
    }

    fn read(path: PathBuf) -> Vec<u8> {
        fs::read(&path).unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
    }

    fn read_to_string(path: PathBuf) -> String {
        String::from_utf8(read(path)).expect("fixture is UTF-8")
    }

    fn proof_manifest_source() -> String {
        read_to_string(proofs_root().join("npa-package.toml"))
    }

    fn validated_proof_manifest() -> ValidatedPackageManifest {
        parse_and_validate_manifest_str(&proof_manifest_source()).unwrap()
    }

    fn proof_lock() -> PackageLockManifest {
        parse_package_lock_json(&read_to_string(
            proofs_root().join("generated/package-lock.json"),
        ))
        .unwrap()
    }

    fn proof_certificate_artifacts(lock: &PackageLockManifest) -> BTreeMap<PackagePath, Vec<u8>> {
        let root = proofs_root();
        lock.entries
            .iter()
            .map(|entry| {
                (
                    entry.certificate.clone(),
                    read(root.join(entry.certificate.as_str())),
                )
            })
            .collect()
    }

    fn package_certificate_artifacts(
        artifacts: &BTreeMap<PackagePath, Vec<u8>>,
    ) -> Vec<PackageCertificateArtifact<'_>> {
        artifacts
            .iter()
            .map(|(path, bytes)| PackageCertificateArtifact {
                path: path.clone(),
                bytes: bytes.as_slice(),
            })
            .collect()
    }

    fn test_hash(byte: u8) -> npa_cert::Hash {
        [byte; 32]
    }

    fn phase8_reference_runner_policy() -> IndependentCheckerRunnerPolicy {
        IndependentCheckerRunnerPolicy {
            id: "package-reference-check".to_owned(),
            version: 1,
            trust_mode: IndependentCheckerTrustMode::Pr,
            required_checker_profiles: vec!["reference".to_owned()],
            optional_checker_profiles: Vec::new(),
            checker_allowlist: vec![IndependentCheckerAllowlistEntry {
                profile: "reference".to_owned(),
                checker_id: "npa-checker-ref".to_owned(),
                binary_id: "npa-checker-ref-test".to_owned(),
                binary_hash: test_hash(10),
                build_hash: test_hash(11),
                allowed_args: vec!["--json".to_owned(), "--canonical-only".to_owned()],
            }],
            checker_identity_manifest: None,
            import_policy: IndependentCheckerRunnerImportPolicy {
                mode: "locked_store".to_owned(),
                network: "forbidden".to_owned(),
                require_import_lock_hash: true,
            },
            axiom_policy: IndependentCheckerRunnerAxiomPolicy {
                path: "generated/checker-requests/axiom-policy.toml".to_owned(),
                hash: test_hash(12),
            },
            budgets: BTreeMap::from([(
                "reference".to_owned(),
                IndependentCheckerRunnerBudget {
                    max_steps: 10_000_000,
                    max_memory_mb: 2048,
                    timeout_ms: 60_000,
                },
            )]),
        }
    }

    fn verify_proof_package(
        validated: &ValidatedPackageManifest,
        lock: &PackageLockManifest,
        artifacts: &BTreeMap<PackagePath, Vec<u8>>,
    ) -> PackageVerificationResult<PackageVerificationReport> {
        verify_package_fast_source_free(validated, lock, package_certificate_artifacts(artifacts))
    }

    fn verify_proof_package_reference(
        validated: &ValidatedPackageManifest,
        lock: &PackageLockManifest,
        artifacts: &BTreeMap<PackagePath, Vec<u8>>,
    ) -> PackageVerificationResult<PackageVerificationReport> {
        verify_package_reference_source_free(
            validated,
            lock,
            package_certificate_artifacts(artifacts),
        )
    }

    #[test]
    fn package_fast_verifier_verifies_proof_package_source_free() {
        run_on_large_stack(
            "package_fast_verifier_verifies_proof_package_source_free",
            package_fast_verifier_verifies_proof_package_source_free_on_large_stack,
        );
    }

    fn package_fast_verifier_verifies_proof_package_source_free_on_large_stack() {
        let mut manifest = parse_manifest_str(&proof_manifest_source()).unwrap();
        for module in &mut manifest.modules {
            let module_path = module.module.as_dotted().replace('.', "/");
            module.source = PackagePath::new(format!("missing/source/{module_path}.npa"));
            module.meta = Some(PackagePath::new(format!("missing/meta/{module_path}.json")));
            module.replay = Some(PackagePath::new(format!(
                "missing/replay/{module_path}.json"
            )));
        }
        let validated = validate_manifest(manifest).unwrap();
        let lock = proof_lock();
        let artifacts = proof_certificate_artifacts(&lock);

        let report = verify_proof_package(&validated, &lock, &artifacts).unwrap();

        assert_eq!(report.status, PackageVerificationStatus::Passed);
        assert_eq!(report.mode, PackageVerificationMode::FastKernel);
        assert_eq!(
            report.verdict_source,
            PackageVerificationVerdictSource::FastKernelCertificateVerifier
        );
        assert!(!report.reference_checker_verdict);
        assert_eq!(report.modules.len(), lock.entries.len());
        assert!(report
            .modules
            .iter()
            .all(|module| module.status == PackageModuleVerificationStatus::Passed));
    }

    #[test]
    fn package_fast_verifier_rejects_missing_certificate_artifact() {
        let validated = validated_proof_manifest();
        let lock = proof_lock();
        let mut artifacts = proof_certificate_artifacts(&lock);
        let missing_path = lock
            .entries
            .iter()
            .find(|entry| entry.module.as_dotted() == "Std.Logic.Eq")
            .expect("proof lock contains Std.Logic.Eq")
            .certificate
            .clone();
        artifacts.remove(&missing_path);

        let report = verify_proof_package(&validated, &lock, &artifacts).unwrap();
        let failed = report
            .modules
            .iter()
            .find(|module| module.status == PackageModuleVerificationStatus::Failed)
            .expect("one module fails");

        assert_eq!(report.status, PackageVerificationStatus::Failed);
        assert_eq!(
            failed.error.as_ref().unwrap().reason_code,
            PackageVerificationErrorReason::CertificateArtifactMissing
        );
    }

    #[test]
    fn package_fast_verifier_rejects_stale_certificate_file_hash() {
        let validated = validated_proof_manifest();
        let lock = proof_lock();
        let mut artifacts = proof_certificate_artifacts(&lock);
        let stale_path = lock
            .entries
            .iter()
            .find(|entry| entry.module.as_dotted() == "Std.Logic.Eq")
            .expect("proof lock contains Std.Logic.Eq")
            .certificate
            .clone();
        artifacts.get_mut(&stale_path).unwrap()[0] ^= 0x01;

        let report = verify_proof_package(&validated, &lock, &artifacts).unwrap();
        let failed = report
            .modules
            .iter()
            .find(|module| module.status == PackageModuleVerificationStatus::Failed)
            .expect("one module fails");

        assert_eq!(report.status, PackageVerificationStatus::Failed);
        assert_eq!(
            failed.error.as_ref().unwrap().reason_code,
            PackageVerificationErrorReason::CertificateFileHashMismatch
        );
    }

    #[test]
    fn package_fast_verifier_rejects_disallowed_axioms_from_certificate() {
        let mut manifest = parse_manifest_str(&proof_manifest_source()).unwrap();
        manifest.policy.allowed_axioms.clear();
        for module in &mut manifest.modules {
            module.axioms = Some(Vec::new());
        }
        let validated = validate_manifest(manifest).unwrap();
        let lock = proof_lock();
        let artifacts = proof_certificate_artifacts(&lock);

        let report = verify_proof_package(&validated, &lock, &artifacts).unwrap();
        let failed = report
            .modules
            .iter()
            .find(|module| module.status == PackageModuleVerificationStatus::Failed)
            .expect("one module fails");

        assert_eq!(report.status, PackageVerificationStatus::Failed);
        assert_eq!(
            failed.error.as_ref().unwrap().reason_code,
            PackageVerificationErrorReason::AxiomPolicyRejected
        );
        assert!(failed
            .error
            .as_ref()
            .unwrap()
            .actual_value
            .as_ref()
            .unwrap()
            .contains("ForbiddenAxiom"));
    }

    #[test]
    fn package_fast_verifier_uses_lock_topological_order_not_lock_entry_order() {
        run_on_large_stack(
            "package_fast_verifier_uses_lock_topological_order_not_lock_entry_order",
            package_fast_verifier_uses_lock_topological_order_not_lock_entry_order_on_large_stack,
        );
    }

    fn package_fast_verifier_uses_lock_topological_order_not_lock_entry_order_on_large_stack() {
        let validated = validated_proof_manifest();
        let mut lock = proof_lock();
        lock.entries.reverse();
        let artifacts = proof_certificate_artifacts(&lock);

        let report = verify_proof_package(&validated, &lock, &artifacts).unwrap();

        assert_eq!(report.status, PackageVerificationStatus::Passed);
        let order = report
            .topological_order
            .iter()
            .map(Name::as_dotted)
            .collect::<Vec<_>>();
        let std_eq = order
            .iter()
            .position(|module| module == "Std.Logic.Eq")
            .unwrap();
        let local_eq = order
            .iter()
            .position(|module| module == "Proofs.Ai.Eq")
            .unwrap();
        assert!(std_eq < local_eq);
        assert_eq!(
            report
                .modules
                .iter()
                .map(|module| module.module.as_dotted())
                .collect::<Vec<_>>(),
            order
        );
    }

    #[test]
    fn package_fast_verifier_rejects_missing_lock_imports_before_kernel_run() {
        let validated = validated_proof_manifest();
        let mut lock = proof_lock();
        lock.entries
            .retain(|entry| entry.module.as_dotted() != "Std.Logic.Eq");
        let artifacts = proof_certificate_artifacts(&proof_lock());

        let error =
            verify_proof_package(&validated, &lock, &artifacts).expect_err("lock graph is invalid");

        assert_eq!(error.kind, PackageVerificationErrorKind::LockGraph);
        assert_eq!(
            error.reason_code,
            PackageVerificationErrorReason::LockGraphInvalid
        );
    }

    #[test]
    fn package_source_free_invalid_graph_fails_before_artifact_or_checker_lookup() {
        let validated = validated_proof_manifest();
        let mut lock = proof_lock();
        lock.entries
            .retain(|entry| entry.module.as_dotted() != "Std.Logic.Eq");

        let fast = verify_package_fast_source_free(
            &validated,
            &lock,
            Vec::<PackageCertificateArtifact<'_>>::new(),
        )
        .expect_err("invalid lock graph fails before fast verifier artifact lookup");
        let reference = verify_package_reference_source_free(
            &validated,
            &lock,
            Vec::<PackageCertificateArtifact<'_>>::new(),
        )
        .expect_err("invalid lock graph fails before reference checker artifact lookup");

        for error in [fast, reference] {
            assert_eq!(error.kind, PackageVerificationErrorKind::LockGraph);
            assert_eq!(
                error.reason_code,
                PackageVerificationErrorReason::LockGraphInvalid
            );
        }
    }

    #[test]
    fn package_reference_verifier_verifies_proof_package_source_free_in_topological_order() {
        run_on_large_stack(
            "package_reference_verifier_verifies_proof_package_source_free_in_topological_order",
            package_reference_verifier_verifies_proof_package_source_free_in_topological_order_on_large_stack,
        );
    }

    fn package_reference_verifier_verifies_proof_package_source_free_in_topological_order_on_large_stack(
    ) {
        let mut manifest = parse_manifest_str(&proof_manifest_source()).unwrap();
        for module in &mut manifest.modules {
            let module_path = module.module.as_dotted().replace('.', "/");
            module.source = PackagePath::new(format!("missing/source/{module_path}.npa"));
            module.meta = Some(PackagePath::new(format!("missing/meta/{module_path}.json")));
            module.replay = Some(PackagePath::new(format!(
                "missing/replay/{module_path}.json"
            )));
        }
        let validated = validate_manifest(manifest).unwrap();
        let mut lock = proof_lock();
        lock.entries.reverse();
        let artifacts = proof_certificate_artifacts(&lock);

        let report = verify_proof_package_reference(&validated, &lock, &artifacts).unwrap();

        assert_eq!(report.status, PackageVerificationStatus::Passed);
        assert_eq!(report.mode, PackageVerificationMode::Reference);
        assert_eq!(
            report.verdict_source,
            PackageVerificationVerdictSource::ReferenceChecker
        );
        assert!(report.reference_checker_verdict);
        assert_eq!(report.modules.len(), lock.entries.len());
        assert!(report.modules.iter().all(|module| {
            module.checker_mode == PackageVerificationMode::Reference
                && module.status == PackageModuleVerificationStatus::Passed
        }));
        let order = report
            .topological_order
            .iter()
            .map(Name::as_dotted)
            .collect::<Vec<_>>();
        let std_eq = order
            .iter()
            .position(|module| module == "Std.Logic.Eq")
            .unwrap();
        let local_eq = order
            .iter()
            .position(|module| module == "Proofs.Ai.Eq")
            .unwrap();
        assert!(std_eq < local_eq);
        assert_eq!(
            report
                .modules
                .iter()
                .map(|module| module.module.as_dotted())
                .collect::<Vec<_>>(),
            order
        );
    }

    #[test]
    fn package_reference_verifier_rejects_disallowed_axioms_from_certificate() {
        let mut manifest = parse_manifest_str(&proof_manifest_source()).unwrap();
        manifest.policy.allowed_axioms.clear();
        for module in &mut manifest.modules {
            module.axioms = Some(Vec::new());
        }
        let validated = validate_manifest(manifest).unwrap();
        let lock = proof_lock();
        let artifacts = proof_certificate_artifacts(&lock);

        let report = verify_proof_package_reference(&validated, &lock, &artifacts).unwrap();
        let failed = report
            .modules
            .iter()
            .find(|module| module.status == PackageModuleVerificationStatus::Failed)
            .expect("one module fails");

        assert_eq!(report.status, PackageVerificationStatus::Failed);
        assert_eq!(
            failed.error.as_ref().unwrap().kind,
            PackageVerificationErrorKind::ReferenceChecker
        );
        assert_eq!(
            failed.error.as_ref().unwrap().reason_code,
            PackageVerificationErrorReason::AxiomPolicyRejected
        );
        assert_eq!(
            failed
                .error
                .as_ref()
                .unwrap()
                .checker_error
                .as_ref()
                .unwrap()
                .checker,
            "npa-checker-ref"
        );
    }

    #[test]
    fn package_source_free_reference_checker_failure_preserves_structured_payload() {
        let mut manifest = parse_manifest_str(&proof_manifest_source()).unwrap();
        manifest.policy.allowed_axioms.clear();
        for module in &mut manifest.modules {
            module.axioms = Some(Vec::new());
        }
        let validated = validate_manifest(manifest).unwrap();
        let lock = proof_lock();
        let artifacts = proof_certificate_artifacts(&lock);

        let report = verify_proof_package_reference(&validated, &lock, &artifacts).unwrap();
        let failed = report
            .modules
            .iter()
            .find(|module| module.status == PackageModuleVerificationStatus::Failed)
            .expect("reference checker rejects one module");
        let error = failed.error.as_ref().unwrap();
        let checker_error = error
            .checker_error
            .as_ref()
            .expect("reference checker failure carries checker payload");

        assert_eq!(report.status, PackageVerificationStatus::Failed);
        assert_eq!(error.kind, PackageVerificationErrorKind::ReferenceChecker);
        assert_eq!(
            error.reason_code,
            PackageVerificationErrorReason::AxiomPolicyRejected
        );
        assert_eq!(checker_error.checker, "npa-checker-ref");
        assert_eq!(checker_error.kind, "axiom_policy");
        assert_eq!(
            checker_error.reason_code.as_deref(),
            Some("forbidden_axiom")
        );
    }

    #[test]
    fn package_reference_verifier_rejects_unsupported_core_feature() {
        run_on_large_stack(
            "package_reference_verifier_rejects_unsupported_core_feature",
            package_reference_verifier_rejects_unsupported_core_feature_on_large_stack,
        );
    }

    fn package_reference_verifier_rejects_unsupported_core_feature_on_large_stack() {
        let validated = validated_proof_manifest();
        let mut lock = proof_lock();
        let mut artifacts = proof_certificate_artifacts(&lock);
        let target_index = lock
            .entries
            .iter()
            .position(|entry| entry.module.as_dotted() == "Proofs.Ai.Algebra.AbstractGroupQuotient")
            .expect("proof lock contains a quotient_v1 certificate");
        let target_path = lock.entries[target_index].certificate.clone();
        let target_module = lock.entries[target_index].module.clone();
        let bytes = artifacts.get_mut(&target_path).unwrap();
        let needle = b"quotient_v1";
        let replacement = b"unknown_v01";
        let offset = bytes
            .windows(needle.len())
            .position(|window| window == needle)
            .expect("target certificate records quotient_v1");
        bytes[offset..offset + needle.len()].copy_from_slice(replacement);
        lock.entries[target_index].certificate_file_hash = npa_package::package_file_hash(bytes);

        let report = verify_proof_package_reference(&validated, &lock, &artifacts).unwrap();
        let failed = report
            .modules
            .iter()
            .find(|module| module.status == PackageModuleVerificationStatus::Failed)
            .expect("one module fails");
        let error = failed.error.as_ref().unwrap();

        assert_eq!(report.status, PackageVerificationStatus::Failed);
        assert_eq!(failed.module, target_module);
        assert_eq!(error.kind, PackageVerificationErrorKind::ReferenceChecker);
        assert_eq!(
            error.reason_code,
            PackageVerificationErrorReason::UnsupportedCoreFeature
        );
        assert_eq!(
            error.checker_error.as_ref().unwrap().kind,
            "unsupported_core_feature"
        );
        assert_eq!(
            error.checker_error.as_ref().unwrap().reason_code.as_deref(),
            Some("unsupported_core_feature")
        );
    }

    #[test]
    fn package_reference_verifier_rejects_missing_lock_imports_before_checker_run() {
        let validated = validated_proof_manifest();
        let mut lock = proof_lock();
        lock.entries
            .retain(|entry| entry.module.as_dotted() != "Std.Logic.Eq");
        let artifacts = proof_certificate_artifacts(&proof_lock());

        let error = verify_proof_package_reference(&validated, &lock, &artifacts)
            .expect_err("lock graph is invalid");

        assert_eq!(error.kind, PackageVerificationErrorKind::LockGraph);
        assert_eq!(
            error.reason_code,
            PackageVerificationErrorReason::LockGraphInvalid
        );
    }

    #[test]
    fn package_phase8_import_lock_adapter_materializes_direct_imports_only() {
        let lock = proof_lock();
        let materialized = materialize_package_phase8_import_locks(&lock, "reference").unwrap();
        let canonical_entries = canonical_lock_entries(&lock);
        let entries_by_module = canonical_entries
            .iter()
            .map(|(_, entry)| (entry.module.clone(), *entry))
            .collect::<BTreeMap<_, _>>();

        assert_eq!(materialized.len(), lock.entries.len());
        for artifact in &materialized {
            let entry = entries_by_module.get(&artifact.module).unwrap();
            let parsed =
                parse_independent_checker_import_lock_manifest(&artifact.manifest.canonical_json())
                    .unwrap();
            assert_eq!(parsed, artifact.manifest);
            assert_eq!(
                artifact.manifest_hash,
                independent_checker_file_hash(artifact.manifest.canonical_json().as_bytes())
            );
            assert_eq!(
                artifact.path,
                format!(
                    "generated/checker-requests/{}/{}/{}/reference/imports.json",
                    lock.package.as_str(),
                    lock.version.as_str(),
                    artifact.module.as_dotted()
                )
            );
            assert_eq!(artifact.manifest.imports.len(), entry.imports.len());
            assert_eq!(
                artifact
                    .manifest
                    .imports
                    .iter()
                    .map(|import| import.module.clone())
                    .collect::<BTreeSet<_>>(),
                entry
                    .imports
                    .iter()
                    .map(|import| import.module.as_dotted())
                    .collect::<BTreeSet<_>>()
            );
            for import in &artifact.manifest.imports {
                let lock_import = entry
                    .imports
                    .iter()
                    .find(|candidate| candidate.module.as_dotted() == import.module)
                    .unwrap();
                let import_entry = entries_by_module.get(&lock_import.module).unwrap();
                assert_eq!(import.export_hash, lock_import.export_hash.into_bytes());
                assert_eq!(import.certificate.path, import_entry.certificate.as_str());
                assert_eq!(
                    import.certificate.file_hash,
                    import_entry.certificate_file_hash.into_bytes()
                );
                assert_eq!(
                    import.certificate.certificate_hash,
                    lock_import.certificate_hash.into_bytes()
                );
            }

            let json = artifact.manifest.canonical_json();
            for forbidden in [
                "source",
                "replay",
                "meta",
                "theorem_index",
                "ai_trace",
                "registry",
                "solver",
            ] {
                assert!(!json.contains(forbidden), "import lock leaked {forbidden}");
            }
        }
    }

    #[test]
    fn package_phase8_request_materialization_builds_valid_requests_and_hashes() {
        let lock = proof_lock();
        let artifacts = proof_certificate_artifacts(&lock);
        let policy = phase8_reference_runner_policy();

        let report = materialize_package_phase8_requests(
            &lock,
            package_certificate_artifacts(&artifacts),
            &policy,
            "reference",
            None,
        )
        .unwrap();

        let canonical_entries = canonical_lock_entries(&lock);
        let entries_by_module = canonical_entries
            .iter()
            .map(|(_, entry)| (entry.module.clone(), *entry))
            .collect::<BTreeMap<_, _>>();

        assert_eq!(report.modules.len(), lock.entries.len());
        assert_eq!(report.request_store.requests.len(), lock.entries.len());
        assert_eq!(
            parse_independent_checker_request_store_manifest(
                &report.request_store.canonical_json()
            )
            .unwrap(),
            report.request_store
        );
        assert_eq!(
            report.request_store_file_hash,
            independent_checker_file_hash(report.request_store.canonical_json().as_bytes())
        );
        assert!(report.request_store_rewrite_required);

        let second = materialize_package_phase8_requests(
            &lock,
            package_certificate_artifacts(&artifacts),
            &policy,
            "reference",
            Some(&report.request_store),
        )
        .unwrap();
        assert!(!second.request_store_rewrite_required);
        assert_eq!(second.request_store, report.request_store);

        for module in &report.modules {
            let entry = entries_by_module.get(&module.module).unwrap();
            let cert_bytes = artifacts.get(&entry.certificate).unwrap();
            let request_json = module.request.canonical_json();

            assert_eq!(
                parse_independent_checker_machine_check_request(&request_json).unwrap(),
                module.request
            );
            assert_eq!(
                independent_checker_machine_check_request_hash(&request_json).unwrap(),
                module.request.request_hash()
            );
            assert_eq!(
                module.request_file_hash,
                independent_checker_file_hash(request_json.as_bytes())
            );
            assert_eq!(
                module.request.request_id,
                format!(
                    "package:{}:{}:{}:reference",
                    lock.package.as_str(),
                    lock.version.as_str(),
                    module.module.as_dotted()
                )
            );
            assert_eq!(
                module.request_path,
                format!(
                    "generated/checker-requests/{}/{}/{}/reference/request.json",
                    lock.package.as_str(),
                    lock.version.as_str(),
                    module.module.as_dotted()
                )
            );
            assert_eq!(module.request.module, module.module.as_dotted());
            assert_eq!(module.request.checker_profile, "reference");
            assert_eq!(module.request.certificate.path, entry.certificate.as_str());
            assert_eq!(
                module.request.certificate.file_hash,
                independent_checker_file_hash(cert_bytes)
            );
            assert_eq!(
                module.request.certificate.expected_certificate_hash,
                entry.certificate_hash.into_bytes()
            );
            assert_eq!(module.request.imports.manifest, module.import_lock_path);
            assert_eq!(
                module.request.imports.manifest_hash,
                module.import_lock_manifest_hash
            );
            assert_eq!(
                parse_independent_checker_import_lock_manifest(
                    &module.import_lock_manifest.canonical_json()
                )
                .unwrap(),
                module.import_lock_manifest
            );

            for forbidden in [
                "source",
                "replay",
                "meta",
                "theorem_index",
                "ai_trace",
                "registry",
                "solver",
            ] {
                assert!(
                    !request_json.contains(forbidden),
                    "request leaked {forbidden}"
                );
            }
        }
    }
}
