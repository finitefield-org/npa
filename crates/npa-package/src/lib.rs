//! Package manifest metadata parsing and validation for external NPA libraries.
//!
//! `npa-package` handles the untrusted `npa.package.v0.1` metadata format used
//! by `npa-package.toml`. It helps package, CLI, and registry tooling reject
//! malformed metadata before build or verification commands run, but it is not
//! part of the trusted base. Theorem acceptance remains with canonical proof
//! certificates, the Rust kernel verdict, and independent source-free checker
//! verdicts.
//!
//! Manifest validation is deliberately lexical and metadata-only. It does not
//! read source files, read certificate files, compare artifact bytes, execute
//! checkers, contact registries, resolve latest versions, query the network, or
//! load plugins. In particular, `npa-package` does not read source or
//! certificate files for proof acceptance; later CLI commands may compare file
//! hashes or invoke checkers, but those results are separate generated
//! artifacts.
//! CLR-03 package locks are also orchestration artifacts: `npa-package` parses
//! and serializes their canonical JSON identity data, but proof acceptance still
//! depends on canonical certificate bytes and checker verdicts.
//!
//! CLI implementers should use the structured error API instead of parsing
//! [`std::fmt::Display`] strings. [`PackageManifestError`] exposes stable
//! [`PackageManifestErrorKind`], [`PackageManifestErrorReason`], manifest path,
//! field, expected value, and actual value fields. [`validate_manifest_source_report`]
//! wraps the same deterministic pass ordering in a report-style API for callers
//! that want to present validation diagnostics without treating display text as
//! a contract.
//!
//! # Minimal manifest example
//!
//! ```rust
//! use npa_package::{parse_and_validate_manifest_str, PACKAGE_MANIFEST_SCHEMA};
//!
//! let source = format!(
//!     r#"schema = "{PACKAGE_MANIFEST_SCHEMA}"
//! package = "fixture-minimal"
//! version = "0.1.0"
//! core_spec = "npa.core.v0.1"
//! kernel_profile = "npa.kernel.v0.1"
//! certificate_format = "npa.certificate.canonical.v0.1"
//! checker_profile = "npa.checker.reference.v0.1"
//!
//! [policy]
//! allow_custom_axioms = false
//! allowed_axioms = []
//!
//! [[modules]]
//! module = "Fixture.Minimal"
//! source = "Fixture/Minimal/source.npa"
//! certificate = "Fixture/Minimal/certificate.npcert"
//! imports = []
//! expected_source_hash = "sha256:0000000000000000000000000000000000000000000000000000000000000000"
//! expected_certificate_file_hash = "sha256:1111111111111111111111111111111111111111111111111111111111111111"
//! expected_export_hash = "sha256:2222222222222222222222222222222222222222222222222222222222222222"
//! expected_axiom_report_hash = "sha256:3333333333333333333333333333333333333333333333333333333333333333"
//! expected_certificate_hash = "sha256:4444444444444444444444444444444444444444444444444444444444444444"
//! definitions = []
//! theorems = ["id"]
//! axioms = []
//! "#
//! );
//!
//! let validated = parse_and_validate_manifest_str(&source)?;
//! assert_eq!(validated.manifest().package.as_str(), "fixture-minimal");
//! assert_eq!(validated.graph().topological_order, vec![0]);
//! # Ok::<(), npa_package::PackageManifestError>(())
//! ```
//!
//! # Structured errors for CLI diagnostics
//!
//! ```rust
//! use npa_package::{
//!     validate_manifest_source_report, PackageManifestErrorKind,
//!     PackageManifestErrorReason,
//! };
//!
//! let report = validate_manifest_source_report(
//!     r#"schema = "npa.package.v0.1"
//! trusted_status = "verified_by_certificate"
//! "#,
//! );
//! let error = report.first_error().unwrap();
//! assert_eq!(error.kind, PackageManifestErrorKind::Schema);
//! assert_eq!(error.reason_code, PackageManifestErrorReason::UnknownField);
//! assert_eq!(error.path, "$");
//! assert_eq!(error.field.as_deref(), Some("trusted_status"));
//! ```

#![deny(missing_docs)]

pub mod error;
pub mod graph;
pub mod hash;
mod json;
pub mod lock;
pub mod manifest;
pub mod name;
pub mod path;
pub mod schema;
pub mod validate;

pub use error::{
    PackageLockError, PackageLockErrorKind, PackageLockErrorReason, PackageLockResult,
    PackageManifestError, PackageManifestErrorKind, PackageManifestErrorReason,
    PackageManifestResult,
};
pub use graph::{
    resolve_package_graph, PackageGraph, ResolvedModuleImport, ResolvedModuleImportKind,
};
pub use hash::{format_package_hash, parse_package_hash, PackageHash, PackageHashBytes};
pub use lock::{
    parse_package_lock_json, validate_package_lock_manifest, PackageLockEntry,
    PackageLockEntryOrigin, PackageLockImport, PackageLockManifest, PackageLockManifestReference,
};
pub use manifest::{
    parse_manifest_str, PackageExternalImport, PackageManifest, PackageModule, PackagePolicy,
    PackageVersion,
};
pub use name::{
    validate_canonical_axiom_name, validate_canonical_declaration_name,
    validate_canonical_module_name, validate_package_id, PackageId,
};
pub use path::{validate_package_path, PackagePath};
pub use schema::{
    CERTIFICATE_FORMAT_CANONICAL_V0_1, CHECKER_PROFILE_REFERENCE_V0_1, CORE_SPEC_V0_1,
    KERNEL_PROFILE_V0_1, PACKAGE_AXIOM_REPORT_SCHEMA, PACKAGE_LOCK_SCHEMA, PACKAGE_MANIFEST_SCHEMA,
    PACKAGE_PUBLISH_PLAN_SCHEMA, PACKAGE_THEOREM_INDEX_SCHEMA, REGISTRY_MODULE_SCHEMA,
};
pub use validate::{
    parse_and_validate_manifest_str, validate_manifest, validate_manifest_report,
    validate_manifest_source_report, validate_manifest_with_options, validate_package_version,
    PackageManifestValidationOptions, PackageManifestValidationReport, ValidatedPackageManifest,
};
