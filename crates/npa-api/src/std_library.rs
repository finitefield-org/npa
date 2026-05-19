use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
};

use npa_cert::{
    decode_module_cert, verify_module_cert, AxiomPolicy, AxiomRef, CertError, DeclCert,
    DeclPayload, ExportEntry, ExportKind, GlobalRef, Hash, ImportEntry, ModuleCert, Name, TermId,
    TermNode, TrustMode, VerifiedModule, VerifierSession,
};
use npa_tactic::{EqFamilyRef, NatFamilyRef, RewriteDirection, SimpRuleRef};
use sha2::{Digest, Sha256};

use crate::{
    current::{encode_machine_axiom_ref_wire, MachineAxiomRefWire},
    json::{JsonDocument, JsonParseErrorKind, JsonValue, JsonValueKind},
    projection::VerifiedImportKey,
    search::MachineTheoremMode,
    types::{
        parse_fully_qualified_name_wire, parse_hash_string,
        parse_machine_surface_renderable_name_wire, parse_machine_universe_param_name,
        parse_module_name_wire, phase5_name_canonical_bytes, MachineWireGrammarError,
        KERNEL_CHECK_PROFILE_BUILTIN_NAT_EQ_REC,
    },
    validation::{parse_strict_u64_token, StrictUnsignedIntegerError},
};

const STD_LOGIC_PATH: &str = "Std/Logic.npcert";
const STD_NAT_PATH: &str = "Std/Nat.npcert";
const STD_LIST_PATH: &str = "Std/List.npcert";
const STD_ALGEBRA_BASIC_PATH: &str = "Std/Algebra/Basic.npcert";
const STD_MACHINE_RELEASE_JSON_PATH: &str = "Std.machine-release.json";
const STD_MACHINE_IMPORT_BUNDLES_JSON_PATH: &str = "Std.machine-import-bundles.json";
const STD_MACHINE_AXIOM_REPORT_JSON_PATH: &str = "Std.machine-axiom-report.json";
const STD_LIBRARY_PROTOCOL_VERSION: &str = "npa.stdlib-machine.v1";
const STD_LIBRARY_PROFILE_ID: &str = "npa.stdlib.mvp.v1";
const STD_CORE_SPEC_ID: &str = "core-spec-v0.1";
const STD_KERNEL_SEMANTICS_PROFILE_ID: &str = "npa-kernel.phase1.v0.1";
const STD_REDUCTION_PROFILE_ID: &str = "beta-delta-iota-zeta.v0.1";
const STD_UNIVERSE_PROFILE_ID: &str = "levels-imax-v0.1";
const STD_KERNEL_CHECK_PROFILE_BUILTIN_NONE: &str = "npa.kernel.v0.1.builtin-none";
const STD_KERNEL_BUILTIN_NONE_PROFILE_ID: &str = "builtin-none-v0.1";
const STD_KERNEL_BUILTIN_NAT_EQ_REC_PROFILE_ID: &str = "builtin-nat-eq-rec-v0.1";
const STD_CERTIFICATE_ENCODING: &str = "npa.certificate.canonical.v0.1.hex";
const STD_MODULE_ARTIFACT_TAG: &str = "npa.phase6.std-module-artifact.v1";
const STD_LIBRARY_RELEASE_TAG: &str = "npa.phase6.std-library-release.v1";
const STD_IMPORT_BUNDLE_TAG: &str = "npa.phase6.std-import-bundle.v1";
const STD_IMPORT_BUNDLE_SET_TAG: &str = "npa.phase6.std-import-bundle-set.v1";
const STD_TACTIC_OPTIONS_RECIPE_TAG: &str = "npa.phase6.std-tactic-options-recipe.v1";
const PHASE4_KERNEL_CHECK_PROFILE_TAG: &str = "npa.phase4.kernel-check-profile.v1";
const STD_AXIOM_REPORT_TAG: &str = "npa.phase6.std-axiom-report.v1";
const STD_THEOREM_INDEX_TAG: &str = "npa.phase6.std-theorem-index.v1";
const STD_GLOBAL_REF_TAG: &str = "npa.phase6.std-global-ref.v1";
const STD_GLOBAL_REF_VIEW_TAG: &str = "npa.phase6.std-global-ref-view.v1";
const PHASE5_AXIOM_REF_WIRE_TAG: &str = "npa.phase5.axiom-ref-wire.v1";
const STD_THEOREM_INDEX_PROFILE_ID: &str = "npa.stdlib.theorem-index.mvp.v1";
const STD_LOGIC_BUNDLE_ID: &str = "std.logic.mvp";
const STD_NAT_BUNDLE_ID: &str = "std.nat.mvp";
const STD_LIST_BUNDLE_ID: &str = "std.list.mvp";
const STD_ALGEBRA_BASIC_BUNDLE_ID: &str = "std.algebra-basic.mvp";
const STD_ALL_BUNDLE_ID: &str = "std.all.mvp";
const STD_LOGIC_RECIPE_ID: &str = "std.logic-basic";
const STD_NAT_RECIPE_ID: &str = "std.nat-simp";
const STD_LIST_RECIPE_ID: &str = "std.list-simp";
const STD_ALL_RECIPE_ID: &str = "std.all-simp";
const STD_MAX_SIMP_REWRITE_STEPS: u64 = 100;
const STD_MAX_OPEN_GOALS: u64 = 32;
const STD_MAX_METAS: u64 = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineStdModuleLocator {
    pub module: Name,
    pub relative_path: String,
}

impl MachineStdModuleLocator {
    pub fn new(module: Name, relative_path: impl Into<String>) -> Self {
        Self {
            module,
            relative_path: relative_path.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MachineStdLoadedRelease {
    modules: Vec<MachineStdLoadedModule>,
    module_index: BTreeMap<Name, usize>,
    verification_order: Vec<Name>,
}

impl MachineStdLoadedRelease {
    pub fn modules(&self) -> &[MachineStdLoadedModule] {
        &self.modules
    }

    pub fn module(&self, module: &Name) -> Option<&MachineStdLoadedModule> {
        self.module_index
            .get(module)
            .map(|index| &self.modules[*index])
    }

    pub fn verification_order(&self) -> &[Name] {
        &self.verification_order
    }
}

#[derive(Clone, Debug)]
pub struct MachineStdLoadedModule {
    pub module: Name,
    pub locator_path: String,
    pub resolved_path: PathBuf,
    pub certificate_bytes: Vec<u8>,
    pub certificate_bytes_hash: Hash,
    pub expected_export_hash: Hash,
    pub expected_certificate_hash: Hash,
    pub axiom_report_hash: Hash,
    pub imports: Vec<ImportEntry>,
    pub verified_module: VerifiedModule,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineStdLibraryRelease {
    pub protocol_version: String,
    pub library_profile_id: String,
    pub core_spec_id: String,
    pub kernel_semantics_profile_id: String,
    pub modules: Vec<MachineStdModuleArtifact>,
    pub import_bundles_hash: Hash,
    pub theorem_index_hash: Hash,
    pub simp_profiles_hash: Hash,
    pub rewrite_profiles_hash: Hash,
    pub axiom_report_hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineStdModuleArtifact {
    pub module: Name,
    pub expected_export_hash: Hash,
    pub expected_certificate_hash: Hash,
    pub certificate_encoding: String,
    pub certificate_bytes_hash: Hash,
    pub axiom_report_hash: Hash,
    pub public_export_count: u64,
    pub theorem_index_entry_count: u64,
    pub simp_rule_count: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineStdAxiomReport {
    pub library_profile_id: String,
    pub modules: Vec<MachineStdModuleAxiomReport>,
    pub axiom_report_hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineStdModuleAxiomReport {
    pub module: Name,
    pub export_hash: Hash,
    pub certificate_hash: Hash,
    pub module_axioms: Vec<MachineStdAxiomRef>,
    pub transitive_axioms: Vec<MachineStdAxiomRef>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineStdAxiomRef {
    pub module: Name,
    pub name: Name,
    pub export_hash: Hash,
    pub decl_interface_hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineStdTheoremIndex {
    pub index_profile_id: String,
    pub library_profile_id: String,
    pub entries: Vec<MachineStdTheoremEntry>,
    pub index_hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineStdTheoremEntry {
    pub global_ref: MachineStdGlobalRef,
    pub kind: MachineStdTheoremKind,
    pub universe_params: Vec<String>,
    pub statement_core_hash: Hash,
    pub statement_head: Option<MachineStdGlobalRefView>,
    pub constants: Vec<MachineStdGlobalRefView>,
    pub modes: Vec<MachineTheoremMode>,
    pub attributes: Vec<MachineStdAttribute>,
    pub rewrite_descriptors: Vec<MachineStdRewriteDescriptor>,
    pub axiom_dependencies: Vec<MachineStdAxiomRef>,
    pub proof_term_size: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MachineStdGlobalRef {
    pub module: Name,
    pub name: Name,
    pub export_hash: Hash,
    pub certificate_hash: Hash,
    pub decl_interface_hash: Hash,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineStdTheoremKind {
    Theorem,
    Axiom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineStdAttribute {
    Simp,
    Rw,
    Intro,
    Elim,
    Apply,
    Refl,
    Trans,
    Congr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MachineStdRewriteDescriptor {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MachineStdGlobalRefView {
    Decl {
        module: Name,
        name: Name,
        export_hash: Hash,
        certificate_hash: Hash,
        decl_interface_hash: Hash,
        public_export: bool,
    },
    Generated {
        module: Name,
        parent_name: Name,
        name: Name,
        export_hash: Hash,
        certificate_hash: Hash,
        parent_decl_interface_hash: Hash,
        decl_interface_hash: Hash,
        public_export: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineStdImportBundleSet {
    pub library_profile_id: String,
    pub bundles: Vec<MachineStdImportBundle>,
    pub import_bundles_hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineStdImportBundle {
    pub bundle_id: String,
    pub root_imports: Vec<VerifiedImportKey>,
    pub import_closure: Vec<MachineStdImportCertificate>,
    pub allow_axioms: Vec<MachineAxiomRefWire>,
    pub recommended_tactic_options: MachineStdTacticOptionsRecipe,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineStdImportCertificate {
    pub module: Name,
    pub expected_export_hash: Hash,
    pub expected_certificate_hash: Hash,
    pub certificate_encoding: String,
    pub certificate_bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineStdTacticOptionsRecipe {
    pub recipe_id: String,
    pub kernel_check_profile: String,
    pub simp_rules: Vec<SimpRuleRef>,
    pub eq_family: Option<EqFamilyRef>,
    pub nat_family: Option<NatFamilyRef>,
    pub max_simp_rewrite_steps: u64,
    pub max_open_goals: u64,
    pub max_metas: u64,
}

#[derive(Clone, Debug)]
pub struct MachineStdValidatedRelease {
    pub manifest: MachineStdLibraryRelease,
    pub loaded: MachineStdLoadedRelease,
    pub axiom_report: MachineStdAxiomReport,
    pub import_bundles: MachineStdImportBundleSet,
    pub std_library_release_hash: Hash,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineStdArtifactKind {
    LibraryRelease,
    ImportBundles,
    TheoremIndex,
    AxiomReport,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineStdArtifactShapeError {
    pub artifact: MachineStdArtifactKind,
    pub path: String,
    pub reason: MachineStdArtifactShapeErrorReason,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MachineStdArtifactShapeErrorReason {
    JsonParse {
        offset: usize,
        kind: JsonParseErrorKind,
    },
    ExpectedObject {
        actual: JsonValueKind,
    },
    ExpectedArray {
        actual: JsonValueKind,
    },
    DuplicateKey {
        key: String,
    },
    UnknownField {
        field: String,
    },
    MissingField {
        field: &'static str,
    },
    NullField {
        field: &'static str,
    },
    TypeMismatch {
        field: &'static str,
        expected: &'static str,
        actual: JsonValueKind,
    },
    InvalidUnsignedInteger {
        field: &'static str,
        raw: String,
        error: StrictUnsignedIntegerError,
    },
    InvalidHashString {
        field: &'static str,
    },
    InvalidName {
        field: &'static str,
    },
    InvalidHexString {
        field: &'static str,
    },
    InvalidEnumString {
        field: &'static str,
    },
}

#[derive(Debug)]
pub enum MachineStdReleaseArtifactError {
    ReadArtifact {
        artifact: MachineStdArtifactKind,
        path: PathBuf,
        source: io::Error,
    },
    InvalidStdArtifactShape(MachineStdArtifactShapeError),
    InvalidStdLibraryRelease(MachineStdLibraryReleaseError),
    InvalidStdAxiomPolicy(MachineStdAxiomPolicyError),
    InvalidStdImportBundle(MachineStdImportBundleError),
    InvalidStdTheoremIndex(MachineStdTheoremIndexError),
}

#[derive(Debug)]
pub enum MachineStdLibraryReleaseError {
    ScalarMismatch {
        field: &'static str,
        expected: &'static str,
        actual: String,
    },
    InvalidModuleMembership {
        expected: Vec<Name>,
        actual: Vec<Name>,
    },
    DuplicateModule {
        module: Name,
    },
    NonCanonicalModuleOrder {
        expected: Vec<Name>,
        actual: Vec<Name>,
    },
    CertificateEncodingMismatch {
        module: Name,
        actual: String,
    },
    CertificateLoader {
        source: Box<MachineStdReleaseLoaderError>,
    },
    ModuleArtifactHashMismatch {
        module: Name,
        field: &'static str,
        expected: Hash,
        actual: Hash,
    },
    ModuleArtifactCountMismatch {
        module: Name,
        field: &'static str,
        expected: u64,
        actual: u64,
    },
    SidecarHashMismatch {
        field: &'static str,
        expected: Hash,
        actual: Hash,
    },
    CanonicalBytes {
        source: MachineStdCanonicalBytesError,
    },
}

#[derive(Debug)]
pub enum MachineStdAxiomPolicyError {
    LibraryProfileMismatch {
        expected: &'static str,
        actual: String,
    },
    AxiomReportHashMismatch {
        expected: Hash,
        actual: Hash,
    },
    InvalidModuleMembership {
        expected: Vec<Name>,
        actual: Vec<Name>,
    },
    DuplicateModule {
        module: Name,
    },
    NonCanonicalModuleOrder {
        expected: Vec<Name>,
        actual: Vec<Name>,
    },
    ModuleHashMismatch {
        module: Name,
        field: &'static str,
        expected: Hash,
        actual: Hash,
    },
    NonCanonicalAxiomOrder {
        module: Name,
        field: &'static str,
    },
    NonEmptyMvpAxiomList {
        module: Name,
        field: &'static str,
    },
    ModuleAxiomsMismatch {
        module: Name,
    },
    TransitiveAxiomsMismatch {
        module: Name,
    },
    AxiomRefProjectionFailed {
        module: Name,
    },
    CanonicalBytes {
        source: MachineStdCanonicalBytesError,
    },
}

#[derive(Debug)]
pub enum MachineStdImportBundleError {
    LibraryProfileMismatch {
        expected: &'static str,
        actual: String,
    },
    ImportBundlesHashMismatch {
        expected: Hash,
        actual: Hash,
    },
    InvalidBundleMembership {
        expected: Vec<String>,
        actual: Vec<String>,
    },
    DuplicateBundle {
        bundle_id: String,
    },
    NonCanonicalBundleOrder {
        expected: Vec<String>,
        actual: Vec<String>,
    },
    DuplicateRootImport {
        bundle_id: String,
        key: Box<VerifiedImportKey>,
    },
    DuplicateImportClosure {
        bundle_id: String,
        key: Box<VerifiedImportKey>,
    },
    NonCanonicalRootImportOrder {
        bundle_id: String,
    },
    NonCanonicalImportClosureOrder {
        bundle_id: String,
    },
    RootImportsMismatch {
        bundle_id: String,
        expected: Vec<VerifiedImportKey>,
        actual: Vec<VerifiedImportKey>,
    },
    ImportClosureMismatch {
        bundle_id: String,
        expected: Vec<VerifiedImportKey>,
        actual: Vec<VerifiedImportKey>,
    },
    CertificateEncodingMismatch {
        bundle_id: String,
        module: Name,
        actual: String,
    },
    CertificateBytesMismatch {
        bundle_id: String,
        module: Name,
    },
    CertificateBytesHashMismatch {
        bundle_id: String,
        module: Name,
        expected: Hash,
        actual: Hash,
    },
    ImportKeyHashMismatch {
        bundle_id: String,
        module: Name,
    },
    MissingDependency {
        bundle_id: String,
        owner: Name,
        missing: Name,
    },
    NonEmptyMvpAllowAxioms {
        bundle_id: String,
    },
    InvalidRecipeIdMapping {
        bundle_id: String,
        expected: &'static str,
        actual: String,
    },
    CanonicalBytes {
        source: MachineStdCanonicalBytesError,
    },
}

#[derive(Debug)]
pub enum MachineStdTheoremIndexError {
    IndexProfileMismatch {
        expected: &'static str,
        actual: String,
    },
    LibraryProfileMismatch {
        expected: &'static str,
        actual: String,
    },
    TheoremIndexHashMismatch {
        expected: Hash,
        actual: Hash,
    },
    InvalidEntryMembership {
        expected: Vec<MachineStdGlobalRef>,
        actual: Vec<MachineStdGlobalRef>,
    },
    DuplicateEntry {
        global_ref: Box<MachineStdGlobalRef>,
    },
    NonCanonicalEntryOrder {
        expected: Vec<MachineStdGlobalRef>,
        actual: Vec<MachineStdGlobalRef>,
    },
    KindMismatch {
        global_ref: Box<MachineStdGlobalRef>,
    },
    UniverseParamsMismatch {
        global_ref: Box<MachineStdGlobalRef>,
    },
    StatementCoreHashMismatch {
        global_ref: Box<MachineStdGlobalRef>,
    },
    StatementHeadMismatch {
        global_ref: Box<MachineStdGlobalRef>,
    },
    ConstantsMismatch {
        global_ref: Box<MachineStdGlobalRef>,
    },
    ModesMismatch {
        global_ref: Box<MachineStdGlobalRef>,
    },
    AttributesMismatch {
        global_ref: Box<MachineStdGlobalRef>,
    },
    RewriteDescriptorsMismatch {
        global_ref: Box<MachineStdGlobalRef>,
    },
    AxiomDependenciesMismatch {
        global_ref: Box<MachineStdGlobalRef>,
    },
    NonNullProofTermSize {
        global_ref: Box<MachineStdGlobalRef>,
    },
    NonCanonicalModes {
        global_ref: Box<MachineStdGlobalRef>,
    },
    NonCanonicalAttributes {
        global_ref: Box<MachineStdGlobalRef>,
    },
    NonCanonicalConstants {
        global_ref: Box<MachineStdGlobalRef>,
    },
    NonCanonicalAxiomDependencies {
        global_ref: Box<MachineStdGlobalRef>,
    },
    InvalidRenderableName {
        module: Name,
        name: Name,
    },
    InvalidUniverseParam {
        module: Name,
        name: Name,
    },
    DuplicateUniverseParam {
        module: Name,
        name: Name,
        param: String,
    },
    InvalidGlobalRef {
        module: Name,
    },
    InvalidTermRef {
        module: Name,
    },
    InvalidExportKind {
        module: Name,
        name: Name,
    },
    AxiomRefProjectionFailed {
        module: Name,
    },
    CanonicalBytes {
        source: MachineStdCanonicalBytesError,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MachineStdCanonicalBytesError {
    InvalidName {
        name: Name,
        source: Box<MachineWireGrammarError>,
    },
}

#[derive(Debug)]
pub enum MachineStdReleaseLoaderError {
    InvalidModuleMembership {
        expected: Vec<Name>,
        actual: Vec<Name>,
    },
    DuplicateModule {
        module: Name,
    },
    NonCanonicalModuleOrder {
        expected: Vec<Name>,
        actual: Vec<Name>,
    },
    FixedPathMismatch {
        module: Name,
        expected: String,
        actual: String,
    },
    InvalidLocatorPath {
        path: String,
        reason: MachineStdLocatorPathError,
    },
    InvalidPackageRoot {
        path: PathBuf,
        source: io::Error,
    },
    MissingCertificateFile {
        module: Name,
        path: PathBuf,
        source: io::Error,
    },
    ReadCertificateFile {
        module: Name,
        path: PathBuf,
        source: io::Error,
    },
    SymlinkEscape {
        module: Name,
        path: PathBuf,
        resolved: PathBuf,
        package_root: PathBuf,
    },
    DecodeFailed {
        module: Name,
        source: Box<CertError>,
    },
    ModuleNameMismatch {
        expected: Name,
        actual: Name,
    },
    MissingImportCertificateHash {
        owner: Name,
        imported_module: Name,
    },
    UnresolvedImport {
        owner: Name,
        imported_module: Name,
    },
    ImportHashMismatch {
        owner: Name,
        imported_module: Name,
    },
    ImportCycle {
        module: Name,
    },
    InvalidCanonicalModuleName {
        module: Name,
        source: Box<MachineWireGrammarError>,
    },
    VerifyFailed {
        module: Name,
        source: Box<CertError>,
    },
    VerifiedIdentityMismatch {
        module: Name,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineStdLocatorPathError {
    Empty,
    Absolute,
    Backslash,
    TrailingSlash,
    DuplicateSlash,
    DotComponent,
    ParentComponent,
}

#[derive(Clone, Debug)]
struct DecodedStdModule {
    locator: MachineStdModuleLocator,
    resolved_path: PathBuf,
    certificate_bytes: Vec<u8>,
    certificate_bytes_hash: Hash,
    cert: ModuleCert,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct CertificateKey {
    module: Name,
    export_hash: Hash,
    certificate_hash: Hash,
}

pub fn machine_std_mvp_module_locators() -> Vec<MachineStdModuleLocator> {
    vec![
        MachineStdModuleLocator::new(Name::from_dotted("Std.Nat"), STD_NAT_PATH),
        MachineStdModuleLocator::new(Name::from_dotted("Std.List"), STD_LIST_PATH),
        MachineStdModuleLocator::new(Name::from_dotted("Std.Logic"), STD_LOGIC_PATH),
        MachineStdModuleLocator::new(
            Name::from_dotted("Std.Algebra.Basic"),
            STD_ALGEBRA_BASIC_PATH,
        ),
    ]
}

pub fn load_machine_std_mvp_certificates(
    package_root: impl AsRef<Path>,
) -> Result<MachineStdLoadedRelease, MachineStdReleaseLoaderError> {
    load_machine_std_certificates_from_locators(package_root, &machine_std_mvp_module_locators())
}

pub fn load_machine_std_certificates_from_locators(
    package_root: impl AsRef<Path>,
    locators: &[MachineStdModuleLocator],
) -> Result<MachineStdLoadedRelease, MachineStdReleaseLoaderError> {
    validate_machine_std_mvp_locators(locators)?;
    let package_root = canonical_package_root(package_root.as_ref())?;
    let decoded = read_and_decode_std_modules(&package_root, locators)?;
    validate_import_graph(&decoded)?;
    let verification_order = topological_verification_order(&decoded)?;
    verify_decoded_modules(decoded, verification_order, AxiomPolicy::high_trust())
}

pub fn load_machine_std_mvp_release(
    package_root: impl AsRef<Path>,
) -> Result<MachineStdValidatedRelease, MachineStdReleaseArtifactError> {
    let root = package_root.as_ref();
    let release_json = read_std_artifact_json(
        root,
        STD_MACHINE_RELEASE_JSON_PATH,
        MachineStdArtifactKind::LibraryRelease,
    )?;
    let import_bundles_json = read_std_artifact_json(
        root,
        STD_MACHINE_IMPORT_BUNDLES_JSON_PATH,
        MachineStdArtifactKind::ImportBundles,
    )?;
    let axiom_report_json = read_std_artifact_json(
        root,
        STD_MACHINE_AXIOM_REPORT_JSON_PATH,
        MachineStdArtifactKind::AxiomReport,
    )?;
    load_machine_std_mvp_release_with_import_bundles_from_json(
        root,
        &release_json,
        &import_bundles_json,
        &axiom_report_json,
    )
}

pub fn load_machine_std_mvp_release_from_json(
    package_root: impl AsRef<Path>,
    release_json: &str,
    axiom_report_json: &str,
) -> Result<MachineStdValidatedRelease, MachineStdReleaseArtifactError> {
    let (manifest, loaded, axiom_report) =
        load_machine_std_mvp_release_core(package_root, release_json, axiom_report_json)?;
    let import_bundles = generate_machine_std_mvp_import_bundle_set(&loaded)
        .map_err(MachineStdReleaseArtifactError::InvalidStdImportBundle)?;
    compare_release_sidecar_hash(
        "import_bundles_hash",
        manifest.import_bundles_hash,
        import_bundles.import_bundles_hash,
    )?;
    finish_machine_std_mvp_release(manifest, loaded, axiom_report, import_bundles)
}

pub fn load_machine_std_mvp_release_with_import_bundles_from_json(
    package_root: impl AsRef<Path>,
    release_json: &str,
    import_bundles_json: &str,
    axiom_report_json: &str,
) -> Result<MachineStdValidatedRelease, MachineStdReleaseArtifactError> {
    let import_bundles = parse_machine_std_import_bundle_set_json(import_bundles_json)
        .map_err(MachineStdReleaseArtifactError::InvalidStdArtifactShape)?;
    let (manifest, loaded, axiom_report) =
        load_machine_std_mvp_release_core(package_root, release_json, axiom_report_json)?;
    let expected_import_bundles = generate_machine_std_mvp_import_bundle_set(&loaded)
        .map_err(MachineStdReleaseArtifactError::InvalidStdImportBundle)?;
    validate_machine_std_mvp_import_bundle_set(&import_bundles, &expected_import_bundles)
        .map_err(MachineStdReleaseArtifactError::InvalidStdImportBundle)?;
    compare_release_sidecar_hash(
        "import_bundles_hash",
        manifest.import_bundles_hash,
        import_bundles.import_bundles_hash,
    )?;
    finish_machine_std_mvp_release(manifest, loaded, axiom_report, import_bundles)
}

fn load_machine_std_mvp_release_core(
    package_root: impl AsRef<Path>,
    release_json: &str,
    axiom_report_json: &str,
) -> Result<
    (
        MachineStdLibraryRelease,
        MachineStdLoadedRelease,
        MachineStdAxiomReport,
    ),
    MachineStdReleaseArtifactError,
> {
    let manifest = parse_machine_std_library_release_json(release_json)
        .map_err(MachineStdReleaseArtifactError::InvalidStdArtifactShape)?;
    let axiom_report = parse_machine_std_axiom_report_json(axiom_report_json)
        .map_err(MachineStdReleaseArtifactError::InvalidStdArtifactShape)?;

    validate_machine_std_library_release_prepass(&manifest)
        .map_err(MachineStdReleaseArtifactError::InvalidStdLibraryRelease)?;

    let loaded = load_machine_std_mvp_certificates_for_manifest_validation(package_root.as_ref())
        .map_err(|source| {
        MachineStdReleaseArtifactError::InvalidStdLibraryRelease(
            MachineStdLibraryReleaseError::CertificateLoader {
                source: Box::new(source),
            },
        )
    })?;
    validate_machine_std_library_release_against_certificates(&manifest, &loaded)
        .map_err(MachineStdReleaseArtifactError::InvalidStdLibraryRelease)?;

    let actual_axiom_report_hash =
        machine_std_axiom_report_hash(&axiom_report).map_err(|source| {
            MachineStdReleaseArtifactError::InvalidStdAxiomPolicy(
                MachineStdAxiomPolicyError::CanonicalBytes { source },
            )
        })?;
    if actual_axiom_report_hash != axiom_report.axiom_report_hash {
        return Err(MachineStdReleaseArtifactError::InvalidStdAxiomPolicy(
            MachineStdAxiomPolicyError::AxiomReportHashMismatch {
                expected: axiom_report.axiom_report_hash,
                actual: actual_axiom_report_hash,
            },
        ));
    }

    validate_machine_std_axiom_report(&manifest, &loaded, &axiom_report)
        .map_err(MachineStdReleaseArtifactError::InvalidStdAxiomPolicy)?;
    if manifest.axiom_report_hash != axiom_report.axiom_report_hash {
        compare_release_sidecar_hash(
            "axiom_report_hash",
            manifest.axiom_report_hash,
            axiom_report.axiom_report_hash,
        )?;
    }

    Ok((manifest, loaded, axiom_report))
}

fn finish_machine_std_mvp_release(
    manifest: MachineStdLibraryRelease,
    loaded: MachineStdLoadedRelease,
    axiom_report: MachineStdAxiomReport,
    import_bundles: MachineStdImportBundleSet,
) -> Result<MachineStdValidatedRelease, MachineStdReleaseArtifactError> {
    let std_library_release_hash =
        machine_std_library_release_hash(&manifest).map_err(|source| {
            MachineStdReleaseArtifactError::InvalidStdLibraryRelease(
                MachineStdLibraryReleaseError::CanonicalBytes { source },
            )
        })?;

    Ok(MachineStdValidatedRelease {
        manifest,
        loaded,
        axiom_report,
        import_bundles,
        std_library_release_hash,
    })
}

fn compare_release_sidecar_hash(
    field: &'static str,
    expected: Hash,
    actual: Hash,
) -> Result<(), MachineStdReleaseArtifactError> {
    if expected == actual {
        Ok(())
    } else {
        Err(MachineStdReleaseArtifactError::InvalidStdLibraryRelease(
            MachineStdLibraryReleaseError::SidecarHashMismatch {
                field,
                expected,
                actual,
            },
        ))
    }
}

pub fn parse_machine_std_library_release_json(
    source: &str,
) -> Result<MachineStdLibraryRelease, MachineStdArtifactShapeError> {
    let doc = parse_std_json(source, MachineStdArtifactKind::LibraryRelease)?;
    parse_library_release_value(doc.root(), "$")
}

pub fn parse_machine_std_axiom_report_json(
    source: &str,
) -> Result<MachineStdAxiomReport, MachineStdArtifactShapeError> {
    let doc = parse_std_json(source, MachineStdArtifactKind::AxiomReport)?;
    parse_axiom_report_value(doc.root(), "$")
}

pub fn parse_machine_std_import_bundle_set_json(
    source: &str,
) -> Result<MachineStdImportBundleSet, MachineStdArtifactShapeError> {
    let doc = parse_std_json(source, MachineStdArtifactKind::ImportBundles)?;
    parse_import_bundle_set_value(doc.root(), "$")
}

pub fn machine_std_module_artifact_canonical_bytes(
    artifact: &MachineStdModuleArtifact,
) -> Result<Vec<u8>, MachineStdCanonicalBytesError> {
    let mut out = Vec::new();
    encode_string(&mut out, STD_MODULE_ARTIFACT_TAG);
    encode_name(&mut out, &artifact.module)?;
    encode_hash(&mut out, &artifact.expected_export_hash);
    encode_hash(&mut out, &artifact.expected_certificate_hash);
    encode_string(&mut out, &artifact.certificate_encoding);
    encode_hash(&mut out, &artifact.certificate_bytes_hash);
    encode_hash(&mut out, &artifact.axiom_report_hash);
    encode_uvar(&mut out, artifact.public_export_count);
    encode_uvar(&mut out, artifact.theorem_index_entry_count);
    encode_uvar(&mut out, artifact.simp_rule_count);
    Ok(out)
}

pub fn machine_std_library_release_canonical_bytes(
    release: &MachineStdLibraryRelease,
) -> Result<Vec<u8>, MachineStdCanonicalBytesError> {
    let mut out = Vec::new();
    encode_string(&mut out, STD_LIBRARY_RELEASE_TAG);
    encode_string(&mut out, &release.protocol_version);
    encode_string(&mut out, &release.library_profile_id);
    encode_string(&mut out, &release.core_spec_id);
    encode_string(&mut out, &release.kernel_semantics_profile_id);
    encode_uvar(&mut out, release.modules.len() as u64);
    for module in &release.modules {
        out.extend(machine_std_module_artifact_canonical_bytes(module)?);
    }
    encode_hash(&mut out, &release.import_bundles_hash);
    encode_hash(&mut out, &release.theorem_index_hash);
    encode_hash(&mut out, &release.simp_profiles_hash);
    encode_hash(&mut out, &release.rewrite_profiles_hash);
    encode_hash(&mut out, &release.axiom_report_hash);
    Ok(out)
}

pub fn machine_std_library_release_hash(
    release: &MachineStdLibraryRelease,
) -> Result<Hash, MachineStdCanonicalBytesError> {
    Ok(sha256(&machine_std_library_release_canonical_bytes(
        release,
    )?))
}

pub fn machine_std_tactic_options_recipe_canonical_bytes(
    recipe: &MachineStdTacticOptionsRecipe,
) -> Result<Vec<u8>, MachineStdCanonicalBytesError> {
    let mut out = Vec::new();
    encode_string(&mut out, STD_TACTIC_OPTIONS_RECIPE_TAG);
    encode_string(&mut out, &recipe.recipe_id);
    out.extend(machine_std_kernel_check_profile_canonical_bytes(
        &recipe.kernel_check_profile,
    ));
    encode_uvar(&mut out, recipe.simp_rules.len() as u64);
    for rule in &recipe.simp_rules {
        encode_simp_rule_ref(&mut out, rule)?;
    }
    encode_option_eq_family(&mut out, recipe.eq_family.as_ref())?;
    encode_option_nat_family(&mut out, recipe.nat_family.as_ref())?;
    encode_uvar(&mut out, recipe.max_simp_rewrite_steps);
    encode_uvar(&mut out, recipe.max_open_goals);
    encode_uvar(&mut out, recipe.max_metas);
    Ok(out)
}

pub fn machine_std_import_bundle_canonical_bytes(
    bundle: &MachineStdImportBundle,
) -> Result<Vec<u8>, MachineStdCanonicalBytesError> {
    let mut out = Vec::new();
    encode_string(&mut out, STD_IMPORT_BUNDLE_TAG);
    encode_string(&mut out, &bundle.bundle_id);
    encode_uvar(&mut out, bundle.root_imports.len() as u64);
    for key in &bundle.root_imports {
        encode_verified_import_key(&mut out, key)?;
    }
    encode_uvar(&mut out, bundle.import_closure.len() as u64);
    for certificate in &bundle.import_closure {
        encode_import_certificate_key(&mut out, certificate)?;
        encode_hash(&mut out, &sha256(&certificate.certificate_bytes));
    }
    encode_uvar(&mut out, bundle.allow_axioms.len() as u64);
    for axiom in &bundle.allow_axioms {
        out.extend(encode_machine_axiom_ref_wire(axiom));
    }
    out.extend(machine_std_tactic_options_recipe_canonical_bytes(
        &bundle.recommended_tactic_options,
    )?);
    Ok(out)
}

pub fn machine_std_import_bundle_hash(
    bundle: &MachineStdImportBundle,
) -> Result<Hash, MachineStdCanonicalBytesError> {
    Ok(sha256(&machine_std_import_bundle_canonical_bytes(bundle)?))
}

pub fn machine_std_import_bundle_set_canonical_bytes(
    bundle_set: &MachineStdImportBundleSet,
) -> Result<Vec<u8>, MachineStdCanonicalBytesError> {
    let mut out = Vec::new();
    encode_string(&mut out, STD_IMPORT_BUNDLE_SET_TAG);
    encode_string(&mut out, &bundle_set.library_profile_id);
    encode_uvar(&mut out, bundle_set.bundles.len() as u64);
    for bundle in &bundle_set.bundles {
        encode_hash(&mut out, &machine_std_import_bundle_hash(bundle)?);
    }
    Ok(out)
}

pub fn machine_std_import_bundle_set_hash(
    bundle_set: &MachineStdImportBundleSet,
) -> Result<Hash, MachineStdCanonicalBytesError> {
    Ok(sha256(&machine_std_import_bundle_set_canonical_bytes(
        bundle_set,
    )?))
}

pub fn machine_std_global_ref_canonical_bytes(
    global_ref: &MachineStdGlobalRef,
) -> Result<Vec<u8>, MachineStdCanonicalBytesError> {
    let mut out = Vec::new();
    encode_string(&mut out, STD_GLOBAL_REF_TAG);
    encode_name(&mut out, &global_ref.module)?;
    encode_name(&mut out, &global_ref.name)?;
    encode_hash(&mut out, &global_ref.export_hash);
    encode_hash(&mut out, &global_ref.certificate_hash);
    encode_hash(&mut out, &global_ref.decl_interface_hash);
    Ok(out)
}

pub fn machine_std_global_ref_view_canonical_bytes(
    view: &MachineStdGlobalRefView,
) -> Result<Vec<u8>, MachineStdCanonicalBytesError> {
    let mut out = Vec::new();
    encode_string(&mut out, STD_GLOBAL_REF_VIEW_TAG);
    match view {
        MachineStdGlobalRefView::Decl {
            module,
            name,
            export_hash,
            certificate_hash,
            decl_interface_hash,
            public_export,
        } => {
            out.push(0x00);
            encode_name(&mut out, module)?;
            encode_name(&mut out, name)?;
            encode_hash(&mut out, export_hash);
            encode_hash(&mut out, certificate_hash);
            encode_hash(&mut out, decl_interface_hash);
            encode_bool(&mut out, *public_export);
        }
        MachineStdGlobalRefView::Generated {
            module,
            parent_name,
            name,
            export_hash,
            certificate_hash,
            parent_decl_interface_hash,
            decl_interface_hash,
            public_export,
        } => {
            out.push(0x01);
            encode_name(&mut out, module)?;
            encode_name(&mut out, parent_name)?;
            encode_name(&mut out, name)?;
            encode_hash(&mut out, export_hash);
            encode_hash(&mut out, certificate_hash);
            encode_hash(&mut out, parent_decl_interface_hash);
            encode_hash(&mut out, decl_interface_hash);
            encode_bool(&mut out, *public_export);
        }
    }
    Ok(out)
}

pub fn machine_std_theorem_entry_canonical_bytes(
    entry: &MachineStdTheoremEntry,
) -> Result<Vec<u8>, MachineStdCanonicalBytesError> {
    let mut out = Vec::new();
    out.extend(machine_std_global_ref_canonical_bytes(&entry.global_ref)?);
    out.push(theorem_kind_byte(entry.kind));
    encode_uvar(&mut out, entry.universe_params.len() as u64);
    for param in &entry.universe_params {
        encode_string(&mut out, param);
    }
    encode_hash(&mut out, &entry.statement_core_hash);
    encode_option_global_ref_view(&mut out, entry.statement_head.as_ref())?;
    encode_uvar(&mut out, entry.constants.len() as u64);
    for constant in &entry.constants {
        out.extend(machine_std_global_ref_view_canonical_bytes(constant)?);
    }
    encode_uvar(&mut out, entry.modes.len() as u64);
    for mode in &entry.modes {
        out.push(theorem_mode_byte(*mode));
    }
    encode_uvar(&mut out, entry.attributes.len() as u64);
    for attribute in &entry.attributes {
        out.push(theorem_attribute_byte(*attribute));
    }
    encode_uvar(&mut out, entry.rewrite_descriptors.len() as u64);
    encode_uvar(&mut out, entry.axiom_dependencies.len() as u64);
    for axiom in &entry.axiom_dependencies {
        out.extend(machine_std_axiom_ref_canonical_bytes(axiom)?);
    }
    encode_option_u64(&mut out, entry.proof_term_size);
    Ok(out)
}

pub fn machine_std_theorem_index_canonical_bytes(
    theorem_index: &MachineStdTheoremIndex,
) -> Result<Vec<u8>, MachineStdCanonicalBytesError> {
    let mut out = Vec::new();
    encode_string(&mut out, STD_THEOREM_INDEX_TAG);
    encode_string(&mut out, &theorem_index.index_profile_id);
    encode_string(&mut out, &theorem_index.library_profile_id);
    encode_uvar(&mut out, theorem_index.entries.len() as u64);
    for entry in &theorem_index.entries {
        out.extend(machine_std_theorem_entry_canonical_bytes(entry)?);
    }
    Ok(out)
}

pub fn machine_std_theorem_index_hash(
    theorem_index: &MachineStdTheoremIndex,
) -> Result<Hash, MachineStdCanonicalBytesError> {
    Ok(sha256(&machine_std_theorem_index_canonical_bytes(
        theorem_index,
    )?))
}

pub fn machine_std_axiom_ref_canonical_bytes(
    axiom: &MachineStdAxiomRef,
) -> Result<Vec<u8>, MachineStdCanonicalBytesError> {
    let mut out = Vec::new();
    encode_string(&mut out, PHASE5_AXIOM_REF_WIRE_TAG);
    out.push(0x00);
    encode_name(&mut out, &axiom.module)?;
    encode_name(&mut out, &axiom.name)?;
    encode_hash(&mut out, &axiom.export_hash);
    encode_hash(&mut out, &axiom.decl_interface_hash);
    Ok(out)
}

pub fn machine_std_axiom_report_canonical_bytes(
    report: &MachineStdAxiomReport,
) -> Result<Vec<u8>, MachineStdCanonicalBytesError> {
    let mut out = Vec::new();
    encode_string(&mut out, STD_AXIOM_REPORT_TAG);
    encode_string(&mut out, &report.library_profile_id);
    encode_uvar(&mut out, report.modules.len() as u64);
    for module in &report.modules {
        encode_name(&mut out, &module.module)?;
        encode_hash(&mut out, &module.export_hash);
        encode_hash(&mut out, &module.certificate_hash);
        encode_uvar(&mut out, module.module_axioms.len() as u64);
        for axiom in &module.module_axioms {
            out.extend(machine_std_axiom_ref_canonical_bytes(axiom)?);
        }
        encode_uvar(&mut out, module.transitive_axioms.len() as u64);
        for axiom in &module.transitive_axioms {
            out.extend(machine_std_axiom_ref_canonical_bytes(axiom)?);
        }
    }
    Ok(out)
}

pub fn machine_std_axiom_report_hash(
    report: &MachineStdAxiomReport,
) -> Result<Hash, MachineStdCanonicalBytesError> {
    Ok(sha256(&machine_std_axiom_report_canonical_bytes(report)?))
}

pub fn validate_machine_std_mvp_locators(
    locators: &[MachineStdModuleLocator],
) -> Result<(), MachineStdReleaseLoaderError> {
    let expected = machine_std_mvp_module_locators();
    let expected_modules = expected
        .iter()
        .map(|locator| locator.module.clone())
        .collect::<Vec<_>>();
    let actual_modules = locators
        .iter()
        .map(|locator| locator.module.clone())
        .collect::<Vec<_>>();

    let mut seen = BTreeSet::new();
    for module in &actual_modules {
        if !seen.insert(module.clone()) {
            return Err(MachineStdReleaseLoaderError::DuplicateModule {
                module: module.clone(),
            });
        }
    }

    let expected_set = expected_modules.iter().cloned().collect::<BTreeSet<_>>();
    let actual_set = actual_modules.iter().cloned().collect::<BTreeSet<_>>();
    if actual_set != expected_set {
        return Err(MachineStdReleaseLoaderError::InvalidModuleMembership {
            expected: expected_modules,
            actual: actual_modules,
        });
    }
    if actual_modules != expected_modules {
        return Err(MachineStdReleaseLoaderError::NonCanonicalModuleOrder {
            expected: expected_modules,
            actual: actual_modules,
        });
    }

    for (locator, expected_locator) in locators.iter().zip(expected.iter()) {
        validate_machine_std_locator_path(&locator.relative_path).map_err(|reason| {
            MachineStdReleaseLoaderError::InvalidLocatorPath {
                path: locator.relative_path.clone(),
                reason,
            }
        })?;
        if locator.relative_path != expected_locator.relative_path {
            return Err(MachineStdReleaseLoaderError::FixedPathMismatch {
                module: locator.module.clone(),
                expected: expected_locator.relative_path.clone(),
                actual: locator.relative_path.clone(),
            });
        }
    }

    Ok(())
}

pub fn validate_machine_std_locator_path(path: &str) -> Result<(), MachineStdLocatorPathError> {
    if path.is_empty() {
        return Err(MachineStdLocatorPathError::Empty);
    }
    if path.starts_with('/') {
        return Err(MachineStdLocatorPathError::Absolute);
    }
    if path.contains('\\') {
        return Err(MachineStdLocatorPathError::Backslash);
    }
    if path.ends_with('/') {
        return Err(MachineStdLocatorPathError::TrailingSlash);
    }
    if path.contains("//") {
        return Err(MachineStdLocatorPathError::DuplicateSlash);
    }
    for component in path.split('/') {
        match component {
            "" => return Err(MachineStdLocatorPathError::Empty),
            "." => return Err(MachineStdLocatorPathError::DotComponent),
            ".." => return Err(MachineStdLocatorPathError::ParentComponent),
            _ => {}
        }
    }
    Ok(())
}

fn read_std_artifact_json(
    root: &Path,
    relative_path: &str,
    artifact: MachineStdArtifactKind,
) -> Result<String, MachineStdReleaseArtifactError> {
    let path = join_posix_relative_path(root, relative_path);
    fs::read_to_string(&path).map_err(|source| MachineStdReleaseArtifactError::ReadArtifact {
        artifact,
        path,
        source,
    })
}

fn load_machine_std_mvp_certificates_for_manifest_validation(
    package_root: &Path,
) -> Result<MachineStdLoadedRelease, MachineStdReleaseLoaderError> {
    let locators = machine_std_mvp_module_locators();
    validate_machine_std_mvp_locators(&locators)?;
    let package_root = canonical_package_root(package_root)?;
    let decoded = read_and_decode_std_modules(&package_root, &locators)?;
    validate_import_graph(&decoded)?;
    let verification_order = topological_verification_order(&decoded)?;
    let policy = high_trust_policy_allowing_decoded_axioms(&decoded);
    verify_decoded_modules(decoded, verification_order, policy)
}

fn high_trust_policy_allowing_decoded_axioms(
    decoded: &BTreeMap<Name, DecodedStdModule>,
) -> AxiomPolicy {
    let mut allowlisted_axioms = BTreeSet::new();
    for module in decoded.values() {
        allowlisted_axioms.extend(
            module
                .cert
                .name_table
                .iter()
                .filter(|name| name.is_canonical())
                .cloned(),
        );
    }
    AxiomPolicy {
        mode: TrustMode::HighTrust,
        allowlisted_axioms,
        deny_sorry: false,
    }
}

fn validate_machine_std_library_release_prepass(
    manifest: &MachineStdLibraryRelease,
) -> Result<(), MachineStdLibraryReleaseError> {
    validate_fixed_scalar(
        "protocol_version",
        STD_LIBRARY_PROTOCOL_VERSION,
        &manifest.protocol_version,
    )?;
    validate_fixed_scalar(
        "library_profile_id",
        STD_LIBRARY_PROFILE_ID,
        &manifest.library_profile_id,
    )?;
    validate_fixed_scalar("core_spec_id", STD_CORE_SPEC_ID, &manifest.core_spec_id)?;
    validate_fixed_scalar(
        "kernel_semantics_profile_id",
        STD_KERNEL_SEMANTICS_PROFILE_ID,
        &manifest.kernel_semantics_profile_id,
    )?;
    validate_manifest_module_membership(&manifest.modules)?;
    for module in &manifest.modules {
        if module.certificate_encoding != STD_CERTIFICATE_ENCODING {
            return Err(MachineStdLibraryReleaseError::CertificateEncodingMismatch {
                module: module.module.clone(),
                actual: module.certificate_encoding.clone(),
            });
        }
    }
    Ok(())
}

fn validate_fixed_scalar(
    field: &'static str,
    expected: &'static str,
    actual: &str,
) -> Result<(), MachineStdLibraryReleaseError> {
    if actual == expected {
        Ok(())
    } else {
        Err(MachineStdLibraryReleaseError::ScalarMismatch {
            field,
            expected,
            actual: actual.to_owned(),
        })
    }
}

fn validate_manifest_module_membership(
    modules: &[MachineStdModuleArtifact],
) -> Result<(), MachineStdLibraryReleaseError> {
    let expected = expected_mvp_modules();
    let actual = modules
        .iter()
        .map(|module| module.module.clone())
        .collect::<Vec<_>>();
    let mut seen = BTreeSet::new();
    for module in &actual {
        if !seen.insert(module.clone()) {
            return Err(MachineStdLibraryReleaseError::DuplicateModule {
                module: module.clone(),
            });
        }
    }
    let expected_set = expected.iter().cloned().collect::<BTreeSet<_>>();
    let actual_set = actual.iter().cloned().collect::<BTreeSet<_>>();
    if expected_set != actual_set {
        return Err(MachineStdLibraryReleaseError::InvalidModuleMembership { expected, actual });
    }
    if expected != actual {
        return Err(MachineStdLibraryReleaseError::NonCanonicalModuleOrder { expected, actual });
    }
    Ok(())
}

fn validate_machine_std_library_release_against_certificates(
    manifest: &MachineStdLibraryRelease,
    loaded: &MachineStdLoadedRelease,
) -> Result<(), MachineStdLibraryReleaseError> {
    for artifact in &manifest.modules {
        let module = loaded
            .module(&artifact.module)
            .expect("manifest prepass checked MVP module membership");
        compare_module_hash(
            &artifact.module,
            "expected_export_hash",
            artifact.expected_export_hash,
            module.expected_export_hash,
        )?;
        compare_module_hash(
            &artifact.module,
            "expected_certificate_hash",
            artifact.expected_certificate_hash,
            module.expected_certificate_hash,
        )?;
        compare_module_hash(
            &artifact.module,
            "certificate_bytes_hash",
            artifact.certificate_bytes_hash,
            module.certificate_bytes_hash,
        )?;
        compare_module_hash(
            &artifact.module,
            "axiom_report_hash",
            artifact.axiom_report_hash,
            module.axiom_report_hash,
        )?;
        compare_module_count(
            &artifact.module,
            "public_export_count",
            artifact.public_export_count,
            module.verified_module.export_block().len() as u64,
        )?;
        compare_module_count(
            &artifact.module,
            "theorem_index_entry_count",
            artifact.theorem_index_entry_count,
            module
                .verified_module
                .export_block()
                .iter()
                .filter(|entry| matches!(entry.kind, ExportKind::Theorem | ExportKind::Axiom))
                .count() as u64,
        )?;
    }
    Ok(())
}

fn compare_module_hash(
    module: &Name,
    field: &'static str,
    expected: Hash,
    actual: Hash,
) -> Result<(), MachineStdLibraryReleaseError> {
    if expected == actual {
        Ok(())
    } else {
        Err(MachineStdLibraryReleaseError::ModuleArtifactHashMismatch {
            module: module.clone(),
            field,
            expected,
            actual,
        })
    }
}

fn compare_module_count(
    module: &Name,
    field: &'static str,
    expected: u64,
    actual: u64,
) -> Result<(), MachineStdLibraryReleaseError> {
    if expected == actual {
        Ok(())
    } else {
        Err(MachineStdLibraryReleaseError::ModuleArtifactCountMismatch {
            module: module.clone(),
            field,
            expected,
            actual,
        })
    }
}

pub fn generate_machine_std_mvp_theorem_index(
    loaded: &MachineStdLoadedRelease,
) -> Result<MachineStdTheoremIndex, MachineStdTheoremIndexError> {
    let mut entries = Vec::new();
    for module in loaded.modules() {
        for export in module.verified_module.export_block() {
            if matches!(export.kind, ExportKind::Theorem | ExportKind::Axiom) {
                entries.push(generate_machine_std_theorem_entry(loaded, module, export)?);
            }
        }
    }
    let mut keyed_entries = entries
        .into_iter()
        .map(|entry| {
            Ok((
                machine_std_global_ref_canonical_bytes(&entry.global_ref)
                    .map_err(|source| MachineStdTheoremIndexError::CanonicalBytes { source })?,
                entry,
            ))
        })
        .collect::<Result<Vec<_>, MachineStdTheoremIndexError>>()?;
    keyed_entries.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));

    Ok(MachineStdTheoremIndex {
        index_profile_id: STD_THEOREM_INDEX_PROFILE_ID.to_owned(),
        library_profile_id: STD_LIBRARY_PROFILE_ID.to_owned(),
        entries: keyed_entries.into_iter().map(|(_, entry)| entry).collect(),
        index_hash: [0; 32],
    })
}

pub fn validate_machine_std_mvp_theorem_index(
    actual: &MachineStdTheoremIndex,
    expected: &MachineStdTheoremIndex,
) -> Result<(), MachineStdTheoremIndexError> {
    if actual.index_profile_id != STD_THEOREM_INDEX_PROFILE_ID {
        return Err(MachineStdTheoremIndexError::IndexProfileMismatch {
            expected: STD_THEOREM_INDEX_PROFILE_ID,
            actual: actual.index_profile_id.clone(),
        });
    }
    if actual.library_profile_id != STD_LIBRARY_PROFILE_ID {
        return Err(MachineStdTheoremIndexError::LibraryProfileMismatch {
            expected: STD_LIBRARY_PROFILE_ID,
            actual: actual.library_profile_id.clone(),
        });
    }

    validate_theorem_entry_membership(&actual.entries, &expected.entries)?;
    let expected_by_key = expected_theorem_entries_by_key(expected)?;
    for actual_entry in &actual.entries {
        let key = machine_std_global_ref_canonical_bytes(&actual_entry.global_ref)
            .map_err(|source| MachineStdTheoremIndexError::CanonicalBytes { source })?;
        let expected_entry = expected_by_key
            .get(&key)
            .expect("membership validation checked entry key");
        validate_theorem_entry_order(actual_entry)?;
        validate_theorem_entry_contents(actual_entry, expected_entry)?;
    }
    Ok(())
}

fn generate_machine_std_theorem_entry(
    loaded: &MachineStdLoadedRelease,
    module: &MachineStdLoadedModule,
    export: &ExportEntry,
) -> Result<MachineStdTheoremEntry, MachineStdTheoremIndexError> {
    let name = theorem_export_name(module, export)?;
    ensure_renderable_theorem_name(module, &name)?;
    let kind = match export.kind {
        ExportKind::Theorem => MachineStdTheoremKind::Theorem,
        ExportKind::Axiom => MachineStdTheoremKind::Axiom,
        _ => {
            return Err(MachineStdTheoremIndexError::InvalidExportKind {
                module: module.module.clone(),
                name,
            });
        }
    };
    let universe_params = theorem_export_universe_params(module, export)?;
    let statement_head = theorem_statement_head(loaded, module, export.ty)?;
    let constants = theorem_statement_constants(loaded, module, export.ty)?;
    let axiom_dependencies = project_export_axiom_dependencies(loaded, module, export)?;
    let mut modes = vec![MachineTheoremMode::Exact];
    if has_leading_pi_term(module, export.ty)? {
        modes.push(MachineTheoremMode::Apply);
    }

    Ok(MachineStdTheoremEntry {
        global_ref: MachineStdGlobalRef {
            module: module.module.clone(),
            name,
            export_hash: module.expected_export_hash,
            certificate_hash: module.expected_certificate_hash,
            decl_interface_hash: export.decl_interface_hash,
        },
        kind,
        universe_params,
        statement_core_hash: export.type_hash,
        statement_head,
        constants,
        modes,
        attributes: Vec::new(),
        rewrite_descriptors: Vec::new(),
        axiom_dependencies,
        proof_term_size: None,
    })
}

fn validate_theorem_entry_membership(
    actual: &[MachineStdTheoremEntry],
    expected: &[MachineStdTheoremEntry],
) -> Result<(), MachineStdTheoremIndexError> {
    let mut actual_pairs = theorem_entry_global_ref_pairs(actual)?;
    let mut seen = BTreeSet::new();
    for (key, global_ref) in &actual_pairs {
        if !seen.insert(key.clone()) {
            return Err(MachineStdTheoremIndexError::DuplicateEntry {
                global_ref: Box::new(global_ref.clone()),
            });
        }
    }
    let mut expected_pairs = theorem_entry_global_ref_pairs(expected)?;
    actual_pairs.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
    expected_pairs.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
    let actual_sorted_refs = actual_pairs
        .iter()
        .map(|(_, global_ref)| global_ref.clone())
        .collect::<Vec<_>>();
    let expected_sorted_refs = expected_pairs
        .iter()
        .map(|(_, global_ref)| global_ref.clone())
        .collect::<Vec<_>>();
    if actual_sorted_refs != expected_sorted_refs {
        return Err(MachineStdTheoremIndexError::InvalidEntryMembership {
            expected: expected_sorted_refs,
            actual: actual_sorted_refs,
        });
    }
    let actual_refs = actual
        .iter()
        .map(|entry| entry.global_ref.clone())
        .collect::<Vec<_>>();
    if actual_refs != expected_sorted_refs {
        return Err(MachineStdTheoremIndexError::NonCanonicalEntryOrder {
            expected: expected_sorted_refs,
            actual: actual_refs,
        });
    }
    Ok(())
}

fn theorem_entry_global_ref_pairs(
    entries: &[MachineStdTheoremEntry],
) -> Result<Vec<(Vec<u8>, MachineStdGlobalRef)>, MachineStdTheoremIndexError> {
    entries
        .iter()
        .map(|entry| {
            Ok((
                machine_std_global_ref_canonical_bytes(&entry.global_ref)
                    .map_err(|source| MachineStdTheoremIndexError::CanonicalBytes { source })?,
                entry.global_ref.clone(),
            ))
        })
        .collect()
}

fn expected_theorem_entries_by_key(
    expected: &MachineStdTheoremIndex,
) -> Result<BTreeMap<Vec<u8>, &MachineStdTheoremEntry>, MachineStdTheoremIndexError> {
    expected
        .entries
        .iter()
        .map(|entry| {
            Ok((
                machine_std_global_ref_canonical_bytes(&entry.global_ref)
                    .map_err(|source| MachineStdTheoremIndexError::CanonicalBytes { source })?,
                entry,
            ))
        })
        .collect()
}

fn validate_theorem_entry_order(
    entry: &MachineStdTheoremEntry,
) -> Result<(), MachineStdTheoremIndexError> {
    validate_theorem_modes_order(entry)?;
    validate_theorem_attributes_order(entry)?;
    validate_theorem_constants_order(entry)?;
    validate_theorem_axiom_dependencies_order(entry)?;
    Ok(())
}

fn validate_theorem_entry_contents(
    actual: &MachineStdTheoremEntry,
    expected: &MachineStdTheoremEntry,
) -> Result<(), MachineStdTheoremIndexError> {
    let global_ref = || Box::new(actual.global_ref.clone());
    if actual.kind != expected.kind {
        return Err(MachineStdTheoremIndexError::KindMismatch {
            global_ref: global_ref(),
        });
    }
    if actual.universe_params != expected.universe_params {
        return Err(MachineStdTheoremIndexError::UniverseParamsMismatch {
            global_ref: global_ref(),
        });
    }
    if actual.statement_core_hash != expected.statement_core_hash {
        return Err(MachineStdTheoremIndexError::StatementCoreHashMismatch {
            global_ref: global_ref(),
        });
    }
    if actual.statement_head != expected.statement_head {
        return Err(MachineStdTheoremIndexError::StatementHeadMismatch {
            global_ref: global_ref(),
        });
    }
    if actual.constants != expected.constants {
        return Err(MachineStdTheoremIndexError::ConstantsMismatch {
            global_ref: global_ref(),
        });
    }
    if actual.modes.contains(&MachineTheoremMode::Exact)
        != expected.modes.contains(&MachineTheoremMode::Exact)
        || actual.modes.contains(&MachineTheoremMode::Apply)
            != expected.modes.contains(&MachineTheoremMode::Apply)
    {
        return Err(MachineStdTheoremIndexError::ModesMismatch {
            global_ref: global_ref(),
        });
    }
    if actual.axiom_dependencies != expected.axiom_dependencies {
        return Err(MachineStdTheoremIndexError::AxiomDependenciesMismatch {
            global_ref: global_ref(),
        });
    }
    if actual.proof_term_size.is_some() {
        return Err(MachineStdTheoremIndexError::NonNullProofTermSize {
            global_ref: global_ref(),
        });
    }
    Ok(())
}

fn validate_theorem_modes_order(
    entry: &MachineStdTheoremEntry,
) -> Result<(), MachineStdTheoremIndexError> {
    let mut previous = None;
    for mode in &entry.modes {
        let current = theorem_mode_byte(*mode);
        if previous.is_some_and(|previous| previous >= current) {
            return Err(MachineStdTheoremIndexError::NonCanonicalModes {
                global_ref: Box::new(entry.global_ref.clone()),
            });
        }
        previous = Some(current);
    }
    Ok(())
}

fn validate_theorem_attributes_order(
    entry: &MachineStdTheoremEntry,
) -> Result<(), MachineStdTheoremIndexError> {
    let mut previous = None;
    for attribute in &entry.attributes {
        let current = theorem_attribute_byte(*attribute);
        if previous.is_some_and(|previous| previous >= current) {
            return Err(MachineStdTheoremIndexError::NonCanonicalAttributes {
                global_ref: Box::new(entry.global_ref.clone()),
            });
        }
        previous = Some(current);
    }
    Ok(())
}

fn validate_theorem_constants_order(
    entry: &MachineStdTheoremEntry,
) -> Result<(), MachineStdTheoremIndexError> {
    let mut previous: Option<Vec<u8>> = None;
    for constant in &entry.constants {
        let current = machine_std_global_ref_view_canonical_bytes(constant)
            .map_err(|source| MachineStdTheoremIndexError::CanonicalBytes { source })?;
        if previous
            .as_ref()
            .is_some_and(|previous| previous >= &current)
        {
            return Err(MachineStdTheoremIndexError::NonCanonicalConstants {
                global_ref: Box::new(entry.global_ref.clone()),
            });
        }
        previous = Some(current);
    }
    Ok(())
}

fn validate_theorem_axiom_dependencies_order(
    entry: &MachineStdTheoremEntry,
) -> Result<(), MachineStdTheoremIndexError> {
    let mut previous: Option<Vec<u8>> = None;
    for axiom in &entry.axiom_dependencies {
        let current = machine_std_axiom_ref_canonical_bytes(axiom)
            .map_err(|source| MachineStdTheoremIndexError::CanonicalBytes { source })?;
        if previous
            .as_ref()
            .is_some_and(|previous| previous >= &current)
        {
            return Err(MachineStdTheoremIndexError::NonCanonicalAxiomDependencies {
                global_ref: Box::new(entry.global_ref.clone()),
            });
        }
        previous = Some(current);
    }
    Ok(())
}

fn theorem_export_name(
    module: &MachineStdLoadedModule,
    export: &ExportEntry,
) -> Result<Name, MachineStdTheoremIndexError> {
    module
        .verified_module
        .name_table()
        .get(export.name)
        .cloned()
        .ok_or_else(|| MachineStdTheoremIndexError::InvalidGlobalRef {
            module: module.module.clone(),
        })
}

fn ensure_renderable_theorem_name(
    module: &MachineStdLoadedModule,
    name: &Name,
) -> Result<(), MachineStdTheoremIndexError> {
    parse_machine_surface_renderable_name_wire(&name.as_dotted())
        .map(|_| ())
        .map_err(|_| MachineStdTheoremIndexError::InvalidRenderableName {
            module: module.module.clone(),
            name: name.clone(),
        })
}

fn theorem_export_universe_params(
    module: &MachineStdLoadedModule,
    export: &ExportEntry,
) -> Result<Vec<String>, MachineStdTheoremIndexError> {
    let export_name = theorem_export_name(module, export)?;
    let mut seen = BTreeSet::new();
    export
        .universe_params
        .iter()
        .map(|name_id| {
            let name = module
                .verified_module
                .name_table()
                .get(*name_id)
                .ok_or_else(|| MachineStdTheoremIndexError::InvalidUniverseParam {
                    module: module.module.clone(),
                    name: export_name.clone(),
                })?;
            let [component] = name.0.as_slice() else {
                return Err(MachineStdTheoremIndexError::InvalidUniverseParam {
                    module: module.module.clone(),
                    name: export_name.clone(),
                });
            };
            let param = parse_machine_universe_param_name(component).map_err(|_| {
                MachineStdTheoremIndexError::InvalidUniverseParam {
                    module: module.module.clone(),
                    name: export_name.clone(),
                }
            })?;
            if !seen.insert(param.clone()) {
                return Err(MachineStdTheoremIndexError::DuplicateUniverseParam {
                    module: module.module.clone(),
                    name: export_name.clone(),
                    param,
                });
            }
            Ok(param)
        })
        .collect()
}

fn theorem_statement_head(
    loaded: &MachineStdLoadedRelease,
    owner: &MachineStdLoadedModule,
    ty: TermId,
) -> Result<Option<MachineStdGlobalRefView>, MachineStdTheoremIndexError> {
    let mut conclusion = ty;
    while let TermNode::Pi { body, .. } = term_node(owner, conclusion)?.clone() {
        conclusion = body;
    }
    let mut current = conclusion;
    while let TermNode::App(func, _) = term_node(owner, current)?.clone() {
        current = func;
    }
    match term_node(owner, current)? {
        TermNode::Const { global_ref, .. } => {
            normalize_std_global_ref_view(loaded, owner, global_ref).map(Some)
        }
        _ => Ok(None),
    }
}

fn theorem_statement_constants(
    loaded: &MachineStdLoadedRelease,
    owner: &MachineStdLoadedModule,
    ty: TermId,
) -> Result<Vec<MachineStdGlobalRefView>, MachineStdTheoremIndexError> {
    let mut constants = BTreeMap::new();
    let mut visited = BTreeSet::new();
    collect_term_constants(loaded, owner, ty, &mut visited, &mut constants)?;
    Ok(constants.into_values().collect())
}

fn collect_term_constants(
    loaded: &MachineStdLoadedRelease,
    owner: &MachineStdLoadedModule,
    term: TermId,
    visited: &mut BTreeSet<TermId>,
    constants: &mut BTreeMap<Vec<u8>, MachineStdGlobalRefView>,
) -> Result<(), MachineStdTheoremIndexError> {
    if !visited.insert(term) {
        return Ok(());
    }
    match term_node(owner, term)?.clone() {
        TermNode::Sort(_) | TermNode::BVar(_) => Ok(()),
        TermNode::Const { global_ref, .. } => {
            let view = normalize_std_global_ref_view(loaded, owner, &global_ref)?;
            let key = machine_std_global_ref_view_canonical_bytes(&view)
                .map_err(|source| MachineStdTheoremIndexError::CanonicalBytes { source })?;
            constants.insert(key, view);
            Ok(())
        }
        TermNode::App(func, arg) => {
            collect_term_constants(loaded, owner, func, visited, constants)?;
            collect_term_constants(loaded, owner, arg, visited, constants)
        }
        TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => {
            collect_term_constants(loaded, owner, ty, visited, constants)?;
            collect_term_constants(loaded, owner, body, visited, constants)
        }
        TermNode::Let { ty, value, body } => {
            collect_term_constants(loaded, owner, ty, visited, constants)?;
            collect_term_constants(loaded, owner, value, visited, constants)?;
            collect_term_constants(loaded, owner, body, visited, constants)
        }
    }
}

fn has_leading_pi_term(
    module: &MachineStdLoadedModule,
    ty: TermId,
) -> Result<bool, MachineStdTheoremIndexError> {
    Ok(matches!(term_node(module, ty)?, TermNode::Pi { .. }))
}

fn term_node(
    module: &MachineStdLoadedModule,
    term: TermId,
) -> Result<&TermNode, MachineStdTheoremIndexError> {
    module
        .verified_module
        .term_table()
        .get(term)
        .ok_or_else(|| MachineStdTheoremIndexError::InvalidTermRef {
            module: module.module.clone(),
        })
}

fn normalize_std_global_ref_view(
    loaded: &MachineStdLoadedRelease,
    owner: &MachineStdLoadedModule,
    global_ref: &GlobalRef,
) -> Result<MachineStdGlobalRefView, MachineStdTheoremIndexError> {
    match global_ref {
        GlobalRef::Builtin { .. } => Err(MachineStdTheoremIndexError::InvalidGlobalRef {
            module: owner.module.clone(),
        }),
        GlobalRef::Imported {
            import_index,
            name,
            decl_interface_hash,
        } => normalize_imported_global_ref_view(
            loaded,
            owner,
            *import_index,
            *name,
            *decl_interface_hash,
        ),
        GlobalRef::Local { decl_index } => normalize_local_global_ref_view(owner, *decl_index),
        GlobalRef::LocalGenerated { decl_index, name } => {
            normalize_local_generated_global_ref_view(owner, *decl_index, *name)
        }
    }
}

fn normalize_imported_global_ref_view(
    loaded: &MachineStdLoadedRelease,
    owner: &MachineStdLoadedModule,
    import_index: usize,
    name_id: usize,
    decl_interface_hash: Hash,
) -> Result<MachineStdGlobalRefView, MachineStdTheoremIndexError> {
    let import = owner.imports.get(import_index).ok_or_else(|| {
        MachineStdTheoremIndexError::InvalidGlobalRef {
            module: owner.module.clone(),
        }
    })?;
    let imported = loaded.module(&import.module).ok_or_else(|| {
        MachineStdTheoremIndexError::InvalidGlobalRef {
            module: owner.module.clone(),
        }
    })?;
    if import.export_hash != imported.expected_export_hash
        || import.certificate_hash != Some(imported.expected_certificate_hash)
    {
        return Err(MachineStdTheoremIndexError::InvalidGlobalRef {
            module: owner.module.clone(),
        });
    }
    let name = owner
        .verified_module
        .name_table()
        .get(name_id)
        .cloned()
        .ok_or_else(|| MachineStdTheoremIndexError::InvalidGlobalRef {
            module: owner.module.clone(),
        })?;
    let export = unique_public_export(imported, &name, decl_interface_hash).ok_or_else(|| {
        MachineStdTheoremIndexError::InvalidGlobalRef {
            module: owner.module.clone(),
        }
    })?;
    match export.kind {
        ExportKind::Constructor | ExportKind::Recursor => {
            let (parent_name, parent_decl_interface_hash) =
                generated_parent_for_public_export(imported, export)?;
            Ok(MachineStdGlobalRefView::Generated {
                module: imported.module.clone(),
                parent_name,
                name,
                export_hash: imported.expected_export_hash,
                certificate_hash: imported.expected_certificate_hash,
                parent_decl_interface_hash,
                decl_interface_hash,
                public_export: true,
            })
        }
        _ => Ok(MachineStdGlobalRefView::Decl {
            module: imported.module.clone(),
            name,
            export_hash: imported.expected_export_hash,
            certificate_hash: imported.expected_certificate_hash,
            decl_interface_hash,
            public_export: true,
        }),
    }
}

fn normalize_local_global_ref_view(
    owner: &MachineStdLoadedModule,
    decl_index: usize,
) -> Result<MachineStdGlobalRefView, MachineStdTheoremIndexError> {
    let decl = owner
        .verified_module
        .declarations()
        .get(decl_index)
        .ok_or_else(|| MachineStdTheoremIndexError::InvalidGlobalRef {
            module: owner.module.clone(),
        })?;
    let name = decl_name(owner, decl)?;
    Ok(MachineStdGlobalRefView::Decl {
        module: owner.module.clone(),
        name: name.clone(),
        export_hash: owner.expected_export_hash,
        certificate_hash: owner.expected_certificate_hash,
        decl_interface_hash: decl.hashes.decl_interface_hash,
        public_export: public_export_exists(owner, &name, decl.hashes.decl_interface_hash),
    })
}

fn normalize_local_generated_global_ref_view(
    owner: &MachineStdLoadedModule,
    decl_index: usize,
    name_id: usize,
) -> Result<MachineStdGlobalRefView, MachineStdTheoremIndexError> {
    let decl = owner
        .verified_module
        .declarations()
        .get(decl_index)
        .ok_or_else(|| MachineStdTheoremIndexError::InvalidGlobalRef {
            module: owner.module.clone(),
        })?;
    let generated_name = owner
        .verified_module
        .name_table()
        .get(name_id)
        .cloned()
        .ok_or_else(|| MachineStdTheoremIndexError::InvalidGlobalRef {
            module: owner.module.clone(),
        })?;
    let parent_name = local_generated_parent_name(owner, decl, &generated_name)?;
    Ok(MachineStdGlobalRefView::Generated {
        module: owner.module.clone(),
        parent_name,
        name: generated_name.clone(),
        export_hash: owner.expected_export_hash,
        certificate_hash: owner.expected_certificate_hash,
        parent_decl_interface_hash: decl.hashes.decl_interface_hash,
        decl_interface_hash: decl.hashes.decl_interface_hash,
        public_export: public_generated_export_exists(
            owner,
            &generated_name,
            decl.hashes.decl_interface_hash,
        ),
    })
}

fn unique_public_export<'a>(
    module: &'a MachineStdLoadedModule,
    name: &Name,
    decl_interface_hash: Hash,
) -> Option<&'a ExportEntry> {
    let mut matches = module
        .verified_module
        .export_block()
        .iter()
        .filter(|entry| {
            module
                .verified_module
                .name_table()
                .get(entry.name)
                .is_some_and(|entry_name| {
                    entry_name == name && entry.decl_interface_hash == decl_interface_hash
                })
        });
    let first = matches.next()?;
    if matches.next().is_none() {
        Some(first)
    } else {
        None
    }
}

fn public_export_exists(
    module: &MachineStdLoadedModule,
    name: &Name,
    decl_interface_hash: Hash,
) -> bool {
    unique_public_export(module, name, decl_interface_hash).is_some()
}

fn public_generated_export_exists(
    module: &MachineStdLoadedModule,
    name: &Name,
    decl_interface_hash: Hash,
) -> bool {
    unique_public_export(module, name, decl_interface_hash)
        .is_some_and(|entry| matches!(entry.kind, ExportKind::Constructor | ExportKind::Recursor))
}

fn generated_parent_for_public_export(
    module: &MachineStdLoadedModule,
    export: &ExportEntry,
) -> Result<(Name, Hash), MachineStdTheoremIndexError> {
    let generated_name = theorem_export_name(module, export)?;
    let mut matches = Vec::new();
    for decl in module.verified_module.declarations() {
        if decl.hashes.decl_interface_hash != export.decl_interface_hash {
            continue;
        }
        if inductive_decl_contains_generated(module, decl, &generated_name, Some(export.kind))? {
            matches.push((decl_name(module, decl)?, decl.hashes.decl_interface_hash));
        }
    }
    match matches.as_slice() {
        [result] => Ok(result.clone()),
        _ => Err(MachineStdTheoremIndexError::InvalidGlobalRef {
            module: module.module.clone(),
        }),
    }
}

fn local_generated_parent_name(
    module: &MachineStdLoadedModule,
    decl: &DeclCert,
    generated_name: &Name,
) -> Result<Name, MachineStdTheoremIndexError> {
    if inductive_decl_contains_generated(module, decl, generated_name, None)? {
        decl_name(module, decl)
    } else {
        Err(MachineStdTheoremIndexError::InvalidGlobalRef {
            module: module.module.clone(),
        })
    }
}

fn inductive_decl_contains_generated(
    module: &MachineStdLoadedModule,
    decl: &DeclCert,
    generated_name: &Name,
    expected_kind: Option<ExportKind>,
) -> Result<bool, MachineStdTheoremIndexError> {
    let DeclPayload::Inductive {
        constructors,
        recursor,
        ..
    } = &decl.decl
    else {
        return Ok(false);
    };
    let constructor_allowed = expected_kind
        .map(|kind| kind == ExportKind::Constructor)
        .unwrap_or(true);
    let recursor_allowed = expected_kind
        .map(|kind| kind == ExportKind::Recursor)
        .unwrap_or(true);
    let constructor_match = constructor_allowed
        && constructors.iter().any(|constructor| {
            module
                .verified_module
                .name_table()
                .get(constructor.name)
                .is_some_and(|name| name == generated_name)
        });
    let recursor_match = recursor_allowed
        && recursor.as_ref().is_some_and(|recursor| {
            module
                .verified_module
                .name_table()
                .get(recursor.name)
                .is_some_and(|name| name == generated_name)
        });
    Ok(constructor_match || recursor_match)
}

fn decl_name(
    module: &MachineStdLoadedModule,
    decl: &DeclCert,
) -> Result<Name, MachineStdTheoremIndexError> {
    let name_id = match &decl.decl {
        DeclPayload::Axiom { name, .. }
        | DeclPayload::Def { name, .. }
        | DeclPayload::Theorem { name, .. }
        | DeclPayload::Inductive { name, .. } => *name,
    };
    module
        .verified_module
        .name_table()
        .get(name_id)
        .cloned()
        .ok_or_else(|| MachineStdTheoremIndexError::InvalidGlobalRef {
            module: module.module.clone(),
        })
}

fn project_export_axiom_dependencies(
    loaded: &MachineStdLoadedRelease,
    owner: &MachineStdLoadedModule,
    export: &ExportEntry,
) -> Result<Vec<MachineStdAxiomRef>, MachineStdTheoremIndexError> {
    let mut projected = BTreeMap::new();
    for axiom in &export.axiom_dependencies {
        let axiom = project_axiom_ref(loaded, owner, axiom).map_err(|_| {
            MachineStdTheoremIndexError::AxiomRefProjectionFailed {
                module: owner.module.clone(),
            }
        })?;
        let key = machine_std_axiom_ref_canonical_bytes(&axiom)
            .map_err(|source| MachineStdTheoremIndexError::CanonicalBytes { source })?;
        projected.insert(key, axiom);
    }
    Ok(projected.into_values().collect())
}

pub fn generate_machine_std_mvp_import_bundle_set(
    loaded: &MachineStdLoadedRelease,
) -> Result<MachineStdImportBundleSet, MachineStdImportBundleError> {
    let mut bundle_set = MachineStdImportBundleSet {
        library_profile_id: STD_LIBRARY_PROFILE_ID.to_owned(),
        bundles: expected_mvp_bundle_specs()
            .into_iter()
            .map(|spec| generate_mvp_import_bundle(loaded, spec))
            .collect::<Result<Vec<_>, _>>()?,
        import_bundles_hash: [0; 32],
    };
    bundle_set.import_bundles_hash = machine_std_import_bundle_set_hash(&bundle_set)
        .map_err(|source| MachineStdImportBundleError::CanonicalBytes { source })?;
    Ok(bundle_set)
}

pub fn validate_machine_std_mvp_import_bundle_set(
    actual: &MachineStdImportBundleSet,
    expected: &MachineStdImportBundleSet,
) -> Result<(), MachineStdImportBundleError> {
    let actual_hash = machine_std_import_bundle_set_hash(actual)
        .map_err(|source| MachineStdImportBundleError::CanonicalBytes { source })?;
    if actual_hash != actual.import_bundles_hash {
        return Err(MachineStdImportBundleError::ImportBundlesHashMismatch {
            expected: actual.import_bundles_hash,
            actual: actual_hash,
        });
    }
    if actual.library_profile_id != STD_LIBRARY_PROFILE_ID {
        return Err(MachineStdImportBundleError::LibraryProfileMismatch {
            expected: STD_LIBRARY_PROFILE_ID,
            actual: actual.library_profile_id.clone(),
        });
    }
    validate_import_bundle_membership(&actual.bundles)?;

    let expected_by_id = expected
        .bundles
        .iter()
        .map(|bundle| (bundle.bundle_id.as_str(), bundle))
        .collect::<BTreeMap<_, _>>();

    for bundle in &actual.bundles {
        let expected_bundle = expected_by_id
            .get(bundle.bundle_id.as_str())
            .expect("bundle membership was validated");
        validate_import_key_order(&bundle.bundle_id, &bundle.root_imports)?;
        validate_import_certificate_order(&bundle.bundle_id, &bundle.import_closure)?;
        let actual_closure_keys = bundle
            .import_closure
            .iter()
            .map(import_certificate_key)
            .collect::<Vec<_>>();
        let expected_closure_keys = expected_bundle
            .import_closure
            .iter()
            .map(import_certificate_key)
            .collect::<Vec<_>>();
        if bundle.root_imports != expected_bundle.root_imports {
            return Err(MachineStdImportBundleError::RootImportsMismatch {
                bundle_id: bundle.bundle_id.clone(),
                expected: expected_bundle.root_imports.clone(),
                actual: bundle.root_imports.clone(),
            });
        }
        if actual_closure_keys != expected_closure_keys {
            return Err(MachineStdImportBundleError::ImportClosureMismatch {
                bundle_id: bundle.bundle_id.clone(),
                expected: expected_closure_keys,
                actual: actual_closure_keys,
            });
        }
        for (actual_certificate, expected_certificate) in bundle
            .import_closure
            .iter()
            .zip(&expected_bundle.import_closure)
        {
            validate_import_certificate_bytes(
                &bundle.bundle_id,
                actual_certificate,
                expected_certificate,
            )?;
        }
        if !bundle.allow_axioms.is_empty() {
            return Err(MachineStdImportBundleError::NonEmptyMvpAllowAxioms {
                bundle_id: bundle.bundle_id.clone(),
            });
        }
        let expected_recipe_id = expected_recipe_id_for_bundle(&bundle.bundle_id)
            .expect("bundle membership was validated");
        if bundle.recommended_tactic_options.recipe_id != expected_recipe_id {
            return Err(MachineStdImportBundleError::InvalidRecipeIdMapping {
                bundle_id: bundle.bundle_id.clone(),
                expected: expected_recipe_id,
                actual: bundle.recommended_tactic_options.recipe_id.clone(),
            });
        }
    }
    if actual.import_bundles_hash != expected.import_bundles_hash {
        return Err(MachineStdImportBundleError::ImportBundlesHashMismatch {
            expected: expected.import_bundles_hash,
            actual: actual.import_bundles_hash,
        });
    }
    Ok(())
}

fn generate_mvp_import_bundle(
    loaded: &MachineStdLoadedRelease,
    spec: MvpBundleSpec,
) -> Result<MachineStdImportBundle, MachineStdImportBundleError> {
    let mut root_imports = spec
        .root_modules
        .iter()
        .map(|module| import_key_for_loaded_module(loaded, &Name::from_dotted(module), spec.id))
        .collect::<Result<Vec<_>, _>>()?;
    root_imports.sort();
    let import_closure = import_closure_for_roots(loaded, spec.id, &root_imports)?;
    Ok(MachineStdImportBundle {
        bundle_id: spec.id.to_owned(),
        root_imports,
        import_closure,
        allow_axioms: Vec::new(),
        recommended_tactic_options: MachineStdTacticOptionsRecipe {
            recipe_id: spec.recipe_id.to_owned(),
            kernel_check_profile: KERNEL_CHECK_PROFILE_BUILTIN_NAT_EQ_REC.to_owned(),
            simp_rules: Vec::new(),
            eq_family: std_logic_eq_family(loaded),
            nat_family: None,
            max_simp_rewrite_steps: STD_MAX_SIMP_REWRITE_STEPS,
            max_open_goals: STD_MAX_OPEN_GOALS,
            max_metas: STD_MAX_METAS,
        },
    })
}

#[derive(Clone, Copy)]
struct MvpBundleSpec {
    id: &'static str,
    root_modules: &'static [&'static str],
    recipe_id: &'static str,
}

fn expected_mvp_bundle_specs() -> Vec<MvpBundleSpec> {
    vec![
        MvpBundleSpec {
            id: STD_ALGEBRA_BASIC_BUNDLE_ID,
            root_modules: &["Std.Algebra.Basic", "Std.Logic"],
            recipe_id: STD_LOGIC_RECIPE_ID,
        },
        MvpBundleSpec {
            id: STD_ALL_BUNDLE_ID,
            root_modules: &["Std.Algebra.Basic", "Std.List", "Std.Logic", "Std.Nat"],
            recipe_id: STD_ALL_RECIPE_ID,
        },
        MvpBundleSpec {
            id: STD_LIST_BUNDLE_ID,
            root_modules: &["Std.Logic", "Std.List"],
            recipe_id: STD_LIST_RECIPE_ID,
        },
        MvpBundleSpec {
            id: STD_LOGIC_BUNDLE_ID,
            root_modules: &["Std.Logic"],
            recipe_id: STD_LOGIC_RECIPE_ID,
        },
        MvpBundleSpec {
            id: STD_NAT_BUNDLE_ID,
            root_modules: &["Std.Logic", "Std.Nat"],
            recipe_id: STD_NAT_RECIPE_ID,
        },
    ]
}

fn expected_mvp_bundle_ids() -> Vec<String> {
    expected_mvp_bundle_specs()
        .into_iter()
        .map(|spec| spec.id.to_owned())
        .collect()
}

fn expected_recipe_id_for_bundle(bundle_id: &str) -> Option<&'static str> {
    expected_mvp_bundle_specs()
        .into_iter()
        .find(|spec| spec.id == bundle_id)
        .map(|spec| spec.recipe_id)
}

fn std_logic_eq_family(loaded: &MachineStdLoadedRelease) -> Option<EqFamilyRef> {
    let logic = loaded.module(&Name::from_dotted("Std.Logic"))?;
    let eq = find_std_logic_export(logic, &[ExportKind::Inductive], &["Std.Logic.Eq", "Eq"])?;
    let refl = find_std_logic_export(
        logic,
        &[ExportKind::Constructor],
        &["Std.Logic.Eq.refl", "Eq.refl"],
    )?;
    let rec = find_std_logic_export(
        logic,
        &[ExportKind::Recursor, ExportKind::Axiom],
        &["Std.Logic.Eq.rec", "Eq.rec"],
    )?;
    Some(EqFamilyRef {
        eq_name: eq.0,
        eq_interface_hash: eq.1,
        refl_name: refl.0,
        refl_interface_hash: refl.1,
        rec_name: rec.0,
        rec_interface_hash: rec.1,
    })
}

fn find_std_logic_export(
    module: &MachineStdLoadedModule,
    kinds: &[ExportKind],
    candidates: &[&str],
) -> Option<(Name, Hash)> {
    module
        .verified_module
        .export_block()
        .iter()
        .filter(|entry| kinds.contains(&entry.kind))
        .find_map(|entry| {
            let name = module.verified_module.name_table().get(entry.name)?;
            candidates
                .iter()
                .any(|candidate| *name == Name::from_dotted(candidate))
                .then(|| (name.clone(), entry.decl_interface_hash))
        })
}

fn validate_import_bundle_membership(
    bundles: &[MachineStdImportBundle],
) -> Result<(), MachineStdImportBundleError> {
    let expected = expected_mvp_bundle_ids();
    let actual = bundles
        .iter()
        .map(|bundle| bundle.bundle_id.clone())
        .collect::<Vec<_>>();
    let mut seen = BTreeSet::new();
    for bundle_id in &actual {
        if !seen.insert(bundle_id.clone()) {
            return Err(MachineStdImportBundleError::DuplicateBundle {
                bundle_id: bundle_id.clone(),
            });
        }
    }
    let expected_set = expected.iter().cloned().collect::<BTreeSet<_>>();
    let actual_set = actual.iter().cloned().collect::<BTreeSet<_>>();
    if expected_set != actual_set {
        return Err(MachineStdImportBundleError::InvalidBundleMembership { expected, actual });
    }
    if expected != actual {
        return Err(MachineStdImportBundleError::NonCanonicalBundleOrder { expected, actual });
    }
    Ok(())
}

fn import_key_for_loaded_module(
    loaded: &MachineStdLoadedRelease,
    module: &Name,
    bundle_id: &str,
) -> Result<VerifiedImportKey, MachineStdImportBundleError> {
    let loaded_module =
        loaded
            .module(module)
            .ok_or_else(|| MachineStdImportBundleError::MissingDependency {
                bundle_id: bundle_id.to_owned(),
                owner: module.clone(),
                missing: module.clone(),
            })?;
    Ok(VerifiedImportKey::new(
        loaded_module.module.clone(),
        loaded_module.expected_export_hash,
        loaded_module.expected_certificate_hash,
    ))
}

fn import_closure_for_roots(
    loaded: &MachineStdLoadedRelease,
    bundle_id: &str,
    root_imports: &[VerifiedImportKey],
) -> Result<Vec<MachineStdImportCertificate>, MachineStdImportBundleError> {
    let mut visited = BTreeSet::new();
    let mut pending = root_imports
        .iter()
        .map(|key| key.module.clone())
        .collect::<Vec<_>>();
    while let Some(module) = pending.pop() {
        if !visited.insert(module.clone()) {
            continue;
        }
        let loaded_module = loaded.module(&module).ok_or_else(|| {
            MachineStdImportBundleError::MissingDependency {
                bundle_id: bundle_id.to_owned(),
                owner: module.clone(),
                missing: module.clone(),
            }
        })?;
        for import in &loaded_module.imports {
            let imported = loaded.module(&import.module).ok_or_else(|| {
                MachineStdImportBundleError::MissingDependency {
                    bundle_id: bundle_id.to_owned(),
                    owner: module.clone(),
                    missing: import.module.clone(),
                }
            })?;
            if import.export_hash != imported.expected_export_hash
                || import.certificate_hash != Some(imported.expected_certificate_hash)
            {
                return Err(MachineStdImportBundleError::MissingDependency {
                    bundle_id: bundle_id.to_owned(),
                    owner: module.clone(),
                    missing: import.module.clone(),
                });
            }
            pending.push(import.module.clone());
        }
    }
    let mut closure = visited
        .into_iter()
        .map(|module| {
            let loaded_module = loaded
                .module(&module)
                .expect("visited modules came from loaded release");
            MachineStdImportCertificate {
                module: loaded_module.module.clone(),
                expected_export_hash: loaded_module.expected_export_hash,
                expected_certificate_hash: loaded_module.expected_certificate_hash,
                certificate_encoding: STD_CERTIFICATE_ENCODING.to_owned(),
                certificate_bytes: loaded_module.certificate_bytes.clone(),
            }
        })
        .collect::<Vec<_>>();
    closure.sort_by_key(import_certificate_key);
    Ok(closure)
}

fn validate_import_key_order(
    bundle_id: &str,
    keys: &[VerifiedImportKey],
) -> Result<(), MachineStdImportBundleError> {
    let mut seen = BTreeSet::new();
    let mut previous: Option<Vec<u8>> = None;
    for key in keys {
        if !seen.insert(key.clone()) {
            return Err(MachineStdImportBundleError::DuplicateRootImport {
                bundle_id: bundle_id.to_owned(),
                key: Box::new(key.clone()),
            });
        }
        let bytes = verified_import_key_canonical_bytes(key)
            .map_err(|source| MachineStdImportBundleError::CanonicalBytes { source })?;
        if previous.as_ref().is_some_and(|previous| previous >= &bytes) {
            return Err(MachineStdImportBundleError::NonCanonicalRootImportOrder {
                bundle_id: bundle_id.to_owned(),
            });
        }
        previous = Some(bytes);
    }
    Ok(())
}

fn validate_import_certificate_order(
    bundle_id: &str,
    certificates: &[MachineStdImportCertificate],
) -> Result<(), MachineStdImportBundleError> {
    let mut seen = BTreeSet::new();
    let mut previous: Option<Vec<u8>> = None;
    for certificate in certificates {
        let key = import_certificate_key(certificate);
        if !seen.insert(key.clone()) {
            return Err(MachineStdImportBundleError::DuplicateImportClosure {
                bundle_id: bundle_id.to_owned(),
                key: Box::new(key),
            });
        }
        let bytes = verified_import_key_canonical_bytes(&key)
            .map_err(|source| MachineStdImportBundleError::CanonicalBytes { source })?;
        if previous.as_ref().is_some_and(|previous| previous >= &bytes) {
            return Err(
                MachineStdImportBundleError::NonCanonicalImportClosureOrder {
                    bundle_id: bundle_id.to_owned(),
                },
            );
        }
        previous = Some(bytes);
    }
    Ok(())
}

fn validate_import_certificate_bytes(
    bundle_id: &str,
    actual: &MachineStdImportCertificate,
    expected: &MachineStdImportCertificate,
) -> Result<(), MachineStdImportBundleError> {
    if actual.certificate_encoding != STD_CERTIFICATE_ENCODING {
        return Err(MachineStdImportBundleError::CertificateEncodingMismatch {
            bundle_id: bundle_id.to_owned(),
            module: actual.module.clone(),
            actual: actual.certificate_encoding.clone(),
        });
    }
    if actual.expected_export_hash != expected.expected_export_hash
        || actual.expected_certificate_hash != expected.expected_certificate_hash
    {
        return Err(MachineStdImportBundleError::ImportKeyHashMismatch {
            bundle_id: bundle_id.to_owned(),
            module: actual.module.clone(),
        });
    }
    let actual_hash = sha256(&actual.certificate_bytes);
    let expected_hash = sha256(&expected.certificate_bytes);
    if actual_hash != expected_hash {
        return Err(MachineStdImportBundleError::CertificateBytesHashMismatch {
            bundle_id: bundle_id.to_owned(),
            module: actual.module.clone(),
            expected: expected_hash,
            actual: actual_hash,
        });
    }
    if actual.certificate_bytes != expected.certificate_bytes {
        return Err(MachineStdImportBundleError::CertificateBytesMismatch {
            bundle_id: bundle_id.to_owned(),
            module: actual.module.clone(),
        });
    }
    Ok(())
}

fn import_certificate_key(certificate: &MachineStdImportCertificate) -> VerifiedImportKey {
    VerifiedImportKey::new(
        certificate.module.clone(),
        certificate.expected_export_hash,
        certificate.expected_certificate_hash,
    )
}

fn validate_machine_std_axiom_report(
    manifest: &MachineStdLibraryRelease,
    loaded: &MachineStdLoadedRelease,
    report: &MachineStdAxiomReport,
) -> Result<(), MachineStdAxiomPolicyError> {
    if report.library_profile_id != STD_LIBRARY_PROFILE_ID {
        return Err(MachineStdAxiomPolicyError::LibraryProfileMismatch {
            expected: STD_LIBRARY_PROFILE_ID,
            actual: report.library_profile_id.clone(),
        });
    }
    validate_axiom_report_module_membership(manifest, report)?;

    let mut expected_transitive_by_module: BTreeMap<Name, Vec<MachineStdAxiomRef>> =
        BTreeMap::new();
    let report_by_module = report
        .modules
        .iter()
        .map(|module| (module.module.clone(), module))
        .collect::<BTreeMap<_, _>>();

    for module_name in loaded.verification_order() {
        let loaded_module = loaded
            .module(module_name)
            .expect("verification order came from loaded module table");
        let report_module = report_by_module
            .get(module_name)
            .expect("axiom report membership was validated");

        compare_axiom_report_hash(
            module_name,
            "export_hash",
            report_module.export_hash,
            loaded_module.expected_export_hash,
        )?;
        compare_axiom_report_hash(
            module_name,
            "certificate_hash",
            report_module.certificate_hash,
            loaded_module.expected_certificate_hash,
        )?;
        validate_axiom_ref_list_order(module_name, "module_axioms", &report_module.module_axioms)?;
        validate_axiom_ref_list_order(
            module_name,
            "transitive_axioms",
            &report_module.transitive_axioms,
        )?;
        if !report_module.module_axioms.is_empty() {
            return Err(MachineStdAxiomPolicyError::NonEmptyMvpAxiomList {
                module: module_name.clone(),
                field: "module_axioms",
            });
        }
        if !report_module.transitive_axioms.is_empty() {
            return Err(MachineStdAxiomPolicyError::NonEmptyMvpAxiomList {
                module: module_name.clone(),
                field: "transitive_axioms",
            });
        }

        let expected_module_axioms = project_module_axioms(loaded, loaded_module)?;
        if report_module.module_axioms != expected_module_axioms {
            return Err(MachineStdAxiomPolicyError::ModuleAxiomsMismatch {
                module: module_name.clone(),
            });
        }

        let mut transitive = BTreeMap::new();
        for axiom in expected_module_axioms {
            let key = machine_std_axiom_ref_canonical_bytes(&axiom)
                .map_err(|source| MachineStdAxiomPolicyError::CanonicalBytes { source })?;
            transitive.insert(key, axiom);
        }
        for import in &loaded_module.imports {
            let imported = expected_transitive_by_module
                .get(&import.module)
                .ok_or_else(|| MachineStdAxiomPolicyError::TransitiveAxiomsMismatch {
                    module: module_name.clone(),
                })?;
            for axiom in imported {
                let key = machine_std_axiom_ref_canonical_bytes(axiom)
                    .map_err(|source| MachineStdAxiomPolicyError::CanonicalBytes { source })?;
                transitive.insert(key, axiom.clone());
            }
        }
        let expected_transitive = transitive.into_values().collect::<Vec<_>>();
        if report_module.transitive_axioms != expected_transitive {
            return Err(MachineStdAxiomPolicyError::TransitiveAxiomsMismatch {
                module: module_name.clone(),
            });
        }
        expected_transitive_by_module.insert(module_name.clone(), expected_transitive);
    }

    Ok(())
}

fn validate_axiom_report_module_membership(
    manifest: &MachineStdLibraryRelease,
    report: &MachineStdAxiomReport,
) -> Result<(), MachineStdAxiomPolicyError> {
    let expected = manifest
        .modules
        .iter()
        .map(|module| module.module.clone())
        .collect::<Vec<_>>();
    let actual = report
        .modules
        .iter()
        .map(|module| module.module.clone())
        .collect::<Vec<_>>();
    let mut seen = BTreeSet::new();
    for module in &actual {
        if !seen.insert(module.clone()) {
            return Err(MachineStdAxiomPolicyError::DuplicateModule {
                module: module.clone(),
            });
        }
    }
    let expected_set = expected.iter().cloned().collect::<BTreeSet<_>>();
    let actual_set = actual.iter().cloned().collect::<BTreeSet<_>>();
    if expected_set != actual_set {
        return Err(MachineStdAxiomPolicyError::InvalidModuleMembership { expected, actual });
    }
    if expected != actual {
        return Err(MachineStdAxiomPolicyError::NonCanonicalModuleOrder { expected, actual });
    }
    Ok(())
}

fn compare_axiom_report_hash(
    module: &Name,
    field: &'static str,
    expected: Hash,
    actual: Hash,
) -> Result<(), MachineStdAxiomPolicyError> {
    if expected == actual {
        Ok(())
    } else {
        Err(MachineStdAxiomPolicyError::ModuleHashMismatch {
            module: module.clone(),
            field,
            expected,
            actual,
        })
    }
}

fn validate_axiom_ref_list_order(
    module: &Name,
    field: &'static str,
    axioms: &[MachineStdAxiomRef],
) -> Result<(), MachineStdAxiomPolicyError> {
    let mut previous: Option<Vec<u8>> = None;
    for axiom in axioms {
        let bytes = machine_std_axiom_ref_canonical_bytes(axiom)
            .map_err(|source| MachineStdAxiomPolicyError::CanonicalBytes { source })?;
        if previous.as_ref().is_some_and(|previous| previous >= &bytes) {
            return Err(MachineStdAxiomPolicyError::NonCanonicalAxiomOrder {
                module: module.clone(),
                field,
            });
        }
        previous = Some(bytes);
    }
    Ok(())
}

fn project_module_axioms(
    loaded: &MachineStdLoadedRelease,
    module: &MachineStdLoadedModule,
) -> Result<Vec<MachineStdAxiomRef>, MachineStdAxiomPolicyError> {
    let mut projected = BTreeMap::new();
    for axiom in &module.verified_module.axiom_report().module_axioms {
        let axiom = project_axiom_ref(loaded, module, axiom)?;
        let key = machine_std_axiom_ref_canonical_bytes(&axiom)
            .map_err(|source| MachineStdAxiomPolicyError::CanonicalBytes { source })?;
        projected.insert(key, axiom);
    }
    Ok(projected.into_values().collect())
}

fn project_axiom_ref(
    loaded: &MachineStdLoadedRelease,
    owner: &MachineStdLoadedModule,
    axiom: &AxiomRef,
) -> Result<MachineStdAxiomRef, MachineStdAxiomPolicyError> {
    match &axiom.global_ref {
        GlobalRef::Local { decl_index } => {
            let Some(decl) = owner.verified_module.declarations().get(*decl_index) else {
                return Err(axiom_projection_error(&owner.module));
            };
            if !matches!(decl.decl, DeclPayload::Axiom { .. }) {
                return Err(axiom_projection_error(&owner.module));
            }
            let Some(name) = owner.verified_module.name_table().get(axiom.name) else {
                return Err(axiom_projection_error(&owner.module));
            };
            Ok(MachineStdAxiomRef {
                module: owner.module.clone(),
                name: name.clone(),
                export_hash: owner.expected_export_hash,
                decl_interface_hash: axiom.decl_interface_hash,
            })
        }
        GlobalRef::Imported {
            import_index,
            name,
            decl_interface_hash,
        } => {
            let Some(import) = owner.imports.get(*import_index) else {
                return Err(axiom_projection_error(&owner.module));
            };
            let Some(imported) = loaded.module(&import.module) else {
                return Err(axiom_projection_error(&owner.module));
            };
            if import.export_hash != imported.expected_export_hash
                || import.certificate_hash != Some(imported.expected_certificate_hash)
            {
                return Err(axiom_projection_error(&owner.module));
            }
            let Some(axiom_name) = owner.verified_module.name_table().get(*name) else {
                return Err(axiom_projection_error(&owner.module));
            };
            let matches_export = imported.verified_module.export_block().iter().any(|entry| {
                imported
                    .verified_module
                    .name_table()
                    .get(entry.name)
                    .is_some_and(|entry_name| {
                        entry.kind == ExportKind::Axiom
                            && entry_name == axiom_name
                            && entry.decl_interface_hash == *decl_interface_hash
                    })
            });
            if !matches_export || axiom.decl_interface_hash != *decl_interface_hash {
                return Err(axiom_projection_error(&owner.module));
            }
            Ok(MachineStdAxiomRef {
                module: imported.module.clone(),
                name: axiom_name.clone(),
                export_hash: imported.expected_export_hash,
                decl_interface_hash: *decl_interface_hash,
            })
        }
        GlobalRef::Builtin { .. } | GlobalRef::LocalGenerated { .. } => {
            Err(axiom_projection_error(&owner.module))
        }
    }
}

fn axiom_projection_error(module: &Name) -> MachineStdAxiomPolicyError {
    MachineStdAxiomPolicyError::AxiomRefProjectionFailed {
        module: module.clone(),
    }
}

fn expected_mvp_modules() -> Vec<Name> {
    machine_std_mvp_module_locators()
        .into_iter()
        .map(|locator| locator.module)
        .collect()
}

const LIBRARY_RELEASE_FIELDS: &[&str] = &[
    "protocol_version",
    "library_profile_id",
    "core_spec_id",
    "kernel_semantics_profile_id",
    "modules",
    "import_bundles_hash",
    "theorem_index_hash",
    "simp_profiles_hash",
    "rewrite_profiles_hash",
    "axiom_report_hash",
];
const MODULE_ARTIFACT_FIELDS: &[&str] = &[
    "module",
    "expected_export_hash",
    "expected_certificate_hash",
    "certificate_encoding",
    "certificate_bytes_hash",
    "axiom_report_hash",
    "public_export_count",
    "theorem_index_entry_count",
    "simp_rule_count",
];
const AXIOM_REPORT_FIELDS: &[&str] = &["library_profile_id", "modules", "axiom_report_hash"];
const MODULE_AXIOM_REPORT_FIELDS: &[&str] = &[
    "module",
    "export_hash",
    "certificate_hash",
    "module_axioms",
    "transitive_axioms",
];
const AXIOM_REF_FIELDS: &[&str] = &["module", "name", "export_hash", "decl_interface_hash"];
const IMPORT_BUNDLE_SET_FIELDS: &[&str] = &["library_profile_id", "bundles", "import_bundles_hash"];
const IMPORT_BUNDLE_FIELDS: &[&str] = &[
    "bundle_id",
    "root_imports",
    "import_closure",
    "allow_axioms",
    "recommended_tactic_options",
];
const IMPORT_KEY_FIELDS: &[&str] = &[
    "module",
    "expected_export_hash",
    "expected_certificate_hash",
];
const IMPORT_CERTIFICATE_FIELDS: &[&str] = &[
    "module",
    "expected_export_hash",
    "expected_certificate_hash",
    "certificate",
];
const CERTIFICATE_WRAPPER_FIELDS: &[&str] = &["encoding", "bytes"];
const TACTIC_OPTIONS_RECIPE_FIELDS: &[&str] = &[
    "recipe_id",
    "kernel_check_profile",
    "simp_rules",
    "eq_family",
    "nat_family",
    "max_simp_rewrite_steps",
    "max_open_goals",
    "max_metas",
];
const SIMP_RULE_FIELDS: &[&str] = &["name", "decl_interface_hash", "direction"];
const EQ_FAMILY_FIELDS: &[&str] = &[
    "eq_name",
    "eq_interface_hash",
    "refl_name",
    "refl_interface_hash",
    "rec_name",
    "rec_interface_hash",
];
const NAT_FAMILY_FIELDS: &[&str] = &[
    "nat_name",
    "nat_interface_hash",
    "zero_name",
    "zero_interface_hash",
    "succ_name",
    "succ_interface_hash",
    "rec_name",
    "rec_interface_hash",
];

fn parse_std_json<'src>(
    source: &'src str,
    artifact: MachineStdArtifactKind,
) -> Result<JsonDocument<'src>, MachineStdArtifactShapeError> {
    JsonDocument::parse(source).map_err(|err| MachineStdArtifactShapeError {
        artifact,
        path: "$".to_owned(),
        reason: MachineStdArtifactShapeErrorReason::JsonParse {
            offset: err.offset,
            kind: err.kind,
        },
    })
}

fn parse_library_release_value(
    value: &JsonValue<'_>,
    path: &str,
) -> Result<MachineStdLibraryRelease, MachineStdArtifactShapeError> {
    let members = validated_object_members(
        value,
        MachineStdArtifactKind::LibraryRelease,
        path,
        LIBRARY_RELEASE_FIELDS,
    )?;
    let modules = required_array(
        members,
        MachineStdArtifactKind::LibraryRelease,
        path,
        "modules",
    )?
    .iter()
    .enumerate()
    .map(|(index, item)| parse_module_artifact_value(item, &array_path(path, "modules", index)))
    .collect::<Result<Vec<_>, _>>()?;

    Ok(MachineStdLibraryRelease {
        protocol_version: required_string(
            members,
            MachineStdArtifactKind::LibraryRelease,
            path,
            "protocol_version",
        )?
        .to_owned(),
        library_profile_id: required_string(
            members,
            MachineStdArtifactKind::LibraryRelease,
            path,
            "library_profile_id",
        )?
        .to_owned(),
        core_spec_id: required_string(
            members,
            MachineStdArtifactKind::LibraryRelease,
            path,
            "core_spec_id",
        )?
        .to_owned(),
        kernel_semantics_profile_id: required_string(
            members,
            MachineStdArtifactKind::LibraryRelease,
            path,
            "kernel_semantics_profile_id",
        )?
        .to_owned(),
        modules,
        import_bundles_hash: required_hash(
            members,
            MachineStdArtifactKind::LibraryRelease,
            path,
            "import_bundles_hash",
        )?,
        theorem_index_hash: required_hash(
            members,
            MachineStdArtifactKind::LibraryRelease,
            path,
            "theorem_index_hash",
        )?,
        simp_profiles_hash: required_hash(
            members,
            MachineStdArtifactKind::LibraryRelease,
            path,
            "simp_profiles_hash",
        )?,
        rewrite_profiles_hash: required_hash(
            members,
            MachineStdArtifactKind::LibraryRelease,
            path,
            "rewrite_profiles_hash",
        )?,
        axiom_report_hash: required_hash(
            members,
            MachineStdArtifactKind::LibraryRelease,
            path,
            "axiom_report_hash",
        )?,
    })
}

fn parse_module_artifact_value(
    value: &JsonValue<'_>,
    path: &str,
) -> Result<MachineStdModuleArtifact, MachineStdArtifactShapeError> {
    let members = validated_object_members(
        value,
        MachineStdArtifactKind::LibraryRelease,
        path,
        MODULE_ARTIFACT_FIELDS,
    )?;
    Ok(MachineStdModuleArtifact {
        module: required_module_name(
            members,
            MachineStdArtifactKind::LibraryRelease,
            path,
            "module",
        )?,
        expected_export_hash: required_hash(
            members,
            MachineStdArtifactKind::LibraryRelease,
            path,
            "expected_export_hash",
        )?,
        expected_certificate_hash: required_hash(
            members,
            MachineStdArtifactKind::LibraryRelease,
            path,
            "expected_certificate_hash",
        )?,
        certificate_encoding: required_string(
            members,
            MachineStdArtifactKind::LibraryRelease,
            path,
            "certificate_encoding",
        )?
        .to_owned(),
        certificate_bytes_hash: required_hash(
            members,
            MachineStdArtifactKind::LibraryRelease,
            path,
            "certificate_bytes_hash",
        )?,
        axiom_report_hash: required_hash(
            members,
            MachineStdArtifactKind::LibraryRelease,
            path,
            "axiom_report_hash",
        )?,
        public_export_count: required_u64(
            members,
            MachineStdArtifactKind::LibraryRelease,
            path,
            "public_export_count",
        )?,
        theorem_index_entry_count: required_u64(
            members,
            MachineStdArtifactKind::LibraryRelease,
            path,
            "theorem_index_entry_count",
        )?,
        simp_rule_count: required_u64(
            members,
            MachineStdArtifactKind::LibraryRelease,
            path,
            "simp_rule_count",
        )?,
    })
}

fn parse_import_bundle_set_value(
    value: &JsonValue<'_>,
    path: &str,
) -> Result<MachineStdImportBundleSet, MachineStdArtifactShapeError> {
    let members = validated_object_members(
        value,
        MachineStdArtifactKind::ImportBundles,
        path,
        IMPORT_BUNDLE_SET_FIELDS,
    )?;
    let bundles = required_array(
        members,
        MachineStdArtifactKind::ImportBundles,
        path,
        "bundles",
    )?
    .iter()
    .enumerate()
    .map(|(index, item)| parse_import_bundle_value(item, &array_path(path, "bundles", index)))
    .collect::<Result<Vec<_>, _>>()?;
    Ok(MachineStdImportBundleSet {
        library_profile_id: required_string(
            members,
            MachineStdArtifactKind::ImportBundles,
            path,
            "library_profile_id",
        )?
        .to_owned(),
        bundles,
        import_bundles_hash: required_hash(
            members,
            MachineStdArtifactKind::ImportBundles,
            path,
            "import_bundles_hash",
        )?,
    })
}

fn parse_import_bundle_value(
    value: &JsonValue<'_>,
    path: &str,
) -> Result<MachineStdImportBundle, MachineStdArtifactShapeError> {
    let members = validated_object_members(
        value,
        MachineStdArtifactKind::ImportBundles,
        path,
        IMPORT_BUNDLE_FIELDS,
    )?;
    Ok(MachineStdImportBundle {
        bundle_id: required_string(
            members,
            MachineStdArtifactKind::ImportBundles,
            path,
            "bundle_id",
        )?
        .to_owned(),
        root_imports: parse_import_key_array(members, path, "root_imports")?,
        import_closure: parse_import_certificate_array(members, path, "import_closure")?,
        allow_axioms: parse_machine_axiom_ref_wire_array(members, path, "allow_axioms")?,
        recommended_tactic_options: parse_tactic_options_recipe_value(
            required_value(members, "recommended_tactic_options"),
            &field_path(path, "recommended_tactic_options"),
        )?,
    })
}

fn parse_import_key_array(
    members: &[crate::json::JsonMember<'_>],
    path: &str,
    field: &'static str,
) -> Result<Vec<VerifiedImportKey>, MachineStdArtifactShapeError> {
    required_array(members, MachineStdArtifactKind::ImportBundles, path, field)?
        .iter()
        .enumerate()
        .map(|(index, item)| parse_import_key_value(item, &array_path(path, field, index)))
        .collect()
}

fn parse_import_certificate_array(
    members: &[crate::json::JsonMember<'_>],
    path: &str,
    field: &'static str,
) -> Result<Vec<MachineStdImportCertificate>, MachineStdArtifactShapeError> {
    required_array(members, MachineStdArtifactKind::ImportBundles, path, field)?
        .iter()
        .enumerate()
        .map(|(index, item)| parse_import_certificate_value(item, &array_path(path, field, index)))
        .collect()
}

fn parse_import_key_value(
    value: &JsonValue<'_>,
    path: &str,
) -> Result<VerifiedImportKey, MachineStdArtifactShapeError> {
    let members = validated_object_members(
        value,
        MachineStdArtifactKind::ImportBundles,
        path,
        IMPORT_KEY_FIELDS,
    )?;
    Ok(VerifiedImportKey::new(
        required_module_name(
            members,
            MachineStdArtifactKind::ImportBundles,
            path,
            "module",
        )?,
        required_hash(
            members,
            MachineStdArtifactKind::ImportBundles,
            path,
            "expected_export_hash",
        )?,
        required_hash(
            members,
            MachineStdArtifactKind::ImportBundles,
            path,
            "expected_certificate_hash",
        )?,
    ))
}

fn parse_import_certificate_value(
    value: &JsonValue<'_>,
    path: &str,
) -> Result<MachineStdImportCertificate, MachineStdArtifactShapeError> {
    let members = validated_object_members(
        value,
        MachineStdArtifactKind::ImportBundles,
        path,
        IMPORT_CERTIFICATE_FIELDS,
    )?;
    let certificate_members = validated_object_members(
        required_value(members, "certificate"),
        MachineStdArtifactKind::ImportBundles,
        &field_path(path, "certificate"),
        CERTIFICATE_WRAPPER_FIELDS,
    )?;
    Ok(MachineStdImportCertificate {
        module: required_module_name(
            members,
            MachineStdArtifactKind::ImportBundles,
            path,
            "module",
        )?,
        expected_export_hash: required_hash(
            members,
            MachineStdArtifactKind::ImportBundles,
            path,
            "expected_export_hash",
        )?,
        expected_certificate_hash: required_hash(
            members,
            MachineStdArtifactKind::ImportBundles,
            path,
            "expected_certificate_hash",
        )?,
        certificate_encoding: required_string(
            certificate_members,
            MachineStdArtifactKind::ImportBundles,
            &field_path(path, "certificate"),
            "encoding",
        )?
        .to_owned(),
        certificate_bytes: required_hex_bytes(
            certificate_members,
            MachineStdArtifactKind::ImportBundles,
            &field_path(path, "certificate"),
            "bytes",
        )?,
    })
}

fn parse_tactic_options_recipe_value(
    value: &JsonValue<'_>,
    path: &str,
) -> Result<MachineStdTacticOptionsRecipe, MachineStdArtifactShapeError> {
    let members = validated_object_members(
        value,
        MachineStdArtifactKind::ImportBundles,
        path,
        TACTIC_OPTIONS_RECIPE_FIELDS,
    )?;
    Ok(MachineStdTacticOptionsRecipe {
        recipe_id: required_string(
            members,
            MachineStdArtifactKind::ImportBundles,
            path,
            "recipe_id",
        )?
        .to_owned(),
        kernel_check_profile: required_string(
            members,
            MachineStdArtifactKind::ImportBundles,
            path,
            "kernel_check_profile",
        )?
        .to_owned(),
        simp_rules: parse_simp_rule_array(members, path, "simp_rules")?,
        eq_family: parse_optional_eq_family_value(required_value(members, "eq_family"), path)?,
        nat_family: parse_optional_nat_family_value(required_value(members, "nat_family"), path)?,
        max_simp_rewrite_steps: required_u64(
            members,
            MachineStdArtifactKind::ImportBundles,
            path,
            "max_simp_rewrite_steps",
        )?,
        max_open_goals: required_u64(
            members,
            MachineStdArtifactKind::ImportBundles,
            path,
            "max_open_goals",
        )?,
        max_metas: required_u64(
            members,
            MachineStdArtifactKind::ImportBundles,
            path,
            "max_metas",
        )?,
    })
}

fn parse_simp_rule_array(
    members: &[crate::json::JsonMember<'_>],
    path: &str,
    field: &'static str,
) -> Result<Vec<SimpRuleRef>, MachineStdArtifactShapeError> {
    required_array(members, MachineStdArtifactKind::ImportBundles, path, field)?
        .iter()
        .enumerate()
        .map(|(index, item)| parse_simp_rule_value(item, &array_path(path, field, index)))
        .collect()
}

fn parse_simp_rule_value(
    value: &JsonValue<'_>,
    path: &str,
) -> Result<SimpRuleRef, MachineStdArtifactShapeError> {
    let members = validated_object_members(
        value,
        MachineStdArtifactKind::ImportBundles,
        path,
        SIMP_RULE_FIELDS,
    )?;
    let direction = match required_string(
        members,
        MachineStdArtifactKind::ImportBundles,
        path,
        "direction",
    )? {
        "forward" => RewriteDirection::Forward,
        "backward" => RewriteDirection::Backward,
        _ => {
            return Err(shape_error(
                MachineStdArtifactKind::ImportBundles,
                &field_path(path, "direction"),
                MachineStdArtifactShapeErrorReason::InvalidEnumString { field: "direction" },
            ));
        }
    };
    Ok(SimpRuleRef {
        name: required_fully_qualified_name(
            members,
            MachineStdArtifactKind::ImportBundles,
            path,
            "name",
        )?,
        decl_interface_hash: required_hash(
            members,
            MachineStdArtifactKind::ImportBundles,
            path,
            "decl_interface_hash",
        )?,
        direction,
    })
}

fn parse_optional_eq_family_value(
    value: &JsonValue<'_>,
    path: &str,
) -> Result<Option<EqFamilyRef>, MachineStdArtifactShapeError> {
    if value.kind() == JsonValueKind::Null {
        return Ok(None);
    }
    let path = field_path(path, "eq_family");
    let members = validated_object_members(
        value,
        MachineStdArtifactKind::ImportBundles,
        &path,
        EQ_FAMILY_FIELDS,
    )?;
    Ok(Some(EqFamilyRef {
        eq_name: required_fully_qualified_name(
            members,
            MachineStdArtifactKind::ImportBundles,
            &path,
            "eq_name",
        )?,
        eq_interface_hash: required_hash(
            members,
            MachineStdArtifactKind::ImportBundles,
            &path,
            "eq_interface_hash",
        )?,
        refl_name: required_fully_qualified_name(
            members,
            MachineStdArtifactKind::ImportBundles,
            &path,
            "refl_name",
        )?,
        refl_interface_hash: required_hash(
            members,
            MachineStdArtifactKind::ImportBundles,
            &path,
            "refl_interface_hash",
        )?,
        rec_name: required_fully_qualified_name(
            members,
            MachineStdArtifactKind::ImportBundles,
            &path,
            "rec_name",
        )?,
        rec_interface_hash: required_hash(
            members,
            MachineStdArtifactKind::ImportBundles,
            &path,
            "rec_interface_hash",
        )?,
    }))
}

fn parse_optional_nat_family_value(
    value: &JsonValue<'_>,
    path: &str,
) -> Result<Option<NatFamilyRef>, MachineStdArtifactShapeError> {
    if value.kind() == JsonValueKind::Null {
        return Ok(None);
    }
    let path = field_path(path, "nat_family");
    let members = validated_object_members(
        value,
        MachineStdArtifactKind::ImportBundles,
        &path,
        NAT_FAMILY_FIELDS,
    )?;
    Ok(Some(NatFamilyRef {
        nat_name: required_fully_qualified_name(
            members,
            MachineStdArtifactKind::ImportBundles,
            &path,
            "nat_name",
        )?,
        nat_interface_hash: required_hash(
            members,
            MachineStdArtifactKind::ImportBundles,
            &path,
            "nat_interface_hash",
        )?,
        zero_name: required_fully_qualified_name(
            members,
            MachineStdArtifactKind::ImportBundles,
            &path,
            "zero_name",
        )?,
        zero_interface_hash: required_hash(
            members,
            MachineStdArtifactKind::ImportBundles,
            &path,
            "zero_interface_hash",
        )?,
        succ_name: required_fully_qualified_name(
            members,
            MachineStdArtifactKind::ImportBundles,
            &path,
            "succ_name",
        )?,
        succ_interface_hash: required_hash(
            members,
            MachineStdArtifactKind::ImportBundles,
            &path,
            "succ_interface_hash",
        )?,
        rec_name: required_fully_qualified_name(
            members,
            MachineStdArtifactKind::ImportBundles,
            &path,
            "rec_name",
        )?,
        rec_interface_hash: required_hash(
            members,
            MachineStdArtifactKind::ImportBundles,
            &path,
            "rec_interface_hash",
        )?,
    }))
}

fn parse_machine_axiom_ref_wire_array(
    members: &[crate::json::JsonMember<'_>],
    path: &str,
    field: &'static str,
) -> Result<Vec<MachineAxiomRefWire>, MachineStdArtifactShapeError> {
    required_array(members, MachineStdArtifactKind::ImportBundles, path, field)?
        .iter()
        .enumerate()
        .map(|(index, item)| {
            parse_machine_axiom_ref_wire_value(item, &array_path(path, field, index))
        })
        .collect()
}

fn parse_machine_axiom_ref_wire_value(
    value: &JsonValue<'_>,
    path: &str,
) -> Result<MachineAxiomRefWire, MachineStdArtifactShapeError> {
    let members = validated_machine_axiom_ref_wire_members(value, path)?;
    let kind = required_string(members, MachineStdArtifactKind::ImportBundles, path, "kind")?;
    match kind {
        "imported" => {
            let members = validated_object_members(
                value,
                MachineStdArtifactKind::ImportBundles,
                path,
                &[
                    "kind",
                    "module",
                    "name",
                    "export_hash",
                    "decl_interface_hash",
                ],
            )?;
            Ok(MachineAxiomRefWire::Imported {
                module: required_module_name(
                    members,
                    MachineStdArtifactKind::ImportBundles,
                    path,
                    "module",
                )?,
                name: required_fully_qualified_name(
                    members,
                    MachineStdArtifactKind::ImportBundles,
                    path,
                    "name",
                )?,
                export_hash: required_hash(
                    members,
                    MachineStdArtifactKind::ImportBundles,
                    path,
                    "export_hash",
                )?,
                decl_interface_hash: required_hash(
                    members,
                    MachineStdArtifactKind::ImportBundles,
                    path,
                    "decl_interface_hash",
                )?,
            })
        }
        "current_module" => {
            let members = validated_object_members(
                value,
                MachineStdArtifactKind::ImportBundles,
                path,
                &[
                    "kind",
                    "module",
                    "name",
                    "source_index",
                    "decl_interface_hash",
                ],
            )?;
            Ok(MachineAxiomRefWire::CurrentModule {
                module: required_module_name(
                    members,
                    MachineStdArtifactKind::ImportBundles,
                    path,
                    "module",
                )?,
                name: required_fully_qualified_name(
                    members,
                    MachineStdArtifactKind::ImportBundles,
                    path,
                    "name",
                )?,
                source_index: required_u64(
                    members,
                    MachineStdArtifactKind::ImportBundles,
                    path,
                    "source_index",
                )?,
                decl_interface_hash: required_hash(
                    members,
                    MachineStdArtifactKind::ImportBundles,
                    path,
                    "decl_interface_hash",
                )?,
            })
        }
        "builtin" => {
            let members = validated_object_members(
                value,
                MachineStdArtifactKind::ImportBundles,
                path,
                &["kind", "name", "decl_interface_hash"],
            )?;
            Ok(MachineAxiomRefWire::Builtin {
                name: required_fully_qualified_name(
                    members,
                    MachineStdArtifactKind::ImportBundles,
                    path,
                    "name",
                )?,
                decl_interface_hash: required_hash(
                    members,
                    MachineStdArtifactKind::ImportBundles,
                    path,
                    "decl_interface_hash",
                )?,
            })
        }
        _ => Err(shape_error(
            MachineStdArtifactKind::ImportBundles,
            &field_path(path, "kind"),
            MachineStdArtifactShapeErrorReason::InvalidEnumString { field: "kind" },
        )),
    }
}

fn validated_machine_axiom_ref_wire_members<'value, 'src>(
    value: &'value JsonValue<'src>,
    path: &str,
) -> Result<&'value [crate::json::JsonMember<'src>], MachineStdArtifactShapeError> {
    let Some(members) = value.object_members() else {
        return Err(shape_error(
            MachineStdArtifactKind::ImportBundles,
            path,
            MachineStdArtifactShapeErrorReason::ExpectedObject {
                actual: value.kind(),
            },
        ));
    };
    let mut seen = BTreeSet::new();
    for member in members {
        if !seen.insert(member.key().to_owned()) {
            return Err(shape_error(
                MachineStdArtifactKind::ImportBundles,
                &field_path(path, member.key()),
                MachineStdArtifactShapeErrorReason::DuplicateKey {
                    key: member.key().to_owned(),
                },
            ));
        }
    }
    if !members.iter().any(|member| member.key() == "kind") {
        return Err(shape_error(
            MachineStdArtifactKind::ImportBundles,
            &field_path(path, "kind"),
            MachineStdArtifactShapeErrorReason::MissingField { field: "kind" },
        ));
    }
    Ok(members)
}

fn parse_axiom_report_value(
    value: &JsonValue<'_>,
    path: &str,
) -> Result<MachineStdAxiomReport, MachineStdArtifactShapeError> {
    let members = validated_object_members(
        value,
        MachineStdArtifactKind::AxiomReport,
        path,
        AXIOM_REPORT_FIELDS,
    )?;
    let modules = required_array(
        members,
        MachineStdArtifactKind::AxiomReport,
        path,
        "modules",
    )?
    .iter()
    .enumerate()
    .map(|(index, item)| parse_module_axiom_report_value(item, &array_path(path, "modules", index)))
    .collect::<Result<Vec<_>, _>>()?;
    Ok(MachineStdAxiomReport {
        library_profile_id: required_string(
            members,
            MachineStdArtifactKind::AxiomReport,
            path,
            "library_profile_id",
        )?
        .to_owned(),
        modules,
        axiom_report_hash: required_hash(
            members,
            MachineStdArtifactKind::AxiomReport,
            path,
            "axiom_report_hash",
        )?,
    })
}

fn parse_module_axiom_report_value(
    value: &JsonValue<'_>,
    path: &str,
) -> Result<MachineStdModuleAxiomReport, MachineStdArtifactShapeError> {
    let members = validated_object_members(
        value,
        MachineStdArtifactKind::AxiomReport,
        path,
        MODULE_AXIOM_REPORT_FIELDS,
    )?;
    Ok(MachineStdModuleAxiomReport {
        module: required_module_name(members, MachineStdArtifactKind::AxiomReport, path, "module")?,
        export_hash: required_hash(
            members,
            MachineStdArtifactKind::AxiomReport,
            path,
            "export_hash",
        )?,
        certificate_hash: required_hash(
            members,
            MachineStdArtifactKind::AxiomReport,
            path,
            "certificate_hash",
        )?,
        module_axioms: parse_axiom_ref_array(members, path, "module_axioms")?,
        transitive_axioms: parse_axiom_ref_array(members, path, "transitive_axioms")?,
    })
}

fn parse_axiom_ref_array(
    members: &[crate::json::JsonMember<'_>],
    path: &str,
    field: &'static str,
) -> Result<Vec<MachineStdAxiomRef>, MachineStdArtifactShapeError> {
    required_array(members, MachineStdArtifactKind::AxiomReport, path, field)?
        .iter()
        .enumerate()
        .map(|(index, item)| parse_axiom_ref_value(item, &array_path(path, field, index)))
        .collect()
}

fn parse_axiom_ref_value(
    value: &JsonValue<'_>,
    path: &str,
) -> Result<MachineStdAxiomRef, MachineStdArtifactShapeError> {
    let members = validated_object_members(
        value,
        MachineStdArtifactKind::AxiomReport,
        path,
        AXIOM_REF_FIELDS,
    )?;
    Ok(MachineStdAxiomRef {
        module: required_module_name(members, MachineStdArtifactKind::AxiomReport, path, "module")?,
        name: required_fully_qualified_name(
            members,
            MachineStdArtifactKind::AxiomReport,
            path,
            "name",
        )?,
        export_hash: required_hash(
            members,
            MachineStdArtifactKind::AxiomReport,
            path,
            "export_hash",
        )?,
        decl_interface_hash: required_hash(
            members,
            MachineStdArtifactKind::AxiomReport,
            path,
            "decl_interface_hash",
        )?,
    })
}

fn validated_object_members<'value, 'src>(
    value: &'value JsonValue<'src>,
    artifact: MachineStdArtifactKind,
    path: &str,
    allowed_fields: &[&'static str],
) -> Result<&'value [crate::json::JsonMember<'src>], MachineStdArtifactShapeError> {
    let Some(members) = value.object_members() else {
        return Err(shape_error(
            artifact,
            path,
            MachineStdArtifactShapeErrorReason::ExpectedObject {
                actual: value.kind(),
            },
        ));
    };
    let mut seen = BTreeSet::new();
    for member in members {
        if !seen.insert(member.key().to_owned()) {
            return Err(shape_error(
                artifact,
                &field_path(path, member.key()),
                MachineStdArtifactShapeErrorReason::DuplicateKey {
                    key: member.key().to_owned(),
                },
            ));
        }
    }
    for member in members {
        if !allowed_fields.iter().any(|field| *field == member.key()) {
            return Err(shape_error(
                artifact,
                &field_path(path, member.key()),
                MachineStdArtifactShapeErrorReason::UnknownField {
                    field: member.key().to_owned(),
                },
            ));
        }
    }
    for field in allowed_fields {
        if !members.iter().any(|member| member.key() == *field) {
            return Err(shape_error(
                artifact,
                &field_path(path, field),
                MachineStdArtifactShapeErrorReason::MissingField { field },
            ));
        }
    }
    Ok(members)
}

fn required_value<'value, 'src>(
    members: &'value [crate::json::JsonMember<'src>],
    field: &'static str,
) -> &'value JsonValue<'src> {
    members
        .iter()
        .find(|member| member.key() == field)
        .expect("validated object contains required field")
        .value()
}

fn required_string<'value, 'src>(
    members: &'value [crate::json::JsonMember<'src>],
    artifact: MachineStdArtifactKind,
    path: &str,
    field: &'static str,
) -> Result<&'value str, MachineStdArtifactShapeError> {
    let value = required_value(members, field);
    match value.kind() {
        JsonValueKind::Null => Err(shape_error(
            artifact,
            &field_path(path, field),
            MachineStdArtifactShapeErrorReason::NullField { field },
        )),
        JsonValueKind::String => Ok(value.string_value().expect("kind checked string")),
        actual => Err(shape_error(
            artifact,
            &field_path(path, field),
            MachineStdArtifactShapeErrorReason::TypeMismatch {
                field,
                expected: "string",
                actual,
            },
        )),
    }
}

fn required_array<'value, 'src>(
    members: &'value [crate::json::JsonMember<'src>],
    artifact: MachineStdArtifactKind,
    path: &str,
    field: &'static str,
) -> Result<&'value [JsonValue<'src>], MachineStdArtifactShapeError> {
    let value = required_value(members, field);
    match value.kind() {
        JsonValueKind::Null => Err(shape_error(
            artifact,
            &field_path(path, field),
            MachineStdArtifactShapeErrorReason::NullField { field },
        )),
        JsonValueKind::Array => Ok(value.array_elements().expect("kind checked array")),
        actual => Err(shape_error(
            artifact,
            &field_path(path, field),
            MachineStdArtifactShapeErrorReason::ExpectedArray { actual },
        )),
    }
}

fn required_hash(
    members: &[crate::json::JsonMember<'_>],
    artifact: MachineStdArtifactKind,
    path: &str,
    field: &'static str,
) -> Result<Hash, MachineStdArtifactShapeError> {
    let value = required_string(members, artifact, path, field)?;
    parse_hash_string(value).map_err(|_| {
        shape_error(
            artifact,
            &field_path(path, field),
            MachineStdArtifactShapeErrorReason::InvalidHashString { field },
        )
    })
}

fn required_hex_bytes(
    members: &[crate::json::JsonMember<'_>],
    artifact: MachineStdArtifactKind,
    path: &str,
    field: &'static str,
) -> Result<Vec<u8>, MachineStdArtifactShapeError> {
    let value = required_string(members, artifact, path, field)?;
    decode_lower_hex_bytes(value).map_err(|_| {
        shape_error(
            artifact,
            &field_path(path, field),
            MachineStdArtifactShapeErrorReason::InvalidHexString { field },
        )
    })
}

fn required_module_name(
    members: &[crate::json::JsonMember<'_>],
    artifact: MachineStdArtifactKind,
    path: &str,
    field: &'static str,
) -> Result<Name, MachineStdArtifactShapeError> {
    let value = required_string(members, artifact, path, field)?;
    parse_module_name_wire(value).map_err(|_| {
        shape_error(
            artifact,
            &field_path(path, field),
            MachineStdArtifactShapeErrorReason::InvalidName { field },
        )
    })
}

fn required_fully_qualified_name(
    members: &[crate::json::JsonMember<'_>],
    artifact: MachineStdArtifactKind,
    path: &str,
    field: &'static str,
) -> Result<Name, MachineStdArtifactShapeError> {
    let value = required_string(members, artifact, path, field)?;
    parse_fully_qualified_name_wire(value).map_err(|_| {
        shape_error(
            artifact,
            &field_path(path, field),
            MachineStdArtifactShapeErrorReason::InvalidName { field },
        )
    })
}

fn required_u64(
    members: &[crate::json::JsonMember<'_>],
    artifact: MachineStdArtifactKind,
    path: &str,
    field: &'static str,
) -> Result<u64, MachineStdArtifactShapeError> {
    let value = required_value(members, field);
    match value.kind() {
        JsonValueKind::Null => Err(shape_error(
            artifact,
            &field_path(path, field),
            MachineStdArtifactShapeErrorReason::NullField { field },
        )),
        JsonValueKind::Number => {
            let raw = value.number_raw().expect("kind checked number");
            parse_strict_u64_token(raw, u64::MAX).map_err(|error| {
                shape_error(
                    artifact,
                    &field_path(path, field),
                    MachineStdArtifactShapeErrorReason::InvalidUnsignedInteger {
                        field,
                        raw: raw.to_owned(),
                        error,
                    },
                )
            })
        }
        actual => Err(shape_error(
            artifact,
            &field_path(path, field),
            MachineStdArtifactShapeErrorReason::TypeMismatch {
                field,
                expected: "unsigned integer",
                actual,
            },
        )),
    }
}

fn decode_lower_hex_bytes(value: &str) -> Result<Vec<u8>, ()> {
    if !value.len().is_multiple_of(2) {
        return Err(());
    }
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|chunk| {
            let high = lowercase_hex_value(chunk[0])?;
            let low = lowercase_hex_value(chunk[1])?;
            Ok((high << 4) | low)
        })
        .collect()
}

fn lowercase_hex_value(byte: u8) -> Result<u8, ()> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => Err(()),
    }
}

fn shape_error(
    artifact: MachineStdArtifactKind,
    path: &str,
    reason: MachineStdArtifactShapeErrorReason,
) -> MachineStdArtifactShapeError {
    MachineStdArtifactShapeError {
        artifact,
        path: path.to_owned(),
        reason,
    }
}

fn field_path(path: &str, field: &str) -> String {
    format!("{path}.{field}")
}

fn array_path(path: &str, field: &str, index: usize) -> String {
    format!("{path}.{field}[{index}]")
}

fn canonical_package_root(package_root: &Path) -> Result<PathBuf, MachineStdReleaseLoaderError> {
    fs::canonicalize(package_root).map_err(|source| {
        MachineStdReleaseLoaderError::InvalidPackageRoot {
            path: package_root.to_path_buf(),
            source,
        }
    })
}

fn read_and_decode_std_modules(
    package_root: &Path,
    locators: &[MachineStdModuleLocator],
) -> Result<BTreeMap<Name, DecodedStdModule>, MachineStdReleaseLoaderError> {
    let mut decoded = BTreeMap::new();
    for locator in locators {
        let (resolved_path, certificate_bytes) = read_locator_certificate(package_root, locator)?;
        let certificate_bytes_hash = sha256(&certificate_bytes);
        let cert = decode_module_cert(&certificate_bytes).map_err(|source| {
            MachineStdReleaseLoaderError::DecodeFailed {
                module: locator.module.clone(),
                source: Box::new(source),
            }
        })?;
        if cert.header.module != locator.module {
            return Err(MachineStdReleaseLoaderError::ModuleNameMismatch {
                expected: locator.module.clone(),
                actual: cert.header.module.clone(),
            });
        }
        decoded.insert(
            locator.module.clone(),
            DecodedStdModule {
                locator: locator.clone(),
                resolved_path,
                certificate_bytes,
                certificate_bytes_hash,
                cert,
            },
        );
    }
    Ok(decoded)
}

fn read_locator_certificate(
    package_root: &Path,
    locator: &MachineStdModuleLocator,
) -> Result<(PathBuf, Vec<u8>), MachineStdReleaseLoaderError> {
    let path = join_posix_relative_path(package_root, &locator.relative_path);
    let resolved = fs::canonicalize(&path).map_err(|source| {
        if source.kind() == io::ErrorKind::NotFound {
            MachineStdReleaseLoaderError::MissingCertificateFile {
                module: locator.module.clone(),
                path: path.clone(),
                source,
            }
        } else {
            MachineStdReleaseLoaderError::ReadCertificateFile {
                module: locator.module.clone(),
                path: path.clone(),
                source,
            }
        }
    })?;
    if !resolved.starts_with(package_root) {
        return Err(MachineStdReleaseLoaderError::SymlinkEscape {
            module: locator.module.clone(),
            path,
            resolved,
            package_root: package_root.to_path_buf(),
        });
    }
    let bytes = fs::read(&resolved).map_err(|source| {
        MachineStdReleaseLoaderError::ReadCertificateFile {
            module: locator.module.clone(),
            path: resolved.clone(),
            source,
        }
    })?;
    Ok((resolved, bytes))
}

fn join_posix_relative_path(root: &Path, relative_path: &str) -> PathBuf {
    let mut path = root.to_path_buf();
    for component in relative_path.split('/') {
        path.push(component);
    }
    path
}

fn validate_import_graph(
    modules: &BTreeMap<Name, DecodedStdModule>,
) -> Result<(), MachineStdReleaseLoaderError> {
    let mut keys = BTreeSet::new();
    for module in modules.values() {
        keys.insert(CertificateKey {
            module: module.cert.header.module.clone(),
            export_hash: module.cert.hashes.export_hash,
            certificate_hash: module.cert.hashes.certificate_hash,
        });
    }

    for module in modules.values() {
        for import in &module.cert.imports {
            let certificate_hash = import.certificate_hash.ok_or_else(|| {
                MachineStdReleaseLoaderError::MissingImportCertificateHash {
                    owner: module.cert.header.module.clone(),
                    imported_module: import.module.clone(),
                }
            })?;
            if !modules.contains_key(&import.module) {
                return Err(MachineStdReleaseLoaderError::UnresolvedImport {
                    owner: module.cert.header.module.clone(),
                    imported_module: import.module.clone(),
                });
            }
            let key = CertificateKey {
                module: import.module.clone(),
                export_hash: import.export_hash,
                certificate_hash,
            };
            if !keys.contains(&key) {
                return Err(MachineStdReleaseLoaderError::ImportHashMismatch {
                    owner: module.cert.header.module.clone(),
                    imported_module: import.module.clone(),
                });
            }
        }
    }
    Ok(())
}

fn topological_verification_order(
    modules: &BTreeMap<Name, DecodedStdModule>,
) -> Result<Vec<Name>, MachineStdReleaseLoaderError> {
    let mut remaining = modules.keys().cloned().collect::<BTreeSet<_>>();
    let mut order = Vec::new();

    while !remaining.is_empty() {
        let mut ready = Vec::new();
        for module in &remaining {
            let record = modules
                .get(module)
                .expect("remaining module came from decoded module table");
            if record
                .cert
                .imports
                .iter()
                .all(|import| !remaining.contains(&import.module))
            {
                ready.push(module.clone());
            }
        }

        let next = ready
            .into_iter()
            .min_by(compare_module_names)
            .ok_or_else(|| {
                let module = remaining
                    .iter()
                    .min_by(|lhs, rhs| compare_module_names(lhs, rhs))
                    .expect("remaining is non-empty")
                    .clone();
                MachineStdReleaseLoaderError::ImportCycle { module }
            })?;
        remaining.remove(&next);
        order.push(next);
    }

    Ok(order)
}

fn compare_module_names(lhs: &Name, rhs: &Name) -> std::cmp::Ordering {
    let lhs_bytes = module_name_canonical_bytes(lhs).unwrap_or_default();
    let rhs_bytes = module_name_canonical_bytes(rhs).unwrap_or_default();
    lhs_bytes.cmp(&rhs_bytes)
}

fn module_name_canonical_bytes(module: &Name) -> Result<Vec<u8>, MachineStdReleaseLoaderError> {
    phase5_name_canonical_bytes(module).map_err(|source| {
        MachineStdReleaseLoaderError::InvalidCanonicalModuleName {
            module: module.clone(),
            source: Box::new(source),
        }
    })
}

fn verify_decoded_modules(
    decoded: BTreeMap<Name, DecodedStdModule>,
    verification_order: Vec<Name>,
    policy: AxiomPolicy,
) -> Result<MachineStdLoadedRelease, MachineStdReleaseLoaderError> {
    let mut session = VerifierSession::new();
    let mut verified_by_module = BTreeMap::new();

    for module in &verification_order {
        let record = decoded
            .get(module)
            .expect("verification order came from decoded module table");
        let verified = verify_module_cert(&record.certificate_bytes, &mut session, &policy)
            .map_err(|source| MachineStdReleaseLoaderError::VerifyFailed {
                module: module.clone(),
                source: Box::new(source),
            })?;
        if verified.module() != module
            || verified.export_hash() != record.cert.hashes.export_hash
            || verified.certificate_hash() != record.cert.hashes.certificate_hash
        {
            return Err(MachineStdReleaseLoaderError::VerifiedIdentityMismatch {
                module: module.clone(),
            });
        }
        verified_by_module.insert(module.clone(), verified);
    }

    let mut modules = Vec::new();
    for locator in machine_std_mvp_module_locators() {
        let record = decoded
            .get(&locator.module)
            .expect("validated locators contain every MVP module");
        let verified_module = verified_by_module
            .remove(&locator.module)
            .expect("every decoded module was verified");
        modules.push(MachineStdLoadedModule {
            module: locator.module.clone(),
            locator_path: record.locator.relative_path.clone(),
            resolved_path: record.resolved_path.clone(),
            certificate_bytes: record.certificate_bytes.clone(),
            certificate_bytes_hash: record.certificate_bytes_hash,
            expected_export_hash: record.cert.hashes.export_hash,
            expected_certificate_hash: record.cert.hashes.certificate_hash,
            axiom_report_hash: record.cert.hashes.axiom_report_hash,
            imports: verified_module.imports().to_vec(),
            verified_module,
        });
    }

    let module_index = modules
        .iter()
        .enumerate()
        .map(|(index, module)| (module.module.clone(), index))
        .collect();

    Ok(MachineStdLoadedRelease {
        modules,
        module_index,
        verification_order,
    })
}

fn machine_std_kernel_check_profile_canonical_bytes(profile: &str) -> Vec<u8> {
    let mut out = Vec::new();
    encode_string(&mut out, PHASE4_KERNEL_CHECK_PROFILE_TAG);
    encode_string(&mut out, STD_CORE_SPEC_ID);
    encode_string(&mut out, STD_KERNEL_SEMANTICS_PROFILE_ID);
    encode_string(&mut out, STD_REDUCTION_PROFILE_ID);
    encode_string(&mut out, STD_UNIVERSE_PROFILE_ID);
    let builtin_profile_id = match profile {
        KERNEL_CHECK_PROFILE_BUILTIN_NAT_EQ_REC => STD_KERNEL_BUILTIN_NAT_EQ_REC_PROFILE_ID,
        STD_KERNEL_CHECK_PROFILE_BUILTIN_NONE => STD_KERNEL_BUILTIN_NONE_PROFILE_ID,
        other => other,
    };
    encode_string(&mut out, builtin_profile_id);
    out
}

fn encode_verified_import_key(
    out: &mut Vec<u8>,
    key: &VerifiedImportKey,
) -> Result<(), MachineStdCanonicalBytesError> {
    encode_name(out, &key.module)?;
    encode_hash(out, &key.export_hash);
    encode_hash(out, &key.certificate_hash);
    Ok(())
}

fn verified_import_key_canonical_bytes(
    key: &VerifiedImportKey,
) -> Result<Vec<u8>, MachineStdCanonicalBytesError> {
    let mut out = Vec::new();
    encode_verified_import_key(&mut out, key)?;
    Ok(out)
}

fn encode_import_certificate_key(
    out: &mut Vec<u8>,
    certificate: &MachineStdImportCertificate,
) -> Result<(), MachineStdCanonicalBytesError> {
    encode_name(out, &certificate.module)?;
    encode_hash(out, &certificate.expected_export_hash);
    encode_hash(out, &certificate.expected_certificate_hash);
    Ok(())
}

fn encode_simp_rule_ref(
    out: &mut Vec<u8>,
    rule: &SimpRuleRef,
) -> Result<(), MachineStdCanonicalBytesError> {
    encode_name(out, &rule.name)?;
    encode_hash(out, &rule.decl_interface_hash);
    encode_rewrite_direction(out, rule.direction);
    Ok(())
}

fn encode_rewrite_direction(out: &mut Vec<u8>, direction: RewriteDirection) {
    out.push(match direction {
        RewriteDirection::Forward => 0x00,
        RewriteDirection::Backward => 0x01,
    });
}

fn encode_option_eq_family(
    out: &mut Vec<u8>,
    value: Option<&EqFamilyRef>,
) -> Result<(), MachineStdCanonicalBytesError> {
    match value {
        Some(value) => {
            out.push(0x01);
            encode_name(out, &value.eq_name)?;
            encode_hash(out, &value.eq_interface_hash);
            encode_name(out, &value.refl_name)?;
            encode_hash(out, &value.refl_interface_hash);
            encode_name(out, &value.rec_name)?;
            encode_hash(out, &value.rec_interface_hash);
        }
        None => out.push(0x00),
    }
    Ok(())
}

fn encode_option_nat_family(
    out: &mut Vec<u8>,
    value: Option<&NatFamilyRef>,
) -> Result<(), MachineStdCanonicalBytesError> {
    match value {
        Some(value) => {
            out.push(0x01);
            encode_name(out, &value.nat_name)?;
            encode_hash(out, &value.nat_interface_hash);
            encode_name(out, &value.zero_name)?;
            encode_hash(out, &value.zero_interface_hash);
            encode_name(out, &value.succ_name)?;
            encode_hash(out, &value.succ_interface_hash);
            encode_name(out, &value.rec_name)?;
            encode_hash(out, &value.rec_interface_hash);
        }
        None => out.push(0x00),
    }
    Ok(())
}

fn encode_option_global_ref_view(
    out: &mut Vec<u8>,
    value: Option<&MachineStdGlobalRefView>,
) -> Result<(), MachineStdCanonicalBytesError> {
    match value {
        Some(value) => {
            out.push(0x01);
            out.extend(machine_std_global_ref_view_canonical_bytes(value)?);
        }
        None => out.push(0x00),
    }
    Ok(())
}

fn encode_option_u64(out: &mut Vec<u8>, value: Option<u64>) {
    match value {
        Some(value) => {
            out.push(0x01);
            encode_uvar(out, value);
        }
        None => out.push(0x00),
    }
}

fn encode_bool(out: &mut Vec<u8>, value: bool) {
    out.push(u8::from(value));
}

fn theorem_kind_byte(kind: MachineStdTheoremKind) -> u8 {
    match kind {
        MachineStdTheoremKind::Theorem => 0x00,
        MachineStdTheoremKind::Axiom => 0x01,
    }
}

fn theorem_mode_byte(mode: MachineTheoremMode) -> u8 {
    match mode {
        MachineTheoremMode::Exact => 0x00,
        MachineTheoremMode::Apply => 0x01,
        MachineTheoremMode::Rw => 0x02,
        MachineTheoremMode::Simp => 0x03,
    }
}

fn theorem_attribute_byte(attribute: MachineStdAttribute) -> u8 {
    match attribute {
        MachineStdAttribute::Simp => 0x00,
        MachineStdAttribute::Rw => 0x01,
        MachineStdAttribute::Intro => 0x02,
        MachineStdAttribute::Elim => 0x03,
        MachineStdAttribute::Apply => 0x04,
        MachineStdAttribute::Refl => 0x05,
        MachineStdAttribute::Trans => 0x06,
        MachineStdAttribute::Congr => 0x07,
    }
}

fn encode_string(out: &mut Vec<u8>, value: &str) {
    encode_uvar(out, value.len() as u64);
    out.extend_from_slice(value.as_bytes());
}

fn encode_name(out: &mut Vec<u8>, name: &Name) -> Result<(), MachineStdCanonicalBytesError> {
    let bytes = phase5_name_canonical_bytes(name).map_err(|source| {
        MachineStdCanonicalBytesError::InvalidName {
            name: name.clone(),
            source: Box::new(source),
        }
    })?;
    out.extend(bytes);
    Ok(())
}

fn encode_hash(out: &mut Vec<u8>, hash: &Hash) {
    out.extend_from_slice(hash);
}

fn encode_uvar(out: &mut Vec<u8>, mut value: u64) {
    while value >= 0x80 {
        out.push((value as u8 & 0x7f) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

fn sha256(bytes: &[u8]) -> Hash {
    let digest = Sha256::digest(bytes);
    let mut out = [0; 32];
    out.copy_from_slice(&digest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::format_hash_string;
    use npa_cert::{build_module_cert, encode_module_cert, CoreModule};
    use npa_kernel::{eq_inductive, eq_rec_type, Decl, Expr, Level};
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn loads_valid_mvp_certificate_package() {
        let package = TestPackage::new("valid_mvp_certificate_package");
        write_valid_mvp_package(package.path());

        let loaded = load_machine_std_mvp_certificates(package.path()).unwrap();
        assert_eq!(loaded.modules().len(), 4);
        assert_eq!(
            loaded
                .modules()
                .iter()
                .map(|module| module.module.as_dotted())
                .collect::<Vec<_>>(),
            vec!["Std.Nat", "Std.List", "Std.Logic", "Std.Algebra.Basic"]
        );
        assert_eq!(
            loaded
                .verification_order()
                .iter()
                .map(Name::as_dotted)
                .collect::<Vec<_>>(),
            vec!["Std.Logic", "Std.Nat", "Std.List", "Std.Algebra.Basic"]
        );
        for module in loaded.modules() {
            assert_eq!(
                module.certificate_bytes_hash,
                sha256(&module.certificate_bytes)
            );
            assert_eq!(
                module.expected_certificate_hash,
                module.verified_module.certificate_hash()
            );
            assert_eq!(
                module.expected_export_hash,
                module.verified_module.export_hash()
            );
        }
    }

    #[test]
    fn rejects_bad_mvp_locator_membership_order_and_path() {
        let mut missing = machine_std_mvp_module_locators();
        missing.pop();
        assert!(matches!(
            validate_machine_std_mvp_locators(&missing),
            Err(MachineStdReleaseLoaderError::InvalidModuleMembership { .. })
        ));

        let mut extra = machine_std_mvp_module_locators();
        extra.push(MachineStdModuleLocator::new(
            Name::from_dotted("Std.Extra"),
            "Std/Extra.npcert",
        ));
        assert!(matches!(
            validate_machine_std_mvp_locators(&extra),
            Err(MachineStdReleaseLoaderError::InvalidModuleMembership { .. })
        ));

        let mut reordered = machine_std_mvp_module_locators();
        reordered.swap(0, 1);
        assert!(matches!(
            validate_machine_std_mvp_locators(&reordered),
            Err(MachineStdReleaseLoaderError::NonCanonicalModuleOrder { .. })
        ));

        let mut wrong_path = machine_std_mvp_module_locators();
        wrong_path[0].relative_path = "Std/NatWrong.npcert".to_owned();
        assert!(matches!(
            validate_machine_std_mvp_locators(&wrong_path),
            Err(MachineStdReleaseLoaderError::FixedPathMismatch { .. })
        ));
    }

    #[test]
    fn rejects_invalid_locator_paths() {
        let cases = [
            ("", MachineStdLocatorPathError::Empty),
            ("/Std/Nat.npcert", MachineStdLocatorPathError::Absolute),
            ("Std\\Nat.npcert", MachineStdLocatorPathError::Backslash),
            ("Std/Nat.npcert/", MachineStdLocatorPathError::TrailingSlash),
            (
                "Std//Nat.npcert",
                MachineStdLocatorPathError::DuplicateSlash,
            ),
            ("Std/./Nat.npcert", MachineStdLocatorPathError::DotComponent),
            (
                "Std/../Nat.npcert",
                MachineStdLocatorPathError::ParentComponent,
            ),
        ];
        for (path, expected) in cases {
            assert_eq!(validate_machine_std_locator_path(path), Err(expected));
        }
    }

    #[test]
    fn rejects_missing_certificate_file() {
        let package = TestPackage::new("missing_certificate_file");
        let err = load_machine_std_mvp_certificates(package.path()).unwrap_err();
        assert!(matches!(
            err,
            MachineStdReleaseLoaderError::MissingCertificateFile { .. }
        ));
    }

    #[test]
    fn rejects_core_or_prelude_import_entry_as_unresolved_release_import() {
        let package = TestPackage::new("core_import_entry");
        let mut certs = mvp_certificate_bytes();
        let mut logic = decode_module_cert(certs.logic.as_slice()).unwrap();
        logic.imports.push(ImportEntry {
            module: Name::from_dotted("Core"),
            export_hash: [0; 32],
            certificate_hash: Some([0; 32]),
        });
        certs.logic = encode_module_cert(&logic).unwrap();
        write_mvp_package(package.path(), &certs);

        let err = load_machine_std_mvp_certificates(package.path()).unwrap_err();
        assert!(matches!(
            err,
            MachineStdReleaseLoaderError::UnresolvedImport {
                imported_module,
                ..
            } if imported_module == Name::from_dotted("Core")
        ));
    }

    #[test]
    fn rejects_release_import_cycles_before_verification() {
        let package = TestPackage::new("import_cycle");
        let mut certs = mvp_certificate_bytes();
        let mut logic = decode_module_cert(certs.logic.as_slice()).unwrap();
        let mut nat = decode_module_cert(certs.nat.as_slice()).unwrap();
        logic.imports.push(ImportEntry {
            module: Name::from_dotted("Std.Nat"),
            export_hash: nat.hashes.export_hash,
            certificate_hash: Some(nat.hashes.certificate_hash),
        });
        nat.imports.push(ImportEntry {
            module: Name::from_dotted("Std.Logic"),
            export_hash: logic.hashes.export_hash,
            certificate_hash: Some(logic.hashes.certificate_hash),
        });
        certs.logic = encode_module_cert(&logic).unwrap();
        certs.nat = encode_module_cert(&nat).unwrap();
        write_mvp_package(package.path(), &certs);

        let err = load_machine_std_mvp_certificates(package.path()).unwrap_err();
        assert!(matches!(
            err,
            MachineStdReleaseLoaderError::ImportCycle { .. }
        ));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_escape_from_package_root() {
        use std::os::unix::fs::symlink;

        let package = TestPackage::new("symlink_escape");
        let outside = TestPackage::new("symlink_escape_outside");
        let certs = mvp_certificate_bytes();
        write_mvp_package(package.path(), &certs);
        fs::write(outside.path().join("Logic.npcert"), &certs.logic).unwrap();
        fs::remove_file(package.path().join(STD_LOGIC_PATH)).unwrap();
        symlink(
            outside.path().join("Logic.npcert"),
            package.path().join(STD_LOGIC_PATH),
        )
        .unwrap();

        let err = load_machine_std_mvp_certificates(package.path()).unwrap_err();
        assert!(matches!(
            err,
            MachineStdReleaseLoaderError::SymlinkEscape { .. }
        ));
    }

    #[test]
    fn loads_valid_mvp_release_manifest_and_axiom_report() {
        let package = TestPackage::new("valid_mvp_release_manifest");
        write_valid_mvp_package(package.path());
        let loaded = load_machine_std_mvp_certificates(package.path()).unwrap();
        let mut axiom_report = empty_axiom_report_for(&loaded);
        axiom_report.axiom_report_hash = machine_std_axiom_report_hash(&axiom_report).unwrap();
        let release = release_manifest_for(&loaded, axiom_report.axiom_report_hash);
        let release_json = release_manifest_json(&release);
        let axiom_report_json = axiom_report_json(&axiom_report);

        let validated = load_machine_std_mvp_release_from_json(
            package.path(),
            &release_json,
            &axiom_report_json,
        )
        .unwrap();
        assert_eq!(validated.loaded.modules().len(), 4);
        assert_eq!(
            validated.axiom_report.axiom_report_hash,
            axiom_report.axiom_report_hash
        );
        assert_eq!(
            validated.std_library_release_hash,
            machine_std_library_release_hash(&validated.manifest).unwrap()
        );
    }

    #[test]
    fn generates_mvp_import_bundle_set_with_canonical_membership() {
        let package = TestPackage::new("mvp_import_bundle_set");
        write_valid_mvp_package(package.path());
        let loaded = load_machine_std_mvp_certificates(package.path()).unwrap();

        let bundle_set = generate_machine_std_mvp_import_bundle_set(&loaded).unwrap();
        assert_eq!(
            bundle_set
                .bundles
                .iter()
                .map(|bundle| bundle.bundle_id.as_str())
                .collect::<Vec<_>>(),
            vec![
                STD_ALGEBRA_BASIC_BUNDLE_ID,
                STD_ALL_BUNDLE_ID,
                STD_LIST_BUNDLE_ID,
                STD_LOGIC_BUNDLE_ID,
                STD_NAT_BUNDLE_ID,
            ]
        );
        let list_bundle = bundle_set
            .bundles
            .iter()
            .find(|bundle| bundle.bundle_id == STD_LIST_BUNDLE_ID)
            .unwrap();
        assert_eq!(
            list_bundle
                .root_imports
                .iter()
                .map(|key| key.module.as_dotted())
                .collect::<Vec<_>>(),
            vec!["Std.List", "Std.Logic"]
        );
        assert_eq!(
            list_bundle
                .import_closure
                .iter()
                .map(|certificate| certificate.module.as_dotted())
                .collect::<Vec<_>>(),
            vec!["Std.Nat", "Std.List", "Std.Logic"]
        );
        assert!(bundle_set
            .bundles
            .iter()
            .all(|bundle| bundle.allow_axioms.is_empty()));
        assert!(bundle_set.bundles.iter().all(|bundle| {
            crate::types::KernelCheckProfileId::parse(
                &bundle.recommended_tactic_options.kernel_check_profile,
            )
            .is_ok()
        }));
        assert_eq!(
            bundle_set.import_bundles_hash,
            machine_std_import_bundle_set_hash(&bundle_set).unwrap()
        );
    }

    #[test]
    fn generates_mvp_theorem_index_from_public_theorem_and_axiom_exports() {
        let package = TestPackage::new("mvp_theorem_index_base");
        let certs = mvp_certificate_bytes_with_logic_axiom_theorem();
        write_mvp_package(package.path(), &certs);
        let loaded =
            load_machine_std_mvp_certificates_for_manifest_validation(package.path()).unwrap();
        let logic = loaded.module(&Name::from_dotted("Std.Logic")).unwrap();

        let theorem_index = generate_machine_std_mvp_theorem_index(&loaded).unwrap();
        validate_machine_std_mvp_theorem_index(&theorem_index, &theorem_index).unwrap();
        assert_eq!(theorem_index.entries.len(), 2);
        assert_eq!(theorem_index.index_hash, [0; 32]);
        assert_ne!(
            machine_std_theorem_index_hash(&theorem_index).unwrap(),
            [0; 32]
        );

        let p_export = export_entry(logic, "P");
        let p_id_export = export_entry(logic, "p_id");
        let p_entry = theorem_index_entry(&theorem_index, "P");
        let p_id_entry = theorem_index_entry(&theorem_index, "p_id");

        assert_eq!(p_entry.kind, MachineStdTheoremKind::Axiom);
        assert_eq!(p_entry.statement_core_hash, p_export.type_hash);
        assert_eq!(p_id_entry.kind, MachineStdTheoremKind::Theorem);
        assert_eq!(p_id_entry.statement_core_hash, p_id_export.type_hash);
        assert_eq!(
            p_id_entry.modes,
            vec![MachineTheoremMode::Exact, MachineTheoremMode::Apply]
        );
        assert!(p_id_entry.attributes.is_empty());
        assert!(p_id_entry.rewrite_descriptors.is_empty());
        assert_eq!(p_id_entry.proof_term_size, None);
        assert_eq!(
            p_id_entry.statement_head.as_ref(),
            p_id_entry.constants.first()
        );
        assert!(matches!(
            p_id_entry.statement_head.as_ref(),
            Some(MachineStdGlobalRefView::Decl {
                module,
                name,
                public_export: true,
                ..
            }) if *module == Name::from_dotted("Std.Logic") && *name == Name::from_dotted("P")
        ));
        assert_eq!(
            p_id_entry
                .axiom_dependencies
                .iter()
                .map(|axiom| axiom.name.as_dotted())
                .collect::<Vec<_>>(),
            vec!["P"]
        );
    }

    #[test]
    fn theorem_index_base_validation_defers_profile_metadata_and_final_hash() {
        let package = TestPackage::new("theorem_index_base_defers_metadata");
        let certs = mvp_certificate_bytes_with_logic_axiom_theorem();
        write_mvp_package(package.path(), &certs);
        let loaded =
            load_machine_std_mvp_certificates_for_manifest_validation(package.path()).unwrap();
        let expected = generate_machine_std_mvp_theorem_index(&loaded).unwrap();
        let p_id_index = expected
            .entries
            .iter()
            .position(|entry| entry.global_ref.name == Name::from_dotted("p_id"))
            .unwrap();

        let mut actual = expected.clone();
        actual.index_hash = test_hash(230);
        actual.entries[p_id_index].modes = vec![
            MachineTheoremMode::Exact,
            MachineTheoremMode::Apply,
            MachineTheoremMode::Rw,
            MachineTheoremMode::Simp,
        ];
        actual.entries[p_id_index].attributes = vec![MachineStdAttribute::Simp];

        validate_machine_std_mvp_theorem_index(&actual, &expected).unwrap();
    }

    #[test]
    fn rejects_missing_extra_generated_and_private_theorem_index_entries() {
        let package = TestPackage::new("bad_theorem_index_membership");
        let certs = mvp_certificate_bytes_with_logic_eq_rec_axiom();
        write_mvp_package(package.path(), &certs);
        let loaded =
            load_machine_std_mvp_certificates_for_manifest_validation(package.path()).unwrap();
        let expected = generate_machine_std_mvp_theorem_index(&loaded).unwrap();
        let logic = loaded.module(&Name::from_dotted("Std.Logic")).unwrap();

        let mut missing = expected.clone();
        missing.entries.clear();
        refresh_theorem_index_hash(&mut missing);
        assert!(matches!(
            validate_machine_std_mvp_theorem_index(&missing, &expected),
            Err(MachineStdTheoremIndexError::InvalidEntryMembership { .. })
        ));

        let generated = export_entry(logic, "Eq.refl");
        let mut extra_generated = expected.clone();
        let mut generated_entry = expected.entries[0].clone();
        generated_entry.global_ref.name = Name::from_dotted("Eq.refl");
        generated_entry.global_ref.decl_interface_hash = generated.decl_interface_hash;
        extra_generated.entries.push(generated_entry);
        refresh_theorem_index_hash(&mut extra_generated);
        assert!(matches!(
            validate_machine_std_mvp_theorem_index(&extra_generated, &expected),
            Err(MachineStdTheoremIndexError::InvalidEntryMembership { .. })
        ));

        let mut private_like = expected.clone();
        let mut private_entry = expected.entries[0].clone();
        private_entry.global_ref.name = Name::from_dotted("private_helper");
        private_entry.global_ref.decl_interface_hash = test_hash(222);
        private_like.entries.push(private_entry);
        refresh_theorem_index_hash(&mut private_like);
        assert!(matches!(
            validate_machine_std_mvp_theorem_index(&private_like, &expected),
            Err(MachineStdTheoremIndexError::InvalidEntryMembership { .. })
        ));
    }

    #[test]
    fn rejects_theorem_index_certificate_derived_field_mismatches() {
        let package = TestPackage::new("bad_theorem_index_fields");
        let certs = mvp_certificate_bytes_with_logic_axiom_theorem();
        write_mvp_package(package.path(), &certs);
        let loaded =
            load_machine_std_mvp_certificates_for_manifest_validation(package.path()).unwrap();
        let expected = generate_machine_std_mvp_theorem_index(&loaded).unwrap();
        let p_id_index = expected
            .entries
            .iter()
            .position(|entry| entry.global_ref.name == Name::from_dotted("p_id"))
            .unwrap();

        let mut bad_hash = expected.clone();
        bad_hash.entries[p_id_index].statement_core_hash = test_hash(201);
        refresh_theorem_index_hash(&mut bad_hash);
        assert!(matches!(
            validate_machine_std_mvp_theorem_index(&bad_hash, &expected),
            Err(MachineStdTheoremIndexError::StatementCoreHashMismatch { .. })
        ));

        let mut bad_constants = expected.clone();
        bad_constants.entries[p_id_index].constants.clear();
        refresh_theorem_index_hash(&mut bad_constants);
        assert!(matches!(
            validate_machine_std_mvp_theorem_index(&bad_constants, &expected),
            Err(MachineStdTheoremIndexError::ConstantsMismatch { .. })
        ));

        let mut bad_axioms = expected.clone();
        bad_axioms.entries[p_id_index].axiom_dependencies.clear();
        refresh_theorem_index_hash(&mut bad_axioms);
        assert!(matches!(
            validate_machine_std_mvp_theorem_index(&bad_axioms, &expected),
            Err(MachineStdTheoremIndexError::AxiomDependenciesMismatch { .. })
        ));

        let mut bad_size = expected.clone();
        bad_size.entries[p_id_index].proof_term_size = Some(1);
        refresh_theorem_index_hash(&mut bad_size);
        assert!(matches!(
            validate_machine_std_mvp_theorem_index(&bad_size, &expected),
            Err(MachineStdTheoremIndexError::NonNullProofTermSize { .. })
        ));
    }

    #[test]
    fn loads_valid_mvp_release_with_import_bundle_sidecar() {
        let package = TestPackage::new("valid_mvp_import_bundle_sidecar");
        write_valid_mvp_package(package.path());
        let loaded = load_machine_std_mvp_certificates(package.path()).unwrap();
        let import_bundles = generate_machine_std_mvp_import_bundle_set(&loaded).unwrap();
        let mut axiom_report = empty_axiom_report_for(&loaded);
        axiom_report.axiom_report_hash = machine_std_axiom_report_hash(&axiom_report).unwrap();
        let release = release_manifest_for(&loaded, axiom_report.axiom_report_hash);

        let validated = load_machine_std_mvp_release_with_import_bundles_from_json(
            package.path(),
            &release_manifest_json(&release),
            &import_bundle_set_json(&import_bundles),
            &axiom_report_json(&axiom_report),
        )
        .unwrap();

        assert_eq!(
            validated.import_bundles.import_bundles_hash,
            import_bundles.import_bundles_hash
        );
        assert_eq!(
            validated.manifest.import_bundles_hash,
            import_bundles.import_bundles_hash
        );
    }

    #[test]
    fn rejects_missing_extra_duplicate_or_reordered_import_bundle_ids() {
        let package = TestPackage::new("bad_import_bundle_membership");
        write_valid_mvp_package(package.path());
        let loaded = load_machine_std_mvp_certificates(package.path()).unwrap();
        let expected = generate_machine_std_mvp_import_bundle_set(&loaded).unwrap();

        let mut missing = expected.clone();
        missing.bundles.pop();
        missing.import_bundles_hash = machine_std_import_bundle_set_hash(&missing).unwrap();
        assert!(matches!(
            validate_machine_std_mvp_import_bundle_set(&missing, &expected),
            Err(MachineStdImportBundleError::InvalidBundleMembership { .. })
        ));

        let mut extra = expected.clone();
        let mut extra_bundle = extra.bundles[0].clone();
        extra_bundle.bundle_id = "std.extra.mvp".to_owned();
        extra.bundles.push(extra_bundle);
        extra.import_bundles_hash = machine_std_import_bundle_set_hash(&extra).unwrap();
        assert!(matches!(
            validate_machine_std_mvp_import_bundle_set(&extra, &expected),
            Err(MachineStdImportBundleError::InvalidBundleMembership { .. })
        ));

        let mut duplicate = expected.clone();
        duplicate.bundles.push(duplicate.bundles[0].clone());
        duplicate.import_bundles_hash = machine_std_import_bundle_set_hash(&duplicate).unwrap();
        assert!(matches!(
            validate_machine_std_mvp_import_bundle_set(&duplicate, &expected),
            Err(MachineStdImportBundleError::DuplicateBundle { .. })
        ));

        let mut reordered = expected.clone();
        reordered.bundles.swap(0, 1);
        reordered.import_bundles_hash = machine_std_import_bundle_set_hash(&reordered).unwrap();
        assert!(matches!(
            validate_machine_std_mvp_import_bundle_set(&reordered, &expected),
            Err(MachineStdImportBundleError::NonCanonicalBundleOrder { .. })
        ));
    }

    #[test]
    fn rejects_noncanonical_import_bundle_roots_and_closure_order() {
        let package = TestPackage::new("bad_import_bundle_order");
        write_valid_mvp_package(package.path());
        let loaded = load_machine_std_mvp_certificates(package.path()).unwrap();
        let expected = generate_machine_std_mvp_import_bundle_set(&loaded).unwrap();

        let mut bad_roots = expected.clone();
        let list = bad_roots
            .bundles
            .iter_mut()
            .find(|bundle| bundle.bundle_id == STD_LIST_BUNDLE_ID)
            .unwrap();
        list.root_imports.swap(0, 1);
        bad_roots.import_bundles_hash = machine_std_import_bundle_set_hash(&bad_roots).unwrap();
        assert!(matches!(
            validate_machine_std_mvp_import_bundle_set(&bad_roots, &expected),
            Err(MachineStdImportBundleError::NonCanonicalRootImportOrder { .. })
        ));

        let mut bad_closure = expected.clone();
        let list = bad_closure
            .bundles
            .iter_mut()
            .find(|bundle| bundle.bundle_id == STD_LIST_BUNDLE_ID)
            .unwrap();
        list.import_closure.swap(0, 1);
        bad_closure.import_bundles_hash = machine_std_import_bundle_set_hash(&bad_closure).unwrap();
        assert!(matches!(
            validate_machine_std_mvp_import_bundle_set(&bad_closure, &expected),
            Err(MachineStdImportBundleError::NonCanonicalImportClosureOrder { .. })
        ));
    }

    #[test]
    fn rejects_import_bundle_certificate_bytes_mismatch() {
        let package = TestPackage::new("bad_import_bundle_certificate_bytes");
        write_valid_mvp_package(package.path());
        let loaded = load_machine_std_mvp_certificates(package.path()).unwrap();
        let expected = generate_machine_std_mvp_import_bundle_set(&loaded).unwrap();

        let mut actual = expected.clone();
        actual.bundles[0].import_closure[0]
            .certificate_bytes
            .push(0xff);
        actual.import_bundles_hash = machine_std_import_bundle_set_hash(&actual).unwrap();

        assert!(matches!(
            validate_machine_std_mvp_import_bundle_set(&actual, &expected),
            Err(MachineStdImportBundleError::CertificateBytesHashMismatch { .. })
        ));
    }

    #[test]
    fn rejects_non_empty_mvp_import_bundle_allow_axioms() {
        let package = TestPackage::new("bad_import_bundle_allow_axioms");
        write_valid_mvp_package(package.path());
        let loaded = load_machine_std_mvp_certificates(package.path()).unwrap();
        let expected = generate_machine_std_mvp_import_bundle_set(&loaded).unwrap();

        let mut actual = expected.clone();
        let key = actual.bundles[0].root_imports[0].clone();
        actual.bundles[0]
            .allow_axioms
            .push(MachineAxiomRefWire::Imported {
                module: key.module,
                name: Name::from_dotted("Std.Logic.synthetic_axiom"),
                export_hash: key.export_hash,
                decl_interface_hash: test_hash(99),
            });
        actual.import_bundles_hash = machine_std_import_bundle_set_hash(&actual).unwrap();

        assert!(matches!(
            validate_machine_std_mvp_import_bundle_set(&actual, &expected),
            Err(MachineStdImportBundleError::NonEmptyMvpAllowAxioms { .. })
        ));
    }

    #[test]
    fn parses_imported_allow_axioms_before_rejecting_non_empty_mvp_bundle() {
        let package = TestPackage::new("import_bundle_allow_axioms_json");
        write_valid_mvp_package(package.path());
        let loaded = load_machine_std_mvp_certificates(package.path()).unwrap();
        let expected = generate_machine_std_mvp_import_bundle_set(&loaded).unwrap();

        let mut actual = expected.clone();
        let key = actual.bundles[0].root_imports[0].clone();
        actual.bundles[0]
            .allow_axioms
            .push(MachineAxiomRefWire::Imported {
                module: key.module,
                name: Name::from_dotted("Std.Logic.synthetic_axiom"),
                export_hash: key.export_hash,
                decl_interface_hash: test_hash(100),
            });
        actual.import_bundles_hash = machine_std_import_bundle_set_hash(&actual).unwrap();

        let parsed = parse_machine_std_import_bundle_set_json(&import_bundle_set_json(&actual))
            .expect("imported allow_axioms variant should parse");

        assert!(matches!(
            validate_machine_std_mvp_import_bundle_set(&parsed, &expected),
            Err(MachineStdImportBundleError::NonEmptyMvpAllowAxioms { .. })
        ));
    }

    #[test]
    fn emits_eq_family_when_std_logic_exports_eq_rec_as_axiom() {
        let package = TestPackage::new("import_bundle_eq_rec_axiom_family");
        let certs = mvp_certificate_bytes_with_logic_eq_rec_axiom();
        write_mvp_package(package.path(), &certs);
        let loaded =
            load_machine_std_mvp_certificates_for_manifest_validation(package.path()).unwrap();
        let logic = loaded.module(&Name::from_dotted("Std.Logic")).unwrap();
        assert!(logic.verified_module.export_block().iter().any(|entry| {
            entry.kind == ExportKind::Axiom
                && logic
                    .verified_module
                    .name_table()
                    .get(entry.name)
                    .is_some_and(|name| *name == Name::from_dotted("Eq.rec"))
        }));

        let bundle_set = generate_machine_std_mvp_import_bundle_set(&loaded).unwrap();
        let logic_bundle = bundle_set
            .bundles
            .iter()
            .find(|bundle| bundle.bundle_id == STD_LOGIC_BUNDLE_ID)
            .unwrap();
        let family = logic_bundle
            .recommended_tactic_options
            .eq_family
            .as_ref()
            .expect("Eq.rec axiom export should still produce an Eq family recipe");

        assert_eq!(family.eq_name, Name::from_dotted("Eq"));
        assert_eq!(family.refl_name, Name::from_dotted("Eq.refl"));
        assert_eq!(family.rec_name, Name::from_dotted("Eq.rec"));
    }

    #[test]
    fn rejects_invalid_import_bundle_recipe_mapping() {
        let package = TestPackage::new("bad_import_bundle_recipe");
        write_valid_mvp_package(package.path());
        let loaded = load_machine_std_mvp_certificates(package.path()).unwrap();
        let expected = generate_machine_std_mvp_import_bundle_set(&loaded).unwrap();

        let mut actual = expected.clone();
        actual.bundles[0].recommended_tactic_options.recipe_id = "std.bad-recipe".to_owned();
        actual.import_bundles_hash = machine_std_import_bundle_set_hash(&actual).unwrap();

        assert!(matches!(
            validate_machine_std_mvp_import_bundle_set(&actual, &expected),
            Err(MachineStdImportBundleError::InvalidRecipeIdMapping { .. })
        ));
    }

    #[test]
    fn rejects_manifest_bound_import_bundle_hash_mismatch_as_library_release() {
        let package = TestPackage::new("manifest_bound_import_bundle_hash_mismatch");
        write_valid_mvp_package(package.path());
        let loaded = load_machine_std_mvp_certificates(package.path()).unwrap();
        let import_bundles = generate_machine_std_mvp_import_bundle_set(&loaded).unwrap();
        let mut axiom_report = empty_axiom_report_for(&loaded);
        axiom_report.axiom_report_hash = machine_std_axiom_report_hash(&axiom_report).unwrap();
        let mut release = release_manifest_for(&loaded, axiom_report.axiom_report_hash);
        release.import_bundles_hash = test_hash(55);

        let err = load_machine_std_mvp_release_with_import_bundles_from_json(
            package.path(),
            &release_manifest_json(&release),
            &import_bundle_set_json(&import_bundles),
            &axiom_report_json(&axiom_report),
        )
        .unwrap_err();

        assert!(matches!(
            err,
            MachineStdReleaseArtifactError::InvalidStdLibraryRelease(
                MachineStdLibraryReleaseError::SidecarHashMismatch {
                    field: "import_bundles_hash",
                    ..
                }
            )
        ));
    }

    #[test]
    fn rejects_stale_import_bundle_self_hash_before_manifest_comparison() {
        let package = TestPackage::new("stale_import_bundle_self_hash");
        write_valid_mvp_package(package.path());
        let loaded = load_machine_std_mvp_certificates(package.path()).unwrap();
        let mut import_bundles = generate_machine_std_mvp_import_bundle_set(&loaded).unwrap();
        import_bundles.import_bundles_hash = test_hash(56);
        let mut axiom_report = empty_axiom_report_for(&loaded);
        axiom_report.axiom_report_hash = machine_std_axiom_report_hash(&axiom_report).unwrap();
        let release = release_manifest_for(&loaded, axiom_report.axiom_report_hash);

        let err = load_machine_std_mvp_release_with_import_bundles_from_json(
            package.path(),
            &release_manifest_json(&release),
            &import_bundle_set_json(&import_bundles),
            &axiom_report_json(&axiom_report),
        )
        .unwrap_err();

        assert!(matches!(
            err,
            MachineStdReleaseArtifactError::InvalidStdImportBundle(
                MachineStdImportBundleError::ImportBundlesHashMismatch { .. }
            )
        ));
    }

    #[test]
    fn rejects_mvp_release_scalar_mismatch_as_library_release() {
        let package = TestPackage::new("release_scalar_mismatch");
        write_valid_mvp_package(package.path());
        let loaded = load_machine_std_mvp_certificates(package.path()).unwrap();
        let mut axiom_report = empty_axiom_report_for(&loaded);
        axiom_report.axiom_report_hash = machine_std_axiom_report_hash(&axiom_report).unwrap();
        let mut release = release_manifest_for(&loaded, axiom_report.axiom_report_hash);
        release.protocol_version = "npa.stdlib-machine.bad".to_owned();

        let err = load_machine_std_mvp_release_from_json(
            package.path(),
            &release_manifest_json(&release),
            &axiom_report_json(&axiom_report),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            MachineStdReleaseArtifactError::InvalidStdLibraryRelease(
                MachineStdLibraryReleaseError::ScalarMismatch {
                    field: "protocol_version",
                    ..
                }
            )
        ));
    }

    #[test]
    fn rejects_std_library_release_hash_field_as_unknown_shape() {
        let package = TestPackage::new("release_hash_unknown");
        write_valid_mvp_package(package.path());
        let loaded = load_machine_std_mvp_certificates(package.path()).unwrap();
        let axiom_report = empty_axiom_report_for(&loaded);
        let release = release_manifest_for(&loaded, test_hash(9));
        let release_json = release_manifest_json(&release).replacen(
            "{\"protocol_version\"",
            &format!(
                "{{\"std_library_release_hash\":\"{}\",\"protocol_version\"",
                format_hash_string(&test_hash(77))
            ),
            1,
        );

        let err = load_machine_std_mvp_release_from_json(
            package.path(),
            &release_json,
            &axiom_report_json(&axiom_report),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            MachineStdReleaseArtifactError::InvalidStdArtifactShape(
                MachineStdArtifactShapeError {
                    reason: MachineStdArtifactShapeErrorReason::UnknownField { field },
                    ..
                }
            ) if field == "std_library_release_hash"
        ));
    }

    #[test]
    fn rejects_non_empty_mvp_axiom_report_lists() {
        let package = TestPackage::new("non_empty_axiom_report");
        write_valid_mvp_package(package.path());
        let loaded = load_machine_std_mvp_certificates(package.path()).unwrap();
        let mut axiom_report = empty_axiom_report_for(&loaded);
        let first = &mut axiom_report.modules[0];
        first.module_axioms.push(MachineStdAxiomRef {
            module: first.module.clone(),
            name: Name::from_dotted("Std.Nat.synthetic_axiom"),
            export_hash: first.export_hash,
            decl_interface_hash: test_hash(88),
        });
        axiom_report.axiom_report_hash = machine_std_axiom_report_hash(&axiom_report).unwrap();
        let release = release_manifest_for(&loaded, axiom_report.axiom_report_hash);

        let err = load_machine_std_mvp_release_from_json(
            package.path(),
            &release_manifest_json(&release),
            &axiom_report_json(&axiom_report),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            MachineStdReleaseArtifactError::InvalidStdAxiomPolicy(
                MachineStdAxiomPolicyError::NonEmptyMvpAxiomList {
                    field: "module_axioms",
                    ..
                }
            )
        ));
    }

    #[test]
    fn rejects_certificate_axioms_as_axiom_policy() {
        let package = TestPackage::new("certificate_axiom_policy");
        let certs = mvp_certificate_bytes_with_logic_axiom("Std.Logic.synthetic_axiom");
        write_mvp_package(package.path(), &certs);
        let loaded =
            load_machine_std_mvp_certificates_for_manifest_validation(package.path()).unwrap();
        let mut axiom_report = empty_axiom_report_for(&loaded);
        axiom_report.axiom_report_hash = machine_std_axiom_report_hash(&axiom_report).unwrap();
        let release = release_manifest_for(&loaded, axiom_report.axiom_report_hash);

        let err = load_machine_std_mvp_release_from_json(
            package.path(),
            &release_manifest_json(&release),
            &axiom_report_json(&axiom_report),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            MachineStdReleaseArtifactError::InvalidStdAxiomPolicy(
                MachineStdAxiomPolicyError::ModuleAxiomsMismatch { .. }
            )
        ));
    }

    #[test]
    fn rejects_stale_axiom_report_self_hash_before_manifest_comparison() {
        let package = TestPackage::new("stale_axiom_report_self_hash");
        write_valid_mvp_package(package.path());
        let loaded = load_machine_std_mvp_certificates(package.path()).unwrap();
        let mut axiom_report = empty_axiom_report_for(&loaded);
        axiom_report.axiom_report_hash = test_hash(44);
        let release = release_manifest_for(&loaded, axiom_report.axiom_report_hash);

        let err = load_machine_std_mvp_release_from_json(
            package.path(),
            &release_manifest_json(&release),
            &axiom_report_json(&axiom_report),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            MachineStdReleaseArtifactError::InvalidStdAxiomPolicy(
                MachineStdAxiomPolicyError::AxiomReportHashMismatch { .. }
            )
        ));
    }

    #[test]
    fn rejects_manifest_bound_axiom_report_hash_mismatch_as_library_release() {
        let package = TestPackage::new("manifest_bound_axiom_hash_mismatch");
        write_valid_mvp_package(package.path());
        let loaded = load_machine_std_mvp_certificates(package.path()).unwrap();
        let mut axiom_report = empty_axiom_report_for(&loaded);
        axiom_report.axiom_report_hash = machine_std_axiom_report_hash(&axiom_report).unwrap();
        let release = release_manifest_for(&loaded, test_hash(45));

        let err = load_machine_std_mvp_release_from_json(
            package.path(),
            &release_manifest_json(&release),
            &axiom_report_json(&axiom_report),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            MachineStdReleaseArtifactError::InvalidStdLibraryRelease(
                MachineStdLibraryReleaseError::SidecarHashMismatch {
                    field: "axiom_report_hash",
                    ..
                }
            )
        ));
    }

    struct MvpCertificateBytes {
        logic: Vec<u8>,
        nat: Vec<u8>,
        list: Vec<u8>,
        algebra_basic: Vec<u8>,
    }

    fn mvp_certificate_bytes() -> MvpCertificateBytes {
        let mut session = VerifierSession::new();
        let policy = AxiomPolicy::high_trust();

        let logic_cert = build_module_cert(empty_module("Std.Logic"), &[]).unwrap();
        let logic = encode_module_cert(&logic_cert).unwrap();
        let logic_verified = verify_module_cert(&logic, &mut session, &policy).unwrap();

        let nat_cert = build_module_cert(
            empty_module("Std.Nat"),
            std::slice::from_ref(&logic_verified),
        )
        .unwrap();
        let nat = encode_module_cert(&nat_cert).unwrap();
        let nat_verified = verify_module_cert(&nat, &mut session, &policy).unwrap();

        let list_cert = build_module_cert(
            empty_module("Std.List"),
            &[logic_verified.clone(), nat_verified.clone()],
        )
        .unwrap();
        let list = encode_module_cert(&list_cert).unwrap();
        verify_module_cert(&list, &mut session, &policy).unwrap();

        let algebra_cert =
            build_module_cert(empty_module("Std.Algebra.Basic"), &[logic_verified]).unwrap();
        let algebra_basic = encode_module_cert(&algebra_cert).unwrap();

        MvpCertificateBytes {
            logic,
            nat,
            list,
            algebra_basic,
        }
    }

    fn mvp_certificate_bytes_with_logic_axiom(axiom_name: &str) -> MvpCertificateBytes {
        let mut session = VerifierSession::new();
        let mut policy = AxiomPolicy::high_trust();
        policy
            .allowlisted_axioms
            .insert(Name::from_dotted(axiom_name));

        let logic_cert = build_module_cert(logic_axiom_module(axiom_name), &[]).unwrap();
        let logic = encode_module_cert(&logic_cert).unwrap();
        let logic_verified = verify_module_cert(&logic, &mut session, &policy).unwrap();

        let nat_cert = build_module_cert(
            empty_module("Std.Nat"),
            std::slice::from_ref(&logic_verified),
        )
        .unwrap();
        let nat = encode_module_cert(&nat_cert).unwrap();
        let nat_verified = verify_module_cert(&nat, &mut session, &policy).unwrap();

        let list_cert = build_module_cert(
            empty_module("Std.List"),
            &[logic_verified.clone(), nat_verified.clone()],
        )
        .unwrap();
        let list = encode_module_cert(&list_cert).unwrap();
        verify_module_cert(&list, &mut session, &policy).unwrap();

        let algebra_cert =
            build_module_cert(empty_module("Std.Algebra.Basic"), &[logic_verified]).unwrap();
        let algebra_basic = encode_module_cert(&algebra_cert).unwrap();

        MvpCertificateBytes {
            logic,
            nat,
            list,
            algebra_basic,
        }
    }

    fn mvp_certificate_bytes_with_logic_axiom_theorem() -> MvpCertificateBytes {
        let mut session = VerifierSession::new();
        let mut policy = AxiomPolicy::high_trust();
        policy.allowlisted_axioms.insert(Name::from_dotted("P"));

        let logic_cert = build_module_cert(logic_axiom_theorem_module(), &[]).unwrap();
        let logic = encode_module_cert(&logic_cert).unwrap();
        let logic_verified = verify_module_cert(&logic, &mut session, &policy).unwrap();

        let nat_cert = build_module_cert(
            empty_module("Std.Nat"),
            std::slice::from_ref(&logic_verified),
        )
        .unwrap();
        let nat = encode_module_cert(&nat_cert).unwrap();
        let nat_verified = verify_module_cert(&nat, &mut session, &policy).unwrap();

        let list_cert = build_module_cert(
            empty_module("Std.List"),
            &[logic_verified.clone(), nat_verified.clone()],
        )
        .unwrap();
        let list = encode_module_cert(&list_cert).unwrap();
        verify_module_cert(&list, &mut session, &policy).unwrap();

        let algebra_cert =
            build_module_cert(empty_module("Std.Algebra.Basic"), &[logic_verified]).unwrap();
        let algebra_basic = encode_module_cert(&algebra_cert).unwrap();

        MvpCertificateBytes {
            logic,
            nat,
            list,
            algebra_basic,
        }
    }

    fn mvp_certificate_bytes_with_logic_eq_rec_axiom() -> MvpCertificateBytes {
        let mut session = VerifierSession::new();
        let mut policy = AxiomPolicy::high_trust();
        policy
            .allowlisted_axioms
            .insert(Name::from_dotted("Eq.rec"));

        let logic_cert = build_module_cert(logic_eq_rec_axiom_module(), &[]).unwrap();
        let logic = encode_module_cert(&logic_cert).unwrap();
        let logic_verified = verify_module_cert(&logic, &mut session, &policy).unwrap();

        let nat_cert = build_module_cert(
            empty_module("Std.Nat"),
            std::slice::from_ref(&logic_verified),
        )
        .unwrap();
        let nat = encode_module_cert(&nat_cert).unwrap();
        let nat_verified = verify_module_cert(&nat, &mut session, &policy).unwrap();

        let list_cert = build_module_cert(
            empty_module("Std.List"),
            &[logic_verified.clone(), nat_verified.clone()],
        )
        .unwrap();
        let list = encode_module_cert(&list_cert).unwrap();
        verify_module_cert(&list, &mut session, &policy).unwrap();

        let algebra_cert =
            build_module_cert(empty_module("Std.Algebra.Basic"), &[logic_verified]).unwrap();
        let algebra_basic = encode_module_cert(&algebra_cert).unwrap();

        MvpCertificateBytes {
            logic,
            nat,
            list,
            algebra_basic,
        }
    }

    fn write_valid_mvp_package(root: &Path) {
        let certs = mvp_certificate_bytes();
        write_mvp_package(root, &certs);
    }

    fn write_mvp_package(root: &Path, certs: &MvpCertificateBytes) {
        write_cert(root, STD_LOGIC_PATH, &certs.logic);
        write_cert(root, STD_NAT_PATH, &certs.nat);
        write_cert(root, STD_LIST_PATH, &certs.list);
        write_cert(root, STD_ALGEBRA_BASIC_PATH, &certs.algebra_basic);
    }

    fn write_cert(root: &Path, relative_path: &str, bytes: &[u8]) {
        let path = join_posix_relative_path(root, relative_path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn empty_module(name: &str) -> CoreModule {
        CoreModule {
            name: Name::from_dotted(name),
            declarations: Vec::new(),
        }
    }

    fn logic_axiom_module(axiom_name: &str) -> CoreModule {
        CoreModule {
            name: Name::from_dotted("Std.Logic"),
            declarations: vec![Decl::Axiom {
                name: axiom_name.to_owned(),
                universe_params: Vec::new(),
                ty: Expr::sort(Level::zero()),
            }],
        }
    }

    fn logic_axiom_theorem_module() -> CoreModule {
        let p = Expr::konst("P", vec![]);
        CoreModule {
            name: Name::from_dotted("Std.Logic"),
            declarations: vec![
                Decl::Axiom {
                    name: "P".to_owned(),
                    universe_params: Vec::new(),
                    ty: Expr::sort(Level::zero()),
                },
                Decl::Theorem {
                    name: "p_id".to_owned(),
                    universe_params: Vec::new(),
                    ty: Expr::pi("h", p.clone(), p.clone()),
                    proof: Expr::lam("h", p, Expr::bvar(0)),
                },
            ],
        }
    }

    fn logic_eq_rec_axiom_module() -> CoreModule {
        CoreModule {
            name: Name::from_dotted("Std.Logic"),
            declarations: vec![
                Decl::Inductive {
                    name: "Eq".to_owned(),
                    universe_params: vec!["u".to_owned()],
                    ty: Expr::pi(
                        "A",
                        Expr::sort(Level::param("u")),
                        Expr::pi(
                            "lhs",
                            Expr::bvar(0),
                            Expr::pi("rhs", Expr::bvar(1), Expr::sort(Level::zero())),
                        ),
                    ),
                    data: Box::new(eq_inductive()),
                },
                Decl::Axiom {
                    name: "Eq.rec".to_owned(),
                    universe_params: vec!["u".to_owned(), "v".to_owned()],
                    ty: eq_rec_type(Level::param("u"), Level::param("v")),
                },
            ],
        }
    }

    fn empty_axiom_report_for(loaded: &MachineStdLoadedRelease) -> MachineStdAxiomReport {
        MachineStdAxiomReport {
            library_profile_id: STD_LIBRARY_PROFILE_ID.to_owned(),
            modules: loaded
                .modules()
                .iter()
                .map(|module| MachineStdModuleAxiomReport {
                    module: module.module.clone(),
                    export_hash: module.expected_export_hash,
                    certificate_hash: module.expected_certificate_hash,
                    module_axioms: Vec::new(),
                    transitive_axioms: Vec::new(),
                })
                .collect(),
            axiom_report_hash: [0; 32],
        }
    }

    fn theorem_index_entry<'a>(
        theorem_index: &'a MachineStdTheoremIndex,
        name: &str,
    ) -> &'a MachineStdTheoremEntry {
        theorem_index
            .entries
            .iter()
            .find(|entry| entry.global_ref.name == Name::from_dotted(name))
            .unwrap()
    }

    fn export_entry<'a>(module: &'a MachineStdLoadedModule, name: &str) -> &'a ExportEntry {
        module
            .verified_module
            .export_block()
            .iter()
            .find(|entry| {
                module
                    .verified_module
                    .name_table()
                    .get(entry.name)
                    .is_some_and(|entry_name| *entry_name == Name::from_dotted(name))
            })
            .unwrap()
    }

    fn refresh_theorem_index_hash(theorem_index: &mut MachineStdTheoremIndex) {
        theorem_index.index_hash = machine_std_theorem_index_hash(theorem_index).unwrap();
    }

    fn release_manifest_for(
        loaded: &MachineStdLoadedRelease,
        axiom_report_hash: Hash,
    ) -> MachineStdLibraryRelease {
        let import_bundles_hash = generate_machine_std_mvp_import_bundle_set(loaded)
            .unwrap()
            .import_bundles_hash;
        MachineStdLibraryRelease {
            protocol_version: STD_LIBRARY_PROTOCOL_VERSION.to_owned(),
            library_profile_id: STD_LIBRARY_PROFILE_ID.to_owned(),
            core_spec_id: STD_CORE_SPEC_ID.to_owned(),
            kernel_semantics_profile_id: STD_KERNEL_SEMANTICS_PROFILE_ID.to_owned(),
            modules: loaded
                .modules()
                .iter()
                .map(|module| MachineStdModuleArtifact {
                    module: module.module.clone(),
                    expected_export_hash: module.expected_export_hash,
                    expected_certificate_hash: module.expected_certificate_hash,
                    certificate_encoding: STD_CERTIFICATE_ENCODING.to_owned(),
                    certificate_bytes_hash: module.certificate_bytes_hash,
                    axiom_report_hash: module.axiom_report_hash,
                    public_export_count: module.verified_module.export_block().len() as u64,
                    theorem_index_entry_count: module
                        .verified_module
                        .export_block()
                        .iter()
                        .filter(|entry| {
                            matches!(entry.kind, ExportKind::Theorem | ExportKind::Axiom)
                        })
                        .count() as u64,
                    simp_rule_count: 0,
                })
                .collect(),
            import_bundles_hash,
            theorem_index_hash: test_hash(2),
            simp_profiles_hash: test_hash(3),
            rewrite_profiles_hash: test_hash(4),
            axiom_report_hash,
        }
    }

    fn release_manifest_json(release: &MachineStdLibraryRelease) -> String {
        format!(
            "{{\"protocol_version\":\"{}\",\"library_profile_id\":\"{}\",\"core_spec_id\":\"{}\",\"kernel_semantics_profile_id\":\"{}\",\"modules\":[{}],\"import_bundles_hash\":\"{}\",\"theorem_index_hash\":\"{}\",\"simp_profiles_hash\":\"{}\",\"rewrite_profiles_hash\":\"{}\",\"axiom_report_hash\":\"{}\"}}",
            release.protocol_version,
            release.library_profile_id,
            release.core_spec_id,
            release.kernel_semantics_profile_id,
            release
                .modules
                .iter()
                .map(module_artifact_json)
                .collect::<Vec<_>>()
                .join(","),
            format_hash_string(&release.import_bundles_hash),
            format_hash_string(&release.theorem_index_hash),
            format_hash_string(&release.simp_profiles_hash),
            format_hash_string(&release.rewrite_profiles_hash),
            format_hash_string(&release.axiom_report_hash),
        )
    }

    fn module_artifact_json(module: &MachineStdModuleArtifact) -> String {
        format!(
            "{{\"module\":\"{}\",\"expected_export_hash\":\"{}\",\"expected_certificate_hash\":\"{}\",\"certificate_encoding\":\"{}\",\"certificate_bytes_hash\":\"{}\",\"axiom_report_hash\":\"{}\",\"public_export_count\":{},\"theorem_index_entry_count\":{},\"simp_rule_count\":{}}}",
            module.module.as_dotted(),
            format_hash_string(&module.expected_export_hash),
            format_hash_string(&module.expected_certificate_hash),
            module.certificate_encoding,
            format_hash_string(&module.certificate_bytes_hash),
            format_hash_string(&module.axiom_report_hash),
            module.public_export_count,
            module.theorem_index_entry_count,
            module.simp_rule_count,
        )
    }

    fn axiom_report_json(report: &MachineStdAxiomReport) -> String {
        format!(
            "{{\"library_profile_id\":\"{}\",\"modules\":[{}],\"axiom_report_hash\":\"{}\"}}",
            report.library_profile_id,
            report
                .modules
                .iter()
                .map(module_axiom_report_json)
                .collect::<Vec<_>>()
                .join(","),
            format_hash_string(&report.axiom_report_hash),
        )
    }

    fn module_axiom_report_json(module: &MachineStdModuleAxiomReport) -> String {
        format!(
            "{{\"module\":\"{}\",\"export_hash\":\"{}\",\"certificate_hash\":\"{}\",\"module_axioms\":[{}],\"transitive_axioms\":[{}]}}",
            module.module.as_dotted(),
            format_hash_string(&module.export_hash),
            format_hash_string(&module.certificate_hash),
            module
                .module_axioms
                .iter()
                .map(axiom_ref_json)
                .collect::<Vec<_>>()
                .join(","),
            module
                .transitive_axioms
                .iter()
                .map(axiom_ref_json)
                .collect::<Vec<_>>()
                .join(","),
        )
    }

    fn axiom_ref_json(axiom: &MachineStdAxiomRef) -> String {
        format!(
            "{{\"module\":\"{}\",\"name\":\"{}\",\"export_hash\":\"{}\",\"decl_interface_hash\":\"{}\"}}",
            axiom.module.as_dotted(),
            axiom.name.as_dotted(),
            format_hash_string(&axiom.export_hash),
            format_hash_string(&axiom.decl_interface_hash),
        )
    }

    fn import_bundle_set_json(bundle_set: &MachineStdImportBundleSet) -> String {
        format!(
            "{{\"library_profile_id\":\"{}\",\"bundles\":[{}],\"import_bundles_hash\":\"{}\"}}",
            bundle_set.library_profile_id,
            bundle_set
                .bundles
                .iter()
                .map(import_bundle_json)
                .collect::<Vec<_>>()
                .join(","),
            format_hash_string(&bundle_set.import_bundles_hash),
        )
    }

    fn import_bundle_json(bundle: &MachineStdImportBundle) -> String {
        format!(
            "{{\"bundle_id\":\"{}\",\"root_imports\":[{}],\"import_closure\":[{}],\"allow_axioms\":[{}],\"recommended_tactic_options\":{}}}",
            bundle.bundle_id,
            bundle
                .root_imports
                .iter()
                .map(import_key_json)
                .collect::<Vec<_>>()
                .join(","),
            bundle
                .import_closure
                .iter()
                .map(import_certificate_json)
                .collect::<Vec<_>>()
                .join(","),
            bundle
                .allow_axioms
                .iter()
                .map(machine_axiom_ref_wire_json)
                .collect::<Vec<_>>()
                .join(","),
            tactic_options_recipe_json(&bundle.recommended_tactic_options),
        )
    }

    fn import_key_json(key: &VerifiedImportKey) -> String {
        format!(
            "{{\"module\":\"{}\",\"expected_export_hash\":\"{}\",\"expected_certificate_hash\":\"{}\"}}",
            key.module.as_dotted(),
            format_hash_string(&key.export_hash),
            format_hash_string(&key.certificate_hash),
        )
    }

    fn import_certificate_json(certificate: &MachineStdImportCertificate) -> String {
        format!(
            "{{\"module\":\"{}\",\"expected_export_hash\":\"{}\",\"expected_certificate_hash\":\"{}\",\"certificate\":{{\"encoding\":\"{}\",\"bytes\":\"{}\"}}}}",
            certificate.module.as_dotted(),
            format_hash_string(&certificate.expected_export_hash),
            format_hash_string(&certificate.expected_certificate_hash),
            certificate.certificate_encoding,
            lower_hex_bytes(&certificate.certificate_bytes),
        )
    }

    fn tactic_options_recipe_json(recipe: &MachineStdTacticOptionsRecipe) -> String {
        format!(
            "{{\"recipe_id\":\"{}\",\"kernel_check_profile\":\"{}\",\"simp_rules\":[{}],\"eq_family\":{},\"nat_family\":{},\"max_simp_rewrite_steps\":{},\"max_open_goals\":{},\"max_metas\":{}}}",
            recipe.recipe_id,
            recipe.kernel_check_profile,
            recipe
                .simp_rules
                .iter()
                .map(simp_rule_json)
                .collect::<Vec<_>>()
                .join(","),
            recipe
                .eq_family
                .as_ref()
                .map(eq_family_json)
                .unwrap_or_else(|| "null".to_owned()),
            recipe
                .nat_family
                .as_ref()
                .map(nat_family_json)
                .unwrap_or_else(|| "null".to_owned()),
            recipe.max_simp_rewrite_steps,
            recipe.max_open_goals,
            recipe.max_metas,
        )
    }

    fn simp_rule_json(rule: &SimpRuleRef) -> String {
        let direction = match rule.direction {
            RewriteDirection::Forward => "forward",
            RewriteDirection::Backward => "backward",
        };
        format!(
            "{{\"name\":\"{}\",\"decl_interface_hash\":\"{}\",\"direction\":\"{}\"}}",
            rule.name.as_dotted(),
            format_hash_string(&rule.decl_interface_hash),
            direction,
        )
    }

    fn eq_family_json(family: &EqFamilyRef) -> String {
        format!(
            "{{\"eq_name\":\"{}\",\"eq_interface_hash\":\"{}\",\"refl_name\":\"{}\",\"refl_interface_hash\":\"{}\",\"rec_name\":\"{}\",\"rec_interface_hash\":\"{}\"}}",
            family.eq_name.as_dotted(),
            format_hash_string(&family.eq_interface_hash),
            family.refl_name.as_dotted(),
            format_hash_string(&family.refl_interface_hash),
            family.rec_name.as_dotted(),
            format_hash_string(&family.rec_interface_hash),
        )
    }

    fn nat_family_json(family: &NatFamilyRef) -> String {
        format!(
            "{{\"nat_name\":\"{}\",\"nat_interface_hash\":\"{}\",\"zero_name\":\"{}\",\"zero_interface_hash\":\"{}\",\"succ_name\":\"{}\",\"succ_interface_hash\":\"{}\",\"rec_name\":\"{}\",\"rec_interface_hash\":\"{}\"}}",
            family.nat_name.as_dotted(),
            format_hash_string(&family.nat_interface_hash),
            family.zero_name.as_dotted(),
            format_hash_string(&family.zero_interface_hash),
            family.succ_name.as_dotted(),
            format_hash_string(&family.succ_interface_hash),
            family.rec_name.as_dotted(),
            format_hash_string(&family.rec_interface_hash),
        )
    }

    fn machine_axiom_ref_wire_json(axiom: &MachineAxiomRefWire) -> String {
        match axiom {
            MachineAxiomRefWire::Imported {
                module,
                name,
                export_hash,
                decl_interface_hash,
            } => format!(
                "{{\"kind\":\"imported\",\"module\":\"{}\",\"name\":\"{}\",\"export_hash\":\"{}\",\"decl_interface_hash\":\"{}\"}}",
                module.as_dotted(),
                name.as_dotted(),
                format_hash_string(export_hash),
                format_hash_string(decl_interface_hash),
            ),
            MachineAxiomRefWire::CurrentModule {
                module,
                name,
                source_index,
                decl_interface_hash,
            } => format!(
                "{{\"kind\":\"current_module\",\"module\":\"{}\",\"name\":\"{}\",\"source_index\":{},\"decl_interface_hash\":\"{}\"}}",
                module.as_dotted(),
                name.as_dotted(),
                source_index,
                format_hash_string(decl_interface_hash),
            ),
            MachineAxiomRefWire::Builtin {
                name,
                decl_interface_hash,
            } => format!(
                "{{\"kind\":\"builtin\",\"name\":\"{}\",\"decl_interface_hash\":\"{}\"}}",
                name.as_dotted(),
                format_hash_string(decl_interface_hash),
            ),
        }
    }

    fn lower_hex_bytes(bytes: &[u8]) -> String {
        let mut out = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            out.push(hex_digit(byte >> 4));
            out.push(hex_digit(byte & 0x0f));
        }
        out
    }

    fn hex_digit(value: u8) -> char {
        match value {
            0..=9 => (b'0' + value) as char,
            10..=15 => (b'a' + (value - 10)) as char,
            _ => unreachable!("hex nybble is in range"),
        }
    }

    fn test_hash(seed: u8) -> Hash {
        [seed; 32]
    }

    struct TestPackage {
        path: PathBuf,
    }

    impl TestPackage {
        fn new(label: &str) -> Self {
            let mut path = std::env::temp_dir();
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            path.push(format!(
                "npa-stdlib-loader-{label}-{}-{nanos}",
                std::process::id()
            ));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestPackage {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
