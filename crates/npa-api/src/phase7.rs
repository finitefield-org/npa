use std::collections::VecDeque;

use npa_cert::Hash;
use npa_tactic::{MachineTacticBatchPolicy, TacticBudget};

use crate::json::{JsonMember, JsonValue, JsonValueKind};
use crate::snapshot::{MachineSnapshotGetError, MachineSnapshotGetOk};
use crate::tactic::parse_deterministic_budget_with_error_kind;
use crate::types::{format_hash_string, MachineProofSession, SessionId, SnapshotId};
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
    MachineTacticBatchError, MachineTacticBatchResponse, MachineTheoremSearchError,
    MachineTheoremSearchResponse, MachineVerifyError, MachineVerifyResponse,
};

const PHASE7_MVP_MAX_TACTICS_PER_NODE: u32 = 16;

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
    use crate::{parse_machine_snapshot_get_request, JsonFieldType, MachineApiRequestErrorReason};

    fn hash(byte: u8) -> Hash {
        [byte; 32]
    }

    fn snapshot_request() -> Phase7SnapshotGetRequest {
        Phase7SnapshotGetRequest {
            session_id: SessionId::parse("msess_001").unwrap(),
            snapshot_id: SnapshotId::from_state_fingerprint(hash(1)),
            state_fingerprint: hash(1),
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
