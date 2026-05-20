use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use npa_cert::Hash;
use sha2::{Digest, Sha256};

use crate::json::{JsonDocument, JsonParseErrorKind, JsonValue, JsonValueKind};
use crate::types::{format_hash_string, parse_hash_string};

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
}
