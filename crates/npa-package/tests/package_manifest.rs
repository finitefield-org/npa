use npa_package::{
    parse_manifest_str, PackageManifestError, PackageManifestErrorKind, PackageManifestErrorReason,
    PACKAGE_MANIFEST_SCHEMA,
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
