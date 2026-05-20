use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use npa_cert::{Hash, Name};
use npa_frontend::{
    lex_machine_surface_tokens, parse_machine_term, FileId, MachineSurfaceTokenKind, MachineTerm,
};
use npa_kernel::Level;
use npa_tactic::{
    goal_id_canonical_bytes, tactic_budget_hash, CandidateApplyArg, CandidateRewriteRuleRef,
    GoalId, MachineTacticBatchPolicy, MachineTacticCandidate, RawMachineTerm, RewriteDirection,
    RewriteSite, SimpRuleRef, TacticBudget, TacticHead,
};
use sha2::{Digest, Sha256};

use crate::current::MachineAxiomRefWire;
use crate::json::{JsonMember, JsonValue, JsonValueKind};
use crate::prompt::FailedCandidateErrorKind;
use crate::renderer::MachineGlobalRefView;
use crate::snapshot::{MachineSnapshotGetError, MachineSnapshotGetOk};
use crate::tactic::parse_deterministic_budget_with_error_kind;
use crate::types::{
    format_goal_id_wire, format_hash_string, is_machine_local_name, MachineApiCompactErrorWire,
    MachineApiErrorWire, MachineApiOkResponse, MachineApiResponseEnvelope,
    MachineApiResponseStatus, MachineApiVersion, MachineGoalView, MachineLocalView,
    MachineProofSession, MachineProofSnapshot, SessionId, SnapshotId,
};
use crate::validation::{
    parse_request_body, parse_strict_u64_token, validate_json_object, FieldSpec, JsonFieldType,
    JsonPath, MachineApiErrorKind, MachineApiRequestError, MachineApiRequestErrorReason,
    ObjectSchema, StrictUnsignedIntegerError, ValidatedObject,
};
use crate::{
    get_machine_snapshot, parse_machine_replay_request, parse_machine_tactic_batch_request,
    parse_machine_theorem_search_request, parse_machine_verify_request, run_machine_replay_request,
    run_machine_tactic_batch_request, run_machine_verify_request, search_machine_theorems_for_goal,
    MachineApiTacticKind, MachineBatchSchedulerLimits, MachineReplayError, MachineReplayOkFields,
    MachineReplayResponse, MachineTacticBatchError, MachineTacticBatchItemResponse,
    MachineTacticBatchOkFields, MachineTacticBatchResponse, MachineTacticBatchSchedulerFields,
    MachineTheoremMode, MachineTheoremSearchError, MachineTheoremSearchOkFields,
    MachineTheoremSearchResponse, MachineTheoremSearchResult, MachineVerifyError,
    MachineVerifyOkFields, MachineVerifyResponse,
};

const PHASE7_MVP_MAX_TACTICS_PER_NODE: u32 = 16;
const PHASE7_MVP_PREMISE_QUERY_LIMIT: u32 = 32;
const PHASE7_CANDIDATE_PAYLOAD_HASH_TAG: &str = "npa.phase7.candidate-payload.v1";

const PHASE7_CONFIG_FIELDS: &[FieldSpec] = &[
    FieldSpec::required("search_budget", JsonFieldType::Object),
    FieldSpec::required("per_tactic_deterministic_budget", JsonFieldType::Object),
    FieldSpec::required("batch_policy", JsonFieldType::Object),
];

const PHASE7_SEARCH_BUDGET_FIELDS: &[FieldSpec] = &[
    FieldSpec::required(
        "wall_clock_ms",
        JsonFieldType::UnsignedInteger { max: u64::MAX },
    ),
    FieldSpec::required(
        "max_nodes",
        JsonFieldType::UnsignedInteger { max: u64::MAX },
    ),
    FieldSpec::required(
        "max_tactics_per_node",
        JsonFieldType::UnsignedInteger { max: u64::MAX },
    ),
    FieldSpec::required(
        "max_depth",
        JsonFieldType::UnsignedInteger { max: u64::MAX },
    ),
];

const PHASE7_BATCH_POLICY_FIELDS: &[FieldSpec] = &[
    FieldSpec::required(
        "max_evaluated_candidates",
        JsonFieldType::UnsignedInteger { max: u64::MAX },
    ),
    FieldSpec::required(
        "stop_after_successes",
        JsonFieldType::UnsignedInteger { max: u64::MAX },
    ),
    FieldSpec::required(
        "stop_after_failures",
        JsonFieldType::UnsignedInteger { max: u64::MAX },
    ),
];

const PHASE7_BATCH_SCHEDULER_FIELDS: &[FieldSpec] = &[
    FieldSpec::optional(
        "per_candidate_timeout_ms",
        JsonFieldType::UnsignedInteger { max: u64::MAX },
    ),
    FieldSpec::optional(
        "batch_timeout_ms",
        JsonFieldType::UnsignedInteger { max: u64::MAX },
    ),
    FieldSpec::optional(
        "max_memory_mb",
        JsonFieldType::UnsignedInteger { max: u64::MAX },
    ),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Phase7SearchBudget {
    pub wall_clock_ms: u64,
    pub max_nodes: u64,
    pub max_tactics_per_node: u32,
    pub max_depth: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Phase7MvpControllerConfig {
    pub search_budget: Phase7SearchBudget,
    pub per_tactic_deterministic_budget: TacticBudget,
    pub scheduler_limits: Option<MachineBatchSchedulerLimits>,
    pub batch_policy: MachineTacticBatchPolicy,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7SnapshotGetRequest {
    pub session_id: SessionId,
    pub snapshot_id: SnapshotId,
    pub state_fingerprint: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7InitialSnapshot {
    pub snapshot: MachineProofSnapshot,
    pub goals: Vec<Phase7GoalSummary>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7GoalSummary {
    pub goal_id: GoalId,
    pub open_goal_index: u32,
    pub goal_fingerprint: Hash,
    pub target_hash: Hash,
    pub target_head: Option<MachineGlobalRefView>,
    pub target_free_local_count: u32,
    pub context_size: u32,
    pub expr_size: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7PremiseQueryRequest {
    pub session_id: SessionId,
    pub snapshot_id: SnapshotId,
    pub state_fingerprint: Hash,
    pub goal_id: GoalId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7RetrievalCacheKey {
    pub session_root_hash: Hash,
    pub query_fingerprint: Hash,
    pub theorem_index_fingerprint: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7PremiseRef {
    pub module: Name,
    pub name: Name,
    pub export_hash: Hash,
    pub decl_interface_hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7PremiseUsage {
    pub premise_ref: Phase7PremiseRef,
    pub universe_params: Vec<String>,
    pub statement_core_hash: Hash,
    pub axioms_used: Vec<MachineAxiomRefWire>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7PremiseCacheEntry {
    pub premise_ref: Phase7PremiseRef,
    pub universe_params: Vec<String>,
    pub statement_core_hash: Hash,
    pub statement_head: Option<MachineGlobalRefView>,
    pub axioms_used: Vec<MachineAxiomRefWire>,
    pub modes: Vec<MachineTheoremMode>,
    pub response_index: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7PremiseRetrieval {
    pub cache_key: Phase7RetrievalCacheKey,
    pub cache_entries: Vec<Phase7PremiseCacheEntry>,
    pub results: Vec<MachineTheoremSearchResult>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7CandidateEnvelope {
    pub candidate: MachineTacticCandidate,
    pub phase7_candidate_payload_hash: Hash,
    pub candidate_hash: Option<Hash>,
    pub metadata: Phase7CandidateMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7CandidateMetadata {
    pub source: Phase7CandidateSource,
    pub rank: Phase7CandidateRankMetadata,
    pub score: Phase7Score,
    pub display_text: Option<String>,
    pub premises_used: Vec<Phase7PremiseUsage>,
    pub expected_effect: Phase7ExpectedEffect,
    pub cost_estimate: Phase7CandidateCostEstimate,
    pub trust_flags: Phase7TrustFlags,
    pub repair: Option<Phase7CandidateRepairMetadata>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase7CandidateSource {
    Phase5Suggested,
    Builtin,
    Model,
    Exploration,
    Repair,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Phase7CandidateRankMetadata {
    pub source_rank: u8,
    pub source_index: u32,
    pub builtin_kind_rank: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase7BuiltinKind {
    Intro,
    LocalExact,
    InductionNat,
    SimpLiteEmpty,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase7ExpectedEffect {
    IntroBinder,
    CloseGoal,
    Rewrite,
    Simplify,
    InductionSplit,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Phase7CandidateCostEstimate {
    pub estimated_timeout_ms: u32,
    pub risk: Phase7CandidateCostRisk,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase7CandidateCostRisk {
    Low,
    Medium,
    High,
}

pub type Phase7Score = i64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7TrustFlags {
    pub uses_axioms: Vec<MachineAxiomRefWire>,
    pub contains_forbidden_tokens: bool,
    pub forbidden_token_class: Option<Phase7ForbiddenTokenClass>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7CandidateRepairMetadata {
    pub parent_candidate_hash: Hash,
    pub error_kind: FailedCandidateErrorKind,
    pub repair_depth: u32,
    pub chain_tried_payload_hashes: Vec<Hash>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7ForbiddenToken {
    pub class: Phase7ForbiddenTokenClass,
    pub spelling: String,
    pub raw_term_index: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase7ForbiddenTokenClass {
    Sorry,
    Admit,
    Axiom,
    Unsafe,
    Import,
    SetOptionUnsafe,
    Declare,
    Eval,
    Shell,
    ExternalCommand,
    DisallowedTacticKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase7CandidateFilterError {
    RawMachineTermLex {
        raw_term_index: u32,
        source: String,
        message: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7CandidateFilterResult {
    pub accepted: Vec<Phase7CandidateEnvelope>,
    pub rejected: Vec<Phase7RejectedCandidateEnvelope>,
    pub errors: Vec<Phase7CandidateFilterError>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7RejectedCandidateEnvelope {
    pub envelope: Phase7CandidateEnvelope,
    pub forbidden_token: Phase7ForbiddenToken,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7AssignedCandidate {
    pub candidate_id: String,
    pub rank_index: u32,
    pub envelope: Phase7CandidateEnvelope,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7TacticBatchRequest {
    pub session_id: SessionId,
    pub snapshot_id: SnapshotId,
    pub state_fingerprint: Hash,
    pub goal_id: GoalId,
    pub candidates: Vec<Phase7AssignedCandidate>,
    pub deterministic_budget: TacticBudget,
    pub batch_policy: MachineTacticBatchPolicy,
    pub scheduler_limits: Option<MachineBatchSchedulerLimits>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7ReplayStep {
    pub previous_state_fingerprint: Hash,
    pub goal_id: GoalId,
    pub candidate: MachineTacticCandidate,
    pub deterministic_budget: TacticBudget,
    pub candidate_hash: Hash,
    pub deterministic_budget_hash: Hash,
    pub proof_delta_hash: Hash,
    pub next_state_fingerprint: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7ReplayPlan {
    pub protocol_version: MachineApiVersion,
    pub session_root_hash: Hash,
    pub initial_state_fingerprint: Hash,
    pub steps: Vec<Phase7ReplayStep>,
    pub final_state_fingerprint: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7AcceptedCandidateFailure {
    pub error_kind: FailedCandidateErrorKind,
    pub phase: crate::MachineApiDiagnosticPhase,
    pub goal_id: Option<GoalId>,
    pub tactic_kind: Option<MachineApiTacticKind>,
    pub candidate_hash: Hash,
    pub deterministic_budget_hash: Hash,
    pub diagnostic_hash: Hash,
    pub retryable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7NonAcceptedCandidateError {
    pub candidate_id: String,
    pub phase7_candidate_payload_hash: Hash,
    pub error_kind: MachineApiErrorKind,
    pub phase: crate::MachineApiDiagnosticPhase,
    pub has_candidate_hash: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7DeferredCandidate {
    pub candidate_id: String,
    pub envelope: Phase7CandidateEnvelope,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Phase7SchedulerStop {
    pub status: MachineApiResponseStatus,
    pub completed_prefix_len: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7BatchEvaluation {
    pub successful_transitions: Vec<Phase7SuccessfulCandidateTransition>,
    pub accepted_failure_records: Vec<Phase7AcceptedCandidateFailureRecord>,
    pub replay_steps: Vec<Phase7ReplayStep>,
    pub accepted_failures: Vec<Phase7AcceptedCandidateFailure>,
    pub non_accepted_errors: Vec<Phase7NonAcceptedCandidateError>,
    pub evaluated_count: u32,
    pub deferred_candidates: Vec<Phase7DeferredCandidate>,
    pub scheduler_stop: Option<Phase7SchedulerStop>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7SuccessfulCandidateTransition {
    pub candidate_id: String,
    pub envelope: Phase7CandidateEnvelope,
    pub next_snapshot_id: SnapshotId,
    pub replay_step: Phase7ReplayStep,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7AcceptedCandidateFailureRecord {
    pub candidate_id: String,
    pub envelope: Phase7CandidateEnvelope,
    pub failure: Phase7AcceptedCandidateFailure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7PendingCandidate {
    pub goal_id: GoalId,
    pub candidate: Phase7CandidateEnvelope,
    pub repair_depth: u32,
    pub parent_candidate_hash: Hash,
    pub error_kind: FailedCandidateErrorKind,
    pub chain_tried_payload_hashes: Vec<Hash>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Phase7RepairCandidateOutput {
    pub pending: Vec<Phase7PendingCandidate>,
    pub repeated_candidate_payload_hashes: Vec<Hash>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Phase7RuleBasedRepair;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase7RuleBasedRepairAction {
    Noop,
    TrySimpLite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase7RepairChainStopReason {
    RepeatedError,
    RepeatedCandidate,
    MaxRepairDepth,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase7MachineControllerErrorKind {
    TopLevelBatchError,
    BatchResponseContractViolation,
    SuggestedCandidateHashMismatch,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7MachineControllerError {
    pub kind: Phase7MachineControllerErrorKind,
    pub endpoint: Phase7MachineApiEndpointKind,
    pub message: String,
    pub candidate_id: Option<String>,
    pub expected_hash: Option<Hash>,
    pub actual_hash: Option<Hash>,
    pub diagnostic_hash: Option<Hash>,
    pub phase: Option<crate::MachineApiDiagnosticPhase>,
    pub status: Option<MachineApiResponseStatus>,
}

pub type Phase7MachineControllerResult<T> = Result<T, Box<Phase7MachineControllerError>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase7TacticBatchRunError {
    MachineApi(Phase7MachineApiError),
    Controller(Box<Phase7MachineControllerError>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Phase7NodeId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase7NodeStatus {
    Queued,
    Expanded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7SearchNode {
    pub node_id: Phase7NodeId,
    pub session_id: SessionId,
    pub session_root_hash: Hash,
    pub initial_state_fingerprint: Hash,
    pub snapshot_id: SnapshotId,
    pub state_fingerprint: Hash,
    pub goals: Vec<Phase7GoalSummary>,
    pub replay_steps: Vec<Phase7ReplayStep>,
    pub depth: u32,
    pub cumulative_score: Phase7Score,
    pub last_candidate: Option<MachineTacticCandidate>,
    pub last_candidate_hash: Option<Hash>,
    pub used_premises: Vec<Phase7PremiseUsage>,
    pub parent: Option<Phase7NodeId>,
    pub status: Phase7NodeStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7SearchInput {
    pub session_id: SessionId,
    pub session_root_hash: Hash,
    pub initial_snapshot: MachineProofSnapshot,
    pub search_budget: Phase7SearchBudget,
    pub per_tactic_deterministic_budget: TacticBudget,
    pub scheduler_limits: Option<MachineBatchSchedulerLimits>,
    pub batch_policy: MachineTacticBatchPolicy,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Phase7SearchStats {
    pub nodes_expanded: u64,
    pub candidates_evaluated: u64,
    pub scheduler_stops: u64,
    pub zero_progress_scheduler_stops: u64,
    pub closed_node_replay_rejections: u64,
    pub closed_node_verify_rejections: u64,
    pub controller_errors: u64,
    pub no_candidate_stops: u64,
    pub max_depth_stops: u64,
    pub best_partial_updates: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase7SearchBudgetLimit {
    WallClock,
    MaxNodes,
    MaxDepth,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase7SearchFailureReason {
    QueueExhausted,
    SearchBudgetExceeded {
        limit: Phase7SearchBudgetLimit,
    },
    MachineControllerError {
        endpoint: String,
        error_kind: String,
        error_phase: Option<String>,
        diagnostic_hash: Option<Hash>,
    },
    NoCandidateForSelectedGoal {
        goal_id: GoalId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7SearchFailure {
    pub reason: Phase7SearchFailureReason,
    pub best_partial_replay_prefix: Option<Vec<Phase7ReplayStep>>,
    pub best_snapshot_id: Option<SnapshotId>,
    pub best_state_fingerprint: Option<Hash>,
    pub remaining_goals: Option<Vec<Phase7GoalSummary>>,
    pub search_stats: Phase7SearchStats,
    pub trace_events: Vec<Phase7SearchTraceEvent>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Phase7MinimizationStats {
    pub pass_kinds_attempted: u64,
    pub rebuilt_plans: u64,
    pub replay_attempts: u64,
    pub verify_attempts: u64,
    pub accepted_proposals: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7MinimizationResult {
    pub replay_plan: Phase7ReplayPlan,
    pub replay_response: MachineReplayOkFields,
    pub verify_response: MachineApiOkResponse<MachineVerifyOkFields>,
    pub minimization_stats: Phase7MinimizationStats,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7ReplayStepEdit {
    pub original_goal_id: GoalId,
    pub original_open_goal_index: u32,
    pub candidate: MachineTacticCandidate,
    pub deterministic_budget: TacticBudget,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7VerifiedProof {
    pub replay_plan: Phase7ReplayPlan,
    pub final_snapshot_id: SnapshotId,
    pub final_state_fingerprint: Hash,
    pub verify_response: MachineApiOkResponse<MachineVerifyOkFields>,
    pub search_stats: Phase7SearchStats,
    pub minimization_stats: Phase7MinimizationStats,
    pub trace_events: Vec<Phase7SearchTraceEvent>,
}

pub type Phase7SearchResult = Result<Phase7VerifiedProof, Box<Phase7SearchFailure>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase7SearchTraceEvent {
    pub event_index: u64,
    pub node_id: Phase7NodeId,
    pub kind: Phase7SearchTraceEventKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase7SearchTraceEventKind {
    NodeExpanded,
    DuplicateStateSkipped {
        duplicate_state_fingerprint: Hash,
    },
    ChildQueued {
        child_node_id: Phase7NodeId,
        state_fingerprint: Hash,
    },
    NoCandidateForSelectedGoal {
        goal_id: GoalId,
    },
    MaxDepthStopped {
        max_depth: u32,
    },
    SchedulerStopped {
        status: MachineApiResponseStatus,
        completed_prefix_len: u32,
    },
    ZeroProgressSchedulerStopped {
        status: MachineApiResponseStatus,
    },
    MaxTacticsPerNodeStopped {
        max_tactics_per_node: u32,
    },
    MachineControllerError {
        endpoint: String,
        error_kind: String,
    },
    RepairChainStopped {
        parent_candidate_hash: Hash,
        error_kind: FailedCandidateErrorKind,
        repair_depth: u32,
        reason: Phase7RepairChainStopReason,
        repeated_candidate_payload_hash: Option<Hash>,
    },
    ClosedNodeReplayRejected {
        endpoint: String,
        status: MachineApiResponseStatus,
    },
    ClosedNodeVerifyRejected {
        endpoint: String,
        status: MachineApiResponseStatus,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Phase7SearchPriorityKey {
    pub open_goal_count: u32,
    pub depth: u32,
    pub replay_step_count: u32,
    pub total_open_goal_target_size: u64,
    pub state_fingerprint: Hash,
    pub node_id: Phase7NodeId,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Phase7BestPartialKey {
    pub open_goal_count: u32,
    pub total_open_goal_target_size: u64,
    pub replay_step_count: u32,
    pub depth: u32,
    pub state_fingerprint: Hash,
    pub node_id: Phase7NodeId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase7MachineApiEndpointKind {
    SnapshotGet,
    SearchForGoal,
    TacticBatch,
    Replay,
    Verify,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase7MachineApiCall {
    SnapshotGet {
        session_id: SessionId,
        snapshot_id: SnapshotId,
        state_fingerprint: Hash,
        include_pretty: bool,
    },
    SearchForGoal {
        source: String,
    },
    TacticBatch {
        source: String,
    },
    Replay {
        source: String,
    },
    Verify {
        source: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase7MachineApiError {
    SnapshotGet(Box<MachineSnapshotGetError>),
    SearchForGoal(Box<MachineTheoremSearchError>),
    TacticBatch(Box<MachineTacticBatchError>),
    Replay(Box<MachineReplayError>),
    Verify(Box<MachineVerifyError>),
    SearchForGoalResponse(Box<MachineApiErrorWire>),
    UnexpectedSchedulerStop {
        endpoint: Phase7MachineApiEndpointKind,
    },
    FakeRequestValidation {
        endpoint: Phase7MachineApiEndpointKind,
        error: MachineApiRequestError,
    },
    FakeResponseExhausted {
        endpoint: Phase7MachineApiEndpointKind,
    },
}

pub type Phase7MachineApiResult<T> = Result<T, Phase7MachineApiError>;

pub trait Phase7MachineApiClient {
    fn get_snapshot(
        &mut self,
        request: Phase7SnapshotGetRequest,
    ) -> Phase7MachineApiResult<MachineSnapshotGetOk>;

    fn search_for_goal(
        &mut self,
        source: &str,
    ) -> Phase7MachineApiResult<MachineTheoremSearchResponse>;

    fn run_tactic_batch(
        &mut self,
        source: &str,
    ) -> Phase7MachineApiResult<MachineTacticBatchResponse>;

    fn replay(&mut self, source: &str) -> Phase7MachineApiResult<MachineReplayResponse>;

    fn verify(&mut self, source: &str) -> Phase7MachineApiResult<MachineVerifyResponse>;
}

pub struct Phase7LocalMachineApiClient<'session> {
    session: &'session mut MachineProofSession,
}

impl<'session> Phase7LocalMachineApiClient<'session> {
    pub fn new(session: &'session mut MachineProofSession) -> Self {
        Self { session }
    }
}

impl Phase7MachineApiClient for Phase7LocalMachineApiClient<'_> {
    fn get_snapshot(
        &mut self,
        request: Phase7SnapshotGetRequest,
    ) -> Phase7MachineApiResult<MachineSnapshotGetOk> {
        let source = phase7_snapshot_get_request_json(&request);
        get_machine_snapshot(&source, std::iter::once(&*self.session))
            .map_err(Phase7MachineApiError::SnapshotGet)
    }

    fn search_for_goal(
        &mut self,
        source: &str,
    ) -> Phase7MachineApiResult<MachineTheoremSearchResponse> {
        search_machine_theorems_for_goal(source, &*self.session)
            .map_err(Phase7MachineApiError::SearchForGoal)
    }

    fn run_tactic_batch(
        &mut self,
        source: &str,
    ) -> Phase7MachineApiResult<MachineTacticBatchResponse> {
        run_machine_tactic_batch_request(source, self.session)
            .map_err(Phase7MachineApiError::TacticBatch)
    }

    fn replay(&mut self, source: &str) -> Phase7MachineApiResult<MachineReplayResponse> {
        run_machine_replay_request(source, self.session).map_err(Phase7MachineApiError::Replay)
    }

    fn verify(&mut self, source: &str) -> Phase7MachineApiResult<MachineVerifyResponse> {
        run_machine_verify_request(source, &*self.session).map_err(Phase7MachineApiError::Verify)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Phase7FakeMachineApiClient {
    calls: Vec<Phase7MachineApiCall>,
    snapshot_get_responses: VecDeque<Phase7MachineApiResult<MachineSnapshotGetOk>>,
    search_for_goal_responses: VecDeque<Phase7MachineApiResult<MachineTheoremSearchResponse>>,
    tactic_batch_responses: VecDeque<Phase7MachineApiResult<MachineTacticBatchResponse>>,
    replay_responses: VecDeque<Phase7MachineApiResult<MachineReplayResponse>>,
    verify_responses: VecDeque<Phase7MachineApiResult<MachineVerifyResponse>>,
}

impl Phase7FakeMachineApiClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn calls(&self) -> &[Phase7MachineApiCall] {
        &self.calls
    }

    pub fn push_snapshot_get_response(
        &mut self,
        response: Phase7MachineApiResult<MachineSnapshotGetOk>,
    ) {
        self.snapshot_get_responses.push_back(response);
    }

    pub fn push_search_for_goal_response(
        &mut self,
        response: Phase7MachineApiResult<MachineTheoremSearchResponse>,
    ) {
        self.search_for_goal_responses.push_back(response);
    }

    pub fn push_tactic_batch_response(
        &mut self,
        response: Phase7MachineApiResult<MachineTacticBatchResponse>,
    ) {
        self.tactic_batch_responses.push_back(response);
    }

    pub fn push_replay_response(
        &mut self,
        response: Phase7MachineApiResult<MachineReplayResponse>,
    ) {
        self.replay_responses.push_back(response);
    }

    pub fn push_verify_response(
        &mut self,
        response: Phase7MachineApiResult<MachineVerifyResponse>,
    ) {
        self.verify_responses.push_back(response);
    }
}

impl Phase7MachineApiClient for Phase7FakeMachineApiClient {
    fn get_snapshot(
        &mut self,
        request: Phase7SnapshotGetRequest,
    ) -> Phase7MachineApiResult<MachineSnapshotGetOk> {
        self.calls.push(Phase7MachineApiCall::SnapshotGet {
            session_id: request.session_id,
            snapshot_id: request.snapshot_id,
            state_fingerprint: request.state_fingerprint,
            include_pretty: false,
        });
        self.snapshot_get_responses.pop_front().unwrap_or(Err(
            Phase7MachineApiError::FakeResponseExhausted {
                endpoint: Phase7MachineApiEndpointKind::SnapshotGet,
            },
        ))
    }

    fn search_for_goal(
        &mut self,
        source: &str,
    ) -> Phase7MachineApiResult<MachineTheoremSearchResponse> {
        parse_machine_theorem_search_request(source).map_err(|error| {
            Phase7MachineApiError::FakeRequestValidation {
                endpoint: Phase7MachineApiEndpointKind::SearchForGoal,
                error,
            }
        })?;
        self.calls.push(Phase7MachineApiCall::SearchForGoal {
            source: source.to_owned(),
        });
        self.search_for_goal_responses.pop_front().unwrap_or(Err(
            Phase7MachineApiError::FakeResponseExhausted {
                endpoint: Phase7MachineApiEndpointKind::SearchForGoal,
            },
        ))
    }

    fn run_tactic_batch(
        &mut self,
        source: &str,
    ) -> Phase7MachineApiResult<MachineTacticBatchResponse> {
        parse_machine_tactic_batch_request(source).map_err(|error| {
            Phase7MachineApiError::FakeRequestValidation {
                endpoint: Phase7MachineApiEndpointKind::TacticBatch,
                error,
            }
        })?;
        self.calls.push(Phase7MachineApiCall::TacticBatch {
            source: source.to_owned(),
        });
        self.tactic_batch_responses.pop_front().unwrap_or(Err(
            Phase7MachineApiError::FakeResponseExhausted {
                endpoint: Phase7MachineApiEndpointKind::TacticBatch,
            },
        ))
    }

    fn replay(&mut self, source: &str) -> Phase7MachineApiResult<MachineReplayResponse> {
        parse_machine_replay_request(source).map_err(|error| {
            Phase7MachineApiError::FakeRequestValidation {
                endpoint: Phase7MachineApiEndpointKind::Replay,
                error,
            }
        })?;
        self.calls.push(Phase7MachineApiCall::Replay {
            source: source.to_owned(),
        });
        self.replay_responses.pop_front().unwrap_or(Err(
            Phase7MachineApiError::FakeResponseExhausted {
                endpoint: Phase7MachineApiEndpointKind::Replay,
            },
        ))
    }

    fn verify(&mut self, source: &str) -> Phase7MachineApiResult<MachineVerifyResponse> {
        parse_machine_verify_request(source).map_err(|error| {
            Phase7MachineApiError::FakeRequestValidation {
                endpoint: Phase7MachineApiEndpointKind::Verify,
                error,
            }
        })?;
        self.calls.push(Phase7MachineApiCall::Verify {
            source: source.to_owned(),
        });
        self.verify_responses.pop_front().unwrap_or(Err(
            Phase7MachineApiError::FakeResponseExhausted {
                endpoint: Phase7MachineApiEndpointKind::Verify,
            },
        ))
    }
}

pub fn phase7_assign_candidate_ids(
    candidates: Vec<Phase7CandidateEnvelope>,
) -> Vec<Phase7AssignedCandidate> {
    candidates
        .into_iter()
        .enumerate()
        .map(|(index, envelope)| Phase7AssignedCandidate {
            candidate_id: format!("c{index}"),
            rank_index: usize_to_u32(index),
            envelope,
        })
        .collect()
}

pub fn phase7_cap_batch_policy(
    policy: MachineTacticBatchPolicy,
    candidate_count: usize,
) -> MachineTacticBatchPolicy {
    let candidate_cap = usize_to_u32(candidate_count).max(1);
    MachineTacticBatchPolicy {
        max_evaluated_candidates: policy.max_evaluated_candidates.min(candidate_cap),
        stop_after_successes: policy.stop_after_successes.min(candidate_cap),
        stop_after_failures: policy.stop_after_failures.min(candidate_cap),
    }
}

pub fn phase7_tactic_batch_request_json(request: &Phase7TacticBatchRequest) -> String {
    let candidates = request
        .candidates
        .iter()
        .map(|candidate| {
            format!(
                r#"{{"candidate_id":{},"candidate":{}}}"#,
                json_string(&candidate.candidate_id),
                phase7_candidate_payload_json(&candidate.envelope.candidate)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let capped_policy = phase7_cap_batch_policy(request.batch_policy, request.candidates.len());
    let mut source = format!(
        r#"{{"session_id":"{}","snapshot_id":"{}","state_fingerprint":"{}","goal_id":"{}","candidates":[{}],"deterministic_budget":{},"batch_policy":{}"#,
        request.session_id.wire(),
        request.snapshot_id.wire(),
        format_hash_string(&request.state_fingerprint),
        format_goal_id_wire(request.goal_id),
        candidates,
        phase7_tactic_budget_json(request.deterministic_budget),
        phase7_batch_policy_json(capped_policy)
    );
    if let Some(scheduler_limits) = request.scheduler_limits {
        source.push_str(r#","scheduler_limits":"#);
        source.push_str(&phase7_scheduler_limits_json(scheduler_limits));
    }
    source.push('}');
    source
}

pub fn phase7_run_tactic_batch(
    client: &mut impl Phase7MachineApiClient,
    request: &Phase7TacticBatchRequest,
) -> Result<Phase7BatchEvaluation, Phase7TacticBatchRunError> {
    let source = phase7_tactic_batch_request_json(request);
    let response = client
        .run_tactic_batch(&source)
        .map_err(Phase7TacticBatchRunError::MachineApi)?;
    phase7_evaluate_tactic_batch_response(request, response)
        .map_err(Phase7TacticBatchRunError::Controller)
}

pub fn phase7_candidate_hash_mismatch(
    envelope: &Phase7CandidateEnvelope,
    response_candidate_hash: Option<Hash>,
) -> bool {
    envelope
        .candidate_hash
        .is_some_and(|expected| response_candidate_hash != Some(expected))
}

pub fn phase7_evaluate_tactic_batch_response(
    request: &Phase7TacticBatchRequest,
    response: MachineTacticBatchResponse,
) -> Phase7MachineControllerResult<Phase7BatchEvaluation> {
    match response {
        MachineApiResponseEnvelope::Ok(ok) => {
            if ok.status != MachineApiResponseStatus::Ok {
                return Err(phase7_batch_contract_violation(
                    format!("batch ok envelope used status {}", ok.status.as_str()),
                    Some(ok.status),
                ));
            }
            let fields = ok.endpoint_fields;
            phase7_validate_ok_batch_fields(request, &fields)?;
            Ok(phase7_build_batch_evaluation(
                request,
                &fields.results,
                fields.deterministic_budget_hash,
                None,
                fields.results.len(),
            ))
        }
        MachineApiResponseEnvelope::SchedulerStopped(stop) => {
            if !matches!(
                stop.status,
                MachineApiResponseStatus::PartialTimeout
                    | MachineApiResponseStatus::PartialResourceLimit
            ) {
                return Err(phase7_batch_contract_violation(
                    format!(
                        "batch scheduler envelope used status {}",
                        stop.status.as_str()
                    ),
                    Some(stop.status),
                ));
            }
            let fields = stop.endpoint_fields;
            phase7_validate_scheduler_batch_fields(request, &fields, stop.status)?;
            let completed_prefix_len = fields.completed_prefix_len;
            let deferred_start = if fields.results.is_empty() {
                request.candidates.len()
            } else {
                (fields.results.len() + 1).min(request.candidates.len())
            };
            Ok(phase7_build_batch_evaluation(
                request,
                &fields.results,
                fields.deterministic_budget_hash,
                Some(Phase7SchedulerStop {
                    status: stop.status,
                    completed_prefix_len,
                }),
                deferred_start,
            ))
        }
        MachineApiResponseEnvelope::Error(error) => Err(Box::new(Phase7MachineControllerError {
            kind: Phase7MachineControllerErrorKind::TopLevelBatchError,
            endpoint: Phase7MachineApiEndpointKind::TacticBatch,
            message: error.error.kind.as_str().to_owned(),
            candidate_id: None,
            expected_hash: None,
            actual_hash: None,
            diagnostic_hash: Some(error.error.diagnostic_hash),
            phase: Some(error.error.phase),
            status: Some(error.status),
        })),
    }
}

pub fn phase7_replay_step_json(step: &Phase7ReplayStep) -> String {
    format!(
        r#"{{"previous_state_fingerprint":{},"goal_id":{},"candidate":{},"deterministic_budget":{},"candidate_hash":{},"deterministic_budget_hash":{},"proof_delta_hash":{},"next_state_fingerprint":{}}}"#,
        json_string(&format_hash_string(&step.previous_state_fingerprint)),
        json_string(&format_goal_id_wire(step.goal_id)),
        phase7_candidate_payload_json(&step.candidate),
        phase7_tactic_budget_json(step.deterministic_budget),
        json_string(&format_hash_string(&step.candidate_hash)),
        json_string(&format_hash_string(&step.deterministic_budget_hash)),
        json_string(&format_hash_string(&step.proof_delta_hash)),
        json_string(&format_hash_string(&step.next_state_fingerprint)),
    )
}

pub fn phase7_build_replay_plan(node: &Phase7SearchNode) -> Phase7ReplayPlan {
    Phase7ReplayPlan {
        protocol_version: MachineApiVersion::V1,
        session_root_hash: node.session_root_hash,
        initial_state_fingerprint: node.initial_state_fingerprint,
        steps: node.replay_steps.clone(),
        final_state_fingerprint: node.state_fingerprint,
    }
}

pub fn phase7_replay_plan_json(plan: &Phase7ReplayPlan) -> String {
    let steps = plan
        .steps
        .iter()
        .map(phase7_replay_step_json)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"{{"protocol_version":{},"session_root_hash":{},"initial_state_fingerprint":{},"steps":[{}],"final_state_fingerprint":{}}}"#,
        json_string(plan.protocol_version.as_str()),
        json_string(&format_hash_string(&plan.session_root_hash)),
        json_string(&format_hash_string(&plan.initial_state_fingerprint)),
        steps,
        json_string(&format_hash_string(&plan.final_state_fingerprint)),
    )
}

pub fn phase7_replay_request_json(session_id: SessionId, plan: &Phase7ReplayPlan) -> String {
    format!(
        r#"{{"session_id":"{}","plan":{}}}"#,
        session_id.wire(),
        phase7_replay_plan_json(plan)
    )
}

pub fn phase7_verify_request_json(
    session_id: SessionId,
    snapshot_id: SnapshotId,
    state_fingerprint: Hash,
) -> String {
    format!(
        r#"{{"session_id":"{}","snapshot_id":"{}","state_fingerprint":"{}","mode":"certificate"}}"#,
        session_id.wire(),
        snapshot_id.wire(),
        format_hash_string(&state_fingerprint),
    )
}

pub fn phase7_minimize_replay_plan(
    client: &mut impl Phase7MachineApiClient,
    session_id: SessionId,
    initial_snapshot: &MachineProofSnapshot,
    verified_replay_plan: Phase7ReplayPlan,
    verified_replay: MachineReplayOkFields,
    verified_response: MachineApiOkResponse<MachineVerifyOkFields>,
) -> Phase7MinimizationResult {
    let mut current_plan = verified_replay_plan;
    let mut current_replay = verified_replay;
    let mut current_verify = verified_response;
    let mut minimization_stats = Phase7MinimizationStats::default();

    for pass in Phase7MinimizationPassKind::ALL {
        minimization_stats.pass_kinds_attempted += 1;
        let mut changed = true;

        while changed {
            changed = false;
            let Some(step_edits) = phase7_make_step_edits_with_goal_indices(
                client,
                session_id.clone(),
                initial_snapshot,
                &current_plan,
            ) else {
                break;
            };

            for proposed_steps in phase7_minimization_proposals(pass, &step_edits) {
                let Some(rebuilt) = phase7_rebuild_replay_plan_from_step_edits(
                    client,
                    session_id.clone(),
                    initial_snapshot,
                    &current_plan,
                    &proposed_steps,
                ) else {
                    continue;
                };
                minimization_stats.rebuilt_plans += 1;

                minimization_stats.replay_attempts += 1;
                let replay_source = phase7_replay_request_json(session_id.clone(), &rebuilt);
                let Ok(MachineApiResponseEnvelope::Ok(replayed)) = client.replay(&replay_source)
                else {
                    continue;
                };
                if replayed.status != MachineApiResponseStatus::Ok {
                    continue;
                }

                minimization_stats.verify_attempts += 1;
                let verify_source = phase7_verify_request_json(
                    session_id.clone(),
                    replayed.endpoint_fields.final_snapshot_id,
                    replayed.endpoint_fields.final_state_fingerprint,
                );
                let Ok(MachineApiResponseEnvelope::Ok(verified)) = client.verify(&verify_source)
                else {
                    continue;
                };
                if verified.status != MachineApiResponseStatus::Verified {
                    continue;
                }

                current_plan = rebuilt;
                current_replay = replayed.endpoint_fields;
                current_verify = verified;
                minimization_stats.accepted_proposals += 1;
                changed = true;
                break;
            }
        }
    }

    Phase7MinimizationResult {
        replay_plan: current_plan,
        replay_response: current_replay,
        verify_response: current_verify,
        minimization_stats,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase7MinimizationPassKind {
    DeleteRedundantSteps,
    ReplaceBlocksWithSimpLiteEmpty,
    MinimizeExistingSimpLiteRules,
}

impl Phase7MinimizationPassKind {
    const ALL: [Self; 3] = [
        Self::DeleteRedundantSteps,
        Self::ReplaceBlocksWithSimpLiteEmpty,
        Self::MinimizeExistingSimpLiteRules,
    ];
}

fn phase7_minimization_proposals(
    pass: Phase7MinimizationPassKind,
    step_edits: &[Phase7ReplayStepEdit],
) -> Vec<Vec<Phase7ReplayStepEdit>> {
    match pass {
        Phase7MinimizationPassKind::DeleteRedundantSteps => {
            phase7_delete_redundant_steps_proposals(step_edits)
        }
        Phase7MinimizationPassKind::ReplaceBlocksWithSimpLiteEmpty => {
            phase7_replace_blocks_with_simp_lite_empty_proposals(step_edits)
        }
        Phase7MinimizationPassKind::MinimizeExistingSimpLiteRules => {
            phase7_minimize_existing_simp_lite_rules_proposals(step_edits)
        }
    }
}

fn phase7_delete_redundant_steps_proposals(
    step_edits: &[Phase7ReplayStepEdit],
) -> Vec<Vec<Phase7ReplayStepEdit>> {
    let mut proposals = Vec::new();
    for index in 0..step_edits.len() {
        let mut proposal = step_edits.to_vec();
        proposal.remove(index);
        if proposal != step_edits {
            proposals.push(proposal);
        }
    }
    proposals
}

fn phase7_replace_blocks_with_simp_lite_empty_proposals(
    step_edits: &[Phase7ReplayStepEdit],
) -> Vec<Vec<Phase7ReplayStepEdit>> {
    let mut proposals = Vec::new();
    for block_len in (1..=step_edits.len()).rev() {
        for start_index in 0..=step_edits.len() - block_len {
            let replacement_source = &step_edits[start_index];
            let mut proposal = Vec::new();
            proposal.extend_from_slice(&step_edits[..start_index]);
            proposal.push(Phase7ReplayStepEdit {
                original_goal_id: replacement_source.original_goal_id,
                original_open_goal_index: replacement_source.original_open_goal_index,
                candidate: MachineTacticCandidate::SimpLite { rules: Vec::new() },
                deterministic_budget: replacement_source.deterministic_budget,
            });
            proposal.extend_from_slice(&step_edits[start_index + block_len..]);
            if proposal != step_edits {
                proposals.push(proposal);
            }
        }
    }
    proposals
}

fn phase7_minimize_existing_simp_lite_rules_proposals(
    step_edits: &[Phase7ReplayStepEdit],
) -> Vec<Vec<Phase7ReplayStepEdit>> {
    let mut proposals = Vec::new();
    for step_index in 0..step_edits.len() {
        let MachineTacticCandidate::SimpLite { rules } = &step_edits[step_index].candidate else {
            continue;
        };
        for rule_index in 0..rules.len() {
            let mut reduced_rules = rules.clone();
            reduced_rules.remove(rule_index);
            let mut proposal = step_edits.to_vec();
            proposal[step_index].candidate = MachineTacticCandidate::SimpLite {
                rules: reduced_rules,
            };
            if proposal != step_edits {
                proposals.push(proposal);
            }
        }
    }
    proposals
}

fn phase7_make_step_edits_with_goal_indices(
    client: &mut impl Phase7MachineApiClient,
    session_id: SessionId,
    initial_snapshot: &MachineProofSnapshot,
    current_plan: &Phase7ReplayPlan,
) -> Option<Vec<Phase7ReplayStepEdit>> {
    let mut snapshot = phase7_minimization_initial_snapshot(
        client,
        session_id.clone(),
        initial_snapshot,
        current_plan,
    )?;
    let mut edits = Vec::new();

    for step in &current_plan.steps {
        let open_goal_index = snapshot
            .open_goals
            .iter()
            .position(|goal_id| *goal_id == step.goal_id)?;
        edits.push(Phase7ReplayStepEdit {
            original_goal_id: step.goal_id,
            original_open_goal_index: usize_to_u32(open_goal_index),
            candidate: step.candidate.clone(),
            deterministic_budget: step.deterministic_budget,
        });

        let (replayed_step, next_snapshot) = phase7_minimization_reexecute_step(
            client,
            session_id.clone(),
            &snapshot,
            step.goal_id,
            step.candidate.clone(),
            step.deterministic_budget,
        )?;
        if replayed_step.candidate_hash != step.candidate_hash
            || replayed_step.deterministic_budget_hash != step.deterministic_budget_hash
            || replayed_step.proof_delta_hash != step.proof_delta_hash
            || replayed_step.next_state_fingerprint != step.next_state_fingerprint
        {
            return None;
        }
        snapshot = next_snapshot;
    }

    Some(edits)
}

fn phase7_rebuild_replay_plan_from_step_edits(
    client: &mut impl Phase7MachineApiClient,
    session_id: SessionId,
    initial_snapshot: &MachineProofSnapshot,
    current_plan: &Phase7ReplayPlan,
    proposed_steps: &[Phase7ReplayStepEdit],
) -> Option<Phase7ReplayPlan> {
    let mut snapshot = phase7_minimization_initial_snapshot(
        client,
        session_id.clone(),
        initial_snapshot,
        current_plan,
    )?;
    let mut replay_steps = Vec::new();

    for edit in proposed_steps {
        let execution_goal_id = phase7_minimization_execution_goal_id(&snapshot, edit)?;
        let (step, next_snapshot) = phase7_minimization_reexecute_step(
            client,
            session_id.clone(),
            &snapshot,
            execution_goal_id,
            edit.candidate.clone(),
            edit.deterministic_budget,
        )?;
        replay_steps.push(step);
        snapshot = next_snapshot;
    }

    Some(Phase7ReplayPlan {
        protocol_version: current_plan.protocol_version,
        session_root_hash: current_plan.session_root_hash,
        initial_state_fingerprint: current_plan.initial_state_fingerprint,
        steps: replay_steps,
        final_state_fingerprint: snapshot.state_fingerprint,
    })
}

fn phase7_minimization_initial_snapshot(
    client: &mut impl Phase7MachineApiClient,
    session_id: SessionId,
    initial_snapshot: &MachineProofSnapshot,
    current_plan: &Phase7ReplayPlan,
) -> Option<MachineProofSnapshot> {
    if initial_snapshot.state_fingerprint != current_plan.initial_state_fingerprint {
        return None;
    }
    client
        .get_snapshot(Phase7SnapshotGetRequest {
            session_id,
            snapshot_id: initial_snapshot.snapshot_id,
            state_fingerprint: current_plan.initial_state_fingerprint,
        })
        .ok()
        .map(|ok| ok.snapshot)
}

fn phase7_minimization_execution_goal_id(
    snapshot: &MachineProofSnapshot,
    edit: &Phase7ReplayStepEdit,
) -> Option<GoalId> {
    if snapshot.open_goals.contains(&edit.original_goal_id) {
        return Some(edit.original_goal_id);
    }
    snapshot
        .open_goals
        .get(edit.original_open_goal_index as usize)
        .copied()
}

fn phase7_minimization_reexecute_step(
    client: &mut impl Phase7MachineApiClient,
    session_id: SessionId,
    snapshot: &MachineProofSnapshot,
    goal_id: GoalId,
    candidate: MachineTacticCandidate,
    deterministic_budget: TacticBudget,
) -> Option<(Phase7ReplayStep, MachineProofSnapshot)> {
    let request = Phase7TacticBatchRequest {
        session_id: session_id.clone(),
        snapshot_id: snapshot.snapshot_id,
        state_fingerprint: snapshot.state_fingerprint,
        goal_id,
        candidates: vec![Phase7AssignedCandidate {
            candidate_id: "c0".to_owned(),
            rank_index: 0,
            envelope: phase7_minimization_candidate_envelope(candidate),
        }],
        deterministic_budget,
        batch_policy: MachineTacticBatchPolicy {
            max_evaluated_candidates: 1,
            stop_after_successes: 1,
            stop_after_failures: 1,
        },
        scheduler_limits: None,
    };
    let evaluation = phase7_run_tactic_batch(client, &request).ok()?;
    if evaluation.scheduler_stop.is_some()
        || evaluation.evaluated_count != 1
        || !evaluation.accepted_failure_records.is_empty()
        || !evaluation.non_accepted_errors.is_empty()
    {
        return None;
    }
    let transition = evaluation.successful_transitions.into_iter().next()?;
    let next_snapshot = client
        .get_snapshot(Phase7SnapshotGetRequest {
            session_id,
            snapshot_id: transition.next_snapshot_id,
            state_fingerprint: transition.replay_step.next_state_fingerprint,
        })
        .ok()?
        .snapshot;
    Some((transition.replay_step, next_snapshot))
}

fn phase7_minimization_candidate_envelope(
    candidate: MachineTacticCandidate,
) -> Phase7CandidateEnvelope {
    let metadata = phase7_candidate_metadata(
        Phase7CandidateSource::Builtin,
        None,
        0,
        Vec::new(),
        Vec::new(),
        &candidate,
    );
    phase7_candidate_envelope(candidate, None, metadata)
}

fn phase7_validate_ok_batch_fields(
    request: &Phase7TacticBatchRequest,
    fields: &MachineTacticBatchOkFields,
) -> Phase7MachineControllerResult<()> {
    phase7_validate_batch_common_fields(
        request,
        fields.previous_state_fingerprint,
        fields.deterministic_budget_hash,
        &fields.results,
        fields.success_count,
        fields.failure_count,
    )
}

fn phase7_validate_scheduler_batch_fields(
    request: &Phase7TacticBatchRequest,
    fields: &MachineTacticBatchSchedulerFields,
    status: MachineApiResponseStatus,
) -> Phase7MachineControllerResult<()> {
    phase7_validate_batch_common_fields(
        request,
        fields.previous_state_fingerprint,
        fields.deterministic_budget_hash,
        &fields.results,
        fields.success_count,
        fields.failure_count,
    )?;
    if fields.completed_prefix_len as usize != fields.results.len() {
        return Err(phase7_batch_contract_violation(
            format!(
                "completed_prefix_len {} did not match result prefix length {}",
                fields.completed_prefix_len,
                fields.results.len()
            ),
            Some(status),
        ));
    }
    if fields.results.len() == request.candidates.len() {
        return Err(phase7_batch_contract_violation(
            "scheduler partial response completed every candidate".to_owned(),
            Some(status),
        ));
    }
    Ok(())
}

fn phase7_validate_batch_common_fields(
    request: &Phase7TacticBatchRequest,
    previous_state_fingerprint: Hash,
    deterministic_budget_hash: Hash,
    results: &[MachineTacticBatchItemResponse],
    success_count: u32,
    failure_count: u32,
) -> Phase7MachineControllerResult<()> {
    if previous_state_fingerprint != request.state_fingerprint {
        return Err(Box::new(Phase7MachineControllerError {
            kind: Phase7MachineControllerErrorKind::BatchResponseContractViolation,
            endpoint: Phase7MachineApiEndpointKind::TacticBatch,
            message: "batch previous_state_fingerprint did not match request".to_owned(),
            candidate_id: None,
            expected_hash: Some(request.state_fingerprint),
            actual_hash: Some(previous_state_fingerprint),
            diagnostic_hash: None,
            phase: None,
            status: None,
        }));
    }

    let expected_budget_hash = tactic_budget_hash(request.deterministic_budget);
    if deterministic_budget_hash != expected_budget_hash {
        return Err(Box::new(Phase7MachineControllerError {
            kind: Phase7MachineControllerErrorKind::BatchResponseContractViolation,
            endpoint: Phase7MachineApiEndpointKind::TacticBatch,
            message: "batch deterministic_budget_hash did not match request budget".to_owned(),
            candidate_id: None,
            expected_hash: Some(expected_budget_hash),
            actual_hash: Some(deterministic_budget_hash),
            diagnostic_hash: None,
            phase: None,
            status: None,
        }));
    }

    if results.len() > request.candidates.len() {
        return Err(phase7_batch_contract_violation(
            format!(
                "batch returned {} results for {} candidates",
                results.len(),
                request.candidates.len()
            ),
            None,
        ));
    }

    let mut seen_ids = BTreeSet::new();
    let mut actual_success_count = 0u32;
    let mut actual_failure_count = 0u32;
    for (index, item) in results.iter().enumerate() {
        let expected_id = &request.candidates[index].candidate_id;
        let actual_id = phase7_batch_item_candidate_id(item);
        if actual_id != expected_id {
            return Err(Box::new(Phase7MachineControllerError {
                kind: Phase7MachineControllerErrorKind::BatchResponseContractViolation,
                endpoint: Phase7MachineApiEndpointKind::TacticBatch,
                message: format!(
                    "batch result at index {index} used candidate_id {actual_id}, expected {expected_id}"
                ),
                candidate_id: Some(actual_id.to_owned()),
                expected_hash: None,
                actual_hash: None,
                diagnostic_hash: None,
                phase: None,
                status: None,
            }));
        }
        if !seen_ids.insert(actual_id) {
            return Err(phase7_batch_contract_violation(
                format!("batch repeated candidate_id {actual_id}"),
                None,
            ));
        }
        if item.is_success() {
            actual_success_count += 1;
        } else {
            actual_failure_count += 1;
        }
    }

    if success_count != actual_success_count || failure_count != actual_failure_count {
        return Err(phase7_batch_contract_violation(
            format!(
                "batch count fields reported {success_count} successes and {failure_count} failures, observed {actual_success_count} successes and {actual_failure_count} failures"
            ),
            None,
        ));
    }

    phase7_validate_candidate_hashes(request, results)
}

fn phase7_validate_candidate_hashes(
    request: &Phase7TacticBatchRequest,
    results: &[MachineTacticBatchItemResponse],
) -> Phase7MachineControllerResult<()> {
    for (index, item) in results.iter().enumerate() {
        let assigned = &request.candidates[index];
        let actual_hash = phase7_batch_item_candidate_hash(item);
        if phase7_candidate_hash_mismatch(&assigned.envelope, actual_hash) {
            return Err(Box::new(Phase7MachineControllerError {
                kind: Phase7MachineControllerErrorKind::SuggestedCandidateHashMismatch,
                endpoint: Phase7MachineApiEndpointKind::TacticBatch,
                message: "batch candidate_hash did not match suggested candidate envelope"
                    .to_owned(),
                candidate_id: Some(assigned.candidate_id.clone()),
                expected_hash: assigned.envelope.candidate_hash,
                actual_hash,
                diagnostic_hash: None,
                phase: None,
                status: None,
            }));
        }
    }
    Ok(())
}

fn phase7_build_batch_evaluation(
    request: &Phase7TacticBatchRequest,
    results: &[MachineTacticBatchItemResponse],
    deterministic_budget_hash: Hash,
    scheduler_stop: Option<Phase7SchedulerStop>,
    deferred_start: usize,
) -> Phase7BatchEvaluation {
    let mut successful_transitions = Vec::new();
    let mut accepted_failure_records = Vec::new();
    let mut replay_steps = Vec::new();
    let mut accepted_failures = Vec::new();
    let mut non_accepted_errors = Vec::new();

    for (index, item) in results.iter().enumerate() {
        let assigned = &request.candidates[index];
        match item {
            MachineTacticBatchItemResponse::Success {
                candidate_id,
                candidate_hash,
                next_snapshot_id,
                next_state_fingerprint,
                proof_delta_hash,
            } => {
                let replay_step = Phase7ReplayStep {
                    previous_state_fingerprint: request.state_fingerprint,
                    goal_id: request.goal_id,
                    candidate: assigned.envelope.candidate.clone(),
                    deterministic_budget: request.deterministic_budget,
                    candidate_hash: *candidate_hash,
                    deterministic_budget_hash,
                    proof_delta_hash: *proof_delta_hash,
                    next_state_fingerprint: *next_state_fingerprint,
                };
                replay_steps.push(replay_step.clone());
                successful_transitions.push(Phase7SuccessfulCandidateTransition {
                    candidate_id: candidate_id.clone(),
                    envelope: assigned.envelope.clone(),
                    next_snapshot_id: *next_snapshot_id,
                    replay_step,
                });
            }
            MachineTacticBatchItemResponse::Error {
                candidate_hash,
                diagnostic,
                ..
            } => {
                if let Some(failure) = phase7_normalize_accepted_candidate_failure(
                    diagnostic,
                    *candidate_hash,
                    deterministic_budget_hash,
                ) {
                    accepted_failure_records.push(Phase7AcceptedCandidateFailureRecord {
                        candidate_id: assigned.candidate_id.clone(),
                        envelope: assigned.envelope.clone(),
                        failure: failure.clone(),
                    });
                    accepted_failures.push(failure);
                } else {
                    non_accepted_errors.push(Phase7NonAcceptedCandidateError {
                        candidate_id: assigned.candidate_id.clone(),
                        phase7_candidate_payload_hash: assigned
                            .envelope
                            .phase7_candidate_payload_hash,
                        error_kind: diagnostic.error_kind,
                        phase: diagnostic.phase,
                        has_candidate_hash: candidate_hash.is_some(),
                    });
                }
            }
        }
    }

    Phase7BatchEvaluation {
        successful_transitions,
        accepted_failure_records,
        replay_steps,
        accepted_failures,
        non_accepted_errors,
        evaluated_count: usize_to_u32(results.len()),
        deferred_candidates: phase7_deferred_candidates(request, deferred_start),
        scheduler_stop,
    }
}

fn phase7_normalize_accepted_candidate_failure(
    diagnostic: &MachineApiCompactErrorWire,
    candidate_hash: Option<Hash>,
    deterministic_budget_hash: Hash,
) -> Option<Phase7AcceptedCandidateFailure> {
    Some(Phase7AcceptedCandidateFailure {
        error_kind: phase7_failed_candidate_error_kind(diagnostic.error_kind)?,
        phase: diagnostic.phase,
        goal_id: diagnostic.goal_id,
        tactic_kind: diagnostic.tactic_kind,
        candidate_hash: candidate_hash?,
        deterministic_budget_hash,
        diagnostic_hash: diagnostic.diagnostic_hash,
        retryable: diagnostic.retryable,
    })
}

fn phase7_failed_candidate_error_kind(
    kind: MachineApiErrorKind,
) -> Option<FailedCandidateErrorKind> {
    match kind {
        MachineApiErrorKind::UnsupportedTactic => Some(FailedCandidateErrorKind::UnsupportedTactic),
        MachineApiErrorKind::MachineTermElaborationError => {
            Some(FailedCandidateErrorKind::MachineTermElaborationError)
        }
        MachineApiErrorKind::UnknownName => Some(FailedCandidateErrorKind::UnknownName),
        MachineApiErrorKind::ImplicitArgumentRequired => {
            Some(FailedCandidateErrorKind::ImplicitArgumentRequired)
        }
        MachineApiErrorKind::TypeMismatch => Some(FailedCandidateErrorKind::TypeMismatch),
        MachineApiErrorKind::ExpectedPiType => Some(FailedCandidateErrorKind::ExpectedPiType),
        MachineApiErrorKind::RewriteRuleInvalid => {
            Some(FailedCandidateErrorKind::RewriteRuleInvalid)
        }
        MachineApiErrorKind::SimpNoProgress => Some(FailedCandidateErrorKind::SimpNoProgress),
        MachineApiErrorKind::InductionTargetNotNat => {
            Some(FailedCandidateErrorKind::InductionTargetNotNat)
        }
        MachineApiErrorKind::BudgetExceeded => Some(FailedCandidateErrorKind::BudgetExceeded),
        MachineApiErrorKind::TooManyGoals => Some(FailedCandidateErrorKind::TooManyGoals),
        MachineApiErrorKind::TooLargeTerm => Some(FailedCandidateErrorKind::TooLargeTerm),
        _ => None,
    }
}

fn phase7_batch_item_candidate_id(item: &MachineTacticBatchItemResponse) -> &str {
    match item {
        MachineTacticBatchItemResponse::Success { candidate_id, .. }
        | MachineTacticBatchItemResponse::Error { candidate_id, .. } => candidate_id,
    }
}

fn phase7_batch_item_candidate_hash(item: &MachineTacticBatchItemResponse) -> Option<Hash> {
    match item {
        MachineTacticBatchItemResponse::Success { candidate_hash, .. } => Some(*candidate_hash),
        MachineTacticBatchItemResponse::Error { candidate_hash, .. } => *candidate_hash,
    }
}

fn phase7_deferred_candidates(
    request: &Phase7TacticBatchRequest,
    start: usize,
) -> Vec<Phase7DeferredCandidate> {
    request
        .candidates
        .iter()
        .skip(start)
        .map(|assigned| Phase7DeferredCandidate {
            candidate_id: assigned.candidate_id.clone(),
            envelope: assigned.envelope.clone(),
        })
        .collect()
}

fn phase7_batch_contract_violation(
    message: String,
    status: Option<MachineApiResponseStatus>,
) -> Box<Phase7MachineControllerError> {
    Box::new(Phase7MachineControllerError {
        kind: Phase7MachineControllerErrorKind::BatchResponseContractViolation,
        endpoint: Phase7MachineApiEndpointKind::TacticBatch,
        message,
        candidate_id: None,
        expected_hash: None,
        actual_hash: None,
        diagnostic_hash: None,
        phase: None,
        status,
    })
}

fn phase7_tactic_budget_json(budget: TacticBudget) -> String {
    format!(
        r#"{{"max_tactic_steps":{},"max_whnf_steps":{},"max_conversion_steps":{},"max_rewrite_steps":{},"max_meta_allocations":{},"max_expr_nodes":{}}}"#,
        budget.max_tactic_steps,
        budget.max_whnf_steps,
        budget.max_conversion_steps,
        budget.max_rewrite_steps,
        budget.max_meta_allocations,
        budget.max_expr_nodes,
    )
}

fn phase7_batch_policy_json(policy: MachineTacticBatchPolicy) -> String {
    format!(
        r#"{{"max_evaluated_candidates":{},"stop_after_successes":{},"stop_after_failures":{}}}"#,
        policy.max_evaluated_candidates, policy.stop_after_successes, policy.stop_after_failures,
    )
}

fn phase7_scheduler_limits_json(limits: MachineBatchSchedulerLimits) -> String {
    let mut fields = Vec::new();
    if let Some(value) = limits.per_candidate_timeout_ms {
        fields.push(format!(r#""per_candidate_timeout_ms":{value}"#));
    }
    if let Some(value) = limits.batch_timeout_ms {
        fields.push(format!(r#""batch_timeout_ms":{value}"#));
    }
    if let Some(value) = limits.max_memory_mb {
        fields.push(format!(r#""max_memory_mb":{value}"#));
    }
    format!("{{{}}}", fields.join(","))
}

impl Phase7RuleBasedRepair {
    pub fn new() -> Self {
        Self
    }

    pub fn repair_candidate(
        self,
        goal: &MachineGoalView,
        failed_envelope: &Phase7CandidateEnvelope,
        failure: &Phase7AcceptedCandidateFailure,
        repair_depth: u32,
    ) -> Phase7RepairCandidateOutput {
        if repair_depth > 2 {
            return Phase7RepairCandidateOutput::default();
        }

        match phase7_rule_based_repair_action(failure.error_kind) {
            Phase7RuleBasedRepairAction::Noop => Phase7RepairCandidateOutput::default(),
            Phase7RuleBasedRepairAction::TrySimpLite => {
                phase7_simp_lite_repair_candidate(goal, failed_envelope, failure, repair_depth)
            }
        }
    }
}

pub fn phase7_rule_based_repair_action(
    kind: FailedCandidateErrorKind,
) -> Phase7RuleBasedRepairAction {
    match kind {
        FailedCandidateErrorKind::UnsupportedTactic
        | FailedCandidateErrorKind::MachineTermElaborationError
        | FailedCandidateErrorKind::UnknownName
        | FailedCandidateErrorKind::ImplicitArgumentRequired
        | FailedCandidateErrorKind::InductionTargetNotNat
        | FailedCandidateErrorKind::BudgetExceeded
        | FailedCandidateErrorKind::TooLargeTerm => Phase7RuleBasedRepairAction::Noop,
        FailedCandidateErrorKind::TypeMismatch
        | FailedCandidateErrorKind::ExpectedPiType
        | FailedCandidateErrorKind::RewriteRuleInvalid
        | FailedCandidateErrorKind::SimpNoProgress
        | FailedCandidateErrorKind::TooManyGoals => Phase7RuleBasedRepairAction::TrySimpLite,
    }
}

pub fn phase7_repair_depth_of(envelope: &Phase7CandidateEnvelope) -> u32 {
    envelope
        .metadata
        .repair
        .as_ref()
        .map_or(0, |repair| repair.repair_depth)
}

fn phase7_simp_lite_repair_candidate(
    goal: &MachineGoalView,
    failed_envelope: &Phase7CandidateEnvelope,
    failure: &Phase7AcceptedCandidateFailure,
    repair_depth: u32,
) -> Phase7RepairCandidateOutput {
    if !phase7_goal_allows_tactic(goal, MachineApiTacticKind::SimpLite) {
        return Phase7RepairCandidateOutput::default();
    }

    let chain_tried_payload_hashes = phase7_repair_chain_tried_payload_hashes(failed_envelope);
    let candidate = MachineTacticCandidate::SimpLite { rules: Vec::new() };
    let mut metadata = phase7_candidate_metadata(
        Phase7CandidateSource::Repair,
        None,
        0,
        Vec::new(),
        Vec::new(),
        &candidate,
    );
    metadata.display_text = Some("simp-lite".to_owned());
    metadata.repair = Some(Phase7CandidateRepairMetadata {
        parent_candidate_hash: failure.candidate_hash,
        error_kind: failure.error_kind,
        repair_depth,
        chain_tried_payload_hashes: chain_tried_payload_hashes.clone(),
    });
    let envelope = phase7_candidate_envelope(candidate, None, metadata);

    if chain_tried_payload_hashes.contains(&envelope.phase7_candidate_payload_hash) {
        return Phase7RepairCandidateOutput {
            pending: Vec::new(),
            repeated_candidate_payload_hashes: vec![envelope.phase7_candidate_payload_hash],
        };
    }

    Phase7RepairCandidateOutput {
        pending: vec![Phase7PendingCandidate {
            goal_id: goal.goal_id,
            repair_depth,
            parent_candidate_hash: failure.candidate_hash,
            error_kind: failure.error_kind,
            chain_tried_payload_hashes,
            candidate: envelope,
        }],
        repeated_candidate_payload_hashes: Vec::new(),
    }
}

fn phase7_repair_chain_tried_payload_hashes(envelope: &Phase7CandidateEnvelope) -> Vec<Hash> {
    let mut chain = envelope
        .metadata
        .repair
        .as_ref()
        .map_or_else(Vec::new, |repair| repair.chain_tried_payload_hashes.clone());
    chain.push(envelope.phase7_candidate_payload_hash);
    chain
}

fn phase7_repeated_repair_error(
    envelope: &Phase7CandidateEnvelope,
    failure: &Phase7AcceptedCandidateFailure,
) -> bool {
    envelope
        .metadata
        .repair
        .as_ref()
        .is_some_and(|repair| repair.error_kind == failure.error_kind)
}

fn phase7_limit_repairs(
    pending_repairs: Vec<Phase7PendingCandidate>,
) -> Vec<Phase7PendingCandidate> {
    let mut seen_payloads = BTreeSet::new();
    let mut per_parent_counts: BTreeMap<(GoalId, Hash, FailedCandidateErrorKind), u32> =
        BTreeMap::new();
    let mut out = Vec::new();

    for pending in pending_repairs {
        if !seen_payloads.insert(pending.candidate.phase7_candidate_payload_hash) {
            continue;
        }

        let key = (
            pending.goal_id,
            pending.parent_candidate_hash,
            pending.error_kind,
        );
        let count = per_parent_counts.entry(key).or_insert(0);
        if *count >= 3 {
            continue;
        }
        *count += 1;
        out.push(pending);
    }

    out
}

fn phase7_merge_node_candidates(
    deferred_candidates: &mut Vec<Phase7DeferredCandidate>,
    pending_repairs: &mut Vec<Phase7PendingCandidate>,
    fresh_candidates: &mut Vec<Phase7CandidateEnvelope>,
) -> Vec<Phase7CandidateEnvelope> {
    let mut candidates = Vec::new();
    candidates.extend(
        deferred_candidates
            .drain(..)
            .map(|deferred| deferred.envelope),
    );

    let mut repairs = phase7_limit_repairs(std::mem::take(pending_repairs));
    repairs.sort_by(phase7_pending_candidate_order);
    candidates.extend(repairs.into_iter().map(|pending| pending.candidate));

    candidates.extend(std::mem::take(fresh_candidates));
    phase7_dedupe_candidate_envelopes(candidates)
}

fn phase7_pending_candidate_order(
    left: &Phase7PendingCandidate,
    right: &Phase7PendingCandidate,
) -> Ordering {
    left.repair_depth
        .cmp(&right.repair_depth)
        .then_with(|| left.parent_candidate_hash.cmp(&right.parent_candidate_hash))
        .then_with(|| left.error_kind.as_str().cmp(right.error_kind.as_str()))
        .then_with(|| {
            left.candidate
                .phase7_candidate_payload_hash
                .cmp(&right.candidate.phase7_candidate_payload_hash)
        })
}

pub fn phase7_search_node_priority_key(node: &Phase7SearchNode) -> Phase7SearchPriorityKey {
    Phase7SearchPriorityKey {
        open_goal_count: usize_to_u32(node.goals.len()),
        depth: node.depth,
        replay_step_count: usize_to_u32(node.replay_steps.len()),
        total_open_goal_target_size: phase7_total_open_goal_target_size(&node.goals),
        state_fingerprint: node.state_fingerprint,
        node_id: node.node_id,
    }
}

pub fn phase7_search_node_best_partial_key(node: &Phase7SearchNode) -> Phase7BestPartialKey {
    Phase7BestPartialKey {
        open_goal_count: usize_to_u32(node.goals.len()),
        total_open_goal_target_size: phase7_total_open_goal_target_size(&node.goals),
        replay_step_count: usize_to_u32(node.replay_steps.len()),
        depth: node.depth,
        state_fingerprint: node.state_fingerprint,
        node_id: node.node_id,
    }
}

pub fn phase7_run_mvp_search(
    client: &mut impl Phase7MachineApiClient,
    input: Phase7SearchInput,
) -> Phase7SearchResult {
    let mut node_ids = Phase7NodeIdAllocator::new();
    let mut queue = Phase7SearchPriorityQueue::new();
    let mut discovered_states = BTreeSet::new();
    let mut stats = Phase7SearchStats::default();
    let mut trace = Phase7TraceBuilder::new();
    let mut best_partial: Option<Phase7SearchNode> = None;
    let mut failure_reason = Phase7SearchFailureReason::QueueExhausted;
    let mut depth_budget_hit = false;
    let mut initial_no_candidate_goal = None;

    let root = phase7_root_search_node(&input, node_ids.allocate());
    discovered_states.insert(root.state_fingerprint);
    queue.push(root);

    while let Some(mut node) = queue.pop_best() {
        node.status = Phase7NodeStatus::Expanded;
        trace.push(&node, Phase7SearchTraceEventKind::NodeExpanded);

        let snapshot = match client.get_snapshot(Phase7SnapshotGetRequest {
            session_id: node.session_id.clone(),
            snapshot_id: node.snapshot_id,
            state_fingerprint: node.state_fingerprint,
        }) {
            Ok(ok) => ok.snapshot,
            Err(error) => {
                stats.controller_errors += 1;
                let reason = phase7_search_failure_reason_from_machine_api_error(
                    Phase7MachineApiEndpointKind::SnapshotGet,
                    &error,
                );
                trace.push(
                    &node,
                    phase7_machine_controller_trace_kind_from_reason(&reason),
                );
                return Err(phase7_search_failure(
                    reason,
                    best_partial,
                    stats,
                    trace.finish(),
                ));
            }
        };
        node.goals = phase7_goal_summaries(&snapshot);

        if node.goals.is_empty() {
            match phase7_attempt_closed_node(client, &node, &mut stats, &mut trace) {
                Phase7ClosedNodeOutcome::Verified(verified) => {
                    let minimization = phase7_minimize_replay_plan(
                        client,
                        node.session_id.clone(),
                        &input.initial_snapshot,
                        verified.replay_plan,
                        verified.replay_response,
                        verified.verify_response,
                    );
                    return Ok(Phase7VerifiedProof {
                        replay_plan: minimization.replay_plan,
                        final_snapshot_id: minimization.replay_response.final_snapshot_id,
                        final_state_fingerprint: minimization
                            .replay_response
                            .final_state_fingerprint,
                        verify_response: minimization.verify_response,
                        search_stats: stats,
                        minimization_stats: minimization.minimization_stats,
                        trace_events: trace.finish(),
                    });
                }
                Phase7ClosedNodeOutcome::Rejected => continue,
                Phase7ClosedNodeOutcome::ControllerError { reason } => {
                    return Err(phase7_search_failure(
                        reason,
                        best_partial,
                        stats,
                        trace.finish(),
                    ));
                }
            }
        }

        if best_partial.as_ref().is_none_or(|best| {
            phase7_search_node_best_partial_key(&node) < phase7_search_node_best_partial_key(best)
        }) {
            best_partial = Some(node.clone());
            stats.best_partial_updates += 1;
        }

        if node.depth >= input.search_budget.max_depth {
            depth_budget_hit = true;
            stats.max_depth_stops += 1;
            trace.push(
                &node,
                Phase7SearchTraceEventKind::MaxDepthStopped {
                    max_depth: input.search_budget.max_depth,
                },
            );
            continue;
        }

        if stats.nodes_expanded >= input.search_budget.max_nodes {
            failure_reason = Phase7SearchFailureReason::SearchBudgetExceeded {
                limit: Phase7SearchBudgetLimit::MaxNodes,
            };
            break;
        }
        stats.nodes_expanded += 1;

        let Some(goal_summary) = select_phase7_goal(&snapshot) else {
            stats.no_candidate_stops += 1;
            continue;
        };
        let goal_id = goal_summary.goal_id;
        let Some(goal) = snapshot.goals.iter().find(|goal| goal.goal_id == goal_id) else {
            stats.controller_errors += 1;
            let reason = Phase7SearchFailureReason::MachineControllerError {
                endpoint: phase7_machine_api_endpoint_wire(
                    Phase7MachineApiEndpointKind::SnapshotGet,
                )
                .to_owned(),
                error_kind: "invalid_machine_proof_state".to_owned(),
                error_phase: None,
                diagnostic_hash: None,
            };
            trace.push(
                &node,
                phase7_machine_controller_trace_kind_from_reason(&reason),
            );
            return Err(phase7_search_failure(
                reason,
                best_partial,
                stats,
                trace.finish(),
            ));
        };

        let retrieval = match retrieve_phase7_premises(
            client,
            &Phase7PremiseQueryRequest {
                session_id: node.session_id.clone(),
                snapshot_id: node.snapshot_id,
                state_fingerprint: node.state_fingerprint,
                goal_id,
            },
            node.session_root_hash,
        ) {
            Ok(retrieval) => retrieval,
            Err(error) => {
                stats.controller_errors += 1;
                let reason = phase7_search_failure_reason_from_machine_api_error(
                    Phase7MachineApiEndpointKind::SearchForGoal,
                    &error,
                );
                trace.push(
                    &node,
                    phase7_machine_controller_trace_kind_from_reason(&reason),
                );
                return Err(phase7_search_failure(
                    reason,
                    best_partial,
                    stats,
                    trace.finish(),
                ));
            }
        };

        let mut fresh_candidates = phase7_mvp_candidate_generation(goal, &retrieval).accepted;
        let mut deferred_candidates = Vec::new();
        let mut pending_repairs = Vec::new();
        let mut evaluated_for_node = 0u32;
        let repair = Phase7RuleBasedRepair::new();

        loop {
            let mut candidates = phase7_merge_node_candidates(
                &mut deferred_candidates,
                &mut pending_repairs,
                &mut fresh_candidates,
            );
            let remaining_tactic_budget = input
                .search_budget
                .max_tactics_per_node
                .saturating_sub(evaluated_for_node);
            let max_tactics_budget_reached_before_batch =
                remaining_tactic_budget == 0 && !candidates.is_empty();
            let candidates_exceeded_remaining_tactic_budget =
                candidates.len() > remaining_tactic_budget as usize;
            candidates =
                phase7_take_remaining_node_tactic_budget(candidates, remaining_tactic_budget);

            if candidates.is_empty() {
                if max_tactics_budget_reached_before_batch {
                    trace.push(
                        &node,
                        Phase7SearchTraceEventKind::MaxTacticsPerNodeStopped {
                            max_tactics_per_node: input.search_budget.max_tactics_per_node,
                        },
                    );
                } else if evaluated_for_node == 0 {
                    stats.no_candidate_stops += 1;
                    if node.parent.is_none() {
                        initial_no_candidate_goal = Some(goal_id);
                    }
                    trace.push(
                        &node,
                        Phase7SearchTraceEventKind::NoCandidateForSelectedGoal { goal_id },
                    );
                }
                break;
            }

            let batch_request = Phase7TacticBatchRequest {
                session_id: node.session_id.clone(),
                snapshot_id: node.snapshot_id,
                state_fingerprint: node.state_fingerprint,
                goal_id,
                candidates: phase7_assign_candidate_ids(candidates),
                deterministic_budget: input.per_tactic_deterministic_budget,
                batch_policy: input.batch_policy,
                scheduler_limits: input.scheduler_limits,
            };
            let evaluation = match phase7_run_tactic_batch(client, &batch_request) {
                Ok(evaluation) => evaluation,
                Err(error) => {
                    stats.controller_errors += 1;
                    let reason = phase7_search_failure_reason_from_tactic_batch_run_error(&error);
                    trace.push(
                        &node,
                        phase7_machine_controller_trace_kind_from_reason(&reason),
                    );
                    return Err(phase7_search_failure(
                        reason,
                        best_partial,
                        stats,
                        trace.finish(),
                    ));
                }
            };

            if let Some(scheduler_stop) = evaluation.scheduler_stop {
                stats.scheduler_stops += 1;
                if evaluation.evaluated_count == 0 {
                    stats.zero_progress_scheduler_stops += 1;
                    trace.push(
                        &node,
                        Phase7SearchTraceEventKind::ZeroProgressSchedulerStopped {
                            status: scheduler_stop.status,
                        },
                    );
                } else {
                    trace.push(
                        &node,
                        Phase7SearchTraceEventKind::SchedulerStopped {
                            status: scheduler_stop.status,
                            completed_prefix_len: scheduler_stop.completed_prefix_len,
                        },
                    );
                }
            }

            if evaluation.evaluated_count == 0
                && evaluation.scheduler_stop.is_none()
                && !evaluation.deferred_candidates.is_empty()
            {
                stats.controller_errors += 1;
                let error = phase7_batch_contract_violation(
                    "batch ok response did not evaluate any candidate".to_owned(),
                    None,
                );
                let reason = phase7_search_failure_reason_from_controller_error(&error);
                trace.push(
                    &node,
                    phase7_machine_controller_trace_kind_from_reason(&reason),
                );
                return Err(phase7_search_failure(
                    reason,
                    best_partial,
                    stats,
                    trace.finish(),
                ));
            }

            evaluated_for_node = evaluated_for_node
                .checked_add(evaluation.evaluated_count)
                .expect("phase7 evaluated candidates for node fits in u32");
            stats.candidates_evaluated += u64::from(evaluation.evaluated_count);

            for transition in evaluation.successful_transitions {
                if discovered_states.contains(&transition.replay_step.next_state_fingerprint) {
                    trace.push(
                        &node,
                        Phase7SearchTraceEventKind::DuplicateStateSkipped {
                            duplicate_state_fingerprint: transition
                                .replay_step
                                .next_state_fingerprint,
                        },
                    );
                    continue;
                }

                let child_snapshot = match client.get_snapshot(Phase7SnapshotGetRequest {
                    session_id: node.session_id.clone(),
                    snapshot_id: transition.next_snapshot_id,
                    state_fingerprint: transition.replay_step.next_state_fingerprint,
                }) {
                    Ok(ok) => ok.snapshot,
                    Err(error) => {
                        stats.controller_errors += 1;
                        let reason = phase7_search_failure_reason_from_machine_api_error(
                            Phase7MachineApiEndpointKind::SnapshotGet,
                            &error,
                        );
                        trace.push(
                            &node,
                            phase7_machine_controller_trace_kind_from_reason(&reason),
                        );
                        return Err(phase7_search_failure(
                            reason,
                            best_partial,
                            stats,
                            trace.finish(),
                        ));
                    }
                };

                let child_node_id = node_ids.allocate();
                let child = phase7_make_child_search_node(
                    &node,
                    child_node_id,
                    transition,
                    &child_snapshot,
                );
                discovered_states.insert(child.state_fingerprint);
                trace.push(
                    &node,
                    Phase7SearchTraceEventKind::ChildQueued {
                        child_node_id,
                        state_fingerprint: child.state_fingerprint,
                    },
                );
                queue.push(child);
            }

            let mut next_repairs = Vec::new();
            for record in evaluation.accepted_failure_records {
                if phase7_repeated_repair_error(&record.envelope, &record.failure) {
                    trace.push(
                        &node,
                        Phase7SearchTraceEventKind::RepairChainStopped {
                            parent_candidate_hash: record.failure.candidate_hash,
                            error_kind: record.failure.error_kind,
                            repair_depth: phase7_repair_depth_of(&record.envelope),
                            reason: Phase7RepairChainStopReason::RepeatedError,
                            repeated_candidate_payload_hash: None,
                        },
                    );
                    continue;
                }

                let parent_repair_depth = phase7_repair_depth_of(&record.envelope);
                if parent_repair_depth >= 2 {
                    trace.push(
                        &node,
                        Phase7SearchTraceEventKind::RepairChainStopped {
                            parent_candidate_hash: record.failure.candidate_hash,
                            error_kind: record.failure.error_kind,
                            repair_depth: parent_repair_depth,
                            reason: Phase7RepairChainStopReason::MaxRepairDepth,
                            repeated_candidate_payload_hash: None,
                        },
                    );
                    continue;
                }

                let repair_output = repair.repair_candidate(
                    goal,
                    &record.envelope,
                    &record.failure,
                    parent_repair_depth + 1,
                );
                for repeated_hash in repair_output.repeated_candidate_payload_hashes {
                    trace.push(
                        &node,
                        Phase7SearchTraceEventKind::RepairChainStopped {
                            parent_candidate_hash: record.failure.candidate_hash,
                            error_kind: record.failure.error_kind,
                            repair_depth: parent_repair_depth,
                            reason: Phase7RepairChainStopReason::RepeatedCandidate,
                            repeated_candidate_payload_hash: Some(repeated_hash),
                        },
                    );
                }
                next_repairs.extend(repair_output.pending);
            }
            pending_repairs = phase7_limit_repairs(next_repairs);

            deferred_candidates = evaluation.deferred_candidates;
            if deferred_candidates.is_empty() && pending_repairs.is_empty() {
                if candidates_exceeded_remaining_tactic_budget
                    && evaluated_for_node >= input.search_budget.max_tactics_per_node
                {
                    trace.push(
                        &node,
                        Phase7SearchTraceEventKind::MaxTacticsPerNodeStopped {
                            max_tactics_per_node: input.search_budget.max_tactics_per_node,
                        },
                    );
                }
                break;
            }
            if evaluated_for_node >= input.search_budget.max_tactics_per_node {
                trace.push(
                    &node,
                    Phase7SearchTraceEventKind::MaxTacticsPerNodeStopped {
                        max_tactics_per_node: input.search_budget.max_tactics_per_node,
                    },
                );
                break;
            }
        }
    }

    if let Some(goal_id) = initial_no_candidate_goal.filter(|_| {
        best_partial
            .as_ref()
            .is_some_and(|partial| partial.parent.is_none())
    }) {
        failure_reason = Phase7SearchFailureReason::NoCandidateForSelectedGoal { goal_id };
    } else if matches!(failure_reason, Phase7SearchFailureReason::QueueExhausted)
        && depth_budget_hit
    {
        failure_reason = Phase7SearchFailureReason::SearchBudgetExceeded {
            limit: Phase7SearchBudgetLimit::MaxDepth,
        };
    }

    Err(phase7_search_failure(
        failure_reason,
        best_partial,
        stats,
        trace.finish(),
    ))
}

struct Phase7ClosedNodeVerified {
    replay_plan: Phase7ReplayPlan,
    replay_response: MachineReplayOkFields,
    verify_response: MachineApiOkResponse<MachineVerifyOkFields>,
}

enum Phase7ClosedNodeOutcome {
    Verified(Box<Phase7ClosedNodeVerified>),
    Rejected,
    ControllerError { reason: Phase7SearchFailureReason },
}

fn phase7_attempt_closed_node(
    client: &mut impl Phase7MachineApiClient,
    node: &Phase7SearchNode,
    stats: &mut Phase7SearchStats,
    trace: &mut Phase7TraceBuilder,
) -> Phase7ClosedNodeOutcome {
    let replay_plan = phase7_build_replay_plan(node);
    let replay_source = phase7_replay_request_json(node.session_id.clone(), &replay_plan);
    let replay_response = match client.replay(&replay_source) {
        Ok(response) => match response {
            MachineApiResponseEnvelope::Ok(ok) => {
                if ok.status == MachineApiResponseStatus::Ok {
                    ok.endpoint_fields
                } else {
                    phase7_record_closed_node_replay_rejection(node, stats, trace, ok.status);
                    return Phase7ClosedNodeOutcome::Rejected;
                }
            }
            MachineApiResponseEnvelope::SchedulerStopped(stop) => {
                phase7_record_closed_node_replay_rejection(node, stats, trace, stop.status);
                return Phase7ClosedNodeOutcome::Rejected;
            }
            MachineApiResponseEnvelope::Error(error) => {
                if phase7_is_replay_controller_error_wire(&error.error) {
                    return phase7_closed_node_controller_error(
                        node,
                        stats,
                        trace,
                        phase7_machine_controller_error_reason_from_wire(
                            Phase7MachineApiEndpointKind::Replay,
                            error.error.kind.as_str(),
                            Some(error.error.phase.as_str()),
                            Some(error.error.diagnostic_hash),
                        ),
                    );
                }
                phase7_record_closed_node_replay_rejection(node, stats, trace, error.status);
                return Phase7ClosedNodeOutcome::Rejected;
            }
        },
        Err(error) => {
            if phase7_is_replay_controller_error(&error) {
                return phase7_closed_node_controller_error(
                    node,
                    stats,
                    trace,
                    phase7_search_failure_reason_from_machine_api_error(
                        Phase7MachineApiEndpointKind::Replay,
                        &error,
                    ),
                );
            }
            phase7_record_closed_node_replay_rejection(
                node,
                stats,
                trace,
                phase7_replay_error_status(&error),
            );
            return Phase7ClosedNodeOutcome::Rejected;
        }
    };

    let verify_source = phase7_verify_request_json(
        node.session_id.clone(),
        replay_response.final_snapshot_id,
        replay_response.final_state_fingerprint,
    );
    let verify_response = match client.verify(&verify_source) {
        Ok(response) => match response {
            MachineApiResponseEnvelope::Ok(ok) => {
                if ok.status == MachineApiResponseStatus::Verified {
                    ok
                } else {
                    phase7_record_closed_node_verify_rejection(node, stats, trace, ok.status);
                    return Phase7ClosedNodeOutcome::Rejected;
                }
            }
            MachineApiResponseEnvelope::SchedulerStopped(stop) => {
                phase7_record_closed_node_verify_rejection(node, stats, trace, stop.status);
                return Phase7ClosedNodeOutcome::Rejected;
            }
            MachineApiResponseEnvelope::Error(error) => {
                if phase7_is_verify_controller_error_wire(&error.error) {
                    return phase7_closed_node_controller_error(
                        node,
                        stats,
                        trace,
                        phase7_machine_controller_error_reason_from_wire(
                            Phase7MachineApiEndpointKind::Verify,
                            error.error.kind.as_str(),
                            Some(error.error.phase.as_str()),
                            Some(error.error.diagnostic_hash),
                        ),
                    );
                }
                phase7_record_closed_node_verify_rejection(node, stats, trace, error.status);
                return Phase7ClosedNodeOutcome::Rejected;
            }
        },
        Err(error) => {
            if phase7_is_verify_controller_error(&error) {
                return phase7_closed_node_controller_error(
                    node,
                    stats,
                    trace,
                    phase7_search_failure_reason_from_machine_api_error(
                        Phase7MachineApiEndpointKind::Verify,
                        &error,
                    ),
                );
            }
            phase7_record_closed_node_verify_rejection(
                node,
                stats,
                trace,
                phase7_verify_error_status(&error),
            );
            return Phase7ClosedNodeOutcome::Rejected;
        }
    };

    Phase7ClosedNodeOutcome::Verified(Box::new(Phase7ClosedNodeVerified {
        replay_plan,
        replay_response,
        verify_response,
    }))
}

fn phase7_closed_node_controller_error(
    node: &Phase7SearchNode,
    stats: &mut Phase7SearchStats,
    trace: &mut Phase7TraceBuilder,
    reason: Phase7SearchFailureReason,
) -> Phase7ClosedNodeOutcome {
    stats.controller_errors += 1;
    trace.push(
        node,
        phase7_machine_controller_trace_kind_from_reason(&reason),
    );
    Phase7ClosedNodeOutcome::ControllerError { reason }
}

fn phase7_record_closed_node_replay_rejection(
    node: &Phase7SearchNode,
    stats: &mut Phase7SearchStats,
    trace: &mut Phase7TraceBuilder,
    status: MachineApiResponseStatus,
) {
    stats.closed_node_replay_rejections += 1;
    trace.push(
        node,
        Phase7SearchTraceEventKind::ClosedNodeReplayRejected {
            endpoint: phase7_machine_api_endpoint_wire(Phase7MachineApiEndpointKind::Replay)
                .to_owned(),
            status,
        },
    );
}

fn phase7_record_closed_node_verify_rejection(
    node: &Phase7SearchNode,
    stats: &mut Phase7SearchStats,
    trace: &mut Phase7TraceBuilder,
    status: MachineApiResponseStatus,
) {
    stats.closed_node_verify_rejections += 1;
    trace.push(
        node,
        Phase7SearchTraceEventKind::ClosedNodeVerifyRejected {
            endpoint: phase7_machine_api_endpoint_wire(Phase7MachineApiEndpointKind::Verify)
                .to_owned(),
            status,
        },
    );
}

fn phase7_is_replay_controller_error(error: &Phase7MachineApiError) -> bool {
    match error {
        Phase7MachineApiError::Replay(error) => {
            phase7_is_replay_controller_error_kind(error.diagnostic.kind)
        }
        _ => true,
    }
}

fn phase7_is_replay_controller_error_wire(error: &MachineApiErrorWire) -> bool {
    phase7_is_replay_controller_error_kind(error.kind)
}

fn phase7_is_replay_controller_error_kind(kind: MachineApiErrorKind) -> bool {
    matches!(
        kind,
        MachineApiErrorKind::InvalidReplayPlan
            | MachineApiErrorKind::UnknownSession
            | MachineApiErrorKind::SessionRootHashMismatch
            | MachineApiErrorKind::StateFingerprintMismatch
            | MachineApiErrorKind::ReplayHashMismatch
            | MachineApiErrorKind::InvalidMachineProofState
    )
}

fn phase7_is_verify_controller_error(error: &Phase7MachineApiError) -> bool {
    match error {
        Phase7MachineApiError::Verify(error) => {
            phase7_is_verify_controller_error_kind(error.diagnostic.kind, error.diagnostic.phase)
        }
        _ => true,
    }
}

fn phase7_is_verify_controller_error_wire(error: &MachineApiErrorWire) -> bool {
    phase7_is_verify_controller_error_kind(error.kind, error.phase)
}

fn phase7_is_verify_controller_error_kind(
    kind: MachineApiErrorKind,
    phase: crate::MachineApiDiagnosticPhase,
) -> bool {
    matches!(
        (kind, phase),
        (
            MachineApiErrorKind::InvalidVerifyRequest,
            crate::MachineApiDiagnosticPhase::RequestValidation
        )
    ) || matches!(
        kind,
        MachineApiErrorKind::UnknownSession
            | MachineApiErrorKind::UnknownSnapshot
            | MachineApiErrorKind::StateFingerprintMismatch
            | MachineApiErrorKind::InvalidMachineProofState
    )
}

fn phase7_replay_error_status(error: &Phase7MachineApiError) -> MachineApiResponseStatus {
    match error {
        Phase7MachineApiError::Replay(error) => phase7_replay_response_status(&error.response),
        _ => MachineApiResponseStatus::Error,
    }
}

fn phase7_verify_error_status(error: &Phase7MachineApiError) -> MachineApiResponseStatus {
    match error {
        Phase7MachineApiError::Verify(error) => phase7_verify_response_status(&error.response),
        _ => MachineApiResponseStatus::Error,
    }
}

fn phase7_replay_response_status(response: &MachineReplayResponse) -> MachineApiResponseStatus {
    match response {
        MachineApiResponseEnvelope::Ok(ok) => ok.status,
        MachineApiResponseEnvelope::Error(error) => error.status,
        MachineApiResponseEnvelope::SchedulerStopped(stop) => stop.status,
    }
}

fn phase7_verify_response_status(response: &MachineVerifyResponse) -> MachineApiResponseStatus {
    match response {
        MachineApiResponseEnvelope::Ok(ok) => ok.status,
        MachineApiResponseEnvelope::Error(error) => error.status,
        MachineApiResponseEnvelope::SchedulerStopped(stop) => stop.status,
    }
}

fn phase7_root_search_node(input: &Phase7SearchInput, node_id: Phase7NodeId) -> Phase7SearchNode {
    Phase7SearchNode {
        node_id,
        session_id: input.session_id.clone(),
        session_root_hash: input.session_root_hash,
        initial_state_fingerprint: input.initial_snapshot.state_fingerprint,
        snapshot_id: input.initial_snapshot.snapshot_id,
        state_fingerprint: input.initial_snapshot.state_fingerprint,
        goals: phase7_goal_summaries(&input.initial_snapshot),
        replay_steps: Vec::new(),
        depth: 0,
        cumulative_score: 0,
        last_candidate: None,
        last_candidate_hash: None,
        used_premises: Vec::new(),
        parent: None,
        status: Phase7NodeStatus::Queued,
    }
}

fn phase7_make_child_search_node(
    parent: &Phase7SearchNode,
    node_id: Phase7NodeId,
    transition: Phase7SuccessfulCandidateTransition,
    child_snapshot: &MachineProofSnapshot,
) -> Phase7SearchNode {
    let mut replay_steps = parent.replay_steps.clone();
    replay_steps.push(transition.replay_step.clone());
    Phase7SearchNode {
        node_id,
        session_id: parent.session_id.clone(),
        session_root_hash: parent.session_root_hash,
        initial_state_fingerprint: parent.initial_state_fingerprint,
        snapshot_id: transition.next_snapshot_id,
        state_fingerprint: transition.replay_step.next_state_fingerprint,
        goals: phase7_goal_summaries(child_snapshot),
        replay_steps,
        depth: parent
            .depth
            .checked_add(1)
            .expect("phase7 search depth fits in u32"),
        cumulative_score: parent.cumulative_score,
        last_candidate: Some(transition.envelope.candidate.clone()),
        last_candidate_hash: Some(transition.replay_step.candidate_hash),
        used_premises: phase7_append_unique_premises(
            &parent.used_premises,
            &transition.envelope.metadata.premises_used,
        ),
        parent: Some(parent.node_id),
        status: Phase7NodeStatus::Queued,
    }
}

fn phase7_append_unique_premises(
    current: &[Phase7PremiseUsage],
    next: &[Phase7PremiseUsage],
) -> Vec<Phase7PremiseUsage> {
    let mut out = current.to_vec();
    for premise in next {
        if !out.contains(premise) {
            out.push(premise.clone());
        }
    }
    out
}

fn phase7_take_remaining_node_tactic_budget(
    candidates: Vec<Phase7CandidateEnvelope>,
    remaining: u32,
) -> Vec<Phase7CandidateEnvelope> {
    candidates.into_iter().take(remaining as usize).collect()
}

fn phase7_total_open_goal_target_size(goals: &[Phase7GoalSummary]) -> u64 {
    goals.iter().map(|goal| u64::from(goal.expr_size)).sum()
}

fn phase7_search_failure(
    reason: Phase7SearchFailureReason,
    best_partial: Option<Phase7SearchNode>,
    search_stats: Phase7SearchStats,
    trace_events: Vec<Phase7SearchTraceEvent>,
) -> Box<Phase7SearchFailure> {
    let (best_partial_replay_prefix, best_snapshot_id, best_state_fingerprint, remaining_goals) =
        if let Some(node) = best_partial {
            (
                Some(node.replay_steps),
                Some(node.snapshot_id),
                Some(node.state_fingerprint),
                Some(node.goals),
            )
        } else {
            (None, None, None, None)
        };
    Box::new(Phase7SearchFailure {
        reason,
        best_partial_replay_prefix,
        best_snapshot_id,
        best_state_fingerprint,
        remaining_goals,
        search_stats,
        trace_events,
    })
}

fn phase7_search_failure_reason_from_tactic_batch_run_error(
    error: &Phase7TacticBatchRunError,
) -> Phase7SearchFailureReason {
    match error {
        Phase7TacticBatchRunError::MachineApi(error) => {
            phase7_search_failure_reason_from_machine_api_error(
                Phase7MachineApiEndpointKind::TacticBatch,
                error,
            )
        }
        Phase7TacticBatchRunError::Controller(error) => {
            phase7_search_failure_reason_from_controller_error(error)
        }
    }
}

fn phase7_search_failure_reason_from_controller_error(
    error: &Phase7MachineControllerError,
) -> Phase7SearchFailureReason {
    Phase7SearchFailureReason::MachineControllerError {
        endpoint: phase7_machine_api_endpoint_wire(error.endpoint).to_owned(),
        error_kind: phase7_machine_controller_error_kind_wire(error),
        error_phase: error.phase.map(|phase| phase.as_str().to_owned()),
        diagnostic_hash: error.diagnostic_hash,
    }
}

fn phase7_machine_controller_error_kind_wire(error: &Phase7MachineControllerError) -> String {
    match error.kind {
        Phase7MachineControllerErrorKind::TopLevelBatchError => error.message.clone(),
        Phase7MachineControllerErrorKind::BatchResponseContractViolation => {
            "batch_response_contract_violation".to_owned()
        }
        Phase7MachineControllerErrorKind::SuggestedCandidateHashMismatch => {
            "suggested_candidate_hash_mismatch".to_owned()
        }
    }
}

fn phase7_search_failure_reason_from_machine_api_error(
    fallback_endpoint: Phase7MachineApiEndpointKind,
    error: &Phase7MachineApiError,
) -> Phase7SearchFailureReason {
    match error {
        Phase7MachineApiError::SnapshotGet(error) => {
            phase7_machine_controller_error_reason_from_wire(
                fallback_endpoint,
                error.error.kind.as_str(),
                Some(error.error.phase.as_str()),
                Some(error.error.diagnostic_hash),
            )
        }
        Phase7MachineApiError::SearchForGoal(error) => {
            phase7_machine_controller_error_reason_from_diagnostic(
                fallback_endpoint,
                &error.diagnostic,
            )
        }
        Phase7MachineApiError::TacticBatch(error) => {
            phase7_machine_controller_error_reason_from_diagnostic(
                fallback_endpoint,
                &error.diagnostic,
            )
        }
        Phase7MachineApiError::Replay(error) => {
            phase7_machine_controller_error_reason_from_diagnostic(
                fallback_endpoint,
                &error.diagnostic,
            )
        }
        Phase7MachineApiError::Verify(error) => {
            phase7_machine_controller_error_reason_from_diagnostic(
                fallback_endpoint,
                &error.diagnostic,
            )
        }
        Phase7MachineApiError::SearchForGoalResponse(error) => {
            phase7_machine_controller_error_reason_from_wire(
                fallback_endpoint,
                error.kind.as_str(),
                Some(error.phase.as_str()),
                Some(error.diagnostic_hash),
            )
        }
        Phase7MachineApiError::UnexpectedSchedulerStop { endpoint } => {
            phase7_machine_controller_error_reason_from_wire(
                *endpoint,
                "scheduler_stopped",
                None,
                None,
            )
        }
        Phase7MachineApiError::FakeRequestValidation { endpoint, error } => {
            phase7_machine_controller_error_reason_from_wire(
                *endpoint,
                error.kind.as_str(),
                Some(crate::MachineApiDiagnosticPhase::RequestValidation.as_str()),
                None,
            )
        }
        Phase7MachineApiError::FakeResponseExhausted { endpoint } => {
            phase7_machine_controller_error_reason_from_wire(
                *endpoint,
                "transport_error",
                None,
                None,
            )
        }
    }
}

fn phase7_machine_controller_error_reason_from_diagnostic(
    endpoint: Phase7MachineApiEndpointKind,
    diagnostic: &crate::MachineApiDiagnosticProjection,
) -> Phase7SearchFailureReason {
    phase7_machine_controller_error_reason_from_wire(
        endpoint,
        diagnostic.kind.as_str(),
        Some(diagnostic.phase.as_str()),
        diagnostic.diagnostic_hash().ok(),
    )
}

fn phase7_machine_controller_error_reason_from_wire(
    endpoint: Phase7MachineApiEndpointKind,
    error_kind: &str,
    error_phase: Option<&str>,
    diagnostic_hash: Option<Hash>,
) -> Phase7SearchFailureReason {
    Phase7SearchFailureReason::MachineControllerError {
        endpoint: phase7_machine_api_endpoint_wire(endpoint).to_owned(),
        error_kind: error_kind.to_owned(),
        error_phase: error_phase.map(str::to_owned),
        diagnostic_hash,
    }
}

fn phase7_machine_controller_trace_kind_from_reason(
    reason: &Phase7SearchFailureReason,
) -> Phase7SearchTraceEventKind {
    match reason {
        Phase7SearchFailureReason::MachineControllerError {
            endpoint,
            error_kind,
            ..
        } => Phase7SearchTraceEventKind::MachineControllerError {
            endpoint: endpoint.clone(),
            error_kind: error_kind.clone(),
        },
        _ => Phase7SearchTraceEventKind::MachineControllerError {
            endpoint: "phase7".to_owned(),
            error_kind: "controller_error".to_owned(),
        },
    }
}

fn phase7_machine_api_endpoint_wire(endpoint: Phase7MachineApiEndpointKind) -> &'static str {
    match endpoint {
        Phase7MachineApiEndpointKind::SnapshotGet => "/machine/snapshots/get",
        Phase7MachineApiEndpointKind::SearchForGoal => "/machine/search/for_goal",
        Phase7MachineApiEndpointKind::TacticBatch => "/machine/tactics/batch",
        Phase7MachineApiEndpointKind::Replay => "/machine/replay",
        Phase7MachineApiEndpointKind::Verify => "/machine/verify",
    }
}

#[derive(Clone, Debug, Default)]
struct Phase7NodeIdAllocator {
    next: u64,
}

impl Phase7NodeIdAllocator {
    fn new() -> Self {
        Self::default()
    }

    fn allocate(&mut self) -> Phase7NodeId {
        let node_id = Phase7NodeId(self.next);
        self.next = self
            .next
            .checked_add(1)
            .expect("phase7 node id fits in u64");
        node_id
    }
}

#[derive(Clone, Debug, Default)]
struct Phase7SearchPriorityQueue {
    nodes: Vec<Phase7SearchNode>,
}

impl Phase7SearchPriorityQueue {
    fn new() -> Self {
        Self::default()
    }

    fn push(&mut self, node: Phase7SearchNode) {
        self.nodes.push(node);
    }

    fn pop_best(&mut self) -> Option<Phase7SearchNode> {
        let best_index = self
            .nodes
            .iter()
            .enumerate()
            .min_by_key(|(_, node)| phase7_search_node_priority_key(node))
            .map(|(index, _)| index)?;
        Some(self.nodes.remove(best_index))
    }
}

#[derive(Clone, Debug, Default)]
struct Phase7TraceBuilder {
    events: Vec<Phase7SearchTraceEvent>,
}

impl Phase7TraceBuilder {
    fn new() -> Self {
        Self::default()
    }

    fn push(&mut self, node: &Phase7SearchNode, kind: Phase7SearchTraceEventKind) {
        self.events.push(Phase7SearchTraceEvent {
            event_index: u64::try_from(self.events.len())
                .expect("phase7 trace event count fits in u64"),
            node_id: node.node_id,
            kind,
        });
    }

    fn finish(self) -> Vec<Phase7SearchTraceEvent> {
        self.events
    }
}

pub fn phase7_snapshot_get_request_json(request: &Phase7SnapshotGetRequest) -> String {
    format!(
        r#"{{"session_id":"{}","snapshot_id":"{}","state_fingerprint":"{}","include_pretty":false}}"#,
        request.session_id.wire(),
        request.snapshot_id.wire(),
        format_hash_string(&request.state_fingerprint)
    )
}

pub fn load_phase7_initial_snapshot(
    client: &mut impl Phase7MachineApiClient,
    request: Phase7SnapshotGetRequest,
) -> Phase7MachineApiResult<Phase7InitialSnapshot> {
    let snapshot = client.get_snapshot(request)?.snapshot;
    let goals = phase7_goal_summaries(&snapshot);
    Ok(Phase7InitialSnapshot { snapshot, goals })
}

pub fn phase7_goal_summaries(snapshot: &MachineProofSnapshot) -> Vec<Phase7GoalSummary> {
    snapshot
        .goals
        .iter()
        .enumerate()
        .map(|(index, goal)| phase7_goal_summary(goal, index))
        .collect()
}

pub fn select_phase7_goal(snapshot: &MachineProofSnapshot) -> Option<Phase7GoalSummary> {
    phase7_goal_summaries(snapshot)
        .into_iter()
        .min_by(phase7_goal_selection_order)
}

pub fn phase7_mvp_premise_query_json(request: &Phase7PremiseQueryRequest) -> String {
    format!(
        r#"{{"session_id":"{}","snapshot_id":"{}","state_fingerprint":"{}","goal_id":"{}","modes":["exact","apply","rw","simp"],"limit":{},"filters":{{"exclude_axioms":true}}}}"#,
        request.session_id.wire(),
        request.snapshot_id.wire(),
        format_hash_string(&request.state_fingerprint),
        format_goal_id_wire(request.goal_id),
        PHASE7_MVP_PREMISE_QUERY_LIMIT
    )
}

pub fn retrieve_phase7_premises(
    client: &mut impl Phase7MachineApiClient,
    request: &Phase7PremiseQueryRequest,
    session_root_hash: Hash,
) -> Phase7MachineApiResult<Phase7PremiseRetrieval> {
    let source = phase7_mvp_premise_query_json(request);
    let response = client.search_for_goal(&source)?;
    match response {
        MachineApiResponseEnvelope::Ok(ok) => Ok(phase7_premise_retrieval_from_search_ok(
            session_root_hash,
            ok.endpoint_fields,
        )),
        MachineApiResponseEnvelope::Error(error) => Err(
            Phase7MachineApiError::SearchForGoalResponse(Box::new(error.error)),
        ),
        MachineApiResponseEnvelope::SchedulerStopped(_) => {
            Err(Phase7MachineApiError::UnexpectedSchedulerStop {
                endpoint: Phase7MachineApiEndpointKind::SearchForGoal,
            })
        }
    }
}

pub fn phase7_premise_retrieval_from_search_ok(
    session_root_hash: Hash,
    search: MachineTheoremSearchOkFields,
) -> Phase7PremiseRetrieval {
    let cache_key = phase7_retrieval_cache_key(session_root_hash, &search);
    let cache_entries = phase7_premise_cache_entries(&search);
    Phase7PremiseRetrieval {
        cache_key,
        cache_entries,
        results: search.results,
    }
}

pub fn phase7_retrieval_cache_key(
    session_root_hash: Hash,
    search: &MachineTheoremSearchOkFields,
) -> Phase7RetrievalCacheKey {
    Phase7RetrievalCacheKey {
        session_root_hash,
        query_fingerprint: search.query_fingerprint,
        theorem_index_fingerprint: search.theorem_index_fingerprint,
    }
}

pub fn phase7_premise_cache_entries(
    search: &MachineTheoremSearchOkFields,
) -> Vec<Phase7PremiseCacheEntry> {
    search
        .results
        .iter()
        .enumerate()
        .map(|(index, result)| phase7_premise_cache_entry(result, index))
        .collect()
}

pub fn phase7_premise_usages(search: &MachineTheoremSearchOkFields) -> Vec<Phase7PremiseUsage> {
    search.results.iter().map(phase7_premise_usage).collect()
}

pub fn phase7_mvp_candidate_envelopes(
    goal: &MachineGoalView,
    retrieval: &Phase7PremiseRetrieval,
) -> Vec<Phase7CandidateEnvelope> {
    phase7_mvp_candidate_generation(goal, retrieval).accepted
}

pub fn phase7_mvp_candidate_generation(
    goal: &MachineGoalView,
    retrieval: &Phase7PremiseRetrieval,
) -> Phase7CandidateFilterResult {
    let mut candidates = phase7_suggested_candidate_envelopes(&retrieval.results);
    candidates.extend(phase7_builtin_candidate_envelopes(goal));
    phase7_rank_filter_and_dedupe_candidate_envelopes(candidates)
}

pub fn phase7_suggested_candidate_envelopes(
    results: &[MachineTheoremSearchResult],
) -> Vec<Phase7CandidateEnvelope> {
    let mut out = Vec::new();
    let mut source_index = 0u32;
    for result in results {
        let premise_usage = phase7_premise_usage(result);
        for suggested in &result.suggested_candidates {
            let candidate = suggested.candidate.clone();
            let metadata = phase7_candidate_metadata(
                Phase7CandidateSource::Phase5Suggested,
                None,
                source_index,
                vec![premise_usage.clone()],
                result.axioms_used.clone(),
                &candidate,
            );
            let envelope =
                phase7_candidate_envelope(candidate, Some(suggested.candidate_hash), metadata);
            out.push(envelope);
            source_index = source_index
                .checked_add(1)
                .expect("phase7 suggested candidate source_index fits in u32");
        }
    }
    out
}

pub fn phase7_builtin_candidate_envelopes(goal: &MachineGoalView) -> Vec<Phase7CandidateEnvelope> {
    let mut out = Vec::new();

    if let Some(candidate) = phase7_builtin_intro_candidate(goal) {
        push_phase7_builtin_candidate(&mut out, Phase7BuiltinKind::Intro, 0, candidate);
    }

    let mut local_exact_index = 0u32;
    for local in &goal.context {
        if phase7_local_exact_prefilter(goal, local) {
            push_phase7_builtin_candidate(
                &mut out,
                Phase7BuiltinKind::LocalExact,
                local_exact_index,
                MachineTacticCandidate::Exact {
                    term: RawMachineTerm::new(local.machine_name.clone()),
                },
            );
            local_exact_index = local_exact_index
                .checked_add(1)
                .expect("phase7 local exact source_index fits in u32");
        }
    }

    let mut induction_index = 0u32;
    for (index, local) in goal.context.iter().enumerate() {
        if phase7_induction_nat_prefilter(goal, index, local) {
            push_phase7_builtin_candidate(
                &mut out,
                Phase7BuiltinKind::InductionNat,
                induction_index,
                MachineTacticCandidate::InductionNat {
                    local_name: local.machine_name.clone(),
                },
            );
            induction_index = induction_index
                .checked_add(1)
                .expect("phase7 induction source_index fits in u32");
        }
    }

    if phase7_goal_allows_tactic(goal, MachineApiTacticKind::SimpLite) {
        push_phase7_builtin_candidate(
            &mut out,
            Phase7BuiltinKind::SimpLiteEmpty,
            0,
            MachineTacticCandidate::SimpLite { rules: Vec::new() },
        );
    }

    out
}

pub fn phase7_rank_filter_and_dedupe_candidate_envelopes(
    mut candidates: Vec<Phase7CandidateEnvelope>,
) -> Phase7CandidateFilterResult {
    candidates.sort_by(phase7_candidate_envelope_order);
    let mut filtered = filter_phase7_candidate_envelopes(candidates);
    filtered.accepted = phase7_dedupe_candidate_envelopes(filtered.accepted);
    filtered
}

pub fn phase7_rank_and_dedupe_candidate_envelopes(
    mut candidates: Vec<Phase7CandidateEnvelope>,
) -> Vec<Phase7CandidateEnvelope> {
    candidates.sort_by(phase7_candidate_envelope_order);
    phase7_dedupe_candidate_envelopes(candidates)
}

fn phase7_dedupe_candidate_envelopes(
    candidates: Vec<Phase7CandidateEnvelope>,
) -> Vec<Phase7CandidateEnvelope> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for candidate in candidates {
        if seen.insert(candidate.phase7_candidate_payload_hash) {
            out.push(candidate);
        }
    }
    out
}

pub fn filter_phase7_candidate_envelopes(
    candidates: Vec<Phase7CandidateEnvelope>,
) -> Phase7CandidateFilterResult {
    let mut accepted = Vec::new();
    let mut rejected = Vec::new();
    let mut errors = Vec::new();
    for mut envelope in candidates {
        match phase7_candidate_forbidden_token(&envelope.candidate) {
            Ok(Some(forbidden_token)) => {
                envelope.metadata.trust_flags.contains_forbidden_tokens = true;
                envelope.metadata.trust_flags.forbidden_token_class = Some(forbidden_token.class);
                rejected.push(Phase7RejectedCandidateEnvelope {
                    envelope,
                    forbidden_token,
                });
            }
            Ok(None) => accepted.push(envelope),
            Err(error) => errors.push(error),
        }
    }
    Phase7CandidateFilterResult {
        accepted,
        rejected,
        errors,
    }
}

pub fn phase7_candidate_envelope(
    candidate: MachineTacticCandidate,
    candidate_hash: Option<Hash>,
    metadata: Phase7CandidateMetadata,
) -> Phase7CandidateEnvelope {
    Phase7CandidateEnvelope {
        phase7_candidate_payload_hash: phase7_candidate_payload_hash(&candidate),
        candidate,
        candidate_hash,
        metadata,
    }
}

pub fn phase7_candidate_payload_hash(candidate: &MachineTacticCandidate) -> Hash {
    let payload = phase7_candidate_payload_json(candidate);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(PHASE7_CANDIDATE_PAYLOAD_HASH_TAG.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(payload.as_bytes());
    sha256(&bytes)
}

pub fn phase7_candidate_payload_json(candidate: &MachineTacticCandidate) -> String {
    match candidate {
        MachineTacticCandidate::Exact { term } => {
            format!(
                r#"{{"kind":"exact","term":{}}}"#,
                phase7_raw_machine_term_json(term)
            )
        }
        MachineTacticCandidate::Intro { name } => {
            format!(r#"{{"kind":"intro","name":{}}}"#, json_string(name))
        }
        MachineTacticCandidate::Apply {
            head,
            universe_args,
            args,
        } => format!(
            r#"{{"args":{},"head":{},"kind":"apply","universe_args":{}}}"#,
            phase7_apply_arg_array_json(args),
            phase7_tactic_head_json(head),
            phase7_level_array_json(universe_args),
        ),
        MachineTacticCandidate::Rewrite {
            rule,
            direction,
            site,
        } => format!(
            r#"{{"direction":{},"kind":"rw","rule":{},"site":{}}}"#,
            json_string(phase7_rewrite_direction_wire(*direction)),
            phase7_rewrite_rule_json(rule),
            json_string(phase7_rewrite_site_wire(*site)),
        ),
        MachineTacticCandidate::SimpLite { rules } => {
            format!(
                r#"{{"kind":"simp-lite","rules":{}}}"#,
                phase7_simp_rule_array_json(rules)
            )
        }
        MachineTacticCandidate::InductionNat { local_name } => format!(
            r#"{{"kind":"induction-nat","local_name":{}}}"#,
            json_string(local_name)
        ),
    }
}

pub fn phase7_candidate_forbidden_token(
    candidate: &MachineTacticCandidate,
) -> Result<Option<Phase7ForbiddenToken>, Phase7CandidateFilterError> {
    let mut best_token = None;
    for (raw_term_index, term) in phase7_candidate_raw_terms(candidate)
        .into_iter()
        .enumerate()
    {
        if let Some(token) = phase7_raw_term_forbidden_token(usize_to_u32(raw_term_index), term)? {
            if phase7_forbidden_token_is_better(best_token.as_ref(), &token) {
                best_token = Some(token);
            }
        }
    }
    Ok(best_token)
}

pub fn phase7_expected_effect(candidate: &MachineTacticCandidate) -> Phase7ExpectedEffect {
    match candidate {
        MachineTacticCandidate::Intro { .. } => Phase7ExpectedEffect::IntroBinder,
        MachineTacticCandidate::Exact { .. } => Phase7ExpectedEffect::CloseGoal,
        MachineTacticCandidate::Rewrite { .. } => Phase7ExpectedEffect::Rewrite,
        MachineTacticCandidate::SimpLite { .. } => Phase7ExpectedEffect::Simplify,
        MachineTacticCandidate::InductionNat { .. } => Phase7ExpectedEffect::InductionSplit,
        MachineTacticCandidate::Apply { .. } => Phase7ExpectedEffect::Unknown,
    }
}

pub fn phase7_candidate_cost_estimate(
    candidate: &MachineTacticCandidate,
) -> Phase7CandidateCostEstimate {
    match candidate {
        MachineTacticCandidate::Intro { .. } | MachineTacticCandidate::Exact { .. } => {
            Phase7CandidateCostEstimate {
                estimated_timeout_ms: 100,
                risk: Phase7CandidateCostRisk::Low,
            }
        }
        MachineTacticCandidate::Rewrite { .. } => Phase7CandidateCostEstimate {
            estimated_timeout_ms: 200,
            risk: Phase7CandidateCostRisk::Medium,
        },
        MachineTacticCandidate::SimpLite { rules } if rules.is_empty() => {
            Phase7CandidateCostEstimate {
                estimated_timeout_ms: 100,
                risk: Phase7CandidateCostRisk::Low,
            }
        }
        MachineTacticCandidate::SimpLite { .. } => Phase7CandidateCostEstimate {
            estimated_timeout_ms: 200,
            risk: Phase7CandidateCostRisk::Medium,
        },
        MachineTacticCandidate::InductionNat { .. } => Phase7CandidateCostEstimate {
            estimated_timeout_ms: 500,
            risk: Phase7CandidateCostRisk::Medium,
        },
        MachineTacticCandidate::Apply { .. } => Phase7CandidateCostEstimate {
            estimated_timeout_ms: 500,
            risk: Phase7CandidateCostRisk::High,
        },
    }
}

pub fn phase7_fresh_intro_name(
    goal: &MachineGoalView,
    outer_binder_name: Option<&str>,
) -> Option<String> {
    let forbidden = goal
        .context
        .iter()
        .map(|local| local.machine_name.as_str())
        .collect::<BTreeSet<_>>();
    let mut bases = Vec::new();
    if let Some(name) = outer_binder_name {
        if is_machine_local_name(name) {
            bases.push(name.to_owned());
        }
    }
    bases.extend(["x".to_owned(), "h".to_owned(), "n".to_owned()]);

    for base in bases {
        let suffix_limit = forbidden.len().saturating_add(1);
        for suffix in 0..=suffix_limit {
            let candidate = if suffix == 0 {
                base.clone()
            } else {
                format!("{base}{suffix}")
            };
            if !is_machine_local_name(&candidate) {
                if suffix > 0 {
                    break;
                }
                continue;
            }
            if !forbidden.contains(candidate.as_str()) {
                return Some(candidate);
            }
        }
    }
    None
}

fn phase7_goal_summary(goal: &MachineGoalView, open_goal_index: usize) -> Phase7GoalSummary {
    Phase7GoalSummary {
        goal_id: goal.goal_id,
        open_goal_index: usize_to_u32(open_goal_index),
        goal_fingerprint: goal.goal_fingerprint,
        target_hash: goal.target_hash,
        target_head: goal.target.head.clone(),
        target_free_local_count: usize_to_u32(goal.target.free_locals.len()),
        context_size: usize_to_u32(goal.context.len()),
        expr_size: goal.target.size,
    }
}

fn phase7_goal_selection_order(left: &Phase7GoalSummary, right: &Phase7GoalSummary) -> Ordering {
    left.expr_size
        .cmp(&right.expr_size)
        .then_with(|| left.context_size.cmp(&right.context_size))
        .then_with(|| {
            left.target_free_local_count
                .cmp(&right.target_free_local_count)
        })
        .then_with(|| left.open_goal_index.cmp(&right.open_goal_index))
        .then_with(|| {
            goal_id_canonical_bytes(left.goal_id).cmp(&goal_id_canonical_bytes(right.goal_id))
        })
}

fn phase7_premise_cache_entry(
    result: &MachineTheoremSearchResult,
    response_index: usize,
) -> Phase7PremiseCacheEntry {
    Phase7PremiseCacheEntry {
        premise_ref: phase7_premise_ref(result),
        universe_params: result.universe_params.clone(),
        statement_core_hash: result.statement.core_hash,
        statement_head: result.statement.head.clone(),
        axioms_used: result.axioms_used.clone(),
        modes: result.modes.clone(),
        response_index: usize_to_u32(response_index),
    }
}

fn phase7_premise_usage(result: &MachineTheoremSearchResult) -> Phase7PremiseUsage {
    Phase7PremiseUsage {
        premise_ref: phase7_premise_ref(result),
        universe_params: result.universe_params.clone(),
        statement_core_hash: result.statement.core_hash,
        axioms_used: result.axioms_used.clone(),
    }
}

fn phase7_premise_ref(result: &MachineTheoremSearchResult) -> Phase7PremiseRef {
    Phase7PremiseRef {
        module: result.global_ref.module.clone(),
        name: result.global_ref.name.clone(),
        export_hash: result.global_ref.export_hash,
        decl_interface_hash: result.global_ref.decl_interface_hash,
    }
}

fn push_phase7_builtin_candidate(
    out: &mut Vec<Phase7CandidateEnvelope>,
    builtin_kind: Phase7BuiltinKind,
    source_index: u32,
    candidate: MachineTacticCandidate,
) {
    let metadata = phase7_candidate_metadata(
        Phase7CandidateSource::Builtin,
        Some(builtin_kind),
        source_index,
        Vec::new(),
        Vec::new(),
        &candidate,
    );
    let envelope = phase7_candidate_envelope(candidate, None, metadata);
    out.push(envelope);
}

fn phase7_candidate_metadata(
    source: Phase7CandidateSource,
    builtin_kind: Option<Phase7BuiltinKind>,
    source_index: u32,
    premises_used: Vec<Phase7PremiseUsage>,
    uses_axioms: Vec<MachineAxiomRefWire>,
    candidate: &MachineTacticCandidate,
) -> Phase7CandidateMetadata {
    Phase7CandidateMetadata {
        source,
        rank: Phase7CandidateRankMetadata {
            source_rank: phase7_candidate_source_rank(source),
            source_index,
            builtin_kind_rank: phase7_builtin_kind_rank(builtin_kind),
        },
        score: 0,
        display_text: None,
        premises_used,
        expected_effect: phase7_expected_effect(candidate),
        cost_estimate: phase7_candidate_cost_estimate(candidate),
        trust_flags: Phase7TrustFlags {
            uses_axioms,
            contains_forbidden_tokens: false,
            forbidden_token_class: None,
        },
        repair: None,
    }
}

fn phase7_candidate_source_rank(source: Phase7CandidateSource) -> u8 {
    match source {
        Phase7CandidateSource::Phase5Suggested => 0,
        Phase7CandidateSource::Builtin => 1,
        Phase7CandidateSource::Model => 2,
        Phase7CandidateSource::Exploration => 3,
        Phase7CandidateSource::Repair => 4,
    }
}

fn phase7_builtin_kind_rank(kind: Option<Phase7BuiltinKind>) -> u8 {
    match kind {
        Some(Phase7BuiltinKind::Intro) => 0,
        Some(Phase7BuiltinKind::LocalExact) => 1,
        Some(Phase7BuiltinKind::InductionNat) => 2,
        Some(Phase7BuiltinKind::SimpLiteEmpty) => 3,
        None => 255,
    }
}

fn phase7_candidate_envelope_order(
    left: &Phase7CandidateEnvelope,
    right: &Phase7CandidateEnvelope,
) -> Ordering {
    left.metadata
        .rank
        .source_rank
        .cmp(&right.metadata.rank.source_rank)
        .then_with(|| {
            left.metadata
                .rank
                .builtin_kind_rank
                .cmp(&right.metadata.rank.builtin_kind_rank)
        })
        .then_with(|| {
            left.metadata
                .rank
                .source_index
                .cmp(&right.metadata.rank.source_index)
        })
        .then_with(|| {
            left.phase7_candidate_payload_hash
                .cmp(&right.phase7_candidate_payload_hash)
        })
}

fn phase7_builtin_intro_candidate(goal: &MachineGoalView) -> Option<MachineTacticCandidate> {
    let term = parse_machine_term(FileId(0), &goal.target.machine).ok()?;
    let MachineTerm::Pi { binders, .. } = term else {
        return None;
    };
    let outer_binder_name = binders.first().map(|binder| binder.name.as_str());
    Some(MachineTacticCandidate::Intro {
        name: phase7_fresh_intro_name(goal, outer_binder_name)?,
    })
}

fn phase7_local_exact_prefilter(goal: &MachineGoalView, local: &MachineLocalView) -> bool {
    local.value.is_none()
        && local.ty.core_hash == goal.target.core_hash
        && phase7_machine_name_is_unique(goal, &local.machine_name)
}

fn phase7_induction_nat_prefilter(
    goal: &MachineGoalView,
    context_index: usize,
    local: &MachineLocalView,
) -> bool {
    phase7_goal_allows_tactic(goal, MachineApiTacticKind::InductionNat)
        && phase7_machine_name_is_unique(goal, &local.machine_name)
        && local.value.is_none()
        && context_index + 1 == goal.context.len()
        && goal.target.free_locals.contains(&local.local_id)
}

fn phase7_goal_allows_tactic(goal: &MachineGoalView, tactic: MachineApiTacticKind) -> bool {
    goal.allowed_tactics.contains(&tactic)
}

fn phase7_machine_name_is_unique(goal: &MachineGoalView, machine_name: &str) -> bool {
    goal.context
        .iter()
        .filter(|local| local.machine_name == machine_name)
        .count()
        == 1
}

fn phase7_raw_term_forbidden_token(
    raw_term_index: u32,
    term: &RawMachineTerm,
) -> Result<Option<Phase7ForbiddenToken>, Phase7CandidateFilterError> {
    let tokens = lex_machine_surface_tokens(&term.source).map_err(|error| {
        Phase7CandidateFilterError::RawMachineTermLex {
            raw_term_index,
            source: term.source.clone(),
            message: error.message,
        }
    })?;
    let semantic_tokens = tokens
        .iter()
        .filter(|token| {
            !matches!(
                token.kind,
                MachineSurfaceTokenKind::Whitespace | MachineSurfaceTokenKind::Comment
            )
        })
        .collect::<Vec<_>>();

    let mut best_token = None;
    for (index, token) in semantic_tokens.iter().enumerate() {
        if matches!(token.kind, MachineSurfaceTokenKind::ExternalCommand) {
            let candidate = Phase7ForbiddenToken {
                class: Phase7ForbiddenTokenClass::ExternalCommand,
                spelling: token.spelling.clone(),
                raw_term_index,
            };
            if phase7_forbidden_token_is_better(best_token.as_ref(), &candidate) {
                best_token = Some(candidate);
            }
        }
        if token.spelling == "set_option"
            && semantic_tokens
                .get(index + 1)
                .is_some_and(|next| next.spelling == "unsafe")
        {
            let candidate = Phase7ForbiddenToken {
                class: Phase7ForbiddenTokenClass::SetOptionUnsafe,
                spelling: "set_option unsafe".to_owned(),
                raw_term_index,
            };
            if phase7_forbidden_token_is_better(best_token.as_ref(), &candidate) {
                best_token = Some(candidate);
            }
            continue;
        }
        if token.spelling == "unsafe"
            && index > 0
            && semantic_tokens[index - 1].spelling == "set_option"
        {
            continue;
        }
        if let Some(class) = phase7_forbidden_token_class_for_spelling(&token.spelling) {
            let candidate = Phase7ForbiddenToken {
                class,
                spelling: token.spelling.clone(),
                raw_term_index,
            };
            if phase7_forbidden_token_is_better(best_token.as_ref(), &candidate) {
                best_token = Some(candidate);
            }
        }
    }
    Ok(best_token)
}

fn phase7_forbidden_token_is_better(
    current: Option<&Phase7ForbiddenToken>,
    candidate: &Phase7ForbiddenToken,
) -> bool {
    current.is_none_or(|current| {
        phase7_forbidden_token_class_rank(candidate.class)
            < phase7_forbidden_token_class_rank(current.class)
    })
}

fn phase7_forbidden_token_class_rank(class: Phase7ForbiddenTokenClass) -> u8 {
    match class {
        Phase7ForbiddenTokenClass::Sorry => 0,
        Phase7ForbiddenTokenClass::Admit => 1,
        Phase7ForbiddenTokenClass::Axiom => 2,
        Phase7ForbiddenTokenClass::Unsafe => 3,
        Phase7ForbiddenTokenClass::Import => 4,
        Phase7ForbiddenTokenClass::SetOptionUnsafe => 5,
        Phase7ForbiddenTokenClass::Declare => 6,
        Phase7ForbiddenTokenClass::Eval => 7,
        Phase7ForbiddenTokenClass::Shell => 8,
        Phase7ForbiddenTokenClass::ExternalCommand => 9,
        Phase7ForbiddenTokenClass::DisallowedTacticKind => 10,
    }
}

fn phase7_forbidden_token_class_for_spelling(spelling: &str) -> Option<Phase7ForbiddenTokenClass> {
    match spelling {
        "sorry" => Some(Phase7ForbiddenTokenClass::Sorry),
        "admit" => Some(Phase7ForbiddenTokenClass::Admit),
        "axiom" => Some(Phase7ForbiddenTokenClass::Axiom),
        "unsafe" => Some(Phase7ForbiddenTokenClass::Unsafe),
        "import" => Some(Phase7ForbiddenTokenClass::Import),
        "declare" => Some(Phase7ForbiddenTokenClass::Declare),
        "eval" => Some(Phase7ForbiddenTokenClass::Eval),
        "shell" => Some(Phase7ForbiddenTokenClass::Shell),
        _ => None,
    }
}

fn phase7_candidate_raw_terms(candidate: &MachineTacticCandidate) -> Vec<&RawMachineTerm> {
    let mut terms = Vec::new();
    match candidate {
        MachineTacticCandidate::Exact { term } => terms.push(term),
        MachineTacticCandidate::Apply { args, .. } => {
            for arg in args {
                if let CandidateApplyArg::Term(term) = arg {
                    terms.push(term);
                }
            }
        }
        MachineTacticCandidate::Rewrite { rule, .. } => {
            for arg in &rule.args {
                if let CandidateApplyArg::Term(term) = arg {
                    terms.push(term);
                }
            }
        }
        MachineTacticCandidate::Intro { .. }
        | MachineTacticCandidate::SimpLite { .. }
        | MachineTacticCandidate::InductionNat { .. } => {}
    }
    terms
}

fn phase7_raw_machine_term_json(term: &RawMachineTerm) -> String {
    format!(r#"{{"source":{}}}"#, json_string(&term.source))
}

fn phase7_tactic_head_json(head: &TacticHead) -> String {
    match head {
        TacticHead::Imported {
            name,
            decl_interface_hash,
        } => format!(
            r#"{{"imported":{{"decl_interface_hash":{},"name":{}}}}}"#,
            json_string(&format_hash_string(decl_interface_hash)),
            json_string(&name.as_dotted()),
        ),
        TacticHead::CurrentModule {
            name,
            decl_interface_hash,
        } => format!(
            r#"{{"current_module":{{"decl_interface_hash":{},"name":{}}}}}"#,
            json_string(&format_hash_string(decl_interface_hash)),
            json_string(&name.as_dotted()),
        ),
        TacticHead::Local { name } => {
            format!(r#"{{"local":{{"name":{}}}}}"#, json_string(name))
        }
    }
}

fn phase7_rewrite_rule_json(rule: &CandidateRewriteRuleRef) -> String {
    format!(
        r#"{{"args":{},"head":{},"universe_args":{}}}"#,
        phase7_apply_arg_array_json(&rule.args),
        phase7_tactic_head_json(&rule.head),
        phase7_level_array_json(&rule.universe_args),
    )
}

fn phase7_apply_arg_array_json(args: &[CandidateApplyArg]) -> String {
    let members = args.iter().map(phase7_apply_arg_json).collect::<Vec<_>>();
    format!("[{}]", members.join(","))
}

fn phase7_apply_arg_json(arg: &CandidateApplyArg) -> String {
    match arg {
        CandidateApplyArg::Term(term) => format!(
            r#"{{"mode":"term","term":{}}}"#,
            phase7_raw_machine_term_json(term)
        ),
        CandidateApplyArg::Subgoal { name_hint } => {
            let name_hint = name_hint
                .as_ref()
                .map(|name| json_string(name))
                .unwrap_or_else(|| "null".to_owned());
            format!(r#"{{"mode":"subgoal","name_hint":{name_hint}}}"#)
        }
        CandidateApplyArg::InferFromTarget => r#"{"mode":"infer_from_target"}"#.to_owned(),
    }
}

fn phase7_simp_rule_array_json(rules: &[SimpRuleRef]) -> String {
    let members = rules.iter().map(phase7_simp_rule_json).collect::<Vec<_>>();
    format!("[{}]", members.join(","))
}

fn phase7_simp_rule_json(rule: &SimpRuleRef) -> String {
    format!(
        r#"{{"decl_interface_hash":{},"direction":{},"name":{}}}"#,
        json_string(&format_hash_string(&rule.decl_interface_hash)),
        json_string(phase7_rewrite_direction_wire(rule.direction)),
        json_string(&rule.name.as_dotted()),
    )
}

fn phase7_level_array_json(levels: &[Level]) -> String {
    let members = levels
        .iter()
        .map(|level| json_string(&phase7_render_level_wire(level)))
        .collect::<Vec<_>>();
    format!("[{}]", members.join(","))
}

fn phase7_render_level_wire(level: &Level) -> String {
    if let Some(value) = phase7_level_as_nat(level) {
        return value.to_string();
    }
    match level {
        Level::Zero => "0".to_owned(),
        Level::Succ(inner) => format!("succ {}", phase7_render_level_wire(inner)),
        Level::Max(lhs, rhs) => {
            format!(
                "max {} {}",
                phase7_render_level_wire(lhs),
                phase7_render_level_wire(rhs)
            )
        }
        Level::IMax(lhs, rhs) => {
            format!(
                "imax {} {}",
                phase7_render_level_wire(lhs),
                phase7_render_level_wire(rhs)
            )
        }
        Level::Param(name) => name.clone(),
    }
}

fn phase7_level_as_nat(level: &Level) -> Option<u64> {
    match level {
        Level::Zero => Some(0),
        Level::Succ(inner) => Some(phase7_level_as_nat(inner)? + 1),
        _ => None,
    }
}

fn phase7_rewrite_direction_wire(direction: RewriteDirection) -> &'static str {
    match direction {
        RewriteDirection::Forward => "forward",
        RewriteDirection::Backward => "backward",
    }
}

fn phase7_rewrite_site_wire(site: RewriteSite) -> &'static str {
    match site {
        RewriteSite::EqTargetLeft => "eq_target_left",
        RewriteSite::EqTargetRight => "eq_target_right",
    }
}

fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            ch if ch <= '\u{1f}' => {
                out.push_str("\\u");
                out.push_str(&format!("{:04x}", ch as u32));
            }
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn sha256(bytes: &[u8]) -> Hash {
    Sha256::digest(bytes).into()
}

fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).expect("machine API vector length fits in u32")
}

pub fn parse_phase7_mvp_controller_config(
    source: &str,
) -> Result<Phase7MvpControllerConfig, MachineApiRequestError> {
    let doc = parse_request_body(source, MachineApiErrorKind::InvalidBatchPolicy)?;
    let members = validate_phase7_config_top_level(doc.root())?;

    let search_budget = parse_phase7_search_budget(
        required_config_field(members, "search_budget"),
        &JsonPath::root().field("search_budget"),
    )?;
    let per_tactic_deterministic_budget = parse_deterministic_budget_with_error_kind(
        required_config_field(members, "per_tactic_deterministic_budget"),
        &JsonPath::root().field("per_tactic_deterministic_budget"),
        MachineApiErrorKind::InvalidBudget,
    )?;
    let scheduler_limits = member_value(members, "scheduler_limits")
        .map(|value| {
            parse_phase7_batch_scheduler_limits(value, &JsonPath::root().field("scheduler_limits"))
        })
        .transpose()?;
    let batch_policy = parse_phase7_batch_policy(
        required_config_field(members, "batch_policy"),
        &JsonPath::root().field("batch_policy"),
    )?;

    validate_phase7_mvp_controller_config(Phase7MvpControllerConfig {
        search_budget,
        per_tactic_deterministic_budget,
        scheduler_limits,
        batch_policy,
    })
}

pub fn validate_phase7_mvp_controller_config(
    config: Phase7MvpControllerConfig,
) -> Result<Phase7MvpControllerConfig, MachineApiRequestError> {
    validate_positive_u64(
        config.search_budget.wall_clock_ms,
        "wall_clock_ms",
        &JsonPath::root()
            .field("search_budget")
            .field("wall_clock_ms"),
        MachineApiErrorKind::InvalidBatchPolicy,
    )?;
    validate_positive_u64(
        config.search_budget.max_nodes,
        "max_nodes",
        &JsonPath::root().field("search_budget").field("max_nodes"),
        MachineApiErrorKind::InvalidBatchPolicy,
    )?;
    if config.search_budget.max_tactics_per_node != PHASE7_MVP_MAX_TACTICS_PER_NODE {
        return Err(invalid_u64(
            "max_tactics_per_node",
            u64::from(config.search_budget.max_tactics_per_node),
            &JsonPath::root()
                .field("search_budget")
                .field("max_tactics_per_node"),
            MachineApiErrorKind::InvalidBatchPolicy,
        ));
    }

    validate_batch_policy_value(
        config.batch_policy.max_evaluated_candidates,
        "max_evaluated_candidates",
    )?;
    validate_batch_policy_value(
        config.batch_policy.stop_after_successes,
        "stop_after_successes",
    )?;
    validate_batch_policy_value(
        config.batch_policy.stop_after_failures,
        "stop_after_failures",
    )?;

    if let Some(scheduler_limits) = config.scheduler_limits {
        validate_optional_scheduler_limit(
            scheduler_limits.per_candidate_timeout_ms,
            "per_candidate_timeout_ms",
        )?;
        validate_optional_scheduler_limit(scheduler_limits.batch_timeout_ms, "batch_timeout_ms")?;
        validate_optional_scheduler_limit(scheduler_limits.max_memory_mb, "max_memory_mb")?;
    }

    Ok(config)
}

fn parse_phase7_search_budget(
    value: &JsonValue<'_>,
    path: &JsonPath,
) -> Result<Phase7SearchBudget, MachineApiRequestError> {
    let object = validate_json_object(
        value,
        ObjectSchema::new(
            MachineApiErrorKind::InvalidBatchPolicy,
            PHASE7_SEARCH_BUDGET_FIELDS,
        ),
        path,
    )?;
    Ok(Phase7SearchBudget {
        wall_clock_ms: required_u64(&object, "wall_clock_ms"),
        max_nodes: required_u64(&object, "max_nodes"),
        max_tactics_per_node: required_u32(
            &object,
            "max_tactics_per_node",
            &path.field("max_tactics_per_node"),
        )?,
        max_depth: required_u32(&object, "max_depth", &path.field("max_depth"))?,
    })
}

fn parse_phase7_batch_policy(
    value: &JsonValue<'_>,
    path: &JsonPath,
) -> Result<MachineTacticBatchPolicy, MachineApiRequestError> {
    let object = validate_json_object(
        value,
        ObjectSchema::new(
            MachineApiErrorKind::InvalidBatchPolicy,
            PHASE7_BATCH_POLICY_FIELDS,
        ),
        path,
    )?;
    Ok(MachineTacticBatchPolicy {
        max_evaluated_candidates: required_batch_policy_u32(
            &object,
            "max_evaluated_candidates",
            &path.field("max_evaluated_candidates"),
        )?,
        stop_after_successes: required_batch_policy_u32(
            &object,
            "stop_after_successes",
            &path.field("stop_after_successes"),
        )?,
        stop_after_failures: required_batch_policy_u32(
            &object,
            "stop_after_failures",
            &path.field("stop_after_failures"),
        )?,
    })
}

fn parse_phase7_batch_scheduler_limits(
    value: &JsonValue<'_>,
    path: &JsonPath,
) -> Result<MachineBatchSchedulerLimits, MachineApiRequestError> {
    let object = validate_json_object(
        value,
        ObjectSchema::new(
            MachineApiErrorKind::InvalidSchedulerLimits,
            PHASE7_BATCH_SCHEDULER_FIELDS,
        ),
        path,
    )?;
    Ok(MachineBatchSchedulerLimits {
        per_candidate_timeout_ms: optional_positive_u64(
            &object,
            "per_candidate_timeout_ms",
            &path.field("per_candidate_timeout_ms"),
        )?,
        batch_timeout_ms: optional_positive_u64(
            &object,
            "batch_timeout_ms",
            &path.field("batch_timeout_ms"),
        )?,
        max_memory_mb: optional_positive_u64(
            &object,
            "max_memory_mb",
            &path.field("max_memory_mb"),
        )?,
    })
}

fn validate_phase7_config_top_level<'value, 'src>(
    root: &'value JsonValue<'src>,
) -> Result<&'value [JsonMember<'src>], MachineApiRequestError> {
    let members = root.object_members().ok_or_else(|| {
        MachineApiRequestError::new(
            MachineApiErrorKind::InvalidBatchPolicy,
            JsonPath::root(),
            MachineApiRequestErrorReason::ExpectedObject {
                actual: root.kind(),
            },
        )
    })?;

    let mut seen = std::collections::BTreeSet::new();
    for member in members {
        if !seen.insert(member.key().to_owned()) {
            return Err(MachineApiRequestError::new(
                MachineApiErrorKind::InvalidBatchPolicy,
                JsonPath::root().field(member.key()),
                MachineApiRequestErrorReason::DuplicateKey {
                    key: member.key().to_owned(),
                },
            ));
        }
    }

    for member in members {
        if member.key() == "scheduler_limits" {
            validate_top_level_field(
                member.value(),
                "scheduler_limits",
                JsonFieldType::Object,
                MachineApiErrorKind::InvalidSchedulerLimits,
            )?;
        } else if let Some(field) = PHASE7_CONFIG_FIELDS
            .iter()
            .find(|field| field.name == member.key())
        {
            validate_top_level_field(
                member.value(),
                field.name,
                field.field_type,
                field_error_kind(field.name),
            )?;
        } else {
            return Err(MachineApiRequestError::new(
                MachineApiErrorKind::InvalidBatchPolicy,
                JsonPath::root().field(member.key()),
                MachineApiRequestErrorReason::UnknownField {
                    field: member.key().to_owned(),
                },
            ));
        }
    }

    for field in PHASE7_CONFIG_FIELDS {
        if !members.iter().any(|member| member.key() == field.name) {
            return Err(MachineApiRequestError::new(
                field_error_kind(field.name),
                JsonPath::root().field(field.name),
                MachineApiRequestErrorReason::MissingField { field: field.name },
            ));
        }
    }

    Ok(members)
}

fn validate_top_level_field(
    value: &JsonValue<'_>,
    field: &'static str,
    expected: JsonFieldType,
    error_kind: MachineApiErrorKind,
) -> Result<(), MachineApiRequestError> {
    if value.kind() == JsonValueKind::Null {
        return Err(MachineApiRequestError::new(
            error_kind,
            JsonPath::root().field(field),
            MachineApiRequestErrorReason::NullField { field },
        ));
    }

    let valid = matches!(
        (expected, value.kind()),
        (JsonFieldType::Object, JsonValueKind::Object)
            | (JsonFieldType::Array, JsonValueKind::Array)
            | (JsonFieldType::String, JsonValueKind::String)
            | (JsonFieldType::Boolean, JsonValueKind::Bool)
    );
    if valid {
        return Ok(());
    }

    Err(MachineApiRequestError::new(
        error_kind,
        JsonPath::root().field(field),
        MachineApiRequestErrorReason::TypeMismatch {
            field,
            expected,
            actual: value.kind(),
        },
    ))
}

fn field_error_kind(field: &str) -> MachineApiErrorKind {
    match field {
        "per_tactic_deterministic_budget" => MachineApiErrorKind::InvalidBudget,
        _ => MachineApiErrorKind::InvalidBatchPolicy,
    }
}

fn required_config_field<'value, 'src>(
    members: &'value [JsonMember<'src>],
    field: &str,
) -> &'value JsonValue<'src> {
    member_value(members, field).expect("schema checked required field")
}

fn member_value<'value, 'src>(
    members: &'value [JsonMember<'src>],
    field: &str,
) -> Option<&'value JsonValue<'src>> {
    members
        .iter()
        .find(|member| member.key() == field)
        .map(JsonMember::value)
}

fn required_u64(object: &ValidatedObject<'_, '_>, field: &str) -> u64 {
    object
        .field(field)
        .and_then(JsonValue::number_raw)
        .and_then(|raw| parse_strict_u64_token(raw, u64::MAX).ok())
        .expect("schema checked required u64 field")
}

fn required_u32(
    object: &ValidatedObject<'_, '_>,
    field: &'static str,
    path: &JsonPath,
) -> Result<u32, MachineApiRequestError> {
    let raw = object
        .field(field)
        .and_then(JsonValue::number_raw)
        .expect("schema checked required u32 field");
    let parsed = parse_strict_u64_token(raw, u64::from(u32::MAX)).map_err(|error| {
        MachineApiRequestError::new(
            MachineApiErrorKind::InvalidBatchPolicy,
            path.clone(),
            MachineApiRequestErrorReason::InvalidUnsignedInteger {
                field,
                raw: raw.to_owned(),
                error,
            },
        )
    })?;
    Ok(parsed as u32)
}

fn required_batch_policy_u32(
    object: &ValidatedObject<'_, '_>,
    field: &'static str,
    path: &JsonPath,
) -> Result<u32, MachineApiRequestError> {
    let raw = object
        .field(field)
        .and_then(JsonValue::number_raw)
        .expect("schema checked required batch policy u64 field");
    let parsed = parse_strict_u64_token(raw, 256).map_err(|error| {
        MachineApiRequestError::new(
            MachineApiErrorKind::InvalidBatchPolicy,
            path.clone(),
            MachineApiRequestErrorReason::InvalidUnsignedInteger {
                field,
                raw: raw.to_owned(),
                error,
            },
        )
    })?;
    if parsed == 0 {
        return Err(MachineApiRequestError::new(
            MachineApiErrorKind::InvalidBatchPolicy,
            path.clone(),
            MachineApiRequestErrorReason::InvalidUnsignedInteger {
                field,
                raw: raw.to_owned(),
                error: StrictUnsignedIntegerError::InvalidGrammar,
            },
        ));
    }
    Ok(parsed as u32)
}

fn optional_positive_u64(
    object: &ValidatedObject<'_, '_>,
    field: &'static str,
    path: &JsonPath,
) -> Result<Option<u64>, MachineApiRequestError> {
    let Some(value) = object.field(field) else {
        return Ok(None);
    };
    let raw = value
        .number_raw()
        .expect("schema checked optional scheduler u64 field");
    let parsed =
        parse_strict_u64_token(raw, u64::MAX).expect("schema checked optional scheduler u64 field");
    if parsed == 0 {
        return Err(invalid_u64(
            field,
            parsed,
            path,
            MachineApiErrorKind::InvalidSchedulerLimits,
        ));
    }
    Ok(Some(parsed))
}

fn validate_positive_u64(
    value: u64,
    field: &'static str,
    path: &JsonPath,
    kind: MachineApiErrorKind,
) -> Result<(), MachineApiRequestError> {
    if value == 0 {
        return Err(invalid_u64(field, value, path, kind));
    }
    Ok(())
}

fn validate_batch_policy_value(
    value: u32,
    field: &'static str,
) -> Result<(), MachineApiRequestError> {
    if value == 0 || value > 256 {
        return Err(invalid_u64(
            field,
            u64::from(value),
            &JsonPath::root().field("batch_policy").field(field),
            MachineApiErrorKind::InvalidBatchPolicy,
        ));
    }
    Ok(())
}

fn validate_optional_scheduler_limit(
    value: Option<u64>,
    field: &'static str,
) -> Result<(), MachineApiRequestError> {
    if value == Some(0) {
        return Err(invalid_u64(
            field,
            0,
            &JsonPath::root().field("scheduler_limits").field(field),
            MachineApiErrorKind::InvalidSchedulerLimits,
        ));
    }
    Ok(())
}

fn invalid_u64(
    field: &'static str,
    value: u64,
    path: &JsonPath,
    kind: MachineApiErrorKind,
) -> MachineApiRequestError {
    MachineApiRequestError::new(
        kind,
        path.clone(),
        MachineApiRequestErrorReason::InvalidUnsignedInteger {
            field,
            raw: value.to_string(),
            error: StrictUnsignedIntegerError::InvalidGrammar,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use npa_tactic::{
        CandidateApplyArg, CandidateRewriteRuleRef, MachineTacticCandidate, MetaVarId,
        RawMachineTerm, RewriteDirection, RewriteSite, TacticHead,
    };

    use crate::{
        parse_machine_snapshot_get_request, parse_machine_tactic_batch_request,
        parse_machine_theorem_search_request, JsonFieldType, LocalId, MachineAllowedModulesFilter,
        MachineApiErrorResponse, MachineApiOkResponse, MachineApiRequestErrorReason,
        MachineApiResponseEnvelope, MachineApiResponseStatus, MachineApiSchedulerResponse,
        MachineCertificateWirePayload, MachineExprView, MachineLocalView, MachineSchedulerArtifact,
        MachineSchedulerArtifactKind, MachineSchedulerArtifactScope, MachineSuggestedCandidate,
        MachineSuggestedCandidateStatus, MachineTheoremGlobalRef, MachineTheoremStatement,
        MachineVerifiedModuleCertificatePayload,
    };

    fn hash(byte: u8) -> Hash {
        [byte; 32]
    }

    fn unwrap_search_failure(result: Phase7SearchResult) -> Phase7SearchFailure {
        match result {
            Ok(proof) => panic!("expected search failure, got verified proof {proof:?}"),
            Err(failure) => *failure,
        }
    }

    fn unwrap_verified_proof(result: Phase7SearchResult) -> Phase7VerifiedProof {
        match result {
            Ok(proof) => proof,
            Err(failure) => panic!("expected verified proof, got search failure {failure:?}"),
        }
    }

    fn name(value: &str) -> Name {
        Name::from_dotted(value)
    }

    fn simp_rule(name_suffix: &str, byte: u8) -> SimpRuleRef {
        SimpRuleRef {
            name: name(&format!("Nat.{name_suffix}")),
            decl_interface_hash: hash(byte),
            direction: RewriteDirection::Forward,
        }
    }

    fn snapshot_request() -> Phase7SnapshotGetRequest {
        Phase7SnapshotGetRequest {
            session_id: SessionId::parse("msess_001").unwrap(),
            snapshot_id: SnapshotId::from_state_fingerprint(hash(1)),
            state_fingerprint: hash(1),
        }
    }

    fn imported_ref(name_suffix: &str, byte: u8) -> MachineGlobalRefView {
        MachineGlobalRefView::Imported {
            module: name("Std.Nat.Basic"),
            name: name(&format!("Nat.{name_suffix}")),
            export_hash: hash(byte),
            decl_interface_hash: hash(byte + 1),
            public_export: true,
            tactic_head_visible: true,
        }
    }

    fn expr_view(
        byte: u8,
        size: u32,
        free_local_count: u32,
        head: Option<MachineGlobalRefView>,
    ) -> MachineExprView {
        MachineExprView {
            core_hash: hash(byte),
            head: head.clone(),
            constants: head.into_iter().collect(),
            free_locals: (0..free_local_count).map(LocalId).collect(),
            size,
            machine: format!("expr_{byte}"),
            pretty: Some(format!("pretty_{byte}")),
        }
    }

    fn local_view(index: u32) -> MachineLocalView {
        MachineLocalView {
            local_id: LocalId(index),
            machine_name: format!("x{index}"),
            display_name: format!("x{index}"),
            ty: expr_view(70 + index as u8, 1, 0, None),
            value: None,
            depends_on: Vec::new(),
            binder_index: index,
        }
    }

    fn goal_view(
        goal_id: GoalId,
        byte: u8,
        expr_size: u32,
        free_local_count: u32,
        context_size: u32,
        head: Option<MachineGlobalRefView>,
    ) -> MachineGoalView {
        MachineGoalView {
            goal_id,
            meta_id: MetaVarId(goal_id.0),
            context_hash: hash(byte + 10),
            local_name_map_hash: hash(byte + 11),
            context: (0..context_size).map(local_view).collect(),
            target: expr_view(byte, expr_size, free_local_count, head),
            target_hash: hash(byte + 12),
            goal_fingerprint: hash(byte + 13),
            allowed_tactics: Vec::new(),
        }
    }

    fn snapshot_with_goals(goals: Vec<MachineGoalView>) -> MachineProofSnapshot {
        snapshot_with_state(1, goals)
    }

    fn snapshot_with_state(byte: u8, goals: Vec<MachineGoalView>) -> MachineProofSnapshot {
        MachineProofSnapshot {
            snapshot_id: SnapshotId::from_state_fingerprint(hash(byte)),
            session_id: SessionId::parse("msess_001").unwrap(),
            state_fingerprint: hash(byte),
            tactic_options_fingerprint: hash(byte + 1),
            open_goals: goals.iter().map(|goal| goal.goal_id).collect(),
            goals,
            proof_skeleton_hash: hash(byte + 2),
        }
    }

    fn theorem_result(
        machine: &str,
        suggested: Vec<MachineSuggestedCandidate>,
    ) -> MachineTheoremSearchResult {
        MachineTheoremSearchResult {
            premise_id: "prem_0".to_owned(),
            global_ref: MachineTheoremGlobalRef {
                module: name("Std.Nat.Basic"),
                name: name("Nat.add_zero"),
                export_hash: hash(10),
                decl_interface_hash: hash(11),
            },
            universe_params: vec!["u".to_owned()],
            statement: MachineTheoremStatement {
                core_hash: hash(12),
                head: Some(imported_ref("Eq", 13)),
                machine: machine.to_owned(),
            },
            modes: vec![MachineTheoremMode::Exact, MachineTheoremMode::Simp],
            suggested_candidates: suggested,
            score: 0,
            axioms_used: vec![MachineAxiomRefWire::Imported {
                module: name("Std.Nat.Basic"),
                name: name("Nat.zero_ax"),
                export_hash: hash(14),
                decl_interface_hash: hash(15),
            }],
        }
    }

    fn search_ok_fields(result: MachineTheoremSearchResult) -> MachineTheoremSearchOkFields {
        MachineTheoremSearchOkFields {
            query_fingerprint: hash(20),
            theorem_index_fingerprint: hash(21),
            search_profile_version: "mvp-zero-score-v1",
            suggestion_profile_version: "mvp-suggested-candidates-v1",
            results: vec![result],
        }
    }

    fn search_ok_response(
        results: Vec<MachineTheoremSearchResult>,
    ) -> MachineTheoremSearchResponse {
        MachineApiResponseEnvelope::Ok(MachineApiOkResponse {
            status: MachineApiResponseStatus::Ok,
            endpoint_fields: MachineTheoremSearchOkFields {
                query_fingerprint: hash(20),
                theorem_index_fingerprint: hash(21),
                search_profile_version: "mvp-zero-score-v1",
                suggestion_profile_version: "mvp-suggested-candidates-v1",
                results,
            },
        })
    }

    fn empty_retrieval() -> Phase7PremiseRetrieval {
        Phase7PremiseRetrieval {
            cache_key: Phase7RetrievalCacheKey {
                session_root_hash: hash(90),
                query_fingerprint: hash(91),
                theorem_index_fingerprint: hash(92),
            },
            cache_entries: Vec::new(),
            results: Vec::new(),
        }
    }

    fn valid_config_json() -> &'static str {
        r#"{
          "search_budget": {
            "wall_clock_ms": 30000,
            "max_nodes": 10000,
            "max_tactics_per_node": 16,
            "max_depth": 64
          },
          "per_tactic_deterministic_budget": {
            "max_tactic_steps": 64,
            "max_whnf_steps": 10000,
            "max_conversion_steps": 10000,
            "max_rewrite_steps": 100,
            "max_meta_allocations": 8,
            "max_expr_nodes": 20000
          },
          "batch_policy": {
            "max_evaluated_candidates": 16,
            "stop_after_successes": 8,
            "stop_after_failures": 16
          }
        }"#
    }

    fn batch_request_with_candidate(candidate_json: &str) -> String {
        let request = snapshot_request();
        format!(
            r#"{{
              "session_id":"{}",
              "snapshot_id":"{}",
              "state_fingerprint":"{}",
              "goal_id":"g0",
              "candidates":[{{"candidate_id":"c0","candidate":{candidate_json}}}],
              "deterministic_budget":{{
                "max_tactic_steps":64,
                "max_whnf_steps":10000,
                "max_conversion_steps":10000,
                "max_rewrite_steps":100,
                "max_meta_allocations":8,
                "max_expr_nodes":20000
              }},
              "batch_policy":{{
                "max_evaluated_candidates":16,
                "stop_after_successes":8,
                "stop_after_failures":16
              }}
            }}"#,
            request.session_id.wire(),
            request.snapshot_id.wire(),
            format_hash_string(&request.state_fingerprint),
        )
    }

    fn mvp_config() -> Phase7MvpControllerConfig {
        parse_phase7_mvp_controller_config(valid_config_json()).unwrap()
    }

    fn phase7_test_envelope(
        source_index: u32,
        candidate_hash: Option<Hash>,
    ) -> Phase7CandidateEnvelope {
        let candidate = MachineTacticCandidate::SimpLite { rules: Vec::new() };
        let metadata = phase7_candidate_metadata(
            Phase7CandidateSource::Builtin,
            Some(Phase7BuiltinKind::SimpLiteEmpty),
            source_index,
            Vec::new(),
            Vec::new(),
            &candidate,
        );
        phase7_candidate_envelope(candidate, candidate_hash, metadata)
    }

    fn phase7_exact_test_envelope(
        source_index: u32,
        candidate_hash: Option<Hash>,
        term: &str,
    ) -> Phase7CandidateEnvelope {
        let candidate = MachineTacticCandidate::Exact {
            term: RawMachineTerm::new(term),
        };
        let metadata = phase7_candidate_metadata(
            Phase7CandidateSource::Builtin,
            Some(Phase7BuiltinKind::LocalExact),
            source_index,
            Vec::new(),
            Vec::new(),
            &candidate,
        );
        phase7_candidate_envelope(candidate, candidate_hash, metadata)
    }

    fn phase7_test_batch_request(
        candidates: Vec<Phase7CandidateEnvelope>,
    ) -> Phase7TacticBatchRequest {
        let snapshot = snapshot_request();
        let config = mvp_config();
        Phase7TacticBatchRequest {
            session_id: snapshot.session_id,
            snapshot_id: snapshot.snapshot_id,
            state_fingerprint: snapshot.state_fingerprint,
            goal_id: GoalId(0),
            candidates: phase7_assign_candidate_ids(candidates),
            deterministic_budget: config.per_tactic_deterministic_budget,
            batch_policy: config.batch_policy,
            scheduler_limits: None,
        }
    }

    fn phase7_test_search_input(initial_snapshot: MachineProofSnapshot) -> Phase7SearchInput {
        let config = mvp_config();
        Phase7SearchInput {
            session_id: initial_snapshot.session_id.clone(),
            session_root_hash: hash(90),
            initial_snapshot,
            search_budget: config.search_budget,
            per_tactic_deterministic_budget: config.per_tactic_deterministic_budget,
            scheduler_limits: config.scheduler_limits,
            batch_policy: config.batch_policy,
        }
    }

    fn suggested_candidate(
        candidate_hash: Hash,
        candidate: MachineTacticCandidate,
    ) -> MachineSuggestedCandidate {
        MachineSuggestedCandidate {
            status: MachineSuggestedCandidateStatus::Validated,
            candidate_hash,
            candidate,
        }
    }

    fn accepted_failure(
        error_kind: FailedCandidateErrorKind,
        candidate_hash: Hash,
    ) -> Phase7AcceptedCandidateFailure {
        Phase7AcceptedCandidateFailure {
            error_kind,
            phase: crate::MachineApiDiagnosticPhase::TacticExecution,
            goal_id: Some(GoalId(0)),
            tactic_kind: Some(MachineApiTacticKind::Exact),
            candidate_hash,
            deterministic_budget_hash: tactic_budget_hash(
                mvp_config().per_tactic_deterministic_budget,
            ),
            diagnostic_hash: hash(55),
            retryable: false,
        }
    }

    fn compact_error(kind: MachineApiErrorKind) -> MachineApiCompactErrorWire {
        MachineApiCompactErrorWire {
            error_kind: kind,
            phase: crate::MachineApiDiagnosticPhase::TacticExecution,
            diagnostic_hash: hash(55),
            retryable: false,
            goal_id: Some(GoalId(0)),
            tactic_kind: Some(MachineApiTacticKind::SimpLite),
            primary_name: None,
            primary_axiom_ref: None,
            expected_hash: None,
            actual_hash: None,
        }
    }

    fn ok_batch_response(
        request: &Phase7TacticBatchRequest,
        results: Vec<MachineTacticBatchItemResponse>,
    ) -> MachineTacticBatchResponse {
        ok_batch_response_for(
            request.state_fingerprint,
            request.deterministic_budget,
            results,
        )
    }

    fn ok_batch_response_for(
        previous_state_fingerprint: Hash,
        deterministic_budget: TacticBudget,
        results: Vec<MachineTacticBatchItemResponse>,
    ) -> MachineTacticBatchResponse {
        let success_count = usize_to_u32(results.iter().filter(|item| item.is_success()).count());
        let failure_count = usize_to_u32(results.len()) - success_count;
        MachineApiResponseEnvelope::Ok(MachineApiOkResponse {
            status: MachineApiResponseStatus::Ok,
            endpoint_fields: MachineTacticBatchOkFields {
                previous_state_fingerprint,
                deterministic_budget_hash: tactic_budget_hash(deterministic_budget),
                results,
                success_count,
                failure_count,
            },
        })
    }

    fn replay_ok_response(
        final_snapshot_id: SnapshotId,
        final_state_fingerprint: Hash,
    ) -> MachineReplayResponse {
        MachineApiResponseEnvelope::Ok(MachineApiOkResponse {
            status: MachineApiResponseStatus::Ok,
            endpoint_fields: MachineReplayOkFields {
                final_snapshot_id,
                final_state_fingerprint,
            },
        })
    }

    fn replay_scheduler_stopped_response() -> MachineReplayResponse {
        MachineApiResponseEnvelope::SchedulerStopped(MachineApiSchedulerResponse {
            status: MachineApiResponseStatus::SchedulerStopped,
            scheduler_artifact: MachineSchedulerArtifact {
                kind: MachineSchedulerArtifactKind::Timeout,
                scope: MachineSchedulerArtifactScope::Replay,
                retryable: true,
            },
            endpoint_fields: (),
        })
    }

    fn verify_ok_response() -> MachineVerifyResponse {
        MachineApiResponseEnvelope::Ok(MachineApiOkResponse {
            status: MachineApiResponseStatus::Verified,
            endpoint_fields: verify_ok_fields(),
        })
    }

    fn verify_ok_fields() -> MachineVerifyOkFields {
        MachineVerifyOkFields {
            root_decl_interface_hash: hash(80),
            root_decl_certificate_hash: hash(81),
            root_axioms_used: Vec::new(),
            module_export_hash: hash(82),
            module_certificate_hash: hash(83),
            module_axioms_used: Vec::new(),
            certificate: certificate_payload(84),
            dependency_import_closure: Vec::new(),
            import_payload: verified_module_certificate_payload(85),
        }
    }

    fn verify_ok_envelope() -> MachineApiOkResponse<MachineVerifyOkFields> {
        MachineApiOkResponse {
            status: MachineApiResponseStatus::Verified,
            endpoint_fields: verify_ok_fields(),
        }
    }

    fn replay_error_response(
        kind: MachineApiErrorKind,
        phase: crate::MachineApiDiagnosticPhase,
    ) -> MachineReplayResponse {
        MachineApiResponseEnvelope::Error(Box::new(MachineApiErrorResponse {
            status: MachineApiResponseStatus::Error,
            error: error_wire(kind, phase),
            endpoint_fields: (),
        }))
    }

    fn verify_error_response(
        kind: MachineApiErrorKind,
        phase: crate::MachineApiDiagnosticPhase,
    ) -> MachineVerifyResponse {
        MachineApiResponseEnvelope::Error(Box::new(MachineApiErrorResponse {
            status: MachineApiResponseStatus::Error,
            error: error_wire(kind, phase),
            endpoint_fields: (),
        }))
    }

    fn error_wire(
        kind: MachineApiErrorKind,
        phase: crate::MachineApiDiagnosticPhase,
    ) -> MachineApiErrorWire {
        MachineApiErrorWire {
            kind,
            phase,
            diagnostic_hash: hash(79),
            retryable: false,
            goal_id: None,
            tactic_kind: None,
            primary_name: None,
            primary_axiom_ref: None,
            expected_hash: None,
            actual_hash: None,
        }
    }

    fn certificate_payload(byte: u8) -> MachineCertificateWirePayload {
        MachineCertificateWirePayload {
            encoding: "npa.certificate.canonical.v0.1.hex",
            bytes: format!("{byte:02x}"),
        }
    }

    fn verified_module_certificate_payload(byte: u8) -> MachineVerifiedModuleCertificatePayload {
        MachineVerifiedModuleCertificatePayload {
            module: name("Test.Module"),
            expected_export_hash: hash(byte),
            expected_certificate_hash: hash(byte + 1),
            certificate: certificate_payload(byte + 2),
        }
    }

    #[test]
    fn phase7_snapshot_get_request_forces_include_pretty_false() {
        let source = phase7_snapshot_get_request_json(&snapshot_request());

        let parsed = parse_machine_snapshot_get_request(&source).unwrap();

        assert!(!parsed.include_pretty);
        assert_eq!(parsed.session_id, SessionId::parse("msess_001").unwrap());
        assert_eq!(parsed.state_fingerprint, hash(1));
    }

    #[test]
    fn fake_client_records_snapshot_get_without_pretty() {
        let request = snapshot_request();
        let mut client = Phase7FakeMachineApiClient::new();

        let err = client.get_snapshot(request.clone()).unwrap_err();

        assert_eq!(
            err,
            Phase7MachineApiError::FakeResponseExhausted {
                endpoint: Phase7MachineApiEndpointKind::SnapshotGet
            }
        );
        assert_eq!(
            client.calls(),
            &[Phase7MachineApiCall::SnapshotGet {
                session_id: request.session_id,
                snapshot_id: request.snapshot_id,
                state_fingerprint: request.state_fingerprint,
                include_pretty: false,
            }]
        );
    }

    #[test]
    fn initial_snapshot_loader_uses_snapshot_boundary_and_derives_goal_summaries() {
        let request = snapshot_request();
        let snapshot = snapshot_with_goals(vec![
            goal_view(GoalId(1), 30, 8, 0, 0, Some(imported_ref("Eq", 40))),
            goal_view(GoalId(0), 31, 3, 1, 2, None),
        ]);
        let mut client = Phase7FakeMachineApiClient::new();
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk {
            snapshot: snapshot.clone(),
        }));

        let loaded = load_phase7_initial_snapshot(&mut client, request.clone()).unwrap();

        assert_eq!(loaded.snapshot, snapshot);
        assert_eq!(loaded.goals.len(), 2);
        assert_eq!(loaded.goals[0].goal_id, GoalId(1));
        assert_eq!(loaded.goals[0].open_goal_index, 0);
        assert_eq!(loaded.goals[0].expr_size, 8);
        assert_eq!(loaded.goals[0].target_free_local_count, 0);
        assert_eq!(
            client.calls(),
            &[Phase7MachineApiCall::SnapshotGet {
                session_id: request.session_id,
                snapshot_id: request.snapshot_id,
                state_fingerprint: request.state_fingerprint,
                include_pretty: false,
            }]
        );
    }

    #[test]
    fn goal_selection_uses_derived_snapshot_fields_only() {
        let snapshot = snapshot_with_goals(vec![
            goal_view(GoalId(2), 30, 10, 0, 1, Some(imported_ref("Eq", 40))),
            goal_view(GoalId(1), 31, 5, 2, 0, Some(imported_ref("And", 42))),
            goal_view(GoalId(0), 32, 5, 1, 0, None),
        ]);

        let summaries = phase7_goal_summaries(&snapshot);
        let selected = select_phase7_goal(&snapshot).unwrap();

        assert_eq!(summaries[0].goal_id, GoalId(2));
        assert_eq!(summaries[1].open_goal_index, 1);
        assert_eq!(summaries[2].target_hash, hash(44));
        assert_eq!(selected.goal_id, GoalId(0));
        assert_eq!(selected.expr_size, 5);
        assert_eq!(selected.target_free_local_count, 1);
    }

    #[test]
    fn phase7_mvp_premise_query_is_fixed_phase5_search_shape() {
        let source = phase7_mvp_premise_query_json(&Phase7PremiseQueryRequest {
            session_id: SessionId::parse("msess_001").unwrap(),
            snapshot_id: SnapshotId::from_state_fingerprint(hash(1)),
            state_fingerprint: hash(1),
            goal_id: GoalId(7),
        });

        let parsed = parse_machine_theorem_search_request(&source).unwrap();

        assert_eq!(
            parsed.modes,
            vec![
                MachineTheoremMode::Exact,
                MachineTheoremMode::Apply,
                MachineTheoremMode::Rw,
                MachineTheoremMode::Simp,
            ]
        );
        assert_eq!(parsed.limit, 32);
        assert!(parsed.filters.exclude_axioms);
        assert_eq!(
            parsed.filters.allowed_modules,
            MachineAllowedModulesFilter::AllDirect
        );
        assert!(!source.contains("allowed_modules"));
    }

    #[test]
    fn retrieve_phase7_premises_uses_fixed_query_and_preserves_phase5_results() {
        let request = Phase7PremiseQueryRequest {
            session_id: SessionId::parse("msess_001").unwrap(),
            snapshot_id: SnapshotId::from_state_fingerprint(hash(1)),
            state_fingerprint: hash(1),
            goal_id: GoalId(7),
        };
        let search = search_ok_fields(theorem_result("display", Vec::new()));
        let mut client = Phase7FakeMachineApiClient::new();
        client.push_search_for_goal_response(Ok(MachineApiResponseEnvelope::Ok(
            MachineApiOkResponse {
                status: MachineApiResponseStatus::Ok,
                endpoint_fields: search.clone(),
            },
        )));

        let retrieval = retrieve_phase7_premises(&mut client, &request, hash(99)).unwrap();

        assert_eq!(
            retrieval.cache_key,
            Phase7RetrievalCacheKey {
                session_root_hash: hash(99),
                query_fingerprint: hash(20),
                theorem_index_fingerprint: hash(21),
            }
        );
        assert_eq!(retrieval.cache_entries.len(), 1);
        assert_eq!(retrieval.results, search.results);
        assert_eq!(client.calls().len(), 1);
        let Phase7MachineApiCall::SearchForGoal { source } = &client.calls()[0] else {
            panic!("expected search_for_goal call");
        };
        let parsed = parse_machine_theorem_search_request(source).unwrap();
        assert_eq!(parsed.goal_id, GoalId(7));
        assert_eq!(
            parsed.modes,
            vec![
                MachineTheoremMode::Exact,
                MachineTheoremMode::Apply,
                MachineTheoremMode::Rw,
                MachineTheoremMode::Simp,
            ]
        );
        assert_eq!(
            parsed.filters.allowed_modules,
            MachineAllowedModulesFilter::AllDirect
        );
    }

    #[test]
    fn premise_cache_entries_use_verified_metadata_not_display_or_suggestions() {
        let suggested = MachineSuggestedCandidate {
            status: MachineSuggestedCandidateStatus::Validated,
            candidate_hash: hash(16),
            candidate: MachineTacticCandidate::SimpLite { rules: Vec::new() },
        };
        let mut search = search_ok_fields(theorem_result("pretty theorem text", vec![suggested]));

        let entries = phase7_premise_cache_entries(&search);
        let usages = phase7_premise_usages(&search);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].premise_ref.module, name("Std.Nat.Basic"));
        assert_eq!(entries[0].premise_ref.name, name("Nat.add_zero"));
        assert_eq!(entries[0].universe_params, vec!["u".to_owned()]);
        assert_eq!(entries[0].statement_core_hash, hash(12));
        assert_eq!(entries[0].statement_head, Some(imported_ref("Eq", 13)));
        assert_eq!(
            entries[0].modes,
            vec![MachineTheoremMode::Exact, MachineTheoremMode::Simp]
        );
        assert_eq!(entries[0].response_index, 0);
        assert_eq!(
            usages[0],
            Phase7PremiseUsage {
                premise_ref: entries[0].premise_ref.clone(),
                universe_params: entries[0].universe_params.clone(),
                statement_core_hash: entries[0].statement_core_hash,
                axioms_used: entries[0].axioms_used.clone(),
            }
        );

        let original_entries = entries;
        search.results[0].statement.machine = "different display".to_owned();
        search.results[0].score = 99;
        search.results[0].suggested_candidates.clear();

        assert_eq!(phase7_premise_cache_entries(&search), original_entries);
    }

    #[test]
    fn retrieval_cache_key_uses_phase5_fingerprints() {
        let search = search_ok_fields(theorem_result("display", Vec::new()));

        let key = phase7_retrieval_cache_key(hash(99), &search);

        assert_eq!(
            key,
            Phase7RetrievalCacheKey {
                session_root_hash: hash(99),
                query_fingerprint: hash(20),
                theorem_index_fingerprint: hash(21),
            }
        );
    }

    #[test]
    fn candidate_payload_json_is_phase5_candidate_shape_and_hash_is_payload_only() {
        let candidate = MachineTacticCandidate::SimpLite { rules: Vec::new() };
        let payload = phase7_candidate_payload_json(&candidate);

        assert_eq!(payload, r#"{"kind":"simp-lite","rules":[]}"#);
        parse_machine_tactic_batch_request(&batch_request_with_candidate(&payload)).unwrap();

        let mut metadata = phase7_candidate_metadata(
            Phase7CandidateSource::Builtin,
            Some(Phase7BuiltinKind::SimpLiteEmpty),
            0,
            Vec::new(),
            Vec::new(),
            &candidate,
        );
        metadata.score = 999;
        metadata.display_text = Some("unsafe display is not payload".to_owned());
        let first = phase7_candidate_envelope(candidate.clone(), None, metadata);
        let second = phase7_candidate_envelope(
            candidate,
            Some(hash(77)),
            phase7_candidate_metadata(
                Phase7CandidateSource::Phase5Suggested,
                None,
                7,
                Vec::new(),
                Vec::new(),
                &MachineTacticCandidate::SimpLite { rules: Vec::new() },
            ),
        );

        assert_eq!(
            first.phase7_candidate_payload_hash,
            second.phase7_candidate_payload_hash
        );
        assert!(!payload.contains("candidate_hash"));
        assert!(!payload.contains("display"));
        assert!(!payload.contains("premises"));
    }

    #[test]
    fn candidate_payload_json_uses_canonical_object_key_order() {
        let apply = MachineTacticCandidate::Apply {
            head: TacticHead::Local {
                name: "f".to_owned(),
            },
            universe_args: Vec::new(),
            args: vec![
                CandidateApplyArg::Term(RawMachineTerm::new("h\n")),
                CandidateApplyArg::Subgoal { name_hint: None },
                CandidateApplyArg::InferFromTarget,
            ],
        };
        let apply_payload = phase7_candidate_payload_json(&apply);

        assert_eq!(
            apply_payload,
            r#"{"args":[{"mode":"term","term":{"source":"h\u000a"}},{"mode":"subgoal","name_hint":null},{"mode":"infer_from_target"}],"head":{"local":{"name":"f"}},"kind":"apply","universe_args":[]}"#
        );
        parse_machine_tactic_batch_request(&batch_request_with_candidate(&apply_payload)).unwrap();

        let rw = MachineTacticCandidate::Rewrite {
            rule: CandidateRewriteRuleRef {
                head: TacticHead::Imported {
                    name: name("Nat.add_zero"),
                    decl_interface_hash: hash(50),
                },
                universe_args: Vec::new(),
                args: vec![CandidateApplyArg::InferFromTarget],
            },
            direction: RewriteDirection::Forward,
            site: RewriteSite::EqTargetLeft,
        };
        let rw_payload = phase7_candidate_payload_json(&rw);

        assert_eq!(
            rw_payload,
            format!(
                r#"{{"direction":"forward","kind":"rw","rule":{{"args":[{{"mode":"infer_from_target"}}],"head":{{"imported":{{"decl_interface_hash":{},"name":"Nat.add_zero"}}}},"universe_args":[]}},"site":"eq_target_left"}}"#,
                json_string(&format_hash_string(&hash(50)))
            )
        );
        parse_machine_tactic_batch_request(&batch_request_with_candidate(&rw_payload)).unwrap();
    }

    #[test]
    fn candidate_metadata_matches_phase7_score_and_repair_shape() {
        let candidate = MachineTacticCandidate::SimpLite { rules: Vec::new() };
        let mut metadata = phase7_candidate_metadata(
            Phase7CandidateSource::Repair,
            None,
            0,
            Vec::new(),
            Vec::new(),
            &candidate,
        );
        metadata.score = -1;
        metadata.repair = Some(Phase7CandidateRepairMetadata {
            parent_candidate_hash: hash(60),
            error_kind: FailedCandidateErrorKind::TypeMismatch,
            repair_depth: 1,
            chain_tried_payload_hashes: vec![hash(61)],
        });

        assert_eq!(metadata.score, -1);
        assert_eq!(
            metadata.repair,
            Some(Phase7CandidateRepairMetadata {
                parent_candidate_hash: hash(60),
                error_kind: FailedCandidateErrorKind::TypeMismatch,
                repair_depth: 1,
                chain_tried_payload_hashes: vec![hash(61)],
            })
        );
    }

    #[test]
    fn suggested_candidate_envelopes_flatten_phase5_results_and_preserve_hashes() {
        let suggested = MachineSuggestedCandidate {
            status: MachineSuggestedCandidateStatus::Validated,
            candidate_hash: hash(40),
            candidate: MachineTacticCandidate::SimpLite { rules: Vec::new() },
        };
        let mut result = theorem_result("display", vec![suggested]);
        result.score = 999;

        let envelopes = phase7_suggested_candidate_envelopes(&[result.clone()]);

        assert_eq!(envelopes.len(), 1);
        assert_eq!(envelopes[0].candidate_hash, Some(hash(40)));
        assert_eq!(
            envelopes[0].metadata.source,
            Phase7CandidateSource::Phase5Suggested
        );
        assert_eq!(
            envelopes[0].metadata.rank,
            Phase7CandidateRankMetadata {
                source_rank: 0,
                source_index: 0,
                builtin_kind_rank: 255
            }
        );
        assert_eq!(envelopes[0].metadata.score, 0);
        assert_eq!(envelopes[0].metadata.premises_used.len(), 1);
        assert_eq!(
            envelopes[0].metadata.premises_used[0].premise_ref,
            phase7_premise_ref(&result)
        );
        assert_eq!(
            envelopes[0].metadata.trust_flags.uses_axioms,
            result.axioms_used
        );
    }

    #[test]
    fn builtin_generator_emits_mvp_candidates_without_phase5_hashes() {
        let mut goal = goal_view(GoalId(0), 30, 5, 1, 1, None);
        goal.target.machine = "forall (p : Prop), Prop".to_owned();
        goal.context[0].machine_name = "n".to_owned();
        goal.context[0].display_name = "n".to_owned();
        goal.allowed_tactics = vec![
            MachineApiTacticKind::InductionNat,
            MachineApiTacticKind::SimpLite,
        ];

        let envelopes = phase7_builtin_candidate_envelopes(&goal);

        assert_eq!(envelopes.len(), 3);
        assert!(envelopes
            .iter()
            .all(|envelope| envelope.candidate_hash.is_none()));
        assert!(matches!(
            envelopes[0].candidate,
            MachineTacticCandidate::Intro { ref name } if name == "p"
        ));
        assert!(matches!(
            envelopes[1].candidate,
            MachineTacticCandidate::InductionNat { ref local_name } if local_name == "n"
        ));
        assert!(matches!(
            envelopes[2].candidate,
            MachineTacticCandidate::SimpLite { ref rules } if rules.is_empty()
        ));
        assert_eq!(envelopes[0].metadata.rank.builtin_kind_rank, 0);
        assert_eq!(envelopes[1].metadata.rank.builtin_kind_rank, 2);
        assert_eq!(envelopes[2].metadata.rank.builtin_kind_rank, 3);
    }

    #[test]
    fn fresh_intro_name_skips_unbounded_suffix_scan_for_max_length_base() {
        let max_length_name = "a".repeat(64);
        assert!(is_machine_local_name(&max_length_name));
        assert!(!is_machine_local_name(&format!("{max_length_name}1")));

        let mut goal = goal_view(GoalId(0), 30, 5, 0, 1, None);
        goal.context[0].machine_name = max_length_name.clone();

        assert_eq!(
            phase7_fresh_intro_name(&goal, Some(&max_length_name)),
            Some("x".to_owned())
        );
    }

    #[test]
    fn builtin_local_exact_requires_unique_assumption_with_matching_target_hash() {
        let mut goal = goal_view(GoalId(0), 30, 5, 0, 0, None);
        let mut local = local_view(0);
        local.machine_name = "h".to_owned();
        local.display_name = "h".to_owned();
        local.ty = goal.target.clone();
        goal.context = vec![local.clone()];

        let envelopes = phase7_builtin_candidate_envelopes(&goal);

        assert_eq!(envelopes.len(), 1);
        assert!(matches!(
            envelopes[0].candidate,
            MachineTacticCandidate::Exact {
                term: RawMachineTerm { ref source }
            } if source == "h"
        ));

        let mut duplicate_goal = goal;
        duplicate_goal.context.push(local);
        assert!(phase7_builtin_candidate_envelopes(&duplicate_goal).is_empty());
    }

    #[test]
    fn induction_nat_prefilter_requires_last_context_assumption_used_by_target() {
        let mut goal = goal_view(GoalId(0), 30, 5, 1, 2, None);
        goal.context[0].machine_name = "n".to_owned();
        goal.context[1].machine_name = "m".to_owned();
        goal.allowed_tactics = vec![MachineApiTacticKind::InductionNat];

        let envelopes = phase7_builtin_candidate_envelopes(&goal);

        assert!(envelopes.is_empty());

        goal.context.swap(0, 1);
        goal.context[1].local_id = LocalId(0);
        goal.context[1].machine_name = "n".to_owned();
        let envelopes = phase7_builtin_candidate_envelopes(&goal);

        assert_eq!(envelopes.len(), 1);
        assert!(matches!(
            envelopes[0].candidate,
            MachineTacticCandidate::InductionNat { ref local_name } if local_name == "n"
        ));
    }

    #[test]
    fn forbidden_token_filter_scans_only_raw_machine_term_tokens() {
        let unsafe_candidate = MachineTacticCandidate::Exact {
            term: RawMachineTerm::new("Std.unsafe.Type"),
        };
        let token = phase7_candidate_forbidden_token(&unsafe_candidate)
            .unwrap()
            .unwrap();
        assert_eq!(token.class, Phase7ForbiddenTokenClass::Unsafe);
        assert_eq!(token.spelling, "unsafe");

        let set_option_candidate = MachineTacticCandidate::Exact {
            term: RawMachineTerm::new("set_option -- comment\n unsafe"),
        };
        let token = phase7_candidate_forbidden_token(&set_option_candidate)
            .unwrap()
            .unwrap();
        assert_eq!(token.class, Phase7ForbiddenTokenClass::SetOptionUnsafe);

        let priority_candidate = MachineTacticCandidate::Exact {
            term: RawMachineTerm::new("import unsafe"),
        };
        let token = phase7_candidate_forbidden_token(&priority_candidate)
            .unwrap()
            .unwrap();
        assert_eq!(token.class, Phase7ForbiddenTokenClass::Unsafe);
        assert_eq!(token.spelling, "unsafe");

        let safe_candidate = MachineTacticCandidate::Exact {
            term: RawMachineTerm::new("hunsafe"),
        };
        assert_eq!(
            phase7_candidate_forbidden_token(&safe_candidate).unwrap(),
            None
        );

        let candidate = MachineTacticCandidate::SimpLite { rules: Vec::new() };
        let mut metadata = phase7_candidate_metadata(
            Phase7CandidateSource::Builtin,
            Some(Phase7BuiltinKind::SimpLiteEmpty),
            0,
            Vec::new(),
            Vec::new(),
            &candidate,
        );
        metadata.display_text = Some("unsafe".to_owned());
        let result = filter_phase7_candidate_envelopes(vec![phase7_candidate_envelope(
            candidate, None, metadata,
        )]);
        assert_eq!(result.accepted.len(), 1);
        assert!(result.rejected.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn mvp_candidate_generation_preserves_forbidden_rejections_before_dedupe() {
        let mut goal = goal_view(GoalId(0), 30, 5, 0, 0, None);
        let mut local = local_view(0);
        local.machine_name = "unsafe".to_owned();
        local.display_name = "unsafe".to_owned();
        local.ty = goal.target.clone();
        goal.context = vec![local];

        let builtin = phase7_builtin_candidate_envelopes(&goal);
        assert_eq!(builtin.len(), 1);
        assert!(matches!(
            builtin[0].candidate,
            MachineTacticCandidate::Exact {
                term: RawMachineTerm { ref source }
            } if source == "unsafe"
        ));

        let result = phase7_mvp_candidate_generation(&goal, &empty_retrieval());

        assert!(result.accepted.is_empty());
        assert!(result.errors.is_empty());
        assert_eq!(result.rejected.len(), 1);
        assert_eq!(
            result.rejected[0].forbidden_token.class,
            Phase7ForbiddenTokenClass::Unsafe
        );
        assert!(
            result.rejected[0]
                .envelope
                .metadata
                .trust_flags
                .contains_forbidden_tokens
        );
    }

    #[test]
    fn candidate_ordering_and_dedupe_use_rank_not_score_or_display_text() {
        let candidate0 = MachineTacticCandidate::Exact {
            term: RawMachineTerm::new("h0"),
        };
        let candidate1 = MachineTacticCandidate::Exact {
            term: RawMachineTerm::new("h1"),
        };
        let duplicate = MachineTacticCandidate::SimpLite { rules: Vec::new() };

        let mut later_metadata = phase7_candidate_metadata(
            Phase7CandidateSource::Builtin,
            Some(Phase7BuiltinKind::LocalExact),
            1,
            Vec::new(),
            Vec::new(),
            &candidate1,
        );
        later_metadata.score = 1000;
        later_metadata.display_text = Some("aaa".to_owned());
        let earlier_metadata = phase7_candidate_metadata(
            Phase7CandidateSource::Builtin,
            Some(Phase7BuiltinKind::LocalExact),
            0,
            Vec::new(),
            Vec::new(),
            &candidate0,
        );
        let builtin_duplicate_metadata = phase7_candidate_metadata(
            Phase7CandidateSource::Builtin,
            Some(Phase7BuiltinKind::SimpLiteEmpty),
            0,
            Vec::new(),
            Vec::new(),
            &duplicate,
        );
        let suggested_duplicate_metadata = phase7_candidate_metadata(
            Phase7CandidateSource::Phase5Suggested,
            None,
            9,
            Vec::new(),
            Vec::new(),
            &duplicate,
        );

        let ordered = phase7_rank_and_dedupe_candidate_envelopes(vec![
            phase7_candidate_envelope(duplicate.clone(), None, builtin_duplicate_metadata),
            phase7_candidate_envelope(candidate1, None, later_metadata),
            phase7_candidate_envelope(candidate0, None, earlier_metadata),
            phase7_candidate_envelope(duplicate, Some(hash(88)), suggested_duplicate_metadata),
        ]);

        assert_eq!(ordered.len(), 3);
        assert!(matches!(
            ordered[0].candidate,
            MachineTacticCandidate::SimpLite { .. }
        ));
        assert_eq!(ordered[0].candidate_hash, Some(hash(88)));
        assert!(matches!(
            ordered[1].candidate,
            MachineTacticCandidate::Exact {
                term: RawMachineTerm { ref source }
            } if source == "h0"
        ));
        assert!(matches!(
            ordered[2].candidate,
            MachineTacticCandidate::Exact {
                term: RawMachineTerm { ref source }
            } if source == "h1"
        ));
    }

    #[test]
    fn batch_request_builder_assigns_ids_caps_policy_and_uses_batch_endpoint() {
        let mut request = phase7_test_batch_request(vec![
            phase7_test_envelope(0, None),
            phase7_test_envelope(1, None),
        ]);
        request.scheduler_limits = Some(MachineBatchSchedulerLimits {
            per_candidate_timeout_ms: Some(100),
            batch_timeout_ms: Some(1000),
            max_memory_mb: None,
        });

        let source = phase7_tactic_batch_request_json(&request);
        let parsed = parse_machine_tactic_batch_request(&source).unwrap();

        assert_eq!(parsed.candidates[0].candidate_id, "c0");
        assert_eq!(parsed.candidates[1].candidate_id, "c1");
        assert_eq!(parsed.batch_policy.max_evaluated_candidates, 2);
        assert_eq!(parsed.batch_policy.stop_after_successes, 2);
        assert_eq!(parsed.batch_policy.stop_after_failures, 2);
        assert_eq!(parsed.scheduler_limits.per_candidate_timeout_ms, Some(100));
        assert_eq!(parsed.scheduler_limits.batch_timeout_ms, Some(1000));

        let mut client = Phase7FakeMachineApiClient::new();
        client.push_tactic_batch_response(Ok(ok_batch_response(&request, Vec::new())));
        let evaluation = phase7_run_tactic_batch(&mut client, &request).unwrap();

        assert_eq!(evaluation.replay_steps.len(), 0);
        assert_eq!(evaluation.deferred_candidates.len(), 2);
        assert_eq!(client.calls().len(), 1);
        assert!(matches!(
            &client.calls()[0],
            Phase7MachineApiCall::TacticBatch { source: actual } if actual == &source
        ));
    }

    #[test]
    fn batch_success_items_build_replay_steps_and_errors_normalize_failures() {
        let request = phase7_test_batch_request(vec![
            phase7_test_envelope(0, None),
            phase7_test_envelope(1, Some(hash(50))),
        ]);
        let response = ok_batch_response(
            &request,
            vec![
                MachineTacticBatchItemResponse::Success {
                    candidate_id: "c0".to_owned(),
                    candidate_hash: hash(40),
                    next_snapshot_id: SnapshotId::from_state_fingerprint(hash(41)),
                    next_state_fingerprint: hash(42),
                    proof_delta_hash: hash(43),
                },
                MachineTacticBatchItemResponse::Error {
                    candidate_id: "c1".to_owned(),
                    candidate_hash: Some(hash(50)),
                    diagnostic: compact_error(MachineApiErrorKind::TypeMismatch),
                },
            ],
        );

        let evaluation = phase7_evaluate_tactic_batch_response(&request, response).unwrap();

        assert_eq!(evaluation.successful_transitions.len(), 1);
        assert_eq!(evaluation.replay_steps.len(), 1);
        assert_eq!(evaluation.accepted_failure_records.len(), 1);
        assert_eq!(evaluation.accepted_failures.len(), 1);
        assert!(evaluation.non_accepted_errors.is_empty());
        assert_eq!(evaluation.deferred_candidates.len(), 0);

        let transition = &evaluation.successful_transitions[0];
        assert_eq!(transition.candidate_id, "c0");
        assert_eq!(
            transition.next_snapshot_id,
            SnapshotId::from_state_fingerprint(hash(41))
        );

        let step = &evaluation.replay_steps[0];
        assert_eq!(&transition.replay_step, step);
        assert_eq!(step.previous_state_fingerprint, request.state_fingerprint);
        assert_eq!(step.goal_id, GoalId(0));
        assert_eq!(step.candidate_hash, hash(40));
        assert_eq!(step.proof_delta_hash, hash(43));
        assert_eq!(step.next_state_fingerprint, hash(42));

        let failure = &evaluation.accepted_failures[0];
        assert_eq!(evaluation.accepted_failure_records[0].candidate_id, "c1");
        assert_eq!(&evaluation.accepted_failure_records[0].failure, failure);
        assert_eq!(failure.error_kind, FailedCandidateErrorKind::TypeMismatch);
        assert_eq!(failure.candidate_hash, hash(50));
        assert_eq!(
            failure.deterministic_budget_hash,
            tactic_budget_hash(request.deterministic_budget)
        );
        assert_eq!(failure.diagnostic_hash, hash(55));

        let step_json = phase7_replay_step_json(step);
        assert!(!step_json.contains("candidate_id"));
        assert!(!step_json.contains("display"));
        let replay_source = format!(
            r#"{{"session_id":"{}","plan":{{"protocol_version":{},"session_root_hash":{},"initial_state_fingerprint":{},"steps":[{}],"final_state_fingerprint":{}}}}}"#,
            request.session_id.wire(),
            json_string(crate::MachineApiVersion::V1.as_str()),
            json_string(&format_hash_string(&hash(90))),
            json_string(&format_hash_string(&request.state_fingerprint)),
            step_json,
            json_string(&format_hash_string(&hash(42))),
        );
        parse_machine_replay_request(&replay_source).unwrap();
    }

    #[test]
    fn batch_candidate_hash_mismatch_is_controller_error_before_replay() {
        let request = phase7_test_batch_request(vec![phase7_test_envelope(0, Some(hash(77)))]);
        let response = ok_batch_response(
            &request,
            vec![MachineTacticBatchItemResponse::Success {
                candidate_id: "c0".to_owned(),
                candidate_hash: hash(78),
                next_snapshot_id: SnapshotId::from_state_fingerprint(hash(41)),
                next_state_fingerprint: hash(42),
                proof_delta_hash: hash(43),
            }],
        );

        let error = phase7_evaluate_tactic_batch_response(&request, response).unwrap_err();

        assert_eq!(
            error.kind,
            Phase7MachineControllerErrorKind::SuggestedCandidateHashMismatch
        );
        assert_eq!(error.candidate_id.as_deref(), Some("c0"));
        assert_eq!(error.expected_hash, Some(hash(77)));
        assert_eq!(error.actual_hash, Some(hash(78)));
    }

    #[test]
    fn zero_progress_scheduler_stop_is_not_a_candidate_failure_or_deferred_suffix() {
        let request = phase7_test_batch_request(vec![
            phase7_test_envelope(0, None),
            phase7_test_envelope(1, None),
        ]);
        let response = MachineApiResponseEnvelope::SchedulerStopped(MachineApiSchedulerResponse {
            status: MachineApiResponseStatus::PartialTimeout,
            scheduler_artifact: MachineSchedulerArtifact {
                kind: MachineSchedulerArtifactKind::Timeout,
                scope: MachineSchedulerArtifactScope::Batch,
                retryable: true,
            },
            endpoint_fields: MachineTacticBatchSchedulerFields {
                previous_state_fingerprint: request.state_fingerprint,
                deterministic_budget_hash: tactic_budget_hash(request.deterministic_budget),
                completed_prefix_len: 0,
                results: Vec::new(),
                success_count: 0,
                failure_count: 0,
            },
        });

        let evaluation = phase7_evaluate_tactic_batch_response(&request, response).unwrap();

        assert_eq!(evaluation.evaluated_count, 0);
        assert!(evaluation.replay_steps.is_empty());
        assert!(evaluation.accepted_failures.is_empty());
        assert!(evaluation.non_accepted_errors.is_empty());
        assert_eq!(
            evaluation.scheduler_stop,
            Some(Phase7SchedulerStop {
                status: MachineApiResponseStatus::PartialTimeout,
                completed_prefix_len: 0,
            })
        );
        assert!(evaluation.deferred_candidates.is_empty());
    }

    #[test]
    fn scheduler_stop_after_prefix_defers_only_suffix_after_stopped_candidate() {
        let request = phase7_test_batch_request(vec![
            phase7_test_envelope(0, None),
            phase7_test_envelope(1, None),
            phase7_test_envelope(2, None),
            phase7_test_envelope(3, None),
        ]);
        let response = MachineApiResponseEnvelope::SchedulerStopped(MachineApiSchedulerResponse {
            status: MachineApiResponseStatus::PartialResourceLimit,
            scheduler_artifact: MachineSchedulerArtifact {
                kind: MachineSchedulerArtifactKind::ResourceLimitExceeded,
                scope: MachineSchedulerArtifactScope::Batch,
                retryable: true,
            },
            endpoint_fields: MachineTacticBatchSchedulerFields {
                previous_state_fingerprint: request.state_fingerprint,
                deterministic_budget_hash: tactic_budget_hash(request.deterministic_budget),
                completed_prefix_len: 1,
                results: vec![MachineTacticBatchItemResponse::Success {
                    candidate_id: "c0".to_owned(),
                    candidate_hash: hash(40),
                    next_snapshot_id: SnapshotId::from_state_fingerprint(hash(41)),
                    next_state_fingerprint: hash(42),
                    proof_delta_hash: hash(43),
                }],
                success_count: 1,
                failure_count: 0,
            },
        });

        let evaluation = phase7_evaluate_tactic_batch_response(&request, response).unwrap();

        assert_eq!(evaluation.replay_steps.len(), 1);
        assert!(evaluation.accepted_failures.is_empty());
        assert!(evaluation.non_accepted_errors.is_empty());
        assert_eq!(
            evaluation.scheduler_stop,
            Some(Phase7SchedulerStop {
                status: MachineApiResponseStatus::PartialResourceLimit,
                completed_prefix_len: 1,
            })
        );
        assert_eq!(evaluation.deferred_candidates.len(), 2);
        assert_eq!(evaluation.deferred_candidates[0].candidate_id, "c2");
        assert_eq!(evaluation.deferred_candidates[1].candidate_id, "c3");
    }

    #[test]
    fn batch_contract_violations_end_as_controller_errors() {
        let request = phase7_test_batch_request(vec![
            phase7_test_envelope(0, None),
            phase7_test_envelope(1, None),
        ]);
        let bad_prefix = ok_batch_response(
            &request,
            vec![MachineTacticBatchItemResponse::Success {
                candidate_id: "c1".to_owned(),
                candidate_hash: hash(40),
                next_snapshot_id: SnapshotId::from_state_fingerprint(hash(41)),
                next_state_fingerprint: hash(42),
                proof_delta_hash: hash(43),
            }],
        );

        let error = phase7_evaluate_tactic_batch_response(&request, bad_prefix).unwrap_err();
        assert_eq!(
            error.kind,
            Phase7MachineControllerErrorKind::BatchResponseContractViolation
        );

        let bad_budget = MachineApiResponseEnvelope::Ok(MachineApiOkResponse {
            status: MachineApiResponseStatus::Ok,
            endpoint_fields: MachineTacticBatchOkFields {
                previous_state_fingerprint: request.state_fingerprint,
                deterministic_budget_hash: hash(99),
                results: Vec::new(),
                success_count: 0,
                failure_count: 0,
            },
        });
        let error = phase7_evaluate_tactic_batch_response(&request, bad_budget).unwrap_err();
        assert_eq!(
            error.kind,
            Phase7MachineControllerErrorKind::BatchResponseContractViolation
        );
        assert_eq!(
            error.expected_hash,
            Some(tactic_budget_hash(request.deterministic_budget))
        );
        assert_eq!(error.actual_hash, Some(hash(99)));
    }

    #[test]
    fn m5_batch_error_without_candidate_hash_is_not_repair_accepted_failure() {
        let request = phase7_test_batch_request(vec![phase7_test_envelope(0, None)]);
        let response = ok_batch_response(
            &request,
            vec![MachineTacticBatchItemResponse::Error {
                candidate_id: "c0".to_owned(),
                candidate_hash: None,
                diagnostic: compact_error(MachineApiErrorKind::TypeMismatch),
            }],
        );

        let evaluation = phase7_evaluate_tactic_batch_response(&request, response).unwrap();

        assert!(evaluation.accepted_failures.is_empty());
        assert!(evaluation.accepted_failure_records.is_empty());
        assert_eq!(evaluation.non_accepted_errors.len(), 1);
        assert_eq!(evaluation.non_accepted_errors[0].candidate_id, "c0");
        assert_eq!(
            evaluation.non_accepted_errors[0].error_kind,
            MachineApiErrorKind::TypeMismatch
        );
        assert!(!evaluation.non_accepted_errors[0].has_candidate_hash);
    }

    #[test]
    fn m5_rule_based_repair_classifies_all_failed_candidate_kinds() {
        let cases = [
            (
                FailedCandidateErrorKind::UnsupportedTactic,
                Phase7RuleBasedRepairAction::Noop,
            ),
            (
                FailedCandidateErrorKind::MachineTermElaborationError,
                Phase7RuleBasedRepairAction::Noop,
            ),
            (
                FailedCandidateErrorKind::UnknownName,
                Phase7RuleBasedRepairAction::Noop,
            ),
            (
                FailedCandidateErrorKind::ImplicitArgumentRequired,
                Phase7RuleBasedRepairAction::Noop,
            ),
            (
                FailedCandidateErrorKind::TypeMismatch,
                Phase7RuleBasedRepairAction::TrySimpLite,
            ),
            (
                FailedCandidateErrorKind::ExpectedPiType,
                Phase7RuleBasedRepairAction::TrySimpLite,
            ),
            (
                FailedCandidateErrorKind::RewriteRuleInvalid,
                Phase7RuleBasedRepairAction::TrySimpLite,
            ),
            (
                FailedCandidateErrorKind::SimpNoProgress,
                Phase7RuleBasedRepairAction::TrySimpLite,
            ),
            (
                FailedCandidateErrorKind::InductionTargetNotNat,
                Phase7RuleBasedRepairAction::Noop,
            ),
            (
                FailedCandidateErrorKind::BudgetExceeded,
                Phase7RuleBasedRepairAction::Noop,
            ),
            (
                FailedCandidateErrorKind::TooManyGoals,
                Phase7RuleBasedRepairAction::TrySimpLite,
            ),
            (
                FailedCandidateErrorKind::TooLargeTerm,
                Phase7RuleBasedRepairAction::Noop,
            ),
        ];

        assert_eq!(cases.len(), 12);
        for (kind, expected) in cases {
            assert_eq!(phase7_rule_based_repair_action(kind), expected);
        }
    }

    #[test]
    fn m5_rule_based_repair_generates_limited_simp_lite_metadata() {
        let mut goal = goal_view(GoalId(0), 30, 5, 0, 0, None);
        goal.allowed_tactics = vec![MachineApiTacticKind::SimpLite];
        let failed = phase7_exact_test_envelope(0, Some(hash(40)), "h");
        let failure = accepted_failure(FailedCandidateErrorKind::TypeMismatch, hash(40));

        let output = Phase7RuleBasedRepair::new().repair_candidate(&goal, &failed, &failure, 1);

        assert!(output.repeated_candidate_payload_hashes.is_empty());
        assert_eq!(output.pending.len(), 1);
        let pending = &output.pending[0];
        assert_eq!(pending.goal_id, GoalId(0));
        assert_eq!(pending.repair_depth, 1);
        assert_eq!(pending.parent_candidate_hash, hash(40));
        assert_eq!(pending.error_kind, FailedCandidateErrorKind::TypeMismatch);
        assert_eq!(
            pending.chain_tried_payload_hashes,
            vec![failed.phase7_candidate_payload_hash]
        );
        assert_eq!(phase7_repair_depth_of(&pending.candidate), 1);
        assert_eq!(pending.candidate.candidate_hash, None);
        assert!(matches!(
            pending.candidate.candidate,
            MachineTacticCandidate::SimpLite { ref rules } if rules.is_empty()
        ));
        assert_eq!(
            pending.candidate.metadata.source,
            Phase7CandidateSource::Repair
        );
        assert_eq!(pending.candidate.metadata.rank.source_rank, 4);
        assert_eq!(pending.candidate.metadata.rank.source_index, 0);
        assert_eq!(pending.candidate.metadata.rank.builtin_kind_rank, 255);
        assert_eq!(
            pending.candidate.metadata.expected_effect,
            Phase7ExpectedEffect::Simplify
        );
        assert!(pending.candidate.metadata.premises_used.is_empty());
        assert_eq!(
            pending.candidate.metadata.repair,
            Some(Phase7CandidateRepairMetadata {
                parent_candidate_hash: hash(40),
                error_kind: FailedCandidateErrorKind::TypeMismatch,
                repair_depth: 1,
                chain_tried_payload_hashes: vec![failed.phase7_candidate_payload_hash],
            })
        );
    }

    #[test]
    fn m5_rule_based_repair_does_not_generate_without_allowed_simp_lite() {
        let goal = goal_view(GoalId(0), 30, 5, 0, 0, None);
        let failed = phase7_exact_test_envelope(0, Some(hash(40)), "h");
        let failure = accepted_failure(FailedCandidateErrorKind::TypeMismatch, hash(40));

        let output = Phase7RuleBasedRepair::new().repair_candidate(&goal, &failed, &failure, 1);

        assert!(output.pending.is_empty());
        assert!(output.repeated_candidate_payload_hashes.is_empty());
    }

    #[test]
    fn m5_rule_based_repair_refuses_depth_above_two() {
        let mut goal = goal_view(GoalId(0), 30, 5, 0, 0, None);
        goal.allowed_tactics = vec![MachineApiTacticKind::SimpLite];
        let failed = phase7_exact_test_envelope(0, Some(hash(40)), "h");
        let failure = accepted_failure(FailedCandidateErrorKind::TypeMismatch, hash(40));

        let output = Phase7RuleBasedRepair::new().repair_candidate(&goal, &failed, &failure, 3);

        assert!(output.pending.is_empty());
        assert!(output.repeated_candidate_payload_hashes.is_empty());
    }

    #[test]
    fn m5_rule_based_repair_reports_chain_duplicate_payload() {
        let mut goal = goal_view(GoalId(0), 30, 5, 0, 0, None);
        goal.allowed_tactics = vec![MachineApiTacticKind::SimpLite];
        let failed = phase7_test_envelope(0, Some(hash(40)));
        let failure = accepted_failure(FailedCandidateErrorKind::SimpNoProgress, hash(40));

        let output = Phase7RuleBasedRepair::new().repair_candidate(&goal, &failed, &failure, 1);

        assert!(output.pending.is_empty());
        assert_eq!(
            output.repeated_candidate_payload_hashes,
            vec![failed.phase7_candidate_payload_hash]
        );
    }

    #[test]
    fn m5_repair_limiter_preserves_first_three_per_parent_and_dedupes_payload() {
        let pending = vec![
            Phase7PendingCandidate {
                goal_id: GoalId(0),
                candidate: phase7_exact_test_envelope(0, None, "h0"),
                repair_depth: 1,
                parent_candidate_hash: hash(40),
                error_kind: FailedCandidateErrorKind::TypeMismatch,
                chain_tried_payload_hashes: vec![hash(90)],
            },
            Phase7PendingCandidate {
                goal_id: GoalId(0),
                candidate: phase7_exact_test_envelope(0, None, "h0"),
                repair_depth: 1,
                parent_candidate_hash: hash(40),
                error_kind: FailedCandidateErrorKind::TypeMismatch,
                chain_tried_payload_hashes: vec![hash(90)],
            },
            Phase7PendingCandidate {
                goal_id: GoalId(0),
                candidate: phase7_exact_test_envelope(1, None, "h1"),
                repair_depth: 1,
                parent_candidate_hash: hash(40),
                error_kind: FailedCandidateErrorKind::TypeMismatch,
                chain_tried_payload_hashes: vec![hash(90)],
            },
            Phase7PendingCandidate {
                goal_id: GoalId(0),
                candidate: phase7_exact_test_envelope(2, None, "h2"),
                repair_depth: 1,
                parent_candidate_hash: hash(40),
                error_kind: FailedCandidateErrorKind::TypeMismatch,
                chain_tried_payload_hashes: vec![hash(90)],
            },
            Phase7PendingCandidate {
                goal_id: GoalId(0),
                candidate: phase7_exact_test_envelope(3, None, "h3"),
                repair_depth: 1,
                parent_candidate_hash: hash(40),
                error_kind: FailedCandidateErrorKind::TypeMismatch,
                chain_tried_payload_hashes: vec![hash(90)],
            },
        ];

        let limited = phase7_limit_repairs(pending);

        assert_eq!(limited.len(), 3);
        assert!(matches!(
            limited[0].candidate.candidate,
            MachineTacticCandidate::Exact {
                term: RawMachineTerm { ref source }
            } if source == "h0"
        ));
        assert!(matches!(
            limited[1].candidate.candidate,
            MachineTacticCandidate::Exact {
                term: RawMachineTerm { ref source }
            } if source == "h1"
        ));
        assert!(matches!(
            limited[2].candidate.candidate,
            MachineTacticCandidate::Exact {
                term: RawMachineTerm { ref source }
            } if source == "h2"
        ));
    }

    #[test]
    fn m5_search_runs_pending_repair_in_same_node_after_accepted_failure() {
        let mut goal = goal_view(GoalId(0), 30, 5, 0, 0, None);
        goal.allowed_tactics = vec![MachineApiTacticKind::SimpLite];
        let root = snapshot_with_state(1, vec![goal]);
        let closed_child = snapshot_with_state(2, Vec::new());
        let config = mvp_config();
        let mut client = Phase7FakeMachineApiClient::new();
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk {
            snapshot: root.clone(),
        }));
        client.push_search_for_goal_response(Ok(search_ok_response(vec![theorem_result(
            "display",
            vec![suggested_candidate(
                hash(40),
                MachineTacticCandidate::Exact {
                    term: RawMachineTerm::new("h"),
                },
            )],
        )])));
        client.push_tactic_batch_response(Ok(ok_batch_response_for(
            root.state_fingerprint,
            config.per_tactic_deterministic_budget,
            vec![
                MachineTacticBatchItemResponse::Error {
                    candidate_id: "c0".to_owned(),
                    candidate_hash: Some(hash(40)),
                    diagnostic: compact_error(MachineApiErrorKind::TypeMismatch),
                },
                MachineTacticBatchItemResponse::Error {
                    candidate_id: "c1".to_owned(),
                    candidate_hash: Some(hash(41)),
                    diagnostic: compact_error(MachineApiErrorKind::SimpNoProgress),
                },
            ],
        )));
        client.push_tactic_batch_response(Ok(ok_batch_response_for(
            root.state_fingerprint,
            config.per_tactic_deterministic_budget,
            vec![MachineTacticBatchItemResponse::Success {
                candidate_id: "c0".to_owned(),
                candidate_hash: hash(42),
                next_snapshot_id: closed_child.snapshot_id,
                next_state_fingerprint: closed_child.state_fingerprint,
                proof_delta_hash: hash(43),
            }],
        )));
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk {
            snapshot: closed_child.clone(),
        }));
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk {
            snapshot: closed_child,
        }));
        client.push_replay_response(Ok(replay_scheduler_stopped_response()));

        let failure = unwrap_search_failure(phase7_run_mvp_search(
            &mut client,
            phase7_test_search_input(root),
        ));

        assert_eq!(failure.search_stats.candidates_evaluated, 3);
        let batch_sources = client
            .calls()
            .iter()
            .filter_map(|call| match call {
                Phase7MachineApiCall::TacticBatch { source } => Some(source),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(batch_sources.len(), 2);
        let repair_batch = parse_machine_tactic_batch_request(batch_sources[1]).unwrap();
        assert_eq!(repair_batch.candidates.len(), 1);
        assert_eq!(repair_batch.candidates[0].candidate_id, "c0");
        assert!(batch_sources[1].contains(r#""kind":"simp-lite""#));
        assert!(batch_sources[1].contains(r#""rules":[]"#));
        assert!(failure.trace_events.iter().any(|event| matches!(
            event.kind,
            Phase7SearchTraceEventKind::RepairChainStopped {
                reason: Phase7RepairChainStopReason::RepeatedCandidate,
                repeated_candidate_payload_hash: Some(_),
                ..
            }
        )));
        assert!(failure.trace_events.iter().any(|event| matches!(
            event.kind,
            Phase7SearchTraceEventKind::ChildQueued {
                child_node_id: Phase7NodeId(1),
                ..
            }
        )));
    }

    #[test]
    fn m6_closed_node_replays_and_verifies_before_success() {
        let root = snapshot_with_state(1, vec![goal_view(GoalId(0), 30, 5, 0, 0, None)]);
        let closed_child = snapshot_with_state(2, Vec::new());
        let replay_final_snapshot_id = SnapshotId::from_state_fingerprint(hash(90));
        let replay_final_state_fingerprint = hash(90);
        let config = mvp_config();
        let mut client = Phase7FakeMachineApiClient::new();
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk {
            snapshot: root.clone(),
        }));
        client.push_search_for_goal_response(Ok(search_ok_response(vec![theorem_result(
            "display",
            vec![suggested_candidate(
                hash(40),
                MachineTacticCandidate::SimpLite { rules: Vec::new() },
            )],
        )])));
        client.push_tactic_batch_response(Ok(ok_batch_response_for(
            root.state_fingerprint,
            config.per_tactic_deterministic_budget,
            vec![MachineTacticBatchItemResponse::Success {
                candidate_id: "c0".to_owned(),
                candidate_hash: hash(40),
                next_snapshot_id: closed_child.snapshot_id,
                next_state_fingerprint: closed_child.state_fingerprint,
                proof_delta_hash: hash(43),
            }],
        )));
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk {
            snapshot: closed_child.clone(),
        }));
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk {
            snapshot: closed_child.clone(),
        }));
        client.push_replay_response(Ok(replay_ok_response(
            replay_final_snapshot_id,
            replay_final_state_fingerprint,
        )));
        client.push_verify_response(Ok(verify_ok_response()));

        let proof = unwrap_verified_proof(phase7_run_mvp_search(
            &mut client,
            phase7_test_search_input(root),
        ));

        assert_eq!(proof.replay_plan.steps.len(), 1);
        assert_eq!(
            proof.replay_plan.final_state_fingerprint,
            closed_child.state_fingerprint
        );
        assert_eq!(proof.final_snapshot_id, replay_final_snapshot_id);
        assert_eq!(
            proof.final_state_fingerprint,
            replay_final_state_fingerprint
        );
        assert_eq!(
            proof.verify_response.status,
            MachineApiResponseStatus::Verified
        );
        assert_eq!(proof.search_stats.candidates_evaluated, 1);
        assert_eq!(proof.search_stats.closed_node_replay_rejections, 0);
        assert_eq!(proof.search_stats.closed_node_verify_rejections, 0);

        let replay_source = client.calls().iter().find_map(|call| match call {
            Phase7MachineApiCall::Replay { source } => Some(source),
            _ => None,
        });
        let replay_source = replay_source.expect("expected replay call");
        assert!(replay_source.contains(r#""steps":[{"#));
        assert!(replay_source.contains(&format!(
            r#""final_state_fingerprint":"{}""#,
            format_hash_string(&closed_child.state_fingerprint)
        )));

        let verify_source = client.calls().iter().find_map(|call| match call {
            Phase7MachineApiCall::Verify { source } => Some(source),
            _ => None,
        });
        let verify_source = verify_source.expect("expected verify call");
        assert!(verify_source.contains(&format!(
            r#""snapshot_id":"{}""#,
            replay_final_snapshot_id.wire()
        )));
        assert!(verify_source.contains(&format!(
            r#""state_fingerprint":"{}""#,
            format_hash_string(&replay_final_state_fingerprint)
        )));
        assert!(verify_source.contains(r#""mode":"certificate""#));
    }

    #[test]
    fn m6_replay_success_without_verify_is_not_verified_proof() {
        let root = snapshot_with_state(1, Vec::new());
        let replay_final_snapshot_id = SnapshotId::from_state_fingerprint(hash(90));
        let replay_final_state_fingerprint = hash(90);
        let mut client = Phase7FakeMachineApiClient::new();
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk {
            snapshot: root.clone(),
        }));
        client.push_replay_response(Ok(replay_ok_response(
            replay_final_snapshot_id,
            replay_final_state_fingerprint,
        )));
        client.push_verify_response(Ok(verify_error_response(
            MachineApiErrorKind::VerifyFailed,
            crate::MachineApiDiagnosticPhase::CertificateVerify,
        )));

        let failure = unwrap_search_failure(phase7_run_mvp_search(
            &mut client,
            phase7_test_search_input(root),
        ));

        assert_eq!(failure.reason, Phase7SearchFailureReason::QueueExhausted);
        assert_eq!(failure.best_partial_replay_prefix, None);
        assert_eq!(failure.best_snapshot_id, None);
        assert_eq!(failure.search_stats.closed_node_verify_rejections, 1);
        assert_eq!(failure.search_stats.closed_node_replay_rejections, 0);
        assert!(failure.trace_events.iter().any(|event| matches!(
            &event.kind,
            Phase7SearchTraceEventKind::ClosedNodeVerifyRejected { endpoint, status }
                if endpoint == "/machine/verify" && *status == MachineApiResponseStatus::Error
        )));
    }

    #[test]
    fn m6_replay_controller_error_preserves_phase_in_failure_reason() {
        let root = snapshot_with_state(1, Vec::new());
        let mut client = Phase7FakeMachineApiClient::new();
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk {
            snapshot: root.clone(),
        }));
        client.push_replay_response(Ok(replay_error_response(
            MachineApiErrorKind::ReplayHashMismatch,
            crate::MachineApiDiagnosticPhase::ReplayExecution,
        )));

        let failure = unwrap_search_failure(phase7_run_mvp_search(
            &mut client,
            phase7_test_search_input(root),
        ));

        assert_eq!(
            failure.reason,
            Phase7SearchFailureReason::MachineControllerError {
                endpoint: "/machine/replay".to_owned(),
                error_kind: "replay_hash_mismatch".to_owned(),
                error_phase: Some("replay_execution".to_owned()),
                diagnostic_hash: Some(hash(79)),
            }
        );
        assert_eq!(failure.search_stats.controller_errors, 1);
        assert!(failure.trace_events.iter().any(|event| matches!(
            &event.kind,
            Phase7SearchTraceEventKind::MachineControllerError { endpoint, error_kind }
                if endpoint == "/machine/replay" && error_kind == "replay_hash_mismatch"
        )));
    }

    #[test]
    fn m6_verify_controller_error_preserves_phase_in_failure_reason() {
        let root = snapshot_with_state(1, Vec::new());
        let replay_final_snapshot_id = SnapshotId::from_state_fingerprint(hash(90));
        let replay_final_state_fingerprint = hash(90);
        let mut client = Phase7FakeMachineApiClient::new();
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk {
            snapshot: root.clone(),
        }));
        client.push_replay_response(Ok(replay_ok_response(
            replay_final_snapshot_id,
            replay_final_state_fingerprint,
        )));
        client.push_verify_response(Ok(verify_error_response(
            MachineApiErrorKind::InvalidVerifyRequest,
            crate::MachineApiDiagnosticPhase::RequestValidation,
        )));

        let failure = unwrap_search_failure(phase7_run_mvp_search(
            &mut client,
            phase7_test_search_input(root),
        ));

        assert_eq!(
            failure.reason,
            Phase7SearchFailureReason::MachineControllerError {
                endpoint: "/machine/verify".to_owned(),
                error_kind: "invalid_verify_request".to_owned(),
                error_phase: Some("request_validation".to_owned()),
                diagnostic_hash: Some(hash(79)),
            }
        );
        assert_eq!(failure.search_stats.controller_errors, 1);
        assert_eq!(failure.search_stats.closed_node_verify_rejections, 0);
    }

    #[test]
    fn m7_minimization_proposal_order_is_fixed() {
        let budget = mvp_config().per_tactic_deterministic_budget;
        let edits = vec![
            Phase7ReplayStepEdit {
                original_goal_id: GoalId(0),
                original_open_goal_index: 0,
                candidate: MachineTacticCandidate::Exact {
                    term: RawMachineTerm::new("h0"),
                },
                deterministic_budget: budget,
            },
            Phase7ReplayStepEdit {
                original_goal_id: GoalId(1),
                original_open_goal_index: 0,
                candidate: MachineTacticCandidate::Exact {
                    term: RawMachineTerm::new("h1"),
                },
                deterministic_budget: budget,
            },
            Phase7ReplayStepEdit {
                original_goal_id: GoalId(2),
                original_open_goal_index: 0,
                candidate: MachineTacticCandidate::SimpLite {
                    rules: vec![simp_rule("add_zero", 40), simp_rule("zero_add", 41)],
                },
                deterministic_budget: budget,
            },
        ];

        assert_eq!(
            Phase7MinimizationPassKind::ALL,
            [
                Phase7MinimizationPassKind::DeleteRedundantSteps,
                Phase7MinimizationPassKind::ReplaceBlocksWithSimpLiteEmpty,
                Phase7MinimizationPassKind::MinimizeExistingSimpLiteRules,
            ]
        );

        let delete = phase7_delete_redundant_steps_proposals(&edits);
        assert_eq!(delete.len(), 3);
        assert_eq!(delete[0][0].original_goal_id, GoalId(1));
        assert_eq!(delete[1][0].original_goal_id, GoalId(0));
        assert_eq!(delete[1][1].original_goal_id, GoalId(2));

        let replace = phase7_replace_blocks_with_simp_lite_empty_proposals(&edits);
        assert_eq!(replace.len(), 6);
        assert_eq!(replace[0].len(), 1);
        assert!(matches!(
            replace[0][0].candidate,
            MachineTacticCandidate::SimpLite { ref rules } if rules.is_empty()
        ));
        assert_eq!(replace[0][0].original_goal_id, GoalId(0));
        assert_eq!(replace[1].len(), 2);
        assert_eq!(replace[1][0].original_goal_id, GoalId(0));
        assert_eq!(replace[2].len(), 2);
        assert_eq!(replace[2][0].original_goal_id, GoalId(0));
        assert_eq!(replace[2][1].original_goal_id, GoalId(1));

        let simp_rules = phase7_minimize_existing_simp_lite_rules_proposals(&edits);
        assert_eq!(simp_rules.len(), 2);
        assert!(matches!(
            simp_rules[0][2].candidate,
            MachineTacticCandidate::SimpLite { ref rules }
                if rules == &vec![simp_rule("zero_add", 41)]
        ));
        assert!(matches!(
            simp_rules[1][2].candidate,
            MachineTacticCandidate::SimpLite { ref rules }
                if rules == &vec![simp_rule("add_zero", 40)]
        ));
    }

    #[test]
    fn m7_rebuild_uses_open_goal_index_fallback_and_fresh_step_fields() {
        let initial = snapshot_with_state(1, vec![goal_view(GoalId(2), 30, 5, 0, 0, None)]);
        let closed = snapshot_with_state(2, Vec::new());
        let budget = mvp_config().per_tactic_deterministic_budget;
        let current_plan = Phase7ReplayPlan {
            protocol_version: MachineApiVersion::V1,
            session_root_hash: hash(90),
            initial_state_fingerprint: initial.state_fingerprint,
            steps: Vec::new(),
            final_state_fingerprint: initial.state_fingerprint,
        };
        let edit = Phase7ReplayStepEdit {
            original_goal_id: GoalId(99),
            original_open_goal_index: 0,
            candidate: MachineTacticCandidate::SimpLite { rules: Vec::new() },
            deterministic_budget: budget,
        };
        let mut client = Phase7FakeMachineApiClient::new();
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk {
            snapshot: initial.clone(),
        }));
        client.push_tactic_batch_response(Ok(ok_batch_response_for(
            initial.state_fingerprint,
            budget,
            vec![MachineTacticBatchItemResponse::Success {
                candidate_id: "c0".to_owned(),
                candidate_hash: hash(70),
                next_snapshot_id: closed.snapshot_id,
                next_state_fingerprint: closed.state_fingerprint,
                proof_delta_hash: hash(71),
            }],
        )));
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk {
            snapshot: closed.clone(),
        }));

        let rebuilt = phase7_rebuild_replay_plan_from_step_edits(
            &mut client,
            initial.session_id.clone(),
            &initial,
            &current_plan,
            &[edit],
        )
        .unwrap();

        assert_eq!(rebuilt.steps.len(), 1);
        assert_eq!(rebuilt.steps[0].goal_id, GoalId(2));
        assert_eq!(rebuilt.steps[0].candidate_hash, hash(70));
        assert_eq!(rebuilt.steps[0].proof_delta_hash, hash(71));
        assert_eq!(
            rebuilt.steps[0].deterministic_budget_hash,
            tactic_budget_hash(budget)
        );
        assert_eq!(rebuilt.final_state_fingerprint, closed.state_fingerprint);
    }

    #[test]
    fn m7_minimizer_accepts_delete_only_after_replay_and_verify() {
        let initial = snapshot_with_state(1, vec![goal_view(GoalId(0), 30, 5, 0, 0, None)]);
        let closed = snapshot_with_state(2, Vec::new());
        let budget = mvp_config().per_tactic_deterministic_budget;
        let step = Phase7ReplayStep {
            previous_state_fingerprint: initial.state_fingerprint,
            goal_id: GoalId(0),
            candidate: MachineTacticCandidate::SimpLite { rules: Vec::new() },
            deterministic_budget: budget,
            candidate_hash: hash(40),
            deterministic_budget_hash: tactic_budget_hash(budget),
            proof_delta_hash: hash(41),
            next_state_fingerprint: closed.state_fingerprint,
        };
        let plan = Phase7ReplayPlan {
            protocol_version: MachineApiVersion::V1,
            session_root_hash: hash(90),
            initial_state_fingerprint: initial.state_fingerprint,
            steps: vec![step],
            final_state_fingerprint: closed.state_fingerprint,
        };
        let mut client = Phase7FakeMachineApiClient::new();
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk {
            snapshot: initial.clone(),
        }));
        client.push_tactic_batch_response(Ok(ok_batch_response_for(
            initial.state_fingerprint,
            budget,
            vec![MachineTacticBatchItemResponse::Success {
                candidate_id: "c0".to_owned(),
                candidate_hash: hash(40),
                next_snapshot_id: closed.snapshot_id,
                next_state_fingerprint: closed.state_fingerprint,
                proof_delta_hash: hash(41),
            }],
        )));
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk { snapshot: closed }));
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk {
            snapshot: initial.clone(),
        }));
        client.push_replay_response(Ok(replay_ok_response(
            initial.snapshot_id,
            initial.state_fingerprint,
        )));
        client.push_verify_response(Ok(verify_ok_response()));

        let result = phase7_minimize_replay_plan(
            &mut client,
            initial.session_id.clone(),
            &initial,
            plan,
            MachineReplayOkFields {
                final_snapshot_id: SnapshotId::from_state_fingerprint(hash(90)),
                final_state_fingerprint: hash(90),
            },
            verify_ok_envelope(),
        );

        assert!(result.replay_plan.steps.is_empty());
        assert_eq!(
            result.replay_plan.final_state_fingerprint,
            initial.state_fingerprint
        );
        assert_eq!(
            result.replay_response.final_snapshot_id,
            initial.snapshot_id
        );
        assert_eq!(
            result.replay_response.final_state_fingerprint,
            initial.state_fingerprint
        );
        assert_eq!(result.minimization_stats.pass_kinds_attempted, 3);
        assert_eq!(result.minimization_stats.rebuilt_plans, 1);
        assert_eq!(result.minimization_stats.replay_attempts, 1);
        assert_eq!(result.minimization_stats.verify_attempts, 1);
        assert_eq!(result.minimization_stats.accepted_proposals, 1);
    }

    #[test]
    fn m7_minimizer_keeps_verified_plan_when_verify_rejects_proposal() {
        let initial = snapshot_with_state(1, vec![goal_view(GoalId(0), 30, 5, 0, 0, None)]);
        let closed = snapshot_with_state(2, Vec::new());
        let budget = mvp_config().per_tactic_deterministic_budget;
        let step = Phase7ReplayStep {
            previous_state_fingerprint: initial.state_fingerprint,
            goal_id: GoalId(0),
            candidate: MachineTacticCandidate::SimpLite { rules: Vec::new() },
            deterministic_budget: budget,
            candidate_hash: hash(40),
            deterministic_budget_hash: tactic_budget_hash(budget),
            proof_delta_hash: hash(41),
            next_state_fingerprint: closed.state_fingerprint,
        };
        let plan = Phase7ReplayPlan {
            protocol_version: MachineApiVersion::V1,
            session_root_hash: hash(90),
            initial_state_fingerprint: initial.state_fingerprint,
            steps: vec![step],
            final_state_fingerprint: closed.state_fingerprint,
        };
        let mut client = Phase7FakeMachineApiClient::new();
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk {
            snapshot: initial.clone(),
        }));
        client.push_tactic_batch_response(Ok(ok_batch_response_for(
            initial.state_fingerprint,
            budget,
            vec![MachineTacticBatchItemResponse::Success {
                candidate_id: "c0".to_owned(),
                candidate_hash: hash(40),
                next_snapshot_id: closed.snapshot_id,
                next_state_fingerprint: closed.state_fingerprint,
                proof_delta_hash: hash(41),
            }],
        )));
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk {
            snapshot: closed.clone(),
        }));
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk { snapshot: initial }));
        client.push_replay_response(Ok(replay_ok_response(
            SnapshotId::from_state_fingerprint(hash(91)),
            hash(91),
        )));
        client.push_verify_response(Ok(verify_error_response(
            MachineApiErrorKind::VerifyFailed,
            crate::MachineApiDiagnosticPhase::CertificateVerify,
        )));

        let result = phase7_minimize_replay_plan(
            &mut client,
            closed.session_id.clone(),
            &snapshot_with_state(1, vec![goal_view(GoalId(0), 30, 5, 0, 0, None)]),
            plan,
            MachineReplayOkFields {
                final_snapshot_id: closed.snapshot_id,
                final_state_fingerprint: closed.state_fingerprint,
            },
            verify_ok_envelope(),
        );

        assert_eq!(result.replay_plan.steps.len(), 1);
        assert_eq!(
            result.replay_plan.final_state_fingerprint,
            closed.state_fingerprint
        );
        assert_eq!(result.replay_response.final_snapshot_id, closed.snapshot_id);
        assert_eq!(result.minimization_stats.replay_attempts, 1);
        assert_eq!(result.minimization_stats.verify_attempts, 1);
        assert_eq!(result.minimization_stats.accepted_proposals, 0);
    }

    #[test]
    fn m4_search_priority_and_best_partial_keys_are_deterministic() {
        let one_goal = snapshot_with_state(1, vec![goal_view(GoalId(0), 30, 9, 0, 0, None)]);
        let two_goals = snapshot_with_state(
            2,
            vec![
                goal_view(GoalId(0), 31, 4, 0, 0, None),
                goal_view(GoalId(1), 32, 5, 0, 0, None),
            ],
        );
        let mut one_goal_node =
            phase7_root_search_node(&phase7_test_search_input(one_goal), Phase7NodeId(1));
        let mut two_goal_node =
            phase7_root_search_node(&phase7_test_search_input(two_goals), Phase7NodeId(2));
        one_goal_node.depth = 1;
        two_goal_node.depth = 0;

        let one_goal_priority = phase7_search_node_priority_key(&one_goal_node);
        assert_eq!(one_goal_priority.open_goal_count, 1);
        assert_eq!(one_goal_priority.depth, 1);
        assert_eq!(one_goal_priority.total_open_goal_target_size, 9);
        assert!(one_goal_priority < phase7_search_node_priority_key(&two_goal_node));

        let two_goal_partial = phase7_search_node_best_partial_key(&two_goal_node);
        assert_eq!(two_goal_partial.open_goal_count, 2);
        assert_eq!(two_goal_partial.total_open_goal_target_size, 9);
        assert!(phase7_search_node_best_partial_key(&one_goal_node) < two_goal_partial);
    }

    #[test]
    fn m4_search_respects_max_depth_without_expanding_node() {
        let root = snapshot_with_state(1, vec![goal_view(GoalId(0), 30, 5, 0, 0, None)]);
        let mut input = phase7_test_search_input(root.clone());
        input.search_budget.max_depth = 0;
        let mut client = Phase7FakeMachineApiClient::new();
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk {
            snapshot: root.clone(),
        }));

        let failure = unwrap_search_failure(phase7_run_mvp_search(&mut client, input));

        assert_eq!(
            failure.reason,
            Phase7SearchFailureReason::SearchBudgetExceeded {
                limit: Phase7SearchBudgetLimit::MaxDepth
            }
        );
        assert_eq!(failure.search_stats.nodes_expanded, 0);
        assert_eq!(failure.search_stats.max_depth_stops, 1);
        assert_eq!(failure.best_snapshot_id, Some(root.snapshot_id));
        assert!(failure.trace_events.iter().any(|event| matches!(
            event.kind,
            Phase7SearchTraceEventKind::MaxDepthStopped { max_depth: 0 }
        )));
        assert_eq!(client.calls().len(), 1);
    }

    #[test]
    fn m4_search_no_candidate_initial_goal_returns_no_candidate_failure() {
        let root = snapshot_with_state(1, vec![goal_view(GoalId(0), 30, 5, 0, 0, None)]);
        let mut client = Phase7FakeMachineApiClient::new();
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk {
            snapshot: root.clone(),
        }));
        client.push_search_for_goal_response(Ok(search_ok_response(Vec::new())));

        let failure = unwrap_search_failure(phase7_run_mvp_search(
            &mut client,
            phase7_test_search_input(root),
        ));

        assert_eq!(
            failure.reason,
            Phase7SearchFailureReason::NoCandidateForSelectedGoal { goal_id: GoalId(0) }
        );
        assert_eq!(failure.search_stats.nodes_expanded, 1);
        assert_eq!(failure.search_stats.no_candidate_stops, 1);
        assert!(failure.trace_events.iter().any(|event| matches!(
            event.kind,
            Phase7SearchTraceEventKind::NoCandidateForSelectedGoal { goal_id: GoalId(0) }
        )));
    }

    #[test]
    fn m4_search_caps_batch_to_max_tactics_per_node() {
        let root = snapshot_with_state(1, vec![goal_view(GoalId(0), 30, 5, 0, 0, None)]);
        let config = mvp_config();
        let mut input = phase7_test_search_input(root.clone());
        input.search_budget.max_tactics_per_node = 1;
        let suggested = vec![
            suggested_candidate(
                hash(40),
                MachineTacticCandidate::SimpLite { rules: Vec::new() },
            ),
            suggested_candidate(
                hash(41),
                MachineTacticCandidate::Exact {
                    term: RawMachineTerm::new("h"),
                },
            ),
        ];
        let mut client = Phase7FakeMachineApiClient::new();
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk {
            snapshot: root.clone(),
        }));
        client.push_search_for_goal_response(Ok(search_ok_response(vec![theorem_result(
            "display", suggested,
        )])));
        client.push_tactic_batch_response(Ok(ok_batch_response_for(
            root.state_fingerprint,
            config.per_tactic_deterministic_budget,
            vec![MachineTacticBatchItemResponse::Error {
                candidate_id: "c0".to_owned(),
                candidate_hash: Some(hash(40)),
                diagnostic: compact_error(MachineApiErrorKind::TypeMismatch),
            }],
        )));

        let failure = unwrap_search_failure(phase7_run_mvp_search(&mut client, input));

        assert_eq!(failure.search_stats.candidates_evaluated, 1);
        assert!(failure.trace_events.iter().any(|event| matches!(
            event.kind,
            Phase7SearchTraceEventKind::MaxTacticsPerNodeStopped {
                max_tactics_per_node: 1
            }
        )));
        let batch_source = client.calls().iter().find_map(|call| match call {
            Phase7MachineApiCall::TacticBatch { source } => Some(source),
            _ => None,
        });
        let parsed = parse_machine_tactic_batch_request(batch_source.unwrap()).unwrap();
        assert_eq!(parsed.candidates.len(), 1);
        assert_eq!(parsed.candidates[0].candidate_id, "c0");
    }

    #[test]
    fn m4_search_rejects_no_progress_ok_batch_as_controller_error() {
        let root = snapshot_with_state(1, vec![goal_view(GoalId(0), 30, 5, 0, 0, None)]);
        let config = mvp_config();
        let mut client = Phase7FakeMachineApiClient::new();
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk {
            snapshot: root.clone(),
        }));
        client.push_search_for_goal_response(Ok(search_ok_response(vec![theorem_result(
            "display",
            vec![suggested_candidate(
                hash(40),
                MachineTacticCandidate::SimpLite { rules: Vec::new() },
            )],
        )])));
        client.push_tactic_batch_response(Ok(ok_batch_response_for(
            root.state_fingerprint,
            config.per_tactic_deterministic_budget,
            Vec::new(),
        )));

        let failure = unwrap_search_failure(phase7_run_mvp_search(
            &mut client,
            phase7_test_search_input(root),
        ));

        assert_eq!(
            failure.reason,
            Phase7SearchFailureReason::MachineControllerError {
                endpoint: "/machine/tactics/batch".to_owned(),
                error_kind: "batch_response_contract_violation".to_owned(),
                error_phase: None,
                diagnostic_hash: None,
            }
        );
        assert_eq!(failure.search_stats.controller_errors, 1);
        assert_eq!(failure.search_stats.candidates_evaluated, 0);
        assert!(failure.trace_events.iter().any(|event| matches!(
            &event.kind,
            Phase7SearchTraceEventKind::MachineControllerError { endpoint, error_kind }
                if endpoint == "/machine/tactics/batch"
                    && error_kind == "batch_response_contract_violation"
        )));
    }

    #[test]
    fn m4_search_records_duplicate_state_without_queueing_it() {
        let root = snapshot_with_state(1, vec![goal_view(GoalId(0), 30, 5, 0, 0, None)]);
        let config = mvp_config();
        let mut client = Phase7FakeMachineApiClient::new();
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk {
            snapshot: root.clone(),
        }));
        client.push_search_for_goal_response(Ok(search_ok_response(vec![theorem_result(
            "display",
            vec![suggested_candidate(
                hash(40),
                MachineTacticCandidate::SimpLite { rules: Vec::new() },
            )],
        )])));
        client.push_tactic_batch_response(Ok(ok_batch_response_for(
            root.state_fingerprint,
            config.per_tactic_deterministic_budget,
            vec![MachineTacticBatchItemResponse::Success {
                candidate_id: "c0".to_owned(),
                candidate_hash: hash(40),
                next_snapshot_id: root.snapshot_id,
                next_state_fingerprint: root.state_fingerprint,
                proof_delta_hash: hash(43),
            }],
        )));

        let failure = unwrap_search_failure(phase7_run_mvp_search(
            &mut client,
            phase7_test_search_input(root.clone()),
        ));

        assert_eq!(failure.search_stats.candidates_evaluated, 1);
        assert!(failure.trace_events.iter().any(|event| matches!(
            event.kind,
            Phase7SearchTraceEventKind::DuplicateStateSkipped {
                duplicate_state_fingerprint
            } if duplicate_state_fingerprint == root.state_fingerprint
        )));
        assert!(!failure
            .trace_events
            .iter()
            .any(|event| { matches!(event.kind, Phase7SearchTraceEventKind::ChildQueued { .. }) }));
        assert_eq!(client.calls().len(), 3);
    }

    #[test]
    fn m4_search_allocates_child_node_ids_in_batch_success_order() {
        let root = snapshot_with_state(1, vec![goal_view(GoalId(0), 30, 5, 0, 0, None)]);
        let child0 = snapshot_with_state(2, vec![goal_view(GoalId(1), 31, 5, 0, 0, None)]);
        let child1 = snapshot_with_state(3, vec![goal_view(GoalId(2), 32, 5, 0, 0, None)]);
        let config = mvp_config();
        let mut input = phase7_test_search_input(root.clone());
        input.search_budget.max_nodes = 1;
        let mut client = Phase7FakeMachineApiClient::new();
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk {
            snapshot: root.clone(),
        }));
        client.push_search_for_goal_response(Ok(search_ok_response(vec![theorem_result(
            "display",
            vec![
                suggested_candidate(
                    hash(40),
                    MachineTacticCandidate::SimpLite { rules: Vec::new() },
                ),
                suggested_candidate(
                    hash(41),
                    MachineTacticCandidate::Exact {
                        term: RawMachineTerm::new("h"),
                    },
                ),
            ],
        )])));
        client.push_tactic_batch_response(Ok(ok_batch_response_for(
            root.state_fingerprint,
            config.per_tactic_deterministic_budget,
            vec![
                MachineTacticBatchItemResponse::Success {
                    candidate_id: "c0".to_owned(),
                    candidate_hash: hash(40),
                    next_snapshot_id: child0.snapshot_id,
                    next_state_fingerprint: child0.state_fingerprint,
                    proof_delta_hash: hash(43),
                },
                MachineTacticBatchItemResponse::Success {
                    candidate_id: "c1".to_owned(),
                    candidate_hash: hash(41),
                    next_snapshot_id: child1.snapshot_id,
                    next_state_fingerprint: child1.state_fingerprint,
                    proof_delta_hash: hash(44),
                },
            ],
        )));
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk {
            snapshot: child0.clone(),
        }));
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk { snapshot: child1 }));
        client.push_snapshot_get_response(Ok(MachineSnapshotGetOk { snapshot: child0 }));

        let failure = unwrap_search_failure(phase7_run_mvp_search(&mut client, input));

        assert_eq!(
            failure.reason,
            Phase7SearchFailureReason::SearchBudgetExceeded {
                limit: Phase7SearchBudgetLimit::MaxNodes
            }
        );
        let child_ids = failure
            .trace_events
            .iter()
            .filter_map(|event| match event.kind {
                Phase7SearchTraceEventKind::ChildQueued { child_node_id, .. } => {
                    Some(child_node_id)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(child_ids, vec![Phase7NodeId(1), Phase7NodeId(2)]);
    }

    #[test]
    fn fake_client_validates_raw_phase5_requests_before_queue_lookup() {
        let mut client = Phase7FakeMachineApiClient::new();

        let cases = [
            (
                client.search_for_goal("{}").unwrap_err(),
                Phase7MachineApiEndpointKind::SearchForGoal,
                MachineApiErrorKind::InvalidTheoremQuery,
            ),
            (
                client.run_tactic_batch("{}").unwrap_err(),
                Phase7MachineApiEndpointKind::TacticBatch,
                MachineApiErrorKind::InvalidBatchPolicy,
            ),
            (
                client.replay("{}").unwrap_err(),
                Phase7MachineApiEndpointKind::Replay,
                MachineApiErrorKind::InvalidReplayPlan,
            ),
            (
                client.verify("{}").unwrap_err(),
                Phase7MachineApiEndpointKind::Verify,
                MachineApiErrorKind::InvalidVerifyRequest,
            ),
        ];

        for (error, endpoint, kind) in cases {
            match error {
                Phase7MachineApiError::FakeRequestValidation {
                    endpoint: actual,
                    error,
                } => {
                    assert_eq!(actual, endpoint);
                    assert_eq!(error.kind, kind);
                }
                other => panic!("expected fake request validation error, got {other:?}"),
            }
        }
        assert!(client.calls().is_empty());
    }

    #[test]
    fn phase7_mvp_config_accepts_omitted_scheduler_limits() {
        let config = parse_phase7_mvp_controller_config(valid_config_json()).unwrap();

        assert_eq!(config.search_budget.max_tactics_per_node, 16);
        assert_eq!(config.scheduler_limits, None);
        assert_eq!(config.batch_policy.max_evaluated_candidates, 16);
    }

    #[test]
    fn phase7_mvp_config_accepts_present_scheduler_limits() {
        let source = valid_config_json().replace(
            r#""batch_policy""#,
            r#""scheduler_limits":{"per_candidate_timeout_ms":100,"batch_timeout_ms":1000,"max_memory_mb":1024},"batch_policy""#,
        );

        let config = parse_phase7_mvp_controller_config(&source).unwrap();

        assert_eq!(
            config.scheduler_limits,
            Some(MachineBatchSchedulerLimits {
                per_candidate_timeout_ms: Some(100),
                batch_timeout_ms: Some(1000),
                max_memory_mb: Some(1024),
            })
        );
    }

    #[test]
    fn phase7_mvp_config_rejects_non_mvp_tactics_per_node() {
        let source = valid_config_json().replace(
            r#""max_tactics_per_node": 16"#,
            r#""max_tactics_per_node": 8"#,
        );

        let err = parse_phase7_mvp_controller_config(&source).unwrap_err();

        assert_eq!(err.kind, MachineApiErrorKind::InvalidBatchPolicy);
        assert_eq!(
            err.path,
            JsonPath::root()
                .field("search_budget")
                .field("max_tactics_per_node")
        );
    }

    #[test]
    fn phase7_mvp_config_rejects_tactics_per_node_outside_u32_range() {
        let source = valid_config_json().replace(
            r#""max_tactics_per_node": 16"#,
            r#""max_tactics_per_node": 4294967296"#,
        );

        let err = parse_phase7_mvp_controller_config(&source).unwrap_err();

        assert_eq!(err.kind, MachineApiErrorKind::InvalidBatchPolicy);
        assert_eq!(
            err.reason,
            MachineApiRequestErrorReason::InvalidUnsignedInteger {
                field: "max_tactics_per_node",
                raw: "4294967296".to_owned(),
                error: StrictUnsignedIntegerError::ExceedsMaximum {
                    max: u64::from(u32::MAX)
                },
            }
        );
    }

    #[test]
    fn phase7_mvp_config_rejects_null_scheduler_limits() {
        let source = valid_config_json().replace(
            r#""batch_policy""#,
            r#""scheduler_limits":null,"batch_policy""#,
        );

        let err = parse_phase7_mvp_controller_config(&source).unwrap_err();

        assert_eq!(err.kind, MachineApiErrorKind::InvalidSchedulerLimits);
        assert_eq!(
            err.reason,
            MachineApiRequestErrorReason::NullField {
                field: "scheduler_limits"
            }
        );
    }

    #[test]
    fn phase7_mvp_config_rejects_unknown_field() {
        let source =
            valid_config_json().replace(r#""batch_policy""#, r#""unknown":true,"batch_policy""#);

        let err = parse_phase7_mvp_controller_config(&source).unwrap_err();

        assert_eq!(err.kind, MachineApiErrorKind::InvalidBatchPolicy);
        assert_eq!(
            err.reason,
            MachineApiRequestErrorReason::UnknownField {
                field: "unknown".to_owned()
            }
        );
    }

    #[test]
    fn phase7_mvp_config_rejects_float_search_budget() {
        let source = valid_config_json().replace(r#""max_nodes": 10000"#, r#""max_nodes": 1.5"#);

        let err = parse_phase7_mvp_controller_config(&source).unwrap_err();

        assert_eq!(err.kind, MachineApiErrorKind::InvalidBatchPolicy);
        assert_eq!(
            err.reason,
            MachineApiRequestErrorReason::InvalidUnsignedInteger {
                field: "max_nodes",
                raw: "1.5".to_owned(),
                error: StrictUnsignedIntegerError::InvalidGrammar,
            }
        );
    }

    #[test]
    fn phase7_mvp_config_rejects_negative_search_budget() {
        let source = valid_config_json().replace(r#""max_nodes": 10000"#, r#""max_nodes": -1"#);

        let err = parse_phase7_mvp_controller_config(&source).unwrap_err();

        assert_eq!(err.kind, MachineApiErrorKind::InvalidBatchPolicy);
        assert_eq!(
            err.reason,
            MachineApiRequestErrorReason::InvalidUnsignedInteger {
                field: "max_nodes",
                raw: "-1".to_owned(),
                error: StrictUnsignedIntegerError::InvalidGrammar,
            }
        );
    }

    #[test]
    fn phase7_mvp_config_rejects_max_depth_outside_u32_range() {
        let source =
            valid_config_json().replace(r#""max_depth": 64"#, r#""max_depth": 4294967296"#);

        let err = parse_phase7_mvp_controller_config(&source).unwrap_err();

        assert_eq!(err.kind, MachineApiErrorKind::InvalidBatchPolicy);
        assert_eq!(
            err.reason,
            MachineApiRequestErrorReason::InvalidUnsignedInteger {
                field: "max_depth",
                raw: "4294967296".to_owned(),
                error: StrictUnsignedIntegerError::ExceedsMaximum {
                    max: u64::from(u32::MAX)
                },
            }
        );
    }

    #[test]
    fn phase7_mvp_config_rejects_scheduler_zero() {
        let source = valid_config_json().replace(
            r#""batch_policy""#,
            r#""scheduler_limits":{"batch_timeout_ms":0},"batch_policy""#,
        );

        let err = parse_phase7_mvp_controller_config(&source).unwrap_err();

        assert_eq!(err.kind, MachineApiErrorKind::InvalidSchedulerLimits);
        assert_eq!(
            err.reason,
            MachineApiRequestErrorReason::InvalidUnsignedInteger {
                field: "batch_timeout_ms",
                raw: "0".to_owned(),
                error: StrictUnsignedIntegerError::InvalidGrammar,
            }
        );
    }

    #[test]
    fn phase7_mvp_config_rejects_scheduler_string() {
        let source = valid_config_json().replace(
            r#""batch_policy""#,
            r#""scheduler_limits":{"batch_timeout_ms":"1000"},"batch_policy""#,
        );

        let err = parse_phase7_mvp_controller_config(&source).unwrap_err();

        assert_eq!(err.kind, MachineApiErrorKind::InvalidSchedulerLimits);
        assert_eq!(
            err.reason,
            MachineApiRequestErrorReason::TypeMismatch {
                field: "batch_timeout_ms",
                expected: JsonFieldType::UnsignedInteger { max: u64::MAX },
                actual: crate::JsonValueKind::String,
            }
        );
    }
}
