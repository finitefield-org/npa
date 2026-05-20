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
pub const PHASE8_COMPARE_VALIDATION_RESULT_SCHEMA: &str = "npa.phase8.compare_validation_result.v1";
pub const PHASE8_AI_AUDIT_INPUT_POLICY_SCHEMA: &str = "npa.phase8.ai_audit_input_policy.v1";
pub const PHASE8_AI_AUDIT_SIDECAR_SCHEMA: &str = "npa.phase8.ai_audit_sidecar.v1";
pub const PHASE8_AI_AUDIT_PROMPT_INPUT_SCHEMA: &str = "npa.phase8.ai_audit_prompt_input.v1";
pub const PHASE8_AUDIT_SIDECAR_VALIDATION_RESULT_SCHEMA: &str =
    "npa.phase8.audit_sidecar_validation_result.v1";
pub const PHASE8_AI_SIDECAR_DIAGNOSTIC_RESULT_SCHEMA: &str =
    "npa.phase8.ai_sidecar_diagnostic_result.v1";
pub const PHASE8_AI_SIDECAR_DIAGNOSTIC_EVALUATION_FAILURE_SCHEMA: &str =
    "npa.phase8.ai_sidecar_diagnostic_evaluation_failure.v1";
pub const PHASE8_RELEASE_POLICY_SCHEMA: &str = "npa.phase8.release_policy.v1";
pub const PHASE8_RELEASE_BUNDLE_STAGING_PLAN_SCHEMA: &str =
    "npa.phase8.release_bundle_staging_plan.v1";
pub const PHASE8_RELEASE_BUNDLE_STAGING_RESULT_SCHEMA: &str =
    "npa.phase8.release_bundle_staging_result.v1";
pub const PHASE8_AUXILIARY_RESULT_SCHEMA: &str = "npa.phase8.auxiliary_result.v1";
pub const PHASE8_AUXILIARY_RESULT_STORE_MANIFEST_SCHEMA: &str =
    "npa.phase8.auxiliary_result_store_manifest.v1";
pub const PHASE8_CHALLENGE_GENERATION_REQUEST_SCHEMA: &str =
    "npa.phase8.challenge_generation_request.v1";
pub const PHASE8_CHALLENGE_MANIFEST_SCHEMA: &str = "npa.phase8.challenge_manifest.v1";
pub const PHASE8_CHALLENGE_OUTPUT_STORE_MANIFEST_SCHEMA: &str =
    "npa.phase8.challenge_output_store_manifest.v1";
pub const PHASE8_CHALLENGE_GENERATION_RESULT_SCHEMA: &str =
    "npa.phase8.challenge_generation_result.v1";
pub const PHASE8_CHALLENGE_REQUEST_MATERIALIZATION_RESULT_SCHEMA: &str =
    "npa.phase8.challenge_request_materialization_result.v1";
pub const PHASE8_CHALLENGE_REPLAY_RESULT_SCHEMA: &str = "npa.phase8.challenge_replay_result.v1";
pub const PHASE8_CHALLENGE_REPLAY_STORE_MANIFEST_SCHEMA: &str =
    "npa.phase8.challenge_replay_store_manifest.v1";
pub const PHASE8_CHALLENGE_COVERAGE_SUMMARY_SCHEMA: &str =
    "npa.phase8.challenge_coverage_summary.v1";
pub const PHASE8_MACHINE_CHECK_REQUEST_ERROR_RESULT_SCHEMA: &str =
    "npa.phase8.machine_check_request_error_result.v1";
pub const PHASE8_NORMALIZE_ERROR_RESULT_SCHEMA: &str = "npa.phase8.normalize_error_result.v1";
pub const PHASE8_COMMAND_ERROR_SCHEMA: &str = "npa.phase8.command_error.v1";
pub const PHASE8_API_ERROR_SCHEMA: &str = "npa.phase8.api_error.v1";
pub const PHASE8_AXIOM_POLICY_TOML_FORMAT: &str = "npa.phase8.axiom_policy.v1";

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
const AUXILIARY_RESULT_HASH_EXCLUDED_FIELDS: &[&str] = &["result_id", "result_hash", "diagnostics"];
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
    AuxiliaryResultHash,
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
            Self::AuxiliaryResultHash => "result_hash",
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
        Phase8HashUsageKind::AuxiliaryResultHash => Phase8HashUsageRule {
            kind,
            field_name: kind.field_name(),
            input_kind: Phase8HashInputKind::CanonicalJsonProjection,
            purpose:
                "AuxiliaryResult oracle outcome identity after local id and diagnostics removal",
            excluded_fields: AUXILIARY_RESULT_HASH_EXCLUDED_FIELDS,
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
pub enum Phase8ChallengeMutationKind {
    ChangeDeclarationBodyWithoutHash,
    ChangeDeclarationHashWithoutBody,
    DropAxiomReportEntry,
    AlterDeBruijnIndex,
    ReplaceNatZeroWithNoncanonicalPlaceholder,
    RemoveDependencyEntry,
    ReplaceImportExportHash,
    AddForbiddenAxiom,
    FlipCanonicalEncodingByte,
    ReorderDeclarations,
    InsertUnsupportedSchemaVersion,
    TruncateCertificateSection,
}

impl Phase8ChallengeMutationKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ChangeDeclarationBodyWithoutHash => "change_declaration_body_without_hash",
            Self::ChangeDeclarationHashWithoutBody => "change_declaration_hash_without_body",
            Self::DropAxiomReportEntry => "drop_axiom_report_entry",
            Self::AlterDeBruijnIndex => "alter_de_bruijn_index",
            Self::ReplaceNatZeroWithNoncanonicalPlaceholder => {
                "replace_nat_zero_with_noncanonical_placeholder"
            }
            Self::RemoveDependencyEntry => "remove_dependency_entry",
            Self::ReplaceImportExportHash => "replace_import_export_hash",
            Self::AddForbiddenAxiom => "add_forbidden_axiom",
            Self::FlipCanonicalEncodingByte => "flip_canonical_encoding_byte",
            Self::ReorderDeclarations => "reorder_declarations",
            Self::InsertUnsupportedSchemaVersion => "insert_unsupported_schema_version",
            Self::TruncateCertificateSection => "truncate_certificate_section",
        }
    }

    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "change_declaration_body_without_hash" => Some(Self::ChangeDeclarationBodyWithoutHash),
            "change_declaration_hash_without_body" => Some(Self::ChangeDeclarationHashWithoutBody),
            "drop_axiom_report_entry" => Some(Self::DropAxiomReportEntry),
            "alter_de_bruijn_index" => Some(Self::AlterDeBruijnIndex),
            "replace_nat_zero_with_noncanonical_placeholder" => {
                Some(Self::ReplaceNatZeroWithNoncanonicalPlaceholder)
            }
            "remove_dependency_entry" => Some(Self::RemoveDependencyEntry),
            "replace_import_export_hash" => Some(Self::ReplaceImportExportHash),
            "add_forbidden_axiom" => Some(Self::AddForbiddenAxiom),
            "flip_canonical_encoding_byte" => Some(Self::FlipCanonicalEncodingByte),
            "reorder_declarations" => Some(Self::ReorderDeclarations),
            "insert_unsupported_schema_version" => Some(Self::InsertUnsupportedSchemaVersion),
            "truncate_certificate_section" => Some(Self::TruncateCertificateSection),
            _ => None,
        }
    }

    pub const fn is_rejection_required(self) -> bool {
        true
    }

    const fn requires_whole_certificate_target(self) -> bool {
        matches!(
            self,
            Self::FlipCanonicalEncodingByte
                | Self::ReorderDeclarations
                | Self::InsertUnsupportedSchemaVersion
                | Self::TruncateCertificateSection
        )
    }

    fn outcome_error_kinds(self) -> Vec<String> {
        match self {
            Self::AddForbiddenAxiom => vec!["forbidden_axiom".to_owned()],
            Self::InsertUnsupportedSchemaVersion => vec!["unsupported_schema_version".to_owned()],
            Self::FlipCanonicalEncodingByte | Self::TruncateCertificateSection => {
                vec!["certificate_decode_error".to_owned()]
            }
            _ => vec!["certificate_hash_mismatch".to_owned()],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8ChallengeGeneratedByKind {
    Ci,
    Ai,
}

impl Phase8ChallengeGeneratedByKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ci => "ci",
            Self::Ai => "ai",
        }
    }

    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "ci" => Some(Self::Ci),
            "ai" => Some(Self::Ai),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeGeneratedBy {
    pub kind: Phase8ChallengeGeneratedByKind,
    pub prompt_hash: Option<Hash>,
}

impl Phase8ChallengeGeneratedBy {
    fn canonical_json(&self) -> String {
        let mut pairs = vec![(
            "kind".to_owned(),
            phase8_json_string_literal(self.kind.as_str()),
        )];
        push_optional_hash_pair(&mut pairs, "prompt_hash", self.prompt_hash);
        canonical_json_object_from_pairs(pairs)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeImports {
    pub mode: String,
    pub manifest: String,
    pub manifest_hash: Hash,
}

impl Phase8ChallengeImports {
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
pub struct Phase8ChallengeBaseCertificate {
    pub path: String,
    pub file_hash: Hash,
    pub claimed_certificate_hash: Hash,
}

impl Phase8ChallengeBaseCertificate {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "claimed_certificate_hash".to_owned(),
                phase8_hash_json_literal(&self.claimed_certificate_hash),
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
pub struct Phase8ChallengeMutatedCertificate {
    pub path: String,
    pub file_hash: Hash,
    pub claimed_certificate_hash: Option<Hash>,
}

impl Phase8ChallengeMutatedCertificate {
    fn canonical_json(&self) -> String {
        let mut pairs = vec![
            (
                "file_hash".to_owned(),
                phase8_hash_json_literal(&self.file_hash),
            ),
            ("path".to_owned(), phase8_json_string_literal(&self.path)),
        ];
        push_optional_hash_pair(
            &mut pairs,
            "claimed_certificate_hash",
            self.claimed_certificate_hash,
        );
        canonical_json_object_from_pairs(pairs)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeMutation {
    pub kind: String,
    pub target: String,
    pub seed: Hash,
}

impl Phase8ChallengeMutation {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            ("kind".to_owned(), phase8_json_string_literal(&self.kind)),
            ("seed".to_owned(), phase8_hash_json_literal(&self.seed)),
            (
                "target".to_owned(),
                phase8_json_string_literal(&self.target),
            ),
        ])
    }

    pub fn known_kind(&self) -> Option<Phase8ChallengeMutationKind> {
        Phase8ChallengeMutationKind::parse(&self.kind)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeGenerationOutput {
    pub store_manifest_path: String,
    pub manifest_path: String,
    pub mutated_certificate_path: String,
}

impl Phase8ChallengeGenerationOutput {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "manifest_path".to_owned(),
                phase8_json_string_literal(&self.manifest_path),
            ),
            (
                "mutated_certificate_path".to_owned(),
                phase8_json_string_literal(&self.mutated_certificate_path),
            ),
            (
                "store_manifest_path".to_owned(),
                phase8_json_string_literal(&self.store_manifest_path),
            ),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeGenerationRequest {
    pub request_id: String,
    pub challenge_id: String,
    pub policy_hash: Hash,
    pub module: String,
    pub imports: Phase8ChallengeImports,
    pub base_certificate: Phase8ChallengeBaseCertificate,
    pub mutation: Phase8ChallengeMutation,
    pub output: Phase8ChallengeGenerationOutput,
    pub generated_by: Phase8ChallengeGeneratedBy,
}

impl Phase8ChallengeGenerationRequest {
    pub fn request_hash(&self) -> Hash {
        phase8_sha256(self.hash_input_canonical_json().as_bytes())
    }

    pub fn hash_input_canonical_json(&self) -> String {
        canonical_json_object_from_pairs(self.canonical_json_pairs_without_request_identity())
    }

    pub fn canonical_json(&self) -> String {
        let mut pairs = self.canonical_json_pairs_without_request_identity();
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

    fn canonical_json_pairs_without_request_identity(&self) -> Vec<(String, String)> {
        vec![
            (
                "base_certificate".to_owned(),
                self.base_certificate.canonical_json(),
            ),
            (
                "challenge_id".to_owned(),
                phase8_json_string_literal(&self.challenge_id),
            ),
            (
                "generated_by".to_owned(),
                self.generated_by.canonical_json(),
            ),
            ("imports".to_owned(), self.imports.canonical_json()),
            (
                "module".to_owned(),
                phase8_json_string_literal(&self.module),
            ),
            ("mutation".to_owned(), self.mutation.canonical_json()),
            ("output".to_owned(), self.output.canonical_json()),
            (
                "policy_hash".to_owned(),
                phase8_hash_json_literal(&self.policy_hash),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_CHALLENGE_GENERATION_REQUEST_SCHEMA),
            ),
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeOutcomeHint {
    pub status: String,
    pub error_kinds: Vec<String>,
}

impl Phase8ChallengeOutcomeHint {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "error_kinds".to_owned(),
                phase8_json_string_array(&self.error_kinds),
            ),
            (
                "status".to_owned(),
                phase8_json_string_literal(&self.status),
            ),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeReplayMetadata {
    pub generator: String,
    pub generator_version: String,
    pub generator_build_hash: Hash,
    pub args_hash: Hash,
}

impl Phase8ChallengeReplayMetadata {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "args_hash".to_owned(),
                phase8_hash_json_literal(&self.args_hash),
            ),
            (
                "generator".to_owned(),
                phase8_json_string_literal(&self.generator),
            ),
            (
                "generator_build_hash".to_owned(),
                phase8_hash_json_literal(&self.generator_build_hash),
            ),
            (
                "generator_version".to_owned(),
                phase8_json_string_literal(&self.generator_version),
            ),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeManifest {
    pub challenge_id: String,
    pub policy_hash: Hash,
    pub module: String,
    pub imports: Phase8ChallengeImports,
    pub base_certificate: Phase8ChallengeBaseCertificate,
    pub mutated_certificate: Phase8ChallengeMutatedCertificate,
    pub mutation: Phase8ChallengeMutation,
    pub outcome_hint: Phase8ChallengeOutcomeHint,
    pub replay: Phase8ChallengeReplayMetadata,
    pub generated_by: Phase8ChallengeGeneratedBy,
}

impl Phase8ChallengeManifest {
    pub fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "base_certificate".to_owned(),
                self.base_certificate.canonical_json(),
            ),
            (
                "challenge_id".to_owned(),
                phase8_json_string_literal(&self.challenge_id),
            ),
            (
                "generated_by".to_owned(),
                self.generated_by.canonical_json(),
            ),
            ("imports".to_owned(), self.imports.canonical_json()),
            (
                "module".to_owned(),
                phase8_json_string_literal(&self.module),
            ),
            ("mutation".to_owned(), self.mutation.canonical_json()),
            (
                "mutated_certificate".to_owned(),
                self.mutated_certificate.canonical_json(),
            ),
            (
                "outcome_hint".to_owned(),
                self.outcome_hint.canonical_json(),
            ),
            (
                "policy_hash".to_owned(),
                phase8_hash_json_literal(&self.policy_hash),
            ),
            ("replay".to_owned(), self.replay.canonical_json()),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_CHALLENGE_MANIFEST_SCHEMA),
            ),
        ])
    }

    pub fn manifest_hash(&self) -> Hash {
        phase8_file_hash(self.canonical_json().as_bytes())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeOutputStoreEntry {
    pub challenge_id: String,
    pub manifest_path: String,
    pub manifest_hash: Hash,
}

impl Phase8ChallengeOutputStoreEntry {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "challenge_id".to_owned(),
                phase8_json_string_literal(&self.challenge_id),
            ),
            (
                "manifest_hash".to_owned(),
                phase8_hash_json_literal(&self.manifest_hash),
            ),
            (
                "manifest_path".to_owned(),
                phase8_json_string_literal(&self.manifest_path),
            ),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeOutputStoreManifest {
    pub entries: Vec<Phase8ChallengeOutputStoreEntry>,
}

impl Phase8ChallengeOutputStoreManifest {
    pub fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "entries".to_owned(),
                canonical_json_array(
                    self.entries
                        .iter()
                        .map(Phase8ChallengeOutputStoreEntry::canonical_json)
                        .collect(),
                ),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_CHALLENGE_OUTPUT_STORE_MANIFEST_SCHEMA),
            ),
        ])
    }

    pub fn file_hash(&self) -> Hash {
        phase8_file_hash(self.canonical_json().as_bytes())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeOutputStoreUpdate {
    pub manifest: Phase8ChallengeOutputStoreManifest,
    pub rewrite_required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeManifestReference {
    pub path: String,
    pub manifest_hash: Hash,
}

impl Phase8ChallengeManifestReference {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "manifest_hash".to_owned(),
                phase8_hash_json_literal(&self.manifest_hash),
            ),
            ("path".to_owned(), phase8_json_string_literal(&self.path)),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeOutputStoreReference {
    pub path: String,
    pub manifest_hash: Hash,
}

impl Phase8ChallengeOutputStoreReference {
    fn canonical_json(&self) -> String {
        phase8_store_reference_canonical_json(&self.path, self.manifest_hash)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeGeneratedCertificate {
    pub path: String,
    pub file_hash: Hash,
    pub claimed_certificate_hash: Option<Hash>,
}

impl Phase8ChallengeGeneratedCertificate {
    fn canonical_json(&self) -> String {
        let mut pairs = vec![
            (
                "file_hash".to_owned(),
                phase8_hash_json_literal(&self.file_hash),
            ),
            ("path".to_owned(), phase8_json_string_literal(&self.path)),
        ];
        push_optional_hash_pair(
            &mut pairs,
            "claimed_certificate_hash",
            self.claimed_certificate_hash,
        );
        canonical_json_object_from_pairs(pairs)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeGenerationResult {
    pub status: String,
    pub challenge_id: String,
    pub request_hash: Hash,
    pub policy_hash: Hash,
    pub challenge_manifest: Phase8ChallengeManifestReference,
    pub mutated_certificate: Phase8ChallengeGeneratedCertificate,
    pub challenge_store: Phase8ChallengeOutputStoreReference,
}

impl Phase8ChallengeGenerationResult {
    pub fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "challenge_id".to_owned(),
                phase8_json_string_literal(&self.challenge_id),
            ),
            (
                "challenge_manifest".to_owned(),
                self.challenge_manifest.canonical_json(),
            ),
            (
                "challenge_store".to_owned(),
                self.challenge_store.canonical_json(),
            ),
            (
                "mutated_certificate".to_owned(),
                self.mutated_certificate.canonical_json(),
            ),
            (
                "policy_hash".to_owned(),
                phase8_hash_json_literal(&self.policy_hash),
            ),
            (
                "request_hash".to_owned(),
                phase8_hash_json_literal(&self.request_hash),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_CHALLENGE_GENERATION_RESULT_SCHEMA),
            ),
            (
                "status".to_owned(),
                phase8_json_string_literal(&self.status),
            ),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeGeneration {
    pub result: Phase8ChallengeGenerationResult,
    pub manifest: Phase8ChallengeManifest,
    pub mutated_certificate_bytes: Vec<u8>,
    pub challenge_store: Phase8ChallengeOutputStoreManifest,
    pub challenge_store_rewrite_required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeMaterializedRequestEntry {
    pub checker_profile: String,
    pub request_hash: Hash,
    pub path: String,
    pub file_hash: Hash,
}

impl Phase8ChallengeMaterializedRequestEntry {
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
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeRequestMaterializationResult {
    pub status: String,
    pub challenge_id: String,
    pub manifest_hash: Hash,
    pub policy_hash: Hash,
    pub request_store: Phase8RequestStoreReference,
    pub requests: Vec<Phase8ChallengeMaterializedRequestEntry>,
}

impl Phase8ChallengeRequestMaterializationResult {
    pub fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "challenge_id".to_owned(),
                phase8_json_string_literal(&self.challenge_id),
            ),
            (
                "manifest_hash".to_owned(),
                phase8_hash_json_literal(&self.manifest_hash),
            ),
            (
                "policy_hash".to_owned(),
                phase8_hash_json_literal(&self.policy_hash),
            ),
            (
                "request_store".to_owned(),
                self.request_store.canonical_json(),
            ),
            (
                "requests".to_owned(),
                canonical_json_array(
                    self.requests
                        .iter()
                        .map(Phase8ChallengeMaterializedRequestEntry::canonical_json)
                        .collect(),
                ),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_CHALLENGE_REQUEST_MATERIALIZATION_RESULT_SCHEMA),
            ),
            (
                "status".to_owned(),
                phase8_json_string_literal(&self.status),
            ),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeRequestMaterialization {
    pub result: Phase8ChallengeRequestMaterializationResult,
    pub requests: Vec<Phase8MachineCheckRequest>,
    pub request_store: Phase8RequestStoreManifest,
    pub request_store_rewrite_required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeReplayCheckerResult {
    pub result_id: String,
    pub result_hash: Hash,
    pub run_artifact_hash: Hash,
    pub checker_profile: String,
}

impl Phase8ChallengeReplayCheckerResult {
    fn canonical_json(&self) -> String {
        let mut pairs = self.hash_projection_pairs();
        pairs.push((
            "result_id".to_owned(),
            phase8_json_string_literal(&self.result_id),
        ));
        canonical_json_object_from_pairs(pairs)
    }

    fn hash_projection_json(&self) -> String {
        canonical_json_object_from_pairs(self.hash_projection_pairs())
    }

    fn hash_projection_pairs(&self) -> Vec<(String, String)> {
        vec![
            (
                "checker_profile".to_owned(),
                phase8_json_string_literal(&self.checker_profile),
            ),
            (
                "result_hash".to_owned(),
                phase8_hash_json_literal(&self.result_hash),
            ),
            (
                "run_artifact_hash".to_owned(),
                phase8_hash_json_literal(&self.run_artifact_hash),
            ),
        ]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeReplayResult {
    pub result_id: String,
    pub challenge_id: String,
    pub manifest_hash: Hash,
    pub mutated_file_hash: Hash,
    pub mutated_claimed_certificate_hash: Option<Hash>,
    pub checker_results: Vec<Phase8ChallengeReplayCheckerResult>,
    pub missing_checker_profiles: Vec<String>,
    pub normalized_result_hash: Option<Hash>,
    pub policy_hash: Hash,
    pub artifact_hash: Hash,
    pub comparison_status: Option<Phase8NormalizedComparisonStatus>,
    pub observed_error_kinds: Vec<String>,
}

impl Phase8ChallengeReplayResult {
    pub fn result_hash(&self) -> Hash {
        phase8_sha256(self.hash_input_canonical_json().as_bytes())
    }

    pub fn hash_input_canonical_json(&self) -> String {
        canonical_json_object_from_pairs(self.hash_input_pairs())
    }

    pub fn canonical_json(&self) -> String {
        let mut pairs = self.canonical_json_pairs_without_result_identity();
        pairs.push((
            "result_hash".to_owned(),
            phase8_hash_json_literal(&self.result_hash()),
        ));
        pairs.push((
            "result_id".to_owned(),
            phase8_json_string_literal(&self.result_id),
        ));
        canonical_json_object_from_pairs(pairs)
    }

    fn hash_input_pairs(&self) -> Vec<(String, String)> {
        self.canonical_json_pairs(false)
    }

    fn canonical_json_pairs_without_result_identity(&self) -> Vec<(String, String)> {
        self.canonical_json_pairs(true)
    }

    fn canonical_json_pairs(&self, include_checker_result_ids: bool) -> Vec<(String, String)> {
        let mut pairs = vec![
            (
                "artifact_hash".to_owned(),
                phase8_hash_json_literal(&self.artifact_hash),
            ),
            (
                "challenge_id".to_owned(),
                phase8_json_string_literal(&self.challenge_id),
            ),
            (
                "checker_results".to_owned(),
                canonical_json_array(if include_checker_result_ids {
                    self.checker_results
                        .iter()
                        .map(Phase8ChallengeReplayCheckerResult::canonical_json)
                        .collect()
                } else {
                    self.checker_results
                        .iter()
                        .map(Phase8ChallengeReplayCheckerResult::hash_projection_json)
                        .collect()
                }),
            ),
            (
                "manifest_hash".to_owned(),
                phase8_hash_json_literal(&self.manifest_hash),
            ),
            (
                "missing_checker_profiles".to_owned(),
                phase8_json_string_array(&self.missing_checker_profiles),
            ),
            (
                "mutated_file_hash".to_owned(),
                phase8_hash_json_literal(&self.mutated_file_hash),
            ),
            (
                "observed_error_kinds".to_owned(),
                phase8_json_string_array(&self.observed_error_kinds),
            ),
            (
                "policy_hash".to_owned(),
                phase8_hash_json_literal(&self.policy_hash),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_CHALLENGE_REPLAY_RESULT_SCHEMA),
            ),
        ];
        push_optional_hash_pair(
            &mut pairs,
            "mutated_claimed_certificate_hash",
            self.mutated_claimed_certificate_hash,
        );
        push_optional_hash_pair(
            &mut pairs,
            "normalized_result_hash",
            self.normalized_result_hash,
        );
        if let Some(status) = self.comparison_status {
            pairs.push((
                "comparison_status".to_owned(),
                phase8_json_string_literal(status.as_str()),
            ));
        }
        pairs
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeReplayStoreEntry {
    pub challenge_id: String,
    pub manifest_hash: Hash,
    pub result_hash: Hash,
    pub artifact_hash: Hash,
    pub path: String,
    pub file_hash: Hash,
}

impl Phase8ChallengeReplayStoreEntry {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "artifact_hash".to_owned(),
                phase8_hash_json_literal(&self.artifact_hash),
            ),
            (
                "challenge_id".to_owned(),
                phase8_json_string_literal(&self.challenge_id),
            ),
            (
                "file_hash".to_owned(),
                phase8_hash_json_literal(&self.file_hash),
            ),
            (
                "manifest_hash".to_owned(),
                phase8_hash_json_literal(&self.manifest_hash),
            ),
            ("path".to_owned(), phase8_json_string_literal(&self.path)),
            (
                "result_hash".to_owned(),
                phase8_hash_json_literal(&self.result_hash),
            ),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeReplayStoreManifest {
    pub results: Vec<Phase8ChallengeReplayStoreEntry>,
}

impl Phase8ChallengeReplayStoreManifest {
    pub fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "results".to_owned(),
                canonical_json_array(
                    self.results
                        .iter()
                        .map(Phase8ChallengeReplayStoreEntry::canonical_json)
                        .collect(),
                ),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_CHALLENGE_REPLAY_STORE_MANIFEST_SCHEMA),
            ),
        ])
    }

    pub fn file_hash(&self) -> Hash {
        phase8_file_hash(self.canonical_json().as_bytes())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeReplayStoreUpdate {
    pub manifest: Phase8ChallengeReplayStoreManifest,
    pub rewrite_required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeReplayAggregation {
    pub result: Phase8ChallengeReplayResult,
    pub replay_store: Phase8ChallengeReplayStoreManifest,
    pub replay_store_rewrite_required: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeCoverageEntry {
    pub challenge_id: String,
    pub manifest_hash: Hash,
    pub replay_result_hash: Hash,
    pub comparison_status: Phase8NormalizedComparisonStatus,
}

impl Phase8ChallengeCoverageEntry {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "challenge_id".to_owned(),
                phase8_json_string_literal(&self.challenge_id),
            ),
            (
                "comparison_status".to_owned(),
                phase8_json_string_literal(self.comparison_status.as_str()),
            ),
            (
                "manifest_hash".to_owned(),
                phase8_hash_json_literal(&self.manifest_hash),
            ),
            (
                "replay_result_hash".to_owned(),
                phase8_hash_json_literal(&self.replay_result_hash),
            ),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ChallengeCoverageSummary {
    pub summary_id: String,
    pub policy_hash: Hash,
    pub artifact_hash: Hash,
    pub target_normalized_result_hash: Hash,
    pub challenge_store_manifest_hash: Hash,
    pub result_store_manifest_hash: Hash,
    pub total_challenges: u64,
    pub replayed_challenges: u64,
    pub unexpected_acceptances: u64,
    pub entries: Vec<Phase8ChallengeCoverageEntry>,
}

impl Phase8ChallengeCoverageSummary {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        policy_hash: Hash,
        artifact_hash: Hash,
        target_normalized_result_hash: Hash,
        challenge_store_manifest_hash: Hash,
        result_store_manifest_hash: Hash,
        total_challenges: u64,
        unexpected_acceptances: u64,
        entries: Vec<Phase8ChallengeCoverageEntry>,
    ) -> Self {
        let replayed_challenges = entries.len() as u64;
        let mut summary = Self {
            summary_id: String::new(),
            policy_hash,
            artifact_hash,
            target_normalized_result_hash,
            challenge_store_manifest_hash,
            result_store_manifest_hash,
            total_challenges,
            replayed_challenges,
            unexpected_acceptances,
            entries,
        };
        summary.summary_id = phase8_challenge_coverage_summary_id(summary.summary_hash());
        summary
    }

    pub fn summary_hash(&self) -> Hash {
        phase8_sha256(self.hash_input_canonical_json().as_bytes())
    }

    pub fn hash_input_canonical_json(&self) -> String {
        canonical_json_object_from_pairs(self.hash_input_pairs())
    }

    pub fn canonical_json(&self) -> String {
        let mut pairs = self.hash_input_pairs();
        pairs.push((
            "summary_hash".to_owned(),
            phase8_hash_json_literal(&self.summary_hash()),
        ));
        pairs.push((
            "summary_id".to_owned(),
            phase8_json_string_literal(&self.summary_id),
        ));
        canonical_json_object_from_pairs(pairs)
    }

    pub fn passes_release_coverage_condition(&self) -> bool {
        self.replayed_challenges == self.total_challenges
            && self.unexpected_acceptances == 0
            && self.entries.iter().all(|entry| {
                entry.comparison_status == Phase8NormalizedComparisonStatus::AllAgreeFailed
            })
    }

    fn hash_input_pairs(&self) -> Vec<(String, String)> {
        canonical_json_coverage_summary_pairs(
            self.policy_hash,
            self.artifact_hash,
            self.target_normalized_result_hash,
            self.challenge_store_manifest_hash,
            self.result_store_manifest_hash,
            self.total_challenges,
            self.replayed_challenges,
            self.unexpected_acceptances,
            &self.entries,
        )
    }
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

    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "all_agree_checked" => Some(Self::AllAgreeChecked),
            "all_agree_failed" => Some(Self::AllAgreeFailed),
            "disagreement" => Some(Self::Disagreement),
            "missing_checker_result" => Some(Self::MissingCheckerResult),
            "policy_failure" => Some(Self::PolicyFailure),
            "inconclusive" => Some(Self::Inconclusive),
            _ => None,
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
    pub kind: String,
    pub error_kind: String,
    pub reason_code: String,
    pub checker_profile: Option<String>,
    pub result_hash: Option<Hash>,
    pub field: Option<String>,
    pub expected_hash: Option<Hash>,
    pub actual_hash: Option<Hash>,
    pub expected_value: Option<String>,
    pub actual_value: Option<String>,
}

impl Phase8NormalizedStatusReason {
    fn canonical_json(&self) -> String {
        let mut pairs = vec![
            (
                "error_kind".to_owned(),
                phase8_json_string_literal(&self.error_kind),
            ),
            ("kind".to_owned(), phase8_json_string_literal(&self.kind)),
            (
                "reason_code".to_owned(),
                phase8_json_string_literal(&self.reason_code),
            ),
        ];
        push_optional_string_pair(
            &mut pairs,
            "checker_profile",
            self.checker_profile.as_deref(),
        );
        push_optional_hash_pair(&mut pairs, "result_hash", self.result_hash);
        push_optional_string_pair(&mut pairs, "field", self.field.as_deref());
        push_optional_hash_pair(&mut pairs, "expected_hash", self.expected_hash);
        push_optional_hash_pair(&mut pairs, "actual_hash", self.actual_hash);
        push_optional_string_pair(&mut pairs, "expected_value", self.expected_value.as_deref());
        push_optional_string_pair(&mut pairs, "actual_value", self.actual_value.as_deref());
        canonical_json_object_from_pairs(pairs)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8CompareValidationStatus {
    Valid,
    Failed,
}

impl Phase8CompareValidationStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Valid => "valid",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8CompareValidationErrorKind {
    NormalizedResultFileUnreadable,
    NormalizedResultJsonInvalid,
    NormalizedResultSchemaInvalid,
    NormalizedArtifactHashMismatch,
    ComparisonMismatch,
    NormalizedResultHashMismatch,
    PolicyFailure,
}

impl Phase8CompareValidationErrorKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NormalizedResultFileUnreadable => "normalized_result_file_unreadable",
            Self::NormalizedResultJsonInvalid => "normalized_result_json_invalid",
            Self::NormalizedResultSchemaInvalid => "normalized_result_schema_invalid",
            Self::NormalizedArtifactHashMismatch => "normalized_artifact_hash_mismatch",
            Self::ComparisonMismatch => "comparison_mismatch",
            Self::NormalizedResultHashMismatch => "normalized_result_hash_mismatch",
            Self::PolicyFailure => "policy_failure",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8CompareValidationError {
    pub kind: Phase8CompareValidationErrorKind,
    pub reason_code: Option<String>,
    pub field: Option<String>,
    pub expected_hash: Option<Hash>,
    pub actual_hash: Option<Hash>,
    pub expected_value: Option<String>,
    pub actual_value: Option<String>,
}

impl Phase8CompareValidationError {
    fn comparison_mismatch(expected_hash: Hash, actual_hash: Hash) -> Self {
        Self {
            kind: Phase8CompareValidationErrorKind::ComparisonMismatch,
            reason_code: None,
            field: Some("comparison".to_owned()),
            expected_hash: Some(expected_hash),
            actual_hash: Some(actual_hash),
            expected_value: None,
            actual_value: None,
        }
    }

    fn canonical_json(&self) -> String {
        let mut pairs = vec![(
            "kind".to_owned(),
            phase8_json_string_literal(self.kind.as_str()),
        )];
        push_optional_string_pair(&mut pairs, "reason_code", self.reason_code.as_deref());
        push_optional_string_pair(&mut pairs, "field", self.field.as_deref());
        push_optional_hash_pair(&mut pairs, "expected_hash", self.expected_hash);
        push_optional_hash_pair(&mut pairs, "actual_hash", self.actual_hash);
        push_optional_string_pair(&mut pairs, "expected_value", self.expected_value.as_deref());
        push_optional_string_pair(&mut pairs, "actual_value", self.actual_value.as_deref());
        canonical_json_object_from_pairs(pairs)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8CompareValidationResult {
    pub status: Phase8CompareValidationStatus,
    pub normalized_result_hash: Option<Hash>,
    pub policy_hash: Option<Hash>,
    pub embedded_comparison_status: Option<Phase8NormalizedComparisonStatus>,
    pub recomputed_comparison_status: Option<Phase8NormalizedComparisonStatus>,
    pub error: Option<Phase8CompareValidationError>,
}

impl Phase8CompareValidationResult {
    fn valid(
        normalized_result_hash: Hash,
        policy_hash: Hash,
        comparison_status: Phase8NormalizedComparisonStatus,
    ) -> Self {
        Self {
            status: Phase8CompareValidationStatus::Valid,
            normalized_result_hash: Some(normalized_result_hash),
            policy_hash: Some(policy_hash),
            embedded_comparison_status: Some(comparison_status),
            recomputed_comparison_status: Some(comparison_status),
            error: None,
        }
    }

    fn comparison_mismatch(
        normalized_result_hash: Hash,
        policy_hash: Hash,
        embedded_status: Phase8NormalizedComparisonStatus,
        recomputed_status: Phase8NormalizedComparisonStatus,
        expected_hash: Hash,
        actual_hash: Hash,
    ) -> Self {
        Self {
            status: Phase8CompareValidationStatus::Failed,
            normalized_result_hash: Some(normalized_result_hash),
            policy_hash: Some(policy_hash),
            embedded_comparison_status: Some(embedded_status),
            recomputed_comparison_status: Some(recomputed_status),
            error: Some(Phase8CompareValidationError::comparison_mismatch(
                expected_hash,
                actual_hash,
            )),
        }
    }

    pub fn canonical_json(&self) -> String {
        let mut pairs = vec![
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_COMPARE_VALIDATION_RESULT_SCHEMA),
            ),
            (
                "status".to_owned(),
                phase8_json_string_literal(self.status.as_str()),
            ),
        ];
        if let Some(error) = &self.error {
            pairs.push(("error".to_owned(), error.canonical_json()));
        }
        push_optional_hash_pair(
            &mut pairs,
            "normalized_result_hash",
            self.normalized_result_hash,
        );
        push_optional_hash_pair(&mut pairs, "policy_hash", self.policy_hash);
        if let Some(status) = self.embedded_comparison_status {
            pairs.push((
                "embedded_comparison_status".to_owned(),
                phase8_json_string_literal(status.as_str()),
            ));
        }
        if let Some(status) = self.recomputed_comparison_status {
            pairs.push((
                "recomputed_comparison_status".to_owned(),
                phase8_json_string_literal(status.as_str()),
            ));
        }
        canonical_json_object_from_pairs(pairs)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8ReleaseMode {
    Nightly,
    Release,
    HighTrust,
}

impl Phase8ReleaseMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Nightly => "nightly",
            Self::Release => "release",
            Self::HighTrust => "high-trust",
        }
    }

    pub const fn trust_mode(self) -> Phase8TrustMode {
        match self {
            Self::Nightly => Phase8TrustMode::Nightly,
            Self::Release => Phase8TrustMode::Release,
            Self::HighTrust => Phase8TrustMode::HighTrust,
        }
    }

    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "nightly" => Some(Self::Nightly),
            "release" => Some(Self::Release),
            "high-trust" => Some(Self::HighTrust),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ReleasePolicyAiTriage {
    pub enabled: bool,
    pub required: bool,
    pub input_policy_hash: Option<Hash>,
}

impl Phase8ReleasePolicyAiTriage {
    fn canonical_json(&self) -> String {
        let mut pairs = vec![
            ("enabled".to_owned(), self.enabled.to_string()),
            ("required".to_owned(), self.required.to_string()),
        ];
        push_optional_hash_pair(&mut pairs, "input_policy_hash", self.input_policy_hash);
        canonical_json_object_from_pairs(pairs)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ReleasePolicy {
    pub id: String,
    pub version: u64,
    pub mode: Phase8ReleaseMode,
    pub runner_policy_hash: Hash,
    pub challenge_runner_policy_hash: Hash,
    pub ai_triage: Phase8ReleasePolicyAiTriage,
}

impl Phase8ReleasePolicy {
    pub fn policy_hash(&self) -> Hash {
        phase8_sha256(self.canonical_json().as_bytes())
    }

    pub fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            ("ai_triage".to_owned(), self.ai_triage.canonical_json()),
            (
                "challenge_runner_policy_hash".to_owned(),
                phase8_hash_json_literal(&self.challenge_runner_policy_hash),
            ),
            ("id".to_owned(), phase8_json_string_literal(&self.id)),
            (
                "mode".to_owned(),
                phase8_json_string_literal(self.mode.as_str()),
            ),
            (
                "runner_policy_hash".to_owned(),
                phase8_hash_json_literal(&self.runner_policy_hash),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_RELEASE_POLICY_SCHEMA),
            ),
            ("version".to_owned(), self.version.to_string()),
        ])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8ReleaseBundleStagingPhase {
    Store,
    Final,
}

impl Phase8ReleaseBundleStagingPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Store => "store",
            Self::Final => "final",
        }
    }

    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "store" => Some(Self::Store),
            "final" => Some(Self::Final),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Phase8ReleaseBundleArtifactKind {
    ReleasePolicy,
    RunnerPolicy,
    CheckerIdentityManifest,
    ImportLock,
    MachineCheckRequest,
    MachineCheckResult,
    NormalizedCheckResult,
    RequestStoreManifest,
    MachineResultStoreManifest,
    NormalizedResultStoreManifest,
    ChallengeManifest,
    ChallengeOutputStoreManifest,
    ChallengeReplayResult,
    ChallengeCoverageSummary,
    AuxiliaryResult,
    AiAuditInputPolicy,
    AiAuditSidecar,
    CompareValidationResponse,
    AuditSidecarValidationResponse,
}

impl Phase8ReleaseBundleArtifactKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReleasePolicy => "release_policy",
            Self::RunnerPolicy => "runner_policy",
            Self::CheckerIdentityManifest => "checker_identity_manifest",
            Self::ImportLock => "import_lock",
            Self::MachineCheckRequest => "machine_check_request",
            Self::MachineCheckResult => "machine_check_result",
            Self::NormalizedCheckResult => "normalized_check_result",
            Self::RequestStoreManifest => "request_store_manifest",
            Self::MachineResultStoreManifest => "machine_result_store_manifest",
            Self::NormalizedResultStoreManifest => "normalized_result_store_manifest",
            Self::ChallengeManifest => "challenge_manifest",
            Self::ChallengeOutputStoreManifest => "challenge_output_store_manifest",
            Self::ChallengeReplayResult => "challenge_replay_result",
            Self::ChallengeCoverageSummary => "challenge_coverage_summary",
            Self::AuxiliaryResult => "auxiliary_result",
            Self::AiAuditInputPolicy => "ai_audit_input_policy",
            Self::AiAuditSidecar => "ai_audit_sidecar",
            Self::CompareValidationResponse => "compare_validation_response",
            Self::AuditSidecarValidationResponse => "audit_sidecar_validation_response",
        }
    }

    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "release_policy" => Some(Self::ReleasePolicy),
            "runner_policy" => Some(Self::RunnerPolicy),
            "checker_identity_manifest" => Some(Self::CheckerIdentityManifest),
            "import_lock" => Some(Self::ImportLock),
            "machine_check_request" => Some(Self::MachineCheckRequest),
            "machine_check_result" => Some(Self::MachineCheckResult),
            "normalized_check_result" => Some(Self::NormalizedCheckResult),
            "request_store_manifest" => Some(Self::RequestStoreManifest),
            "machine_result_store_manifest" => Some(Self::MachineResultStoreManifest),
            "normalized_result_store_manifest" => Some(Self::NormalizedResultStoreManifest),
            "challenge_manifest" => Some(Self::ChallengeManifest),
            "challenge_output_store_manifest" => Some(Self::ChallengeOutputStoreManifest),
            "challenge_replay_result" => Some(Self::ChallengeReplayResult),
            "challenge_coverage_summary" => Some(Self::ChallengeCoverageSummary),
            "auxiliary_result" => Some(Self::AuxiliaryResult),
            "ai_audit_input_policy" => Some(Self::AiAuditInputPolicy),
            "ai_audit_sidecar" => Some(Self::AiAuditSidecar),
            "compare_validation_response" => Some(Self::CompareValidationResponse),
            "audit_sidecar_validation_response" => Some(Self::AuditSidecarValidationResponse),
            _ => None,
        }
    }

    const fn is_store_source_manifest(self) -> bool {
        matches!(
            self,
            Self::RequestStoreManifest
                | Self::MachineResultStoreManifest
                | Self::NormalizedResultStoreManifest
        )
    }

    const fn is_store_phase_direct_artifact(self) -> bool {
        matches!(self, Self::ChallengeOutputStoreManifest)
    }

    const fn allowed_for_phase(self, phase: Phase8ReleaseBundleStagingPhase) -> bool {
        match phase {
            Phase8ReleaseBundleStagingPhase::Store => {
                self.is_store_source_manifest() || self.is_store_phase_direct_artifact()
            }
            Phase8ReleaseBundleStagingPhase::Final => matches!(
                self,
                Self::ReleasePolicy
                    | Self::RunnerPolicy
                    | Self::CheckerIdentityManifest
                    | Self::ImportLock
                    | Self::ChallengeManifest
                    | Self::ChallengeReplayResult
                    | Self::ChallengeCoverageSummary
                    | Self::AuxiliaryResult
                    | Self::AiAuditInputPolicy
                    | Self::AiAuditSidecar
                    | Self::CompareValidationResponse
                    | Self::AuditSidecarValidationResponse
            ),
        }
    }

    const fn required_hash_fields(self) -> &'static [&'static str] {
        match self {
            Self::ReleasePolicy | Self::RunnerPolicy => &["policy_hash"],
            Self::CheckerIdentityManifest
            | Self::ImportLock
            | Self::RequestStoreManifest
            | Self::MachineResultStoreManifest
            | Self::NormalizedResultStoreManifest
            | Self::ChallengeManifest
            | Self::ChallengeOutputStoreManifest => &["manifest_hash"],
            Self::MachineCheckRequest => &["request_hash"],
            Self::MachineCheckResult => &["result_hash", "run_artifact_hash"],
            Self::NormalizedCheckResult => &["artifact_hash", "normalized_result_hash"],
            Self::ChallengeReplayResult | Self::AuxiliaryResult => &["result_hash"],
            Self::ChallengeCoverageSummary => &["summary_hash"],
            Self::AiAuditInputPolicy => &["input_policy_hash"],
            Self::AiAuditSidecar
            | Self::CompareValidationResponse
            | Self::AuditSidecarValidationResponse => &[],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ReleaseBundleStagingInput {
    pub kind: Phase8ReleaseBundleArtifactKind,
    pub path: String,
    pub file_hash: Hash,
    pub hashes: BTreeMap<String, Hash>,
}

impl Phase8ReleaseBundleStagingInput {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "file_hash".to_owned(),
                phase8_hash_json_literal(&self.file_hash),
            ),
            (
                "hashes".to_owned(),
                phase8_hashes_canonical_json(&self.hashes),
            ),
            (
                "kind".to_owned(),
                phase8_json_string_literal(self.kind.as_str()),
            ),
            ("path".to_owned(), phase8_json_string_literal(&self.path)),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ReleaseBundleStagingPlan {
    pub phase: Phase8ReleaseBundleStagingPhase,
    pub bundle_root: String,
    pub inputs: Vec<Phase8ReleaseBundleStagingInput>,
}

impl Phase8ReleaseBundleStagingPlan {
    pub fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "bundle_root".to_owned(),
                phase8_json_string_literal(&self.bundle_root),
            ),
            (
                "inputs".to_owned(),
                canonical_json_array(
                    self.inputs
                        .iter()
                        .map(Phase8ReleaseBundleStagingInput::canonical_json)
                        .collect(),
                ),
            ),
            (
                "phase".to_owned(),
                phase8_json_string_literal(self.phase.as_str()),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_RELEASE_BUNDLE_STAGING_PLAN_SCHEMA),
            ),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ReleaseBundleStagedArtifact {
    pub kind: Phase8ReleaseBundleArtifactKind,
    pub path: String,
    pub file_hash: Hash,
}

impl Phase8ReleaseBundleStagedArtifact {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "file_hash".to_owned(),
                phase8_hash_json_literal(&self.file_hash),
            ),
            (
                "kind".to_owned(),
                phase8_json_string_literal(self.kind.as_str()),
            ),
            ("path".to_owned(), phase8_json_string_literal(&self.path)),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ReleaseBundleStagedStoreManifest {
    pub kind: Phase8ReleaseBundleArtifactKind,
    pub path: String,
    pub manifest_hash: Hash,
}

impl Phase8ReleaseBundleStagedStoreManifest {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "kind".to_owned(),
                phase8_json_string_literal(self.kind.as_str()),
            ),
            (
                "manifest_hash".to_owned(),
                phase8_hash_json_literal(&self.manifest_hash),
            ),
            ("path".to_owned(), phase8_json_string_literal(&self.path)),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ReleaseBundleStagedFile {
    pub path: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ReleaseBundleStagingResult {
    pub phase: Phase8ReleaseBundleStagingPhase,
    pub bundle_root: String,
    pub staged_artifacts: Vec<Phase8ReleaseBundleStagedArtifact>,
    pub store_manifests: Vec<Phase8ReleaseBundleStagedStoreManifest>,
}

impl Phase8ReleaseBundleStagingResult {
    pub fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "bundle_root".to_owned(),
                phase8_json_string_literal(&self.bundle_root),
            ),
            (
                "phase".to_owned(),
                phase8_json_string_literal(self.phase.as_str()),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_RELEASE_BUNDLE_STAGING_RESULT_SCHEMA),
            ),
            (
                "staged_artifacts".to_owned(),
                canonical_json_array(
                    self.staged_artifacts
                        .iter()
                        .map(Phase8ReleaseBundleStagedArtifact::canonical_json)
                        .collect(),
                ),
            ),
            (
                "store_manifests".to_owned(),
                canonical_json_array(
                    self.store_manifests
                        .iter()
                        .map(Phase8ReleaseBundleStagedStoreManifest::canonical_json)
                        .collect(),
                ),
            ),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ReleaseBundleStaging {
    pub result: Phase8ReleaseBundleStagingResult,
    pub files: Vec<Phase8ReleaseBundleStagedFile>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8AxiomPolicy {
    pub allowed_axioms: Vec<String>,
}

impl Phase8AxiomPolicy {
    pub fn allows(&self, axiom: &str) -> bool {
        self.allowed_axioms
            .iter()
            .any(|candidate| candidate == axiom)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8AuxiliaryResultKind {
    AxiomPolicy,
    Reproducibility,
    ImportCertificateHash,
    AuditBundle,
}

impl Phase8AuxiliaryResultKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AxiomPolicy => "axiom_policy",
            Self::Reproducibility => "reproducibility",
            Self::ImportCertificateHash => "import_certificate_hash",
            Self::AuditBundle => "audit_bundle",
        }
    }

    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "axiom_policy" => Some(Self::AxiomPolicy),
            "reproducibility" => Some(Self::Reproducibility),
            "import_certificate_hash" => Some(Self::ImportCertificateHash),
            "audit_bundle" => Some(Self::AuditBundle),
            _ => None,
        }
    }

    const fn requires_selector(self) -> bool {
        matches!(self, Self::AxiomPolicy | Self::Reproducibility)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8AuxiliaryStatus {
    Passed,
    Failed,
    Inconclusive,
}

impl Phase8AuxiliaryStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Inconclusive => "inconclusive",
        }
    }

    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "passed" => Some(Self::Passed),
            "failed" => Some(Self::Failed),
            "inconclusive" => Some(Self::Inconclusive),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8AuxiliaryReasonCode {
    AxiomPolicyFailed,
    AxiomPolicyInconclusive,
    ReproducibilityMismatch,
    ReproducibilityInconclusive,
    ImportCertificateHashMismatch,
    ImportCertificateHashInconclusive,
    AuditBundleMissing,
    AuditBundleInvalid,
}

impl Phase8AuxiliaryReasonCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AxiomPolicyFailed => "axiom_policy_failed",
            Self::AxiomPolicyInconclusive => "axiom_policy_inconclusive",
            Self::ReproducibilityMismatch => "reproducibility_mismatch",
            Self::ReproducibilityInconclusive => "reproducibility_inconclusive",
            Self::ImportCertificateHashMismatch => "import_certificate_hash_mismatch",
            Self::ImportCertificateHashInconclusive => "import_certificate_hash_inconclusive",
            Self::AuditBundleMissing => "audit_bundle_missing",
            Self::AuditBundleInvalid => "audit_bundle_invalid",
        }
    }

    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "axiom_policy_failed" => Some(Self::AxiomPolicyFailed),
            "axiom_policy_inconclusive" => Some(Self::AxiomPolicyInconclusive),
            "reproducibility_mismatch" => Some(Self::ReproducibilityMismatch),
            "reproducibility_inconclusive" => Some(Self::ReproducibilityInconclusive),
            "import_certificate_hash_mismatch" => Some(Self::ImportCertificateHashMismatch),
            "import_certificate_hash_inconclusive" => Some(Self::ImportCertificateHashInconclusive),
            "audit_bundle_missing" => Some(Self::AuditBundleMissing),
            "audit_bundle_invalid" => Some(Self::AuditBundleInvalid),
            _ => None,
        }
    }

    const fn is_compatible(
        self,
        kind: Phase8AuxiliaryResultKind,
        status: Phase8AuxiliaryStatus,
    ) -> bool {
        matches!(
            (kind, status, self),
            (
                Phase8AuxiliaryResultKind::AxiomPolicy,
                Phase8AuxiliaryStatus::Failed,
                Self::AxiomPolicyFailed
            ) | (
                Phase8AuxiliaryResultKind::AxiomPolicy,
                Phase8AuxiliaryStatus::Inconclusive,
                Self::AxiomPolicyInconclusive
            ) | (
                Phase8AuxiliaryResultKind::Reproducibility,
                Phase8AuxiliaryStatus::Failed,
                Self::ReproducibilityMismatch
            ) | (
                Phase8AuxiliaryResultKind::Reproducibility,
                Phase8AuxiliaryStatus::Inconclusive,
                Self::ReproducibilityInconclusive
            ) | (
                Phase8AuxiliaryResultKind::ImportCertificateHash,
                Phase8AuxiliaryStatus::Failed,
                Self::ImportCertificateHashMismatch
            ) | (
                Phase8AuxiliaryResultKind::ImportCertificateHash,
                Phase8AuxiliaryStatus::Inconclusive,
                Self::ImportCertificateHashInconclusive
            ) | (
                Phase8AuxiliaryResultKind::AuditBundle,
                Phase8AuxiliaryStatus::Failed,
                Self::AuditBundleMissing
            ) | (
                Phase8AuxiliaryResultKind::AuditBundle,
                Phase8AuxiliaryStatus::Failed,
                Self::AuditBundleInvalid
            )
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8AuxiliaryError {
    pub reason_code: Phase8AuxiliaryReasonCode,
    pub field: Option<String>,
    pub expected_hash: Option<Hash>,
    pub actual_hash: Option<Hash>,
    pub expected_value: Option<String>,
    pub actual_value: Option<String>,
}

impl Phase8AuxiliaryError {
    pub fn value(
        reason_code: Phase8AuxiliaryReasonCode,
        field: impl Into<String>,
        expected_value: impl Into<String>,
        actual_value: impl Into<String>,
    ) -> Self {
        Self {
            reason_code,
            field: Some(field.into()),
            expected_hash: None,
            actual_hash: None,
            expected_value: Some(expected_value.into()),
            actual_value: Some(actual_value.into()),
        }
    }

    pub fn hash(
        reason_code: Phase8AuxiliaryReasonCode,
        field: impl Into<String>,
        expected_hash: Hash,
        actual_hash: Hash,
    ) -> Self {
        Self {
            reason_code,
            field: Some(field.into()),
            expected_hash: Some(expected_hash),
            actual_hash: Some(actual_hash),
            expected_value: None,
            actual_value: None,
        }
    }

    fn canonical_json(&self) -> String {
        let mut pairs = vec![
            (
                "kind".to_owned(),
                phase8_json_string_literal("auxiliary_failure"),
            ),
            (
                "reason_code".to_owned(),
                phase8_json_string_literal(self.reason_code.as_str()),
            ),
        ];
        push_optional_string_pair(&mut pairs, "field", self.field.as_deref());
        push_optional_hash_pair(&mut pairs, "expected_hash", self.expected_hash);
        push_optional_hash_pair(&mut pairs, "actual_hash", self.actual_hash);
        push_optional_string_pair(&mut pairs, "expected_value", self.expected_value.as_deref());
        push_optional_string_pair(&mut pairs, "actual_value", self.actual_value.as_deref());
        canonical_json_object_from_pairs(pairs)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase8AuxiliarySelector {
    AxiomPolicy {
        normalized_result_hash: Hash,
        checker_profile: String,
        result_hash: Hash,
        axiom_report_hash: Hash,
    },
    Reproducibility {
        request_hash: Hash,
        checker_profile: String,
        baseline_run_artifact_hash: Hash,
        repeated_run_artifact_hash: Hash,
    },
}

impl Phase8AuxiliarySelector {
    pub const fn kind(&self) -> Phase8AuxiliaryResultKind {
        match self {
            Self::AxiomPolicy { .. } => Phase8AuxiliaryResultKind::AxiomPolicy,
            Self::Reproducibility { .. } => Phase8AuxiliaryResultKind::Reproducibility,
        }
    }

    fn canonical_json(&self) -> String {
        match self {
            Self::AxiomPolicy {
                normalized_result_hash,
                checker_profile,
                result_hash,
                axiom_report_hash,
            } => canonical_json_object_from_pairs(vec![
                (
                    "axiom_report_hash".to_owned(),
                    phase8_hash_json_literal(axiom_report_hash),
                ),
                (
                    "checker_profile".to_owned(),
                    phase8_json_string_literal(checker_profile),
                ),
                (
                    "normalized_result_hash".to_owned(),
                    phase8_hash_json_literal(normalized_result_hash),
                ),
                (
                    "result_hash".to_owned(),
                    phase8_hash_json_literal(result_hash),
                ),
            ]),
            Self::Reproducibility {
                request_hash,
                checker_profile,
                baseline_run_artifact_hash,
                repeated_run_artifact_hash,
            } => canonical_json_object_from_pairs(vec![
                (
                    "baseline_run_artifact_hash".to_owned(),
                    phase8_hash_json_literal(baseline_run_artifact_hash),
                ),
                (
                    "checker_profile".to_owned(),
                    phase8_json_string_literal(checker_profile),
                ),
                (
                    "repeated_run_artifact_hash".to_owned(),
                    phase8_hash_json_literal(repeated_run_artifact_hash),
                ),
                (
                    "request_hash".to_owned(),
                    phase8_hash_json_literal(request_hash),
                ),
            ]),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8AuxiliaryResult {
    pub kind: Phase8AuxiliaryResultKind,
    pub result_id: String,
    pub policy_hash: Hash,
    pub artifact_hash: Hash,
    pub selector: Option<Phase8AuxiliarySelector>,
    pub status: Phase8AuxiliaryStatus,
    pub error: Option<Phase8AuxiliaryError>,
    pub diagnostics: Vec<String>,
}

impl Phase8AuxiliaryResult {
    pub fn passed(
        result_id: impl Into<String>,
        kind: Phase8AuxiliaryResultKind,
        policy_hash: Hash,
        artifact_hash: Hash,
        selector: Option<Phase8AuxiliarySelector>,
    ) -> Self {
        Self {
            kind,
            result_id: result_id.into(),
            policy_hash,
            artifact_hash,
            selector,
            status: Phase8AuxiliaryStatus::Passed,
            error: None,
            diagnostics: Vec::new(),
        }
    }

    pub fn failed(
        result_id: impl Into<String>,
        kind: Phase8AuxiliaryResultKind,
        policy_hash: Hash,
        artifact_hash: Hash,
        selector: Option<Phase8AuxiliarySelector>,
        error: Phase8AuxiliaryError,
    ) -> Self {
        Self {
            kind,
            result_id: result_id.into(),
            policy_hash,
            artifact_hash,
            selector,
            status: Phase8AuxiliaryStatus::Failed,
            error: Some(error),
            diagnostics: Vec::new(),
        }
    }

    pub fn inconclusive(
        result_id: impl Into<String>,
        kind: Phase8AuxiliaryResultKind,
        policy_hash: Hash,
        artifact_hash: Hash,
        selector: Option<Phase8AuxiliarySelector>,
        error: Phase8AuxiliaryError,
    ) -> Self {
        Self {
            kind,
            result_id: result_id.into(),
            policy_hash,
            artifact_hash,
            selector,
            status: Phase8AuxiliaryStatus::Inconclusive,
            error: Some(error),
            diagnostics: Vec::new(),
        }
    }

    pub fn result_hash(&self) -> Hash {
        phase8_sha256(self.hash_input_canonical_json().as_bytes())
    }

    pub fn hash_input_canonical_json(&self) -> String {
        canonical_json_object_from_pairs(self.canonical_json_pairs_without_local_identity())
    }

    pub fn canonical_json(&self) -> String {
        let mut pairs = self.canonical_json_pairs_without_local_identity();
        pairs.push((
            "result_hash".to_owned(),
            phase8_hash_json_literal(&self.result_hash()),
        ));
        pairs.push((
            "result_id".to_owned(),
            phase8_json_string_literal(&self.result_id),
        ));
        if !self.diagnostics.is_empty() {
            pairs.push((
                "diagnostics".to_owned(),
                phase8_json_string_array(&self.diagnostics),
            ));
        }
        canonical_json_object_from_pairs(pairs)
    }

    fn canonical_json_pairs_without_local_identity(&self) -> Vec<(String, String)> {
        let mut pairs = vec![
            (
                "artifact_hash".to_owned(),
                phase8_hash_json_literal(&self.artifact_hash),
            ),
            (
                "kind".to_owned(),
                phase8_json_string_literal(self.kind.as_str()),
            ),
            (
                "policy_hash".to_owned(),
                phase8_hash_json_literal(&self.policy_hash),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_AUXILIARY_RESULT_SCHEMA),
            ),
            (
                "status".to_owned(),
                phase8_json_string_literal(self.status.as_str()),
            ),
        ];
        if let Some(selector) = &self.selector {
            pairs.push(("selector".to_owned(), selector.canonical_json()));
        }
        if let Some(error) = &self.error {
            pairs.push(("error".to_owned(), error.canonical_json()));
        }
        pairs
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8AuxiliaryResultStoreEntry {
    pub result_hash: Hash,
    pub kind: Phase8AuxiliaryResultKind,
    pub policy_hash: Hash,
    pub artifact_hash: Hash,
    pub path: String,
    pub file_hash: Hash,
}

impl Phase8AuxiliaryResultStoreEntry {
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
                "kind".to_owned(),
                phase8_json_string_literal(self.kind.as_str()),
            ),
            ("path".to_owned(), phase8_json_string_literal(&self.path)),
            (
                "policy_hash".to_owned(),
                phase8_hash_json_literal(&self.policy_hash),
            ),
            (
                "result_hash".to_owned(),
                phase8_hash_json_literal(&self.result_hash),
            ),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8AuxiliaryResultStoreManifest {
    pub results: Vec<Phase8AuxiliaryResultStoreEntry>,
}

impl Phase8AuxiliaryResultStoreManifest {
    pub fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "results".to_owned(),
                canonical_json_array(
                    self.results
                        .iter()
                        .map(Phase8AuxiliaryResultStoreEntry::canonical_json)
                        .collect(),
                ),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_AUXILIARY_RESULT_STORE_MANIFEST_SCHEMA),
            ),
        ])
    }

    pub fn file_hash(&self) -> Hash {
        phase8_file_hash(self.canonical_json().as_bytes())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8AuxiliaryResultStoreUpdate {
    pub manifest: Phase8AuxiliaryResultStoreManifest,
    pub rewrite_required: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8AiAuditSidecarSourceKind {
    MachineResult,
    NormalizedComparison,
}

impl Phase8AiAuditSidecarSourceKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MachineResult => "machine_result",
            Self::NormalizedComparison => "normalized_comparison",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8AiAuditSidecarStatus {
    Summarized,
    Triaged,
    SuggestedFix,
    SuggestedChallenge,
    Inconclusive,
}

impl Phase8AiAuditSidecarStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Summarized => "summarized",
            Self::Triaged => "triaged",
            Self::SuggestedFix => "suggested_fix",
            Self::SuggestedChallenge => "suggested_challenge",
            Self::Inconclusive => "inconclusive",
        }
    }

    const fn requires_classification(self) -> bool {
        matches!(
            self,
            Self::Triaged | Self::SuggestedFix | Self::SuggestedChallenge
        )
    }

    const fn requires_next_actions(self) -> bool {
        matches!(self, Self::SuggestedFix | Self::SuggestedChallenge)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8AiAuditInputPolicy {
    pub id: String,
    pub version: u64,
    pub included_fields: Vec<String>,
    pub redaction: String,
    pub allow_source_text: bool,
    pub allow_tactic_trace: bool,
}

impl Phase8AiAuditInputPolicy {
    pub fn input_policy_hash(&self) -> Hash {
        phase8_sha256(self.canonical_json().as_bytes())
    }

    pub fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            (
                "allow_source_text".to_owned(),
                self.allow_source_text.to_string(),
            ),
            (
                "allow_tactic_trace".to_owned(),
                self.allow_tactic_trace.to_string(),
            ),
            ("id".to_owned(), phase8_json_string_literal(&self.id)),
            (
                "included_fields".to_owned(),
                phase8_json_string_array(&self.included_fields),
            ),
            (
                "redaction".to_owned(),
                phase8_json_string_literal(&self.redaction),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_AI_AUDIT_INPUT_POLICY_SCHEMA),
            ),
            ("version".to_owned(), self.version.to_string()),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8AiAuditSidecarSource {
    pub kind: Phase8AiAuditSidecarSourceKind,
    pub result_hash: Option<Hash>,
    pub request_hash: Option<Hash>,
    pub run_artifact_hash: Option<Hash>,
    pub normalized_result_hash: Option<Hash>,
    pub result_id: Option<String>,
    pub normalized_result_id: Option<String>,
}

impl Phase8AiAuditSidecarSource {
    fn canonical_json(&self) -> String {
        let mut pairs = vec![(
            "kind".to_owned(),
            phase8_json_string_literal(self.kind.as_str()),
        )];
        push_optional_hash_pair(&mut pairs, "result_hash", self.result_hash);
        push_optional_hash_pair(&mut pairs, "request_hash", self.request_hash);
        push_optional_hash_pair(&mut pairs, "run_artifact_hash", self.run_artifact_hash);
        push_optional_hash_pair(
            &mut pairs,
            "normalized_result_hash",
            self.normalized_result_hash,
        );
        push_optional_string_pair(&mut pairs, "result_id", self.result_id.as_deref());
        push_optional_string_pair(
            &mut pairs,
            "normalized_result_id",
            self.normalized_result_id.as_deref(),
        );
        canonical_json_object_from_pairs(pairs)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8AiAuditSidecarInputPolicy {
    pub id: String,
    pub version: u64,
    pub hash: Hash,
    pub included_fields: Vec<String>,
    pub redaction: String,
}

impl Phase8AiAuditSidecarInputPolicy {
    fn from_policy(policy: &Phase8AiAuditInputPolicy) -> Self {
        Self {
            id: policy.id.clone(),
            version: policy.version,
            hash: policy.input_policy_hash(),
            included_fields: policy.included_fields.clone(),
            redaction: policy.redaction.clone(),
        }
    }

    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            ("hash".to_owned(), phase8_hash_json_literal(&self.hash)),
            ("id".to_owned(), phase8_json_string_literal(&self.id)),
            (
                "included_fields".to_owned(),
                phase8_json_string_array(&self.included_fields),
            ),
            (
                "redaction".to_owned(),
                phase8_json_string_literal(&self.redaction),
            ),
            ("version".to_owned(), self.version.to_string()),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8AiAuditSidecarAi {
    pub agent: String,
    pub model: String,
    pub prompt_hash: Hash,
}

impl Phase8AiAuditSidecarAi {
    fn canonical_json(&self) -> String {
        canonical_json_object_from_pairs(vec![
            ("agent".to_owned(), phase8_json_string_literal(&self.agent)),
            ("model".to_owned(), phase8_json_string_literal(&self.model)),
            (
                "prompt_hash".to_owned(),
                phase8_hash_json_literal(&self.prompt_hash),
            ),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8AiAuditSidecarClassification {
    pub category: String,
    pub confidence: String,
    pub checker_error_kind: Option<String>,
}

impl Phase8AiAuditSidecarClassification {
    fn canonical_json(&self) -> String {
        let mut pairs = vec![
            (
                "category".to_owned(),
                phase8_json_string_literal(&self.category),
            ),
            (
                "confidence".to_owned(),
                phase8_json_string_literal(&self.confidence),
            ),
        ];
        push_optional_string_pair(
            &mut pairs,
            "checker_error_kind",
            self.checker_error_kind.as_deref(),
        );
        canonical_json_object_from_pairs(pairs)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Phase8AiAuditPolicyGatedFieldValue {
    String(String),
    Strings(Vec<String>),
}

impl Phase8AiAuditPolicyGatedFieldValue {
    fn canonical_json(&self) -> String {
        match self {
            Self::String(value) => phase8_json_string_literal(value),
            Self::Strings(values) => phase8_json_string_array(values),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8AiAuditSidecar {
    pub source: Phase8AiAuditSidecarSource,
    pub input_policy: Phase8AiAuditSidecarInputPolicy,
    pub ai: Phase8AiAuditSidecarAi,
    pub status: Phase8AiAuditSidecarStatus,
    pub classification: Option<Phase8AiAuditSidecarClassification>,
    pub summary: String,
    pub suggested_next_actions: Option<Vec<String>>,
    policy_gated_fields: BTreeMap<String, Phase8AiAuditPolicyGatedFieldValue>,
}

impl Phase8AiAuditSidecar {
    pub fn canonical_json(&self) -> String {
        let mut pairs = vec![
            ("ai".to_owned(), self.ai.canonical_json()),
            (
                "input_policy".to_owned(),
                self.input_policy.canonical_json(),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_AI_AUDIT_SIDECAR_SCHEMA),
            ),
            ("source".to_owned(), self.source.canonical_json()),
            (
                "status".to_owned(),
                phase8_json_string_literal(self.status.as_str()),
            ),
            (
                "summary".to_owned(),
                phase8_json_string_literal(&self.summary),
            ),
        ];
        if let Some(classification) = &self.classification {
            pairs.push(("classification".to_owned(), classification.canonical_json()));
        }
        if let Some(actions) = &self.suggested_next_actions {
            pairs.push((
                "suggested_next_actions".to_owned(),
                phase8_json_string_array(actions),
            ));
        }
        for (field, value) in &self.policy_gated_fields {
            pairs.push((field.clone(), value.canonical_json()));
        }
        canonical_json_object_from_pairs(pairs)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8AuditSidecarValidationMode {
    SchemaOnly,
    CrossArtifact,
}

impl Phase8AuditSidecarValidationMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SchemaOnly => "schema_only",
            Self::CrossArtifact => "cross_artifact",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8AuditSidecarValidationStatus {
    Valid,
    Failed,
}

impl Phase8AuditSidecarValidationStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Valid => "valid",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8AuditSidecarValidationReasonCode {
    SidecarFileUnreadable,
    SidecarJsonInvalid,
    SidecarSchemaInvalid,
    ForbiddenSidecarField,
    ValidationReferenceMissing,
    ValidationReferenceSchemaInvalid,
    InputPolicyFileUnreadable,
    InputPolicyJsonInvalid,
    InputPolicySchemaInvalid,
    InputPolicyHashMismatch,
    InputPolicyFieldMismatch,
    ResultStoreManifestHashMismatch,
    ResultStoreManifestInvalid,
    NormalizedStoreManifestHashMismatch,
    NormalizedStoreManifestInvalid,
    ReferencedFileHashMismatch,
    ReferencedArtifactHashMismatch,
    ReferencedArtifactValueMismatch,
    SourceResultNotFound,
    SourceNormalizedResultNotFound,
    SourceHashMismatch,
    SourceIdMismatch,
    NormalizedResultMissingSource,
    PromptHashMismatch,
}

impl Phase8AuditSidecarValidationReasonCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SidecarFileUnreadable => "sidecar_file_unreadable",
            Self::SidecarJsonInvalid => "sidecar_json_invalid",
            Self::SidecarSchemaInvalid => "sidecar_schema_invalid",
            Self::ForbiddenSidecarField => "forbidden_sidecar_field",
            Self::ValidationReferenceMissing => "validation_reference_missing",
            Self::ValidationReferenceSchemaInvalid => "validation_reference_schema_invalid",
            Self::InputPolicyFileUnreadable => "input_policy_file_unreadable",
            Self::InputPolicyJsonInvalid => "input_policy_json_invalid",
            Self::InputPolicySchemaInvalid => "input_policy_schema_invalid",
            Self::InputPolicyHashMismatch => "input_policy_hash_mismatch",
            Self::InputPolicyFieldMismatch => "input_policy_field_mismatch",
            Self::ResultStoreManifestHashMismatch => "result_store_manifest_hash_mismatch",
            Self::ResultStoreManifestInvalid => "result_store_manifest_invalid",
            Self::NormalizedStoreManifestHashMismatch => "normalized_store_manifest_hash_mismatch",
            Self::NormalizedStoreManifestInvalid => "normalized_store_manifest_invalid",
            Self::ReferencedFileHashMismatch => "referenced_file_hash_mismatch",
            Self::ReferencedArtifactHashMismatch => "referenced_artifact_hash_mismatch",
            Self::ReferencedArtifactValueMismatch => "referenced_artifact_value_mismatch",
            Self::SourceResultNotFound => "source_result_not_found",
            Self::SourceNormalizedResultNotFound => "source_normalized_result_not_found",
            Self::SourceHashMismatch => "source_hash_mismatch",
            Self::SourceIdMismatch => "source_id_mismatch",
            Self::NormalizedResultMissingSource => "normalized_result_missing_source",
            Self::PromptHashMismatch => "prompt_hash_mismatch",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8AuditSidecarValidationError {
    pub reason_code: Phase8AuditSidecarValidationReasonCode,
    pub field: String,
    pub expected_hash: Option<Box<Hash>>,
    pub actual_hash: Option<Box<Hash>>,
    pub expected_value: Option<Box<str>>,
    pub actual_value: Option<Box<str>>,
}

impl Phase8AuditSidecarValidationError {
    fn value(
        reason_code: Phase8AuditSidecarValidationReasonCode,
        field: impl Into<String>,
        expected_value: impl Into<String>,
        actual_value: impl Into<String>,
    ) -> Self {
        Self {
            reason_code,
            field: field.into(),
            expected_hash: None,
            actual_hash: None,
            expected_value: Some(expected_value.into().into_boxed_str()),
            actual_value: Some(actual_value.into().into_boxed_str()),
        }
    }

    fn actual_value(
        reason_code: Phase8AuditSidecarValidationReasonCode,
        field: impl Into<String>,
        actual_value: impl Into<String>,
    ) -> Self {
        Self {
            reason_code,
            field: field.into(),
            expected_hash: None,
            actual_hash: None,
            expected_value: None,
            actual_value: Some(actual_value.into().into_boxed_str()),
        }
    }

    fn expected_hash(
        reason_code: Phase8AuditSidecarValidationReasonCode,
        field: impl Into<String>,
        expected_hash: Hash,
    ) -> Self {
        Self {
            reason_code,
            field: field.into(),
            expected_hash: Some(Box::new(expected_hash)),
            actual_hash: None,
            expected_value: None,
            actual_value: None,
        }
    }

    fn hash(
        reason_code: Phase8AuditSidecarValidationReasonCode,
        field: impl Into<String>,
        expected_hash: Hash,
        actual_hash: Hash,
    ) -> Self {
        Self {
            reason_code,
            field: field.into(),
            expected_hash: Some(Box::new(expected_hash)),
            actual_hash: Some(Box::new(actual_hash)),
            expected_value: None,
            actual_value: None,
        }
    }

    fn canonical_json(&self) -> String {
        let mut pairs = vec![
            ("field".to_owned(), phase8_json_string_literal(&self.field)),
            (
                "kind".to_owned(),
                phase8_json_string_literal("audit_sidecar_validation_failure"),
            ),
            (
                "reason_code".to_owned(),
                phase8_json_string_literal(self.reason_code.as_str()),
            ),
        ];
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
        canonical_json_object_from_pairs(pairs)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8AuditSidecarValidationResult {
    pub mode: Phase8AuditSidecarValidationMode,
    pub status: Phase8AuditSidecarValidationStatus,
    pub sidecar_file_hash: Option<Hash>,
    pub input_policy_hash: Option<Hash>,
    pub source_kind: Option<Phase8AiAuditSidecarSourceKind>,
    pub source_result_hash: Option<Hash>,
    pub source_normalized_result_hash: Option<Hash>,
    pub error: Option<Phase8AuditSidecarValidationError>,
}

impl Phase8AuditSidecarValidationResult {
    fn valid(
        mode: Phase8AuditSidecarValidationMode,
        sidecar_file_hash: Hash,
        input_policy_hash: Option<Hash>,
        sidecar: Option<&Phase8AiAuditSidecar>,
    ) -> Self {
        Self {
            mode,
            status: Phase8AuditSidecarValidationStatus::Valid,
            sidecar_file_hash: Some(sidecar_file_hash),
            input_policy_hash,
            source_kind: (mode == Phase8AuditSidecarValidationMode::CrossArtifact)
                .then(|| sidecar.map(|sidecar| sidecar.source.kind))
                .flatten(),
            source_result_hash: (mode == Phase8AuditSidecarValidationMode::CrossArtifact)
                .then(|| sidecar.and_then(|sidecar| sidecar.source.result_hash))
                .flatten(),
            source_normalized_result_hash: (mode
                == Phase8AuditSidecarValidationMode::CrossArtifact)
                .then(|| sidecar.and_then(|sidecar| sidecar.source.normalized_result_hash))
                .flatten(),
            error: None,
        }
    }

    fn failed(
        mode: Phase8AuditSidecarValidationMode,
        sidecar_file_hash: Option<Hash>,
        input_policy_hash: Option<Hash>,
        sidecar: Option<&Phase8AiAuditSidecar>,
        error: Phase8AuditSidecarValidationError,
    ) -> Self {
        Self {
            mode,
            status: Phase8AuditSidecarValidationStatus::Failed,
            sidecar_file_hash,
            input_policy_hash,
            source_kind: (mode == Phase8AuditSidecarValidationMode::CrossArtifact)
                .then(|| sidecar.map(|sidecar| sidecar.source.kind))
                .flatten(),
            source_result_hash: (mode == Phase8AuditSidecarValidationMode::CrossArtifact)
                .then(|| sidecar.and_then(|sidecar| sidecar.source.result_hash))
                .flatten(),
            source_normalized_result_hash: (mode
                == Phase8AuditSidecarValidationMode::CrossArtifact)
                .then(|| sidecar.and_then(|sidecar| sidecar.source.normalized_result_hash))
                .flatten(),
            error: Some(error),
        }
    }

    pub fn canonical_json(&self) -> String {
        let mut pairs = vec![
            (
                "mode".to_owned(),
                phase8_json_string_literal(self.mode.as_str()),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_AUDIT_SIDECAR_VALIDATION_RESULT_SCHEMA),
            ),
            (
                "status".to_owned(),
                phase8_json_string_literal(self.status.as_str()),
            ),
        ];
        if let Some(error) = &self.error {
            pairs.push(("error".to_owned(), error.canonical_json()));
        }
        push_optional_hash_pair(&mut pairs, "sidecar_file_hash", self.sidecar_file_hash);
        push_optional_hash_pair(&mut pairs, "input_policy_hash", self.input_policy_hash);
        if let Some(source_kind) = self.source_kind {
            pairs.push((
                "source_kind".to_owned(),
                phase8_json_string_literal(source_kind.as_str()),
            ));
        }
        push_optional_hash_pair(&mut pairs, "source_result_hash", self.source_result_hash);
        push_optional_hash_pair(
            &mut pairs,
            "source_normalized_result_hash",
            self.source_normalized_result_hash,
        );
        canonical_json_object_from_pairs(pairs)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8AiSidecarDiagnosticStatus {
    Passed,
    Failed,
}

impl Phase8AiSidecarDiagnosticStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8RequiredAiSidecarDiagnosticTargetKind {
    MachineResult,
    NormalizedComparison,
}

impl Phase8RequiredAiSidecarDiagnosticTargetKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MachineResult => "machine_result",
            Self::NormalizedComparison => "normalized_comparison",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase8RequiredAiSidecarDiagnosticTarget {
    MachineResult {
        result_index: usize,
        normalized_result_hash: Hash,
        checker_profile: String,
        request_hash: Hash,
        result_hash: Hash,
        policy_hash: Hash,
    },
    NormalizedComparison {
        normalized_result_hash: Hash,
    },
}

impl Phase8RequiredAiSidecarDiagnosticTarget {
    pub const fn kind(&self) -> Phase8RequiredAiSidecarDiagnosticTargetKind {
        match self {
            Self::MachineResult { .. } => {
                Phase8RequiredAiSidecarDiagnosticTargetKind::MachineResult
            }
            Self::NormalizedComparison { .. } => {
                Phase8RequiredAiSidecarDiagnosticTargetKind::NormalizedComparison
            }
        }
    }

    pub const fn normalized_result_hash(&self) -> Hash {
        match self {
            Self::MachineResult {
                normalized_result_hash,
                ..
            }
            | Self::NormalizedComparison {
                normalized_result_hash,
            } => *normalized_result_hash,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8AiSidecarDiagnosticFailureReasonCode {
    RequiredAiSidecarSelectedRawMissing,
    RequiredAiSidecarSelectedRawDuplicate,
    RequiredAiSidecarMissing,
    RequiredAiSidecarDuplicate,
    RequiredAiSidecarInputPolicyMismatch,
    RequiredAiSidecarValidationMissing,
    RequiredAiSidecarValidationDuplicate,
    RequiredAiSidecarValidationModeMismatch,
    RequiredAiSidecarValidationFailed,
    RequiredAiSidecarValidationInputPolicyMismatch,
    RequiredAiSidecarValidationSourceMismatch,
    RequiredAiSidecarSourceMismatch,
}

impl Phase8AiSidecarDiagnosticFailureReasonCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequiredAiSidecarSelectedRawMissing => "required_ai_sidecar_selected_raw_missing",
            Self::RequiredAiSidecarSelectedRawDuplicate => {
                "required_ai_sidecar_selected_raw_duplicate"
            }
            Self::RequiredAiSidecarMissing => "required_ai_sidecar_missing",
            Self::RequiredAiSidecarDuplicate => "required_ai_sidecar_duplicate",
            Self::RequiredAiSidecarInputPolicyMismatch => {
                "required_ai_sidecar_input_policy_mismatch"
            }
            Self::RequiredAiSidecarValidationMissing => "required_ai_sidecar_validation_missing",
            Self::RequiredAiSidecarValidationDuplicate => {
                "required_ai_sidecar_validation_duplicate"
            }
            Self::RequiredAiSidecarValidationModeMismatch => {
                "required_ai_sidecar_validation_mode_mismatch"
            }
            Self::RequiredAiSidecarValidationFailed => "required_ai_sidecar_validation_failed",
            Self::RequiredAiSidecarValidationInputPolicyMismatch => {
                "required_ai_sidecar_validation_input_policy_mismatch"
            }
            Self::RequiredAiSidecarValidationSourceMismatch => {
                "required_ai_sidecar_validation_source_mismatch"
            }
            Self::RequiredAiSidecarSourceMismatch => "required_ai_sidecar_source_mismatch",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8AiSidecarDiagnosticError {
    pub reason_code: Phase8AiSidecarDiagnosticFailureReasonCode,
    pub field: String,
    pub expected_hash: Option<Hash>,
    pub actual_hash: Option<Hash>,
    pub expected_value: Option<String>,
    pub actual_value: Option<String>,
}

impl Phase8AiSidecarDiagnosticError {
    fn value(
        reason_code: Phase8AiSidecarDiagnosticFailureReasonCode,
        field: impl Into<String>,
        expected_value: impl Into<String>,
        actual_value: impl Into<String>,
    ) -> Self {
        Self {
            reason_code,
            field: field.into(),
            expected_hash: None,
            actual_hash: None,
            expected_value: Some(expected_value.into()),
            actual_value: Some(actual_value.into()),
        }
    }

    fn hash(
        reason_code: Phase8AiSidecarDiagnosticFailureReasonCode,
        field: impl Into<String>,
        expected_hash: Hash,
        actual_hash: Hash,
    ) -> Self {
        Self {
            reason_code,
            field: field.into(),
            expected_hash: Some(expected_hash),
            actual_hash: Some(actual_hash),
            expected_value: None,
            actual_value: None,
        }
    }

    fn canonical_json(&self) -> String {
        let mut pairs = vec![
            ("field".to_owned(), phase8_json_string_literal(&self.field)),
            (
                "kind".to_owned(),
                phase8_json_string_literal("ai_sidecar_diagnostic_failure"),
            ),
            (
                "reason_code".to_owned(),
                phase8_json_string_literal(self.reason_code.as_str()),
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
pub struct Phase8AiSidecarDiagnosticResult {
    pub policy_hash: Hash,
    pub input_policy_hash: Hash,
    pub normalized_result_hash: Hash,
    pub status: Phase8AiSidecarDiagnosticStatus,
    pub target_count: u64,
    pub error: Option<Phase8AiSidecarDiagnosticError>,
}

impl Phase8AiSidecarDiagnosticResult {
    fn passed(
        policy_hash: Hash,
        input_policy_hash: Hash,
        normalized_result_hash: Hash,
        target_count: u64,
    ) -> Self {
        Self {
            policy_hash,
            input_policy_hash,
            normalized_result_hash,
            status: Phase8AiSidecarDiagnosticStatus::Passed,
            target_count,
            error: None,
        }
    }

    fn failed(
        policy_hash: Hash,
        input_policy_hash: Hash,
        normalized_result_hash: Hash,
        target_count: u64,
        error: Phase8AiSidecarDiagnosticError,
    ) -> Self {
        Self {
            policy_hash,
            input_policy_hash,
            normalized_result_hash,
            status: Phase8AiSidecarDiagnosticStatus::Failed,
            target_count,
            error: Some(error),
        }
    }

    pub fn canonical_json(&self) -> String {
        let mut pairs = vec![
            (
                "input_policy_hash".to_owned(),
                phase8_hash_json_literal(&self.input_policy_hash),
            ),
            (
                "normalized_result_hash".to_owned(),
                phase8_hash_json_literal(&self.normalized_result_hash),
            ),
            (
                "policy_hash".to_owned(),
                phase8_hash_json_literal(&self.policy_hash),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_AI_SIDECAR_DIAGNOSTIC_RESULT_SCHEMA),
            ),
            (
                "status".to_owned(),
                phase8_json_string_literal(self.status.as_str()),
            ),
            ("target_count".to_owned(), self.target_count.to_string()),
        ];
        if let Some(error) = &self.error {
            pairs.push(("error".to_owned(), error.canonical_json()));
        }
        canonical_json_object_from_pairs(pairs)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8AiSidecarDiagnosticEvaluationFailureReasonCode {
    AiSidecarInputUnreadable,
    AiSidecarInputHashMismatch,
    AiSidecarInputJsonInvalid,
    AiSidecarInputSchemaInvalid,
    AuditSidecarValidationInputUnreadable,
    AuditSidecarValidationInputHashMismatch,
    AuditSidecarValidationInputJsonInvalid,
    AuditSidecarValidationInputSchemaInvalid,
    AiSidecarDiagnosticInputUnreadable,
    AiSidecarDiagnosticInputHashMismatch,
    AiSidecarDiagnosticInputJsonInvalid,
    AiSidecarDiagnosticInputSchemaInvalid,
    CiDiagnosticTargetDuplicate,
    AiSidecarDiagnosticTargetCountMismatch,
}

impl Phase8AiSidecarDiagnosticEvaluationFailureReasonCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AiSidecarInputUnreadable => "ai_sidecar_input_unreadable",
            Self::AiSidecarInputHashMismatch => "ai_sidecar_input_hash_mismatch",
            Self::AiSidecarInputJsonInvalid => "ai_sidecar_input_json_invalid",
            Self::AiSidecarInputSchemaInvalid => "ai_sidecar_input_schema_invalid",
            Self::AuditSidecarValidationInputUnreadable => {
                "audit_sidecar_validation_input_unreadable"
            }
            Self::AuditSidecarValidationInputHashMismatch => {
                "audit_sidecar_validation_input_hash_mismatch"
            }
            Self::AuditSidecarValidationInputJsonInvalid => {
                "audit_sidecar_validation_input_json_invalid"
            }
            Self::AuditSidecarValidationInputSchemaInvalid => {
                "audit_sidecar_validation_input_schema_invalid"
            }
            Self::AiSidecarDiagnosticInputUnreadable => "ai_sidecar_diagnostic_input_unreadable",
            Self::AiSidecarDiagnosticInputHashMismatch => {
                "ai_sidecar_diagnostic_input_hash_mismatch"
            }
            Self::AiSidecarDiagnosticInputJsonInvalid => "ai_sidecar_diagnostic_input_json_invalid",
            Self::AiSidecarDiagnosticInputSchemaInvalid => {
                "ai_sidecar_diagnostic_input_schema_invalid"
            }
            Self::CiDiagnosticTargetDuplicate => "ci_diagnostic_target_duplicate",
            Self::AiSidecarDiagnosticTargetCountMismatch => {
                "ai_sidecar_diagnostic_target_count_mismatch"
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8AiSidecarDiagnosticEvaluationFailure {
    pub reason_code: Phase8AiSidecarDiagnosticEvaluationFailureReasonCode,
    pub field: Box<str>,
    pub expected_hash: Option<Box<Hash>>,
    pub actual_hash: Option<Box<Hash>>,
    pub expected_value: Option<Box<str>>,
    pub actual_value: Option<Box<str>>,
}

impl Phase8AiSidecarDiagnosticEvaluationFailure {
    fn value(
        reason_code: Phase8AiSidecarDiagnosticEvaluationFailureReasonCode,
        field: impl Into<String>,
        expected_value: impl Into<String>,
        actual_value: impl Into<String>,
    ) -> Self {
        Self {
            reason_code,
            field: field.into().into_boxed_str(),
            expected_hash: None,
            actual_hash: None,
            expected_value: Some(expected_value.into().into_boxed_str()),
            actual_value: Some(actual_value.into().into_boxed_str()),
        }
    }

    pub fn hash(
        reason_code: Phase8AiSidecarDiagnosticEvaluationFailureReasonCode,
        field: impl Into<String>,
        expected_hash: Hash,
        actual_hash: Hash,
    ) -> Self {
        Self {
            reason_code,
            field: field.into().into_boxed_str(),
            expected_hash: Some(Box::new(expected_hash)),
            actual_hash: Some(Box::new(actual_hash)),
            expected_value: None,
            actual_value: None,
        }
    }

    pub fn canonical_json(&self) -> String {
        let mut pairs = vec![
            ("field".to_owned(), phase8_json_string_literal(&self.field)),
            (
                "reason_code".to_owned(),
                phase8_json_string_literal(self.reason_code.as_str()),
            ),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_AI_SIDECAR_DIAGNOSTIC_EVALUATION_FAILURE_SCHEMA),
            ),
            ("status".to_owned(), phase8_json_string_literal("failed")),
        ];
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
        canonical_json_object_from_pairs(pairs)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase8AiSidecarDiagnosticPassFailureReasonCode {
    RequiredAiSidecarDiagnosticMissing,
    RequiredAiSidecarDiagnosticDuplicate,
    RequiredAiSidecarDiagnosticNotPassed,
    RequiredAiSidecarDiagnosticCanonicalMismatch,
}

impl Phase8AiSidecarDiagnosticPassFailureReasonCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequiredAiSidecarDiagnosticMissing => "required_ai_sidecar_diagnostic_missing",
            Self::RequiredAiSidecarDiagnosticDuplicate => {
                "required_ai_sidecar_diagnostic_duplicate"
            }
            Self::RequiredAiSidecarDiagnosticNotPassed => {
                "required_ai_sidecar_diagnostic_not_passed"
            }
            Self::RequiredAiSidecarDiagnosticCanonicalMismatch => {
                "required_ai_sidecar_diagnostic_canonical_mismatch"
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8AiSidecarDiagnosticPassFailure {
    pub reason_code: Phase8AiSidecarDiagnosticPassFailureReasonCode,
    pub field: String,
    pub expected_hash: Option<Hash>,
    pub actual_hash: Option<Hash>,
    pub expected_value: Option<String>,
    pub actual_value: Option<String>,
}

impl Phase8AiSidecarDiagnosticPassFailure {
    fn value(
        reason_code: Phase8AiSidecarDiagnosticPassFailureReasonCode,
        field: impl Into<String>,
        expected_value: impl Into<String>,
        actual_value: impl Into<String>,
    ) -> Self {
        Self {
            reason_code,
            field: field.into(),
            expected_hash: None,
            actual_hash: None,
            expected_value: Some(expected_value.into()),
            actual_value: Some(actual_value.into()),
        }
    }

    fn hash(
        reason_code: Phase8AiSidecarDiagnosticPassFailureReasonCode,
        field: impl Into<String>,
        expected_hash: Hash,
        actual_hash: Hash,
    ) -> Self {
        Self {
            reason_code,
            field: field.into(),
            expected_hash: Some(expected_hash),
            actual_hash: Some(actual_hash),
            expected_value: None,
            actual_value: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8CiDiagnosticTargetContext {
    pub artifact: Phase8NormalizedCheckResult,
    pub normalizer_machine_results: Vec<Phase8MachineCheckResult>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ResolvedAiAuditSidecarEntry {
    pub path: String,
    pub file_hash: Hash,
    pub artifact: Phase8AiAuditSidecar,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ResolvedAuditSidecarValidationEntry {
    pub path: String,
    pub file_hash: Hash,
    pub artifact: Phase8AuditSidecarValidationResult,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8ResolvedAiSidecarDiagnosticResultEntry {
    pub path: String,
    pub file_hash: Hash,
    pub artifact: Phase8AiSidecarDiagnosticResult,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8RequiredAiSidecarDiagnosticEvaluation {
    pub recomputed_results: Vec<Phase8AiSidecarDiagnosticResult>,
    pub pass_failure: Option<Phase8AiSidecarDiagnosticPassFailure>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase8AiAuditPromptInput {
    pub agent: String,
    pub model: String,
    pub source_kind: Phase8AiAuditSidecarSourceKind,
    pub source_hash: Hash,
    pub source_run_artifact_hash: Option<Hash>,
    pub source_membership_hash: Option<Hash>,
    pub input_policy: Phase8AiAuditSidecarInputPolicy,
    pub fields: Vec<(String, String)>,
}

impl Phase8AiAuditPromptInput {
    pub fn prompt_hash(&self) -> Hash {
        phase8_sha256(self.canonical_json().as_bytes())
    }

    pub fn canonical_json(&self) -> String {
        let fields = canonical_json_object_from_pairs(self.fields.clone());
        let mut pairs = vec![
            ("agent".to_owned(), phase8_json_string_literal(&self.agent)),
            ("fields".to_owned(), fields),
            (
                "input_policy".to_owned(),
                self.input_policy.canonical_json(),
            ),
            ("model".to_owned(), phase8_json_string_literal(&self.model)),
            (
                "schema".to_owned(),
                phase8_json_string_literal(PHASE8_AI_AUDIT_PROMPT_INPUT_SCHEMA),
            ),
            (
                "source_hash".to_owned(),
                phase8_hash_json_literal(&self.source_hash),
            ),
            (
                "source_kind".to_owned(),
                phase8_json_string_literal(self.source_kind.as_str()),
            ),
        ];
        push_optional_hash_pair(
            &mut pairs,
            "source_run_artifact_hash",
            self.source_run_artifact_hash,
        );
        push_optional_hash_pair(
            &mut pairs,
            "source_membership_hash",
            self.source_membership_hash,
        );
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

pub fn parse_phase8_challenge_generation_request(
    source: &str,
) -> Result<Phase8ChallengeGenerationRequest, Phase8RequestValidationError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        Phase8RequestValidationError::value_failure(
            "generation_request",
            "valid_json",
            "invalid_json",
        )
    })?;
    parse_phase8_challenge_generation_request_value(document.root())
}

pub fn parse_phase8_challenge_manifest(
    source: &str,
) -> Result<Phase8ChallengeManifest, Phase8RequestValidationError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        Phase8RequestValidationError::value_failure(
            "challenge_manifest",
            "valid_json",
            "invalid_json",
        )
    })?;
    parse_phase8_challenge_manifest_value(document.root(), "$")
}

pub fn parse_phase8_challenge_output_store_manifest(
    source: &str,
) -> Result<Phase8ChallengeOutputStoreManifest, Phase8RequestValidationError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        Phase8RequestValidationError::value_failure(
            "challenge_output_store",
            "valid_json",
            "invalid_json",
        )
    })?;
    parse_phase8_challenge_output_store_manifest_value(document.root(), "challenge_output_store")
}

pub fn parse_phase8_challenge_replay_result(
    source: &str,
) -> Result<Phase8ChallengeReplayResult, Phase8RequestValidationError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        Phase8RequestValidationError::value_failure(
            "challenge_replay_result",
            "valid_json",
            "invalid_json",
        )
    })?;
    parse_phase8_challenge_replay_result_value(document.root(), "$")
}

pub fn parse_phase8_challenge_replay_store_manifest(
    source: &str,
) -> Result<Phase8ChallengeReplayStoreManifest, Phase8RequestValidationError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        Phase8RequestValidationError::value_failure(
            "challenge_replay_store",
            "valid_json",
            "invalid_json",
        )
    })?;
    parse_phase8_challenge_replay_store_manifest_value(document.root(), "challenge_replay_store")
}

pub fn parse_phase8_challenge_coverage_summary(
    source: &str,
) -> Result<Phase8ChallengeCoverageSummary, Phase8RequestValidationError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        Phase8RequestValidationError::value_failure(
            "challenge_coverage_summary",
            "valid_json",
            "invalid_json",
        )
    })?;
    parse_phase8_challenge_coverage_summary_value(document.root(), "$")
}

pub fn phase8_machine_check_request_hash(
    source: &str,
) -> Result<Hash, Phase8RequestValidationError> {
    Ok(parse_phase8_machine_check_request(source)?.request_hash())
}

pub fn phase8_challenge_generation_request_hash(
    source: &str,
) -> Result<Hash, Phase8RequestValidationError> {
    Ok(parse_phase8_challenge_generation_request(source)?.request_hash())
}

pub fn phase8_challenge_replay_result_hash(
    source: &str,
) -> Result<Hash, Phase8RequestValidationError> {
    Ok(parse_phase8_challenge_replay_result(source)?.result_hash())
}

pub fn phase8_challenge_coverage_summary_hash(
    source: &str,
) -> Result<Hash, Phase8RequestValidationError> {
    Ok(parse_phase8_challenge_coverage_summary(source)?.summary_hash())
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
    phase8_request_store_with_materialized_entries(
        Phase8CommandName::RequestMaterialize,
        existing_store,
        std::slice::from_ref(&generated_entry),
    )
}

fn phase8_request_store_with_materialized_entries(
    command: Phase8CommandName,
    existing_store: Option<&Phase8RequestStoreManifest>,
    generated_entries: &[Phase8RequestStoreEntry],
) -> Result<Phase8RequestStoreUpdate, Phase8CommandError> {
    let mut manifest = existing_store
        .cloned()
        .unwrap_or(Phase8RequestStoreManifest {
            requests: Vec::new(),
        });
    let mut rewrite_required = false;

    for generated_entry in generated_entries {
        let mut exact_existing = false;
        for existing in &manifest.requests {
            let same_request_hash = existing.request_hash == generated_entry.request_hash;
            let same_path = existing.path == generated_entry.path;
            let same_file_hash = existing.file_hash == generated_entry.file_hash;
            if same_request_hash && same_path && same_file_hash {
                exact_existing = true;
                break;
            }
            if same_request_hash || same_path {
                return Err(phase8_command_value_error(
                    command,
                    "request_store_entry_conflict",
                    "request_store.requests[]",
                    generated_entry.canonical_json(),
                    existing.canonical_json(),
                ));
            }
        }
        if exact_existing {
            continue;
        }
        manifest.requests.push(generated_entry.clone());
        rewrite_required = true;
    }

    manifest
        .requests
        .sort_by(|left, right| left.request_hash.cmp(&right.request_hash));
    Ok(Phase8RequestStoreUpdate {
        manifest,
        rewrite_required,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn phase8_challenge_generate(
    request: &Phase8ChallengeGenerationRequest,
    policy: &Phase8RunnerPolicy,
    base_certificate_bytes: &[u8],
    imports_manifest_bytes: &[u8],
    existing_store: Option<&Phase8ChallengeOutputStoreManifest>,
) -> Result<Phase8ChallengeGeneration, Phase8CommandError> {
    validate_phase8_challenge_generation_request_domain(request).map_err(|error| {
        phase8_command_error_from_request_validation(
            Phase8CommandName::ChallengeGenerate,
            "generation_request_schema_invalid",
            error.field.to_string(),
            error,
        )
    })?;
    let request_hash = request.request_hash();
    let policy_hash = policy.policy_hash();
    if request.policy_hash != policy_hash {
        return Err(phase8_command_hash_error(
            Phase8CommandName::ChallengeGenerate,
            "policy_hash_mismatch",
            "policy_hash",
            policy_hash,
            request.policy_hash,
        ));
    }
    if request.imports.mode != policy.import_policy.mode {
        return Err(phase8_command_value_error(
            Phase8CommandName::ChallengeGenerate,
            "import_mode_mismatch",
            "imports.mode",
            &policy.import_policy.mode,
            &request.imports.mode,
        ));
    }
    let actual_import_hash = phase8_file_hash(imports_manifest_bytes);
    if actual_import_hash != request.imports.manifest_hash {
        return Err(phase8_command_hash_error(
            Phase8CommandName::ChallengeGenerate,
            "import_manifest_hash_mismatch",
            "imports.manifest_hash",
            request.imports.manifest_hash,
            actual_import_hash,
        ));
    }
    let imports_source = std::str::from_utf8(imports_manifest_bytes).map_err(|_| {
        phase8_command_value_error(
            Phase8CommandName::ChallengeGenerate,
            "import_manifest_file_unreadable",
            "imports.manifest",
            "valid_utf8_file",
            "unreadable",
        )
    })?;
    parse_phase8_import_lock_manifest(imports_source).map_err(|error| {
        let field = error.field.to_string();
        phase8_command_error_from_request_validation(
            Phase8CommandName::ChallengeGenerate,
            "generation_request_schema_invalid",
            field,
            error,
        )
    })?;

    let actual_base_file_hash = phase8_file_hash(base_certificate_bytes);
    if actual_base_file_hash != request.base_certificate.file_hash {
        return Err(phase8_command_hash_error(
            Phase8CommandName::ChallengeGenerate,
            "base_certificate_file_hash_mismatch",
            "base_certificate.file_hash",
            request.base_certificate.file_hash,
            actual_base_file_hash,
        ));
    }
    let actual_claimed_hash =
        phase8_raw_claimed_certificate_hash(base_certificate_bytes).map_err(|_| {
            phase8_command_value_error(
                Phase8CommandName::ChallengeGenerate,
                "base_certificate_claimed_hash_decode_failed",
                "base_certificate.claimed_certificate_hash",
                "raw_claimed_certificate_hash",
                "decode_failed",
            )
        })?;
    if actual_claimed_hash != request.base_certificate.claimed_certificate_hash {
        return Err(phase8_command_hash_error(
            Phase8CommandName::ChallengeGenerate,
            "base_certificate_claimed_hash_mismatch",
            "base_certificate.claimed_certificate_hash",
            request.base_certificate.claimed_certificate_hash,
            actual_claimed_hash,
        ));
    }

    let mutated_certificate_bytes =
        phase8_mutate_challenge_certificate_bytes(request, base_certificate_bytes)?;
    let mutated_file_hash = phase8_file_hash(&mutated_certificate_bytes);
    let mutated_claimed_certificate_hash =
        phase8_raw_claimed_certificate_hash(&mutated_certificate_bytes).ok();
    let known_kind = request
        .mutation
        .known_kind()
        .expect("generation request domain checked known mutation kind");
    let manifest = Phase8ChallengeManifest {
        challenge_id: request.challenge_id.clone(),
        policy_hash: request.policy_hash,
        module: request.module.clone(),
        imports: request.imports.clone(),
        base_certificate: request.base_certificate.clone(),
        mutated_certificate: Phase8ChallengeMutatedCertificate {
            path: request.output.mutated_certificate_path.clone(),
            file_hash: mutated_file_hash,
            claimed_certificate_hash: mutated_claimed_certificate_hash,
        },
        mutation: request.mutation.clone(),
        outcome_hint: Phase8ChallengeOutcomeHint {
            status: "should_fail".to_owned(),
            error_kinds: known_kind.outcome_error_kinds(),
        },
        replay: Phase8ChallengeReplayMetadata {
            generator: "npa-check challenge generate".to_owned(),
            generator_version: env!("CARGO_PKG_VERSION").to_owned(),
            generator_build_hash: phase8_sha256(PHASE8_CHALLENGE_MANIFEST_SCHEMA.as_bytes()),
            args_hash: request_hash,
        },
        generated_by: request.generated_by.clone(),
    };
    let generated_entry = Phase8ChallengeOutputStoreEntry {
        challenge_id: manifest.challenge_id.clone(),
        manifest_path: request.output.manifest_path.clone(),
        manifest_hash: manifest.manifest_hash(),
    };
    let store_update = phase8_challenge_output_store_with_entry(existing_store, generated_entry)?;
    let result = Phase8ChallengeGenerationResult {
        status: "written".to_owned(),
        challenge_id: request.challenge_id.clone(),
        request_hash,
        policy_hash: request.policy_hash,
        challenge_manifest: Phase8ChallengeManifestReference {
            path: request.output.manifest_path.clone(),
            manifest_hash: manifest.manifest_hash(),
        },
        mutated_certificate: Phase8ChallengeGeneratedCertificate {
            path: request.output.mutated_certificate_path.clone(),
            file_hash: mutated_file_hash,
            claimed_certificate_hash: mutated_claimed_certificate_hash,
        },
        challenge_store: Phase8ChallengeOutputStoreReference {
            path: request.output.store_manifest_path.clone(),
            manifest_hash: store_update.manifest.file_hash(),
        },
    };
    Ok(Phase8ChallengeGeneration {
        result,
        manifest,
        mutated_certificate_bytes,
        challenge_store: store_update.manifest,
        challenge_store_rewrite_required: store_update.rewrite_required,
    })
}

pub fn phase8_challenge_manifest_expected_certificate_hash(
    manifest: &Phase8ChallengeManifest,
) -> Hash {
    manifest
        .mutated_certificate
        .claimed_certificate_hash
        .unwrap_or(manifest.base_certificate.claimed_certificate_hash)
}

pub fn phase8_challenge_manifest_is_rejection_required(manifest: &Phase8ChallengeManifest) -> bool {
    manifest
        .mutation
        .known_kind()
        .is_some_and(Phase8ChallengeMutationKind::is_rejection_required)
}

fn phase8_policy_profiles_in_replay_order(policy: &Phase8RunnerPolicy) -> Vec<String> {
    let mut profiles = policy.required_checker_profiles.clone();
    profiles.extend(policy.optional_checker_profiles.iter().cloned());
    profiles
}

fn phase8_join_workspace_path(parent: &str, leaf: &str) -> String {
    let parent = parent.trim_end_matches('/');
    if parent.is_empty() {
        leaf.to_owned()
    } else {
        format!("{parent}/{leaf}")
    }
}

pub fn phase8_challenge_output_store_with_entry(
    existing_store: Option<&Phase8ChallengeOutputStoreManifest>,
    generated_entry: Phase8ChallengeOutputStoreEntry,
) -> Result<Phase8ChallengeOutputStoreUpdate, Phase8CommandError> {
    let mut manifest = existing_store
        .cloned()
        .unwrap_or(Phase8ChallengeOutputStoreManifest {
            entries: Vec::new(),
        });
    for existing in &manifest.entries {
        let same_challenge_id = existing.challenge_id == generated_entry.challenge_id;
        let same_manifest_path = existing.manifest_path == generated_entry.manifest_path;
        let exact = same_challenge_id
            && same_manifest_path
            && existing.manifest_hash == generated_entry.manifest_hash;
        if exact {
            return Ok(Phase8ChallengeOutputStoreUpdate {
                manifest,
                rewrite_required: false,
            });
        }
        if same_challenge_id {
            return Err(phase8_command_value_error(
                Phase8CommandName::ChallengeGenerate,
                "challenge_id_conflict",
                "challenge_id",
                "unique_challenge_id",
                &generated_entry.challenge_id,
            ));
        }
        if same_manifest_path {
            return Err(phase8_command_value_error(
                Phase8CommandName::ChallengeGenerate,
                "challenge_output_store_entry_conflict",
                "challenge_output_store.entries[]",
                generated_entry.canonical_json(),
                existing.canonical_json(),
            ));
        }
    }
    manifest.entries.push(generated_entry);
    manifest
        .entries
        .sort_by(|left, right| left.challenge_id.cmp(&right.challenge_id));
    Ok(Phase8ChallengeOutputStoreUpdate {
        manifest,
        rewrite_required: true,
    })
}

pub fn phase8_challenge_materialize_requests(
    manifest: &Phase8ChallengeManifest,
    manifest_hash: Hash,
    policy: &Phase8RunnerPolicy,
    request_output_dir: impl Into<String>,
    request_store_output_path: impl Into<String>,
    existing_store: Option<&Phase8RequestStoreManifest>,
) -> Result<Phase8ChallengeRequestMaterialization, Phase8CommandError> {
    let request_output_dir = request_output_dir.into();
    let request_store_output_path = request_store_output_path.into();
    validate_challenge_materialize_output_paths(&request_output_dir, &request_store_output_path)?;
    let policy_hash = policy.policy_hash();
    if manifest.policy_hash != policy_hash {
        return Err(phase8_command_hash_error(
            Phase8CommandName::ChallengeMaterializeRequests,
            "policy_hash_mismatch",
            "challenge_manifest.policy_hash",
            policy_hash,
            manifest.policy_hash,
        ));
    }
    if manifest.imports.mode != policy.import_policy.mode {
        return Err(phase8_command_value_error(
            Phase8CommandName::ChallengeMaterializeRequests,
            "import_mode_mismatch",
            "challenge_manifest.imports.mode",
            &policy.import_policy.mode,
            &manifest.imports.mode,
        ));
    }
    let mut generated_requests = Vec::new();
    let mut generated_entries = Vec::new();
    let mut result_entries = Vec::new();
    let expected_certificate_hash = phase8_challenge_manifest_expected_certificate_hash(manifest);
    let profiles = phase8_policy_profiles_in_replay_order(policy);
    for profile in profiles {
        let Some(selected_budget) = policy.budgets.get(&profile) else {
            return Err(phase8_command_value_error(
                Phase8CommandName::ChallengeMaterializeRequests,
                "policy_reference_invalid",
                "policy.budgets",
                "budget_for_checker_profile",
                profile,
            ));
        };
        let request_path =
            phase8_join_workspace_path(&request_output_dir, &format!("{profile}.json"));
        if request_path == request_store_output_path {
            return Err(phase8_command_value_error(
                Phase8CommandName::ChallengeMaterializeRequests,
                "request_output_path_conflict",
                "request_store_output_path",
                "distinct_request_and_store_paths",
                "overlap",
            ));
        }
        let request = Phase8MachineCheckRequest {
            request_id: format!("chreq:{}:{profile}", manifest.challenge_id),
            module: manifest.module.clone(),
            policy: Phase8MachineCheckRequestPolicy {
                id: policy.id.clone(),
                version: policy.version,
                hash: policy_hash,
            },
            certificate: Phase8MachineCheckRequestCertificate {
                path: manifest.mutated_certificate.path.clone(),
                file_hash: manifest.mutated_certificate.file_hash,
                expected_certificate_hash,
            },
            imports: Phase8MachineCheckRequestImports {
                mode: manifest.imports.mode.clone(),
                manifest: manifest.imports.manifest.clone(),
                manifest_hash: manifest.imports.manifest_hash,
            },
            checker_profile: profile.clone(),
            trust_mode: policy.trust_mode,
            axiom_policy: policy.axiom_policy.path.clone(),
            budget: selected_budget.clone(),
        };
        let file_hash = phase8_file_hash(request.canonical_json().as_bytes());
        let request_hash = request.request_hash();
        generated_entries.push(Phase8RequestStoreEntry {
            request_hash,
            path: request_path.clone(),
            file_hash,
        });
        result_entries.push(Phase8ChallengeMaterializedRequestEntry {
            checker_profile: profile,
            request_hash,
            path: request_path,
            file_hash,
        });
        generated_requests.push(request);
    }
    let store_update = phase8_request_store_with_materialized_entries(
        Phase8CommandName::ChallengeMaterializeRequests,
        existing_store,
        &generated_entries,
    )?;
    let result = Phase8ChallengeRequestMaterializationResult {
        status: "written".to_owned(),
        challenge_id: manifest.challenge_id.clone(),
        manifest_hash,
        policy_hash,
        request_store: Phase8RequestStoreReference {
            path: request_store_output_path,
            manifest_hash: store_update.manifest.file_hash(),
        },
        requests: result_entries,
    };
    Ok(Phase8ChallengeRequestMaterialization {
        result,
        requests: generated_requests,
        request_store: store_update.manifest,
        request_store_rewrite_required: store_update.rewrite_required,
    })
}

pub fn phase8_challenge_replay_store_entry_for_result(
    result: &Phase8ChallengeReplayResult,
    path: impl Into<String>,
) -> Result<Phase8ChallengeReplayStoreEntry, Phase8CommandError> {
    let path = path.into();
    if !phase8_valid_workspace_relative_path(&path) {
        return Err(phase8_command_value_error(
            Phase8CommandName::ChallengeReplay,
            "input_reference_invalid",
            "replay_result.path",
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    let bytes = result.canonical_json();
    Ok(Phase8ChallengeReplayStoreEntry {
        challenge_id: result.challenge_id.clone(),
        manifest_hash: result.manifest_hash,
        result_hash: result.result_hash(),
        artifact_hash: result.artifact_hash,
        path,
        file_hash: phase8_file_hash(bytes.as_bytes()),
    })
}

pub fn phase8_challenge_replay_store_with_entry(
    existing_store: Option<&Phase8ChallengeReplayStoreManifest>,
    generated_entry: Phase8ChallengeReplayStoreEntry,
) -> Result<Phase8ChallengeReplayStoreUpdate, Phase8CommandError> {
    let mut manifest = existing_store
        .cloned()
        .unwrap_or(Phase8ChallengeReplayStoreManifest {
            results: Vec::new(),
        });
    for existing in &manifest.results {
        let same_result_hash = existing.result_hash == generated_entry.result_hash;
        let same_path = existing.path == generated_entry.path;
        let same_challenge_manifest = existing.challenge_id == generated_entry.challenge_id
            && existing.manifest_hash == generated_entry.manifest_hash;
        let exact = same_result_hash
            && same_path
            && same_challenge_manifest
            && existing.artifact_hash == generated_entry.artifact_hash
            && existing.file_hash == generated_entry.file_hash;
        if exact {
            return Ok(Phase8ChallengeReplayStoreUpdate {
                manifest,
                rewrite_required: false,
            });
        }
        if same_result_hash || same_path || same_challenge_manifest {
            return Err(phase8_command_value_error(
                Phase8CommandName::ChallengeReplay,
                "replay_store_entry_conflict",
                "replay_store.results[]",
                generated_entry.canonical_json(),
                existing.canonical_json(),
            ));
        }
    }
    manifest.results.push(generated_entry);
    manifest
        .results
        .sort_by(|left, right| left.result_hash.cmp(&right.result_hash));
    Ok(Phase8ChallengeReplayStoreUpdate {
        manifest,
        rewrite_required: true,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn phase8_challenge_replay_aggregate(
    result_id: impl Into<String>,
    manifest: &Phase8ChallengeManifest,
    manifest_hash: Hash,
    policy: &Phase8RunnerPolicy,
    request_store: &Phase8RequestStoreManifest,
    stored_requests: &[Phase8StoredMachineCheckRequest],
    result_store: &Phase8MachineResultStoreManifest,
    machine_results: &[Phase8MachineCheckResult],
    normalized_store: Option<&Phase8NormalizedResultStoreManifest>,
    normalized_results: &[Phase8NormalizedCheckResult],
    coverage_required: bool,
    replay_output_path: impl Into<String>,
    existing_replay_store: Option<&Phase8ChallengeReplayStoreManifest>,
) -> Result<Phase8ChallengeReplayAggregation, Phase8CommandError> {
    let result_id = result_id.into();
    if !phase8_valid_request_id(&result_id) {
        return Err(phase8_command_value_error(
            Phase8CommandName::ChallengeReplay,
            "input_reference_invalid",
            "result_id",
            "result_id",
            if result_id.is_empty() {
                "empty_string"
            } else {
                "invalid_string_format"
            },
        ));
    }
    let replay_output_path = replay_output_path.into();
    if !phase8_valid_workspace_relative_path(&replay_output_path) {
        return Err(phase8_command_value_error(
            Phase8CommandName::ChallengeReplay,
            "input_reference_invalid",
            "replay_output_path",
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    if manifest.manifest_hash() != manifest_hash {
        return Err(phase8_command_hash_error(
            Phase8CommandName::ChallengeReplay,
            "manifest_hash_mismatch",
            "challenge_manifest.manifest_hash",
            manifest.manifest_hash(),
            manifest_hash,
        ));
    }
    let policy_hash = policy.policy_hash();
    if manifest.policy_hash != policy_hash {
        return Err(phase8_command_hash_error(
            Phase8CommandName::ChallengeReplay,
            "policy_hash_mismatch",
            "challenge_manifest.policy_hash",
            policy_hash,
            manifest.policy_hash,
        ));
    }
    validate_request_store_domain(&request_store.requests, "request_store").map_err(|error| {
        phase8_command_error_from_request_validation(
            Phase8CommandName::ChallengeReplay,
            "request_store_manifest_invalid",
            error.field.to_string(),
            error,
        )
    })?;
    validate_machine_result_store_domain(&result_store.results, "result_store").map_err(
        |error| {
            phase8_command_error_from_request_validation(
                Phase8CommandName::ChallengeReplay,
                "result_store_manifest_invalid",
                error.field.to_string(),
                error,
            )
        },
    )?;
    if let Some(store) = normalized_store {
        validate_normalized_result_store_domain(&store.results, "normalized_store").map_err(
            |error| {
                phase8_command_error_from_request_validation(
                    Phase8CommandName::ChallengeReplay,
                    "normalized_store_manifest_invalid",
                    error.field.to_string(),
                    error,
                )
            },
        )?;
    } else if coverage_required {
        return Err(phase8_command_value_error(
            Phase8CommandName::ChallengeReplay,
            "normalized_store_required",
            "normalized_store",
            "explicit_normalized_store",
            "missing",
        ));
    }

    let expected_requests =
        phase8_expected_challenge_replay_requests(manifest, manifest_hash, policy)?;
    let baseline_request = expected_requests.first().ok_or_else(|| {
        phase8_command_value_error(
            Phase8CommandName::ChallengeReplay,
            "policy_reference_invalid",
            "policy.required_checker_profiles",
            "one_or_more_checker_profiles",
            "empty",
        )
    })?;
    let artifact_hash =
        Phase8NormalizedArtifact::from_request(baseline_request, policy).artifact_hash();

    let mut checker_results = Vec::new();
    let mut included_machine_results = Vec::new();
    let mut missing_checker_profiles = Vec::new();
    for expected_request in &expected_requests {
        let profile = &expected_request.checker_profile;
        let required = policy.required_checker_profiles.contains(profile);
        let request_hash = expected_request.request_hash();
        let request_present = phase8_validate_challenge_replay_request_binding(
            request_store,
            stored_requests,
            expected_request,
        )?;
        if !request_present {
            if required {
                missing_checker_profiles.push(profile.clone());
            }
            continue;
        }
        let candidates = result_store
            .results
            .iter()
            .filter(|entry| entry.request_hash == request_hash && entry.checker_profile == *profile)
            .collect::<Vec<_>>();
        if candidates.len() > 1 {
            return Err(phase8_command_value_error(
                Phase8CommandName::ChallengeReplay,
                "result_attempt_ambiguous",
                "result_store.results[]",
                "at_most_one_result_per_request_profile",
                profile,
            ));
        }
        let Some(entry) = candidates.first().copied() else {
            if required {
                missing_checker_profiles.push(profile.clone());
            }
            continue;
        };
        let machine_result =
            phase8_resolve_replay_machine_result(entry, machine_results, policy_hash)?;
        checker_results.push(Phase8ChallengeReplayCheckerResult {
            result_id: machine_result.result_id.clone(),
            result_hash: machine_result.result_hash(),
            run_artifact_hash: machine_result.run_artifact_hash(),
            checker_profile: machine_result.checker.profile.clone(),
        });
        included_machine_results.push(machine_result);
    }

    let mut observed_error_kinds = included_machine_results
        .iter()
        .filter_map(|result| result.error.as_ref().map(|error| error.kind.clone()))
        .collect::<Vec<_>>();
    observed_error_kinds.sort();
    observed_error_kinds.dedup();

    let normalized = phase8_resolve_challenge_replay_normalized_result(
        policy_hash,
        artifact_hash,
        &checker_results,
        normalized_store,
        normalized_results,
        coverage_required,
    )?;
    let (normalized_result_hash, comparison_status, artifact_hash) = match normalized {
        Some(result) => (
            Some(result.normalized_result_hash()),
            Some(result.comparison.status),
            result.artifact_hash(),
        ),
        None => (None, None, artifact_hash),
    };

    let result = Phase8ChallengeReplayResult {
        result_id,
        challenge_id: manifest.challenge_id.clone(),
        manifest_hash,
        mutated_file_hash: manifest.mutated_certificate.file_hash,
        mutated_claimed_certificate_hash: manifest.mutated_certificate.claimed_certificate_hash,
        checker_results,
        missing_checker_profiles,
        normalized_result_hash,
        policy_hash,
        artifact_hash,
        comparison_status,
        observed_error_kinds,
    };
    let entry = phase8_challenge_replay_store_entry_for_result(&result, replay_output_path)?;
    let store_update = phase8_challenge_replay_store_with_entry(existing_replay_store, entry)?;
    Ok(Phase8ChallengeReplayAggregation {
        result,
        replay_store: store_update.manifest,
        replay_store_rewrite_required: store_update.rewrite_required,
    })
}

#[allow(clippy::too_many_arguments)]
pub fn phase8_challenge_coverage_summary(
    policy: &Phase8RunnerPolicy,
    target_normalized_result: &Phase8NormalizedCheckResult,
    challenge_store: &Phase8ChallengeOutputStoreManifest,
    challenge_manifests: &[Phase8ChallengeManifest],
    replay_store: &Phase8ChallengeReplayStoreManifest,
    replay_results: &[Phase8ChallengeReplayResult],
    result_store: &Phase8MachineResultStoreManifest,
    machine_results: &[Phase8MachineCheckResult],
) -> Result<Phase8ChallengeCoverageSummary, Phase8CommandError> {
    validate_challenge_output_store_domain(&challenge_store.entries, "challenge_store").map_err(
        |error| {
            phase8_command_error_from_request_validation(
                Phase8CommandName::ChallengeCoverageSummary,
                "challenge_store_manifest_invalid",
                error.field.to_string(),
                error,
            )
        },
    )?;
    validate_challenge_replay_store_domain(&replay_store.results, "replay_store").map_err(
        |error| {
            phase8_command_error_from_request_validation(
                Phase8CommandName::ChallengeCoverageSummary,
                "replay_store_manifest_invalid",
                error.field.to_string(),
                error,
            )
        },
    )?;
    validate_machine_result_store_domain(&result_store.results, "result_store").map_err(
        |error| {
            phase8_command_error_from_request_validation(
                Phase8CommandName::ChallengeCoverageSummary,
                "result_store_manifest_invalid",
                error.field.to_string(),
                error,
            )
        },
    )?;
    phase8_reject_duplicate_challenge_manifest_hashes(challenge_store)?;

    let policy_hash = policy.policy_hash();
    let target_artifact_hash = target_normalized_result.artifact_hash();
    let target_normalized_result_hash = target_normalized_result.normalized_result_hash();
    let mut entries = Vec::new();
    let mut unexpected_acceptances = 0_u64;

    for challenge_entry in &challenge_store.entries {
        let manifest = phase8_find_challenge_manifest(
            challenge_manifests,
            &challenge_entry.challenge_id,
            challenge_entry.manifest_hash,
        )?;
        phase8_validate_coverage_manifest_binding(
            manifest,
            challenge_entry,
            policy_hash,
            target_normalized_result,
        )?;
        let Some(replay_entry) = replay_store.results.iter().find(|entry| {
            entry.challenge_id == challenge_entry.challenge_id
                && entry.manifest_hash == challenge_entry.manifest_hash
        }) else {
            continue;
        };
        let replay_result = phase8_find_replay_result(replay_results, replay_entry.result_hash)?;
        phase8_validate_coverage_replay_binding(
            replay_result,
            replay_entry,
            manifest,
            policy_hash,
        )?;
        if replay_result.normalized_result_hash.is_none() {
            if replay_result.comparison_status.is_some() {
                return Err(phase8_command_value_error(
                    Phase8CommandName::ChallengeCoverageSummary,
                    "coverage_summary_generation_failed",
                    "replay_store.results[].comparison_status",
                    "absent_without_normalized_result_hash",
                    "present",
                ));
            }
            return Err(phase8_command_value_error(
                Phase8CommandName::ChallengeCoverageSummary,
                "coverage_summary_generation_failed",
                "replay_store.results[].normalized_result_hash",
                "required_for_coverage_summary",
                "missing",
            ));
        }
        let Some(comparison_status) = replay_result.comparison_status else {
            return Err(phase8_command_value_error(
                Phase8CommandName::ChallengeCoverageSummary,
                "coverage_summary_generation_failed",
                "replay_store.results[].comparison_status",
                "NormalizedCheckResult.comparison.status",
                "missing",
            ));
        };
        let required_checker_acceptance = phase8_replay_has_required_checker_acceptance(
            policy,
            replay_result,
            result_store,
            machine_results,
        )?;
        if comparison_status == Phase8NormalizedComparisonStatus::AllAgreeChecked
            || required_checker_acceptance
        {
            unexpected_acceptances += 1;
        }
        entries.push(Phase8ChallengeCoverageEntry {
            challenge_id: challenge_entry.challenge_id.clone(),
            manifest_hash: challenge_entry.manifest_hash,
            replay_result_hash: replay_result.result_hash(),
            comparison_status,
        });
    }
    entries.sort_by(|left, right| {
        left.challenge_id
            .cmp(&right.challenge_id)
            .then_with(|| left.manifest_hash.cmp(&right.manifest_hash))
    });

    Ok(Phase8ChallengeCoverageSummary::new(
        policy_hash,
        target_artifact_hash,
        target_normalized_result_hash,
        challenge_store.file_hash(),
        result_store.file_hash(),
        challenge_store.entries.len() as u64,
        unexpected_acceptances,
        entries,
    ))
}

fn phase8_challenge_coverage_summary_id(summary_hash: Hash) -> String {
    let wire = format_hash_string(&summary_hash);
    format!(
        "chcov_{}",
        wire.strip_prefix("sha256:")
            .expect("format_hash_string always prefixes sha256")
    )
}

#[allow(clippy::too_many_arguments)]
fn canonical_json_coverage_summary_pairs(
    policy_hash: Hash,
    artifact_hash: Hash,
    target_normalized_result_hash: Hash,
    challenge_store_manifest_hash: Hash,
    result_store_manifest_hash: Hash,
    total_challenges: u64,
    replayed_challenges: u64,
    unexpected_acceptances: u64,
    entries: &[Phase8ChallengeCoverageEntry],
) -> Vec<(String, String)> {
    vec![
        (
            "artifact_hash".to_owned(),
            phase8_hash_json_literal(&artifact_hash),
        ),
        (
            "challenge_store_manifest_hash".to_owned(),
            phase8_hash_json_literal(&challenge_store_manifest_hash),
        ),
        (
            "entries".to_owned(),
            canonical_json_array(
                entries
                    .iter()
                    .map(Phase8ChallengeCoverageEntry::canonical_json)
                    .collect(),
            ),
        ),
        (
            "policy_hash".to_owned(),
            phase8_hash_json_literal(&policy_hash),
        ),
        (
            "replayed_challenges".to_owned(),
            replayed_challenges.to_string(),
        ),
        (
            "result_store_manifest_hash".to_owned(),
            phase8_hash_json_literal(&result_store_manifest_hash),
        ),
        (
            "schema".to_owned(),
            phase8_json_string_literal(PHASE8_CHALLENGE_COVERAGE_SUMMARY_SCHEMA),
        ),
        (
            "target_normalized_result_hash".to_owned(),
            phase8_hash_json_literal(&target_normalized_result_hash),
        ),
        ("total_challenges".to_owned(), total_challenges.to_string()),
        (
            "unexpected_acceptances".to_owned(),
            unexpected_acceptances.to_string(),
        ),
    ]
}

fn phase8_expected_challenge_replay_requests(
    manifest: &Phase8ChallengeManifest,
    manifest_hash: Hash,
    policy: &Phase8RunnerPolicy,
) -> Result<Vec<Phase8MachineCheckRequest>, Phase8CommandError> {
    Ok(phase8_challenge_materialize_requests(
        manifest,
        manifest_hash,
        policy,
        "build/challenge-replay-requests",
        "build/challenge-replay-requests/manifest.json",
        None,
    )?
    .requests)
}

fn phase8_validate_challenge_replay_request_binding(
    request_store: &Phase8RequestStoreManifest,
    stored_requests: &[Phase8StoredMachineCheckRequest],
    expected_request: &Phase8MachineCheckRequest,
) -> Result<bool, Phase8CommandError> {
    let request_hash = expected_request.request_hash();
    let Some(entry) = request_store
        .requests
        .iter()
        .find(|entry| entry.request_hash == request_hash)
    else {
        return Ok(false);
    };
    let Some(stored) = stored_requests
        .iter()
        .find(|stored| stored.request.request_hash() == request_hash)
    else {
        return Err(phase8_command_value_error(
            Phase8CommandName::ChallengeReplay,
            "request_store_entry_unreadable",
            "request_store.requests[].path",
            "readable",
            "unreadable",
        ));
    };
    if stored.path != entry.path {
        return Err(phase8_command_value_error(
            Phase8CommandName::ChallengeReplay,
            "request_store_entry_mismatch",
            "request_store.requests[].path",
            &entry.path,
            &stored.path,
        ));
    }
    if stored.file_hash != entry.file_hash {
        return Err(phase8_command_hash_error(
            Phase8CommandName::ChallengeReplay,
            "request_file_hash_mismatch",
            "request_store.requests[].file_hash",
            entry.file_hash,
            stored.file_hash,
        ));
    }
    let canonical_file_hash = phase8_file_hash(stored.request.canonical_json().as_bytes());
    if canonical_file_hash != stored.file_hash {
        return Err(phase8_command_hash_error(
            Phase8CommandName::ChallengeReplay,
            "request_file_hash_mismatch",
            "request_store.requests[].file_hash",
            canonical_file_hash,
            stored.file_hash,
        ));
    }
    if stored.request.request_hash() != expected_request.request_hash() {
        return Err(phase8_command_hash_error(
            Phase8CommandName::ChallengeReplay,
            "request_hash_mismatch",
            "request_hash",
            expected_request.request_hash(),
            stored.request.request_hash(),
        ));
    }
    Ok(true)
}

fn phase8_resolve_replay_machine_result<'a>(
    entry: &Phase8MachineResultStoreEntry,
    machine_results: &'a [Phase8MachineCheckResult],
    policy_hash: Hash,
) -> Result<&'a Phase8MachineCheckResult, Phase8CommandError> {
    let Some(result) = machine_results
        .iter()
        .find(|result| result.run_artifact_hash() == entry.run_artifact_hash)
    else {
        return Err(phase8_command_value_error(
            Phase8CommandName::ChallengeReplay,
            "machine_result_not_found",
            "result_store.results[].run_artifact_hash",
            "loaded_machine_result",
            "missing",
        ));
    };
    if let Err((field, expected, actual)) = phase8_validate_machine_result_for_normalization(result)
    {
        return Err(phase8_command_value_error(
            Phase8CommandName::ChallengeReplay,
            "machine_result_schema_invalid",
            field,
            expected,
            actual,
        ));
    }
    if result.result_hash() != entry.result_hash {
        return Err(phase8_command_hash_error(
            Phase8CommandName::ChallengeReplay,
            "machine_result_hash_mismatch",
            "result_store.results[].result_hash",
            entry.result_hash,
            result.result_hash(),
        ));
    }
    if result.request_hash != entry.request_hash {
        return Err(phase8_command_hash_error(
            Phase8CommandName::ChallengeReplay,
            "machine_result_request_hash_mismatch",
            "result_store.results[].request_hash",
            entry.request_hash,
            result.request_hash,
        ));
    }
    if result.checker.profile != entry.checker_profile {
        return Err(phase8_command_value_error(
            Phase8CommandName::ChallengeReplay,
            "machine_result_checker_profile_mismatch",
            "result_store.results[].checker_profile",
            &entry.checker_profile,
            &result.checker.profile,
        ));
    }
    if result.policy.hash != policy_hash {
        return Err(phase8_command_hash_error(
            Phase8CommandName::ChallengeReplay,
            "machine_result_policy_hash_mismatch",
            "machine_results[].policy.hash",
            policy_hash,
            result.policy.hash,
        ));
    }
    let canonical_file_hash = phase8_file_hash(result.canonical_json().as_bytes());
    if canonical_file_hash != entry.file_hash {
        return Err(phase8_command_hash_error(
            Phase8CommandName::ChallengeReplay,
            "machine_result_file_hash_mismatch",
            "result_store.results[].file_hash",
            entry.file_hash,
            canonical_file_hash,
        ));
    }
    Ok(result)
}

fn phase8_resolve_challenge_replay_normalized_result<'a>(
    policy_hash: Hash,
    artifact_hash: Hash,
    checker_results: &[Phase8ChallengeReplayCheckerResult],
    normalized_store: Option<&Phase8NormalizedResultStoreManifest>,
    normalized_results: &'a [Phase8NormalizedCheckResult],
    coverage_required: bool,
) -> Result<Option<&'a Phase8NormalizedCheckResult>, Phase8CommandError> {
    let Some(store) = normalized_store else {
        return Ok(None);
    };
    let mut expected_hashes = checker_results
        .iter()
        .map(|result| result.result_hash)
        .collect::<Vec<_>>();
    expected_hashes.sort();
    expected_hashes.dedup();
    let mut matches = Vec::new();
    for normalized in normalized_results {
        let normalized_hash = normalized.normalized_result_hash();
        let Some(entry) = store
            .results
            .iter()
            .find(|entry| entry.normalized_result_hash == normalized_hash)
        else {
            continue;
        };
        if normalized.artifact_hash() != entry.artifact_hash {
            return Err(phase8_command_hash_error(
                Phase8CommandName::ChallengeReplay,
                "normalized_store_entry_artifact_hash_mismatch",
                "normalized_store.results[].artifact_hash",
                entry.artifact_hash,
                normalized.artifact_hash(),
            ));
        }
        let canonical_file_hash = phase8_file_hash(normalized.canonical_json().as_bytes());
        if canonical_file_hash != entry.file_hash {
            return Err(phase8_command_hash_error(
                Phase8CommandName::ChallengeReplay,
                "normalized_store_entry_file_hash_mismatch",
                "normalized_store.results[].file_hash",
                entry.file_hash,
                canonical_file_hash,
            ));
        }
        if normalized.policy.hash != policy_hash || normalized.artifact_hash() != artifact_hash {
            continue;
        }
        let mut actual_hashes = normalized
            .results
            .iter()
            .map(|entry| entry.result_hash)
            .collect::<Vec<_>>();
        actual_hashes.sort();
        actual_hashes.dedup();
        if actual_hashes == expected_hashes {
            matches.push(normalized);
        }
    }
    match matches.len() {
        0 if coverage_required => Err(phase8_command_value_error(
            Phase8CommandName::ChallengeReplay,
            "normalized_result_not_found",
            "normalized_store.results[]",
            "matching_normalized_result",
            "missing",
        )),
        0 => Ok(None),
        1 => Ok(Some(matches[0])),
        _ => Err(phase8_command_value_error(
            Phase8CommandName::ChallengeReplay,
            "normalized_result_ambiguous",
            "normalized_store.results[]",
            "unique_matching_normalized_result",
            "ambiguous",
        )),
    }
}

fn phase8_reject_duplicate_challenge_manifest_hashes(
    challenge_store: &Phase8ChallengeOutputStoreManifest,
) -> Result<(), Phase8CommandError> {
    let mut seen = BTreeSet::new();
    for (index, entry) in challenge_store.entries.iter().enumerate() {
        if !seen.insert(entry.manifest_hash) {
            return Err(phase8_command_value_error(
                Phase8CommandName::ChallengeCoverageSummary,
                "coverage_summary_generation_failed",
                format!("challenge_store.entries[{index}].manifest_hash"),
                "unique_manifest_hashes",
                "duplicate_manifest_hash",
            ));
        }
    }
    Ok(())
}

fn phase8_find_challenge_manifest<'a>(
    manifests: &'a [Phase8ChallengeManifest],
    challenge_id: &str,
    manifest_hash: Hash,
) -> Result<&'a Phase8ChallengeManifest, Phase8CommandError> {
    let matches = manifests
        .iter()
        .filter(|manifest| {
            manifest.challenge_id == challenge_id && manifest.manifest_hash() == manifest_hash
        })
        .collect::<Vec<_>>();
    match matches.len() {
        1 => Ok(matches[0]),
        0 => Err(phase8_command_value_error(
            Phase8CommandName::ChallengeCoverageSummary,
            "coverage_summary_generation_failed",
            "challenge_store.entries[].manifest_hash",
            "loaded_challenge_manifest",
            "missing",
        )),
        _ => Err(phase8_command_value_error(
            Phase8CommandName::ChallengeCoverageSummary,
            "coverage_summary_generation_failed",
            "challenge_store.entries[].manifest_hash",
            "unique_loaded_challenge_manifest",
            "ambiguous",
        )),
    }
}

fn phase8_validate_coverage_manifest_binding(
    manifest: &Phase8ChallengeManifest,
    challenge_entry: &Phase8ChallengeOutputStoreEntry,
    policy_hash: Hash,
    target_normalized_result: &Phase8NormalizedCheckResult,
) -> Result<(), Phase8CommandError> {
    if manifest.challenge_id != challenge_entry.challenge_id {
        return Err(phase8_command_value_error(
            Phase8CommandName::ChallengeCoverageSummary,
            "coverage_summary_generation_failed",
            "challenge_store.entries[].challenge_id",
            &manifest.challenge_id,
            &challenge_entry.challenge_id,
        ));
    }
    if manifest.policy_hash != policy_hash {
        return Err(phase8_command_hash_error(
            Phase8CommandName::ChallengeCoverageSummary,
            "coverage_summary_generation_failed",
            "challenge_manifest.policy_hash",
            policy_hash,
            manifest.policy_hash,
        ));
    }
    if !phase8_challenge_manifest_is_rejection_required(manifest) {
        return Err(phase8_command_value_error(
            Phase8CommandName::ChallengeCoverageSummary,
            "coverage_summary_generation_failed",
            "challenge_manifest.mutation.kind",
            "rejection_required_challenge",
            &manifest.mutation.kind,
        ));
    }
    if manifest.base_certificate.file_hash != target_normalized_result.artifact.input_file_hash {
        return Err(phase8_command_hash_error(
            Phase8CommandName::ChallengeCoverageSummary,
            "coverage_summary_generation_failed",
            "challenge_manifest.base_certificate.file_hash",
            target_normalized_result.artifact.input_file_hash,
            manifest.base_certificate.file_hash,
        ));
    }
    if manifest.base_certificate.claimed_certificate_hash
        != target_normalized_result.artifact.expected_certificate_hash
    {
        return Err(phase8_command_hash_error(
            Phase8CommandName::ChallengeCoverageSummary,
            "coverage_summary_generation_failed",
            "challenge_manifest.base_certificate.claimed_certificate_hash",
            target_normalized_result.artifact.expected_certificate_hash,
            manifest.base_certificate.claimed_certificate_hash,
        ));
    }
    Ok(())
}

fn phase8_find_replay_result(
    replay_results: &[Phase8ChallengeReplayResult],
    result_hash: Hash,
) -> Result<&Phase8ChallengeReplayResult, Phase8CommandError> {
    let matches = replay_results
        .iter()
        .filter(|result| result.result_hash() == result_hash)
        .collect::<Vec<_>>();
    match matches.len() {
        1 => Ok(matches[0]),
        0 => Err(phase8_command_value_error(
            Phase8CommandName::ChallengeCoverageSummary,
            "coverage_summary_generation_failed",
            "replay_store.results[].result_hash",
            "loaded_challenge_replay_result",
            "missing",
        )),
        _ => Err(phase8_command_value_error(
            Phase8CommandName::ChallengeCoverageSummary,
            "coverage_summary_generation_failed",
            "replay_store.results[].result_hash",
            "unique_loaded_challenge_replay_result",
            "ambiguous",
        )),
    }
}

fn phase8_validate_coverage_replay_binding(
    replay_result: &Phase8ChallengeReplayResult,
    replay_entry: &Phase8ChallengeReplayStoreEntry,
    manifest: &Phase8ChallengeManifest,
    policy_hash: Hash,
) -> Result<(), Phase8CommandError> {
    if replay_result.challenge_id != replay_entry.challenge_id {
        return Err(phase8_command_value_error(
            Phase8CommandName::ChallengeCoverageSummary,
            "coverage_summary_generation_failed",
            "replay_store.results[].challenge_id",
            &replay_entry.challenge_id,
            &replay_result.challenge_id,
        ));
    }
    if replay_result.manifest_hash != replay_entry.manifest_hash {
        return Err(phase8_command_hash_error(
            Phase8CommandName::ChallengeCoverageSummary,
            "coverage_summary_generation_failed",
            "replay_store.results[].manifest_hash",
            replay_entry.manifest_hash,
            replay_result.manifest_hash,
        ));
    }
    if replay_result.artifact_hash != replay_entry.artifact_hash {
        return Err(phase8_command_hash_error(
            Phase8CommandName::ChallengeCoverageSummary,
            "coverage_summary_generation_failed",
            "replay_store.results[].artifact_hash",
            replay_entry.artifact_hash,
            replay_result.artifact_hash,
        ));
    }
    let canonical_file_hash = phase8_file_hash(replay_result.canonical_json().as_bytes());
    if canonical_file_hash != replay_entry.file_hash {
        return Err(phase8_command_hash_error(
            Phase8CommandName::ChallengeCoverageSummary,
            "coverage_summary_generation_failed",
            "replay_store.results[].file_hash",
            replay_entry.file_hash,
            canonical_file_hash,
        ));
    }
    if replay_result.policy_hash != policy_hash {
        return Err(phase8_command_hash_error(
            Phase8CommandName::ChallengeCoverageSummary,
            "coverage_summary_generation_failed",
            "replay_result.policy_hash",
            policy_hash,
            replay_result.policy_hash,
        ));
    }
    if replay_result.mutated_file_hash != manifest.mutated_certificate.file_hash {
        return Err(phase8_command_hash_error(
            Phase8CommandName::ChallengeCoverageSummary,
            "coverage_summary_generation_failed",
            "replay_result.mutated_file_hash",
            manifest.mutated_certificate.file_hash,
            replay_result.mutated_file_hash,
        ));
    }
    match (
        manifest.mutated_certificate.claimed_certificate_hash,
        replay_result.mutated_claimed_certificate_hash,
    ) {
        (Some(expected), Some(actual)) if expected != actual => Err(phase8_command_hash_error(
            Phase8CommandName::ChallengeCoverageSummary,
            "coverage_summary_generation_failed",
            "replay_result.mutated_claimed_certificate_hash",
            expected,
            actual,
        )),
        (Some(_), None) => Err(phase8_command_value_error(
            Phase8CommandName::ChallengeCoverageSummary,
            "coverage_summary_generation_failed",
            "replay_result.mutated_claimed_certificate_hash",
            "sha256:<lower-hex>",
            "missing",
        )),
        (None, Some(_)) => Err(phase8_command_value_error(
            Phase8CommandName::ChallengeCoverageSummary,
            "coverage_summary_generation_failed",
            "replay_result.mutated_claimed_certificate_hash",
            "absent_when_manifest_omits_mutated_claimed_certificate_hash",
            "present",
        )),
        _ => Ok(()),
    }
}

fn phase8_replay_has_required_checker_acceptance(
    policy: &Phase8RunnerPolicy,
    replay_result: &Phase8ChallengeReplayResult,
    result_store: &Phase8MachineResultStoreManifest,
    machine_results: &[Phase8MachineCheckResult],
) -> Result<bool, Phase8CommandError> {
    for checker_result in &replay_result.checker_results {
        if !policy
            .required_checker_profiles
            .contains(&checker_result.checker_profile)
        {
            continue;
        }
        let Some(store_entry) = result_store.results.iter().find(|entry| {
            entry.run_artifact_hash == checker_result.run_artifact_hash
                && entry.result_hash == checker_result.result_hash
                && entry.checker_profile == checker_result.checker_profile
        }) else {
            return Err(phase8_command_value_error(
                Phase8CommandName::ChallengeCoverageSummary,
                "coverage_summary_generation_failed",
                "result_store.results[].run_artifact_hash",
                "matching_machine_result_store_entry",
                "missing",
            ));
        };
        let Some(machine_result) = machine_results
            .iter()
            .find(|result| result.run_artifact_hash() == store_entry.run_artifact_hash)
        else {
            return Err(phase8_command_value_error(
                Phase8CommandName::ChallengeCoverageSummary,
                "coverage_summary_generation_failed",
                "machine_results[].run_artifact_hash",
                "loaded_machine_result",
                "missing",
            ));
        };
        if machine_result.result_hash() != checker_result.result_hash
            || machine_result.checker.profile != checker_result.checker_profile
        {
            return Err(phase8_command_value_error(
                Phase8CommandName::ChallengeCoverageSummary,
                "coverage_summary_generation_failed",
                "machine_results[]",
                "matching_replay_checker_result",
                "mismatch",
            ));
        }
        if machine_result.request_hash != store_entry.request_hash {
            return Err(phase8_command_hash_error(
                Phase8CommandName::ChallengeCoverageSummary,
                "coverage_summary_generation_failed",
                "result_store.results[].request_hash",
                store_entry.request_hash,
                machine_result.request_hash,
            ));
        }
        let canonical_file_hash = phase8_file_hash(machine_result.canonical_json().as_bytes());
        if canonical_file_hash != store_entry.file_hash {
            return Err(phase8_command_hash_error(
                Phase8CommandName::ChallengeCoverageSummary,
                "coverage_summary_generation_failed",
                "result_store.results[].file_hash",
                store_entry.file_hash,
                canonical_file_hash,
            ));
        }
        if machine_result.status == Phase8MachineCheckStatus::Checked {
            return Ok(true);
        }
    }
    Ok(false)
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

    let normalized_policy = Phase8MachineCheckRequestPolicy {
        id: policy.id.clone(),
        version: policy.version,
        hash: policy_hash,
    };
    let comparison = phase8_build_normalized_comparison_parts(
        policy,
        normalized_policy.hash,
        artifact.artifact_hash(),
        &entries,
    );
    Ok(Phase8NormalizedCheckResult {
        normalized_result_id,
        artifact,
        policy: normalized_policy,
        results: entries,
        comparison,
    })
}

pub fn phase8_build_normalized_comparison(
    policy: &Phase8RunnerPolicy,
    normalized_result: &Phase8NormalizedCheckResult,
) -> Phase8NormalizedComparison {
    phase8_build_normalized_comparison_parts(
        policy,
        normalized_result.policy.hash,
        normalized_result.artifact_hash(),
        &normalized_result.results,
    )
}

pub fn phase8_compare_normalized_result(
    policy: &Phase8RunnerPolicy,
    normalized_result: &Phase8NormalizedCheckResult,
) -> Phase8CompareValidationResult {
    let policy_hash = policy.policy_hash();
    let recomputed = phase8_build_normalized_comparison(policy, normalized_result);
    let embedded_json = normalized_result.comparison.canonical_json();
    let recomputed_json = recomputed.canonical_json();
    if embedded_json == recomputed_json {
        Phase8CompareValidationResult::valid(
            normalized_result.normalized_result_hash(),
            policy_hash,
            normalized_result.comparison.status,
        )
    } else {
        Phase8CompareValidationResult::comparison_mismatch(
            normalized_result.normalized_result_hash(),
            policy_hash,
            normalized_result.comparison.status,
            recomputed.status,
            phase8_sha256(recomputed_json.as_bytes()),
            phase8_sha256(embedded_json.as_bytes()),
        )
    }
}

pub fn parse_phase8_release_policy(
    source: &str,
) -> Result<Phase8ReleasePolicy, Phase8PolicyValidationError> {
    let document = JsonDocument::parse(source)
        .map_err(|_| Phase8PolicyValidationError::new("$", "valid_json", "invalid_json"))?;
    parse_phase8_release_policy_value(document.root())
}

pub fn phase8_release_policy_hash(source: &str) -> Result<Hash, Phase8PolicyValidationError> {
    Ok(parse_phase8_release_policy(source)?.policy_hash())
}

pub fn phase8_validate_release_policy_runner_trust(
    policy: &Phase8ReleasePolicy,
    runner_policy: &Phase8RunnerPolicy,
    challenge_runner_policy: &Phase8RunnerPolicy,
) -> Result<(), Phase8PolicyValidationError> {
    let expected = policy.mode.trust_mode();
    if runner_policy.trust_mode != expected {
        return Err(Phase8PolicyValidationError::new(
            "runner_policy_hash",
            format!("RunnerPolicy.trust_mode:{}", policy.mode.as_str()),
            format!(
                "RunnerPolicy.trust_mode:{}",
                runner_policy.trust_mode.as_str()
            ),
        ));
    }
    if challenge_runner_policy.trust_mode != expected {
        return Err(Phase8PolicyValidationError::new(
            "challenge_runner_policy_hash",
            format!("RunnerPolicy.trust_mode:{}", policy.mode.as_str()),
            format!(
                "RunnerPolicy.trust_mode:{}",
                challenge_runner_policy.trust_mode.as_str()
            ),
        ));
    }
    Ok(())
}

pub fn parse_phase8_release_bundle_staging_plan(
    source: &str,
) -> Result<Phase8ReleaseBundleStagingPlan, Phase8CommandError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        phase8_release_stage_value_error(
            "input_json_invalid",
            "plan.path",
            "valid_json",
            "invalid_json",
        )
    })?;
    parse_phase8_release_bundle_staging_plan_value(document.root())
}

pub fn phase8_release_stage_bundle_inputs(
    phase: Phase8ReleaseBundleStagingPhase,
    bundle_root: impl Into<String>,
    plan_path: impl Into<String>,
    plan_hash: Hash,
    plan_source: &str,
    workspace_files: &BTreeMap<String, Vec<u8>>,
    existing_bundle_files: Option<&BTreeMap<String, Vec<u8>>>,
) -> Result<Phase8ReleaseBundleStaging, Phase8CommandError> {
    let bundle_root = bundle_root.into();
    let plan_path = plan_path.into();
    if !phase8_valid_workspace_relative_path(&plan_path) {
        return Err(phase8_release_stage_value_error(
            "input_reference_invalid",
            "plan.path",
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    if !phase8_valid_workspace_relative_path(&bundle_root) {
        return Err(phase8_release_stage_value_error(
            "input_reference_invalid",
            "bundle_root",
            "workspace_relative_path",
            "invalid_path",
        ));
    }

    let actual_plan_hash = phase8_file_hash(plan_source.as_bytes());
    if actual_plan_hash != plan_hash {
        return Err(phase8_release_stage_hash_error(
            "input_hash_mismatch",
            "plan_hash",
            plan_hash,
            actual_plan_hash,
        ));
    }

    let plan = parse_phase8_release_bundle_staging_plan(plan_source)?;
    if plan.phase != phase {
        return Err(phase8_release_stage_value_error(
            "input_schema_invalid",
            "phase",
            format!("--phase:{}", phase.as_str()),
            format!("plan.phase:{}", plan.phase.as_str()),
        ));
    }
    if plan.bundle_root != bundle_root {
        return Err(phase8_release_stage_value_error(
            "input_schema_invalid",
            "bundle_root",
            format!("--bundle-root:{bundle_root}"),
            format!("plan.bundle_root:{}", plan.bundle_root),
        ));
    }

    let mut staged_files = BTreeMap::<String, Vec<u8>>::new();
    let mut staged_artifacts = Vec::<Phase8ReleaseBundleStagedArtifact>::new();
    let mut request_entries = BTreeMap::<Hash, Phase8RequestStoreEntry>::new();
    let mut machine_entries = BTreeMap::<Hash, Phase8MachineResultStoreEntry>::new();
    let mut normalized_entries = BTreeMap::<Hash, Phase8NormalizedResultStoreEntry>::new();

    for (index, input) in plan.inputs.iter().enumerate() {
        let source_bytes = workspace_files.get(&input.path).ok_or_else(|| {
            phase8_release_stage_value_error(
                "input_file_unreadable",
                format!("inputs[{index}].path"),
                "readable_file",
                "missing",
            )
        })?;
        let actual_file_hash = phase8_file_hash(source_bytes);
        if actual_file_hash != input.file_hash {
            return Err(phase8_release_stage_hash_error(
                "input_hash_mismatch",
                format!("inputs[{index}].file_hash"),
                input.file_hash,
                actual_file_hash,
            ));
        }
        let input_json_reason = if input.kind.is_store_source_manifest()
            || input.kind == Phase8ReleaseBundleArtifactKind::ChallengeOutputStoreManifest
        {
            "input_store_manifest_invalid"
        } else {
            "input_json_invalid"
        };
        let source = phase8_release_stage_utf8(
            source_bytes,
            input_json_reason,
            format!("inputs[{index}].path"),
        )?;

        if input.kind.is_store_source_manifest() {
            phase8_release_validate_planned_hash(input, index, "manifest_hash", input.file_hash)?;
            match input.kind {
                Phase8ReleaseBundleArtifactKind::RequestStoreManifest => {
                    let manifest =
                        parse_phase8_request_store_manifest(source).map_err(|error| {
                            phase8_release_stage_store_manifest_error(index, input.kind, error)
                        })?;
                    phase8_release_stage_request_manifest_entries(
                        index,
                        &manifest,
                        workspace_files,
                        existing_bundle_files,
                        &mut staged_files,
                        &mut staged_artifacts,
                        &mut request_entries,
                    )?;
                }
                Phase8ReleaseBundleArtifactKind::MachineResultStoreManifest => {
                    let manifest =
                        parse_phase8_machine_result_store_manifest(source).map_err(|error| {
                            phase8_release_stage_store_manifest_error(index, input.kind, error)
                        })?;
                    phase8_release_stage_machine_manifest_entries(
                        index,
                        &manifest,
                        workspace_files,
                        existing_bundle_files,
                        &mut staged_files,
                        &mut staged_artifacts,
                        &mut machine_entries,
                    )?;
                }
                Phase8ReleaseBundleArtifactKind::NormalizedResultStoreManifest => {
                    let manifest =
                        parse_phase8_normalized_result_store_manifest(source).map_err(|error| {
                            phase8_release_stage_store_manifest_error(index, input.kind, error)
                        })?;
                    phase8_release_stage_normalized_manifest_entries(
                        index,
                        &manifest,
                        workspace_files,
                        existing_bundle_files,
                        &mut staged_files,
                        &mut staged_artifacts,
                        &mut normalized_entries,
                    )?;
                }
                _ => unreachable!("store source manifest predicate is exhaustive"),
            }
            continue;
        }

        phase8_release_validate_direct_artifact(input, index, source)?;
        phase8_release_stage_file(
            input.kind,
            input.file_hash,
            source_bytes,
            existing_bundle_files,
            &mut staged_files,
            &mut staged_artifacts,
        )?;
    }

    let mut store_manifests = Vec::new();
    if phase == Phase8ReleaseBundleStagingPhase::Store {
        let request_manifest = Phase8RequestStoreManifest {
            requests: request_entries.into_values().collect(),
        };
        let machine_manifest = Phase8MachineResultStoreManifest {
            results: machine_entries.into_values().collect(),
        };
        let normalized_manifest = Phase8NormalizedResultStoreManifest {
            results: normalized_entries.into_values().collect(),
        };
        phase8_release_stage_generated_store_manifest(
            Phase8ReleaseBundleArtifactKind::RequestStoreManifest,
            request_manifest.canonical_json(),
            existing_bundle_files,
            &mut staged_files,
            &mut store_manifests,
        )?;
        phase8_release_stage_generated_store_manifest(
            Phase8ReleaseBundleArtifactKind::MachineResultStoreManifest,
            machine_manifest.canonical_json(),
            existing_bundle_files,
            &mut staged_files,
            &mut store_manifests,
        )?;
        phase8_release_stage_generated_store_manifest(
            Phase8ReleaseBundleArtifactKind::NormalizedResultStoreManifest,
            normalized_manifest.canonical_json(),
            existing_bundle_files,
            &mut staged_files,
            &mut store_manifests,
        )?;
    }

    staged_artifacts.sort_by(|left, right| {
        (left.kind.as_str(), left.path.as_str(), left.file_hash).cmp(&(
            right.kind.as_str(),
            right.path.as_str(),
            right.file_hash,
        ))
    });
    store_manifests.sort_by(|left, right| {
        (left.kind.as_str(), left.path.as_str(), left.manifest_hash).cmp(&(
            right.kind.as_str(),
            right.path.as_str(),
            right.manifest_hash,
        ))
    });

    let files = staged_files
        .into_iter()
        .map(|(path, bytes)| Phase8ReleaseBundleStagedFile { path, bytes })
        .collect();
    Ok(Phase8ReleaseBundleStaging {
        result: Phase8ReleaseBundleStagingResult {
            phase,
            bundle_root,
            staged_artifacts,
            store_manifests,
        },
        files,
    })
}

pub fn parse_phase8_axiom_policy_toml(
    source: &str,
) -> Result<Phase8AxiomPolicy, Phase8PolicyValidationError> {
    phase8_parse_axiom_policy_toml(source)
}

pub fn parse_phase8_auxiliary_result(
    source: &str,
) -> Result<Phase8AuxiliaryResult, Phase8RequestValidationError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        Phase8RequestValidationError::value_failure(
            "auxiliary_result",
            "valid_json",
            "invalid_json",
        )
    })?;
    parse_phase8_auxiliary_result_value(document.root())
}

pub fn parse_phase8_auxiliary_result_store_manifest(
    source: &str,
) -> Result<Phase8AuxiliaryResultStoreManifest, Phase8RequestValidationError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        Phase8RequestValidationError::value_failure(
            "auxiliary_result_store",
            "valid_json",
            "invalid_json",
        )
    })?;
    parse_phase8_auxiliary_result_store_manifest_value(document.root())
}

pub fn phase8_auxiliary_result_store_entry_for_result(
    result: &Phase8AuxiliaryResult,
    path: impl Into<String>,
) -> Result<Phase8AuxiliaryResultStoreEntry, Phase8CommandError> {
    let path = path.into();
    if !phase8_valid_workspace_relative_path(&path) {
        return Err(phase8_command_value_error(
            phase8_command_for_auxiliary_kind(result.kind),
            "input_reference_invalid",
            "auxiliary_result.path",
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    let bytes = result.canonical_json();
    Ok(Phase8AuxiliaryResultStoreEntry {
        result_hash: result.result_hash(),
        kind: result.kind,
        policy_hash: result.policy_hash,
        artifact_hash: result.artifact_hash,
        path,
        file_hash: phase8_file_hash(bytes.as_bytes()),
    })
}

pub fn phase8_auxiliary_result_store_with_entry(
    existing_store: Option<&Phase8AuxiliaryResultStoreManifest>,
    generated_entry: Phase8AuxiliaryResultStoreEntry,
) -> Result<Phase8AuxiliaryResultStoreUpdate, Phase8CommandError> {
    let mut manifest = existing_store
        .cloned()
        .unwrap_or(Phase8AuxiliaryResultStoreManifest {
            results: Vec::new(),
        });

    for existing in &manifest.results {
        let same_result_hash = existing.result_hash == generated_entry.result_hash;
        let same_path = existing.path == generated_entry.path;
        let exact = same_result_hash
            && same_path
            && existing.kind == generated_entry.kind
            && existing.policy_hash == generated_entry.policy_hash
            && existing.artifact_hash == generated_entry.artifact_hash
            && existing.file_hash == generated_entry.file_hash;
        if exact {
            return Ok(Phase8AuxiliaryResultStoreUpdate {
                manifest,
                rewrite_required: false,
            });
        }
        if same_result_hash || same_path {
            return Err(phase8_command_value_error(
                phase8_command_for_auxiliary_kind(generated_entry.kind),
                "auxiliary_store_entry_conflict",
                "auxiliary_result_store.results[]",
                generated_entry.canonical_json(),
                existing.canonical_json(),
            ));
        }
    }

    manifest.results.push(generated_entry);
    manifest.results.sort_by(|left, right| {
        left.result_hash
            .cmp(&right.result_hash)
            .then_with(|| left.kind.as_str().cmp(right.kind.as_str()))
            .then_with(|| left.policy_hash.cmp(&right.policy_hash))
            .then_with(|| left.artifact_hash.cmp(&right.artifact_hash))
    });
    Ok(Phase8AuxiliaryResultStoreUpdate {
        manifest,
        rewrite_required: true,
    })
}

pub fn phase8_auxiliary_axiom_policy_result(
    result_id: impl Into<String>,
    policy: &Phase8RunnerPolicy,
    axiom_policy: &Phase8AxiomPolicy,
    normalized_result: &Phase8NormalizedCheckResult,
    axiom_report: &Phase8AxiomReport,
) -> Result<Phase8AuxiliaryResult, Phase8CommandError> {
    let Some(checker_profile) = policy.required_checker_profiles.first() else {
        return Err(phase8_command_value_error(
            Phase8CommandName::AuxiliaryAxiomPolicy,
            "policy_reference_invalid",
            "policy.required_checker_profiles",
            "baseline_required_checker_profile",
            "missing",
        ));
    };
    let selected = normalized_result
        .results
        .iter()
        .find(|entry| &entry.checker_profile == checker_profile);
    let selector = selected.map(|entry| Phase8AuxiliarySelector::AxiomPolicy {
        normalized_result_hash: normalized_result.normalized_result_hash(),
        checker_profile: checker_profile.clone(),
        result_hash: entry.result_hash,
        axiom_report_hash: entry.axiom_report_hash.unwrap_or([0; 32]),
    });
    phase8_auxiliary_axiom_policy_result_with_selector(
        result_id,
        policy,
        axiom_policy,
        normalized_result,
        axiom_report,
        selector,
    )
}

pub fn phase8_auxiliary_axiom_policy_result_with_selector(
    result_id: impl Into<String>,
    policy: &Phase8RunnerPolicy,
    axiom_policy: &Phase8AxiomPolicy,
    normalized_result: &Phase8NormalizedCheckResult,
    axiom_report: &Phase8AxiomReport,
    selector: Option<Phase8AuxiliarySelector>,
) -> Result<Phase8AuxiliaryResult, Phase8CommandError> {
    let result_id = result_id.into();
    validate_auxiliary_result_id(&result_id, Phase8CommandName::AuxiliaryAxiomPolicy)?;
    let policy_hash = policy.policy_hash();
    let artifact_hash = normalized_result.artifact_hash();
    let selector = match selector {
        Some(Phase8AuxiliarySelector::AxiomPolicy { .. }) => selector,
        Some(_) | None => {
            return Ok(Phase8AuxiliaryResult::inconclusive(
                result_id,
                Phase8AuxiliaryResultKind::AxiomPolicy,
                policy_hash,
                artifact_hash,
                selector,
                Phase8AuxiliaryError::value(
                    Phase8AuxiliaryReasonCode::AxiomPolicyInconclusive,
                    "selector",
                    "axiom_policy_selector",
                    "missing",
                ),
            ))
        }
    };
    let Phase8AuxiliarySelector::AxiomPolicy {
        normalized_result_hash,
        checker_profile,
        result_hash,
        axiom_report_hash,
    } = selector
        .clone()
        .expect("selector is Some and checked as axiom_policy")
    else {
        unreachable!("selector kind checked above")
    };
    let actual_normalized_hash = normalized_result.normalized_result_hash();
    if normalized_result_hash != actual_normalized_hash {
        return Ok(Phase8AuxiliaryResult::inconclusive(
            result_id,
            Phase8AuxiliaryResultKind::AxiomPolicy,
            policy_hash,
            artifact_hash,
            selector,
            Phase8AuxiliaryError::hash(
                Phase8AuxiliaryReasonCode::AxiomPolicyInconclusive,
                "selector.normalized_result_hash",
                actual_normalized_hash,
                normalized_result_hash,
            ),
        ));
    }
    let selected = normalized_result
        .results
        .iter()
        .find(|entry| entry.checker_profile == checker_profile);
    let Some(selected) = selected else {
        return Ok(Phase8AuxiliaryResult::inconclusive(
            result_id,
            Phase8AuxiliaryResultKind::AxiomPolicy,
            policy_hash,
            artifact_hash,
            selector,
            Phase8AuxiliaryError::value(
                Phase8AuxiliaryReasonCode::AxiomPolicyInconclusive,
                "selector.checker_profile",
                "checked_normalized_result_entry",
                "missing",
            ),
        ));
    };
    if selected.status != Phase8MachineCheckStatus::Checked {
        return Ok(Phase8AuxiliaryResult::inconclusive(
            result_id,
            Phase8AuxiliaryResultKind::AxiomPolicy,
            policy_hash,
            artifact_hash,
            selector,
            Phase8AuxiliaryError::value(
                Phase8AuxiliaryReasonCode::AxiomPolicyInconclusive,
                "selector.checker_profile",
                "checked_normalized_result_entry",
                "not_checked",
            ),
        ));
    }
    if selected.result_hash != result_hash {
        return Ok(Phase8AuxiliaryResult::inconclusive(
            result_id,
            Phase8AuxiliaryResultKind::AxiomPolicy,
            policy_hash,
            artifact_hash,
            selector,
            Phase8AuxiliaryError::hash(
                Phase8AuxiliaryReasonCode::AxiomPolicyInconclusive,
                "selector.result_hash",
                selected.result_hash,
                result_hash,
            ),
        ));
    }
    let Some(selected_axiom_report_hash) = selected.axiom_report_hash else {
        return Ok(Phase8AuxiliaryResult::inconclusive(
            result_id,
            Phase8AuxiliaryResultKind::AxiomPolicy,
            policy_hash,
            artifact_hash,
            selector,
            Phase8AuxiliaryError::value(
                Phase8AuxiliaryReasonCode::AxiomPolicyInconclusive,
                "selector.checker_profile",
                "checked_normalized_result_entry",
                "missing_axiom_report_hash",
            ),
        ));
    };
    if selected_axiom_report_hash != axiom_report_hash {
        return Ok(Phase8AuxiliaryResult::inconclusive(
            result_id,
            Phase8AuxiliaryResultKind::AxiomPolicy,
            policy_hash,
            artifact_hash,
            selector,
            Phase8AuxiliaryError::hash(
                Phase8AuxiliaryReasonCode::AxiomPolicyInconclusive,
                "selector.axiom_report_hash",
                selected_axiom_report_hash,
                axiom_report_hash,
            ),
        ));
    }
    let parsed_report_hash = axiom_report.axiom_report_hash();
    if parsed_report_hash != axiom_report_hash {
        return Ok(Phase8AuxiliaryResult::failed(
            result_id,
            Phase8AuxiliaryResultKind::AxiomPolicy,
            policy_hash,
            artifact_hash,
            selector,
            Phase8AuxiliaryError::hash(
                Phase8AuxiliaryReasonCode::AxiomPolicyFailed,
                "selector.axiom_report_hash",
                axiom_report_hash,
                parsed_report_hash,
            ),
        ));
    }
    if axiom_report.module != normalized_result.artifact.module {
        return Ok(Phase8AuxiliaryResult::inconclusive(
            result_id,
            Phase8AuxiliaryResultKind::AxiomPolicy,
            policy_hash,
            artifact_hash,
            selector,
            Phase8AuxiliaryError::value(
                Phase8AuxiliaryReasonCode::AxiomPolicyInconclusive,
                "axiom_report.module",
                &normalized_result.artifact.module,
                &axiom_report.module,
            ),
        ));
    }
    match selected.certificate_hash {
        Some(certificate_hash) if certificate_hash == axiom_report.certificate_hash => {}
        Some(certificate_hash) => {
            return Ok(Phase8AuxiliaryResult::inconclusive(
                result_id,
                Phase8AuxiliaryResultKind::AxiomPolicy,
                policy_hash,
                artifact_hash,
                selector,
                Phase8AuxiliaryError::hash(
                    Phase8AuxiliaryReasonCode::AxiomPolicyInconclusive,
                    "axiom_report.certificate_hash",
                    certificate_hash,
                    axiom_report.certificate_hash,
                ),
            ))
        }
        None => {
            return Ok(Phase8AuxiliaryResult::inconclusive(
                result_id,
                Phase8AuxiliaryResultKind::AxiomPolicy,
                policy_hash,
                artifact_hash,
                selector,
                Phase8AuxiliaryError::value(
                    Phase8AuxiliaryReasonCode::AxiomPolicyInconclusive,
                    "axiom_report.certificate_hash",
                    "present",
                    "missing",
                ),
            ))
        }
    }
    for (index, axiom) in axiom_report.axioms.iter().enumerate() {
        if !axiom_policy.allows(&axiom.name) {
            return Ok(Phase8AuxiliaryResult::failed(
                result_id,
                Phase8AuxiliaryResultKind::AxiomPolicy,
                policy_hash,
                artifact_hash,
                selector,
                Phase8AuxiliaryError::value(
                    Phase8AuxiliaryReasonCode::AxiomPolicyFailed,
                    format!("axiom_report.axioms[{index}].name"),
                    "allowed_axiom",
                    &axiom.name,
                ),
            ));
        }
    }
    Ok(Phase8AuxiliaryResult::passed(
        result_id,
        Phase8AuxiliaryResultKind::AxiomPolicy,
        policy_hash,
        artifact_hash,
        selector,
    ))
}

pub fn phase8_auxiliary_reproducibility_result(
    result_id: impl Into<String>,
    policy: &Phase8RunnerPolicy,
    artifact_hash: Hash,
    baseline: &Phase8MachineCheckResult,
    repeated: &Phase8MachineCheckResult,
) -> Result<Phase8AuxiliaryResult, Phase8CommandError> {
    let selector = Phase8AuxiliarySelector::Reproducibility {
        request_hash: baseline.request_hash,
        checker_profile: baseline.checker.profile.clone(),
        baseline_run_artifact_hash: baseline.run_artifact_hash(),
        repeated_run_artifact_hash: repeated.run_artifact_hash(),
    };
    phase8_auxiliary_reproducibility_result_with_selector(
        result_id,
        policy,
        artifact_hash,
        baseline,
        repeated,
        selector,
    )
}

pub fn phase8_auxiliary_reproducibility_result_with_selector(
    result_id: impl Into<String>,
    policy: &Phase8RunnerPolicy,
    artifact_hash: Hash,
    baseline: &Phase8MachineCheckResult,
    repeated: &Phase8MachineCheckResult,
    selector: Phase8AuxiliarySelector,
) -> Result<Phase8AuxiliaryResult, Phase8CommandError> {
    let result_id = result_id.into();
    validate_auxiliary_result_id(&result_id, Phase8CommandName::AuxiliaryReproducibility)?;
    let policy_hash = policy.policy_hash();
    let Phase8AuxiliarySelector::Reproducibility {
        request_hash,
        checker_profile,
        baseline_run_artifact_hash,
        repeated_run_artifact_hash,
    } = selector.clone()
    else {
        return Ok(Phase8AuxiliaryResult::inconclusive(
            result_id,
            Phase8AuxiliaryResultKind::Reproducibility,
            policy_hash,
            artifact_hash,
            Some(selector),
            Phase8AuxiliaryError::value(
                Phase8AuxiliaryReasonCode::ReproducibilityInconclusive,
                "selector",
                "reproducibility_selector",
                "wrong_type",
            ),
        ));
    };
    if baseline_run_artifact_hash == repeated_run_artifact_hash {
        return Err(phase8_command_value_error(
            Phase8CommandName::AuxiliaryReproducibility,
            "input_reference_invalid",
            "selector.repeated_run_artifact_hash",
            "distinct_run_artifact_hash",
            "duplicate",
        ));
    }
    let selector = Some(selector);
    for (label, result, expected_run_hash) in [
        ("baseline", baseline, baseline_run_artifact_hash),
        ("repeated", repeated, repeated_run_artifact_hash),
    ] {
        if result.run_artifact_hash() != expected_run_hash {
            return Ok(Phase8AuxiliaryResult::inconclusive(
                result_id,
                Phase8AuxiliaryResultKind::Reproducibility,
                policy_hash,
                artifact_hash,
                selector,
                Phase8AuxiliaryError::hash(
                    Phase8AuxiliaryReasonCode::ReproducibilityInconclusive,
                    format!("selector.{label}_run_artifact_hash"),
                    expected_run_hash,
                    result.run_artifact_hash(),
                ),
            ));
        }
    }
    if let Some(error) = phase8_reproducibility_comparability_mismatch(
        policy,
        request_hash,
        &checker_profile,
        baseline,
        repeated,
    ) {
        return Ok(Phase8AuxiliaryResult::inconclusive(
            result_id,
            Phase8AuxiliaryResultKind::Reproducibility,
            policy_hash,
            artifact_hash,
            selector,
            error,
        ));
    }
    if let Some(error) = phase8_reproducibility_identity_mismatch(baseline, repeated) {
        return Ok(Phase8AuxiliaryResult::failed(
            result_id,
            Phase8AuxiliaryResultKind::Reproducibility,
            policy_hash,
            artifact_hash,
            selector,
            error,
        ));
    }
    Ok(Phase8AuxiliaryResult::passed(
        result_id,
        Phase8AuxiliaryResultKind::Reproducibility,
        policy_hash,
        artifact_hash,
        selector,
    ))
}

pub fn phase8_auxiliary_import_certificate_hash_result(
    result_id: impl Into<String>,
    release_policy: &Phase8ReleasePolicy,
    import_lock_manifest_hash: Hash,
    import_lock: &Phase8ImportLockManifest,
    certificate_files: &BTreeMap<String, Vec<u8>>,
) -> Result<Phase8AuxiliaryResult, Phase8CommandError> {
    let result_id = result_id.into();
    validate_auxiliary_result_id(
        &result_id,
        Phase8CommandName::AuxiliaryImportCertificateHash,
    )?;
    if release_policy.mode != Phase8ReleaseMode::HighTrust {
        return Err(phase8_command_value_error(
            Phase8CommandName::AuxiliaryImportCertificateHash,
            "policy_reference_invalid",
            "release_policy.mode",
            "high-trust",
            release_policy.mode.as_str(),
        ));
    }
    let policy_hash = release_policy.policy_hash();
    for (index, import) in import_lock.imports.iter().enumerate() {
        let Some(bytes) = certificate_files.get(&import.certificate.path) else {
            return Ok(Phase8AuxiliaryResult::inconclusive(
                result_id,
                Phase8AuxiliaryResultKind::ImportCertificateHash,
                policy_hash,
                import_lock_manifest_hash,
                None,
                Phase8AuxiliaryError::value(
                    Phase8AuxiliaryReasonCode::ImportCertificateHashInconclusive,
                    format!("import_lock.imports[{index}].certificate.path"),
                    "readable_file",
                    "missing",
                ),
            ));
        };
        let actual_file_hash = phase8_file_hash(bytes);
        if actual_file_hash != import.certificate.file_hash {
            return Ok(Phase8AuxiliaryResult::failed(
                result_id,
                Phase8AuxiliaryResultKind::ImportCertificateHash,
                policy_hash,
                import_lock_manifest_hash,
                None,
                Phase8AuxiliaryError::hash(
                    Phase8AuxiliaryReasonCode::ImportCertificateHashMismatch,
                    format!("import_lock.imports[{index}].certificate.file_hash"),
                    import.certificate.file_hash,
                    actual_file_hash,
                ),
            ));
        }
        let actual_certificate_hash = match phase8_raw_claimed_certificate_hash(bytes) {
            Ok(hash) => hash,
            Err(_) => {
                return Ok(Phase8AuxiliaryResult::failed(
                    result_id,
                    Phase8AuxiliaryResultKind::ImportCertificateHash,
                    policy_hash,
                    import_lock_manifest_hash,
                    None,
                    Phase8AuxiliaryError::value(
                        Phase8AuxiliaryReasonCode::ImportCertificateHashMismatch,
                        format!("import_lock.imports[{index}].certificate.path"),
                        "canonical_certificate",
                        "invalid_certificate_encoding",
                    ),
                ))
            }
        };
        if actual_certificate_hash != import.certificate.certificate_hash {
            return Ok(Phase8AuxiliaryResult::failed(
                result_id,
                Phase8AuxiliaryResultKind::ImportCertificateHash,
                policy_hash,
                import_lock_manifest_hash,
                None,
                Phase8AuxiliaryError::hash(
                    Phase8AuxiliaryReasonCode::ImportCertificateHashMismatch,
                    format!("import_lock.imports[{index}].certificate.certificate_hash"),
                    import.certificate.certificate_hash,
                    actual_certificate_hash,
                ),
            ));
        }
    }
    Ok(Phase8AuxiliaryResult::passed(
        result_id,
        Phase8AuxiliaryResultKind::ImportCertificateHash,
        policy_hash,
        import_lock_manifest_hash,
        None,
    ))
}

pub fn phase8_auxiliary_command_exit_success(
    result: &Result<Phase8AuxiliaryResult, Phase8CommandError>,
) -> bool {
    result.is_ok()
}

pub fn phase8_auxiliary_result_passes_release_condition(result: &Phase8AuxiliaryResult) -> bool {
    result.status == Phase8AuxiliaryStatus::Passed
}

pub fn phase8_auxiliary_results_all_passed(results: &[Phase8AuxiliaryResult]) -> bool {
    results
        .iter()
        .all(phase8_auxiliary_result_passes_release_condition)
}

pub fn phase8_auxiliary_output_write_failure(
    command: Phase8CommandName,
    field: impl Into<String>,
) -> Phase8CommandError {
    phase8_command_value_error(
        command,
        "output_write_failure",
        field,
        "write_success",
        "write_failed",
    )
}

pub fn parse_phase8_ai_audit_input_policy(
    source: &str,
) -> Result<Phase8AiAuditInputPolicy, Phase8AuditSidecarValidationError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        Phase8AuditSidecarValidationError::actual_value(
            Phase8AuditSidecarValidationReasonCode::InputPolicyJsonInvalid,
            "input_policy.path",
            "invalid_json",
        )
    })?;
    phase8_parse_ai_audit_input_policy_value(document.root())
}

pub fn phase8_ai_audit_input_policy_hash(
    source: &str,
) -> Result<Hash, Phase8AuditSidecarValidationError> {
    Ok(parse_phase8_ai_audit_input_policy(source)?.input_policy_hash())
}

pub fn parse_phase8_ai_audit_sidecar(
    source: &str,
) -> Result<Phase8AiAuditSidecar, Phase8AuditSidecarValidationError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        Phase8AuditSidecarValidationError::actual_value(
            Phase8AuditSidecarValidationReasonCode::SidecarJsonInvalid,
            "sidecar.path",
            "invalid_json",
        )
    })?;
    phase8_parse_ai_audit_sidecar_value(document.root())
}

pub fn phase8_validate_ai_audit_sidecar_schema_only(
    sidecar_source: &str,
) -> Phase8AuditSidecarValidationResult {
    let sidecar_file_hash = phase8_file_hash(sidecar_source.as_bytes());
    let mode = Phase8AuditSidecarValidationMode::SchemaOnly;
    let document = match JsonDocument::parse(sidecar_source) {
        Ok(document) => document,
        Err(_) => {
            return Phase8AuditSidecarValidationResult::failed(
                mode,
                Some(sidecar_file_hash),
                None,
                None,
                Phase8AuditSidecarValidationError::actual_value(
                    Phase8AuditSidecarValidationReasonCode::SidecarJsonInvalid,
                    "sidecar.path",
                    "invalid_json",
                ),
            )
        }
    };
    match phase8_parse_ai_audit_sidecar_value(document.root()) {
        Ok(_) => Phase8AuditSidecarValidationResult::valid(mode, sidecar_file_hash, None, None),
        Err(error) => Phase8AuditSidecarValidationResult::failed(
            mode,
            Some(sidecar_file_hash),
            None,
            None,
            error,
        ),
    }
}

pub fn phase8_validate_ai_audit_sidecar_cross_artifact(
    sidecar_source: &str,
    input_policy_source: &str,
    input_policy_reference_hash: Hash,
    machine_results: &[Phase8MachineCheckResult],
    normalized_results: &[Phase8NormalizedCheckResult],
) -> Phase8AuditSidecarValidationResult {
    let sidecar_file_hash = phase8_file_hash(sidecar_source.as_bytes());
    let mode = Phase8AuditSidecarValidationMode::CrossArtifact;
    let document = match JsonDocument::parse(sidecar_source) {
        Ok(document) => document,
        Err(_) => {
            return Phase8AuditSidecarValidationResult::failed(
                mode,
                Some(sidecar_file_hash),
                None,
                None,
                Phase8AuditSidecarValidationError::actual_value(
                    Phase8AuditSidecarValidationReasonCode::SidecarJsonInvalid,
                    "sidecar.path",
                    "invalid_json",
                ),
            )
        }
    };
    let sidecar = match phase8_parse_ai_audit_sidecar_value(document.root()) {
        Ok(sidecar) => sidecar,
        Err(error) => {
            return Phase8AuditSidecarValidationResult::failed(
                mode,
                Some(sidecar_file_hash),
                None,
                None,
                error,
            )
        }
    };

    let policy_document = match JsonDocument::parse(input_policy_source) {
        Ok(document) => document,
        Err(_) => {
            return Phase8AuditSidecarValidationResult::failed(
                mode,
                Some(sidecar_file_hash),
                Some(input_policy_reference_hash),
                Some(&sidecar),
                Phase8AuditSidecarValidationError::value(
                    Phase8AuditSidecarValidationReasonCode::InputPolicyJsonInvalid,
                    "input_policy.path",
                    "valid_json",
                    "invalid_json",
                ),
            )
        }
    };
    let input_policy = match phase8_parse_ai_audit_input_policy_value(policy_document.root()) {
        Ok(policy) => policy,
        Err(error) => {
            return Phase8AuditSidecarValidationResult::failed(
                mode,
                Some(sidecar_file_hash),
                Some(input_policy_reference_hash),
                Some(&sidecar),
                error,
            )
        }
    };
    let input_policy_hash = input_policy.input_policy_hash();
    if sidecar.input_policy.hash != input_policy_reference_hash {
        return Phase8AuditSidecarValidationResult::failed(
            mode,
            Some(sidecar_file_hash),
            Some(input_policy_reference_hash),
            Some(&sidecar),
            Phase8AuditSidecarValidationError::hash(
                Phase8AuditSidecarValidationReasonCode::InputPolicyHashMismatch,
                "input_policy.hash",
                input_policy_reference_hash,
                sidecar.input_policy.hash,
            ),
        );
    }
    if input_policy_hash != input_policy_reference_hash {
        return Phase8AuditSidecarValidationResult::failed(
            mode,
            Some(sidecar_file_hash),
            Some(input_policy_reference_hash),
            Some(&sidecar),
            Phase8AuditSidecarValidationError::hash(
                Phase8AuditSidecarValidationReasonCode::InputPolicyHashMismatch,
                "input_policy.hash",
                input_policy_reference_hash,
                input_policy_hash,
            ),
        );
    }

    if let Some(error) = phase8_ai_audit_copied_policy_mismatch(&sidecar, &input_policy) {
        return Phase8AuditSidecarValidationResult::failed(
            mode,
            Some(sidecar_file_hash),
            Some(input_policy_reference_hash),
            Some(&sidecar),
            error,
        );
    }
    if let Some(error) = phase8_ai_audit_policy_gated_field_error(&sidecar, &input_policy) {
        return Phase8AuditSidecarValidationResult::failed(
            mode,
            Some(sidecar_file_hash),
            Some(input_policy_reference_hash),
            Some(&sidecar),
            error,
        );
    }

    let source =
        match phase8_resolve_ai_audit_sidecar_source(&sidecar, machine_results, normalized_results)
        {
            Ok(source) => source,
            Err(error) => {
                return Phase8AuditSidecarValidationResult::failed(
                    mode,
                    Some(sidecar_file_hash),
                    Some(input_policy_reference_hash),
                    Some(&sidecar),
                    error,
                )
            }
        };
    if let Some(error) = phase8_validate_ai_audit_source_dependent_fields(&sidecar, &source) {
        return Phase8AuditSidecarValidationResult::failed(
            mode,
            Some(sidecar_file_hash),
            Some(input_policy_reference_hash),
            Some(&sidecar),
            error,
        );
    }
    let prompt_input = phase8_ai_audit_prompt_input(&sidecar, &input_policy, &source);
    let expected_prompt_hash = prompt_input.prompt_hash();
    if expected_prompt_hash != sidecar.ai.prompt_hash {
        return Phase8AuditSidecarValidationResult::failed(
            mode,
            Some(sidecar_file_hash),
            Some(input_policy_reference_hash),
            Some(&sidecar),
            Phase8AuditSidecarValidationError::hash(
                Phase8AuditSidecarValidationReasonCode::PromptHashMismatch,
                "ai.prompt_hash",
                expected_prompt_hash,
                sidecar.ai.prompt_hash,
            ),
        );
    }
    Phase8AuditSidecarValidationResult::valid(
        mode,
        sidecar_file_hash,
        Some(input_policy_reference_hash),
        Some(&sidecar),
    )
}

pub fn phase8_ai_audit_prompt_input_for_sidecar(
    sidecar: &Phase8AiAuditSidecar,
    input_policy: &Phase8AiAuditInputPolicy,
    machine_result: Option<&Phase8MachineCheckResult>,
    normalized_result: Option<&Phase8NormalizedCheckResult>,
) -> Option<Phase8AiAuditPromptInput> {
    let source = match sidecar.source.kind {
        Phase8AiAuditSidecarSourceKind::MachineResult => {
            Phase8ResolvedAiAuditSource::MachineResult {
                machine_result: machine_result?,
                normalized_result: if sidecar.source.normalized_result_hash.is_some() {
                    Some(normalized_result?)
                } else {
                    normalized_result
                },
            }
        }
        Phase8AiAuditSidecarSourceKind::NormalizedComparison => {
            Phase8ResolvedAiAuditSource::NormalizedComparison {
                normalized_result: normalized_result?,
            }
        }
    };
    Some(phase8_ai_audit_prompt_input(sidecar, input_policy, &source))
}

pub fn phase8_ai_sidecar_diagnostic_targets_from_normalized_result(
    normalized_result: &Phase8NormalizedCheckResult,
) -> Vec<Phase8RequiredAiSidecarDiagnosticTarget> {
    let normalized_result_hash = normalized_result.normalized_result_hash();
    let mut targets = Vec::new();
    for (index, entry) in normalized_result.results.iter().enumerate() {
        if entry.status == Phase8MachineCheckStatus::Failed {
            targets.push(Phase8RequiredAiSidecarDiagnosticTarget::MachineResult {
                result_index: index,
                normalized_result_hash,
                checker_profile: entry.checker_profile.clone(),
                request_hash: entry.request_hash,
                result_hash: entry.result_hash,
                policy_hash: entry.policy_hash,
            });
        }
    }
    if phase8_normalized_comparison_requires_ai_diagnostic(normalized_result.comparison.status) {
        targets.push(
            Phase8RequiredAiSidecarDiagnosticTarget::NormalizedComparison {
                normalized_result_hash,
            },
        );
    }
    targets
}

pub fn phase8_required_ai_sidecar_diagnostic_targets(
    release_policy: &Phase8ReleasePolicy,
    normalized_result: &Phase8NormalizedCheckResult,
) -> Vec<Phase8RequiredAiSidecarDiagnosticTarget> {
    if phase8_release_policy_required_ai_diagnostic_input_policy_hash(release_policy).is_none() {
        return Vec::new();
    }
    phase8_ai_sidecar_diagnostic_targets_from_normalized_result(normalized_result)
}

pub fn phase8_release_target_has_required_ai_sidecar_diagnostic_targets(
    release_policy: &Phase8ReleasePolicy,
    normalized_result: &Phase8NormalizedCheckResult,
) -> bool {
    !phase8_required_ai_sidecar_diagnostic_targets(release_policy, normalized_result).is_empty()
}

pub fn evaluate_required_ai_sidecar_diagnostics(
    release_policy: &Phase8ReleasePolicy,
    ci_diagnostic_targets: &[Phase8CiDiagnosticTargetContext],
    ai_sidecars: &[Phase8ResolvedAiAuditSidecarEntry],
    audit_sidecar_validation_results: &[Phase8ResolvedAuditSidecarValidationEntry],
    ai_sidecar_diagnostic_results: &[Phase8ResolvedAiSidecarDiagnosticResultEntry],
) -> Result<Phase8RequiredAiSidecarDiagnosticEvaluation, Phase8AiSidecarDiagnosticEvaluationFailure>
{
    let Some(input_policy_hash) =
        phase8_release_policy_required_ai_diagnostic_input_policy_hash(release_policy)
    else {
        return Ok(Phase8RequiredAiSidecarDiagnosticEvaluation {
            recomputed_results: Vec::new(),
            pass_failure: None,
        });
    };

    phase8_validate_required_ai_sidecar_diagnostic_entry_metadata(
        ai_sidecars,
        audit_sidecar_validation_results,
        ai_sidecar_diagnostic_results,
    )?;
    phase8_validate_ci_diagnostic_target_uniqueness(ci_diagnostic_targets)?;

    let policy_hash = release_policy.policy_hash();
    let target_sets = ci_diagnostic_targets
        .iter()
        .map(|target| phase8_ai_sidecar_diagnostic_targets_from_normalized_result(&target.artifact))
        .collect::<Vec<_>>();
    phase8_validate_saved_ai_sidecar_diagnostic_target_counts(
        policy_hash,
        input_policy_hash,
        ci_diagnostic_targets,
        &target_sets,
        ai_sidecar_diagnostic_results,
    )?;

    let mut recomputed_results = Vec::new();
    for (context_index, context) in ci_diagnostic_targets.iter().enumerate() {
        let normalized_result_hash = context.artifact.normalized_result_hash();
        let targets = &target_sets[context_index];
        let target_count = targets.len() as u64;
        let error = phase8_first_required_ai_sidecar_diagnostic_error(
            context,
            targets,
            input_policy_hash,
            ai_sidecars,
            audit_sidecar_validation_results,
        );
        let result = match error {
            Some(error) => Phase8AiSidecarDiagnosticResult::failed(
                policy_hash,
                input_policy_hash,
                normalized_result_hash,
                target_count,
                error,
            ),
            None => Phase8AiSidecarDiagnosticResult::passed(
                policy_hash,
                input_policy_hash,
                normalized_result_hash,
                target_count,
            ),
        };
        recomputed_results.push(result);
    }

    let pass_failure = phase8_first_required_ai_sidecar_diagnostic_pass_failure(
        policy_hash,
        input_policy_hash,
        ci_diagnostic_targets,
        &target_sets,
        &recomputed_results,
        ai_sidecar_diagnostic_results,
    );
    Ok(Phase8RequiredAiSidecarDiagnosticEvaluation {
        recomputed_results,
        pass_failure,
    })
}

fn phase8_release_policy_required_ai_diagnostic_input_policy_hash(
    release_policy: &Phase8ReleasePolicy,
) -> Option<Hash> {
    (release_policy.ai_triage.enabled && release_policy.ai_triage.required)
        .then_some(release_policy.ai_triage.input_policy_hash)
        .flatten()
}

const fn phase8_normalized_comparison_requires_ai_diagnostic(
    status: Phase8NormalizedComparisonStatus,
) -> bool {
    matches!(
        status,
        Phase8NormalizedComparisonStatus::Disagreement
            | Phase8NormalizedComparisonStatus::MissingCheckerResult
            | Phase8NormalizedComparisonStatus::PolicyFailure
            | Phase8NormalizedComparisonStatus::Inconclusive
    )
}

fn phase8_validate_required_ai_sidecar_diagnostic_entry_metadata(
    ai_sidecars: &[Phase8ResolvedAiAuditSidecarEntry],
    audit_sidecar_validation_results: &[Phase8ResolvedAuditSidecarValidationEntry],
    ai_sidecar_diagnostic_results: &[Phase8ResolvedAiSidecarDiagnosticResultEntry],
) -> Result<(), Phase8AiSidecarDiagnosticEvaluationFailure> {
    for (index, entry) in ai_sidecars.iter().enumerate() {
        if !phase8_valid_workspace_relative_path(&entry.path) {
            return Err(Phase8AiSidecarDiagnosticEvaluationFailure::value(
                Phase8AiSidecarDiagnosticEvaluationFailureReasonCode::AiSidecarInputSchemaInvalid,
                format!("ai_sidecars[{index}].path"),
                "workspace_relative_path",
                "invalid_path",
            ));
        }
    }
    for (index, entry) in audit_sidecar_validation_results.iter().enumerate() {
        if !phase8_valid_workspace_relative_path(&entry.path) {
            return Err(Phase8AiSidecarDiagnosticEvaluationFailure::value(
                Phase8AiSidecarDiagnosticEvaluationFailureReasonCode::AuditSidecarValidationInputSchemaInvalid,
                format!("audit_sidecar_validation_results[{index}].path"),
                "workspace_relative_path",
                "invalid_path",
            ));
        }
    }
    for (index, entry) in ai_sidecar_diagnostic_results.iter().enumerate() {
        if !phase8_valid_workspace_relative_path(&entry.path) {
            return Err(Phase8AiSidecarDiagnosticEvaluationFailure::value(
                Phase8AiSidecarDiagnosticEvaluationFailureReasonCode::AiSidecarDiagnosticInputSchemaInvalid,
                format!("ai_sidecar_diagnostic_results[{index}].path"),
                "workspace_relative_path",
                "invalid_path",
            ));
        }
    }
    Ok(())
}

fn phase8_validate_ci_diagnostic_target_uniqueness(
    ci_diagnostic_targets: &[Phase8CiDiagnosticTargetContext],
) -> Result<(), Phase8AiSidecarDiagnosticEvaluationFailure> {
    let mut artifact_hashes = BTreeMap::<Hash, usize>::new();
    let mut normalized_result_hashes = BTreeMap::<Hash, usize>::new();
    for (index, target) in ci_diagnostic_targets.iter().enumerate() {
        let artifact_hash = target.artifact.artifact_hash();
        if artifact_hashes.insert(artifact_hash, index).is_some() {
            return Err(Phase8AiSidecarDiagnosticEvaluationFailure::value(
                Phase8AiSidecarDiagnosticEvaluationFailureReasonCode::CiDiagnosticTargetDuplicate,
                format!("ci_diagnostic_targets[{index}].artifact_hash"),
                "unique_artifact_hash",
                "duplicate",
            ));
        }
        let normalized_result_hash = target.artifact.normalized_result_hash();
        if normalized_result_hashes
            .insert(normalized_result_hash, index)
            .is_some()
        {
            return Err(Phase8AiSidecarDiagnosticEvaluationFailure::value(
                Phase8AiSidecarDiagnosticEvaluationFailureReasonCode::CiDiagnosticTargetDuplicate,
                format!("ci_diagnostic_targets[{index}].normalized_result_hash"),
                "unique_normalized_result_hash",
                "duplicate",
            ));
        }
    }
    Ok(())
}

fn phase8_validate_saved_ai_sidecar_diagnostic_target_counts(
    policy_hash: Hash,
    input_policy_hash: Hash,
    ci_diagnostic_targets: &[Phase8CiDiagnosticTargetContext],
    target_sets: &[Vec<Phase8RequiredAiSidecarDiagnosticTarget>],
    ai_sidecar_diagnostic_results: &[Phase8ResolvedAiSidecarDiagnosticResultEntry],
) -> Result<(), Phase8AiSidecarDiagnosticEvaluationFailure> {
    for (diagnostic_index, diagnostic) in ai_sidecar_diagnostic_results.iter().enumerate() {
        for (context_index, context) in ci_diagnostic_targets.iter().enumerate() {
            let target_count = target_sets[context_index].len() as u64;
            if target_count == 0 {
                continue;
            }
            let artifact = &diagnostic.artifact;
            if artifact.policy_hash == policy_hash
                && artifact.input_policy_hash == input_policy_hash
                && artifact.normalized_result_hash == context.artifact.normalized_result_hash()
                && artifact.target_count != target_count
            {
                return Err(Phase8AiSidecarDiagnosticEvaluationFailure::value(
                    Phase8AiSidecarDiagnosticEvaluationFailureReasonCode::AiSidecarDiagnosticTargetCountMismatch,
                    format!("ai_sidecar_diagnostic[{diagnostic_index}].artifact.target_count"),
                    target_count.to_string(),
                    artifact.target_count.to_string(),
                ));
            }
        }
    }
    Ok(())
}

fn phase8_first_required_ai_sidecar_diagnostic_error(
    context: &Phase8CiDiagnosticTargetContext,
    targets: &[Phase8RequiredAiSidecarDiagnosticTarget],
    input_policy_hash: Hash,
    ai_sidecars: &[Phase8ResolvedAiAuditSidecarEntry],
    audit_sidecar_validation_results: &[Phase8ResolvedAuditSidecarValidationEntry],
) -> Option<Phase8AiSidecarDiagnosticError> {
    for (target_index, target) in targets.iter().enumerate() {
        let error = match target {
            Phase8RequiredAiSidecarDiagnosticTarget::MachineResult { .. } => {
                phase8_required_machine_result_ai_sidecar_diagnostic_error(
                    target_index,
                    target,
                    context,
                    input_policy_hash,
                    ai_sidecars,
                    audit_sidecar_validation_results,
                )
            }
            Phase8RequiredAiSidecarDiagnosticTarget::NormalizedComparison {
                normalized_result_hash,
            } => phase8_required_normalized_comparison_ai_sidecar_diagnostic_error(
                target_index,
                *normalized_result_hash,
                input_policy_hash,
                ai_sidecars,
                audit_sidecar_validation_results,
            ),
        };
        if error.is_some() {
            return error;
        }
    }
    None
}

fn phase8_required_machine_result_ai_sidecar_diagnostic_error(
    target_index: usize,
    target: &Phase8RequiredAiSidecarDiagnosticTarget,
    context: &Phase8CiDiagnosticTargetContext,
    input_policy_hash: Hash,
    ai_sidecars: &[Phase8ResolvedAiAuditSidecarEntry],
    audit_sidecar_validation_results: &[Phase8ResolvedAuditSidecarValidationEntry],
) -> Option<Phase8AiSidecarDiagnosticError> {
    let Phase8RequiredAiSidecarDiagnosticTarget::MachineResult {
        checker_profile,
        request_hash,
        result_hash,
        policy_hash,
        ..
    } = target
    else {
        return None;
    };
    let selected_raw_results = context
        .normalizer_machine_results
        .iter()
        .filter(|result| {
            result.checker.profile == *checker_profile
                && result.request_hash == *request_hash
                && result.result_hash() == *result_hash
                && result.policy.hash == *policy_hash
        })
        .collect::<Vec<_>>();
    let selected_raw_result =
        match selected_raw_results.as_slice() {
            [] => {
                return Some(Phase8AiSidecarDiagnosticError::value(
                    Phase8AiSidecarDiagnosticFailureReasonCode::RequiredAiSidecarSelectedRawMissing,
                    format!("required_ai_sidecar_targets[{target_index}].selected_raw_result"),
                    "exactly_one_selected_raw_machine_check_result",
                    "missing",
                ))
            }
            [selected] => *selected,
            _ => return Some(Phase8AiSidecarDiagnosticError::value(
                Phase8AiSidecarDiagnosticFailureReasonCode::RequiredAiSidecarSelectedRawDuplicate,
                format!("required_ai_sidecar_targets[{target_index}].selected_raw_result"),
                "exactly_one_selected_raw_machine_check_result",
                "duplicate",
            )),
        };
    let normalized_result_hash = context.artifact.normalized_result_hash();
    let candidates = ai_sidecars
        .iter()
        .enumerate()
        .filter(|(_, sidecar)| {
            phase8_machine_result_diagnostic_sidecar_matches(
                &sidecar.artifact,
                selected_raw_result,
                normalized_result_hash,
            )
        })
        .collect::<Vec<_>>();
    let (sidecar_index, sidecar) = match candidates.as_slice() {
        [] => {
            return Some(Phase8AiSidecarDiagnosticError::value(
                Phase8AiSidecarDiagnosticFailureReasonCode::RequiredAiSidecarMissing,
                format!("required_ai_sidecar_targets[{target_index}].ai_sidecar"),
                "exactly_one_ai_sidecar",
                "missing",
            ))
        }
        [(index, sidecar)] => (*index, *sidecar),
        _ => {
            return Some(Phase8AiSidecarDiagnosticError::value(
                Phase8AiSidecarDiagnosticFailureReasonCode::RequiredAiSidecarDuplicate,
                format!("required_ai_sidecar_targets[{target_index}].ai_sidecar"),
                "exactly_one_ai_sidecar",
                "duplicate",
            ))
        }
    };
    phase8_required_ai_sidecar_candidate_error(
        sidecar_index,
        sidecar,
        Some(selected_raw_result.run_artifact_hash()),
        input_policy_hash,
        audit_sidecar_validation_results,
    )
}

fn phase8_required_normalized_comparison_ai_sidecar_diagnostic_error(
    target_index: usize,
    normalized_result_hash: Hash,
    input_policy_hash: Hash,
    ai_sidecars: &[Phase8ResolvedAiAuditSidecarEntry],
    audit_sidecar_validation_results: &[Phase8ResolvedAuditSidecarValidationEntry],
) -> Option<Phase8AiSidecarDiagnosticError> {
    let candidates = ai_sidecars
        .iter()
        .enumerate()
        .filter(|(_, sidecar)| {
            sidecar.artifact.source.kind == Phase8AiAuditSidecarSourceKind::NormalizedComparison
                && sidecar.artifact.source.normalized_result_hash == Some(normalized_result_hash)
        })
        .collect::<Vec<_>>();
    let (sidecar_index, sidecar) = match candidates.as_slice() {
        [] => {
            return Some(Phase8AiSidecarDiagnosticError::value(
                Phase8AiSidecarDiagnosticFailureReasonCode::RequiredAiSidecarMissing,
                format!("required_ai_sidecar_targets[{target_index}].ai_sidecar"),
                "exactly_one_ai_sidecar",
                "missing",
            ))
        }
        [(index, sidecar)] => (*index, *sidecar),
        _ => {
            return Some(Phase8AiSidecarDiagnosticError::value(
                Phase8AiSidecarDiagnosticFailureReasonCode::RequiredAiSidecarDuplicate,
                format!("required_ai_sidecar_targets[{target_index}].ai_sidecar"),
                "exactly_one_ai_sidecar",
                "duplicate",
            ))
        }
    };
    phase8_required_ai_sidecar_candidate_error(
        sidecar_index,
        sidecar,
        None,
        input_policy_hash,
        audit_sidecar_validation_results,
    )
}

fn phase8_machine_result_diagnostic_sidecar_matches(
    sidecar: &Phase8AiAuditSidecar,
    selected_raw_result: &Phase8MachineCheckResult,
    normalized_result_hash: Hash,
) -> bool {
    sidecar.source.kind == Phase8AiAuditSidecarSourceKind::MachineResult
        && sidecar.source.request_hash == Some(selected_raw_result.request_hash)
        && sidecar.source.result_hash == Some(selected_raw_result.result_hash())
        && sidecar
            .source
            .normalized_result_hash
            .is_none_or(|hash| hash == normalized_result_hash)
}

fn phase8_required_ai_sidecar_candidate_error(
    sidecar_index: usize,
    sidecar: &Phase8ResolvedAiAuditSidecarEntry,
    selected_raw_run_artifact_hash: Option<Hash>,
    input_policy_hash: Hash,
    audit_sidecar_validation_results: &[Phase8ResolvedAuditSidecarValidationEntry],
) -> Option<Phase8AiSidecarDiagnosticError> {
    if sidecar.artifact.input_policy.hash != input_policy_hash {
        return Some(Phase8AiSidecarDiagnosticError::hash(
            Phase8AiSidecarDiagnosticFailureReasonCode::RequiredAiSidecarInputPolicyMismatch,
            format!("ai_sidecar[{sidecar_index}].artifact.input_policy.hash"),
            input_policy_hash,
            sidecar.artifact.input_policy.hash,
        ));
    }
    let validation_candidates = audit_sidecar_validation_results
        .iter()
        .enumerate()
        .filter(|(_, validation)| validation.artifact.sidecar_file_hash == Some(sidecar.file_hash))
        .collect::<Vec<_>>();
    let (validation_index, validation) =
        match validation_candidates.as_slice() {
            [] => {
                return Some(Phase8AiSidecarDiagnosticError::value(
                    Phase8AiSidecarDiagnosticFailureReasonCode::RequiredAiSidecarValidationMissing,
                    format!("ai_sidecar[{sidecar_index}].validation_response"),
                    "exactly_one_audit_sidecar_validation_result",
                    "missing",
                ))
            }
            [(index, validation)] => (*index, *validation),
            _ => return Some(Phase8AiSidecarDiagnosticError::value(
                Phase8AiSidecarDiagnosticFailureReasonCode::RequiredAiSidecarValidationDuplicate,
                format!("ai_sidecar[{sidecar_index}].validation_response"),
                "exactly_one_audit_sidecar_validation_result",
                "duplicate",
            )),
        };
    if validation.artifact.mode != Phase8AuditSidecarValidationMode::CrossArtifact {
        return Some(Phase8AiSidecarDiagnosticError::value(
            Phase8AiSidecarDiagnosticFailureReasonCode::RequiredAiSidecarValidationModeMismatch,
            format!("audit_sidecar_validation[{validation_index}].artifact.mode"),
            Phase8AuditSidecarValidationMode::CrossArtifact.as_str(),
            validation.artifact.mode.as_str(),
        ));
    }
    if validation.artifact.status != Phase8AuditSidecarValidationStatus::Valid {
        return Some(Phase8AiSidecarDiagnosticError::value(
            Phase8AiSidecarDiagnosticFailureReasonCode::RequiredAiSidecarValidationFailed,
            format!("audit_sidecar_validation[{validation_index}].artifact.status"),
            Phase8AuditSidecarValidationStatus::Valid.as_str(),
            validation.artifact.status.as_str(),
        ));
    }
    if validation.artifact.input_policy_hash != Some(input_policy_hash) {
        return Some(phase8_diagnostic_expected_hash_actual_optional_hash_error(
            Phase8AiSidecarDiagnosticFailureReasonCode::RequiredAiSidecarValidationInputPolicyMismatch,
            format!("audit_sidecar_validation[{validation_index}].artifact.input_policy_hash"),
            input_policy_hash,
            validation.artifact.input_policy_hash,
        ));
    }
    if !phase8_audit_sidecar_validation_source_matches_sidecar(
        &validation.artifact,
        &sidecar.artifact,
    ) {
        return Some(Phase8AiSidecarDiagnosticError::value(
            Phase8AiSidecarDiagnosticFailureReasonCode::RequiredAiSidecarValidationSourceMismatch,
            format!("audit_sidecar_validation[{validation_index}].artifact"),
            "source_kind_specific_matching_key",
            "source_key_mismatch",
        ));
    }
    if let Some(expected_run_artifact_hash) = selected_raw_run_artifact_hash {
        let actual_run_artifact_hash = sidecar
            .artifact
            .source
            .run_artifact_hash
            .expect("machine-result AI sidecar source is schema-valid");
        if actual_run_artifact_hash != expected_run_artifact_hash {
            return Some(Phase8AiSidecarDiagnosticError::hash(
                Phase8AiSidecarDiagnosticFailureReasonCode::RequiredAiSidecarSourceMismatch,
                format!("ai_sidecar[{sidecar_index}].artifact.source.run_artifact_hash"),
                expected_run_artifact_hash,
                actual_run_artifact_hash,
            ));
        }
    }
    None
}

fn phase8_diagnostic_expected_hash_actual_optional_hash_error(
    reason_code: Phase8AiSidecarDiagnosticFailureReasonCode,
    field: impl Into<String>,
    expected_hash: Hash,
    actual_hash: Option<Hash>,
) -> Phase8AiSidecarDiagnosticError {
    match actual_hash {
        Some(actual_hash) => {
            Phase8AiSidecarDiagnosticError::hash(reason_code, field, expected_hash, actual_hash)
        }
        None => Phase8AiSidecarDiagnosticError {
            reason_code,
            field: field.into(),
            expected_hash: Some(expected_hash),
            actual_hash: None,
            expected_value: None,
            actual_value: Some("missing".to_owned()),
        },
    }
}

fn phase8_audit_sidecar_validation_source_matches_sidecar(
    validation: &Phase8AuditSidecarValidationResult,
    sidecar: &Phase8AiAuditSidecar,
) -> bool {
    if validation.source_kind != Some(sidecar.source.kind) {
        return false;
    }
    match sidecar.source.kind {
        Phase8AiAuditSidecarSourceKind::MachineResult => {
            validation.source_result_hash == sidecar.source.result_hash
                && match sidecar.source.normalized_result_hash {
                    Some(hash) => validation.source_normalized_result_hash == Some(hash),
                    None => validation.source_normalized_result_hash.is_none(),
                }
        }
        Phase8AiAuditSidecarSourceKind::NormalizedComparison => {
            validation.source_result_hash.is_none()
                && validation.source_normalized_result_hash == sidecar.source.normalized_result_hash
        }
    }
}

fn phase8_first_required_ai_sidecar_diagnostic_pass_failure(
    policy_hash: Hash,
    input_policy_hash: Hash,
    ci_diagnostic_targets: &[Phase8CiDiagnosticTargetContext],
    target_sets: &[Vec<Phase8RequiredAiSidecarDiagnosticTarget>],
    recomputed_results: &[Phase8AiSidecarDiagnosticResult],
    saved_results: &[Phase8ResolvedAiSidecarDiagnosticResultEntry],
) -> Option<Phase8AiSidecarDiagnosticPassFailure> {
    for (context_index, context) in ci_diagnostic_targets.iter().enumerate() {
        if target_sets[context_index].is_empty() {
            continue;
        }
        let normalized_result_hash = context.artifact.normalized_result_hash();
        let candidates = saved_results
            .iter()
            .enumerate()
            .filter(|(_, saved)| {
                let artifact = &saved.artifact;
                artifact.policy_hash == policy_hash
                    && artifact.input_policy_hash == input_policy_hash
                    && artifact.normalized_result_hash == normalized_result_hash
            })
            .collect::<Vec<_>>();
        let (saved_index, saved) = match candidates.as_slice() {
            [] => {
                return Some(Phase8AiSidecarDiagnosticPassFailure::value(
                    Phase8AiSidecarDiagnosticPassFailureReasonCode::RequiredAiSidecarDiagnosticMissing,
                    format!("ci_diagnostic_targets[{context_index}].ai_sidecar_diagnostic_result"),
                    "exactly_one_saved_ai_sidecar_diagnostic_result",
                    "missing",
                ))
            }
            [(index, saved)] => (*index, *saved),
            _ => {
                return Some(Phase8AiSidecarDiagnosticPassFailure::value(
                    Phase8AiSidecarDiagnosticPassFailureReasonCode::RequiredAiSidecarDiagnosticDuplicate,
                    format!("ci_diagnostic_targets[{context_index}].ai_sidecar_diagnostic_result"),
                    "exactly_one_saved_ai_sidecar_diagnostic_result",
                    "duplicate",
                ))
            }
        };
        if saved.artifact.status != Phase8AiSidecarDiagnosticStatus::Passed {
            return Some(Phase8AiSidecarDiagnosticPassFailure::value(
                Phase8AiSidecarDiagnosticPassFailureReasonCode::RequiredAiSidecarDiagnosticNotPassed,
                format!("ai_sidecar_diagnostic[{saved_index}].artifact.status"),
                Phase8AiSidecarDiagnosticStatus::Passed.as_str(),
                saved.artifact.status.as_str(),
            ));
        }
        let expected_json = recomputed_results[context_index].canonical_json();
        let actual_json = saved.artifact.canonical_json();
        if expected_json != actual_json {
            return Some(Phase8AiSidecarDiagnosticPassFailure::hash(
                Phase8AiSidecarDiagnosticPassFailureReasonCode::RequiredAiSidecarDiagnosticCanonicalMismatch,
                format!("ai_sidecar_diagnostic[{saved_index}].artifact"),
                phase8_sha256(expected_json.as_bytes()),
                phase8_sha256(actual_json.as_bytes()),
            ));
        }
    }
    None
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

fn phase8_build_normalized_comparison_parts(
    policy: &Phase8RunnerPolicy,
    normalized_policy_hash: Hash,
    artifact_hash: Hash,
    entries: &[Phase8NormalizedCheckResultEntry],
) -> Phase8NormalizedComparison {
    let participating_entries = entries
        .iter()
        .filter(|entry| phase8_policy_has_checker_profile(policy, &entry.checker_profile))
        .collect::<Vec<_>>();

    let policy_reasons =
        phase8_normalized_comparison_policy_reasons(policy, normalized_policy_hash, entries);
    if !policy_reasons.is_empty() {
        return phase8_normalized_comparison_with_reasons(
            Phase8NormalizedComparisonStatus::PolicyFailure,
            policy_reasons,
        );
    }

    let copied_policy_reasons = entries
        .iter()
        .filter(|entry| !phase8_malformed_process_state(entry))
        .filter_map(|entry| {
            let error = entry.error.as_ref()?;
            (error.kind == "policy_failure")
                .then(|| phase8_status_reason_from_error(entry, "policy_failure"))
        })
        .collect::<Vec<_>>();
    if !copied_policy_reasons.is_empty() {
        return phase8_normalized_comparison_with_reasons(
            Phase8NormalizedComparisonStatus::PolicyFailure,
            copied_policy_reasons,
        );
    }

    let identity_reasons =
        phase8_normalized_comparison_identity_reasons(policy, &participating_entries);
    if !identity_reasons.is_empty() {
        return phase8_normalized_comparison_with_reasons(
            Phase8NormalizedComparisonStatus::PolicyFailure,
            identity_reasons,
        );
    }

    let malformed_reasons = participating_entries
        .iter()
        .copied()
        .filter(|entry| phase8_malformed_process_state(entry))
        .map(phase8_malformed_process_state_reason)
        .collect::<Vec<_>>();
    if !malformed_reasons.is_empty() {
        return phase8_normalized_comparison_with_reasons(
            Phase8NormalizedComparisonStatus::Inconclusive,
            malformed_reasons,
        );
    }

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
    for entry in &participating_entries {
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
        phase8_sort_normalized_disagreements(&mut disagreements);
        return Phase8NormalizedComparison {
            status: Phase8NormalizedComparisonStatus::Disagreement,
            matching_fields: Vec::new(),
            missing_checker_profiles: Vec::new(),
            disagreements,
            status_reasons: Vec::new(),
        };
    }

    let inconclusive_reasons = participating_entries
        .iter()
        .copied()
        .filter(|entry| phase8_inconclusive_comparison_source(entry))
        .map(|entry| phase8_status_reason_from_error(entry, "inconclusive"))
        .collect::<Vec<_>>();
    if !inconclusive_reasons.is_empty() {
        return phase8_normalized_comparison_with_reasons(
            Phase8NormalizedComparisonStatus::Inconclusive,
            inconclusive_reasons,
        );
    }

    let Some(baseline) = participating_entries.first().copied() else {
        return Phase8NormalizedComparison {
            status: Phase8NormalizedComparisonStatus::Inconclusive,
            matching_fields: Vec::new(),
            missing_checker_profiles: Vec::new(),
            disagreements: Vec::new(),
            status_reasons: Vec::new(),
        };
    };

    if participating_entries
        .iter()
        .all(|entry| entry.status == Phase8MachineCheckStatus::Checked)
    {
        for entry in participating_entries.iter().skip(1).copied() {
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
    } else if participating_entries
        .iter()
        .all(|entry| entry.status == Phase8MachineCheckStatus::Failed)
    {
        let baseline_hash = baseline
            .failure_key
            .as_ref()
            .map(Phase8NormalizedFailureKey::failure_key_hash);
        for entry in participating_entries.iter().skip(1).copied() {
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
        phase8_push_status_disagreements(&mut disagreements, baseline, &participating_entries);
        for entry in participating_entries.iter().skip(1).copied() {
            if baseline.status != entry.status {
                continue;
            }
            match baseline.status {
                Phase8MachineCheckStatus::Checked => {
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
                Phase8MachineCheckStatus::Failed => {
                    let baseline_hash = baseline
                        .failure_key
                        .as_ref()
                        .map(Phase8NormalizedFailureKey::failure_key_hash);
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
            }
        }
    }

    phase8_sort_normalized_disagreements(&mut disagreements);
    Phase8NormalizedComparison {
        status: Phase8NormalizedComparisonStatus::Disagreement,
        matching_fields: Vec::new(),
        missing_checker_profiles: Vec::new(),
        disagreements,
        status_reasons: Vec::new(),
    }
}

fn phase8_normalized_comparison_with_reasons(
    status: Phase8NormalizedComparisonStatus,
    mut status_reasons: Vec<Phase8NormalizedStatusReason>,
) -> Phase8NormalizedComparison {
    phase8_sort_normalized_status_reasons(&mut status_reasons);
    Phase8NormalizedComparison {
        status,
        matching_fields: Vec::new(),
        missing_checker_profiles: Vec::new(),
        disagreements: Vec::new(),
        status_reasons,
    }
}

fn phase8_policy_has_checker_profile(policy: &Phase8RunnerPolicy, profile: &str) -> bool {
    policy
        .required_checker_profiles
        .iter()
        .chain(policy.optional_checker_profiles.iter())
        .any(|allowed| allowed == profile)
}

fn phase8_normalized_comparison_policy_reasons(
    policy: &Phase8RunnerPolicy,
    normalized_policy_hash: Hash,
    entries: &[Phase8NormalizedCheckResultEntry],
) -> Vec<Phase8NormalizedStatusReason> {
    let expected_policy_hash = policy.policy_hash();
    let mut reasons = Vec::new();
    if normalized_policy_hash != expected_policy_hash {
        reasons.push(phase8_global_hash_status_reason(
            "policy_failure",
            "policy_failure",
            "policy_hash_mismatch",
            "policy.hash",
            expected_policy_hash,
            normalized_policy_hash,
        ));
    }
    for entry in entries {
        if entry.policy_hash != normalized_policy_hash {
            reasons.push(phase8_entry_hash_status_reason(
                entry,
                "policy_failure",
                "policy_failure",
                "result_policy_hash_mismatch",
                "results[].policy_hash",
                normalized_policy_hash,
                entry.policy_hash,
            ));
        }
        if !phase8_policy_has_checker_profile(policy, &entry.checker_profile) {
            reasons.push(phase8_entry_value_status_reason(
                entry,
                "policy_failure",
                "policy_failure",
                "checker_profile_not_allowed",
                "results[].checker_profile",
                "required_or_optional_checker_profile",
                &entry.checker_profile,
            ));
        }
    }
    reasons
}

fn phase8_normalized_comparison_identity_reasons(
    policy: &Phase8RunnerPolicy,
    entries: &[&Phase8NormalizedCheckResultEntry],
) -> Vec<Phase8NormalizedStatusReason> {
    let mut reasons = Vec::new();
    for entry in entries {
        if !entry.process_launched {
            continue;
        }
        let Some(selected) = policy.selected_checker_policy(&entry.checker_profile) else {
            continue;
        };
        if let Some(binary_id) = entry.checker_binary_id.as_deref() {
            if binary_id != selected.binary_id {
                reasons.push(phase8_entry_value_status_reason(
                    entry,
                    "policy_failure",
                    "policy_failure",
                    "checker_binary_id_mismatch",
                    "results[].checker_binary_id",
                    &selected.binary_id,
                    binary_id,
                ));
            }
        }
        if let Some(binary_hash) = entry.checker_binary_hash {
            if binary_hash != selected.binary_hash {
                reasons.push(phase8_entry_hash_status_reason(
                    entry,
                    "policy_failure",
                    "policy_failure",
                    "checker_binary_hash_mismatch",
                    "results[].checker_binary_hash",
                    selected.binary_hash,
                    binary_hash,
                ));
            }
        }
        if let Some(checker_id) = entry.checker_id.as_deref() {
            if checker_id != selected.checker_id {
                reasons.push(phase8_entry_value_status_reason(
                    entry,
                    "policy_failure",
                    "policy_failure",
                    "checker_identity_mismatch",
                    "results[].checker_id",
                    &selected.checker_id,
                    checker_id,
                ));
            }
        }
        if let Some(build_hash) = entry.checker_build_hash {
            if build_hash != selected.build_hash {
                reasons.push(phase8_entry_hash_status_reason(
                    entry,
                    "policy_failure",
                    "policy_failure",
                    "checker_build_hash_mismatch",
                    "results[].checker_build_hash",
                    selected.build_hash,
                    build_hash,
                ));
            }
        }
        if !phase8_checker_identity_missing_exempt(entry) {
            if entry.checker_id.is_none() {
                reasons.push(phase8_entry_value_status_reason(
                    entry,
                    "policy_failure",
                    "policy_failure",
                    "checker_identity_missing",
                    "results[].checker_id",
                    "required_for_launched_non_inconclusive_result",
                    "missing",
                ));
            }
            if entry.checker_build_hash.is_none() {
                reasons.push(phase8_entry_value_status_reason(
                    entry,
                    "policy_failure",
                    "policy_failure",
                    "checker_identity_missing",
                    "results[].checker_build_hash",
                    "required_for_launched_non_inconclusive_result",
                    "missing",
                ));
            }
        }
    }
    reasons
}

fn phase8_checker_identity_missing_exempt(entry: &Phase8NormalizedCheckResultEntry) -> bool {
    matches!(
        entry.error.as_ref().map(|error| error.kind.as_str()),
        Some("checker_internal_error" | "resource_exhausted" | "timeout")
    )
}

fn phase8_malformed_process_state(entry: &Phase8NormalizedCheckResultEntry) -> bool {
    if !entry.process_launched && entry.status == Phase8MachineCheckStatus::Checked {
        return true;
    }
    let error_kind = entry.error.as_ref().map(|error| error.kind.as_str());
    let reason_code = entry
        .error
        .as_ref()
        .and_then(|error| error.reason_code.as_deref());
    if !entry.process_launched
        && !matches!(
            error_kind,
            Some("policy_failure" | "timeout" | "resource_exhausted")
        )
    {
        return true;
    }
    if entry.process_launched
        && matches!(
            reason_code,
            Some("launch_timeout" | "launch_resource_exhausted")
        )
    {
        return true;
    }
    !entry.process_launched
        && matches!(
            reason_code,
            Some("checker_timeout" | "checker_resource_exhausted" | "process_exit_failure")
        )
}

fn phase8_malformed_process_state_reason(
    entry: &Phase8NormalizedCheckResultEntry,
) -> Phase8NormalizedStatusReason {
    phase8_entry_value_status_reason(
        entry,
        "inconclusive",
        "checker_internal_error",
        "malformed_process_state",
        "results[].process_launched",
        "process_state_consistent_with_error_kind",
        "malformed_process_state",
    )
}

fn phase8_inconclusive_comparison_source(entry: &Phase8NormalizedCheckResultEntry) -> bool {
    let Some(error) = entry.error.as_ref() else {
        return false;
    };
    matches!(
        (
            entry.process_launched,
            error.kind.as_str(),
            error.reason_code.as_deref(),
        ),
        (false, "timeout", Some("launch_timeout"))
            | (
                false,
                "resource_exhausted",
                Some("launch_resource_exhausted")
            )
            | (true, "timeout", Some("checker_timeout"))
            | (
                true,
                "resource_exhausted",
                Some("checker_resource_exhausted")
            )
            | (
                true,
                "checker_internal_error",
                Some(
                    "checker_reported_internal_error"
                        | "malformed_success_output"
                        | "success_exit_status_mismatch"
                        | "missing_rejection_error"
                        | "malformed_rejection_output"
                        | "malformed_internal_error_output"
                        | "checker_module_mismatch"
                        | "process_exit_failure",
                ),
            )
    )
}

fn phase8_status_reason_from_error(
    entry: &Phase8NormalizedCheckResultEntry,
    kind: &str,
) -> Phase8NormalizedStatusReason {
    let error = entry
        .error
        .as_ref()
        .expect("status reason copied from an entry error");
    Phase8NormalizedStatusReason {
        kind: kind.to_owned(),
        error_kind: error.kind.clone(),
        reason_code: error
            .reason_code
            .clone()
            .unwrap_or_else(|| error.kind.clone()),
        checker_profile: Some(entry.checker_profile.clone()),
        result_hash: Some(entry.result_hash),
        field: error.field.clone(),
        expected_hash: error.expected_hash,
        actual_hash: error.actual_hash,
        expected_value: error.expected_value.clone(),
        actual_value: error.actual_value.clone(),
    }
}

fn phase8_global_hash_status_reason(
    kind: &str,
    error_kind: &str,
    reason_code: &str,
    field: &str,
    expected_hash: Hash,
    actual_hash: Hash,
) -> Phase8NormalizedStatusReason {
    Phase8NormalizedStatusReason {
        kind: kind.to_owned(),
        error_kind: error_kind.to_owned(),
        reason_code: reason_code.to_owned(),
        checker_profile: None,
        result_hash: None,
        field: Some(field.to_owned()),
        expected_hash: Some(expected_hash),
        actual_hash: Some(actual_hash),
        expected_value: None,
        actual_value: None,
    }
}

fn phase8_entry_hash_status_reason(
    entry: &Phase8NormalizedCheckResultEntry,
    kind: &str,
    error_kind: &str,
    reason_code: &str,
    field: &str,
    expected_hash: Hash,
    actual_hash: Hash,
) -> Phase8NormalizedStatusReason {
    Phase8NormalizedStatusReason {
        kind: kind.to_owned(),
        error_kind: error_kind.to_owned(),
        reason_code: reason_code.to_owned(),
        checker_profile: Some(entry.checker_profile.clone()),
        result_hash: Some(entry.result_hash),
        field: Some(field.to_owned()),
        expected_hash: Some(expected_hash),
        actual_hash: Some(actual_hash),
        expected_value: None,
        actual_value: None,
    }
}

fn phase8_entry_value_status_reason(
    entry: &Phase8NormalizedCheckResultEntry,
    kind: &str,
    error_kind: &str,
    reason_code: &str,
    field: &str,
    expected_value: impl Into<String>,
    actual_value: impl Into<String>,
) -> Phase8NormalizedStatusReason {
    Phase8NormalizedStatusReason {
        kind: kind.to_owned(),
        error_kind: error_kind.to_owned(),
        reason_code: reason_code.to_owned(),
        checker_profile: Some(entry.checker_profile.clone()),
        result_hash: Some(entry.result_hash),
        field: Some(field.to_owned()),
        expected_hash: None,
        actual_hash: None,
        expected_value: Some(expected_value.into()),
        actual_value: Some(actual_value.into()),
    }
}

fn phase8_push_status_disagreements(
    disagreements: &mut Vec<Phase8NormalizedDisagreement>,
    baseline: &Phase8NormalizedCheckResultEntry,
    entries: &[&Phase8NormalizedCheckResultEntry],
) {
    for entry in entries.iter().skip(1).copied() {
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

fn phase8_sort_normalized_disagreements(disagreements: &mut [Phase8NormalizedDisagreement]) {
    disagreements.sort_by(|left, right| {
        left.field
            .cmp(&right.field)
            .then_with(|| left.checker_profile.cmp(&right.checker_profile))
            .then_with(|| {
                left.baseline_checker_profile
                    .cmp(&right.baseline_checker_profile)
            })
            .then_with(|| {
                left.baseline_hash
                    .map(|hash| format_hash_string(&hash))
                    .cmp(&right.baseline_hash.map(|hash| format_hash_string(&hash)))
            })
            .then_with(|| {
                left.actual_hash
                    .map(|hash| format_hash_string(&hash))
                    .cmp(&right.actual_hash.map(|hash| format_hash_string(&hash)))
            })
            .then_with(|| left.baseline_value.cmp(&right.baseline_value))
            .then_with(|| left.actual_value.cmp(&right.actual_value))
    });
}

fn phase8_sort_normalized_status_reasons(reasons: &mut [Phase8NormalizedStatusReason]) {
    reasons.sort_by(|left, right| {
        phase8_status_reason_sort_key(left).cmp(&phase8_status_reason_sort_key(right))
    });
}

fn phase8_status_reason_sort_key(reason: &Phase8NormalizedStatusReason) -> Vec<String> {
    vec![
        reason.kind.clone(),
        reason.checker_profile.clone().unwrap_or_default(),
        reason.field.clone().unwrap_or_default(),
        reason.reason_code.clone(),
        reason
            .result_hash
            .map(|hash| format_hash_string(&hash))
            .unwrap_or_default(),
        reason.error_kind.clone(),
        phase8_status_reason_payload_rank(reason).to_string(),
        reason
            .expected_hash
            .map(|hash| format_hash_string(&hash))
            .unwrap_or_default(),
        reason
            .actual_hash
            .map(|hash| format_hash_string(&hash))
            .unwrap_or_default(),
        reason
            .expected_value
            .as_deref()
            .map(phase8_json_string_literal)
            .unwrap_or_default(),
        reason
            .actual_value
            .as_deref()
            .map(phase8_json_string_literal)
            .unwrap_or_default(),
    ]
}

fn phase8_status_reason_payload_rank(reason: &Phase8NormalizedStatusReason) -> u8 {
    if reason.expected_hash.is_some() || reason.actual_hash.is_some() {
        1
    } else if reason.expected_value.is_some() {
        2
    } else if reason.actual_value.is_some() {
        3
    } else {
        0
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

enum Phase8ResolvedAiAuditSource<'a> {
    MachineResult {
        machine_result: &'a Phase8MachineCheckResult,
        normalized_result: Option<&'a Phase8NormalizedCheckResult>,
    },
    NormalizedComparison {
        normalized_result: &'a Phase8NormalizedCheckResult,
    },
}

fn phase8_parse_ai_audit_input_policy_value(
    value: &JsonValue<'_>,
) -> Result<Phase8AiAuditInputPolicy, Phase8AuditSidecarValidationError> {
    let members = phase8_ai_object_members(
        value,
        "input_policy",
        "object",
        Phase8AuditSidecarValidationReasonCode::InputPolicySchemaInvalid,
    )?;
    phase8_ai_required_fixed_string(
        members,
        "schema",
        "input_policy.schema",
        PHASE8_AI_AUDIT_INPUT_POLICY_SCHEMA,
        Phase8AuditSidecarValidationReasonCode::InputPolicySchemaInvalid,
    )?;
    let id = phase8_ai_required_string(
        members,
        "id",
        "input_policy.id",
        "phase8_policy_id",
        Phase8AuditSidecarValidationReasonCode::InputPolicySchemaInvalid,
    )?;
    if id.is_empty() {
        return Err(Phase8AuditSidecarValidationError::value(
            Phase8AuditSidecarValidationReasonCode::InputPolicySchemaInvalid,
            "input_policy.id",
            "phase8_policy_id",
            "empty_string",
        ));
    }
    if !phase8_valid_runner_policy_id(&id) {
        return Err(Phase8AuditSidecarValidationError::value(
            Phase8AuditSidecarValidationReasonCode::InputPolicySchemaInvalid,
            "input_policy.id",
            "phase8_policy_id",
            "invalid_name_format",
        ));
    }
    let version = phase8_ai_required_positive_i64(
        members,
        "version",
        "input_policy.version",
        Phase8AuditSidecarValidationReasonCode::InputPolicySchemaInvalid,
    )?;
    let included_fields = phase8_ai_required_included_fields(
        members,
        "included_fields",
        "input_policy.included_fields",
        Phase8AuditSidecarValidationReasonCode::InputPolicySchemaInvalid,
    )?;
    let redaction = phase8_ai_required_string(
        members,
        "redaction",
        "input_policy.redaction",
        "AiAuditInputPolicy.redaction",
        Phase8AuditSidecarValidationReasonCode::InputPolicySchemaInvalid,
    )?;
    if !phase8_ai_audit_redaction_allowed(&redaction) {
        return Err(Phase8AuditSidecarValidationError::value(
            Phase8AuditSidecarValidationReasonCode::InputPolicySchemaInvalid,
            "input_policy.redaction",
            "AiAuditInputPolicy.redaction",
            "invalid_enum",
        ));
    }
    let allow_source_text = phase8_ai_required_bool(
        members,
        "allow_source_text",
        "input_policy.allow_source_text",
        Phase8AuditSidecarValidationReasonCode::InputPolicySchemaInvalid,
    )?;
    let allow_tactic_trace = phase8_ai_required_bool(
        members,
        "allow_tactic_trace",
        "input_policy.allow_tactic_trace",
        Phase8AuditSidecarValidationReasonCode::InputPolicySchemaInvalid,
    )?;
    phase8_ai_reject_unknown_fields(
        members,
        &[
            "schema",
            "id",
            "version",
            "included_fields",
            "redaction",
            "allow_source_text",
            "allow_tactic_trace",
        ],
        "input_policy",
        Phase8AuditSidecarValidationReasonCode::InputPolicySchemaInvalid,
    )?;
    Ok(Phase8AiAuditInputPolicy {
        id,
        version,
        included_fields,
        redaction,
        allow_source_text,
        allow_tactic_trace,
    })
}

fn phase8_parse_ai_audit_sidecar_value(
    value: &JsonValue<'_>,
) -> Result<Phase8AiAuditSidecar, Phase8AuditSidecarValidationError> {
    let members = phase8_ai_object_members(
        value,
        "$",
        "object",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;
    phase8_ai_required_fixed_string(
        members,
        "schema",
        "schema",
        PHASE8_AI_AUDIT_SIDECAR_SCHEMA,
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;
    if let Some(field) = phase8_find_static_forbidden_sidecar_field(value, "$") {
        return Err(Phase8AuditSidecarValidationError::actual_value(
            Phase8AuditSidecarValidationReasonCode::ForbiddenSidecarField,
            field,
            "present",
        ));
    }

    let source_value = phase8_ai_required_value(
        members,
        "source",
        "source",
        "object",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;
    let source = phase8_parse_ai_audit_sidecar_source(source_value)?;
    let input_policy_value = phase8_ai_required_value(
        members,
        "input_policy",
        "input_policy",
        "object",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;
    let input_policy = phase8_parse_ai_audit_sidecar_input_policy(input_policy_value)?;
    if source.kind == Phase8AiAuditSidecarSourceKind::MachineResult
        && source.normalized_result_hash.is_none()
        && input_policy.included_fields.iter().any(|field| {
            matches!(
                field.as_str(),
                "input_file_hash" | "expected_certificate_hash"
            )
        })
    {
        return Err(Phase8AuditSidecarValidationError::value(
            Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
            "source.normalized_result_hash",
            "required_for_input_field",
            "missing",
        ));
    }
    let ai_value = phase8_ai_required_value(
        members,
        "ai",
        "ai",
        "object",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;
    let ai = phase8_parse_ai_audit_sidecar_ai(ai_value)?;
    let status = phase8_parse_ai_audit_sidecar_status(
        &phase8_ai_required_string(
            members,
            "status",
            "status",
            "AiAuditSidecar.status",
            Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
        )?,
        "status",
    )?;

    let classification_value = phase8_ai_optional_value(
        members,
        "classification",
        "classification",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;
    if source.kind == Phase8AiAuditSidecarSourceKind::NormalizedComparison
        && classification_value
            .and_then(JsonValue::object_members)
            .is_some_and(|members| {
                members
                    .iter()
                    .any(|member| member.key() == "checker_error_kind")
            })
    {
        return Err(Phase8AuditSidecarValidationError::actual_value(
            Phase8AuditSidecarValidationReasonCode::ForbiddenSidecarField,
            "classification.checker_error_kind",
            "present",
        ));
    }
    let classification = match classification_value {
        Some(value) => Some(phase8_parse_ai_audit_sidecar_classification(value)?),
        None if status.requires_classification() => {
            return Err(Phase8AuditSidecarValidationError::value(
                Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
                "classification",
                format!("required_for_status:{}", status.as_str()),
                "missing",
            ))
        }
        None => None,
    };

    let summary = phase8_ai_required_text_string(members, "summary", "summary")?;
    let suggested_next_actions = phase8_parse_ai_audit_suggested_next_actions(members, status)?;
    let policy_gated_fields = phase8_parse_ai_audit_policy_gated_fields(members)?;

    let mut allowed = vec![
        "schema",
        "source",
        "input_policy",
        "ai",
        "status",
        "classification",
        "summary",
        "suggested_next_actions",
    ];
    allowed.extend(PHASE8_AI_AUDIT_POLICY_GATED_FIELDS.iter().copied());
    phase8_ai_reject_unknown_fields(
        members,
        &allowed,
        "$",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;

    Ok(Phase8AiAuditSidecar {
        source,
        input_policy,
        ai,
        status,
        classification,
        summary,
        suggested_next_actions,
        policy_gated_fields,
    })
}

fn phase8_parse_ai_audit_sidecar_source(
    value: &JsonValue<'_>,
) -> Result<Phase8AiAuditSidecarSource, Phase8AuditSidecarValidationError> {
    let members = phase8_ai_object_members(
        value,
        "source",
        "object",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;
    let kind_raw = phase8_ai_required_string(
        members,
        "kind",
        "source.kind",
        "AiAuditSidecar.source.kind",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;
    let kind = match kind_raw.as_str() {
        "machine_result" => Phase8AiAuditSidecarSourceKind::MachineResult,
        "normalized_comparison" => Phase8AiAuditSidecarSourceKind::NormalizedComparison,
        _ => {
            return Err(Phase8AuditSidecarValidationError::value(
                Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
                "source.kind",
                "AiAuditSidecar.source.kind",
                "invalid_enum",
            ))
        }
    };
    let result_hash = match kind {
        Phase8AiAuditSidecarSourceKind::MachineResult => Some(phase8_ai_required_hash(
            members,
            "result_hash",
            "source.result_hash",
            Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
        )?),
        Phase8AiAuditSidecarSourceKind::NormalizedComparison => None,
    };
    let request_hash = match kind {
        Phase8AiAuditSidecarSourceKind::MachineResult => Some(phase8_ai_required_hash(
            members,
            "request_hash",
            "source.request_hash",
            Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
        )?),
        Phase8AiAuditSidecarSourceKind::NormalizedComparison => None,
    };
    let run_artifact_hash = match kind {
        Phase8AiAuditSidecarSourceKind::MachineResult => Some(phase8_ai_required_hash(
            members,
            "run_artifact_hash",
            "source.run_artifact_hash",
            Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
        )?),
        Phase8AiAuditSidecarSourceKind::NormalizedComparison => None,
    };
    let normalized_result_hash = match kind {
        Phase8AiAuditSidecarSourceKind::MachineResult => phase8_ai_optional_hash(
            members,
            "normalized_result_hash",
            "source.normalized_result_hash",
            Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
        )?,
        Phase8AiAuditSidecarSourceKind::NormalizedComparison => Some(phase8_ai_required_hash(
            members,
            "normalized_result_hash",
            "source.normalized_result_hash",
            Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
        )?),
    };
    let result_id = match kind {
        Phase8AiAuditSidecarSourceKind::MachineResult => {
            phase8_ai_optional_visible_ascii(members, "result_id", "source.result_id", "result_id")?
        }
        Phase8AiAuditSidecarSourceKind::NormalizedComparison => None,
    };
    let normalized_result_id = phase8_ai_optional_visible_ascii(
        members,
        "normalized_result_id",
        "source.normalized_result_id",
        "normalized_result_id",
    )?;

    if kind == Phase8AiAuditSidecarSourceKind::NormalizedComparison {
        for field in [
            "result_hash",
            "request_hash",
            "run_artifact_hash",
            "result_id",
        ] {
            if members.iter().any(|member| member.key() == field) {
                return Err(Phase8AuditSidecarValidationError::actual_value(
                    Phase8AuditSidecarValidationReasonCode::ForbiddenSidecarField,
                    format!("source.{field}"),
                    "present",
                ));
            }
        }
    }
    if kind == Phase8AiAuditSidecarSourceKind::MachineResult
        && normalized_result_id.is_some()
        && normalized_result_hash.is_none()
    {
        return Err(Phase8AuditSidecarValidationError::value(
            Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
            "source.normalized_result_hash",
            "required_with_source.normalized_result_id",
            "missing",
        ));
    }

    phase8_ai_reject_unknown_fields(
        members,
        &[
            "kind",
            "result_hash",
            "request_hash",
            "run_artifact_hash",
            "normalized_result_hash",
            "result_id",
            "normalized_result_id",
        ],
        "source",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;

    Ok(Phase8AiAuditSidecarSource {
        kind,
        result_hash,
        request_hash,
        run_artifact_hash,
        normalized_result_hash,
        result_id,
        normalized_result_id,
    })
}

fn phase8_parse_ai_audit_sidecar_input_policy(
    value: &JsonValue<'_>,
) -> Result<Phase8AiAuditSidecarInputPolicy, Phase8AuditSidecarValidationError> {
    let members = phase8_ai_object_members(
        value,
        "input_policy",
        "object",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;
    let id = phase8_ai_required_string(
        members,
        "id",
        "input_policy.id",
        "phase8_policy_id",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;
    if id.is_empty() || !phase8_valid_runner_policy_id(&id) {
        return Err(Phase8AuditSidecarValidationError::value(
            Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
            "input_policy.id",
            "phase8_policy_id",
            if id.is_empty() {
                "empty_string"
            } else {
                "invalid_name_format"
            },
        ));
    }
    let version = phase8_ai_required_positive_i64(
        members,
        "version",
        "input_policy.version",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;
    let hash = phase8_ai_required_hash(
        members,
        "hash",
        "input_policy.hash",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;
    let included_fields = phase8_ai_required_included_fields(
        members,
        "included_fields",
        "input_policy.included_fields",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;
    let redaction = phase8_ai_required_string(
        members,
        "redaction",
        "input_policy.redaction",
        "AiAuditInputPolicy.redaction",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;
    if !phase8_ai_audit_redaction_allowed(&redaction) {
        return Err(Phase8AuditSidecarValidationError::value(
            Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
            "input_policy.redaction",
            "AiAuditInputPolicy.redaction",
            "invalid_enum",
        ));
    }
    phase8_ai_reject_unknown_fields(
        members,
        &["id", "version", "hash", "included_fields", "redaction"],
        "input_policy",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;
    Ok(Phase8AiAuditSidecarInputPolicy {
        id,
        version,
        hash,
        included_fields,
        redaction,
    })
}

fn phase8_parse_ai_audit_sidecar_ai(
    value: &JsonValue<'_>,
) -> Result<Phase8AiAuditSidecarAi, Phase8AuditSidecarValidationError> {
    let members = phase8_ai_object_members(
        value,
        "ai",
        "object",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;
    let agent = phase8_ai_required_visible_ascii(
        members,
        "agent",
        "ai.agent",
        "non_empty_visible_ascii_string",
    )?;
    let model = phase8_ai_required_visible_ascii(
        members,
        "model",
        "ai.model",
        "non_empty_visible_ascii_string",
    )?;
    let prompt_hash = phase8_ai_required_hash(
        members,
        "prompt_hash",
        "ai.prompt_hash",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;
    phase8_ai_reject_unknown_fields(
        members,
        &["agent", "model", "prompt_hash"],
        "ai",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;
    Ok(Phase8AiAuditSidecarAi {
        agent,
        model,
        prompt_hash,
    })
}

fn phase8_parse_ai_audit_sidecar_classification(
    value: &JsonValue<'_>,
) -> Result<Phase8AiAuditSidecarClassification, Phase8AuditSidecarValidationError> {
    let members = phase8_ai_object_members(
        value,
        "classification",
        "object",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;
    let category = phase8_ai_required_string(
        members,
        "category",
        "classification.category",
        "AiAuditSidecar.classification.category",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;
    if !phase8_ai_audit_classification_category_allowed(&category) {
        return Err(Phase8AuditSidecarValidationError::value(
            Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
            "classification.category",
            "AiAuditSidecar.classification.category",
            "invalid_enum",
        ));
    }
    let confidence = phase8_ai_required_string(
        members,
        "confidence",
        "classification.confidence",
        "AiAuditSidecar.classification.confidence",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;
    if !matches!(confidence.as_str(), "low" | "medium" | "high" | "unknown") {
        return Err(Phase8AuditSidecarValidationError::value(
            Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
            "classification.confidence",
            "AiAuditSidecar.classification.confidence",
            "invalid_enum",
        ));
    }
    let checker_error_kind = phase8_ai_optional_string(
        members,
        "checker_error_kind",
        "classification.checker_error_kind",
        "MachineCheckResult.error.kind",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;
    if let Some(kind) = checker_error_kind.as_deref() {
        if !phase8_raw_checker_error_kind_allowed(kind)
            && !matches!(
                kind,
                "policy_failure" | "timeout" | "resource_exhausted" | "checker_internal_error"
            )
        {
            return Err(Phase8AuditSidecarValidationError::value(
                Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
                "classification.checker_error_kind",
                "MachineCheckResult.error.kind",
                "invalid_enum",
            ));
        }
    }
    phase8_ai_reject_unknown_fields(
        members,
        &["category", "confidence", "checker_error_kind"],
        "classification",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;
    Ok(Phase8AiAuditSidecarClassification {
        category,
        confidence,
        checker_error_kind,
    })
}

fn phase8_parse_ai_audit_suggested_next_actions(
    members: &[JsonMember<'_>],
    status: Phase8AiAuditSidecarStatus,
) -> Result<Option<Vec<String>>, Phase8AuditSidecarValidationError> {
    let Some(value) = phase8_ai_optional_value(
        members,
        "suggested_next_actions",
        "suggested_next_actions",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?
    else {
        if status.requires_next_actions() {
            return Err(Phase8AuditSidecarValidationError::value(
                Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
                "suggested_next_actions",
                format!("non_empty_array_for_status:{}", status.as_str()),
                "missing",
            ));
        }
        return Ok(None);
    };
    let Some(elements) = value.array_elements() else {
        return Err(Phase8AuditSidecarValidationError::value(
            Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
            "suggested_next_actions",
            if status.requires_next_actions() {
                format!("non_empty_array_for_status:{}", status.as_str())
            } else {
                "array".to_owned()
            },
            if value.kind() == JsonValueKind::Null {
                "null_not_allowed"
            } else {
                "wrong_type"
            },
        ));
    };
    if elements.is_empty() && status.requires_next_actions() {
        return Err(Phase8AuditSidecarValidationError::value(
            Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
            "suggested_next_actions",
            format!("non_empty_array_for_status:{}", status.as_str()),
            "empty_array",
        ));
    }
    let mut actions = Vec::new();
    for (index, element) in elements.iter().enumerate() {
        let path = format!("suggested_next_actions[{index}]");
        let Some(action) = element.string_value() else {
            return Err(Phase8AuditSidecarValidationError::value(
                Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
                path,
                "non_empty_text_string",
                if element.kind() == JsonValueKind::Null {
                    "null_not_allowed"
                } else {
                    "wrong_type"
                },
            ));
        };
        phase8_validate_ai_text(action, &path)?;
        actions.push(action.to_owned());
    }
    Ok(Some(actions))
}

fn phase8_parse_ai_audit_policy_gated_fields(
    members: &[JsonMember<'_>],
) -> Result<BTreeMap<String, Phase8AiAuditPolicyGatedFieldValue>, Phase8AuditSidecarValidationError>
{
    let mut fields = BTreeMap::new();
    for member in members {
        if !phase8_ai_policy_gated_field_name(member.key()) {
            continue;
        }
        let value = member.value();
        if let Some(text) = value.string_value() {
            fields.insert(
                member.key().to_owned(),
                Phase8AiAuditPolicyGatedFieldValue::String(text.to_owned()),
            );
            continue;
        }
        let Some(elements) = value.array_elements() else {
            return Err(Phase8AuditSidecarValidationError::value(
                Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
                member.key(),
                "string_or_string_array",
                if value.kind() == JsonValueKind::Null {
                    "null_not_allowed"
                } else {
                    "wrong_type"
                },
            ));
        };
        let mut strings = Vec::new();
        for (index, element) in elements.iter().enumerate() {
            let Some(text) = element.string_value() else {
                return Err(Phase8AuditSidecarValidationError::value(
                    Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
                    format!("{}[{index}]", member.key()),
                    "string",
                    if element.kind() == JsonValueKind::Null {
                        "null_not_allowed"
                    } else {
                        "wrong_type"
                    },
                ));
            };
            strings.push(text.to_owned());
        }
        fields.insert(
            member.key().to_owned(),
            Phase8AiAuditPolicyGatedFieldValue::Strings(strings),
        );
    }
    Ok(fields)
}

fn phase8_ai_audit_copied_policy_mismatch(
    sidecar: &Phase8AiAuditSidecar,
    policy: &Phase8AiAuditInputPolicy,
) -> Option<Phase8AuditSidecarValidationError> {
    if sidecar.input_policy.id != policy.id {
        return Some(Phase8AuditSidecarValidationError::value(
            Phase8AuditSidecarValidationReasonCode::InputPolicyFieldMismatch,
            "input_policy.id",
            &policy.id,
            &sidecar.input_policy.id,
        ));
    }
    if sidecar.input_policy.version != policy.version {
        return Some(Phase8AuditSidecarValidationError::value(
            Phase8AuditSidecarValidationReasonCode::InputPolicyFieldMismatch,
            "input_policy.version",
            policy.version.to_string(),
            sidecar.input_policy.version.to_string(),
        ));
    }
    if sidecar.input_policy.included_fields != policy.included_fields {
        return Some(Phase8AuditSidecarValidationError::value(
            Phase8AuditSidecarValidationReasonCode::InputPolicyFieldMismatch,
            "input_policy.included_fields",
            phase8_json_string_array(&policy.included_fields),
            phase8_json_string_array(&sidecar.input_policy.included_fields),
        ));
    }
    if sidecar.input_policy.redaction != policy.redaction {
        return Some(Phase8AuditSidecarValidationError::value(
            Phase8AuditSidecarValidationReasonCode::InputPolicyFieldMismatch,
            "input_policy.redaction",
            &policy.redaction,
            &sidecar.input_policy.redaction,
        ));
    }
    None
}

fn phase8_ai_audit_policy_gated_field_error(
    sidecar: &Phase8AiAuditSidecar,
    policy: &Phase8AiAuditInputPolicy,
) -> Option<Phase8AuditSidecarValidationError> {
    for field in [
        "source_text",
        "source_excerpt",
        "theorem_statement",
        "proof_script",
    ] {
        if !policy.allow_source_text && sidecar.policy_gated_fields.contains_key(field) {
            return Some(Phase8AuditSidecarValidationError::actual_value(
                Phase8AuditSidecarValidationReasonCode::ForbiddenSidecarField,
                field,
                "present",
            ));
        }
    }
    for field in [
        "tactic_trace",
        "tactic_script",
        "elaboration_trace",
        "ai_search_trace",
    ] {
        if !policy.allow_tactic_trace && sidecar.policy_gated_fields.contains_key(field) {
            return Some(Phase8AuditSidecarValidationError::actual_value(
                Phase8AuditSidecarValidationReasonCode::ForbiddenSidecarField,
                field,
                "present",
            ));
        }
    }
    None
}

fn phase8_resolve_ai_audit_sidecar_source<'a>(
    sidecar: &Phase8AiAuditSidecar,
    machine_results: &'a [Phase8MachineCheckResult],
    normalized_results: &'a [Phase8NormalizedCheckResult],
) -> Result<Phase8ResolvedAiAuditSource<'a>, Phase8AuditSidecarValidationError> {
    match sidecar.source.kind {
        Phase8AiAuditSidecarSourceKind::MachineResult => {
            let run_artifact_hash = sidecar
                .source
                .run_artifact_hash
                .expect("machine result source run hash is schema-valid");
            let Some(machine_result) = machine_results
                .iter()
                .find(|result| result.run_artifact_hash() == run_artifact_hash)
            else {
                return Err(Phase8AuditSidecarValidationError::expected_hash(
                    Phase8AuditSidecarValidationReasonCode::SourceResultNotFound,
                    "source.run_artifact_hash",
                    run_artifact_hash,
                ));
            };
            let source_result_hash = sidecar
                .source
                .result_hash
                .expect("machine result source result hash is schema-valid");
            if source_result_hash != machine_result.result_hash() {
                return Err(Phase8AuditSidecarValidationError::hash(
                    Phase8AuditSidecarValidationReasonCode::SourceHashMismatch,
                    "source.result_hash",
                    source_result_hash,
                    machine_result.result_hash(),
                ));
            }
            let source_request_hash = sidecar
                .source
                .request_hash
                .expect("machine result source request hash is schema-valid");
            if source_request_hash != machine_result.request_hash {
                return Err(Phase8AuditSidecarValidationError::hash(
                    Phase8AuditSidecarValidationReasonCode::SourceHashMismatch,
                    "source.request_hash",
                    source_request_hash,
                    machine_result.request_hash,
                ));
            }
            if let Some(result_id) = &sidecar.source.result_id {
                if result_id != &machine_result.result_id {
                    return Err(Phase8AuditSidecarValidationError::value(
                        Phase8AuditSidecarValidationReasonCode::SourceIdMismatch,
                        "source.result_id",
                        result_id,
                        &machine_result.result_id,
                    ));
                }
            }
            let normalized_result =
                if let Some(normalized_hash) = sidecar.source.normalized_result_hash {
                    let Some(normalized) = normalized_results
                        .iter()
                        .find(|result| result.normalized_result_hash() == normalized_hash)
                    else {
                        return Err(Phase8AuditSidecarValidationError::expected_hash(
                            Phase8AuditSidecarValidationReasonCode::SourceNormalizedResultNotFound,
                            "source.normalized_result_hash",
                            normalized_hash,
                        ));
                    };
                    if !normalized
                        .results
                        .iter()
                        .any(|entry| entry.result_hash == source_result_hash)
                    {
                        return Err(Phase8AuditSidecarValidationError::expected_hash(
                            Phase8AuditSidecarValidationReasonCode::NormalizedResultMissingSource,
                            "normalized_result.results[].result_hash",
                            source_result_hash,
                        ));
                    }
                    if let Some(normalized_id) = &sidecar.source.normalized_result_id {
                        if normalized_id != &normalized.normalized_result_id {
                            return Err(Phase8AuditSidecarValidationError::value(
                                Phase8AuditSidecarValidationReasonCode::SourceIdMismatch,
                                "source.normalized_result_id",
                                normalized_id,
                                &normalized.normalized_result_id,
                            ));
                        }
                    }
                    Some(normalized)
                } else {
                    None
                };
            Ok(Phase8ResolvedAiAuditSource::MachineResult {
                machine_result,
                normalized_result,
            })
        }
        Phase8AiAuditSidecarSourceKind::NormalizedComparison => {
            let normalized_hash = sidecar
                .source
                .normalized_result_hash
                .expect("normalized comparison source hash is schema-valid");
            let Some(normalized_result) = normalized_results
                .iter()
                .find(|result| result.normalized_result_hash() == normalized_hash)
            else {
                return Err(Phase8AuditSidecarValidationError::expected_hash(
                    Phase8AuditSidecarValidationReasonCode::SourceNormalizedResultNotFound,
                    "source.normalized_result_hash",
                    normalized_hash,
                ));
            };
            if let Some(normalized_id) = &sidecar.source.normalized_result_id {
                if normalized_id != &normalized_result.normalized_result_id {
                    return Err(Phase8AuditSidecarValidationError::value(
                        Phase8AuditSidecarValidationReasonCode::SourceIdMismatch,
                        "source.normalized_result_id",
                        normalized_id,
                        &normalized_result.normalized_result_id,
                    ));
                }
            }
            Ok(Phase8ResolvedAiAuditSource::NormalizedComparison { normalized_result })
        }
    }
}

fn phase8_validate_ai_audit_source_dependent_fields(
    sidecar: &Phase8AiAuditSidecar,
    source: &Phase8ResolvedAiAuditSource<'_>,
) -> Option<Phase8AuditSidecarValidationError> {
    match source {
        Phase8ResolvedAiAuditSource::MachineResult { machine_result, .. } => {
            if machine_result.status == Phase8MachineCheckStatus::Failed {
                if let Some(classification) = &sidecar.classification {
                    let expected = machine_result
                        .error
                        .as_ref()
                        .map(|error| error.kind.as_str())
                        .unwrap_or("missing");
                    match classification.checker_error_kind.as_deref() {
                        Some(actual) if actual == expected => {}
                        Some(actual) => return Some(Phase8AuditSidecarValidationError::value(
                            Phase8AuditSidecarValidationReasonCode::ReferencedArtifactValueMismatch,
                            "classification.checker_error_kind",
                            expected,
                            actual,
                        )),
                        None => return Some(Phase8AuditSidecarValidationError::value(
                            Phase8AuditSidecarValidationReasonCode::ReferencedArtifactValueMismatch,
                            "classification.checker_error_kind",
                            expected,
                            "missing",
                        )),
                    }
                }
            } else if let Some(actual) = sidecar
                .classification
                .as_ref()
                .and_then(|classification| classification.checker_error_kind.as_ref())
            {
                return Some(Phase8AuditSidecarValidationError::value(
                    Phase8AuditSidecarValidationReasonCode::ReferencedArtifactValueMismatch,
                    "classification.checker_error_kind",
                    "absent",
                    actual,
                ));
            }
            if machine_result.status == Phase8MachineCheckStatus::Checked
                && sidecar.status != Phase8AiAuditSidecarStatus::Summarized
            {
                return Some(Phase8AuditSidecarValidationError::value(
                    Phase8AuditSidecarValidationReasonCode::ReferencedArtifactValueMismatch,
                    "status",
                    "sidecar_status:summarized_only",
                    sidecar.status.as_str(),
                ));
            }
        }
        Phase8ResolvedAiAuditSource::NormalizedComparison { normalized_result } => {
            if matches!(
                normalized_result.comparison.status,
                Phase8NormalizedComparisonStatus::AllAgreeChecked
                    | Phase8NormalizedComparisonStatus::AllAgreeFailed
            ) && sidecar.status != Phase8AiAuditSidecarStatus::Summarized
            {
                return Some(Phase8AuditSidecarValidationError::value(
                    Phase8AuditSidecarValidationReasonCode::ReferencedArtifactValueMismatch,
                    "status",
                    "sidecar_status:summarized_only",
                    sidecar.status.as_str(),
                ));
            }
        }
    }
    None
}

fn phase8_ai_audit_prompt_input(
    sidecar: &Phase8AiAuditSidecar,
    input_policy: &Phase8AiAuditInputPolicy,
    source: &Phase8ResolvedAiAuditSource<'_>,
) -> Phase8AiAuditPromptInput {
    let source_hash = match sidecar.source.kind {
        Phase8AiAuditSidecarSourceKind::MachineResult => sidecar
            .source
            .result_hash
            .expect("machine result source hash is schema-valid"),
        Phase8AiAuditSidecarSourceKind::NormalizedComparison => sidecar
            .source
            .normalized_result_hash
            .expect("normalized comparison source hash is schema-valid"),
    };
    let mut fields = Vec::new();
    for field in &input_policy.included_fields {
        if let Some(value) = phase8_ai_audit_resolve_prompt_field(field, source) {
            fields.push((field.clone(), value));
        }
    }
    fields.sort_by(|left, right| phase8_rfc8785_object_key_cmp(&left.0, &right.0));
    Phase8AiAuditPromptInput {
        agent: sidecar.ai.agent.clone(),
        model: sidecar.ai.model.clone(),
        source_kind: sidecar.source.kind,
        source_hash,
        source_run_artifact_hash: (sidecar.source.kind
            == Phase8AiAuditSidecarSourceKind::MachineResult)
            .then_some(
                sidecar
                    .source
                    .run_artifact_hash
                    .expect("machine result source run hash is schema-valid"),
            ),
        source_membership_hash: (sidecar.source.kind
            == Phase8AiAuditSidecarSourceKind::MachineResult)
            .then_some(sidecar.source.normalized_result_hash)
            .flatten(),
        input_policy: Phase8AiAuditSidecarInputPolicy::from_policy(input_policy),
        fields,
    }
}

fn phase8_ai_audit_resolve_prompt_field(
    field: &str,
    source: &Phase8ResolvedAiAuditSource<'_>,
) -> Option<String> {
    match source {
        Phase8ResolvedAiAuditSource::MachineResult {
            machine_result,
            normalized_result,
        } => phase8_ai_audit_machine_result_field(field, machine_result, *normalized_result),
        Phase8ResolvedAiAuditSource::NormalizedComparison { normalized_result } => {
            phase8_ai_audit_normalized_comparison_field(field, normalized_result)
        }
    }
}

fn phase8_ai_audit_machine_result_field(
    field: &str,
    result: &Phase8MachineCheckResult,
    normalized_result: Option<&Phase8NormalizedCheckResult>,
) -> Option<String> {
    match field {
        "module" => Some(phase8_json_string_literal(&result.module)),
        "status" => Some(phase8_json_string_literal(result.status.as_str())),
        "certificate_hash" => result
            .certificate_hash
            .map(|hash| phase8_hash_json_literal(&hash)),
        "checker_binary_hash" => result
            .checker
            .binary_hash
            .map(|hash| phase8_hash_json_literal(&hash)),
        "checker_binary_id" => result
            .checker
            .binary_id
            .as_deref()
            .map(phase8_json_string_literal),
        "checker_build_hash" => result
            .checker
            .build_hash
            .map(|hash| phase8_hash_json_literal(&hash)),
        "checker_id" => result.checker.id.as_deref().map(phase8_json_string_literal),
        "checker_profile" => Some(phase8_json_string_literal(&result.checker.profile)),
        "checker_version" => result
            .checker
            .version
            .as_deref()
            .map(phase8_json_string_literal),
        "error.actual_hash" => result
            .error
            .as_ref()
            .and_then(|error| error.actual_hash)
            .map(|hash| phase8_hash_json_literal(&hash)),
        "error.core_path" => result
            .error
            .as_ref()
            .and_then(|error| error.core_path.as_ref())
            .map(|path| phase8_json_string_array(path)),
        "error.declaration" => result
            .error
            .as_ref()
            .and_then(|error| error.declaration.as_deref())
            .map(phase8_json_string_literal),
        "error.expected_hash" => result
            .error
            .as_ref()
            .and_then(|error| error.expected_hash)
            .map(|hash| phase8_hash_json_literal(&hash)),
        "error.kind" => result
            .error
            .as_ref()
            .map(|error| phase8_json_string_literal(&error.kind)),
        "error.reason_code" => result
            .error
            .as_ref()
            .and_then(|error| error.reason_code.as_deref())
            .map(phase8_json_string_literal),
        "policy.hash" => Some(phase8_hash_json_literal(&result.policy.hash)),
        "policy.id" => Some(phase8_json_string_literal(&result.policy.id)),
        "policy.version" => Some(result.policy.version.to_string()),
        "input_file_hash" => normalized_result
            .map(|normalized| phase8_hash_json_literal(&normalized.artifact.input_file_hash)),
        "expected_certificate_hash" => normalized_result.map(|normalized| {
            phase8_hash_json_literal(&normalized.artifact.expected_certificate_hash)
        }),
        _ => None,
    }
}

fn phase8_ai_audit_normalized_comparison_field(
    field: &str,
    result: &Phase8NormalizedCheckResult,
) -> Option<String> {
    match field {
        "module" => Some(phase8_json_string_literal(&result.artifact.module)),
        "input_file_hash" => Some(phase8_hash_json_literal(&result.artifact.input_file_hash)),
        "expected_certificate_hash" => Some(phase8_hash_json_literal(
            &result.artifact.expected_certificate_hash,
        )),
        "artifact_hash" => Some(phase8_hash_json_literal(&result.artifact_hash())),
        "policy.hash" => Some(phase8_hash_json_literal(&result.policy.hash)),
        "policy.id" => Some(phase8_json_string_literal(&result.policy.id)),
        "policy.version" => Some(result.policy.version.to_string()),
        "comparison.status" => Some(phase8_json_string_literal(
            result.comparison.status.as_str(),
        )),
        "comparison.disagreements" => Some(canonical_json_array(
            result
                .comparison
                .disagreements
                .iter()
                .map(Phase8NormalizedDisagreement::canonical_json)
                .collect(),
        )),
        "comparison.matching_fields" => {
            Some(phase8_json_string_array(&result.comparison.matching_fields))
        }
        "comparison.missing_checker_profiles" => Some(phase8_json_string_array(
            &result.comparison.missing_checker_profiles,
        )),
        "comparison.status_reasons" => Some(canonical_json_array(
            result
                .comparison
                .status_reasons
                .iter()
                .map(Phase8NormalizedStatusReason::canonical_json)
                .collect(),
        )),
        _ => None,
    }
}

const PHASE8_AI_AUDIT_POLICY_GATED_FIELDS: &[&str] = &[
    "source_text",
    "source_excerpt",
    "theorem_statement",
    "proof_script",
    "tactic_trace",
    "tactic_script",
    "elaboration_trace",
    "ai_search_trace",
];

const PHASE8_AI_AUDIT_MACHINE_FIELDS: &[&str] = &[
    "certificate_hash",
    "checker_binary_hash",
    "checker_binary_id",
    "checker_build_hash",
    "checker_id",
    "checker_profile",
    "checker_version",
    "error.actual_hash",
    "error.core_path",
    "error.declaration",
    "error.expected_hash",
    "error.kind",
    "error.reason_code",
    "expected_certificate_hash",
    "input_file_hash",
    "module",
    "policy.hash",
    "policy.id",
    "policy.version",
    "status",
];

const PHASE8_AI_AUDIT_NORMALIZED_COMPARISON_FIELDS: &[&str] = &[
    "artifact_hash",
    "comparison.disagreements",
    "comparison.matching_fields",
    "comparison.missing_checker_profiles",
    "comparison.status",
    "comparison.status_reasons",
    "expected_certificate_hash",
    "input_file_hash",
    "module",
    "policy.hash",
    "policy.id",
    "policy.version",
];

fn phase8_ai_included_field_allowed(field: &str) -> bool {
    PHASE8_AI_AUDIT_MACHINE_FIELDS.contains(&field)
        || PHASE8_AI_AUDIT_NORMALIZED_COMPARISON_FIELDS.contains(&field)
}

fn phase8_ai_policy_gated_field_name(field: &str) -> bool {
    PHASE8_AI_AUDIT_POLICY_GATED_FIELDS.contains(&field)
}

fn phase8_ai_audit_redaction_allowed(value: &str) -> bool {
    matches!(value, "default" | "strict" | "release")
}

fn phase8_ai_audit_classification_category_allowed(value: &str) -> bool {
    matches!(
        value,
        "certificate_encoding_bug"
            | "import_resolution_bug"
            | "certificate_generator_bug"
            | "kernel_checker_disagreement"
            | "axiom_policy_violation"
            | "source_to_certificate_staleness"
            | "unsupported_feature"
            | "checker_resource_limit"
            | "checker_internal_bug"
            | "unknown"
    )
}

fn phase8_parse_ai_audit_sidecar_status(
    value: &str,
    field: &str,
) -> Result<Phase8AiAuditSidecarStatus, Phase8AuditSidecarValidationError> {
    match value {
        "summarized" => Ok(Phase8AiAuditSidecarStatus::Summarized),
        "triaged" => Ok(Phase8AiAuditSidecarStatus::Triaged),
        "suggested_fix" => Ok(Phase8AiAuditSidecarStatus::SuggestedFix),
        "suggested_challenge" => Ok(Phase8AiAuditSidecarStatus::SuggestedChallenge),
        "inconclusive" => Ok(Phase8AiAuditSidecarStatus::Inconclusive),
        _ => Err(Phase8AuditSidecarValidationError::value(
            Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
            field,
            "AiAuditSidecar.status",
            "invalid_enum",
        )),
    }
}

fn phase8_find_static_forbidden_sidecar_field(value: &JsonValue<'_>, path: &str) -> Option<String> {
    if let Some(members) = value.object_members() {
        for member in members {
            let key = member.key();
            let field_path = phase8_ai_join_path(path, key);
            if phase8_ai_reserved_forbidden_field_name(key)
                || phase8_ai_secret_forbidden_field_name(key)
                || (phase8_ai_policy_gated_field_name(key) && path != "$")
            {
                return Some(field_path);
            }
            if let Some(nested) =
                phase8_find_static_forbidden_sidecar_field(member.value(), &field_path)
            {
                return Some(nested);
            }
        }
        return None;
    }
    if let Some(elements) = value.array_elements() {
        for (index, element) in elements.iter().enumerate() {
            if let Some(nested) =
                phase8_find_static_forbidden_sidecar_field(element, &format!("{path}[{index}]"))
            {
                return Some(nested);
            }
        }
    }
    None
}

fn phase8_ai_reserved_forbidden_field_name(value: &str) -> bool {
    matches!(
        value,
        "verdict"
            | "accepted"
            | "checked"
            | "verified"
            | "checker_status"
            | "certificate_valid"
            | "proof_valid"
            | "generated_certificate"
            | "generated_certificate_bytes"
            | "certificate_bytes"
            | "proof_term"
            | "raw_certificate"
            | "raw_proof"
    )
}

fn phase8_ai_secret_forbidden_field_name(value: &str) -> bool {
    matches!(
        value,
        "secret"
            | "token"
            | "access_token"
            | "refresh_token"
            | "api_key"
            | "password"
            | "authorization"
            | "private_key"
    )
}

fn phase8_ai_object_members<'value, 'src>(
    value: &'value JsonValue<'src>,
    field: &str,
    expected: &str,
    reason_code: Phase8AuditSidecarValidationReasonCode,
) -> Result<&'value [JsonMember<'src>], Phase8AuditSidecarValidationError> {
    value.object_members().ok_or_else(|| {
        Phase8AuditSidecarValidationError::value(
            reason_code,
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

fn phase8_ai_required_value<'value, 'src>(
    members: &'value [JsonMember<'src>],
    name: &str,
    field: &str,
    expected: &str,
    reason_code: Phase8AuditSidecarValidationReasonCode,
) -> Result<&'value JsonValue<'src>, Phase8AuditSidecarValidationError> {
    if duplicate_member(members, name) {
        return Err(Phase8AuditSidecarValidationError::value(
            reason_code,
            field,
            "unique_object_keys",
            "duplicate_field",
        ));
    }
    let Some(member) = members.iter().find(|member| member.key() == name) else {
        return Err(Phase8AuditSidecarValidationError::value(
            reason_code,
            field,
            expected,
            "missing",
        ));
    };
    if member.value().kind() == JsonValueKind::Null {
        return Err(Phase8AuditSidecarValidationError::value(
            reason_code,
            field,
            expected,
            "null_not_allowed",
        ));
    }
    Ok(member.value())
}

fn phase8_ai_optional_value<'value, 'src>(
    members: &'value [JsonMember<'src>],
    name: &str,
    field: &str,
    reason_code: Phase8AuditSidecarValidationReasonCode,
) -> Result<Option<&'value JsonValue<'src>>, Phase8AuditSidecarValidationError> {
    if duplicate_member(members, name) {
        return Err(Phase8AuditSidecarValidationError::value(
            reason_code,
            field,
            "unique_object_keys",
            "duplicate_field",
        ));
    }
    Ok(members
        .iter()
        .find(|member| member.key() == name)
        .map(JsonMember::value))
}

fn phase8_ai_required_fixed_string(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
    fixed: &str,
    reason_code: Phase8AuditSidecarValidationReasonCode,
) -> Result<(), Phase8AuditSidecarValidationError> {
    let value = phase8_ai_required_string(members, name, field, fixed, reason_code)?;
    if value != fixed {
        return Err(Phase8AuditSidecarValidationError::value(
            reason_code,
            field,
            fixed,
            "invalid_enum",
        ));
    }
    Ok(())
}

fn phase8_ai_required_string(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
    expected: &str,
    reason_code: Phase8AuditSidecarValidationReasonCode,
) -> Result<String, Phase8AuditSidecarValidationError> {
    let value = phase8_ai_required_value(members, name, field, expected, reason_code)?;
    value.string_value().map(ToOwned::to_owned).ok_or_else(|| {
        Phase8AuditSidecarValidationError::value(
            reason_code,
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

fn phase8_ai_optional_string(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
    expected: &str,
    reason_code: Phase8AuditSidecarValidationReasonCode,
) -> Result<Option<String>, Phase8AuditSidecarValidationError> {
    let Some(value) = phase8_ai_optional_value(members, name, field, reason_code)? else {
        return Ok(None);
    };
    if value.kind() == JsonValueKind::Null {
        return Err(Phase8AuditSidecarValidationError::value(
            reason_code,
            field,
            expected,
            "null_not_allowed",
        ));
    }
    value
        .string_value()
        .map(|value| Some(value.to_owned()))
        .ok_or_else(|| {
            Phase8AuditSidecarValidationError::value(reason_code, field, expected, "wrong_type")
        })
}

fn phase8_ai_required_hash(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
    reason_code: Phase8AuditSidecarValidationReasonCode,
) -> Result<Hash, Phase8AuditSidecarValidationError> {
    let value = phase8_ai_required_string(members, name, field, "sha256:<lower-hex>", reason_code)?;
    parse_hash_string(&value).map_err(|_| {
        Phase8AuditSidecarValidationError::value(
            reason_code,
            field,
            "sha256:<lower-hex>",
            "invalid_hash_format",
        )
    })
}

fn phase8_ai_optional_hash(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
    reason_code: Phase8AuditSidecarValidationReasonCode,
) -> Result<Option<Hash>, Phase8AuditSidecarValidationError> {
    let Some(value) =
        phase8_ai_optional_string(members, name, field, "sha256:<lower-hex>", reason_code)?
    else {
        return Ok(None);
    };
    parse_hash_string(&value).map(Some).map_err(|_| {
        Phase8AuditSidecarValidationError::value(
            reason_code,
            field,
            "sha256:<lower-hex>",
            "invalid_hash_format",
        )
    })
}

fn phase8_ai_required_bool(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
    reason_code: Phase8AuditSidecarValidationReasonCode,
) -> Result<bool, Phase8AuditSidecarValidationError> {
    let value = phase8_ai_required_value(members, name, field, "bool", reason_code)?;
    value.bool_value().ok_or_else(|| {
        Phase8AuditSidecarValidationError::value(
            reason_code,
            field,
            "bool",
            if value.kind() == JsonValueKind::Null {
                "null_not_allowed"
            } else {
                "wrong_type"
            },
        )
    })
}

fn phase8_ai_required_positive_i64(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
    reason_code: Phase8AuditSidecarValidationReasonCode,
) -> Result<u64, Phase8AuditSidecarValidationError> {
    let value = phase8_ai_required_value(members, name, field, "positive_i64", reason_code)?;
    let Some(raw) = value.number_raw() else {
        return Err(Phase8AuditSidecarValidationError::value(
            reason_code,
            field,
            "positive_i64",
            "wrong_type",
        ));
    };
    if raw.contains('.') || raw.contains('e') || raw.contains('E') {
        return Err(Phase8AuditSidecarValidationError::value(
            reason_code,
            field,
            "positive_i64",
            "wrong_type",
        ));
    }
    let value = raw.parse::<u64>().map_err(|_| {
        Phase8AuditSidecarValidationError::value(
            reason_code,
            field,
            "positive_i64",
            "integer_out_of_range",
        )
    })?;
    if value == 0 {
        return Err(Phase8AuditSidecarValidationError::value(
            reason_code,
            field,
            "positive_i64",
            "non_positive_integer",
        ));
    }
    if value > i64::MAX as u64 {
        return Err(Phase8AuditSidecarValidationError::value(
            reason_code,
            field,
            "positive_i64",
            "integer_out_of_range",
        ));
    }
    Ok(value)
}

fn phase8_ai_required_visible_ascii(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
    expected: &str,
) -> Result<String, Phase8AuditSidecarValidationError> {
    let value = phase8_ai_required_string(
        members,
        name,
        field,
        expected,
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;
    phase8_validate_visible_ascii(&value, field, expected)?;
    Ok(value)
}

fn phase8_ai_optional_visible_ascii(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
    expected: &str,
) -> Result<Option<String>, Phase8AuditSidecarValidationError> {
    let Some(value) = phase8_ai_optional_string(
        members,
        name,
        field,
        expected,
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?
    else {
        return Ok(None);
    };
    phase8_validate_visible_ascii(&value, field, expected)?;
    Ok(Some(value))
}

fn phase8_ai_required_text_string(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
) -> Result<String, Phase8AuditSidecarValidationError> {
    let value = phase8_ai_required_string(
        members,
        name,
        field,
        "non_empty_text_string",
        Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
    )?;
    phase8_validate_ai_text(&value, field)?;
    Ok(value)
}

fn phase8_ai_required_included_fields(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
    reason_code: Phase8AuditSidecarValidationReasonCode,
) -> Result<Vec<String>, Phase8AuditSidecarValidationError> {
    let value = phase8_ai_required_value(members, name, field, "array", reason_code)?;
    let Some(elements) = value.array_elements() else {
        return Err(Phase8AuditSidecarValidationError::value(
            reason_code,
            field,
            "array",
            "wrong_type",
        ));
    };
    let mut fields = Vec::new();
    for (index, element) in elements.iter().enumerate() {
        let path = format!("{field}[{index}]");
        let Some(field_path) = element.string_value() else {
            return Err(Phase8AuditSidecarValidationError::value(
                reason_code,
                path,
                "field_path_string",
                if element.kind() == JsonValueKind::Null {
                    "null_not_allowed"
                } else {
                    "wrong_type"
                },
            ));
        };
        fields.push(field_path.to_owned());
    }
    for (index, field_path) in fields.iter().enumerate() {
        if !phase8_ai_included_field_allowed(field_path) {
            return Err(Phase8AuditSidecarValidationError::value(
                reason_code,
                format!("{field}[{index}]"),
                "allowed_input_policy_field",
                "unknown_field",
            ));
        }
    }
    for index in 1..fields.len() {
        if fields[index - 1] > fields[index] {
            return Err(Phase8AuditSidecarValidationError::value(
                reason_code,
                format!("{field}[{index}]"),
                "field_path_bytewise_ascending",
                "order_violation",
            ));
        }
    }
    let mut seen = BTreeSet::new();
    for (index, field_path) in fields.iter().enumerate() {
        if !seen.insert(field_path) {
            return Err(Phase8AuditSidecarValidationError::value(
                reason_code,
                format!("{field}[{index}]"),
                "unique_included_fields",
                "duplicate_field",
            ));
        }
    }
    Ok(fields)
}

fn phase8_ai_reject_unknown_fields(
    members: &[JsonMember<'_>],
    allowed: &[&str],
    container_path: &str,
    reason_code: Phase8AuditSidecarValidationReasonCode,
) -> Result<(), Phase8AuditSidecarValidationError> {
    let mut counts = BTreeMap::<String, usize>::new();
    for member in members {
        *counts.entry(member.key().to_owned()).or_default() += 1;
    }
    let mut duplicates = counts
        .iter()
        .filter_map(|(field, count)| (*count > 1).then_some(field.as_str()))
        .collect::<Vec<_>>();
    duplicates.sort_by(|left, right| phase8_rfc8785_object_key_cmp(left, right));
    if let Some(field) = duplicates.first() {
        return Err(Phase8AuditSidecarValidationError::value(
            reason_code,
            phase8_ai_join_path(container_path, field),
            "unique_object_keys",
            "duplicate_field",
        ));
    }
    let mut unknown = counts
        .keys()
        .filter(|field| !allowed.contains(&field.as_str()))
        .map(String::as_str)
        .collect::<Vec<_>>();
    unknown.sort_by(|left, right| phase8_rfc8785_object_key_cmp(left, right));
    if let Some(field) = unknown.first() {
        return Err(Phase8AuditSidecarValidationError::value(
            reason_code,
            phase8_ai_join_path(container_path, field),
            "absent",
            "unknown_field",
        ));
    }
    Ok(())
}

fn phase8_validate_visible_ascii(
    value: &str,
    field: &str,
    expected: &str,
) -> Result<(), Phase8AuditSidecarValidationError> {
    if value.is_empty() {
        return Err(Phase8AuditSidecarValidationError::value(
            Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
            field,
            expected,
            "empty_string",
        ));
    }
    if !value.bytes().all(|byte| (0x21..=0x7e).contains(&byte)) {
        return Err(Phase8AuditSidecarValidationError::value(
            Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
            field,
            expected,
            "invalid_string_format",
        ));
    }
    Ok(())
}

fn phase8_validate_ai_text(
    value: &str,
    field: &str,
) -> Result<(), Phase8AuditSidecarValidationError> {
    if value.is_empty() {
        return Err(Phase8AuditSidecarValidationError::value(
            Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
            field,
            "non_empty_text_string",
            "empty_string",
        ));
    }
    if value
        .chars()
        .any(|ch| matches!(ch, '\u{0000}'..='\u{001f}' | '\u{007f}'))
    {
        return Err(Phase8AuditSidecarValidationError::value(
            Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid,
            field,
            "non_empty_text_string",
            "invalid_string_format",
        ));
    }
    Ok(())
}

fn phase8_ai_join_path(container: &str, field: &str) -> String {
    if container == "$" {
        field.to_owned()
    } else {
        format!("{container}.{field}")
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

fn parse_phase8_challenge_generation_request_value(
    value: &JsonValue<'_>,
) -> Result<Phase8ChallengeGenerationRequest, Phase8RequestValidationError> {
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
        PHASE8_CHALLENGE_GENERATION_REQUEST_SCHEMA,
        PHASE8_CHALLENGE_GENERATION_REQUEST_SCHEMA,
    )?;
    let request_id = required_string_field(members, "request_id", "request_id", "request_id")?;
    if !phase8_valid_request_id(&request_id) {
        return Err(Phase8RequestValidationError::value_failure(
            "request_id",
            "request_id",
            "invalid_string_format",
        ));
    }
    let parsed_request_hash = required_hash_field(
        members,
        "request_hash",
        "request_hash",
        "sha256:<lower-hex>",
    )?;
    let challenge_id =
        required_string_field(members, "challenge_id", "challenge_id", "challenge_id")?;
    if !phase8_valid_challenge_id(&challenge_id) {
        return Err(Phase8RequestValidationError::value_failure(
            "challenge_id",
            "challenge_id",
            "invalid_name_format",
        ));
    }
    let policy_hash =
        required_hash_field(members, "policy_hash", "policy_hash", "sha256:<lower-hex>")?;
    let module = required_string_field(members, "module", "module", "module_name")?;
    if !phase8_valid_dotted_name(&module) {
        return Err(Phase8RequestValidationError::value_failure(
            "module",
            "module_name",
            "invalid_name_format",
        ));
    }
    let imports = parse_phase8_challenge_imports_field(members, "imports")?;
    let base_certificate =
        parse_phase8_challenge_base_certificate_field(members, "base_certificate")?;
    let mutation = parse_phase8_challenge_mutation_field(members, "mutation", false)?;
    let output = parse_phase8_challenge_generation_output_field(members)?;
    let generated_by = parse_phase8_challenge_generated_by_field(members, "generated_by")?;
    reject_unknown_fields(members, CHALLENGE_GENERATION_REQUEST_FIELDS, "$")?;
    let request = Phase8ChallengeGenerationRequest {
        request_id,
        challenge_id,
        policy_hash,
        module,
        imports,
        base_certificate,
        mutation,
        output,
        generated_by,
    };
    validate_phase8_challenge_generation_request_domain(&request)?;
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

fn parse_phase8_challenge_manifest_value(
    value: &JsonValue<'_>,
    root_path: &str,
) -> Result<Phase8ChallengeManifest, Phase8RequestValidationError> {
    let members = object_members_or_policy_error(value, root_path, "object")?;
    required_fixed_string_field(
        members,
        "schema",
        &phase8_join_json_path(root_path, "schema"),
        PHASE8_CHALLENGE_MANIFEST_SCHEMA,
        PHASE8_CHALLENGE_MANIFEST_SCHEMA,
    )?;
    let challenge_id = required_string_field(
        members,
        "challenge_id",
        &phase8_join_json_path(root_path, "challenge_id"),
        "challenge_id",
    )?;
    if !phase8_valid_challenge_id(&challenge_id) {
        return Err(Phase8RequestValidationError::value_failure(
            phase8_join_json_path(root_path, "challenge_id"),
            "challenge_id",
            "invalid_name_format",
        ));
    }
    let policy_hash = required_hash_field(
        members,
        "policy_hash",
        &phase8_join_json_path(root_path, "policy_hash"),
        "sha256:<lower-hex>",
    )?;
    let module = required_string_field(
        members,
        "module",
        &phase8_join_json_path(root_path, "module"),
        "module_name",
    )?;
    if !phase8_valid_dotted_name(&module) {
        return Err(Phase8RequestValidationError::value_failure(
            phase8_join_json_path(root_path, "module"),
            "module_name",
            "invalid_name_format",
        ));
    }
    let imports = parse_phase8_challenge_imports_field(
        members,
        &phase8_join_json_path(root_path, "imports"),
    )?;
    let base_certificate = parse_phase8_challenge_base_certificate_field(
        members,
        &phase8_join_json_path(root_path, "base_certificate"),
    )?;
    let mutated_certificate = parse_phase8_challenge_mutated_certificate_field(
        members,
        &phase8_join_json_path(root_path, "mutated_certificate"),
    )?;
    let mutation = parse_phase8_challenge_mutation_field(
        members,
        &phase8_join_json_path(root_path, "mutation"),
        true,
    )?;
    let outcome_hint = parse_phase8_challenge_outcome_hint_field(
        members,
        &phase8_join_json_path(root_path, "outcome_hint"),
    )?;
    let replay =
        parse_phase8_challenge_replay_field(members, &phase8_join_json_path(root_path, "replay"))?;
    let generated_by = parse_phase8_challenge_generated_by_field(
        members,
        &phase8_join_json_path(root_path, "generated_by"),
    )?;
    reject_unknown_fields(members, CHALLENGE_MANIFEST_FIELDS, root_path)?;
    Ok(Phase8ChallengeManifest {
        challenge_id,
        policy_hash,
        module,
        imports,
        base_certificate,
        mutated_certificate,
        mutation,
        outcome_hint,
        replay,
        generated_by,
    })
}

fn parse_phase8_challenge_output_store_manifest_value(
    value: &JsonValue<'_>,
    root_path: &str,
) -> Result<Phase8ChallengeOutputStoreManifest, Phase8RequestValidationError> {
    let members = object_members_or_policy_error(value, root_path, "object")?;
    required_fixed_string_field(
        members,
        "schema",
        &format!("{root_path}.schema"),
        PHASE8_CHALLENGE_OUTPUT_STORE_MANIFEST_SCHEMA,
        PHASE8_CHALLENGE_OUTPUT_STORE_MANIFEST_SCHEMA,
    )?;
    let entries_value =
        required_field_value(members, "entries", &format!("{root_path}.entries"), "array")?;
    let Some(entry_values) = entries_value.array_elements() else {
        return Err(wrong_type_error(
            &format!("{root_path}.entries"),
            "array",
            entries_value.kind(),
        )
        .into());
    };
    let mut entries = Vec::new();
    for (index, entry_value) in entry_values.iter().enumerate() {
        let path = format!("{root_path}.entries[{index}]");
        let entry_members = object_members_or_policy_error(entry_value, &path, "object")?;
        let challenge_id = required_string_field(
            entry_members,
            "challenge_id",
            &format!("{path}.challenge_id"),
            "challenge_id",
        )?;
        if !phase8_valid_challenge_id(&challenge_id) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{path}.challenge_id"),
                "challenge_id",
                "invalid_name_format",
            ));
        }
        let manifest_path = required_string_field(
            entry_members,
            "manifest_path",
            &format!("{path}.manifest_path"),
            "workspace_relative_path",
        )?;
        if !phase8_valid_workspace_relative_path(&manifest_path) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{path}.manifest_path"),
                "workspace_relative_path",
                "invalid_path",
            ));
        }
        let manifest_hash = required_hash_field(
            entry_members,
            "manifest_hash",
            &format!("{path}.manifest_hash"),
            "sha256:<lower-hex>",
        )?;
        reject_unknown_fields(entry_members, CHALLENGE_OUTPUT_STORE_ENTRY_FIELDS, &path)?;
        entries.push(Phase8ChallengeOutputStoreEntry {
            challenge_id,
            manifest_path,
            manifest_hash,
        });
    }
    reject_unknown_fields(members, CHALLENGE_OUTPUT_STORE_MANIFEST_FIELDS, root_path)?;
    validate_challenge_output_store_domain(&entries, root_path)?;
    Ok(Phase8ChallengeOutputStoreManifest { entries })
}

fn parse_phase8_challenge_replay_result_value(
    value: &JsonValue<'_>,
    root_path: &str,
) -> Result<Phase8ChallengeReplayResult, Phase8RequestValidationError> {
    let members = object_members_or_policy_error(value, root_path, "object")?;
    required_fixed_string_field(
        members,
        "schema",
        &phase8_join_json_path(root_path, "schema"),
        PHASE8_CHALLENGE_REPLAY_RESULT_SCHEMA,
        PHASE8_CHALLENGE_REPLAY_RESULT_SCHEMA,
    )?;
    let result_id = required_string_field(
        members,
        "result_id",
        &phase8_join_json_path(root_path, "result_id"),
        "result_id",
    )?;
    if !phase8_valid_request_id(&result_id) {
        return Err(Phase8RequestValidationError::value_failure(
            phase8_join_json_path(root_path, "result_id"),
            "result_id",
            "invalid_string_format",
        ));
    }
    let parsed_result_hash = required_hash_field(
        members,
        "result_hash",
        &phase8_join_json_path(root_path, "result_hash"),
        "sha256:<lower-hex>",
    )?;
    let challenge_id = required_string_field(
        members,
        "challenge_id",
        &phase8_join_json_path(root_path, "challenge_id"),
        "challenge_id",
    )?;
    if !phase8_valid_challenge_id(&challenge_id) {
        return Err(Phase8RequestValidationError::value_failure(
            phase8_join_json_path(root_path, "challenge_id"),
            "challenge_id",
            "invalid_name_format",
        ));
    }
    let manifest_hash = required_hash_field(
        members,
        "manifest_hash",
        &phase8_join_json_path(root_path, "manifest_hash"),
        "sha256:<lower-hex>",
    )?;
    let mutated_file_hash = required_hash_field(
        members,
        "mutated_file_hash",
        &phase8_join_json_path(root_path, "mutated_file_hash"),
        "sha256:<lower-hex>",
    )?;
    let mutated_claimed_certificate_hash = optional_hash_field(
        members,
        "mutated_claimed_certificate_hash",
        &phase8_join_json_path(root_path, "mutated_claimed_certificate_hash"),
        "sha256:<lower-hex>",
    )?;
    let checker_results = parse_phase8_challenge_replay_checker_results(
        members,
        &phase8_join_json_path(root_path, "checker_results"),
    )?;
    let missing_checker_profiles = required_string_array_field(
        members,
        "missing_checker_profiles",
        &phase8_join_json_path(root_path, "missing_checker_profiles"),
        "checker_profile",
    )?;
    let normalized_result_hash = optional_hash_field(
        members,
        "normalized_result_hash",
        &phase8_join_json_path(root_path, "normalized_result_hash"),
        "sha256:<lower-hex>",
    )?;
    let policy_hash = required_hash_field(
        members,
        "policy_hash",
        &phase8_join_json_path(root_path, "policy_hash"),
        "sha256:<lower-hex>",
    )?;
    let artifact_hash = required_hash_field(
        members,
        "artifact_hash",
        &phase8_join_json_path(root_path, "artifact_hash"),
        "sha256:<lower-hex>",
    )?;
    let comparison_status = parse_optional_phase8_normalized_comparison_status_field(
        members,
        "comparison_status",
        &phase8_join_json_path(root_path, "comparison_status"),
    )?;
    let observed_error_kinds = required_string_array_field(
        members,
        "observed_error_kinds",
        &phase8_join_json_path(root_path, "observed_error_kinds"),
        "error_kind",
    )?;
    reject_unknown_fields(members, CHALLENGE_REPLAY_RESULT_FIELDS, root_path)?;
    if normalized_result_hash.is_some() != comparison_status.is_some() {
        return Err(Phase8RequestValidationError::value_failure(
            phase8_join_json_path(root_path, "comparison_status"),
            if normalized_result_hash.is_some() {
                "NormalizedCheckResult.comparison.status"
            } else {
                "absent_without_normalized_result_hash"
            },
            if comparison_status.is_some() {
                "present"
            } else {
                "missing"
            },
        ));
    }
    validate_string_set_unique(
        &missing_checker_profiles,
        &phase8_join_json_path(root_path, "missing_checker_profiles"),
        "duplicate_checker_profile",
    )?;
    validate_string_set_sorted_unique(
        &observed_error_kinds,
        &phase8_join_json_path(root_path, "observed_error_kinds"),
        "error_kind_bytewise_ascending",
        "duplicate_error_kind",
    )?;
    validate_challenge_replay_checker_results_domain(
        &checker_results,
        &phase8_join_json_path(root_path, "checker_results"),
    )?;
    let result = Phase8ChallengeReplayResult {
        result_id,
        challenge_id,
        manifest_hash,
        mutated_file_hash,
        mutated_claimed_certificate_hash,
        checker_results,
        missing_checker_profiles,
        normalized_result_hash,
        policy_hash,
        artifact_hash,
        comparison_status,
        observed_error_kinds,
    };
    let recomputed = result.result_hash();
    if recomputed != parsed_result_hash {
        return Err(Phase8RequestValidationError::hash_failure(
            phase8_join_json_path(root_path, "result_hash"),
            recomputed,
            parsed_result_hash,
        ));
    }
    Ok(result)
}

fn parse_phase8_challenge_replay_checker_results(
    members: &[JsonMember<'_>],
    field: &str,
) -> Result<Vec<Phase8ChallengeReplayCheckerResult>, Phase8RequestValidationError> {
    let value = required_field_value(members, last_json_path_component(field), field, "array")?;
    let Some(elements) = value.array_elements() else {
        return Err(wrong_type_error(field, "array", value.kind()).into());
    };
    let mut out = Vec::new();
    for (index, element) in elements.iter().enumerate() {
        let path = format!("{field}[{index}]");
        let result_members = object_members_or_policy_error(element, &path, "object")?;
        let result_id = required_string_field(
            result_members,
            "result_id",
            &format!("{path}.result_id"),
            "result_id",
        )?;
        if !phase8_valid_request_id(&result_id) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{path}.result_id"),
                "result_id",
                "invalid_string_format",
            ));
        }
        let result_hash = required_hash_field(
            result_members,
            "result_hash",
            &format!("{path}.result_hash"),
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
            "checker_profile",
        )?;
        if !phase8_visible_ascii_nonempty(&checker_profile) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{path}.checker_profile"),
                "checker_profile",
                "invalid_string_format",
            ));
        }
        reject_unknown_fields(
            result_members,
            CHALLENGE_REPLAY_CHECKER_RESULT_FIELDS,
            &path,
        )?;
        out.push(Phase8ChallengeReplayCheckerResult {
            result_id,
            result_hash,
            run_artifact_hash,
            checker_profile,
        });
    }
    Ok(out)
}

fn parse_phase8_challenge_replay_store_manifest_value(
    value: &JsonValue<'_>,
    root_path: &str,
) -> Result<Phase8ChallengeReplayStoreManifest, Phase8RequestValidationError> {
    let members = object_members_or_policy_error(value, root_path, "object")?;
    required_fixed_string_field(
        members,
        "schema",
        &format!("{root_path}.schema"),
        PHASE8_CHALLENGE_REPLAY_STORE_MANIFEST_SCHEMA,
        PHASE8_CHALLENGE_REPLAY_STORE_MANIFEST_SCHEMA,
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
        let challenge_id = required_string_field(
            result_members,
            "challenge_id",
            &format!("{path}.challenge_id"),
            "challenge_id",
        )?;
        if !phase8_valid_challenge_id(&challenge_id) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{path}.challenge_id"),
                "challenge_id",
                "invalid_name_format",
            ));
        }
        let manifest_hash = required_hash_field(
            result_members,
            "manifest_hash",
            &format!("{path}.manifest_hash"),
            "sha256:<lower-hex>",
        )?;
        let result_hash = required_hash_field(
            result_members,
            "result_hash",
            &format!("{path}.result_hash"),
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
        reject_unknown_fields(result_members, CHALLENGE_REPLAY_STORE_ENTRY_FIELDS, &path)?;
        results.push(Phase8ChallengeReplayStoreEntry {
            challenge_id,
            manifest_hash,
            result_hash,
            artifact_hash,
            path: result_path,
            file_hash,
        });
    }
    reject_unknown_fields(members, CHALLENGE_REPLAY_STORE_MANIFEST_FIELDS, root_path)?;
    validate_challenge_replay_store_domain(&results, root_path)?;
    Ok(Phase8ChallengeReplayStoreManifest { results })
}

fn parse_phase8_challenge_coverage_summary_value(
    value: &JsonValue<'_>,
    root_path: &str,
) -> Result<Phase8ChallengeCoverageSummary, Phase8RequestValidationError> {
    let members = object_members_or_policy_error(value, root_path, "object")?;
    required_fixed_string_field(
        members,
        "schema",
        &phase8_join_json_path(root_path, "schema"),
        PHASE8_CHALLENGE_COVERAGE_SUMMARY_SCHEMA,
        PHASE8_CHALLENGE_COVERAGE_SUMMARY_SCHEMA,
    )?;
    let summary_id = required_string_field(
        members,
        "summary_id",
        &phase8_join_json_path(root_path, "summary_id"),
        "chcov_<64-lower-hex>",
    )?;
    if !phase8_valid_challenge_coverage_summary_id(&summary_id) {
        return Err(Phase8RequestValidationError::value_failure(
            phase8_join_json_path(root_path, "summary_id"),
            "chcov_<64-lower-hex>",
            "invalid_name_format",
        ));
    }
    let parsed_summary_hash = required_hash_field(
        members,
        "summary_hash",
        &phase8_join_json_path(root_path, "summary_hash"),
        "sha256:<lower-hex>",
    )?;
    let policy_hash = required_hash_field(
        members,
        "policy_hash",
        &phase8_join_json_path(root_path, "policy_hash"),
        "sha256:<lower-hex>",
    )?;
    let artifact_hash = required_hash_field(
        members,
        "artifact_hash",
        &phase8_join_json_path(root_path, "artifact_hash"),
        "sha256:<lower-hex>",
    )?;
    let target_normalized_result_hash = required_hash_field(
        members,
        "target_normalized_result_hash",
        &phase8_join_json_path(root_path, "target_normalized_result_hash"),
        "sha256:<lower-hex>",
    )?;
    let challenge_store_manifest_hash = required_hash_field(
        members,
        "challenge_store_manifest_hash",
        &phase8_join_json_path(root_path, "challenge_store_manifest_hash"),
        "sha256:<lower-hex>",
    )?;
    let result_store_manifest_hash = required_hash_field(
        members,
        "result_store_manifest_hash",
        &phase8_join_json_path(root_path, "result_store_manifest_hash"),
        "sha256:<lower-hex>",
    )?;
    let total_challenges = required_nonnegative_u64_field(
        members,
        "total_challenges",
        &phase8_join_json_path(root_path, "total_challenges"),
    )?;
    let replayed_challenges = required_nonnegative_u64_field(
        members,
        "replayed_challenges",
        &phase8_join_json_path(root_path, "replayed_challenges"),
    )?;
    let unexpected_acceptances = required_nonnegative_u64_field(
        members,
        "unexpected_acceptances",
        &phase8_join_json_path(root_path, "unexpected_acceptances"),
    )?;
    let entries = parse_phase8_challenge_coverage_entries(
        members,
        &phase8_join_json_path(root_path, "entries"),
    )?;
    reject_unknown_fields(members, CHALLENGE_COVERAGE_SUMMARY_FIELDS, root_path)?;
    validate_challenge_coverage_summary_domain(
        &entries,
        total_challenges,
        replayed_challenges,
        unexpected_acceptances,
        root_path,
    )?;
    let summary = Phase8ChallengeCoverageSummary {
        summary_id,
        policy_hash,
        artifact_hash,
        target_normalized_result_hash,
        challenge_store_manifest_hash,
        result_store_manifest_hash,
        total_challenges,
        replayed_challenges,
        unexpected_acceptances,
        entries,
    };
    let recomputed = summary.summary_hash();
    if recomputed != parsed_summary_hash {
        return Err(Phase8RequestValidationError::hash_failure(
            phase8_join_json_path(root_path, "summary_hash"),
            recomputed,
            parsed_summary_hash,
        ));
    }
    let expected_summary_id = phase8_challenge_coverage_summary_id(parsed_summary_hash);
    if summary.summary_id != expected_summary_id {
        return Err(Phase8RequestValidationError::value_failure(
            phase8_join_json_path(root_path, "summary_id"),
            expected_summary_id,
            summary.summary_id,
        ));
    }
    Ok(summary)
}

fn parse_phase8_challenge_coverage_entries(
    members: &[JsonMember<'_>],
    field: &str,
) -> Result<Vec<Phase8ChallengeCoverageEntry>, Phase8RequestValidationError> {
    let value = required_field_value(members, last_json_path_component(field), field, "array")?;
    let Some(elements) = value.array_elements() else {
        return Err(wrong_type_error(field, "array", value.kind()).into());
    };
    let mut entries = Vec::new();
    for (index, element) in elements.iter().enumerate() {
        let path = format!("{field}[{index}]");
        let entry_members = object_members_or_policy_error(element, &path, "object")?;
        let challenge_id = required_string_field(
            entry_members,
            "challenge_id",
            &format!("{path}.challenge_id"),
            "challenge_id",
        )?;
        if !phase8_valid_challenge_id(&challenge_id) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{path}.challenge_id"),
                "challenge_id",
                "invalid_name_format",
            ));
        }
        let manifest_hash = required_hash_field(
            entry_members,
            "manifest_hash",
            &format!("{path}.manifest_hash"),
            "sha256:<lower-hex>",
        )?;
        let replay_result_hash = required_hash_field(
            entry_members,
            "replay_result_hash",
            &format!("{path}.replay_result_hash"),
            "sha256:<lower-hex>",
        )?;
        let comparison_status = parse_required_phase8_normalized_comparison_status_field(
            entry_members,
            "comparison_status",
            &format!("{path}.comparison_status"),
        )?;
        reject_unknown_fields(
            entry_members,
            CHALLENGE_COVERAGE_SUMMARY_ENTRY_FIELDS,
            &path,
        )?;
        entries.push(Phase8ChallengeCoverageEntry {
            challenge_id,
            manifest_hash,
            replay_result_hash,
            comparison_status,
        });
    }
    Ok(entries)
}

fn parse_phase8_challenge_imports_field(
    members: &[JsonMember<'_>],
    field: &str,
) -> Result<Phase8ChallengeImports, Phase8RequestValidationError> {
    let value = required_field_value(members, last_json_path_component(field), field, "object")?;
    let import_members = object_members_or_policy_error(value, field, "object")?;
    let mode = required_string_field(
        import_members,
        "mode",
        &format!("{field}.mode"),
        "locked_store",
    )?;
    if mode != "locked_store" {
        return Err(Phase8RequestValidationError::value_failure(
            format!("{field}.mode"),
            "locked_store",
            "invalid_enum",
        ));
    }
    let manifest = required_string_field(
        import_members,
        "manifest",
        &format!("{field}.manifest"),
        "workspace_relative_path",
    )?;
    if !phase8_valid_workspace_relative_path(&manifest) {
        return Err(Phase8RequestValidationError::value_failure(
            format!("{field}.manifest"),
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    let manifest_hash = required_hash_field(
        import_members,
        "manifest_hash",
        &format!("{field}.manifest_hash"),
        "sha256:<lower-hex>",
    )?;
    reject_unknown_fields(import_members, CHALLENGE_IMPORTS_FIELDS, field)?;
    Ok(Phase8ChallengeImports {
        mode,
        manifest,
        manifest_hash,
    })
}

fn parse_phase8_challenge_base_certificate_field(
    members: &[JsonMember<'_>],
    field: &str,
) -> Result<Phase8ChallengeBaseCertificate, Phase8RequestValidationError> {
    let value = required_field_value(members, last_json_path_component(field), field, "object")?;
    let certificate_members = object_members_or_policy_error(value, field, "object")?;
    let path = required_string_field(
        certificate_members,
        "path",
        &format!("{field}.path"),
        "workspace_relative_path",
    )?;
    if !phase8_valid_workspace_relative_path(&path) {
        return Err(Phase8RequestValidationError::value_failure(
            format!("{field}.path"),
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    let file_hash = required_hash_field(
        certificate_members,
        "file_hash",
        &format!("{field}.file_hash"),
        "sha256:<lower-hex>",
    )?;
    let claimed_certificate_hash = required_hash_field(
        certificate_members,
        "claimed_certificate_hash",
        &format!("{field}.claimed_certificate_hash"),
        "sha256:<lower-hex>",
    )?;
    reject_unknown_fields(
        certificate_members,
        CHALLENGE_BASE_CERTIFICATE_FIELDS,
        field,
    )?;
    Ok(Phase8ChallengeBaseCertificate {
        path,
        file_hash,
        claimed_certificate_hash,
    })
}

fn parse_phase8_challenge_mutated_certificate_field(
    members: &[JsonMember<'_>],
    field: &str,
) -> Result<Phase8ChallengeMutatedCertificate, Phase8RequestValidationError> {
    let value = required_field_value(members, last_json_path_component(field), field, "object")?;
    let certificate_members = object_members_or_policy_error(value, field, "object")?;
    let path = required_string_field(
        certificate_members,
        "path",
        &format!("{field}.path"),
        "workspace_relative_path",
    )?;
    if !phase8_valid_workspace_relative_path(&path) {
        return Err(Phase8RequestValidationError::value_failure(
            format!("{field}.path"),
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    let file_hash = required_hash_field(
        certificate_members,
        "file_hash",
        &format!("{field}.file_hash"),
        "sha256:<lower-hex>",
    )?;
    let claimed_certificate_hash = optional_hash_field(
        certificate_members,
        "claimed_certificate_hash",
        &format!("{field}.claimed_certificate_hash"),
        "sha256:<lower-hex>",
    )?;
    reject_unknown_fields(
        certificate_members,
        CHALLENGE_MUTATED_CERTIFICATE_FIELDS,
        field,
    )?;
    Ok(Phase8ChallengeMutatedCertificate {
        path,
        file_hash,
        claimed_certificate_hash,
    })
}

fn parse_phase8_challenge_mutation_field(
    members: &[JsonMember<'_>],
    field: &str,
    allow_informational: bool,
) -> Result<Phase8ChallengeMutation, Phase8RequestValidationError> {
    let value = required_field_value(members, last_json_path_component(field), field, "object")?;
    let mutation_members = object_members_or_policy_error(value, field, "object")?;
    let kind = required_string_field(
        mutation_members,
        "kind",
        &format!("{field}.kind"),
        "challenge_mutation_kind",
    )?;
    let known_kind = Phase8ChallengeMutationKind::parse(&kind);
    if known_kind.is_none() {
        if allow_informational {
            if !phase8_valid_informational_challenge_kind(&kind) {
                return Err(Phase8RequestValidationError::value_failure(
                    format!("{field}.kind"),
                    "challenge_mutation_kind",
                    "invalid_name_format",
                ));
            }
        } else {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{field}.kind"),
                "challenge_mutation_kind",
                "invalid_enum",
            ));
        }
    }
    let target = required_string_field(
        mutation_members,
        "target",
        &format!("{field}.target"),
        "challenge_mutation_target",
    )?;
    validate_challenge_mutation_target(&target, known_kind, allow_informational).map_err(
        |actual| {
            Phase8RequestValidationError::value_failure(
                format!("{field}.target"),
                "challenge_mutation_target",
                actual,
            )
        },
    )?;
    let seed = required_hash_field(
        mutation_members,
        "seed",
        &format!("{field}.seed"),
        "sha256:<lower-hex>",
    )?;
    reject_unknown_fields(mutation_members, CHALLENGE_MUTATION_FIELDS, field)?;
    Ok(Phase8ChallengeMutation { kind, target, seed })
}

fn parse_phase8_challenge_generation_output_field(
    members: &[JsonMember<'_>],
) -> Result<Phase8ChallengeGenerationOutput, Phase8RequestValidationError> {
    let value = required_field_value(members, "output", "output", "object")?;
    let output_members = object_members_or_policy_error(value, "output", "object")?;
    let store_manifest_path = required_string_field(
        output_members,
        "store_manifest_path",
        "output.store_manifest_path",
        "workspace_relative_path",
    )?;
    if !phase8_valid_workspace_relative_path(&store_manifest_path) {
        return Err(Phase8RequestValidationError::value_failure(
            "output.store_manifest_path",
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    let manifest_path = required_string_field(
        output_members,
        "manifest_path",
        "output.manifest_path",
        "workspace_relative_path",
    )?;
    if !phase8_valid_workspace_relative_path(&manifest_path) {
        return Err(Phase8RequestValidationError::value_failure(
            "output.manifest_path",
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    let mutated_certificate_path = required_string_field(
        output_members,
        "mutated_certificate_path",
        "output.mutated_certificate_path",
        "workspace_relative_path",
    )?;
    if !phase8_valid_workspace_relative_path(&mutated_certificate_path) {
        return Err(Phase8RequestValidationError::value_failure(
            "output.mutated_certificate_path",
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    reject_unknown_fields(output_members, CHALLENGE_GENERATION_OUTPUT_FIELDS, "output")?;
    Ok(Phase8ChallengeGenerationOutput {
        store_manifest_path,
        manifest_path,
        mutated_certificate_path,
    })
}

fn parse_phase8_challenge_generated_by_field(
    members: &[JsonMember<'_>],
    field: &str,
) -> Result<Phase8ChallengeGeneratedBy, Phase8RequestValidationError> {
    let value = required_field_value(members, last_json_path_component(field), field, "object")?;
    let generated_by_members = object_members_or_policy_error(value, field, "object")?;
    let kind_raw = required_string_field(
        generated_by_members,
        "kind",
        &format!("{field}.kind"),
        "ci_or_ai",
    )?;
    let kind = Phase8ChallengeGeneratedByKind::parse(&kind_raw).ok_or_else(|| {
        Phase8RequestValidationError::value_failure(
            format!("{field}.kind"),
            "ci_or_ai",
            "invalid_enum",
        )
    })?;
    let prompt_hash = optional_hash_field(
        generated_by_members,
        "prompt_hash",
        &format!("{field}.prompt_hash"),
        "sha256:<lower-hex>",
    )?;
    reject_unknown_fields(generated_by_members, CHALLENGE_GENERATED_BY_FIELDS, field)?;
    match (kind, prompt_hash) {
        (Phase8ChallengeGeneratedByKind::Ai, None) => {
            Err(Phase8RequestValidationError::value_failure(
                format!("{field}.prompt_hash"),
                "sha256:<lower-hex>",
                "missing",
            ))
        }
        (Phase8ChallengeGeneratedByKind::Ci, Some(_)) => {
            Err(Phase8RequestValidationError::value_failure(
                format!("{field}.prompt_hash"),
                "absent",
                "present",
            ))
        }
        _ => Ok(Phase8ChallengeGeneratedBy { kind, prompt_hash }),
    }
}

fn parse_phase8_challenge_outcome_hint_field(
    members: &[JsonMember<'_>],
    field: &str,
) -> Result<Phase8ChallengeOutcomeHint, Phase8RequestValidationError> {
    let value = required_field_value(members, last_json_path_component(field), field, "object")?;
    let hint_members = object_members_or_policy_error(value, field, "object")?;
    let status = required_string_field(
        hint_members,
        "status",
        &format!("{field}.status"),
        "outcome_hint_status",
    )?;
    if !phase8_visible_ascii_nonempty(&status) {
        return Err(Phase8RequestValidationError::value_failure(
            format!("{field}.status"),
            "outcome_hint_status",
            "invalid_name_format",
        ));
    }
    let error_kinds = optional_string_array_field(
        hint_members,
        "error_kinds",
        &format!("{field}.error_kinds"),
        "error_kind",
    )?;
    for (index, error_kind) in error_kinds.iter().enumerate() {
        if !phase8_visible_ascii_nonempty(error_kind) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{field}.error_kinds[{index}]"),
                "error_kind",
                "invalid_name_format",
            ));
        }
    }
    reject_unknown_fields(hint_members, CHALLENGE_OUTCOME_HINT_FIELDS, field)?;
    Ok(Phase8ChallengeOutcomeHint {
        status,
        error_kinds,
    })
}

fn parse_phase8_challenge_replay_field(
    members: &[JsonMember<'_>],
    field: &str,
) -> Result<Phase8ChallengeReplayMetadata, Phase8RequestValidationError> {
    let value = required_field_value(members, last_json_path_component(field), field, "object")?;
    let replay_members = object_members_or_policy_error(value, field, "object")?;
    let generator = required_string_field(
        replay_members,
        "generator",
        &format!("{field}.generator"),
        "generator",
    )?;
    if !phase8_printable_ascii_nonempty(&generator) {
        return Err(Phase8RequestValidationError::value_failure(
            format!("{field}.generator"),
            "generator",
            "invalid_string_format",
        ));
    }
    let generator_version = required_string_field(
        replay_members,
        "generator_version",
        &format!("{field}.generator_version"),
        "generator_version",
    )?;
    if !phase8_printable_ascii_nonempty(&generator_version) {
        return Err(Phase8RequestValidationError::value_failure(
            format!("{field}.generator_version"),
            "generator_version",
            "invalid_string_format",
        ));
    }
    let generator_build_hash = required_hash_field(
        replay_members,
        "generator_build_hash",
        &format!("{field}.generator_build_hash"),
        "sha256:<lower-hex>",
    )?;
    let args_hash = required_hash_field(
        replay_members,
        "args_hash",
        &format!("{field}.args_hash"),
        "sha256:<lower-hex>",
    )?;
    reject_unknown_fields(replay_members, CHALLENGE_REPLAY_FIELDS, field)?;
    Ok(Phase8ChallengeReplayMetadata {
        generator,
        generator_version,
        generator_build_hash,
        args_hash,
    })
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

fn parse_phase8_release_policy_value(
    value: &JsonValue<'_>,
) -> Result<Phase8ReleasePolicy, Phase8PolicyValidationError> {
    let members = object_members_or_policy_error(value, "$", "object")?;
    required_fixed_string_field(
        members,
        "schema",
        "schema",
        PHASE8_RELEASE_POLICY_SCHEMA,
        PHASE8_RELEASE_POLICY_SCHEMA,
    )?;
    let id = required_string_field(members, "id", "id", "phase8_policy_id")?;
    let version_raw = required_integer_raw_field(members, "version", "version", "positive_i64")?;
    let mode_raw = required_string_field(members, "mode", "mode", "release_policy_mode")?;
    let mode = Phase8ReleaseMode::parse(&mode_raw).ok_or_else(|| {
        Phase8PolicyValidationError::new("mode", "release_policy_mode", "invalid_enum")
    })?;
    let runner_policy_hash = required_hash_field(
        members,
        "runner_policy_hash",
        "runner_policy_hash",
        "sha256:<lower-hex>",
    )?;
    let challenge_runner_policy_hash = required_hash_field(
        members,
        "challenge_runner_policy_hash",
        "challenge_runner_policy_hash",
        "sha256:<lower-hex>",
    )?;
    let ai_triage_value = required_field_value(members, "ai_triage", "ai_triage", "object")?;
    let ai_triage_members = object_members_or_policy_error(ai_triage_value, "ai_triage", "object")?;
    let enabled =
        required_bool_field(ai_triage_members, "enabled", "ai_triage.enabled", "boolean")?;
    let required = required_bool_field(
        ai_triage_members,
        "required",
        "ai_triage.required",
        "boolean",
    )?;
    let input_policy_hash = optional_hash_field(
        ai_triage_members,
        "input_policy_hash",
        "ai_triage.input_policy_hash",
        "sha256:<lower-hex>",
    )?;
    reject_unknown_fields(
        ai_triage_members,
        RELEASE_POLICY_AI_TRIAGE_FIELDS,
        "ai_triage",
    )?;
    reject_unknown_fields(members, RELEASE_POLICY_FIELDS, "$")?;

    if !phase8_valid_runner_policy_id(&id) {
        return Err(Phase8PolicyValidationError::new(
            "id",
            "phase8_policy_id",
            if id.is_empty() {
                "empty_string"
            } else {
                "invalid_name_format"
            },
        ));
    }
    let version = parse_positive_i64_domain(&version_raw, "version")?;
    if enabled && input_policy_hash.is_none() {
        return Err(Phase8PolicyValidationError::new(
            "ai_triage.input_policy_hash",
            "sha256:<lower-hex>",
            "missing",
        ));
    }
    if !enabled && input_policy_hash.is_some() {
        return Err(Phase8PolicyValidationError::new(
            "ai_triage.input_policy_hash",
            "absent",
            "present",
        ));
    }
    if !enabled && required {
        return Err(Phase8PolicyValidationError::new(
            "ai_triage.required",
            "false_when_ai_triage_disabled",
            "true",
        ));
    }

    Ok(Phase8ReleasePolicy {
        id,
        version,
        mode,
        runner_policy_hash,
        challenge_runner_policy_hash,
        ai_triage: Phase8ReleasePolicyAiTriage {
            enabled,
            required,
            input_policy_hash,
        },
    })
}

fn parse_phase8_auxiliary_result_value(
    value: &JsonValue<'_>,
) -> Result<Phase8AuxiliaryResult, Phase8RequestValidationError> {
    let members = object_members_or_policy_error(value, "$", "object")?;
    required_fixed_string_field(
        members,
        "schema",
        "schema",
        PHASE8_AUXILIARY_RESULT_SCHEMA,
        PHASE8_AUXILIARY_RESULT_SCHEMA,
    )?;
    let kind_raw = required_string_field(members, "kind", "kind", "auxiliary_result_kind")?;
    let kind = Phase8AuxiliaryResultKind::parse(&kind_raw).ok_or_else(|| {
        Phase8RequestValidationError::value_failure("kind", "auxiliary_result_kind", "invalid_enum")
    })?;
    let result_id = required_string_field(members, "result_id", "result_id", "result_id")?;
    let parsed_result_hash =
        required_hash_field(members, "result_hash", "result_hash", "sha256:<lower-hex>")?;
    let policy_hash =
        required_hash_field(members, "policy_hash", "policy_hash", "sha256:<lower-hex>")?;
    let artifact_hash = required_hash_field(
        members,
        "artifact_hash",
        "artifact_hash",
        "sha256:<lower-hex>",
    )?;
    let selector = match unique_optional_field_value(members, "selector", "selector", "object")? {
        Some(selector_value) => Some(parse_phase8_auxiliary_selector_value(kind, selector_value)?),
        None => None,
    };
    let status_raw = required_string_field(members, "status", "status", "auxiliary_status")?;
    let status = Phase8AuxiliaryStatus::parse(&status_raw).ok_or_else(|| {
        Phase8RequestValidationError::value_failure("status", "auxiliary_status", "invalid_enum")
    })?;
    let error = match unique_optional_field_value(members, "error", "error", "object")? {
        Some(error_value) => Some(parse_phase8_auxiliary_error_value(error_value)?),
        None => None,
    };
    let diagnostics =
        optional_string_array_field(members, "diagnostics", "diagnostics", "diagnostic_token")?;
    reject_unknown_fields(members, AUXILIARY_RESULT_FIELDS, "$")?;

    let result = Phase8AuxiliaryResult {
        kind,
        result_id,
        policy_hash,
        artifact_hash,
        selector,
        status,
        error,
        diagnostics,
    };
    validate_phase8_auxiliary_result_envelope(&result)?;
    let recomputed = result.result_hash();
    if recomputed != parsed_result_hash {
        return Err(Phase8RequestValidationError::hash_failure(
            "result_hash",
            recomputed,
            parsed_result_hash,
        ));
    }
    Ok(result)
}

fn parse_phase8_auxiliary_selector_value(
    kind: Phase8AuxiliaryResultKind,
    value: &JsonValue<'_>,
) -> Result<Phase8AuxiliarySelector, Phase8RequestValidationError> {
    let members = object_members_or_policy_error(value, "selector", "object")?;
    match kind {
        Phase8AuxiliaryResultKind::AxiomPolicy => {
            let normalized_result_hash = required_hash_field(
                members,
                "normalized_result_hash",
                "selector.normalized_result_hash",
                "sha256:<lower-hex>",
            )?;
            let checker_profile = required_string_field(
                members,
                "checker_profile",
                "selector.checker_profile",
                "checker_profile_name",
            )?;
            if !phase8_valid_checker_profile_name(&checker_profile) {
                return Err(Phase8RequestValidationError::value_failure(
                    "selector.checker_profile",
                    "checker_profile_name",
                    "invalid_name_format",
                ));
            }
            let result_hash = required_hash_field(
                members,
                "result_hash",
                "selector.result_hash",
                "sha256:<lower-hex>",
            )?;
            let axiom_report_hash = required_hash_field(
                members,
                "axiom_report_hash",
                "selector.axiom_report_hash",
                "sha256:<lower-hex>",
            )?;
            reject_unknown_fields(members, AUXILIARY_AXIOM_SELECTOR_FIELDS, "selector")?;
            Ok(Phase8AuxiliarySelector::AxiomPolicy {
                normalized_result_hash,
                checker_profile,
                result_hash,
                axiom_report_hash,
            })
        }
        Phase8AuxiliaryResultKind::Reproducibility => {
            let request_hash = required_hash_field(
                members,
                "request_hash",
                "selector.request_hash",
                "sha256:<lower-hex>",
            )?;
            let checker_profile = required_string_field(
                members,
                "checker_profile",
                "selector.checker_profile",
                "checker_profile_name",
            )?;
            if !phase8_valid_checker_profile_name(&checker_profile) {
                return Err(Phase8RequestValidationError::value_failure(
                    "selector.checker_profile",
                    "checker_profile_name",
                    "invalid_name_format",
                ));
            }
            let baseline_run_artifact_hash = required_hash_field(
                members,
                "baseline_run_artifact_hash",
                "selector.baseline_run_artifact_hash",
                "sha256:<lower-hex>",
            )?;
            let repeated_run_artifact_hash = required_hash_field(
                members,
                "repeated_run_artifact_hash",
                "selector.repeated_run_artifact_hash",
                "sha256:<lower-hex>",
            )?;
            reject_unknown_fields(
                members,
                AUXILIARY_REPRODUCIBILITY_SELECTOR_FIELDS,
                "selector",
            )?;
            if baseline_run_artifact_hash == repeated_run_artifact_hash {
                return Err(Phase8RequestValidationError::value_failure(
                    "selector.repeated_run_artifact_hash",
                    "distinct_run_artifact_hash",
                    "duplicate",
                ));
            }
            Ok(Phase8AuxiliarySelector::Reproducibility {
                request_hash,
                checker_profile,
                baseline_run_artifact_hash,
                repeated_run_artifact_hash,
            })
        }
        Phase8AuxiliaryResultKind::ImportCertificateHash
        | Phase8AuxiliaryResultKind::AuditBundle => Err(
            Phase8RequestValidationError::value_failure("selector", "absent", "present"),
        ),
    }
}

fn parse_phase8_auxiliary_error_value(
    value: &JsonValue<'_>,
) -> Result<Phase8AuxiliaryError, Phase8RequestValidationError> {
    let members = object_members_or_policy_error(value, "error", "object")?;
    required_fixed_string_field(
        members,
        "kind",
        "error.kind",
        "auxiliary_failure",
        "auxiliary_failure",
    )?;
    let reason_raw = required_string_field(
        members,
        "reason_code",
        "error.reason_code",
        "auxiliary_reason_code",
    )?;
    let reason_code = Phase8AuxiliaryReasonCode::parse(&reason_raw).ok_or_else(|| {
        Phase8RequestValidationError::value_failure(
            "error.reason_code",
            "auxiliary_reason_code",
            "invalid_enum",
        )
    })?;
    let field = optional_string_field(members, "field", "error.field", "json_path")?;
    let expected_hash = optional_hash_field(
        members,
        "expected_hash",
        "error.expected_hash",
        "sha256:<lower-hex>",
    )?;
    let actual_hash = optional_hash_field(
        members,
        "actual_hash",
        "error.actual_hash",
        "sha256:<lower-hex>",
    )?;
    let expected_value =
        optional_string_field(members, "expected_value", "error.expected_value", "string")?;
    let actual_value =
        optional_string_field(members, "actual_value", "error.actual_value", "string")?;
    reject_unknown_fields(members, AUXILIARY_ERROR_FIELDS, "error")?;
    Ok(Phase8AuxiliaryError {
        reason_code,
        field,
        expected_hash,
        actual_hash,
        expected_value,
        actual_value,
    })
}

fn parse_phase8_auxiliary_result_store_manifest_value(
    value: &JsonValue<'_>,
) -> Result<Phase8AuxiliaryResultStoreManifest, Phase8RequestValidationError> {
    let members = object_members_or_policy_error(value, "auxiliary_result_store", "object")?;
    required_fixed_string_field(
        members,
        "schema",
        "auxiliary_result_store.schema",
        PHASE8_AUXILIARY_RESULT_STORE_MANIFEST_SCHEMA,
        PHASE8_AUXILIARY_RESULT_STORE_MANIFEST_SCHEMA,
    )?;
    let results_value = required_field_value(
        members,
        "results",
        "auxiliary_result_store.results",
        "array",
    )?;
    let Some(result_values) = results_value.array_elements() else {
        return Err(wrong_type_error(
            "auxiliary_result_store.results",
            "array",
            results_value.kind(),
        )
        .into());
    };
    let mut results = Vec::new();
    for (index, result_value) in result_values.iter().enumerate() {
        let path = format!("auxiliary_result_store.results[{index}]");
        let result_members = object_members_or_policy_error(result_value, &path, "object")?;
        let result_hash = required_hash_field(
            result_members,
            "result_hash",
            &format!("{path}.result_hash"),
            "sha256:<lower-hex>",
        )?;
        let kind_raw = required_string_field(
            result_members,
            "kind",
            &format!("{path}.kind"),
            "auxiliary_result_kind",
        )?;
        let kind = Phase8AuxiliaryResultKind::parse(&kind_raw).ok_or_else(|| {
            Phase8RequestValidationError::value_failure(
                format!("{path}.kind"),
                "auxiliary_result_kind",
                "invalid_enum",
            )
        })?;
        let policy_hash = required_hash_field(
            result_members,
            "policy_hash",
            &format!("{path}.policy_hash"),
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
        reject_unknown_fields(result_members, AUXILIARY_RESULT_STORE_ENTRY_FIELDS, &path)?;
        results.push(Phase8AuxiliaryResultStoreEntry {
            result_hash,
            kind,
            policy_hash,
            artifact_hash,
            path: result_path,
            file_hash,
        });
    }
    reject_unknown_fields(
        members,
        AUXILIARY_RESULT_STORE_MANIFEST_FIELDS,
        "auxiliary_result_store",
    )?;
    validate_auxiliary_result_store_domain(&results)?;
    Ok(Phase8AuxiliaryResultStoreManifest { results })
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
const RELEASE_POLICY_FIELDS: &[&str] = &[
    "schema",
    "id",
    "version",
    "mode",
    "runner_policy_hash",
    "challenge_runner_policy_hash",
    "ai_triage",
];
const RELEASE_POLICY_AI_TRIAGE_FIELDS: &[&str] = &["enabled", "required", "input_policy_hash"];
const RELEASE_BUNDLE_STAGING_PLAN_FIELDS: &[&str] = &["schema", "phase", "bundle_root", "inputs"];
const RELEASE_BUNDLE_STAGING_INPUT_FIELDS: &[&str] = &["kind", "path", "file_hash", "hashes"];
const AUXILIARY_RESULT_FIELDS: &[&str] = &[
    "schema",
    "kind",
    "result_id",
    "result_hash",
    "policy_hash",
    "artifact_hash",
    "selector",
    "status",
    "error",
    "diagnostics",
];
const AUXILIARY_AXIOM_SELECTOR_FIELDS: &[&str] = &[
    "normalized_result_hash",
    "checker_profile",
    "result_hash",
    "axiom_report_hash",
];
const AUXILIARY_REPRODUCIBILITY_SELECTOR_FIELDS: &[&str] = &[
    "request_hash",
    "checker_profile",
    "baseline_run_artifact_hash",
    "repeated_run_artifact_hash",
];
const AUXILIARY_ERROR_FIELDS: &[&str] = &[
    "kind",
    "reason_code",
    "field",
    "expected_hash",
    "actual_hash",
    "expected_value",
    "actual_value",
];
const AUXILIARY_RESULT_STORE_MANIFEST_FIELDS: &[&str] = &["schema", "results"];
const AUXILIARY_RESULT_STORE_ENTRY_FIELDS: &[&str] = &[
    "result_hash",
    "kind",
    "policy_hash",
    "artifact_hash",
    "path",
    "file_hash",
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
const MACHINE_CHECK_RESULT_FIELDS: &[&str] = &[
    "schema",
    "request_id",
    "request_hash",
    "result_id",
    "policy",
    "runner",
    "checker",
    "attempt",
    "status",
    "module",
    "process",
    "resource_usage",
    "error",
    "certificate_hash",
    "export_hash",
    "axiom_report_hash",
    "diagnostics",
    "axioms_used",
    "declarations_checked",
    "result_hash",
    "run_artifact_hash",
];
const MACHINE_CHECK_RESULT_CHECKER_FIELDS: &[&str] = &[
    "profile",
    "binary_id",
    "binary_hash",
    "id",
    "build_hash",
    "version",
];
const NORMALIZED_CHECK_RESULT_FIELDS: &[&str] = &[
    "schema",
    "normalized_result_id",
    "normalized_result_hash",
    "artifact",
    "artifact_hash",
    "policy",
    "results",
    "comparison",
];
const NORMALIZED_CHECK_RESULT_ENTRY_FIELDS: &[&str] = &[
    "result_id",
    "result_hash",
    "request_hash",
    "policy_hash",
    "artifact_hash",
    "checker_profile",
    "process_launched",
    "status",
    "checker_binary_id",
    "checker_binary_hash",
    "checker_id",
    "checker_build_hash",
    "certificate_hash",
    "export_hash",
    "axiom_report_hash",
    "error",
    "failure_key",
];
const COMPARE_VALIDATION_RESULT_FIELDS: &[&str] = &[
    "schema",
    "status",
    "normalized_result_hash",
    "policy_hash",
    "embedded_comparison_status",
    "recomputed_comparison_status",
    "error",
];
const AUDIT_SIDECAR_VALIDATION_RESULT_FIELDS: &[&str] = &[
    "schema",
    "mode",
    "status",
    "sidecar_file_hash",
    "input_policy_hash",
    "source_kind",
    "source_result_hash",
    "source_normalized_result_hash",
    "error",
];
const CHALLENGE_GENERATION_REQUEST_FIELDS: &[&str] = &[
    "schema",
    "request_id",
    "request_hash",
    "challenge_id",
    "policy_hash",
    "module",
    "imports",
    "base_certificate",
    "mutation",
    "output",
    "generated_by",
];
const CHALLENGE_MANIFEST_FIELDS: &[&str] = &[
    "schema",
    "challenge_id",
    "policy_hash",
    "module",
    "imports",
    "base_certificate",
    "mutated_certificate",
    "mutation",
    "outcome_hint",
    "replay",
    "generated_by",
];
const CHALLENGE_IMPORTS_FIELDS: &[&str] = &["mode", "manifest", "manifest_hash"];
const CHALLENGE_BASE_CERTIFICATE_FIELDS: &[&str] =
    &["path", "file_hash", "claimed_certificate_hash"];
const CHALLENGE_MUTATED_CERTIFICATE_FIELDS: &[&str] =
    &["path", "file_hash", "claimed_certificate_hash"];
const CHALLENGE_MUTATION_FIELDS: &[&str] = &["kind", "target", "seed"];
const CHALLENGE_GENERATION_OUTPUT_FIELDS: &[&str] = &[
    "store_manifest_path",
    "manifest_path",
    "mutated_certificate_path",
];
const CHALLENGE_GENERATED_BY_FIELDS: &[&str] = &["kind", "prompt_hash"];
const CHALLENGE_OUTCOME_HINT_FIELDS: &[&str] = &["status", "error_kinds"];
const CHALLENGE_REPLAY_FIELDS: &[&str] = &[
    "generator",
    "generator_version",
    "generator_build_hash",
    "args_hash",
];
const CHALLENGE_OUTPUT_STORE_MANIFEST_FIELDS: &[&str] = &["schema", "entries"];
const CHALLENGE_OUTPUT_STORE_ENTRY_FIELDS: &[&str] =
    &["challenge_id", "manifest_path", "manifest_hash"];
const CHALLENGE_REPLAY_RESULT_FIELDS: &[&str] = &[
    "schema",
    "result_id",
    "result_hash",
    "challenge_id",
    "manifest_hash",
    "mutated_file_hash",
    "mutated_claimed_certificate_hash",
    "checker_results",
    "missing_checker_profiles",
    "normalized_result_hash",
    "policy_hash",
    "artifact_hash",
    "comparison_status",
    "observed_error_kinds",
];
const CHALLENGE_REPLAY_CHECKER_RESULT_FIELDS: &[&str] = &[
    "result_id",
    "result_hash",
    "run_artifact_hash",
    "checker_profile",
];
const CHALLENGE_REPLAY_STORE_MANIFEST_FIELDS: &[&str] = &["schema", "results"];
const CHALLENGE_REPLAY_STORE_ENTRY_FIELDS: &[&str] = &[
    "challenge_id",
    "manifest_hash",
    "result_hash",
    "artifact_hash",
    "path",
    "file_hash",
];
const CHALLENGE_COVERAGE_SUMMARY_FIELDS: &[&str] = &[
    "schema",
    "summary_id",
    "summary_hash",
    "policy_hash",
    "artifact_hash",
    "target_normalized_result_hash",
    "challenge_store_manifest_hash",
    "result_store_manifest_hash",
    "total_challenges",
    "replayed_challenges",
    "unexpected_acceptances",
    "entries",
];
const CHALLENGE_COVERAGE_SUMMARY_ENTRY_FIELDS: &[&str] = &[
    "challenge_id",
    "manifest_hash",
    "replay_result_hash",
    "comparison_status",
];
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct Phase8ReleaseBundleMachineResultSummary {
    result_hash: Hash,
    request_hash: Hash,
    run_artifact_hash: Hash,
    checker_profile: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Phase8ReleaseBundleNormalizedResultSummary {
    normalized_result_hash: Hash,
    artifact_hash: Hash,
}

fn parse_phase8_release_bundle_staging_plan_value(
    value: &JsonValue<'_>,
) -> Result<Phase8ReleaseBundleStagingPlan, Phase8CommandError> {
    let members = phase8_release_stage_object_members(value, "plan", "object")?;
    phase8_release_stage_required_fixed_string(
        members,
        "schema",
        "schema",
        PHASE8_RELEASE_BUNDLE_STAGING_PLAN_SCHEMA,
        PHASE8_RELEASE_BUNDLE_STAGING_PLAN_SCHEMA,
    )?;
    let phase_raw = phase8_release_stage_required_string(
        members,
        "phase",
        "phase",
        "release_bundle_staging_phase",
    )?;
    let phase = Phase8ReleaseBundleStagingPhase::parse(&phase_raw).ok_or_else(|| {
        phase8_release_stage_value_error(
            "input_schema_invalid",
            "phase",
            "store|final",
            "invalid_enum",
        )
    })?;
    let bundle_root = phase8_release_stage_required_string(
        members,
        "bundle_root",
        "bundle_root",
        "workspace_relative_path",
    )?;
    if !phase8_valid_workspace_relative_path(&bundle_root) {
        return Err(phase8_release_stage_value_error(
            "input_schema_invalid",
            "bundle_root",
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    let inputs_value = phase8_release_stage_required_value(members, "inputs", "inputs", "array")?;
    let Some(input_values) = inputs_value.array_elements() else {
        return Err(phase8_release_stage_value_error(
            "input_schema_invalid",
            "inputs",
            "array",
            "wrong_type",
        ));
    };
    let mut inputs = Vec::new();
    for (index, input_value) in input_values.iter().enumerate() {
        inputs.push(parse_phase8_release_bundle_staging_input_value(
            input_value,
            index,
        )?);
    }
    phase8_release_stage_reject_unknown_fields(members, RELEASE_BUNDLE_STAGING_PLAN_FIELDS, "$")?;

    phase8_release_validate_staging_plan_inputs(phase, &inputs)?;
    Ok(Phase8ReleaseBundleStagingPlan {
        phase,
        bundle_root,
        inputs,
    })
}

fn parse_phase8_release_bundle_staging_input_value(
    value: &JsonValue<'_>,
    index: usize,
) -> Result<Phase8ReleaseBundleStagingInput, Phase8CommandError> {
    let input_path = format!("inputs[{index}]");
    let members = phase8_release_stage_object_members(value, &input_path, "object")?;
    let kind_raw = phase8_release_stage_required_string(
        members,
        "kind",
        format!("{input_path}.kind"),
        "release_bundle_artifact_kind",
    )?;
    let kind = Phase8ReleaseBundleArtifactKind::parse(&kind_raw).ok_or_else(|| {
        phase8_release_stage_value_error(
            "input_schema_invalid",
            format!("{input_path}.kind"),
            "release_bundle_artifact_kind",
            "invalid_enum",
        )
    })?;
    let path = phase8_release_stage_required_string(
        members,
        "path",
        format!("{input_path}.path"),
        "workspace_relative_path",
    )?;
    if !phase8_valid_workspace_relative_path(&path) {
        return Err(phase8_release_stage_value_error(
            "input_schema_invalid",
            format!("{input_path}.path"),
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    let file_hash = phase8_release_stage_required_hash(
        members,
        "file_hash",
        format!("{input_path}.file_hash"),
        "sha256:<lower-hex>",
    )?;
    let hashes_value = phase8_release_stage_required_value(
        members,
        "hashes",
        format!("{input_path}.hashes"),
        "object",
    )?;
    let hashes = parse_phase8_release_bundle_input_hashes(hashes_value, &input_path, kind)?;
    phase8_release_stage_reject_unknown_fields(
        members,
        RELEASE_BUNDLE_STAGING_INPUT_FIELDS,
        &input_path,
    )?;

    Ok(Phase8ReleaseBundleStagingInput {
        kind,
        path,
        file_hash,
        hashes,
    })
}

fn parse_phase8_release_bundle_input_hashes(
    value: &JsonValue<'_>,
    input_path: &str,
    kind: Phase8ReleaseBundleArtifactKind,
) -> Result<BTreeMap<String, Hash>, Phase8CommandError> {
    let hashes_path = format!("{input_path}.hashes");
    let members = phase8_release_stage_object_members(value, &hashes_path, "object")?;
    let required = kind.required_hash_fields();
    if required.is_empty() && !members.is_empty() {
        return Err(phase8_release_stage_value_error(
            "input_schema_invalid",
            &hashes_path,
            "empty_object",
            "non_empty",
        ));
    }
    let mut hashes = BTreeMap::new();
    for member in members {
        if hashes.contains_key(member.key()) {
            return Err(phase8_release_stage_value_error(
                "input_schema_invalid",
                format!("{hashes_path}.{}", member.key()),
                "unique_object_keys",
                "duplicate_field",
            ));
        }
        let Some(raw) = member.value().string_value() else {
            return Err(phase8_release_stage_value_error(
                "input_schema_invalid",
                format!("{hashes_path}.{}", member.key()),
                "sha256:<lower-hex>",
                if member.value().kind() == JsonValueKind::Null {
                    "null_not_allowed"
                } else {
                    "wrong_type"
                },
            ));
        };
        let hash = parse_hash_string(raw).map_err(|_| {
            phase8_release_stage_value_error(
                "input_schema_invalid",
                format!("{hashes_path}.{}", member.key()),
                "sha256:<lower-hex>",
                "invalid_hash_format",
            )
        })?;
        hashes.insert(member.key().to_owned(), hash);
    }
    for required_field in required {
        if !hashes.contains_key(*required_field) {
            return Err(phase8_release_stage_value_error(
                "input_schema_invalid",
                format!("{hashes_path}.{required_field}"),
                "sha256:<lower-hex>",
                "missing",
            ));
        }
    }
    for key in hashes.keys() {
        if !required.contains(&key.as_str()) {
            return Err(phase8_release_stage_value_error(
                "input_schema_invalid",
                format!("{hashes_path}.{key}"),
                format!("hash_field_for_kind:{}", kind.as_str()),
                "unknown_field",
            ));
        }
    }
    Ok(hashes)
}

fn phase8_release_validate_staging_plan_inputs(
    phase: Phase8ReleaseBundleStagingPhase,
    inputs: &[Phase8ReleaseBundleStagingInput],
) -> Result<(), Phase8CommandError> {
    for index in 1..inputs.len() {
        let left = (
            inputs[index].kind.as_str(),
            inputs[index].path.as_str(),
            inputs[index].file_hash,
        );
        let right = (
            inputs[index - 1].kind.as_str(),
            inputs[index - 1].path.as_str(),
            inputs[index - 1].file_hash,
        );
        if left < right {
            return Err(phase8_release_stage_value_error(
                "input_schema_invalid",
                format!("inputs[{index}]"),
                "inputs_sorted_by_kind_path_file_hash",
                "order_violation",
            ));
        }
    }

    let mut kind_paths = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for (index, input) in inputs.iter().enumerate() {
        if !kind_paths.insert((input.kind, input.path.as_str())) {
            return Err(phase8_release_stage_value_error(
                "input_schema_invalid",
                format!("inputs[{index}]"),
                "unique_kind_path",
                "duplicate_entry",
            ));
        }
        if !paths.insert(input.path.as_str()) {
            return Err(phase8_release_stage_value_error(
                "input_schema_invalid",
                format!("inputs[{index}].path"),
                "unique_path",
                "duplicate_path",
            ));
        }
    }

    if phase == Phase8ReleaseBundleStagingPhase::Store {
        phase8_release_require_store_input_kind(
            inputs,
            Phase8ReleaseBundleArtifactKind::RequestStoreManifest,
            "one_or_more",
        )?;
        phase8_release_require_store_input_kind(
            inputs,
            Phase8ReleaseBundleArtifactKind::MachineResultStoreManifest,
            "one_or_more",
        )?;
        phase8_release_require_store_input_kind(
            inputs,
            Phase8ReleaseBundleArtifactKind::NormalizedResultStoreManifest,
            "one_or_more",
        )?;
        let challenge_count = inputs
            .iter()
            .filter(|input| {
                input.kind == Phase8ReleaseBundleArtifactKind::ChallengeOutputStoreManifest
            })
            .count();
        if challenge_count != 1 {
            return Err(phase8_release_stage_value_error(
                "input_schema_invalid",
                "inputs[]",
                "exactly_one:challenge_output_store_manifest",
                if challenge_count == 0 {
                    "missing:challenge_output_store_manifest".to_owned()
                } else {
                    format!("count:{challenge_count}")
                },
            ));
        }
    }
    for (index, input) in inputs.iter().enumerate() {
        if !input.kind.allowed_for_phase(phase) {
            return Err(phase8_release_stage_value_error(
                "input_schema_invalid",
                format!("inputs[{index}].kind"),
                format!("allowed_kind_for_phase:{}", phase.as_str()),
                input.kind.as_str(),
            ));
        }
    }
    Ok(())
}

fn phase8_release_require_store_input_kind(
    inputs: &[Phase8ReleaseBundleStagingInput],
    kind: Phase8ReleaseBundleArtifactKind,
    expected_prefix: &str,
) -> Result<(), Phase8CommandError> {
    if inputs.iter().any(|input| input.kind == kind) {
        return Ok(());
    }
    Err(phase8_release_stage_value_error(
        "input_schema_invalid",
        "inputs[]",
        format!("{expected_prefix}:{}", kind.as_str()),
        format!("missing:{}", kind.as_str()),
    ))
}

fn phase8_release_validate_direct_artifact(
    input: &Phase8ReleaseBundleStagingInput,
    index: usize,
    source: &str,
) -> Result<(), Phase8CommandError> {
    let direct_json_reason = if matches!(
        input.kind,
        Phase8ReleaseBundleArtifactKind::RequestStoreManifest
            | Phase8ReleaseBundleArtifactKind::MachineResultStoreManifest
            | Phase8ReleaseBundleArtifactKind::NormalizedResultStoreManifest
            | Phase8ReleaseBundleArtifactKind::ChallengeOutputStoreManifest
    ) {
        "input_store_manifest_invalid"
    } else {
        "input_json_invalid"
    };
    JsonDocument::parse(source).map_err(|_| {
        phase8_release_stage_value_error(
            direct_json_reason,
            format!("inputs[{index}].path"),
            "valid_json",
            "invalid_json",
        )
    })?;
    match input.kind {
        Phase8ReleaseBundleArtifactKind::ReleasePolicy => {
            let policy = parse_phase8_release_policy(source).map_err(|error| {
                phase8_release_stage_artifact_policy_error(
                    "input_schema_invalid",
                    index,
                    input.kind,
                    error,
                )
            })?;
            phase8_release_validate_planned_hash(input, index, "policy_hash", policy.policy_hash())
        }
        Phase8ReleaseBundleArtifactKind::RunnerPolicy => {
            let policy = parse_phase8_runner_policy(source).map_err(|error| {
                phase8_release_stage_artifact_policy_error(
                    "input_schema_invalid",
                    index,
                    input.kind,
                    error,
                )
            })?;
            phase8_release_validate_planned_hash(input, index, "policy_hash", policy.policy_hash())
        }
        Phase8ReleaseBundleArtifactKind::CheckerIdentityManifest => {
            parse_phase8_checker_identity_manifest(source).map_err(|error| {
                phase8_release_stage_artifact_policy_error(
                    "input_schema_invalid",
                    index,
                    input.kind,
                    error,
                )
            })?;
            phase8_release_validate_planned_hash(input, index, "manifest_hash", input.file_hash)
        }
        Phase8ReleaseBundleArtifactKind::ImportLock => {
            parse_phase8_import_lock_manifest(source).map_err(|error| {
                phase8_release_stage_artifact_request_error(
                    "input_schema_invalid",
                    index,
                    input.kind,
                    error,
                )
            })?;
            phase8_release_validate_planned_hash(input, index, "manifest_hash", input.file_hash)
        }
        Phase8ReleaseBundleArtifactKind::MachineCheckRequest => {
            let request = parse_phase8_machine_check_request(source).map_err(|error| {
                phase8_release_stage_artifact_request_error(
                    "input_schema_invalid",
                    index,
                    input.kind,
                    error,
                )
            })?;
            phase8_release_validate_planned_hash(
                input,
                index,
                "request_hash",
                request.request_hash(),
            )
        }
        Phase8ReleaseBundleArtifactKind::MachineCheckResult => {
            let summary =
                parse_phase8_machine_check_result_summary(source, "$").map_err(|error| {
                    phase8_release_stage_artifact_request_error(
                        "input_schema_invalid",
                        index,
                        input.kind,
                        error,
                    )
                })?;
            phase8_release_validate_planned_hash(input, index, "result_hash", summary.result_hash)?;
            phase8_release_validate_planned_hash(
                input,
                index,
                "run_artifact_hash",
                summary.run_artifact_hash,
            )
        }
        Phase8ReleaseBundleArtifactKind::NormalizedCheckResult => {
            let summary =
                parse_phase8_normalized_check_result_summary(source, "$").map_err(|error| {
                    phase8_release_stage_artifact_request_error(
                        "input_schema_invalid",
                        index,
                        input.kind,
                        error,
                    )
                })?;
            phase8_release_validate_planned_hash(
                input,
                index,
                "artifact_hash",
                summary.artifact_hash,
            )?;
            phase8_release_validate_planned_hash(
                input,
                index,
                "normalized_result_hash",
                summary.normalized_result_hash,
            )
        }
        Phase8ReleaseBundleArtifactKind::RequestStoreManifest => {
            parse_phase8_request_store_manifest(source).map_err(|error| {
                phase8_release_stage_artifact_request_error(
                    "input_store_manifest_invalid",
                    index,
                    input.kind,
                    error,
                )
            })?;
            phase8_release_validate_planned_hash(input, index, "manifest_hash", input.file_hash)
        }
        Phase8ReleaseBundleArtifactKind::MachineResultStoreManifest => {
            parse_phase8_machine_result_store_manifest(source).map_err(|error| {
                phase8_release_stage_artifact_request_error(
                    "input_store_manifest_invalid",
                    index,
                    input.kind,
                    error,
                )
            })?;
            phase8_release_validate_planned_hash(input, index, "manifest_hash", input.file_hash)
        }
        Phase8ReleaseBundleArtifactKind::NormalizedResultStoreManifest => {
            parse_phase8_normalized_result_store_manifest(source).map_err(|error| {
                phase8_release_stage_artifact_request_error(
                    "input_store_manifest_invalid",
                    index,
                    input.kind,
                    error,
                )
            })?;
            phase8_release_validate_planned_hash(input, index, "manifest_hash", input.file_hash)
        }
        Phase8ReleaseBundleArtifactKind::ChallengeManifest => {
            parse_phase8_challenge_manifest(source).map_err(|error| {
                phase8_release_stage_artifact_request_error(
                    "input_schema_invalid",
                    index,
                    input.kind,
                    error,
                )
            })?;
            phase8_release_validate_planned_hash(input, index, "manifest_hash", input.file_hash)
        }
        Phase8ReleaseBundleArtifactKind::ChallengeOutputStoreManifest => {
            parse_phase8_challenge_output_store_manifest(source).map_err(|error| {
                phase8_release_stage_artifact_request_error(
                    "input_store_manifest_invalid",
                    index,
                    input.kind,
                    error,
                )
            })?;
            phase8_release_validate_planned_hash(input, index, "manifest_hash", input.file_hash)
        }
        Phase8ReleaseBundleArtifactKind::ChallengeReplayResult => {
            let result = parse_phase8_challenge_replay_result(source).map_err(|error| {
                phase8_release_stage_artifact_request_error(
                    "input_schema_invalid",
                    index,
                    input.kind,
                    error,
                )
            })?;
            phase8_release_validate_planned_hash(input, index, "result_hash", result.result_hash())
        }
        Phase8ReleaseBundleArtifactKind::ChallengeCoverageSummary => {
            let summary = parse_phase8_challenge_coverage_summary(source).map_err(|error| {
                phase8_release_stage_artifact_request_error(
                    "input_schema_invalid",
                    index,
                    input.kind,
                    error,
                )
            })?;
            phase8_release_validate_planned_hash(
                input,
                index,
                "summary_hash",
                summary.summary_hash(),
            )
        }
        Phase8ReleaseBundleArtifactKind::AuxiliaryResult => {
            let result = parse_phase8_auxiliary_result(source).map_err(|error| {
                phase8_release_stage_artifact_request_error(
                    "input_schema_invalid",
                    index,
                    input.kind,
                    error,
                )
            })?;
            phase8_release_validate_planned_hash(input, index, "result_hash", result.result_hash())
        }
        Phase8ReleaseBundleArtifactKind::AiAuditInputPolicy => {
            let policy = parse_phase8_ai_audit_input_policy(source).map_err(|error| {
                phase8_release_stage_artifact_audit_error(
                    "input_schema_invalid",
                    index,
                    input.kind,
                    error,
                )
            })?;
            phase8_release_validate_planned_hash(
                input,
                index,
                "input_policy_hash",
                policy.input_policy_hash(),
            )
        }
        Phase8ReleaseBundleArtifactKind::AiAuditSidecar => {
            parse_phase8_ai_audit_sidecar(source).map_err(|error| {
                phase8_release_stage_artifact_audit_error(
                    "input_schema_invalid",
                    index,
                    input.kind,
                    error,
                )
            })?;
            Ok(())
        }
        Phase8ReleaseBundleArtifactKind::CompareValidationResponse => {
            validate_phase8_release_schema_only_artifact(
                source,
                PHASE8_COMPARE_VALIDATION_RESULT_SCHEMA,
                COMPARE_VALIDATION_RESULT_FIELDS,
                &["valid", "failed"],
                format!("inputs[{index}].artifact"),
            )
        }
        Phase8ReleaseBundleArtifactKind::AuditSidecarValidationResponse => {
            validate_phase8_release_schema_only_artifact(
                source,
                PHASE8_AUDIT_SIDECAR_VALIDATION_RESULT_SCHEMA,
                AUDIT_SIDECAR_VALIDATION_RESULT_FIELDS,
                &["valid", "failed"],
                format!("inputs[{index}].artifact"),
            )
        }
    }
}

fn phase8_release_validate_planned_hash(
    input: &Phase8ReleaseBundleStagingInput,
    index: usize,
    field: &str,
    actual: Hash,
) -> Result<(), Phase8CommandError> {
    let expected = *input
        .hashes
        .get(field)
        .expect("plan hash shape validates required fields");
    if expected != actual {
        return Err(phase8_release_stage_hash_error(
            "input_hash_mismatch",
            format!("inputs[{index}].hashes.{field}"),
            expected,
            actual,
        ));
    }
    Ok(())
}

fn validate_phase8_release_schema_only_artifact(
    source: &str,
    schema: &str,
    allowed_fields: &[&str],
    valid_statuses: &[&str],
    field: String,
) -> Result<(), Phase8CommandError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        phase8_release_stage_value_error("input_json_invalid", &field, "valid_json", "invalid_json")
    })?;
    let members = phase8_release_stage_object_members(document.root(), &field, "object")?;
    phase8_release_stage_required_fixed_string(
        members,
        "schema",
        format!("{field}.schema"),
        schema,
        schema,
    )?;
    let status = phase8_release_stage_required_string(
        members,
        "status",
        format!("{field}.status"),
        "status",
    )?;
    if !valid_statuses.contains(&status.as_str()) {
        return Err(phase8_release_stage_value_error(
            "input_schema_invalid",
            format!("{field}.status"),
            "status",
            "invalid_enum",
        ));
    }
    phase8_release_stage_reject_unknown_fields(members, allowed_fields, &field)
}

fn phase8_release_stage_request_manifest_entries(
    input_index: usize,
    manifest: &Phase8RequestStoreManifest,
    workspace_files: &BTreeMap<String, Vec<u8>>,
    existing_bundle_files: Option<&BTreeMap<String, Vec<u8>>>,
    staged_files: &mut BTreeMap<String, Vec<u8>>,
    staged_artifacts: &mut Vec<Phase8ReleaseBundleStagedArtifact>,
    merged_entries: &mut BTreeMap<Hash, Phase8RequestStoreEntry>,
) -> Result<(), Phase8CommandError> {
    for (index, entry) in manifest.requests.iter().enumerate() {
        let entry_path = format!("inputs[{input_index}].store.request_store.requests[{index}]");
        let source_bytes = phase8_release_store_entry_bytes(
            workspace_files,
            &entry.path,
            format!("{entry_path}.path"),
        )?;
        let actual_file_hash = phase8_file_hash(source_bytes);
        if actual_file_hash != entry.file_hash {
            return Err(phase8_release_stage_hash_error(
                "input_store_entry_invalid",
                format!("{entry_path}.file_hash"),
                entry.file_hash,
                actual_file_hash,
            ));
        }
        let source = phase8_release_stage_utf8(
            source_bytes,
            "input_store_entry_invalid",
            format!("{entry_path}.path"),
        )?;
        let request = parse_phase8_machine_check_request(source).map_err(|error| {
            phase8_release_stage_request_error(
                "input_store_entry_invalid",
                format!("{entry_path}.artifact"),
                error,
            )
        })?;
        if request.request_hash() != entry.request_hash {
            return Err(phase8_release_stage_hash_error(
                "input_store_entry_invalid",
                format!("{entry_path}.request_hash"),
                entry.request_hash,
                request.request_hash(),
            ));
        }
        let staged_path = phase8_release_stage_file(
            Phase8ReleaseBundleArtifactKind::MachineCheckRequest,
            entry.file_hash,
            source_bytes,
            existing_bundle_files,
            staged_files,
            staged_artifacts,
        )?;
        let staged_entry = Phase8RequestStoreEntry {
            request_hash: entry.request_hash,
            path: staged_path,
            file_hash: entry.file_hash,
        };
        phase8_release_merge_request_entry(merged_entries, staged_entry)?;
    }
    Ok(())
}

fn phase8_release_stage_machine_manifest_entries(
    input_index: usize,
    manifest: &Phase8MachineResultStoreManifest,
    workspace_files: &BTreeMap<String, Vec<u8>>,
    existing_bundle_files: Option<&BTreeMap<String, Vec<u8>>>,
    staged_files: &mut BTreeMap<String, Vec<u8>>,
    staged_artifacts: &mut Vec<Phase8ReleaseBundleStagedArtifact>,
    merged_entries: &mut BTreeMap<Hash, Phase8MachineResultStoreEntry>,
) -> Result<(), Phase8CommandError> {
    for (index, entry) in manifest.results.iter().enumerate() {
        let entry_path =
            format!("inputs[{input_index}].store.machine_result_store.results[{index}]");
        let source_bytes = phase8_release_store_entry_bytes(
            workspace_files,
            &entry.path,
            format!("{entry_path}.path"),
        )?;
        let actual_file_hash = phase8_file_hash(source_bytes);
        if actual_file_hash != entry.file_hash {
            return Err(phase8_release_stage_hash_error(
                "input_store_entry_invalid",
                format!("{entry_path}.file_hash"),
                entry.file_hash,
                actual_file_hash,
            ));
        }
        let source = phase8_release_stage_utf8(
            source_bytes,
            "input_store_entry_invalid",
            format!("{entry_path}.path"),
        )?;
        let summary = parse_phase8_machine_check_result_summary(source, "$").map_err(|error| {
            phase8_release_stage_request_error(
                "input_store_entry_invalid",
                format!("{entry_path}.artifact"),
                error,
            )
        })?;
        if summary.result_hash != entry.result_hash {
            return Err(phase8_release_stage_hash_error(
                "input_store_entry_invalid",
                format!("{entry_path}.result_hash"),
                entry.result_hash,
                summary.result_hash,
            ));
        }
        if summary.request_hash != entry.request_hash {
            return Err(phase8_release_stage_hash_error(
                "input_store_entry_invalid",
                format!("{entry_path}.request_hash"),
                entry.request_hash,
                summary.request_hash,
            ));
        }
        if summary.run_artifact_hash != entry.run_artifact_hash {
            return Err(phase8_release_stage_hash_error(
                "input_store_entry_invalid",
                format!("{entry_path}.run_artifact_hash"),
                entry.run_artifact_hash,
                summary.run_artifact_hash,
            ));
        }
        if summary.checker_profile != entry.checker_profile {
            return Err(phase8_release_stage_value_error(
                "input_store_entry_invalid",
                format!("{entry_path}.checker_profile"),
                &entry.checker_profile,
                &summary.checker_profile,
            ));
        }
        let staged_path = phase8_release_stage_file(
            Phase8ReleaseBundleArtifactKind::MachineCheckResult,
            entry.file_hash,
            source_bytes,
            existing_bundle_files,
            staged_files,
            staged_artifacts,
        )?;
        let staged_entry = Phase8MachineResultStoreEntry {
            result_hash: entry.result_hash,
            request_hash: entry.request_hash,
            run_artifact_hash: entry.run_artifact_hash,
            checker_profile: entry.checker_profile.clone(),
            path: staged_path,
            file_hash: entry.file_hash,
        };
        phase8_release_merge_machine_entry(merged_entries, staged_entry)?;
    }
    Ok(())
}

fn phase8_release_stage_normalized_manifest_entries(
    input_index: usize,
    manifest: &Phase8NormalizedResultStoreManifest,
    workspace_files: &BTreeMap<String, Vec<u8>>,
    existing_bundle_files: Option<&BTreeMap<String, Vec<u8>>>,
    staged_files: &mut BTreeMap<String, Vec<u8>>,
    staged_artifacts: &mut Vec<Phase8ReleaseBundleStagedArtifact>,
    merged_entries: &mut BTreeMap<Hash, Phase8NormalizedResultStoreEntry>,
) -> Result<(), Phase8CommandError> {
    for (index, entry) in manifest.results.iter().enumerate() {
        let entry_path =
            format!("inputs[{input_index}].store.normalized_result_store.results[{index}]");
        let source_bytes = phase8_release_store_entry_bytes(
            workspace_files,
            &entry.path,
            format!("{entry_path}.path"),
        )?;
        let actual_file_hash = phase8_file_hash(source_bytes);
        if actual_file_hash != entry.file_hash {
            return Err(phase8_release_stage_hash_error(
                "input_store_entry_invalid",
                format!("{entry_path}.file_hash"),
                entry.file_hash,
                actual_file_hash,
            ));
        }
        let source = phase8_release_stage_utf8(
            source_bytes,
            "input_store_entry_invalid",
            format!("{entry_path}.path"),
        )?;
        let summary =
            parse_phase8_normalized_check_result_summary(source, "$").map_err(|error| {
                phase8_release_stage_request_error(
                    "input_store_entry_invalid",
                    format!("{entry_path}.artifact"),
                    error,
                )
            })?;
        if summary.normalized_result_hash != entry.normalized_result_hash {
            return Err(phase8_release_stage_hash_error(
                "input_store_entry_invalid",
                format!("{entry_path}.normalized_result_hash"),
                entry.normalized_result_hash,
                summary.normalized_result_hash,
            ));
        }
        if summary.artifact_hash != entry.artifact_hash {
            return Err(phase8_release_stage_hash_error(
                "input_store_entry_invalid",
                format!("{entry_path}.artifact_hash"),
                entry.artifact_hash,
                summary.artifact_hash,
            ));
        }
        let staged_path = phase8_release_stage_file(
            Phase8ReleaseBundleArtifactKind::NormalizedCheckResult,
            entry.file_hash,
            source_bytes,
            existing_bundle_files,
            staged_files,
            staged_artifacts,
        )?;
        let staged_entry = Phase8NormalizedResultStoreEntry {
            normalized_result_hash: entry.normalized_result_hash,
            artifact_hash: entry.artifact_hash,
            path: staged_path,
            file_hash: entry.file_hash,
        };
        phase8_release_merge_normalized_entry(merged_entries, staged_entry)?;
    }
    Ok(())
}

fn phase8_release_store_entry_bytes<'a>(
    workspace_files: &'a BTreeMap<String, Vec<u8>>,
    path: &str,
    field: String,
) -> Result<&'a [u8], Phase8CommandError> {
    workspace_files.get(path).map(Vec::as_slice).ok_or_else(|| {
        phase8_release_stage_value_error(
            "input_store_entry_invalid",
            field,
            "readable_file",
            "missing",
        )
    })
}

fn phase8_release_stage_file(
    kind: Phase8ReleaseBundleArtifactKind,
    file_hash: Hash,
    bytes: &[u8],
    existing_bundle_files: Option<&BTreeMap<String, Vec<u8>>>,
    staged_files: &mut BTreeMap<String, Vec<u8>>,
    staged_artifacts: &mut Vec<Phase8ReleaseBundleStagedArtifact>,
) -> Result<String, Phase8CommandError> {
    let path = phase8_release_bundle_artifact_path(kind, file_hash);
    phase8_release_insert_staged_file(&path, bytes, existing_bundle_files, staged_files)?;
    if !staged_artifacts.iter().any(|artifact| {
        artifact.kind == kind && artifact.path == path && artifact.file_hash == file_hash
    }) {
        staged_artifacts.push(Phase8ReleaseBundleStagedArtifact {
            kind,
            path: path.clone(),
            file_hash,
        });
    }
    Ok(path)
}

fn phase8_release_stage_generated_store_manifest(
    kind: Phase8ReleaseBundleArtifactKind,
    canonical_json: String,
    existing_bundle_files: Option<&BTreeMap<String, Vec<u8>>>,
    staged_files: &mut BTreeMap<String, Vec<u8>>,
    store_manifests: &mut Vec<Phase8ReleaseBundleStagedStoreManifest>,
) -> Result<(), Phase8CommandError> {
    let bytes = canonical_json.into_bytes();
    let manifest_hash = phase8_file_hash(&bytes);
    let path = phase8_release_bundle_artifact_path(kind, manifest_hash);
    phase8_release_insert_staged_file(&path, &bytes, existing_bundle_files, staged_files)?;
    store_manifests.push(Phase8ReleaseBundleStagedStoreManifest {
        kind,
        path,
        manifest_hash,
    });
    Ok(())
}

fn phase8_release_insert_staged_file(
    path: &str,
    bytes: &[u8],
    existing_bundle_files: Option<&BTreeMap<String, Vec<u8>>>,
    staged_files: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), Phase8CommandError> {
    if let Some(existing) = existing_bundle_files.and_then(|files| files.get(path)) {
        if existing.as_slice() != bytes {
            return Err(phase8_release_stage_value_error(
                "output_path_conflict",
                path,
                "absent_or_exact_match",
                "different_bytes",
            ));
        }
    }
    if let Some(existing) = staged_files.get(path) {
        if existing.as_slice() != bytes {
            return Err(phase8_release_stage_value_error(
                "output_path_conflict",
                path,
                "unique_staged_bytes",
                "different_bytes",
            ));
        }
        return Ok(());
    }
    staged_files.insert(path.to_owned(), bytes.to_vec());
    Ok(())
}

fn phase8_release_bundle_artifact_path(
    kind: Phase8ReleaseBundleArtifactKind,
    file_hash: Hash,
) -> String {
    let formatted = format_hash_string(&file_hash);
    let hash_hex = formatted
        .strip_prefix("sha256:")
        .expect("hash formatter emits sha256 prefix");
    format!("artifacts/{}/{hash_hex}.json", kind.as_str())
}

fn phase8_release_merge_request_entry(
    entries: &mut BTreeMap<Hash, Phase8RequestStoreEntry>,
    entry: Phase8RequestStoreEntry,
) -> Result<(), Phase8CommandError> {
    match entries.get(&entry.request_hash) {
        Some(existing) if existing.path == entry.path && existing.file_hash == entry.file_hash => {
            Ok(())
        }
        Some(existing) => Err(phase8_release_stage_value_error(
            "release_bundle_generation_failed",
            "request_store.requests[]",
            entry.canonical_json(),
            existing.canonical_json(),
        )),
        None => {
            entries.insert(entry.request_hash, entry);
            Ok(())
        }
    }
}

fn phase8_release_merge_machine_entry(
    entries: &mut BTreeMap<Hash, Phase8MachineResultStoreEntry>,
    entry: Phase8MachineResultStoreEntry,
) -> Result<(), Phase8CommandError> {
    match entries.get(&entry.run_artifact_hash) {
        Some(existing)
            if existing.result_hash == entry.result_hash
                && existing.request_hash == entry.request_hash
                && existing.checker_profile == entry.checker_profile
                && existing.path == entry.path
                && existing.file_hash == entry.file_hash =>
        {
            Ok(())
        }
        Some(existing) => Err(phase8_release_stage_value_error(
            "release_bundle_generation_failed",
            "result_store.results[]",
            entry.canonical_json(),
            existing.canonical_json(),
        )),
        None => {
            entries.insert(entry.run_artifact_hash, entry);
            Ok(())
        }
    }
}

fn phase8_release_merge_normalized_entry(
    entries: &mut BTreeMap<Hash, Phase8NormalizedResultStoreEntry>,
    entry: Phase8NormalizedResultStoreEntry,
) -> Result<(), Phase8CommandError> {
    match entries.get(&entry.normalized_result_hash) {
        Some(existing)
            if existing.artifact_hash == entry.artifact_hash
                && existing.path == entry.path
                && existing.file_hash == entry.file_hash =>
        {
            Ok(())
        }
        Some(existing) => Err(phase8_release_stage_value_error(
            "release_bundle_generation_failed",
            "normalized_store.results[]",
            entry.canonical_json(),
            existing.canonical_json(),
        )),
        None => {
            entries.insert(entry.normalized_result_hash, entry);
            Ok(())
        }
    }
}

fn parse_phase8_machine_check_result_summary(
    source: &str,
    root_path: &str,
) -> Result<Phase8ReleaseBundleMachineResultSummary, Phase8RequestValidationError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        Phase8RequestValidationError::value_failure(root_path, "valid_json", "invalid_json")
    })?;
    let members = object_members_or_policy_error(document.root(), root_path, "object")?;
    required_fixed_string_field(
        members,
        "schema",
        &phase8_join_json_path(root_path, "schema"),
        PHASE8_MACHINE_CHECK_RESULT_SCHEMA,
        PHASE8_MACHINE_CHECK_RESULT_SCHEMA,
    )?;
    let parsed_result_hash = required_hash_field(
        members,
        "result_hash",
        &phase8_join_json_path(root_path, "result_hash"),
        "sha256:<lower-hex>",
    )?;
    let parsed_run_artifact_hash = required_hash_field(
        members,
        "run_artifact_hash",
        &phase8_join_json_path(root_path, "run_artifact_hash"),
        "sha256:<lower-hex>",
    )?;
    let request_hash = required_hash_field(
        members,
        "request_hash",
        &phase8_join_json_path(root_path, "request_hash"),
        "sha256:<lower-hex>",
    )?;
    let status = required_string_field(
        members,
        "status",
        &phase8_join_json_path(root_path, "status"),
        "MachineCheckResult.status",
    )?;
    if !matches!(status.as_str(), "checked" | "failed") {
        return Err(Phase8RequestValidationError::value_failure(
            phase8_join_json_path(root_path, "status"),
            "MachineCheckResult.status",
            "invalid_enum",
        ));
    }
    let checker_value = required_field_value(
        members,
        "checker",
        &phase8_join_json_path(root_path, "checker"),
        "object",
    )?;
    let checker_members = object_members_or_policy_error(
        checker_value,
        &phase8_join_json_path(root_path, "checker"),
        "object",
    )?;
    let checker_profile = required_string_field(
        checker_members,
        "profile",
        &phase8_join_json_path(root_path, "checker.profile"),
        "checker_profile_name",
    )?;
    if !phase8_valid_checker_profile_name(&checker_profile) {
        return Err(Phase8RequestValidationError::value_failure(
            phase8_join_json_path(root_path, "checker.profile"),
            "checker_profile_name",
            "invalid_name_format",
        ));
    }
    reject_unknown_fields(
        checker_members,
        MACHINE_CHECK_RESULT_CHECKER_FIELDS,
        &phase8_join_json_path(root_path, "checker"),
    )?;
    reject_unknown_fields(members, MACHINE_CHECK_RESULT_FIELDS, root_path)?;

    let result_hash = phase8_machine_result_projection_hash(document.root()).map_err(|error| {
        Phase8RequestValidationError::value_failure(
            phase8_canonical_error_path(&error),
            "canonical_json",
            "invalid_canonical_json",
        )
    })?;
    if result_hash != parsed_result_hash {
        return Err(Phase8RequestValidationError::hash_failure(
            phase8_join_json_path(root_path, "result_hash"),
            result_hash,
            parsed_result_hash,
        ));
    }
    let run_artifact_hash = phase8_hash_json_object_excluding_top_level_fields(
        source,
        RUN_ARTIFACT_HASH_EXCLUDED_FIELDS,
    )
    .map_err(|error| {
        Phase8RequestValidationError::value_failure(
            phase8_canonical_error_path(&error),
            "canonical_json",
            "invalid_canonical_json",
        )
    })?;
    if run_artifact_hash != parsed_run_artifact_hash {
        return Err(Phase8RequestValidationError::hash_failure(
            phase8_join_json_path(root_path, "run_artifact_hash"),
            run_artifact_hash,
            parsed_run_artifact_hash,
        ));
    }
    Ok(Phase8ReleaseBundleMachineResultSummary {
        result_hash,
        request_hash,
        run_artifact_hash,
        checker_profile,
    })
}

fn parse_phase8_normalized_check_result_summary(
    source: &str,
    root_path: &str,
) -> Result<Phase8ReleaseBundleNormalizedResultSummary, Phase8RequestValidationError> {
    let document = JsonDocument::parse(source).map_err(|_| {
        Phase8RequestValidationError::value_failure(root_path, "valid_json", "invalid_json")
    })?;
    let members = object_members_or_policy_error(document.root(), root_path, "object")?;
    required_fixed_string_field(
        members,
        "schema",
        &phase8_join_json_path(root_path, "schema"),
        PHASE8_NORMALIZED_CHECK_RESULT_SCHEMA,
        PHASE8_NORMALIZED_CHECK_RESULT_SCHEMA,
    )?;
    let parsed_artifact_hash = required_hash_field(
        members,
        "artifact_hash",
        &phase8_join_json_path(root_path, "artifact_hash"),
        "sha256:<lower-hex>",
    )?;
    let parsed_normalized_result_hash = required_hash_field(
        members,
        "normalized_result_hash",
        &phase8_join_json_path(root_path, "normalized_result_hash"),
        "sha256:<lower-hex>",
    )?;
    let artifact_value = required_field_value(
        members,
        "artifact",
        &phase8_join_json_path(root_path, "artifact"),
        "object",
    )?;
    object_members_or_policy_error(
        artifact_value,
        &phase8_join_json_path(root_path, "artifact"),
        "object",
    )?;
    let results_value = required_field_value(
        members,
        "results",
        &phase8_join_json_path(root_path, "results"),
        "array",
    )?;
    let Some(results) = results_value.array_elements() else {
        return Err(Phase8RequestValidationError::value_failure(
            phase8_join_json_path(root_path, "results"),
            "array",
            "wrong_type",
        ));
    };
    for (index, result_value) in results.iter().enumerate() {
        let entry_path = format!("{}.results[{index}]", root_path.trim_end_matches('.'));
        let entry_members = object_members_or_policy_error(result_value, &entry_path, "object")?;
        reject_unknown_fields(
            entry_members,
            NORMALIZED_CHECK_RESULT_ENTRY_FIELDS,
            &entry_path,
        )?;
    }
    required_field_value(
        members,
        "policy",
        &phase8_join_json_path(root_path, "policy"),
        "object",
    )?;
    required_field_value(
        members,
        "comparison",
        &phase8_join_json_path(root_path, "comparison"),
        "object",
    )?;
    reject_unknown_fields(members, NORMALIZED_CHECK_RESULT_FIELDS, root_path)?;

    let artifact_hash = phase8_canonical_json_value_hash(artifact_value).map_err(|error| {
        Phase8RequestValidationError::value_failure(
            phase8_canonical_error_path(&error),
            "canonical_json",
            "invalid_canonical_json",
        )
    })?;
    if artifact_hash != parsed_artifact_hash {
        return Err(Phase8RequestValidationError::hash_failure(
            phase8_join_json_path(root_path, "artifact_hash"),
            artifact_hash,
            parsed_artifact_hash,
        ));
    }
    let normalized_result_hash = phase8_normalized_result_projection_hash(document.root())
        .map_err(|error| {
            Phase8RequestValidationError::value_failure(
                phase8_canonical_error_path(&error),
                "canonical_json",
                "invalid_canonical_json",
            )
        })?;
    if normalized_result_hash != parsed_normalized_result_hash {
        return Err(Phase8RequestValidationError::hash_failure(
            phase8_join_json_path(root_path, "normalized_result_hash"),
            normalized_result_hash,
            parsed_normalized_result_hash,
        ));
    }
    Ok(Phase8ReleaseBundleNormalizedResultSummary {
        normalized_result_hash,
        artifact_hash,
    })
}

fn phase8_machine_result_projection_hash(
    value: &JsonValue<'_>,
) -> Result<Hash, Phase8CanonicalJsonError> {
    let members =
        value
            .object_members()
            .ok_or_else(|| Phase8CanonicalJsonError::ExpectedObject {
                path: Phase8JsonPath::root(),
                actual: value.kind(),
            })?;
    let checker_value = phase8_required_member_value_unchecked(members, "checker");
    let checker_json =
        phase8_canonical_json_object_excluding_value_fields(checker_value, &["version"])?;
    let mut pairs = vec![
        ("checker".to_owned(), checker_json),
        (
            "module".to_owned(),
            phase8_required_member_canonical_json(members, "module")?,
        ),
        (
            "policy".to_owned(),
            phase8_required_member_canonical_json(members, "policy")?,
        ),
        (
            "runner".to_owned(),
            phase8_required_member_canonical_json(members, "runner")?,
        ),
        (
            "schema".to_owned(),
            phase8_json_string_literal(PHASE8_MACHINE_CHECK_RESULT_SCHEMA),
        ),
        (
            "status".to_owned(),
            phase8_required_member_canonical_json(members, "status")?,
        ),
    ];
    for optional in [
        "error",
        "certificate_hash",
        "export_hash",
        "axiom_report_hash",
    ] {
        if let Some(value) = phase8_optional_member_value(members, optional) {
            pairs.push((
                optional.to_owned(),
                phase8_canonical_json_value_to_string(value)?,
            ));
        }
    }
    Ok(phase8_sha256(
        canonical_json_object_from_pairs(pairs).as_bytes(),
    ))
}

fn phase8_normalized_result_projection_hash(
    value: &JsonValue<'_>,
) -> Result<Hash, Phase8CanonicalJsonError> {
    let members =
        value
            .object_members()
            .ok_or_else(|| Phase8CanonicalJsonError::ExpectedObject {
                path: Phase8JsonPath::root(),
                actual: value.kind(),
            })?;
    let results_value = phase8_required_member_value_unchecked(members, "results");
    let result_values =
        results_value
            .array_elements()
            .ok_or_else(|| Phase8CanonicalJsonError::ExpectedObject {
                path: Phase8JsonPath::root().field("results"),
                actual: results_value.kind(),
            })?;
    let mut result_json = Vec::new();
    for result_value in result_values {
        result_json.push(phase8_canonical_json_object_excluding_value_fields(
            result_value,
            &["result_id"],
        )?);
    }
    let artifact_hash = phase8_canonical_json_value_hash(phase8_required_member_value_unchecked(
        members, "artifact",
    ))?;
    let pairs = vec![
        (
            "artifact".to_owned(),
            phase8_required_member_canonical_json(members, "artifact")?,
        ),
        (
            "artifact_hash".to_owned(),
            phase8_hash_json_literal(&artifact_hash),
        ),
        (
            "comparison".to_owned(),
            phase8_required_member_canonical_json(members, "comparison")?,
        ),
        (
            "policy".to_owned(),
            phase8_required_member_canonical_json(members, "policy")?,
        ),
        ("results".to_owned(), canonical_json_array(result_json)),
        (
            "schema".to_owned(),
            phase8_json_string_literal(PHASE8_NORMALIZED_CHECK_RESULT_SCHEMA),
        ),
    ];
    Ok(phase8_sha256(
        canonical_json_object_from_pairs(pairs).as_bytes(),
    ))
}

fn phase8_required_member_value_unchecked<'value, 'src>(
    members: &'value [JsonMember<'src>],
    name: &str,
) -> &'value JsonValue<'src> {
    members
        .iter()
        .find(|member| member.key() == name)
        .map(JsonMember::value)
        .expect("schema validation checked required member")
}

fn phase8_optional_member_value<'value, 'src>(
    members: &'value [JsonMember<'src>],
    name: &str,
) -> Option<&'value JsonValue<'src>> {
    members
        .iter()
        .find(|member| member.key() == name)
        .map(JsonMember::value)
}

fn phase8_required_member_canonical_json(
    members: &[JsonMember<'_>],
    name: &str,
) -> Result<String, Phase8CanonicalJsonError> {
    phase8_canonical_json_value_to_string(phase8_required_member_value_unchecked(members, name))
}

fn phase8_canonical_json_value_to_string(
    value: &JsonValue<'_>,
) -> Result<String, Phase8CanonicalJsonError> {
    let bytes = phase8_canonical_json_value_bytes(value)?;
    Ok(String::from_utf8(bytes).expect("canonical JSON is UTF-8"))
}

fn phase8_canonical_json_value_hash(
    value: &JsonValue<'_>,
) -> Result<Hash, Phase8CanonicalJsonError> {
    Ok(phase8_sha256(&phase8_canonical_json_value_bytes(value)?))
}

fn phase8_canonical_json_object_excluding_value_fields(
    value: &JsonValue<'_>,
    excluded_fields: &[&str],
) -> Result<String, Phase8CanonicalJsonError> {
    let members =
        value
            .object_members()
            .ok_or_else(|| Phase8CanonicalJsonError::ExpectedObject {
                path: Phase8JsonPath::root(),
                actual: value.kind(),
            })?;
    let excluded = excluded_fields.iter().copied().collect::<BTreeSet<_>>();
    let mut out = Vec::new();
    write_phase8_canonical_json_object_members(
        members,
        &Phase8JsonPath::root(),
        &mut out,
        Some(&excluded),
    )?;
    Ok(String::from_utf8(out).expect("canonical JSON is UTF-8"))
}

fn phase8_canonical_error_path(error: &Phase8CanonicalJsonError) -> String {
    match error {
        Phase8CanonicalJsonError::JsonParse { .. } => "$".to_owned(),
        Phase8CanonicalJsonError::ExpectedObject { path, .. }
        | Phase8CanonicalJsonError::DuplicateObjectKey { path, .. }
        | Phase8CanonicalJsonError::FloatNumber { path, .. }
        | Phase8CanonicalJsonError::NegativeZeroInteger { path }
        | Phase8CanonicalJsonError::InvalidInteger { path, .. } => path.artifact_local_string(),
    }
}

fn phase8_hashes_canonical_json(hashes: &BTreeMap<String, Hash>) -> String {
    canonical_json_object_from_pairs(
        hashes
            .iter()
            .map(|(field, hash)| (field.clone(), phase8_hash_json_literal(hash)))
            .collect(),
    )
}

fn phase8_release_stage_utf8<'a>(
    bytes: &'a [u8],
    reason_code: &str,
    field: impl Into<String>,
) -> Result<&'a str, Phase8CommandError> {
    std::str::from_utf8(bytes).map_err(|_| {
        phase8_release_stage_value_error(reason_code, field, "valid_json", "invalid_json")
    })
}

fn phase8_release_stage_object_members<'value, 'src>(
    value: &'value JsonValue<'src>,
    field: impl Into<String>,
    expected: impl Into<String>,
) -> Result<&'value [JsonMember<'src>], Phase8CommandError> {
    let field = field.into();
    let expected = expected.into();
    value.object_members().ok_or_else(|| {
        phase8_release_stage_value_error(
            "input_schema_invalid",
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

fn phase8_release_stage_required_value<'value, 'src>(
    members: &'value [JsonMember<'src>],
    name: &str,
    field: impl Into<String>,
    expected: impl Into<String>,
) -> Result<&'value JsonValue<'src>, Phase8CommandError> {
    let field = field.into();
    let expected = expected.into();
    if duplicate_member(members, name) {
        return Err(phase8_release_stage_value_error(
            "input_schema_invalid",
            field,
            "unique_object_keys",
            "duplicate_field",
        ));
    }
    let Some(member) = members.iter().find(|member| member.key() == name) else {
        return Err(phase8_release_stage_value_error(
            "input_schema_invalid",
            field,
            expected,
            "missing",
        ));
    };
    if member.value().kind() == JsonValueKind::Null {
        return Err(phase8_release_stage_value_error(
            "input_schema_invalid",
            field,
            expected,
            "null_not_allowed",
        ));
    }
    Ok(member.value())
}

fn phase8_release_stage_required_string(
    members: &[JsonMember<'_>],
    name: &str,
    field: impl Into<String>,
    expected: impl Into<String>,
) -> Result<String, Phase8CommandError> {
    let field = field.into();
    let expected = expected.into();
    let value =
        phase8_release_stage_required_value(members, name, field.clone(), expected.clone())?;
    value.string_value().map(ToOwned::to_owned).ok_or_else(|| {
        phase8_release_stage_value_error(
            "input_schema_invalid",
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

fn phase8_release_stage_required_fixed_string(
    members: &[JsonMember<'_>],
    name: &str,
    field: impl Into<String>,
    expected: impl Into<String>,
    fixed: &str,
) -> Result<(), Phase8CommandError> {
    let field = field.into();
    let expected = expected.into();
    let actual = phase8_release_stage_required_string(members, name, &field, &expected)?;
    if actual != fixed {
        return Err(phase8_release_stage_value_error(
            "input_schema_invalid",
            field,
            expected,
            actual,
        ));
    }
    Ok(())
}

fn phase8_release_stage_required_hash(
    members: &[JsonMember<'_>],
    name: &str,
    field: impl Into<String>,
    expected: impl Into<String>,
) -> Result<Hash, Phase8CommandError> {
    let field = field.into();
    let expected = expected.into();
    let raw = phase8_release_stage_required_string(members, name, field.clone(), expected.clone())?;
    parse_hash_string(&raw).map_err(|_| {
        phase8_release_stage_value_error(
            "input_schema_invalid",
            field,
            expected,
            "invalid_hash_format",
        )
    })
}

fn phase8_release_stage_reject_unknown_fields(
    members: &[JsonMember<'_>],
    allowed: &[&str],
    container_path: &str,
) -> Result<(), Phase8CommandError> {
    let mut counts = BTreeMap::<String, usize>::new();
    for member in members {
        *counts.entry(member.key().to_owned()).or_default() += 1;
    }
    let mut duplicates = counts
        .iter()
        .filter_map(|(field, count)| (*count > 1).then_some(field.as_str()))
        .collect::<Vec<_>>();
    duplicates.sort_by(|left, right| phase8_rfc8785_object_key_cmp(left, right));
    if let Some(field) = duplicates.first() {
        return Err(phase8_release_stage_value_error(
            "input_schema_invalid",
            phase8_join_json_path(container_path, field),
            "unique_object_keys",
            "duplicate_field",
        ));
    }
    let mut unknown = counts
        .keys()
        .filter(|field| !allowed.contains(&field.as_str()))
        .map(String::as_str)
        .collect::<Vec<_>>();
    unknown.sort_by(|left, right| phase8_rfc8785_object_key_cmp(left, right));
    if let Some(field) = unknown.first() {
        return Err(phase8_release_stage_value_error(
            "input_schema_invalid",
            phase8_join_json_path(container_path, field),
            "absent",
            "unknown_field",
        ));
    }
    Ok(())
}

fn phase8_release_stage_value_error(
    reason_code: impl Into<String>,
    field: impl Into<String>,
    expected_value: impl Into<String>,
    actual_value: impl Into<String>,
) -> Phase8CommandError {
    phase8_command_value_error(
        Phase8CommandName::ReleaseStageBundleInputs,
        reason_code,
        field,
        expected_value,
        actual_value,
    )
}

fn phase8_release_stage_hash_error(
    reason_code: impl Into<String>,
    field: impl Into<String>,
    expected_hash: Hash,
    actual_hash: Hash,
) -> Phase8CommandError {
    phase8_command_hash_error(
        Phase8CommandName::ReleaseStageBundleInputs,
        reason_code,
        field,
        expected_hash,
        actual_hash,
    )
}

fn phase8_release_stage_request_error(
    reason_code: &str,
    field_prefix: impl Into<String>,
    error: Phase8RequestValidationError,
) -> Phase8CommandError {
    let mut command_error = Phase8CommandError::new(
        Phase8CommandName::ReleaseStageBundleInputs,
        reason_code.to_owned(),
    );
    command_error.field =
        Some(phase8_prefixed_json_field(field_prefix.into(), &error.field).into_boxed_str());
    command_error.expected_hash = error.expected_hash;
    command_error.actual_hash = error.actual_hash;
    command_error.expected_value = error.expected_value;
    command_error.actual_value = error.actual_value;
    command_error
}

fn phase8_release_stage_store_manifest_error(
    input_index: usize,
    kind: Phase8ReleaseBundleArtifactKind,
    error: Phase8RequestValidationError,
) -> Phase8CommandError {
    let field = if error.expected_value.as_deref() == Some("valid_json")
        && error.actual_value.as_deref() == Some("invalid_json")
    {
        format!("inputs[{input_index}].path")
    } else {
        phase8_release_stage_store_manifest_field(input_index, kind, &error.field)
    };
    let mut command_error = Phase8CommandError::new(
        Phase8CommandName::ReleaseStageBundleInputs,
        "input_store_manifest_invalid",
    );
    command_error.field = Some(field.into_boxed_str());
    command_error.expected_hash = error.expected_hash;
    command_error.actual_hash = error.actual_hash;
    command_error.expected_value = error.expected_value;
    command_error.actual_value = error.actual_value;
    command_error
}

fn phase8_release_stage_store_manifest_field(
    input_index: usize,
    kind: Phase8ReleaseBundleArtifactKind,
    field: &str,
) -> String {
    let (source_root, target_root) = match kind {
        Phase8ReleaseBundleArtifactKind::RequestStoreManifest => ("request_store", "request_store"),
        Phase8ReleaseBundleArtifactKind::MachineResultStoreManifest => {
            ("result_store", "machine_result_store")
        }
        Phase8ReleaseBundleArtifactKind::NormalizedResultStoreManifest => {
            ("normalized_store", "normalized_result_store")
        }
        _ => return phase8_prefixed_json_field(format!("inputs[{input_index}].store"), field),
    };
    let prefix = format!("inputs[{input_index}].store.{target_root}");
    if field == "$" || field == source_root {
        return prefix;
    }
    if let Some(suffix) = field
        .strip_prefix(source_root)
        .and_then(|rest| rest.strip_prefix('.'))
    {
        return format!("{prefix}.{suffix}");
    }
    phase8_prefixed_json_field(format!("inputs[{input_index}].store"), field)
}

fn phase8_release_stage_artifact_request_error(
    reason_code: &str,
    input_index: usize,
    kind: Phase8ReleaseBundleArtifactKind,
    error: Phase8RequestValidationError,
) -> Phase8CommandError {
    let mut command_error = Phase8CommandError::new(
        Phase8CommandName::ReleaseStageBundleInputs,
        reason_code.to_owned(),
    );
    command_error.field =
        Some(phase8_release_stage_artifact_field(input_index, kind, &error.field).into_boxed_str());
    command_error.expected_hash = error.expected_hash;
    command_error.actual_hash = error.actual_hash;
    command_error.expected_value = error.expected_value;
    command_error.actual_value = error.actual_value;
    command_error
}

fn phase8_release_stage_artifact_policy_error(
    reason_code: &str,
    input_index: usize,
    kind: Phase8ReleaseBundleArtifactKind,
    error: Phase8PolicyValidationError,
) -> Phase8CommandError {
    phase8_release_stage_value_error(
        reason_code,
        phase8_release_stage_artifact_field(input_index, kind, &error.field),
        error.expected_value,
        error.actual_value,
    )
}

fn phase8_release_stage_artifact_audit_error(
    reason_code: &str,
    input_index: usize,
    kind: Phase8ReleaseBundleArtifactKind,
    error: Phase8AuditSidecarValidationError,
) -> Phase8CommandError {
    let mut command_error =
        Phase8CommandError::new(Phase8CommandName::ReleaseStageBundleInputs, reason_code);
    command_error.field =
        Some(phase8_release_stage_artifact_field(input_index, kind, &error.field).into_boxed_str());
    command_error.expected_hash = error.expected_hash;
    command_error.actual_hash = error.actual_hash;
    command_error.expected_value = error.expected_value;
    command_error.actual_value = error.actual_value;
    command_error
}

fn phase8_release_stage_artifact_field(
    input_index: usize,
    kind: Phase8ReleaseBundleArtifactKind,
    field: &str,
) -> String {
    let prefix = format!("inputs[{input_index}].artifact");
    for root in phase8_release_stage_artifact_virtual_roots(kind) {
        if field == *root || field == format!("{root}.$") {
            return prefix;
        }
        if let Some(suffix) = field
            .strip_prefix(*root)
            .and_then(|rest| rest.strip_prefix('.'))
        {
            return format!("{prefix}.{suffix}");
        }
    }
    phase8_prefixed_json_field(prefix, field)
}

fn phase8_release_stage_artifact_virtual_roots(
    kind: Phase8ReleaseBundleArtifactKind,
) -> &'static [&'static str] {
    match kind {
        Phase8ReleaseBundleArtifactKind::ReleasePolicy => &["release_policy"],
        Phase8ReleaseBundleArtifactKind::AiAuditInputPolicy => &["input_policy"],
        Phase8ReleaseBundleArtifactKind::CheckerIdentityManifest => &["checker_identity_manifest"],
        Phase8ReleaseBundleArtifactKind::ImportLock => &["imports.manifest"],
        Phase8ReleaseBundleArtifactKind::ChallengeManifest => &["challenge_manifest"],
        Phase8ReleaseBundleArtifactKind::ChallengeOutputStoreManifest => {
            &["challenge_output_store"]
        }
        Phase8ReleaseBundleArtifactKind::RequestStoreManifest => &["request_store"],
        Phase8ReleaseBundleArtifactKind::MachineResultStoreManifest => &["result_store"],
        Phase8ReleaseBundleArtifactKind::NormalizedResultStoreManifest => &["normalized_store"],
        _ => &[],
    }
}

fn phase8_prefixed_json_field(prefix: String, field: &str) -> String {
    if field == "$" {
        return prefix;
    }
    if field.starts_with('[') {
        format!("{prefix}{field}")
    } else {
        format!("{prefix}.{field}")
    }
}

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
        "object",
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

fn validate_challenge_output_store_domain(
    entries: &[Phase8ChallengeOutputStoreEntry],
    root_path: &str,
) -> Result<(), Phase8RequestValidationError> {
    for index in 1..entries.len() {
        if entries[index].challenge_id < entries[index - 1].challenge_id {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{root_path}.entries[{index}]"),
                "challenge_id_bytewise_ascending",
                "order_violation",
            ));
        }
    }

    let mut challenge_ids = BTreeSet::new();
    for (index, entry) in entries.iter().enumerate() {
        if !challenge_ids.insert(&entry.challenge_id) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{root_path}.entries[{index}].challenge_id"),
                "unique_challenge_ids",
                "duplicate_challenge_id",
            ));
        }
    }

    let mut manifest_paths = BTreeSet::new();
    for (index, entry) in entries.iter().enumerate() {
        if !manifest_paths.insert(&entry.manifest_path) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{root_path}.entries[{index}].manifest_path"),
                "unique_manifest_paths",
                "duplicate_manifest_path",
            ));
        }
    }
    Ok(())
}

fn validate_challenge_replay_checker_results_domain(
    results: &[Phase8ChallengeReplayCheckerResult],
    root_path: &str,
) -> Result<(), Phase8RequestValidationError> {
    let mut profiles = BTreeSet::new();
    for (index, result) in results.iter().enumerate() {
        if !profiles.insert(&result.checker_profile) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{root_path}[{index}].checker_profile"),
                "unique_checker_profiles",
                "duplicate_checker_profile",
            ));
        }
    }
    Ok(())
}

fn validate_challenge_replay_store_domain(
    results: &[Phase8ChallengeReplayStoreEntry],
    root_path: &str,
) -> Result<(), Phase8RequestValidationError> {
    for index in 1..results.len() {
        if results[index].result_hash < results[index - 1].result_hash {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{root_path}.results[{index}]"),
                "result_hash_bytewise_ascending",
                "order_violation",
            ));
        }
    }

    let mut result_hashes = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut challenge_manifests = BTreeSet::new();
    for (index, result) in results.iter().enumerate() {
        if !result_hashes.insert(result.result_hash) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{root_path}.results[{index}].result_hash"),
                "unique_result_hashes",
                "duplicate_result_hash",
            ));
        }
        if !paths.insert(&result.path) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{root_path}.results[{index}].path"),
                "unique_paths",
                "duplicate_path",
            ));
        }
        if !challenge_manifests.insert((&result.challenge_id, result.manifest_hash)) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{root_path}.results[{index}].challenge_id"),
                "unique_challenge_id_manifest_hash_pair",
                "duplicate_challenge_manifest",
            ));
        }
    }
    Ok(())
}

fn validate_challenge_coverage_summary_domain(
    entries: &[Phase8ChallengeCoverageEntry],
    total_challenges: u64,
    replayed_challenges: u64,
    unexpected_acceptances: u64,
    root_path: &str,
) -> Result<(), Phase8RequestValidationError> {
    for index in 1..entries.len() {
        let left = (&entries[index].challenge_id, entries[index].manifest_hash);
        let right = (
            &entries[index - 1].challenge_id,
            entries[index - 1].manifest_hash,
        );
        if left < right {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{root_path}.entries[{index}]"),
                "challenge_id_manifest_hash_bytewise_ascending",
                "order_violation",
            ));
        }
    }

    let mut challenge_manifests = BTreeSet::new();
    let mut replay_result_hashes = BTreeSet::new();
    for (index, entry) in entries.iter().enumerate() {
        if !challenge_manifests.insert((&entry.challenge_id, entry.manifest_hash)) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{root_path}.entries[{index}].challenge_id"),
                "unique_challenge_id_manifest_hash_pair",
                "duplicate_challenge_manifest",
            ));
        }
        if !replay_result_hashes.insert(entry.replay_result_hash) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{root_path}.entries[{index}].replay_result_hash"),
                "unique_replay_result_hashes",
                "duplicate_replay_result_hash",
            ));
        }
    }
    if replayed_challenges != entries.len() as u64 {
        return Err(Phase8RequestValidationError::value_failure(
            phase8_join_json_path(root_path, "replayed_challenges"),
            "entries.length",
            "count_mismatch",
        ));
    }
    if replayed_challenges > total_challenges {
        return Err(Phase8RequestValidationError::value_failure(
            phase8_join_json_path(root_path, "replayed_challenges"),
            "at_most_total_challenges",
            "count_mismatch",
        ));
    }
    if unexpected_acceptances > replayed_challenges {
        return Err(Phase8RequestValidationError::value_failure(
            phase8_join_json_path(root_path, "unexpected_acceptances"),
            "at_most_replayed_challenges",
            "count_mismatch",
        ));
    }
    Ok(())
}

fn validate_string_set_sorted_unique(
    values: &[String],
    field: &str,
    order_expected: &str,
    duplicate_actual: &str,
) -> Result<(), Phase8RequestValidationError> {
    for index in 1..values.len() {
        if values[index] < values[index - 1] {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{field}[{index}]"),
                order_expected,
                "order_violation",
            ));
        }
    }
    let mut seen = BTreeSet::new();
    for (index, value) in values.iter().enumerate() {
        if !seen.insert(value) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{field}[{index}]"),
                "unique_values",
                duplicate_actual,
            ));
        }
    }
    Ok(())
}

fn validate_string_set_unique(
    values: &[String],
    field: &str,
    duplicate_actual: &str,
) -> Result<(), Phase8RequestValidationError> {
    let mut seen = BTreeSet::new();
    for (index, value) in values.iter().enumerate() {
        if !seen.insert(value) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("{field}[{index}]"),
                "unique_values",
                duplicate_actual,
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

fn validate_phase8_auxiliary_result_envelope(
    result: &Phase8AuxiliaryResult,
) -> Result<(), Phase8RequestValidationError> {
    if !phase8_visible_ascii_nonempty(&result.result_id) {
        return Err(Phase8RequestValidationError::value_failure(
            "result_id",
            "result_id",
            if result.result_id.is_empty() {
                "empty_string"
            } else {
                "invalid_string_format"
            },
        ));
    }
    match (&result.selector, result.kind.requires_selector()) {
        (None, true) => {
            return Err(Phase8RequestValidationError::value_failure(
                "selector", "object", "missing",
            ))
        }
        (Some(_), false) => {
            return Err(Phase8RequestValidationError::value_failure(
                "selector", "absent", "present",
            ))
        }
        (Some(selector), true) if selector.kind() != result.kind => {
            return Err(Phase8RequestValidationError::value_failure(
                "selector",
                format!("selector_for_kind:{}", result.kind.as_str()),
                format!("selector_for_kind:{}", selector.kind().as_str()),
            ))
        }
        _ => {}
    }
    match (result.status, &result.error) {
        (Phase8AuxiliaryStatus::Passed, Some(_)) => {
            return Err(Phase8RequestValidationError::value_failure(
                "error", "absent", "present",
            ))
        }
        (Phase8AuxiliaryStatus::Failed | Phase8AuxiliaryStatus::Inconclusive, None) => {
            return Err(Phase8RequestValidationError::value_failure(
                "error",
                "auxiliary_failure",
                "missing",
            ))
        }
        _ => {}
    }
    if let Some(error) = &result.error {
        if !error.reason_code.is_compatible(result.kind, result.status) {
            return Err(Phase8RequestValidationError::value_failure(
                "error.reason_code",
                format!(
                    "auxiliary_reason_code:{}:{}",
                    result.kind.as_str(),
                    result.status.as_str()
                ),
                "invalid_enum",
            ));
        }
    }
    Ok(())
}

fn validate_auxiliary_result_store_domain(
    results: &[Phase8AuxiliaryResultStoreEntry],
) -> Result<(), Phase8RequestValidationError> {
    for index in 1..results.len() {
        if auxiliary_result_store_key(&results[index])
            < auxiliary_result_store_key(&results[index - 1])
        {
            return Err(Phase8RequestValidationError::value_failure(
                format!("auxiliary_result_store.results[{index}]"),
                "result_hash_kind_policy_hash_artifact_hash_ascending",
                "order_violation",
            ));
        }
    }
    let mut hashes = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for (index, result) in results.iter().enumerate() {
        if !hashes.insert(result.result_hash) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("auxiliary_result_store.results[{index}].result_hash"),
                "unique_result_hashes",
                "duplicate_result_hash",
            ));
        }
        if !paths.insert(&result.path) {
            return Err(Phase8RequestValidationError::value_failure(
                format!("auxiliary_result_store.results[{index}].path"),
                "unique_paths",
                "duplicate_path",
            ));
        }
    }
    Ok(())
}

fn auxiliary_result_store_key(
    result: &Phase8AuxiliaryResultStoreEntry,
) -> (Hash, &'static str, Hash, Hash) {
    (
        result.result_hash,
        result.kind.as_str(),
        result.policy_hash,
        result.artifact_hash,
    )
}

fn validate_auxiliary_result_id(
    result_id: &str,
    command: Phase8CommandName,
) -> Result<(), Phase8CommandError> {
    if phase8_visible_ascii_nonempty(result_id) {
        return Ok(());
    }
    Err(phase8_command_value_error(
        command,
        "input_reference_invalid",
        "result_id",
        "result_id",
        if result_id.is_empty() {
            "empty_string"
        } else {
            "invalid_string_format"
        },
    ))
}

fn phase8_command_for_auxiliary_kind(kind: Phase8AuxiliaryResultKind) -> Phase8CommandName {
    match kind {
        Phase8AuxiliaryResultKind::AxiomPolicy => Phase8CommandName::AuxiliaryAxiomPolicy,
        Phase8AuxiliaryResultKind::Reproducibility => Phase8CommandName::AuxiliaryReproducibility,
        Phase8AuxiliaryResultKind::ImportCertificateHash => {
            Phase8CommandName::AuxiliaryImportCertificateHash
        }
        Phase8AuxiliaryResultKind::AuditBundle => Phase8CommandName::ReleaseValidateBundle,
    }
}

fn phase8_reproducibility_identity_mismatch(
    baseline: &Phase8MachineCheckResult,
    repeated: &Phase8MachineCheckResult,
) -> Option<Phase8AuxiliaryError> {
    if baseline.status != repeated.status {
        return Some(Phase8AuxiliaryError::value(
            Phase8AuxiliaryReasonCode::ReproducibilityMismatch,
            "repeated.status",
            baseline.status.as_str(),
            repeated.status.as_str(),
        ));
    }
    if baseline.status == Phase8MachineCheckStatus::Failed {
        let baseline_key = baseline
            .error
            .as_ref()
            .map(Phase8NormalizedFailureKey::from_error);
        let repeated_key = repeated
            .error
            .as_ref()
            .map(Phase8NormalizedFailureKey::from_error);
        match (baseline_key, repeated_key) {
            (Some(left), Some(right)) if left.failure_key_hash() != right.failure_key_hash() => {
                return Some(Phase8AuxiliaryError::hash(
                    Phase8AuxiliaryReasonCode::ReproducibilityMismatch,
                    "repeated.derived_failure_key",
                    left.failure_key_hash(),
                    right.failure_key_hash(),
                ));
            }
            (Some(_), None) => {
                return Some(Phase8AuxiliaryError::value(
                    Phase8AuxiliaryReasonCode::ReproducibilityMismatch,
                    "repeated.derived_failure_key",
                    "present",
                    "missing",
                ));
            }
            (None, Some(_)) => {
                return Some(Phase8AuxiliaryError::value(
                    Phase8AuxiliaryReasonCode::ReproducibilityMismatch,
                    "repeated.derived_failure_key",
                    "absent",
                    "present",
                ));
            }
            _ => {}
        }
    }
    for (field, baseline_hash, repeated_hash) in [
        (
            "certificate_hash",
            baseline.certificate_hash,
            repeated.certificate_hash,
        ),
        ("export_hash", baseline.export_hash, repeated.export_hash),
        (
            "axiom_report_hash",
            baseline.axiom_report_hash,
            repeated.axiom_report_hash,
        ),
    ] {
        if let Some(error) =
            phase8_optional_hash_reproducibility_mismatch(field, baseline_hash, repeated_hash)
        {
            return Some(error);
        }
    }
    let baseline_result_hash = baseline.result_hash();
    let repeated_result_hash = repeated.result_hash();
    if baseline_result_hash != repeated_result_hash {
        return Some(Phase8AuxiliaryError::hash(
            Phase8AuxiliaryReasonCode::ReproducibilityMismatch,
            "repeated.result_hash",
            baseline_result_hash,
            repeated_result_hash,
        ));
    }
    None
}

fn phase8_reproducibility_comparability_mismatch(
    policy: &Phase8RunnerPolicy,
    request_hash: Hash,
    checker_profile: &str,
    baseline: &Phase8MachineCheckResult,
    repeated: &Phase8MachineCheckResult,
) -> Option<Phase8AuxiliaryError> {
    let policy_hash = policy.policy_hash();
    for (label, result) in [("baseline", baseline), ("repeated", repeated)] {
        if result.request_hash != request_hash {
            return Some(Phase8AuxiliaryError::hash(
                Phase8AuxiliaryReasonCode::ReproducibilityInconclusive,
                format!("{label}.request_hash"),
                request_hash,
                result.request_hash,
            ));
        }
    }
    for (label, result) in [("baseline", baseline), ("repeated", repeated)] {
        if result.checker.profile != checker_profile {
            return Some(Phase8AuxiliaryError::value(
                Phase8AuxiliaryReasonCode::ReproducibilityInconclusive,
                format!("{label}.checker.profile"),
                checker_profile,
                &result.checker.profile,
            ));
        }
    }
    for (label, result) in [("baseline", baseline), ("repeated", repeated)] {
        if result.policy.hash != policy_hash {
            return Some(Phase8AuxiliaryError::hash(
                Phase8AuxiliaryReasonCode::ReproducibilityInconclusive,
                format!("{label}.policy.hash"),
                policy_hash,
                result.policy.hash,
            ));
        }
    }

    let Some(selected) = policy.selected_checker_policy(checker_profile) else {
        return Some(Phase8AuxiliaryError::value(
            Phase8AuxiliaryReasonCode::ReproducibilityInconclusive,
            "selector.checker_profile",
            "allowed_checker_profile",
            checker_profile,
        ));
    };
    if let Some(error) = phase8_required_string_reproducibility_inconclusive(
        "baseline.checker.binary_id",
        &selected.binary_id,
        baseline.checker.binary_id.as_deref(),
    ) {
        return Some(error);
    }
    if let Some(error) = phase8_optional_string_reproducibility_inconclusive(
        "repeated.checker.binary_id",
        baseline.checker.binary_id.as_deref(),
        repeated.checker.binary_id.as_deref(),
    ) {
        return Some(error);
    }
    if let Some(error) = phase8_required_hash_reproducibility_inconclusive(
        "baseline.checker.binary_hash",
        selected.binary_hash,
        baseline.checker.binary_hash,
    ) {
        return Some(error);
    }
    if let Some(error) = phase8_optional_hash_reproducibility_inconclusive(
        "repeated.checker.binary_hash",
        baseline.checker.binary_hash,
        repeated.checker.binary_hash,
    ) {
        return Some(error);
    }
    if let Some(error) = phase8_required_string_reproducibility_inconclusive(
        "baseline.checker.id",
        &selected.checker_id,
        baseline.checker.id.as_deref(),
    ) {
        return Some(error);
    }
    if let Some(error) = phase8_optional_string_reproducibility_inconclusive(
        "repeated.checker.id",
        baseline.checker.id.as_deref(),
        repeated.checker.id.as_deref(),
    ) {
        return Some(error);
    }
    if let Some(error) = phase8_required_hash_reproducibility_inconclusive(
        "baseline.checker.build_hash",
        selected.build_hash,
        baseline.checker.build_hash,
    ) {
        return Some(error);
    }
    phase8_optional_hash_reproducibility_inconclusive(
        "repeated.checker.build_hash",
        baseline.checker.build_hash,
        repeated.checker.build_hash,
    )
}

fn phase8_optional_hash_reproducibility_mismatch(
    field: &str,
    baseline: Option<Hash>,
    repeated: Option<Hash>,
) -> Option<Phase8AuxiliaryError> {
    match (baseline, repeated) {
        (Some(expected), Some(actual)) if expected != actual => Some(Phase8AuxiliaryError::hash(
            Phase8AuxiliaryReasonCode::ReproducibilityMismatch,
            format!("repeated.{field}"),
            expected,
            actual,
        )),
        (Some(_), None) => Some(Phase8AuxiliaryError::value(
            Phase8AuxiliaryReasonCode::ReproducibilityMismatch,
            format!("repeated.{field}"),
            "present",
            "missing",
        )),
        (None, Some(_)) => Some(Phase8AuxiliaryError::value(
            Phase8AuxiliaryReasonCode::ReproducibilityMismatch,
            format!("repeated.{field}"),
            "absent",
            "present",
        )),
        _ => None,
    }
}

fn phase8_required_string_reproducibility_inconclusive(
    field: &str,
    expected: &str,
    actual: Option<&str>,
) -> Option<Phase8AuxiliaryError> {
    match actual {
        Some(actual) if actual != expected => Some(Phase8AuxiliaryError::value(
            Phase8AuxiliaryReasonCode::ReproducibilityInconclusive,
            field,
            expected,
            actual,
        )),
        Some(_) => None,
        None => Some(Phase8AuxiliaryError::value(
            Phase8AuxiliaryReasonCode::ReproducibilityInconclusive,
            field,
            "present",
            "missing",
        )),
    }
}

fn phase8_optional_string_reproducibility_inconclusive(
    field: &str,
    expected: Option<&str>,
    actual: Option<&str>,
) -> Option<Phase8AuxiliaryError> {
    match (expected, actual) {
        (Some(expected), Some(actual)) if expected != actual => Some(Phase8AuxiliaryError::value(
            Phase8AuxiliaryReasonCode::ReproducibilityInconclusive,
            field,
            expected,
            actual,
        )),
        (Some(_), None) => Some(Phase8AuxiliaryError::value(
            Phase8AuxiliaryReasonCode::ReproducibilityInconclusive,
            field,
            "present",
            "missing",
        )),
        (None, Some(_)) => Some(Phase8AuxiliaryError::value(
            Phase8AuxiliaryReasonCode::ReproducibilityInconclusive,
            field,
            "absent",
            "present",
        )),
        _ => None,
    }
}

fn phase8_required_hash_reproducibility_inconclusive(
    field: &str,
    expected: Hash,
    actual: Option<Hash>,
) -> Option<Phase8AuxiliaryError> {
    match actual {
        Some(actual) if actual != expected => Some(Phase8AuxiliaryError::hash(
            Phase8AuxiliaryReasonCode::ReproducibilityInconclusive,
            field,
            expected,
            actual,
        )),
        Some(_) => None,
        None => Some(Phase8AuxiliaryError::value(
            Phase8AuxiliaryReasonCode::ReproducibilityInconclusive,
            field,
            "present",
            "missing",
        )),
    }
}

fn phase8_optional_hash_reproducibility_inconclusive(
    field: &str,
    expected: Option<Hash>,
    actual: Option<Hash>,
) -> Option<Phase8AuxiliaryError> {
    match (expected, actual) {
        (Some(expected), Some(actual)) if expected != actual => Some(Phase8AuxiliaryError::hash(
            Phase8AuxiliaryReasonCode::ReproducibilityInconclusive,
            field,
            expected,
            actual,
        )),
        (Some(_), None) => Some(Phase8AuxiliaryError::value(
            Phase8AuxiliaryReasonCode::ReproducibilityInconclusive,
            field,
            "present",
            "missing",
        )),
        (None, Some(_)) => Some(Phase8AuxiliaryError::value(
            Phase8AuxiliaryReasonCode::ReproducibilityInconclusive,
            field,
            "absent",
            "present",
        )),
        _ => None,
    }
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

fn validate_phase8_challenge_generation_request_domain(
    request: &Phase8ChallengeGenerationRequest,
) -> Result<(), Phase8RequestValidationError> {
    if !phase8_valid_request_id(&request.request_id) {
        return Err(Phase8RequestValidationError::value_failure(
            "request_id",
            "request_id",
            if request.request_id.is_empty() {
                "empty_string"
            } else {
                "invalid_string_format"
            },
        ));
    }
    if !phase8_valid_challenge_id(&request.challenge_id) {
        return Err(Phase8RequestValidationError::value_failure(
            "challenge_id",
            "challenge_id",
            "invalid_name_format",
        ));
    }
    if !phase8_valid_dotted_name(&request.module) {
        return Err(Phase8RequestValidationError::value_failure(
            "module",
            "module_name",
            "invalid_name_format",
        ));
    }
    if request.imports.mode != "locked_store" {
        return Err(Phase8RequestValidationError::value_failure(
            "imports.mode",
            "locked_store",
            "invalid_enum",
        ));
    }
    for (field, path) in [
        ("imports.manifest", request.imports.manifest.as_str()),
        (
            "base_certificate.path",
            request.base_certificate.path.as_str(),
        ),
        (
            "output.store_manifest_path",
            request.output.store_manifest_path.as_str(),
        ),
        (
            "output.manifest_path",
            request.output.manifest_path.as_str(),
        ),
        (
            "output.mutated_certificate_path",
            request.output.mutated_certificate_path.as_str(),
        ),
    ] {
        if !phase8_valid_workspace_relative_path(path) {
            return Err(Phase8RequestValidationError::value_failure(
                field,
                "workspace_relative_path",
                "invalid_path",
            ));
        }
    }
    if request.output.store_manifest_path == request.output.manifest_path
        || request.output.store_manifest_path == request.output.mutated_certificate_path
        || request.output.manifest_path == request.output.mutated_certificate_path
    {
        return Err(Phase8RequestValidationError::value_failure(
            "output",
            "distinct_output_paths",
            "path_conflict",
        ));
    }
    let Some(known_kind) = request.mutation.known_kind() else {
        return Err(Phase8RequestValidationError::value_failure(
            "mutation.kind",
            "challenge_mutation_kind",
            "invalid_enum",
        ));
    };
    validate_challenge_mutation_target(&request.mutation.target, Some(known_kind), false).map_err(
        |actual| {
            Phase8RequestValidationError::value_failure(
                "mutation.target",
                "challenge_mutation_target",
                actual,
            )
        },
    )?;
    match (
        request.generated_by.kind,
        request.generated_by.prompt_hash.is_some(),
    ) {
        (Phase8ChallengeGeneratedByKind::Ai, false) => {
            return Err(Phase8RequestValidationError::value_failure(
                "generated_by.prompt_hash",
                "sha256:<lower-hex>",
                "missing",
            ))
        }
        (Phase8ChallengeGeneratedByKind::Ci, true) => {
            return Err(Phase8RequestValidationError::value_failure(
                "generated_by.prompt_hash",
                "absent",
                "present",
            ))
        }
        _ => {}
    }
    Ok(())
}

fn validate_challenge_materialize_output_paths(
    request_output_dir: &str,
    request_store_output_path: &str,
) -> Result<(), Phase8CommandError> {
    if !phase8_valid_workspace_relative_path(request_output_dir) {
        return Err(phase8_command_value_error(
            Phase8CommandName::ChallengeMaterializeRequests,
            "input_reference_invalid",
            "request_output_dir",
            "workspace_relative_path",
            "invalid_path",
        ));
    }
    if !phase8_valid_workspace_relative_path(request_store_output_path) {
        return Err(phase8_command_value_error(
            Phase8CommandName::ChallengeMaterializeRequests,
            "input_reference_invalid",
            "request_store_output_path",
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

#[derive(Clone, Debug)]
struct Phase8AxiomPolicyTomlAssignment {
    key: String,
    value: String,
}

fn phase8_parse_axiom_policy_toml(
    source: &str,
) -> Result<Phase8AxiomPolicy, Phase8PolicyValidationError> {
    if source.as_bytes().starts_with(&[0xef, 0xbb, 0xbf]) {
        return Err(Phase8PolicyValidationError::new(
            "axiom_policy",
            "valid_toml",
            "invalid_toml",
        ));
    }
    let assignments = phase8_collect_axiom_policy_toml_assignments(source)?;
    let mut seen = BTreeSet::new();
    for assignment in &assignments {
        if !seen.insert(assignment.key.clone()) {
            return Err(Phase8PolicyValidationError::new(
                phase8_axiom_policy_toml_field(&assignment.key),
                "unique_object_keys",
                "duplicate_field",
            ));
        }
    }

    let format_assignment = assignments
        .iter()
        .find(|assignment| assignment.key == "format");
    let Some(format_assignment) = format_assignment else {
        return Err(Phase8PolicyValidationError::new(
            "axiom_policy.format",
            PHASE8_AXIOM_POLICY_TOML_FORMAT,
            "missing",
        ));
    };
    let format_value =
        phase8_toml_parse_string_value(&format_assignment.value)?.ok_or_else(|| {
            Phase8PolicyValidationError::new(
                "axiom_policy.format",
                PHASE8_AXIOM_POLICY_TOML_FORMAT,
                "wrong_type",
            )
        })?;
    if format_value != PHASE8_AXIOM_POLICY_TOML_FORMAT {
        return Err(Phase8PolicyValidationError::new(
            "axiom_policy.format",
            PHASE8_AXIOM_POLICY_TOML_FORMAT,
            "invalid_fixed_value",
        ));
    }

    let allowed_assignment = assignments
        .iter()
        .find(|assignment| assignment.key == "allowed_axioms");
    let Some(allowed_assignment) = allowed_assignment else {
        return Err(Phase8PolicyValidationError::new(
            "axiom_policy.allowed_axioms",
            "array",
            "missing",
        ));
    };
    let allowed_entries = phase8_toml_parse_string_array_value(&allowed_assignment.value)?
        .ok_or_else(|| {
            Phase8PolicyValidationError::new("axiom_policy.allowed_axioms", "array", "wrong_type")
        })?;
    let mut allowed_axioms = Vec::new();
    for (index, entry) in allowed_entries.into_iter().enumerate() {
        let axiom = entry.map_err(|()| {
            Phase8PolicyValidationError::new(
                format!("axiom_policy.allowed_axioms[{index}]"),
                "axiom_name",
                "wrong_type",
            )
        })?;
        if !phase8_valid_dotted_name(&axiom) {
            return Err(Phase8PolicyValidationError::new(
                format!("axiom_policy.allowed_axioms[{index}]"),
                "axiom_name",
                "invalid_name_format",
            ));
        }
        allowed_axioms.push(axiom);
    }
    for index in 1..allowed_axioms.len() {
        let ordering = phase8_dotted_name_cmp(&allowed_axioms[index], &allowed_axioms[index - 1]);
        if ordering == Ordering::Less {
            return Err(Phase8PolicyValidationError::new(
                format!("axiom_policy.allowed_axioms[{index}]"),
                "axiom_name_canonical_order",
                "order_violation",
            ));
        }
    }
    let mut axiom_names = BTreeSet::new();
    for (index, axiom) in allowed_axioms.iter().enumerate() {
        if !axiom_names.insert(axiom) {
            return Err(Phase8PolicyValidationError::new(
                format!("axiom_policy.allowed_axioms[{index}]"),
                "unique_axiom_name",
                "duplicate_axiom_name",
            ));
        }
    }

    let mut unknown = assignments
        .iter()
        .map(|assignment| assignment.key.as_str())
        .filter(|key| !matches!(*key, "format" | "allowed_axioms"))
        .collect::<Vec<_>>();
    unknown.sort_by(|left, right| phase8_rfc8785_object_key_cmp(left, right));
    if let Some(key) = unknown.first() {
        return Err(Phase8PolicyValidationError::new(
            phase8_axiom_policy_toml_field(key),
            "absent",
            "unknown_field",
        ));
    }

    Ok(Phase8AxiomPolicy { allowed_axioms })
}

fn phase8_collect_axiom_policy_toml_assignments(
    source: &str,
) -> Result<Vec<Phase8AxiomPolicyTomlAssignment>, Phase8PolicyValidationError> {
    let lines = source.lines().collect::<Vec<_>>();
    let mut assignments = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = phase8_toml_strip_comment(lines[index])?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            index += 1;
            continue;
        }
        if trimmed.starts_with('[') {
            let Some(end) = trimmed.find(']') else {
                return Err(phase8_axiom_policy_invalid_toml());
            };
            if !trimmed[end + 1..].trim().is_empty() {
                return Err(phase8_axiom_policy_invalid_toml());
            }
            let key = trimmed[1..end].trim();
            if key.is_empty() {
                return Err(phase8_axiom_policy_invalid_toml());
            }
            assignments.push(Phase8AxiomPolicyTomlAssignment {
                key: key.to_owned(),
                value: "{table}".to_owned(),
            });
            index += 1;
            continue;
        }
        let Some(eq_index) = trimmed.find('=') else {
            return Err(phase8_axiom_policy_invalid_toml());
        };
        let key = trimmed[..eq_index].trim();
        if key.is_empty() || !key.split('.').all(phase8_budget_key_path_component) {
            return Err(phase8_axiom_policy_invalid_toml());
        }
        let mut value = trimmed[eq_index + 1..].trim().to_owned();
        while value.trim_start().starts_with('[') && !phase8_toml_array_closed(&value)? {
            index += 1;
            if index >= lines.len() {
                return Err(phase8_axiom_policy_invalid_toml());
            }
            value.push('\n');
            value.push_str(phase8_toml_strip_comment(lines[index])?.trim());
        }
        assignments.push(Phase8AxiomPolicyTomlAssignment {
            key: key.to_owned(),
            value,
        });
        index += 1;
    }
    Ok(assignments)
}

fn phase8_toml_strip_comment(line: &str) -> Result<&str, Phase8PolicyValidationError> {
    let mut in_string = false;
    let mut escaped = false;
    for (index, ch) in line.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }
            match ch {
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '#' => return Ok(&line[..index]),
            _ => {}
        }
    }
    if in_string || escaped {
        return Err(phase8_axiom_policy_invalid_toml());
    }
    Ok(line)
}

fn phase8_toml_array_closed(value: &str) -> Result<bool, Phase8PolicyValidationError> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for ch in value.chars() {
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }
            match ch {
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }
        match ch {
            '"' => in_string = true,
            '[' => depth = depth.saturating_add(1),
            ']' => {
                if depth == 0 {
                    return Err(phase8_axiom_policy_invalid_toml());
                }
                depth -= 1;
                if depth == 0 {
                    return Ok(true);
                }
            }
            _ => {}
        }
    }
    if in_string || escaped {
        return Err(phase8_axiom_policy_invalid_toml());
    }
    Ok(false)
}

fn phase8_toml_parse_string_value(
    value: &str,
) -> Result<Option<String>, Phase8PolicyValidationError> {
    let trimmed = value.trim();
    if trimmed == "null" {
        return Err(phase8_axiom_policy_invalid_toml());
    }
    if !trimmed.starts_with('"') {
        return Ok(None);
    }
    let (string, next) = phase8_toml_parse_basic_string_at(trimmed, 0)?;
    if !trimmed[next..].trim().is_empty() {
        return Err(phase8_axiom_policy_invalid_toml());
    }
    Ok(Some(string))
}

fn phase8_toml_parse_string_array_value(
    value: &str,
) -> Result<Option<Vec<Result<String, ()>>>, Phase8PolicyValidationError> {
    let trimmed = value.trim();
    if trimmed == "null" {
        return Err(phase8_axiom_policy_invalid_toml());
    }
    if !trimmed.starts_with('[') {
        return Ok(None);
    }
    let mut out = Vec::new();
    let mut index = 1usize;
    loop {
        index = phase8_toml_skip_ws(trimmed, index);
        if index >= trimmed.len() {
            return Err(phase8_axiom_policy_invalid_toml());
        }
        if trimmed[index..].starts_with(']') {
            index += 1;
            if !trimmed[index..].trim().is_empty() {
                return Err(phase8_axiom_policy_invalid_toml());
            }
            return Ok(Some(out));
        }
        if trimmed[index..].starts_with('"') {
            let (string, next) = phase8_toml_parse_basic_string_at(trimmed, index)?;
            out.push(Ok(string));
            index = phase8_toml_skip_ws(trimmed, next);
        } else {
            let start = index;
            while index < trimmed.len() && !matches!(trimmed.as_bytes()[index], b',' | b']') {
                index += 1;
            }
            if trimmed[start..index].trim() == "null" {
                return Err(phase8_axiom_policy_invalid_toml());
            }
            out.push(Err(()));
            index = phase8_toml_skip_ws(trimmed, index);
        }
        if index >= trimmed.len() {
            return Err(phase8_axiom_policy_invalid_toml());
        }
        if trimmed[index..].starts_with(',') {
            index += 1;
            continue;
        }
        if trimmed[index..].starts_with(']') {
            continue;
        }
        return Err(phase8_axiom_policy_invalid_toml());
    }
}

fn phase8_toml_parse_basic_string_at(
    value: &str,
    mut index: usize,
) -> Result<(String, usize), Phase8PolicyValidationError> {
    if !value[index..].starts_with('"') {
        return Err(phase8_axiom_policy_invalid_toml());
    }
    index += 1;
    let mut out = String::new();
    while index < value.len() {
        let ch = value[index..]
            .chars()
            .next()
            .ok_or_else(phase8_axiom_policy_invalid_toml)?;
        index += ch.len_utf8();
        match ch {
            '"' => return Ok((out, index)),
            '\\' => {
                let escaped = value[index..]
                    .chars()
                    .next()
                    .ok_or_else(phase8_axiom_policy_invalid_toml)?;
                index += escaped.len_utf8();
                match escaped {
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    'b' => out.push('\u{0008}'),
                    't' => out.push('\t'),
                    'n' => out.push('\n'),
                    'f' => out.push('\u{000c}'),
                    'r' => out.push('\r'),
                    _ => return Err(phase8_axiom_policy_invalid_toml()),
                }
            }
            '\u{0000}'..='\u{001f}' => return Err(phase8_axiom_policy_invalid_toml()),
            _ => out.push(ch),
        }
    }
    Err(phase8_axiom_policy_invalid_toml())
}

fn phase8_toml_skip_ws(value: &str, mut index: usize) -> usize {
    while index < value.len() {
        let ch = value[index..].chars().next().expect("index is in bounds");
        if ch.is_whitespace() {
            index += ch.len_utf8();
        } else {
            break;
        }
    }
    index
}

fn phase8_axiom_policy_invalid_toml() -> Phase8PolicyValidationError {
    Phase8PolicyValidationError::new("axiom_policy", "valid_toml", "invalid_toml")
}

fn phase8_axiom_policy_toml_field(key: &str) -> String {
    if key.split('.').all(phase8_budget_key_path_component) {
        format!("axiom_policy.{key}")
    } else {
        "axiom_policy".to_owned()
    }
}

fn phase8_raw_certificate_header_module(
    certificate_bytes: &[u8],
) -> Result<String, Phase8RawCertificateClaimError> {
    let mut decoder = Phase8RawCertificateHeaderDecoder::new(certificate_bytes);
    let _format = decoder.string()?;
    let _core_spec = decoder.string()?;
    decoder.name()
}

fn phase8_mutate_challenge_certificate_bytes(
    request: &Phase8ChallengeGenerationRequest,
    base_certificate_bytes: &[u8],
) -> Result<Vec<u8>, Phase8CommandError> {
    let kind = request
        .mutation
        .known_kind()
        .ok_or_else(|| phase8_challenge_mutation_error("mutation.kind", "known_kind", "unknown"))?;
    validate_challenge_mutation_target(&request.mutation.target, Some(kind), false).map_err(
        |actual| {
            phase8_challenge_mutation_error("mutation.target", "challenge_mutation_target", actual)
        },
    )?;

    match kind {
        Phase8ChallengeMutationKind::InsertUnsupportedSchemaVersion => {
            phase8_mutate_raw_certificate_schema_version(base_certificate_bytes)
        }
        Phase8ChallengeMutationKind::TruncateCertificateSection => {
            phase8_truncate_challenge_certificate_bytes(request, kind, base_certificate_bytes)
        }
        _ => phase8_flip_challenge_certificate_payload_byte(request, kind, base_certificate_bytes),
    }
}

fn phase8_mutate_raw_certificate_schema_version(
    base_certificate_bytes: &[u8],
) -> Result<Vec<u8>, Phase8CommandError> {
    let (format_start, format_end, format) =
        phase8_raw_certificate_format_range(base_certificate_bytes).map_err(|_| {
            phase8_challenge_mutation_error(
                "mutation.raw_layout",
                "raw_certificate_header",
                "decode_failed",
            )
        })?;
    if format != "NPA-CERT-0.1" {
        return Err(phase8_challenge_mutation_error(
            "mutation.raw_layout",
            "NPA-CERT-0.1",
            format,
        ));
    }

    let replacement = b"NPA-CERT-9.9";
    if replacement.len() != format_end - format_start {
        return Err(phase8_challenge_mutation_error(
            "mutation.raw_layout",
            "same_length_schema_replacement",
            "invalid_replacement_length",
        ));
    }
    let mut mutated = base_certificate_bytes.to_vec();
    mutated[format_start..format_end].copy_from_slice(replacement);
    Ok(mutated)
}

fn phase8_truncate_challenge_certificate_bytes(
    request: &Phase8ChallengeGenerationRequest,
    kind: Phase8ChallengeMutationKind,
    base_certificate_bytes: &[u8],
) -> Result<Vec<u8>, Phase8CommandError> {
    let candidates =
        phase8_challenge_ordered_payload_byte_positions(request, kind, base_certificate_bytes);
    if candidates.is_empty() {
        return Err(phase8_challenge_mutation_error(
            "mutation.candidates",
            "non_empty_certificate_payload",
            "empty",
        ));
    }
    let selected =
        phase8_challenge_mutation_selector_index(&request.mutation.seed, candidates.len());
    let truncate_at = candidates[selected];
    if truncate_at == 0 || truncate_at >= base_certificate_bytes.len() {
        return Err(phase8_challenge_mutation_error(
            "mutation.candidates",
            "valid_truncation_point",
            "invalid_offset",
        ));
    }
    Ok(base_certificate_bytes[..truncate_at].to_vec())
}

fn phase8_flip_challenge_certificate_payload_byte(
    request: &Phase8ChallengeGenerationRequest,
    kind: Phase8ChallengeMutationKind,
    base_certificate_bytes: &[u8],
) -> Result<Vec<u8>, Phase8CommandError> {
    let candidates =
        phase8_challenge_ordered_payload_byte_positions(request, kind, base_certificate_bytes);
    if candidates.is_empty() {
        return Err(phase8_challenge_mutation_error(
            "mutation.candidates",
            "non_empty_certificate_payload",
            "empty",
        ));
    }
    let selected =
        phase8_challenge_mutation_selector_index(&request.mutation.seed, candidates.len());
    let mut mutated = base_certificate_bytes.to_vec();
    mutated[candidates[selected]] ^= 0x01;
    Ok(mutated)
}

fn phase8_challenge_payload_byte_positions(certificate_bytes: &[u8]) -> Vec<usize> {
    if certificate_bytes.len() <= 32 {
        return Vec::new();
    }
    let claimed_hash_start = certificate_bytes.len() - 32;
    let header_end = phase8_raw_certificate_header_end(certificate_bytes)
        .ok()
        .filter(|offset| *offset < claimed_hash_start)
        .unwrap_or(0);
    let mut positions = (header_end..claimed_hash_start).collect::<Vec<_>>();
    if positions.is_empty() && claimed_hash_start > 0 {
        positions.extend(0..claimed_hash_start);
    }
    positions
}

fn phase8_challenge_ordered_payload_byte_positions(
    request: &Phase8ChallengeGenerationRequest,
    kind: Phase8ChallengeMutationKind,
    certificate_bytes: &[u8],
) -> Vec<usize> {
    let mut positions = phase8_challenge_payload_byte_positions(certificate_bytes);
    if !kind.requires_whole_certificate_target() && !positions.is_empty() {
        let rotation = phase8_challenge_target_candidate_rotation(
            kind,
            &request.mutation.target,
            positions.len(),
        );
        positions.rotate_left(rotation);
    }
    positions
}

fn phase8_challenge_target_candidate_rotation(
    kind: Phase8ChallengeMutationKind,
    target: &str,
    candidate_count: usize,
) -> usize {
    debug_assert!(candidate_count > 0);
    let mut hasher = Sha256::new();
    hasher.update(kind.as_str().as_bytes());
    hasher.update([0]);
    hasher.update(target.as_bytes());
    let digest = hasher.finalize();
    let mut prefix = [0u8; 8];
    prefix.copy_from_slice(&digest[..8]);
    (u64::from_be_bytes(prefix) as usize) % candidate_count
}

fn phase8_challenge_mutation_selector_index(seed: &Hash, candidate_count: usize) -> usize {
    debug_assert!(candidate_count > 0);
    let mut prefix = [0u8; 8];
    prefix.copy_from_slice(&seed[..8]);
    (u64::from_be_bytes(prefix) as usize) % candidate_count
}

fn phase8_raw_certificate_format_range(
    certificate_bytes: &[u8],
) -> Result<(usize, usize, String), Phase8RawCertificateClaimError> {
    let mut decoder = Phase8RawCertificateHeaderDecoder::new(certificate_bytes);
    let len = decoder.usize()?;
    let start = decoder.offset;
    let bytes = decoder.take(len)?;
    let value = std::str::from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|_| Phase8RawCertificateClaimError::DecodeFailed)?;
    Ok((start, decoder.offset, value))
}

fn phase8_raw_certificate_header_end(
    certificate_bytes: &[u8],
) -> Result<usize, Phase8RawCertificateClaimError> {
    let mut decoder = Phase8RawCertificateHeaderDecoder::new(certificate_bytes);
    let _format = decoder.string()?;
    let _core_spec = decoder.string()?;
    let _module = decoder.name()?;
    Ok(decoder.offset)
}

fn phase8_challenge_mutation_error(
    field: impl Into<String>,
    expected: impl Into<String>,
    actual: impl Into<String>,
) -> Phase8CommandError {
    phase8_command_value_error(
        Phase8CommandName::ChallengeGenerate,
        "mutation_target_invalid",
        field,
        expected,
        actual,
    )
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
    expected: &str,
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
            expected,
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
    let Some(value) = unique_optional_field_value(members, name, field, expected)? else {
        return Ok(None);
    };
    value
        .string_value()
        .map(|value| Some(value.to_owned()))
        .ok_or_else(|| wrong_type_error(field, expected, value.kind()))
}

fn optional_string_array_field(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
    expected_entry: &str,
) -> Result<Vec<String>, Phase8PolicyValidationError> {
    let Some(value) = unique_optional_field_value(members, name, field, "array")? else {
        return Ok(Vec::new());
    };
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

fn optional_hash_field(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
    expected: &str,
) -> Result<Option<Hash>, Phase8PolicyValidationError> {
    let Some(value) = unique_optional_field_value(members, name, field, expected)? else {
        return Ok(None);
    };
    let Some(raw) = value.string_value() else {
        return Err(wrong_type_error(field, expected, value.kind()));
    };
    parse_hash_string(raw)
        .map(Some)
        .map_err(|_| Phase8PolicyValidationError::new(field, expected, "invalid_hash_format"))
}

fn required_nonnegative_u64_field(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
) -> Result<u64, Phase8PolicyValidationError> {
    let raw = required_integer_raw_field(members, name, field, "non_negative_i64")?;
    let value = raw.parse::<u64>().map_err(|_| {
        Phase8PolicyValidationError::new(field, "non_negative_i64", "integer_out_of_range")
    })?;
    if value > i64::MAX as u64 {
        return Err(Phase8PolicyValidationError::new(
            field,
            "non_negative_i64",
            "integer_out_of_range",
        ));
    }
    Ok(value)
}

fn parse_required_phase8_normalized_comparison_status_field(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
) -> Result<Phase8NormalizedComparisonStatus, Phase8PolicyValidationError> {
    let raw = required_string_field(
        members,
        name,
        field,
        "NormalizedCheckResult.comparison.status",
    )?;
    Phase8NormalizedComparisonStatus::parse(&raw).ok_or_else(|| {
        Phase8PolicyValidationError::new(
            field,
            "NormalizedCheckResult.comparison.status",
            "invalid_enum",
        )
    })
}

fn parse_optional_phase8_normalized_comparison_status_field(
    members: &[JsonMember<'_>],
    name: &str,
    field: &str,
) -> Result<Option<Phase8NormalizedComparisonStatus>, Phase8PolicyValidationError> {
    let Some(raw) = optional_string_field(
        members,
        name,
        field,
        "NormalizedCheckResult.comparison.status",
    )?
    else {
        return Ok(None);
    };
    Phase8NormalizedComparisonStatus::parse(&raw)
        .map(Some)
        .ok_or_else(|| {
            Phase8PolicyValidationError::new(
                field,
                "NormalizedCheckResult.comparison.status",
                "invalid_enum",
            )
        })
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

fn phase8_printable_ascii_nonempty(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| (0x20..=0x7e).contains(&byte))
}

fn phase8_valid_request_id(value: &str) -> bool {
    phase8_visible_ascii_nonempty(value)
}

fn phase8_valid_challenge_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() || bytes.len() > 128 || !bytes[0].is_ascii_lowercase() {
        return false;
    }
    bytes
        .iter()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_')
}

fn phase8_valid_challenge_coverage_summary_id(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("chcov_") else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn phase8_valid_informational_challenge_kind(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() || bytes.len() > 64 || !bytes[0].is_ascii_lowercase() {
        return false;
    }
    bytes
        .iter()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_')
}

fn phase8_valid_informational_challenge_target(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 255
        && value.bytes().all(|byte| (0x21..=0x7e).contains(&byte))
}

fn validate_challenge_mutation_target(
    target: &str,
    known_kind: Option<Phase8ChallengeMutationKind>,
    allow_informational: bool,
) -> Result<(), &'static str> {
    match known_kind {
        Some(kind) if kind.requires_whole_certificate_target() => {
            if target == "$whole_certificate" {
                Ok(())
            } else {
                Err("invalid_enum")
            }
        }
        Some(_) => {
            if phase8_valid_dotted_name(target) {
                Ok(())
            } else {
                Err("invalid_name_format")
            }
        }
        None if allow_informational => {
            if phase8_valid_informational_challenge_target(target) {
                Ok(())
            } else {
                Err("invalid_name_format")
            }
        }
        None => Err("invalid_enum"),
    }
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

fn last_json_path_component(path: &str) -> &str {
    path.rsplit('.').next().unwrap_or(path)
}

fn phase8_join_json_path(root_path: &str, field: &str) -> String {
    if root_path == "$" {
        field.to_owned()
    } else {
        format!("{root_path}.{field}")
    }
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

        let auxiliary_hash = phase8_hash_usage_rule(Phase8HashUsageKind::AuxiliaryResultHash);
        assert_eq!(
            auxiliary_hash.excluded_fields,
            ["result_id", "result_hash", "diagnostics"]
        );
        assert!(auxiliary_hash.excludes_self_field);

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

    #[test]
    fn m5_missing_required_profiles_are_recorded_and_compare_validates_integrity() {
        let policy = parse_phase8_runner_policy(&m4_runner_policy_json()).unwrap();
        let fast = m4_stored_request(&policy, "fast-kernel", "mchkreq_fast");
        let request_store = m4_request_store_manifest(std::slice::from_ref(&fast));
        let fast_result = m4_checked_result(&fast.request, &policy, "mchkres_fast");
        let normalized = phase8_normalize_results(
            "norm_Std.Nat_missing",
            "normerr_Std.Nat_missing",
            &policy,
            &request_store,
            std::slice::from_ref(&fast),
            std::slice::from_ref(&fast_result),
            None,
        )
        .unwrap();

        assert_eq!(
            normalized.comparison.status,
            Phase8NormalizedComparisonStatus::MissingCheckerResult
        );
        assert_eq!(
            normalized.comparison.missing_checker_profiles,
            vec!["reference".to_owned(), "external".to_owned()]
        );
        let validation = phase8_compare_normalized_result(&policy, &normalized);
        assert_eq!(validation.status, Phase8CompareValidationStatus::Valid);
        assert_eq!(
            validation.embedded_comparison_status,
            Some(Phase8NormalizedComparisonStatus::MissingCheckerResult)
        );
    }

    #[test]
    fn m5_policy_outside_profile_is_comparison_policy_failure_not_normalize_error() {
        let policy = parse_phase8_runner_policy(&m4_runner_policy_json()).unwrap();
        let fast = m4_stored_request(&policy, "fast-kernel", "mchkreq_fast");
        let request_store = m4_request_store_manifest(std::slice::from_ref(&fast));
        let mut outside_result = m4_checked_result(&fast.request, &policy, "mchkres_outside");
        outside_result.checker.profile = "z-outside".to_owned();
        let normalized = phase8_normalize_results(
            "norm_Std.Nat_outside",
            "normerr_Std.Nat_outside",
            &policy,
            &request_store,
            std::slice::from_ref(&fast),
            std::slice::from_ref(&outside_result),
            Some(Phase8ArtifactSelector {
                module: "Std.Nat".to_owned(),
                request_hash: fast.request.request_hash(),
            }),
        )
        .unwrap();

        assert_eq!(
            normalized.comparison.status,
            Phase8NormalizedComparisonStatus::PolicyFailure
        );
        assert_eq!(normalized.comparison.status_reasons.len(), 1);
        let reason = &normalized.comparison.status_reasons[0];
        assert_eq!(reason.kind, "policy_failure");
        assert_eq!(reason.error_kind, "policy_failure");
        assert_eq!(reason.reason_code, "checker_profile_not_allowed");
        assert_eq!(reason.field.as_deref(), Some("results[].checker_profile"));
        assert_eq!(reason.actual_value.as_deref(), Some("z-outside"));
        assert!(normalized.comparison.disagreements.is_empty());
    }

    #[test]
    fn m5_copied_policy_failure_reason_is_not_reclassified_as_inconclusive() {
        let (request, policy) = m3_request_and_policy();
        let mut bad_request = request.clone();
        bad_request.budget.timeout_ms += 1;
        let stored = Phase8StoredMachineCheckRequest {
            path: "build/check-requests/Std.Nat.reference.json".to_owned(),
            file_hash: phase8_file_hash(bad_request.canonical_json().as_bytes()),
            request: bad_request.clone(),
        };
        let request_store = m4_request_store_manifest(std::slice::from_ref(&stored));
        let result = phase8_machine_check_run(
            &bad_request,
            &policy,
            Phase8CheckerRunObservation {
                result_id: "mchkres_policy_failure".to_owned(),
                attempt: 1,
                runner: m3_runner(),
                process: Phase8MachineCheckProcess::not_launched(),
                resource_usage: Phase8MachineCheckResourceUsage::zero(),
                stdout: Vec::new(),
                stderr: Vec::new(),
            },
        )
        .unwrap();
        let normalized = phase8_normalize_results(
            "norm_Std.Nat_policy_failure",
            "normerr_Std.Nat_policy_failure",
            &policy,
            &request_store,
            std::slice::from_ref(&stored),
            std::slice::from_ref(&result),
            None,
        )
        .unwrap();

        assert_eq!(
            normalized.comparison.status,
            Phase8NormalizedComparisonStatus::PolicyFailure
        );
        let reason = &normalized.comparison.status_reasons[0];
        assert_eq!(reason.kind, "policy_failure");
        assert_eq!(reason.error_kind, "policy_failure");
        assert_eq!(reason.reason_code, "request_budget_mismatch");
        assert_eq!(reason.field.as_deref(), Some("budget.timeout_ms"));
        assert!(normalized.comparison.disagreements.is_empty());
    }

    #[test]
    fn m5_inconclusive_sources_are_limited_to_closed_infrastructure_reasons() {
        let (request, policy) = m3_request_and_policy();
        let stored = Phase8StoredMachineCheckRequest {
            path: "build/check-requests/Std.Nat.reference.json".to_owned(),
            file_hash: phase8_file_hash(request.canonical_json().as_bytes()),
            request: request.clone(),
        };
        let request_store = m4_request_store_manifest(std::slice::from_ref(&stored));
        let result = phase8_machine_check_run(
            &request,
            &policy,
            Phase8CheckerRunObservation {
                result_id: "mchkres_malformed".to_owned(),
                attempt: 1,
                runner: m3_runner(),
                process: Phase8MachineCheckProcess::exited(0),
                resource_usage: m3_resource_usage(1),
                stdout: b"not json".to_vec(),
                stderr: Vec::new(),
            },
        )
        .unwrap();
        let normalized = phase8_normalize_results(
            "norm_Std.Nat_inconclusive",
            "normerr_Std.Nat_inconclusive",
            &policy,
            &request_store,
            std::slice::from_ref(&stored),
            std::slice::from_ref(&result),
            None,
        )
        .unwrap();

        assert_eq!(
            normalized.comparison.status,
            Phase8NormalizedComparisonStatus::Inconclusive
        );
        let reason = &normalized.comparison.status_reasons[0];
        assert_eq!(reason.kind, "inconclusive");
        assert_eq!(reason.error_kind, "checker_internal_error");
        assert_eq!(reason.reason_code, "malformed_success_output");
    }

    fn m5_failed_result(
        request: &Phase8MachineCheckRequest,
        policy: &Phase8RunnerPolicy,
        result_id: &str,
        declaration: &str,
    ) -> Phase8MachineCheckResult {
        let checker = policy
            .selected_checker_policy(&request.checker_profile)
            .unwrap();
        let raw = format!(
            r#"{{
              "schema":"npa.phase8.checker_raw_result.v1",
              "checker_id":"{}",
              "checker_version":"0.8.0",
              "checker_build_hash":"{}",
              "status":"failed",
              "module":"Std.Nat",
              "certificate_hash":"{}",
              "error":{{
                "kind":"type_mismatch",
                "declaration":"{}"
              }}
            }}"#,
            checker.checker_id,
            format_hash_string(&checker.build_hash),
            hash_wire(70),
            declaration
        );
        phase8_machine_check_run(
            request,
            policy,
            Phase8CheckerRunObservation {
                result_id: result_id.to_owned(),
                attempt: 1,
                runner: m3_runner(),
                process: Phase8MachineCheckProcess::exited(1),
                resource_usage: m3_resource_usage(100),
                stdout: raw.into_bytes(),
                stderr: Vec::new(),
            },
        )
        .unwrap()
    }

    #[test]
    fn m5_all_agree_failed_uses_full_failure_key_not_error_kind_only() {
        let policy = parse_phase8_runner_policy(&m4_runner_policy_json()).unwrap();
        let fast = m4_stored_request(&policy, "fast-kernel", "mchkreq_fast");
        let reference = m4_stored_request(&policy, "reference", "mchkreq_ref");
        let external = m4_stored_request(&policy, "external", "mchkreq_ext");
        let stored_requests = vec![fast.clone(), reference.clone(), external.clone()];
        let request_store = m4_request_store_manifest(&stored_requests);
        let fast_result = m5_failed_result(&fast.request, &policy, "mchkres_fast", "Std.Nat");
        let reference_result =
            m5_failed_result(&reference.request, &policy, "mchkres_ref", "Std.Nat.other");
        let external_result =
            m5_failed_result(&external.request, &policy, "mchkres_ext", "Std.Nat");
        let normalized = phase8_normalize_results(
            "norm_Std.Nat_failed_mismatch",
            "normerr_Std.Nat_failed_mismatch",
            &policy,
            &request_store,
            &stored_requests,
            &[fast_result, reference_result, external_result],
            None,
        )
        .unwrap();

        assert_eq!(
            normalized.comparison.status,
            Phase8NormalizedComparisonStatus::Disagreement
        );
        assert_eq!(normalized.comparison.disagreements.len(), 1);
        assert_eq!(normalized.comparison.disagreements[0].field, "failure_key");
        assert_eq!(
            normalized.comparison.disagreements[0]
                .baseline_checker_profile
                .as_deref(),
            Some("fast-kernel")
        );
        assert_eq!(
            normalized.comparison.disagreements[0]
                .checker_profile
                .as_str(),
            "reference"
        );
    }

    #[test]
    fn m5_disagreement_emits_status_and_same_status_hash_mismatches() {
        let policy = parse_phase8_runner_policy(&m4_runner_policy_json()).unwrap();
        let fast = m4_stored_request(&policy, "fast-kernel", "mchkreq_fast");
        let reference = m4_stored_request(&policy, "reference", "mchkreq_ref");
        let external = m4_stored_request(&policy, "external", "mchkreq_ext");
        let stored_requests = vec![fast.clone(), reference.clone(), external.clone()];
        let request_store = m4_request_store_manifest(&stored_requests);
        let fast_result = m4_checked_result(&fast.request, &policy, "mchkres_fast");
        let reference_result =
            m5_failed_result(&reference.request, &policy, "mchkres_ref", "Std.Nat");
        let external_result = m4_checked_result(&external.request, &policy, "mchkres_ext");
        let mut normalized = phase8_normalize_results(
            "norm_Std.Nat_mixed_mismatch",
            "normerr_Std.Nat_mixed_mismatch",
            &policy,
            &request_store,
            &stored_requests,
            &[fast_result, reference_result, external_result],
            None,
        )
        .unwrap();
        normalized.results[2].certificate_hash = Some(test_hash(177));
        let comparison = phase8_build_normalized_comparison(&policy, &normalized);

        assert_eq!(
            comparison.status,
            Phase8NormalizedComparisonStatus::Disagreement
        );
        assert_eq!(
            comparison
                .disagreements
                .iter()
                .map(|disagreement| (
                    disagreement.field.as_str(),
                    disagreement.checker_profile.as_str()
                ))
                .collect::<Vec<_>>(),
            vec![("certificate_hash", "external"), ("status", "reference")]
        );
    }

    #[test]
    fn m5_launched_checked_missing_checker_identity_is_policy_failure() {
        let (request, policy) = m3_request_and_policy();
        let stored = Phase8StoredMachineCheckRequest {
            path: "build/check-requests/Std.Nat.reference.json".to_owned(),
            file_hash: phase8_file_hash(request.canonical_json().as_bytes()),
            request: request.clone(),
        };
        let request_store = m4_request_store_manifest(std::slice::from_ref(&stored));
        let result = m4_checked_result(&request, &policy, "mchkres_checked");
        let mut normalized = phase8_normalize_results(
            "norm_Std.Nat_checked",
            "normerr_Std.Nat_checked",
            &policy,
            &request_store,
            std::slice::from_ref(&stored),
            std::slice::from_ref(&result),
            None,
        )
        .unwrap();
        normalized.results[0].checker_id = None;
        normalized.results[0].checker_build_hash = None;
        let comparison = phase8_build_normalized_comparison(&policy, &normalized);

        assert_eq!(
            comparison.status,
            Phase8NormalizedComparisonStatus::PolicyFailure
        );
        assert_eq!(comparison.status_reasons.len(), 2);
        assert!(comparison.status_reasons.iter().all(|reason| {
            reason.kind == "policy_failure"
                && reason.error_kind == "policy_failure"
                && reason.reason_code == "checker_identity_missing"
        }));
        assert_eq!(
            comparison
                .status_reasons
                .iter()
                .map(|reason| reason.field.as_deref().unwrap())
                .collect::<Vec<_>>(),
            vec!["results[].checker_build_hash", "results[].checker_id"]
        );
    }

    #[test]
    fn m5_compare_validation_result_is_transient_and_detects_mismatch() {
        let (request, policy) = m3_request_and_policy();
        let stored = Phase8StoredMachineCheckRequest {
            path: "build/check-requests/Std.Nat.reference.json".to_owned(),
            file_hash: phase8_file_hash(request.canonical_json().as_bytes()),
            request: request.clone(),
        };
        let request_store = m4_request_store_manifest(std::slice::from_ref(&stored));
        let result = m4_checked_result(&request, &policy, "mchkres_checked");
        let normalized = phase8_normalize_results(
            "norm_Std.Nat_checked",
            "normerr_Std.Nat_checked",
            &policy,
            &request_store,
            std::slice::from_ref(&stored),
            std::slice::from_ref(&result),
            None,
        )
        .unwrap();
        let valid = phase8_compare_normalized_result(&policy, &normalized);
        assert_eq!(valid.status, Phase8CompareValidationStatus::Valid);
        assert!(!valid.canonical_json().contains("\"result_hash\":"));

        let mut tampered = normalized.clone();
        tampered.comparison.status = Phase8NormalizedComparisonStatus::Inconclusive;
        let failed = phase8_compare_normalized_result(&policy, &tampered);
        assert_eq!(failed.status, Phase8CompareValidationStatus::Failed);
        assert_eq!(
            failed.error.as_ref().unwrap().kind,
            Phase8CompareValidationErrorKind::ComparisonMismatch
        );
        assert_eq!(
            failed.embedded_comparison_status,
            Some(Phase8NormalizedComparisonStatus::Inconclusive)
        );
        assert_eq!(
            failed.recomputed_comparison_status,
            Some(Phase8NormalizedComparisonStatus::AllAgreeChecked)
        );
        assert!(!failed.canonical_json().contains("\"result_hash\":"));
    }

    fn m6_input_policy_json(
        included_fields: &[&str],
        allow_source_text: bool,
        allow_tactic_trace: bool,
    ) -> String {
        let fields = included_fields
            .iter()
            .map(|field| (*field).to_owned())
            .collect::<Vec<_>>();
        format!(
            r#"{{
              "schema":"{}",
              "id":"phase8-ai-audit",
              "version":1,
              "included_fields":{},
              "redaction":"default",
              "allow_source_text":{},
              "allow_tactic_trace":{}
            }}"#,
            PHASE8_AI_AUDIT_INPUT_POLICY_SCHEMA,
            phase8_json_string_array(&fields),
            allow_source_text,
            allow_tactic_trace
        )
    }

    fn m6_machine_sidecar_json(
        result: &Phase8MachineCheckResult,
        input_policy: &Phase8AiAuditInputPolicy,
        prompt_hash: Hash,
        source_result_hash: Hash,
        source_request_hash: Hash,
        source_run_artifact_hash: Hash,
        extra_top_level: &str,
    ) -> String {
        let extra = if extra_top_level.is_empty() {
            String::new()
        } else {
            format!(",{extra_top_level}")
        };
        format!(
            r#"{{
              "schema":"{}",
              "source":{{
                "kind":"machine_result",
                "result_hash":"{}",
                "request_hash":"{}",
                "run_artifact_hash":"{}",
                "result_id":"{}"
              }},
              "input_policy":{{
                "id":"{}",
                "version":{},
                "hash":"{}",
                "included_fields":{},
                "redaction":"{}"
              }},
              "ai":{{
                "agent":"codex",
                "model":"gpt-5",
                "prompt_hash":"{}"
              }},
              "status":"summarized",
              "summary":"Reviewed deterministic machine result metadata."{}
            }}"#,
            PHASE8_AI_AUDIT_SIDECAR_SCHEMA,
            format_hash_string(&source_result_hash),
            format_hash_string(&source_request_hash),
            format_hash_string(&source_run_artifact_hash),
            result.result_id,
            input_policy.id,
            input_policy.version,
            format_hash_string(&input_policy.input_policy_hash()),
            phase8_json_string_array(&input_policy.included_fields),
            input_policy.redaction,
            format_hash_string(&prompt_hash),
            extra
        )
    }

    fn m6_machine_sidecar_with_prompt_hash(
        result: &Phase8MachineCheckResult,
        input_policy: &Phase8AiAuditInputPolicy,
        extra_top_level: &str,
    ) -> String {
        let placeholder = m6_machine_sidecar_json(
            result,
            input_policy,
            test_hash(220),
            result.result_hash(),
            result.request_hash,
            result.run_artifact_hash(),
            extra_top_level,
        );
        let sidecar = parse_phase8_ai_audit_sidecar(&placeholder).unwrap();
        let prompt_hash =
            phase8_ai_audit_prompt_input_for_sidecar(&sidecar, input_policy, Some(result), None)
                .unwrap()
                .prompt_hash();
        m6_machine_sidecar_json(
            result,
            input_policy,
            prompt_hash,
            result.result_hash(),
            result.request_hash,
            result.run_artifact_hash(),
            extra_top_level,
        )
    }

    fn m6_checked_result() -> Phase8MachineCheckResult {
        let (request, policy) = m3_request_and_policy();
        m4_checked_result(&request, &policy, "mchkres_ai_audit")
    }

    #[test]
    fn m6_input_policy_rejects_source_text_and_tactic_trace_fields() {
        let source_err = parse_phase8_ai_audit_input_policy(&m6_input_policy_json(
            &["source_text"],
            false,
            false,
        ))
        .unwrap_err();
        assert_eq!(
            source_err.reason_code,
            Phase8AuditSidecarValidationReasonCode::InputPolicySchemaInvalid
        );
        assert_eq!(source_err.field, "input_policy.included_fields[0]");
        assert_eq!(source_err.actual_value.as_deref(), Some("unknown_field"));

        let trace_err = parse_phase8_ai_audit_input_policy(&m6_input_policy_json(
            &["tactic_trace"],
            false,
            false,
        ))
        .unwrap_err();
        assert_eq!(
            trace_err.reason_code,
            Phase8AuditSidecarValidationReasonCode::InputPolicySchemaInvalid
        );
        assert_eq!(trace_err.field, "input_policy.included_fields[0]");
        assert_eq!(trace_err.actual_value.as_deref(), Some("unknown_field"));
    }

    #[test]
    fn m6_schema_only_rejects_verdict_fields_and_omits_cross_metadata() {
        let result = m6_checked_result();
        let policy_json = m6_input_policy_json(&["module", "status"], false, false);
        let input_policy = parse_phase8_ai_audit_input_policy(&policy_json).unwrap();
        let sidecar_json = m6_machine_sidecar_with_prompt_hash(&result, &input_policy, "");

        let valid = phase8_validate_ai_audit_sidecar_schema_only(&sidecar_json);
        assert_eq!(valid.status, Phase8AuditSidecarValidationStatus::Valid);
        let valid_json = valid.canonical_json();
        assert!(!valid_json.contains("input_policy_hash"));
        assert!(!valid_json.contains("source_result_hash"));
        assert!(!valid_json.contains("source_kind"));

        let prompt_hash = parse_phase8_ai_audit_sidecar(&sidecar_json)
            .unwrap()
            .ai
            .prompt_hash;
        let verdict_sidecar = m6_machine_sidecar_json(
            &result,
            &input_policy,
            prompt_hash,
            result.result_hash(),
            result.request_hash,
            result.run_artifact_hash(),
            r#""verdict":"checked""#,
        );
        let failed = phase8_validate_ai_audit_sidecar_schema_only(&verdict_sidecar);
        assert_eq!(failed.status, Phase8AuditSidecarValidationStatus::Failed);
        let error = failed.error.unwrap();
        assert_eq!(
            error.reason_code,
            Phase8AuditSidecarValidationReasonCode::ForbiddenSidecarField
        );
        assert_eq!(error.field, "verdict");

        let nested_secret_sidecar = m6_machine_sidecar_json(
            &result,
            &input_policy,
            prompt_hash,
            result.result_hash(),
            result.request_hash,
            result.run_artifact_hash(),
            r#""notes":[{"token":"provider-token"}]"#,
        );
        let failed = phase8_validate_ai_audit_sidecar_schema_only(&nested_secret_sidecar);
        assert_eq!(failed.status, Phase8AuditSidecarValidationStatus::Failed);
        let error = failed.error.unwrap();
        assert_eq!(
            error.reason_code,
            Phase8AuditSidecarValidationReasonCode::ForbiddenSidecarField
        );
        assert_eq!(error.field, "notes[0].token");
    }

    #[test]
    fn m6_machine_result_prompt_membership_fields_require_normalized_source() {
        let result = m6_checked_result();
        let policy_json = m6_input_policy_json(&["input_file_hash", "module"], false, false);
        let input_policy = parse_phase8_ai_audit_input_policy(&policy_json).unwrap();
        let sidecar_json = m6_machine_sidecar_json(
            &result,
            &input_policy,
            test_hash(221),
            result.result_hash(),
            result.request_hash,
            result.run_artifact_hash(),
            "",
        );
        let failed = phase8_validate_ai_audit_sidecar_schema_only(&sidecar_json);
        assert_eq!(failed.status, Phase8AuditSidecarValidationStatus::Failed);
        let error = failed.error.unwrap();
        assert_eq!(
            error.reason_code,
            Phase8AuditSidecarValidationReasonCode::SidecarSchemaInvalid
        );
        assert_eq!(error.field, "source.normalized_result_hash");
        assert_eq!(
            error.expected_value.as_deref(),
            Some("required_for_input_field")
        );
    }

    #[test]
    fn m6_cross_artifact_rejects_source_policy_and_prompt_mismatches() {
        let result = m6_checked_result();
        let policy_json = m6_input_policy_json(&["module", "status"], false, false);
        let input_policy = parse_phase8_ai_audit_input_policy(&policy_json).unwrap();
        let policy_hash = input_policy.input_policy_hash();
        let sidecar_json = m6_machine_sidecar_with_prompt_hash(&result, &input_policy, "");

        let valid = phase8_validate_ai_audit_sidecar_cross_artifact(
            &sidecar_json,
            &policy_json,
            policy_hash,
            std::slice::from_ref(&result),
            &[],
        );
        assert_eq!(valid.status, Phase8AuditSidecarValidationStatus::Valid);
        assert_eq!(valid.input_policy_hash, Some(policy_hash));
        assert_eq!(valid.source_result_hash, Some(result.result_hash()));
        assert!(!valid.canonical_json().contains("\"result_hash\":"));

        let source_mismatch_json = m6_machine_sidecar_json(
            &result,
            &input_policy,
            parse_phase8_ai_audit_sidecar(&sidecar_json)
                .unwrap()
                .ai
                .prompt_hash,
            test_hash(222),
            result.request_hash,
            result.run_artifact_hash(),
            "",
        );
        let source_mismatch = phase8_validate_ai_audit_sidecar_cross_artifact(
            &source_mismatch_json,
            &policy_json,
            policy_hash,
            std::slice::from_ref(&result),
            &[],
        );
        assert_eq!(
            source_mismatch.error.unwrap().reason_code,
            Phase8AuditSidecarValidationReasonCode::SourceHashMismatch
        );

        let policy_mismatch = phase8_validate_ai_audit_sidecar_cross_artifact(
            &sidecar_json,
            &policy_json,
            test_hash(223),
            std::slice::from_ref(&result),
            &[],
        );
        assert_eq!(
            policy_mismatch.error.unwrap().reason_code,
            Phase8AuditSidecarValidationReasonCode::InputPolicyHashMismatch
        );

        let prompt_mismatch_json = m6_machine_sidecar_json(
            &result,
            &input_policy,
            test_hash(224),
            result.result_hash(),
            result.request_hash,
            result.run_artifact_hash(),
            "",
        );
        let prompt_mismatch = phase8_validate_ai_audit_sidecar_cross_artifact(
            &prompt_mismatch_json,
            &policy_json,
            policy_hash,
            std::slice::from_ref(&result),
            &[],
        );
        assert_eq!(
            prompt_mismatch.error.unwrap().reason_code,
            Phase8AuditSidecarValidationReasonCode::PromptHashMismatch
        );
    }

    #[test]
    fn m6_prompt_projection_excludes_source_text_and_tactic_trace_payloads() {
        let result = m6_checked_result();
        let policy_json = m6_input_policy_json(&["module", "status"], false, false);
        let input_policy = parse_phase8_ai_audit_input_policy(&policy_json).unwrap();
        let sidecar_json = m6_machine_sidecar_with_prompt_hash(
            &result,
            &input_policy,
            r#""source_text":"secret theorem text","tactic_trace":["intro","exact"]"#,
        );
        let sidecar = parse_phase8_ai_audit_sidecar(&sidecar_json).unwrap();
        let prompt_input =
            phase8_ai_audit_prompt_input_for_sidecar(&sidecar, &input_policy, Some(&result), None)
                .unwrap();
        let prompt_json = prompt_input.canonical_json();
        assert!(prompt_json.contains("\"module\""));
        assert!(prompt_json.contains("\"status\""));
        assert!(!prompt_json.contains("source_text"));
        assert!(!prompt_json.contains("tactic_trace"));
        assert!(!prompt_json.contains("secret theorem text"));
        assert!(!prompt_json.contains("intro"));

        let validation = phase8_validate_ai_audit_sidecar_cross_artifact(
            &sidecar_json,
            &policy_json,
            input_policy.input_policy_hash(),
            std::slice::from_ref(&result),
            &[],
        );
        assert_eq!(
            validation.error.unwrap().reason_code,
            Phase8AuditSidecarValidationReasonCode::ForbiddenSidecarField
        );
    }

    #[test]
    fn m6_normalized_comparison_schema_forbids_checker_error_kind() {
        let policy_json = m6_input_policy_json(&["comparison.status", "module"], false, false);
        let input_policy = parse_phase8_ai_audit_input_policy(&policy_json).unwrap();
        let sidecar_json = format!(
            r#"{{
              "schema":"{}",
              "source":{{
                "kind":"normalized_comparison",
                "normalized_result_hash":"{}"
              }},
              "input_policy":{{
                "id":"{}",
                "version":{},
                "hash":"{}",
                "included_fields":{},
                "redaction":"{}"
              }},
              "ai":{{
                "agent":"codex",
                "model":"gpt-5",
                "prompt_hash":"{}"
              }},
              "status":"triaged",
              "classification":{{
                "category":"unknown",
                "confidence":"low",
                "checker_error_kind":123
              }},
              "summary":"Reviewed normalized comparison metadata."
            }}"#,
            PHASE8_AI_AUDIT_SIDECAR_SCHEMA,
            hash_wire(225),
            input_policy.id,
            input_policy.version,
            format_hash_string(&input_policy.input_policy_hash()),
            phase8_json_string_array(&input_policy.included_fields),
            input_policy.redaction,
            hash_wire(226)
        );
        let validation = phase8_validate_ai_audit_sidecar_schema_only(&sidecar_json);
        assert_eq!(
            validation.status,
            Phase8AuditSidecarValidationStatus::Failed
        );
        let error = validation.error.unwrap();
        assert_eq!(
            error.reason_code,
            Phase8AuditSidecarValidationReasonCode::ForbiddenSidecarField
        );
        assert_eq!(error.field, "classification.checker_error_kind");
    }

    fn m7_release_policy(mode: Phase8ReleaseMode) -> Phase8ReleasePolicy {
        Phase8ReleasePolicy {
            id: "phase8-release".to_owned(),
            version: 1,
            mode,
            runner_policy_hash: test_hash(1),
            challenge_runner_policy_hash: test_hash(2),
            ai_triage: Phase8ReleasePolicyAiTriage {
                enabled: false,
                required: false,
                input_policy_hash: None,
            },
        }
    }

    #[test]
    fn m7_axiom_policy_toml_validator_is_deterministic() {
        let policy = parse_phase8_axiom_policy_toml(
            r#"
            format = "npa.phase8.axiom_policy.v1"
            allowed_axioms = [
              "Std.Logic.choice",
              "Std.Logic.propExt",
            ]
            "#,
        )
        .unwrap();
        assert!(policy.allows("Std.Logic.choice"));
        assert!(!policy.allows("Std.Logic.em"));

        let duplicate = parse_phase8_axiom_policy_toml(
            r#"
            format = "npa.phase8.axiom_policy.v1"
            format = "npa.phase8.axiom_policy.v1"
            allowed_axioms = []
            "#,
        )
        .unwrap_err();
        assert_eq!(duplicate.field, "axiom_policy.format");
        assert_eq!(duplicate.actual_value, "duplicate_field");

        let order = parse_phase8_axiom_policy_toml(
            r#"
            format = "npa.phase8.axiom_policy.v1"
            allowed_axioms = ["Std.Logic.propExt", "Std.Logic.choice"]
            "#,
        )
        .unwrap_err();
        assert_eq!(order.field, "axiom_policy.allowed_axioms[1]");
        assert_eq!(order.actual_value, "order_violation");
    }

    #[test]
    fn m7_axiom_policy_auxiliary_checks_axiom_report_only() {
        let (request, policy) = m3_request_and_policy();
        let stored = Phase8StoredMachineCheckRequest {
            path: "build/check-requests/Std.Nat.reference.json".to_owned(),
            file_hash: phase8_file_hash(request.canonical_json().as_bytes()),
            request: request.clone(),
        };
        let request_store = m4_request_store_manifest(std::slice::from_ref(&stored));
        let result = m4_checked_result(&request, &policy, "mchkres_axiom_policy");
        let mut normalized = phase8_normalize_results(
            "norm_Std.Nat_axiom_policy",
            "normerr_Std.Nat_axiom_policy",
            &policy,
            &request_store,
            std::slice::from_ref(&stored),
            std::slice::from_ref(&result),
            None,
        )
        .unwrap();
        let report = Phase8AxiomReport {
            module: "Std.Nat".to_owned(),
            certificate_hash: test_hash(70),
            axioms: vec![Phase8AxiomReportEntry {
                name: "Std.Logic.choice".to_owned(),
            }],
        };
        normalized.results[0].axiom_report_hash = Some(report.axiom_report_hash());
        let axiom_policy = Phase8AxiomPolicy {
            allowed_axioms: vec!["Std.Logic.choice".to_owned()],
        };

        let passed = phase8_auxiliary_axiom_policy_result(
            "aux_axiom_001",
            &policy,
            &axiom_policy,
            &normalized,
            &report,
        )
        .unwrap();
        assert_eq!(passed.status, Phase8AuxiliaryStatus::Passed);
        assert_eq!(passed.kind, Phase8AuxiliaryResultKind::AxiomPolicy);

        let rejecting_policy = Phase8AxiomPolicy {
            allowed_axioms: Vec::new(),
        };
        let failed = phase8_auxiliary_axiom_policy_result(
            "aux_axiom_002",
            &policy,
            &rejecting_policy,
            &normalized,
            &report,
        )
        .unwrap();
        assert_eq!(failed.status, Phase8AuxiliaryStatus::Failed);
        let error = failed.error.as_ref().unwrap();
        assert_eq!(
            error.reason_code,
            Phase8AuxiliaryReasonCode::AxiomPolicyFailed
        );
        assert_eq!(error.field.as_deref(), Some("axiom_report.axioms[0].name"));
    }

    #[test]
    fn m7_reproducibility_uses_run_artifact_hash_and_deterministic_equality() {
        let (request, policy) = m3_request_and_policy();
        let baseline = m4_checked_result(&request, &policy, "mchkres_repro_base");
        let mut repeated = baseline.clone();
        repeated.result_id = "mchkres_repro_repeated".to_owned();
        repeated.attempt = 2;
        assert_eq!(baseline.result_hash(), repeated.result_hash());
        assert_ne!(baseline.run_artifact_hash(), repeated.run_artifact_hash());

        let passed = phase8_auxiliary_reproducibility_result(
            "aux_repro_001",
            &policy,
            test_hash(201),
            &baseline,
            &repeated,
        )
        .unwrap();
        assert_eq!(passed.status, Phase8AuxiliaryStatus::Passed);

        repeated.certificate_hash = Some(test_hash(202));
        let failed = phase8_auxiliary_reproducibility_result(
            "aux_repro_002",
            &policy,
            test_hash(201),
            &baseline,
            &repeated,
        )
        .unwrap();
        assert_eq!(failed.status, Phase8AuxiliaryStatus::Failed);
        let error = failed.error.as_ref().unwrap();
        assert_eq!(
            error.reason_code,
            Phase8AuxiliaryReasonCode::ReproducibilityMismatch
        );
        assert_eq!(error.field.as_deref(), Some("repeated.certificate_hash"));
    }

    #[test]
    fn m7_reproducibility_checker_identity_drift_is_inconclusive() {
        let (request, policy) = m3_request_and_policy();
        let baseline = m4_checked_result(&request, &policy, "mchkres_repro_identity_base");
        let mut repeated = baseline.clone();
        repeated.result_id = "mchkres_repro_identity_repeated".to_owned();
        repeated.attempt = 2;
        repeated.checker.build_hash = Some(test_hash(203));

        let inconclusive = phase8_auxiliary_reproducibility_result(
            "aux_repro_identity",
            &policy,
            test_hash(201),
            &baseline,
            &repeated,
        )
        .unwrap();
        assert_eq!(inconclusive.status, Phase8AuxiliaryStatus::Inconclusive);
        let error = inconclusive.error.as_ref().unwrap();
        assert_eq!(
            error.reason_code,
            Phase8AuxiliaryReasonCode::ReproducibilityInconclusive
        );
        assert_eq!(error.field.as_deref(), Some("repeated.checker.build_hash"));
    }

    #[test]
    fn m7_import_certificate_hash_requires_high_trust_release_policy() {
        let certificate_hash = test_hash(70);
        let certificate_bytes = test_raw_certificate_bytes("Std.Nat", certificate_hash);
        let certificate_path = "build/certs/Std/Nat.npcert".to_owned();
        let import_lock = Phase8ImportLockManifest {
            imports: vec![Phase8ImportLockEntry {
                module: "Std.Nat".to_owned(),
                export_hash: test_hash(71),
                certificate: Phase8ImportLockCertificate {
                    path: certificate_path.clone(),
                    file_hash: phase8_file_hash(&certificate_bytes),
                    certificate_hash,
                },
            }],
        };
        let mut files = BTreeMap::new();
        files.insert(certificate_path, certificate_bytes);

        let release_policy = m7_release_policy(Phase8ReleaseMode::Release);
        let err = phase8_auxiliary_import_certificate_hash_result(
            "aux_import_001",
            &release_policy,
            phase8_file_hash(import_lock.canonical_json().as_bytes()),
            &import_lock,
            &files,
        )
        .unwrap_err();
        assert_eq!(err.reason_code.as_ref(), "policy_reference_invalid");
        assert_eq!(err.field.as_deref(), Some("release_policy.mode"));

        let high_trust = m7_release_policy(Phase8ReleaseMode::HighTrust);
        let passed = phase8_auxiliary_import_certificate_hash_result(
            "aux_import_002",
            &high_trust,
            phase8_file_hash(import_lock.canonical_json().as_bytes()),
            &import_lock,
            &files,
        )
        .unwrap();
        assert_eq!(passed.status, Phase8AuxiliaryStatus::Passed);
        assert!(passed.selector.is_none());
    }

    #[test]
    fn m7_auxiliary_result_envelope_store_and_pass_condition_are_closed() {
        let passed = Phase8AuxiliaryResult::passed(
            "aux_import_passed",
            Phase8AuxiliaryResultKind::ImportCertificateHash,
            test_hash(1),
            test_hash(2),
            None,
        );
        let parsed = parse_phase8_auxiliary_result(&passed.canonical_json()).unwrap();
        assert_eq!(parsed, passed);

        let mut diagnostic_retry = passed.clone();
        diagnostic_retry
            .diagnostics
            .push("local-note-not-in-result-hash".to_owned());
        assert_eq!(passed.result_hash(), diagnostic_retry.result_hash());
        assert_ne!(passed.canonical_json(), diagnostic_retry.canonical_json());

        let entry =
            phase8_auxiliary_result_store_entry_for_result(&passed, "build/aux/import.json")
                .unwrap();
        let update = phase8_auxiliary_result_store_with_entry(None, entry).unwrap();
        let parsed_store =
            parse_phase8_auxiliary_result_store_manifest(&update.manifest.canonical_json())
                .unwrap();
        assert_eq!(parsed_store, update.manifest);

        let failed = Phase8AuxiliaryResult::failed(
            "aux_import_failed",
            Phase8AuxiliaryResultKind::ImportCertificateHash,
            test_hash(1),
            test_hash(2),
            None,
            Phase8AuxiliaryError::hash(
                Phase8AuxiliaryReasonCode::ImportCertificateHashMismatch,
                "import_lock.imports[0].certificate.certificate_hash",
                test_hash(3),
                test_hash(4),
            ),
        );
        let failed_command: Result<Phase8AuxiliaryResult, Phase8CommandError> = Ok(failed.clone());
        assert!(phase8_auxiliary_command_exit_success(&failed_command));
        assert!(phase8_auxiliary_result_passes_release_condition(&passed));
        assert!(!phase8_auxiliary_result_passes_release_condition(&failed));
        assert!(!phase8_auxiliary_results_all_passed(&[passed, failed]));

        let write_failure = phase8_auxiliary_output_write_failure(
            Phase8CommandName::AuxiliaryAxiomPolicy,
            "out.path",
        );
        assert_eq!(write_failure.reason_code.as_ref(), "output_write_failure");
        let failed_write: Result<Phase8AuxiliaryResult, Phase8CommandError> = Err(write_failure);
        assert!(!phase8_auxiliary_command_exit_success(&failed_write));
    }

    fn m8_generation_request(
        mutation_kind: Phase8ChallengeMutationKind,
        target: &str,
        seed: Hash,
    ) -> (
        Phase8ChallengeGenerationRequest,
        Phase8RunnerPolicy,
        Vec<u8>,
        String,
    ) {
        let policy = parse_phase8_runner_policy(&valid_runner_policy_json()).unwrap();
        let imports_json = valid_import_lock_manifest_json();
        let base_certificate_bytes = test_raw_certificate_bytes("Std.Nat", test_hash(70));
        let base_file_hash = phase8_file_hash(&base_certificate_bytes);
        let imports_hash = phase8_file_hash(imports_json.as_bytes());
        let request = Phase8ChallengeGenerationRequest {
            request_id: "chgen_001".to_owned(),
            challenge_id: "pch_001".to_owned(),
            policy_hash: policy.policy_hash(),
            module: "Std.Nat".to_owned(),
            imports: Phase8ChallengeImports {
                mode: "locked_store".to_owned(),
                manifest: "build/certs/import-lock.json".to_owned(),
                manifest_hash: imports_hash,
            },
            base_certificate: Phase8ChallengeBaseCertificate {
                path: "build/certs/Std/Nat.npcert".to_owned(),
                file_hash: base_file_hash,
                claimed_certificate_hash: test_hash(70),
            },
            mutation: Phase8ChallengeMutation {
                kind: mutation_kind.as_str().to_owned(),
                target: target.to_owned(),
                seed,
            },
            output: Phase8ChallengeGenerationOutput {
                store_manifest_path: "build/challenges/manifest.json".to_owned(),
                manifest_path: "build/challenges/pch_001/manifest.json".to_owned(),
                mutated_certificate_path: "build/challenges/pch_001/mutated.npcert".to_owned(),
            },
            generated_by: Phase8ChallengeGeneratedBy {
                kind: Phase8ChallengeGeneratedByKind::Ai,
                prompt_hash: Some(test_hash(241)),
            },
        };
        (request, policy, base_certificate_bytes, imports_json)
    }

    #[test]
    fn m8_challenge_generation_request_hash_and_store_retry_are_deterministic() {
        let (request, policy, base_certificate_bytes, imports_json) = m8_generation_request(
            Phase8ChallengeMutationKind::InsertUnsupportedSchemaVersion,
            "$whole_certificate",
            test_hash(240),
        );

        let parsed = parse_phase8_challenge_generation_request(&request.canonical_json()).unwrap();
        assert_eq!(parsed, request);
        assert_eq!(
            phase8_challenge_generation_request_hash(&request.canonical_json()).unwrap(),
            request.request_hash()
        );
        assert!(!request
            .hash_input_canonical_json()
            .contains("\"request_id\""));
        assert!(!request
            .hash_input_canonical_json()
            .contains("\"request_hash\""));

        let generated = phase8_challenge_generate(
            &request,
            &policy,
            &base_certificate_bytes,
            imports_json.as_bytes(),
            None,
        )
        .unwrap();
        assert_eq!(generated.manifest.challenge_id, request.challenge_id);
        assert_eq!(generated.manifest.policy_hash, policy.policy_hash());
        assert_eq!(generated.manifest.imports, request.imports);
        assert_eq!(generated.manifest.replay.args_hash, request.request_hash());
        assert_eq!(generated.result.status, "written");
        assert!(generated
            .result
            .canonical_json()
            .contains(PHASE8_CHALLENGE_GENERATION_RESULT_SCHEMA));
        assert!(std::str::from_utf8(&generated.mutated_certificate_bytes)
            .unwrap()
            .contains("NPA-CERT-9.9"));

        let parsed_manifest =
            parse_phase8_challenge_manifest(&generated.manifest.canonical_json()).unwrap();
        assert_eq!(parsed_manifest, generated.manifest);
        let parsed_store = parse_phase8_challenge_output_store_manifest(
            &generated.challenge_store.canonical_json(),
        )
        .unwrap();
        assert_eq!(parsed_store, generated.challenge_store);

        let retry = phase8_challenge_generate(
            &request,
            &policy,
            &base_certificate_bytes,
            imports_json.as_bytes(),
            Some(&generated.challenge_store),
        )
        .unwrap();
        assert!(!retry.challenge_store_rewrite_required);
        assert_eq!(retry.challenge_store, generated.challenge_store);

        let conflict_store = Phase8ChallengeOutputStoreManifest {
            entries: vec![Phase8ChallengeOutputStoreEntry {
                challenge_id: request.challenge_id.clone(),
                manifest_path: "build/challenges/other/manifest.json".to_owned(),
                manifest_hash: test_hash(250),
            }],
        };
        let conflict = phase8_challenge_output_store_with_entry(
            Some(&conflict_store),
            generated.challenge_store.entries[0].clone(),
        )
        .unwrap_err();
        assert_eq!(conflict.reason_code.as_ref(), "challenge_id_conflict");
    }

    #[test]
    fn m8_materialize_requests_uses_manifest_claimed_hash_and_never_checks() {
        let (request, policy, base_certificate_bytes, imports_json) = m8_generation_request(
            Phase8ChallengeMutationKind::FlipCanonicalEncodingByte,
            "$whole_certificate",
            test_hash(242),
        );
        let generated = phase8_challenge_generate(
            &request,
            &policy,
            &base_certificate_bytes,
            imports_json.as_bytes(),
            None,
        )
        .unwrap();

        let mut manifest = generated.manifest.clone();
        manifest.mutated_certificate.claimed_certificate_hash = None;
        let materialized = phase8_challenge_materialize_requests(
            &manifest,
            manifest.manifest_hash(),
            &policy,
            "build/check-requests/challenges/pch_001",
            "build/check-requests/challenges/manifest.json",
            None,
        )
        .unwrap();
        assert_eq!(materialized.requests.len(), 1);
        assert_eq!(
            materialized.requests[0].request_id,
            "chreq:pch_001:reference"
        );
        assert_eq!(
            materialized.requests[0]
                .certificate
                .expected_certificate_hash,
            manifest.base_certificate.claimed_certificate_hash
        );
        assert_eq!(
            materialized.requests[0].certificate.file_hash,
            manifest.mutated_certificate.file_hash
        );
        assert_eq!(
            materialized.result.request_store.manifest_hash,
            materialized.request_store.file_hash()
        );
        assert!(materialized
            .result
            .canonical_json()
            .contains(PHASE8_CHALLENGE_REQUEST_MATERIALIZATION_RESULT_SCHEMA));

        let retry = phase8_challenge_materialize_requests(
            &manifest,
            manifest.manifest_hash(),
            &policy,
            "build/check-requests/challenges/pch_001",
            "build/check-requests/challenges/manifest.json",
            Some(&materialized.request_store),
        )
        .unwrap();
        assert!(!retry.request_store_rewrite_required);

        manifest.mutated_certificate.claimed_certificate_hash = Some(test_hash(243));
        let with_mutated_claim = phase8_challenge_materialize_requests(
            &manifest,
            manifest.manifest_hash(),
            &policy,
            "build/check-requests/challenges/pch_001b",
            "build/check-requests/challenges/manifest-b.json",
            None,
        )
        .unwrap();
        assert_eq!(
            with_mutated_claim.requests[0]
                .certificate
                .expected_certificate_hash,
            test_hash(243)
        );
    }

    #[test]
    fn m8_generation_preserves_import_manifest_error_path() {
        let (mut request, policy, base_certificate_bytes, imports_json) = m8_generation_request(
            Phase8ChallengeMutationKind::FlipCanonicalEncodingByte,
            "$whole_certificate",
            test_hash(243),
        );
        let bad_imports_json = imports_json.replace(
            PHASE8_IMPORT_LOCK_MANIFEST_SCHEMA,
            "npa.phase8.bad_imports.v1",
        );
        request.imports.manifest_hash = phase8_file_hash(bad_imports_json.as_bytes());

        let err = phase8_challenge_generate(
            &request,
            &policy,
            &base_certificate_bytes,
            bad_imports_json.as_bytes(),
            None,
        )
        .unwrap_err();
        assert_eq!(
            err.reason_code.as_ref(),
            "generation_request_schema_invalid"
        );
        assert_eq!(err.field.as_deref(), Some("imports.manifest.schema"));
    }

    #[test]
    fn m8_informational_manifest_is_not_release_rejection_requirement() {
        let (request, policy, base_certificate_bytes, imports_json) = m8_generation_request(
            Phase8ChallengeMutationKind::FlipCanonicalEncodingByte,
            "$whole_certificate",
            test_hash(244),
        );
        let generated = phase8_challenge_generate(
            &request,
            &policy,
            &base_certificate_bytes,
            imports_json.as_bytes(),
            None,
        )
        .unwrap();

        let mut informational = generated.manifest.clone();
        informational.mutation.kind = "third_party_probe".to_owned();
        informational.mutation.target = "opaque-target".to_owned();
        let parsed = parse_phase8_challenge_manifest(&informational.canonical_json()).unwrap();
        assert_eq!(parsed, informational);
        assert!(!phase8_challenge_manifest_is_rejection_required(&parsed));

        let unknown_request_json = request
            .canonical_json()
            .replace("flip_canonical_encoding_byte", "third_party_probe");
        let err = parse_phase8_challenge_generation_request(&unknown_request_json).unwrap_err();
        assert_eq!(err.field.as_ref(), "mutation.kind");
        assert_eq!(err.actual_value.as_deref(), Some("invalid_enum"));
    }

    #[test]
    fn m8_mutation_selection_depends_on_kind_target_and_seed() {
        let seed = test_hash(245);
        let (request, policy, base_certificate_bytes, imports_json) = m8_generation_request(
            Phase8ChallengeMutationKind::DropAxiomReportEntry,
            "Std.Nat.foo",
            seed,
        );
        let first = phase8_challenge_generate(
            &request,
            &policy,
            &base_certificate_bytes,
            imports_json.as_bytes(),
            None,
        )
        .unwrap();
        let repeated = phase8_challenge_generate(
            &request,
            &policy,
            &base_certificate_bytes,
            imports_json.as_bytes(),
            None,
        )
        .unwrap();
        assert_eq!(
            first.mutated_certificate_bytes,
            repeated.mutated_certificate_bytes
        );

        let candidates = phase8_challenge_payload_byte_positions(&base_certificate_bytes);
        let base_index = phase8_challenge_mutation_selector_index(&seed, candidates.len());
        let mut seed_prefix = [0u8; 8];
        seed_prefix.copy_from_slice(&seed[..8]);
        assert_eq!(
            base_index,
            (u64::from_be_bytes(seed_prefix) as usize) % candidates.len()
        );
        let different_seed = (246..=255)
            .map(test_hash)
            .find(|candidate| {
                phase8_challenge_mutation_selector_index(candidate, candidates.len()) != base_index
            })
            .unwrap();
        let base_candidate = phase8_challenge_ordered_payload_byte_positions(
            &request,
            Phase8ChallengeMutationKind::DropAxiomReportEntry,
            &base_certificate_bytes,
        )[base_index];
        let different_target = ["Std.Nat.bar", "Std.Nat.baz", "Std.Nat.qux"]
            .into_iter()
            .find(|target| {
                let mut target_request = request.clone();
                target_request.mutation.target = (*target).to_owned();
                phase8_challenge_ordered_payload_byte_positions(
                    &target_request,
                    Phase8ChallengeMutationKind::DropAxiomReportEntry,
                    &base_certificate_bytes,
                )[base_index]
                    != base_candidate
            })
            .unwrap();

        let (seed_request, _, _, _) = m8_generation_request(
            Phase8ChallengeMutationKind::DropAxiomReportEntry,
            "Std.Nat.foo",
            different_seed,
        );
        let seed_changed = phase8_challenge_generate(
            &seed_request,
            &policy,
            &base_certificate_bytes,
            imports_json.as_bytes(),
            None,
        )
        .unwrap();
        assert_ne!(
            first.mutated_certificate_bytes,
            seed_changed.mutated_certificate_bytes
        );

        let (target_request, _, _, _) = m8_generation_request(
            Phase8ChallengeMutationKind::DropAxiomReportEntry,
            different_target,
            seed,
        );
        let target_changed = phase8_challenge_generate(
            &target_request,
            &policy,
            &base_certificate_bytes,
            imports_json.as_bytes(),
            None,
        )
        .unwrap();
        assert_ne!(
            first.mutated_certificate_bytes,
            target_changed.mutated_certificate_bytes
        );
    }

    fn m9_stored_requests(
        materialized: &Phase8ChallengeRequestMaterialization,
    ) -> Vec<Phase8StoredMachineCheckRequest> {
        materialized
            .requests
            .iter()
            .map(|request| {
                let entry = materialized
                    .request_store
                    .requests
                    .iter()
                    .find(|entry| entry.request_hash == request.request_hash())
                    .unwrap();
                Phase8StoredMachineCheckRequest {
                    path: entry.path.clone(),
                    file_hash: entry.file_hash,
                    request: request.clone(),
                }
            })
            .collect()
    }

    fn m9_result_store(
        result: &Phase8MachineCheckResult,
        path: &str,
    ) -> Phase8MachineResultStoreManifest {
        let entry = phase8_machine_result_store_entry_for_result(result, path).unwrap();
        phase8_machine_result_store_with_entry(None, entry)
            .unwrap()
            .manifest
    }

    fn m9_normalized_store(
        result: &Phase8NormalizedCheckResult,
    ) -> Phase8NormalizedResultStoreManifest {
        let entry = phase8_normalized_result_store_entry_for_result(
            result,
            "build/normalized/challenge-pch_001.json",
        )
        .unwrap();
        phase8_normalized_result_store_with_entry(None, entry)
            .unwrap()
            .manifest
    }

    fn m9_target_normalized_result(policy: &Phase8RunnerPolicy) -> Phase8NormalizedCheckResult {
        let imports_json = valid_import_lock_manifest_json();
        let cert_bytes = test_raw_certificate_bytes("Std.Nat", test_hash(70));
        let materialized = phase8_request_materialize(
            policy,
            "Std.Nat",
            "build/certs/Std/Nat.npcert",
            &cert_bytes,
            "build/certs/import-lock.json",
            imports_json.as_bytes(),
            phase8_file_hash(imports_json.as_bytes()),
            "reference",
            "mchkreq_target",
            "build/check-requests/Std.Nat.reference.json",
            None,
        )
        .unwrap();
        let stored = Phase8StoredMachineCheckRequest {
            path: "build/check-requests/Std.Nat.reference.json".to_owned(),
            file_hash: materialized.request_file_hash,
            request: materialized.request.clone(),
        };
        let result = m4_checked_result(&materialized.request, policy, "mchkres_target");
        phase8_normalize_results(
            "norm_Std.Nat_target",
            "normerr_Std.Nat_target",
            policy,
            &materialized.request_store,
            &[stored],
            &[result],
            None,
        )
        .unwrap()
    }

    fn m9_generated_challenge() -> (
        Phase8RunnerPolicy,
        Phase8ChallengeGeneration,
        Phase8ChallengeRequestMaterialization,
    ) {
        let (request, policy, base_certificate_bytes, imports_json) = m8_generation_request(
            Phase8ChallengeMutationKind::InsertUnsupportedSchemaVersion,
            "$whole_certificate",
            test_hash(248),
        );
        let generated = phase8_challenge_generate(
            &request,
            &policy,
            &base_certificate_bytes,
            imports_json.as_bytes(),
            None,
        )
        .unwrap();
        let materialized = phase8_challenge_materialize_requests(
            &generated.manifest,
            generated.manifest.manifest_hash(),
            &policy,
            "build/check-requests/challenges/pch_001",
            "build/check-requests/challenges/manifest.json",
            None,
        )
        .unwrap();
        (policy, generated, materialized)
    }

    #[test]
    fn m9_challenge_replay_result_binds_machine_and_normalized_oracles() {
        let (policy, generated, materialized) = m9_generated_challenge();
        let stored_requests = m9_stored_requests(&materialized);
        let failed = m5_failed_result(
            &materialized.requests[0],
            &policy,
            "mchkres_challenge_ref_001",
            "Std.Nat",
        );
        let result_store = m9_result_store(&failed, "build/results/challenge-ref.json");
        let normalized = phase8_normalize_results(
            "norm_challenge_pch_001",
            "normerr_challenge_pch_001",
            &policy,
            &materialized.request_store,
            &stored_requests,
            std::slice::from_ref(&failed),
            None,
        )
        .unwrap();
        assert_eq!(
            normalized.comparison.status,
            Phase8NormalizedComparisonStatus::AllAgreeFailed
        );
        let normalized_store = m9_normalized_store(&normalized);

        let replay = phase8_challenge_replay_aggregate(
            "chreplay_pch_001",
            &generated.manifest,
            generated.manifest.manifest_hash(),
            &policy,
            &materialized.request_store,
            &stored_requests,
            &result_store,
            std::slice::from_ref(&failed),
            Some(&normalized_store),
            std::slice::from_ref(&normalized),
            true,
            "build/challenge-replays/pch_001.json",
            None,
        )
        .unwrap();

        assert_eq!(replay.result.challenge_id, generated.manifest.challenge_id);
        assert_eq!(
            replay.result.mutated_file_hash,
            generated.manifest.mutated_certificate.file_hash
        );
        assert_eq!(
            replay.result.mutated_claimed_certificate_hash,
            generated
                .manifest
                .mutated_certificate
                .claimed_certificate_hash
        );
        assert_eq!(
            replay.result.checker_results[0].result_hash,
            failed.result_hash()
        );
        assert_eq!(
            replay.result.normalized_result_hash,
            Some(normalized.normalized_result_hash())
        );
        assert_eq!(
            replay.result.comparison_status,
            Some(Phase8NormalizedComparisonStatus::AllAgreeFailed)
        );
        assert_eq!(
            replay.result.observed_error_kinds,
            vec!["type_mismatch".to_owned()]
        );
        assert!(!replay
            .result
            .hash_input_canonical_json()
            .contains("mchkres_challenge_ref_001"));
        assert!(replay
            .result
            .canonical_json()
            .contains("mchkres_challenge_ref_001"));

        let mut renamed = replay.result.clone();
        renamed.result_id = "chreplay_pch_001_retry".to_owned();
        renamed.checker_results[0].result_id = "mchkres_ref_retry".to_owned();
        assert_eq!(renamed.result_hash(), replay.result.result_hash());
        assert_eq!(
            parse_phase8_challenge_replay_result(&replay.result.canonical_json()).unwrap(),
            replay.result
        );
        assert_eq!(
            parse_phase8_challenge_replay_store_manifest(&replay.replay_store.canonical_json())
                .unwrap(),
            replay.replay_store
        );
        let retry = phase8_challenge_replay_store_with_entry(
            Some(&replay.replay_store),
            phase8_challenge_replay_store_entry_for_result(
                &replay.result,
                "build/challenge-replays/pch_001.json",
            )
            .unwrap(),
        )
        .unwrap();
        assert!(!retry.rewrite_required);
    }

    #[test]
    fn m9_coverage_summary_records_non_failing_rejection_and_unexpected_acceptance() {
        let (policy, generated, materialized) = m9_generated_challenge();
        let stored_requests = m9_stored_requests(&materialized);
        let checked = m4_checked_result(
            &materialized.requests[0],
            &policy,
            "mchkres_challenge_ref_checked",
        );
        let result_store = m9_result_store(&checked, "build/results/challenge-ref-checked.json");
        let normalized = phase8_normalize_results(
            "norm_challenge_pch_checked",
            "normerr_challenge_pch_checked",
            &policy,
            &materialized.request_store,
            &stored_requests,
            std::slice::from_ref(&checked),
            None,
        )
        .unwrap();
        assert_eq!(
            normalized.comparison.status,
            Phase8NormalizedComparisonStatus::AllAgreeChecked
        );
        let normalized_store = m9_normalized_store(&normalized);
        let replay = phase8_challenge_replay_aggregate(
            "chreplay_pch_checked",
            &generated.manifest,
            generated.manifest.manifest_hash(),
            &policy,
            &materialized.request_store,
            &stored_requests,
            &result_store,
            std::slice::from_ref(&checked),
            Some(&normalized_store),
            std::slice::from_ref(&normalized),
            true,
            "build/challenge-replays/pch_001_checked.json",
            None,
        )
        .unwrap();
        let target_normalized = m9_target_normalized_result(&policy);
        let summary = phase8_challenge_coverage_summary(
            &policy,
            &target_normalized,
            &generated.challenge_store,
            std::slice::from_ref(&generated.manifest),
            &replay.replay_store,
            std::slice::from_ref(&replay.result),
            &result_store,
            std::slice::from_ref(&checked),
        )
        .unwrap();

        assert_eq!(summary.total_challenges, 1);
        assert_eq!(summary.replayed_challenges, 1);
        assert_eq!(summary.unexpected_acceptances, 1);
        assert_eq!(
            summary.entries[0].comparison_status,
            Phase8NormalizedComparisonStatus::AllAgreeChecked
        );
        assert!(!summary.passes_release_coverage_condition());
        assert_eq!(
            parse_phase8_challenge_coverage_summary(&summary.canonical_json()).unwrap(),
            summary
        );
        assert_eq!(
            phase8_challenge_coverage_summary_hash(&summary.canonical_json()).unwrap(),
            summary.summary_hash()
        );
    }

    #[test]
    fn m9_coverage_required_replay_requires_normalized_store_match() {
        let (policy, generated, materialized) = m9_generated_challenge();
        let stored_requests = m9_stored_requests(&materialized);
        let failed = m5_failed_result(
            &materialized.requests[0],
            &policy,
            "mchkres_challenge_ref_missing_norm",
            "Std.Nat",
        );
        let result_store = m9_result_store(&failed, "build/results/challenge-ref-missing.json");
        let empty_normalized_store = Phase8NormalizedResultStoreManifest {
            results: Vec::new(),
        };

        let err = phase8_challenge_replay_aggregate(
            "chreplay_pch_missing_norm",
            &generated.manifest,
            generated.manifest.manifest_hash(),
            &policy,
            &materialized.request_store,
            &stored_requests,
            &result_store,
            std::slice::from_ref(&failed),
            Some(&empty_normalized_store),
            &[],
            true,
            "build/challenge-replays/pch_001_missing_norm.json",
            None,
        )
        .unwrap_err();
        assert_eq!(err.reason_code.as_ref(), "normalized_result_not_found");
    }

    fn m10_single_failed_diagnostic_context() -> (
        Phase8RunnerPolicy,
        Phase8MachineCheckResult,
        Phase8NormalizedCheckResult,
        Phase8CiDiagnosticTargetContext,
    ) {
        let (request, policy) = m3_request_and_policy();
        let stored = Phase8StoredMachineCheckRequest {
            path: "build/check-requests/Std.Nat.reference.json".to_owned(),
            file_hash: phase8_file_hash(request.canonical_json().as_bytes()),
            request: request.clone(),
        };
        let request_store = m4_request_store_manifest(std::slice::from_ref(&stored));
        let failed = m5_failed_result(&request, &policy, "mchkres_m10_failed", "Std.Nat");
        let normalized = phase8_normalize_results(
            "norm_Std.Nat_m10_failed",
            "normerr_Std.Nat_m10_failed",
            &policy,
            &request_store,
            std::slice::from_ref(&stored),
            std::slice::from_ref(&failed),
            None,
        )
        .unwrap();
        assert_eq!(
            normalized.comparison.status,
            Phase8NormalizedComparisonStatus::AllAgreeFailed
        );
        let context = Phase8CiDiagnosticTargetContext {
            artifact: normalized.clone(),
            normalizer_machine_results: vec![failed.clone()],
        };
        (policy, failed, normalized, context)
    }

    fn m10_required_release_policy(
        runner_policy: &Phase8RunnerPolicy,
        input_policy: &Phase8AiAuditInputPolicy,
    ) -> Phase8ReleasePolicy {
        let mut release_policy = m7_release_policy(Phase8ReleaseMode::Release);
        release_policy.runner_policy_hash = runner_policy.policy_hash();
        release_policy.challenge_runner_policy_hash = runner_policy.policy_hash();
        release_policy.ai_triage = Phase8ReleasePolicyAiTriage {
            enabled: true,
            required: true,
            input_policy_hash: Some(input_policy.input_policy_hash()),
        };
        release_policy
    }

    fn m10_resolved_sidecar(path: &str, source_json: &str) -> Phase8ResolvedAiAuditSidecarEntry {
        Phase8ResolvedAiAuditSidecarEntry {
            path: path.to_owned(),
            file_hash: phase8_file_hash(source_json.as_bytes()),
            artifact: parse_phase8_ai_audit_sidecar(source_json).unwrap(),
        }
    }

    fn m10_resolved_validation(
        path: &str,
        artifact: Phase8AuditSidecarValidationResult,
    ) -> Phase8ResolvedAuditSidecarValidationEntry {
        Phase8ResolvedAuditSidecarValidationEntry {
            path: path.to_owned(),
            file_hash: phase8_file_hash(artifact.canonical_json().as_bytes()),
            artifact,
        }
    }

    fn m10_resolved_diagnostic(
        path: &str,
        artifact: Phase8AiSidecarDiagnosticResult,
    ) -> Phase8ResolvedAiSidecarDiagnosticResultEntry {
        Phase8ResolvedAiSidecarDiagnosticResultEntry {
            path: path.to_owned(),
            file_hash: phase8_file_hash(artifact.canonical_json().as_bytes()),
            artifact,
        }
    }

    #[test]
    fn m10_required_machine_diagnostic_matches_saved_canonical_result() {
        let (runner_policy, failed, normalized, context) = m10_single_failed_diagnostic_context();
        let input_policy_json = m6_input_policy_json(&["module", "status"], false, false);
        let input_policy = parse_phase8_ai_audit_input_policy(&input_policy_json).unwrap();
        let release_policy = m10_required_release_policy(&runner_policy, &input_policy);
        let targets = phase8_required_ai_sidecar_diagnostic_targets(&release_policy, &normalized);
        assert_eq!(targets.len(), 1);
        assert!(matches!(
            targets[0],
            Phase8RequiredAiSidecarDiagnosticTarget::MachineResult { .. }
        ));
        assert!(
            phase8_release_target_has_required_ai_sidecar_diagnostic_targets(
                &release_policy,
                &normalized
            )
        );

        let sidecar_json = m6_machine_sidecar_with_prompt_hash(&failed, &input_policy, "");
        let sidecar_entry =
            m10_resolved_sidecar("build/ai-sidecars/Std.Nat.m10.failed.json", &sidecar_json);
        let validation = phase8_validate_ai_audit_sidecar_cross_artifact(
            &sidecar_json,
            &input_policy_json,
            input_policy.input_policy_hash(),
            std::slice::from_ref(&failed),
            &[],
        );
        assert_eq!(validation.status, Phase8AuditSidecarValidationStatus::Valid);
        let validation_entry = m10_resolved_validation(
            "build/ai-sidecars/Std.Nat.m10.failed.validation.json",
            validation,
        );

        let without_saved = evaluate_required_ai_sidecar_diagnostics(
            &release_policy,
            std::slice::from_ref(&context),
            std::slice::from_ref(&sidecar_entry),
            std::slice::from_ref(&validation_entry),
            &[],
        )
        .unwrap();
        assert_eq!(
            without_saved.recomputed_results[0].status,
            Phase8AiSidecarDiagnosticStatus::Passed
        );
        assert_eq!(without_saved.recomputed_results[0].target_count, 1);
        assert!(!without_saved.recomputed_results[0]
            .canonical_json()
            .contains("\"result_hash\""));
        assert_eq!(
            without_saved.pass_failure.as_ref().unwrap().reason_code,
            Phase8AiSidecarDiagnosticPassFailureReasonCode::RequiredAiSidecarDiagnosticMissing
        );

        let saved_entry = m10_resolved_diagnostic(
            "build/ai-sidecar-diagnostics/Std.Nat.m10.failed.json",
            without_saved.recomputed_results[0].clone(),
        );
        let with_saved = evaluate_required_ai_sidecar_diagnostics(
            &release_policy,
            &[context],
            &[sidecar_entry],
            &[validation_entry],
            &[saved_entry],
        )
        .unwrap();
        assert_eq!(with_saved.pass_failure, None);
    }

    #[test]
    fn m10_disagreement_targets_and_saved_target_count_mismatch_are_deterministic() {
        let runner_policy = parse_phase8_runner_policy(&m4_runner_policy_json()).unwrap();
        let fast = m4_stored_request(&runner_policy, "fast-kernel", "mchkreq_fast");
        let reference = m4_stored_request(&runner_policy, "reference", "mchkreq_ref");
        let external = m4_stored_request(&runner_policy, "external", "mchkreq_ext");
        let stored_requests = vec![fast.clone(), reference.clone(), external.clone()];
        let request_store = m4_request_store_manifest(&stored_requests);
        let fast_result = m4_checked_result(&fast.request, &runner_policy, "mchkres_fast");
        let reference_result =
            m5_failed_result(&reference.request, &runner_policy, "mchkres_ref", "Std.Nat");
        let external_result = m4_checked_result(&external.request, &runner_policy, "mchkres_ext");
        let machine_results = vec![
            fast_result.clone(),
            reference_result.clone(),
            external_result.clone(),
        ];
        let normalized = phase8_normalize_results(
            "norm_Std.Nat_m10_disagreement",
            "normerr_Std.Nat_m10_disagreement",
            &runner_policy,
            &request_store,
            &stored_requests,
            &machine_results,
            None,
        )
        .unwrap();
        assert_eq!(
            normalized.comparison.status,
            Phase8NormalizedComparisonStatus::Disagreement
        );
        let input_policy = parse_phase8_ai_audit_input_policy(&m6_input_policy_json(
            &["module", "status"],
            false,
            false,
        ))
        .unwrap();
        let release_policy = m10_required_release_policy(&runner_policy, &input_policy);
        let targets = phase8_ai_sidecar_diagnostic_targets_from_normalized_result(&normalized);
        assert_eq!(targets.len(), 2);
        assert!(matches!(
            targets[0],
            Phase8RequiredAiSidecarDiagnosticTarget::MachineResult { .. }
        ));
        assert!(matches!(
            targets[1],
            Phase8RequiredAiSidecarDiagnosticTarget::NormalizedComparison { .. }
        ));

        let saved_wrong_count = Phase8AiSidecarDiagnosticResult::passed(
            release_policy.policy_hash(),
            input_policy.input_policy_hash(),
            normalized.normalized_result_hash(),
            1,
        );
        let err = evaluate_required_ai_sidecar_diagnostics(
            &release_policy,
            &[Phase8CiDiagnosticTargetContext {
                artifact: normalized,
                normalizer_machine_results: machine_results,
            }],
            &[],
            &[],
            &[m10_resolved_diagnostic(
                "build/ai-sidecar-diagnostics/Std.Nat.bad-count.json",
                saved_wrong_count,
            )],
        )
        .unwrap_err();
        assert_eq!(
            err.reason_code,
            Phase8AiSidecarDiagnosticEvaluationFailureReasonCode::AiSidecarDiagnosticTargetCountMismatch
        );
    }

    #[test]
    fn m10_wrong_retry_source_fails_before_saved_result_can_pass() {
        let (runner_policy, failed, normalized, context) = m10_single_failed_diagnostic_context();
        let input_policy = parse_phase8_ai_audit_input_policy(&m6_input_policy_json(
            &["module", "status"],
            false,
            false,
        ))
        .unwrap();
        let release_policy = m10_required_release_policy(&runner_policy, &input_policy);
        let wrong_retry_sidecar_json = m6_machine_sidecar_json(
            &failed,
            &input_policy,
            test_hash(250),
            failed.result_hash(),
            failed.request_hash,
            test_hash(251),
            "",
        );
        let sidecar_entry = m10_resolved_sidecar(
            "build/ai-sidecars/Std.Nat.m10.wrong-retry.json",
            &wrong_retry_sidecar_json,
        );
        let validation = Phase8AuditSidecarValidationResult::valid(
            Phase8AuditSidecarValidationMode::CrossArtifact,
            sidecar_entry.file_hash,
            Some(input_policy.input_policy_hash()),
            Some(&sidecar_entry.artifact),
        );
        let validation_entry = m10_resolved_validation(
            "build/ai-sidecars/Std.Nat.m10.wrong-retry.validation.json",
            validation,
        );
        let evaluation = evaluate_required_ai_sidecar_diagnostics(
            &release_policy,
            std::slice::from_ref(&context),
            std::slice::from_ref(&sidecar_entry),
            std::slice::from_ref(&validation_entry),
            &[],
        )
        .unwrap();
        let error = evaluation.recomputed_results[0].error.as_ref().unwrap();
        assert_eq!(
            error.reason_code,
            Phase8AiSidecarDiagnosticFailureReasonCode::RequiredAiSidecarSourceMismatch
        );

        let saved_claiming_pass = Phase8AiSidecarDiagnosticResult::passed(
            release_policy.policy_hash(),
            input_policy.input_policy_hash(),
            normalized.normalized_result_hash(),
            1,
        );
        let with_saved = evaluate_required_ai_sidecar_diagnostics(
            &release_policy,
            &[context],
            &[sidecar_entry],
            &[validation_entry],
            &[m10_resolved_diagnostic(
                "build/ai-sidecar-diagnostics/Std.Nat.claimed-pass.json",
                saved_claiming_pass,
            )],
        )
        .unwrap();
        assert_eq!(
            with_saved.pass_failure.as_ref().unwrap().reason_code,
            Phase8AiSidecarDiagnosticPassFailureReasonCode::RequiredAiSidecarDiagnosticCanonicalMismatch
        );
    }

    fn m11_input(
        kind: Phase8ReleaseBundleArtifactKind,
        path: &str,
        source: &str,
        hashes: &[(&str, Hash)],
    ) -> Phase8ReleaseBundleStagingInput {
        Phase8ReleaseBundleStagingInput {
            kind,
            path: path.to_owned(),
            file_hash: phase8_file_hash(source.as_bytes()),
            hashes: hashes
                .iter()
                .map(|(field, hash)| ((*field).to_owned(), *hash))
                .collect(),
        }
    }

    fn m11_plan(
        phase: Phase8ReleaseBundleStagingPhase,
        mut inputs: Vec<Phase8ReleaseBundleStagingInput>,
    ) -> Phase8ReleaseBundleStagingPlan {
        inputs.sort_by(|left, right| {
            (left.kind.as_str(), left.path.as_str(), left.file_hash).cmp(&(
                right.kind.as_str(),
                right.path.as_str(),
                right.file_hash,
            ))
        });
        Phase8ReleaseBundleStagingPlan {
            phase,
            bundle_root: "dist/release-bundle".to_owned(),
            inputs,
        }
    }

    #[test]
    fn m11_store_phase_stages_manifest_entries_and_generates_bundle_local_stores() {
        let policy = parse_phase8_runner_policy(&m4_runner_policy_json()).unwrap();
        let stored = m4_stored_request(&policy, "fast-kernel", "mchkreq_m11_store");
        let request_store = m4_request_store_manifest(std::slice::from_ref(&stored));
        let result = m4_checked_result(&stored.request, &policy, "mchkres_m11_store");
        let result_path = "build/results/m11-store-fast.json";
        let result_store = m9_result_store(&result, result_path);
        let normalized = phase8_normalize_results(
            "norm_Std.Nat_m11_store",
            "normerr_Std.Nat_m11_store",
            &policy,
            &request_store,
            std::slice::from_ref(&stored),
            std::slice::from_ref(&result),
            None,
        )
        .unwrap();
        let normalized_path = "build/normalized/m11-store.json";
        let normalized_entry =
            phase8_normalized_result_store_entry_for_result(&normalized, normalized_path).unwrap();
        let normalized_store = phase8_normalized_result_store_with_entry(None, normalized_entry)
            .unwrap()
            .manifest;
        let challenge_store = Phase8ChallengeOutputStoreManifest { entries: vec![] };

        let request_store_json = request_store.canonical_json();
        let result_store_json = result_store.canonical_json();
        let normalized_store_json = normalized_store.canonical_json();
        let challenge_store_json = challenge_store.canonical_json();
        let plan = m11_plan(
            Phase8ReleaseBundleStagingPhase::Store,
            vec![
                m11_input(
                    Phase8ReleaseBundleArtifactKind::ChallengeOutputStoreManifest,
                    "build/challenges/store.json",
                    &challenge_store_json,
                    &[(
                        "manifest_hash",
                        phase8_file_hash(challenge_store_json.as_bytes()),
                    )],
                ),
                m11_input(
                    Phase8ReleaseBundleArtifactKind::MachineResultStoreManifest,
                    "build/stores/result-store.json",
                    &result_store_json,
                    &[(
                        "manifest_hash",
                        phase8_file_hash(result_store_json.as_bytes()),
                    )],
                ),
                m11_input(
                    Phase8ReleaseBundleArtifactKind::NormalizedResultStoreManifest,
                    "build/stores/normalized-store.json",
                    &normalized_store_json,
                    &[(
                        "manifest_hash",
                        phase8_file_hash(normalized_store_json.as_bytes()),
                    )],
                ),
                m11_input(
                    Phase8ReleaseBundleArtifactKind::RequestStoreManifest,
                    "build/stores/request-store.json",
                    &request_store_json,
                    &[(
                        "manifest_hash",
                        phase8_file_hash(request_store_json.as_bytes()),
                    )],
                ),
            ],
        );
        let plan_json = plan.canonical_json();
        let mut workspace_files = BTreeMap::new();
        workspace_files.insert(
            stored.path.clone(),
            stored.request.canonical_json().into_bytes(),
        );
        workspace_files.insert(result_path.to_owned(), result.canonical_json().into_bytes());
        workspace_files.insert(
            normalized_path.to_owned(),
            normalized.canonical_json().into_bytes(),
        );
        workspace_files.insert(
            "build/challenges/store.json".to_owned(),
            challenge_store_json.into_bytes(),
        );
        workspace_files.insert(
            "build/stores/result-store.json".to_owned(),
            result_store_json.into_bytes(),
        );
        workspace_files.insert(
            "build/stores/normalized-store.json".to_owned(),
            normalized_store_json.into_bytes(),
        );
        workspace_files.insert(
            "build/stores/request-store.json".to_owned(),
            request_store_json.into_bytes(),
        );

        let staged = phase8_release_stage_bundle_inputs(
            Phase8ReleaseBundleStagingPhase::Store,
            "dist/release-bundle",
            "build/staging-plan.json",
            phase8_file_hash(plan_json.as_bytes()),
            &plan_json,
            &workspace_files,
            None,
        )
        .unwrap();

        assert_eq!(staged.result.staged_artifacts.len(), 4);
        assert_eq!(staged.result.store_manifests.len(), 3);
        assert!(!staged.result.canonical_json().contains("result_hash"));
        assert!(staged.result.staged_artifacts.iter().any(|artifact| {
            artifact.kind == Phase8ReleaseBundleArtifactKind::MachineCheckRequest
                && artifact
                    .path
                    .starts_with("artifacts/machine_check_request/")
        }));
        assert!(staged.result.staged_artifacts.iter().all(|artifact| {
            !artifact.path.starts_with("build/")
                && !artifact.path.contains("request_store_manifest")
                && !artifact.path.contains("machine_result_store_manifest")
                && !artifact.path.contains("normalized_result_store_manifest")
        }));

        let request_manifest_entry = staged
            .result
            .store_manifests
            .iter()
            .find(|manifest| manifest.kind == Phase8ReleaseBundleArtifactKind::RequestStoreManifest)
            .unwrap();
        let request_manifest_bytes = staged
            .files
            .iter()
            .find(|file| file.path == request_manifest_entry.path)
            .unwrap();
        let request_manifest_json = std::str::from_utf8(&request_manifest_bytes.bytes).unwrap();
        let generated_request_store =
            parse_phase8_request_store_manifest(request_manifest_json).unwrap();
        assert_eq!(generated_request_store.requests.len(), 1);
        assert!(generated_request_store.requests[0]
            .path
            .starts_with("artifacts/machine_check_request/"));
    }

    #[test]
    fn m11_final_phase_rejects_store_inputs_and_non_bundle_error_results() {
        let normalized_input = Phase8ReleaseBundleStagingInput {
            kind: Phase8ReleaseBundleArtifactKind::NormalizedCheckResult,
            path: "build/normalized/m11.json".to_owned(),
            file_hash: test_hash(1),
            hashes: BTreeMap::from([
                ("artifact_hash".to_owned(), test_hash(2)),
                ("normalized_result_hash".to_owned(), test_hash(3)),
            ]),
        };
        let plan_json = m11_plan(
            Phase8ReleaseBundleStagingPhase::Final,
            vec![normalized_input],
        )
        .canonical_json();
        let err = phase8_release_stage_bundle_inputs(
            Phase8ReleaseBundleStagingPhase::Final,
            "dist/release-bundle",
            "build/staging-plan.json",
            phase8_file_hash(plan_json.as_bytes()),
            &plan_json,
            &BTreeMap::new(),
            None,
        )
        .unwrap_err();
        assert_eq!(err.reason_code.as_ref(), "input_schema_invalid");
        assert_eq!(err.field.as_deref(), Some("inputs[0].kind"));
        assert_eq!(
            err.expected_value.as_deref(),
            Some("allowed_kind_for_phase:final")
        );

        let invalid_kind_plan = format!(
            r#"{{"bundle_root":"dist/release-bundle","inputs":[{{"file_hash":"{}","hashes":{{}},"kind":"machine_check_request_error_result","path":"build/errors/request.json"}}],"phase":"final","schema":"{}"}}"#,
            hash_wire(4),
            PHASE8_RELEASE_BUNDLE_STAGING_PLAN_SCHEMA
        );
        let err = parse_phase8_release_bundle_staging_plan(&invalid_kind_plan).unwrap_err();
        assert_eq!(err.reason_code.as_ref(), "input_schema_invalid");
        assert_eq!(err.field.as_deref(), Some("inputs[0].kind"));
        assert_eq!(err.actual_value.as_deref(), Some("invalid_enum"));
    }

    #[test]
    fn m11_plan_error_shapes_match_staging_contract() {
        let sidecar_input = Phase8ReleaseBundleStagingInput {
            kind: Phase8ReleaseBundleArtifactKind::AiAuditSidecar,
            path: "build/ai-sidecars/m11.json".to_owned(),
            file_hash: test_hash(1),
            hashes: BTreeMap::from([("unexpected_hash".to_owned(), test_hash(2))]),
        };
        let err = parse_phase8_release_bundle_staging_plan(
            &m11_plan(Phase8ReleaseBundleStagingPhase::Final, vec![sidecar_input]).canonical_json(),
        )
        .unwrap_err();
        assert_eq!(err.reason_code.as_ref(), "input_schema_invalid");
        assert_eq!(err.field.as_deref(), Some("inputs[0].hashes"));
        assert_eq!(err.expected_value.as_deref(), Some("empty_object"));
        assert_eq!(err.actual_value.as_deref(), Some("non_empty"));

        let invalid_path_plan = format!(
            r#"{{"bundle_root":"dist/release-bundle","inputs":[{{"file_hash":"{}","hashes":{{"policy_hash":"{}"}},"kind":"release_policy","path":"../policy.json"}}],"phase":"final","schema":"{}"}}"#,
            hash_wire(1),
            hash_wire(2),
            PHASE8_RELEASE_BUNDLE_STAGING_PLAN_SCHEMA
        );
        let err = parse_phase8_release_bundle_staging_plan(&invalid_path_plan).unwrap_err();
        assert_eq!(err.reason_code.as_ref(), "input_schema_invalid");
        assert_eq!(err.field.as_deref(), Some("inputs[0].path"));
        assert_eq!(err.actual_value.as_deref(), Some("invalid_path"));

        let challenge_store_json =
            Phase8ChallengeOutputStoreManifest { entries: vec![] }.canonical_json();
        let err = parse_phase8_release_bundle_staging_plan(
            &m11_plan(
                Phase8ReleaseBundleStagingPhase::Store,
                vec![m11_input(
                    Phase8ReleaseBundleArtifactKind::ChallengeOutputStoreManifest,
                    "build/challenges/store.json",
                    &challenge_store_json,
                    &[(
                        "manifest_hash",
                        phase8_file_hash(challenge_store_json.as_bytes()),
                    )],
                )],
            )
            .canonical_json(),
        )
        .unwrap_err();
        assert_eq!(err.reason_code.as_ref(), "input_schema_invalid");
        assert_eq!(err.field.as_deref(), Some("inputs[]"));
        assert_eq!(
            err.expected_value.as_deref(),
            Some("one_or_more:request_store_manifest")
        );
        assert_eq!(
            err.actual_value.as_deref(),
            Some("missing:request_store_manifest")
        );

        let checker_identity_json = "{}";
        let checker_identity_input = m11_input(
            Phase8ReleaseBundleArtifactKind::CheckerIdentityManifest,
            "build/checkers/identity.json",
            checker_identity_json,
            &[(
                "manifest_hash",
                phase8_file_hash(checker_identity_json.as_bytes()),
            )],
        );
        let plan_json = m11_plan(
            Phase8ReleaseBundleStagingPhase::Final,
            vec![checker_identity_input.clone()],
        )
        .canonical_json();
        let workspace_files = BTreeMap::from([(
            checker_identity_input.path.clone(),
            checker_identity_json.as_bytes().to_vec(),
        )]);
        let err = phase8_release_stage_bundle_inputs(
            Phase8ReleaseBundleStagingPhase::Final,
            "dist/release-bundle",
            "build/staging-plan.json",
            phase8_file_hash(plan_json.as_bytes()),
            &plan_json,
            &workspace_files,
            None,
        )
        .unwrap_err();
        assert_eq!(err.reason_code.as_ref(), "input_schema_invalid");
        assert_eq!(err.field.as_deref(), Some("inputs[0].artifact.schema"));
    }

    #[test]
    fn m11_store_entry_errors_are_plan_input_prefixed() {
        let request_store = Phase8RequestStoreManifest {
            requests: vec![Phase8RequestStoreEntry {
                request_hash: test_hash(10),
                path: "build/missing-request.json".to_owned(),
                file_hash: test_hash(11),
            }],
        };
        let result_store = Phase8MachineResultStoreManifest { results: vec![] };
        let normalized_store = Phase8NormalizedResultStoreManifest { results: vec![] };
        let challenge_store = Phase8ChallengeOutputStoreManifest { entries: vec![] };

        let request_store_json = request_store.canonical_json();
        let result_store_json = result_store.canonical_json();
        let normalized_store_json = normalized_store.canonical_json();
        let challenge_store_json = challenge_store.canonical_json();
        let plan = m11_plan(
            Phase8ReleaseBundleStagingPhase::Store,
            vec![
                m11_input(
                    Phase8ReleaseBundleArtifactKind::ChallengeOutputStoreManifest,
                    "build/challenges/store.json",
                    &challenge_store_json,
                    &[(
                        "manifest_hash",
                        phase8_file_hash(challenge_store_json.as_bytes()),
                    )],
                ),
                m11_input(
                    Phase8ReleaseBundleArtifactKind::MachineResultStoreManifest,
                    "build/stores/result-store.json",
                    &result_store_json,
                    &[(
                        "manifest_hash",
                        phase8_file_hash(result_store_json.as_bytes()),
                    )],
                ),
                m11_input(
                    Phase8ReleaseBundleArtifactKind::NormalizedResultStoreManifest,
                    "build/stores/normalized-store.json",
                    &normalized_store_json,
                    &[(
                        "manifest_hash",
                        phase8_file_hash(normalized_store_json.as_bytes()),
                    )],
                ),
                m11_input(
                    Phase8ReleaseBundleArtifactKind::RequestStoreManifest,
                    "build/stores/request-store.json",
                    &request_store_json,
                    &[(
                        "manifest_hash",
                        phase8_file_hash(request_store_json.as_bytes()),
                    )],
                ),
            ],
        );
        let plan_json = plan.canonical_json();
        let workspace_files = BTreeMap::from([
            (
                "build/challenges/store.json".to_owned(),
                challenge_store_json.into_bytes(),
            ),
            (
                "build/stores/result-store.json".to_owned(),
                result_store_json.into_bytes(),
            ),
            (
                "build/stores/normalized-store.json".to_owned(),
                normalized_store_json.into_bytes(),
            ),
            (
                "build/stores/request-store.json".to_owned(),
                request_store_json.into_bytes(),
            ),
        ]);

        let err = phase8_release_stage_bundle_inputs(
            Phase8ReleaseBundleStagingPhase::Store,
            "dist/release-bundle",
            "build/staging-plan.json",
            phase8_file_hash(plan_json.as_bytes()),
            &plan_json,
            &workspace_files,
            None,
        )
        .unwrap_err();
        assert_eq!(err.reason_code.as_ref(), "input_store_entry_invalid");
        assert_eq!(
            err.field.as_deref(),
            Some("inputs[3].store.request_store.requests[0].path")
        );
    }

    #[test]
    fn m11_final_phase_adopts_exact_existing_output_and_rejects_conflict() {
        let release_policy = m7_release_policy(Phase8ReleaseMode::Release);
        let release_policy_json = release_policy.canonical_json();
        let input = m11_input(
            Phase8ReleaseBundleArtifactKind::ReleasePolicy,
            "policies/release-policy.json",
            &release_policy_json,
            &[("policy_hash", release_policy.policy_hash())],
        );
        let plan = m11_plan(Phase8ReleaseBundleStagingPhase::Final, vec![input.clone()]);
        let plan_json = plan.canonical_json();
        let mut workspace_files = BTreeMap::new();
        workspace_files.insert(input.path.clone(), release_policy_json.clone().into_bytes());
        let target_path = phase8_release_bundle_artifact_path(input.kind, input.file_hash);
        let mut existing = BTreeMap::new();
        existing.insert(
            target_path.clone(),
            release_policy_json.clone().into_bytes(),
        );
        let staged = phase8_release_stage_bundle_inputs(
            Phase8ReleaseBundleStagingPhase::Final,
            "dist/release-bundle",
            "build/staging-plan.json",
            phase8_file_hash(plan_json.as_bytes()),
            &plan_json,
            &workspace_files,
            Some(&existing),
        )
        .unwrap();
        assert_eq!(staged.result.staged_artifacts[0].path, target_path);

        existing.insert(target_path, b"different".to_vec());
        let err = phase8_release_stage_bundle_inputs(
            Phase8ReleaseBundleStagingPhase::Final,
            "dist/release-bundle",
            "build/staging-plan.json",
            phase8_file_hash(plan_json.as_bytes()),
            &plan_json,
            &workspace_files,
            Some(&existing),
        )
        .unwrap_err();
        assert_eq!(err.reason_code.as_ref(), "output_path_conflict");
    }
}
