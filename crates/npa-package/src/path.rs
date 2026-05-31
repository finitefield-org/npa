//! Package-relative path validation helpers.

/// Package-relative path string accepted by `npa.package.v0.1`.
///
/// Validation is lexical and package-relative. It does not require file
/// existence, resolve symlinks, fetch registries, or consult the network.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackagePath(pub String);

impl PackagePath {
    /// Build a package path wrapper from a path string.
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    /// Return the package-relative path string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
