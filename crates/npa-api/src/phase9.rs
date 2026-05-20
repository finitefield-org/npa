use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use npa_cert::{
    AxiomPolicy, CoreModule, ExportKind, Hash, InductiveArtifactProfileCheckV1, ModuleName, Name,
    VerifierSession,
};
use npa_kernel::{Binder, ConstructorDecl, Ctx, Decl, Env, Expr, InductiveDecl, Level};
use npa_tactic::{machine_local_context_canonical_bytes, MachineLocalDecl, VerifiedImportRef};
use sha2::{Digest, Sha256};

use crate::types::phase5_name_canonical_bytes;

const CANDIDATE_HASH_TAG: &str = "npa.phase9_ai.candidate.v1";
const OPTIONS_HASH_TAG: &str = "npa.phase9_ai.options.v1";
const ENV_FINGERPRINT_TAG: &str = "npa.phase9_ai.env.v1";
const GOAL_FINGERPRINT_TAG: &str = "npa.phase9_ai.goal.v1";
const VALIDATION_RESULT_HASH_TAG: &str = "npa.phase9_ai.validation_result.v1";
const UNIVERSE_CONSTRAINT_SET_HASH_TAG: &str = "npa.phase9_ai.universe.constraints.v1";
const THEOREM_GRAPH_SNAPSHOT_HASH_TAG: &str = "npa.phase9_ai.theorem_graph.snapshot.v1";
const THEOREM_GRAPH_QUERY_FEATURES_HASH_TAG: &str = "npa.phase9_ai.theorem_graph.query_features.v1";

const MAX_OPTIONS_BYTES: usize = 16_000_000;
const MAX_PHASE9_GLOBAL_REFS: u64 = 65_536;
const MAX_PHASE9_INDUCTIVE_ITEMS: u64 = 65_536;
const MAX_PHASE9_INDUCTIVE_EXPR_NODES: u64 = 1_000_000;
const MAX_PHASE9_INDUCTIVE_LEVEL_NODES: u64 = 1_000_000;
const MAX_PHASE9_THEOREM_GRAPH_SNAPSHOT_BYTES: usize = 128_000_000;
const MAX_PHASE9_THEOREM_GRAPH_QUERY_FEATURES_BYTES: usize = 16_000_000;
const MAX_PHASE9_THEOREM_GRAPH_NODES: u64 = 1_000_000;
const MAX_PHASE9_THEOREM_GRAPH_EDGES: u64 = 1_000_000;
const MAX_PHASE9_THEOREM_GRAPH_FEATURES: u64 = 65_536;
const MAX_PHASE9_THEOREM_GRAPH_RESULT_LIMIT: u32 = 256;
const MAX_PHASE9_UNIVERSE_REPAIR_ITEMS: u64 = 65_536;
const MAX_NAME_COMPONENTS: u64 = 256;
const MAX_STRING_BYTES: u64 = 1_048_576;

pub const PHASE9_INDUCTIVE_CHECK_ENDPOINT: &str = "/machine/phase9/inductive/check";
pub const PHASE9_UNIVERSE_REPAIR_CHECK_ENDPOINT: &str = "/machine/phase9/universe/repair/check";
pub const PHASE9_TYPECLASS_RESOLVE_ENDPOINT: &str = "/machine/phase9/typeclass/resolve";
pub const PHASE9_QUOTIENT_CHECK_ENDPOINT: &str = "/machine/phase9/quotient/check";
pub const PHASE9_SMT_RECONSTRUCT_ENDPOINT: &str = "/machine/phase9/smt/reconstruct";
pub const PHASE9_THEOREM_GRAPH_QUERY_ENDPOINT: &str = "/machine/phase9/theorem-graph/query";
pub const PHASE9_FORMALIZE_CHECK_ENDPOINT: &str = "/machine/phase9/formalize/check";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9AiProfileVersion {
    MvpV1,
}

impl Phase9AiProfileVersion {
    fn tag(self) -> u8 {
        match self {
            Self::MvpV1 => 0,
        }
    }

    fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0 => Some(Self::MvpV1),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9AiTaskKind {
    AdvancedInductive,
    UniverseRepair,
    TypeclassResolution,
    QuotientConstruction,
    SmtCertificate,
    TheoremGraphQuery,
    NaturalLanguageFormalization,
}

impl Phase9AiTaskKind {
    fn tag(self) -> u8 {
        match self {
            Self::AdvancedInductive => 0,
            Self::UniverseRepair => 1,
            Self::TypeclassResolution => 2,
            Self::QuotientConstruction => 3,
            Self::SmtCertificate => 4,
            Self::TheoremGraphQuery => 5,
            Self::NaturalLanguageFormalization => 6,
        }
    }

    fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0 => Some(Self::AdvancedInductive),
            1 => Some(Self::UniverseRepair),
            2 => Some(Self::TypeclassResolution),
            3 => Some(Self::QuotientConstruction),
            4 => Some(Self::SmtCertificate),
            5 => Some(Self::TheoremGraphQuery),
            6 => Some(Self::NaturalLanguageFormalization),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9AiTarget {
    pub env_fingerprint: Hash,
    pub target_decl_hash: Option<Hash>,
    pub goal_fingerprint: Option<Hash>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9ImportIdentity {
    pub module: ModuleName,
    pub export_hash: Hash,
    pub certificate_hash: Hash,
}

impl Phase9ImportIdentity {
    pub fn from_verified_import(import: &VerifiedImportRef) -> Self {
        Self {
            module: import.module().clone(),
            export_hash: import.export_hash(),
            certificate_hash: import.certificate_hash(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase9AiOptionsRef {
    Inline {
        options_hash: Hash,
        canonical_bytes: Vec<u8>,
    },
    Artifact {
        path: String,
        file_hash: Hash,
        options_hash: Hash,
        size_bytes: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9AiCandidateEnvelope {
    pub profile_version: Phase9AiProfileVersion,
    pub task_kind: Phase9AiTaskKind,
    pub target: Phase9AiTarget,
    pub imports: Vec<Phase9ImportIdentity>,
    pub options: Phase9AiOptionsRef,
    pub payload: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9AiOptionsVersion {
    MvpV1,
}

impl Phase9AiOptionsVersion {
    fn tag(self) -> u8 {
        match self {
            Self::MvpV1 => 0,
        }
    }

    fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0 => Some(Self::MvpV1),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9IndependentCheckerProfile {
    Phase8MvpReference,
}

impl Phase9IndependentCheckerProfile {
    fn tag(self) -> u8 {
        match self {
            Self::Phase8MvpReference => 0,
        }
    }

    fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0 => Some(Self::Phase8MvpReference),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9IndependentCheckerOptions {
    pub profile: Phase9IndependentCheckerProfile,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9AdvancedInductiveOptions {
    pub approved_nested_type_constructors: Vec<Phase9AiGlobalRef>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9TypeclassOptions {
    pub class_declarations: Vec<Phase9AiGlobalRef>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9QuotientOptions {
    pub setoid: Phase9AiGlobalRef,
    pub setoid_mk: Phase9AiGlobalRef,
    pub setoid_relation: Phase9AiGlobalRef,
    pub rel_equiv: Phase9AiGlobalRef,
    pub quotient: Phase9AiGlobalRef,
    pub quotient_mk: Phase9AiGlobalRef,
    pub quotient_sound: Phase9AiGlobalRef,
    pub quotient_lift: Phase9AiGlobalRef,
    pub eq: Phase9AiGlobalRef,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9SmtOptions {
    pub eq: Phase9AiGlobalRef,
    pub prop_false: Option<Phase9AiGlobalRef>,
    pub prop_not: Option<Phase9AiGlobalRef>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9FormalizationOptions {
    pub tactic_options_canonical_bytes: Vec<u8>,
    pub tactic_budget_canonical_bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9AiOptions {
    pub schema_version: Phase9AiOptionsVersion,
    pub independent_checker: Phase9IndependentCheckerOptions,
    pub advanced_inductive: Phase9AdvancedInductiveOptions,
    pub typeclass: Phase9TypeclassOptions,
    pub quotient: Option<Phase9QuotientOptions>,
    pub smt: Option<Phase9SmtOptions>,
    pub formalization: Option<Phase9FormalizationOptions>,
}

impl Default for Phase9AiOptions {
    fn default() -> Self {
        Self {
            schema_version: Phase9AiOptionsVersion::MvpV1,
            independent_checker: Phase9IndependentCheckerOptions {
                profile: Phase9IndependentCheckerProfile::Phase8MvpReference,
            },
            advanced_inductive: Phase9AdvancedInductiveOptions {
                approved_nested_type_constructors: Vec::new(),
            },
            typeclass: Phase9TypeclassOptions {
                class_declarations: Vec::new(),
            },
            quotient: None,
            smt: None,
            formalization: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9AiGlobalRef {
    pub module: ModuleName,
    pub export_hash: Hash,
    pub certificate_hash: Hash,
    pub name: Name,
    pub decl_interface_hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9MachineInductiveProposal {
    pub block_name: Option<Name>,
    pub expected_decl_hash: Option<Hash>,
    pub universe_params: Vec<String>,
    pub inductives: Vec<Phase9MachineInductiveFamilyProposal>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9MachineInductiveFamilyProposal {
    pub name: Name,
    pub params: Vec<Phase9MachineTelescopeBinder>,
    pub indices: Vec<Phase9MachineTelescopeBinder>,
    pub result_sort: Level,
    pub constructors: Vec<Phase9MachineConstructorProposal>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9MachineTelescopeBinder {
    pub ty: Expr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9MachineConstructorProposal {
    pub name: Name,
    pub ty: Expr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9AiGoal {
    pub universe_params: Vec<String>,
    pub local_context: Vec<MachineLocalDecl>,
    pub target: Expr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9UniverseRepairCandidate {
    pub goal: Option<Phase9AiGoal>,
    pub target_expr: Expr,
    pub instantiations: Vec<Phase9UniverseInstantiationPatch>,
    pub constraint_hints: Vec<Phase9UniverseConstraintHint>,
    pub minimization_hint: Option<Phase9UniverseMinimizationHint>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9UniverseInstantiationPatch {
    pub occurrence: Phase9MachineExprOccurrence,
    pub explicit_level_args: Vec<Level>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9MachineExprOccurrence {
    pub path: Vec<Phase9MachineExprPathStep>,
    pub expected_ref: Phase9AiGlobalRef,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Phase9MachineExprPathStep {
    AppFun,
    AppArg,
    LamType,
    LamBody,
    PiDomain,
    PiCodomain,
    LetType,
    LetValue,
    LetBody,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9UniverseConstraintHint {
    pub constraint: Phase9UniverseConstraint,
    pub reason: Phase9UniverseConstraintHintReason,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9UniverseConstraint {
    pub lhs: Level,
    pub relation: Phase9UniverseConstraintRelation,
    pub rhs: Level,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9UniverseConstraintRelation {
    Le,
    Eq,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9UniverseConstraintHintReason {
    KernelDiagnostic,
    RepairCandidate,
    MinimizationExplanation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9UniverseMinimizationHint {
    KernelDefault,
    PreferLowerLevels,
    PreferExistingExplicitArgs,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9AiValidationError {
    EnvelopeMalformed,
    TargetFingerprintMismatch,
    ImportClosureMismatch,
    PayloadHashMismatch,
    KernelRejected,
    IndependentCheckerRejected,
    NonDeterministicResult,
    BudgetExceeded,
    AmbiguousResolution,
    NoSolution,
    FeatureRejected,
    UnsupportedFeature,
}

impl Phase9AiValidationError {
    fn tag(self) -> u8 {
        match self {
            Self::EnvelopeMalformed => 0,
            Self::TargetFingerprintMismatch => 1,
            Self::ImportClosureMismatch => 2,
            Self::PayloadHashMismatch => 3,
            Self::KernelRejected => 4,
            Self::IndependentCheckerRejected => 5,
            Self::NonDeterministicResult => 6,
            Self::BudgetExceeded => 7,
            Self::AmbiguousResolution => 8,
            Self::NoSolution => 9,
            Self::FeatureRejected => 10,
            Self::UnsupportedFeature => 11,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9AiEndpointError {
    NonCanonicalRequestBytes,
    ArtifactUnavailable,
    InternalValidatorFailure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9AiFeatureError {
    AdvancedInductive(Phase9AdvancedInductiveError),
    UniverseRepair(Phase9UniverseRepairError),
    TypeclassResolution(Phase9TypeclassResolutionError),
    QuotientConstruction(Phase9QuotientConstructionError),
    SmtCertificate(Phase9SmtCertificateError),
    TheoremGraphQuery(Phase9TheoremGraphError),
    Formalization(Phase9FormalizationError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9AdvancedInductiveError {
    TargetRefMismatch,
    PositivityProfileUnsupported,
    ArtifactGeneratorUnavailable,
    GeneratedArtifactMismatch,
    NameCollision,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9UniverseRepairError {
    UnknownUniverseParam,
    IllFormedLevelExpr,
    UnsatisfiedConstraint,
    NonCanonicalSolution,
    TargetFingerprintMismatch,
    InvalidOccurrencePath,
    AmbiguousOccurrence,
    TargetRefMismatch,
    ConstraintHintMismatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9TypeclassResolutionError {
    ClassDeclarationMismatch,
    CandidateInterfaceInvalid,
    ClassHeadUnsupported,
    NoSolution,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9QuotientConstructionError {
    TargetRefMismatch,
    PrimitiveInterfaceMismatch,
    UniverseLevelMismatch,
    CompatibilityProofMismatch,
    QuotientTypeMismatch,
    RelationTypeMismatch,
    EquivalenceProofMismatch,
    RawFunctionTypeMismatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9SmtCertificateError {
    EncodingMismatch,
    RuleFingerprintMismatch,
    RuleRegistryMismatch,
    NonCanonicalPayload,
    ReconstructionProofMismatch,
    ConclusionEncodingMismatch,
    PayloadBindingMismatch,
    ReconstructionConclusionMismatch,
    ReconstructionPremiseMismatch,
    PublicInterfaceMismatch,
    TheoryRefMismatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9TheoremGraphError {
    SnapshotMalformed,
    QueryFeaturesMalformed,
    NodeResolutionMismatch,
    LimitOutOfRange,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9FormalizationError {
    IntentRecordMismatch,
    CandidateStatementElaborationFailed,
    FormalizationProofStatementMismatch,
    RejectedIntentHasProofCandidate,
    ProofBridgeFailed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase9AiSuccessPayload {
    AdvancedInductive {
        decl_interface_hash: Hash,
        decl_certificate_hash: Hash,
    },
    UniverseRepair {
        repaired_expr: Expr,
        constraint_set_hash: Hash,
    },
    TypeclassResolution {
        proof: Expr,
    },
    QuotientConstruction {
        decl_certificate_hash: Hash,
    },
    SmtCertificate {
        final_proof: Expr,
    },
    TheoremGraphQuery {
        result: Phase9MachineTheoremGraphResult,
    },
    NaturalLanguageFormalization {
        kind: Phase9FormalizationSuccessKind,
        accepted_statement_hash: Option<Hash>,
        formalization_proof_root_hash: Option<Hash>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9MachineTheoremGraphResult {
    pub entries: Vec<Phase9MachineTheoremGraphResultEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9MachineTheoremGraphResultEntry {
    pub node: Phase9MachineTheoremGraphNodeRef,
    pub score: Phase9MachineTheoremGraphScore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Phase9MachineTheoremGraphScore {
    pub score_microunits: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9MachineTheoremGraphNodeRef {
    pub module: ModuleName,
    pub name: Name,
    pub export_hash: Hash,
    pub decl_certificate_hash: Hash,
    pub type_hash: Hash,
    pub certificate_hash: Hash,
    pub decl_interface_hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9MachineTheoremGraphQuery {
    pub env_fingerprint: Hash,
    pub goal_fingerprint: Hash,
    pub goal: Phase9AiGoal,
    pub snapshot: Phase9MachineTheoremGraphSnapshotRef,
    pub query_features: Phase9MachineTheoremGraphQueryFeaturesRef,
    pub ranking_profile: Phase9TheoremGraphRankingProfile,
    pub limit: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9MachineTheoremGraphSnapshotRef {
    pub source_release_hash: Hash,
    pub extractor_version: Phase9TheoremGraphExtractorVersion,
    pub source: Phase9MachineTheoremGraphSnapshotSource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase9MachineTheoremGraphSnapshotSource {
    Inline {
        graph_snapshot_hash: Hash,
        canonical_bytes: Vec<u8>,
    },
    Artifact {
        path: String,
        file_hash: Hash,
        graph_snapshot_hash: Hash,
        size_bytes: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase9MachineTheoremGraphQueryFeaturesRef {
    Inline {
        query_features_hash: Hash,
        canonical_bytes: Vec<u8>,
    },
    Artifact {
        path: String,
        file_hash: Hash,
        query_features_hash: Hash,
        size_bytes: u64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9TheoremGraphRankingProfile {
    MvpTupleOrder,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9MachineTheoremGraphSnapshot {
    pub source_release_hash: Hash,
    pub extractor_version: Phase9TheoremGraphExtractorVersion,
    pub nodes: Vec<Phase9MachineTheoremGraphNodeRef>,
    pub edges: Vec<Phase9MachineTheoremGraphEdge>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9MachineTheoremGraphEdge {
    pub from: Phase9MachineTheoremGraphNodeRef,
    pub to: Phase9MachineTheoremGraphNodeRef,
    pub kind: Phase9TheoremGraphEdgeKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9MachineTheoremGraphQueryFeatures {
    pub env_fingerprint: Hash,
    pub goal_fingerprint: Hash,
    pub feature_schema_version: Phase9TheoremGraphFeatureSchemaVersion,
    pub features: Vec<Phase9MachineTheoremGraphFeature>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9MachineTheoremGraphFeature {
    pub key: Phase9TheoremGraphFeatureKey,
    pub value: Phase9TheoremGraphFeatureValue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9TheoremGraphExtractorVersion {
    MvpCertificateGraphV1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9TheoremGraphFeatureSchemaVersion {
    MvpGoalFeaturesV1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9TheoremGraphEdgeKind {
    ImportsDeclaration,
    UsesConstant,
    MentionsType,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9TheoremGraphFeatureKey {
    pub namespace_ascii: Vec<u8>,
    pub name_ascii: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase9TheoremGraphFeatureValue {
    Bool(bool),
    I64(i64),
    Hash(Hash),
}

impl Phase9TheoremGraphRankingProfile {
    fn tag(self) -> u8 {
        match self {
            Self::MvpTupleOrder => 0,
        }
    }

    fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0 => Some(Self::MvpTupleOrder),
            _ => None,
        }
    }
}

impl Phase9TheoremGraphExtractorVersion {
    fn tag(self) -> u8 {
        match self {
            Self::MvpCertificateGraphV1 => 0,
        }
    }

    fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0 => Some(Self::MvpCertificateGraphV1),
            _ => None,
        }
    }
}

impl Phase9TheoremGraphFeatureSchemaVersion {
    fn tag(self) -> u8 {
        match self {
            Self::MvpGoalFeaturesV1 => 0,
        }
    }

    fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0 => Some(Self::MvpGoalFeaturesV1),
            _ => None,
        }
    }
}

impl Phase9TheoremGraphEdgeKind {
    fn tag(self) -> u8 {
        match self {
            Self::ImportsDeclaration => 0,
            Self::UsesConstant => 1,
            Self::MentionsType => 2,
        }
    }

    fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0 => Some(Self::ImportsDeclaration),
            1 => Some(Self::UsesConstant),
            2 => Some(Self::MentionsType),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9FormalizationSuccessKind {
    CandidateStatementChecked,
    IntentRecordOnly,
    ProofBridgeChecked,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase9AiEndpointResponse {
    Success {
        candidate_hash: Hash,
        validation_result_hash: Hash,
        payload: Box<Phase9AiSuccessPayload>,
    },
    Rejected {
        candidate_hash: Hash,
        validation_result_hash: Hash,
        error: Phase9AiValidationError,
        feature_error: Option<Phase9AiFeatureError>,
    },
    Error {
        error: Phase9AiEndpointError,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase9ValidatedCommonEnvelope {
    pub candidate_hash: Hash,
    pub options_hash: Hash,
    pub env_fingerprint: Hash,
    pub envelope: Phase9AiCandidateEnvelope,
    pub options: Phase9AiOptions,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase9AiCanonicalError {
    InvalidName,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DecodeError {
    Malformed,
    TheoremGraphSnapshotBytesTooLarge,
    TheoremGraphQueryFeaturesBytesTooLarge,
}

pub fn phase9_ai_candidate_hash(envelope_canonical_bytes: &[u8]) -> Hash {
    hash_with_domain(CANDIDATE_HASH_TAG, envelope_canonical_bytes)
}

pub fn phase9_ai_options_hash(options_canonical_bytes: &[u8]) -> Hash {
    hash_with_domain(OPTIONS_HASH_TAG, options_canonical_bytes)
}

pub fn phase9_file_hash(bytes: &[u8]) -> Hash {
    sha256(bytes)
}

pub fn phase9_ai_validation_result_hash_for_rejection(
    candidate_hash: Hash,
    error: Phase9AiValidationError,
    feature_error: Option<Phase9AiFeatureError>,
) -> Hash {
    let mut payload = Vec::new();
    payload.push(1);
    encode_validation_error_to(&mut payload, error);
    encode_feature_error_option_to(&mut payload, feature_error);
    validation_result_hash(candidate_hash, &payload)
}

pub fn phase9_ai_validation_result_hash_for_success(
    candidate_hash: Hash,
    success: &Phase9AiSuccessPayload,
) -> Hash {
    let mut payload = Vec::new();
    payload.push(0);
    encode_success_payload_to(&mut payload, success);
    validation_result_hash(candidate_hash, &payload)
}

pub fn phase9_ai_env_fingerprint(
    profile_version: Phase9AiProfileVersion,
    task_kind: Phase9AiTaskKind,
    imports: &[Phase9ImportIdentity],
    options_hash: Hash,
) -> std::result::Result<Hash, Phase9AiCanonicalError> {
    let mut payload = Vec::new();
    payload.push(profile_version.tag());
    payload.push(task_kind.tag());
    encode_import_identities_to(&mut payload, imports)?;
    encode_hash_to(&mut payload, &options_hash);
    Ok(hash_with_domain(ENV_FINGERPRINT_TAG, &payload))
}

pub fn phase9_ai_goal_fingerprint(env_fingerprint: Hash, goal: &Phase9AiGoal) -> Hash {
    let mut payload = Vec::new();
    encode_hash_to(&mut payload, &env_fingerprint);
    payload.extend_from_slice(&phase9_universe_params_canonical_bytes(
        &goal.universe_params,
    ));
    payload.extend_from_slice(&machine_local_context_canonical_bytes(&goal.local_context));
    payload.extend_from_slice(&npa_cert::core_expr_canonical_bytes(&goal.target));
    hash_with_domain(GOAL_FINGERPRINT_TAG, &payload)
}

pub fn phase9_ai_goal_canonical_bytes(
    goal: &Phase9AiGoal,
) -> std::result::Result<Vec<u8>, Phase9AiCanonicalError> {
    let mut out = Vec::new();
    encode_goal_to(&mut out, goal)?;
    Ok(out)
}

pub fn phase9_inductive_proposal_canonical_bytes(
    proposal: &Phase9MachineInductiveProposal,
) -> std::result::Result<Vec<u8>, Phase9AiCanonicalError> {
    let mut out = Vec::new();
    encode_inductive_proposal_to(&mut out, proposal)?;
    Ok(out)
}

pub fn phase9_theorem_graph_query_canonical_bytes(
    query: &Phase9MachineTheoremGraphQuery,
) -> std::result::Result<Vec<u8>, Phase9AiCanonicalError> {
    let mut out = Vec::new();
    encode_theorem_graph_query_to(&mut out, query)?;
    Ok(out)
}

pub fn phase9_theorem_graph_snapshot_canonical_bytes(
    snapshot: &Phase9MachineTheoremGraphSnapshot,
) -> std::result::Result<Vec<u8>, Phase9AiCanonicalError> {
    let mut out = Vec::new();
    encode_theorem_graph_snapshot_to(&mut out, snapshot)?;
    Ok(out)
}

pub fn phase9_theorem_graph_query_features_canonical_bytes(
    features: &Phase9MachineTheoremGraphQueryFeatures,
) -> std::result::Result<Vec<u8>, Phase9AiCanonicalError> {
    let mut out = Vec::new();
    encode_theorem_graph_query_features_to(&mut out, features)?;
    Ok(out)
}

pub fn phase9_theorem_graph_snapshot_hash(
    canonical_bytes: &[u8],
) -> std::result::Result<Hash, Phase9AiCanonicalError> {
    decode_theorem_graph_snapshot(canonical_bytes)
        .map_err(|_| Phase9AiCanonicalError::InvalidName)?;
    Ok(hash_with_domain(
        THEOREM_GRAPH_SNAPSHOT_HASH_TAG,
        canonical_bytes,
    ))
}

pub fn phase9_theorem_graph_query_features_hash(
    canonical_bytes: &[u8],
) -> std::result::Result<Hash, Phase9AiCanonicalError> {
    decode_theorem_graph_query_features(canonical_bytes)
        .map_err(|_| Phase9AiCanonicalError::InvalidName)?;
    Ok(hash_with_domain(
        THEOREM_GRAPH_QUERY_FEATURES_HASH_TAG,
        canonical_bytes,
    ))
}

pub fn phase9_universe_repair_candidate_canonical_bytes(
    candidate: &Phase9UniverseRepairCandidate,
) -> std::result::Result<Vec<u8>, Phase9AiCanonicalError> {
    let mut out = Vec::new();
    encode_universe_repair_candidate_to(&mut out, candidate)?;
    Ok(out)
}

pub fn phase9_ai_options_canonical_bytes(
    options: &Phase9AiOptions,
) -> std::result::Result<Vec<u8>, Phase9AiCanonicalError> {
    let mut out = Vec::new();
    encode_options_to(&mut out, options)?;
    Ok(out)
}

pub fn phase9_ai_candidate_envelope_canonical_bytes(
    envelope: &Phase9AiCandidateEnvelope,
) -> std::result::Result<Vec<u8>, Phase9AiCanonicalError> {
    let mut out = Vec::new();
    encode_candidate_envelope_to(&mut out, envelope)?;
    Ok(out)
}

pub fn validate_phase9_ai_common_envelope(
    request_canonical_bytes: &[u8],
    verified_imports: &[VerifiedImportRef],
    workspace_root: &Path,
    expected_task_kind: Phase9AiTaskKind,
) -> std::result::Result<Phase9ValidatedCommonEnvelope, Phase9AiEndpointResponse> {
    let envelope = match decode_candidate_envelope(request_canonical_bytes) {
        Ok(envelope) => envelope,
        Err(_) => {
            return Err(Phase9AiEndpointResponse::Error {
                error: Phase9AiEndpointError::NonCanonicalRequestBytes,
            });
        }
    };
    let candidate_hash = phase9_ai_candidate_hash(request_canonical_bytes);

    if envelope.task_kind != expected_task_kind {
        return Err(rejected_response(
            candidate_hash,
            Phase9AiValidationError::EnvelopeMalformed,
            None,
        ));
    }

    validate_imports(candidate_hash, &envelope.imports, verified_imports)?;

    let (options, options_hash) =
        validate_options_ref(candidate_hash, &envelope.options, workspace_root)?;

    if !options
        .advanced_inductive
        .approved_nested_type_constructors
        .is_empty()
    {
        return Err(rejected_response(
            candidate_hash,
            Phase9AiValidationError::UnsupportedFeature,
            Some(Phase9AiFeatureError::AdvancedInductive(
                Phase9AdvancedInductiveError::PositivityProfileUnsupported,
            )),
        ));
    }

    let env_fingerprint = phase9_ai_env_fingerprint(
        envelope.profile_version,
        envelope.task_kind,
        &envelope.imports,
        options_hash,
    )
    .map_err(|_| {
        rejected_response(
            candidate_hash,
            Phase9AiValidationError::EnvelopeMalformed,
            None,
        )
    })?;

    if envelope.target.env_fingerprint != env_fingerprint {
        return Err(rejected_response(
            candidate_hash,
            Phase9AiValidationError::TargetFingerprintMismatch,
            None,
        ));
    }

    validate_target_shape(candidate_hash, envelope.task_kind, &envelope.target)?;
    validate_required_options(candidate_hash, envelope.task_kind, &options)?;

    Ok(Phase9ValidatedCommonEnvelope {
        candidate_hash,
        options_hash,
        env_fingerprint,
        envelope,
        options,
    })
}

pub fn run_phase9_inductive_check_request(
    request_canonical_bytes: &[u8],
    verified_imports: &[VerifiedImportRef],
    workspace_root: &Path,
) -> Phase9AiEndpointResponse {
    match validate_phase9_ai_common_envelope(
        request_canonical_bytes,
        verified_imports,
        workspace_root,
        Phase9AiTaskKind::AdvancedInductive,
    ) {
        Ok(validated) => run_phase9_inductive_validated(validated, verified_imports),
        Err(response) => response,
    }
}

pub fn run_phase9_universe_repair_check_request(
    request_canonical_bytes: &[u8],
    verified_imports: &[VerifiedImportRef],
    workspace_root: &Path,
) -> Phase9AiEndpointResponse {
    match validate_phase9_ai_common_envelope(
        request_canonical_bytes,
        verified_imports,
        workspace_root,
        Phase9AiTaskKind::UniverseRepair,
    ) {
        Ok(validated) => run_phase9_universe_repair_validated(validated, verified_imports),
        Err(response) => response,
    }
}

pub fn run_phase9_typeclass_resolve_request(
    request_canonical_bytes: &[u8],
    verified_imports: &[VerifiedImportRef],
    workspace_root: &Path,
) -> Phase9AiEndpointResponse {
    run_phase9_skeleton_request(
        request_canonical_bytes,
        verified_imports,
        workspace_root,
        Phase9AiTaskKind::TypeclassResolution,
    )
}

pub fn run_phase9_quotient_check_request(
    request_canonical_bytes: &[u8],
    verified_imports: &[VerifiedImportRef],
    workspace_root: &Path,
) -> Phase9AiEndpointResponse {
    run_phase9_skeleton_request(
        request_canonical_bytes,
        verified_imports,
        workspace_root,
        Phase9AiTaskKind::QuotientConstruction,
    )
}

pub fn run_phase9_smt_reconstruct_request(
    request_canonical_bytes: &[u8],
    verified_imports: &[VerifiedImportRef],
    workspace_root: &Path,
) -> Phase9AiEndpointResponse {
    run_phase9_skeleton_request(
        request_canonical_bytes,
        verified_imports,
        workspace_root,
        Phase9AiTaskKind::SmtCertificate,
    )
}

pub fn run_phase9_theorem_graph_query_request(
    request_canonical_bytes: &[u8],
    verified_imports: &[VerifiedImportRef],
    workspace_root: &Path,
) -> Phase9AiEndpointResponse {
    match validate_phase9_ai_common_envelope(
        request_canonical_bytes,
        verified_imports,
        workspace_root,
        Phase9AiTaskKind::TheoremGraphQuery,
    ) {
        Ok(validated) => {
            run_phase9_theorem_graph_query_validated(validated, verified_imports, workspace_root)
        }
        Err(response) => response,
    }
}

pub fn run_phase9_formalize_check_request(
    request_canonical_bytes: &[u8],
    verified_imports: &[VerifiedImportRef],
    workspace_root: &Path,
) -> Phase9AiEndpointResponse {
    run_phase9_skeleton_request(
        request_canonical_bytes,
        verified_imports,
        workspace_root,
        Phase9AiTaskKind::NaturalLanguageFormalization,
    )
}

fn run_phase9_skeleton_request(
    request_canonical_bytes: &[u8],
    verified_imports: &[VerifiedImportRef],
    workspace_root: &Path,
    expected_task_kind: Phase9AiTaskKind,
) -> Phase9AiEndpointResponse {
    match validate_phase9_ai_common_envelope(
        request_canonical_bytes,
        verified_imports,
        workspace_root,
        expected_task_kind,
    ) {
        Ok(validated) => rejected_response(
            validated.candidate_hash,
            Phase9AiValidationError::UnsupportedFeature,
            None,
        ),
        Err(response) => response,
    }
}

fn run_phase9_inductive_validated(
    validated: Phase9ValidatedCommonEnvelope,
    verified_imports: &[VerifiedImportRef],
) -> Phase9AiEndpointResponse {
    let candidate_hash = validated.candidate_hash;
    let proposal = match decode_inductive_proposal(&validated.envelope.payload) {
        Ok(proposal) => proposal,
        Err(_) => {
            return rejected_response(
                candidate_hash,
                Phase9AiValidationError::EnvelopeMalformed,
                None,
            );
        }
    };

    let [family] = proposal.inductives.as_slice() else {
        return rejected_response(
            candidate_hash,
            Phase9AiValidationError::UnsupportedFeature,
            Some(Phase9AiFeatureError::AdvancedInductive(
                Phase9AdvancedInductiveError::PositivityProfileUnsupported,
            )),
        );
    };

    let family_public_name = phase9_family_public_name(proposal.block_name.as_ref(), &family.name);
    let constructor_public_names = family
        .constructors
        .iter()
        .map(|constructor| phase9_append_name(&family_public_name, &constructor.name))
        .collect::<Vec<_>>();
    let recursor_public_name = phase9_append_name(&family_public_name, &Name::from_dotted("rec"));
    if phase9_inductive_names_collide(
        family,
        &family_public_name,
        &constructor_public_names,
        &recursor_public_name,
    ) {
        return rejected_response(
            candidate_hash,
            Phase9AiValidationError::FeatureRejected,
            Some(Phase9AiFeatureError::AdvancedInductive(
                Phase9AdvancedInductiveError::NameCollision,
            )),
        );
    }

    if !phase9_string_list_is_unique(&proposal.universe_params)
        || !level_is_in_scope(&family.result_sort, &proposal.universe_params)
        || !phase9_inductive_family_levels_are_in_scope(family, &proposal.universe_params)
    {
        return rejected_response(
            candidate_hash,
            Phase9AiValidationError::EnvelopeMalformed,
            None,
        );
    }

    if phase9_telescope_contains_const_name(&family.params, &family_public_name.as_dotted())
        || phase9_telescope_contains_const_name(&family.indices, &family_public_name.as_dotted())
    {
        return rejected_response(
            candidate_hash,
            Phase9AiValidationError::FeatureRejected,
            Some(Phase9AiFeatureError::AdvancedInductive(
                Phase9AdvancedInductiveError::TargetRefMismatch,
            )),
        );
    }
    if !phase9_telescope_imported_refs_are_resolved(
        &family.params,
        verified_imports,
        &BTreeSet::new(),
    ) || !phase9_telescope_imported_refs_are_resolved(
        &family.indices,
        verified_imports,
        &BTreeSet::new(),
    ) {
        return rejected_response(
            candidate_hash,
            Phase9AiValidationError::ImportClosureMismatch,
            None,
        );
    }

    let env = match phase9_kernel_env_from_imports(verified_imports) {
        Ok(env) => env,
        Err(_) => {
            return rejected_response(
                candidate_hash,
                Phase9AiValidationError::KernelRejected,
                None,
            );
        }
    };
    if phase9_check_telescope_kernel(
        &env,
        &proposal.universe_params,
        family.params.iter().chain(&family.indices),
    )
    .is_err()
    {
        return rejected_response(
            candidate_hash,
            Phase9AiValidationError::KernelRejected,
            None,
        );
    }

    let generated_names = constructor_public_names
        .iter()
        .chain(std::iter::once(&recursor_public_name))
        .map(Name::as_dotted)
        .collect::<BTreeSet<_>>();
    if family.constructors.iter().any(|constructor| {
        generated_names
            .iter()
            .any(|name| expr_contains_const_name(&constructor.ty, name))
    }) {
        return rejected_response(
            candidate_hash,
            Phase9AiValidationError::FeatureRejected,
            Some(Phase9AiFeatureError::AdvancedInductive(
                Phase9AdvancedInductiveError::TargetRefMismatch,
            )),
        );
    }
    let mut allowed_local_names = BTreeSet::new();
    allowed_local_names.insert(family_public_name.as_dotted());
    if !family.constructors.iter().all(|constructor| {
        expr_imported_refs_are_resolved_with_allowed_locals(
            &constructor.ty,
            verified_imports,
            &allowed_local_names,
        )
    }) {
        return rejected_response(
            candidate_hash,
            Phase9AiValidationError::ImportClosureMismatch,
            None,
        );
    }

    let base_decl = phase9_base_inductive_decl(
        &proposal,
        family,
        &family_public_name,
        &constructor_public_names,
    );
    let mut constructor_env = env.clone();
    if constructor_env
        .add_axiom(
            base_decl.name.clone(),
            base_decl.universe_params.clone(),
            phase9_inductive_type(&base_decl),
        )
        .is_err()
    {
        return rejected_response(
            candidate_hash,
            Phase9AiValidationError::FeatureRejected,
            Some(Phase9AiFeatureError::AdvancedInductive(
                Phase9AdvancedInductiveError::NameCollision,
            )),
        );
    }
    for constructor in &base_decl.constructors {
        if expect_sort_public(
            &constructor_env,
            &Ctx::new(),
            &proposal.universe_params,
            &constructor.ty,
        )
        .is_err()
        {
            return rejected_response(
                candidate_hash,
                Phase9AiValidationError::KernelRejected,
                None,
            );
        }
    }

    for constructor in &base_decl.constructors {
        match phase9_check_constructor_result(&constructor_env, &base_decl, constructor) {
            Ok(()) => {}
            Err(Phase9InductiveCheckError::TargetRefMismatch) => {
                return rejected_response(
                    candidate_hash,
                    Phase9AiValidationError::FeatureRejected,
                    Some(Phase9AiFeatureError::AdvancedInductive(
                        Phase9AdvancedInductiveError::TargetRefMismatch,
                    )),
                );
            }
            Err(Phase9InductiveCheckError::KernelRejected) => {
                return rejected_response(
                    candidate_hash,
                    Phase9AiValidationError::KernelRejected,
                    None,
                );
            }
            Err(Phase9InductiveCheckError::UnsupportedPositivity) => {
                return rejected_response(
                    candidate_hash,
                    Phase9AiValidationError::UnsupportedFeature,
                    Some(Phase9AiFeatureError::AdvancedInductive(
                        Phase9AdvancedInductiveError::PositivityProfileUnsupported,
                    )),
                );
            }
        }
    }

    for constructor in &base_decl.constructors {
        match phase9_check_constructor_positivity(&base_decl, constructor) {
            Ok(()) => {}
            Err(Phase9InductiveCheckError::TargetRefMismatch) => {
                return rejected_response(
                    candidate_hash,
                    Phase9AiValidationError::FeatureRejected,
                    Some(Phase9AiFeatureError::AdvancedInductive(
                        Phase9AdvancedInductiveError::TargetRefMismatch,
                    )),
                );
            }
            Err(Phase9InductiveCheckError::UnsupportedPositivity) => {
                return rejected_response(
                    candidate_hash,
                    Phase9AiValidationError::UnsupportedFeature,
                    Some(Phase9AiFeatureError::AdvancedInductive(
                        Phase9AdvancedInductiveError::PositivityProfileUnsupported,
                    )),
                );
            }
            Err(Phase9InductiveCheckError::KernelRejected) => {
                return rejected_response(
                    candidate_hash,
                    Phase9AiValidationError::KernelRejected,
                    None,
                );
            }
        }
    }

    if npa_cert::classify_inductive_artifact_profile_v1(&base_decl)
        != InductiveArtifactProfileCheckV1::SupportedMvpRecursor
    {
        return rejected_response(
            candidate_hash,
            Phase9AiValidationError::UnsupportedFeature,
            Some(Phase9AiFeatureError::AdvancedInductive(
                Phase9AdvancedInductiveError::PositivityProfileUnsupported,
            )),
        );
    }
    let final_decl = match npa_cert::generate_inductive_artifacts_v1(&base_decl) {
        Ok(final_decl) => final_decl,
        Err(_) => {
            return Phase9AiEndpointResponse::Error {
                error: Phase9AiEndpointError::InternalValidatorFailure,
            };
        }
    };
    let cert_decl = Decl::Inductive {
        name: final_decl.name.clone(),
        universe_params: final_decl.universe_params.clone(),
        ty: phase9_inductive_type(&final_decl),
        data: Box::new(final_decl),
    };
    let import_modules = verified_imports
        .iter()
        .map(|import| import.verified_module().clone())
        .collect::<Vec<_>>();
    let cert = match npa_cert::build_module_cert(
        CoreModule {
            name: family_public_name.clone(),
            declarations: vec![cert_decl],
        },
        &import_modules,
    ) {
        Ok(cert) => cert,
        Err(npa_cert::CertError::Kernel(_)) => {
            return rejected_response(
                candidate_hash,
                Phase9AiValidationError::KernelRejected,
                None,
            );
        }
        Err(_) => {
            return Phase9AiEndpointResponse::Error {
                error: Phase9AiEndpointError::InternalValidatorFailure,
            };
        }
    };
    let cert_bytes = match npa_cert::encode_module_cert(&cert) {
        Ok(bytes) => bytes,
        Err(_) => {
            return Phase9AiEndpointResponse::Error {
                error: Phase9AiEndpointError::InternalValidatorFailure,
            };
        }
    };
    let mut verifier_session = VerifierSession::new();
    for import in import_modules {
        verifier_session.register_verified_module(import);
    }
    if npa_cert::verify_module_cert(&cert_bytes, &mut verifier_session, &AxiomPolicy::normal())
        .is_err()
    {
        return Phase9AiEndpointResponse::Error {
            error: Phase9AiEndpointError::InternalValidatorFailure,
        };
    }
    let Some(decl) = cert.declarations.first() else {
        return Phase9AiEndpointResponse::Error {
            error: Phase9AiEndpointError::InternalValidatorFailure,
        };
    };
    if proposal
        .expected_decl_hash
        .is_some_and(|expected| expected != decl.hashes.decl_certificate_hash)
    {
        return rejected_response(
            candidate_hash,
            Phase9AiValidationError::TargetFingerprintMismatch,
            None,
        );
    }
    success_response(
        candidate_hash,
        Phase9AiSuccessPayload::AdvancedInductive {
            decl_interface_hash: decl.hashes.decl_interface_hash,
            decl_certificate_hash: decl.hashes.decl_certificate_hash,
        },
    )
}

fn run_phase9_theorem_graph_query_validated(
    validated: Phase9ValidatedCommonEnvelope,
    verified_imports: &[VerifiedImportRef],
    workspace_root: &Path,
) -> Phase9AiEndpointResponse {
    let candidate_hash = validated.candidate_hash;
    let query = match decode_theorem_graph_query(&validated.envelope.payload) {
        Ok(query) => query,
        Err(DecodeError::TheoremGraphSnapshotBytesTooLarge) => {
            return theorem_graph_rejected_response(
                candidate_hash,
                Phase9AiValidationError::EnvelopeMalformed,
                Phase9TheoremGraphError::SnapshotMalformed,
            );
        }
        Err(DecodeError::TheoremGraphQueryFeaturesBytesTooLarge) => {
            return theorem_graph_rejected_response(
                candidate_hash,
                Phase9AiValidationError::EnvelopeMalformed,
                Phase9TheoremGraphError::QueryFeaturesMalformed,
            );
        }
        Err(_) => {
            return rejected_response(
                candidate_hash,
                Phase9AiValidationError::EnvelopeMalformed,
                None,
            );
        }
    };

    if query.env_fingerprint != validated.envelope.target.env_fingerprint
        || Some(query.goal_fingerprint) != validated.envelope.target.goal_fingerprint
        || phase9_ai_goal_fingerprint(validated.env_fingerprint, &query.goal)
            != query.goal_fingerprint
    {
        return rejected_response(
            candidate_hash,
            Phase9AiValidationError::TargetFingerprintMismatch,
            None,
        );
    }

    match validate_phase9_theorem_graph_goal(&query.goal, verified_imports) {
        Ok(()) => {}
        Err(Phase9GoalValidationError::EnvelopeMalformed) => {
            return rejected_response(
                candidate_hash,
                Phase9AiValidationError::EnvelopeMalformed,
                None,
            );
        }
        Err(Phase9GoalValidationError::ImportClosureMismatch) => {
            return rejected_response(
                candidate_hash,
                Phase9AiValidationError::ImportClosureMismatch,
                None,
            );
        }
        Err(Phase9GoalValidationError::KernelRejected) => {
            return rejected_response(
                candidate_hash,
                Phase9AiValidationError::KernelRejected,
                None,
            );
        }
    }

    if query.limit > MAX_PHASE9_THEOREM_GRAPH_RESULT_LIMIT {
        return theorem_graph_rejected_response(
            candidate_hash,
            Phase9AiValidationError::EnvelopeMalformed,
            Phase9TheoremGraphError::LimitOutOfRange,
        );
    }

    let snapshot_bytes = match phase9_theorem_graph_snapshot_bytes(
        candidate_hash,
        &query.snapshot.source,
        workspace_root,
    ) {
        Ok(bytes) => bytes,
        Err(response) => return response,
    };
    let snapshot = match phase9_validate_theorem_graph_snapshot_bytes(
        candidate_hash,
        &snapshot_bytes,
        &query.snapshot,
    ) {
        Ok(snapshot) => snapshot,
        Err(response) => return response,
    };

    let feature_bytes = match phase9_theorem_graph_query_features_bytes(
        candidate_hash,
        &query.query_features,
        workspace_root,
    ) {
        Ok(bytes) => bytes,
        Err(response) => return response,
    };
    let query_features = match phase9_validate_theorem_graph_query_features_bytes(
        candidate_hash,
        &feature_bytes,
        &query,
    ) {
        Ok(query_features) => query_features,
        Err(response) => return response,
    };

    if snapshot.source_release_hash != query.snapshot.source_release_hash
        || snapshot.extractor_version != query.snapshot.extractor_version
    {
        return theorem_graph_rejected_response(
            candidate_hash,
            Phase9AiValidationError::EnvelopeMalformed,
            Phase9TheoremGraphError::SnapshotMalformed,
        );
    }
    if query_features.env_fingerprint != query.env_fingerprint
        || query_features.goal_fingerprint != query.goal_fingerprint
        || query_features.feature_schema_version
            != Phase9TheoremGraphFeatureSchemaVersion::MvpGoalFeaturesV1
    {
        return theorem_graph_rejected_response(
            candidate_hash,
            Phase9AiValidationError::EnvelopeMalformed,
            Phase9TheoremGraphError::QueryFeaturesMalformed,
        );
    }
    if !phase9_theorem_graph_features_are_well_formed(&query_features.features) {
        return theorem_graph_rejected_response(
            candidate_hash,
            Phase9AiValidationError::EnvelopeMalformed,
            Phase9TheoremGraphError::QueryFeaturesMalformed,
        );
    }
    if !phase9_theorem_graph_snapshot_is_well_formed(&snapshot) {
        return theorem_graph_rejected_response(
            candidate_hash,
            Phase9AiValidationError::EnvelopeMalformed,
            Phase9TheoremGraphError::SnapshotMalformed,
        );
    }

    let mut entries = Vec::new();
    for node in &snapshot.nodes {
        match phase9_resolve_theorem_graph_node(node, verified_imports) {
            Phase9TheoremGraphNodeResolution::Missing => {}
            Phase9TheoremGraphNodeResolution::Mismatch => {
                return theorem_graph_rejected_response(
                    candidate_hash,
                    Phase9AiValidationError::FeatureRejected,
                    Phase9TheoremGraphError::NodeResolutionMismatch,
                );
            }
            Phase9TheoremGraphNodeResolution::Resolved { eligible } => {
                if eligible && entries.len() < query.limit as usize {
                    entries.push(Phase9MachineTheoremGraphResultEntry {
                        node: node.clone(),
                        score: Phase9MachineTheoremGraphScore {
                            score_microunits: 0,
                        },
                    });
                }
            }
        }
    }

    success_response(
        candidate_hash,
        Phase9AiSuccessPayload::TheoremGraphQuery {
            result: Phase9MachineTheoremGraphResult { entries },
        },
    )
}

struct Phase9UniverseRepairCandidateOuter {
    goal: Option<Phase9AiGoal>,
    target_expr: Expr,
    instantiation_items: Vec<Vec<u8>>,
    constraint_hint_items: Vec<Vec<u8>>,
    minimization_hint: Option<Phase9UniverseMinimizationHint>,
}

fn run_phase9_universe_repair_validated(
    validated: Phase9ValidatedCommonEnvelope,
    verified_imports: &[VerifiedImportRef],
) -> Phase9AiEndpointResponse {
    let candidate_hash = validated.candidate_hash;
    let raw = match decode_universe_repair_candidate_outer(&validated.envelope.payload) {
        Ok(raw) => raw,
        Err(_) => {
            return rejected_response(
                candidate_hash,
                Phase9AiValidationError::EnvelopeMalformed,
                None,
            );
        }
    };

    if validated.envelope.target.target_decl_hash.is_some() {
        return rejected_response(
            candidate_hash,
            Phase9AiValidationError::UnsupportedFeature,
            None,
        );
    }

    let goal = match raw.goal.as_ref() {
        Some(goal) => goal,
        None => {
            return rejected_response(
                candidate_hash,
                Phase9AiValidationError::EnvelopeMalformed,
                None,
            );
        }
    };
    if !phase9_core_expr_bytes_eq(&goal.target, &raw.target_expr) {
        return rejected_response(
            candidate_hash,
            Phase9AiValidationError::TargetFingerprintMismatch,
            Some(Phase9AiFeatureError::UniverseRepair(
                Phase9UniverseRepairError::TargetFingerprintMismatch,
            )),
        );
    }
    if validated.envelope.target.goal_fingerprint
        != Some(phase9_ai_goal_fingerprint(validated.env_fingerprint, goal))
    {
        return rejected_response(
            candidate_hash,
            Phase9AiValidationError::TargetFingerprintMismatch,
            Some(Phase9AiFeatureError::UniverseRepair(
                Phase9UniverseRepairError::TargetFingerprintMismatch,
            )),
        );
    }

    if !phase9_string_list_is_unique(&goal.universe_params)
        || !expr_levels_are_in_scope(&goal.target, &goal.universe_params)
        || !goal
            .local_context
            .iter()
            .all(|local| local_decl_levels_are_in_scope(local, &goal.universe_params))
    {
        return rejected_response(
            candidate_hash,
            Phase9AiValidationError::EnvelopeMalformed,
            None,
        );
    }
    if !goal_imported_refs_are_resolved(goal, verified_imports) {
        return rejected_response(
            candidate_hash,
            Phase9AiValidationError::ImportClosureMismatch,
            None,
        );
    }
    if validate_goal_kernel(goal, verified_imports).is_err() {
        return rejected_response(
            candidate_hash,
            Phase9AiValidationError::KernelRejected,
            None,
        );
    }

    let instantiations = match decode_universe_instantiation_items(&raw.instantiation_items) {
        Ok(instantiations) => instantiations,
        Err(_) => {
            return rejected_response(
                candidate_hash,
                Phase9AiValidationError::EnvelopeMalformed,
                None,
            );
        }
    };
    if !universe_instantiations_are_strictly_sorted(&instantiations) {
        return rejected_response(
            candidate_hash,
            Phase9AiValidationError::EnvelopeMalformed,
            None,
        );
    }

    let constraint_hints = match decode_universe_constraint_hint_items(&raw.constraint_hint_items) {
        Ok(hints) => hints,
        Err(_) => {
            return rejected_response(
                candidate_hash,
                Phase9AiValidationError::EnvelopeMalformed,
                None,
            );
        }
    };
    if !universe_constraint_hints_are_strictly_sorted(&constraint_hints) {
        return rejected_response(
            candidate_hash,
            Phase9AiValidationError::EnvelopeMalformed,
            None,
        );
    }
    for hint in &constraint_hints {
        if !constraint_levels_are_in_scope(&hint.constraint, &goal.universe_params) {
            return rejected_response(
                candidate_hash,
                Phase9AiValidationError::FeatureRejected,
                Some(Phase9AiFeatureError::UniverseRepair(
                    Phase9UniverseRepairError::UnknownUniverseParam,
                )),
            );
        }
    }

    let mut repaired_expr = raw.target_expr.clone();
    for patch in &instantiations {
        let reached = match expr_at_path(&raw.target_expr, &patch.occurrence.path) {
            Some(reached) => reached,
            None => {
                return rejected_response(
                    candidate_hash,
                    Phase9AiValidationError::FeatureRejected,
                    Some(Phase9AiFeatureError::UniverseRepair(
                        Phase9UniverseRepairError::InvalidOccurrencePath,
                    )),
                );
            }
        };
        let Expr::Const { name, .. } = reached else {
            return rejected_response(
                candidate_hash,
                Phase9AiValidationError::FeatureRejected,
                Some(Phase9AiFeatureError::UniverseRepair(
                    Phase9UniverseRepairError::InvalidOccurrencePath,
                )),
            );
        };
        let resolved =
            match resolve_phase9_global_ref(&patch.occurrence.expected_ref, verified_imports) {
                Some(resolved) => resolved,
                None => {
                    return rejected_response(
                        candidate_hash,
                        Phase9AiValidationError::ImportClosureMismatch,
                        None,
                    );
                }
            };
        if name != &resolved.const_name {
            return rejected_response(
                candidate_hash,
                Phase9AiValidationError::FeatureRejected,
                Some(Phase9AiFeatureError::UniverseRepair(
                    Phase9UniverseRepairError::TargetRefMismatch,
                )),
            );
        }
        if patch.explicit_level_args.len() != resolved.universe_arity {
            return rejected_response(
                candidate_hash,
                Phase9AiValidationError::FeatureRejected,
                Some(Phase9AiFeatureError::UniverseRepair(
                    Phase9UniverseRepairError::IllFormedLevelExpr,
                )),
            );
        }
        for level in &patch.explicit_level_args {
            if !level_is_in_scope(level, &goal.universe_params) {
                return rejected_response(
                    candidate_hash,
                    Phase9AiValidationError::FeatureRejected,
                    Some(Phase9AiFeatureError::UniverseRepair(
                        Phase9UniverseRepairError::UnknownUniverseParam,
                    )),
                );
            }
        }
        if replace_const_levels_at_path(
            &mut repaired_expr,
            &patch.occurrence.path,
            patch.explicit_level_args.clone(),
        )
        .is_none()
        {
            return Phase9AiEndpointResponse::Error {
                error: Phase9AiEndpointError::InternalValidatorFailure,
            };
        }
    }

    let constraints = match derive_universe_constraints(goal, &repaired_expr, verified_imports) {
        Ok(constraints) => constraints,
        Err(_) => {
            return rejected_response(
                candidate_hash,
                Phase9AiValidationError::NoSolution,
                Some(Phase9AiFeatureError::UniverseRepair(
                    Phase9UniverseRepairError::UnsatisfiedConstraint,
                )),
            );
        }
    };
    let constraint_keys = constraints
        .iter()
        .map(phase9_universe_constraint_canonical_bytes)
        .collect::<BTreeSet<_>>();
    for hint in &constraint_hints {
        let key = phase9_universe_constraint_canonical_bytes(&hint.constraint);
        if !constraint_keys.contains(&key) {
            return rejected_response(
                candidate_hash,
                Phase9AiValidationError::FeatureRejected,
                Some(Phase9AiFeatureError::UniverseRepair(
                    Phase9UniverseRepairError::ConstraintHintMismatch,
                )),
            );
        }
    }
    if constraints
        .iter()
        .any(|constraint| !universe_constraint_is_satisfiable(constraint))
    {
        return rejected_response(
            candidate_hash,
            Phase9AiValidationError::NoSolution,
            Some(Phase9AiFeatureError::UniverseRepair(
                Phase9UniverseRepairError::UnsatisfiedConstraint,
            )),
        );
    }
    let constraint_set_hash = phase9_universe_constraint_set_hash(&constraints);
    let success = Phase9AiSuccessPayload::UniverseRepair {
        repaired_expr,
        constraint_set_hash,
    };
    success_response(candidate_hash, success)
}

fn success_response(
    candidate_hash: Hash,
    payload: Phase9AiSuccessPayload,
) -> Phase9AiEndpointResponse {
    let validation_result_hash =
        phase9_ai_validation_result_hash_for_success(candidate_hash, &payload);
    Phase9AiEndpointResponse::Success {
        candidate_hash,
        validation_result_hash,
        payload: Box::new(payload),
    }
}

fn validate_imports(
    candidate_hash: Hash,
    imports: &[Phase9ImportIdentity],
    verified_imports: &[VerifiedImportRef],
) -> std::result::Result<(), Phase9AiEndpointResponse> {
    let mut previous: Option<&Phase9ImportIdentity> = None;
    for import in imports {
        if !import.module.is_canonical() {
            return Err(rejected_response(
                candidate_hash,
                Phase9AiValidationError::EnvelopeMalformed,
                None,
            ));
        }
        if let Some(previous) = previous {
            match compare_import_identities(previous, import) {
                Ok(Ordering::Greater) => {
                    return Err(rejected_response(
                        candidate_hash,
                        Phase9AiValidationError::EnvelopeMalformed,
                        None,
                    ));
                }
                Ok(Ordering::Equal) => {
                    return Err(rejected_response(
                        candidate_hash,
                        Phase9AiValidationError::ImportClosureMismatch,
                        None,
                    ));
                }
                Ok(Ordering::Less) => {}
                Err(_) => {
                    return Err(rejected_response(
                        candidate_hash,
                        Phase9AiValidationError::EnvelopeMalformed,
                        None,
                    ));
                }
            }
        }
        previous = Some(import);
    }

    if imports.len() != verified_imports.len() {
        return Err(rejected_response(
            candidate_hash,
            Phase9AiValidationError::ImportClosureMismatch,
            None,
        ));
    }

    for (expected, actual) in imports.iter().zip(verified_imports) {
        let actual = Phase9ImportIdentity::from_verified_import(actual);
        if expected != &actual {
            return Err(rejected_response(
                candidate_hash,
                Phase9AiValidationError::ImportClosureMismatch,
                None,
            ));
        }
    }

    Ok(())
}

fn validate_options_ref(
    candidate_hash: Hash,
    options_ref: &Phase9AiOptionsRef,
    workspace_root: &Path,
) -> std::result::Result<(Phase9AiOptions, Hash), Phase9AiEndpointResponse> {
    let (declared_options_hash, canonical_bytes) = match options_ref {
        Phase9AiOptionsRef::Inline {
            options_hash,
            canonical_bytes,
        } => {
            if canonical_bytes.len() > MAX_OPTIONS_BYTES {
                return Err(rejected_response(
                    candidate_hash,
                    Phase9AiValidationError::EnvelopeMalformed,
                    None,
                ));
            }
            (*options_hash, canonical_bytes.clone())
        }
        Phase9AiOptionsRef::Artifact {
            path,
            file_hash,
            options_hash,
            size_bytes,
        } => {
            if usize::try_from(*size_bytes)
                .map(|size| size > MAX_OPTIONS_BYTES)
                .unwrap_or(true)
            {
                return Err(rejected_response(
                    candidate_hash,
                    Phase9AiValidationError::EnvelopeMalformed,
                    None,
                ));
            }
            let path = match validate_artifact_path(workspace_root, path) {
                Ok(path) => path,
                Err(ArtifactPathError::EnvelopeMalformed) => {
                    return Err(rejected_response(
                        candidate_hash,
                        Phase9AiValidationError::EnvelopeMalformed,
                        None,
                    ));
                }
                Err(ArtifactPathError::ArtifactUnavailable) => {
                    return Err(Phase9AiEndpointResponse::Error {
                        error: Phase9AiEndpointError::ArtifactUnavailable,
                    });
                }
            };
            let bytes = std::fs::read(path).map_err(|_| Phase9AiEndpointResponse::Error {
                error: Phase9AiEndpointError::ArtifactUnavailable,
            })?;
            if bytes.len() as u64 != *size_bytes || phase9_file_hash(&bytes) != *file_hash {
                return Err(rejected_response(
                    candidate_hash,
                    Phase9AiValidationError::PayloadHashMismatch,
                    None,
                ));
            }
            (*options_hash, bytes)
        }
    };

    let options = decode_options(&canonical_bytes).map_err(|_| {
        rejected_response(
            candidate_hash,
            Phase9AiValidationError::EnvelopeMalformed,
            None,
        )
    })?;
    let actual_options_hash = phase9_ai_options_hash(&canonical_bytes);
    if actual_options_hash != declared_options_hash {
        return Err(rejected_response(
            candidate_hash,
            Phase9AiValidationError::PayloadHashMismatch,
            None,
        ));
    }

    Ok((options, actual_options_hash))
}

fn validate_target_shape(
    candidate_hash: Hash,
    task_kind: Phase9AiTaskKind,
    target: &Phase9AiTarget,
) -> std::result::Result<(), Phase9AiEndpointResponse> {
    let valid = match task_kind {
        Phase9AiTaskKind::AdvancedInductive
        | Phase9AiTaskKind::QuotientConstruction
        | Phase9AiTaskKind::NaturalLanguageFormalization => {
            target.target_decl_hash.is_none() && target.goal_fingerprint.is_none()
        }
        Phase9AiTaskKind::UniverseRepair => {
            (target.target_decl_hash.is_none() && target.goal_fingerprint.is_some())
                || (target.target_decl_hash.is_some() && target.goal_fingerprint.is_none())
        }
        Phase9AiTaskKind::TypeclassResolution
        | Phase9AiTaskKind::SmtCertificate
        | Phase9AiTaskKind::TheoremGraphQuery => {
            target.target_decl_hash.is_none() && target.goal_fingerprint.is_some()
        }
    };
    if valid {
        Ok(())
    } else {
        Err(rejected_response(
            candidate_hash,
            Phase9AiValidationError::EnvelopeMalformed,
            None,
        ))
    }
}

fn validate_required_options(
    candidate_hash: Hash,
    task_kind: Phase9AiTaskKind,
    options: &Phase9AiOptions,
) -> std::result::Result<(), Phase9AiEndpointResponse> {
    let valid = match task_kind {
        Phase9AiTaskKind::QuotientConstruction => options.quotient.is_some(),
        Phase9AiTaskKind::SmtCertificate => options.smt.is_some(),
        Phase9AiTaskKind::NaturalLanguageFormalization => options.formalization.is_some(),
        Phase9AiTaskKind::AdvancedInductive
        | Phase9AiTaskKind::UniverseRepair
        | Phase9AiTaskKind::TypeclassResolution
        | Phase9AiTaskKind::TheoremGraphQuery => true,
    };
    if valid {
        Ok(())
    } else {
        Err(rejected_response(
            candidate_hash,
            Phase9AiValidationError::EnvelopeMalformed,
            None,
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase9GoalValidationError {
    EnvelopeMalformed,
    ImportClosureMismatch,
    KernelRejected,
}

fn validate_phase9_theorem_graph_goal(
    goal: &Phase9AiGoal,
    verified_imports: &[VerifiedImportRef],
) -> std::result::Result<(), Phase9GoalValidationError> {
    if !phase9_string_list_is_unique(&goal.universe_params)
        || !expr_levels_are_in_scope(&goal.target, &goal.universe_params)
        || !goal
            .local_context
            .iter()
            .all(|local| local_decl_levels_are_in_scope(local, &goal.universe_params))
    {
        return Err(Phase9GoalValidationError::EnvelopeMalformed);
    }
    if !goal_imported_refs_are_resolved(goal, verified_imports) {
        return Err(Phase9GoalValidationError::ImportClosureMismatch);
    }
    validate_goal_kernel(goal, verified_imports)
        .map_err(|_| Phase9GoalValidationError::KernelRejected)
}

fn theorem_graph_rejected_response(
    candidate_hash: Hash,
    error: Phase9AiValidationError,
    graph_error: Phase9TheoremGraphError,
) -> Phase9AiEndpointResponse {
    rejected_response(
        candidate_hash,
        error,
        Some(Phase9AiFeatureError::TheoremGraphQuery(graph_error)),
    )
}

fn phase9_theorem_graph_snapshot_bytes(
    candidate_hash: Hash,
    source: &Phase9MachineTheoremGraphSnapshotSource,
    workspace_root: &Path,
) -> std::result::Result<Vec<u8>, Phase9AiEndpointResponse> {
    match source {
        Phase9MachineTheoremGraphSnapshotSource::Inline {
            canonical_bytes, ..
        } => {
            if canonical_bytes.len() > MAX_PHASE9_THEOREM_GRAPH_SNAPSHOT_BYTES {
                return Err(theorem_graph_rejected_response(
                    candidate_hash,
                    Phase9AiValidationError::EnvelopeMalformed,
                    Phase9TheoremGraphError::SnapshotMalformed,
                ));
            }
            Ok(canonical_bytes.clone())
        }
        Phase9MachineTheoremGraphSnapshotSource::Artifact {
            path,
            file_hash,
            size_bytes,
            ..
        } => phase9_theorem_graph_artifact_bytes(
            candidate_hash,
            workspace_root,
            path,
            *file_hash,
            *size_bytes,
            MAX_PHASE9_THEOREM_GRAPH_SNAPSHOT_BYTES,
            Phase9TheoremGraphError::SnapshotMalformed,
        ),
    }
}

fn phase9_theorem_graph_query_features_bytes(
    candidate_hash: Hash,
    source: &Phase9MachineTheoremGraphQueryFeaturesRef,
    workspace_root: &Path,
) -> std::result::Result<Vec<u8>, Phase9AiEndpointResponse> {
    match source {
        Phase9MachineTheoremGraphQueryFeaturesRef::Inline {
            canonical_bytes, ..
        } => {
            if canonical_bytes.len() > MAX_PHASE9_THEOREM_GRAPH_QUERY_FEATURES_BYTES {
                return Err(theorem_graph_rejected_response(
                    candidate_hash,
                    Phase9AiValidationError::EnvelopeMalformed,
                    Phase9TheoremGraphError::QueryFeaturesMalformed,
                ));
            }
            Ok(canonical_bytes.clone())
        }
        Phase9MachineTheoremGraphQueryFeaturesRef::Artifact {
            path,
            file_hash,
            size_bytes,
            ..
        } => phase9_theorem_graph_artifact_bytes(
            candidate_hash,
            workspace_root,
            path,
            *file_hash,
            *size_bytes,
            MAX_PHASE9_THEOREM_GRAPH_QUERY_FEATURES_BYTES,
            Phase9TheoremGraphError::QueryFeaturesMalformed,
        ),
    }
}

fn phase9_theorem_graph_artifact_bytes(
    candidate_hash: Hash,
    workspace_root: &Path,
    path: &str,
    file_hash: Hash,
    size_bytes: u64,
    max_bytes: usize,
    malformed_error: Phase9TheoremGraphError,
) -> std::result::Result<Vec<u8>, Phase9AiEndpointResponse> {
    if usize::try_from(size_bytes)
        .map(|size| size > max_bytes)
        .unwrap_or(true)
    {
        return Err(theorem_graph_rejected_response(
            candidate_hash,
            Phase9AiValidationError::EnvelopeMalformed,
            malformed_error,
        ));
    }
    let path = match validate_artifact_path(workspace_root, path) {
        Ok(path) => path,
        Err(ArtifactPathError::EnvelopeMalformed) => {
            return Err(theorem_graph_rejected_response(
                candidate_hash,
                Phase9AiValidationError::EnvelopeMalformed,
                malformed_error,
            ));
        }
        Err(ArtifactPathError::ArtifactUnavailable) => {
            return Err(Phase9AiEndpointResponse::Error {
                error: Phase9AiEndpointError::ArtifactUnavailable,
            });
        }
    };
    let metadata = std::fs::metadata(&path).map_err(|_| Phase9AiEndpointResponse::Error {
        error: Phase9AiEndpointError::ArtifactUnavailable,
    })?;
    if metadata.len() != size_bytes {
        return Err(rejected_response(
            candidate_hash,
            Phase9AiValidationError::PayloadHashMismatch,
            None,
        ));
    }
    let bytes = std::fs::read(path).map_err(|_| Phase9AiEndpointResponse::Error {
        error: Phase9AiEndpointError::ArtifactUnavailable,
    })?;
    if phase9_file_hash(&bytes) != file_hash {
        return Err(rejected_response(
            candidate_hash,
            Phase9AiValidationError::PayloadHashMismatch,
            None,
        ));
    }
    Ok(bytes)
}

fn phase9_validate_theorem_graph_snapshot_bytes(
    candidate_hash: Hash,
    bytes: &[u8],
    snapshot_ref: &Phase9MachineTheoremGraphSnapshotRef,
) -> std::result::Result<Phase9MachineTheoremGraphSnapshot, Phase9AiEndpointResponse> {
    phase9_precheck_theorem_graph_snapshot_outer(bytes).map_err(|_| {
        theorem_graph_rejected_response(
            candidate_hash,
            Phase9AiValidationError::EnvelopeMalformed,
            Phase9TheoremGraphError::SnapshotMalformed,
        )
    })?;
    let expected_hash = match &snapshot_ref.source {
        Phase9MachineTheoremGraphSnapshotSource::Inline {
            graph_snapshot_hash,
            ..
        }
        | Phase9MachineTheoremGraphSnapshotSource::Artifact {
            graph_snapshot_hash,
            ..
        } => *graph_snapshot_hash,
    };
    if hash_with_domain(THEOREM_GRAPH_SNAPSHOT_HASH_TAG, bytes) != expected_hash {
        return Err(rejected_response(
            candidate_hash,
            Phase9AiValidationError::PayloadHashMismatch,
            None,
        ));
    }
    let snapshot = decode_theorem_graph_snapshot(bytes).map_err(|_| {
        theorem_graph_rejected_response(
            candidate_hash,
            Phase9AiValidationError::EnvelopeMalformed,
            Phase9TheoremGraphError::SnapshotMalformed,
        )
    })?;
    Ok(snapshot)
}

fn phase9_validate_theorem_graph_query_features_bytes(
    candidate_hash: Hash,
    bytes: &[u8],
    query: &Phase9MachineTheoremGraphQuery,
) -> std::result::Result<Phase9MachineTheoremGraphQueryFeatures, Phase9AiEndpointResponse> {
    phase9_precheck_theorem_graph_query_features_outer(bytes).map_err(|_| {
        theorem_graph_rejected_response(
            candidate_hash,
            Phase9AiValidationError::EnvelopeMalformed,
            Phase9TheoremGraphError::QueryFeaturesMalformed,
        )
    })?;
    let expected_hash = match &query.query_features {
        Phase9MachineTheoremGraphQueryFeaturesRef::Inline {
            query_features_hash,
            ..
        }
        | Phase9MachineTheoremGraphQueryFeaturesRef::Artifact {
            query_features_hash,
            ..
        } => *query_features_hash,
    };
    if hash_with_domain(THEOREM_GRAPH_QUERY_FEATURES_HASH_TAG, bytes) != expected_hash {
        return Err(rejected_response(
            candidate_hash,
            Phase9AiValidationError::PayloadHashMismatch,
            None,
        ));
    }
    let query_features = decode_theorem_graph_query_features(bytes).map_err(|_| {
        theorem_graph_rejected_response(
            candidate_hash,
            Phase9AiValidationError::EnvelopeMalformed,
            Phase9TheoremGraphError::QueryFeaturesMalformed,
        )
    })?;
    Ok(query_features)
}

fn phase9_precheck_theorem_graph_snapshot_outer(
    bytes: &[u8],
) -> std::result::Result<(), DecodeError> {
    let mut decoder = Decoder::new(bytes);
    decoder.hash()?;
    Phase9TheoremGraphExtractorVersion::from_tag(decoder.u8()?).ok_or(DecodeError::Malformed)?;
    let node_len = decoder.u64()?;
    if node_len > MAX_PHASE9_THEOREM_GRAPH_NODES {
        return Err(DecodeError::Malformed);
    }
    for _ in 0..node_len {
        decoder.skip_theorem_graph_node()?;
    }
    let edge_len = decoder.u64()?;
    if edge_len > MAX_PHASE9_THEOREM_GRAPH_EDGES {
        return Err(DecodeError::Malformed);
    }
    for _ in 0..edge_len {
        decoder.skip_theorem_graph_edge()?;
    }
    decoder.done()
}

fn phase9_precheck_theorem_graph_query_features_outer(
    bytes: &[u8],
) -> std::result::Result<(), DecodeError> {
    let mut decoder = Decoder::new(bytes);
    decoder.hash()?;
    decoder.hash()?;
    Phase9TheoremGraphFeatureSchemaVersion::from_tag(decoder.u8()?)
        .ok_or(DecodeError::Malformed)?;
    let feature_len = decoder.u64()?;
    if feature_len > MAX_PHASE9_THEOREM_GRAPH_FEATURES {
        return Err(DecodeError::Malformed);
    }
    for _ in 0..feature_len {
        decoder.skip_theorem_graph_feature()?;
    }
    decoder.done()
}

fn phase9_theorem_graph_features_are_well_formed(
    features: &[Phase9MachineTheoremGraphFeature],
) -> bool {
    let mut previous = None;
    for feature in features {
        if !phase9_theorem_graph_feature_key_is_valid(&feature.key) {
            return false;
        }
        let key = phase9_theorem_graph_feature_key_canonical_bytes(&feature.key);
        if previous.as_ref().is_some_and(|previous| previous >= &key) {
            return false;
        }
        previous = Some(key);
    }
    true
}

fn phase9_theorem_graph_feature_key_is_valid(key: &Phase9TheoremGraphFeatureKey) -> bool {
    phase9_theorem_graph_feature_key_component_is_valid(&key.namespace_ascii)
        && phase9_theorem_graph_feature_key_component_is_valid(&key.name_ascii)
}

fn phase9_theorem_graph_feature_key_component_is_valid(bytes: &[u8]) -> bool {
    if bytes.is_empty() || bytes.len() > 64 {
        return false;
    }
    let Some(first) = bytes.first().copied() else {
        return false;
    };
    if !first.is_ascii_alphabetic() && first != b'_' {
        return false;
    }
    bytes[1..]
        .iter()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(*byte, b'_' | b'.' | b':' | b'-'))
}

fn phase9_theorem_graph_feature_key_canonical_bytes(key: &Phase9TheoremGraphFeatureKey) -> Vec<u8> {
    let mut out = Vec::new();
    encode_bytes_to(&mut out, &key.namespace_ascii);
    encode_bytes_to(&mut out, &key.name_ascii);
    out
}

fn phase9_theorem_graph_snapshot_is_well_formed(
    snapshot: &Phase9MachineTheoremGraphSnapshot,
) -> bool {
    let mut previous_node = None;
    let mut node_bytes = BTreeSet::new();
    for node in &snapshot.nodes {
        let identity = phase9_theorem_graph_node_identity_key(node);
        if previous_node
            .as_ref()
            .is_some_and(|previous| previous >= &identity)
        {
            return false;
        }
        previous_node = Some(identity);
        let Ok(bytes) = phase9_theorem_graph_node_canonical_bytes(node) else {
            return false;
        };
        node_bytes.insert(bytes);
    }

    let mut previous_edge = None;
    for edge in &snapshot.edges {
        let key = phase9_theorem_graph_edge_key(edge);
        if previous_edge
            .as_ref()
            .is_some_and(|previous| previous >= &key)
        {
            return false;
        }
        previous_edge = Some(key);

        let Ok(from) = phase9_theorem_graph_node_canonical_bytes(&edge.from) else {
            return false;
        };
        let Ok(to) = phase9_theorem_graph_node_canonical_bytes(&edge.to) else {
            return false;
        };
        if !node_bytes.contains(&from) || !node_bytes.contains(&to) {
            return false;
        }
    }
    true
}

fn phase9_theorem_graph_node_identity_key(node: &Phase9MachineTheoremGraphNodeRef) -> Vec<u8> {
    let mut out = Vec::new();
    encode_name_to(&mut out, &node.module).expect("decoded theorem graph module is canonical");
    encode_name_to(&mut out, &node.name).expect("decoded theorem graph name is canonical");
    encode_hash_to(&mut out, &node.export_hash);
    encode_hash_to(&mut out, &node.certificate_hash);
    encode_hash_to(&mut out, &node.decl_interface_hash);
    out
}

fn phase9_theorem_graph_edge_key(edge: &Phase9MachineTheoremGraphEdge) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&phase9_theorem_graph_node_identity_key(&edge.from));
    out.extend_from_slice(&phase9_theorem_graph_node_identity_key(&edge.to));
    out.push(edge.kind.tag());
    out
}

enum Phase9TheoremGraphNodeResolution {
    Missing,
    Mismatch,
    Resolved { eligible: bool },
}

fn phase9_resolve_theorem_graph_node(
    node: &Phase9MachineTheoremGraphNodeRef,
    imports: &[VerifiedImportRef],
) -> Phase9TheoremGraphNodeResolution {
    let Some(import) = imports.iter().find(|import| {
        import.module() == &node.module
            && import.export_hash() == node.export_hash
            && import.certificate_hash() == node.certificate_hash
    }) else {
        return Phase9TheoremGraphNodeResolution::Missing;
    };

    let matches = import
        .exports()
        .iter()
        .filter(|export| {
            export.name == node.name && export.decl_interface_hash == node.decl_interface_hash
        })
        .collect::<Vec<_>>();
    let [export] = matches.as_slice() else {
        return Phase9TheoremGraphNodeResolution::Missing;
    };
    if export.type_hash != node.type_hash {
        return Phase9TheoremGraphNodeResolution::Mismatch;
    }
    let Some(decl) = import
        .verified_module()
        .declarations()
        .iter()
        .find(|decl| decl.hashes.decl_interface_hash == export.decl_interface_hash)
    else {
        return Phase9TheoremGraphNodeResolution::Mismatch;
    };
    if decl.hashes.decl_certificate_hash != node.decl_certificate_hash {
        return Phase9TheoremGraphNodeResolution::Mismatch;
    }
    Phase9TheoremGraphNodeResolution::Resolved {
        eligible: matches!(export.kind, ExportKind::Axiom | ExportKind::Theorem),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase9InductiveCheckError {
    TargetRefMismatch,
    KernelRejected,
    UnsupportedPositivity,
}

struct ResolvedPhase9GlobalRef {
    const_name: String,
    universe_arity: usize,
}

fn phase9_family_public_name(block_name: Option<&Name>, family_name: &Name) -> Name {
    match block_name {
        Some(block_name) => phase9_append_name(block_name, family_name),
        None => family_name.clone(),
    }
}

fn phase9_append_name(prefix: &Name, suffix: &Name) -> Name {
    let mut components = prefix.0.clone();
    components.extend(suffix.0.iter().cloned());
    Name(components)
}

fn phase9_inductive_names_collide(
    family: &Phase9MachineInductiveFamilyProposal,
    family_public_name: &Name,
    constructor_public_names: &[Name],
    recursor_public_name: &Name,
) -> bool {
    let mut local_names = BTreeSet::new();
    if !local_names.insert(family.name.clone()) {
        return true;
    }
    for constructor in &family.constructors {
        if !local_names.insert(constructor.name.clone()) {
            return true;
        }
    }

    let mut public_names = BTreeSet::new();
    if !public_names.insert(family_public_name.clone()) {
        return true;
    }
    for constructor_name in constructor_public_names {
        if !public_names.insert(constructor_name.clone()) {
            return true;
        }
    }
    !public_names.insert(recursor_public_name.clone())
}

fn phase9_inductive_family_levels_are_in_scope(
    family: &Phase9MachineInductiveFamilyProposal,
    params: &[String],
) -> bool {
    family
        .params
        .iter()
        .chain(&family.indices)
        .all(|binder| expr_levels_are_in_scope(&binder.ty, params))
        && family
            .constructors
            .iter()
            .all(|constructor| expr_levels_are_in_scope(&constructor.ty, params))
}

fn phase9_telescope_contains_const_name(
    telescope: &[Phase9MachineTelescopeBinder],
    name: &str,
) -> bool {
    telescope
        .iter()
        .any(|binder| expr_contains_const_name(&binder.ty, name))
}

fn expr_contains_const_name(expr: &Expr, needle: &str) -> bool {
    match expr {
        Expr::Sort(_) | Expr::BVar(_) => false,
        Expr::Const { name, .. } => name == needle,
        Expr::App(fun, arg) => {
            expr_contains_const_name(fun, needle) || expr_contains_const_name(arg, needle)
        }
        Expr::Lam { ty, body, .. } | Expr::Pi { ty, body, .. } => {
            expr_contains_const_name(ty, needle) || expr_contains_const_name(body, needle)
        }
        Expr::Let {
            ty, value, body, ..
        } => {
            expr_contains_const_name(ty, needle)
                || expr_contains_const_name(value, needle)
                || expr_contains_const_name(body, needle)
        }
    }
}

fn phase9_telescope_imported_refs_are_resolved(
    telescope: &[Phase9MachineTelescopeBinder],
    imports: &[VerifiedImportRef],
    allowed_local_names: &BTreeSet<String>,
) -> bool {
    telescope.iter().all(|binder| {
        expr_imported_refs_are_resolved_with_allowed_locals(
            &binder.ty,
            imports,
            allowed_local_names,
        )
    })
}

fn expr_imported_refs_are_resolved_with_allowed_locals(
    expr: &Expr,
    imports: &[VerifiedImportRef],
    allowed_local_names: &BTreeSet<String>,
) -> bool {
    match expr {
        Expr::Sort(_) | Expr::BVar(_) => true,
        Expr::Const { name, .. } => {
            allowed_local_names.contains(name) || const_name_is_exported_once(name, imports)
        }
        Expr::App(fun, arg) => {
            expr_imported_refs_are_resolved_with_allowed_locals(fun, imports, allowed_local_names)
                && expr_imported_refs_are_resolved_with_allowed_locals(
                    arg,
                    imports,
                    allowed_local_names,
                )
        }
        Expr::Lam { ty, body, .. } | Expr::Pi { ty, body, .. } => {
            expr_imported_refs_are_resolved_with_allowed_locals(ty, imports, allowed_local_names)
                && expr_imported_refs_are_resolved_with_allowed_locals(
                    body,
                    imports,
                    allowed_local_names,
                )
        }
        Expr::Let {
            ty, value, body, ..
        } => {
            expr_imported_refs_are_resolved_with_allowed_locals(ty, imports, allowed_local_names)
                && expr_imported_refs_are_resolved_with_allowed_locals(
                    value,
                    imports,
                    allowed_local_names,
                )
                && expr_imported_refs_are_resolved_with_allowed_locals(
                    body,
                    imports,
                    allowed_local_names,
                )
        }
    }
}

fn phase9_check_telescope_kernel<'a>(
    env: &Env,
    delta: &[String],
    telescope: impl Iterator<Item = &'a Phase9MachineTelescopeBinder>,
) -> std::result::Result<(), ()> {
    let mut ctx = Ctx::new();
    for (index, binder) in telescope.enumerate() {
        expect_sort_public(env, &ctx, delta, &binder.ty)?;
        ctx.push_assumption(format!("x{index}"), binder.ty.clone());
    }
    Ok(())
}

fn phase9_base_inductive_decl(
    proposal: &Phase9MachineInductiveProposal,
    family: &Phase9MachineInductiveFamilyProposal,
    family_public_name: &Name,
    constructor_public_names: &[Name],
) -> InductiveDecl {
    InductiveDecl::new(
        family_public_name.as_dotted(),
        proposal.universe_params.clone(),
        family
            .params
            .iter()
            .enumerate()
            .map(|(index, binder)| Binder::new(format!("p{index}"), binder.ty.clone()))
            .collect(),
        family
            .indices
            .iter()
            .enumerate()
            .map(|(index, binder)| Binder::new(format!("i{index}"), binder.ty.clone()))
            .collect(),
        family.result_sort.clone(),
        family
            .constructors
            .iter()
            .zip(constructor_public_names)
            .map(|(constructor, public_name)| {
                ConstructorDecl::new(public_name.as_dotted(), constructor.ty.clone())
            })
            .collect(),
        None,
    )
}

fn phase9_inductive_type(data: &InductiveDecl) -> Expr {
    data.params
        .iter()
        .chain(&data.indices)
        .rev()
        .fold(Expr::sort(data.sort.clone()), |body, binder| {
            Expr::pi(binder.name.clone(), binder.ty.clone(), body)
        })
}

fn phase9_check_constructor_result(
    env: &Env,
    data: &InductiveDecl,
    constructor: &ConstructorDecl,
) -> std::result::Result<(), Phase9InductiveCheckError> {
    let (domains, result) = phase9_peel_pi_domains(&constructor.ty);
    let result = env
        .whnf(&Ctx::new(), &data.universe_params, &result)
        .map_err(|_| Phase9InductiveCheckError::KernelRejected)?;
    let (head, args) = npa_kernel::expr::collect_apps(&result);
    let levels = match head {
        Expr::Const { name, levels } if name == data.name => levels,
        _ => return Err(Phase9InductiveCheckError::TargetRefMismatch),
    };
    let expected_levels = data
        .universe_params
        .iter()
        .map(|param| Level::param(param.clone()))
        .collect::<Vec<_>>();
    if !npa_kernel::level::levels_eq(&levels, &expected_levels)
        || args.len() != data.params.len() + data.indices.len()
        || domains.len() < data.params.len()
    {
        return Err(Phase9InductiveCheckError::TargetRefMismatch);
    }
    for (param_index, arg) in args.iter().take(data.params.len()).enumerate() {
        let expected = phase9_bvar_for_abs(domains.len(), param_index)
            .ok_or(Phase9InductiveCheckError::TargetRefMismatch)?;
        if arg != &expected {
            return Err(Phase9InductiveCheckError::TargetRefMismatch);
        }
    }
    Ok(())
}

fn phase9_check_constructor_positivity(
    data: &InductiveDecl,
    constructor: &ConstructorDecl,
) -> std::result::Result<(), Phase9InductiveCheckError> {
    let (domains, _) = phase9_peel_pi_domains(&constructor.ty);
    for (domain_index, domain) in domains.iter().enumerate() {
        if domain_index >= data.params.len() {
            match phase9_direct_recursive_domain_status(data, domain, domain_index)? {
                Phase9DirectRecursiveDomain::Direct => continue,
                Phase9DirectRecursiveDomain::BadTarget => {
                    return Err(Phase9InductiveCheckError::TargetRefMismatch)
                }
                Phase9DirectRecursiveDomain::NotRecursive => {}
            }
        }
        if expr_contains_const_name(domain, &data.name) {
            return Err(Phase9InductiveCheckError::UnsupportedPositivity);
        }
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase9DirectRecursiveDomain {
    Direct,
    BadTarget,
    NotRecursive,
}

fn phase9_direct_recursive_domain_status(
    data: &InductiveDecl,
    domain: &Expr,
    ctx_len: usize,
) -> std::result::Result<Phase9DirectRecursiveDomain, Phase9InductiveCheckError> {
    let (head, args) = npa_kernel::expr::collect_apps(domain);
    let levels = match head {
        Expr::Const { name, levels } if name == data.name => levels,
        _ => return Ok(Phase9DirectRecursiveDomain::NotRecursive),
    };
    let expected_levels = data
        .universe_params
        .iter()
        .map(|param| Level::param(param.clone()))
        .collect::<Vec<_>>();
    if !npa_kernel::level::levels_eq(&levels, &expected_levels)
        || args.len() != data.params.len() + data.indices.len()
    {
        return Ok(Phase9DirectRecursiveDomain::BadTarget);
    }
    for (param_index, arg) in args.iter().take(data.params.len()).enumerate() {
        let expected = phase9_bvar_for_abs(ctx_len, param_index)
            .ok_or(Phase9InductiveCheckError::TargetRefMismatch)?;
        if arg != &expected {
            return Ok(Phase9DirectRecursiveDomain::BadTarget);
        }
    }
    if args
        .iter()
        .skip(data.params.len())
        .any(|arg| expr_contains_const_name(arg, &data.name))
    {
        return Err(Phase9InductiveCheckError::UnsupportedPositivity);
    }
    Ok(Phase9DirectRecursiveDomain::Direct)
}

fn phase9_peel_pi_domains(ty: &Expr) -> (Vec<Expr>, Expr) {
    let mut domains = Vec::new();
    let mut current = ty.clone();
    while let Expr::Pi { ty, body, .. } = current {
        domains.push(*ty);
        current = *body;
    }
    (domains, current)
}

fn phase9_bvar_for_abs(ctx_len: usize, abs: usize) -> Option<Expr> {
    if abs >= ctx_len {
        return None;
    }
    Some(Expr::bvar((ctx_len - 1 - abs) as u32))
}

fn phase9_universe_constraint_set_hash(constraints: &[Phase9UniverseConstraint]) -> Hash {
    let mut canonical = constraints.to_vec();
    canonical.sort_by_key(phase9_universe_constraint_canonical_bytes);
    let mut out = Vec::new();
    encode_len_to(&mut out, canonical.len());
    for constraint in &canonical {
        encode_universe_constraint_to(&mut out, constraint);
    }
    hash_with_domain(UNIVERSE_CONSTRAINT_SET_HASH_TAG, &out)
}

fn phase9_universe_params_canonical_bytes(params: &[String]) -> Vec<u8> {
    let mut out = Vec::new();
    encode_len_to(&mut out, params.len());
    for param in params {
        encode_string_to(&mut out, param);
    }
    out
}

fn phase9_core_expr_bytes_eq(lhs: &Expr, rhs: &Expr) -> bool {
    npa_cert::core_expr_canonical_bytes(lhs) == npa_cert::core_expr_canonical_bytes(rhs)
}

fn phase9_string_list_is_unique(values: &[String]) -> bool {
    let mut seen = BTreeSet::new();
    values.iter().all(|value| seen.insert(value))
}

fn local_decl_levels_are_in_scope(local: &MachineLocalDecl, params: &[String]) -> bool {
    expr_levels_are_in_scope(&local.ty, params)
        && local
            .value
            .as_ref()
            .is_none_or(|value| expr_levels_are_in_scope(value, params))
}

fn expr_levels_are_in_scope(expr: &Expr, params: &[String]) -> bool {
    match expr {
        Expr::Sort(level) => level_is_in_scope(level, params),
        Expr::BVar(_) => true,
        Expr::Const { levels, .. } => levels.iter().all(|level| level_is_in_scope(level, params)),
        Expr::App(fun, arg) => {
            expr_levels_are_in_scope(fun, params) && expr_levels_are_in_scope(arg, params)
        }
        Expr::Lam { ty, body, .. } | Expr::Pi { ty, body, .. } => {
            expr_levels_are_in_scope(ty, params) && expr_levels_are_in_scope(body, params)
        }
        Expr::Let {
            ty, value, body, ..
        } => {
            expr_levels_are_in_scope(ty, params)
                && expr_levels_are_in_scope(value, params)
                && expr_levels_are_in_scope(body, params)
        }
    }
}

fn level_is_in_scope(level: &Level, params: &[String]) -> bool {
    match level {
        Level::Zero => true,
        Level::Succ(inner) => level_is_in_scope(inner, params),
        Level::Max(lhs, rhs) | Level::IMax(lhs, rhs) => {
            level_is_in_scope(lhs, params) && level_is_in_scope(rhs, params)
        }
        Level::Param(name) => params.iter().any(|param| param == name),
    }
}

fn constraint_levels_are_in_scope(
    constraint: &Phase9UniverseConstraint,
    params: &[String],
) -> bool {
    level_is_in_scope(&constraint.lhs, params) && level_is_in_scope(&constraint.rhs, params)
}

fn goal_imported_refs_are_resolved(goal: &Phase9AiGoal, imports: &[VerifiedImportRef]) -> bool {
    goal.local_context.iter().all(|local| {
        expr_imported_refs_are_resolved(&local.ty, imports)
            && local
                .value
                .as_ref()
                .is_none_or(|value| expr_imported_refs_are_resolved(value, imports))
    }) && expr_imported_refs_are_resolved(&goal.target, imports)
}

fn expr_imported_refs_are_resolved(expr: &Expr, imports: &[VerifiedImportRef]) -> bool {
    match expr {
        Expr::Sort(_) | Expr::BVar(_) => true,
        Expr::Const { name, .. } => const_name_is_exported_once(name, imports),
        Expr::App(fun, arg) => {
            expr_imported_refs_are_resolved(fun, imports)
                && expr_imported_refs_are_resolved(arg, imports)
        }
        Expr::Lam { ty, body, .. } | Expr::Pi { ty, body, .. } => {
            expr_imported_refs_are_resolved(ty, imports)
                && expr_imported_refs_are_resolved(body, imports)
        }
        Expr::Let {
            ty, value, body, ..
        } => {
            expr_imported_refs_are_resolved(ty, imports)
                && expr_imported_refs_are_resolved(value, imports)
                && expr_imported_refs_are_resolved(body, imports)
        }
    }
}

fn const_name_is_exported_once(name: &str, imports: &[VerifiedImportRef]) -> bool {
    let mut matches = 0usize;
    for import in imports {
        for export in import.exports() {
            if export.name.as_dotted() == name {
                matches += 1;
            }
        }
    }
    matches == 1
}

fn validate_goal_kernel(
    goal: &Phase9AiGoal,
    imports: &[VerifiedImportRef],
) -> std::result::Result<(), ()> {
    let env = phase9_kernel_env_from_imports(imports)?;
    let mut ctx = Ctx::new();
    for local in &goal.local_context {
        expect_sort_public(&env, &ctx, &goal.universe_params, &local.ty)?;
        if let Some(value) = &local.value {
            env.check(&ctx, &goal.universe_params, value, &local.ty)
                .map_err(|_| ())?;
            ctx.push_definition(local.name.clone(), local.ty.clone(), value.clone());
        } else {
            ctx.push_assumption(local.name.clone(), local.ty.clone());
        }
    }
    expect_sort_public(&env, &ctx, &goal.universe_params, &goal.target)
}

fn derive_universe_constraints(
    goal: &Phase9AiGoal,
    repaired_expr: &Expr,
    imports: &[VerifiedImportRef],
) -> std::result::Result<Vec<Phase9UniverseConstraint>, ()> {
    // The current kernel stores no declaration-local universe constraints, so
    // rechecking the repaired goal is the deterministic solver boundary for M2.
    let mut repaired_goal = goal.clone();
    repaired_goal.target = repaired_expr.clone();
    validate_goal_kernel(&repaired_goal, imports)?;
    Ok(Vec::new())
}

fn phase9_kernel_env_from_imports(imports: &[VerifiedImportRef]) -> std::result::Result<Env, ()> {
    let mut env = Env::new();
    for import in imports {
        for decl in import.certified_env_decls() {
            if env.decl(decl.name()).is_some() {
                continue;
            }
            match decl {
                npa_kernel::Decl::Axiom {
                    name,
                    universe_params,
                    ty,
                } => env
                    .add_axiom(name.clone(), universe_params.clone(), ty.clone())
                    .map_err(|_| ())?,
                npa_kernel::Decl::Def {
                    name,
                    universe_params,
                    ty,
                    value,
                    reducibility,
                } => env
                    .add_def(
                        name.clone(),
                        universe_params.clone(),
                        ty.clone(),
                        value.clone(),
                        reducibility.clone(),
                    )
                    .map_err(|_| ())?,
                npa_kernel::Decl::Theorem {
                    name,
                    universe_params,
                    ty,
                    proof,
                } => env
                    .add_theorem(
                        name.clone(),
                        universe_params.clone(),
                        ty.clone(),
                        proof.clone(),
                    )
                    .map_err(|_| ())?,
                npa_kernel::Decl::Inductive { data, .. } => {
                    env.add_inductive((**data).clone()).map_err(|_| ())?
                }
                npa_kernel::Decl::Constructor { .. } | npa_kernel::Decl::Recursor { .. } => {}
            }
        }
    }
    Ok(env)
}

fn expect_sort_public(
    env: &Env,
    ctx: &Ctx,
    delta: &[String],
    term: &Expr,
) -> std::result::Result<(), ()> {
    match env
        .whnf(ctx, delta, &env.infer(ctx, delta, term).map_err(|_| ())?)
        .map_err(|_| ())?
    {
        Expr::Sort(_) => Ok(()),
        _ => Err(()),
    }
}

fn resolve_phase9_global_ref(
    global_ref: &Phase9AiGlobalRef,
    imports: &[VerifiedImportRef],
) -> Option<ResolvedPhase9GlobalRef> {
    let mut matches = Vec::new();
    for import in imports {
        let identity = Phase9ImportIdentity::from_verified_import(import);
        if identity.module != global_ref.module
            || identity.export_hash != global_ref.export_hash
            || identity.certificate_hash != global_ref.certificate_hash
        {
            continue;
        }
        for export in import.exports().iter().filter(|export| {
            export.name == global_ref.name
                && export.decl_interface_hash == global_ref.decl_interface_hash
        }) {
            let decl = import
                .certified_env_decls()
                .iter()
                .find(|decl| decl.name() == export.name.as_dotted())?;
            matches.push(ResolvedPhase9GlobalRef {
                const_name: export.name.as_dotted(),
                universe_arity: decl.universe_params().len(),
            });
        }
    }
    let [resolved] = matches.as_slice() else {
        return None;
    };
    Some(ResolvedPhase9GlobalRef {
        const_name: resolved.const_name.clone(),
        universe_arity: resolved.universe_arity,
    })
}

fn expr_at_path<'a>(expr: &'a Expr, path: &[Phase9MachineExprPathStep]) -> Option<&'a Expr> {
    let mut current = expr;
    for step in path {
        current = match (current, step) {
            (Expr::App(fun, _), Phase9MachineExprPathStep::AppFun) => fun,
            (Expr::App(_, arg), Phase9MachineExprPathStep::AppArg) => arg,
            (Expr::Lam { ty, .. }, Phase9MachineExprPathStep::LamType) => ty,
            (Expr::Lam { body, .. }, Phase9MachineExprPathStep::LamBody) => body,
            (Expr::Pi { ty, .. }, Phase9MachineExprPathStep::PiDomain) => ty,
            (Expr::Pi { body, .. }, Phase9MachineExprPathStep::PiCodomain) => body,
            (Expr::Let { ty, .. }, Phase9MachineExprPathStep::LetType) => ty,
            (Expr::Let { value, .. }, Phase9MachineExprPathStep::LetValue) => value,
            (Expr::Let { body, .. }, Phase9MachineExprPathStep::LetBody) => body,
            _ => return None,
        };
    }
    Some(current)
}

fn replace_const_levels_at_path(
    expr: &mut Expr,
    path: &[Phase9MachineExprPathStep],
    explicit_level_args: Vec<Level>,
) -> Option<()> {
    let current = expr_at_path_mut(expr, path)?;
    let Expr::Const { levels, .. } = current else {
        return None;
    };
    *levels = explicit_level_args;
    Some(())
}

fn expr_at_path_mut<'a>(
    expr: &'a mut Expr,
    path: &[Phase9MachineExprPathStep],
) -> Option<&'a mut Expr> {
    let mut current = expr;
    for step in path {
        current = match (current, step) {
            (Expr::App(fun, _), Phase9MachineExprPathStep::AppFun) => fun,
            (Expr::App(_, arg), Phase9MachineExprPathStep::AppArg) => arg,
            (Expr::Lam { ty, .. }, Phase9MachineExprPathStep::LamType) => ty,
            (Expr::Lam { body, .. }, Phase9MachineExprPathStep::LamBody) => body,
            (Expr::Pi { ty, .. }, Phase9MachineExprPathStep::PiDomain) => ty,
            (Expr::Pi { body, .. }, Phase9MachineExprPathStep::PiCodomain) => body,
            (Expr::Let { ty, .. }, Phase9MachineExprPathStep::LetType) => ty,
            (Expr::Let { value, .. }, Phase9MachineExprPathStep::LetValue) => value,
            (Expr::Let { body, .. }, Phase9MachineExprPathStep::LetBody) => body,
            _ => return None,
        };
    }
    Some(current)
}

fn decode_universe_instantiation_items(
    items: &[Vec<u8>],
) -> std::result::Result<Vec<Phase9UniverseInstantiationPatch>, DecodeError> {
    items
        .iter()
        .map(|item| decode_universe_instantiation_patch(item))
        .collect()
}

fn decode_universe_constraint_hint_items(
    items: &[Vec<u8>],
) -> std::result::Result<Vec<Phase9UniverseConstraintHint>, DecodeError> {
    items
        .iter()
        .map(|item| decode_universe_constraint_hint(item))
        .collect()
}

fn universe_instantiations_are_strictly_sorted(
    instantiations: &[Phase9UniverseInstantiationPatch],
) -> bool {
    let mut previous: Option<Vec<u8>> = None;
    for patch in instantiations {
        let key = universe_instantiation_key(patch);
        if previous.as_ref().is_some_and(|previous| previous >= &key) {
            return false;
        }
        previous = Some(key);
    }
    true
}

fn universe_instantiation_key(patch: &Phase9UniverseInstantiationPatch) -> Vec<u8> {
    let mut out = Vec::new();
    encode_path_steps_to(&mut out, &patch.occurrence.path);
    encode_global_ref_to(&mut out, &patch.occurrence.expected_ref)
        .expect("decoded global refs must be canonical");
    out
}

fn universe_constraint_hints_are_strictly_sorted(hints: &[Phase9UniverseConstraintHint]) -> bool {
    let mut previous: Option<Vec<u8>> = None;
    for hint in hints {
        let key = phase9_universe_constraint_canonical_bytes(&hint.constraint);
        if previous.as_ref().is_some_and(|previous| previous >= &key) {
            return false;
        }
        previous = Some(key);
    }
    true
}

fn phase9_universe_constraint_canonical_bytes(constraint: &Phase9UniverseConstraint) -> Vec<u8> {
    let mut out = Vec::new();
    encode_universe_constraint_to(&mut out, constraint);
    out
}

fn universe_constraint_is_satisfiable(constraint: &Phase9UniverseConstraint) -> bool {
    match constraint.relation {
        Phase9UniverseConstraintRelation::Eq => {
            normalized_levels_are_equal(&constraint.lhs, &constraint.rhs)
        }
        Phase9UniverseConstraintRelation::Le => {
            level_leq_is_satisfiable(&constraint.lhs, &constraint.rhs)
        }
    }
}

fn normalized_levels_are_equal(lhs: &Level, rhs: &Level) -> bool {
    npa_kernel::level::normalize_level(lhs.clone())
        == npa_kernel::level::normalize_level(rhs.clone())
}

fn level_leq_is_satisfiable(lhs: &Level, rhs: &Level) -> bool {
    let lhs = npa_kernel::level::normalize_level(lhs.clone());
    let rhs = npa_kernel::level::normalize_level(rhs.clone());
    if lhs == rhs || lhs == Level::Zero {
        return true;
    }
    match (&lhs, &rhs) {
        (Level::Succ(inner), _) if **inner == rhs => false,
        (Level::Succ(lhs_inner), Level::Succ(rhs_inner)) => {
            level_leq_is_satisfiable(lhs_inner, rhs_inner)
        }
        (Level::Param(_), Level::Succ(_)) => true,
        (_, Level::Max(left, right)) => {
            level_leq_is_satisfiable(&lhs, left) || level_leq_is_satisfiable(&lhs, right)
        }
        _ => true,
    }
}

fn rejected_response(
    candidate_hash: Hash,
    error: Phase9AiValidationError,
    feature_error: Option<Phase9AiFeatureError>,
) -> Phase9AiEndpointResponse {
    Phase9AiEndpointResponse::Rejected {
        candidate_hash,
        validation_result_hash: phase9_ai_validation_result_hash_for_rejection(
            candidate_hash,
            error,
            feature_error,
        ),
        error,
        feature_error,
    }
}

fn validation_result_hash(candidate_hash: Hash, payload: &[u8]) -> Hash {
    let mut bytes = Vec::new();
    encode_hash_to(&mut bytes, &candidate_hash);
    bytes.extend_from_slice(payload);
    hash_with_domain(VALIDATION_RESULT_HASH_TAG, &bytes)
}

fn encode_success_payload_to(out: &mut Vec<u8>, success: &Phase9AiSuccessPayload) {
    match success {
        Phase9AiSuccessPayload::AdvancedInductive {
            decl_interface_hash,
            decl_certificate_hash,
        } => {
            out.push(0);
            encode_hash_to(out, decl_interface_hash);
            encode_hash_to(out, decl_certificate_hash);
        }
        Phase9AiSuccessPayload::UniverseRepair {
            repaired_expr,
            constraint_set_hash,
        } => {
            out.push(1);
            encode_expr_to(out, repaired_expr);
            encode_hash_to(out, constraint_set_hash);
        }
        Phase9AiSuccessPayload::TypeclassResolution { proof } => {
            out.push(2);
            encode_expr_to(out, proof);
        }
        Phase9AiSuccessPayload::QuotientConstruction {
            decl_certificate_hash,
        } => {
            out.push(3);
            encode_hash_to(out, decl_certificate_hash);
        }
        Phase9AiSuccessPayload::SmtCertificate { final_proof } => {
            out.push(4);
            encode_expr_to(out, final_proof);
        }
        Phase9AiSuccessPayload::TheoremGraphQuery { result } => {
            out.push(5);
            encode_theorem_graph_result_to(out, result);
        }
        Phase9AiSuccessPayload::NaturalLanguageFormalization {
            kind,
            accepted_statement_hash,
            formalization_proof_root_hash,
        } => {
            out.push(6);
            out.push(match kind {
                Phase9FormalizationSuccessKind::CandidateStatementChecked => 0,
                Phase9FormalizationSuccessKind::IntentRecordOnly => 1,
                Phase9FormalizationSuccessKind::ProofBridgeChecked => 2,
            });
            encode_option_hash_to(out, accepted_statement_hash.as_ref());
            encode_option_hash_to(out, formalization_proof_root_hash.as_ref());
        }
    }
}

fn encode_candidate_envelope_to(
    out: &mut Vec<u8>,
    envelope: &Phase9AiCandidateEnvelope,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    out.push(envelope.profile_version.tag());
    out.push(envelope.task_kind.tag());
    encode_target_to(out, &envelope.target);
    encode_import_identities_to(out, &envelope.imports)?;
    encode_options_ref_to(out, &envelope.options);
    encode_bytes_to(out, &envelope.payload);
    Ok(())
}

fn encode_target_to(out: &mut Vec<u8>, target: &Phase9AiTarget) {
    encode_hash_to(out, &target.env_fingerprint);
    encode_option_hash_to(out, target.target_decl_hash.as_ref());
    encode_option_hash_to(out, target.goal_fingerprint.as_ref());
}

fn encode_import_identities_to(
    out: &mut Vec<u8>,
    imports: &[Phase9ImportIdentity],
) -> std::result::Result<(), Phase9AiCanonicalError> {
    encode_len_to(out, imports.len());
    for import in imports {
        encode_name_to(out, &import.module)?;
        encode_hash_to(out, &import.export_hash);
        encode_hash_to(out, &import.certificate_hash);
    }
    Ok(())
}

fn encode_options_ref_to(out: &mut Vec<u8>, options_ref: &Phase9AiOptionsRef) {
    match options_ref {
        Phase9AiOptionsRef::Inline {
            options_hash,
            canonical_bytes,
        } => {
            out.push(0);
            encode_hash_to(out, options_hash);
            encode_bytes_to(out, canonical_bytes);
        }
        Phase9AiOptionsRef::Artifact {
            path,
            file_hash,
            options_hash,
            size_bytes,
        } => {
            out.push(1);
            encode_string_to(out, path);
            encode_hash_to(out, file_hash);
            encode_hash_to(out, options_hash);
            encode_u64_to(out, *size_bytes);
        }
    }
}

fn encode_options_to(
    out: &mut Vec<u8>,
    options: &Phase9AiOptions,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    out.push(options.schema_version.tag());
    out.push(options.independent_checker.profile.tag());
    encode_global_ref_list_to(
        out,
        &options.advanced_inductive.approved_nested_type_constructors,
    )?;
    encode_global_ref_list_to(out, &options.typeclass.class_declarations)?;
    encode_option_quotient_to(out, options.quotient.as_ref())?;
    encode_option_smt_to(out, options.smt.as_ref())?;
    encode_option_formalization_to(out, options.formalization.as_ref());
    Ok(())
}

fn encode_global_ref_list_to(
    out: &mut Vec<u8>,
    refs: &[Phase9AiGlobalRef],
) -> std::result::Result<(), Phase9AiCanonicalError> {
    encode_len_to(out, refs.len());
    for global_ref in refs {
        encode_global_ref_to(out, global_ref)?;
    }
    Ok(())
}

fn encode_global_ref_to(
    out: &mut Vec<u8>,
    global_ref: &Phase9AiGlobalRef,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    encode_name_to(out, &global_ref.module)?;
    encode_hash_to(out, &global_ref.export_hash);
    encode_hash_to(out, &global_ref.certificate_hash);
    encode_name_to(out, &global_ref.name)?;
    encode_hash_to(out, &global_ref.decl_interface_hash);
    Ok(())
}

fn encode_option_quotient_to(
    out: &mut Vec<u8>,
    options: Option<&Phase9QuotientOptions>,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    match options {
        Some(options) => {
            out.push(1);
            encode_global_ref_to(out, &options.setoid)?;
            encode_global_ref_to(out, &options.setoid_mk)?;
            encode_global_ref_to(out, &options.setoid_relation)?;
            encode_global_ref_to(out, &options.rel_equiv)?;
            encode_global_ref_to(out, &options.quotient)?;
            encode_global_ref_to(out, &options.quotient_mk)?;
            encode_global_ref_to(out, &options.quotient_sound)?;
            encode_global_ref_to(out, &options.quotient_lift)?;
            encode_global_ref_to(out, &options.eq)?;
        }
        None => out.push(0),
    }
    Ok(())
}

fn encode_option_smt_to(
    out: &mut Vec<u8>,
    options: Option<&Phase9SmtOptions>,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    match options {
        Some(options) => {
            out.push(1);
            encode_global_ref_to(out, &options.eq)?;
            encode_option_global_ref_to(out, options.prop_false.as_ref())?;
            encode_option_global_ref_to(out, options.prop_not.as_ref())?;
        }
        None => out.push(0),
    }
    Ok(())
}

fn encode_option_global_ref_to(
    out: &mut Vec<u8>,
    global_ref: Option<&Phase9AiGlobalRef>,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    match global_ref {
        Some(global_ref) => {
            out.push(1);
            encode_global_ref_to(out, global_ref)?;
        }
        None => out.push(0),
    }
    Ok(())
}

fn encode_option_formalization_to(out: &mut Vec<u8>, options: Option<&Phase9FormalizationOptions>) {
    match options {
        Some(options) => {
            out.push(1);
            encode_bytes_to(out, &options.tactic_options_canonical_bytes);
            encode_bytes_to(out, &options.tactic_budget_canonical_bytes);
        }
        None => out.push(0),
    }
}

fn encode_inductive_proposal_to(
    out: &mut Vec<u8>,
    proposal: &Phase9MachineInductiveProposal,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    encode_option_name_to(out, proposal.block_name.as_ref())?;
    encode_option_hash_to(out, proposal.expected_decl_hash.as_ref());
    encode_len_to(out, proposal.universe_params.len());
    for param in &proposal.universe_params {
        encode_string_to(out, param);
    }
    encode_len_to(out, proposal.inductives.len());
    for family in &proposal.inductives {
        encode_inductive_family_to(out, family)?;
    }
    Ok(())
}

fn encode_option_name_to(
    out: &mut Vec<u8>,
    name: Option<&Name>,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    match name {
        Some(name) => {
            out.push(1);
            encode_name_to(out, name)?;
        }
        None => out.push(0),
    }
    Ok(())
}

fn encode_inductive_family_to(
    out: &mut Vec<u8>,
    family: &Phase9MachineInductiveFamilyProposal,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    encode_name_to(out, &family.name)?;
    encode_telescope_to(out, &family.params);
    encode_telescope_to(out, &family.indices);
    encode_level_to(out, &family.result_sort);
    encode_len_to(out, family.constructors.len());
    for constructor in &family.constructors {
        encode_name_to(out, &constructor.name)?;
        encode_expr_to(out, &constructor.ty);
    }
    Ok(())
}

fn encode_telescope_to(out: &mut Vec<u8>, telescope: &[Phase9MachineTelescopeBinder]) {
    encode_len_to(out, telescope.len());
    for binder in telescope {
        encode_expr_to(out, &binder.ty);
    }
}

fn encode_theorem_graph_query_to(
    out: &mut Vec<u8>,
    query: &Phase9MachineTheoremGraphQuery,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    encode_hash_to(out, &query.env_fingerprint);
    encode_hash_to(out, &query.goal_fingerprint);
    encode_goal_to(out, &query.goal)?;
    encode_theorem_graph_snapshot_ref_to(out, &query.snapshot)?;
    encode_theorem_graph_query_features_ref_to(out, &query.query_features);
    out.push(query.ranking_profile.tag());
    encode_u64_to(out, u64::from(query.limit));
    Ok(())
}

fn encode_theorem_graph_snapshot_ref_to(
    out: &mut Vec<u8>,
    snapshot: &Phase9MachineTheoremGraphSnapshotRef,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    encode_hash_to(out, &snapshot.source_release_hash);
    out.push(snapshot.extractor_version.tag());
    match &snapshot.source {
        Phase9MachineTheoremGraphSnapshotSource::Inline {
            graph_snapshot_hash,
            canonical_bytes,
        } => {
            out.push(0);
            encode_hash_to(out, graph_snapshot_hash);
            encode_bytes_to(out, canonical_bytes);
        }
        Phase9MachineTheoremGraphSnapshotSource::Artifact {
            path,
            file_hash,
            graph_snapshot_hash,
            size_bytes,
        } => {
            out.push(1);
            encode_string_to(out, path);
            encode_hash_to(out, file_hash);
            encode_hash_to(out, graph_snapshot_hash);
            encode_u64_to(out, *size_bytes);
        }
    }
    Ok(())
}

fn encode_theorem_graph_query_features_ref_to(
    out: &mut Vec<u8>,
    features: &Phase9MachineTheoremGraphQueryFeaturesRef,
) {
    match features {
        Phase9MachineTheoremGraphQueryFeaturesRef::Inline {
            query_features_hash,
            canonical_bytes,
        } => {
            out.push(0);
            encode_hash_to(out, query_features_hash);
            encode_bytes_to(out, canonical_bytes);
        }
        Phase9MachineTheoremGraphQueryFeaturesRef::Artifact {
            path,
            file_hash,
            query_features_hash,
            size_bytes,
        } => {
            out.push(1);
            encode_string_to(out, path);
            encode_hash_to(out, file_hash);
            encode_hash_to(out, query_features_hash);
            encode_u64_to(out, *size_bytes);
        }
    }
}

fn encode_theorem_graph_snapshot_to(
    out: &mut Vec<u8>,
    snapshot: &Phase9MachineTheoremGraphSnapshot,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    encode_hash_to(out, &snapshot.source_release_hash);
    out.push(snapshot.extractor_version.tag());
    encode_len_to(out, snapshot.nodes.len());
    for node in &snapshot.nodes {
        encode_theorem_graph_node_to(out, node)?;
    }
    encode_len_to(out, snapshot.edges.len());
    for edge in &snapshot.edges {
        encode_theorem_graph_edge_to(out, edge)?;
    }
    Ok(())
}

fn encode_theorem_graph_query_features_to(
    out: &mut Vec<u8>,
    features: &Phase9MachineTheoremGraphQueryFeatures,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    encode_hash_to(out, &features.env_fingerprint);
    encode_hash_to(out, &features.goal_fingerprint);
    out.push(features.feature_schema_version.tag());
    encode_len_to(out, features.features.len());
    for feature in &features.features {
        encode_theorem_graph_feature_to(out, feature);
    }
    Ok(())
}

fn encode_theorem_graph_result_to(out: &mut Vec<u8>, result: &Phase9MachineTheoremGraphResult) {
    encode_len_to(out, result.entries.len());
    for entry in &result.entries {
        encode_theorem_graph_node_to(out, &entry.node)
            .expect("validated theorem graph result node names are canonical");
        encode_i64_to(out, entry.score.score_microunits);
    }
}

fn encode_theorem_graph_edge_to(
    out: &mut Vec<u8>,
    edge: &Phase9MachineTheoremGraphEdge,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    encode_theorem_graph_node_to(out, &edge.from)?;
    encode_theorem_graph_node_to(out, &edge.to)?;
    out.push(edge.kind.tag());
    Ok(())
}

fn encode_theorem_graph_node_to(
    out: &mut Vec<u8>,
    node: &Phase9MachineTheoremGraphNodeRef,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    encode_name_to(out, &node.module)?;
    encode_name_to(out, &node.name)?;
    encode_hash_to(out, &node.export_hash);
    encode_hash_to(out, &node.decl_certificate_hash);
    encode_hash_to(out, &node.type_hash);
    encode_hash_to(out, &node.certificate_hash);
    encode_hash_to(out, &node.decl_interface_hash);
    Ok(())
}

fn phase9_theorem_graph_node_canonical_bytes(
    node: &Phase9MachineTheoremGraphNodeRef,
) -> std::result::Result<Vec<u8>, Phase9AiCanonicalError> {
    let mut out = Vec::new();
    encode_theorem_graph_node_to(&mut out, node)?;
    Ok(out)
}

fn encode_theorem_graph_feature_to(out: &mut Vec<u8>, feature: &Phase9MachineTheoremGraphFeature) {
    encode_bytes_to(out, &feature.key.namespace_ascii);
    encode_bytes_to(out, &feature.key.name_ascii);
    match &feature.value {
        Phase9TheoremGraphFeatureValue::Bool(value) => {
            out.push(0);
            out.push(u8::from(*value));
        }
        Phase9TheoremGraphFeatureValue::I64(value) => {
            out.push(1);
            encode_i64_to(out, *value);
        }
        Phase9TheoremGraphFeatureValue::Hash(value) => {
            out.push(2);
            encode_hash_to(out, value);
        }
    }
}

fn encode_universe_repair_candidate_to(
    out: &mut Vec<u8>,
    candidate: &Phase9UniverseRepairCandidate,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    encode_option_goal_to(out, candidate.goal.as_ref())?;
    encode_expr_to(out, &candidate.target_expr);
    encode_len_to(out, candidate.instantiations.len());
    for patch in &candidate.instantiations {
        let mut item = Vec::new();
        encode_universe_instantiation_patch_to(&mut item, patch)?;
        encode_bytes_to(out, &item);
    }
    encode_len_to(out, candidate.constraint_hints.len());
    for hint in &candidate.constraint_hints {
        let mut item = Vec::new();
        encode_universe_constraint_hint_to(&mut item, hint);
        encode_bytes_to(out, &item);
    }
    encode_option_minimization_hint_to(out, candidate.minimization_hint);
    Ok(())
}

fn encode_universe_repair_candidate_outer_to(
    out: &mut Vec<u8>,
    candidate: &Phase9UniverseRepairCandidateOuter,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    encode_option_goal_to(out, candidate.goal.as_ref())?;
    encode_expr_to(out, &candidate.target_expr);
    encode_raw_bytes_list_to(out, &candidate.instantiation_items);
    encode_raw_bytes_list_to(out, &candidate.constraint_hint_items);
    encode_option_minimization_hint_to(out, candidate.minimization_hint);
    Ok(())
}

fn encode_option_goal_to(
    out: &mut Vec<u8>,
    goal: Option<&Phase9AiGoal>,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    match goal {
        Some(goal) => {
            out.push(1);
            encode_goal_to(out, goal)?;
        }
        None => out.push(0),
    }
    Ok(())
}

fn encode_goal_to(
    out: &mut Vec<u8>,
    goal: &Phase9AiGoal,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    encode_len_to(out, goal.universe_params.len());
    for param in &goal.universe_params {
        encode_string_to(out, param);
    }
    encode_len_to(out, goal.local_context.len());
    for local in &goal.local_context {
        encode_machine_local_decl_to(out, local);
    }
    encode_expr_to(out, &goal.target);
    Ok(())
}

fn encode_machine_local_decl_to(out: &mut Vec<u8>, local: &MachineLocalDecl) {
    encode_string_to(out, &local.name);
    encode_expr_to(out, &local.ty);
    match &local.value {
        Some(value) => {
            out.push(1);
            encode_expr_to(out, value);
        }
        None => out.push(0),
    }
}

fn encode_universe_instantiation_patch_to(
    out: &mut Vec<u8>,
    patch: &Phase9UniverseInstantiationPatch,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    encode_path_steps_to(out, &patch.occurrence.path);
    encode_global_ref_to(out, &patch.occurrence.expected_ref)?;
    encode_len_to(out, patch.explicit_level_args.len());
    for level in &patch.explicit_level_args {
        encode_level_to(out, level);
    }
    Ok(())
}

fn encode_universe_constraint_hint_to(out: &mut Vec<u8>, hint: &Phase9UniverseConstraintHint) {
    encode_universe_constraint_to(out, &hint.constraint);
    out.push(match hint.reason {
        Phase9UniverseConstraintHintReason::KernelDiagnostic => 0,
        Phase9UniverseConstraintHintReason::RepairCandidate => 1,
        Phase9UniverseConstraintHintReason::MinimizationExplanation => 2,
    });
}

fn encode_universe_constraint_to(out: &mut Vec<u8>, constraint: &Phase9UniverseConstraint) {
    encode_level_to(out, &constraint.lhs);
    out.push(match constraint.relation {
        Phase9UniverseConstraintRelation::Le => 0,
        Phase9UniverseConstraintRelation::Eq => 1,
    });
    encode_level_to(out, &constraint.rhs);
}

fn encode_option_minimization_hint_to(
    out: &mut Vec<u8>,
    hint: Option<Phase9UniverseMinimizationHint>,
) {
    match hint {
        Some(hint) => {
            out.push(1);
            out.push(match hint {
                Phase9UniverseMinimizationHint::KernelDefault => 0,
                Phase9UniverseMinimizationHint::PreferLowerLevels => 1,
                Phase9UniverseMinimizationHint::PreferExistingExplicitArgs => 2,
            });
        }
        None => out.push(0),
    }
}

fn encode_path_steps_to(out: &mut Vec<u8>, path: &[Phase9MachineExprPathStep]) {
    encode_len_to(out, path.len());
    for step in path {
        out.push(match step {
            Phase9MachineExprPathStep::AppFun => 0,
            Phase9MachineExprPathStep::AppArg => 1,
            Phase9MachineExprPathStep::LamType => 2,
            Phase9MachineExprPathStep::LamBody => 3,
            Phase9MachineExprPathStep::PiDomain => 4,
            Phase9MachineExprPathStep::PiCodomain => 5,
            Phase9MachineExprPathStep::LetType => 6,
            Phase9MachineExprPathStep::LetValue => 7,
            Phase9MachineExprPathStep::LetBody => 8,
        });
    }
}

fn encode_expr_to(out: &mut Vec<u8>, expr: &Expr) {
    match expr {
        Expr::Sort(level) => {
            out.push(0);
            encode_level_to(out, level);
        }
        Expr::BVar(index) => {
            out.push(1);
            encode_u64_to(out, u64::from(*index));
        }
        Expr::Const { name, levels } => {
            out.push(2);
            encode_string_to(out, name);
            encode_len_to(out, levels.len());
            for level in levels {
                encode_level_to(out, level);
            }
        }
        Expr::App(fun, arg) => {
            out.push(3);
            encode_expr_to(out, fun);
            encode_expr_to(out, arg);
        }
        Expr::Lam { ty, body, .. } => {
            out.push(4);
            encode_expr_to(out, ty);
            encode_expr_to(out, body);
        }
        Expr::Pi { ty, body, .. } => {
            out.push(5);
            encode_expr_to(out, ty);
            encode_expr_to(out, body);
        }
        Expr::Let {
            ty, value, body, ..
        } => {
            out.push(6);
            encode_expr_to(out, ty);
            encode_expr_to(out, value);
            encode_expr_to(out, body);
        }
    }
}

fn encode_level_to(out: &mut Vec<u8>, level: &Level) {
    match npa_kernel::level::normalize_level(level.clone()) {
        Level::Zero => out.push(0),
        Level::Succ(inner) => {
            out.push(1);
            encode_level_to(out, &inner);
        }
        Level::Max(lhs, rhs) => {
            out.push(2);
            encode_level_to(out, &lhs);
            encode_level_to(out, &rhs);
        }
        Level::IMax(lhs, rhs) => {
            out.push(3);
            encode_level_to(out, &lhs);
            encode_level_to(out, &rhs);
        }
        Level::Param(name) => {
            out.push(4);
            encode_string_to(out, &name);
        }
    }
}

fn encode_raw_bytes_list_to(out: &mut Vec<u8>, items: &[Vec<u8>]) {
    encode_len_to(out, items.len());
    for item in items {
        encode_bytes_to(out, item);
    }
}

fn decode_candidate_envelope(
    input: &[u8],
) -> std::result::Result<Phase9AiCandidateEnvelope, DecodeError> {
    let mut decoder = Decoder::new(input);
    let profile_version =
        Phase9AiProfileVersion::from_tag(decoder.u8()?).ok_or(DecodeError::Malformed)?;
    let task_kind = Phase9AiTaskKind::from_tag(decoder.u8()?).ok_or(DecodeError::Malformed)?;
    let target = decoder.target()?;
    let imports = decoder.import_identities()?;
    let options = decoder.options_ref()?;
    let payload = decoder.bytes()?;
    decoder.done()?;

    let envelope = Phase9AiCandidateEnvelope {
        profile_version,
        task_kind,
        target,
        imports,
        options,
        payload,
    };
    let encoded = phase9_ai_candidate_envelope_canonical_bytes(&envelope)
        .map_err(|_| DecodeError::Malformed)?;
    if encoded != input {
        return Err(DecodeError::Malformed);
    }
    Ok(envelope)
}

fn decode_options(input: &[u8]) -> std::result::Result<Phase9AiOptions, DecodeError> {
    let mut decoder = Decoder::new(input);
    let schema_version =
        Phase9AiOptionsVersion::from_tag(decoder.u8()?).ok_or(DecodeError::Malformed)?;
    let independent_checker = Phase9IndependentCheckerOptions {
        profile: Phase9IndependentCheckerProfile::from_tag(decoder.u8()?)
            .ok_or(DecodeError::Malformed)?,
    };
    let approved_nested_type_constructors = decoder.global_ref_list()?;
    ensure_sorted_global_refs(&approved_nested_type_constructors)?;
    let class_declarations = decoder.global_ref_list()?;
    ensure_sorted_global_refs(&class_declarations)?;
    let quotient = decoder.option_quotient()?;
    let smt = decoder.option_smt()?;
    let formalization = decoder.option_formalization()?;
    decoder.done()?;

    let options = Phase9AiOptions {
        schema_version,
        independent_checker,
        advanced_inductive: Phase9AdvancedInductiveOptions {
            approved_nested_type_constructors,
        },
        typeclass: Phase9TypeclassOptions { class_declarations },
        quotient,
        smt,
        formalization,
    };
    let encoded =
        phase9_ai_options_canonical_bytes(&options).map_err(|_| DecodeError::Malformed)?;
    if encoded != input {
        return Err(DecodeError::Malformed);
    }
    Ok(options)
}

struct Phase9InductiveDecodeBudget {
    expr_nodes: u64,
    level_nodes: u64,
}

impl Phase9InductiveDecodeBudget {
    fn new() -> Self {
        Self {
            expr_nodes: 0,
            level_nodes: 0,
        }
    }

    fn spend_expr(&mut self) -> std::result::Result<(), DecodeError> {
        self.expr_nodes = self
            .expr_nodes
            .checked_add(1)
            .ok_or(DecodeError::Malformed)?;
        if self.expr_nodes > MAX_PHASE9_INDUCTIVE_EXPR_NODES {
            return Err(DecodeError::Malformed);
        }
        Ok(())
    }

    fn spend_level(&mut self) -> std::result::Result<(), DecodeError> {
        self.level_nodes = self
            .level_nodes
            .checked_add(1)
            .ok_or(DecodeError::Malformed)?;
        if self.level_nodes > MAX_PHASE9_INDUCTIVE_LEVEL_NODES {
            return Err(DecodeError::Malformed);
        }
        Ok(())
    }
}

fn decode_inductive_proposal(
    input: &[u8],
) -> std::result::Result<Phase9MachineInductiveProposal, DecodeError> {
    let mut decoder = Decoder::new(input);
    let mut budget = Phase9InductiveDecodeBudget::new();
    let block_name = decoder.option_name()?;
    let expected_decl_hash = decoder.option_hash()?;
    let universe_params = decoder.string_list_with_cap(MAX_PHASE9_INDUCTIVE_ITEMS)?;
    let inductive_len = decoder.u64()?;
    if inductive_len > MAX_PHASE9_INDUCTIVE_ITEMS {
        return Err(DecodeError::Malformed);
    }
    let mut inductives = Vec::new();
    for _ in 0..inductive_len {
        inductives.push(decoder.inductive_family(&mut budget)?);
    }
    decoder.done()?;
    let proposal = Phase9MachineInductiveProposal {
        block_name,
        expected_decl_hash,
        universe_params,
        inductives,
    };
    let encoded =
        phase9_inductive_proposal_canonical_bytes(&proposal).map_err(|_| DecodeError::Malformed)?;
    if encoded != input {
        return Err(DecodeError::Malformed);
    }
    Ok(proposal)
}

fn decode_theorem_graph_query(
    input: &[u8],
) -> std::result::Result<Phase9MachineTheoremGraphQuery, DecodeError> {
    let mut decoder = Decoder::new(input);
    let query = decoder.theorem_graph_query()?;
    decoder.done()?;
    let encoded =
        phase9_theorem_graph_query_canonical_bytes(&query).map_err(|_| DecodeError::Malformed)?;
    if encoded != input {
        return Err(DecodeError::Malformed);
    }
    Ok(query)
}

fn decode_theorem_graph_snapshot(
    input: &[u8],
) -> std::result::Result<Phase9MachineTheoremGraphSnapshot, DecodeError> {
    let mut decoder = Decoder::new(input);
    let snapshot = decoder.theorem_graph_snapshot()?;
    decoder.done()?;
    let encoded = phase9_theorem_graph_snapshot_canonical_bytes(&snapshot)
        .map_err(|_| DecodeError::Malformed)?;
    if encoded != input {
        return Err(DecodeError::Malformed);
    }
    Ok(snapshot)
}

fn decode_theorem_graph_query_features(
    input: &[u8],
) -> std::result::Result<Phase9MachineTheoremGraphQueryFeatures, DecodeError> {
    let mut decoder = Decoder::new(input);
    let features = decoder.theorem_graph_query_features()?;
    decoder.done()?;
    let encoded = phase9_theorem_graph_query_features_canonical_bytes(&features)
        .map_err(|_| DecodeError::Malformed)?;
    if encoded != input {
        return Err(DecodeError::Malformed);
    }
    Ok(features)
}

fn decode_universe_repair_candidate_outer(
    input: &[u8],
) -> std::result::Result<Phase9UniverseRepairCandidateOuter, DecodeError> {
    let mut decoder = Decoder::new(input);
    let goal = decoder.option_goal()?;
    let target_expr = decoder.expr()?;
    let instantiation_items = decoder.bytes_list_with_cap(MAX_PHASE9_UNIVERSE_REPAIR_ITEMS)?;
    let constraint_hint_items = decoder.bytes_list_with_cap(MAX_PHASE9_UNIVERSE_REPAIR_ITEMS)?;
    let minimization_hint = decoder.option_minimization_hint()?;
    decoder.done()?;

    let candidate = Phase9UniverseRepairCandidateOuter {
        goal,
        target_expr,
        instantiation_items,
        constraint_hint_items,
        minimization_hint,
    };
    let mut encoded = Vec::new();
    encode_universe_repair_candidate_outer_to(&mut encoded, &candidate)
        .map_err(|_| DecodeError::Malformed)?;
    if encoded != input {
        return Err(DecodeError::Malformed);
    }
    Ok(candidate)
}

fn decode_universe_instantiation_patch(
    input: &[u8],
) -> std::result::Result<Phase9UniverseInstantiationPatch, DecodeError> {
    let mut decoder = Decoder::new(input);
    let path = decoder.path_steps()?;
    let expected_ref = decoder.global_ref()?;
    let explicit_level_args = decoder.level_list_with_cap(MAX_PHASE9_UNIVERSE_REPAIR_ITEMS)?;
    decoder.done()?;
    let patch = Phase9UniverseInstantiationPatch {
        occurrence: Phase9MachineExprOccurrence { path, expected_ref },
        explicit_level_args,
    };
    let mut encoded = Vec::new();
    encode_universe_instantiation_patch_to(&mut encoded, &patch)
        .map_err(|_| DecodeError::Malformed)?;
    if encoded != input {
        return Err(DecodeError::Malformed);
    }
    Ok(patch)
}

fn decode_universe_constraint_hint(
    input: &[u8],
) -> std::result::Result<Phase9UniverseConstraintHint, DecodeError> {
    let mut decoder = Decoder::new(input);
    let constraint = decoder.universe_constraint()?;
    let reason = decoder.constraint_hint_reason()?;
    decoder.done()?;
    let hint = Phase9UniverseConstraintHint { constraint, reason };
    let mut encoded = Vec::new();
    encode_universe_constraint_hint_to(&mut encoded, &hint);
    if encoded != input {
        return Err(DecodeError::Malformed);
    }
    Ok(hint)
}

struct Decoder<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> Decoder<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self { input, pos: 0 }
    }

    fn done(&self) -> std::result::Result<(), DecodeError> {
        if self.pos == self.input.len() {
            Ok(())
        } else {
            Err(DecodeError::Malformed)
        }
    }

    fn u8(&mut self) -> std::result::Result<u8, DecodeError> {
        let value = *self.input.get(self.pos).ok_or(DecodeError::Malformed)?;
        self.pos += 1;
        Ok(value)
    }

    fn u64(&mut self) -> std::result::Result<u64, DecodeError> {
        let end = self.pos.checked_add(8).ok_or(DecodeError::Malformed)?;
        let bytes = self
            .input
            .get(self.pos..end)
            .ok_or(DecodeError::Malformed)?;
        self.pos = end;
        Ok(u64::from_be_bytes(bytes.try_into().unwrap()))
    }

    fn i64(&mut self) -> std::result::Result<i64, DecodeError> {
        let end = self.pos.checked_add(8).ok_or(DecodeError::Malformed)?;
        let bytes = self
            .input
            .get(self.pos..end)
            .ok_or(DecodeError::Malformed)?;
        self.pos = end;
        Ok(i64::from_be_bytes(bytes.try_into().unwrap()))
    }

    fn hash(&mut self) -> std::result::Result<Hash, DecodeError> {
        let end = self.pos.checked_add(32).ok_or(DecodeError::Malformed)?;
        let bytes = self
            .input
            .get(self.pos..end)
            .ok_or(DecodeError::Malformed)?;
        self.pos = end;
        Ok(bytes.try_into().unwrap())
    }

    fn bytes(&mut self) -> std::result::Result<Vec<u8>, DecodeError> {
        let len = usize::try_from(self.u64()?).map_err(|_| DecodeError::Malformed)?;
        let end = self.pos.checked_add(len).ok_or(DecodeError::Malformed)?;
        let bytes = self
            .input
            .get(self.pos..end)
            .ok_or(DecodeError::Malformed)?;
        self.pos = end;
        Ok(bytes.to_vec())
    }

    fn bytes_with_cap(
        &mut self,
        cap: usize,
        cap_error: DecodeError,
    ) -> std::result::Result<Vec<u8>, DecodeError> {
        let len = self.u64()?;
        if usize::try_from(len).map(|len| len > cap).unwrap_or(true) {
            return Err(cap_error);
        }
        let len = usize::try_from(len).map_err(|_| DecodeError::Malformed)?;
        let end = self.pos.checked_add(len).ok_or(DecodeError::Malformed)?;
        let bytes = self
            .input
            .get(self.pos..end)
            .ok_or(DecodeError::Malformed)?;
        self.pos = end;
        Ok(bytes.to_vec())
    }

    fn skip_bytes(&mut self) -> std::result::Result<(), DecodeError> {
        let len = usize::try_from(self.u64()?).map_err(|_| DecodeError::Malformed)?;
        self.skip_raw_bytes(len)
    }

    fn skip_string(&mut self) -> std::result::Result<(), DecodeError> {
        let len = usize::try_from(self.u64()?).map_err(|_| DecodeError::Malformed)?;
        if len as u64 > MAX_STRING_BYTES {
            return Err(DecodeError::Malformed);
        }
        self.skip_raw_bytes(len)
    }

    fn skip_raw_bytes(&mut self, len: usize) -> std::result::Result<(), DecodeError> {
        let end = self.pos.checked_add(len).ok_or(DecodeError::Malformed)?;
        self.input
            .get(self.pos..end)
            .ok_or(DecodeError::Malformed)?;
        self.pos = end;
        Ok(())
    }

    fn bytes_list_with_cap(&mut self, cap: u64) -> std::result::Result<Vec<Vec<u8>>, DecodeError> {
        let len = self.u64()?;
        if len > cap {
            return Err(DecodeError::Malformed);
        }
        let len = usize::try_from(len).map_err(|_| DecodeError::Malformed)?;
        let mut items = Vec::new();
        for _ in 0..len {
            items.push(self.bytes()?);
        }
        Ok(items)
    }

    fn string(&mut self) -> std::result::Result<String, DecodeError> {
        let bytes = self.bytes()?;
        if bytes.len() as u64 > MAX_STRING_BYTES {
            return Err(DecodeError::Malformed);
        }
        String::from_utf8(bytes).map_err(|_| DecodeError::Malformed)
    }

    fn name(&mut self) -> std::result::Result<Name, DecodeError> {
        let len = self.u64()?;
        if len == 0 || len > MAX_NAME_COMPONENTS {
            return Err(DecodeError::Malformed);
        }
        let mut components = Vec::new();
        for _ in 0..len {
            let component = self.string()?;
            components.push(component);
        }
        let name = Name(components);
        if name.is_canonical() {
            Ok(name)
        } else {
            Err(DecodeError::Malformed)
        }
    }

    fn option_name(&mut self) -> std::result::Result<Option<Name>, DecodeError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.name()?)),
            _ => Err(DecodeError::Malformed),
        }
    }

    fn target(&mut self) -> std::result::Result<Phase9AiTarget, DecodeError> {
        Ok(Phase9AiTarget {
            env_fingerprint: self.hash()?,
            target_decl_hash: self.option_hash()?,
            goal_fingerprint: self.option_hash()?,
        })
    }

    fn option_hash(&mut self) -> std::result::Result<Option<Hash>, DecodeError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.hash()?)),
            _ => Err(DecodeError::Malformed),
        }
    }

    fn string_list_with_cap(&mut self, cap: u64) -> std::result::Result<Vec<String>, DecodeError> {
        let len = self.u64()?;
        if len > cap {
            return Err(DecodeError::Malformed);
        }
        let len = usize::try_from(len).map_err(|_| DecodeError::Malformed)?;
        let mut values = Vec::new();
        for _ in 0..len {
            values.push(self.string()?);
        }
        Ok(values)
    }

    fn import_identities(&mut self) -> std::result::Result<Vec<Phase9ImportIdentity>, DecodeError> {
        let len = usize::try_from(self.u64()?).map_err(|_| DecodeError::Malformed)?;
        let mut imports = Vec::new();
        for _ in 0..len {
            imports.push(Phase9ImportIdentity {
                module: self.name()?,
                export_hash: self.hash()?,
                certificate_hash: self.hash()?,
            });
        }
        Ok(imports)
    }

    fn options_ref(&mut self) -> std::result::Result<Phase9AiOptionsRef, DecodeError> {
        match self.u8()? {
            0 => Ok(Phase9AiOptionsRef::Inline {
                options_hash: self.hash()?,
                canonical_bytes: self.bytes()?,
            }),
            1 => Ok(Phase9AiOptionsRef::Artifact {
                path: self.string()?,
                file_hash: self.hash()?,
                options_hash: self.hash()?,
                size_bytes: self.u64()?,
            }),
            _ => Err(DecodeError::Malformed),
        }
    }

    fn option_goal(&mut self) -> std::result::Result<Option<Phase9AiGoal>, DecodeError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.goal()?)),
            _ => Err(DecodeError::Malformed),
        }
    }

    fn goal(&mut self) -> std::result::Result<Phase9AiGoal, DecodeError> {
        let param_len = self.u64()?;
        if param_len > MAX_NAME_COMPONENTS {
            return Err(DecodeError::Malformed);
        }
        let mut universe_params = Vec::new();
        for _ in 0..param_len {
            universe_params.push(self.string()?);
        }
        let local_len = self.u64()?;
        if local_len > MAX_PHASE9_UNIVERSE_REPAIR_ITEMS {
            return Err(DecodeError::Malformed);
        }
        let mut local_context = Vec::new();
        for _ in 0..local_len {
            local_context.push(self.machine_local_decl()?);
        }
        let target = self.expr()?;
        Ok(Phase9AiGoal {
            universe_params,
            local_context,
            target,
        })
    }

    fn machine_local_decl(&mut self) -> std::result::Result<MachineLocalDecl, DecodeError> {
        let name = self.string()?;
        let ty = self.expr()?;
        let value = match self.u8()? {
            0 => None,
            1 => Some(self.expr()?),
            _ => return Err(DecodeError::Malformed),
        };
        Ok(MachineLocalDecl { name, ty, value })
    }

    fn expr(&mut self) -> std::result::Result<Expr, DecodeError> {
        match self.u8()? {
            0 => Ok(Expr::sort(self.level()?)),
            1 => {
                let index = u32::try_from(self.u64()?).map_err(|_| DecodeError::Malformed)?;
                Ok(Expr::bvar(index))
            }
            2 => {
                let name = self.string()?;
                let levels = self.level_list_with_cap(MAX_PHASE9_UNIVERSE_REPAIR_ITEMS)?;
                Ok(Expr::konst(name, levels))
            }
            3 => {
                let fun = self.expr()?;
                let arg = self.expr()?;
                Ok(Expr::app(fun, arg))
            }
            4 => {
                let ty = self.expr()?;
                let body = self.expr()?;
                Ok(Expr::lam("_", ty, body))
            }
            5 => {
                let ty = self.expr()?;
                let body = self.expr()?;
                Ok(Expr::pi("_", ty, body))
            }
            6 => {
                let ty = self.expr()?;
                let value = self.expr()?;
                let body = self.expr()?;
                Ok(Expr::let_in("_", ty, value, body))
            }
            _ => Err(DecodeError::Malformed),
        }
    }

    fn level(&mut self) -> std::result::Result<Level, DecodeError> {
        match self.u8()? {
            0 => Ok(Level::Zero),
            1 => Ok(Level::succ(self.level()?)),
            2 => {
                let lhs = self.level()?;
                let rhs = self.level()?;
                Ok(Level::max(lhs, rhs))
            }
            3 => {
                let lhs = self.level()?;
                let rhs = self.level()?;
                Ok(Level::imax(lhs, rhs))
            }
            4 => Ok(Level::param(self.string()?)),
            _ => Err(DecodeError::Malformed),
        }
    }

    fn level_list_with_cap(&mut self, cap: u64) -> std::result::Result<Vec<Level>, DecodeError> {
        let len = self.u64()?;
        if len > cap {
            return Err(DecodeError::Malformed);
        }
        let len = usize::try_from(len).map_err(|_| DecodeError::Malformed)?;
        let mut levels = Vec::new();
        for _ in 0..len {
            levels.push(self.level()?);
        }
        Ok(levels)
    }

    fn path_steps(&mut self) -> std::result::Result<Vec<Phase9MachineExprPathStep>, DecodeError> {
        let len = self.u64()?;
        if len > MAX_PHASE9_UNIVERSE_REPAIR_ITEMS {
            return Err(DecodeError::Malformed);
        }
        let len = usize::try_from(len).map_err(|_| DecodeError::Malformed)?;
        let mut path = Vec::new();
        for _ in 0..len {
            path.push(match self.u8()? {
                0 => Phase9MachineExprPathStep::AppFun,
                1 => Phase9MachineExprPathStep::AppArg,
                2 => Phase9MachineExprPathStep::LamType,
                3 => Phase9MachineExprPathStep::LamBody,
                4 => Phase9MachineExprPathStep::PiDomain,
                5 => Phase9MachineExprPathStep::PiCodomain,
                6 => Phase9MachineExprPathStep::LetType,
                7 => Phase9MachineExprPathStep::LetValue,
                8 => Phase9MachineExprPathStep::LetBody,
                _ => return Err(DecodeError::Malformed),
            });
        }
        Ok(path)
    }

    fn option_minimization_hint(
        &mut self,
    ) -> std::result::Result<Option<Phase9UniverseMinimizationHint>, DecodeError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(match self.u8()? {
                0 => Phase9UniverseMinimizationHint::KernelDefault,
                1 => Phase9UniverseMinimizationHint::PreferLowerLevels,
                2 => Phase9UniverseMinimizationHint::PreferExistingExplicitArgs,
                _ => return Err(DecodeError::Malformed),
            })),
            _ => Err(DecodeError::Malformed),
        }
    }

    fn universe_constraint(
        &mut self,
    ) -> std::result::Result<Phase9UniverseConstraint, DecodeError> {
        let lhs = self.level()?;
        let relation = match self.u8()? {
            0 => Phase9UniverseConstraintRelation::Le,
            1 => Phase9UniverseConstraintRelation::Eq,
            _ => return Err(DecodeError::Malformed),
        };
        let rhs = self.level()?;
        Ok(Phase9UniverseConstraint { lhs, relation, rhs })
    }

    fn constraint_hint_reason(
        &mut self,
    ) -> std::result::Result<Phase9UniverseConstraintHintReason, DecodeError> {
        match self.u8()? {
            0 => Ok(Phase9UniverseConstraintHintReason::KernelDiagnostic),
            1 => Ok(Phase9UniverseConstraintHintReason::RepairCandidate),
            2 => Ok(Phase9UniverseConstraintHintReason::MinimizationExplanation),
            _ => Err(DecodeError::Malformed),
        }
    }

    fn global_ref_list(&mut self) -> std::result::Result<Vec<Phase9AiGlobalRef>, DecodeError> {
        let len = self.u64()?;
        if len > MAX_PHASE9_GLOBAL_REFS {
            return Err(DecodeError::Malformed);
        }
        let len = usize::try_from(len).map_err(|_| DecodeError::Malformed)?;
        let mut refs = Vec::with_capacity(len);
        for _ in 0..len {
            refs.push(self.global_ref()?);
        }
        Ok(refs)
    }

    fn global_ref(&mut self) -> std::result::Result<Phase9AiGlobalRef, DecodeError> {
        Ok(Phase9AiGlobalRef {
            module: self.name()?,
            export_hash: self.hash()?,
            certificate_hash: self.hash()?,
            name: self.name()?,
            decl_interface_hash: self.hash()?,
        })
    }

    fn inductive_family(
        &mut self,
        budget: &mut Phase9InductiveDecodeBudget,
    ) -> std::result::Result<Phase9MachineInductiveFamilyProposal, DecodeError> {
        let name = self.name()?;
        let params = self.telescope_with_cap(MAX_PHASE9_INDUCTIVE_ITEMS, budget)?;
        let indices = self.telescope_with_cap(MAX_PHASE9_INDUCTIVE_ITEMS, budget)?;
        let result_sort = self.level_counted(budget)?;
        let constructor_len = self.u64()?;
        if constructor_len > MAX_PHASE9_INDUCTIVE_ITEMS {
            return Err(DecodeError::Malformed);
        }
        let constructor_len =
            usize::try_from(constructor_len).map_err(|_| DecodeError::Malformed)?;
        let mut constructors = Vec::new();
        for _ in 0..constructor_len {
            constructors.push(Phase9MachineConstructorProposal {
                name: self.name()?,
                ty: self.expr_counted(budget)?,
            });
        }
        Ok(Phase9MachineInductiveFamilyProposal {
            name,
            params,
            indices,
            result_sort,
            constructors,
        })
    }

    fn telescope_with_cap(
        &mut self,
        cap: u64,
        budget: &mut Phase9InductiveDecodeBudget,
    ) -> std::result::Result<Vec<Phase9MachineTelescopeBinder>, DecodeError> {
        let len = self.u64()?;
        if len > cap {
            return Err(DecodeError::Malformed);
        }
        let len = usize::try_from(len).map_err(|_| DecodeError::Malformed)?;
        let mut telescope = Vec::new();
        for _ in 0..len {
            telescope.push(Phase9MachineTelescopeBinder {
                ty: self.expr_counted(budget)?,
            });
        }
        Ok(telescope)
    }

    fn expr_counted(
        &mut self,
        budget: &mut Phase9InductiveDecodeBudget,
    ) -> std::result::Result<Expr, DecodeError> {
        budget.spend_expr()?;
        match self.u8()? {
            0 => Ok(Expr::sort(self.level_counted(budget)?)),
            1 => {
                let index = u32::try_from(self.u64()?).map_err(|_| DecodeError::Malformed)?;
                Ok(Expr::bvar(index))
            }
            2 => {
                let name = self.string()?;
                let levels =
                    self.level_list_with_cap_counted(MAX_PHASE9_INDUCTIVE_ITEMS, budget)?;
                Ok(Expr::konst(name, levels))
            }
            3 => {
                let fun = self.expr_counted(budget)?;
                let arg = self.expr_counted(budget)?;
                Ok(Expr::app(fun, arg))
            }
            4 => {
                let ty = self.expr_counted(budget)?;
                let body = self.expr_counted(budget)?;
                Ok(Expr::lam("_", ty, body))
            }
            5 => {
                let ty = self.expr_counted(budget)?;
                let body = self.expr_counted(budget)?;
                Ok(Expr::pi("_", ty, body))
            }
            6 => {
                let ty = self.expr_counted(budget)?;
                let value = self.expr_counted(budget)?;
                let body = self.expr_counted(budget)?;
                Ok(Expr::let_in("_", ty, value, body))
            }
            _ => Err(DecodeError::Malformed),
        }
    }

    fn level_counted(
        &mut self,
        budget: &mut Phase9InductiveDecodeBudget,
    ) -> std::result::Result<Level, DecodeError> {
        budget.spend_level()?;
        match self.u8()? {
            0 => Ok(Level::Zero),
            1 => Ok(Level::succ(self.level_counted(budget)?)),
            2 => {
                let lhs = self.level_counted(budget)?;
                let rhs = self.level_counted(budget)?;
                Ok(Level::max(lhs, rhs))
            }
            3 => {
                let lhs = self.level_counted(budget)?;
                let rhs = self.level_counted(budget)?;
                Ok(Level::imax(lhs, rhs))
            }
            4 => Ok(Level::param(self.string()?)),
            _ => Err(DecodeError::Malformed),
        }
    }

    fn level_list_with_cap_counted(
        &mut self,
        cap: u64,
        budget: &mut Phase9InductiveDecodeBudget,
    ) -> std::result::Result<Vec<Level>, DecodeError> {
        let len = self.u64()?;
        if len > cap {
            return Err(DecodeError::Malformed);
        }
        let len = usize::try_from(len).map_err(|_| DecodeError::Malformed)?;
        let mut levels = Vec::new();
        for _ in 0..len {
            levels.push(self.level_counted(budget)?);
        }
        Ok(levels)
    }

    fn theorem_graph_query(
        &mut self,
    ) -> std::result::Result<Phase9MachineTheoremGraphQuery, DecodeError> {
        let env_fingerprint = self.hash()?;
        let goal_fingerprint = self.hash()?;
        let goal = self.goal()?;
        let snapshot = self.theorem_graph_snapshot_ref()?;
        let query_features = self.theorem_graph_query_features_ref()?;
        let ranking_profile =
            Phase9TheoremGraphRankingProfile::from_tag(self.u8()?).ok_or(DecodeError::Malformed)?;
        let limit = u32::try_from(self.u64()?).map_err(|_| DecodeError::Malformed)?;
        Ok(Phase9MachineTheoremGraphQuery {
            env_fingerprint,
            goal_fingerprint,
            goal,
            snapshot,
            query_features,
            ranking_profile,
            limit,
        })
    }

    fn theorem_graph_snapshot_ref(
        &mut self,
    ) -> std::result::Result<Phase9MachineTheoremGraphSnapshotRef, DecodeError> {
        let source_release_hash = self.hash()?;
        let extractor_version = Phase9TheoremGraphExtractorVersion::from_tag(self.u8()?)
            .ok_or(DecodeError::Malformed)?;
        let source = match self.u8()? {
            0 => Phase9MachineTheoremGraphSnapshotSource::Inline {
                graph_snapshot_hash: self.hash()?,
                canonical_bytes: self.bytes_with_cap(
                    MAX_PHASE9_THEOREM_GRAPH_SNAPSHOT_BYTES,
                    DecodeError::TheoremGraphSnapshotBytesTooLarge,
                )?,
            },
            1 => Phase9MachineTheoremGraphSnapshotSource::Artifact {
                path: self.string()?,
                file_hash: self.hash()?,
                graph_snapshot_hash: self.hash()?,
                size_bytes: self.u64()?,
            },
            _ => return Err(DecodeError::Malformed),
        };
        Ok(Phase9MachineTheoremGraphSnapshotRef {
            source_release_hash,
            extractor_version,
            source,
        })
    }

    fn theorem_graph_query_features_ref(
        &mut self,
    ) -> std::result::Result<Phase9MachineTheoremGraphQueryFeaturesRef, DecodeError> {
        match self.u8()? {
            0 => Ok(Phase9MachineTheoremGraphQueryFeaturesRef::Inline {
                query_features_hash: self.hash()?,
                canonical_bytes: self.bytes_with_cap(
                    MAX_PHASE9_THEOREM_GRAPH_QUERY_FEATURES_BYTES,
                    DecodeError::TheoremGraphQueryFeaturesBytesTooLarge,
                )?,
            }),
            1 => Ok(Phase9MachineTheoremGraphQueryFeaturesRef::Artifact {
                path: self.string()?,
                file_hash: self.hash()?,
                query_features_hash: self.hash()?,
                size_bytes: self.u64()?,
            }),
            _ => Err(DecodeError::Malformed),
        }
    }

    fn theorem_graph_snapshot(
        &mut self,
    ) -> std::result::Result<Phase9MachineTheoremGraphSnapshot, DecodeError> {
        let source_release_hash = self.hash()?;
        let extractor_version = Phase9TheoremGraphExtractorVersion::from_tag(self.u8()?)
            .ok_or(DecodeError::Malformed)?;
        let node_len = self.u64()?;
        if node_len > MAX_PHASE9_THEOREM_GRAPH_NODES {
            return Err(DecodeError::Malformed);
        }
        let node_len = usize::try_from(node_len).map_err(|_| DecodeError::Malformed)?;
        let mut nodes = Vec::new();
        for _ in 0..node_len {
            nodes.push(self.theorem_graph_node()?);
        }
        let edge_len = self.u64()?;
        if edge_len > MAX_PHASE9_THEOREM_GRAPH_EDGES {
            return Err(DecodeError::Malformed);
        }
        let edge_len = usize::try_from(edge_len).map_err(|_| DecodeError::Malformed)?;
        let mut edges = Vec::new();
        for _ in 0..edge_len {
            edges.push(self.theorem_graph_edge()?);
        }
        Ok(Phase9MachineTheoremGraphSnapshot {
            source_release_hash,
            extractor_version,
            nodes,
            edges,
        })
    }

    fn theorem_graph_query_features(
        &mut self,
    ) -> std::result::Result<Phase9MachineTheoremGraphQueryFeatures, DecodeError> {
        let env_fingerprint = self.hash()?;
        let goal_fingerprint = self.hash()?;
        let feature_schema_version = Phase9TheoremGraphFeatureSchemaVersion::from_tag(self.u8()?)
            .ok_or(DecodeError::Malformed)?;
        let feature_len = self.u64()?;
        if feature_len > MAX_PHASE9_THEOREM_GRAPH_FEATURES {
            return Err(DecodeError::Malformed);
        }
        let feature_len = usize::try_from(feature_len).map_err(|_| DecodeError::Malformed)?;
        let mut features = Vec::new();
        for _ in 0..feature_len {
            features.push(self.theorem_graph_feature()?);
        }
        Ok(Phase9MachineTheoremGraphQueryFeatures {
            env_fingerprint,
            goal_fingerprint,
            feature_schema_version,
            features,
        })
    }

    fn theorem_graph_edge(
        &mut self,
    ) -> std::result::Result<Phase9MachineTheoremGraphEdge, DecodeError> {
        let from = self.theorem_graph_node()?;
        let to = self.theorem_graph_node()?;
        let kind =
            Phase9TheoremGraphEdgeKind::from_tag(self.u8()?).ok_or(DecodeError::Malformed)?;
        Ok(Phase9MachineTheoremGraphEdge { from, to, kind })
    }

    fn theorem_graph_node(
        &mut self,
    ) -> std::result::Result<Phase9MachineTheoremGraphNodeRef, DecodeError> {
        Ok(Phase9MachineTheoremGraphNodeRef {
            module: self.name()?,
            name: self.name()?,
            export_hash: self.hash()?,
            decl_certificate_hash: self.hash()?,
            type_hash: self.hash()?,
            certificate_hash: self.hash()?,
            decl_interface_hash: self.hash()?,
        })
    }

    fn theorem_graph_feature(
        &mut self,
    ) -> std::result::Result<Phase9MachineTheoremGraphFeature, DecodeError> {
        let key = Phase9TheoremGraphFeatureKey {
            namespace_ascii: self.bytes()?,
            name_ascii: self.bytes()?,
        };
        let value = match self.u8()? {
            0 => match self.u8()? {
                0 => Phase9TheoremGraphFeatureValue::Bool(false),
                1 => Phase9TheoremGraphFeatureValue::Bool(true),
                _ => return Err(DecodeError::Malformed),
            },
            1 => Phase9TheoremGraphFeatureValue::I64(self.i64()?),
            2 => Phase9TheoremGraphFeatureValue::Hash(self.hash()?),
            _ => return Err(DecodeError::Malformed),
        };
        Ok(Phase9MachineTheoremGraphFeature { key, value })
    }

    fn skip_theorem_graph_edge(&mut self) -> std::result::Result<(), DecodeError> {
        self.skip_theorem_graph_node()?;
        self.skip_theorem_graph_node()?;
        Phase9TheoremGraphEdgeKind::from_tag(self.u8()?).ok_or(DecodeError::Malformed)?;
        Ok(())
    }

    fn skip_theorem_graph_node(&mut self) -> std::result::Result<(), DecodeError> {
        self.skip_name()?;
        self.skip_name()?;
        self.hash()?;
        self.hash()?;
        self.hash()?;
        self.hash()?;
        self.hash()?;
        Ok(())
    }

    fn skip_theorem_graph_feature(&mut self) -> std::result::Result<(), DecodeError> {
        self.skip_bytes()?;
        self.skip_bytes()?;
        match self.u8()? {
            0 => match self.u8()? {
                0 | 1 => Ok(()),
                _ => Err(DecodeError::Malformed),
            },
            1 => {
                self.i64()?;
                Ok(())
            }
            2 => {
                self.hash()?;
                Ok(())
            }
            _ => Err(DecodeError::Malformed),
        }
    }

    fn skip_name(&mut self) -> std::result::Result<(), DecodeError> {
        let len = self.u64()?;
        if len == 0 || len > MAX_NAME_COMPONENTS {
            return Err(DecodeError::Malformed);
        }
        for _ in 0..len {
            self.skip_string()?;
        }
        Ok(())
    }

    fn option_quotient(
        &mut self,
    ) -> std::result::Result<Option<Phase9QuotientOptions>, DecodeError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(Phase9QuotientOptions {
                setoid: self.global_ref()?,
                setoid_mk: self.global_ref()?,
                setoid_relation: self.global_ref()?,
                rel_equiv: self.global_ref()?,
                quotient: self.global_ref()?,
                quotient_mk: self.global_ref()?,
                quotient_sound: self.global_ref()?,
                quotient_lift: self.global_ref()?,
                eq: self.global_ref()?,
            })),
            _ => Err(DecodeError::Malformed),
        }
    }

    fn option_smt(&mut self) -> std::result::Result<Option<Phase9SmtOptions>, DecodeError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(Phase9SmtOptions {
                eq: self.global_ref()?,
                prop_false: self.option_global_ref()?,
                prop_not: self.option_global_ref()?,
            })),
            _ => Err(DecodeError::Malformed),
        }
    }

    fn option_global_ref(&mut self) -> std::result::Result<Option<Phase9AiGlobalRef>, DecodeError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.global_ref()?)),
            _ => Err(DecodeError::Malformed),
        }
    }

    fn option_formalization(
        &mut self,
    ) -> std::result::Result<Option<Phase9FormalizationOptions>, DecodeError> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(Phase9FormalizationOptions {
                tactic_options_canonical_bytes: self.bytes()?,
                tactic_budget_canonical_bytes: self.bytes()?,
            })),
            _ => Err(DecodeError::Malformed),
        }
    }
}

fn ensure_sorted_global_refs(refs: &[Phase9AiGlobalRef]) -> std::result::Result<(), DecodeError> {
    let mut previous: Option<Vec<u8>> = None;
    for global_ref in refs {
        let key = global_ref_sort_key(global_ref).map_err(|_| DecodeError::Malformed)?;
        if let Some(previous) = previous.as_ref() {
            if previous >= &key {
                return Err(DecodeError::Malformed);
            }
        }
        previous = Some(key);
    }
    Ok(())
}

fn compare_import_identities(
    left: &Phase9ImportIdentity,
    right: &Phase9ImportIdentity,
) -> std::result::Result<Ordering, Phase9AiCanonicalError> {
    Ok(import_identity_sort_key(left)?.cmp(&import_identity_sort_key(right)?))
}

fn import_identity_sort_key(
    import: &Phase9ImportIdentity,
) -> std::result::Result<Vec<u8>, Phase9AiCanonicalError> {
    let mut key = phase5_name_canonical_bytes(&import.module)
        .map_err(|_| Phase9AiCanonicalError::InvalidName)?;
    key.extend_from_slice(&import.export_hash);
    key.extend_from_slice(&import.certificate_hash);
    Ok(key)
}

fn global_ref_sort_key(
    global_ref: &Phase9AiGlobalRef,
) -> std::result::Result<Vec<u8>, Phase9AiCanonicalError> {
    let mut key = phase5_name_canonical_bytes(&global_ref.module)
        .map_err(|_| Phase9AiCanonicalError::InvalidName)?;
    key.extend_from_slice(&global_ref.export_hash);
    key.extend_from_slice(&global_ref.certificate_hash);
    key.extend_from_slice(
        &phase5_name_canonical_bytes(&global_ref.name)
            .map_err(|_| Phase9AiCanonicalError::InvalidName)?,
    );
    key.extend_from_slice(&global_ref.decl_interface_hash);
    Ok(key)
}

fn encode_validation_error_to(out: &mut Vec<u8>, error: Phase9AiValidationError) {
    out.push(error.tag());
}

fn encode_feature_error_option_to(out: &mut Vec<u8>, feature: Option<Phase9AiFeatureError>) {
    match feature {
        Some(feature) => {
            out.push(1);
            encode_feature_error_to(out, feature);
        }
        None => out.push(0),
    }
}

fn encode_feature_error_to(out: &mut Vec<u8>, feature: Phase9AiFeatureError) {
    match feature {
        Phase9AiFeatureError::AdvancedInductive(error) => {
            out.push(0);
            out.push(match error {
                Phase9AdvancedInductiveError::TargetRefMismatch => 0,
                Phase9AdvancedInductiveError::PositivityProfileUnsupported => 1,
                Phase9AdvancedInductiveError::ArtifactGeneratorUnavailable => 2,
                Phase9AdvancedInductiveError::GeneratedArtifactMismatch => 3,
                Phase9AdvancedInductiveError::NameCollision => 4,
            });
        }
        Phase9AiFeatureError::UniverseRepair(error) => {
            out.push(1);
            out.push(match error {
                Phase9UniverseRepairError::UnknownUniverseParam => 0,
                Phase9UniverseRepairError::IllFormedLevelExpr => 1,
                Phase9UniverseRepairError::UnsatisfiedConstraint => 2,
                Phase9UniverseRepairError::NonCanonicalSolution => 3,
                Phase9UniverseRepairError::TargetFingerprintMismatch => 4,
                Phase9UniverseRepairError::InvalidOccurrencePath => 5,
                Phase9UniverseRepairError::AmbiguousOccurrence => 6,
                Phase9UniverseRepairError::TargetRefMismatch => 7,
                Phase9UniverseRepairError::ConstraintHintMismatch => 8,
            });
        }
        Phase9AiFeatureError::TypeclassResolution(error) => {
            out.push(2);
            out.push(match error {
                Phase9TypeclassResolutionError::ClassDeclarationMismatch => 0,
                Phase9TypeclassResolutionError::CandidateInterfaceInvalid => 1,
                Phase9TypeclassResolutionError::ClassHeadUnsupported => 2,
                Phase9TypeclassResolutionError::NoSolution => 3,
            });
        }
        Phase9AiFeatureError::QuotientConstruction(error) => {
            out.push(3);
            out.push(match error {
                Phase9QuotientConstructionError::TargetRefMismatch => 0,
                Phase9QuotientConstructionError::PrimitiveInterfaceMismatch => 1,
                Phase9QuotientConstructionError::UniverseLevelMismatch => 2,
                Phase9QuotientConstructionError::CompatibilityProofMismatch => 3,
                Phase9QuotientConstructionError::QuotientTypeMismatch => 4,
                Phase9QuotientConstructionError::RelationTypeMismatch => 5,
                Phase9QuotientConstructionError::EquivalenceProofMismatch => 6,
                Phase9QuotientConstructionError::RawFunctionTypeMismatch => 7,
            });
        }
        Phase9AiFeatureError::SmtCertificate(error) => {
            out.push(4);
            out.push(match error {
                Phase9SmtCertificateError::EncodingMismatch => 0,
                Phase9SmtCertificateError::RuleFingerprintMismatch => 1,
                Phase9SmtCertificateError::RuleRegistryMismatch => 2,
                Phase9SmtCertificateError::NonCanonicalPayload => 3,
                Phase9SmtCertificateError::ReconstructionProofMismatch => 4,
                Phase9SmtCertificateError::ConclusionEncodingMismatch => 5,
                Phase9SmtCertificateError::PayloadBindingMismatch => 6,
                Phase9SmtCertificateError::ReconstructionConclusionMismatch => 7,
                Phase9SmtCertificateError::ReconstructionPremiseMismatch => 8,
                Phase9SmtCertificateError::PublicInterfaceMismatch => 9,
                Phase9SmtCertificateError::TheoryRefMismatch => 10,
            });
        }
        Phase9AiFeatureError::TheoremGraphQuery(error) => {
            out.push(5);
            out.push(match error {
                Phase9TheoremGraphError::SnapshotMalformed => 0,
                Phase9TheoremGraphError::QueryFeaturesMalformed => 1,
                Phase9TheoremGraphError::NodeResolutionMismatch => 2,
                Phase9TheoremGraphError::LimitOutOfRange => 3,
            });
        }
        Phase9AiFeatureError::Formalization(error) => {
            out.push(6);
            out.push(match error {
                Phase9FormalizationError::IntentRecordMismatch => 0,
                Phase9FormalizationError::CandidateStatementElaborationFailed => 1,
                Phase9FormalizationError::FormalizationProofStatementMismatch => 2,
                Phase9FormalizationError::RejectedIntentHasProofCandidate => 3,
                Phase9FormalizationError::ProofBridgeFailed => 4,
            });
        }
    }
}

fn encode_name_to(
    out: &mut Vec<u8>,
    name: &Name,
) -> std::result::Result<(), Phase9AiCanonicalError> {
    if !name.is_canonical() {
        return Err(Phase9AiCanonicalError::InvalidName);
    }
    encode_len_to(out, name.0.len());
    for component in &name.0 {
        encode_string_to(out, component);
    }
    Ok(())
}

fn encode_option_hash_to(out: &mut Vec<u8>, hash: Option<&Hash>) {
    match hash {
        Some(hash) => {
            out.push(1);
            encode_hash_to(out, hash);
        }
        None => out.push(0),
    }
}

fn encode_hash_to(out: &mut Vec<u8>, hash: &Hash) {
    out.extend_from_slice(hash);
}

fn encode_bytes_to(out: &mut Vec<u8>, bytes: &[u8]) {
    encode_len_to(out, bytes.len());
    out.extend_from_slice(bytes);
}

fn encode_string_to(out: &mut Vec<u8>, value: &str) {
    encode_bytes_to(out, value.as_bytes());
}

fn encode_len_to(out: &mut Vec<u8>, len: usize) {
    encode_u64_to(out, len as u64);
}

fn encode_u64_to(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_be_bytes());
}

fn encode_i64_to(out: &mut Vec<u8>, value: i64) {
    out.extend_from_slice(&value.to_be_bytes());
}

fn hash_with_domain(domain: &str, payload: &[u8]) -> Hash {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(domain.as_bytes());
    bytes.extend_from_slice(payload);
    sha256(&bytes)
}

fn sha256(bytes: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().into()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ArtifactPathError {
    EnvelopeMalformed,
    ArtifactUnavailable,
}

fn validate_artifact_path(
    workspace_root: &Path,
    path: &str,
) -> std::result::Result<PathBuf, ArtifactPathError> {
    if path.is_empty() || path.as_bytes().contains(&0) {
        return Err(ArtifactPathError::EnvelopeMalformed);
    }
    if path
        .split('/')
        .any(|component| component.is_empty() || component == "." || component == "..")
    {
        return Err(ArtifactPathError::EnvelopeMalformed);
    }
    let relative = Path::new(path);
    if relative.is_absolute() {
        return Err(ArtifactPathError::EnvelopeMalformed);
    }
    for component in relative.components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir
            | Component::ParentDir
            | Component::RootDir
            | Component::Prefix(_) => {
                return Err(ArtifactPathError::EnvelopeMalformed);
            }
        }
    }

    let root = workspace_root
        .canonicalize()
        .map_err(|_| ArtifactPathError::ArtifactUnavailable)?;
    let mut current = root.clone();
    for component in relative.components() {
        let Component::Normal(component) = component else {
            return Err(ArtifactPathError::EnvelopeMalformed);
        };
        current.push(component);
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                let resolved = current
                    .canonicalize()
                    .map_err(|_| ArtifactPathError::ArtifactUnavailable)?;
                if !resolved.starts_with(&root) {
                    return Err(ArtifactPathError::EnvelopeMalformed);
                }
                current = resolved;
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }
    Ok(workspace_root.join(relative))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn hash(byte: u8) -> Hash {
        [byte; 32]
    }

    fn empty_options_bytes() -> Vec<u8> {
        phase9_ai_options_canonical_bytes(&Phase9AiOptions::default()).unwrap()
    }

    fn global_ref(seed: u8) -> Phase9AiGlobalRef {
        Phase9AiGlobalRef {
            module: Name::from_dotted("Std.Prim"),
            export_hash: hash(seed),
            certificate_hash: hash(seed.wrapping_add(1)),
            name: Name::from_dotted(format!("ref{seed}")),
            decl_interface_hash: hash(seed.wrapping_add(2)),
        }
    }

    fn verified_universe_import() -> VerifiedImportRef {
        let module = npa_cert::CoreModule {
            name: Name::from_dotted("Lib"),
            declarations: vec![
                npa_kernel::Decl::Axiom {
                    name: "Lib.T".to_owned(),
                    universe_params: vec!["u".to_owned()],
                    ty: Expr::sort(Level::succ(Level::param("u"))),
                },
                npa_kernel::Decl::Axiom {
                    name: "Lib.F".to_owned(),
                    universe_params: vec!["u".to_owned()],
                    ty: Expr::pi(
                        "A",
                        Expr::sort(Level::param("u")),
                        Expr::sort(Level::param("u")),
                    ),
                },
            ],
        };
        let cert = npa_cert::build_module_cert(module, &[]).unwrap();
        let bytes = npa_cert::encode_module_cert(&cert).unwrap();
        let mut session = npa_cert::VerifierSession::new();
        let verified =
            npa_cert::verify_module_cert(&bytes, &mut session, &npa_cert::AxiomPolicy::normal())
                .unwrap();
        VerifiedImportRef::from_verified_module(&verified).unwrap()
    }

    fn universe_global_ref_for(import: &VerifiedImportRef, name: &str) -> Phase9AiGlobalRef {
        let export = import
            .exports()
            .iter()
            .find(|export| export.name == Name::from_dotted(name))
            .unwrap();
        Phase9AiGlobalRef {
            module: import.module().clone(),
            export_hash: import.export_hash(),
            certificate_hash: import.certificate_hash(),
            name: export.name.clone(),
            decl_interface_hash: export.decl_interface_hash,
        }
    }

    fn universe_global_ref(import: &VerifiedImportRef) -> Phase9AiGlobalRef {
        universe_global_ref_for(import, "Lib.T")
    }

    fn universe_target_expr() -> Expr {
        Expr::konst("Lib.T", vec![Level::param("u")])
    }

    fn verified_theorem_graph_import() -> VerifiedImportRef {
        let module = npa_cert::CoreModule {
            name: Name::from_dotted("GraphLib"),
            declarations: vec![
                npa_kernel::Decl::Axiom {
                    name: "GraphLib.P".to_owned(),
                    universe_params: Vec::new(),
                    ty: Expr::sort(Level::succ(Level::zero())),
                },
                npa_kernel::Decl::Def {
                    name: "GraphLib.Type0".to_owned(),
                    universe_params: Vec::new(),
                    ty: Expr::sort(Level::succ(Level::zero())),
                    value: Expr::sort(Level::zero()),
                    reducibility: npa_kernel::Reducibility::Reducible,
                },
            ],
        };
        let cert = npa_cert::build_module_cert(module, &[]).unwrap();
        let bytes = npa_cert::encode_module_cert(&cert).unwrap();
        let mut session = npa_cert::VerifierSession::new();
        let verified =
            npa_cert::verify_module_cert(&bytes, &mut session, &npa_cert::AxiomPolicy::normal())
                .unwrap();
        VerifiedImportRef::from_verified_module(&verified).unwrap()
    }

    fn theorem_graph_node(
        import: &VerifiedImportRef,
        name: &str,
    ) -> Phase9MachineTheoremGraphNodeRef {
        let export = import
            .exports()
            .iter()
            .find(|export| export.name == Name::from_dotted(name))
            .unwrap();
        let decl = import
            .verified_module()
            .declarations()
            .iter()
            .find(|decl| decl.hashes.decl_interface_hash == export.decl_interface_hash)
            .unwrap();
        Phase9MachineTheoremGraphNodeRef {
            module: import.module().clone(),
            name: export.name.clone(),
            export_hash: import.export_hash(),
            decl_certificate_hash: decl.hashes.decl_certificate_hash,
            type_hash: export.type_hash,
            certificate_hash: import.certificate_hash(),
            decl_interface_hash: export.decl_interface_hash,
        }
    }

    fn missing_theorem_graph_node() -> Phase9MachineTheoremGraphNodeRef {
        Phase9MachineTheoremGraphNodeRef {
            module: Name::from_dotted("Missing"),
            name: Name::from_dotted("Missing.P"),
            export_hash: hash(31),
            decl_certificate_hash: hash(32),
            type_hash: hash(33),
            certificate_hash: hash(34),
            decl_interface_hash: hash(35),
        }
    }

    fn theorem_graph_goal() -> Phase9AiGoal {
        Phase9AiGoal {
            universe_params: Vec::new(),
            local_context: Vec::new(),
            target: Expr::sort(Level::zero()),
        }
    }

    fn theorem_graph_features(
        env_fingerprint: Hash,
        goal_fingerprint: Hash,
    ) -> Phase9MachineTheoremGraphQueryFeatures {
        Phase9MachineTheoremGraphQueryFeatures {
            env_fingerprint,
            goal_fingerprint,
            feature_schema_version: Phase9TheoremGraphFeatureSchemaVersion::MvpGoalFeaturesV1,
            features: Vec::new(),
        }
    }

    fn theorem_graph_snapshot(
        source_release_hash: Hash,
        mut nodes: Vec<Phase9MachineTheoremGraphNodeRef>,
    ) -> Phase9MachineTheoremGraphSnapshot {
        nodes.sort_by_key(phase9_theorem_graph_node_identity_key);
        Phase9MachineTheoremGraphSnapshot {
            source_release_hash,
            extractor_version: Phase9TheoremGraphExtractorVersion::MvpCertificateGraphV1,
            nodes,
            edges: Vec::new(),
        }
    }

    fn theorem_graph_snapshot_bytes_with_noncanonical_node_name(
        source_release_hash: Hash,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        encode_hash_to(&mut bytes, &source_release_hash);
        bytes.push(Phase9TheoremGraphExtractorVersion::MvpCertificateGraphV1.tag());
        encode_u64_to(&mut bytes, 1);
        encode_u64_to(&mut bytes, 1);
        encode_bytes_to(&mut bytes, b"");
        encode_name_to(&mut bytes, &Name::from_dotted("GraphLib.P")).unwrap();
        for seed in 51..=55 {
            encode_hash_to(&mut bytes, &hash(seed));
        }
        encode_u64_to(&mut bytes, 0);
        bytes
    }

    fn theorem_graph_inline_query_request(
        import: &VerifiedImportRef,
        snapshot_hash_override: Option<Hash>,
        query_features_hash_override: Option<Hash>,
        snapshot: Phase9MachineTheoremGraphSnapshot,
        query_features_override: Option<Phase9MachineTheoremGraphQueryFeatures>,
        limit: u32,
    ) -> Vec<u8> {
        let options_bytes = empty_options_bytes();
        let options_hash = phase9_ai_options_hash(&options_bytes);
        let imports = vec![Phase9ImportIdentity::from_verified_import(import)];
        let env_fingerprint = phase9_ai_env_fingerprint(
            Phase9AiProfileVersion::MvpV1,
            Phase9AiTaskKind::TheoremGraphQuery,
            &imports,
            options_hash,
        )
        .unwrap();
        let goal = theorem_graph_goal();
        let goal_fingerprint = phase9_ai_goal_fingerprint(env_fingerprint, &goal);
        let snapshot_bytes = phase9_theorem_graph_snapshot_canonical_bytes(&snapshot).unwrap();
        let snapshot_hash = snapshot_hash_override
            .unwrap_or_else(|| phase9_theorem_graph_snapshot_hash(&snapshot_bytes).unwrap());
        let query_features = query_features_override
            .unwrap_or_else(|| theorem_graph_features(env_fingerprint, goal_fingerprint));
        let query_features_bytes =
            phase9_theorem_graph_query_features_canonical_bytes(&query_features).unwrap();
        let query_features_hash = query_features_hash_override.unwrap_or_else(|| {
            phase9_theorem_graph_query_features_hash(&query_features_bytes).unwrap()
        });
        let query = Phase9MachineTheoremGraphQuery {
            env_fingerprint,
            goal_fingerprint,
            goal,
            snapshot: Phase9MachineTheoremGraphSnapshotRef {
                source_release_hash: snapshot.source_release_hash,
                extractor_version: Phase9TheoremGraphExtractorVersion::MvpCertificateGraphV1,
                source: Phase9MachineTheoremGraphSnapshotSource::Inline {
                    graph_snapshot_hash: snapshot_hash,
                    canonical_bytes: snapshot_bytes,
                },
            },
            query_features: Phase9MachineTheoremGraphQueryFeaturesRef::Inline {
                query_features_hash,
                canonical_bytes: query_features_bytes,
            },
            ranking_profile: Phase9TheoremGraphRankingProfile::MvpTupleOrder,
            limit,
        };
        let envelope = Phase9AiCandidateEnvelope {
            profile_version: Phase9AiProfileVersion::MvpV1,
            task_kind: Phase9AiTaskKind::TheoremGraphQuery,
            target: Phase9AiTarget {
                env_fingerprint,
                target_decl_hash: None,
                goal_fingerprint: Some(goal_fingerprint),
            },
            imports,
            options: Phase9AiOptionsRef::Inline {
                options_hash,
                canonical_bytes: options_bytes,
            },
            payload: phase9_theorem_graph_query_canonical_bytes(&query).unwrap(),
        };
        phase9_ai_candidate_envelope_canonical_bytes(&envelope).unwrap()
    }

    fn phase9_unary_expr() -> Expr {
        Expr::konst("Unary", vec![])
    }

    fn valid_inductive_proposal() -> Phase9MachineInductiveProposal {
        Phase9MachineInductiveProposal {
            block_name: None,
            expected_decl_hash: None,
            universe_params: Vec::new(),
            inductives: vec![Phase9MachineInductiveFamilyProposal {
                name: Name::from_dotted("Unary"),
                params: Vec::new(),
                indices: Vec::new(),
                result_sort: Level::succ(Level::zero()),
                constructors: vec![
                    Phase9MachineConstructorProposal {
                        name: Name::from_dotted("zero"),
                        ty: phase9_unary_expr(),
                    },
                    Phase9MachineConstructorProposal {
                        name: Name::from_dotted("succ"),
                        ty: Expr::pi("_", phase9_unary_expr(), phase9_unary_expr()),
                    },
                ],
            }],
        }
    }

    fn inductive_request(proposal: Phase9MachineInductiveProposal) -> Vec<u8> {
        inductive_request_with_imports(proposal, Vec::new())
    }

    fn inductive_request_with_imports(
        proposal: Phase9MachineInductiveProposal,
        verified_imports: Vec<&VerifiedImportRef>,
    ) -> Vec<u8> {
        let options_bytes = empty_options_bytes();
        let options_hash = phase9_ai_options_hash(&options_bytes);
        let imports = verified_imports
            .iter()
            .map(|import| Phase9ImportIdentity::from_verified_import(import))
            .collect::<Vec<_>>();
        let envelope = Phase9AiCandidateEnvelope {
            profile_version: Phase9AiProfileVersion::MvpV1,
            task_kind: Phase9AiTaskKind::AdvancedInductive,
            target: Phase9AiTarget {
                env_fingerprint: phase9_ai_env_fingerprint(
                    Phase9AiProfileVersion::MvpV1,
                    Phase9AiTaskKind::AdvancedInductive,
                    &imports,
                    options_hash,
                )
                .unwrap(),
                target_decl_hash: None,
                goal_fingerprint: None,
            },
            imports,
            options: Phase9AiOptionsRef::Inline {
                options_hash,
                canonical_bytes: options_bytes,
            },
            payload: phase9_inductive_proposal_canonical_bytes(&proposal).unwrap(),
        };
        phase9_ai_candidate_envelope_canonical_bytes(&envelope).unwrap()
    }

    fn universe_goal(target: Expr) -> Phase9AiGoal {
        Phase9AiGoal {
            universe_params: vec!["u".to_owned()],
            local_context: Vec::new(),
            target,
        }
    }

    fn valid_universe_candidate(import: &VerifiedImportRef) -> Phase9UniverseRepairCandidate {
        let target = universe_target_expr();
        Phase9UniverseRepairCandidate {
            goal: Some(universe_goal(target.clone())),
            target_expr: target,
            instantiations: vec![Phase9UniverseInstantiationPatch {
                occurrence: Phase9MachineExprOccurrence {
                    path: Vec::new(),
                    expected_ref: universe_global_ref(import),
                },
                explicit_level_args: vec![Level::succ(Level::param("u"))],
            }],
            constraint_hints: Vec::new(),
            minimization_hint: None,
        }
    }

    fn universe_request_with_target(
        import: &VerifiedImportRef,
        candidate: Phase9UniverseRepairCandidate,
        target_decl_hash: Option<Hash>,
        goal_fingerprint: Option<Hash>,
    ) -> Vec<u8> {
        let options_bytes = empty_options_bytes();
        let options_hash = phase9_ai_options_hash(&options_bytes);
        let imports = vec![Phase9ImportIdentity::from_verified_import(import)];
        let env_fingerprint = phase9_ai_env_fingerprint(
            Phase9AiProfileVersion::MvpV1,
            Phase9AiTaskKind::UniverseRepair,
            &imports,
            options_hash,
        )
        .unwrap();
        let goal_fingerprint = if target_decl_hash.is_some() {
            goal_fingerprint
        } else {
            goal_fingerprint.or_else(|| {
                candidate
                    .goal
                    .as_ref()
                    .map(|goal| phase9_ai_goal_fingerprint(env_fingerprint, goal))
            })
        };
        let payload = phase9_universe_repair_candidate_canonical_bytes(&candidate).unwrap();
        let envelope = Phase9AiCandidateEnvelope {
            profile_version: Phase9AiProfileVersion::MvpV1,
            task_kind: Phase9AiTaskKind::UniverseRepair,
            target: Phase9AiTarget {
                env_fingerprint,
                target_decl_hash,
                goal_fingerprint,
            },
            imports,
            options: Phase9AiOptionsRef::Inline {
                options_hash,
                canonical_bytes: options_bytes,
            },
            payload,
        };
        phase9_ai_candidate_envelope_canonical_bytes(&envelope).unwrap()
    }

    fn valid_universe_request(import: &VerifiedImportRef) -> Vec<u8> {
        universe_request_with_target(import, valid_universe_candidate(import), None, None)
    }

    fn target_for(
        task_kind: Phase9AiTaskKind,
        imports: &[Phase9ImportIdentity],
        options_hash: Hash,
        goal_fingerprint: Option<Hash>,
    ) -> Phase9AiTarget {
        Phase9AiTarget {
            env_fingerprint: phase9_ai_env_fingerprint(
                Phase9AiProfileVersion::MvpV1,
                task_kind,
                imports,
                options_hash,
            )
            .unwrap(),
            target_decl_hash: None,
            goal_fingerprint,
        }
    }

    fn inline_request(
        task_kind: Phase9AiTaskKind,
        options_bytes: Vec<u8>,
        imports: Vec<Phase9ImportIdentity>,
        goal_fingerprint: Option<Hash>,
    ) -> Vec<u8> {
        let options_hash = phase9_ai_options_hash(&options_bytes);
        let envelope = Phase9AiCandidateEnvelope {
            profile_version: Phase9AiProfileVersion::MvpV1,
            task_kind,
            target: target_for(task_kind, &imports, options_hash, goal_fingerprint),
            imports,
            options: Phase9AiOptionsRef::Inline {
                options_hash,
                canonical_bytes: options_bytes,
            },
            payload: b"opaque-payload".to_vec(),
        };
        phase9_ai_candidate_envelope_canonical_bytes(&envelope).unwrap()
    }

    fn workspace_root() -> PathBuf {
        std::env::current_dir().unwrap()
    }

    fn assert_rejected(
        response: Phase9AiEndpointResponse,
        expected_error: Phase9AiValidationError,
        expected_feature_error: Option<Phase9AiFeatureError>,
    ) -> Hash {
        match response {
            Phase9AiEndpointResponse::Rejected {
                candidate_hash,
                validation_result_hash,
                error,
                feature_error,
            } => {
                assert_eq!(error, expected_error);
                assert_eq!(feature_error, expected_feature_error);
                assert_eq!(
                    validation_result_hash,
                    phase9_ai_validation_result_hash_for_rejection(
                        candidate_hash,
                        error,
                        feature_error
                    )
                );
                candidate_hash
            }
            other => panic!("expected rejected response, got {other:?}"),
        }
    }

    #[test]
    fn common_candidate_hash_is_available_when_options_decode_fails() {
        let request = inline_request(
            Phase9AiTaskKind::AdvancedInductive,
            b"not-options".to_vec(),
            Vec::new(),
            None,
        );
        let expected_candidate_hash = phase9_ai_candidate_hash(&request);

        let candidate_hash = assert_rejected(
            run_phase9_inductive_check_request(&request, &[], &workspace_root()),
            Phase9AiValidationError::EnvelopeMalformed,
            None,
        );

        assert_eq!(candidate_hash, expected_candidate_hash);
    }

    #[test]
    fn top_level_decode_failure_is_endpoint_error_without_candidate_hash() {
        assert_eq!(
            run_phase9_inductive_check_request(b"not-an-envelope", &[], &workspace_root()),
            Phase9AiEndpointResponse::Error {
                error: Phase9AiEndpointError::NonCanonicalRequestBytes
            }
        );
    }

    #[test]
    fn options_hash_mismatch_is_payload_hash_mismatch() {
        let options_bytes = empty_options_bytes();
        let envelope = Phase9AiCandidateEnvelope {
            profile_version: Phase9AiProfileVersion::MvpV1,
            task_kind: Phase9AiTaskKind::AdvancedInductive,
            target: target_for(Phase9AiTaskKind::AdvancedInductive, &[], hash(9), None),
            imports: Vec::new(),
            options: Phase9AiOptionsRef::Inline {
                options_hash: hash(9),
                canonical_bytes: options_bytes,
            },
            payload: Vec::new(),
        };
        let request = phase9_ai_candidate_envelope_canonical_bytes(&envelope).unwrap();

        assert_rejected(
            run_phase9_inductive_check_request(&request, &[], &workspace_root()),
            Phase9AiValidationError::PayloadHashMismatch,
            None,
        );
    }

    #[test]
    fn quotient_options_round_trip_named_primitive_refs() {
        let options = Phase9AiOptions {
            quotient: Some(Phase9QuotientOptions {
                setoid: global_ref(1),
                setoid_mk: global_ref(4),
                setoid_relation: global_ref(7),
                rel_equiv: global_ref(10),
                quotient: global_ref(13),
                quotient_mk: global_ref(16),
                quotient_sound: global_ref(19),
                quotient_lift: global_ref(22),
                eq: global_ref(25),
            }),
            ..Default::default()
        };
        let bytes = phase9_ai_options_canonical_bytes(&options).unwrap();

        assert_eq!(decode_options(&bytes).unwrap(), options);

        let mut changed = options.clone();
        changed.quotient.as_mut().unwrap().eq.decl_interface_hash = hash(99);
        assert_ne!(phase9_ai_options_canonical_bytes(&changed).unwrap(), bytes);
    }

    #[test]
    fn formalization_options_preserve_nested_phase4_bytes() {
        let options = Phase9AiOptions {
            formalization: Some(Phase9FormalizationOptions {
                tactic_options_canonical_bytes: b"phase4-options".to_vec(),
                tactic_budget_canonical_bytes: b"phase4-budget".to_vec(),
            }),
            ..Default::default()
        };
        let bytes = phase9_ai_options_canonical_bytes(&options).unwrap();

        assert_eq!(decode_options(&bytes).unwrap(), options);

        let mut changed = options.clone();
        changed
            .formalization
            .as_mut()
            .unwrap()
            .tactic_budget_canonical_bytes
            .push(0);
        assert_ne!(phase9_ai_options_canonical_bytes(&changed).unwrap(), bytes);
    }

    #[test]
    fn phase9_domain_hashes_use_documented_tag_concatenation() {
        let payload = b"payload";
        let mut expected = Vec::new();
        expected.extend_from_slice(CANDIDATE_HASH_TAG.as_bytes());
        expected.extend_from_slice(payload);

        assert_eq!(phase9_ai_candidate_hash(payload), sha256(&expected));
    }

    #[test]
    fn artifact_hash_and_size_mismatch_is_candidate_rejection() {
        let root = std::env::temp_dir().join(format!("npa-phase9-m1-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("options.bin"), empty_options_bytes()).unwrap();
        let envelope = Phase9AiCandidateEnvelope {
            profile_version: Phase9AiProfileVersion::MvpV1,
            task_kind: Phase9AiTaskKind::AdvancedInductive,
            target: Phase9AiTarget {
                env_fingerprint: hash(1),
                target_decl_hash: None,
                goal_fingerprint: None,
            },
            imports: Vec::new(),
            options: Phase9AiOptionsRef::Artifact {
                path: "options.bin".to_owned(),
                file_hash: hash(2),
                options_hash: phase9_ai_options_hash(&empty_options_bytes()),
                size_bytes: empty_options_bytes().len() as u64,
            },
            payload: Vec::new(),
        };
        let request = phase9_ai_candidate_envelope_canonical_bytes(&envelope).unwrap();

        assert_rejected(
            run_phase9_inductive_check_request(&request, &[], &root),
            Phase9AiValidationError::PayloadHashMismatch,
            None,
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn artifact_path_shape_failure_is_candidate_rejection() {
        let envelope = Phase9AiCandidateEnvelope {
            profile_version: Phase9AiProfileVersion::MvpV1,
            task_kind: Phase9AiTaskKind::AdvancedInductive,
            target: Phase9AiTarget {
                env_fingerprint: hash(1),
                target_decl_hash: None,
                goal_fingerprint: None,
            },
            imports: Vec::new(),
            options: Phase9AiOptionsRef::Artifact {
                path: "../options.bin".to_owned(),
                file_hash: hash(2),
                options_hash: phase9_ai_options_hash(&empty_options_bytes()),
                size_bytes: empty_options_bytes().len() as u64,
            },
            payload: Vec::new(),
        };
        let request = phase9_ai_candidate_envelope_canonical_bytes(&envelope).unwrap();

        assert_rejected(
            run_phase9_inductive_check_request(&request, &[], &workspace_root()),
            Phase9AiValidationError::EnvelopeMalformed,
            None,
        );
    }

    #[cfg(unix)]
    #[test]
    fn artifact_symlink_escape_is_candidate_rejection() {
        let root =
            std::env::temp_dir().join(format!("npa-phase9-symlink-root-{}", std::process::id()));
        let outside =
            std::env::temp_dir().join(format!("npa-phase9-symlink-outside-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(&outside, empty_options_bytes()).unwrap();
        std::os::unix::fs::symlink(&outside, root.join("escaped-options.bin")).unwrap();
        let envelope = Phase9AiCandidateEnvelope {
            profile_version: Phase9AiProfileVersion::MvpV1,
            task_kind: Phase9AiTaskKind::AdvancedInductive,
            target: Phase9AiTarget {
                env_fingerprint: hash(1),
                target_decl_hash: None,
                goal_fingerprint: None,
            },
            imports: Vec::new(),
            options: Phase9AiOptionsRef::Artifact {
                path: "escaped-options.bin".to_owned(),
                file_hash: phase9_file_hash(&empty_options_bytes()),
                options_hash: phase9_ai_options_hash(&empty_options_bytes()),
                size_bytes: empty_options_bytes().len() as u64,
            },
            payload: Vec::new(),
        };
        let request = phase9_ai_candidate_envelope_canonical_bytes(&envelope).unwrap();

        assert_rejected(
            run_phase9_inductive_check_request(&request, &[], &root),
            Phase9AiValidationError::EnvelopeMalformed,
            None,
        );
        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_file(outside);
    }

    #[test]
    fn duplicate_import_identity_is_import_closure_mismatch() {
        let import = Phase9ImportIdentity {
            module: Name::from_dotted("A"),
            export_hash: hash(1),
            certificate_hash: hash(2),
        };
        let request = inline_request(
            Phase9AiTaskKind::AdvancedInductive,
            empty_options_bytes(),
            vec![import.clone(), import],
            None,
        );

        assert_rejected(
            run_phase9_inductive_check_request(&request, &[], &workspace_root()),
            Phase9AiValidationError::ImportClosureMismatch,
            None,
        );
    }

    #[test]
    fn import_sort_order_uses_phase5_name_canonical_bytes() {
        let import_b = Phase9ImportIdentity {
            module: Name::from_dotted("B"),
            export_hash: hash(1),
            certificate_hash: hash(2),
        };
        let import_aa = Phase9ImportIdentity {
            module: Name::from_dotted("AA"),
            export_hash: hash(3),
            certificate_hash: hash(4),
        };
        assert_eq!(
            compare_import_identities(&import_b, &import_aa).unwrap(),
            Ordering::Less
        );
        let request = inline_request(
            Phase9AiTaskKind::AdvancedInductive,
            empty_options_bytes(),
            vec![import_aa, import_b],
            None,
        );

        assert_rejected(
            run_phase9_inductive_check_request(&request, &[], &workspace_root()),
            Phase9AiValidationError::EnvelopeMalformed,
            None,
        );
    }

    #[test]
    fn env_fingerprint_mismatch_is_target_fingerprint_mismatch() {
        let mut request = decode_candidate_envelope(&inline_request(
            Phase9AiTaskKind::AdvancedInductive,
            empty_options_bytes(),
            Vec::new(),
            None,
        ))
        .unwrap();
        request.target.env_fingerprint = hash(7);
        let request = phase9_ai_candidate_envelope_canonical_bytes(&request).unwrap();

        assert_rejected(
            run_phase9_inductive_check_request(&request, &[], &workspace_root()),
            Phase9AiValidationError::TargetFingerprintMismatch,
            None,
        );
    }

    #[test]
    fn advanced_inductive_valid_candidate_returns_decl_hashes() {
        let request = inductive_request(valid_inductive_proposal());
        let expected_candidate_hash = phase9_ai_candidate_hash(&request);

        let response = run_phase9_inductive_check_request(&request, &[], &workspace_root());

        let Phase9AiEndpointResponse::Success {
            candidate_hash,
            validation_result_hash,
            payload,
        } = response
        else {
            panic!("expected success response");
        };
        assert_eq!(candidate_hash, expected_candidate_hash);
        let Phase9AiSuccessPayload::AdvancedInductive {
            decl_interface_hash,
            decl_certificate_hash,
        } = *payload
        else {
            panic!("expected advanced inductive payload");
        };
        assert_ne!(decl_interface_hash, [0; 32]);
        assert_ne!(decl_certificate_hash, [0; 32]);
        let expected_payload = Phase9AiSuccessPayload::AdvancedInductive {
            decl_interface_hash,
            decl_certificate_hash,
        };
        assert_eq!(
            validation_result_hash,
            phase9_ai_validation_result_hash_for_success(candidate_hash, &expected_payload)
        );
    }

    #[test]
    fn advanced_inductive_expected_decl_hash_mismatch_is_target_mismatch() {
        let mut proposal = valid_inductive_proposal();
        proposal.expected_decl_hash = Some(hash(77));
        let request = inductive_request(proposal);

        assert_rejected(
            run_phase9_inductive_check_request(&request, &[], &workspace_root()),
            Phase9AiValidationError::TargetFingerprintMismatch,
            None,
        );
    }

    #[test]
    fn advanced_inductive_constructor_result_mismatch_is_target_ref_mismatch() {
        let mut proposal = valid_inductive_proposal();
        proposal.inductives[0].constructors[0].ty = Expr::sort(Level::zero());
        let request = inductive_request(proposal);

        assert_rejected(
            run_phase9_inductive_check_request(&request, &[], &workspace_root()),
            Phase9AiValidationError::FeatureRejected,
            Some(Phase9AiFeatureError::AdvancedInductive(
                Phase9AdvancedInductiveError::TargetRefMismatch,
            )),
        );
    }

    #[test]
    fn advanced_inductive_name_collision_is_feature_rejection() {
        let mut proposal = valid_inductive_proposal();
        proposal.inductives[0].constructors[0].name = Name::from_dotted("rec");
        let request = inductive_request(proposal);

        assert_rejected(
            run_phase9_inductive_check_request(&request, &[], &workspace_root()),
            Phase9AiValidationError::FeatureRejected,
            Some(Phase9AiFeatureError::AdvancedInductive(
                Phase9AdvancedInductiveError::NameCollision,
            )),
        );
    }

    #[test]
    fn advanced_inductive_bad_positivity_is_unsupported() {
        let mut proposal = valid_inductive_proposal();
        proposal.inductives[0]
            .constructors
            .push(Phase9MachineConstructorProposal {
                name: Name::from_dotted("bad"),
                ty: Expr::pi(
                    "_",
                    Expr::pi("_", phase9_unary_expr(), phase9_unary_expr()),
                    phase9_unary_expr(),
                ),
            });
        let request = inductive_request(proposal);

        assert_rejected(
            run_phase9_inductive_check_request(&request, &[], &workspace_root()),
            Phase9AiValidationError::UnsupportedFeature,
            Some(Phase9AiFeatureError::AdvancedInductive(
                Phase9AdvancedInductiveError::PositivityProfileUnsupported,
            )),
        );
    }

    #[test]
    fn advanced_inductive_nested_recursive_occurrence_is_unsupported() {
        let import = verified_universe_import();
        let mut proposal = valid_inductive_proposal();
        proposal.inductives[0]
            .constructors
            .push(Phase9MachineConstructorProposal {
                name: Name::from_dotted("boxed"),
                ty: Expr::pi(
                    "_",
                    Expr::app(
                        Expr::konst("Lib.F", vec![Level::succ(Level::zero())]),
                        phase9_unary_expr(),
                    ),
                    phase9_unary_expr(),
                ),
            });
        let request = inductive_request_with_imports(proposal, vec![&import]);

        assert_rejected(
            run_phase9_inductive_check_request(
                &request,
                std::slice::from_ref(&import),
                &workspace_root(),
            ),
            Phase9AiValidationError::UnsupportedFeature,
            Some(Phase9AiFeatureError::AdvancedInductive(
                Phase9AdvancedInductiveError::PositivityProfileUnsupported,
            )),
        );
    }

    #[test]
    fn advanced_inductive_mutual_block_is_unsupported_before_name_checks() {
        let mut proposal = valid_inductive_proposal();
        let mut second = proposal.inductives[0].clone();
        second.name = Name::from_dotted("Unary");
        proposal.inductives.push(second);
        let request = inductive_request(proposal);

        assert_rejected(
            run_phase9_inductive_check_request(&request, &[], &workspace_root()),
            Phase9AiValidationError::UnsupportedFeature,
            Some(Phase9AiFeatureError::AdvancedInductive(
                Phase9AdvancedInductiveError::PositivityProfileUnsupported,
            )),
        );
    }

    #[test]
    fn advanced_inductive_indexed_family_result_check_runs_before_generator_rejection() {
        let proposal = Phase9MachineInductiveProposal {
            block_name: None,
            expected_decl_hash: None,
            universe_params: Vec::new(),
            inductives: vec![Phase9MachineInductiveFamilyProposal {
                name: Name::from_dotted("Ix"),
                params: Vec::new(),
                indices: vec![Phase9MachineTelescopeBinder {
                    ty: Expr::sort(Level::zero()),
                }],
                result_sort: Level::succ(Level::zero()),
                constructors: vec![Phase9MachineConstructorProposal {
                    name: Name::from_dotted("mk"),
                    ty: Expr::pi(
                        "_",
                        Expr::sort(Level::zero()),
                        Expr::app(Expr::konst("Ix", vec![]), Expr::bvar(0)),
                    ),
                }],
            }],
        };
        let request = inductive_request(proposal);

        assert_rejected(
            run_phase9_inductive_check_request(&request, &[], &workspace_root()),
            Phase9AiValidationError::UnsupportedFeature,
            Some(Phase9AiFeatureError::AdvancedInductive(
                Phase9AdvancedInductiveError::PositivityProfileUnsupported,
            )),
        );
    }

    #[test]
    fn theorem_graph_query_returns_only_resolved_public_axiom_nodes_with_zero_score() {
        let import = verified_theorem_graph_import();
        let eligible = theorem_graph_node(&import, "GraphLib.P");
        let ineligible = theorem_graph_node(&import, "GraphLib.Type0");
        let missing = missing_theorem_graph_node();
        let snapshot =
            theorem_graph_snapshot(hash(41), vec![ineligible, missing, eligible.clone()]);
        let request = theorem_graph_inline_query_request(&import, None, None, snapshot, None, 16);

        let response = run_phase9_theorem_graph_query_request(
            &request,
            std::slice::from_ref(&import),
            &workspace_root(),
        );

        let Phase9AiEndpointResponse::Success { payload, .. } = response else {
            panic!("expected theorem graph success");
        };
        let Phase9AiSuccessPayload::TheoremGraphQuery { result } = *payload else {
            panic!("expected theorem graph payload");
        };
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].node, eligible);
        assert_eq!(result.entries[0].score.score_microunits, 0);
    }

    #[test]
    fn theorem_graph_snapshot_hash_mismatch_is_payload_hash_mismatch() {
        let import = verified_theorem_graph_import();
        let snapshot =
            theorem_graph_snapshot(hash(41), vec![theorem_graph_node(&import, "GraphLib.P")]);
        let request =
            theorem_graph_inline_query_request(&import, Some(hash(99)), None, snapshot, None, 16);

        assert_rejected(
            run_phase9_theorem_graph_query_request(
                &request,
                std::slice::from_ref(&import),
                &workspace_root(),
            ),
            Phase9AiValidationError::PayloadHashMismatch,
            None,
        );
    }

    #[test]
    fn theorem_graph_query_features_hash_mismatch_is_payload_hash_mismatch() {
        let import = verified_theorem_graph_import();
        let snapshot =
            theorem_graph_snapshot(hash(41), vec![theorem_graph_node(&import, "GraphLib.P")]);
        let request =
            theorem_graph_inline_query_request(&import, None, Some(hash(98)), snapshot, None, 16);

        assert_rejected(
            run_phase9_theorem_graph_query_request(
                &request,
                std::slice::from_ref(&import),
                &workspace_root(),
            ),
            Phase9AiValidationError::PayloadHashMismatch,
            None,
        );
    }

    #[test]
    fn theorem_graph_snapshot_metadata_mismatch_is_snapshot_malformed() {
        let import = verified_theorem_graph_import();
        let snapshot =
            theorem_graph_snapshot(hash(41), vec![theorem_graph_node(&import, "GraphLib.P")]);
        let mut request = decode_candidate_envelope(&theorem_graph_inline_query_request(
            &import, None, None, snapshot, None, 16,
        ))
        .unwrap();
        let mut query = decode_theorem_graph_query(&request.payload).unwrap();
        query.snapshot.source_release_hash = hash(42);
        request.payload = phase9_theorem_graph_query_canonical_bytes(&query).unwrap();
        let request = phase9_ai_candidate_envelope_canonical_bytes(&request).unwrap();

        assert_rejected(
            run_phase9_theorem_graph_query_request(
                &request,
                std::slice::from_ref(&import),
                &workspace_root(),
            ),
            Phase9AiValidationError::EnvelopeMalformed,
            Some(Phase9AiFeatureError::TheoremGraphQuery(
                Phase9TheoremGraphError::SnapshotMalformed,
            )),
        );
    }

    #[test]
    fn theorem_graph_query_features_metadata_mismatch_is_query_features_malformed() {
        let import = verified_theorem_graph_import();
        let snapshot =
            theorem_graph_snapshot(hash(41), vec![theorem_graph_node(&import, "GraphLib.P")]);
        let request_base =
            theorem_graph_inline_query_request(&import, None, None, snapshot, None, 16);
        let envelope = decode_candidate_envelope(&request_base).unwrap();
        let query = decode_theorem_graph_query(&envelope.payload).unwrap();
        let bad_features = theorem_graph_features(query.env_fingerprint, hash(77));
        let request = theorem_graph_inline_query_request(
            &import,
            None,
            None,
            decode_theorem_graph_snapshot(match &query.snapshot.source {
                Phase9MachineTheoremGraphSnapshotSource::Inline {
                    canonical_bytes, ..
                } => canonical_bytes,
                Phase9MachineTheoremGraphSnapshotSource::Artifact { .. } => unreachable!(),
            })
            .unwrap(),
            Some(bad_features),
            16,
        );

        assert_rejected(
            run_phase9_theorem_graph_query_request(
                &request,
                std::slice::from_ref(&import),
                &workspace_root(),
            ),
            Phase9AiValidationError::EnvelopeMalformed,
            Some(Phase9AiFeatureError::TheoremGraphQuery(
                Phase9TheoremGraphError::QueryFeaturesMalformed,
            )),
        );
    }

    #[test]
    fn theorem_graph_node_hash_mismatch_is_node_resolution_mismatch() {
        let import = verified_theorem_graph_import();
        let mut node = theorem_graph_node(&import, "GraphLib.P");
        node.type_hash = hash(97);
        let snapshot = theorem_graph_snapshot(hash(41), vec![node]);
        let request = theorem_graph_inline_query_request(&import, None, None, snapshot, None, 16);

        assert_rejected(
            run_phase9_theorem_graph_query_request(
                &request,
                std::slice::from_ref(&import),
                &workspace_root(),
            ),
            Phase9AiValidationError::FeatureRejected,
            Some(Phase9AiFeatureError::TheoremGraphQuery(
                Phase9TheoremGraphError::NodeResolutionMismatch,
            )),
        );
    }

    #[test]
    fn theorem_graph_limit_is_checked_before_artifact_hashes() {
        let import = verified_theorem_graph_import();
        let snapshot =
            theorem_graph_snapshot(hash(41), vec![theorem_graph_node(&import, "GraphLib.P")]);
        let request =
            theorem_graph_inline_query_request(&import, Some(hash(99)), None, snapshot, None, 257);

        assert_rejected(
            run_phase9_theorem_graph_query_request(
                &request,
                std::slice::from_ref(&import),
                &workspace_root(),
            ),
            Phase9AiValidationError::EnvelopeMalformed,
            Some(Phase9AiFeatureError::TheoremGraphQuery(
                Phase9TheoremGraphError::LimitOutOfRange,
            )),
        );
    }

    #[test]
    fn theorem_graph_inline_snapshot_raw_cap_is_snapshot_malformed() {
        let import = verified_theorem_graph_import();
        let snapshot =
            theorem_graph_snapshot(hash(41), vec![theorem_graph_node(&import, "GraphLib.P")]);
        let mut envelope = decode_candidate_envelope(&theorem_graph_inline_query_request(
            &import, None, None, snapshot, None, 16,
        ))
        .unwrap();
        let query = decode_theorem_graph_query(&envelope.payload).unwrap();
        let mut payload = Vec::new();
        encode_hash_to(&mut payload, &query.env_fingerprint);
        encode_hash_to(&mut payload, &query.goal_fingerprint);
        encode_goal_to(&mut payload, &query.goal).unwrap();
        encode_hash_to(&mut payload, &query.snapshot.source_release_hash);
        payload.push(query.snapshot.extractor_version.tag());
        payload.push(0);
        encode_hash_to(&mut payload, &hash(99));
        encode_u64_to(
            &mut payload,
            u64::try_from(MAX_PHASE9_THEOREM_GRAPH_SNAPSHOT_BYTES).unwrap() + 1,
        );
        encode_theorem_graph_query_features_ref_to(&mut payload, &query.query_features);
        payload.push(query.ranking_profile.tag());
        encode_u64_to(&mut payload, u64::from(query.limit));
        envelope.payload = payload;
        let request = phase9_ai_candidate_envelope_canonical_bytes(&envelope).unwrap();

        assert_rejected(
            run_phase9_theorem_graph_query_request(
                &request,
                std::slice::from_ref(&import),
                &workspace_root(),
            ),
            Phase9AiValidationError::EnvelopeMalformed,
            Some(Phase9AiFeatureError::TheoremGraphQuery(
                Phase9TheoremGraphError::SnapshotMalformed,
            )),
        );
    }

    #[test]
    fn theorem_graph_inline_query_features_raw_cap_is_query_features_malformed() {
        let import = verified_theorem_graph_import();
        let snapshot =
            theorem_graph_snapshot(hash(41), vec![theorem_graph_node(&import, "GraphLib.P")]);
        let mut envelope = decode_candidate_envelope(&theorem_graph_inline_query_request(
            &import, None, None, snapshot, None, 16,
        ))
        .unwrap();
        let query = decode_theorem_graph_query(&envelope.payload).unwrap();
        let mut payload = Vec::new();
        encode_hash_to(&mut payload, &query.env_fingerprint);
        encode_hash_to(&mut payload, &query.goal_fingerprint);
        encode_goal_to(&mut payload, &query.goal).unwrap();
        encode_theorem_graph_snapshot_ref_to(&mut payload, &query.snapshot).unwrap();
        payload.push(0);
        encode_hash_to(&mut payload, &hash(98));
        encode_u64_to(
            &mut payload,
            u64::try_from(MAX_PHASE9_THEOREM_GRAPH_QUERY_FEATURES_BYTES).unwrap() + 1,
        );
        payload.push(query.ranking_profile.tag());
        encode_u64_to(&mut payload, u64::from(query.limit));
        envelope.payload = payload;
        let request = phase9_ai_candidate_envelope_canonical_bytes(&envelope).unwrap();

        assert_rejected(
            run_phase9_theorem_graph_query_request(
                &request,
                std::slice::from_ref(&import),
                &workspace_root(),
            ),
            Phase9AiValidationError::EnvelopeMalformed,
            Some(Phase9AiFeatureError::TheoremGraphQuery(
                Phase9TheoremGraphError::QueryFeaturesMalformed,
            )),
        );
    }

    #[test]
    fn theorem_graph_snapshot_hash_mismatch_precedes_full_decode_failure() {
        let import = verified_theorem_graph_import();
        let snapshot =
            theorem_graph_snapshot(hash(41), vec![theorem_graph_node(&import, "GraphLib.P")]);
        let mut envelope = decode_candidate_envelope(&theorem_graph_inline_query_request(
            &import, None, None, snapshot, None, 16,
        ))
        .unwrap();
        let mut query = decode_theorem_graph_query(&envelope.payload).unwrap();
        query.snapshot.source = Phase9MachineTheoremGraphSnapshotSource::Inline {
            graph_snapshot_hash: hash(99),
            canonical_bytes: theorem_graph_snapshot_bytes_with_noncanonical_node_name(hash(41)),
        };
        envelope.payload = phase9_theorem_graph_query_canonical_bytes(&query).unwrap();
        let request = phase9_ai_candidate_envelope_canonical_bytes(&envelope).unwrap();

        assert_rejected(
            run_phase9_theorem_graph_query_request(
                &request,
                std::slice::from_ref(&import),
                &workspace_root(),
            ),
            Phase9AiValidationError::PayloadHashMismatch,
            None,
        );
    }

    #[test]
    fn theorem_graph_snapshot_artifact_file_hash_mismatch_is_payload_hash_mismatch() {
        let import = verified_theorem_graph_import();
        let root = std::env::temp_dir().join(format!("npa-phase9-m4-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let snapshot =
            theorem_graph_snapshot(hash(41), vec![theorem_graph_node(&import, "GraphLib.P")]);
        let snapshot_bytes = phase9_theorem_graph_snapshot_canonical_bytes(&snapshot).unwrap();
        fs::write(root.join("snapshot.bin"), &snapshot_bytes).unwrap();
        let query_features_env = {
            let options_bytes = empty_options_bytes();
            let options_hash = phase9_ai_options_hash(&options_bytes);
            let imports = vec![Phase9ImportIdentity::from_verified_import(&import)];
            phase9_ai_env_fingerprint(
                Phase9AiProfileVersion::MvpV1,
                Phase9AiTaskKind::TheoremGraphQuery,
                &imports,
                options_hash,
            )
            .unwrap()
        };
        let goal = theorem_graph_goal();
        let goal_fingerprint = phase9_ai_goal_fingerprint(query_features_env, &goal);
        let features = theorem_graph_features(query_features_env, goal_fingerprint);
        let feature_bytes = phase9_theorem_graph_query_features_canonical_bytes(&features).unwrap();
        let query = Phase9MachineTheoremGraphQuery {
            env_fingerprint: query_features_env,
            goal_fingerprint,
            goal,
            snapshot: Phase9MachineTheoremGraphSnapshotRef {
                source_release_hash: snapshot.source_release_hash,
                extractor_version: snapshot.extractor_version,
                source: Phase9MachineTheoremGraphSnapshotSource::Artifact {
                    path: "snapshot.bin".to_owned(),
                    file_hash: hash(1),
                    graph_snapshot_hash: phase9_theorem_graph_snapshot_hash(&snapshot_bytes)
                        .unwrap(),
                    size_bytes: snapshot_bytes.len() as u64,
                },
            },
            query_features: Phase9MachineTheoremGraphQueryFeaturesRef::Inline {
                query_features_hash: phase9_theorem_graph_query_features_hash(&feature_bytes)
                    .unwrap(),
                canonical_bytes: feature_bytes,
            },
            ranking_profile: Phase9TheoremGraphRankingProfile::MvpTupleOrder,
            limit: 16,
        };
        let options_bytes = empty_options_bytes();
        let options_hash = phase9_ai_options_hash(&options_bytes);
        let envelope = Phase9AiCandidateEnvelope {
            profile_version: Phase9AiProfileVersion::MvpV1,
            task_kind: Phase9AiTaskKind::TheoremGraphQuery,
            target: Phase9AiTarget {
                env_fingerprint: query_features_env,
                target_decl_hash: None,
                goal_fingerprint: Some(goal_fingerprint),
            },
            imports: vec![Phase9ImportIdentity::from_verified_import(&import)],
            options: Phase9AiOptionsRef::Inline {
                options_hash,
                canonical_bytes: options_bytes,
            },
            payload: phase9_theorem_graph_query_canonical_bytes(&query).unwrap(),
        };
        let request = phase9_ai_candidate_envelope_canonical_bytes(&envelope).unwrap();

        assert_rejected(
            run_phase9_theorem_graph_query_request(&request, std::slice::from_ref(&import), &root),
            Phase9AiValidationError::PayloadHashMismatch,
            None,
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn theorem_graph_query_features_artifact_file_hash_mismatch_is_payload_hash_mismatch() {
        let import = verified_theorem_graph_import();
        let root =
            std::env::temp_dir().join(format!("npa-phase9-m4-features-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let snapshot =
            theorem_graph_snapshot(hash(41), vec![theorem_graph_node(&import, "GraphLib.P")]);
        let mut envelope = decode_candidate_envelope(&theorem_graph_inline_query_request(
            &import, None, None, snapshot, None, 16,
        ))
        .unwrap();
        let mut query = decode_theorem_graph_query(&envelope.payload).unwrap();
        let (query_features_hash, feature_bytes) = match &query.query_features {
            Phase9MachineTheoremGraphQueryFeaturesRef::Inline {
                query_features_hash,
                canonical_bytes,
            } => (*query_features_hash, canonical_bytes.clone()),
            Phase9MachineTheoremGraphQueryFeaturesRef::Artifact { .. } => unreachable!(),
        };
        fs::write(root.join("features.bin"), &feature_bytes).unwrap();
        query.query_features = Phase9MachineTheoremGraphQueryFeaturesRef::Artifact {
            path: "features.bin".to_owned(),
            file_hash: hash(2),
            query_features_hash,
            size_bytes: feature_bytes.len() as u64,
        };
        envelope.payload = phase9_theorem_graph_query_canonical_bytes(&query).unwrap();
        let request = phase9_ai_candidate_envelope_canonical_bytes(&envelope).unwrap();

        assert_rejected(
            run_phase9_theorem_graph_query_request(&request, std::slice::from_ref(&import), &root),
            Phase9AiValidationError::PayloadHashMismatch,
            None,
        );
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn universe_repair_valid_patch_returns_repaired_expr_and_constraint_hash() {
        let import = verified_universe_import();
        let request = valid_universe_request(&import);
        let expected_candidate_hash = phase9_ai_candidate_hash(&request);

        let response = run_phase9_universe_repair_check_request(
            &request,
            std::slice::from_ref(&import),
            &workspace_root(),
        );

        let Phase9AiEndpointResponse::Success {
            candidate_hash,
            validation_result_hash,
            payload,
        } = response
        else {
            panic!("expected success response");
        };
        assert_eq!(candidate_hash, expected_candidate_hash);
        let expected_payload = Phase9AiSuccessPayload::UniverseRepair {
            repaired_expr: Expr::konst("Lib.T", vec![Level::succ(Level::param("u"))]),
            constraint_set_hash: phase9_universe_constraint_set_hash(&[]),
        };
        assert_eq!(*payload, expected_payload);
        assert_eq!(
            validation_result_hash,
            phase9_ai_validation_result_hash_for_success(candidate_hash, &expected_payload)
        );
    }

    #[test]
    fn universe_repair_target_decl_hash_mode_is_unsupported() {
        let import = verified_universe_import();
        let request = universe_request_with_target(
            &import,
            valid_universe_candidate(&import),
            Some(hash(88)),
            None,
        );

        assert_rejected(
            run_phase9_universe_repair_check_request(
                &request,
                std::slice::from_ref(&import),
                &workspace_root(),
            ),
            Phase9AiValidationError::UnsupportedFeature,
            None,
        );
    }

    #[test]
    fn universe_repair_invalid_path_is_feature_rejection() {
        let import = verified_universe_import();
        let mut candidate = valid_universe_candidate(&import);
        candidate.instantiations[0].occurrence.path = vec![Phase9MachineExprPathStep::AppFun];
        let request = universe_request_with_target(&import, candidate, None, None);

        assert_rejected(
            run_phase9_universe_repair_check_request(
                &request,
                std::slice::from_ref(&import),
                &workspace_root(),
            ),
            Phase9AiValidationError::FeatureRejected,
            Some(Phase9AiFeatureError::UniverseRepair(
                Phase9UniverseRepairError::InvalidOccurrencePath,
            )),
        );
    }

    #[test]
    fn universe_repair_unknown_universe_param_is_feature_rejection() {
        let import = verified_universe_import();
        let mut candidate = valid_universe_candidate(&import);
        candidate.instantiations[0].explicit_level_args = vec![Level::param("v")];
        let request = universe_request_with_target(&import, candidate, None, None);

        assert_rejected(
            run_phase9_universe_repair_check_request(
                &request,
                std::slice::from_ref(&import),
                &workspace_root(),
            ),
            Phase9AiValidationError::FeatureRejected,
            Some(Phase9AiFeatureError::UniverseRepair(
                Phase9UniverseRepairError::UnknownUniverseParam,
            )),
        );
    }

    #[test]
    fn universe_repair_arity_mismatch_is_ill_formed_level_expr() {
        let import = verified_universe_import();
        let mut candidate = valid_universe_candidate(&import);
        candidate.instantiations[0].explicit_level_args = Vec::new();
        let request = universe_request_with_target(&import, candidate, None, None);

        assert_rejected(
            run_phase9_universe_repair_check_request(
                &request,
                std::slice::from_ref(&import),
                &workspace_root(),
            ),
            Phase9AiValidationError::FeatureRejected,
            Some(Phase9AiFeatureError::UniverseRepair(
                Phase9UniverseRepairError::IllFormedLevelExpr,
            )),
        );
    }

    #[test]
    fn universe_repair_unsatisfied_constraint_is_no_solution() {
        let import = verified_universe_import();
        let target = Expr::app(
            Expr::konst("Lib.F", vec![Level::succ(Level::param("u"))]),
            universe_target_expr(),
        );
        let candidate = Phase9UniverseRepairCandidate {
            goal: Some(universe_goal(target.clone())),
            target_expr: target,
            instantiations: vec![Phase9UniverseInstantiationPatch {
                occurrence: Phase9MachineExprOccurrence {
                    path: vec![Phase9MachineExprPathStep::AppFun],
                    expected_ref: universe_global_ref_for(&import, "Lib.F"),
                },
                explicit_level_args: vec![Level::param("u")],
            }],
            constraint_hints: Vec::new(),
            minimization_hint: None,
        };
        let request = universe_request_with_target(&import, candidate, None, None);

        assert_rejected(
            run_phase9_universe_repair_check_request(
                &request,
                std::slice::from_ref(&import),
                &workspace_root(),
            ),
            Phase9AiValidationError::NoSolution,
            Some(Phase9AiFeatureError::UniverseRepair(
                Phase9UniverseRepairError::UnsatisfiedConstraint,
            )),
        );
    }

    #[test]
    fn universe_repair_constraint_hint_cannot_add_solver_input() {
        let import = verified_universe_import();
        let mut candidate = valid_universe_candidate(&import);
        candidate.constraint_hints = vec![Phase9UniverseConstraintHint {
            constraint: Phase9UniverseConstraint {
                lhs: Level::param("u"),
                relation: Phase9UniverseConstraintRelation::Le,
                rhs: Level::param("u"),
            },
            reason: Phase9UniverseConstraintHintReason::RepairCandidate,
        }];
        let request = universe_request_with_target(&import, candidate, None, None);

        assert_rejected(
            run_phase9_universe_repair_check_request(
                &request,
                std::slice::from_ref(&import),
                &workspace_root(),
            ),
            Phase9AiValidationError::FeatureRejected,
            Some(Phase9AiFeatureError::UniverseRepair(
                Phase9UniverseRepairError::ConstraintHintMismatch,
            )),
        );
    }

    #[test]
    fn universe_repair_minimization_hint_does_not_change_result_payload() {
        let import = verified_universe_import();
        let mut first_candidate = valid_universe_candidate(&import);
        first_candidate.minimization_hint = Some(Phase9UniverseMinimizationHint::KernelDefault);
        let mut second_candidate = valid_universe_candidate(&import);
        second_candidate.minimization_hint =
            Some(Phase9UniverseMinimizationHint::PreferLowerLevels);
        let first = run_phase9_universe_repair_check_request(
            &universe_request_with_target(&import, first_candidate, None, None),
            std::slice::from_ref(&import),
            &workspace_root(),
        );
        let second = run_phase9_universe_repair_check_request(
            &universe_request_with_target(&import, second_candidate, None, None),
            std::slice::from_ref(&import),
            &workspace_root(),
        );

        let Phase9AiEndpointResponse::Success { payload: first, .. } = first else {
            panic!("expected first success");
        };
        let Phase9AiEndpointResponse::Success {
            payload: second, ..
        } = second
        else {
            panic!("expected second success");
        };
        assert_eq!(first, second);
    }

    #[test]
    fn approved_nested_type_constructor_is_common_unsupported_feature() {
        let mut options = Phase9AiOptions::default();
        options
            .advanced_inductive
            .approved_nested_type_constructors
            .push(Phase9AiGlobalRef {
                module: Name::from_dotted("Std.List"),
                export_hash: hash(1),
                certificate_hash: hash(2),
                name: Name::from_dotted("List"),
                decl_interface_hash: hash(3),
            });
        let options_bytes = phase9_ai_options_canonical_bytes(&options).unwrap();
        let request = inline_request(
            Phase9AiTaskKind::AdvancedInductive,
            options_bytes,
            Vec::new(),
            None,
        );

        assert_rejected(
            run_phase9_inductive_check_request(&request, &[], &workspace_root()),
            Phase9AiValidationError::UnsupportedFeature,
            Some(Phase9AiFeatureError::AdvancedInductive(
                Phase9AdvancedInductiveError::PositivityProfileUnsupported,
            )),
        );
    }

    #[test]
    fn route_skeletons_bind_each_endpoint_to_its_task_kind() {
        type Phase9Route = fn(&[u8], &[VerifiedImportRef], &Path) -> Phase9AiEndpointResponse;

        let routes: [(&str, Phase9Route); 7] = [
            (
                PHASE9_INDUCTIVE_CHECK_ENDPOINT,
                run_phase9_inductive_check_request,
            ),
            (
                PHASE9_UNIVERSE_REPAIR_CHECK_ENDPOINT,
                run_phase9_universe_repair_check_request,
            ),
            (
                PHASE9_TYPECLASS_RESOLVE_ENDPOINT,
                run_phase9_typeclass_resolve_request,
            ),
            (
                PHASE9_QUOTIENT_CHECK_ENDPOINT,
                run_phase9_quotient_check_request,
            ),
            (
                PHASE9_SMT_RECONSTRUCT_ENDPOINT,
                run_phase9_smt_reconstruct_request,
            ),
            (
                PHASE9_THEOREM_GRAPH_QUERY_ENDPOINT,
                run_phase9_theorem_graph_query_request,
            ),
            (
                PHASE9_FORMALIZE_CHECK_ENDPOINT,
                run_phase9_formalize_check_request,
            ),
        ];
        assert_eq!(routes.len(), 7);

        let import = verified_universe_import();
        let universe = valid_universe_request(&import);
        assert_rejected(
            run_phase9_inductive_check_request(&universe, &[], &workspace_root()),
            Phase9AiValidationError::EnvelopeMalformed,
            None,
        );
        assert!(matches!(
            run_phase9_universe_repair_check_request(
                &universe,
                std::slice::from_ref(&import),
                &workspace_root()
            ),
            Phase9AiEndpointResponse::Success { .. }
        ));
    }

    #[test]
    fn common_validation_success_is_deterministic_for_same_replay_input() {
        let request = inline_request(
            Phase9AiTaskKind::AdvancedInductive,
            empty_options_bytes(),
            Vec::new(),
            None,
        );

        let first = run_phase9_inductive_check_request(&request, &[], &workspace_root());
        let second = run_phase9_inductive_check_request(&request, &[], &workspace_root());

        assert_eq!(first, second);
        assert_rejected(first, Phase9AiValidationError::EnvelopeMalformed, None);
    }
}
