//! Small reference-checker API boundary for Phase 8.
//!
//! This crate is intentionally independent from `npa-api`, `npa-tactic`,
//! `npa-frontend`, and the fast `npa-cert` verifier. The public entry point
//! accepts only canonical certificate bytes, an import store, and a checker
//! policy. It cannot receive `.npa` source, tactic scripts, AI traces, or a
//! theorem-search index.
//!
//! P8H-02 adds the source-free canonical certificate decoder. Later milestones
//! fill in hash verification, type checking, conversion checking, inductive
//! checking, and axiom report recomputation.

#![deny(missing_docs)]
#![forbid(unsafe_code)]

mod decode;

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
        if let Some(index) = components
            .iter()
            .position(|component| component.contains('.'))
        {
            return Err(ReferenceNameError::ComponentContainsDot { index });
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
    /// One component contains a dotted separator.
    ComponentContainsDot {
        /// Component index that contained a dot.
        index: usize,
    },
}

/// Decoded source-free canonical certificate.
///
/// This is intentionally an opaque boundary object for P8H-02. The reference
/// checker can inspect canonical certificate structure without accepting source
/// files, tactic scripts, AI sidecars, or semantic import resolution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceDecodedCertificate {
    header: ReferenceCertificateHeader,
    imports_len: usize,
    name_table_len: usize,
    level_table_len: usize,
    term_table_len: usize,
    declarations_len: usize,
    export_block_len: usize,
    hashes: ReferenceModuleHashes,
}

impl ReferenceDecodedCertificate {
    /// Returns the decoded certificate header.
    pub const fn header(&self) -> &ReferenceCertificateHeader {
        &self.header
    }

    /// Returns the number of decoded import entries.
    pub const fn imports_len(&self) -> usize {
        self.imports_len
    }

    /// Returns the number of decoded canonical name table entries.
    pub const fn name_table_len(&self) -> usize {
        self.name_table_len
    }

    /// Returns the number of decoded canonical level table entries.
    pub const fn level_table_len(&self) -> usize {
        self.level_table_len
    }

    /// Returns the number of decoded canonical term table entries.
    pub const fn term_table_len(&self) -> usize {
        self.term_table_len
    }

    /// Returns the number of decoded declaration certificates.
    pub const fn declarations_len(&self) -> usize {
        self.declarations_len
    }

    /// Returns the number of decoded export entries.
    pub const fn export_block_len(&self) -> usize {
        self.export_block_len
    }

    /// Returns the stored canonical module hashes.
    pub const fn hashes(&self) -> &ReferenceModuleHashes {
        &self.hashes
    }
}

/// Decoded source-free certificate header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceCertificateHeader {
    /// Canonical certificate format tag.
    pub format: String,
    /// Core specification version tag.
    pub core_spec: String,
    /// Certified module name.
    pub module: ReferenceModuleName,
}

/// Canonical hashes stored at the end of a decoded module certificate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferenceModuleHashes {
    /// Stored export hash.
    pub export_hash: ReferenceHash,
    /// Stored axiom report hash.
    pub axiom_report_hash: ReferenceHash,
    /// Stored certificate hash.
    pub certificate_hash: ReferenceHash,
}

impl ReferenceDecodedCertificate {
    /// Builds a decoded certificate summary from decoder-owned data.
    pub(crate) const fn new(
        header: ReferenceCertificateHeader,
        counts: ReferenceDecodedCertificateCounts,
        hashes: ReferenceModuleHashes,
    ) -> Self {
        Self {
            header,
            imports_len: counts.imports_len,
            name_table_len: counts.name_table_len,
            level_table_len: counts.level_table_len,
            term_table_len: counts.term_table_len,
            declarations_len: counts.declarations_len,
            export_block_len: counts.export_block_len,
            hashes,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReferenceDecodedCertificateCounts {
    imports_len: usize,
    name_table_len: usize,
    level_table_len: usize,
    term_table_len: usize,
    declarations_len: usize,
    export_block_len: usize,
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
    pub(crate) fn empty() -> Self {
        Self {
            kind: ReferenceCheckErrorKind::EmptyCertificate,
            section: ReferenceCertificateSection::HeaderFormat,
            offset: 0,
            reason: None,
        }
    }

    pub(crate) fn malformed(
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

    pub(crate) fn unsupported(offset: usize) -> Self {
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
    /// The certificate was malformed or not canonical.
    MalformedCertificate,
    /// The P8H-02 decoder accepted the bytes but semantic checking is pending.
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
    /// Import table.
    Imports,
    /// Canonical name table.
    NameTable,
    /// Canonical universe level table.
    LevelTable,
    /// Canonical term table.
    TermTable,
    /// Declaration certificate table.
    Declarations,
    /// Export block.
    ExportBlock,
    /// Axiom report block.
    AxiomReport,
    /// Final stored module hashes.
    Hashes,
    /// Whole certificate after section-level decoding.
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
    /// A tag byte is not defined by the canonical certificate format.
    UnknownTag {
        /// Unknown tag byte.
        tag: u8,
    },
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
    /// A canonical name component contained a dotted separator.
    DottedNameComponent,
    /// An index referenced a missing table entry.
    DanglingReference,
    /// A canonical table was not in strict canonical order.
    NonCanonicalOrder,
    /// A canonical name table contained duplicate entries.
    DuplicateName,
    /// A canonical declaration table contained duplicate names.
    DuplicateDeclarationName,
    /// A level table entry was not normalized.
    NonNormalizedLevel,
    /// A term table entry was not normalized.
    NonNormalizedTerm,
    /// A canonical table entry was not reachable from certificate roots.
    UnusedTableEntry,
    /// Extra bytes remained after the canonical certificate sections.
    TrailingBytes,
    /// The P8H-02 decoder intentionally has no semantic checker body.
    ReferenceCheckerBodyUnimplemented,
}

/// Decode a source-free canonical certificate without semantic checking.
///
/// This function accepts only `.npcert` canonical binary bytes. It validates
/// section order, known tags, canonical table shape, dangling references, and
/// table reachability. It does not resolve imports, type check declarations, or
/// validate any AI sidecar.
pub fn decode_certificate(
    cert_bytes: &[u8],
) -> Result<ReferenceDecodedCertificate, ReferenceCheckError> {
    decode::decode_certificate_impl(cert_bytes)
}

/// Check a canonical certificate with the Phase 8 reference-checker API.
///
/// The P8H-02 implementation decodes canonical source-free certificate bytes
/// but intentionally does not call the fast Rust kernel or
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

    match decode_certificate(cert_bytes) {
        Ok(_) => ReferenceCheckResult::Rejected(ReferenceCheckError::unsupported(cert_bytes.len())),
        Err(error) => ReferenceCheckResult::Rejected(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use sha2::{Digest, Sha256};

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

    fn encode_string(out: &mut Vec<u8>, value: &str) {
        out.extend(encode_uvar(value.len() as u64));
        out.extend(value.as_bytes());
    }

    fn encode_name(out: &mut Vec<u8>, components: &[&str]) {
        out.extend(encode_uvar(components.len() as u64));
        for component in components {
            encode_string(out, component);
        }
    }

    fn header_bytes() -> Vec<u8> {
        let mut bytes = Vec::new();
        encode_string(&mut bytes, REFERENCE_CERTIFICATE_FORMAT);
        encode_string(&mut bytes, REFERENCE_CORE_SPEC);
        encode_name(&mut bytes, &["Std", "Nat"]);
        bytes
    }

    fn hash_with_domain(domain: &[u8], payload: &[u8]) -> ReferenceHash {
        let mut hasher = Sha256::new();
        hasher.update(domain);
        hasher.update(payload);
        hasher.finalize().into()
    }

    fn append_common_empty_suffix(bytes: &mut Vec<u8>) {
        bytes.extend(encode_uvar(0)); // level table
        bytes.extend(encode_uvar(0)); // term table
        bytes.extend(encode_uvar(0)); // declarations
        let export_block = encode_uvar(0);
        bytes.extend(&export_block);
        let mut axiom_report = encode_uvar(0); // per-declaration entries
        axiom_report.extend(encode_uvar(0)); // module axioms
        bytes.extend(&axiom_report);
        let export_hash = hash_with_domain(b"NPA-MODULE-EXPORT-0.1", &export_block);
        let axiom_report_hash = hash_with_domain(b"NPA-AXIOM-REPORT-0.1", &axiom_report);
        bytes.extend(export_hash);
        bytes.extend(axiom_report_hash);
        let certificate_hash = hash_with_domain(b"NPA-MODULE-CERT-0.1", bytes);
        bytes.extend(certificate_hash);
    }

    fn empty_module_certificate() -> Vec<u8> {
        let mut bytes = header_bytes();
        bytes.extend(encode_uvar(0)); // imports
        bytes.extend(encode_uvar(1)); // name table contains the header module name
        encode_name(&mut bytes, &["Std", "Nat"]);
        append_common_empty_suffix(&mut bytes);
        bytes
    }

    fn certificate_with_name_table(names: &[&[&str]]) -> Vec<u8> {
        let mut bytes = header_bytes();
        bytes.extend(encode_uvar(0)); // imports
        bytes.extend(encode_uvar(names.len() as u64));
        for name in names {
            encode_name(&mut bytes, name);
        }
        append_common_empty_suffix(&mut bytes);
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
        let cert = empty_module_certificate();

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
    fn decode_valid_golden_certificate_without_source_sections() {
        let cert = empty_module_certificate();

        let decoded = decode_certificate(&cert).expect("minimal canonical certificate decodes");

        assert_eq!(decoded.header().format, REFERENCE_CERTIFICATE_FORMAT);
        assert_eq!(decoded.header().core_spec, REFERENCE_CORE_SPEC);
        assert_eq!(decoded.header().module.dotted(), "Std.Nat");
        assert_eq!(decoded.imports_len(), 0);
        assert_eq!(decoded.name_table_len(), 1);
        assert_eq!(decoded.level_table_len(), 0);
        assert_eq!(decoded.term_table_len(), 0);
        assert_eq!(decoded.declarations_len(), 0);
        assert_eq!(decoded.export_block_len(), 0);
        assert_ne!(decoded.hashes().certificate_hash, [0; 32]);
    }

    #[test]
    fn decode_rejects_noncanonical_uvar_with_section_and_offset() {
        let mut cert = header_bytes();
        let offset = cert.len();
        cert.extend([0x80, 0x00]);

        let error = decode_certificate(&cert).expect_err("noncanonical uvar must reject");

        assert_eq!(
            error,
            ReferenceCheckError {
                kind: ReferenceCheckErrorKind::MalformedCertificate,
                section: ReferenceCertificateSection::Imports,
                offset,
                reason: Some(ReferenceCheckReason::NonCanonicalUvar),
            }
        );
    }

    #[test]
    fn decode_rejects_unknown_level_tag() {
        let mut cert = header_bytes();
        cert.extend(encode_uvar(0)); // imports
        cert.extend(encode_uvar(1));
        encode_name(&mut cert, &["Std", "Nat"]);
        cert.extend(encode_uvar(1)); // one level entry
        let offset = cert.len();
        cert.push(0xff);

        let error = decode_certificate(&cert).expect_err("unknown tag must reject");

        assert_eq!(error.kind, ReferenceCheckErrorKind::MalformedCertificate);
        assert_eq!(error.section, ReferenceCertificateSection::LevelTable);
        assert_eq!(error.offset, offset);
        assert_eq!(
            error.reason,
            Some(ReferenceCheckReason::UnknownTag { tag: 0xff })
        );
    }

    #[test]
    fn decode_rejects_duplicate_name_table_entry() {
        let duplicate: &[&str] = &["Std", "Nat"];
        let cert = certificate_with_name_table(&[duplicate, duplicate]);

        let error = decode_certificate(&cert).expect_err("duplicate names must reject");

        assert_eq!(error.kind, ReferenceCheckErrorKind::MalformedCertificate);
        assert_eq!(error.section, ReferenceCertificateSection::NameTable);
        assert_eq!(error.reason, Some(ReferenceCheckReason::DuplicateName));
        assert!(error.offset > 0);
    }

    #[test]
    fn decode_rejects_unused_name_table_entry() {
        let cert = certificate_with_name_table(&[&["Std", "Nat"], &["ZZ"]]);

        let error = decode_certificate(&cert).expect_err("unused names must reject");

        assert_eq!(error.kind, ReferenceCheckErrorKind::MalformedCertificate);
        assert_eq!(error.section, ReferenceCertificateSection::NameTable);
        assert_eq!(error.reason, Some(ReferenceCheckReason::UnusedTableEntry));
        assert!(error.offset > 0);
    }

    #[test]
    fn decode_rejects_dangling_level_reference() {
        let mut cert = header_bytes();
        cert.extend(encode_uvar(0)); // imports
        cert.extend(encode_uvar(1)); // name table
        encode_name(&mut cert, &["Std", "Nat"]);
        cert.extend(encode_uvar(1)); // level table
        let offset = cert.len();
        cert.push(0x04); // Param
        cert.extend(encode_uvar(1)); // missing name id
        cert.extend(encode_uvar(0)); // term table
        cert.extend(encode_uvar(0)); // declarations
        cert.extend(encode_uvar(0)); // export block
        cert.extend(encode_uvar(0)); // axiom report per-declaration
        cert.extend(encode_uvar(0)); // module axioms
        cert.extend([0; 96]);

        let error = decode_certificate(&cert).expect_err("dangling level name must reject");

        assert_eq!(error.kind, ReferenceCheckErrorKind::MalformedCertificate);
        assert_eq!(error.section, ReferenceCertificateSection::LevelTable);
        assert_eq!(error.offset, offset);
        assert_eq!(error.reason, Some(ReferenceCheckReason::DanglingReference));
    }

    #[test]
    fn decode_rejects_non_normalized_level_entry() {
        let mut cert = header_bytes();
        cert.extend(encode_uvar(0)); // imports
        cert.extend(encode_uvar(2)); // name table
        encode_name(&mut cert, &["Std", "Nat"]);
        encode_name(&mut cert, &["u"]);
        cert.extend(encode_uvar(3)); // level table
        cert.push(0x00); // Zero
        cert.push(0x04); // Param u
        cert.extend(encode_uvar(1));
        let offset = cert.len();
        cert.push(0x02); // Max Zero u, normalizes to u
        cert.extend(encode_uvar(0));
        cert.extend(encode_uvar(1));
        cert.extend(encode_uvar(0)); // term table
        cert.extend(encode_uvar(0)); // declarations
        cert.extend(encode_uvar(0)); // export block
        cert.extend(encode_uvar(0)); // axiom report per-declaration
        cert.extend(encode_uvar(0)); // module axioms
        cert.extend([0; 96]);

        let error = decode_certificate(&cert).expect_err("non-normalized level must reject");

        assert_eq!(error.kind, ReferenceCheckErrorKind::MalformedCertificate);
        assert_eq!(error.section, ReferenceCertificateSection::LevelTable);
        assert_eq!(error.offset, offset);
        assert_eq!(error.reason, Some(ReferenceCheckReason::NonNormalizedLevel));
    }

    #[test]
    fn decode_rejects_trailing_bytes() {
        let mut cert = empty_module_certificate();
        let offset = cert.len();
        cert.push(0);

        let error = decode_certificate(&cert).expect_err("trailing bytes must reject");

        assert_eq!(
            error,
            ReferenceCheckError {
                kind: ReferenceCheckErrorKind::MalformedCertificate,
                section: ReferenceCertificateSection::FullCertificate,
                offset,
                reason: Some(ReferenceCheckReason::TrailingBytes),
            }
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
