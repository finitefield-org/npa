//! Phase 5 API substrate.
//!
//! This crate is intentionally outside the trusted kernel. It handles wire JSON
//! decoding and request-shape validation for machine-facing endpoints.

mod adapter;
mod callable;
mod current;
mod json;
mod projection;
mod renderer;
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
pub use validation::{
    delayed_json_payload, parse_request_body, parse_request_body_with_limits,
    parse_strict_u64_token, validate_json_object, DelayedJsonPayload, FieldSpec, JsonFieldType,
    JsonPath, JsonPathElement, MachineApiErrorKind, MachineApiRequestError,
    MachineApiRequestErrorReason, ObjectSchema, StrictUnsignedIntegerError, ValidatedObject,
};
