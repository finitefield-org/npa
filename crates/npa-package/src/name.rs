//! Package name type adapters.

/// Canonical dotted name type shared with certificate artifacts.
pub type CanonicalPackageName = npa_cert::Name;

/// Package identity for `npa.package.v0.1` manifests.
///
/// Validation fixes the grammar to lowercase ASCII package ids beginning with a
/// letter and continuing with letters, digits, or hyphens.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageId(pub String);

impl PackageId {
    /// Build a package id wrapper from a package id string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Return the package id string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
