use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use npa_cert::Name;
use npa_cli::args::{PackageAxiomReportOptions, PackageCommonOptions};
use npa_cli::diagnostic::{CommandExitCode, DiagnosticKind, PACKAGE_COMMAND_RESULT_SCHEMA};
use npa_cli::package::PACKAGE_MANIFEST_PATH;
use npa_cli::package_artifacts::{PACKAGE_AXIOM_REPORT_PATH, PACKAGE_THEOREM_INDEX_PATH};
use npa_cli::package_axiom_report::run_package_axiom_report;
use npa_package::{
    build_package_lock_from_package_root, format_package_hash, parse_and_validate_manifest_str,
    parse_package_axiom_report_json, PackageExternalImport, PackageHash, PackageModule,
    PackagePath,
};

const LOCK_PATH: &str = "generated/package-lock.json";

static NEXT_TEMP_DIR: AtomicUsize = AtomicUsize::new(0);

struct TestPackage {
    path: PathBuf,
}

impl TestPackage {
    fn new(label: &str) -> Self {
        let index = NEXT_TEMP_DIR.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "npa-cli-package-axiom-report-{}-{label}-{index}",
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

#[derive(Clone)]
struct ManifestModule {
    module: Name,
    source: String,
    certificate: String,
    meta: Option<String>,
    replay: Option<String>,
    imports: Vec<Name>,
    source_hash: PackageHash,
    certificate_file_hash: PackageHash,
    export_hash: PackageHash,
    axiom_report_hash: PackageHash,
    certificate_hash: PackageHash,
}

#[test]
fn package_axiom_report_write_creates_only_report_from_source_free_inputs() {
    let package =
        build_source_free_fixture("write-source-free", "Proofs.Ai.Basic", false, &["Eq.rec"]);
    assert!(!package.artifact_path("Proofs/Ai/Basic/source.npa").exists());
    assert!(!package
        .artifact_path("Proofs/Ai/Basic/replay.json")
        .exists());
    assert!(!package.artifact_path("Proofs/Ai/Basic/meta.json").exists());
    assert!(!package.artifact_path(PACKAGE_THEOREM_INDEX_PATH).exists());
    let before = collect_files(package.path());

    let result = run_write(&package);

    assert_eq!(result.exit_code(), CommandExitCode::Success);
    assert!(result.diagnostics.is_empty());
    assert_eq!(result.artifacts.len(), 1);
    assert_eq!(result.artifacts[0].kind, "package_axiom_report");
    assert_eq!(result.artifacts[0].path, PACKAGE_AXIOM_REPORT_PATH);
    let after = collect_files(package.path());
    let added = after
        .keys()
        .filter(|path| !before.contains_key(*path))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(added, vec![PACKAGE_AXIOM_REPORT_PATH.to_owned()]);
    assert!(parse_package_axiom_report_json(
        &fs::read_to_string(package.artifact_path(PACKAGE_AXIOM_REPORT_PATH)).unwrap()
    )
    .is_ok());
}

#[test]
fn package_axiom_report_check_succeeds_and_writes_no_files() {
    let package =
        build_source_free_fixture("check-no-write", "Proofs.Ai.Basic", false, &["Eq.rec"]);
    assert_eq!(run_write(&package).exit_code(), CommandExitCode::Success);
    let before = collect_files(package.path());

    let result = run_check(&package);

    assert_eq!(result.exit_code(), CommandExitCode::Success);
    assert!(result.diagnostics.is_empty());
    assert_eq!(collect_files(package.path()), before);
}

#[test]
fn package_axiom_report_check_rejects_missing_stale_and_noncanonical_reports() {
    let missing = build_source_free_fixture("missing", "Proofs.Ai.Basic", false, &["Eq.rec"]);
    assert_failure(
        &run_check(&missing),
        DiagnosticKind::AxiomReport,
        "axiom_report_missing",
    );

    let stale = build_source_free_fixture("stale", "Proofs.Ai.Basic", false, &["Eq.rec"]);
    assert_eq!(run_write(&stale).exit_code(), CommandExitCode::Success);
    let lock_path = stale.artifact_path(LOCK_PATH);
    let mut lock_source = fs::read_to_string(&lock_path).unwrap();
    lock_source.push('\n');
    fs::write(lock_path, lock_source).unwrap();
    assert_failure(
        &run_check(&stale),
        DiagnosticKind::AxiomReport,
        "axiom_report_stale",
    );

    let noncanonical =
        build_source_free_fixture("noncanonical", "Proofs.Ai.Basic", false, &["Eq.rec"]);
    assert_eq!(
        run_write(&noncanonical).exit_code(),
        CommandExitCode::Success
    );
    let report_path = noncanonical.artifact_path(PACKAGE_AXIOM_REPORT_PATH);
    let mut report_source = fs::read_to_string(&report_path).unwrap();
    report_source.push('\n');
    fs::write(report_path, report_source).unwrap();
    assert_failure(
        &run_check(&noncanonical),
        DiagnosticKind::AxiomReport,
        "axiom_report_non_canonical_order",
    );
}

#[test]
fn package_axiom_report_policy_failure_is_structured() {
    let package = build_source_free_fixture("policy-failure", "Proofs.Ai.EqReasoning", true, &[]);

    let result = run_write(&package);

    assert_failure(
        &result,
        DiagnosticKind::PackagePolicy,
        "axiom_report_policy_violation",
    );
    assert!(!package.artifact_path(PACKAGE_AXIOM_REPORT_PATH).exists());
    assert!(!result.render_json().contains("/tmp/"));
}

#[test]
fn package_axiom_report_cli_check_json_uses_command_result_schema() {
    let package = build_source_free_fixture("cli-json", "Proofs.Ai.Basic", false, &["Eq.rec"]);
    assert_eq!(run_write(&package).exit_code(), CommandExitCode::Success);

    let output = Command::new(env!("CARGO_BIN_EXE_npa"))
        .args(["package", "axiom-report", "--root"])
        .arg(package.path())
        .args(["--check", "--json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with(&format!(
        "{{\"schema\":\"{PACKAGE_COMMAND_RESULT_SCHEMA}\",\"command\":\"package axiom-report\","
    )));
    assert!(stdout.contains("\"status\":\"passed\""));
    assert!(stdout.contains("\"kind\":\"package_axiom_report\""));
    assert!(stdout.contains(&format!("\"path\":\"{PACKAGE_AXIOM_REPORT_PATH}\"")));
    assert!(!stdout.contains(&package.path().to_string_lossy().to_string()));
}

#[test]
fn package_axiom_report_cli_usage_errors_return_exit_two() {
    let output = Command::new(env!("CARGO_BIN_EXE_npa"))
        .args(["package", "axiom-report", "--include-source", "--json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"command\":\"package axiom-report\""));
    assert!(stdout.contains("\"kind\":\"Usage\""));
    assert!(stdout.contains("\"reason_code\":\"unsupported_flag\""));
}

fn run_write(package: &TestPackage) -> npa_cli::diagnostic::CommandResult {
    run_axiom_report(package, false)
}

fn run_check(package: &TestPackage) -> npa_cli::diagnostic::CommandResult {
    run_axiom_report(package, true)
}

fn run_axiom_report(package: &TestPackage, check: bool) -> npa_cli::diagnostic::CommandResult {
    run_package_axiom_report(PackageAxiomReportOptions {
        common: PackageCommonOptions {
            root: package.path().to_path_buf(),
            json: true,
        },
        check,
    })
}

fn assert_failure(result: &npa_cli::diagnostic::CommandResult, kind: DiagnosticKind, reason: &str) {
    assert_eq!(result.exit_code(), CommandExitCode::PackageFailure);
    assert_eq!(result.diagnostics.len(), 1);
    assert_eq!(result.diagnostics[0].kind, kind);
    assert_eq!(result.diagnostics[0].reason_code, reason);
}

fn build_source_free_fixture(
    label: &str,
    module_name: &str,
    include_external: bool,
    allowed_axioms: &[&str],
) -> TestPackage {
    let package = TestPackage::new(label);
    let proof_manifest = proof_manifest();
    let manifest = proof_manifest.manifest();
    let module = manifest
        .modules
        .iter()
        .find(|module| module.module.as_dotted() == module_name)
        .unwrap();
    copy_artifact(&package, module.certificate.as_str());

    let imports = if include_external {
        manifest
            .imports
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .filter(|import| module.imports.contains(&import.module))
            .cloned()
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    for import in &imports {
        copy_artifact(&package, import.certificate.as_str());
    }

    let manifest_source = fixture_manifest(
        allowed_axioms,
        &imports,
        &[manifest_module_from_package(module)],
    );
    fs::write(
        package.artifact_path(PACKAGE_MANIFEST_PATH),
        &manifest_source,
    )
    .unwrap();
    write_lock(&package, &manifest_source);
    package
}

fn manifest_module_from_package(module: &PackageModule) -> ManifestModule {
    ManifestModule {
        module: module.module.clone(),
        source: module.source.as_str().to_owned(),
        certificate: module.certificate.as_str().to_owned(),
        meta: module.meta.as_ref().map(|path| path.as_str().to_owned()),
        replay: module.replay.as_ref().map(|path| path.as_str().to_owned()),
        imports: module.imports.clone(),
        source_hash: module.expected_source_hash,
        certificate_file_hash: module.expected_certificate_file_hash,
        export_hash: module.expected_export_hash,
        axiom_report_hash: module.expected_axiom_report_hash,
        certificate_hash: module.expected_certificate_hash,
    }
}

fn fixture_manifest(
    allowed_axioms: &[&str],
    imports: &[PackageExternalImport],
    modules: &[ManifestModule],
) -> String {
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
allowed_axioms = {}

"#,
        name_array(allowed_axioms),
    );
    for import in imports {
        source.push_str(&format!(
            r#"[[imports]]
module = "{}"
package = "{}"
version = "{}"
certificate = "{}"
export_hash = "{}"
certificate_hash = "{}"

"#,
            import.module.as_dotted(),
            import.package.as_str(),
            import.version.as_str(),
            import.certificate.as_str(),
            format_package_hash(&import.export_hash),
            format_package_hash(&import.certificate_hash),
        ));
    }
    for module in modules {
        source.push_str(&format!(
            r#"[[modules]]
module = "{}"
source = "{}"
certificate = "{}"
"#,
            module.module.as_dotted(),
            module.source,
            module.certificate,
        ));
        if let Some(meta) = &module.meta {
            source.push_str(&format!("meta = \"{meta}\"\n"));
        }
        if let Some(replay) = &module.replay {
            source.push_str(&format!("replay = \"{replay}\"\n"));
        }
        source.push_str(&format!(
            r#"imports = {}
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
            module_imports_array(&module.imports),
            format_package_hash(&module.source_hash),
            format_package_hash(&module.certificate_file_hash),
            format_package_hash(&module.export_hash),
            format_package_hash(&module.axiom_report_hash),
            format_package_hash(&module.certificate_hash),
        ));
    }
    source
}

fn collect_files(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut files = BTreeMap::new();
    collect_files_inner(root, root, &mut files);
    files
}

fn collect_files_inner(root: &Path, current: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
    let mut entries = fs::read_dir(current)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            collect_files_inner(root, &path, files);
        } else {
            let relative = path
                .strip_prefix(root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            files.insert(relative, fs::read(path).unwrap());
        }
    }
}

fn name_array(names: &[&str]) -> String {
    let names = names
        .iter()
        .map(|name| format!("\"{name}\""))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{names}]")
}

fn module_imports_array(imports: &[Name]) -> String {
    let imports = imports
        .iter()
        .map(|name| format!("\"{}\"", name.as_dotted()))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{imports}]")
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

fn copy_artifact(package: &TestPackage, relative: &str) {
    let source = repo_root().join("proofs").join(relative);
    let target = package.artifact_path(relative);
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::copy(source, target).unwrap();
}

fn proof_manifest() -> npa_package::ValidatedPackageManifest {
    let source = fs::read_to_string(repo_root().join("proofs/npa-package.toml")).unwrap();
    parse_and_validate_manifest_str(&source).unwrap()
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .components()
        .collect()
}
