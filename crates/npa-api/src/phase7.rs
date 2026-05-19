use std::cmp::Ordering;
use std::collections::VecDeque;

use npa_cert::{Hash, Name};
use npa_tactic::{goal_id_canonical_bytes, GoalId, MachineTacticBatchPolicy, TacticBudget};

use crate::current::MachineAxiomRefWire;
use crate::json::{JsonMember, JsonValue, JsonValueKind};
use crate::renderer::MachineGlobalRefView;
use crate::snapshot::{MachineSnapshotGetError, MachineSnapshotGetOk};
use crate::tactic::parse_deterministic_budget_with_error_kind;
use crate::types::{
    format_goal_id_wire, format_hash_string, MachineApiErrorWire, MachineApiResponseEnvelope,
    MachineGoalView, MachineProofSession, MachineProofSnapshot, SessionId, SnapshotId,
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
    MachineBatchSchedulerLimits, MachineReplayError, MachineReplayResponse,
    MachineTacticBatchError, MachineTacticBatchResponse, MachineTheoremMode,
    MachineTheoremSearchError, MachineTheoremSearchOkFields, MachineTheoremSearchResponse,
    MachineTheoremSearchResult, MachineVerifyError, MachineVerifyResponse,
};

const PHASE7_MVP_MAX_TACTICS_PER_NODE: u32 = 16;
const PHASE7_MVP_PREMISE_QUERY_LIMIT: u32 = 32;

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
    use npa_tactic::{MachineTacticCandidate, MetaVarId};

    use crate::{
        parse_machine_snapshot_get_request, parse_machine_theorem_search_request, JsonFieldType,
        LocalId, MachineAllowedModulesFilter, MachineApiOkResponse, MachineApiRequestErrorReason,
        MachineApiResponseEnvelope, MachineApiResponseStatus, MachineExprView, MachineLocalView,
        MachineSuggestedCandidate, MachineSuggestedCandidateStatus, MachineTheoremGlobalRef,
        MachineTheoremStatement,
    };

    fn hash(byte: u8) -> Hash {
        [byte; 32]
    }

    fn name(value: &str) -> Name {
        Name::from_dotted(value)
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
        MachineProofSnapshot {
            snapshot_id: SnapshotId::from_state_fingerprint(hash(1)),
            session_id: SessionId::parse("msess_001").unwrap(),
            state_fingerprint: hash(1),
            tactic_options_fingerprint: hash(2),
            open_goals: goals.iter().map(|goal| goal.goal_id).collect(),
            goals,
            proof_skeleton_hash: hash(3),
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
