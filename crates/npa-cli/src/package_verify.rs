//! Implementation of `npa package verify-certs`.

use std::{
    collections::BTreeMap,
    fs, io,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use npa_api::{
    format_hash_string, independent_checker_file_hash, independent_checker_machine_check_run,
    independent_checker_npa_checker_ext_launch_plan,
    independent_checker_resolve_checker_executable, materialize_package_phase8_requests,
    parse_hash_string, parse_independent_checker_binary_registry,
    parse_independent_checker_runner_policy, verify_package_fast_source_free,
    verify_package_reference_source_free, IndependentCheckerAllowlistEntry,
    IndependentCheckerBinaryRegistry, IndependentCheckerMachineCheckChecker,
    IndependentCheckerMachineCheckError, IndependentCheckerMachineCheckProcess,
    IndependentCheckerMachineCheckRequestPolicy, IndependentCheckerMachineCheckResourceUsage,
    IndependentCheckerMachineCheckResult, IndependentCheckerMachineCheckRunner,
    IndependentCheckerMachineCheckStatus, IndependentCheckerPolicyFailure,
    IndependentCheckerPolicyFailureReasonCode, IndependentCheckerPolicyValidationError,
    IndependentCheckerResolvedCheckerExecutable, IndependentCheckerRunObservation,
    IndependentCheckerRunnerPolicy, PackageCertificateArtifact, PackageModuleVerificationResult,
    PackageModuleVerificationStatus, PackagePhase8RequestMaterialization, PackageVerificationError,
    PackageVerificationErrorKind, PackageVerificationErrorReason, PackageVerificationMode,
    PackageVerificationReport, PackageVerificationStatus, PackageVerificationVerdictSource,
};
use npa_cert::{Hash, Name};
use npa_package::{
    build_package_lock_from_artifacts, format_package_hash, package_file_hash,
    parse_package_lock_json, PackageHash, PackageLockArtifact, PackageLockEntry,
    PackageLockManifest, PackagePath,
};

use crate::args::{PackageChecker, PackageExternalCheckerOptions, PackageVerifyCertsOptions};
use crate::diagnostic::{CommandArtifact, CommandDiagnostic, CommandResult, DiagnosticKind};
use crate::fs::{join_package_path, render_package_path, render_package_root};
use crate::package::{load_package_root, LoadedPackageRoot};

const COMMAND: &str = "package verify-certs";
const EXTERNAL_CHECKER_PROFILE: &str = "external";
const EXTERNAL_CHECKER_LABEL: &str = "npa-checker-ext";
const PACKAGE_LOCK_PATH: &str = "generated/package-lock.json";
const PACKAGE_VERIFY_STACK_BYTES: usize = 64 * 1024 * 1024;
const PACKAGE_EXTERNAL_RUNNER_ID: &str = "npa-cli-package-external-runner";
const PACKAGE_EXTERNAL_RUNNER_VERSION: &str = "0.1.0";

#[derive(Clone, Debug)]
struct CertificateArtifactBuffer {
    path: PackagePath,
    bytes: Vec<u8>,
}

/// Run source-free package certificate verification.
///
/// This command reads the package manifest, `generated/package-lock.json`, and
/// local/external certificate files. It intentionally does not read source,
/// replay, metadata, theorem-index, AI trace, network registry, or
/// checker-result sidecars. External checker mode additionally reads the
/// explicitly supplied runner policy, checker binary registry, checker binary,
/// and axiom policy.
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

    if checker == PackageChecker::External {
        let Some(external_options) = options.external.as_ref() else {
            return CommandResult::failed(
                COMMAND,
                loaded.root_display,
                vec![CommandDiagnostic::error(
                    DiagnosticKind::Usage,
                    "missing_external_checker_options",
                )
                .with_checker(EXTERNAL_CHECKER_LABEL)],
            );
        };
        return run_package_verify_external(&loaded, &checked_lock, &artifacts, external_options);
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

fn run_package_verify_external(
    loaded: &LoadedPackageRoot,
    lock: &PackageLockManifest,
    artifacts: &[CertificateArtifactBuffer],
    options: &PackageExternalCheckerOptions,
) -> CommandResult {
    let (policy, policy_path_display) = match load_external_runner_policy(loaded, options) {
        Ok(policy) => policy,
        Err(diagnostic) => {
            return CommandResult::failed(COMMAND, loaded.root_display.clone(), vec![*diagnostic]);
        }
    };
    if let Err(diagnostic) = validate_external_axiom_policy(loaded, &policy) {
        return CommandResult::failed(COMMAND, loaded.root_display.clone(), vec![*diagnostic]);
    }
    let registry = match load_external_checker_registry(loaded, options) {
        Ok(registry) => registry,
        Err(diagnostic) => {
            return CommandResult::failed(COMMAND, loaded.root_display.clone(), vec![*diagnostic]);
        }
    };
    let selected = match policy.selected_checker_policy(EXTERNAL_CHECKER_PROFILE) {
        Some(selected) => selected,
        None => {
            return CommandResult::failed(
                COMMAND,
                loaded.root_display.clone(),
                vec![CommandDiagnostic::error(
                    DiagnosticKind::ExternalVerifier,
                    "external_checker_profile_missing",
                )
                .with_field("checker_profile")
                .with_expected_value(EXTERNAL_CHECKER_PROFILE)
                .with_actual_value("missing")
                .with_checker(EXTERNAL_CHECKER_LABEL)],
            );
        }
    };
    let resolved = match resolve_external_checker_binary(loaded, &registry, selected) {
        Ok(resolved) => resolved,
        Err(diagnostic) => {
            return CommandResult::failed(COMMAND, loaded.root_display.clone(), vec![*diagnostic]);
        }
    };
    let materialized = match materialize_package_phase8_requests(
        lock,
        package_certificate_artifacts(artifacts),
        &policy,
        EXTERNAL_CHECKER_PROFILE,
        None,
    ) {
        Ok(report) => report,
        Err(error) => {
            return CommandResult::failed(
                COMMAND,
                loaded.root_display.clone(),
                vec![verification_error_diagnostic(
                    &error,
                    None,
                    DiagnosticKind::ExternalVerifier,
                    EXTERNAL_CHECKER_LABEL,
                )],
            );
        }
    };

    let mut machine_results = Vec::new();
    let mut result_artifacts = Vec::new();
    let artifact_bytes = artifact_bytes_by_path(artifacts);
    for module in &materialized.modules {
        if let Err(diagnostic) =
            materialize_external_import_dir(loaded, lock, module, &artifact_bytes)
        {
            return CommandResult::failed(COMMAND, loaded.root_display.clone(), vec![*diagnostic]);
        }
        let run = run_external_machine_check(loaded, lock, &policy, &resolved, module);
        let result_path = external_machine_result_path(lock, &module.module);
        if let Err(diagnostic) = write_external_machine_result(loaded, &result_path, &run) {
            return CommandResult::failed(COMMAND, loaded.root_display.clone(), vec![*diagnostic]);
        }
        result_artifacts.push(CommandArtifact {
            kind: "machine_check_result".to_owned(),
            path: result_path,
        });
        machine_results.push(run);
    }

    external_command_result_from_machine_results(
        loaded.root_display.clone(),
        lock,
        &policy_path_display,
        machine_results,
        result_artifacts,
    )
}

fn load_external_runner_policy(
    loaded: &LoadedPackageRoot,
    options: &PackageExternalCheckerOptions,
) -> Result<(IndependentCheckerRunnerPolicy, String), Box<CommandDiagnostic>> {
    let path = package_path_from_cli(&options.runner_policy, "--runner-policy")?;
    let path_display = render_package_path(&path);
    let source = read_package_text(loaded, &path, "runner_policy_missing")?;
    let expected_hash = parse_hash_string(&options.runner_policy_hash).map_err(|_| {
        Box::new(
            CommandDiagnostic::error(DiagnosticKind::PackagePolicy, "invalid_hash_format")
                .with_path(path_display.clone())
                .with_field("--runner-policy-hash")
                .with_expected_value("sha256:<lower-hex>")
                .with_actual_value(options.runner_policy_hash.clone()),
        )
    })?;
    let policy = parse_independent_checker_runner_policy(&source)
        .map_err(|error| Box::new(policy_validation_diagnostic("runner_policy_invalid", error)))?;
    let actual_hash = policy.policy_hash();
    if actual_hash != expected_hash {
        return Err(Box::new(
            CommandDiagnostic::error(DiagnosticKind::HashMismatch, "runner_policy_hash_mismatch")
                .with_path(path_display)
                .with_field("--runner-policy-hash")
                .with_hashes(
                    format_hash_string(&expected_hash),
                    format_hash_string(&actual_hash),
                ),
        ));
    }
    Ok((policy, render_package_path(&path)))
}

fn validate_external_axiom_policy(
    loaded: &LoadedPackageRoot,
    policy: &IndependentCheckerRunnerPolicy,
) -> Result<(), Box<CommandDiagnostic>> {
    let path = PackagePath::new(policy.axiom_policy.path.clone());
    let bytes = read_package_bytes(loaded, &path, "axiom_policy_missing")?;
    let actual_hash = independent_checker_file_hash(&bytes);
    if actual_hash != policy.axiom_policy.hash {
        return Err(Box::new(
            CommandDiagnostic::error(DiagnosticKind::HashMismatch, "axiom_policy_hash_mismatch")
                .with_path(render_package_path(&path))
                .with_field("runner_policy.axiom_policy.hash")
                .with_hashes(
                    format_hash_string(&policy.axiom_policy.hash),
                    format_hash_string(&actual_hash),
                ),
        ));
    }
    Ok(())
}

fn load_external_checker_registry(
    loaded: &LoadedPackageRoot,
    options: &PackageExternalCheckerOptions,
) -> Result<IndependentCheckerBinaryRegistry, Box<CommandDiagnostic>> {
    let path = package_path_from_cli(&options.checker_registry, "--checker-registry")?;
    let source = read_package_text(loaded, &path, "checker_registry_missing")?;
    parse_independent_checker_binary_registry(&source).map_err(|error| {
        Box::new(policy_validation_diagnostic(
            "checker_registry_invalid",
            error,
        ))
    })
}

fn resolve_external_checker_binary(
    loaded: &LoadedPackageRoot,
    registry: &IndependentCheckerBinaryRegistry,
    selected: &IndependentCheckerAllowlistEntry,
) -> Result<IndependentCheckerResolvedCheckerExecutable, Box<CommandDiagnostic>> {
    let Some(entry) = registry
        .entries
        .iter()
        .find(|entry| entry.binary_id == selected.binary_id)
    else {
        let failure = IndependentCheckerPolicyFailure {
            reason_code: IndependentCheckerPolicyFailureReasonCode::CheckerBinaryFileUnreadable,
            field: "checker.binary_id".to_owned().into_boxed_str(),
            expected_value: Some("readable_executable".to_owned().into_boxed_str()),
            actual_value: Some("binary_id_not_found".to_owned().into_boxed_str()),
            expected_hash: None,
            actual_hash: None,
        };
        return Err(Box::new(policy_failure_diagnostic(failure, None)));
    };
    let binary_path = PackagePath::new(entry.path.clone());
    let binary_bytes = read_package_bytes(loaded, &binary_path, "checker_binary_file_unreadable")?;
    let actual_binary_hash = independent_checker_file_hash(&binary_bytes);
    independent_checker_resolve_checker_executable(registry, selected, actual_binary_hash).map_err(
        |failure| {
            Box::new(policy_failure_diagnostic(
                failure,
                Some(render_package_path(&binary_path)),
            ))
        },
    )
}

fn materialize_external_import_dir(
    loaded: &LoadedPackageRoot,
    lock: &PackageLockManifest,
    module: &PackagePhase8RequestMaterialization,
    artifact_bytes: &BTreeMap<String, &[u8]>,
) -> Result<(), Box<CommandDiagnostic>> {
    let import_dir = external_import_dir_path(lock, &module.module);
    let full_import_dir = loaded.root.join(&import_dir);
    if full_import_dir.exists() {
        fs::remove_dir_all(&full_import_dir).map_err(|_| {
            Box::new(
                CommandDiagnostic::error(DiagnosticKind::ArtifactIo, "import_dir_rewrite_failed")
                    .with_path(import_dir.clone()),
            )
        })?;
    }
    fs::create_dir_all(&full_import_dir).map_err(|_| {
        Box::new(
            CommandDiagnostic::error(DiagnosticKind::ArtifactIo, "import_dir_create_failed")
                .with_path(import_dir.clone()),
        )
    })?;
    for import in &module.import_lock_manifest.imports {
        let Some(bytes) = artifact_bytes.get(&import.certificate.path) else {
            return Err(Box::new(
                CommandDiagnostic::error(DiagnosticKind::ArtifactIo, "certificate_missing")
                    .with_path(import.certificate.path.clone())
                    .with_module(import.module.clone()),
            ));
        };
        let target = full_import_dir.join(&import.certificate.path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|_| {
                Box::new(
                    CommandDiagnostic::error(
                        DiagnosticKind::ArtifactIo,
                        "import_certificate_dir_create_failed",
                    )
                    .with_path(import.certificate.path.clone()),
                )
            })?;
        }
        fs::write(&target, bytes).map_err(|_| {
            Box::new(
                CommandDiagnostic::error(
                    DiagnosticKind::ArtifactIo,
                    "import_certificate_write_failed",
                )
                .with_path(import.certificate.path.clone()),
            )
        })?;
    }
    Ok(())
}

fn run_external_machine_check(
    loaded: &LoadedPackageRoot,
    lock: &PackageLockManifest,
    policy: &IndependentCheckerRunnerPolicy,
    resolved: &IndependentCheckerResolvedCheckerExecutable,
    module: &PackagePhase8RequestMaterialization,
) -> IndependentCheckerMachineCheckResult {
    let import_dir = external_import_dir_path(lock, &module.module);
    let launch = independent_checker_npa_checker_ext_launch_plan(
        resolved,
        &module.request,
        import_dir.clone(),
    );
    let executable = loaded.root.join(&resolved.path);
    let observation = external_run_observation(
        &loaded.root,
        &executable,
        &launch.argv,
        &launch.environment,
        module,
    );
    independent_checker_machine_check_run(&module.request, policy, observation).unwrap_or_else(
        |error| {
            let mut machine_error =
                IndependentCheckerMachineCheckError::new("checker_internal_error")
                    .with_reason_code(error.reason_code.to_string());
            if let (Some(field), Some(expected), Some(actual)) = (
                error.field.clone(),
                error.expected_value.clone(),
                error.actual_value.clone(),
            ) {
                machine_error = machine_error.with_value_payload(
                    field.into_string(),
                    expected.into_string(),
                    actual.into_string(),
                );
            } else if let (Some(field), Some(expected), Some(actual)) =
                (error.field, error.expected_hash, error.actual_hash)
            {
                machine_error =
                    machine_error.with_hash_payload(field.into_string(), *expected, *actual);
            }
            IndependentCheckerMachineCheckResult {
                request_id: module.request.request_id.clone(),
                request_hash: module.request.request_hash(),
                result_id: external_machine_result_id(&module.module),
                policy: IndependentCheckerMachineCheckRequestPolicy {
                    id: policy.id.clone(),
                    version: policy.version,
                    hash: policy.policy_hash(),
                },
                runner: external_runner_identity(),
                checker: IndependentCheckerMachineCheckChecker {
                    profile: EXTERNAL_CHECKER_PROFILE.to_owned(),
                    binary_id: Some(resolved.binary_id.clone()),
                    binary_hash: Some(resolved.binary_hash),
                    id: None,
                    build_hash: None,
                    version: None,
                },
                attempt: 1,
                status: IndependentCheckerMachineCheckStatus::Failed,
                module: module.module.as_dotted(),
                process: IndependentCheckerMachineCheckProcess::not_launched(),
                resource_usage: IndependentCheckerMachineCheckResourceUsage::zero(),
                error: Some(machine_error),
                certificate_hash: None,
                export_hash: None,
                axiom_report_hash: None,
                diagnostics: Vec::new(),
                axioms_used: None,
                declarations_checked: None,
                raw_checker_output_hex: None,
            }
        },
    )
}

fn external_run_observation(
    root: &Path,
    executable: &Path,
    argv: &[String],
    environment: &[(String, String)],
    module: &PackagePhase8RequestMaterialization,
) -> IndependentCheckerRunObservation {
    let started = Instant::now();
    let mut command = Command::new(executable);
    command
        .args(argv.iter().skip(1))
        .current_dir(root)
        .env_clear()
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (key, value) in environment {
        command.env(key, value);
    }

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            return IndependentCheckerRunObservation {
                result_id: external_machine_result_id(&module.module),
                attempt: 1,
                runner: external_runner_identity(),
                process: IndependentCheckerMachineCheckProcess::not_launched(),
                resource_usage: IndependentCheckerMachineCheckResourceUsage {
                    steps: 0,
                    memory_peak_mb: 0,
                    elapsed_ms: elapsed_ms(started),
                },
                stdout: Vec::new(),
                stderr: error.to_string().into_bytes(),
            };
        }
    };

    let timeout = Duration::from_millis(module.request.budget.timeout_ms);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                return match child.wait_with_output() {
                    Ok(output) => IndependentCheckerRunObservation {
                        result_id: external_machine_result_id(&module.module),
                        attempt: 1,
                        runner: external_runner_identity(),
                        process: process_from_exit_status(output.status.code()),
                        resource_usage: IndependentCheckerMachineCheckResourceUsage {
                            steps: 0,
                            memory_peak_mb: 0,
                            elapsed_ms: elapsed_ms(started),
                        },
                        stdout: output.stdout,
                        stderr: output.stderr,
                    },
                    Err(error) => IndependentCheckerRunObservation {
                        result_id: external_machine_result_id(&module.module),
                        attempt: 1,
                        runner: external_runner_identity(),
                        process: IndependentCheckerMachineCheckProcess::terminated(
                            "killed_without_exit_status",
                        ),
                        resource_usage: IndependentCheckerMachineCheckResourceUsage {
                            steps: 0,
                            memory_peak_mb: 0,
                            elapsed_ms: elapsed_ms(started),
                        },
                        stdout: Vec::new(),
                        stderr: error.to_string().into_bytes(),
                    },
                };
            }
            Ok(None) if started.elapsed() >= timeout => {
                let _ = child.kill();
                let output = child.wait_with_output();
                let (stdout, stderr) = output
                    .map(|output| (output.stdout, output.stderr))
                    .unwrap_or_else(|error| (Vec::new(), error.to_string().into_bytes()));
                return IndependentCheckerRunObservation {
                    result_id: external_machine_result_id(&module.module),
                    attempt: 1,
                    runner: external_runner_identity(),
                    process: IndependentCheckerMachineCheckProcess::terminated("timeout"),
                    resource_usage: IndependentCheckerMachineCheckResourceUsage {
                        steps: 0,
                        memory_peak_mb: 0,
                        elapsed_ms: elapsed_ms(started),
                    },
                    stdout,
                    stderr,
                };
            }
            Ok(None) => thread::sleep(Duration::from_millis(5)),
            Err(error) => {
                return IndependentCheckerRunObservation {
                    result_id: external_machine_result_id(&module.module),
                    attempt: 1,
                    runner: external_runner_identity(),
                    process: IndependentCheckerMachineCheckProcess::terminated(
                        "killed_without_exit_status",
                    ),
                    resource_usage: IndependentCheckerMachineCheckResourceUsage {
                        steps: 0,
                        memory_peak_mb: 0,
                        elapsed_ms: elapsed_ms(started),
                    },
                    stdout: Vec::new(),
                    stderr: error.to_string().into_bytes(),
                };
            }
        }
    }
}

fn external_command_result_from_machine_results(
    root_display: String,
    lock: &PackageLockManifest,
    policy_path: &str,
    machine_results: Vec<IndependentCheckerMachineCheckResult>,
    artifacts: Vec<CommandArtifact>,
) -> CommandResult {
    let entries_by_module = lock_entries_by_module(lock);
    let mut diagnostics = Vec::new();
    for result in &machine_results {
        let result_path = external_machine_result_path(lock, &module_name_from_result(result));
        if let Some(diagnostic) =
            external_result_failure_diagnostic(result, &result_path, &entries_by_module)
        {
            diagnostics.push(diagnostic);
        }
    }

    if diagnostics.is_empty() {
        let mut result = CommandResult::passed(COMMAND, root_display);
        result.diagnostics = external_passed_diagnostics(lock, policy_path, &machine_results);
        result.artifacts = artifacts;
        result
    } else {
        let mut result = CommandResult::failed(COMMAND, root_display, diagnostics);
        result.artifacts = artifacts;
        result
    }
}

fn external_passed_diagnostics(
    lock: &PackageLockManifest,
    policy_path: &str,
    machine_results: &[IndependentCheckerMachineCheckResult],
) -> Vec<CommandDiagnostic> {
    let entries_by_module = lock_entries_by_module(lock);
    let mut diagnostics = vec![
        CommandDiagnostic::info(DiagnosticKind::ExternalVerifier, "package_verified")
            .with_field("verdict_source")
            .with_path(policy_path)
            .with_actual_value(format!(
                "mode=external;verdict_source={EXTERNAL_CHECKER_LABEL};reference_checker_verdict=false;modules={}",
                machine_results.len()
            ))
            .with_checker(EXTERNAL_CHECKER_LABEL),
    ];
    diagnostics.extend(machine_results.iter().map(|result| {
        let path = entries_by_module
            .get(&module_name_from_result(result))
            .map(|entry| entry.certificate.as_str())
            .unwrap_or("<unknown-certificate>");
        CommandDiagnostic::info(DiagnosticKind::ExternalVerifier, "module_verified")
            .with_module(result.module.clone())
            .with_path(path)
            .with_field("status")
            .with_expected_value(IndependentCheckerMachineCheckStatus::Checked.as_str())
            .with_actual_value(result.status.as_str())
            .with_checker(EXTERNAL_CHECKER_LABEL)
    }));
    diagnostics
}

fn external_result_failure_diagnostic(
    result: &IndependentCheckerMachineCheckResult,
    result_path: &str,
    entries_by_module: &BTreeMap<Name, &PackageLockEntry>,
) -> Option<CommandDiagnostic> {
    if result.status != IndependentCheckerMachineCheckStatus::Checked {
        return Some(machine_result_error_diagnostic(result, result_path));
    }
    let module = module_name_from_result(result);
    let Some(entry) = entries_by_module.get(&module) else {
        return Some(
            CommandDiagnostic::error(
                DiagnosticKind::ExternalVerifier,
                "module_not_in_package_lock",
            )
            .with_path(result_path)
            .with_module(result.module.clone())
            .with_checker(EXTERNAL_CHECKER_LABEL),
        );
    };
    external_hash_failure(ExternalHashCheck {
        result_path,
        module: &result.module,
        field: "certificate_hash",
        missing_reason: "certificate_hash_missing",
        mismatch_reason: "certificate_hash_mismatch",
        expected: entry.certificate_hash,
        actual: result.certificate_hash,
    })
    .or_else(|| {
        external_hash_failure(ExternalHashCheck {
            result_path,
            module: &result.module,
            field: "export_hash",
            missing_reason: "export_hash_missing",
            mismatch_reason: "export_hash_mismatch",
            expected: entry.export_hash,
            actual: result.export_hash,
        })
    })
    .or_else(|| {
        external_hash_failure(ExternalHashCheck {
            result_path,
            module: &result.module,
            field: "axiom_report_hash",
            missing_reason: "axiom_report_hash_missing",
            mismatch_reason: "axiom_report_hash_mismatch",
            expected: entry.axiom_report_hash,
            actual: result.axiom_report_hash,
        })
    })
}

struct ExternalHashCheck<'a> {
    result_path: &'a str,
    module: &'a str,
    field: &'static str,
    missing_reason: &'static str,
    mismatch_reason: &'static str,
    expected: PackageHash,
    actual: Option<Hash>,
}

fn external_hash_failure(check: ExternalHashCheck<'_>) -> Option<CommandDiagnostic> {
    match check.actual {
        Some(actual) if actual == check.expected.into_bytes() => None,
        Some(actual) => Some(
            CommandDiagnostic::error(DiagnosticKind::HashMismatch, check.mismatch_reason)
                .with_path(check.result_path)
                .with_module(check.module)
                .with_field(check.field)
                .with_hashes(
                    format_package_hash(&check.expected),
                    format_hash_string(&actual),
                )
                .with_checker(EXTERNAL_CHECKER_LABEL),
        ),
        None => Some(
            CommandDiagnostic::error(DiagnosticKind::ExternalVerifier, check.missing_reason)
                .with_path(check.result_path)
                .with_module(check.module)
                .with_field(check.field)
                .with_expected_value(format_package_hash(&check.expected))
                .with_actual_value("missing")
                .with_checker(EXTERNAL_CHECKER_LABEL),
        ),
    }
}

fn machine_result_error_diagnostic(
    result: &IndependentCheckerMachineCheckResult,
    result_path: &str,
) -> CommandDiagnostic {
    let Some(error) = result.error.as_ref() else {
        return CommandDiagnostic::error(
            DiagnosticKind::ExternalVerifier,
            "external_checker_failed",
        )
        .with_path(result_path)
        .with_module(result.module.clone())
        .with_field("status")
        .with_expected_value(IndependentCheckerMachineCheckStatus::Checked.as_str())
        .with_actual_value(result.status.as_str())
        .with_checker(EXTERNAL_CHECKER_LABEL);
    };
    let mut diagnostic = CommandDiagnostic::error(
        if error.expected_hash.is_some() || error.actual_hash.is_some() {
            DiagnosticKind::HashMismatch
        } else {
            DiagnosticKind::ExternalVerifier
        },
        error.reason_code.as_deref().unwrap_or(&error.kind),
    )
    .with_path(result_path)
    .with_module(result.module.clone())
    .with_checker(EXTERNAL_CHECKER_LABEL);
    if let Some(field) = &error.field {
        diagnostic = diagnostic.with_field(field.clone());
    }
    if let (Some(expected), Some(actual)) = (error.expected_hash, error.actual_hash) {
        diagnostic =
            diagnostic.with_hashes(format_hash_string(&expected), format_hash_string(&actual));
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

fn write_external_machine_result(
    loaded: &LoadedPackageRoot,
    result_path: &str,
    result: &IndependentCheckerMachineCheckResult,
) -> Result<(), Box<CommandDiagnostic>> {
    let full_path = loaded.root.join(result_path);
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).map_err(|_| {
            Box::new(
                CommandDiagnostic::error(
                    DiagnosticKind::ArtifactIo,
                    "machine_result_dir_create_failed",
                )
                .with_path(result_path),
            )
        })?;
    }
    fs::write(&full_path, result.canonical_json()).map_err(|_| {
        Box::new(
            CommandDiagnostic::error(DiagnosticKind::ArtifactIo, "machine_result_write_failed")
                .with_path(result_path),
        )
    })
}

fn package_path_from_cli(
    path: &Path,
    field: &'static str,
) -> Result<PackagePath, Box<CommandDiagnostic>> {
    let value = path.to_string_lossy().replace('\\', "/");
    let package_path = PackagePath::new(value);
    npa_package::validate_package_path(&package_path, field).map_err(|error| {
        Box::new(CommandDiagnostic::from_package_manifest_error(&error).with_field(field))
    })?;
    Ok(package_path)
}

fn read_package_text(
    loaded: &LoadedPackageRoot,
    path: &PackagePath,
    missing_reason: &str,
) -> Result<String, Box<CommandDiagnostic>> {
    let bytes = read_package_bytes(loaded, path, missing_reason)?;
    String::from_utf8(bytes).map_err(|_| {
        Box::new(
            CommandDiagnostic::error(DiagnosticKind::ArtifactIo, "artifact_not_utf8")
                .with_path(render_package_path(path)),
        )
    })
}

fn read_package_bytes(
    loaded: &LoadedPackageRoot,
    path: &PackagePath,
    missing_reason: &str,
) -> Result<Vec<u8>, Box<CommandDiagnostic>> {
    let full_path = join_package_path(&loaded.root, path, "external_checker.path")?;
    fs::read(full_path).map_err(|_| {
        Box::new(
            CommandDiagnostic::error(DiagnosticKind::ArtifactIo, missing_reason)
                .with_path(render_package_path(path)),
        )
    })
}

fn policy_validation_diagnostic(
    reason_code: &str,
    error: IndependentCheckerPolicyValidationError,
) -> CommandDiagnostic {
    CommandDiagnostic::error(DiagnosticKind::PackagePolicy, reason_code)
        .with_field(error.field)
        .with_expected_value(error.expected_value)
        .with_actual_value(error.actual_value)
        .with_checker(EXTERNAL_CHECKER_LABEL)
}

fn policy_failure_diagnostic(
    failure: IndependentCheckerPolicyFailure,
    path: Option<String>,
) -> CommandDiagnostic {
    let mut diagnostic = CommandDiagnostic::error(
        if failure.expected_hash.is_some() || failure.actual_hash.is_some() {
            DiagnosticKind::HashMismatch
        } else {
            DiagnosticKind::ExternalVerifier
        },
        failure.reason_code.as_str(),
    )
    .with_field(failure.field.to_string())
    .with_checker(EXTERNAL_CHECKER_LABEL);
    if let Some(path) = path {
        diagnostic = diagnostic.with_path(path);
    }
    if let (Some(expected), Some(actual)) = (failure.expected_hash, failure.actual_hash) {
        diagnostic =
            diagnostic.with_hashes(format_hash_string(&expected), format_hash_string(&actual));
    } else {
        if let Some(expected) = failure.expected_value {
            diagnostic = diagnostic.with_expected_value(expected.to_string());
        }
        if let Some(actual) = failure.actual_value {
            diagnostic = diagnostic.with_actual_value(actual.to_string());
        }
    }
    diagnostic
}

fn artifact_bytes_by_path(artifacts: &[CertificateArtifactBuffer]) -> BTreeMap<String, &[u8]> {
    artifacts
        .iter()
        .map(|artifact| (artifact.path.as_str().to_owned(), artifact.bytes.as_slice()))
        .collect()
}

fn external_runner_identity() -> IndependentCheckerMachineCheckRunner {
    IndependentCheckerMachineCheckRunner {
        id: PACKAGE_EXTERNAL_RUNNER_ID.to_owned(),
        version: PACKAGE_EXTERNAL_RUNNER_VERSION.to_owned(),
        build_hash: independent_checker_file_hash(
            format!("{PACKAGE_EXTERNAL_RUNNER_ID}:{PACKAGE_EXTERNAL_RUNNER_VERSION}").as_bytes(),
        ),
    }
}

fn process_from_exit_status(code: Option<i32>) -> IndependentCheckerMachineCheckProcess {
    code.and_then(|code| u8::try_from(code).ok())
        .map(IndependentCheckerMachineCheckProcess::exited)
        .unwrap_or_else(|| {
            IndependentCheckerMachineCheckProcess::terminated("killed_without_exit_status")
        })
}

fn elapsed_ms(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn module_name_from_result(result: &IndependentCheckerMachineCheckResult) -> Name {
    Name::from_dotted(&result.module)
}

fn external_machine_result_id(module: &Name) -> String {
    format!(
        "mchkres_package_{}_external",
        module.as_dotted().replace('.', "_")
    )
}

fn external_import_dir_path(lock: &PackageLockManifest, module: &Name) -> String {
    format!(
        "generated/checker-imports/{}/{}/{}/external",
        lock.package.as_str(),
        lock.version.as_str(),
        module.as_dotted()
    )
}

fn external_machine_result_path(lock: &PackageLockManifest, module: &Name) -> String {
    format!(
        "generated/checker-results/{}/{}/{}/external/result.json",
        lock.package.as_str(),
        lock.version.as_str(),
        module.as_dotted()
    )
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
        PackageChecker::External => {
            unreachable!("external checker is handled before verify_package")
        }
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
        PackageChecker::External => DiagnosticKind::ExternalVerifier,
    }
}

fn checker_label(checker: PackageChecker) -> &'static str {
    match checker {
        PackageChecker::Reference => PackageVerificationVerdictSource::ReferenceChecker.as_str(),
        PackageChecker::Fast => {
            PackageVerificationVerdictSource::FastKernelCertificateVerifier.as_str()
        }
        PackageChecker::External => EXTERNAL_CHECKER_LABEL,
    }
}

fn lock_entries_by_module(lock: &PackageLockManifest) -> BTreeMap<Name, &PackageLockEntry> {
    lock.entries
        .iter()
        .map(|entry| (entry.module.clone(), entry))
        .collect()
}
