//! Phase 5 API substrate.
//!
//! This crate is intentionally outside the trusted kernel. It handles wire JSON
//! decoding and request-shape validation for machine-facing endpoints.

mod adapter;
mod callable;
mod current;
mod diagnostic;
mod json;
mod phase7;
mod phase8;
mod projection;
mod prompt;
mod renderer;
mod replay;
mod search;
mod session;
mod snapshot;
mod std_library;
mod tactic;
mod types;
mod validation;
mod verify;

pub use adapter::{
    map_phase3_diagnostic_kind, map_phase4_diagnostic_kind,
    phase4_extract_closed_machine_theorem_decl, phase4_machine_tactic_result_error,
    phase4_run_machine_tactic, phase4_run_machine_tactic_with_budget, phase4_start_machine_proof,
    phase4_validate_machine_tactic_candidate, MachineApiDiagnosticPhase,
    MachineApiDiagnosticProjection, MachineApiTacticKind, Phase4AdapterError, Phase4AdapterResult,
    Phase4ExtractedTheorem, Phase4StartProofOutput, Phase4TacticRunOutput, Phase4ValidatedTactic,
    Phase5UpstreamDiagnostic,
};
pub use callable::{
    build_machine_surface_callable_interface_table,
    build_machine_surface_callable_interface_table_from_parts,
    MachineSurfaceCallableInterfaceBuildError,
};
pub use current::{
    project_checked_current_decl_context, project_checked_current_decl_context_with_kernel_profile,
    CheckedCurrentDeclPackageInput, CheckedCurrentDeclProjectionError, CurrentDeclDependencyEntry,
    CurrentDeclDependencyReport, CurrentDeclIndexEntry, CurrentGeneratedDeclEntry,
    CurrentGeneratedDeclKind, MachineAxiomRefWire, MachineCheckedCurrentDeclContext,
    MachineCheckedDeclSignature, MachineDependencyRefWire,
};
pub use diagnostic::{
    machine_api_diagnostic_canonical_bytes, machine_api_diagnostic_hash,
    MachineApiDiagnosticCanonicalizationError,
};
pub use json::{
    JsonDocument, JsonMember, JsonParseError, JsonParseErrorKind, JsonParseLimits, JsonSpan,
    JsonValue, JsonValueKind,
};
pub use phase7::{
    filter_phase7_candidate_envelopes, load_phase7_initial_snapshot,
    parse_phase7_mvp_controller_config, phase7_assign_candidate_ids, phase7_build_replay_plan,
    phase7_builtin_candidate_envelopes, phase7_candidate_cost_estimate, phase7_candidate_envelope,
    phase7_candidate_forbidden_token, phase7_candidate_hash_mismatch,
    phase7_candidate_payload_hash, phase7_candidate_payload_json, phase7_cap_batch_policy,
    phase7_evaluate_tactic_batch_response, phase7_expected_effect, phase7_fresh_intro_name,
    phase7_goal_summaries, phase7_minimize_replay_plan, phase7_mvp_candidate_envelopes,
    phase7_mvp_candidate_generation, phase7_mvp_premise_query_json,
    phase7_negative_training_identity, phase7_negative_training_identity_hash,
    phase7_negative_training_identity_json, phase7_positive_training_identity,
    phase7_positive_training_identity_hash, phase7_positive_training_identity_json,
    phase7_premise_cache_entries, phase7_premise_retrieval_from_search_ok, phase7_premise_usages,
    phase7_rank_and_dedupe_candidate_envelopes, phase7_rank_filter_and_dedupe_candidate_envelopes,
    phase7_repair_depth_of, phase7_replay_plan_json, phase7_replay_request_json,
    phase7_replay_step_json, phase7_retrieval_cache_key, phase7_rule_based_repair_action,
    phase7_run_mvp_search, phase7_run_tactic_batch, phase7_search_node_best_partial_key,
    phase7_search_node_priority_key, phase7_snapshot_get_request_json,
    phase7_suggested_candidate_envelopes, phase7_tactic_batch_request_json,
    phase7_training_trace_record_json, phase7_training_trace_records_json,
    phase7_verify_request_json, retrieve_phase7_premises, select_phase7_goal,
    validate_phase7_mvp_controller_config, Phase7AcceptedCandidateFailure,
    Phase7AcceptedCandidateFailureRecord, Phase7AssignedCandidate, Phase7BatchEvaluation,
    Phase7BestPartialKey, Phase7BuiltinKind, Phase7CandidateCostEstimate, Phase7CandidateCostRisk,
    Phase7CandidateEnvelope, Phase7CandidateFilterError, Phase7CandidateFilterResult,
    Phase7CandidateMetadata, Phase7CandidateRankMetadata, Phase7CandidateRepairMetadata,
    Phase7CandidateSource, Phase7DeferredCandidate, Phase7DeferredCandidateDropReason,
    Phase7ExpectedEffect, Phase7FakeMachineApiClient, Phase7ForbiddenToken,
    Phase7ForbiddenTokenClass, Phase7GoalSummary, Phase7InitialSnapshot,
    Phase7LocalMachineApiClient, Phase7MachineApiCall, Phase7MachineApiClient,
    Phase7MachineApiEndpointKind, Phase7MachineApiError, Phase7MachineApiResult,
    Phase7MachineControllerError, Phase7MachineControllerErrorKind, Phase7MachineControllerResult,
    Phase7MinimizationResult, Phase7MinimizationStats, Phase7MvpControllerConfig,
    Phase7NegativeTrainingIdentity, Phase7NodeId, Phase7NodeStatus,
    Phase7NonAcceptedCandidateError, Phase7PendingCandidate, Phase7PositiveTrainingIdentity,
    Phase7PremiseCacheEntry, Phase7PremiseQueryRequest, Phase7PremiseRef, Phase7PremiseRetrieval,
    Phase7PremiseUsage, Phase7RejectedCandidateEnvelope, Phase7RepairCandidateOutput,
    Phase7RepairChainStopReason, Phase7ReplayPlan, Phase7ReplayStep, Phase7ReplayStepEdit,
    Phase7RetrievalCacheKey, Phase7RuleBasedRepair, Phase7RuleBasedRepairAction,
    Phase7SchedulerStop, Phase7Score, Phase7SearchBudget, Phase7SearchBudgetLimit,
    Phase7SearchFailure, Phase7SearchFailureReason, Phase7SearchInput, Phase7SearchNode,
    Phase7SearchPriorityKey, Phase7SearchResult, Phase7SearchStats, Phase7SearchTraceEvent,
    Phase7SearchTraceEventKind, Phase7SnapshotGetRequest, Phase7SuccessfulCandidateTransition,
    Phase7TacticBatchRequest, Phase7TacticBatchRunError, Phase7TraceEvent,
    Phase7TrainingTraceCandidate, Phase7TrainingTraceRecord, Phase7TrustFlags, Phase7VerifiedProof,
    PHASE7_TRAINING_TRACE_SCHEMA,
};
pub use phase8::{
    phase8_canonical_json_bytes, phase8_canonical_json_object_excluding_top_level_fields,
    phase8_canonical_json_string, phase8_canonical_json_value_bytes, phase8_file_hash,
    phase8_first_forbidden_verification_input_field, phase8_forbids_verification_input_field,
    phase8_hash_json_object_excluding_top_level_fields, phase8_hash_usage_rule,
    phase8_json_artifact_hash_from_source, phase8_rfc8785_object_key_cmp, phase8_sha256,
    validate_phase8_closed_object, Phase8ApiError, Phase8ApiErrorReasonCode,
    Phase8ArtifactClassification, Phase8ArtifactKind, Phase8CanonicalJsonError, Phase8ClosedSchema,
    Phase8CommandError, Phase8CommandName, Phase8FieldSpec, Phase8HashInputKind,
    Phase8HashUsageKind, Phase8HashUsageRule, Phase8JsonFieldType, Phase8JsonPath,
    Phase8JsonPathElement, Phase8MachineCheckRequestErrorReasonCode,
    Phase8MachineCheckRequestErrorResult, Phase8NormalizeErrorReasonCode,
    Phase8NormalizeErrorResult, Phase8PipelineErrorKind, Phase8SchemaError,
    Phase8SchemaErrorReason, Phase8StructuredError, Phase8ValidatedObject, PHASE8_API_ERROR_SCHEMA,
    PHASE8_COMMAND_ERROR_SCHEMA, PHASE8_FORBIDDEN_VERIFICATION_INPUT_FIELDS,
    PHASE8_MACHINE_CHECK_REQUEST_ERROR_RESULT_SCHEMA, PHASE8_NORMALIZE_ERROR_RESULT_SCHEMA,
};
pub use projection::{
    project_import_certificate_context, GeneratedDeclKind, ImportProjectionError,
    MachineImportCertificateContext, VerifiedImportDeclIndexEntry,
    VerifiedImportGeneratedDeclEntry, VerifiedImportGeneratedDeclPayload, VerifiedImportKey,
    VerifiedModuleCertificateInput, VerifiedModuleContextEntry,
};
pub use prompt::{
    build_machine_prompt_payload, build_machine_prompt_payload_in_sessions,
    parse_machine_prompt_payload_request, FailedCandidateErrorKind, FailedCandidatePromptItem,
    MachinePromptGoal, MachinePromptLocal, MachinePromptPayloadError, MachinePromptPayloadOkFields,
    MachinePromptPayloadRequest, MachinePromptPayloadResponse, MachinePromptPremise,
    MachinePromptPremiseSelection,
};
pub use renderer::{
    render_machine_expr_source, render_machine_expr_view, renderer_qa_round_trip, LocalId,
    MachineDisplayRenderScope, MachineDisplayRenderScopeEntry, MachineExprRendererContext,
    MachineExprRendererError, MachineExprView, MachineGlobalRefView,
    Phase5ResolvedDisplayCoreRefOwner,
};
pub use replay::{
    parse_machine_replay_request, run_machine_replay_request,
    run_machine_replay_request_in_sessions, MachineReplayError, MachineReplayOkFields,
    MachineReplayPlan, MachineReplayRequest, MachineReplayResponse, MachineReplayStep,
};
pub use search::{
    parse_machine_theorem_search_request, search_machine_theorems_for_goal,
    search_machine_theorems_for_goal_in_sessions, MachineAllowedModulesFilter,
    MachineSuggestedCandidate, MachineSuggestedCandidateStatus, MachineTheoremFilters,
    MachineTheoremGlobalRef, MachineTheoremMode, MachineTheoremSearchError,
    MachineTheoremSearchOkFields, MachineTheoremSearchRequest, MachineTheoremSearchResponse,
    MachineTheoremSearchResult, MachineTheoremStatement,
};
pub use session::{
    create_machine_session, MachineSessionCreateError, MachineSessionCreateOk,
    MachineSessionCreateRequest,
};
pub use snapshot::{
    get_machine_snapshot, get_machine_snapshot_from_session, materialize_machine_proof_snapshot,
    parse_machine_snapshot_get_request, stored_snapshot_view_canonical_bytes,
    MachineSnapshotGetError, MachineSnapshotGetOk, MachineSnapshotGetRequest,
    MachineSnapshotLookupError, MachineSnapshotMaterializationContext,
    MachineSnapshotMaterializationError, MachineSnapshotStore, MachineSnapshotStoreError,
    StoredSnapshotEntry, StoredSnapshotView,
};
pub use std_library::{
    audit_machine_std_mvp_release_artifacts, audit_machine_std_mvp_validated_release,
    finalize_machine_std_mvp_import_bundle_recipes, finalize_machine_std_mvp_theorem_index,
    generate_machine_std_mvp_final_import_bundle_set, generate_machine_std_mvp_final_theorem_index,
    generate_machine_std_mvp_import_bundle_set, generate_machine_std_mvp_rewrite_profile_set,
    generate_machine_std_mvp_simp_profile_set, generate_machine_std_mvp_theorem_index,
    load_machine_std_certificates_from_locators, load_machine_std_mvp_certificates,
    load_machine_std_mvp_release,
    load_machine_std_mvp_release_with_optional_prompt_metadata_from_json,
    load_machine_std_mvp_release_with_sidecars_from_json, machine_std_audit_check_canonical_bytes,
    machine_std_audit_report_canonical_bytes, machine_std_audit_report_hash,
    machine_std_axiom_ref_canonical_bytes, machine_std_axiom_report_canonical_bytes,
    machine_std_axiom_report_hash, machine_std_global_ref_canonical_bytes,
    machine_std_global_ref_view_canonical_bytes, machine_std_import_bundle_canonical_bytes,
    machine_std_import_bundle_hash, machine_std_import_bundle_set_canonical_bytes,
    machine_std_import_bundle_set_hash, machine_std_library_release_canonical_bytes,
    machine_std_library_release_hash, machine_std_module_artifact_canonical_bytes,
    machine_std_mvp_module_locators, machine_std_prompt_example_canonical_bytes,
    machine_std_prompt_metadata_canonical_bytes, machine_std_prompt_metadata_hash,
    machine_std_prompt_metadata_set_canonical_bytes,
    machine_std_rewrite_descriptor_canonical_bytes, machine_std_rewrite_profile_canonical_bytes,
    machine_std_rewrite_profile_hash, machine_std_rewrite_profile_set_canonical_bytes,
    machine_std_rewrite_profile_set_hash, machine_std_rule_telescope_canonical_bytes,
    machine_std_rule_telescope_hash, machine_std_simp_profile_canonical_bytes,
    machine_std_simp_profile_hash, machine_std_simp_profile_set_canonical_bytes,
    machine_std_simp_profile_set_hash, machine_std_tactic_options_recipe_canonical_bytes,
    machine_std_tactic_options_recipe_request, machine_std_theorem_entry_canonical_bytes,
    machine_std_theorem_index_canonical_bytes, machine_std_theorem_index_hash,
    parse_machine_std_axiom_report_json, parse_machine_std_import_bundle_set_json,
    parse_machine_std_library_release_json, parse_machine_std_prompt_metadata_json,
    parse_machine_std_rewrite_profile_set_json, parse_machine_std_simp_profile_set_json,
    parse_machine_std_theorem_index_json, validate_machine_std_locator_path,
    validate_machine_std_mvp_final_theorem_index, validate_machine_std_mvp_import_bundle_recipes,
    validate_machine_std_mvp_import_bundle_set, validate_machine_std_mvp_locators,
    validate_machine_std_mvp_optional_prompt_metadata, validate_machine_std_mvp_prompt_metadata,
    validate_machine_std_mvp_release_final_sidecar_counts,
    validate_machine_std_mvp_rewrite_profile_set, validate_machine_std_mvp_simp_profile_set,
    validate_machine_std_mvp_theorem_index, MachineStdArtifactKind, MachineStdArtifactShapeError,
    MachineStdArtifactShapeErrorReason, MachineStdAttribute, MachineStdAuditArtifacts,
    MachineStdAuditCheck, MachineStdAuditError, MachineStdAuditReport, MachineStdAxiomPolicyError,
    MachineStdAxiomRef, MachineStdAxiomReport, MachineStdCanonicalBytesError, MachineStdGlobalRef,
    MachineStdGlobalRefView, MachineStdImportBundle, MachineStdImportBundleError,
    MachineStdImportBundleSet, MachineStdImportCertificate, MachineStdLibraryRelease,
    MachineStdLibraryReleaseError, MachineStdLoadedModule, MachineStdLoadedRelease,
    MachineStdLocatorPathError, MachineStdModuleArtifact, MachineStdModuleAxiomReport,
    MachineStdModuleLocator, MachineStdPromptExample, MachineStdPromptMetadata,
    MachineStdPromptMetadataError, MachineStdPromptMetadataSet, MachineStdReleaseArtifactError,
    MachineStdReleaseLoaderError, MachineStdReleaseSidecarJson, MachineStdRewriteDescriptor,
    MachineStdRewriteProfile, MachineStdRewriteProfileError, MachineStdRewriteProfileSet,
    MachineStdRewriteSafety, MachineStdSimpProfile, MachineStdSimpProfileError,
    MachineStdSimpProfileSet, MachineStdTacticOptionsRecipe, MachineStdTheoremEntry,
    MachineStdTheoremIndex, MachineStdTheoremIndexError, MachineStdTheoremKind,
    MachineStdValidatedRelease,
};
pub use tactic::{
    parse_machine_tactic_batch_request, parse_machine_tactic_run_request,
    run_machine_tactic_batch_request, run_machine_tactic_batch_request_in_sessions,
    run_machine_tactic_request, run_machine_tactic_request_in_sessions,
    MachineBatchSchedulerLimits, MachineRunSchedulerLimits, MachineTacticBatchCandidateRequest,
    MachineTacticBatchError, MachineTacticBatchItemResponse, MachineTacticBatchOkFields,
    MachineTacticBatchRequest, MachineTacticBatchResponse, MachineTacticBatchSchedulerFields,
    MachineTacticRunDeltaSummary, MachineTacticRunError, MachineTacticRunErrorFields,
    MachineTacticRunErrorObject, MachineTacticRunRequest, MachineTacticRunResponse,
    MachineTacticRunResultKind, MachineTacticRunSchedulerFields, MachineTacticRunSuccessFields,
    MachineTacticRunSuccessResult,
};
pub use types::{
    format_goal_id_wire, format_hash_string, format_meta_var_id_wire, is_machine_local_name,
    is_machine_surface_name_component, is_machine_surface_renderable_name_wire,
    is_machine_surface_term_head_component, is_machine_universe_param_name,
    machine_endpoint_envelope_spec, parse_fully_qualified_name_wire, parse_goal_id_wire,
    parse_hash_string, parse_local_id_wire, parse_machine_surface_renderable_name_wire,
    parse_machine_universe_param_name, parse_meta_var_id_wire, parse_module_name_wire,
    parse_phase5_name, phase5_name_canonical_bytes, validate_delete_session_request,
    validate_machine_endpoint_envelope, CheckedMachineProofRoot, HashString, KernelCheckProfileId,
    MachineApiCompactErrorWire, MachineApiEndpoint, MachineApiErrorResponse, MachineApiErrorWire,
    MachineApiOkResponse, MachineApiOptions, MachineApiResponseEnvelope, MachineApiResponseStatus,
    MachineApiSchedulerResponse, MachineApiVersion, MachineEndpointEnvelopeSpec,
    MachineEndpointFieldSpec, MachineEndpointFieldType, MachineGoalView, MachineLocalView,
    MachineProofSession, MachineProofSnapshot, MachineRootTermSource, MachineSchedulerArtifact,
    MachineSchedulerArtifactKind, MachineSchedulerArtifactScope,
    MachineTacticOptionsConversionError, MachineTacticOptionsRequest,
    MachineValidatedEndpointEnvelope, MachineWireGrammarError, MachineWireGrammarErrorKind,
    SessionId, SnapshotId, KERNEL_CHECK_PROFILE_BUILTIN_NAT_EQ_REC,
    KERNEL_CHECK_PROFILE_BUILTIN_NONE, MACHINE_API_VERSION, MACHINE_DISPLAY_PROFILE_ID,
    MACHINE_TACTIC_CANDIDATE_OUTPUT_SCHEMA,
};
pub use validation::{
    delayed_json_payload, parse_request_body, parse_request_body_with_limits,
    parse_strict_u64_token, validate_json_object, DelayedJsonPayload, FieldSpec, JsonFieldType,
    JsonPath, JsonPathElement, MachineApiErrorKind, MachineApiRequestError,
    MachineApiRequestErrorReason, ObjectSchema, StrictUnsignedIntegerError, ValidatedObject,
};
pub use verify::{
    parse_machine_verify_request, run_machine_verify_request,
    run_machine_verify_request_in_sessions, MachineCertificateWirePayload,
    MachineVerifiedModuleCertificatePayload, MachineVerifyError, MachineVerifyOkFields,
    MachineVerifyRequest, MachineVerifyResponse,
};
