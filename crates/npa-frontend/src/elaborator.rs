use std::collections::BTreeMap;

use npa_kernel::{
    subst::{instantiate, shift},
    Binder, ConstructorDecl, Ctx, Decl, Env, Error as KernelError, Expr, InductiveDecl, Level,
    RecursorDecl, Reducibility,
};

use crate::{
    resolve_module, resolve_source, BinderInfo, Diagnostic, DiagnosticKind, ElabGlobalRef, FileId,
    ImplicitMode, ImportedTypeMetadata, Name, ResolvedBinder, ResolvedCtorDecl, ResolvedDecl,
    ResolvedExpr, ResolvedImport, ResolvedItem, ResolvedModule, ResolvedName, Result, Span,
    SurfaceBinderKind, SurfaceLevel, SurfaceModule, SurfaceName, SurfaceUniverseParam,
    VerifiedImport,
};

const MAX_NUMERIC_UNIVERSE_LEVEL: u64 = 1024;
const TERM_META_PREFIX: &str = "__npa_meta..term.";
const UNIVERSE_META_PREFIX: &str = "__npa_meta..univ.";
const SOLVER_FUEL: usize = 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElaboratedModule {
    pub current_module: Name,
    pub declarations: Vec<Decl>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn elaborate_source(
    file_id: FileId,
    current_module: Name,
    source: &str,
    verified_imports: &[VerifiedImport],
) -> Result<ElaboratedModule> {
    let resolved = resolve_source(file_id, current_module, source, verified_imports)?;
    elaborate_resolved_module(&resolved)
}

pub fn elaborate_module(
    current_module: Name,
    module: &SurfaceModule,
    verified_imports: &[VerifiedImport],
) -> Result<ElaboratedModule> {
    let resolved = resolve_module(current_module, module, verified_imports)?;
    elaborate_resolved_module_with_span(&resolved, module.span)
}

pub fn elaborate_resolved_module(resolved: &ResolvedModule) -> Result<ElaboratedModule> {
    let fallback_span = resolved
        .items
        .first()
        .map(resolved_item_span)
        .unwrap_or_else(|| Span::new(FileId(0), 0, 0));
    elaborate_resolved_module_with_span(resolved, fallback_span)
}

fn elaborate_resolved_module_with_span(
    resolved: &ResolvedModule,
    fallback_span: Span,
) -> Result<ElaboratedModule> {
    let elaborator = ModuleElaborator::new(fallback_span)?;
    elaborator.elaborate(resolved)
}

struct ModuleElaborator {
    env: Env,
    signatures: SignatureEnv,
    declarations: Vec<Decl>,
    diagnostics: Vec<Diagnostic>,
}

impl ModuleElaborator {
    fn new(fallback_span: Span) -> Result<Self> {
        let env = Env::with_builtins().map_err(|error| kernel_rejected(fallback_span, error))?;
        let signatures = SignatureEnv::from_env(&env);
        Ok(Self {
            env,
            signatures,
            declarations: Vec::new(),
            diagnostics: Vec::new(),
        })
    }

    fn elaborate(mut self, resolved: &ResolvedModule) -> Result<ElaboratedModule> {
        self.diagnostics.extend(resolved.diagnostics.clone());
        for item in &resolved.items {
            match item {
                ResolvedItem::Def(decl) => self.elaborate_value_decl(decl, ValueDeclKind::Def)?,
                ResolvedItem::Theorem(decl) => {
                    self.elaborate_value_decl(decl, ValueDeclKind::Theorem)?
                }
                ResolvedItem::Axiom(decl) => {
                    self.elaborate_value_decl(decl, ValueDeclKind::Axiom)?
                }
                ResolvedItem::Inductive {
                    name,
                    universe_params,
                    binders,
                    ty,
                    constructors,
                    span,
                } => self.elaborate_inductive_item(
                    name,
                    universe_params,
                    binders,
                    ty,
                    constructors,
                    *span,
                )?,
                ResolvedItem::Import {
                    module,
                    export_hash,
                    duplicate,
                    span,
                } => {
                    if !duplicate {
                        let import = resolved
                            .state
                            .imports
                            .iter()
                            .find(|import| {
                                &import.module == module && &import.export_hash == export_hash
                            })
                            .ok_or_else(|| {
                                Diagnostic::error(
                                    DiagnosticKind::KernelRejected,
                                    *span,
                                    format!(
                                        "resolved import `{module}` was not available for kernel handoff"
                                    ),
                                )
                            })?;
                        for decl in &import.kernel_declarations {
                            add_decl_to_env(&mut self.env, decl, *span)?;
                        }
                        insert_imported_signatures(&mut self.signatures, &self.env, import);
                    }
                }
                ResolvedItem::Open { .. }
                | ResolvedItem::Namespace { .. }
                | ResolvedItem::End { .. }
                | ResolvedItem::Notation(_) => {}
            }
        }

        Ok(ElaboratedModule {
            current_module: resolved.current_module.clone(),
            declarations: self.declarations,
            diagnostics: self.diagnostics,
        })
    }

    fn elaborate_value_decl(&mut self, decl: &ResolvedDecl, kind: ValueDeclKind) -> Result<()> {
        let universe_params = universe_param_names(&decl.universe_params)?;
        let mut engine = ExprElaborator::new(
            self.env.clone(),
            self.signatures.clone(),
            universe_params.clone(),
        );
        let binders = engine.elaborate_decl_binders(&decl.binders)?;
        let result_ty = engine.elab_type(&decl.ty)?;
        let closed_ty = close_pi(result_ty.core.clone(), &binders);
        let source_signature_metadata = close_metadata(result_ty.metadata.clone(), &binders);
        let name = decl.name.to_dotted();

        match kind {
            ValueDeclKind::Axiom => {
                let closed_ty = engine.finish_expr(closed_ty, decl.span)?;
                let metadata_ty = engine.whnf_closed_expr(&closed_ty, decl.span)?;
                self.env
                    .add_axiom(name.clone(), universe_params.clone(), closed_ty.clone())
                    .map_err(|error| kernel_rejected(decl.span, error))?;
                self.signatures.insert_decl_metadata(
                    name.clone(),
                    metadata_ty,
                    source_signature_metadata.clone(),
                );
                self.declarations.push(Decl::Axiom {
                    name,
                    universe_params,
                    ty: closed_ty,
                });
            }
            ValueDeclKind::Def => {
                let value = decl.value.as_ref().ok_or_else(|| {
                    Diagnostic::error(
                        DiagnosticKind::KernelRejected,
                        decl.span,
                        "definition is missing a value",
                    )
                })?;
                let body = engine.elab_check_result(value, &result_ty)?;
                let type_value_metadata = body.type_value_metadata.clone();
                let signature_metadata =
                    declaration_signature_metadata(&decl.ty, &binders, &result_ty, &body);
                let closed_value = close_lam(body.core, &binders);
                let closed_ty = engine.finish_expr(closed_ty, decl.span)?;
                let closed_value = engine.finish_expr(closed_value, decl.span)?;
                let metadata_ty = engine.whnf_closed_expr(&closed_ty, decl.span)?;
                self.env
                    .add_def(
                        name.clone(),
                        universe_params.clone(),
                        closed_ty.clone(),
                        closed_value.clone(),
                        Reducibility::Reducible,
                    )
                    .map_err(|error| kernel_rejected(decl.span, error))?;
                self.signatures
                    .insert_decl_metadata(name.clone(), metadata_ty, signature_metadata);
                if result_ty.sort_level.is_some() {
                    if let Some(metadata) = type_value_metadata {
                        self.signatures
                            .insert_type_value_metadata(name.clone(), metadata);
                    }
                }
                self.declarations.push(Decl::Def {
                    name,
                    universe_params,
                    ty: closed_ty,
                    value: closed_value,
                    reducibility: Reducibility::Reducible,
                });
            }
            ValueDeclKind::Theorem => {
                let proof = decl.value.as_ref().ok_or_else(|| {
                    Diagnostic::error(
                        DiagnosticKind::KernelRejected,
                        decl.span,
                        "theorem is missing a proof",
                    )
                })?;
                let body = engine.elab_check_result(proof, &result_ty)?;
                let signature_metadata =
                    declaration_signature_metadata(&decl.ty, &binders, &result_ty, &body);
                let closed_proof = close_lam(body.core, &binders);
                let closed_ty = engine.finish_expr(closed_ty, decl.span)?;
                let closed_proof = engine.finish_expr(closed_proof, decl.span)?;
                let metadata_ty = engine.whnf_closed_expr(&closed_ty, decl.span)?;
                self.env
                    .add_theorem(
                        name.clone(),
                        universe_params.clone(),
                        closed_ty.clone(),
                        closed_proof.clone(),
                    )
                    .map_err(|error| kernel_rejected(decl.span, error))?;
                self.signatures
                    .insert_decl_metadata(name.clone(), metadata_ty, signature_metadata);
                self.declarations.push(Decl::Theorem {
                    name,
                    universe_params,
                    ty: closed_ty,
                    proof: closed_proof,
                });
            }
        }

        Ok(())
    }

    fn elaborate_inductive_item(
        &mut self,
        name: &Name,
        universe_params: &[SurfaceUniverseParam],
        binders: &[ResolvedBinder],
        ty: &ResolvedExpr,
        constructors: &[ResolvedCtorDecl],
        span: Span,
    ) -> Result<()> {
        let universe_params = universe_param_names(universe_params)?;
        let name = name.to_dotted();
        let mut engine = ExprElaborator::new(
            self.env.clone(),
            self.signatures.clone(),
            universe_params.clone(),
        );

        let params = engine.elaborate_decl_binders(binders)?;
        let result_ty = engine.elab_type(ty)?;
        let (indices, sort) =
            engine.peel_inductive_result(&result_ty.core, &result_ty.metadata, span)?;
        let source_signature_metadata = close_metadata(result_ty.metadata.clone(), &params);

        let all_binders: Vec<_> = params.iter().chain(indices.iter()).cloned().collect();
        let closed_inductive_ty =
            engine.finish_expr(close_pi(Expr::sort(sort), &all_binders), span)?;
        let metadata_ty = engine.whnf_closed_expr(&closed_inductive_ty, span)?;
        let params = engine.zonk_core_binders_from(&params, 0);
        let indices = engine.zonk_core_binders_from(&indices, params.len());
        let sort = inductive_result_sort(&closed_inductive_ty, span)?;

        let mut temp_env = self.env.clone();
        temp_env
            .add_axiom(
                name.clone(),
                universe_params.clone(),
                closed_inductive_ty.clone(),
            )
            .map_err(|error| kernel_rejected(span, error))?;
        engine.env = temp_env;
        engine.signatures.insert_decl_metadata(
            name.clone(),
            metadata_ty,
            source_signature_metadata.clone(),
        );

        let mut constructor_decls = Vec::new();
        let mut constructor_signatures = Vec::new();
        for constructor in constructors {
            let ctor_ty = engine.elab_type(&constructor.ty)?;
            let source_metadata = close_metadata(ctor_ty.metadata.clone(), &params);
            let closed_ctor_ty =
                engine.finish_expr(close_pi(ctor_ty.core, &params), constructor.span)?;
            let metadata_ty = engine.whnf_closed_expr(&closed_ctor_ty, constructor.span)?;
            let ctor_name = constructor.name.to_dotted();
            constructor_signatures.push((ctor_name.clone(), metadata_ty, source_metadata));
            constructor_decls.push(ConstructorDecl::new(ctor_name, closed_ctor_ty));
        }

        let mut data = InductiveDecl::new(
            name.clone(),
            universe_params.clone(),
            params.iter().map(core_binder_to_kernel).collect(),
            indices.iter().map(core_binder_to_kernel).collect(),
            sort,
            constructor_decls,
            None,
        );
        data.recursor = generated_recursor_decl(&data, span)?;
        let recursor_signature = data.recursor.as_ref().map(|recursor| {
            (
                recursor.name.clone(),
                recursor.ty.clone(),
                generated_recursor_metadata(&data, &params),
            )
        });
        let inductive_ty = inductive_core_type(&data);

        self.env
            .add_inductive(data.clone())
            .map_err(|error| kernel_rejected(span, error))?;
        self.signatures.insert_decl_metadata(
            name.clone(),
            inductive_ty.clone(),
            source_signature_metadata,
        );
        for (name, metadata_ty, metadata) in constructor_signatures {
            self.signatures
                .insert_decl_metadata(name, metadata_ty, metadata);
        }
        if let Some((name, metadata_ty, metadata)) = recursor_signature {
            self.signatures
                .insert_decl_metadata(name, metadata_ty, metadata);
        }

        self.declarations.push(Decl::Inductive {
            name,
            universe_params,
            ty: inductive_ty,
            data: Box::new(data),
        });
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum ValueDeclKind {
    Def,
    Theorem,
    Axiom,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct TypeMetadata {
    binder_infos: Vec<BinderInfo>,
    domain_infos: Vec<TypeMetadata>,
}

impl TypeMetadata {
    fn is_empty(&self) -> bool {
        self.binder_infos.is_empty() && self.domain_infos.is_empty()
    }

    fn explicit_for_ty(ty: &Expr) -> Self {
        let mut binder_infos = Vec::new();
        let mut domain_infos = Vec::new();
        let mut cursor = ty;
        while let Expr::Pi {
            ty: domain, body, ..
        } = cursor
        {
            binder_infos.push(BinderInfo::Explicit);
            domain_infos.push(Self::explicit_for_ty(domain));
            cursor = body;
        }
        Self {
            binder_infos,
            domain_infos,
        }
    }

    fn normalize_for_ty(mut self, ty: &Expr) -> Self {
        let domains = pi_domains(ty);
        let pi_count = domains.len();
        if self.is_empty() {
            return Self::explicit_for_ty(ty);
        }
        if self.binder_infos.len() < pi_count {
            self.binder_infos.extend(std::iter::repeat_n(
                BinderInfo::Explicit,
                pi_count - self.binder_infos.len(),
            ));
        } else {
            self.binder_infos.truncate(pi_count);
        }

        if self.domain_infos.len() < pi_count {
            self.domain_infos.extend(
                domains[self.domain_infos.len()..]
                    .iter()
                    .map(|domain| Self::explicit_for_ty(domain)),
            );
        } else {
            self.domain_infos.truncate(pi_count);
        }
        for (domain_info, domain) in self.domain_infos.iter_mut().zip(domains) {
            *domain_info = std::mem::take(domain_info).normalize_for_ty(domain);
        }
        self
    }

    fn domain_for(&self, index: usize, ty: &Expr) -> Self {
        self.domain_infos
            .get(index)
            .cloned()
            .unwrap_or_else(|| Self::explicit_for_ty(ty))
    }

    fn after_binder(&self) -> Self {
        Self {
            binder_infos: self.binder_infos.get(1..).unwrap_or_default().to_vec(),
            domain_infos: self.domain_infos.get(1..).unwrap_or_default().to_vec(),
        }
    }

    fn from_imported(metadata: &ImportedTypeMetadata) -> Self {
        Self {
            binder_infos: metadata.binder_infos.clone(),
            domain_infos: metadata
                .domain_infos
                .iter()
                .map(Self::from_imported)
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct SignatureEnv {
    metadata: BTreeMap<String, TypeMetadata>,
    type_value_metadata: BTreeMap<String, TypeMetadata>,
}

impl SignatureEnv {
    fn from_env(env: &Env) -> Self {
        let mut signatures = Self::default();
        for name in ["Nat", "Nat.zero", "Nat.succ", "Nat.rec", "Eq", "Eq.refl"] {
            if let Some(decl) = env.decl(name) {
                let source_binders = match name {
                    "Eq" => vec![
                        BinderInfo::Implicit,
                        BinderInfo::Explicit,
                        BinderInfo::Explicit,
                    ],
                    "Eq.refl" => vec![BinderInfo::Implicit, BinderInfo::Explicit],
                    _ => Vec::new(),
                };
                signatures.insert_decl(name.to_owned(), decl.ty().clone(), source_binders);
            }
        }
        signatures
    }

    fn insert_decl(&mut self, name: String, ty: Expr, source_binders: Vec<BinderInfo>) {
        let metadata = TypeMetadata {
            binder_infos: source_binders,
            domain_infos: Vec::new(),
        }
        .normalize_for_ty(&ty);
        self.metadata.insert(name, metadata);
    }

    fn insert_decl_metadata(&mut self, name: String, metadata_ty: Expr, metadata: TypeMetadata) {
        self.metadata
            .insert(name, metadata.normalize_for_ty(&metadata_ty));
    }

    fn insert_type_value_metadata(&mut self, name: String, metadata: TypeMetadata) {
        if !metadata.is_empty() {
            self.type_value_metadata.insert(name, metadata);
        }
    }

    fn metadata_for(&self, name: &str, ty: &Expr) -> TypeMetadata {
        self.metadata
            .get(name)
            .cloned()
            .unwrap_or_else(|| TypeMetadata::explicit_for_ty(ty))
    }

    fn type_value_metadata_for(&self, name: &str) -> Option<TypeMetadata> {
        self.type_value_metadata.get(name).cloned()
    }
}

fn insert_imported_signatures(signatures: &mut SignatureEnv, env: &Env, import: &ResolvedImport) {
    for imported_decl in &import.declarations {
        let name = imported_decl.name.to_dotted();
        if let Some(decl) = env.decl(&name) {
            let metadata_ty = env
                .whnf(&Ctx::new(), decl.universe_params(), decl.ty())
                .unwrap_or_else(|_| decl.ty().clone());
            signatures.insert_decl_metadata(
                name.clone(),
                metadata_ty,
                TypeMetadata {
                    binder_infos: imported_decl.binder_infos.clone(),
                    domain_infos: imported_decl
                        .domain_infos
                        .iter()
                        .map(TypeMetadata::from_imported)
                        .collect(),
                },
            );
            if let Some(metadata) = &imported_decl.type_value_metadata {
                signatures.insert_type_value_metadata(name, TypeMetadata::from_imported(metadata));
            }
        }
    }
}

fn declaration_signature_metadata(
    decl_ty: &ResolvedExpr,
    binders: &[CoreBinder],
    result_ty: &TypeCore,
    body: &CheckResult,
) -> TypeMetadata {
    let body_metadata = if matches!(decl_ty, ResolvedExpr::Hole { .. })
        && result_ty.metadata.binder_infos.is_empty()
    {
        body.ty_metadata.clone()
    } else {
        result_ty.metadata.clone()
    };
    close_metadata(body_metadata, binders)
}

struct ExprElaborator {
    env: Env,
    signatures: SignatureEnv,
    ctx: Ctx,
    locals: Vec<LocalCtxEntry>,
    delta: Vec<String>,
    term_metas: Vec<TermMeta>,
    universe_metas: Vec<UniverseMeta>,
    named_holes: BTreeMap<String, TermMetaId>,
    constraints: Vec<Constraint>,
}

#[derive(Clone, Debug)]
struct TypeCore {
    core: Expr,
    metadata: TypeMetadata,
    sort_level: Option<Level>,
}

#[derive(Clone, Debug)]
struct InferResult {
    core: Expr,
    ty: Expr,
    ty_metadata: TypeMetadata,
    type_value_metadata: Option<TypeMetadata>,
}

#[derive(Clone, Debug)]
struct CheckResult {
    core: Expr,
    ty_metadata: TypeMetadata,
    type_value_metadata: Option<TypeMetadata>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LocalCtxEntry {
    ty: Expr,
    ty_metadata: TypeMetadata,
    value_type_metadata: Option<TypeMetadata>,
    value: Option<Expr>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct TermMetaId(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct UniverseMetaId(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TermMetaKind {
    UserHole,
    SyntheticImplicit,
}

#[derive(Clone, Debug)]
struct TermMeta {
    name: Option<String>,
    context: Vec<LocalCtxEntry>,
    ty: Expr,
    kind: TermMetaKind,
    span: Span,
    assignment: Option<Expr>,
}

#[derive(Clone, Debug)]
struct UniverseMeta {
    span: Span,
    default: Option<Level>,
    assignment: Option<Level>,
}

#[derive(Clone, Debug)]
enum Constraint {
    TypeEq {
        lhs: Expr,
        rhs: Expr,
        context: Vec<LocalCtxEntry>,
        span: Span,
    },
    TermEq {
        ty: Expr,
        lhs: Expr,
        rhs: Expr,
        context: Vec<LocalCtxEntry>,
        span: Span,
    },
    LevelEq {
        lhs: Level,
        rhs: Level,
        span: Span,
    },
    LevelLe {
        lhs: Level,
        rhs: Level,
        span: Span,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SolveStatus {
    Solved,
    Stuck,
}

#[derive(Clone)]
struct ElabSnapshot {
    ctx: Ctx,
    locals: Vec<LocalCtxEntry>,
    term_metas: Vec<TermMeta>,
    universe_metas: Vec<UniverseMeta>,
    named_holes: BTreeMap<String, TermMetaId>,
    constraints: Vec<Constraint>,
    delta: Vec<String>,
}

impl ExprElaborator {
    fn new(env: Env, signatures: SignatureEnv, delta: Vec<String>) -> Self {
        Self {
            env,
            signatures,
            ctx: Ctx::new(),
            locals: Vec::new(),
            delta,
            term_metas: Vec::new(),
            universe_metas: Vec::new(),
            named_holes: BTreeMap::new(),
            constraints: Vec::new(),
        }
    }

    fn snapshot(&self) -> ElabSnapshot {
        ElabSnapshot {
            ctx: self.ctx.clone(),
            locals: self.locals.clone(),
            term_metas: self.term_metas.clone(),
            universe_metas: self.universe_metas.clone(),
            named_holes: self.named_holes.clone(),
            constraints: self.constraints.clone(),
            delta: self.delta.clone(),
        }
    }

    fn restore(&mut self, snapshot: ElabSnapshot) {
        self.ctx = snapshot.ctx;
        self.locals = snapshot.locals;
        self.term_metas = snapshot.term_metas;
        self.universe_metas = snapshot.universe_metas;
        self.named_holes = snapshot.named_holes;
        self.constraints = snapshot.constraints;
        self.delta = snapshot.delta;
    }

    fn elaborate_decl_binders(&mut self, binders: &[ResolvedBinder]) -> Result<Vec<CoreBinder>> {
        self.elaborate_typed_binder_groups(
            binders,
            DiagnosticKind::ExpectedSort,
            "declaration binder must have a type annotation",
        )
    }

    fn elab_type(&mut self, expr: &ResolvedExpr) -> Result<TypeCore> {
        if let ResolvedExpr::Hole { name, span } = expr {
            let level = self.fresh_universe_meta(*span);
            let sort = Expr::sort(level.clone());
            let meta = self.fresh_or_reuse_user_hole(name.as_ref(), sort, *span)?;
            return Ok(TypeCore {
                core: term_meta_expr(meta),
                metadata: TypeMetadata::default(),
                sort_level: Some(level),
            });
        }

        let span = resolved_expr_span(expr);
        let inferred = self.elab_infer(expr)?;
        let mut inferred_ty = self
            .env
            .whnf(
                &self.ctx,
                &self.delta,
                &self.zonk_current_expr(&inferred.ty),
            )
            .map_err(|error| diagnostic_from_kernel_error(span, error))?;
        if !matches!(inferred_ty, Expr::Sort(_)) && self.expr_contains_term_meta(&inferred_ty) {
            let level = self.fresh_type_hole_sort_level(span);
            self.ensure_type_eq(&inferred.ty, &Expr::sort(level), span)?;
            inferred_ty = self
                .env
                .whnf(
                    &self.ctx,
                    &self.delta,
                    &self.zonk_current_expr(&inferred.ty),
                )
                .map_err(|error| diagnostic_from_kernel_error(span, error))?;
        }

        match inferred_ty {
            Expr::Sort(level) => {
                let type_core = self
                    .env
                    .whnf(
                        &self.ctx,
                        &self.delta,
                        &self.zonk_current_expr(&inferred.core),
                    )
                    .map_err(|error| diagnostic_from_kernel_error(span, error))?;
                let metadata = inferred
                    .type_value_metadata
                    .or_else(|| self.type_value_metadata_for_core(&inferred.core))
                    .unwrap_or_else(|| source_type_metadata(expr))
                    .normalize_for_ty(&type_core);
                Ok(TypeCore {
                    core: inferred.core,
                    metadata,
                    sort_level: Some(level),
                })
            }
            actual => Err(Diagnostic::error(
                DiagnosticKind::ExpectedSort,
                span,
                format!("expected a type, found `{actual:?}`"),
            )),
        }
    }

    fn peel_inductive_result(
        &self,
        core: &Expr,
        metadata: &TypeMetadata,
        span: Span,
    ) -> Result<(Vec<CoreBinder>, Level)> {
        let mut ctx = self.ctx.clone();
        let mut context_len = self.locals.len();
        let mut cursor = core.clone();
        let mut metadata = metadata.clone();
        let mut indices = Vec::new();

        loop {
            let cursor_zonked = self.zonk_expr_in_context(&cursor, context_len);
            if self.expr_contains_term_meta(&cursor_zonked) {
                return Err(Diagnostic::error(
                    DiagnosticKind::UnsolvedHole,
                    span,
                    "inductive result type contains an unsolved hole",
                ));
            }
            if self.level_contains_universe_meta_in_expr(&cursor_zonked) {
                return Err(Diagnostic::error(
                    DiagnosticKind::UnsolvedUniverseMeta,
                    span,
                    "inductive result type contains an unsolved universe metavariable",
                ));
            }

            match self
                .env
                .whnf(&ctx, &self.delta, &cursor_zonked)
                .map_err(|error| diagnostic_from_kernel_error(span, error))?
            {
                Expr::Pi { binder, ty, body } => {
                    let binder_info = metadata
                        .binder_infos
                        .first()
                        .cloned()
                        .unwrap_or(BinderInfo::Explicit);
                    let ty_metadata = metadata.domain_for(0, &ty);
                    let ty = (*ty).clone();
                    ctx.push_assumption(binder.clone(), ty.clone());
                    indices.push(CoreBinder {
                        name: binder,
                        ty,
                        binder_info,
                        ty_metadata,
                        sort_level: None,
                    });
                    metadata = metadata.after_binder();
                    context_len += 1;
                    cursor = *body;
                }
                Expr::Sort(level) => return Ok((indices, level)),
                actual => {
                    return Err(Diagnostic::error(
                        DiagnosticKind::ExpectedSort,
                        span,
                        format!("expected inductive result to end in a Sort, found `{actual:?}`"),
                    ));
                }
            }
        }
    }

    fn elab_infer(&mut self, expr: &ResolvedExpr) -> Result<InferResult> {
        match expr {
            ResolvedExpr::Ident {
                resolved,
                universe_args,
                span,
                implicit_mode,
                ..
            } => {
                let levels = self.elab_universe_args(resolved, universe_args.as_deref(), *span)?;
                let (core, ty_metadata) = match resolved {
                    ResolvedName::Local(local) => {
                        if universe_args.is_some() {
                            return Err(Diagnostic::error(
                                DiagnosticKind::UnsolvedUniverseMeta,
                                *span,
                                "local names do not accept universe arguments",
                            ));
                        }
                        let ty_metadata = self
                            .local_info(local.de_bruijn_index, *span)?
                            .ty_metadata
                            .clone();
                        (Expr::bvar(local.de_bruijn_index), ty_metadata)
                    }
                    ResolvedName::Global(global) => {
                        let name = global_name(global);
                        let ty_metadata = self
                            .env
                            .decl(&name)
                            .map(|decl| self.signatures.metadata_for(&name, decl.ty()))
                            .unwrap_or_default();
                        (Expr::konst(name, levels), ty_metadata)
                    }
                    ResolvedName::Overloaded(candidates) => {
                        return self.elab_infer_overloaded_name(
                            candidates,
                            universe_args.as_deref(),
                            *implicit_mode,
                            *span,
                        );
                    }
                };
                let ty = self
                    .env
                    .infer(&self.ctx, &self.delta, &core)
                    .map_err(|error| diagnostic_from_kernel_error(*span, error))?;
                let type_value_metadata = self.type_value_metadata_for_core(&core);
                Ok(InferResult {
                    core,
                    ty,
                    ty_metadata,
                    type_value_metadata,
                })
            }
            ResolvedExpr::Sort { level, span } => {
                let core = Expr::sort(self.elab_level(level)?);
                let ty = self
                    .env
                    .infer(&self.ctx, &self.delta, &core)
                    .map_err(|error| diagnostic_from_kernel_error(*span, error))?;
                Ok(InferResult {
                    core,
                    ty,
                    ty_metadata: TypeMetadata::default(),
                    type_value_metadata: None,
                })
            }
            ResolvedExpr::App { func, arg, span } => {
                if let Some(spine) = overloaded_app_spine(expr) {
                    self.elab_infer_overloaded_app(
                        &spine.candidates,
                        spine.universe_args.as_deref(),
                        spine.implicit_mode,
                        &spine.args,
                        *span,
                    )
                } else {
                    self.elab_infer_app(func, arg, *span)
                }
            }
            ResolvedExpr::Lam {
                binders,
                body,
                span,
            } => self.elab_infer_lam(binders, body, *span),
            ResolvedExpr::Pi {
                binders,
                body,
                span,
            } => self.elab_infer_pi(binders, body, *span),
            ResolvedExpr::Let {
                name,
                ty,
                value,
                body,
                span,
                ..
            } => self.elab_infer_let(name.parts.join("."), ty.as_deref(), value, body, *span),
            ResolvedExpr::Annot { expr, ty, .. } => {
                let expected = self.elab_type(ty)?;
                let checked = self.elab_check_result(expr, &expected)?;
                let ty_metadata = annotation_metadata(ty, &expected, &checked);
                Ok(InferResult {
                    core: checked.core,
                    ty: expected.core,
                    ty_metadata,
                    type_value_metadata: checked.type_value_metadata,
                })
            }
            ResolvedExpr::Hole { name, span } => self.elab_infer_hole(name.as_ref(), *span),
            ResolvedExpr::Notation {
                head,
                candidates,
                args,
                span,
            } => self.elab_infer_notation(head, candidates, args, *span),
        }
    }

    fn elab_infer_app(
        &mut self,
        func: &ResolvedExpr,
        arg: &ResolvedExpr,
        span: Span,
    ) -> Result<InferResult> {
        let auto_insert = implicit_insertion_enabled_for_head(func);
        let mut result = self.elab_infer(func)?;
        result = self.insert_implicit_args(result, auto_insert, true, span)?;

        let func_ty = self
            .env
            .whnf(&self.ctx, &self.delta, &self.zonk_current_expr(&result.ty))
            .map_err(|error| diagnostic_from_kernel_error(span, error))?;
        let Expr::Pi { ty, body, .. } = func_ty else {
            return Err(Diagnostic::error(
                DiagnosticKind::ExpectedFunctionType,
                span,
                "expected function type in application",
            ));
        };

        let arg_core = self.elab_check(
            arg,
            &TypeCore {
                core: (*ty).clone(),
                metadata: result.ty_metadata.domain_for(0, &ty),
                sort_level: None,
            },
        )?;
        let core = Expr::app(result.core, arg_core.clone());
        let result_ty = instantiate(&body, &arg_core)
            .map_err(|error| diagnostic_from_kernel_error(span, error))?;
        let type_value_metadata = self.type_value_metadata_for_core(&core);
        Ok(InferResult {
            core,
            ty: result_ty,
            ty_metadata: result.ty_metadata.after_binder(),
            type_value_metadata,
        })
    }

    fn elab_infer_hole(
        &mut self,
        name: Option<&crate::SurfaceName>,
        span: Span,
    ) -> Result<InferResult> {
        let level = self.fresh_universe_meta(span);
        let ty_ty = Expr::sort(level);
        let ty_meta = self.fresh_term_meta(None, ty_ty, TermMetaKind::SyntheticImplicit, span);
        let ty = term_meta_expr(ty_meta);
        let term_meta = self.fresh_or_reuse_user_hole(name, ty.clone(), span)?;
        Ok(InferResult {
            core: term_meta_expr(term_meta),
            ty,
            ty_metadata: TypeMetadata::default(),
            type_value_metadata: None,
        })
    }

    fn elab_infer_overloaded_name(
        &mut self,
        candidates: &[ElabGlobalRef],
        universe_args: Option<&[SurfaceLevel]>,
        implicit_mode: ImplicitMode,
        span: Span,
    ) -> Result<InferResult> {
        let mut successes = Vec::new();
        let initial = self.snapshot();
        for candidate in sorted_global_candidates(candidates) {
            let snapshot = self.snapshot();
            let expr = resolved_ident_for_global(&candidate, universe_args, implicit_mode, span);
            if let Ok(result) = self.elab_infer(&expr).and_then(|result| {
                self.complete_candidate_since(&snapshot, span)?;
                Ok(result)
            }) {
                successes.push((candidate, self.snapshot(), result));
            }
            self.restore(snapshot);
        }

        match successes.len() {
            0 => {
                self.restore(initial);
                Err(Diagnostic::error(
                    DiagnosticKind::AmbiguousName,
                    span,
                    "overloaded name could not be resolved",
                ))
            }
            1 => {
                let (_, snapshot, result) = successes.remove(0);
                self.restore(snapshot);
                Ok(result)
            }
            _ => {
                self.restore(initial);
                Err(Diagnostic::error(
                    DiagnosticKind::AmbiguousName,
                    span,
                    "overloaded name is ambiguous",
                ))
            }
        }
    }

    fn elab_check_overloaded_name(
        &mut self,
        candidates: &[ElabGlobalRef],
        universe_args: Option<&[SurfaceLevel]>,
        implicit_mode: ImplicitMode,
        expected: &TypeCore,
        span: Span,
    ) -> Result<CheckResult> {
        let mut successes = Vec::new();
        let initial = self.snapshot();
        for candidate in sorted_global_candidates(candidates) {
            let snapshot = self.snapshot();
            let expr = resolved_ident_for_global(&candidate, universe_args, implicit_mode, span);
            if let Ok(result) = self.elab_check_result(&expr, expected).and_then(|result| {
                self.complete_candidate_since(&snapshot, span)?;
                Ok(result)
            }) {
                successes.push((candidate, self.snapshot(), result));
            }
            self.restore(snapshot);
        }

        match successes.len() {
            0 => {
                self.restore(initial);
                Err(Diagnostic::error(
                    DiagnosticKind::AmbiguousName,
                    span,
                    "overloaded name could not be resolved",
                ))
            }
            1 => {
                let (_, snapshot, result) = successes.remove(0);
                self.restore(snapshot);
                Ok(result)
            }
            _ => {
                self.restore(initial);
                Err(Diagnostic::error(
                    DiagnosticKind::AmbiguousName,
                    span,
                    "overloaded name is ambiguous",
                ))
            }
        }
    }

    fn elab_infer_overloaded_app(
        &mut self,
        candidates: &[ElabGlobalRef],
        universe_args: Option<&[SurfaceLevel]>,
        implicit_mode: ImplicitMode,
        args: &[ResolvedExpr],
        span: Span,
    ) -> Result<InferResult> {
        let mut successes = Vec::new();
        let initial = self.snapshot();
        for candidate in sorted_global_candidates(candidates) {
            let snapshot = self.snapshot();
            let expr =
                overloaded_candidate_app_expr(&candidate, universe_args, implicit_mode, args, span);
            if let Ok(result) = self.elab_infer(&expr).and_then(|result| {
                self.complete_candidate_since(&snapshot, span)?;
                Ok(result)
            }) {
                successes.push((candidate, self.snapshot(), result));
            }
            self.restore(snapshot);
        }

        match successes.len() {
            0 => {
                self.restore(initial);
                Err(Diagnostic::error(
                    DiagnosticKind::AmbiguousName,
                    span,
                    "overloaded application could not be resolved",
                ))
            }
            1 => {
                let (_, snapshot, result) = successes.remove(0);
                self.restore(snapshot);
                Ok(result)
            }
            _ => {
                self.restore(initial);
                Err(Diagnostic::error(
                    DiagnosticKind::AmbiguousName,
                    span,
                    "overloaded application is ambiguous",
                ))
            }
        }
    }

    fn elab_check_overloaded_app(
        &mut self,
        candidates: &[ElabGlobalRef],
        universe_args: Option<&[SurfaceLevel]>,
        implicit_mode: ImplicitMode,
        args: &[ResolvedExpr],
        expected: &TypeCore,
        span: Span,
    ) -> Result<CheckResult> {
        let mut successes = Vec::new();
        let initial = self.snapshot();
        for candidate in sorted_global_candidates(candidates) {
            let snapshot = self.snapshot();
            let expr =
                overloaded_candidate_app_expr(&candidate, universe_args, implicit_mode, args, span);
            if let Ok(result) = self.elab_check_result(&expr, expected).and_then(|result| {
                self.complete_candidate_since(&snapshot, span)?;
                Ok(result)
            }) {
                successes.push((candidate, self.snapshot(), result));
            }
            self.restore(snapshot);
        }

        match successes.len() {
            0 => {
                self.restore(initial);
                Err(Diagnostic::error(
                    DiagnosticKind::AmbiguousName,
                    span,
                    "overloaded application could not be resolved",
                ))
            }
            1 => {
                let (_, snapshot, result) = successes.remove(0);
                self.restore(snapshot);
                Ok(result)
            }
            _ => {
                self.restore(initial);
                Err(Diagnostic::error(
                    DiagnosticKind::AmbiguousName,
                    span,
                    "overloaded application is ambiguous",
                ))
            }
        }
    }

    fn elab_infer_notation(
        &mut self,
        head: &crate::NotationHead,
        candidates: &[ElabGlobalRef],
        args: &[ResolvedExpr],
        span: Span,
    ) -> Result<InferResult> {
        let mut successes = Vec::new();
        let initial = self.snapshot();
        for candidate in sorted_global_candidates(candidates) {
            let snapshot = self.snapshot();
            let expr = notation_candidate_expr(&candidate, args, span);
            if let Ok(result) = self.elab_infer(&expr).and_then(|result| {
                self.complete_candidate_since(&snapshot, span)?;
                Ok(result)
            }) {
                successes.push((candidate, self.snapshot(), result));
            }
            self.restore(snapshot);
        }
        self.finish_notation_infer(head, span, initial, successes)
    }

    fn elab_check_notation(
        &mut self,
        head: &crate::NotationHead,
        candidates: &[ElabGlobalRef],
        args: &[ResolvedExpr],
        expected: &TypeCore,
        span: Span,
    ) -> Result<CheckResult> {
        let mut successes = Vec::new();
        let initial = self.snapshot();
        for candidate in sorted_global_candidates(candidates) {
            let snapshot = self.snapshot();
            let expr = notation_candidate_expr(&candidate, args, span);
            if let Ok(result) = self.elab_check_result(&expr, expected).and_then(|result| {
                self.complete_candidate_since(&snapshot, span)?;
                Ok(result)
            }) {
                successes.push((candidate, self.snapshot(), result));
            }
            self.restore(snapshot);
        }
        self.finish_notation_check(head, span, initial, successes)
    }

    fn finish_notation_infer(
        &mut self,
        head: &crate::NotationHead,
        span: Span,
        initial: ElabSnapshot,
        mut successes: Vec<(ElabGlobalRef, ElabSnapshot, InferResult)>,
    ) -> Result<InferResult> {
        match successes.len() {
            0 => {
                self.restore(initial);
                Err(Diagnostic::error(
                    DiagnosticKind::AmbiguousNotation,
                    span,
                    format!("notation `{}` could not be resolved", head.symbol),
                ))
            }
            1 => {
                let (_, snapshot, result) = successes.remove(0);
                self.restore(snapshot);
                Ok(result)
            }
            _ => {
                self.restore(initial);
                Err(Diagnostic::error(
                    DiagnosticKind::AmbiguousNotation,
                    span,
                    format!("notation `{}` is ambiguous", head.symbol),
                ))
            }
        }
    }

    fn finish_notation_check(
        &mut self,
        head: &crate::NotationHead,
        span: Span,
        initial: ElabSnapshot,
        mut successes: Vec<(ElabGlobalRef, ElabSnapshot, CheckResult)>,
    ) -> Result<CheckResult> {
        match successes.len() {
            0 => {
                self.restore(initial);
                Err(Diagnostic::error(
                    DiagnosticKind::AmbiguousNotation,
                    span,
                    format!("notation `{}` could not be resolved", head.symbol),
                ))
            }
            1 => {
                let (_, snapshot, result) = successes.remove(0);
                self.restore(snapshot);
                Ok(result)
            }
            _ => {
                self.restore(initial);
                Err(Diagnostic::error(
                    DiagnosticKind::AmbiguousNotation,
                    span,
                    format!("notation `{}` is ambiguous", head.symbol),
                ))
            }
        }
    }

    fn complete_candidate_since(&mut self, baseline: &ElabSnapshot, span: Span) -> Result<()> {
        self.solve_constraints(span)?;
        self.assign_default_universe_metas_since(baseline.universe_metas.len());
        self.solve_constraints(span)?;
        self.reject_unsolved_candidate_metas_since(baseline)
    }

    fn elab_check(&mut self, expr: &ResolvedExpr, expected: &TypeCore) -> Result<Expr> {
        Ok(self.elab_check_result(expr, expected)?.core)
    }

    fn elab_check_result(
        &mut self,
        expr: &ResolvedExpr,
        expected: &TypeCore,
    ) -> Result<CheckResult> {
        if let ResolvedExpr::Lam {
            binders,
            body,
            span,
        } = expr
        {
            if term_meta_id(&self.zonk_current_expr(&expected.core)).is_none() {
                let lambda = self.elab_check_lam(binders, body, expected, *span)?;
                return Ok(CheckResult {
                    core: lambda.core,
                    ty_metadata: expected.metadata.clone(),
                    type_value_metadata: lambda.type_value_metadata,
                });
            }
        }

        if let ResolvedExpr::Hole { name, span } = expr {
            let meta =
                self.fresh_or_reuse_user_hole(name.as_ref(), expected.core.clone(), *span)?;
            return Ok(CheckResult {
                core: term_meta_expr(meta),
                ty_metadata: expected.metadata.clone(),
                type_value_metadata: None,
            });
        }

        if let ResolvedExpr::Ident {
            resolved: ResolvedName::Overloaded(candidates),
            universe_args,
            implicit_mode,
            span,
            ..
        } = expr
        {
            return self.elab_check_overloaded_name(
                candidates,
                universe_args.as_deref(),
                *implicit_mode,
                expected,
                *span,
            );
        }

        if let Some(spine) = overloaded_app_spine(expr) {
            return self.elab_check_overloaded_app(
                &spine.candidates,
                spine.universe_args.as_deref(),
                spine.implicit_mode,
                &spine.args,
                expected,
                resolved_expr_span(expr),
            );
        }

        if let ResolvedExpr::Notation {
            head,
            candidates,
            args,
            span,
        } = expr
        {
            return self.elab_check_notation(head, candidates, args, expected, *span);
        }

        let mut inferred = self.elab_infer(expr)?;
        inferred = self.insert_implicit_args_for_expected(
            inferred,
            implicit_insertion_enabled_for_head(expr),
            expected,
            resolved_expr_span(expr),
        )?;
        self.ensure_type_eq(&inferred.ty, &expected.core, resolved_expr_span(expr))?;
        Ok(CheckResult {
            core: inferred.core,
            ty_metadata: inferred.ty_metadata,
            type_value_metadata: inferred.type_value_metadata,
        })
    }

    fn elab_infer_lam(
        &mut self,
        binders: &[ResolvedBinder],
        body: &ResolvedExpr,
        span: Span,
    ) -> Result<InferResult> {
        let saved_ctx = self.ctx.clone();
        let saved_locals = self.locals.clone();
        let core_binders = match self.elaborate_typed_binder_groups(
            binders,
            DiagnosticKind::ExpectedFunctionType,
            "lambda binder needs a type annotation in infer mode",
        ) {
            Ok(core_binders) => core_binders,
            Err(err) => {
                self.ctx = saved_ctx;
                self.locals = saved_locals;
                return Err(err);
            }
        };
        let body_result = match self.elab_infer(body) {
            Ok(body_result) => body_result,
            Err(err) => {
                self.ctx = saved_ctx;
                self.locals = saved_locals;
                return Err(err);
            }
        };
        self.ctx = saved_ctx;
        self.locals = saved_locals;

        let core = close_lam(body_result.core, &core_binders);
        let ty = close_pi(body_result.ty, &core_binders);
        if !self.expr_contains_term_meta(&core) {
            self.env
                .infer(&self.ctx, &self.delta, &core)
                .map_err(|error| diagnostic_from_kernel_error(span, error))?;
        }
        Ok(InferResult {
            core,
            ty,
            ty_metadata: close_metadata(body_result.ty_metadata, &core_binders),
            type_value_metadata: body_result.type_value_metadata,
        })
    }

    fn elab_check_lam(
        &mut self,
        binders: &[ResolvedBinder],
        body: &ResolvedExpr,
        expected: &TypeCore,
        span: Span,
    ) -> Result<CheckResult> {
        let saved_ctx = self.ctx.clone();
        let saved_locals = self.locals.clone();
        let mut expected_ty = expected.core.clone();
        let mut expected_metadata = expected.metadata.clone();
        let mut core_binders = Vec::new();
        let mut pending_pi_metas = Vec::new();
        let mut index = 0;

        while index < binders.len() {
            let group_start = index;
            let group_end = binder_group_end(binders, group_start);
            let group_source_tys = match self
                .elaborate_optional_group_source_types(&binders[group_start..group_end])
            {
                Ok(source_tys) => source_tys,
                Err(err) => {
                    self.ctx = saved_ctx;
                    self.locals = saved_locals;
                    return Err(err);
                }
            };

            for (offset, binder) in binders[group_start..group_end].iter().enumerate() {
                let source_ty = match &group_source_tys[offset] {
                    Some(source_ty) => Some(TypeCore {
                        core: weaken_group_type(&source_ty.core, offset, binder.span)?,
                        metadata: source_ty.metadata.clone(),
                        sort_level: source_ty.sort_level.clone(),
                    }),
                    None => None,
                };
                let expected_ty_zonked = self.zonk_current_expr(&expected_ty);
                let whnf = self
                    .env
                    .whnf(&self.ctx, &self.delta, &expected_ty_zonked)
                    .map_err(|error| diagnostic_from_kernel_error(span, error))?;

                let mut expected_body = None;
                let mut pending_meta = None;
                let binder_ty = match whnf {
                    Expr::Pi { ty, body, .. } => {
                        expected_body = Some((*body).clone());
                        if let Some(source_ty) = source_ty {
                            if let Err(err) = self.ensure_type_eq(&source_ty.core, &ty, binder.span)
                            {
                                self.ctx = saved_ctx;
                                self.locals = saved_locals;
                                return Err(err);
                            }
                            source_ty
                        } else {
                            TypeCore {
                                core: (*ty).clone(),
                                metadata: expected_metadata.domain_for(0, &ty),
                                sort_level: None,
                            }
                        }
                    }
                    other => {
                        let Some(id) = term_meta_id(&other) else {
                            self.ctx = saved_ctx;
                            self.locals = saved_locals;
                            return Err(Diagnostic::error(
                                DiagnosticKind::ExpectedFunctionType,
                                binder.span,
                                "lambda expects a function type",
                            ));
                        };
                        let Some(source_ty) = source_ty else {
                            self.ctx = saved_ctx;
                            self.locals = saved_locals;
                            return Err(Diagnostic::error(
                                DiagnosticKind::ExpectedFunctionType,
                                binder.span,
                                "lambda binder needs a type annotation to refine an expected type hole",
                            ));
                        };
                        pending_meta = Some((id, self.locals.clone()));
                        source_ty
                    }
                };

                let expected_binder_info = expected_metadata
                    .binder_infos
                    .first()
                    .cloned()
                    .unwrap_or_else(|| binder.binder_info.clone());
                let name = binder_name(binder);
                self.push_assumption(
                    name.clone(),
                    binder_ty.core.clone(),
                    binder_ty.metadata.clone(),
                );
                core_binders.push(CoreBinder {
                    name: name.clone(),
                    ty: binder_ty.core.clone(),
                    binder_info: expected_binder_info,
                    ty_metadata: binder_ty.metadata.clone(),
                    sort_level: binder_ty.sort_level.clone(),
                });

                if let Some((id, context)) = pending_meta {
                    let body_level = self.fresh_universe_meta(binder.span);
                    let body_meta = self.fresh_term_meta(
                        None,
                        Expr::sort(body_level),
                        TermMetaKind::SyntheticImplicit,
                        binder.span,
                    );
                    let body = term_meta_expr(body_meta);
                    pending_pi_metas.push(PendingPiMeta {
                        id,
                        context,
                        binder: name,
                        binder_ty: binder_ty.core,
                        body: body.clone(),
                        span: binder.span,
                    });
                    expected_ty = body;
                    expected_metadata = TypeMetadata::default();
                } else {
                    expected_ty = expected_body.expect("Pi branch must set expected body");
                    expected_metadata = expected_metadata.after_binder();
                }
            }

            index = group_end;
        }

        let body_result = match self.elab_check_result(
            body,
            &TypeCore {
                core: expected_ty,
                metadata: expected_metadata,
                sort_level: None,
            },
        ) {
            Ok(body_result) => body_result,
            Err(err) => {
                self.ctx = saved_ctx;
                self.locals = saved_locals;
                return Err(err);
            }
        };
        if let Err(err) = self.solve_constraints(span) {
            self.ctx = saved_ctx;
            self.locals = saved_locals;
            return Err(err);
        }
        for pending in pending_pi_metas.iter().rev() {
            let body = self.zonk_expr_in_context(&pending.body, pending.context.len() + 1);
            let pi = Expr::pi(pending.binder.clone(), pending.binder_ty.clone(), body);
            if let Err(err) = self.assign_term_meta(pending.id, &pi, &pending.context, pending.span)
            {
                self.ctx = saved_ctx;
                self.locals = saved_locals;
                return Err(err);
            }
        }
        if let Err(err) = self.solve_constraints(span) {
            self.ctx = saved_ctx;
            self.locals = saved_locals;
            return Err(err);
        }
        self.ctx = saved_ctx;
        self.locals = saved_locals;
        let core = close_lam(body_result.core, &core_binders);
        Ok(CheckResult {
            core,
            ty_metadata: expected.metadata.clone(),
            type_value_metadata: body_result.type_value_metadata,
        })
    }

    fn elaborate_typed_binder_groups(
        &mut self,
        binders: &[ResolvedBinder],
        missing_kind: DiagnosticKind,
        missing_message: &'static str,
    ) -> Result<Vec<CoreBinder>> {
        let mut core_binders = Vec::new();
        let mut index = 0;
        while index < binders.len() {
            let group_start = index;
            let group_end = binder_group_end(binders, group_start);
            let group_ty = match &binders[group_start].ty {
                Some(ty) => self.elab_type(ty)?,
                None => {
                    return Err(Diagnostic::error(
                        missing_kind.clone(),
                        binders[group_start].span,
                        missing_message,
                    ));
                }
            };
            let mut group = Vec::new();
            for binder in &binders[group_start..group_end] {
                if binder.ty.is_none() {
                    return Err(Diagnostic::error(
                        missing_kind.clone(),
                        binder.span,
                        missing_message,
                    ));
                }
                group.push((binder_name(binder), group_ty.clone(), binder.span));
            }

            for (offset, (name, binder_ty, span)) in group.into_iter().enumerate() {
                let binder_info = binders[group_start + offset].binder_info.clone();
                let binder_ty_core = weaken_group_type(&binder_ty.core, offset, span)?;
                self.push_assumption(
                    name.clone(),
                    binder_ty_core.clone(),
                    binder_ty.metadata.clone(),
                );
                core_binders.push(CoreBinder {
                    name,
                    ty: binder_ty_core,
                    binder_info,
                    ty_metadata: binder_ty.metadata,
                    sort_level: binder_ty.sort_level,
                });
            }

            index = group_end;
        }
        Ok(core_binders)
    }

    fn elaborate_optional_group_source_types(
        &mut self,
        binders: &[ResolvedBinder],
    ) -> Result<Vec<Option<TypeCore>>> {
        let group_ty = match binders.first().and_then(|binder| binder.ty.as_deref()) {
            Some(source_ty) => Some(self.elab_type(source_ty)?),
            None => None,
        };
        let mut source_tys = Vec::new();
        for binder in binders {
            source_tys.push(match &binder.ty {
                Some(_) => group_ty.clone(),
                None => None,
            });
        }
        Ok(source_tys)
    }

    fn elab_infer_pi(
        &mut self,
        binders: &[ResolvedBinder],
        body: &ResolvedExpr,
        span: Span,
    ) -> Result<InferResult> {
        let saved_ctx = self.ctx.clone();
        let saved_locals = self.locals.clone();
        let core_binders = match self.elaborate_typed_binder_groups(
            binders,
            DiagnosticKind::ExpectedSort,
            "Pi binder must have a type annotation",
        ) {
            Ok(core_binders) => core_binders,
            Err(err) => {
                self.ctx = saved_ctx;
                self.locals = saved_locals;
                return Err(err);
            }
        };
        let body_ty = match self.elab_type(body) {
            Ok(body_ty) => body_ty,
            Err(err) => {
                self.ctx = saved_ctx;
                self.locals = saved_locals;
                return Err(err);
            }
        };
        self.ctx = saved_ctx;
        self.locals = saved_locals;

        let type_value_metadata = close_metadata(body_ty.metadata.clone(), &core_binders);
        let core = close_pi(body_ty.core, &core_binders);
        let ty = Expr::sort(close_pi_sort(
            body_ty.sort_level.ok_or_else(|| {
                Diagnostic::error(
                    DiagnosticKind::ExpectedSort,
                    span,
                    "Pi body sort was not available",
                )
            })?,
            &core_binders,
            span,
        )?);
        Ok(InferResult {
            core,
            ty,
            ty_metadata: TypeMetadata::default(),
            type_value_metadata: Some(type_value_metadata),
        })
    }

    fn elab_infer_let(
        &mut self,
        name: String,
        ty: Option<&ResolvedExpr>,
        value: &ResolvedExpr,
        body: &ResolvedExpr,
        span: Span,
    ) -> Result<InferResult> {
        let saved_ctx = self.ctx.clone();
        let saved_locals = self.locals.clone();
        let (value_core, value_ty, value_ty_metadata, value_type_metadata) = if let Some(ty) = ty {
            let ty_core = self.elab_type(ty)?;
            let value_result = self.elab_check_result(value, &ty_core)?;
            let value_ty_metadata = annotation_metadata(ty, &ty_core, &value_result);
            (
                value_result.core,
                ty_core.core,
                value_ty_metadata,
                value_result.type_value_metadata,
            )
        } else {
            let value_result = self.elab_infer(value)?;
            (
                value_result.core,
                value_result.ty,
                value_result.ty_metadata,
                value_result.type_value_metadata,
            )
        };

        self.push_definition(
            name.clone(),
            value_ty.clone(),
            value_ty_metadata,
            value_core.clone(),
            value_type_metadata,
        );
        let body_result = match self.elab_infer(body) {
            Ok(body_result) => body_result,
            Err(err) => {
                self.ctx = saved_ctx;
                self.locals = saved_locals;
                return Err(err);
            }
        };
        self.ctx = saved_ctx;
        self.locals = saved_locals;

        let core = Expr::let_in(name, value_ty, value_core.clone(), body_result.core);
        let ty = instantiate(&body_result.ty, &value_core)
            .map_err(|error| diagnostic_from_kernel_error(span, error))?;
        Ok(InferResult {
            core,
            ty,
            ty_metadata: body_result.ty_metadata,
            type_value_metadata: body_result.type_value_metadata,
        })
    }

    fn elab_universe_args(
        &mut self,
        resolved: &ResolvedName,
        args: Option<&[SurfaceLevel]>,
        span: Span,
    ) -> Result<Vec<Level>> {
        let Some(args) = args else {
            if let ResolvedName::Global(global) = resolved {
                let name = global_name(global);
                if let Some(decl) = self.env.decl(&name) {
                    let universe_param_count = decl.universe_params().len();
                    return Ok((0..universe_param_count)
                        .map(|_| self.fresh_universe_meta(span))
                        .collect());
                }
            }
            return Ok(Vec::new());
        };
        args.iter().map(|level| self.elab_level(level)).collect()
    }

    fn elab_level(&self, level: &SurfaceLevel) -> Result<Level> {
        match level {
            SurfaceLevel::Nat { value, span } => {
                if *value > MAX_NUMERIC_UNIVERSE_LEVEL {
                    return Err(Diagnostic::error(
                        DiagnosticKind::UnsolvedUniverseMeta,
                        *span,
                        format!(
                            "numeric universe level `{value}` exceeds maximum `{MAX_NUMERIC_UNIVERSE_LEVEL}`"
                        ),
                    ));
                }
                Ok((0..*value).fold(Level::zero(), |level, _| Level::succ(level)))
            }
            SurfaceLevel::Param { name, span } => {
                if self.delta.iter().any(|param| param == name) {
                    Ok(Level::param(name.clone()))
                } else {
                    Err(Diagnostic::error(
                        DiagnosticKind::UnknownUniverseParam,
                        *span,
                        format!("unknown universe parameter `{name}`"),
                    ))
                }
            }
            SurfaceLevel::Succ { level, .. } => Ok(Level::succ(self.elab_level(level)?)),
            SurfaceLevel::Max { lhs, rhs, .. } => {
                Ok(Level::max(self.elab_level(lhs)?, self.elab_level(rhs)?))
            }
            SurfaceLevel::IMax { lhs, rhs, .. } => {
                Ok(Level::imax(self.elab_level(lhs)?, self.elab_level(rhs)?))
            }
        }
    }

    fn insert_implicit_args(
        &mut self,
        mut result: InferResult,
        auto_insert: bool,
        must_consume_user_arg: bool,
        span: Span,
    ) -> Result<InferResult> {
        loop {
            let Some(BinderInfo::Implicit) = result.ty_metadata.binder_infos.first() else {
                return Ok(result);
            };
            if !auto_insert || !must_consume_user_arg {
                return Ok(result);
            }

            let func_ty = self
                .env
                .whnf(&self.ctx, &self.delta, &self.zonk_current_expr(&result.ty))
                .map_err(|error| diagnostic_from_kernel_error(span, error))?;
            let Expr::Pi { ty, body, .. } = func_ty else {
                return Ok(result);
            };
            let meta =
                self.fresh_term_meta(None, (*ty).clone(), TermMetaKind::SyntheticImplicit, span);
            let meta_expr = term_meta_expr(meta);
            result.core = Expr::app(result.core, meta_expr.clone());
            result.ty = instantiate(&body, &meta_expr)
                .map_err(|error| diagnostic_from_kernel_error(span, error))?;
            result.ty_metadata = result.ty_metadata.after_binder();
            result.type_value_metadata = self.type_value_metadata_for_core(&result.core);
        }
    }

    fn insert_implicit_args_for_expected(
        &mut self,
        mut result: InferResult,
        auto_insert: bool,
        expected: &TypeCore,
        span: Span,
    ) -> Result<InferResult> {
        loop {
            let Some(BinderInfo::Implicit) = result.ty_metadata.binder_infos.first() else {
                return Ok(result);
            };
            if !auto_insert {
                return Ok(result);
            }
            if self.type_eq_without_implicit_insertion(&result.ty, &expected.core, span)? {
                return Ok(result);
            }
            if !expected_needs_implicit_instantiation(expected) {
                return Ok(result);
            }

            let func_ty = self
                .env
                .whnf(&self.ctx, &self.delta, &self.zonk_current_expr(&result.ty))
                .map_err(|error| diagnostic_from_kernel_error(span, error))?;
            let Expr::Pi { ty, body, .. } = func_ty else {
                return Ok(result);
            };
            let meta =
                self.fresh_term_meta(None, (*ty).clone(), TermMetaKind::SyntheticImplicit, span);
            let meta_expr = term_meta_expr(meta);
            result.core = Expr::app(result.core, meta_expr.clone());
            result.ty = instantiate(&body, &meta_expr)
                .map_err(|error| diagnostic_from_kernel_error(span, error))?;
            result.ty_metadata = result.ty_metadata.after_binder();
            result.type_value_metadata = self.type_value_metadata_for_core(&result.core);
        }
    }

    fn type_eq_without_implicit_insertion(
        &mut self,
        actual: &Expr,
        expected: &Expr,
        span: Span,
    ) -> Result<bool> {
        let actual = self.zonk_current_expr(actual);
        let expected = self.zonk_current_expr(expected);

        let saved_term_metas = self.term_metas.clone();
        let saved_universe_metas = self.universe_metas.clone();
        let saved_constraints = self.constraints.clone();
        let saved_delta = self.delta.clone();

        self.constraints.push(Constraint::TypeEq {
            lhs: actual.clone(),
            rhs: expected.clone(),
            context: self.locals.clone(),
            span,
        });

        let solved = match self.solve_constraints(span) {
            Ok(()) => {
                let actual = self.zonk_current_expr(&actual);
                let expected = self.zonk_current_expr(&expected);
                !self.expr_contains_term_meta(&actual)
                    && !self.expr_contains_term_meta(&expected)
                    && !self.level_contains_universe_meta_in_expr(&actual)
                    && !self.level_contains_universe_meta_in_expr(&expected)
                    && self
                        .env
                        .is_defeq(&self.ctx, &self.delta, &actual, &expected)
                        .map_err(|error| diagnostic_from_kernel_error(span, error))?
            }
            Err(_) => false,
        };

        if solved {
            Ok(true)
        } else {
            self.term_metas = saved_term_metas;
            self.universe_metas = saved_universe_metas;
            self.constraints = saved_constraints;
            self.delta = saved_delta;
            Ok(false)
        }
    }

    fn ensure_type_eq(&mut self, actual: &Expr, expected: &Expr, span: Span) -> Result<()> {
        self.constraints.push(Constraint::TypeEq {
            lhs: actual.clone(),
            rhs: expected.clone(),
            context: self.locals.clone(),
            span,
        });
        self.solve_constraints(span)?;

        let actual = self.zonk_current_expr(actual);
        let expected = self.zonk_current_expr(expected);
        if self.expr_contains_term_meta(&actual)
            || self.expr_contains_term_meta(&expected)
            || self.level_contains_universe_meta_in_expr(&actual)
            || self.level_contains_universe_meta_in_expr(&expected)
        {
            return Ok(());
        }

        if self
            .env
            .is_defeq(&self.ctx, &self.delta, &actual, &expected)
            .map_err(|error| diagnostic_from_kernel_error(span, error))?
        {
            Ok(())
        } else {
            Err(Diagnostic::error(
                DiagnosticKind::TypeMismatch,
                span,
                format!("expected `{expected:?}`, found `{actual:?}`"),
            ))
        }
    }

    fn finish_expr(&mut self, expr: Expr, span: Span) -> Result<Expr> {
        self.solve_constraints(span)?;
        self.assign_default_universe_metas();
        self.solve_constraints(span)?;
        self.reject_unsolved_metas()?;
        let expr = self.zonk_closed_expr(&expr);
        if self.expr_contains_term_meta(&expr) {
            return Err(Diagnostic::error(
                DiagnosticKind::UnsolvedHole,
                span,
                "term metavariable was not solved",
            ));
        }
        if self.level_contains_universe_meta_in_expr(&expr) {
            return Err(Diagnostic::error(
                DiagnosticKind::UnsolvedUniverseMeta,
                span,
                "universe metavariable was not solved",
            ));
        }
        Ok(expr)
    }

    fn whnf_closed_expr(&self, expr: &Expr, span: Span) -> Result<Expr> {
        self.env
            .whnf(&Ctx::new(), &self.delta, expr)
            .map_err(|error| diagnostic_from_kernel_error(span, error))
    }

    fn solve_constraints(&mut self, fallback_span: Span) -> Result<()> {
        for _ in 0..SOLVER_FUEL {
            if self.constraints.is_empty() {
                return Ok(());
            }

            let mut progress = false;
            let constraints = std::mem::take(&mut self.constraints);
            for constraint in constraints {
                match self.solve_constraint(&constraint)? {
                    SolveStatus::Solved => progress = true,
                    SolveStatus::Stuck => self.constraints.push(constraint),
                }
            }

            if !progress {
                return Ok(());
            }
        }

        Err(Diagnostic::error(
            DiagnosticKind::IncompleteDependency,
            fallback_span,
            "metavariable constraint solving reached its resource limit",
        ))
    }

    fn solve_constraint(&mut self, constraint: &Constraint) -> Result<SolveStatus> {
        match constraint {
            Constraint::TypeEq {
                lhs,
                rhs,
                context,
                span,
            } => self.solve_expr_eq(lhs, rhs, None, context, *span),
            Constraint::TermEq {
                ty,
                lhs,
                rhs,
                context,
                span,
            } => self.solve_expr_eq(lhs, rhs, Some(ty), context, *span),
            Constraint::LevelEq { lhs, rhs, span } => self.solve_level_eq(lhs, rhs, *span),
            Constraint::LevelLe { lhs, rhs, span } => self.solve_level_le(lhs, rhs, *span),
        }
    }

    fn solve_expr_eq(
        &mut self,
        lhs: &Expr,
        rhs: &Expr,
        ty: Option<&Expr>,
        context: &[LocalCtxEntry],
        span: Span,
    ) -> Result<SolveStatus> {
        let ctx = self.ctx_for_snapshot(context);
        let lhs = self
            .env
            .whnf(
                &ctx,
                &self.delta,
                &self.zonk_expr_in_context(lhs, context.len()),
            )
            .map_err(|error| diagnostic_from_kernel_error(span, error))?;
        let rhs = self
            .env
            .whnf(
                &ctx,
                &self.delta,
                &self.zonk_expr_in_context(rhs, context.len()),
            )
            .map_err(|error| diagnostic_from_kernel_error(span, error))?;
        if lhs == rhs {
            return Ok(SolveStatus::Solved);
        }

        if let Some(id) = term_meta_id(&lhs) {
            if self.assign_term_meta(id, &rhs, context, span)? {
                return Ok(SolveStatus::Solved);
            }
        }
        if let Some(id) = term_meta_id(&rhs) {
            if self.assign_term_meta(id, &lhs, context, span)? {
                return Ok(SolveStatus::Solved);
            }
        }

        let lhs_has_meta =
            self.expr_contains_term_meta(&lhs) || self.level_contains_universe_meta_in_expr(&lhs);
        let rhs_has_meta =
            self.expr_contains_term_meta(&rhs) || self.level_contains_universe_meta_in_expr(&rhs);
        if !lhs_has_meta && !rhs_has_meta {
            return if self
                .env
                .is_defeq(&ctx, &self.delta, &lhs, &rhs)
                .map_err(|error| diagnostic_from_kernel_error(span, error))?
            {
                Ok(SolveStatus::Solved)
            } else {
                Err(Diagnostic::error(
                    DiagnosticKind::TypeMismatch,
                    span,
                    format!("expected `{rhs:?}`, found `{lhs:?}`"),
                ))
            };
        }

        match (&lhs, &rhs) {
            (Expr::Sort(lhs), Expr::Sort(rhs)) => {
                self.constraints.push(Constraint::LevelLe {
                    lhs: lhs.clone(),
                    rhs: rhs.clone(),
                    span,
                });
                self.constraints.push(Constraint::LevelLe {
                    lhs: rhs.clone(),
                    rhs: lhs.clone(),
                    span,
                });
                Ok(SolveStatus::Solved)
            }
            (
                Expr::Const {
                    name: lhs_name,
                    levels: lhs_levels,
                },
                Expr::Const {
                    name: rhs_name,
                    levels: rhs_levels,
                },
            ) if lhs_name == rhs_name && lhs_levels.len() == rhs_levels.len() => {
                for (lhs, rhs) in lhs_levels.iter().zip(rhs_levels) {
                    self.constraints.push(Constraint::LevelEq {
                        lhs: lhs.clone(),
                        rhs: rhs.clone(),
                        span,
                    });
                }
                Ok(SolveStatus::Solved)
            }
            (Expr::App(lhs_f, lhs_a), Expr::App(rhs_f, rhs_a)) => {
                self.constraints.push(Constraint::TermEq {
                    ty: ty.cloned().unwrap_or_else(|| Expr::sort(Level::zero())),
                    lhs: (**lhs_f).clone(),
                    rhs: (**rhs_f).clone(),
                    context: context.to_vec(),
                    span,
                });
                self.constraints.push(Constraint::TermEq {
                    ty: ty.cloned().unwrap_or_else(|| Expr::sort(Level::zero())),
                    lhs: (**lhs_a).clone(),
                    rhs: (**rhs_a).clone(),
                    context: context.to_vec(),
                    span,
                });
                Ok(SolveStatus::Solved)
            }
            (
                Expr::Pi {
                    ty: lhs_ty,
                    body: lhs_body,
                    ..
                },
                Expr::Pi {
                    ty: rhs_ty,
                    body: rhs_body,
                    ..
                },
            )
            | (
                Expr::Lam {
                    ty: lhs_ty,
                    body: lhs_body,
                    ..
                },
                Expr::Lam {
                    ty: rhs_ty,
                    body: rhs_body,
                    ..
                },
            ) => {
                let body_context = extend_snapshot_with_assumption(context, (**lhs_ty).clone());
                self.constraints.push(Constraint::TypeEq {
                    lhs: (**lhs_ty).clone(),
                    rhs: (**rhs_ty).clone(),
                    context: context.to_vec(),
                    span,
                });
                self.constraints.push(Constraint::TermEq {
                    ty: (**lhs_ty).clone(),
                    lhs: (**lhs_body).clone(),
                    rhs: (**rhs_body).clone(),
                    context: body_context,
                    span,
                });
                Ok(SolveStatus::Solved)
            }
            _ => {
                debug_assert!(lhs_has_meta || rhs_has_meta);
                Ok(SolveStatus::Stuck)
            }
        }
    }

    fn solve_level_eq(&mut self, lhs: &Level, rhs: &Level, span: Span) -> Result<SolveStatus> {
        let lhs = self.zonk_level(lhs);
        let rhs = self.zonk_level(rhs);
        if lhs == rhs {
            return Ok(SolveStatus::Solved);
        }
        if let Some(id) = universe_meta_id(&lhs) {
            self.assign_universe_meta(id, rhs, span)?;
            return Ok(SolveStatus::Solved);
        }
        if let Some(id) = universe_meta_id(&rhs) {
            self.assign_universe_meta(id, lhs, span)?;
            return Ok(SolveStatus::Solved);
        }
        if let (Level::Succ(lhs), Level::Succ(rhs)) = (&lhs, &rhs) {
            self.constraints.push(Constraint::LevelEq {
                lhs: (**lhs).clone(),
                rhs: (**rhs).clone(),
                span,
            });
            return Ok(SolveStatus::Solved);
        }
        if let Some((id, value)) = imax_succ_self_solution(&lhs, &rhs) {
            self.assign_universe_meta(id, value, span)?;
            return Ok(SolveStatus::Solved);
        }
        if let Some((id, value)) = imax_succ_self_solution(&rhs, &lhs) {
            self.assign_universe_meta(id, value, span)?;
            return Ok(SolveStatus::Solved);
        }
        if level_contains_universe_meta(&lhs) || level_contains_universe_meta(&rhs) {
            Ok(SolveStatus::Stuck)
        } else {
            Err(Diagnostic::error(
                DiagnosticKind::UnsolvedUniverseMeta,
                span,
                format!("could not solve universe equality `{lhs:?} = {rhs:?}`"),
            ))
        }
    }

    fn solve_level_le(&mut self, lhs: &Level, rhs: &Level, span: Span) -> Result<SolveStatus> {
        self.solve_level_eq(lhs, rhs, span)
    }

    fn assign_term_meta(
        &mut self,
        id: TermMetaId,
        value: &Expr,
        context: &[LocalCtxEntry],
        span: Span,
    ) -> Result<bool> {
        let Some(meta) = self.term_metas.get(id.0) else {
            return Ok(false);
        };
        if meta.assignment.is_some() {
            return Ok(false);
        }
        let meta_context = meta.context.clone();
        let meta_ty_raw = meta.ty.clone();
        if !self.local_context_prefix_eq(&meta_context, context) {
            return Ok(false);
        }
        if expr_occurs_term_meta(value, id) {
            return Err(Diagnostic::error(
                DiagnosticKind::OccursCheckFailed,
                span,
                "metavariable assignment would be recursive",
            ));
        }

        let value = self.zonk_expr_in_context(value, context.len());
        if self.expr_contains_term_meta(&value) {
            return Ok(false);
        }
        let Some(value) =
            rebase_expr_to_prefix_context(&value, context.len(), meta_context.len(), span)?
        else {
            return Ok(false);
        };
        let meta_ty = self.zonk_expr_in_context(&meta_ty_raw, meta_context.len());
        let value_ty = self
            .env
            .infer(&self.ctx_for_snapshot(&meta_context), &self.delta, &value)
            .map_err(|error| diagnostic_from_kernel_error(span, error))?;
        self.constraints.push(Constraint::TypeEq {
            lhs: value_ty,
            rhs: meta_ty,
            context: meta_context,
            span,
        });
        self.term_metas[id.0].assignment = Some(value);
        Ok(true)
    }

    fn assign_universe_meta(&mut self, id: UniverseMetaId, value: Level, span: Span) -> Result<()> {
        if level_occurs_universe_meta(&value, id) {
            return Err(Diagnostic::error(
                DiagnosticKind::OccursCheckFailed,
                span,
                "universe metavariable assignment would be recursive",
            ));
        }
        self.universe_metas[id.0].assignment = Some(value);
        Ok(())
    }

    fn reject_unsolved_metas(&self) -> Result<()> {
        for meta in &self.term_metas {
            if meta.assignment.is_none() && meta.kind == TermMetaKind::UserHole {
                return Err(Diagnostic::error(
                    DiagnosticKind::UnsolvedHole,
                    meta.span,
                    match &meta.name {
                        Some(name) => format!("unsolved hole `?{name}`"),
                        None => "unsolved hole".to_owned(),
                    },
                ));
            }
        }
        for meta in &self.term_metas {
            if meta.assignment.is_none() && meta.kind == TermMetaKind::SyntheticImplicit {
                return Err(Diagnostic::error(
                    DiagnosticKind::UnsolvedImplicit,
                    meta.span,
                    "could not infer implicit argument",
                ));
            }
        }
        for meta in &self.universe_metas {
            if meta.assignment.is_none() {
                return Err(Diagnostic::error(
                    DiagnosticKind::UnsolvedUniverseMeta,
                    meta.span,
                    "could not infer universe argument",
                ));
            }
        }
        Ok(())
    }

    fn reject_unsolved_candidate_metas_since(&self, baseline: &ElabSnapshot) -> Result<()> {
        for meta in self.term_metas.iter().skip(baseline.term_metas.len()) {
            if meta.assignment.is_none() && meta.kind == TermMetaKind::SyntheticImplicit {
                return Err(Diagnostic::error(
                    DiagnosticKind::UnsolvedImplicit,
                    meta.span,
                    "could not infer implicit argument",
                ));
            }
        }
        for meta in self
            .universe_metas
            .iter()
            .skip(baseline.universe_metas.len())
        {
            if meta.assignment.is_none() {
                return Err(Diagnostic::error(
                    DiagnosticKind::UnsolvedUniverseMeta,
                    meta.span,
                    "could not infer universe argument",
                ));
            }
        }
        Ok(())
    }

    fn fresh_term_meta(
        &mut self,
        name: Option<String>,
        ty: Expr,
        kind: TermMetaKind,
        span: Span,
    ) -> TermMetaId {
        let id = TermMetaId(self.term_metas.len());
        self.term_metas.push(TermMeta {
            name,
            context: self.locals.clone(),
            ty,
            kind,
            span,
            assignment: None,
        });
        id
    }

    fn fresh_or_reuse_user_hole(
        &mut self,
        name: Option<&crate::SurfaceName>,
        ty: Expr,
        span: Span,
    ) -> Result<TermMetaId> {
        let Some(name) = name else {
            return Ok(self.fresh_term_meta(None, ty, TermMetaKind::UserHole, span));
        };
        let key = name.parts.join(".");
        if let Some(id) = self.named_holes.get(&key).copied() {
            let meta_context = self.term_metas[id.0].context.clone();
            let meta_ty = self.term_metas[id.0].ty.clone();
            if !self.local_context_eq(&meta_context, &self.locals) {
                return Err(Diagnostic::error(
                    DiagnosticKind::NamedHoleContextMismatch,
                    span,
                    format!("named hole `?{key}` was reused in a different local context"),
                ));
            }
            self.constraints.push(Constraint::TypeEq {
                lhs: meta_ty,
                rhs: ty,
                context: self.locals.clone(),
                span,
            });
            return Ok(id);
        }

        let id = self.fresh_term_meta(Some(key.clone()), ty, TermMetaKind::UserHole, span);
        self.named_holes.insert(key, id);
        Ok(id)
    }

    fn fresh_universe_meta(&mut self, span: Span) -> Level {
        self.fresh_universe_meta_with_default(span, None)
    }

    fn fresh_type_hole_sort_level(&mut self, span: Span) -> Level {
        self.fresh_universe_meta_with_default(span, Some(Level::succ(Level::zero())))
    }

    fn fresh_universe_meta_with_default(&mut self, span: Span, default: Option<Level>) -> Level {
        let id = UniverseMetaId(self.universe_metas.len());
        let name = format!("{UNIVERSE_META_PREFIX}{}", id.0);
        self.delta.push(name.clone());
        self.universe_metas.push(UniverseMeta {
            span,
            default,
            assignment: None,
        });
        Level::param(name)
    }

    fn assign_default_universe_metas(&mut self) {
        for meta in &mut self.universe_metas {
            if meta.assignment.is_none() {
                if let Some(default) = meta.default.clone() {
                    meta.assignment = Some(default);
                }
            }
        }
    }

    fn assign_default_universe_metas_since(&mut self, start: usize) {
        for meta in self.universe_metas.iter_mut().skip(start) {
            if meta.assignment.is_none() {
                if let Some(default) = meta.default.clone() {
                    meta.assignment = Some(default);
                }
            }
        }
    }

    fn push_assumption(&mut self, name: String, ty: Expr, ty_metadata: TypeMetadata) {
        self.ctx.push_assumption(name, ty.clone());
        self.locals.push(LocalCtxEntry {
            ty,
            ty_metadata,
            value_type_metadata: None,
            value: None,
        });
    }

    fn push_definition(
        &mut self,
        name: String,
        ty: Expr,
        ty_metadata: TypeMetadata,
        value: Expr,
        value_type_metadata: Option<TypeMetadata>,
    ) {
        self.ctx.push_definition(name, ty.clone(), value.clone());
        self.locals.push(LocalCtxEntry {
            ty,
            ty_metadata,
            value_type_metadata,
            value: Some(value),
        });
    }

    fn local_info(&self, index: u32, span: Span) -> Result<&LocalCtxEntry> {
        let index = index as usize;
        if index >= self.locals.len() {
            return Err(Diagnostic::error(
                DiagnosticKind::KernelRejected,
                span,
                format!("invalid local de Bruijn index `{index}`"),
            ));
        }
        Ok(&self.locals[self.locals.len() - 1 - index])
    }

    fn ctx_for_snapshot(&self, snapshot: &[LocalCtxEntry]) -> Ctx {
        let mut ctx = Ctx::new();
        for (index, local) in snapshot.iter().enumerate() {
            let name = format!("_{}", index);
            let ty = self.zonk_expr_in_context(&local.ty, index);
            if let Some(value) = &local.value {
                ctx.push_definition(name, ty, self.zonk_expr_in_context(value, index));
            } else {
                ctx.push_assumption(name, ty);
            }
        }
        ctx
    }

    fn zonk_current_expr(&self, expr: &Expr) -> Expr {
        self.zonk_expr_in_context(expr, self.locals.len())
    }

    fn zonk_closed_expr(&self, expr: &Expr) -> Expr {
        self.zonk_expr_in_context(expr, 0)
    }

    fn zonk_core_binders_from(
        &self,
        binders: &[CoreBinder],
        start_context_len: usize,
    ) -> Vec<CoreBinder> {
        binders
            .iter()
            .enumerate()
            .map(|(offset, binder)| CoreBinder {
                name: binder.name.clone(),
                ty: self.zonk_expr_in_context(&binder.ty, start_context_len + offset),
                binder_info: binder.binder_info.clone(),
                ty_metadata: binder.ty_metadata.clone(),
                sort_level: binder
                    .sort_level
                    .as_ref()
                    .map(|level| self.zonk_level(level)),
            })
            .collect()
    }

    fn zonk_expr_in_context(&self, expr: &Expr, context_len: usize) -> Expr {
        match expr {
            Expr::Sort(level) => Expr::sort(self.zonk_level(level)),
            Expr::BVar(index) => Expr::bvar(*index),
            Expr::Const { name, levels } => {
                if let Some(id) = term_meta_id_from_name(name) {
                    if let Some(meta) = self.term_metas.get(id.0) {
                        if let Some(assignment) = meta.assignment.as_ref() {
                            let assignment =
                                self.zonk_expr_in_context(assignment, meta.context.len());
                            if let Some(extra_depth) = context_len.checked_sub(meta.context.len()) {
                                return shift(&assignment, extra_depth as i32, 0)
                                    .expect("positive metavariable assignment shift must succeed");
                            }
                            return assignment;
                        }
                    }
                }
                Expr::konst(
                    name.clone(),
                    levels.iter().map(|level| self.zonk_level(level)).collect(),
                )
            }
            Expr::App(fun, arg) => Expr::app(
                self.zonk_expr_in_context(fun, context_len),
                self.zonk_expr_in_context(arg, context_len),
            ),
            Expr::Lam { binder, ty, body } => Expr::lam(
                binder.clone(),
                self.zonk_expr_in_context(ty, context_len),
                self.zonk_expr_in_context(body, context_len + 1),
            ),
            Expr::Pi { binder, ty, body } => Expr::pi(
                binder.clone(),
                self.zonk_expr_in_context(ty, context_len),
                self.zonk_expr_in_context(body, context_len + 1),
            ),
            Expr::Let {
                binder,
                ty,
                value,
                body,
            } => Expr::let_in(
                binder.clone(),
                self.zonk_expr_in_context(ty, context_len),
                self.zonk_expr_in_context(value, context_len),
                self.zonk_expr_in_context(body, context_len + 1),
            ),
        }
    }

    fn zonk_level(&self, level: &Level) -> Level {
        match level {
            Level::Zero => Level::zero(),
            Level::Succ(level) => Level::succ(self.zonk_level(level)),
            Level::Max(lhs, rhs) => Level::max(self.zonk_level(lhs), self.zonk_level(rhs)),
            Level::IMax(lhs, rhs) => Level::imax(self.zonk_level(lhs), self.zonk_level(rhs)),
            Level::Param(name) => {
                if let Some(id) = universe_meta_id_from_name(name) {
                    if let Some(assignment) = self
                        .universe_metas
                        .get(id.0)
                        .and_then(|meta| meta.assignment.as_ref())
                    {
                        return self.zonk_level(assignment);
                    }
                }
                Level::param(name.clone())
            }
        }
    }

    fn expr_contains_term_meta(&self, expr: &Expr) -> bool {
        expr_contains_term_meta(expr)
    }

    fn level_contains_universe_meta_in_expr(&self, expr: &Expr) -> bool {
        level_contains_universe_meta_in_expr(expr)
    }

    fn type_value_metadata_for_core(&self, core: &Expr) -> Option<TypeMetadata> {
        match app_head(core) {
            Expr::Const { name, .. } => self.signatures.type_value_metadata_for(name),
            Expr::BVar(index) => {
                let index = *index as usize;
                self.locals
                    .get(self.locals.len().checked_sub(index + 1)?)
                    .and_then(|local| local.value_type_metadata.clone())
            }
            _ => None,
        }
    }

    fn local_context_eq(&self, lhs: &[LocalCtxEntry], rhs: &[LocalCtxEntry]) -> bool {
        lhs.len() == rhs.len() && self.local_context_prefix_eq(lhs, rhs)
    }

    fn local_context_prefix_eq(&self, prefix: &[LocalCtxEntry], full: &[LocalCtxEntry]) -> bool {
        prefix.len() <= full.len()
            && prefix
                .iter()
                .zip(full)
                .enumerate()
                .all(|(index, (prefix, full))| {
                    self.zonk_expr_in_context(&prefix.ty, index)
                        == self.zonk_expr_in_context(&full.ty, index)
                        && match (&prefix.value, &full.value) {
                            (Some(prefix), Some(full)) => {
                                self.zonk_expr_in_context(prefix, index)
                                    == self.zonk_expr_in_context(full, index)
                            }
                            (None, None) => true,
                            _ => false,
                        }
                })
    }
}

#[derive(Clone, Debug)]
struct CoreBinder {
    name: String,
    ty: Expr,
    binder_info: BinderInfo,
    ty_metadata: TypeMetadata,
    sort_level: Option<Level>,
}

#[derive(Clone, Debug)]
struct PendingPiMeta {
    id: TermMetaId,
    context: Vec<LocalCtxEntry>,
    binder: String,
    binder_ty: Expr,
    body: Expr,
    span: Span,
}

fn term_meta_expr(id: TermMetaId) -> Expr {
    Expr::konst(format!("{TERM_META_PREFIX}{}", id.0), Vec::new())
}

fn term_meta_id(expr: &Expr) -> Option<TermMetaId> {
    match expr {
        Expr::Const { name, levels } if levels.is_empty() => term_meta_id_from_name(name),
        _ => None,
    }
}

fn term_meta_id_from_name(name: &str) -> Option<TermMetaId> {
    name.strip_prefix(TERM_META_PREFIX)?
        .parse()
        .ok()
        .map(TermMetaId)
}

fn universe_meta_id(level: &Level) -> Option<UniverseMetaId> {
    match level {
        Level::Param(name) => universe_meta_id_from_name(name),
        _ => None,
    }
}

fn universe_meta_id_from_name(name: &str) -> Option<UniverseMetaId> {
    name.strip_prefix(UNIVERSE_META_PREFIX)?
        .parse()
        .ok()
        .map(UniverseMetaId)
}

fn imax_succ_self_solution(imax: &Level, target: &Level) -> Option<(UniverseMetaId, Level)> {
    let Level::IMax(lhs, rhs) = imax else {
        return None;
    };
    let id = universe_meta_id(rhs)?;
    let Level::Succ(inner) = lhs.as_ref() else {
        return None;
    };
    if universe_meta_id(inner) != Some(id) {
        return None;
    }
    match target {
        Level::Zero => Some((id, Level::zero())),
        Level::Succ(inner) => Some((id, (**inner).clone())),
        _ => None,
    }
}

fn expr_contains_term_meta(expr: &Expr) -> bool {
    match expr {
        Expr::Sort(_) | Expr::BVar(_) => false,
        Expr::Const { name, .. } => term_meta_id_from_name(name).is_some(),
        Expr::App(fun, arg) => expr_contains_term_meta(fun) || expr_contains_term_meta(arg),
        Expr::Lam { ty, body, .. } | Expr::Pi { ty, body, .. } => {
            expr_contains_term_meta(ty) || expr_contains_term_meta(body)
        }
        Expr::Let {
            ty, value, body, ..
        } => {
            expr_contains_term_meta(ty)
                || expr_contains_term_meta(value)
                || expr_contains_term_meta(body)
        }
    }
}

fn expr_occurs_term_meta(expr: &Expr, id: TermMetaId) -> bool {
    match expr {
        Expr::Sort(_) | Expr::BVar(_) => false,
        Expr::Const { name, .. } => term_meta_id_from_name(name) == Some(id),
        Expr::App(fun, arg) => expr_occurs_term_meta(fun, id) || expr_occurs_term_meta(arg, id),
        Expr::Lam { ty, body, .. } | Expr::Pi { ty, body, .. } => {
            expr_occurs_term_meta(ty, id) || expr_occurs_term_meta(body, id)
        }
        Expr::Let {
            ty, value, body, ..
        } => {
            expr_occurs_term_meta(ty, id)
                || expr_occurs_term_meta(value, id)
                || expr_occurs_term_meta(body, id)
        }
    }
}

fn level_contains_universe_meta(level: &Level) -> bool {
    match level {
        Level::Zero => false,
        Level::Succ(level) => level_contains_universe_meta(level),
        Level::Max(lhs, rhs) | Level::IMax(lhs, rhs) => {
            level_contains_universe_meta(lhs) || level_contains_universe_meta(rhs)
        }
        Level::Param(name) => universe_meta_id_from_name(name).is_some(),
    }
}

fn level_occurs_universe_meta(level: &Level, id: UniverseMetaId) -> bool {
    match level {
        Level::Zero => false,
        Level::Succ(level) => level_occurs_universe_meta(level, id),
        Level::Max(lhs, rhs) | Level::IMax(lhs, rhs) => {
            level_occurs_universe_meta(lhs, id) || level_occurs_universe_meta(rhs, id)
        }
        Level::Param(name) => universe_meta_id_from_name(name) == Some(id),
    }
}

fn level_contains_universe_meta_in_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Sort(level) => level_contains_universe_meta(level),
        Expr::BVar(_) => false,
        Expr::Const { levels, .. } => levels.iter().any(level_contains_universe_meta),
        Expr::App(fun, arg) => {
            level_contains_universe_meta_in_expr(fun) || level_contains_universe_meta_in_expr(arg)
        }
        Expr::Lam { ty, body, .. } | Expr::Pi { ty, body, .. } => {
            level_contains_universe_meta_in_expr(ty) || level_contains_universe_meta_in_expr(body)
        }
        Expr::Let {
            ty, value, body, ..
        } => {
            level_contains_universe_meta_in_expr(ty)
                || level_contains_universe_meta_in_expr(value)
                || level_contains_universe_meta_in_expr(body)
        }
    }
}

fn app_head(mut expr: &Expr) -> &Expr {
    while let Expr::App(fun, _) = expr {
        expr = fun;
    }
    expr
}

fn pi_domains(mut expr: &Expr) -> Vec<&Expr> {
    let mut domains = Vec::new();
    while let Expr::Pi { ty, body, .. } = expr {
        domains.push(ty.as_ref());
        expr = body;
    }
    domains
}

fn source_type_metadata(expr: &ResolvedExpr) -> TypeMetadata {
    match expr {
        ResolvedExpr::Pi { binders, body, .. } => {
            let body_metadata = source_type_metadata(body);
            let binder_infos = binders
                .iter()
                .map(|binder| binder.binder_info.clone())
                .chain(body_metadata.binder_infos)
                .collect();
            let domain_infos = binders
                .iter()
                .map(|binder| {
                    binder
                        .ty
                        .as_deref()
                        .map(source_type_metadata)
                        .unwrap_or_default()
                })
                .chain(body_metadata.domain_infos)
                .collect();
            TypeMetadata {
                binder_infos,
                domain_infos,
            }
        }
        ResolvedExpr::Annot { expr, .. } => source_type_metadata(expr),
        _ => TypeMetadata::default(),
    }
}

fn close_metadata(body_metadata: TypeMetadata, binders: &[CoreBinder]) -> TypeMetadata {
    TypeMetadata {
        binder_infos: binders
            .iter()
            .map(|binder| binder.binder_info.clone())
            .chain(body_metadata.binder_infos)
            .collect(),
        domain_infos: binders
            .iter()
            .map(|binder| binder.ty_metadata.clone())
            .chain(body_metadata.domain_infos)
            .collect(),
    }
}

fn annotation_metadata(
    annotation_ty: &ResolvedExpr,
    expected: &TypeCore,
    checked: &CheckResult,
) -> TypeMetadata {
    if matches!(annotation_ty, ResolvedExpr::Hole { .. })
        && expected.metadata.binder_infos.is_empty()
    {
        checked.ty_metadata.clone()
    } else {
        expected.metadata.clone()
    }
}

fn implicit_insertion_enabled_for_head(expr: &ResolvedExpr) -> bool {
    match expr {
        ResolvedExpr::Ident { implicit_mode, .. } => *implicit_mode == ImplicitMode::Insert,
        ResolvedExpr::App { func, .. } => implicit_insertion_enabled_for_head(func),
        ResolvedExpr::Annot { expr, .. } => implicit_insertion_enabled_for_head(expr),
        _ => true,
    }
}

fn expected_needs_implicit_instantiation(expected: &TypeCore) -> bool {
    if term_meta_id(&expected.core).is_some() {
        return false;
    }
    expected.metadata.binder_infos.first() != Some(&BinderInfo::Implicit)
}

fn extend_snapshot_with_assumption(snapshot: &[LocalCtxEntry], ty: Expr) -> Vec<LocalCtxEntry> {
    let mut extended = snapshot.to_vec();
    extended.push(LocalCtxEntry {
        ty,
        ty_metadata: TypeMetadata::default(),
        value_type_metadata: None,
        value: None,
    });
    extended
}

fn rebase_expr_to_prefix_context(
    expr: &Expr,
    source_len: usize,
    target_len: usize,
    span: Span,
) -> Result<Option<Expr>> {
    if target_len > source_len {
        return Ok(None);
    }
    let dropped = u32::try_from(source_len - target_len).map_err(|_| {
        Diagnostic::error(
            DiagnosticKind::KernelRejected,
            span,
            "local context is too large to rebase metavariable assignment",
        )
    })?;
    rebase_expr_dropping_locals(expr, dropped, 0, span)
}

fn rebase_expr_dropping_locals(
    expr: &Expr,
    dropped: u32,
    cutoff: u32,
    span: Span,
) -> Result<Option<Expr>> {
    match expr {
        Expr::Sort(level) => Ok(Some(Expr::sort(level.clone()))),
        Expr::BVar(index) if *index < cutoff => Ok(Some(Expr::bvar(*index))),
        Expr::BVar(index) => {
            let dropped_end = cutoff.checked_add(dropped).ok_or_else(|| {
                Diagnostic::error(
                    DiagnosticKind::KernelRejected,
                    span,
                    "local context is too large to rebase metavariable assignment",
                )
            })?;
            if *index < dropped_end {
                return Ok(None);
            }
            Ok(Some(Expr::bvar(index - dropped)))
        }
        Expr::Const { name, levels } => Ok(Some(Expr::konst(name.clone(), levels.clone()))),
        Expr::App(fun, arg) => {
            let Some(fun) = rebase_expr_dropping_locals(fun, dropped, cutoff, span)? else {
                return Ok(None);
            };
            let Some(arg) = rebase_expr_dropping_locals(arg, dropped, cutoff, span)? else {
                return Ok(None);
            };
            Ok(Some(Expr::app(fun, arg)))
        }
        Expr::Lam { binder, ty, body } => {
            let Some(ty) = rebase_expr_dropping_locals(ty, dropped, cutoff, span)? else {
                return Ok(None);
            };
            let Some(body) = rebase_expr_dropping_locals(body, dropped, cutoff + 1, span)? else {
                return Ok(None);
            };
            Ok(Some(Expr::lam(binder.clone(), ty, body)))
        }
        Expr::Pi { binder, ty, body } => {
            let Some(ty) = rebase_expr_dropping_locals(ty, dropped, cutoff, span)? else {
                return Ok(None);
            };
            let Some(body) = rebase_expr_dropping_locals(body, dropped, cutoff + 1, span)? else {
                return Ok(None);
            };
            Ok(Some(Expr::pi(binder.clone(), ty, body)))
        }
        Expr::Let {
            binder,
            ty,
            value,
            body,
        } => {
            let Some(ty) = rebase_expr_dropping_locals(ty, dropped, cutoff, span)? else {
                return Ok(None);
            };
            let Some(value) = rebase_expr_dropping_locals(value, dropped, cutoff, span)? else {
                return Ok(None);
            };
            let Some(body) = rebase_expr_dropping_locals(body, dropped, cutoff + 1, span)? else {
                return Ok(None);
            };
            Ok(Some(Expr::let_in(binder.clone(), ty, value, body)))
        }
    }
}

fn binder_group_end(binders: &[ResolvedBinder], start: usize) -> usize {
    let group_span = binders[start].span;
    let mut end = start + 1;
    while end < binders.len() && binders[end].span == group_span {
        end += 1;
    }
    end
}

fn weaken_group_type(ty: &Expr, offset: usize, span: Span) -> Result<Expr> {
    if offset == 0 {
        return Ok(ty.clone());
    }
    let amount = i32::try_from(offset).map_err(|_| {
        Diagnostic::error(
            DiagnosticKind::KernelRejected,
            span,
            "binder group is too large to elaborate",
        )
    })?;
    shift(ty, amount, 0).map_err(|error| diagnostic_from_kernel_error(span, error))
}

fn close_pi(body: Expr, binders: &[CoreBinder]) -> Expr {
    binders.iter().rev().fold(body, |body, binder| {
        Expr::pi(binder.name.clone(), binder.ty.clone(), body)
    })
}

fn core_binder_to_kernel(binder: &CoreBinder) -> Binder {
    Binder::new(binder.name.clone(), binder.ty.clone())
}

fn inductive_core_type(data: &InductiveDecl) -> Expr {
    let binders: Vec<_> = data
        .params
        .iter()
        .chain(data.indices.iter())
        .map(|binder| CoreBinder {
            name: binder.name.clone(),
            ty: binder.ty.clone(),
            binder_info: BinderInfo::Explicit,
            ty_metadata: TypeMetadata::default(),
            sort_level: None,
        })
        .collect();
    close_pi(Expr::sort(data.sort.clone()), &binders)
}

fn inductive_result_sort(ty: &Expr, span: Span) -> Result<Level> {
    let mut cursor = ty;
    while let Expr::Pi { body, .. } = cursor {
        cursor = body;
    }
    match cursor {
        Expr::Sort(level) => Ok(level.clone()),
        actual => Err(Diagnostic::error(
            DiagnosticKind::ExpectedSort,
            span,
            format!("expected inductive type to end in a Sort, found `{actual:?}`"),
        )),
    }
}

fn generated_recursor_decl(data: &InductiveDecl, span: Span) -> Result<Option<RecursorDecl>> {
    if !data.indices.is_empty() {
        return Ok(None);
    }

    let (universe_params, motive_sort) = recursor_universe_params_and_sort(data);
    let ty = generated_recursor_type(data, motive_sort, span)?;
    Ok(Some(RecursorDecl::new(
        format!("{}.rec", data.name),
        universe_params,
        ty,
    )))
}

fn recursor_universe_params_and_sort(data: &InductiveDecl) -> (Vec<String>, Level) {
    if data.sort == Level::zero() {
        return (data.universe_params.clone(), Level::zero());
    }

    let mut universe_params = data.universe_params.clone();
    let mut candidate = "u".to_owned();
    let mut index = 0usize;
    while universe_params.iter().any(|param| param == &candidate) {
        index += 1;
        candidate = format!("u{index}");
    }
    let motive_sort = Level::param(candidate.clone());
    universe_params.push(candidate);
    (universe_params, motive_sort)
}

fn generated_recursor_type(data: &InductiveDecl, motive_sort: Level, span: Span) -> Result<Expr> {
    let param_count = data.params.len();
    let motive_abs = param_count;
    let mut domains: Vec<(String, Expr)> = data
        .params
        .iter()
        .map(|param| (param.name.clone(), param.ty.clone()))
        .collect();

    let motive_target = inductive_target_in_ctx(data, param_count, span)?;
    domains.push((
        "motive".to_owned(),
        Expr::pi("_", motive_target, Expr::sort(motive_sort)),
    ));

    for (constructor_index, constructor) in data.constructors.iter().enumerate() {
        domains.push((
            minor_name(constructor),
            expected_minor_type(data, constructor, constructor_index, span)?,
        ));
    }

    let major_abs = motive_abs + 1 + data.constructors.len();
    let major_ctx_len = major_abs;
    domains.push((
        "major".to_owned(),
        inductive_target_in_ctx(data, major_ctx_len, span)?,
    ));
    let result = motive_app(
        major_ctx_len + 1,
        motive_abs,
        bvar_for_abs(major_ctx_len + 1, major_abs, span)?,
        span,
    )?;

    Ok(mk_pi_from_named_domains(domains, result))
}

fn minor_name(constructor: &ConstructorDecl) -> String {
    constructor
        .name
        .rsplit('.')
        .next()
        .map(|name| format!("{name}_case"))
        .unwrap_or_else(|| "minor".to_owned())
}

fn generated_recursor_metadata(data: &InductiveDecl, params: &[CoreBinder]) -> TypeMetadata {
    let mut binder_infos: Vec<_> = params
        .iter()
        .map(|binder| binder.binder_info.clone())
        .collect();
    binder_infos.push(BinderInfo::Explicit);
    binder_infos.extend(std::iter::repeat_n(
        BinderInfo::Explicit,
        data.constructors.len() + 1,
    ));
    TypeMetadata {
        binder_infos,
        domain_infos: Vec::new(),
    }
}

fn inductive_target_in_ctx(data: &InductiveDecl, ctx_len: usize, span: Span) -> Result<Expr> {
    let args = (0..data.params.len())
        .map(|param_abs| bvar_for_abs(ctx_len, param_abs, span))
        .collect::<Result<Vec<_>>>()?;
    Ok(Expr::apps(
        Expr::konst(
            data.name.clone(),
            data.universe_params
                .iter()
                .map(|param| Level::param(param.clone()))
                .collect(),
        ),
        args,
    ))
}

fn expected_minor_type(
    data: &InductiveDecl,
    constructor: &ConstructorDecl,
    constructor_index: usize,
    span: Span,
) -> Result<Expr> {
    let (domains, _) = peel_pi_domains_owned(&constructor.ty);
    let param_count = data.params.len();
    if domains.len() < param_count {
        return Err(Diagnostic::error(
            DiagnosticKind::KernelRejected,
            span,
            format!(
                "constructor `{}` is missing parameter binders",
                constructor.name
            ),
        ));
    }

    let prefix_len = param_count + 1 + constructor_index;
    let motive_abs = param_count;
    let mut source_to_target: Vec<usize> = (0..param_count).collect();
    let mut target_ctx_len = prefix_len;
    let mut expected_domains = Vec::new();
    let mut field_abs = Vec::new();

    for (field_index, field_domain) in domains[param_count..].iter().enumerate() {
        let source_ctx_len = param_count + field_index;
        expected_domains.push(remap_bvars(
            field_domain,
            source_ctx_len,
            target_ctx_len,
            &source_to_target,
            span,
        )?);

        source_to_target.push(target_ctx_len);
        field_abs.push(target_ctx_len);
        target_ctx_len += 1;

        if is_direct_recursive_domain(data, field_domain, source_ctx_len, span)? {
            expected_domains.push(motive_app(target_ctx_len, motive_abs, Expr::bvar(0), span)?);
            target_ctx_len += 1;
        }
    }

    let mut constructor_args = Vec::with_capacity(param_count + field_abs.len());
    for param_abs in 0..param_count {
        constructor_args.push(bvar_for_abs(target_ctx_len, param_abs, span)?);
    }
    for field_abs in field_abs {
        constructor_args.push(bvar_for_abs(target_ctx_len, field_abs, span)?);
    }

    let levels = data
        .universe_params
        .iter()
        .map(|param| Level::param(param.clone()))
        .collect();
    let constructor_value = Expr::apps(
        Expr::konst(constructor.name.clone(), levels),
        constructor_args,
    );
    let result = motive_app(target_ctx_len, motive_abs, constructor_value, span)?;

    Ok(mk_pi_from_domains(expected_domains, result))
}

fn motive_app(ctx_len: usize, motive_abs: usize, target: Expr, span: Span) -> Result<Expr> {
    Ok(Expr::app(bvar_for_abs(ctx_len, motive_abs, span)?, target))
}

fn bvar_for_abs(ctx_len: usize, abs: usize, span: Span) -> Result<Expr> {
    if abs >= ctx_len {
        return Err(Diagnostic::error(
            DiagnosticKind::KernelRejected,
            span,
            format!("binder index {abs} escapes context of length {ctx_len}"),
        ));
    }
    Ok(Expr::bvar((ctx_len - 1 - abs) as u32))
}

fn mk_pi_from_named_domains(domains: Vec<(String, Expr)>, body: Expr) -> Expr {
    domains
        .into_iter()
        .rev()
        .fold(body, |body, (name, domain)| Expr::pi(name, domain, body))
}

fn mk_pi_from_domains(domains: Vec<Expr>, body: Expr) -> Expr {
    domains
        .into_iter()
        .rev()
        .fold(body, |body, domain| Expr::pi("_", domain, body))
}

fn peel_pi_domains_owned(ty: &Expr) -> (Vec<Expr>, Expr) {
    let mut domains = Vec::new();
    let mut current = ty.clone();
    while let Expr::Pi { ty, body, .. } = current {
        domains.push(*ty);
        current = *body;
    }
    (domains, current)
}

fn remap_bvars(
    expr: &Expr,
    source_ctx_len: usize,
    target_ctx_len: usize,
    source_to_target: &[usize],
    span: Span,
) -> Result<Expr> {
    match expr {
        Expr::Sort(level) => Ok(Expr::sort(level.clone())),
        Expr::BVar(index) => {
            let index = *index as usize;
            if index >= source_ctx_len {
                return Err(Diagnostic::error(
                    DiagnosticKind::KernelRejected,
                    span,
                    format!("binder index {index} escapes context of length {source_ctx_len}"),
                ));
            }
            let source_abs = source_ctx_len - 1 - index;
            let Some(target_abs) = source_to_target.get(source_abs).copied() else {
                return Err(Diagnostic::error(
                    DiagnosticKind::KernelRejected,
                    span,
                    format!("binder index {index} has no target in recursor minor"),
                ));
            };
            bvar_for_abs(target_ctx_len, target_abs, span)
        }
        Expr::Const { name, levels } => Ok(Expr::konst(name.clone(), levels.clone())),
        Expr::App(fun, arg) => Ok(Expr::app(
            remap_bvars(fun, source_ctx_len, target_ctx_len, source_to_target, span)?,
            remap_bvars(arg, source_ctx_len, target_ctx_len, source_to_target, span)?,
        )),
        Expr::Lam { binder, ty, body } => {
            let mut body_map = source_to_target.to_vec();
            body_map.push(target_ctx_len);
            Ok(Expr::lam(
                binder.clone(),
                remap_bvars(ty, source_ctx_len, target_ctx_len, source_to_target, span)?,
                remap_bvars(
                    body,
                    source_ctx_len + 1,
                    target_ctx_len + 1,
                    &body_map,
                    span,
                )?,
            ))
        }
        Expr::Pi { binder, ty, body } => {
            let mut body_map = source_to_target.to_vec();
            body_map.push(target_ctx_len);
            Ok(Expr::pi(
                binder.clone(),
                remap_bvars(ty, source_ctx_len, target_ctx_len, source_to_target, span)?,
                remap_bvars(
                    body,
                    source_ctx_len + 1,
                    target_ctx_len + 1,
                    &body_map,
                    span,
                )?,
            ))
        }
        Expr::Let {
            binder,
            ty,
            value,
            body,
        } => {
            let mut body_map = source_to_target.to_vec();
            body_map.push(target_ctx_len);
            Ok(Expr::let_in(
                binder.clone(),
                remap_bvars(ty, source_ctx_len, target_ctx_len, source_to_target, span)?,
                remap_bvars(
                    value,
                    source_ctx_len,
                    target_ctx_len,
                    source_to_target,
                    span,
                )?,
                remap_bvars(
                    body,
                    source_ctx_len + 1,
                    target_ctx_len + 1,
                    &body_map,
                    span,
                )?,
            ))
        }
    }
}

fn is_direct_recursive_domain(
    data: &InductiveDecl,
    domain: &Expr,
    ctx_len: usize,
    span: Span,
) -> Result<bool> {
    let (head, args) = npa_kernel::expr::collect_apps(domain);
    let levels = match head {
        Expr::Const { name, levels } if name == data.name => levels,
        _ => return Ok(false),
    };

    let expected_levels: Vec<_> = data
        .universe_params
        .iter()
        .map(|param| Level::param(param.clone()))
        .collect();
    if !npa_kernel::level::levels_eq(&levels, &expected_levels)
        || args.len() != data.params.len() + data.indices.len()
    {
        return Ok(false);
    }

    for (param_index, arg) in args.iter().take(data.params.len()).enumerate() {
        if arg != &bvar_for_abs(ctx_len, param_index, span)? {
            return Ok(false);
        }
    }

    Ok(args.iter().all(|arg| !contains_const(arg, &data.name)))
}

fn contains_const(expr: &Expr, needle: &str) -> bool {
    match expr {
        Expr::Sort(_) | Expr::BVar(_) => false,
        Expr::Const { name, .. } => name == needle,
        Expr::App(fun, arg) => contains_const(fun, needle) || contains_const(arg, needle),
        Expr::Lam { ty, body, .. } | Expr::Pi { ty, body, .. } => {
            contains_const(ty, needle) || contains_const(body, needle)
        }
        Expr::Let {
            ty, value, body, ..
        } => {
            contains_const(ty, needle)
                || contains_const(value, needle)
                || contains_const(body, needle)
        }
    }
}

fn close_pi_sort(body_sort: Level, binders: &[CoreBinder], span: Span) -> Result<Level> {
    binders.iter().rev().try_fold(body_sort, |sort, binder| {
        let domain_sort = binder.sort_level.clone().ok_or_else(|| {
            Diagnostic::error(
                DiagnosticKind::ExpectedSort,
                span,
                "Pi binder sort was not available",
            )
        })?;
        Ok(Level::imax(domain_sort, sort))
    })
}

fn close_lam(body: Expr, binders: &[CoreBinder]) -> Expr {
    binders.iter().rev().fold(body, |body, binder| {
        Expr::lam(binder.name.clone(), binder.ty.clone(), body)
    })
}

fn binder_name(binder: &ResolvedBinder) -> String {
    match &binder.kind {
        SurfaceBinderKind::Named(name) => name.parts.join("."),
        SurfaceBinderKind::Anonymous => "_".to_owned(),
    }
}

fn universe_param_names(params: &[SurfaceUniverseParam]) -> Result<Vec<String>> {
    let mut names = Vec::new();
    for param in params {
        if names.iter().any(|name| name == &param.name) {
            return Err(Diagnostic::error(
                DiagnosticKind::DuplicateUniverseParam,
                param.span,
                format!("duplicate universe parameter `{}`", param.name),
            ));
        }
        names.push(param.name.clone());
    }
    Ok(names)
}

fn global_name(global: &crate::ElabGlobalRef) -> String {
    match global {
        crate::ElabGlobalRef::Local { name, .. }
        | crate::ElabGlobalRef::LocalGenerated { name, .. }
        | crate::ElabGlobalRef::Imported { name, .. } => name.to_dotted(),
    }
}

fn sorted_global_candidates(candidates: &[ElabGlobalRef]) -> Vec<ElabGlobalRef> {
    let mut candidates = candidates.to_vec();
    candidates.sort_by(global_candidate_cmp);
    candidates.dedup();
    candidates
}

fn global_candidate_cmp(lhs: &ElabGlobalRef, rhs: &ElabGlobalRef) -> std::cmp::Ordering {
    global_name(lhs)
        .cmp(&global_name(rhs))
        .then(global_candidate_kind_rank(lhs).cmp(&global_candidate_kind_rank(rhs)))
        .then_with(|| match (lhs, rhs) {
            (
                ElabGlobalRef::Local {
                    decl_index: lhs_index,
                    ..
                },
                ElabGlobalRef::Local {
                    decl_index: rhs_index,
                    ..
                },
            )
            | (
                ElabGlobalRef::LocalGenerated {
                    decl_index: lhs_index,
                    ..
                },
                ElabGlobalRef::LocalGenerated {
                    decl_index: rhs_index,
                    ..
                },
            ) => lhs_index.cmp(rhs_index),
            (
                ElabGlobalRef::Imported {
                    module: lhs_module,
                    decl_interface_hash: lhs_hash,
                    ..
                },
                ElabGlobalRef::Imported {
                    module: rhs_module,
                    decl_interface_hash: rhs_hash,
                    ..
                },
            ) => lhs_module.cmp(rhs_module).then(lhs_hash.cmp(rhs_hash)),
            _ => std::cmp::Ordering::Equal,
        })
}

fn global_candidate_kind_rank(global: &ElabGlobalRef) -> u8 {
    match global {
        ElabGlobalRef::Local { .. } => 0,
        ElabGlobalRef::LocalGenerated { .. } => 1,
        ElabGlobalRef::Imported { .. } => 2,
    }
}

fn resolved_ident_for_global(
    global: &ElabGlobalRef,
    universe_args: Option<&[SurfaceLevel]>,
    implicit_mode: ImplicitMode,
    span: Span,
) -> ResolvedExpr {
    let name = surface_name_for_global(global, span);
    ResolvedExpr::Ident {
        name,
        resolved: ResolvedName::Global(global.clone()),
        universe_args: universe_args.map(|args| args.to_vec()),
        implicit_mode,
        span,
    }
}

fn notation_candidate_expr(
    global: &ElabGlobalRef,
    args: &[ResolvedExpr],
    span: Span,
) -> ResolvedExpr {
    overloaded_candidate_app_expr(global, None, ImplicitMode::Insert, args, span)
}

fn overloaded_candidate_app_expr(
    global: &ElabGlobalRef,
    universe_args: Option<&[SurfaceLevel]>,
    implicit_mode: ImplicitMode,
    args: &[ResolvedExpr],
    span: Span,
) -> ResolvedExpr {
    args.iter().cloned().fold(
        resolved_ident_for_global(global, universe_args, implicit_mode, span),
        |func, arg| {
            let app_span = resolved_expr_span(&func).join(resolved_expr_span(&arg));
            ResolvedExpr::App {
                func: Box::new(func),
                arg: Box::new(arg),
                span: app_span,
            }
        },
    )
}

struct OverloadedAppSpine {
    candidates: Vec<ElabGlobalRef>,
    universe_args: Option<Vec<SurfaceLevel>>,
    implicit_mode: ImplicitMode,
    args: Vec<ResolvedExpr>,
}

fn overloaded_app_spine(expr: &ResolvedExpr) -> Option<OverloadedAppSpine> {
    let mut args = Vec::new();
    let mut cursor = expr;
    while let ResolvedExpr::App { func, arg, .. } = cursor {
        args.push((**arg).clone());
        cursor = func;
    }
    args.reverse();
    let ResolvedExpr::Ident {
        resolved: ResolvedName::Overloaded(candidates),
        universe_args,
        implicit_mode,
        ..
    } = cursor
    else {
        return None;
    };
    Some(OverloadedAppSpine {
        candidates: candidates.clone(),
        universe_args: universe_args.clone(),
        implicit_mode: *implicit_mode,
        args,
    })
}

fn surface_name_for_global(global: &ElabGlobalRef, span: Span) -> SurfaceName {
    SurfaceName {
        parts: global_name(global).split('.').map(str::to_owned).collect(),
        span,
    }
}

fn add_decl_to_env(env: &mut Env, decl: &Decl, span: Span) -> Result<()> {
    if let Some(existing) = env.decl(decl.name()) {
        if existing == decl {
            return Ok(());
        }
    }

    match decl.clone() {
        Decl::Axiom {
            name,
            universe_params,
            ty,
        } => env
            .add_axiom(name, universe_params, ty)
            .map_err(|error| kernel_rejected(span, error))?,
        Decl::Def {
            name,
            universe_params,
            ty,
            value,
            reducibility,
        } => env
            .add_def(name, universe_params, ty, value, reducibility)
            .map_err(|error| kernel_rejected(span, error))?,
        Decl::Theorem {
            name,
            universe_params,
            ty,
            proof,
        } => env
            .add_theorem(name, universe_params, ty, proof)
            .map_err(|error| kernel_rejected(span, error))?,
        Decl::Inductive { data, .. } => env
            .add_inductive(*data)
            .map_err(|error| kernel_rejected(span, error))?,
        Decl::Constructor { name, .. } | Decl::Recursor { name, .. } => {
            return Err(Diagnostic::error(
                DiagnosticKind::KernelRejected,
                span,
                format!("imported generated declaration `{name}` is missing its inductive"),
            ));
        }
    }

    Ok(())
}

fn diagnostic_from_kernel_error(span: Span, error: KernelError) -> Diagnostic {
    match error {
        KernelError::ExpectedSort { .. } => Diagnostic::error(
            DiagnosticKind::ExpectedSort,
            span,
            format!("kernel expected a sort: {error:?}"),
        ),
        KernelError::ExpectedPi { .. } => Diagnostic::error(
            DiagnosticKind::ExpectedFunctionType,
            span,
            format!("kernel expected a function type: {error:?}"),
        ),
        KernelError::TypeMismatch { .. } | KernelError::NotDefEq { .. } => Diagnostic::error(
            DiagnosticKind::TypeMismatch,
            span,
            format!("kernel type mismatch: {error:?}"),
        ),
        KernelError::UnknownUniverseParam(name) => Diagnostic::error(
            DiagnosticKind::UnknownUniverseParam,
            span,
            format!("unknown universe parameter `{name}`"),
        ),
        KernelError::BadUniverseArity { .. } => Diagnostic::error(
            DiagnosticKind::UnsolvedUniverseMeta,
            span,
            format!("bad universe argument arity: {error:?}"),
        ),
        other => kernel_rejected(span, other),
    }
}

fn kernel_rejected(span: Span, error: KernelError) -> Diagnostic {
    Diagnostic::error(
        DiagnosticKind::KernelRejected,
        span,
        format!("kernel rejected elaborated declaration: {error:?}"),
    )
}

fn resolved_expr_span(expr: &ResolvedExpr) -> Span {
    match expr {
        ResolvedExpr::Ident { span, .. }
        | ResolvedExpr::Sort { span, .. }
        | ResolvedExpr::App { span, .. }
        | ResolvedExpr::Lam { span, .. }
        | ResolvedExpr::Pi { span, .. }
        | ResolvedExpr::Let { span, .. }
        | ResolvedExpr::Annot { span, .. }
        | ResolvedExpr::Hole { span, .. }
        | ResolvedExpr::Notation { span, .. } => *span,
    }
}

fn resolved_item_span(item: &ResolvedItem) -> Span {
    match item {
        ResolvedItem::Import { span, .. }
        | ResolvedItem::Open { span, .. }
        | ResolvedItem::Namespace { span, .. }
        | ResolvedItem::End { span, .. }
        | ResolvedItem::Inductive { span, .. } => *span,
        ResolvedItem::Notation(decl) => decl.span,
        ResolvedItem::Def(decl) | ResolvedItem::Theorem(decl) | ResolvedItem::Axiom(decl) => {
            decl.span
        }
    }
}

#[cfg(test)]
mod tests {
    use npa_kernel::{Binder, ConstructorDecl, Decl, Expr, InductiveDecl, Level, Reducibility};

    use super::*;
    use crate::{ImportedDeclaration, ImportedNotation, ImportedTypeMetadata, Name};

    fn prelude_import() -> VerifiedImport {
        VerifiedImport {
            module: Name::from_dotted("Std.Prelude"),
            export_hash: "sha256:prelude".to_owned(),
            declarations: vec![
                ImportedDeclaration {
                    name: Name::from_dotted("Nat"),
                    decl_interface_hash: "sha256:Nat".to_owned(),
                    binder_infos: Vec::new(),
                    domain_infos: Vec::new(),
                    type_value_metadata: None,
                },
                ImportedDeclaration {
                    name: Name::from_dotted("Nat.zero"),
                    decl_interface_hash: "sha256:Nat.zero".to_owned(),
                    binder_infos: Vec::new(),
                    domain_infos: Vec::new(),
                    type_value_metadata: None,
                },
                ImportedDeclaration {
                    name: Name::from_dotted("Nat.succ"),
                    decl_interface_hash: "sha256:Nat.succ".to_owned(),
                    binder_infos: Vec::new(),
                    domain_infos: Vec::new(),
                    type_value_metadata: None,
                },
                ImportedDeclaration {
                    name: Name::from_dotted("Eq"),
                    decl_interface_hash: "sha256:Eq".to_owned(),
                    binder_infos: vec![
                        BinderInfo::Implicit,
                        BinderInfo::Explicit,
                        BinderInfo::Explicit,
                    ],
                    domain_infos: Vec::new(),
                    type_value_metadata: None,
                },
                ImportedDeclaration {
                    name: Name::from_dotted("Eq.refl"),
                    decl_interface_hash: "sha256:Eq.refl".to_owned(),
                    binder_infos: vec![BinderInfo::Implicit, BinderInfo::Explicit],
                    domain_infos: Vec::new(),
                    type_value_metadata: None,
                },
            ],
            notations: Vec::new(),
            kernel_declarations: Vec::new(),
        }
    }

    fn custom_type_import() -> VerifiedImport {
        VerifiedImport {
            module: Name::from_dotted("Custom"),
            export_hash: "sha256:custom".to_owned(),
            declarations: vec![ImportedDeclaration {
                name: Name::from_dotted("Foo"),
                decl_interface_hash: "sha256:Foo".to_owned(),
                binder_infos: Vec::new(),
                domain_infos: Vec::new(),
                type_value_metadata: None,
            }],
            notations: Vec::new(),
            kernel_declarations: vec![Decl::Axiom {
                name: "Foo".to_owned(),
                universe_params: Vec::new(),
                ty: Expr::sort(Level::succ(Level::zero())),
            }],
        }
    }

    fn custom_poly_import() -> VerifiedImport {
        VerifiedImport {
            module: Name::from_dotted("CustomPoly"),
            export_hash: "sha256:custom-poly".to_owned(),
            declarations: vec![ImportedDeclaration {
                name: Name::from_dotted("poly_id"),
                decl_interface_hash: "sha256:poly_id".to_owned(),
                binder_infos: vec![BinderInfo::Implicit, BinderInfo::Explicit],
                domain_infos: Vec::new(),
                type_value_metadata: None,
            }],
            notations: Vec::new(),
            kernel_declarations: vec![Decl::Axiom {
                name: "poly_id".to_owned(),
                universe_params: vec!["u".to_owned()],
                ty: Expr::pi(
                    "A",
                    Expr::sort(Level::param("u")),
                    Expr::pi("x", Expr::bvar(0), Expr::bvar(1)),
                ),
            }],
        }
    }

    fn custom_poly_notation_import() -> VerifiedImport {
        let mut import = custom_poly_import();
        import.notations = vec![ImportedNotation {
            kind: crate::NotationKind::Prefix,
            precedence: 70,
            symbol: "!".to_owned(),
            target: Name::from_dotted("poly_id"),
            decl_interface_hash: "sha256:poly_id".to_owned(),
            namespace: None,
        }];
        import
    }

    fn custom_higher_order_import() -> VerifiedImport {
        let type0 = Expr::sort(Level::succ(Level::zero()));
        let id_ty = Expr::pi("A", type0, Expr::pi("x", Expr::bvar(0), Expr::bvar(1)));
        let higher_order_arg_ty = Expr::pi("f", id_ty.clone(), Expr::konst("Nat", Vec::new()));

        VerifiedImport {
            module: Name::from_dotted("CustomHigherOrder"),
            export_hash: "sha256:custom-higher-order".to_owned(),
            declarations: vec![ImportedDeclaration {
                name: Name::from_dotted("k"),
                decl_interface_hash: "sha256:k".to_owned(),
                binder_infos: Vec::new(),
                domain_infos: vec![ImportedTypeMetadata {
                    binder_infos: vec![BinderInfo::Explicit],
                    domain_infos: vec![ImportedTypeMetadata {
                        binder_infos: vec![BinderInfo::Implicit, BinderInfo::Explicit],
                        domain_infos: Vec::new(),
                    }],
                }],
                type_value_metadata: None,
            }],
            notations: Vec::new(),
            kernel_declarations: vec![Decl::Axiom {
                name: "k".to_owned(),
                universe_params: Vec::new(),
                ty: Expr::pi("g", higher_order_arg_ty, Expr::konst("Nat", Vec::new())),
            }],
        }
    }

    fn custom_type_alias_import() -> VerifiedImport {
        let type0 = Expr::sort(Level::succ(Level::zero()));
        let id_ty = Expr::pi("A", type0, Expr::pi("x", Expr::bvar(0), Expr::bvar(1)));

        VerifiedImport {
            module: Name::from_dotted("CustomTypeAlias"),
            export_hash: "sha256:custom-type-alias".to_owned(),
            declarations: vec![ImportedDeclaration {
                name: Name::from_dotted("IdTy"),
                decl_interface_hash: "sha256:IdTy".to_owned(),
                binder_infos: Vec::new(),
                domain_infos: Vec::new(),
                type_value_metadata: Some(ImportedTypeMetadata {
                    binder_infos: vec![BinderInfo::Implicit, BinderInfo::Explicit],
                    domain_infos: Vec::new(),
                }),
            }],
            notations: Vec::new(),
            kernel_declarations: vec![Decl::Def {
                name: "IdTy".to_owned(),
                universe_params: Vec::new(),
                ty: Expr::sort(Level::succ(Level::succ(Level::zero()))),
                value: id_ty,
                reducibility: Reducibility::Reducible,
            }],
        }
    }

    fn custom_box_import() -> VerifiedImport {
        let u = Level::param("u");
        let box_ty = Expr::pi("A", Expr::sort(u.clone()), Expr::sort(u.clone()));
        let box_ctor_ty = Expr::pi(
            "A",
            Expr::sort(u.clone()),
            Expr::pi(
                "x",
                Expr::bvar(0),
                Expr::app(Expr::konst("Box", vec![u.clone()]), Expr::bvar(1)),
            ),
        );
        let data = InductiveDecl::new(
            "Box",
            vec!["u".to_owned()],
            vec![Binder::new("A", Expr::sort(u.clone()))],
            Vec::new(),
            u,
            vec![ConstructorDecl::new("Box.mk", box_ctor_ty)],
            None,
        );

        VerifiedImport {
            module: Name::from_dotted("CustomBox"),
            export_hash: "sha256:custom-box".to_owned(),
            declarations: vec![
                ImportedDeclaration {
                    name: Name::from_dotted("Box"),
                    decl_interface_hash: "sha256:Box".to_owned(),
                    binder_infos: vec![BinderInfo::Implicit],
                    domain_infos: Vec::new(),
                    type_value_metadata: None,
                },
                ImportedDeclaration {
                    name: Name::from_dotted("Box.mk"),
                    decl_interface_hash: "sha256:Box.mk".to_owned(),
                    binder_infos: vec![BinderInfo::Implicit, BinderInfo::Explicit],
                    domain_infos: Vec::new(),
                    type_value_metadata: None,
                },
            ],
            notations: Vec::new(),
            kernel_declarations: vec![Decl::Inductive {
                name: "Box".to_owned(),
                universe_params: vec!["u".to_owned()],
                ty: box_ty,
                data: Box::new(data),
            }],
        }
    }

    fn elaborate(source: &str) -> Result<ElaboratedModule> {
        elaborate_source(
            FileId(0),
            Name::from_dotted("Scratch"),
            source,
            &[prelude_import()],
        )
    }

    #[test]
    fn elaborates_explicit_polymorphic_identity_def() {
        let module = elaborate(
            r#"
def id.{u} (A : Sort u) (x : A) : A := x
"#,
        )
        .expect("definition should elaborate");

        assert_eq!(module.declarations.len(), 1);
        let Decl::Def {
            name,
            universe_params,
            ..
        } = &module.declarations[0]
        else {
            panic!("expected definition");
        };
        assert_eq!(name, "id");
        assert_eq!(universe_params, &["u".to_owned()]);
    }

    #[test]
    fn elaborates_local_global_app_lambda_pi_let_and_annotation() {
        let module = elaborate(
            r#"
import Std.Prelude
def id_nat : forall (x : Nat), Nat := fun x => x
def use : Nat := let x : Nat := Nat.zero in (id_nat x : Nat)
"#,
        )
        .expect("definitions should elaborate");

        assert_eq!(module.declarations.len(), 2);
        assert!(matches!(
            &module.declarations[0],
            Decl::Def { name, .. } if name == "id_nat"
        ));
        assert!(matches!(
            &module.declarations[1],
            Decl::Def { name, .. } if name == "use"
        ));
    }

    #[test]
    fn elaborates_axiom_and_theorem() {
        let module = elaborate(
            r#"
import Std.Prelude
axiom Trusted : Type
theorem zero_refl : @Eq.{1} Nat Nat.zero Nat.zero := @Eq.refl.{1} Nat Nat.zero
"#,
        )
        .expect("axiom and theorem should elaborate");

        assert_eq!(module.declarations.len(), 2);
        assert!(matches!(
            &module.declarations[0],
            Decl::Axiom { name, .. } if name == "Trusted"
        ));
        assert!(matches!(
            &module.declarations[1],
            Decl::Theorem { name, .. } if name == "zero_refl"
        ));
    }

    #[test]
    fn elaborates_simple_inductive_and_generated_recursor() {
        let module = elaborate(
            r#"
inductive MyNat : Type where
| zero : MyNat
| succ : MyNat -> MyNat
axiom P : MyNat -> Type
axiom z : P MyNat.zero
axiom s : forall (n : MyNat), P n -> P (MyNat.succ n)
def rec_use (n : MyNat) : P n := MyNat.rec P z s n
"#,
        )
        .expect("simple inductive and generated recursor should elaborate");

        assert_eq!(module.declarations.len(), 5);
        let Decl::Inductive { name, data, .. } = &module.declarations[0] else {
            panic!("expected inductive declaration");
        };
        assert_eq!(name, "MyNat");
        assert!(data.recursor.is_some());
        assert!(matches!(
            &module.declarations[4],
            Decl::Def { name, .. } if name == "rec_use"
        ));
    }

    #[test]
    fn elaborates_indexed_constructor_with_temporary_inductive_global() {
        let module = elaborate(
            r#"
import Std.Prelude
inductive Eq2.{u} {A : Sort u} (a : A) : forall (b : A), Prop where
| refl : Eq2.{u} a a
theorem zero_refl2 : Eq2.{1} Nat.zero Nat.zero := Eq2.refl Nat.zero
"#,
        )
        .expect("indexed constructor should see the temporary inductive head");

        assert_eq!(module.declarations.len(), 2);
        let Decl::Inductive { data, .. } = &module.declarations[0] else {
            panic!("expected inductive declaration");
        };
        assert!(data.recursor.is_none());
        assert!(matches!(
            &module.declarations[1],
            Decl::Theorem { name, .. } if name == "zero_refl2"
        ));
    }

    #[test]
    fn does_not_reserve_recursor_for_hidden_indexed_inductive() {
        let module = elaborate(
            r#"
import Std.Prelude
def Indexed : Type 1 := forall (n : Nat), Type
inductive Hidden : Indexed where
| mk : Hidden Nat.zero
namespace Hidden
axiom rec : Type
end Hidden
"#,
        )
        .expect("hidden indexed inductive should not reserve a missing recursor name");

        assert_eq!(module.declarations.len(), 3);
        let Decl::Inductive { data, .. } = &module.declarations[1] else {
            panic!("expected inductive declaration");
        };
        assert!(data.recursor.is_none());
        assert!(matches!(
            &module.declarations[2],
            Decl::Axiom { name, .. } if name == "Hidden.rec"
        ));
    }

    #[test]
    fn elaborates_qualified_constructor_names_relative_to_inductive() {
        let module = elaborate(
            r#"
inductive Qual : Type where
| Extra.mk : Qual
def use_qualified_ctor : Qual := Qual.Extra.mk
"#,
        )
        .expect("qualified constructor names should be relative to the inductive name");

        assert_eq!(module.declarations.len(), 2);
        let Decl::Inductive { data, .. } = &module.declarations[0] else {
            panic!("expected inductive declaration");
        };
        assert_eq!(data.constructors[0].name, "Qual.Extra.mk");
        assert!(matches!(
            &module.declarations[1],
            Decl::Def { name, .. } if name == "use_qualified_ctor"
        ));
    }

    #[test]
    fn rejects_bad_inductive_constructor_result_at_kernel_handoff() {
        let err = elaborate(
            r#"
inductive Bad : Type where
| mk : Type
"#,
        )
        .expect_err("kernel must reject constructor results outside the inductive family");
        assert_eq!(err.kind, DiagnosticKind::KernelRejected);
    }

    #[test]
    fn elaborates_implicit_arguments_and_universe_metas() {
        let module = elaborate(
            r#"
import Std.Prelude
axiom poly_id.{u} {A : Sort u} (x : A) : A
def use_auto : Nat := poly_id Nat.zero
def use_explicit : Nat := @poly_id Nat Nat.zero
"#,
        )
        .expect("implicit arguments and universe metas should elaborate");

        assert_eq!(module.declarations.len(), 3);
        assert!(matches!(
            &module.declarations[1],
            Decl::Def { name, .. } if name == "use_auto"
        ));
        assert!(matches!(
            &module.declarations[2],
            Decl::Def { name, .. } if name == "use_explicit"
        ));
    }

    #[test]
    fn accepts_implicit_function_against_explicit_pi_expected_type() {
        let module = elaborate(
            r#"
import Std.Prelude
axiom poly_id {A : Type} (x : A) : A
def use_pi : (forall (A : Type), A -> A) := poly_id
"#,
        )
        .expect("implicit BinderInfo should not affect core Pi equality");

        assert_eq!(module.declarations.len(), 2);
        assert!(matches!(
            &module.declarations[1],
            Decl::Def { name, .. } if name == "use_pi"
        ));
    }

    #[test]
    fn solves_expected_type_holes_before_inserting_implicit_args() {
        let module = elaborate(
            r#"
import Std.Prelude
axiom poly_id {A : Type} (x : A) : A
def use_pi : (forall (A : Type), _) := poly_id
"#,
        )
        .expect("expected type holes should be solved before inserting implicit args");

        assert_eq!(module.declarations.len(), 2);
        assert!(matches!(
            &module.declarations[1],
            Decl::Def { name, .. } if name == "use_pi"
        ));
    }

    #[test]
    fn elaborates_builtin_eq_refl_with_implicit_type_argument() {
        let module = elaborate(
            r#"
import Std.Prelude
theorem zero_refl_builtin : Eq.{1} Nat.zero Nat.zero := Eq.refl Nat.zero
"#,
        )
        .expect("Eq.refl should use built-in implicit binder metadata");

        assert_eq!(module.declarations.len(), 1);
        assert!(matches!(
            &module.declarations[0],
            Decl::Theorem { name, .. } if name == "zero_refl_builtin"
        ));
    }

    #[test]
    fn at_mode_does_not_auto_insert_implicit_term_arguments() {
        let err = elaborate(
            r#"
import Std.Prelude
axiom poly_id.{u} {A : Sort u} (x : A) : A
def bad : Nat := @poly_id Nat.zero
"#,
        )
        .expect_err("@ mode must require the implicit argument position explicitly");

        assert_eq!(err.kind, DiagnosticKind::TypeMismatch);
    }

    #[test]
    fn solves_user_hole_from_expected_type_constraints() {
        let module = elaborate(
            r#"
import Std.Prelude
axiom refl.{u} {A : Sort u} (x : A) : @Eq.{u} A x x
theorem zero_refl : @Eq.{1} Nat Nat.zero Nat.zero := refl _
"#,
        )
        .expect("hole should be solved by the expected equality target");

        assert_eq!(module.declarations.len(), 2);
        assert!(matches!(
            &module.declarations[1],
            Decl::Theorem { name, .. } if name == "zero_refl"
        ));
    }

    #[test]
    fn rejects_unsolved_user_holes_before_kernel_handoff() {
        let err = elaborate("def missing : Type := _").expect_err("unsolved hole must fail");
        assert_eq!(err.kind, DiagnosticKind::UnsolvedHole);
    }

    #[test]
    fn rejects_unsolved_pi_type_holes_before_kernel_handoff() {
        let err = elaborate("axiom f : forall (x : _), Type")
            .expect_err("Pi type hole must fail through frontend metavariable diagnostics");
        assert_eq!(err.kind, DiagnosticKind::UnsolvedHole);
    }

    #[test]
    fn solves_pi_type_holes_from_annotated_lambda_binders() {
        let module = elaborate(
            r#"
import Std.Prelude
def f : (forall (x : _), Type) := fun (x : Nat) => Nat
"#,
        )
        .expect("Pi binder type hole should be solved without leaking a universe meta");

        assert_eq!(module.declarations.len(), 1);
        assert!(matches!(
            &module.declarations[0],
            Decl::Def { name, .. } if name == "f"
        ));
    }

    #[test]
    fn zonks_solved_annotation_type_before_sort_check() {
        let module = elaborate(
            r#"
import Std.Prelude
axiom T : (Nat : _)
"#,
        )
        .expect("solved annotation type hole should be visible to type elaboration");

        assert_eq!(module.declarations.len(), 1);
        assert!(matches!(
            &module.declarations[0],
            Decl::Axiom { name, .. } if name == "T"
        ));
    }

    #[test]
    fn preserves_implicit_binders_from_hole_annotation() {
        let module = elaborate(
            r#"
import Std.Prelude
def use : Nat := ((fun {A : Type} (x : A) => x) : _) Nat.zero
"#,
        )
        .expect("hole annotation should keep inferred implicit binder metadata");

        assert_eq!(module.declarations.len(), 1);
        assert!(matches!(
            &module.declarations[0],
            Decl::Def { name, .. } if name == "use"
        ));
    }

    #[test]
    fn preserves_implicit_binders_from_let_type_hole() {
        let module = elaborate(
            r#"
import Std.Prelude
def use : Nat :=
  let f : _ := fun {A : Type} (x : A) => x in f Nat.zero
"#,
        )
        .expect("let type hole should keep inferred implicit binder metadata");

        assert_eq!(module.declarations.len(), 1);
        assert!(matches!(
            &module.declarations[0],
            Decl::Def { name, .. } if name == "use"
        ));
    }

    #[test]
    fn solves_dependent_pi_holes_after_zonking_contexts() {
        let module = elaborate(
            r#"
import Std.Prelude
def f : (forall (x : _), _) := fun (x : Nat) => x
"#,
        )
        .expect("dependent Pi holes should compare solved context snapshots");

        assert_eq!(module.declarations.len(), 1);
        assert!(matches!(
            &module.declarations[0],
            Decl::Def { name, .. } if name == "f"
        ));
    }

    #[test]
    fn synthesizes_pi_for_remaining_checked_lambda_binders() {
        let module = elaborate(
            r#"
import Std.Prelude
def f : (forall (A : Type), _) := fun (A : Type) (x : A) => x
"#,
        )
        .expect("annotated lambda binders should refine an expected type hole into a Pi");

        assert_eq!(module.declarations.len(), 1);
        assert!(matches!(
            &module.declarations[0],
            Decl::Def { name, .. } if name == "f"
        ));
    }

    #[test]
    fn preserves_implicit_metadata_for_checked_lambda_domain_locals() {
        let module = elaborate(
            r#"
import Std.Prelude
def use : (forall (f : forall {A : Type}, A -> A), Nat) := fun f => f Nat.zero
"#,
        )
        .expect("checked lambda locals should keep implicit metadata from their expected domain");

        assert_eq!(module.declarations.len(), 1);
        assert!(matches!(
            &module.declarations[0],
            Decl::Def { name, .. } if name == "use"
        ));
    }

    #[test]
    fn preserves_implicit_metadata_for_application_domain_arguments() {
        let module = elaborate(
            r#"
import Std.Prelude
def use (k : forall (h : forall (f : forall {A : Type}, A -> A), Nat), Nat) : Nat :=
  k (fun f => f Nat.zero)
"#,
        )
        .expect("application argument expected types should preserve nested implicit metadata");

        assert_eq!(module.declarations.len(), 1);
        assert!(matches!(
            &module.declarations[0],
            Decl::Def { name, .. } if name == "use"
        ));
    }

    #[test]
    fn solves_binder_type_holes_from_extended_context() {
        let module = elaborate(
            r#"
import Std.Prelude
def f (x : _) : Nat := x
"#,
        )
        .expect("binder type hole should solve from body constraints");

        assert_eq!(module.declarations.len(), 1);
        assert!(matches!(
            &module.declarations[0],
            Decl::Def { name, .. } if name == "f"
        ));
    }

    #[test]
    fn rebases_binder_type_holes_to_outer_context() {
        let module = elaborate(
            r#"
def f (A : Type) (x : _) : A := x
"#,
        )
        .expect("binder type hole assignment should rebase to the creation context");

        assert_eq!(module.declarations.len(), 1);
        assert!(matches!(
            &module.declarations[0],
            Decl::Def { name, .. } if name == "f"
        ));
    }

    #[test]
    fn constrains_inferred_type_holes_before_sort_check() {
        let module = elaborate(
            r#"
def f : Type 1 := forall (A : _), A
"#,
        )
        .expect("type-position locals with inferred hole types should refine to sorts");

        assert_eq!(module.declarations.len(), 1);
        assert!(matches!(
            &module.declarations[0],
            Decl::Def { name, .. } if name == "f"
        ));
    }

    #[test]
    fn solves_prop_type_holes_before_defaulting_to_type() {
        let module = elaborate(
            r#"
def f : Prop := forall (A : _), A
"#,
        )
        .expect("expected Prop should solve the type hole universe before defaulting");

        assert_eq!(module.declarations.len(), 1);
        assert!(matches!(
            &module.declarations[0],
            Decl::Def { name, .. } if name == "f"
        ));
    }

    #[test]
    fn constrains_dependent_binder_type_holes_before_sort_check() {
        let module = elaborate(
            r#"
def f (A : _) (x : A) : A := x
"#,
        )
        .expect("dependent binder type holes should refine through later type positions");

        assert_eq!(module.declarations.len(), 1);
        assert!(matches!(
            &module.declarations[0],
            Decl::Def { name, .. } if name == "f"
        ));
    }

    #[test]
    fn rejects_meta_to_meta_holes_before_kernel_handoff() {
        let err = elaborate("def f : _ := (_ : _)")
            .expect_err("unsolved meta chain must fail in frontend");
        assert_eq!(err.kind, DiagnosticKind::UnsolvedHole);
    }

    #[test]
    fn solves_type_holes_from_checked_values() {
        let module = elaborate(
            r#"
import Std.Prelude
def inferred_type : _ := Nat.zero
"#,
        )
        .expect("type hole should be solved from the checked value");

        assert_eq!(module.declarations.len(), 1);
        assert!(matches!(
            &module.declarations[0],
            Decl::Def { name, .. } if name == "inferred_type"
        ));
    }

    #[test]
    fn solves_type_holes_from_typed_lambda_values() {
        let module = elaborate(
            r#"
import Std.Prelude
def inferred_lambda_type : _ := fun (A : Type) (x : A) => x
"#,
        )
        .expect("typed lambda should infer a declaration type hole");

        assert_eq!(module.declarations.len(), 1);
        assert!(matches!(
            &module.declarations[0],
            Decl::Def { name, .. } if name == "inferred_lambda_type"
        ));
    }

    #[test]
    fn accepts_defeq_type_applications_before_structural_decomposition() {
        let module = elaborate(
            r#"
import Std.Prelude
def non_injective_app_defeq : ((fun (ignored : Nat) => Type) Nat.zero) :=
  (Nat : ((fun (ignored : Nat) => Type) (Nat.succ Nat.zero)))
"#,
        )
        .expect("whole application defeq should run before argument decomposition");

        assert_eq!(module.declarations.len(), 1);
        assert!(matches!(
            &module.declarations[0],
            Decl::Def { name, .. } if name == "non_injective_app_defeq"
        ));
    }

    #[test]
    fn compares_pi_bodies_under_extended_constraint_context() {
        let module = elaborate(
            r#"
import Std.Prelude
def Alias (A : Type) : Type := A
def pi_body_context :
  (forall (A : Type), A -> Alias A) :=
  ((fun A x => x) : (forall (A : Type), A -> A))
"#,
        )
        .expect("Pi body defeq should use a context extended by the binder");

        assert_eq!(module.declarations.len(), 2);
        assert!(matches!(
            &module.declarations[1],
            Decl::Def { name, .. } if name == "pi_body_context"
        ));
    }

    #[test]
    fn rejects_named_hole_reuse_with_different_context_snapshot() {
        let err = elaborate("def bad : Type := let x : Type := ?m in ?m")
            .expect_err("same named hole in different contexts must fail");
        assert_eq!(err.kind, DiagnosticKind::NamedHoleContextMismatch);
    }

    #[test]
    fn canonical_import_names_do_not_match_internal_meta_names() {
        assert!(!expr_contains_term_meta(&Expr::konst(
            "__npa_meta.term.0",
            Vec::new()
        )));
        assert_eq!(universe_meta_id_from_name("__npa_meta.univ.0"), None);
        assert!(expr_contains_term_meta(&term_meta_expr(TermMetaId(0))));
    }

    #[test]
    fn rejects_unsolved_universe_metas_before_kernel_handoff() {
        let err = elaborate(
            r#"
axiom Box.{u} : Sort u
axiom X : Box
"#,
        )
        .expect_err("ambiguous universe argument must fail");
        assert_eq!(err.kind, DiagnosticKind::UnsolvedUniverseMeta);
    }

    #[test]
    fn solves_successor_universe_meta_equalities() {
        let module = elaborate(
            r#"
axiom Box.{u} : Type u
def T : Type := Box
"#,
        )
        .expect("successor universe equality should solve the inner metavariable");

        assert_eq!(module.declarations.len(), 2);
        assert!(matches!(
            &module.declarations[1],
            Decl::Def { name, .. } if name == "T"
        ));
    }

    #[test]
    fn preserves_imported_implicit_binder_metadata() {
        let module = elaborate_source(
            FileId(0),
            Name::from_dotted("Scratch"),
            r#"
import Std.Prelude
import CustomPoly
def use_imported : Nat := poly_id Nat.zero
"#,
            &[prelude_import(), custom_poly_import()],
        )
        .expect("imported implicit binder metadata should drive insertion");

        assert_eq!(module.declarations.len(), 1);
        assert!(matches!(
            &module.declarations[0],
            Decl::Def { name, .. } if name == "use_imported"
        ));
    }

    #[test]
    fn elaborates_imported_notation() {
        let module = elaborate_source(
            FileId(0),
            Name::from_dotted("Scratch"),
            r#"
import Std.Prelude
import CustomPoly
def use_imported_notation : Nat := ! Nat.zero
"#,
            &[prelude_import(), custom_poly_notation_import()],
        )
        .expect("imported top-level notation should be active while parsing terms");

        assert_eq!(module.declarations.len(), 1);
        assert!(matches!(
            &module.declarations[0],
            Decl::Def { name, .. } if name == "use_imported_notation"
        ));
    }

    #[test]
    fn preserves_imported_domain_implicit_binder_metadata() {
        let module = elaborate_source(
            FileId(0),
            Name::from_dotted("Scratch"),
            r#"
import Std.Prelude
import CustomHigherOrder
def use_imported_domain : Nat := k (fun f => f Nat.zero)
"#,
            &[prelude_import(), custom_higher_order_import()],
        )
        .expect("imported nested domain metadata should drive insertion");

        assert_eq!(module.declarations.len(), 1);
        assert!(matches!(
            &module.declarations[0],
            Decl::Def { name, .. } if name == "use_imported_domain"
        ));
    }

    #[test]
    fn preserves_imported_type_alias_value_metadata() {
        let module = elaborate_source(
            FileId(0),
            Name::from_dotted("Scratch"),
            r#"
import Std.Prelude
import CustomTypeAlias
axiom f : IdTy
def use_imported_alias : Nat := f Nat.zero
"#,
            &[prelude_import(), custom_type_alias_import()],
        )
        .expect("imported type alias value metadata should drive implicit insertion");

        assert_eq!(module.declarations.len(), 2);
        assert!(matches!(
            &module.declarations[1],
            Decl::Def { name, .. } if name == "use_imported_alias"
        ));
    }

    #[test]
    fn preserves_generated_import_implicit_binder_metadata() {
        let module = elaborate_source(
            FileId(0),
            Name::from_dotted("Scratch"),
            r#"
import Std.Prelude
import CustomBox
def boxed_zero : @Box.{1} Nat := Box.mk Nat.zero
"#,
            &[prelude_import(), custom_box_import()],
        )
        .expect("generated imported constructor metadata should drive insertion");

        assert_eq!(module.declarations.len(), 1);
        assert!(matches!(
            &module.declarations[0],
            Decl::Def { name, .. } if name == "boxed_zero"
        ));
    }

    #[test]
    fn preserves_inferred_lambda_implicit_binders_in_signatures() {
        let module = elaborate(
            r#"
import Std.Prelude
def inferred_id : _ := fun {A : Type} (x : A) => x
def use_inferred : Nat := inferred_id Nat.zero
"#,
        )
        .expect("inferred declaration signature should preserve implicit binder metadata");

        assert_eq!(module.declarations.len(), 2);
        assert!(matches!(
            &module.declarations[1],
            Decl::Def { name, .. } if name == "use_inferred"
        ));
    }

    #[test]
    fn preserves_implicit_binders_through_type_aliases() {
        let module = elaborate(
            r#"
import Std.Prelude
def IdTy : Type 1 := forall {A : Type}, A -> A
axiom f : IdTy
def use_alias : Nat := f Nat.zero
"#,
        )
        .expect("type alias metadata should drive implicit insertion");

        assert_eq!(module.declarations.len(), 3);
        assert!(matches!(
            &module.declarations[2],
            Decl::Def { name, .. } if name == "use_alias"
        ));
    }

    #[test]
    fn preserves_implicit_binders_through_parameterized_type_aliases() {
        let module = elaborate(
            r#"
import Std.Prelude
def IdTy (ignored : Type) : Type 1 := forall {A : Type}, A -> A
axiom f : IdTy Nat
def use_alias_param : Nat := f Nat.zero
"#,
        )
        .expect("parameterized type alias metadata should drive implicit insertion");

        assert_eq!(module.declarations.len(), 3);
        assert!(matches!(
            &module.declarations[2],
            Decl::Def { name, .. } if name == "use_alias_param"
        ));
    }

    #[test]
    fn preserves_implicit_binders_through_checked_lambda_type_aliases() {
        let module = elaborate(
            r#"
import Std.Prelude
def IdTy : Type -> Type 1 := fun (ignored : Type) => forall {A : Type}, A -> A
axiom f : IdTy Nat
def use_alias_lambda : Nat := f Nat.zero
"#,
        )
        .expect("checked lambda type alias metadata should drive implicit insertion");

        assert_eq!(module.declarations.len(), 3);
        assert!(matches!(
            &module.declarations[2],
            Decl::Def { name, .. } if name == "use_alias_lambda"
        ));
    }

    #[test]
    fn inserts_implicit_arguments_from_expected_type() {
        let module = elaborate(
            r#"
import Std.Prelude
axiom witness.{u} {A : Sort u} : A
def use_expected : Nat := witness
"#,
        )
        .expect("expected type should trigger implicit insertion");

        assert_eq!(module.declarations.len(), 2);
        assert!(matches!(
            &module.declarations[1],
            Decl::Def { name, .. } if name == "use_expected"
        ));
    }

    #[test]
    fn elaborates_infix_notation_and_erases_it_from_core() {
        let module = elaborate(
            r#"
import Std.Prelude
axiom plus (n m : Nat) : Nat
infixl:65 " + " => plus
infix:50 " = " => Eq
theorem plus_refl (n : Nat) : n + Nat.zero = n + Nat.zero :=
  Eq.refl (n + Nat.zero)
"#,
        )
        .expect("notation should elaborate through global targets");

        assert_eq!(module.declarations.len(), 2);
        let Decl::Theorem { ty, proof, .. } = &module.declarations[1] else {
            panic!("expected theorem");
        };
        assert!(!format!("{ty:?}").contains("Notation"));
        assert!(!format!("{proof:?}").contains("Notation"));
    }

    #[test]
    fn elaborates_prefix_and_postfix_notation() {
        let module = elaborate(
            r#"
import Std.Prelude
axiom id_nat (n : Nat) : Nat
prefix:70 "!" => id_nat
postfix:80 "$" => id_nat
def use_prefix_postfix : Nat := ! Nat.zero$
"#,
        )
        .expect("prefix and postfix notation should elaborate");

        assert_eq!(module.declarations.len(), 2);
        assert!(matches!(
            &module.declarations[1],
            Decl::Def { name, .. } if name == "use_prefix_postfix"
        ));
    }

    #[test]
    fn elaborates_notation_after_relative_open_inside_namespace() {
        let module = elaborate(
            r#"
import Std.Prelude
namespace A
namespace N
axiom id_nat (n : Nat) : Nat
prefix:70 "!" => id_nat
end N
end A
namespace A
open N
def use_relative_open : Nat := ! Nat.zero
end A
"#,
        )
        .expect("relative open should activate namespaced notation before parsing terms");

        assert_eq!(module.declarations.len(), 2);
        assert!(matches!(
            &module.declarations[1],
            Decl::Def { name, .. } if name == "A.use_relative_open"
        ));
    }

    #[test]
    fn elaborates_notation_after_open_through_opened_namespace() {
        let module = elaborate(
            r#"
import Std.Prelude
namespace A
namespace N
axiom id_nat (n : Nat) : Nat
prefix:70 "!" => id_nat
end N
end A
open A
open N
def use_opened_namespace : Nat := ! Nat.zero
"#,
        )
        .expect("open through an opened namespace should activate notation before parsing terms");

        assert_eq!(module.declarations.len(), 2);
        assert!(matches!(
            &module.declarations[1],
            Decl::Def { name, .. } if name == "use_opened_namespace"
        ));
    }

    #[test]
    fn resolves_overloaded_notation_by_expected_and_argument_types() {
        let module = elaborate(
            r#"
import Std.Prelude
axiom Foo : Type
axiom foo_zero : Foo
axiom plus_nat (n m : Nat) : Nat
axiom plus_foo (n m : Foo) : Foo
infixl:65 " + " => plus_nat
infixl:65 " + " => plus_foo
def use_nat_plus : Nat := Nat.zero + Nat.zero
"#,
        )
        .expect("argument types should select the Nat notation target");

        assert_eq!(module.declarations.len(), 5);
        assert!(matches!(
            &module.declarations[4],
            Decl::Def { name, .. } if name == "use_nat_plus"
        ));
    }

    #[test]
    fn ignores_overload_candidates_with_unsolved_fresh_implicits() {
        let module = elaborate(
            r#"
import Std.Prelude
axiom needs_unknown {A : Type} (n : Nat) : Nat
axiom plain (n : Nat) : Nat
prefix:70 "!" => needs_unknown
prefix:70 "!" => plain
def use_complete_candidate : Nat := ! Nat.zero
"#,
        )
        .expect("incomplete overload candidate should not make notation ambiguous");

        assert_eq!(module.declarations.len(), 3);
        assert!(matches!(
            &module.declarations[2],
            Decl::Def { name, .. } if name == "use_complete_candidate"
        ));
    }

    #[test]
    fn keeps_user_holes_after_selecting_notation_candidate() {
        let err = elaborate(
            r#"
import Std.Prelude
axiom plus (n m : Nat) : Nat
infixl:65 " + " => plus
def bad : Nat := Nat.zero + ?m
"#,
        )
        .expect_err("selected notation target should preserve user holes for final validation");
        assert_eq!(err.kind, DiagnosticKind::UnsolvedHole);
    }

    #[test]
    fn resolves_overloaded_applications_by_argument_types() {
        let module = elaborate(
            r#"
import Std.Prelude
namespace Nat
axiom add (n m : Nat) : Nat
end Nat
axiom Foo : Type
namespace Foo
axiom zero : Foo
axiom add (n m : Foo) : Foo
end Foo
open Nat
open Foo
def use_nat_add : Nat := add Nat.zero Nat.zero
"#,
        )
        .expect("argument types should select the Nat.add overload");

        assert_eq!(module.declarations.len(), 5);
        assert!(matches!(
            &module.declarations[4],
            Decl::Def { name, .. } if name == "use_nat_add"
        ));
    }

    #[test]
    fn resolves_overloaded_applications_by_expected_result_type() {
        let module = elaborate(
            r#"
import Std.Prelude
axiom Foo : Type
namespace Nat
axiom of (n : Nat) : Nat
end Nat
namespace Foo
axiom of (n : Nat) : Foo
end Foo
open Nat
open Foo
def use_expected_result : Nat := of Nat.zero
"#,
        )
        .expect("expected result type should select the Nat overload");

        assert_eq!(module.declarations.len(), 4);
        assert!(matches!(
            &module.declarations[3],
            Decl::Def { name, .. } if name == "use_expected_result"
        ));
    }

    #[test]
    fn preserves_at_mode_for_overloaded_applications() {
        let module = elaborate(
            r#"
import Std.Prelude
namespace Nat
axiom id {A : Type} (x : A) : A
end Nat
axiom Foo : Type
namespace Foo
axiom zero : Foo
axiom id (x : Foo) : Foo
end Foo
open Nat
open Foo
def use_explicit_overload : Nat := @id Nat Nat.zero
"#,
        )
        .expect("@ mode should be preserved while trying overloaded candidates");

        assert_eq!(module.declarations.len(), 5);
        assert!(matches!(
            &module.declarations[4],
            Decl::Def { name, .. } if name == "use_explicit_overload"
        ));
    }

    #[test]
    fn rejects_ambiguous_overloaded_notation() {
        let err = elaborate(
            r#"
import Std.Prelude
axiom plus1 (n m : Nat) : Nat
axiom plus2 (n m : Nat) : Nat
infixl:65 " + " => plus1
infixl:65 " + " => plus2
def bad (n : Nat) : Nat := n + n
"#,
        )
        .expect_err("two successful notation targets must be ambiguous");
        assert_eq!(err.kind, DiagnosticKind::AmbiguousNotation);
    }

    #[test]
    fn solves_metas_after_unfolding_reducible_type_heads() {
        let module = elaborate(
            r#"
import Std.Prelude
def Alias (A : Type) : Type := A
def alias_expected : Alias _ := Nat.zero
"#,
        )
        .expect("constraint solving should unfold reducible heads containing metas");

        assert_eq!(module.declarations.len(), 2);
        assert!(matches!(
            &module.declarations[1],
            Decl::Def { name, .. } if name == "alias_expected"
        ));
    }

    #[test]
    fn elaborates_grouped_binder_annotations_before_extending_context() {
        let module = elaborate(
            r#"
import Std.Prelude
def first (A : Type) (x y : A) : A := x
def checked (A : Type) : forall (x y : A), A := fun (x y : A) => x
def inferred (A : Type) : forall (x y : A), A := let g := fun (x y : A) => x in g
def grouped_hole_decl (x y : _) : Nat := x
def grouped_hole_pi : (forall (x y : _), Nat) := fun x y => x
"#,
        )
        .expect("grouped binders should elaborate");

        assert_eq!(module.declarations.len(), 5);
    }

    #[test]
    fn rejects_ill_typed_definition() {
        let err = elaborate(
            r#"
import Std.Prelude
def bad : Nat := Nat
"#,
        )
        .expect_err("ill-typed definition must fail");
        assert_eq!(err.kind, DiagnosticKind::TypeMismatch);
    }

    #[test]
    fn rejects_huge_numeric_universe_level_before_expansion() {
        let err = elaborate("axiom Huge : Sort 18446744073709551615")
            .expect_err("huge universe literals must fail before allocation");
        assert_eq!(err.kind, DiagnosticKind::UnsolvedUniverseMeta);
    }

    #[test]
    fn keeps_self_reference_out_of_the_global_env() {
        let err = elaborate("def self : Type := self")
            .expect_err("self reference must fail during resolution");
        assert_eq!(err.kind, DiagnosticKind::ForwardReference);
    }

    #[test]
    fn returns_kernel_rejected_when_kernel_rejects_declaration() {
        let err = elaborate("axiom Nat : Type").expect_err("kernel duplicate must fail");
        assert_eq!(err.kind, DiagnosticKind::KernelRejected);
    }

    #[test]
    fn loads_verified_imports_into_kernel_env() {
        let module = elaborate_source(
            FileId(0),
            Name::from_dotted("Scratch"),
            r#"
import Custom
axiom x : Foo
"#,
            &[custom_type_import()],
        )
        .expect("imported declarations should be available to kernel elaboration");

        assert_eq!(module.declarations.len(), 1);
        assert!(matches!(
            &module.declarations[0],
            Decl::Axiom { name, .. } if name == "x"
        ));
    }
}
