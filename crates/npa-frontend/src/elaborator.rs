use npa_kernel::{
    subst::{instantiate, shift},
    Ctx, Decl, Env, Error as KernelError, Expr, Level, Reducibility,
};

use crate::{
    parse_module, resolve_module, BinderInfo, Diagnostic, DiagnosticKind, FileId, Name,
    ResolvedBinder, ResolvedDecl, ResolvedExpr, ResolvedItem, ResolvedModule, ResolvedName, Result,
    Span, SurfaceBinderKind, SurfaceLevel, SurfaceModule, SurfaceUniverseParam, VerifiedImport,
};

const MAX_NUMERIC_UNIVERSE_LEVEL: u64 = 1024;

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
    let module = parse_module(file_id, source)?;
    elaborate_module(current_module, &module, verified_imports)
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
    declarations: Vec<Decl>,
    diagnostics: Vec<Diagnostic>,
}

impl ModuleElaborator {
    fn new(fallback_span: Span) -> Result<Self> {
        let env = Env::with_builtins().map_err(|error| kernel_rejected(fallback_span, error))?;
        Ok(Self {
            env,
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
                ResolvedItem::Inductive { span, .. } => {
                    return Err(Diagnostic::error(
                        DiagnosticKind::KernelRejected,
                        *span,
                        "inductive elaboration is implemented in a later milestone",
                    ));
                }
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
        let mut engine = ExprElaborator::new(self.env.clone(), universe_params.clone());
        let binders = engine.elaborate_decl_binders(&decl.binders)?;
        let result_ty = engine.elab_type(&decl.ty)?;
        let closed_ty = close_pi(result_ty.clone(), &binders);
        let name = decl.name.to_dotted();

        match kind {
            ValueDeclKind::Axiom => {
                self.env
                    .add_axiom(name.clone(), universe_params.clone(), closed_ty.clone())
                    .map_err(|error| kernel_rejected(decl.span, error))?;
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
                let body = engine.elab_check(value, &result_ty)?;
                let closed_value = close_lam(body, &binders);
                self.env
                    .add_def(
                        name.clone(),
                        universe_params.clone(),
                        closed_ty.clone(),
                        closed_value.clone(),
                        Reducibility::Reducible,
                    )
                    .map_err(|error| kernel_rejected(decl.span, error))?;
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
                let body = engine.elab_check(proof, &result_ty)?;
                let closed_proof = close_lam(body, &binders);
                self.env
                    .add_theorem(
                        name.clone(),
                        universe_params.clone(),
                        closed_ty.clone(),
                        closed_proof.clone(),
                    )
                    .map_err(|error| kernel_rejected(decl.span, error))?;
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
}

#[derive(Clone, Copy)]
enum ValueDeclKind {
    Def,
    Theorem,
    Axiom,
}

struct ExprElaborator {
    env: Env,
    ctx: Ctx,
    delta: Vec<String>,
}

impl ExprElaborator {
    fn new(env: Env, delta: Vec<String>) -> Self {
        Self {
            env,
            ctx: Ctx::new(),
            delta,
        }
    }

    fn elaborate_decl_binders(&mut self, binders: &[ResolvedBinder]) -> Result<Vec<CoreBinder>> {
        self.elaborate_typed_binder_groups(
            binders,
            DiagnosticKind::ExpectedSort,
            "declaration binder must have a type annotation",
        )
    }

    fn elab_type(&mut self, expr: &ResolvedExpr) -> Result<Expr> {
        let (core, ty) = self.elab_infer(expr)?;
        match self
            .env
            .whnf(&self.ctx, &self.delta, &ty)
            .map_err(|error| diagnostic_from_kernel_error(resolved_expr_span(expr), error))?
        {
            Expr::Sort(_) => Ok(core),
            actual => Err(Diagnostic::error(
                DiagnosticKind::ExpectedSort,
                resolved_expr_span(expr),
                format!("expected a type, found `{actual:?}`"),
            )),
        }
    }

    fn elab_infer(&mut self, expr: &ResolvedExpr) -> Result<(Expr, Expr)> {
        match expr {
            ResolvedExpr::Ident {
                resolved,
                universe_args,
                span,
                ..
            } => {
                let levels = self.elab_universe_args(resolved, universe_args.as_deref(), *span)?;
                let core = match resolved {
                    ResolvedName::Local(local) => {
                        if universe_args.is_some() {
                            return Err(Diagnostic::error(
                                DiagnosticKind::UnsolvedUniverseMeta,
                                *span,
                                "local names do not accept universe arguments",
                            ));
                        }
                        Expr::bvar(local.de_bruijn_index)
                    }
                    ResolvedName::Global(global) => Expr::konst(global_name(global), levels),
                    ResolvedName::Overloaded(_) => {
                        return Err(Diagnostic::error(
                            DiagnosticKind::AmbiguousName,
                            *span,
                            "overloaded name was not resolved by type information",
                        ));
                    }
                };
                let ty = self
                    .env
                    .infer(&self.ctx, &self.delta, &core)
                    .map_err(|error| diagnostic_from_kernel_error(*span, error))?;
                Ok((core, ty))
            }
            ResolvedExpr::Sort { level, span } => {
                let core = Expr::sort(self.elab_level(level)?);
                let ty = self
                    .env
                    .infer(&self.ctx, &self.delta, &core)
                    .map_err(|error| diagnostic_from_kernel_error(*span, error))?;
                Ok((core, ty))
            }
            ResolvedExpr::App { func, arg, span } => {
                let (func_core, func_ty) = self.elab_infer(func)?;
                let func_ty = self
                    .env
                    .whnf(&self.ctx, &self.delta, &func_ty)
                    .map_err(|error| diagnostic_from_kernel_error(*span, error))?;
                let Expr::Pi { ty, body, .. } = func_ty else {
                    return Err(Diagnostic::error(
                        DiagnosticKind::ExpectedFunctionType,
                        *span,
                        "expected function type in application",
                    ));
                };
                let arg_core = self.elab_check(arg, &ty)?;
                let result_ty = instantiate(&body, &arg_core)
                    .map_err(|error| diagnostic_from_kernel_error(*span, error))?;
                Ok((Expr::app(func_core, arg_core), result_ty))
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
                let core = self.elab_check(expr, &expected)?;
                Ok((core, expected))
            }
            ResolvedExpr::Hole { span, .. } => Err(Diagnostic::error(
                DiagnosticKind::UnsolvedHole,
                *span,
                "holes cannot be lowered to core terms in M3",
            )),
            ResolvedExpr::Notation { span, .. } => Err(Diagnostic::error(
                DiagnosticKind::AmbiguousNotation,
                *span,
                "notation elaboration is implemented in a later milestone",
            )),
        }
    }

    fn elab_check(&mut self, expr: &ResolvedExpr, expected: &Expr) -> Result<Expr> {
        if let ResolvedExpr::Lam {
            binders,
            body,
            span,
        } = expr
        {
            return self.elab_check_lam(binders, body, expected, *span);
        }

        let (core, actual) = self.elab_infer(expr)?;
        if self
            .env
            .is_defeq(&self.ctx, &self.delta, &actual, expected)
            .map_err(|error| diagnostic_from_kernel_error(resolved_expr_span(expr), error))?
        {
            Ok(core)
        } else {
            Err(Diagnostic::error(
                DiagnosticKind::TypeMismatch,
                resolved_expr_span(expr),
                format!("expected `{expected:?}`, found `{actual:?}`"),
            ))
        }
    }

    fn elab_infer_lam(
        &mut self,
        binders: &[ResolvedBinder],
        body: &ResolvedExpr,
        span: Span,
    ) -> Result<(Expr, Expr)> {
        let saved_ctx = self.ctx.clone();
        let core_binders = match self.elaborate_typed_binder_groups(
            binders,
            DiagnosticKind::ExpectedFunctionType,
            "lambda binder needs a type annotation in infer mode",
        ) {
            Ok(core_binders) => core_binders,
            Err(err) => {
                self.ctx = saved_ctx;
                return Err(err);
            }
        };
        let (body_core, body_ty) = self.elab_infer(body)?;
        self.ctx = saved_ctx;

        let core = close_lam(body_core, &core_binders);
        let ty = close_pi(body_ty, &core_binders);
        self.env
            .infer(&self.ctx, &self.delta, &core)
            .map_err(|error| diagnostic_from_kernel_error(span, error))?;
        Ok((core, ty))
    }

    fn elab_check_lam(
        &mut self,
        binders: &[ResolvedBinder],
        body: &ResolvedExpr,
        expected: &Expr,
        span: Span,
    ) -> Result<Expr> {
        let saved_ctx = self.ctx.clone();
        let mut expected_ty = expected.clone();
        let mut core_binders = Vec::new();
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
                    return Err(err);
                }
            };

            for (offset, binder) in binders[group_start..group_end].iter().enumerate() {
                let whnf = self
                    .env
                    .whnf(&self.ctx, &self.delta, &expected_ty)
                    .map_err(|error| diagnostic_from_kernel_error(span, error))?;
                let Expr::Pi { ty, body, .. } = whnf else {
                    self.ctx = saved_ctx;
                    return Err(Diagnostic::error(
                        DiagnosticKind::ExpectedFunctionType,
                        binder.span,
                        "lambda expects a function type",
                    ));
                };

                let binder_ty = if let Some(source_ty) = &group_source_tys[offset] {
                    let source_ty = weaken_group_type(source_ty, offset, binder.span)?;
                    if !self
                        .env
                        .is_defeq(&self.ctx, &self.delta, &source_ty, &ty)
                        .map_err(|error| diagnostic_from_kernel_error(binder.span, error))?
                    {
                        self.ctx = saved_ctx;
                        return Err(Diagnostic::error(
                            DiagnosticKind::TypeMismatch,
                            binder.span,
                            format!("lambda binder type `{source_ty:?}` does not match `{ty:?}`"),
                        ));
                    }
                    source_ty
                } else {
                    (*ty).clone()
                };

                let name = binder_name(binder);
                self.ctx.push_assumption(name.clone(), binder_ty.clone());
                core_binders.push(CoreBinder {
                    name,
                    ty: binder_ty,
                });
                expected_ty = (*body).clone();
            }

            index = group_end;
        }

        let body_core = self.elab_check(body, &expected_ty)?;
        self.ctx = saved_ctx;
        let core = close_lam(body_core, &core_binders);
        Ok(core)
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
            let mut group = Vec::new();
            for binder in &binders[group_start..group_end] {
                ensure_explicit_binder(binder)?;
                let Some(ty) = &binder.ty else {
                    return Err(Diagnostic::error(
                        missing_kind.clone(),
                        binder.span,
                        missing_message,
                    ));
                };
                group.push((binder_name(binder), self.elab_type(ty)?, binder.span));
            }

            for (offset, (name, binder_ty, span)) in group.into_iter().enumerate() {
                let binder_ty = weaken_group_type(&binder_ty, offset, span)?;
                self.ctx.push_assumption(name.clone(), binder_ty.clone());
                core_binders.push(CoreBinder {
                    name,
                    ty: binder_ty,
                });
            }

            index = group_end;
        }
        Ok(core_binders)
    }

    fn elaborate_optional_group_source_types(
        &mut self,
        binders: &[ResolvedBinder],
    ) -> Result<Vec<Option<Expr>>> {
        let mut source_tys = Vec::new();
        for binder in binders {
            ensure_explicit_binder(binder)?;
            source_tys.push(match &binder.ty {
                Some(source_ty) => Some(self.elab_type(source_ty)?),
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
    ) -> Result<(Expr, Expr)> {
        let saved_ctx = self.ctx.clone();
        let core_binders = match self.elaborate_typed_binder_groups(
            binders,
            DiagnosticKind::ExpectedSort,
            "Pi binder must have a type annotation",
        ) {
            Ok(core_binders) => core_binders,
            Err(err) => {
                self.ctx = saved_ctx;
                return Err(err);
            }
        };
        let body_ty = self.elab_type(body)?;
        self.ctx = saved_ctx;

        let core = close_pi(body_ty, &core_binders);
        let ty = self
            .env
            .infer(&self.ctx, &self.delta, &core)
            .map_err(|error| diagnostic_from_kernel_error(span, error))?;
        Ok((core, ty))
    }

    fn elab_infer_let(
        &mut self,
        name: String,
        ty: Option<&ResolvedExpr>,
        value: &ResolvedExpr,
        body: &ResolvedExpr,
        span: Span,
    ) -> Result<(Expr, Expr)> {
        let saved_ctx = self.ctx.clone();
        let (value_core, value_ty) = if let Some(ty) = ty {
            let ty_core = self.elab_type(ty)?;
            let value_core = self.elab_check(value, &ty_core)?;
            (value_core, ty_core)
        } else {
            self.elab_infer(value)?
        };

        self.ctx
            .push_definition(name.clone(), value_ty.clone(), value_core.clone());
        let (body_core, _) = self.elab_infer(body)?;
        self.ctx = saved_ctx;

        let core = Expr::let_in(name, value_ty, value_core, body_core);
        let ty = self
            .env
            .infer(&self.ctx, &self.delta, &core)
            .map_err(|error| diagnostic_from_kernel_error(span, error))?;
        Ok((core, ty))
    }

    fn elab_universe_args(
        &self,
        resolved: &ResolvedName,
        args: Option<&[SurfaceLevel]>,
        span: Span,
    ) -> Result<Vec<Level>> {
        let Some(args) = args else {
            if let ResolvedName::Global(global) = resolved {
                let name = global_name(global);
                if let Some(decl) = self.env.decl(&name) {
                    if !decl.universe_params().is_empty() {
                        return Err(Diagnostic::error(
                            DiagnosticKind::UnsolvedUniverseMeta,
                            span,
                            format!("missing universe arguments for `{name}`"),
                        ));
                    }
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
}

#[derive(Clone, Debug)]
struct CoreBinder {
    name: String,
    ty: Expr,
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

fn close_lam(body: Expr, binders: &[CoreBinder]) -> Expr {
    binders.iter().rev().fold(body, |body, binder| {
        Expr::lam(binder.name.clone(), binder.ty.clone(), body)
    })
}

fn ensure_explicit_binder(binder: &ResolvedBinder) -> Result<()> {
    if binder.binder_info == BinderInfo::Explicit {
        Ok(())
    } else {
        Err(Diagnostic::error(
            DiagnosticKind::UnsolvedImplicit,
            binder.span,
            "implicit binders are implemented in a later milestone",
        ))
    }
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
    use npa_kernel::{Decl, Expr, Level};

    use super::*;
    use crate::{ImportedDeclaration, Name};

    fn prelude_import() -> VerifiedImport {
        VerifiedImport {
            module: Name::from_dotted("Std.Prelude"),
            export_hash: "sha256:prelude".to_owned(),
            declarations: vec![
                ImportedDeclaration {
                    name: Name::from_dotted("Nat"),
                    decl_interface_hash: "sha256:Nat".to_owned(),
                },
                ImportedDeclaration {
                    name: Name::from_dotted("Nat.zero"),
                    decl_interface_hash: "sha256:Nat.zero".to_owned(),
                },
                ImportedDeclaration {
                    name: Name::from_dotted("Nat.succ"),
                    decl_interface_hash: "sha256:Nat.succ".to_owned(),
                },
                ImportedDeclaration {
                    name: Name::from_dotted("Eq"),
                    decl_interface_hash: "sha256:Eq".to_owned(),
                },
                ImportedDeclaration {
                    name: Name::from_dotted("Eq.refl"),
                    decl_interface_hash: "sha256:Eq.refl".to_owned(),
                },
            ],
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
            }],
            kernel_declarations: vec![Decl::Axiom {
                name: "Foo".to_owned(),
                universe_params: Vec::new(),
                ty: Expr::sort(Level::succ(Level::zero())),
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
theorem zero_refl : Eq.{1} Nat Nat.zero Nat.zero := Eq.refl.{1} Nat Nat.zero
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
    fn elaborates_grouped_binder_annotations_before_extending_context() {
        let module = elaborate(
            r#"
def first (A : Type) (x y : A) : A := x
def checked (A : Type) : forall (x y : A), A := fun (x y : A) => x
def inferred (A : Type) : forall (x y : A), A := let g := fun (x y : A) => x in g
"#,
        )
        .expect("grouped binders should elaborate");

        assert_eq!(module.declarations.len(), 3);
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
