use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use npa_cert::{Hash, Name};
use npa_kernel::Level;
use npa_tactic::{
    tactic_budget_hash, CandidateApplyArg, CandidateRewriteRuleRef, MachineTacticBatchPolicy,
    MachineTacticCandidate, MachineTacticDiagnostic, MachineTacticDiagnosticKind, RawMachineTerm,
    RewriteDirection, RewriteSite, SimpRuleRef, TacticBudget, TacticHead,
};

use crate::adapter::{
    phase4_run_machine_tactic_with_budget, phase4_validate_machine_tactic_candidate,
    MachineApiDiagnosticPhase, MachineApiDiagnosticProjection, MachineApiTacticKind,
    Phase4AdapterError,
};
use crate::json::{JsonDocument, JsonMember, JsonValue, JsonValueKind};
use crate::snapshot::{
    MachineSnapshotLookupError, MachineSnapshotMaterializationContext,
    MachineSnapshotMaterializationError, MachineSnapshotStoreError,
};
use crate::types::{
    is_machine_local_name, is_machine_universe_param_name, parse_goal_id_wire,
    parse_machine_surface_renderable_name_wire, HashString, MachineApiCompactErrorWire,
    MachineApiErrorResponse, MachineApiErrorWire, MachineApiResponseStatus,
    MachineApiSchedulerResponse, MachineProofSession, MachineSchedulerArtifact,
    MachineSchedulerArtifactKind, MachineSchedulerArtifactScope, SessionId, SnapshotId,
};
use crate::validation::{
    delayed_json_payload, parse_request_body, parse_strict_u64_token, validate_json_object,
    DelayedJsonPayload, FieldSpec, JsonFieldType, JsonPath, JsonPathElement, MachineApiErrorKind,
    MachineApiRequestError, MachineApiRequestErrorReason, ObjectSchema, StrictUnsignedIntegerError,
};
use crate::{MachineApiResponseEnvelope, Phase5UpstreamDiagnostic};

const BUDGET_FIELDS: &[FieldSpec] = &[
    FieldSpec::required(
        "max_tactic_steps",
        JsonFieldType::UnsignedInteger { max: u64::MAX },
    ),
    FieldSpec::required(
        "max_whnf_steps",
        JsonFieldType::UnsignedInteger { max: u64::MAX },
    ),
    FieldSpec::required(
        "max_conversion_steps",
        JsonFieldType::UnsignedInteger { max: u64::MAX },
    ),
    FieldSpec::required(
        "max_rewrite_steps",
        JsonFieldType::UnsignedInteger { max: u64::MAX },
    ),
    FieldSpec::required(
        "max_meta_allocations",
        JsonFieldType::UnsignedInteger { max: u64::MAX },
    ),
    FieldSpec::required(
        "max_expr_nodes",
        JsonFieldType::UnsignedInteger { max: u64::MAX },
    ),
];

const RUN_SCHEDULER_FIELDS: &[FieldSpec] = &[
    FieldSpec::optional(
        "timeout_ms",
        JsonFieldType::UnsignedInteger { max: u64::MAX },
    ),
    FieldSpec::optional(
        "max_memory_mb",
        JsonFieldType::UnsignedInteger { max: u64::MAX },
    ),
];

const BATCH_POLICY_FIELDS: &[FieldSpec] = &[
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

const BATCH_SCHEDULER_FIELDS: &[FieldSpec] = &[
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

const RAW_MACHINE_TERM_FIELDS: &[FieldSpec] =
    &[FieldSpec::required("source", JsonFieldType::String)];
const EXACT_FIELDS: &[FieldSpec] = &[
    FieldSpec::required("kind", JsonFieldType::String),
    FieldSpec::required("term", JsonFieldType::Object),
];
const INTRO_FIELDS: &[FieldSpec] = &[
    FieldSpec::required("kind", JsonFieldType::String),
    FieldSpec::required("name", JsonFieldType::String),
];
const APPLY_FIELDS: &[FieldSpec] = &[
    FieldSpec::required("kind", JsonFieldType::String),
    FieldSpec::required("head", JsonFieldType::Object),
    FieldSpec::required("universe_args", JsonFieldType::Array),
    FieldSpec::required("args", JsonFieldType::Array),
];
const REWRITE_FIELDS: &[FieldSpec] = &[
    FieldSpec::required("kind", JsonFieldType::String),
    FieldSpec::required("rule", JsonFieldType::Object),
    FieldSpec::required("direction", JsonFieldType::String),
    FieldSpec::required("site", JsonFieldType::String),
];
const REWRITE_RULE_FIELDS: &[FieldSpec] = &[
    FieldSpec::required("head", JsonFieldType::Object),
    FieldSpec::required("universe_args", JsonFieldType::Array),
    FieldSpec::required("args", JsonFieldType::Array),
];
const SIMP_LITE_FIELDS: &[FieldSpec] = &[
    FieldSpec::required("kind", JsonFieldType::String),
    FieldSpec::required("rules", JsonFieldType::Array),
];
const INDUCTION_NAT_FIELDS: &[FieldSpec] = &[
    FieldSpec::required("kind", JsonFieldType::String),
    FieldSpec::required("local_name", JsonFieldType::String),
];
const IMPORTED_HEAD_FIELDS: &[FieldSpec] = &[
    FieldSpec::required("name", JsonFieldType::String),
    FieldSpec::required("decl_interface_hash", JsonFieldType::String),
];
const CURRENT_HEAD_FIELDS: &[FieldSpec] = IMPORTED_HEAD_FIELDS;
const LOCAL_HEAD_FIELDS: &[FieldSpec] = &[FieldSpec::required("name", JsonFieldType::String)];
const ARG_TERM_FIELDS: &[FieldSpec] = &[
    FieldSpec::required("mode", JsonFieldType::String),
    FieldSpec::required("term", JsonFieldType::Object),
];
const ARG_SUBGOAL_FIELDS: &[FieldSpec] = &[
    FieldSpec::required("mode", JsonFieldType::String),
    FieldSpec::required("name_hint", JsonFieldType::String).allow_null(),
];
const ARG_INFER_FIELDS: &[FieldSpec] = &[FieldSpec::required("mode", JsonFieldType::String)];
const SIMP_RULE_FIELDS: &[FieldSpec] = &[
    FieldSpec::required("name", JsonFieldType::String),
    FieldSpec::required("decl_interface_hash", JsonFieldType::String),
    FieldSpec::required("direction", JsonFieldType::String),
];

const MAX_NUMERIC_UNIVERSE_LEVEL: u64 = 1024;

pub type MachineTacticRunResponse = MachineApiResponseEnvelope<
    MachineTacticRunSuccessFields,
    MachineTacticRunErrorObject,
    MachineTacticRunErrorFields,
    MachineTacticRunSchedulerFields,
>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTacticRunSuccessFields {
    pub result: MachineTacticRunSuccessResult,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTacticRunSuccessResult {
    pub kind: MachineTacticRunResultKind,
    pub previous_state_fingerprint: Hash,
    pub candidate_hash: Hash,
    pub deterministic_budget_hash: Hash,
    pub next_snapshot_id: SnapshotId,
    pub next_state_fingerprint: Hash,
    pub closed_goals: Vec<npa_tactic::GoalId>,
    pub new_goals: Vec<npa_tactic::GoalId>,
    pub delta: MachineTacticRunDeltaSummary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineTacticRunResultKind {
    Closed,
    Expanded,
}

impl MachineTacticRunResultKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Closed => "closed",
            Self::Expanded => "expanded",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTacticRunDeltaSummary {
    pub proof_delta_hash: Hash,
    pub assigned_goal: npa_tactic::GoalId,
    pub assigned_proof_expr_hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTacticRunErrorObject {
    pub diagnostic: MachineApiErrorWire,
    pub candidate_hash: Option<Hash>,
    pub deterministic_budget_hash: Option<Hash>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTacticRunErrorFields {
    pub unchanged_state_fingerprint: Option<Hash>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTacticRunSchedulerFields {
    pub previous_state_fingerprint: Hash,
    pub deterministic_budget_hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTacticRunError {
    pub diagnostic: MachineApiDiagnosticProjection,
    pub response: MachineTacticRunResponse,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTacticRunRequest<'src> {
    pub session_id: SessionId,
    pub snapshot_id: SnapshotId,
    pub state_fingerprint: Hash,
    pub goal_id: npa_tactic::GoalId,
    pub candidate: DelayedJsonPayload<'src>,
    pub deterministic_budget: TacticBudget,
    pub scheduler_limits: MachineRunSchedulerLimits,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MachineRunSchedulerLimits {
    pub timeout_ms: Option<u64>,
    pub max_memory_mb: Option<u64>,
}

pub type MachineTacticBatchResponse = MachineApiResponseEnvelope<
    MachineTacticBatchOkFields,
    MachineApiErrorWire,
    (),
    MachineTacticBatchSchedulerFields,
>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTacticBatchOkFields {
    pub previous_state_fingerprint: Hash,
    pub deterministic_budget_hash: Hash,
    pub results: Vec<MachineTacticBatchItemResponse>,
    pub success_count: u32,
    pub failure_count: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MachineTacticBatchItemResponse {
    Success {
        candidate_id: String,
        candidate_hash: Hash,
        next_snapshot_id: SnapshotId,
        next_state_fingerprint: Hash,
        proof_delta_hash: Hash,
    },
    Error {
        candidate_id: String,
        candidate_hash: Option<Hash>,
        diagnostic: MachineApiCompactErrorWire,
    },
}

impl MachineTacticBatchItemResponse {
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTacticBatchSchedulerFields {
    pub previous_state_fingerprint: Hash,
    pub deterministic_budget_hash: Hash,
    pub completed_prefix_len: u32,
    pub results: Vec<MachineTacticBatchItemResponse>,
    pub success_count: u32,
    pub failure_count: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTacticBatchError {
    pub diagnostic: MachineApiDiagnosticProjection,
    pub response: MachineTacticBatchResponse,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTacticBatchRequest<'src> {
    pub session_id: SessionId,
    pub snapshot_id: SnapshotId,
    pub state_fingerprint: Hash,
    pub goal_id: npa_tactic::GoalId,
    pub candidates: Vec<MachineTacticBatchCandidateRequest<'src>>,
    pub deterministic_budget: TacticBudget,
    pub batch_policy: MachineTacticBatchPolicy,
    pub scheduler_limits: MachineBatchSchedulerLimits,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTacticBatchCandidateRequest<'src> {
    pub candidate_id: String,
    pub candidate: DelayedJsonPayload<'src>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MachineBatchSchedulerLimits {
    pub per_candidate_timeout_ms: Option<u64>,
    pub batch_timeout_ms: Option<u64>,
    pub max_memory_mb: Option<u64>,
}

#[derive(Clone, Copy, Debug, Default)]
struct RunErrorCorrelation {
    unchanged_state_fingerprint: Option<Hash>,
    candidate_hash: Option<Hash>,
    deterministic_budget_hash: Option<Hash>,
    goal_id: Option<npa_tactic::GoalId>,
    tactic_kind: Option<MachineApiTacticKind>,
}

pub fn run_machine_tactic_request(
    source: &str,
    session: &mut MachineProofSession,
) -> Result<MachineTacticRunResponse, Box<MachineTacticRunError>> {
    run_machine_tactic_request_in_sessions(source, std::iter::once(session))
}

pub fn run_machine_tactic_request_in_sessions<'session>(
    source: &str,
    sessions: impl IntoIterator<Item = &'session mut MachineProofSession>,
) -> Result<MachineTacticRunResponse, Box<MachineTacticRunError>> {
    let request = parse_machine_tactic_run_request(source).map_err(request_error)?;
    let Some(session) = sessions
        .into_iter()
        .find(|session| session.session_id == request.session_id)
    else {
        return Err(plain_error(
            MachineApiErrorKind::UnknownSession,
            MachineApiDiagnosticPhase::SessionLookup,
            format!("unknown session {}", request.session_id.wire()),
            RunErrorCorrelation::default(),
        ));
    };

    run_machine_tactic_request_parsed(session, request)
}

pub fn run_machine_tactic_batch_request(
    source: &str,
    session: &mut MachineProofSession,
) -> Result<MachineTacticBatchResponse, Box<MachineTacticBatchError>> {
    run_machine_tactic_batch_request_in_sessions(source, std::iter::once(session))
}

pub fn run_machine_tactic_batch_request_in_sessions<'session>(
    source: &str,
    sessions: impl IntoIterator<Item = &'session mut MachineProofSession>,
) -> Result<MachineTacticBatchResponse, Box<MachineTacticBatchError>> {
    let request = parse_machine_tactic_batch_request(source).map_err(batch_request_error)?;
    let Some(session) = sessions
        .into_iter()
        .find(|session| session.session_id == request.session_id)
    else {
        return Err(batch_plain_error(
            MachineApiErrorKind::UnknownSession,
            MachineApiDiagnosticPhase::SessionLookup,
            format!("unknown session {}", request.session_id.wire()),
            None,
        ));
    };

    run_machine_tactic_batch_request_parsed(session, request)
}

pub fn parse_machine_tactic_run_request<'src>(
    source: &'src str,
) -> Result<MachineTacticRunRequest<'src>, MachineApiRequestError> {
    let doc = parse_request_body(source, MachineApiErrorKind::InvalidTacticRunRequest)?;
    let members = validate_run_top_level(doc.root())?;

    let session_id = SessionId::parse(required_string_member(
        members,
        "session_id",
        MachineApiErrorKind::InvalidTacticRunRequest,
        &JsonPath::root().field("session_id"),
    )?)
    .map_err(|_| grammar_error("session_id", MachineApiErrorKind::InvalidTacticRunRequest))?;
    let snapshot_id = SnapshotId::parse(required_string_member(
        members,
        "snapshot_id",
        MachineApiErrorKind::InvalidTacticRunRequest,
        &JsonPath::root().field("snapshot_id"),
    )?)
    .map_err(|_| grammar_error("snapshot_id", MachineApiErrorKind::InvalidTacticRunRequest))?;
    let state_fingerprint = HashString::parse(required_string_member(
        members,
        "state_fingerprint",
        MachineApiErrorKind::InvalidTacticRunRequest,
        &JsonPath::root().field("state_fingerprint"),
    )?)
    .map_err(|_| {
        grammar_error(
            "state_fingerprint",
            MachineApiErrorKind::InvalidTacticRunRequest,
        )
    })?
    .digest();
    let goal_id = parse_goal_id_wire(required_string_member(
        members,
        "goal_id",
        MachineApiErrorKind::InvalidTacticRunRequest,
        &JsonPath::root().field("goal_id"),
    )?)
    .map_err(|_| grammar_error("goal_id", MachineApiErrorKind::InvalidTacticRunRequest))?;

    let candidate_value = required_value_member(
        members,
        "candidate",
        MachineApiErrorKind::InvalidTacticRunRequest,
        &JsonPath::root().field("candidate"),
    )?;
    if candidate_value.kind() == JsonValueKind::Null {
        return Err(null_field(
            "candidate",
            MachineApiErrorKind::InvalidTacticRunRequest,
            &JsonPath::root().field("candidate"),
        ));
    }
    if candidate_value.kind() != JsonValueKind::Object {
        return Err(type_mismatch(
            "candidate",
            JsonFieldType::Object,
            candidate_value,
            MachineApiErrorKind::InvalidTacticRunRequest,
            &JsonPath::root().field("candidate"),
        ));
    }
    let candidate = delayed_json_payload(candidate_value);

    let budget_value = required_value_member(
        members,
        "deterministic_budget",
        MachineApiErrorKind::InvalidBudget,
        &JsonPath::root().field("deterministic_budget"),
    )?;
    let deterministic_budget = parse_deterministic_budget(
        budget_value,
        &JsonPath::root().field("deterministic_budget"),
    )?;

    let scheduler_limits = match member_value(members, "scheduler_limits") {
        Some(value) => {
            parse_run_scheduler_limits(value, &JsonPath::root().field("scheduler_limits"))?
        }
        None => MachineRunSchedulerLimits::default(),
    };

    Ok(MachineTacticRunRequest {
        session_id,
        snapshot_id,
        state_fingerprint,
        goal_id,
        candidate,
        deterministic_budget,
        scheduler_limits,
    })
}

pub fn parse_machine_tactic_batch_request<'src>(
    source: &'src str,
) -> Result<MachineTacticBatchRequest<'src>, MachineApiRequestError> {
    let doc = parse_request_body(source, MachineApiErrorKind::InvalidBatchPolicy)?;
    let members = validate_batch_top_level(doc.root())?;

    let session_id = SessionId::parse(required_string_member(
        members,
        "session_id",
        MachineApiErrorKind::InvalidBatchPolicy,
        &JsonPath::root().field("session_id"),
    )?)
    .map_err(|_| grammar_error("session_id", MachineApiErrorKind::InvalidBatchPolicy))?;
    let snapshot_id = SnapshotId::parse(required_string_member(
        members,
        "snapshot_id",
        MachineApiErrorKind::InvalidBatchPolicy,
        &JsonPath::root().field("snapshot_id"),
    )?)
    .map_err(|_| grammar_error("snapshot_id", MachineApiErrorKind::InvalidBatchPolicy))?;
    let state_fingerprint = HashString::parse(required_string_member(
        members,
        "state_fingerprint",
        MachineApiErrorKind::InvalidBatchPolicy,
        &JsonPath::root().field("state_fingerprint"),
    )?)
    .map_err(|_| grammar_error("state_fingerprint", MachineApiErrorKind::InvalidBatchPolicy))?
    .digest();
    let goal_id = parse_goal_id_wire(required_string_member(
        members,
        "goal_id",
        MachineApiErrorKind::InvalidBatchPolicy,
        &JsonPath::root().field("goal_id"),
    )?)
    .map_err(|_| grammar_error("goal_id", MachineApiErrorKind::InvalidBatchPolicy))?;

    let candidates = parse_batch_candidates(
        required_value_member(
            members,
            "candidates",
            MachineApiErrorKind::InvalidBatchPolicy,
            &JsonPath::root().field("candidates"),
        )?,
        &JsonPath::root().field("candidates"),
    )?;

    let budget_value = required_value_member(
        members,
        "deterministic_budget",
        MachineApiErrorKind::InvalidBudget,
        &JsonPath::root().field("deterministic_budget"),
    )?;
    let deterministic_budget = parse_deterministic_budget(
        budget_value,
        &JsonPath::root().field("deterministic_budget"),
    )?;

    let batch_policy = parse_batch_policy(
        required_value_member(
            members,
            "batch_policy",
            MachineApiErrorKind::InvalidBatchPolicy,
            &JsonPath::root().field("batch_policy"),
        )?,
        &JsonPath::root().field("batch_policy"),
    )?;

    let scheduler_limits = match member_value(members, "scheduler_limits") {
        Some(value) => {
            parse_batch_scheduler_limits(value, &JsonPath::root().field("scheduler_limits"))?
        }
        None => MachineBatchSchedulerLimits::default(),
    };

    Ok(MachineTacticBatchRequest {
        session_id,
        snapshot_id,
        state_fingerprint,
        goal_id,
        candidates,
        deterministic_budget,
        batch_policy,
        scheduler_limits,
    })
}

fn run_machine_tactic_request_parsed(
    session: &mut MachineProofSession,
    request: MachineTacticRunRequest<'_>,
) -> Result<MachineTacticRunResponse, Box<MachineTacticRunError>> {
    if session.snapshots.session_id() != &session.session_id {
        return Err(plain_error(
            MachineApiErrorKind::InvalidMachineProofState,
            MachineApiDiagnosticPhase::SnapshotLookup,
            "session snapshot store belongs to a different session",
            RunErrorCorrelation::default(),
        ));
    }

    let scheduler_observation = RunSchedulerObservationContext::start();
    let context = MachineSnapshotMaterializationContext {
        session_id: &session.session_id,
        display_scope: &session.machine_display_render_scope,
        callable_interface_table: &session.machine_surface_callable_interface_table,
    };
    let input_state = {
        let entry = session
            .snapshots
            .lookup_checked(&context, request.snapshot_id, request.state_fingerprint)
            .map_err(snapshot_lookup_error)?;
        if !entry
            .materialized_view_payload
            .open_goals
            .contains(&request.goal_id)
        {
            return Err(plain_error_with_goal(
                MachineApiErrorKind::GoalNotOpen,
                MachineApiDiagnosticPhase::SnapshotLookup,
                format!("goal {} is not open", request.goal_id.0),
                request.goal_id,
                RunErrorCorrelation::default(),
            ));
        }
        entry.executable_state_payload.clone()
    };

    let deterministic_budget_hash = tactic_budget_hash(request.deterministic_budget);
    if let Some(kind) = scheduler_observation.observe(request.scheduler_limits) {
        return Ok(scheduler_stop(
            request.state_fingerprint,
            deterministic_budget_hash,
            kind,
        ));
    }

    let candidate_tactic_kind = candidate_tactic_kind_for_diagnostic(request.candidate.raw);
    let candidate = parse_candidate_payload(
        request.candidate.raw,
        &session.root.universe_params,
        candidate_tactic_kind,
    )
    .map_err(|error| {
        candidate_request_error(
            error,
            request.state_fingerprint,
            deterministic_budget_hash,
            request.goal_id,
            candidate_tactic_kind,
        )
    })?;
    if let Some(kind) = scheduler_observation.observe(request.scheduler_limits) {
        return Ok(scheduler_stop(
            request.state_fingerprint,
            deterministic_budget_hash,
            kind,
        ));
    }

    let validated =
        phase4_validate_machine_tactic_candidate(request.goal_id, candidate).map_err(|error| {
            adapter_error(
                error,
                request.state_fingerprint,
                Some(deterministic_budget_hash),
            )
        })?;
    if let Some(kind) = scheduler_observation.observe(request.scheduler_limits) {
        return Ok(scheduler_stop(
            request.state_fingerprint,
            deterministic_budget_hash,
            kind,
        ));
    }
    let tactic_kind = validated.tactic_kind;
    let candidate_hash = validated.candidate_hash;
    let run = phase4_run_machine_tactic_with_budget(
        &input_state,
        validated.tactic,
        request.deterministic_budget,
    )
    .map_err(|error| adapter_error(error, request.state_fingerprint, None))?;
    if let Some(kind) = scheduler_observation.observe(request.scheduler_limits) {
        return Ok(scheduler_stop(
            request.state_fingerprint,
            deterministic_budget_hash,
            kind,
        ));
    }

    let next_snapshot = match session.snapshots.insert_state(&context, run.state) {
        Ok(snapshot) => snapshot,
        Err(MachineSnapshotStoreError::SnapshotQuotaExceeded { .. }) => {
            return Ok(scheduler_stop(
                request.state_fingerprint,
                deterministic_budget_hash,
                MachineSchedulerArtifactKind::ResourceLimitExceeded,
            ));
        }
        Err(error) => {
            return Err(next_snapshot_store_error(
                error,
                request.state_fingerprint,
                candidate_hash,
                deterministic_budget_hash,
                request.goal_id,
                tactic_kind,
            ));
        }
    };
    let new_goals =
        new_goals_in_next_snapshot_order(&next_snapshot.open_goals, &run.delta.added_goals)
            .ok_or_else(|| {
                next_snapshot_invariant_error(
                    "proof delta added_goals are not present in the next snapshot",
                    request.state_fingerprint,
                    candidate_hash,
                    deterministic_budget_hash,
                    request.goal_id,
                    tactic_kind,
                )
            })?;
    let kind = if new_goals.is_empty() {
        MachineTacticRunResultKind::Closed
    } else {
        MachineTacticRunResultKind::Expanded
    };

    Ok(MachineApiResponseEnvelope::Ok(
        crate::MachineApiOkResponse {
            status: MachineApiResponseStatus::Success,
            endpoint_fields: MachineTacticRunSuccessFields {
                result: MachineTacticRunSuccessResult {
                    kind,
                    previous_state_fingerprint: request.state_fingerprint,
                    candidate_hash,
                    deterministic_budget_hash: run.deterministic_budget_hash,
                    next_snapshot_id: next_snapshot.snapshot_id,
                    next_state_fingerprint: next_snapshot.state_fingerprint,
                    closed_goals: vec![run.delta.assigned_goal],
                    new_goals,
                    delta: MachineTacticRunDeltaSummary {
                        proof_delta_hash: run.proof_delta_hash,
                        assigned_goal: run.delta.assigned_goal,
                        assigned_proof_expr_hash: run.delta.proof_expr_hash,
                    },
                },
            },
        },
    ))
}

fn run_machine_tactic_batch_request_parsed(
    session: &mut MachineProofSession,
    request: MachineTacticBatchRequest<'_>,
) -> Result<MachineTacticBatchResponse, Box<MachineTacticBatchError>> {
    if session.snapshots.session_id() != &session.session_id {
        return Err(batch_plain_error(
            MachineApiErrorKind::InvalidMachineProofState,
            MachineApiDiagnosticPhase::SnapshotLookup,
            "session snapshot store belongs to a different session",
            None,
        ));
    }

    let mut scheduler_observation = BatchSchedulerObservationContext::start();
    let context = MachineSnapshotMaterializationContext {
        session_id: &session.session_id,
        display_scope: &session.machine_display_render_scope,
        callable_interface_table: &session.machine_surface_callable_interface_table,
    };
    let input_state = {
        let entry = session
            .snapshots
            .lookup_checked(&context, request.snapshot_id, request.state_fingerprint)
            .map_err(batch_snapshot_lookup_error)?;
        if !entry
            .materialized_view_payload
            .open_goals
            .contains(&request.goal_id)
        {
            return Err(batch_plain_error(
                MachineApiErrorKind::GoalNotOpen,
                MachineApiDiagnosticPhase::SnapshotLookup,
                format!("goal {} is not open", request.goal_id.0),
                Some(request.goal_id),
            ));
        }
        entry.executable_state_payload.clone()
    };

    let deterministic_budget_hash = tactic_budget_hash(request.deterministic_budget);
    let candidate_count = request.candidates.len();
    let mut results = Vec::new();
    let mut success_count = 0u32;
    let mut failure_count = 0u32;
    let mut evaluated_count = 0u32;

    if let Some(stop) = scheduler_observation.observe(request.scheduler_limits) {
        return Ok(batch_scheduler_stop(
            request.state_fingerprint,
            deterministic_budget_hash,
            results,
            success_count,
            failure_count,
            stop,
        ));
    }

    for (index, item) in request.candidates.into_iter().enumerate() {
        if batch_policy_stop(
            evaluated_count,
            success_count,
            failure_count,
            candidate_count,
            request.batch_policy,
        ) {
            break;
        }

        scheduler_observation.begin_candidate();
        if let Some(stop) = scheduler_observation.observe(request.scheduler_limits) {
            return Ok(batch_scheduler_stop(
                request.state_fingerprint,
                deterministic_budget_hash,
                results,
                success_count,
                failure_count,
                stop,
            ));
        }

        let candidate_id = item.candidate_id;
        let candidate_tactic_kind = candidate_tactic_kind_for_diagnostic(item.candidate.raw);
        let candidate_path = JsonPath::root()
            .field("candidates")
            .index(index)
            .field("candidate");
        let candidate = match parse_candidate_payload_at(
            item.candidate.raw,
            &session.root.universe_params,
            candidate_tactic_kind,
            &candidate_path,
        ) {
            Ok(candidate) => candidate,
            Err(error) => {
                results.push(batch_candidate_request_error_item(
                    candidate_id,
                    error,
                    request.goal_id,
                    candidate_tactic_kind,
                ));
                evaluated_count += 1;
                failure_count += 1;
                if batch_policy_stop(
                    evaluated_count,
                    success_count,
                    failure_count,
                    candidate_count,
                    request.batch_policy,
                ) {
                    break;
                }
                if let Some(stop) = scheduler_observation.observe(request.scheduler_limits) {
                    return Ok(batch_scheduler_stop(
                        request.state_fingerprint,
                        deterministic_budget_hash,
                        results,
                        success_count,
                        failure_count,
                        stop,
                    ));
                }
                continue;
            }
        };

        if let Some(stop) = scheduler_observation.observe(request.scheduler_limits) {
            return Ok(batch_scheduler_stop(
                request.state_fingerprint,
                deterministic_budget_hash,
                results,
                success_count,
                failure_count,
                stop,
            ));
        }

        let validated = match phase4_validate_machine_tactic_candidate(request.goal_id, candidate) {
            Ok(validated) => validated,
            Err(error) => {
                results.push(batch_adapter_error_item(candidate_id, error));
                evaluated_count += 1;
                failure_count += 1;
                if batch_policy_stop(
                    evaluated_count,
                    success_count,
                    failure_count,
                    candidate_count,
                    request.batch_policy,
                ) {
                    break;
                }
                if let Some(stop) = scheduler_observation.observe(request.scheduler_limits) {
                    return Ok(batch_scheduler_stop(
                        request.state_fingerprint,
                        deterministic_budget_hash,
                        results,
                        success_count,
                        failure_count,
                        stop,
                    ));
                }
                continue;
            }
        };

        if let Some(stop) = scheduler_observation.observe(request.scheduler_limits) {
            return Ok(batch_scheduler_stop(
                request.state_fingerprint,
                deterministic_budget_hash,
                results,
                success_count,
                failure_count,
                stop,
            ));
        }

        let tactic_kind = validated.tactic_kind;
        let candidate_hash = validated.candidate_hash;
        let run = match phase4_run_machine_tactic_with_budget(
            &input_state,
            validated.tactic,
            request.deterministic_budget,
        ) {
            Ok(run) => run,
            Err(error) => {
                results.push(batch_adapter_error_item(candidate_id, error));
                evaluated_count += 1;
                failure_count += 1;
                if batch_policy_stop(
                    evaluated_count,
                    success_count,
                    failure_count,
                    candidate_count,
                    request.batch_policy,
                ) {
                    break;
                }
                if let Some(stop) = scheduler_observation.observe(request.scheduler_limits) {
                    return Ok(batch_scheduler_stop(
                        request.state_fingerprint,
                        deterministic_budget_hash,
                        results,
                        success_count,
                        failure_count,
                        stop,
                    ));
                }
                continue;
            }
        };

        if let Some(stop) = scheduler_observation.observe(request.scheduler_limits) {
            return Ok(batch_scheduler_stop(
                request.state_fingerprint,
                deterministic_budget_hash,
                results,
                success_count,
                failure_count,
                stop,
            ));
        }

        match session.snapshots.insert_state(&context, run.state) {
            Ok(next_snapshot) => {
                results.push(MachineTacticBatchItemResponse::Success {
                    candidate_id,
                    candidate_hash,
                    next_snapshot_id: next_snapshot.snapshot_id,
                    next_state_fingerprint: next_snapshot.state_fingerprint,
                    proof_delta_hash: run.proof_delta_hash,
                });
                evaluated_count += 1;
                success_count += 1;
            }
            Err(MachineSnapshotStoreError::SnapshotQuotaExceeded { .. }) => {
                return Ok(batch_scheduler_stop(
                    request.state_fingerprint,
                    deterministic_budget_hash,
                    results,
                    success_count,
                    failure_count,
                    BatchSchedulerStop {
                        kind: MachineSchedulerArtifactKind::ResourceLimitExceeded,
                        scope: MachineSchedulerArtifactScope::Batch,
                    },
                ));
            }
            Err(error) => {
                results.push(batch_next_snapshot_error_item(
                    candidate_id,
                    error,
                    request.goal_id,
                    tactic_kind,
                    candidate_hash,
                ));
                evaluated_count += 1;
                failure_count += 1;
            }
        }

        if batch_policy_stop(
            evaluated_count,
            success_count,
            failure_count,
            candidate_count,
            request.batch_policy,
        ) {
            break;
        }
        if let Some(stop) = scheduler_observation.observe(request.scheduler_limits) {
            return Ok(batch_scheduler_stop(
                request.state_fingerprint,
                deterministic_budget_hash,
                results,
                success_count,
                failure_count,
                stop,
            ));
        }
    }

    Ok(MachineApiResponseEnvelope::Ok(
        crate::MachineApiOkResponse {
            status: MachineApiResponseStatus::Ok,
            endpoint_fields: MachineTacticBatchOkFields {
                previous_state_fingerprint: request.state_fingerprint,
                deterministic_budget_hash,
                results,
                success_count,
                failure_count,
            },
        },
    ))
}

fn parse_candidate_payload(
    raw: &str,
    universe_params: &[String],
    tactic_kind: Option<MachineApiTacticKind>,
) -> Result<MachineTacticCandidate, MachineApiRequestError> {
    parse_candidate_payload_at(
        raw,
        universe_params,
        tactic_kind,
        &JsonPath::root().field("candidate"),
    )
}

fn parse_candidate_payload_at(
    raw: &str,
    universe_params: &[String],
    tactic_kind: Option<MachineApiTacticKind>,
    path: &JsonPath,
) -> Result<MachineTacticCandidate, MachineApiRequestError> {
    let doc = JsonDocument::parse(raw).map_err(|err| {
        MachineApiRequestError::new(
            MachineApiErrorKind::InvalidCandidate,
            path.clone(),
            MachineApiRequestErrorReason::JsonParse {
                offset: err.offset,
                kind: err.kind,
            },
        )
    })?;
    parse_candidate_value(doc.root(), universe_params, tactic_kind, path)
}

fn parse_candidate_value(
    value: &JsonValue<'_>,
    universe_params: &[String],
    tactic_kind: Option<MachineApiTacticKind>,
    path: &JsonPath,
) -> Result<MachineTacticCandidate, MachineApiRequestError> {
    let members = value.object_members().ok_or_else(|| {
        MachineApiRequestError::new(
            MachineApiErrorKind::InvalidCandidate,
            path.clone(),
            MachineApiRequestErrorReason::ExpectedObject {
                actual: value.kind(),
            },
        )
    })?;
    reject_duplicate_keys(members, MachineApiErrorKind::InvalidCandidate, path)?;
    let kind_value = member_value(members, "kind").ok_or_else(|| {
        MachineApiRequestError::new(
            MachineApiErrorKind::InvalidCandidate,
            path.field("kind"),
            MachineApiRequestErrorReason::MissingField { field: "kind" },
        )
    })?;
    let kind = string_value(
        kind_value,
        "kind",
        MachineApiErrorKind::InvalidCandidate,
        &path.field("kind"),
    )?;

    match kind {
        "exact" => {
            let object = validate_json_object(
                value,
                ObjectSchema::new(MachineApiErrorKind::InvalidCandidate, EXACT_FIELDS),
                path,
            )?;
            Ok(MachineTacticCandidate::Exact {
                term: parse_raw_machine_term(
                    required_object_field(&object, "term"),
                    &path.field("term"),
                )?,
            })
        }
        "intro" => {
            let object = validate_json_object(
                value,
                ObjectSchema::new(MachineApiErrorKind::InvalidCandidate, INTRO_FIELDS),
                path,
            )?;
            let name = required_schema_string(&object, "name");
            validate_machine_local_name(name, "name", &path.field("name"))?;
            Ok(MachineTacticCandidate::Intro {
                name: name.to_owned(),
            })
        }
        "apply" => {
            let object = validate_json_object(
                value,
                ObjectSchema::new(MachineApiErrorKind::InvalidCandidate, APPLY_FIELDS),
                path,
            )?;
            Ok(MachineTacticCandidate::Apply {
                head: parse_tactic_head(
                    required_object_field(&object, "head"),
                    &path.field("head"),
                )?,
                universe_args: parse_level_array(
                    required_array_field(&object, "universe_args"),
                    universe_params,
                    &path.field("universe_args"),
                )?,
                args: parse_apply_arg_array(
                    required_array_field(&object, "args"),
                    &path.field("args"),
                )?,
            })
        }
        "rw" => {
            let object = validate_json_object(
                value,
                ObjectSchema::new(MachineApiErrorKind::InvalidCandidate, REWRITE_FIELDS),
                path,
            )?;
            Ok(MachineTacticCandidate::Rewrite {
                rule: parse_rewrite_rule(
                    required_object_field(&object, "rule"),
                    universe_params,
                    &path.field("rule"),
                )?,
                direction: parse_rewrite_direction(
                    required_schema_string(&object, "direction"),
                    "direction",
                    &path.field("direction"),
                )?,
                site: parse_rewrite_site(
                    required_schema_string(&object, "site"),
                    "site",
                    &path.field("site"),
                )?,
            })
        }
        "simp-lite" => {
            let object = validate_json_object(
                value,
                ObjectSchema::new(MachineApiErrorKind::InvalidCandidate, SIMP_LITE_FIELDS),
                path,
            )?;
            Ok(MachineTacticCandidate::SimpLite {
                rules: parse_simp_rule_array(
                    required_array_field(&object, "rules"),
                    &path.field("rules"),
                )?,
            })
        }
        "induction-nat" => {
            let object = validate_json_object(
                value,
                ObjectSchema::new(MachineApiErrorKind::InvalidCandidate, INDUCTION_NAT_FIELDS),
                path,
            )?;
            let local_name = required_schema_string(&object, "local_name");
            validate_machine_local_name(local_name, "local_name", &path.field("local_name"))?;
            Ok(MachineTacticCandidate::InductionNat {
                local_name: local_name.to_owned(),
            })
        }
        _ => Err(invalid_string_literal(
            "kind",
            tactic_kind,
            MachineApiErrorKind::InvalidCandidate,
            &path.field("kind"),
        )),
    }
}

fn parse_raw_machine_term(
    value: &JsonValue<'_>,
    path: &JsonPath,
) -> Result<RawMachineTerm, MachineApiRequestError> {
    let object = validate_json_object(
        value,
        ObjectSchema::new(
            MachineApiErrorKind::InvalidCandidate,
            RAW_MACHINE_TERM_FIELDS,
        ),
        path,
    )?;
    Ok(RawMachineTerm::new(required_schema_string(
        &object, "source",
    )))
}

fn parse_tactic_head(
    value: &JsonValue<'_>,
    path: &JsonPath,
) -> Result<TacticHead, MachineApiRequestError> {
    let members = value.object_members().ok_or_else(|| {
        MachineApiRequestError::new(
            MachineApiErrorKind::InvalidCandidate,
            path.clone(),
            MachineApiRequestErrorReason::ExpectedObject {
                actual: value.kind(),
            },
        )
    })?;
    reject_duplicate_keys(members, MachineApiErrorKind::InvalidCandidate, path)?;
    if members.len() != 1 {
        return Err(MachineApiRequestError::new(
            MachineApiErrorKind::InvalidCandidate,
            path.clone(),
            MachineApiRequestErrorReason::MissingField { field: "head" },
        ));
    }
    let member = &members[0];
    match member.key() {
        "imported" => {
            let object = validate_json_object(
                member.value(),
                ObjectSchema::new(MachineApiErrorKind::InvalidCandidate, IMPORTED_HEAD_FIELDS),
                &path.field("imported"),
            )?;
            Ok(TacticHead::Imported {
                name: parse_renderable_name_field(
                    &object,
                    "name",
                    &path.field("imported").field("name"),
                )?,
                decl_interface_hash: parse_hash_field(
                    &object,
                    "decl_interface_hash",
                    &path.field("imported").field("decl_interface_hash"),
                )?,
            })
        }
        "current_module" => {
            let object = validate_json_object(
                member.value(),
                ObjectSchema::new(MachineApiErrorKind::InvalidCandidate, CURRENT_HEAD_FIELDS),
                &path.field("current_module"),
            )?;
            Ok(TacticHead::CurrentModule {
                name: parse_renderable_name_field(
                    &object,
                    "name",
                    &path.field("current_module").field("name"),
                )?,
                decl_interface_hash: parse_hash_field(
                    &object,
                    "decl_interface_hash",
                    &path.field("current_module").field("decl_interface_hash"),
                )?,
            })
        }
        "local" => {
            let object = validate_json_object(
                member.value(),
                ObjectSchema::new(MachineApiErrorKind::InvalidCandidate, LOCAL_HEAD_FIELDS),
                &path.field("local"),
            )?;
            let name = required_schema_string(&object, "name");
            validate_machine_local_name(name, "name", &path.field("local").field("name"))?;
            Ok(TacticHead::Local {
                name: name.to_owned(),
            })
        }
        other => Err(MachineApiRequestError::new(
            MachineApiErrorKind::InvalidCandidate,
            path.field(other),
            MachineApiRequestErrorReason::UnknownField {
                field: other.to_owned(),
            },
        )),
    }
}

fn parse_rewrite_rule(
    value: &JsonValue<'_>,
    universe_params: &[String],
    path: &JsonPath,
) -> Result<CandidateRewriteRuleRef, MachineApiRequestError> {
    let object = validate_json_object(
        value,
        ObjectSchema::new(MachineApiErrorKind::InvalidCandidate, REWRITE_RULE_FIELDS),
        path,
    )?;
    Ok(CandidateRewriteRuleRef {
        head: parse_tactic_head(required_object_field(&object, "head"), &path.field("head"))?,
        universe_args: parse_level_array(
            required_array_field(&object, "universe_args"),
            universe_params,
            &path.field("universe_args"),
        )?,
        args: parse_apply_arg_array(required_array_field(&object, "args"), &path.field("args"))?,
    })
}

fn parse_apply_arg_array(
    elements: &[JsonValue<'_>],
    path: &JsonPath,
) -> Result<Vec<CandidateApplyArg>, MachineApiRequestError> {
    elements
        .iter()
        .enumerate()
        .map(|(index, value)| parse_apply_arg(value, &path.index(index)))
        .collect()
}

fn parse_apply_arg(
    value: &JsonValue<'_>,
    path: &JsonPath,
) -> Result<CandidateApplyArg, MachineApiRequestError> {
    let members = value.object_members().ok_or_else(|| {
        MachineApiRequestError::new(
            MachineApiErrorKind::InvalidCandidate,
            path.clone(),
            MachineApiRequestErrorReason::ExpectedObject {
                actual: value.kind(),
            },
        )
    })?;
    reject_duplicate_keys(members, MachineApiErrorKind::InvalidCandidate, path)?;
    let mode_value = member_value(members, "mode").ok_or_else(|| {
        MachineApiRequestError::new(
            MachineApiErrorKind::InvalidCandidate,
            path.field("mode"),
            MachineApiRequestErrorReason::MissingField { field: "mode" },
        )
    })?;
    let mode = string_value(
        mode_value,
        "mode",
        MachineApiErrorKind::InvalidCandidate,
        &path.field("mode"),
    )?;

    match mode {
        "term" => {
            let object = validate_json_object(
                value,
                ObjectSchema::new(MachineApiErrorKind::InvalidCandidate, ARG_TERM_FIELDS),
                path,
            )?;
            Ok(CandidateApplyArg::Term(parse_raw_machine_term(
                required_object_field(&object, "term"),
                &path.field("term"),
            )?))
        }
        "subgoal" => {
            let object = validate_json_object(
                value,
                ObjectSchema::new(MachineApiErrorKind::InvalidCandidate, ARG_SUBGOAL_FIELDS),
                path,
            )?;
            let name_hint = object
                .field("name_hint")
                .expect("schema checked required name_hint");
            let name_hint = match name_hint.kind() {
                JsonValueKind::Null => None,
                JsonValueKind::String => {
                    let name = name_hint
                        .string_value()
                        .expect("kind checked string name_hint");
                    validate_machine_local_name(name, "name_hint", &path.field("name_hint"))?;
                    Some(name.to_owned())
                }
                _ => unreachable!("schema checked nullable string name_hint"),
            };
            Ok(CandidateApplyArg::Subgoal { name_hint })
        }
        "infer_from_target" => {
            validate_json_object(
                value,
                ObjectSchema::new(MachineApiErrorKind::InvalidCandidate, ARG_INFER_FIELDS),
                path,
            )?;
            Ok(CandidateApplyArg::InferFromTarget)
        }
        _ => Err(invalid_string_literal(
            "mode",
            None,
            MachineApiErrorKind::InvalidCandidate,
            &path.field("mode"),
        )),
    }
}

fn parse_simp_rule_array(
    elements: &[JsonValue<'_>],
    path: &JsonPath,
) -> Result<Vec<SimpRuleRef>, MachineApiRequestError> {
    elements
        .iter()
        .enumerate()
        .map(|(index, value)| parse_simp_rule(value, &path.index(index)))
        .collect()
}

fn parse_simp_rule(
    value: &JsonValue<'_>,
    path: &JsonPath,
) -> Result<SimpRuleRef, MachineApiRequestError> {
    let object = validate_json_object(
        value,
        ObjectSchema::new(MachineApiErrorKind::InvalidCandidate, SIMP_RULE_FIELDS),
        path,
    )?;
    Ok(SimpRuleRef {
        name: parse_renderable_name_field(&object, "name", &path.field("name"))?,
        decl_interface_hash: parse_hash_field(
            &object,
            "decl_interface_hash",
            &path.field("decl_interface_hash"),
        )?,
        direction: parse_rewrite_direction(
            required_schema_string(&object, "direction"),
            "direction",
            &path.field("direction"),
        )?,
    })
}

fn parse_level_array(
    elements: &[JsonValue<'_>],
    universe_params: &[String],
    path: &JsonPath,
) -> Result<Vec<Level>, MachineApiRequestError> {
    elements
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let source = string_value(
                value,
                "level",
                MachineApiErrorKind::InvalidCandidate,
                &path.index(index),
            )?;
            parse_level_wire(source, universe_params).map_err(|_| {
                invalid_string_literal(
                    "level",
                    None,
                    MachineApiErrorKind::InvalidCandidate,
                    &path.index(index),
                )
            })
        })
        .collect()
}

fn parse_level_wire(source: &str, universe_params: &[String]) -> Result<Level, ()> {
    if source.is_empty()
        || source.starts_with(' ')
        || source.ends_with(' ')
        || source.contains("  ")
    {
        return Err(());
    }
    let tokens = source.split(' ').collect::<Vec<_>>();
    let mut cursor = 0;
    let level = parse_level_tokens(&tokens, &mut cursor, universe_params)?;
    if cursor != tokens.len() {
        return Err(());
    }
    if render_level_wire(&level) != source {
        return Err(());
    }
    Ok(level)
}

fn parse_level_tokens(
    tokens: &[&str],
    cursor: &mut usize,
    universe_params: &[String],
) -> Result<Level, ()> {
    let token = tokens.get(*cursor).copied().ok_or(())?;
    *cursor += 1;
    match token {
        "succ" => Ok(Level::Succ(Box::new(parse_level_tokens(
            tokens,
            cursor,
            universe_params,
        )?))),
        "max" => {
            let lhs = parse_level_tokens(tokens, cursor, universe_params)?;
            let rhs = parse_level_tokens(tokens, cursor, universe_params)?;
            Ok(Level::Max(Box::new(lhs), Box::new(rhs)))
        }
        "imax" => {
            let lhs = parse_level_tokens(tokens, cursor, universe_params)?;
            let rhs = parse_level_tokens(tokens, cursor, universe_params)?;
            Ok(Level::IMax(Box::new(lhs), Box::new(rhs)))
        }
        _ if is_canonical_decimal(token) => decimal_level(token),
        _ if is_machine_universe_param_name(token)
            && universe_params.iter().any(|param| param == token) =>
        {
            Ok(Level::Param(token.to_owned()))
        }
        _ => Err(()),
    }
}

fn decimal_level(token: &str) -> Result<Level, ()> {
    let value = token.parse::<u64>().map_err(|_| ())?;
    if value > MAX_NUMERIC_UNIVERSE_LEVEL {
        return Err(());
    }
    let mut level = Level::Zero;
    for _ in 0..value {
        level = Level::Succ(Box::new(level));
    }
    Ok(level)
}

fn is_canonical_decimal(token: &str) -> bool {
    if token == "0" {
        return true;
    }
    token
        .as_bytes()
        .first()
        .is_some_and(|byte| matches!(byte, b'1'..=b'9'))
        && token.as_bytes()[1..].iter().all(u8::is_ascii_digit)
}

fn render_level_wire(level: &Level) -> String {
    if let Some(value) = level_as_nat(level) {
        return value.to_string();
    }
    match level {
        Level::Zero => "0".to_owned(),
        Level::Succ(inner) => format!("succ {}", render_level_wire(inner)),
        Level::Max(lhs, rhs) => {
            format!("max {} {}", render_level_wire(lhs), render_level_wire(rhs))
        }
        Level::IMax(lhs, rhs) => {
            format!("imax {} {}", render_level_wire(lhs), render_level_wire(rhs))
        }
        Level::Param(name) => name.clone(),
    }
}

fn level_as_nat(level: &Level) -> Option<u64> {
    match level {
        Level::Zero => Some(0),
        Level::Succ(inner) => Some(level_as_nat(inner)? + 1),
        _ => None,
    }
}

fn parse_rewrite_direction(
    value: &str,
    field: &'static str,
    path: &JsonPath,
) -> Result<RewriteDirection, MachineApiRequestError> {
    match value {
        "forward" => Ok(RewriteDirection::Forward),
        "backward" => Ok(RewriteDirection::Backward),
        _ => Err(invalid_string_literal(
            field,
            None,
            MachineApiErrorKind::InvalidCandidate,
            path,
        )),
    }
}

fn parse_rewrite_site(
    value: &str,
    field: &'static str,
    path: &JsonPath,
) -> Result<RewriteSite, MachineApiRequestError> {
    match value {
        "eq_target_left" => Ok(RewriteSite::EqTargetLeft),
        "eq_target_right" => Ok(RewriteSite::EqTargetRight),
        _ => Err(invalid_string_literal(
            field,
            None,
            MachineApiErrorKind::InvalidCandidate,
            path,
        )),
    }
}

fn validate_run_top_level<'value, 'src>(
    root: &'value JsonValue<'src>,
) -> Result<&'value [JsonMember<'src>], MachineApiRequestError> {
    let members = root.object_members().ok_or_else(|| {
        MachineApiRequestError::new(
            MachineApiErrorKind::InvalidTacticRunRequest,
            JsonPath::root(),
            MachineApiRequestErrorReason::ExpectedObject {
                actual: root.kind(),
            },
        )
    })?;
    reject_duplicate_keys(
        members,
        MachineApiErrorKind::InvalidTacticRunRequest,
        &JsonPath::root(),
    )?;
    let allowed = [
        "session_id",
        "snapshot_id",
        "state_fingerprint",
        "goal_id",
        "candidate",
        "deterministic_budget",
        "scheduler_limits",
    ];
    for member in members {
        if !allowed.contains(&member.key()) {
            return Err(MachineApiRequestError::new(
                MachineApiErrorKind::InvalidTacticRunRequest,
                JsonPath::root().field(member.key()),
                MachineApiRequestErrorReason::UnknownField {
                    field: member.key().to_owned(),
                },
            ));
        }
    }
    Ok(members)
}

fn validate_batch_top_level<'value, 'src>(
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
    reject_duplicate_keys(
        members,
        MachineApiErrorKind::InvalidBatchPolicy,
        &JsonPath::root(),
    )?;
    let allowed = [
        "session_id",
        "snapshot_id",
        "state_fingerprint",
        "goal_id",
        "candidates",
        "deterministic_budget",
        "batch_policy",
        "scheduler_limits",
    ];
    for member in members {
        if !allowed.contains(&member.key()) {
            return Err(MachineApiRequestError::new(
                MachineApiErrorKind::InvalidBatchPolicy,
                JsonPath::root().field(member.key()),
                MachineApiRequestErrorReason::UnknownField {
                    field: member.key().to_owned(),
                },
            ));
        }
    }
    Ok(members)
}

fn parse_batch_candidates<'src>(
    value: &JsonValue<'src>,
    path: &JsonPath,
) -> Result<Vec<MachineTacticBatchCandidateRequest<'src>>, MachineApiRequestError> {
    if value.kind() == JsonValueKind::Null {
        return Err(null_field(
            "candidates",
            MachineApiErrorKind::InvalidBatchPolicy,
            path,
        ));
    }
    let elements = value.array_elements().ok_or_else(|| {
        type_mismatch(
            "candidates",
            JsonFieldType::Array,
            value,
            MachineApiErrorKind::InvalidBatchPolicy,
            path,
        )
    })?;
    if elements.is_empty() || elements.len() > 256 {
        return Err(MachineApiRequestError::new(
            MachineApiErrorKind::InvalidBatchPolicy,
            path.clone(),
            MachineApiRequestErrorReason::TypeMismatch {
                field: "candidates",
                expected: JsonFieldType::Array,
                actual: JsonValueKind::Array,
            },
        ));
    }

    let mut ids = BTreeSet::new();
    let mut candidates = Vec::with_capacity(elements.len());
    for (index, item) in elements.iter().enumerate() {
        let item_path = path.index(index);
        let members = item.object_members().ok_or_else(|| {
            MachineApiRequestError::new(
                MachineApiErrorKind::InvalidBatchPolicy,
                item_path.clone(),
                MachineApiRequestErrorReason::ExpectedObject {
                    actual: item.kind(),
                },
            )
        })?;
        reject_duplicate_keys(members, MachineApiErrorKind::InvalidBatchPolicy, &item_path)?;
        for member in members {
            if !matches!(member.key(), "candidate_id" | "candidate") {
                return Err(MachineApiRequestError::new(
                    MachineApiErrorKind::InvalidBatchPolicy,
                    item_path.field(member.key()),
                    MachineApiRequestErrorReason::UnknownField {
                        field: member.key().to_owned(),
                    },
                ));
            }
        }

        let candidate_id = string_value(
            member_value(members, "candidate_id").ok_or_else(|| {
                MachineApiRequestError::new(
                    MachineApiErrorKind::InvalidBatchPolicy,
                    item_path.field("candidate_id"),
                    MachineApiRequestErrorReason::MissingField {
                        field: "candidate_id",
                    },
                )
            })?,
            "candidate_id",
            MachineApiErrorKind::InvalidBatchPolicy,
            &item_path.field("candidate_id"),
        )?;
        validate_machine_candidate_id(candidate_id, &item_path.field("candidate_id"))?;
        if !ids.insert(candidate_id.to_owned()) {
            return Err(MachineApiRequestError::new(
                MachineApiErrorKind::InvalidBatchPolicy,
                item_path.field("candidate_id"),
                MachineApiRequestErrorReason::DuplicateKey {
                    key: candidate_id.to_owned(),
                },
            ));
        }

        let candidate = member_value(members, "candidate").ok_or_else(|| {
            MachineApiRequestError::new(
                MachineApiErrorKind::InvalidBatchPolicy,
                item_path.field("candidate"),
                MachineApiRequestErrorReason::MissingField { field: "candidate" },
            )
        })?;
        candidates.push(MachineTacticBatchCandidateRequest {
            candidate_id: candidate_id.to_owned(),
            candidate: delayed_json_payload(candidate),
        });
    }
    Ok(candidates)
}

fn parse_deterministic_budget(
    value: &JsonValue<'_>,
    path: &JsonPath,
) -> Result<TacticBudget, MachineApiRequestError> {
    let object = validate_json_object(
        value,
        ObjectSchema::new(MachineApiErrorKind::InvalidBudget, BUDGET_FIELDS),
        path,
    )?;
    Ok(TacticBudget {
        max_tactic_steps: required_u64(&object, "max_tactic_steps"),
        max_whnf_steps: required_u64(&object, "max_whnf_steps"),
        max_conversion_steps: required_u64(&object, "max_conversion_steps"),
        max_rewrite_steps: required_u64(&object, "max_rewrite_steps"),
        max_meta_allocations: required_u64(&object, "max_meta_allocations"),
        max_expr_nodes: required_u64(&object, "max_expr_nodes"),
    })
}

fn parse_batch_policy(
    value: &JsonValue<'_>,
    path: &JsonPath,
) -> Result<MachineTacticBatchPolicy, MachineApiRequestError> {
    let object = validate_json_object(
        value,
        ObjectSchema::new(MachineApiErrorKind::InvalidBatchPolicy, BATCH_POLICY_FIELDS),
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

fn required_batch_policy_u32(
    object: &crate::validation::ValidatedObject<'_, '_>,
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

fn parse_run_scheduler_limits(
    value: &JsonValue<'_>,
    path: &JsonPath,
) -> Result<MachineRunSchedulerLimits, MachineApiRequestError> {
    let object = validate_json_object(
        value,
        ObjectSchema::new(
            MachineApiErrorKind::InvalidSchedulerLimits,
            RUN_SCHEDULER_FIELDS,
        ),
        path,
    )?;
    Ok(MachineRunSchedulerLimits {
        timeout_ms: optional_positive_u64(&object, "timeout_ms", &path.field("timeout_ms"))?,
        max_memory_mb: optional_positive_u64(
            &object,
            "max_memory_mb",
            &path.field("max_memory_mb"),
        )?,
    })
}

fn parse_batch_scheduler_limits(
    value: &JsonValue<'_>,
    path: &JsonPath,
) -> Result<MachineBatchSchedulerLimits, MachineApiRequestError> {
    let object = validate_json_object(
        value,
        ObjectSchema::new(
            MachineApiErrorKind::InvalidSchedulerLimits,
            BATCH_SCHEDULER_FIELDS,
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

#[derive(Clone, Copy)]
struct RunSchedulerObservationContext {
    started_at: Instant,
    memory_usage_bytes: fn() -> Option<u64>,
}

impl RunSchedulerObservationContext {
    fn start() -> Self {
        Self {
            started_at: Instant::now(),
            memory_usage_bytes: current_process_resident_bytes,
        }
    }

    fn observe(self, limits: MachineRunSchedulerLimits) -> Option<MachineSchedulerArtifactKind> {
        observe_run_scheduler_limits(
            limits,
            self.started_at.elapsed(),
            (self.memory_usage_bytes)(),
        )
    }
}

fn observe_run_scheduler_limits(
    limits: MachineRunSchedulerLimits,
    elapsed: Duration,
    memory_usage_bytes: Option<u64>,
) -> Option<MachineSchedulerArtifactKind> {
    if let (Some(limit_mb), Some(usage_bytes)) = (limits.max_memory_mb, memory_usage_bytes) {
        if usage_bytes > memory_limit_bytes(limit_mb) {
            return Some(MachineSchedulerArtifactKind::ResourceLimitExceeded);
        }
    }

    if let Some(timeout_ms) = limits.timeout_ms {
        if elapsed >= Duration::from_millis(timeout_ms) {
            return Some(MachineSchedulerArtifactKind::Timeout);
        }
    }

    None
}

#[derive(Clone, Copy)]
struct BatchSchedulerObservationContext {
    batch_started_at: Instant,
    candidate_started_at: Option<Instant>,
    memory_usage_bytes: fn() -> Option<u64>,
}

impl BatchSchedulerObservationContext {
    fn start() -> Self {
        Self {
            batch_started_at: Instant::now(),
            candidate_started_at: None,
            memory_usage_bytes: current_process_resident_bytes,
        }
    }

    fn begin_candidate(&mut self) {
        self.candidate_started_at = Some(Instant::now());
    }

    fn observe(self, limits: MachineBatchSchedulerLimits) -> Option<BatchSchedulerStop> {
        observe_batch_scheduler_limits(
            limits,
            self.batch_started_at.elapsed(),
            self.candidate_started_at
                .map(|started_at| started_at.elapsed()),
            (self.memory_usage_bytes)(),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BatchSchedulerStop {
    kind: MachineSchedulerArtifactKind,
    scope: MachineSchedulerArtifactScope,
}

fn observe_batch_scheduler_limits(
    limits: MachineBatchSchedulerLimits,
    batch_elapsed: Duration,
    candidate_elapsed: Option<Duration>,
    memory_usage_bytes: Option<u64>,
) -> Option<BatchSchedulerStop> {
    if let (Some(limit_mb), Some(usage_bytes)) = (limits.max_memory_mb, memory_usage_bytes) {
        if usage_bytes > memory_limit_bytes(limit_mb) {
            return Some(BatchSchedulerStop {
                kind: MachineSchedulerArtifactKind::ResourceLimitExceeded,
                scope: MachineSchedulerArtifactScope::Batch,
            });
        }
    }

    if let Some(timeout_ms) = limits.batch_timeout_ms {
        if batch_elapsed >= Duration::from_millis(timeout_ms) {
            return Some(BatchSchedulerStop {
                kind: MachineSchedulerArtifactKind::Timeout,
                scope: MachineSchedulerArtifactScope::Batch,
            });
        }
    }

    if let (Some(timeout_ms), Some(elapsed)) = (limits.per_candidate_timeout_ms, candidate_elapsed)
    {
        if elapsed >= Duration::from_millis(timeout_ms) {
            return Some(BatchSchedulerStop {
                kind: MachineSchedulerArtifactKind::Timeout,
                scope: MachineSchedulerArtifactScope::Candidate,
            });
        }
    }

    None
}

fn memory_limit_bytes(max_memory_mb: u64) -> u64 {
    max_memory_mb.saturating_mul(1024 * 1024)
}

#[cfg(any(target_os = "android", target_os = "linux"))]
fn current_process_resident_bytes() -> Option<u64> {
    let statm = std::fs::read_to_string("/proc/self/statm").ok()?;
    let resident_pages = statm.split_whitespace().nth(1)?.parse::<u64>().ok()?;
    resident_pages.checked_mul(system_page_size_bytes()?)
}

#[cfg(any(target_os = "android", target_os = "linux"))]
fn system_page_size_bytes() -> Option<u64> {
    // SAFETY: sysconf reads a process-global setting and does not dereference caller pointers.
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    u64::try_from(page_size).ok().filter(|bytes| *bytes > 0)
}

#[cfg(any(target_os = "ios", target_os = "macos"))]
fn current_process_resident_bytes() -> Option<u64> {
    let mut info = std::mem::MaybeUninit::<libc::mach_task_basic_info>::uninit();
    let mut count = libc::MACH_TASK_BASIC_INFO_COUNT;
    // SAFETY: task_info writes mach_task_basic_info into the provided buffer for the current
    // task and does not retain caller pointers after returning.
    let rc = unsafe {
        libc::task_info(
            current_mach_task_self(),
            libc::MACH_TASK_BASIC_INFO,
            info.as_mut_ptr().cast::<libc::integer_t>(),
            &mut count,
        )
    };
    if rc != libc::KERN_SUCCESS {
        return None;
    }
    // SAFETY: task_info returned success, so mach_task_basic_info has been initialized.
    let info = unsafe { info.assume_init() };
    Some(info.resident_size)
}

#[cfg(any(target_os = "ios", target_os = "macos"))]
#[allow(deprecated)]
fn current_mach_task_self() -> libc::mach_port_t {
    // SAFETY: mach_task_self reads libSystem's current-task port for this process.
    unsafe { libc::mach_task_self() }
}

#[cfg(not(any(
    target_os = "android",
    target_os = "ios",
    target_os = "linux",
    target_os = "macos"
)))]
fn current_process_resident_bytes() -> Option<u64> {
    None
}

fn new_goals_in_next_snapshot_order(
    next_open_goals: &[npa_tactic::GoalId],
    added_goals: &[npa_tactic::GoalId],
) -> Option<Vec<npa_tactic::GoalId>> {
    let added = added_goals.iter().copied().collect::<BTreeSet<_>>();
    let new_goals = next_open_goals
        .iter()
        .copied()
        .filter(|goal| added.contains(goal))
        .collect::<Vec<_>>();
    (new_goals.len() == added.len()).then_some(new_goals)
}

fn batch_policy_stop(
    evaluated_count: u32,
    success_count: u32,
    failure_count: u32,
    candidate_count: usize,
    policy: MachineTacticBatchPolicy,
) -> bool {
    usize::try_from(evaluated_count).is_ok_and(|count| count >= candidate_count)
        || evaluated_count >= policy.max_evaluated_candidates
        || success_count >= policy.stop_after_successes
        || failure_count >= policy.stop_after_failures
}

fn request_error(error: MachineApiRequestError) -> Box<MachineTacticRunError> {
    plain_error(
        error.kind,
        MachineApiDiagnosticPhase::RequestValidation,
        format!(
            "request validation failed at {}: {:?}",
            json_path_display(&error.path),
            error.reason
        ),
        RunErrorCorrelation::default(),
    )
}

fn batch_request_error(error: MachineApiRequestError) -> Box<MachineTacticBatchError> {
    batch_plain_error(
        error.kind,
        MachineApiDiagnosticPhase::RequestValidation,
        format!(
            "request validation failed at {}: {:?}",
            json_path_display(&error.path),
            error.reason
        ),
        None,
    )
}

fn candidate_request_error(
    error: MachineApiRequestError,
    unchanged_state_fingerprint: Hash,
    deterministic_budget_hash: Hash,
    goal_id: npa_tactic::GoalId,
    tactic_kind: Option<MachineApiTacticKind>,
) -> Box<MachineTacticRunError> {
    plain_error_with_goal(
        MachineApiErrorKind::InvalidCandidate,
        MachineApiDiagnosticPhase::CandidateValidation,
        format!(
            "candidate validation failed at {}: {:?}",
            json_path_display(&error.path),
            error.reason
        ),
        goal_id,
        RunErrorCorrelation {
            unchanged_state_fingerprint: Some(unchanged_state_fingerprint),
            deterministic_budget_hash: Some(deterministic_budget_hash),
            tactic_kind,
            ..RunErrorCorrelation::default()
        },
    )
}

fn snapshot_lookup_error(error: MachineSnapshotLookupError) -> Box<MachineTacticRunError> {
    let kind = match error {
        MachineSnapshotLookupError::UnknownSnapshot { .. } => MachineApiErrorKind::UnknownSnapshot,
        MachineSnapshotLookupError::StateFingerprintMismatch { .. } => {
            MachineApiErrorKind::StateFingerprintMismatch
        }
        MachineSnapshotLookupError::SnapshotIdentityMismatch { .. }
        | MachineSnapshotLookupError::InvalidMachineProofState { .. }
        | MachineSnapshotLookupError::ExecutableStateFingerprintMismatch { .. }
        | MachineSnapshotLookupError::StoredSnapshotViewMismatch { .. } => {
            MachineApiErrorKind::InvalidMachineProofState
        }
    };
    plain_error(
        kind,
        MachineApiDiagnosticPhase::SnapshotLookup,
        format!("snapshot lookup failed: {error:?}"),
        RunErrorCorrelation::default(),
    )
}

fn batch_snapshot_lookup_error(error: MachineSnapshotLookupError) -> Box<MachineTacticBatchError> {
    let kind = match error {
        MachineSnapshotLookupError::UnknownSnapshot { .. } => MachineApiErrorKind::UnknownSnapshot,
        MachineSnapshotLookupError::StateFingerprintMismatch { .. } => {
            MachineApiErrorKind::StateFingerprintMismatch
        }
        MachineSnapshotLookupError::SnapshotIdentityMismatch { .. }
        | MachineSnapshotLookupError::InvalidMachineProofState { .. }
        | MachineSnapshotLookupError::ExecutableStateFingerprintMismatch { .. }
        | MachineSnapshotLookupError::StoredSnapshotViewMismatch { .. } => {
            MachineApiErrorKind::InvalidMachineProofState
        }
    };
    batch_plain_error(
        kind,
        MachineApiDiagnosticPhase::SnapshotLookup,
        format!("snapshot lookup failed: {error:?}"),
        None,
    )
}

fn adapter_error(
    error: Box<Phase4AdapterError>,
    unchanged_state_fingerprint: Hash,
    deterministic_budget_hash_override: Option<Hash>,
) -> Box<MachineTacticRunError> {
    let deterministic_budget_hash = error
        .deterministic_budget_hash
        .or(deterministic_budget_hash_override);
    error_response(
        error.diagnostic,
        Some(unchanged_state_fingerprint),
        error.candidate_hash,
        deterministic_budget_hash,
    )
}

fn next_snapshot_store_error(
    error: MachineSnapshotStoreError,
    unchanged_state_fingerprint: Hash,
    candidate_hash: Hash,
    deterministic_budget_hash: Hash,
    goal_id: npa_tactic::GoalId,
    tactic_kind: MachineApiTacticKind,
) -> Box<MachineTacticRunError> {
    match error {
        MachineSnapshotStoreError::SnapshotQuotaExceeded { .. } => next_snapshot_invariant_error(
            "unexpected snapshot quota error after scheduler handling",
            unchanged_state_fingerprint,
            candidate_hash,
            deterministic_budget_hash,
            goal_id,
            tactic_kind,
        ),
        MachineSnapshotStoreError::Materialization(source) => next_snapshot_materialization_error(
            source,
            unchanged_state_fingerprint,
            candidate_hash,
            deterministic_budget_hash,
            goal_id,
            tactic_kind,
        ),
        MachineSnapshotStoreError::Lookup(source) => next_snapshot_invariant_error(
            format!("next snapshot store consistency check failed: {source:?}"),
            unchanged_state_fingerprint,
            candidate_hash,
            deterministic_budget_hash,
            goal_id,
            tactic_kind,
        ),
    }
}

fn next_snapshot_materialization_error(
    source: MachineSnapshotMaterializationError,
    unchanged_state_fingerprint: Hash,
    candidate_hash: Hash,
    deterministic_budget_hash: Hash,
    goal_id: npa_tactic::GoalId,
    tactic_kind: MachineApiTacticKind,
) -> Box<MachineTacticRunError> {
    next_snapshot_invariant_error(
        format!("next snapshot materialization failed: {source:?}"),
        unchanged_state_fingerprint,
        candidate_hash,
        deterministic_budget_hash,
        goal_id,
        tactic_kind,
    )
}

fn next_snapshot_invariant_error(
    message: impl Into<String>,
    unchanged_state_fingerprint: Hash,
    candidate_hash: Hash,
    deterministic_budget_hash: Hash,
    goal_id: npa_tactic::GoalId,
    tactic_kind: MachineApiTacticKind,
) -> Box<MachineTacticRunError> {
    plain_error_with_goal(
        MachineApiErrorKind::InvalidMachineProofState,
        MachineApiDiagnosticPhase::TacticExecution,
        message,
        goal_id,
        RunErrorCorrelation {
            unchanged_state_fingerprint: Some(unchanged_state_fingerprint),
            candidate_hash: Some(candidate_hash),
            deterministic_budget_hash: Some(deterministic_budget_hash),
            tactic_kind: Some(tactic_kind),
            ..RunErrorCorrelation::default()
        },
    )
}

fn scheduler_stop(
    previous_state_fingerprint: Hash,
    deterministic_budget_hash: Hash,
    kind: MachineSchedulerArtifactKind,
) -> MachineTacticRunResponse {
    MachineApiResponseEnvelope::SchedulerStopped(MachineApiSchedulerResponse {
        status: MachineApiResponseStatus::SchedulerStopped,
        scheduler_artifact: MachineSchedulerArtifact {
            kind,
            scope: MachineSchedulerArtifactScope::Candidate,
            retryable: true,
        },
        endpoint_fields: MachineTacticRunSchedulerFields {
            previous_state_fingerprint,
            deterministic_budget_hash,
        },
    })
}

fn batch_scheduler_stop(
    previous_state_fingerprint: Hash,
    deterministic_budget_hash: Hash,
    results: Vec<MachineTacticBatchItemResponse>,
    success_count: u32,
    failure_count: u32,
    stop: BatchSchedulerStop,
) -> MachineTacticBatchResponse {
    let status = match stop.kind {
        MachineSchedulerArtifactKind::Timeout => MachineApiResponseStatus::PartialTimeout,
        MachineSchedulerArtifactKind::ResourceLimitExceeded => {
            MachineApiResponseStatus::PartialResourceLimit
        }
    };
    MachineApiResponseEnvelope::SchedulerStopped(MachineApiSchedulerResponse {
        status,
        scheduler_artifact: MachineSchedulerArtifact {
            kind: stop.kind,
            scope: stop.scope,
            retryable: true,
        },
        endpoint_fields: MachineTacticBatchSchedulerFields {
            previous_state_fingerprint,
            deterministic_budget_hash,
            completed_prefix_len: results
                .len()
                .try_into()
                .expect("batch protocol caps results at 256"),
            results,
            success_count,
            failure_count,
        },
    })
}

fn batch_candidate_request_error_item(
    candidate_id: String,
    error: MachineApiRequestError,
    goal_id: npa_tactic::GoalId,
    tactic_kind: Option<MachineApiTacticKind>,
) -> MachineTacticBatchItemResponse {
    let message = format!(
        "candidate validation failed at {}: {:?}",
        json_path_display(&error.path),
        error.reason
    );
    batch_plain_item_error(
        candidate_id,
        None,
        error.kind,
        MachineApiDiagnosticPhase::CandidateValidation,
        message,
        goal_id,
        tactic_kind,
    )
}

fn batch_adapter_error_item(
    candidate_id: String,
    error: Box<Phase4AdapterError>,
) -> MachineTacticBatchItemResponse {
    batch_error_item(candidate_id, error.candidate_hash, error.diagnostic)
}

fn batch_next_snapshot_error_item(
    candidate_id: String,
    error: MachineSnapshotStoreError,
    goal_id: npa_tactic::GoalId,
    tactic_kind: MachineApiTacticKind,
    candidate_hash: Hash,
) -> MachineTacticBatchItemResponse {
    let message = match error {
        MachineSnapshotStoreError::SnapshotQuotaExceeded { .. } => {
            "unexpected snapshot quota error after scheduler handling".to_owned()
        }
        MachineSnapshotStoreError::Materialization(source) => {
            format!("next snapshot materialization failed: {source:?}")
        }
        MachineSnapshotStoreError::Lookup(source) => {
            format!("next snapshot store consistency check failed: {source:?}")
        }
    };
    batch_plain_item_error(
        candidate_id,
        Some(candidate_hash),
        MachineApiErrorKind::InvalidMachineProofState,
        MachineApiDiagnosticPhase::TacticExecution,
        message,
        goal_id,
        Some(tactic_kind),
    )
}

fn batch_plain_item_error(
    candidate_id: String,
    candidate_hash: Option<Hash>,
    kind: MachineApiErrorKind,
    phase: MachineApiDiagnosticPhase,
    message: impl Into<String>,
    goal_id: npa_tactic::GoalId,
    tactic_kind: Option<MachineApiTacticKind>,
) -> MachineTacticBatchItemResponse {
    let message = message.into();
    let diagnostic = MachineApiDiagnosticProjection {
        kind,
        phase,
        retryable: false,
        goal_id: Some(goal_id),
        tactic_kind,
        primary_name: None,
        primary_axiom_ref: None,
        expected_hash: None,
        actual_hash: None,
        source_message: message.clone(),
        upstream: Phase5UpstreamDiagnostic::Phase4(MachineTacticDiagnostic::new(
            phase4_kind_for_api_kind(kind),
            message,
        )),
    };
    batch_error_item(candidate_id, candidate_hash, diagnostic)
}

fn batch_error_item(
    candidate_id: String,
    candidate_hash: Option<Hash>,
    diagnostic: MachineApiDiagnosticProjection,
) -> MachineTacticBatchItemResponse {
    let wire = MachineApiErrorWire::from_projection(&diagnostic)
        .expect("batch per-candidate diagnostics must satisfy Phase 5 wire invariants");
    MachineTacticBatchItemResponse::Error {
        candidate_id,
        candidate_hash,
        diagnostic: wire.into(),
    }
}

fn batch_plain_error(
    kind: MachineApiErrorKind,
    phase: MachineApiDiagnosticPhase,
    message: impl Into<String>,
    goal_id: Option<npa_tactic::GoalId>,
) -> Box<MachineTacticBatchError> {
    let message = message.into();
    let diagnostic = MachineApiDiagnosticProjection {
        kind,
        phase,
        retryable: false,
        goal_id,
        tactic_kind: None,
        primary_name: None,
        primary_axiom_ref: None,
        expected_hash: None,
        actual_hash: None,
        source_message: message.clone(),
        upstream: Phase5UpstreamDiagnostic::Phase4(MachineTacticDiagnostic::new(
            phase4_kind_for_api_kind(kind),
            message,
        )),
    };
    let wire = MachineApiErrorWire::from_projection(&diagnostic)
        .expect("batch top-level diagnostics must satisfy Phase 5 wire invariants");
    let response = MachineApiResponseEnvelope::Error(Box::new(MachineApiErrorResponse {
        status: MachineApiResponseStatus::Error,
        error: wire,
        endpoint_fields: (),
    }));
    Box::new(MachineTacticBatchError {
        diagnostic,
        response,
    })
}

fn plain_error(
    kind: MachineApiErrorKind,
    phase: MachineApiDiagnosticPhase,
    message: impl Into<String>,
    correlation: RunErrorCorrelation,
) -> Box<MachineTacticRunError> {
    plain_error_projected(kind, phase, message, correlation)
}

fn plain_error_with_goal(
    kind: MachineApiErrorKind,
    phase: MachineApiDiagnosticPhase,
    message: impl Into<String>,
    goal_id: npa_tactic::GoalId,
    correlation: RunErrorCorrelation,
) -> Box<MachineTacticRunError> {
    plain_error_projected(
        kind,
        phase,
        message,
        RunErrorCorrelation {
            goal_id: Some(goal_id),
            ..correlation
        },
    )
}

fn plain_error_projected(
    kind: MachineApiErrorKind,
    phase: MachineApiDiagnosticPhase,
    message: impl Into<String>,
    correlation: RunErrorCorrelation,
) -> Box<MachineTacticRunError> {
    let message = message.into();
    let diagnostic = MachineApiDiagnosticProjection {
        kind,
        phase,
        retryable: false,
        goal_id: correlation.goal_id,
        tactic_kind: correlation.tactic_kind,
        primary_name: None,
        primary_axiom_ref: None,
        expected_hash: None,
        actual_hash: None,
        source_message: message.clone(),
        upstream: Phase5UpstreamDiagnostic::Phase4(MachineTacticDiagnostic::new(
            phase4_kind_for_api_kind(kind),
            message,
        )),
    };
    error_response(
        diagnostic,
        correlation.unchanged_state_fingerprint,
        correlation.candidate_hash,
        correlation.deterministic_budget_hash,
    )
}

fn error_response(
    diagnostic: MachineApiDiagnosticProjection,
    unchanged_state_fingerprint: Option<Hash>,
    candidate_hash: Option<Hash>,
    deterministic_budget_hash: Option<Hash>,
) -> Box<MachineTacticRunError> {
    let wire = MachineApiErrorWire::from_projection(&diagnostic)
        .expect("run diagnostics must satisfy Phase 5 wire invariants");
    let response = MachineApiResponseEnvelope::Error(Box::new(MachineApiErrorResponse {
        status: MachineApiResponseStatus::Error,
        error: MachineTacticRunErrorObject {
            diagnostic: wire,
            candidate_hash,
            deterministic_budget_hash,
        },
        endpoint_fields: MachineTacticRunErrorFields {
            unchanged_state_fingerprint,
        },
    }));
    Box::new(MachineTacticRunError {
        diagnostic,
        response,
    })
}

fn phase4_kind_for_api_kind(kind: MachineApiErrorKind) -> MachineTacticDiagnosticKind {
    match kind {
        MachineApiErrorKind::GoalNotOpen => MachineTacticDiagnosticKind::UnknownGoal,
        MachineApiErrorKind::InvalidCandidate => MachineTacticDiagnosticKind::InvalidMachineTactic,
        MachineApiErrorKind::InvalidBudget => MachineTacticDiagnosticKind::TacticFuelExhausted {
            kind: npa_tactic::TacticFuelKind::TacticStep,
        },
        _ => MachineTacticDiagnosticKind::InvalidMachineProofState,
    }
}

fn candidate_tactic_kind_for_diagnostic(raw: &str) -> Option<MachineApiTacticKind> {
    let doc = JsonDocument::parse(raw).ok()?;
    let members = doc.root().object_members()?;
    let kind = members
        .iter()
        .filter(|member| member.key() == "kind")
        .exactly_one()
        .ok()?
        .value()
        .string_value()?;
    tactic_kind_from_wire(kind)
}

fn tactic_kind_from_wire(value: &str) -> Option<MachineApiTacticKind> {
    match value {
        "intro" => Some(MachineApiTacticKind::Intro),
        "exact" => Some(MachineApiTacticKind::Exact),
        "apply" => Some(MachineApiTacticKind::Apply),
        "rw" => Some(MachineApiTacticKind::Rw),
        "simp-lite" => Some(MachineApiTacticKind::SimpLite),
        "induction-nat" => Some(MachineApiTacticKind::InductionNat),
        _ => None,
    }
}

trait ExactlyOne: Iterator + Sized {
    fn exactly_one(mut self) -> Result<Self::Item, ()> {
        let Some(item) = self.next() else {
            return Err(());
        };
        if self.next().is_some() {
            return Err(());
        }
        Ok(item)
    }
}

impl<I: Iterator> ExactlyOne for I {}

fn reject_duplicate_keys(
    members: &[JsonMember<'_>],
    kind: MachineApiErrorKind,
    path: &JsonPath,
) -> Result<(), MachineApiRequestError> {
    let mut seen = BTreeSet::new();
    for member in members {
        if !seen.insert(member.key().to_owned()) {
            return Err(MachineApiRequestError::new(
                kind,
                path.field(member.key()),
                MachineApiRequestErrorReason::DuplicateKey {
                    key: member.key().to_owned(),
                },
            ));
        }
    }
    Ok(())
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

fn required_value_member<'value, 'src>(
    members: &'value [JsonMember<'src>],
    field: &'static str,
    kind: MachineApiErrorKind,
    path: &JsonPath,
) -> Result<&'value JsonValue<'src>, MachineApiRequestError> {
    member_value(members, field).ok_or_else(|| {
        MachineApiRequestError::new(
            kind,
            path.clone(),
            MachineApiRequestErrorReason::MissingField { field },
        )
    })
}

fn required_string_member<'value, 'src>(
    members: &'value [JsonMember<'src>],
    field: &'static str,
    kind: MachineApiErrorKind,
    path: &JsonPath,
) -> Result<&'value str, MachineApiRequestError> {
    let value = required_value_member(members, field, kind, path)?;
    string_value(value, field, kind, path)
}

fn string_value<'value>(
    value: &'value JsonValue<'_>,
    field: &'static str,
    kind: MachineApiErrorKind,
    path: &JsonPath,
) -> Result<&'value str, MachineApiRequestError> {
    if value.kind() == JsonValueKind::Null {
        return Err(null_field(field, kind, path));
    }
    value
        .string_value()
        .ok_or_else(|| type_mismatch(field, JsonFieldType::String, value, kind, path))
}

fn required_object_field<'value, 'src>(
    object: &crate::validation::ValidatedObject<'value, 'src>,
    field: &str,
) -> &'value JsonValue<'src> {
    object
        .field(field)
        .expect("schema checked required object field")
}

fn required_array_field<'value, 'src>(
    object: &crate::validation::ValidatedObject<'value, 'src>,
    field: &str,
) -> &'value [JsonValue<'src>] {
    object
        .field(field)
        .and_then(JsonValue::array_elements)
        .expect("schema checked required array field")
}

fn required_schema_string<'value, 'src>(
    object: &crate::validation::ValidatedObject<'value, 'src>,
    field: &str,
) -> &'value str {
    object
        .field(field)
        .and_then(JsonValue::string_value)
        .expect("schema checked required string field")
}

fn required_u64(object: &crate::validation::ValidatedObject<'_, '_>, field: &str) -> u64 {
    object
        .field(field)
        .and_then(JsonValue::number_raw)
        .and_then(|raw| parse_strict_u64_token(raw, u64::MAX).ok())
        .expect("schema checked required u64 field")
}

fn optional_positive_u64(
    object: &crate::validation::ValidatedObject<'_, '_>,
    field: &'static str,
    path: &JsonPath,
) -> Result<Option<u64>, MachineApiRequestError> {
    let Some(value) = object.field(field) else {
        return Ok(None);
    };
    let raw = value
        .number_raw()
        .expect("schema checked optional u64 field");
    let parsed = parse_strict_u64_token(raw, u64::MAX).expect("schema checked optional u64 field");
    if parsed == 0 {
        return Err(MachineApiRequestError::new(
            MachineApiErrorKind::InvalidSchedulerLimits,
            path.clone(),
            MachineApiRequestErrorReason::InvalidUnsignedInteger {
                field,
                raw: raw.to_owned(),
                error: StrictUnsignedIntegerError::InvalidGrammar,
            },
        ));
    }
    Ok(Some(parsed))
}

fn parse_renderable_name_field(
    object: &crate::validation::ValidatedObject<'_, '_>,
    field: &'static str,
    path: &JsonPath,
) -> Result<Name, MachineApiRequestError> {
    parse_machine_surface_renderable_name_wire(required_schema_string(object, field)).map_err(
        |_| invalid_string_literal(field, None, MachineApiErrorKind::InvalidCandidate, path),
    )
}

fn parse_hash_field(
    object: &crate::validation::ValidatedObject<'_, '_>,
    field: &'static str,
    path: &JsonPath,
) -> Result<Hash, MachineApiRequestError> {
    HashString::parse(required_schema_string(object, field))
        .map(HashString::digest)
        .map_err(|_| {
            invalid_string_literal(field, None, MachineApiErrorKind::InvalidCandidate, path)
        })
}

fn validate_machine_local_name(
    value: &str,
    field: &'static str,
    path: &JsonPath,
) -> Result<(), MachineApiRequestError> {
    if is_machine_local_name(value) {
        Ok(())
    } else {
        Err(invalid_string_literal(
            field,
            None,
            MachineApiErrorKind::InvalidCandidate,
            path,
        ))
    }
}

fn validate_machine_candidate_id(
    value: &str,
    path: &JsonPath,
) -> Result<(), MachineApiRequestError> {
    if (1..=64).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        Ok(())
    } else {
        Err(invalid_string_literal(
            "candidate_id",
            None,
            MachineApiErrorKind::InvalidBatchPolicy,
            path,
        ))
    }
}

fn invalid_string_literal(
    field: &'static str,
    _tactic_kind: Option<MachineApiTacticKind>,
    kind: MachineApiErrorKind,
    path: &JsonPath,
) -> MachineApiRequestError {
    MachineApiRequestError::new(
        kind,
        path.clone(),
        MachineApiRequestErrorReason::TypeMismatch {
            field,
            expected: JsonFieldType::String,
            actual: JsonValueKind::String,
        },
    )
}

fn grammar_error(field: &'static str, kind: MachineApiErrorKind) -> MachineApiRequestError {
    MachineApiRequestError::new(
        kind,
        JsonPath::root().field(field),
        MachineApiRequestErrorReason::TypeMismatch {
            field,
            expected: JsonFieldType::String,
            actual: JsonValueKind::String,
        },
    )
}

fn null_field(
    field: &'static str,
    kind: MachineApiErrorKind,
    path: &JsonPath,
) -> MachineApiRequestError {
    MachineApiRequestError::new(
        kind,
        path.clone(),
        MachineApiRequestErrorReason::NullField { field },
    )
}

fn type_mismatch(
    field: &'static str,
    expected: JsonFieldType,
    value: &JsonValue<'_>,
    kind: MachineApiErrorKind,
    path: &JsonPath,
) -> MachineApiRequestError {
    MachineApiRequestError::new(
        kind,
        path.clone(),
        MachineApiRequestErrorReason::TypeMismatch {
            field,
            expected,
            actual: value.kind(),
        },
    )
}

fn json_path_display(path: &JsonPath) -> String {
    if path.elements.is_empty() {
        return "$".to_owned();
    }
    let mut out = "$".to_owned();
    for element in &path.elements {
        match element {
            JsonPathElement::Field(field) => {
                out.push('.');
                out.push_str(field);
            }
            JsonPathElement::Index(index) => {
                out.push('[');
                out.push_str(&index.to_string());
                out.push(']');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        create_machine_session, format_hash_string, get_machine_snapshot, MachineSnapshotGetOk,
    };

    fn default_options_json() -> String {
        r#"{
          "kernel_check_profile":"npa.kernel.v0.1.builtin-nat-eq-rec",
          "allow_axioms": [],
          "tactic_options": {
            "simp_rules": [],
            "eq_family": null,
            "nat_family": null,
            "max_simp_rewrite_steps": 100,
            "max_open_goals": 32,
            "max_metas": 64
          }
        }"#
        .to_owned()
    }

    fn minimal_session_json(theorem_type: &str) -> String {
        format!(
            r#"{{
              "protocol_version":"npa.machine-api.v1",
              "root":{{
                "module":"Scratch",
                "theorem_name":"Scratch.t",
                "source_index":0,
                "universe_params":[],
                "theorem_type":{{"format":"machine_surface_v1","source":"{theorem_type}"}}
              }},
              "import_closure":[],
              "imports":[],
              "checked_current_decls":[],
              "options":{}
            }}"#,
            default_options_json()
        )
    }

    fn budget_json() -> &'static str {
        r#"{
          "max_tactic_steps":64,
          "max_whnf_steps":10000,
          "max_conversion_steps":10000,
          "max_rewrite_steps":100,
          "max_meta_allocations":8,
          "max_expr_nodes":20000
        }"#
    }

    fn run_json(session: &MachineProofSession, state_fingerprint: Hash, candidate: &str) -> String {
        format!(
            r#"{{
              "session_id":"{}",
              "snapshot_id":"{}",
              "state_fingerprint":"{}",
              "goal_id":"g0",
              "candidate":{},
              "deterministic_budget":{}
            }}"#,
            session.session_id.wire(),
            session.initial_snapshot.snapshot_id.wire(),
            format_hash_string(&state_fingerprint),
            candidate,
            budget_json()
        )
    }

    fn run_json_with_scheduler(
        session: &MachineProofSession,
        state_fingerprint: Hash,
        candidate: &str,
        scheduler_limits: &str,
    ) -> String {
        format!(
            r#"{{
              "session_id":"{}",
              "snapshot_id":"{}",
              "state_fingerprint":"{}",
              "goal_id":"g0",
              "candidate":{},
              "deterministic_budget":{},
              "scheduler_limits":{}
            }}"#,
            session.session_id.wire(),
            session.initial_snapshot.snapshot_id.wire(),
            format_hash_string(&state_fingerprint),
            candidate,
            budget_json(),
            scheduler_limits
        )
    }

    fn batch_json(
        session: &MachineProofSession,
        state_fingerprint: Hash,
        candidates: &str,
    ) -> String {
        batch_json_with_policy(
            session,
            state_fingerprint,
            candidates,
            r#"{
              "max_evaluated_candidates":256,
              "stop_after_successes":256,
              "stop_after_failures":256
            }"#,
        )
    }

    fn batch_json_with_policy(
        session: &MachineProofSession,
        state_fingerprint: Hash,
        candidates: &str,
        batch_policy: &str,
    ) -> String {
        format!(
            r#"{{
              "session_id":"{}",
              "snapshot_id":"{}",
              "state_fingerprint":"{}",
              "goal_id":"g0",
              "candidates":{},
              "deterministic_budget":{},
              "batch_policy":{}
            }}"#,
            session.session_id.wire(),
            session.initial_snapshot.snapshot_id.wire(),
            format_hash_string(&state_fingerprint),
            candidates,
            budget_json(),
            batch_policy
        )
    }

    fn batch_json_with_scheduler(
        session: &MachineProofSession,
        state_fingerprint: Hash,
        candidates: &str,
        scheduler_limits: &str,
    ) -> String {
        format!(
            r#"{{
              "session_id":"{}",
              "snapshot_id":"{}",
              "state_fingerprint":"{}",
              "goal_id":"g0",
              "candidates":{},
              "deterministic_budget":{},
              "batch_policy":{{
                "max_evaluated_candidates":256,
                "stop_after_successes":256,
                "stop_after_failures":256
              }},
              "scheduler_limits":{}
            }}"#,
            session.session_id.wire(),
            session.initial_snapshot.snapshot_id.wire(),
            format_hash_string(&state_fingerprint),
            candidates,
            budget_json(),
            scheduler_limits
        )
    }

    #[test]
    fn tactic_run_exact_success_stores_next_snapshot() {
        let mut session = create_machine_session(&minimal_session_json("Type 0"))
            .unwrap()
            .session;
        let request = run_json(
            &session,
            session.initial_snapshot.state_fingerprint,
            r#"{"kind":"exact","term":{"source":"Prop"}}"#,
        );

        let response = run_machine_tactic_request(&request, &mut session).unwrap();

        let MachineApiResponseEnvelope::Ok(ok) = response else {
            panic!("expected success response");
        };
        assert_eq!(ok.status, MachineApiResponseStatus::Success);
        assert_eq!(
            ok.endpoint_fields.result.kind,
            MachineTacticRunResultKind::Closed
        );
        assert_eq!(
            ok.endpoint_fields.result.closed_goals,
            vec![npa_tactic::GoalId(0)]
        );
        assert!(ok.endpoint_fields.result.new_goals.is_empty());

        let get_request = format!(
            r#"{{
              "session_id":"{}",
              "snapshot_id":"{}",
              "state_fingerprint":"{}",
              "include_pretty":false
            }}"#,
            session.session_id.wire(),
            ok.endpoint_fields.result.next_snapshot_id.wire(),
            format_hash_string(&ok.endpoint_fields.result.next_state_fingerprint)
        );
        let MachineSnapshotGetOk { snapshot } =
            get_machine_snapshot(&get_request, [&session]).unwrap();
        assert!(snapshot.open_goals.is_empty());
    }

    #[test]
    fn tactic_batch_runs_candidates_against_same_input_and_stores_success_snapshots() {
        let mut session = create_machine_session(&minimal_session_json("Type 0"))
            .unwrap()
            .session;
        let request = batch_json(
            &session,
            session.initial_snapshot.state_fingerprint,
            r#"[
              {"candidate_id":"c0","candidate":{"kind":"exact","term":{"source":"Prop"}}},
              {"candidate_id":"c1","candidate":{"kind":"intro","name":"p"}}
            ]"#,
        );

        let response = run_machine_tactic_batch_request(&request, &mut session).unwrap();

        let MachineApiResponseEnvelope::Ok(ok) = response else {
            panic!("expected batch ok response");
        };
        assert_eq!(ok.status, MachineApiResponseStatus::Ok);
        assert_eq!(
            ok.endpoint_fields.previous_state_fingerprint,
            session.initial_snapshot.state_fingerprint
        );
        assert_eq!(ok.endpoint_fields.results.len(), 2);
        assert_eq!(ok.endpoint_fields.success_count, 1);
        assert_eq!(ok.endpoint_fields.failure_count, 1);

        let first = &ok.endpoint_fields.results[0];
        let MachineTacticBatchItemResponse::Success {
            candidate_id,
            next_snapshot_id,
            next_state_fingerprint,
            ..
        } = first
        else {
            panic!("first candidate should succeed: {first:?}");
        };
        assert_eq!(candidate_id, "c0");
        let get_request = format!(
            r#"{{
              "session_id":"{}",
              "snapshot_id":"{}",
              "state_fingerprint":"{}",
              "include_pretty":false
            }}"#,
            session.session_id.wire(),
            next_snapshot_id.wire(),
            format_hash_string(next_state_fingerprint)
        );
        let MachineSnapshotGetOk { snapshot } =
            get_machine_snapshot(&get_request, [&session]).unwrap();
        assert!(snapshot.open_goals.is_empty());

        let second = &ok.endpoint_fields.results[1];
        let MachineTacticBatchItemResponse::Error {
            candidate_id,
            candidate_hash,
            diagnostic,
        } = second
        else {
            panic!("second candidate should fail independently: {second:?}");
        };
        assert_eq!(candidate_id, "c1");
        assert!(candidate_hash.is_some());
        assert_eq!(diagnostic.error_kind, MachineApiErrorKind::TypeMismatch);
        assert_eq!(
            diagnostic.phase,
            MachineApiDiagnosticPhase::MachineTermCheck
        );
        assert_eq!(diagnostic.goal_id, Some(npa_tactic::GoalId(0)));
        assert_eq!(diagnostic.tactic_kind, Some(MachineApiTacticKind::Intro));
    }

    #[test]
    fn tactic_batch_delays_inner_candidate_validation_until_after_snapshot_lookup() {
        let mut session = create_machine_session(&minimal_session_json("Type 0"))
            .unwrap()
            .session;
        let request = batch_json(
            &session,
            [7; 32],
            r#"[{"candidate_id":"c0","candidate":{"kind":"bogus","extra":true}}]"#,
        );

        let err = run_machine_tactic_batch_request(&request, &mut session).unwrap_err();

        assert_eq!(
            err.diagnostic.kind,
            MachineApiErrorKind::StateFingerprintMismatch
        );
        assert_eq!(
            err.diagnostic.phase,
            MachineApiDiagnosticPhase::SnapshotLookup
        );
    }

    #[test]
    fn tactic_batch_policy_stops_before_validating_prefix_external_candidates() {
        let mut session = create_machine_session(&minimal_session_json("Type 0"))
            .unwrap()
            .session;
        let request = batch_json_with_policy(
            &session,
            session.initial_snapshot.state_fingerprint,
            r#"[
              {"candidate_id":"c0","candidate":{"kind":"exact","term":{"source":"Prop"}}},
              {"candidate_id":"c1","candidate":null}
            ]"#,
            r#"{
              "max_evaluated_candidates":1,
              "stop_after_successes":256,
              "stop_after_failures":256
            }"#,
        );

        let response = run_machine_tactic_batch_request(&request, &mut session).unwrap();

        let MachineApiResponseEnvelope::Ok(ok) = response else {
            panic!("expected policy-limited ok response");
        };
        assert_eq!(ok.endpoint_fields.results.len(), 1);
        assert_eq!(ok.endpoint_fields.success_count, 1);
        assert_eq!(ok.endpoint_fields.failure_count, 0);
        assert!(ok.endpoint_fields.results[0].is_success());
    }

    #[test]
    fn tactic_batch_raw_term_parse_error_has_no_candidate_hash() {
        let mut session = create_machine_session(&minimal_session_json("Type 0"))
            .unwrap()
            .session;
        let request = batch_json(
            &session,
            session.initial_snapshot.state_fingerprint,
            r#"[{"candidate_id":"c0","candidate":{"kind":"exact","term":{"source":"("}}}]"#,
        );

        let response = run_machine_tactic_batch_request(&request, &mut session).unwrap();

        let MachineApiResponseEnvelope::Ok(ok) = response else {
            panic!("expected batch ok response with per-candidate error");
        };
        assert_eq!(ok.endpoint_fields.success_count, 0);
        assert_eq!(ok.endpoint_fields.failure_count, 1);
        let MachineTacticBatchItemResponse::Error {
            candidate_id,
            candidate_hash,
            diagnostic,
        } = &ok.endpoint_fields.results[0]
        else {
            panic!("expected per-candidate parse error");
        };
        assert_eq!(candidate_id, "c0");
        assert!(candidate_hash.is_none());
        assert_eq!(
            diagnostic.error_kind,
            MachineApiErrorKind::MachineTermParseError
        );
        assert_eq!(
            diagnostic.phase,
            MachineApiDiagnosticPhase::MachineTermParse
        );
        assert_eq!(diagnostic.goal_id, Some(npa_tactic::GoalId(0)));
        assert_eq!(diagnostic.tactic_kind, Some(MachineApiTacticKind::Exact));
    }

    #[test]
    fn tactic_batch_partial_scheduler_response_preserves_completed_prefix() {
        let previous_state_fingerprint = [1; 32];
        let deterministic_budget_hash = [2; 32];
        let candidate_hash = [3; 32];
        let next_state_fingerprint = [4; 32];
        let proof_delta_hash = [5; 32];
        let result = MachineTacticBatchItemResponse::Success {
            candidate_id: "c0".to_owned(),
            candidate_hash,
            next_snapshot_id: SnapshotId::from_state_fingerprint(next_state_fingerprint),
            next_state_fingerprint,
            proof_delta_hash,
        };

        let response = batch_scheduler_stop(
            previous_state_fingerprint,
            deterministic_budget_hash,
            vec![result.clone()],
            1,
            0,
            BatchSchedulerStop {
                kind: MachineSchedulerArtifactKind::Timeout,
                scope: MachineSchedulerArtifactScope::Candidate,
            },
        );

        let MachineApiResponseEnvelope::SchedulerStopped(response) = response else {
            panic!("expected partial scheduler response");
        };
        assert_eq!(response.status, MachineApiResponseStatus::PartialTimeout);
        assert_eq!(
            response.scheduler_artifact.kind,
            MachineSchedulerArtifactKind::Timeout
        );
        assert_eq!(
            response.scheduler_artifact.scope,
            MachineSchedulerArtifactScope::Candidate
        );
        assert_eq!(response.endpoint_fields.completed_prefix_len, 1);
        assert_eq!(response.endpoint_fields.success_count, 1);
        assert_eq!(response.endpoint_fields.failure_count, 0);
        assert_eq!(response.endpoint_fields.results, vec![result]);
    }

    #[test]
    fn tactic_batch_rejects_duplicate_candidate_ids_as_batch_policy() {
        let session = create_machine_session(&minimal_session_json("Type 0"))
            .unwrap()
            .session;
        let request = batch_json(
            &session,
            session.initial_snapshot.state_fingerprint,
            r#"[
              {"candidate_id":"c0","candidate":{"kind":"exact","term":{"source":"Prop"}}},
              {"candidate_id":"c0","candidate":{"kind":"exact","term":{"source":"Prop"}}}
            ]"#,
        );

        let err = parse_machine_tactic_batch_request(&request).unwrap_err();

        assert_eq!(err.kind, MachineApiErrorKind::InvalidBatchPolicy);
        assert_eq!(
            err.reason,
            MachineApiRequestErrorReason::DuplicateKey {
                key: "c0".to_owned()
            }
        );
    }

    #[test]
    fn tactic_batch_rejects_policy_values_outside_protocol_cap() {
        let session = create_machine_session(&minimal_session_json("Type 0"))
            .unwrap()
            .session;
        let request = batch_json_with_policy(
            &session,
            session.initial_snapshot.state_fingerprint,
            r#"[{"candidate_id":"c0","candidate":{"kind":"exact","term":{"source":"Prop"}}}]"#,
            r#"{
              "max_evaluated_candidates":257,
              "stop_after_successes":1,
              "stop_after_failures":1
            }"#,
        );

        let err = parse_machine_tactic_batch_request(&request).unwrap_err();

        assert_eq!(err.kind, MachineApiErrorKind::InvalidBatchPolicy);
    }

    #[test]
    fn tactic_run_delays_candidate_validation_until_after_snapshot_lookup() {
        let mut session = create_machine_session(&minimal_session_json("Type 0"))
            .unwrap()
            .session;
        let request = run_json(&session, [7; 32], r#"{"kind":"bogus","extra":true}"#);

        let err = run_machine_tactic_request(&request, &mut session).unwrap_err();

        assert_eq!(
            err.diagnostic.kind,
            MachineApiErrorKind::StateFingerprintMismatch
        );
        assert_eq!(
            err.diagnostic.phase,
            MachineApiDiagnosticPhase::SnapshotLookup
        );
    }

    #[test]
    fn tactic_run_candidate_schema_error_keeps_state_unchanged_and_budget_hash() {
        let mut session = create_machine_session(&minimal_session_json("Type 0"))
            .unwrap()
            .session;
        let request = run_json(
            &session,
            session.initial_snapshot.state_fingerprint,
            r#"{"kind":"exact","term":{"source":"Prop"},"candidate_hash":"sha256:0000000000000000000000000000000000000000000000000000000000000000"}"#,
        );

        let err = run_machine_tactic_request(&request, &mut session).unwrap_err();

        assert_eq!(err.diagnostic.kind, MachineApiErrorKind::InvalidCandidate);
        assert_eq!(
            err.diagnostic.phase,
            MachineApiDiagnosticPhase::CandidateValidation
        );
        let MachineApiResponseEnvelope::Error(response) = err.response else {
            panic!("expected error response");
        };
        assert_eq!(
            response.endpoint_fields.unchanged_state_fingerprint,
            Some(session.initial_snapshot.state_fingerprint)
        );
        assert!(response.error.candidate_hash.is_none());
        assert!(response.error.deterministic_budget_hash.is_some());
    }

    #[cfg(any(
        target_os = "android",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos"
    ))]
    #[test]
    fn tactic_run_explicit_resource_limit_returns_scheduler_after_lookup() {
        let mut session = create_machine_session(&minimal_session_json("Type 0"))
            .unwrap()
            .session;
        let request = run_json_with_scheduler(
            &session,
            session.initial_snapshot.state_fingerprint,
            r#"{"kind":"bogus","extra":true}"#,
            r#"{"max_memory_mb":1}"#,
        );
        let deterministic_budget_hash = tactic_budget_hash(
            parse_machine_tactic_run_request(&request)
                .unwrap()
                .deterministic_budget,
        );

        let response = run_machine_tactic_request(&request, &mut session).unwrap();

        let MachineApiResponseEnvelope::SchedulerStopped(response) = response else {
            panic!("expected scheduler stop response");
        };
        assert_eq!(response.status, MachineApiResponseStatus::SchedulerStopped);
        assert_eq!(
            response.scheduler_artifact.kind,
            MachineSchedulerArtifactKind::ResourceLimitExceeded
        );
        assert_eq!(
            response.scheduler_artifact.scope,
            MachineSchedulerArtifactScope::Candidate
        );
        assert!(response.scheduler_artifact.retryable);
        assert_eq!(
            response.endpoint_fields.previous_state_fingerprint,
            session.initial_snapshot.state_fingerprint
        );
        assert_eq!(
            response.endpoint_fields.deterministic_budget_hash,
            deterministic_budget_hash
        );
    }

    #[cfg(any(
        target_os = "android",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos"
    ))]
    #[test]
    fn tactic_batch_explicit_resource_limit_returns_partial_after_lookup() {
        let mut session = create_machine_session(&minimal_session_json("Type 0"))
            .unwrap()
            .session;
        let request = batch_json_with_scheduler(
            &session,
            session.initial_snapshot.state_fingerprint,
            r#"[{"candidate_id":"c0","candidate":{"kind":"bogus","extra":true}}]"#,
            r#"{"max_memory_mb":1}"#,
        );
        let deterministic_budget_hash = tactic_budget_hash(
            parse_machine_tactic_batch_request(&request)
                .unwrap()
                .deterministic_budget,
        );

        let response = run_machine_tactic_batch_request(&request, &mut session).unwrap();

        let MachineApiResponseEnvelope::SchedulerStopped(response) = response else {
            panic!("expected partial scheduler response");
        };
        assert_eq!(
            response.status,
            MachineApiResponseStatus::PartialResourceLimit
        );
        assert_eq!(
            response.scheduler_artifact.kind,
            MachineSchedulerArtifactKind::ResourceLimitExceeded
        );
        assert_eq!(
            response.scheduler_artifact.scope,
            MachineSchedulerArtifactScope::Batch
        );
        assert_eq!(response.endpoint_fields.completed_prefix_len, 0);
        assert!(response.endpoint_fields.results.is_empty());
        assert_eq!(
            response.endpoint_fields.previous_state_fingerprint,
            session.initial_snapshot.state_fingerprint
        );
        assert_eq!(
            response.endpoint_fields.deterministic_budget_hash,
            deterministic_budget_hash
        );
    }

    #[test]
    fn scheduler_observation_reports_timeout() {
        let kind = observe_run_scheduler_limits(
            MachineRunSchedulerLimits {
                timeout_ms: Some(5),
                max_memory_mb: None,
            },
            Duration::from_millis(5),
            None,
        );

        assert_eq!(kind, Some(MachineSchedulerArtifactKind::Timeout));
    }

    #[test]
    fn scheduler_observation_prefers_resource_limit_over_timeout() {
        let kind = observe_run_scheduler_limits(
            MachineRunSchedulerLimits {
                timeout_ms: Some(5),
                max_memory_mb: Some(1),
            },
            Duration::from_millis(5),
            Some(memory_limit_bytes(1) + 1),
        );

        assert_eq!(
            kind,
            Some(MachineSchedulerArtifactKind::ResourceLimitExceeded)
        );
    }

    #[test]
    fn scheduler_observation_allows_current_memory_below_limit() {
        let kind = observe_run_scheduler_limits(
            MachineRunSchedulerLimits {
                timeout_ms: None,
                max_memory_mb: Some(2),
            },
            Duration::ZERO,
            Some(memory_limit_bytes(1)),
        );

        assert_eq!(kind, None);
    }

    #[test]
    fn batch_scheduler_observation_prioritizes_resource_then_batch_timeout() {
        let resource = observe_batch_scheduler_limits(
            MachineBatchSchedulerLimits {
                per_candidate_timeout_ms: Some(5),
                batch_timeout_ms: Some(5),
                max_memory_mb: Some(1),
            },
            Duration::from_millis(5),
            Some(Duration::from_millis(5)),
            Some(memory_limit_bytes(1) + 1),
        );
        assert_eq!(
            resource,
            Some(BatchSchedulerStop {
                kind: MachineSchedulerArtifactKind::ResourceLimitExceeded,
                scope: MachineSchedulerArtifactScope::Batch
            })
        );

        let timeout = observe_batch_scheduler_limits(
            MachineBatchSchedulerLimits {
                per_candidate_timeout_ms: Some(5),
                batch_timeout_ms: Some(5),
                max_memory_mb: None,
            },
            Duration::from_millis(5),
            Some(Duration::from_millis(5)),
            None,
        );
        assert_eq!(
            timeout,
            Some(BatchSchedulerStop {
                kind: MachineSchedulerArtifactKind::Timeout,
                scope: MachineSchedulerArtifactScope::Batch
            })
        );
    }

    #[test]
    fn level_wire_rejects_noncanonical_succ_numeral() {
        assert!(parse_level_wire("1", &[]).is_ok());
        assert!(parse_level_wire("succ 0", &[]).is_err());
        assert!(parse_level_wire("max 0 0", &[]).is_ok());
    }
}
