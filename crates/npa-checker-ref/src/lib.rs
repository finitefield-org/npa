//! Small reference-checker API boundary for Phase 8.
//!
//! This crate is intentionally independent from `npa-api`, `npa-tactic`,
//! `npa-frontend`, and the fast `npa-cert` verifier. The public entry point
//! accepts only canonical certificate bytes, an import store, and a checker
//! policy. It cannot receive `.npa` source, tactic scripts, AI traces, or a
//! theorem-search index.
//!
//! P8H-01 fixes the crate/API skeleton and deterministic error surface. Later
//! milestones fill in the source-free decoder, hash verifier, type checker,
//! conversion checker, inductive checker, and axiom report recomputation.

#![deny(missing_docs)]
#![forbid(unsafe_code)]

/// Canonical certificate format tag accepted by the reference checker.
pub const REFERENCE_CERTIFICATE_FORMAT: &str = "NPA-CERT-0.1";

/// Canonical core spec tag accepted by the reference checker.
pub const REFERENCE_CORE_SPEC: &str = "NPA-Core-0.1";

/// A SHA-256 hash stored in canonical certificate-facing artifacts.
pub type ReferenceHash = [u8; 32];

/// Certificate-only import environment supplied to the reference checker.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReferenceImportStore {
    /// Verified import entries available to the checked certificate.
    pub imports: Vec<ReferenceImportEntry>,
}

/// One import entry available to a reference checker run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceImportEntry {
    /// Imported module name.
    pub module: ReferenceModuleName,
    /// Expected export hash for the imported module.
    pub export_hash: ReferenceHash,
    /// Expected certificate hash for high-trust import checks.
    pub certificate_hash: Option<ReferenceHash>,
}

/// Canonical dotted module or declaration name.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReferenceModuleName {
    components: Vec<String>,
}

impl ReferenceModuleName {
    /// Builds a module name from already separated canonical components.
    pub fn new(components: Vec<String>) -> Result<Self, ReferenceNameError> {
        if components.is_empty() {
            return Err(ReferenceNameError::Empty);
        }
        if let Some(index) = components.iter().position(|component| component.is_empty()) {
            return Err(ReferenceNameError::EmptyComponent { index });
        }
        Ok(Self { components })
    }

    /// Builds a module name from a dotted string.
    pub fn from_dotted(value: &str) -> Result<Self, ReferenceNameError> {
        Self::new(value.split('.').map(str::to_owned).collect())
    }

    /// Returns the canonical name components.
    pub fn components(&self) -> &[String] {
        &self.components
    }

    /// Returns the dotted display form.
    pub fn dotted(&self) -> String {
        self.components.join(".")
    }
}

/// Structured error for invalid reference-checker names.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceNameError {
    /// The name has no components.
    Empty,
    /// One component is empty.
    EmptyComponent {
        /// Component index that was empty.
        index: usize,
    },
}

/// Deterministic policy input for the reference checker.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceCheckerPolicy {
    /// Trust mode for import and axiom policy gates.
    pub trust_mode: ReferenceTrustMode,
    /// Exact axiom names allowed by the policy.
    pub allowed_axioms: Vec<String>,
}

impl Default for ReferenceCheckerPolicy {
    fn default() -> Self {
        Self {
            trust_mode: ReferenceTrustMode::Normal,
            allowed_axioms: Vec::new(),
        }
    }
}

/// Reference checker trust mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceTrustMode {
    /// Normal certificate check mode.
    Normal,
    /// High-trust mode requiring certificate hashes for imports.
    HighTrust,
}

/// Result returned by [`check_certificate`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReferenceCheckResult {
    /// The certificate was accepted by the reference checker.
    Checked(ReferenceCheckedModule),
    /// The certificate was rejected with a deterministic structured error.
    Rejected(ReferenceCheckError),
}

impl ReferenceCheckResult {
    /// Returns true when the certificate was checked and accepted.
    pub const fn is_checked(&self) -> bool {
        matches!(self, Self::Checked(_))
    }

    /// Returns the rejection error, if any.
    pub const fn error(&self) -> Option<&ReferenceCheckError> {
        match self {
            Self::Checked(_) => None,
            Self::Rejected(error) => Some(error),
        }
    }
}

/// Accepted module summary produced by the reference checker.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceCheckedModule {
    /// Checked module name.
    pub module: ReferenceModuleName,
    /// Recomputed export hash.
    pub export_hash: ReferenceHash,
    /// Recomputed axiom report hash.
    pub axiom_report_hash: ReferenceHash,
    /// Recomputed certificate hash.
    pub certificate_hash: ReferenceHash,
}

/// Deterministic structured reference checker error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceCheckError {
    /// Stable machine-readable error kind.
    pub kind: ReferenceCheckErrorKind,
    /// Certificate section where the error was detected.
    pub section: ReferenceCertificateSection,
    /// Byte offset where the error was detected.
    pub offset: usize,
    /// Optional stable reason code for this error.
    pub reason: Option<ReferenceCheckReason>,
}

impl ReferenceCheckError {
    fn empty() -> Self {
        Self {
            kind: ReferenceCheckErrorKind::EmptyCertificate,
            section: ReferenceCertificateSection::HeaderFormat,
            offset: 0,
            reason: None,
        }
    }

    fn malformed(
        section: ReferenceCertificateSection,
        offset: usize,
        reason: ReferenceCheckReason,
    ) -> Self {
        Self {
            kind: ReferenceCheckErrorKind::MalformedCertificate,
            section,
            offset,
            reason: Some(reason),
        }
    }

    fn unsupported(offset: usize) -> Self {
        Self {
            kind: ReferenceCheckErrorKind::UnsupportedSkeleton,
            section: ReferenceCertificateSection::FullCertificate,
            offset,
            reason: Some(ReferenceCheckReason::ReferenceCheckerBodyUnimplemented),
        }
    }
}

/// Stable top-level reference checker error kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceCheckErrorKind {
    /// The certificate byte input was empty.
    EmptyCertificate,
    /// The certificate prefix was malformed.
    MalformedCertificate,
    /// The P8H-01 skeleton parsed the header but has no checker body yet.
    UnsupportedSkeleton,
}

/// Stable certificate section label for diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceCertificateSection {
    /// Certificate format string in the header.
    HeaderFormat,
    /// Core spec string in the header.
    HeaderCoreSpec,
    /// Module name in the header.
    HeaderModule,
    /// Full certificate body after the header.
    FullCertificate,
}

/// Stable reason code for malformed or unsupported certificates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceCheckReason {
    /// A varint ended after the input ended.
    UnexpectedEof,
    /// A varint used a noncanonical byte sequence.
    NonCanonicalUvar,
    /// A varint exceeded the supported `u64` range.
    UvarOverflow,
    /// A length did not fit into the host `usize`.
    LengthOverflow,
    /// A string was not valid UTF-8.
    InvalidUtf8,
    /// The certificate format tag was not `NPA-CERT-0.1`.
    FormatMismatch,
    /// The core spec tag was not `NPA-Core-0.1`.
    CoreSpecMismatch,
    /// The module name had no components.
    EmptyModuleName,
    /// A module name component was empty.
    EmptyModuleNameComponent,
    /// The P8H-01 skeleton intentionally has no checker body.
    ReferenceCheckerBodyUnimplemented,
}

/// Check a canonical certificate with the Phase 8 reference-checker API.
///
/// The P8H-01 implementation fixes the public boundary and deterministic
/// errors only. It does not call the fast Rust kernel or
/// `npa_cert::verify_module_cert`, and it does not yet accept certificates as
/// checked.
pub fn check_certificate(
    cert_bytes: &[u8],
    import_store: &ReferenceImportStore,
    policy: &ReferenceCheckerPolicy,
) -> ReferenceCheckResult {
    let _ = (import_store, policy);
    if cert_bytes.is_empty() {
        return ReferenceCheckResult::Rejected(ReferenceCheckError::empty());
    }

    let mut decoder = ReferencePrefixDecoder::new(cert_bytes);
    match decoder.header() {
        Ok(()) => ReferenceCheckResult::Rejected(ReferenceCheckError::unsupported(decoder.offset)),
        Err(error) => ReferenceCheckResult::Rejected(error),
    }
}

struct ReferencePrefixDecoder<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> ReferencePrefixDecoder<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn header(&mut self) -> Result<(), ReferenceCheckError> {
        let format = self.string(ReferenceCertificateSection::HeaderFormat)?;
        if format != REFERENCE_CERTIFICATE_FORMAT {
            return Err(ReferenceCheckError::malformed(
                ReferenceCertificateSection::HeaderFormat,
                self.offset,
                ReferenceCheckReason::FormatMismatch,
            ));
        }

        let core_spec = self.string(ReferenceCertificateSection::HeaderCoreSpec)?;
        if core_spec != REFERENCE_CORE_SPEC {
            return Err(ReferenceCheckError::malformed(
                ReferenceCertificateSection::HeaderCoreSpec,
                self.offset,
                ReferenceCheckReason::CoreSpecMismatch,
            ));
        }

        let component_count = self.usize(ReferenceCertificateSection::HeaderModule)?;
        if component_count == 0 {
            return Err(ReferenceCheckError::malformed(
                ReferenceCertificateSection::HeaderModule,
                self.offset,
                ReferenceCheckReason::EmptyModuleName,
            ));
        }
        for _ in 0..component_count {
            let component = self.string(ReferenceCertificateSection::HeaderModule)?;
            if component.is_empty() {
                return Err(ReferenceCheckError::malformed(
                    ReferenceCertificateSection::HeaderModule,
                    self.offset,
                    ReferenceCheckReason::EmptyModuleNameComponent,
                ));
            }
        }
        Ok(())
    }

    fn string(
        &mut self,
        section: ReferenceCertificateSection,
    ) -> Result<String, ReferenceCheckError> {
        let len = self.usize(section)?;
        let start = self.offset;
        let end = start.checked_add(len).ok_or_else(|| {
            ReferenceCheckError::malformed(
                section,
                self.offset,
                ReferenceCheckReason::LengthOverflow,
            )
        })?;
        let bytes = self.bytes.get(start..end).ok_or_else(|| {
            ReferenceCheckError::malformed(
                section,
                self.offset,
                ReferenceCheckReason::UnexpectedEof,
            )
        })?;
        self.offset = end;
        String::from_utf8(bytes.to_vec()).map_err(|_| {
            ReferenceCheckError::malformed(section, start, ReferenceCheckReason::InvalidUtf8)
        })
    }

    fn usize(
        &mut self,
        section: ReferenceCertificateSection,
    ) -> Result<usize, ReferenceCheckError> {
        let value = self.uvar(section)?;
        usize::try_from(value).map_err(|_| {
            ReferenceCheckError::malformed(
                section,
                self.offset,
                ReferenceCheckReason::LengthOverflow,
            )
        })
    }

    fn uvar(&mut self, section: ReferenceCertificateSection) -> Result<u64, ReferenceCheckError> {
        let start = self.offset;
        let mut shift = 0u32;
        let mut value = 0u64;
        loop {
            let byte = *self.bytes.get(self.offset).ok_or_else(|| {
                ReferenceCheckError::malformed(
                    section,
                    self.offset,
                    ReferenceCheckReason::UnexpectedEof,
                )
            })?;
            self.offset += 1;
            value |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                if encode_uvar(value) != self.bytes[start..self.offset] {
                    return Err(ReferenceCheckError::malformed(
                        section,
                        start,
                        ReferenceCheckReason::NonCanonicalUvar,
                    ));
                }
                return Ok(value);
            }
            shift += 7;
            if shift >= 64 {
                return Err(ReferenceCheckError::malformed(
                    section,
                    start,
                    ReferenceCheckReason::UvarOverflow,
                ));
            }
        }
    }
}

fn encode_uvar(mut value: u64) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if value == 0 {
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode_string(out: &mut Vec<u8>, value: &str) {
        out.extend(encode_uvar(value.len() as u64));
        out.extend(value.as_bytes());
    }

    fn header_bytes() -> Vec<u8> {
        let mut bytes = Vec::new();
        encode_string(&mut bytes, REFERENCE_CERTIFICATE_FORMAT);
        encode_string(&mut bytes, REFERENCE_CORE_SPEC);
        bytes.extend(encode_uvar(2));
        encode_string(&mut bytes, "Std");
        encode_string(&mut bytes, "Nat");
        bytes
    }

    #[test]
    fn public_api_is_certificate_bytes_import_store_and_policy_only() {
        let _: fn(&[u8], &ReferenceImportStore, &ReferenceCheckerPolicy) -> ReferenceCheckResult =
            check_certificate;
        let imports = ReferenceImportStore::default();
        let policy = ReferenceCheckerPolicy::default();

        let result = check_certificate(&[], &imports, &policy);

        assert_eq!(
            result,
            ReferenceCheckResult::Rejected(ReferenceCheckError {
                kind: ReferenceCheckErrorKind::EmptyCertificate,
                section: ReferenceCertificateSection::HeaderFormat,
                offset: 0,
                reason: None,
            })
        );
    }

    #[test]
    fn empty_certificate_returns_deterministic_structured_error() {
        let imports = ReferenceImportStore::default();
        let policy = ReferenceCheckerPolicy::default();

        let first = check_certificate(&[], &imports, &policy);
        let second = check_certificate(&[], &imports, &policy);

        assert_eq!(first, second);
        assert_eq!(
            first.error().unwrap().kind,
            ReferenceCheckErrorKind::EmptyCertificate
        );
    }

    #[test]
    fn malformed_certificate_returns_deterministic_structured_error() {
        let imports = ReferenceImportStore::default();
        let policy = ReferenceCheckerPolicy::default();
        let malformed = [0x00];

        let result = check_certificate(&malformed, &imports, &policy);

        assert_eq!(
            result,
            ReferenceCheckResult::Rejected(ReferenceCheckError {
                kind: ReferenceCheckErrorKind::MalformedCertificate,
                section: ReferenceCertificateSection::HeaderFormat,
                offset: 1,
                reason: Some(ReferenceCheckReason::FormatMismatch),
            })
        );
    }

    #[test]
    fn header_only_certificate_is_not_accepted_by_the_skeleton() {
        let imports = ReferenceImportStore::default();
        let policy = ReferenceCheckerPolicy::default();
        let mut cert = header_bytes();
        cert.extend([0x00, 0x01, 0x02]);

        let result = check_certificate(&cert, &imports, &policy);

        assert!(!result.is_checked());
        assert_eq!(
            result.error().unwrap().kind,
            ReferenceCheckErrorKind::UnsupportedSkeleton
        );
        assert_eq!(
            result.error().unwrap().reason,
            Some(ReferenceCheckReason::ReferenceCheckerBodyUnimplemented)
        );
    }

    #[test]
    fn invalid_utf8_header_is_structured_without_human_string_matching() {
        let imports = ReferenceImportStore::default();
        let policy = ReferenceCheckerPolicy::default();
        let malformed = [0x01, 0xff];

        let result = check_certificate(&malformed, &imports, &policy);

        assert_eq!(
            result.error().unwrap(),
            &ReferenceCheckError {
                kind: ReferenceCheckErrorKind::MalformedCertificate,
                section: ReferenceCertificateSection::HeaderFormat,
                offset: 1,
                reason: Some(ReferenceCheckReason::InvalidUtf8),
            }
        );
    }

    #[test]
    fn module_name_validation_is_structured() {
        assert_eq!(
            ReferenceModuleName::from_dotted(""),
            Err(ReferenceNameError::EmptyComponent { index: 0 })
        );
        assert_eq!(
            ReferenceModuleName::from_dotted("Std..Nat"),
            Err(ReferenceNameError::EmptyComponent { index: 1 })
        );

        let name = ReferenceModuleName::from_dotted("Std.Nat").unwrap();
        assert_eq!(name.components(), ["Std", "Nat"]);
        assert_eq!(name.dotted(), "Std.Nat");
    }
}
