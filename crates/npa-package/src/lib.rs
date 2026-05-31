//! Package manifest metadata parsing and validation for external NPA libraries.
//!
//! This crate is an untrusted package tooling layer. It may reject malformed
//! package metadata before build or verification commands run, but theorem
//! acceptance remains with canonical certificates, the Rust kernel verdict, and
//! source-free checker verdicts.

#![deny(missing_docs)]

pub mod error;
pub mod graph;
pub mod hash;
pub mod manifest;
pub mod name;
pub mod path;
pub mod schema;
pub mod validate;

pub use hash::{PackageHash, PackageHashBytes};
pub use manifest::{
    PackageExternalImport, PackageManifest, PackageModule, PackagePolicy, PackageVersion,
};
pub use name::PackageId;
pub use path::PackagePath;
pub use schema::{
    CERTIFICATE_FORMAT_CANONICAL_V0_1, CHECKER_PROFILE_REFERENCE_V0_1, CORE_SPEC_V0_1,
    KERNEL_PROFILE_V0_1, PACKAGE_AXIOM_REPORT_SCHEMA, PACKAGE_LOCK_SCHEMA, PACKAGE_MANIFEST_SCHEMA,
    PACKAGE_PUBLISH_PLAN_SCHEMA, PACKAGE_THEOREM_INDEX_SCHEMA, REGISTRY_MODULE_SCHEMA,
};
