use npa_cert::Name;
use npa_package::{
    build_package_publish_artifacts, compute_package_publish_plan_hash, format_package_hash,
    package_checksum_only_signature_policy, parse_package_hash, parse_package_publish_plan_json,
    parse_registry_module_json, PackageArtifactError, PackageArtifactErrorKind,
    PackageArtifactErrorReason, PackageArtifactFileReference, PackageArtifactOrigin,
    PackageCheckerMode, PackageCheckerSummary, PackageDownstreamImportBundle,
    PackageDownstreamImportModule, PackageHash, PackageId, PackageLockEntry,
    PackageLockEntryOrigin, PackageLockManifest, PackageLockManifestReference, PackagePath,
    PackagePublishArtifact, PackagePublishArtifactListInput, PackagePublishArtifactRole,
    PackagePublishPlan, PackagePublishRelease, PackagePublishReleaseReference,
    PackagePublishSummary, PackageRegistryArtifactHashes, PackageRegistryCheckerResult,
    PackageRegistryCheckerStatus, PackageRegistryImport, PackageRegistryModule,
    PackageSignaturePolicy, PackageVersion, PACKAGE_AXIOM_REPORT_SCHEMA, PACKAGE_LOCK_SCHEMA,
    PACKAGE_MANIFEST_SCHEMA, PACKAGE_PUBLISH_PLAN_PATH, PACKAGE_PUBLISH_PLAN_SCHEMA,
    PACKAGE_THEOREM_INDEX_SCHEMA, REGISTRY_MODULE_SCHEMA,
};

const CHECKER_BINARY_REGISTRY_SCHEMA: &str = "npa.independent-checker.checker_binary_registry.v1";
const ZERO_HASH: &str = "sha256:0000000000000000000000000000000000000000000000000000000000000000";
const ONE_HASH: &str = "sha256:1111111111111111111111111111111111111111111111111111111111111111";
const TWO_HASH: &str = "sha256:2222222222222222222222222222222222222222222222222222222222222222";
const THREE_HASH: &str = "sha256:3333333333333333333333333333333333333333333333333333333333333333";
const FOUR_HASH: &str = "sha256:4444444444444444444444444444444444444444444444444444444444444444";
const FIVE_HASH: &str = "sha256:5555555555555555555555555555555555555555555555555555555555555555";
const SIX_HASH: &str = "sha256:6666666666666666666666666666666666666666666666666666666666666666";
const SEVEN_HASH: &str = "sha256:7777777777777777777777777777777777777777777777777777777777777777";
const EIGHT_HASH: &str = "sha256:8888888888888888888888888888888888888888888888888888888888888888";
const A_HASH: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const B_HASH: &str = "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const D_HASH: &str = "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const E_HASH: &str = "sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

fn hash(value: &str) -> PackageHash {
    parse_package_hash(value, "test").unwrap()
}

fn name(value: &str) -> Name {
    Name::from_dotted(value)
}

fn release_ref(
    path: &str,
    file_hash: &str,
    content_hash: Option<&str>,
    schema: &str,
) -> PackagePublishReleaseReference {
    PackagePublishReleaseReference {
        path: PackagePath::new(path),
        file_hash: hash(file_hash),
        content_hash: content_hash.map(hash),
        schema: Some(schema.to_owned()),
    }
}

fn artifact_ref(path: &str, file_hash: &str) -> PackageArtifactFileReference {
    PackageArtifactFileReference {
        path: PackagePath::new(path),
        file_hash: hash(file_hash),
    }
}

fn checker_summary(module: &str) -> PackageCheckerSummary {
    PackageCheckerSummary {
        module: name(module),
        checker: "npa-checker-ref".to_owned(),
        profile: "npa.checker.reference.v0.1".to_owned(),
        mode: PackageCheckerMode::Reference,
        status: "passed".to_owned(),
        export_hash: hash(THREE_HASH),
        certificate_hash: hash(FOUR_HASH),
        axiom_report_hash: hash(FIVE_HASH),
    }
}

fn lock_manifest(entries: Vec<PackageLockEntry>) -> PackageLockManifest {
    PackageLockManifest {
        schema: PACKAGE_LOCK_SCHEMA.to_owned(),
        package: PackageId::new("npa-proof-corpus"),
        version: PackageVersion::new("0.1.0"),
        manifest: PackageLockManifestReference {
            path: PackagePath::new("npa-package.toml"),
            file_hash: hash(ZERO_HASH),
        },
        entries,
    }
}

fn lock_entry(
    module: &str,
    origin: PackageLockEntryOrigin,
    certificate: &str,
    certificate_file_hash: &str,
) -> PackageLockEntry {
    PackageLockEntry {
        module: name(module),
        origin,
        certificate: PackagePath::new(certificate),
        certificate_file_hash: hash(certificate_file_hash),
        export_hash: hash(THREE_HASH),
        axiom_report_hash: hash(FIVE_HASH),
        certificate_hash: hash(FOUR_HASH),
        imports: Vec::new(),
        package: (origin == PackageLockEntryOrigin::External).then(|| PackageId::new("npa-std")),
        version: (origin == PackageLockEntryOrigin::External).then(|| PackageVersion::new("0.1.0")),
    }
}

fn registry_module(module: &str, certificate: &str) -> PackageRegistryModule {
    PackageRegistryModule {
        schema: REGISTRY_MODULE_SCHEMA.to_owned(),
        package: PackageId::new("npa-proof-corpus"),
        package_version: PackageVersion::new("0.1.0"),
        module: name(module),
        core_spec: "npa.core.v0.1".to_owned(),
        kernel_profile: "npa.kernel.v0.1".to_owned(),
        certificate_format: "npa.certificate.canonical.v0.1".to_owned(),
        export_hash: hash(THREE_HASH),
        certificate_hash: hash(FOUR_HASH),
        axiom_report_hash: hash(FIVE_HASH),
        certificate: npa_package::PackageArtifactFileReference {
            path: PackagePath::new(certificate),
            file_hash: hash(SIX_HASH),
        },
        imports: vec![PackageRegistryImport {
            module: name("Std.Logic.Eq"),
            origin: PackageArtifactOrigin::External,
            package: Some(PackageId::new("npa-std")),
            version: Some(PackageVersion::new("0.1.0")),
            export_hash: hash(SEVEN_HASH),
            certificate_hash: hash(EIGHT_HASH),
        }],
        checker_results: vec![PackageRegistryCheckerResult {
            checker: "npa-checker-ref".to_owned(),
            profile: "npa.checker.reference.v0.1".to_owned(),
            mode: "reference".to_owned(),
            status: PackageRegistryCheckerStatus::Accepted,
            export_hash: hash(THREE_HASH),
            certificate_hash: hash(FOUR_HASH),
            axiom_report_hash: hash(FIVE_HASH),
        }],
        artifact_hashes: PackageRegistryArtifactHashes {
            package_lock_file_hash: hash(ONE_HASH),
            axiom_report_file_hash: hash(TWO_HASH),
            theorem_index_file_hash: hash(THREE_HASH),
        },
    }
}

fn local_certificate_artifact(module: &str, path: &str, file_hash: &str) -> PackagePublishArtifact {
    PackagePublishArtifact {
        role: PackagePublishArtifactRole::LocalCertificate,
        path: PackagePath::new(path),
        file_hash: hash(file_hash),
        module: Some(name(module)),
        origin: Some(PackageArtifactOrigin::Local),
        schema: None,
    }
}

fn base_publish_plan() -> PackagePublishPlan {
    PackagePublishPlan {
        schema: PACKAGE_PUBLISH_PLAN_SCHEMA.to_owned(),
        package: PackageId::new("npa-proof-corpus"),
        version: PackageVersion::new("0.1.0"),
        release: PackagePublishRelease {
            core_spec: "npa.core.v0.1".to_owned(),
            kernel_profile: "npa.kernel.v0.1".to_owned(),
            certificate_format: "npa.certificate.canonical.v0.1".to_owned(),
            checker_profile: "npa.checker.reference.v0.1".to_owned(),
            manifest: release_ref("npa-package.toml", ZERO_HASH, None, PACKAGE_MANIFEST_SCHEMA),
            package_lock: release_ref(
                "generated/package-lock.json",
                ONE_HASH,
                None,
                PACKAGE_LOCK_SCHEMA,
            ),
            axiom_report: release_ref(
                "generated/axiom-report.json",
                TWO_HASH,
                Some(THREE_HASH),
                PACKAGE_AXIOM_REPORT_SCHEMA,
            ),
            theorem_index: release_ref(
                "generated/theorem-index.json",
                FOUR_HASH,
                Some(FIVE_HASH),
                PACKAGE_THEOREM_INDEX_SCHEMA,
            ),
        },
        artifacts: vec![
            local_certificate_artifact("Proofs.Z", "Proofs/Z/certificate.npcert", E_HASH),
            PackagePublishArtifact {
                role: PackagePublishArtifactRole::TheoremIndex,
                path: PackagePath::new("generated/theorem-index.json"),
                file_hash: hash(FOUR_HASH),
                module: None,
                origin: None,
                schema: Some(PACKAGE_THEOREM_INDEX_SCHEMA.to_owned()),
            },
            local_certificate_artifact("Proofs.A", "Proofs/A/certificate.npcert", D_HASH),
            PackagePublishArtifact {
                role: PackagePublishArtifactRole::PackageManifest,
                path: PackagePath::new("npa-package.toml"),
                file_hash: hash(ZERO_HASH),
                module: None,
                origin: None,
                schema: Some(PACKAGE_MANIFEST_SCHEMA.to_owned()),
            },
            PackagePublishArtifact {
                role: PackagePublishArtifactRole::AxiomReport,
                path: PackagePath::new("generated/axiom-report.json"),
                file_hash: hash(TWO_HASH),
                module: None,
                origin: None,
                schema: Some(PACKAGE_AXIOM_REPORT_SCHEMA.to_owned()),
            },
            PackagePublishArtifact {
                role: PackagePublishArtifactRole::PackageLock,
                path: PackagePath::new("generated/package-lock.json"),
                file_hash: hash(ONE_HASH),
                module: None,
                origin: None,
                schema: Some(PACKAGE_LOCK_SCHEMA.to_owned()),
            },
        ],
        module_registry_entries: vec![
            registry_module("Proofs.Z", "Proofs/Z/certificate.npcert"),
            registry_module("Proofs.A", "Proofs/A/certificate.npcert"),
        ],
        downstream_import_bundle: PackageDownstreamImportBundle {
            package: PackageId::new("npa-proof-corpus"),
            version: PackageVersion::new("0.1.0"),
            modules: vec![
                PackageDownstreamImportModule {
                    module: name("Proofs.Z"),
                    package: PackageId::new("npa-proof-corpus"),
                    version: PackageVersion::new("0.1.0"),
                    export_hash: hash(THREE_HASH),
                    certificate_hash: hash(FOUR_HASH),
                    axiom_report_hash: hash(FIVE_HASH),
                    certificate: PackagePath::new("Proofs/Z/certificate.npcert"),
                    certificate_file_hash: hash(E_HASH),
                },
                PackageDownstreamImportModule {
                    module: name("Proofs.A"),
                    package: PackageId::new("npa-proof-corpus"),
                    version: PackageVersion::new("0.1.0"),
                    export_hash: hash(THREE_HASH),
                    certificate_hash: hash(FOUR_HASH),
                    axiom_report_hash: hash(FIVE_HASH),
                    certificate: PackagePath::new("Proofs/A/certificate.npcert"),
                    certificate_file_hash: hash(D_HASH),
                },
            ],
        },
        checker_summaries: vec![checker_summary("Proofs.Z"), checker_summary("Proofs.A")],
        signature_policy: PackageSignaturePolicy {
            mode: "checksum-only".to_owned(),
            hash_algorithm: "sha256".to_owned(),
            signature_required: false,
            signatures: vec![],
        },
        summary: PackagePublishSummary {
            local_module_count: 2,
            external_import_count: 0,
            artifact_count: 6,
            registry_entry_count: 2,
            checker_summary_count: 2,
        },
        publish_plan_hash: hash(ZERO_HASH),
    }
}

fn assert_artifact_error(
    error: PackageArtifactError,
    kind: PackageArtifactErrorKind,
    reason: PackageArtifactErrorReason,
    field: Option<&str>,
) {
    assert_eq!(error.kind, kind);
    assert_eq!(error.reason_code, reason);
    assert_eq!(error.reason_code.as_str(), reason.as_str());
    assert_eq!(error.field.as_deref(), field);
}

#[test]
fn publish_plan_artifacts_build_canonical_release_list_and_checksum_policy() {
    let lock = lock_manifest(vec![
        lock_entry(
            "Proofs.Z",
            PackageLockEntryOrigin::Local,
            "Proofs/Z/certificate.npcert",
            E_HASH,
        ),
        lock_entry(
            "Std.Logic.Eq",
            PackageLockEntryOrigin::External,
            "vendor/std/Std/Logic/Eq/certificate.npcert",
            SEVEN_HASH,
        ),
        lock_entry(
            "Proofs.A",
            PackageLockEntryOrigin::Local,
            "Proofs/A/certificate.npcert",
            D_HASH,
        ),
    ]);

    let artifacts = build_package_publish_artifacts(PackagePublishArtifactListInput {
        manifest: artifact_ref("npa-package.toml", ZERO_HASH),
        package_lock: artifact_ref("generated/package-lock.json", ONE_HASH),
        axiom_report: artifact_ref("generated/axiom-report.json", TWO_HASH),
        theorem_index: artifact_ref("generated/theorem-index.json", THREE_HASH),
        package_lock_manifest: &lock,
    })
    .unwrap();

    let keys = artifacts
        .iter()
        .map(|artifact| {
            format!(
                "{}|{}|{}",
                artifact.role.as_str(),
                artifact
                    .module
                    .as_ref()
                    .map(Name::as_dotted)
                    .unwrap_or_default(),
                artifact.path.as_str()
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        keys,
        vec![
            "axiom_report||generated/axiom-report.json",
            "external_import_certificate|Std.Logic.Eq|vendor/std/Std/Logic/Eq/certificate.npcert",
            "local_certificate|Proofs.A|Proofs/A/certificate.npcert",
            "local_certificate|Proofs.Z|Proofs/Z/certificate.npcert",
            "package_lock||generated/package-lock.json",
            "package_manifest||npa-package.toml",
            "theorem_index||generated/theorem-index.json",
        ]
    );
    assert!(!artifacts
        .iter()
        .any(|artifact| artifact.path.as_str() == PACKAGE_PUBLISH_PLAN_PATH));

    let external = artifacts
        .iter()
        .find(|artifact| artifact.role == PackagePublishArtifactRole::ExternalImportCertificate)
        .unwrap();
    assert_eq!(external.file_hash, hash(SEVEN_HASH));
    assert_eq!(external.origin, Some(PackageArtifactOrigin::External));

    let policy = package_checksum_only_signature_policy();
    assert_eq!(policy.mode, "checksum-only");
    assert_eq!(policy.hash_algorithm, "sha256");
    assert!(!policy.signature_required);
    assert!(policy.signatures.is_empty());
}

#[test]
fn publish_plan_artifacts_reject_missing_or_mismatched_release_entries() {
    let mut missing_manifest = base_publish_plan();
    missing_manifest
        .artifacts
        .retain(|artifact| artifact.role != PackagePublishArtifactRole::PackageManifest);
    assert_artifact_error(
        missing_manifest.with_computed_hash().unwrap_err(),
        PackageArtifactErrorKind::ArtifactSchema,
        PackageArtifactErrorReason::MissingField,
        Some("package_manifest"),
    );

    let mut missing_local_certificate = base_publish_plan();
    missing_local_certificate
        .artifacts
        .retain(|artifact| artifact.module.as_ref() != Some(&name("Proofs.A")));
    assert_artifact_error(
        missing_local_certificate.with_computed_hash().unwrap_err(),
        PackageArtifactErrorKind::ArtifactSchema,
        PackageArtifactErrorReason::MissingField,
        Some("Proofs.A"),
    );

    let mut mismatched_lock = base_publish_plan();
    mismatched_lock
        .artifacts
        .iter_mut()
        .find(|artifact| artifact.role == PackagePublishArtifactRole::PackageLock)
        .unwrap()
        .file_hash = hash(B_HASH);
    assert_artifact_error(
        mismatched_lock.with_computed_hash().unwrap_err(),
        PackageArtifactErrorKind::Domain,
        PackageArtifactErrorReason::InvalidEnumValue,
        Some("file_hash"),
    );

    let lock = lock_manifest(vec![lock_entry(
        "Proofs.Cycle",
        PackageLockEntryOrigin::Local,
        PACKAGE_PUBLISH_PLAN_PATH,
        D_HASH,
    )]);
    assert_artifact_error(
        build_package_publish_artifacts(PackagePublishArtifactListInput {
            manifest: artifact_ref("npa-package.toml", ZERO_HASH),
            package_lock: artifact_ref("generated/package-lock.json", ONE_HASH),
            axiom_report: artifact_ref("generated/axiom-report.json", TWO_HASH),
            theorem_index: artifact_ref("generated/theorem-index.json", THREE_HASH),
            package_lock_manifest: &lock,
        })
        .unwrap_err(),
        PackageArtifactErrorKind::ArtifactSchema,
        PackageArtifactErrorReason::ReleaseArtifactSelfReference,
        Some("artifacts"),
    );
}

#[test]
fn publish_plan_schema_constants_and_rejections() {
    let plan = base_publish_plan().with_computed_hash().unwrap();
    let canonical = plan.canonical_json().unwrap();
    let parsed = parse_package_publish_plan_json(&canonical).unwrap();

    assert_eq!(parsed.schema, "npa.package.publish_plan.v0.1");
    assert_ne!(parsed.schema, CHECKER_BINARY_REGISTRY_SCHEMA);

    let checker_registry_schema = canonical.replacen(
        PACKAGE_PUBLISH_PLAN_SCHEMA,
        CHECKER_BINARY_REGISTRY_SCHEMA,
        1,
    );
    assert_artifact_error(
        parse_package_publish_plan_json(&checker_registry_schema).unwrap_err(),
        PackageArtifactErrorKind::ArtifactSchema,
        PackageArtifactErrorReason::UnsupportedSchema,
        Some("schema"),
    );

    let unknown_timestamp = canonical.replacen(
        r#""package":"npa-proof-corpus""#,
        r#""timestamp":"2026-01-01T00:00:00Z","package":"npa-proof-corpus""#,
        1,
    );
    assert_artifact_error(
        parse_package_publish_plan_json(&unknown_timestamp).unwrap_err(),
        PackageArtifactErrorKind::ArtifactSchema,
        PackageArtifactErrorReason::UnknownField,
        Some("timestamp"),
    );

    let absolute_path = canonical.replacen("Proofs/A/certificate.npcert", "/tmp/cert.npcert", 1);
    assert_artifact_error(
        parse_package_publish_plan_json(&absolute_path).unwrap_err(),
        PackageArtifactErrorKind::Path,
        PackageArtifactErrorReason::InvalidPath,
        None,
    );

    let latest_version = canonical.replacen(
        r#""version":"0.1.0","modules":"#,
        r#""version":"latest","modules":"#,
        1,
    );
    assert_artifact_error(
        parse_package_publish_plan_json(&latest_version).unwrap_err(),
        PackageArtifactErrorKind::Domain,
        PackageArtifactErrorReason::InvalidVersion,
        None,
    );

    let mut self_reference = base_publish_plan();
    self_reference.artifacts[0].path = PackagePath::new(PACKAGE_PUBLISH_PLAN_PATH);
    assert_artifact_error(
        self_reference.with_computed_hash().unwrap_err(),
        PackageArtifactErrorKind::ArtifactSchema,
        PackageArtifactErrorReason::ReleaseArtifactSelfReference,
        Some("artifacts"),
    );
}

#[test]
fn registry_module_schema_constants_and_checker_schema_separation() {
    let entry = registry_module("Proofs.A", "Proofs/A/certificate.npcert");
    let canonical = entry.canonical_json().unwrap();
    let parsed = parse_registry_module_json(&canonical).unwrap();

    assert_eq!(parsed.schema, "npa.registry.module.v0.1");
    assert_ne!(parsed.schema, CHECKER_BINARY_REGISTRY_SCHEMA);

    let checker_registry_schema =
        canonical.replacen(REGISTRY_MODULE_SCHEMA, CHECKER_BINARY_REGISTRY_SCHEMA, 1);
    assert_artifact_error(
        parse_registry_module_json(&checker_registry_schema).unwrap_err(),
        PackageArtifactErrorKind::ArtifactSchema,
        PackageArtifactErrorReason::UnsupportedSchema,
        Some("schema"),
    );

    let registry_url = canonical.replacen(
        r#""artifact_hashes":"#,
        r#""download_url":"https://registry.example/Proofs/A","artifact_hashes":"#,
        1,
    );
    assert_artifact_error(
        parse_registry_module_json(&registry_url).unwrap_err(),
        PackageArtifactErrorKind::ArtifactSchema,
        PackageArtifactErrorReason::UnknownField,
        Some("download_url"),
    );

    let absolute_path = canonical.replacen(
        "Proofs/A/certificate.npcert",
        "https://registry.example/Proofs/A/certificate.npcert",
        1,
    );
    assert_artifact_error(
        parse_registry_module_json(&absolute_path).unwrap_err(),
        PackageArtifactErrorKind::Path,
        PackageArtifactErrorReason::InvalidPath,
        None,
    );

    let latest_import = canonical.replacen(
        r#""version":"0.1.0","export_hash":"#,
        r#""version":"latest","export_hash":"#,
        1,
    );
    assert_artifact_error(
        parse_registry_module_json(&latest_import).unwrap_err(),
        PackageArtifactErrorKind::Domain,
        PackageArtifactErrorReason::InvalidVersion,
        None,
    );

    let mut noncanonical = canonical;
    noncanonical.push('\n');
    assert_artifact_error(
        parse_registry_module_json(&noncanonical).unwrap_err(),
        PackageArtifactErrorKind::CanonicalJson,
        PackageArtifactErrorReason::NonCanonicalOrder,
        None,
    );
}

#[test]
fn publish_plan_canonical_json_sorts_self_hashes_and_rejects_stale_or_ambiguous_metadata() {
    let plan = base_publish_plan().with_computed_hash().unwrap();
    let canonical = plan.canonical_json().unwrap();
    let parsed = parse_package_publish_plan_json(&canonical).unwrap();

    assert!(
        canonical.find(r#""role":"axiom_report""#).unwrap()
            < canonical.find(r#""role":"local_certificate""#).unwrap()
    );
    assert!(
        canonical.find(r#""module":"Proofs.A""#).unwrap()
            < canonical.find(r#""module":"Proofs.Z""#).unwrap()
    );
    assert!(!canonical.contains(&format!(r#""path":"{PACKAGE_PUBLISH_PLAN_PATH}""#)));
    assert_eq!(parsed.module_registry_entries.len(), 2);

    let mut changed_hash = plan.clone();
    changed_hash.publish_plan_hash = hash(A_HASH);
    assert_eq!(
        compute_package_publish_plan_hash(&changed_hash).unwrap(),
        plan.publish_plan_hash
    );

    let stale = canonical.replace(&format_package_hash(&plan.publish_plan_hash), B_HASH);
    assert_artifact_error(
        parse_package_publish_plan_json(&stale).unwrap_err(),
        PackageArtifactErrorKind::SelfHash,
        PackageArtifactErrorReason::SelfHashMismatch,
        Some("publish_plan_hash"),
    );

    let mut noncanonical = canonical.clone();
    noncanonical.push('\n');
    assert_artifact_error(
        parse_package_publish_plan_json(&noncanonical).unwrap_err(),
        PackageArtifactErrorKind::CanonicalJson,
        PackageArtifactErrorReason::NonCanonicalOrder,
        None,
    );

    let mut duplicate_registry = base_publish_plan();
    duplicate_registry.module_registry_entries[1].module = name("Proofs.Z");
    assert_artifact_error(
        duplicate_registry.with_computed_hash().unwrap_err(),
        PackageArtifactErrorKind::Duplicate,
        PackageArtifactErrorReason::DuplicateModule,
        Some("module_registry_entries"),
    );

    let mut duplicate_artifact = base_publish_plan();
    duplicate_artifact.artifacts[0].path = PackagePath::new("generated/package-lock.json");
    assert_artifact_error(
        duplicate_artifact.with_computed_hash().unwrap_err(),
        PackageArtifactErrorKind::Duplicate,
        PackageArtifactErrorReason::DuplicateArtifact,
        Some("artifacts"),
    );
}
