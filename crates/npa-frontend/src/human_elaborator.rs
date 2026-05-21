use std::collections::BTreeMap;

use crate::{
    builtin_machine_callable_profile,
    elaborator::elaborate_machine_module,
    machine_callable_profile_from_human_binders, parse_human_module, resolve_human_module,
    resolver::{
        find_unique_verified_import_by_module, resolve_machine_module_with_options,
        VerifiedImportLookupError,
    },
    HumanBinder, HumanBinderKind, HumanCompileOptions, HumanDiagnostic, HumanDiagnosticKind,
    HumanExpr, HumanGlobalRef, HumanImplicitMode, HumanItem, HumanLevel, HumanResolvedName,
    HumanResolvedNameUse, HumanResolvedNotationUse, HumanResult, HumanSourceDeclarationMetadata,
    MachineBinder, MachineCallableBinderVisibility, MachineCompileOptions, MachineDecl,
    MachineItem, MachineLevel, MachineModule, MachineName, MachineTerm, ResolvedHumanModule, Span,
    VerifiedImport,
};
use npa_kernel::{Ctx, Decl, Env, Expr, Level, Reducibility};

const MAX_HUMAN_IMPLICIT_INSERTION_STEPS: usize = 64;

pub fn elaborate_human_module(
    module_name: npa_cert::ModuleName,
    module: ResolvedHumanModule,
    verified_imports: &[VerifiedImport],
    options: &HumanCompileOptions,
) -> HumanResult<npa_cert::CoreModule> {
    let span = module.module.span;
    let plans = notation_candidate_plans(&module, options.max_notation_candidates)?;
    let mut first_error = None;
    let mut success = None;

    for plan in plans {
        match elaborate_human_module_with_notation_plan(
            module_name.clone(),
            &module,
            verified_imports,
            &plan,
        ) {
            Ok(core) if success.is_none() => success = Some(core),
            Ok(_) => {
                return Err(HumanDiagnostic::error(
                    HumanDiagnosticKind::AmbiguousNotation,
                    span,
                    "multiple notation candidates elaborated successfully",
                ));
            }
            Err(err) => {
                first_error.get_or_insert(err);
            }
        }
    }

    if let Some(core) = success {
        Ok(core)
    } else if let Some(err) = first_error {
        Err(err)
    } else {
        Err(HumanDiagnostic::error(
            HumanDiagnosticKind::AmbiguousNotation,
            span,
            "no notation candidate plan was available",
        ))
    }
}

pub fn compile_human_source_to_core(
    file_id: crate::FileId,
    module_name: npa_cert::ModuleName,
    source: &str,
    verified_imports: &[VerifiedImport],
    options: &HumanCompileOptions,
) -> HumanResult<npa_cert::CoreModule> {
    let module = parse_human_module(file_id, source)?;
    let resolved = resolve_human_module(module_name.clone(), module, verified_imports, options)?;
    elaborate_human_module(module_name, resolved, verified_imports, options)
}

pub fn compile_human_source_to_certificate(
    file_id: crate::FileId,
    module_name: npa_cert::ModuleName,
    source: &str,
    verified_modules: &[npa_cert::VerifiedModule],
    options: &HumanCompileOptions,
) -> HumanResult<npa_cert::ModuleCert> {
    let verified_imports: Vec<_> = verified_modules.iter().map(VerifiedImport::from).collect();
    let _core =
        compile_human_source_to_core(file_id, module_name, source, &verified_imports, options)?;
    Err(HumanDiagnostic::not_implemented(
        source_span(file_id, source),
        "compile_human_source_to_certificate",
    ))
}

fn source_span(file_id: crate::FileId, source: &str) -> Span {
    Span::new(file_id, 0, source.len() as u32)
}

fn elaborate_human_module_with_notation_plan(
    module_name: npa_cert::ModuleName,
    module: &ResolvedHumanModule,
    verified_imports: &[VerifiedImport],
    notation_plan: &[usize],
) -> HumanResult<npa_cert::CoreModule> {
    let span = module.module.span;
    let mut lowering = HumanToMachineLowering::new(module, verified_imports, notation_plan)?;
    let machine_module = lowering.lower_module(module)?;
    let machine_options = MachineCompileOptions::default();
    let resolved =
        resolve_machine_module_with_options(machine_module, verified_imports, &machine_options)
            .map_err(machine_diagnostic_to_human)?;
    elaborate_machine_module(module_name, resolved, verified_imports, &machine_options)
        .map_err(machine_diagnostic_to_human)
        .map_err(|diagnostic| {
            if diagnostic.primary_span == Span::empty(crate::FileId(0)) {
                HumanDiagnostic::error(diagnostic.kind, span, diagnostic.message)
            } else {
                diagnostic
            }
        })
}

fn notation_candidate_plans(
    module: &ResolvedHumanModule,
    max_plans: usize,
) -> HumanResult<Vec<Vec<usize>>> {
    let mut plans = vec![Vec::new()];

    for notation in &module.resolved_notations {
        if notation.candidates.is_empty() {
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::AmbiguousNotation,
                notation.head.span,
                format!("notation {} has no candidates", notation.head.token),
            ));
        }
        let mut next = Vec::new();
        for plan in &plans {
            for candidate_index in 0..notation.candidates.len() {
                let mut plan = plan.clone();
                plan.push(candidate_index);
                next.push(plan);
                if next.len() > max_plans {
                    return Err(HumanDiagnostic::error(
                        HumanDiagnosticKind::TooManyNotationCandidates,
                        notation.head.span,
                        format!(
                            "notation {} exceeds the bounded elaboration candidate budget",
                            notation.head.token
                        ),
                    ));
                }
            }
        }
        plans = next;
    }

    Ok(plans)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HumanLoweredDeclKind {
    Def,
    Theorem,
}

#[derive(Clone, Debug)]
struct HumanCallableSignature {
    universe_params: Vec<String>,
    implicit_profile: Vec<MachineCallableBinderVisibility>,
}

#[derive(Clone, Debug)]
struct SyntheticImplicitMeta {
    value: Expr,
}

#[derive(Clone, Debug)]
struct HumanElaboratedBinder {
    name: String,
    ty: Expr,
}

#[derive(Clone, Debug)]
struct HumanLocalDecl {
    name: String,
    ty: Expr,
    value: Option<Expr>,
}

#[derive(Clone, Debug, Default)]
struct HumanLocalContext {
    locals: Vec<HumanLocalDecl>,
}

impl HumanLocalContext {
    fn push_assumption(&mut self, name: String, ty: Expr) {
        self.locals.push(HumanLocalDecl {
            name,
            ty,
            value: None,
        });
    }

    fn push_definition(&mut self, name: String, ty: Expr, value: Expr) {
        self.locals.push(HumanLocalDecl {
            name,
            ty,
            value: Some(value),
        });
    }

    fn lookup_bvar(&self, name: &str) -> Option<u32> {
        self.locals
            .iter()
            .rev()
            .position(|local| local.name == name)
            .map(|index| index as u32)
    }

    fn name_for_bvar(&self, index: u32) -> Option<&str> {
        let index = usize::try_from(index).ok()?;
        self.locals
            .len()
            .checked_sub(index + 1)
            .and_then(|local_index| self.locals.get(local_index))
            .map(|local| local.name.as_str())
    }

    fn to_kernel_ctx(&self) -> Ctx {
        let mut ctx = Ctx::new();
        for local in &self.locals {
            match &local.value {
                Some(value) => {
                    ctx.push_definition(local.name.clone(), local.ty.clone(), value.clone())
                }
                None => ctx.push_assumption(local.name.clone(), local.ty.clone()),
            }
        }
        ctx
    }
}

struct HumanImplicitInserter {
    env: Env,
    signatures: BTreeMap<String, HumanCallableSignature>,
    insertion_steps: usize,
}

impl HumanImplicitInserter {
    fn new(module: &ResolvedHumanModule, verified_imports: &[VerifiedImport]) -> HumanResult<Self> {
        let mut inserter = Self {
            env: Env::new(),
            signatures: BTreeMap::new(),
            insertion_steps: 0,
        };

        let active_imports = active_human_imports(module, verified_imports)?;
        for import in active_imports {
            inserter.add_import(import, module.module.span)?;
        }

        Ok(inserter)
    }

    fn add_import(&mut self, import: &VerifiedImport, span: Span) -> HumanResult<()> {
        for decl in kernel_decls_for_human_import(import) {
            self.add_kernel_decl(decl, span)?;
        }
        for export in &import.exports {
            let implicit_profile = if npa_cert::builtin_decl_interface_hash(&export.name)
                == Some(export.decl_interface_hash)
            {
                builtin_machine_callable_profile(&export.name).unwrap_or_default()
            } else {
                Vec::new()
            };
            self.signatures.insert(
                export.name.as_dotted(),
                HumanCallableSignature {
                    universe_params: export.universe_params.clone(),
                    implicit_profile,
                },
            );
        }
        Ok(())
    }

    fn insert_decl(
        &mut self,
        mut decl: MachineDecl,
        metadata: &HumanSourceDeclarationMetadata,
        kind: HumanLoweredDeclKind,
    ) -> HumanResult<MachineDecl> {
        let delta: Vec<_> = decl
            .universe_params
            .iter()
            .map(|param| param.name.clone())
            .collect();
        let mut locals = HumanLocalContext::default();
        let mut elaborated_binders = Vec::with_capacity(decl.binders.len());
        let mut transformed_binders = Vec::with_capacity(decl.binders.len());

        for binder in decl.binders {
            let ty = self.insert_term(binder.ty, &mut locals, &delta)?;
            let ty_expr = self.elaborate_machine_term(&ty, &locals, &delta)?;
            locals.push_assumption(binder.name.clone(), ty_expr.clone());
            elaborated_binders.push(HumanElaboratedBinder {
                name: binder.name.clone(),
                ty: ty_expr,
            });
            transformed_binders.push(MachineBinder {
                name: binder.name,
                ty,
                span: binder.span,
            });
        }

        decl.binders = transformed_binders;
        decl.ty = self.insert_term(decl.ty, &mut locals, &delta)?;
        decl.value = self.insert_term(decl.value, &mut locals, &delta)?;

        let ty_expr = self.elaborate_machine_term(&decl.ty, &locals, &delta)?;
        let value_expr = self.elaborate_machine_term(&decl.value, &locals, &delta)?;
        let closed_ty = human_close_pi(&elaborated_binders, ty_expr);
        let closed_value = human_close_lam(&elaborated_binders, value_expr);
        let name = decl.name.as_dotted();
        let universe_params = delta.clone();
        let core_decl = match kind {
            HumanLoweredDeclKind::Def => Decl::Def {
                name: name.clone(),
                universe_params,
                ty: closed_ty,
                value: closed_value,
                reducibility: Reducibility::Reducible,
            },
            HumanLoweredDeclKind::Theorem => Decl::Theorem {
                name: name.clone(),
                universe_params,
                ty: closed_ty,
                proof: closed_value,
            },
        };
        self.add_kernel_decl(core_decl, decl.span)?;
        self.signatures.insert(
            name,
            HumanCallableSignature {
                universe_params: delta,
                implicit_profile: machine_callable_profile_from_human_binders(&metadata.binders),
            },
        );

        Ok(decl)
    }

    fn insert_term(
        &mut self,
        term: MachineTerm,
        locals: &mut HumanLocalContext,
        delta: &[String],
    ) -> HumanResult<MachineTerm> {
        match term {
            MachineTerm::App { .. } => {
                let (head, args, span) = collect_machine_app_spine(term);
                let head = self.insert_app_head(head, locals, delta)?;
                let args = args
                    .into_iter()
                    .map(|arg| self.insert_term(arg, locals, delta))
                    .collect::<HumanResult<Vec<_>>>()?;
                self.insert_implicit_spine(head, args, span, locals, delta)
            }
            MachineTerm::Lam {
                binders,
                body,
                span,
            } => {
                let mut nested_locals = locals.clone();
                let binders = self.insert_binders(binders, &mut nested_locals, delta)?;
                Ok(MachineTerm::Lam {
                    binders,
                    body: Box::new(self.insert_term(*body, &mut nested_locals, delta)?),
                    span,
                })
            }
            MachineTerm::Pi {
                binders,
                body,
                span,
            } => {
                let mut nested_locals = locals.clone();
                let binders = self.insert_binders(binders, &mut nested_locals, delta)?;
                Ok(MachineTerm::Pi {
                    binders,
                    body: Box::new(self.insert_term(*body, &mut nested_locals, delta)?),
                    span,
                })
            }
            MachineTerm::Let {
                name,
                ty,
                value,
                body,
                span,
            } => {
                let ty = self.insert_term(*ty, locals, delta)?;
                let value = self.insert_term(*value, locals, delta)?;
                let ty_expr = self.elaborate_machine_term(&ty, locals, delta)?;
                let value_expr = self.elaborate_machine_term(&value, locals, delta)?;
                let mut nested_locals = locals.clone();
                nested_locals.push_definition(name.clone(), ty_expr, value_expr);
                Ok(MachineTerm::Let {
                    name,
                    ty: Box::new(ty),
                    value: Box::new(value),
                    body: Box::new(self.insert_term(*body, &mut nested_locals, delta)?),
                    span,
                })
            }
            MachineTerm::Annot { expr, ty, span } => Ok(MachineTerm::Annot {
                expr: Box::new(self.insert_term(*expr, locals, delta)?),
                ty: Box::new(self.insert_term(*ty, locals, delta)?),
                span,
            }),
            MachineTerm::Ident {
                name,
                universe_args,
                explicit_mode: false,
                span,
            } if self
                .signatures
                .get(&name.as_dotted())
                .is_some_and(|signature| {
                    signature
                        .implicit_profile
                        .contains(&MachineCallableBinderVisibility::Implicit)
                }) =>
            {
                let _ = universe_args;
                Err(self.unsolved_implicit(
                    span,
                    format!(
                        "global name {} still has unresolved implicit arguments",
                        name.as_dotted()
                    ),
                ))
            }
            term => Ok(term),
        }
    }

    fn insert_app_head(
        &mut self,
        term: MachineTerm,
        locals: &mut HumanLocalContext,
        delta: &[String],
    ) -> HumanResult<MachineTerm> {
        match term {
            MachineTerm::Ident { .. } => Ok(term),
            term => self.insert_term(term, locals, delta),
        }
    }

    fn insert_binders(
        &mut self,
        binders: Vec<MachineBinder>,
        locals: &mut HumanLocalContext,
        delta: &[String],
    ) -> HumanResult<Vec<MachineBinder>> {
        binders
            .into_iter()
            .map(|binder| {
                let ty = self.insert_term(binder.ty, locals, delta)?;
                let ty_expr = self.elaborate_machine_term(&ty, locals, delta)?;
                locals.push_assumption(binder.name.clone(), ty_expr);
                Ok(MachineBinder {
                    name: binder.name,
                    ty,
                    span: binder.span,
                })
            })
            .collect()
    }

    fn insert_implicit_spine(
        &mut self,
        head: MachineTerm,
        args: Vec<MachineTerm>,
        span: Span,
        locals: &HumanLocalContext,
        delta: &[String],
    ) -> HumanResult<MachineTerm> {
        let MachineTerm::Ident {
            name,
            universe_args,
            explicit_mode,
            span: head_span,
        } = head
        else {
            return Ok(rebuild_machine_apps(head, args, span));
        };
        if explicit_mode {
            return Ok(rebuild_machine_apps(
                MachineTerm::Ident {
                    name,
                    universe_args,
                    explicit_mode,
                    span: head_span,
                },
                args,
                span,
            ));
        }

        let Some(signature) = self.signatures.get(&name.as_dotted()).cloned() else {
            return Ok(rebuild_machine_apps(
                MachineTerm::Ident {
                    name,
                    universe_args,
                    explicit_mode,
                    span: head_span,
                },
                args,
                span,
            ));
        };
        if !signature
            .implicit_profile
            .contains(&MachineCallableBinderVisibility::Implicit)
        {
            return Ok(rebuild_machine_apps(
                MachineTerm::Ident {
                    name,
                    universe_args,
                    explicit_mode,
                    span: head_span,
                },
                args,
                span,
            ));
        }

        let mut args = args.into_iter().peekable();
        let mut expanded_args = Vec::new();
        let mut synthetic_implicits = Vec::new();
        for visibility in &signature.implicit_profile {
            match visibility {
                MachineCallableBinderVisibility::Explicit => {
                    let Some(arg) = args.next() else {
                        break;
                    };
                    expanded_args.push(arg);
                }
                MachineCallableBinderVisibility::Implicit => {
                    let Some(next_explicit_arg) = args.peek() else {
                        return Err(self.unsolved_implicit(
                            head_span,
                            format!(
                                "cannot infer implicit argument for {} without a supplied explicit argument",
                                name.as_dotted()
                            ),
                        ));
                    };
                    self.bump_insertion_step(head_span)?;
                    let inferred_type =
                        self.infer_machine_term_type(next_explicit_arg, locals, delta)?;
                    let inserted = core_expr_to_machine_term(&inferred_type, locals, head_span)
                        .ok_or_else(|| {
                            self.unsolved_implicit(
                                head_span,
                                format!(
                                    "cannot materialize inferred implicit argument for {}",
                                    name.as_dotted()
                                ),
                            )
                        })?;
                    synthetic_implicits.push(SyntheticImplicitMeta {
                        value: inferred_type,
                    });
                    expanded_args.push(inserted);
                }
            }
        }
        expanded_args.extend(args);

        let universe_args = match universe_args {
            Some(args) => Some(args),
            None if signature.universe_params.is_empty() => None,
            None => Some(self.infer_universe_args(
                &signature,
                &synthetic_implicits,
                locals,
                delta,
                head_span,
                &name,
            )?),
        };
        let head = MachineTerm::Ident {
            name,
            universe_args,
            explicit_mode: true,
            span: head_span,
        };
        Ok(rebuild_machine_apps(head, expanded_args, span))
    }

    fn infer_universe_args(
        &self,
        signature: &HumanCallableSignature,
        synthetic_implicits: &[SyntheticImplicitMeta],
        locals: &HumanLocalContext,
        delta: &[String],
        span: Span,
        name: &MachineName,
    ) -> HumanResult<Vec<MachineLevel>> {
        let mut levels = Vec::with_capacity(signature.universe_params.len());
        for synthetic in synthetic_implicits {
            if levels.len() == signature.universe_params.len() {
                break;
            }
            let inferred = self.infer_core_expr_type(&synthetic.value, locals, delta, span)?;
            let Expr::Sort(level) = inferred else {
                return Err(self.unsolved_implicit(
                    span,
                    format!(
                        "inferred implicit argument for {} is not a type",
                        name.as_dotted()
                    ),
                ));
            };
            levels.push(core_level_to_machine_level(&level, span));
        }
        if levels.len() != signature.universe_params.len() {
            return Err(self.unsolved_implicit(
                span,
                format!("cannot infer universe arguments for {}", name.as_dotted()),
            ));
        }
        Ok(levels)
    }

    fn elaborate_machine_term(
        &self,
        term: &MachineTerm,
        locals: &HumanLocalContext,
        delta: &[String],
    ) -> HumanResult<Expr> {
        let _universe_context_len = delta.len();
        Ok(match term {
            MachineTerm::Ident {
                name,
                universe_args,
                span,
                ..
            } => {
                let name = name.as_dotted();
                let expected = self
                    .env
                    .decl(&name)
                    .map(|decl| decl.universe_params().len())
                    .unwrap_or(0);
                let levels = match universe_args {
                    Some(args) if args.len() == expected => args
                        .iter()
                        .cloned()
                        .map(elaborate_machine_level)
                        .collect::<HumanResult<Vec<_>>>()?,
                    Some(args) => {
                        return Err(self.unsolved_implicit(
                            *span,
                            format!(
                                "global name {name} expects {expected} universe arguments, got {}",
                                args.len()
                            ),
                        ));
                    }
                    None if expected == 0 => Vec::new(),
                    None => {
                        return Err(self.unsolved_implicit(
                            *span,
                            format!("global name {name} still has unresolved universe arguments"),
                        ));
                    }
                };
                Expr::konst(name, levels)
            }
            MachineTerm::Local { name, span } => {
                locals.lookup_bvar(name).map(Expr::bvar).ok_or_else(|| {
                    HumanDiagnostic::error(
                        HumanDiagnosticKind::MachineElaborationError,
                        *span,
                        format!("unknown local name {name}"),
                    )
                })?
            }
            MachineTerm::Prop { .. } => Expr::sort(Level::zero()),
            MachineTerm::Type { level, .. } => {
                Expr::sort(Level::succ(elaborate_machine_level(level.clone())?))
            }
            MachineTerm::Sort { level, .. } => Expr::sort(elaborate_machine_level(level.clone())?),
            MachineTerm::App { func, arg, .. } => Expr::app(
                self.elaborate_machine_term(func, locals, delta)?,
                self.elaborate_machine_term(arg, locals, delta)?,
            ),
            MachineTerm::Lam { binders, body, .. } => {
                let mut nested = locals.clone();
                let mut elaborated_binders = Vec::with_capacity(binders.len());
                for binder in binders {
                    let ty = self.elaborate_machine_term(&binder.ty, &nested, delta)?;
                    nested.push_assumption(binder.name.clone(), ty.clone());
                    elaborated_binders.push(HumanElaboratedBinder {
                        name: binder.name.clone(),
                        ty,
                    });
                }
                let body = self.elaborate_machine_term(body, &nested, delta)?;
                human_close_lam(&elaborated_binders, body)
            }
            MachineTerm::Pi { binders, body, .. } => {
                let mut nested = locals.clone();
                let mut elaborated_binders = Vec::with_capacity(binders.len());
                for binder in binders {
                    let ty = self.elaborate_machine_term(&binder.ty, &nested, delta)?;
                    nested.push_assumption(binder.name.clone(), ty.clone());
                    elaborated_binders.push(HumanElaboratedBinder {
                        name: binder.name.clone(),
                        ty,
                    });
                }
                let body = self.elaborate_machine_term(body, &nested, delta)?;
                human_close_pi(&elaborated_binders, body)
            }
            MachineTerm::Let {
                name,
                ty,
                value,
                body,
                ..
            } => {
                let ty = self.elaborate_machine_term(ty, locals, delta)?;
                let value = self.elaborate_machine_term(value, locals, delta)?;
                let mut nested = locals.clone();
                nested.push_definition(name.clone(), ty, value);
                self.elaborate_machine_term(body, &nested, delta)?
            }
            MachineTerm::Annot { expr, .. } => self.elaborate_machine_term(expr, locals, delta)?,
        })
    }

    fn infer_machine_term_type(
        &self,
        term: &MachineTerm,
        locals: &HumanLocalContext,
        delta: &[String],
    ) -> HumanResult<Expr> {
        let expr = self.elaborate_machine_term(term, locals, delta)?;
        self.infer_core_expr_type(&expr, locals, delta, term.span())
    }

    fn infer_core_expr_type(
        &self,
        expr: &Expr,
        locals: &HumanLocalContext,
        delta: &[String],
        span: Span,
    ) -> HumanResult<Expr> {
        self.env
            .infer(&locals.to_kernel_ctx(), delta, expr)
            .map_err(|err| {
                HumanDiagnostic::error(
                    HumanDiagnosticKind::MachineElaborationError,
                    span,
                    format!("kernel rejected Human implicit inference: {err:?}"),
                )
            })
    }

    fn add_kernel_decl(&mut self, decl: Decl, span: Span) -> HumanResult<()> {
        if let Some(existing) = self.env.decl(decl.name()) {
            if existing == &decl {
                return Ok(());
            }
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::MachineElaborationError,
                span,
                format!(
                    "kernel declaration {} conflicts with an existing declaration",
                    decl.name()
                ),
            ));
        }

        match decl {
            Decl::Axiom {
                name,
                universe_params,
                ty,
            } => self.env.add_axiom(name, universe_params, ty),
            Decl::Def {
                name,
                universe_params,
                ty,
                value,
                reducibility,
            } => self
                .env
                .add_def(name, universe_params, ty, value, reducibility),
            Decl::Theorem {
                name,
                universe_params,
                ty,
                proof,
            } => self.env.add_theorem(name, universe_params, ty, proof),
            Decl::Inductive { data, .. } => self.env.add_inductive(*data),
            Decl::Constructor { .. } | Decl::Recursor { .. } => Ok(()),
        }
        .map_err(|err| {
            HumanDiagnostic::error(
                HumanDiagnosticKind::MachineElaborationError,
                span,
                format!("kernel rejected Human implicit environment: {err:?}"),
            )
        })
    }

    fn bump_insertion_step(&mut self, span: Span) -> HumanResult<()> {
        self.insertion_steps += 1;
        if self.insertion_steps > MAX_HUMAN_IMPLICIT_INSERTION_STEPS {
            return Err(self.unsolved_implicit(
                span,
                "Human implicit insertion exceeded the bounded search limit".to_owned(),
            ));
        }
        Ok(())
    }

    fn unsolved_implicit(&self, span: Span, message: String) -> HumanDiagnostic {
        HumanDiagnostic::error(HumanDiagnosticKind::UnsolvedImplicit, span, message)
    }
}

fn active_human_imports<'a>(
    module: &ResolvedHumanModule,
    verified_imports: &'a [VerifiedImport],
) -> HumanResult<Vec<&'a VerifiedImport>> {
    let mut imports = Vec::new();
    for item in &module.module.items {
        let HumanItem::Import {
            module: import_name,
            span,
        } = item
        else {
            continue;
        };
        let import_module = npa_cert::Name(import_name.parts.clone());
        match find_unique_verified_import_by_module(verified_imports, &import_module) {
            Ok(import) => imports.push(import),
            Err(VerifiedImportLookupError::Missing) => {
                return Err(HumanDiagnostic::error(
                    HumanDiagnosticKind::MissingVerifiedImport,
                    *span,
                    format!(
                        "missing verified import for module {}",
                        import_name.as_dotted()
                    ),
                ));
            }
            Err(VerifiedImportLookupError::Ambiguous) => {
                return Err(HumanDiagnostic::error(
                    HumanDiagnosticKind::AmbiguousName,
                    *span,
                    format!(
                        "ambiguous verified import for module {}",
                        import_name.as_dotted()
                    ),
                ));
            }
        }
    }
    Ok(imports)
}

fn kernel_decls_for_human_import(import: &VerifiedImport) -> Vec<Decl> {
    if !import.kernel_decls.is_empty() {
        return import.kernel_decls.clone();
    }

    import
        .exports
        .iter()
        .map(|export| Decl::Axiom {
            name: export.name.as_dotted(),
            universe_params: export.universe_params.clone(),
            ty: export.ty.clone(),
        })
        .collect()
}

fn collect_machine_app_spine(term: MachineTerm) -> (MachineTerm, Vec<MachineTerm>, Span) {
    let span = term.span();
    let mut args = Vec::new();
    let mut head = term;
    while let MachineTerm::App { func, arg, .. } = head {
        args.push(*arg);
        head = *func;
    }
    args.reverse();
    (head, args, span)
}

fn rebuild_machine_apps(head: MachineTerm, args: Vec<MachineTerm>, span: Span) -> MachineTerm {
    let mut term = head;
    for arg in args {
        let app_span = term.span().join(arg.span());
        term = MachineTerm::App {
            func: Box::new(term),
            arg: Box::new(arg),
            span: app_span,
        };
    }
    if matches!(term, MachineTerm::App { .. }) {
        term
    } else {
        let _ = span;
        term
    }
}

fn human_close_lam(binders: &[HumanElaboratedBinder], mut body: Expr) -> Expr {
    for binder in binders.iter().rev() {
        body = Expr::lam(binder.name.clone(), binder.ty.clone(), body);
    }
    body
}

fn human_close_pi(binders: &[HumanElaboratedBinder], mut body: Expr) -> Expr {
    for binder in binders.iter().rev() {
        body = Expr::pi(binder.name.clone(), binder.ty.clone(), body);
    }
    body
}

fn elaborate_machine_level(level: MachineLevel) -> HumanResult<Level> {
    Ok(match level {
        MachineLevel::Nat { value, .. } => level_from_nat(value),
        MachineLevel::Param { name, .. } => Level::param(name),
        MachineLevel::Succ { level, .. } => Level::succ(elaborate_machine_level(*level)?),
        MachineLevel::Max { lhs, rhs, .. } => Level::max(
            elaborate_machine_level(*lhs)?,
            elaborate_machine_level(*rhs)?,
        ),
        MachineLevel::IMax { lhs, rhs, .. } => Level::imax(
            elaborate_machine_level(*lhs)?,
            elaborate_machine_level(*rhs)?,
        ),
    })
}

fn level_from_nat(value: u64) -> Level {
    let mut level = Level::zero();
    for _ in 0..value {
        level = Level::succ(level);
    }
    level
}

fn core_expr_to_machine_term(
    expr: &Expr,
    locals: &HumanLocalContext,
    span: Span,
) -> Option<MachineTerm> {
    match expr {
        Expr::Sort(level) => Some(MachineTerm::Sort {
            level: core_level_to_machine_level(level, span),
            span,
        }),
        Expr::BVar(index) => Some(MachineTerm::Local {
            name: locals.name_for_bvar(*index)?.to_owned(),
            span,
        }),
        Expr::Const { name, levels } => Some(MachineTerm::Ident {
            name: MachineName {
                parts: name.split('.').map(str::to_owned).collect(),
                span,
            },
            universe_args: (!levels.is_empty()).then(|| {
                levels
                    .iter()
                    .map(|level| core_level_to_machine_level(level, span))
                    .collect()
            }),
            explicit_mode: !levels.is_empty(),
            span,
        }),
        Expr::App(_, _) => {
            let (head, args) = npa_kernel::expr::collect_apps(expr);
            let head = core_expr_to_machine_term(&head, locals, span)?;
            let args = args
                .iter()
                .map(|arg| core_expr_to_machine_term(arg, locals, span))
                .collect::<Option<Vec<_>>>()?;
            Some(rebuild_machine_apps(head, args, span))
        }
        Expr::Lam { .. } | Expr::Pi { .. } | Expr::Let { .. } => None,
    }
}

fn core_level_to_machine_level(level: &Level, span: Span) -> MachineLevel {
    if let Some(value) = core_level_as_u64(level) {
        return MachineLevel::Nat { value, span };
    }

    match level {
        Level::Zero => MachineLevel::Nat { value: 0, span },
        Level::Param(name) => MachineLevel::Param {
            name: name.clone(),
            span,
        },
        Level::Succ(level) => MachineLevel::Succ {
            level: Box::new(core_level_to_machine_level(level, span)),
            span,
        },
        Level::Max(lhs, rhs) => MachineLevel::Max {
            lhs: Box::new(core_level_to_machine_level(lhs, span)),
            rhs: Box::new(core_level_to_machine_level(rhs, span)),
            span,
        },
        Level::IMax(lhs, rhs) => MachineLevel::IMax {
            lhs: Box::new(core_level_to_machine_level(lhs, span)),
            rhs: Box::new(core_level_to_machine_level(rhs, span)),
            span,
        },
    }
}

fn core_level_as_u64(level: &Level) -> Option<u64> {
    match npa_kernel::level::normalize_level(level.clone()) {
        Level::Zero => Some(0),
        Level::Succ(level) => Some(core_level_as_u64(&level)? + 1),
        _ => None,
    }
}

struct HumanToMachineLowering<'a> {
    name_uses: std::slice::Iter<'a, HumanResolvedNameUse>,
    notation_uses: std::slice::Iter<'a, HumanResolvedNotationUse>,
    notation_choices: std::slice::Iter<'a, usize>,
    implicit_inserter: HumanImplicitInserter,
}

impl<'a> HumanToMachineLowering<'a> {
    fn new(
        module: &'a ResolvedHumanModule,
        verified_imports: &[VerifiedImport],
        notation_plan: &'a [usize],
    ) -> HumanResult<Self> {
        Ok(Self {
            name_uses: module.resolved_names.iter(),
            notation_uses: module.resolved_notations.iter(),
            notation_choices: notation_plan.iter(),
            implicit_inserter: HumanImplicitInserter::new(module, verified_imports)?,
        })
    }

    fn lower_module(&mut self, module: &ResolvedHumanModule) -> HumanResult<MachineModule> {
        let mut machine_items = Vec::new();
        let mut declarations = module.state.source_interfaces.current.declarations.iter();

        for item in &module.module.items {
            match item {
                HumanItem::Import { module, span } => {
                    machine_items.push(MachineItem::Import {
                        module: machine_name(module.clone()),
                        span: *span,
                    });
                }
                HumanItem::Def(decl) => {
                    let metadata = declarations.next().ok_or_else(|| {
                        HumanDiagnostic::not_implemented(decl.span, "Human declaration metadata")
                    })?;
                    let lowered = self.lower_decl(decl.clone(), metadata)?;
                    let lowered = self.implicit_inserter.insert_decl(
                        lowered,
                        metadata,
                        HumanLoweredDeclKind::Def,
                    )?;
                    machine_items.push(MachineItem::Def(lowered));
                }
                HumanItem::Theorem(decl) => {
                    let metadata = declarations.next().ok_or_else(|| {
                        HumanDiagnostic::not_implemented(decl.span, "Human declaration metadata")
                    })?;
                    let lowered = self.lower_decl(decl.clone(), metadata)?;
                    let lowered = self.implicit_inserter.insert_decl(
                        lowered,
                        metadata,
                        HumanLoweredDeclKind::Theorem,
                    )?;
                    machine_items.push(MachineItem::Theorem(lowered));
                }
                HumanItem::Axiom(decl) => {
                    return Err(HumanDiagnostic::not_implemented(
                        decl.span,
                        "Human source-level axiom elaboration",
                    ));
                }
                HumanItem::Inductive(decl) => {
                    return Err(HumanDiagnostic::not_implemented(
                        decl.span,
                        "Human source-level inductive elaboration",
                    ));
                }
                HumanItem::Open { .. }
                | HumanItem::NamespaceStart { .. }
                | HumanItem::NamespaceEnd { .. }
                | HumanItem::Notation(_) => {}
            }
        }

        Ok(MachineModule {
            file_id: module.module.file_id,
            items: machine_items,
            span: module.module.span,
        })
    }

    fn lower_decl(
        &mut self,
        decl: crate::HumanDecl,
        metadata: &HumanSourceDeclarationMetadata,
    ) -> HumanResult<MachineDecl> {
        Ok(MachineDecl {
            name: machine_name(metadata.name.clone()),
            universe_params: decl
                .universe_params
                .into_iter()
                .map(|param| crate::MachineUniverseParam {
                    name: param.name,
                    span: param.span,
                })
                .collect(),
            binders: self.lower_binders(decl.binders)?,
            ty: self.lower_expr(decl.ty)?,
            value: self.lower_expr(decl.value)?,
            span: decl.span,
        })
    }

    fn lower_binders(&mut self, binders: Vec<HumanBinder>) -> HumanResult<Vec<MachineBinder>> {
        binders
            .into_iter()
            .map(|binder| {
                let HumanBinderKind::Named(name) = binder.kind else {
                    return Err(HumanDiagnostic::not_implemented(
                        binder.span,
                        "anonymous Human binder lowering",
                    ));
                };
                let Some(ty) = binder.ty else {
                    return Err(HumanDiagnostic::not_implemented(
                        binder.span,
                        "unannotated Human binder lowering",
                    ));
                };
                Ok(MachineBinder {
                    name: name.as_dotted(),
                    ty: self.lower_expr(*ty)?,
                    span: binder.span,
                })
            })
            .collect()
    }

    fn lower_expr(&mut self, expr: HumanExpr) -> HumanResult<MachineTerm> {
        Ok(match expr {
            HumanExpr::Ident {
                name,
                universe_args,
                implicit_mode,
                span,
            } => {
                let resolved = self.next_name_use(&name)?;
                match resolved {
                    HumanResolvedName::Local { name, .. } => MachineTerm::Local {
                        name: name.as_dotted(),
                        span,
                    },
                    HumanResolvedName::Global(reference) => MachineTerm::Ident {
                        name: machine_name_from_global_ref(&reference, span),
                        universe_args: universe_args.map(|levels| {
                            levels
                                .into_iter()
                                .map(lower_level)
                                .collect::<Vec<MachineLevel>>()
                        }),
                        explicit_mode: implicit_mode == HumanImplicitMode::Explicit,
                        span,
                    },
                }
            }
            HumanExpr::Sort { level, span } => MachineTerm::Sort {
                level: lower_level(level),
                span,
            },
            HumanExpr::App { func, arg, span } => MachineTerm::App {
                func: Box::new(self.lower_expr(*func)?),
                arg: Box::new(self.lower_expr(*arg)?),
                span,
            },
            HumanExpr::Lam {
                binders,
                body,
                span,
            } => MachineTerm::Lam {
                binders: self.lower_binders(binders)?,
                body: Box::new(self.lower_expr(*body)?),
                span,
            },
            HumanExpr::Pi {
                binders,
                body,
                span,
            } => MachineTerm::Pi {
                binders: self.lower_binders(binders)?,
                body: Box::new(self.lower_expr(*body)?),
                span,
            },
            HumanExpr::Let {
                name,
                ty,
                value,
                body,
                span,
            } => {
                let Some(ty) = ty else {
                    return Err(HumanDiagnostic::not_implemented(
                        span,
                        "unannotated Human let lowering",
                    ));
                };
                MachineTerm::Let {
                    name: name.as_dotted(),
                    ty: Box::new(self.lower_expr(*ty)?),
                    value: Box::new(self.lower_expr(*value)?),
                    body: Box::new(self.lower_expr(*body)?),
                    span,
                }
            }
            HumanExpr::Annot { expr, ty, span } => MachineTerm::Annot {
                expr: Box::new(self.lower_expr(*expr)?),
                ty: Box::new(self.lower_expr(*ty)?),
                span,
            },
            HumanExpr::Arrow {
                domain,
                codomain,
                span,
            } => MachineTerm::Pi {
                binders: vec![MachineBinder {
                    name: "_".to_owned(),
                    ty: self.lower_expr(*domain)?,
                    span,
                }],
                body: Box::new(self.lower_expr(*codomain)?),
                span,
            },
            HumanExpr::Hole { span, .. } => {
                return Err(HumanDiagnostic::not_implemented(
                    span,
                    "Human hole elaboration",
                ));
            }
            HumanExpr::NotationApp { head, args, span } => {
                let lowered_args = args
                    .into_iter()
                    .map(|arg| self.lower_expr(arg))
                    .collect::<HumanResult<Vec<_>>>()?;
                let notation = self.next_notation_use(&head)?;
                let choice = self.next_notation_choice(&head)?;
                let Some(candidate) = notation.candidates.get(choice) else {
                    return Err(HumanDiagnostic::error(
                        HumanDiagnosticKind::AmbiguousNotation,
                        head.span,
                        format!("notation {} candidate plan is out of range", head.token),
                    ));
                };
                let mut term = MachineTerm::Ident {
                    name: machine_name_from_global_ref(candidate, head.span),
                    universe_args: None,
                    explicit_mode: false,
                    span: head.span,
                };
                for arg in lowered_args {
                    let app_span = term.span().join(arg.span());
                    term = MachineTerm::App {
                        func: Box::new(term),
                        arg: Box::new(arg),
                        span: app_span,
                    };
                }
                let _ = span;
                term
            }
        })
    }

    fn next_name_use(&mut self, source: &crate::HumanName) -> HumanResult<HumanResolvedName> {
        let Some(resolved) = self.name_uses.next() else {
            return Err(HumanDiagnostic::not_implemented(
                source.span,
                "Human resolved name cursor",
            ));
        };
        debug_assert_eq!(resolved.source.as_dotted(), source.as_dotted());
        Ok(resolved.resolved.clone())
    }

    fn next_notation_use(
        &mut self,
        source: &crate::HumanNotationHead,
    ) -> HumanResult<HumanResolvedNotationUse> {
        let Some(resolved) = self.notation_uses.next() else {
            return Err(HumanDiagnostic::not_implemented(
                source.span,
                "Human resolved notation cursor",
            ));
        };
        debug_assert_eq!(resolved.head.token, source.token);
        Ok(resolved.clone())
    }

    fn next_notation_choice(&mut self, source: &crate::HumanNotationHead) -> HumanResult<usize> {
        self.notation_choices.next().copied().ok_or_else(|| {
            HumanDiagnostic::not_implemented(source.span, "Human notation choice cursor")
        })
    }
}

fn lower_level(level: HumanLevel) -> MachineLevel {
    match level {
        HumanLevel::Nat { value, span } => MachineLevel::Nat { value, span },
        HumanLevel::Param { name, span } => MachineLevel::Param { name, span },
        HumanLevel::Succ { level, span } => MachineLevel::Succ {
            level: Box::new(lower_level(*level)),
            span,
        },
        HumanLevel::Max { lhs, rhs, span } => MachineLevel::Max {
            lhs: Box::new(lower_level(*lhs)),
            rhs: Box::new(lower_level(*rhs)),
            span,
        },
        HumanLevel::IMax { lhs, rhs, span } => MachineLevel::IMax {
            lhs: Box::new(lower_level(*lhs)),
            rhs: Box::new(lower_level(*rhs)),
            span,
        },
    }
}

fn machine_name(name: crate::HumanName) -> MachineName {
    MachineName {
        parts: name.parts,
        span: name.span,
    }
}

fn machine_name_from_global_ref(reference: &HumanGlobalRef, span: Span) -> MachineName {
    match reference {
        HumanGlobalRef::Imported { name, .. }
        | HumanGlobalRef::Local { name, .. }
        | HumanGlobalRef::LocalGenerated { name, .. } => MachineName {
            parts: name.0.clone(),
            span,
        },
    }
}

fn machine_diagnostic_to_human(diagnostic: crate::MachineDiagnostic) -> HumanDiagnostic {
    HumanDiagnostic::error(
        HumanDiagnosticKind::MachineElaborationError,
        diagnostic.primary_span,
        diagnostic.message,
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::{FileId, HumanDiagnosticKind};
    use npa_kernel::{eq_refl, eq_refl_type, eq_type, nat, type0, Decl, Expr, Level, Reducibility};

    fn hash(seed: u8) -> npa_cert::Hash {
        [seed; 32]
    }

    fn verified_import(module: &str, exports: &[(&str, &[&str])]) -> VerifiedImport {
        let exports: Vec<_> = exports
            .iter()
            .enumerate()
            .map(|(index, (name, universe_params))| {
                let name = npa_cert::Name::from_dotted(name);
                crate::VerifiedExport {
                    universe_params: universe_params
                        .iter()
                        .map(|param| param.to_string())
                        .collect(),
                    ty: export_ty(&name.as_dotted()),
                    decl_interface_hash: npa_cert::builtin_decl_interface_hash(&name)
                        .unwrap_or_else(|| hash(index as u8 + 2)),
                    name,
                }
            })
            .collect();
        let kernel_decls = exports
            .iter()
            .map(|export| Decl::Axiom {
                name: export.name.as_dotted(),
                universe_params: export.universe_params.clone(),
                ty: export.ty.clone(),
            })
            .collect();
        let decl_interface_hashes = exports
            .iter()
            .map(|export| (export.name.clone(), export.decl_interface_hash))
            .collect();

        VerifiedImport {
            module: npa_cert::Name::from_dotted(module),
            export_hash: hash(1),
            certificate_hash: None,
            exports,
            decl_interface_hashes,
            kernel_decls,
            kernel_decl_dependencies: BTreeMap::new(),
        }
    }

    fn nat_import() -> VerifiedImport {
        verified_import("Std.Nat.Basic", &[("Nat", &[])])
    }

    fn eq_import() -> VerifiedImport {
        verified_import("Std.Logic.Eq", &[("Eq", &["u"]), ("Eq.refl", &["u"])])
    }

    fn non_builtin_hash_eq_import() -> VerifiedImport {
        let mut import = eq_import();
        for export in &mut import.exports {
            if export.name == npa_cert::Name::from_dotted("Eq.refl") {
                export.decl_interface_hash = hash(99);
            }
        }
        import.decl_interface_hashes = import
            .exports
            .iter()
            .map(|export| (export.name.clone(), export.decl_interface_hash))
            .collect();
        import
    }

    fn export_ty(name: &str) -> Expr {
        match name {
            "Nat" => Expr::sort(type0()),
            "Eq" => eq_type(Level::param("u")),
            "Eq.refl" => eq_refl_type(Level::param("u")),
            _ => Expr::sort(Level::zero()),
        }
    }

    #[test]
    fn compile_human_source_checks_verified_imports_before_elaboration() {
        let err = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Current.Module"),
            "import Std.Nat.Basic",
            &[],
            &HumanCompileOptions::default(),
        )
        .expect_err("missing import should fail during Human resolution");

        assert_eq!(err.kind, HumanDiagnosticKind::MissingVerifiedImport);
    }

    #[test]
    fn elaborates_single_candidate_infix_notation_to_machine_application() {
        let module = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
def add (n m : Sort 2) : Sort 2 := n
infixl:65 \" + \" => add
def use (n : Sort 2) : Sort 2 := n + Type",
            &[],
            &HumanCompileOptions::default(),
        )
        .expect("single-candidate notation should elaborate through the Machine path");

        let sort_1 = Expr::sort(Level::succ(Level::zero()));
        let sort_2 = Expr::sort(Level::succ(Level::succ(Level::zero())));
        assert_eq!(module.declarations.len(), 2);
        assert_eq!(
            module.declarations[1],
            Decl::Def {
                name: "use".to_owned(),
                universe_params: Vec::new(),
                ty: Expr::pi("n", sort_2.clone(), sort_2.clone()),
                value: Expr::lam(
                    "n",
                    sort_2,
                    Expr::app(Expr::app(Expr::konst("add", vec![]), Expr::bvar(0)), sort_1)
                ),
                reducibility: Reducibility::Reducible,
            }
        );
    }

    #[test]
    fn notation_elaboration_rolls_back_failed_candidate_and_uses_successful_one() {
        let module = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
def add_bad (n m : Type) : Type := n
def add_ok (n m : Sort 2) : Sort 2 := n
infixl:65 \" + \" => add_bad
infixl:65 \" + \" => add_ok
def use (n : Sort 2) : Sort 2 := n + Type",
            &[],
            &HumanCompileOptions::default(),
        )
        .expect("elaboration should try the second notation candidate after the first fails");

        assert_eq!(module.declarations.len(), 3);
        let Decl::Def { value, .. } = &module.declarations[2] else {
            panic!("expected use definition");
        };
        assert_eq!(
            value,
            &Expr::lam(
                "n",
                Expr::sort(Level::succ(Level::succ(Level::zero()))),
                Expr::app(
                    Expr::app(Expr::konst("add_ok", vec![]), Expr::bvar(0)),
                    Expr::sort(Level::succ(Level::zero()))
                )
            )
        );
    }

    #[test]
    fn human_path_inserts_implicit_type_argument_for_eq_refl() {
        let imports = [nat_import(), eq_import()];
        let module = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Nat.Basic
import Std.Logic.Eq
theorem self_eq (n : Nat) : Eq.{1} Nat n n := Eq.refl n",
            &imports,
            &HumanCompileOptions::default(),
        )
        .expect("Human path should insert Eq.refl implicit type argument");

        assert_eq!(module.declarations.len(), 1);
        let Decl::Theorem { proof, .. } = &module.declarations[0] else {
            panic!("expected theorem");
        };
        assert_eq!(
            proof,
            &Expr::lam("n", nat(), eq_refl(type0(), nat(), Expr::bvar(0)))
        );
    }

    #[test]
    fn human_explicit_mode_suppresses_implicit_insertion() {
        let imports = [nat_import(), eq_import()];
        let err = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Nat.Basic
import Std.Logic.Eq
theorem bad (n : Nat) : Eq.{1} Nat n n := @Eq.refl.{1} n",
            &imports,
            &HumanCompileOptions::default(),
        )
        .expect_err("@ mode should not insert the implicit type argument");

        assert_eq!(err.kind, HumanDiagnosticKind::MachineElaborationError);
    }

    #[test]
    fn human_explicit_mode_accepts_explicit_implicit_argument() {
        let imports = [nat_import(), eq_import()];
        compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Nat.Basic
import Std.Logic.Eq
theorem self_eq (n : Nat) : Eq.{1} Nat n n := @Eq.refl.{1} Nat n",
            &imports,
            &HumanCompileOptions::default(),
        )
        .expect("explicit @ mode should accept an explicitly supplied implicit argument");
    }

    #[test]
    fn human_builtin_profile_requires_builtin_interface_hash() {
        let imports = [nat_import(), non_builtin_hash_eq_import()];
        let err = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Nat.Basic
import Std.Logic.Eq
theorem bad (n : Nat) : Eq.{1} Nat n n := Eq.refl n",
            &imports,
            &HumanCompileOptions::default(),
        )
        .expect_err("name-only Eq.refl should not get the builtin implicit profile");

        assert_eq!(err.kind, HumanDiagnosticKind::UnsolvedImplicit);
    }

    #[test]
    fn human_current_callable_profile_uses_implicit_binder_metadata() {
        let imports = [nat_import()];
        let module = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Nat.Basic
def id {A : Type} (x : A) : A := x
def use (n : Nat) : Nat := id n",
            &imports,
            &HumanCompileOptions::default(),
        )
        .expect("Human path should insert current declaration implicit type argument");

        assert_eq!(module.declarations.len(), 2);
        let Decl::Def { value, .. } = &module.declarations[1] else {
            panic!("expected def");
        };
        assert_eq!(
            value,
            &Expr::lam(
                "n",
                nat(),
                Expr::app(Expr::app(Expr::konst("id", vec![]), nat()), Expr::bvar(0))
            )
        );
    }

    #[test]
    fn human_unresolved_implicit_is_rejected_before_certificate_output() {
        let imports = [nat_import(), eq_import()];
        let err = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Nat.Basic
import Std.Logic.Eq
def bad : Type := Eq.refl",
            &imports,
            &HumanCompileOptions::default(),
        )
        .expect_err("unresolved implicit should reject the declaration");

        assert_eq!(err.kind, HumanDiagnosticKind::UnsolvedImplicit);
    }
}
