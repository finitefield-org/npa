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

fn read(path: PathBuf) -> Vec<u8> {
    fs::read(&path).unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
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
