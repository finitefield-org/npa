use crate::{
    elaborator::elaborate_machine_module, parse_human_module, resolve_human_module,
    resolver::resolve_machine_module_with_options, HumanBinder, HumanBinderKind,
    HumanCompileOptions, HumanDiagnostic, HumanDiagnosticKind, HumanExpr, HumanGlobalRef,
    HumanImplicitMode, HumanItem, HumanLevel, HumanResolvedName, HumanResolvedNameUse,
    HumanResolvedNotationUse, HumanResult, MachineBinder, MachineCompileOptions, MachineDecl,
    MachineItem, MachineLevel, MachineModule, MachineName, MachineTerm, ResolvedHumanModule, Span,
    VerifiedImport,
};

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
    let mut lowering = HumanToMachineLowering::new(module, notation_plan);
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

struct HumanToMachineLowering<'a> {
    name_uses: std::slice::Iter<'a, HumanResolvedNameUse>,
    notation_uses: std::slice::Iter<'a, HumanResolvedNotationUse>,
    notation_choices: std::slice::Iter<'a, usize>,
}

impl<'a> HumanToMachineLowering<'a> {
    fn new(module: &'a ResolvedHumanModule, notation_plan: &'a [usize]) -> Self {
        Self {
            name_uses: module.resolved_names.iter(),
            notation_uses: module.resolved_notations.iter(),
            notation_choices: notation_plan.iter(),
        }
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
                    machine_items.push(MachineItem::Def(
                        self.lower_decl(decl.clone(), &metadata.name)?,
                    ));
                }
                HumanItem::Theorem(decl) => {
                    let metadata = declarations.next().ok_or_else(|| {
                        HumanDiagnostic::not_implemented(decl.span, "Human declaration metadata")
                    })?;
                    machine_items.push(MachineItem::Theorem(
                        self.lower_decl(decl.clone(), &metadata.name)?,
                    ));
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
        resolved_name: &crate::HumanName,
    ) -> HumanResult<MachineDecl> {
        Ok(MachineDecl {
            name: machine_name(resolved_name.clone()),
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
    use super::*;
    use crate::{FileId, HumanDiagnosticKind};
    use npa_kernel::{Decl, Expr, Level, Reducibility};

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
}
