//! Small reference-checker API boundary for Phase 8.
//!
//! This crate is intentionally independent from `npa-api`, `npa-tactic`,
//! `npa-frontend`, and the fast `npa-cert` verifier. The public entry point
//! accepts only canonical certificate bytes, an import store, and a checker
//! policy. It cannot receive `.npa` source, tactic scripts, AI traces, or a
//! theorem-search index.
//!
//! P8H-04 adds source-free import-store resolution and public import
//! environment construction. Later milestones fill in type checking,
//! conversion checking, inductive checking, and axiom report recomputation.

#![deny(missing_docs)]
#![forbid(unsafe_code)]

mod decode;

use std::collections::BTreeSet;

/// Canonical certificate format tag accepted by the reference checker.
pub const REFERENCE_CERTIFICATE_FORMAT: &str = "NPA-CERT-0.1";

/// Canonical core spec tag accepted by the reference checker.
pub const REFERENCE_CORE_SPEC: &str = "NPA-Core-0.1";

/// A SHA-256 hash stored in canonical certificate-facing artifacts.
pub type ReferenceHash = [u8; 32];

/// Certificate-only import environment supplied to the reference checker.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReferenceImportStore {
    entries: Vec<ReferenceImportEntry>,
}

impl ReferenceImportStore {
    /// Builds an import store from explicit source-free certificate bytes.
    ///
    /// The certificates are decoded and hash-verified, but they are not marked
    /// as high-trust checked modules because semantic checking is a later
    /// milestone. No filesystem, package discovery, network, or remote import
    /// lookup is performed.
    pub fn from_source_free_certificates<I, B>(certificates: I) -> Result<Self, ReferenceCheckError>
    where
        I: IntoIterator<Item = B>,
        B: AsRef<[u8]>,
    {
        let entries = certificates
            .into_iter()
            .map(|bytes| decode::import_entry_from_source_free_certificate_impl(bytes.as_ref()))
            .collect::<Result<Vec<_>, _>>()?;
        Self::from_entries(entries)
    }

    /// Builds an import store from modules already checked by this checker.
    pub fn from_checked_modules<I>(modules: I) -> Result<Self, ReferenceCheckError>
    where
        I: IntoIterator<Item = ReferenceCheckedModule>,
    {
        let entries = modules
            .into_iter()
            .map(ReferenceCheckedModule::into_import_entry)
            .collect();
        Self::from_entries(entries)
    }

    /// Returns the available import module interfaces.
    pub fn entries(&self) -> &[ReferenceImportEntry] {
        &self.entries
    }

    /// Returns the number of available import module interfaces.
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true when the store has no import module interfaces.
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn from_entries(entries: Vec<ReferenceImportEntry>) -> Result<Self, ReferenceCheckError> {
        validate_unique_import_store_entries(&entries)?;
        Ok(Self { entries })
    }
}

/// One import entry available to a reference checker run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceImportEntry {
    module: ReferenceModuleName,
    export_hash: ReferenceHash,
    axiom_report_hash: ReferenceHash,
    certificate_hash: ReferenceHash,
    public_environment: ReferencePublicEnvironment,
    checked_by_reference_checker: bool,
}

impl ReferenceImportEntry {
    pub(crate) fn new(
        module: ReferenceModuleName,
        export_hash: ReferenceHash,
        axiom_report_hash: ReferenceHash,
        certificate_hash: ReferenceHash,
        public_environment: ReferencePublicEnvironment,
        checked_by_reference_checker: bool,
    ) -> Self {
        Self {
            module,
            export_hash,
            axiom_report_hash,
            certificate_hash,
            public_environment,
            checked_by_reference_checker,
        }
    }

    /// Returns the imported module name.
    pub const fn module(&self) -> &ReferenceModuleName {
        &self.module
    }

    /// Returns the imported module export hash.
    pub const fn export_hash(&self) -> &ReferenceHash {
        &self.export_hash
    }

    /// Returns the imported module axiom-report hash.
    pub const fn axiom_report_hash(&self) -> &ReferenceHash {
        &self.axiom_report_hash
    }

    /// Returns the imported module certificate hash.
    pub const fn certificate_hash(&self) -> &ReferenceHash {
        &self.certificate_hash
    }

    /// Returns the imported module public environment.
    pub const fn public_environment(&self) -> &ReferencePublicEnvironment {
        &self.public_environment
    }

    /// Returns true when this module was checked by this reference checker.
    pub const fn checked_by_reference_checker(&self) -> bool {
        self.checked_by_reference_checker
    }
}

fn validate_unique_import_store_entries(
    entries: &[ReferenceImportEntry],
) -> Result<(), ReferenceCheckError> {
    let mut seen = BTreeSet::new();
    for (index, entry) in entries.iter().enumerate() {
        if !seen.insert((entry.module.clone(), entry.export_hash)) {
            return Err(ReferenceCheckError::import_resolution(
                ReferenceCertificateSection::ImportStore,
                index,
                ReferenceCheckReason::DuplicateImport,
            ));
        }
    }
    Ok(())
}

/// Public environment exported by one imported certificate.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReferencePublicEnvironment {
    exports: Vec<ReferencePublicExport>,
    module_axioms: Vec<ReferenceAxiomDependency>,
}

impl ReferencePublicEnvironment {
    pub(crate) fn new(
        exports: Vec<ReferencePublicExport>,
        module_axioms: Vec<ReferenceAxiomDependency>,
    ) -> Self {
        Self {
            exports,
            module_axioms,
        }
    }

    /// Returns public exports in canonical export-block order.
    pub fn exports(&self) -> &[ReferencePublicExport] {
        &self.exports
    }

    /// Returns module-level transitive axiom dependencies.
    pub fn module_axioms(&self) -> &[ReferenceAxiomDependency] {
        &self.module_axioms
    }
}

/// One declaration exported by an imported module.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferencePublicExport {
    /// Exported declaration name.
    pub name: ReferenceModuleName,
    /// Exported declaration kind.
    pub kind: ReferenceExportKind,
    /// Declaration interface hash.
    pub decl_interface_hash: ReferenceHash,
    /// Transitive axiom dependencies committed by this export.
    pub axiom_dependencies: Vec<ReferenceAxiomDependency>,
}

/// Kind of an imported public export.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceExportKind {
    /// Axiom export.
    Axiom,
    /// Definition export.
    Def,
    /// Theorem export.
    Theorem,
    /// Inductive type export.
    Inductive,
    /// Generated constructor export.
    Constructor,
    /// Generated recursor export.
    Recursor,
}

/// Axiom dependency carried by an imported public environment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceAxiomDependency {
    /// Axiom declaration name.
    pub name: ReferenceModuleName,
    /// Axiom declaration interface hash.
    pub decl_interface_hash: ReferenceHash,
}

/// Import environment resolved for the module currently being checked.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReferenceImportEnvironment {
    imports: Vec<ReferenceResolvedImport>,
}

impl ReferenceImportEnvironment {
    pub(crate) fn new(imports: Vec<ReferenceResolvedImport>) -> Self {
        Self { imports }
    }

    /// Returns resolved imports in the current certificate's canonical order.
    pub fn imports(&self) -> &[ReferenceResolvedImport] {
        &self.imports
    }

    /// Returns the number of resolved imports.
    pub const fn len(&self) -> usize {
        self.imports.len()
    }

    /// Returns true when no imports were resolved.
    pub const fn is_empty(&self) -> bool {
        self.imports.is_empty()
    }
}

/// One resolved import attached to the current module environment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReferenceResolvedImport {
    /// Imported module name.
    pub module: ReferenceModuleName,
    /// Resolved export hash.
    pub export_hash: ReferenceHash,
    /// Resolved certificate hash.
    pub certificate_hash: ReferenceHash,
    /// Imported public environment.
    pub public_environment: ReferencePublicEnvironment,
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

/// Hash role used in structured reference checker hash mismatch errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceHashObject {
    /// Declaration interface hash.
    DeclInterface,
    /// Declaration certificate hash.
    DeclCertificate,
    /// Export block hash.
    ExportBlock,
    /// Axiom report hash.
    AxiomReport,
    /// Full module certificate hash.
    ModuleCertificate,
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
    module: ReferenceModuleName,
    export_hash: ReferenceHash,
    axiom_report_hash: ReferenceHash,
    certificate_hash: ReferenceHash,
    public_environment: ReferencePublicEnvironment,
    checked_by_reference_checker: bool,
}

impl ReferenceCheckedModule {
    #[cfg(test)]
    pub(crate) fn from_import_entry(entry: ReferenceImportEntry) -> Self {
        Self {
            module: entry.module,
            export_hash: entry.export_hash,
            axiom_report_hash: entry.axiom_report_hash,
            certificate_hash: entry.certificate_hash,
            public_environment: entry.public_environment,
            checked_by_reference_checker: true,
        }
    }

    fn into_import_entry(self) -> ReferenceImportEntry {
        ReferenceImportEntry::new(
            self.module,
            self.export_hash,
            self.axiom_report_hash,
            self.certificate_hash,
            self.public_environment,
            self.checked_by_reference_checker,
        )
    }

    /// Returns the checked module name.
    pub const fn module(&self) -> &ReferenceModuleName {
        &self.module
    }

    /// Returns the checked module export hash.
    pub const fn export_hash(&self) -> &ReferenceHash {
        &self.export_hash
    }

    /// Returns the checked module axiom-report hash.
    pub const fn axiom_report_hash(&self) -> &ReferenceHash {
        &self.axiom_report_hash
    }

    /// Returns the checked module certificate hash.
    pub const fn certificate_hash(&self) -> &ReferenceHash {
        &self.certificate_hash
    }

    /// Returns the checked module public environment.
    pub const fn public_environment(&self) -> &ReferencePublicEnvironment {
        &self.public_environment
    }
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

    pub(crate) fn import_resolution(
        section: ReferenceCertificateSection,
        offset: usize,
        reason: ReferenceCheckReason,
    ) -> Self {
        Self {
            kind: ReferenceCheckErrorKind::ImportResolution,
            section,
            offset,
            reason: Some(reason),
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
    /// A stored hash did not match the reference checker recomputation.
    HashMismatch,
    /// Import store resolution or import policy failed.
    ImportResolution,
    /// The P8H-04 decoder/hash/import verifier accepted the bytes but semantic checking is pending.
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
    /// Explicit import store supplied to the checker.
    ImportStore,
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
    /// An import binding appeared more than once.
    DuplicateImport,
    /// A level table entry was not normalized.
    NonNormalizedLevel,
    /// A term table entry was not normalized.
    NonNormalizedTerm,
    /// A canonical table entry was not reachable from certificate roots.
    UnusedTableEntry,
    /// Extra bytes remained after the canonical certificate sections.
    TrailingBytes,
    /// A requested import module was not available in the explicit import store.
    MissingImport,
    /// An import module was present, but not with the requested export hash.
    ImportExportHashMismatch,
    /// High-trust mode required a certificate hash in the import entry.
    MissingImportCertificateHash,
    /// A present import certificate hash did not match the resolved import.
    ImportCertificateHashMismatch,
    /// High-trust mode rejected an import that was not checked by this checker.
    UncheckedImport,
    /// A stored hash did not match the reference checker recomputation.
    HashMismatch {
        /// Hash role that mismatched.
        object: ReferenceHashObject,
    },
    /// The P8H-03 decoder/hash verifier intentionally has no semantic checker body.
    ReferenceCheckerBodyUnimplemented,
}

impl ReferenceCheckError {
    pub(crate) fn hash_mismatch(
        section: ReferenceCertificateSection,
        offset: usize,
        object: ReferenceHashObject,
    ) -> Self {
        Self {
            kind: ReferenceCheckErrorKind::HashMismatch,
            section,
            offset,
            reason: Some(ReferenceCheckReason::HashMismatch { object }),
        }
    }
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

/// Decode and verify all stored canonical hashes without semantic checking.
///
/// This recomputes term, declaration, export, axiom-report, and full
/// certificate hashes inside the reference checker boundary. It does not resolve
/// imports, type check declarations, or validate any AI sidecar.
pub fn verify_certificate_hashes(
    cert_bytes: &[u8],
) -> Result<ReferenceDecodedCertificate, ReferenceCheckError> {
    decode::verify_certificate_hashes_impl(cert_bytes)
}

/// Decode, hash-verify, and resolve the current certificate's imports.
///
/// Import resolution only consults the explicit [`ReferenceImportStore`]. It
/// does not access the filesystem, discover package paths, use the network, or
/// fetch remote imports.
pub fn build_import_environment(
    cert_bytes: &[u8],
    import_store: &ReferenceImportStore,
    policy: &ReferenceCheckerPolicy,
) -> Result<ReferenceImportEnvironment, ReferenceCheckError> {
    if cert_bytes.is_empty() {
        return Err(ReferenceCheckError::empty());
    }
    decode::build_import_environment_impl(cert_bytes, import_store, policy)
}

/// Check a canonical certificate with the Phase 8 reference-checker API.
///
/// The P8H-04 implementation decodes canonical source-free certificate bytes,
/// verifies stored hashes, and resolves explicit imports. It intentionally does
/// not call the fast Rust kernel or `npa_cert::verify_module_cert`, and it does
/// not yet accept certificates as checked.
pub fn check_certificate(
    cert_bytes: &[u8],
    import_store: &ReferenceImportStore,
    policy: &ReferenceCheckerPolicy,
) -> ReferenceCheckResult {
    if cert_bytes.is_empty() {
        return ReferenceCheckResult::Rejected(ReferenceCheckError::empty());
    }

    match build_import_environment(cert_bytes, import_store, policy) {
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

    fn header_bytes_for(module: &[&str]) -> Vec<u8> {
        let mut bytes = Vec::new();
        encode_string(&mut bytes, REFERENCE_CERTIFICATE_FORMAT);
        encode_string(&mut bytes, REFERENCE_CORE_SPEC);
        encode_name(&mut bytes, module);
        bytes
    }

    fn header_bytes() -> Vec<u8> {
        header_bytes_for(&["Std", "Nat"])
    }

    fn hash_with_domain(domain: &[u8], payload: &[u8]) -> ReferenceHash {
        let mut hasher = Sha256::new();
        hasher.update(domain);
        hasher.update(payload);
        hasher.finalize().into()
    }

    fn encode_usize_vec(out: &mut Vec<u8>, values: &[usize]) {
        out.extend(encode_uvar(values.len() as u64));
        for value in values {
            out.extend(encode_uvar(*value as u64));
        }
    }

    fn encode_option_usize(out: &mut Vec<u8>, value: Option<usize>) {
        match value {
            Some(value) => {
                out.push(0x01);
                out.extend(encode_uvar(value as u64));
            }
            None => out.push(0x00),
        }
    }

    fn encode_option_hash(out: &mut Vec<u8>, value: Option<&ReferenceHash>) {
        match value {
            Some(value) => {
                out.push(0x01);
                out.extend(value);
            }
            None => out.push(0x00),
        }
    }

    fn encode_dependency_entries_empty(out: &mut Vec<u8>) {
        out.extend(encode_uvar(0));
    }

    fn encode_axiom_refs_empty(out: &mut Vec<u8>) {
        out.extend(encode_uvar(0));
    }

    fn encode_local_global_ref(out: &mut Vec<u8>, decl_index: usize) {
        out.push(0x01);
        out.extend(encode_uvar(decl_index as u64));
    }

    fn encode_axiom_refs_self(out: &mut Vec<u8>, decl_interface_hash: &ReferenceHash) {
        out.extend(encode_uvar(1));
        encode_local_global_ref(out, 0);
        out.extend(encode_uvar(0)); // name A
        out.extend(decl_interface_hash);
    }

    fn encode_axiom_refs_optional_self(
        out: &mut Vec<u8>,
        include_self: bool,
        decl_interface_hash: &ReferenceHash,
    ) {
        if include_self {
            encode_axiom_refs_self(out, decl_interface_hash);
        } else {
            encode_axiom_refs_empty(out);
        }
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

    fn empty_module_certificate_importing_std_nat(
        export_hash: ReferenceHash,
        certificate_hash: Option<ReferenceHash>,
    ) -> Vec<u8> {
        let mut bytes = header_bytes_for(&["Use", "Import"]);
        bytes.extend(encode_uvar(1)); // imports
        encode_name(&mut bytes, &["Std", "Nat"]);
        bytes.extend(export_hash);
        encode_option_hash(&mut bytes, certificate_hash.as_ref());
        bytes.extend(encode_uvar(2)); // name table
        encode_name(&mut bytes, &["Std", "Nat"]);
        encode_name(&mut bytes, &["Use", "Import"]);
        append_common_empty_suffix(&mut bytes);
        bytes
    }

    fn empty_module_certificate_importing_std_nat_twice(export_hash: ReferenceHash) -> Vec<u8> {
        let mut bytes = header_bytes_for(&["Use", "Import"]);
        bytes.extend(encode_uvar(2)); // imports
        for _ in 0..2 {
            encode_name(&mut bytes, &["Std", "Nat"]);
            bytes.extend(export_hash);
            encode_option_hash(&mut bytes, None);
        }
        bytes.extend(encode_uvar(2)); // name table
        encode_name(&mut bytes, &["Std", "Nat"]);
        encode_name(&mut bytes, &["Use", "Import"]);
        append_common_empty_suffix(&mut bytes);
        bytes
    }

    #[derive(Clone, Debug)]
    struct AxiomCertificateFixture {
        bytes: Vec<u8>,
        decl_interface_hash_offset: usize,
        decl_certificate_hash_offset: usize,
        export_hash_offset: usize,
        axiom_report_hash_offset: usize,
        certificate_hash_offset: usize,
        export_hash: ReferenceHash,
        axiom_report_hash: ReferenceHash,
        certificate_hash: ReferenceHash,
    }

    fn axiom_certificate_fixture() -> AxiomCertificateFixture {
        axiom_certificate_fixture_with_axiom_dependencies(false)
    }

    fn axiom_certificate_fixture_with_axiom_dependencies(
        include_self_axiom: bool,
    ) -> AxiomCertificateFixture {
        let mut bytes = header_bytes();
        bytes.extend(encode_uvar(0)); // imports
        bytes.extend(encode_uvar(2)); // name table: A, Std.Nat
        encode_name(&mut bytes, &["A"]);
        encode_name(&mut bytes, &["Std", "Nat"]);

        bytes.extend(encode_uvar(1)); // level table
        bytes.push(0x00); // Zero

        let level_hash = hash_with_domain(b"NPA-LEVEL-0.1", &[0x00]);
        let mut term_payload = Vec::new();
        term_payload.push(0x00); // Sort
        term_payload.extend(level_hash);
        let term_hash = hash_with_domain(b"NPA-TERM-0.1", &term_payload);

        bytes.extend(encode_uvar(1)); // term table
        bytes.push(0x00); // Sort
        bytes.extend(encode_uvar(0)); // level 0

        let mut iface_payload = Vec::new();
        iface_payload.push(0x00); // Axiom
        encode_name(&mut iface_payload, &["A"]);
        encode_usize_vec(&mut iface_payload, &[]);
        iface_payload.extend(term_hash);
        encode_dependency_entries_empty(&mut iface_payload);
        let decl_interface_hash = hash_with_domain(b"NPA-DECL-IFACE-0.1", &iface_payload);

        bytes.extend(encode_uvar(1)); // declarations
        bytes.push(0x00); // Axiom
        bytes.extend(encode_uvar(0)); // name A
        encode_usize_vec(&mut bytes, &[]); // universe params
        bytes.extend(encode_uvar(0)); // ty term
        encode_dependency_entries_empty(&mut bytes);
        encode_axiom_refs_optional_self(&mut bytes, include_self_axiom, &decl_interface_hash);

        let mut decl_cert_payload = Vec::new();
        decl_cert_payload.extend(decl_interface_hash);
        encode_axiom_refs_optional_self(
            &mut decl_cert_payload,
            include_self_axiom,
            &decl_interface_hash,
        );
        let decl_certificate_hash = hash_with_domain(b"NPA-DECL-CERT-0.1", &decl_cert_payload);

        let decl_interface_hash_offset = bytes.len();
        bytes.extend(decl_interface_hash);
        let decl_certificate_hash_offset = bytes.len();
        bytes.extend(decl_certificate_hash);

        let mut export_block = Vec::new();
        export_block.extend(encode_uvar(1));
        export_block.extend(encode_uvar(0)); // name A
        export_block.push(0x00); // Axiom export
        encode_usize_vec(&mut export_block, &[]);
        export_block.extend(encode_uvar(0)); // ty term
        encode_option_usize(&mut export_block, None);
        export_block.extend(term_hash);
        encode_option_hash(&mut export_block, None);
        export_block.push(0x00); // no reducibility
        export_block.push(0x00); // no opacity
        export_block.extend(decl_interface_hash);
        encode_axiom_refs_optional_self(
            &mut export_block,
            include_self_axiom,
            &decl_interface_hash,
        );

        let mut axiom_report = Vec::new();
        axiom_report.extend(encode_uvar(1));
        axiom_report.extend(encode_uvar(0)); // decl index
        encode_axiom_refs_optional_self(
            &mut axiom_report,
            include_self_axiom,
            &decl_interface_hash,
        );
        encode_axiom_refs_optional_self(
            &mut axiom_report,
            include_self_axiom,
            &decl_interface_hash,
        );
        encode_axiom_refs_optional_self(
            &mut axiom_report,
            include_self_axiom,
            &decl_interface_hash,
        ); // module axioms

        bytes.extend(&export_block);
        bytes.extend(&axiom_report);

        let export_hash = hash_with_domain(b"NPA-MODULE-EXPORT-0.1", &export_block);
        let axiom_report_hash = hash_with_domain(b"NPA-AXIOM-REPORT-0.1", &axiom_report);
        let export_hash_offset = bytes.len();
        bytes.extend(export_hash);
        let axiom_report_hash_offset = bytes.len();
        bytes.extend(axiom_report_hash);
        let certificate_hash_offset = bytes.len();
        let certificate_hash = hash_with_domain(b"NPA-MODULE-CERT-0.1", &bytes);
        bytes.extend(certificate_hash);

        AxiomCertificateFixture {
            bytes,
            decl_interface_hash_offset,
            decl_certificate_hash_offset,
            export_hash_offset,
            axiom_report_hash_offset,
            certificate_hash_offset,
            export_hash,
            axiom_report_hash,
            certificate_hash,
        }
    }

    fn assert_hash_mismatch(
        error: ReferenceCheckError,
        section: ReferenceCertificateSection,
        offset: usize,
        object: ReferenceHashObject,
    ) {
        assert_eq!(error.kind, ReferenceCheckErrorKind::HashMismatch);
        assert_eq!(error.section, section);
        assert_eq!(error.offset, offset);
        assert_eq!(
            error.reason,
            Some(ReferenceCheckReason::HashMismatch { object })
        );
    }

    fn assert_import_resolution(error: ReferenceCheckError, reason: ReferenceCheckReason) {
        assert_eq!(error.kind, ReferenceCheckErrorKind::ImportResolution);
        assert_eq!(error.section, ReferenceCertificateSection::Imports);
        assert_eq!(error.reason, Some(reason));
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
    fn hash_verifier_accepts_golden_axiom_certificate_without_source_sections() {
        let fixture = axiom_certificate_fixture();

        let verified =
            verify_certificate_hashes(&fixture.bytes).expect("golden axiom certificate verifies");

        assert_eq!(verified.header().module.dotted(), "Std.Nat");
        assert_eq!(verified.declarations_len(), 1);
        assert_eq!(verified.hashes().export_hash, fixture.export_hash);
        assert_eq!(
            verified.hashes().axiom_report_hash,
            fixture.axiom_report_hash
        );
        assert_eq!(verified.hashes().certificate_hash, fixture.certificate_hash);
    }

    #[test]
    fn hash_verifier_rejects_decl_interface_hash_mismatch_by_object() {
        let fixture = axiom_certificate_fixture();
        let mut cert = fixture.bytes;
        cert[fixture.decl_interface_hash_offset] ^= 0x01;

        let error =
            verify_certificate_hashes(&cert).expect_err("decl interface hash mismatch rejects");

        assert_hash_mismatch(
            error,
            ReferenceCertificateSection::Declarations,
            fixture.decl_interface_hash_offset,
            ReferenceHashObject::DeclInterface,
        );
    }

    #[test]
    fn hash_verifier_rejects_decl_certificate_hash_mismatch_by_object() {
        let fixture = axiom_certificate_fixture();
        let mut cert = fixture.bytes;
        cert[fixture.decl_certificate_hash_offset] ^= 0x01;

        let error =
            verify_certificate_hashes(&cert).expect_err("decl certificate hash mismatch rejects");

        assert_hash_mismatch(
            error,
            ReferenceCertificateSection::Declarations,
            fixture.decl_certificate_hash_offset,
            ReferenceHashObject::DeclCertificate,
        );
    }

    #[test]
    fn hash_verifier_rejects_export_hash_mismatch_by_object() {
        let fixture = axiom_certificate_fixture();
        let mut cert = fixture.bytes;
        cert[fixture.export_hash_offset] ^= 0x01;

        let error = verify_certificate_hashes(&cert).expect_err("export hash mismatch rejects");

        assert_hash_mismatch(
            error,
            ReferenceCertificateSection::Hashes,
            fixture.export_hash_offset,
            ReferenceHashObject::ExportBlock,
        );
    }

    #[test]
    fn hash_verifier_rejects_axiom_report_hash_mismatch_by_object() {
        let fixture = axiom_certificate_fixture();
        let mut cert = fixture.bytes;
        cert[fixture.axiom_report_hash_offset] ^= 0x01;

        let error =
            verify_certificate_hashes(&cert).expect_err("axiom report hash mismatch rejects");

        assert_hash_mismatch(
            error,
            ReferenceCertificateSection::Hashes,
            fixture.axiom_report_hash_offset,
            ReferenceHashObject::AxiomReport,
        );
    }

    #[test]
    fn hash_verifier_rejects_certificate_hash_mismatch_by_object() {
        let fixture = axiom_certificate_fixture();
        let mut cert = fixture.bytes;
        cert[fixture.certificate_hash_offset] ^= 0x01;

        let error =
            verify_certificate_hashes(&cert).expect_err("certificate hash mismatch rejects");

        assert_hash_mismatch(
            error,
            ReferenceCertificateSection::Hashes,
            fixture.certificate_hash_offset,
            ReferenceHashObject::ModuleCertificate,
        );
    }

    #[test]
    fn check_certificate_runs_hash_verifier_before_semantic_skeleton() {
        let imports = ReferenceImportStore::default();
        let policy = ReferenceCheckerPolicy::default();
        let fixture = axiom_certificate_fixture();
        let mut cert = fixture.bytes;
        cert[fixture.certificate_hash_offset] ^= 0x01;

        let result = check_certificate(&cert, &imports, &policy);

        assert_hash_mismatch(
            result.error().unwrap().clone(),
            ReferenceCertificateSection::Hashes,
            fixture.certificate_hash_offset,
            ReferenceHashObject::ModuleCertificate,
        );
    }

    #[test]
    fn import_store_from_source_free_certificate_resolves_normal_mode_by_export_hash() {
        let fixture = axiom_certificate_fixture();
        let store = ReferenceImportStore::from_source_free_certificates([fixture.bytes.as_slice()])
            .expect("source-free import store builds");
        let policy = ReferenceCheckerPolicy::default();
        let cert = empty_module_certificate_importing_std_nat(fixture.export_hash, None);

        let env =
            build_import_environment(&cert, &store, &policy).expect("import environment resolves");

        assert_eq!(store.len(), 1);
        assert!(!store.entries()[0].checked_by_reference_checker());
        assert_eq!(env.len(), 1);
        assert_eq!(env.imports()[0].module.dotted(), "Std.Nat");
        assert_eq!(env.imports()[0].export_hash, fixture.export_hash);
        assert_eq!(env.imports()[0].public_environment.exports().len(), 1);
    }

    #[test]
    fn import_resolution_does_not_resolve_by_name_only() {
        let fixture = axiom_certificate_fixture();
        let store = ReferenceImportStore::from_source_free_certificates([fixture.bytes.as_slice()])
            .expect("source-free import store builds");
        let policy = ReferenceCheckerPolicy::default();
        let mut wrong_export_hash = fixture.export_hash;
        wrong_export_hash[0] ^= 0x01;
        let cert = empty_module_certificate_importing_std_nat(wrong_export_hash, None);

        let error = build_import_environment(&cert, &store, &policy)
            .expect_err("wrong export hash must reject");

        assert_import_resolution(error, ReferenceCheckReason::ImportExportHashMismatch);
    }

    #[test]
    fn import_resolution_rejects_missing_import_deterministically() {
        let fixture = axiom_certificate_fixture();
        let store = ReferenceImportStore::default();
        let policy = ReferenceCheckerPolicy::default();
        let cert = empty_module_certificate_importing_std_nat(fixture.export_hash, None);

        let first =
            build_import_environment(&cert, &store, &policy).expect_err("missing import rejects");
        let second = build_import_environment(&cert, &store, &policy)
            .expect_err("missing import rejects deterministically");

        assert_eq!(first, second);
        assert_import_resolution(first, ReferenceCheckReason::MissingImport);
    }

    #[test]
    fn normal_mode_rejects_present_import_certificate_hash_mismatch() {
        let fixture = axiom_certificate_fixture();
        let store = ReferenceImportStore::from_source_free_certificates([fixture.bytes.as_slice()])
            .expect("source-free import store builds");
        let policy = ReferenceCheckerPolicy::default();
        let mut wrong_certificate_hash = fixture.certificate_hash;
        wrong_certificate_hash[0] ^= 0x01;
        let cert = empty_module_certificate_importing_std_nat(
            fixture.export_hash,
            Some(wrong_certificate_hash),
        );

        let error = build_import_environment(&cert, &store, &policy)
            .expect_err("certificate hash mismatch rejects");

        assert_import_resolution(error, ReferenceCheckReason::ImportCertificateHashMismatch);
    }

    #[test]
    fn high_trust_rejects_unchecked_source_free_import() {
        let fixture = axiom_certificate_fixture();
        let store = ReferenceImportStore::from_source_free_certificates([fixture.bytes.as_slice()])
            .expect("source-free import store builds");
        let policy = ReferenceCheckerPolicy {
            trust_mode: ReferenceTrustMode::HighTrust,
            ..ReferenceCheckerPolicy::default()
        };
        let cert = empty_module_certificate_importing_std_nat(
            fixture.export_hash,
            Some(fixture.certificate_hash),
        );

        let error = build_import_environment(&cert, &store, &policy)
            .expect_err("unchecked high-trust import rejects");

        assert_import_resolution(error, ReferenceCheckReason::UncheckedImport);
    }

    #[test]
    fn high_trust_rejects_missing_import_certificate_hash() {
        let fixture = axiom_certificate_fixture();
        let unchecked_store =
            ReferenceImportStore::from_source_free_certificates([fixture.bytes.as_slice()])
                .expect("source-free import store builds");
        let checked =
            ReferenceCheckedModule::from_import_entry(unchecked_store.entries()[0].clone());
        let store = ReferenceImportStore::from_checked_modules([checked])
            .expect("checked import store builds");
        let policy = ReferenceCheckerPolicy {
            trust_mode: ReferenceTrustMode::HighTrust,
            ..ReferenceCheckerPolicy::default()
        };
        let cert = empty_module_certificate_importing_std_nat(fixture.export_hash, None);

        let error = build_import_environment(&cert, &store, &policy)
            .expect_err("missing high-trust certificate hash rejects");

        assert_import_resolution(error, ReferenceCheckReason::MissingImportCertificateHash);
    }

    #[test]
    fn high_trust_accepts_same_checker_checked_module_interface() {
        let fixture = axiom_certificate_fixture();
        let unchecked_store =
            ReferenceImportStore::from_source_free_certificates([fixture.bytes.as_slice()])
                .expect("source-free import store builds");
        let checked =
            ReferenceCheckedModule::from_import_entry(unchecked_store.entries()[0].clone());
        let store = ReferenceImportStore::from_checked_modules([checked])
            .expect("checked import store builds");
        let policy = ReferenceCheckerPolicy {
            trust_mode: ReferenceTrustMode::HighTrust,
            ..ReferenceCheckerPolicy::default()
        };
        let cert = empty_module_certificate_importing_std_nat(
            fixture.export_hash,
            Some(fixture.certificate_hash),
        );

        let env =
            build_import_environment(&cert, &store, &policy).expect("high-trust import resolves");

        assert_eq!(env.len(), 1);
        assert_eq!(env.imports()[0].certificate_hash, fixture.certificate_hash);
    }

    #[test]
    fn import_store_rejects_duplicate_import_bindings() {
        let fixture = axiom_certificate_fixture();

        let error = ReferenceImportStore::from_source_free_certificates([
            fixture.bytes.as_slice(),
            fixture.bytes.as_slice(),
        ])
        .expect_err("duplicate import store entries reject");

        assert_eq!(error.kind, ReferenceCheckErrorKind::ImportResolution);
        assert_eq!(error.section, ReferenceCertificateSection::ImportStore);
        assert_eq!(error.reason, Some(ReferenceCheckReason::DuplicateImport));
    }

    #[test]
    fn current_certificate_rejects_duplicate_import_bindings() {
        let fixture = axiom_certificate_fixture();
        let store = ReferenceImportStore::from_source_free_certificates([fixture.bytes.as_slice()])
            .expect("source-free import store builds");
        let policy = ReferenceCheckerPolicy::default();
        let cert = empty_module_certificate_importing_std_nat_twice(fixture.export_hash);

        let error = build_import_environment(&cert, &store, &policy)
            .expect_err("duplicate certificate imports reject");

        assert_import_resolution(error, ReferenceCheckReason::DuplicateImport);
    }

    #[test]
    fn resolved_import_environment_preserves_imported_axiom_dependencies() {
        let fixture = axiom_certificate_fixture_with_axiom_dependencies(true);
        let store = ReferenceImportStore::from_source_free_certificates([fixture.bytes.as_slice()])
            .expect("source-free import store builds");
        let policy = ReferenceCheckerPolicy::default();
        let cert = empty_module_certificate_importing_std_nat(fixture.export_hash, None);

        let env =
            build_import_environment(&cert, &store, &policy).expect("import environment resolves");
        let import_env = &env.imports()[0].public_environment;

        assert_eq!(import_env.exports().len(), 1);
        assert_eq!(import_env.exports()[0].axiom_dependencies.len(), 1);
        assert_eq!(import_env.module_axioms().len(), 1);
    }

    #[test]
    fn check_certificate_runs_import_resolution_before_semantic_skeleton() {
        let fixture = axiom_certificate_fixture();
        let store = ReferenceImportStore::default();
        let policy = ReferenceCheckerPolicy::default();
        let cert = empty_module_certificate_importing_std_nat(fixture.export_hash, None);

        let result = check_certificate(&cert, &store, &policy);

        assert_import_resolution(
            result.error().unwrap().clone(),
            ReferenceCheckReason::MissingImport,
        );
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
