use std::collections::{BTreeMap, BTreeSet};

use npa_cert::{AxiomRef, ExportEntry, ExportKind, GlobalRef, Hash, Name, TermId, TermNode};
use npa_frontend::{MachineLocalDecl, MachineSurfaceCallableRef};
use npa_kernel::Expr;
use npa_tactic::{
    CandidateApplyArg, CandidateRewriteRuleRef, MachineTacticCandidate, ResolvedSimpRule,
    RewriteDirection, RewriteSite, SimpRuleRef, TacticHead,
};
use sha2::{Digest, Sha256};

use crate::adapter::{
    phase4_validate_machine_tactic_candidate, MachineApiDiagnosticPhase,
    MachineApiDiagnosticProjection,
};
use crate::current::{
    encode_machine_axiom_ref_wire, imported_axiom_ref_to_wire, MachineAxiomRefWire,
};
use crate::json::{JsonValue, JsonValueKind};
use crate::projection::VerifiedModuleContextEntry;
use crate::renderer::{
    render_machine_expr_view, MachineDisplayRenderScope, MachineDisplayRenderScopeEntry,
    MachineExprRendererContext, MachineGlobalRefView, Phase5ResolvedDisplayCoreRefOwner,
};
use crate::snapshot::{MachineSnapshotLookupError, MachineSnapshotMaterializationContext};
use crate::types::{
    format_goal_id_wire, parse_goal_id_wire, parse_module_name_wire, HashString,
    MachineApiErrorResponse, MachineApiErrorWire, MachineApiOkResponse, MachineApiResponseEnvelope,
    MachineApiResponseStatus, MachineProofSession, SessionId, SnapshotId,
};
use crate::validation::{
    parse_request_body, validate_json_object, FieldSpec, JsonFieldType, JsonPath, JsonPathElement,
    MachineApiErrorKind, MachineApiRequestError, MachineApiRequestErrorReason, ObjectSchema,
};
use crate::{
    phase5_name_canonical_bytes, validate_machine_endpoint_envelope, MachineApiVersion,
    Phase5UpstreamDiagnostic,
};

const THEOREM_INDEX_SCHEMA_VERSION: &str =
    "mvp-export-entry-v4-entry-bytes-visible-heads-universe-params";
const SEARCH_PROFILE_VERSION: &str = "mvp-zero-score-v1";
const SUGGESTION_PROFILE_VERSION: &str = "mvp-suggested-candidates-v1";

const FILTER_FIELDS: &[FieldSpec] = &[
    FieldSpec::required("exclude_axioms", JsonFieldType::Boolean),
    FieldSpec::optional("allowed_modules", JsonFieldType::Array),
];

pub type MachineTheoremSearchResponse =
    MachineApiResponseEnvelope<MachineTheoremSearchOkFields, MachineApiErrorWire, ()>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTheoremSearchOkFields {
    pub query_fingerprint: Hash,
    pub theorem_index_fingerprint: Hash,
    pub search_profile_version: &'static str,
    pub suggestion_profile_version: &'static str,
    pub results: Vec<MachineTheoremSearchResult>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTheoremSearchResult {
    pub premise_id: String,
    pub global_ref: MachineTheoremGlobalRef,
    pub universe_params: Vec<String>,
    pub statement: MachineTheoremStatement,
    pub modes: Vec<MachineTheoremMode>,
    pub suggested_candidates: Vec<MachineSuggestedCandidate>,
    pub score: u64,
    pub axioms_used: Vec<MachineAxiomRefWire>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTheoremGlobalRef {
    pub module: Name,
    pub name: Name,
    pub export_hash: Hash,
    pub decl_interface_hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTheoremStatement {
    pub core_hash: Hash,
    pub head: Option<MachineGlobalRefView>,
    pub machine: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineSuggestedCandidate {
    pub status: MachineSuggestedCandidateStatus,
    pub candidate_hash: Hash,
    pub candidate: MachineTacticCandidate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineSuggestedCandidateStatus {
    Validated,
}

impl MachineSuggestedCandidateStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Validated => "validated",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MachineTheoremMode {
    Exact,
    Apply,
    Rw,
    Simp,
}

impl MachineTheoremMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Apply => "apply",
            Self::Rw => "rw",
            Self::Simp => "simp",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "exact" => Some(Self::Exact),
            "apply" => Some(Self::Apply),
            "rw" => Some(Self::Rw),
            "simp" => Some(Self::Simp),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTheoremSearchRequest {
    pub session_id: SessionId,
    pub snapshot_id: SnapshotId,
    pub state_fingerprint: Hash,
    pub goal_id: npa_tactic::GoalId,
    pub modes: Vec<MachineTheoremMode>,
    pub limit: u32,
    pub filters: MachineTheoremFilters,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTheoremFilters {
    pub exclude_axioms: bool,
    pub allowed_modules: MachineAllowedModulesFilter,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MachineAllowedModulesFilter {
    AllDirect,
    Explicit(Vec<Name>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTheoremSearchError {
    pub diagnostic: MachineApiDiagnosticProjection,
    pub response: MachineTheoremSearchResponse,
}

#[derive(Clone, Debug)]
struct TheoremIndex {
    entries: Vec<TheoremIndexEntry>,
    fingerprint: Hash,
}

#[derive(Clone, Debug)]
struct TheoremIndexEntry {
    global_ref: MachineTheoremGlobalRef,
    universe_params: Vec<String>,
    statement_type: Expr,
    statement_display_scope: MachineDisplayRenderScope,
    statement_core_hash: Hash,
    head: Option<MachineGlobalRefView>,
    axioms_used: Vec<MachineAxiomRefWire>,
    modes: Vec<MachineTheoremMode>,
    canonical_bytes: Vec<u8>,
}

pub fn search_machine_theorems_for_goal(
    source: &str,
    session: &MachineProofSession,
) -> Result<MachineTheoremSearchResponse, Box<MachineTheoremSearchError>> {
    search_machine_theorems_for_goal_in_sessions(source, std::iter::once(session))
}

pub fn search_machine_theorems_for_goal_in_sessions<'session>(
    source: &str,
    sessions: impl IntoIterator<Item = &'session MachineProofSession>,
) -> Result<MachineTheoremSearchResponse, Box<MachineTheoremSearchError>> {
    let request = parse_machine_theorem_search_request(source).map_err(search_request_error)?;
    let Some(session) = sessions
        .into_iter()
        .find(|session| session.session_id == request.session_id)
    else {
        return Err(search_plain_error(
            MachineApiErrorKind::UnknownSession,
            MachineApiDiagnosticPhase::SessionLookup,
            format!("unknown session {}", request.session_id.wire()),
        ));
    };

    search_machine_theorems_for_goal_parsed(session, request)
}

pub fn parse_machine_theorem_search_request(
    source: &str,
) -> Result<MachineTheoremSearchRequest, MachineApiRequestError> {
    let doc = parse_request_body(source, MachineApiErrorKind::InvalidTheoremQuery)?;
    let envelope = validate_machine_endpoint_envelope(
        doc.root(),
        crate::MachineApiEndpoint::SearchForGoal,
        &JsonPath::root(),
    )?;

    let session_id = SessionId::parse(required_string(&envelope, "session_id"))
        .expect("endpoint validation checked session_id grammar");
    let snapshot_id = SnapshotId::parse(required_string(&envelope, "snapshot_id"))
        .expect("endpoint validation checked snapshot_id grammar");
    let state_fingerprint = HashString::parse(required_string(&envelope, "state_fingerprint"))
        .expect("endpoint validation checked state_fingerprint grammar")
        .digest();
    let goal_id = parse_goal_id_wire(required_string(&envelope, "goal_id"))
        .expect("endpoint validation checked goal_id grammar");
    let modes = parse_modes(
        required_field(&envelope, "modes"),
        &JsonPath::root().field("modes"),
    )?;
    let limit = parse_limit(required_field(&envelope, "limit"));
    let filters = parse_filters(
        required_field(&envelope, "filters"),
        &JsonPath::root().field("filters"),
    )?;

    Ok(MachineTheoremSearchRequest {
        session_id,
        snapshot_id,
        state_fingerprint,
        goal_id,
        modes,
        limit,
        filters,
    })
}

fn search_machine_theorems_for_goal_parsed(
    session: &MachineProofSession,
    mut request: MachineTheoremSearchRequest,
) -> Result<MachineTheoremSearchResponse, Box<MachineTheoremSearchError>> {
    if session.snapshots.session_id() != &session.session_id {
        return Err(search_plain_error(
            MachineApiErrorKind::InvalidMachineProofState,
            MachineApiDiagnosticPhase::SnapshotLookup,
            "session snapshot store belongs to a different session",
        ));
    }

    canonicalize_allowed_modules_for_session(session, &mut request.filters)?;

    let context = MachineSnapshotMaterializationContext {
        session_id: &session.session_id,
        display_scope: &session.machine_display_render_scope,
        callable_interface_table: &session.machine_surface_callable_interface_table,
    };
    let entry = session
        .snapshots
        .lookup_checked(&context, request.snapshot_id, request.state_fingerprint)
        .map_err(search_snapshot_lookup_error)?;
    let goal = entry
        .materialized_view_payload
        .goals
        .iter()
        .find(|goal| goal.goal_id == request.goal_id)
        .ok_or_else(|| {
            search_goal_error(
                MachineApiErrorKind::GoalNotOpen,
                MachineApiDiagnosticPhase::SnapshotLookup,
                request.goal_id,
                format!("goal {} is not open", format_goal_id_wire(request.goal_id)),
            )
        })?;
    let input_state = &entry.executable_state_payload;

    let index = build_theorem_index(session, input_state).map_err(search_theorem_index_error)?;
    let query_fingerprint = theorem_query_fingerprint(QueryFingerprintInput {
        protocol_version: session.protocol_version,
        state_fingerprint: request.state_fingerprint,
        goal_id: request.goal_id,
        goal_fingerprint: goal.goal_fingerprint,
        theorem_index_fingerprint: index.fingerprint,
        modes: &request.modes,
        filters: &request.filters,
        limit: request.limit,
    });

    let mut eligible = index
        .entries
        .iter()
        .filter(|entry| theorem_entry_matches_query(entry, &request))
        .collect::<Vec<_>>();
    eligible.sort_by_key(|entry| theorem_entry_sort_key(entry));
    eligible.truncate(request.limit as usize);

    let mut results = Vec::with_capacity(eligible.len());
    for (index, entry) in eligible.into_iter().enumerate() {
        let statement = render_statement(session, entry).map_err(search_theorem_index_error)?;
        let suggested_candidates =
            suggested_candidates_for_entry(entry, &request.modes, input_state, request.goal_id)
                .map_err(search_theorem_index_error)?;
        results.push(MachineTheoremSearchResult {
            premise_id: format!("prem_{index}"),
            global_ref: entry.global_ref.clone(),
            universe_params: entry.universe_params.clone(),
            statement,
            modes: entry.modes.clone(),
            suggested_candidates,
            score: 0,
            axioms_used: entry.axioms_used.clone(),
        });
    }

    Ok(MachineApiResponseEnvelope::Ok(MachineApiOkResponse {
        status: MachineApiResponseStatus::Ok,
        endpoint_fields: MachineTheoremSearchOkFields {
            query_fingerprint,
            theorem_index_fingerprint: index.fingerprint,
            search_profile_version: SEARCH_PROFILE_VERSION,
            suggestion_profile_version: SUGGESTION_PROFILE_VERSION,
            results,
        },
    }))
}

fn build_theorem_index(
    session: &MachineProofSession,
    state: &npa_tactic::MachineProofState,
) -> Result<TheoremIndex, TheoremSearchBuildError> {
    let mut entries = Vec::new();
    let mut public_names = BTreeSet::new();
    let eq_head = resolved_eq_head(session, state)?;
    for import in session.import_certificate_context.direct_import_entries() {
        let kernel_decls = npa_cert::verified_module_to_kernel_decls(&import.verified_module)
            .map_err(|_| TheoremSearchBuildError::InvalidVerifiedImport)?;
        let kernel_decls_by_name = kernel_decls
            .iter()
            .map(|decl| (decl.name().to_owned(), decl))
            .collect::<BTreeMap<_, _>>();
        for export in &import.export_block {
            if !matches!(export.kind, ExportKind::Axiom | ExportKind::Theorem) {
                continue;
            }
            let name = export_name(import, export)?;
            if !public_names.insert(name.clone()) {
                return Err(TheoremSearchBuildError::DuplicatePublicName);
            }
            let universe_params = export_universe_params(import, export)?;
            let statement_type = kernel_decls_by_name
                .get(&name.as_dotted())
                .ok_or(TheoremSearchBuildError::MissingKernelDecl)?
                .ty()
                .clone();
            let head = theorem_statement_head(session, import, export.ty)?;
            let statement_display_scope =
                theorem_statement_display_scope(session, import, export.ty)?;
            let mut modes = vec![MachineTheoremMode::Exact];
            if has_leading_pi(&statement_type) {
                modes.push(MachineTheoremMode::Apply);
            }
            if eq_head
                .as_ref()
                .zip(head.as_ref())
                .is_some_and(|(eq_head, head)| eq_head.canonical_bytes() == head.canonical_bytes())
            {
                modes.push(MachineTheoremMode::Rw);
            }
            if has_matching_imported_simp_rule(state, &name, &export.decl_interface_hash) {
                modes.push(MachineTheoremMode::Simp);
            }

            let mut axioms_used = export
                .axiom_dependencies
                .iter()
                .map(|axiom| {
                    imported_axiom_ref_to_wire(
                        0,
                        &session.import_certificate_context,
                        import,
                        axiom,
                    )
                    .map_err(|_| TheoremSearchBuildError::InvalidAxiomRef)
                })
                .collect::<Result<Vec<_>, _>>()?;
            sort_dedup_axiom_refs(&mut axioms_used);
            let axiom_dependencies_hash = axiom_dependencies_hash(&export.axiom_dependencies);
            let global_ref = MachineTheoremGlobalRef {
                module: import.key.module.clone(),
                name,
                export_hash: import.key.export_hash,
                decl_interface_hash: export.decl_interface_hash,
            };
            let canonical_bytes = theorem_index_entry_canonical_bytes(
                &global_ref,
                &universe_params,
                export.type_hash,
                head.as_ref(),
                axiom_dependencies_hash,
                &modes,
            );
            entries.push(TheoremIndexEntry {
                global_ref,
                universe_params,
                statement_type,
                statement_display_scope,
                statement_core_hash: export.type_hash,
                head,
                axioms_used,
                modes,
                canonical_bytes,
            });
        }
    }
    entries.sort_by_key(theorem_entry_sort_key);
    let fingerprint = theorem_index_fingerprint(session, &entries);
    Ok(TheoremIndex {
        entries,
        fingerprint,
    })
}

fn suggested_candidates_for_entry(
    entry: &TheoremIndexEntry,
    request_modes: &[MachineTheoremMode],
    state: &npa_tactic::MachineProofState,
    goal_id: npa_tactic::GoalId,
) -> Result<Vec<MachineSuggestedCandidate>, TheoremSearchBuildError> {
    if !entry.universe_params.is_empty() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for mode in canonical_mode_intersection(request_modes, &entry.modes) {
        match mode {
            MachineTheoremMode::Exact | MachineTheoremMode::Apply => {}
            MachineTheoremMode::Rw => {
                if let Some(rule) = first_matching_simp_rule(
                    state,
                    &entry.global_ref.name,
                    &entry.global_ref.decl_interface_hash,
                    Some(RewriteDirection::Forward),
                ) {
                    let candidate = MachineTacticCandidate::Rewrite {
                        rule: CandidateRewriteRuleRef {
                            head: premise_tactic_head(&entry.global_ref),
                            universe_args: Vec::new(),
                            args: rule
                                .rule_telescope
                                .iter()
                                .map(|_| CandidateApplyArg::InferFromTarget)
                                .collect(),
                        },
                        direction: RewriteDirection::Forward,
                        site: RewriteSite::EqTargetLeft,
                    };
                    if let Some(candidate) = validate_suggested_candidate(
                        state,
                        goal_id,
                        candidate,
                        Some(rule),
                        &entry.global_ref,
                    )? {
                        out.push(candidate);
                    }
                }
            }
            MachineTheoremMode::Simp => {
                if let Some(rule) = first_matching_simp_rule(
                    state,
                    &entry.global_ref.name,
                    &entry.global_ref.decl_interface_hash,
                    None,
                ) {
                    let candidate = MachineTacticCandidate::SimpLite {
                        rules: vec![rule.key.clone()],
                    };
                    if let Some(candidate) = validate_suggested_candidate(
                        state,
                        goal_id,
                        candidate,
                        Some(rule),
                        &entry.global_ref,
                    )? {
                        out.push(candidate);
                    }
                }
            }
        }
    }
    Ok(out)
}

fn validate_suggested_candidate(
    state: &npa_tactic::MachineProofState,
    goal_id: npa_tactic::GoalId,
    candidate: MachineTacticCandidate,
    rule: Option<&ResolvedSimpRule>,
    global_ref: &MachineTheoremGlobalRef,
) -> Result<Option<MachineSuggestedCandidate>, TheoremSearchBuildError> {
    if !candidate_head_and_rule_resolve(state, &candidate, rule, global_ref) {
        return Ok(None);
    }
    let validated = phase4_validate_machine_tactic_candidate(goal_id, candidate.clone())
        .map_err(|_| TheoremSearchBuildError::SuggestedCandidateInvalid)?;
    Ok(Some(MachineSuggestedCandidate {
        status: MachineSuggestedCandidateStatus::Validated,
        candidate_hash: validated.candidate_hash,
        candidate,
    }))
}

fn candidate_head_and_rule_resolve(
    state: &npa_tactic::MachineProofState,
    candidate: &MachineTacticCandidate,
    rule: Option<&ResolvedSimpRule>,
    global_ref: &MachineTheoremGlobalRef,
) -> bool {
    match candidate {
        MachineTacticCandidate::Rewrite {
            rule: rewrite_rule,
            direction: RewriteDirection::Forward,
            site: RewriteSite::EqTargetLeft,
        } => {
            let Some(rule) = rule else {
                return false;
            };
            rewrite_rule.head == premise_tactic_head(global_ref)
                && rewrite_rule.universe_args.is_empty()
                && rewrite_rule.args.len() == rule.rule_telescope.len()
                && rewrite_rule
                    .args
                    .iter()
                    .all(|arg| matches!(arg, CandidateApplyArg::InferFromTarget))
                && matches_imported_simp_rule(
                    rule,
                    &global_ref.name,
                    &global_ref.decl_interface_hash,
                    Some(RewriteDirection::Forward),
                )
                && imported_head_resolves(state, &global_ref.name, &global_ref.decl_interface_hash)
        }
        MachineTacticCandidate::SimpLite { rules } => {
            let [candidate_rule] = rules.as_slice() else {
                return false;
            };
            state
                .env
                .simp_registry
                .rules
                .iter()
                .any(|resolved| resolved.key == *candidate_rule)
        }
        _ => false,
    }
}

fn render_statement(
    session: &MachineProofSession,
    entry: &TheoremIndexEntry,
) -> Result<MachineTheoremStatement, TheoremSearchBuildError> {
    let context = MachineExprRendererContext {
        display_scope: &entry.statement_display_scope,
        callable_interface_table: &session.machine_surface_callable_interface_table,
        base_context: &[] as &[MachineLocalDecl],
        universe_params: &entry.universe_params,
    };
    let view = render_machine_expr_view(&entry.statement_type, &context)
        .map_err(|_| TheoremSearchBuildError::RenderFailed)?;
    Ok(MachineTheoremStatement {
        core_hash: entry.statement_core_hash,
        head: entry.head.clone(),
        machine: view.machine,
    })
}

fn theorem_entry_matches_query(
    entry: &TheoremIndexEntry,
    request: &MachineTheoremSearchRequest,
) -> bool {
    if request.filters.exclude_axioms && !entry.axioms_used.is_empty() {
        return false;
    }
    match &request.filters.allowed_modules {
        MachineAllowedModulesFilter::AllDirect => {}
        MachineAllowedModulesFilter::Explicit(modules) => {
            if !modules.contains(&entry.global_ref.module) {
                return false;
            }
        }
    }
    entry.modes.iter().any(|mode| request.modes.contains(mode))
}

fn canonical_mode_intersection(
    lhs: &[MachineTheoremMode],
    rhs: &[MachineTheoremMode],
) -> Vec<MachineTheoremMode> {
    canonical_modes()
        .into_iter()
        .filter(|mode| lhs.contains(mode) && rhs.contains(mode))
        .collect()
}

fn first_matching_simp_rule<'a>(
    state: &'a npa_tactic::MachineProofState,
    name: &Name,
    decl_interface_hash: &Hash,
    direction: Option<RewriteDirection>,
) -> Option<&'a ResolvedSimpRule> {
    state
        .env
        .simp_registry
        .rules
        .iter()
        .filter(|rule| matches_imported_simp_rule(rule, name, decl_interface_hash, direction))
        .min_by_key(|rule| simp_rule_ref_canonical_bytes(&rule.key))
}

fn matches_imported_simp_rule(
    rule: &ResolvedSimpRule,
    name: &Name,
    decl_interface_hash: &Hash,
    direction: Option<RewriteDirection>,
) -> bool {
    matches!(
        &rule.source,
        TacticHead::Imported {
            name: source_name,
            decl_interface_hash: source_hash,
        } if source_name == name && source_hash == decl_interface_hash
    ) && direction.is_none_or(|direction| rule.key.direction == direction)
}

fn has_matching_imported_simp_rule(
    state: &npa_tactic::MachineProofState,
    name: &Name,
    decl_interface_hash: &Hash,
) -> bool {
    first_matching_simp_rule(state, name, decl_interface_hash, None).is_some()
}

fn imported_head_resolves(
    state: &npa_tactic::MachineProofState,
    name: &Name,
    decl_interface_hash: &Hash,
) -> bool {
    state
        .env
        .imports
        .iter()
        .flat_map(|import| import.exports())
        .filter(|export| export.name == *name && export.decl_interface_hash == *decl_interface_hash)
        .count()
        == 1
}

fn premise_tactic_head(global_ref: &MachineTheoremGlobalRef) -> TacticHead {
    TacticHead::Imported {
        name: global_ref.name.clone(),
        decl_interface_hash: global_ref.decl_interface_hash,
    }
}

fn resolved_eq_head(
    session: &MachineProofSession,
    state: &npa_tactic::MachineProofState,
) -> Result<Option<MachineGlobalRefView>, TheoremSearchBuildError> {
    let Some(eq_family) = state.env.options.eq_family.as_ref() else {
        return Ok(None);
    };
    let mut matches = session
        .import_certificate_context
        .direct_import_entries()
        .into_iter()
        .flat_map(|import| {
            import
                .export_block
                .iter()
                .filter(move |export| {
                    export.kind != ExportKind::Constructor
                        && export.kind != ExportKind::Recursor
                        && export.decl_interface_hash == eq_family.eq_interface_hash
                })
                .filter_map(move |export| {
                    export_name(import, export)
                        .ok()
                        .filter(|name| *name == eq_family.eq_name)
                        .map(|name| MachineGlobalRefView::Imported {
                            module: import.key.module.clone(),
                            name,
                            export_hash: import.key.export_hash,
                            decl_interface_hash: export.decl_interface_hash,
                            public_export: true,
                            tactic_head_visible: true,
                        })
                })
        })
        .collect::<Vec<_>>();
    Ok(match matches.len() {
        1 => matches.pop(),
        _ => None,
    })
}

fn theorem_statement_head(
    session: &MachineProofSession,
    owner: &VerifiedModuleContextEntry,
    ty: TermId,
) -> Result<Option<MachineGlobalRefView>, TheoremSearchBuildError> {
    let mut conclusion = ty;
    while let TermNode::Pi { body, .. } = term_node(owner, conclusion)? {
        conclusion = *body;
    }
    syntactic_term_head(owner, conclusion)?
        .map(|global_ref| normalized_global_ref_view(session, owner, &global_ref))
        .transpose()
}

fn syntactic_term_head(
    owner: &VerifiedModuleContextEntry,
    term: TermId,
) -> Result<Option<GlobalRef>, TheoremSearchBuildError> {
    let mut current = term;
    while let TermNode::App(func, _) = term_node(owner, current)? {
        current = *func;
    }
    Ok(match term_node(owner, current)? {
        TermNode::Const { global_ref, .. } => Some(global_ref.clone()),
        _ => None,
    })
}

fn theorem_statement_display_scope(
    session: &MachineProofSession,
    owner: &VerifiedModuleContextEntry,
    term: TermId,
) -> Result<MachineDisplayRenderScope, TheoremSearchBuildError> {
    let mut entries = session.machine_display_render_scope.entries().to_vec();
    let mut views_by_name = entries
        .iter()
        .map(|entry| (entry.name.as_dotted(), entry.view.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut visited = BTreeSet::new();
    collect_term_display_scope_entries(
        session,
        owner,
        term,
        &mut visited,
        &mut views_by_name,
        &mut entries,
    )?;
    MachineDisplayRenderScope::from_entries(entries)
        .map_err(|_| TheoremSearchBuildError::DisplayRefMismatch)
}

fn collect_term_display_scope_entries(
    session: &MachineProofSession,
    owner: &VerifiedModuleContextEntry,
    term: TermId,
    visited: &mut BTreeSet<TermId>,
    views_by_name: &mut BTreeMap<String, MachineGlobalRefView>,
    entries: &mut Vec<MachineDisplayRenderScopeEntry>,
) -> Result<(), TheoremSearchBuildError> {
    if !visited.insert(term) {
        return Ok(());
    }
    match term_node(owner, term)?.clone() {
        TermNode::Sort(_) | TermNode::BVar(_) => Ok(()),
        TermNode::Const { global_ref, .. } => {
            let view = normalized_global_ref_view(session, owner, &global_ref)?;
            push_statement_display_entry(owner, view, views_by_name, entries)
        }
        TermNode::App(func, arg) => {
            collect_term_display_scope_entries(
                session,
                owner,
                func,
                visited,
                views_by_name,
                entries,
            )?;
            collect_term_display_scope_entries(session, owner, arg, visited, views_by_name, entries)
        }
        TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => {
            collect_term_display_scope_entries(
                session,
                owner,
                ty,
                visited,
                views_by_name,
                entries,
            )?;
            collect_term_display_scope_entries(
                session,
                owner,
                body,
                visited,
                views_by_name,
                entries,
            )
        }
        TermNode::Let { ty, value, body } => {
            collect_term_display_scope_entries(
                session,
                owner,
                ty,
                visited,
                views_by_name,
                entries,
            )?;
            collect_term_display_scope_entries(
                session,
                owner,
                value,
                visited,
                views_by_name,
                entries,
            )?;
            collect_term_display_scope_entries(
                session,
                owner,
                body,
                visited,
                views_by_name,
                entries,
            )
        }
    }
}

fn push_statement_display_entry(
    owner: &VerifiedModuleContextEntry,
    view: MachineGlobalRefView,
    views_by_name: &mut BTreeMap<String, MachineGlobalRefView>,
    entries: &mut Vec<MachineDisplayRenderScopeEntry>,
) -> Result<(), TheoremSearchBuildError> {
    let dotted = view.name().as_dotted();
    if let Some(existing) = views_by_name.get(&dotted) {
        if existing == &view {
            return Ok(());
        }
        return Err(TheoremSearchBuildError::DisplayRefMismatch);
    }

    let callable_ref = display_callable_ref_for_view(&view)?;
    let entry = MachineDisplayRenderScopeEntry::new(
        view.clone(),
        Phase5ResolvedDisplayCoreRefOwner::VerifiedImportedModule {
            owner_module: owner.key.module.clone(),
            owner_export_hash: owner.key.export_hash,
        },
        callable_ref,
    );
    views_by_name.insert(dotted, view);
    entries.push(entry);
    Ok(())
}

fn display_callable_ref_for_view(
    view: &MachineGlobalRefView,
) -> Result<MachineSurfaceCallableRef, TheoremSearchBuildError> {
    match view {
        MachineGlobalRefView::Imported {
            module,
            name,
            export_hash,
            decl_interface_hash,
            ..
        } => Ok(MachineSurfaceCallableRef::Imported {
            module: module.clone(),
            name: name.clone(),
            export_hash: *export_hash,
            decl_interface_hash: *decl_interface_hash,
        }),
        MachineGlobalRefView::LocalGenerated {
            module,
            export_hash: Some(export_hash),
            name,
            decl_interface_hash,
            ..
        } => Ok(MachineSurfaceCallableRef::Imported {
            module: module.clone(),
            name: name.clone(),
            export_hash: *export_hash,
            decl_interface_hash: *decl_interface_hash,
        }),
        MachineGlobalRefView::CurrentModule {
            module,
            name,
            source_index,
            decl_interface_hash,
        } => Ok(MachineSurfaceCallableRef::CurrentModule {
            module: module.clone(),
            name: name.clone(),
            source_index: *source_index,
            decl_interface_hash: *decl_interface_hash,
        }),
        MachineGlobalRefView::LocalGenerated {
            export_hash: None, ..
        } => Err(TheoremSearchBuildError::DisplayRefMissing),
    }
}

fn normalized_global_ref_view(
    session: &MachineProofSession,
    owner: &VerifiedModuleContextEntry,
    global_ref: &GlobalRef,
) -> Result<MachineGlobalRefView, TheoremSearchBuildError> {
    match global_ref {
        GlobalRef::Local { decl_index } => {
            let decl = owner
                .decl_index_table
                .get(*decl_index)
                .ok_or(TheoremSearchBuildError::MissingDeclIndex)?;
            let public_export =
                ordinary_public_export(owner, &decl.name, &decl.hashes.decl_interface_hash)?
                    .is_some();
            let tactic_head_visible = public_export
                && direct_public_tactic_head_visible(
                    session,
                    &owner.key.module,
                    &decl.name,
                    &owner.key.export_hash,
                    &decl.hashes.decl_interface_hash,
                );
            Ok(MachineGlobalRefView::Imported {
                module: owner.key.module.clone(),
                name: decl.name.clone(),
                export_hash: owner.key.export_hash,
                decl_interface_hash: decl.hashes.decl_interface_hash,
                public_export,
                tactic_head_visible,
            })
        }
        GlobalRef::LocalGenerated { decl_index, name } => {
            let generated_name = name_from_owner(owner, *name)?;
            let generated = owner
                .generated_decl_table
                .iter()
                .find(|entry| {
                    entry.parent_decl_index == *decl_index && entry.name == generated_name
                })
                .ok_or(TheoremSearchBuildError::MissingGeneratedDecl)?;
            let parent = owner
                .decl_index_table
                .get(generated.parent_decl_index)
                .ok_or(TheoremSearchBuildError::MissingDeclIndex)?;
            let tactic_head_visible = direct_public_tactic_head_visible(
                session,
                &owner.key.module,
                &generated.name,
                &owner.key.export_hash,
                &generated.export.decl_interface_hash,
            );
            Ok(MachineGlobalRefView::LocalGenerated {
                module: owner.key.module.clone(),
                export_hash: Some(owner.key.export_hash),
                parent_name: parent.name.clone(),
                name: generated.name.clone(),
                parent_decl_interface_hash: parent.hashes.decl_interface_hash,
                decl_interface_hash: generated.export.decl_interface_hash,
                public_export: true,
                tactic_head_visible,
            })
        }
        GlobalRef::Imported {
            import_index,
            name,
            decl_interface_hash,
        } => {
            let key = owner
                .certificate_import_table
                .get(*import_index)
                .ok_or(TheoremSearchBuildError::MissingImportTableEntry)?;
            let imported = session
                .import_certificate_context
                .verified_modules()
                .iter()
                .find(|entry| &entry.key == key)
                .ok_or(TheoremSearchBuildError::MissingImportedModule)?;
            let imported_name = name_from_owner(owner, *name)?;
            imported_public_export_view(session, imported, &imported_name, decl_interface_hash)
        }
        GlobalRef::Builtin { .. } => Err(TheoremSearchBuildError::BuiltinGlobalRefUnsupported),
    }
}

fn imported_public_export_view(
    session: &MachineProofSession,
    import: &VerifiedModuleContextEntry,
    name: &Name,
    decl_interface_hash: &Hash,
) -> Result<MachineGlobalRefView, TheoremSearchBuildError> {
    let export = unique_public_export(import, name, decl_interface_hash)?
        .ok_or(TheoremSearchBuildError::MissingPublicExport)?;
    match export.kind {
        ExportKind::Constructor | ExportKind::Recursor => {
            let generated = import
                .generated_decl_table
                .iter()
                .find(|entry| {
                    entry.name == *name
                        && entry.export.kind == export.kind
                        && entry.export.decl_interface_hash == *decl_interface_hash
                })
                .ok_or(TheoremSearchBuildError::MissingGeneratedDecl)?;
            let parent = import
                .decl_index_table
                .get(generated.parent_decl_index)
                .ok_or(TheoremSearchBuildError::MissingDeclIndex)?;
            Ok(MachineGlobalRefView::LocalGenerated {
                module: import.key.module.clone(),
                export_hash: Some(import.key.export_hash),
                parent_name: parent.name.clone(),
                name: generated.name.clone(),
                parent_decl_interface_hash: parent.hashes.decl_interface_hash,
                decl_interface_hash: generated.export.decl_interface_hash,
                public_export: true,
                tactic_head_visible: direct_public_tactic_head_visible(
                    session,
                    &import.key.module,
                    &generated.name,
                    &import.key.export_hash,
                    &generated.export.decl_interface_hash,
                ),
            })
        }
        ExportKind::Axiom | ExportKind::Def | ExportKind::Theorem | ExportKind::Inductive => {
            Ok(MachineGlobalRefView::Imported {
                module: import.key.module.clone(),
                name: name.clone(),
                export_hash: import.key.export_hash,
                decl_interface_hash: *decl_interface_hash,
                public_export: true,
                tactic_head_visible: direct_public_tactic_head_visible(
                    session,
                    &import.key.module,
                    name,
                    &import.key.export_hash,
                    decl_interface_hash,
                ),
            })
        }
    }
}

fn ordinary_public_export<'a>(
    import: &'a VerifiedModuleContextEntry,
    name: &Name,
    decl_interface_hash: &Hash,
) -> Result<Option<&'a ExportEntry>, TheoremSearchBuildError> {
    let export = unique_public_export(import, name, decl_interface_hash)?;
    Ok(
        export
            .filter(|entry| !matches!(entry.kind, ExportKind::Constructor | ExportKind::Recursor)),
    )
}

fn unique_public_export<'a>(
    import: &'a VerifiedModuleContextEntry,
    name: &Name,
    decl_interface_hash: &Hash,
) -> Result<Option<&'a ExportEntry>, TheoremSearchBuildError> {
    let mut matches = import.export_block.iter().filter(|export| {
        export.decl_interface_hash == *decl_interface_hash
            && export_name(import, export).is_ok_and(|export_name| export_name == *name)
    });
    let first = matches.next();
    if matches.next().is_some() {
        return Err(TheoremSearchBuildError::DuplicatePublicName);
    }
    Ok(first)
}

fn direct_public_tactic_head_visible(
    session: &MachineProofSession,
    module: &Name,
    name: &Name,
    export_hash: &Hash,
    decl_interface_hash: &Hash,
) -> bool {
    session
        .import_certificate_context
        .direct_import_entries()
        .into_iter()
        .any(|entry| {
            entry.key.module == *module
                && entry.key.export_hash == *export_hash
                && unique_public_export(entry, name, decl_interface_hash)
                    .is_ok_and(|export| export.is_some())
        })
}

fn term_node(
    owner: &VerifiedModuleContextEntry,
    term: TermId,
) -> Result<&TermNode, TheoremSearchBuildError> {
    owner
        .verified_module
        .term_table()
        .get(term)
        .ok_or(TheoremSearchBuildError::MissingTerm)
}

fn name_from_owner(
    owner: &VerifiedModuleContextEntry,
    name: npa_cert::NameId,
) -> Result<Name, TheoremSearchBuildError> {
    owner
        .decoded_name_table
        .get(name)
        .cloned()
        .ok_or(TheoremSearchBuildError::MissingName)
}

fn has_leading_pi(expr: &Expr) -> bool {
    matches!(expr, Expr::Pi { .. })
}

fn export_name(
    import: &VerifiedModuleContextEntry,
    export: &ExportEntry,
) -> Result<Name, TheoremSearchBuildError> {
    import
        .decoded_name_table
        .get(export.name)
        .cloned()
        .ok_or(TheoremSearchBuildError::MissingName)
}

fn export_universe_params(
    import: &VerifiedModuleContextEntry,
    export: &ExportEntry,
) -> Result<Vec<String>, TheoremSearchBuildError> {
    export
        .universe_params
        .iter()
        .map(|name_id| {
            let name = import
                .decoded_name_table
                .get(*name_id)
                .ok_or(TheoremSearchBuildError::MissingName)?;
            let [component] = name.0.as_slice() else {
                return Err(TheoremSearchBuildError::InvalidUniverseParamName);
            };
            if crate::is_machine_universe_param_name(component) {
                Ok(component.clone())
            } else {
                Err(TheoremSearchBuildError::InvalidUniverseParamName)
            }
        })
        .collect()
}

fn parse_modes(
    value: &JsonValue<'_>,
    path: &JsonPath,
) -> Result<Vec<MachineTheoremMode>, MachineApiRequestError> {
    let elements = value.array_elements().ok_or_else(|| {
        request_error(
            path,
            MachineApiRequestErrorReason::TypeMismatch {
                field: "modes",
                expected: JsonFieldType::Array,
                actual: value.kind(),
            },
        )
    })?;
    if elements.is_empty() {
        return Err(request_error(
            path,
            MachineApiRequestErrorReason::MissingField { field: "modes" },
        ));
    }
    let mut seen = BTreeSet::new();
    let mut modes = Vec::new();
    for (index, item) in elements.iter().enumerate() {
        let item_path = path.index(index);
        let Some(text) = item.string_value() else {
            return Err(request_error(
                &item_path,
                if item.kind() == JsonValueKind::Null {
                    MachineApiRequestErrorReason::NullField { field: "modes" }
                } else {
                    MachineApiRequestErrorReason::TypeMismatch {
                        field: "modes",
                        expected: JsonFieldType::String,
                        actual: item.kind(),
                    }
                },
            ));
        };
        let Some(mode) = MachineTheoremMode::parse(text) else {
            return Err(request_error(
                &item_path,
                MachineApiRequestErrorReason::TypeMismatch {
                    field: "modes",
                    expected: JsonFieldType::String,
                    actual: JsonValueKind::String,
                },
            ));
        };
        if !seen.insert(mode) {
            return Err(request_error(
                &item_path,
                MachineApiRequestErrorReason::DuplicateKey {
                    key: text.to_owned(),
                },
            ));
        }
        modes.push(mode);
    }
    modes.sort();
    Ok(modes)
}

fn parse_limit(value: &JsonValue<'_>) -> u32 {
    let raw = value
        .number_raw()
        .expect("endpoint validation checked search limit integer");
    raw.parse::<u32>()
        .expect("endpoint validation checked search limit <= 256")
}

fn parse_filters(
    value: &JsonValue<'_>,
    path: &JsonPath,
) -> Result<MachineTheoremFilters, MachineApiRequestError> {
    let object = validate_json_object(
        value,
        ObjectSchema::new(MachineApiErrorKind::InvalidTheoremQuery, FILTER_FIELDS),
        path,
    )?;
    let exclude_axioms = object
        .field("exclude_axioms")
        .and_then(JsonValue::bool_value)
        .expect("filter schema checked exclude_axioms bool");
    let allowed_modules = match object.field("allowed_modules") {
        Some(value) => {
            let elements = value.array_elements().ok_or_else(|| {
                request_error(
                    &path.field("allowed_modules"),
                    MachineApiRequestErrorReason::TypeMismatch {
                        field: "allowed_modules",
                        expected: JsonFieldType::Array,
                        actual: value.kind(),
                    },
                )
            })?;
            let mut modules = Vec::with_capacity(elements.len());
            for (index, item) in elements.iter().enumerate() {
                let item_path = path.field("allowed_modules").index(index);
                let Some(text) = item.string_value() else {
                    return Err(request_error(
                        &item_path,
                        if item.kind() == JsonValueKind::Null {
                            MachineApiRequestErrorReason::NullField {
                                field: "allowed_modules",
                            }
                        } else {
                            MachineApiRequestErrorReason::TypeMismatch {
                                field: "allowed_modules",
                                expected: JsonFieldType::String,
                                actual: item.kind(),
                            }
                        },
                    ));
                };
                let module = parse_module_name_wire(text).map_err(|_| {
                    request_error(
                        &item_path,
                        MachineApiRequestErrorReason::TypeMismatch {
                            field: "allowed_modules",
                            expected: JsonFieldType::String,
                            actual: JsonValueKind::String,
                        },
                    )
                })?;
                modules.push(module);
            }
            modules.sort_by_key(|module| phase5_name_canonical_bytes(module).unwrap_or_default());
            modules.dedup();
            MachineAllowedModulesFilter::Explicit(modules)
        }
        None => MachineAllowedModulesFilter::AllDirect,
    };
    Ok(MachineTheoremFilters {
        exclude_axioms,
        allowed_modules,
    })
}

fn canonicalize_allowed_modules_for_session(
    session: &MachineProofSession,
    filters: &mut MachineTheoremFilters,
) -> Result<(), Box<MachineTheoremSearchError>> {
    let mut direct_modules = session
        .import_certificate_context
        .direct_import_entries()
        .into_iter()
        .map(|entry| entry.key.module.clone())
        .collect::<Vec<_>>();
    direct_modules.sort_by_key(|module| phase5_name_canonical_bytes(module).unwrap_or_default());
    direct_modules.dedup();

    if let MachineAllowedModulesFilter::Explicit(modules) = &mut filters.allowed_modules {
        for module in modules.iter() {
            if !direct_modules.contains(module) {
                return Err(search_plain_error(
                    MachineApiErrorKind::InvalidTheoremQuery,
                    MachineApiDiagnosticPhase::RequestValidation,
                    format!(
                        "allowed module {} is not a direct import of the session",
                        module.as_dotted()
                    ),
                ));
            }
        }
        if *modules == direct_modules {
            filters.allowed_modules = MachineAllowedModulesFilter::AllDirect;
        }
    }
    Ok(())
}

fn required_field<'value, 'src>(
    envelope: &crate::MachineValidatedEndpointEnvelope<'value, 'src>,
    field: &str,
) -> &'value JsonValue<'src> {
    envelope
        .field(field)
        .expect("endpoint validation checked required field")
}

fn required_string<'value, 'src>(
    envelope: &crate::MachineValidatedEndpointEnvelope<'value, 'src>,
    field: &str,
) -> &'value str {
    required_field(envelope, field)
        .string_value()
        .expect("endpoint validation checked required string field")
}

fn request_error(path: &JsonPath, reason: MachineApiRequestErrorReason) -> MachineApiRequestError {
    MachineApiRequestError::new(
        MachineApiErrorKind::InvalidTheoremQuery,
        path.clone(),
        reason,
    )
}

fn theorem_index_fingerprint(session: &MachineProofSession, entries: &[TheoremIndexEntry]) -> Hash {
    let mut out = Vec::new();
    encode_string(&mut out, "npa.phase5.theorem-index.v1");
    encode_string(&mut out, session.protocol_version.as_str());
    out.extend(session.session_root_hash);
    encode_string(&mut out, THEOREM_INDEX_SCHEMA_VERSION);
    encode_list_len(&mut out, entries.len());
    for entry in entries {
        out.extend(&entry.canonical_bytes);
    }
    sha256(&out)
}

fn theorem_index_entry_canonical_bytes(
    global_ref: &MachineTheoremGlobalRef,
    universe_params: &[String],
    statement_core_hash: Hash,
    head: Option<&MachineGlobalRefView>,
    axiom_dependencies_hash: Hash,
    modes: &[MachineTheoremMode],
) -> Vec<u8> {
    let mut out = Vec::new();
    encode_string(&mut out, "npa.phase5.theorem-index-entry.v1");
    encode_theorem_global_ref(&mut out, global_ref);
    encode_list_len(&mut out, universe_params.len());
    for param in universe_params {
        encode_string(&mut out, param);
    }
    out.extend(statement_core_hash);
    encode_option_global_ref_view(&mut out, head);
    out.extend(axiom_dependencies_hash);
    encode_list_len(&mut out, modes.len());
    for mode in modes {
        encode_string(&mut out, mode.as_str());
    }
    out
}

struct QueryFingerprintInput<'a> {
    protocol_version: MachineApiVersion,
    state_fingerprint: Hash,
    goal_id: npa_tactic::GoalId,
    goal_fingerprint: Hash,
    theorem_index_fingerprint: Hash,
    modes: &'a [MachineTheoremMode],
    filters: &'a MachineTheoremFilters,
    limit: u32,
}

fn theorem_query_fingerprint(input: QueryFingerprintInput<'_>) -> Hash {
    let mut out = Vec::new();
    encode_string(&mut out, "npa.phase5.theorem-query.v1");
    encode_string(&mut out, input.protocol_version.as_str());
    out.extend(input.state_fingerprint);
    out.extend(npa_tactic::goal_id_canonical_bytes(input.goal_id));
    out.extend(input.goal_fingerprint);
    out.extend(input.theorem_index_fingerprint);
    encode_list_len(&mut out, input.modes.len());
    for mode in input.modes {
        encode_string(&mut out, mode.as_str());
    }
    encode_filters(&mut out, input.filters);
    encode_string(&mut out, SEARCH_PROFILE_VERSION);
    encode_string(&mut out, SUGGESTION_PROFILE_VERSION);
    encode_uvar(&mut out, u64::from(input.limit));
    sha256(&out)
}

fn encode_filters(out: &mut Vec<u8>, filters: &MachineTheoremFilters) {
    encode_string(out, "npa.phase5.theorem-filters.v1");
    encode_bool(out, filters.exclude_axioms);
    match &filters.allowed_modules {
        MachineAllowedModulesFilter::AllDirect => out.push(0x00),
        MachineAllowedModulesFilter::Explicit(modules) => {
            out.push(0x01);
            encode_list_len(out, modules.len());
            for module in modules {
                encode_name(out, module);
            }
        }
    }
}

fn encode_theorem_global_ref(out: &mut Vec<u8>, global_ref: &MachineTheoremGlobalRef) {
    encode_name(out, &global_ref.module);
    encode_name(out, &global_ref.name);
    out.extend(global_ref.export_hash);
    out.extend(global_ref.decl_interface_hash);
}

fn encode_option_global_ref_view(out: &mut Vec<u8>, value: Option<&MachineGlobalRefView>) {
    match value {
        Some(value) => {
            out.push(0x01);
            out.extend(value.canonical_bytes());
        }
        None => out.push(0x00),
    }
}

fn axiom_dependencies_hash(axioms: &[AxiomRef]) -> Hash {
    let mut ordered = axioms.to_vec();
    ordered.sort();
    let mut out = Vec::new();
    encode_list_len(&mut out, ordered.len());
    for axiom in &ordered {
        encode_global_ref(&mut out, &axiom.global_ref);
        encode_uvar(&mut out, axiom.name as u64);
        out.extend(axiom.decl_interface_hash);
    }
    sha256(&out)
}

fn sort_dedup_axiom_refs(entries: &mut Vec<MachineAxiomRefWire>) {
    entries.sort_by_key(encode_machine_axiom_ref_wire);
    entries.dedup_by(|lhs, rhs| {
        encode_machine_axiom_ref_wire(lhs) == encode_machine_axiom_ref_wire(rhs)
    });
}

fn theorem_entry_sort_key(entry: &TheoremIndexEntry) -> Vec<u8> {
    let mut out = Vec::new();
    encode_name(&mut out, &entry.global_ref.module);
    encode_name(&mut out, &entry.global_ref.name);
    out.extend(entry.global_ref.export_hash);
    out.extend(entry.global_ref.decl_interface_hash);
    out
}

fn simp_rule_ref_canonical_bytes(rule: &SimpRuleRef) -> Vec<u8> {
    let mut out = Vec::new();
    encode_name(&mut out, &rule.name);
    out.extend(rule.decl_interface_hash);
    out.push(match rule.direction {
        RewriteDirection::Forward => 0x00,
        RewriteDirection::Backward => 0x01,
    });
    out
}

fn canonical_modes() -> Vec<MachineTheoremMode> {
    vec![
        MachineTheoremMode::Exact,
        MachineTheoremMode::Apply,
        MachineTheoremMode::Rw,
        MachineTheoremMode::Simp,
    ]
}

fn encode_global_ref(out: &mut Vec<u8>, global_ref: &GlobalRef) {
    match global_ref {
        GlobalRef::Imported {
            import_index,
            name,
            decl_interface_hash,
        } => {
            out.push(0x00);
            encode_uvar(out, *import_index as u64);
            encode_uvar(out, *name as u64);
            out.extend(decl_interface_hash);
        }
        GlobalRef::Local { decl_index } => {
            out.push(0x01);
            encode_uvar(out, *decl_index as u64);
        }
        GlobalRef::LocalGenerated { decl_index, name } => {
            out.push(0x02);
            encode_uvar(out, *decl_index as u64);
            encode_uvar(out, *name as u64);
        }
        GlobalRef::Builtin {
            name,
            decl_interface_hash,
        } => {
            out.push(0x03);
            encode_uvar(out, *name as u64);
            out.extend(decl_interface_hash);
        }
    }
}

fn encode_name(out: &mut Vec<u8>, name: &Name) {
    encode_uvar(out, name.0.len() as u64);
    for component in &name.0 {
        encode_string(out, component);
    }
}

fn encode_string(out: &mut Vec<u8>, value: &str) {
    encode_uvar(out, value.len() as u64);
    out.extend(value.as_bytes());
}

fn encode_list_len(out: &mut Vec<u8>, len: usize) {
    encode_uvar(out, len as u64);
}

fn encode_bool(out: &mut Vec<u8>, value: bool) {
    out.push(if value { 0x01 } else { 0x00 });
}

fn encode_uvar(out: &mut Vec<u8>, mut value: u64) {
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
}

fn sha256(bytes: &[u8]) -> Hash {
    Sha256::digest(bytes).into()
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TheoremSearchBuildError {
    InvalidVerifiedImport,
    DuplicatePublicName,
    MissingKernelDecl,
    MissingTerm,
    MissingName,
    MissingDeclIndex,
    MissingGeneratedDecl,
    MissingImportTableEntry,
    MissingImportedModule,
    MissingPublicExport,
    InvalidUniverseParamName,
    BuiltinGlobalRefUnsupported,
    DisplayRefMissing,
    DisplayRefMismatch,
    InvalidAxiomRef,
    RenderFailed,
    SuggestedCandidateInvalid,
}

fn search_request_error(error: MachineApiRequestError) -> Box<MachineTheoremSearchError> {
    search_plain_error(
        error.kind,
        MachineApiDiagnosticPhase::RequestValidation,
        format!(
            "request validation failed at {}: {:?}",
            json_path_display(&error.path),
            error.reason
        ),
    )
}

fn search_snapshot_lookup_error(
    error: MachineSnapshotLookupError,
) -> Box<MachineTheoremSearchError> {
    let kind = match error {
        MachineSnapshotLookupError::UnknownSnapshot { .. } => MachineApiErrorKind::UnknownSnapshot,
        MachineSnapshotLookupError::StateFingerprintMismatch { .. } => {
            MachineApiErrorKind::StateFingerprintMismatch
        }
        MachineSnapshotLookupError::SnapshotIdentityMismatch { .. }
        | MachineSnapshotLookupError::InvalidMachineProofState { .. }
        | MachineSnapshotLookupError::ExecutableStateFingerprintMismatch { .. }
        | MachineSnapshotLookupError::StoredSnapshotViewMismatch { .. } => {
            MachineApiErrorKind::InvalidMachineProofState
        }
    };
    search_plain_error(
        kind,
        MachineApiDiagnosticPhase::SnapshotLookup,
        format!("snapshot lookup failed: {error:?}"),
    )
}

fn search_theorem_index_error(error: TheoremSearchBuildError) -> Box<MachineTheoremSearchError> {
    search_plain_error(
        MachineApiErrorKind::InvalidTheoremIndex,
        MachineApiDiagnosticPhase::TheoremSearch,
        format!("theorem index construction failed: {error:?}"),
    )
}

fn search_plain_error(
    kind: MachineApiErrorKind,
    phase: MachineApiDiagnosticPhase,
    message: impl Into<String>,
) -> Box<MachineTheoremSearchError> {
    search_error(kind, phase, None, message)
}

fn search_goal_error(
    kind: MachineApiErrorKind,
    phase: MachineApiDiagnosticPhase,
    goal_id: npa_tactic::GoalId,
    message: impl Into<String>,
) -> Box<MachineTheoremSearchError> {
    search_error(kind, phase, Some(goal_id), message)
}

fn search_error(
    kind: MachineApiErrorKind,
    phase: MachineApiDiagnosticPhase,
    goal_id: Option<npa_tactic::GoalId>,
    message: impl Into<String>,
) -> Box<MachineTheoremSearchError> {
    let message = message.into();
    let diagnostic = MachineApiDiagnosticProjection {
        kind,
        phase,
        retryable: false,
        goal_id,
        tactic_kind: None,
        primary_name: None,
        primary_axiom_ref: None,
        expected_hash: None,
        actual_hash: None,
        source_message: message.clone(),
        upstream: Phase5UpstreamDiagnostic::Phase4(npa_tactic::MachineTacticDiagnostic::new(
            npa_tactic::MachineTacticDiagnosticKind::InvalidMachineProofState,
            message,
        )),
    };
    let wire = MachineApiErrorWire::from_projection(&diagnostic)
        .expect("search diagnostics must satisfy Phase 5 wire invariants");
    let response = MachineApiResponseEnvelope::Error(Box::new(MachineApiErrorResponse {
        status: MachineApiResponseStatus::Error,
        error: wire,
        endpoint_fields: (),
    }));
    Box::new(MachineTheoremSearchError {
        diagnostic,
        response,
    })
}

fn json_path_display(path: &JsonPath) -> String {
    if path.elements.is_empty() {
        return "$".to_owned();
    }
    let mut out = "$".to_owned();
    for element in &path.elements {
        match element {
            JsonPathElement::Field(field) => {
                out.push('.');
                out.push_str(field);
            }
            JsonPathElement::Index(index) => {
                out.push('[');
                out.push_str(&index.to_string());
                out.push(']');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        create_machine_session, format_hash_string, project_import_certificate_context,
        MachineDisplayRenderScope, MachineDisplayRenderScopeEntry,
        Phase5ResolvedDisplayCoreRefOwner, VerifiedImportKey, VerifiedModuleCertificateInput,
    };
    use npa_cert::{
        build_module_cert, encode_module_cert, verify_module_cert, AxiomPolicy, CoreModule,
        VerifierSession,
    };
    use npa_frontend::{MachineGlobalScopeEntry, MachineSurfaceCallableRef};
    use npa_kernel::{Decl, Level};

    fn prop() -> Expr {
        Expr::sort(Level::zero())
    }

    fn imported_axiom_type() -> Expr {
        Expr::pi("p", prop(), prop())
    }

    fn default_options_json(allow_axioms: &str) -> String {
        format!(
            r#"{{
              "kernel_check_profile":"npa.kernel.v0.1.builtin-nat-eq-rec",
              "allow_axioms": {allow_axioms},
              "tactic_options": {{
                "simp_rules": [],
                "eq_family": null,
                "nat_family": null,
                "max_simp_rewrite_steps": 100,
                "max_open_goals": 32,
                "max_metas": 64
              }}
            }}"#
        )
    }

    fn imported_axiom_session() -> MachineProofSession {
        let module = CoreModule {
            name: Name::from_dotted("A"),
            declarations: vec![Decl::Axiom {
                name: "A.id".to_owned(),
                universe_params: Vec::new(),
                ty: imported_axiom_type(),
            }],
        };
        let cert = build_module_cert(module, &[]).unwrap();
        let bytes = encode_module_cert(&cert).unwrap();
        let mut verifier = VerifierSession::new();
        let mut policy = AxiomPolicy::high_trust();
        policy.allowlisted_axioms.insert(Name::from_dotted("A.id"));
        let verified = verify_module_cert(&bytes, &mut verifier, &policy).unwrap();
        let export_hash = format_hash_string(&verified.export_hash());
        let certificate_hash = format_hash_string(&verified.certificate_hash());
        let decl_interface_hash =
            format_hash_string(&verified.declarations()[0].hashes.decl_interface_hash);
        let cert_hex = hex_bytes(&bytes);
        let allow_axioms = format!(
            r#"[{{
              "kind":"imported",
              "module":"A",
              "name":"A.id",
              "export_hash":"{export_hash}",
              "decl_interface_hash":"{decl_interface_hash}"
            }}]"#
        );
        let body = format!(
            r#"{{
              "protocol_version":"npa.machine-api.v1",
              "root":{{
                "module":"Scratch",
                "theorem_name":"Scratch.t",
                "source_index":0,
                "universe_params":[],
                "theorem_type":{{"format":"machine_surface_v1","source":"Prop"}}
              }},
              "import_closure":[{{
                "module":"A",
                "expected_export_hash":"{export_hash}",
                "expected_certificate_hash":"{certificate_hash}",
                "certificate":{{
                  "encoding":"npa.certificate.canonical.v0.1.hex",
                  "bytes":"{cert_hex}"
                }}
              }}],
              "imports":[{{
                "module":"A",
                "expected_export_hash":"{export_hash}",
                "expected_certificate_hash":"{certificate_hash}"
              }}],
              "checked_current_decls":[],
              "options":{}
            }}"#,
            default_options_json(&allow_axioms)
        );
        create_machine_session(&body).unwrap().session
    }

    fn head_collision_context() -> crate::MachineImportCertificateContext {
        let mut verifier = VerifierSession::new();
        let mut policy = AxiomPolicy::high_trust();
        policy.allowlisted_axioms.insert(Name::from_dotted("X"));
        policy.allowlisted_axioms.insert(Name::from_dotted("A.t"));

        let p_module = CoreModule {
            name: Name::from_dotted("P"),
            declarations: vec![Decl::Axiom {
                name: "X".to_owned(),
                universe_params: Vec::new(),
                ty: prop(),
            }],
        };
        let p_cert = build_module_cert(p_module, &[]).unwrap();
        let p_bytes = encode_module_cert(&p_cert).unwrap();
        let p_verified = verify_module_cert(&p_bytes, &mut verifier, &policy).unwrap();

        let a_module = CoreModule {
            name: Name::from_dotted("A"),
            declarations: vec![Decl::Axiom {
                name: "A.t".to_owned(),
                universe_params: Vec::new(),
                ty: Expr::konst("X", Vec::new()),
            }],
        };
        let a_cert = build_module_cert(a_module, std::slice::from_ref(&p_verified)).unwrap();
        let a_bytes = encode_module_cert(&a_cert).unwrap();
        let a_verified = verify_module_cert(&a_bytes, &mut verifier, &policy).unwrap();

        let b_module = CoreModule {
            name: Name::from_dotted("B"),
            declarations: vec![Decl::Axiom {
                name: "X".to_owned(),
                universe_params: Vec::new(),
                ty: prop(),
            }],
        };
        let b_cert = build_module_cert(b_module, &[]).unwrap();
        let b_bytes = encode_module_cert(&b_cert).unwrap();
        let b_verified = verify_module_cert(&b_bytes, &mut verifier, &policy).unwrap();

        let p_name = Name::from_dotted("P");
        let a_name = Name::from_dotted("A");
        let b_name = Name::from_dotted("B");
        let closure = vec![
            VerifiedModuleCertificateInput {
                module: &p_name,
                expected_export_hash: p_verified.export_hash(),
                expected_certificate_hash: p_verified.certificate_hash(),
                certificate_bytes: &p_bytes,
            },
            VerifiedModuleCertificateInput {
                module: &a_name,
                expected_export_hash: a_verified.export_hash(),
                expected_certificate_hash: a_verified.certificate_hash(),
                certificate_bytes: &a_bytes,
            },
            VerifiedModuleCertificateInput {
                module: &b_name,
                expected_export_hash: b_verified.export_hash(),
                expected_certificate_hash: b_verified.certificate_hash(),
                certificate_bytes: &b_bytes,
            },
        ];
        let direct = vec![
            VerifiedImportKey::new(
                a_name.clone(),
                a_verified.export_hash(),
                a_verified.certificate_hash(),
            ),
            VerifiedImportKey::new(
                b_name.clone(),
                b_verified.export_hash(),
                b_verified.certificate_hash(),
            ),
        ];
        project_import_certificate_context(&closure, &direct, &policy).unwrap()
    }

    fn direct_axiom_display_scope(
        context: &crate::MachineImportCertificateContext,
        module: &str,
    ) -> MachineDisplayRenderScope {
        let module = Name::from_dotted(module);
        let (import_index, entry) = context
            .direct_import_entries()
            .into_iter()
            .enumerate()
            .find(|(_, entry)| entry.key.module == module)
            .unwrap();
        let export = entry
            .export_block
            .iter()
            .find(|export| matches!(export.kind, ExportKind::Axiom))
            .unwrap();
        let export_name = export_name(entry, export).unwrap();
        let view = MachineGlobalRefView::Imported {
            module: entry.key.module.clone(),
            name: export_name.clone(),
            export_hash: entry.key.export_hash,
            decl_interface_hash: export.decl_interface_hash,
            public_export: true,
            tactic_head_visible: true,
        };
        MachineDisplayRenderScope::from_entries([MachineDisplayRenderScopeEntry::new(
            view,
            Phase5ResolvedDisplayCoreRefOwner::VerifiedImportedModule {
                owner_module: entry.key.module.clone(),
                owner_export_hash: entry.key.export_hash,
            },
            MachineSurfaceCallableRef::Imported {
                module: entry.key.module.clone(),
                name: export_name.clone(),
                export_hash: entry.key.export_hash,
                decl_interface_hash: export.decl_interface_hash,
            },
        )
        .with_candidate_resolution(MachineGlobalScopeEntry::Imported {
            name: export_name,
            import_index: import_index as u32,
            decl_interface_hash: export.decl_interface_hash,
        })])
        .unwrap()
    }

    fn head_collision_context_and_scope() -> (
        crate::MachineImportCertificateContext,
        MachineDisplayRenderScope,
    ) {
        let context = head_collision_context();
        let display_scope = direct_axiom_display_scope(&context, "B");
        (context, display_scope)
    }

    fn search_json(session: &MachineProofSession, filters: &str) -> String {
        search_json_for_goal(session, "g0", filters)
    }

    fn search_json_for_goal(session: &MachineProofSession, goal_id: &str, filters: &str) -> String {
        format!(
            r#"{{
              "session_id":"{}",
              "snapshot_id":"{}",
              "state_fingerprint":"{}",
              "goal_id":"{}",
              "modes":["apply","exact","rw","simp"],
              "limit":20,
              "filters":{}
            }}"#,
            session.session_id.wire(),
            session.initial_snapshot.snapshot_id.wire(),
            format_hash_string(&session.initial_snapshot.state_fingerprint),
            goal_id,
            filters
        )
    }

    #[test]
    fn search_returns_direct_imported_axiom_metadata_deterministically() {
        let session = imported_axiom_session();
        let filters = r#"{"exclude_axioms":false,"allowed_modules":["A"]}"#;
        let first =
            search_machine_theorems_for_goal(&search_json(&session, filters), &session).unwrap();
        let second =
            search_machine_theorems_for_goal(&search_json(&session, filters), &session).unwrap();

        let (first_fields, second_fields) = match (first, second) {
            (MachineApiResponseEnvelope::Ok(first), MachineApiResponseEnvelope::Ok(second)) => {
                (first.endpoint_fields, second.endpoint_fields)
            }
            _ => panic!("search should succeed"),
        };

        assert_eq!(
            first_fields.query_fingerprint,
            second_fields.query_fingerprint
        );
        assert_eq!(
            first_fields.theorem_index_fingerprint,
            second_fields.theorem_index_fingerprint
        );
        assert_eq!(first_fields.results.len(), 1);

        let result = &first_fields.results[0];
        assert_eq!(result.premise_id, "prem_0");
        assert_eq!(result.global_ref.module, Name::from_dotted("A"));
        assert_eq!(result.global_ref.name, Name::from_dotted("A.id"));
        assert_eq!(
            result.modes,
            vec![MachineTheoremMode::Exact, MachineTheoremMode::Apply]
        );
        assert_eq!(result.statement.machine, "forall (x : Prop), Prop");
        assert_eq!(result.suggested_candidates, Vec::new());
        assert_eq!(result.score, 0);
        assert_eq!(result.axioms_used.len(), 1);
    }

    #[test]
    fn search_exclude_axioms_filters_axiom_dependencies() {
        let session = imported_axiom_session();
        let filters = r#"{"exclude_axioms":true,"allowed_modules":["A"]}"#;
        let response =
            search_machine_theorems_for_goal(&search_json(&session, filters), &session).unwrap();
        let MachineApiResponseEnvelope::Ok(ok) = response else {
            panic!("search should succeed");
        };
        assert!(ok.endpoint_fields.results.is_empty());
    }

    #[test]
    fn search_rejects_non_direct_allowed_module_before_snapshot_lookup() {
        let session = imported_axiom_session();
        let body = search_json(
            &session,
            r#"{"exclude_axioms":false,"allowed_modules":["Missing"]}"#,
        );
        let err = search_machine_theorems_for_goal(&body, &session).unwrap_err();

        assert_eq!(
            err.diagnostic.kind,
            MachineApiErrorKind::InvalidTheoremQuery
        );
        assert_eq!(
            err.diagnostic.phase,
            MachineApiDiagnosticPhase::RequestValidation
        );
    }

    #[test]
    fn search_goal_not_open_returns_structured_error() {
        let session = imported_axiom_session();
        let body = search_json_for_goal(
            &session,
            "g99",
            r#"{"exclude_axioms":false,"allowed_modules":["A"]}"#,
        );
        let err = search_machine_theorems_for_goal(&body, &session).unwrap_err();

        assert_eq!(err.diagnostic.kind, MachineApiErrorKind::GoalNotOpen);
        assert_eq!(
            err.diagnostic.phase,
            MachineApiDiagnosticPhase::SnapshotLookup
        );
        assert_eq!(err.diagnostic.goal_id, Some(npa_tactic::GoalId(99)));
        match err.response {
            MachineApiResponseEnvelope::Error(error) => {
                assert_eq!(error.error.kind, MachineApiErrorKind::GoalNotOpen);
                assert_eq!(error.error.goal_id, Some(npa_tactic::GoalId(99)));
            }
            MachineApiResponseEnvelope::Ok(_) => panic!("search should fail"),
            MachineApiResponseEnvelope::SchedulerStopped(_) => panic!("search should fail"),
        }
    }

    #[test]
    fn search_renders_transitive_statement_ref_with_display_extension() {
        let base = imported_axiom_session();
        let snapshot_context = MachineSnapshotMaterializationContext {
            session_id: &base.session_id,
            display_scope: &base.machine_display_render_scope,
            callable_interface_table: &base.machine_surface_callable_interface_table,
        };
        let state = base
            .snapshots
            .lookup_checked(
                &snapshot_context,
                base.initial_snapshot.snapshot_id,
                base.initial_snapshot.state_fingerprint,
            )
            .unwrap()
            .executable_state_payload
            .clone();
        let context = head_collision_context();
        let display_scope = direct_axiom_display_scope(&context, "A");
        let mut session = base;
        session.import_certificate_context = context;
        session.machine_display_render_scope = display_scope;

        let index = build_theorem_index(&session, &state).unwrap();
        let entry = index
            .entries
            .iter()
            .find(|entry| entry.global_ref.name == Name::from_dotted("A.t"))
            .unwrap();
        let Some(MachineGlobalRefView::Imported {
            module,
            name,
            public_export,
            tactic_head_visible,
            ..
        }) = entry.head.as_ref()
        else {
            panic!("transitive theorem statement head should resolve to an imported ref");
        };

        assert_eq!(module, &Name::from_dotted("P"));
        assert_eq!(name, &Name::from_dotted("X"));
        assert!(*public_export);
        assert!(!*tactic_head_visible);
        assert_eq!(render_statement(&session, entry).unwrap().machine, "X");
    }

    #[test]
    fn search_rejects_head_name_collision_with_transitive_owner() {
        let base = imported_axiom_session();
        let snapshot_context = MachineSnapshotMaterializationContext {
            session_id: &base.session_id,
            display_scope: &base.machine_display_render_scope,
            callable_interface_table: &base.machine_surface_callable_interface_table,
        };
        let state = base
            .snapshots
            .lookup_checked(
                &snapshot_context,
                base.initial_snapshot.snapshot_id,
                base.initial_snapshot.state_fingerprint,
            )
            .unwrap()
            .executable_state_payload
            .clone();
        let (context, display_scope) = head_collision_context_and_scope();
        let mut session = base;
        session.import_certificate_context = context;
        session.machine_display_render_scope = display_scope;

        assert_eq!(
            build_theorem_index(&session, &state).unwrap_err(),
            TheoremSearchBuildError::DisplayRefMismatch
        );
    }

    fn hex_bytes(bytes: &[u8]) -> String {
        let mut out = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            out.push(hex_digit(byte >> 4));
            out.push(hex_digit(byte & 0x0f));
        }
        out
    }

    fn hex_digit(value: u8) -> char {
        match value {
            0..=9 => char::from(b'0' + value),
            10..=15 => char::from(b'a' + (value - 10)),
            _ => unreachable!("hex nybble is in range"),
        }
    }
}
