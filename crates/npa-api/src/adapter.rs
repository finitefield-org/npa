use npa_cert::{Hash, Name};
use npa_kernel::Decl;
use npa_tactic::{
    extract_closed_machine_theorem_decl, machine_tactic_cache_key, machine_tactic_cache_key_hash,
    machine_tactic_candidate_kind, machine_tactic_goal_id, machine_tactic_hash,
    run_machine_tactic_with_budget, start_machine_proof, tactic_budget_hash,
    validate_machine_proof_state, validate_machine_tactic_candidate, CandidateApplyArg,
    CandidateRewriteRuleRef, CheckedCurrentDecl, GoalId, MachineProofDelta, MachineProofSpec,
    MachineProofState, MachineTactic, MachineTacticCandidate, MachineTacticDiagnostic,
    MachineTacticDiagnosticKind, MachineTacticOptions, RawMachineTerm, TacticBudget,
    TacticFuelKind, VerifiedImportRef,
};

use crate::current::MachineAxiomRefWire;
use crate::MachineApiErrorKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MachineApiDiagnosticPhase {
    RequestValidation,
    SessionLookup,
    SessionCreate,
    SnapshotLookup,
    CandidateValidation,
    MachineTermParse,
    MachineTermCheck,
    TacticExecution,
    TheoremSearch,
    PromptPayload,
    ReplayValidation,
    ReplayExecution,
    KernelCheck,
    CertificateGeneration,
    CertificateVerify,
}

impl MachineApiDiagnosticPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RequestValidation => "request_validation",
            Self::SessionLookup => "session_lookup",
            Self::SessionCreate => "session_create",
            Self::SnapshotLookup => "snapshot_lookup",
            Self::CandidateValidation => "candidate_validation",
            Self::MachineTermParse => "machine_term_parse",
            Self::MachineTermCheck => "machine_term_check",
            Self::TacticExecution => "tactic_execution",
            Self::TheoremSearch => "theorem_search",
            Self::PromptPayload => "prompt_payload",
            Self::ReplayValidation => "replay_validation",
            Self::ReplayExecution => "replay_execution",
            Self::KernelCheck => "kernel_check",
            Self::CertificateGeneration => "certificate_generation",
            Self::CertificateVerify => "certificate_verify",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MachineApiTacticKind {
    Intro,
    Exact,
    Apply,
    Rw,
    SimpLite,
    InductionNat,
}

impl MachineApiTacticKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Intro => "intro",
            Self::Exact => "exact",
            Self::Apply => "apply",
            Self::Rw => "rw",
            Self::SimpLite => "simp-lite",
            Self::InductionNat => "induction-nat",
        }
    }

    fn from_phase4_kind(kind: &str) -> Option<Self> {
        match kind {
            "intro" => Some(Self::Intro),
            "exact" => Some(Self::Exact),
            "apply" => Some(Self::Apply),
            "rw" => Some(Self::Rw),
            "simp-lite" => Some(Self::SimpLite),
            "induction-nat" => Some(Self::InductionNat),
            _ => None,
        }
    }

    fn from_candidate(candidate: &MachineTacticCandidate) -> Self {
        Self::from_phase4_kind(machine_tactic_candidate_kind(candidate))
            .expect("Phase 4 exposes only MVP tactic candidate kinds")
    }

    fn from_tactic(tactic: &MachineTactic) -> Self {
        match tactic {
            MachineTactic::Exact { .. } => Self::Exact,
            MachineTactic::Intro { .. } => Self::Intro,
            MachineTactic::Apply { .. } => Self::Apply,
            MachineTactic::Rewrite { .. } => Self::Rw,
            MachineTactic::SimpLite { .. } => Self::SimpLite,
            MachineTactic::InductionNat { .. } => Self::InductionNat,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase5UpstreamDiagnostic {
    Phase3(npa_frontend::MachineDiagnostic),
    Phase4(MachineTacticDiagnostic),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineApiDiagnosticProjection {
    pub kind: MachineApiErrorKind,
    pub phase: MachineApiDiagnosticPhase,
    pub retryable: bool,
    pub goal_id: Option<GoalId>,
    pub tactic_kind: Option<MachineApiTacticKind>,
    pub primary_name: Option<Name>,
    pub primary_axiom_ref: Option<MachineAxiomRefWire>,
    pub expected_hash: Option<Hash>,
    pub actual_hash: Option<Hash>,
    pub source_message: String,
    pub upstream: Phase5UpstreamDiagnostic,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase4AdapterError {
    pub diagnostic: MachineApiDiagnosticProjection,
    pub candidate_hash: Option<Hash>,
    pub deterministic_budget_hash: Option<Hash>,
    pub cache_key_hash: Option<Hash>,
}

pub type Phase4AdapterResult<T> = Result<T, Box<Phase4AdapterError>>;

#[derive(Clone, Debug)]
pub struct Phase4StartProofOutput {
    pub state: MachineProofState,
    pub state_fingerprint: Hash,
    pub options_fingerprint: Hash,
    pub env_fingerprint: Hash,
    pub simp_registry_hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase4ValidatedTactic {
    pub tactic: MachineTactic,
    pub goal_id: GoalId,
    pub tactic_kind: MachineApiTacticKind,
    pub candidate_hash: Hash,
}

#[derive(Clone, Debug)]
pub struct Phase4TacticRunOutput {
    pub state: MachineProofState,
    pub delta: MachineProofDelta,
    pub goal_id: GoalId,
    pub tactic_kind: MachineApiTacticKind,
    pub candidate_hash: Hash,
    pub deterministic_budget_hash: Hash,
    pub cache_key_hash: Hash,
    pub next_state_fingerprint: Hash,
    pub proof_delta_hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phase4ExtractedTheorem {
    pub theorem: Decl,
}

pub fn phase4_start_machine_proof(
    spec: MachineProofSpec,
    imports: Vec<VerifiedImportRef>,
    checked_current_decls: Vec<CheckedCurrentDecl>,
    options: MachineTacticOptions,
) -> Phase4AdapterResult<Phase4StartProofOutput> {
    start_machine_proof(spec, imports, checked_current_decls, options)
        .map(|state| Phase4StartProofOutput {
            state_fingerprint: state.fingerprint,
            options_fingerprint: state.env.options_fingerprint,
            env_fingerprint: state.env.env_fingerprint,
            simp_registry_hash: npa_tactic::simp_registry_hash(&state.env.simp_registry),
            state,
        })
        .map_err(|diagnostic| {
            let phase = phase4_start_proof_phase(&diagnostic);
            phase4_error(diagnostic, phase)
        })
}

pub fn phase4_validate_machine_tactic_candidate(
    goal_id: GoalId,
    candidate: MachineTacticCandidate,
) -> Phase4AdapterResult<Phase4ValidatedTactic> {
    let tactic_kind = MachineApiTacticKind::from_candidate(&candidate);
    prepass_candidate_terms(&candidate, goal_id, tactic_kind)?;
    validate_machine_tactic_candidate(goal_id, candidate)
        .map(|tactic| {
            let candidate_hash = machine_tactic_hash(&tactic);
            Phase4ValidatedTactic {
                tactic,
                goal_id,
                tactic_kind,
                candidate_hash,
            }
        })
        .map_err(|diagnostic| phase4_candidate_validation_error(diagnostic, goal_id, tactic_kind))
}

pub fn phase4_run_machine_tactic(
    state: &MachineProofState,
    tactic: MachineTactic,
) -> Phase4AdapterResult<Phase4TacticRunOutput> {
    phase4_run_machine_tactic_with_budget(state, tactic, TacticBudget::default())
}

pub fn phase4_run_machine_tactic_with_budget(
    state: &MachineProofState,
    tactic: MachineTactic,
    budget: TacticBudget,
) -> Phase4AdapterResult<Phase4TacticRunOutput> {
    if let Err(diagnostic) = validate_machine_proof_state(state) {
        return Err(phase4_error(
            diagnostic,
            MachineApiDiagnosticPhase::SnapshotLookup,
        ));
    }

    let goal_id = machine_tactic_goal_id(&tactic);
    let tactic_kind = MachineApiTacticKind::from_tactic(&tactic);
    let candidate_hash = machine_tactic_hash(&tactic);
    let deterministic_budget_hash = tactic_budget_hash(budget);
    let cache_key_hash =
        machine_tactic_cache_key_hash(&machine_tactic_cache_key(state, &tactic, budget));
    run_machine_tactic_with_budget(state, tactic, budget)
        .map(|(state, delta)| Phase4TacticRunOutput {
            next_state_fingerprint: state.fingerprint,
            proof_delta_hash: delta.delta_hash,
            state,
            delta,
            goal_id,
            tactic_kind,
            candidate_hash,
            deterministic_budget_hash,
            cache_key_hash,
        })
        .map_err(|diagnostic| {
            let include_correlation_hashes = tactic_run_correlation_hashes_allowed(&diagnostic);
            let phase = phase4_tactic_run_phase(&diagnostic);
            let mut error = phase4_error(diagnostic, phase);
            if include_correlation_hashes {
                error.candidate_hash = Some(candidate_hash);
                error.deterministic_budget_hash = Some(deterministic_budget_hash);
                error.cache_key_hash = Some(cache_key_hash);
            }
            error
        })
}

pub fn phase4_extract_closed_machine_theorem_decl(
    state: &MachineProofState,
    phase: MachineApiDiagnosticPhase,
) -> Phase4AdapterResult<Phase4ExtractedTheorem> {
    extract_closed_machine_theorem_decl(state)
        .map(|theorem| Phase4ExtractedTheorem { theorem })
        .map_err(|diagnostic| phase4_error(diagnostic, phase))
}

pub fn phase4_machine_tactic_result_error(
    state: &MachineProofState,
    tactic: MachineTactic,
    budget: TacticBudget,
) -> Option<Box<Phase4AdapterError>> {
    if let Err(diagnostic) = validate_machine_proof_state(state) {
        return Some(phase4_error(
            diagnostic,
            MachineApiDiagnosticPhase::SnapshotLookup,
        ));
    }

    match run_machine_tactic_with_budget(state, tactic.clone(), budget) {
        Ok(_) => None,
        Err(diagnostic) => {
            let candidate_hash = machine_tactic_hash(&tactic);
            let deterministic_budget_hash = tactic_budget_hash(budget);
            let cache_key_hash =
                machine_tactic_cache_key_hash(&machine_tactic_cache_key(state, &tactic, budget));
            let include_correlation_hashes = tactic_run_correlation_hashes_allowed(&diagnostic);
            let phase = phase4_tactic_run_phase(&diagnostic);
            let mut error = phase4_error(diagnostic, phase);
            if include_correlation_hashes {
                error.candidate_hash = Some(candidate_hash);
                error.deterministic_budget_hash = Some(deterministic_budget_hash);
                error.cache_key_hash = Some(cache_key_hash);
            }
            Some(error)
        }
    }
}

pub fn map_phase4_diagnostic_kind(diagnostic: &MachineTacticDiagnostic) -> MachineApiErrorKind {
    use MachineApiErrorKind as Api;
    use MachineTacticDiagnosticKind as Phase4;

    match &diagnostic.kind {
        Phase4::InvalidMachineProofState
        | Phase4::UnknownMeta
        | Phase4::InvalidMetaContext
        | Phase4::InvalidMetaDependency
        | Phase4::ProofExprScopeError
        | Phase4::UnresolvedGoal
        | Phase4::AmbiguousKernelEnvDecl
        | Phase4::KernelRejected
        | Phase4::InvalidMachineTermSource => Api::InvalidMachineProofState,
        Phase4::MachineTermElaborationError => Api::MachineTermElaborationError,
        Phase4::UnknownName => Api::UnknownName,
        Phase4::ImplicitArgumentRequired | Phase4::MissingExplicitArgument => {
            Api::ImplicitArgumentRequired
        }
        Phase4::InvalidMachineProofSpec => Api::InvalidSessionRequest,
        Phase4::InvalidMachineTactic => Api::InvalidCandidate,
        Phase4::InvalidBatchPolicy => Api::InvalidBatchPolicy,
        Phase4::UnknownGoal | Phase4::GoalAlreadyAssigned => Api::GoalNotOpen,
        Phase4::UnknownTacticHead
        | Phase4::AmbiguousTacticHead
        | Phase4::UnknownLocalName
        | Phase4::AmbiguousLocalName
        | Phase4::InvalidLocalHead
        | Phase4::UnknownSimpRule
        | Phase4::AmbiguousSimpRule
        | Phase4::InvalidSimpRule
        | Phase4::AmbiguousApplyArgument
        | Phase4::TooManyApplyArguments
        | Phase4::TooFewApplyArguments
        | Phase4::SubgoalDataArgument => Api::InvalidCandidate,
        Phase4::ExpectedFunctionType | Phase4::ExpectedPiTarget => Api::ExpectedPiType,
        Phase4::ExpectedEqTarget | Phase4::AmbiguousRewriteRule => Api::RewriteRuleInvalid,
        Phase4::UniverseArgumentMismatch | Phase4::TypeMismatch | Phase4::ProofExprTypeMismatch
            if has_expected_actual(diagnostic) =>
        {
            Api::TypeMismatch
        }
        Phase4::UniverseArgumentMismatch | Phase4::TypeMismatch => Api::MachineTermElaborationError,
        Phase4::ProofExprTypeMismatch => Api::InvalidMachineProofState,
        Phase4::SimpNoProgress => Api::SimpNoProgress,
        Phase4::SimpStepLimitExceeded => Api::BudgetExceeded,
        Phase4::UnsupportedMachineTactic | Phase4::TacticPrimitiveUnavailable => {
            Api::UnsupportedTactic
        }
        Phase4::InvalidInductionTarget => Api::InductionTargetNotNat,
        Phase4::GoalLimitExceeded => Api::TooManyGoals,
        Phase4::MetaLimitExceeded => Api::BudgetExceeded,
        Phase4::TacticFuelExhausted {
            kind: TacticFuelKind::ExprNode,
        } => Api::TooLargeTerm,
        Phase4::TacticFuelExhausted { .. } => Api::BudgetExceeded,
        Phase4::InvalidCurrentDeclOrder
        | Phase4::UncheckedCurrentDecl
        | Phase4::CurrentDeclSignatureMismatch => Api::InvalidCheckedCurrentDecl,
        Phase4::InvalidVerifiedImport => Api::InvalidVerifiedImport,
        Phase4::InvalidTacticOption
        | Phase4::UnsupportedTacticOption
        | Phase4::InvalidEqFamily
        | Phase4::InvalidNatFamily => Api::InvalidMachineApiOptions,
    }
}

pub fn map_phase3_diagnostic_kind(
    diagnostic: &npa_frontend::MachineDiagnostic,
) -> MachineApiErrorKind {
    use npa_frontend::MachineDiagnosticKind as Phase3;
    use MachineApiErrorKind as Api;

    match diagnostic.kind {
        Phase3::ParseError => Api::MachineTermParseError,
        Phase3::UnsupportedSyntax
        | Phase3::UnsupportedItem
        | Phase3::ImportAfterItem
        | Phase3::ImportResolutionError
        | Phase3::MissingVerifiedImport
        | Phase3::DuplicateDeclaration
        | Phase3::DuplicateUniverseParam
        | Phase3::UnknownUniverseParam
        | Phase3::UniverseLevelTooLarge
        | Phase3::UnannotatedBinder
        | Phase3::UnannotatedLet
        | Phase3::HoleNotAllowed
        | Phase3::UnsolvedUniverseMeta
        | Phase3::KernelRejected
        | Phase3::ExpectedSort
        | Phase3::TooManyArguments
        | Phase3::TooFewArguments => Api::MachineTermElaborationError,
        Phase3::UnknownGlobalName
        | Phase3::ShortGlobalName
        | Phase3::AmbiguousGlobalName
        | Phase3::GlobalShadowedByLocal
        | Phase3::UnknownLocalName => Api::UnknownName,
        Phase3::ImplicitArgumentRequired | Phase3::MissingExplicitUniverse => {
            Api::ImplicitArgumentRequired
        }
        Phase3::ExpectedFunctionType => Api::ExpectedPiType,
        Phase3::TypeMismatch if phase3_has_expected_actual(diagnostic) => Api::TypeMismatch,
        Phase3::TypeMismatch => Api::MachineTermElaborationError,
        Phase3::CertificateRejected => Api::VerifyFailed,
    }
}

fn prepass_candidate_terms(
    candidate: &MachineTacticCandidate,
    goal_id: GoalId,
    tactic_kind: MachineApiTacticKind,
) -> Phase4AdapterResult<()> {
    match candidate {
        MachineTacticCandidate::Exact { term } => prepass_raw_term(term, goal_id, tactic_kind),
        MachineTacticCandidate::Intro { .. }
        | MachineTacticCandidate::SimpLite { .. }
        | MachineTacticCandidate::InductionNat { .. } => Ok(()),
        MachineTacticCandidate::Apply { args, .. } => {
            for arg in args {
                if let CandidateApplyArg::Term(term) = arg {
                    prepass_raw_term(term, goal_id, tactic_kind)?;
                }
            }
            Ok(())
        }
        MachineTacticCandidate::Rewrite { rule, .. } => {
            prepass_rewrite_rule(rule, goal_id, tactic_kind)
        }
    }
}

fn prepass_rewrite_rule(
    rule: &CandidateRewriteRuleRef,
    goal_id: GoalId,
    tactic_kind: MachineApiTacticKind,
) -> Phase4AdapterResult<()> {
    for arg in &rule.args {
        if let CandidateApplyArg::Term(term) = arg {
            prepass_raw_term(term, goal_id, tactic_kind)?;
        }
    }
    Ok(())
}

fn prepass_raw_term(
    term: &RawMachineTerm,
    goal_id: GoalId,
    tactic_kind: MachineApiTacticKind,
) -> Phase4AdapterResult<()> {
    npa_frontend::canonicalize_machine_term_source(&term.source)
        .map(|_| ())
        .map_err(|diagnostic| {
            let phase = phase3_term_phase(&diagnostic);
            Box::new(Phase4AdapterError {
                diagnostic: project_phase3_diagnostic(
                    diagnostic,
                    phase,
                    Some(goal_id),
                    Some(tactic_kind),
                ),
                candidate_hash: None,
                deterministic_budget_hash: None,
                cache_key_hash: None,
            })
        })
}

fn phase4_candidate_validation_error(
    diagnostic: MachineTacticDiagnostic,
    goal_id: GoalId,
    tactic_kind: MachineApiTacticKind,
) -> Box<Phase4AdapterError> {
    let mut error = phase4_error(diagnostic, MachineApiDiagnosticPhase::CandidateValidation);
    error.diagnostic.goal_id = Some(goal_id);
    error.diagnostic.tactic_kind = Some(tactic_kind);
    error
}

fn phase4_error(
    diagnostic: MachineTacticDiagnostic,
    phase: MachineApiDiagnosticPhase,
) -> Box<Phase4AdapterError> {
    Box::new(Phase4AdapterError {
        diagnostic: project_phase4_diagnostic(diagnostic, phase),
        candidate_hash: None,
        deterministic_budget_hash: None,
        cache_key_hash: None,
    })
}

fn project_phase4_diagnostic(
    diagnostic: MachineTacticDiagnostic,
    phase: MachineApiDiagnosticPhase,
) -> MachineApiDiagnosticProjection {
    let kind = map_phase4_diagnostic_kind(&diagnostic);
    let (expected_hash, actual_hash) = mismatch_hashes_for_api(kind, &diagnostic);
    let goal_id = diagnostic.goal_id;
    let tactic_kind = phase4_tactic_kind_for_api(kind, &diagnostic);
    let primary_name = phase4_primary_name_for_api(kind, &diagnostic);
    let source_message = diagnostic.message.to_string();
    MachineApiDiagnosticProjection {
        kind,
        phase,
        retryable: false,
        goal_id,
        tactic_kind,
        primary_name,
        primary_axiom_ref: None,
        expected_hash,
        actual_hash,
        source_message,
        upstream: Phase5UpstreamDiagnostic::Phase4(diagnostic),
    }
}

fn project_phase3_diagnostic(
    diagnostic: npa_frontend::MachineDiagnostic,
    phase: MachineApiDiagnosticPhase,
    goal_id: Option<GoalId>,
    tactic_kind: Option<MachineApiTacticKind>,
) -> MachineApiDiagnosticProjection {
    let kind = map_phase3_diagnostic_kind(&diagnostic);
    let (expected_hash, actual_hash) = phase3_mismatch_hashes_for_api(kind, &diagnostic);
    let source_message = diagnostic.message.clone();
    MachineApiDiagnosticProjection {
        kind,
        phase,
        retryable: false,
        goal_id,
        tactic_kind,
        primary_name: None,
        primary_axiom_ref: None,
        expected_hash,
        actual_hash,
        source_message,
        upstream: Phase5UpstreamDiagnostic::Phase3(diagnostic),
    }
}

fn phase4_start_proof_phase(diagnostic: &MachineTacticDiagnostic) -> MachineApiDiagnosticPhase {
    match map_phase4_diagnostic_kind(diagnostic) {
        MachineApiErrorKind::MachineTermParseError => MachineApiDiagnosticPhase::MachineTermParse,
        MachineApiErrorKind::MachineTermElaborationError
        | MachineApiErrorKind::UnknownName
        | MachineApiErrorKind::ImplicitArgumentRequired
        | MachineApiErrorKind::TypeMismatch
        | MachineApiErrorKind::ExpectedPiType => MachineApiDiagnosticPhase::MachineTermCheck,
        _ => MachineApiDiagnosticPhase::SessionCreate,
    }
}

fn phase4_tactic_run_phase(diagnostic: &MachineTacticDiagnostic) -> MachineApiDiagnosticPhase {
    match diagnostic.kind {
        MachineTacticDiagnosticKind::UnknownGoal
        | MachineTacticDiagnosticKind::GoalAlreadyAssigned => {
            return MachineApiDiagnosticPhase::SnapshotLookup;
        }
        MachineTacticDiagnosticKind::InvalidMachineTermSource => {
            return MachineApiDiagnosticPhase::CandidateValidation;
        }
        _ => {}
    }

    match map_phase4_diagnostic_kind(diagnostic) {
        MachineApiErrorKind::InvalidCandidate => MachineApiDiagnosticPhase::CandidateValidation,
        MachineApiErrorKind::MachineTermParseError => MachineApiDiagnosticPhase::MachineTermParse,
        MachineApiErrorKind::MachineTermElaborationError
        | MachineApiErrorKind::UnknownName
        | MachineApiErrorKind::ImplicitArgumentRequired
        | MachineApiErrorKind::TypeMismatch
        | MachineApiErrorKind::ExpectedPiType => MachineApiDiagnosticPhase::MachineTermCheck,
        _ => MachineApiDiagnosticPhase::TacticExecution,
    }
}

fn tactic_run_correlation_hashes_allowed(diagnostic: &MachineTacticDiagnostic) -> bool {
    !matches!(
        diagnostic.kind,
        MachineTacticDiagnosticKind::UnknownGoal
            | MachineTacticDiagnosticKind::GoalAlreadyAssigned
            | MachineTacticDiagnosticKind::InvalidMachineTermSource
    )
}

fn phase4_tactic_kind_for_api(
    kind: MachineApiErrorKind,
    diagnostic: &MachineTacticDiagnostic,
) -> Option<MachineApiTacticKind> {
    if kind == MachineApiErrorKind::GoalNotOpen {
        return None;
    }

    diagnostic
        .tactic_kind
        .as_deref()
        .and_then(MachineApiTacticKind::from_phase4_kind)
}

fn phase4_primary_name_for_api(
    kind: MachineApiErrorKind,
    diagnostic: &MachineTacticDiagnostic,
) -> Option<Name> {
    if !matches!(
        kind,
        MachineApiErrorKind::InvalidCandidate
            | MachineApiErrorKind::InvalidMachineApiOptions
            | MachineApiErrorKind::MachineTermElaborationError
            | MachineApiErrorKind::UnknownName
            | MachineApiErrorKind::ImplicitArgumentRequired
            | MachineApiErrorKind::RewriteRuleInvalid
    ) {
        return None;
    }

    diagnostic
        .primary_name
        .as_ref()
        .filter(|name| is_fully_qualified_name(name))
        .cloned()
}

fn phase3_term_phase(diagnostic: &npa_frontend::MachineDiagnostic) -> MachineApiDiagnosticPhase {
    if map_phase3_diagnostic_kind(diagnostic) == MachineApiErrorKind::MachineTermParseError {
        MachineApiDiagnosticPhase::MachineTermParse
    } else {
        MachineApiDiagnosticPhase::MachineTermCheck
    }
}

fn mismatch_hashes_for_api(
    kind: MachineApiErrorKind,
    diagnostic: &MachineTacticDiagnostic,
) -> (Option<Hash>, Option<Hash>) {
    if kind == MachineApiErrorKind::TypeMismatch && has_expected_actual(diagnostic) {
        (
            diagnostic.expected_hash.as_deref().copied(),
            diagnostic.actual_hash.as_deref().copied(),
        )
    } else {
        (None, None)
    }
}

fn phase3_mismatch_hashes_for_api(
    kind: MachineApiErrorKind,
    diagnostic: &npa_frontend::MachineDiagnostic,
) -> (Option<Hash>, Option<Hash>) {
    let payload = diagnostic.payload.as_deref();
    if kind == MachineApiErrorKind::TypeMismatch {
        (
            payload.and_then(|payload| payload.expected_hash),
            payload.and_then(|payload| payload.actual_hash),
        )
    } else {
        (None, None)
    }
}

fn has_expected_actual(diagnostic: &MachineTacticDiagnostic) -> bool {
    diagnostic.expected_hash.is_some() && diagnostic.actual_hash.is_some()
}

fn phase3_has_expected_actual(diagnostic: &npa_frontend::MachineDiagnostic) -> bool {
    diagnostic
        .payload
        .as_ref()
        .is_some_and(|payload| payload.expected_hash.is_some() && payload.actual_hash.is_some())
}

fn is_fully_qualified_name(name: &Name) -> bool {
    name.is_canonical()
}

#[cfg(test)]
mod tests {
    use super::*;
    use npa_kernel::{Expr, Level};
    use npa_tactic::RawMachineTerm;

    fn prop() -> Expr {
        Expr::sort(Level::zero())
    }

    fn type0() -> Expr {
        Expr::sort(Level::succ(Level::zero()))
    }

    fn trivial_spec(theorem_type: Expr) -> MachineProofSpec {
        MachineProofSpec {
            module: Name::from_dotted("Test"),
            theorem_name: Name::from_dotted("Test.thm"),
            source_index: 0,
            universe_params: Vec::new(),
            theorem_type,
        }
    }

    fn start_state(theorem_type: Expr) -> MachineProofState {
        phase4_start_machine_proof(
            trivial_spec(theorem_type),
            Vec::new(),
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .unwrap()
        .state
    }

    #[test]
    fn validates_candidate_and_returns_phase4_candidate_hash() {
        let candidate = MachineTacticCandidate::Exact {
            term: RawMachineTerm::new("Prop"),
        };

        let validated = phase4_validate_machine_tactic_candidate(GoalId(0), candidate).unwrap();

        assert_eq!(validated.goal_id, GoalId(0));
        assert_eq!(validated.tactic_kind, MachineApiTacticKind::Exact);
        assert_eq!(
            validated.candidate_hash,
            machine_tactic_hash(&validated.tactic)
        );
    }

    #[test]
    fn raw_machine_term_prepass_failure_has_no_candidate_hash() {
        let candidate = MachineTacticCandidate::Exact {
            term: RawMachineTerm::new("("),
        };

        let err = phase4_validate_machine_tactic_candidate(GoalId(7), candidate).unwrap_err();

        assert_eq!(
            err.diagnostic.kind,
            MachineApiErrorKind::MachineTermParseError
        );
        assert_eq!(
            err.diagnostic.phase,
            MachineApiDiagnosticPhase::MachineTermParse
        );
        assert_eq!(err.diagnostic.goal_id, Some(GoalId(7)));
        assert_eq!(
            err.diagnostic.tactic_kind,
            Some(MachineApiTacticKind::Exact)
        );
        assert_eq!(err.candidate_hash, None);
        assert_eq!(err.deterministic_budget_hash, None);
    }

    #[test]
    fn start_proof_theorem_type_check_error_uses_machine_term_check_phase() {
        let spec = MachineProofSpec {
            theorem_type: Expr::lam("x", type0(), Expr::bvar(0)),
            ..trivial_spec(type0())
        };

        let err = phase4_start_machine_proof(
            spec,
            Vec::new(),
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .unwrap_err();

        assert_eq!(
            err.diagnostic.kind,
            MachineApiErrorKind::MachineTermElaborationError
        );
        assert_eq!(
            err.diagnostic.phase,
            MachineApiDiagnosticPhase::MachineTermCheck
        );
        assert_eq!(err.diagnostic.goal_id, None);
        assert_eq!(err.diagnostic.tactic_kind, None);
        assert_eq!(err.candidate_hash, None);
        assert_eq!(err.deterministic_budget_hash, None);
    }

    #[test]
    fn start_proof_theorem_type_expected_pi_uses_machine_term_check_phase() {
        let spec = MachineProofSpec {
            theorem_type: Expr::app(prop(), prop()),
            ..trivial_spec(type0())
        };

        let err = phase4_start_machine_proof(
            spec,
            Vec::new(),
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .unwrap_err();

        assert_eq!(err.diagnostic.kind, MachineApiErrorKind::ExpectedPiType);
        assert_eq!(
            err.diagnostic.phase,
            MachineApiDiagnosticPhase::MachineTermCheck
        );
        assert_eq!(err.diagnostic.goal_id, None);
        assert_eq!(err.diagnostic.tactic_kind, None);
        assert_eq!(err.diagnostic.expected_hash, None);
        assert_eq!(err.diagnostic.actual_hash, None);
    }

    #[test]
    fn start_proof_theorem_type_type_mismatch_keeps_hashes() {
        let spec = MachineProofSpec {
            theorem_type: Expr::let_in("x", prop(), type0(), Expr::bvar(0)),
            ..trivial_spec(type0())
        };

        let err = phase4_start_machine_proof(
            spec,
            Vec::new(),
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .unwrap_err();

        assert_eq!(err.diagnostic.kind, MachineApiErrorKind::TypeMismatch);
        assert_eq!(
            err.diagnostic.phase,
            MachineApiDiagnosticPhase::MachineTermCheck
        );
        assert_eq!(err.diagnostic.goal_id, None);
        assert_eq!(err.diagnostic.tactic_kind, None);
        assert!(err.diagnostic.expected_hash.is_some());
        assert!(err.diagnostic.actual_hash.is_some());
    }

    #[test]
    fn run_error_maps_phase4_type_mismatch_with_correlation_hashes() {
        let state = start_state(prop());
        let validated = phase4_validate_machine_tactic_candidate(
            GoalId(0),
            MachineTacticCandidate::Exact {
                term: RawMachineTerm::new("Type"),
            },
        )
        .unwrap();
        let budget = TacticBudget::default();

        let err =
            phase4_run_machine_tactic_with_budget(&state, validated.tactic, budget).unwrap_err();

        assert_eq!(err.diagnostic.kind, MachineApiErrorKind::TypeMismatch);
        assert_eq!(
            err.diagnostic.phase,
            MachineApiDiagnosticPhase::MachineTermCheck
        );
        assert_eq!(err.diagnostic.goal_id, Some(GoalId(0)));
        assert_eq!(
            err.diagnostic.tactic_kind,
            Some(MachineApiTacticKind::Exact)
        );
        assert_eq!(err.candidate_hash, Some(validated.candidate_hash));
        assert_eq!(
            err.deterministic_budget_hash,
            Some(tactic_budget_hash(budget))
        );
        assert!(err.cache_key_hash.is_some());
        assert!(err.diagnostic.expected_hash.is_some());
        assert!(err.diagnostic.actual_hash.is_some());
    }

    #[test]
    fn run_goal_not_open_maps_to_snapshot_lookup_without_tactic_correlation() {
        let state = start_state(type0());
        let validated = phase4_validate_machine_tactic_candidate(
            GoalId(99),
            MachineTacticCandidate::Exact {
                term: RawMachineTerm::new("Prop"),
            },
        )
        .unwrap();

        let err = phase4_run_machine_tactic(&state, validated.tactic).unwrap_err();

        assert_eq!(err.diagnostic.kind, MachineApiErrorKind::GoalNotOpen);
        assert_eq!(
            err.diagnostic.phase,
            MachineApiDiagnosticPhase::SnapshotLookup
        );
        assert_eq!(err.diagnostic.goal_id, Some(GoalId(99)));
        assert_eq!(err.diagnostic.tactic_kind, None);
        assert_eq!(err.candidate_hash, None);
        assert_eq!(err.deterministic_budget_hash, None);
        assert_eq!(err.cache_key_hash, None);
    }

    #[test]
    fn run_stale_snapshot_maps_to_snapshot_lookup_without_correlation_hashes() {
        let mut state = start_state(type0());
        let validated = phase4_validate_machine_tactic_candidate(
            GoalId(0),
            MachineTacticCandidate::Exact {
                term: RawMachineTerm::new("Prop"),
            },
        )
        .unwrap();
        state.state_id = "stale".to_owned();

        let err = phase4_run_machine_tactic(&state, validated.tactic.clone()).unwrap_err();

        assert_eq!(
            err.diagnostic.kind,
            MachineApiErrorKind::InvalidMachineProofState
        );
        assert_eq!(
            err.diagnostic.phase,
            MachineApiDiagnosticPhase::SnapshotLookup
        );
        assert_eq!(err.diagnostic.goal_id, None);
        assert_eq!(err.diagnostic.tactic_kind, None);
        assert_eq!(err.candidate_hash, None);
        assert_eq!(err.deterministic_budget_hash, None);
        assert_eq!(err.cache_key_hash, None);

        let result_err =
            phase4_machine_tactic_result_error(&state, validated.tactic, TacticBudget::default())
                .unwrap();
        assert_eq!(
            result_err.diagnostic.kind,
            MachineApiErrorKind::InvalidMachineProofState
        );
        assert_eq!(
            result_err.diagnostic.phase,
            MachineApiDiagnosticPhase::SnapshotLookup
        );
        assert_eq!(result_err.diagnostic.goal_id, None);
        assert_eq!(result_err.diagnostic.tactic_kind, None);
        assert_eq!(result_err.candidate_hash, None);
        assert_eq!(result_err.deterministic_budget_hash, None);
        assert_eq!(result_err.cache_key_hash, None);
    }

    #[test]
    fn run_intro_name_collision_is_post_canonical_candidate_error() {
        let state = start_state(Expr::pi("p", prop(), Expr::pi("q", prop(), prop())));
        let intro_p = phase4_validate_machine_tactic_candidate(
            GoalId(0),
            MachineTacticCandidate::Intro {
                name: "p".to_owned(),
            },
        )
        .unwrap();
        let run = phase4_run_machine_tactic(&state, intro_p.tactic).unwrap();
        let duplicate = phase4_validate_machine_tactic_candidate(
            GoalId(1),
            MachineTacticCandidate::Intro {
                name: "p".to_owned(),
            },
        )
        .unwrap();

        let err = phase4_run_machine_tactic(&run.state, duplicate.tactic).unwrap_err();

        assert_eq!(err.diagnostic.kind, MachineApiErrorKind::InvalidCandidate);
        assert_eq!(
            err.diagnostic.phase,
            MachineApiDiagnosticPhase::CandidateValidation
        );
        assert_eq!(err.diagnostic.goal_id, Some(GoalId(1)));
        assert_eq!(
            err.diagnostic.tactic_kind,
            Some(MachineApiTacticKind::Intro)
        );
        assert_eq!(err.candidate_hash, Some(duplicate.candidate_hash));
        assert_eq!(
            err.deterministic_budget_hash,
            Some(tactic_budget_hash(TacticBudget::default()))
        );
        assert!(err.cache_key_hash.is_some());
    }

    #[test]
    fn extract_closed_theorem_maps_open_goal_to_caller_phase() {
        let state = start_state(type0());

        let err = phase4_extract_closed_machine_theorem_decl(
            &state,
            MachineApiDiagnosticPhase::SnapshotLookup,
        )
        .unwrap_err();

        assert_eq!(
            err.diagnostic.kind,
            MachineApiErrorKind::InvalidMachineProofState
        );
        assert_eq!(
            err.diagnostic.phase,
            MachineApiDiagnosticPhase::SnapshotLookup
        );
    }

    #[test]
    fn extract_closed_theorem_succeeds_after_exact() {
        let state = start_state(type0());
        let validated = phase4_validate_machine_tactic_candidate(
            GoalId(0),
            MachineTacticCandidate::Exact {
                term: RawMachineTerm::new("Prop"),
            },
        )
        .unwrap();
        let run = phase4_run_machine_tactic(&state, validated.tactic).unwrap();

        assert_eq!(run.next_state_fingerprint, run.state.fingerprint);
        assert_eq!(run.proof_delta_hash, run.delta.delta_hash);

        let closed = run.state;

        let extracted = phase4_extract_closed_machine_theorem_decl(
            &closed,
            MachineApiDiagnosticPhase::KernelCheck,
        )
        .unwrap();

        assert_eq!(
            extracted.theorem,
            Decl::Theorem {
                name: "Test.thm".to_owned(),
                universe_params: Vec::new(),
                ty: type0(),
                proof: prop(),
            }
        );
    }

    #[test]
    fn phase4_mapping_is_exhaustive_for_current_tactic_diagnostics() {
        let diagnostic = MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::ExprNode,
            },
            "expr nodes exhausted",
        );

        assert_eq!(
            map_phase4_diagnostic_kind(&diagnostic),
            MachineApiErrorKind::TooLargeTerm
        );
    }

    #[test]
    fn phase3_universe_level_too_large_is_mapped() {
        let diagnostic = npa_frontend::MachineDiagnostic::error(
            npa_frontend::MachineDiagnosticKind::UniverseLevelTooLarge,
            npa_frontend::Span::new(npa_frontend::FileId(0), 0, 1),
            "universe too large",
        );

        assert_eq!(
            map_phase3_diagnostic_kind(&diagnostic),
            MachineApiErrorKind::MachineTermElaborationError
        );
    }

    #[test]
    fn single_component_primary_name_is_preserved() {
        let mut diagnostic = MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::MachineTermElaborationError,
            "elaboration failed",
        );
        diagnostic.primary_name = Some(Name::from_dotted("Eq"));

        let projected =
            project_phase4_diagnostic(diagnostic, MachineApiDiagnosticPhase::MachineTermCheck);

        assert_eq!(projected.primary_name, Some(Name::from_dotted("Eq")));
    }

    #[test]
    fn noncanonical_primary_name_is_omitted() {
        let mut diagnostic = MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::MachineTermElaborationError,
            "elaboration failed",
        );
        diagnostic.primary_name = Some(Name::from_dotted("Bad..Name"));

        let projected =
            project_phase4_diagnostic(diagnostic, MachineApiDiagnosticPhase::MachineTermCheck);

        assert_eq!(projected.primary_name, None);
    }

    #[test]
    fn proof_expr_type_mismatch_without_hashes_stays_proof_state_error() {
        let diagnostic = MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::ProofExprTypeMismatch,
            "proof expression mismatch",
        );

        assert_eq!(
            map_phase4_diagnostic_kind(&diagnostic),
            MachineApiErrorKind::InvalidMachineProofState
        );
    }
}
