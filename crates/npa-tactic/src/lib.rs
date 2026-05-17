use std::collections::{BTreeMap, BTreeSet};

use npa_cert::{
    CoreModule, DeclHashes, DeclPayload, ExportKind, Hash, ModuleName, Name, VerifiedModule,
};
use npa_kernel::subst::{instantiate, shift};
use npa_kernel::{Ctx, Decl, Env, Expr};
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
    UnsupportedTacticOption,
    InvalidMachineTactic,
    InvalidMachineTermSource,
    UnsupportedMachineTactic,
    TacticFuelExhausted { kind: TacticFuelKind },
    InvalidMachineProofState,
    InvalidMachineProofSpec,
    InvalidVerifiedImport,
    AmbiguousKernelEnvDecl,
    InvalidCurrentDeclOrder,
    CurrentDeclSignatureMismatch,
    UnknownGoal,
    AssignedGoal,
    GoalLimitExceeded,
    MetaLimitExceeded,
    InvalidMetaDependency,
    InvalidMetaContext,
    ProofExprScopeError,
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
            meta_id: None,
        }
    }

    fn with_expected_actual_payloads(
        mut self,
        payload_kind: DiagnosticPayloadKind,
        expected_payload: &[u8],
        actual_payload: &[u8],
    ) -> Self {
        self.expected_hash = Some(Box::new(expected_actual_diagnostic_hash(
            &self.kind,
            DiagnosticHashSide::Expected,
            payload_kind,
            expected_payload,
        )));
        self.actual_hash = Some(Box::new(expected_actual_diagnostic_hash(
            &self.kind,
            DiagnosticHashSide::Actual,
            payload_kind,
            actual_payload,
        )));
        self
    }

    fn with_goal(mut self, goal_id: GoalId) -> Self {
        self.goal_id = Some(goal_id);
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
        goal_id: GoalId,
        term: RawMachineTerm,
    },
    Intro {
        goal_id: GoalId,
        name: String,
    },
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
    pub rules: Vec<SimpRuleRef>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimpRuleRef {
    pub name: Name,
    pub decl_interface_hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EqFamilyRef {
    pub eq_name: Name,
    pub refl_name: Name,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NatFamilyRef {
    pub nat_name: Name,
    pub zero_name: Name,
    pub succ_name: Name,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedEqFamily {
    pub eq_name: Name,
    pub refl_name: Name,
    pub fingerprint: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedNatFamily {
    pub nat_name: Name,
    pub zero_name: Name,
    pub succ_name: Name,
    pub fingerprint: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineTacticOptions {
    pub max_simp_rewrite_steps: u64,
    pub max_open_goals: usize,
    pub max_metas: usize,
    pub eq_family: Option<EqFamilyRef>,
    pub nat_family: Option<NatFamilyRef>,
}

impl Default for MachineTacticOptions {
    fn default() -> Self {
        Self {
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
                MachineTacticDiagnosticKind::KernelRejected,
                format!(
                    "current declaration {name} is an axiom; checked current declarations must carry a kernel-checkable body"
                ),
            ));
        }
        Decl::Constructor { name, .. } | Decl::Recursor { name, .. } => {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::KernelRejected,
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
            MachineTacticDiagnosticKind::KernelRejected,
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

        let mut kernel_env = Env::new();
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
                add_decl_to_kernel_env(&mut kernel_env, decl.clone()).map_err(|err| {
                    MachineTacticDiagnostic::new(
                        MachineTacticDiagnosticKind::InvalidVerifiedImport,
                        format!(
                            "kernel env rejected import {} declaration {}: {err:?}",
                            import.module.as_dotted(),
                            decl.name()
                        ),
                    )
                })?;
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
                                MachineTacticDiagnosticKind::KernelRejected,
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

        let options_fingerprint = machine_tactic_options_hash(&options);
        let simp_registry = SimpRegistry::default();
        let mut env = Self {
            imports,
            checked_current_decls: normalized_current,
            simp_registry,
            eq_family: None,
            nat_family: None,
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
                MachineTacticDiagnosticKind::AssignedGoal,
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
    Assign {
        goal_id: GoalId,
        proof: ProofExpr,
        new_goals: Vec<MachineNewGoalSpec>,
    },
}

pub fn validate_machine_tactic_candidate(
    candidate: MachineTacticCandidate,
) -> Result<MachineTactic> {
    match candidate {
        MachineTacticCandidate::Exact { goal_id, term } => Ok(MachineTactic::Exact {
            goal_id,
            term: MachineTermSource::new_checked(term.source)?,
        }),
        MachineTacticCandidate::Intro { goal_id, name } => {
            validate_intro_name_shape(&name)?;
            Ok(MachineTactic::Intro { goal_id, name })
        }
    }
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
    validate_machine_proof_state(state)?;
    match tactic {
        MachineTactic::Exact { goal_id, term } => {
            run_exact_tactic_with_budget(state, goal_id, term, budget)
        }
        MachineTactic::Intro { goal_id, name } => {
            run_intro_tactic_with_budget(state, goal_id, name, budget)
        }
        MachineTactic::Assign {
            goal_id,
            proof,
            new_goals,
        } => assign_goal_with_budget(state, goal_id, proof, new_goals, budget),
    }
}

fn run_exact_tactic_with_budget(
    state: &MachineProofState,
    goal_id: GoalId,
    term: MachineTermSource,
    budget: TacticBudget,
) -> Result<(MachineProofState, MachineProofDelta)> {
    let goal = state.goal(goal_id)?;
    validate_machine_term_source(&term)?;
    let context = machine_term_elab_context(state, &goal.context)?;
    let checked = npa_frontend::elaborate_machine_term_check(
        term.source(),
        &context,
        &goal.target,
        &npa_frontend::MachineCompileOptions::default(),
    )
    .map_err(machine_term_elaboration_diag)?;
    ensure_tactic_step_fuel(budget, 2, goal_id, goal.meta_id)?;
    assign_goal_with_budget_and_steps(
        state,
        goal_id,
        ProofExpr::Core(checked.expr),
        Vec::new(),
        budget,
        0,
    )
}

fn run_intro_tactic_with_budget(
    state: &MachineProofState,
    goal_id: GoalId,
    name: String,
    budget: TacticBudget,
) -> Result<(MachineProofState, MachineProofDelta)> {
    let goal = state.goal(goal_id)?;
    validate_intro_name_shape(&name)?;
    validate_intro_name_available(state, &goal.context, &name)?;
    let ctx = local_context_to_ctx(
        state.env.kernel_env(),
        &goal.context,
        &state.root.universe_params,
    )?;
    let target_whnf = state
        .env
        .kernel_env()
        .whnf(&ctx, &state.root.universe_params, &goal.target)
        .map_err(kernel_diag)?;
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
    )
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
    assign_goal_with_budget_and_steps(state, goal_id, proof_expr, new_goal_specs, budget, 1)
}

fn assign_goal_with_budget_and_steps(
    state: &MachineProofState,
    goal_id: GoalId,
    proof_expr: ProofExpr,
    new_goal_specs: Vec<MachineNewGoalSpec>,
    budget: TacticBudget,
    required_tactic_steps: u64,
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
            MachineTacticDiagnosticKind::AssignedGoal,
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
        if !machine_local_context_is_prefix(&assigned_meta.context, &spec.context) {
            return Err(MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::InvalidMetaContext,
                "new goal context must extend the assigned goal context",
            )
            .with_goal(goal_id)
            .with_meta(assigned_meta_id));
        }
        let ctx = local_context_to_ctx(
            state.env.kernel_env(),
            &spec.context,
            &state.root.universe_params,
        )?;
        ensure_type_is_sort(
            state.env.kernel_env(),
            &ctx,
            &state.root.universe_params,
            &spec.target,
        )
        .map_err(|err| {
            MachineTacticDiagnostic::new(
                MachineTacticDiagnosticKind::KernelRejected,
                format!("kernel rejected new goal target: {err:?}"),
            )
            .with_goal(goal_id)
            .with_meta(assigned_meta_id)
        })?;
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
                &mut BTreeSet::new(),
            )?;
            let ctx = local_context_to_ctx(env, context, universe_params)?;
            if env
                .is_defeq(&ctx, universe_params, &checked.ty, expected)
                .map_err(kernel_diag)?
            {
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
) -> Result<CheckedProofExpr> {
    let ctx = local_context_to_ctx(env, context, universe_params)?;
    ensure_type_is_sort(env, &ctx, universe_params, ty).map_err(kernel_diag)?;
    let expected_whnf = env
        .whnf(&ctx, universe_params, expected)
        .map_err(kernel_diag)?;
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
    if !env
        .is_defeq(&ctx, universe_params, ty, &expected_ty)
        .map_err(kernel_diag)?
    {
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
) -> Result<CheckedProofExpr> {
    let ctx = local_context_to_ctx(env, context, universe_params)?;
    ensure_type_is_sort(env, &ctx, universe_params, ty).map_err(kernel_diag)?;
    let value_checked = check_proof_expr(
        env,
        metas,
        context,
        universe_params,
        value,
        ty,
        allowed_new_metas,
        assigning_meta,
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
    visiting: &mut BTreeSet<MetaVarId>,
) -> Result<CheckedProofExpr> {
    match expr {
        ProofExpr::Core(core) => {
            let ctx = local_context_to_ctx(env, context, universe_params)?;
            let ty = env
                .infer(&ctx, universe_params, core)
                .map_err(kernel_diag)?;
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
                visiting,
            )?;
            let ctx = local_context_to_ctx(env, context, universe_params)?;
            let fun_ty_whnf = env
                .whnf(&ctx, universe_params, &fun_checked.ty)
                .map_err(kernel_diag)?;
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
            let ctx = local_context_to_ctx(env, context, universe_params)?;
            ensure_type_is_sort(env, &ctx, universe_params, ty).map_err(kernel_diag)?;
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
            let ctx = local_context_to_ctx(env, context, universe_params)?;
            ensure_type_is_sort(env, &ctx, universe_params, ty).map_err(kernel_diag)?;
            let value_checked = check_proof_expr(
                env,
                metas,
                context,
                universe_params,
                value,
                ty,
                allowed_new_metas,
                assigning_meta,
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
    if options.eq_family.is_some() {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::UnsupportedTacticOption,
            "custom Eq family resolution is a Phase 4 post-M1 feature",
        ));
    }
    if options.nat_family.is_some() {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::UnsupportedTacticOption,
            "custom Nat family resolution is a Phase 4 post-M1 feature",
        ));
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
            MachineTacticDiagnosticKind::InvalidMetaContext,
            format!("intro name {name} already exists in the local context"),
        ));
    }
    if machine_global_roots(state).contains(name) {
        return Err(MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidMetaContext,
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
    npa_frontend::MachineTermElabContext::from_verified_modules_and_current_decls(
        &imports,
        &imports,
        &checked_current_decls,
        &current_generated_decls,
        local_context,
        state.root.universe_params.clone(),
    )
    .map_err(machine_term_elaboration_diag)
}

fn machine_term_elaboration_diag(err: npa_frontend::MachineDiagnostic) -> MachineTacticDiagnostic {
    match err.kind {
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
        npa_frontend::MachineDiagnosticKind::GlobalShadowedByLocal => MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidMetaContext,
            format!("machine term context is invalid: {}", err.message),
        ),
        npa_frontend::MachineDiagnosticKind::ParseError
        | npa_frontend::MachineDiagnosticKind::UnsupportedSyntax
        | npa_frontend::MachineDiagnosticKind::HoleNotAllowed => MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::InvalidMachineTermSource,
            format!("machine term source is invalid: {}", err.message),
        ),
        npa_frontend::MachineDiagnosticKind::KernelRejected => MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::KernelRejected,
            format!(
                "machine term elaboration was rejected by the kernel: {}",
                err.message
            ),
        ),
        _ => MachineTacticDiagnostic::new(
            MachineTacticDiagnosticKind::KernelRejected,
            format!("machine term elaboration failed: {}", err.message),
        ),
    }
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

fn local_context_to_ctx(
    env: &Env,
    context: &[MachineLocalDecl],
    universe_params: &[String],
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
        ensure_type_is_sort(env, &ctx, universe_params, &local.ty).map_err(kernel_diag)?;
        match &local.value {
            Some(value) => {
                env.check(&ctx, universe_params, value, &local.ty)
                    .map_err(kernel_diag)?;
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
            MachineTacticDiagnosticKind::KernelRejected,
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
            MachineTacticDiagnosticKind::InvalidMachineProofSpec,
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
    encode_string_to(&mut out, "builtin-none-v0.1");
    hash_with_domain("npa.phase4.kernel-check-profile.v1", &out)
}

fn encode_diagnostic_kind_to(out: &mut Vec<u8>, kind: &MachineTacticDiagnosticKind) {
    out.push(match kind {
        MachineTacticDiagnosticKind::InvalidTacticOption => 0x00,
        MachineTacticDiagnosticKind::UnsupportedTacticOption => 0x01,
        MachineTacticDiagnosticKind::InvalidMachineTactic => 0x15,
        MachineTacticDiagnosticKind::InvalidMachineTermSource => 0x16,
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
        MachineTacticDiagnosticKind::AssignedGoal => 0x0a,
        MachineTacticDiagnosticKind::GoalLimitExceeded => 0x0b,
        MachineTacticDiagnosticKind::MetaLimitExceeded => 0x0c,
        MachineTacticDiagnosticKind::InvalidMetaDependency => 0x0e,
        MachineTacticDiagnosticKind::InvalidMetaContext => 0x0f,
        MachineTacticDiagnosticKind::ProofExprScopeError => 0x10,
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
    });
}

fn encode_core_expr_bytes_to(out: &mut Vec<u8>, expr: &Expr) {
    out.extend_from_slice(&core_expr_canonical_bytes(expr));
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
    encode_list_len_to(out, 0);
    encode_option_eq_ref_to(out, options.eq_family.as_ref());
    encode_option_nat_ref_to(out, options.nat_family.as_ref());
    encode_u64_to(out, options.max_simp_rewrite_steps);
    encode_usize_to(out, options.max_open_goals);
    encode_usize_to(out, options.max_metas);
}

fn encode_simp_registry_to(out: &mut Vec<u8>, registry: &SimpRegistry) {
    encode_list_len_to(out, registry.rules.len());
    for rule in &registry.rules {
        encode_name_to(out, &rule.name);
        encode_hash_to(out, &rule.decl_interface_hash);
    }
}

fn encode_option_eq_ref_to(out: &mut Vec<u8>, value: Option<&EqFamilyRef>) {
    match value {
        Some(value) => {
            out.push(1);
            encode_name_to(out, &value.eq_name);
            encode_name_to(out, &value.refl_name);
        }
        None => out.push(0),
    }
}

fn encode_option_nat_ref_to(out: &mut Vec<u8>, value: Option<&NatFamilyRef>) {
    match value {
        Some(value) => {
            out.push(1);
            encode_name_to(out, &value.nat_name);
            encode_name_to(out, &value.zero_name);
            encode_name_to(out, &value.succ_name);
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

    fn checked_term(source: &str) -> MachineTermSource {
        MachineTermSource::new_checked(source).expect("test term source should canonicalize")
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
    fn m1_rejects_custom_eq_or_nat_resolution() {
        let options = MachineTacticOptions {
            eq_family: Some(EqFamilyRef {
                eq_name: Name::from_dotted("Eq"),
                refl_name: Name::from_dotted("Eq.refl"),
            }),
            ..MachineTacticOptions::default()
        };
        let err = start_machine_proof(trivial_spec(), Vec::new(), Vec::new(), options)
            .expect_err("custom Eq family is post-M1");

        assert_eq!(
            err.kind,
            MachineTacticDiagnosticKind::UnsupportedTacticOption
        );
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
        let first = validate_machine_tactic_candidate(MachineTacticCandidate::Exact {
            goal_id: GoalId(0),
            term: RawMachineTerm::new("Prop"),
        })
        .expect("exact candidate should validate");
        let second = validate_machine_tactic_candidate(MachineTacticCandidate::Exact {
            goal_id: GoalId(0),
            term: RawMachineTerm::new("  Prop  "),
        })
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
            MachineTacticDiagnosticKind::InvalidMetaContext
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
            MachineTacticDiagnosticKind::InvalidMetaContext
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
    fn zero_tactic_step_budget_fails_after_input_validation() {
        let state = start_trivial();
        let budget = TacticBudget {
            max_tactic_steps: 0,
            ..TacticBudget::default()
        };
        let err = run_machine_tactic_with_budget(
            &state,
            MachineTactic::Assign {
                goal_id: GoalId(0),
                proof: ProofExpr::Core(prop()),
                new_goals: Vec::new(),
            },
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
        assert_eq!(state.open_goals, vec![GoalId(0)]);

        let unknown_goal_err = run_machine_tactic_with_budget(
            &state,
            MachineTactic::Assign {
                goal_id: GoalId(99),
                proof: ProofExpr::Core(prop()),
                new_goals: Vec::new(),
            },
            budget,
        )
        .expect_err("input validation should run before tactic step fuel consumption");

        assert_eq!(
            unknown_goal_err.kind,
            MachineTacticDiagnosticKind::UnknownGoal
        );

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
        let invalid_new_goal_err = run_machine_tactic_with_budget(
            &pi_state,
            MachineTactic::Assign {
                goal_id: GoalId(1),
                proof: ProofExpr::Meta(MetaVarId(2)),
                new_goals: vec![MachineNewGoalSpec::new(Vec::new(), prop())],
            },
            budget,
        )
        .expect_err("new goal static validation should run before tactic step fuel");

        assert_eq!(
            invalid_new_goal_err.kind,
            MachineTacticDiagnosticKind::InvalidMetaContext
        );
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

        assert_eq!(err.kind, MachineTacticDiagnosticKind::KernelRejected);
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
