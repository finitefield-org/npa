//! Package manifest parsing entry points and raw accepted input types.

use npa_cert::Name;

use crate::{hash::PackageHash, name::PackageId, path::PackagePath};

/// Exact package version string accepted by `npa.package.v0.1`.
///
/// The grammar is fixed by CLR-01 validation: `MAJOR.MINOR.PATCH`, no leading
/// zeroes except the single digit `0`, and no pre-release or build metadata.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageVersion(pub String);

impl PackageVersion {
    /// Build a package version wrapper from a version string.
    pub fn new(version: impl Into<String>) -> Self {
        Self(version.into())
    }

    /// Return the version string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Accepted `npa.package.v0.1` manifest input shape.
///
/// This is package metadata, not proof evidence. The struct intentionally
/// contains only accepted manifest fields; generated checker verdicts,
/// registry lookups, implicit version resolution, and status fields are not
/// accepted manifest inputs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageManifest {
    /// Manifest schema string; must equal [`crate::schema::PACKAGE_MANIFEST_SCHEMA`].
    pub schema: String,
    /// Package identity.
    pub package: PackageId,
    /// Exact package version.
    pub version: PackageVersion,
    /// Core spec profile, for example `npa.core.v0.1`.
    pub core_spec: String,
    /// Kernel compatibility profile, for example `npa.kernel.v0.1`.
    pub kernel_profile: String,
    /// Certificate format profile, for example `npa.certificate.canonical.v0.1`.
    pub certificate_format: String,
    /// Required checker profile, for example `npa.checker.reference.v0.1`.
    pub checker_profile: String,
    /// Package axiom policy.
    pub policy: PackagePolicy,
    /// Local modules declared by this package.
    pub modules: Vec<PackageModule>,
    /// Optional package license expression.
    pub license: Option<String>,
    /// Optional informational source repository URL.
    pub repository: Option<String>,
    /// Optional informational package description.
    pub description: Option<String>,
    /// Optional hash-pinned external module imports.
    pub imports: Option<Vec<PackageExternalImport>>,
}

/// Package-level axiom policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackagePolicy {
    /// Whether axioms outside [`Self::allowed_axioms`] may appear.
    pub allow_custom_axioms: bool,
    /// Exact axiom names permitted by package policy.
    pub allowed_axioms: Vec<Name>,
}

/// Hash-pinned top-level external package/module import.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageExternalImport {
    /// External module name.
    pub module: Name,
    /// External package identity.
    pub package: PackageId,
    /// Exact external package version.
    pub version: PackageVersion,
    /// Package-relative path to the vendored external certificate.
    pub certificate: PackagePath,
    /// Exact canonical export hash for the external module.
    pub export_hash: PackageHash,
    /// Exact canonical certificate hash for high-trust identity.
    pub certificate_hash: PackageHash,
}

/// Local module entry in an `npa.package.v0.1` manifest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageModule {
    /// Local module name.
    pub module: Name,
    /// Package-relative source path.
    pub source: PackagePath,
    /// Package-relative certificate path.
    pub certificate: PackagePath,
    /// Direct module imports, resolved by package graph validation.
    pub imports: Vec<Name>,
    /// Expected SHA-256 hash of source file bytes.
    pub expected_source_hash: PackageHash,
    /// Expected SHA-256 hash of certificate file bytes.
    pub expected_certificate_file_hash: PackageHash,
    /// Expected canonical export hash from the certificate.
    pub expected_export_hash: PackageHash,
    /// Expected canonical axiom report hash from the certificate.
    pub expected_axiom_report_hash: PackageHash,
    /// Expected canonical certificate hash from the certificate.
    pub expected_certificate_hash: PackageHash,
    /// Optional untrusted metadata path.
    pub meta: Option<PackagePath>,
    /// Optional untrusted replay path.
    pub replay: Option<PackagePath>,
    /// Optional producer profile metadata.
    pub producer_profile: Option<String>,
    /// Optional inductive declaration summary.
    pub inductives: Option<Vec<Name>>,
    /// Optional definition declaration summary.
    pub definitions: Option<Vec<Name>>,
    /// Optional theorem declaration summary.
    pub theorems: Option<Vec<Name>>,
    /// Optional axiom declaration summary checked against package policy.
    pub axioms: Option<Vec<Name>>,
    /// Optional search/docs metadata tags.
    pub tags: Option<Vec<String>>,
}

/// Parse package manifest TOML into a structured value without reading files.
pub fn parse_toml_value(source: &str) -> Result<toml::Value, toml::de::Error> {
    source.parse()
}

#[cfg(test)]
mod tests {
    use npa_cert::Name;

    use super::{
        parse_toml_value, PackageExternalImport, PackageManifest, PackageModule, PackagePolicy,
        PackageVersion,
    };
    use crate::{PackageHash, PackageId, PackagePath, PACKAGE_MANIFEST_SCHEMA};

    #[test]
    fn package_manifest_skeleton_uses_structured_toml_parser() {
        let parsed = parse_toml_value("schema = \"npa.package.v0.1\"").unwrap();
        assert_eq!(parsed["schema"].as_str(), Some("npa.package.v0.1"));
    }

    #[test]
    fn package_manifest_schema_types_model_allowed_fields() {
        let zero_hash = PackageHash::new([0; 32]);
        let module = PackageModule {
            module: Name::from_dotted("Proofs.Ai.Basic"),
            source: PackagePath::new("Proofs/Ai/Basic/source.npa"),
            certificate: PackagePath::new("Proofs/Ai/Basic/certificate.npcert"),
            imports: vec![Name::from_dotted("Std.Logic.Eq")],
            expected_source_hash: zero_hash,
            expected_certificate_file_hash: zero_hash,
            expected_export_hash: zero_hash,
            expected_axiom_report_hash: zero_hash,
            expected_certificate_hash: zero_hash,
            meta: Some(PackagePath::new("Proofs/Ai/Basic/meta.json")),
            replay: Some(PackagePath::new("Proofs/Ai/Basic/replay.json")),
            producer_profile: Some("human-surface-explicit-term".to_owned()),
            inductives: Some(Vec::new()),
            definitions: Some(Vec::new()),
            theorems: Some(vec![Name::from_dotted("id")]),
            axioms: Some(Vec::new()),
            tags: Some(vec!["basic".to_owned()]),
        };
        let import = PackageExternalImport {
            module: Name::from_dotted("Std.Logic.Eq"),
            package: PackageId::new("npa-std"),
            version: PackageVersion::new("0.1.0"),
            certificate: PackagePath::new("vendor/npa-std/Std/Logic/Eq/certificate.npcert"),
            export_hash: zero_hash,
            certificate_hash: zero_hash,
        };
        let manifest = PackageManifest {
            schema: PACKAGE_MANIFEST_SCHEMA.to_owned(),
            package: PackageId::new("npa-proof-corpus"),
            version: PackageVersion::new("0.1.0"),
            core_spec: "npa.core.v0.1".to_owned(),
            kernel_profile: "npa.kernel.v0.1".to_owned(),
            certificate_format: "npa.certificate.canonical.v0.1".to_owned(),
            checker_profile: "npa.checker.reference.v0.1".to_owned(),
            policy: PackagePolicy {
                allow_custom_axioms: false,
                allowed_axioms: vec![Name::from_dotted("Eq.rec")],
            },
            modules: vec![module],
            license: Some("MIT".to_owned()),
            repository: Some("https://github.com/finitefield-org/npa".to_owned()),
            description: Some("proof corpus fixture".to_owned()),
            imports: Some(vec![import]),
        };

        assert_eq!(manifest.schema, PACKAGE_MANIFEST_SCHEMA);
        assert_eq!(manifest.modules[0].expected_export_hash, zero_hash);
        assert_eq!(
            manifest.imports.as_ref().unwrap()[0].certificate_hash,
            zero_hash
        );
    }
}
