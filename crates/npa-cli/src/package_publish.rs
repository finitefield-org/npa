//! Publish-plan input collection for CLR-06.
//!
//! This module loads and validates the source-free inputs that later CLR-06
//! milestones use to build `generated/publish-plan.json`. It also implements
//! the check/write `package publish-plan` command over that source-free
//! metadata.

use std::{fs, io, path::Path};

use npa_api::{
    project_package_axiom_report_from_extraction, project_package_theorem_index_from_extraction,
    PackageArtifactExtraction, PackageArtifactReferenceSummaryMode, PackageVerificationReport,
    PackageVerificationStatus, PackageVerificationVerdictSource,
};
use npa_package::{
    build_package_downstream_import_bundle, build_package_publish_artifacts,
    build_package_registry_modules, format_package_hash, package_checksum_only_signature_policy,
    package_file_hash, parse_package_axiom_report_json, parse_package_publish_plan_json,
    parse_package_theorem_index_json, PackageArtifactError, PackageArtifactErrorReason,
    PackageArtifactFileReference, PackageAxiomReport, PackageCheckerMode, PackageCheckerSummary,
    PackageDownstreamImportBundle, PackageDownstreamImportBundleInput, PackageHash,
    PackageLockManifest, PackagePath, PackagePublishArtifact, PackagePublishArtifactListInput,
    PackagePublishArtifactRole, PackagePublishPlan, PackagePublishRelease,
    PackagePublishReleaseReference, PackagePublishSummary, PackageRegistryArtifactHashes,
    PackageRegistryModule, PackageRegistryModuleSeedInput, PackageSignaturePolicy,
    PackageTheoremIndex, ValidatedPackageManifest, PACKAGE_AXIOM_REPORT_SCHEMA,
    PACKAGE_LOCK_SCHEMA, PACKAGE_MANIFEST_SCHEMA, PACKAGE_PUBLISH_PLAN_PATH,
    PACKAGE_PUBLISH_PLAN_SCHEMA, PACKAGE_THEOREM_INDEX_SCHEMA,
};

use crate::args::{PackageCommonOptions, PackagePublishPlanOptions};
use crate::diagnostic::{CommandDiagnostic, CommandResult, DiagnosticKind};
use crate::fs::join_package_path;
use crate::package_artifacts::{
    load_package_artifact_extraction, load_package_artifact_extraction_with_timings,
    LoadedPackageArtifactExtraction, PackageGeneratedArtifactReadMode, PACKAGE_AXIOM_REPORT_PATH,
    PACKAGE_LOCK_PATH, PACKAGE_THEOREM_INDEX_PATH,
};
use crate::timing::{
    PackageTimingCollector, TIMING_ARTIFACT_COMPARE_MS, TIMING_JSON_WRITE_MS, TIMING_PROJECTION_MS,
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

/// Run `package publish-plan`.
pub fn run_package_publish_plan(options: PackagePublishPlanOptions) -> CommandResult {
    let mut timings = PackageTimingCollector::new(options.timings);
    let result = if options.check {
        run_package_publish_plan_check(options.common, &mut timings)
    } else {
        run_package_publish_plan_write(options.common, &mut timings)
    };
    timings.finish_result(result)
}

fn run_package_publish_plan_check(
    options: PackageCommonOptions,
    timings: &mut PackageTimingCollector,
) -> CommandResult {
    let (inputs, generated_plan, generated_json) =
        match generate_package_publish_plan(&options, timings) {
            Ok(generated) => generated,
            Err(result) => return result,
        };
    let checked_json = match timings.time_phase(TIMING_ARTIFACT_COMPARE_MS, || {
        read_checked_publish_plan(&options)
    }) {
        Ok(json) => json,
        Err(diagnostic) => {
            return CommandResult::failed(COMMAND, inputs.root_display, vec![*diagnostic]);
        }
    };
    let checked_plan = match timings.time_phase(TIMING_ARTIFACT_COMPARE_MS, || {
        parse_package_publish_plan_json(&checked_json)
    }) {
        Ok(plan) => plan,
        Err(error) => {
            return CommandResult::failed(
                COMMAND,
                inputs.root_display,
                vec![publish_plan_error_diagnostic(&error)],
            );
        }
    };

    let registry_stale = timings.time_phase(TIMING_ARTIFACT_COMPARE_MS, || {
        checked_plan.module_registry_entries != generated_plan.module_registry_entries
    });
    if registry_stale {
        return CommandResult::failed(
            COMMAND,
            inputs.root_display,
            vec![publish_plan_stale_diagnostic(
                "registry_entry_mismatch",
                Some("module_registry_entries"),
                &checked_json,
                &generated_json,
            )],
        );
    }
    let downstream_stale = timings.time_phase(TIMING_ARTIFACT_COMPARE_MS, || {
        checked_plan.downstream_import_bundle != generated_plan.downstream_import_bundle
    });
    if downstream_stale {
        return CommandResult::failed(
            COMMAND,
            inputs.root_display,
            vec![publish_plan_stale_diagnostic(
                "downstream_import_bundle_mismatch",
                Some("downstream_import_bundle"),
                &checked_json,
                &generated_json,
            )],
        );
    }
    let plan_stale = timings.time_phase(TIMING_ARTIFACT_COMPARE_MS, || {
        checked_json != generated_json
    });
    if plan_stale {
        return CommandResult::failed(
            COMMAND,
            inputs.root_display,
            vec![publish_plan_stale_diagnostic(
                "publish_plan_stale",
                None,
                &checked_json,
                &generated_json,
            )],
        );
    }

    passed_result(inputs.root_display)
}

fn run_package_publish_plan_write(
    options: PackageCommonOptions,
    timings: &mut PackageTimingCollector,
) -> CommandResult {
    let (inputs, _plan, generated_json) = match generate_package_publish_plan(&options, timings) {
        Ok(generated) => generated,
        Err(result) => return result,
    };
    if let Err(error) = timings.time_phase(TIMING_ARTIFACT_COMPARE_MS, || {
        parse_package_publish_plan_json(&generated_json)
    }) {
        return CommandResult::failed(
            COMMAND,
            inputs.root_display,
            vec![publish_plan_error_diagnostic(&error)],
        );
    }
    let write_result = timings.time_phase(TIMING_JSON_WRITE_MS, || {
        write_publish_plan(&options, generated_json.as_bytes())
    });
    if let Err(diagnostic) = write_result {
        return CommandResult::failed(COMMAND, inputs.root_display, vec![*diagnostic]);
    }

    passed_result(inputs.root_display)
}

fn generate_package_publish_plan(
    options: &PackageCommonOptions,
    timings: &mut PackageTimingCollector,
) -> Result<(LoadedPackagePublishInputs, PackagePublishPlan, String), CommandResult> {
    let inputs = load_package_publish_inputs_with_timings(&options.root, timings)?;
    let artifacts = timings.time_phase(TIMING_PROJECTION_MS, || {
        collect_package_publish_artifacts(&inputs)
    })?;
    let module_registry_entries = timings.time_phase(TIMING_PROJECTION_MS, || {
        collect_package_publish_registry_entries(&inputs)
    })?;
    let downstream_import_bundle = timings
        .time_phase(TIMING_PROJECTION_MS, || {
            build_package_downstream_import_bundle(PackageDownstreamImportBundleInput {
                package: &inputs.validated.manifest().package,
                version: &inputs.validated.manifest().version,
                module_registry_entries: &module_registry_entries,
                theorem_index: &inputs.theorem_index,
                checker_summaries: &inputs.checker_summaries,
            })
        })
        .map_err(|error| {
            CommandResult::failed(
                COMMAND,
                inputs.root_display.clone(),
                vec![publish_downstream_import_bundle_error_diagnostic(error)],
            )
        })?;
    let plan = timings
        .time_phase(TIMING_PROJECTION_MS, || {
            PackagePublishPlan {
                schema: PACKAGE_PUBLISH_PLAN_SCHEMA.to_owned(),
                package: inputs.validated.manifest().package.clone(),
                version: inputs.validated.manifest().version.clone(),
                release: publish_release(&inputs),
                summary: publish_summary(
                    &artifacts,
                    &module_registry_entries,
                    &inputs.checker_summaries,
                ),
                artifacts,
                module_registry_entries,
                downstream_import_bundle,
                checker_summaries: inputs.checker_summaries.clone(),
                signature_policy: package_checksum_only_signature_policy(),
                publish_plan_hash: package_file_hash(b""),
            }
            .with_computed_hash()
        })
        .map_err(|error| {
            CommandResult::failed(
                COMMAND,
                inputs.root_display.clone(),
                vec![publish_plan_error_diagnostic(&error)],
            )
        })?;
    let plan_json = timings
        .time_phase(TIMING_JSON_WRITE_MS, || plan.canonical_json())
        .map_err(|error| {
            CommandResult::failed(
                COMMAND,
                inputs.root_display.clone(),
                vec![publish_plan_error_diagnostic(&error)],
            )
        })?;
    Ok((inputs, plan, plan_json))
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
    load_package_publish_inputs_impl(root.as_ref(), None)
}

fn load_package_publish_inputs_with_timings(
    root: impl AsRef<Path>,
    timings: &mut PackageTimingCollector,
) -> Result<LoadedPackagePublishInputs, CommandResult> {
    load_package_publish_inputs_impl(root.as_ref(), Some(timings))
}

fn load_package_publish_inputs_impl(
    root: &Path,
    mut timings: Option<&mut PackageTimingCollector>,
) -> Result<LoadedPackagePublishInputs, CommandResult> {
    let loaded = match timings.as_mut() {
        Some(timings) => load_package_artifact_extraction_with_timings(
            root,
            COMMAND,
            PackageGeneratedArtifactReadMode::all(),
            PackageArtifactReferenceSummaryMode::Omit,
            timings,
        ),
        None => load_package_artifact_extraction(
            root,
            COMMAND,
            PackageGeneratedArtifactReadMode::all(),
            PackageArtifactReferenceSummaryMode::Omit,
        ),
    }?;
    match timings.as_mut() {
        Some(timings) => timings.time_phase(TIMING_ARTIFACT_COMPARE_MS, || {
            ensure_checked_package_lock_canonical(&loaded)
        }),
        None => ensure_checked_package_lock_canonical(&loaded),
    }?;

    let (axiom_report, axiom_report_json) = match timings.as_mut() {
        Some(timings) => timings.time_phase(TIMING_ARTIFACT_COMPARE_MS, || {
            parse_checked_axiom_report(&loaded)
        }),
        None => parse_checked_axiom_report(&loaded),
    }?;
    match timings.as_mut() {
        Some(timings) => timings.time_phase(TIMING_ARTIFACT_COMPARE_MS, || {
            ensure_axiom_report_current(&loaded, &axiom_report_json)
        }),
        None => ensure_axiom_report_current(&loaded, &axiom_report_json),
    }?;
    let (theorem_index, theorem_index_json) = match timings.as_mut() {
        Some(timings) => timings.time_phase(TIMING_ARTIFACT_COMPARE_MS, || {
            parse_checked_theorem_index(&loaded)
        }),
        None => parse_checked_theorem_index(&loaded),
    }?;
    match timings.as_mut() {
        Some(timings) => timings.time_phase(TIMING_ARTIFACT_COMPARE_MS, || {
            ensure_theorem_index_current(&loaded, &theorem_index_json)
        }),
        None => ensure_theorem_index_current(&loaded, &theorem_index_json),
    }?;

    let reference_loaded = match timings.as_mut() {
        Some(timings) => load_package_artifact_extraction_with_timings(
            root,
            COMMAND,
            PackageGeneratedArtifactReadMode::none(),
            PackageArtifactReferenceSummaryMode::Include,
            timings,
        ),
        None => load_package_artifact_extraction(
            root,
            COMMAND,
            PackageGeneratedArtifactReadMode::none(),
            PackageArtifactReferenceSummaryMode::Include,
        ),
    }?;
    let reference_verification_report = match timings.as_mut() {
        Some(timings) => timings.time_phase(TIMING_ARTIFACT_COMPARE_MS, || {
            require_reference_checker_report(&reference_loaded)
        }),
        None => require_reference_checker_report(&reference_loaded),
    }?;
    match timings.as_mut() {
        Some(timings) => timings.time_phase(TIMING_ARTIFACT_COMPARE_MS, || {
            validate_publish_checker_summaries(
                &loaded.package_lock_manifest,
                &loaded.validated.manifest().checker_profile,
                &reference_loaded.extraction.checker_summaries,
            )
        }),
        None => validate_publish_checker_summaries(
            &loaded.package_lock_manifest,
            &loaded.validated.manifest().checker_profile,
            &reference_loaded.extraction.checker_summaries,
        ),
    }
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
        theorem_index: &inputs.theorem_index,
        checker_summaries: &inputs.checker_summaries,
    })
    .map_err(|error| {
        CommandResult::failed(
            COMMAND,
            inputs.root_display.clone(),
            vec![publish_downstream_import_bundle_error_diagnostic(error)],
        )
    })
}

fn publish_release(inputs: &LoadedPackagePublishInputs) -> PackagePublishRelease {
    let manifest = inputs.validated.manifest();
    PackagePublishRelease {
        core_spec: manifest.core_spec.clone(),
        kernel_profile: manifest.kernel_profile.clone(),
        certificate_format: manifest.certificate_format.clone(),
        checker_profile: manifest.checker_profile.clone(),
        manifest: release_reference(inputs.manifest.clone(), None, PACKAGE_MANIFEST_SCHEMA),
        package_lock: release_reference(inputs.package_lock.clone(), None, PACKAGE_LOCK_SCHEMA),
        axiom_report: release_reference(
            inputs.axiom_report_file.clone(),
            Some(inputs.axiom_report.package_axiom_report_hash),
            PACKAGE_AXIOM_REPORT_SCHEMA,
        ),
        theorem_index: release_reference(
            inputs.theorem_index_file.clone(),
            Some(inputs.theorem_index.theorem_index_hash),
            PACKAGE_THEOREM_INDEX_SCHEMA,
        ),
    }
}

fn release_reference(
    reference: PackageArtifactFileReference,
    content_hash: Option<PackageHash>,
    schema: &'static str,
) -> PackagePublishReleaseReference {
    PackagePublishReleaseReference {
        path: reference.path,
        file_hash: reference.file_hash,
        content_hash,
        schema: Some(schema.to_owned()),
    }
}

fn publish_summary(
    artifacts: &[PackagePublishArtifact],
    registry_entries: &[PackageRegistryModule],
    checker_summaries: &[PackageCheckerSummary],
) -> PackagePublishSummary {
    PackagePublishSummary {
        local_module_count: u64::try_from(registry_entries.len()).unwrap(),
        external_import_count: u64::try_from(
            artifacts
                .iter()
                .filter(|artifact| {
                    artifact.role == PackagePublishArtifactRole::ExternalImportCertificate
                })
                .count(),
        )
        .unwrap(),
        artifact_count: u64::try_from(artifacts.len()).unwrap(),
        registry_entry_count: u64::try_from(registry_entries.len()).unwrap(),
        checker_summary_count: u64::try_from(checker_summaries.len()).unwrap(),
    }
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

fn read_checked_publish_plan(
    options: &PackageCommonOptions,
) -> Result<String, Box<CommandDiagnostic>> {
    let package_path = PackagePath::new(PACKAGE_PUBLISH_PLAN_PATH);
    let full_path = join_package_path(&options.root, &package_path, "publish_plan.path")?;
    fs::read_to_string(full_path).map_err(|error| {
        let reason = if error.kind() == io::ErrorKind::NotFound {
            "publish_plan_missing"
        } else {
            "generated_artifact_read_failed"
        };
        Box::new(
            CommandDiagnostic::error(DiagnosticKind::GeneratedArtifact, reason)
                .with_path(PACKAGE_PUBLISH_PLAN_PATH),
        )
    })
}

fn write_publish_plan(
    options: &PackageCommonOptions,
    publish_plan_json: &[u8],
) -> Result<(), Box<CommandDiagnostic>> {
    let package_path = PackagePath::new(PACKAGE_PUBLISH_PLAN_PATH);
    let full_path = join_package_path(&options.root, &package_path, "publish_plan.path")?;
    match fs::read(&full_path) {
        Ok(existing) if existing == publish_plan_json => return Ok(()),
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(_) => return Err(Box::new(write_failed_diagnostic())),
    }
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).map_err(|_| Box::new(write_failed_diagnostic()))?;
    }
    let temp_path = temporary_write_path(&full_path);
    if fs::write(&temp_path, publish_plan_json).is_err() {
        return Err(Box::new(write_failed_diagnostic()));
    }
    if fs::rename(&temp_path, &full_path).is_err() {
        let _ = fs::remove_file(&temp_path);
        return Err(Box::new(write_failed_diagnostic()));
    }
    Ok(())
}

fn temporary_write_path(path: &Path) -> std::path::PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("publish-plan.json");
    path.with_file_name(format!(".{file_name}.npa-publish-plan.tmp"))
}

fn passed_result(root_display: String) -> CommandResult {
    let mut result = CommandResult::passed(COMMAND, root_display);
    result.artifacts.push(crate::diagnostic::CommandArtifact {
        kind: "package_publish_plan".to_owned(),
        path: PACKAGE_PUBLISH_PLAN_PATH.to_owned(),
    });
    result
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

fn publish_plan_error_diagnostic(error: &PackageArtifactError) -> CommandDiagnostic {
    let reason_code = match error.reason_code {
        PackageArtifactErrorReason::NonCanonicalOrder => "publish_plan_non_canonical_order",
        PackageArtifactErrorReason::SelfHashMismatch => "publish_plan_hash_mismatch",
        _ => error.reason_code.as_str(),
    };
    let mut diagnostic = CommandDiagnostic::error(DiagnosticKind::GeneratedArtifact, reason_code)
        .with_path(PACKAGE_PUBLISH_PLAN_PATH);
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

fn publish_plan_stale_diagnostic(
    reason_code: &'static str,
    field: Option<&'static str>,
    checked_json: &str,
    generated_json: &str,
) -> CommandDiagnostic {
    let mut diagnostic = CommandDiagnostic::error(DiagnosticKind::GeneratedArtifact, reason_code)
        .with_path(PACKAGE_PUBLISH_PLAN_PATH)
        .with_hashes(
            format_package_hash(&package_file_hash(generated_json.as_bytes())),
            format_package_hash(&package_file_hash(checked_json.as_bytes())),
        );
    if let Some(field) = field {
        diagnostic = diagnostic.with_field(field);
    }
    diagnostic
}

fn write_failed_diagnostic() -> CommandDiagnostic {
    CommandDiagnostic::error(
        DiagnosticKind::GeneratedArtifact,
        "generated_artifact_write_failed",
    )
    .with_path(PACKAGE_PUBLISH_PLAN_PATH)
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
