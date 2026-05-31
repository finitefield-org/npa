use npa_package::{
    parse_and_validate_manifest_str, parse_manifest_str, PackageManifestError,
    PackageManifestErrorKind, PackageManifestErrorReason, PACKAGE_MANIFEST_SCHEMA,
};

const ZERO_HASH: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";

fn valid_manifest() -> String {
    format!(
        r#"schema = "{PACKAGE_MANIFEST_SCHEMA}"
package = "npa-proof-corpus"
version = "0.1.0"
core_spec = "npa.core.v0.1"
kernel_profile = "npa.kernel.v0.1"
certificate_format = "npa.certificate.canonical.v0.1"
checker_profile = "npa.checker.reference.v0.1"
license = "MIT"
repository = "https://example.invalid/npa-proof-corpus"
description = "proof corpus fixture"

[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]

[[imports]]
module = "Std.Logic.Eq"
package = "npa-std"
version = "0.1.0"
certificate = "vendor/npa-std/Std/Logic/Eq/certificate.npcert"
export_hash = "{ZERO_HASH}"
certificate_hash = "{ZERO_HASH}"

[[modules]]
module = "Proofs.Ai.Basic"
source = "Proofs/Ai/Basic/source.npa"
certificate = "Proofs/Ai/Basic/certificate.npcert"
imports = ["Std.Logic.Eq"]
expected_source_hash = "{ZERO_HASH}"
expected_certificate_file_hash = "{ZERO_HASH}"
expected_export_hash = "{ZERO_HASH}"
expected_axiom_report_hash = "{ZERO_HASH}"
expected_certificate_hash = "{ZERO_HASH}"
meta = "Proofs/Ai/Basic/meta.json"
replay = "Proofs/Ai/Basic/replay.json"
producer_profile = "human-surface-explicit-term"
inductives = []
definitions = []
theorems = ["id"]
axioms = []
tags = ["basic"]
"#
    )
}

fn assert_manifest_error(
    error: &PackageManifestError,
    kind: PackageManifestErrorKind,
    reason: PackageManifestErrorReason,
    path: &str,
    field: Option<&str>,
) {
    assert_eq!(error.kind, kind);
    assert_eq!(error.reason_code, reason);
    assert_eq!(error.reason_code.as_str(), reason.as_str());
    assert_eq!(error.path, path);
    assert_eq!(error.field.as_deref(), field);
}

fn assert_manifest_error_values(
    error: &PackageManifestError,
    expected: Option<&str>,
    actual: Option<&str>,
) {
    assert_eq!(error.expected_value.as_deref(), expected);
    assert_eq!(error.actual_value.as_deref(), actual);
}

fn validation_error(source: String) -> PackageManifestError {
    parse_and_validate_manifest_str(&source).unwrap_err()
}

fn manifest_with_root_entries(root_entries: &str, policy: &str) -> String {
    format!(
        r#"schema = "{PACKAGE_MANIFEST_SCHEMA}"
package = "npa-proof-corpus"
version = "0.1.0"
core_spec = "npa.core.v0.1"
kernel_profile = "npa.kernel.v0.1"
certificate_format = "npa.certificate.canonical.v0.1"
checker_profile = "npa.checker.reference.v0.1"

{root_entries}

{policy}
"#
    )
}

#[test]
fn package_manifest_parse_accepts_valid_closed_manifest() {
    let manifest = parse_manifest_str(&valid_manifest()).unwrap();

    assert_eq!(manifest.schema, PACKAGE_MANIFEST_SCHEMA);
    assert_eq!(manifest.package.as_str(), "npa-proof-corpus");
    assert_eq!(manifest.version.as_str(), "0.1.0");
    assert!(!manifest.policy.allow_custom_axioms);
    assert_eq!(manifest.policy.allowed_axioms[0].as_dotted(), "Eq.rec");
    assert_eq!(manifest.imports.as_ref().unwrap().len(), 1);
    assert_eq!(manifest.modules.len(), 1);
    assert_eq!(manifest.modules[0].module.as_dotted(), "Proofs.Ai.Basic");
    assert_eq!(
        manifest.modules[0].expected_export_hash.as_bytes(),
        &[0; 32]
    );
}

#[test]
fn package_manifest_parse_rejects_invalid_toml_before_schema_validation() {
    let error = parse_manifest_str(
        r#"schema = "npa.package.v0.1"
["#,
    )
    .unwrap_err();

    assert_manifest_error(
        &error,
        PackageManifestErrorKind::TomlSyntax,
        PackageManifestErrorReason::InvalidToml,
        "$",
        None,
    );
}

#[test]
fn package_manifest_parse_rejects_duplicate_key_as_schema_error() {
    let error = parse_manifest_str(
        r#"schema = "npa.package.v0.1"
schema = "npa.package.v0.1"
"#,
    )
    .unwrap_err();

    assert_manifest_error(
        &error,
        PackageManifestErrorKind::Schema,
        PackageManifestErrorReason::DuplicateField,
        "$",
        None,
    );
}

#[test]
fn package_manifest_closed_objects_reports_missing_required_field_path() {
    let source = valid_manifest().replace(
        r#"checker_profile = "npa.checker.reference.v0.1"
"#,
        "",
    );

    let error = parse_manifest_str(&source).unwrap_err();

    assert_manifest_error(
        &error,
        PackageManifestErrorKind::Schema,
        PackageManifestErrorReason::MissingField,
        "$",
        Some("checker_profile"),
    );
}

#[test]
fn package_manifest_closed_objects_rejects_unknown_top_level_field() {
    let source = valid_manifest().replacen(
        r#"schema = "npa.package.v0.1"
"#,
        r#"schema = "npa.package.v0.1"
trusted_status = "verified"
"#,
        1,
    );

    let error = parse_manifest_str(&source).unwrap_err();

    assert_manifest_error(
        &error,
        PackageManifestErrorKind::Schema,
        PackageManifestErrorReason::UnknownField,
        "$",
        Some("trusted_status"),
    );
}

#[test]
fn package_manifest_closed_objects_rejects_unknown_policy_field() {
    let source =
        valid_manifest().replacen("[policy]\n", "[policy]\nunknown_policy_field = true\n", 1);

    let error = parse_manifest_str(&source).unwrap_err();

    assert_manifest_error(
        &error,
        PackageManifestErrorKind::Schema,
        PackageManifestErrorReason::UnknownField,
        "policy",
        Some("unknown_policy_field"),
    );
}

#[test]
fn package_manifest_closed_objects_rejects_unknown_import_field() {
    let source = valid_manifest().replacen("[[imports]]\n", "[[imports]]\nlatest = true\n", 1);

    let error = parse_manifest_str(&source).unwrap_err();

    assert_manifest_error(
        &error,
        PackageManifestErrorKind::Schema,
        PackageManifestErrorReason::UnknownField,
        "imports[0]",
        Some("latest"),
    );
}

#[test]
fn package_manifest_closed_objects_rejects_unknown_module_field() {
    let source = valid_manifest().replacen(
        "[[modules]]\n",
        "[[modules]]\nchecker_result = \"accepted\"\n",
        1,
    );

    let error = parse_manifest_str(&source).unwrap_err();

    assert_manifest_error(
        &error,
        PackageManifestErrorKind::Schema,
        PackageManifestErrorReason::UnknownField,
        "modules[0]",
        Some("checker_result"),
    );
}

#[test]
fn package_manifest_closed_objects_rejects_wrong_field_type() {
    let source = valid_manifest().replace(
        "allow_custom_axioms = false",
        r#"allow_custom_axioms = "false""#,
    );

    let error = parse_manifest_str(&source).unwrap_err();

    assert_manifest_error(
        &error,
        PackageManifestErrorKind::Schema,
        PackageManifestErrorReason::WrongType,
        "policy.allow_custom_axioms",
        Some("allow_custom_axioms"),
    );
    assert_eq!(error.expected_value.as_deref(), Some("bool"));
    assert_eq!(error.actual_value.as_deref(), Some("string"));
}

#[test]
fn package_manifest_closed_objects_rejects_wrong_object_types() {
    let policy_error = parse_manifest_str(&manifest_with_root_entries(
        r#"policy = "strict"
modules = []"#,
        "",
    ))
    .unwrap_err();
    assert_manifest_error(
        &policy_error,
        PackageManifestErrorKind::Schema,
        PackageManifestErrorReason::WrongType,
        "policy",
        Some("policy"),
    );
    assert_eq!(policy_error.expected_value.as_deref(), Some("table"));
    assert_eq!(policy_error.actual_value.as_deref(), Some("string"));

    let import_error = parse_manifest_str(&manifest_with_root_entries(
        r#"imports = "none"
modules = []"#,
        r#"[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]"#,
    ))
    .unwrap_err();
    assert_manifest_error(
        &import_error,
        PackageManifestErrorKind::Schema,
        PackageManifestErrorReason::WrongType,
        "imports",
        Some("imports"),
    );
    assert_eq!(import_error.expected_value.as_deref(), Some("array"));
    assert_eq!(import_error.actual_value.as_deref(), Some("string"));

    let module_error = parse_manifest_str(&manifest_with_root_entries(
        r#"modules = ["Proofs.Ai.Basic"]"#,
        r#"[policy]
allow_custom_axioms = false
allowed_axioms = ["Eq.rec"]"#,
    ))
    .unwrap_err();
    assert_manifest_error(
        &module_error,
        PackageManifestErrorKind::Schema,
        PackageManifestErrorReason::WrongType,
        "modules[0]",
        None,
    );
    assert_eq!(module_error.expected_value.as_deref(), Some("table"));
    assert_eq!(module_error.actual_value.as_deref(), Some("string"));
}

#[test]
fn package_manifest_closed_objects_rejects_wrong_array_item_type() {
    let source = valid_manifest().replace("imports = [\"Std.Logic.Eq\"]", "imports = [1]");

    let error = parse_manifest_str(&source).unwrap_err();

    assert_manifest_error(
        &error,
        PackageManifestErrorKind::Schema,
        PackageManifestErrorReason::WrongType,
        "modules[0].imports[0]",
        None,
    );
    assert_eq!(error.expected_value.as_deref(), Some("string"));
    assert_eq!(error.actual_value.as_deref(), Some("integer"));
}

#[test]
fn package_manifest_scalar_domains_accepts_valid_manifest() {
    let manifest = parse_and_validate_manifest_str(&valid_manifest()).unwrap();

    assert_eq!(manifest.manifest().package.as_str(), "npa-proof-corpus");
    assert_eq!(
        manifest.manifest().modules[0].module.as_dotted(),
        "Proofs.Ai.Basic"
    );
}

#[test]
fn package_manifest_scalar_domains_rejects_exact_schema_and_profile_mismatches() {
    let schema_error = validation_error(valid_manifest().replace(
        r#"schema = "npa.package.v0.1""#,
        r#"schema = "npa.package.v0.2""#,
    ));
    assert_manifest_error(
        &schema_error,
        PackageManifestErrorKind::UnsupportedVersion,
        PackageManifestErrorReason::UnsupportedSchema,
        "schema",
        Some("schema"),
    );
    assert_manifest_error_values(
        &schema_error,
        Some("npa.package.v0.1"),
        Some("npa.package.v0.2"),
    );

    let profile_error = validation_error(valid_manifest().replace(
        r#"kernel_profile = "npa.kernel.v0.1""#,
        r#"kernel_profile = "npa.kernel.v0.2""#,
    ));
    assert_manifest_error(
        &profile_error,
        PackageManifestErrorKind::Domain,
        PackageManifestErrorReason::InvalidProfile,
        "kernel_profile",
        Some("kernel_profile"),
    );
    assert_manifest_error_values(
        &profile_error,
        Some("npa.kernel.v0.1"),
        Some("npa.kernel.v0.2"),
    );
}

#[test]
fn package_manifest_scalar_domains_rejects_package_id_and_version_grammar() {
    let package_error = validation_error(valid_manifest().replace(
        r#"package = "npa-proof-corpus""#,
        r#"package = "Npa-proof-corpus""#,
    ));
    assert_manifest_error(
        &package_error,
        PackageManifestErrorKind::Domain,
        PackageManifestErrorReason::InvalidPackageId,
        "package",
        None,
    );

    let version_error =
        validation_error(valid_manifest().replace(r#"version = "0.1.0""#, r#"version = "0.01.0""#));
    assert_manifest_error(
        &version_error,
        PackageManifestErrorKind::Domain,
        PackageManifestErrorReason::InvalidVersion,
        "version",
        None,
    );

    let prerelease_error = validation_error(
        valid_manifest().replace(r#"version = "0.1.0""#, r#"version = "0.1.0-alpha""#),
    );
    assert_manifest_error(
        &prerelease_error,
        PackageManifestErrorKind::Domain,
        PackageManifestErrorReason::InvalidVersion,
        "version",
        None,
    );
}

#[test]
fn package_manifest_scalar_domains_aligns_names_with_npa_cert_canonical_names() {
    let module_error = validation_error(valid_manifest().replace(
        r#"module = "Proofs.Ai.Basic""#,
        r#"module = "Proofs..Basic""#,
    ));
    assert_manifest_error(
        &module_error,
        PackageManifestErrorKind::Domain,
        PackageManifestErrorReason::InvalidModuleName,
        "modules[0].module",
        None,
    );

    let import_name_error = validation_error(
        valid_manifest().replace(r#"imports = ["Std.Logic.Eq"]"#, r#"imports = ["Std..Eq"]"#),
    );
    assert_manifest_error(
        &import_name_error,
        PackageManifestErrorKind::Domain,
        PackageManifestErrorReason::InvalidModuleName,
        "modules[0].imports[0]",
        None,
    );

    let declaration_error =
        validation_error(valid_manifest().replace(r#"theorems = ["id"]"#, r#"theorems = [""]"#));
    assert_manifest_error(
        &declaration_error,
        PackageManifestErrorKind::Domain,
        PackageManifestErrorReason::InvalidDeclarationName,
        "modules[0].theorems[0]",
        None,
    );

    let axiom_error = validation_error(valid_manifest().replace(
        r#"allowed_axioms = ["Eq.rec"]"#,
        r#"allowed_axioms = ["Eq..rec"]"#,
    ));
    assert_manifest_error(
        &axiom_error,
        PackageManifestErrorKind::Domain,
        PackageManifestErrorReason::InvalidAxiomName,
        "policy.allowed_axioms[0]",
        None,
    );
}

#[test]
fn package_manifest_paths_rejects_invalid_lexical_paths() {
    for (replacement, path) in [
        ("/Proofs/Ai/Basic/source.npa", "modules[0].source"),
        ("Proofs/Ai/../source.npa", "modules[0].source"),
        ("Proofs/Ai/./source.npa", "modules[0].source"),
        ("Proofs/Ai//source.npa", "modules[0].source"),
        (r#"Proofs\\Ai\\source.npa"#, "modules[0].source"),
        ("https://example.invalid/source.npa", "modules[0].source"),
    ] {
        let error = validation_error(valid_manifest().replace(
            r#"source = "Proofs/Ai/Basic/source.npa""#,
            &format!(r#"source = "{replacement}""#),
        ));
        assert_manifest_error(
            &error,
            PackageManifestErrorKind::Path,
            PackageManifestErrorReason::InvalidPath,
            path,
            None,
        );
    }

    let control_error = validation_error(valid_manifest().replace(
        r#"source = "Proofs/Ai/Basic/source.npa""#,
        r#"source = "Proofs/Ai/\u0008/source.npa""#,
    ));
    assert_manifest_error(
        &control_error,
        PackageManifestErrorKind::Path,
        PackageManifestErrorReason::InvalidPath,
        "modules[0].source",
        None,
    );
}

#[test]
fn package_manifest_paths_checks_external_import_certificate_path() {
    let error = validation_error(valid_manifest().replace(
        r#"certificate = "vendor/npa-std/Std/Logic/Eq/certificate.npcert""#,
        r#"certificate = "file://vendor/npa-std/Std/Logic/Eq/certificate.npcert""#,
    ));

    assert_manifest_error(
        &error,
        PackageManifestErrorKind::Path,
        PackageManifestErrorReason::InvalidPath,
        "imports[0].certificate",
        None,
    );
}

#[test]
fn package_manifest_hashes_rejects_uppercase_hash_hex() {
    let error = parse_and_validate_manifest_str(&valid_manifest().replace(
        r#"expected_export_hash = "sha256:0000000000000000000000000000000000000000000000000000000000000000""#,
        r#"expected_export_hash = "sha256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA""#,
    ))
    .unwrap_err();

    assert_manifest_error(
        &error,
        PackageManifestErrorKind::Hash,
        PackageManifestErrorReason::InvalidHashFormat,
        "modules[0].expected_export_hash",
        None,
    );
}

#[test]
fn package_manifest_hashes_rejects_bad_hash_prefix_and_length() {
    let bad_prefix_error = parse_and_validate_manifest_str(&valid_manifest().replace(
        r#"expected_source_hash = "sha256:0000000000000000000000000000000000000000000000000000000000000000""#,
        r#"expected_source_hash = "sha512:0000000000000000000000000000000000000000000000000000000000000000""#,
    ))
    .unwrap_err();
    assert_manifest_error(
        &bad_prefix_error,
        PackageManifestErrorKind::Hash,
        PackageManifestErrorReason::InvalidHashFormat,
        "modules[0].expected_source_hash",
        None,
    );

    let bad_length_error = parse_and_validate_manifest_str(&valid_manifest().replacen(
        r#"certificate_hash = "sha256:0000000000000000000000000000000000000000000000000000000000000000""#,
        r#"certificate_hash = "sha256:0000""#,
        1,
    ))
    .unwrap_err();
    assert_manifest_error(
        &bad_length_error,
        PackageManifestErrorKind::Hash,
        PackageManifestErrorReason::InvalidHashFormat,
        "imports[0].certificate_hash",
        None,
    );
}
