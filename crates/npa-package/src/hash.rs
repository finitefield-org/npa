//! Package hash type adapters.

/// SHA-256 digest type shared with canonical certificate artifacts.
pub type PackageHashBytes = npa_cert::Hash;

/// Parsed SHA-256 package hash digest.
///
/// Manifest text uses `sha256:<64 lowercase hex>` strings, but validated package
/// data stores the parsed digest bytes so later package logic does not trust or
/// compare display strings.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackageHash(pub PackageHashBytes);

impl PackageHash {
    /// Build a package hash from already parsed SHA-256 digest bytes.
    pub const fn new(digest: PackageHashBytes) -> Self {
        Self(digest)
    }

    /// Return the underlying SHA-256 digest bytes.
    pub const fn as_bytes(&self) -> &PackageHashBytes {
        &self.0
    }

    /// Consume this package hash and return its digest bytes.
    pub const fn into_bytes(self) -> PackageHashBytes {
        self.0
    }
}

impl From<PackageHashBytes> for PackageHash {
    fn from(digest: PackageHashBytes) -> Self {
        Self::new(digest)
    }
}
