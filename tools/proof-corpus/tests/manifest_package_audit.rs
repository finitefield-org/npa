use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;

const LEGACY_MANIFEST_SCHEMA: &str = "npa-ai-proof-corpus-v0.1";
const PACKAGE_MANIFEST_DISPLAY_PATH: &str = "proofs/npa-package.toml";
const PROOF_CORPUS_PACKAGE: &str = "npa-proof-corpus";
const PROOF_CORPUS_VERSION: &str = "0.1.0";
const PROOF_CORPUS_LICENSE: &str = "MIT";
const PLANNED_PACKAGE_EXTERNAL_IMPORTS: &[&str] = &["Std.Logic.Eq", "Std.Nat.Basic"];
const PACKAGE_POLICY_ALLOWED_AXIOMS: &[&str] = &["Eq.rec"];
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
    assert_eq!(modules.len(), 66);

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
    assert_eq!(external_import_reference_count, 66);
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
        63
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
