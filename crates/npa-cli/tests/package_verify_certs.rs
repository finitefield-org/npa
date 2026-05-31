use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use npa_cert::Name;
use npa_cli::args::{PackageChecker, PackageCommonOptions, PackageVerifyCertsOptions};
use npa_cli::diagnostic::{CommandExitCode, DiagnosticKind, DiagnosticSeverity};
use npa_cli::package::PACKAGE_MANIFEST_PATH;
use npa_cli::package_verify::run_package_verify_certs;
use npa_package::{
    build_package_lock_from_package_root, format_package_hash, package_file_hash,
    parse_and_validate_manifest_str, PackageExternalImport, PackageHash, PackageModule,
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
            "npa-cli-package-verify-certs-{}-{label}-{index}",
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
fn package_verify_certs_reference_succeeds_without_source_replay_or_meta() {
    let package = build_source_free_fixture(
        "reference-source-free",
        "Proofs.Ai.Basic",
        false,
        &["Eq.rec"],
    );
    assert!(!package.artifact_path("Proofs/Ai/Basic/source.npa").exists());
    assert!(!package
        .artifact_path("Proofs/Ai/Basic/replay.json")
        .exists());
    assert!(!package.artifact_path("Proofs/Ai/Basic/meta.json").exists());

    let result = run_verify(&package, PackageChecker::Reference);

    assert_eq!(result.exit_code(), CommandExitCode::Success);
    assert_eq!(result.diagnostics.len(), 2);
    assert_info(
        &result.diagnostics[0],
        DiagnosticKind::ReferenceVerifier,
        "package_verified",
        Some("npa-checker-ref"),
    );
    assert!(result.diagnostics[0]
        .actual_value
        .as_deref()
        .unwrap()
        .contains("reference_checker_verdict=true"));
    assert_info(
        &result.diagnostics[1],
        DiagnosticKind::ReferenceVerifier,
        "module_verified",
        Some("npa-checker-ref"),
    );
    assert_eq!(
        result.diagnostics[1].module.as_deref(),
        Some("Proofs.Ai.Basic")
    );
    assert_eq!(
        result.diagnostics[1].path.as_deref(),
        Some("Proofs/Ai/Basic/certificate.npcert")
    );
    assert!(!result.render_json().contains("/tmp/"));
}

#[test]
fn package_verify_certs_fast_succeeds_and_is_labeled_fast_kernel() {
    let package =
        build_source_free_fixture("fast-source-free", "Proofs.Ai.Basic", false, &["Eq.rec"]);

    let result = run_verify(&package, PackageChecker::Fast);

    assert_eq!(result.exit_code(), CommandExitCode::Success);
    assert_eq!(result.diagnostics.len(), 2);
    assert_info(
        &result.diagnostics[0],
        DiagnosticKind::FastVerifier,
        "package_verified",
        Some("fast-kernel-certificate-verifier"),
    );
    let aggregate = result.diagnostics[0].actual_value.as_deref().unwrap();
    assert!(aggregate.contains("mode=fast-kernel"));
    assert!(aggregate.contains("reference_checker_verdict=false"));
    assert!(result
        .diagnostics
        .iter()
        .all(|diagnostic| diagnostic.checker.as_deref() != Some("npa-checker-ref")));
}

#[test]
fn package_verify_certs_fast_cli_succeeds_json() {
    let package = build_source_free_fixture("cli-fast", "Proofs.Ai.Basic", false, &["Eq.rec"]);

    let output = Command::new(env!("CARGO_BIN_EXE_npa"))
        .args(["package", "verify-certs", "--root"])
        .arg(package.path())
        .args(["--checker", "fast", "--json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"command\":\"package verify-certs\""));
    assert!(stdout.contains("\"status\":\"passed\""));
    assert!(stdout.contains("\"kind\":\"FastVerifier\""));
    assert!(stdout.contains("\"reason_code\":\"package_verified\""));
    assert!(stdout.contains("\"checker\":\"fast-kernel-certificate-verifier\""));
    assert!(!stdout.contains(&package.path().to_string_lossy().to_string()));
}

#[test]
fn package_verify_certs_rejects_stale_package_lock_before_checker_status() {
    let package = build_source_free_fixture("stale-lock", "Proofs.Ai.Basic", false, &["Eq.rec"]);
    let lock_path = package.artifact_path(LOCK_PATH);
    let mut lock_source = fs::read_to_string(&lock_path).unwrap();
    lock_source.push('\n');
    fs::write(lock_path, lock_source).unwrap();

    let result = run_verify(&package, PackageChecker::Reference);

    assert_failure(
        &result,
        DiagnosticKind::HashMismatch,
        "package_lock_stale",
        Some(LOCK_PATH),
        None,
    );
    assert!(!result.render_json().contains("module_verified"));
    assert!(!result.render_json().contains("package_verified"));
}

#[test]
fn package_verify_certs_rejects_stale_certificate_hash_before_checker_status() {
    let package =
        build_source_free_fixture("stale-certificate", "Proofs.Ai.Basic", false, &["Eq.rec"]);
    fs::write(
        package.artifact_path("Proofs/Ai/Basic/certificate.npcert"),
        fs::read(repo_root().join("proofs/Proofs/Ai/Prop/certificate.npcert")).unwrap(),
    )
    .unwrap();

    let result = run_verify(&package, PackageChecker::Reference);

    assert_failure(
        &result,
        DiagnosticKind::HashMismatch,
        "certificate_file_hash_mismatch",
        Some("modules[0].expected_certificate_file_hash"),
        Some("expected_certificate_file_hash"),
    );
    assert!(!result.render_json().contains("module_verified"));
    assert!(!result.render_json().contains("package_verified"));
}

#[test]
fn package_verify_certs_reference_preserves_checker_rejection_diagnostic() {
    let package =
        build_source_free_fixture("reference-rejection", "Proofs.Ai.Eq", true, &["Eq.rec"]);
    let certificate_path = package.artifact_path("Proofs/Ai/Eq/certificate.npcert");
    tamper_certificate_core_spec_without_rehash(&certificate_path);
    refresh_expected_certificate_file_hash(&package, &certificate_path);
    let manifest_source = fs::read_to_string(package.artifact_path(PACKAGE_MANIFEST_PATH)).unwrap();
    write_lock(&package, &manifest_source);

    let result = run_verify(&package, PackageChecker::Reference);

    assert_eq!(result.exit_code(), CommandExitCode::PackageFailure);
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.kind == DiagnosticKind::ReferenceVerifier)
        .expect("reference checker rejection is reported");
    assert_eq!(diagnostic.kind, DiagnosticKind::ReferenceVerifier);
    assert_eq!(diagnostic.reason_code, "reference_checker_rejected");
    assert_eq!(diagnostic.checker.as_deref(), Some("npa-checker-ref"));
    assert_eq!(diagnostic.field.as_deref(), Some("certificate"));
    assert_eq!(diagnostic.module.as_deref(), Some("Proofs.Ai.Eq"));
    assert!(diagnostic
        .actual_value
        .as_deref()
        .unwrap()
        .contains("CoreSpecMismatch"));
    assert!(!result.render_json().contains("module_verified"));
    assert!(!result.render_json().contains("package_verified"));
}

fn run_verify(
    package: &TestPackage,
    checker: PackageChecker,
) -> npa_cli::diagnostic::CommandResult {
    run_package_verify_certs(PackageVerifyCertsOptions {
        common: PackageCommonOptions {
            root: package.path().to_path_buf(),
            json: true,
        },
        checker,
    })
}

fn assert_info(
    diagnostic: &npa_cli::diagnostic::CommandDiagnostic,
    kind: DiagnosticKind,
    reason: &str,
    checker: Option<&str>,
) {
    assert_eq!(diagnostic.kind, kind);
    assert_eq!(diagnostic.reason_code, reason);
    assert_eq!(diagnostic.severity, DiagnosticSeverity::Info);
    assert_eq!(diagnostic.checker.as_deref(), checker);
}

fn assert_failure(
    result: &npa_cli::diagnostic::CommandResult,
    kind: DiagnosticKind,
    reason: &str,
    path: Option<&str>,
    field: Option<&str>,
) {
    assert_eq!(result.exit_code(), CommandExitCode::PackageFailure);
    assert_eq!(result.diagnostics.len(), 1);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.kind, kind);
    assert_eq!(diagnostic.reason_code, reason);
    if let Some(path) = path {
        assert_eq!(diagnostic.path.as_deref(), Some(path));
    }
    if let Some(field) = field {
        assert_eq!(diagnostic.field.as_deref(), Some(field));
    }
    assert!(diagnostic.checker.is_none());
    assert!(!result.render_json().contains("/tmp/"));
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

fn tamper_certificate_core_spec_without_rehash(path: &Path) {
    let mut cert = npa_cert::decode_module_cert(&fs::read(path).unwrap()).unwrap();
    cert.header.core_spec.push_str(".tampered");
    fs::write(path, npa_cert::encode_module_cert(&cert).unwrap()).unwrap();
}

fn refresh_expected_certificate_file_hash(package: &TestPackage, certificate: &Path) {
    let file_hash = package_file_hash(&fs::read(certificate).unwrap());
    let path = package.artifact_path(PACKAGE_MANIFEST_PATH);
    let source = fs::read_to_string(&path).unwrap();
    let line = source
        .lines()
        .find(|line| line.starts_with("expected_certificate_file_hash = \""))
        .unwrap();
    let replacement = format!(
        "expected_certificate_file_hash = \"{}\"",
        format_package_hash(&file_hash)
    );
    fs::write(path, source.replacen(line, &replacement, 1)).unwrap();
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
