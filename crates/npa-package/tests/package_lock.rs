use npa_cert::Name;
use npa_package::{
    parse_package_hash, parse_package_lock_json, PackageHash, PackageId, PackageLockEntry,
    PackageLockEntryOrigin, PackageLockError, PackageLockErrorKind, PackageLockErrorReason,
    PackageLockImport, PackageLockManifest, PackageLockManifestReference, PackagePath,
    PackageVersion, PACKAGE_LOCK_SCHEMA,
};

const ZERO_HASH: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";
const ONE_HASH: &str = "sha256:1111111111111111111111111111111111111111111111111111111111111111";
const TWO_HASH: &str = "sha256:2222222222222222222222222222222222222222222222222222222222222222";
const THREE_HASH: &str = "sha256:3333333333333333333333333333333333333333333333333333333333333333";
const FOUR_HASH: &str = "sha256:4444444444444444444444444444444444444444444444444444444444444444";
const FIVE_HASH: &str = "sha256:5555555555555555555555555555555555555555555555555555555555555555";
const SIX_HASH: &str = "sha256:6666666666666666666666666666666666666666666666666666666666666666";
const EQ_EXPORT_HASH: &str =
    "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const EQ_CERT_HASH: &str =
    "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const EQ_AXIOM_HASH: &str =
    "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const NAT_EXPORT_HASH: &str =
    "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const NAT_CERT_HASH: &str =
    "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
const NAT_AXIOM_HASH: &str =
    "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";

fn expected_canonical_json() -> String {
    format!(
        concat!(
            r#"{{"schema":"npa.package.lock.v0.1","package":"npa-proof-corpus","version":"0.1.0","#,
            r#""manifest":{{"path":"npa-package.toml","file_hash":"{zero}"}},"entries":["#,
            r#"{{"module":"Proofs.Ai.Basic","origin":"local","certificate":"Proofs/Ai/Basic/certificate.npcert","#,
            r#""certificate_file_hash":"{one}","export_hash":"{two}","axiom_report_hash":"{three}","#,
            r#""certificate_hash":"{four}","imports":["#,
            r#"{{"module":"Std.Logic.Eq","export_hash":"{eq_export}","certificate_hash":"{eq_cert}"}},"#,
            r#"{{"module":"Std.Nat.Basic","export_hash":"{nat_export}","certificate_hash":"{nat_cert}"}}"#,
            r#"]}},"#,
            r#"{{"module":"Std.Logic.Eq","origin":"external","package":"npa-std","version":"0.1.0","#,
            r#""certificate":"vendor/npa-std/Std/Logic/Eq/certificate.npcert","certificate_file_hash":"{five}","#,
            r#""export_hash":"{eq_export}","axiom_report_hash":"{eq_axiom}","certificate_hash":"{eq_cert}","imports":[]}},"#,
            r#"{{"module":"Std.Nat.Basic","origin":"external","package":"npa-std","version":"0.1.0","#,
            r#""certificate":"vendor/npa-std/Std/Nat/Basic/certificate.npcert","certificate_file_hash":"{six}","#,
            r#""export_hash":"{nat_export}","axiom_report_hash":"{nat_axiom}","certificate_hash":"{nat_cert}","imports":[]}}"#,
            r#"]}}"#
        ),
        zero = ZERO_HASH,
        one = ONE_HASH,
        two = TWO_HASH,
        three = THREE_HASH,
        four = FOUR_HASH,
        five = FIVE_HASH,
        six = SIX_HASH,
        eq_export = EQ_EXPORT_HASH,
        eq_cert = EQ_CERT_HASH,
        eq_axiom = EQ_AXIOM_HASH,
        nat_export = NAT_EXPORT_HASH,
        nat_cert = NAT_CERT_HASH,
        nat_axiom = NAT_AXIOM_HASH,
    )
}

fn hash(value: &str) -> PackageHash {
    parse_package_hash(value, "test").unwrap()
}

fn import(module: &str, export_hash: &str, certificate_hash: &str) -> PackageLockImport {
    PackageLockImport {
        module: Name::from_dotted(module),
        export_hash: hash(export_hash),
        certificate_hash: hash(certificate_hash),
    }
}

fn external_entry(
    module: &str,
    certificate: &str,
    certificate_file_hash: &str,
    export_hash: &str,
    axiom_report_hash: &str,
    certificate_hash: &str,
) -> PackageLockEntry {
    PackageLockEntry {
        module: Name::from_dotted(module),
        origin: PackageLockEntryOrigin::External,
        certificate: PackagePath::new(certificate),
        certificate_file_hash: hash(certificate_file_hash),
        export_hash: hash(export_hash),
        axiom_report_hash: hash(axiom_report_hash),
        certificate_hash: hash(certificate_hash),
        imports: vec![],
        package: Some(PackageId::new("npa-std")),
        version: Some(PackageVersion::new("0.1.0")),
    }
}

fn unsorted_lock() -> PackageLockManifest {
    PackageLockManifest {
        schema: PACKAGE_LOCK_SCHEMA.to_owned(),
        package: PackageId::new("npa-proof-corpus"),
        version: PackageVersion::new("0.1.0"),
        manifest: PackageLockManifestReference {
            path: PackagePath::new("npa-package.toml"),
            file_hash: hash(ZERO_HASH),
        },
        entries: vec![
            external_entry(
                "Std.Nat.Basic",
                "vendor/npa-std/Std/Nat/Basic/certificate.npcert",
                SIX_HASH,
                NAT_EXPORT_HASH,
                NAT_AXIOM_HASH,
                NAT_CERT_HASH,
            ),
            PackageLockEntry {
                module: Name::from_dotted("Proofs.Ai.Basic"),
                origin: PackageLockEntryOrigin::Local,
                certificate: PackagePath::new("Proofs/Ai/Basic/certificate.npcert"),
                certificate_file_hash: hash(ONE_HASH),
                export_hash: hash(TWO_HASH),
                axiom_report_hash: hash(THREE_HASH),
                certificate_hash: hash(FOUR_HASH),
                imports: vec![
                    import("Std.Nat.Basic", NAT_EXPORT_HASH, NAT_CERT_HASH),
                    import("Std.Logic.Eq", EQ_EXPORT_HASH, EQ_CERT_HASH),
                ],
                package: None,
                version: None,
            },
            external_entry(
                "Std.Logic.Eq",
                "vendor/npa-std/Std/Logic/Eq/certificate.npcert",
                FIVE_HASH,
                EQ_EXPORT_HASH,
                EQ_AXIOM_HASH,
                EQ_CERT_HASH,
            ),
        ],
    }
}

fn assert_lock_error(
    error: &PackageLockError,
    kind: PackageLockErrorKind,
    reason: PackageLockErrorReason,
    path: &str,
    field: Option<&str>,
) {
    assert_eq!(error.kind, kind);
    assert_eq!(error.reason_code, reason);
    assert_eq!(error.reason_code.as_str(), reason.as_str());
    assert_eq!(error.path, path);
    assert_eq!(error.field.as_deref(), field);
}

#[test]
fn package_lock_canonical_json_sorts_entries_and_imports() {
    let canonical = unsorted_lock().canonical_json().unwrap();

    assert_eq!(canonical, expected_canonical_json());
}

#[test]
fn package_lock_canonical_json_round_trips_to_the_same_bytes() {
    let parsed = parse_package_lock_json(&expected_canonical_json()).unwrap();

    assert_eq!(parsed.entries[0].module.as_dotted(), "Proofs.Ai.Basic");
    assert_eq!(
        parsed.entries[0].imports[0].module.as_dotted(),
        "Std.Logic.Eq"
    );
    assert_eq!(
        parsed.entries[0].imports[1].module.as_dotted(),
        "Std.Nat.Basic"
    );
    assert_eq!(parsed.entries[1].origin, PackageLockEntryOrigin::External);
    assert_eq!(
        parsed.entries[1].package.as_ref().unwrap().as_str(),
        "npa-std"
    );
    assert_eq!(parsed.canonical_json().unwrap(), expected_canonical_json());
}

#[test]
fn package_lock_schema_rejects_unknown_fields() {
    let source = expected_canonical_json().replacen(
        r#""entries":["#,
        r#""source":"Proofs/Ai/Basic/source.npa","entries":["#,
        1,
    );

    let error = parse_package_lock_json(&source).unwrap_err();

    assert_lock_error(
        &error,
        PackageLockErrorKind::LockSchema,
        PackageLockErrorReason::UnknownField,
        "$",
        Some("source"),
    );
}

#[test]
fn package_lock_schema_rejects_unknown_nested_fields() {
    let source = expected_canonical_json().replacen(
        r#""module":"Std.Logic.Eq","export_hash":"#,
        r#""module":"Std.Logic.Eq","source":"Std/Logic/Eq.npa","export_hash":"#,
        1,
    );

    let error = parse_package_lock_json(&source).unwrap_err();

    assert_lock_error(
        &error,
        PackageLockErrorKind::LockSchema,
        PackageLockErrorReason::UnknownField,
        "entries[0].imports[0]",
        Some("source"),
    );
}

#[test]
fn package_lock_schema_rejects_duplicate_json_fields() {
    let source = expected_canonical_json().replacen(
        r#""schema":"npa.package.lock.v0.1","#,
        r#""schema":"npa.package.lock.v0.1","schema":"npa.package.lock.v0.1","#,
        1,
    );

    let error = parse_package_lock_json(&source).unwrap_err();

    assert_lock_error(
        &error,
        PackageLockErrorKind::LockSchema,
        PackageLockErrorReason::DuplicateField,
        "$",
        Some("schema"),
    );
}

#[test]
fn package_lock_schema_rejects_duplicate_modules() {
    let source = expected_canonical_json().replacen(
        r#""module":"Std.Nat.Basic","origin":"external""#,
        r#""module":"Std.Logic.Eq","origin":"external""#,
        1,
    );

    let error = parse_package_lock_json(&source).unwrap_err();

    assert_lock_error(
        &error,
        PackageLockErrorKind::Duplicate,
        PackageLockErrorReason::DuplicateLockEntry,
        "entries[2].module",
        Some("module"),
    );
}

#[test]
fn package_lock_schema_rejects_duplicate_certificate_paths() {
    let source = expected_canonical_json().replacen(
        "vendor/npa-std/Std/Nat/Basic/certificate.npcert",
        "vendor/npa-std/Std/Logic/Eq/certificate.npcert",
        1,
    );

    let error = parse_package_lock_json(&source).unwrap_err();

    assert_lock_error(
        &error,
        PackageLockErrorKind::Duplicate,
        PackageLockErrorReason::DuplicateCertificatePath,
        "entries[2].certificate",
        Some("certificate"),
    );
}

#[test]
fn package_lock_schema_rejects_duplicate_imports() {
    let source = expected_canonical_json().replacen(
        r#""module":"Std.Nat.Basic","export_hash":"#,
        r#""module":"Std.Logic.Eq","export_hash":"#,
        1,
    );

    let error = parse_package_lock_json(&source).unwrap_err();

    assert_lock_error(
        &error,
        PackageLockErrorKind::Duplicate,
        PackageLockErrorReason::DuplicateImport,
        "entries[0].imports[1].module",
        Some("module"),
    );
}

#[test]
fn package_lock_schema_rejects_malformed_hashes() {
    let source = expected_canonical_json().replacen(ONE_HASH, "sha256:bad", 1);

    let error = parse_package_lock_json(&source).unwrap_err();

    assert_lock_error(
        &error,
        PackageLockErrorKind::Hash,
        PackageLockErrorReason::InvalidHashFormat,
        "entries[0].certificate_file_hash",
        None,
    );
}

#[test]
fn package_lock_schema_rejects_malformed_paths() {
    let source = expected_canonical_json().replacen(
        "Proofs/Ai/Basic/certificate.npcert",
        "../certificate.npcert",
        1,
    );

    let error = parse_package_lock_json(&source).unwrap_err();

    assert_lock_error(
        &error,
        PackageLockErrorKind::Path,
        PackageLockErrorReason::InvalidPath,
        "entries[0].certificate",
        None,
    );
}

#[test]
fn package_lock_schema_rejects_malformed_package_identity() {
    let source = expected_canonical_json().replacen(
        r#""package":"npa-proof-corpus""#,
        r#""package":"NPA-Proof-Corpus""#,
        1,
    );

    let error = parse_package_lock_json(&source).unwrap_err();

    assert_lock_error(
        &error,
        PackageLockErrorKind::Domain,
        PackageLockErrorReason::InvalidPackageId,
        "package",
        None,
    );
}

#[test]
fn package_lock_schema_rejects_malformed_versions() {
    let source =
        expected_canonical_json().replacen(r#""version":"0.1.0""#, r#""version":"01.0.0""#, 1);

    let error = parse_package_lock_json(&source).unwrap_err();

    assert_lock_error(
        &error,
        PackageLockErrorKind::Domain,
        PackageLockErrorReason::InvalidVersion,
        "version",
        None,
    );
}

#[test]
fn package_lock_schema_rejects_malformed_names() {
    let source = expected_canonical_json().replacen(
        r#""module":"Proofs.Ai.Basic""#,
        r#""module":"Proofs..Bad""#,
        1,
    );

    let error = parse_package_lock_json(&source).unwrap_err();

    assert_lock_error(
        &error,
        PackageLockErrorKind::Domain,
        PackageLockErrorReason::InvalidModuleName,
        "entries[0].module",
        None,
    );
}

#[test]
fn package_lock_schema_requires_external_package_and_version() {
    let source = expected_canonical_json().replacen(
        r#""origin":"external","package":"npa-std","version":"0.1.0""#,
        r#""origin":"external","version":"0.1.0""#,
        1,
    );

    let error = parse_package_lock_json(&source).unwrap_err();

    assert_lock_error(
        &error,
        PackageLockErrorKind::LockSchema,
        PackageLockErrorReason::ExternalFieldRequired,
        "entries[1].package",
        Some("package"),
    );
}

#[test]
fn package_lock_schema_rejects_local_package_identity_fields() {
    let source = expected_canonical_json().replacen(
        r#""module":"Proofs.Ai.Basic","origin":"local","certificate":"#,
        r#""module":"Proofs.Ai.Basic","origin":"local","package":"npa-proof-corpus","certificate":"#,
        1,
    );

    let error = parse_package_lock_json(&source).unwrap_err();

    assert_lock_error(
        &error,
        PackageLockErrorKind::LockSchema,
        PackageLockErrorReason::LocalFieldForbidden,
        "entries[0].package",
        Some("package"),
    );
}
