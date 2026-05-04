use std::collections::{BTreeMap, BTreeSet};

use npa_kernel::{Decl, Reducibility};

/// SHA-256 digest used for canonical certificate objects.
pub type Hash = [u8; 32];

/// Index into a certificate name table.
pub type NameId = usize;

/// Index into a certificate level table.
pub type LevelId = usize;

/// Index into a certificate term table.
pub type TermId = usize;

/// Dotted module, declaration, or axiom name represented as canonical path components.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Name(
    /// Canonical name components.
    pub Vec<String>,
);

impl Name {
    /// Build a name from a dotted string, preserving empty path components for validation.
    pub fn from_dotted(name: impl AsRef<str>) -> Self {
        Self(name.as_ref().split('.').map(ToOwned::to_owned).collect())
    }

    /// Render the name as a dot-separated string.
    pub fn as_dotted(&self) -> String {
        self.0.join(".")
    }

    /// Return whether this name is canonical for trusted certificate payloads.
    pub fn is_canonical(&self) -> bool {
        !self.0.is_empty()
            && self
                .0
                .iter()
                .all(|component| !component.is_empty() && !component.contains('.'))
    }
}

/// Canonical module name.
pub type ModuleName = Name;

/// Canonical axiom name.
pub type AxiomName = Name;

/// Input module made of already elaborated kernel declarations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoreModule {
    /// Module name stored in the certificate header.
    pub name: ModuleName,
    /// Kernel declarations to canonicalize into certificate declarations.
    pub declarations: Vec<Decl>,
}

/// Import trust mode used by certificate verification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrustMode {
    /// Resolve imports by module and export hash; certificate hash may be omitted.
    Normal,
    /// Require imports to be verified in-session by module, export hash, and certificate hash.
    HighTrust,
}

/// Axiom admission policy enforced while verifying certificates and imports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AxiomPolicy {
    /// Import trust mode for the verification run.
    pub mode: TrustMode,
    /// Exact set of allowed axioms. In normal mode an empty set permits every non-sorry axiom.
    /// In high-trust mode every axiom must be allowlisted.
    pub allowlisted_axioms: BTreeSet<AxiomName>,
    /// Reject declarations that depend on `sorry`.
    pub deny_sorry: bool,
}

impl AxiomPolicy {
    /// Return the default normal-mode policy.
    pub fn normal() -> Self {
        Self {
            mode: TrustMode::Normal,
            allowlisted_axioms: BTreeSet::new(),
            deny_sorry: true,
        }
    }

    /// Return the default high-trust policy.
    pub fn high_trust() -> Self {
        Self {
            mode: TrustMode::HighTrust,
            allowlisted_axioms: BTreeSet::new(),
            deny_sorry: true,
        }
    }
}

/// Lookup key for a verified import inside a verifier session.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ImportKey {
    /// Imported module name.
    pub module: Name,
    /// Export hash required by the import entry.
    pub export_hash: Hash,
    /// Certificate hash required by high-trust imports.
    pub certificate_hash: Option<Hash>,
}

/// In-memory registry of modules already verified during this trust session.
#[derive(Clone, Debug, Default)]
pub struct VerifierSession {
    checked: BTreeMap<ImportKey, SessionEntry>,
}

#[derive(Clone, Debug)]
struct SessionEntry {
    module: VerifiedModule,
    mode: TrustMode,
}

impl VerifierSession {
    /// Create an empty verifier session.
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn insert_verified(&mut self, module: VerifiedModule, mode: TrustMode) {
        let key = ImportKey {
            module: module.module.clone(),
            export_hash: module.export_hash,
            certificate_hash: Some(module.certificate_hash),
        };
        let entry = SessionEntry { module, mode };
        match self.checked.get_mut(&key) {
            Some(existing) if existing.mode == TrustMode::HighTrust => {
                if mode == TrustMode::HighTrust {
                    *existing = entry;
                }
            }
            Some(existing) => *existing = entry,
            None => {
                self.checked.insert(key, entry);
            }
        }
    }

    pub(crate) fn find_import(
        &self,
        entry: &ImportEntry,
        mode: TrustMode,
    ) -> Result<&VerifiedModule> {
        let module_export_matches = self.checked.values().any(|checked| {
            checked.module.module == entry.module && checked.module.export_hash == entry.export_hash
        });
        let high_trust_module_export_matches = self.checked.values().any(|checked| {
            checked.mode == TrustMode::HighTrust
                && checked.module.module == entry.module
                && checked.module.export_hash == entry.export_hash
        });

        let found = self.checked.values().find(|checked| {
            (mode == TrustMode::Normal || checked.mode == TrustMode::HighTrust)
                && checked.module.module == entry.module
                && checked.module.export_hash == entry.export_hash
                && match (mode, entry.certificate_hash) {
                    (TrustMode::Normal, None) => true,
                    (_, Some(hash)) => checked.module.certificate_hash == hash,
                    (TrustMode::HighTrust, None) => false,
                }
        });

        if let Some(checked) = found {
            return Ok(&checked.module);
        }

        if mode == TrustMode::HighTrust && !high_trust_module_export_matches {
            return Err(CertError::ImportNotVerifiedInSession {
                module: entry.module.clone(),
            });
        }

        if entry.certificate_hash.is_some() && module_export_matches {
            return Err(CertError::ImportCertificateHashMismatch {
                module: entry.module.clone(),
            });
        }

        Err(CertError::ImportHashMismatch {
            module: entry.module.clone(),
        })
    }
}

/// Verified module payload that can be imported by later certificate verification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedModule {
    /// Module name from the verified certificate.
    pub module: Name,
    /// Canonical name table from the verified certificate.
    pub name_table: Vec<Name>,
    /// Canonical level table from the verified certificate.
    pub level_table: Vec<LevelNode>,
    /// Canonical term table from the verified certificate.
    pub term_table: Vec<TermNode>,
    /// Verified declaration certificates.
    pub declarations: Vec<DeclCert>,
    /// Module export hash used by downstream imports.
    pub export_hash: Hash,
    /// Full certificate hash used by high-trust imports.
    pub certificate_hash: Hash,
    /// Public export interface derived from declarations.
    pub export_block: ExportBlock,
    /// Axiom report recomputed during verification.
    pub axiom_report: AxiomReport,
}

/// Syntactic module certificate as represented after canonical binary decoding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleCert {
    /// Certificate format, core spec, and module identity.
    pub header: CertHeader,
    /// Canonical import list.
    pub imports: Vec<ImportEntry>,
    /// Canonical table of all names referenced by the certificate.
    pub name_table: Vec<Name>,
    /// Canonical DAG table of levels.
    pub level_table: Vec<LevelNode>,
    /// Canonical DAG table of core terms.
    pub term_table: Vec<TermNode>,
    /// Declaration certificates in canonical dependency order.
    pub declarations: Vec<DeclCert>,
    /// Public export interface derived from declarations.
    pub export_block: ExportBlock,
    /// Direct and transitive axiom dependencies.
    pub axiom_report: AxiomReport,
    /// Export, axiom-report, and full-certificate hashes.
    pub hashes: ModuleHashes,
}

/// Certificate header identifying the certificate and core specification versions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CertHeader {
    /// Certificate format version string.
    pub format: String,
    /// Core specification version string.
    pub core_spec: String,
    /// Module name carried by the certificate.
    pub module: Name,
}

/// Import dependency declared by a module certificate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportEntry {
    /// Imported module name.
    pub module: Name,
    /// Required export hash for the imported module.
    pub export_hash: Hash,
    /// Optional full certificate hash, mandatory in high-trust verification.
    pub certificate_hash: Option<Hash>,
}

/// Hashes committed by a module certificate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleHashes {
    /// Hash of the derived export block.
    pub export_hash: Hash,
    /// Hash of the derived axiom report.
    pub axiom_report_hash: Hash,
    /// Hash of the full certificate with this field zeroed.
    pub certificate_hash: Hash,
}

/// Canonical binary level node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LevelNode {
    /// Universe level zero.
    Zero,
    /// Successor of a previous level table entry.
    Succ(LevelId),
    /// Maximum of two previous level table entries.
    Max(LevelId, LevelId),
    /// Impredicative maximum of two previous level table entries.
    IMax(LevelId, LevelId),
    /// Universe parameter stored in the name table.
    Param(NameId),
}

/// Canonical binary core term node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TermNode {
    /// Sort at a level table entry.
    Sort(LevelId),
    /// De Bruijn bound variable.
    BVar(u32),
    /// Constant reference with universe instantiation.
    Const {
        /// Imported, local, or generated declaration reference.
        global_ref: GlobalRef,
        /// Universe level arguments.
        levels: Vec<LevelId>,
    },
    /// Application node.
    App(TermId, TermId),
    /// Lambda abstraction.
    Lam {
        /// Binder type.
        ty: TermId,
        /// Body under one additional binder.
        body: TermId,
    },
    /// Dependent function type.
    Pi {
        /// Binder type.
        ty: TermId,
        /// Body under one additional binder.
        body: TermId,
    },
    /// Let binding.
    Let {
        /// Bound value type.
        ty: TermId,
        /// Bound value.
        value: TermId,
        /// Body under one additional binder.
        body: TermId,
    },
}

/// Canonical declaration reference used by terms, dependencies, and axiom reports.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GlobalRef {
    /// Declaration exported by an imported module.
    Imported {
        /// Index into the import table.
        import_index: usize,
        /// Name table index for the imported declaration.
        name: NameId,
        /// Interface hash expected for the imported declaration.
        decl_interface_hash: Hash,
    },
    /// Local source declaration by declaration index.
    Local {
        /// Index into the local declaration table.
        decl_index: usize,
    },
    /// Local generated declaration such as an inductive constructor or recursor.
    LocalGenerated {
        /// Index of the source inductive declaration.
        decl_index: usize,
        /// Name table index for the generated declaration.
        name: NameId,
    },
}

/// Certificate data for one source declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclCert {
    /// Canonical declaration payload.
    pub decl: DeclPayload,
    /// Direct declaration dependencies with interface hashes.
    pub dependencies: Vec<DependencyEntry>,
    /// Transitive axiom dependencies for this declaration.
    pub axiom_dependencies: Vec<AxiomRef>,
    /// Declaration interface and certificate hashes.
    pub hashes: DeclHashes,
}

/// Canonical declaration payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeclPayload {
    /// Assumed axiom declaration.
    Axiom {
        /// Name table index of the declaration.
        name: NameId,
        /// Universe parameter name ids.
        universe_params: Vec<NameId>,
        /// Type term id.
        ty: TermId,
    },
    /// Definition declaration.
    Def {
        /// Name table index of the declaration.
        name: NameId,
        /// Universe parameter name ids.
        universe_params: Vec<NameId>,
        /// Type term id.
        ty: TermId,
        /// Value term id.
        value: TermId,
        /// Reducibility exported for downstream checking.
        reducibility: CertReducibility,
    },
    /// Opaque theorem declaration.
    Theorem {
        /// Name table index of the declaration.
        name: NameId,
        /// Universe parameter name ids.
        universe_params: Vec<NameId>,
        /// Proposition type term id.
        ty: TermId,
        /// Proof term id checked by the kernel but not exported as body.
        proof: TermId,
        /// Theorem opacity marker.
        opacity: Opacity,
    },
    /// Inductive declaration with generated constructors and optional recursor.
    Inductive {
        /// Name table index of the inductive declaration.
        name: NameId,
        /// Universe parameter name ids.
        universe_params: Vec<NameId>,
        /// Parameter telescope.
        params: Vec<BinderType>,
        /// Index telescope.
        indices: Vec<BinderType>,
        /// Result sort level.
        sort: LevelId,
        /// Generated constructor specifications.
        constructors: Vec<ConstructorSpec>,
        /// Generated recursor specification when present.
        recursor: Option<RecursorSpec>,
    },
}

/// Binder type in an inductive telescope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BinderType {
    /// Type term for the binder.
    pub ty: TermId,
}

/// Generated inductive constructor certificate entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstructorSpec {
    /// Constructor name table index.
    pub name: NameId,
    /// Constructor type term id.
    pub ty: TermId,
}

/// Generated inductive recursor certificate entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecursorSpec {
    /// Recursor name table index.
    pub name: NameId,
    /// Universe parameter name ids.
    pub universe_params: Vec<NameId>,
    /// Recursor type term id.
    pub ty: TermId,
    /// Recursor rule-shape metadata.
    pub rules: RecursorRulesSpec,
}

/// Canonical recursor rule-shape metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecursorRulesSpec {
    /// Index of the first minor premise argument.
    pub minor_start: usize,
    /// Index of the major premise argument.
    pub major_index: usize,
}

/// Reducibility exported by a definition certificate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CertReducibility {
    /// Definition body is transparent to downstream checking.
    Reducible,
    /// Definition body is opaque outside the local proof check.
    Opaque,
}

impl From<&Reducibility> for CertReducibility {
    fn from(value: &Reducibility) -> Self {
        match value {
            Reducibility::Reducible => Self::Reducible,
            Reducibility::Opaque => Self::Opaque,
        }
    }
}

impl From<CertReducibility> for Reducibility {
    fn from(value: CertReducibility) -> Self {
        match value {
            CertReducibility::Reducible => Self::Reducible,
            CertReducibility::Opaque => Self::Opaque,
        }
    }
}

/// Opacity marker for theorem exports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opacity {
    /// Theorem proofs are not exported as reducible bodies.
    Opaque,
}

/// Direct dependency on another declaration interface.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DependencyEntry {
    /// Referenced declaration.
    pub global_ref: GlobalRef,
    /// Expected interface hash for the referenced declaration.
    pub decl_interface_hash: Hash,
}

/// Canonical reference to an axiom dependency.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AxiomRef {
    /// Referenced axiom declaration.
    pub global_ref: GlobalRef,
    /// Axiom name table index.
    pub name: NameId,
    /// Expected interface hash for the axiom declaration.
    pub decl_interface_hash: Hash,
}

/// Hash pair associated with a declaration certificate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclHashes {
    /// Public interface hash for downstream imports and dependency checks.
    pub decl_interface_hash: Hash,
    /// Full declaration certificate hash.
    pub decl_certificate_hash: Hash,
}

/// Canonical public export entries for a verified module.
pub type ExportBlock = Vec<ExportEntry>;

/// One exported declaration interface.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportEntry {
    /// Exported name table index.
    pub name: NameId,
    /// Kind of exported declaration.
    pub kind: ExportKind,
    /// Universe parameter name ids.
    pub universe_params: Vec<NameId>,
    /// Exported type term id.
    pub ty: TermId,
    /// Optional exported body term id for transparent definitions.
    pub body: Option<TermId>,
    /// Structural hash of the exported type.
    pub type_hash: Hash,
    /// Structural hash of the exported body when present.
    pub body_hash: Option<Hash>,
    /// Reducibility metadata for definitions.
    pub reducibility: Option<CertReducibility>,
    /// Opacity metadata for theorems.
    pub opacity: Option<Opacity>,
    /// Interface hash of the exported declaration.
    pub decl_interface_hash: Hash,
    /// Transitive axiom dependencies for the export.
    pub axiom_dependencies: Vec<AxiomRef>,
}

/// Kind of an exported declaration interface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExportKind {
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

/// Module-level axiom dependency report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AxiomReport {
    /// Per-declaration axiom dependency reports.
    pub per_declaration: Vec<DeclAxiomReport>,
    /// Union of all transitive axiom dependencies in the module.
    pub module_axioms: Vec<AxiomRef>,
}

/// Axiom dependency report for a single declaration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeclAxiomReport {
    /// Declaration index in the certificate declaration table.
    pub decl_index: usize,
    /// Direct axioms referenced by this declaration.
    pub direct_axioms: Vec<AxiomRef>,
    /// Transitive axioms reachable from this declaration.
    pub transitive_axioms: Vec<AxiomRef>,
}

/// Hash role used in structured certificate hash mismatch errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HashObject {
    /// Level table hash.
    Level,
    /// Term table hash.
    Term,
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

/// Structured certificate construction, decoding, and verification error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CertError {
    /// Generic malformed binary or invalid table reference.
    DecodeError,
    /// Certificate format or core spec version is unsupported.
    UnsupportedFormat {
        /// Found certificate format.
        format: String,
        /// Found core spec version.
        core_spec: String,
    },
    /// Unknown canonical binary tag.
    UnsupportedEncoding {
        /// Unsupported byte tag.
        tag: u8,
    },
    /// Bytes decode but are not in canonical form.
    NonCanonicalEncoding {
        /// Object whose canonical encoding was violated.
        object: &'static str,
    },
    /// Recomputed hash did not match the committed value.
    HashMismatch {
        /// Hash role that mismatched.
        object: HashObject,
        /// Expected committed hash.
        expected: Hash,
        /// Recomputed actual hash.
        actual: Hash,
    },
    /// No verified import matched the required module/export hash.
    ImportHashMismatch {
        /// Imported module.
        module: ModuleName,
    },
    /// High-trust mode requires an import certificate hash.
    MissingImportCertificateHash {
        /// Imported module.
        module: ModuleName,
    },
    /// Import export hash matched but certificate hash differed.
    ImportCertificateHashMismatch {
        /// Imported module.
        module: ModuleName,
    },
    /// High-trust mode could not find the import in the current verifier session.
    ImportNotVerifiedInSession {
        /// Imported module.
        module: ModuleName,
    },
    /// Duplicate canonical declaration or generated name.
    DuplicateName {
        /// Duplicated name.
        name: ModuleName,
    },
    /// Referenced dependency could not be resolved.
    UnknownDependency {
        /// Unknown dependency name.
        name: ModuleName,
    },
    /// Source declarations contain a dependency cycle.
    DependencyCycle {
        /// Name participating in the cycle.
        name: ModuleName,
    },
    /// Certificate axiom report does not match recomputation.
    AxiomReportMismatch {
        /// Declaration whose report mismatched, or none for module-level mismatch.
        decl: Option<ModuleName>,
    },
    /// Axiom is not allowed by the active policy.
    ForbiddenAxiom {
        /// Forbidden axiom name.
        axiom: ModuleName,
    },
    /// `sorry` is denied by the active policy.
    SorryDenied {
        /// Denied axiom name.
        axiom: ModuleName,
    },
    /// Certificate input still contains an unresolved metavariable.
    UnresolvedMetavariable,
    /// De Bruijn index is out of scope.
    InvalidBVar {
        /// Invalid variable index.
        index: u32,
    },
    /// Inductive generated constructor or recursor payload is not derivable.
    InductiveGeneratedArtifactMismatch {
        /// Generated declaration name.
        name: ModuleName,
    },
    /// Inductive wrapper fields disagree with the checked inductive payload.
    InductiveWrapperMismatch {
        /// Inductive declaration name.
        name: ModuleName,
    },
    /// Underlying Rust kernel rejected a declaration.
    Kernel(npa_kernel::Error),
}

/// Result type returned by certificate APIs.
pub type Result<T> = std::result::Result<T, CertError>;

impl From<npa_kernel::Error> for CertError {
    fn from(value: npa_kernel::Error) -> Self {
        Self::Kernel(value)
    }
}
