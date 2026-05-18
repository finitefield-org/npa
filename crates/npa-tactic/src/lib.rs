use std::{
    cell::Cell,
    collections::{BTreeMap, BTreeSet},
};

use npa_cert::{
    CoreModule, DeclHashes, DeclPayload, ExportKind, Hash, ModuleName, Name, VerifiedModule,
};
use npa_kernel::expr::collect_apps;
use npa_kernel::level::{ensure_level_wf, levels_eq, normalize_level};
use npa_kernel::subst::{instantiate, shift, subst_levels_expr};
use npa_kernel::{Ctx, Decl, Env, Expr, Level, ResourceLimitKind};
use sha2::{Digest, Sha256};

pub type Result<T> = std::result::Result<T, MachineTacticDiagnostic>;

const ZERO_HASH: Hash = [0; 32];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MetaVarId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GoalId(pub u64);

impl From<MetaVarId> for GoalId {
    fn from(id: MetaVarId) -> Self {
        Self(id.0)
    }
}

impl From<GoalId> for MetaVarId {
    fn from(id: GoalId) -> Self {
        Self(id.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MachineTacticDiagnosticKind {
    InvalidTacticOption,
    InvalidBatchPolicy,
    UnsupportedTacticOption,
    InvalidMachineTactic,
    InvalidMachineTermSource,
    MachineTermElaborationError,
    UnknownName,
    ImplicitArgumentRequired,
    UnsupportedMachineTactic,
    TacticFuelExhausted { kind: TacticFuelKind },
    InvalidMachineProofState,
    InvalidMachineProofSpec,
    InvalidVerifiedImport,
    AmbiguousKernelEnvDecl,
    InvalidCurrentDeclOrder,
    UncheckedCurrentDecl,
    CurrentDeclSignatureMismatch,
    UnknownGoal,
    GoalAlreadyAssigned,
    UnknownMeta,
    GoalLimitExceeded,
    MetaLimitExceeded,
    InvalidMetaDependency,
    InvalidMetaContext,
    ProofExprScopeError,
    ProofExprTypeMismatch,
    UnknownTacticHead,
    AmbiguousTacticHead,
    UnknownLocalName,
    AmbiguousLocalName,
    InvalidLocalHead,
    ExpectedFunctionType,
    ExpectedPiTarget,
    UniverseArgumentMismatch,
    MissingExplicitArgument,
    AmbiguousApplyArgument,
    TooManyApplyArguments,
    TooFewApplyArguments,
    SubgoalDataArgument,
    ExpectedEqTarget,
    UnknownSimpRule,
    AmbiguousSimpRule,
    InvalidSimpRule,
    SimpNoProgress,
    SimpStepLimitExceeded,
    AmbiguousRewriteRule,
    TacticPrimitiveUnavailable,
    InvalidEqFamily,
    InvalidNatFamily,
    InvalidInductionTarget,
    TypeMismatch,
    KernelRejected,
    UnresolvedGoal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TacticFuelKind {
    TacticStep,
    Whnf,
    Conversion,
    Rewrite,
    MetaAllocation,
    ExprNode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTacticDiagnostic {
    pub kind: MachineTacticDiagnosticKind,
    pub message: Box<str>,
    pub expected_hash: Option<Box<Hash>>,
    pub actual_hash: Option<Box<Hash>>,
    pub goal_id: Option<GoalId>,
    pub tactic_kind: Option<String>,
    pub primary_name: Option<Name>,
    pub meta_id: Option<MetaVarId>,
}

impl MachineTacticDiagnostic {
    pub fn new(kind: MachineTacticDiagnosticKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into().into_boxed_str(),
            expected_hash: None,
            actual_hash: None,
            goal_id: None,
            tactic_kind: None,
            primary_name: None,
            meta_id: None,
        }
    }

    fn with_expected_actual_payloads(
        mut self,
        payload_kind: DiagnosticPayloadKind,
        expected_payload: &[u8],
        actual_payload: &[u8],
    ) -> Self {
        self = self.with_expected_actual_payload_kinds(
            payload_kind,
            expected_payload,
            payload_kind,
            actual_payload,
        );
        self
    }

    fn with_expected_actual_payload_kinds(
        mut self,
        expected_payload_kind: DiagnosticPayloadKind,
        expected_payload: &[u8],
        actual_payload_kind: DiagnosticPayloadKind,
        actual_payload: &[u8],
    ) -> Self {
        self.expected_hash = Some(Box::new(expected_actual_diagnostic_hash(
            &self.kind,
            DiagnosticHashSide::Expected,
            expected_payload_kind,
            expected_payload,
        )));
        self.actual_hash = Some(Box::new(expected_actual_diagnostic_hash(
            &self.kind,
            DiagnosticHashSide::Actual,
            actual_payload_kind,
            actual_payload,
        )));
        self
    }

    fn with_goal(mut self, goal_id: GoalId) -> Self {
        self.goal_id = Some(goal_id);
        self
    }

    fn with_primary_name(mut self, primary_name: Name) -> Self {
        self.primary_name = Some(primary_name);
        self
    }

    fn with_meta(mut self, meta_id: MetaVarId) -> Self {
        self.meta_id = Some(meta_id);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DiagnosticHashSide {
    Expected,
    Actual,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DiagnosticPayloadKind {
    Expr,
    CheckedDeclSignature,
    MachineTermSource,
    UniverseParamList,
    LevelArgList,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineProofSpec {
    pub module: ModuleName,
    pub theorem_name: Name,
    pub source_index: u64,
    pub universe_params: Vec<String>,
    pub theorem_type: Expr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawMachineTerm {
    pub source: String,
}

impl RawMachineTerm {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MachineTacticCandidate {
    Exact {
        term: RawMachineTerm,
    },
    Intro {
        name: String,
    },
    Apply {
        head: TacticHead,
        universe_args: Vec<Level>,
        args: Vec<CandidateApplyArg>,
    },
    Rewrite {
        rule: CandidateRewriteRuleRef,
        direction: RewriteDirection,
        site: RewriteSite,
    },
    SimpLite {
        rules: Vec<SimpRuleRef>,
    },
    InductionNat {
        local_name: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CandidateApplyArg {
    Term(RawMachineTerm),
    Subgoal { name_hint: Option<String> },
    InferFromTarget,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TacticHead {
    Imported {
        name: Name,
        decl_interface_hash: Hash,
    },
    CurrentModule {
        name: Name,
        decl_interface_hash: Hash,
    },
    Local {
        name: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApplyArg {
    Term(MachineTermSource),
    Subgoal { name_hint: Option<String> },
    InferFromTarget,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CandidateRewriteRuleRef {
    pub head: TacticHead,
    pub universe_args: Vec<Level>,
    pub args: Vec<CandidateApplyArg>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewriteRuleRef {
    pub head: TacticHead,
    pub universe_args: Vec<Level>,
    pub args: Vec<ApplyArg>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RewriteDirection {
    Forward,
    Backward,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RewriteSite {
    EqTargetLeft,
    EqTargetRight,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTermSource {
    source: String,
    canonical_hash: Hash,
}

impl MachineTermSource {
    pub fn new_checked(source: impl Into<String>) -> Result<Self> {
        let source = source.into();
        let canonical = npa_frontend::canonicalize_machine_term_source(&source).map_err(|err| {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidMachineTermSource,
                format!(
                    "machine term source canonicalization failed: {}",
                    err.message
                ),
            )
        })?;
        Ok(Self {
            source,
            canonical_hash: machine_term_source_hash_from_phase3(&canonical.canonical_bytes),
        })
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn canonical_hash(&self) -> Hash {
        self.canonical_hash
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofRoot {
    pub module: ModuleName,
    pub theorem_name: Name,
    pub source_index: u64,
    pub universe_params: Vec<String>,
    pub theorem_type: Expr,
    pub body: ProofExpr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProofExpr {
    Core(Expr),
    Meta(MetaVarId),
    App(Box<ProofExpr>, Box<ProofExpr>),
    Lam {
        binder: String,
        ty: Expr,
        body: Box<ProofExpr>,
    },
    Let {
        binder: String,
        ty: Expr,
        value: Box<ProofExpr>,
        body: Box<ProofExpr>,
    },
}

impl ProofExpr {
    pub fn core(expr: Expr) -> Self {
        Self::Core(expr)
    }

    pub fn meta(id: MetaVarId) -> Self {
        Self::Meta(id)
    }

    pub fn app(fun: Self, arg: Self) -> Self {
        Self::App(Box::new(fun), Box::new(arg))
    }

    pub fn lam(binder: impl Into<String>, ty: Expr, body: Self) -> Self {
        Self::Lam {
            binder: binder.into(),
            ty,
            body: Box::new(body),
        }
    }

    pub fn let_in(binder: impl Into<String>, ty: Expr, value: Self, body: Self) -> Self {
        Self::Let {
            binder: binder.into(),
            ty,
            value: Box::new(value),
            body: Box::new(body),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineLocalDecl {
    pub name: String,
    pub ty: Expr,
    pub value: Option<Expr>,
}

impl MachineLocalDecl {
    pub fn assumption(name: impl Into<String>, ty: Expr) -> Self {
        Self {
            name: name.into(),
            ty,
            value: None,
        }
    }

    pub fn definition(name: impl Into<String>, ty: Expr, value: Expr) -> Self {
        Self {
            name: name.into(),
            ty,
            value: Some(value),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineGoal {
    pub id: GoalId,
    pub meta_id: MetaVarId,
    pub context: Vec<MachineLocalDecl>,
    pub context_hash: Hash,
    pub target: Expr,
    pub target_hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineMetaVar {
    pub id: MetaVarId,
    pub goal_id: GoalId,
    pub context: Vec<MachineLocalDecl>,
    pub target: Expr,
    pub assignment: Option<ProofExpr>,
    pub creation_index: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MetaVarStore {
    pub metas: BTreeMap<MetaVarId, MachineMetaVar>,
    pub next_id: u64,
}

impl MetaVarStore {
    pub fn get(&self, id: MetaVarId) -> Option<&MachineMetaVar> {
        self.metas.get(&id)
    }

    pub fn get_mut(&mut self, id: MetaVarId) -> Option<&mut MachineMetaVar> {
        self.metas.get_mut(&id)
    }

    pub fn len(&self) -> usize {
        self.metas.len()
    }

    pub fn is_empty(&self) -> bool {
        self.metas.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineNewGoalSpec {
    pub context: Vec<MachineLocalDecl>,
    pub target: Expr,
}

impl MachineNewGoalSpec {
    pub fn new(context: Vec<MachineLocalDecl>, target: Expr) -> Self {
        Self { context, target }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineNewMetaDelta {
    pub meta_id: MetaVarId,
    pub goal_id: GoalId,
    pub context_hash: Hash,
    pub target_hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineProofDelta {
    pub from_state_fingerprint: Hash,
    pub to_state_fingerprint: Hash,
    pub assigned_goal: GoalId,
    pub assigned_meta: MetaVarId,
    pub proof_expr_hash: Hash,
    pub added_goals: Vec<GoalId>,
    pub new_metas: Vec<MachineNewMetaDelta>,
    pub delta_hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedExportSignature {
    pub name: Name,
    pub kind: ExportKind,
    pub decl_interface_hash: Hash,
    pub type_hash: Hash,
    pub body_hash: Option<Hash>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedImportRef {
    module: ModuleName,
    export_hash: Hash,
    certificate_hash: Hash,
    exports: Vec<VerifiedExportSignature>,
    certified_env_decls: Vec<Decl>,
    certified_env_decl_hashes: Vec<Hash>,
    verified_module: Box<VerifiedModule>,
}

impl VerifiedImportRef {
    pub fn from_verified_module(module: &VerifiedModule) -> Result<Self> {
        let mut exported_names = BTreeSet::new();
        let mut exports = Vec::new();
        for entry in module.export_block() {
            let name = module
                .name_table()
                .get(entry.name)
                .cloned()
                .ok_or_else(|| {
                    MachineTacticDiagnostic::new(
                        MachineTacticDiagnosticKind::InvalidVerifiedImport,
                        "verified import export entry references a missing name table entry",
                    )
                })?;
            if !exported_names.insert(name.clone()) {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::InvalidVerifiedImport,
                    format!(
                        "verified import {} exports duplicate name {}",
                        module.module().as_dotted(),
                        name.as_dotted()
                    ),
                ));
            }
            exports.push(VerifiedExportSignature {
                name,
                kind: entry.kind,
                decl_interface_hash: entry.decl_interface_hash,
                type_hash: entry.type_hash,
                body_hash: entry.body_hash,
            });
        }
        let certified_env_decls =
            npa_cert::verified_module_to_kernel_decls(module).map_err(|err| {
                MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::InvalidVerifiedImport,
                    format!("verified import could not be reconstructed as kernel env: {err:?}"),
                )
            })?;
        let certified_env_decl_hashes = module
            .declarations()
            .iter()
            .map(|decl| decl.hashes.decl_certificate_hash)
            .collect::<Vec<_>>();
        if certified_env_decls.len() != certified_env_decl_hashes.len() {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidVerifiedImport,
                format!(
                    "verified import {} reconstructed {} env declarations but has {} certificate declarations",
                    module.module().as_dotted(),
                    certified_env_decls.len(),
                    certified_env_decl_hashes.len()
                ),
            ));
        }
        Ok(Self {
            module: module.module().clone(),
            export_hash: module.export_hash(),
            certificate_hash: module.certificate_hash(),
            exports,
            certified_env_decls,
            certified_env_decl_hashes,
            verified_module: Box::new(module.clone()),
        })
    }

    pub fn module(&self) -> &ModuleName {
        &self.module
    }

    pub fn export_hash(&self) -> Hash {
        self.export_hash
    }

    pub fn certificate_hash(&self) -> Hash {
        self.certificate_hash
    }

    pub fn exports(&self) -> &[VerifiedExportSignature] {
        &self.exports
    }

    pub fn certified_env_decls(&self) -> &[Decl] {
        &self.certified_env_decls
    }

    pub fn certified_env_decl_hashes(&self) -> &[Hash] {
        &self.certified_env_decl_hashes
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedDeclSignature {
    name: Name,
    universe_params: Vec<String>,
    ty: Expr,
    decl_interface_hash: Hash,
}

impl CheckedDeclSignature {
    fn from_core_decl(decl: &Decl, decl_interface_hash: Hash) -> Self {
        Self {
            name: Name::from_dotted(decl.name()),
            universe_params: decl.universe_params().to_vec(),
            ty: decl.ty().clone(),
            decl_interface_hash,
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn universe_params(&self) -> &[String] {
        &self.universe_params
    }

    pub fn ty(&self) -> &Expr {
        &self.ty
    }

    pub fn decl_interface_hash(&self) -> Hash {
        self.decl_interface_hash
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedCurrentDecl {
    source_index: u64,
    signature: CheckedDeclSignature,
    core_decl: Decl,
    core_decl_hash: Hash,
    prior_chain_fingerprint: Hash,
    checked_env_fingerprint: Hash,
}

impl CheckedCurrentDecl {
    fn from_checked_parts(
        source_index: u64,
        core_decl: Decl,
        decl_interface_hash: Hash,
        core_decl_hash: Hash,
        prior_chain_fingerprint: Hash,
        checked_env_fingerprint: Hash,
    ) -> Self {
        let signature = CheckedDeclSignature::from_core_decl(&core_decl, decl_interface_hash);
        Self {
            source_index,
            signature,
            core_decl,
            core_decl_hash,
            prior_chain_fingerprint,
            checked_env_fingerprint,
        }
    }

    pub fn source_index(&self) -> u64 {
        self.source_index
    }

    pub fn signature(&self) -> &CheckedDeclSignature {
        &self.signature
    }

    pub fn core_decl(&self) -> &Decl {
        &self.core_decl
    }

    pub fn core_decl_hash(&self) -> Hash {
        self.core_decl_hash
    }

    pub fn prior_chain_fingerprint(&self) -> Hash {
        self.prior_chain_fingerprint
    }

    pub fn checked_env_fingerprint(&self) -> Hash {
        self.checked_env_fingerprint
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SimpRegistry {
    pub rules: Vec<ResolvedSimpRule>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SimpRuleRef {
    pub name: Name,
    pub decl_interface_hash: Hash,
    pub direction: RewriteDirection,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SimpRuleKey {
    name: Name,
    decl_interface_hash: Hash,
    direction: RewriteDirection,
}

impl From<&SimpRuleRef> for SimpRuleKey {
    fn from(rule: &SimpRuleRef) -> Self {
        Self {
            name: rule.name.clone(),
            decl_interface_hash: rule.decl_interface_hash,
            direction: rule.direction,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedSimpRule {
    pub key: SimpRuleRef,
    pub source: TacticHead,
    pub signature: CheckedDeclSignature,
    pub core_decl_hash: Hash,
    pub theorem_ty: Expr,
    pub universe_params: Vec<String>,
    pub rule_telescope: Vec<ResolvedRuleParam>,
    pub eq_levels: Vec<Level>,
    pub eq_type: Expr,
    pub theorem_lhs: Expr,
    pub theorem_rhs: Expr,
    pub from_pattern: Expr,
    pub to_pattern: Expr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedRuleParam {
    pub name: String,
    pub ty: Expr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EqFamilyRef {
    pub eq_name: Name,
    pub eq_interface_hash: Hash,
    pub refl_name: Name,
    pub refl_interface_hash: Hash,
    pub rec_name: Name,
    pub rec_interface_hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NatFamilyRef {
    pub nat_name: Name,
    pub nat_interface_hash: Hash,
    pub zero_name: Name,
    pub zero_interface_hash: Hash,
    pub succ_name: Name,
    pub succ_interface_hash: Hash,
    pub rec_name: Name,
    pub rec_interface_hash: Hash,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TacticBudget {
    pub max_tactic_steps: u64,
    pub max_whnf_steps: u64,
    pub max_conversion_steps: u64,
    pub max_rewrite_steps: u64,
    pub max_meta_allocations: u64,
    pub max_expr_nodes: u64,
}

impl Default for TacticBudget {
    fn default() -> Self {
        Self {
            max_tactic_steps: 8,
            max_whnf_steps: 20_000,
            max_conversion_steps: 20_000,
            max_rewrite_steps: 1_000,
            max_meta_allocations: 64,
            max_expr_nodes: 100_000,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MachineTacticBatchPolicy {
    pub max_evaluated_candidates: u32,
    pub stop_after_successes: u32,
    pub stop_after_failures: u32,
}

impl Default for MachineTacticBatchPolicy {
    fn default() -> Self {
        Self {
            max_evaluated_candidates: 256,
            stop_after_successes: 256,
            stop_after_failures: 256,
        }
    }
}

#[derive(Debug)]
struct TacticRunFuel {
    whnf_steps: Cell<usize>,
    conversion_steps: Cell<usize>,
}

impl TacticRunFuel {
    fn new(budget: TacticBudget) -> Self {
        Self {
            whnf_steps: Cell::new(fuel_to_usize(budget.max_whnf_steps)),
            conversion_steps: Cell::new(fuel_to_usize(budget.max_conversion_steps)),
        }
    }
}

type KernelFuelContext<'a> = (&'a TacticRunFuel, GoalId, MetaVarId);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedEqFamily {
    pub eq_name: Name,
    pub refl_name: Name,
    pub rec_name: Name,
    pub fingerprint: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedNatFamily {
    pub nat_name: Name,
    pub zero_name: Name,
    pub succ_name: Name,
    pub rec_name: Name,
    pub fingerprint: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTacticOptions {
    pub simp_rules: Vec<SimpRuleRef>,
    pub max_simp_rewrite_steps: u64,
    pub max_open_goals: usize,
    pub max_metas: usize,
    pub eq_family: Option<EqFamilyRef>,
    pub nat_family: Option<NatFamilyRef>,
}

impl Default for MachineTacticOptions {
    fn default() -> Self {
        Self {
            simp_rules: Vec::new(),
            max_simp_rewrite_steps: 1_000,
            max_open_goals: 256,
            max_metas: 1_024,
            eq_family: None,
            nat_family: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct MachineTacticEnv {
    pub imports: Vec<VerifiedImportRef>,
    pub checked_current_decls: Vec<CheckedCurrentDecl>,
    pub simp_registry: SimpRegistry,
    pub eq_family: Option<ResolvedEqFamily>,
    pub nat_family: Option<ResolvedNatFamily>,
    pub options: MachineTacticOptions,
    pub options_fingerprint: Hash,
    pub env_fingerprint: Hash,
    kernel_env: Env,
}

pub fn check_current_decl_for_machine_tactic(
    imports: &[VerifiedModule],
    checked_prior_current_decls: &[CheckedCurrentDecl],
    source_index: u64,
    decl: Decl,
) -> Result<CheckedCurrentDecl> {
    let imports = imports
        .iter()
        .map(VerifiedImportRef::from_verified_module)
        .collect::<Result<Vec<_>>>()?;
    check_current_decl_for_machine_tactic_from_verified_imports(
        &imports,
        checked_prior_current_decls,
        source_index,
        decl,
    )
}

pub fn check_current_decl_for_machine_tactic_from_verified_imports(
    imports: &[VerifiedImportRef],
    checked_prior_current_decls: &[CheckedCurrentDecl],
    source_index: u64,
    decl: Decl,
) -> Result<CheckedCurrentDecl> {
    if source_index != checked_prior_current_decls.len() as u64 {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidCurrentDeclOrder,
            format!(
                "current declaration source_index {source_index} is not the next prefix index {}",
                checked_prior_current_decls.len()
            ),
        ));
    }
    match &decl {
        Decl::Axiom { name, .. } => {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::UncheckedCurrentDecl,
                format!(
                    "current declaration {name} is an axiom; checked current declarations must carry a kernel-checkable body"
                ),
            ));
        }
        Decl::Constructor { name, .. } | Decl::Recursor { name, .. } => {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::UncheckedCurrentDecl,
                format!(
                    "generated declaration {name} cannot be checked as a standalone current declaration"
                ),
            ));
        }
        Decl::Def { .. } | Decl::Theorem { .. } | Decl::Inductive { .. } => {}
    }

    let canonical_imports = canonicalize_imports(imports.to_vec());
    let mut env = MachineTacticEnv::new(
        canonical_imports.clone(),
        checked_prior_current_decls.to_vec(),
        MachineTacticOptions::default(),
    )?;
    add_decl_to_kernel_env(&mut env.kernel_env, decl.clone()).map_err(|err| {
        MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::UncheckedCurrentDecl,
            format!(
                "kernel rejected current declaration {}: {err:?}",
                decl.name()
            ),
        )
    })?;
    let decl_hashes =
        phase2_current_decl_hashes(&canonical_imports, checked_prior_current_decls, &decl)?;
    Ok(CheckedCurrentDecl::from_checked_parts(
        source_index,
        decl,
        decl_hashes.decl_interface_hash,
        decl_hashes.decl_certificate_hash,
        checked_current_chain_fingerprint(checked_prior_current_decls),
        checked_env_fingerprint(&canonical_imports, checked_prior_current_decls),
    ))
}

impl MachineTacticEnv {
    pub fn new(
        imports: Vec<VerifiedImportRef>,
        checked_current_decls: Vec<CheckedCurrentDecl>,
        options: MachineTacticOptions,
    ) -> Result<Self> {
        validate_options(&options)?;
        let imports = canonicalize_imports(imports);
        validate_imports(&imports)?;

        let mut kernel_env = Env::with_builtins().map_err(|err| {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::KernelRejected,
                format!("kernel rejected builtin environment: {err:?}"),
            )
        })?;
        let mut env_decl_hashes = BTreeMap::new();
        for import in &imports {
            for (decl, hash) in import
                .certified_env_decls
                .iter()
                .zip(&import.certified_env_decl_hashes)
            {
                let name = decl.name().to_owned();
                if let Some(existing_hash) = env_decl_hashes.get(&name) {
                    if *existing_hash == *hash {
                        continue;
                    }
                    return Err(MachineTacticDiagnostic::new(
                        MachineTacticDiagnosticKind::AmbiguousKernelEnvDecl,
                        format!(
                            "kernel env has multiple declarations named {name} with different hashes"
                        ),
                    ));
                }
                match add_decl_to_kernel_env(&mut kernel_env, decl.clone()) {
                    Ok(()) => {}
                    Err(npa_kernel::Error::DuplicateDecl(_))
                        if verified_builtin_decl_matches_kernel_env(&kernel_env, decl) => {}
                    Err(err) => {
                        return Err(MachineTacticDiagnostic::new(
                            MachineTacticDiagnosticKind::InvalidVerifiedImport,
                            format!(
                                "kernel env rejected import {} declaration {}: {err:?}",
                                import.module.as_dotted(),
                                decl.name()
                            ),
                        ));
                    }
                }
                env_decl_hashes.insert(name, *hash);
            }
        }

        let mut normalized_current = Vec::new();
        let mut current_names = BTreeSet::new();
        for (expected_index, checked) in checked_current_decls.into_iter().enumerate() {
            if checked.source_index != expected_index as u64 {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::InvalidCurrentDeclOrder,
                    format!(
                        "checked current declaration source_index {} is not prefix index {}",
                        checked.source_index, expected_index
                    ),
                ));
            }
            let expected_decl_hashes =
                phase2_current_decl_hashes(&imports, &normalized_current, &checked.core_decl)?;
            let expected_signature = CheckedDeclSignature::from_core_decl(
                &checked.core_decl,
                expected_decl_hashes.decl_interface_hash,
            );
            if checked.signature != expected_signature {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::CurrentDeclSignatureMismatch,
                    format!(
                        "checked current declaration {} has a stale signature",
                        checked.core_decl.name()
                    ),
                )
                .with_expected_actual_payloads(
                    DiagnosticPayloadKind::CheckedDeclSignature,
                    &checked_decl_signature_canonical_bytes(&expected_signature),
                    &checked_decl_signature_canonical_bytes(&checked.signature),
                ));
            }
            if checked.core_decl_hash != expected_decl_hashes.decl_certificate_hash {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::CurrentDeclSignatureMismatch,
                    format!(
                        "checked current declaration {} has a stale Phase 2 declaration hash",
                        checked.core_decl.name()
                    ),
                ));
            }

            let prior_chain = checked_current_chain_fingerprint(&normalized_current);
            let checked_env_fingerprint = checked_env_fingerprint(&imports, &normalized_current);
            if checked.prior_chain_fingerprint != prior_chain {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::CurrentDeclSignatureMismatch,
                    format!(
                        "checked current declaration {} has a stale prior-chain fingerprint",
                        checked.core_decl.name()
                    ),
                ));
            }
            if checked.checked_env_fingerprint != checked_env_fingerprint {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::CurrentDeclSignatureMismatch,
                    format!(
                        "checked current declaration {} has a stale checked-env fingerprint",
                        checked.core_decl.name()
                    ),
                ));
            }

            let mut normalized = checked;
            normalized.prior_chain_fingerprint = prior_chain;
            normalized.checked_env_fingerprint = checked_env_fingerprint;
            let name = normalized.core_decl.name().to_owned();
            if !current_names.insert(name.clone()) {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::InvalidCurrentDeclOrder,
                    format!("checked current declaration name {name} is duplicated"),
                ));
            }
            let hash = normalized.core_decl_hash;
            match env_decl_hashes.get(&name) {
                Some(existing_hash) if *existing_hash == hash => {}
                Some(_) => {
                    return Err(MachineTacticDiagnostic::new(
                        MachineTacticDiagnosticKind::AmbiguousKernelEnvDecl,
                        format!(
                            "kernel env has multiple declarations named {name} with different hashes"
                        ),
                    ));
                }
                None => {
                    add_decl_to_kernel_env(&mut kernel_env, normalized.core_decl.clone()).map_err(
                        |err| {
                            MachineTacticDiagnostic::new(
                                MachineTacticDiagnosticKind::UncheckedCurrentDecl,
                                format!(
                                    "kernel rejected checked current declaration {}: {err:?}",
                                    normalized.core_decl.name()
                                ),
                            )
                        },
                    )?;
                    env_decl_hashes.insert(name, hash);
                }
            }
            normalized_current.push(normalized);
        }

        let mut options = canonicalize_options(options)?;
        let eq_family = resolve_eq_family(
            &kernel_env,
            &imports,
            &normalized_current,
            options.eq_family.as_ref(),
        )?;
        let simp_registry = resolve_simp_registry(
            &kernel_env,
            &imports,
            &normalized_current,
            &eq_family,
            &options.simp_rules,
        )?;
        let nat_family = resolve_nat_family(
            &kernel_env,
            &imports,
            &normalized_current,
            options.nat_family.as_ref(),
        )?;
        options.simp_rules = simp_registry
            .rules
            .iter()
            .map(|rule| rule.key.clone())
            .collect();
        let options_fingerprint = machine_tactic_options_hash(&options);
        let mut env = Self {
            imports,
            checked_current_decls: normalized_current,
            simp_registry,
            eq_family,
            nat_family,
            options,
            options_fingerprint,
            env_fingerprint: ZERO_HASH,
            kernel_env,
        };
        env.env_fingerprint = machine_tactic_env_hash(&env);
        Ok(env)
    }

    pub fn kernel_env(&self) -> &Env {
        &self.kernel_env
    }
}

#[derive(Clone, Debug)]
pub struct MachineProofState {
    pub state_id: String,
    pub root: ProofRoot,
    pub open_goals: Vec<GoalId>,
    pub metas: MetaVarStore,
    pub env: MachineTacticEnv,
    pub reserved_local_names: Vec<String>,
    pub fingerprint: Hash,
}

impl MachineProofState {
    pub fn goal(&self, goal_id: GoalId) -> Result<MachineGoal> {
        if !self.open_goals.contains(&goal_id) {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::UnknownGoal,
                format!("goal {} is not open", goal_id.0),
            )
            .with_goal(goal_id));
        }
        let meta_id = MetaVarId::from(goal_id);
        let meta = self.metas.get(meta_id).ok_or_else(|| {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::UnknownGoal,
                format!("goal {} does not have a backing metavariable", goal_id.0),
            )
            .with_goal(goal_id)
        })?;
        if meta.goal_id != goal_id {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidMachineProofState,
                format!(
                    "goal {} is not the canonical goal for meta {}",
                    goal_id.0, meta_id.0
                ),
            )
            .with_goal(goal_id)
            .with_meta(meta_id));
        }
        if meta.assignment.is_some() {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::GoalAlreadyAssigned,
                format!("goal {} is already assigned", goal_id.0),
            )
            .with_goal(goal_id)
            .with_meta(meta_id));
        }
        Ok(MachineGoal {
            id: goal_id,
            meta_id,
            context: meta.context.clone(),
            context_hash: machine_local_context_hash(&meta.context),
            target: meta.target.clone(),
            target_hash: core_expr_hash(&meta.target),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MachineTactic {
    Exact {
        goal_id: GoalId,
        term: MachineTermSource,
    },
    Intro {
        goal_id: GoalId,
        name: String,
    },
    Apply {
        goal_id: GoalId,
        head: TacticHead,
        universe_args: Vec<Level>,
        args: Vec<ApplyArg>,
    },
    Rewrite {
        goal_id: GoalId,
        rule: RewriteRuleRef,
        direction: RewriteDirection,
        site: RewriteSite,
    },
    SimpLite {
        goal_id: GoalId,
        rules: Vec<SimpRuleRef>,
    },
    InductionNat {
        goal_id: GoalId,
        local_name: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTacticCacheKey {
    pub state_fingerprint: Hash,
    pub goal_id: GoalId,
    pub tactic_hash: Hash,
    pub deterministic_budget_hash: Hash,
}

#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum MachineTacticResult {
    Success {
        state: MachineProofState,
        delta: MachineProofDelta,
    },
    Error {
        diagnostic: MachineTacticDiagnostic,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTacticBatchCandidate {
    pub candidate_id: String,
    pub candidate: MachineTacticCandidate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineTacticBatchErrorPhase {
    CandidateValidation,
    TacticExecution,
}

#[derive(Clone, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum MachineTacticBatchItemResult {
    Success {
        candidate_id: String,
        candidate_hash: Hash,
        state: MachineProofState,
        delta: MachineProofDelta,
    },
    Error {
        candidate_id: String,
        candidate_hash: Option<Hash>,
        phase: MachineTacticBatchErrorPhase,
        diagnostic: MachineTacticDiagnostic,
        retryable: bool,
    },
}

#[derive(Clone, Debug)]
pub struct MachineTacticBatchResult {
    pub previous_state_fingerprint: Hash,
    pub deterministic_budget_hash: Hash,
    pub results: Vec<MachineTacticBatchItemResult>,
}

pub fn machine_tactic_goal_id(tactic: &MachineTactic) -> GoalId {
    match tactic {
        MachineTactic::Exact { goal_id, .. }
        | MachineTactic::Intro { goal_id, .. }
        | MachineTactic::Apply { goal_id, .. }
        | MachineTactic::Rewrite { goal_id, .. }
        | MachineTactic::SimpLite { goal_id, .. }
        | MachineTactic::InductionNat { goal_id, .. } => *goal_id,
    }
}

pub fn machine_tactic_kind(tactic: &MachineTactic) -> Option<&'static str> {
    match tactic {
        MachineTactic::Exact { .. } => Some("exact"),
        MachineTactic::Intro { .. } => Some("intro"),
        MachineTactic::Apply { .. } => Some("apply"),
        MachineTactic::Rewrite { .. } => Some("rw"),
        MachineTactic::SimpLite { .. } => Some("simp-lite"),
        MachineTactic::InductionNat { .. } => Some("induction-nat"),
    }
}

pub fn machine_tactic_candidate_kind(candidate: &MachineTacticCandidate) -> &'static str {
    match candidate {
        MachineTacticCandidate::Exact { .. } => "exact",
        MachineTacticCandidate::Intro { .. } => "intro",
        MachineTacticCandidate::Apply { .. } => "apply",
        MachineTacticCandidate::Rewrite { .. } => "rw",
        MachineTacticCandidate::SimpLite { .. } => "simp-lite",
        MachineTacticCandidate::InductionNat { .. } => "induction-nat",
    }
}

pub fn validate_machine_tactic_candidate(
    goal_id: GoalId,
    candidate: MachineTacticCandidate,
) -> Result<MachineTactic> {
    let tactic_kind = machine_tactic_candidate_kind(&candidate);
    let result = (|| -> Result<MachineTactic> {
        match candidate {
            MachineTacticCandidate::Exact { term } => Ok(MachineTactic::Exact {
                goal_id,
                term: MachineTermSource::new_checked(term.source)?,
            }),
            MachineTacticCandidate::Intro { name } => {
                validate_intro_name_shape(&name)?;
                Ok(MachineTactic::Intro { goal_id, name })
            }
            MachineTacticCandidate::Apply {
                head,
                universe_args,
                args,
            } => {
                validate_tactic_head_shape(&head)?;
                let args = args
                    .into_iter()
                    .map(validate_candidate_apply_arg)
                    .collect::<Result<Vec<_>>>()?;
                Ok(MachineTactic::Apply {
                    goal_id,
                    head,
                    universe_args,
                    args,
                })
            }
            MachineTacticCandidate::Rewrite {
                rule,
                direction,
                site,
            } => Ok(MachineTactic::Rewrite {
                goal_id,
                rule: validate_candidate_rewrite_rule(rule)?,
                direction,
                site,
            }),
            MachineTacticCandidate::SimpLite { rules } => {
                validate_simp_rule_refs(&rules)?;
                Ok(MachineTactic::SimpLite { goal_id, rules })
            }
            MachineTacticCandidate::InductionNat { local_name } => {
                validate_intro_name_shape(&local_name)?;
                Ok(MachineTactic::InductionNat {
                    goal_id,
                    local_name,
                })
            }
        }
    })();
    result.map_err(|diag| attach_tactic_context(diag, goal_id, Some(tactic_kind)))
}

pub fn start_machine_proof(
    spec: MachineProofSpec,
    imports: Vec<VerifiedImportRef>,
    checked_current_decls: Vec<CheckedCurrentDecl>,
    options: MachineTacticOptions,
) -> Result<MachineProofState> {
    validate_proof_spec(&spec)?;
    if checked_current_decls.len() as u64 != spec.source_index {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidCurrentDeclOrder,
            format!(
                "proof source_index {} requires exactly {} checked prior declarations",
                spec.source_index, spec.source_index
            ),
        ));
    }
    validate_checked_current_decls_for_module(
        &spec.module,
        &spec.theorem_name,
        &checked_current_decls,
    )?;

    let env = MachineTacticEnv::new(imports, checked_current_decls, options)?;
    ensure_type_is_sort(
        env.kernel_env(),
        &Ctx::new(),
        &spec.universe_params,
        &spec.theorem_type,
    )
    .map_err(theorem_type_diag)?;

    let m0 = MetaVarId(0);
    let g0 = GoalId::from(m0);
    let root = ProofRoot {
        module: spec.module,
        theorem_name: spec.theorem_name,
        source_index: spec.source_index,
        universe_params: spec.universe_params,
        theorem_type: spec.theorem_type.clone(),
        body: ProofExpr::Meta(m0),
    };
    let mut metas = BTreeMap::new();
    metas.insert(
        m0,
        MachineMetaVar {
            id: m0,
            goal_id: g0,
            context: Vec::new(),
            target: spec.theorem_type,
            assignment: None,
            creation_index: 0,
        },
    );
    let mut state = MachineProofState {
        state_id: String::new(),
        root,
        open_goals: vec![g0],
        metas: MetaVarStore { metas, next_id: 1 },
        env,
        reserved_local_names: Vec::new(),
        fingerprint: ZERO_HASH,
    };
    refresh_state_identity(&mut state);
    validate_machine_proof_state(&state)?;
    Ok(state)
}

pub fn run_machine_tactic(
    state: &MachineProofState,
    tactic: MachineTactic,
) -> Result<(MachineProofState, MachineProofDelta)> {
    run_machine_tactic_with_budget(state, tactic, TacticBudget::default())
}

pub fn run_machine_tactic_with_budget(
    state: &MachineProofState,
    tactic: MachineTactic,
    budget: TacticBudget,
) -> Result<(MachineProofState, MachineProofDelta)> {
    let goal_id = machine_tactic_goal_id(&tactic);
    let tactic_kind = machine_tactic_kind(&tactic);
    let result = (|| -> Result<(MachineProofState, MachineProofDelta)> {
        validate_machine_proof_state(state)?;
        match tactic {
            MachineTactic::Exact { goal_id, term } => {
                run_exact_tactic_with_budget(state, goal_id, term, budget)
            }
            MachineTactic::Intro { goal_id, name } => {
                run_intro_tactic_with_budget(state, goal_id, name, budget)
            }
            MachineTactic::Apply {
                goal_id,
                head,
                universe_args,
                args,
            } => run_apply_tactic_with_budget(state, goal_id, head, universe_args, args, budget),
            MachineTactic::Rewrite {
                goal_id,
                rule,
                direction,
                site,
            } => run_rewrite_tactic_with_budget(state, goal_id, rule, direction, site, budget),
            MachineTactic::SimpLite { goal_id, rules } => {
                run_simp_lite_tactic_with_budget(state, goal_id, rules, budget)
            }
            MachineTactic::InductionNat {
                goal_id,
                local_name,
            } => run_induction_nat_tactic_with_budget(state, goal_id, local_name, budget),
        }
    })();
    result.map_err(|diag| attach_tactic_context(diag, goal_id, tactic_kind))
}

pub fn run_machine_tactic_transactional(
    state: &MachineProofState,
    tactic: MachineTactic,
    budget: TacticBudget,
) -> MachineTacticResult {
    match run_machine_tactic_with_budget(state, tactic, budget) {
        Ok((state, delta)) => MachineTacticResult::Success { state, delta },
        Err(diagnostic) => MachineTacticResult::Error { diagnostic },
    }
}

pub fn run_machine_tactic_candidates_batch(
    state: &MachineProofState,
    goal_id: GoalId,
    candidates: Vec<MachineTacticBatchCandidate>,
    budget: TacticBudget,
    policy: MachineTacticBatchPolicy,
) -> Result<MachineTacticBatchResult> {
    validate_machine_tactic_batch_request(&candidates, policy)?;
    validate_machine_proof_state(state)?;
    state.goal(goal_id)?;

    let mut results = Vec::new();
    let mut successes = 0usize;
    let mut failures = 0usize;
    let max_evaluated = policy.max_evaluated_candidates as usize;
    let stop_after_successes = policy.stop_after_successes as usize;
    let stop_after_failures = policy.stop_after_failures as usize;

    for (evaluated, item) in candidates.into_iter().enumerate() {
        if evaluated >= max_evaluated
            || successes >= stop_after_successes
            || failures >= stop_after_failures
        {
            break;
        }

        let candidate_id = item.candidate_id;
        let tactic = match validate_machine_tactic_candidate(goal_id, item.candidate) {
            Ok(tactic) => tactic,
            Err(diagnostic) => {
                failures += 1;
                results.push(MachineTacticBatchItemResult::Error {
                    candidate_id,
                    candidate_hash: None,
                    phase: MachineTacticBatchErrorPhase::CandidateValidation,
                    diagnostic,
                    retryable: false,
                });
                continue;
            }
        };
        let candidate_hash = machine_tactic_hash(&tactic);

        match run_machine_tactic_transactional(state, tactic, budget) {
            MachineTacticResult::Success { state, delta } => {
                successes += 1;
                results.push(MachineTacticBatchItemResult::Success {
                    candidate_id,
                    candidate_hash,
                    state,
                    delta,
                });
            }
            MachineTacticResult::Error { diagnostic } => {
                failures += 1;
                results.push(MachineTacticBatchItemResult::Error {
                    candidate_id,
                    candidate_hash: Some(candidate_hash),
                    phase: MachineTacticBatchErrorPhase::TacticExecution,
                    diagnostic,
                    retryable: false,
                });
            }
        }
    }

    Ok(MachineTacticBatchResult {
        previous_state_fingerprint: state.fingerprint,
        deterministic_budget_hash: tactic_budget_hash(budget),
        results,
    })
}

fn validate_machine_tactic_batch_request(
    candidates: &[MachineTacticBatchCandidate],
    policy: MachineTacticBatchPolicy,
) -> Result<()> {
    const MAX_BATCH_CANDIDATES: usize = 256;
    if candidates.is_empty() {
        return Err(invalid_batch_policy("batch candidates must be non-empty"));
    }
    if candidates.len() > MAX_BATCH_CANDIDATES {
        return Err(invalid_batch_policy(
            "batch candidates must not exceed 256 entries",
        ));
    }

    let mut ids = BTreeSet::new();
    for item in candidates {
        if !is_machine_candidate_id(&item.candidate_id) {
            return Err(invalid_batch_policy(format!(
                "candidate_id {:?} does not match the machine batch grammar",
                item.candidate_id
            )));
        }
        if !ids.insert(item.candidate_id.as_str()) {
            return Err(invalid_batch_policy(format!(
                "duplicate candidate_id {:?}",
                item.candidate_id
            )));
        }
    }

    validate_batch_policy_field("max_evaluated_candidates", policy.max_evaluated_candidates)?;
    validate_batch_policy_field("stop_after_successes", policy.stop_after_successes)?;
    validate_batch_policy_field("stop_after_failures", policy.stop_after_failures)?;
    Ok(())
}

fn validate_batch_policy_field(name: &str, value: u32) -> Result<()> {
    if !(1..=256).contains(&value) {
        return Err(invalid_batch_policy(format!(
            "{name} must be in the inclusive range 1..=256"
        )));
    }
    Ok(())
}

fn invalid_batch_policy(message: impl Into<String>) -> MachineTacticDiagnostic {
    MachineTacticDiagnostic::new(MachineTacticDiagnosticKind::InvalidBatchPolicy, message)
}

fn run_exact_tactic_with_budget(
    state: &MachineProofState,
    goal_id: GoalId,
    term: MachineTermSource,
    budget: TacticBudget,
) -> Result<(MachineProofState, MachineProofDelta)> {
    let goal = state.goal(goal_id)?;
    validate_machine_term_source(&term)
        .map_err(|diag| attach_goal_meta(diag, goal_id, goal.meta_id))?;
    let context = machine_term_elab_context(state, &goal.context)?;
    let checked = npa_frontend::elaborate_machine_term_check(
        term.source(),
        &context,
        &goal.target,
        &npa_frontend::MachineCompileOptions::default(),
    )
    .map_err(machine_term_elaboration_diag)?;
    ensure_tactic_step_fuel(budget, 2, goal_id, goal.meta_id)?;
    let fuel = TacticRunFuel::new(budget);
    assign_goal_with_budget_and_steps(
        state,
        goal_id,
        ProofExpr::Core(checked.expr),
        Vec::new(),
        budget,
        0,
        &fuel,
    )
}

fn run_intro_tactic_with_budget(
    state: &MachineProofState,
    goal_id: GoalId,
    name: String,
    budget: TacticBudget,
) -> Result<(MachineProofState, MachineProofDelta)> {
    let goal = state.goal(goal_id)?;
    validate_intro_name_shape(&name)
        .map_err(|diag| attach_goal_meta(diag, goal_id, goal.meta_id))?;
    validate_intro_name_available(state, &goal.context, &name)
        .map_err(|diag| attach_goal_meta(diag, goal_id, goal.meta_id))?;
    let fuel = TacticRunFuel::new(budget);
    let ctx = local_context_to_ctx_with_budget(
        state.env.kernel_env(),
        &goal.context,
        &state.root.universe_params,
        &fuel,
        goal_id,
        goal.meta_id,
    )?;
    let target_whnf = kernel_whnf_with_budget(
        state.env.kernel_env(),
        &ctx,
        &state.root.universe_params,
        &goal.target,
        &fuel,
        goal_id,
        goal.meta_id,
    )?;
    let Expr::Pi { ty, body, .. } = target_whnf else {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::TypeMismatch,
            "intro requires the goal target to reduce to a Pi",
        )
        .with_expected_actual_payloads(
            DiagnosticPayloadKind::Expr,
            &core_expr_canonical_bytes(&goal.target),
            &core_expr_canonical_bytes(&target_whnf),
        )
        .with_goal(goal_id)
        .with_meta(goal.meta_id));
    };
    ensure_tactic_step_fuel(budget, 3, goal_id, goal.meta_id)?;
    let body_target = *body;
    let binder_ty = *ty;
    let new_meta_id = MetaVarId(state.metas.next_id);
    let mut body_context = goal.context.clone();
    body_context.push(MachineLocalDecl::assumption(
        name.clone(),
        binder_ty.clone(),
    ));
    let proof = ProofExpr::lam(name, binder_ty, ProofExpr::Meta(new_meta_id));
    assign_goal_with_budget_and_steps(
        state,
        goal_id,
        proof,
        vec![MachineNewGoalSpec::new(body_context, body_target)],
        budget,
        0,
        &fuel,
    )
}

fn attach_goal_meta(
    mut diag: MachineTacticDiagnostic,
    goal_id: GoalId,
    meta_id: MetaVarId,
) -> MachineTacticDiagnostic {
    if diag.goal_id.is_none() {
        diag.goal_id = Some(goal_id);
    }
    if diag.meta_id.is_none() {
        diag.meta_id = Some(meta_id);
    }
    diag
}

fn attach_tactic_context(
    mut diag: MachineTacticDiagnostic,
    goal_id: GoalId,
    tactic_kind: Option<&'static str>,
) -> MachineTacticDiagnostic {
    if diag.goal_id.is_none() {
        diag.goal_id = Some(goal_id);
    }
    if diag.tactic_kind.is_none() {
        if let Some(tactic_kind) = tactic_kind {
            diag.tactic_kind = Some(tactic_kind.to_owned());
        }
    }
    diag
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct PatternMetaId(u64);

#[derive(Clone, Debug)]
struct ResolvedApplyHead {
    proof: ProofExpr,
    ty: Expr,
}

#[derive(Clone, Debug)]
enum PendingApplyArg {
    Term {
        term: MachineTermSource,
        domain_pattern: Expr,
    },
    Subgoal {
        domain_pattern: Expr,
    },
    InferFromTarget {
        id: PatternMetaId,
        domain_pattern: Expr,
    },
}

#[derive(Clone, Debug)]
enum CheckedApplyArg {
    Core(Expr),
    Subgoal(Expr),
}

#[derive(Clone, Debug)]
struct ApplyAssembly {
    proof: ProofExpr,
    new_goal_specs: Vec<MachineNewGoalSpec>,
}

#[derive(Clone, Debug)]
struct EqTarget {
    levels: Vec<Level>,
    ty: Expr,
    lhs: Expr,
    rhs: Expr,
    expr: Expr,
}

#[derive(Clone, Debug)]
struct RewriteAssembly {
    proof: ProofExpr,
    new_goal_specs: Vec<MachineNewGoalSpec>,
    required_tactic_steps: u64,
}

#[derive(Clone, Debug)]
struct RewriteRuleInstance {
    proof: ProofExpr,
    eq_target: EqTarget,
    new_goal_specs: Vec<MachineNewGoalSpec>,
    required_tactic_steps: u64,
}

#[derive(Clone, Debug)]
struct RewriteStep {
    old_eq_target: EqTarget,
    rule_eq_target: EqTarget,
    rule_proof: ProofExpr,
    direction: RewriteDirection,
    site: RewriteSite,
}

fn run_apply_tactic_with_budget(
    state: &MachineProofState,
    goal_id: GoalId,
    head: TacticHead,
    universe_args: Vec<Level>,
    args: Vec<ApplyArg>,
    budget: TacticBudget,
) -> Result<(MachineProofState, MachineProofDelta)> {
    let goal = state.goal(goal_id)?;
    validate_tactic_head_shape(&head)
        .map_err(|diag| attach_goal_meta(diag, goal_id, goal.meta_id))?;
    for arg in &args {
        validate_apply_arg(arg).map_err(|diag| attach_goal_meta(diag, goal_id, goal.meta_id))?;
    }
    for level in &universe_args {
        ensure_level_wf(&state.root.universe_params, level).map_err(|err| {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidMachineTactic,
                format!("apply universe argument is not well formed: {err:?}"),
            )
            .with_goal(goal_id)
            .with_meta(goal.meta_id)
        })?;
    }
    validate_apply_head_static(state, &goal, &head, &universe_args)?;

    let fuel = TacticRunFuel::new(budget);
    let ctx = local_context_to_ctx_with_budget(
        state.env.kernel_env(),
        &goal.context,
        &state.root.universe_params,
        &fuel,
        goal_id,
        goal.meta_id,
    )?;
    let resolved = resolve_apply_head(state, &goal, &ctx, &head, &universe_args, &fuel)?;
    let elab_context = machine_term_elab_context(state, &goal.context)?;
    let assembly = assemble_apply(
        state,
        &goal,
        &ctx,
        &elab_context,
        resolved,
        &args,
        budget,
        &fuel,
    )?;
    assign_goal_with_budget_and_steps(
        state,
        goal_id,
        assembly.proof,
        assembly.new_goal_specs,
        budget,
        0,
        &fuel,
    )
}

fn resolve_apply_head(
    state: &MachineProofState,
    goal: &MachineGoal,
    ctx: &Ctx,
    head: &TacticHead,
    universe_args: &[Level],
    fuel: &TacticRunFuel,
) -> Result<ResolvedApplyHead> {
    match head {
        TacticHead::Local { name } => {
            resolve_local_apply_head(state, goal, ctx, name, universe_args, fuel)
        }
        TacticHead::Imported {
            name,
            decl_interface_hash,
        } => {
            let signature =
                resolve_imported_apply_signature(state, goal, name, decl_interface_hash)?;
            resolve_global_apply_head(state, goal, ctx, signature, universe_args, fuel)
        }
        TacticHead::CurrentModule {
            name,
            decl_interface_hash,
        } => {
            let signature =
                resolve_current_apply_signature(state, goal, name, decl_interface_hash)?;
            resolve_global_apply_head(state, goal, ctx, signature, universe_args, fuel)
        }
    }
}

fn validate_apply_head_static(
    state: &MachineProofState,
    goal: &MachineGoal,
    head: &TacticHead,
    universe_args: &[Level],
) -> Result<()> {
    match head {
        TacticHead::Local { name } => {
            resolve_local_apply_head_index(goal, name, universe_args)?;
        }
        TacticHead::Imported {
            name,
            decl_interface_hash,
        } => {
            let signature =
                resolve_imported_apply_signature(state, goal, name, decl_interface_hash)?;
            validate_global_apply_signature(state, goal, signature, universe_args)?;
        }
        TacticHead::CurrentModule {
            name,
            decl_interface_hash,
        } => {
            let signature =
                resolve_current_apply_signature(state, goal, name, decl_interface_hash)?;
            validate_global_apply_signature(state, goal, signature, universe_args)?;
        }
    }
    Ok(())
}

fn resolve_local_apply_head(
    state: &MachineProofState,
    goal: &MachineGoal,
    ctx: &Ctx,
    name: &str,
    universe_args: &[Level],
    fuel: &TacticRunFuel,
) -> Result<ResolvedApplyHead> {
    let index = resolve_local_apply_head_index(goal, name, universe_args)?;
    let bvar = Expr::bvar((goal.context.len() - 1 - index) as u32);
    let ty = kernel_infer_with_budget(
        state.env.kernel_env(),
        ctx,
        &state.root.universe_params,
        &bvar,
        fuel,
        goal.id,
        goal.meta_id,
    )?;
    Ok(ResolvedApplyHead {
        proof: ProofExpr::Core(bvar),
        ty,
    })
}

fn resolve_local_apply_head_index(
    goal: &MachineGoal,
    name: &str,
    universe_args: &[Level],
) -> Result<usize> {
    if !universe_args.is_empty() {
        return Err(universe_argument_mismatch_diag(
            &[],
            universe_args,
            goal.id,
            goal.meta_id,
        ));
    }
    let matches = goal
        .context
        .iter()
        .enumerate()
        .filter(|(_, local)| local.name == name)
        .collect::<Vec<_>>();
    let [(index, local)] = matches.as_slice() else {
        return Err(if matches.is_empty() {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::UnknownLocalName,
                format!("apply local head {name:?} is not in the goal context"),
            )
        } else {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::AmbiguousLocalName,
                format!("apply local head {name:?} resolves to multiple locals"),
            )
        }
        .with_goal(goal.id)
        .with_meta(goal.meta_id));
    };
    if local.value.is_some() {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidLocalHead,
            format!("apply local head {name:?} resolves to a let declaration"),
        )
        .with_goal(goal.id)
        .with_meta(goal.meta_id));
    }
    Ok(*index)
}

fn resolve_imported_apply_signature<'a>(
    state: &'a MachineProofState,
    goal: &MachineGoal,
    name: &Name,
    decl_interface_hash: &Hash,
) -> Result<&'a VerifiedExportSignature> {
    let matches = state
        .env
        .imports
        .iter()
        .flat_map(|import| import.exports.iter())
        .filter(|export| &export.name == name && &export.decl_interface_hash == decl_interface_hash)
        .collect::<Vec<_>>();
    let [signature] = matches.as_slice() else {
        return Err(if matches.is_empty() {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::UnknownTacticHead,
                format!(
                    "imported apply head {} with the requested interface hash is unknown",
                    name.as_dotted()
                ),
            )
            .with_primary_name(name.clone())
        } else {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::AmbiguousTacticHead,
                format!("imported apply head {} is ambiguous", name.as_dotted()),
            )
            .with_primary_name(name.clone())
        }
        .with_goal(goal.id)
        .with_meta(goal.meta_id));
    };
    Ok(signature)
}

fn resolve_current_apply_signature<'a>(
    state: &'a MachineProofState,
    goal: &MachineGoal,
    name: &Name,
    decl_interface_hash: &Hash,
) -> Result<&'a CheckedDeclSignature> {
    let matches = state
        .env
        .checked_current_decls
        .iter()
        .map(|decl| decl.signature())
        .filter(|signature| {
            signature.name() == name && signature.decl_interface_hash() == *decl_interface_hash
        })
        .collect::<Vec<_>>();
    let [signature] = matches.as_slice() else {
        return Err(if matches.is_empty() {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::UnknownTacticHead,
                format!(
                    "current-module apply head {} with the requested interface hash is unknown",
                    name.as_dotted()
                ),
            )
            .with_primary_name(name.clone())
        } else {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::AmbiguousTacticHead,
                format!(
                    "current-module apply head {} is ambiguous",
                    name.as_dotted()
                ),
            )
            .with_primary_name(name.clone())
        }
        .with_goal(goal.id)
        .with_meta(goal.meta_id));
    };
    Ok(signature)
}

trait ApplySignature {
    fn name(&self) -> &Name;
    fn universe_params(&self) -> &[String];
}

impl ApplySignature for VerifiedExportSignature {
    fn name(&self) -> &Name {
        &self.name
    }

    fn universe_params(&self) -> &[String] {
        // Verified export signatures do not store universe params directly. The
        // canonical kernel declaration is authoritative for arity checks below.
        &[]
    }
}

impl ApplySignature for CheckedDeclSignature {
    fn name(&self) -> &Name {
        &self.name
    }

    fn universe_params(&self) -> &[String] {
        &self.universe_params
    }
}

fn resolve_global_apply_head<S: ApplySignature>(
    state: &MachineProofState,
    goal: &MachineGoal,
    ctx: &Ctx,
    signature: &S,
    universe_args: &[Level],
    fuel: &TacticRunFuel,
) -> Result<ResolvedApplyHead> {
    validate_global_apply_signature(state, goal, signature, universe_args)?;
    let proof = Expr::konst(signature.name().as_dotted(), universe_args.to_vec());
    let ty = kernel_infer_with_budget(
        state.env.kernel_env(),
        ctx,
        &state.root.universe_params,
        &proof,
        fuel,
        goal.id,
        goal.meta_id,
    )?;
    Ok(ResolvedApplyHead {
        proof: ProofExpr::Core(proof),
        ty,
    })
}

fn validate_global_apply_signature<S: ApplySignature>(
    state: &MachineProofState,
    goal: &MachineGoal,
    signature: &S,
    universe_args: &[Level],
) -> Result<()> {
    let decl = state
        .env
        .kernel_env()
        .decl(&signature.name().as_dotted())
        .ok_or_else(|| {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::UnknownTacticHead,
                format!(
                    "apply head {} is not present in the kernel environment",
                    signature.name().as_dotted()
                ),
            )
            .with_primary_name(signature.name().clone())
            .with_goal(goal.id)
            .with_meta(goal.meta_id)
        })?;
    let universe_params = if signature.universe_params().is_empty() {
        decl.universe_params()
    } else {
        signature.universe_params()
    };
    if universe_params.len() != universe_args.len() {
        return Err(universe_argument_mismatch_diag(
            universe_params,
            universe_args,
            goal.id,
            goal.meta_id,
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn assemble_apply(
    state: &MachineProofState,
    goal: &MachineGoal,
    ctx: &Ctx,
    elab_context: &npa_frontend::MachineTermElabContext,
    resolved: ResolvedApplyHead,
    args: &[ApplyArg],
    budget: TacticBudget,
    fuel: &TacticRunFuel,
) -> Result<ApplyAssembly> {
    let env = state.env.kernel_env();
    let delta = &state.root.universe_params;
    let mut result_pattern = resolved.ty;
    let mut pending = Vec::new();
    let mut next_arg = 0;
    let mut next_pattern_meta = 0;
    let mut required_tactic_steps = 1;

    ensure_tactic_step_fuel(budget, required_tactic_steps, goal.id, goal.meta_id)?;

    loop {
        let result_whnf = kernel_whnf_with_budget(
            env,
            ctx,
            delta,
            &result_pattern,
            fuel,
            goal.id,
            goal.meta_id,
        )?;
        if !contains_pattern_meta(&result_whnf)
            && kernel_is_defeq_with_budget(
                env,
                ctx,
                delta,
                &result_whnf,
                &goal.target,
                fuel,
                goal.id,
                goal.meta_id,
            )?
        {
            if next_arg < args.len() {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::TooManyApplyArguments,
                    "apply arguments remain after the result already matches the goal target",
                )
                .with_goal(goal.id)
                .with_meta(goal.meta_id));
            }
            break;
        }

        let Expr::Pi { ty, body, .. } = result_whnf else {
            if next_arg < args.len() {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::TooManyApplyArguments,
                    "apply arguments remain after the head type stopped being a Pi",
                )
                .with_goal(goal.id)
                .with_meta(goal.meta_id));
            }
            break;
        };
        let Some(arg) = args.get(next_arg) else {
            result_pattern = Expr::pi("_", *ty, *body);
            break;
        };
        required_tactic_steps += 1;
        ensure_tactic_step_fuel(budget, required_tactic_steps, goal.id, goal.meta_id)?;
        let domain_pattern = *ty;
        result_pattern = match arg {
            ApplyArg::Term(term) => {
                let (expr, _) =
                    elaborate_apply_term_infer(term, elab_context).map_err(|mut diag| {
                        diag.goal_id = Some(goal.id);
                        diag.meta_id = Some(goal.meta_id);
                        diag
                    })?;
                let next = instantiate(&body, &expr).map_err(kernel_diag)?;
                pending.push(PendingApplyArg::Term {
                    term: term.clone(),
                    domain_pattern,
                });
                next
            }
            ApplyArg::Subgoal { .. } => {
                if !contains_pattern_meta(&domain_pattern) {
                    ensure_subgoal_domain_is_prop(
                        state,
                        ctx,
                        goal,
                        &domain_pattern,
                        Some((fuel, goal.id, goal.meta_id)),
                    )?;
                }
                if contains_bound_var(&body, 0) {
                    return Err(MachineTacticDiagnostic::new(
                        MachineTacticDiagnosticKind::InvalidMetaDependency,
                        "apply subgoal arguments cannot occur in dependent result positions",
                    )
                    .with_goal(goal.id)
                    .with_meta(goal.meta_id));
                }
                let next = instantiate(&body, &Expr::bvar(0)).map_err(kernel_diag)?;
                pending.push(PendingApplyArg::Subgoal { domain_pattern });
                next
            }
            ApplyArg::InferFromTarget => {
                let id = PatternMetaId(next_pattern_meta);
                next_pattern_meta += 1;
                let next = instantiate(&body, &pattern_meta_expr(id)).map_err(kernel_diag)?;
                pending.push(PendingApplyArg::InferFromTarget { id, domain_pattern });
                next
            }
        };
        next_arg += 1;
    }

    let infer_ids = pending
        .iter()
        .filter_map(|arg| match arg {
            PendingApplyArg::InferFromTarget { id, .. } => Some(*id),
            PendingApplyArg::Term { .. } | PendingApplyArg::Subgoal { .. } => None,
        })
        .collect::<Vec<_>>();
    let ran_infer_matcher = !infer_ids.is_empty();
    let solutions = if ran_infer_matcher {
        required_tactic_steps += 1;
        ensure_tactic_step_fuel(budget, required_tactic_steps, goal.id, goal.meta_id)?;
        infer_apply_args_from_target(state, ctx, goal, &result_pattern, &infer_ids, fuel)?
    } else {
        BTreeMap::new()
    };
    let result_after_solutions = replace_pattern_metas(&result_pattern, &solutions)?;
    if contains_pattern_meta(&result_after_solutions) {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::AmbiguousApplyArgument,
            "apply result still contains unresolved inferred arguments",
        )
        .with_goal(goal.id)
        .with_meta(goal.meta_id));
    }

    let result_whnf = kernel_whnf_with_budget(
        env,
        ctx,
        delta,
        &result_after_solutions,
        fuel,
        goal.id,
        goal.meta_id,
    )?;
    if !kernel_is_defeq_with_budget(
        env,
        ctx,
        delta,
        &result_whnf,
        &goal.target,
        fuel,
        goal.id,
        goal.meta_id,
    )? {
        return if matches!(result_whnf, Expr::Pi { .. }) {
            Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::TooFewApplyArguments,
                "apply arguments were exhausted before the result reached the goal target",
            )
            .with_goal(goal.id)
            .with_meta(goal.meta_id))
        } else {
            Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::TypeMismatch,
                "apply result type does not match the goal target",
            )
            .with_expected_actual_payloads(
                DiagnosticPayloadKind::Expr,
                &core_expr_canonical_bytes(&goal.target),
                &core_expr_canonical_bytes(&result_whnf),
            )
            .with_goal(goal.id)
            .with_meta(goal.meta_id))
        };
    }

    let mut checked_args = Vec::new();
    for pending_arg in pending {
        let checked_arg = match pending_arg {
            PendingApplyArg::Term {
                term,
                domain_pattern,
            } => {
                let domain = replace_pattern_metas(&domain_pattern, &solutions)?;
                let checked = npa_frontend::elaborate_machine_term_check(
                    term.source(),
                    elab_context,
                    &domain,
                    &npa_frontend::MachineCompileOptions::default(),
                )
                .map_err(|err| {
                    let mut diag = machine_term_elaboration_diag(err);
                    diag.goal_id = Some(goal.id);
                    diag.meta_id = Some(goal.meta_id);
                    diag
                })?;
                CheckedApplyArg::Core(checked.expr)
            }
            PendingApplyArg::Subgoal { domain_pattern } => {
                let domain = replace_pattern_metas(&domain_pattern, &solutions)?;
                ensure_subgoal_domain_is_prop(
                    state,
                    ctx,
                    goal,
                    &domain,
                    Some((fuel, goal.id, goal.meta_id)),
                )?;
                CheckedApplyArg::Subgoal(domain)
            }
            PendingApplyArg::InferFromTarget { id, domain_pattern } => {
                let domain = replace_pattern_metas(&domain_pattern, &solutions)?;
                let inferred = solutions.get(&id).ok_or_else(|| {
                    MachineTacticDiagnostic::new(
                        MachineTacticDiagnosticKind::MissingExplicitArgument,
                        "apply inferred argument was not solved by the target matcher",
                    )
                    .with_goal(goal.id)
                    .with_meta(goal.meta_id)
                })?;
                kernel_check_with_budget(
                    state.env.kernel_env(),
                    ctx,
                    delta,
                    inferred,
                    &domain,
                    fuel,
                    goal.id,
                    goal.meta_id,
                )?;
                CheckedApplyArg::Core(inferred.clone())
            }
        };
        checked_args.push(checked_arg);
    }

    required_tactic_steps += 1;
    ensure_tactic_step_fuel(budget, required_tactic_steps, goal.id, goal.meta_id)?;
    let mut proof = resolved.proof;
    let mut new_goal_specs = Vec::new();
    for checked_arg in checked_args {
        let proof_arg = match checked_arg {
            CheckedApplyArg::Core(expr) => ProofExpr::Core(expr),
            CheckedApplyArg::Subgoal(domain) => {
                let meta_id = MetaVarId(state.metas.next_id + new_goal_specs.len() as u64);
                new_goal_specs.push(MachineNewGoalSpec::new(goal.context.clone(), domain));
                ProofExpr::Meta(meta_id)
            }
        };
        proof = ProofExpr::app(proof, proof_arg);
    }

    Ok(ApplyAssembly {
        proof,
        new_goal_specs,
    })
}

fn run_rewrite_tactic_with_budget(
    state: &MachineProofState,
    goal_id: GoalId,
    rule: RewriteRuleRef,
    direction: RewriteDirection,
    site: RewriteSite,
    budget: TacticBudget,
) -> Result<(MachineProofState, MachineProofDelta)> {
    let goal = state.goal(goal_id)?;
    validate_rewrite_rule_ref(&rule)
        .map_err(|diag| attach_goal_meta(diag, goal_id, goal.meta_id))?;
    for level in &rule.universe_args {
        ensure_level_wf(&state.root.universe_params, level).map_err(|err| {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidMachineTactic,
                format!("rw universe argument is not well formed: {err:?}"),
            )
            .with_goal(goal_id)
            .with_meta(goal.meta_id)
        })?;
    }
    validate_apply_head_static(state, &goal, &rule.head, &rule.universe_args)?;
    let family = state.env.eq_family.as_ref().ok_or_else(|| {
        MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::TacticPrimitiveUnavailable,
            "rw requires a resolved Eq family",
        )
        .with_goal(goal_id)
        .with_meta(goal.meta_id)
    })?;

    let fuel = TacticRunFuel::new(budget);
    let ctx = local_context_to_ctx_with_budget(
        state.env.kernel_env(),
        &goal.context,
        &state.root.universe_params,
        &fuel,
        goal_id,
        goal.meta_id,
    )?;
    let resolved = resolve_apply_head(state, &goal, &ctx, &rule.head, &rule.universe_args, &fuel)?;
    let old_eq_target = whnf_eq_target(state, &goal, &ctx, family, &fuel)?;
    let elab_context = machine_term_elab_context(state, &goal.context)?;
    let assembly = assemble_rewrite(
        state,
        &goal,
        &ctx,
        &elab_context,
        family,
        old_eq_target,
        resolved,
        &rule.args,
        direction,
        site,
        budget,
        &fuel,
    )?;
    assign_goal_with_budget_and_steps(
        state,
        goal_id,
        assembly.proof,
        assembly.new_goal_specs,
        budget,
        assembly.required_tactic_steps,
        &fuel,
    )
}

#[allow(clippy::too_many_arguments)]
fn assemble_rewrite(
    state: &MachineProofState,
    goal: &MachineGoal,
    ctx: &Ctx,
    elab_context: &npa_frontend::MachineTermElabContext,
    family: &ResolvedEqFamily,
    old_eq_target: EqTarget,
    resolved: ResolvedApplyHead,
    args: &[ApplyArg],
    direction: RewriteDirection,
    site: RewriteSite,
    budget: TacticBudget,
    fuel: &TacticRunFuel,
) -> Result<RewriteAssembly> {
    let instance = instantiate_rewrite_rule(
        state,
        goal,
        ctx,
        elab_context,
        family,
        resolved,
        args,
        &old_eq_target,
        direction,
        site,
        budget,
        fuel,
        0,
    )?;
    let mut required_tactic_steps = instance.required_tactic_steps;

    required_tactic_steps += 1;
    ensure_tactic_step_fuel(budget, required_tactic_steps, goal.id, goal.meta_id)?;
    let new_target =
        rewrite_eq_target_expr(family, &old_eq_target, &instance.eq_target, direction, site);
    let rewritten_meta_id = MetaVarId(state.metas.next_id + instance.new_goal_specs.len() as u64);
    let rewritten_goal = MachineNewGoalSpec::new(goal.context.clone(), new_target.clone());
    let mut new_goal_specs = instance.new_goal_specs;
    new_goal_specs.push(rewritten_goal);
    ensure_rewrite_fuel(budget, 1, goal.id, goal.meta_id)?;
    required_tactic_steps += 1;
    ensure_tactic_step_fuel(budget, required_tactic_steps, goal.id, goal.meta_id)?;
    let proof = mk_rewrite_transport(
        family,
        &old_eq_target,
        &instance.eq_target,
        instance.proof,
        direction,
        site,
        ProofExpr::Meta(rewritten_meta_id),
    )?;
    Ok(RewriteAssembly {
        proof,
        new_goal_specs,
        required_tactic_steps,
    })
}

#[allow(clippy::too_many_arguments)]
fn instantiate_rewrite_rule(
    state: &MachineProofState,
    goal: &MachineGoal,
    ctx: &Ctx,
    elab_context: &npa_frontend::MachineTermElabContext,
    family: &ResolvedEqFamily,
    resolved: ResolvedApplyHead,
    args: &[ApplyArg],
    old_eq_target: &EqTarget,
    direction: RewriteDirection,
    site: RewriteSite,
    budget: TacticBudget,
    fuel: &TacticRunFuel,
    initial_tactic_steps: u64,
) -> Result<RewriteRuleInstance> {
    let env = state.env.kernel_env();
    let delta = &state.root.universe_params;
    let selected_side = match site {
        RewriteSite::EqTargetLeft => &old_eq_target.lhs,
        RewriteSite::EqTargetRight => &old_eq_target.rhs,
    };
    let mut result_pattern = resolved.ty;
    let mut pending = Vec::new();
    let mut next_arg = 0;
    let mut next_pattern_meta = 0;
    let mut required_tactic_steps = initial_tactic_steps + 1;
    ensure_tactic_step_fuel(budget, required_tactic_steps, goal.id, goal.meta_id)?;

    loop {
        let result_whnf = kernel_whnf_with_budget(
            env,
            ctx,
            delta,
            &result_pattern,
            fuel,
            goal.id,
            goal.meta_id,
        )?;
        if parse_eq_target_from_expr(family, &result_whnf).is_some() {
            if next_arg < args.len() {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::TooManyApplyArguments,
                    "rewrite arguments remain after the rule conclusion reached Eq",
                )
                .with_goal(goal.id)
                .with_meta(goal.meta_id));
            }
            result_pattern = result_whnf;
            break;
        }
        let Expr::Pi { ty, body, .. } = result_whnf else {
            return Err(ambiguous_rewrite_rule_diag(
                "rewrite rule type does not reduce to an Eq theorem",
                goal,
            ));
        };
        let Some(arg) = args.get(next_arg) else {
            return Err(ambiguous_rewrite_rule_diag(
                "rewrite rule has unsolved binders before its Eq conclusion",
                goal,
            ));
        };
        required_tactic_steps += 1;
        ensure_tactic_step_fuel(budget, required_tactic_steps, goal.id, goal.meta_id)?;
        let domain_pattern = *ty;
        result_pattern = match arg {
            ApplyArg::Term(term) => {
                let (expr, _) =
                    elaborate_apply_term_infer(term, elab_context).map_err(|mut diag| {
                        diag.goal_id = Some(goal.id);
                        diag.meta_id = Some(goal.meta_id);
                        diag
                    })?;
                pending.push(PendingApplyArg::Term {
                    term: term.clone(),
                    domain_pattern,
                });
                instantiate(&body, &expr).map_err(kernel_diag)?
            }
            ApplyArg::Subgoal { .. } => {
                if !contains_pattern_meta(&domain_pattern) {
                    ensure_subgoal_domain_is_prop(
                        state,
                        ctx,
                        goal,
                        &domain_pattern,
                        Some((fuel, goal.id, goal.meta_id)),
                    )?;
                }
                if contains_bound_var(&body, 0) {
                    return Err(MachineTacticDiagnostic::new(
                        MachineTacticDiagnosticKind::InvalidMetaDependency,
                        "rw subgoal arguments cannot occur in dependent result positions",
                    )
                    .with_goal(goal.id)
                    .with_meta(goal.meta_id));
                }
                pending.push(PendingApplyArg::Subgoal { domain_pattern });
                instantiate(&body, &Expr::bvar(0)).map_err(kernel_diag)?
            }
            ApplyArg::InferFromTarget => {
                let id = PatternMetaId(next_pattern_meta);
                next_pattern_meta += 1;
                pending.push(PendingApplyArg::InferFromTarget { id, domain_pattern });
                instantiate(&body, &pattern_meta_expr(id)).map_err(kernel_diag)?
            }
        };
        next_arg += 1;
    }

    let infer_ids = pending
        .iter()
        .filter_map(|arg| match arg {
            PendingApplyArg::InferFromTarget { id, .. } => Some(*id),
            PendingApplyArg::Term { .. } | PendingApplyArg::Subgoal { .. } => None,
        })
        .collect::<Vec<_>>();
    let mut solutions = BTreeMap::new();
    let pattern_eq = parse_eq_target_from_expr(family, &result_pattern).ok_or_else(|| {
        ambiguous_rewrite_rule_diag("rewrite rule conclusion is not an Eq theorem", goal)
    })?;
    match_apply_pattern(
        env,
        ctx,
        delta,
        &pattern_eq.ty,
        &old_eq_target.ty,
        true,
        &mut solutions,
        Some((fuel, goal.id, goal.meta_id)),
    )
    .map_err(|err| {
        rewrite_match_error_or_ambiguous(
            err,
            "rewrite rule Eq type does not match target Eq type",
            goal,
        )
    })?;
    let from_pattern = rewrite_rule_from_side(&pattern_eq, direction);
    match_apply_pattern(
        env,
        ctx,
        delta,
        from_pattern,
        selected_side,
        true,
        &mut solutions,
        Some((fuel, goal.id, goal.meta_id)),
    )
    .map_err(|err| {
        rewrite_match_error_or_ambiguous(
            err,
            "rewrite rule selected side does not match target",
            goal,
        )
    })?;
    for id in &infer_ids {
        if !solutions.contains_key(id) {
            return Err(ambiguous_rewrite_rule_diag(
                "rewrite inferred argument was not uniquely solved",
                goal,
            ));
        }
    }
    let result_after_solutions =
        replace_pattern_metas(&result_pattern, &solutions).map_err(|_| {
            ambiguous_rewrite_rule_diag(
                "rewrite rule still contains unresolved inferred arguments",
                goal,
            )
        })?;
    if contains_pattern_meta(&result_after_solutions) {
        return Err(ambiguous_rewrite_rule_diag(
            "rewrite rule still contains unresolved inferred arguments",
            goal,
        ));
    }
    let eq_target =
        parse_eq_target_from_expr(family, &result_after_solutions).ok_or_else(|| {
            ambiguous_rewrite_rule_diag("rewrite instantiated result is not an Eq theorem", goal)
        })?;
    let selected_from = rewrite_rule_from_side(&eq_target, direction);
    if !kernel_is_defeq_with_budget(
        env,
        ctx,
        delta,
        &eq_target.ty,
        &old_eq_target.ty,
        fuel,
        goal.id,
        goal.meta_id,
    )? || !kernel_is_defeq_with_budget(
        env,
        ctx,
        delta,
        selected_from,
        selected_side,
        fuel,
        goal.id,
        goal.meta_id,
    )? {
        return Err(ambiguous_rewrite_rule_diag(
            "rewrite instantiated Eq theorem is not applicable to the selected side",
            goal,
        ));
    }

    let mut checked_args = Vec::new();
    for pending_arg in pending {
        let checked_arg = match pending_arg {
            PendingApplyArg::Term {
                term,
                domain_pattern,
            } => {
                let domain = replace_pattern_metas(&domain_pattern, &solutions).map_err(|_| {
                    ambiguous_rewrite_rule_diag("rewrite term argument domain is unresolved", goal)
                })?;
                let checked = npa_frontend::elaborate_machine_term_check(
                    term.source(),
                    elab_context,
                    &domain,
                    &npa_frontend::MachineCompileOptions::default(),
                )
                .map_err(|err| {
                    let mut diag = machine_term_elaboration_diag(err);
                    diag.goal_id = Some(goal.id);
                    diag.meta_id = Some(goal.meta_id);
                    diag
                })?;
                CheckedApplyArg::Core(checked.expr)
            }
            PendingApplyArg::Subgoal { domain_pattern } => {
                let domain = replace_pattern_metas(&domain_pattern, &solutions).map_err(|_| {
                    ambiguous_rewrite_rule_diag("rewrite subgoal domain is unresolved", goal)
                })?;
                ensure_subgoal_domain_is_prop(
                    state,
                    ctx,
                    goal,
                    &domain,
                    Some((fuel, goal.id, goal.meta_id)),
                )?;
                CheckedApplyArg::Subgoal(domain)
            }
            PendingApplyArg::InferFromTarget { id, domain_pattern } => {
                let domain = replace_pattern_metas(&domain_pattern, &solutions).map_err(|_| {
                    ambiguous_rewrite_rule_diag(
                        "rewrite inferred argument domain is unresolved",
                        goal,
                    )
                })?;
                let inferred = solutions.get(&id).ok_or_else(|| {
                    ambiguous_rewrite_rule_diag("rewrite inferred argument is missing", goal)
                })?;
                kernel_check_with_budget(
                    env,
                    ctx,
                    delta,
                    inferred,
                    &domain,
                    fuel,
                    goal.id,
                    goal.meta_id,
                )?;
                CheckedApplyArg::Core(inferred.clone())
            }
        };
        checked_args.push(checked_arg);
    }

    let mut proof = resolved.proof;
    let mut new_goal_specs = Vec::new();
    for checked_arg in checked_args {
        let proof_arg = match checked_arg {
            CheckedApplyArg::Core(expr) => ProofExpr::Core(expr),
            CheckedApplyArg::Subgoal(domain) => {
                let meta_id = MetaVarId(state.metas.next_id + new_goal_specs.len() as u64);
                new_goal_specs.push(MachineNewGoalSpec::new(goal.context.clone(), domain));
                ProofExpr::Meta(meta_id)
            }
        };
        proof = ProofExpr::app(proof, proof_arg);
    }

    Ok(RewriteRuleInstance {
        proof,
        eq_target,
        new_goal_specs,
        required_tactic_steps,
    })
}

fn ambiguous_rewrite_rule_diag(message: &str, goal: &MachineGoal) -> MachineTacticDiagnostic {
    MachineTacticDiagnostic::new(MachineTacticDiagnosticKind::AmbiguousRewriteRule, message)
        .with_goal(goal.id)
        .with_meta(goal.meta_id)
}

fn rewrite_match_error_or_ambiguous(
    err: MachineTacticDiagnostic,
    message: &str,
    goal: &MachineGoal,
) -> MachineTacticDiagnostic {
    if err.kind == MachineTacticDiagnosticKind::AmbiguousApplyArgument {
        ambiguous_rewrite_rule_diag(message, goal)
    } else {
        attach_goal_meta(err, goal.id, goal.meta_id)
    }
}

fn whnf_eq_target(
    state: &MachineProofState,
    goal: &MachineGoal,
    ctx: &Ctx,
    family: &ResolvedEqFamily,
    fuel: &TacticRunFuel,
) -> Result<EqTarget> {
    let target_whnf = kernel_whnf_with_budget(
        state.env.kernel_env(),
        ctx,
        &state.root.universe_params,
        &goal.target,
        fuel,
        goal.id,
        goal.meta_id,
    )?;
    parse_eq_target_from_expr(family, &target_whnf).ok_or_else(|| {
        MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::ExpectedEqTarget,
            "rw/simp-lite requires the goal target to reduce to the resolved Eq family",
        )
        .with_expected_actual_payloads(
            DiagnosticPayloadKind::Expr,
            &core_expr_canonical_bytes(&goal.target),
            &core_expr_canonical_bytes(&target_whnf),
        )
        .with_goal(goal.id)
        .with_meta(goal.meta_id)
    })
}

fn parse_eq_target_from_expr(family: &ResolvedEqFamily, expr: &Expr) -> Option<EqTarget> {
    let (head, args) = collect_apps(expr);
    let Expr::Const { name, levels } = head else {
        return None;
    };
    if name != family.eq_name.as_dotted() || levels.len() != 1 || args.len() != 3 {
        return None;
    }
    Some(EqTarget {
        levels,
        ty: args[0].clone(),
        lhs: args[1].clone(),
        rhs: args[2].clone(),
        expr: expr.clone(),
    })
}

fn mk_eq_expr(family: &ResolvedEqFamily, levels: &[Level], ty: Expr, lhs: Expr, rhs: Expr) -> Expr {
    Expr::apps(
        Expr::konst(family.eq_name.as_dotted(), levels.to_vec()),
        vec![ty, lhs, rhs],
    )
}

fn mk_eq_refl_expr(family: &ResolvedEqFamily, levels: &[Level], ty: Expr, value: Expr) -> Expr {
    Expr::apps(
        Expr::konst(family.refl_name.as_dotted(), levels.to_vec()),
        vec![ty, value],
    )
}

fn rewrite_rule_from_side(eq_target: &EqTarget, direction: RewriteDirection) -> &Expr {
    match direction {
        RewriteDirection::Forward => &eq_target.lhs,
        RewriteDirection::Backward => &eq_target.rhs,
    }
}

fn rewrite_rule_to_side(eq_target: &EqTarget, direction: RewriteDirection) -> &Expr {
    match direction {
        RewriteDirection::Forward => &eq_target.rhs,
        RewriteDirection::Backward => &eq_target.lhs,
    }
}

fn rewrite_eq_target_expr(
    family: &ResolvedEqFamily,
    old_eq_target: &EqTarget,
    rule_eq_target: &EqTarget,
    direction: RewriteDirection,
    site: RewriteSite,
) -> Expr {
    let replacement = rewrite_rule_to_side(rule_eq_target, direction).clone();
    let (lhs, rhs) = match site {
        RewriteSite::EqTargetLeft => (replacement, old_eq_target.rhs.clone()),
        RewriteSite::EqTargetRight => (old_eq_target.lhs.clone(), replacement),
    };
    mk_eq_expr(
        family,
        &old_eq_target.levels,
        old_eq_target.ty.clone(),
        lhs,
        rhs,
    )
}

fn mk_rewrite_transport(
    family: &ResolvedEqFamily,
    old_eq_target: &EqTarget,
    rule_eq_target: &EqTarget,
    rule_proof: ProofExpr,
    direction: RewriteDirection,
    site: RewriteSite,
    p_new: ProofExpr,
) -> Result<ProofExpr> {
    let from = rewrite_rule_from_side(rule_eq_target, direction).clone();
    let to = rewrite_rule_to_side(rule_eq_target, direction).clone();
    let h_to_from = match direction {
        RewriteDirection::Forward => mk_eq_symm(family, rule_eq_target, rule_proof)?,
        RewriteDirection::Backward => rule_proof,
    };
    let motive = rewrite_motive_expr(family, old_eq_target, &to, site)?;
    let mut proof = ProofExpr::Core(Expr::konst(
        family.rec_name.as_dotted(),
        vec![old_eq_target.levels[0].clone(), Level::zero()],
    ));
    for arg in [
        ProofExpr::Core(old_eq_target.ty.clone()),
        ProofExpr::Core(to),
        ProofExpr::Core(motive),
        p_new,
        ProofExpr::Core(from),
        h_to_from,
    ] {
        proof = ProofExpr::app(proof, arg);
    }
    Ok(proof)
}

fn mk_eq_symm(
    family: &ResolvedEqFamily,
    eq_target: &EqTarget,
    proof: ProofExpr,
) -> Result<ProofExpr> {
    let level = eq_target.levels[0].clone();
    let ty_shift_1 = shift(&eq_target.ty, 1, 0).map_err(kernel_diag)?;
    let lhs_shift_1 = shift(&eq_target.lhs, 1, 0).map_err(kernel_diag)?;
    let ty_shift_2 = shift(&eq_target.ty, 2, 0).map_err(kernel_diag)?;
    let lhs_shift_2 = shift(&eq_target.lhs, 2, 0).map_err(kernel_diag)?;
    let motive = Expr::lam(
        "b",
        eq_target.ty.clone(),
        Expr::lam(
            "_",
            mk_eq_expr(
                family,
                &eq_target.levels,
                ty_shift_1,
                lhs_shift_1,
                Expr::bvar(0),
            ),
            mk_eq_expr(
                family,
                &eq_target.levels,
                ty_shift_2,
                Expr::bvar(1),
                lhs_shift_2,
            ),
        ),
    );
    let minor = mk_eq_refl_expr(
        family,
        &eq_target.levels,
        eq_target.ty.clone(),
        eq_target.lhs.clone(),
    );
    let mut symm = ProofExpr::Core(Expr::konst(
        family.rec_name.as_dotted(),
        vec![level, Level::zero()],
    ));
    for arg in [
        ProofExpr::Core(eq_target.ty.clone()),
        ProofExpr::Core(eq_target.lhs.clone()),
        ProofExpr::Core(motive),
        ProofExpr::Core(minor),
        ProofExpr::Core(eq_target.rhs.clone()),
        proof,
    ] {
        symm = ProofExpr::app(symm, arg);
    }
    Ok(symm)
}

fn rewrite_motive_expr(
    family: &ResolvedEqFamily,
    old_eq_target: &EqTarget,
    to: &Expr,
    site: RewriteSite,
) -> Result<Expr> {
    let ty_shift_1 = shift(&old_eq_target.ty, 1, 0).map_err(kernel_diag)?;
    let to_shift_1 = shift(to, 1, 0).map_err(kernel_diag)?;
    let ty_shift_2 = shift(&old_eq_target.ty, 2, 0).map_err(kernel_diag)?;
    let lhs_shift_2 = shift(&old_eq_target.lhs, 2, 0).map_err(kernel_diag)?;
    let rhs_shift_2 = shift(&old_eq_target.rhs, 2, 0).map_err(kernel_diag)?;
    let body = match site {
        RewriteSite::EqTargetLeft => mk_eq_expr(
            family,
            &old_eq_target.levels,
            ty_shift_2,
            Expr::bvar(1),
            rhs_shift_2,
        ),
        RewriteSite::EqTargetRight => mk_eq_expr(
            family,
            &old_eq_target.levels,
            ty_shift_2,
            lhs_shift_2,
            Expr::bvar(1),
        ),
    };
    Ok(Expr::lam(
        "x",
        old_eq_target.ty.clone(),
        Expr::lam(
            "_",
            mk_eq_expr(
                family,
                &old_eq_target.levels,
                ty_shift_1,
                to_shift_1,
                Expr::bvar(0),
            ),
            body,
        ),
    ))
}

fn ensure_rewrite_fuel(
    budget: TacticBudget,
    needed: u64,
    goal_id: GoalId,
    meta_id: MetaVarId,
) -> Result<()> {
    if budget.max_rewrite_steps < needed {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::Rewrite,
            },
            format!(
                "tactic requires {needed} rewrite fuel, remaining rewrite fuel is {}",
                budget.max_rewrite_steps
            ),
        )
        .with_goal(goal_id)
        .with_meta(meta_id));
    }
    Ok(())
}

fn kernel_whnf_with_budget(
    env: &Env,
    ctx: &Ctx,
    delta: &[String],
    term: &Expr,
    fuel: &TacticRunFuel,
    goal_id: GoalId,
    meta_id: MetaVarId,
) -> Result<Expr> {
    let mut remaining = fuel.whnf_steps.get();
    let result = env.whnf_with_fuel_metered(ctx, delta, term, &mut remaining);
    fuel.whnf_steps.set(remaining);
    result.map_err(|err| match err {
        npa_kernel::Error::ResourceLimit { .. } => MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::Whnf,
            },
            format!(
                "tactic exhausted WHNF fuel; remaining WHNF fuel is {}",
                fuel.whnf_steps.get()
            ),
        )
        .with_goal(goal_id)
        .with_meta(meta_id),
        err => kernel_diag(err),
    })
}

#[allow(clippy::too_many_arguments)]
fn kernel_is_defeq_with_budget(
    env: &Env,
    ctx: &Ctx,
    delta: &[String],
    lhs: &Expr,
    rhs: &Expr,
    fuel: &TacticRunFuel,
    goal_id: GoalId,
    meta_id: MetaVarId,
) -> Result<bool> {
    let mut remaining = fuel.conversion_steps.get();
    let result = env.is_defeq_with_fuel_metered(ctx, delta, lhs, rhs, &mut remaining);
    fuel.conversion_steps.set(remaining);
    result.map_err(|err| match err {
        npa_kernel::Error::ResourceLimit { .. } => MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::Conversion,
            },
            format!(
                "tactic exhausted conversion fuel; remaining conversion fuel is {}",
                fuel.conversion_steps.get()
            ),
        )
        .with_goal(goal_id)
        .with_meta(meta_id),
        err => kernel_diag(err),
    })
}

#[allow(clippy::too_many_arguments)]
fn kernel_infer_with_budget(
    env: &Env,
    ctx: &Ctx,
    delta: &[String],
    term: &Expr,
    fuel: &TacticRunFuel,
    goal_id: GoalId,
    meta_id: MetaVarId,
) -> Result<Expr> {
    let mut whnf_remaining = fuel.whnf_steps.get();
    let mut conversion_remaining = fuel.conversion_steps.get();
    let result = env.infer_with_fuel_metered(
        ctx,
        delta,
        term,
        &mut whnf_remaining,
        &mut conversion_remaining,
    );
    fuel.whnf_steps.set(whnf_remaining);
    fuel.conversion_steps.set(conversion_remaining);
    result.map_err(|err| match err {
        npa_kernel::Error::ResourceLimit { kind } => {
            kernel_resource_limit_diag(kind, fuel, goal_id, meta_id)
        }
        err => kernel_diag(err),
    })
}

#[allow(clippy::too_many_arguments)]
fn kernel_check_with_budget(
    env: &Env,
    ctx: &Ctx,
    delta: &[String],
    term: &Expr,
    expected: &Expr,
    fuel: &TacticRunFuel,
    goal_id: GoalId,
    meta_id: MetaVarId,
) -> Result<()> {
    let mut whnf_remaining = fuel.whnf_steps.get();
    let mut conversion_remaining = fuel.conversion_steps.get();
    let result = env.check_with_fuel_metered(
        ctx,
        delta,
        term,
        expected,
        &mut whnf_remaining,
        &mut conversion_remaining,
    );
    fuel.whnf_steps.set(whnf_remaining);
    fuel.conversion_steps.set(conversion_remaining);
    result.map_err(|err| match err {
        npa_kernel::Error::ResourceLimit { kind } => {
            kernel_resource_limit_diag(kind, fuel, goal_id, meta_id)
        }
        err => kernel_diag(err),
    })
}

#[allow(clippy::too_many_arguments)]
fn kernel_expect_sort_with_budget(
    env: &Env,
    ctx: &Ctx,
    delta: &[String],
    term: &Expr,
    fuel: &TacticRunFuel,
    goal_id: GoalId,
    meta_id: MetaVarId,
) -> Result<()> {
    let ty = kernel_infer_with_budget(env, ctx, delta, term, fuel, goal_id, meta_id)?;
    match kernel_whnf_with_budget(env, ctx, delta, &ty, fuel, goal_id, meta_id)? {
        Expr::Sort(_) => Ok(()),
        actual => Err(kernel_diag(npa_kernel::Error::ExpectedSort { actual })),
    }
}

fn kernel_resource_limit_diag(
    kind: ResourceLimitKind,
    fuel: &TacticRunFuel,
    goal_id: GoalId,
    meta_id: MetaVarId,
) -> MachineTacticDiagnostic {
    let (kind, label, remaining) = match kind {
        ResourceLimitKind::Whnf => (TacticFuelKind::Whnf, "WHNF", fuel.whnf_steps.get()),
        ResourceLimitKind::Conversion => (
            TacticFuelKind::Conversion,
            "conversion",
            fuel.conversion_steps.get(),
        ),
    };
    MachineTacticDiagnostic::new(
        MachineTacticDiagnosticKind::TacticFuelExhausted { kind },
        format!("tactic exhausted {label} fuel; remaining {label} fuel is {remaining}"),
    )
    .with_goal(goal_id)
    .with_meta(meta_id)
}

fn kernel_is_defeq_with_optional_budget(
    env: &Env,
    ctx: &Ctx,
    delta: &[String],
    lhs: &Expr,
    rhs: &Expr,
    budget_context: Option<KernelFuelContext<'_>>,
) -> Result<bool> {
    match budget_context {
        Some((fuel, goal_id, meta_id)) => {
            kernel_is_defeq_with_budget(env, ctx, delta, lhs, rhs, fuel, goal_id, meta_id)
        }
        None => env.is_defeq(ctx, delta, lhs, rhs).map_err(kernel_diag),
    }
}

fn fuel_to_usize(value: u64) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

fn run_simp_lite_tactic_with_budget(
    state: &MachineProofState,
    goal_id: GoalId,
    rules: Vec<SimpRuleRef>,
    budget: TacticBudget,
) -> Result<(MachineProofState, MachineProofDelta)> {
    let goal = state.goal(goal_id)?;
    validate_simp_rule_refs(&rules)
        .map_err(|diag| attach_goal_meta(diag, goal_id, goal.meta_id))?;
    let selected_rules = resolve_simp_lite_allowlist(state, &goal, rules)?;
    let family = state.env.eq_family.as_ref().ok_or_else(|| {
        MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::TacticPrimitiveUnavailable,
            "simp-lite requires a resolved Eq family",
        )
        .with_goal(goal_id)
        .with_meta(goal.meta_id)
    })?;
    let fuel = TacticRunFuel::new(budget);
    let ctx = local_context_to_ctx_with_budget(
        state.env.kernel_env(),
        &goal.context,
        &state.root.universe_params,
        &fuel,
        goal_id,
        goal.meta_id,
    )?;
    let initial_eq_target = whnf_eq_target(state, &goal, &ctx, family, &fuel)?;
    let simp = assemble_simp_lite(
        state,
        &goal,
        &ctx,
        family,
        initial_eq_target,
        &selected_rules,
        budget,
        &fuel,
    )?;
    assign_goal_with_budget_and_steps(
        state,
        goal_id,
        simp.proof,
        simp.new_goal_specs,
        budget,
        simp.required_tactic_steps,
        &fuel,
    )
}

fn resolve_simp_lite_allowlist<'a>(
    state: &'a MachineProofState,
    goal: &MachineGoal,
    rules: Vec<SimpRuleRef>,
) -> Result<Vec<&'a ResolvedSimpRule>> {
    let allowlist = canonicalize_simp_rule_refs(rules)?;
    if allowlist.is_empty() {
        return Ok(state.env.simp_registry.rules.iter().collect());
    }
    let mut selected = Vec::new();
    for rule in &allowlist {
        let matches = state
            .env
            .simp_registry
            .rules
            .iter()
            .filter(|candidate| candidate.key == *rule)
            .collect::<Vec<_>>();
        let [resolved] = matches.as_slice() else {
            return Err(if matches.is_empty() {
                MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::UnknownSimpRule,
                    format!(
                        "simp-lite rule {} with the requested interface hash is not registered",
                        rule.name.as_dotted()
                    ),
                )
                .with_primary_name(rule.name.clone())
            } else {
                MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::AmbiguousSimpRule,
                    format!("simp-lite rule {} is ambiguous", rule.name.as_dotted()),
                )
                .with_primary_name(rule.name.clone())
            }
            .with_goal(goal.id)
            .with_meta(goal.meta_id));
        };
        selected.push(*resolved);
    }
    Ok(selected)
}

#[allow(clippy::too_many_arguments)]
fn assemble_simp_lite(
    state: &MachineProofState,
    goal: &MachineGoal,
    ctx: &Ctx,
    family: &ResolvedEqFamily,
    initial_eq_target: EqTarget,
    rules: &[&ResolvedSimpRule],
    budget: TacticBudget,
    fuel: &TacticRunFuel,
) -> Result<RewriteAssembly> {
    let env = state.env.kernel_env();
    let delta = &state.root.universe_params;
    let mut current_eq = initial_eq_target;
    let mut rewrite_count = 0_u64;
    let mut steps = Vec::new();
    let mut required_tactic_steps = 0_u64;

    loop {
        required_tactic_steps += 1;
        ensure_tactic_step_fuel(budget, required_tactic_steps, goal.id, goal.meta_id)?;

        if kernel_is_defeq_with_budget(
            env,
            ctx,
            delta,
            &current_eq.lhs,
            &current_eq.rhs,
            fuel,
            goal.id,
            goal.meta_id,
        )? {
            let proof = mk_nested_rewrite_proof_from_refl(family, &current_eq, steps)?;
            return Ok(RewriteAssembly {
                proof,
                new_goal_specs: Vec::new(),
                required_tactic_steps,
            });
        }
        if rewrite_count >= state.env.options.max_simp_rewrite_steps {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::SimpStepLimitExceeded,
                "simp-lite reached max_simp_rewrite_steps before reaching a fixed point",
            )
            .with_goal(goal.id)
            .with_meta(goal.meta_id));
        }

        let current_hash = core_expr_hash(&current_eq.expr);
        let mut progress = None;
        'scan: for rule in rules {
            for site in [RewriteSite::EqTargetLeft, RewriteSite::EqTargetRight] {
                let Some(universe_args) = solve_simp_universe_args(rule, &current_eq, site) else {
                    continue;
                };
                let instance = match instantiate_simp_rule(
                    state,
                    goal,
                    ctx,
                    family,
                    rule,
                    &universe_args,
                    &current_eq,
                    rule.key.direction,
                    site,
                    budget,
                    fuel,
                    required_tactic_steps,
                ) {
                    Ok(instance) => instance,
                    Err(err) if simp_candidate_not_applicable(&err) => continue,
                    Err(err) => return Err(err),
                };
                required_tactic_steps = instance.required_tactic_steps;
                if !instance.new_goal_specs.is_empty() {
                    continue;
                }
                let new_target = rewrite_eq_target_expr(
                    family,
                    &current_eq,
                    &instance.eq_target,
                    rule.key.direction,
                    site,
                );
                let new_whnf = kernel_whnf_with_budget(
                    env,
                    ctx,
                    delta,
                    &new_target,
                    fuel,
                    goal.id,
                    goal.meta_id,
                )?;
                let Some(new_eq) = parse_eq_target_from_expr(family, &new_whnf) else {
                    continue;
                };
                if core_expr_hash(&new_eq.expr) == current_hash {
                    continue;
                }
                progress = Some((rule.key.direction, site, instance, new_eq));
                break 'scan;
            }
        }

        let Some((direction, site, instance, new_eq)) = progress else {
            if rewrite_count == 0 {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::SimpNoProgress,
                    "simp-lite could not close by Eq.refl and no registered rule made progress",
                )
                .with_goal(goal.id)
                .with_meta(goal.meta_id));
            }
            required_tactic_steps += 1;
            ensure_tactic_step_fuel(budget, required_tactic_steps, goal.id, goal.meta_id)?;
            let final_meta = MetaVarId(state.metas.next_id);
            let proof = mk_nested_rewrite_proof_from_meta(family, final_meta, steps)?;
            return Ok(RewriteAssembly {
                proof,
                new_goal_specs: vec![MachineNewGoalSpec::new(
                    goal.context.clone(),
                    current_eq.expr,
                )],
                required_tactic_steps,
            });
        };

        ensure_rewrite_fuel(budget, rewrite_count + 1, goal.id, goal.meta_id)?;
        required_tactic_steps += 1;
        ensure_tactic_step_fuel(budget, required_tactic_steps, goal.id, goal.meta_id)?;
        steps.push(RewriteStep {
            old_eq_target: current_eq,
            rule_eq_target: instance.eq_target,
            rule_proof: instance.proof,
            direction,
            site,
        });
        rewrite_count += 1;
        current_eq = new_eq;
    }
}

fn simp_candidate_not_applicable(err: &MachineTacticDiagnostic) -> bool {
    matches!(
        err.kind,
        MachineTacticDiagnosticKind::AmbiguousRewriteRule
            | MachineTacticDiagnosticKind::AmbiguousApplyArgument
            | MachineTacticDiagnosticKind::TypeMismatch
    )
}

#[derive(Clone, Debug)]
struct InductionNatAssembly {
    proof: ProofExpr,
    new_goal_specs: Vec<MachineNewGoalSpec>,
    required_tactic_steps: u64,
}

fn run_induction_nat_tactic_with_budget(
    state: &MachineProofState,
    goal_id: GoalId,
    local_name: String,
    budget: TacticBudget,
) -> Result<(MachineProofState, MachineProofDelta)> {
    let goal = state.goal(goal_id)?;
    validate_intro_name_shape(&local_name)
        .map_err(|diag| attach_goal_meta(diag, goal_id, goal.meta_id))?;
    let target = resolve_induction_nat_target(&goal, &local_name)?;
    let family = state.env.nat_family.as_ref().ok_or_else(|| {
        MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::TacticPrimitiveUnavailable,
            "induction-nat requires a resolved Nat family",
        )
        .with_goal(goal_id)
        .with_meta(goal.meta_id)
    })?;
    ensure_resolved_nat_family_heads(state, &goal, family)?;

    let fuel = TacticRunFuel::new(budget);
    let prefix_ctx = local_context_to_ctx_with_budget(
        state.env.kernel_env(),
        &goal.context[..target.index],
        &state.root.universe_params,
        &fuel,
        goal_id,
        goal.meta_id,
    )?;
    let nat_expr = nat_family_type_expr(family);
    if !kernel_is_defeq_with_budget(
        state.env.kernel_env(),
        &prefix_ctx,
        &state.root.universe_params,
        &target.local.ty,
        &nat_expr,
        &fuel,
        goal_id,
        goal.meta_id,
    )? {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidInductionTarget,
            "induction-nat target local type is not definitionally equal to the resolved Nat family",
        )
        .with_expected_actual_payloads(
            DiagnosticPayloadKind::Expr,
            &core_expr_canonical_bytes(&nat_expr),
            &core_expr_canonical_bytes(&target.local.ty),
        )
        .with_goal(goal_id)
        .with_meta(goal.meta_id));
    }
    let ctx = local_context_to_ctx_with_budget(
        state.env.kernel_env(),
        &goal.context,
        &state.root.universe_params,
        &fuel,
        goal_id,
        goal.meta_id,
    )?;
    let assembly = assemble_induction_nat(state, &goal, &ctx, family, target.index, budget, &fuel)?;
    assign_goal_with_budget_and_steps(
        state,
        goal_id,
        assembly.proof,
        assembly.new_goal_specs,
        budget,
        assembly.required_tactic_steps,
        &fuel,
    )
}

#[derive(Clone, Debug)]
struct InductionNatTarget {
    index: usize,
    local: MachineLocalDecl,
}

fn resolve_induction_nat_target(
    goal: &MachineGoal,
    local_name: &str,
) -> Result<InductionNatTarget> {
    let matches = goal
        .context
        .iter()
        .enumerate()
        .filter(|(_, local)| local.name == local_name)
        .collect::<Vec<_>>();
    let [(index, local)] = matches.as_slice() else {
        return Err(if matches.is_empty() {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::UnknownLocalName,
                format!("induction-nat target {local_name:?} is not in the goal context"),
            )
        } else {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::AmbiguousLocalName,
                format!("induction-nat target {local_name:?} resolves to multiple locals"),
            )
        }
        .with_goal(goal.id)
        .with_meta(goal.meta_id));
    };
    if local.value.is_some() {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidInductionTarget,
            format!("induction-nat target {local_name:?} resolves to a let declaration"),
        )
        .with_goal(goal.id)
        .with_meta(goal.meta_id));
    }
    if *index + 1 != goal.context.len() {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidInductionTarget,
            "induction-nat MVP requires the target local to be the last local declaration",
        )
        .with_goal(goal.id)
        .with_meta(goal.meta_id));
    }
    Ok(InductionNatTarget {
        index: *index,
        local: (*local).clone(),
    })
}

fn ensure_resolved_nat_family_heads(
    state: &MachineProofState,
    goal: &MachineGoal,
    family: &ResolvedNatFamily,
) -> Result<()> {
    for name in [
        &family.nat_name,
        &family.zero_name,
        &family.succ_name,
        &family.rec_name,
    ] {
        if state.env.kernel_env().decl(&name.as_dotted()).is_none() {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidMachineProofState,
                format!(
                    "resolved Nat family head {} is missing from the kernel environment",
                    name.as_dotted()
                ),
            )
            .with_goal(goal.id)
            .with_meta(goal.meta_id));
        }
    }
    Ok(())
}

fn assemble_induction_nat(
    state: &MachineProofState,
    goal: &MachineGoal,
    ctx: &Ctx,
    family: &ResolvedNatFamily,
    target_index: usize,
    budget: TacticBudget,
    fuel: &TacticRunFuel,
) -> Result<InductionNatAssembly> {
    ensure_tactic_step_fuel(budget, 1, goal.id, goal.meta_id)?;
    let sort_level = target_sort_level_with_budget(state, goal, ctx, fuel)?;
    let nat_expr = nat_family_type_expr(family);
    let zero_expr = nat_family_zero_expr(family);
    let (step_n_name, step_ih_name) = deterministic_induction_local_names(&goal.context);

    let motive_body = replace_induction_target_keep(&goal.target, Expr::bvar(0), 1)
        .map_err(|err| invalid_induction_target_kernel_err(goal, err))?;
    let motive = Expr::lam(step_n_name.clone(), nat_expr.clone(), motive_body);

    ensure_tactic_step_fuel(budget, 2, goal.id, goal.meta_id)?;
    let base_motive = Expr::lam(step_n_name.clone(), nat_expr.clone(), goal.target.clone());
    let base_target = Expr::app(base_motive, zero_expr);
    let ih_ty = Expr::app(
        shift(&motive, 1, 0).map_err(|err| invalid_induction_target_kernel_err(goal, err))?,
        Expr::bvar(0),
    );
    let step_target = Expr::app(
        shift(&motive, 2, 0).map_err(|err| invalid_induction_target_kernel_err(goal, err))?,
        nat_family_succ_expr(family, Expr::bvar(1)),
    );

    let base_context = goal.context[..target_index].to_vec();
    let mut step_context = goal.context.clone();
    step_context.push(MachineLocalDecl::assumption(
        step_n_name.clone(),
        nat_expr.clone(),
    ));
    step_context.push(MachineLocalDecl::assumption(
        step_ih_name.clone(),
        ih_ty.clone(),
    ));

    ensure_tactic_step_fuel(budget, 3, goal.id, goal.meta_id)?;
    let base_meta = MetaVarId(state.metas.next_id);
    let step_meta = MetaVarId(state.metas.next_id + 1);
    let step_fun = ProofExpr::lam(
        step_n_name,
        nat_expr,
        ProofExpr::lam(step_ih_name, ih_ty, ProofExpr::Meta(step_meta)),
    );
    let mut proof = ProofExpr::Core(Expr::konst(family.rec_name.as_dotted(), vec![sort_level]));
    for arg in [
        ProofExpr::Core(motive),
        ProofExpr::Meta(base_meta),
        step_fun,
        ProofExpr::Core(Expr::bvar(0)),
    ] {
        proof = ProofExpr::app(proof, arg);
    }

    Ok(InductionNatAssembly {
        proof,
        new_goal_specs: vec![
            MachineNewGoalSpec::new(base_context, base_target),
            MachineNewGoalSpec::new(step_context, step_target),
        ],
        required_tactic_steps: 3,
    })
}

fn target_sort_level_with_budget(
    state: &MachineProofState,
    goal: &MachineGoal,
    ctx: &Ctx,
    fuel: &TacticRunFuel,
) -> Result<Level> {
    let ty = kernel_infer_with_budget(
        state.env.kernel_env(),
        ctx,
        &state.root.universe_params,
        &goal.target,
        fuel,
        goal.id,
        goal.meta_id,
    )?;
    match kernel_whnf_with_budget(
        state.env.kernel_env(),
        ctx,
        &state.root.universe_params,
        &ty,
        fuel,
        goal.id,
        goal.meta_id,
    )? {
        Expr::Sort(level) => Ok(level),
        actual => Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidInductionTarget,
            "induction-nat target motive result is not a sort",
        )
        .with_expected_actual_payloads(
            DiagnosticPayloadKind::Expr,
            &core_expr_canonical_bytes(&Expr::sort(npa_kernel::type0())),
            &core_expr_canonical_bytes(&actual),
        )
        .with_goal(goal.id)
        .with_meta(goal.meta_id)),
    }
}

fn invalid_induction_target_kernel_err(
    goal: &MachineGoal,
    err: npa_kernel::Error,
) -> MachineTacticDiagnostic {
    MachineTacticDiagnostic::new(
        MachineTacticDiagnosticKind::InvalidInductionTarget,
        format!("induction-nat could not structurally extract a motive: {err:?}"),
    )
    .with_goal(goal.id)
    .with_meta(goal.meta_id)
}

fn nat_family_type_expr(family: &ResolvedNatFamily) -> Expr {
    Expr::konst(family.nat_name.as_dotted(), Vec::new())
}

fn nat_family_zero_expr(family: &ResolvedNatFamily) -> Expr {
    Expr::konst(family.zero_name.as_dotted(), Vec::new())
}

fn nat_family_succ_expr(family: &ResolvedNatFamily, arg: Expr) -> Expr {
    Expr::app(Expr::konst(family.succ_name.as_dotted(), Vec::new()), arg)
}

fn deterministic_induction_local_names(context: &[MachineLocalDecl]) -> (String, String) {
    let used = context
        .iter()
        .map(|local| local.name.as_str())
        .collect::<BTreeSet<_>>();
    for index in 0.. {
        let n = format!("n{index}");
        let ih = format!("ih{index}");
        if !used.contains(n.as_str()) && !used.contains(ih.as_str()) {
            return (n, ih);
        }
    }
    unreachable!("unbounded deterministic name search should always find a pair")
}

fn replace_induction_target_keep(
    expr: &Expr,
    replacement: Expr,
    added_after_original_context: u32,
) -> npa_kernel::Result<Expr> {
    replace_induction_target_keep_at_depth(expr, &replacement, added_after_original_context, 0)
}

fn replace_induction_target_keep_at_depth(
    expr: &Expr,
    replacement: &Expr,
    added_after_original_context: u32,
    depth: u32,
) -> npa_kernel::Result<Expr> {
    match expr {
        Expr::Sort(level) => Ok(Expr::sort(level.clone())),
        Expr::BVar(index) if *index < depth => Ok(Expr::bvar(*index)),
        Expr::BVar(index) if *index == depth => shift(replacement, depth as i32, 0),
        Expr::BVar(index) => Ok(Expr::bvar(index + added_after_original_context)),
        Expr::Const { name, levels } => Ok(Expr::konst(name.clone(), levels.clone())),
        Expr::App(fun, arg) => Ok(Expr::app(
            replace_induction_target_keep_at_depth(
                fun,
                replacement,
                added_after_original_context,
                depth,
            )?,
            replace_induction_target_keep_at_depth(
                arg,
                replacement,
                added_after_original_context,
                depth,
            )?,
        )),
        Expr::Lam { binder, ty, body } => Ok(Expr::lam(
            binder.clone(),
            replace_induction_target_keep_at_depth(
                ty,
                replacement,
                added_after_original_context,
                depth,
            )?,
            replace_induction_target_keep_at_depth(
                body,
                replacement,
                added_after_original_context,
                depth + 1,
            )?,
        )),
        Expr::Pi { binder, ty, body } => Ok(Expr::pi(
            binder.clone(),
            replace_induction_target_keep_at_depth(
                ty,
                replacement,
                added_after_original_context,
                depth,
            )?,
            replace_induction_target_keep_at_depth(
                body,
                replacement,
                added_after_original_context,
                depth + 1,
            )?,
        )),
        Expr::Let {
            binder,
            ty,
            value,
            body,
        } => Ok(Expr::let_in(
            binder.clone(),
            replace_induction_target_keep_at_depth(
                ty,
                replacement,
                added_after_original_context,
                depth,
            )?,
            replace_induction_target_keep_at_depth(
                value,
                replacement,
                added_after_original_context,
                depth,
            )?,
            replace_induction_target_keep_at_depth(
                body,
                replacement,
                added_after_original_context,
                depth + 1,
            )?,
        )),
    }
}

#[allow(clippy::too_many_arguments)]
fn instantiate_simp_rule(
    state: &MachineProofState,
    goal: &MachineGoal,
    ctx: &Ctx,
    family: &ResolvedEqFamily,
    rule: &ResolvedSimpRule,
    universe_args: &[Level],
    old_eq_target: &EqTarget,
    direction: RewriteDirection,
    site: RewriteSite,
    budget: TacticBudget,
    fuel: &TacticRunFuel,
    initial_tactic_steps: u64,
) -> Result<RewriteRuleInstance> {
    let env = state.env.kernel_env();
    let delta = &state.root.universe_params;
    let mut required_tactic_steps = initial_tactic_steps + 1;
    ensure_tactic_step_fuel(budget, required_tactic_steps, goal.id, goal.meta_id)?;

    let param_count = rule.rule_telescope.len();
    let eq_type_pattern = instantiate_simp_pattern(
        &rule.eq_type,
        param_count,
        &rule.universe_params,
        universe_args,
    )?;
    let from_pattern = instantiate_simp_pattern(
        &rule.from_pattern,
        param_count,
        &rule.universe_params,
        universe_args,
    )?;
    let mut solutions = BTreeMap::new();
    match_apply_pattern(
        env,
        ctx,
        delta,
        &eq_type_pattern,
        &old_eq_target.ty,
        true,
        &mut solutions,
        Some((fuel, goal.id, goal.meta_id)),
    )
    .map_err(|err| {
        rewrite_match_error_or_ambiguous(
            err,
            "simp rule Eq type does not match target Eq type",
            goal,
        )
    })?;
    let selected_side = match site {
        RewriteSite::EqTargetLeft => &old_eq_target.lhs,
        RewriteSite::EqTargetRight => &old_eq_target.rhs,
    };
    match_apply_pattern(
        env,
        ctx,
        delta,
        &from_pattern,
        selected_side,
        true,
        &mut solutions,
        Some((fuel, goal.id, goal.meta_id)),
    )
    .map_err(|err| {
        rewrite_match_error_or_ambiguous(err, "simp rule selected side does not match target", goal)
    })?;
    for id in 0..param_count {
        if !solutions.contains_key(&PatternMetaId(id as u64)) {
            return Err(ambiguous_rewrite_rule_diag(
                "simp rule inferred argument was not uniquely solved",
                goal,
            ));
        }
    }

    let mut proof = ProofExpr::Core(Expr::konst(
        rule.signature.name().as_dotted(),
        universe_args.to_vec(),
    ));
    for (index, param) in rule.rule_telescope.iter().enumerate() {
        let domain_pattern =
            instantiate_simp_param_domain(param, index, &rule.universe_params, universe_args)?;
        let domain = replace_pattern_metas(&domain_pattern, &solutions).map_err(|_| {
            ambiguous_rewrite_rule_diag("simp rule inferred argument domain is unresolved", goal)
        })?;
        let inferred = solutions.get(&PatternMetaId(index as u64)).ok_or_else(|| {
            ambiguous_rewrite_rule_diag("simp rule inferred argument is missing", goal)
        })?;
        kernel_check_with_budget(
            env,
            ctx,
            delta,
            inferred,
            &domain,
            fuel,
            goal.id,
            goal.meta_id,
        )?;
        proof = ProofExpr::app(proof, ProofExpr::Core(inferred.clone()));
        required_tactic_steps += 1;
        ensure_tactic_step_fuel(budget, required_tactic_steps, goal.id, goal.meta_id)?;
    }

    let levels = rule
        .eq_levels
        .iter()
        .map(|level| subst_simp_level(level, &rule.universe_params, universe_args))
        .collect::<Vec<_>>();
    let eq_type = replace_pattern_metas(&eq_type_pattern, &solutions).map_err(|_| {
        ambiguous_rewrite_rule_diag("simp rule Eq type is unresolved after matching", goal)
    })?;
    let lhs_pattern = instantiate_simp_pattern(
        &rule.theorem_lhs,
        param_count,
        &rule.universe_params,
        universe_args,
    )?;
    let rhs_pattern = instantiate_simp_pattern(
        &rule.theorem_rhs,
        param_count,
        &rule.universe_params,
        universe_args,
    )?;
    let lhs = replace_pattern_metas(&lhs_pattern, &solutions).map_err(|_| {
        ambiguous_rewrite_rule_diag("simp rule lhs is unresolved after matching", goal)
    })?;
    let rhs = replace_pattern_metas(&rhs_pattern, &solutions).map_err(|_| {
        ambiguous_rewrite_rule_diag("simp rule rhs is unresolved after matching", goal)
    })?;
    let expr = mk_eq_expr(family, &levels, eq_type.clone(), lhs.clone(), rhs.clone());
    let eq_target = EqTarget {
        levels: levels.clone(),
        ty: eq_type.clone(),
        lhs,
        rhs,
        expr,
    };
    let selected_from = rewrite_rule_from_side(&eq_target, direction);
    if !kernel_is_defeq_with_budget(
        env,
        ctx,
        delta,
        &eq_target.ty,
        &old_eq_target.ty,
        fuel,
        goal.id,
        goal.meta_id,
    )? || !kernel_is_defeq_with_budget(
        env,
        ctx,
        delta,
        selected_from,
        selected_side,
        fuel,
        goal.id,
        goal.meta_id,
    )? {
        return Err(ambiguous_rewrite_rule_diag(
            "simp instantiated Eq theorem is not applicable to the selected side",
            goal,
        ));
    }

    Ok(RewriteRuleInstance {
        proof,
        eq_target,
        new_goal_specs: Vec::new(),
        required_tactic_steps,
    })
}

fn instantiate_simp_pattern(
    expr: &Expr,
    param_count: usize,
    universe_params: &[String],
    universe_args: &[Level],
) -> Result<Expr> {
    Ok(subst_levels_expr(
        &replace_simp_param_bvars(expr, param_count, 0),
        universe_params,
        universe_args,
    ))
}

fn instantiate_simp_param_domain(
    param: &ResolvedRuleParam,
    prior_param_count: usize,
    universe_params: &[String],
    universe_args: &[Level],
) -> Result<Expr> {
    Ok(subst_levels_expr(
        &replace_simp_param_bvars(&param.ty, prior_param_count, 0),
        universe_params,
        universe_args,
    ))
}

fn replace_simp_param_bvars(expr: &Expr, param_count: usize, depth: u32) -> Expr {
    match expr {
        Expr::Sort(level) => Expr::sort(level.clone()),
        Expr::BVar(index) if *index < depth => Expr::bvar(*index),
        Expr::BVar(index) => {
            let relative = *index - depth;
            if relative < param_count as u32 {
                let id = param_count as u32 - 1 - relative;
                pattern_meta_expr(PatternMetaId(id as u64))
            } else {
                Expr::bvar(index - param_count as u32)
            }
        }
        Expr::Const { name, levels } => Expr::konst(name.clone(), levels.clone()),
        Expr::App(fun, arg) => Expr::app(
            replace_simp_param_bvars(fun, param_count, depth),
            replace_simp_param_bvars(arg, param_count, depth),
        ),
        Expr::Lam { binder, ty, body } => Expr::lam(
            binder.clone(),
            replace_simp_param_bvars(ty, param_count, depth),
            replace_simp_param_bvars(body, param_count, depth + 1),
        ),
        Expr::Pi { binder, ty, body } => Expr::pi(
            binder.clone(),
            replace_simp_param_bvars(ty, param_count, depth),
            replace_simp_param_bvars(body, param_count, depth + 1),
        ),
        Expr::Let {
            binder,
            ty,
            value,
            body,
        } => Expr::let_in(
            binder.clone(),
            replace_simp_param_bvars(ty, param_count, depth),
            replace_simp_param_bvars(value, param_count, depth),
            replace_simp_param_bvars(body, param_count, depth + 1),
        ),
    }
}

fn subst_simp_level(level: &Level, params: &[String], args: &[Level]) -> Level {
    match level {
        Level::Zero => Level::Zero,
        Level::Succ(level) => Level::succ(subst_simp_level(level, params, args)),
        Level::Max(lhs, rhs) => Level::max(
            subst_simp_level(lhs, params, args),
            subst_simp_level(rhs, params, args),
        ),
        Level::IMax(lhs, rhs) => Level::imax(
            subst_simp_level(lhs, params, args),
            subst_simp_level(rhs, params, args),
        ),
        Level::Param(name) => params
            .iter()
            .position(|param| param == name)
            .and_then(|index| args.get(index))
            .cloned()
            .unwrap_or_else(|| Level::param(name.clone())),
    }
}

fn solve_simp_universe_args(
    rule: &ResolvedSimpRule,
    current_eq: &EqTarget,
    site: RewriteSite,
) -> Option<Vec<Level>> {
    let mut solutions = BTreeMap::new();
    if !match_simp_universe_levels(
        &rule.universe_params,
        &rule.eq_levels,
        &current_eq.levels,
        &mut solutions,
    ) {
        return None;
    }
    if !match_simp_universe_expr(
        &rule.universe_params,
        &rule.eq_type,
        &current_eq.ty,
        &mut solutions,
    ) {
        return None;
    }
    let selected_side = match site {
        RewriteSite::EqTargetLeft => &current_eq.lhs,
        RewriteSite::EqTargetRight => &current_eq.rhs,
    };
    if !match_simp_universe_expr(
        &rule.universe_params,
        &rule.from_pattern,
        selected_side,
        &mut solutions,
    ) {
        return None;
    }
    rule.universe_params
        .iter()
        .map(|param| solutions.get(param).cloned())
        .collect()
}

fn match_simp_universe_expr(
    params: &[String],
    pattern: &Expr,
    target: &Expr,
    solutions: &mut BTreeMap<String, Level>,
) -> bool {
    match (pattern, target) {
        (Expr::Sort(pattern), Expr::Sort(target)) => {
            match_simp_universe_level(params, pattern, target, solutions)
        }
        (Expr::BVar(_), _) => true,
        (
            Expr::Const {
                name: pattern_name,
                levels: pattern_levels,
            },
            Expr::Const {
                name: target_name,
                levels: target_levels,
            },
        ) if pattern_name == target_name => {
            match_simp_universe_levels(params, pattern_levels, target_levels, solutions)
        }
        (Expr::App(pattern_fun, pattern_arg), Expr::App(target_fun, target_arg)) => {
            match_simp_universe_expr(params, pattern_fun, target_fun, solutions)
                && match_simp_universe_expr(params, pattern_arg, target_arg, solutions)
        }
        (
            Expr::Lam {
                ty: pattern_ty,
                body: pattern_body,
                ..
            }
            | Expr::Pi {
                ty: pattern_ty,
                body: pattern_body,
                ..
            },
            Expr::Lam {
                ty: target_ty,
                body: target_body,
                ..
            }
            | Expr::Pi {
                ty: target_ty,
                body: target_body,
                ..
            },
        ) => {
            match_simp_universe_expr(params, pattern_ty, target_ty, solutions)
                && match_simp_universe_expr(params, pattern_body, target_body, solutions)
        }
        (
            Expr::Let {
                ty: pattern_ty,
                value: pattern_value,
                body: pattern_body,
                ..
            },
            Expr::Let {
                ty: target_ty,
                value: target_value,
                body: target_body,
                ..
            },
        ) => {
            match_simp_universe_expr(params, pattern_ty, target_ty, solutions)
                && match_simp_universe_expr(params, pattern_value, target_value, solutions)
                && match_simp_universe_expr(params, pattern_body, target_body, solutions)
        }
        _ => !contains_rule_universe_param_expr(params, pattern),
    }
}

fn match_simp_universe_levels(
    params: &[String],
    pattern: &[Level],
    target: &[Level],
    solutions: &mut BTreeMap<String, Level>,
) -> bool {
    pattern.len() == target.len()
        && pattern
            .iter()
            .zip(target)
            .all(|(pattern, target)| match_simp_universe_level(params, pattern, target, solutions))
}

fn match_simp_universe_level(
    params: &[String],
    pattern: &Level,
    target: &Level,
    solutions: &mut BTreeMap<String, Level>,
) -> bool {
    match pattern {
        Level::Param(name) if params.iter().any(|param| param == name) => {
            match solutions.get(name) {
                Some(existing) => {
                    normalize_level(existing.clone()) == normalize_level(target.clone())
                }
                None => {
                    solutions.insert(name.clone(), target.clone());
                    true
                }
            }
        }
        Level::Zero => matches!(target, Level::Zero),
        Level::Succ(pattern) => {
            matches!(target, Level::Succ(target) if match_simp_universe_level(params, pattern, target, solutions))
        }
        Level::Max(pattern_lhs, pattern_rhs) => {
            matches!(target, Level::Max(target_lhs, target_rhs)
                if match_simp_universe_level(params, pattern_lhs, target_lhs, solutions)
                    && match_simp_universe_level(params, pattern_rhs, target_rhs, solutions))
        }
        Level::IMax(pattern_lhs, pattern_rhs) => {
            matches!(target, Level::IMax(target_lhs, target_rhs)
                if match_simp_universe_level(params, pattern_lhs, target_lhs, solutions)
                    && match_simp_universe_level(params, pattern_rhs, target_rhs, solutions))
        }
        Level::Param(name) => matches!(target, Level::Param(target) if target == name),
    }
}

fn contains_rule_universe_param_expr(params: &[String], expr: &Expr) -> bool {
    match expr {
        Expr::Sort(level) => contains_rule_universe_param_level(params, level),
        Expr::BVar(_) => false,
        Expr::Const { levels, .. } => levels
            .iter()
            .any(|level| contains_rule_universe_param_level(params, level)),
        Expr::App(fun, arg) => {
            contains_rule_universe_param_expr(params, fun)
                || contains_rule_universe_param_expr(params, arg)
        }
        Expr::Lam { ty, body, .. } | Expr::Pi { ty, body, .. } => {
            contains_rule_universe_param_expr(params, ty)
                || contains_rule_universe_param_expr(params, body)
        }
        Expr::Let {
            ty, value, body, ..
        } => {
            contains_rule_universe_param_expr(params, ty)
                || contains_rule_universe_param_expr(params, value)
                || contains_rule_universe_param_expr(params, body)
        }
    }
}

fn contains_rule_universe_param_level(params: &[String], level: &Level) -> bool {
    match level {
        Level::Zero => false,
        Level::Succ(level) => contains_rule_universe_param_level(params, level),
        Level::Max(lhs, rhs) | Level::IMax(lhs, rhs) => {
            contains_rule_universe_param_level(params, lhs)
                || contains_rule_universe_param_level(params, rhs)
        }
        Level::Param(name) => params.iter().any(|param| param == name),
    }
}

fn mk_nested_rewrite_proof_from_refl(
    family: &ResolvedEqFamily,
    final_eq: &EqTarget,
    steps: Vec<RewriteStep>,
) -> Result<ProofExpr> {
    let mut proof = ProofExpr::Core(mk_eq_refl_expr(
        family,
        &final_eq.levels,
        final_eq.ty.clone(),
        final_eq.lhs.clone(),
    ));
    for step in steps.into_iter().rev() {
        proof = mk_rewrite_transport(
            family,
            &step.old_eq_target,
            &step.rule_eq_target,
            step.rule_proof,
            step.direction,
            step.site,
            proof,
        )?;
    }
    Ok(proof)
}

fn mk_nested_rewrite_proof_from_meta(
    family: &ResolvedEqFamily,
    final_meta: MetaVarId,
    steps: Vec<RewriteStep>,
) -> Result<ProofExpr> {
    let mut proof = ProofExpr::Meta(final_meta);
    for step in steps.into_iter().rev() {
        proof = mk_rewrite_transport(
            family,
            &step.old_eq_target,
            &step.rule_eq_target,
            step.rule_proof,
            step.direction,
            step.site,
            proof,
        )?;
    }
    Ok(proof)
}

pub fn assign_goal(
    state: &MachineProofState,
    goal_id: GoalId,
    proof_expr: ProofExpr,
    new_goal_specs: Vec<MachineNewGoalSpec>,
) -> Result<(MachineProofState, MachineProofDelta)> {
    assign_goal_with_budget(
        state,
        goal_id,
        proof_expr,
        new_goal_specs,
        TacticBudget::default(),
    )
}

pub fn assign_goal_with_budget(
    state: &MachineProofState,
    goal_id: GoalId,
    proof_expr: ProofExpr,
    new_goal_specs: Vec<MachineNewGoalSpec>,
    budget: TacticBudget,
) -> Result<(MachineProofState, MachineProofDelta)> {
    let fuel = TacticRunFuel::new(budget);
    assign_goal_with_budget_and_steps(state, goal_id, proof_expr, new_goal_specs, budget, 1, &fuel)
}

fn assign_goal_with_budget_and_steps(
    state: &MachineProofState,
    goal_id: GoalId,
    proof_expr: ProofExpr,
    new_goal_specs: Vec<MachineNewGoalSpec>,
    budget: TacticBudget,
    required_tactic_steps: u64,
    fuel: &TacticRunFuel,
) -> Result<(MachineProofState, MachineProofDelta)> {
    validate_machine_proof_state(state)?;
    let goal_index = state
        .open_goals
        .iter()
        .position(|open_goal| *open_goal == goal_id)
        .ok_or_else(|| {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::UnknownGoal,
                format!("goal {} is not open", goal_id.0),
            )
            .with_goal(goal_id)
        })?;
    let assigned_meta_id = MetaVarId::from(goal_id);
    let assigned_meta = state.metas.get(assigned_meta_id).ok_or_else(|| {
        MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::UnknownGoal,
            format!("goal {} does not have a backing metavariable", goal_id.0),
        )
        .with_goal(goal_id)
        .with_meta(assigned_meta_id)
    })?;
    if assigned_meta.assignment.is_some() {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::GoalAlreadyAssigned,
            format!("goal {} is already assigned", goal_id.0),
        )
        .with_goal(goal_id)
        .with_meta(assigned_meta_id));
    }
    if new_goal_specs.len() as u64 > budget.max_meta_allocations {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::MetaAllocation,
            },
            format!(
                "assign_goal requested {} new metas, remaining meta allocation fuel is {}",
                new_goal_specs.len(),
                budget.max_meta_allocations
            ),
        )
        .with_goal(goal_id)
        .with_meta(assigned_meta_id));
    }
    let resulting_open_goals = state.open_goals.len() - 1 + new_goal_specs.len();
    if resulting_open_goals > state.env.options.max_open_goals {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::GoalLimitExceeded,
            format!(
                "assign_goal would leave {} open goals, limit is {}",
                resulting_open_goals, state.env.options.max_open_goals
            ),
        )
        .with_goal(goal_id)
        .with_meta(assigned_meta_id));
    }
    if state.metas.len() + new_goal_specs.len() > state.env.options.max_metas {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::MetaLimitExceeded,
            format!(
                "assign_goal would create {} metas total, limit is {}",
                state.metas.len() + new_goal_specs.len(),
                state.env.options.max_metas
            ),
        )
        .with_goal(goal_id)
        .with_meta(assigned_meta_id));
    }

    let mut new_goal_expr_nodes = 0;
    for spec in &new_goal_specs {
        if !machine_local_context_is_prefix(&assigned_meta.context, &spec.context)
            && !machine_local_context_is_prefix(&spec.context, &assigned_meta.context)
        {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidMetaContext,
                "new goal context must be comparable with the assigned goal context",
            )
            .with_goal(goal_id)
            .with_meta(assigned_meta_id));
        }
        let ctx = local_context_to_ctx_with_budget(
            state.env.kernel_env(),
            &spec.context,
            &state.root.universe_params,
            fuel,
            goal_id,
            assigned_meta_id,
        )?;
        kernel_expect_sort_with_budget(
            state.env.kernel_env(),
            &ctx,
            &state.root.universe_params,
            &spec.target,
            fuel,
            goal_id,
            assigned_meta_id,
        )?;
        new_goal_expr_nodes += core_expr_node_count(&spec.target);
    }

    ensure_tactic_step_fuel(budget, required_tactic_steps, goal_id, assigned_meta_id)?;

    let mut candidate = state.clone();
    let mut new_meta_ids = BTreeSet::new();
    let mut added_goals = Vec::new();
    let mut new_metas_delta = Vec::new();
    for spec in new_goal_specs {
        let meta_id = MetaVarId(candidate.metas.next_id);
        let new_goal_id = GoalId::from(meta_id);
        candidate.metas.next_id += 1;
        new_meta_ids.insert(meta_id);
        candidate.metas.metas.insert(
            meta_id,
            MachineMetaVar {
                id: meta_id,
                goal_id: new_goal_id,
                context: spec.context.clone(),
                target: spec.target.clone(),
                assignment: None,
                creation_index: meta_id.0,
            },
        );
        added_goals.push(new_goal_id);
        new_metas_delta.push(MachineNewMetaDelta {
            meta_id,
            goal_id: new_goal_id,
            context_hash: machine_local_context_hash(&spec.context),
            target_hash: core_expr_hash(&spec.target),
        });
    }

    ensure_expr_node_fuel(
        budget,
        proof_expr_node_count(&proof_expr) + new_goal_expr_nodes,
        goal_id,
        assigned_meta_id,
    )?;

    check_proof_expr(
        candidate.env.kernel_env(),
        &candidate.metas,
        &assigned_meta.context,
        &candidate.root.universe_params,
        &proof_expr,
        &assigned_meta.target,
        &new_meta_ids,
        assigned_meta_id,
        fuel,
    )?;

    candidate
        .metas
        .get_mut(assigned_meta_id)
        .expect("assigned meta was looked up before cloning")
        .assignment = Some(proof_expr.clone());
    candidate
        .open_goals
        .splice(goal_index..goal_index + 1, added_goals.iter().copied());
    refresh_state_identity(&mut candidate);
    validate_machine_proof_state(&candidate)?;

    let proof_expr_hash = proof_expr_hash(&proof_expr);
    let mut delta = MachineProofDelta {
        from_state_fingerprint: state.fingerprint,
        to_state_fingerprint: candidate.fingerprint,
        assigned_goal: goal_id,
        assigned_meta: assigned_meta_id,
        proof_expr_hash,
        added_goals,
        new_metas: new_metas_delta,
        delta_hash: ZERO_HASH,
    };
    delta.delta_hash = machine_proof_delta_hash(&delta);
    Ok((candidate, delta))
}

pub fn extract_closed_machine_proof(state: &MachineProofState) -> Result<Expr> {
    validate_machine_proof_state(state)?;
    if !state.open_goals.is_empty() {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::UnresolvedGoal,
            format!("proof has {} open goals", state.open_goals.len()),
        ));
    }
    let proof = expand_proof_expr(state, &state.root.body, &[], &mut BTreeSet::new())?;
    state
        .env
        .kernel_env()
        .check(
            &Ctx::new(),
            &state.root.universe_params,
            &proof,
            &state.root.theorem_type,
        )
        .map_err(|err| {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::KernelRejected,
                format!("kernel rejected extracted proof: {err:?}"),
            )
        })?;
    Ok(proof)
}

pub fn extract_closed_machine_theorem_decl(state: &MachineProofState) -> Result<Decl> {
    let proof = extract_closed_machine_proof(state)?;
    Ok(Decl::Theorem {
        name: state.root.theorem_name.as_dotted(),
        universe_params: state.root.universe_params.clone(),
        ty: state.root.theorem_type.clone(),
        proof,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineProofCertificate {
    pub core_module: CoreModule,
    pub certificate: npa_cert::ModuleCert,
    pub certificate_bytes: Vec<u8>,
    pub verified_module: VerifiedModule,
}

pub fn extract_closed_machine_core_module(state: &MachineProofState) -> Result<CoreModule> {
    let theorem = extract_closed_machine_theorem_decl(state)?;
    let mut declarations = state
        .env
        .checked_current_decls
        .iter()
        .map(|decl| decl.core_decl().clone())
        .collect::<Vec<_>>();
    declarations.push(theorem);
    Ok(CoreModule {
        name: state.root.module.clone(),
        declarations,
    })
}

pub fn extract_closed_machine_certificate(
    state: &MachineProofState,
) -> Result<MachineProofCertificate> {
    let core_module = extract_closed_machine_core_module(state)?;
    let imports = state
        .env
        .imports
        .iter()
        .map(|import| import.verified_module.as_ref().clone())
        .collect::<Vec<_>>();
    let certificate = npa_cert::build_module_cert(core_module.clone(), &imports)
        .map_err(|err| certificate_handoff_diag("build_module_cert", err))?;
    let certificate_bytes = npa_cert::encode_module_cert(&certificate)
        .map_err(|err| certificate_handoff_diag("encode_module_cert", err))?;
    let mut session = npa_cert::VerifierSession::new();
    for import in imports {
        session.register_verified_module(import);
    }
    let verified_module = npa_cert::verify_module_cert(
        &certificate_bytes,
        &mut session,
        &npa_cert::AxiomPolicy::normal(),
    )
    .map_err(|err| certificate_handoff_diag("verify_module_cert", err))?;
    Ok(MachineProofCertificate {
        core_module,
        certificate,
        certificate_bytes,
        verified_module,
    })
}

fn certificate_handoff_diag(stage: &str, err: npa_cert::CertError) -> MachineTacticDiagnostic {
    MachineTacticDiagnostic::new(
        MachineTacticDiagnosticKind::KernelRejected,
        format!("certificate handoff {stage} failed: {err:?}"),
    )
}

pub fn validate_machine_proof_state(state: &MachineProofState) -> Result<()> {
    validate_options(&state.env.options)?;
    if state.env.options_fingerprint != machine_tactic_options_hash(&state.env.options) {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidMachineProofState,
            "machine tactic options fingerprint is stale",
        ));
    }
    if state.env.env_fingerprint != machine_tactic_env_hash(&state.env) {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidMachineProofState,
            "machine tactic env fingerprint is stale",
        ));
    }
    let expected_reserved = recompute_reserved_local_names(state);
    if state.reserved_local_names != expected_reserved {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidMachineProofState,
            "reserved local names are stale",
        ));
    }
    let expected_fingerprint = machine_proof_state_hash(state);
    if state.fingerprint != expected_fingerprint {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidMachineProofState,
            "machine proof state fingerprint is stale",
        ));
    }
    let expected_state_id = hex_hash(&expected_fingerprint);
    if state.state_id != expected_state_id {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidMachineProofState,
            "machine proof state_id is stale",
        ));
    }
    for (meta_id, meta) in &state.metas.metas {
        if *meta_id != meta.id || meta.goal_id != GoalId::from(*meta_id) {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidMachineProofState,
                format!("metavariable {} has inconsistent id fields", meta_id.0),
            )
            .with_meta(*meta_id));
        }
        if meta.id.0 >= state.metas.next_id {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidMachineProofState,
                format!(
                    "metavariable {} is outside the allocated next_id {}",
                    meta.id.0, state.metas.next_id
                ),
            )
            .with_meta(*meta_id));
        }
    }
    let mut seen_open_goals = BTreeSet::new();
    for goal_id in &state.open_goals {
        if !seen_open_goals.insert(*goal_id) {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidMachineProofState,
                format!("goal {} appears more than once", goal_id.0),
            )
            .with_goal(*goal_id));
        }
        let meta_id = MetaVarId::from(*goal_id);
        let meta = state.metas.get(meta_id).ok_or_else(|| {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidMachineProofState,
                format!("open goal {} has no metavariable", goal_id.0),
            )
            .with_goal(*goal_id)
            .with_meta(meta_id)
        })?;
        if meta.assignment.is_some() {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidMachineProofState,
                format!("open goal {} is already assigned", goal_id.0),
            )
            .with_goal(*goal_id)
            .with_meta(meta_id));
        }
    }
    for meta in state.metas.metas.values() {
        let is_open = seen_open_goals.contains(&meta.goal_id);
        if meta.assignment.is_none() && !is_open {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidMachineProofState,
                format!("unassigned meta {} is missing from open goals", meta.id.0),
            )
            .with_goal(meta.goal_id)
            .with_meta(meta.id));
        }
        if meta.assignment.is_some() && is_open {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidMachineProofState,
                format!("assigned meta {} is still listed as open", meta.id.0),
            )
            .with_goal(meta.goal_id)
            .with_meta(meta.id));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn check_proof_expr(
    env: &Env,
    metas: &MetaVarStore,
    context: &[MachineLocalDecl],
    universe_params: &[String],
    expr: &ProofExpr,
    expected: &Expr,
    allowed_new_metas: &BTreeSet<MetaVarId>,
    assigning_meta: MetaVarId,
    fuel: &TacticRunFuel,
) -> Result<CheckedProofExpr> {
    match expr {
        ProofExpr::Lam { binder, ty, body } => check_lam_proof_expr(
            env,
            metas,
            context,
            universe_params,
            binder,
            ty,
            body,
            expected,
            allowed_new_metas,
            assigning_meta,
            fuel,
        ),
        ProofExpr::Let {
            binder,
            ty,
            value,
            body,
        } => check_let_proof_expr(
            env,
            metas,
            context,
            universe_params,
            binder,
            ty,
            value,
            body,
            expected,
            allowed_new_metas,
            assigning_meta,
            fuel,
        ),
        _ => {
            let checked = infer_proof_expr(
                env,
                metas,
                context,
                universe_params,
                expr,
                allowed_new_metas,
                assigning_meta,
                fuel,
                &mut BTreeSet::new(),
            )?;
            let ctx = local_context_to_ctx_with_budget(
                env,
                context,
                universe_params,
                fuel,
                GoalId::from(assigning_meta),
                assigning_meta,
            )?;
            if kernel_is_defeq_with_budget(
                env,
                &ctx,
                universe_params,
                &checked.ty,
                expected,
                fuel,
                GoalId::from(assigning_meta),
                assigning_meta,
            )? {
                Ok(checked)
            } else {
                Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::TypeMismatch,
                    "proof expression type does not match goal target",
                )
                .with_expected_actual_payloads(
                    DiagnosticPayloadKind::Expr,
                    &core_expr_canonical_bytes(expected),
                    &core_expr_canonical_bytes(&checked.ty),
                ))
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn check_lam_proof_expr(
    env: &Env,
    metas: &MetaVarStore,
    context: &[MachineLocalDecl],
    universe_params: &[String],
    binder: &str,
    ty: &Expr,
    body: &ProofExpr,
    expected: &Expr,
    allowed_new_metas: &BTreeSet<MetaVarId>,
    assigning_meta: MetaVarId,
    fuel: &TacticRunFuel,
) -> Result<CheckedProofExpr> {
    let ctx = local_context_to_ctx_with_budget(
        env,
        context,
        universe_params,
        fuel,
        GoalId::from(assigning_meta),
        assigning_meta,
    )?;
    kernel_expect_sort_with_budget(
        env,
        &ctx,
        universe_params,
        ty,
        fuel,
        GoalId::from(assigning_meta),
        assigning_meta,
    )?;
    let expected_whnf = kernel_whnf_with_budget(
        env,
        &ctx,
        universe_params,
        expected,
        fuel,
        GoalId::from(assigning_meta),
        assigning_meta,
    )?;
    let Expr::Pi {
        ty: expected_ty,
        body: expected_body,
        ..
    } = expected_whnf
    else {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::TypeMismatch,
            "lambda proof expression requires a Pi goal target",
        )
        .with_expected_actual_payloads(
            DiagnosticPayloadKind::Expr,
            &core_expr_canonical_bytes(expected),
            &core_expr_canonical_bytes(&expected_whnf),
        ));
    };
    if !kernel_is_defeq_with_budget(
        env,
        &ctx,
        universe_params,
        ty,
        &expected_ty,
        fuel,
        GoalId::from(assigning_meta),
        assigning_meta,
    )? {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::TypeMismatch,
            "lambda binder type does not match Pi domain",
        )
        .with_expected_actual_payloads(
            DiagnosticPayloadKind::Expr,
            &core_expr_canonical_bytes(&expected_ty),
            &core_expr_canonical_bytes(ty),
        ));
    }
    let mut body_context = context.to_vec();
    body_context.push(MachineLocalDecl::assumption(
        binder.to_owned(),
        *expected_ty,
    ));
    let body_checked = check_proof_expr(
        env,
        metas,
        &body_context,
        universe_params,
        body,
        &expected_body,
        allowed_new_metas,
        assigning_meta,
        fuel,
    )?;
    Ok(CheckedProofExpr {
        expr: body_checked
            .expr
            .map(|body_expr| Expr::lam(binder.to_owned(), ty.clone(), body_expr)),
        ty: expected.clone(),
    })
}

#[allow(clippy::too_many_arguments)]
fn check_let_proof_expr(
    env: &Env,
    metas: &MetaVarStore,
    context: &[MachineLocalDecl],
    universe_params: &[String],
    binder: &str,
    ty: &Expr,
    value: &ProofExpr,
    body: &ProofExpr,
    expected: &Expr,
    allowed_new_metas: &BTreeSet<MetaVarId>,
    assigning_meta: MetaVarId,
    fuel: &TacticRunFuel,
) -> Result<CheckedProofExpr> {
    let ctx = local_context_to_ctx_with_budget(
        env,
        context,
        universe_params,
        fuel,
        GoalId::from(assigning_meta),
        assigning_meta,
    )?;
    kernel_expect_sort_with_budget(
        env,
        &ctx,
        universe_params,
        ty,
        fuel,
        GoalId::from(assigning_meta),
        assigning_meta,
    )?;
    let value_checked = check_proof_expr(
        env,
        metas,
        context,
        universe_params,
        value,
        ty,
        allowed_new_metas,
        assigning_meta,
        fuel,
    )?;
    let Some(value_expr) = value_checked.expr else {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidMetaDependency,
            "let value must be a closed skeleton expression before it can extend a local context",
        ));
    };
    let mut body_context = context.to_vec();
    body_context.push(MachineLocalDecl::definition(
        binder.to_owned(),
        ty.clone(),
        value_expr.clone(),
    ));
    let body_expected = shift(expected, 1, 0).map_err(kernel_diag)?;
    let body_checked = check_proof_expr(
        env,
        metas,
        &body_context,
        universe_params,
        body,
        &body_expected,
        allowed_new_metas,
        assigning_meta,
        fuel,
    )?;
    Ok(CheckedProofExpr {
        expr: body_checked
            .expr
            .map(|body_expr| Expr::let_in(binder.to_owned(), ty.clone(), value_expr, body_expr)),
        ty: expected.clone(),
    })
}

#[derive(Clone, Debug)]
struct CheckedProofExpr {
    expr: Option<Expr>,
    ty: Expr,
}

#[allow(clippy::too_many_arguments)]
fn infer_proof_expr(
    env: &Env,
    metas: &MetaVarStore,
    context: &[MachineLocalDecl],
    universe_params: &[String],
    expr: &ProofExpr,
    allowed_new_metas: &BTreeSet<MetaVarId>,
    assigning_meta: MetaVarId,
    fuel: &TacticRunFuel,
    visiting: &mut BTreeSet<MetaVarId>,
) -> Result<CheckedProofExpr> {
    match expr {
        ProofExpr::Core(core) => {
            let ctx = local_context_to_ctx_with_budget(
                env,
                context,
                universe_params,
                fuel,
                GoalId::from(assigning_meta),
                assigning_meta,
            )?;
            let ty = kernel_infer_with_budget(
                env,
                &ctx,
                universe_params,
                core,
                fuel,
                GoalId::from(assigning_meta),
                assigning_meta,
            )?;
            Ok(CheckedProofExpr {
                expr: Some(core.clone()),
                ty,
            })
        }
        ProofExpr::Meta(meta_id) => infer_meta_ref(
            env,
            metas,
            context,
            universe_params,
            *meta_id,
            allowed_new_metas,
            assigning_meta,
            visiting,
        ),
        ProofExpr::App(fun, arg) => {
            let fun_checked = infer_proof_expr(
                env,
                metas,
                context,
                universe_params,
                fun,
                allowed_new_metas,
                assigning_meta,
                fuel,
                visiting,
            )?;
            let ctx = local_context_to_ctx_with_budget(
                env,
                context,
                universe_params,
                fuel,
                GoalId::from(assigning_meta),
                assigning_meta,
            )?;
            let fun_ty_whnf = kernel_whnf_with_budget(
                env,
                &ctx,
                universe_params,
                &fun_checked.ty,
                fuel,
                GoalId::from(assigning_meta),
                assigning_meta,
            )?;
            let Expr::Pi { ty, body, .. } = fun_ty_whnf else {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::TypeMismatch,
                    "application head is not a function",
                )
                .with_expected_actual_payloads(
                    DiagnosticPayloadKind::Expr,
                    &core_expr_canonical_bytes(&fun_checked.ty),
                    &core_expr_canonical_bytes(&fun_ty_whnf),
                ));
            };
            let arg_checked = check_proof_expr(
                env,
                metas,
                context,
                universe_params,
                arg,
                &ty,
                allowed_new_metas,
                assigning_meta,
                fuel,
            )?;
            let result_ty = if let Some(arg_expr) = &arg_checked.expr {
                instantiate(&body, arg_expr).map_err(kernel_diag)?
            } else if !contains_bound_var(&body, 0) {
                instantiate(&body, &Expr::bvar(0)).map_err(kernel_diag)?
            } else {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::InvalidMetaDependency,
                    "dependent application result type cannot be computed from an unresolved meta argument",
                ));
            };
            let expr = match (fun_checked.expr, arg_checked.expr) {
                (Some(fun_expr), Some(arg_expr)) => Some(Expr::app(fun_expr, arg_expr)),
                _ => None,
            };
            Ok(CheckedProofExpr {
                expr,
                ty: result_ty,
            })
        }
        ProofExpr::Lam { binder, ty, body } => {
            let ctx = local_context_to_ctx_with_budget(
                env,
                context,
                universe_params,
                fuel,
                GoalId::from(assigning_meta),
                assigning_meta,
            )?;
            kernel_expect_sort_with_budget(
                env,
                &ctx,
                universe_params,
                ty,
                fuel,
                GoalId::from(assigning_meta),
                assigning_meta,
            )?;
            let mut body_context = context.to_vec();
            body_context.push(MachineLocalDecl::assumption(binder.clone(), ty.clone()));
            let body_checked = infer_proof_expr(
                env,
                metas,
                &body_context,
                universe_params,
                body,
                allowed_new_metas,
                assigning_meta,
                fuel,
                visiting,
            )?;
            Ok(CheckedProofExpr {
                expr: body_checked
                    .expr
                    .map(|body_expr| Expr::lam(binder.clone(), ty.clone(), body_expr)),
                ty: Expr::pi(binder.clone(), ty.clone(), body_checked.ty),
            })
        }
        ProofExpr::Let {
            binder,
            ty,
            value,
            body,
        } => {
            let ctx = local_context_to_ctx_with_budget(
                env,
                context,
                universe_params,
                fuel,
                GoalId::from(assigning_meta),
                assigning_meta,
            )?;
            kernel_expect_sort_with_budget(
                env,
                &ctx,
                universe_params,
                ty,
                fuel,
                GoalId::from(assigning_meta),
                assigning_meta,
            )?;
            let value_checked = check_proof_expr(
                env,
                metas,
                context,
                universe_params,
                value,
                ty,
                allowed_new_metas,
                assigning_meta,
                fuel,
            )?;
            let Some(value_expr) = value_checked.expr else {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::InvalidMetaDependency,
                    "let value must be a closed skeleton expression before it can extend a local context",
                ));
            };
            let mut body_context = context.to_vec();
            body_context.push(MachineLocalDecl::definition(
                binder.clone(),
                ty.clone(),
                value_expr.clone(),
            ));
            let body_checked = infer_proof_expr(
                env,
                metas,
                &body_context,
                universe_params,
                body,
                allowed_new_metas,
                assigning_meta,
                fuel,
                visiting,
            )?;
            let result_ty = instantiate(&body_checked.ty, &value_expr).map_err(kernel_diag)?;
            Ok(CheckedProofExpr {
                expr: body_checked.expr.map(|body_expr| {
                    Expr::let_in(binder.clone(), ty.clone(), value_expr, body_expr)
                }),
                ty: result_ty,
            })
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn infer_meta_ref(
    env: &Env,
    metas: &MetaVarStore,
    context: &[MachineLocalDecl],
    universe_params: &[String],
    meta_id: MetaVarId,
    allowed_new_metas: &BTreeSet<MetaVarId>,
    assigning_meta: MetaVarId,
    visiting: &mut BTreeSet<MetaVarId>,
) -> Result<CheckedProofExpr> {
    if meta_id == assigning_meta {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidMetaDependency,
            "proof expression cannot directly reference the metavariable being assigned",
        )
        .with_meta(meta_id));
    }
    let meta = metas.get(meta_id).ok_or_else(|| {
        MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidMetaDependency,
            format!("proof expression references unknown meta {}", meta_id.0),
        )
        .with_meta(meta_id)
    })?;
    if !machine_local_context_is_prefix(&meta.context, context) {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidMetaContext,
            format!(
                "meta {} context is not a prefix of the current proof expression context",
                meta_id.0
            ),
        )
        .with_meta(meta_id));
    }
    let weakening = context.len() - meta.context.len();
    let target = shift(&meta.target, weakening as i32, 0).map_err(kernel_diag)?;
    match &meta.assignment {
        None if allowed_new_metas.contains(&meta_id) => Ok(CheckedProofExpr {
            expr: None,
            ty: target,
        }),
        None => Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidMetaDependency,
            format!(
                "proof expression references old unresolved meta {}",
                meta_id.0
            ),
        )
        .with_meta(meta_id)),
        Some(assignment) => {
            if !visiting.insert(meta_id) {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::InvalidMetaDependency,
                    format!("metavariable dependency cycle at meta {}", meta_id.0),
                )
                .with_meta(meta_id));
            }
            let expr = expand_proof_expr_maybe(
                env,
                metas,
                assignment,
                &meta.context,
                universe_params,
                allowed_new_metas,
                assigning_meta,
                visiting,
            )?
            .map(|expanded| shift(&expanded, weakening as i32, 0))
            .transpose()
            .map_err(kernel_diag)?;
            visiting.remove(&meta_id);
            Ok(CheckedProofExpr { expr, ty: target })
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn expand_proof_expr_maybe(
    env: &Env,
    metas: &MetaVarStore,
    expr: &ProofExpr,
    context: &[MachineLocalDecl],
    universe_params: &[String],
    allowed_new_metas: &BTreeSet<MetaVarId>,
    assigning_meta: MetaVarId,
    visiting: &mut BTreeSet<MetaVarId>,
) -> Result<Option<Expr>> {
    Ok(match expr {
        ProofExpr::Core(core) => Some(core.clone()),
        ProofExpr::Meta(meta_id) => {
            let checked = infer_meta_ref(
                env,
                metas,
                context,
                universe_params,
                *meta_id,
                allowed_new_metas,
                assigning_meta,
                visiting,
            )?;
            checked.expr
        }
        ProofExpr::App(fun, arg) => {
            match (
                expand_proof_expr_maybe(
                    env,
                    metas,
                    fun,
                    context,
                    universe_params,
                    allowed_new_metas,
                    assigning_meta,
                    visiting,
                )?,
                expand_proof_expr_maybe(
                    env,
                    metas,
                    arg,
                    context,
                    universe_params,
                    allowed_new_metas,
                    assigning_meta,
                    visiting,
                )?,
            ) {
                (Some(fun), Some(arg)) => Some(Expr::app(fun, arg)),
                _ => None,
            }
        }
        ProofExpr::Lam { binder, ty, body } => {
            let mut body_context = context.to_vec();
            body_context.push(MachineLocalDecl::assumption(binder.clone(), ty.clone()));
            expand_proof_expr_maybe(
                env,
                metas,
                body,
                &body_context,
                universe_params,
                allowed_new_metas,
                assigning_meta,
                visiting,
            )?
            .map(|body| Expr::lam(binder.clone(), ty.clone(), body))
        }
        ProofExpr::Let {
            binder,
            ty,
            value,
            body,
        } => {
            let value = expand_proof_expr_maybe(
                env,
                metas,
                value,
                context,
                universe_params,
                allowed_new_metas,
                assigning_meta,
                visiting,
            )?;
            let Some(value) = value else {
                return Ok(None);
            };
            let mut body_context = context.to_vec();
            body_context.push(MachineLocalDecl::definition(
                binder.clone(),
                ty.clone(),
                value.clone(),
            ));
            expand_proof_expr_maybe(
                env,
                metas,
                body,
                &body_context,
                universe_params,
                allowed_new_metas,
                assigning_meta,
                visiting,
            )?
            .map(|body| Expr::let_in(binder.clone(), ty.clone(), value, body))
        }
    })
}

fn expand_proof_expr(
    state: &MachineProofState,
    expr: &ProofExpr,
    context: &[MachineLocalDecl],
    visiting: &mut BTreeSet<MetaVarId>,
) -> Result<Expr> {
    match expr {
        ProofExpr::Core(core) => Ok(core.clone()),
        ProofExpr::Meta(meta_id) => {
            let meta = state.metas.get(*meta_id).ok_or_else(|| {
                MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::InvalidMetaDependency,
                    format!("proof references unknown meta {}", meta_id.0),
                )
                .with_meta(*meta_id)
            })?;
            if !machine_local_context_is_prefix(&meta.context, context) {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::InvalidMetaContext,
                    format!("meta {} context is not a prefix at extraction", meta_id.0),
                )
                .with_meta(*meta_id));
            }
            let assignment = meta.assignment.as_ref().ok_or_else(|| {
                MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::UnresolvedGoal,
                    format!("meta {} is unresolved", meta_id.0),
                )
                .with_goal(meta.goal_id)
                .with_meta(*meta_id)
            })?;
            if !visiting.insert(*meta_id) {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::InvalidMetaDependency,
                    format!("metavariable dependency cycle at meta {}", meta_id.0),
                )
                .with_meta(*meta_id));
            }
            let expanded = expand_proof_expr(state, assignment, &meta.context, visiting)?;
            visiting.remove(meta_id);
            let weakening = context.len() - meta.context.len();
            shift(&expanded, weakening as i32, 0).map_err(kernel_diag)
        }
        ProofExpr::App(fun, arg) => Ok(Expr::app(
            expand_proof_expr(state, fun, context, visiting)?,
            expand_proof_expr(state, arg, context, visiting)?,
        )),
        ProofExpr::Lam { binder, ty, body } => {
            let mut body_context = context.to_vec();
            body_context.push(MachineLocalDecl::assumption(binder.clone(), ty.clone()));
            Ok(Expr::lam(
                binder.clone(),
                ty.clone(),
                expand_proof_expr(state, body, &body_context, visiting)?,
            ))
        }
        ProofExpr::Let {
            binder,
            ty,
            value,
            body,
        } => {
            let value_expr = expand_proof_expr(state, value, context, visiting)?;
            let mut body_context = context.to_vec();
            body_context.push(MachineLocalDecl::definition(
                binder.clone(),
                ty.clone(),
                value_expr.clone(),
            ));
            Ok(Expr::let_in(
                binder.clone(),
                ty.clone(),
                value_expr,
                expand_proof_expr(state, body, &body_context, visiting)?,
            ))
        }
    }
}

fn refresh_state_identity(state: &mut MachineProofState) {
    state.reserved_local_names = recompute_reserved_local_names(state);
    state.fingerprint = machine_proof_state_hash(state);
    state.state_id = hex_hash(&state.fingerprint);
}

fn recompute_reserved_local_names(state: &MachineProofState) -> Vec<String> {
    let mut names = BTreeSet::new();
    collect_proof_expr_binders(&state.root.body, &mut names);
    for meta in state.metas.metas.values() {
        for local in &meta.context {
            names.insert(local.name.clone());
        }
        if let Some(assignment) = &meta.assignment {
            collect_proof_expr_binders(assignment, &mut names);
        }
    }
    names.into_iter().collect()
}

fn collect_proof_expr_binders(expr: &ProofExpr, names: &mut BTreeSet<String>) {
    match expr {
        ProofExpr::Core(core) => collect_core_binders(core, names),
        ProofExpr::Meta(_) => {}
        ProofExpr::App(fun, arg) => {
            collect_proof_expr_binders(fun, names);
            collect_proof_expr_binders(arg, names);
        }
        ProofExpr::Lam { binder, ty, body } => {
            names.insert(binder.clone());
            collect_core_binders(ty, names);
            collect_proof_expr_binders(body, names);
        }
        ProofExpr::Let {
            binder,
            ty,
            value,
            body,
        } => {
            names.insert(binder.clone());
            collect_core_binders(ty, names);
            collect_proof_expr_binders(value, names);
            collect_proof_expr_binders(body, names);
        }
    }
}

fn collect_core_binders(expr: &Expr, names: &mut BTreeSet<String>) {
    match expr {
        Expr::Sort(_) | Expr::BVar(_) | Expr::Const { .. } => {}
        Expr::App(fun, arg) => {
            collect_core_binders(fun, names);
            collect_core_binders(arg, names);
        }
        Expr::Lam { binder, ty, body } | Expr::Pi { binder, ty, body } => {
            names.insert(binder.clone());
            collect_core_binders(ty, names);
            collect_core_binders(body, names);
        }
        Expr::Let {
            binder,
            ty,
            value,
            body,
        } => {
            names.insert(binder.clone());
            collect_core_binders(ty, names);
            collect_core_binders(value, names);
            collect_core_binders(body, names);
        }
    }
}

fn validate_options(options: &MachineTacticOptions) -> Result<()> {
    validate_simp_rule_refs(&options.simp_rules)?;
    if options.max_simp_rewrite_steps == 0 {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidTacticOption,
            "max_simp_rewrite_steps must be nonzero",
        ));
    }
    if options.max_open_goals == 0 {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidTacticOption,
            "max_open_goals must be nonzero",
        ));
    }
    if options.max_metas == 0 {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidTacticOption,
            "max_metas must be nonzero",
        ));
    }
    Ok(())
}

fn canonicalize_options(mut options: MachineTacticOptions) -> Result<MachineTacticOptions> {
    validate_options(&options)?;
    options.simp_rules = canonicalize_simp_rule_refs(options.simp_rules)?;
    Ok(options)
}

fn validate_simp_rule_refs(rules: &[SimpRuleRef]) -> Result<()> {
    for rule in rules {
        if !rule.name.is_canonical() {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidMachineTactic,
                format!("simp rule name {} is not canonical", rule.name.as_dotted()),
            ));
        }
    }
    Ok(())
}

fn canonicalize_simp_rule_refs(rules: Vec<SimpRuleRef>) -> Result<Vec<SimpRuleRef>> {
    validate_simp_rule_refs(&rules)?;
    let mut keyed = BTreeMap::new();
    for rule in rules {
        keyed.entry(SimpRuleKey::from(&rule)).or_insert(rule);
    }
    Ok(keyed.into_values().collect())
}

fn validate_candidate_rewrite_rule(rule: CandidateRewriteRuleRef) -> Result<RewriteRuleRef> {
    validate_tactic_head_shape(&rule.head)?;
    let args = rule
        .args
        .into_iter()
        .map(validate_candidate_apply_arg)
        .collect::<Result<Vec<_>>>()?;
    Ok(RewriteRuleRef {
        head: rule.head,
        universe_args: rule.universe_args,
        args,
    })
}

fn validate_rewrite_rule_ref(rule: &RewriteRuleRef) -> Result<()> {
    validate_tactic_head_shape(&rule.head)?;
    for arg in &rule.args {
        validate_apply_arg(arg)?;
    }
    Ok(())
}

fn resolve_eq_family(
    kernel_env: &Env,
    imports: &[VerifiedImportRef],
    current: &[CheckedCurrentDecl],
    family_ref: Option<&EqFamilyRef>,
) -> Result<Option<ResolvedEqFamily>> {
    match family_ref {
        None => {
            if kernel_env.decl("Eq").is_some()
                && kernel_env.decl("Eq.refl").is_some()
                && kernel_env.decl("Eq.rec").is_some()
            {
                let mut out = tagged_bytes("npa.phase4.resolved-eq-family.builtin.v1");
                encode_string_to(&mut out, "builtin-nat-eq-rec-v0.1");
                Ok(Some(ResolvedEqFamily {
                    eq_name: Name::from_dotted("Eq"),
                    refl_name: Name::from_dotted("Eq.refl"),
                    rec_name: Name::from_dotted("Eq.rec"),
                    fingerprint: hash_with_domain("npa.phase4.resolved-eq-family.v1", &out),
                }))
            } else {
                Ok(None)
            }
        }
        Some(reference) => {
            let eq = resolve_family_signature(
                kernel_env,
                imports,
                current,
                &reference.eq_name,
                &reference.eq_interface_hash,
            )?;
            let refl = resolve_family_signature(
                kernel_env,
                imports,
                current,
                &reference.refl_name,
                &reference.refl_interface_hash,
            )?;
            let rec = resolve_family_signature(
                kernel_env,
                imports,
                current,
                &reference.rec_name,
                &reference.rec_interface_hash,
            )?;
            check_eq_family_interfaces(kernel_env, &eq, &refl, &rec)?;
            if eq.origin != refl.origin || eq.origin != rec.origin {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::InvalidEqFamily,
                    "Eq family heads must resolve to the same verified import or checked current declaration",
                ));
            }
            let mut out = tagged_bytes("npa.phase4.resolved-eq-family.decl.v1");
            encode_family_origin_to(&mut out, &eq.origin);
            encode_checked_decl_signature_to(&mut out, &eq.signature);
            encode_checked_decl_signature_to(&mut out, &refl.signature);
            encode_checked_decl_signature_to(&mut out, &rec.signature);
            Ok(Some(ResolvedEqFamily {
                eq_name: reference.eq_name.clone(),
                refl_name: reference.refl_name.clone(),
                rec_name: reference.rec_name.clone(),
                fingerprint: hash_with_domain("npa.phase4.resolved-eq-family.v1", &out),
            }))
        }
    }
}

#[derive(Clone, Debug)]
struct ResolvedFamilySignature {
    signature: CheckedDeclSignature,
    origin: FamilyOrigin,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum FamilyOrigin {
    Imported {
        module: ModuleName,
        export_hash: Hash,
        certificate_hash: Hash,
    },
    CurrentSourceDecl {
        module: ModuleName,
        source_index: u64,
        core_decl_hash: Hash,
    },
}

fn resolve_family_signature(
    kernel_env: &Env,
    imports: &[VerifiedImportRef],
    current: &[CheckedCurrentDecl],
    name: &Name,
    decl_interface_hash: &Hash,
) -> Result<ResolvedFamilySignature> {
    let mut matches = Vec::new();
    for import in imports {
        for export in import.exports().iter().filter(|export| {
            &export.name == name && &export.decl_interface_hash == decl_interface_hash
        }) {
            let decl = kernel_env.decl(&export.name.as_dotted()).ok_or_else(|| {
                MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::InvalidEqFamily,
                    format!(
                        "Eq family head {} is exported but missing from kernel env",
                        export.name.as_dotted()
                    ),
                )
                .with_primary_name(export.name.clone())
            })?;
            matches.push(ResolvedFamilySignature {
                signature: CheckedDeclSignature::from_core_decl(decl, export.decl_interface_hash),
                origin: FamilyOrigin::Imported {
                    module: import.module.clone(),
                    export_hash: import.export_hash,
                    certificate_hash: import.certificate_hash,
                },
            });
        }
    }
    for decl in current {
        if let Some(signature) =
            resolve_current_family_signature(kernel_env, decl, name, decl_interface_hash)?
        {
            matches.push(signature);
        }
    }
    let [resolved] = matches.as_slice() else {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidEqFamily,
            if matches.is_empty() {
                format!(
                    "Eq family head {} with the requested interface hash is unknown",
                    name.as_dotted()
                )
            } else {
                format!("Eq family head {} is ambiguous", name.as_dotted())
            },
        )
        .with_primary_name(name.clone()));
    };
    Ok(resolved.clone())
}

fn resolve_current_family_signature(
    kernel_env: &Env,
    decl: &CheckedCurrentDecl,
    name: &Name,
    decl_interface_hash: &Hash,
) -> Result<Option<ResolvedFamilySignature>> {
    let parent_signature = decl.signature();
    if parent_signature.name() == name
        && parent_signature.decl_interface_hash() == *decl_interface_hash
    {
        return Ok(Some(ResolvedFamilySignature {
            signature: parent_signature.clone(),
            origin: FamilyOrigin::CurrentSourceDecl {
                module: current_decl_module_name(parent_signature.name()),
                source_index: decl.source_index(),
                core_decl_hash: decl.core_decl_hash(),
            },
        }));
    }

    if parent_signature.decl_interface_hash() != *decl_interface_hash {
        return Ok(None);
    }
    let Decl::Inductive { data, .. } = decl.core_decl() else {
        return Ok(None);
    };
    let generated_by_parent = data
        .constructors
        .iter()
        .any(|constructor| Name::from_dotted(&constructor.name) == *name)
        || data
            .recursor
            .as_ref()
            .map(|recursor| Name::from_dotted(&recursor.name) == *name)
            .unwrap_or(false);
    if !generated_by_parent {
        return Ok(None);
    }

    let generated = kernel_env.decl(&name.as_dotted()).ok_or_else(|| {
        MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidEqFamily,
            format!(
                "current generated Eq family head {} is missing from kernel env",
                name.as_dotted()
            ),
        )
        .with_primary_name(name.clone())
    })?;
    match generated {
        Decl::Constructor { inductive, .. } | Decl::Recursor { inductive, .. }
            if inductive == decl.core_decl().name() => {}
        _ => {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidEqFamily,
                format!(
                    "current generated Eq family head {} is not generated by {}",
                    name.as_dotted(),
                    decl.core_decl().name()
                ),
            )
            .with_primary_name(name.clone()));
        }
    }

    Ok(Some(ResolvedFamilySignature {
        signature: CheckedDeclSignature::from_core_decl(generated, *decl_interface_hash),
        origin: FamilyOrigin::CurrentSourceDecl {
            module: current_decl_module_name(parent_signature.name()),
            source_index: decl.source_index(),
            core_decl_hash: decl.core_decl_hash(),
        },
    }))
}

fn current_decl_module_name(name: &Name) -> ModuleName {
    Name(vec![name
        .0
        .first()
        .cloned()
        .unwrap_or_else(|| name.as_dotted())])
}

fn encode_family_origin_to(out: &mut Vec<u8>, origin: &FamilyOrigin) {
    match origin {
        FamilyOrigin::Imported {
            module,
            export_hash,
            certificate_hash,
        } => {
            out.push(0x00);
            encode_name_to(out, module);
            encode_hash_to(out, export_hash);
            encode_hash_to(out, certificate_hash);
        }
        FamilyOrigin::CurrentSourceDecl {
            module,
            source_index,
            core_decl_hash,
        } => {
            out.push(0x01);
            encode_name_to(out, module);
            encode_u64_to(out, *source_index);
            encode_hash_to(out, core_decl_hash);
        }
    }
}

fn check_eq_family_interfaces(
    kernel_env: &Env,
    eq: &ResolvedFamilySignature,
    refl: &ResolvedFamilySignature,
    rec: &ResolvedFamilySignature,
) -> Result<()> {
    let eq_params = eq.signature.universe_params();
    let refl_params = refl.signature.universe_params();
    let rec_params = rec.signature.universe_params();
    if eq_params.len() != 1 || refl_params.len() != 1 || rec_params.len() != 2 {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidEqFamily,
            "Eq family universe arity does not match Eq/Eq.refl/Eq.rec",
        ));
    }
    let eq_level = Level::param(eq_params[0].clone());
    let refl_level = Level::param(refl_params[0].clone());
    let rec_value_level = Level::param(rec_params[0].clone());
    let rec_motive_level = Level::param(rec_params[1].clone());
    let expected_eq = expected_eq_type(&eq.signature.name().as_dotted(), eq_level.clone());
    let expected_refl = expected_eq_refl_type(
        &eq.signature.name().as_dotted(),
        &refl.signature.name().as_dotted(),
        refl_level,
    );
    let expected_rec = expected_eq_rec_type(
        &eq.signature.name().as_dotted(),
        &refl.signature.name().as_dotted(),
        rec_value_level,
        rec_motive_level,
    );
    for (actual, expected, label) in [
        (eq.signature.ty(), &expected_eq, "Eq"),
        (refl.signature.ty(), &expected_refl, "Eq.refl"),
        (rec.signature.ty(), &expected_rec, "Eq.rec"),
    ] {
        if !kernel_env
            .is_defeq(
                &Ctx::new(),
                actual_universe_params_for_signature(label, eq, refl, rec),
                actual,
                expected,
            )
            .map_err(kernel_diag)?
        {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidEqFamily,
                format!("{label} family interface does not match the required M4 shape"),
            )
            .with_expected_actual_payloads(
                DiagnosticPayloadKind::Expr,
                &core_expr_canonical_bytes(expected),
                &core_expr_canonical_bytes(actual),
            ));
        }
    }
    Ok(())
}

fn actual_universe_params_for_signature<'a>(
    label: &str,
    eq: &'a ResolvedFamilySignature,
    refl: &'a ResolvedFamilySignature,
    rec: &'a ResolvedFamilySignature,
) -> &'a [String] {
    match label {
        "Eq" => eq.signature.universe_params(),
        "Eq.refl" => refl.signature.universe_params(),
        "Eq.rec" => rec.signature.universe_params(),
        _ => &[],
    }
}

fn expected_eq_type(eq_name: &str, level: Level) -> Expr {
    let _ = eq_name;
    npa_kernel::eq_type(level)
}

fn expected_eq_refl_type(eq_name: &str, refl_name: &str, level: Level) -> Expr {
    let _ = refl_name;
    Expr::pi(
        "A",
        Expr::sort(level.clone()),
        Expr::pi(
            "x",
            Expr::bvar(0),
            Expr::apps(
                Expr::konst(eq_name, vec![level]),
                vec![Expr::bvar(1), Expr::bvar(0), Expr::bvar(0)],
            ),
        ),
    )
}

fn expected_eq_rec_type(
    eq_name: &str,
    refl_name: &str,
    value_level: Level,
    motive_level: Level,
) -> Expr {
    let a_sort_level = value_level.clone();
    let motive_ty = Expr::pi(
        "b",
        Expr::bvar(1),
        Expr::pi(
            "h",
            Expr::apps(
                Expr::konst(eq_name, vec![value_level.clone()]),
                vec![Expr::bvar(2), Expr::bvar(1), Expr::bvar(0)],
            ),
            Expr::sort(motive_level),
        ),
    );
    let refl_proof = Expr::apps(
        Expr::konst(refl_name, vec![value_level.clone()]),
        vec![Expr::bvar(2), Expr::bvar(1)],
    );
    let minor_ty = Expr::apps(Expr::bvar(0), vec![Expr::bvar(1), refl_proof]);
    let major_ty = Expr::apps(
        Expr::konst(eq_name, vec![value_level]),
        vec![Expr::bvar(4), Expr::bvar(3), Expr::bvar(0)],
    );
    let result_ty = Expr::apps(Expr::bvar(3), vec![Expr::bvar(1), Expr::bvar(0)]);
    Expr::pi(
        "A",
        Expr::sort(a_sort_level),
        Expr::pi(
            "a",
            Expr::bvar(0),
            Expr::pi(
                "motive",
                motive_ty,
                Expr::pi(
                    "minor",
                    minor_ty,
                    Expr::pi("b", Expr::bvar(3), Expr::pi("h", major_ty, result_ty)),
                ),
            ),
        ),
    )
}

fn resolve_nat_family(
    kernel_env: &Env,
    imports: &[VerifiedImportRef],
    current: &[CheckedCurrentDecl],
    family_ref: Option<&NatFamilyRef>,
) -> Result<Option<ResolvedNatFamily>> {
    let Some(reference) = family_ref else {
        return Ok(None);
    };
    let nat = resolve_nat_family_signature(
        kernel_env,
        imports,
        current,
        &reference.nat_name,
        &reference.nat_interface_hash,
    )?;
    let zero = resolve_nat_family_signature(
        kernel_env,
        imports,
        current,
        &reference.zero_name,
        &reference.zero_interface_hash,
    )?;
    let succ = resolve_nat_family_signature(
        kernel_env,
        imports,
        current,
        &reference.succ_name,
        &reference.succ_interface_hash,
    )?;
    let rec = resolve_nat_family_signature(
        kernel_env,
        imports,
        current,
        &reference.rec_name,
        &reference.rec_interface_hash,
    )?;
    check_nat_family_interfaces(kernel_env, &nat, &zero, &succ, &rec)?;
    if nat.origin != zero.origin || nat.origin != succ.origin || nat.origin != rec.origin {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidNatFamily,
            "Nat family heads must resolve to the same verified import or checked current declaration",
        ));
    }
    let mut out = tagged_bytes("npa.phase4.resolved-nat-family.decl.v1");
    encode_family_origin_to(&mut out, &nat.origin);
    encode_checked_decl_signature_to(&mut out, &nat.signature);
    encode_checked_decl_signature_to(&mut out, &zero.signature);
    encode_checked_decl_signature_to(&mut out, &succ.signature);
    encode_checked_decl_signature_to(&mut out, &rec.signature);
    Ok(Some(ResolvedNatFamily {
        nat_name: reference.nat_name.clone(),
        zero_name: reference.zero_name.clone(),
        succ_name: reference.succ_name.clone(),
        rec_name: reference.rec_name.clone(),
        fingerprint: hash_with_domain("npa.phase4.resolved-nat-family.v1", &out),
    }))
}

fn resolve_nat_family_signature(
    kernel_env: &Env,
    imports: &[VerifiedImportRef],
    current: &[CheckedCurrentDecl],
    name: &Name,
    decl_interface_hash: &Hash,
) -> Result<ResolvedFamilySignature> {
    let mut matches = Vec::new();
    for import in imports {
        for export in import.exports().iter().filter(|export| {
            &export.name == name && &export.decl_interface_hash == decl_interface_hash
        }) {
            let decl = kernel_env.decl(&export.name.as_dotted()).ok_or_else(|| {
                MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::InvalidNatFamily,
                    format!(
                        "Nat family head {} is exported but missing from kernel env",
                        export.name.as_dotted()
                    ),
                )
                .with_primary_name(export.name.clone())
            })?;
            matches.push(ResolvedFamilySignature {
                signature: CheckedDeclSignature::from_core_decl(decl, export.decl_interface_hash),
                origin: FamilyOrigin::Imported {
                    module: import.module.clone(),
                    export_hash: import.export_hash,
                    certificate_hash: import.certificate_hash,
                },
            });
        }
    }
    for decl in current {
        if let Some(signature) =
            resolve_current_nat_family_signature(kernel_env, decl, name, decl_interface_hash)?
        {
            matches.push(signature);
        }
    }
    let [resolved] = matches.as_slice() else {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidNatFamily,
            if matches.is_empty() {
                format!(
                    "Nat family head {} with the requested interface hash is unknown",
                    name.as_dotted()
                )
            } else {
                format!("Nat family head {} is ambiguous", name.as_dotted())
            },
        )
        .with_primary_name(name.clone()));
    };
    Ok(resolved.clone())
}

fn resolve_current_nat_family_signature(
    kernel_env: &Env,
    decl: &CheckedCurrentDecl,
    name: &Name,
    decl_interface_hash: &Hash,
) -> Result<Option<ResolvedFamilySignature>> {
    let parent_signature = decl.signature();
    if parent_signature.name() == name
        && parent_signature.decl_interface_hash() == *decl_interface_hash
    {
        return Ok(Some(ResolvedFamilySignature {
            signature: parent_signature.clone(),
            origin: FamilyOrigin::CurrentSourceDecl {
                module: current_decl_module_name(parent_signature.name()),
                source_index: decl.source_index(),
                core_decl_hash: decl.core_decl_hash(),
            },
        }));
    }

    if parent_signature.decl_interface_hash() != *decl_interface_hash {
        return Ok(None);
    }
    let Decl::Inductive { data, .. } = decl.core_decl() else {
        return Ok(None);
    };
    let generated_by_parent = data
        .constructors
        .iter()
        .any(|constructor| Name::from_dotted(&constructor.name) == *name)
        || data
            .recursor
            .as_ref()
            .map(|recursor| Name::from_dotted(&recursor.name) == *name)
            .unwrap_or(false);
    if !generated_by_parent {
        return Ok(None);
    }

    let generated = kernel_env.decl(&name.as_dotted()).ok_or_else(|| {
        MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidNatFamily,
            format!(
                "current generated Nat family head {} is missing from kernel env",
                name.as_dotted()
            ),
        )
        .with_primary_name(name.clone())
    })?;
    match generated {
        Decl::Constructor { inductive, .. } | Decl::Recursor { inductive, .. }
            if inductive == decl.core_decl().name() => {}
        _ => {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidNatFamily,
                format!(
                    "current generated Nat family head {} is not generated by {}",
                    name.as_dotted(),
                    decl.core_decl().name()
                ),
            )
            .with_primary_name(name.clone()));
        }
    }

    Ok(Some(ResolvedFamilySignature {
        signature: CheckedDeclSignature::from_core_decl(generated, *decl_interface_hash),
        origin: FamilyOrigin::CurrentSourceDecl {
            module: current_decl_module_name(parent_signature.name()),
            source_index: decl.source_index(),
            core_decl_hash: decl.core_decl_hash(),
        },
    }))
}

fn check_nat_family_interfaces(
    kernel_env: &Env,
    nat: &ResolvedFamilySignature,
    zero: &ResolvedFamilySignature,
    succ: &ResolvedFamilySignature,
    rec: &ResolvedFamilySignature,
) -> Result<()> {
    check_nat_family_decl_kinds(kernel_env, nat, zero, succ, rec)?;
    if !nat.signature.universe_params().is_empty()
        || !zero.signature.universe_params().is_empty()
        || !succ.signature.universe_params().is_empty()
        || rec.signature.universe_params().len() != 1
    {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidNatFamily,
            "Nat family universe arity does not match Nat/Nat.zero/Nat.succ/Nat.rec",
        ));
    }
    let nat_expr = Expr::konst(nat.signature.name().as_dotted(), Vec::new());
    let expected_succ = Expr::pi("_", nat_expr.clone(), nat_expr.clone());
    let rec_level = Level::param(rec.signature.universe_params()[0].clone());
    let expected_rec = expected_nat_rec_type(
        &nat.signature.name().as_dotted(),
        &zero.signature.name().as_dotted(),
        &succ.signature.name().as_dotted(),
        rec_level,
    );
    for (actual, expected, delta, label) in [
        (
            zero.signature.ty(),
            &nat_expr,
            zero.signature.universe_params(),
            "Nat.zero",
        ),
        (
            succ.signature.ty(),
            &expected_succ,
            succ.signature.universe_params(),
            "Nat.succ",
        ),
        (
            rec.signature.ty(),
            &expected_rec,
            rec.signature.universe_params(),
            "Nat.rec",
        ),
    ] {
        if !kernel_env
            .is_defeq(&Ctx::new(), delta, actual, expected)
            .map_err(kernel_diag)?
        {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidNatFamily,
                format!("{label} family interface does not match the required M5 shape"),
            )
            .with_expected_actual_payloads(
                DiagnosticPayloadKind::Expr,
                &core_expr_canonical_bytes(expected),
                &core_expr_canonical_bytes(actual),
            ));
        }
    }
    match nat.signature.ty() {
        Expr::Sort(_) => Ok(()),
        actual => Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidNatFamily,
            "Nat family head must have a sort type",
        )
        .with_expected_actual_payloads(
            DiagnosticPayloadKind::Expr,
            &core_expr_canonical_bytes(&Expr::sort(npa_kernel::type0())),
            &core_expr_canonical_bytes(actual),
        )),
    }
}

fn check_nat_family_decl_kinds(
    kernel_env: &Env,
    nat: &ResolvedFamilySignature,
    zero: &ResolvedFamilySignature,
    succ: &ResolvedFamilySignature,
    rec: &ResolvedFamilySignature,
) -> Result<()> {
    let nat_name = nat.signature.name().as_dotted();
    let nat_decl = kernel_env.decl(&nat_name).ok_or_else(|| {
        MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidNatFamily,
            format!("Nat family head {nat_name} is missing from kernel env"),
        )
    })?;
    if !matches!(nat_decl, Decl::Inductive { .. }) {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidNatFamily,
            format!("Nat family head {nat_name} must be an inductive declaration"),
        ));
    }
    for (signature, label) in [(zero, "Nat.zero"), (succ, "Nat.succ")] {
        let name = signature.signature.name().as_dotted();
        match kernel_env.decl(&name) {
            Some(Decl::Constructor { inductive, .. }) if inductive == &nat_name => {}
            _ => {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::InvalidNatFamily,
                    format!(
                        "{label} family head {name} must be a constructor generated by {nat_name}"
                    ),
                ));
            }
        }
    }
    let rec_name = rec.signature.name().as_dotted();
    match kernel_env.decl(&rec_name) {
        Some(Decl::Recursor { inductive, .. }) if inductive == &nat_name => Ok(()),
        _ => Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidNatFamily,
            format!("Nat.rec family head {rec_name} must be a recursor generated by {nat_name}"),
        )),
    }
}

fn expected_nat_rec_type(nat_name: &str, zero_name: &str, succ_name: &str, level: Level) -> Expr {
    let nat = Expr::konst(nat_name, Vec::new());
    let zero = Expr::konst(zero_name, Vec::new());
    let succ = |arg| Expr::app(Expr::konst(succ_name, Vec::new()), arg);
    let motive_ty = Expr::pi("_", nat.clone(), Expr::sort(level.clone()));
    let z_ty = Expr::app(Expr::bvar(0), zero);
    let s_ty = Expr::pi(
        "n",
        nat.clone(),
        Expr::pi(
            "ih",
            Expr::app(Expr::bvar(2), Expr::bvar(0)),
            Expr::app(Expr::bvar(3), succ(Expr::bvar(1))),
        ),
    );
    Expr::pi(
        "motive",
        motive_ty,
        Expr::pi(
            "z",
            z_ty,
            Expr::pi(
                "s",
                s_ty,
                Expr::pi("n", nat, Expr::app(Expr::bvar(3), Expr::bvar(0))),
            ),
        ),
    )
}

fn resolve_simp_registry(
    kernel_env: &Env,
    imports: &[VerifiedImportRef],
    current: &[CheckedCurrentDecl],
    eq_family: &Option<ResolvedEqFamily>,
    rules: &[SimpRuleRef],
) -> Result<SimpRegistry> {
    let canonical = canonicalize_simp_rule_refs(rules.to_vec())?;
    let mut resolved_rules = Vec::new();
    for rule in canonical {
        resolved_rules.push(resolve_simp_rule(
            kernel_env, imports, current, eq_family, rule,
        )?);
    }
    resolved_rules.sort_by_key(|rule| SimpRuleKey::from(&rule.key));
    Ok(SimpRegistry {
        rules: resolved_rules,
    })
}

fn resolve_simp_rule(
    kernel_env: &Env,
    imports: &[VerifiedImportRef],
    current: &[CheckedCurrentDecl],
    eq_family: &Option<ResolvedEqFamily>,
    rule: SimpRuleRef,
) -> Result<ResolvedSimpRule> {
    let mut matches = Vec::new();
    for import in imports {
        for export in import.exports().iter().filter(|export| {
            export.name == rule.name && export.decl_interface_hash == rule.decl_interface_hash
        }) {
            let decl = kernel_env.decl(&export.name.as_dotted()).ok_or_else(|| {
                MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::UnknownSimpRule,
                    format!(
                        "simp rule {} is exported but missing from kernel env",
                        export.name.as_dotted()
                    ),
                )
                .with_primary_name(export.name.clone())
            })?;
            let core_hash = import
                .certified_env_decls()
                .iter()
                .zip(import.certified_env_decl_hashes())
                .find(|(decl, _)| decl.name() == export.name.as_dotted())
                .map(|(_, hash)| *hash)
                .unwrap_or(export.decl_interface_hash);
            matches.push((
                TacticHead::Imported {
                    name: export.name.clone(),
                    decl_interface_hash: export.decl_interface_hash,
                },
                CheckedDeclSignature::from_core_decl(decl, export.decl_interface_hash),
                core_hash,
                decl.clone(),
                SimpRuleOrigin::Imported { kind: export.kind },
            ));
        }
    }
    for decl in current {
        if decl.signature().name() == &rule.name
            && decl.signature().decl_interface_hash() == rule.decl_interface_hash
        {
            matches.push((
                TacticHead::CurrentModule {
                    name: rule.name.clone(),
                    decl_interface_hash: rule.decl_interface_hash,
                },
                decl.signature().clone(),
                decl.core_decl_hash(),
                decl.core_decl().clone(),
                SimpRuleOrigin::CurrentModule,
            ));
        }
    }
    let [(source, signature, core_decl_hash, core_decl, origin)] = matches.as_slice() else {
        return Err(if matches.is_empty() {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::UnknownSimpRule,
                format!(
                    "simp rule {} with the requested interface hash is unknown",
                    rule.name.as_dotted()
                ),
            )
            .with_primary_name(rule.name.clone())
        } else {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::AmbiguousSimpRule,
                format!("simp rule {} is ambiguous", rule.name.as_dotted()),
            )
            .with_primary_name(rule.name.clone())
        });
    };
    ensure_simp_rule_decl_kind(core_decl, &rule.name, origin)?;
    let family = eq_family.as_ref().ok_or_else(|| {
        MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::TacticPrimitiveUnavailable,
            "simp registry resolution requires a resolved Eq family",
        )
    })?;
    let (rule_telescope, eq_target, theorem_ty) = analyze_simp_rule_type(
        kernel_env,
        family,
        signature.ty(),
        signature.universe_params(),
    )?;
    let (from_pattern, to_pattern) = match rule.direction {
        RewriteDirection::Forward => (eq_target.lhs.clone(), eq_target.rhs.clone()),
        RewriteDirection::Backward => (eq_target.rhs.clone(), eq_target.lhs.clone()),
    };
    Ok(ResolvedSimpRule {
        key: rule,
        source: source.clone(),
        signature: signature.clone(),
        core_decl_hash: *core_decl_hash,
        theorem_ty,
        universe_params: signature.universe_params().to_vec(),
        rule_telescope,
        eq_levels: eq_target.levels,
        eq_type: eq_target.ty,
        theorem_lhs: eq_target.lhs,
        theorem_rhs: eq_target.rhs,
        from_pattern,
        to_pattern,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SimpRuleOrigin {
    Imported { kind: ExportKind },
    CurrentModule,
}

fn ensure_simp_rule_decl_kind(decl: &Decl, name: &Name, origin: &SimpRuleOrigin) -> Result<()> {
    let valid = match origin {
        SimpRuleOrigin::Imported {
            kind: ExportKind::Theorem,
        } => true,
        SimpRuleOrigin::Imported {
            kind: ExportKind::Def,
        }
        | SimpRuleOrigin::CurrentModule => matches!(
            decl,
            Decl::Theorem { .. }
                | Decl::Def {
                    reducibility: npa_kernel::Reducibility::Reducible,
                    ..
                }
        ),
        SimpRuleOrigin::Imported { .. } => false,
    };
    if valid {
        Ok(())
    } else {
        Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidSimpRule,
            format!(
                "simp rule {} must resolve to a theorem or reducible proof definition",
                name.as_dotted()
            ),
        ))
    }
}

fn analyze_simp_rule_type(
    kernel_env: &Env,
    family: &ResolvedEqFamily,
    theorem_ty: &Expr,
    universe_params: &[String],
) -> Result<(Vec<ResolvedRuleParam>, EqTarget, Expr)> {
    let mut ctx = Ctx::new();
    let mut ty = theorem_ty.clone();
    let mut params = Vec::new();
    let mut theorem_ty_whnf = None;
    loop {
        let whnf = kernel_env
            .whnf(&ctx, universe_params, &ty)
            .map_err(kernel_diag)?;
        if theorem_ty_whnf.is_none() {
            theorem_ty_whnf = Some(whnf.clone());
        }
        if let Some(eq_target) = parse_eq_target_from_expr(family, &whnf) {
            ensure_simp_rule_params_inferable(&params, &eq_target)?;
            return Ok((
                params,
                eq_target,
                theorem_ty_whnf.expect("WHNF was stored before analysis returns"),
            ));
        }
        let Expr::Pi {
            binder,
            ty: domain,
            body,
        } = whnf
        else {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidSimpRule,
                "simp rule type does not reduce to an Eq theorem",
            ));
        };
        let domain_ty = kernel_env
            .infer(&ctx, universe_params, &domain)
            .map_err(kernel_diag)?;
        let domain_ty_whnf = kernel_env
            .whnf(&ctx, universe_params, &domain_ty)
            .map_err(kernel_diag)?;
        if matches!(domain_ty_whnf, Expr::Sort(level) if normalize_level(level.clone()) == Level::zero())
        {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidSimpRule,
                "simp-lite MVP rejects simp rules with proof premises",
            ));
        }
        params.push(ResolvedRuleParam {
            name: binder.clone(),
            ty: (*domain).clone(),
        });
        ctx.push_assumption(binder, *domain);
        ty = *body;
    }
}

fn ensure_simp_rule_params_inferable(
    params: &[ResolvedRuleParam],
    eq_target: &EqTarget,
) -> Result<()> {
    for (index, param) in params.iter().enumerate() {
        let bvar = (params.len() - 1 - index) as u32;
        if !contains_bound_var(&eq_target.ty, bvar)
            && !contains_bound_var(&eq_target.lhs, bvar)
            && !contains_bound_var(&eq_target.rhs, bvar)
        {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidSimpRule,
                format!(
                    "simp-lite MVP rejects simp rule binder {} because it is not inferable from the Eq conclusion",
                    param.name
                ),
            ));
        }
    }
    Ok(())
}

fn validate_machine_term_source(term: &MachineTermSource) -> Result<()> {
    let canonical =
        npa_frontend::canonicalize_machine_term_source(&term.source).map_err(|err| {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidMachineTermSource,
                format!(
                    "machine term source canonicalization failed: {}",
                    err.message
                ),
            )
        })?;
    let expected = machine_term_source_hash_from_phase3(&canonical.canonical_bytes);
    if term.canonical_hash != expected {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidMachineTermSource,
            "machine term source canonical hash is stale",
        )
        .with_expected_actual_payloads(
            DiagnosticPayloadKind::MachineTermSource,
            &machine_term_source_canonical_bytes_from_phase3(&canonical.canonical_bytes),
            &term.canonical_hash,
        ));
    }
    Ok(())
}

fn validate_tactic_head_shape(head: &TacticHead) -> Result<()> {
    match head {
        TacticHead::Imported { name, .. } | TacticHead::CurrentModule { name, .. } => {
            if !name.is_canonical() {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::InvalidMachineTactic,
                    format!("tactic head name {} is not canonical", name.as_dotted()),
                ));
            }
        }
        TacticHead::Local { name } => {
            if !is_machine_identifier(name) || is_reserved_spelling(name) {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::InvalidMachineTactic,
                    format!("local tactic head {name:?} is not a valid machine local identifier"),
                ));
            }
        }
    }
    Ok(())
}

fn validate_candidate_apply_arg(arg: CandidateApplyArg) -> Result<ApplyArg> {
    match arg {
        CandidateApplyArg::Term(term) => {
            Ok(ApplyArg::Term(MachineTermSource::new_checked(term.source)?))
        }
        CandidateApplyArg::Subgoal { name_hint } => Ok(ApplyArg::Subgoal { name_hint }),
        CandidateApplyArg::InferFromTarget => Ok(ApplyArg::InferFromTarget),
    }
}

fn validate_apply_arg(arg: &ApplyArg) -> Result<()> {
    match arg {
        ApplyArg::Term(term) => validate_machine_term_source(term),
        ApplyArg::Subgoal { .. } | ApplyArg::InferFromTarget => Ok(()),
    }
}

fn elaborate_apply_term_infer(
    term: &MachineTermSource,
    context: &npa_frontend::MachineTermElabContext,
) -> Result<(Expr, Expr)> {
    validate_machine_term_source(term)?;
    let canonical =
        npa_frontend::canonicalize_machine_term_source(term.source()).map_err(|err| {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidMachineTermSource,
                format!(
                    "machine term source canonicalization failed: {}",
                    err.message
                ),
            )
        })?;
    let ast = npa_frontend::decode_machine_term_source_canonical(&canonical.canonical_bytes)
        .map_err(|err| {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidMachineTermSource,
                format!("machine term canonical decode failed: {}", err.message),
            )
        })?;
    npa_frontend::elaborate_machine_term_infer_from_ast(
        &ast,
        context,
        &npa_frontend::MachineCompileOptions::default(),
    )
    .map_err(machine_term_elaboration_diag)
}

fn universe_argument_mismatch_diag(
    expected_params: &[String],
    actual_levels: &[Level],
    goal_id: GoalId,
    meta_id: MetaVarId,
) -> MachineTacticDiagnostic {
    MachineTacticDiagnostic::new(
        MachineTacticDiagnosticKind::UniverseArgumentMismatch,
        format!(
            "tactic head expects {} universe arguments, got {}",
            expected_params.len(),
            actual_levels.len()
        ),
    )
    .with_expected_actual_payload_kinds(
        DiagnosticPayloadKind::UniverseParamList,
        &universe_param_list_canonical_bytes(expected_params),
        DiagnosticPayloadKind::LevelArgList,
        &level_arg_list_canonical_bytes(actual_levels),
    )
    .with_goal(goal_id)
    .with_meta(meta_id)
}

fn ensure_subgoal_domain_is_prop(
    state: &MachineProofState,
    ctx: &Ctx,
    goal: &MachineGoal,
    domain: &Expr,
    budget_context: Option<KernelFuelContext<'_>>,
) -> Result<()> {
    let env = state.env.kernel_env();
    let domain_ty = match budget_context {
        Some((fuel, goal_id, meta_id)) => kernel_infer_with_budget(
            env,
            ctx,
            &state.root.universe_params,
            domain,
            fuel,
            goal_id,
            meta_id,
        )?,
        None => env
            .infer(ctx, &state.root.universe_params, domain)
            .map_err(kernel_diag)?,
    };
    let domain_ty_whnf = match budget_context {
        Some((fuel, goal_id, meta_id)) => kernel_whnf_with_budget(
            env,
            ctx,
            &state.root.universe_params,
            &domain_ty,
            fuel,
            goal_id,
            meta_id,
        )?,
        None => env
            .whnf(ctx, &state.root.universe_params, &domain_ty)
            .map_err(kernel_diag)?,
    };
    match domain_ty_whnf {
        Expr::Sort(level) if normalize_level(level.clone()) == Level::Zero => Ok(()),
        actual => Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::SubgoalDataArgument,
            "apply Subgoal is only allowed for proposition-valued binder domains",
        )
        .with_expected_actual_payloads(
            DiagnosticPayloadKind::Expr,
            &core_expr_canonical_bytes(&Expr::sort(Level::zero())),
            &core_expr_canonical_bytes(&actual),
        )
        .with_goal(goal.id)
        .with_meta(goal.meta_id)),
    }
}

fn infer_apply_args_from_target(
    state: &MachineProofState,
    ctx: &Ctx,
    goal: &MachineGoal,
    result_pattern: &Expr,
    infer_ids: &[PatternMetaId],
    fuel: &TacticRunFuel,
) -> Result<BTreeMap<PatternMetaId, Expr>> {
    let env = state.env.kernel_env();
    let delta = &state.root.universe_params;
    let pattern_whnf =
        kernel_whnf_with_budget(env, ctx, delta, result_pattern, fuel, goal.id, goal.meta_id)?;
    let target_whnf =
        kernel_whnf_with_budget(env, ctx, delta, &goal.target, fuel, goal.id, goal.meta_id)?;
    let mut present = BTreeSet::new();
    collect_pattern_meta_ids(&pattern_whnf, &mut present);
    for id in infer_ids {
        if !present.contains(id) {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::MissingExplicitArgument,
                "InferFromTarget argument does not occur in the result pattern",
            )
            .with_goal(goal.id)
            .with_meta(goal.meta_id));
        }
    }
    let mut solutions = BTreeMap::new();
    match_apply_pattern(
        env,
        ctx,
        delta,
        &pattern_whnf,
        &target_whnf,
        false,
        &mut solutions,
        Some((fuel, goal.id, goal.meta_id)),
    )
    .map_err(|mut diag| {
        diag.goal_id = Some(goal.id);
        diag.meta_id = Some(goal.meta_id);
        diag
    })?;
    for id in infer_ids {
        if !solutions.contains_key(id) {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::AmbiguousApplyArgument,
                "InferFromTarget argument was not uniquely solved",
            )
            .with_goal(goal.id)
            .with_meta(goal.meta_id));
        }
    }
    Ok(solutions)
}

#[allow(clippy::too_many_arguments)]
fn match_apply_pattern(
    env: &Env,
    ctx: &Ctx,
    delta: &[String],
    pattern: &Expr,
    target: &Expr,
    allow_pattern_meta: bool,
    solutions: &mut BTreeMap<PatternMetaId, Expr>,
    budget_context: Option<KernelFuelContext<'_>>,
) -> Result<()> {
    if let Some(id) = pattern_meta_id(pattern) {
        if !allow_pattern_meta {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::AmbiguousApplyArgument,
                "InferFromTarget cannot be solved from a non-rigid result position",
            ));
        }
        return assign_pattern_meta_solution(
            env,
            ctx,
            delta,
            id,
            target,
            solutions,
            budget_context,
        );
    }
    if !contains_pattern_meta(pattern) {
        if kernel_is_defeq_with_optional_budget(env, ctx, delta, pattern, target, budget_context)? {
            return Ok(());
        }
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::AmbiguousApplyArgument,
            "rigid apply result pattern does not match the goal target",
        ));
    }

    match pattern {
        Expr::App(_, _) => {
            let (pattern_head, pattern_args) = collect_core_apps(pattern);
            if contains_pattern_meta(&pattern_head) {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::AmbiguousApplyArgument,
                    "InferFromTarget cannot occur in function-head position",
                ));
            }
            let (target_head, target_args) = collect_core_apps(target);
            if pattern_args.len() != target_args.len()
                || !rigid_heads_match(env, ctx, delta, &pattern_head, &target_head, budget_context)?
            {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::AmbiguousApplyArgument,
                    "apply result pattern and target have incompatible rigid heads",
                ));
            }
            for (pattern_arg, target_arg) in pattern_args.iter().zip(&target_args) {
                match_apply_pattern(
                    env,
                    ctx,
                    delta,
                    pattern_arg,
                    target_arg,
                    true,
                    solutions,
                    budget_context,
                )?;
            }
            Ok(())
        }
        Expr::Lam { .. } | Expr::Pi { .. } | Expr::Let { .. } => Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::AmbiguousApplyArgument,
            "InferFromTarget under binders or let bodies is not supported",
        )),
        Expr::Sort(_) | Expr::BVar(_) | Expr::Const { .. } => Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::AmbiguousApplyArgument,
            "unsupported InferFromTarget result pattern shape",
        )),
    }
}

fn assign_pattern_meta_solution(
    env: &Env,
    ctx: &Ctx,
    delta: &[String],
    id: PatternMetaId,
    target: &Expr,
    solutions: &mut BTreeMap<PatternMetaId, Expr>,
    budget_context: Option<KernelFuelContext<'_>>,
) -> Result<()> {
    if contains_pattern_meta(target) {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::AmbiguousApplyArgument,
            "InferFromTarget solution cannot contain another inferred argument",
        ));
    }
    match solutions.get(&id) {
        Some(existing)
            if kernel_is_defeq_with_optional_budget(
                env,
                ctx,
                delta,
                existing,
                target,
                budget_context,
            )? =>
        {
            Ok(())
        }
        Some(_) => Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::AmbiguousApplyArgument,
            "InferFromTarget has multiple non-convertible target matches",
        )),
        None => {
            solutions.insert(id, target.clone());
            Ok(())
        }
    }
}

fn rigid_heads_match(
    env: &Env,
    ctx: &Ctx,
    delta: &[String],
    pattern_head: &Expr,
    target_head: &Expr,
    budget_context: Option<KernelFuelContext<'_>>,
) -> Result<bool> {
    match (pattern_head, target_head) {
        (
            Expr::Const {
                name: lhs_name,
                levels: lhs_levels,
            },
            Expr::Const {
                name: rhs_name,
                levels: rhs_levels,
            },
        ) => Ok(lhs_name == rhs_name && levels_eq(lhs_levels, rhs_levels)),
        (Expr::BVar(lhs), Expr::BVar(rhs)) => Ok(lhs == rhs),
        _ => kernel_is_defeq_with_optional_budget(
            env,
            ctx,
            delta,
            pattern_head,
            target_head,
            budget_context,
        ),
    }
}

fn collect_core_apps(expr: &Expr) -> (Expr, Vec<Expr>) {
    let mut args = Vec::new();
    let mut head = expr.clone();
    while let Expr::App(fun, arg) = head {
        args.push(*arg);
        head = *fun;
    }
    args.reverse();
    (head, args)
}

const PATTERN_META_PREFIX: &str = "\0npa.phase4.pattern-meta.";

fn pattern_meta_expr(id: PatternMetaId) -> Expr {
    Expr::konst(format!("{PATTERN_META_PREFIX}{}", id.0), Vec::new())
}

fn pattern_meta_id(expr: &Expr) -> Option<PatternMetaId> {
    let Expr::Const { name, levels } = expr else {
        return None;
    };
    if !levels.is_empty() {
        return None;
    }
    let suffix = name.strip_prefix(PATTERN_META_PREFIX)?;
    suffix.parse::<u64>().ok().map(PatternMetaId)
}

fn contains_pattern_meta(expr: &Expr) -> bool {
    if pattern_meta_id(expr).is_some() {
        return true;
    }
    match expr {
        Expr::Sort(_) | Expr::BVar(_) | Expr::Const { .. } => false,
        Expr::App(fun, arg) => contains_pattern_meta(fun) || contains_pattern_meta(arg),
        Expr::Lam { ty, body, .. } | Expr::Pi { ty, body, .. } => {
            contains_pattern_meta(ty) || contains_pattern_meta(body)
        }
        Expr::Let {
            ty, value, body, ..
        } => {
            contains_pattern_meta(ty) || contains_pattern_meta(value) || contains_pattern_meta(body)
        }
    }
}

fn collect_pattern_meta_ids(expr: &Expr, ids: &mut BTreeSet<PatternMetaId>) {
    if let Some(id) = pattern_meta_id(expr) {
        ids.insert(id);
        return;
    }
    match expr {
        Expr::Sort(_) | Expr::BVar(_) | Expr::Const { .. } => {}
        Expr::App(fun, arg) => {
            collect_pattern_meta_ids(fun, ids);
            collect_pattern_meta_ids(arg, ids);
        }
        Expr::Lam { ty, body, .. } | Expr::Pi { ty, body, .. } => {
            collect_pattern_meta_ids(ty, ids);
            collect_pattern_meta_ids(body, ids);
        }
        Expr::Let {
            ty, value, body, ..
        } => {
            collect_pattern_meta_ids(ty, ids);
            collect_pattern_meta_ids(value, ids);
            collect_pattern_meta_ids(body, ids);
        }
    }
}

fn replace_pattern_metas(expr: &Expr, solutions: &BTreeMap<PatternMetaId, Expr>) -> Result<Expr> {
    replace_pattern_metas_at_depth(expr, solutions, 0)
}

fn replace_pattern_metas_at_depth(
    expr: &Expr,
    solutions: &BTreeMap<PatternMetaId, Expr>,
    depth: u32,
) -> Result<Expr> {
    if let Some(id) = pattern_meta_id(expr) {
        let solution = solutions.get(&id).ok_or_else(|| {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::AmbiguousApplyArgument,
                "unresolved InferFromTarget argument remains after matching",
            )
        })?;
        return shift(solution, depth as i32, 0).map_err(kernel_diag);
    }
    Ok(match expr {
        Expr::Sort(level) => Expr::sort(level.clone()),
        Expr::BVar(index) => Expr::bvar(*index),
        Expr::Const { name, levels } => Expr::konst(name.clone(), levels.clone()),
        Expr::App(fun, arg) => Expr::app(
            replace_pattern_metas_at_depth(fun, solutions, depth)?,
            replace_pattern_metas_at_depth(arg, solutions, depth)?,
        ),
        Expr::Lam { binder, ty, body } => Expr::lam(
            binder.clone(),
            replace_pattern_metas_at_depth(ty, solutions, depth)?,
            replace_pattern_metas_at_depth(body, solutions, depth + 1)?,
        ),
        Expr::Pi { binder, ty, body } => Expr::pi(
            binder.clone(),
            replace_pattern_metas_at_depth(ty, solutions, depth)?,
            replace_pattern_metas_at_depth(body, solutions, depth + 1)?,
        ),
        Expr::Let {
            binder,
            ty,
            value,
            body,
        } => Expr::let_in(
            binder.clone(),
            replace_pattern_metas_at_depth(ty, solutions, depth)?,
            replace_pattern_metas_at_depth(value, solutions, depth)?,
            replace_pattern_metas_at_depth(body, solutions, depth + 1)?,
        ),
    })
}

fn validate_intro_name_shape(name: &str) -> Result<()> {
    if !is_machine_identifier(name) || is_reserved_spelling(name) {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidMachineTactic,
            format!("intro name {name:?} is not a valid machine local identifier"),
        ));
    }
    Ok(())
}

fn validate_intro_name_available(
    state: &MachineProofState,
    context: &[MachineLocalDecl],
    name: &str,
) -> Result<()> {
    if context.iter().any(|local| local.name == name) {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidMachineTactic,
            format!("intro name {name} already exists in the local context"),
        ));
    }
    if machine_global_roots(state).contains(name) {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidMachineTactic,
            format!("intro name {name} shadows a global namespace root"),
        ));
    }
    Ok(())
}

fn machine_global_roots(state: &MachineProofState) -> BTreeSet<String> {
    let mut roots = BTreeSet::new();
    insert_name_root(&mut roots, &state.root.theorem_name);
    for import in &state.env.imports {
        for export in &import.exports {
            insert_name_root(&mut roots, &export.name);
        }
    }
    for checked in &state.env.checked_current_decls {
        insert_name_root(&mut roots, checked.signature.name());
        if let Decl::Inductive { data, .. } = &checked.core_decl {
            for constructor in &data.constructors {
                insert_dotted_name_root(&mut roots, &constructor.name);
            }
            if let Some(recursor) = &data.recursor {
                insert_dotted_name_root(&mut roots, &recursor.name);
            }
        }
    }
    roots
}

fn insert_name_root(roots: &mut BTreeSet<String>, name: &Name) {
    if let Some(root) = name.0.first() {
        roots.insert(root.clone());
    }
}

fn insert_dotted_name_root(roots: &mut BTreeSet<String>, name: &str) {
    if let Some(root) = name.split('.').next() {
        if !root.is_empty() {
            roots.insert(root.to_owned());
        }
    }
}

fn is_machine_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_alphabetic()
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '\'')
}

fn is_machine_candidate_id(value: &str) -> bool {
    (1..=64).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn is_reserved_spelling(value: &str) -> bool {
    matches!(
        value,
        "import"
            | "def"
            | "theorem"
            | "fun"
            | "forall"
            | "let"
            | "in"
            | "Prop"
            | "Type"
            | "Sort"
            | "open"
            | "namespace"
            | "match"
            | "with"
            | "notation"
            | "infix"
            | "infixl"
            | "infixr"
            | "axiom"
            | "inductive"
            | "succ"
            | "max"
            | "imax"
    )
}

fn ensure_tactic_step_fuel(
    budget: TacticBudget,
    needed: u64,
    goal_id: GoalId,
    meta_id: MetaVarId,
) -> Result<()> {
    if budget.max_tactic_steps < needed {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::TacticStep,
            },
            format!(
                "tactic requires {needed} tactic step fuel, remaining tactic step fuel is {}",
                budget.max_tactic_steps
            ),
        )
        .with_goal(goal_id)
        .with_meta(meta_id));
    }
    Ok(())
}

fn ensure_expr_node_fuel(
    budget: TacticBudget,
    needed: u64,
    goal_id: GoalId,
    meta_id: MetaVarId,
) -> Result<()> {
    if budget.max_expr_nodes < needed {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::ExprNode,
            },
            format!(
                "tactic requires {needed} expression node fuel, remaining expression node fuel is {}",
                budget.max_expr_nodes
            ),
        )
        .with_goal(goal_id)
        .with_meta(meta_id));
    }
    Ok(())
}

fn machine_term_elab_context(
    state: &MachineProofState,
    context: &[MachineLocalDecl],
) -> Result<npa_frontend::MachineTermElabContext> {
    let callable_interface_table = machine_surface_callable_interface_table(state)?;
    let imports = state
        .env
        .imports
        .iter()
        .map(|import| import.verified_module.as_ref().clone())
        .collect::<Vec<_>>();
    let checked_current_decls = state
        .env
        .checked_current_decls
        .iter()
        .map(|decl| npa_frontend::MachineCheckedCurrentDecl {
            name: decl.signature.name().clone(),
            source_index: decl.source_index,
            decl_interface_hash: decl.signature.decl_interface_hash(),
            decl: decl.core_decl.clone(),
        })
        .collect::<Vec<_>>();
    let mut current_generated_decls = Vec::new();
    for decl in &state.env.checked_current_decls {
        if let Decl::Inductive { data, .. } = &decl.core_decl {
            for constructor in &data.constructors {
                current_generated_decls.push(npa_frontend::MachineCheckedCurrentGeneratedDecl {
                    name: Name::from_dotted(&constructor.name),
                    parent_source_index: decl.source_index,
                    decl_interface_hash: decl.signature.decl_interface_hash(),
                });
            }
            if let Some(recursor) = &data.recursor {
                current_generated_decls.push(npa_frontend::MachineCheckedCurrentGeneratedDecl {
                    name: Name::from_dotted(&recursor.name),
                    parent_source_index: decl.source_index,
                    decl_interface_hash: decl.signature.decl_interface_hash(),
                });
            }
        }
    }
    let local_context = context
        .iter()
        .map(|local| npa_frontend::MachineLocalDecl {
            name: local.name.clone(),
            ty: local.ty.clone(),
            value: local.value.clone(),
        })
        .collect();
    npa_frontend::MachineTermElabContext::from_verified_modules_and_current_decls_in_module(
        &imports,
        &imports,
        state.root.module.clone(),
        &checked_current_decls,
        &current_generated_decls,
        local_context,
        state.root.universe_params.clone(),
    )
    .map(|context| context.with_callable_interface_table(callable_interface_table))
    .map_err(machine_term_elaboration_diag)
}

fn machine_surface_callable_interface_table(
    state: &MachineProofState,
) -> Result<npa_frontend::MachineSurfaceCallableInterfaceTable> {
    let mut entries = Vec::new();
    for import in &state.env.imports {
        for export in &import.exports {
            ensure_imported_machine_surface_callable_name(import, &export.name)?;
            let ty = import_export_type(import, export)?;
            entries.push(
                npa_frontend::MachineSurfaceCallableInterfaceEntry::all_explicit(
                    npa_frontend::MachineSurfaceCallableRef::Imported {
                        module: import.module.clone(),
                        name: export.name.clone(),
                        export_hash: import.export_hash,
                        decl_interface_hash: export.decl_interface_hash,
                    },
                    expr_pi_telescope_len(&ty),
                ),
            );
        }
    }
    for checked in &state.env.checked_current_decls {
        ensure_current_machine_surface_callable_name(checked.signature.name())?;
        entries.push(
            npa_frontend::MachineSurfaceCallableInterfaceEntry::all_explicit(
                npa_frontend::MachineSurfaceCallableRef::CurrentModule {
                    module: state.root.module.clone(),
                    name: checked.signature.name.clone(),
                    source_index: checked.source_index,
                    decl_interface_hash: checked.signature.decl_interface_hash,
                },
                expr_pi_telescope_len(&checked.signature.ty),
            ),
        );
        if let Decl::Inductive { data, .. } = &checked.core_decl {
            for constructor in &data.constructors {
                let name = Name::from_dotted(&constructor.name);
                ensure_current_machine_surface_callable_name(&name)?;
                entries.push(
                    npa_frontend::MachineSurfaceCallableInterfaceEntry::all_explicit(
                        npa_frontend::MachineSurfaceCallableRef::CurrentGenerated {
                            module: state.root.module.clone(),
                            name,
                            parent_source_index: checked.source_index,
                            decl_interface_hash: checked.signature.decl_interface_hash,
                        },
                        expr_pi_telescope_len(&constructor.ty),
                    ),
                );
            }
            if let Some(recursor) = &data.recursor {
                let name = Name::from_dotted(&recursor.name);
                ensure_current_machine_surface_callable_name(&name)?;
                entries.push(
                    npa_frontend::MachineSurfaceCallableInterfaceEntry::all_explicit(
                        npa_frontend::MachineSurfaceCallableRef::CurrentGenerated {
                            module: state.root.module.clone(),
                            name,
                            parent_source_index: checked.source_index,
                            decl_interface_hash: checked.signature.decl_interface_hash,
                        },
                        expr_pi_telescope_len(&recursor.ty),
                    ),
                );
            }
        }
    }

    npa_frontend::MachineSurfaceCallableInterfaceTable::from_entries(entries).map_err(|err| {
        MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidMachineProofSpec,
            format!("machine surface callable interface table is invalid: {err:?}"),
        )
    })
}

fn ensure_imported_machine_surface_callable_name(
    import: &VerifiedImportRef,
    name: &Name,
) -> Result<()> {
    if npa_frontend::is_machine_surface_renderable_name(name) {
        return Ok(());
    }

    Err(MachineTacticDiagnostic::new(
        MachineTacticDiagnosticKind::InvalidVerifiedImport,
        format!(
            "verified import {} export {} is not Machine Surface renderable",
            import.module.as_dotted(),
            name.as_dotted()
        ),
    ))
}

fn ensure_current_machine_surface_callable_name(name: &Name) -> Result<()> {
    if npa_frontend::is_machine_surface_renderable_name(name) {
        return Ok(());
    }

    Err(MachineTacticDiagnostic::new(
        MachineTacticDiagnosticKind::InvalidMachineProofSpec,
        format!(
            "current declaration {} is not Machine Surface renderable",
            name.as_dotted()
        ),
    ))
}

fn import_export_type(
    import: &VerifiedImportRef,
    export: &VerifiedExportSignature,
) -> Result<Expr> {
    match export.kind {
        ExportKind::Axiom | ExportKind::Def | ExportKind::Theorem | ExportKind::Inductive => {
            import_ordinary_export_type(import, export)
        }
        ExportKind::Constructor | ExportKind::Recursor => {
            import_generated_export_type(import, export)
        }
    }
}

fn import_ordinary_export_type(
    import: &VerifiedImportRef,
    export: &VerifiedExportSignature,
) -> Result<Expr> {
    let export_name = export.name.as_dotted();
    import
        .certified_env_decls
        .iter()
        .find(|decl| decl.name() == export_name)
        .map(|decl| decl.ty().clone())
        .ok_or_else(|| {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidVerifiedImport,
                format!(
                    "verified import {} export {} is missing from the reconstructed kernel env",
                    import.module.as_dotted(),
                    export.name.as_dotted()
                ),
            )
        })
}

fn import_generated_export_type(
    import: &VerifiedImportRef,
    export: &VerifiedExportSignature,
) -> Result<Expr> {
    let export_name = export.name.as_dotted();
    for decl in &import.certified_env_decls {
        let Decl::Inductive { data, .. } = decl else {
            continue;
        };
        match export.kind {
            ExportKind::Constructor => {
                if let Some(constructor) = data
                    .constructors
                    .iter()
                    .find(|constructor| constructor.name == export_name)
                {
                    return Ok(constructor.ty.clone());
                }
            }
            ExportKind::Recursor => {
                if let Some(recursor) = data
                    .recursor
                    .as_ref()
                    .filter(|recursor| recursor.name == export_name)
                {
                    return Ok(recursor.ty.clone());
                }
            }
            _ => unreachable!("caller dispatches only generated export kinds"),
        }
    }

    Err(MachineTacticDiagnostic::new(
        MachineTacticDiagnosticKind::InvalidVerifiedImport,
        format!(
            "verified import {} generated export {} is missing from the reconstructed kernel env",
            import.module.as_dotted(),
            export.name.as_dotted()
        ),
    ))
}

fn expr_pi_telescope_len(expr: &Expr) -> usize {
    let mut len = 0;
    let mut current = expr;
    while let Expr::Pi { body, .. } = current {
        len += 1;
        current = body;
    }
    len
}

fn machine_term_elaboration_diag(err: npa_frontend::MachineDiagnostic) -> MachineTacticDiagnostic {
    match err.kind {
        npa_frontend::MachineDiagnosticKind::UnknownGlobalName
        | npa_frontend::MachineDiagnosticKind::ShortGlobalName
        | npa_frontend::MachineDiagnosticKind::AmbiguousGlobalName
        | npa_frontend::MachineDiagnosticKind::GlobalShadowedByLocal
        | npa_frontend::MachineDiagnosticKind::UnknownLocalName => {
            machine_term_frontend_diag_with_optional_name(
                MachineTacticDiagnosticKind::UnknownName,
                "machine term name resolution failed",
                &err,
            )
        }
        npa_frontend::MachineDiagnosticKind::ImplicitArgumentRequired
        | npa_frontend::MachineDiagnosticKind::MissingExplicitUniverse => {
            machine_term_frontend_diag_with_optional_name(
                MachineTacticDiagnosticKind::ImplicitArgumentRequired,
                "machine term requires explicit arguments",
                &err,
            )
        }
        npa_frontend::MachineDiagnosticKind::ExpectedFunctionType => MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::ExpectedFunctionType,
            format!("machine term expected a function type: {}", err.message),
        ),
        npa_frontend::MachineDiagnosticKind::TypeMismatch => {
            let mut diag = MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::TypeMismatch,
                format!("machine term type check failed: {}", err.message),
            );
            if let Some(payload) = err.payload {
                if let (Some(expected), Some(actual)) =
                    (payload.expected_hash.as_ref(), payload.actual_hash.as_ref())
                {
                    diag.expected_hash = Some(Box::new(*expected));
                    diag.actual_hash = Some(Box::new(*actual));
                }
            }
            diag
        }
        npa_frontend::MachineDiagnosticKind::ParseError
        | npa_frontend::MachineDiagnosticKind::UnsupportedSyntax
        | npa_frontend::MachineDiagnosticKind::HoleNotAllowed => MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidMachineTermSource,
            format!("machine term source is invalid: {}", err.message),
        ),
        npa_frontend::MachineDiagnosticKind::KernelRejected => {
            machine_term_frontend_diag_with_optional_name(
                MachineTacticDiagnosticKind::MachineTermElaborationError,
                "machine term elaboration was rejected by the kernel",
                &err,
            )
        }
        _ => machine_term_frontend_diag_with_optional_name(
            MachineTacticDiagnosticKind::MachineTermElaborationError,
            "machine term elaboration failed",
            &err,
        ),
    }
}

fn machine_term_frontend_diag_with_optional_name(
    kind: MachineTacticDiagnosticKind,
    prefix: &str,
    err: &npa_frontend::MachineDiagnostic,
) -> MachineTacticDiagnostic {
    let mut diag = MachineTacticDiagnostic::new(kind, format!("{prefix}: {}", err.message));
    if let Some(payload) = err.payload.as_ref() {
        if let Some(head_symbol) = payload.head_symbol.as_ref() {
            let name = Name::from_dotted(head_symbol);
            if name.is_canonical() {
                diag = diag.with_primary_name(name);
            }
        }
    }
    diag
}

fn canonicalize_imports(mut imports: Vec<VerifiedImportRef>) -> Vec<VerifiedImportRef> {
    imports.sort_by(|lhs, rhs| {
        lhs.module
            .cmp(&rhs.module)
            .then_with(|| lhs.export_hash.cmp(&rhs.export_hash))
            .then_with(|| lhs.certificate_hash.cmp(&rhs.certificate_hash))
    });
    imports
}

fn validate_imports(imports: &[VerifiedImportRef]) -> Result<()> {
    let mut import_keys = BTreeSet::new();
    let mut module_hashes = BTreeMap::new();
    let mut exported_names = BTreeSet::new();
    for import in imports {
        let key = (
            import.module.clone(),
            import.export_hash,
            import.certificate_hash,
        );
        if let Some((existing_export_hash, existing_certificate_hash)) =
            module_hashes.get(&import.module)
        {
            if *existing_export_hash != import.export_hash
                || *existing_certificate_hash != import.certificate_hash
            {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::InvalidVerifiedImport,
                    format!(
                        "verified imports contain multiple hashes for module {}",
                        import.module.as_dotted()
                    ),
                ));
            }
        } else {
            module_hashes.insert(
                import.module.clone(),
                (import.export_hash, import.certificate_hash),
            );
        }
        if !import_keys.insert(key) {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidVerifiedImport,
                format!("duplicate verified import {}", import.module.as_dotted()),
            ));
        }
        let mut module_exported_names = BTreeSet::new();
        for export in &import.exports {
            if !module_exported_names.insert(export.name.clone()) {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::InvalidVerifiedImport,
                    format!(
                        "verified import {} exports duplicate name {}",
                        import.module.as_dotted(),
                        export.name.as_dotted()
                    ),
                ));
            }
            if !exported_names.insert(export.name.clone()) {
                return Err(MachineTacticDiagnostic::new(
                    MachineTacticDiagnosticKind::InvalidVerifiedImport,
                    format!(
                        "verified imports export duplicate name {}",
                        export.name.as_dotted()
                    ),
                ));
            }
        }
    }
    Ok(())
}

fn validate_proof_spec(spec: &MachineProofSpec) -> Result<()> {
    if !spec.module.is_canonical() {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidMachineProofSpec,
            "proof module name is not canonical",
        ));
    }
    if !spec.theorem_name.is_canonical() {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidMachineProofSpec,
            "theorem name is not canonical",
        ));
    }
    if !name_is_in_module(&spec.theorem_name, &spec.module) {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidMachineProofSpec,
            format!(
                "theorem name {} is not in proof module {}",
                spec.theorem_name.as_dotted(),
                spec.module.as_dotted()
            ),
        ));
    }
    validate_universe_params(&spec.universe_params)?;
    Ok(())
}

fn validate_checked_current_decls_for_module(
    module: &ModuleName,
    theorem_name: &Name,
    checked_current_decls: &[CheckedCurrentDecl],
) -> Result<()> {
    for checked in checked_current_decls {
        let core_name = Name::from_dotted(checked.core_decl.name());
        if !name_is_in_module(&core_name, module) {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidCurrentDeclOrder,
                format!(
                    "checked current declaration {} is not in proof module {}",
                    core_name.as_dotted(),
                    module.as_dotted()
                ),
            ));
        }
        if &core_name == theorem_name || checked.signature.name() == theorem_name {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidCurrentDeclOrder,
                format!(
                    "checked current declaration {} duplicates theorem name {}",
                    core_name.as_dotted(),
                    theorem_name.as_dotted()
                ),
            ));
        }
    }
    Ok(())
}

fn name_is_in_module(name: &Name, module: &ModuleName) -> bool {
    name.0.len() > module.0.len() && name.0.starts_with(&module.0)
}

fn validate_universe_params(params: &[String]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for param in params {
        if param.is_empty() {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidMachineProofSpec,
                "universe parameter names must be nonempty",
            ));
        }
        if !seen.insert(param) {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidMachineProofSpec,
                format!("duplicate universe parameter {param}"),
            ));
        }
    }
    Ok(())
}

fn add_decl_to_kernel_env(env: &mut Env, decl: Decl) -> npa_kernel::Result<()> {
    match decl {
        Decl::Axiom {
            name,
            universe_params,
            ty,
        } => env.add_axiom(name, universe_params, ty),
        Decl::Def {
            name,
            universe_params,
            ty,
            value,
            reducibility,
        } => env.add_def(name, universe_params, ty, value, reducibility),
        Decl::Theorem {
            name,
            universe_params,
            ty,
            proof,
        } => env.add_theorem(name, universe_params, ty, proof),
        Decl::Inductive { data, .. } => env.add_inductive(*data),
        Decl::Constructor { .. } | Decl::Recursor { .. } => {
            Err(npa_kernel::Error::InvalidInductive(
                "generated declarations cannot be added directly".to_owned(),
            ))
        }
    }
}

fn verified_builtin_decl_matches_kernel_env(env: &Env, decl: &Decl) -> bool {
    match decl {
        Decl::Inductive { name, .. } if name == "Nat" || name == "Eq" => {
            let Ok(candidate) = kernel_env_from_single_decl(decl.clone()) else {
                return false;
            };
            let Some(names) = generated_decl_closure_names(decl) else {
                return false;
            };
            names.iter().all(|name| {
                let Some(existing) = env.decl(name) else {
                    return false;
                };
                let Some(candidate) = candidate.decl(name) else {
                    return false;
                };
                kernel_decl_interfaces_match(existing, candidate)
            })
        }
        Decl::Axiom {
            name,
            universe_params,
            ty,
        } if name == "Eq.rec" => {
            env.decl("Eq.rec").is_some()
                && universe_params.len() == 2
                && universe_params[0] == "u"
                && universe_params[1] == "v"
                && core_expr_hash(ty)
                    == core_expr_hash(&npa_kernel::eq_rec_type(
                        Level::param("u"),
                        Level::param("v"),
                    ))
        }
        _ => false,
    }
}

fn kernel_env_from_single_decl(decl: Decl) -> npa_kernel::Result<Env> {
    let mut env = Env::new();
    add_decl_to_kernel_env(&mut env, decl)?;
    Ok(env)
}

fn generated_decl_closure_names(decl: &Decl) -> Option<Vec<String>> {
    let Decl::Inductive { data, .. } = decl else {
        return None;
    };
    let mut names = Vec::new();
    names.push(data.name.clone());
    names.extend(
        data.constructors
            .iter()
            .map(|constructor| constructor.name.clone()),
    );
    if let Some(recursor) = &data.recursor {
        names.push(recursor.name.clone());
    }
    Some(names)
}

fn kernel_decl_interfaces_match(lhs: &Decl, rhs: &Decl) -> bool {
    match (lhs, rhs) {
        (
            Decl::Axiom {
                name: lhs_name,
                universe_params: lhs_params,
                ty: lhs_ty,
            },
            Decl::Axiom {
                name: rhs_name,
                universe_params: rhs_params,
                ty: rhs_ty,
            },
        ) => {
            lhs_name == rhs_name
                && lhs_params == rhs_params
                && core_expr_hash(lhs_ty) == core_expr_hash(rhs_ty)
        }
        (
            Decl::Inductive {
                name: lhs_name,
                universe_params: lhs_params,
                ty: lhs_ty,
                data: lhs_data,
            },
            Decl::Inductive {
                name: rhs_name,
                universe_params: rhs_params,
                ty: rhs_ty,
                data: rhs_data,
            },
        ) => {
            lhs_name == rhs_name
                && lhs_params == rhs_params
                && core_expr_hash(lhs_ty) == core_expr_hash(rhs_ty)
                && inductive_interfaces_match(lhs_data, rhs_data)
        }
        (
            Decl::Constructor {
                name: lhs_name,
                universe_params: lhs_params,
                ty: lhs_ty,
                inductive: lhs_inductive,
            },
            Decl::Constructor {
                name: rhs_name,
                universe_params: rhs_params,
                ty: rhs_ty,
                inductive: rhs_inductive,
            },
        ) => {
            lhs_name == rhs_name
                && lhs_params == rhs_params
                && lhs_inductive == rhs_inductive
                && core_expr_hash(lhs_ty) == core_expr_hash(rhs_ty)
        }
        (
            Decl::Recursor {
                name: lhs_name,
                universe_params: lhs_params,
                ty: lhs_ty,
                inductive: lhs_inductive,
                rules: lhs_rules,
            },
            Decl::Recursor {
                name: rhs_name,
                universe_params: rhs_params,
                ty: rhs_ty,
                inductive: rhs_inductive,
                rules: rhs_rules,
            },
        ) => {
            lhs_name == rhs_name
                && lhs_params == rhs_params
                && lhs_inductive == rhs_inductive
                && lhs_rules == rhs_rules
                && core_expr_hash(lhs_ty) == core_expr_hash(rhs_ty)
        }
        _ => false,
    }
}

fn inductive_interfaces_match(
    lhs: &npa_kernel::InductiveDecl,
    rhs: &npa_kernel::InductiveDecl,
) -> bool {
    lhs.name == rhs.name
        && lhs.universe_params == rhs.universe_params
        && lhs.sort == rhs.sort
        && binder_type_hashes(&lhs.params) == binder_type_hashes(&rhs.params)
        && binder_type_hashes(&lhs.indices) == binder_type_hashes(&rhs.indices)
        && constructor_interfaces_match(&lhs.constructors, &rhs.constructors)
        && recursor_interfaces_match(&lhs.recursor, &rhs.recursor, lhs)
}

fn binder_type_hashes(binders: &[npa_kernel::Binder]) -> Vec<Hash> {
    binders
        .iter()
        .map(|binder| core_expr_hash(&binder.ty))
        .collect()
}

fn constructor_interfaces_match(
    lhs: &[npa_kernel::ConstructorDecl],
    rhs: &[npa_kernel::ConstructorDecl],
) -> bool {
    lhs.len() == rhs.len()
        && lhs.iter().zip(rhs).all(|(lhs, rhs)| {
            lhs.name == rhs.name && core_expr_hash(&lhs.ty) == core_expr_hash(&rhs.ty)
        })
}

fn recursor_interfaces_match(
    lhs: &Option<npa_kernel::RecursorDecl>,
    rhs: &Option<npa_kernel::RecursorDecl>,
    data: &npa_kernel::InductiveDecl,
) -> bool {
    match (lhs, rhs) {
        (Some(lhs), Some(rhs)) => {
            lhs.name == rhs.name
                && lhs.universe_params == rhs.universe_params
                && core_expr_hash(&lhs.ty) == core_expr_hash(&rhs.ty)
                && effective_recursor_rules(lhs, data) == effective_recursor_rules(rhs, data)
        }
        (None, None) => true,
        _ => false,
    }
}

fn effective_recursor_rules(
    recursor: &npa_kernel::RecursorDecl,
    data: &npa_kernel::InductiveDecl,
) -> npa_kernel::RecursorRules {
    recursor.rules.clone().unwrap_or_else(|| {
        let minor_start = data.params.len() + 1;
        npa_kernel::RecursorRules::new(minor_start, minor_start + data.constructors.len())
    })
}

fn ensure_type_is_sort(
    env: &Env,
    ctx: &Ctx,
    delta: &[String],
    term: &Expr,
) -> npa_kernel::Result<()> {
    match env.whnf(ctx, delta, &env.infer(ctx, delta, term)?)? {
        Expr::Sort(_) => Ok(()),
        actual => Err(npa_kernel::Error::ExpectedSort { actual }),
    }
}

fn local_context_to_ctx_with_budget(
    env: &Env,
    context: &[MachineLocalDecl],
    universe_params: &[String],
    fuel: &TacticRunFuel,
    goal_id: GoalId,
    meta_id: MetaVarId,
) -> Result<Ctx> {
    let mut ctx = Ctx::new();
    let mut names = BTreeSet::new();
    for local in context {
        if local.name.is_empty() || !names.insert(local.name.clone()) {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidMetaContext,
                format!("invalid or duplicate local name {}", local.name),
            ));
        }
        kernel_expect_sort_with_budget(
            env,
            &ctx,
            universe_params,
            &local.ty,
            fuel,
            goal_id,
            meta_id,
        )?;
        match &local.value {
            Some(value) => {
                kernel_check_with_budget(
                    env,
                    &ctx,
                    universe_params,
                    value,
                    &local.ty,
                    fuel,
                    goal_id,
                    meta_id,
                )?;
                ctx.push_definition(local.name.clone(), local.ty.clone(), value.clone());
            }
            None => ctx.push_assumption(local.name.clone(), local.ty.clone()),
        }
    }
    Ok(ctx)
}

pub fn machine_local_context_is_prefix(
    prefix: &[MachineLocalDecl],
    full: &[MachineLocalDecl],
) -> bool {
    prefix.len() <= full.len() && prefix.iter().zip(full).all(|(lhs, rhs)| lhs == rhs)
}

fn kernel_diag(err: npa_kernel::Error) -> MachineTacticDiagnostic {
    match err {
        npa_kernel::Error::InvalidBVar(_) => MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::ProofExprScopeError,
            format!("kernel rejected proof expression scope: {err:?}"),
        ),
        npa_kernel::Error::TypeMismatch { expected, actual } => {
            let message = format!(
                "kernel rejected proof expression type: {:?}",
                npa_kernel::Error::TypeMismatch {
                    expected: expected.clone(),
                    actual: actual.clone()
                }
            );
            MachineTacticDiagnostic::new(MachineTacticDiagnosticKind::TypeMismatch, message)
                .with_expected_actual_payloads(
                    DiagnosticPayloadKind::Expr,
                    &core_expr_canonical_bytes(&expected),
                    &core_expr_canonical_bytes(&actual),
                )
        }
        _ => MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::KernelRejected,
            format!("kernel rejected proof expression: {err:?}"),
        ),
    }
}

fn theorem_type_diag(err: npa_kernel::Error) -> MachineTacticDiagnostic {
    match err {
        npa_kernel::Error::ExpectedSort { actual } => MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::TypeMismatch,
            format!("theorem type must have a sort type, got {actual:?}"),
        ),
        npa_kernel::Error::ExpectedPi { actual } => MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::ExpectedFunctionType,
            format!("theorem type contains an application whose function has type {actual:?}"),
        ),
        npa_kernel::Error::TypeMismatch { expected, actual } => {
            let message = format!(
                "kernel rejected theorem type: {:?}",
                npa_kernel::Error::TypeMismatch {
                    expected: expected.clone(),
                    actual: actual.clone()
                }
            );
            MachineTacticDiagnostic::new(MachineTacticDiagnosticKind::TypeMismatch, message)
                .with_expected_actual_payloads(
                    DiagnosticPayloadKind::Expr,
                    &core_expr_canonical_bytes(&expected),
                    &core_expr_canonical_bytes(&actual),
                )
        }
        npa_kernel::Error::InvalidBVar(_) => MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::ProofExprScopeError,
            format!("kernel rejected theorem type scope: {err:?}"),
        ),
        _ => MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::KernelRejected,
            format!("kernel rejected theorem type: {err:?}"),
        ),
    }
}

fn contains_bound_var(expr: &Expr, target: u32) -> bool {
    match expr {
        Expr::Sort(_) | Expr::Const { .. } => false,
        Expr::BVar(index) => *index == target,
        Expr::App(fun, arg) => contains_bound_var(fun, target) || contains_bound_var(arg, target),
        Expr::Lam { ty, body, .. } | Expr::Pi { ty, body, .. } => {
            contains_bound_var(ty, target) || contains_bound_var(body, target + 1)
        }
        Expr::Let {
            ty, value, body, ..
        } => {
            contains_bound_var(ty, target)
                || contains_bound_var(value, target)
                || contains_bound_var(body, target + 1)
        }
    }
}

fn core_expr_node_count(expr: &Expr) -> u64 {
    match expr {
        Expr::Sort(_) | Expr::BVar(_) | Expr::Const { .. } => 1,
        Expr::App(fun, arg) => 1 + core_expr_node_count(fun) + core_expr_node_count(arg),
        Expr::Lam { ty, body, .. } | Expr::Pi { ty, body, .. } => {
            1 + core_expr_node_count(ty) + core_expr_node_count(body)
        }
        Expr::Let {
            ty, value, body, ..
        } => {
            1 + core_expr_node_count(ty) + core_expr_node_count(value) + core_expr_node_count(body)
        }
    }
}

fn proof_expr_node_count(expr: &ProofExpr) -> u64 {
    match expr {
        ProofExpr::Core(core) => 1 + core_expr_node_count(core),
        ProofExpr::Meta(_) => 1,
        ProofExpr::App(fun, arg) => 1 + proof_expr_node_count(fun) + proof_expr_node_count(arg),
        ProofExpr::Lam { ty, body, .. } => {
            1 + core_expr_node_count(ty) + proof_expr_node_count(body)
        }
        ProofExpr::Let {
            ty, value, body, ..
        } => {
            1 + core_expr_node_count(ty)
                + proof_expr_node_count(value)
                + proof_expr_node_count(body)
        }
    }
}

fn phase2_current_decl_hashes(
    imports: &[VerifiedImportRef],
    checked_prior_current_decls: &[CheckedCurrentDecl],
    decl: &Decl,
) -> Result<DeclHashes> {
    let module_name = infer_current_module_name(decl)?;
    let declarations = checked_prior_current_decls
        .iter()
        .map(|checked| checked.core_decl.clone())
        .chain(std::iter::once(decl.clone()))
        .collect();
    let verified_imports = imports
        .iter()
        .map(|import| import.verified_module.as_ref().clone())
        .collect::<Vec<_>>();
    let cert = npa_cert::build_module_cert(
        CoreModule {
            name: module_name,
            declarations,
        },
        &verified_imports,
    )
    .map_err(|err| {
        MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::UncheckedCurrentDecl,
            format!(
                "Phase 2 rejected current declaration interface hash materialization for {}: {err:?}",
                decl.name()
            ),
        )
    })?;
    let target_name = Name::from_dotted(decl.name());
    for cert_decl in &cert.declarations {
        if decl_payload_name(&cert, &cert_decl.decl)? == target_name {
            return Ok(cert_decl.hashes.clone());
        }
    }
    Err(MachineTacticDiagnostic::new(
        MachineTacticDiagnosticKind::CurrentDeclSignatureMismatch,
        format!(
            "Phase 2 materialization did not return current declaration {}",
            decl.name()
        ),
    ))
}

fn infer_current_module_name(decl: &Decl) -> Result<ModuleName> {
    let name = Name::from_dotted(decl.name());
    if !name.is_canonical() || name.0.len() < 2 {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::UncheckedCurrentDecl,
            format!(
                "current declaration {} must be a canonical qualified module declaration",
                decl.name()
            ),
        ));
    }
    Ok(Name(name.0[..name.0.len() - 1].to_vec()))
}

fn decl_payload_name(cert: &npa_cert::ModuleCert, payload: &DeclPayload) -> Result<Name> {
    let name_id = match payload {
        DeclPayload::Axiom { name, .. }
        | DeclPayload::Def { name, .. }
        | DeclPayload::Theorem { name, .. }
        | DeclPayload::Inductive { name, .. } => *name,
    };
    cert.name_table.get(name_id).cloned().ok_or_else(|| {
        MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::CurrentDeclSignatureMismatch,
            "Phase 2 materialization returned a declaration with an invalid name id",
        )
    })
}

fn verified_export_signature_hash(export: &VerifiedExportSignature) -> Hash {
    let mut out = Vec::new();
    encode_name_to(&mut out, &export.name);
    encode_export_kind_to(&mut out, export.kind);
    encode_hash_to(&mut out, &export.decl_interface_hash);
    encode_hash_to(&mut out, &export.type_hash);
    encode_option_hash_to(&mut out, export.body_hash.as_ref());
    hash_with_domain("npa.phase4.verified-export-signature.v1", &out)
}

fn checked_current_chain_fingerprint(checked_current_decls: &[CheckedCurrentDecl]) -> Hash {
    let mut out = Vec::new();
    encode_list_len_to(&mut out, checked_current_decls.len());
    for decl in checked_current_decls {
        encode_u64_to(&mut out, decl.source_index);
        encode_hash_to(&mut out, &checked_decl_signature_hash(&decl.signature));
        encode_hash_to(&mut out, &decl.core_decl_hash);
        encode_hash_to(&mut out, &decl.prior_chain_fingerprint);
        encode_hash_to(&mut out, &decl.checked_env_fingerprint);
    }
    hash_with_domain("npa.phase4.current.prior-chain.v1", &out)
}

fn checked_env_fingerprint(
    imports: &[VerifiedImportRef],
    checked_current_decls: &[CheckedCurrentDecl],
) -> Hash {
    let mut out = Vec::new();
    encode_list_len_to(&mut out, imports.len());
    for import in imports {
        encode_name_to(&mut out, &import.module);
        encode_hash_to(&mut out, &import.export_hash);
        encode_hash_to(&mut out, &import.certificate_hash);
        let mut export_signature_hashes = import
            .exports
            .iter()
            .map(verified_export_signature_hash)
            .collect::<Vec<_>>();
        export_signature_hashes.sort();
        encode_list_len_to(&mut out, export_signature_hashes.len());
        for hash in export_signature_hashes {
            encode_hash_to(&mut out, &hash);
        }
        encode_list_len_to(&mut out, import.certified_env_decl_hashes.len());
        for hash in &import.certified_env_decl_hashes {
            encode_hash_to(&mut out, hash);
        }
    }
    encode_hash_to(
        &mut out,
        &checked_current_chain_fingerprint(checked_current_decls),
    );
    encode_list_len_to(&mut out, checked_current_decls.len());
    for decl in checked_current_decls {
        encode_u64_to(&mut out, decl.source_index);
        encode_hash_to(&mut out, &checked_decl_signature_hash(&decl.signature));
        encode_hash_to(&mut out, &decl.core_decl_hash);
        encode_hash_to(&mut out, &decl.checked_env_fingerprint);
    }
    encode_hash_to(&mut out, &kernel_check_profile_hash());
    hash_with_domain("npa.phase4.current.checked-env.v1", &out)
}

fn machine_term_source_canonical_bytes_from_phase3(phase3_canonical_bytes: &[u8]) -> Vec<u8> {
    let mut out = tagged_bytes("npa.phase4.machine-term-source.v1");
    out.extend_from_slice(phase3_canonical_bytes);
    out
}

fn machine_term_source_hash_from_phase3(phase3_canonical_bytes: &[u8]) -> Hash {
    hash_with_domain(
        "npa.phase4.machine-term-source.hash.v1",
        &machine_term_source_canonical_bytes_from_phase3(phase3_canonical_bytes),
    )
}

fn expected_actual_diagnostic_hash(
    kind: &MachineTacticDiagnosticKind,
    side: DiagnosticHashSide,
    payload_kind: DiagnosticPayloadKind,
    payload: &[u8],
) -> Hash {
    let mut out = tagged_bytes("npa.phase4.diagnostic.expected-actual.v1");
    encode_diagnostic_kind_to(&mut out, kind);
    encode_diagnostic_hash_side_to(&mut out, side);
    encode_diagnostic_payload_kind_to(&mut out, payload_kind);
    encode_list_len_to(&mut out, payload.len());
    out.extend_from_slice(payload);
    hash_with_domain("npa.phase4.diagnostic.expected-actual.hash.v1", &out)
}

pub fn meta_var_id_canonical_bytes(id: MetaVarId) -> Vec<u8> {
    let mut out = tagged_bytes("npa.phase4.meta-var-id.v1");
    encode_u64_to(&mut out, id.0);
    out
}

pub fn goal_id_canonical_bytes(id: GoalId) -> Vec<u8> {
    let mut out = tagged_bytes("npa.phase4.goal-id.v1");
    encode_u64_to(&mut out, id.0);
    out
}

pub fn proof_expr_canonical_bytes(expr: &ProofExpr) -> Vec<u8> {
    let mut out = tagged_bytes("npa.phase4.proof-expr.v1");
    encode_proof_expr_to(&mut out, expr);
    out
}

pub fn proof_expr_hash(expr: &ProofExpr) -> Hash {
    hash_with_domain(
        "npa.phase4.proof-expr.hash.v1",
        &proof_expr_canonical_bytes(expr),
    )
}

pub fn machine_local_decl_canonical_bytes(local: &MachineLocalDecl) -> Vec<u8> {
    let mut out = tagged_bytes("npa.phase4.machine-local-decl.v1");
    encode_machine_local_decl_to(&mut out, local);
    out
}

pub fn machine_local_decl_hash(local: &MachineLocalDecl) -> Hash {
    hash_with_domain(
        "npa.phase4.machine-local-decl.hash.v1",
        &machine_local_decl_canonical_bytes(local),
    )
}

pub fn machine_local_context_canonical_bytes(context: &[MachineLocalDecl]) -> Vec<u8> {
    let mut out = tagged_bytes("npa.phase4.machine-local-context.v1");
    encode_list_len_to(&mut out, context.len());
    for local in context {
        encode_machine_local_decl_to(&mut out, local);
    }
    out
}

pub fn machine_local_context_hash(context: &[MachineLocalDecl]) -> Hash {
    hash_with_domain(
        "npa.phase4.machine-local-context.hash.v1",
        &machine_local_context_canonical_bytes(context),
    )
}

pub fn checked_decl_signature_canonical_bytes(signature: &CheckedDeclSignature) -> Vec<u8> {
    let mut out = tagged_bytes("npa.phase4.checked-decl-signature.v1");
    encode_checked_decl_signature_to(&mut out, signature);
    out
}

pub fn checked_decl_signature_hash(signature: &CheckedDeclSignature) -> Hash {
    hash_with_domain(
        "npa.phase4.checked-decl-signature.hash.v1",
        &checked_decl_signature_canonical_bytes(signature),
    )
}

pub fn core_expr_hash(expr: &Expr) -> Hash {
    npa_cert::core_expr_hash(expr)
}

pub fn core_expr_canonical_bytes(expr: &Expr) -> Vec<u8> {
    npa_cert::core_expr_canonical_bytes(expr)
}

pub fn machine_term_source_canonical_bytes(term: &MachineTermSource) -> Result<Vec<u8>> {
    let canonical =
        npa_frontend::canonicalize_machine_term_source(term.source()).map_err(|err| {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidMachineTermSource,
                format!(
                    "machine term source canonicalization failed: {}",
                    err.message
                ),
            )
        })?;
    Ok(machine_term_source_canonical_bytes_from_phase3(
        &canonical.canonical_bytes,
    ))
}

pub fn machine_term_source_hash(term: &MachineTermSource) -> Result<Hash> {
    Ok(hash_with_domain(
        "npa.phase4.machine-term-source.hash.v1",
        &machine_term_source_canonical_bytes(term)?,
    ))
}

pub fn machine_tactic_canonical_bytes(tactic: &MachineTactic) -> Vec<u8> {
    let mut out = tagged_bytes("npa.phase4.machine-tactic.v1");
    encode_machine_tactic_to(&mut out, tactic);
    out
}

pub fn machine_tactic_hash(tactic: &MachineTactic) -> Hash {
    hash_with_domain(
        "npa.phase4.machine-tactic.hash.v1",
        &machine_tactic_canonical_bytes(tactic),
    )
}

pub fn tactic_budget_canonical_bytes(budget: TacticBudget) -> Vec<u8> {
    let mut out = tagged_bytes("npa.phase4.tactic-budget.v1");
    encode_u64_to(&mut out, budget.max_tactic_steps);
    encode_u64_to(&mut out, budget.max_whnf_steps);
    encode_u64_to(&mut out, budget.max_conversion_steps);
    encode_u64_to(&mut out, budget.max_rewrite_steps);
    encode_u64_to(&mut out, budget.max_meta_allocations);
    encode_u64_to(&mut out, budget.max_expr_nodes);
    out
}

pub fn tactic_budget_hash(budget: TacticBudget) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(tactic_budget_canonical_bytes(budget));
    hasher.finalize().into()
}

pub fn machine_tactic_cache_key(
    state: &MachineProofState,
    tactic: &MachineTactic,
    budget: TacticBudget,
) -> MachineTacticCacheKey {
    MachineTacticCacheKey {
        state_fingerprint: state.fingerprint,
        goal_id: machine_tactic_goal_id(tactic),
        tactic_hash: machine_tactic_hash(tactic),
        deterministic_budget_hash: tactic_budget_hash(budget),
    }
}

pub fn machine_tactic_cache_key_canonical_bytes(key: &MachineTacticCacheKey) -> Vec<u8> {
    let mut out = tagged_bytes("npa.phase4.machine-tactic-cache-key.v1");
    encode_hash_to(&mut out, &key.state_fingerprint);
    encode_goal_id_to(&mut out, key.goal_id);
    encode_hash_to(&mut out, &key.tactic_hash);
    encode_hash_to(&mut out, &key.deterministic_budget_hash);
    out
}

pub fn machine_tactic_cache_key_hash(key: &MachineTacticCacheKey) -> Hash {
    hash_with_domain(
        "npa.phase4.machine-tactic-cache-key.hash.v1",
        &machine_tactic_cache_key_canonical_bytes(key),
    )
}

fn universe_param_list_canonical_bytes(params: &[String]) -> Vec<u8> {
    let mut out = tagged_bytes("npa.phase4.universe-param-list.v1");
    encode_list_len_to(&mut out, params.len());
    for param in params {
        encode_string_to(&mut out, param);
    }
    out
}

fn level_arg_list_canonical_bytes(levels: &[Level]) -> Vec<u8> {
    let mut out = tagged_bytes("npa.phase4.level-arg-list.v1");
    encode_list_len_to(&mut out, levels.len());
    for level in levels {
        encode_level_to(&mut out, level);
    }
    out
}

pub fn machine_tactic_options_hash(options: &MachineTacticOptions) -> Hash {
    let mut out = tagged_bytes("npa.phase4.tactic-options.v1");
    encode_machine_tactic_options_to(&mut out, options);
    hash_with_domain("npa.phase4.machine-tactic-options.v1", &out)
}

pub fn simp_registry_hash(registry: &SimpRegistry) -> Hash {
    let mut out = Vec::new();
    encode_simp_registry_to(&mut out, registry);
    hash_with_domain("npa.phase4.simp-registry.v1", &out)
}

pub fn machine_proof_delta_hash(delta: &MachineProofDelta) -> Hash {
    let mut out = Vec::new();
    encode_hash_to(&mut out, &delta.from_state_fingerprint);
    encode_goal_id_to(&mut out, delta.assigned_goal);
    encode_meta_var_id_to(&mut out, delta.assigned_meta);
    encode_hash_to(&mut out, &delta.proof_expr_hash);
    encode_list_len_to(&mut out, delta.added_goals.len());
    for goal in &delta.added_goals {
        encode_goal_id_to(&mut out, *goal);
    }
    encode_list_len_to(&mut out, delta.new_metas.len());
    for meta in &delta.new_metas {
        encode_meta_var_id_to(&mut out, meta.meta_id);
        encode_goal_id_to(&mut out, meta.goal_id);
        encode_hash_to(&mut out, &meta.context_hash);
        encode_hash_to(&mut out, &meta.target_hash);
    }
    encode_hash_to(&mut out, &delta.to_state_fingerprint);
    hash_with_domain("npa.phase4.machine-proof-delta.v1", &out)
}

fn machine_tactic_env_hash(env: &MachineTacticEnv) -> Hash {
    let mut out = Vec::new();
    encode_list_len_to(&mut out, env.imports.len());
    for import in &env.imports {
        encode_name_to(&mut out, &import.module);
        encode_hash_to(&mut out, &import.export_hash);
        encode_hash_to(&mut out, &import.certificate_hash);
        encode_list_len_to(&mut out, import.exports.len());
        for export in &import.exports {
            encode_name_to(&mut out, &export.name);
            encode_export_kind_to(&mut out, export.kind);
            encode_hash_to(&mut out, &export.decl_interface_hash);
            encode_hash_to(&mut out, &export.type_hash);
            encode_option_hash_to(&mut out, export.body_hash.as_ref());
        }
        encode_list_len_to(&mut out, import.certified_env_decl_hashes.len());
        for hash in &import.certified_env_decl_hashes {
            encode_hash_to(&mut out, hash);
        }
    }
    encode_list_len_to(&mut out, env.checked_current_decls.len());
    for decl in &env.checked_current_decls {
        encode_u64_to(&mut out, decl.source_index);
        encode_checked_decl_signature_to(&mut out, &decl.signature);
        encode_hash_to(&mut out, &decl.prior_chain_fingerprint);
        encode_hash_to(&mut out, &decl.checked_env_fingerprint);
        encode_hash_to(&mut out, &decl.core_decl_hash);
    }
    encode_hash_to(&mut out, &simp_registry_hash(&env.simp_registry));
    encode_option_resolved_eq_to(&mut out, env.eq_family.as_ref());
    encode_option_resolved_nat_to(&mut out, env.nat_family.as_ref());
    encode_hash_to(&mut out, &env.options_fingerprint);
    encode_hash_to(&mut out, &kernel_check_profile_hash());
    hash_with_domain("npa.phase4.machine-tactic-env.v1", &out)
}

fn machine_proof_state_hash(state: &MachineProofState) -> Hash {
    let mut out = Vec::new();
    encode_name_to(&mut out, &state.root.module);
    encode_name_to(&mut out, &state.root.theorem_name);
    encode_u64_to(&mut out, state.root.source_index);
    encode_list_len_to(&mut out, state.root.universe_params.len());
    for param in &state.root.universe_params {
        encode_string_to(&mut out, param);
    }
    encode_hash_to(&mut out, &core_expr_hash(&state.root.theorem_type));
    encode_hash_to(&mut out, &proof_expr_hash(&state.root.body));
    encode_list_len_to(&mut out, state.open_goals.len());
    for goal in &state.open_goals {
        encode_goal_id_to(&mut out, *goal);
    }
    encode_u64_to(&mut out, state.metas.next_id);
    encode_list_len_to(&mut out, state.metas.metas.len());
    for (meta_id, meta) in &state.metas.metas {
        encode_meta_var_id_to(&mut out, *meta_id);
        encode_goal_id_to(&mut out, meta.goal_id);
        encode_hash_to(&mut out, &machine_local_context_hash(&meta.context));
        encode_hash_to(&mut out, &core_expr_hash(&meta.target));
        encode_option_proof_expr_to(&mut out, meta.assignment.as_ref());
        encode_u64_to(&mut out, meta.creation_index);
    }
    encode_hash_to(&mut out, &state.env.env_fingerprint);
    encode_list_len_to(&mut out, state.reserved_local_names.len());
    for name in &state.reserved_local_names {
        encode_string_to(&mut out, name);
    }
    encode_hash_to(&mut out, &kernel_check_profile_hash());
    hash_with_domain("npa.phase4.machine-proof-state.v1", &out)
}

fn kernel_check_profile_hash() -> Hash {
    let mut out = Vec::new();
    encode_string_to(&mut out, "core-spec-v0.1");
    encode_string_to(&mut out, "npa-kernel.phase1.v0.1");
    encode_string_to(&mut out, "beta-delta-iota-zeta.v0.1");
    encode_string_to(&mut out, "levels-imax-v0.1");
    encode_string_to(&mut out, "builtin-nat-eq-rec-v0.1");
    hash_with_domain("npa.phase4.kernel-check-profile.v1", &out)
}

fn encode_diagnostic_kind_to(out: &mut Vec<u8>, kind: &MachineTacticDiagnosticKind) {
    out.push(match kind {
        MachineTacticDiagnosticKind::InvalidTacticOption => 0x00,
        MachineTacticDiagnosticKind::InvalidBatchPolicy => 0x2d,
        MachineTacticDiagnosticKind::UnsupportedTacticOption => 0x01,
        MachineTacticDiagnosticKind::InvalidMachineTactic => 0x15,
        MachineTacticDiagnosticKind::InvalidMachineTermSource => 0x16,
        MachineTacticDiagnosticKind::MachineTermElaborationError => 0x32,
        MachineTacticDiagnosticKind::UnknownName => 0x33,
        MachineTacticDiagnosticKind::ImplicitArgumentRequired => 0x34,
        MachineTacticDiagnosticKind::UnsupportedMachineTactic => 0x02,
        MachineTacticDiagnosticKind::TacticFuelExhausted { kind } => {
            out.push(0x14);
            encode_tactic_fuel_kind_to(out, *kind);
            return;
        }
        MachineTacticDiagnosticKind::InvalidMachineProofState => 0x03,
        MachineTacticDiagnosticKind::InvalidMachineProofSpec => 0x04,
        MachineTacticDiagnosticKind::InvalidVerifiedImport => 0x05,
        MachineTacticDiagnosticKind::AmbiguousKernelEnvDecl => 0x06,
        MachineTacticDiagnosticKind::InvalidCurrentDeclOrder => 0x07,
        MachineTacticDiagnosticKind::CurrentDeclSignatureMismatch => 0x08,
        MachineTacticDiagnosticKind::UnknownGoal => 0x09,
        MachineTacticDiagnosticKind::GoalAlreadyAssigned => 0x0a,
        MachineTacticDiagnosticKind::UnknownMeta => 0x0d,
        MachineTacticDiagnosticKind::GoalLimitExceeded => 0x0b,
        MachineTacticDiagnosticKind::MetaLimitExceeded => 0x0c,
        MachineTacticDiagnosticKind::InvalidMetaDependency => 0x0e,
        MachineTacticDiagnosticKind::InvalidMetaContext => 0x0f,
        MachineTacticDiagnosticKind::ProofExprScopeError => 0x10,
        MachineTacticDiagnosticKind::ProofExprTypeMismatch => 0x2e,
        MachineTacticDiagnosticKind::UnknownTacticHead => 0x17,
        MachineTacticDiagnosticKind::AmbiguousTacticHead => 0x18,
        MachineTacticDiagnosticKind::UnknownLocalName => 0x19,
        MachineTacticDiagnosticKind::AmbiguousLocalName => 0x1a,
        MachineTacticDiagnosticKind::InvalidLocalHead => 0x1b,
        MachineTacticDiagnosticKind::ExpectedFunctionType => 0x2f,
        MachineTacticDiagnosticKind::ExpectedPiTarget => 0x30,
        MachineTacticDiagnosticKind::UniverseArgumentMismatch => 0x1c,
        MachineTacticDiagnosticKind::MissingExplicitArgument => 0x1d,
        MachineTacticDiagnosticKind::AmbiguousApplyArgument => 0x1e,
        MachineTacticDiagnosticKind::TooManyApplyArguments => 0x1f,
        MachineTacticDiagnosticKind::TooFewApplyArguments => 0x20,
        MachineTacticDiagnosticKind::SubgoalDataArgument => 0x21,
        MachineTacticDiagnosticKind::ExpectedEqTarget => 0x22,
        MachineTacticDiagnosticKind::UnknownSimpRule => 0x23,
        MachineTacticDiagnosticKind::AmbiguousSimpRule => 0x24,
        MachineTacticDiagnosticKind::InvalidSimpRule => 0x25,
        MachineTacticDiagnosticKind::SimpNoProgress => 0x26,
        MachineTacticDiagnosticKind::SimpStepLimitExceeded => 0x27,
        MachineTacticDiagnosticKind::AmbiguousRewriteRule => 0x28,
        MachineTacticDiagnosticKind::TacticPrimitiveUnavailable => 0x29,
        MachineTacticDiagnosticKind::InvalidEqFamily => 0x2a,
        MachineTacticDiagnosticKind::InvalidNatFamily => 0x2b,
        MachineTacticDiagnosticKind::InvalidInductionTarget => 0x2c,
        MachineTacticDiagnosticKind::UncheckedCurrentDecl => 0x31,
        MachineTacticDiagnosticKind::TypeMismatch => 0x11,
        MachineTacticDiagnosticKind::KernelRejected => 0x12,
        MachineTacticDiagnosticKind::UnresolvedGoal => 0x13,
    });
}

fn encode_tactic_fuel_kind_to(out: &mut Vec<u8>, kind: TacticFuelKind) {
    out.push(match kind {
        TacticFuelKind::TacticStep => 0x00,
        TacticFuelKind::Whnf => 0x01,
        TacticFuelKind::Conversion => 0x02,
        TacticFuelKind::Rewrite => 0x03,
        TacticFuelKind::MetaAllocation => 0x04,
        TacticFuelKind::ExprNode => 0x05,
    });
}

fn encode_diagnostic_hash_side_to(out: &mut Vec<u8>, side: DiagnosticHashSide) {
    out.push(match side {
        DiagnosticHashSide::Expected => 0x00,
        DiagnosticHashSide::Actual => 0x01,
    });
}

fn encode_diagnostic_payload_kind_to(out: &mut Vec<u8>, kind: DiagnosticPayloadKind) {
    out.push(match kind {
        DiagnosticPayloadKind::Expr => 0x00,
        DiagnosticPayloadKind::CheckedDeclSignature => 0x01,
        DiagnosticPayloadKind::MachineTermSource => 0x02,
        DiagnosticPayloadKind::UniverseParamList => 0x03,
        DiagnosticPayloadKind::LevelArgList => 0x04,
    });
}

fn encode_core_expr_bytes_to(out: &mut Vec<u8>, expr: &Expr) {
    out.extend_from_slice(&core_expr_canonical_bytes(expr));
}

fn encode_level_to(out: &mut Vec<u8>, level: &Level) {
    match normalize_level(level.clone()) {
        Level::Zero => out.push(0x00),
        Level::Succ(inner) => {
            out.push(0x01);
            encode_level_to(out, &inner);
        }
        Level::Max(lhs, rhs) => {
            out.push(0x02);
            encode_level_to(out, &lhs);
            encode_level_to(out, &rhs);
        }
        Level::IMax(lhs, rhs) => {
            out.push(0x03);
            encode_level_to(out, &lhs);
            encode_level_to(out, &rhs);
        }
        Level::Param(name) => {
            out.push(0x04);
            encode_name_to(out, &Name::from_dotted(&name));
        }
    }
}

fn encode_machine_tactic_to(out: &mut Vec<u8>, tactic: &MachineTactic) {
    match tactic {
        MachineTactic::Exact { term, .. } => {
            out.push(0x00);
            encode_hash_to(out, &term.canonical_hash());
        }
        MachineTactic::Intro { name, .. } => {
            out.push(0x01);
            encode_string_to(out, name);
        }
        MachineTactic::Apply {
            head,
            universe_args,
            args,
            ..
        } => {
            out.push(0x02);
            encode_tactic_head_to(out, head);
            encode_list_len_to(out, universe_args.len());
            for level in universe_args {
                encode_level_to(out, level);
            }
            encode_list_len_to(out, args.len());
            for arg in args {
                encode_apply_arg_to(out, arg);
            }
        }
        MachineTactic::Rewrite {
            rule,
            direction,
            site,
            ..
        } => {
            out.push(0x03);
            encode_rewrite_rule_ref_to(out, rule);
            encode_rewrite_direction_to(out, *direction);
            encode_rewrite_site_to(out, *site);
        }
        MachineTactic::SimpLite { rules, .. } => {
            out.push(0x04);
            let canonical =
                canonicalize_simp_rule_refs(rules.clone()).unwrap_or_else(|_| rules.clone());
            encode_list_len_to(out, canonical.len());
            for rule in &canonical {
                encode_simp_rule_ref_to(out, rule);
            }
        }
        MachineTactic::InductionNat { local_name, .. } => {
            out.push(0x05);
            encode_string_to(out, local_name);
        }
    }
}

fn encode_rewrite_rule_ref_to(out: &mut Vec<u8>, rule: &RewriteRuleRef) {
    encode_tactic_head_to(out, &rule.head);
    encode_list_len_to(out, rule.universe_args.len());
    for level in &rule.universe_args {
        encode_level_to(out, level);
    }
    encode_list_len_to(out, rule.args.len());
    for arg in &rule.args {
        encode_apply_arg_to(out, arg);
    }
}

fn encode_tactic_head_to(out: &mut Vec<u8>, head: &TacticHead) {
    match head {
        TacticHead::Imported {
            name,
            decl_interface_hash,
        } => {
            out.push(0x00);
            encode_name_to(out, name);
            encode_hash_to(out, decl_interface_hash);
        }
        TacticHead::CurrentModule {
            name,
            decl_interface_hash,
        } => {
            out.push(0x01);
            encode_name_to(out, name);
            encode_hash_to(out, decl_interface_hash);
        }
        TacticHead::Local { name } => {
            out.push(0x02);
            encode_string_to(out, name);
        }
    }
}

fn encode_apply_arg_to(out: &mut Vec<u8>, arg: &ApplyArg) {
    match arg {
        ApplyArg::Term(term) => {
            out.push(0x00);
            encode_hash_to(out, &term.canonical_hash());
        }
        ApplyArg::Subgoal { .. } => {
            out.push(0x01);
        }
        ApplyArg::InferFromTarget => {
            out.push(0x02);
        }
    }
}

fn encode_rewrite_direction_to(out: &mut Vec<u8>, direction: RewriteDirection) {
    out.push(match direction {
        RewriteDirection::Forward => 0x00,
        RewriteDirection::Backward => 0x01,
    });
}

fn encode_rewrite_site_to(out: &mut Vec<u8>, site: RewriteSite) {
    out.push(match site {
        RewriteSite::EqTargetLeft => 0x00,
        RewriteSite::EqTargetRight => 0x01,
    });
}

fn encode_proof_expr_to(out: &mut Vec<u8>, expr: &ProofExpr) {
    match expr {
        ProofExpr::Core(core) => {
            out.push(0x00);
            encode_core_expr_bytes_to(out, core);
        }
        ProofExpr::Meta(id) => {
            out.push(0x01);
            encode_meta_var_id_to(out, *id);
        }
        ProofExpr::App(fun, arg) => {
            out.push(0x02);
            encode_proof_expr_to(out, fun);
            encode_proof_expr_to(out, arg);
        }
        ProofExpr::Lam { binder, ty, body } => {
            out.push(0x03);
            encode_string_to(out, binder);
            encode_core_expr_bytes_to(out, ty);
            encode_proof_expr_to(out, body);
        }
        ProofExpr::Let {
            binder,
            ty,
            value,
            body,
        } => {
            out.push(0x04);
            encode_string_to(out, binder);
            encode_core_expr_bytes_to(out, ty);
            encode_proof_expr_to(out, value);
            encode_proof_expr_to(out, body);
        }
    }
}

fn encode_option_proof_expr_to(out: &mut Vec<u8>, expr: Option<&ProofExpr>) {
    match expr {
        Some(expr) => {
            out.push(1);
            encode_proof_expr_to(out, expr);
        }
        None => out.push(0),
    }
}

fn encode_machine_local_decl_to(out: &mut Vec<u8>, local: &MachineLocalDecl) {
    encode_string_to(out, &local.name);
    encode_core_expr_bytes_to(out, &local.ty);
    match &local.value {
        Some(value) => {
            out.push(1);
            encode_core_expr_bytes_to(out, value);
        }
        None => out.push(0),
    }
}

fn encode_checked_decl_signature_to(out: &mut Vec<u8>, signature: &CheckedDeclSignature) {
    encode_name_to(out, &signature.name);
    encode_list_len_to(out, signature.universe_params.len());
    for param in &signature.universe_params {
        encode_string_to(out, param);
    }
    encode_core_expr_bytes_to(out, &signature.ty);
    encode_hash_to(out, &signature.decl_interface_hash);
}

fn encode_machine_tactic_options_to(out: &mut Vec<u8>, options: &MachineTacticOptions) {
    let rules = canonicalize_simp_rule_refs(options.simp_rules.clone())
        .unwrap_or_else(|_| options.simp_rules.clone());
    encode_list_len_to(out, rules.len());
    for rule in &rules {
        encode_simp_rule_ref_to(out, rule);
    }
    encode_option_eq_ref_to(out, options.eq_family.as_ref());
    encode_option_nat_ref_to(out, options.nat_family.as_ref());
    encode_u64_to(out, options.max_simp_rewrite_steps);
    encode_usize_to(out, options.max_open_goals);
    encode_usize_to(out, options.max_metas);
}

fn encode_simp_registry_to(out: &mut Vec<u8>, registry: &SimpRegistry) {
    encode_list_len_to(out, registry.rules.len());
    for rule in &registry.rules {
        encode_simp_rule_ref_to(out, &rule.key);
        encode_tactic_head_to(out, &rule.source);
        encode_checked_decl_signature_to(out, &rule.signature);
        encode_hash_to(out, &rule.core_decl_hash);
        encode_hash_to(out, &core_expr_hash(&rule.theorem_ty));
        encode_list_len_to(out, rule.universe_params.len());
        for param in &rule.universe_params {
            encode_string_to(out, param);
        }
        encode_list_len_to(out, rule.rule_telescope.len());
        for param in &rule.rule_telescope {
            encode_string_to(out, &param.name);
            encode_hash_to(out, &core_expr_hash(&param.ty));
        }
        encode_list_len_to(out, rule.eq_levels.len());
        for level in &rule.eq_levels {
            encode_level_to(out, level);
        }
        encode_hash_to(out, &core_expr_hash(&rule.eq_type));
        encode_hash_to(out, &core_expr_hash(&rule.theorem_lhs));
        encode_hash_to(out, &core_expr_hash(&rule.theorem_rhs));
        encode_hash_to(out, &core_expr_hash(&rule.from_pattern));
        encode_hash_to(out, &core_expr_hash(&rule.to_pattern));
    }
}

fn encode_simp_rule_ref_to(out: &mut Vec<u8>, rule: &SimpRuleRef) {
    encode_name_to(out, &rule.name);
    encode_hash_to(out, &rule.decl_interface_hash);
    encode_rewrite_direction_to(out, rule.direction);
}

fn encode_option_eq_ref_to(out: &mut Vec<u8>, value: Option<&EqFamilyRef>) {
    match value {
        Some(value) => {
            out.push(1);
            encode_name_to(out, &value.eq_name);
            encode_hash_to(out, &value.eq_interface_hash);
            encode_name_to(out, &value.refl_name);
            encode_hash_to(out, &value.refl_interface_hash);
            encode_name_to(out, &value.rec_name);
            encode_hash_to(out, &value.rec_interface_hash);
        }
        None => out.push(0),
    }
}

fn encode_option_nat_ref_to(out: &mut Vec<u8>, value: Option<&NatFamilyRef>) {
    match value {
        Some(value) => {
            out.push(1);
            encode_name_to(out, &value.nat_name);
            encode_hash_to(out, &value.nat_interface_hash);
            encode_name_to(out, &value.zero_name);
            encode_hash_to(out, &value.zero_interface_hash);
            encode_name_to(out, &value.succ_name);
            encode_hash_to(out, &value.succ_interface_hash);
            encode_name_to(out, &value.rec_name);
            encode_hash_to(out, &value.rec_interface_hash);
        }
        None => out.push(0),
    }
}

fn encode_option_resolved_eq_to(out: &mut Vec<u8>, value: Option<&ResolvedEqFamily>) {
    match value {
        Some(value) => {
            out.push(1);
            encode_name_to(out, &value.eq_name);
            encode_name_to(out, &value.refl_name);
            encode_name_to(out, &value.rec_name);
            encode_hash_to(out, &value.fingerprint);
        }
        None => out.push(0),
    }
}

fn encode_option_resolved_nat_to(out: &mut Vec<u8>, value: Option<&ResolvedNatFamily>) {
    match value {
        Some(value) => {
            out.push(1);
            encode_name_to(out, &value.nat_name);
            encode_name_to(out, &value.zero_name);
            encode_name_to(out, &value.succ_name);
            encode_name_to(out, &value.rec_name);
            encode_hash_to(out, &value.fingerprint);
        }
        None => out.push(0),
    }
}

fn encode_export_kind_to(out: &mut Vec<u8>, kind: ExportKind) {
    out.push(match kind {
        ExportKind::Axiom => 0,
        ExportKind::Def => 1,
        ExportKind::Theorem => 2,
        ExportKind::Inductive => 3,
        ExportKind::Constructor => 4,
        ExportKind::Recursor => 5,
    });
}

fn encode_meta_var_id_to(out: &mut Vec<u8>, id: MetaVarId) {
    encode_u64_to(out, id.0);
}

fn encode_goal_id_to(out: &mut Vec<u8>, id: GoalId) {
    encode_u64_to(out, id.0);
}

fn encode_name_to(out: &mut Vec<u8>, name: &Name) {
    encode_list_len_to(out, name.0.len());
    for component in &name.0 {
        encode_string_to(out, component);
    }
}

fn encode_option_hash_to(out: &mut Vec<u8>, value: Option<&Hash>) {
    match value {
        Some(value) => {
            out.push(1);
            encode_hash_to(out, value);
        }
        None => out.push(0),
    }
}

fn encode_hash_to(out: &mut Vec<u8>, hash: &Hash) {
    out.extend_from_slice(hash);
}

fn encode_string_to(out: &mut Vec<u8>, value: &str) {
    encode_u64_to(out, value.len() as u64);
    out.extend_from_slice(value.as_bytes());
}

fn encode_list_len_to(out: &mut Vec<u8>, len: usize) {
    encode_usize_to(out, len);
}

fn encode_usize_to(out: &mut Vec<u8>, value: usize) {
    encode_u64_to(out, value as u64);
}

fn encode_u64_to(out: &mut Vec<u8>, mut value: u64) {
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

fn tagged_bytes(tag: &str) -> Vec<u8> {
    let mut out = Vec::new();
    encode_string_to(&mut out, tag);
    out
}

fn hash_with_domain(domain: &str, bytes: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(domain.as_bytes());
    hasher.update([0]);
    hasher.update(bytes);
    hasher.finalize().into()
}

fn hex_hash(hash: &Hash) -> String {
    let mut out = String::with_capacity(64);
    for byte in hash {
        out.push(nibble_hex(byte >> 4));
        out.push(nibble_hex(byte & 0x0f));
    }
    out
}

fn nibble_hex(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'a' + value - 10) as char,
        _ => unreachable!("nibble must be in 0..16"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use npa_kernel::{Level, Reducibility};

    fn prop() -> Expr {
        Expr::sort(Level::zero())
    }

    fn type0() -> Expr {
        Expr::sort(Level::succ(Level::zero()))
    }

    fn verified_axiom_module(module: &str, decl_name: &str) -> npa_cert::VerifiedModule {
        let module = npa_cert::CoreModule {
            name: Name::from_dotted(module),
            declarations: vec![Decl::Axiom {
                name: decl_name.to_owned(),
                universe_params: Vec::new(),
                ty: type0(),
            }],
        };
        let cert = npa_cert::build_module_cert(module, &[]).unwrap();
        let bytes = npa_cert::encode_module_cert(&cert).unwrap();
        let mut session = npa_cert::VerifierSession::new();
        npa_cert::verify_module_cert(&bytes, &mut session, &npa_cert::AxiomPolicy::normal())
            .unwrap()
    }

    fn verified_core_module(module: CoreModule) -> npa_cert::VerifiedModule {
        let cert = npa_cert::build_module_cert(module, &[]).unwrap();
        let bytes = npa_cert::encode_module_cert(&cert).unwrap();
        let mut session = npa_cert::VerifierSession::new();
        npa_cert::verify_module_cert(&bytes, &mut session, &npa_cert::AxiomPolicy::normal())
            .unwrap()
    }

    fn verified_nat_builtin_module() -> npa_cert::VerifiedModule {
        verified_core_module(CoreModule {
            name: Name::from_dotted("Std.Nat.Basic"),
            declarations: vec![Decl::Inductive {
                name: "Nat".to_owned(),
                universe_params: Vec::new(),
                ty: Expr::sort(npa_kernel::type0()),
                data: Box::new(npa_kernel::nat_inductive()),
            }],
        })
    }

    fn verified_eq_builtin_module_with_rec() -> npa_cert::VerifiedModule {
        verified_core_module(CoreModule {
            name: Name::from_dotted("Std.Logic.Eq"),
            declarations: vec![
                Decl::Inductive {
                    name: "Eq".to_owned(),
                    universe_params: vec!["u".to_owned()],
                    ty: npa_kernel::eq_type(Level::param("u")),
                    data: Box::new(npa_kernel::eq_inductive()),
                },
                Decl::Axiom {
                    name: "Eq.rec".to_owned(),
                    universe_params: vec!["u".to_owned(), "v".to_owned()],
                    ty: npa_kernel::eq_rec_type(Level::param("u"), Level::param("v")),
                },
            ],
        })
    }

    fn verified_imported_simp_rule_module() -> npa_cert::VerifiedModule {
        let one = Expr::konst("Lib.one", Vec::new());
        verified_core_module(CoreModule {
            name: Name::from_dotted("Lib.Simp"),
            declarations: vec![
                Decl::Def {
                    name: "Lib.one".to_owned(),
                    universe_params: Vec::new(),
                    ty: nat(),
                    value: nat_succ(nat_zero()),
                    reducibility: Reducibility::Reducible,
                },
                Decl::Theorem {
                    name: "Lib.one_unfold".to_owned(),
                    universe_params: Vec::new(),
                    ty: eq_nat(one, nat_succ(nat_zero())),
                    proof: eq_refl_nat(nat_succ(nat_zero())),
                },
            ],
        })
    }

    fn trivial_spec() -> MachineProofSpec {
        MachineProofSpec {
            module: Name::from_dotted("Test"),
            theorem_name: Name::from_dotted("Test.thm"),
            source_index: 0,
            universe_params: Vec::new(),
            theorem_type: type0(),
        }
    }

    fn start_trivial() -> MachineProofState {
        start_machine_proof(
            trivial_spec(),
            Vec::new(),
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .expect("trivial proof state should start")
    }

    fn start_goal_with_local_definition(target: Expr) -> MachineProofState {
        let mut state = start_trivial();
        state.metas.get_mut(MetaVarId(0)).unwrap().assignment = Some(ProofExpr::Core(prop()));
        state.metas.metas.insert(
            MetaVarId(1),
            MachineMetaVar {
                id: MetaVarId(1),
                goal_id: GoalId(1),
                context: vec![MachineLocalDecl::definition("x", type0(), prop())],
                target,
                assignment: None,
                creation_index: 1,
            },
        );
        state.metas.next_id = 2;
        state.open_goals = vec![GoalId(1)];
        refresh_state_identity(&mut state);
        validate_machine_proof_state(&state).unwrap();
        state
    }

    fn checked_term(source: &str) -> MachineTermSource {
        MachineTermSource::new_checked(source).expect("test term source should canonicalize")
    }

    fn prop_id_type() -> Expr {
        Expr::pi("p", prop(), Expr::pi("hp", Expr::bvar(0), Expr::bvar(1)))
    }

    fn prop_id_proof() -> Expr {
        Expr::lam("p", prop(), Expr::lam("hp", Expr::bvar(0), Expr::bvar(0)))
    }

    fn nat() -> Expr {
        npa_kernel::nat()
    }

    fn nat_zero() -> Expr {
        npa_kernel::nat_zero()
    }

    fn nat_succ(arg: Expr) -> Expr {
        npa_kernel::nat_succ(arg)
    }

    fn eq_nat(lhs: Expr, rhs: Expr) -> Expr {
        npa_kernel::eq(npa_kernel::type0(), nat(), lhs, rhs)
    }

    fn eq_refl_nat(value: Expr) -> Expr {
        npa_kernel::eq_refl(npa_kernel::type0(), nat(), value)
    }

    fn rw_test_type() -> Expr {
        Expr::pi(
            "a",
            nat(),
            Expr::pi(
                "b",
                nat(),
                Expr::pi(
                    "h",
                    eq_nat(Expr::bvar(1), Expr::bvar(0)),
                    eq_nat(Expr::bvar(2), Expr::bvar(1)),
                ),
            ),
        )
    }

    fn start_rw_goal_with_local_eq() -> MachineProofState {
        let state = start_machine_proof(
            MachineProofSpec {
                theorem_type: rw_test_type(),
                ..trivial_spec()
            },
            Vec::new(),
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .unwrap();
        let (state, _) = run_machine_tactic(
            &state,
            MachineTactic::Intro {
                goal_id: GoalId(0),
                name: "a".to_owned(),
            },
        )
        .unwrap();
        let (state, _) = run_machine_tactic(
            &state,
            MachineTactic::Intro {
                goal_id: GoalId(1),
                name: "b".to_owned(),
            },
        )
        .unwrap();
        let (state, _) = run_machine_tactic(
            &state,
            MachineTactic::Intro {
                goal_id: GoalId(2),
                name: "h".to_owned(),
            },
        )
        .unwrap();
        state
    }

    fn checked_current_theorem(name: &str, ty: Expr, proof: Expr) -> CheckedCurrentDecl {
        checked_current_theorem_with_universes(name, Vec::new(), ty, proof)
    }

    fn checked_current_theorem_with_universes(
        name: &str,
        universe_params: Vec<String>,
        ty: Expr,
        proof: Expr,
    ) -> CheckedCurrentDecl {
        check_current_decl_for_machine_tactic_from_verified_imports(
            &[],
            &[],
            0,
            Decl::Theorem {
                name: name.to_owned(),
                universe_params,
                ty,
                proof,
            },
        )
        .expect("current theorem should be checked")
    }

    fn lib_q(arg: Expr) -> Expr {
        Expr::app(Expr::konst("Lib.Q", Vec::new()), arg)
    }

    fn verified_apply_module() -> npa_cert::VerifiedModule {
        let module = npa_cert::CoreModule {
            name: Name::from_dotted("Lib"),
            declarations: vec![
                Decl::Axiom {
                    name: "Lib.Q".to_owned(),
                    universe_params: Vec::new(),
                    ty: Expr::pi("p", prop(), prop()),
                },
                Decl::Axiom {
                    name: "Lib.mp".to_owned(),
                    universe_params: Vec::new(),
                    ty: Expr::pi(
                        "p",
                        prop(),
                        Expr::pi("hp", Expr::bvar(0), lib_q(Expr::bvar(1))),
                    ),
                },
                Decl::Axiom {
                    name: "Lib.drop".to_owned(),
                    universe_params: Vec::new(),
                    ty: Expr::pi("p", prop(), prop()),
                },
            ],
        };
        let cert = npa_cert::build_module_cert(module, &[]).unwrap();
        let bytes = npa_cert::encode_module_cert(&cert).unwrap();
        let mut session = npa_cert::VerifierSession::new();
        npa_cert::verify_module_cert(&bytes, &mut session, &npa_cert::AxiomPolicy::normal())
            .unwrap()
    }

    fn export_interface_hash(import: &VerifiedImportRef, name: &str) -> Hash {
        import
            .exports()
            .iter()
            .find(|export| export.name == Name::from_dotted(name))
            .expect("test export should exist")
            .decl_interface_hash
    }

    fn nat_family_ref(import: &VerifiedImportRef) -> NatFamilyRef {
        NatFamilyRef {
            nat_name: Name::from_dotted("Nat"),
            nat_interface_hash: export_interface_hash(import, "Nat"),
            zero_name: Name::from_dotted("Nat.zero"),
            zero_interface_hash: export_interface_hash(import, "Nat.zero"),
            succ_name: Name::from_dotted("Nat.succ"),
            succ_interface_hash: export_interface_hash(import, "Nat.succ"),
            rec_name: Name::from_dotted("Nat.rec"),
            rec_interface_hash: export_interface_hash(import, "Nat.rec"),
        }
    }

    fn nat_family_options(import: &VerifiedImportRef) -> MachineTacticOptions {
        MachineTacticOptions {
            nat_family: Some(nat_family_ref(import)),
            ..MachineTacticOptions::default()
        }
    }

    fn start_nat_induction_goal() -> MachineProofState {
        let nat_import =
            VerifiedImportRef::from_verified_module(&verified_nat_builtin_module()).unwrap();
        start_machine_proof(
            MachineProofSpec {
                theorem_type: Expr::pi("n", nat(), eq_nat(Expr::bvar(0), Expr::bvar(0))),
                ..trivial_spec()
            },
            vec![nat_import.clone()],
            Vec::new(),
            nat_family_options(&nat_import),
        )
        .unwrap()
    }

    #[test]
    fn start_machine_proof_allocates_deterministic_root_goal() {
        let state = start_trivial();

        assert_eq!(state.open_goals, vec![GoalId(0)]);
        assert_eq!(state.metas.next_id, 1);
        assert_eq!(state.state_id, hex_hash(&state.fingerprint));
        assert_eq!(state.state_id.len(), 64);
        assert!(matches!(state.root.body, ProofExpr::Meta(MetaVarId(0))));

        let state_again = start_trivial();
        assert_eq!(state.fingerprint, state_again.fingerprint);
    }

    #[test]
    fn invalid_numeric_limits_are_rejected() {
        let options = MachineTacticOptions {
            max_open_goals: 0,
            ..MachineTacticOptions::default()
        };
        let err = start_machine_proof(trivial_spec(), Vec::new(), Vec::new(), options)
            .expect_err("zero goal limit must be rejected");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::InvalidTacticOption);
    }

    #[test]
    fn theorem_name_must_belong_to_proof_module() {
        let spec = MachineProofSpec {
            theorem_name: Name::from_dotted("Other.thm"),
            ..trivial_spec()
        };
        let err = start_machine_proof(
            spec,
            Vec::new(),
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .expect_err("proof theorem name must stay inside the proof module");

        assert_eq!(
            err.kind,
            MachineTacticDiagnosticKind::InvalidMachineProofSpec
        );
    }

    #[test]
    fn theorem_type_must_have_sort_type() {
        let spec = MachineProofSpec {
            theorem_type: Expr::lam("x", type0(), Expr::bvar(0)),
            ..trivial_spec()
        };
        let err = start_machine_proof(
            spec,
            Vec::new(),
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .expect_err("a proof term is not a theorem type");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::TypeMismatch);
    }

    #[test]
    fn theorem_type_application_requires_function_type() {
        let spec = MachineProofSpec {
            theorem_type: Expr::app(prop(), prop()),
            ..trivial_spec()
        };
        let err = start_machine_proof(
            spec,
            Vec::new(),
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .expect_err("a non-function theorem type application must be rejected");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::ExpectedFunctionType);
        assert!(err.expected_hash.is_none());
        assert!(err.actual_hash.is_none());
    }

    #[test]
    fn theorem_type_inner_type_mismatch_carries_hashes() {
        let spec = MachineProofSpec {
            theorem_type: Expr::let_in("x", prop(), type0(), Expr::bvar(0)),
            ..trivial_spec()
        };
        let err = start_machine_proof(
            spec,
            Vec::new(),
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .expect_err("a theorem type with an ill-typed subterm must be rejected");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::TypeMismatch);
        assert!(err.expected_hash.is_some());
        assert!(err.actual_hash.is_some());
    }

    #[test]
    fn m4_rejects_unknown_custom_eq_family() {
        let options = MachineTacticOptions {
            eq_family: Some(EqFamilyRef {
                eq_name: Name::from_dotted("Eq"),
                eq_interface_hash: ZERO_HASH,
                refl_name: Name::from_dotted("Eq.refl"),
                refl_interface_hash: ZERO_HASH,
                rec_name: Name::from_dotted("Eq.rec"),
                rec_interface_hash: ZERO_HASH,
            }),
            ..MachineTacticOptions::default()
        };
        let err = start_machine_proof(trivial_spec(), Vec::new(), Vec::new(), options)
            .expect_err("unknown custom Eq family must be rejected");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::InvalidEqFamily);
        assert_eq!(err.primary_name, Some(Name::from_dotted("Eq")));
    }

    #[test]
    fn m4_resolves_current_generated_family_signatures_from_parent_hash() {
        let mut env = Env::new();
        env.add_inductive(npa_kernel::nat_inductive()).unwrap();
        let parent_decl = env.decl("Nat").unwrap().clone();
        let interface_hash = [7; 32];
        let core_decl_hash = [8; 32];
        let checked = CheckedCurrentDecl::from_checked_parts(
            3,
            parent_decl,
            interface_hash,
            core_decl_hash,
            [9; 32],
            [10; 32],
        );
        let current = vec![checked];

        let zero = resolve_family_signature(
            &env,
            &[],
            &current,
            &Name::from_dotted("Nat.zero"),
            &interface_hash,
        )
        .expect("current generated constructor should resolve through parent interface hash");
        assert_eq!(zero.signature.name(), &Name::from_dotted("Nat.zero"));
        assert_eq!(
            zero.origin,
            FamilyOrigin::CurrentSourceDecl {
                module: Name::from_dotted("Nat"),
                source_index: 3,
                core_decl_hash,
            }
        );

        let rec = resolve_family_signature(
            &env,
            &[],
            &current,
            &Name::from_dotted("Nat.rec"),
            &interface_hash,
        )
        .expect("current generated recursor should resolve through parent interface hash");
        assert_eq!(rec.signature.name(), &Name::from_dotted("Nat.rec"));
        assert_eq!(rec.origin, zero.origin);
    }

    #[test]
    fn m5_resolves_imported_nat_family() {
        let nat_import =
            VerifiedImportRef::from_verified_module(&verified_nat_builtin_module()).unwrap();
        let state = start_machine_proof(
            trivial_spec(),
            vec![nat_import.clone()],
            Vec::new(),
            nat_family_options(&nat_import),
        )
        .unwrap();

        let family = state
            .env
            .nat_family
            .as_ref()
            .expect("Nat family should resolve");
        assert_eq!(family.nat_name, Name::from_dotted("Nat"));
        assert_eq!(family.zero_name, Name::from_dotted("Nat.zero"));
        assert_eq!(family.succ_name, Name::from_dotted("Nat.succ"));
        assert_eq!(family.rec_name, Name::from_dotted("Nat.rec"));
    }

    #[test]
    fn m5_rejects_unknown_custom_nat_family() {
        let options = MachineTacticOptions {
            nat_family: Some(NatFamilyRef {
                nat_name: Name::from_dotted("Nat"),
                nat_interface_hash: ZERO_HASH,
                zero_name: Name::from_dotted("Nat.zero"),
                zero_interface_hash: ZERO_HASH,
                succ_name: Name::from_dotted("Nat.succ"),
                succ_interface_hash: ZERO_HASH,
                rec_name: Name::from_dotted("Nat.rec"),
                rec_interface_hash: ZERO_HASH,
            }),
            ..MachineTacticOptions::default()
        };
        let err = start_machine_proof(trivial_spec(), Vec::new(), Vec::new(), options)
            .expect_err("unknown custom Nat family must be rejected");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::InvalidNatFamily);
        assert_eq!(err.primary_name, Some(Name::from_dotted("Nat")));
    }

    #[test]
    fn m5_rejects_axiomatic_nat_family_heads() {
        let fake_nat = Expr::konst("Fake.Nat", Vec::new());
        let fake = verified_core_module(CoreModule {
            name: Name::from_dotted("Fake"),
            declarations: vec![
                Decl::Axiom {
                    name: "Fake.Nat".to_owned(),
                    universe_params: Vec::new(),
                    ty: type0(),
                },
                Decl::Axiom {
                    name: "Fake.zero".to_owned(),
                    universe_params: Vec::new(),
                    ty: fake_nat.clone(),
                },
                Decl::Axiom {
                    name: "Fake.succ".to_owned(),
                    universe_params: Vec::new(),
                    ty: Expr::pi("_", fake_nat, Expr::konst("Fake.Nat", Vec::new())),
                },
                Decl::Axiom {
                    name: "Fake.rec".to_owned(),
                    universe_params: vec!["u".to_owned()],
                    ty: expected_nat_rec_type(
                        "Fake.Nat",
                        "Fake.zero",
                        "Fake.succ",
                        Level::param("u"),
                    ),
                },
            ],
        });
        let fake_import = VerifiedImportRef::from_verified_module(&fake).unwrap();
        let options = MachineTacticOptions {
            nat_family: Some(NatFamilyRef {
                nat_name: Name::from_dotted("Fake.Nat"),
                nat_interface_hash: export_interface_hash(&fake_import, "Fake.Nat"),
                zero_name: Name::from_dotted("Fake.zero"),
                zero_interface_hash: export_interface_hash(&fake_import, "Fake.zero"),
                succ_name: Name::from_dotted("Fake.succ"),
                succ_interface_hash: export_interface_hash(&fake_import, "Fake.succ"),
                rec_name: Name::from_dotted("Fake.rec"),
                rec_interface_hash: export_interface_hash(&fake_import, "Fake.rec"),
            }),
            ..MachineTacticOptions::default()
        };
        let err = start_machine_proof(trivial_spec(), vec![fake_import], Vec::new(), options)
            .expect_err("Nat family must come from an inductive generated closure");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::InvalidNatFamily);
    }

    #[test]
    fn local_context_hash_and_prefix_are_deterministic() {
        let context = vec![
            MachineLocalDecl::assumption("A", prop()),
            MachineLocalDecl::definition("x", Expr::bvar(0), Expr::bvar(0)),
        ];
        let extended = vec![
            MachineLocalDecl::assumption("A", prop()),
            MachineLocalDecl::definition("x", Expr::bvar(0), Expr::bvar(0)),
            MachineLocalDecl::assumption("y", Expr::bvar(1)),
        ];

        assert!(machine_local_context_is_prefix(&context, &extended));
        assert_eq!(
            machine_local_context_hash(&context),
            machine_local_context_hash(&context)
        );
        assert_ne!(
            machine_local_context_hash(&context),
            machine_local_context_hash(&extended)
        );
    }

    #[test]
    fn canonical_goal_and_meta_ids_have_distinct_domains() {
        assert_ne!(
            meta_var_id_canonical_bytes(MetaVarId(7)),
            goal_id_canonical_bytes(GoalId(7))
        );
    }

    #[test]
    fn machine_tactic_candidate_canonicalizes_exact_term_source() {
        let first = validate_machine_tactic_candidate(
            GoalId(0),
            MachineTacticCandidate::Exact {
                term: RawMachineTerm::new("Prop"),
            },
        )
        .expect("exact candidate should validate");
        let second = validate_machine_tactic_candidate(
            GoalId(0),
            MachineTacticCandidate::Exact {
                term: RawMachineTerm::new("  Prop  "),
            },
        )
        .expect("exact candidate should canonicalize whitespace-insensitive source");

        let (
            MachineTactic::Exact {
                term: first_term, ..
            },
            MachineTactic::Exact {
                term: second_term, ..
            },
        ) = (first, second)
        else {
            panic!("validated exact candidate must produce exact tactic");
        };
        assert_eq!(first_term.canonical_hash(), second_term.canonical_hash());
        assert_eq!(
            first_term.canonical_hash(),
            machine_term_source_hash(&first_term).unwrap()
        );
    }

    #[test]
    fn machine_tactic_candidate_validation_attaches_goal_and_tactic_kind() {
        let err = validate_machine_tactic_candidate(
            GoalId(7),
            MachineTacticCandidate::Intro {
                name: "bad-name".to_owned(),
            },
        )
        .expect_err("invalid intro candidate should keep structured context");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::InvalidMachineTactic);
        assert_eq!(err.goal_id, Some(GoalId(7)));
        assert_eq!(err.tactic_kind.as_deref(), Some("intro"));
        assert_eq!(err.primary_name, None);
    }

    #[test]
    fn machine_term_diagnostic_preserves_repair_relevant_frontend_kinds() {
        let span = npa_frontend::Span::empty(npa_frontend::FileId(0));
        let cases = [
            (
                npa_frontend::MachineDiagnosticKind::UnknownGlobalName,
                MachineTacticDiagnosticKind::UnknownName,
            ),
            (
                npa_frontend::MachineDiagnosticKind::ShortGlobalName,
                MachineTacticDiagnosticKind::UnknownName,
            ),
            (
                npa_frontend::MachineDiagnosticKind::AmbiguousGlobalName,
                MachineTacticDiagnosticKind::UnknownName,
            ),
            (
                npa_frontend::MachineDiagnosticKind::GlobalShadowedByLocal,
                MachineTacticDiagnosticKind::UnknownName,
            ),
            (
                npa_frontend::MachineDiagnosticKind::UnknownLocalName,
                MachineTacticDiagnosticKind::UnknownName,
            ),
            (
                npa_frontend::MachineDiagnosticKind::ImplicitArgumentRequired,
                MachineTacticDiagnosticKind::ImplicitArgumentRequired,
            ),
            (
                npa_frontend::MachineDiagnosticKind::MissingExplicitUniverse,
                MachineTacticDiagnosticKind::ImplicitArgumentRequired,
            ),
            (
                npa_frontend::MachineDiagnosticKind::ExpectedFunctionType,
                MachineTacticDiagnosticKind::ExpectedFunctionType,
            ),
        ];

        for (frontend_kind, tactic_kind) in cases {
            let diagnostic =
                npa_frontend::MachineDiagnostic::error(frontend_kind, span, "frontend failure");
            let mapped = machine_term_elaboration_diag(diagnostic);

            assert_eq!(mapped.kind, tactic_kind);
        }
    }

    #[test]
    fn machine_term_diagnostic_carries_structured_primary_name_when_available() {
        let span = npa_frontend::Span::empty(npa_frontend::FileId(0));
        let diagnostic = npa_frontend::MachineDiagnostic::error(
            npa_frontend::MachineDiagnosticKind::ImplicitArgumentRequired,
            span,
            "frontend failure",
        )
        .with_payload(npa_frontend::MachineDiagnosticPayload {
            head_symbol: Some("Eq".to_owned()),
            ..npa_frontend::MachineDiagnosticPayload::default()
        });

        let mapped = machine_term_elaboration_diag(diagnostic);

        assert_eq!(
            mapped.kind,
            MachineTacticDiagnosticKind::ImplicitArgumentRequired
        );
        assert_eq!(mapped.primary_name, Some(Name::from_dotted("Eq")));
    }

    #[test]
    fn machine_term_elaboration_error_keeps_structured_primary_name() {
        let span = npa_frontend::Span::empty(npa_frontend::FileId(0));
        let cases = [
            npa_frontend::MachineDiagnosticKind::KernelRejected,
            npa_frontend::MachineDiagnosticKind::ExpectedSort,
        ];

        for frontend_kind in cases {
            let diagnostic =
                npa_frontend::MachineDiagnostic::error(frontend_kind, span, "frontend failure")
                    .with_payload(npa_frontend::MachineDiagnosticPayload {
                        head_symbol: Some("Nat".to_owned()),
                        ..npa_frontend::MachineDiagnosticPayload::default()
                    });
            let mapped = machine_term_elaboration_diag(diagnostic);

            assert_eq!(
                mapped.kind,
                MachineTacticDiagnosticKind::MachineTermElaborationError
            );
            assert_eq!(mapped.primary_name, Some(Name::from_dotted("Nat")));
        }
    }

    #[test]
    fn rewrite_candidate_validation_uses_rw_tactic_kind() {
        let err = validate_machine_tactic_candidate(
            GoalId(7),
            MachineTacticCandidate::Rewrite {
                rule: CandidateRewriteRuleRef {
                    head: TacticHead::Local {
                        name: "bad-name".to_owned(),
                    },
                    universe_args: Vec::new(),
                    args: Vec::new(),
                },
                direction: RewriteDirection::Forward,
                site: RewriteSite::EqTargetLeft,
            },
        )
        .expect_err("invalid rw candidate should keep the wire tactic kind");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::InvalidMachineTactic);
        assert_eq!(err.goal_id, Some(GoalId(7)));
        assert_eq!(err.tactic_kind.as_deref(), Some("rw"));
        assert_eq!(err.primary_name, None);
    }

    #[test]
    fn tactic_budget_hash_is_deterministic_and_field_sensitive() {
        let budget = TacticBudget {
            max_tactic_steps: 1,
            max_whnf_steps: 2,
            max_conversion_steps: 3,
            max_rewrite_steps: 4,
            max_meta_allocations: 5,
            max_expr_nodes: 6,
        };
        let changed = TacticBudget {
            max_expr_nodes: 7,
            ..budget
        };

        assert_eq!(tactic_budget_hash(budget), tactic_budget_hash(budget));
        assert_ne!(tactic_budget_hash(budget), tactic_budget_hash(changed));
        let mut expected = Sha256::new();
        expected.update(tactic_budget_canonical_bytes(budget));
        let expected_hash: Hash = expected.finalize().into();
        assert_eq!(tactic_budget_hash(budget), expected_hash);
        assert_ne!(
            tactic_budget_canonical_bytes(budget),
            tactic_budget_canonical_bytes(changed)
        );
    }

    #[test]
    fn tactic_cache_key_includes_goal_and_deterministic_budget() {
        let state = start_trivial();
        let tactic = MachineTactic::Exact {
            goal_id: GoalId(0),
            term: checked_term("Prop"),
        };
        let same_payload_other_goal = MachineTactic::Exact {
            goal_id: GoalId(1),
            term: checked_term("Prop"),
        };
        let budget = TacticBudget::default();
        let bigger_budget = TacticBudget {
            max_whnf_steps: budget.max_whnf_steps + 1,
            ..budget
        };

        assert_eq!(
            machine_tactic_hash(&tactic),
            machine_tactic_hash(&same_payload_other_goal),
            "canonical tactic hash intentionally excludes the selected goal"
        );

        let key = machine_tactic_cache_key(&state, &tactic, budget);
        let other_goal_key = machine_tactic_cache_key(&state, &same_payload_other_goal, budget);
        let other_budget_key = machine_tactic_cache_key(&state, &tactic, bigger_budget);

        assert_eq!(key.goal_id, GoalId(0));
        assert_eq!(key.deterministic_budget_hash, tactic_budget_hash(budget));
        assert_ne!(
            machine_tactic_cache_key_hash(&key),
            machine_tactic_cache_key_hash(&other_goal_key)
        );
        assert_ne!(
            machine_tactic_cache_key_hash(&key),
            machine_tactic_cache_key_hash(&other_budget_key)
        );
    }

    #[test]
    fn core_hashes_ignore_display_binder_names() {
        let pi_x = Expr::pi("x", type0(), Expr::bvar(0));
        let pi_y = Expr::pi("y", type0(), Expr::bvar(0));
        assert_eq!(core_expr_hash(&pi_x), core_expr_hash(&pi_y));

        let decl_x = Decl::Theorem {
            name: "Test.A".to_owned(),
            universe_params: Vec::new(),
            ty: Expr::pi("x", type0(), type0()),
            proof: Expr::lam("x", type0(), Expr::bvar(0)),
        };
        let decl_y = Decl::Theorem {
            name: "Test.A".to_owned(),
            universe_params: Vec::new(),
            ty: Expr::pi("y", type0(), type0()),
            proof: Expr::lam("y", type0(), Expr::bvar(0)),
        };
        let checked_x = check_current_decl_for_machine_tactic_from_verified_imports(
            &[],
            &[],
            0,
            decl_x.clone(),
        )
        .unwrap();
        let checked_y =
            check_current_decl_for_machine_tactic_from_verified_imports(&[], &[], 0, decl_y)
                .unwrap();
        let cert_x = npa_cert::build_module_cert(
            CoreModule {
                name: Name::from_dotted("Test"),
                declarations: vec![decl_x],
            },
            &[],
        )
        .unwrap();

        assert_eq!(checked_x.core_decl_hash(), checked_y.core_decl_hash());
        assert_eq!(
            checked_x.core_decl_hash(),
            cert_x.declarations[0].hashes.decl_certificate_hash
        );
    }

    #[test]
    fn kernel_profile_hash_commits_to_semantic_profile_fields() {
        let old_profile_hash = hash_with_domain(
            "npa.phase4.kernel-check-profile.v1",
            b"npa-kernel:core-spec-v0.1",
        );

        assert_ne!(kernel_check_profile_hash(), old_profile_hash);
        assert_eq!(kernel_check_profile_hash(), kernel_check_profile_hash());
    }

    #[test]
    fn assign_goal_with_core_proof_closes_the_root_goal() {
        let state = start_trivial();
        let (closed, delta) = assign_goal(&state, GoalId(0), ProofExpr::Core(prop()), Vec::new())
            .expect("Sort 0 has type Sort 1 in the current kernel");

        assert!(closed.open_goals.is_empty());
        assert_eq!(delta.assigned_goal, GoalId(0));
        assert_eq!(delta.added_goals, Vec::<GoalId>::new());
        assert_ne!(state.fingerprint, closed.fingerprint);

        let proof = extract_closed_machine_proof(&closed).expect("proof should extract");
        assert_eq!(proof, prop());
    }

    #[test]
    fn exact_tactic_closes_goal_with_machine_surface_term() {
        let state = start_trivial();
        let (closed, delta) = run_machine_tactic(
            &state,
            MachineTactic::Exact {
                goal_id: GoalId(0),
                term: checked_term("Prop"),
            },
        )
        .expect("exact Prop should close a Type goal");

        assert!(closed.open_goals.is_empty());
        assert_eq!(delta.assigned_goal, GoalId(0));
        assert_eq!(
            extract_closed_machine_proof(&closed).expect("proof should extract"),
            prop()
        );
    }

    #[test]
    fn machine_term_elab_context_populates_callable_table_for_tactic_terms() {
        let verified = verified_apply_module();
        let import = VerifiedImportRef::from_verified_module(&verified).unwrap();
        let state = start_machine_proof(
            MachineProofSpec {
                module: Name::from_dotted("Test"),
                theorem_name: Name::from_dotted("Test.thm"),
                source_index: 0,
                universe_params: Vec::new(),
                theorem_type: prop(),
            },
            vec![import],
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .unwrap();

        let context = machine_term_elab_context(&state, &[]).unwrap();

        assert!(context
            .callable_interface_table()
            .entries()
            .iter()
            .any(|entry| matches!(
                entry.callable_ref(),
                npa_frontend::MachineSurfaceCallableRef::Imported { name, .. }
                    if name == &Name::from_dotted("Lib.mp")
            )));
    }

    #[test]
    fn machine_term_elab_context_populates_imported_generated_callable_entries() {
        let verified = verified_nat_builtin_module();
        let import = VerifiedImportRef::from_verified_module(&verified).unwrap();
        let state = start_machine_proof(
            MachineProofSpec {
                module: Name::from_dotted("Test"),
                theorem_name: Name::from_dotted("Test.thm"),
                source_index: 0,
                universe_params: Vec::new(),
                theorem_type: prop(),
            },
            vec![import],
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .unwrap();

        let context = machine_term_elab_context(&state, &[]).unwrap();

        assert!(context
            .callable_interface_table()
            .entries()
            .iter()
            .any(|entry| matches!(
                entry.callable_ref(),
                npa_frontend::MachineSurfaceCallableRef::Imported { name, .. }
                    if name == &Name::from_dotted("Nat.succ")
            )));
        assert!(context
            .callable_interface_table()
            .entries()
            .iter()
            .any(|entry| matches!(
                entry.callable_ref(),
                npa_frontend::MachineSurfaceCallableRef::Imported { name, .. }
                    if name == &Name::from_dotted("Nat.rec")
            )));
    }

    #[test]
    fn machine_term_elab_context_rejects_unrenderable_import_callable_name() {
        let verified = verified_axiom_module("Lib", "_bad");
        let import = VerifiedImportRef::from_verified_module(&verified).unwrap();
        let state = start_machine_proof(
            MachineProofSpec {
                module: Name::from_dotted("Test"),
                theorem_name: Name::from_dotted("Test.thm"),
                source_index: 0,
                universe_params: Vec::new(),
                theorem_type: prop(),
            },
            vec![import],
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .unwrap();

        let err = machine_term_elab_context(&state, &[]).unwrap_err();

        assert_eq!(err.kind, MachineTacticDiagnosticKind::InvalidVerifiedImport);
    }

    #[test]
    fn transactional_result_is_deterministic_for_same_input_and_budget() {
        let state = start_trivial();
        let tactic = MachineTactic::Exact {
            goal_id: GoalId(0),
            term: checked_term("Prop"),
        };
        let budget = TacticBudget::default();

        let first = run_machine_tactic_transactional(&state, tactic.clone(), budget);
        let second = run_machine_tactic_transactional(&state, tactic, budget);

        let (
            MachineTacticResult::Success {
                state: first_state,
                delta: first_delta,
            },
            MachineTacticResult::Success {
                state: second_state,
                delta: second_delta,
            },
        ) = (first, second)
        else {
            panic!("exact Prop should succeed deterministically");
        };

        assert_eq!(first_state.fingerprint, second_state.fingerprint);
        assert_eq!(first_delta.delta_hash, second_delta.delta_hash);
        assert_eq!(state.open_goals, vec![GoalId(0)]);
    }

    #[test]
    fn transactional_result_keeps_failed_tactic_as_structured_error() {
        let state = start_trivial();
        let tactic = MachineTactic::Intro {
            goal_id: GoalId(0),
            name: "p".to_owned(),
        };

        let first =
            run_machine_tactic_transactional(&state, tactic.clone(), TacticBudget::default());
        let second = run_machine_tactic_transactional(&state, tactic, TacticBudget::default());

        let (
            MachineTacticResult::Error {
                diagnostic: first_diagnostic,
            },
            MachineTacticResult::Error {
                diagnostic: second_diagnostic,
            },
        ) = (first, second)
        else {
            panic!("intro on a non-Pi target should fail deterministically");
        };

        assert_eq!(first_diagnostic, second_diagnostic);
        assert_eq!(
            first_diagnostic.kind,
            MachineTacticDiagnosticKind::TypeMismatch
        );
        assert_eq!(first_diagnostic.goal_id, Some(GoalId(0)));
        assert_eq!(first_diagnostic.tactic_kind.as_deref(), Some("intro"));
        assert_eq!(state.open_goals, vec![GoalId(0)]);
    }

    #[test]
    fn tactic_batch_runs_candidates_against_the_same_input_state() {
        let state = start_trivial();
        let budget = TacticBudget::default();
        let batch = run_machine_tactic_candidates_batch(
            &state,
            GoalId(0),
            vec![
                MachineTacticBatchCandidate {
                    candidate_id: "c0".to_owned(),
                    candidate: MachineTacticCandidate::Exact {
                        term: RawMachineTerm::new("Prop"),
                    },
                },
                MachineTacticBatchCandidate {
                    candidate_id: "c1".to_owned(),
                    candidate: MachineTacticCandidate::Intro {
                        name: "p".to_owned(),
                    },
                },
            ],
            budget,
            MachineTacticBatchPolicy::default(),
        )
        .expect("valid batch should run");

        assert_eq!(batch.previous_state_fingerprint, state.fingerprint);
        assert_eq!(batch.deterministic_budget_hash, tactic_budget_hash(budget));
        assert_eq!(batch.results.len(), 2);
        assert_eq!(state.open_goals, vec![GoalId(0)]);

        match &batch.results[0] {
            MachineTacticBatchItemResult::Success {
                candidate_id,
                state: next_state,
                delta,
                ..
            } => {
                assert_eq!(candidate_id, "c0");
                assert!(next_state.open_goals.is_empty());
                assert_eq!(delta.assigned_goal, GoalId(0));
            }
            other => panic!("first candidate should succeed: {other:?}"),
        }

        match &batch.results[1] {
            MachineTacticBatchItemResult::Error {
                candidate_id,
                candidate_hash,
                phase,
                diagnostic,
                retryable,
            } => {
                assert_eq!(candidate_id, "c1");
                assert!(candidate_hash.is_some());
                assert_eq!(*phase, MachineTacticBatchErrorPhase::TacticExecution);
                assert_eq!(diagnostic.kind, MachineTacticDiagnosticKind::TypeMismatch);
                assert!(!retryable);
            }
            other => panic!("second candidate should fail independently: {other:?}"),
        }
    }

    #[test]
    fn tactic_batch_reports_candidate_validation_without_candidate_hash() {
        let state = start_trivial();
        let batch = run_machine_tactic_candidates_batch(
            &state,
            GoalId(0),
            vec![MachineTacticBatchCandidate {
                candidate_id: "c0".to_owned(),
                candidate: MachineTacticCandidate::Intro {
                    name: "bad-name".to_owned(),
                },
            }],
            TacticBudget::default(),
            MachineTacticBatchPolicy::default(),
        )
        .expect("candidate validation failures are per-candidate batch results");

        assert_eq!(batch.results.len(), 1);
        match &batch.results[0] {
            MachineTacticBatchItemResult::Error {
                candidate_hash,
                phase,
                diagnostic,
                ..
            } => {
                assert!(candidate_hash.is_none());
                assert_eq!(*phase, MachineTacticBatchErrorPhase::CandidateValidation);
                assert_eq!(
                    diagnostic.kind,
                    MachineTacticDiagnosticKind::InvalidMachineTactic
                );
                assert_eq!(diagnostic.goal_id, Some(GoalId(0)));
                assert_eq!(diagnostic.tactic_kind.as_deref(), Some("intro"));
            }
            other => panic!("invalid candidate should produce an error item: {other:?}"),
        }
    }

    #[test]
    fn tactic_batch_rejects_unknown_selected_goal_before_candidate_validation() {
        let state = start_trivial();
        let err = run_machine_tactic_candidates_batch(
            &state,
            GoalId(99),
            vec![MachineTacticBatchCandidate {
                candidate_id: "c0".to_owned(),
                candidate: MachineTacticCandidate::Intro {
                    name: "bad-name".to_owned(),
                },
            }],
            TacticBudget::default(),
            MachineTacticBatchPolicy::default(),
        )
        .expect_err("selected goal lookup should precede candidate validation");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::UnknownGoal);
        assert_eq!(err.goal_id, Some(GoalId(99)));
        assert_eq!(err.tactic_kind, None);
    }

    #[test]
    fn tactic_batch_policy_stops_on_request_order_prefix() {
        let state = start_trivial();
        let batch = run_machine_tactic_candidates_batch(
            &state,
            GoalId(0),
            vec![
                MachineTacticBatchCandidate {
                    candidate_id: "c0".to_owned(),
                    candidate: MachineTacticCandidate::Exact {
                        term: RawMachineTerm::new("Prop"),
                    },
                },
                MachineTacticBatchCandidate {
                    candidate_id: "c1".to_owned(),
                    candidate: MachineTacticCandidate::Exact {
                        term: RawMachineTerm::new("Prop"),
                    },
                },
            ],
            TacticBudget::default(),
            MachineTacticBatchPolicy {
                max_evaluated_candidates: 1,
                stop_after_successes: 256,
                stop_after_failures: 256,
            },
        )
        .expect("policy-limited batch should run");

        assert_eq!(batch.results.len(), 1);
        match &batch.results[0] {
            MachineTacticBatchItemResult::Success { candidate_id, .. } => {
                assert_eq!(candidate_id, "c0");
            }
            other => panic!("first candidate should be the only evaluated item: {other:?}"),
        }
    }

    #[test]
    fn tactic_batch_rejects_invalid_policy_before_candidate_validation() {
        let state = start_trivial();
        let err = run_machine_tactic_candidates_batch(
            &state,
            GoalId(0),
            vec![MachineTacticBatchCandidate {
                candidate_id: "bad space".to_owned(),
                candidate: MachineTacticCandidate::Intro {
                    name: "bad-name".to_owned(),
                },
            }],
            TacticBudget::default(),
            MachineTacticBatchPolicy::default(),
        )
        .expect_err("invalid candidate_id is a batch policy error");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::InvalidBatchPolicy);
    }

    #[test]
    fn tactic_batch_policy_validation_precedes_state_validation() {
        let mut state = start_trivial();
        state.state_id.clear();

        let err = run_machine_tactic_candidates_batch(
            &state,
            GoalId(0),
            vec![MachineTacticBatchCandidate {
                candidate_id: "bad space".to_owned(),
                candidate: MachineTacticCandidate::Exact {
                    term: RawMachineTerm::new("Prop"),
                },
            }],
            TacticBudget::default(),
            MachineTacticBatchPolicy::default(),
        )
        .expect_err("batch policy validation must run before state validation");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::InvalidBatchPolicy);
    }

    #[test]
    fn exact_tactic_revalidates_machine_term_source_hash() {
        let state = start_trivial();
        let stale = MachineTermSource {
            source: "Prop".to_owned(),
            canonical_hash: ZERO_HASH,
        };
        let err = run_machine_tactic(
            &state,
            MachineTactic::Exact {
                goal_id: GoalId(0),
                term: stale,
            },
        )
        .expect_err("stale term source hash must be rejected");

        assert_eq!(
            err.kind,
            MachineTacticDiagnosticKind::InvalidMachineTermSource
        );
        assert_eq!(err.goal_id, Some(GoalId(0)));
        assert_eq!(err.meta_id, Some(MetaVarId(0)));
        assert_eq!(state.open_goals, vec![GoalId(0)]);
    }

    #[test]
    fn exact_term_error_precedes_expr_node_budget() {
        let state = start_trivial();
        let budget = TacticBudget {
            max_expr_nodes: 0,
            ..TacticBudget::default()
        };
        let err = run_machine_tactic_with_budget(
            &state,
            MachineTactic::Exact {
                goal_id: GoalId(0),
                term: checked_term("Type"),
            },
            budget,
        )
        .expect_err("term type mismatch should be reported before expression-node fuel");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::TypeMismatch);
    }

    #[test]
    fn exact_expr_node_budget_is_checked_after_elaboration() {
        let state = start_trivial();
        let budget = TacticBudget {
            max_expr_nodes: 0,
            ..TacticBudget::default()
        };
        let err = run_machine_tactic_with_budget(
            &state,
            MachineTactic::Exact {
                goal_id: GoalId(0),
                term: checked_term("Prop"),
            },
            budget,
        )
        .expect_err("valid exact proof still needs expression-node fuel");

        assert_eq!(
            err.kind,
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::ExprNode
            }
        );
        assert_eq!(state.open_goals, vec![GoalId(0)]);
    }

    #[test]
    fn exact_tactic_consumes_dispatch_and_construction_steps() {
        let state = start_trivial();
        let budget = TacticBudget {
            max_tactic_steps: 1,
            max_expr_nodes: 0,
            ..TacticBudget::default()
        };
        let err = run_machine_tactic_with_budget(
            &state,
            MachineTactic::Exact {
                goal_id: GoalId(0),
                term: checked_term("Prop"),
            },
            budget,
        )
        .expect_err("exact requires dispatch and proof construction step fuel");

        assert_eq!(
            err.kind,
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::TacticStep
            }
        );
        assert_eq!(state.open_goals, vec![GoalId(0)]);
    }

    #[test]
    fn intro_tactic_creates_lambda_and_body_goal() {
        let spec = MachineProofSpec {
            theorem_type: Expr::pi("p", prop(), prop()),
            ..trivial_spec()
        };
        let state = start_machine_proof(
            spec,
            Vec::new(),
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .expect("Pi theorem type should start");

        let (state, delta) = run_machine_tactic(
            &state,
            MachineTactic::Intro {
                goal_id: GoalId(0),
                name: "p".to_owned(),
            },
        )
        .expect("intro should open the Pi body");

        assert_eq!(state.open_goals, vec![GoalId(1)]);
        assert_eq!(delta.added_goals, vec![GoalId(1)]);
        let body_goal = state.goal(GoalId(1)).unwrap();
        assert_eq!(
            body_goal.context,
            vec![MachineLocalDecl::assumption("p", prop())]
        );
        assert_eq!(body_goal.target, prop());

        let (closed, _) = run_machine_tactic(
            &state,
            MachineTactic::Exact {
                goal_id: GoalId(1),
                term: checked_term("p"),
            },
        )
        .expect("introduced local should close the body goal");
        assert_eq!(
            extract_closed_machine_proof(&closed).unwrap(),
            Expr::lam("p", prop(), Expr::bvar(0))
        );
    }

    #[test]
    fn induction_nat_generates_base_and_step_goals() {
        let state = start_nat_induction_goal();
        let (state, _) = run_machine_tactic(
            &state,
            MachineTactic::Intro {
                goal_id: GoalId(0),
                name: "n".to_owned(),
            },
        )
        .unwrap();
        let (state, delta) = run_machine_tactic(
            &state,
            MachineTactic::InductionNat {
                goal_id: GoalId(1),
                local_name: "n".to_owned(),
            },
        )
        .expect("induction-nat should produce base and step goals");

        assert_eq!(delta.added_goals, vec![GoalId(2), GoalId(3)]);
        assert_eq!(state.open_goals, vec![GoalId(2), GoalId(3)]);
        let motive = Expr::lam("n0", nat(), eq_nat(Expr::bvar(0), Expr::bvar(0)));
        let base = state.goal(GoalId(2)).unwrap();
        assert!(base.context.is_empty());
        assert_eq!(base.target, Expr::app(motive.clone(), nat_zero()));
        let step = state.goal(GoalId(3)).unwrap();
        assert_eq!(
            step.context,
            vec![
                MachineLocalDecl::assumption("n", nat()),
                MachineLocalDecl::assumption("n0", nat()),
                MachineLocalDecl::assumption("ih0", Expr::app(motive.clone(), Expr::bvar(0)),),
            ]
        );
        assert_eq!(step.target, Expr::app(motive, nat_succ(Expr::bvar(1))));

        let (state, _) = assign_goal(
            &state,
            GoalId(2),
            ProofExpr::Core(eq_refl_nat(nat_zero())),
            Vec::new(),
        )
        .unwrap();
        let (state, _) = assign_goal(
            &state,
            GoalId(3),
            ProofExpr::Core(eq_refl_nat(nat_succ(Expr::bvar(1)))),
            Vec::new(),
        )
        .unwrap();
        extract_closed_machine_proof(&state).expect("closed induction proof should check");
    }

    #[test]
    fn induction_nat_requires_explicit_nat_family() {
        let state = start_machine_proof(
            MachineProofSpec {
                theorem_type: Expr::pi("n", nat(), eq_nat(Expr::bvar(0), Expr::bvar(0))),
                ..trivial_spec()
            },
            Vec::new(),
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .unwrap();
        let (state, _) = run_machine_tactic(
            &state,
            MachineTactic::Intro {
                goal_id: GoalId(0),
                name: "n".to_owned(),
            },
        )
        .unwrap();
        let err = run_machine_tactic(
            &state,
            MachineTactic::InductionNat {
                goal_id: GoalId(1),
                local_name: "n".to_owned(),
            },
        )
        .expect_err("induction-nat is disabled without nat_family");

        assert_eq!(
            err.kind,
            MachineTacticDiagnosticKind::TacticPrimitiveUnavailable
        );
    }

    #[test]
    fn induction_nat_rejects_non_last_target_before_fuel() {
        let nat_import =
            VerifiedImportRef::from_verified_module(&verified_nat_builtin_module()).unwrap();
        let state = start_machine_proof(
            MachineProofSpec {
                theorem_type: Expr::pi(
                    "n",
                    nat(),
                    Expr::pi("m", nat(), eq_nat(Expr::bvar(1), Expr::bvar(1))),
                ),
                ..trivial_spec()
            },
            vec![nat_import.clone()],
            Vec::new(),
            nat_family_options(&nat_import),
        )
        .unwrap();
        let (state, _) = run_machine_tactic(
            &state,
            MachineTactic::Intro {
                goal_id: GoalId(0),
                name: "n".to_owned(),
            },
        )
        .unwrap();
        let (state, _) = run_machine_tactic(
            &state,
            MachineTactic::Intro {
                goal_id: GoalId(1),
                name: "m".to_owned(),
            },
        )
        .unwrap();
        let budget = TacticBudget {
            max_tactic_steps: 0,
            ..TacticBudget::default()
        };
        let err = run_machine_tactic_with_budget(
            &state,
            MachineTactic::InductionNat {
                goal_id: GoalId(2),
                local_name: "n".to_owned(),
            },
            budget,
        )
        .expect_err("later locals should be rejected before tactic-step fuel");

        assert_eq!(
            err.kind,
            MachineTacticDiagnosticKind::InvalidInductionTarget
        );
    }

    #[test]
    fn induction_nat_consumes_two_meta_allocations() {
        let state = start_nat_induction_goal();
        let (state, _) = run_machine_tactic(
            &state,
            MachineTactic::Intro {
                goal_id: GoalId(0),
                name: "n".to_owned(),
            },
        )
        .unwrap();
        let budget = TacticBudget {
            max_meta_allocations: 1,
            ..TacticBudget::default()
        };
        let err = run_machine_tactic_with_budget(
            &state,
            MachineTactic::InductionNat {
                goal_id: GoalId(1),
                local_name: "n".to_owned(),
            },
            budget,
        )
        .expect_err("induction-nat needs base and step metas");

        assert_eq!(
            err.kind,
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::MetaAllocation
            }
        );
        assert_eq!(state.open_goals, vec![GoalId(1)]);
    }

    #[test]
    fn intro_rejects_non_pi_before_tactic_step_fuel() {
        let state = start_trivial();
        let budget = TacticBudget {
            max_tactic_steps: 0,
            ..TacticBudget::default()
        };
        let err = run_machine_tactic_with_budget(
            &state,
            MachineTactic::Intro {
                goal_id: GoalId(0),
                name: "x".to_owned(),
            },
            budget,
        )
        .expect_err("non-Pi target should be reported before tactic-step fuel");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::TypeMismatch);
    }

    #[test]
    fn intro_tactic_consumes_dispatch_body_goal_and_lambda_steps() {
        let spec = MachineProofSpec {
            theorem_type: Expr::pi("p", prop(), prop()),
            ..trivial_spec()
        };
        let state = start_machine_proof(
            spec,
            Vec::new(),
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .expect("Pi theorem type should start");
        let budget = TacticBudget {
            max_tactic_steps: 2,
            max_expr_nodes: 0,
            ..TacticBudget::default()
        };
        let err = run_machine_tactic_with_budget(
            &state,
            MachineTactic::Intro {
                goal_id: GoalId(0),
                name: "p".to_owned(),
            },
            budget,
        )
        .expect_err("intro requires dispatch, body goal, and lambda step fuel");

        assert_eq!(
            err.kind,
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::TacticStep
            }
        );
        assert_eq!(state.open_goals, vec![GoalId(0)]);
    }

    #[test]
    fn intro_rejects_local_and_global_root_shadowing() {
        let spec = MachineProofSpec {
            theorem_type: Expr::pi("p", prop(), Expr::pi("q", prop(), prop())),
            ..trivial_spec()
        };
        let state = start_machine_proof(
            spec,
            Vec::new(),
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .unwrap();
        let (state, _) = run_machine_tactic(
            &state,
            MachineTactic::Intro {
                goal_id: GoalId(0),
                name: "p".to_owned(),
            },
        )
        .unwrap();
        let duplicate = run_machine_tactic(
            &state,
            MachineTactic::Intro {
                goal_id: GoalId(1),
                name: "p".to_owned(),
            },
        )
        .expect_err("intro should not reuse an existing local name");
        assert_eq!(
            duplicate.kind,
            MachineTacticDiagnosticKind::InvalidMachineTactic
        );

        let import =
            VerifiedImportRef::from_verified_module(&verified_axiom_module("Nat", "Nat.T"))
                .unwrap();
        let state = start_machine_proof(
            MachineProofSpec {
                theorem_type: Expr::pi("x", prop(), prop()),
                ..trivial_spec()
            },
            vec![import],
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .unwrap();
        let global_shadow = run_machine_tactic(
            &state,
            MachineTactic::Intro {
                goal_id: GoalId(0),
                name: "Nat".to_owned(),
            },
        )
        .expect_err("intro should follow Phase 3 local/global shadowing");
        assert_eq!(
            global_shadow.kind,
            MachineTacticDiagnosticKind::InvalidMachineTactic
        );
    }

    #[test]
    fn apply_local_assumption_closes_goal() {
        let state = start_machine_proof(
            MachineProofSpec {
                theorem_type: prop_id_type(),
                ..trivial_spec()
            },
            Vec::new(),
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .unwrap();
        let (state, _) = run_machine_tactic(
            &state,
            MachineTactic::Intro {
                goal_id: GoalId(0),
                name: "p".to_owned(),
            },
        )
        .unwrap();
        let (state, _) = run_machine_tactic(
            &state,
            MachineTactic::Intro {
                goal_id: GoalId(1),
                name: "hp".to_owned(),
            },
        )
        .unwrap();

        let (closed, delta) = run_machine_tactic(
            &state,
            MachineTactic::Apply {
                goal_id: GoalId(2),
                head: TacticHead::Local {
                    name: "hp".to_owned(),
                },
                universe_args: Vec::new(),
                args: Vec::new(),
            },
        )
        .expect("local hypothesis should apply directly to the goal");

        assert!(closed.open_goals.is_empty());
        assert_eq!(delta.added_goals, Vec::<GoalId>::new());
        assert_eq!(
            extract_closed_machine_proof(&closed).unwrap(),
            prop_id_proof()
        );
    }

    #[test]
    fn rw_local_eq_rewrites_whole_eq_side_and_orders_new_goal_last() {
        let state = start_rw_goal_with_local_eq();

        let (state, delta) = run_machine_tactic(
            &state,
            MachineTactic::Rewrite {
                goal_id: GoalId(3),
                rule: RewriteRuleRef {
                    head: TacticHead::Local {
                        name: "h".to_owned(),
                    },
                    universe_args: Vec::new(),
                    args: Vec::new(),
                },
                direction: RewriteDirection::Backward,
                site: RewriteSite::EqTargetRight,
            },
        )
        .expect("rw should rewrite the target Eq right side");

        assert_eq!(state.open_goals, vec![GoalId(4)]);
        assert_eq!(delta.added_goals, vec![GoalId(4)]);
        let rewritten = state.goal(GoalId(4)).unwrap();
        assert_eq!(rewritten.target, eq_nat(Expr::bvar(2), Expr::bvar(2)));

        let (closed, _) = assign_goal(
            &state,
            GoalId(4),
            ProofExpr::Core(eq_refl_nat(Expr::bvar(2))),
            Vec::new(),
        )
        .expect("rewritten reflexive goal should close");
        assert!(closed.open_goals.is_empty());
        extract_closed_machine_proof(&closed).expect("rw proof should extract and kernel-check");
        let theorem = extract_closed_machine_theorem_decl(&closed).unwrap();
        let cert = npa_cert::build_module_cert(
            npa_cert::CoreModule {
                name: Name::from_dotted("Test"),
                declarations: vec![theorem],
            },
            &[],
        )
        .expect("rw theorem should materialize as a certificate with builtin refs");
        let bytes = npa_cert::encode_module_cert(&cert).unwrap();
        let mut session = npa_cert::VerifierSession::new();
        npa_cert::verify_module_cert(&bytes, &mut session, &npa_cert::AxiomPolicy::normal())
            .expect("rw certificate should verify with builtin Eq.rec available");
    }

    #[test]
    fn rw_counts_target_rewrite_and_transport_tactic_steps() {
        let state = start_rw_goal_with_local_eq();
        let err = run_machine_tactic_with_budget(
            &state,
            MachineTactic::Rewrite {
                goal_id: GoalId(3),
                rule: RewriteRuleRef {
                    head: TacticHead::Local {
                        name: "h".to_owned(),
                    },
                    universe_args: Vec::new(),
                    args: Vec::new(),
                },
                direction: RewriteDirection::Backward,
                site: RewriteSite::EqTargetRight,
            },
            TacticBudget {
                max_tactic_steps: 2,
                ..TacticBudget::default()
            },
        )
        .expect_err("rw transport proof construction must consume tactic-step fuel");

        assert_eq!(
            err.kind,
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::TacticStep
            }
        );
        assert_eq!(state.open_goals, vec![GoalId(3)]);
    }

    #[test]
    fn rw_validates_universe_args_before_head_resolution() {
        let state = start_machine_proof(
            MachineProofSpec {
                theorem_type: eq_nat(nat_zero(), nat_zero()),
                ..trivial_spec()
            },
            Vec::new(),
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .unwrap();
        let err = run_machine_tactic(
            &state,
            MachineTactic::Rewrite {
                goal_id: GoalId(0),
                rule: RewriteRuleRef {
                    head: TacticHead::Local {
                        name: "missing".to_owned(),
                    },
                    universe_args: vec![Level::param("missing")],
                    args: Vec::new(),
                },
                direction: RewriteDirection::Forward,
                site: RewriteSite::EqTargetLeft,
            },
        )
        .expect_err("rw should reject malformed universe arguments");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::InvalidMachineTactic);
        assert!(err.message.contains("rw universe argument"));
    }

    #[test]
    fn rw_respects_whnf_and_conversion_fuel() {
        let state = start_rw_goal_with_local_eq();
        let whnf_err = run_machine_tactic_with_budget(
            &state,
            MachineTactic::Rewrite {
                goal_id: GoalId(3),
                rule: RewriteRuleRef {
                    head: TacticHead::Local {
                        name: "h".to_owned(),
                    },
                    universe_args: Vec::new(),
                    args: Vec::new(),
                },
                direction: RewriteDirection::Backward,
                site: RewriteSite::EqTargetRight,
            },
            TacticBudget {
                max_whnf_steps: 0,
                ..TacticBudget::default()
            },
        )
        .expect_err("rw must consume WHNF fuel while recognizing the Eq target");
        assert_eq!(
            whnf_err.kind,
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::Whnf
            }
        );

        let conversion_err = run_machine_tactic_with_budget(
            &state,
            MachineTactic::Rewrite {
                goal_id: GoalId(3),
                rule: RewriteRuleRef {
                    head: TacticHead::Local {
                        name: "h".to_owned(),
                    },
                    universe_args: Vec::new(),
                    args: Vec::new(),
                },
                direction: RewriteDirection::Backward,
                site: RewriteSite::EqTargetRight,
            },
            TacticBudget {
                max_conversion_steps: 0,
                ..TacticBudget::default()
            },
        )
        .expect_err("rw must consume conversion fuel while matching the rule");
        assert_eq!(
            conversion_err.kind,
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::Conversion
            }
        );
    }

    #[test]
    fn rw_reports_too_many_arguments_after_eq_conclusion() {
        let state = start_rw_goal_with_local_eq();
        let err = run_machine_tactic(
            &state,
            MachineTactic::Rewrite {
                goal_id: GoalId(3),
                rule: RewriteRuleRef {
                    head: TacticHead::Local {
                        name: "h".to_owned(),
                    },
                    universe_args: Vec::new(),
                    args: vec![ApplyArg::InferFromTarget],
                },
                direction: RewriteDirection::Backward,
                site: RewriteSite::EqTargetRight,
            },
        )
        .expect_err("extra rw arguments after the Eq conclusion must be rejected");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::TooManyApplyArguments);
    }

    #[test]
    fn simp_lite_closes_initial_refl_without_rewrite_fuel() {
        let state = start_machine_proof(
            MachineProofSpec {
                theorem_type: eq_nat(nat_zero(), nat_zero()),
                ..trivial_spec()
            },
            Vec::new(),
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .unwrap();
        let budget = TacticBudget {
            max_rewrite_steps: 0,
            ..TacticBudget::default()
        };
        let (closed, delta) = run_machine_tactic_with_budget(
            &state,
            MachineTactic::SimpLite {
                goal_id: GoalId(0),
                rules: Vec::new(),
            },
            budget,
        )
        .expect("simp-lite Eq.refl closure should not consume rewrite fuel");

        assert!(closed.open_goals.is_empty());
        assert_eq!(delta.added_goals, Vec::<GoalId>::new());
        assert_eq!(
            extract_closed_machine_proof(&closed).unwrap(),
            eq_refl_nat(nat_zero())
        );
    }

    #[test]
    fn simp_lite_reports_no_progress_after_refl_fails() {
        let state = start_machine_proof(
            MachineProofSpec {
                theorem_type: eq_nat(nat_zero(), nat_succ(nat_zero())),
                ..trivial_spec()
            },
            Vec::new(),
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .unwrap();
        let err = run_machine_tactic(
            &state,
            MachineTactic::SimpLite {
                goal_id: GoalId(0),
                rules: Vec::new(),
            },
        )
        .expect_err("simp-lite with no rules should report no progress");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::SimpNoProgress);
        assert_eq!(state.open_goals, vec![GoalId(0)]);
    }

    #[test]
    fn simp_lite_counts_initial_refl_and_scan_tactic_step() {
        let state = start_machine_proof(
            MachineProofSpec {
                theorem_type: eq_nat(nat_zero(), nat_succ(nat_zero())),
                ..trivial_spec()
            },
            Vec::new(),
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .unwrap();
        let err = run_machine_tactic_with_budget(
            &state,
            MachineTactic::SimpLite {
                goal_id: GoalId(0),
                rules: Vec::new(),
            },
            TacticBudget {
                max_tactic_steps: 0,
                ..TacticBudget::default()
            },
        )
        .expect_err("simp-lite should charge the initial Eq.refl/applicability scan step");

        assert_eq!(
            err.kind,
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::TacticStep
            }
        );
    }

    #[test]
    fn simp_lite_propagates_fuel_errors_from_rule_instantiation() {
        let rule = checked_current_theorem(
            "Test.zero_zero",
            eq_nat(nat_zero(), nat_zero()),
            eq_refl_nat(nat_zero()),
        );
        let state = start_machine_proof(
            MachineProofSpec {
                source_index: 1,
                theorem_type: eq_nat(nat_succ(nat_zero()), nat_zero()),
                ..trivial_spec()
            },
            Vec::new(),
            vec![rule.clone()],
            MachineTacticOptions {
                simp_rules: vec![SimpRuleRef {
                    name: Name::from_dotted("Test.zero_zero"),
                    decl_interface_hash: rule.signature().decl_interface_hash(),
                    direction: RewriteDirection::Forward,
                }],
                ..MachineTacticOptions::default()
            },
        )
        .unwrap();
        let err = run_machine_tactic_with_budget(
            &state,
            MachineTactic::SimpLite {
                goal_id: GoalId(0),
                rules: Vec::new(),
            },
            TacticBudget {
                max_tactic_steps: 0,
                ..TacticBudget::default()
            },
        )
        .expect_err("simp-lite must not classify fuel exhaustion as not-applicable");

        assert_eq!(
            err.kind,
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::TacticStep
            }
        );
    }

    #[test]
    fn simp_registry_accepts_universe_polymorphic_rules() {
        let u = Level::param("u");
        let poly_ty = Expr::pi(
            "A",
            Expr::sort(u.clone()),
            Expr::pi(
                "x",
                Expr::bvar(0),
                npa_kernel::eq(u.clone(), Expr::bvar(1), Expr::bvar(0), Expr::bvar(0)),
            ),
        );
        let poly_proof = Expr::lam(
            "A",
            Expr::sort(u.clone()),
            Expr::lam(
                "x",
                Expr::bvar(0),
                npa_kernel::eq_refl(u, Expr::bvar(1), Expr::bvar(0)),
            ),
        );
        let rule = checked_current_theorem_with_universes(
            "Test.poly_refl",
            vec!["u".to_owned()],
            poly_ty,
            poly_proof,
        );
        let state = start_machine_proof(
            MachineProofSpec {
                source_index: 1,
                theorem_type: eq_nat(nat_zero(), nat_succ(nat_zero())),
                ..trivial_spec()
            },
            Vec::new(),
            vec![rule.clone()],
            MachineTacticOptions {
                simp_rules: vec![SimpRuleRef {
                    name: Name::from_dotted("Test.poly_refl"),
                    decl_interface_hash: rule.signature().decl_interface_hash(),
                    direction: RewriteDirection::Forward,
                }],
                ..MachineTacticOptions::default()
            },
        )
        .expect("polymorphic simp rule should register");

        assert_eq!(state.env.simp_registry.rules[0].universe_params, ["u"]);
    }

    #[test]
    fn simp_registry_rejects_axiom_rule_heads() {
        let module = CoreModule {
            name: Name::from_dotted("Lib"),
            declarations: vec![Decl::Axiom {
                name: "Lib.zero_zero_axiom".to_owned(),
                universe_params: Vec::new(),
                ty: eq_nat(nat_zero(), nat_zero()),
            }],
        };
        let cert = npa_cert::build_module_cert(module, &[]).unwrap();
        let bytes = npa_cert::encode_module_cert(&cert).unwrap();
        let mut session = npa_cert::VerifierSession::new();
        let verified =
            npa_cert::verify_module_cert(&bytes, &mut session, &npa_cert::AxiomPolicy::normal())
                .unwrap();
        let import = VerifiedImportRef::from_verified_module(&verified).unwrap();
        let rule_hash = export_interface_hash(&import, "Lib.zero_zero_axiom");

        let err = start_machine_proof(
            MachineProofSpec {
                theorem_type: eq_nat(nat_zero(), nat_zero()),
                ..trivial_spec()
            },
            vec![import],
            Vec::new(),
            MachineTacticOptions {
                simp_rules: vec![SimpRuleRef {
                    name: Name::from_dotted("Lib.zero_zero_axiom"),
                    decl_interface_hash: rule_hash,
                    direction: RewriteDirection::Forward,
                }],
                ..MachineTacticOptions::default()
            },
        )
        .expect_err("axiom-backed simp rules must be rejected at registration time");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::InvalidSimpRule);
    }

    #[test]
    fn simp_registry_accepts_imported_verified_theorem_rules() {
        let import =
            VerifiedImportRef::from_verified_module(&verified_imported_simp_rule_module()).unwrap();
        let rule_hash = export_interface_hash(&import, "Lib.one_unfold");
        let state = start_machine_proof(
            MachineProofSpec {
                theorem_type: eq_nat(Expr::konst("Lib.one", Vec::new()), nat_zero()),
                ..trivial_spec()
            },
            vec![import],
            Vec::new(),
            MachineTacticOptions {
                simp_rules: vec![SimpRuleRef {
                    name: Name::from_dotted("Lib.one_unfold"),
                    decl_interface_hash: rule_hash,
                    direction: RewriteDirection::Forward,
                }],
                ..MachineTacticOptions::default()
            },
        )
        .expect("imported verified theorem should register as a simp rule");

        assert!(matches!(
            state.env.simp_registry.rules[0].source,
            TacticHead::Imported { .. }
        ));
        let (state, delta) = run_machine_tactic(
            &state,
            MachineTactic::SimpLite {
                goal_id: GoalId(0),
                rules: Vec::new(),
            },
        )
        .expect("imported theorem simp rule should rewrite the target");

        assert_eq!(delta.added_goals, vec![GoalId(1)]);
        assert_eq!(
            state.goal(GoalId(1)).unwrap().target,
            eq_nat(nat_succ(nat_zero()), nat_zero())
        );
    }

    #[test]
    fn simp_registry_rejects_opaque_def_rule_heads() {
        let rule = check_current_decl_for_machine_tactic_from_verified_imports(
            &[],
            &[],
            0,
            Decl::Def {
                name: "Test.opaque_zero_zero".to_owned(),
                universe_params: Vec::new(),
                ty: eq_nat(nat_zero(), nat_zero()),
                value: eq_refl_nat(nat_zero()),
                reducibility: Reducibility::Opaque,
            },
        )
        .unwrap();

        let err = start_machine_proof(
            MachineProofSpec {
                source_index: 1,
                theorem_type: eq_nat(nat_zero(), nat_zero()),
                ..trivial_spec()
            },
            Vec::new(),
            vec![rule.clone()],
            MachineTacticOptions {
                simp_rules: vec![SimpRuleRef {
                    name: Name::from_dotted("Test.opaque_zero_zero"),
                    decl_interface_hash: rule.signature().decl_interface_hash(),
                    direction: RewriteDirection::Forward,
                }],
                ..MachineTacticOptions::default()
            },
        )
        .expect_err("opaque proof definitions must not be registered as simp rules");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::InvalidSimpRule);
    }

    #[test]
    fn simp_registry_rejects_uninferable_term_binders() {
        let rule = checked_current_theorem(
            "Test.unused_param_rule",
            Expr::pi("x", nat(), eq_nat(nat_zero(), nat_zero())),
            Expr::lam("x", nat(), eq_refl_nat(nat_zero())),
        );

        let err = start_machine_proof(
            MachineProofSpec {
                source_index: 1,
                theorem_type: eq_nat(nat_zero(), nat_zero()),
                ..trivial_spec()
            },
            Vec::new(),
            vec![rule.clone()],
            MachineTacticOptions {
                simp_rules: vec![SimpRuleRef {
                    name: Name::from_dotted("Test.unused_param_rule"),
                    decl_interface_hash: rule.signature().decl_interface_hash(),
                    direction: RewriteDirection::Forward,
                }],
                ..MachineTacticOptions::default()
            },
        )
        .expect_err("uninferable term binders must be rejected when building the registry");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::InvalidSimpRule);
    }

    #[test]
    fn simp_registry_stores_whnf_theorem_type() {
        let alias = check_current_decl_for_machine_tactic_from_verified_imports(
            &[],
            &[],
            0,
            Decl::Def {
                name: "Test.zero_zero_ty".to_owned(),
                universe_params: Vec::new(),
                ty: prop(),
                value: eq_nat(nat_zero(), nat_zero()),
                reducibility: Reducibility::Reducible,
            },
        )
        .unwrap();
        let rule = check_current_decl_for_machine_tactic_from_verified_imports(
            &[],
            std::slice::from_ref(&alias),
            1,
            Decl::Theorem {
                name: "Test.alias_rule".to_owned(),
                universe_params: Vec::new(),
                ty: Expr::konst("Test.zero_zero_ty", Vec::new()),
                proof: eq_refl_nat(nat_zero()),
            },
        )
        .unwrap();

        let state = start_machine_proof(
            MachineProofSpec {
                source_index: 2,
                theorem_type: eq_nat(nat_zero(), nat_zero()),
                ..trivial_spec()
            },
            Vec::new(),
            vec![alias, rule.clone()],
            MachineTacticOptions {
                simp_rules: vec![SimpRuleRef {
                    name: Name::from_dotted("Test.alias_rule"),
                    decl_interface_hash: rule.signature().decl_interface_hash(),
                    direction: RewriteDirection::Forward,
                }],
                ..MachineTacticOptions::default()
            },
        )
        .expect("simp rule with reducible theorem type alias should register");

        assert_eq!(
            state.env.simp_registry.rules[0].theorem_ty,
            eq_nat(nat_zero(), nat_zero())
        );
    }

    #[test]
    fn simp_lite_uses_resolved_registry_patterns_instead_of_theorem_ty() {
        let one = check_current_decl_for_machine_tactic_from_verified_imports(
            &[],
            &[],
            0,
            Decl::Def {
                name: "Test.one".to_owned(),
                universe_params: Vec::new(),
                ty: nat(),
                value: nat_succ(nat_zero()),
                reducibility: Reducibility::Reducible,
            },
        )
        .unwrap();
        let rule_ty = eq_nat(Expr::konst("Test.one", Vec::new()), nat_succ(nat_zero()));
        let rule = check_current_decl_for_machine_tactic_from_verified_imports(
            &[],
            std::slice::from_ref(&one),
            1,
            Decl::Theorem {
                name: "Test.one_unfold".to_owned(),
                universe_params: Vec::new(),
                ty: rule_ty,
                proof: eq_refl_nat(nat_succ(nat_zero())),
            },
        )
        .unwrap();
        let mut state = start_machine_proof(
            MachineProofSpec {
                source_index: 2,
                theorem_type: eq_nat(Expr::konst("Test.one", Vec::new()), nat_zero()),
                ..trivial_spec()
            },
            Vec::new(),
            vec![one, rule.clone()],
            MachineTacticOptions {
                simp_rules: vec![SimpRuleRef {
                    name: Name::from_dotted("Test.one_unfold"),
                    decl_interface_hash: rule.signature().decl_interface_hash(),
                    direction: RewriteDirection::Forward,
                }],
                ..MachineTacticOptions::default()
            },
        )
        .expect("simp rule should register from its checked theorem type");

        state.env.simp_registry.rules[0].theorem_ty = prop();
        state.env.env_fingerprint = machine_tactic_env_hash(&state.env);
        refresh_state_identity(&mut state);
        validate_machine_proof_state(&state).unwrap();

        let (state, delta) = run_machine_tactic(
            &state,
            MachineTactic::SimpLite {
                goal_id: GoalId(0),
                rules: Vec::new(),
            },
        )
        .expect("simp-lite should use resolved patterns, not reparse theorem_ty");

        assert_eq!(delta.added_goals, vec![GoalId(1)]);
        assert_eq!(
            state.goal(GoalId(1)).unwrap().target,
            eq_nat(nat_succ(nat_zero()), nat_zero())
        );
    }

    #[test]
    fn apply_current_theorem_uses_positional_term_arguments() {
        let id = checked_current_theorem("Test.id", prop_id_type(), prop_id_proof());
        let state = start_machine_proof(
            MachineProofSpec {
                source_index: 1,
                theorem_type: prop_id_type(),
                ..trivial_spec()
            },
            Vec::new(),
            vec![id.clone()],
            MachineTacticOptions::default(),
        )
        .unwrap();
        let (state, _) = run_machine_tactic(
            &state,
            MachineTactic::Intro {
                goal_id: GoalId(0),
                name: "p".to_owned(),
            },
        )
        .unwrap();
        let (state, _) = run_machine_tactic(
            &state,
            MachineTactic::Intro {
                goal_id: GoalId(1),
                name: "hp".to_owned(),
            },
        )
        .unwrap();

        let (closed, _) = run_machine_tactic(
            &state,
            MachineTactic::Apply {
                goal_id: GoalId(2),
                head: TacticHead::CurrentModule {
                    name: Name::from_dotted("Test.id"),
                    decl_interface_hash: id.signature().decl_interface_hash(),
                },
                universe_args: Vec::new(),
                args: vec![
                    ApplyArg::Term(checked_term("p")),
                    ApplyArg::Term(checked_term("hp")),
                ],
            },
        )
        .expect("current theorem should apply with explicit term arguments");

        assert!(closed.open_goals.is_empty());
        extract_closed_machine_proof(&closed).expect("closed apply proof should extract");
    }

    #[test]
    fn apply_imported_theorem_infers_target_arg_and_orders_subgoals() {
        let import = VerifiedImportRef::from_verified_module(&verified_apply_module()).unwrap();
        let mp_hash = export_interface_hash(&import, "Lib.mp");
        let state = start_machine_proof(
            MachineProofSpec {
                theorem_type: Expr::pi(
                    "p",
                    prop(),
                    Expr::pi("hp", Expr::bvar(0), lib_q(Expr::bvar(1))),
                ),
                ..trivial_spec()
            },
            vec![import],
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .unwrap();
        let (state, _) = run_machine_tactic(
            &state,
            MachineTactic::Intro {
                goal_id: GoalId(0),
                name: "p".to_owned(),
            },
        )
        .unwrap();
        let (state, _) = run_machine_tactic(
            &state,
            MachineTactic::Intro {
                goal_id: GoalId(1),
                name: "hp".to_owned(),
            },
        )
        .unwrap();

        let (state, delta) = run_machine_tactic(
            &state,
            MachineTactic::Apply {
                goal_id: GoalId(2),
                head: TacticHead::Imported {
                    name: Name::from_dotted("Lib.mp"),
                    decl_interface_hash: mp_hash,
                },
                universe_args: Vec::new(),
                args: vec![
                    ApplyArg::InferFromTarget,
                    ApplyArg::Subgoal {
                        name_hint: Some("premise".to_owned()),
                    },
                ],
            },
        )
        .expect("imported theorem should infer p from the target and open premise goal");

        assert_eq!(state.open_goals, vec![GoalId(3)]);
        assert_eq!(delta.added_goals, vec![GoalId(3)]);
        let premise = state.goal(GoalId(3)).unwrap();
        assert_eq!(premise.context.len(), 2);
        assert_eq!(premise.target, Expr::bvar(1));

        let (closed, _) = run_machine_tactic(
            &state,
            MachineTactic::Exact {
                goal_id: GoalId(3),
                term: checked_term("hp"),
            },
        )
        .expect("premise subgoal should close with the local proof");
        assert!(closed.open_goals.is_empty());
        extract_closed_machine_proof(&closed).expect("closed imported apply proof should extract");
    }

    #[test]
    fn apply_rejects_local_let_head() {
        let mut state = start_trivial();
        state.metas.get_mut(MetaVarId(0)).unwrap().assignment = Some(ProofExpr::Core(prop()));
        state.metas.metas.insert(
            MetaVarId(1),
            MachineMetaVar {
                id: MetaVarId(1),
                goal_id: GoalId(1),
                context: vec![MachineLocalDecl::definition("x", type0(), prop())],
                target: type0(),
                assignment: None,
                creation_index: 1,
            },
        );
        state.metas.next_id = 2;
        state.open_goals = vec![GoalId(1)];
        refresh_state_identity(&mut state);
        validate_machine_proof_state(&state).unwrap();

        let err = run_machine_tactic(
            &state,
            MachineTactic::Apply {
                goal_id: GoalId(1),
                head: TacticHead::Local {
                    name: "x".to_owned(),
                },
                universe_args: Vec::new(),
                args: Vec::new(),
            },
        )
        .expect_err("local let declarations cannot be apply proof heads");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::InvalidLocalHead);
        assert_eq!(state.open_goals, vec![GoalId(1)]);
    }

    #[test]
    fn apply_validation_errors_include_goal_and_meta() {
        let state = start_trivial();
        let invalid_head = run_machine_tactic(
            &state,
            MachineTactic::Apply {
                goal_id: GoalId(0),
                head: TacticHead::Local {
                    name: "let".to_owned(),
                },
                universe_args: Vec::new(),
                args: Vec::new(),
            },
        )
        .expect_err("invalid apply head shape should be rejected before execution");

        assert_eq!(
            invalid_head.kind,
            MachineTacticDiagnosticKind::InvalidMachineTactic
        );
        assert_eq!(invalid_head.goal_id, Some(GoalId(0)));
        assert_eq!(invalid_head.meta_id, Some(MetaVarId(0)));

        let stale = MachineTermSource {
            source: "Prop".to_owned(),
            canonical_hash: ZERO_HASH,
        };
        let stale_term = run_machine_tactic(
            &state,
            MachineTactic::Apply {
                goal_id: GoalId(0),
                head: TacticHead::Local {
                    name: "h".to_owned(),
                },
                universe_args: Vec::new(),
                args: vec![ApplyArg::Term(stale)],
            },
        )
        .expect_err("stale apply term source hash should include goal context");

        assert_eq!(
            stale_term.kind,
            MachineTacticDiagnosticKind::InvalidMachineTermSource
        );
        assert_eq!(stale_term.goal_id, Some(GoalId(0)));
        assert_eq!(stale_term.meta_id, Some(MetaVarId(0)));
        assert_eq!(state.open_goals, vec![GoalId(0)]);
    }

    #[test]
    fn apply_rejects_missing_infer_and_data_subgoal() {
        let import = VerifiedImportRef::from_verified_module(&verified_apply_module()).unwrap();
        let mp_hash = export_interface_hash(&import, "Lib.mp");
        let drop_hash = export_interface_hash(&import, "Lib.drop");
        let state = start_machine_proof(
            MachineProofSpec {
                theorem_type: prop(),
                ..trivial_spec()
            },
            vec![import],
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .unwrap();
        let missing = run_machine_tactic(
            &state,
            MachineTactic::Apply {
                goal_id: GoalId(0),
                head: TacticHead::Imported {
                    name: Name::from_dotted("Lib.drop"),
                    decl_interface_hash: drop_hash,
                },
                universe_args: Vec::new(),
                args: vec![ApplyArg::InferFromTarget],
            },
        )
        .expect_err("InferFromTarget must occur in the rigid result pattern");
        assert_eq!(
            missing.kind,
            MachineTacticDiagnosticKind::MissingExplicitArgument
        );

        let data_subgoal = run_machine_tactic(
            &state,
            MachineTactic::Apply {
                goal_id: GoalId(0),
                head: TacticHead::Imported {
                    name: Name::from_dotted("Lib.mp"),
                    decl_interface_hash: mp_hash,
                },
                universe_args: Vec::new(),
                args: vec![ApplyArg::Subgoal { name_hint: None }],
            },
        )
        .expect_err("Subgoal cannot stand for a type parameter");
        assert_eq!(
            data_subgoal.kind,
            MachineTacticDiagnosticKind::SubgoalDataArgument
        );
    }

    #[test]
    fn apply_tactic_step_fuel_precedes_binder_semantics() {
        let import = VerifiedImportRef::from_verified_module(&verified_apply_module()).unwrap();
        let mp_hash = export_interface_hash(&import, "Lib.mp");
        let state = start_machine_proof(
            MachineProofSpec {
                theorem_type: prop(),
                ..trivial_spec()
            },
            vec![import],
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .unwrap();
        let err = run_machine_tactic_with_budget(
            &state,
            MachineTactic::Apply {
                goal_id: GoalId(0),
                head: TacticHead::Imported {
                    name: Name::from_dotted("Lib.mp"),
                    decl_interface_hash: mp_hash,
                },
                universe_args: Vec::new(),
                args: vec![ApplyArg::Subgoal { name_hint: None }],
            },
            TacticBudget {
                max_tactic_steps: 1,
                ..TacticBudget::default()
            },
        )
        .expect_err("binder consumption fuel must be checked before Subgoal domain semantics");

        assert_eq!(
            err.kind,
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::TacticStep
            }
        );
        assert_eq!(state.open_goals, vec![GoalId(0)]);
    }

    #[test]
    fn apply_tactic_step_fuel_precedes_matcher_and_final_construction() {
        let import = VerifiedImportRef::from_verified_module(&verified_apply_module()).unwrap();
        let mp_hash = export_interface_hash(&import, "Lib.mp");
        let state = start_machine_proof(
            MachineProofSpec {
                theorem_type: Expr::pi(
                    "p",
                    prop(),
                    Expr::pi("hp", Expr::bvar(0), lib_q(Expr::bvar(1))),
                ),
                ..trivial_spec()
            },
            vec![import],
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .unwrap();
        let (state, _) = run_machine_tactic(
            &state,
            MachineTactic::Intro {
                goal_id: GoalId(0),
                name: "p".to_owned(),
            },
        )
        .unwrap();
        let (state, _) = run_machine_tactic(
            &state,
            MachineTactic::Intro {
                goal_id: GoalId(1),
                name: "hp".to_owned(),
            },
        )
        .unwrap();

        let matcher_err = run_machine_tactic_with_budget(
            &state,
            MachineTactic::Apply {
                goal_id: GoalId(2),
                head: TacticHead::Imported {
                    name: Name::from_dotted("Lib.mp"),
                    decl_interface_hash: mp_hash,
                },
                universe_args: Vec::new(),
                args: vec![
                    ApplyArg::InferFromTarget,
                    ApplyArg::Subgoal {
                        name_hint: Some("premise".to_owned()),
                    },
                ],
            },
            TacticBudget {
                max_tactic_steps: 3,
                ..TacticBudget::default()
            },
        )
        .expect_err("matcher fuel must be checked before InferFromTarget matching");
        assert_eq!(
            matcher_err.kind,
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::TacticStep
            }
        );
        assert_eq!(state.open_goals, vec![GoalId(2)]);

        let direct_state = start_machine_proof(
            MachineProofSpec {
                theorem_type: prop_id_type(),
                ..trivial_spec()
            },
            Vec::new(),
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .unwrap();
        let (direct_state, _) = run_machine_tactic(
            &direct_state,
            MachineTactic::Intro {
                goal_id: GoalId(0),
                name: "p".to_owned(),
            },
        )
        .unwrap();
        let (direct_state, _) = run_machine_tactic(
            &direct_state,
            MachineTactic::Intro {
                goal_id: GoalId(1),
                name: "hp".to_owned(),
            },
        )
        .unwrap();
        let final_err = run_machine_tactic_with_budget(
            &direct_state,
            MachineTactic::Apply {
                goal_id: GoalId(2),
                head: TacticHead::Local {
                    name: "hp".to_owned(),
                },
                universe_args: Vec::new(),
                args: Vec::new(),
            },
            TacticBudget {
                max_tactic_steps: 1,
                ..TacticBudget::default()
            },
        )
        .expect_err("final proof construction fuel must be checked before closing the goal");
        assert_eq!(
            final_err.kind,
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::TacticStep
            }
        );
        assert_eq!(direct_state.open_goals, vec![GoalId(2)]);
    }

    #[test]
    fn rw_consumes_conversion_fuel_across_kernel_calls() {
        let rule = checked_current_theorem(
            "Test.zero_zero",
            eq_nat(nat_zero(), nat_zero()),
            eq_refl_nat(nat_zero()),
        );
        let state = start_machine_proof(
            MachineProofSpec {
                source_index: 1,
                theorem_type: eq_nat(nat_zero(), nat_succ(nat_zero())),
                ..trivial_spec()
            },
            Vec::new(),
            vec![rule.clone()],
            MachineTacticOptions::default(),
        )
        .unwrap();

        let err = run_machine_tactic_with_budget(
            &state,
            MachineTactic::Rewrite {
                goal_id: GoalId(0),
                rule: RewriteRuleRef {
                    head: TacticHead::CurrentModule {
                        name: Name::from_dotted("Test.zero_zero"),
                        decl_interface_hash: rule.signature().decl_interface_hash(),
                    },
                    universe_args: Vec::new(),
                    args: Vec::new(),
                },
                direction: RewriteDirection::Forward,
                site: RewriteSite::EqTargetLeft,
            },
            TacticBudget {
                max_conversion_steps: 5,
                ..TacticBudget::default()
            },
        )
        .expect_err("conversion fuel must be shared across rw kernel checks");

        assert_eq!(
            err.kind,
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::Conversion
            }
        );
        assert_eq!(state.open_goals, vec![GoalId(0)]);
    }

    #[test]
    fn apply_tactic_hash_excludes_goal_id_and_subgoal_name_hint() {
        let head = TacticHead::Local {
            name: "h".to_owned(),
        };
        let first = MachineTactic::Apply {
            goal_id: GoalId(0),
            head: head.clone(),
            universe_args: Vec::new(),
            args: vec![ApplyArg::Subgoal {
                name_hint: Some("left".to_owned()),
            }],
        };
        let second = MachineTactic::Apply {
            goal_id: GoalId(99),
            head,
            universe_args: Vec::new(),
            args: vec![ApplyArg::Subgoal {
                name_hint: Some("right".to_owned()),
            }],
        };

        assert_eq!(machine_tactic_hash(&first), machine_tactic_hash(&second));
        assert_ne!(
            machine_tactic_hash(&first),
            machine_tactic_hash(&MachineTactic::Intro {
                goal_id: GoalId(0),
                name: "h".to_owned(),
            })
        );
    }

    #[test]
    fn proof_delta_hash_uses_spec_payload_order() {
        let state = start_trivial();
        let specs = vec![MachineNewGoalSpec::new(Vec::new(), type0())];
        let (_, delta) = assign_goal(&state, GoalId(0), ProofExpr::Meta(MetaVarId(1)), specs)
            .expect("root can be assigned to a fresh subgoal");

        let mut out = Vec::new();
        encode_hash_to(&mut out, &delta.from_state_fingerprint);
        encode_goal_id_to(&mut out, delta.assigned_goal);
        encode_meta_var_id_to(&mut out, delta.assigned_meta);
        encode_hash_to(&mut out, &delta.proof_expr_hash);
        encode_list_len_to(&mut out, delta.added_goals.len());
        for goal in &delta.added_goals {
            encode_goal_id_to(&mut out, *goal);
        }
        encode_list_len_to(&mut out, delta.new_metas.len());
        for meta in &delta.new_metas {
            encode_meta_var_id_to(&mut out, meta.meta_id);
            encode_goal_id_to(&mut out, meta.goal_id);
            encode_hash_to(&mut out, &meta.context_hash);
            encode_hash_to(&mut out, &meta.target_hash);
        }
        encode_hash_to(&mut out, &delta.to_state_fingerprint);
        let expected_hash = hash_with_domain("npa.phase4.machine-proof-delta.v1", &out);

        assert_eq!(delta.delta_hash, expected_hash);
    }

    #[test]
    fn type_mismatch_hashes_use_diagnostic_payload_wrapper() {
        let state = start_trivial();
        let err = assign_goal(&state, GoalId(0), ProofExpr::Core(type0()), Vec::new())
            .expect_err("Sort 1 has type Sort 2, not Sort 1");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::TypeMismatch);
        let expected_hash = *err.expected_hash.as_ref().expect("expected hash").as_ref();
        let actual_hash = *err.actual_hash.as_ref().expect("actual hash").as_ref();
        assert_ne!(expected_hash, actual_hash);
        assert_ne!(expected_hash, core_expr_hash(&type0()));
    }

    #[test]
    fn assign_goal_rejects_old_unresolved_meta_and_is_transactional() {
        let state = start_trivial();
        let spec = MachineNewGoalSpec::new(Vec::new(), type0());
        let (state, _) = assign_goal(&state, GoalId(0), ProofExpr::Meta(MetaVarId(1)), vec![spec])
            .expect("root can be assigned to a fresh subgoal");
        let before = state.clone();
        let err = assign_goal(&state, GoalId(1), ProofExpr::Meta(MetaVarId(1)), Vec::new())
            .expect_err("a goal cannot solve itself through an old unresolved meta");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::InvalidMetaDependency);
        assert_eq!(state.fingerprint, before.fingerprint);
        assert_eq!(state.open_goals, before.open_goals);
    }

    #[test]
    fn assign_goal_allocates_new_metas_left_to_right_and_obeys_limits() {
        let state = start_trivial();
        let specs = vec![
            MachineNewGoalSpec::new(Vec::new(), type0()),
            MachineNewGoalSpec::new(Vec::new(), type0()),
        ];
        let (state, delta) = assign_goal(&state, GoalId(0), ProofExpr::Meta(MetaVarId(1)), specs)
            .expect("first new meta may be used in the assigned proof skeleton");

        assert_eq!(state.open_goals, vec![GoalId(1), GoalId(2)]);
        assert_eq!(
            delta
                .new_metas
                .iter()
                .map(|new_meta| new_meta.meta_id)
                .collect::<Vec<_>>(),
            vec![MetaVarId(1), MetaVarId(2)]
        );

        let state = start_trivial();
        let specs = vec![
            MachineNewGoalSpec::new(Vec::new(), type0()),
            MachineNewGoalSpec::new(Vec::new(), type0()),
        ];
        let budget = TacticBudget {
            max_meta_allocations: 1,
            ..TacticBudget::default()
        };
        let err = assign_goal_with_budget(
            &state,
            GoalId(0),
            ProofExpr::Meta(MetaVarId(1)),
            specs,
            budget,
        )
        .expect_err("allocation limit must be enforced");
        assert_eq!(
            err.kind,
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::MetaAllocation
            }
        );
    }

    #[test]
    fn zero_meta_allocation_budget_only_fails_when_allocation_is_needed() {
        let state = start_trivial();
        let budget = TacticBudget {
            max_meta_allocations: 0,
            ..TacticBudget::default()
        };

        assign_goal_with_budget(
            &state,
            GoalId(0),
            ProofExpr::Core(prop()),
            Vec::new(),
            budget,
        )
        .expect("zero meta allocation fuel still permits a proof with no new metas");

        let err = assign_goal_with_budget(
            &state,
            GoalId(0),
            ProofExpr::Meta(MetaVarId(1)),
            vec![MachineNewGoalSpec::new(Vec::new(), type0())],
            budget,
        )
        .expect_err("zero meta allocation fuel rejects fresh metas");

        assert_eq!(
            err.kind,
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::MetaAllocation
            }
        );
    }

    #[test]
    fn assign_goal_proof_check_consumes_conversion_fuel() {
        let state = start_trivial();
        let budget = TacticBudget {
            max_conversion_steps: 0,
            ..TacticBudget::default()
        };
        let err = assign_goal_with_budget(
            &state,
            GoalId(0),
            ProofExpr::Core(prop()),
            Vec::new(),
            budget,
        )
        .expect_err("proof skeleton type check must consume conversion fuel");

        assert_eq!(
            err.kind,
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::Conversion
            }
        );
        assert_eq!(state.open_goals, vec![GoalId(0)]);
    }

    #[test]
    fn kernel_check_reports_conversion_exhaustion_when_whnf_fuel_is_empty() {
        let env = Env::new();
        let fuel = TacticRunFuel::new(TacticBudget {
            max_whnf_steps: 0,
            max_conversion_steps: 0,
            ..TacticBudget::default()
        });

        let err = kernel_check_with_budget(
            &env,
            &Ctx::new(),
            &[],
            &prop(),
            &type0(),
            &fuel,
            GoalId(0),
            MetaVarId(0),
        )
        .expect_err("conversion exhaustion must not be misreported as WHNF exhaustion");

        assert_eq!(
            err.kind,
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::Conversion
            }
        );
    }

    #[test]
    fn tactic_entry_context_conversion_consumes_conversion_fuel() {
        let state = start_goal_with_local_definition(Expr::pi("p", prop(), prop()));

        let err = run_machine_tactic_with_budget(
            &state,
            MachineTactic::Intro {
                goal_id: GoalId(1),
                name: "p".to_owned(),
            },
            TacticBudget {
                max_conversion_steps: 0,
                ..TacticBudget::default()
            },
        )
        .expect_err("goal local context checking must share the tactic conversion budget");

        assert_eq!(
            err.kind,
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::Conversion
            }
        );
        assert_eq!(err.goal_id, Some(GoalId(1)));
        assert_eq!(err.meta_id, Some(MetaVarId(1)));
        assert_eq!(state.open_goals, vec![GoalId(1)]);
    }

    #[test]
    fn apply_head_lookup_precedes_context_fuel() {
        let state = start_goal_with_local_definition(type0());
        let err = run_machine_tactic_with_budget(
            &state,
            MachineTactic::Apply {
                goal_id: GoalId(1),
                head: TacticHead::CurrentModule {
                    name: Name::from_dotted("Test.missing"),
                    decl_interface_hash: ZERO_HASH,
                },
                universe_args: Vec::new(),
                args: Vec::new(),
            },
            TacticBudget {
                max_whnf_steps: 0,
                max_conversion_steps: 0,
                ..TacticBudget::default()
            },
        )
        .expect_err("apply head lookup must run before goal context kernel fuel is consumed");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::UnknownTacticHead);
        assert_eq!(err.goal_id, Some(GoalId(1)));
        assert_eq!(err.tactic_kind.as_deref(), Some("apply"));
        assert_eq!(err.primary_name, Some(Name::from_dotted("Test.missing")));
        assert_eq!(err.meta_id, Some(MetaVarId(1)));
        assert_eq!(state.open_goals, vec![GoalId(1)]);
    }

    #[test]
    fn rw_rule_lookup_precedes_context_fuel() {
        let state = start_goal_with_local_definition(eq_nat(nat_zero(), nat_zero()));
        let err = run_machine_tactic_with_budget(
            &state,
            MachineTactic::Rewrite {
                goal_id: GoalId(1),
                rule: RewriteRuleRef {
                    head: TacticHead::CurrentModule {
                        name: Name::from_dotted("Test.missing"),
                        decl_interface_hash: ZERO_HASH,
                    },
                    universe_args: Vec::new(),
                    args: Vec::new(),
                },
                direction: RewriteDirection::Forward,
                site: RewriteSite::EqTargetLeft,
            },
            TacticBudget {
                max_whnf_steps: 0,
                max_conversion_steps: 0,
                ..TacticBudget::default()
            },
        )
        .expect_err("rw rule lookup must run before goal context kernel fuel is consumed");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::UnknownTacticHead);
        assert_eq!(err.goal_id, Some(GoalId(1)));
        assert_eq!(err.tactic_kind.as_deref(), Some("rw"));
        assert_eq!(err.primary_name, Some(Name::from_dotted("Test.missing")));
        assert_eq!(err.meta_id, Some(MetaVarId(1)));
        assert_eq!(state.open_goals, vec![GoalId(1)]);
    }

    #[test]
    fn zero_tactic_step_budget_fails_after_input_validation() {
        let state = start_trivial();
        let budget = TacticBudget {
            max_tactic_steps: 0,
            ..TacticBudget::default()
        };
        let err = assign_goal_with_budget(
            &state,
            GoalId(0),
            ProofExpr::Core(prop()),
            Vec::new(),
            budget,
        )
        .expect_err("zero tactic step fuel should reject the first semantic transition");

        assert_eq!(
            err.kind,
            MachineTacticDiagnosticKind::TacticFuelExhausted {
                kind: TacticFuelKind::TacticStep
            }
        );
        assert_eq!(err.goal_id, Some(GoalId(0)));
        assert_eq!(err.tactic_kind, None);
        assert_eq!(state.open_goals, vec![GoalId(0)]);

        let unknown_goal_err = assign_goal_with_budget(
            &state,
            GoalId(99),
            ProofExpr::Core(prop()),
            Vec::new(),
            budget,
        )
        .expect_err("input validation should run before tactic step fuel consumption");

        assert_eq!(
            unknown_goal_err.kind,
            MachineTacticDiagnosticKind::UnknownGoal
        );
        assert_eq!(unknown_goal_err.goal_id, Some(GoalId(99)));
        assert_eq!(unknown_goal_err.tactic_kind, None);

        let pi_state = start_machine_proof(
            MachineProofSpec {
                theorem_type: Expr::pi("p", prop(), prop()),
                ..trivial_spec()
            },
            Vec::new(),
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .unwrap();
        let (pi_state, _) = run_machine_tactic(
            &pi_state,
            MachineTactic::Intro {
                goal_id: GoalId(0),
                name: "p".to_owned(),
            },
        )
        .unwrap();
        let invalid_new_goal_err = assign_goal_with_budget(
            &pi_state,
            GoalId(1),
            ProofExpr::Meta(MetaVarId(2)),
            vec![MachineNewGoalSpec::new(
                vec![MachineLocalDecl::assumption("q", prop())],
                prop(),
            )],
            budget,
        )
        .expect_err("new goal static validation should run before tactic step fuel");

        assert_eq!(
            invalid_new_goal_err.kind,
            MachineTacticDiagnosticKind::InvalidMetaContext
        );
        assert_eq!(invalid_new_goal_err.goal_id, Some(GoalId(1)));
        assert_eq!(invalid_new_goal_err.tactic_kind, None);
    }

    #[test]
    fn extraction_rejects_unresolved_goals() {
        let state = start_trivial();
        let err = extract_closed_machine_proof(&state)
            .expect_err("initial proof state still has an open goal");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::UnresolvedGoal);
    }

    #[test]
    fn state_validation_rejects_unopened_unassigned_metas() {
        let mut state = start_trivial();
        let meta_id = MetaVarId(1);
        state.metas.metas.insert(
            meta_id,
            MachineMetaVar {
                id: meta_id,
                goal_id: GoalId(1),
                context: Vec::new(),
                target: type0(),
                assignment: None,
                creation_index: 1,
            },
        );
        state.metas.next_id = 2;
        refresh_state_identity(&mut state);
        let err = validate_machine_proof_state(&state)
            .expect_err("unassigned metas must be exactly represented by open goals");

        assert_eq!(
            err.kind,
            MachineTacticDiagnosticKind::InvalidMachineProofState
        );
    }

    #[test]
    fn extract_closed_theorem_decl_uses_root_metadata_only() {
        let state = start_trivial();
        let (state, _) =
            assign_goal(&state, GoalId(0), ProofExpr::Core(prop()), Vec::new()).unwrap();
        let decl = extract_closed_machine_theorem_decl(&state).unwrap();

        assert_eq!(
            decl,
            Decl::Theorem {
                name: "Test.thm".to_owned(),
                universe_params: Vec::new(),
                ty: type0(),
                proof: prop(),
            }
        );
    }

    #[test]
    fn certificate_handoff_builds_and_verifies_closed_state() {
        let state = start_trivial();
        let (state, _) =
            assign_goal(&state, GoalId(0), ProofExpr::Core(prop()), Vec::new()).unwrap();
        let handoff = extract_closed_machine_certificate(&state)
            .expect("closed machine proof should hand off to a verified certificate");

        assert_eq!(handoff.core_module.name, Name::from_dotted("Test"));
        assert_eq!(handoff.core_module.declarations.len(), 1);
        assert_eq!(handoff.core_module.declarations[0].name(), "Test.thm");
        assert_eq!(
            handoff.verified_module.certificate_hash(),
            handoff.certificate.hashes.certificate_hash
        );
        assert_eq!(
            npa_cert::decode_module_cert(&handoff.certificate_bytes)
                .expect("handoff certificate bytes should decode"),
            handoff.certificate
        );
    }

    #[test]
    fn certificate_handoff_rejects_unresolved_goals() {
        let state = start_trivial();
        let err = extract_closed_machine_certificate(&state)
            .expect_err("open proof states must not be certificate handoff inputs");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::UnresolvedGoal);
    }

    #[test]
    fn certificate_handoff_includes_checked_current_prefix() {
        let prior_decl = Decl::Def {
            name: "Test.A".to_owned(),
            universe_params: Vec::new(),
            ty: Expr::sort(Level::succ(Level::succ(Level::zero()))),
            value: type0(),
            reducibility: Reducibility::Reducible,
        };
        let prior =
            check_current_decl_for_machine_tactic_from_verified_imports(&[], &[], 0, prior_decl)
                .unwrap();
        let state = start_machine_proof(
            MachineProofSpec {
                source_index: 1,
                theorem_type: Expr::konst("Test.A", Vec::new()),
                ..trivial_spec()
            },
            Vec::new(),
            vec![prior],
            MachineTacticOptions::default(),
        )
        .unwrap();
        let (state, _) =
            assign_goal(&state, GoalId(0), ProofExpr::Core(prop()), Vec::new()).unwrap();
        let handoff = extract_closed_machine_certificate(&state)
            .expect("checked current prefix should be included in certificate handoff");

        assert_eq!(handoff.core_module.declarations.len(), 2);
        assert_eq!(handoff.core_module.declarations[0].name(), "Test.A");
        assert_eq!(handoff.core_module.declarations[1].name(), "Test.thm");
    }

    #[test]
    fn certificate_handoff_excludes_tactic_metadata() {
        let direct = start_trivial();
        let (direct, _) =
            assign_goal(&direct, GoalId(0), ProofExpr::Core(prop()), Vec::new()).unwrap();

        let staged = start_trivial();
        let (staged, _) = assign_goal(
            &staged,
            GoalId(0),
            ProofExpr::Meta(MetaVarId(1)),
            vec![MachineNewGoalSpec::new(Vec::new(), type0())],
        )
        .unwrap();
        let (staged, _) =
            assign_goal(&staged, GoalId(1), ProofExpr::Core(prop()), Vec::new()).unwrap();

        assert_ne!(direct.fingerprint, staged.fingerprint);
        assert_eq!(
            extract_closed_machine_proof(&direct).unwrap(),
            extract_closed_machine_proof(&staged).unwrap()
        );

        let direct_handoff = extract_closed_machine_certificate(&direct).unwrap();
        let staged_handoff = extract_closed_machine_certificate(&staged).unwrap();

        assert_eq!(
            direct_handoff.certificate_bytes,
            staged_handoff.certificate_bytes
        );
    }

    #[test]
    fn checked_current_decls_are_rehashed_into_env_fingerprints() {
        let prior = check_current_decl_for_machine_tactic_from_verified_imports(
            &[],
            &[],
            0,
            Decl::Def {
                name: "Test.A".to_owned(),
                universe_params: Vec::new(),
                ty: type0(),
                value: prop(),
                reducibility: Reducibility::Reducible,
            },
        )
        .unwrap();
        let spec = MachineProofSpec {
            source_index: 1,
            theorem_type: Expr::konst("Test.A", Vec::new()),
            ..trivial_spec()
        };
        let state = start_machine_proof(
            spec,
            Vec::new(),
            vec![prior],
            MachineTacticOptions::default(),
        )
        .expect("checked prior declaration should be available to the theorem");

        assert_eq!(state.env.checked_current_decls[0].source_index(), 0);
        assert_ne!(
            state.env.checked_current_decls[0]
                .signature()
                .decl_interface_hash(),
            ZERO_HASH
        );
        assert_ne!(
            state.env.checked_current_decls[0].checked_env_fingerprint(),
            ZERO_HASH
        );
    }

    #[test]
    fn current_decl_signature_uses_phase2_interface_hash() {
        let decl = Decl::Def {
            name: "Test.A".to_owned(),
            universe_params: Vec::new(),
            ty: type0(),
            value: prop(),
            reducibility: Reducibility::Reducible,
        };
        let checked =
            check_current_decl_for_machine_tactic_from_verified_imports(&[], &[], 0, decl.clone())
                .unwrap();
        let cert = npa_cert::build_module_cert(
            npa_cert::CoreModule {
                name: Name::from_dotted("Test"),
                declarations: vec![decl],
            },
            &[],
        )
        .unwrap();

        assert_eq!(
            checked.signature().decl_interface_hash(),
            cert.declarations[0].hashes.decl_interface_hash
        );
    }

    #[test]
    fn verified_import_env_hashes_use_phase2_declaration_hashes() {
        let module = CoreModule {
            name: Name::from_dotted("A"),
            declarations: vec![Decl::Def {
                name: "A.T".to_owned(),
                universe_params: Vec::new(),
                ty: type0(),
                value: prop(),
                reducibility: Reducibility::Reducible,
            }],
        };
        let cert = npa_cert::build_module_cert(module, &[]).unwrap();
        let bytes = npa_cert::encode_module_cert(&cert).unwrap();
        let mut session = npa_cert::VerifierSession::new();
        let verified =
            npa_cert::verify_module_cert(&bytes, &mut session, &npa_cert::AxiomPolicy::normal())
                .unwrap();
        let import = VerifiedImportRef::from_verified_module(&verified).unwrap();

        assert_eq!(
            import.certified_env_decl_hashes(),
            &[cert.declarations[0].hashes.decl_certificate_hash]
        );
        assert_ne!(
            cert.declarations[0].hashes.decl_interface_hash,
            cert.declarations[0].hashes.decl_certificate_hash
        );
    }

    #[test]
    fn verified_builtin_imports_reuse_preloaded_kernel_profile() {
        let nat = VerifiedImportRef::from_verified_module(&verified_nat_builtin_module()).unwrap();
        let eq = VerifiedImportRef::from_verified_module(&verified_eq_builtin_module_with_rec())
            .unwrap();
        assert!(nat
            .certified_env_decls()
            .iter()
            .any(|decl| decl.name() == "Nat"));
        assert!(eq
            .certified_env_decls()
            .iter()
            .any(|decl| decl.name() == "Eq.rec"));

        let state = start_machine_proof(
            MachineProofSpec {
                theorem_type: eq_nat(nat_zero(), nat_zero()),
                ..trivial_spec()
            },
            vec![nat, eq],
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .expect("verified imports matching builtins should not collide with the builtin profile");

        assert!(state.env.kernel_env().decl("Nat").is_some());
        assert!(state.env.kernel_env().decl("Eq").is_some());
        assert!(state.env.kernel_env().decl("Eq.rec").is_some());
    }

    #[test]
    fn current_decl_chain_rejects_stale_core_declaration_hash() {
        let prior = check_current_decl_for_machine_tactic_from_verified_imports(
            &[],
            &[],
            0,
            Decl::Theorem {
                name: "Test.A".to_owned(),
                universe_params: Vec::new(),
                ty: type0(),
                proof: prop(),
            },
        )
        .unwrap();
        let next = check_current_decl_for_machine_tactic_from_verified_imports(
            &[],
            std::slice::from_ref(&prior),
            1,
            Decl::Def {
                name: "Test.B".to_owned(),
                universe_params: Vec::new(),
                ty: type0(),
                value: Expr::konst("Test.A", Vec::new()),
                reducibility: Reducibility::Reducible,
            },
        )
        .unwrap();
        let mut stale_prior = prior;
        stale_prior.core_decl = Decl::Theorem {
            name: "Test.A".to_owned(),
            universe_params: Vec::new(),
            ty: type0(),
            proof: Expr::let_in("p", type0(), prop(), Expr::bvar(0)),
        };
        let spec = MachineProofSpec {
            source_index: 2,
            theorem_type: Expr::konst("Test.B", Vec::new()),
            ..trivial_spec()
        };
        let err = start_machine_proof(
            spec,
            Vec::new(),
            vec![stale_prior, next],
            MachineTacticOptions::default(),
        )
        .expect_err("prior-chain fingerprint must commit to prior core declarations");

        assert_eq!(
            err.kind,
            MachineTacticDiagnosticKind::CurrentDeclSignatureMismatch
        );
    }

    #[test]
    fn checked_current_decls_must_belong_to_proof_module() {
        let prior = check_current_decl_for_machine_tactic_from_verified_imports(
            &[],
            &[],
            0,
            Decl::Def {
                name: "Other.A".to_owned(),
                universe_params: Vec::new(),
                ty: type0(),
                value: prop(),
                reducibility: Reducibility::Reducible,
            },
        )
        .unwrap();
        let spec = MachineProofSpec {
            source_index: 1,
            theorem_type: type0(),
            ..trivial_spec()
        };
        let err = start_machine_proof(
            spec,
            Vec::new(),
            vec![prior],
            MachineTacticOptions::default(),
        )
        .expect_err("checked current declarations must stay inside the proof module");

        assert_eq!(
            err.kind,
            MachineTacticDiagnosticKind::InvalidCurrentDeclOrder
        );
    }

    #[test]
    fn checked_current_decl_constructor_rejects_axioms() {
        let err = check_current_decl_for_machine_tactic_from_verified_imports(
            &[],
            &[],
            0,
            Decl::Axiom {
                name: "Test.bad".to_owned(),
                universe_params: Vec::new(),
                ty: type0(),
            },
        )
        .expect_err("current checked declarations must not be forgeable axioms");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::UncheckedCurrentDecl);
    }

    #[test]
    fn goal_api_only_returns_open_goals() {
        let state = start_trivial();
        let goal = state.goal(GoalId(0)).unwrap();
        assert_eq!(goal.id, GoalId(0));
        assert_eq!(goal.context_hash, machine_local_context_hash(&goal.context));
        assert_eq!(goal.target_hash, core_expr_hash(&goal.target));

        let (state, _) =
            assign_goal(&state, GoalId(0), ProofExpr::Core(prop()), Vec::new()).unwrap();
        let err = state
            .goal(GoalId(0))
            .expect_err("assigned root goal should no longer be exposed as open");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::UnknownGoal);
    }

    #[test]
    fn import_order_is_canonicalized_before_fingerprinting() {
        let import_a =
            VerifiedImportRef::from_verified_module(&verified_axiom_module("A", "A.T")).unwrap();
        let import_b =
            VerifiedImportRef::from_verified_module(&verified_axiom_module("B", "B.T")).unwrap();

        let state_ab = start_machine_proof(
            trivial_spec(),
            vec![import_a.clone(), import_b.clone()],
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .unwrap();
        let state_ba = start_machine_proof(
            trivial_spec(),
            vec![import_b, import_a],
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .unwrap();

        assert_eq!(state_ab.fingerprint, state_ba.fingerprint);
        assert_eq!(state_ab.env.imports[0].module, Name::from_dotted("A"));
        assert_eq!(state_ab.env.imports[1].module, Name::from_dotted("B"));
    }

    #[test]
    fn same_module_with_different_hashes_is_rejected() {
        let import_v1 =
            VerifiedImportRef::from_verified_module(&verified_axiom_module("A", "A.T")).unwrap();
        let import_v2 =
            VerifiedImportRef::from_verified_module(&verified_axiom_module("A", "A.U")).unwrap();

        let err = start_machine_proof(
            trivial_spec(),
            vec![import_v1, import_v2],
            Vec::new(),
            MachineTacticOptions::default(),
        )
        .expect_err("one module name cannot resolve to multiple verified hashes");

        assert_eq!(err.kind, MachineTacticDiagnosticKind::InvalidVerifiedImport);
    }
}
