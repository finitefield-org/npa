use std::collections::{BTreeMap, VecDeque};

use crate::{
    builtin_machine_callable_profile, machine_callable_profile_from_human_binders,
    parse_human_module, resolve_human_module,
    resolver::{find_unique_verified_import_by_module, VerifiedImportLookupError},
    HumanBinder, HumanBinderKind, HumanCompileOptions, HumanDiagnostic, HumanDiagnosticKind,
    HumanDiagnosticPayload, HumanExpr, HumanGlobalRef, HumanHoleGoal, HumanHoleGoalLocal,
    HumanImplicitMode, HumanItem, HumanLevel, HumanName, HumanResolvedName, HumanResolvedNameUse,
    HumanResolvedNotationUse, HumanResult, HumanSourceDeclarationMetadata, MachineBinder,
    MachineCallableBinderVisibility, MachineDecl, MachineItem, MachineLevel, MachineModule,
    MachineName, MachineTerm, ResolvedHumanModule, Span, VerifiedImport,
};
use npa_kernel::{subst, Ctx, Decl, Env, Error, Expr, Level, Reducibility};

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
    HumanBidirectionalElaborator::new(module, verified_imports)?
        .elaborate_module(module_name, machine_module)
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct HumanTermMetaId(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct HumanUniverseMetaId(u32);

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum HumanTermMetaKind {
    UserHole,
    SyntheticImplicit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HumanTermMeta {
    id: HumanTermMetaId,
    name: Option<String>,
    context: HumanMetaContextSnapshot,
    goal_context: Vec<HumanHoleGoalLocal>,
    target: Option<String>,
    assignment: Option<HumanMetaExpr>,
    kind: HumanTermMetaKind,
    span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HumanUniverseMeta {
    id: HumanUniverseMetaId,
    assignment: Option<HumanMetaLevel>,
    span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HumanMetaContextSnapshot {
    locals: Vec<HumanMetaLocalSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HumanMetaLocalSnapshot {
    ty: MachineTerm,
    value: Option<MachineTerm>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum HumanMetaExpr {
    Core(Expr),
    Meta(HumanTermMetaId),
    App(Box<HumanMetaExpr>, Box<HumanMetaExpr>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum HumanMetaLevel {
    Core(Level),
    Meta(HumanUniverseMetaId),
    Succ(Box<HumanMetaLevel>),
    Max(Box<HumanMetaLevel>, Box<HumanMetaLevel>),
    IMax(Box<HumanMetaLevel>, Box<HumanMetaLevel>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HumanTermConstraintKind {
    TypeEq,
    TermEq,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum HumanConstraint {
    TypeEq {
        lhs: HumanMetaExpr,
        rhs: HumanMetaExpr,
        span: Span,
    },
    TermEq {
        lhs: HumanMetaExpr,
        rhs: HumanMetaExpr,
        span: Span,
    },
    LevelEq {
        lhs: HumanMetaLevel,
        rhs: HumanMetaLevel,
        span: Span,
    },
    LevelLe {
        lhs: HumanMetaLevel,
        rhs: HumanMetaLevel,
        span: Span,
    },
}

#[derive(Clone, Debug, Default)]
struct HumanMetaStore {
    term_metas: Vec<HumanTermMeta>,
    universe_metas: Vec<HumanUniverseMeta>,
    constraints: VecDeque<HumanConstraint>,
    named_holes: BTreeMap<String, HumanTermMetaId>,
}

impl HumanMetaStore {
    fn begin_declaration(&mut self) {
        self.term_metas.clear();
        self.universe_metas.clear();
        self.constraints.clear();
        self.named_holes.clear();
    }

    fn fresh_user_hole(
        &mut self,
        name: Option<&HumanName>,
        context: &HumanLoweringLocalContext,
        expected: Option<&MachineTerm>,
        span: Span,
    ) -> HumanResult<HumanTermMetaId> {
        let snapshot = context.meta_snapshot();
        let hole_name = name.map(|name| format!("?{}", name.as_dotted()));

        if let Some(hole_name) = &hole_name {
            if let Some(existing) = self.named_holes.get(hole_name).copied() {
                let existing_meta = self.term_meta(existing);
                if existing_meta.context != snapshot {
                    return Err(HumanDiagnostic::error(
                        HumanDiagnosticKind::NamedHoleContextMismatch,
                        span,
                        format!("named hole {hole_name} was reused with a different context"),
                    )
                    .with_payload(HumanDiagnosticPayload {
                        hole_goals: vec![
                            self.hole_goal(existing_meta),
                            HumanHoleGoal {
                                hole: Some(hole_name.clone()),
                                context: context.hole_goal_context(),
                                target: expected.map(render_machine_term),
                                source_span: span,
                            },
                        ],
                        ..HumanDiagnosticPayload::default()
                    }));
                }
                return Ok(existing);
            }
        }

        let id = HumanTermMetaId(self.term_metas.len() as u32);
        let meta = HumanTermMeta {
            id,
            name: hole_name.clone(),
            context: snapshot,
            goal_context: context.hole_goal_context(),
            target: expected.map(render_machine_term),
            assignment: None,
            kind: HumanTermMetaKind::UserHole,
            span,
        };
        self.term_metas.push(meta);
        if let Some(hole_name) = hole_name {
            self.named_holes.insert(hole_name, id);
        }
        Ok(id)
    }

    #[allow(dead_code)]
    fn fresh_synthetic_implicit(
        &mut self,
        context: &HumanLoweringLocalContext,
        expected: Option<&MachineTerm>,
        span: Span,
    ) -> HumanTermMetaId {
        let id = HumanTermMetaId(self.term_metas.len() as u32);
        self.term_metas.push(HumanTermMeta {
            id,
            name: None,
            context: context.meta_snapshot(),
            goal_context: context.hole_goal_context(),
            target: expected.map(render_machine_term),
            assignment: None,
            kind: HumanTermMetaKind::SyntheticImplicit,
            span,
        });
        id
    }

    #[allow(dead_code)]
    fn fresh_universe_meta(&mut self, span: Span) -> HumanUniverseMetaId {
        let id = HumanUniverseMetaId(self.universe_metas.len() as u32);
        self.universe_metas.push(HumanUniverseMeta {
            id,
            assignment: None,
            span,
        });
        id
    }

    fn add_constraint(&mut self, constraint: HumanConstraint) {
        self.constraints.push_back(constraint);
    }

    fn solve_constraints(&mut self) -> HumanResult<()> {
        while let Some(constraint) = self.constraints.pop_front() {
            match constraint {
                HumanConstraint::TypeEq { lhs, rhs, span } => {
                    self.solve_term_eq(HumanTermConstraintKind::TypeEq, lhs, rhs, span)?;
                }
                HumanConstraint::TermEq { lhs, rhs, span } => {
                    self.solve_term_eq(HumanTermConstraintKind::TermEq, lhs, rhs, span)?;
                }
                HumanConstraint::LevelEq { lhs, rhs, span } => {
                    self.solve_level_eq(lhs, rhs, span)?;
                }
                HumanConstraint::LevelLe { lhs, rhs, span } => {
                    self.solve_level_le(lhs, rhs, span)?;
                }
            }
        }
        Ok(())
    }

    fn reject_unsolved_for_decl(&mut self, span: Span) -> HumanResult<()> {
        self.solve_constraints()?;

        if let Some(meta) = self
            .term_metas
            .iter()
            .find(|meta| meta.assignment.is_none())
        {
            return match meta.kind {
                HumanTermMetaKind::UserHole => Err(HumanDiagnostic::error(
                    HumanDiagnosticKind::UnsolvedHole,
                    meta.span,
                    match &meta.name {
                        Some(name) => format!("unsolved hole {name}"),
                        None => "unsolved anonymous hole".to_owned(),
                    },
                )
                .with_payload(HumanDiagnosticPayload {
                    hole_goals: vec![self.hole_goal(meta)],
                    ..HumanDiagnosticPayload::default()
                })),
                HumanTermMetaKind::SyntheticImplicit => Err(HumanDiagnostic::error(
                    HumanDiagnosticKind::UnsolvedImplicit,
                    meta.span,
                    "unsolved synthetic implicit argument",
                )),
            };
        }

        if let Some(meta) = self
            .universe_metas
            .iter()
            .find(|meta| meta.assignment.is_none())
        {
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::UnsolvedUniverseMeta,
                meta.span,
                "unsolved universe metavariable",
            ));
        }

        let _ = span;
        Ok(())
    }

    fn solve_term_eq(
        &mut self,
        kind: HumanTermConstraintKind,
        lhs: HumanMetaExpr,
        rhs: HumanMetaExpr,
        span: Span,
    ) -> HumanResult<()> {
        let lhs = self.resolve_meta_expr(lhs);
        let rhs = self.resolve_meta_expr(rhs);
        match (lhs, rhs) {
            (lhs, rhs) if lhs == rhs => Ok(()),
            (HumanMetaExpr::Meta(id), value) | (value, HumanMetaExpr::Meta(id)) => {
                self.assign_term_meta(id, value, span)
            }
            (HumanMetaExpr::App(lhs_fun, lhs_arg), HumanMetaExpr::App(rhs_fun, rhs_arg)) => {
                self.add_constraint(term_constraint(kind, *lhs_fun, *rhs_fun, span));
                self.add_constraint(term_constraint(kind, *lhs_arg, *rhs_arg, span));
                Ok(())
            }
            (HumanMetaExpr::Core(lhs), HumanMetaExpr::Core(rhs)) if lhs == rhs => Ok(()),
            _ => Err(HumanDiagnostic::error(
                HumanDiagnosticKind::MachineElaborationError,
                span,
                "Human metavariable constraint could not be unified",
            )),
        }
    }

    fn solve_level_eq(
        &mut self,
        lhs: HumanMetaLevel,
        rhs: HumanMetaLevel,
        span: Span,
    ) -> HumanResult<()> {
        let lhs = self.resolve_meta_level(lhs);
        let rhs = self.resolve_meta_level(rhs);
        match (lhs, rhs) {
            (lhs, rhs) if lhs == rhs => Ok(()),
            (HumanMetaLevel::Meta(id), value) | (value, HumanMetaLevel::Meta(id)) => {
                self.assign_universe_meta(id, value, span)
            }
            (HumanMetaLevel::Succ(lhs), HumanMetaLevel::Succ(rhs)) => {
                self.add_constraint(HumanConstraint::LevelEq {
                    lhs: *lhs,
                    rhs: *rhs,
                    span,
                });
                Ok(())
            }
            (HumanMetaLevel::Max(lhs_a, lhs_b), HumanMetaLevel::Max(rhs_a, rhs_b)) => {
                self.add_constraint(HumanConstraint::LevelEq {
                    lhs: *lhs_a,
                    rhs: *rhs_a,
                    span,
                });
                self.add_constraint(HumanConstraint::LevelEq {
                    lhs: *lhs_b,
                    rhs: *rhs_b,
                    span,
                });
                Ok(())
            }
            (HumanMetaLevel::IMax(lhs_a, lhs_b), HumanMetaLevel::IMax(rhs_a, rhs_b)) => {
                self.add_constraint(HumanConstraint::LevelEq {
                    lhs: *lhs_a,
                    rhs: *rhs_a,
                    span,
                });
                self.add_constraint(HumanConstraint::LevelEq {
                    lhs: *lhs_b,
                    rhs: *rhs_b,
                    span,
                });
                Ok(())
            }
            (HumanMetaLevel::Core(lhs), HumanMetaLevel::Core(rhs)) if lhs == rhs => Ok(()),
            _ => Err(HumanDiagnostic::error(
                HumanDiagnosticKind::MachineElaborationError,
                span,
                "Human universe metavariable constraint could not be unified",
            )),
        }
    }

    fn solve_level_le(
        &mut self,
        lhs: HumanMetaLevel,
        rhs: HumanMetaLevel,
        span: Span,
    ) -> HumanResult<()> {
        let lhs = self.resolve_meta_level(lhs);
        let rhs = self.resolve_meta_level(rhs);
        match (lhs, rhs) {
            (lhs, rhs) if lhs == rhs => Ok(()),
            (HumanMetaLevel::Meta(id), value) => self.assign_universe_meta(id, value, span),
            (HumanMetaLevel::Core(lhs), HumanMetaLevel::Core(rhs))
                if human_level_leq(&lhs, &rhs) =>
            {
                Ok(())
            }
            _ => Err(HumanDiagnostic::error(
                HumanDiagnosticKind::MachineElaborationError,
                span,
                "Human universe inequality constraint could not be solved",
            )),
        }
    }

    fn assign_term_meta(
        &mut self,
        id: HumanTermMetaId,
        value: HumanMetaExpr,
        span: Span,
    ) -> HumanResult<()> {
        if meta_expr_occurs(id, &value) {
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::OccursCheckFailed,
                span,
                "Human metavariable assignment failed the occurs check",
            ));
        }

        let index = id.0 as usize;
        if let Some(existing) = self.term_metas[index].assignment.clone() {
            self.add_constraint(HumanConstraint::TermEq {
                lhs: existing,
                rhs: value,
                span,
            });
        } else {
            self.term_metas[index].assignment = Some(value);
        }
        Ok(())
    }

    fn assign_universe_meta(
        &mut self,
        id: HumanUniverseMetaId,
        value: HumanMetaLevel,
        span: Span,
    ) -> HumanResult<()> {
        if meta_level_occurs(id, &value) {
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::OccursCheckFailed,
                span,
                "Human universe metavariable assignment failed the occurs check",
            ));
        }

        let index = id.0 as usize;
        if let Some(existing) = self.universe_metas[index].assignment.clone() {
            self.add_constraint(HumanConstraint::LevelEq {
                lhs: existing,
                rhs: value,
                span,
            });
        } else {
            self.universe_metas[index].assignment = Some(value);
        }
        Ok(())
    }

    fn resolve_meta_expr(&self, value: HumanMetaExpr) -> HumanMetaExpr {
        match value {
            HumanMetaExpr::Meta(id) => self.term_metas[id.0 as usize]
                .assignment
                .clone()
                .map(|assignment| self.resolve_meta_expr(assignment))
                .unwrap_or(HumanMetaExpr::Meta(id)),
            HumanMetaExpr::App(func, arg) => HumanMetaExpr::App(
                Box::new(self.resolve_meta_expr(*func)),
                Box::new(self.resolve_meta_expr(*arg)),
            ),
            HumanMetaExpr::Core(expr) => HumanMetaExpr::Core(expr),
        }
    }

    fn resolve_meta_level(&self, value: HumanMetaLevel) -> HumanMetaLevel {
        match value {
            HumanMetaLevel::Meta(id) => self.universe_metas[id.0 as usize]
                .assignment
                .clone()
                .map(|assignment| self.resolve_meta_level(assignment))
                .unwrap_or(HumanMetaLevel::Meta(id)),
            HumanMetaLevel::Succ(level) => {
                HumanMetaLevel::Succ(Box::new(self.resolve_meta_level(*level)))
            }
            HumanMetaLevel::Max(lhs, rhs) => HumanMetaLevel::Max(
                Box::new(self.resolve_meta_level(*lhs)),
                Box::new(self.resolve_meta_level(*rhs)),
            ),
            HumanMetaLevel::IMax(lhs, rhs) => HumanMetaLevel::IMax(
                Box::new(self.resolve_meta_level(*lhs)),
                Box::new(self.resolve_meta_level(*rhs)),
            ),
            HumanMetaLevel::Core(level) => HumanMetaLevel::Core(level),
        }
    }

    fn term_meta(&self, id: HumanTermMetaId) -> &HumanTermMeta {
        &self.term_metas[id.0 as usize]
    }

    fn hole_goal(&self, meta: &HumanTermMeta) -> HumanHoleGoal {
        HumanHoleGoal {
            hole: meta.name.clone(),
            context: meta.goal_context.clone(),
            target: meta.target.clone(),
            source_span: meta.span,
        }
    }
}

fn term_constraint(
    kind: HumanTermConstraintKind,
    lhs: HumanMetaExpr,
    rhs: HumanMetaExpr,
    span: Span,
) -> HumanConstraint {
    match kind {
        HumanTermConstraintKind::TypeEq => HumanConstraint::TypeEq { lhs, rhs, span },
        HumanTermConstraintKind::TermEq => HumanConstraint::TermEq { lhs, rhs, span },
    }
}

fn meta_expr_occurs(id: HumanTermMetaId, value: &HumanMetaExpr) -> bool {
    match value {
        HumanMetaExpr::Core(_) => false,
        HumanMetaExpr::Meta(value_id) => *value_id == id,
        HumanMetaExpr::App(func, arg) => meta_expr_occurs(id, func) || meta_expr_occurs(id, arg),
    }
}

fn meta_level_occurs(id: HumanUniverseMetaId, value: &HumanMetaLevel) -> bool {
    match value {
        HumanMetaLevel::Core(_) => false,
        HumanMetaLevel::Meta(value_id) => *value_id == id,
        HumanMetaLevel::Succ(level) => meta_level_occurs(id, level),
        HumanMetaLevel::Max(lhs, rhs) | HumanMetaLevel::IMax(lhs, rhs) => {
            meta_level_occurs(id, lhs) || meta_level_occurs(id, rhs)
        }
    }
}

fn human_meta_placeholder(id: HumanTermMetaId, span: Span) -> MachineTerm {
    MachineTerm::Local {
        name: format!("__human_meta_{}", id.0),
        span,
    }
}

fn canonicalize_machine_term_for_meta_context(term: &MachineTerm) -> MachineTerm {
    let span = meta_context_span();
    match term {
        MachineTerm::Ident {
            name,
            universe_args,
            explicit_mode,
            ..
        } => MachineTerm::Ident {
            name: MachineName {
                parts: name.parts.clone(),
                span,
            },
            universe_args: universe_args.as_ref().map(|args| {
                args.iter()
                    .map(canonicalize_machine_level_for_meta_context)
                    .collect()
            }),
            explicit_mode: *explicit_mode,
            span,
        },
        MachineTerm::Local { name, .. } => MachineTerm::Local {
            name: name.clone(),
            span,
        },
        MachineTerm::Prop { .. } => MachineTerm::Prop { span },
        MachineTerm::Type { level, .. } => MachineTerm::Type {
            level: canonicalize_machine_level_for_meta_context(level),
            span,
        },
        MachineTerm::Sort { level, .. } => MachineTerm::Sort {
            level: canonicalize_machine_level_for_meta_context(level),
            span,
        },
        MachineTerm::App { func, arg, .. } => MachineTerm::App {
            func: Box::new(canonicalize_machine_term_for_meta_context(func)),
            arg: Box::new(canonicalize_machine_term_for_meta_context(arg)),
            span,
        },
        MachineTerm::Lam { binders, body, .. } => MachineTerm::Lam {
            binders: canonicalize_machine_binders_for_meta_context(binders),
            body: Box::new(canonicalize_machine_term_for_meta_context(body)),
            span,
        },
        MachineTerm::Pi { binders, body, .. } => MachineTerm::Pi {
            binders: canonicalize_machine_binders_for_meta_context(binders),
            body: Box::new(canonicalize_machine_term_for_meta_context(body)),
            span,
        },
        MachineTerm::Let {
            name,
            ty,
            value,
            body,
            ..
        } => MachineTerm::Let {
            name: name.clone(),
            ty: Box::new(canonicalize_machine_term_for_meta_context(ty)),
            value: Box::new(canonicalize_machine_term_for_meta_context(value)),
            body: Box::new(canonicalize_machine_term_for_meta_context(body)),
            span,
        },
        MachineTerm::Annot { expr, ty, .. } => MachineTerm::Annot {
            expr: Box::new(canonicalize_machine_term_for_meta_context(expr)),
            ty: Box::new(canonicalize_machine_term_for_meta_context(ty)),
            span,
        },
    }
}

fn canonicalize_machine_binders_for_meta_context(binders: &[MachineBinder]) -> Vec<MachineBinder> {
    let span = meta_context_span();
    binders
        .iter()
        .map(|binder| MachineBinder {
            name: binder.name.clone(),
            ty: canonicalize_machine_term_for_meta_context(&binder.ty),
            span,
        })
        .collect()
}

fn canonicalize_machine_level_for_meta_context(level: &MachineLevel) -> MachineLevel {
    let span = meta_context_span();
    match level {
        MachineLevel::Nat { value, .. } => MachineLevel::Nat {
            value: *value,
            span,
        },
        MachineLevel::Param { name, .. } => MachineLevel::Param {
            name: name.clone(),
            span,
        },
        MachineLevel::Succ { level, .. } => MachineLevel::Succ {
            level: Box::new(canonicalize_machine_level_for_meta_context(level)),
            span,
        },
        MachineLevel::Max { lhs, rhs, .. } => MachineLevel::Max {
            lhs: Box::new(canonicalize_machine_level_for_meta_context(lhs)),
            rhs: Box::new(canonicalize_machine_level_for_meta_context(rhs)),
            span,
        },
        MachineLevel::IMax { lhs, rhs, .. } => MachineLevel::IMax {
            lhs: Box::new(canonicalize_machine_level_for_meta_context(lhs)),
            rhs: Box::new(canonicalize_machine_level_for_meta_context(rhs)),
            span,
        },
    }
}

fn meta_context_span() -> Span {
    Span::empty(crate::FileId(0))
}

fn render_machine_term(term: &MachineTerm) -> String {
    match term {
        MachineTerm::Ident {
            name,
            universe_args,
            explicit_mode,
            ..
        } => {
            let mut rendered = if *explicit_mode {
                format!("@{}", name.as_dotted())
            } else {
                name.as_dotted()
            };
            if let Some(args) = universe_args {
                rendered.push_str(".{");
                rendered.push_str(
                    &args
                        .iter()
                        .map(render_machine_level)
                        .collect::<Vec<_>>()
                        .join(", "),
                );
                rendered.push('}');
            }
            rendered
        }
        MachineTerm::Local { name, .. } => name.clone(),
        MachineTerm::Prop { .. } => "Prop".to_owned(),
        MachineTerm::Type { level, .. } => format!("Type {}", render_machine_level(level)),
        MachineTerm::Sort { level, .. } => format!("Sort {}", render_machine_level(level)),
        MachineTerm::App { func, arg, .. } => {
            format!(
                "({} {})",
                render_machine_term(func),
                render_machine_term(arg)
            )
        }
        MachineTerm::Lam { binders, body, .. } => format!(
            "(fun {} => {})",
            render_machine_binders(binders),
            render_machine_term(body)
        ),
        MachineTerm::Pi { binders, body, .. } => format!(
            "(forall {}, {})",
            render_machine_binders(binders),
            render_machine_term(body)
        ),
        MachineTerm::Let {
            name,
            ty,
            value,
            body,
            ..
        } => format!(
            "(let {name} : {} := {} in {})",
            render_machine_term(ty),
            render_machine_term(value),
            render_machine_term(body)
        ),
        MachineTerm::Annot { expr, ty, .. } => {
            format!(
                "({} : {})",
                render_machine_term(expr),
                render_machine_term(ty)
            )
        }
    }
}

fn render_machine_binders(binders: &[MachineBinder]) -> String {
    binders
        .iter()
        .map(|binder| format!("({} : {})", binder.name, render_machine_term(&binder.ty)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_machine_level(level: &MachineLevel) -> String {
    match level {
        MachineLevel::Nat { value, .. } => value.to_string(),
        MachineLevel::Param { name, .. } => name.clone(),
        MachineLevel::Succ { level, .. } => format!("succ {}", render_machine_level(level)),
        MachineLevel::Max { lhs, rhs, .. } => {
            format!(
                "max {} {}",
                render_machine_level(lhs),
                render_machine_level(rhs)
            )
        }
        MachineLevel::IMax { lhs, rhs, .. } => {
            format!(
                "imax {} {}",
                render_machine_level(lhs),
                render_machine_level(rhs)
            )
        }
    }
}

fn human_level_leq(lhs: &Level, rhs: &Level) -> bool {
    match (
        core_level_as_u64(&npa_kernel::level::normalize_level(lhs.clone())),
        core_level_as_u64(&npa_kernel::level::normalize_level(rhs.clone())),
    ) {
        (Some(lhs), Some(rhs)) => lhs <= rhs,
        _ => lhs == rhs,
    }
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

    fn lookup(&self, name: &str, span: Span) -> HumanResult<(u32, Expr)> {
        let Some((index, local)) = self
            .locals
            .iter()
            .rev()
            .enumerate()
            .find(|(_, local)| local.name == name)
        else {
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::UnknownIdentifier,
                span,
                format!("unknown local name {name}"),
            ));
        };
        let index = index as u32;
        let ty = subst::shift(&local.ty, index as i32 + 1, 0)
            .map_err(|err| human_kernel_expr_diagnostic(span, err, "local type lookup"))?;
        Ok((index, ty))
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

struct HumanBidirectionalElaborator {
    env: Env,
}

impl HumanBidirectionalElaborator {
    fn new(module: &ResolvedHumanModule, verified_imports: &[VerifiedImport]) -> HumanResult<Self> {
        let mut elaborator = Self { env: Env::new() };

        let active_imports = active_human_imports(module, verified_imports)?;
        for import in active_imports {
            elaborator.add_import(import, module.module.span)?;
        }

        Ok(elaborator)
    }

    fn elaborate_module(
        mut self,
        module_name: npa_cert::ModuleName,
        module: MachineModule,
    ) -> HumanResult<npa_cert::CoreModule> {
        let mut declarations = Vec::new();

        for item in module.items {
            match item {
                MachineItem::Import { .. } => {}
                MachineItem::Def(decl) => {
                    let span = decl.span;
                    let decl = self.elaborate_decl(decl, HumanLoweredDeclKind::Def)?;
                    self.add_kernel_decl(decl.clone(), span)?;
                    declarations.push(decl);
                }
                MachineItem::Theorem(decl) => {
                    let span = decl.span;
                    let decl = self.elaborate_decl(decl, HumanLoweredDeclKind::Theorem)?;
                    self.add_kernel_decl(decl.clone(), span)?;
                    declarations.push(decl);
                }
            }
        }

        Ok(npa_cert::CoreModule {
            name: module_name,
            declarations,
        })
    }

    fn add_import(&mut self, import: &VerifiedImport, span: Span) -> HumanResult<()> {
        for decl in kernel_decls_for_human_import(import) {
            self.add_kernel_decl(decl, span)?;
        }
        Ok(())
    }

    fn elaborate_decl(&self, decl: MachineDecl, kind: HumanLoweredDeclKind) -> HumanResult<Decl> {
        let delta: Vec<_> = decl
            .universe_params
            .iter()
            .map(|param| param.name.clone())
            .collect();
        let mut locals = HumanLocalContext::default();
        let mut elaborated_binders = Vec::with_capacity(decl.binders.len());

        for binder in &decl.binders {
            let (ty, ty_type) = self.infer_human_expr(&binder.ty, &locals, &delta)?;
            self.expect_human_sort(&ty_type, &locals, &delta, binder.ty.span())?;
            locals.push_assumption(binder.name.clone(), ty.clone());
            elaborated_binders.push(HumanElaboratedBinder {
                name: binder.name.clone(),
                ty,
            });
        }

        let (ty, ty_type) = self.infer_human_expr(&decl.ty, &locals, &delta)?;
        self.expect_human_sort(&ty_type, &locals, &delta, decl.ty.span())?;
        let value = self.check_human_expr(&decl.value, &ty, &locals, &delta)?;

        let name = decl.name.as_dotted();
        let closed_ty = human_close_pi(&elaborated_binders, ty);
        let closed_value = human_close_lam(&elaborated_binders, value);
        let universe_params = delta;

        Ok(match kind {
            HumanLoweredDeclKind::Def => Decl::Def {
                name,
                universe_params,
                ty: closed_ty,
                value: closed_value,
                reducibility: Reducibility::Reducible,
            },
            HumanLoweredDeclKind::Theorem => Decl::Theorem {
                name,
                universe_params,
                ty: closed_ty,
                proof: closed_value,
            },
        })
    }

    fn infer_human_expr(
        &self,
        term: &MachineTerm,
        locals: &HumanLocalContext,
        delta: &[String],
    ) -> HumanResult<(Expr, Expr)> {
        Ok(match term {
            MachineTerm::Ident {
                name,
                universe_args,
                span,
                ..
            } => {
                let expr = self.elaborate_human_global(name, universe_args.as_deref(), *span)?;
                let ty = self.infer_core_expr_type(&expr, locals, delta, *span)?;
                (expr, ty)
            }
            MachineTerm::Local { name, span } => {
                let (index, ty) = locals.lookup(name, *span)?;
                (Expr::bvar(index), ty)
            }
            MachineTerm::Prop { .. } => (
                Expr::sort(Level::zero()),
                Expr::sort(Level::succ(Level::zero())),
            ),
            MachineTerm::Type { level, .. } => {
                let level = elaborate_machine_level(level.clone())?;
                let sort = Level::succ(level);
                (Expr::sort(sort.clone()), Expr::sort(Level::succ(sort)))
            }
            MachineTerm::Sort { level, .. } => {
                let level = elaborate_machine_level(level.clone())?;
                (Expr::sort(level.clone()), Expr::sort(Level::succ(level)))
            }
            MachineTerm::App { func, arg, span } => {
                let (func_expr, func_ty) = self.infer_human_expr(func, locals, delta)?;
                let func_ty = self.whnf_human_expr(&func_ty, locals, delta, *span)?;
                let Expr::Pi { ty, body, .. } = func_ty else {
                    return Err(HumanDiagnostic::error(
                        HumanDiagnosticKind::ExpectedFunctionType,
                        *span,
                        format!("application head is not a function: {func_ty:?}"),
                    ));
                };
                let arg_expr = self.check_human_expr(arg, &ty, locals, delta)?;
                let result_ty = subst::instantiate(&body, &arg_expr).map_err(|err| {
                    human_kernel_expr_diagnostic(*span, err, "Human application result type")
                })?;
                (Expr::app(func_expr, arg_expr), result_ty)
            }
            MachineTerm::Lam {
                binders,
                body,
                span: _,
            } => {
                let mut nested = locals.clone();
                let mut elaborated_binders = Vec::with_capacity(binders.len());
                for binder in binders {
                    let (ty, ty_type) = self.infer_human_expr(&binder.ty, &nested, delta)?;
                    self.expect_human_sort(&ty_type, &nested, delta, binder.ty.span())?;
                    nested.push_assumption(binder.name.clone(), ty.clone());
                    elaborated_binders.push(HumanElaboratedBinder {
                        name: binder.name.clone(),
                        ty,
                    });
                }
                let (body, body_ty) = self.infer_human_expr(body, &nested, delta)?;
                (
                    human_close_lam(&elaborated_binders, body),
                    human_close_pi(&elaborated_binders, body_ty),
                )
            }
            MachineTerm::Pi {
                binders,
                body,
                span,
            } => {
                let mut nested = locals.clone();
                let mut elaborated_binders = Vec::with_capacity(binders.len());
                for binder in binders {
                    let (ty, ty_type) = self.infer_human_expr(&binder.ty, &nested, delta)?;
                    self.expect_human_sort(&ty_type, &nested, delta, binder.ty.span())?;
                    nested.push_assumption(binder.name.clone(), ty.clone());
                    elaborated_binders.push(HumanElaboratedBinder {
                        name: binder.name.clone(),
                        ty,
                    });
                }
                let body_span = body.span();
                let (body_expr, body_type) = self.infer_human_expr(body, &nested, delta)?;
                self.expect_human_sort(&body_type, &nested, delta, body_span)?;
                let pi = human_close_pi(&elaborated_binders, body_expr);
                let pi_ty = self.infer_core_expr_type(&pi, locals, delta, *span)?;
                (pi, pi_ty)
            }
            MachineTerm::Let {
                name,
                ty,
                value,
                body,
                span,
            } => {
                let (ty_expr, ty_type) = self.infer_human_expr(ty, locals, delta)?;
                self.expect_human_sort(&ty_type, locals, delta, ty.span())?;
                let value_expr = self.check_human_expr(value, &ty_expr, locals, delta)?;
                let mut nested = locals.clone();
                nested.push_definition(name.clone(), ty_expr.clone(), value_expr.clone());
                let (body_expr, body_ty) = self.infer_human_expr(body, &nested, delta)?;
                let result_ty = subst::instantiate(&body_ty, &value_expr).map_err(|err| {
                    human_kernel_expr_diagnostic(*span, err, "Human let result type")
                })?;
                (
                    Expr::let_in(name.clone(), ty_expr, value_expr, body_expr),
                    result_ty,
                )
            }
            MachineTerm::Annot { expr, ty, span: _ } => {
                let (ty_expr, ty_type) = self.infer_human_expr(ty, locals, delta)?;
                self.expect_human_sort(&ty_type, locals, delta, ty.span())?;
                let expr = self.check_human_expr(expr, &ty_expr, locals, delta)?;
                (expr, ty_expr)
            }
        })
    }

    fn check_human_expr(
        &self,
        term: &MachineTerm,
        expected: &Expr,
        locals: &HumanLocalContext,
        delta: &[String],
    ) -> HumanResult<Expr> {
        if let MachineTerm::Lam {
            binders,
            body,
            span: _,
        } = term
        {
            return self.check_human_lambda(binders, body, expected, locals, delta);
        }

        let (expr, actual) = self.infer_human_expr(term, locals, delta)?;
        if self.is_human_defeq(&actual, expected, locals, delta, term.span())? {
            Ok(expr)
        } else {
            Err(HumanDiagnostic::error(
                HumanDiagnosticKind::TypeMismatch,
                term.span(),
                format!("type mismatch: expected {expected:?}, got {actual:?}"),
            ))
        }
    }

    fn check_human_lambda(
        &self,
        binders: &[MachineBinder],
        body: &MachineTerm,
        expected: &Expr,
        locals: &HumanLocalContext,
        delta: &[String],
    ) -> HumanResult<Expr> {
        let mut nested = locals.clone();
        let mut expected = expected.clone();
        let mut elaborated_binders = Vec::with_capacity(binders.len());

        for binder in binders {
            let expected_whnf = self.whnf_human_expr(&expected, &nested, delta, binder.span)?;
            let Expr::Pi { ty, body, .. } = expected_whnf else {
                return Err(HumanDiagnostic::error(
                    HumanDiagnosticKind::ExpectedFunctionType,
                    binder.span,
                    format!("lambda is checked against a non-function type: {expected_whnf:?}"),
                ));
            };
            let (binder_ty, binder_ty_type) = self.infer_human_expr(&binder.ty, &nested, delta)?;
            self.expect_human_sort(&binder_ty_type, &nested, delta, binder.ty.span())?;
            if !self.is_human_defeq(&binder_ty, &ty, &nested, delta, binder.ty.span())? {
                return Err(HumanDiagnostic::error(
                    HumanDiagnosticKind::TypeMismatch,
                    binder.ty.span(),
                    format!("lambda binder type mismatch: expected {ty:?}, got {binder_ty:?}"),
                ));
            }
            nested.push_assumption(binder.name.clone(), (*ty).clone());
            elaborated_binders.push(HumanElaboratedBinder {
                name: binder.name.clone(),
                ty: (*ty).clone(),
            });
            expected = *body;
        }

        let body = self.check_human_expr(body, &expected, &nested, delta)?;
        Ok(human_close_lam(&elaborated_binders, body))
    }

    fn elaborate_human_global(
        &self,
        name: &MachineName,
        universe_args: Option<&[MachineLevel]>,
        span: Span,
    ) -> HumanResult<Expr> {
        let name = name.as_dotted();
        let Some(decl) = self.env.decl(&name) else {
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::UnknownIdentifier,
                span,
                format!("unknown global name {name}"),
            ));
        };
        let expected = decl.universe_params().len();
        let levels = match universe_args {
            Some(args) if args.len() == expected => args
                .iter()
                .cloned()
                .map(elaborate_machine_level)
                .collect::<HumanResult<Vec<_>>>()?,
            Some(args) => {
                return Err(HumanDiagnostic::error(
                    HumanDiagnosticKind::UnsolvedUniverseMeta,
                    span,
                    format!(
                        "global name {name} expects {expected} universe arguments, got {}",
                        args.len()
                    ),
                ));
            }
            None if expected == 0 => Vec::new(),
            None => {
                return Err(HumanDiagnostic::error(
                    HumanDiagnosticKind::UnsolvedUniverseMeta,
                    span,
                    format!("global name {name} still has unresolved universe arguments"),
                ));
            }
        };
        Ok(Expr::konst(name, levels))
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
            .map_err(|err| human_kernel_expr_diagnostic(span, err, "Human expression inference"))
    }

    fn expect_human_sort(
        &self,
        inferred_type: &Expr,
        locals: &HumanLocalContext,
        delta: &[String],
        span: Span,
    ) -> HumanResult<()> {
        let whnf = self.whnf_human_expr(inferred_type, locals, delta, span)?;
        if matches!(whnf, Expr::Sort(_)) {
            Ok(())
        } else {
            Err(HumanDiagnostic::error(
                HumanDiagnosticKind::ExpectedSort,
                span,
                format!("expected a type, got {whnf:?}"),
            ))
        }
    }

    fn whnf_human_expr(
        &self,
        expr: &Expr,
        locals: &HumanLocalContext,
        delta: &[String],
        span: Span,
    ) -> HumanResult<Expr> {
        self.env
            .whnf(&locals.to_kernel_ctx(), delta, expr)
            .map_err(|err| human_kernel_expr_diagnostic(span, err, "Human weak-head reduction"))
    }

    fn is_human_defeq(
        &self,
        lhs: &Expr,
        rhs: &Expr,
        locals: &HumanLocalContext,
        delta: &[String],
        span: Span,
    ) -> HumanResult<bool> {
        self.env
            .is_defeq(&locals.to_kernel_ctx(), delta, lhs, rhs)
            .map_err(|err| human_kernel_expr_diagnostic(span, err, "Human definitional equality"))
    }

    fn add_kernel_decl(&mut self, decl: Decl, span: Span) -> HumanResult<()> {
        if let Some(existing) = self.env.decl(decl.name()) {
            if existing == &decl {
                return Ok(());
            }
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::KernelRejected,
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
        .map_err(|err| human_kernel_decl_diagnostic(span, err, "Human declaration handoff"))
    }
}

#[derive(Clone, Debug, Default)]
struct HumanLoweringLocalContext {
    locals: Vec<HumanLoweringLocalDecl>,
}

#[derive(Clone, Debug)]
struct HumanLoweringLocalDecl {
    name: String,
    ty: MachineTerm,
    value: Option<MachineTerm>,
}

impl HumanLoweringLocalContext {
    fn push_assumption(&mut self, name: String, ty: MachineTerm) {
        self.locals.push(HumanLoweringLocalDecl {
            name,
            ty,
            value: None,
        });
    }

    fn push_definition(&mut self, name: String, ty: MachineTerm, value: MachineTerm) {
        self.locals.push(HumanLoweringLocalDecl {
            name,
            ty,
            value: Some(value),
        });
    }

    fn meta_snapshot(&self) -> HumanMetaContextSnapshot {
        HumanMetaContextSnapshot {
            locals: self
                .locals
                .iter()
                .map(|local| HumanMetaLocalSnapshot {
                    ty: canonicalize_machine_term_for_meta_context(&local.ty),
                    value: local
                        .value
                        .as_ref()
                        .map(canonicalize_machine_term_for_meta_context),
                })
                .collect(),
        }
    }

    fn hole_goal_context(&self) -> Vec<HumanHoleGoalLocal> {
        self.locals
            .iter()
            .map(|local| HumanHoleGoalLocal {
                name: local.name.clone(),
                ty: render_machine_term(&local.ty),
                value: local.value.as_ref().map(render_machine_term),
            })
            .collect()
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
                        HumanDiagnosticKind::UnknownIdentifier,
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
            .map_err(|err| human_kernel_expr_diagnostic(span, err, "Human implicit inference"))
    }

    fn add_kernel_decl(&mut self, decl: Decl, span: Span) -> HumanResult<()> {
        if let Some(existing) = self.env.decl(decl.name()) {
            if existing == &decl {
                return Ok(());
            }
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::KernelRejected,
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
        .map_err(|err| human_kernel_decl_diagnostic(span, err, "Human implicit environment"))
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

fn human_kernel_expr_diagnostic(span: Span, err: Error, context: &str) -> HumanDiagnostic {
    match err {
        Error::ExpectedPi { actual } => HumanDiagnostic::error(
            HumanDiagnosticKind::ExpectedFunctionType,
            span,
            format!("{context}: expected a function type, got {actual:?}"),
        ),
        Error::ExpectedSort { actual } => HumanDiagnostic::error(
            HumanDiagnosticKind::ExpectedSort,
            span,
            format!("{context}: expected a type, got {actual:?}"),
        ),
        Error::TypeMismatch { expected, actual } => HumanDiagnostic::error(
            HumanDiagnosticKind::TypeMismatch,
            span,
            format!("{context}: expected {expected:?}, got {actual:?}"),
        ),
        Error::UnknownConstant(name) => HumanDiagnostic::error(
            HumanDiagnosticKind::UnknownIdentifier,
            span,
            format!("{context}: unknown global name {name}"),
        ),
        err => HumanDiagnostic::error(
            HumanDiagnosticKind::KernelRejected,
            span,
            format!("{context}: kernel rejected elaborated Human expression: {err:?}"),
        ),
    }
}

fn human_kernel_decl_diagnostic(span: Span, err: Error, context: &str) -> HumanDiagnostic {
    match err {
        Error::ExpectedPi { actual } => HumanDiagnostic::error(
            HumanDiagnosticKind::ExpectedFunctionType,
            span,
            format!("{context}: expected a function type, got {actual:?}"),
        ),
        Error::ExpectedSort { actual } => HumanDiagnostic::error(
            HumanDiagnosticKind::ExpectedSort,
            span,
            format!("{context}: expected a declaration type, got {actual:?}"),
        ),
        Error::TypeMismatch { expected, actual } => HumanDiagnostic::error(
            HumanDiagnosticKind::TypeMismatch,
            span,
            format!("{context}: declaration value has type {actual:?}, expected {expected:?}"),
        ),
        err => HumanDiagnostic::error(
            HumanDiagnosticKind::KernelRejected,
            span,
            format!("{context}: kernel rejected elaborated Human declaration: {err:?}"),
        ),
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

fn take_expected_pi_binder(expected: MachineTerm) -> Option<(MachineBinder, MachineTerm)> {
    let MachineTerm::Pi {
        mut binders,
        body,
        span,
    } = expected
    else {
        return None;
    };
    if binders.is_empty() {
        return None;
    }
    let binder = binders.remove(0);
    let rest = if binders.is_empty() {
        *body
    } else {
        MachineTerm::Pi {
            binders,
            body,
            span,
        }
    };
    Some((binder, rest))
}

fn rename_machine_local(term: MachineTerm, from: &str, to: &str) -> MachineTerm {
    rename_machine_local_scoped(term, from, to, false)
}

fn rename_machine_local_scoped(
    term: MachineTerm,
    from: &str,
    to: &str,
    shadowed: bool,
) -> MachineTerm {
    match term {
        MachineTerm::Ident {
            name,
            universe_args,
            explicit_mode,
            span,
        } => MachineTerm::Ident {
            name,
            universe_args,
            explicit_mode,
            span,
        },
        MachineTerm::Local { name, span } if !shadowed && name == from => MachineTerm::Local {
            name: to.to_owned(),
            span,
        },
        MachineTerm::Local { name, span } => MachineTerm::Local { name, span },
        MachineTerm::Prop { span } => MachineTerm::Prop { span },
        MachineTerm::Type { level, span } => MachineTerm::Type { level, span },
        MachineTerm::Sort { level, span } => MachineTerm::Sort { level, span },
        MachineTerm::App { func, arg, span } => MachineTerm::App {
            func: Box::new(rename_machine_local_scoped(*func, from, to, shadowed)),
            arg: Box::new(rename_machine_local_scoped(*arg, from, to, shadowed)),
            span,
        },
        MachineTerm::Lam {
            binders,
            body,
            span,
        } => {
            let (binders, body_shadowed) =
                rename_machine_binders_scoped(binders, from, to, shadowed);
            MachineTerm::Lam {
                binders,
                body: Box::new(rename_machine_local_scoped(*body, from, to, body_shadowed)),
                span,
            }
        }
        MachineTerm::Pi {
            binders,
            body,
            span,
        } => {
            let (binders, body_shadowed) =
                rename_machine_binders_scoped(binders, from, to, shadowed);
            MachineTerm::Pi {
                binders,
                body: Box::new(rename_machine_local_scoped(*body, from, to, body_shadowed)),
                span,
            }
        }
        MachineTerm::Let {
            name,
            ty,
            value,
            body,
            span,
        } => MachineTerm::Let {
            body: Box::new(rename_machine_local_scoped(
                *body,
                from,
                to,
                shadowed || name == from,
            )),
            name,
            ty: Box::new(rename_machine_local_scoped(*ty, from, to, shadowed)),
            value: Box::new(rename_machine_local_scoped(*value, from, to, shadowed)),
            span,
        },
        MachineTerm::Annot { expr, ty, span } => MachineTerm::Annot {
            expr: Box::new(rename_machine_local_scoped(*expr, from, to, shadowed)),
            ty: Box::new(rename_machine_local_scoped(*ty, from, to, shadowed)),
            span,
        },
    }
}

fn rename_machine_binders_scoped(
    binders: Vec<MachineBinder>,
    from: &str,
    to: &str,
    mut shadowed: bool,
) -> (Vec<MachineBinder>, bool) {
    let binders = binders
        .into_iter()
        .map(|binder| {
            let ty = rename_machine_local_scoped(binder.ty, from, to, shadowed);
            if binder.name == from {
                shadowed = true;
            }
            MachineBinder {
                name: binder.name,
                ty,
                span: binder.span,
            }
        })
        .collect();
    (binders, shadowed)
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
    meta_store: HumanMetaStore,
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
            meta_store: HumanMetaStore::default(),
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
        self.meta_store.begin_declaration();
        let mut local_context = HumanLoweringLocalContext::default();
        let binders = self.lower_binders(decl.binders, &mut local_context)?;
        let ty = self.lower_expr(decl.ty, &mut local_context, None)?;
        let value = self.lower_expr(decl.value, &mut local_context, Some(&ty))?;
        self.meta_store.reject_unsolved_for_decl(decl.span)?;

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
            binders,
            ty,
            value,
            span: decl.span,
        })
    }

    fn lower_binders(
        &mut self,
        binders: Vec<HumanBinder>,
        context: &mut HumanLoweringLocalContext,
    ) -> HumanResult<Vec<MachineBinder>> {
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
                let machine_name = name.as_dotted();
                let ty = self.lower_expr(*ty, context, None)?;
                context.push_assumption(machine_name.clone(), ty.clone());
                Ok(MachineBinder {
                    name: machine_name,
                    ty,
                    span: binder.span,
                })
            })
            .collect()
    }

    fn lower_lambda_binders(
        &mut self,
        binders: Vec<HumanBinder>,
        context: &mut HumanLoweringLocalContext,
        expected: Option<&MachineTerm>,
    ) -> HumanResult<(Vec<MachineBinder>, Option<MachineTerm>)> {
        let mut expected = expected.cloned();
        let mut lowered = Vec::with_capacity(binders.len());

        for binder in binders {
            let name = match binder.kind {
                HumanBinderKind::Named(name) => name.as_dotted(),
                HumanBinderKind::Anonymous => "_".to_owned(),
            };
            let (expected_binder, expected_body) = match expected.take() {
                Some(expected_term) => take_expected_pi_binder(expected_term),
                None => None,
            }
            .map_or((None, None), |(binder, body)| (Some(binder), Some(body)));

            let ty = match binder.ty {
                Some(ty) => self.lower_expr(*ty, context, None)?,
                None => {
                    let Some(expected_binder) = &expected_binder else {
                        return Err(HumanDiagnostic::error(
                            HumanDiagnosticKind::ExpectedFunctionType,
                            binder.span,
                            "unannotated Human lambda binder requires an expected function type",
                        ));
                    };
                    expected_binder.ty.clone()
                }
            };

            expected = match (expected_binder, expected_body) {
                (Some(expected_binder), Some(body)) => {
                    Some(rename_machine_local(body, &expected_binder.name, &name))
                }
                _ => None,
            };

            context.push_assumption(name.clone(), ty.clone());
            lowered.push(MachineBinder {
                name,
                ty,
                span: binder.span,
            });
        }

        Ok((lowered, expected))
    }

    fn lower_expr(
        &mut self,
        expr: HumanExpr,
        context: &mut HumanLoweringLocalContext,
        expected: Option<&MachineTerm>,
    ) -> HumanResult<MachineTerm> {
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
                func: Box::new(self.lower_expr(*func, context, None)?),
                arg: Box::new(self.lower_expr(*arg, context, None)?),
                span,
            },
            HumanExpr::Lam {
                binders,
                body,
                span,
            } => {
                let mut nested = context.clone();
                let (binders, body_expected) =
                    self.lower_lambda_binders(binders, &mut nested, expected)?;
                MachineTerm::Lam {
                    binders,
                    body: Box::new(self.lower_expr(*body, &mut nested, body_expected.as_ref())?),
                    span,
                }
            }
            HumanExpr::Pi {
                binders,
                body,
                span,
            } => {
                let mut nested = context.clone();
                MachineTerm::Pi {
                    binders: self.lower_binders(binders, &mut nested)?,
                    body: Box::new(self.lower_expr(*body, &mut nested, None)?),
                    span,
                }
            }
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
                let ty = self.lower_expr(*ty, context, None)?;
                let value = self.lower_expr(*value, context, Some(&ty))?;
                let mut nested = context.clone();
                nested.push_definition(name.as_dotted(), ty.clone(), value.clone());
                MachineTerm::Let {
                    name: name.as_dotted(),
                    ty: Box::new(ty),
                    value: Box::new(value),
                    body: Box::new(self.lower_expr(*body, &mut nested, expected)?),
                    span,
                }
            }
            HumanExpr::Annot { expr, ty, span } => {
                let ty = self.lower_expr(*ty, context, None)?;
                MachineTerm::Annot {
                    expr: Box::new(self.lower_expr(*expr, context, Some(&ty))?),
                    ty: Box::new(ty),
                    span,
                }
            }
            HumanExpr::Arrow {
                domain,
                codomain,
                span,
            } => MachineTerm::Pi {
                binders: vec![MachineBinder {
                    name: "_".to_owned(),
                    ty: self.lower_expr(*domain, context, None)?,
                    span,
                }],
                body: Box::new(self.lower_expr(*codomain, context, None)?),
                span,
            },
            HumanExpr::Hole { name, span } => {
                let id = self
                    .meta_store
                    .fresh_user_hole(name.as_ref(), context, expected, span)?;
                human_meta_placeholder(id, span)
            }
            HumanExpr::NotationApp { head, args, span } => {
                let lowered_args = args
                    .into_iter()
                    .map(|arg| self.lower_expr(arg, context, None))
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::{FileId, HumanDiagnosticKind, MachineDiagnosticKind};
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

        assert_eq!(err.kind, HumanDiagnosticKind::TypeMismatch);
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
    fn human_expected_type_elaborates_unannotated_lambda_to_core_declaration() {
        let imports = [nat_import()];
        let module = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Nat.Basic
def id : forall (x : Nat), Nat := fun x => x",
            &imports,
            &HumanCompileOptions::default(),
        )
        .expect("Human checker should use the expected Pi type for lambda binders");

        assert_eq!(
            module.declarations,
            vec![Decl::Def {
                name: "id".to_owned(),
                universe_params: Vec::new(),
                ty: Expr::pi("x", nat(), nat()),
                value: Expr::lam("x", nat(), Expr::bvar(0)),
                reducibility: Reducibility::Reducible,
            }]
        );
    }

    #[test]
    fn human_ill_typed_term_returns_structured_type_mismatch() {
        let imports = [nat_import()];
        let err = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Nat.Basic
def bad : Nat := Type",
            &imports,
            &HumanCompileOptions::default(),
        )
        .expect_err("ill-typed Human value should be rejected as a structured diagnostic");

        assert_eq!(err.kind, HumanDiagnosticKind::TypeMismatch);
    }

    #[test]
    fn human_unannotated_lambda_requires_expected_function_type() {
        let err = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "def bad : Type := fun x => x",
            &[],
            &HumanCompileOptions::default(),
        )
        .expect_err("unannotated lambda should not trigger open-ended search");

        assert_eq!(err.kind, HumanDiagnosticKind::ExpectedFunctionType);
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

    #[test]
    fn human_anonymous_hole_returns_goal_diagnostic_payload() {
        let imports = [nat_import()];
        let err = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Nat.Basic
def bad (n : Nat) : Nat := _",
            &imports,
            &HumanCompileOptions::default(),
        )
        .expect_err("unresolved Human hole should stop before Machine elaboration");

        assert_eq!(err.kind, HumanDiagnosticKind::UnsolvedHole);
        let payload = err.payload.expect("hole diagnostic should carry a payload");
        assert_eq!(payload.hole_goals.len(), 1);
        let goal = &payload.hole_goals[0];
        assert_eq!(goal.hole, None);
        assert_eq!(goal.target.as_deref(), Some("Nat"));
        assert_eq!(goal.context.len(), 1);
        assert_eq!(goal.context[0].name, "n");
        assert_eq!(goal.context[0].ty, "Nat");
    }

    #[test]
    fn human_unresolved_hole_rejects_certificate_path_before_certificate_output() {
        let err = compile_human_source_to_certificate(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "def bad : Type := _",
            &[],
            &HumanCompileOptions::default(),
        )
        .expect_err("unresolved Human hole should not reach certificate construction");

        assert_eq!(err.kind, HumanDiagnosticKind::UnsolvedHole);
    }

    #[test]
    fn human_named_hole_reuse_requires_same_context_snapshot() {
        let imports = [nat_import()];
        let err = compile_human_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Nat.Basic
def bad_named_hole : Nat := let x : Nat := ?m in ?m",
            &imports,
            &HumanCompileOptions::default(),
        )
        .expect_err("same named hole under a different context should be rejected");

        assert_eq!(err.kind, HumanDiagnosticKind::NamedHoleContextMismatch);
        let payload = err
            .payload
            .expect("context mismatch should carry both hole contexts");
        assert_eq!(payload.hole_goals.len(), 2);
        assert_eq!(payload.hole_goals[0].hole.as_deref(), Some("?m"));
        assert_eq!(payload.hole_goals[0].context.len(), 0);
        assert_eq!(payload.hole_goals[1].hole.as_deref(), Some("?m"));
        assert_eq!(payload.hole_goals[1].context.len(), 1);
        assert_eq!(payload.hole_goals[1].context[0].name, "x");
    }

    #[test]
    fn machine_path_still_rejects_holes_before_ast_elaboration() {
        let err = crate::parse_machine_module(FileId(0), "def Test.bad : Prop := _")
            .expect_err("Machine Surface Complete path should reject holes");

        assert_eq!(err.kind, MachineDiagnosticKind::HoleNotAllowed);
    }

    #[test]
    fn human_meta_store_solves_simple_term_and_universe_constraints() {
        let span = Span::empty(FileId(0));
        let mut store = HumanMetaStore::default();
        let context = HumanLoweringLocalContext::default();
        let term_meta = store
            .fresh_user_hole(None, &context, None, span)
            .expect("hole meta should allocate");
        let universe_meta = store.fresh_universe_meta(span);

        store.add_constraint(HumanConstraint::TermEq {
            lhs: HumanMetaExpr::Meta(term_meta),
            rhs: HumanMetaExpr::Core(nat()),
            span,
        });
        store.add_constraint(HumanConstraint::TypeEq {
            lhs: HumanMetaExpr::App(
                Box::new(HumanMetaExpr::Core(Expr::konst("F", vec![]))),
                Box::new(HumanMetaExpr::Meta(term_meta)),
            ),
            rhs: HumanMetaExpr::App(
                Box::new(HumanMetaExpr::Core(Expr::konst("F", vec![]))),
                Box::new(HumanMetaExpr::Core(nat())),
            ),
            span,
        });
        store.add_constraint(HumanConstraint::LevelEq {
            lhs: HumanMetaLevel::Succ(Box::new(HumanMetaLevel::Meta(universe_meta))),
            rhs: HumanMetaLevel::Succ(Box::new(HumanMetaLevel::Core(Level::zero()))),
            span,
        });
        store.add_constraint(HumanConstraint::LevelEq {
            lhs: HumanMetaLevel::Max(
                Box::new(HumanMetaLevel::Core(Level::zero())),
                Box::new(HumanMetaLevel::Core(Level::zero())),
            ),
            rhs: HumanMetaLevel::Max(
                Box::new(HumanMetaLevel::Core(Level::zero())),
                Box::new(HumanMetaLevel::Core(Level::zero())),
            ),
            span,
        });
        store.add_constraint(HumanConstraint::LevelLe {
            lhs: HumanMetaLevel::Core(Level::zero()),
            rhs: HumanMetaLevel::Core(type0()),
            span,
        });

        store
            .solve_constraints()
            .expect("simple constraints should solve");
        assert_eq!(
            store.term_metas[term_meta.0 as usize].assignment,
            Some(HumanMetaExpr::Core(nat()))
        );
        assert_eq!(
            store.universe_metas[universe_meta.0 as usize].assignment,
            Some(HumanMetaLevel::Core(Level::zero()))
        );
    }

    #[test]
    fn human_meta_store_rejects_occurs_check_cycles() {
        let span = Span::empty(FileId(0));
        let mut store = HumanMetaStore::default();
        let context = HumanLoweringLocalContext::default();
        let term_meta = store
            .fresh_user_hole(None, &context, None, span)
            .expect("hole meta should allocate");
        store.add_constraint(HumanConstraint::TermEq {
            lhs: HumanMetaExpr::Meta(term_meta),
            rhs: HumanMetaExpr::App(
                Box::new(HumanMetaExpr::Meta(term_meta)),
                Box::new(HumanMetaExpr::Core(nat())),
            ),
            span,
        });

        let err = store
            .solve_constraints()
            .expect_err("cyclic assignment should fail occurs check");
        assert_eq!(err.kind, HumanDiagnosticKind::OccursCheckFailed);
    }
}
