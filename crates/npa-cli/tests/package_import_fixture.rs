use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use npa_cli::args::{PackageChecker, PackageCommonOptions, PackageVerifyCertsOptions};
use npa_cli::diagnostic::{CommandExitCode, DiagnosticKind};
use npa_cli::package::PACKAGE_MANIFEST_PATH;
use npa_cli::package_hashes::run_package_check_hashes;
use npa_cli::package_verify::run_package_verify_certs;
use npa_package::{
    package_file_hash, parse_and_validate_manifest_str, parse_package_publish_plan_json,
    PackageCheckerMode, PackageDownstreamImportModule, PackageId, PackageVersion,
};

const DOWNSTREAM_FIXTURE_ROOT: &str = "fixtures/npa-mathlib-seed-downstream";
const SEED_RELEASE_ROOT: &str = "fixtures/npa-mathlib-seed";
const SEED_PUBLISH_PLAN: &str = "generated/publish-plan.json";
const SEED_PACKAGE: &str = "npa-mathlib-seed";
const SEED_VERSION: &str = "0.1.0";
const SEED_MODULE: &str = "Proofs.Ai.Basic";
const VENDORED_SEED_ROOT: &str = "vendor/npa-mathlib-seed";
const ZERO_HASH: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";

static NEXT_TEMP_DIR: AtomicUsize = AtomicUsize::new(0);

struct TestFixture {
    path: PathBuf,
}

impl TestFixture {
    fn new(label: &str) -> Self {
        let index = NEXT_TEMP_DIR.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "npa-cli-package-import-fixture-{}-{label}-{index}",
            std::process::id()
        ));
        if path.exists() {
            fs::remove_dir_all(&path).unwrap();
        }
        copy_dir(&repo_root().join(DOWNSTREAM_FIXTURE_ROOT), &path);
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn artifact_path(&self, relative: &str) -> PathBuf {
        self.path.join(relative)
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[derive(Clone, Debug)]
struct SeedReleaseImport {
    module: PackageDownstreamImportModule,
    certificate_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SeedImportError {
    ArtifactFileHash,
    FixtureCertificateHash,
    FixtureExportHash,
    FixturePackageName,
    FixturePackageVersion,
    MissingExport,
    PackageName,
    PackageVersion,
    ReferenceSummary,
}

#[test]
fn package_import_fixture_accepts_seed_release_artifacts_source_free() {
    let seed = load_seed_basic_release_import(|_| {}).unwrap();
    let fixture = materialize_downstream_fixture("valid", &seed).unwrap();

    assert_source_free_seed_vendor(&fixture);

    let hashes = run_hashes(&fixture);
    assert_eq!(hashes.exit_code(), CommandExitCode::Success);
    assert!(hashes.diagnostics.is_empty());

    let verify = run_verify(&fixture);
    assert_eq!(verify.exit_code(), CommandExitCode::Success);
    assert!(verify.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == DiagnosticKind::ReferenceVerifier
            && diagnostic.reason_code == "module_verified"
            && diagnostic.module.as_deref() == Some(SEED_MODULE)
            && diagnostic.path.as_deref()
                == Some("vendor/npa-mathlib-seed/Proofs/Ai/Basic/certificate.npcert")
    }));
    assert!(verify.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == DiagnosticKind::ReferenceVerifier
            && diagnostic.reason_code == "module_verified"
            && diagnostic.module.as_deref() == Some("Downstream.SeedBasic")
    }));

    let rendered = verify.render_json();
    assert!(!rendered.contains("source.npa"));
    assert!(!rendered.contains("replay.json"));
    assert!(!rendered.contains("meta.json"));
    assert!(!rendered.contains("theorem-index.json"));
    assert!(!rendered.contains("registry"));
}

#[test]
fn package_import_fixture_rejects_corrupted_seed_release_metadata() {
    let artifact_hash = load_seed_basic_release_import(|module| {
        module.certificate_file_hash = package_file_hash(b"corrupt seed artifact hash");
    })
    .unwrap_err();
    assert_eq!(artifact_hash, SeedImportError::ArtifactFileHash);

    let package_name = load_seed_basic_release_import(|module| {
        module.package = PackageId::new("npa-mathlib-seed-corrupt");
    })
    .unwrap_err();
    assert_eq!(package_name, SeedImportError::PackageName);

    let package_version = load_seed_basic_release_import(|module| {
        module.version = PackageVersion::new("9.9.9");
    })
    .unwrap_err();
    assert_eq!(package_version, SeedImportError::PackageVersion);
}

#[test]
fn package_import_fixture_rejects_corrupted_manifest_hash_pins() {
    let seed = load_seed_basic_release_import(|_| {}).unwrap();

    let export_fixture = materialize_downstream_fixture("bad-export", &seed).unwrap();
    replace_manifest_line_prefix(&export_fixture, "export_hash = \"", ZERO_HASH);
    assert_hash_failure(
        &run_hashes(&export_fixture),
        "export_hash_mismatch",
        Some("imports[0].export_hash"),
        Some("export_hash"),
    );

    let certificate_fixture = materialize_downstream_fixture("bad-certificate", &seed).unwrap();
    replace_manifest_line_prefix(&certificate_fixture, "certificate_hash = \"", ZERO_HASH);
    assert_hash_failure(
        &run_hashes(&certificate_fixture),
        "certificate_hash_mismatch",
        Some("imports[0].certificate_hash"),
        Some("certificate_hash"),
    );
}

fn load_seed_basic_release_import<F>(mutate: F) -> Result<SeedReleaseImport, SeedImportError>
where
    F: FnOnce(&mut PackageDownstreamImportModule),
{
    let publish_plan_path = repo_root().join(SEED_RELEASE_ROOT).join(SEED_PUBLISH_PLAN);
    let publish_plan_source = fs::read_to_string(publish_plan_path).unwrap();
    let publish_plan = parse_package_publish_plan_json(&publish_plan_source).unwrap();
    assert_eq!(publish_plan.package.as_str(), SEED_PACKAGE);
    assert_eq!(publish_plan.version.as_str(), SEED_VERSION);
    assert_eq!(
        publish_plan.downstream_import_bundle.package.as_str(),
        SEED_PACKAGE
    );
    assert_eq!(
        publish_plan.downstream_import_bundle.version.as_str(),
        SEED_VERSION
    );

    let mut module = publish_plan
        .downstream_import_bundle
        .modules
        .iter()
        .find(|module| module.module.as_dotted() == SEED_MODULE)
        .cloned()
        .unwrap();
    mutate(&mut module);

    if module.package.as_str() != publish_plan.downstream_import_bundle.package.as_str()
        || module.package.as_str() != SEED_PACKAGE
    {
        return Err(SeedImportError::PackageName);
    }
    if module.version.as_str() != publish_plan.downstream_import_bundle.version.as_str()
        || module.version.as_str() != SEED_VERSION
    {
        return Err(SeedImportError::PackageVersion);
    }
    if !module
        .exported_declarations
        .iter()
        .any(|declaration| declaration.as_dotted() == "id")
    {
        return Err(SeedImportError::MissingExport);
    }
    if !module.checker_summaries.iter().any(|summary| {
        summary.module == module.module
            && summary.mode == PackageCheckerMode::Reference
            && summary.checker == "npa-checker-ref"
            && summary.status == "passed"
            && summary.export_hash == module.export_hash
            && summary.certificate_hash == module.certificate_hash
    }) {
        return Err(SeedImportError::ReferenceSummary);
    }

    let certificate_path = repo_root()
        .join(SEED_RELEASE_ROOT)
        .join(module.certificate.as_str());
    let certificate_bytes = fs::read(certificate_path).unwrap();
    if package_file_hash(&certificate_bytes) != module.certificate_file_hash {
        return Err(SeedImportError::ArtifactFileHash);
    }

    Ok(SeedReleaseImport {
        module,
        certificate_bytes,
    })
}

fn materialize_downstream_fixture(
    label: &str,
    seed: &SeedReleaseImport,
) -> Result<TestFixture, SeedImportError> {
    let fixture = TestFixture::new(label);
    assert_fixture_manifest_matches_seed_bundle(&fixture, &seed.module)?;

    let target = fixture.artifact_path(&format!(
        "{VENDORED_SEED_ROOT}/{}",
        seed.module.certificate.as_str()
    ));
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, &seed.certificate_bytes).unwrap();
    Ok(fixture)
}

fn assert_fixture_manifest_matches_seed_bundle(
    fixture: &TestFixture,
    seed_module: &PackageDownstreamImportModule,
) -> Result<(), SeedImportError> {
    let manifest_source = fs::read_to_string(fixture.artifact_path(PACKAGE_MANIFEST_PATH)).unwrap();
    assert!(!manifest_source.contains("registry"));
    assert!(!manifest_source.contains("latest"));

    let validated = parse_and_validate_manifest_str(&manifest_source).unwrap();
    let import = validated
        .manifest()
        .imports
        .as_ref()
        .unwrap()
        .iter()
        .find(|import| import.module == seed_module.module)
        .unwrap();

    if import.package != seed_module.package {
        return Err(SeedImportError::FixturePackageName);
    }
    if import.version != seed_module.version {
        return Err(SeedImportError::FixturePackageVersion);
    }
    if import.export_hash != seed_module.export_hash {
        return Err(SeedImportError::FixtureExportHash);
    }
    if import.certificate_hash != seed_module.certificate_hash {
        return Err(SeedImportError::FixtureCertificateHash);
    }
    assert_eq!(
        import.certificate.as_str(),
        format!("{VENDORED_SEED_ROOT}/{}", seed_module.certificate.as_str())
    );
    Ok(())
}

fn assert_source_free_seed_vendor(fixture: &TestFixture) {
    let vendor_root = fixture.artifact_path(VENDORED_SEED_ROOT);
    assert!(vendor_root.exists());
    let forbidden_suffixes = [
        "source.npa",
        "replay.json",
        "meta.json",
        "theorem-index.json",
        "registry.json",
    ];
    for path in collect_files(&vendor_root) {
        let display = path.strip_prefix(&fixture.path).unwrap().to_string_lossy();
        assert!(
            !forbidden_suffixes
                .iter()
                .any(|suffix| display.ends_with(suffix)),
            "{display}"
        );
    }
}

fn run_hashes(fixture: &TestFixture) -> npa_cli::diagnostic::CommandResult {
    run_package_check_hashes(PackageCommonOptions {
        root: fixture.path().to_path_buf(),
        json: true,
    })
}

fn run_verify(fixture: &TestFixture) -> npa_cli::diagnostic::CommandResult {
    run_package_verify_certs(PackageVerifyCertsOptions {
        common: PackageCommonOptions {
            root: fixture.path().to_path_buf(),
            json: true,
        },
        checker: PackageChecker::Reference,
        external: None,
    })
}

fn assert_hash_failure(
    result: &npa_cli::diagnostic::CommandResult,
    reason: &str,
    path: Option<&str>,
    field: Option<&str>,
) {
    assert_eq!(result.exit_code(), CommandExitCode::PackageFailure);
    assert_eq!(result.diagnostics.len(), 1);
    let diagnostic = &result.diagnostics[0];
    assert_eq!(diagnostic.kind, DiagnosticKind::HashMismatch);
    assert_eq!(diagnostic.reason_code, reason);
    if let Some(path) = path {
        assert_eq!(diagnostic.path.as_deref(), Some(path));
    }
    if let Some(field) = field {
        assert_eq!(diagnostic.field.as_deref(), Some(field));
    }
    assert!(!result.render_json().contains("/tmp/"));
}

fn replace_manifest_line_prefix(fixture: &TestFixture, prefix: &str, replacement_hash: &str) {
    let manifest_path = fixture.artifact_path(PACKAGE_MANIFEST_PATH);
    let source = fs::read_to_string(&manifest_path).unwrap();
    let line = source
        .lines()
        .find(|line| line.starts_with(prefix))
        .unwrap();
    let replacement = format!("{prefix}{replacement_hash}\"");
    fs::write(manifest_path, source.replacen(line, &replacement, 1)).unwrap();
}

fn copy_dir(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let target_path = target.join(entry.file_name());
        let file_type = entry.file_type().unwrap();
        if file_type.is_dir() {
            copy_dir(&path, &target_path);
        } else if file_type.is_file() {
            fs::copy(path, target_path).unwrap();
        }
    }
}

fn collect_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_files_into(root, &mut files);
    files.sort();
    files
}

fn collect_files_into(root: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let file_type = entry.file_type().unwrap();
        if file_type.is_dir() {
            collect_files_into(&path, files);
        } else if file_type.is_file() {
            files.push(path);
        }
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .components()
        .collect()
}
