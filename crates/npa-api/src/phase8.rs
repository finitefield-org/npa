use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use npa_cert::Hash;
use sha2::{Digest, Sha256};

use crate::json::{JsonDocument, JsonMember, JsonParseErrorKind, JsonValue, JsonValueKind};
use crate::types::{format_hash_string, parse_hash_string};

pub const PHASE8_RUNNER_POLICY_SCHEMA: &str = "npa.phase8.runner_policy.v1";
pub const PHASE8_CHECKER_IDENTITY_MANIFEST_SCHEMA: &str = "npa.phase8.checker_identity_manifest.v1";
pub const PHASE8_CHECKER_BINARY_REGISTRY_SCHEMA: &str = "npa.phase8.checker_binary_registry.v1";
pub const PHASE8_MACHINE_CHECK_REQUEST_ERROR_RESULT_SCHEMA: &str =
    "npa.phase8.machine_check_request_error_result.v1";
pub const PHASE8_NORMALIZE_ERROR_RESULT_SCHEMA: &str = "npa.phase8.normalize_error_result.v1";
pub const PHASE8_COMMAND_ERROR_SCHEMA: &str = "npa.phase8.command_error.v1";
pub const PHASE8_API_ERROR_SCHEMA: &str = "npa.phase8.api_error.v1";

pub const PHASE8_FORBIDDEN_VERIFICATION_INPUT_FIELDS: &[&str] = &[
    "source",
    "source_text",
    "tactic",
    "tactic_trace",
    "ai_trace",
];

pub const PHASE8_RUNNER_FIXED_ENVIRONMENT: &[(&str, &str)] =
    &[("LC_ALL", "C.UTF-8"), ("LANG", "C.UTF-8"), ("TZ", "UTC")];

pub const PHASE8_RUNNER_DYNAMIC_FLAGS: &[&str] = &[
    "--imports",
    "--imports-hash",
    "--trust-mode",
    "--axiom-policy",
    "--axiom-policy-hash",
    "--max-steps",
    "--max-memory-mb",
    "--timeout-ms",
];

const REQUEST_HASH_EXCLUDED_FIELDS: &[&str] = &["request_id", "request_hash"];
const MACHINE_CHECK_RESULT_HASH_EXCLUDED_FIELDS: &[&str] = &[
    "request_id",
    "result_id",
    "request_hash",
    "result_hash",
    "run_artifact_hash",
    "checker.version",
    "attempt",
    "process",
    "resource_usage",
    "diagnostics",
    "axioms_used",
    "declarations_checked",
];
const ERROR_RESULT_HASH_EXCLUDED_FIELDS: &[&str] = &["result_id", "result_hash"];
const RUN_ARTIFACT_HASH_EXCLUDED_FIELDS: &[&str] = &["run_artifact_hash"];
const NORMALIZED_RESULT_HASH_EXCLUDED_FIELDS: &[&str] = &[
    "normalized_result_id",
    "normalized_result_hash",
    "results[].result_id",
];
const ARTIFACT_HASH_EXCLUDED_FIELDS: &[&str] = &["artifact_hash"];
const FILE_HASH_EXCLUDED_FIELDS: &[&str] = &[];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8JsonPath {
    pub elements: Vec<Phase8JsonPathElement>,
}

impl Phase8JsonPath {
    pub const fn root() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    pub fn field(&self, field: impl Into<String>) -> Self {
        let mut elements = self.elements.clone();
        elements.push(Phase8JsonPathElement::Field(field.into()));
        Self { elements }
    }

    pub fn index(&self, index: usize) -> Self {
        let mut elements = self.elements.clone();
        elements.push(Phase8JsonPathElement::Index(index));
        Self { elements }
    }

    pub fn artifact_local_string(&self) -> String {
        if self.elements.is_empty() {
            return "$".to_owned();
        }

        let mut out = String::new();
        for (index, element) in self.elements.iter().enumerate() {
            match element {
                Phase8JsonPathElement::Field(field) => {
                    if index > 0 {
                        out.push('.');
                    }
                    out.push_str(field);
                }
                Phase8JsonPathElement::Index(array_index) => {
                    out.push('[');
                    out.push_str(&array_index.to_string());
                    out.push(']');
                }
            }
        }
        out
    }
}

impl Default for Phase8JsonPath {
    fn default() -> Self {
        Self::root()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase8JsonPathElement {
    Field(String),
    Index(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase8CanonicalJsonError {
    JsonParse {
        offset: usize,
        kind: JsonParseErrorKind,
    },
    ExpectedObject {
        path: Phase8JsonPath,
        actual: JsonValueKind,
    },
    DuplicateObjectKey {
        path: Phase8JsonPath,
        key: String,
    },
    FloatNumber {
        path: Phase8JsonPath,
        raw: String,
    },
    NegativeZeroInteger {
        path: Phase8JsonPath,
    },
    InvalidInteger {
        path: Phase8JsonPath,
        raw: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8HashInputKind {
    CanonicalJson,
    CanonicalJsonProjection,
    ExactFileBytes,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8HashUsageKind {
    RequestHash,
    MachineCheckResultHash,
    ErrorResultHash,
    RunArtifactHash,
    NormalizedResultHash,
    ArtifactHash,
    FileHash,
}

impl Phase8HashUsageKind {
    pub const fn field_name(self) -> &'static str {
        match self {
            Self::RequestHash => "request_hash",
            Self::MachineCheckResultHash | Self::ErrorResultHash => "result_hash",
            Self::RunArtifactHash => "run_artifact_hash",
            Self::NormalizedResultHash => "normalized_result_hash",
            Self::ArtifactHash => "artifact_hash",
            Self::FileHash => "file_hash",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Phase8HashUsageRule {
    pub kind: Phase8HashUsageKind,
    pub field_name: &'static str,
    pub input_kind: Phase8HashInputKind,
    pub purpose: &'static str,
    pub excluded_fields: &'static [&'static str],
    pub excludes_self_field: bool,
}

pub fn phase8_hash_usage_rule(kind: Phase8HashUsageKind) -> Phase8HashUsageRule {
    match kind {
        Phase8HashUsageKind::RequestHash => Phase8HashUsageRule {
            kind,
            field_name: kind.field_name(),
            input_kind: Phase8HashInputKind::CanonicalJsonProjection,
            purpose: "MachineCheckRequest identity after request_id/request_hash removal",
            excluded_fields: REQUEST_HASH_EXCLUDED_FIELDS,
            excludes_self_field: true,
        },
        Phase8HashUsageKind::MachineCheckResultHash => Phase8HashUsageRule {
            kind,
            field_name: kind.field_name(),
            input_kind: Phase8HashInputKind::CanonicalJsonProjection,
            purpose: "semantic checker result projection",
            excluded_fields: MACHINE_CHECK_RESULT_HASH_EXCLUDED_FIELDS,
            excludes_self_field: true,
        },
        Phase8HashUsageKind::ErrorResultHash => Phase8HashUsageRule {
            kind,
            field_name: kind.field_name(),
            input_kind: Phase8HashInputKind::CanonicalJsonProjection,
            purpose: "pipeline error artifact identity after result_id/result_hash removal",
            excluded_fields: ERROR_RESULT_HASH_EXCLUDED_FIELDS,
            excludes_self_field: true,
        },
        Phase8HashUsageKind::RunArtifactHash => Phase8HashUsageRule {
            kind,
            field_name: kind.field_name(),
            input_kind: Phase8HashInputKind::CanonicalJsonProjection,
            purpose: "full MachineCheckResult run artifact after result_hash is written",
            excluded_fields: RUN_ARTIFACT_HASH_EXCLUDED_FIELDS,
            excludes_self_field: true,
        },
        Phase8HashUsageKind::NormalizedResultHash => Phase8HashUsageRule {
            kind,
            field_name: kind.field_name(),
            input_kind: Phase8HashInputKind::CanonicalJsonProjection,
            purpose: "NormalizedCheckResult identity independent of local result identifiers",
            excluded_fields: NORMALIZED_RESULT_HASH_EXCLUDED_FIELDS,
            excludes_self_field: true,
        },
        Phase8HashUsageKind::ArtifactHash => Phase8HashUsageRule {
            kind,
            field_name: kind.field_name(),
            input_kind: Phase8HashInputKind::CanonicalJson,
            purpose: "schema-defined canonical JSON artifact identity",
            excluded_fields: ARTIFACT_HASH_EXCLUDED_FIELDS,
            excludes_self_field: true,
        },
        Phase8HashUsageKind::FileHash => Phase8HashUsageRule {
            kind,
            field_name: kind.field_name(),
            input_kind: Phase8HashInputKind::ExactFileBytes,
            purpose: "exact bytes read from the referenced file",
            excluded_fields: FILE_HASH_EXCLUDED_FIELDS,
            excludes_self_field: false,
        },
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8ArtifactClassification {
    SavedArtifact,
    UntrustedSidecar,
    TransientResponse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8ArtifactKind {
    MachineCheckResult,
    NormalizedCheckResult,
    MachineCheckRequestErrorResult,
    NormalizeErrorResult,
    AuxiliaryResult,
    ChallengeReplayResult,
    ChallengeCoverageSummary,
    AxiomReport,
    AiAuditSidecar,
    CompareValidationResult,
    AuditSidecarValidationResult,
    AiSidecarDiagnosticResult,
    AiSidecarDiagnosticEvaluationFailure,
    NormalizationWriteResult,
    ChallengeRequestMaterializationResult,
    ChallengeGenerationResult,
    ReleaseBundleStagingResult,
    CommandError,
    ApiError,
}

impl Phase8ArtifactKind {
    pub const fn classification(self) -> Phase8ArtifactClassification {
        match self {
            Self::MachineCheckResult
            | Self::NormalizedCheckResult
            | Self::MachineCheckRequestErrorResult
            | Self::NormalizeErrorResult
            | Self::AuxiliaryResult
            | Self::ChallengeReplayResult
            | Self::ChallengeCoverageSummary
            | Self::AxiomReport => Phase8ArtifactClassification::SavedArtifact,
            Self::AiAuditSidecar => Phase8ArtifactClassification::UntrustedSidecar,
            Self::CompareValidationResult
            | Self::AuditSidecarValidationResult
            | Self::AiSidecarDiagnosticResult
            | Self::AiSidecarDiagnosticEvaluationFailure
            | Self::NormalizationWriteResult
            | Self::ChallengeRequestMaterializationResult
            | Self::ChallengeGenerationResult
            | Self::ReleaseBundleStagingResult
            | Self::CommandError
            | Self::ApiError => Phase8ArtifactClassification::TransientResponse,
        }
    }

    pub const fn is_checker_verdict(self) -> bool {
        matches!(self, Self::MachineCheckResult)
    }

    pub const fn is_pipeline_error_artifact(self) -> bool {
        matches!(
            self,
            Self::MachineCheckRequestErrorResult | Self::NormalizeErrorResult
        )
    }

    pub const fn is_normalization_input(self) -> bool {
        matches!(self, Self::MachineCheckResult)
    }

    pub const fn has_result_hash_field(self) -> bool {
        matches!(
            self,
            Self::MachineCheckResult
                | Self::MachineCheckRequestErrorResult
                | Self::NormalizeErrorResult
                | Self::AuxiliaryResult
                | Self::ChallengeReplayResult
                | Self::ChallengeCoverageSummary
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8JsonFieldType {
    Object,
    Array,
    String,
    Boolean,
    Integer,
    HashString,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Phase8FieldSpec {
    pub name: &'static str,
    pub required: bool,
    pub field_type: Phase8JsonFieldType,
    pub allow_null: bool,
}

impl Phase8FieldSpec {
    pub const fn required(name: &'static str, field_type: Phase8JsonFieldType) -> Self {
        Self {
            name,
            required: true,
            field_type,
            allow_null: false,
        }
    }

    pub const fn optional(name: &'static str, field_type: Phase8JsonFieldType) -> Self {
        Self {
            name,
            required: false,
            field_type,
            allow_null: false,
        }
    }

    pub const fn allow_null(mut self) -> Self {
        self.allow_null = true;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Phase8ClosedSchema<'a> {
    pub fields: &'a [Phase8FieldSpec],
}

impl<'a> Phase8ClosedSchema<'a> {
    pub const fn new(fields: &'a [Phase8FieldSpec]) -> Self {
        Self { fields }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Phase8ValidatedObject<'value, 'src> {
    members: &'value [crate::json::JsonMember<'src>],
}

impl<'value, 'src> Phase8ValidatedObject<'value, 'src> {
    pub fn field(&self, field_name: &str) -> Option<&'value JsonValue<'src>> {
        self.members
            .iter()
            .find(|member| member.key() == field_name)
            .map(|member| member.value())
    }

    pub const fn members(&self) -> &'value [crate::json::JsonMember<'src>] {
        self.members
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8SchemaError {
    pub path: Phase8JsonPath,
    pub reason: Phase8SchemaErrorReason,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase8SchemaErrorReason {
    ExpectedObject {
        actual: JsonValueKind,
    },
    DuplicateField {
        field: String,
    },
    UnknownField {
        field: String,
    },
    MissingField {
        field: &'static str,
    },
    NullNotAllowed {
        field: &'static str,
    },
    WrongType {
        field: &'static str,
        expected: Phase8JsonFieldType,
        actual: JsonValueKind,
    },
    InvalidInteger {
        field: &'static str,
        raw: String,
    },
    InvalidHashFormat {
        field: &'static str,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8PipelineErrorKind {
    RequestLoadFailure,
    NormalizeFailure,
}

impl Phase8PipelineErrorKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestLoadFailure => "request_load_failure",
            Self::NormalizeFailure => "normalize_failure",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8MachineCheckRequestErrorReasonCode {
    RequestFileUnreadable,
    RequestJsonInvalid,
    RequestSchemaInvalid,
    RequestHashMissing,
    RequestHashMismatch,
}

impl Phase8MachineCheckRequestErrorReasonCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestFileUnreadable => "request_file_unreadable",
            Self::RequestJsonInvalid => "request_json_invalid",
            Self::RequestSchemaInvalid => "request_schema_invalid",
            Self::RequestHashMissing => "request_hash_missing",
            Self::RequestHashMismatch => "request_hash_mismatch",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8NormalizeErrorReasonCode {
    MachineResultFileUnreadable,
    MachineResultJsonInvalid,
    MachineResultWrongSchema,
    MachineResultSchemaInvalid,
    MachineResultHashMismatch,
    MachineResultRunArtifactHashMismatch,
    MachineResultRequestHashMismatch,
    RequestHashNotFound,
    RequestFileUnreadable,
    RequestJsonInvalid,
    RequestSchemaInvalid,
    RequestHashMissing,
    RequestFileHashMismatch,
    RequestHashMismatch,
    RequestStoreReferenceInvalid,
    RequestStoreManifestHashMismatch,
    RequestStoreManifestInvalid,
    OutputPathConflict,
    OutputWriteFailure,
    NormalizedStoreEntryFileUnreadable,
    NormalizedStoreEntryJsonInvalid,
    NormalizedStoreEntrySchemaInvalid,
    NormalizedStoreEntryFileHashMismatch,
    NormalizedStoreEntryArtifactHashMismatch,
    NormalizedStoreManifestInvalid,
    NormalizedStoreEntryConflict,
    NormalizedStoreWriteFailure,
    PolicyReferenceInvalid,
    PolicyFileUnreadable,
    PolicyHashMismatch,
    DuplicateCheckerProfileResult,
    SelectorSchemaInvalid,
    SelectorModuleMismatch,
    SelectorAmbiguous,
}

impl Phase8NormalizeErrorReasonCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MachineResultFileUnreadable => "machine_result_file_unreadable",
            Self::MachineResultJsonInvalid => "machine_result_json_invalid",
            Self::MachineResultWrongSchema => "machine_result_wrong_schema",
            Self::MachineResultSchemaInvalid => "machine_result_schema_invalid",
            Self::MachineResultHashMismatch => "machine_result_hash_mismatch",
            Self::MachineResultRunArtifactHashMismatch => {
                "machine_result_run_artifact_hash_mismatch"
            }
            Self::MachineResultRequestHashMismatch => "machine_result_request_hash_mismatch",
            Self::RequestHashNotFound => "request_hash_not_found",
            Self::RequestFileUnreadable => "request_file_unreadable",
            Self::RequestJsonInvalid => "request_json_invalid",
            Self::RequestSchemaInvalid => "request_schema_invalid",
            Self::RequestHashMissing => "request_hash_missing",
            Self::RequestFileHashMismatch => "request_file_hash_mismatch",
            Self::RequestHashMismatch => "request_hash_mismatch",
            Self::RequestStoreReferenceInvalid => "request_store_reference_invalid",
            Self::RequestStoreManifestHashMismatch => "request_store_manifest_hash_mismatch",
            Self::RequestStoreManifestInvalid => "request_store_manifest_invalid",
            Self::OutputPathConflict => "output_path_conflict",
            Self::OutputWriteFailure => "output_write_failure",
            Self::NormalizedStoreEntryFileUnreadable => "normalized_store_entry_file_unreadable",
            Self::NormalizedStoreEntryJsonInvalid => "normalized_store_entry_json_invalid",
            Self::NormalizedStoreEntrySchemaInvalid => "normalized_store_entry_schema_invalid",
            Self::NormalizedStoreEntryFileHashMismatch => {
                "normalized_store_entry_file_hash_mismatch"
            }
            Self::NormalizedStoreEntryArtifactHashMismatch => {
                "normalized_store_entry_artifact_hash_mismatch"
            }
            Self::NormalizedStoreManifestInvalid => "normalized_store_manifest_invalid",
            Self::NormalizedStoreEntryConflict => "normalized_store_entry_conflict",
            Self::NormalizedStoreWriteFailure => "normalized_store_write_failure",
            Self::PolicyReferenceInvalid => "policy_reference_invalid",
            Self::PolicyFileUnreadable => "policy_file_unreadable",
            Self::PolicyHashMismatch => "policy_hash_mismatch",
            Self::DuplicateCheckerProfileResult => "duplicate_checker_profile_result",
            Self::SelectorSchemaInvalid => "selector_schema_invalid",
            Self::SelectorModuleMismatch => "selector_module_mismatch",
            Self::SelectorAmbiguous => "selector_ambiguous",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8StructuredError {
    pub kind: Phase8PipelineErrorKind,
    pub reason_code: String,
    pub field: String,
    pub expected_hash: Option<Hash>,
    pub actual_hash: Option<Hash>,
    pub expected_value: Option<String>,
    pub actual_value: Option<String>,
}

impl Phase8StructuredError {
    pub fn request_load_failure(
        reason_code: Phase8MachineCheckRequestErrorReasonCode,
        field: impl Into<String>,
    ) -> Self {
        Self::new(
            Phase8PipelineErrorKind::RequestLoadFailure,
            reason_code.as_str(),
            field,
        )
    }

    pub fn normalize_failure(
        reason_code: Phase8NormalizeErrorReasonCode,
        field: impl Into<String>,
    ) -> Self {
        Self::new(
            Phase8PipelineErrorKind::NormalizeFailure,
            reason_code.as_str(),
            field,
        )
    }

    pub fn new(
        kind: Phase8PipelineErrorKind,
        reason_code: impl Into<String>,
        field: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            reason_code: reason_code.into(),
            field: field.into(),
            expected_hash: None,
            actual_hash: None,
            expected_value: None,
            actual_value: None,
        }
    }

    pub fn with_expected_hash(mut self, hash: Hash) -> Self {
        self.expected_hash = Some(hash);
        self
    }

    pub fn with_actual_hash(mut self, hash: Hash) -> Self {
        self.actual_hash = Some(hash);
        self
    }

    pub fn with_expected_value(mut self, value: impl Into<String>) -> Self {
        self.expected_value = Some(value.into());
        self
    }

    pub fn with_actual_value(mut self, value: impl Into<String>) -> Self {
        self.actual_value = Some(value.into());
        self
    }

    pub fn canonical_json(&self) -> String {
        let mut pairs = vec![
            ("field".to_owned(), phase8_json_string_literal(&self.field)),
            (
                "kind".to_owned(),
                phase8_json_string_literal(self.kind.as_str()),
            ),
            (
                "reason_code".to_owned(),
                phase8_json_string_literal(&self.reason_code),
            ),
        ];
        push_optional_hash_pair(&mut pairs, "expected_hash", self.expected_hash);
        push_optional_hash_pair(&mut pairs, "actual_hash", self.actual_hash);
        push_optional_string_pair(&mut pairs, "expected_value", self.expected_value.as_deref());
        push_optional_string_pair(&mut pairs, "actual_value", self.actual_value.as_deref());
        canonical_json_object_from_pairs(pairs)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8MachineCheckRequestErrorResult {
    pub result_id: String,
    pub request_path: Option<String>,
    pub request_file_hash: Option<Hash>,
    error: Phase8StructuredError,
}

impl Phase8MachineCheckRequestErrorResult {
    pub fn request_load_failure(
        result_id: impl Into<String>,
        reason_code: Phase8MachineCheckRequestErrorReasonCode,
        field: impl Into<String>,
    ) -> Self {
        Self {
            result_id: result_id.into(),
            request_path: None,
            request_file_hash: None,
            error: Phase8StructuredError::request_load_failure(reason_code, field),
        }
    }

    pub fn with_request_path(mut self, path: impl Into<String>) -> Self {
        self.request_path = Some(path.into());
        self
    }

    pub fn with_request_file_hash(mut self, hash: Hash) -> Self {
        self.request_file_hash = Some(hash);
        self
    }

    pub fn with_error_expected_hash(mut self, hash: Hash) -> Self {
        self.error.expected_hash = Some(hash);
        self
    }

    pub fn with_error_actual_hash(mut self, hash: Hash) -> Self {
        self.error.actual_hash = Some(hash);
        self
    }

    pub fn with_error_expected_value(mut self, value: impl Into<String>) -> Self {
        self.error.expected_value = Some(value.into());
        self
    }

    pub fn with_error_actual_value(mut self, value: impl Into<String>) -> Self {
        self.error.actual_value = Some(value.into());
        self
    }

    pub const fn error(&self) -> &Phase8StructuredError {
        &self.error
    }

    pub fn result_hash(&self) -> Hash {
        phase8_sha256(self.hash_input_canonical_json().as_bytes())
    }

    pub fn hash_input_canonical_json(&self) -> String {
        self.canonical_json_pairs_without_identity()
    }

    pub fn canonical_json(&self) -> String {
        let result_hash = self.result_hash();
        let mut pairs = self.base_canonical_json_pairs();
        pairs.push((
            "result_id".to_owned(),
            phase8_json_string_literal(&self.result_id),
        ));
        pairs.push((
            "result_hash".to_owned(),
            phase8_json_string_literal(&format_hash_string(&result_hash)),
        ));
        canonical_json_object_from_pairs(pairs)
    }

    fn canonical_json_pairs_without_identity(&self) -> String {
        canonical_json_object_from_pairs(self.base_canonical_json_pairs())
    }

    fn base_canonical_json_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = vec![
            ("error".to_owned(), self.error.canonical_json()),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_MACHINE_CHECK_REQUEST_ERROR_RESULT_SCHEMA),
            ),
            ("status".to_owned(), phase8_json_string_literal("failed")),
        ];
        push_optional_string_pair(&mut pairs, "request_path", self.request_path.as_deref());
        push_optional_hash_pair(&mut pairs, "request_file_hash", self.request_file_hash);
        pairs
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8NormalizeErrorResult {
    pub result_id: String,
    pub policy_hash: Option<Hash>,
    error: Phase8StructuredError,
}

impl Phase8NormalizeErrorResult {
    pub fn normalize_failure(
        result_id: impl Into<String>,
        reason_code: Phase8NormalizeErrorReasonCode,
        field: impl Into<String>,
    ) -> Self {
        Self {
            result_id: result_id.into(),
            policy_hash: None,
            error: Phase8StructuredError::normalize_failure(reason_code, field),
        }
    }

    pub fn with_policy_hash(mut self, hash: Hash) -> Self {
        self.policy_hash = Some(hash);
        self
    }

    pub fn with_error_expected_hash(mut self, hash: Hash) -> Self {
        self.error.expected_hash = Some(hash);
        self
    }

    pub fn with_error_actual_hash(mut self, hash: Hash) -> Self {
        self.error.actual_hash = Some(hash);
        self
    }

    pub fn with_error_expected_value(mut self, value: impl Into<String>) -> Self {
        self.error.expected_value = Some(value.into());
        self
    }

    pub fn with_error_actual_value(mut self, value: impl Into<String>) -> Self {
        self.error.actual_value = Some(value.into());
        self
    }

    pub const fn error(&self) -> &Phase8StructuredError {
        &self.error
    }

    pub fn result_hash(&self) -> Hash {
        phase8_sha256(self.hash_input_canonical_json().as_bytes())
    }

    pub fn hash_input_canonical_json(&self) -> String {
        canonical_json_object_from_pairs(self.base_canonical_json_pairs())
    }

    pub fn canonical_json(&self) -> String {
        let result_hash = self.result_hash();
        let mut pairs = self.base_canonical_json_pairs();
        pairs.push((
            "result_id".to_owned(),
            phase8_json_string_literal(&self.result_id),
        ));
        pairs.push((
            "result_hash".to_owned(),
            phase8_json_string_literal(&format_hash_string(&result_hash)),
        ));
        canonical_json_object_from_pairs(pairs)
    }

    fn base_canonical_json_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = vec![
            ("error".to_owned(), self.error.canonical_json()),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_NORMALIZE_ERROR_RESULT_SCHEMA),
            ),
            ("status".to_owned(), phase8_json_string_literal("failed")),
        ];
        push_optional_hash_pair(&mut pairs, "policy_hash", self.policy_hash);
        pairs
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8CommandName {
    RequestMaterialize,
    ChallengeGenerate,
    ChallengeMaterializeRequests,
    ChallengeReplay,
    ChallengeCoverageSummary,
    AuxiliaryAxiomPolicy,
    AuxiliaryReproducibility,
    AuxiliaryImportCertificateHash,
    ReleaseStageBundleInputs,
    ReleaseBundle,
    ReleaseValidateBundle,
    TrainingExport,
}

impl Phase8CommandName {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestMaterialize => "request materialize",
            Self::ChallengeGenerate => "challenge generate",
            Self::ChallengeMaterializeRequests => "challenge materialize-requests",
            Self::ChallengeReplay => "challenge replay",
            Self::ChallengeCoverageSummary => "challenge coverage-summary",
            Self::AuxiliaryAxiomPolicy => "auxiliary axiom-policy",
            Self::AuxiliaryReproducibility => "auxiliary reproducibility",
            Self::AuxiliaryImportCertificateHash => "auxiliary import-certificate-hash",
            Self::ReleaseStageBundleInputs => "release stage-bundle-inputs",
            Self::ReleaseBundle => "release bundle",
            Self::ReleaseValidateBundle => "release validate-bundle",
            Self::TrainingExport => "training export",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8CommandError {
    pub command: Phase8CommandName,
    pub reason_code: String,
    pub field: Option<String>,
    pub expected_hash: Option<Hash>,
    pub actual_hash: Option<Hash>,
    pub expected_value: Option<String>,
    pub actual_value: Option<String>,
    pub diagnostics: Vec<String>,
}

impl Phase8CommandError {
    pub fn new(command: Phase8CommandName, reason_code: impl Into<String>) -> Self {
        Self {
            command,
            reason_code: reason_code.into(),
            field: None,
            expected_hash: None,
            actual_hash: None,
            expected_value: None,
            actual_value: None,
            diagnostics: Vec::new(),
        }
    }

    pub fn canonical_json(&self) -> String {
        let mut pairs = vec![
            (
                "command".to_owned(),
                phase8_json_string_literal(self.command.as_str()),
            ),
            (
                "reason_code".to_owned(),
                phase8_json_string_literal(&self.reason_code),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_COMMAND_ERROR_SCHEMA),
            ),
        ];
        push_optional_string_pair(&mut pairs, "field", self.field.as_deref());
        push_optional_hash_pair(&mut pairs, "expected_hash", self.expected_hash);
        push_optional_hash_pair(&mut pairs, "actual_hash", self.actual_hash);
        push_optional_string_pair(&mut pairs, "expected_value", self.expected_value.as_deref());
        push_optional_string_pair(&mut pairs, "actual_value", self.actual_value.as_deref());
        if !self.diagnostics.is_empty() {
            pairs.push((
                "diagnostics".to_owned(),
                phase8_json_string_array(&self.diagnostics),
            ));
        }
        canonical_json_object_from_pairs(pairs)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8ApiErrorReasonCode {
    ApiJsonInvalid,
    ApiRequestSchemaInvalid,
    ApiPathOutsideWorkspace,
    ApiEndpointNotFound,
    ApiMethodNotAllowed,
}

impl Phase8ApiErrorReasonCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ApiJsonInvalid => "api_json_invalid",
            Self::ApiRequestSchemaInvalid => "api_request_schema_invalid",
            Self::ApiPathOutsideWorkspace => "api_path_outside_workspace",
            Self::ApiEndpointNotFound => "api_endpoint_not_found",
            Self::ApiMethodNotAllowed => "api_method_not_allowed",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ApiError {
    pub endpoint: String,
    pub reason_code: Phase8ApiErrorReasonCode,
    pub field: String,
    pub expected_value: String,
    pub actual_value: String,
}

impl Phase8ApiError {
    pub fn new(
        endpoint: impl Into<String>,
        reason_code: Phase8ApiErrorReasonCode,
        field: impl Into<String>,
        expected_value: impl Into<String>,
        actual_value: impl Into<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            reason_code,
            field: field.into(),
            expected_value: expected_value.into(),
            actual_value: actual_value.into(),
        }
    }

    pub fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "actual_value".to_owned(),
                phase8_json_string_literal(&self.actual_value),
            ),
            (
                "endpoint".to_owned(),
                phase8_json_string_literal(&self.endpoint),
            ),
            (
                "expected_value".to_owned(),
                phase8_json_string_literal(&self.expected_value),
            ),
            ("field".to_owned(), phase8_json_string_literal(&self.field)),
            (
                "reason_code".to_owned(),
                phase8_json_string_literal(self.reason_code.as_str()),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_API_ERROR_SCHEMA),
            ),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8PolicyValidationError {
    pub field: String,
    pub expected_value: String,
    pub actual_value: String,
}

impl Phase8PolicyValidationError {
    fn new(
        field: impl Into<String>,
        expected_value: impl Into<String>,
        actual_value: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            expected_value: expected_value.into(),
            actual_value: actual_value.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8TrustMode {
    Pr,
    Nightly,
    Release,
    HighTrust,
}

impl Phase8TrustMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pr => "pr",
            Self::Nightly => "nightly",
            Self::Release => "release",
            Self::HighTrust => "high-trust",
        }
    }

    pub const fn required_checker_profiles(self) -> &'static [&'static str] {
        match self {
            Self::Pr => &["reference"],
            Self::Nightly => &["reference", "external"],
            Self::Release => &["fast-kernel", "reference", "external"],
            Self::HighTrust => &[
                "fast-kernel",
                "reference",
                "external",
                "high-trust-reference",
            ],
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "pr" => Some(Self::Pr),
            "nightly" => Some(Self::Nightly),
            "release" => Some(Self::Release),
            "high-trust" => Some(Self::HighTrust),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8RunnerPolicyReference {
    pub path: String,
    pub hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8CheckerAllowlistEntry {
    pub profile: String,
    pub checker_id: String,
    pub binary_id: String,
    pub binary_hash: Hash,
    pub build_hash: Hash,
    pub allowed_args: Vec<String>,
}

impl Phase8CheckerAllowlistEntry {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "allowed_args".to_owned(),
                phase8_json_string_array(&self.allowed_args),
            ),
            (
                "binary_hash".to_owned(),
                phase8_hash_json_literal(&self.binary_hash),
            ),
            (
                "binary_id".to_owned(),
                phase8_json_string_literal(&self.binary_id),
            ),
            (
                "build_hash".to_owned(),
                phase8_hash_json_literal(&self.build_hash),
            ),
            (
                "checker_id".to_owned(),
                phase8_json_string_literal(&self.checker_id),
            ),
            (
                "profile".to_owned(),
                phase8_json_string_literal(&self.profile),
            ),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8CheckerIdentityManifestReference {
    pub path: String,
    pub manifest_hash: Hash,
}

impl Phase8CheckerIdentityManifestReference {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            ("kind".to_owned(), phase8_json_string_literal("file")),
            (
                "manifest_hash".to_owned(),
                phase8_hash_json_literal(&self.manifest_hash),
            ),
            ("path".to_owned(), phase8_json_string_literal(&self.path)),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8RunnerImportPolicy {
    pub mode: String,
    pub network: String,
    pub require_import_lock_hash: bool,
}

impl Phase8RunnerImportPolicy {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            ("mode".to_owned(), phase8_json_string_literal(&self.mode)),
            (
                "network".to_owned(),
                phase8_json_string_literal(&self.network),
            ),
            (
                "require_import_lock_hash".to_owned(),
                self.require_import_lock_hash.to_string(),
            ),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8RunnerAxiomPolicy {
    pub path: String,
    pub hash: Hash,
}

impl Phase8RunnerAxiomPolicy {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            ("hash".to_owned(), phase8_hash_json_literal(&self.hash)),
            ("path".to_owned(), phase8_json_string_literal(&self.path)),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8RunnerBudget {
    pub max_steps: u64,
    pub max_memory_mb: u64,
    pub timeout_ms: u64,
}

impl Phase8RunnerBudget {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            ("max_memory_mb".to_owned(), self.max_memory_mb.to_string()),
            ("max_steps".to_owned(), self.max_steps.to_string()),
            ("timeout_ms".to_owned(), self.timeout_ms.to_string()),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8RunnerPolicy {
    pub id: String,
    pub version: u64,
    pub trust_mode: Phase8TrustMode,
    pub required_checker_profiles: Vec<String>,
    pub optional_checker_profiles: Vec<String>,
    pub checker_allowlist: Vec<Phase8CheckerAllowlistEntry>,
    pub checker_identity_manifest: Option<Phase8CheckerIdentityManifestReference>,
    pub import_policy: Phase8RunnerImportPolicy,
    pub axiom_policy: Phase8RunnerAxiomPolicy,
    pub budgets: BTreeMap<String, Phase8RunnerBudget>,
}

impl Phase8RunnerPolicy {
    pub fn canonical_json(&self) -> String {
        let mut pairs = vec![
            (
                "axiom_policy".to_owned(),
                self.axiom_policy.canonical_json(),
            ),
            (
                "budgets".to_owned(),
                canonical_json_object_from_pairs(
                    self.budgets
                        .iter()
                        .map(|(profile, budget)| (profile.clone(), budget.canonical_json()))
                        .collect(),
                ),
            ),
            (
                "checker_allowlist".to_owned(),
                canonical_json_array(
                    self.checker_allowlist
                        .iter()
                        .map(Phase8CheckerAllowlistEntry::canonical_json)
                        .collect(),
                ),
            ),
            ("id".to_owned(), phase8_json_string_literal(&self.id)),
            (
                "import_policy".to_owned(),
                self.import_policy.canonical_json(),
            ),
            (
                "on_missing_required_checker".to_owned(),
                phase8_json_string_literal("fail"),
            ),
            (
                "on_profile_requested_by_ai".to_owned(),
                phase8_json_string_literal("ignore_unless_policy_allows"),
            ),
            (
                "on_resource_exhausted".to_owned(),
                phase8_json_string_literal("fail"),
            ),
            (
                "optional_checker_profiles".to_owned(),
                phase8_json_string_array(&self.optional_checker_profiles),
            ),
            (
                "required_checker_profiles".to_owned(),
                phase8_json_string_array(&self.required_checker_profiles),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_RUNNER_POLICY_SCHEMA),
            ),
            (
                "trust_mode".to_owned(),
                phase8_json_string_literal(self.trust_mode.as_str()),
            ),
            ("version".to_owned(), self.version.to_string()),
        ];
        if let Some(reference) = &self.checker_identity_manifest {
            pairs.push((
                "checker_identity_manifest".to_owned(),
                reference.canonical_json(),
            ));
        }
        canonical_json_object_from_pairs(pairs)
    }

    pub fn policy_hash(&self) -> Hash {
        phase8_sha256(self.canonical_json().as_bytes())
    }

    pub fn selected_checker_policy(&self, profile: &str) -> Option<&Phase8CheckerAllowlistEntry> {
        self.checker_allowlist
            .iter()
            .find(|entry| entry.profile == profile)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Phase8RawRunnerBudget {
    max_steps: String,
    max_memory_mb: String,
    timeout_ms: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8CheckerBinaryRegistryRootKind {
    Workspace,
    RunnerInstall,
}

impl Phase8CheckerBinaryRegistryRootKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Workspace => "workspace",
            Self::RunnerInstall => "runner_install",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "workspace" => Some(Self::Workspace),
            "runner_install" => Some(Self::RunnerInstall),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8CheckerBinaryRegistryEntry {
    pub binary_id: String,
    pub path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8CheckerBinaryRegistry {
    pub root_kind: Phase8CheckerBinaryRegistryRootKind,
    pub entries: Vec<Phase8CheckerBinaryRegistryEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8CheckerIdentityGeneratedBy {
    pub runner_id: String,
    pub runner_version: String,
    pub runner_build_hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8CheckerIdentityEntry {
    pub profile: String,
    pub checker_id: String,
    pub checker_version: Option<String>,
    pub binary_id: String,
    pub binary_hash: Hash,
    pub build_hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8CheckerIdentityManifest {
    pub generated_by: Phase8CheckerIdentityGeneratedBy,
    pub checkers: Vec<Phase8CheckerIdentityEntry>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8PolicyFailureReasonCode {
    RunnerPolicyReferenceInvalid,
    RunnerPolicyInvalid,
    CheckerBinaryFileUnreadable,
    CheckerBinaryHashMismatch,
    CheckerIdentityManifestInvalid,
    CheckerIdentityMissing,
    CheckerIdentityMismatch,
    CheckerBuildHashMismatch,
}

impl Phase8PolicyFailureReasonCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RunnerPolicyReferenceInvalid => "runner_policy_reference_invalid",
            Self::RunnerPolicyInvalid => "runner_policy_invalid",
            Self::CheckerBinaryFileUnreadable => "checker_binary_file_unreadable",
            Self::CheckerBinaryHashMismatch => "checker_binary_hash_mismatch",
            Self::CheckerIdentityManifestInvalid => "checker_identity_manifest_invalid",
            Self::CheckerIdentityMissing => "checker_identity_missing",
            Self::CheckerIdentityMismatch => "checker_identity_mismatch",
            Self::CheckerBuildHashMismatch => "checker_build_hash_mismatch",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8PolicyFailure {
    pub reason_code: Phase8PolicyFailureReasonCode,
    pub field: Box<str>,
    pub expected_value: Option<Box<str>>,
    pub actual_value: Option<Box<str>>,
    pub expected_hash: Option<Hash>,
    pub actual_hash: Option<Hash>,
}

impl Phase8PolicyFailure {
    fn value_failure(
        reason_code: Phase8PolicyFailureReasonCode,
        field: impl Into<String>,
        expected_value: impl Into<String>,
        actual_value: impl Into<String>,
    ) -> Self {
        let field = field.into().into_boxed_str();
        let expected_value = expected_value.into().into_boxed_str();
        let actual_value = actual_value.into().into_boxed_str();
        Self {
            reason_code,
            field,
            expected_value: Some(expected_value),
            actual_value: Some(actual_value),
            expected_hash: None,
            actual_hash: None,
        }
    }

    fn hash_failure(
        reason_code: Phase8PolicyFailureReasonCode,
        field: impl Into<String>,
        expected_hash: Hash,
        actual_hash: Hash,
    ) -> Self {
        let field = field.into().into_boxed_str();
        Self {
            reason_code,
            field,
            expected_value: None,
            actual_value: None,
            expected_hash: Some(expected_hash),
            actual_hash: Some(actual_hash),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8RunnerDynamicArgs {
    pub imports_manifest: String,
    pub imports_manifest_hash: Hash,
    pub trust_mode: String,
    pub axiom_policy_path: String,
    pub axiom_policy_hash: Hash,
    pub max_steps: u64,
    pub max_memory_mb: u64,
    pub timeout_ms: u64,
    pub certificate_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ResolvedCheckerExecutable {
    pub binary_id: String,
    pub path: String,
    pub binary_hash: Hash,
}

pub fn parse_phase8_runner_policy(
    source: &str,
) -> Result<Phase8RunnerPolicy, Phase8PolicyValidationError> {
    let document = JsonDocument::parse(source)
        .map_err(|_| Phase8PolicyValidationError::new("$", "valid_json", "invalid_json"))?;
    parse_phase8_runner_policy_value(document.root())
}

pub fn phase8_runner_policy_hash(source: &str) -> Result<Hash, Phase8PolicyValidationError> {
    Ok(parse_phase8_runner_policy(source)?.policy_hash())
}

pub fn parse_phase8_runner_policy_reference(
    source: &str,
) -> Result<Phase8RunnerPolicyReference, Phase8PolicyValidationError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        Phase8PolicyValidationError::new("policy", "RunnerPolicyReference", "invalid_json")
    })?;
    parse_phase8_runner_policy_reference_value(document.root(), "policy")
}

pub fn parse_phase8_checker_binary_registry(
    source: &str,
) -> Result<Phase8CheckerBinaryRegistry, Phase8PolicyValidationError> {
    let document = JsonDocument::parse(source)
        .map_err(|_| Phase8PolicyValidationError::new("$", "valid_json", "invalid_json"))?;
    parse_phase8_checker_binary_registry_value(document.root())
}

pub fn parse_phase8_checker_identity_manifest(
    source: &str,
) -> Result<Phase8CheckerIdentityManifest, Phase8PolicyValidationError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        Phase8PolicyValidationError::new("checker_identity_manifest", "valid_json", "invalid_json")
    })?;
    parse_phase8_checker_identity_manifest_value(document.root())
}

pub fn phase8_resolve_checker_executable(
    registry: &Phase8CheckerBinaryRegistry,
    selected: &Phase8CheckerAllowlistEntry,
    actual_binary_hash: Hash,
) -> Result<Phase8ResolvedCheckerExecutable, Phase8PolicyFailure> {
    let Some(entry) = registry
        .entries
        .iter()
        .find(|entry| entry.binary_id == selected.binary_id)
    else {
        return Err(Phase8PolicyFailure::value_failure(
            Phase8PolicyFailureReasonCode::CheckerBinaryFileUnreadable,
            "checker.binary_id",
            "readable_executable",
            "binary_id_not_found",
        ));
    };

    if actual_binary_hash != selected.binary_hash {
        return Err(Phase8PolicyFailure::hash_failure(
            Phase8PolicyFailureReasonCode::CheckerBinaryHashMismatch,
            "checker.binary_hash",
            selected.binary_hash,
            actual_binary_hash,
        ));
    }

    Ok(Phase8ResolvedCheckerExecutable {
        binary_id: entry.binary_id.clone(),
        path: entry.path.clone(),
        binary_hash: actual_binary_hash,
    })
}

pub fn phase8_validate_selected_checker_identity_manifest(
    selected: &Phase8CheckerAllowlistEntry,
    manifest: &Phase8CheckerIdentityManifest,
) -> Result<(), Phase8PolicyFailure> {
    let Some(entry) = manifest
        .checkers
        .iter()
        .find(|entry| entry.profile == selected.profile)
    else {
        return Err(Phase8PolicyFailure::value_failure(
            Phase8PolicyFailureReasonCode::CheckerIdentityMissing,
            "checker_identity_manifest.checkers[]",
            "selected_checker_profile",
            "missing",
        ));
    };

    if entry.checker_id != selected.checker_id {
        return Err(Phase8PolicyFailure::value_failure(
            Phase8PolicyFailureReasonCode::CheckerIdentityMismatch,
            "checker_identity_manifest.checkers[].checker_id",
            &selected.checker_id,
            &entry.checker_id,
        ));
    }
    if entry.binary_id != selected.binary_id {
        return Err(Phase8PolicyFailure::value_failure(
            Phase8PolicyFailureReasonCode::CheckerIdentityMismatch,
            "checker_identity_manifest.checkers[].binary_id",
            &selected.binary_id,
            &entry.binary_id,
        ));
    }
    if entry.binary_hash != selected.binary_hash {
        return Err(Phase8PolicyFailure::hash_failure(
            Phase8PolicyFailureReasonCode::CheckerBinaryHashMismatch,
            "checker_identity_manifest.checkers[].binary_hash",
            selected.binary_hash,
            entry.binary_hash,
        ));
    }
    if entry.build_hash != selected.build_hash {
        return Err(Phase8PolicyFailure::hash_failure(
            Phase8PolicyFailureReasonCode::CheckerBuildHashMismatch,
            "checker_identity_manifest.checkers[].build_hash",
            selected.build_hash,
            entry.build_hash,
        ));
    }

    Ok(())
}

pub fn phase8_checker_argv(
    executable_path: &str,
    selected: &Phase8CheckerAllowlistEntry,
    dynamic: &Phase8RunnerDynamicArgs,
) -> Vec<String> {
    let mut argv = Vec::new();
    argv.push(executable_path.to_owned());
    argv.extend(selected.allowed_args.iter().cloned());
    argv.push("--imports".to_owned());
    argv.push(dynamic.imports_manifest.clone());
    argv.push("--imports-hash".to_owned());
    argv.push(format_hash_string(&dynamic.imports_manifest_hash));
    argv.push("--trust-mode".to_owned());
    argv.push(dynamic.trust_mode.clone());
    argv.push("--axiom-policy".to_owned());
    argv.push(dynamic.axiom_policy_path.clone());
    argv.push("--axiom-policy-hash".to_owned());
    argv.push(format_hash_string(&dynamic.axiom_policy_hash));
    argv.push("--max-steps".to_owned());
    argv.push(dynamic.max_steps.to_string());
    argv.push("--max-memory-mb".to_owned());
    argv.push(dynamic.max_memory_mb.to_string());
    argv.push("--timeout-ms".to_owned());
    argv.push(dynamic.timeout_ms.to_string());
    argv.push(dynamic.certificate_path.clone());
    argv
}

pub fn phase8_runner_fixed_environment() -> Vec<(String, String)> {
    PHASE8_RUNNER_FIXED_ENVIRONMENT
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect()
}

fn parse_phase8_runner_policy_value(
    value: &JsonValue<'_>,
) -> Result<Phase8RunnerPolicy, Phase8PolicyValidationError> {
    let members = object_members_or_policy_error(value, "$", "object")?;

    required_fixed_string_field(
        members,
        "schema",
        "schema",
        PHASE8_RUNNER_POLICY_SCHEMA,
        PHASE8_RUNNER_POLICY_SCHEMA,
    )?;
    let id = required_string_field(members, "id", "id", "runner_policy_id")?;
    let version_raw = required_integer_raw_field(members, "version", "version", "positive_i64")?;
    let trust_mode_raw = required_string_field(members, "trust_mode", "trust_mode", "trust_mode")?;
    let trust_mode = Phase8TrustMode::parse(&trust_mode_raw).ok_or_else(|| {
        Phase8PolicyValidationError::new("trust_mode", "trust_mode", "invalid_enum")
    })?;
    let required_checker_profiles = required_string_array_field(
        members,
        "required_checker_profiles",
        "required_checker_profiles",
        "checker_profile_name",
    )?;
    let optional_checker_profiles = required_string_array_field(
        members,
        "optional_checker_profiles",
        "optional_checker_profiles",
        "checker_profile_name",
    )?;
    let checker_allowlist = parse_checker_allowlist_field(members)?;
    let checker_identity_manifest = optional_checker_identity_manifest_reference(members)?;
    let import_policy = parse_runner_import_policy_field(members)?;
    let axiom_policy = parse_runner_axiom_policy_field(members)?;
    let raw_budgets = parse_runner_budgets_field(members)?;
    required_fixed_string_field(
        members,
        "on_resource_exhausted",
        "on_resource_exhausted",
        "fail",
        "fail",
    )?;
    required_fixed_string_field(
        members,
        "on_missing_required_checker",
        "on_missing_required_checker",
        "fail",
        "fail",
    )?;
    required_fixed_string_field(
        members,
        "on_profile_requested_by_ai",
        "on_profile_requested_by_ai",
        "ignore_unless_policy_allows",
        "ignore_unless_policy_allows",
    )?;
    reject_unknown_fields(members, RUNNER_POLICY_FIELDS, "$")?;

    if !phase8_valid_runner_policy_id(&id) {
        return Err(Phase8PolicyValidationError::new(
            "id",
            "runner_policy_id",
            "invalid_name_format",
        ));
    }
    let version = parse_positive_i64_domain(&version_raw, "version")?;
    validate_profiles_domain(
        trust_mode,
        &required_checker_profiles,
        &optional_checker_profiles,
        &checker_allowlist,
        &raw_budgets,
    )?;
    validate_checker_allowlist_domain(
        &required_checker_profiles,
        &optional_checker_profiles,
        &checker_allowlist,
    )?;
    let budgets = validate_runner_budgets_domain(
        &required_checker_profiles,
        &optional_checker_profiles,
        raw_budgets,
    )?;

    Ok(Phase8RunnerPolicy {
        id,
        version,
        trust_mode,
        required_checker_profiles,
        optional_checker_profiles,
        checker_allowlist,
        checker_identity_manifest,
        import_policy,
        axiom_policy,
        budgets,
    })
}

const RUNNER_POLICY_FIELDS: &[&str] = &[
    "schema",
    "id",
    "version",
    "trust_mode",
    "required_checker_profiles",
    "optional_checker_profiles",
    "checker_allowlist",
    "checker_identity_manifest",
    "import_policy",
    "axiom_policy",
    "budgets",
    "on_resource_exhausted",
    "on_missing_required_checker",
    "on_profile_requested_by_ai",
];

const CHECKER_ALLOWLIST_FIELDS: &[&str] = &[
    "profile",
    "checker_id",
    "binary_id",
    "binary_hash",
    "build_hash",
    "allowed_args",
];

const CHECKER_IDENTITY_MANIFEST_FIELDS: &[&str] = &["schema", "generated_by", "checkers"];
const CHECKER_IDENTITY_GENERATED_BY_FIELDS: &[&str] =
    &["runner_id", "runner_version", "runner_build_hash"];
const CHECKER_IDENTITY_ENTRY_FIELDS: &[&str] = &[
    "profile",
    "checker_id",
    "checker_version",
    "binary_id",
    "binary_hash",
    "build_hash",
];

fn parse_phase8_runner_policy_reference_value(
    value: &JsonValue<'_>,
    field_prefix: &str,
) -> Result<Phase8RunnerPolicyReference, Phase8PolicyValidationError> {
    let members = object_members_or_policy_error(value, field_prefix, "RunnerPolicyReference")?;
    required_fixed_string_field(
        members,
        "kind",
        &format!("{field_prefix}.kind"),
        "file",
        "file",
    )?;
    let path = required_string_field(
        members,
        "path",
        &format!("{field_prefix}.path"),
        "workspace_relative_path",
    )?;
    if !phase8_valid_workspace_relative_path(&path) {
        return Err(Phase8PolicyValidationError::new(
            format!("{field_prefix}.path"),
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    let hash = required_hash_field(
        members,
        "hash",
        &format!("{field_prefix}.hash"),
        "sha256:<lower-hex>",
    )?;
    reject_unknown_fields(members, &["kind", "path", "hash"], field_prefix)?;
    Ok(Phase8RunnerPolicyReference { path, hash })
}

fn parse_checker_allowlist_field(
    members: &[JsonMember<'_>],
) -> Result<Vec<Phase8CheckerAllowlistEntry>, Phase8PolicyValidationError> {
    let value = required_field_value(members, "checker_allowlist", "checker_allowlist", "array")?;
    let Some(entries) = value.array_elements() else {
        return Err(wrong_type_error("checker_allowlist", "array", value.kind()));
    };
    let mut out = Vec::new();
    for (index, entry_value) in entries.iter().enumerate() {
        let path = format!("checker_allowlist[{index}]");
        let entry_members = object_members_or_policy_error(entry_value, &path, "object")?;
        let profile = required_string_field(
            entry_members,
            "profile",
            &format!("{path}.profile"),
            "checker_profile_name",
        )?;
        let checker_id = required_string_field(
            entry_members,
            "checker_id",
            &format!("{path}.checker_id"),
            "checker_id",
        )?;
        let binary_id = required_string_field(
            entry_members,
            "binary_id",
            &format!("{path}.binary_id"),
            "checker_binary_id",
        )?;
        let binary_hash = required_hash_field(
            entry_members,
            "binary_hash",
            &format!("{path}.binary_hash"),
            "sha256:<lower-hex>",
        )?;
        let build_hash = required_hash_field(
            entry_members,
            "build_hash",
            &format!("{path}.build_hash"),
            "sha256:<lower-hex>",
        )?;
        let allowed_args = required_string_array_field(
            entry_members,
            "allowed_args",
            &format!("{path}.allowed_args"),
            "static_checker_option_without_runner_owned_dynamic_args",
        )?;
        reject_unknown_fields(entry_members, CHECKER_ALLOWLIST_FIELDS, &path)?;
        out.push(Phase8CheckerAllowlistEntry {
            profile,
            checker_id,
            binary_id,
            binary_hash,
            build_hash,
            allowed_args,
        });
    }
    Ok(out)
}

fn optional_checker_identity_manifest_reference(
    members: &[JsonMember<'_>],
) -> Result<Option<Phase8CheckerIdentityManifestReference>, Phase8PolicyValidationError> {
    let Some(value) = unique_optional_field_value(
        members,
        "checker_identity_manifest",
        "checker_identity_manifest",
    )?
    else {
        return Ok(None);
    };
    let entry_members =
        object_members_or_policy_error(value, "checker_identity_manifest", "object")?;
    required_fixed_string_field(
        entry_members,
        "kind",
        "checker_identity_manifest.kind",
        "file",
        "file",
    )?;
    let path = required_string_field(
        entry_members,
        "path",
        "checker_identity_manifest.path",
        "workspace_relative_path",
    )?;
    if !phase8_valid_workspace_relative_path(&path) {
        return Err(Phase8PolicyValidationError::new(
            "checker_identity_manifest.path",
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    let manifest_hash = required_hash_field(
        entry_members,
        "manifest_hash",
        "checker_identity_manifest.manifest_hash",
        "sha256:<lower-hex>",
    )?;
    reject_unknown_fields(
        entry_members,
        &["kind", "path", "manifest_hash"],
        "checker_identity_manifest",
    )?;
    Ok(Some(Phase8CheckerIdentityManifestReference {
        path,
        manifest_hash,
    }))
}

fn parse_runner_import_policy_field(
    members: &[JsonMember<'_>],
) -> Result<Phase8RunnerImportPolicy, Phase8PolicyValidationError> {
    let value = required_field_value(members, "import_policy", "import_policy", "object")?;
    let entry_members = object_members_or_policy_error(value, "import_policy", "object")?;
    let mode = required_string_field(entry_members, "mode", "import_policy.mode", "locked_store")?;
    if mode != "locked_store" {
        return Err(Phase8PolicyValidationError::new(
            "import_policy.mode",
            "locked_store",
            "invalid_enum",
        ));
    }
    let network = required_string_field(
        entry_members,
        "network",
        "import_policy.network",
        "forbidden",
    )?;
    if network != "forbidden" {
        return Err(Phase8PolicyValidationError::new(
            "import_policy.network",
            "forbidden",
            "invalid_enum",
        ));
    }
    let require_import_lock_hash = required_bool_field(
        entry_members,
        "require_import_lock_hash",
        "import_policy.require_import_lock_hash",
        "true",
    )?;
    if !require_import_lock_hash {
        return Err(Phase8PolicyValidationError::new(
            "import_policy.require_import_lock_hash",
            "true",
            "invalid_fixed_value",
        ));
    }
    reject_unknown_fields(
        entry_members,
        &["mode", "network", "require_import_lock_hash"],
        "import_policy",
    )?;
    Ok(Phase8RunnerImportPolicy {
        mode,
        network,
        require_import_lock_hash,
    })
}

fn parse_runner_axiom_policy_field(
    members: &[JsonMember<'_>],
) -> Result<Phase8RunnerAxiomPolicy, Phase8PolicyValidationError> {
    let value = required_field_value(members, "axiom_policy", "axiom_policy", "object")?;
    let entry_members = object_members_or_policy_error(value, "axiom_policy", "object")?;
    let path = required_string_field(
        entry_members,
        "path",
        "axiom_policy.path",
        "workspace_relative_path",
    )?;
    if !phase8_valid_workspace_relative_path(&path) {
        return Err(Phase8PolicyValidationError::new(
            "axiom_policy.path",
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    let hash = required_hash_field(
        entry_members,
        "hash",
        "axiom_policy.hash",
        "sha256:<lower-hex>",
    )?;
    reject_unknown_fields(entry_members, &["path", "hash"], "axiom_policy")?;
    Ok(Phase8RunnerAxiomPolicy { path, hash })
}

fn parse_runner_budgets_field(
    members: &[JsonMember<'_>],
) -> Result<BTreeMap<String, Phase8RawRunnerBudget>, Phase8PolicyValidationError> {
    let value = required_field_value(members, "budgets", "budgets", "object")?;
    let entry_members = object_members_or_policy_error(value, "budgets", "object")?;
    let mut out = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for member in entry_members {
        if !seen.insert(member.key().to_owned()) {
            return Err(Phase8PolicyValidationError::new(
                format_budget_key_path(member.key()),
                "unique_object_keys",
                "duplicate_field",
            ));
        }
        let path = format_budget_key_path(member.key());
        let budget_members = object_members_or_policy_error(member.value(), &path, "object")?;
        let max_steps = required_integer_raw_field(
            budget_members,
            "max_steps",
            &format!("{path}.max_steps"),
            "positive_i64",
        )?;
        let max_memory_mb = required_integer_raw_field(
            budget_members,
            "max_memory_mb",
            &format!("{path}.max_memory_mb"),
            "positive_i64",
        )?;
        let timeout_ms = required_integer_raw_field(
            budget_members,
            "timeout_ms",
            &format!("{path}.timeout_ms"),
            "positive_i64",
        )?;
        reject_unknown_fields(
            budget_members,
            &["max_steps", "max_memory_mb", "timeout_ms"],
            &path,
        )?;
        out.insert(
            member.key().to_owned(),
            Phase8RawRunnerBudget {
                max_steps,
                max_memory_mb,
                timeout_ms,
            },
        );
    }
    Ok(out)
}

fn validate_profiles_domain(
    trust_mode: Phase8TrustMode,
    required: &[String],
    optional: &[String],
    checker_allowlist: &[Phase8CheckerAllowlistEntry],
    budgets: &BTreeMap<String, Phase8RawRunnerBudget>,
) -> Result<(), Phase8PolicyValidationError> {
    for (index, profile) in required.iter().enumerate() {
        if !phase8_valid_checker_profile_name(profile) {
            return Err(Phase8PolicyValidationError::new(
                format!("required_checker_profiles[{index}]"),
                "checker_profile_name",
                "invalid_name_format",
            ));
        }
    }
    for (index, profile) in optional.iter().enumerate() {
        if !phase8_valid_checker_profile_name(profile) {
            return Err(Phase8PolicyValidationError::new(
                format!("optional_checker_profiles[{index}]"),
                "checker_profile_name",
                "invalid_name_format",
            ));
        }
    }
    for (index, entry) in checker_allowlist.iter().enumerate() {
        if !phase8_valid_checker_profile_name(&entry.profile) {
            return Err(Phase8PolicyValidationError::new(
                format!("checker_allowlist[{index}].profile"),
                "checker_profile_name",
                "invalid_name_format",
            ));
        }
    }
    for profile in budgets.keys() {
        if !phase8_valid_checker_profile_name(profile) {
            return Err(Phase8PolicyValidationError::new(
                format_budget_key_path(profile),
                "checker_profile_name",
                "invalid_name_format",
            ));
        }
    }
    for (index, entry) in checker_allowlist.iter().enumerate() {
        if !phase8_valid_checker_id(&entry.checker_id) {
            return Err(Phase8PolicyValidationError::new(
                format!("checker_allowlist[{index}].checker_id"),
                "checker_id",
                "invalid_name_format",
            ));
        }
        if !phase8_valid_checker_id(&entry.binary_id) {
            return Err(Phase8PolicyValidationError::new(
                format!("checker_allowlist[{index}].binary_id"),
                "checker_binary_id",
                "invalid_name_format",
            ));
        }
    }

    let expected = trust_mode.required_checker_profiles();
    let actual = required.iter().map(String::as_str).collect::<Vec<_>>();
    if actual.len() != expected.len() || actual.iter().any(|profile| !expected.contains(profile)) {
        return Err(Phase8PolicyValidationError::new(
            "required_checker_profiles",
            format!("profiles_for_trust_mode:{}", trust_mode.as_str()),
            "profile_set_mismatch",
        ));
    }
    if actual != expected {
        return Err(Phase8PolicyValidationError::new(
            "required_checker_profiles",
            format!("profiles_for_trust_mode:{}", trust_mode.as_str()),
            "profile_order_mismatch",
        ));
    }
    Ok(())
}

fn validate_checker_allowlist_domain(
    required: &[String],
    optional: &[String],
    checker_allowlist: &[Phase8CheckerAllowlistEntry],
) -> Result<(), Phase8PolicyValidationError> {
    let required_set = required.iter().cloned().collect::<BTreeSet<_>>();
    let mut optional_seen = BTreeSet::new();
    for (index, profile) in optional.iter().enumerate() {
        if required_set.contains(profile) {
            return Err(Phase8PolicyValidationError::new(
                format!("optional_checker_profiles[{index}]"),
                "exclude_required_checker_profiles",
                "required_profile_in_optional",
            ));
        }
        if !optional_seen.insert(profile) {
            return Err(Phase8PolicyValidationError::new(
                format!("optional_checker_profiles[{index}]"),
                "unique_profiles",
                "duplicate_profile",
            ));
        }
    }

    let allowed_profiles = required
        .iter()
        .chain(optional.iter())
        .cloned()
        .collect::<BTreeSet<_>>();
    let allowlist_profiles = checker_allowlist
        .iter()
        .map(|entry| entry.profile.clone())
        .collect::<BTreeSet<_>>();
    if !allowed_profiles.is_subset(&allowlist_profiles) {
        return Err(Phase8PolicyValidationError::new(
            "checker_allowlist",
            "entry_for_each_required_and_optional_profile",
            "missing_checker_allowlist_entry",
        ));
    }
    for (index, entry) in checker_allowlist.iter().enumerate() {
        if !allowed_profiles.contains(&entry.profile) {
            return Err(Phase8PolicyValidationError::new(
                format!("checker_allowlist[{index}].profile"),
                "only_required_and_optional_profiles",
                "unexpected_checker_allowlist_entry",
            ));
        }
    }
    for index in 1..checker_allowlist.len() {
        if checker_allowlist[index].profile < checker_allowlist[index - 1].profile {
            return Err(Phase8PolicyValidationError::new(
                format!("checker_allowlist[{index}]"),
                "profile_bytewise_ascending",
                "order_violation",
            ));
        }
    }
    let mut profiles = BTreeSet::new();
    let mut binary_ids = BTreeSet::new();
    for (index, entry) in checker_allowlist.iter().enumerate() {
        if !profiles.insert(&entry.profile) {
            return Err(Phase8PolicyValidationError::new(
                format!("checker_allowlist[{index}].profile"),
                "unique_profiles",
                "duplicate_profile",
            ));
        }
        if !binary_ids.insert(&entry.binary_id) {
            return Err(Phase8PolicyValidationError::new(
                format!("checker_allowlist[{index}].binary_id"),
                "unique_binary_ids",
                "duplicate_binary_id",
            ));
        }
    }
    validate_allowed_args_domain(checker_allowlist)
}

fn validate_allowed_args_domain(
    checker_allowlist: &[Phase8CheckerAllowlistEntry],
) -> Result<(), Phase8PolicyValidationError> {
    for (checker_index, entry) in checker_allowlist.iter().enumerate() {
        for (arg_index, arg) in entry.allowed_args.iter().enumerate() {
            if !phase8_visible_ascii_nonempty(arg) || !arg.starts_with("--") {
                return Err(Phase8PolicyValidationError::new(
                    format!("checker_allowlist[{checker_index}].allowed_args[{arg_index}]"),
                    "static_checker_option_without_runner_owned_dynamic_args",
                    if arg.starts_with("--") {
                        "invalid_arg_text"
                    } else {
                        "positional_arg"
                    },
                ));
            }
            if arg == "--" {
                return Err(Phase8PolicyValidationError::new(
                    format!("checker_allowlist[{checker_index}].allowed_args[{arg_index}]"),
                    "static_checker_option_without_runner_owned_dynamic_args",
                    "end_of_options_marker",
                ));
            }
            if PHASE8_RUNNER_DYNAMIC_FLAGS
                .iter()
                .any(|flag| arg == flag || arg.starts_with(&format!("{flag}=")))
            {
                return Err(Phase8PolicyValidationError::new(
                    format!("checker_allowlist[{checker_index}].allowed_args[{arg_index}]"),
                    "static_checker_option_without_runner_owned_dynamic_args",
                    "reserved_dynamic_arg",
                ));
            }
        }
    }

    for (checker_index, entry) in checker_allowlist.iter().enumerate() {
        let json_positions = entry
            .allowed_args
            .iter()
            .enumerate()
            .filter_map(|(index, arg)| (arg == "--json").then_some(index))
            .collect::<Vec<_>>();
        if json_positions.is_empty() {
            return Err(Phase8PolicyValidationError::new(
                format!("checker_allowlist[{checker_index}].allowed_args"),
                "exactly_one_json_output_arg",
                "missing_required_json_arg",
            ));
        }
        if json_positions.len() > 1 {
            return Err(Phase8PolicyValidationError::new(
                format!(
                    "checker_allowlist[{checker_index}].allowed_args[{}]",
                    json_positions[1]
                ),
                "exactly_one_json_output_arg",
                "duplicate_required_json_arg",
            ));
        }
    }

    for (checker_index, entry) in checker_allowlist.iter().enumerate() {
        for (arg_index, arg) in entry.allowed_args.iter().enumerate() {
            if arg.starts_with("--json=")
                || arg == "--no-json"
                || arg == "--format"
                || arg.starts_with("--format=")
                || arg == "--output-format"
                || arg.starts_with("--output-format=")
            {
                return Err(Phase8PolicyValidationError::new(
                    format!("checker_allowlist[{checker_index}].allowed_args[{arg_index}]"),
                    "json_output_contract",
                    "json_output_arg_conflict",
                ));
            }
        }
    }
    Ok(())
}

fn validate_runner_budgets_domain(
    required: &[String],
    optional: &[String],
    raw_budgets: BTreeMap<String, Phase8RawRunnerBudget>,
) -> Result<BTreeMap<String, Phase8RunnerBudget>, Phase8PolicyValidationError> {
    let expected = required
        .iter()
        .chain(optional.iter())
        .cloned()
        .collect::<BTreeSet<_>>();
    let actual = raw_budgets.keys().cloned().collect::<BTreeSet<_>>();
    if !expected.is_subset(&actual) {
        return Err(Phase8PolicyValidationError::new(
            "budgets",
            "budget_for_each_required_and_optional_profile",
            "missing_budget_entry",
        ));
    }
    if let Some(profile) = actual.difference(&expected).next() {
        return Err(Phase8PolicyValidationError::new(
            format_budget_key_path(profile),
            "only_required_and_optional_profiles",
            "unexpected_budget_entry",
        ));
    }

    let mut budgets = BTreeMap::new();
    for (profile, raw) in raw_budgets {
        let base = format_budget_key_path(&profile);
        budgets.insert(
            profile,
            Phase8RunnerBudget {
                max_steps: parse_positive_i64_domain(&raw.max_steps, &format!("{base}.max_steps"))?,
                max_memory_mb: parse_positive_i64_domain(
                    &raw.max_memory_mb,
                    &format!("{base}.max_memory_mb"),
                )?,
                timeout_ms: parse_positive_i64_domain(
                    &raw.timeout_ms,
                    &format!("{base}.timeout_ms"),
                )?,
            },
        );
    }
    Ok(budgets)
}

fn parse_phase8_checker_binary_registry_value(
    value: &JsonValue<'_>,
) -> Result<Phase8CheckerBinaryRegistry, Phase8PolicyValidationError> {
    let members = object_members_or_policy_error(value, "$", "object")?;
    required_fixed_string_field(
        members,
        "schema",
        "schema",
        PHASE8_CHECKER_BINARY_REGISTRY_SCHEMA,
        PHASE8_CHECKER_BINARY_REGISTRY_SCHEMA,
    )?;
    let root_kind_raw =
        required_string_field(members, "root_kind", "root_kind", "registry_root_kind")?;
    let root_kind =
        Phase8CheckerBinaryRegistryRootKind::parse(&root_kind_raw).ok_or_else(|| {
            Phase8PolicyValidationError::new("root_kind", "registry_root_kind", "invalid_enum")
        })?;
    let entries_value = required_field_value(members, "entries", "entries", "array")?;
    let Some(entry_values) = entries_value.array_elements() else {
        return Err(wrong_type_error("entries", "array", entries_value.kind()));
    };
    let mut entries = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, entry_value) in entry_values.iter().enumerate() {
        let path = format!("entries[{index}]");
        let entry_members = object_members_or_policy_error(entry_value, &path, "object")?;
        let binary_id = required_string_field(
            entry_members,
            "binary_id",
            &format!("{path}.binary_id"),
            "checker_binary_id",
        )?;
        if !phase8_valid_checker_id(&binary_id) {
            return Err(Phase8PolicyValidationError::new(
                format!("{path}.binary_id"),
                "checker_binary_id",
                "invalid_name_format",
            ));
        }
        if !seen.insert(binary_id.clone()) {
            return Err(Phase8PolicyValidationError::new(
                format!("{path}.binary_id"),
                "unique_binary_ids",
                "duplicate_binary_id",
            ));
        }
        let entry_path = required_string_field(
            entry_members,
            "path",
            &format!("{path}.path"),
            "relative_path",
        )?;
        if !phase8_valid_workspace_relative_path(&entry_path) {
            return Err(Phase8PolicyValidationError::new(
                format!("{path}.path"),
                "relative_path",
                "invalid_path",
            ));
        }
        reject_unknown_fields(entry_members, &["binary_id", "path"], &path)?;
        entries.push(Phase8CheckerBinaryRegistryEntry {
            binary_id,
            path: entry_path,
        });
    }
    reject_unknown_fields(members, &["schema", "root_kind", "entries"], "$")?;
    Ok(Phase8CheckerBinaryRegistry { root_kind, entries })
}

fn parse_phase8_checker_identity_manifest_value(
    value: &JsonValue<'_>,
) -> Result<Phase8CheckerIdentityManifest, Phase8PolicyValidationError> {
    let members = object_members_or_policy_error(value, "checker_identity_manifest.$", "object")?;
    required_fixed_string_field(
        members,
        "schema",
        "checker_identity_manifest.schema",
        PHASE8_CHECKER_IDENTITY_MANIFEST_SCHEMA,
        PHASE8_CHECKER_IDENTITY_MANIFEST_SCHEMA,
    )?;
    let generated_by_value = required_field_value(
        members,
        "generated_by",
        "checker_identity_manifest.generated_by",
        "object",
    )?;
    let generated_by_members = object_members_or_policy_error(
        generated_by_value,
        "checker_identity_manifest.generated_by",
        "object",
    )?;
    let runner_id = required_string_field(
        generated_by_members,
        "runner_id",
        "checker_identity_manifest.generated_by.runner_id",
        "non_empty_string",
    )?;
    let runner_version = required_string_field(
        generated_by_members,
        "runner_version",
        "checker_identity_manifest.generated_by.runner_version",
        "non_empty_string",
    )?;
    let runner_build_hash = required_hash_field(
        generated_by_members,
        "runner_build_hash",
        "checker_identity_manifest.generated_by.runner_build_hash",
        "sha256:<lower-hex>",
    )?;
    reject_unknown_fields(
        generated_by_members,
        CHECKER_IDENTITY_GENERATED_BY_FIELDS,
        "checker_identity_manifest.generated_by",
    )?;

    let checkers_value = required_field_value(
        members,
        "checkers",
        "checker_identity_manifest.checkers",
        "array",
    )?;
    let Some(checker_values) = checkers_value.array_elements() else {
        return Err(wrong_type_error(
            "checker_identity_manifest.checkers",
            "array",
            checkers_value.kind(),
        ));
    };
    let mut checkers = Vec::new();
    for (index, checker_value) in checker_values.iter().enumerate() {
        let path = format!("checker_identity_manifest.checkers[{index}]");
        let checker_members = object_members_or_policy_error(checker_value, &path, "object")?;
        let profile = required_string_field(
            checker_members,
            "profile",
            &format!("{path}.profile"),
            "checker_profile_name",
        )?;
        let checker_id = required_string_field(
            checker_members,
            "checker_id",
            &format!("{path}.checker_id"),
            "checker_id",
        )?;
        let checker_version = optional_string_field(
            checker_members,
            "checker_version",
            &format!("{path}.checker_version"),
            "string",
        )?;
        let binary_id = required_string_field(
            checker_members,
            "binary_id",
            &format!("{path}.binary_id"),
            "checker_binary_id",
        )?;
        let binary_hash = required_hash_field(
            checker_members,
            "binary_hash",
            &format!("{path}.binary_hash"),
            "sha256:<lower-hex>",
        )?;
        let build_hash = required_hash_field(
            checker_members,
            "build_hash",
            &format!("{path}.build_hash"),
            "sha256:<lower-hex>",
        )?;
        reject_unknown_fields(checker_members, CHECKER_IDENTITY_ENTRY_FIELDS, &path)?;
        checkers.push(Phase8CheckerIdentityEntry {
            profile,
            checker_id,
            checker_version,
            binary_id,
            binary_hash,
            build_hash,
        });
    }
    reject_unknown_fields(
        members,
        CHECKER_IDENTITY_MANIFEST_FIELDS,
        "checker_identity_manifest",
    )?;

    if runner_id.is_empty() {
        return Err(Phase8PolicyValidationError::new(
            "checker_identity_manifest.generated_by.runner_id",
            "non_empty_string",
            "empty_string",
        ));
    }
    if runner_version.is_empty() {
        return Err(Phase8PolicyValidationError::new(
            "checker_identity_manifest.generated_by.runner_version",
            "non_empty_string",
            "empty_string",
        ));
    }

    for index in 1..checkers.len() {
        if checkers[index].profile < checkers[index - 1].profile {
            return Err(Phase8PolicyValidationError::new(
                format!("checker_identity_manifest.checkers[{index}].profile"),
                "profile_bytewise_ascending",
                "order_violation",
            ));
        }
    }
    let mut profiles = BTreeSet::new();
    let mut binary_ids = BTreeSet::new();
    for (index, entry) in checkers.iter().enumerate() {
        if !phase8_valid_checker_profile_name(&entry.profile) {
            return Err(Phase8PolicyValidationError::new(
                format!("checker_identity_manifest.checkers[{index}].profile"),
                "checker_profile_name",
                "invalid_name_format",
            ));
        }
        if !phase8_valid_checker_id(&entry.checker_id) {
            return Err(Phase8PolicyValidationError::new(
                format!("checker_identity_manifest.checkers[{index}].checker_id"),
                "checker_id",
                "invalid_name_format",
            ));
        }
        if !phase8_valid_checker_id(&entry.binary_id) {
            return Err(Phase8PolicyValidationError::new(
                format!("checker_identity_manifest.checkers[{index}].binary_id"),
                "checker_binary_id",
                "invalid_name_format",
            ));
        }
        if !profiles.insert(&entry.profile) {
            return Err(Phase8PolicyValidationError::new(
                format!("checker_identity_manifest.checkers[{index}].profile"),
                "unique_profiles",
                "duplicate_profile",
            ));
        }
        if !binary_ids.insert(&entry.binary_id) {
            return Err(Phase8PolicyValidationError::new(
                format!("checker_identity_manifest.checkers[{index}].binary_id"),
                "unique_binary_ids",
                "duplicate_binary_id",
            ));
        }
    }

    Ok(Phase8CheckerIdentityManifest {
        generated_by: Phase8CheckerIdentityGeneratedBy {
            runner_id,
            runner_version,
            runner_build_hash,
        },
        checkers,
    })
}

fn object_members_or_policy_error<'value, 'src>(
    value: &'value JsonValue<'src>,
    field: &str,
    expected: &str,
) -> Result<&'value [JsonMember<'src>], Phase8PolicyValidationError> {
    value.object_members().ok_or_else(|| {
        Phase8PolicyValidationError::new(
            field,
            expected,
            if value.kind() == JsonValueKind::Null {
                "null_not_allowed"
            } else {
                "wrong_type"
            },
        )
    })
}

fn required_field_value<'value, 'src>(
    members: &'value [JsonMember<'src>],
    name: &str,
    field: &str,
    expected: &str,
) -> Result<&'value JsonValue<'src>, Phase8PolicyValidationError> {
    if duplicate_member(members, name) {
        return Err(Phase8PolicyValidationError::new(
            field,
            "unique_object_keys",
            "duplicate_field",
        ));
    }
    let Some(member) = members.iter().find(|member| member.key() == name) else {
        return Err(Phase8PolicyValidationError::new(field, expected, "missing"));
    };
    if member.value().kind() == JsonValueKind::Null {
        return Err(Phase8PolicyValidationError::new(
            field,
            expected,
            "null_not_allowed",
        ));
    }
    Ok(member.value())
}

fn unique_optional_field_value<'value, 'src>(
    members: &'value [JsonMember<'src>],
    name: &str,
    field: &str,
) -> Result<Option<&'value JsonValue<'src>>, Phase8PolicyValidationError> {
    if duplicate_member(members, name) {
        return Err(Phase8PolicyValidationError::new(
            field,
            "unique_object_keys",
            "duplicate_field",
        ));
    }
    let Some(member) = members.iter().find(|member| member.key() == name) else {
        return Ok(None);
    };
    if member.value().kind() == JsonValueKind::Null {
        return Err(Phase8PolicyValidationError::new(
            field,
            "object",
            "null_not_allowed",
        ));
    }
    Ok(Some(member.value()))
}

fn required_string_field(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
    expected: &str,
) -> Result<String, Phase8PolicyValidationError> {
    let value = required_field_value(members, name, field, expected)?;
    value
        .string_value()
        .map(ToOwned::to_owned)
        .ok_or_else(|| wrong_type_error(field, expected, value.kind()))
}

fn optional_string_field(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
    expected: &str,
) -> Result<Option<String>, Phase8PolicyValidationError> {
    let Some(value) = unique_optional_field_value(members, name, field)? else {
        return Ok(None);
    };
    value
        .string_value()
        .map(|value| Some(value.to_owned()))
        .ok_or_else(|| wrong_type_error(field, expected, value.kind()))
}

fn required_fixed_string_field(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
    expected: &str,
    fixed: &str,
) -> Result<String, Phase8PolicyValidationError> {
    let value = required_string_field(members, name, field, expected)?;
    if value != fixed {
        return Err(Phase8PolicyValidationError::new(
            field,
            expected,
            if field.ends_with("schema") || field == "schema" {
                value
            } else if expected == "file" {
                "invalid_enum".to_owned()
            } else {
                "invalid_fixed_value".to_owned()
            },
        ));
    }
    Ok(value)
}

fn required_integer_raw_field(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
    expected: &str,
) -> Result<String, Phase8PolicyValidationError> {
    let value = required_field_value(members, name, field, expected)?;
    let Some(raw) = value.number_raw() else {
        return Err(wrong_type_error(field, expected, value.kind()));
    };
    if raw.contains('.') || raw.contains('e') || raw.contains('E') {
        return Err(wrong_type_error(field, expected, value.kind()));
    }
    Ok(raw.to_owned())
}

fn required_bool_field(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
    expected: &str,
) -> Result<bool, Phase8PolicyValidationError> {
    let value = required_field_value(members, name, field, expected)?;
    value
        .bool_value()
        .ok_or_else(|| wrong_type_error(field, expected, value.kind()))
}

fn required_hash_field(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
    expected: &str,
) -> Result<Hash, Phase8PolicyValidationError> {
    let raw = required_string_field(members, name, field, expected)?;
    parse_hash_string(&raw)
        .map_err(|_| Phase8PolicyValidationError::new(field, expected, "invalid_hash_format"))
}

fn required_string_array_field(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
    expected_entry: &str,
) -> Result<Vec<String>, Phase8PolicyValidationError> {
    let value = required_field_value(members, name, field, "array")?;
    let Some(elements) = value.array_elements() else {
        return Err(wrong_type_error(field, "array", value.kind()));
    };
    let mut out = Vec::new();
    for (index, element) in elements.iter().enumerate() {
        let path = format!("{field}[{index}]");
        let Some(value) = element.string_value() else {
            return Err(wrong_type_error(&path, expected_entry, element.kind()));
        };
        out.push(value.to_owned());
    }
    Ok(out)
}

fn reject_unknown_fields(
    members: &[JsonMember<'_>],
    allowed: &[&str],
    container_path: &str,
) -> Result<(), Phase8PolicyValidationError> {
    let mut counts = BTreeMap::<String, usize>::new();
    for member in members {
        *counts.entry(member.key().to_owned()).or_default() += 1;
    }
    let mut unknown = counts
        .keys()
        .filter(|field| !allowed.contains(&field.as_str()))
        .map(String::as_str)
        .collect::<Vec<_>>();
    unknown.sort_by(|left, right| phase8_rfc8785_object_key_cmp(left, right));
    if let Some(field) = unknown.first() {
        let report_path = unknown_field_report_path(container_path, field);
        let actual = if counts.get(*field).copied().unwrap_or(0) > 1 {
            "duplicate_field"
        } else {
            "unknown_field"
        };
        let expected = if actual == "duplicate_field" {
            "unique_object_keys"
        } else {
            "absent"
        };
        return Err(Phase8PolicyValidationError::new(
            report_path,
            expected,
            actual,
        ));
    }
    Ok(())
}

fn wrong_type_error(
    field: &str,
    expected: &str,
    actual: JsonValueKind,
) -> Phase8PolicyValidationError {
    Phase8PolicyValidationError::new(
        field,
        expected,
        if actual == JsonValueKind::Null {
            "null_not_allowed"
        } else {
            "wrong_type"
        },
    )
}

fn duplicate_member(members: &[JsonMember<'_>], name: &str) -> bool {
    members
        .iter()
        .filter(|member| member.key() == name)
        .take(2)
        .count()
        > 1
}

fn parse_positive_i64_domain(raw: &str, field: &str) -> Result<u64, Phase8PolicyValidationError> {
    let value = raw.parse::<u64>().map_err(|_| {
        Phase8PolicyValidationError::new(field, "positive_i64", "integer_out_of_range")
    })?;
    if value == 0 {
        return Err(Phase8PolicyValidationError::new(
            field,
            "positive_i64",
            "non_positive_integer",
        ));
    }
    if value > i64::MAX as u64 {
        return Err(Phase8PolicyValidationError::new(
            field,
            "positive_i64",
            "integer_out_of_range",
        ));
    }
    Ok(value)
}

fn phase8_valid_runner_policy_id(value: &str) -> bool {
    phase8_valid_lower_profile_like(value, 64, false)
}

fn phase8_valid_checker_profile_name(value: &str) -> bool {
    phase8_valid_lower_profile_like(value, 64, false)
}

fn phase8_valid_lower_profile_like(value: &str, max_len: usize, allow_dot: bool) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() || bytes.len() > max_len || !bytes[0].is_ascii_lowercase() {
        return false;
    }
    bytes.iter().all(|byte| {
        byte.is_ascii_lowercase()
            || byte.is_ascii_digit()
            || *byte == b'-'
            || (allow_dot && *byte == b'.')
            || (allow_dot && *byte == b'_')
    })
}

fn phase8_valid_checker_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() || bytes.len() > 128 || !bytes[0].is_ascii_lowercase() {
        return false;
    }
    bytes.iter().all(|byte| {
        byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(*byte, b'.' | b'_' | b'-')
    })
}

fn phase8_visible_ascii_nonempty(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| (0x21..=0x7e).contains(&byte))
}

fn phase8_valid_workspace_relative_path(value: &str) -> bool {
    if value.is_empty()
        || value.starts_with('/')
        || value.contains('\\')
        || value.contains(':')
        || value.contains("://")
        || value.bytes().any(|byte| byte <= 0x20 || byte == 0x7f)
    {
        return false;
    }
    value
        .split('/')
        .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

fn format_budget_key_path(profile: &str) -> String {
    if phase8_budget_key_path_component(profile) {
        format!("budgets.{profile}")
    } else {
        "budgets".to_owned()
    }
}

fn phase8_budget_key_path_component(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn unknown_field_report_path(container: &str, field: &str) -> String {
    if phase8_budget_key_path_component(field) {
        if container == "$" {
            field.to_owned()
        } else {
            format!("{container}.{field}")
        }
    } else {
        container.to_owned()
    }
}

fn phase8_hash_json_literal(hash: &Hash) -> String {
    phase8_json_string_literal(&format_hash_string(hash))
}

fn canonical_json_array(values: Vec<String>) -> String {
    let mut out = String::new();
    out.push('[');
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(value);
    }
    out.push(']');
    out
}

pub fn phase8_canonical_json_string(source: &str) -> Result<String, Phase8CanonicalJsonError> {
    let bytes = phase8_canonical_json_bytes(source)?;
    Ok(String::from_utf8(bytes).expect("canonical JSON is UTF-8"))
}

pub fn phase8_canonical_json_bytes(source: &str) -> Result<Vec<u8>, Phase8CanonicalJsonError> {
    let document =
        JsonDocument::parse(source).map_err(|err| Phase8CanonicalJsonError::JsonParse {
            offset: err.offset,
            kind: err.kind,
        })?;
    phase8_canonical_json_value_bytes(document.root())
}

pub fn phase8_canonical_json_value_bytes(
    value: &JsonValue<'_>,
) -> Result<Vec<u8>, Phase8CanonicalJsonError> {
    let mut out = Vec::new();
    let root = Phase8JsonPath::root();
    write_phase8_canonical_json_value(value, &root, &mut out)?;
    Ok(out)
}

pub fn phase8_canonical_json_object_excluding_top_level_fields(
    source: &str,
    excluded_fields: &[&str],
) -> Result<Vec<u8>, Phase8CanonicalJsonError> {
    let document =
        JsonDocument::parse(source).map_err(|err| Phase8CanonicalJsonError::JsonParse {
            offset: err.offset,
            kind: err.kind,
        })?;
    let root = Phase8JsonPath::root();
    let Some(members) = document.root().object_members() else {
        return Err(Phase8CanonicalJsonError::ExpectedObject {
            path: root,
            actual: document.root().kind(),
        });
    };
    let excluded = excluded_fields.iter().copied().collect::<BTreeSet<_>>();
    let mut out = Vec::new();
    write_phase8_canonical_json_object_members(members, &root, &mut out, Some(&excluded))?;
    Ok(out)
}

pub fn phase8_hash_json_object_excluding_top_level_fields(
    source: &str,
    excluded_fields: &[&str],
) -> Result<Hash, Phase8CanonicalJsonError> {
    let canonical =
        phase8_canonical_json_object_excluding_top_level_fields(source, excluded_fields)?;
    Ok(phase8_sha256(&canonical))
}

pub fn phase8_json_artifact_hash_from_source(
    source: &str,
) -> Result<Hash, Phase8CanonicalJsonError> {
    let canonical = phase8_canonical_json_bytes(source)?;
    Ok(phase8_sha256(&canonical))
}

pub fn phase8_file_hash(bytes: &[u8]) -> Hash {
    phase8_sha256(bytes)
}

pub fn phase8_sha256(bytes: &[u8]) -> Hash {
    let digest = Sha256::digest(bytes);
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&digest);
    hash
}

pub fn phase8_rfc8785_object_key_cmp(left: &str, right: &str) -> Ordering {
    let mut left_units = left.encode_utf16();
    let mut right_units = right.encode_utf16();
    loop {
        match (left_units.next(), right_units.next()) {
            (Some(left), Some(right)) => match left.cmp(&right) {
                Ordering::Equal => {}
                ordering => return ordering,
            },
            (Some(_), None) => return Ordering::Greater,
            (None, Some(_)) => return Ordering::Less,
            (None, None) => return Ordering::Equal,
        }
    }
}

pub fn phase8_forbids_verification_input_field(field: &str) -> bool {
    PHASE8_FORBIDDEN_VERIFICATION_INPUT_FIELDS.contains(&field)
}

pub fn phase8_first_forbidden_verification_input_field<'value, 'src>(
    value: &'value JsonValue<'src>,
) -> Option<&'value str> {
    let members = value.object_members()?;
    let mut fields = members
        .iter()
        .map(crate::json::JsonMember::key)
        .filter(|field| phase8_forbids_verification_input_field(field))
        .collect::<Vec<_>>();
    fields.sort_by(|left, right| phase8_rfc8785_object_key_cmp(left, right));
    fields.first().copied()
}

pub fn validate_phase8_closed_object<'value, 'src>(
    value: &'value JsonValue<'src>,
    schema: Phase8ClosedSchema<'_>,
    path: &Phase8JsonPath,
) -> Result<Phase8ValidatedObject<'value, 'src>, Phase8SchemaError> {
    let Some(members) = value.object_members() else {
        return Err(Phase8SchemaError {
            path: path.clone(),
            reason: Phase8SchemaErrorReason::ExpectedObject {
                actual: value.kind(),
            },
        });
    };

    let mut counts = BTreeMap::<String, usize>::new();
    for member in members {
        *counts.entry(member.key().to_owned()).or_insert(0) += 1;
    }

    for field in schema.fields {
        if counts.get(field.name).copied().unwrap_or(0) > 1 {
            return Err(Phase8SchemaError {
                path: path.field(field.name),
                reason: Phase8SchemaErrorReason::DuplicateField {
                    field: field.name.to_owned(),
                },
            });
        }
    }

    let mut duplicate_unknowns = counts
        .iter()
        .filter(|(field, count)| {
            **count > 1 && !schema.fields.iter().any(|spec| spec.name == field.as_str())
        })
        .map(|(field, _)| field.as_str())
        .collect::<Vec<_>>();
    duplicate_unknowns.sort_by(|left, right| phase8_rfc8785_object_key_cmp(left, right));
    if let Some(field) = duplicate_unknowns.first() {
        return Err(Phase8SchemaError {
            path: path.clone(),
            reason: Phase8SchemaErrorReason::DuplicateField {
                field: (*field).to_owned(),
            },
        });
    }

    let mut unknowns = counts
        .keys()
        .filter(|field| !schema.fields.iter().any(|spec| spec.name == field.as_str()))
        .map(String::as_str)
        .collect::<Vec<_>>();
    unknowns.sort_by(|left, right| phase8_rfc8785_object_key_cmp(left, right));
    if let Some(field) = unknowns.first() {
        return Err(Phase8SchemaError {
            path: path.clone(),
            reason: Phase8SchemaErrorReason::UnknownField {
                field: (*field).to_owned(),
            },
        });
    }

    for field in schema.fields {
        if field.required && !members.iter().any(|member| member.key() == field.name) {
            return Err(Phase8SchemaError {
                path: path.field(field.name),
                reason: Phase8SchemaErrorReason::MissingField { field: field.name },
            });
        }
    }

    for field in schema.fields {
        let Some(member) = members.iter().find(|member| member.key() == field.name) else {
            continue;
        };
        validate_phase8_field(field, member.value(), &path.field(field.name))?;
    }

    Ok(Phase8ValidatedObject { members })
}

fn write_phase8_canonical_json_value(
    value: &JsonValue<'_>,
    path: &Phase8JsonPath,
    out: &mut Vec<u8>,
) -> Result<(), Phase8CanonicalJsonError> {
    match value.kind() {
        JsonValueKind::Null => out.extend_from_slice(b"null"),
        JsonValueKind::Bool => {
            if value.bool_value() == Some(true) {
                out.extend_from_slice(b"true");
            } else {
                out.extend_from_slice(b"false");
            }
        }
        JsonValueKind::Number => {
            let raw = value.number_raw().expect("number kind has raw number");
            validate_phase8_canonical_integer(raw, path)?;
            out.extend_from_slice(raw.as_bytes());
        }
        JsonValueKind::String => {
            let value = value.string_value().expect("string kind has string value");
            out.extend_from_slice(phase8_json_string_literal(value).as_bytes());
        }
        JsonValueKind::Array => {
            out.push(b'[');
            let elements = value.array_elements().expect("array kind has array values");
            for (index, element) in elements.iter().enumerate() {
                if index > 0 {
                    out.push(b',');
                }
                write_phase8_canonical_json_value(element, &path.index(index), out)?;
            }
            out.push(b']');
        }
        JsonValueKind::Object => {
            let members = value.object_members().expect("object kind has members");
            write_phase8_canonical_json_object_members(members, path, out, None)?;
        }
    }
    Ok(())
}

fn write_phase8_canonical_json_object_members(
    members: &[crate::json::JsonMember<'_>],
    path: &Phase8JsonPath,
    out: &mut Vec<u8>,
    excluded_top_level: Option<&BTreeSet<&str>>,
) -> Result<(), Phase8CanonicalJsonError> {
    let mut seen = BTreeSet::new();
    for member in members {
        if !seen.insert(member.key()) {
            return Err(Phase8CanonicalJsonError::DuplicateObjectKey {
                path: path.field(member.key()),
                key: member.key().to_owned(),
            });
        }
    }

    let mut entries = Vec::new();
    for member in members {
        if excluded_top_level
            .map(|excluded| excluded.contains(member.key()))
            .unwrap_or(false)
        {
            continue;
        }
        entries.push((member.key(), member.value()));
    }
    entries.sort_by(|(left, _), (right, _)| phase8_rfc8785_object_key_cmp(left, right));

    out.push(b'{');
    for (index, (key, value)) in entries.iter().enumerate() {
        if index > 0 {
            out.push(b',');
        }
        out.extend_from_slice(phase8_json_string_literal(key).as_bytes());
        out.push(b':');
        write_phase8_canonical_json_value(value, &path.field(*key), out)?;
    }
    out.push(b'}');
    Ok(())
}

fn validate_phase8_canonical_integer(
    raw: &str,
    path: &Phase8JsonPath,
) -> Result<(), Phase8CanonicalJsonError> {
    if raw.contains('.') || raw.contains('e') || raw.contains('E') {
        return Err(Phase8CanonicalJsonError::FloatNumber {
            path: path.clone(),
            raw: raw.to_owned(),
        });
    }
    if raw == "-0" {
        return Err(Phase8CanonicalJsonError::NegativeZeroInteger { path: path.clone() });
    }
    let digits = raw.strip_prefix('-').unwrap_or(raw);
    if digits.is_empty()
        || (digits.len() > 1 && digits.starts_with('0'))
        || !digits.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(Phase8CanonicalJsonError::InvalidInteger {
            path: path.clone(),
            raw: raw.to_owned(),
        });
    }
    Ok(())
}

fn validate_phase8_field(
    field: &Phase8FieldSpec,
    value: &JsonValue<'_>,
    path: &Phase8JsonPath,
) -> Result<(), Phase8SchemaError> {
    if value.kind() == JsonValueKind::Null {
        if field.allow_null {
            return Ok(());
        }
        return Err(Phase8SchemaError {
            path: path.clone(),
            reason: Phase8SchemaErrorReason::NullNotAllowed { field: field.name },
        });
    }

    match field.field_type {
        Phase8JsonFieldType::Object if value.kind() == JsonValueKind::Object => Ok(()),
        Phase8JsonFieldType::Array if value.kind() == JsonValueKind::Array => Ok(()),
        Phase8JsonFieldType::String if value.kind() == JsonValueKind::String => Ok(()),
        Phase8JsonFieldType::Boolean if value.kind() == JsonValueKind::Bool => Ok(()),
        Phase8JsonFieldType::Integer => {
            let Some(raw) = value.number_raw() else {
                return Err(phase8_wrong_type(field, value, path));
            };
            validate_phase8_schema_integer(raw).map_err(|()| Phase8SchemaError {
                path: path.clone(),
                reason: Phase8SchemaErrorReason::InvalidInteger {
                    field: field.name,
                    raw: raw.to_owned(),
                },
            })
        }
        Phase8JsonFieldType::HashString => {
            let Some(raw) = value.string_value() else {
                return Err(phase8_wrong_type(field, value, path));
            };
            parse_hash_string(raw)
                .map(|_| ())
                .map_err(|_| Phase8SchemaError {
                    path: path.clone(),
                    reason: Phase8SchemaErrorReason::InvalidHashFormat { field: field.name },
                })
        }
        _ => Err(phase8_wrong_type(field, value, path)),
    }
}

fn validate_phase8_schema_integer(raw: &str) -> Result<(), ()> {
    if raw.contains('.') || raw.contains('e') || raw.contains('E') || raw == "-0" {
        return Err(());
    }
    let digits = raw.strip_prefix('-').unwrap_or(raw);
    if digits.is_empty()
        || (digits.len() > 1 && digits.starts_with('0'))
        || !digits.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(());
    }
    Ok(())
}

fn phase8_wrong_type(
    field: &Phase8FieldSpec,
    value: &JsonValue<'_>,
    path: &Phase8JsonPath,
) -> Phase8SchemaError {
    Phase8SchemaError {
        path: path.clone(),
        reason: Phase8SchemaErrorReason::WrongType {
            field: field.name,
            expected: field.field_type,
            actual: value.kind(),
        },
    }
}

fn phase8_json_string_literal(value: &str) -> String {
    let mut out = String::new();
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0008}' => out.push_str("\\b"),
            '\t' => out.push_str("\\t"),
            '\n' => out.push_str("\\n"),
            '\u{000c}' => out.push_str("\\f"),
            '\r' => out.push_str("\\r"),
            '\u{0000}'..='\u{001f}' => {
                out.push_str("\\u00");
                out.push(hex_digit((ch as u8) >> 4));
                out.push(hex_digit((ch as u8) & 0x0f));
            }
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn phase8_json_string_array(values: &[String]) -> String {
    let mut out = String::new();
    out.push('[');
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(&phase8_json_string_literal(value));
    }
    out.push(']');
    out
}

fn canonical_json_object_from_pairs(mut pairs: Vec<(String, String)>) -> String {
    pairs.sort_by(|(left, _), (right, _)| phase8_rfc8785_object_key_cmp(left, right));
    let mut out = String::new();
    out.push('{');
    for (index, (key, value)) in pairs.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(&phase8_json_string_literal(key));
        out.push(':');
        out.push_str(value);
    }
    out.push('}');
    out
}

fn push_optional_string_pair(pairs: &mut Vec<(String, String)>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        pairs.push((key.to_owned(), phase8_json_string_literal(value)));
    }
}

fn push_optional_hash_pair(pairs: &mut Vec<(String, String)>, key: &str, value: Option<Hash>) {
    if let Some(value) = value {
        pairs.push((
            key.to_owned(),
            phase8_json_string_literal(&format_hash_string(&value)),
        ));
    }
}

fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'a' + (value - 10)),
        _ => unreachable!("hex digit out of range"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_SCHEMA_FIELDS: &[Phase8FieldSpec] = &[
        Phase8FieldSpec::required("schema", Phase8JsonFieldType::String),
        Phase8FieldSpec::required("artifact_hash", Phase8JsonFieldType::HashString),
        Phase8FieldSpec::required("count", Phase8JsonFieldType::Integer),
        Phase8FieldSpec::optional("note", Phase8JsonFieldType::String),
    ];
    const SIMPLE_SCHEMA: Phase8ClosedSchema<'_> = Phase8ClosedSchema::new(SIMPLE_SCHEMA_FIELDS);

    fn test_hash(byte: u8) -> Hash {
        [byte; 32]
    }

    fn hash_wire(byte: u8) -> String {
        format_hash_string(&test_hash(byte))
    }

    fn valid_runner_policy_json() -> String {
        format!(
            r#"{{
              "schema":"npa.phase8.runner_policy.v1",
              "id":"phase8-pr",
              "version":1,
              "trust_mode":"pr",
              "required_checker_profiles":["reference"],
              "optional_checker_profiles":[],
              "checker_allowlist":[{{
                "profile":"reference",
                "checker_id":"npa-checker-ref",
                "binary_id":"npa-checker-ref-macos-aarch64",
                "binary_hash":"{}",
                "build_hash":"{}",
                "allowed_args":["--json","--canonical-only"]
              }}],
              "checker_identity_manifest":{{
                "kind":"file",
                "path":"ci/checker-identity-manifest.json",
                "manifest_hash":"{}"
              }},
              "import_policy":{{
                "mode":"locked_store",
                "network":"forbidden",
                "require_import_lock_hash":true
              }},
              "axiom_policy":{{
                "path":"ci/axiom-policy.toml",
                "hash":"{}"
              }},
              "budgets":{{
                "reference":{{"max_steps":10000000,"max_memory_mb":2048,"timeout_ms":60000}}
              }},
              "on_resource_exhausted":"fail",
              "on_missing_required_checker":"fail",
              "on_profile_requested_by_ai":"ignore_unless_policy_allows"
            }}"#,
            hash_wire(10),
            hash_wire(11),
            hash_wire(12),
            hash_wire(13)
        )
    }

    fn valid_identity_manifest_json() -> String {
        format!(
            r#"{{
              "schema":"npa.phase8.checker_identity_manifest.v1",
              "generated_by":{{
                "runner_id":"npa-check-runner",
                "runner_version":"0.8.0",
                "runner_build_hash":"{}"
              }},
              "checkers":[{{
                "profile":"reference",
                "checker_id":"npa-checker-ref",
                "checker_version":"0.8.0",
                "binary_id":"npa-checker-ref-macos-aarch64",
                "binary_hash":"{}",
                "build_hash":"{}"
              }}]
            }}"#,
            hash_wire(20),
            hash_wire(10),
            hash_wire(11)
        )
    }

    #[test]
    fn canonical_json_uses_rfc8785_object_order_and_string_rules() {
        let canonical = phase8_canonical_json_string(
            r#"{"b":2,"a":"\/é","\u000a":"line\n","control":"\u0001"}"#,
        )
        .unwrap();

        assert_eq!(
            canonical,
            "{\"\\n\":\"line\\n\",\"a\":\"/é\",\"b\":2,\"control\":\"\\u0001\"}"
        );
    }

    #[test]
    fn canonical_json_rejects_duplicate_keys_before_value_contents() {
        let err = phase8_canonical_json_string(r#"{"a":{"bad":1.5},"\u0061":0}"#).unwrap_err();

        assert_eq!(
            err,
            Phase8CanonicalJsonError::DuplicateObjectKey {
                path: Phase8JsonPath::root().field("a"),
                key: "a".to_owned(),
            }
        );
    }

    #[test]
    fn canonical_json_rejects_floats_and_negative_zero() {
        let float_err = phase8_canonical_json_string(r#"{"n":1e0}"#).unwrap_err();
        assert!(matches!(
            float_err,
            Phase8CanonicalJsonError::FloatNumber { .. }
        ));

        let negative_zero_err = phase8_canonical_json_string(r#"{"n":-0}"#).unwrap_err();
        assert!(matches!(
            negative_zero_err,
            Phase8CanonicalJsonError::NegativeZeroInteger { .. }
        ));
    }

    #[test]
    fn hash_usage_table_pins_hash_domains_and_self_exclusions() {
        let request_hash = phase8_hash_usage_rule(Phase8HashUsageKind::RequestHash);
        assert_eq!(request_hash.excluded_fields, ["request_id", "request_hash"]);
        assert!(request_hash.excludes_self_field);

        let result_hash = phase8_hash_usage_rule(Phase8HashUsageKind::MachineCheckResultHash);
        assert_eq!(
            result_hash.excluded_fields,
            MACHINE_CHECK_RESULT_HASH_EXCLUDED_FIELDS
        );
        assert_eq!(
            result_hash.input_kind,
            Phase8HashInputKind::CanonicalJsonProjection
        );

        let run_artifact_hash = phase8_hash_usage_rule(Phase8HashUsageKind::RunArtifactHash);
        assert_eq!(run_artifact_hash.excluded_fields, ["run_artifact_hash"]);

        let file_hash = phase8_hash_usage_rule(Phase8HashUsageKind::FileHash);
        assert_eq!(file_hash.input_kind, Phase8HashInputKind::ExactFileBytes);
        assert!(!file_hash.excludes_self_field);
    }

    #[test]
    fn top_level_hash_exclusion_keeps_duplicate_rejection() {
        let hash_a = phase8_hash_json_object_excluding_top_level_fields(
            r#"{"result_id":"r1","result_hash":"sha256:0000000000000000000000000000000000000000000000000000000000000000","status":"failed"}"#,
            &["result_id", "result_hash"],
        )
        .unwrap();
        let hash_b = phase8_hash_json_object_excluding_top_level_fields(
            r#"{"result_id":"r2","result_hash":"sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff","status":"failed"}"#,
            &["result_id", "result_hash"],
        )
        .unwrap();
        assert_eq!(hash_a, hash_b);

        let duplicate = phase8_hash_json_object_excluding_top_level_fields(
            r#"{"result_hash":"sha256:0000000000000000000000000000000000000000000000000000000000000000","result_hash":"sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff","status":"failed"}"#,
            &["result_hash"],
        )
        .unwrap_err();
        assert!(matches!(
            duplicate,
            Phase8CanonicalJsonError::DuplicateObjectKey { .. }
        ));
    }

    #[test]
    fn artifact_classification_separates_verdicts_errors_and_transient_envelopes() {
        assert_eq!(
            Phase8ArtifactKind::MachineCheckRequestErrorResult.classification(),
            Phase8ArtifactClassification::SavedArtifact
        );
        assert!(Phase8ArtifactKind::MachineCheckRequestErrorResult.is_pipeline_error_artifact());
        assert!(!Phase8ArtifactKind::MachineCheckRequestErrorResult.is_checker_verdict());
        assert!(!Phase8ArtifactKind::MachineCheckRequestErrorResult.is_normalization_input());

        assert_eq!(
            Phase8ArtifactKind::CommandError.classification(),
            Phase8ArtifactClassification::TransientResponse
        );
        assert_eq!(
            Phase8ArtifactKind::ApiError.classification(),
            Phase8ArtifactClassification::TransientResponse
        );
        assert!(!Phase8ArtifactKind::CommandError.has_result_hash_field());
        assert!(!Phase8ArtifactKind::ApiError.has_result_hash_field());

        assert!(phase8_forbids_verification_input_field("source"));
        assert!(phase8_forbids_verification_input_field("tactic_trace"));
        assert!(phase8_forbids_verification_input_field("ai_trace"));

        let forbidden_doc = JsonDocument::parse(r#"{"source":"x","ai_trace":[]}"#).unwrap();
        assert_eq!(
            phase8_first_forbidden_verification_input_field(forbidden_doc.root()),
            Some("ai_trace")
        );
    }

    #[test]
    fn closed_world_schema_failure_order_is_deterministic() {
        let duplicate_doc =
            JsonDocument::parse(r#"{"schema":"x","artifact_hash":"bad","schema":null}"#).unwrap();
        let duplicate = validate_phase8_closed_object(
            duplicate_doc.root(),
            SIMPLE_SCHEMA,
            &Phase8JsonPath::root(),
        )
        .unwrap_err();
        assert_eq!(
            duplicate.reason,
            Phase8SchemaErrorReason::DuplicateField {
                field: "schema".to_owned()
            }
        );

        let unknown_doc = JsonDocument::parse(
            r#"{"schema":"x","artifact_hash":"bad","count":1,"zzz":true,"aaa":true}"#,
        )
        .unwrap();
        let unknown = validate_phase8_closed_object(
            unknown_doc.root(),
            SIMPLE_SCHEMA,
            &Phase8JsonPath::root(),
        )
        .unwrap_err();
        assert_eq!(
            unknown.reason,
            Phase8SchemaErrorReason::UnknownField {
                field: "aaa".to_owned()
            }
        );

        let missing_doc = JsonDocument::parse(r#"{"schema":"x","artifact_hash":"bad"}"#).unwrap();
        let missing = validate_phase8_closed_object(
            missing_doc.root(),
            SIMPLE_SCHEMA,
            &Phase8JsonPath::root(),
        )
        .unwrap_err();
        assert_eq!(
            missing.reason,
            Phase8SchemaErrorReason::MissingField { field: "count" }
        );

        let null_doc =
            JsonDocument::parse(r#"{"schema":null,"artifact_hash":"bad","count":1}"#).unwrap();
        let null =
            validate_phase8_closed_object(null_doc.root(), SIMPLE_SCHEMA, &Phase8JsonPath::root())
                .unwrap_err();
        assert_eq!(
            null.reason,
            Phase8SchemaErrorReason::NullNotAllowed { field: "schema" }
        );

        let wrong_type_doc =
            JsonDocument::parse(r#"{"schema":"x","artifact_hash":"bad","count":"1"}"#).unwrap();
        let wrong_type = validate_phase8_closed_object(
            wrong_type_doc.root(),
            SIMPLE_SCHEMA,
            &Phase8JsonPath::root(),
        )
        .unwrap_err();
        assert_eq!(
            wrong_type.reason,
            Phase8SchemaErrorReason::InvalidHashFormat {
                field: "artifact_hash"
            }
        );

        let wrong_type_doc = JsonDocument::parse(
            r#"{"schema":"x","artifact_hash":"sha256:0000000000000000000000000000000000000000000000000000000000000000","count":"1"}"#,
        )
        .unwrap();
        let wrong_type = validate_phase8_closed_object(
            wrong_type_doc.root(),
            SIMPLE_SCHEMA,
            &Phase8JsonPath::root(),
        )
        .unwrap_err();
        assert_eq!(
            wrong_type.reason,
            Phase8SchemaErrorReason::WrongType {
                field: "count",
                expected: Phase8JsonFieldType::Integer,
                actual: JsonValueKind::String
            }
        );
    }

    #[test]
    fn error_results_are_hashed_artifacts_not_checker_verdicts() {
        let request_error = Phase8MachineCheckRequestErrorResult::request_load_failure(
            "mchkreqerr_001",
            Phase8MachineCheckRequestErrorReasonCode::RequestHashMissing,
            "request_hash",
        )
        .with_request_path("build/check-requests/Std.Nat.reference.json")
        .with_request_file_hash(test_hash(1));
        assert_eq!(
            request_error.error().kind,
            Phase8PipelineErrorKind::RequestLoadFailure
        );
        let request_json = request_error.canonical_json();
        assert!(request_json.contains(PHASE8_MACHINE_CHECK_REQUEST_ERROR_RESULT_SCHEMA));
        assert!(request_json.contains("\"status\":\"failed\""));
        assert!(request_json.contains("\"result_hash\":\"sha256:"));
        assert!(!request_error
            .hash_input_canonical_json()
            .contains("result_id"));
        assert!(!request_error
            .hash_input_canonical_json()
            .contains("result_hash"));

        let normalize_error = Phase8NormalizeErrorResult::normalize_failure(
            "normerr_Std.Nat_001",
            Phase8NormalizeErrorReasonCode::RequestHashNotFound,
            "request_hash",
        )
        .with_policy_hash(test_hash(2));
        assert_eq!(
            normalize_error.error().kind,
            Phase8PipelineErrorKind::NormalizeFailure
        );
        let normalize_json = normalize_error.canonical_json();
        assert!(normalize_json.contains(PHASE8_NORMALIZE_ERROR_RESULT_SCHEMA));
        assert!(normalize_json.contains("\"kind\":\"normalize_failure\""));
        assert!(normalize_json.contains("\"policy_hash\":\"sha256:"));
    }

    #[test]
    fn command_and_api_errors_are_transient_and_do_not_carry_result_hash() {
        let mut command = Phase8CommandError::new(
            Phase8CommandName::ChallengeMaterializeRequests,
            "output_write_failure",
        );
        command.field = Some("request_output_dir/reference.json".to_owned());
        command.expected_hash = Some(test_hash(3));
        command.actual_hash = Some(test_hash(4));
        let command_json = command.canonical_json();
        assert!(command_json.contains(PHASE8_COMMAND_ERROR_SCHEMA));
        assert!(!command_json.contains("result_hash"));

        let api_error = Phase8ApiError::new(
            "/machine/check/challenge",
            Phase8ApiErrorReasonCode::ApiPathOutsideWorkspace,
            "policy.path",
            "workspace_relative_path",
            "api_path_outside_workspace",
        );
        let api_json = api_error.canonical_json();
        assert!(api_json.contains(PHASE8_API_ERROR_SCHEMA));
        assert!(!api_json.contains("result_hash"));
        assert!(!api_json.contains("expected_hash"));
        assert!(!api_json.contains("actual_hash"));
    }

    #[test]
    fn m1_runner_policy_validates_and_hashes_canonical_json() {
        let policy = parse_phase8_runner_policy(&valid_runner_policy_json()).unwrap();
        assert_eq!(policy.id, "phase8-pr");
        assert_eq!(policy.trust_mode, Phase8TrustMode::Pr);
        assert_eq!(policy.required_checker_profiles, ["reference"]);
        assert_eq!(
            policy
                .selected_checker_policy("reference")
                .unwrap()
                .allowed_args,
            ["--json", "--canonical-only"]
        );

        let canonical_hash = policy.policy_hash();
        assert_eq!(
            phase8_runner_policy_hash(&policy.canonical_json()).unwrap(),
            canonical_hash
        );
    }

    #[test]
    fn m1_runner_policy_reference_has_fixed_member_priority() {
        let err = parse_phase8_runner_policy_reference(
            r#"{"kind":"file","path":"/abs/policy.json","hash":"bad"}"#,
        )
        .unwrap_err();
        assert_eq!(
            err,
            Phase8PolicyValidationError::new(
                "policy.path",
                "workspace_relative_path",
                "invalid_path"
            )
        );

        let ok = parse_phase8_runner_policy_reference(&format!(
            r#"{{"kind":"file","path":"ci/policy.json","hash":"{}"}}"#,
            hash_wire(30)
        ))
        .unwrap();
        assert_eq!(ok.path, "ci/policy.json");
        assert_eq!(ok.hash, test_hash(30));
    }

    #[test]
    fn m1_policy_rejects_required_profile_and_allowed_arg_contract_violations() {
        let err = parse_phase8_runner_policy(&valid_runner_policy_json().replace(
            r#""required_checker_profiles":["reference"]"#,
            r#""required_checker_profiles":["external"]"#,
        ))
        .unwrap_err();
        assert_eq!(
            err,
            Phase8PolicyValidationError::new(
                "required_checker_profiles",
                "profiles_for_trust_mode:pr",
                "profile_set_mismatch"
            )
        );

        let err = parse_phase8_runner_policy(&valid_runner_policy_json().replace(
            r#""allowed_args":["--json","--canonical-only"]"#,
            r#""allowed_args":["--json","--imports=lock.json"]"#,
        ))
        .unwrap_err();
        assert_eq!(
            err,
            Phase8PolicyValidationError::new(
                "checker_allowlist[0].allowed_args[1]",
                "static_checker_option_without_runner_owned_dynamic_args",
                "reserved_dynamic_arg"
            )
        );

        let err = parse_phase8_runner_policy(&valid_runner_policy_json().replace(
            r#""allowed_args":["--json","--canonical-only"]"#,
            r#""allowed_args":["--json","--format=json"]"#,
        ))
        .unwrap_err();
        assert_eq!(
            err,
            Phase8PolicyValidationError::new(
                "checker_allowlist[0].allowed_args[1]",
                "json_output_contract",
                "json_output_arg_conflict"
            )
        );
    }

    #[test]
    fn m1_registry_identity_manifest_and_command_boundary_are_runner_owned() {
        let policy = parse_phase8_runner_policy(&valid_runner_policy_json()).unwrap();
        let selected = policy.selected_checker_policy("reference").unwrap();

        let registry = parse_phase8_checker_binary_registry(
            r#"{
              "schema":"npa.phase8.checker_binary_registry.v1",
              "root_kind":"workspace",
              "entries":[{"binary_id":"npa-checker-ref-macos-aarch64","path":"tools/checkers/npa-checker-ref"}]
            }"#,
        )
        .unwrap();
        let resolved =
            phase8_resolve_checker_executable(&registry, selected, test_hash(10)).unwrap();
        assert_eq!(resolved.path, "tools/checkers/npa-checker-ref");

        let missing_registry = parse_phase8_checker_binary_registry(
            r#"{"schema":"npa.phase8.checker_binary_registry.v1","root_kind":"workspace","entries":[]}"#,
        )
        .unwrap();
        let missing = phase8_resolve_checker_executable(&missing_registry, selected, test_hash(10))
            .unwrap_err();
        assert_eq!(
            missing.reason_code,
            Phase8PolicyFailureReasonCode::CheckerBinaryFileUnreadable
        );
        assert_eq!(missing.actual_value.as_deref(), Some("binary_id_not_found"));

        let bad_hash =
            phase8_resolve_checker_executable(&registry, selected, test_hash(99)).unwrap_err();
        assert_eq!(
            bad_hash.reason_code,
            Phase8PolicyFailureReasonCode::CheckerBinaryHashMismatch
        );

        let manifest =
            parse_phase8_checker_identity_manifest(&valid_identity_manifest_json()).unwrap();
        phase8_validate_selected_checker_identity_manifest(selected, &manifest).unwrap();
        let mismatch_manifest = parse_phase8_checker_identity_manifest(
            &valid_identity_manifest_json().replace("npa-checker-ref", "other-checker"),
        )
        .unwrap();
        let mismatch =
            phase8_validate_selected_checker_identity_manifest(selected, &mismatch_manifest)
                .unwrap_err();
        assert_eq!(
            mismatch.reason_code,
            Phase8PolicyFailureReasonCode::CheckerIdentityMismatch
        );
        assert_eq!(
            mismatch.field.as_ref(),
            "checker_identity_manifest.checkers[].checker_id"
        );

        let dynamic = Phase8RunnerDynamicArgs {
            imports_manifest: "build/certs/import-lock.json".to_owned(),
            imports_manifest_hash: test_hash(40),
            trust_mode: "pr".to_owned(),
            axiom_policy_path: "ci/axiom-policy.toml".to_owned(),
            axiom_policy_hash: test_hash(13),
            max_steps: 10000000,
            max_memory_mb: 2048,
            timeout_ms: 60000,
            certificate_path: "build/certs/Std/Nat.npcert".to_owned(),
        };
        let argv = phase8_checker_argv(&resolved.path, selected, &dynamic);
        let expected_argv = vec![
            "tools/checkers/npa-checker-ref".to_owned(),
            "--json".to_owned(),
            "--canonical-only".to_owned(),
            "--imports".to_owned(),
            "build/certs/import-lock.json".to_owned(),
            "--imports-hash".to_owned(),
            hash_wire(40),
            "--trust-mode".to_owned(),
            "pr".to_owned(),
            "--axiom-policy".to_owned(),
            "ci/axiom-policy.toml".to_owned(),
            "--axiom-policy-hash".to_owned(),
            hash_wire(13),
            "--max-steps".to_owned(),
            "10000000".to_owned(),
            "--max-memory-mb".to_owned(),
            "2048".to_owned(),
            "--timeout-ms".to_owned(),
            "60000".to_owned(),
            "build/certs/Std/Nat.npcert".to_owned(),
        ];
        assert_eq!(argv, expected_argv);
        assert_eq!(
            phase8_runner_fixed_environment(),
            vec![
                ("LC_ALL".to_owned(), "C.UTF-8".to_owned()),
                ("LANG".to_owned(), "C.UTF-8".to_owned()),
                ("TZ".to_owned(), "UTC".to_owned()),
            ]
        );
    }
}
