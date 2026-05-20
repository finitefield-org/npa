use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use npa_cert::Hash;
use sha2::{Digest, Sha256};

use crate::json::{JsonDocument, JsonMember, JsonParseErrorKind, JsonValue, JsonValueKind};
use crate::types::{
    format_hash_string, parse_hash_string, parse_module_name_wire, phase5_name_canonical_bytes,
};

pub const PHASE8_RUNNER_POLICY_SCHEMA: &str = "npa.phase8.runner_policy.v1";
pub const PHASE8_CHECKER_IDENTITY_MANIFEST_SCHEMA: &str = "npa.phase8.checker_identity_manifest.v1";
pub const PHASE8_CHECKER_BINARY_REGISTRY_SCHEMA: &str = "npa.phase8.checker_binary_registry.v1";
pub const PHASE8_IMPORT_LOCK_MANIFEST_SCHEMA: &str = "npa.phase8.import_lock_manifest.v1";
pub const PHASE8_MACHINE_CHECK_REQUEST_SCHEMA: &str = "npa.phase8.machine_check_request.v1";
pub const PHASE8_REQUEST_STORE_MANIFEST_SCHEMA: &str = "npa.phase8.request_store_manifest.v1";
pub const PHASE8_CHECKER_RAW_RESULT_SCHEMA: &str = "npa.phase8.checker_raw_result.v1";
pub const PHASE8_MACHINE_CHECK_RESULT_SCHEMA: &str = "npa.phase8.machine_check_result.v1";
pub const PHASE8_MACHINE_RESULT_STORE_MANIFEST_SCHEMA: &str =
    "npa.phase8.machine_result_store_manifest.v1";
pub const PHASE8_AXIOM_REPORT_SCHEMA: &str = "npa.phase8.axiom_report.v1";
pub const PHASE8_AXIOM_REPORT_STORE_MANIFEST_SCHEMA: &str =
    "npa.phase8.axiom_report_store_manifest.v1";
pub const PHASE8_NORMALIZED_CHECK_RESULT_SCHEMA: &str = "npa.phase8.normalized_check_result.v1";
pub const PHASE8_NORMALIZED_RESULT_STORE_MANIFEST_SCHEMA: &str =
    "npa.phase8.normalized_result_store_manifest.v1";
pub const PHASE8_NORMALIZATION_WRITE_RESULT_SCHEMA: &str =
    "npa.phase8.normalization_write_result.v1";
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
    error: Box<Phase8StructuredError>,
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
            error: Box::new(Phase8StructuredError::normalize_failure(reason_code, field)),
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

    pub fn error(&self) -> &Phase8StructuredError {
        self.error.as_ref()
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
    Run,
    NormalizeResults,
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
            Self::Run => "run",
            Self::NormalizeResults => "normalize-results",
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
    pub reason_code: Box<str>,
    pub field: Option<Box<str>>,
    pub expected_hash: Option<Box<Hash>>,
    pub actual_hash: Option<Box<Hash>>,
    pub expected_value: Option<Box<str>>,
    pub actual_value: Option<Box<str>>,
    pub diagnostics: Vec<String>,
}

impl Phase8CommandError {
    pub fn new(command: Phase8CommandName, reason_code: impl Into<String>) -> Self {
        Self {
            command,
            reason_code: reason_code.into().into_boxed_str(),
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
        push_optional_hash_pair(
            &mut pairs,
            "expected_hash",
            self.expected_hash.as_deref().copied(),
        );
        push_optional_hash_pair(
            &mut pairs,
            "actual_hash",
            self.actual_hash.as_deref().copied(),
        );
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8RequestValidationError {
    pub field: Box<str>,
    pub expected_value: Option<Box<str>>,
    pub actual_value: Option<Box<str>>,
    pub expected_hash: Option<Box<Hash>>,
    pub actual_hash: Option<Box<Hash>>,
}

impl Phase8RequestValidationError {
    fn value_failure(
        field: impl Into<String>,
        expected_value: impl Into<String>,
        actual_value: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into().into_boxed_str(),
            expected_value: Some(expected_value.into().into_boxed_str()),
            actual_value: Some(actual_value.into().into_boxed_str()),
            expected_hash: None,
            actual_hash: None,
        }
    }

    fn hash_failure(field: impl Into<String>, expected_hash: Hash, actual_hash: Hash) -> Self {
        Self {
            field: field.into().into_boxed_str(),
            expected_value: None,
            actual_value: None,
            expected_hash: Some(Box::new(expected_hash)),
            actual_hash: Some(Box::new(actual_hash)),
        }
    }
}

impl From<Phase8PolicyValidationError> for Phase8RequestValidationError {
    fn from(error: Phase8PolicyValidationError) -> Self {
        Self::value_failure(error.field, error.expected_value, error.actual_value)
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
pub struct Phase8MachineCheckRequestPolicy {
    pub id: String,
    pub version: u64,
    pub hash: Hash,
}

impl Phase8MachineCheckRequestPolicy {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            ("hash".to_owned(), phase8_hash_json_literal(&self.hash)),
            ("id".to_owned(), phase8_json_string_literal(&self.id)),
            ("version".to_owned(), self.version.to_string()),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8MachineCheckRequestCertificate {
    pub path: String,
    pub file_hash: Hash,
    pub expected_certificate_hash: Hash,
}

impl Phase8MachineCheckRequestCertificate {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "expected_certificate_hash".to_owned(),
                phase8_hash_json_literal(&self.expected_certificate_hash),
            ),
            (
                "file_hash".to_owned(),
                phase8_hash_json_literal(&self.file_hash),
            ),
            ("kind".to_owned(), phase8_json_string_literal("path")),
            ("path".to_owned(), phase8_json_string_literal(&self.path)),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8MachineCheckRequestImports {
    pub mode: String,
    pub manifest: String,
    pub manifest_hash: Hash,
}

impl Phase8MachineCheckRequestImports {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "manifest".to_owned(),
                phase8_json_string_literal(&self.manifest),
            ),
            (
                "manifest_hash".to_owned(),
                phase8_hash_json_literal(&self.manifest_hash),
            ),
            ("mode".to_owned(), phase8_json_string_literal(&self.mode)),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8MachineCheckRequest {
    pub request_id: String,
    pub module: String,
    pub policy: Phase8MachineCheckRequestPolicy,
    pub certificate: Phase8MachineCheckRequestCertificate,
    pub imports: Phase8MachineCheckRequestImports,
    pub checker_profile: String,
    pub trust_mode: Phase8TrustMode,
    pub axiom_policy: String,
    pub budget: Phase8RunnerBudget,
}

impl Phase8MachineCheckRequest {
    pub fn request_hash(&self) -> Hash {
        phase8_sha256(self.hash_input_canonical_json().as_bytes())
    }

    pub fn hash_input_canonical_json(&self) -> String {
        canonical_json_object_from_pairs(self.semantic_canonical_json_pairs())
    }

    pub fn canonical_json(&self) -> String {
        let mut pairs = self.semantic_canonical_json_pairs();
        pairs.push((
            "request_hash".to_owned(),
            phase8_hash_json_literal(&self.request_hash()),
        ));
        pairs.push((
            "request_id".to_owned(),
            phase8_json_string_literal(&self.request_id),
        ));
        canonical_json_object_from_pairs(pairs)
    }

    fn semantic_canonical_json_pairs(&self) -> Vec<(String, String)> {
        vec![
            (
                "axiom_policy".to_owned(),
                phase8_json_string_literal(&self.axiom_policy),
            ),
            ("budget".to_owned(), self.budget.canonical_json()),
            ("certificate".to_owned(), self.certificate.canonical_json()),
            (
                "checker_profile".to_owned(),
                phase8_json_string_literal(&self.checker_profile),
            ),
            ("imports".to_owned(), self.imports.canonical_json()),
            (
                "module".to_owned(),
                phase8_json_string_literal(&self.module),
            ),
            ("policy".to_owned(), self.policy.canonical_json()),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_MACHINE_CHECK_REQUEST_SCHEMA),
            ),
            (
                "trust_mode".to_owned(),
                phase8_json_string_literal(self.trust_mode.as_str()),
            ),
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ImportLockCertificate {
    pub path: String,
    pub file_hash: Hash,
    pub certificate_hash: Hash,
}

impl Phase8ImportLockCertificate {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "certificate_hash".to_owned(),
                phase8_hash_json_literal(&self.certificate_hash),
            ),
            (
                "file_hash".to_owned(),
                phase8_hash_json_literal(&self.file_hash),
            ),
            ("kind".to_owned(), phase8_json_string_literal("path")),
            ("path".to_owned(), phase8_json_string_literal(&self.path)),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ImportLockEntry {
    pub module: String,
    pub export_hash: Hash,
    pub certificate: Phase8ImportLockCertificate,
}

impl Phase8ImportLockEntry {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            ("certificate".to_owned(), self.certificate.canonical_json()),
            (
                "export_hash".to_owned(),
                phase8_hash_json_literal(&self.export_hash),
            ),
            (
                "module".to_owned(),
                phase8_json_string_literal(&self.module),
            ),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ImportLockManifest {
    pub imports: Vec<Phase8ImportLockEntry>,
}

impl Phase8ImportLockManifest {
    pub fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "imports".to_owned(),
                canonical_json_array(
                    self.imports
                        .iter()
                        .map(Phase8ImportLockEntry::canonical_json)
                        .collect(),
                ),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_IMPORT_LOCK_MANIFEST_SCHEMA),
            ),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8RequestStoreEntry {
    pub request_hash: Hash,
    pub path: String,
    pub file_hash: Hash,
}

impl Phase8RequestStoreEntry {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "file_hash".to_owned(),
                phase8_hash_json_literal(&self.file_hash),
            ),
            ("path".to_owned(), phase8_json_string_literal(&self.path)),
            (
                "request_hash".to_owned(),
                phase8_hash_json_literal(&self.request_hash),
            ),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8RequestStoreManifest {
    pub requests: Vec<Phase8RequestStoreEntry>,
}

impl Phase8RequestStoreManifest {
    pub fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "requests".to_owned(),
                canonical_json_array(
                    self.requests
                        .iter()
                        .map(Phase8RequestStoreEntry::canonical_json)
                        .collect(),
                ),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_REQUEST_STORE_MANIFEST_SCHEMA),
            ),
        ])
    }

    pub fn file_hash(&self) -> Hash {
        phase8_file_hash(self.canonical_json().as_bytes())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8RequestStoreUpdate {
    pub manifest: Phase8RequestStoreManifest,
    pub rewrite_required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8RawCertificateClaims {
    pub module: String,
    pub certificate_hash: Hash,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8RawCertificateClaimError {
    DecodeFailed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8RequestMaterialization {
    pub request: Phase8MachineCheckRequest,
    pub request_file_hash: Hash,
    pub request_store: Phase8RequestStoreManifest,
    pub request_store_file_hash: Hash,
    pub request_store_rewrite_required: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8MachineCheckStatus {
    Checked,
    Failed,
}

impl Phase8MachineCheckStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Checked => "checked",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8MachineCheckError {
    pub kind: String,
    pub reason_code: Option<String>,
    pub field: Option<String>,
    pub declaration: Option<String>,
    pub core_path: Option<Vec<String>>,
    pub section: Option<String>,
    pub offset: Option<u64>,
    pub expected_hash: Option<Hash>,
    pub actual_hash: Option<Hash>,
    pub expected_value: Option<String>,
    pub actual_value: Option<String>,
}

impl Phase8MachineCheckError {
    pub fn new(kind: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            reason_code: None,
            field: None,
            declaration: None,
            core_path: None,
            section: None,
            offset: None,
            expected_hash: None,
            actual_hash: None,
            expected_value: None,
            actual_value: None,
        }
    }

    pub fn with_reason_code(mut self, reason_code: impl Into<String>) -> Self {
        self.reason_code = Some(reason_code.into());
        self
    }

    pub fn with_value_payload(
        mut self,
        field: impl Into<String>,
        expected_value: impl Into<String>,
        actual_value: impl Into<String>,
    ) -> Self {
        self.field = Some(field.into());
        self.expected_value = Some(expected_value.into());
        self.actual_value = Some(actual_value.into());
        self
    }

    pub fn with_hash_payload(
        mut self,
        field: impl Into<String>,
        expected_hash: Hash,
        actual_hash: Hash,
    ) -> Self {
        self.field = Some(field.into());
        self.expected_hash = Some(expected_hash);
        self.actual_hash = Some(actual_hash);
        self
    }

    fn canonical_json(&self) -> String {
        let mut pairs = vec![("kind".to_owned(), phase8_json_string_literal(&self.kind))];
        push_optional_string_pair(&mut pairs, "reason_code", self.reason_code.as_deref());
        push_optional_string_pair(&mut pairs, "field", self.field.as_deref());
        push_optional_string_pair(&mut pairs, "declaration", self.declaration.as_deref());
        if let Some(core_path) = &self.core_path {
            pairs.push(("core_path".to_owned(), phase8_json_string_array(core_path)));
        }
        push_optional_string_pair(&mut pairs, "section", self.section.as_deref());
        if let Some(offset) = self.offset {
            pairs.push(("offset".to_owned(), offset.to_string()));
        }
        push_optional_hash_pair(&mut pairs, "expected_hash", self.expected_hash);
        push_optional_hash_pair(&mut pairs, "actual_hash", self.actual_hash);
        push_optional_string_pair(&mut pairs, "expected_value", self.expected_value.as_deref());
        push_optional_string_pair(&mut pairs, "actual_value", self.actual_value.as_deref());
        canonical_json_object_from_pairs(pairs)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8MachineCheckRunner {
    pub id: String,
    pub version: String,
    pub build_hash: Hash,
}

impl Phase8MachineCheckRunner {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "build_hash".to_owned(),
                phase8_hash_json_literal(&self.build_hash),
            ),
            ("id".to_owned(), phase8_json_string_literal(&self.id)),
            (
                "version".to_owned(),
                phase8_json_string_literal(&self.version),
            ),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8MachineCheckChecker {
    pub profile: String,
    pub binary_id: Option<String>,
    pub binary_hash: Option<Hash>,
    pub id: Option<String>,
    pub build_hash: Option<Hash>,
    pub version: Option<String>,
}

impl Phase8MachineCheckChecker {
    fn canonical_json(&self) -> String {
        let mut pairs = vec![(
            "profile".to_owned(),
            phase8_json_string_literal(&self.profile),
        )];
        push_optional_string_pair(&mut pairs, "binary_id", self.binary_id.as_deref());
        push_optional_hash_pair(&mut pairs, "binary_hash", self.binary_hash);
        push_optional_string_pair(&mut pairs, "id", self.id.as_deref());
        push_optional_hash_pair(&mut pairs, "build_hash", self.build_hash);
        push_optional_string_pair(&mut pairs, "version", self.version.as_deref());
        canonical_json_object_from_pairs(pairs)
    }

    fn result_hash_projection_json(&self) -> String {
        let mut pairs = vec![(
            "profile".to_owned(),
            phase8_json_string_literal(&self.profile),
        )];
        push_optional_string_pair(&mut pairs, "binary_id", self.binary_id.as_deref());
        push_optional_hash_pair(&mut pairs, "binary_hash", self.binary_hash);
        push_optional_string_pair(&mut pairs, "id", self.id.as_deref());
        push_optional_hash_pair(&mut pairs, "build_hash", self.build_hash);
        canonical_json_object_from_pairs(pairs)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8MachineCheckProcess {
    pub launched: bool,
    pub exit_code: Option<u8>,
    pub termination_reason: Option<String>,
}

impl Phase8MachineCheckProcess {
    pub const fn not_launched() -> Self {
        Self {
            launched: false,
            exit_code: None,
            termination_reason: None,
        }
    }

    pub const fn exited(exit_code: u8) -> Self {
        Self {
            launched: true,
            exit_code: Some(exit_code),
            termination_reason: None,
        }
    }

    pub fn terminated(reason: impl Into<String>) -> Self {
        Self {
            launched: true,
            exit_code: None,
            termination_reason: Some(reason.into()),
        }
    }

    fn canonical_json(&self) -> String {
        let mut pairs = vec![("launched".to_owned(), self.launched.to_string())];
        if let Some(exit_code) = self.exit_code {
            pairs.push(("exit_code".to_owned(), exit_code.to_string()));
        }
        push_optional_string_pair(
            &mut pairs,
            "termination_reason",
            self.termination_reason.as_deref(),
        );
        canonical_json_object_from_pairs(pairs)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8MachineCheckResourceUsage {
    pub steps: u64,
    pub memory_peak_mb: u64,
    pub elapsed_ms: u64,
}

impl Phase8MachineCheckResourceUsage {
    pub const fn zero() -> Self {
        Self {
            steps: 0,
            memory_peak_mb: 0,
            elapsed_ms: 0,
        }
    }

    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            ("elapsed_ms".to_owned(), self.elapsed_ms.to_string()),
            ("memory_peak_mb".to_owned(), self.memory_peak_mb.to_string()),
            ("steps".to_owned(), self.steps.to_string()),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8MachineCheckResult {
    pub request_id: String,
    pub request_hash: Hash,
    pub result_id: String,
    pub policy: Phase8MachineCheckRequestPolicy,
    pub runner: Phase8MachineCheckRunner,
    pub checker: Phase8MachineCheckChecker,
    pub attempt: u64,
    pub status: Phase8MachineCheckStatus,
    pub module: String,
    pub process: Phase8MachineCheckProcess,
    pub resource_usage: Phase8MachineCheckResourceUsage,
    pub error: Option<Phase8MachineCheckError>,
    pub certificate_hash: Option<Hash>,
    pub export_hash: Option<Hash>,
    pub axiom_report_hash: Option<Hash>,
    pub diagnostics: Vec<String>,
    pub axioms_used: Option<Vec<String>>,
    pub declarations_checked: Option<u64>,
}

impl Phase8MachineCheckResult {
    pub fn result_hash(&self) -> Hash {
        phase8_sha256(self.result_hash_projection_json().as_bytes())
    }

    pub fn run_artifact_hash(&self) -> Hash {
        phase8_sha256(self.canonical_json_without_run_artifact_hash().as_bytes())
    }

    pub fn canonical_json(&self) -> String {
        let run_artifact_hash = self.run_artifact_hash();
        let mut pairs = self.canonical_json_pairs_with_result_hash();
        pairs.push((
            "run_artifact_hash".to_owned(),
            phase8_hash_json_literal(&run_artifact_hash),
        ));
        canonical_json_object_from_pairs(pairs)
    }

    pub fn result_hash_projection_json(&self) -> String {
        let mut pairs = vec![
            (
                "checker".to_owned(),
                self.checker.result_hash_projection_json(),
            ),
            (
                "module".to_owned(),
                phase8_json_string_literal(&self.module),
            ),
            ("policy".to_owned(), self.policy.canonical_json()),
            ("runner".to_owned(), self.runner.canonical_json()),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_MACHINE_CHECK_RESULT_SCHEMA),
            ),
            (
                "status".to_owned(),
                phase8_json_string_literal(self.status.as_str()),
            ),
        ];
        if let Some(error) = &self.error {
            pairs.push(("error".to_owned(), error.canonical_json()));
        }
        push_optional_hash_pair(&mut pairs, "certificate_hash", self.certificate_hash);
        push_optional_hash_pair(&mut pairs, "export_hash", self.export_hash);
        push_optional_hash_pair(&mut pairs, "axiom_report_hash", self.axiom_report_hash);
        canonical_json_object_from_pairs(pairs)
    }

    fn canonical_json_without_run_artifact_hash(&self) -> String {
        canonical_json_object_from_pairs(self.canonical_json_pairs_with_result_hash())
    }

    fn canonical_json_pairs_with_result_hash(&self) -> Vec<(String, String)> {
        let result_hash = self.result_hash();
        let mut pairs = vec![
            ("attempt".to_owned(), self.attempt.to_string()),
            ("checker".to_owned(), self.checker.canonical_json()),
            (
                "module".to_owned(),
                phase8_json_string_literal(&self.module),
            ),
            ("policy".to_owned(), self.policy.canonical_json()),
            ("process".to_owned(), self.process.canonical_json()),
            (
                "request_hash".to_owned(),
                phase8_hash_json_literal(&self.request_hash),
            ),
            (
                "request_id".to_owned(),
                phase8_json_string_literal(&self.request_id),
            ),
            (
                "resource_usage".to_owned(),
                self.resource_usage.canonical_json(),
            ),
            (
                "result_hash".to_owned(),
                phase8_hash_json_literal(&result_hash),
            ),
            (
                "result_id".to_owned(),
                phase8_json_string_literal(&self.result_id),
            ),
            ("runner".to_owned(), self.runner.canonical_json()),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_MACHINE_CHECK_RESULT_SCHEMA),
            ),
            (
                "status".to_owned(),
                phase8_json_string_literal(self.status.as_str()),
            ),
        ];
        if let Some(error) = &self.error {
            pairs.push(("error".to_owned(), error.canonical_json()));
        }
        push_optional_hash_pair(&mut pairs, "certificate_hash", self.certificate_hash);
        push_optional_hash_pair(&mut pairs, "export_hash", self.export_hash);
        push_optional_hash_pair(&mut pairs, "axiom_report_hash", self.axiom_report_hash);
        if !self.diagnostics.is_empty() {
            pairs.push((
                "diagnostics".to_owned(),
                phase8_json_string_array(&self.diagnostics),
            ));
        }
        if let Some(axioms_used) = &self.axioms_used {
            pairs.push((
                "axioms_used".to_owned(),
                phase8_json_string_array(axioms_used),
            ));
        }
        if let Some(declarations_checked) = self.declarations_checked {
            pairs.push((
                "declarations_checked".to_owned(),
                declarations_checked.to_string(),
            ));
        }
        pairs
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8CheckerRawResult {
    pub status: Phase8MachineCheckStatus,
    pub checker_id: Option<String>,
    pub checker_version: Option<String>,
    pub checker_build_hash: Option<Hash>,
    pub module: Option<String>,
    pub certificate_hash: Option<Hash>,
    pub export_hash: Option<Hash>,
    pub axiom_report_hash: Option<Hash>,
    pub error: Option<Phase8MachineCheckError>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8RawResultSchemaError {
    pub field: String,
    pub expected_value: String,
    pub actual_value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8CheckerRunObservation {
    pub result_id: String,
    pub attempt: u64,
    pub runner: Phase8MachineCheckRunner,
    pub process: Phase8MachineCheckProcess,
    pub resource_usage: Phase8MachineCheckResourceUsage,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8MachineResultStoreEntry {
    pub result_hash: Hash,
    pub request_hash: Hash,
    pub run_artifact_hash: Hash,
    pub checker_profile: String,
    pub path: String,
    pub file_hash: Hash,
}

impl Phase8MachineResultStoreEntry {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "checker_profile".to_owned(),
                phase8_json_string_literal(&self.checker_profile),
            ),
            (
                "file_hash".to_owned(),
                phase8_hash_json_literal(&self.file_hash),
            ),
            ("path".to_owned(), phase8_json_string_literal(&self.path)),
            (
                "request_hash".to_owned(),
                phase8_hash_json_literal(&self.request_hash),
            ),
            (
                "result_hash".to_owned(),
                phase8_hash_json_literal(&self.result_hash),
            ),
            (
                "run_artifact_hash".to_owned(),
                phase8_hash_json_literal(&self.run_artifact_hash),
            ),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8MachineResultStoreManifest {
    pub results: Vec<Phase8MachineResultStoreEntry>,
}

impl Phase8MachineResultStoreManifest {
    pub fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "results".to_owned(),
                canonical_json_array(
                    self.results
                        .iter()
                        .map(Phase8MachineResultStoreEntry::canonical_json)
                        .collect(),
                ),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_MACHINE_RESULT_STORE_MANIFEST_SCHEMA),
            ),
        ])
    }

    pub fn file_hash(&self) -> Hash {
        phase8_file_hash(self.canonical_json().as_bytes())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8MachineResultStoreUpdate {
    pub manifest: Phase8MachineResultStoreManifest,
    pub rewrite_required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8AxiomReportEntry {
    pub name: String,
}

impl Phase8AxiomReportEntry {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![(
            "name".to_owned(),
            phase8_json_string_literal(&self.name),
        )])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8AxiomReport {
    pub module: String,
    pub certificate_hash: Hash,
    pub axioms: Vec<Phase8AxiomReportEntry>,
}

impl Phase8AxiomReport {
    pub fn axiom_report_hash(&self) -> Hash {
        phase8_sha256(self.hash_input_canonical_json().as_bytes())
    }

    pub fn hash_input_canonical_json(&self) -> String {
        canonical_json_object_from_pairs(self.base_canonical_json_pairs())
    }

    pub fn canonical_json(&self) -> String {
        let mut pairs = self.base_canonical_json_pairs();
        pairs.push((
            "axiom_report_hash".to_owned(),
            phase8_hash_json_literal(&self.axiom_report_hash()),
        ));
        canonical_json_object_from_pairs(pairs)
    }

    fn base_canonical_json_pairs(&self) -> Vec<(String, String)> {
        vec![
            (
                "axioms".to_owned(),
                canonical_json_array(
                    self.axioms
                        .iter()
                        .map(Phase8AxiomReportEntry::canonical_json)
                        .collect(),
                ),
            ),
            (
                "certificate_hash".to_owned(),
                phase8_hash_json_literal(&self.certificate_hash),
            ),
            (
                "module".to_owned(),
                phase8_json_string_literal(&self.module),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_AXIOM_REPORT_SCHEMA),
            ),
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8AxiomReportStoreEntry {
    pub axiom_report_hash: Hash,
    pub path: String,
    pub file_hash: Hash,
}

impl Phase8AxiomReportStoreEntry {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "axiom_report_hash".to_owned(),
                phase8_hash_json_literal(&self.axiom_report_hash),
            ),
            (
                "file_hash".to_owned(),
                phase8_hash_json_literal(&self.file_hash),
            ),
            ("path".to_owned(), phase8_json_string_literal(&self.path)),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8AxiomReportStoreManifest {
    pub reports: Vec<Phase8AxiomReportStoreEntry>,
}

impl Phase8AxiomReportStoreManifest {
    pub fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "reports".to_owned(),
                canonical_json_array(
                    self.reports
                        .iter()
                        .map(Phase8AxiomReportStoreEntry::canonical_json)
                        .collect(),
                ),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_AXIOM_REPORT_STORE_MANIFEST_SCHEMA),
            ),
        ])
    }

    pub fn file_hash(&self) -> Hash {
        phase8_file_hash(self.canonical_json().as_bytes())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ArtifactSelector {
    pub module: String,
    pub request_hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8RequestStoreReference {
    pub path: String,
    pub manifest_hash: Hash,
}

impl Phase8RequestStoreReference {
    pub fn canonical_json(&self) -> String {
        phase8_store_reference_canonical_json(&self.path, self.manifest_hash)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8MachineResultStoreReference {
    pub path: String,
    pub manifest_hash: Hash,
}

impl Phase8MachineResultStoreReference {
    pub fn canonical_json(&self) -> String {
        phase8_store_reference_canonical_json(&self.path, self.manifest_hash)
    }
}

fn phase8_store_reference_canonical_json(path: &str, manifest_hash: Hash) -> String {
    canonical_json_object_from_pairs(vec![
        ("kind".to_owned(), phase8_json_string_literal("manifest")),
        (
            "manifest_hash".to_owned(),
            phase8_hash_json_literal(&manifest_hash),
        ),
        ("path".to_owned(), phase8_json_string_literal(path)),
    ])
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8StoredMachineCheckRequest {
    pub path: String,
    pub file_hash: Hash,
    pub request: Phase8MachineCheckRequest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8NormalizedArtifact {
    pub module: String,
    pub input_file_hash: Hash,
    pub expected_certificate_hash: Hash,
    pub import_lock_hash: Hash,
    pub axiom_policy_hash: Hash,
}

impl Phase8NormalizedArtifact {
    fn from_request(request: &Phase8MachineCheckRequest, policy: &Phase8RunnerPolicy) -> Self {
        Self {
            module: request.module.clone(),
            input_file_hash: request.certificate.file_hash,
            expected_certificate_hash: request.certificate.expected_certificate_hash,
            import_lock_hash: request.imports.manifest_hash,
            axiom_policy_hash: policy.axiom_policy.hash,
        }
    }

    pub fn artifact_hash(&self) -> Hash {
        phase8_sha256(self.canonical_json().as_bytes())
    }

    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "axiom_policy_hash".to_owned(),
                phase8_hash_json_literal(&self.axiom_policy_hash),
            ),
            (
                "expected_certificate_hash".to_owned(),
                phase8_hash_json_literal(&self.expected_certificate_hash),
            ),
            (
                "import_lock_hash".to_owned(),
                phase8_hash_json_literal(&self.import_lock_hash),
            ),
            (
                "input_file_hash".to_owned(),
                phase8_hash_json_literal(&self.input_file_hash),
            ),
            (
                "module".to_owned(),
                phase8_json_string_literal(&self.module),
            ),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8NormalizedFailureKey {
    pub kind: String,
    pub reason_code: Option<String>,
    pub field: Option<String>,
    pub declaration: Option<String>,
    pub core_path: Option<Vec<String>>,
    pub section: Option<String>,
    pub offset: Option<u64>,
    pub expected_hash: Option<Hash>,
    pub actual_hash: Option<Hash>,
    pub expected_value: Option<String>,
    pub actual_value: Option<String>,
}

impl Phase8NormalizedFailureKey {
    pub fn from_error(error: &Phase8MachineCheckError) -> Self {
        Self {
            kind: error.kind.clone(),
            reason_code: error.reason_code.clone(),
            field: error.field.clone(),
            declaration: error.declaration.clone(),
            core_path: error.core_path.clone(),
            section: error.section.clone(),
            offset: error.offset,
            expected_hash: error.expected_hash,
            actual_hash: error.actual_hash,
            expected_value: error.expected_value.clone(),
            actual_value: error.actual_value.clone(),
        }
    }

    pub fn failure_key_hash(&self) -> Hash {
        phase8_sha256(self.canonical_json().as_bytes())
    }

    fn canonical_json(&self) -> String {
        let mut pairs = vec![("kind".to_owned(), phase8_json_string_literal(&self.kind))];
        push_optional_string_pair(&mut pairs, "reason_code", self.reason_code.as_deref());
        push_optional_string_pair(&mut pairs, "field", self.field.as_deref());
        push_optional_string_pair(&mut pairs, "declaration", self.declaration.as_deref());
        if let Some(core_path) = &self.core_path {
            pairs.push(("core_path".to_owned(), phase8_json_string_array(core_path)));
        }
        push_optional_string_pair(&mut pairs, "section", self.section.as_deref());
        if let Some(offset) = self.offset {
            pairs.push(("offset".to_owned(), offset.to_string()));
        }
        push_optional_hash_pair(&mut pairs, "expected_hash", self.expected_hash);
        push_optional_hash_pair(&mut pairs, "actual_hash", self.actual_hash);
        push_optional_string_pair(&mut pairs, "expected_value", self.expected_value.as_deref());
        push_optional_string_pair(&mut pairs, "actual_value", self.actual_value.as_deref());
        canonical_json_object_from_pairs(pairs)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8NormalizedCheckResultEntry {
    pub result_id: String,
    pub result_hash: Hash,
    pub request_hash: Hash,
    pub policy_hash: Hash,
    pub artifact_hash: Hash,
    pub checker_profile: String,
    pub process_launched: bool,
    pub status: Phase8MachineCheckStatus,
    pub checker_binary_id: Option<String>,
    pub checker_binary_hash: Option<Hash>,
    pub checker_id: Option<String>,
    pub checker_build_hash: Option<Hash>,
    pub certificate_hash: Option<Hash>,
    pub export_hash: Option<Hash>,
    pub axiom_report_hash: Option<Hash>,
    pub error: Option<Phase8MachineCheckError>,
    pub failure_key: Option<Phase8NormalizedFailureKey>,
}

impl Phase8NormalizedCheckResultEntry {
    fn canonical_json(&self) -> String {
        self.canonical_json_with_result_id(true)
    }

    fn normalized_hash_projection_json(&self) -> String {
        self.canonical_json_with_result_id(false)
    }

    fn canonical_json_with_result_id(&self, include_result_id: bool) -> String {
        let mut pairs = vec![
            (
                "artifact_hash".to_owned(),
                phase8_hash_json_literal(&self.artifact_hash),
            ),
            (
                "checker_profile".to_owned(),
                phase8_json_string_literal(&self.checker_profile),
            ),
            (
                "policy_hash".to_owned(),
                phase8_hash_json_literal(&self.policy_hash),
            ),
            (
                "process_launched".to_owned(),
                self.process_launched.to_string(),
            ),
            (
                "request_hash".to_owned(),
                phase8_hash_json_literal(&self.request_hash),
            ),
            (
                "result_hash".to_owned(),
                phase8_hash_json_literal(&self.result_hash),
            ),
            (
                "status".to_owned(),
                phase8_json_string_literal(self.status.as_str()),
            ),
        ];
        if include_result_id {
            pairs.push((
                "result_id".to_owned(),
                phase8_json_string_literal(&self.result_id),
            ));
        }
        push_optional_string_pair(
            &mut pairs,
            "checker_binary_id",
            self.checker_binary_id.as_deref(),
        );
        push_optional_hash_pair(&mut pairs, "checker_binary_hash", self.checker_binary_hash);
        push_optional_string_pair(&mut pairs, "checker_id", self.checker_id.as_deref());
        push_optional_hash_pair(&mut pairs, "checker_build_hash", self.checker_build_hash);
        push_optional_hash_pair(&mut pairs, "certificate_hash", self.certificate_hash);
        push_optional_hash_pair(&mut pairs, "export_hash", self.export_hash);
        push_optional_hash_pair(&mut pairs, "axiom_report_hash", self.axiom_report_hash);
        if let Some(error) = &self.error {
            pairs.push(("error".to_owned(), error.canonical_json()));
        }
        if let Some(failure_key) = &self.failure_key {
            pairs.push(("failure_key".to_owned(), failure_key.canonical_json()));
        }
        canonical_json_object_from_pairs(pairs)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8NormalizedComparisonStatus {
    AllAgreeChecked,
    AllAgreeFailed,
    Disagreement,
    MissingCheckerResult,
    PolicyFailure,
    Inconclusive,
}

impl Phase8NormalizedComparisonStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AllAgreeChecked => "all_agree_checked",
            Self::AllAgreeFailed => "all_agree_failed",
            Self::Disagreement => "disagreement",
            Self::MissingCheckerResult => "missing_checker_result",
            Self::PolicyFailure => "policy_failure",
            Self::Inconclusive => "inconclusive",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8NormalizedDisagreement {
    pub field: String,
    pub baseline_checker_profile: Option<String>,
    pub baseline_hash: Option<Hash>,
    pub baseline_value: Option<String>,
    pub checker_profile: String,
    pub actual_hash: Option<Hash>,
    pub actual_value: Option<String>,
}

impl Phase8NormalizedDisagreement {
    fn canonical_json(&self) -> String {
        let mut pairs = vec![
            ("field".to_owned(), phase8_json_string_literal(&self.field)),
            (
                "checker_profile".to_owned(),
                phase8_json_string_literal(&self.checker_profile),
            ),
        ];
        push_optional_string_pair(
            &mut pairs,
            "baseline_checker_profile",
            self.baseline_checker_profile.as_deref(),
        );
        push_optional_hash_pair(&mut pairs, "baseline_hash", self.baseline_hash);
        push_optional_string_pair(&mut pairs, "baseline_value", self.baseline_value.as_deref());
        push_optional_hash_pair(&mut pairs, "actual_hash", self.actual_hash);
        push_optional_string_pair(&mut pairs, "actual_value", self.actual_value.as_deref());
        canonical_json_object_from_pairs(pairs)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8NormalizedStatusReason {
    pub checker_profile: String,
    pub reason_code: String,
    pub field: Option<String>,
}

impl Phase8NormalizedStatusReason {
    fn canonical_json(&self) -> String {
        let mut pairs = vec![
            (
                "checker_profile".to_owned(),
                phase8_json_string_literal(&self.checker_profile),
            ),
            (
                "reason_code".to_owned(),
                phase8_json_string_literal(&self.reason_code),
            ),
        ];
        push_optional_string_pair(&mut pairs, "field", self.field.as_deref());
        canonical_json_object_from_pairs(pairs)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8NormalizedComparison {
    pub status: Phase8NormalizedComparisonStatus,
    pub matching_fields: Vec<String>,
    pub missing_checker_profiles: Vec<String>,
    pub disagreements: Vec<Phase8NormalizedDisagreement>,
    pub status_reasons: Vec<Phase8NormalizedStatusReason>,
}

impl Phase8NormalizedComparison {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "disagreements".to_owned(),
                canonical_json_array(
                    self.disagreements
                        .iter()
                        .map(Phase8NormalizedDisagreement::canonical_json)
                        .collect(),
                ),
            ),
            (
                "matching_fields".to_owned(),
                phase8_json_string_array(&self.matching_fields),
            ),
            (
                "missing_checker_profiles".to_owned(),
                phase8_json_string_array(&self.missing_checker_profiles),
            ),
            (
                "status".to_owned(),
                phase8_json_string_literal(self.status.as_str()),
            ),
            (
                "status_reasons".to_owned(),
                canonical_json_array(
                    self.status_reasons
                        .iter()
                        .map(Phase8NormalizedStatusReason::canonical_json)
                        .collect(),
                ),
            ),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8NormalizedCheckResult {
    pub normalized_result_id: String,
    pub artifact: Phase8NormalizedArtifact,
    pub policy: Phase8MachineCheckRequestPolicy,
    pub results: Vec<Phase8NormalizedCheckResultEntry>,
    pub comparison: Phase8NormalizedComparison,
}

impl Phase8NormalizedCheckResult {
    pub fn artifact_hash(&self) -> Hash {
        self.artifact.artifact_hash()
    }

    pub fn normalized_result_hash(&self) -> Hash {
        phase8_sha256(self.hash_input_canonical_json().as_bytes())
    }

    pub fn hash_input_canonical_json(&self) -> String {
        let artifact_hash = self.artifact_hash();
        canonical_json_object_from_pairs(vec![
            ("artifact".to_owned(), self.artifact.canonical_json()),
            (
                "artifact_hash".to_owned(),
                phase8_hash_json_literal(&artifact_hash),
            ),
            ("comparison".to_owned(), self.comparison.canonical_json()),
            ("policy".to_owned(), self.policy.canonical_json()),
            (
                "results".to_owned(),
                canonical_json_array(
                    self.results
                        .iter()
                        .map(Phase8NormalizedCheckResultEntry::normalized_hash_projection_json)
                        .collect(),
                ),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_NORMALIZED_CHECK_RESULT_SCHEMA),
            ),
        ])
    }

    pub fn canonical_json(&self) -> String {
        let artifact_hash = self.artifact_hash();
        let normalized_result_hash = self.normalized_result_hash();
        canonical_json_object_from_pairs(vec![
            ("artifact".to_owned(), self.artifact.canonical_json()),
            (
                "artifact_hash".to_owned(),
                phase8_hash_json_literal(&artifact_hash),
            ),
            ("comparison".to_owned(), self.comparison.canonical_json()),
            (
                "normalized_result_hash".to_owned(),
                phase8_hash_json_literal(&normalized_result_hash),
            ),
            (
                "normalized_result_id".to_owned(),
                phase8_json_string_literal(&self.normalized_result_id),
            ),
            ("policy".to_owned(), self.policy.canonical_json()),
            (
                "results".to_owned(),
                canonical_json_array(
                    self.results
                        .iter()
                        .map(Phase8NormalizedCheckResultEntry::canonical_json)
                        .collect(),
                ),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_NORMALIZED_CHECK_RESULT_SCHEMA),
            ),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8NormalizedResultStoreEntry {
    pub normalized_result_hash: Hash,
    pub artifact_hash: Hash,
    pub path: String,
    pub file_hash: Hash,
}

impl Phase8NormalizedResultStoreEntry {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "artifact_hash".to_owned(),
                phase8_hash_json_literal(&self.artifact_hash),
            ),
            (
                "file_hash".to_owned(),
                phase8_hash_json_literal(&self.file_hash),
            ),
            (
                "normalized_result_hash".to_owned(),
                phase8_hash_json_literal(&self.normalized_result_hash),
            ),
            ("path".to_owned(), phase8_json_string_literal(&self.path)),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8NormalizedResultStoreManifest {
    pub results: Vec<Phase8NormalizedResultStoreEntry>,
}

impl Phase8NormalizedResultStoreManifest {
    pub fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "results".to_owned(),
                canonical_json_array(
                    self.results
                        .iter()
                        .map(Phase8NormalizedResultStoreEntry::canonical_json)
                        .collect(),
                ),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_NORMALIZED_RESULT_STORE_MANIFEST_SCHEMA),
            ),
        ])
    }

    pub fn file_hash(&self) -> Hash {
        phase8_file_hash(self.canonical_json().as_bytes())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8NormalizedResultStoreUpdate {
    pub manifest: Phase8NormalizedResultStoreManifest,
    pub rewrite_required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8NormalizationWriteStore {
    pub path: String,
    pub manifest_hash: Hash,
}

impl Phase8NormalizationWriteStore {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            ("kind".to_owned(), phase8_json_string_literal("manifest")),
            (
                "manifest_hash".to_owned(),
                phase8_hash_json_literal(&self.manifest_hash),
            ),
            ("path".to_owned(), phase8_json_string_literal(&self.path)),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8NormalizationWriteResult {
    pub normalized_result_hash: Hash,
    pub artifact_hash: Hash,
    pub output_path: String,
    pub output_file_hash: Hash,
    pub normalized_store: Option<Phase8NormalizationWriteStore>,
}

impl Phase8NormalizationWriteResult {
    pub fn canonical_json(&self) -> String {
        let mut pairs = vec![
            (
                "artifact_hash".to_owned(),
                phase8_hash_json_literal(&self.artifact_hash),
            ),
            (
                "normalized_result_hash".to_owned(),
                phase8_hash_json_literal(&self.normalized_result_hash),
            ),
            (
                "output_file_hash".to_owned(),
                phase8_hash_json_literal(&self.output_file_hash),
            ),
            (
                "output_path".to_owned(),
                phase8_json_string_literal(&self.output_path),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_NORMALIZATION_WRITE_RESULT_SCHEMA),
            ),
            ("status".to_owned(), phase8_json_string_literal("written")),
        ];
        if let Some(store) = &self.normalized_store {
            pairs.push(("normalized_store".to_owned(), store.canonical_json()));
        }
        canonical_json_object_from_pairs(pairs)
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

pub fn parse_phase8_import_lock_manifest(
    source: &str,
) -> Result<Phase8ImportLockManifest, Phase8RequestValidationError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        Phase8RequestValidationError::value_failure(
            "imports.manifest",
            "valid_json",
            "invalid_json",
        )
    })?;
    parse_phase8_import_lock_manifest_value(document.root(), "imports.manifest")
}

pub fn parse_phase8_machine_check_request(
    source: &str,
) -> Result<Phase8MachineCheckRequest, Phase8RequestValidationError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        Phase8RequestValidationError::value_failure("request", "valid_json", "invalid_json")
    })?;
    parse_phase8_machine_check_request_value(document.root())
}

pub fn parse_phase8_request_store_manifest(
    source: &str,
) -> Result<Phase8RequestStoreManifest, Phase8RequestValidationError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        Phase8RequestValidationError::value_failure("request_store", "valid_json", "invalid_json")
    })?;
    parse_phase8_request_store_manifest_value(document.root(), "request_store")
}

pub fn phase8_machine_check_request_hash(
    source: &str,
) -> Result<Hash, Phase8RequestValidationError> {
    Ok(parse_phase8_machine_check_request(source)?.request_hash())
}

pub fn phase8_raw_certificate_claims(
    certificate_bytes: &[u8],
) -> Result<Phase8RawCertificateClaims, Phase8RawCertificateClaimError> {
    let module = phase8_raw_certificate_header_module(certificate_bytes)?;
    let certificate_hash = phase8_raw_claimed_certificate_hash(certificate_bytes)?;
    Ok(Phase8RawCertificateClaims {
        module,
        certificate_hash,
    })
}

pub fn phase8_raw_claimed_certificate_hash(
    certificate_bytes: &[u8],
) -> Result<Hash, Phase8RawCertificateClaimError> {
    let Some(hash_bytes) = certificate_bytes.get(certificate_bytes.len().saturating_sub(32)..)
    else {
        return Err(Phase8RawCertificateClaimError::DecodeFailed);
    };
    if certificate_bytes.len() < 32 {
        return Err(Phase8RawCertificateClaimError::DecodeFailed);
    }
    let mut hash = [0; 32];
    hash.copy_from_slice(hash_bytes);
    Ok(hash)
}

#[allow(clippy::too_many_arguments)]
pub fn phase8_request_materialize(
    policy: &Phase8RunnerPolicy,
    module: impl Into<String>,
    certificate_path: impl Into<String>,
    certificate_bytes: &[u8],
    imports_manifest_path: impl Into<String>,
    imports_manifest_bytes: &[u8],
    expected_imports_manifest_hash: Hash,
    checker_profile: impl Into<String>,
    request_id: impl Into<String>,
    output_request_path: impl Into<String>,
    existing_store: Option<&Phase8RequestStoreManifest>,
) -> Result<Phase8RequestMaterialization, Phase8CommandError> {
    let module = module.into();
    let certificate_path = certificate_path.into();
    let imports_manifest_path = imports_manifest_path.into();
    let checker_profile = checker_profile.into();
    let request_id = request_id.into();
    let output_request_path = output_request_path.into();

    validate_request_materialize_input_shape(
        &module,
        &checker_profile,
        &request_id,
        &certificate_path,
        &imports_manifest_path,
        &output_request_path,
    )?;

    let actual_imports_manifest_hash = phase8_file_hash(imports_manifest_bytes);
    if actual_imports_manifest_hash != expected_imports_manifest_hash {
        return Err(phase8_command_hash_error(
            Phase8CommandName::RequestMaterialize,
            "input_hash_mismatch",
            "imports.manifest_hash",
            expected_imports_manifest_hash,
            actual_imports_manifest_hash,
        ));
    }
    let import_lock_source = std::str::from_utf8(imports_manifest_bytes).map_err(|_| {
        phase8_command_value_error(
            Phase8CommandName::RequestMaterialize,
            "input_json_invalid",
            "imports.manifest",
            "valid_json",
            "invalid_json",
        )
    })?;
    parse_phase8_import_lock_manifest(import_lock_source).map_err(|error| {
        let field = error.field.to_string();
        phase8_command_error_from_request_validation(
            Phase8CommandName::RequestMaterialize,
            "input_schema_invalid",
            field,
            error,
        )
    })?;

    let selected_budget = policy.budgets.get(&checker_profile);
    if policy.selected_checker_policy(&checker_profile).is_none() || selected_budget.is_none() {
        return Err(phase8_command_value_error(
            Phase8CommandName::RequestMaterialize,
            "request_materialization_failed",
            "checker_profile",
            "required_or_optional_checker_profile",
            checker_profile,
        ));
    }

    let claims = phase8_raw_certificate_claims(certificate_bytes).map_err(|_| {
        phase8_command_value_error(
            Phase8CommandName::RequestMaterialize,
            "request_materialization_failed",
            "certificate.expected_certificate_hash",
            "raw_claimed_certificate_hash",
            "decode_failed",
        )
    })?;
    if claims.module != module {
        return Err(phase8_command_value_error(
            Phase8CommandName::RequestMaterialize,
            "request_materialization_failed",
            "module",
            module,
            claims.module,
        ));
    }

    let request = Phase8MachineCheckRequest {
        request_id,
        module,
        policy: Phase8MachineCheckRequestPolicy {
            id: policy.id.clone(),
            version: policy.version,
            hash: policy.policy_hash(),
        },
        certificate: Phase8MachineCheckRequestCertificate {
            path: certificate_path,
            file_hash: phase8_file_hash(certificate_bytes),
            expected_certificate_hash: claims.certificate_hash,
        },
        imports: Phase8MachineCheckRequestImports {
            mode: policy.import_policy.mode.clone(),
            manifest: imports_manifest_path,
            manifest_hash: expected_imports_manifest_hash,
        },
        checker_profile: checker_profile.clone(),
        trust_mode: policy.trust_mode,
        axiom_policy: policy.axiom_policy.path.clone(),
        budget: selected_budget
            .expect("selected budget checked above")
            .clone(),
    };
    let request_file_hash = phase8_file_hash(request.canonical_json().as_bytes());
    let generated_entry = Phase8RequestStoreEntry {
        request_hash: request.request_hash(),
        path: output_request_path,
        file_hash: request_file_hash,
    };
    let store_update =
        phase8_request_store_with_materialized_entry(existing_store, generated_entry)?;
    let request_store_file_hash = store_update.manifest.file_hash();

    Ok(Phase8RequestMaterialization {
        request,
        request_file_hash,
        request_store: store_update.manifest,
        request_store_file_hash,
        request_store_rewrite_required: store_update.rewrite_required,
    })
}

pub fn phase8_request_store_with_materialized_entry(
    existing_store: Option<&Phase8RequestStoreManifest>,
    generated_entry: Phase8RequestStoreEntry,
) -> Result<Phase8RequestStoreUpdate, Phase8CommandError> {
    let mut manifest = existing_store
        .cloned()
        .unwrap_or(Phase8RequestStoreManifest {
            requests: Vec::new(),
        });

    for existing in &manifest.requests {
        let same_request_hash = existing.request_hash == generated_entry.request_hash;
        let same_path = existing.path == generated_entry.path;
        let same_file_hash = existing.file_hash == generated_entry.file_hash;
        if same_request_hash && same_path && same_file_hash {
            return Ok(Phase8RequestStoreUpdate {
                manifest,
                rewrite_required: false,
            });
        }
        if same_request_hash || same_path {
            return Err(phase8_command_value_error(
                Phase8CommandName::RequestMaterialize,
                "request_store_entry_conflict",
                "request_store.requests[]",
                generated_entry.canonical_json(),
                existing.canonical_json(),
            ));
        }
    }

    manifest.requests.push(generated_entry);
    manifest
        .requests
        .sort_by(|left, right| left.request_hash.cmp(&right.request_hash));
    Ok(Phase8RequestStoreUpdate {
        manifest,
        rewrite_required: true,
    })
}

pub fn parse_phase8_checker_raw_result(
    source: &str,
) -> Result<Phase8CheckerRawResult, Phase8RawResultSchemaError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        Phase8RawResultSchemaError::new("checker_raw", "valid_json", "invalid_json")
    })?;
    parse_phase8_checker_raw_result_value(document.root())
}

pub fn phase8_machine_check_run(
    request: &Phase8MachineCheckRequest,
    policy: &Phase8RunnerPolicy,
    observation: Phase8CheckerRunObservation,
) -> Result<Phase8MachineCheckResult, Phase8CommandError> {
    if !phase8_valid_request_id(&observation.result_id) {
        return Err(phase8_command_value_error(
            Phase8CommandName::Run,
            "input_reference_invalid",
            "result_id",
            "result_id",
            if observation.result_id.is_empty() {
                "empty_string"
            } else {
                "invalid_string_format"
            },
        ));
    }
    if observation.attempt == 0 || observation.attempt > i64::MAX as u64 {
        return Err(phase8_command_value_error(
            Phase8CommandName::Run,
            "input_reference_invalid",
            "attempt",
            "positive_i64",
            if observation.attempt == 0 {
                "non_positive_integer"
            } else {
                "integer_out_of_range"
            },
        ));
    }

    if request.policy.id != policy.id {
        return Ok(phase8_policy_failure_result(
            request,
            policy,
            &observation,
            "request_policy_hash_mismatch",
            Phase8MachineCheckError::new("policy_failure")
                .with_reason_code("request_policy_hash_mismatch")
                .with_value_payload("policy.id", &policy.id, &request.policy.id),
        ));
    }
    if request.policy.version != policy.version {
        return Ok(phase8_policy_failure_result(
            request,
            policy,
            &observation,
            "request_policy_hash_mismatch",
            Phase8MachineCheckError::new("policy_failure")
                .with_reason_code("request_policy_hash_mismatch")
                .with_value_payload(
                    "policy.version",
                    policy.version.to_string(),
                    request.policy.version.to_string(),
                ),
        ));
    }
    let loaded_policy_hash = policy.policy_hash();
    if request.policy.hash != loaded_policy_hash {
        return Ok(phase8_policy_failure_result(
            request,
            policy,
            &observation,
            "request_policy_hash_mismatch",
            Phase8MachineCheckError::new("policy_failure")
                .with_reason_code("request_policy_hash_mismatch")
                .with_hash_payload("policy.hash", loaded_policy_hash, request.policy.hash),
        ));
    }
    if request.trust_mode != policy.trust_mode {
        return Ok(phase8_policy_failure_result(
            request,
            policy,
            &observation,
            "request_trust_mode_mismatch",
            Phase8MachineCheckError::new("policy_failure")
                .with_reason_code("request_trust_mode_mismatch")
                .with_value_payload(
                    "trust_mode",
                    policy.trust_mode.as_str(),
                    request.trust_mode.as_str(),
                ),
        ));
    }
    if request.axiom_policy != policy.axiom_policy.path {
        return Ok(phase8_policy_failure_result(
            request,
            policy,
            &observation,
            "request_axiom_policy_mismatch",
            Phase8MachineCheckError::new("policy_failure")
                .with_reason_code("request_axiom_policy_mismatch")
                .with_value_payload(
                    "axiom_policy",
                    &policy.axiom_policy.path,
                    &request.axiom_policy,
                ),
        ));
    }
    if request.imports.mode != policy.import_policy.mode {
        return Ok(phase8_policy_failure_result(
            request,
            policy,
            &observation,
            "request_import_mode_mismatch",
            Phase8MachineCheckError::new("policy_failure")
                .with_reason_code("request_import_mode_mismatch")
                .with_value_payload(
                    "imports.mode",
                    &policy.import_policy.mode,
                    &request.imports.mode,
                ),
        ));
    }

    let Some(selected_checker) = policy.selected_checker_policy(&request.checker_profile) else {
        return Ok(phase8_policy_failure_result(
            request,
            policy,
            &observation,
            "request_checker_profile_not_allowed",
            Phase8MachineCheckError::new("policy_failure")
                .with_reason_code("request_checker_profile_not_allowed")
                .with_value_payload(
                    "checker_profile",
                    "policy_allowed_checker_profile",
                    &request.checker_profile,
                ),
        ));
    };
    let Some(selected_budget) = policy.budgets.get(&request.checker_profile) else {
        return Ok(phase8_policy_failure_result(
            request,
            policy,
            &observation,
            "request_checker_profile_not_allowed",
            Phase8MachineCheckError::new("policy_failure")
                .with_reason_code("request_checker_profile_not_allowed")
                .with_value_payload(
                    "checker_profile",
                    "policy_allowed_checker_profile",
                    &request.checker_profile,
                ),
        ));
    };
    if selected_budget != &request.budget {
        return Ok(phase8_policy_failure_result(
            request,
            policy,
            &observation,
            "request_budget_mismatch",
            phase8_budget_mismatch_error(selected_budget, &request.budget),
        ));
    }

    if !observation.process.launched {
        return Ok(phase8_infrastructure_failure_result(
            request,
            policy,
            selected_checker,
            &observation,
            "checker_internal_error",
            "process_not_launched",
        ));
    }

    Ok(phase8_adopt_checker_observation(
        request,
        policy,
        selected_checker,
        observation,
    ))
}

pub fn parse_phase8_machine_result_store_manifest(
    source: &str,
) -> Result<Phase8MachineResultStoreManifest, Phase8RequestValidationError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        Phase8RequestValidationError::value_failure("result_store", "valid_json", "invalid_json")
    })?;
    parse_phase8_machine_result_store_manifest_value(document.root(), "result_store")
}

pub fn phase8_machine_result_store_entry_for_result(
    result: &Phase8MachineCheckResult,
    path: impl Into<String>,
) -> Result<Phase8MachineResultStoreEntry, Phase8CommandError> {
    let path = path.into();
    if !phase8_valid_workspace_relative_path(&path) {
        return Err(phase8_command_value_error(
            Phase8CommandName::Run,
            "input_reference_invalid",
            "result.path",
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    let bytes = result.canonical_json();
    Ok(Phase8MachineResultStoreEntry {
        result_hash: result.result_hash(),
        request_hash: result.request_hash,
        run_artifact_hash: result.run_artifact_hash(),
        checker_profile: result.checker.profile.clone(),
        path,
        file_hash: phase8_file_hash(bytes.as_bytes()),
    })
}

pub fn phase8_machine_result_store_with_entry(
    existing_store: Option<&Phase8MachineResultStoreManifest>,
    generated_entry: Phase8MachineResultStoreEntry,
) -> Result<Phase8MachineResultStoreUpdate, Phase8CommandError> {
    let mut manifest = existing_store
        .cloned()
        .unwrap_or(Phase8MachineResultStoreManifest {
            results: Vec::new(),
        });

    for existing in &manifest.results {
        let same_run_artifact_hash =
            existing.run_artifact_hash == generated_entry.run_artifact_hash;
        let same_path = existing.path == generated_entry.path;
        let exact = same_run_artifact_hash
            && same_path
            && existing.file_hash == generated_entry.file_hash
            && existing.result_hash == generated_entry.result_hash
            && existing.request_hash == generated_entry.request_hash
            && existing.checker_profile == generated_entry.checker_profile;
        if exact {
            return Ok(Phase8MachineResultStoreUpdate {
                manifest,
                rewrite_required: false,
            });
        }
        if same_run_artifact_hash || same_path {
            return Err(phase8_command_value_error(
                Phase8CommandName::Run,
                "result_store_entry_conflict",
                "result_store.results[]",
                generated_entry.canonical_json(),
                existing.canonical_json(),
            ));
        }
    }

    manifest.results.push(generated_entry);
    manifest
        .results
        .sort_by(|left, right| left.run_artifact_hash.cmp(&right.run_artifact_hash));
    Ok(Phase8MachineResultStoreUpdate {
        manifest,
        rewrite_required: true,
    })
}

pub fn parse_phase8_axiom_report(
    source: &str,
) -> Result<Phase8AxiomReport, Phase8RequestValidationError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        Phase8RequestValidationError::value_failure("axiom_report", "valid_json", "invalid_json")
    })?;
    parse_phase8_axiom_report_value(document.root())
}

pub fn parse_phase8_axiom_report_store_manifest(
    source: &str,
) -> Result<Phase8AxiomReportStoreManifest, Phase8RequestValidationError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        Phase8RequestValidationError::value_failure(
            "axiom_report_store",
            "valid_json",
            "invalid_json",
        )
    })?;
    parse_phase8_axiom_report_store_manifest_value(document.root())
}

pub fn parse_phase8_request_store_reference(
    source: &str,
) -> Result<Phase8RequestStoreReference, Phase8RequestValidationError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        Phase8RequestValidationError::value_failure("request_store", "valid_json", "invalid_json")
    })?;
    let (path, manifest_hash) =
        parse_phase8_store_reference_value(document.root(), "request_store")?;
    Ok(Phase8RequestStoreReference {
        path,
        manifest_hash,
    })
}

pub fn parse_phase8_machine_result_store_reference(
    source: &str,
) -> Result<Phase8MachineResultStoreReference, Phase8RequestValidationError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        Phase8RequestValidationError::value_failure("result_store", "valid_json", "invalid_json")
    })?;
    let (path, manifest_hash) =
        parse_phase8_store_reference_value(document.root(), "result_store")?;
    Ok(Phase8MachineResultStoreReference {
        path,
        manifest_hash,
    })
}

pub fn phase8_normalize_results(
    normalized_result_id: impl Into<String>,
    normalize_error_result_id: impl Into<String>,
    policy: &Phase8RunnerPolicy,
    request_store_manifest: &Phase8RequestStoreManifest,
    stored_requests: &[Phase8StoredMachineCheckRequest],
    machine_results: &[Phase8MachineCheckResult],
    selector: Option<Phase8ArtifactSelector>,
) -> Result<Phase8NormalizedCheckResult, Phase8NormalizeErrorResult> {
    let normalized_result_id = normalized_result_id.into();
    let normalize_error_result_id = normalize_error_result_id.into();
    let policy_hash = policy.policy_hash();

    if !phase8_valid_request_id(&normalized_result_id) {
        return Err(phase8_normalize_value_error(
            normalize_error_result_id,
            policy_hash,
            Phase8NormalizeErrorReasonCode::SelectorSchemaInvalid,
            "normalized_result_id",
            "normalized_result_id",
            if normalized_result_id.is_empty() {
                "empty_string"
            } else {
                "invalid_string_format"
            },
        ));
    }

    if machine_results.is_empty() {
        return Err(phase8_normalize_value_error(
            normalize_error_result_id,
            policy_hash,
            Phase8NormalizeErrorReasonCode::SelectorAmbiguous,
            "machine_results",
            "one_or_more_machine_results",
            "empty",
        ));
    }

    if let Some(selector) = &selector {
        if !phase8_valid_dotted_name(&selector.module) {
            return Err(phase8_normalize_value_error(
                normalize_error_result_id,
                policy_hash,
                Phase8NormalizeErrorReasonCode::SelectorSchemaInvalid,
                "artifact_selector.module",
                "module_name",
                "invalid_name_format",
            ));
        }
    }

    if let Err(error) =
        validate_request_store_domain(&request_store_manifest.requests, "request_store")
    {
        return Err(phase8_normalize_request_validation_error(
            normalize_error_result_id,
            policy_hash,
            Phase8NormalizeErrorReasonCode::RequestStoreManifestInvalid,
            error,
        ));
    }

    let mut seen_profiles = BTreeSet::new();
    for result in machine_results {
        if !seen_profiles.insert(result.checker.profile.clone()) {
            return Err(phase8_normalize_value_error(
                normalize_error_result_id,
                policy_hash,
                Phase8NormalizeErrorReasonCode::DuplicateCheckerProfileResult,
                "machine_results[].checker.profile",
                "unique_checker_profile",
                &result.checker.profile,
            ));
        }
        if let Err((field, expected, actual)) =
            phase8_validate_machine_result_for_normalization(result)
        {
            return Err(phase8_normalize_value_error(
                normalize_error_result_id,
                policy_hash,
                Phase8NormalizeErrorReasonCode::MachineResultSchemaInvalid,
                field,
                expected,
                actual,
            ));
        }
    }

    let ordered_results = phase8_order_machine_results_for_normalization(policy, machine_results);
    let baseline_request_hash = if let Some(selector) = selector.as_ref() {
        selector.request_hash
    } else {
        let Some(required_baseline_profile) = policy.required_checker_profiles.first() else {
            return Err(phase8_normalize_value_error(
                normalize_error_result_id,
                policy_hash,
                Phase8NormalizeErrorReasonCode::SelectorAmbiguous,
                "artifact_selector",
                "baseline_required_checker_profile",
                "missing",
            ));
        };
        let Some(result) = machine_results
            .iter()
            .find(|result| &result.checker.profile == required_baseline_profile)
        else {
            return Err(phase8_normalize_value_error(
                normalize_error_result_id,
                policy_hash,
                Phase8NormalizeErrorReasonCode::SelectorAmbiguous,
                "artifact_selector",
                "single_baseline_result",
                "missing",
            ));
        };
        result.request_hash
    };

    let baseline_request = phase8_lookup_stored_request(
        request_store_manifest,
        stored_requests,
        baseline_request_hash,
        "artifact_selector.request_hash",
        normalize_error_result_id.clone(),
        policy_hash,
    )?;
    if let Some(selector) = selector.as_ref() {
        if selector.module != baseline_request.request.module {
            return Err(phase8_normalize_value_error(
                normalize_error_result_id,
                policy_hash,
                Phase8NormalizeErrorReasonCode::SelectorModuleMismatch,
                "artifact_selector.module",
                &baseline_request.request.module,
                &selector.module,
            ));
        }
    }

    let artifact = Phase8NormalizedArtifact::from_request(&baseline_request.request, policy);
    let mut entries = Vec::new();
    for result in ordered_results {
        let request = phase8_lookup_stored_request(
            request_store_manifest,
            stored_requests,
            result.request_hash,
            "machine_results[].request_hash",
            normalize_error_result_id.clone(),
            policy_hash,
        )?;
        let entry_artifact =
            Phase8NormalizedArtifact::from_request(&request.request, policy).artifact_hash();
        let entry = phase8_normalized_entry_from_machine_result(result, entry_artifact);
        phase8_assert_normalized_entry_source_copy(&entry, result);
        entries.push(entry);
    }

    let comparison = phase8_m4_normalized_comparison(policy, artifact.artifact_hash(), &entries);
    Ok(Phase8NormalizedCheckResult {
        normalized_result_id,
        artifact,
        policy: Phase8MachineCheckRequestPolicy {
            id: policy.id.clone(),
            version: policy.version,
            hash: policy_hash,
        },
        results: entries,
        comparison,
    })
}

pub fn parse_phase8_normalized_result_store_manifest(
    source: &str,
) -> Result<Phase8NormalizedResultStoreManifest, Phase8RequestValidationError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        Phase8RequestValidationError::value_failure(
            "normalized_store",
            "valid_json",
            "invalid_json",
        )
    })?;
    parse_phase8_normalized_result_store_manifest_value(document.root(), "normalized_store")
}

pub fn phase8_normalized_result_store_entry_for_result(
    result: &Phase8NormalizedCheckResult,
    path: impl Into<String>,
) -> Result<Phase8NormalizedResultStoreEntry, Phase8CommandError> {
    let path = path.into();
    if !phase8_valid_workspace_relative_path(&path) {
        return Err(phase8_command_value_error(
            Phase8CommandName::NormalizeResults,
            "input_reference_invalid",
            "normalized_result.path",
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    let bytes = result.canonical_json();
    Ok(Phase8NormalizedResultStoreEntry {
        normalized_result_hash: result.normalized_result_hash(),
        artifact_hash: result.artifact_hash(),
        path,
        file_hash: phase8_file_hash(bytes.as_bytes()),
    })
}

pub fn phase8_normalized_result_store_with_entry(
    existing_store: Option<&Phase8NormalizedResultStoreManifest>,
    generated_entry: Phase8NormalizedResultStoreEntry,
) -> Result<Phase8NormalizedResultStoreUpdate, Phase8CommandError> {
    let mut manifest = existing_store
        .cloned()
        .unwrap_or(Phase8NormalizedResultStoreManifest {
            results: Vec::new(),
        });

    for existing in &manifest.results {
        let same_normalized_result_hash =
            existing.normalized_result_hash == generated_entry.normalized_result_hash;
        let same_path = existing.path == generated_entry.path;
        let exact = same_normalized_result_hash
            && same_path
            && existing.artifact_hash == generated_entry.artifact_hash
            && existing.file_hash == generated_entry.file_hash;
        if exact {
            return Ok(Phase8NormalizedResultStoreUpdate {
                manifest,
                rewrite_required: false,
            });
        }
        if same_normalized_result_hash || same_path {
            return Err(phase8_command_value_error(
                Phase8CommandName::NormalizeResults,
                "normalized_store_entry_conflict",
                "normalized_store.results[]",
                generated_entry.canonical_json(),
                existing.canonical_json(),
            ));
        }
    }

    manifest.results.push(generated_entry);
    manifest.results.sort_by(|left, right| {
        left.normalized_result_hash
            .cmp(&right.normalized_result_hash)
    });
    Ok(Phase8NormalizedResultStoreUpdate {
        manifest,
        rewrite_required: true,
    })
}

pub fn phase8_normalization_write_result(
    normalized_result: &Phase8NormalizedCheckResult,
    output_path: impl Into<String>,
    normalized_store: Option<Phase8NormalizationWriteStore>,
) -> Result<Phase8NormalizationWriteResult, Phase8CommandError> {
    let output_path = output_path.into();
    if !phase8_valid_workspace_relative_path(&output_path) {
        return Err(phase8_command_value_error(
            Phase8CommandName::NormalizeResults,
            "input_reference_invalid",
            "output_path",
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    if let Some(store) = &normalized_store {
        if !phase8_valid_workspace_relative_path(&store.path) {
            return Err(phase8_command_value_error(
                Phase8CommandName::NormalizeResults,
                "input_reference_invalid",
                "normalized_store.path",
                "workspace_relative_path",
                "invalid_path",
            ));
        }
    }
    let bytes = normalized_result.canonical_json();
    Ok(Phase8NormalizationWriteResult {
        normalized_result_hash: normalized_result.normalized_result_hash(),
        artifact_hash: normalized_result.artifact_hash(),
        output_path,
        output_file_hash: phase8_file_hash(bytes.as_bytes()),
        normalized_store,
    })
}

impl Phase8RawResultSchemaError {
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

fn phase8_budget_mismatch_error(
    expected: &Phase8RunnerBudget,
    actual: &Phase8RunnerBudget,
) -> Phase8MachineCheckError {
    let (field, expected_value, actual_value) = if expected.max_steps != actual.max_steps {
        (
            "budget.max_steps",
            expected.max_steps.to_string(),
            actual.max_steps.to_string(),
        )
    } else if expected.max_memory_mb != actual.max_memory_mb {
        (
            "budget.max_memory_mb",
            expected.max_memory_mb.to_string(),
            actual.max_memory_mb.to_string(),
        )
    } else {
        (
            "budget.timeout_ms",
            expected.timeout_ms.to_string(),
            actual.timeout_ms.to_string(),
        )
    };
    Phase8MachineCheckError::new("policy_failure")
        .with_reason_code("request_budget_mismatch")
        .with_value_payload(field, expected_value, actual_value)
}

fn phase8_normalize_value_error(
    result_id: impl Into<String>,
    policy_hash: Hash,
    reason_code: Phase8NormalizeErrorReasonCode,
    field: impl Into<String>,
    expected_value: impl Into<String>,
    actual_value: impl Into<String>,
) -> Phase8NormalizeErrorResult {
    Phase8NormalizeErrorResult::normalize_failure(result_id, reason_code, field)
        .with_policy_hash(policy_hash)
        .with_error_expected_value(expected_value)
        .with_error_actual_value(actual_value)
}

fn phase8_normalize_hash_error(
    result_id: impl Into<String>,
    policy_hash: Hash,
    reason_code: Phase8NormalizeErrorReasonCode,
    field: impl Into<String>,
    expected_hash: Hash,
    actual_hash: Hash,
) -> Phase8NormalizeErrorResult {
    Phase8NormalizeErrorResult::normalize_failure(result_id, reason_code, field)
        .with_policy_hash(policy_hash)
        .with_error_expected_hash(expected_hash)
        .with_error_actual_hash(actual_hash)
}

fn phase8_normalize_actual_hash_error(
    result_id: impl Into<String>,
    policy_hash: Hash,
    reason_code: Phase8NormalizeErrorReasonCode,
    field: impl Into<String>,
    actual_hash: Hash,
) -> Phase8NormalizeErrorResult {
    Phase8NormalizeErrorResult::normalize_failure(result_id, reason_code, field)
        .with_policy_hash(policy_hash)
        .with_error_actual_hash(actual_hash)
}

fn phase8_normalize_request_validation_error(
    result_id: impl Into<String>,
    policy_hash: Hash,
    reason_code: Phase8NormalizeErrorReasonCode,
    error: Phase8RequestValidationError,
) -> Phase8NormalizeErrorResult {
    let mut out =
        Phase8NormalizeErrorResult::normalize_failure(result_id, reason_code, error.field)
            .with_policy_hash(policy_hash);
    if let Some(expected_value) = error.expected_value {
        out = out.with_error_expected_value(expected_value);
    }
    if let Some(actual_value) = error.actual_value {
        out = out.with_error_actual_value(actual_value);
    }
    if let Some(expected_hash) = error.expected_hash {
        out = out.with_error_expected_hash(*expected_hash);
    }
    if let Some(actual_hash) = error.actual_hash {
        out = out.with_error_actual_hash(*actual_hash);
    }
    out
}

fn phase8_validate_machine_result_for_normalization(
    result: &Phase8MachineCheckResult,
) -> Result<(), (&'static str, &'static str, &'static str)> {
    if !phase8_valid_request_id(&result.result_id) {
        return Err((
            "machine_results[].result_id",
            "result_id",
            if result.result_id.is_empty() {
                "empty_string"
            } else {
                "invalid_string_format"
            },
        ));
    }
    if !phase8_valid_checker_profile_name(&result.checker.profile) {
        return Err((
            "machine_results[].checker.profile",
            "checker_profile_name",
            "invalid_name_format",
        ));
    }
    if result.process.launched {
        if result.checker.binary_id.is_none() {
            return Err((
                "machine_results[].checker.binary_id",
                "checker_binary_id",
                "missing",
            ));
        }
        if result.checker.binary_hash.is_none() {
            return Err((
                "machine_results[].checker.binary_hash",
                "sha256:<lower-hex>",
                "missing",
            ));
        }
    }
    match result.status {
        Phase8MachineCheckStatus::Checked => {
            if result.certificate_hash.is_none() {
                return Err((
                    "machine_results[].certificate_hash",
                    "sha256:<lower-hex>",
                    "missing",
                ));
            }
            if result.export_hash.is_none() {
                return Err((
                    "machine_results[].export_hash",
                    "sha256:<lower-hex>",
                    "missing",
                ));
            }
            if result.axiom_report_hash.is_none() {
                return Err((
                    "machine_results[].axiom_report_hash",
                    "sha256:<lower-hex>",
                    "missing",
                ));
            }
            if result.error.is_some() {
                return Err((
                    "machine_results[].error",
                    "absent_for_status",
                    "forbidden_field",
                ));
            }
        }
        Phase8MachineCheckStatus::Failed => {
            if result.error.is_none() {
                return Err((
                    "machine_results[].error",
                    "MachineCheckResult.error",
                    "missing",
                ));
            }
        }
    }
    Ok(())
}

fn phase8_order_machine_results_for_normalization<'a>(
    policy: &Phase8RunnerPolicy,
    machine_results: &'a [Phase8MachineCheckResult],
) -> Vec<&'a Phase8MachineCheckResult> {
    let mut out = Vec::new();
    for profile in &policy.required_checker_profiles {
        if let Some(result) = machine_results
            .iter()
            .find(|result| &result.checker.profile == profile)
        {
            out.push(result);
        }
    }
    for profile in &policy.optional_checker_profiles {
        if let Some(result) = machine_results
            .iter()
            .find(|result| &result.checker.profile == profile)
        {
            out.push(result);
        }
    }
    let known = policy
        .required_checker_profiles
        .iter()
        .chain(policy.optional_checker_profiles.iter())
        .collect::<BTreeSet<_>>();
    let mut outside_policy = machine_results
        .iter()
        .filter(|result| !known.contains(&result.checker.profile))
        .collect::<Vec<_>>();
    outside_policy.sort_by(|left, right| left.checker.profile.cmp(&right.checker.profile));
    out.extend(outside_policy);
    out
}

fn phase8_lookup_stored_request<'a>(
    manifest: &Phase8RequestStoreManifest,
    stored_requests: &'a [Phase8StoredMachineCheckRequest],
    request_hash: Hash,
    field: &'static str,
    result_id: String,
    policy_hash: Hash,
) -> Result<&'a Phase8StoredMachineCheckRequest, Phase8NormalizeErrorResult> {
    let Some(entry) = manifest
        .requests
        .iter()
        .find(|entry| entry.request_hash == request_hash)
    else {
        return Err(phase8_normalize_actual_hash_error(
            result_id,
            policy_hash,
            Phase8NormalizeErrorReasonCode::RequestHashNotFound,
            field,
            request_hash,
        ));
    };
    let Some(stored) = stored_requests
        .iter()
        .find(|request| request.request.request_hash() == request_hash)
    else {
        return Err(phase8_normalize_value_error(
            result_id,
            policy_hash,
            Phase8NormalizeErrorReasonCode::RequestFileUnreadable,
            "request_store.requests[].path",
            "readable",
            "unreadable",
        ));
    };
    if stored.path != entry.path {
        return Err(phase8_normalize_value_error(
            result_id,
            policy_hash,
            Phase8NormalizeErrorReasonCode::RequestStoreManifestInvalid,
            "request_store.requests[].path",
            &entry.path,
            &stored.path,
        ));
    }
    if stored.file_hash != entry.file_hash {
        return Err(phase8_normalize_hash_error(
            result_id,
            policy_hash,
            Phase8NormalizeErrorReasonCode::RequestFileHashMismatch,
            "request_store.requests[].file_hash",
            entry.file_hash,
            stored.file_hash,
        ));
    }
    Ok(stored)
}

fn phase8_normalized_entry_from_machine_result(
    result: &Phase8MachineCheckResult,
    artifact_hash: Hash,
) -> Phase8NormalizedCheckResultEntry {
    let failure_key = result
        .error
        .as_ref()
        .map(Phase8NormalizedFailureKey::from_error);
    Phase8NormalizedCheckResultEntry {
        result_id: result.result_id.clone(),
        result_hash: result.result_hash(),
        request_hash: result.request_hash,
        policy_hash: result.policy.hash,
        artifact_hash,
        checker_profile: result.checker.profile.clone(),
        process_launched: result.process.launched,
        status: result.status,
        checker_binary_id: result
            .process
            .launched
            .then(|| result.checker.binary_id.clone())
            .flatten(),
        checker_binary_hash: result
            .process
            .launched
            .then_some(result.checker.binary_hash)
            .flatten(),
        checker_id: result
            .process
            .launched
            .then(|| result.checker.id.clone())
            .flatten(),
        checker_build_hash: result
            .process
            .launched
            .then_some(result.checker.build_hash)
            .flatten(),
        certificate_hash: result.certificate_hash,
        export_hash: result.export_hash,
        axiom_report_hash: result.axiom_report_hash,
        error: result.error.clone(),
        failure_key,
    }
}

fn phase8_assert_normalized_entry_source_copy(
    entry: &Phase8NormalizedCheckResultEntry,
    result: &Phase8MachineCheckResult,
) {
    debug_assert_eq!(entry.result_id, result.result_id);
    debug_assert_eq!(entry.result_hash, result.result_hash());
    debug_assert_eq!(entry.request_hash, result.request_hash);
    debug_assert_eq!(entry.policy_hash, result.policy.hash);
    debug_assert_eq!(entry.checker_profile, result.checker.profile);
    debug_assert_eq!(entry.process_launched, result.process.launched);
    debug_assert_eq!(entry.status, result.status);
    if result.process.launched {
        debug_assert_eq!(entry.checker_binary_id, result.checker.binary_id);
        debug_assert_eq!(entry.checker_binary_hash, result.checker.binary_hash);
        debug_assert_eq!(entry.checker_id, result.checker.id);
        debug_assert_eq!(entry.checker_build_hash, result.checker.build_hash);
    } else {
        debug_assert_eq!(entry.checker_binary_id, None);
        debug_assert_eq!(entry.checker_binary_hash, None);
        debug_assert_eq!(entry.checker_id, None);
        debug_assert_eq!(entry.checker_build_hash, None);
    }
    debug_assert_eq!(entry.certificate_hash, result.certificate_hash);
    debug_assert_eq!(entry.export_hash, result.export_hash);
    debug_assert_eq!(entry.axiom_report_hash, result.axiom_report_hash);
    debug_assert_eq!(entry.error, result.error);
}

fn phase8_m4_normalized_comparison(
    policy: &Phase8RunnerPolicy,
    artifact_hash: Hash,
    entries: &[Phase8NormalizedCheckResultEntry],
) -> Phase8NormalizedComparison {
    let present_profiles = entries
        .iter()
        .map(|entry| entry.checker_profile.as_str())
        .collect::<BTreeSet<_>>();
    let missing_checker_profiles = policy
        .required_checker_profiles
        .iter()
        .filter(|profile| !present_profiles.contains(profile.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if !missing_checker_profiles.is_empty() {
        return Phase8NormalizedComparison {
            status: Phase8NormalizedComparisonStatus::MissingCheckerResult,
            matching_fields: Vec::new(),
            missing_checker_profiles,
            disagreements: Vec::new(),
            status_reasons: Vec::new(),
        };
    }

    let mut disagreements = Vec::new();
    for entry in entries {
        if entry.artifact_hash != artifact_hash {
            disagreements.push(Phase8NormalizedDisagreement {
                field: "artifact_hash".to_owned(),
                baseline_checker_profile: None,
                baseline_hash: Some(artifact_hash),
                baseline_value: None,
                checker_profile: entry.checker_profile.clone(),
                actual_hash: Some(entry.artifact_hash),
                actual_value: None,
            });
        }
    }
    if !disagreements.is_empty() {
        disagreements.sort_by(|left, right| {
            left.field
                .cmp(&right.field)
                .then_with(|| left.checker_profile.cmp(&right.checker_profile))
        });
        return Phase8NormalizedComparison {
            status: Phase8NormalizedComparisonStatus::Disagreement,
            matching_fields: Vec::new(),
            missing_checker_profiles: Vec::new(),
            disagreements,
            status_reasons: Vec::new(),
        };
    }

    let Some(baseline) = entries.first() else {
        return Phase8NormalizedComparison {
            status: Phase8NormalizedComparisonStatus::Inconclusive,
            matching_fields: Vec::new(),
            missing_checker_profiles: Vec::new(),
            disagreements: Vec::new(),
            status_reasons: Vec::new(),
        };
    };

    if entries
        .iter()
        .all(|entry| entry.status == Phase8MachineCheckStatus::Checked)
    {
        for entry in entries.iter().skip(1) {
            phase8_push_hash_disagreement(
                &mut disagreements,
                "certificate_hash",
                baseline,
                entry,
                baseline.certificate_hash,
                entry.certificate_hash,
            );
            phase8_push_hash_disagreement(
                &mut disagreements,
                "export_hash",
                baseline,
                entry,
                baseline.export_hash,
                entry.export_hash,
            );
            phase8_push_hash_disagreement(
                &mut disagreements,
                "axiom_report_hash",
                baseline,
                entry,
                baseline.axiom_report_hash,
                entry.axiom_report_hash,
            );
        }
        if disagreements.is_empty() {
            return Phase8NormalizedComparison {
                status: Phase8NormalizedComparisonStatus::AllAgreeChecked,
                matching_fields: vec![
                    "certificate_hash".to_owned(),
                    "export_hash".to_owned(),
                    "axiom_report_hash".to_owned(),
                ],
                missing_checker_profiles: Vec::new(),
                disagreements,
                status_reasons: Vec::new(),
            };
        }
    } else if entries
        .iter()
        .all(|entry| entry.status == Phase8MachineCheckStatus::Failed)
    {
        let baseline_hash = baseline
            .failure_key
            .as_ref()
            .map(Phase8NormalizedFailureKey::failure_key_hash);
        for entry in entries.iter().skip(1) {
            let actual_hash = entry
                .failure_key
                .as_ref()
                .map(Phase8NormalizedFailureKey::failure_key_hash);
            phase8_push_hash_disagreement(
                &mut disagreements,
                "failure_key",
                baseline,
                entry,
                baseline_hash,
                actual_hash,
            );
        }
        if disagreements.is_empty() {
            return Phase8NormalizedComparison {
                status: Phase8NormalizedComparisonStatus::AllAgreeFailed,
                matching_fields: vec!["failure_key".to_owned()],
                missing_checker_profiles: Vec::new(),
                disagreements,
                status_reasons: Vec::new(),
            };
        }
    } else {
        for entry in entries.iter().skip(1) {
            if entry.status != baseline.status {
                disagreements.push(Phase8NormalizedDisagreement {
                    field: "status".to_owned(),
                    baseline_checker_profile: Some(baseline.checker_profile.clone()),
                    baseline_hash: None,
                    baseline_value: Some(baseline.status.as_str().to_owned()),
                    checker_profile: entry.checker_profile.clone(),
                    actual_hash: None,
                    actual_value: Some(entry.status.as_str().to_owned()),
                });
            }
        }
    }

    disagreements.sort_by(|left, right| {
        left.field
            .cmp(&right.field)
            .then_with(|| left.checker_profile.cmp(&right.checker_profile))
    });
    Phase8NormalizedComparison {
        status: Phase8NormalizedComparisonStatus::Disagreement,
        matching_fields: Vec::new(),
        missing_checker_profiles: Vec::new(),
        disagreements,
        status_reasons: Vec::new(),
    }
}

fn phase8_push_hash_disagreement(
    disagreements: &mut Vec<Phase8NormalizedDisagreement>,
    field: &str,
    baseline: &Phase8NormalizedCheckResultEntry,
    actual: &Phase8NormalizedCheckResultEntry,
    baseline_hash: Option<Hash>,
    actual_hash: Option<Hash>,
) {
    if baseline_hash != actual_hash {
        disagreements.push(Phase8NormalizedDisagreement {
            field: field.to_owned(),
            baseline_checker_profile: Some(baseline.checker_profile.clone()),
            baseline_hash,
            baseline_value: None,
            checker_profile: actual.checker_profile.clone(),
            actual_hash,
            actual_value: None,
        });
    }
}

fn phase8_adopt_checker_observation(
    request: &Phase8MachineCheckRequest,
    policy: &Phase8RunnerPolicy,
    selected_checker: &Phase8CheckerAllowlistEntry,
    observation: Phase8CheckerRunObservation,
) -> Phase8MachineCheckResult {
    if let Some(reason) = observation.process.termination_reason.as_deref() {
        return match reason {
            "timeout" => phase8_infrastructure_failure_result(
                request,
                policy,
                selected_checker,
                &observation,
                "timeout",
                "checker_timeout",
            ),
            "resource_exhausted" => phase8_infrastructure_failure_result(
                request,
                policy,
                selected_checker,
                &observation,
                "resource_exhausted",
                "checker_resource_exhausted",
            ),
            "killed_without_exit_status" => {
                let error = Phase8MachineCheckError::new("checker_internal_error")
                    .with_reason_code("process_exit_failure")
                    .with_value_payload(
                        "process.termination_reason",
                        "process_exit_status",
                        "killed_without_exit_status",
                    );
                phase8_machine_check_result_base(
                    request,
                    policy,
                    Some(selected_checker),
                    &observation,
                    Phase8MachineCheckStatus::Failed,
                    Some(error),
                )
            }
            _ => phase8_infrastructure_failure_result(
                request,
                policy,
                selected_checker,
                &observation,
                "checker_internal_error",
                "process_exit_failure",
            ),
        };
    }

    let Some(exit_code) = observation.process.exit_code else {
        let error = Phase8MachineCheckError::new("checker_internal_error")
            .with_reason_code("process_exit_failure");
        return phase8_machine_check_result_base(
            request,
            policy,
            Some(selected_checker),
            &observation,
            Phase8MachineCheckStatus::Failed,
            Some(error),
        );
    };

    if exit_code >= 3 {
        let error = Phase8MachineCheckError::new("checker_internal_error")
            .with_reason_code("process_exit_failure");
        let mut result = phase8_machine_check_result_base(
            request,
            policy,
            Some(selected_checker),
            &observation,
            Phase8MachineCheckStatus::Failed,
            Some(error),
        );
        result.diagnostics.clear();
        return result;
    }

    let stdout = std::str::from_utf8(&observation.stdout);
    let raw = stdout
        .ok()
        .and_then(|source| parse_phase8_checker_raw_result(source).ok());
    let raw_error = match stdout {
        Ok(source) => parse_phase8_checker_raw_result(source).err(),
        Err(_) => Some(Phase8RawResultSchemaError::new(
            "checker_raw",
            "valid_json",
            "invalid_json",
        )),
    };
    let Some(raw) = raw else {
        let reason_code = match exit_code {
            0 => "malformed_success_output",
            1 => "malformed_rejection_output",
            2 => "malformed_internal_error_output",
            _ => unreachable!("exit_code >= 3 handled above"),
        };
        let mut error =
            Phase8MachineCheckError::new("checker_internal_error").with_reason_code(reason_code);
        if let Some(raw_error) = raw_error {
            error = error.with_value_payload(
                raw_error.field,
                raw_error.expected_value,
                raw_error.actual_value,
            );
        }
        let mut result = phase8_machine_check_result_base(
            request,
            policy,
            Some(selected_checker),
            &observation,
            Phase8MachineCheckStatus::Failed,
            Some(error),
        );
        if !observation.stdout.is_empty() {
            result
                .diagnostics
                .push("checker_process:stdout_present".to_owned());
        }
        phase8_push_stderr_diagnostic(&mut result, &observation);
        return result;
    };

    let mut checker = phase8_machine_check_checker_for_request(request, Some(selected_checker));
    checker.id = raw.checker_id.clone();
    checker.build_hash = raw.checker_build_hash;
    checker.version = raw.checker_version.clone();

    if exit_code == 0 && raw.status != Phase8MachineCheckStatus::Checked {
        let error = Phase8MachineCheckError::new("checker_internal_error")
            .with_reason_code("success_exit_status_mismatch");
        let mut result = phase8_machine_check_result_base(
            request,
            policy,
            Some(selected_checker),
            &observation,
            Phase8MachineCheckStatus::Failed,
            Some(error),
        );
        result.checker = checker;
        phase8_push_stderr_diagnostic(&mut result, &observation);
        return result;
    }
    if exit_code == 1 && (raw.status != Phase8MachineCheckStatus::Failed || raw.error.is_none()) {
        let error = Phase8MachineCheckError::new("checker_internal_error")
            .with_reason_code("missing_rejection_error");
        let mut result = phase8_machine_check_result_base(
            request,
            policy,
            Some(selected_checker),
            &observation,
            Phase8MachineCheckStatus::Failed,
            Some(error),
        );
        result.checker = checker;
        phase8_push_stderr_diagnostic(&mut result, &observation);
        return result;
    }
    if exit_code == 1
        && raw
            .error
            .as_ref()
            .map(|error| error.kind.as_str() == "checker_internal_error")
            .unwrap_or(false)
    {
        let error = Phase8MachineCheckError::new("checker_internal_error")
            .with_reason_code("malformed_rejection_output");
        let mut result = phase8_machine_check_result_base(
            request,
            policy,
            Some(selected_checker),
            &observation,
            Phase8MachineCheckStatus::Failed,
            Some(error),
        );
        result.checker = checker;
        phase8_push_stderr_diagnostic(&mut result, &observation);
        return result;
    }
    if exit_code == 2
        && raw
            .error
            .as_ref()
            .map(|error| error.kind.as_str() != "checker_internal_error")
            .unwrap_or(true)
    {
        let error = Phase8MachineCheckError::new("checker_internal_error")
            .with_reason_code("malformed_internal_error_output");
        let mut result = phase8_machine_check_result_base(
            request,
            policy,
            Some(selected_checker),
            &observation,
            Phase8MachineCheckStatus::Failed,
            Some(error),
        );
        result.checker = checker;
        phase8_push_stderr_diagnostic(&mut result, &observation);
        return result;
    }

    if raw.checker_id.as_deref() != Some(selected_checker.checker_id.as_str()) {
        let actual = raw.checker_id.as_deref().unwrap_or("missing");
        let error = Phase8MachineCheckError::new("policy_failure")
            .with_reason_code(if raw.checker_id.is_some() {
                "checker_identity_mismatch"
            } else {
                "checker_identity_missing"
            })
            .with_value_payload("checker.id", &selected_checker.checker_id, actual);
        let mut result = phase8_machine_check_result_base(
            request,
            policy,
            Some(selected_checker),
            &observation,
            Phase8MachineCheckStatus::Failed,
            Some(error),
        );
        result.checker = checker;
        phase8_push_stderr_diagnostic(&mut result, &observation);
        return result;
    }
    if raw.checker_build_hash != Some(selected_checker.build_hash) {
        let mut error = Phase8MachineCheckError::new("policy_failure").with_reason_code(
            if raw.checker_build_hash.is_some() {
                "checker_build_hash_mismatch"
            } else {
                "checker_identity_missing"
            },
        );
        error.field = Some("checker.build_hash".to_owned());
        error.expected_hash = Some(selected_checker.build_hash);
        error.actual_hash = raw.checker_build_hash;
        let mut result = phase8_machine_check_result_base(
            request,
            policy,
            Some(selected_checker),
            &observation,
            Phase8MachineCheckStatus::Failed,
            Some(error),
        );
        result.checker = checker;
        phase8_push_stderr_diagnostic(&mut result, &observation);
        return result;
    }

    if let Some(raw_module) = raw.module.as_deref() {
        if raw_module != request.module {
            let error = Phase8MachineCheckError::new("checker_internal_error")
                .with_reason_code("checker_module_mismatch")
                .with_value_payload("module", &request.module, raw_module);
            let mut result = phase8_machine_check_result_base(
                request,
                policy,
                Some(selected_checker),
                &observation,
                Phase8MachineCheckStatus::Failed,
                Some(error),
            );
            result.checker = checker;
            phase8_push_stderr_diagnostic(&mut result, &observation);
            return result;
        }
    }

    if raw.status == Phase8MachineCheckStatus::Checked
        || raw
            .error
            .as_ref()
            .map(phase8_error_requires_certificate_hash)
            .unwrap_or(false)
    {
        if let Some(certificate_hash) = raw.certificate_hash {
            if certificate_hash != request.certificate.expected_certificate_hash {
                let error = Phase8MachineCheckError::new("certificate_hash_mismatch")
                    .with_hash_payload(
                        "certificate_hash",
                        request.certificate.expected_certificate_hash,
                        certificate_hash,
                    );
                let mut result = phase8_machine_check_result_base(
                    request,
                    policy,
                    Some(selected_checker),
                    &observation,
                    Phase8MachineCheckStatus::Failed,
                    Some(error),
                );
                result.checker = checker;
                result.certificate_hash = Some(certificate_hash);
                phase8_push_stderr_diagnostic(&mut result, &observation);
                return result;
            }
        }
    }

    match exit_code {
        0 => {
            let mut result = phase8_machine_check_result_base(
                request,
                policy,
                Some(selected_checker),
                &observation,
                Phase8MachineCheckStatus::Checked,
                None,
            );
            result.checker = checker;
            result.certificate_hash = raw.certificate_hash;
            result.export_hash = raw.export_hash;
            result.axiom_report_hash = raw.axiom_report_hash;
            phase8_push_stderr_diagnostic(&mut result, &observation);
            result
        }
        1 => {
            let mut result = phase8_machine_check_result_base(
                request,
                policy,
                Some(selected_checker),
                &observation,
                Phase8MachineCheckStatus::Failed,
                raw.error,
            );
            result.checker = checker;
            result.certificate_hash = raw.certificate_hash;
            result.export_hash = raw.export_hash;
            result.axiom_report_hash = raw.axiom_report_hash;
            phase8_push_stderr_diagnostic(&mut result, &observation);
            result
        }
        2 => {
            let error = raw.error.unwrap_or_else(|| {
                Phase8MachineCheckError::new("checker_internal_error")
                    .with_reason_code("checker_reported_internal_error")
            });
            let mut result = phase8_machine_check_result_base(
                request,
                policy,
                Some(selected_checker),
                &observation,
                Phase8MachineCheckStatus::Failed,
                Some(error),
            );
            result.checker = checker;
            result.certificate_hash = raw.certificate_hash;
            result.export_hash = raw.export_hash;
            result.axiom_report_hash = raw.axiom_report_hash;
            phase8_push_stderr_diagnostic(&mut result, &observation);
            result
        }
        _ => unreachable!("exit_code >= 3 handled above"),
    }
}

fn phase8_policy_failure_result(
    request: &Phase8MachineCheckRequest,
    policy: &Phase8RunnerPolicy,
    observation: &Phase8CheckerRunObservation,
    _reason_code: &str,
    error: Phase8MachineCheckError,
) -> Phase8MachineCheckResult {
    let mut result = phase8_machine_check_result_base(
        request,
        policy,
        None,
        observation,
        Phase8MachineCheckStatus::Failed,
        Some(error),
    );
    result.process = Phase8MachineCheckProcess::not_launched();
    result.resource_usage = Phase8MachineCheckResourceUsage::zero();
    result.diagnostics.clear();
    result
}

fn phase8_infrastructure_failure_result(
    request: &Phase8MachineCheckRequest,
    policy: &Phase8RunnerPolicy,
    selected_checker: &Phase8CheckerAllowlistEntry,
    observation: &Phase8CheckerRunObservation,
    kind: &str,
    reason_code: &str,
) -> Phase8MachineCheckResult {
    let error = Phase8MachineCheckError::new(kind).with_reason_code(reason_code);
    phase8_machine_check_result_base(
        request,
        policy,
        Some(selected_checker),
        observation,
        Phase8MachineCheckStatus::Failed,
        Some(error),
    )
}

fn phase8_machine_check_result_base(
    request: &Phase8MachineCheckRequest,
    policy: &Phase8RunnerPolicy,
    selected_checker: Option<&Phase8CheckerAllowlistEntry>,
    observation: &Phase8CheckerRunObservation,
    status: Phase8MachineCheckStatus,
    error: Option<Phase8MachineCheckError>,
) -> Phase8MachineCheckResult {
    let mut diagnostics = Vec::new();
    if observation.process.exit_code.is_some()
        && observation.process.exit_code.unwrap_or(255) <= 2
        && !observation.stderr.is_empty()
    {
        diagnostics.push("checker_process:stderr_present".to_owned());
    }
    Phase8MachineCheckResult {
        request_id: request.request_id.clone(),
        request_hash: request.request_hash(),
        result_id: observation.result_id.clone(),
        policy: Phase8MachineCheckRequestPolicy {
            id: policy.id.clone(),
            version: policy.version,
            hash: policy.policy_hash(),
        },
        runner: observation.runner.clone(),
        checker: phase8_machine_check_checker_for_request(request, selected_checker),
        attempt: observation.attempt,
        status,
        module: request.module.clone(),
        process: observation.process.clone(),
        resource_usage: observation.resource_usage.clone(),
        error,
        certificate_hash: None,
        export_hash: None,
        axiom_report_hash: None,
        diagnostics,
        axioms_used: None,
        declarations_checked: None,
    }
}

fn phase8_machine_check_checker_for_request(
    request: &Phase8MachineCheckRequest,
    selected_checker: Option<&Phase8CheckerAllowlistEntry>,
) -> Phase8MachineCheckChecker {
    Phase8MachineCheckChecker {
        profile: request.checker_profile.clone(),
        binary_id: selected_checker.map(|checker| checker.binary_id.clone()),
        binary_hash: selected_checker.map(|checker| checker.binary_hash),
        id: None,
        build_hash: None,
        version: None,
    }
}

fn phase8_push_stderr_diagnostic(
    result: &mut Phase8MachineCheckResult,
    observation: &Phase8CheckerRunObservation,
) {
    if !observation.stderr.is_empty()
        && observation.process.exit_code.is_some()
        && observation.process.exit_code.unwrap_or(255) <= 2
        && !result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic == "checker_process:stderr_present")
    {
        result
            .diagnostics
            .push("checker_process:stderr_present".to_owned());
    }
}

fn phase8_error_requires_certificate_hash(error: &Phase8MachineCheckError) -> bool {
    matches!(
        error.kind.as_str(),
        "import_not_found"
            | "import_hash_mismatch"
            | "certificate_hash_mismatch"
            | "axiom_report_mismatch"
            | "export_hash_mismatch"
            | "type_mismatch"
            | "conversion_failure"
            | "universe_inconsistency"
            | "inductive_invalid"
            | "positivity_failure"
            | "declaration_hash_mismatch"
            | "dependency_hash_mismatch"
            | "forbidden_axiom"
    )
}

fn parse_phase8_checker_raw_result_value(
    value: &JsonValue<'_>,
) -> Result<Phase8CheckerRawResult, Phase8RawResultSchemaError> {
    let members = value
        .object_members()
        .ok_or_else(|| Phase8RawResultSchemaError::new("checker_raw", "object", "wrong_type"))?;
    raw_required_fixed_string(
        members,
        "schema",
        "checker_raw.schema",
        PHASE8_CHECKER_RAW_RESULT_SCHEMA,
    )?;
    let status_raw = raw_required_string(
        members,
        "status",
        "checker_raw.status",
        "CheckerRawResult.status",
    )?;
    let status = match status_raw.as_str() {
        "checked" => Phase8MachineCheckStatus::Checked,
        "failed" => Phase8MachineCheckStatus::Failed,
        _ => {
            return Err(Phase8RawResultSchemaError::new(
                "checker_raw.status",
                "CheckerRawResult.status",
                "invalid_enum",
            ))
        }
    };
    let checker_id = raw_optional_identity_string(
        members,
        "checker_id",
        "checker_raw.checker_id",
        "checker_id",
    )?;
    if let Some(checker_id) = checker_id.as_deref() {
        if !phase8_valid_checker_id(checker_id) {
            return Err(Phase8RawResultSchemaError::new(
                "checker_raw.checker_id",
                "checker_id",
                "invalid_name_format",
            ));
        }
    }
    let checker_version = raw_optional_identity_string(
        members,
        "checker_version",
        "checker_raw.checker_version",
        "string",
    )?;
    let checker_build_hash = raw_optional_identity_hash(
        members,
        "checker_build_hash",
        "checker_raw.checker_build_hash",
    )?;

    let module = raw_optional_module(members, "module", "checker_raw.module")?;
    let certificate_hash =
        raw_optional_hash(members, "certificate_hash", "checker_raw.certificate_hash")?;
    let export_hash = raw_optional_hash(members, "export_hash", "checker_raw.export_hash")?;
    let axiom_report_hash = raw_optional_hash(
        members,
        "axiom_report_hash",
        "checker_raw.axiom_report_hash",
    )?;
    let error = raw_optional_error(members)?;

    match status {
        Phase8MachineCheckStatus::Checked => {
            if module.is_none() {
                return Err(Phase8RawResultSchemaError::new(
                    "checker_raw.module",
                    "module_name",
                    "missing",
                ));
            }
            if certificate_hash.is_none() {
                return Err(Phase8RawResultSchemaError::new(
                    "checker_raw.certificate_hash",
                    "sha256:<lower-hex>",
                    "missing",
                ));
            }
            if export_hash.is_none() {
                return Err(Phase8RawResultSchemaError::new(
                    "checker_raw.export_hash",
                    "sha256:<lower-hex>",
                    "missing",
                ));
            }
            if axiom_report_hash.is_none() {
                return Err(Phase8RawResultSchemaError::new(
                    "checker_raw.axiom_report_hash",
                    "sha256:<lower-hex>",
                    "missing",
                ));
            }
            if duplicate_member(members, "error")
                || members.iter().any(|member| member.key() == "error")
            {
                return Err(Phase8RawResultSchemaError::new(
                    "checker_raw.error",
                    "absent_for_status_kind",
                    "forbidden_field",
                ));
            }
        }
        Phase8MachineCheckStatus::Failed => {
            let Some(error) = &error else {
                return Err(Phase8RawResultSchemaError::new(
                    "checker_raw.error",
                    "object",
                    "missing",
                ));
            };
            if phase8_error_requires_certificate_hash(error) {
                if module.is_none() {
                    return Err(Phase8RawResultSchemaError::new(
                        "checker_raw.module",
                        "module_name",
                        "missing",
                    ));
                }
                if certificate_hash.is_none() {
                    return Err(Phase8RawResultSchemaError::new(
                        "checker_raw.certificate_hash",
                        "sha256:<lower-hex>",
                        "missing",
                    ));
                }
            }
        }
    }

    raw_reject_unknown_fields(
        members,
        &[
            "schema",
            "checker_id",
            "checker_version",
            "checker_build_hash",
            "status",
            "module",
            "certificate_hash",
            "export_hash",
            "axiom_report_hash",
            "error",
        ],
        "checker_raw",
    )?;

    Ok(Phase8CheckerRawResult {
        status,
        checker_id,
        checker_version,
        checker_build_hash,
        module,
        certificate_hash,
        export_hash,
        axiom_report_hash,
        error,
    })
}

fn raw_required_string(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
    expected: &str,
) -> Result<String, Phase8RawResultSchemaError> {
    if duplicate_member(members, name) {
        return Err(Phase8RawResultSchemaError::new(
            field,
            "unique_object_keys",
            "duplicate_field",
        ));
    }
    let Some(value) = members
        .iter()
        .find(|member| member.key() == name)
        .map(JsonMember::value)
    else {
        return Err(Phase8RawResultSchemaError::new(field, expected, "missing"));
    };
    if value.kind() == JsonValueKind::Null {
        return Err(Phase8RawResultSchemaError::new(
            field,
            expected,
            "null_not_allowed",
        ));
    }
    value
        .string_value()
        .map(ToOwned::to_owned)
        .ok_or_else(|| Phase8RawResultSchemaError::new(field, expected, "wrong_type"))
}

fn raw_required_fixed_string(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
    fixed: &str,
) -> Result<(), Phase8RawResultSchemaError> {
    let value = raw_required_string(members, name, field, fixed)?;
    if value != fixed {
        return Err(Phase8RawResultSchemaError::new(
            field,
            fixed,
            "invalid_enum",
        ));
    }
    Ok(())
}

fn raw_optional_identity_string(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
    expected: &str,
) -> Result<Option<String>, Phase8RawResultSchemaError> {
    if duplicate_member(members, name) {
        return Err(Phase8RawResultSchemaError::new(
            field,
            "unique_object_keys",
            "duplicate_field",
        ));
    }
    let Some(value) = members
        .iter()
        .find(|member| member.key() == name)
        .map(JsonMember::value)
    else {
        return Ok(None);
    };
    if value.kind() == JsonValueKind::Null {
        return Err(Phase8RawResultSchemaError::new(
            field,
            expected,
            "null_not_allowed",
        ));
    }
    value
        .string_value()
        .map(|value| Some(value.to_owned()))
        .ok_or_else(|| Phase8RawResultSchemaError::new(field, expected, "wrong_type"))
}

fn raw_optional_identity_hash(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
) -> Result<Option<Hash>, Phase8RawResultSchemaError> {
    if duplicate_member(members, name) {
        return Err(Phase8RawResultSchemaError::new(
            field,
            "unique_object_keys",
            "duplicate_field",
        ));
    }
    let Some(value) = members
        .iter()
        .find(|member| member.key() == name)
        .map(JsonMember::value)
    else {
        return Ok(None);
    };
    if value.kind() == JsonValueKind::Null {
        return Err(Phase8RawResultSchemaError::new(
            field,
            "sha256:<lower-hex>",
            "null_not_allowed",
        ));
    }
    let Some(raw) = value.string_value() else {
        return Err(Phase8RawResultSchemaError::new(
            field,
            "sha256:<lower-hex>",
            "wrong_type",
        ));
    };
    parse_hash_string(raw).map(Some).map_err(|_| {
        Phase8RawResultSchemaError::new(field, "sha256:<lower-hex>", "invalid_hash_format")
    })
}

fn raw_optional_module(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
) -> Result<Option<String>, Phase8RawResultSchemaError> {
    if duplicate_member(members, name) {
        return Err(Phase8RawResultSchemaError::new(
            field,
            "unique_object_keys",
            "duplicate_field",
        ));
    }
    let Some(value) = members
        .iter()
        .find(|member| member.key() == name)
        .map(JsonMember::value)
    else {
        return Ok(None);
    };
    if value.kind() == JsonValueKind::Null {
        return Err(Phase8RawResultSchemaError::new(
            field,
            "module_name",
            "null_not_allowed",
        ));
    }
    let Some(text) = value.string_value() else {
        return Err(Phase8RawResultSchemaError::new(
            field,
            "module_name",
            "wrong_type",
        ));
    };
    if !phase8_valid_dotted_name(text) {
        return Err(Phase8RawResultSchemaError::new(
            field,
            "module_name",
            "invalid_name_format",
        ));
    }
    Ok(Some(text.to_owned()))
}

fn raw_optional_hash(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
) -> Result<Option<Hash>, Phase8RawResultSchemaError> {
    if duplicate_member(members, name) {
        return Err(Phase8RawResultSchemaError::new(
            field,
            "unique_object_keys",
            "duplicate_field",
        ));
    }
    let Some(value) = members
        .iter()
        .find(|member| member.key() == name)
        .map(JsonMember::value)
    else {
        return Ok(None);
    };
    if value.kind() == JsonValueKind::Null {
        return Err(Phase8RawResultSchemaError::new(
            field,
            "sha256:<lower-hex>",
            "null_not_allowed",
        ));
    }
    let Some(text) = value.string_value() else {
        return Err(Phase8RawResultSchemaError::new(
            field,
            "sha256:<lower-hex>",
            "wrong_type",
        ));
    };
    parse_hash_string(text).map(Some).map_err(|_| {
        Phase8RawResultSchemaError::new(field, "sha256:<lower-hex>", "invalid_hash_format")
    })
}

fn raw_optional_error(
    members: &[JsonMember<'_>],
) -> Result<Option<Phase8MachineCheckError>, Phase8RawResultSchemaError> {
    if duplicate_member(members, "error") {
        return Err(Phase8RawResultSchemaError::new(
            "checker_raw.error",
            "unique_object_keys",
            "duplicate_field",
        ));
    }
    let Some(value) = members
        .iter()
        .find(|member| member.key() == "error")
        .map(JsonMember::value)
    else {
        return Ok(None);
    };
    if value.kind() == JsonValueKind::Null {
        return Err(Phase8RawResultSchemaError::new(
            "checker_raw.error",
            "object",
            "null_not_allowed",
        ));
    }
    let Some(error_members) = value.object_members() else {
        return Err(Phase8RawResultSchemaError::new(
            "checker_raw.error",
            "object",
            "wrong_type",
        ));
    };
    let kind = raw_required_string(
        error_members,
        "kind",
        "checker_raw.error.kind",
        "checker_raw_error_kind",
    )?;
    if !phase8_raw_checker_error_kind_allowed(&kind) {
        return Err(Phase8RawResultSchemaError::new(
            "checker_raw.error.kind",
            "checker_raw_error_kind",
            "invalid_enum",
        ));
    }
    let mut error = Phase8MachineCheckError::new(kind.clone());
    if kind == "checker_internal_error" {
        let reason_code = raw_required_string(
            error_members,
            "reason_code",
            "checker_raw.error.reason_code",
            "checker_raw_internal_reason_code",
        )?;
        if reason_code != "checker_reported_internal_error" {
            return Err(Phase8RawResultSchemaError::new(
                "checker_raw.error.reason_code",
                "checker_raw_internal_reason_code",
                "invalid_enum",
            ));
        }
        error.reason_code = Some(reason_code);
    } else if duplicate_member(error_members, "reason_code")
        || error_members
            .iter()
            .any(|member| member.key() == "reason_code")
    {
        return Err(Phase8RawResultSchemaError::new(
            "checker_raw.error.reason_code",
            "absent_for_error_kind",
            "forbidden_field",
        ));
    }
    if let Some(declaration) = raw_optional_module(
        error_members,
        "declaration",
        "checker_raw.error.declaration",
    )? {
        error.declaration = Some(declaration);
    }
    if let Some(core_path) = raw_optional_core_path(error_members)? {
        error.core_path = Some(core_path);
    }
    if let Some(section) = raw_optional_identity_string(
        error_members,
        "section",
        "checker_raw.error.section",
        "string",
    )? {
        error.section = Some(section);
    }
    if let Some(offset) =
        raw_optional_u64(error_members, "offset", "checker_raw.error.offset", "u64")?
    {
        error.offset = Some(offset);
    }
    error.expected_hash = raw_optional_hash(
        error_members,
        "expected_hash",
        "checker_raw.error.expected_hash",
    )?;
    error.actual_hash = raw_optional_hash(
        error_members,
        "actual_hash",
        "checker_raw.error.actual_hash",
    )?;
    raw_reject_unknown_fields(
        error_members,
        &[
            "kind",
            "reason_code",
            "declaration",
            "core_path",
            "section",
            "offset",
            "expected_hash",
            "actual_hash",
        ],
        "checker_raw.error",
    )?;
    Ok(Some(error))
}

fn raw_optional_core_path(
    members: &[JsonMember<'_>],
) -> Result<Option<Vec<String>>, Phase8RawResultSchemaError> {
    if duplicate_member(members, "core_path") {
        return Err(Phase8RawResultSchemaError::new(
            "checker_raw.error.core_path",
            "unique_object_keys",
            "duplicate_field",
        ));
    }
    let Some(value) = members
        .iter()
        .find(|member| member.key() == "core_path")
        .map(JsonMember::value)
    else {
        return Ok(None);
    };
    if value.kind() == JsonValueKind::Null {
        return Err(Phase8RawResultSchemaError::new(
            "checker_raw.error.core_path",
            "array",
            "null_not_allowed",
        ));
    }
    let Some(elements) = value.array_elements() else {
        return Err(Phase8RawResultSchemaError::new(
            "checker_raw.error.core_path",
            "array",
            "wrong_type",
        ));
    };
    let mut out = Vec::new();
    for (index, element) in elements.iter().enumerate() {
        let Some(segment) = element.string_value() else {
            return Err(Phase8RawResultSchemaError::new(
                format!("checker_raw.error.core_path[{index}]"),
                "string",
                if element.kind() == JsonValueKind::Null {
                    "null_not_allowed"
                } else {
                    "wrong_type"
                },
            ));
        };
        out.push(segment.to_owned());
    }
    Ok(Some(out))
}

fn raw_optional_u64(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
    expected: &str,
) -> Result<Option<u64>, Phase8RawResultSchemaError> {
    if duplicate_member(members, name) {
        return Err(Phase8RawResultSchemaError::new(
            field,
            "unique_object_keys",
            "duplicate_field",
        ));
    }
    let Some(value) = members
        .iter()
        .find(|member| member.key() == name)
        .map(JsonMember::value)
    else {
        return Ok(None);
    };
    if value.kind() == JsonValueKind::Null {
        return Err(Phase8RawResultSchemaError::new(
            field,
            expected,
            "null_not_allowed",
        ));
    }
    let Some(raw) = value.number_raw() else {
        return Err(Phase8RawResultSchemaError::new(
            field,
            expected,
            "wrong_type",
        ));
    };
    if raw.contains('.') || raw.contains('e') || raw.contains('E') {
        return Err(Phase8RawResultSchemaError::new(
            field,
            expected,
            "invalid_integer_format",
        ));
    }
    raw.parse::<u64>()
        .map(Some)
        .map_err(|_| Phase8RawResultSchemaError::new(field, expected, "integer_out_of_range"))
}

fn phase8_raw_checker_error_kind_allowed(kind: &str) -> bool {
    matches!(
        kind,
        "certificate_decode_error"
            | "noncanonical_encoding"
            | "unsupported_schema_version"
            | "import_not_found"
            | "import_hash_mismatch"
            | "certificate_hash_mismatch"
            | "axiom_report_mismatch"
            | "export_hash_mismatch"
            | "type_mismatch"
            | "conversion_failure"
            | "universe_inconsistency"
            | "inductive_invalid"
            | "positivity_failure"
            | "declaration_hash_mismatch"
            | "dependency_hash_mismatch"
            | "forbidden_axiom"
            | "checker_internal_error"
    )
}

fn raw_reject_unknown_fields(
    members: &[JsonMember<'_>],
    allowed: &[&str],
    container_path: &str,
) -> Result<(), Phase8RawResultSchemaError> {
    let mut unknown = members
        .iter()
        .map(JsonMember::key)
        .filter(|field| !allowed.contains(field))
        .collect::<Vec<_>>();
    unknown.sort_by(|left, right| phase8_rfc8785_object_key_cmp(left, right));
    if let Some(field) = unknown.first() {
        let report_path = unknown_field_report_path(container_path, field);
        return Err(Phase8RawResultSchemaError::new(
            report_path,
            "absent",
            "unknown_field",
        ));
    }
    Ok(())
}

fn parse_phase8_machine_result_store_manifest_value(
    value: &JsonValue<'_>,
    root_path: &str,
) -> Result<Phase8MachineResultStoreManifest, Phase8RequestValidationError> {
    let members = object_members_or_policy_error(value, root_path, "object")?;
    required_fixed_string_field(
        members,
        "schema",
        &format!("{root_path}.schema"),
        PHASE8_MACHINE_RESULT_STORE_MANIFEST_SCHEMA,
        PHASE8_MACHINE_RESULT_STORE_MANIFEST_SCHEMA,
    )?;
    let results_value =
        required_field_value(members, "results", &format!("{root_path}.results"), "array")?;
    let Some(result_values) = results_value.array_elements() else {
        return Err(wrong_type_error(
            &format!("{root_path}.results"),
            "array",
            results_value.kind(),
        )
        .into());
    };
    let mut results = Vec::new();
    for (index, result_value) in result_values.iter().enumerate() {
        let path = format!("{root_path}.results[{index}]");
        let result_members = object_members_or_policy_error(result_value, &path, "object")?;
        let result_hash = required_hash_field(
            result_members,
            "result_hash",
            &format!("{path}.result_hash"),
            "sha256:<lower-hex>",
        )?;
        let request_hash = required_hash_field(
            result_members,
            "request_hash",
            &format!("{path}.request_hash"),
            "sha256:<lower-hex>",
        )?;
        let run_artifact_hash = required_hash_field(
            result_members,
            "run_artifact_hash",
            &format!("{path}.run_artifact_hash"),
            "sha256:<lower-hex>",
        )?;
        let checker_profile = required_string_field(
            result_members,
            "checker_profile",
            &format!("{path}.checker_profile"),
            "checker_profile_name",
        )?;
        if !phase8_valid_checker_profile_name(&checker_profile) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{path}.checker_profile"),
                "checker_profile_name",
                "invalid_name_format",
            ));
        }
        let result_path = required_string_field(
            result_members,
            "path",
            &format!("{path}.path"),
            "workspace_relative_path",
        )?;
        if !phase8_valid_workspace_relative_path(&result_path) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{path}.path"),
                "workspace_relative_path",
                "invalid_path",
            ));
        }
        let file_hash = required_hash_field(
            result_members,
            "file_hash",
            &format!("{path}.file_hash"),
            "sha256:<lower-hex>",
        )?;
        reject_unknown_fields(result_members, MACHINE_RESULT_STORE_ENTRY_FIELDS, &path)?;
        results.push(Phase8MachineResultStoreEntry {
            result_hash,
            request_hash,
            run_artifact_hash,
            checker_profile,
            path: result_path,
            file_hash,
        });
    }
    reject_unknown_fields(members, MACHINE_RESULT_STORE_MANIFEST_FIELDS, root_path)?;
    validate_machine_result_store_domain(&results, root_path)?;
    Ok(Phase8MachineResultStoreManifest { results })
}

fn parse_phase8_axiom_report_value(
    value: &JsonValue<'_>,
) -> Result<Phase8AxiomReport, Phase8RequestValidationError> {
    let members = object_members_or_policy_error(value, "$", "object")?;
    required_fixed_string_field(
        members,
        "schema",
        "schema",
        PHASE8_AXIOM_REPORT_SCHEMA,
        PHASE8_AXIOM_REPORT_SCHEMA,
    )?;
    let parsed_hash = required_hash_field(
        members,
        "axiom_report_hash",
        "axiom_report_hash",
        "sha256:<lower-hex>",
    )?;
    let module = required_string_field(members, "module", "module", "module_name")?;
    if !phase8_valid_dotted_name(&module) {
        return Err(Phase8RequestValidationError::value_failure(
            "module",
            "module_name",
            "invalid_name_format",
        ));
    }
    let certificate_hash = required_hash_field(
        members,
        "certificate_hash",
        "certificate_hash",
        "sha256:<lower-hex>",
    )?;
    let axioms_value = required_field_value(members, "axioms", "axioms", "array")?;
    let Some(axiom_values) = axioms_value.array_elements() else {
        return Err(wrong_type_error("axioms", "array", axioms_value.kind()).into());
    };
    let mut axioms = Vec::new();
    for (index, axiom_value) in axiom_values.iter().enumerate() {
        let path = format!("axioms[{index}]");
        let axiom_members = object_members_or_policy_error(axiom_value, &path, "object")?;
        let name = required_string_field(
            axiom_members,
            "name",
            &format!("{path}.name"),
            "phase2_name",
        )?;
        if !phase8_valid_dotted_name(&name) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{path}.name"),
                "phase2_name",
                "invalid_name_format",
            ));
        }
        reject_unknown_fields(axiom_members, AXIOM_REPORT_ENTRY_FIELDS, &path)?;
        axioms.push(Phase8AxiomReportEntry { name });
    }
    reject_unknown_fields(members, AXIOM_REPORT_FIELDS, "$")?;
    validate_axiom_report_domain(&axioms)?;
    let report = Phase8AxiomReport {
        module,
        certificate_hash,
        axioms,
    };
    let recomputed = report.axiom_report_hash();
    if recomputed != parsed_hash {
        return Err(Phase8RequestValidationError::hash_failure(
            "axiom_report_hash",
            recomputed,
            parsed_hash,
        ));
    }
    Ok(report)
}

fn parse_phase8_axiom_report_store_manifest_value(
    value: &JsonValue<'_>,
) -> Result<Phase8AxiomReportStoreManifest, Phase8RequestValidationError> {
    let members = object_members_or_policy_error(value, "$", "object")?;
    required_fixed_string_field(
        members,
        "schema",
        "schema",
        PHASE8_AXIOM_REPORT_STORE_MANIFEST_SCHEMA,
        PHASE8_AXIOM_REPORT_STORE_MANIFEST_SCHEMA,
    )?;
    let reports_value = required_field_value(members, "reports", "reports", "array")?;
    let Some(report_values) = reports_value.array_elements() else {
        return Err(wrong_type_error("reports", "array", reports_value.kind()).into());
    };
    let mut reports = Vec::new();
    for (index, report_value) in report_values.iter().enumerate() {
        let path = format!("reports[{index}]");
        let report_members = object_members_or_policy_error(report_value, &path, "object")?;
        let axiom_report_hash = required_hash_field(
            report_members,
            "axiom_report_hash",
            &format!("{path}.axiom_report_hash"),
            "sha256:<lower-hex>",
        )?;
        let report_path = required_string_field(
            report_members,
            "path",
            &format!("{path}.path"),
            "workspace_relative_path",
        )?;
        if !phase8_valid_workspace_relative_path(&report_path) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{path}.path"),
                "workspace_relative_path",
                "invalid_path",
            ));
        }
        let file_hash = required_hash_field(
            report_members,
            "file_hash",
            &format!("{path}.file_hash"),
            "sha256:<lower-hex>",
        )?;
        reject_unknown_fields(report_members, AXIOM_REPORT_STORE_ENTRY_FIELDS, &path)?;
        reports.push(Phase8AxiomReportStoreEntry {
            axiom_report_hash,
            path: report_path,
            file_hash,
        });
    }
    reject_unknown_fields(members, AXIOM_REPORT_STORE_MANIFEST_FIELDS, "$")?;
    validate_axiom_report_store_domain(&reports)?;
    Ok(Phase8AxiomReportStoreManifest { reports })
}

fn parse_phase8_normalized_result_store_manifest_value(
    value: &JsonValue<'_>,
    root_path: &str,
) -> Result<Phase8NormalizedResultStoreManifest, Phase8RequestValidationError> {
    let members = object_members_or_policy_error(value, root_path, "object")?;
    required_fixed_string_field(
        members,
        "schema",
        &format!("{root_path}.schema"),
        PHASE8_NORMALIZED_RESULT_STORE_MANIFEST_SCHEMA,
        PHASE8_NORMALIZED_RESULT_STORE_MANIFEST_SCHEMA,
    )?;
    let results_value =
        required_field_value(members, "results", &format!("{root_path}.results"), "array")?;
    let Some(result_values) = results_value.array_elements() else {
        return Err(wrong_type_error(
            &format!("{root_path}.results"),
            "array",
            results_value.kind(),
        )
        .into());
    };
    let mut results = Vec::new();
    for (index, result_value) in result_values.iter().enumerate() {
        let path = format!("{root_path}.results[{index}]");
        let result_members = object_members_or_policy_error(result_value, &path, "object")?;
        let normalized_result_hash = required_hash_field(
            result_members,
            "normalized_result_hash",
            &format!("{path}.normalized_result_hash"),
            "sha256:<lower-hex>",
        )?;
        let artifact_hash = required_hash_field(
            result_members,
            "artifact_hash",
            &format!("{path}.artifact_hash"),
            "sha256:<lower-hex>",
        )?;
        let result_path = required_string_field(
            result_members,
            "path",
            &format!("{path}.path"),
            "workspace_relative_path",
        )?;
        if !phase8_valid_workspace_relative_path(&result_path) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{path}.path"),
                "workspace_relative_path",
                "invalid_path",
            ));
        }
        let file_hash = required_hash_field(
            result_members,
            "file_hash",
            &format!("{path}.file_hash"),
            "sha256:<lower-hex>",
        )?;
        reject_unknown_fields(result_members, NORMALIZED_RESULT_STORE_ENTRY_FIELDS, &path)?;
        results.push(Phase8NormalizedResultStoreEntry {
            normalized_result_hash,
            artifact_hash,
            path: result_path,
            file_hash,
        });
    }
    reject_unknown_fields(members, NORMALIZED_RESULT_STORE_MANIFEST_FIELDS, root_path)?;
    validate_normalized_result_store_domain(&results, root_path)?;
    Ok(Phase8NormalizedResultStoreManifest { results })
}

fn parse_phase8_store_reference_value(
    value: &JsonValue<'_>,
    root_path: &str,
) -> Result<(String, Hash), Phase8RequestValidationError> {
    let members = object_members_or_policy_error(value, root_path, "object")?;
    required_fixed_string_field(
        members,
        "kind",
        &format!("{root_path}.kind"),
        "manifest",
        "manifest",
    )?;
    let path = required_string_field(
        members,
        "path",
        &format!("{root_path}.path"),
        "workspace_relative_path",
    )?;
    if !phase8_valid_workspace_relative_path(&path) {
        return Err(Phase8RequestValidationError::value_failure(
            format!("{root_path}.path"),
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    let manifest_hash = required_hash_field(
        members,
        "manifest_hash",
        &format!("{root_path}.manifest_hash"),
        "sha256:<lower-hex>",
    )?;
    reject_unknown_fields(members, STORE_REFERENCE_FIELDS, root_path)?;
    Ok((path, manifest_hash))
}

fn parse_phase8_import_lock_manifest_value(
    value: &JsonValue<'_>,
    root_path: &str,
) -> Result<Phase8ImportLockManifest, Phase8RequestValidationError> {
    let members = object_members_or_policy_error(value, root_path, "object")?;
    required_fixed_string_field(
        members,
        "schema",
        &format!("{root_path}.schema"),
        PHASE8_IMPORT_LOCK_MANIFEST_SCHEMA,
        PHASE8_IMPORT_LOCK_MANIFEST_SCHEMA,
    )?;
    let imports_value =
        required_field_value(members, "imports", &format!("{root_path}.imports"), "array")?;
    let Some(import_values) = imports_value.array_elements() else {
        return Err(wrong_type_error(
            &format!("{root_path}.imports"),
            "array",
            imports_value.kind(),
        )
        .into());
    };

    let mut imports = Vec::new();
    for (index, import_value) in import_values.iter().enumerate() {
        let path = format!("{root_path}.imports[{index}]");
        let import_members = object_members_or_policy_error(import_value, &path, "object")?;
        let module = required_string_field(
            import_members,
            "module",
            &format!("{path}.module"),
            "module_name",
        )?;
        if !phase8_valid_dotted_name(&module) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{path}.module"),
                "module_name",
                "invalid_name_format",
            ));
        }
        let export_hash = required_hash_field(
            import_members,
            "export_hash",
            &format!("{path}.export_hash"),
            "sha256:<lower-hex>",
        )?;
        let certificate_value = required_field_value(
            import_members,
            "certificate",
            &format!("{path}.certificate"),
            "object",
        )?;
        let certificate_members = object_members_or_policy_error(
            certificate_value,
            &format!("{path}.certificate"),
            "object",
        )?;
        required_fixed_string_field(
            certificate_members,
            "kind",
            &format!("{path}.certificate.kind"),
            "path",
            "path",
        )?;
        let certificate_path = required_string_field(
            certificate_members,
            "path",
            &format!("{path}.certificate.path"),
            "workspace_relative_path",
        )?;
        if !phase8_valid_workspace_relative_path(&certificate_path) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{path}.certificate.path"),
                "workspace_relative_path",
                "invalid_path",
            ));
        }
        let file_hash = required_hash_field(
            certificate_members,
            "file_hash",
            &format!("{path}.certificate.file_hash"),
            "sha256:<lower-hex>",
        )?;
        let certificate_hash = required_hash_field(
            certificate_members,
            "certificate_hash",
            &format!("{path}.certificate.certificate_hash"),
            "sha256:<lower-hex>",
        )?;
        reject_unknown_fields(
            certificate_members,
            IMPORT_LOCK_CERTIFICATE_FIELDS,
            &format!("{path}.certificate"),
        )?;
        reject_unknown_fields(import_members, IMPORT_LOCK_ENTRY_FIELDS, &path)?;
        imports.push(Phase8ImportLockEntry {
            module,
            export_hash,
            certificate: Phase8ImportLockCertificate {
                path: certificate_path,
                file_hash,
                certificate_hash,
            },
        });
    }
    reject_unknown_fields(members, IMPORT_LOCK_MANIFEST_FIELDS, root_path)?;
    validate_import_lock_domain(&imports, root_path)?;
    Ok(Phase8ImportLockManifest { imports })
}

fn parse_phase8_machine_check_request_value(
    value: &JsonValue<'_>,
) -> Result<Phase8MachineCheckRequest, Phase8RequestValidationError> {
    let members = object_members_or_policy_error(value, "$", "object")?;
    if !members.iter().any(|member| member.key() == "request_hash") {
        return Err(Phase8RequestValidationError::value_failure(
            "request_hash",
            "sha256:<lower-hex>",
            "missing",
        ));
    }

    required_fixed_string_field(
        members,
        "schema",
        "schema",
        PHASE8_MACHINE_CHECK_REQUEST_SCHEMA,
        PHASE8_MACHINE_CHECK_REQUEST_SCHEMA,
    )?;
    let request_id = required_string_field(members, "request_id", "request_id", "request_id")?;
    if !phase8_valid_request_id(&request_id) {
        return Err(Phase8RequestValidationError::value_failure(
            "request_id",
            "request_id",
            if request_id.is_empty() {
                "empty_string"
            } else {
                "invalid_string_format"
            },
        ));
    }
    let parsed_request_hash = required_hash_field(
        members,
        "request_hash",
        "request_hash",
        "sha256:<lower-hex>",
    )?;
    let module = required_string_field(members, "module", "module", "module_name")?;
    if !phase8_valid_dotted_name(&module) {
        return Err(Phase8RequestValidationError::value_failure(
            "module",
            "module_name",
            "invalid_name_format",
        ));
    }
    let policy = parse_machine_check_request_policy(members)?;
    let certificate = parse_machine_check_request_certificate(members)?;
    let imports = parse_machine_check_request_imports(members)?;
    let checker_profile = required_string_field(
        members,
        "checker_profile",
        "checker_profile",
        "checker_profile_name",
    )?;
    if !phase8_valid_checker_profile_name(&checker_profile) {
        return Err(Phase8RequestValidationError::value_failure(
            "checker_profile",
            "checker_profile_name",
            "invalid_name_format",
        ));
    }
    let trust_mode_raw = required_string_field(members, "trust_mode", "trust_mode", "trust_mode")?;
    let trust_mode = Phase8TrustMode::parse(&trust_mode_raw).ok_or_else(|| {
        Phase8RequestValidationError::value_failure("trust_mode", "trust_mode", "invalid_enum")
    })?;
    let axiom_policy = required_string_field(
        members,
        "axiom_policy",
        "axiom_policy",
        "workspace_relative_path",
    )?;
    if !phase8_valid_workspace_relative_path(&axiom_policy) {
        return Err(Phase8RequestValidationError::value_failure(
            "axiom_policy",
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    let budget = parse_machine_check_request_budget(members)?;
    reject_unknown_fields(members, MACHINE_CHECK_REQUEST_FIELDS, "$")?;

    let request = Phase8MachineCheckRequest {
        request_id,
        module,
        policy,
        certificate,
        imports,
        checker_profile,
        trust_mode,
        axiom_policy,
        budget,
    };
    let recomputed = request.request_hash();
    if recomputed != parsed_request_hash {
        return Err(Phase8RequestValidationError::hash_failure(
            "request_hash",
            recomputed,
            parsed_request_hash,
        ));
    }
    Ok(request)
}

fn parse_phase8_request_store_manifest_value(
    value: &JsonValue<'_>,
    root_path: &str,
) -> Result<Phase8RequestStoreManifest, Phase8RequestValidationError> {
    let members = object_members_or_policy_error(value, root_path, "object")?;
    required_fixed_string_field(
        members,
        "schema",
        &format!("{root_path}.schema"),
        PHASE8_REQUEST_STORE_MANIFEST_SCHEMA,
        PHASE8_REQUEST_STORE_MANIFEST_SCHEMA,
    )?;
    let requests_value = required_field_value(
        members,
        "requests",
        &format!("{root_path}.requests"),
        "array",
    )?;
    let Some(request_values) = requests_value.array_elements() else {
        return Err(wrong_type_error(
            &format!("{root_path}.requests"),
            "array",
            requests_value.kind(),
        )
        .into());
    };

    let mut requests = Vec::new();
    for (index, request_value) in request_values.iter().enumerate() {
        let path = format!("{root_path}.requests[{index}]");
        let request_members = object_members_or_policy_error(request_value, &path, "object")?;
        let request_hash = required_hash_field(
            request_members,
            "request_hash",
            &format!("{path}.request_hash"),
            "sha256:<lower-hex>",
        )?;
        let request_path = required_string_field(
            request_members,
            "path",
            &format!("{path}.path"),
            "workspace_relative_path",
        )?;
        if !phase8_valid_workspace_relative_path(&request_path) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{path}.path"),
                "workspace_relative_path",
                "invalid_path",
            ));
        }
        let file_hash = required_hash_field(
            request_members,
            "file_hash",
            &format!("{path}.file_hash"),
            "sha256:<lower-hex>",
        )?;
        reject_unknown_fields(request_members, REQUEST_STORE_ENTRY_FIELDS, &path)?;
        requests.push(Phase8RequestStoreEntry {
            request_hash,
            path: request_path,
            file_hash,
        });
    }
    reject_unknown_fields(members, REQUEST_STORE_MANIFEST_FIELDS, root_path)?;
    validate_request_store_domain(&requests, root_path)?;
    Ok(Phase8RequestStoreManifest { requests })
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

const IMPORT_LOCK_MANIFEST_FIELDS: &[&str] = &["schema", "imports"];
const IMPORT_LOCK_ENTRY_FIELDS: &[&str] = &["module", "export_hash", "certificate"];
const IMPORT_LOCK_CERTIFICATE_FIELDS: &[&str] = &["kind", "path", "file_hash", "certificate_hash"];
const MACHINE_CHECK_REQUEST_FIELDS: &[&str] = &[
    "schema",
    "request_id",
    "request_hash",
    "module",
    "policy",
    "certificate",
    "imports",
    "checker_profile",
    "trust_mode",
    "axiom_policy",
    "budget",
];
const MACHINE_CHECK_REQUEST_POLICY_FIELDS: &[&str] = &["id", "version", "hash"];
const MACHINE_CHECK_REQUEST_CERTIFICATE_FIELDS: &[&str] =
    &["kind", "path", "file_hash", "expected_certificate_hash"];
const MACHINE_CHECK_REQUEST_IMPORTS_FIELDS: &[&str] = &["mode", "manifest", "manifest_hash"];
const MACHINE_CHECK_REQUEST_BUDGET_FIELDS: &[&str] = &["max_steps", "max_memory_mb", "timeout_ms"];
const REQUEST_STORE_MANIFEST_FIELDS: &[&str] = &["schema", "requests"];
const REQUEST_STORE_ENTRY_FIELDS: &[&str] = &["request_hash", "path", "file_hash"];
const MACHINE_RESULT_STORE_MANIFEST_FIELDS: &[&str] = &["schema", "results"];
const MACHINE_RESULT_STORE_ENTRY_FIELDS: &[&str] = &[
    "result_hash",
    "request_hash",
    "run_artifact_hash",
    "checker_profile",
    "path",
    "file_hash",
];
const AXIOM_REPORT_FIELDS: &[&str] = &[
    "schema",
    "axiom_report_hash",
    "module",
    "certificate_hash",
    "axioms",
];
const AXIOM_REPORT_ENTRY_FIELDS: &[&str] = &["name"];
const AXIOM_REPORT_STORE_MANIFEST_FIELDS: &[&str] = &["schema", "reports"];
const AXIOM_REPORT_STORE_ENTRY_FIELDS: &[&str] = &["axiom_report_hash", "path", "file_hash"];
const NORMALIZED_RESULT_STORE_MANIFEST_FIELDS: &[&str] = &["schema", "results"];
const NORMALIZED_RESULT_STORE_ENTRY_FIELDS: &[&str] = &[
    "normalized_result_hash",
    "artifact_hash",
    "path",
    "file_hash",
];
const STORE_REFERENCE_FIELDS: &[&str] = &["kind", "path", "manifest_hash"];

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

fn parse_machine_check_request_policy(
    members: &[JsonMember<'_>],
) -> Result<Phase8MachineCheckRequestPolicy, Phase8RequestValidationError> {
    let value = required_field_value(members, "policy", "policy", "object")?;
    let policy_members = object_members_or_policy_error(value, "policy", "object")?;
    let id = required_string_field(policy_members, "id", "policy.id", "runner_policy_id")?;
    if !phase8_valid_runner_policy_id(&id) {
        return Err(Phase8RequestValidationError::value_failure(
            "policy.id",
            "runner_policy_id",
            "invalid_name_format",
        ));
    }
    let version_raw =
        required_integer_raw_field(policy_members, "version", "policy.version", "positive_i64")?;
    let version = parse_positive_i64_domain(&version_raw, "policy.version")?;
    let hash = required_hash_field(policy_members, "hash", "policy.hash", "sha256:<lower-hex>")?;
    reject_unknown_fields(
        policy_members,
        MACHINE_CHECK_REQUEST_POLICY_FIELDS,
        "policy",
    )?;
    Ok(Phase8MachineCheckRequestPolicy { id, version, hash })
}

fn parse_machine_check_request_certificate(
    members: &[JsonMember<'_>],
) -> Result<Phase8MachineCheckRequestCertificate, Phase8RequestValidationError> {
    let value = required_field_value(members, "certificate", "certificate", "object")?;
    let certificate_members = object_members_or_policy_error(value, "certificate", "object")?;
    required_fixed_string_field(
        certificate_members,
        "kind",
        "certificate.kind",
        "path",
        "path",
    )?;
    let path = required_string_field(
        certificate_members,
        "path",
        "certificate.path",
        "workspace_relative_path",
    )?;
    if !phase8_valid_workspace_relative_path(&path) {
        return Err(Phase8RequestValidationError::value_failure(
            "certificate.path",
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    let file_hash = required_hash_field(
        certificate_members,
        "file_hash",
        "certificate.file_hash",
        "sha256:<lower-hex>",
    )?;
    let expected_certificate_hash = required_hash_field(
        certificate_members,
        "expected_certificate_hash",
        "certificate.expected_certificate_hash",
        "sha256:<lower-hex>",
    )?;
    reject_unknown_fields(
        certificate_members,
        MACHINE_CHECK_REQUEST_CERTIFICATE_FIELDS,
        "certificate",
    )?;
    Ok(Phase8MachineCheckRequestCertificate {
        path,
        file_hash,
        expected_certificate_hash,
    })
}

fn parse_machine_check_request_imports(
    members: &[JsonMember<'_>],
) -> Result<Phase8MachineCheckRequestImports, Phase8RequestValidationError> {
    let value = required_field_value(members, "imports", "imports", "object")?;
    let imports_members = object_members_or_policy_error(value, "imports", "object")?;
    let mode = required_string_field(imports_members, "mode", "imports.mode", "locked_store")?;
    if mode != "locked_store" {
        return Err(Phase8RequestValidationError::value_failure(
            "imports.mode",
            "locked_store",
            "invalid_enum",
        ));
    }
    let manifest = required_string_field(
        imports_members,
        "manifest",
        "imports.manifest",
        "workspace_relative_path",
    )?;
    if !phase8_valid_workspace_relative_path(&manifest) {
        return Err(Phase8RequestValidationError::value_failure(
            "imports.manifest",
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    let manifest_hash = required_hash_field(
        imports_members,
        "manifest_hash",
        "imports.manifest_hash",
        "sha256:<lower-hex>",
    )?;
    reject_unknown_fields(
        imports_members,
        MACHINE_CHECK_REQUEST_IMPORTS_FIELDS,
        "imports",
    )?;
    Ok(Phase8MachineCheckRequestImports {
        mode,
        manifest,
        manifest_hash,
    })
}

fn parse_machine_check_request_budget(
    members: &[JsonMember<'_>],
) -> Result<Phase8RunnerBudget, Phase8RequestValidationError> {
    let value = required_field_value(members, "budget", "budget", "object")?;
    let budget_members = object_members_or_policy_error(value, "budget", "object")?;
    let max_steps_raw = required_integer_raw_field(
        budget_members,
        "max_steps",
        "budget.max_steps",
        "positive_i64",
    )?;
    let max_memory_mb_raw = required_integer_raw_field(
        budget_members,
        "max_memory_mb",
        "budget.max_memory_mb",
        "positive_i64",
    )?;
    let timeout_ms_raw = required_integer_raw_field(
        budget_members,
        "timeout_ms",
        "budget.timeout_ms",
        "positive_i64",
    )?;
    reject_unknown_fields(
        budget_members,
        MACHINE_CHECK_REQUEST_BUDGET_FIELDS,
        "budget",
    )?;
    Ok(Phase8RunnerBudget {
        max_steps: parse_positive_i64_domain(&max_steps_raw, "budget.max_steps")?,
        max_memory_mb: parse_positive_i64_domain(&max_memory_mb_raw, "budget.max_memory_mb")?,
        timeout_ms: parse_positive_i64_domain(&timeout_ms_raw, "budget.timeout_ms")?,
    })
}

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

fn validate_import_lock_domain(
    imports: &[Phase8ImportLockEntry],
    root_path: &str,
) -> Result<(), Phase8RequestValidationError> {
    for index in 1..imports.len() {
        if import_lock_sort_key_cmp(&imports[index], &imports[index - 1]) == Ordering::Less {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{root_path}.imports[{index}]"),
                "imports_module_path_certificate_hash_file_hash_ascending",
                "order_violation",
            ));
        }
    }

    let mut modules = BTreeSet::new();
    for (index, import) in imports.iter().enumerate() {
        if !modules.insert(&import.module) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{root_path}.imports[{index}].module"),
                "unique_modules",
                "duplicate_module",
            ));
        }
    }

    let mut paths = BTreeSet::new();
    for (index, import) in imports.iter().enumerate() {
        if !paths.insert(&import.certificate.path) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{root_path}.imports[{index}].certificate.path"),
                "unique_certificate_paths",
                "duplicate_path",
            ));
        }
    }

    let mut certificate_hashes = BTreeSet::new();
    for (index, import) in imports.iter().enumerate() {
        if !certificate_hashes.insert(import.certificate.certificate_hash) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{root_path}.imports[{index}].certificate.certificate_hash"),
                "unique_certificate_hashes",
                "duplicate_certificate_hash",
            ));
        }
    }

    let mut file_hashes = BTreeSet::new();
    for (index, import) in imports.iter().enumerate() {
        if !file_hashes.insert(import.certificate.file_hash) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{root_path}.imports[{index}].certificate.file_hash"),
                "unique_file_hashes",
                "duplicate_file_hash",
            ));
        }
    }
    Ok(())
}

fn import_lock_sort_key_cmp(
    left: &Phase8ImportLockEntry,
    right: &Phase8ImportLockEntry,
) -> Ordering {
    phase8_dotted_name_cmp(&left.module, &right.module)
        .then_with(|| left.certificate.path.cmp(&right.certificate.path))
        .then_with(|| {
            left.certificate
                .certificate_hash
                .cmp(&right.certificate.certificate_hash)
        })
        .then_with(|| left.certificate.file_hash.cmp(&right.certificate.file_hash))
}

fn validate_request_store_domain(
    requests: &[Phase8RequestStoreEntry],
    root_path: &str,
) -> Result<(), Phase8RequestValidationError> {
    for index in 1..requests.len() {
        if requests[index].request_hash < requests[index - 1].request_hash {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{root_path}.requests[{index}]"),
                "request_hash_bytewise_ascending",
                "order_violation",
            ));
        }
    }

    let mut request_hashes = BTreeSet::new();
    for (index, request) in requests.iter().enumerate() {
        if !request_hashes.insert(request.request_hash) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{root_path}.requests[{index}].request_hash"),
                "unique_request_hashes",
                "duplicate_request_hash",
            ));
        }
    }

    let mut paths = BTreeSet::new();
    for (index, request) in requests.iter().enumerate() {
        if !paths.insert(&request.path) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{root_path}.requests[{index}].path"),
                "unique_paths",
                "duplicate_path",
            ));
        }
    }
    Ok(())
}

fn validate_machine_result_store_domain(
    results: &[Phase8MachineResultStoreEntry],
    root_path: &str,
) -> Result<(), Phase8RequestValidationError> {
    for index in 1..results.len() {
        if results[index].run_artifact_hash < results[index - 1].run_artifact_hash {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{root_path}.results[{index}]"),
                "run_artifact_hash_bytewise_ascending",
                "order_violation",
            ));
        }
    }

    let mut run_artifact_hashes = BTreeSet::new();
    for (index, result) in results.iter().enumerate() {
        if !run_artifact_hashes.insert(result.run_artifact_hash) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{root_path}.results[{index}].run_artifact_hash"),
                "unique_run_artifact_hashes",
                "duplicate_run_artifact_hash",
            ));
        }
    }

    let mut paths = BTreeSet::new();
    for (index, result) in results.iter().enumerate() {
        if !paths.insert(&result.path) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{root_path}.results[{index}].path"),
                "unique_paths",
                "duplicate_path",
            ));
        }
    }
    Ok(())
}

fn validate_axiom_report_domain(
    axioms: &[Phase8AxiomReportEntry],
) -> Result<(), Phase8RequestValidationError> {
    for index in 1..axioms.len() {
        if phase8_dotted_name_cmp(&axioms[index].name, &axioms[index - 1].name) == Ordering::Less {
            return Err(Phase8RequestValidationError::value_failure(
                format!("axioms[{index}]"),
                "axiom_name_ascending",
                "order_violation",
            ));
        }
    }

    let mut names = BTreeSet::new();
    for (index, axiom) in axioms.iter().enumerate() {
        if !names.insert(&axiom.name) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("axioms[{index}].name"),
                "unique_axiom_names",
                "duplicate_axiom_name",
            ));
        }
    }
    Ok(())
}

fn validate_axiom_report_store_domain(
    reports: &[Phase8AxiomReportStoreEntry],
) -> Result<(), Phase8RequestValidationError> {
    for index in 1..reports.len() {
        if reports[index].axiom_report_hash < reports[index - 1].axiom_report_hash {
            return Err(Phase8RequestValidationError::value_failure(
                format!("reports[{index}]"),
                "axiom_report_hash_bytewise_ascending",
                "order_violation",
            ));
        }
    }

    let mut hashes = BTreeSet::new();
    for (index, report) in reports.iter().enumerate() {
        if !hashes.insert(report.axiom_report_hash) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("reports[{index}].axiom_report_hash"),
                "unique_axiom_report_hashes",
                "duplicate_axiom_report_hash",
            ));
        }
    }

    let mut paths = BTreeSet::new();
    for (index, report) in reports.iter().enumerate() {
        if !paths.insert(&report.path) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("reports[{index}].path"),
                "unique_paths",
                "duplicate_path",
            ));
        }
    }
    Ok(())
}

fn validate_normalized_result_store_domain(
    results: &[Phase8NormalizedResultStoreEntry],
    root_path: &str,
) -> Result<(), Phase8RequestValidationError> {
    for index in 1..results.len() {
        if results[index].normalized_result_hash < results[index - 1].normalized_result_hash {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{root_path}.results[{index}]"),
                "normalized_result_hash_bytewise_ascending",
                "order_violation",
            ));
        }
    }

    let mut hashes = BTreeSet::new();
    for (index, result) in results.iter().enumerate() {
        if !hashes.insert(result.normalized_result_hash) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{root_path}.results[{index}].normalized_result_hash"),
                "unique_normalized_result_hashes",
                "duplicate_normalized_result_hash",
            ));
        }
    }

    let mut paths = BTreeSet::new();
    for (index, result) in results.iter().enumerate() {
        if !paths.insert(&result.path) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{root_path}.results[{index}].path"),
                "unique_paths",
                "duplicate_path",
            ));
        }
    }
    Ok(())
}

fn validate_request_materialize_input_shape(
    module: &str,
    checker_profile: &str,
    request_id: &str,
    certificate_path: &str,
    imports_manifest_path: &str,
    output_request_path: &str,
) -> Result<(), Phase8CommandError> {
    if !phase8_valid_dotted_name(module) {
        return Err(phase8_command_value_error(
            Phase8CommandName::RequestMaterialize,
            "input_reference_invalid",
            "module",
            "module_name",
            "invalid_name_format",
        ));
    }
    if !phase8_valid_checker_profile_name(checker_profile) {
        return Err(phase8_command_value_error(
            Phase8CommandName::RequestMaterialize,
            "input_reference_invalid",
            "checker_profile",
            "checker_profile",
            "invalid_name_format",
        ));
    }
    if !phase8_valid_request_id(request_id) {
        return Err(phase8_command_value_error(
            Phase8CommandName::RequestMaterialize,
            "input_reference_invalid",
            "request_id",
            "request_id",
            if request_id.is_empty() {
                "empty_string"
            } else {
                "invalid_string_format"
            },
        ));
    }
    if !phase8_valid_workspace_relative_path(certificate_path) {
        return Err(phase8_command_value_error(
            Phase8CommandName::RequestMaterialize,
            "input_reference_invalid",
            "certificate.path",
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    if !phase8_valid_workspace_relative_path(imports_manifest_path) {
        return Err(phase8_command_value_error(
            Phase8CommandName::RequestMaterialize,
            "input_reference_invalid",
            "imports.path",
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    if !phase8_valid_workspace_relative_path(output_request_path) {
        return Err(phase8_command_value_error(
            Phase8CommandName::RequestMaterialize,
            "input_reference_invalid",
            "out.path",
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    Ok(())
}

fn phase8_command_error_from_request_validation(
    command: Phase8CommandName,
    reason_code: &str,
    field: String,
    error: Phase8RequestValidationError,
) -> Phase8CommandError {
    let mut command_error = Phase8CommandError::new(command, reason_code);
    command_error.field = Some(field.into_boxed_str());
    command_error.expected_hash = error.expected_hash;
    command_error.actual_hash = error.actual_hash;
    command_error.expected_value = error.expected_value;
    command_error.actual_value = error.actual_value;
    command_error
}

fn phase8_command_value_error(
    command: Phase8CommandName,
    reason_code: impl Into<String>,
    field: impl Into<String>,
    expected_value: impl Into<String>,
    actual_value: impl Into<String>,
) -> Phase8CommandError {
    let mut error = Phase8CommandError::new(command, reason_code);
    error.field = Some(field.into().into_boxed_str());
    error.expected_value = Some(expected_value.into().into_boxed_str());
    error.actual_value = Some(actual_value.into().into_boxed_str());
    error
}

fn phase8_command_hash_error(
    command: Phase8CommandName,
    reason_code: impl Into<String>,
    field: impl Into<String>,
    expected_hash: Hash,
    actual_hash: Hash,
) -> Phase8CommandError {
    let mut error = Phase8CommandError::new(command, reason_code);
    error.field = Some(field.into().into_boxed_str());
    error.expected_hash = Some(Box::new(expected_hash));
    error.actual_hash = Some(Box::new(actual_hash));
    error
}

fn phase8_raw_certificate_header_module(
    certificate_bytes: &[u8],
) -> Result<String, Phase8RawCertificateClaimError> {
    let mut decoder = Phase8RawCertificateHeaderDecoder::new(certificate_bytes);
    let _format = decoder.string()?;
    let _core_spec = decoder.string()?;
    decoder.name()
}

struct Phase8RawCertificateHeaderDecoder<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Phase8RawCertificateHeaderDecoder<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn name(&mut self) -> Result<String, Phase8RawCertificateClaimError> {
        let len = self.usize()?;
        if len == 0 {
            return Err(Phase8RawCertificateClaimError::DecodeFailed);
        }
        let mut components = Vec::with_capacity(len);
        for _ in 0..len {
            let component = self.string()?;
            if component.is_empty() || component.contains('.') {
                return Err(Phase8RawCertificateClaimError::DecodeFailed);
            }
            components.push(component);
        }
        let module = components.join(".");
        if !phase8_valid_dotted_name(&module) {
            return Err(Phase8RawCertificateClaimError::DecodeFailed);
        }
        Ok(module)
    }

    fn string(&mut self) -> Result<String, Phase8RawCertificateClaimError> {
        let len = self.usize()?;
        let bytes = self.take(len)?;
        std::str::from_utf8(bytes)
            .map(str::to_owned)
            .map_err(|_| Phase8RawCertificateClaimError::DecodeFailed)
    }

    fn usize(&mut self) -> Result<usize, Phase8RawCertificateClaimError> {
        usize::try_from(self.uvar()?).map_err(|_| Phase8RawCertificateClaimError::DecodeFailed)
    }

    fn uvar(&mut self) -> Result<u64, Phase8RawCertificateClaimError> {
        let start = self.offset;
        let mut shift = 0;
        let mut value = 0u64;
        loop {
            let byte = self.byte()?;
            value |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                if phase8_encode_uvar(value) != self.bytes[start..self.offset] {
                    return Err(Phase8RawCertificateClaimError::DecodeFailed);
                }
                return Ok(value);
            }
            shift += 7;
            if shift >= 64 {
                return Err(Phase8RawCertificateClaimError::DecodeFailed);
            }
        }
    }

    fn byte(&mut self) -> Result<u8, Phase8RawCertificateClaimError> {
        let byte = *self
            .bytes
            .get(self.offset)
            .ok_or(Phase8RawCertificateClaimError::DecodeFailed)?;
        self.offset += 1;
        Ok(byte)
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], Phase8RawCertificateClaimError> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or(Phase8RawCertificateClaimError::DecodeFailed)?;
        let bytes = self
            .bytes
            .get(self.offset..end)
            .ok_or(Phase8RawCertificateClaimError::DecodeFailed)?;
        self.offset = end;
        Ok(bytes)
    }
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
            } else if expected == "file" || expected == "path" {
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

fn phase8_valid_request_id(value: &str) -> bool {
    phase8_visible_ascii_nonempty(value)
}

fn phase8_valid_dotted_name(value: &str) -> bool {
    parse_module_name_wire(value).is_ok()
}

fn phase8_dotted_name_cmp(left: &str, right: &str) -> Ordering {
    phase8_dotted_name_sort_key(left).cmp(&phase8_dotted_name_sort_key(right))
}

fn phase8_dotted_name_sort_key(value: &str) -> Vec<u8> {
    parse_module_name_wire(value)
        .and_then(|name| phase5_name_canonical_bytes(&name))
        .unwrap_or_else(|_| value.as_bytes().to_vec())
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

fn phase8_encode_uvar(mut value: u64) -> Vec<u8> {
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

    fn valid_import_lock_manifest_json() -> String {
        format!(
            r#"{{
              "schema":"npa.phase8.import_lock_manifest.v1",
              "imports":[
                {{
                  "module":"Std.Nat",
                  "export_hash":"{}",
                  "certificate":{{
                    "kind":"path",
                    "path":"build/certs/Std/Nat.npcert",
                    "file_hash":"{}",
                    "certificate_hash":"{}"
                  }}
                }},
                {{
                  "module":"Std.Logic",
                  "export_hash":"{}",
                  "certificate":{{
                    "kind":"path",
                    "path":"build/certs/Std/Logic.npcert",
                    "file_hash":"{}",
                    "certificate_hash":"{}"
                  }}
                }}
              ]
            }}"#,
            hash_wire(50),
            hash_wire(51),
            hash_wire(52),
            hash_wire(53),
            hash_wire(54),
            hash_wire(55)
        )
    }

    fn test_raw_certificate_bytes(module: &str, certificate_hash: Hash) -> Vec<u8> {
        fn push_string(out: &mut Vec<u8>, value: &str) {
            out.extend(phase8_encode_uvar(value.len() as u64));
            out.extend(value.as_bytes());
        }

        let mut out = Vec::new();
        push_string(&mut out, "NPA-CERT-0.1");
        push_string(&mut out, "NPA-Core-0.1");
        let components = module.split('.').collect::<Vec<_>>();
        out.extend(phase8_encode_uvar(components.len() as u64));
        for component in components {
            push_string(&mut out, component);
        }
        out.extend(b"opaque certificate payload not decoded by phase8 request materialize");
        out.extend(certificate_hash);
        out
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
        command.field = Some("request_output_dir/reference.json".into());
        command.expected_hash = Some(Box::new(test_hash(3)));
        command.actual_hash = Some(Box::new(test_hash(4)));
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

    #[test]
    fn m2_import_lock_manifest_validates_sorted_unique_identity_fields() {
        let manifest =
            parse_phase8_import_lock_manifest(&valid_import_lock_manifest_json()).unwrap();
        assert_eq!(manifest.imports.len(), 2);
        assert_eq!(manifest.imports[0].module, "Std.Nat");
        assert_eq!(
            manifest.imports[1].certificate.path,
            "build/certs/Std/Logic.npcert"
        );

        let unsorted = parse_phase8_import_lock_manifest(
            &valid_import_lock_manifest_json().replace("Std.Nat", "Std.Zzzzz"),
        )
        .unwrap_err();
        assert_eq!(unsorted.field.as_ref(), "imports.manifest.imports[1]");
        assert_eq!(unsorted.actual_value.as_deref(), Some("order_violation"));

        let bad_module = parse_phase8_import_lock_manifest(
            &valid_import_lock_manifest_json().replace("Std.Nat", "Std..Nat"),
        )
        .unwrap_err();
        assert_eq!(
            bad_module.field.as_ref(),
            "imports.manifest.imports[0].module"
        );
        assert_eq!(
            bad_module.actual_value.as_deref(),
            Some("invalid_name_format")
        );

        let duplicate_module_source = valid_import_lock_manifest_json()
            .replace(r#""module":"Std.Logic""#, r#""module":"Std.Nat""#)
            .replace("build/certs/Std/Logic.npcert", "build/certs/Std/Zzz.npcert");
        let duplicate_module =
            parse_phase8_import_lock_manifest(&duplicate_module_source).unwrap_err();
        assert_eq!(
            duplicate_module.field.as_ref(),
            "imports.manifest.imports[1].module"
        );
        assert_eq!(
            duplicate_module.actual_value.as_deref(),
            Some("duplicate_module")
        );

        let duplicate_path = parse_phase8_import_lock_manifest(
            &valid_import_lock_manifest_json()
                .replace("build/certs/Std/Nat.npcert", "build/certs/Std/Logic.npcert"),
        )
        .unwrap_err();
        assert_eq!(
            duplicate_path.field.as_ref(),
            "imports.manifest.imports[1].certificate.path"
        );
        assert_eq!(
            duplicate_path.actual_value.as_deref(),
            Some("duplicate_path")
        );

        let duplicate_certificate_hash = parse_phase8_import_lock_manifest(
            &valid_import_lock_manifest_json().replace(&hash_wire(55), &hash_wire(52)),
        )
        .unwrap_err();
        assert_eq!(
            duplicate_certificate_hash.field.as_ref(),
            "imports.manifest.imports[1].certificate.certificate_hash"
        );
        assert_eq!(
            duplicate_certificate_hash.actual_value.as_deref(),
            Some("duplicate_certificate_hash")
        );

        let duplicate_file_hash = parse_phase8_import_lock_manifest(
            &valid_import_lock_manifest_json().replace(&hash_wire(54), &hash_wire(51)),
        )
        .unwrap_err();
        assert_eq!(
            duplicate_file_hash.field.as_ref(),
            "imports.manifest.imports[1].certificate.file_hash"
        );
        assert_eq!(
            duplicate_file_hash.actual_value.as_deref(),
            Some("duplicate_file_hash")
        );

        let bad_kind = parse_phase8_import_lock_manifest(
            &valid_import_lock_manifest_json().replace(r#""kind":"path""#, r#""kind":"url""#),
        )
        .unwrap_err();
        assert_eq!(
            bad_kind.field.as_ref(),
            "imports.manifest.imports[0].certificate.kind"
        );
        assert_eq!(bad_kind.actual_value.as_deref(), Some("invalid_enum"));
    }

    #[test]
    fn m2_request_hash_is_semantic_and_file_hash_tracks_exact_bytes() {
        let policy = parse_phase8_runner_policy(&valid_runner_policy_json()).unwrap();
        let imports_json = valid_import_lock_manifest_json();
        let imports_hash = phase8_file_hash(imports_json.as_bytes());
        let cert_bytes = test_raw_certificate_bytes("Std.Nat", test_hash(70));

        let first = phase8_request_materialize(
            &policy,
            "Std.Nat",
            "build/certs/Std/Nat.npcert",
            &cert_bytes,
            "build/certs/import-lock.json",
            imports_json.as_bytes(),
            imports_hash,
            "reference",
            "mchkreq_001",
            "build/check-requests/Std.Nat.reference.json",
            None,
        )
        .unwrap();
        let second = phase8_request_materialize(
            &policy,
            "Std.Nat",
            "build/certs/Std/Nat.npcert",
            &cert_bytes,
            "build/certs/import-lock.json",
            imports_json.as_bytes(),
            imports_hash,
            "reference",
            "mchkreq_002",
            "build/check-requests/Std.Nat.reference.json",
            None,
        )
        .unwrap();

        assert_eq!(first.request.request_hash(), second.request.request_hash());
        assert_ne!(first.request_file_hash, second.request_file_hash);
        assert_eq!(
            first.request.certificate.file_hash,
            phase8_file_hash(&cert_bytes)
        );
        assert_eq!(
            first.request.certificate.expected_certificate_hash,
            test_hash(70)
        );
        assert_eq!(first.request.imports.manifest_hash, imports_hash);
        assert_eq!(
            parse_phase8_machine_check_request(&first.request.canonical_json())
                .unwrap()
                .request_hash(),
            first.request.request_hash()
        );
    }

    #[test]
    fn m2_materialize_rejects_import_hash_and_raw_certificate_claim_failures() {
        let policy = parse_phase8_runner_policy(&valid_runner_policy_json()).unwrap();
        let imports_json = valid_import_lock_manifest_json();
        let cert_bytes = test_raw_certificate_bytes("Std.Nat", test_hash(70));

        let import_hash_err = phase8_request_materialize(
            &policy,
            "Std.Nat",
            "build/certs/Std/Nat.npcert",
            &cert_bytes,
            "build/certs/import-lock.json",
            imports_json.as_bytes(),
            test_hash(99),
            "reference",
            "mchkreq_001",
            "build/check-requests/Std.Nat.reference.json",
            None,
        )
        .unwrap_err();
        assert_eq!(import_hash_err.reason_code.as_ref(), "input_hash_mismatch");
        assert_eq!(
            import_hash_err.field.as_deref(),
            Some("imports.manifest_hash")
        );

        let malformed_cert = b"too short".to_vec();
        let cert_err = phase8_request_materialize(
            &policy,
            "Std.Nat",
            "build/certs/Std/Nat.npcert",
            &malformed_cert,
            "build/certs/import-lock.json",
            imports_json.as_bytes(),
            phase8_file_hash(imports_json.as_bytes()),
            "reference",
            "mchkreq_001",
            "build/check-requests/Std.Nat.reference.json",
            None,
        )
        .unwrap_err();
        assert_eq!(
            cert_err.reason_code.as_ref(),
            "request_materialization_failed"
        );
        assert_eq!(
            cert_err.field.as_deref(),
            Some("certificate.expected_certificate_hash")
        );

        let module_err = phase8_request_materialize(
            &policy,
            "Std.Bool",
            "build/certs/Std/Nat.npcert",
            &cert_bytes,
            "build/certs/import-lock.json",
            imports_json.as_bytes(),
            phase8_file_hash(imports_json.as_bytes()),
            "reference",
            "mchkreq_001",
            "build/check-requests/Std.Nat.reference.json",
            None,
        )
        .unwrap_err();
        assert_eq!(
            module_err.reason_code.as_ref(),
            "request_materialization_failed"
        );
        assert_eq!(module_err.field.as_deref(), Some("module"));
        assert_eq!(module_err.actual_value.as_deref(), Some("Std.Nat"));
    }

    #[test]
    fn m2_request_store_exact_match_retry_and_conflict_are_deterministic() {
        let generated = Phase8RequestStoreEntry {
            request_hash: test_hash(80),
            path: "build/check-requests/Std.Nat.reference.json".to_owned(),
            file_hash: test_hash(81),
        };
        let existing = Phase8RequestStoreManifest {
            requests: vec![generated.clone()],
        };
        let idempotent =
            phase8_request_store_with_materialized_entry(Some(&existing), generated.clone())
                .unwrap();
        assert!(!idempotent.rewrite_required);
        assert_eq!(idempotent.manifest, existing);

        let conflict = Phase8RequestStoreManifest {
            requests: vec![Phase8RequestStoreEntry {
                request_hash: generated.request_hash,
                path: generated.path.clone(),
                file_hash: test_hash(82),
            }],
        };
        let err =
            phase8_request_store_with_materialized_entry(Some(&conflict), generated).unwrap_err();
        assert_eq!(err.reason_code.as_ref(), "request_store_entry_conflict");
        assert_eq!(err.field.as_deref(), Some("request_store.requests[]"));
        assert!(err
            .expected_value
            .as_deref()
            .unwrap()
            .contains("\"file_hash\":\"sha256:"));
        assert_ne!(err.expected_value, err.actual_value);
    }

    fn m3_request_and_policy() -> (Phase8MachineCheckRequest, Phase8RunnerPolicy) {
        let policy = parse_phase8_runner_policy(&valid_runner_policy_json()).unwrap();
        let imports_json = valid_import_lock_manifest_json();
        let cert_bytes = test_raw_certificate_bytes("Std.Nat", test_hash(70));
        let materialized = phase8_request_materialize(
            &policy,
            "Std.Nat",
            "build/certs/Std/Nat.npcert",
            &cert_bytes,
            "build/certs/import-lock.json",
            imports_json.as_bytes(),
            phase8_file_hash(imports_json.as_bytes()),
            "reference",
            "mchkreq_001",
            "build/check-requests/Std.Nat.reference.json",
            None,
        )
        .unwrap();
        (materialized.request, policy)
    }

    fn m3_runner() -> Phase8MachineCheckRunner {
        Phase8MachineCheckRunner {
            id: "npa-check-runner".to_owned(),
            version: "0.8.0".to_owned(),
            build_hash: test_hash(20),
        }
    }

    fn m3_resource_usage(elapsed_ms: u64) -> Phase8MachineCheckResourceUsage {
        Phase8MachineCheckResourceUsage {
            steps: 128,
            memory_peak_mb: 64,
            elapsed_ms,
        }
    }

    fn m3_raw_checked(version: &str) -> String {
        format!(
            r#"{{
              "schema":"npa.phase8.checker_raw_result.v1",
              "checker_id":"npa-checker-ref",
              "checker_version":"{}",
              "checker_build_hash":"{}",
              "status":"checked",
              "module":"Std.Nat",
              "certificate_hash":"{}",
              "export_hash":"{}",
              "axiom_report_hash":"{}"
            }}"#,
            version,
            hash_wire(11),
            hash_wire(70),
            hash_wire(90),
            hash_wire(91)
        )
    }

    #[test]
    fn m3_adopts_checked_raw_result_and_computes_distinct_hashes() {
        let (request, policy) = m3_request_and_policy();
        let result = phase8_machine_check_run(
            &request,
            &policy,
            Phase8CheckerRunObservation {
                result_id: "mchkres_001".to_owned(),
                attempt: 1,
                runner: m3_runner(),
                process: Phase8MachineCheckProcess::exited(0),
                resource_usage: m3_resource_usage(1732),
                stdout: m3_raw_checked("0.8.0").into_bytes(),
                stderr: Vec::new(),
            },
        )
        .unwrap();

        assert_eq!(result.status, Phase8MachineCheckStatus::Checked);
        assert_eq!(result.error, None);
        assert_eq!(result.certificate_hash, Some(test_hash(70)));
        assert_eq!(result.export_hash, Some(test_hash(90)));
        assert_eq!(result.axiom_report_hash, Some(test_hash(91)));
        assert_eq!(result.checker.id.as_deref(), Some("npa-checker-ref"));
        assert_eq!(result.checker.build_hash, Some(test_hash(11)));
        assert_ne!(result.result_hash(), result.run_artifact_hash());
        assert!(result
            .canonical_json()
            .contains(PHASE8_MACHINE_CHECK_RESULT_SCHEMA));
    }

    #[test]
    fn m3_result_hash_excludes_checker_version_process_and_diagnostics() {
        let (request, policy) = m3_request_and_policy();
        let first = phase8_machine_check_run(
            &request,
            &policy,
            Phase8CheckerRunObservation {
                result_id: "mchkres_001".to_owned(),
                attempt: 1,
                runner: m3_runner(),
                process: Phase8MachineCheckProcess::exited(0),
                resource_usage: m3_resource_usage(100),
                stdout: m3_raw_checked("0.8.0").into_bytes(),
                stderr: Vec::new(),
            },
        )
        .unwrap();
        let second = phase8_machine_check_run(
            &request,
            &policy,
            Phase8CheckerRunObservation {
                result_id: "mchkres_002".to_owned(),
                attempt: 1,
                runner: m3_runner(),
                process: Phase8MachineCheckProcess::exited(0),
                resource_usage: m3_resource_usage(200),
                stdout: m3_raw_checked("0.8.1").into_bytes(),
                stderr: b"free form stderr".to_vec(),
            },
        )
        .unwrap();

        assert_eq!(first.result_hash(), second.result_hash());
        assert_ne!(first.run_artifact_hash(), second.run_artifact_hash());
        assert_eq!(
            second.diagnostics,
            vec!["checker_process:stderr_present".to_owned()]
        );
        assert!(!second.canonical_json().contains("free form stderr"));
    }

    #[test]
    fn m3_malformed_raw_output_is_saved_as_checker_internal_error() {
        let (request, policy) = m3_request_and_policy();
        let result = phase8_machine_check_run(
            &request,
            &policy,
            Phase8CheckerRunObservation {
                result_id: "mchkres_bad".to_owned(),
                attempt: 1,
                runner: m3_runner(),
                process: Phase8MachineCheckProcess::exited(0),
                resource_usage: m3_resource_usage(1),
                stdout: b"not json raw stdout text".to_vec(),
                stderr: b"raw stderr text".to_vec(),
            },
        )
        .unwrap();

        let error = result.error.as_ref().unwrap();
        assert_eq!(result.status, Phase8MachineCheckStatus::Failed);
        assert_eq!(error.kind, "checker_internal_error");
        assert_eq!(
            error.reason_code.as_deref(),
            Some("malformed_success_output")
        );
        assert_eq!(error.field.as_deref(), Some("checker_raw"));
        assert_eq!(error.actual_value.as_deref(), Some("invalid_json"));
        assert_eq!(
            result.diagnostics,
            vec![
                "checker_process:stderr_present".to_owned(),
                "checker_process:stdout_present".to_owned(),
            ]
        );
        let canonical = result.canonical_json();
        assert!(!canonical.contains("not json raw stdout text"));
        assert!(!canonical.contains("raw stderr text"));
    }

    #[test]
    fn m3_malformed_identity_fields_are_raw_schema_errors() {
        let (request, policy) = m3_request_and_policy();
        let raw = format!(
            r#"{{
              "schema":"npa.phase8.checker_raw_result.v1",
              "checker_id":"npa-checker-ref",
              "checker_id":"npa-checker-ref",
              "checker_version":"0.8.0",
              "checker_build_hash":"{}",
              "status":"checked",
              "module":"Std.Nat",
              "certificate_hash":"{}",
              "export_hash":"{}",
              "axiom_report_hash":"{}"
            }}"#,
            hash_wire(11),
            hash_wire(70),
            hash_wire(90),
            hash_wire(91)
        );
        let result = phase8_machine_check_run(
            &request,
            &policy,
            Phase8CheckerRunObservation {
                result_id: "mchkres_dup_identity".to_owned(),
                attempt: 1,
                runner: m3_runner(),
                process: Phase8MachineCheckProcess::exited(0),
                resource_usage: m3_resource_usage(1),
                stdout: raw.into_bytes(),
                stderr: Vec::new(),
            },
        )
        .unwrap();

        let error = result.error.as_ref().unwrap();
        assert_eq!(result.status, Phase8MachineCheckStatus::Failed);
        assert_eq!(error.kind, "checker_internal_error");
        assert_eq!(
            error.reason_code.as_deref(),
            Some("malformed_success_output")
        );
        assert_eq!(error.field.as_deref(), Some("checker_raw.checker_id"));
        assert_eq!(error.expected_value.as_deref(), Some("unique_object_keys"));
        assert_eq!(error.actual_value.as_deref(), Some("duplicate_field"));

        let raw = format!(
            r#"{{
              "schema":"npa.phase8.checker_raw_result.v1",
              "checker_id":"npa-checker-ref",
              "checker_version":"0.8.0",
              "checker_build_hash":"{}",
              "status":"failed",
              "module":"Std.Nat",
              "certificate_hash":"{}",
              "error":{{
                "kind":"type_mismatch",
                "reason_code":"checker_reported_internal_error"
              }}
            }}"#,
            hash_wire(11),
            hash_wire(70)
        );
        let error = parse_phase8_checker_raw_result(&raw).unwrap_err();
        assert_eq!(error.field, "checker_raw.error.reason_code");
        assert_eq!(error.expected_value, "absent_for_error_kind");
        assert_eq!(error.actual_value, "forbidden_field");
    }

    #[test]
    fn m3_adopts_checker_internal_error_payload() {
        let (request, policy) = m3_request_and_policy();
        let raw = format!(
            r#"{{
              "schema":"npa.phase8.checker_raw_result.v1",
              "checker_id":"npa-checker-ref",
              "checker_version":"0.8.0",
              "checker_build_hash":"{}",
              "status":"failed",
              "error":{{
                "kind":"checker_internal_error",
                "reason_code":"checker_reported_internal_error",
                "core_path":["decl","body"],
                "section":"type",
                "offset":3
              }}
            }}"#,
            hash_wire(11)
        );
        let result = phase8_machine_check_run(
            &request,
            &policy,
            Phase8CheckerRunObservation {
                result_id: "mchkres_internal".to_owned(),
                attempt: 1,
                runner: m3_runner(),
                process: Phase8MachineCheckProcess::exited(2),
                resource_usage: m3_resource_usage(1),
                stdout: raw.into_bytes(),
                stderr: Vec::new(),
            },
        )
        .unwrap();

        let error = result.error.as_ref().unwrap();
        assert_eq!(result.status, Phase8MachineCheckStatus::Failed);
        assert_eq!(error.kind, "checker_internal_error");
        assert_eq!(
            error.reason_code.as_deref(),
            Some("checker_reported_internal_error")
        );
        assert_eq!(
            error.core_path,
            Some(vec!["decl".to_owned(), "body".to_owned()])
        );
        assert_eq!(error.section.as_deref(), Some("type"));
        assert_eq!(error.offset, Some(3));
    }

    #[test]
    fn m3_checked_status_requires_launched_process() {
        let (request, policy) = m3_request_and_policy();
        let result = phase8_machine_check_run(
            &request,
            &policy,
            Phase8CheckerRunObservation {
                result_id: "mchkres_not_launched".to_owned(),
                attempt: 1,
                runner: m3_runner(),
                process: Phase8MachineCheckProcess {
                    launched: false,
                    exit_code: Some(0),
                    termination_reason: None,
                },
                resource_usage: Phase8MachineCheckResourceUsage::zero(),
                stdout: m3_raw_checked("0.8.0").into_bytes(),
                stderr: Vec::new(),
            },
        )
        .unwrap();

        let error = result.error.as_ref().unwrap();
        assert_eq!(result.status, Phase8MachineCheckStatus::Failed);
        assert_eq!(error.kind, "checker_internal_error");
        assert_eq!(error.reason_code.as_deref(), Some("process_not_launched"));
        assert_eq!(result.checker.id, None);
    }

    #[test]
    fn m3_policy_identity_gate_prevents_raw_checked_status_adoption() {
        let (request, policy) = m3_request_and_policy();
        let mismatched_raw =
            m3_raw_checked("0.8.0").replace("npa-checker-ref", "npa-checker-other");
        let result = phase8_machine_check_run(
            &request,
            &policy,
            Phase8CheckerRunObservation {
                result_id: "mchkres_identity".to_owned(),
                attempt: 1,
                runner: m3_runner(),
                process: Phase8MachineCheckProcess::exited(0),
                resource_usage: m3_resource_usage(10),
                stdout: mismatched_raw.into_bytes(),
                stderr: Vec::new(),
            },
        )
        .unwrap();

        let error = result.error.as_ref().unwrap();
        assert_eq!(result.status, Phase8MachineCheckStatus::Failed);
        assert_eq!(error.kind, "policy_failure");
        assert_eq!(
            error.reason_code.as_deref(),
            Some("checker_identity_mismatch")
        );
        assert_eq!(error.field.as_deref(), Some("checker.id"));
        assert_eq!(result.certificate_hash, None);
    }

    #[test]
    fn m3_machine_result_store_uses_run_artifact_hash_not_result_hash() {
        let (request, policy) = m3_request_and_policy();
        let first = phase8_machine_check_run(
            &request,
            &policy,
            Phase8CheckerRunObservation {
                result_id: "mchkres_001".to_owned(),
                attempt: 1,
                runner: m3_runner(),
                process: Phase8MachineCheckProcess::exited(0),
                resource_usage: m3_resource_usage(100),
                stdout: m3_raw_checked("0.8.0").into_bytes(),
                stderr: Vec::new(),
            },
        )
        .unwrap();
        let second = phase8_machine_check_run(
            &request,
            &policy,
            Phase8CheckerRunObservation {
                result_id: "mchkres_002".to_owned(),
                attempt: 2,
                runner: m3_runner(),
                process: Phase8MachineCheckProcess::exited(0),
                resource_usage: m3_resource_usage(101),
                stdout: m3_raw_checked("0.8.0").into_bytes(),
                stderr: Vec::new(),
            },
        )
        .unwrap();
        assert_eq!(first.result_hash(), second.result_hash());
        assert_ne!(first.run_artifact_hash(), second.run_artifact_hash());

        let first_entry =
            phase8_machine_result_store_entry_for_result(&first, "build/results/first.json")
                .unwrap();
        let second_entry =
            phase8_machine_result_store_entry_for_result(&second, "build/results/second.json")
                .unwrap();
        let update = phase8_machine_result_store_with_entry(None, first_entry.clone()).unwrap();
        let update =
            phase8_machine_result_store_with_entry(Some(&update.manifest), second_entry).unwrap();
        assert_eq!(update.manifest.results.len(), 2);
        assert_eq!(
            parse_phase8_machine_result_store_manifest(&update.manifest.canonical_json()).unwrap(),
            update.manifest
        );

        let mut conflicting_entry = first_entry;
        conflicting_entry.file_hash = test_hash(200);
        let conflict =
            phase8_machine_result_store_with_entry(Some(&update.manifest), conflicting_entry)
                .unwrap_err();
        assert_eq!(conflict.reason_code.as_ref(), "result_store_entry_conflict");
    }

    #[test]
    fn m3_axiom_report_and_store_hashes_are_deterministic() {
        let report = Phase8AxiomReport {
            module: "Std.Nat".to_owned(),
            certificate_hash: test_hash(70),
            axioms: vec![
                Phase8AxiomReportEntry {
                    name: "Aaa.choice".to_owned(),
                },
                Phase8AxiomReportEntry {
                    name: "Bbb.propExt".to_owned(),
                },
            ],
        };
        let parsed = parse_phase8_axiom_report(&report.canonical_json()).unwrap();
        assert_eq!(parsed, report);

        let bad_order = report
            .canonical_json()
            .replace("Aaa.choice", "Zzz.choice")
            .replace("Bbb.propExt", "Aaa.propExt");
        let bad_order = parse_phase8_axiom_report(&bad_order).unwrap_err();
        assert_eq!(bad_order.field.as_ref(), "axioms[1]");
        assert_eq!(bad_order.actual_value.as_deref(), Some("order_violation"));

        let entry = Phase8AxiomReportStoreEntry {
            axiom_report_hash: report.axiom_report_hash(),
            path: "build/axiom-reports/Std.Nat.json".to_owned(),
            file_hash: phase8_file_hash(report.canonical_json().as_bytes()),
        };
        let store = Phase8AxiomReportStoreManifest {
            reports: vec![entry],
        };
        assert_eq!(
            parse_phase8_axiom_report_store_manifest(&store.canonical_json()).unwrap(),
            store
        );
    }

    fn m4_runner_policy_json() -> String {
        format!(
            r#"{{
              "schema":"npa.phase8.runner_policy.v1",
              "id":"phase8-release",
              "version":1,
              "trust_mode":"release",
              "required_checker_profiles":["fast-kernel","reference","external"],
              "optional_checker_profiles":[],
              "checker_allowlist":[
                {{
                  "profile":"external",
                  "checker_id":"npa-checker-ext",
                  "binary_id":"npa-checker-ext-macos-aarch64",
                  "binary_hash":"{}",
                  "build_hash":"{}",
                  "allowed_args":["--json","--canonical-only"]
                }},
                {{
                  "profile":"fast-kernel",
                  "checker_id":"npa-fast-kernel",
                  "binary_id":"npa-fast-kernel-macos-aarch64",
                  "binary_hash":"{}",
                  "build_hash":"{}",
                  "allowed_args":["--json","--canonical-only"]
                }},
                {{
                  "profile":"reference",
                  "checker_id":"npa-checker-ref",
                  "binary_id":"npa-checker-ref-macos-aarch64",
                  "binary_hash":"{}",
                  "build_hash":"{}",
                  "allowed_args":["--json","--canonical-only"]
                }}
              ],
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
                "external":{{"max_steps":10000000,"max_memory_mb":2048,"timeout_ms":60000}},
                "fast-kernel":{{"max_steps":10000000,"max_memory_mb":2048,"timeout_ms":60000}},
                "reference":{{"max_steps":10000000,"max_memory_mb":2048,"timeout_ms":60000}}
              }},
              "on_resource_exhausted":"fail",
              "on_missing_required_checker":"fail",
              "on_profile_requested_by_ai":"ignore_unless_policy_allows"
            }}"#,
            hash_wire(30),
            hash_wire(31),
            hash_wire(32),
            hash_wire(33),
            hash_wire(10),
            hash_wire(11),
            hash_wire(12),
            hash_wire(13)
        )
    }

    fn m4_stored_request(
        policy: &Phase8RunnerPolicy,
        profile: &str,
        request_id: &str,
    ) -> Phase8StoredMachineCheckRequest {
        let imports_json = valid_import_lock_manifest_json();
        let cert_bytes = test_raw_certificate_bytes("Std.Nat", test_hash(70));
        let path = format!("build/check-requests/Std.Nat.{profile}.json");
        let materialized = phase8_request_materialize(
            policy,
            "Std.Nat",
            "build/certs/Std/Nat.npcert",
            &cert_bytes,
            "build/certs/import-lock.json",
            imports_json.as_bytes(),
            phase8_file_hash(imports_json.as_bytes()),
            profile,
            request_id,
            &path,
            None,
        )
        .unwrap();
        Phase8StoredMachineCheckRequest {
            path,
            file_hash: materialized.request_file_hash,
            request: materialized.request,
        }
    }

    fn m4_request_store_manifest(
        requests: &[Phase8StoredMachineCheckRequest],
    ) -> Phase8RequestStoreManifest {
        let mut entries = requests
            .iter()
            .map(|stored| Phase8RequestStoreEntry {
                request_hash: stored.request.request_hash(),
                path: stored.path.clone(),
                file_hash: stored.file_hash,
            })
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| left.request_hash.cmp(&right.request_hash));
        Phase8RequestStoreManifest { requests: entries }
    }

    fn m4_raw_checked(checker_id: &str, build_hash: Hash) -> String {
        format!(
            r#"{{
              "schema":"npa.phase8.checker_raw_result.v1",
              "checker_id":"{}",
              "checker_version":"0.8.0",
              "checker_build_hash":"{}",
              "status":"checked",
              "module":"Std.Nat",
              "certificate_hash":"{}",
              "export_hash":"{}",
              "axiom_report_hash":"{}"
            }}"#,
            checker_id,
            format_hash_string(&build_hash),
            hash_wire(70),
            hash_wire(90),
            hash_wire(91)
        )
    }

    fn m4_checked_result(
        request: &Phase8MachineCheckRequest,
        policy: &Phase8RunnerPolicy,
        result_id: &str,
    ) -> Phase8MachineCheckResult {
        let checker = policy
            .selected_checker_policy(&request.checker_profile)
            .unwrap();
        phase8_machine_check_run(
            request,
            policy,
            Phase8CheckerRunObservation {
                result_id: result_id.to_owned(),
                attempt: 1,
                runner: m3_runner(),
                process: Phase8MachineCheckProcess::exited(0),
                resource_usage: m3_resource_usage(100),
                stdout: m4_raw_checked(&checker.checker_id, checker.build_hash).into_bytes(),
                stderr: Vec::new(),
            },
        )
        .unwrap()
    }

    #[test]
    fn m4_normalizes_results_in_policy_order_and_copies_source_fields() {
        let policy = parse_phase8_runner_policy(&m4_runner_policy_json()).unwrap();
        let fast = m4_stored_request(&policy, "fast-kernel", "mchkreq_fast");
        let reference = m4_stored_request(&policy, "reference", "mchkreq_ref");
        let external = m4_stored_request(&policy, "external", "mchkreq_ext");
        let stored_requests = vec![fast.clone(), reference.clone(), external.clone()];
        let request_store = m4_request_store_manifest(&stored_requests);

        let external_result = m4_checked_result(&external.request, &policy, "mchkres_ext");
        let reference_result = m4_checked_result(&reference.request, &policy, "mchkres_ref");
        let fast_result = m4_checked_result(&fast.request, &policy, "mchkres_fast");
        let normalized = phase8_normalize_results(
            "norm_Std.Nat_001",
            "normerr_Std.Nat_001",
            &policy,
            &request_store,
            &stored_requests,
            &[
                external_result.clone(),
                reference_result.clone(),
                fast_result.clone(),
            ],
            None,
        )
        .unwrap();

        assert_eq!(
            normalized
                .results
                .iter()
                .map(|entry| entry.checker_profile.as_str())
                .collect::<Vec<_>>(),
            vec!["fast-kernel", "reference", "external"]
        );
        let fast_entry = &normalized.results[0];
        assert_eq!(fast_entry.result_id, fast_result.result_id);
        assert_eq!(fast_entry.result_hash, fast_result.result_hash());
        assert_eq!(fast_entry.request_hash, fast_result.request_hash);
        assert_eq!(fast_entry.policy_hash, fast_result.policy.hash);
        assert_eq!(fast_entry.process_launched, fast_result.process.launched);
        assert_eq!(fast_entry.checker_binary_id, fast_result.checker.binary_id);
        assert_eq!(
            fast_entry.checker_binary_hash,
            fast_result.checker.binary_hash
        );
        assert_eq!(fast_entry.checker_id, fast_result.checker.id);
        assert_eq!(
            fast_entry.checker_build_hash,
            fast_result.checker.build_hash
        );
        assert_eq!(fast_entry.certificate_hash, fast_result.certificate_hash);
        assert_eq!(fast_entry.export_hash, fast_result.export_hash);
        assert_eq!(fast_entry.axiom_report_hash, fast_result.axiom_report_hash);
        assert_eq!(
            normalized.comparison.status,
            Phase8NormalizedComparisonStatus::AllAgreeChecked
        );
    }

    #[test]
    fn m4_normalized_hash_excludes_result_ids() {
        let (request, policy) = m3_request_and_policy();
        let stored = Phase8StoredMachineCheckRequest {
            path: "build/check-requests/Std.Nat.reference.json".to_owned(),
            file_hash: phase8_file_hash(request.canonical_json().as_bytes()),
            request: request.clone(),
        };
        let request_store = m4_request_store_manifest(std::slice::from_ref(&stored));
        let result = m4_checked_result(&request, &policy, "mchkres_001");
        let first = phase8_normalize_results(
            "norm_Std.Nat_001",
            "normerr_Std.Nat_001",
            &policy,
            &request_store,
            std::slice::from_ref(&stored),
            std::slice::from_ref(&result),
            None,
        )
        .unwrap();
        let mut second = first.clone();
        second.normalized_result_id = "norm_Std.Nat_retry".to_owned();
        second.results[0].result_id = "mchkres_retry".to_owned();

        assert_eq!(
            first.normalized_result_hash(),
            second.normalized_result_hash()
        );
        assert_ne!(first.canonical_json(), second.canonical_json());
        assert!(!first
            .hash_input_canonical_json()
            .contains("norm_Std.Nat_001"));
        assert!(!first.hash_input_canonical_json().contains("mchkres_001"));
    }

    #[test]
    fn m4_failed_entries_derive_failure_key_from_error() {
        let (request, policy) = m3_request_and_policy();
        let stored = Phase8StoredMachineCheckRequest {
            path: "build/check-requests/Std.Nat.reference.json".to_owned(),
            file_hash: phase8_file_hash(request.canonical_json().as_bytes()),
            request: request.clone(),
        };
        let request_store = m4_request_store_manifest(std::slice::from_ref(&stored));
        let raw = format!(
            r#"{{
              "schema":"npa.phase8.checker_raw_result.v1",
              "checker_id":"npa-checker-ref",
              "checker_version":"0.8.0",
              "checker_build_hash":"{}",
              "status":"failed",
              "module":"Std.Nat",
              "certificate_hash":"{}",
              "error":{{
                "kind":"type_mismatch",
                "declaration":"Std.Nat"
              }}
            }}"#,
            hash_wire(11),
            hash_wire(70)
        );
        let failed = phase8_machine_check_run(
            &request,
            &policy,
            Phase8CheckerRunObservation {
                result_id: "mchkres_failed".to_owned(),
                attempt: 1,
                runner: m3_runner(),
                process: Phase8MachineCheckProcess::exited(1),
                resource_usage: m3_resource_usage(100),
                stdout: raw.into_bytes(),
                stderr: Vec::new(),
            },
        )
        .unwrap();
        let normalized = phase8_normalize_results(
            "norm_Std.Nat_failed",
            "normerr_Std.Nat_failed",
            &policy,
            &request_store,
            std::slice::from_ref(&stored),
            std::slice::from_ref(&failed),
            None,
        )
        .unwrap();

        let expected = Phase8NormalizedFailureKey::from_error(failed.error.as_ref().unwrap());
        assert_eq!(normalized.results[0].failure_key, Some(expected));
        assert_eq!(
            normalized.comparison.status,
            Phase8NormalizedComparisonStatus::AllAgreeFailed
        );
    }

    #[test]
    fn m4_normalizer_failures_do_not_create_partial_artifacts() {
        let (request, policy) = m3_request_and_policy();
        let stored = Phase8StoredMachineCheckRequest {
            path: "build/check-requests/Std.Nat.reference.json".to_owned(),
            file_hash: phase8_file_hash(request.canonical_json().as_bytes()),
            request: request.clone(),
        };
        let request_store = m4_request_store_manifest(std::slice::from_ref(&stored));
        let first = m4_checked_result(&request, &policy, "mchkres_001");
        let second = m4_checked_result(&request, &policy, "mchkres_002");
        let err = phase8_normalize_results(
            "norm_Std.Nat_dup",
            "normerr_Std.Nat_dup",
            &policy,
            &request_store,
            std::slice::from_ref(&stored),
            &[first, second],
            None,
        )
        .unwrap_err();

        assert_eq!(
            err.error().reason_code,
            Phase8NormalizeErrorReasonCode::DuplicateCheckerProfileResult.as_str()
        );
        assert_eq!(err.error().field, "machine_results[].checker.profile");
        assert!(err
            .canonical_json()
            .contains(PHASE8_NORMALIZE_ERROR_RESULT_SCHEMA));
    }

    #[test]
    fn m4_normalizer_revalidates_request_store_manifest_domain() {
        let (request, policy) = m3_request_and_policy();
        let stored = Phase8StoredMachineCheckRequest {
            path: "build/check-requests/Std.Nat.reference.json".to_owned(),
            file_hash: phase8_file_hash(request.canonical_json().as_bytes()),
            request: request.clone(),
        };
        let mut request_store = m4_request_store_manifest(std::slice::from_ref(&stored));
        request_store
            .requests
            .push(request_store.requests[0].clone());
        let result = m4_checked_result(&request, &policy, "mchkres_001");
        let err = phase8_normalize_results(
            "norm_Std.Nat_store_bad",
            "normerr_Std.Nat_store_bad",
            &policy,
            &request_store,
            std::slice::from_ref(&stored),
            std::slice::from_ref(&result),
            None,
        )
        .unwrap_err();

        assert_eq!(
            err.error().reason_code,
            Phase8NormalizeErrorReasonCode::RequestStoreManifestInvalid.as_str()
        );
        assert_eq!(
            err.error().actual_value.as_deref(),
            Some("duplicate_request_hash")
        );
    }

    #[test]
    fn m4_normalized_store_and_write_result_are_deterministic() {
        let (request, policy) = m3_request_and_policy();
        let stored = Phase8StoredMachineCheckRequest {
            path: "build/check-requests/Std.Nat.reference.json".to_owned(),
            file_hash: phase8_file_hash(request.canonical_json().as_bytes()),
            request: request.clone(),
        };
        let request_store = m4_request_store_manifest(std::slice::from_ref(&stored));
        let result = m4_checked_result(&request, &policy, "mchkres_001");
        let normalized = phase8_normalize_results(
            "norm_Std.Nat_001",
            "normerr_Std.Nat_001",
            &policy,
            &request_store,
            std::slice::from_ref(&stored),
            std::slice::from_ref(&result),
            None,
        )
        .unwrap();
        let entry = phase8_normalized_result_store_entry_for_result(
            &normalized,
            "build/normalized/Std.Nat.json",
        )
        .unwrap();
        let update = phase8_normalized_result_store_with_entry(None, entry.clone()).unwrap();
        assert_eq!(
            parse_phase8_normalized_result_store_manifest(&update.manifest.canonical_json())
                .unwrap(),
            update.manifest
        );
        let retry =
            phase8_normalized_result_store_with_entry(Some(&update.manifest), entry.clone())
                .unwrap();
        assert!(!retry.rewrite_required);

        let write = phase8_normalization_write_result(
            &normalized,
            entry.path.clone(),
            Some(Phase8NormalizationWriteStore {
                path: "build/normalized/manifest.json".to_owned(),
                manifest_hash: update.manifest.file_hash(),
            }),
        )
        .unwrap();
        let json = write.canonical_json();
        assert!(json.contains(PHASE8_NORMALIZATION_WRITE_RESULT_SCHEMA));
        assert!(!json.contains("\"result_hash\":"));
        assert!(json.contains("\"status\":\"written\""));

        let invalid_store = phase8_normalization_write_result(
            &normalized,
            entry.path,
            Some(Phase8NormalizationWriteStore {
                path: "/tmp/normalized-store.json".to_owned(),
                manifest_hash: update.manifest.file_hash(),
            }),
        )
        .unwrap_err();
        assert_eq!(invalid_store.command, Phase8CommandName::NormalizeResults);
        assert_eq!(
            invalid_store.field.as_deref(),
            Some("normalized_store.path")
        );
        assert_eq!(invalid_store.actual_value.as_deref(), Some("invalid_path"));
    }

    #[test]
    fn m4_store_references_are_closed_manifest_references() {
        let request_reference = parse_phase8_request_store_reference(&format!(
            r#"{{
              "kind":"manifest",
              "path":"build/check-requests/manifest.json",
              "manifest_hash":"{}"
            }}"#,
            hash_wire(44)
        ))
        .unwrap();
        assert_eq!(request_reference.path, "build/check-requests/manifest.json");
        assert!(request_reference
            .canonical_json()
            .contains("\"kind\":\"manifest\""));

        let result_reference = parse_phase8_machine_result_store_reference(&format!(
            r#"{{
              "kind":"manifest",
              "path":"build/check-results/manifest.json",
              "manifest_hash":"{}"
            }}"#,
            hash_wire(45)
        ))
        .unwrap();
        assert_eq!(result_reference.path, "build/check-results/manifest.json");

        let invalid = parse_phase8_request_store_reference(&format!(
            r#"{{
              "kind":"manifest",
              "path":"/tmp/manifest.json",
              "manifest_hash":"{}"
            }}"#,
            hash_wire(46)
        ))
        .unwrap_err();
        assert_eq!(invalid.field.as_ref(), "request_store.path");
        assert_eq!(invalid.actual_value.as_deref(), Some("invalid_path"));
    }
}
