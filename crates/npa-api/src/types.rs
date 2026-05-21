use std::collections::BTreeSet;

use npa_cert::{CoreModule, Hash, ModuleCert, ModuleName, Name, VerifiedModule};
use npa_frontend::{
    FileId, HumanCompileOptions, HumanDiagnostic, HumanExpr, HumanImportedSourceInterface,
    HumanSourceInterface, MachineSurfaceCallableInterfaceTable,
};
use npa_kernel::Expr;
use npa_tactic::{
    GoalId, MachineProofDelta, MachineProofState, MachineTacticDiagnostic, MetaVarId,
};

use crate::current::{MachineAxiomRefWire, MachineCheckedCurrentDeclContext};
use crate::json::{JsonMember, JsonValue, JsonValueKind};
use crate::projection::{MachineImportCertificateContext, VerifiedImportKey};
use crate::renderer::{LocalId, MachineDisplayRenderScope, MachineExprView};
use crate::snapshot::MachineSnapshotStore;
use crate::validation::{
    parse_strict_u64_token, JsonPath, MachineApiErrorKind, MachineApiRequestError,
    MachineApiRequestErrorReason, StrictUnsignedIntegerError,
};
use crate::{
    MachineApiDiagnosticCanonicalizationError, MachineApiDiagnosticPhase,
    MachineApiDiagnosticProjection, MachineApiTacticKind,
};

pub const MACHINE_API_VERSION: &str = "npa.machine-api.v1";
pub const MACHINE_DISPLAY_PROFILE_ID: &str = "npa.phase5.display.v1";
pub const MACHINE_TACTIC_CANDIDATE_OUTPUT_SCHEMA: &str = "npa.machine_tactic_candidate.v1";
pub const KERNEL_CHECK_PROFILE_BUILTIN_NAT_EQ_REC: &str = "npa.kernel.v0.1.builtin-nat-eq-rec";
pub const KERNEL_CHECK_PROFILE_BUILTIN_NONE: &str = "npa.kernel.v0.1.builtin-none";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanApiCompileOptions {
    pub max_notation_candidates: usize,
}

impl Default for HumanApiCompileOptions {
    fn default() -> Self {
        let frontend = HumanCompileOptions::default();
        Self {
            max_notation_candidates: frontend.max_notation_candidates,
        }
    }
}

impl From<&HumanApiCompileOptions> for HumanCompileOptions {
    fn from(value: &HumanApiCompileOptions) -> Self {
        Self {
            max_notation_candidates: value.max_notation_candidates,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HumanCurrentModuleSource<'src> {
    pub file_id: FileId,
    pub source: &'src str,
}

#[derive(Clone, Debug)]
pub struct HumanCompileCoreRequest<'src, 'imports> {
    pub current_module: ModuleName,
    pub current_source: HumanCurrentModuleSource<'src>,
    pub verified_imports: &'imports [npa_frontend::VerifiedImport],
    pub imported_source_interfaces: &'imports [HumanImportedSourceInterface],
    pub options: HumanApiCompileOptions,
}

#[derive(Clone, Debug)]
pub struct HumanCompileCertificateRequest<'src, 'imports> {
    pub current_module: ModuleName,
    pub current_source: HumanCurrentModuleSource<'src>,
    pub verified_modules: &'imports [VerifiedModule],
    pub imported_source_interfaces: &'imports [HumanImportedSourceInterface],
    pub options: HumanApiCompileOptions,
}

#[derive(Clone, Debug)]
pub struct HumanStartProofRequest<'src, 'imports> {
    pub current_module: ModuleName,
    pub theorem_name: Name,
    pub current_source: HumanCurrentModuleSource<'src>,
    pub verified_modules: &'imports [VerifiedModule],
    pub imported_source_interfaces: &'imports [HumanImportedSourceInterface],
    pub options: HumanApiCompileOptions,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanCompileCoreOk {
    pub core_module: CoreModule,
    pub source_interface: HumanSourceInterface,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanCompileCertificateOk {
    pub certificate: ModuleCert,
    pub source_interface: HumanSourceInterface,
}

#[derive(Clone, Debug)]
pub struct HumanStartProofOk {
    pub state: MachineProofState,
    pub source_interface: HumanSourceInterface,
}

#[derive(Clone, Debug)]
pub struct HumanTacticTermCheckRequest<'term, 'ctx> {
    pub state: &'ctx MachineProofState,
    pub goal_id: GoalId,
    pub term: &'term HumanExpr,
    pub current_source_interface: &'ctx HumanSourceInterface,
    pub imported_source_interfaces: &'ctx [HumanImportedSourceInterface],
    pub options: HumanApiCompileOptions,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanTacticTermCheckOk {
    pub expr: Expr,
    pub inferred_type: Expr,
}

#[derive(Clone, Debug)]
pub struct HumanExactTacticRequest<'term, 'ctx> {
    pub state: &'ctx MachineProofState,
    pub goal_id: GoalId,
    pub term: &'term HumanExpr,
    pub current_source_interface: &'ctx HumanSourceInterface,
    pub imported_source_interfaces: &'ctx [HumanImportedSourceInterface],
    pub options: HumanApiCompileOptions,
}

#[derive(Clone, Debug)]
pub struct HumanExactTacticOk {
    pub state: MachineProofState,
    pub delta: MachineProofDelta,
    pub expr: Expr,
    pub inferred_type: Expr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanCompileError {
    pub diagnostic: HumanDiagnostic,
}

impl From<HumanDiagnostic> for HumanCompileError {
    fn from(diagnostic: HumanDiagnostic) -> Self {
        Self { diagnostic }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HumanStartProofError {
    Human(HumanCompileError),
    Machine(MachineTacticDiagnostic),
}

impl From<HumanDiagnostic> for HumanStartProofError {
    fn from(diagnostic: HumanDiagnostic) -> Self {
        Self::Human(HumanCompileError::from(diagnostic))
    }
}

impl From<MachineTacticDiagnostic> for HumanStartProofError {
    fn from(diagnostic: MachineTacticDiagnostic) -> Self {
        Self::Machine(diagnostic)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HumanTacticTermError {
    Human(HumanCompileError),
    Machine(MachineTacticDiagnostic),
}

impl From<HumanDiagnostic> for HumanTacticTermError {
    fn from(diagnostic: HumanDiagnostic) -> Self {
        Self::Human(HumanCompileError::from(diagnostic))
    }
}

impl From<MachineTacticDiagnostic> for HumanTacticTermError {
    fn from(diagnostic: MachineTacticDiagnostic) -> Self {
        Self::Machine(diagnostic)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MachineApiVersion {
    V1,
}

impl MachineApiVersion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::V1 => MACHINE_API_VERSION,
        }
    }

    pub fn parse(value: &str) -> Result<Self, MachineWireGrammarError> {
        if value == MACHINE_API_VERSION {
            Ok(Self::V1)
        } else {
            Err(MachineWireGrammarError::new(
                MachineWireGrammarErrorKind::UnsupportedLiteral,
            ))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KernelCheckProfileId {
    BuiltinNone,
    BuiltinNatEqRec,
}

impl KernelCheckProfileId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BuiltinNone => KERNEL_CHECK_PROFILE_BUILTIN_NONE,
            Self::BuiltinNatEqRec => KERNEL_CHECK_PROFILE_BUILTIN_NAT_EQ_REC,
        }
    }

    pub fn parse(value: &str) -> Result<Self, MachineWireGrammarError> {
        match value {
            KERNEL_CHECK_PROFILE_BUILTIN_NONE => Ok(Self::BuiltinNone),
            KERNEL_CHECK_PROFILE_BUILTIN_NAT_EQ_REC => Ok(Self::BuiltinNatEqRec),
            _ => Err(MachineWireGrammarError::new(
                MachineWireGrammarErrorKind::UnsupportedLiteral,
            )),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MachineProofSession {
    pub session_id: SessionId,
    pub protocol_version: MachineApiVersion,
    pub session_root_hash: Hash,
    pub root: CheckedMachineProofRoot,
    pub imports: Vec<VerifiedImportKey>,
    pub import_certificate_context: MachineImportCertificateContext,
    pub machine_display_render_scope: MachineDisplayRenderScope,
    pub machine_surface_callable_interface_table: MachineSurfaceCallableInterfaceTable,
    pub checked_current_decls: MachineCheckedCurrentDeclContext,
    pub options: MachineApiOptions,
    pub initial_snapshot: MachineProofSnapshot,
    pub snapshots: MachineSnapshotStore,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedMachineProofRoot {
    pub module: ModuleName,
    pub theorem_name: Name,
    pub source_index: u64,
    pub universe_params: Vec<String>,
    pub theorem_type_source: MachineRootTermSource,
    pub theorem_type_core_hash: Hash,
}

impl CheckedMachineProofRoot {
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        encode_string(&mut out, "npa.phase5.checked-machine-proof-root.v1");
        encode_name(&mut out, &self.module);
        encode_name(&mut out, &self.theorem_name);
        encode_uvar(&mut out, self.source_index);
        encode_list_len(&mut out, self.universe_params.len());
        for param in &self.universe_params {
            encode_string(&mut out, param);
        }
        encode_hash(&mut out, &self.theorem_type_source.phase3_canonical_hash);
        encode_hash(&mut out, &self.theorem_type_core_hash);
        out
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineRootTermSource {
    pub source: String,
    pub phase3_canonical_hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineApiOptions {
    pub kernel_check_profile: KernelCheckProfileId,
    pub allow_axioms: Vec<MachineAxiomRefWire>,
    pub tactic_options: MachineTacticOptionsRequest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTacticOptionsRequest {
    pub simp_rules: Vec<npa_tactic::SimpRuleRef>,
    pub eq_family: Option<npa_tactic::EqFamilyRef>,
    pub nat_family: Option<npa_tactic::NatFamilyRef>,
    pub max_simp_rewrite_steps: u64,
    pub max_open_goals: u64,
    pub max_metas: u64,
}

impl TryFrom<MachineTacticOptionsRequest> for npa_tactic::MachineTacticOptions {
    type Error = MachineTacticOptionsConversionError;

    fn try_from(value: MachineTacticOptionsRequest) -> Result<Self, Self::Error> {
        let max_open_goals = usize::try_from(value.max_open_goals)
            .map_err(|_| MachineTacticOptionsConversionError::ValueExceedsUsize)?;
        let max_metas = usize::try_from(value.max_metas)
            .map_err(|_| MachineTacticOptionsConversionError::ValueExceedsUsize)?;
        Ok(Self {
            simp_rules: value.simp_rules,
            max_simp_rewrite_steps: value.max_simp_rewrite_steps,
            max_open_goals,
            max_metas,
            eq_family: value.eq_family,
            nat_family: value.nat_family,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineTacticOptionsConversionError {
    ValueExceedsUsize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineProofSnapshot {
    pub snapshot_id: SnapshotId,
    pub session_id: SessionId,
    pub state_fingerprint: Hash,
    pub tactic_options_fingerprint: Hash,
    pub open_goals: Vec<GoalId>,
    pub goals: Vec<MachineGoalView>,
    pub proof_skeleton_hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineGoalView {
    pub goal_id: GoalId,
    pub meta_id: MetaVarId,
    pub context_hash: Hash,
    pub local_name_map_hash: Hash,
    pub context: Vec<MachineLocalView>,
    pub target: MachineExprView,
    pub target_hash: Hash,
    pub goal_fingerprint: Hash,
    pub allowed_tactics: Vec<MachineApiTacticKind>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineLocalView {
    pub local_id: LocalId,
    pub machine_name: String,
    pub display_name: String,
    pub ty: MachineExprView,
    pub value: Option<MachineExprView>,
    pub depends_on: Vec<LocalId>,
    pub binder_index: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineApiErrorWire {
    pub kind: MachineApiErrorKind,
    pub phase: MachineApiDiagnosticPhase,
    pub diagnostic_hash: Hash,
    pub retryable: bool,
    pub goal_id: Option<GoalId>,
    pub tactic_kind: Option<MachineApiTacticKind>,
    pub primary_name: Option<Name>,
    pub primary_axiom_ref: Option<MachineAxiomRefWire>,
    pub expected_hash: Option<Hash>,
    pub actual_hash: Option<Hash>,
}

impl MachineApiErrorWire {
    pub fn from_projection(
        diagnostic: &MachineApiDiagnosticProjection,
    ) -> Result<Self, MachineApiDiagnosticCanonicalizationError> {
        Ok(Self {
            kind: diagnostic.kind,
            phase: diagnostic.phase,
            diagnostic_hash: diagnostic.diagnostic_hash()?,
            retryable: diagnostic.retryable,
            goal_id: diagnostic.goal_id,
            tactic_kind: diagnostic.tactic_kind,
            primary_name: diagnostic.primary_name.clone(),
            primary_axiom_ref: diagnostic.primary_axiom_ref.clone(),
            expected_hash: diagnostic.expected_hash,
            actual_hash: diagnostic.actual_hash,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineApiCompactErrorWire {
    pub error_kind: MachineApiErrorKind,
    pub phase: MachineApiDiagnosticPhase,
    pub diagnostic_hash: Hash,
    pub retryable: bool,
    pub goal_id: Option<GoalId>,
    pub tactic_kind: Option<MachineApiTacticKind>,
    pub primary_name: Option<Name>,
    pub primary_axiom_ref: Option<MachineAxiomRefWire>,
    pub expected_hash: Option<Hash>,
    pub actual_hash: Option<Hash>,
}

impl From<MachineApiErrorWire> for MachineApiCompactErrorWire {
    fn from(value: MachineApiErrorWire) -> Self {
        Self {
            error_kind: value.kind,
            phase: value.phase,
            diagnostic_hash: value.diagnostic_hash,
            retryable: value.retryable,
            goal_id: value.goal_id,
            tactic_kind: value.tactic_kind,
            primary_name: value.primary_name,
            primary_axiom_ref: value.primary_axiom_ref,
            expected_hash: value.expected_hash,
            actual_hash: value.actual_hash,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineApiErrorResponse<ErrorObject = MachineApiErrorWire, TopLevelFields = ()> {
    pub status: MachineApiResponseStatus,
    pub error: ErrorObject,
    pub endpoint_fields: TopLevelFields,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineApiOkResponse<TopLevelFields> {
    pub status: MachineApiResponseStatus,
    pub endpoint_fields: TopLevelFields,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineApiSchedulerResponse<TopLevelFields = ()> {
    pub status: MachineApiResponseStatus,
    pub scheduler_artifact: MachineSchedulerArtifact,
    pub endpoint_fields: TopLevelFields,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MachineApiResponseEnvelope<
    OkTopLevelFields,
    ErrorObject = MachineApiErrorWire,
    ErrorTopLevelFields = (),
    SchedulerTopLevelFields = (),
> {
    Ok(MachineApiOkResponse<OkTopLevelFields>),
    Error(Box<MachineApiErrorResponse<ErrorObject, ErrorTopLevelFields>>),
    SchedulerStopped(MachineApiSchedulerResponse<SchedulerTopLevelFields>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MachineApiResponseStatus {
    Ok,
    Deleted,
    Success,
    Error,
    SchedulerStopped,
    PartialTimeout,
    PartialResourceLimit,
    Verified,
}

impl MachineApiResponseStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Deleted => "deleted",
            Self::Success => "success",
            Self::Error => "error",
            Self::SchedulerStopped => "scheduler_stopped",
            Self::PartialTimeout => "partial_timeout",
            Self::PartialResourceLimit => "partial_resource_limit",
            Self::Verified => "verified",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineSchedulerArtifact {
    pub kind: MachineSchedulerArtifactKind,
    pub scope: MachineSchedulerArtifactScope,
    pub retryable: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MachineSchedulerArtifactKind {
    Timeout,
    ResourceLimitExceeded,
}

impl MachineSchedulerArtifactKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Timeout => "timeout",
            Self::ResourceLimitExceeded => "resource_limit_exceeded",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MachineSchedulerArtifactScope {
    Candidate,
    Batch,
    Replay,
}

impl MachineSchedulerArtifactScope {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Candidate => "candidate",
            Self::Batch => "batch",
            Self::Replay => "replay",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionId(String);

impl SessionId {
    pub fn parse(value: &str) -> Result<Self, MachineWireGrammarError> {
        validate_session_id(value)?;
        Ok(Self(value.to_owned()))
    }

    pub fn new_unchecked(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn wire(&self) -> &str {
        self.as_str()
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        encode_string(&mut out, self.as_str());
        out
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SnapshotId {
    digest: Hash,
}

impl SnapshotId {
    pub fn parse(value: &str) -> Result<Self, MachineWireGrammarError> {
        let suffix = value.strip_prefix("mst_").ok_or_else(|| {
            MachineWireGrammarError::new(MachineWireGrammarErrorKind::InvalidPrefix)
        })?;
        let digest = parse_hex_digest(suffix)?;
        Ok(Self { digest })
    }

    pub const fn from_digest(digest: Hash) -> Self {
        Self { digest }
    }

    pub const fn from_state_fingerprint(state_fingerprint: Hash) -> Self {
        Self {
            digest: state_fingerprint,
        }
    }

    pub const fn digest(self) -> Hash {
        self.digest
    }

    pub fn wire(self) -> String {
        format!("mst_{}", lower_hex_hash(&self.digest))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HashString {
    digest: Hash,
}

impl HashString {
    pub fn parse(value: &str) -> Result<Self, MachineWireGrammarError> {
        parse_hash_string(value).map(Self::from_digest)
    }

    pub const fn from_digest(digest: Hash) -> Self {
        Self { digest }
    }

    pub const fn digest(self) -> Hash {
        self.digest
    }

    pub fn wire(self) -> String {
        format_hash_string(&self.digest)
    }
}

pub fn parse_hash_string(value: &str) -> Result<Hash, MachineWireGrammarError> {
    let suffix = value
        .strip_prefix("sha256:")
        .ok_or_else(|| MachineWireGrammarError::new(MachineWireGrammarErrorKind::InvalidPrefix))?;
    parse_hex_digest(suffix)
}

pub fn format_hash_string(hash: &Hash) -> String {
    format!("sha256:{}", lower_hex_hash(hash))
}

pub fn parse_goal_id_wire(value: &str) -> Result<GoalId, MachineWireGrammarError> {
    let suffix = strip_prefixed_decimal(value, 'g')?;
    Ok(GoalId(parse_decimal_u64(suffix)?))
}

pub fn format_goal_id_wire(id: GoalId) -> String {
    format!("g{}", id.0)
}

pub fn parse_meta_var_id_wire(value: &str) -> Result<MetaVarId, MachineWireGrammarError> {
    let suffix = strip_prefixed_decimal(value, 'm')?;
    Ok(MetaVarId(parse_decimal_u64(suffix)?))
}

pub fn format_meta_var_id_wire(id: MetaVarId) -> String {
    format!("m{}", id.0)
}

pub fn parse_local_id_wire(value: &str) -> Result<LocalId, MachineWireGrammarError> {
    let suffix = strip_prefixed_decimal(value, 'l')?;
    let value = parse_decimal_u64(suffix)?;
    let value = u32::try_from(value)
        .map_err(|_| MachineWireGrammarError::new(MachineWireGrammarErrorKind::Overflow))?;
    Ok(LocalId(value))
}

pub fn parse_phase5_name(value: &str) -> Result<Name, MachineWireGrammarError> {
    if value.is_empty() || value.starts_with('.') || value.ends_with('.') || value.contains("..") {
        return Err(MachineWireGrammarError::new(
            MachineWireGrammarErrorKind::InvalidName,
        ));
    }
    let components = value.split('.').map(ToOwned::to_owned).collect::<Vec<_>>();
    let name = Name(components);
    if name.is_canonical() {
        Ok(name)
    } else {
        Err(MachineWireGrammarError::new(
            MachineWireGrammarErrorKind::InvalidName,
        ))
    }
}

pub fn parse_module_name_wire(value: &str) -> Result<ModuleName, MachineWireGrammarError> {
    parse_phase5_name(value)
}

pub fn parse_fully_qualified_name_wire(value: &str) -> Result<Name, MachineWireGrammarError> {
    parse_phase5_name(value)
}

pub fn phase5_name_canonical_bytes(name: &Name) -> Result<Vec<u8>, MachineWireGrammarError> {
    if !name.is_canonical() {
        return Err(MachineWireGrammarError::new(
            MachineWireGrammarErrorKind::InvalidName,
        ));
    }
    let mut out = Vec::new();
    encode_name(&mut out, name);
    Ok(out)
}

pub fn is_machine_surface_name_component(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() || bytes.len() > 64 || value == "_" {
        return false;
    }
    matches!(bytes[0], b'A'..=b'Z' | b'a'..=b'z' | b'_')
        && bytes[1..]
            .iter()
            .all(|byte| matches!(byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'\''))
}

pub fn is_machine_surface_term_head_component(value: &str) -> bool {
    is_machine_surface_name_component(value) && !is_machine_surface_reserved(value)
}

pub fn is_machine_surface_renderable_name_wire(name: &Name) -> bool {
    let Some((head, tail)) = name.0.split_first() else {
        return false;
    };
    is_machine_surface_term_head_component(head)
        && tail
            .iter()
            .all(|component| is_machine_surface_name_component(component))
}

pub fn is_machine_universe_param_name(value: &str) -> bool {
    is_machine_surface_name_component(value) && !is_machine_surface_reserved(value)
}

pub fn is_machine_local_name(value: &str) -> bool {
    is_machine_surface_term_head_component(value)
}

pub fn parse_machine_surface_renderable_name_wire(
    value: &str,
) -> Result<Name, MachineWireGrammarError> {
    let name = parse_phase5_name(value)?;
    if is_machine_surface_renderable_name_wire(&name) {
        Ok(name)
    } else {
        Err(MachineWireGrammarError::new(
            MachineWireGrammarErrorKind::NonRenderableName,
        ))
    }
}

pub fn parse_machine_universe_param_name(value: &str) -> Result<String, MachineWireGrammarError> {
    if is_machine_universe_param_name(value) {
        Ok(value.to_owned())
    } else {
        Err(MachineWireGrammarError::new(
            MachineWireGrammarErrorKind::InvalidName,
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MachineApiEndpoint {
    CreateSession,
    DeleteSession,
    SnapshotGet,
    TacticRun,
    TacticBatch,
    SearchForGoal,
    PromptPayload,
    Replay,
    Verify,
}

impl MachineApiEndpoint {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CreateSession => "POST /machine/sessions",
            Self::DeleteSession => "DELETE /machine/sessions/{id}",
            Self::SnapshotGet => "POST /machine/snapshots/get",
            Self::TacticRun => "POST /machine/tactics/run",
            Self::TacticBatch => "POST /machine/tactics/batch",
            Self::SearchForGoal => "POST /machine/search/for_goal",
            Self::PromptPayload => "POST /machine/prompt_payload",
            Self::Replay => "POST /machine/replay",
            Self::Verify => "POST /machine/verify",
        }
    }

    pub const fn envelope_error_kind(self) -> MachineApiErrorKind {
        match self {
            Self::CreateSession | Self::DeleteSession => MachineApiErrorKind::InvalidSessionRequest,
            Self::SnapshotGet => MachineApiErrorKind::InvalidSnapshotRequest,
            Self::TacticRun => MachineApiErrorKind::InvalidTacticRunRequest,
            Self::TacticBatch => MachineApiErrorKind::InvalidBatchPolicy,
            Self::SearchForGoal => MachineApiErrorKind::InvalidTheoremQuery,
            Self::PromptPayload => MachineApiErrorKind::InvalidPromptPayloadRequest,
            Self::Replay => MachineApiErrorKind::InvalidReplayPlan,
            Self::Verify => MachineApiErrorKind::InvalidVerifyRequest,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineEndpointFieldType {
    Object,
    Array,
    String,
    Boolean,
    UnsignedInteger { min: u64, max: u64 },
    SessionId,
    SnapshotId,
    HashString,
    GoalId,
    ProtocolVersion,
    VerifyMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MachineEndpointFieldSpec {
    pub name: &'static str,
    pub required: bool,
    pub field_type: MachineEndpointFieldType,
    pub error_kind: MachineApiErrorKind,
}

impl MachineEndpointFieldSpec {
    pub const fn required(
        name: &'static str,
        field_type: MachineEndpointFieldType,
        error_kind: MachineApiErrorKind,
    ) -> Self {
        Self {
            name,
            required: true,
            field_type,
            error_kind,
        }
    }

    pub const fn optional(
        name: &'static str,
        field_type: MachineEndpointFieldType,
        error_kind: MachineApiErrorKind,
    ) -> Self {
        Self {
            name,
            required: false,
            field_type,
            error_kind,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MachineEndpointEnvelopeSpec {
    pub endpoint: MachineApiEndpoint,
    pub fields: &'static [MachineEndpointFieldSpec],
}

pub fn machine_endpoint_envelope_spec(endpoint: MachineApiEndpoint) -> MachineEndpointEnvelopeSpec {
    MachineEndpointEnvelopeSpec {
        endpoint,
        fields: endpoint_fields(endpoint),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MachineValidatedEndpointEnvelope<'value, 'src> {
    members: &'value [JsonMember<'src>],
}

impl<'value, 'src> MachineValidatedEndpointEnvelope<'value, 'src> {
    pub fn field(&self, field_name: &str) -> Option<&'value JsonValue<'src>> {
        self.members
            .iter()
            .find(|member| member.key() == field_name)
            .map(|member| member.value())
    }

    pub const fn members(&self) -> &'value [JsonMember<'src>] {
        self.members
    }
}

pub fn validate_machine_endpoint_envelope<'value, 'src>(
    value: &'value JsonValue<'src>,
    endpoint: MachineApiEndpoint,
    path: &JsonPath,
) -> Result<MachineValidatedEndpointEnvelope<'value, 'src>, MachineApiRequestError> {
    let envelope_kind = endpoint.envelope_error_kind();
    let Some(members) = value.object_members() else {
        return Err(MachineApiRequestError::new(
            envelope_kind,
            path.clone(),
            MachineApiRequestErrorReason::ExpectedObject {
                actual: value.kind(),
            },
        ));
    };

    let fields = endpoint_fields(endpoint);
    let mut seen = BTreeSet::new();
    for member in members {
        if !seen.insert(member.key().to_owned()) {
            return Err(MachineApiRequestError::new(
                envelope_kind,
                path.field(member.key()),
                MachineApiRequestErrorReason::DuplicateKey {
                    key: member.key().to_owned(),
                },
            ));
        }
    }

    for member in members {
        if !fields.iter().any(|field| field.name == member.key()) {
            return Err(MachineApiRequestError::new(
                envelope_kind,
                path.field(member.key()),
                MachineApiRequestErrorReason::UnknownField {
                    field: member.key().to_owned(),
                },
            ));
        }
    }

    for field in fields {
        let Some(member) = members.iter().find(|member| member.key() == field.name) else {
            if field.required {
                return Err(MachineApiRequestError::new(
                    field.error_kind,
                    path.field(field.name),
                    MachineApiRequestErrorReason::MissingField { field: field.name },
                ));
            }
            continue;
        };
        validate_endpoint_field(endpoint, field, member.value(), &path.field(field.name))?;
    }

    Ok(MachineValidatedEndpointEnvelope { members })
}

pub fn validate_delete_session_request(
    session_id: &str,
    has_body: bool,
) -> Result<SessionId, MachineApiRequestError> {
    if has_body {
        return Err(MachineApiRequestError::new(
            MachineApiErrorKind::InvalidSessionRequest,
            JsonPath::root(),
            MachineApiRequestErrorReason::UnknownField {
                field: "body".to_owned(),
            },
        ));
    }
    SessionId::parse(session_id).map_err(|_| {
        MachineApiRequestError::new(
            MachineApiErrorKind::InvalidSessionRequest,
            JsonPath::root().field("session_id"),
            MachineApiRequestErrorReason::TypeMismatch {
                field: "session_id",
                expected: crate::JsonFieldType::String,
                actual: JsonValueKind::String,
            },
        )
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MachineWireGrammarError {
    pub kind: MachineWireGrammarErrorKind,
}

impl MachineWireGrammarError {
    pub const fn new(kind: MachineWireGrammarErrorKind) -> Self {
        Self { kind }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineWireGrammarErrorKind {
    InvalidPrefix,
    InvalidLength,
    InvalidCharacter,
    InvalidHex,
    InvalidDecimal,
    LeadingZero,
    Overflow,
    InvalidName,
    NonRenderableName,
    UnsupportedLiteral,
}

const SESSION_CREATE_FIELDS: &[MachineEndpointFieldSpec] = &[
    MachineEndpointFieldSpec::required(
        "protocol_version",
        MachineEndpointFieldType::ProtocolVersion,
        MachineApiErrorKind::InvalidSessionRequest,
    ),
    MachineEndpointFieldSpec::required(
        "root",
        MachineEndpointFieldType::Object,
        MachineApiErrorKind::InvalidSessionRequest,
    ),
    MachineEndpointFieldSpec::required(
        "import_closure",
        MachineEndpointFieldType::Array,
        MachineApiErrorKind::InvalidSessionRequest,
    ),
    MachineEndpointFieldSpec::required(
        "imports",
        MachineEndpointFieldType::Array,
        MachineApiErrorKind::InvalidSessionRequest,
    ),
    MachineEndpointFieldSpec::required(
        "checked_current_decls",
        MachineEndpointFieldType::Array,
        MachineApiErrorKind::InvalidSessionRequest,
    ),
    MachineEndpointFieldSpec::required(
        "options",
        MachineEndpointFieldType::Object,
        MachineApiErrorKind::InvalidSessionRequest,
    ),
];

const SNAPSHOT_GET_FIELDS: &[MachineEndpointFieldSpec] = &[
    common_required(
        "session_id",
        MachineEndpointFieldType::SessionId,
        MachineApiErrorKind::InvalidSnapshotRequest,
    ),
    common_required(
        "snapshot_id",
        MachineEndpointFieldType::SnapshotId,
        MachineApiErrorKind::InvalidSnapshotRequest,
    ),
    common_required(
        "state_fingerprint",
        MachineEndpointFieldType::HashString,
        MachineApiErrorKind::InvalidSnapshotRequest,
    ),
    common_required(
        "include_pretty",
        MachineEndpointFieldType::Boolean,
        MachineApiErrorKind::InvalidSnapshotRequest,
    ),
];

const TACTIC_RUN_FIELDS: &[MachineEndpointFieldSpec] = &[
    common_required(
        "session_id",
        MachineEndpointFieldType::SessionId,
        MachineApiErrorKind::InvalidTacticRunRequest,
    ),
    common_required(
        "snapshot_id",
        MachineEndpointFieldType::SnapshotId,
        MachineApiErrorKind::InvalidTacticRunRequest,
    ),
    common_required(
        "state_fingerprint",
        MachineEndpointFieldType::HashString,
        MachineApiErrorKind::InvalidTacticRunRequest,
    ),
    common_required(
        "goal_id",
        MachineEndpointFieldType::GoalId,
        MachineApiErrorKind::InvalidTacticRunRequest,
    ),
    common_required(
        "candidate",
        MachineEndpointFieldType::Object,
        MachineApiErrorKind::InvalidTacticRunRequest,
    ),
    common_required(
        "deterministic_budget",
        MachineEndpointFieldType::Object,
        MachineApiErrorKind::InvalidBudget,
    ),
    common_optional(
        "scheduler_limits",
        MachineEndpointFieldType::Object,
        MachineApiErrorKind::InvalidSchedulerLimits,
    ),
];

const TACTIC_BATCH_FIELDS: &[MachineEndpointFieldSpec] = &[
    common_required(
        "session_id",
        MachineEndpointFieldType::SessionId,
        MachineApiErrorKind::InvalidBatchPolicy,
    ),
    common_required(
        "snapshot_id",
        MachineEndpointFieldType::SnapshotId,
        MachineApiErrorKind::InvalidBatchPolicy,
    ),
    common_required(
        "state_fingerprint",
        MachineEndpointFieldType::HashString,
        MachineApiErrorKind::InvalidBatchPolicy,
    ),
    common_required(
        "goal_id",
        MachineEndpointFieldType::GoalId,
        MachineApiErrorKind::InvalidBatchPolicy,
    ),
    common_required(
        "candidates",
        MachineEndpointFieldType::Array,
        MachineApiErrorKind::InvalidBatchPolicy,
    ),
    common_required(
        "deterministic_budget",
        MachineEndpointFieldType::Object,
        MachineApiErrorKind::InvalidBudget,
    ),
    common_required(
        "batch_policy",
        MachineEndpointFieldType::Object,
        MachineApiErrorKind::InvalidBatchPolicy,
    ),
    common_optional(
        "scheduler_limits",
        MachineEndpointFieldType::Object,
        MachineApiErrorKind::InvalidSchedulerLimits,
    ),
];

const SEARCH_FOR_GOAL_FIELDS: &[MachineEndpointFieldSpec] = &[
    common_required(
        "session_id",
        MachineEndpointFieldType::SessionId,
        MachineApiErrorKind::InvalidTheoremQuery,
    ),
    common_required(
        "snapshot_id",
        MachineEndpointFieldType::SnapshotId,
        MachineApiErrorKind::InvalidTheoremQuery,
    ),
    common_required(
        "state_fingerprint",
        MachineEndpointFieldType::HashString,
        MachineApiErrorKind::InvalidTheoremQuery,
    ),
    common_required(
        "goal_id",
        MachineEndpointFieldType::GoalId,
        MachineApiErrorKind::InvalidTheoremQuery,
    ),
    common_required(
        "modes",
        MachineEndpointFieldType::Array,
        MachineApiErrorKind::InvalidTheoremQuery,
    ),
    common_required(
        "limit",
        MachineEndpointFieldType::UnsignedInteger { min: 1, max: 256 },
        MachineApiErrorKind::InvalidTheoremQuery,
    ),
    common_required(
        "filters",
        MachineEndpointFieldType::Object,
        MachineApiErrorKind::InvalidTheoremQuery,
    ),
];

const PROMPT_PAYLOAD_FIELDS: &[MachineEndpointFieldSpec] = &[
    common_required(
        "session_id",
        MachineEndpointFieldType::SessionId,
        MachineApiErrorKind::InvalidPromptPayloadRequest,
    ),
    common_required(
        "snapshot_id",
        MachineEndpointFieldType::SnapshotId,
        MachineApiErrorKind::InvalidPromptPayloadRequest,
    ),
    common_required(
        "state_fingerprint",
        MachineEndpointFieldType::HashString,
        MachineApiErrorKind::InvalidPromptPayloadRequest,
    ),
    common_required(
        "goal_id",
        MachineEndpointFieldType::GoalId,
        MachineApiErrorKind::InvalidPromptPayloadRequest,
    ),
    common_required(
        "include_pretty",
        MachineEndpointFieldType::Boolean,
        MachineApiErrorKind::InvalidPromptPayloadRequest,
    ),
    common_required(
        "include_failed_candidates",
        MachineEndpointFieldType::Boolean,
        MachineApiErrorKind::InvalidPromptPayloadRequest,
    ),
    common_required(
        "premise_selection",
        MachineEndpointFieldType::Object,
        MachineApiErrorKind::InvalidPromptPayloadRequest,
    ),
    common_required(
        "failed_candidates",
        MachineEndpointFieldType::Array,
        MachineApiErrorKind::InvalidPromptPayloadRequest,
    ),
];

const REPLAY_FIELDS: &[MachineEndpointFieldSpec] = &[
    common_required(
        "session_id",
        MachineEndpointFieldType::SessionId,
        MachineApiErrorKind::InvalidReplayPlan,
    ),
    common_required(
        "plan",
        MachineEndpointFieldType::Object,
        MachineApiErrorKind::InvalidReplayPlan,
    ),
];

const VERIFY_FIELDS: &[MachineEndpointFieldSpec] = &[
    common_required(
        "session_id",
        MachineEndpointFieldType::SessionId,
        MachineApiErrorKind::InvalidVerifyRequest,
    ),
    common_required(
        "snapshot_id",
        MachineEndpointFieldType::SnapshotId,
        MachineApiErrorKind::InvalidVerifyRequest,
    ),
    common_required(
        "state_fingerprint",
        MachineEndpointFieldType::HashString,
        MachineApiErrorKind::InvalidVerifyRequest,
    ),
    common_required(
        "mode",
        MachineEndpointFieldType::VerifyMode,
        MachineApiErrorKind::InvalidVerifyRequest,
    ),
];

const fn common_required(
    name: &'static str,
    field_type: MachineEndpointFieldType,
    error_kind: MachineApiErrorKind,
) -> MachineEndpointFieldSpec {
    MachineEndpointFieldSpec::required(name, field_type, error_kind)
}

const fn common_optional(
    name: &'static str,
    field_type: MachineEndpointFieldType,
    error_kind: MachineApiErrorKind,
) -> MachineEndpointFieldSpec {
    MachineEndpointFieldSpec::optional(name, field_type, error_kind)
}

fn endpoint_fields(endpoint: MachineApiEndpoint) -> &'static [MachineEndpointFieldSpec] {
    match endpoint {
        MachineApiEndpoint::CreateSession => SESSION_CREATE_FIELDS,
        MachineApiEndpoint::DeleteSession => &[],
        MachineApiEndpoint::SnapshotGet => SNAPSHOT_GET_FIELDS,
        MachineApiEndpoint::TacticRun => TACTIC_RUN_FIELDS,
        MachineApiEndpoint::TacticBatch => TACTIC_BATCH_FIELDS,
        MachineApiEndpoint::SearchForGoal => SEARCH_FOR_GOAL_FIELDS,
        MachineApiEndpoint::PromptPayload => PROMPT_PAYLOAD_FIELDS,
        MachineApiEndpoint::Replay => REPLAY_FIELDS,
        MachineApiEndpoint::Verify => VERIFY_FIELDS,
    }
}

fn validate_endpoint_field(
    _endpoint: MachineApiEndpoint,
    field: &MachineEndpointFieldSpec,
    value: &JsonValue<'_>,
    path: &JsonPath,
) -> Result<(), MachineApiRequestError> {
    if value.kind() == JsonValueKind::Null {
        return Err(MachineApiRequestError::new(
            field.error_kind,
            path.clone(),
            MachineApiRequestErrorReason::NullField { field: field.name },
        ));
    }

    if let MachineEndpointFieldType::UnsignedInteger { min, max } = field.field_type {
        let Some(raw) = value.number_raw() else {
            return Err(endpoint_type_mismatch(field, value, path));
        };
        return parse_strict_u64_token(raw, max)
            .and_then(|parsed| {
                if parsed >= min {
                    Ok(parsed)
                } else {
                    Err(StrictUnsignedIntegerError::InvalidGrammar)
                }
            })
            .map(|_| ())
            .map_err(|error| {
                MachineApiRequestError::new(
                    field.error_kind,
                    path.clone(),
                    MachineApiRequestErrorReason::InvalidUnsignedInteger {
                        field: field.name,
                        raw: raw.to_owned(),
                        error,
                    },
                )
            });
    }

    let grammar_result = match field.field_type {
        MachineEndpointFieldType::Object if value.kind() == JsonValueKind::Object => Ok(()),
        MachineEndpointFieldType::Array if value.kind() == JsonValueKind::Array => Ok(()),
        MachineEndpointFieldType::String if value.kind() == JsonValueKind::String => Ok(()),
        MachineEndpointFieldType::Boolean if value.kind() == JsonValueKind::Bool => Ok(()),
        MachineEndpointFieldType::SessionId => value
            .string_value()
            .ok_or(())
            .and_then(|text| SessionId::parse(text).map(|_| ()).map_err(|_| ())),
        MachineEndpointFieldType::SnapshotId => value
            .string_value()
            .ok_or(())
            .and_then(|text| SnapshotId::parse(text).map(|_| ()).map_err(|_| ())),
        MachineEndpointFieldType::HashString => value
            .string_value()
            .ok_or(())
            .and_then(|text| HashString::parse(text).map(|_| ()).map_err(|_| ())),
        MachineEndpointFieldType::GoalId => value
            .string_value()
            .ok_or(())
            .and_then(|text| parse_goal_id_wire(text).map(|_| ()).map_err(|_| ())),
        MachineEndpointFieldType::ProtocolVersion => value
            .string_value()
            .ok_or(())
            .and_then(|text| MachineApiVersion::parse(text).map(|_| ()).map_err(|_| ())),
        MachineEndpointFieldType::VerifyMode => value.string_value().ok_or(()).and_then(|text| {
            if text == "certificate" {
                Ok(())
            } else {
                Err(())
            }
        }),
        _ => Err(()),
    };

    grammar_result.map_err(|_| endpoint_type_mismatch(field, value, path))
}

fn endpoint_type_mismatch(
    field: &MachineEndpointFieldSpec,
    value: &JsonValue<'_>,
    path: &JsonPath,
) -> MachineApiRequestError {
    MachineApiRequestError::new(
        field.error_kind,
        path.clone(),
        MachineApiRequestErrorReason::TypeMismatch {
            field: field.name,
            expected: json_field_type_for_endpoint_field(field.field_type),
            actual: value.kind(),
        },
    )
}

fn json_field_type_for_endpoint_field(
    field_type: MachineEndpointFieldType,
) -> crate::JsonFieldType {
    match field_type {
        MachineEndpointFieldType::Object => crate::JsonFieldType::Object,
        MachineEndpointFieldType::Array => crate::JsonFieldType::Array,
        MachineEndpointFieldType::String
        | MachineEndpointFieldType::SessionId
        | MachineEndpointFieldType::SnapshotId
        | MachineEndpointFieldType::HashString
        | MachineEndpointFieldType::GoalId
        | MachineEndpointFieldType::ProtocolVersion
        | MachineEndpointFieldType::VerifyMode => crate::JsonFieldType::String,
        MachineEndpointFieldType::UnsignedInteger { max, .. } => {
            crate::JsonFieldType::UnsignedInteger { max }
        }
        MachineEndpointFieldType::Boolean => crate::JsonFieldType::Boolean,
    }
}

fn validate_session_id(value: &str) -> Result<(), MachineWireGrammarError> {
    let suffix = value
        .strip_prefix("msess_")
        .ok_or_else(|| MachineWireGrammarError::new(MachineWireGrammarErrorKind::InvalidPrefix))?;
    if suffix.is_empty() || suffix.len() > 64 {
        return Err(MachineWireGrammarError::new(
            MachineWireGrammarErrorKind::InvalidLength,
        ));
    }
    if suffix
        .as_bytes()
        .iter()
        .all(|byte| matches!(byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'.' | b'_' | b'-'))
    {
        Ok(())
    } else {
        Err(MachineWireGrammarError::new(
            MachineWireGrammarErrorKind::InvalidCharacter,
        ))
    }
}

fn parse_hex_digest(value: &str) -> Result<Hash, MachineWireGrammarError> {
    if value.len() != 64 {
        return Err(MachineWireGrammarError::new(
            MachineWireGrammarErrorKind::InvalidLength,
        ));
    }
    let mut out = [0u8; 32];
    for (index, chunk) in value.as_bytes().chunks_exact(2).enumerate() {
        let high = lowercase_hex_value(chunk[0])?;
        let low = lowercase_hex_value(chunk[1])?;
        out[index] = (high << 4) | low;
    }
    Ok(out)
}

fn lowercase_hex_value(byte: u8) -> Result<u8, MachineWireGrammarError> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Err(MachineWireGrammarError::new(
            MachineWireGrammarErrorKind::InvalidHex,
        )),
        _ => Err(MachineWireGrammarError::new(
            MachineWireGrammarErrorKind::InvalidHex,
        )),
    }
}

fn lower_hex_hash(hash: &Hash) -> String {
    let mut out = String::with_capacity(64);
    for byte in hash {
        out.push(hex_digit(byte >> 4));
        out.push(hex_digit(byte & 0x0f));
    }
    out
}

fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'a' + value - 10),
        _ => unreachable!("hex nybble is in range"),
    }
}

fn strip_prefixed_decimal(value: &str, prefix: char) -> Result<&str, MachineWireGrammarError> {
    value
        .strip_prefix(prefix)
        .ok_or_else(|| MachineWireGrammarError::new(MachineWireGrammarErrorKind::InvalidPrefix))
}

fn parse_decimal_u64(value: &str) -> Result<u64, MachineWireGrammarError> {
    if value.is_empty() {
        return Err(MachineWireGrammarError::new(
            MachineWireGrammarErrorKind::InvalidDecimal,
        ));
    }
    if value.len() > 1 && value.as_bytes()[0] == b'0' {
        return Err(MachineWireGrammarError::new(
            MachineWireGrammarErrorKind::LeadingZero,
        ));
    }
    let mut out = 0u64;
    for byte in value.as_bytes() {
        if !byte.is_ascii_digit() {
            return Err(MachineWireGrammarError::new(
                MachineWireGrammarErrorKind::InvalidDecimal,
            ));
        }
        out = out
            .checked_mul(10)
            .and_then(|prefix| prefix.checked_add(u64::from(byte - b'0')))
            .ok_or_else(|| MachineWireGrammarError::new(MachineWireGrammarErrorKind::Overflow))?;
    }
    Ok(out)
}

fn is_machine_surface_reserved(value: &str) -> bool {
    matches!(
        value,
        "import"
            | "def"
            | "theorem"
            | "forall"
            | "fun"
            | "let"
            | "in"
            | "Prop"
            | "Type"
            | "Sort"
            | "succ"
            | "max"
            | "imax"
            | "open"
            | "namespace"
            | "match"
            | "with"
    )
}

fn encode_string(out: &mut Vec<u8>, value: &str) {
    encode_uvar(out, value.len() as u64);
    out.extend_from_slice(value.as_bytes());
}

fn encode_name(out: &mut Vec<u8>, name: &Name) {
    encode_uvar(out, name.0.len() as u64);
    for component in &name.0 {
        encode_string(out, component);
    }
}

fn encode_list_len(out: &mut Vec<u8>, len: usize) {
    encode_uvar(out, len as u64);
}

fn encode_hash(out: &mut Vec<u8>, hash: &Hash) {
    out.extend_from_slice(hash);
}

fn encode_uvar(out: &mut Vec<u8>, mut value: u64) {
    while value >= 0x80 {
        out.push((value as u8 & 0x7f) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse_request_body, MachineApiRequestErrorReason, Phase5UpstreamDiagnostic};

    #[test]
    fn hash_and_snapshot_wire_grammar_is_canonical_lowercase_sha256() {
        let mut hash = [0u8; 32];
        hash[0] = 0xab;
        hash[31] = 0x05;

        let wire = format_hash_string(&hash);
        assert_eq!(
            wire,
            "sha256:ab00000000000000000000000000000000000000000000000000000000000005"
        );
        assert_eq!(parse_hash_string(&wire), Ok(hash));
        assert_eq!(
            parse_hash_string(
                "sha256:AB00000000000000000000000000000000000000000000000000000000000005"
            )
            .unwrap_err()
            .kind,
            MachineWireGrammarErrorKind::InvalidHex
        );

        let snapshot = SnapshotId::from_state_fingerprint(hash);
        assert_eq!(
            snapshot.wire(),
            "mst_ab00000000000000000000000000000000000000000000000000000000000005"
        );
        assert_eq!(SnapshotId::parse(&snapshot.wire()).unwrap(), snapshot);
    }

    #[test]
    fn id_wire_grammar_rejects_noncanonical_decimal_forms() {
        assert_eq!(parse_goal_id_wire("g0"), Ok(GoalId(0)));
        assert_eq!(parse_meta_var_id_wire("m42"), Ok(MetaVarId(42)));
        assert_eq!(parse_local_id_wire("l7"), Ok(LocalId(7)));
        assert_eq!(
            parse_goal_id_wire("g01").unwrap_err().kind,
            MachineWireGrammarErrorKind::LeadingZero
        );
        assert_eq!(
            parse_goal_id_wire("x1").unwrap_err().kind,
            MachineWireGrammarErrorKind::InvalidPrefix
        );
        assert_eq!(
            parse_local_id_wire("l4294967296").unwrap_err().kind,
            MachineWireGrammarErrorKind::Overflow
        );
    }

    #[test]
    fn session_id_wire_grammar_matches_mvp_regex() {
        assert!(SessionId::parse("msess_aZ09._-").is_ok());
        assert!(SessionId::parse("msess_").is_err());
        assert!(SessionId::parse("msess_space bad").is_err());
        assert!(SessionId::parse("other_a").is_err());
    }

    #[test]
    fn phase5_names_and_renderable_names_are_distinct() {
        assert_eq!(
            parse_phase5_name("Std.Nat.Basic").unwrap().as_dotted(),
            "Std.Nat.Basic"
        );
        assert!(parse_phase5_name("Std..Nat").is_err());
        assert!(parse_machine_surface_renderable_name_wire("Nat.succ").is_ok());
        assert!(parse_machine_surface_renderable_name_wire("Prop").is_err());
        assert!(is_machine_universe_param_name("u"));
        assert!(!is_machine_universe_param_name("forall"));
        assert!(is_machine_local_name("x'"));
        assert!(!is_machine_local_name("Nat.x"));
    }

    #[test]
    fn endpoint_envelope_classifies_tactic_run_budget_before_scheduler() {
        let doc = parse_request_body(
            r#"{
              "session_id":"msess_1",
              "snapshot_id":"mst_0000000000000000000000000000000000000000000000000000000000000000",
              "state_fingerprint":"sha256:0000000000000000000000000000000000000000000000000000000000000000",
              "goal_id":"g0",
              "candidate":{},
              "scheduler_limits": null
            }"#,
            MachineApiErrorKind::InvalidTacticRunRequest,
        )
        .unwrap();

        let err = validate_machine_endpoint_envelope(
            doc.root(),
            MachineApiEndpoint::TacticRun,
            &JsonPath::root(),
        )
        .unwrap_err();

        assert_eq!(err.kind, MachineApiErrorKind::InvalidBudget);
        assert_eq!(
            err.reason,
            MachineApiRequestErrorReason::MissingField {
                field: "deterministic_budget"
            }
        );
    }

    #[test]
    fn endpoint_envelope_classifies_scheduler_limits_with_scheduler_error_kind() {
        let doc = parse_request_body(
            r#"{
              "session_id":"msess_1",
              "snapshot_id":"mst_0000000000000000000000000000000000000000000000000000000000000000",
              "state_fingerprint":"sha256:0000000000000000000000000000000000000000000000000000000000000000",
              "goal_id":"g0",
              "candidate":{},
              "deterministic_budget":{},
              "scheduler_limits": null
            }"#,
            MachineApiErrorKind::InvalidTacticRunRequest,
        )
        .unwrap();

        let err = validate_machine_endpoint_envelope(
            doc.root(),
            MachineApiEndpoint::TacticRun,
            &JsonPath::root(),
        )
        .unwrap_err();

        assert_eq!(err.kind, MachineApiErrorKind::InvalidSchedulerLimits);
        assert_eq!(
            err.reason,
            MachineApiRequestErrorReason::NullField {
                field: "scheduler_limits"
            }
        );
    }

    #[test]
    fn endpoint_envelope_uses_endpoint_field_order_not_request_order() {
        let doc = parse_request_body(
            r#"{
              "session_id":"msess_1",
              "snapshot_id":"mst_0000000000000000000000000000000000000000000000000000000000000000",
              "state_fingerprint":"sha256:0000000000000000000000000000000000000000000000000000000000000000",
              "scheduler_limits": null,
              "goal_id":"g01",
              "candidate":{},
              "deterministic_budget":{}
            }"#,
            MachineApiErrorKind::InvalidTacticRunRequest,
        )
        .unwrap();

        let err = validate_machine_endpoint_envelope(
            doc.root(),
            MachineApiEndpoint::TacticRun,
            &JsonPath::root(),
        )
        .unwrap_err();

        assert_eq!(err.kind, MachineApiErrorKind::InvalidTacticRunRequest);
        assert_eq!(
            err.reason,
            MachineApiRequestErrorReason::TypeMismatch {
                field: "goal_id",
                expected: crate::JsonFieldType::String,
                actual: JsonValueKind::String
            }
        );
    }

    #[test]
    fn endpoint_envelope_validates_early_id_grammar_before_later_missing_fields() {
        let doc = parse_request_body(
            r#"{
              "session_id":"msess_1",
              "snapshot_id":"mst_0000000000000000000000000000000000000000000000000000000000000000",
              "state_fingerprint":"sha256:0000000000000000000000000000000000000000000000000000000000000000",
              "goal_id":"g01",
              "candidate":{}
            }"#,
            MachineApiErrorKind::InvalidTacticRunRequest,
        )
        .unwrap();

        let err = validate_machine_endpoint_envelope(
            doc.root(),
            MachineApiEndpoint::TacticRun,
            &JsonPath::root(),
        )
        .unwrap_err();

        assert_eq!(err.kind, MachineApiErrorKind::InvalidTacticRunRequest);
        assert_eq!(
            err.reason,
            MachineApiRequestErrorReason::TypeMismatch {
                field: "goal_id",
                expected: crate::JsonFieldType::String,
                actual: JsonValueKind::String
            }
        );
    }

    #[test]
    fn endpoint_envelope_rejects_unknown_and_duplicate_keys_with_endpoint_kind() {
        let doc = parse_request_body(
            r#"{"session_id":"msess_1","session_id":"msess_2"}"#,
            MachineApiErrorKind::InvalidReplayPlan,
        )
        .unwrap();
        let err = validate_machine_endpoint_envelope(
            doc.root(),
            MachineApiEndpoint::Replay,
            &JsonPath::root(),
        )
        .unwrap_err();
        assert_eq!(err.kind, MachineApiErrorKind::InvalidReplayPlan);
        assert_eq!(
            err.reason,
            MachineApiRequestErrorReason::DuplicateKey {
                key: "session_id".to_owned()
            }
        );

        let doc = parse_request_body(
            r#"{"extra":true}"#,
            MachineApiErrorKind::InvalidVerifyRequest,
        )
        .unwrap();
        let err = validate_machine_endpoint_envelope(
            doc.root(),
            MachineApiEndpoint::Verify,
            &JsonPath::root(),
        )
        .unwrap_err();
        assert_eq!(err.kind, MachineApiErrorKind::InvalidVerifyRequest);
        assert_eq!(
            err.reason,
            MachineApiRequestErrorReason::UnknownField {
                field: "extra".to_owned()
            }
        );
    }

    #[test]
    fn endpoint_envelope_validates_hash_and_id_grammar_before_lookup() {
        let doc = parse_request_body(
            r#"{
              "session_id":"msess_1",
              "snapshot_id":"mst_0000000000000000000000000000000000000000000000000000000000000000",
              "state_fingerprint":"sha256:0000000000000000000000000000000000000000000000000000000000000000",
              "goal_id":"g01",
              "modes":["exact"],
              "limit":20,
              "filters":{}
            }"#,
            MachineApiErrorKind::InvalidTheoremQuery,
        )
        .unwrap();

        let err = validate_machine_endpoint_envelope(
            doc.root(),
            MachineApiEndpoint::SearchForGoal,
            &JsonPath::root(),
        )
        .unwrap_err();
        assert_eq!(err.kind, MachineApiErrorKind::InvalidTheoremQuery);
        assert_eq!(
            err.reason,
            MachineApiRequestErrorReason::TypeMismatch {
                field: "goal_id",
                expected: crate::JsonFieldType::String,
                actual: JsonValueKind::String
            }
        );
    }

    #[test]
    fn error_wire_projects_canonical_diagnostic_fields() {
        let diagnostic = MachineApiDiagnosticProjection {
            kind: MachineApiErrorKind::GoalNotOpen,
            phase: MachineApiDiagnosticPhase::SnapshotLookup,
            retryable: false,
            goal_id: Some(GoalId(7)),
            tactic_kind: None,
            primary_name: None,
            primary_axiom_ref: None,
            expected_hash: None,
            actual_hash: None,
            source_message: "goal is not open".to_owned(),
            upstream: Phase5UpstreamDiagnostic::Phase4(npa_tactic::MachineTacticDiagnostic::new(
                npa_tactic::MachineTacticDiagnosticKind::UnknownGoal,
                "goal is not open",
            )),
        };

        let wire = MachineApiErrorWire::from_projection(&diagnostic).unwrap();
        assert_eq!(wire.kind, MachineApiErrorKind::GoalNotOpen);
        assert_eq!(wire.phase, MachineApiDiagnosticPhase::SnapshotLookup);
        assert_eq!(wire.goal_id, Some(GoalId(7)));
        assert_eq!(wire.diagnostic_hash, diagnostic.diagnostic_hash().unwrap());

        let compact: MachineApiCompactErrorWire = wire.into();
        assert_eq!(compact.error_kind, MachineApiErrorKind::GoalNotOpen);
        assert_eq!(compact.goal_id, Some(GoalId(7)));
    }

    #[test]
    fn response_envelope_allows_endpoint_specific_top_level_fields() {
        #[derive(Clone, Debug, PartialEq, Eq)]
        struct TacticRunErrorFields {
            unchanged_state_fingerprint: Hash,
        }

        #[derive(Clone, Debug, PartialEq, Eq)]
        struct TacticRunErrorObject {
            diagnostic: MachineApiErrorWire,
            candidate_hash: Hash,
            deterministic_budget_hash: Hash,
        }

        let zero = [0u8; 32];
        let diagnostic = MachineApiDiagnosticProjection {
            kind: MachineApiErrorKind::GoalNotOpen,
            phase: MachineApiDiagnosticPhase::SnapshotLookup,
            retryable: false,
            goal_id: Some(GoalId(7)),
            tactic_kind: None,
            primary_name: None,
            primary_axiom_ref: None,
            expected_hash: None,
            actual_hash: None,
            source_message: "goal is not open".to_owned(),
            upstream: Phase5UpstreamDiagnostic::Phase4(npa_tactic::MachineTacticDiagnostic::new(
                npa_tactic::MachineTacticDiagnosticKind::UnknownGoal,
                "goal is not open",
            )),
        };
        let error = TacticRunErrorObject {
            diagnostic: MachineApiErrorWire::from_projection(&diagnostic).unwrap(),
            candidate_hash: zero,
            deterministic_budget_hash: zero,
        };
        let envelope: MachineApiResponseEnvelope<(), TacticRunErrorObject, TacticRunErrorFields> =
            MachineApiResponseEnvelope::Error(Box::new(MachineApiErrorResponse {
                status: MachineApiResponseStatus::Error,
                error,
                endpoint_fields: TacticRunErrorFields {
                    unchanged_state_fingerprint: zero,
                },
            }));

        match envelope {
            MachineApiResponseEnvelope::Error(response) => {
                assert_eq!(response.status, MachineApiResponseStatus::Error);
                assert_eq!(response.error.candidate_hash, zero);
                assert_eq!(response.endpoint_fields.unchanged_state_fingerprint, zero);
            }
            _ => panic!("expected an error response"),
        }
    }

    #[test]
    fn scheduler_response_envelope_allows_endpoint_specific_top_level_fields() {
        #[derive(Clone, Debug, PartialEq, Eq)]
        struct TacticRunSchedulerFields {
            previous_state_fingerprint: Hash,
            deterministic_budget_hash: Hash,
        }

        #[derive(Clone, Debug, PartialEq, Eq)]
        struct BatchPartialFields {
            previous_state_fingerprint: Hash,
            deterministic_budget_hash: Hash,
            completed_prefix_len: u32,
        }

        let zero = [0u8; 32];
        let artifact = MachineSchedulerArtifact {
            kind: MachineSchedulerArtifactKind::Timeout,
            scope: MachineSchedulerArtifactScope::Candidate,
            retryable: true,
        };
        let envelope: MachineApiResponseEnvelope<
            (),
            MachineApiErrorWire,
            (),
            TacticRunSchedulerFields,
        > = MachineApiResponseEnvelope::SchedulerStopped(MachineApiSchedulerResponse {
            status: MachineApiResponseStatus::SchedulerStopped,
            scheduler_artifact: artifact.clone(),
            endpoint_fields: TacticRunSchedulerFields {
                previous_state_fingerprint: zero,
                deterministic_budget_hash: zero,
            },
        });

        match envelope {
            MachineApiResponseEnvelope::SchedulerStopped(response) => {
                assert_eq!(response.status, MachineApiResponseStatus::SchedulerStopped);
                assert_eq!(response.scheduler_artifact, artifact);
                assert_eq!(response.endpoint_fields.previous_state_fingerprint, zero);
                assert_eq!(response.endpoint_fields.deterministic_budget_hash, zero);
            }
            _ => panic!("expected a scheduler response"),
        }

        let batch_artifact = MachineSchedulerArtifact {
            kind: MachineSchedulerArtifactKind::ResourceLimitExceeded,
            scope: MachineSchedulerArtifactScope::Batch,
            retryable: true,
        };
        let envelope: MachineApiResponseEnvelope<(), MachineApiErrorWire, (), BatchPartialFields> =
            MachineApiResponseEnvelope::SchedulerStopped(MachineApiSchedulerResponse {
                status: MachineApiResponseStatus::PartialResourceLimit,
                scheduler_artifact: batch_artifact,
                endpoint_fields: BatchPartialFields {
                    previous_state_fingerprint: zero,
                    deterministic_budget_hash: zero,
                    completed_prefix_len: 1,
                },
            });

        match envelope {
            MachineApiResponseEnvelope::SchedulerStopped(response) => {
                assert_eq!(
                    response.status,
                    MachineApiResponseStatus::PartialResourceLimit
                );
                assert_eq!(response.endpoint_fields.completed_prefix_len, 1);
            }
            _ => panic!("expected a partial scheduler response"),
        }
    }
}
