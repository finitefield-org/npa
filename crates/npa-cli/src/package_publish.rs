//! Publish-plan input collection for CLR-06.
//!
//! This module loads and validates the source-free inputs that later CLR-06
//! milestones use to build `generated/publish-plan.json`. It also projects the
//! deterministic release artifact list, module registry seed entries,
//! downstream import bundle, and checksum-only signature policy. It
//! intentionally does not write the publish-plan file yet.

use std::path::Path;

use npa_api::{
    project_package_axiom_report_from_extraction, project_package_theorem_index_from_extraction,
    PackageArtifactExtraction, PackageArtifactReferenceSummaryMode, PackageVerificationReport,
    PackageVerificationStatus, PackageVerificationVerdictSource,
};
use npa_package::{
    build_package_downstream_import_bundle, build_package_publish_artifacts,
    build_package_registry_modules, format_package_hash, package_checksum_only_signature_policy,
    package_file_hash, parse_package_axiom_report_json, parse_package_theorem_index_json,
    PackageArtifactError, PackageArtifactErrorReason, PackageArtifactFileReference,
    PackageAxiomReport, PackageCheckerMode, PackageCheckerSummary, PackageDownstreamImportBundle,
    PackageDownstreamImportBundleInput, PackageHash, PackageLockManifest, PackagePath,
    PackagePublishArtifact, PackagePublishArtifactListInput, PackageRegistryArtifactHashes,
    PackageRegistryModule, PackageRegistryModuleSeedInput, PackageSignaturePolicy,
    PackageTheoremIndex, ValidatedPackageManifest,
};

use crate::diagnostic::{CommandDiagnostic, CommandResult, DiagnosticKind};
use crate::package_artifacts::{
    load_package_artifact_extraction, LoadedPackageArtifactExtraction,
    PackageGeneratedArtifactReadMode, PACKAGE_AXIOM_REPORT_PATH, PACKAGE_LOCK_PATH,
    PACKAGE_THEOREM_INDEX_PATH,
};

/// Stable command name reserved for the later `npa package publish-plan` command.
pub const COMMAND: &str = "package publish-plan";

/// Source-free publish inputs loaded and freshness-checked for CLR-06.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadedPackagePublishInputs {
    /// Sanitized package root display string for diagnostics.
    pub root_display: String,
    /// Validated package manifest.
    pub validated: ValidatedPackageManifest,
    /// Exact manifest file identity used by package metadata.
    pub manifest: PackageArtifactFileReference,
    /// Parsed package-lock manifest.
    pub package_lock_manifest: PackageLockManifest,
    /// Exact package-lock file identity.
    pub package_lock: PackageArtifactFileReference,
    /// Certificate file identities copied from the package lock.
    pub certificate_files: Vec<PackageArtifactFileReference>,
    /// Parsed checked axiom report.
    pub axiom_report: PackageAxiomReport,
    /// Exact checked axiom-report file identity.
    pub axiom_report_file: PackageArtifactFileReference,
    /// Parsed checked theorem index.
    pub theorem_index: PackageTheoremIndex,
    /// Exact checked theorem-index file identity.
    pub theorem_index_file: PackageArtifactFileReference,
    /// Fast source-free extraction used to refresh checked CLR-05 artifacts.
    pub artifact_extraction: PackageArtifactExtraction,
    /// Fast and reference source-free checker summaries for publish metadata.
    pub checker_summaries: Vec<PackageCheckerSummary>,
    /// Source-free reference checker report used to validate release metadata.
    pub reference_verification_report: PackageVerificationReport,
}

/// Load and freshness-check the CLR-06 publish inputs.
///
/// The collector reads the package manifest, package lock, checked axiom
/// report, checked theorem index, and certificate bytes required by the package
/// lock/manifest graph. It does not read source, replay, meta, AI, registry,
/// network, Git-host, theorem graph, or existing publish-plan files.
pub fn load_package_publish_inputs(
    root: impl AsRef<Path>,
) -> Result<LoadedPackagePublishInputs, CommandResult> {
    let loaded = load_package_artifact_extraction(
        root.as_ref(),
        COMMAND,
        PackageGeneratedArtifactReadMode::all(),
        PackageArtifactReferenceSummaryMode::Omit,
    )?;
    ensure_checked_package_lock_canonical(&loaded)?;

    let (axiom_report, axiom_report_json) = parse_checked_axiom_report(&loaded)?;
    ensure_axiom_report_current(&loaded, &axiom_report_json)?;
    let (theorem_index, theorem_index_json) = parse_checked_theorem_index(&loaded)?;
    ensure_theorem_index_current(&loaded, &theorem_index_json)?;

    let reference_loaded = load_package_artifact_extraction(
        root,
        COMMAND,
        PackageGeneratedArtifactReadMode::none(),
        PackageArtifactReferenceSummaryMode::Include,
    )?;
    let reference_verification_report = require_reference_checker_report(&reference_loaded)?;
    validate_publish_checker_summaries(
        &loaded.package_lock_manifest,
        &loaded.validated.manifest().checker_profile,
        &reference_loaded.extraction.checker_summaries,
    )
    .map_err(|diagnostic| {
        CommandResult::failed(
            COMMAND,
            reference_loaded.root_display.clone(),
            vec![*diagnostic],
        )
    })?;

    Ok(LoadedPackagePublishInputs {
        root_display: loaded.root_display,
        validated: loaded.validated,
        manifest: PackageArtifactFileReference {
            path: loaded.extraction.manifest.path.clone(),
            file_hash: loaded.extraction.manifest.file_hash,
        },
        certificate_files: certificate_file_references(&loaded.package_lock_manifest),
        package_lock_manifest: loaded.package_lock_manifest,
        package_lock: loaded.package_lock,
        axiom_report_file: PackageArtifactFileReference {
            path: PackagePath::new(PACKAGE_AXIOM_REPORT_PATH),
            file_hash: package_file_hash(axiom_report_json.as_bytes()),
        },
        axiom_report,
        theorem_index_file: PackageArtifactFileReference {
            path: PackagePath::new(PACKAGE_THEOREM_INDEX_PATH),
            file_hash: package_file_hash(theorem_index_json.as_bytes()),
        },
        theorem_index,
        artifact_extraction: loaded.extraction,
        checker_summaries: reference_loaded.extraction.checker_summaries,
        reference_verification_report,
    })
}

/// Build the deterministic release artifact list from loaded publish inputs.
pub fn collect_package_publish_artifacts(
    inputs: &LoadedPackagePublishInputs,
) -> Result<Vec<PackagePublishArtifact>, CommandResult> {
    build_package_publish_artifacts(PackagePublishArtifactListInput {
        manifest: inputs.manifest.clone(),
        package_lock: inputs.package_lock.clone(),
        axiom_report: inputs.axiom_report_file.clone(),
        theorem_index: inputs.theorem_index_file.clone(),
        package_lock_manifest: &inputs.package_lock_manifest,
    })
    .map_err(|error| {
        CommandResult::failed(
            COMMAND,
            inputs.root_display.clone(),
            vec![publish_artifact_error_diagnostic(error)],
        )
    })
}

/// Build deterministic module registry seed entries from loaded publish inputs.
pub fn collect_package_publish_registry_entries(
    inputs: &LoadedPackagePublishInputs,
) -> Result<Vec<PackageRegistryModule>, CommandResult> {
    build_package_registry_modules(PackageRegistryModuleSeedInput {
        manifest: inputs.validated.manifest(),
        package_lock: &inputs.package_lock_manifest,
        checker_summaries: &inputs.checker_summaries,
        artifact_hashes: PackageRegistryArtifactHashes {
            package_lock_file_hash: inputs.package_lock.file_hash,
            axiom_report_file_hash: inputs.axiom_report_file.file_hash,
            theorem_index_file_hash: inputs.theorem_index_file.file_hash,
        },
    })
    .map_err(|error| {
        CommandResult::failed(
            COMMAND,
            inputs.root_display.clone(),
            vec![publish_registry_error_diagnostic(error)],
        )
    })
}

/// Build the embedded downstream import bundle from local registry seed entries.
pub fn collect_package_publish_downstream_import_bundle(
    inputs: &LoadedPackagePublishInputs,
) -> Result<PackageDownstreamImportBundle, CommandResult> {
    let module_registry_entries = collect_package_publish_registry_entries(inputs)?;
    build_package_downstream_import_bundle(PackageDownstreamImportBundleInput {
        package: &inputs.validated.manifest().package,
        version: &inputs.validated.manifest().version,
        module_registry_entries: &module_registry_entries,
    })
    .map_err(|error| {
        CommandResult::failed(
            COMMAND,
            inputs.root_display.clone(),
            vec![publish_downstream_import_bundle_error_diagnostic(error)],
        )
    })
}

/// Return the explicit CLR-06 checksum-only signature policy.
pub fn checksum_only_signature_policy() -> PackageSignaturePolicy {
    package_checksum_only_signature_policy()
}

/// Validate publish-plan checker summaries against the package lock.
///
/// CLR-06 requires a source-free reference-checker summary for every lock entry.
/// Fast summaries may be present, but they must not be labeled as
/// `npa-checker-ref` verdicts.
pub fn validate_publish_checker_summaries(
    lock: &PackageLockManifest,
    checker_profile: &str,
    summaries: &[PackageCheckerSummary],
) -> Result<(), Box<CommandDiagnostic>> {
    for summary in summaries {
        let module = summary.module.as_dotted();
        if summary.mode == PackageCheckerMode::Fast && summary.checker == "npa-checker-ref" {
            return Err(checker_summary_stale(
                &module,
                "mode",
                "fast summary must not use npa-checker-ref checker identity",
                summary.mode.as_str(),
            ));
        }
        if summary.mode == PackageCheckerMode::Reference {
            if summary.checker != "npa-checker-ref" {
                return Err(checker_summary_stale(
                    &module,
                    "checker",
                    "npa-checker-ref",
                    &summary.checker,
                ));
            }
            if summary.profile != checker_profile {
                return Err(checker_summary_stale(
                    &module,
                    "profile",
                    checker_profile,
                    &summary.profile,
                ));
            }
            let Some(entry) = lock
                .entries
                .iter()
                .find(|entry| entry.module == summary.module)
            else {
                return Err(checker_summary_stale(
                    &module,
                    "module",
                    "package lock entry",
                    &module,
                ));
            };
            if summary.status != "passed" {
                return Err(checker_summary_stale(
                    &module,
                    "status",
                    "passed",
                    &summary.status,
                ));
            }
            ensure_summary_hash(
                &module,
                "export_hash",
                entry.export_hash,
                summary.export_hash,
            )?;
            ensure_summary_hash(
                &module,
                "certificate_hash",
                entry.certificate_hash,
                summary.certificate_hash,
            )?;
            ensure_summary_hash(
                &module,
                "axiom_report_hash",
                entry.axiom_report_hash,
                summary.axiom_report_hash,
            )?;
        }
    }

    for entry in &lock.entries {
        let module = entry.module.as_dotted();
        if summaries.iter().any(|summary| {
            summary.module == entry.module && summary.mode == PackageCheckerMode::Reference
        }) {
            continue;
        }
        return Err(Box::new(
            CommandDiagnostic::error(DiagnosticKind::ReferenceVerifier, "checker_summary_missing")
                .with_path("checker_summaries")
                .with_module(module),
        ));
    }

    Ok(())
}

fn ensure_checked_package_lock_canonical(
    loaded: &LoadedPackageArtifactExtraction,
) -> Result<(), CommandResult> {
    let canonical = loaded
        .package_lock_manifest
        .canonical_json()
        .map_err(|error| {
            CommandResult::failed(
                COMMAND,
                loaded.root_display.clone(),
                vec![
                    CommandDiagnostic::from_package_lock_error(&error).with_path(PACKAGE_LOCK_PATH)
                ],
            )
        })?;
    if loaded.package_lock_json == canonical {
        Ok(())
    } else {
        Err(CommandResult::failed(
            COMMAND,
            loaded.root_display.clone(),
            vec![
                CommandDiagnostic::error(DiagnosticKind::HashMismatch, "package_lock_stale")
                    .with_path(PACKAGE_LOCK_PATH)
                    .with_hashes(
                        format_package_hash(&package_file_hash(canonical.as_bytes())),
                        format_package_hash(&package_file_hash(
                            loaded.package_lock_json.as_bytes(),
                        )),
                    ),
            ],
        ))
    }
}

fn parse_checked_axiom_report(
    loaded: &LoadedPackageArtifactExtraction,
) -> Result<(PackageAxiomReport, String), CommandResult> {
    let json = loaded
        .checked_generated
        .axiom_report_json
        .clone()
        .expect("publish input collection requests axiom-report JSON");
    let report = parse_package_axiom_report_json(&json).map_err(|error| {
        CommandResult::failed(
            COMMAND,
            loaded.root_display.clone(),
            vec![artifact_error_diagnostic(
                &error,
                DiagnosticKind::AxiomReport,
                PACKAGE_AXIOM_REPORT_PATH,
                "axiom_report_non_canonical_order",
                "axiom_report_hash_mismatch",
            )],
        )
    })?;
    Ok((report, json))
}

fn parse_checked_theorem_index(
    loaded: &LoadedPackageArtifactExtraction,
) -> Result<(PackageTheoremIndex, String), CommandResult> {
    let json = loaded
        .checked_generated
        .theorem_index_json
        .clone()
        .expect("publish input collection requests theorem-index JSON");
    let index = parse_package_theorem_index_json(&json).map_err(|error| {
        CommandResult::failed(
            COMMAND,
            loaded.root_display.clone(),
            vec![artifact_error_diagnostic(
                &error,
                DiagnosticKind::TheoremIndex,
                PACKAGE_THEOREM_INDEX_PATH,
                "theorem_index_non_canonical_order",
                "theorem_index_hash_mismatch",
            )],
        )
    })?;
    Ok((index, json))
}

fn ensure_axiom_report_current(
    loaded: &LoadedPackageArtifactExtraction,
    checked_json: &str,
) -> Result<(), CommandResult> {
    let generated = project_package_axiom_report_from_extraction(
        &loaded.validated,
        &loaded.extraction,
        loaded.package_lock.clone(),
    )
    .and_then(|report| report.canonical_json())
    .map_err(|error| {
        CommandResult::failed(
            COMMAND,
            loaded.root_display.clone(),
            vec![metadata_extraction_diagnostic(
                DiagnosticKind::AxiomReport,
                PACKAGE_AXIOM_REPORT_PATH,
                error,
            )],
        )
    })?;
    ensure_generated_current(
        loaded,
        DiagnosticKind::AxiomReport,
        PACKAGE_AXIOM_REPORT_PATH,
        "axiom_report_stale",
        checked_json,
        &generated,
    )
}

fn ensure_theorem_index_current(
    loaded: &LoadedPackageArtifactExtraction,
    checked_json: &str,
) -> Result<(), CommandResult> {
    let generated = project_package_theorem_index_from_extraction(
        &loaded.validated,
        &loaded.extraction,
        loaded.package_lock.clone(),
    )
    .and_then(|index| index.canonical_json())
    .map_err(|error| {
        CommandResult::failed(
            COMMAND,
            loaded.root_display.clone(),
            vec![metadata_extraction_diagnostic(
                DiagnosticKind::TheoremIndex,
                PACKAGE_THEOREM_INDEX_PATH,
                error,
            )],
        )
    })?;
    ensure_generated_current(
        loaded,
        DiagnosticKind::TheoremIndex,
        PACKAGE_THEOREM_INDEX_PATH,
        "theorem_index_stale",
        checked_json,
        &generated,
    )
}

fn ensure_generated_current(
    loaded: &LoadedPackageArtifactExtraction,
    kind: DiagnosticKind,
    path: &'static str,
    reason_code: &'static str,
    checked_json: &str,
    generated_json: &str,
) -> Result<(), CommandResult> {
    if checked_json == generated_json {
        Ok(())
    } else {
        Err(CommandResult::failed(
            COMMAND,
            loaded.root_display.clone(),
            vec![CommandDiagnostic::error(kind, reason_code)
                .with_path(path)
                .with_hashes(
                    format_package_hash(&package_file_hash(generated_json.as_bytes())),
                    format_package_hash(&package_file_hash(checked_json.as_bytes())),
                )],
        ))
    }
}

fn require_reference_checker_report(
    loaded: &LoadedPackageArtifactExtraction,
) -> Result<PackageVerificationReport, CommandResult> {
    let Some(report) = loaded.extraction.reference_verification_report.clone() else {
        return Err(CommandResult::failed(
            COMMAND,
            loaded.root_display.clone(),
            vec![CommandDiagnostic::error(
                DiagnosticKind::ReferenceVerifier,
                "checker_summary_missing",
            )
            .with_path("checker_summaries")
            .with_checker("npa-checker-ref")],
        ));
    };
    if report.mode.as_str() != "reference"
        || report.verdict_source != PackageVerificationVerdictSource::ReferenceChecker
        || !report.reference_checker_verdict
        || report.status != PackageVerificationStatus::Passed
    {
        return Err(CommandResult::failed(
            COMMAND,
            loaded.root_display.clone(),
            vec![CommandDiagnostic::error(
                DiagnosticKind::ReferenceVerifier,
                "checker_summary_stale",
            )
            .with_path("checker_summaries")
            .with_checker("npa-checker-ref")
            .with_expected_value("passed source-free reference checker report")
            .with_actual_value(report.status.as_str())],
        ));
    }
    Ok(report)
}

fn ensure_summary_hash(
    module: &str,
    field: &'static str,
    expected: PackageHash,
    actual: PackageHash,
) -> Result<(), Box<CommandDiagnostic>> {
    if expected == actual {
        Ok(())
    } else {
        Err(Box::new(
            CommandDiagnostic::error(DiagnosticKind::ReferenceVerifier, "checker_summary_stale")
                .with_path("checker_summaries")
                .with_module(module)
                .with_field(field)
                .with_hashes(format_package_hash(&expected), format_package_hash(&actual)),
        ))
    }
}

fn checker_summary_stale(
    module: &str,
    field: &'static str,
    expected: impl Into<String>,
    actual: impl Into<String>,
) -> Box<CommandDiagnostic> {
    Box::new(
        CommandDiagnostic::error(DiagnosticKind::ReferenceVerifier, "checker_summary_stale")
            .with_path("checker_summaries")
            .with_module(module)
            .with_field(field)
            .with_expected_value(expected)
            .with_actual_value(actual),
    )
}

fn artifact_error_diagnostic(
    error: &PackageArtifactError,
    kind: DiagnosticKind,
    artifact_path: &'static str,
    noncanonical_reason: &'static str,
    self_hash_reason: &'static str,
) -> CommandDiagnostic {
    let reason_code = match error.reason_code {
        PackageArtifactErrorReason::NonCanonicalOrder => noncanonical_reason,
        PackageArtifactErrorReason::SelfHashMismatch => self_hash_reason,
        _ => error.reason_code.as_str(),
    };
    let mut diagnostic = CommandDiagnostic::error(kind, reason_code).with_path(artifact_path);
    if let Some(field) = error.field.clone().or_else(|| {
        if error.path == "$" {
            None
        } else {
            Some(error.path.clone())
        }
    }) {
        diagnostic = diagnostic.with_field(field);
    }
    if error.reason_code == PackageArtifactErrorReason::SelfHashMismatch {
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

fn metadata_extraction_diagnostic(
    kind: DiagnosticKind,
    artifact_path: &'static str,
    error: PackageArtifactError,
) -> CommandDiagnostic {
    let message = error.to_string();
    CommandDiagnostic::error(kind, "metadata_extraction_failed")
        .with_path(artifact_path)
        .with_field(error.path)
        .with_actual_value(message)
}

fn publish_artifact_error_diagnostic(error: PackageArtifactError) -> CommandDiagnostic {
    publish_metadata_error_diagnostic(error, "artifacts")
}

fn publish_registry_error_diagnostic(error: PackageArtifactError) -> CommandDiagnostic {
    publish_metadata_error_diagnostic(error, "module_registry_entries")
}

fn publish_downstream_import_bundle_error_diagnostic(
    error: PackageArtifactError,
) -> CommandDiagnostic {
    publish_metadata_error_diagnostic(error, "downstream_import_bundle")
}

fn publish_metadata_error_diagnostic(
    error: PackageArtifactError,
    artifact_path: &'static str,
) -> CommandDiagnostic {
    let mut diagnostic = CommandDiagnostic::error(
        DiagnosticKind::GeneratedArtifact,
        error.reason_code.as_str(),
    )
    .with_path(artifact_path);
    if let Some(field) = error.field.clone().or_else(|| {
        if error.path == "$" {
            None
        } else {
            Some(error.path.clone())
        }
    }) {
        diagnostic = diagnostic.with_field(field);
    }
    if let (Some(expected), Some(actual)) = (error.expected_value, error.actual_value) {
        diagnostic = diagnostic
            .with_expected_value(expected)
            .with_actual_value(actual);
    }
    diagnostic
}

fn certificate_file_references(lock: &PackageLockManifest) -> Vec<PackageArtifactFileReference> {
    lock.entries
        .iter()
        .map(|entry| PackageArtifactFileReference {
            path: entry.certificate.clone(),
            file_hash: entry.certificate_file_hash,
        })
        .collect()
}
