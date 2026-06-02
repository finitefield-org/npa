use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;

const LEGACY_MANIFEST_SCHEMA: &str = "npa-ai-proof-corpus-v0.1";
const PACKAGE_MANIFEST_DISPLAY_PATH: &str = "proofs/npa-package.toml";
const PACKAGE_LOCK_DISPLAY_PATH: &str = "proofs/generated/package-lock.json";
const PROOF_CORPUS_PACKAGE: &str = "npa-proof-corpus";
const PROOF_CORPUS_VERSION: &str = "0.1.0";
const PROOF_CORPUS_LICENSE: &str = "Apache-2.0";
const PLANNED_PACKAGE_EXTERNAL_IMPORTS: &[&str] = &["Std.Logic.Eq", "Std.Nat.Basic"];
const PACKAGE_POLICY_ALLOWED_AXIOMS: &[&str] = &["Eq.rec"];
const PACKAGE_MODULE_HASH_FIELDS: &[&str] = &[
    "expected_source_hash",
    "expected_certificate_file_hash",
    "expected_export_hash",
    "expected_axiom_report_hash",
    "expected_certificate_hash",
];
const PACKAGE_IMPORT_HASH_FIELDS: &[&str] = &["export_hash", "certificate_hash"];
const PACKAGE_FAST_SOURCE_FREE_TEST_STACK_BYTES: usize = 64 * 1024 * 1024;
const PACKAGE_REFERENCE_SOURCE_FREE_TEST_STACK_BYTES: usize = 64 * 1024 * 1024;
const PACKAGE_FIXTURE_FORBIDDEN_FIELDS: &[&str] = &[
    "trusted_status",
    "verified_by_certificate",
    "checker_result",
    "registry_url",
    "latest",
    "generated_at",
];
const VENDORED_STD_IMPORT_ARTIFACTS: &[(&str, &str)] = &[
    (
        "Std.Logic.Eq",
        "vendor/npa-std/Std/Logic/Eq/certificate.npcert",
    ),
    (
        "Std.Nat.Basic",
        "vendor/npa-std/Std/Nat/Basic/certificate.npcert",
    ),
];

#[test]
fn package_fixture_validates_with_npa_package() {
    let source = read_to_string(corpus_root().join("npa-package.toml"));
    for forbidden in PACKAGE_FIXTURE_FORBIDDEN_FIELDS {
        assert!(
            !source.contains(forbidden),
            "{PACKAGE_MANIFEST_DISPLAY_PATH} must not contain forbidden package field or legacy trust token {forbidden}"
        );
    }

    let validated = npa_package::parse_and_validate_manifest_str(&source).unwrap_or_else(|error| {
        panic!("{PACKAGE_MANIFEST_DISPLAY_PATH} should validate with npa-package: {error:?}")
    });
    let manifest = validated.manifest();

    assert_eq!(manifest.schema, npa_package::PACKAGE_MANIFEST_SCHEMA);
    assert_eq!(manifest.package.as_str(), PROOF_CORPUS_PACKAGE);
    assert_eq!(manifest.version.as_str(), PROOF_CORPUS_VERSION);
    assert_eq!(manifest.license.as_deref(), Some(PROOF_CORPUS_LICENSE));
    assert_eq!(manifest.core_spec, npa_package::CORE_SPEC_V0_1);
    assert_eq!(manifest.kernel_profile, npa_package::KERNEL_PROFILE_V0_1);
    assert_eq!(
        manifest.certificate_format,
        npa_package::CERTIFICATE_FORMAT_CANONICAL_V0_1
    );
    assert_eq!(
        manifest.checker_profile,
        npa_package::CHECKER_PROFILE_REFERENCE_V0_1
    );
    assert!(!manifest.policy.allow_custom_axioms);
    assert_eq!(
        manifest
            .policy
            .allowed_axioms
            .iter()
            .map(|axiom| axiom.as_dotted())
            .collect::<Vec<_>>(),
        PACKAGE_POLICY_ALLOWED_AXIOMS
    );
}

#[test]
fn package_fixture_hashes_match_checked_in_artifacts() {
    let root = corpus_root();
    let package_source = read_to_string(root.join("npa-package.toml"));
    let package_manifest = toml_value(root.join("npa-package.toml"));
    let validated = npa_package::parse_and_validate_manifest_str(&package_source)
        .expect("package fixture validates before artifact hash checks");
    let manifest = validated.manifest();

    for (module_index, raw_module) in array_field(&package_manifest, "modules").iter().enumerate() {
        for field in PACKAGE_MODULE_HASH_FIELDS {
            npa_package::parse_package_hash(
                string_field(raw_module, field),
                format!("modules[{module_index}].{field}"),
            )
            .unwrap_or_else(|error| panic!("module package hash field {field} parses: {error:?}"));
        }
    }
    for (import_index, raw_import) in array_field(&package_manifest, "imports").iter().enumerate() {
        for field in PACKAGE_IMPORT_HASH_FIELDS {
            npa_package::parse_package_hash(
                string_field(raw_import, field),
                format!("imports[{import_index}].{field}"),
            )
            .unwrap_or_else(|error| panic!("import package hash field {field} parses: {error:?}"));
        }
    }

    for module in &manifest.modules {
        let module_name = module.module.as_dotted();
        let source_bytes = read(root.join(module.source.as_str()));
        assert_eq!(
            sha256(&source_bytes),
            *module.expected_source_hash.as_bytes(),
            "source file hash mismatch for {module_name}"
        );

        let certificate_bytes = read(root.join(module.certificate.as_str()));
        assert_eq!(
            sha256(&certificate_bytes),
            *module.expected_certificate_file_hash.as_bytes(),
            "certificate file hash mismatch for {module_name}"
        );

        let decoded = npa_cert::decode_module_cert(&certificate_bytes).unwrap_or_else(|error| {
            panic!("package module certificate decodes for {module_name}: {error:?}")
        });
        assert_eq!(
            decoded.header.module, module.module,
            "certificate module mismatch for {module_name}"
        );
        assert_eq!(
            decoded.hashes.export_hash,
            *module.expected_export_hash.as_bytes(),
            "certificate export hash mismatch for {module_name}"
        );
        assert_eq!(
            decoded.hashes.axiom_report_hash,
            *module.expected_axiom_report_hash.as_bytes(),
            "certificate axiom report hash mismatch for {module_name}"
        );
        assert_eq!(
            decoded.hashes.certificate_hash,
            *module.expected_certificate_hash.as_bytes(),
            "certificate canonical hash mismatch for {module_name}"
        );
    }

    for import in manifest.imports.as_deref().unwrap_or(&[]) {
        let module_name = import.module.as_dotted();
        let certificate_bytes = read(root.join(import.certificate.as_str()));
        let decoded = npa_cert::decode_module_cert(&certificate_bytes).unwrap_or_else(|error| {
            panic!("package external import certificate decodes for {module_name}: {error:?}")
        });
        assert_eq!(
            npa_cert::encode_module_cert(&decoded).unwrap_or_else(|error| panic!(
                "package external import certificate re-encodes for {module_name}: {error:?}"
            )),
            certificate_bytes,
            "external import certificate bytes must be canonical for {module_name}"
        );
        assert_eq!(
            decoded.header.module, import.module,
            "external import certificate module mismatch for {module_name}"
        );
        assert_eq!(
            decoded.hashes.export_hash,
            *import.export_hash.as_bytes(),
            "external import export hash mismatch for {module_name}"
        );
        assert_eq!(
            decoded.hashes.certificate_hash,
            *import.certificate_hash.as_bytes(),
            "external import certificate hash mismatch for {module_name}"
        );
    }
}

#[test]
fn package_lock_fixture_matches_builder_output() {
    let root = corpus_root();
    let source = read_to_string(root.join("npa-package.toml"));
    let validated = npa_package::parse_and_validate_manifest_str(&source)
        .expect("package fixture validates before lock fixture build");
    let lock = npa_package::build_package_lock_from_package_root(
        &validated,
        &root,
        npa_package::PackagePath::new("npa-package.toml"),
    )
    .unwrap_or_else(|error| panic!("package lock fixture should build: {error:?}"));
    let canonical = lock
        .canonical_json()
        .expect("package lock fixture serializes canonically");
    let checked_in = read_to_string(root.join("generated/package-lock.json"));

    assert_eq!(
        checked_in, canonical,
        "{PACKAGE_LOCK_DISPLAY_PATH} must match deterministic package lock builder output"
    );

    let parsed = npa_package::parse_package_lock_json(&checked_in)
        .unwrap_or_else(|error| panic!("{PACKAGE_LOCK_DISPLAY_PATH} should parse: {error:?}"));
    assert_eq!(parsed, lock);
    assert_eq!(parsed.schema, npa_package::PACKAGE_LOCK_SCHEMA);
    assert_eq!(parsed.package.as_str(), PROOF_CORPUS_PACKAGE);
    assert_eq!(parsed.version.as_str(), PROOF_CORPUS_VERSION);
    assert!(parsed.entries.iter().any(|entry| {
        entry.module.as_dotted() == "Proofs.Ai.Eq"
            && entry
                .imports
                .iter()
                .any(|import| import.module.as_dotted() == "Std.Logic.Eq")
    }));
}

#[test]
fn package_fast_source_free_verifies_checked_in_package_lock() {
    std::thread::Builder::new()
        .name("package_fast_source_free_verifies_checked_in_package_lock".to_owned())
        .stack_size(PACKAGE_FAST_SOURCE_FREE_TEST_STACK_BYTES)
        .spawn(package_fast_source_free_verifies_checked_in_package_lock_on_large_stack)
        .expect("package fast source-free test thread should spawn")
        .join()
        .expect("package fast source-free test thread should not panic");
}

fn package_fast_source_free_verifies_checked_in_package_lock_on_large_stack() {
    let root = corpus_root();
    let source = read_to_string(root.join("npa-package.toml"));
    let validated = npa_package::parse_and_validate_manifest_str(&source)
        .expect("package fixture validates before fast verification");
    let lock = npa_package::parse_package_lock_json(&read_to_string(
        root.join("generated/package-lock.json"),
    ))
    .expect("package lock fixture parses before fast verification");
    let certificate_buffers = lock
        .entries
        .iter()
        .map(|entry| {
            (
                entry.certificate.clone(),
                read(root.join(entry.certificate.as_str())),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let artifacts = certificate_buffers
        .iter()
        .map(|(path, bytes)| npa_api::PackageCertificateArtifact {
            path: path.clone(),
            bytes: bytes.as_slice(),
        })
        .collect::<Vec<_>>();

    let report = npa_api::verify_package_fast_source_free(&validated, &lock, artifacts)
        .expect("fast package verification should run");

    assert_eq!(report.status, npa_api::PackageVerificationStatus::Passed);
    assert!(!report.reference_checker_verdict);
    assert_eq!(
        report.verdict_source,
        npa_api::PackageVerificationVerdictSource::FastKernelCertificateVerifier
    );
    assert_eq!(report.modules.len(), lock.entries.len());
}

#[test]
fn package_reference_source_free_verifies_checked_in_package_lock() {
    std::thread::Builder::new()
        .name("package_reference_source_free_verifies_checked_in_package_lock".to_owned())
        .stack_size(PACKAGE_REFERENCE_SOURCE_FREE_TEST_STACK_BYTES)
        .spawn(package_reference_source_free_verifies_checked_in_package_lock_on_large_stack)
        .expect("package reference source-free test thread should spawn")
        .join()
        .expect("package reference source-free test thread should not panic");
}

fn package_reference_source_free_verifies_checked_in_package_lock_on_large_stack() {
    let root = corpus_root();
    let source = read_to_string(root.join("npa-package.toml"));
    let validated = npa_package::parse_and_validate_manifest_str(&source)
        .expect("package fixture validates before reference verification");
    let lock = npa_package::parse_package_lock_json(&read_to_string(
        root.join("generated/package-lock.json"),
    ))
    .expect("package lock fixture parses before reference verification");
    let certificate_buffers = lock
        .entries
        .iter()
        .map(|entry| {
            (
                entry.certificate.clone(),
                read(root.join(entry.certificate.as_str())),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let artifacts = certificate_buffers
        .iter()
        .map(|(path, bytes)| npa_api::PackageCertificateArtifact {
            path: path.clone(),
            bytes: bytes.as_slice(),
        })
        .collect::<Vec<_>>();

    let report = npa_api::verify_package_reference_source_free(&validated, &lock, artifacts)
        .expect("reference package verification should run");

    assert_eq!(report.status, npa_api::PackageVerificationStatus::Passed);
    assert!(report.reference_checker_verdict);
    assert_eq!(report.mode, npa_api::PackageVerificationMode::Reference);
    assert_eq!(
        report.verdict_source,
        npa_api::PackageVerificationVerdictSource::ReferenceChecker
    );
    assert_eq!(report.modules.len(), lock.entries.len());
}

#[test]
fn package_source_free_temp_copy_without_source_replay_or_meta_verifies_certificates() {
    let source_root = corpus_root();
    let package_root = temp_package_root("package-source-free-boundary");
    let certificate_path = "Proofs/Ai/Basic/certificate.npcert";
    let source_path = "missing/source/Proofs/Ai/Basic/source.npa";
    let meta_path = "missing/meta/Proofs/Ai/Basic/meta.json";
    let replay_path = "missing/replay/Proofs/Ai/Basic/replay.json";
    let certificate_bytes = read(source_root.join(certificate_path));
    let manifest_source = source_free_single_module_manifest(
        certificate_path,
        source_path,
        meta_path,
        replay_path,
        &certificate_bytes,
    );

    write_package_file(
        &package_root,
        "npa-package.toml",
        manifest_source.as_bytes(),
    );
    copy_package_file(&source_root, &package_root, certificate_path);
    assert!(!package_root.join(source_path).exists());
    assert!(!package_root.join(meta_path).exists());
    assert!(!package_root.join(replay_path).exists());

    let validated = npa_package::parse_and_validate_manifest_str(&manifest_source)
        .expect("source-free temp package manifest validates");
    let lock = npa_package::build_package_lock_from_package_root(
        &validated,
        &package_root,
        npa_package::PackagePath::new("npa-package.toml"),
    )
    .expect("lock builder reads only manifest and certificate bytes");
    let temp_certificate_bytes = read(package_root.join(certificate_path));
    let artifact_path = npa_package::PackagePath::new(certificate_path);
    let fast_report = npa_api::verify_package_fast_source_free(
        &validated,
        &lock,
        vec![npa_api::PackageCertificateArtifact {
            path: artifact_path.clone(),
            bytes: temp_certificate_bytes.as_slice(),
        }],
    )
    .expect("fast source-free verification succeeds without source files");
    let reference_report = npa_api::verify_package_reference_source_free(
        &validated,
        &lock,
        vec![npa_api::PackageCertificateArtifact {
            path: artifact_path,
            bytes: temp_certificate_bytes.as_slice(),
        }],
    )
    .expect("reference source-free verification succeeds without source files");

    assert_eq!(lock.entries.len(), 1);
    assert_eq!(
        fast_report.status,
        npa_api::PackageVerificationStatus::Passed
    );
    assert_eq!(
        reference_report.status,
        npa_api::PackageVerificationStatus::Passed
    );
    assert!(!package_root.join(source_path).exists());
    assert!(!package_root.join(meta_path).exists());
    assert!(!package_root.join(replay_path).exists());

    let _ = fs::remove_dir_all(package_root);
}

#[test]
fn package_manifest_parity_matches_legacy_manifest() {
    let root = corpus_root();
    let legacy_manifest = toml_value(root.join("manifest.toml"));
    let package_manifest = toml_value(root.join("npa-package.toml"));
    npa_package::parse_and_validate_manifest_str(&read_to_string(root.join("npa-package.toml")))
        .expect("package fixture validates before parity checks");

    let legacy_modules = array_field(&legacy_manifest, "proof_modules");
    let package_modules = array_field(&package_manifest, "modules");
    assert_eq!(
        package_modules.len(),
        legacy_modules.len(),
        "package fixture must contain exactly the legacy local proof modules"
    );

    let legacy_by_module = module_map(legacy_modules, "legacy proof_modules");
    let package_by_module = module_map(package_modules, "package modules");

    assert_eq!(
        package_by_module.keys().copied().collect::<Vec<_>>(),
        legacy_by_module.keys().copied().collect::<Vec<_>>(),
        "package fixture must not add or omit local modules"
    );

    let local_modules = legacy_by_module.keys().copied().collect::<BTreeSet<_>>();
    let mut external_imports_from_legacy = BTreeSet::new();

    for (module_name, legacy_module) in &legacy_by_module {
        let package_module = package_by_module
            .get(module_name)
            .unwrap_or_else(|| panic!("package module {module_name} should exist"));

        assert_eq!(
            string_field(package_module, "module"),
            string_field(legacy_module, "module"),
            "module name mismatch for {module_name}"
        );
        assert_eq!(
            string_field(package_module, "source"),
            string_field(legacy_module, "source"),
            "source path mismatch for {module_name}"
        );
        assert_eq!(
            string_field(package_module, "certificate"),
            string_field(legacy_module, "certificate"),
            "certificate path mismatch for {module_name}"
        );
        assert_eq!(
            string_field(package_module, "meta"),
            string_field(legacy_module, "meta"),
            "meta path mismatch for {module_name}"
        );
        assert_eq!(
            string_field(package_module, "replay"),
            string_field(legacy_module, "replay"),
            "replay path mismatch for {module_name}"
        );
        assert_eq!(
            string_field(package_module, "producer_profile"),
            string_field(legacy_module, "producer_profile"),
            "producer profile mismatch for {module_name}"
        );

        assert_eq!(
            string_array_field(package_module, "imports"),
            string_array_field(legacy_module, "imports"),
            "import order mismatch for {module_name}"
        );
        for import in string_array_field(legacy_module, "imports") {
            if !local_modules.contains(import) {
                external_imports_from_legacy.insert(import.to_owned());
            }
        }

        assert_eq!(
            string_field(package_module, "expected_source_hash"),
            string_field(legacy_module, "source_sha256"),
            "source hash mismatch for {module_name}"
        );
        assert_eq!(
            string_field(package_module, "expected_certificate_file_hash"),
            string_field(legacy_module, "certificate_file_sha256"),
            "certificate file hash mismatch for {module_name}"
        );
        assert_eq!(
            string_field(package_module, "expected_export_hash"),
            string_field(legacy_module, "export_hash"),
            "export hash mismatch for {module_name}"
        );
        assert_eq!(
            string_field(package_module, "expected_axiom_report_hash"),
            string_field(legacy_module, "axiom_report_hash"),
            "axiom report hash mismatch for {module_name}"
        );
        assert_eq!(
            string_field(package_module, "expected_certificate_hash"),
            string_field(legacy_module, "certificate_hash"),
            "certificate hash mismatch for {module_name}"
        );

        assert_eq!(
            optional_string_array_field(package_module, "inductives"),
            optional_string_array_field(legacy_module, "inductives"),
            "inductive summary mismatch for {module_name}"
        );
        assert_eq!(
            string_array_field(package_module, "definitions"),
            string_array_field(legacy_module, "definitions"),
            "definition summary mismatch for {module_name}"
        );
        assert_eq!(
            string_array_field(package_module, "theorems"),
            string_array_field(legacy_module, "theorems"),
            "theorem summary mismatch for {module_name}"
        );
        assert_eq!(
            string_array_field(package_module, "axioms"),
            string_array_field(legacy_module, "axioms"),
            "axiom summary mismatch for {module_name}"
        );
    }

    let package_imports = array_field(&package_manifest, "imports");
    let package_external_imports = package_imports
        .iter()
        .map(|import| string_field(import, "module").to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        package_external_imports, external_imports_from_legacy,
        "top-level package imports must exactly cover legacy non-local imports"
    );
    for import in package_imports {
        let module_name = string_field(import, "module");
        assert_ne!(
            string_field(import, "package"),
            "",
            "package import {module_name} must record package identity"
        );
        assert_ne!(
            string_field(import, "version"),
            "",
            "package import {module_name} must record version identity"
        );
        assert_ne!(
            string_field(import, "certificate"),
            "",
            "package import {module_name} must record certificate path"
        );
        assert_ne!(
            string_field(import, "export_hash"),
            "",
            "package import {module_name} must record export hash"
        );
        assert_ne!(
            string_field(import, "certificate_hash"),
            "",
            "package import {module_name} must record certificate hash"
        );
    }
}

#[test]
fn legacy_manifest_imports_and_axioms_are_package_ready() {
    let manifest = read_to_string(corpus_root().join("manifest.toml"));
    let manifest = manifest
        .parse::<Value>()
        .expect("proof corpus manifest should be valid TOML");
    assert_eq!(string_field(&manifest, "schema"), LEGACY_MANIFEST_SCHEMA);

    let modules = array_field(&manifest, "proof_modules");
    assert_eq!(modules.len(), 68);

    let mut local_modules = BTreeSet::new();
    for module in modules {
        let module_name = string_field(module, "module");
        assert!(
            local_modules.insert(module_name.to_owned()),
            "duplicate local proof module {module_name}"
        );
    }

    let planned_external_imports = PLANNED_PACKAGE_EXTERNAL_IMPORTS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let package_policy_axioms = PACKAGE_POLICY_ALLOWED_AXIOMS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();

    let mut local_imports_by_module = BTreeMap::new();
    let mut external_imports_by_module = BTreeMap::new();
    let mut discovered_external_imports = BTreeSet::new();
    let mut discovered_axioms = BTreeSet::new();
    let mut local_import_reference_count = 0usize;
    let mut external_import_reference_count = 0usize;

    for module in modules {
        let module_name = string_field(module, "module");
        let mut local_imports = Vec::new();
        let mut external_imports = Vec::new();

        for import in string_array_field(module, "imports") {
            if local_modules.contains(import) {
                local_import_reference_count += 1;
                local_imports.push(import.to_owned());
            } else if planned_external_imports.contains(import) {
                external_import_reference_count += 1;
                discovered_external_imports.insert(import.to_owned());
                external_imports.push(import.to_owned());
            } else {
                panic!(
                    "manifest import {import} from {module_name} is neither a local proof module nor a planned package external import"
                );
            }
        }

        for axiom in string_array_field(module, "axioms") {
            discovered_axioms.insert(axiom.to_owned());
        }

        local_imports_by_module.insert(module_name.to_owned(), local_imports);
        external_imports_by_module.insert(module_name.to_owned(), external_imports);
    }

    assert_eq!(local_import_reference_count, 261);
    assert_eq!(external_import_reference_count, 68);
    assert_eq!(
        discovered_external_imports,
        planned_external_imports
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<BTreeSet<_>>()
    );
    assert_eq!(
        discovered_axioms,
        package_policy_axioms
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<BTreeSet<_>>()
    );

    assert_eq!(
        modules_importing(&external_imports_by_module, "Std.Logic.Eq"),
        65
    );
    assert_eq!(
        modules_importing(&external_imports_by_module, "Std.Nat.Basic"),
        3
    );
    assert_eq!(
        modules_with_axiom(modules, "Eq.rec"),
        39,
        "the CLR-02 package policy allowlist must remain exactly Eq.rec until intentionally changed"
    );

    assert!(
        local_imports_by_module
            .values()
            .flatten()
            .all(|import| local_modules.contains(import.as_str())),
        "all local import classifications should point at manifest-local proof modules"
    );
}

#[test]
fn vendored_std_import_artifacts_are_canonical_certificates() {
    let root = corpus_root();

    for (module, certificate_path) in VENDORED_STD_IMPORT_ARTIFACTS {
        let certificate_bytes = read(root.join(certificate_path));
        let decoded =
            npa_cert::decode_module_cert(&certificate_bytes).expect("vendor certificate decodes");
        assert_eq!(decoded.header.module.as_dotted(), *module);
        assert!(
            decoded.imports.is_empty(),
            "vendored Std import certificates should be self-contained"
        );
        assert_eq!(
            npa_cert::encode_module_cert(&decoded).expect("vendor certificate re-encodes"),
            certificate_bytes,
            "vendored Std import certificate bytes should be canonical"
        );

        let verified = npa_cert::verify_module_cert(
            &certificate_bytes,
            &mut npa_cert::VerifierSession::new(),
            &npa_cert::AxiomPolicy::normal(),
        )
        .expect("vendor certificate verifies");
        assert_eq!(verified.module().as_dotted(), *module);
        assert!(verified.imports().is_empty());
        assert_eq!(decoded.hashes.export_hash, verified.export_hash());
        assert_eq!(decoded.hashes.certificate_hash, verified.certificate_hash());
    }
}

fn modules_importing(imports_by_module: &BTreeMap<String, Vec<String>>, import: &str) -> usize {
    imports_by_module
        .values()
        .filter(|imports| imports.iter().any(|candidate| candidate == import))
        .count()
}

fn modules_with_axiom(modules: &[Value], axiom: &str) -> usize {
    modules
        .iter()
        .filter(|module| string_array_field(module, "axioms").contains(&axiom))
        .count()
}

fn corpus_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("proof-corpus crate lives under tools/")
        .join("proofs")
}

fn temp_package_root(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("npa-proof-corpus-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap_or_else(|err| {
        panic!(
            "failed to create temp package root {}: {err}",
            path.display()
        )
    });
    path
}

fn read_to_string(path: PathBuf) -> String {
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn toml_value(path: PathBuf) -> Value {
    read_to_string(path)
        .parse::<Value>()
        .expect("manifest should be valid TOML")
}

fn read(path: PathBuf) -> Vec<u8> {
    fs::read(&path).unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn write_package_file(root: &Path, package_path: &str, bytes: &[u8]) {
    let path = root.join(package_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|err| panic!("failed to create {}: {err}", parent.display()));
    }
    fs::write(&path, bytes)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
}

fn copy_package_file(source_root: &Path, target_root: &Path, package_path: &str) {
    let target_path = target_root.join(package_path);
    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)
            .unwrap_or_else(|err| panic!("failed to create {}: {err}", parent.display()));
    }
    fs::copy(source_root.join(package_path), &target_path)
        .unwrap_or_else(|err| panic!("failed to copy {package_path}: {err}"));
}

fn source_free_single_module_manifest(
    certificate_path: &str,
    source_path: &str,
    meta_path: &str,
    replay_path: &str,
    certificate_bytes: &[u8],
) -> String {
    let decoded =
        npa_cert::decode_module_cert(certificate_bytes).expect("source-free certificate decodes");
    assert!(
        decoded.imports.is_empty(),
        "single-module source-free fixture should not need imports"
    );
    let certificate_file_hash = npa_package::package_file_hash(certificate_bytes);
    let export_hash = npa_package::PackageHash::from(decoded.hashes.export_hash);
    let axiom_report_hash = npa_package::PackageHash::from(decoded.hashes.axiom_report_hash);
    let certificate_hash = npa_package::PackageHash::from(decoded.hashes.certificate_hash);

    format!(
        concat!(
            "schema = \"npa.package.v0.1\"\n",
            "package = \"source-free-boundary\"\n",
            "version = \"0.1.0\"\n",
            "core_spec = \"npa.core.v0.1\"\n",
            "kernel_profile = \"npa.kernel.v0.1\"\n",
            "certificate_format = \"npa.certificate.canonical.v0.1\"\n",
            "checker_profile = \"npa.checker.reference.v0.1\"\n\n",
            "[policy]\n",
            "allow_custom_axioms = false\n",
            "allowed_axioms = []\n\n",
            "[[modules]]\n",
            "module = \"{}\"\n",
            "source = \"{}\"\n",
            "certificate = \"{}\"\n",
            "meta = \"{}\"\n",
            "replay = \"{}\"\n",
            "expected_source_hash = \"sha256:0000000000000000000000000000000000000000000000000000000000000000\"\n",
            "expected_certificate_file_hash = \"{}\"\n",
            "expected_export_hash = \"{}\"\n",
            "expected_axiom_report_hash = \"{}\"\n",
            "expected_certificate_hash = \"{}\"\n",
            "imports = []\n",
            "definitions = []\n",
            "theorems = []\n",
            "axioms = []\n"
        ),
        decoded.header.module.as_dotted(),
        source_path,
        certificate_path,
        meta_path,
        replay_path,
        npa_package::format_package_hash(&certificate_file_hash),
        npa_package::format_package_hash(&export_hash),
        npa_package::format_package_hash(&axiom_report_hash),
        npa_package::format_package_hash(&certificate_hash),
    )
}

fn sha256(bytes: &[u8]) -> npa_cert::Hash {
    let digest = Sha256::digest(bytes);
    let mut hash = [0_u8; 32];
    hash.copy_from_slice(&digest);
    hash
}

fn module_map<'a>(modules: &'a [Value], table_name: &str) -> BTreeMap<&'a str, &'a Value> {
    let mut by_module = BTreeMap::new();
    for module in modules {
        let module_name = string_field(module, "module");
        assert!(
            by_module.insert(module_name, module).is_none(),
            "duplicate module {module_name} in {table_name}"
        );
    }
    by_module
}

fn array_field<'a>(value: &'a Value, key: &str) -> &'a [Value] {
    value
        .get(key)
        .and_then(Value::as_array)
        .unwrap_or_else(|| panic!("manifest field {key} should be an array"))
}

fn string_field<'a>(value: &'a Value, key: &str) -> &'a str {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("manifest field {key} should be a string"))
}

fn string_array_field<'a>(value: &'a Value, key: &str) -> Vec<&'a str> {
    array_field(value, key)
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .unwrap_or_else(|| panic!("manifest field {key} should contain only strings"))
        })
        .collect()
}

fn optional_string_array_field<'a>(value: &'a Value, key: &str) -> Option<Vec<&'a str>> {
    value.get(key).map(|_| string_array_field(value, key))
}
