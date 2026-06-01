use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use npa_api::{
    project_package_axiom_report_from_extraction, project_package_theorem_index_from_extraction,
    PackageArtifactReferenceSummaryMode,
};
use npa_cli::diagnostic::{CommandResult, PACKAGE_COMMAND_RESULT_SCHEMA};
use npa_cli::package::PACKAGE_MANIFEST_PATH;
use npa_cli::package_artifacts::{
    load_package_artifact_extraction, PackageGeneratedArtifactReadMode, PACKAGE_AXIOM_REPORT_PATH,
    PACKAGE_THEOREM_INDEX_PATH,
};
use npa_cli::package_publish::{load_package_publish_inputs, validate_publish_checker_summaries};
use npa_package::{
    build_package_lock_from_package_root, format_package_hash, parse_and_validate_manifest_str,
    parse_package_axiom_report_json, parse_package_theorem_index_json, PackageCheckerMode,
    PackageModule, PackagePath,
};

const LOCK_PATH: &str = "generated/package-lock.json";

static NEXT_TEMP_DIR: AtomicUsize = AtomicUsize::new(0);

struct Example<'a> {
    args: &'a [&'a str],
    success_prefix: &'a str,
    required_output: &'a [&'a str],
}

struct TestPackage {
    path: PathBuf,
}

impl TestPackage {
    fn new(label: &str) -> Self {
        let index = NEXT_TEMP_DIR.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "npa-cli-package-cli-{}-{label}-{index}",
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

impl Drop for TestPackage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn package_cli_examples_pass_on_proof_corpus() {
    let examples = [
        Example {
            args: ["package", "check", "--root", "proofs"].as_slice(),
            success_prefix: "package check: passed\n",
            required_output: &[],
        },
        Example {
            args: ["package", "build-certs", "--root", "proofs", "--check"].as_slice(),
            success_prefix: "package build-certs: passed\n",
            required_output: &[],
        },
        Example {
            args: [
                "package",
                "verify-certs",
                "--root",
                "proofs",
                "--checker",
                "reference",
            ]
            .as_slice(),
            success_prefix: "package verify-certs: passed\n",
            required_output: &["package_verified", "module_verified", "npa-checker-ref"],
        },
        Example {
            args: ["package", "check-hashes", "--root", "proofs"].as_slice(),
            success_prefix: "package check-hashes: passed\n",
            required_output: &[],
        },
    ];

    for example in examples {
        let args = example.args;
        let output = run_cli(args);

        assert_eq!(output.status.code(), Some(0), "{}", args.join(" "));
        assert!(output.stderr.is_empty(), "{}", args.join(" "));
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(
            stdout.starts_with(example.success_prefix),
            "{}",
            args.join(" ")
        );
        for required in example.required_output {
            assert!(
                stdout.contains(required),
                "{} missing {required}",
                args.join(" ")
            );
        }
    }
}

#[test]
fn package_cli_source_free_verify_succeeds_without_source_replay_or_meta() {
    let package = build_basic_package("source-free", false);
    assert!(!package.artifact_path("Proofs/Ai/Basic/source.npa").exists());
    assert!(!package
        .artifact_path("Proofs/Ai/Basic/replay.json")
        .exists());
    assert!(!package.artifact_path("Proofs/Ai/Basic/meta.json").exists());

    let output = Command::new(env!("CARGO_BIN_EXE_npa"))
        .args(["package", "verify-certs", "--root"])
        .arg(package.path())
        .args(["--checker", "reference", "--json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_json_envelope(&stdout, "package verify-certs", "passed");
    assert!(stdout.contains("\"root\":\"<absolute-root>\""));
    assert!(stdout.contains("\"kind\":\"ReferenceVerifier\""));
    assert!(stdout.contains("\"reason_code\":\"package_verified\""));
    assert!(stdout.contains("\"reason_code\":\"module_verified\""));
    assert!(stdout.contains("\"checker\":\"npa-checker-ref\""));
    assert_host_path_free(&stdout, &package);
}

#[test]
fn package_publish_inputs_collects_manifest_generated_metadata_and_reference_summaries() {
    let package = build_basic_package("publish-inputs", false);
    write_publish_input_metadata(&package);

    let loaded = load_package_publish_inputs(package.path()).unwrap();

    assert_eq!(
        loaded.validated.manifest().package.as_str(),
        "fixture-package"
    );
    assert_eq!(loaded.manifest.path.as_str(), PACKAGE_MANIFEST_PATH);
    assert_eq!(loaded.package_lock.path.as_str(), LOCK_PATH);
    assert_eq!(
        loaded.axiom_report_file.path.as_str(),
        PACKAGE_AXIOM_REPORT_PATH
    );
    assert_eq!(
        loaded.theorem_index_file.path.as_str(),
        PACKAGE_THEOREM_INDEX_PATH
    );
    assert_eq!(loaded.certificate_files.len(), 1);
    assert_eq!(
        loaded.reference_verification_report.verdict_source.as_str(),
        "npa-checker-ref"
    );
    assert!(
        loaded
            .reference_verification_report
            .reference_checker_verdict
    );

    let reference_summary = loaded
        .checker_summaries
        .iter()
        .find(|summary| summary.mode == PackageCheckerMode::Reference)
        .expect("collector records reference checker summary");
    assert_eq!(reference_summary.checker, "npa-checker-ref");
    assert_eq!(reference_summary.profile, "npa.checker.reference.v0.1");
    assert_eq!(reference_summary.status, "passed");
    assert!(loaded
        .checker_summaries
        .iter()
        .filter(|summary| summary.mode == PackageCheckerMode::Fast)
        .all(|summary| summary.checker != "npa-checker-ref"));
}

#[test]
fn package_publish_inputs_rejects_stale_lock_metadata_certificate_and_checker_summaries() {
    let stale_lock = build_basic_package("publish-stale-lock", false);
    write_publish_input_metadata(&stale_lock);
    let lock_path = stale_lock.artifact_path(LOCK_PATH);
    let mut lock_source = fs::read_to_string(&lock_path).unwrap();
    lock_source.push('\n');
    fs::write(&lock_path, lock_source).unwrap();
    assert_command_result_failure(
        load_package_publish_inputs(stale_lock.path()).unwrap_err(),
        "HashMismatch",
        "package_lock_stale",
    );

    let stale_axiom = build_basic_package("publish-stale-axiom", false);
    write_publish_input_metadata(&stale_axiom);
    rewrite_axiom_report_status(&stale_axiom, "failed");
    assert_command_result_failure(
        load_package_publish_inputs(stale_axiom.path()).unwrap_err(),
        "AxiomReport",
        "axiom_report_stale",
    );

    let stale_index = build_basic_package("publish-stale-index", false);
    write_publish_input_metadata(&stale_index);
    rewrite_theorem_index_status(&stale_index, "failed");
    assert_command_result_failure(
        load_package_publish_inputs(stale_index.path()).unwrap_err(),
        "TheoremIndex",
        "theorem_index_stale",
    );

    let stale_certificate = build_basic_package("publish-stale-certificate", false);
    write_publish_input_metadata(&stale_certificate);
    fs::copy(
        repo_root().join("proofs/Proofs/Ai/Prop/certificate.npcert"),
        stale_certificate.artifact_path("Proofs/Ai/Basic/certificate.npcert"),
    )
    .unwrap();
    assert_command_result_failure(
        load_package_publish_inputs(stale_certificate.path()).unwrap_err(),
        "HashMismatch",
        "certificate_file_hash_mismatch",
    );

    let valid = build_basic_package("publish-summary-validation", false);
    write_publish_input_metadata(&valid);
    let loaded = load_package_publish_inputs(valid.path()).unwrap();

    let mut missing_reference = loaded.checker_summaries.clone();
    missing_reference.retain(|summary| summary.mode != PackageCheckerMode::Reference);
    let missing = validate_publish_checker_summaries(
        &loaded.package_lock_manifest,
        &loaded.validated.manifest().checker_profile,
        &missing_reference,
    )
    .unwrap_err();
    assert_eq!(missing.kind.as_str(), "ReferenceVerifier");
    assert_eq!(missing.reason_code, "checker_summary_missing");

    let mut rejected = loaded.checker_summaries.clone();
    rejected
        .iter_mut()
        .find(|summary| summary.mode == PackageCheckerMode::Reference)
        .unwrap()
        .status = "failed".to_owned();
    let rejected = validate_publish_checker_summaries(
        &loaded.package_lock_manifest,
        &loaded.validated.manifest().checker_profile,
        &rejected,
    )
    .unwrap_err();
    assert_eq!(rejected.kind.as_str(), "ReferenceVerifier");
    assert_eq!(rejected.reason_code, "checker_summary_stale");
    assert_eq!(rejected.field.as_deref(), Some("status"));

    let mut mislabeled_fast = loaded.checker_summaries.clone();
    mislabeled_fast
        .iter_mut()
        .find(|summary| summary.mode == PackageCheckerMode::Fast)
        .unwrap()
        .checker = "npa-checker-ref".to_owned();
    let mislabeled = validate_publish_checker_summaries(
        &loaded.package_lock_manifest,
        &loaded.validated.manifest().checker_profile,
        &mislabeled_fast,
    )
    .unwrap_err();
    assert_eq!(mislabeled.kind.as_str(), "ReferenceVerifier");
    assert_eq!(mislabeled.reason_code, "checker_summary_stale");
    assert_eq!(mislabeled.field.as_deref(), Some("mode"));
}

#[test]
fn package_publish_source_free_boundary_ignores_source_replay_meta_ai_and_publish_plan() {
    let package = build_basic_package("publish-source-free", false);
    write_publish_input_metadata(&package);
    let module = proof_basic_module();
    write_directory(package.artifact_path(module.source.as_str()));
    if let Some(replay) = &module.replay {
        write_directory(package.artifact_path(replay.as_str()));
    }
    if let Some(meta) = &module.meta {
        write_directory(package.artifact_path(meta.as_str()));
    }
    write_directory(package.artifact_path("generated/publish-plan.json"));
    write_directory(package.artifact_path("ai/trace.json"));

    let loaded = load_package_publish_inputs(package.path()).unwrap();

    assert_eq!(loaded.artifact_extraction.verified_modules.len(), 1);
    assert!(package
        .artifact_path("generated/publish-plan.json")
        .is_dir());
}

#[test]
fn package_cli_temp_fixture_rejects_invalid_manifest() {
    let package = TestPackage::new("invalid-manifest");
    fs::write(package.artifact_path(PACKAGE_MANIFEST_PATH), "schema = ").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_npa"))
        .args(["package", "check", "--root"])
        .arg(package.path())
        .arg("--json")
        .output()
        .unwrap();

    assert_json_failure(
        output,
        &package,
        1,
        "package check",
        "PackageManifest",
        "invalid_toml",
    );
}

#[test]
fn package_cli_temp_fixture_rejects_stale_source_certificate_and_lock() {
    let stale_source = build_basic_package("stale-source", true);
    fs::write(
        stale_source.artifact_path("Proofs/Ai/Basic/source.npa"),
        b"changed source bytes",
    )
    .unwrap();
    let stale_source_output = Command::new(env!("CARGO_BIN_EXE_npa"))
        .args(["package", "check-hashes", "--root"])
        .arg(stale_source.path())
        .arg("--json")
        .output()
        .unwrap();
    assert_json_failure(
        stale_source_output,
        &stale_source,
        1,
        "package check-hashes",
        "HashMismatch",
        "source_hash_mismatch",
    );

    let stale_certificate = build_basic_package("stale-certificate", true);
    fs::copy(
        repo_root().join("proofs/Proofs/Ai/Prop/certificate.npcert"),
        stale_certificate.artifact_path("Proofs/Ai/Basic/certificate.npcert"),
    )
    .unwrap();
    let stale_certificate_output = Command::new(env!("CARGO_BIN_EXE_npa"))
        .args(["package", "check-hashes", "--root"])
        .arg(stale_certificate.path())
        .arg("--json")
        .output()
        .unwrap();
    assert_json_failure(
        stale_certificate_output,
        &stale_certificate,
        1,
        "package check-hashes",
        "HashMismatch",
        "certificate_file_hash_mismatch",
    );

    let stale_lock = build_basic_package("stale-lock", false);
    let lock_path = stale_lock.artifact_path(LOCK_PATH);
    let mut lock_source = fs::read_to_string(&lock_path).unwrap();
    lock_source.push('\n');
    fs::write(&lock_path, lock_source).unwrap();
    let stale_lock_output = Command::new(env!("CARGO_BIN_EXE_npa"))
        .args(["package", "verify-certs", "--root"])
        .arg(stale_lock.path())
        .args(["--checker", "reference", "--json"])
        .output()
        .unwrap();
    assert_json_failure(
        stale_lock_output,
        &stale_lock,
        1,
        "package verify-certs",
        "HashMismatch",
        "package_lock_stale",
    );
}

#[test]
fn package_cli_usage_failures_return_exit_two() {
    let unsupported_checker =
        run_cli(&["package", "verify-certs", "--checker", "external", "--json"]);
    assert_usage_failure(
        unsupported_checker,
        "package verify-certs",
        "unsupported_checker",
    );

    let unsupported_flag = run_cli(&["package", "check", "--changed", "--json"]);
    assert_usage_failure(unsupported_flag, "package check", "unsupported_flag");
}

fn build_basic_package(label: &str, include_source_sidecars: bool) -> TestPackage {
    let package = TestPackage::new(label);
    let module = proof_basic_module();
    assert!(module.imports.is_empty());
    copy_artifact(&package, module.certificate.as_str());
    if include_source_sidecars {
        copy_artifact(&package, module.source.as_str());
        if let Some(replay) = &module.replay {
            copy_artifact(&package, replay.as_str());
        }
        if let Some(meta) = &module.meta {
            copy_artifact(&package, meta.as_str());
        }
    }

    let manifest_source = basic_manifest(&module);
    fs::write(
        package.artifact_path(PACKAGE_MANIFEST_PATH),
        &manifest_source,
    )
    .unwrap();
    write_lock(&package, &manifest_source);
    package
}

fn basic_manifest(module: &PackageModule) -> String {
    let mut source = format!(
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
module = "{}"
source = "{}"
certificate = "{}"
"#,
        module.module.as_dotted(),
        module.source.as_str(),
        module.certificate.as_str(),
    );
    if let Some(meta) = &module.meta {
        source.push_str(&format!("meta = \"{}\"\n", meta.as_str()));
    }
    if let Some(replay) = &module.replay {
        source.push_str(&format!("replay = \"{}\"\n", replay.as_str()));
    }
    source.push_str(&format!(
        r#"imports = []
expected_source_hash = "{}"
expected_certificate_file_hash = "{}"
expected_export_hash = "{}"
expected_axiom_report_hash = "{}"
expected_certificate_hash = "{}"
inductives = []
definitions = []
theorems = []
axioms = []
tags = []
"#,
        format_package_hash(&module.expected_source_hash),
        format_package_hash(&module.expected_certificate_file_hash),
        format_package_hash(&module.expected_export_hash),
        format_package_hash(&module.expected_axiom_report_hash),
        format_package_hash(&module.expected_certificate_hash),
    ));
    source
}

fn proof_basic_module() -> PackageModule {
    let source = fs::read_to_string(repo_root().join("proofs/npa-package.toml")).unwrap();
    parse_and_validate_manifest_str(&source)
        .unwrap()
        .manifest()
        .modules
        .iter()
        .find(|module| module.module.as_dotted() == "Proofs.Ai.Basic")
        .unwrap()
        .clone()
}

fn write_lock(package: &TestPackage, manifest_source: &str) {
    let validated = parse_and_validate_manifest_str(manifest_source).unwrap();
    let lock = build_package_lock_from_package_root(
        &validated,
        package.path(),
        PackagePath::new(PACKAGE_MANIFEST_PATH),
    )
    .unwrap();
    let lock_json = lock.canonical_json().unwrap();
    let lock_path = package.artifact_path(LOCK_PATH);
    fs::create_dir_all(lock_path.parent().unwrap()).unwrap();
    fs::write(lock_path, lock_json).unwrap();
}

fn write_publish_input_metadata(package: &TestPackage) {
    let loaded = load_package_artifact_extraction(
        package.path(),
        "test package publish-plan metadata",
        PackageGeneratedArtifactReadMode::none(),
        PackageArtifactReferenceSummaryMode::Omit,
    )
    .unwrap();
    let axiom_report = project_package_axiom_report_from_extraction(
        &loaded.validated,
        &loaded.extraction,
        loaded.package_lock.clone(),
    )
    .unwrap();
    let theorem_index = project_package_theorem_index_from_extraction(
        &loaded.validated,
        &loaded.extraction,
        loaded.package_lock,
    )
    .unwrap();
    write_file(
        package.artifact_path(PACKAGE_AXIOM_REPORT_PATH),
        &axiom_report.canonical_json().unwrap(),
    );
    write_file(
        package.artifact_path(PACKAGE_THEOREM_INDEX_PATH),
        &theorem_index.canonical_json().unwrap(),
    );
}

fn rewrite_axiom_report_status(package: &TestPackage, status: &str) {
    let path = package.artifact_path(PACKAGE_AXIOM_REPORT_PATH);
    let mut report = parse_package_axiom_report_json(&fs::read_to_string(&path).unwrap()).unwrap();
    report.checker_summaries[0].status = status.to_owned();
    let report = report.with_computed_hash().unwrap();
    fs::write(path, report.canonical_json().unwrap()).unwrap();
}

fn rewrite_theorem_index_status(package: &TestPackage, status: &str) {
    let path = package.artifact_path(PACKAGE_THEOREM_INDEX_PATH);
    let mut index = parse_package_theorem_index_json(&fs::read_to_string(&path).unwrap()).unwrap();
    index.checker_summaries[0].status = status.to_owned();
    let index = index.with_computed_hash().unwrap();
    fs::write(path, index.canonical_json().unwrap()).unwrap();
}

fn write_file(path: PathBuf, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

fn write_directory(path: PathBuf) {
    fs::create_dir_all(path).unwrap();
}

fn copy_artifact(package: &TestPackage, relative: &str) {
    let source = repo_root().join("proofs").join(relative);
    let target = package.artifact_path(relative);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::copy(source, target).unwrap();
}

fn assert_command_result_failure(result: CommandResult, kind: &str, reason: &str) {
    assert_eq!(result.exit_code().as_u8(), 1);
    assert_eq!(result.command, "package publish-plan");
    assert_eq!(result.status.as_str(), "failed");
    assert_eq!(result.diagnostics[0].kind.as_str(), kind);
    assert_eq!(result.diagnostics[0].reason_code, reason);
}

fn assert_json_failure(
    output: Output,
    package: &TestPackage,
    code: i32,
    command: &str,
    kind: &str,
    reason: &str,
) {
    assert_eq!(output.status.code(), Some(code));
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_json_envelope(&stdout, command, "failed");
    assert!(stdout.contains(&format!("\"kind\":\"{kind}\"")));
    assert!(stdout.contains(&format!("\"reason_code\":\"{reason}\"")));
    assert_host_path_free(&stdout, package);
}

fn assert_usage_failure(output: Output, command: &str, reason: &str) {
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_json_envelope(&stdout, command, "failed");
    assert!(stdout.contains("\"kind\":\"Usage\""));
    assert!(stdout.contains(&format!("\"reason_code\":\"{reason}\"")));
}

fn assert_json_envelope(stdout: &str, command: &str, status: &str) {
    assert!(stdout.starts_with(&format!(
        "{{\"schema\":\"{PACKAGE_COMMAND_RESULT_SCHEMA}\",\"command\":\"{command}\","
    )));
    assert!(stdout.contains(&format!("\"status\":\"{status}\"")));
    assert!(stdout.contains("\"diagnostics\":["));
    assert!(stdout.ends_with("\"artifacts\":[]}\n"));
}

fn assert_host_path_free(stdout: &str, package: &TestPackage) {
    assert!(!stdout.contains(&package.path().to_string_lossy().to_string()));
    assert!(!stdout.contains("/tmp/"));
}

fn run_cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_npa"))
        .current_dir(repo_root())
        .args(args)
        .output()
        .unwrap()
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .components()
        .collect()
}
