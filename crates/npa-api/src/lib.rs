//! Phase 5 API substrate.
//!
//! This crate is intentionally outside the trusted kernel. It handles wire JSON
//! decoding and request-shape validation for machine-facing endpoints.

mod adapter;
mod callable;
mod current;
mod diagnostic;
mod json;
mod projection;
mod renderer;
mod session;
mod snapshot;
mod tactic;
mod types;
mod validation;

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
    project_checked_current_decl_context, CheckedCurrentDeclPackageInput,
    CheckedCurrentDeclProjectionError, CurrentDeclDependencyEntry, CurrentDeclDependencyReport,
    CurrentDeclIndexEntry, CurrentGeneratedDeclEntry, CurrentGeneratedDeclKind,
    MachineAxiomRefWire, MachineCheckedCurrentDeclContext, MachineCheckedDeclSignature,
    MachineDependencyRefWire,
};
pub use diagnostic::{
    machine_api_diagnostic_canonical_bytes, machine_api_diagnostic_hash,
    MachineApiDiagnosticCanonicalizationError,
};
pub use json::{
    JsonDocument, JsonMember, JsonParseError, JsonParseErrorKind, JsonParseLimits, JsonSpan,
    JsonValue, JsonValueKind,
};
pub use projection::{
    project_import_certificate_context, GeneratedDeclKind, ImportProjectionError,
    MachineImportCertificateContext, VerifiedImportDeclIndexEntry,
    VerifiedImportGeneratedDeclEntry, VerifiedImportGeneratedDeclPayload, VerifiedImportKey,
    VerifiedModuleCertificateInput, VerifiedModuleContextEntry,
};
pub use renderer::{
    render_machine_expr_source, render_machine_expr_view, renderer_qa_round_trip, LocalId,
    MachineDisplayRenderScope, MachineDisplayRenderScopeEntry, MachineExprRendererContext,
    MachineExprRendererError, MachineExprView, MachineGlobalRefView,
    Phase5ResolvedDisplayCoreRefOwner,
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
pub use tactic::{
    parse_machine_tactic_run_request, run_machine_tactic_request,
    run_machine_tactic_request_in_sessions, MachineRunSchedulerLimits,
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
    SessionId, SnapshotId, KERNEL_CHECK_PROFILE_BUILTIN_NAT_EQ_REC, MACHINE_API_VERSION,
    MACHINE_DISPLAY_PROFILE_ID, MACHINE_TACTIC_CANDIDATE_OUTPUT_SCHEMA,
};
pub use validation::{
    delayed_json_payload, parse_request_body, parse_request_body_with_limits,
    parse_strict_u64_token, validate_json_object, DelayedJsonPayload, FieldSpec, JsonFieldType,
    JsonPath, JsonPathElement, MachineApiErrorKind, MachineApiRequestError,
    MachineApiRequestErrorReason, ObjectSchema, StrictUnsignedIntegerError, ValidatedObject,
};
