use std::collections::{BTreeMap, BTreeSet};

use crate::{
    builtins::{eq_inductive, nat_inductive},
    context::Ctx,
    decl::{ConstructorDecl, Decl, InductiveDecl, RecursorDecl, RecursorRules, Reducibility},
    error::{Error, Result},
    expr::{collect_apps, Expr},
    level::{ensure_level_wf, level_eq, levels_eq, Level},
    subst::{instantiate, subst_levels_expr},
};

#[derive(Clone, Debug, Default)]
pub struct Env {
    decls: BTreeMap<String, Decl>,
}

impl Env {
    const WHNF_FUEL: usize = 20_000;
    const DEFEQ_FUEL: usize = 20_000;

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_builtins() -> Result<Self> {
        let mut env = Self::new();
        env.add_inductive(nat_inductive())?;
        env.add_inductive(eq_inductive())?;
        Ok(env)
    }

    pub fn decl(&self, name: &str) -> Option<&Decl> {
        self.decls.get(name)
    }

    pub fn add_axiom(
        &mut self,
        name: impl Into<String>,
        universe_params: Vec<String>,
        ty: Expr,
    ) -> Result<()> {
        let name = name.into();
        self.ensure_fresh(&name)?;
        let delta = validate_universe_params(&universe_params)?;
        self.expect_sort(&Ctx::new(), &delta, &ty)?;
        self.decls.insert(
            name.clone(),
            Decl::Axiom {
                name,
                universe_params,
                ty,
            },
        );
        Ok(())
    }

    pub fn add_def(
        &mut self,
        name: impl Into<String>,
        universe_params: Vec<String>,
        ty: Expr,
        value: Expr,
        reducibility: Reducibility,
    ) -> Result<()> {
        let name = name.into();
        self.ensure_fresh(&name)?;
        let delta = validate_universe_params(&universe_params)?;
        self.expect_sort(&Ctx::new(), &delta, &ty)?;
        self.check(&Ctx::new(), &delta, &value, &ty)?;
        self.decls.insert(
            name.clone(),
            Decl::Def {
                name,
                universe_params,
                ty,
                value,
                reducibility,
            },
        );
        Ok(())
    }

    pub fn add_theorem(
        &mut self,
        name: impl Into<String>,
        universe_params: Vec<String>,
        ty: Expr,
        proof: Expr,
    ) -> Result<()> {
        let name = name.into();
        self.ensure_fresh(&name)?;
        let delta = validate_universe_params(&universe_params)?;
        self.expect_sort(&Ctx::new(), &delta, &ty)?;
        self.check(&Ctx::new(), &delta, &proof, &ty)?;
        self.decls.insert(
            name.clone(),
            Decl::Theorem {
                name,
                universe_params,
                ty,
                proof,
            },
        );
        Ok(())
    }

    pub fn add_inductive(&mut self, data: InductiveDecl) -> Result<()> {
        let delta = validate_universe_params(&data.universe_params)?;
        ensure_level_wf(&delta, &data.sort)?;
        self.ensure_inductive_names_fresh(&data)?;

        let ty = inductive_type(&data);
        self.expect_sort(&Ctx::new(), &delta, &ty)?;

        let mut candidate = self.clone();
        candidate.decls.insert(
            data.name.clone(),
            Decl::Inductive {
                name: data.name.clone(),
                universe_params: data.universe_params.clone(),
                ty,
                data: Box::new(data.clone()),
            },
        );

        for constructor in &data.constructors {
            candidate.check_constructor_decl(&data, constructor, &delta)?;
            candidate.decls.insert(
                constructor.name.clone(),
                Decl::Constructor {
                    name: constructor.name.clone(),
                    universe_params: data.universe_params.clone(),
                    ty: constructor.ty.clone(),
                    inductive: data.name.clone(),
                },
            );
        }

        if let Some(recursor) = &data.recursor {
            let recursor_delta = validate_universe_params(&recursor.universe_params)?;
            candidate.expect_sort(&Ctx::new(), &recursor_delta, &recursor.ty)?;
            let rules = recursor
                .rules
                .clone()
                .unwrap_or_else(|| generated_recursor_rules(&data));
            candidate.check_recursor_decl(&data, recursor, &rules, &recursor_delta)?;
            candidate.decls.insert(
                recursor.name.clone(),
                Decl::Recursor {
                    name: recursor.name.clone(),
                    universe_params: recursor.universe_params.clone(),
                    ty: recursor.ty.clone(),
                    inductive: data.name.clone(),
                    rules,
                },
            );
        }

        *self = candidate;
        Ok(())
    }

    pub fn infer(&self, ctx: &Ctx, delta: &[String], term: &Expr) -> Result<Expr> {
        match term {
            Expr::Sort(level) => {
                ensure_level_wf(delta, level)?;
                Ok(Expr::sort(Level::succ(level.clone())))
            }
            Expr::BVar(index) => ctx.lookup_type(*index),
            Expr::Const { name, levels } => {
                let decl = self
                    .decls
                    .get(name)
                    .ok_or_else(|| Error::UnknownConstant(name.clone()))?;
                let params = decl.universe_params();
                if params.len() != levels.len() {
                    return Err(Error::BadUniverseArity {
                        name: name.clone(),
                        expected: params.len(),
                        actual: levels.len(),
                    });
                }
                for level in levels {
                    ensure_level_wf(delta, level)?;
                }
                Ok(subst_levels_expr(decl.ty(), params, levels))
            }
            Expr::Pi { binder, ty, body } => {
                let domain_sort = self.expect_sort(ctx, delta, ty)?;
                let mut body_ctx = ctx.clone();
                body_ctx.push_assumption(binder.clone(), (**ty).clone());
                let body_sort = self.expect_sort(&body_ctx, delta, body)?;
                Ok(Expr::sort(Level::imax(domain_sort, body_sort)))
            }
            Expr::Lam { binder, ty, body } => {
                self.expect_sort(ctx, delta, ty)?;
                let mut body_ctx = ctx.clone();
                body_ctx.push_assumption(binder.clone(), (**ty).clone());
                let body_ty = self.infer(&body_ctx, delta, body)?;
                Ok(Expr::pi(binder.clone(), (**ty).clone(), body_ty))
            }
            Expr::App(fun, arg) => {
                let fun_ty = self.infer(ctx, delta, fun)?;
                match self.whnf(ctx, delta, &fun_ty)? {
                    Expr::Pi { ty, body, .. } => {
                        self.check(ctx, delta, arg, &ty)?;
                        instantiate(&body, arg)
                    }
                    actual => Err(Error::ExpectedPi { actual }),
                }
            }
            Expr::Let {
                binder,
                ty,
                value,
                body,
            } => {
                self.expect_sort(ctx, delta, ty)?;
                self.check(ctx, delta, value, ty)?;
                let mut body_ctx = ctx.clone();
                body_ctx.push_definition(binder.clone(), (**ty).clone(), (**value).clone());
                let body_ty = self.infer(&body_ctx, delta, body)?;
                instantiate(&body_ty, value)
            }
        }
    }

    pub fn check(&self, ctx: &Ctx, delta: &[String], term: &Expr, expected: &Expr) -> Result<()> {
        let actual = self.infer(ctx, delta, term)?;
        if self.is_defeq(ctx, delta, &actual, expected)? {
            Ok(())
        } else {
            Err(Error::TypeMismatch {
                expected: expected.clone(),
                actual,
            })
        }
    }

    pub fn whnf(&self, ctx: &Ctx, delta: &[String], term: &Expr) -> Result<Expr> {
        self.whnf_with_fuel(ctx, delta, term, Self::WHNF_FUEL)
    }

    pub fn is_defeq(&self, ctx: &Ctx, delta: &[String], lhs: &Expr, rhs: &Expr) -> Result<bool> {
        self.is_defeq_with_fuel(ctx, delta, lhs, rhs, Self::DEFEQ_FUEL)
    }

    fn ensure_fresh(&self, name: &str) -> Result<()> {
        if self.decls.contains_key(name) {
            Err(Error::DuplicateDecl(name.to_owned()))
        } else {
            Ok(())
        }
    }

    fn ensure_inductive_names_fresh(&self, data: &InductiveDecl) -> Result<()> {
        let mut names = BTreeSet::new();
        for name in std::iter::once(&data.name)
            .chain(
                data.constructors
                    .iter()
                    .map(|constructor| &constructor.name),
            )
            .chain(data.recursor.iter().map(|recursor| &recursor.name))
        {
            if !names.insert(name) {
                return Err(Error::DuplicateDecl(name.clone()));
            }
            self.ensure_fresh(name)?;
        }
        Ok(())
    }

    fn expect_sort(&self, ctx: &Ctx, delta: &[String], term: &Expr) -> Result<Level> {
        match self.whnf(ctx, delta, &self.infer(ctx, delta, term)?)? {
            Expr::Sort(level) => Ok(level),
            actual => Err(Error::ExpectedSort { actual }),
        }
    }

    fn check_constructor_decl(
        &self,
        data: &InductiveDecl,
        constructor: &ConstructorDecl,
        delta: &[String],
    ) -> Result<()> {
        self.expect_sort(&Ctx::new(), delta, &constructor.ty)?;
        let (domains, result) = peel_pi_domains(&constructor.ty);
        for domain in domains {
            check_constructor_domain_positive(&data.name, &constructor.name, &domain)?;
        }

        let result = self.whnf(&Ctx::new(), delta, &result)?;
        let (head, _) = collect_apps(&result);
        match head {
            Expr::Const { name, .. } if name == data.name => Ok(()),
            _ => Err(Error::BadConstructorResult {
                inductive: data.name.clone(),
                constructor: constructor.name.clone(),
                result,
            }),
        }
    }

    fn check_recursor_decl(
        &self,
        data: &InductiveDecl,
        recursor: &RecursorDecl,
        rules: &RecursorRules,
        delta: &[String],
    ) -> Result<()> {
        if rules.minor_start != data.params.len() + 1 {
            return Err(Error::InvalidInductive(format!(
                "{} recursor minor_start must be params + motive",
                data.name
            )));
        }
        if rules.major_index != rules.minor_start + data.constructors.len() {
            return Err(Error::InvalidInductive(format!(
                "{} recursor major_index must follow constructor minor premises",
                data.name
            )));
        }

        let (domains, result) = peel_pi_domains(&recursor.ty);
        if domains.len() <= rules.major_index {
            return Err(Error::InvalidInductive(format!(
                "{} recursor has no major premise",
                recursor.name
            )));
        }

        for param in &data.params {
            if domains.is_empty() {
                return Err(Error::InvalidInductive(format!(
                    "{} recursor is missing parameter binders",
                    recursor.name
                )));
            }
            self.expect_sort(&Ctx::new(), delta, &param.ty)?;
        }

        let motive_domain = domains.get(data.params.len()).ok_or_else(|| {
            Error::InvalidInductive(format!("{} recursor is missing motive", recursor.name))
        })?;
        self.check_motive_domain(data, recursor, motive_domain)?;

        let major_domain = &domains[rules.major_index];
        self.check_recursor_target(data, recursor, major_domain, "major premise")?;
        self.check_recursor_result(data, recursor, &result)?;

        for (constructor_index, constructor) in data.constructors.iter().enumerate() {
            let minor_domain = &domains[rules.minor_start + constructor_index];
            let (minor_fields, minor_result) = peel_pi_domains(minor_domain);
            let (constructor_fields, _) = peel_pi_domains(&constructor.ty);
            if minor_fields.len() < constructor_fields.len().saturating_sub(data.params.len()) {
                return Err(Error::InvalidInductive(format!(
                    "{} minor premise for {} has too few fields",
                    recursor.name, constructor.name
                )));
            }
            self.check_motive_application_result(data, recursor, &minor_result)?;
        }

        Ok(())
    }

    fn check_motive_domain(
        &self,
        data: &InductiveDecl,
        recursor: &RecursorDecl,
        motive_domain: &Expr,
    ) -> Result<()> {
        let (motive_domains, motive_result) = peel_pi_domains(motive_domain);
        if motive_domains.len() != 1 {
            return Err(Error::InvalidInductive(format!(
                "{} motive must take one major premise in Phase 1",
                recursor.name
            )));
        }
        self.check_recursor_target(data, recursor, &motive_domains[0], "motive domain")?;
        if !matches!(motive_result, Expr::Sort(_)) {
            return Err(Error::InvalidInductive(format!(
                "{} motive must return a Sort",
                recursor.name
            )));
        }
        Ok(())
    }

    fn check_recursor_target(
        &self,
        data: &InductiveDecl,
        recursor: &RecursorDecl,
        target: &Expr,
        label: &str,
    ) -> Result<()> {
        let (head, _) = collect_apps(target);
        match head {
            Expr::Const { name, .. } if name == data.name => Ok(()),
            _ => Err(Error::InvalidInductive(format!(
                "{} {} must target {}",
                recursor.name, label, data.name
            ))),
        }
    }

    fn check_recursor_result(
        &self,
        data: &InductiveDecl,
        recursor: &RecursorDecl,
        result: &Expr,
    ) -> Result<()> {
        self.check_motive_application_result(data, recursor, result)
    }

    fn check_motive_application_result(
        &self,
        data: &InductiveDecl,
        recursor: &RecursorDecl,
        result: &Expr,
    ) -> Result<()> {
        let (head, args) = collect_apps(result);
        if args.len() != 1 {
            return Err(Error::InvalidInductive(format!(
                "{} result must apply motive to one major premise in Phase 1",
                recursor.name
            )));
        }
        match head {
            Expr::BVar(_) => self.check_result_target(data, recursor, &args[0]),
            _ => Err(Error::InvalidInductive(format!(
                "{} result must be motive application",
                recursor.name
            ))),
        }
    }

    fn check_result_target(
        &self,
        data: &InductiveDecl,
        recursor: &RecursorDecl,
        target: &Expr,
    ) -> Result<()> {
        let (head, _) = collect_apps(target);
        match head {
            Expr::BVar(_) => Ok(()),
            Expr::Const { name, .. } if self.constructor_belongs_to(&name, &data.name) => Ok(()),
            _ => Err(Error::InvalidInductive(format!(
                "{} result target must be major premise or constructor-headed {} value",
                recursor.name, data.name
            ))),
        }
    }

    fn whnf_with_fuel(
        &self,
        ctx: &Ctx,
        delta: &[String],
        term: &Expr,
        mut fuel: usize,
    ) -> Result<Expr> {
        let mut current = term.clone();
        loop {
            if fuel == 0 {
                return Err(Error::ResourceLimit);
            }
            fuel -= 1;

            match current {
                Expr::BVar(index) => {
                    if let Some(value) = ctx.lookup_value(index)? {
                        current = value;
                    } else {
                        return Ok(Expr::BVar(index));
                    }
                }
                Expr::Const {
                    ref name,
                    ref levels,
                } => {
                    if let Some(Decl::Def {
                        universe_params,
                        value,
                        reducibility: Reducibility::Reducible,
                        ..
                    }) = self.decls.get(name)
                    {
                        current = subst_levels_expr(value, universe_params, levels);
                    } else {
                        return Ok(current);
                    }
                }
                Expr::App(fun, arg) => {
                    let fun_whnf = self.whnf_with_fuel(ctx, delta, &fun, fuel)?;
                    if let Expr::Lam { body, .. } = fun_whnf {
                        current = instantiate(&body, &arg)?;
                        continue;
                    }

                    let app = Expr::app(fun_whnf, (*arg).clone());
                    if let Some(reduced) = self.reduce_recursor(ctx, delta, &app, fuel)? {
                        current = reduced;
                        continue;
                    }
                    return Ok(app);
                }
                Expr::Let { value, body, .. } => {
                    current = instantiate(&body, &value)?;
                }
                _ => return Ok(current),
            }
        }
    }

    fn reduce_recursor(
        &self,
        ctx: &Ctx,
        delta: &[String],
        term: &Expr,
        fuel: usize,
    ) -> Result<Option<Expr>> {
        let (head, args) = collect_apps(term);
        let Expr::Const {
            name: recursor_name,
            levels,
        } = head
        else {
            return Ok(None);
        };
        let Some(Decl::Recursor {
            inductive, rules, ..
        }) = self.decls.get(&recursor_name)
        else {
            return Ok(None);
        };
        if args.len() <= rules.major_index {
            return Ok(None);
        }

        let major = args[rules.major_index].clone();
        let rest = args[rules.major_index + 1..].to_vec();
        let major_whnf = self.whnf_with_fuel(ctx, delta, &major, fuel)?;
        let (ctor_head, ctor_args) = collect_apps(&major_whnf);
        let Expr::Const {
            name: ctor_name, ..
        } = ctor_head
        else {
            return Ok(None);
        };
        if !self.constructor_belongs_to(&ctor_name, inductive) {
            return Ok(None);
        }

        let data = self.inductive_data(inductive)?;
        let Some(ctor_index) = data
            .constructors
            .iter()
            .position(|constructor| constructor.name == ctor_name)
        else {
            return Ok(None);
        };
        let Some(minor) = args.get(rules.minor_start + ctor_index).cloned() else {
            return Ok(None);
        };

        let constructor = &data.constructors[ctor_index];
        let (domains, _) = peel_pi_domains(&constructor.ty);
        let param_count = data.params.len();
        if ctor_args.len() < param_count {
            return Ok(None);
        }
        let field_args = &ctor_args[param_count..];
        let field_domains = &domains[param_count..];
        if field_args.len() < field_domains.len() {
            return Ok(None);
        }

        let mut reduced = minor;
        for (field_arg, field_domain) in field_args.iter().zip(field_domains) {
            reduced = Expr::app(reduced, field_arg.clone());
            if is_direct_recursive_domain(inductive, field_domain) {
                let mut recursive_args = args[..rules.major_index].to_vec();
                recursive_args.push(field_arg.clone());
                reduced = Expr::app(
                    reduced,
                    Expr::apps(
                        Expr::konst(recursor_name.clone(), levels.clone()),
                        recursive_args,
                    ),
                );
            }
        }

        Ok(Some(Expr::apps(reduced, rest)))
    }

    fn constructor_belongs_to(&self, constructor: &str, inductive: &str) -> bool {
        matches!(
            self.decls.get(constructor),
            Some(Decl::Constructor {
                inductive: owner, ..
            }) if owner == inductive
        )
    }

    fn inductive_data(&self, name: &str) -> Result<&InductiveDecl> {
        match self.decls.get(name) {
            Some(Decl::Inductive { data, .. }) => Ok(data.as_ref()),
            _ => Err(Error::InvalidInductive(name.to_owned())),
        }
    }

    fn is_defeq_with_fuel(
        &self,
        ctx: &Ctx,
        delta: &[String],
        lhs: &Expr,
        rhs: &Expr,
        fuel: usize,
    ) -> Result<bool> {
        if fuel == 0 {
            return Err(Error::ResourceLimit);
        }

        let lhs = self.whnf_with_fuel(ctx, delta, lhs, fuel)?;
        let rhs = self.whnf_with_fuel(ctx, delta, rhs, fuel)?;

        match (&lhs, &rhs) {
            (Expr::Sort(lhs), Expr::Sort(rhs)) => Ok(level_eq(lhs, rhs)),
            (Expr::BVar(lhs), Expr::BVar(rhs)) => Ok(lhs == rhs),
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
            (Expr::App(lhs_f, lhs_a), Expr::App(rhs_f, rhs_a)) => {
                Ok(self.is_defeq_with_fuel(ctx, delta, lhs_f, rhs_f, fuel - 1)?
                    && self.is_defeq_with_fuel(ctx, delta, lhs_a, rhs_a, fuel - 1)?)
            }
            (
                Expr::Pi {
                    binder,
                    ty: lhs_ty,
                    body: lhs_body,
                },
                Expr::Pi {
                    ty: rhs_ty,
                    body: rhs_body,
                    ..
                },
            ) => {
                if !self.is_defeq_with_fuel(ctx, delta, lhs_ty, rhs_ty, fuel - 1)? {
                    return Ok(false);
                }
                let mut body_ctx = ctx.clone();
                body_ctx.push_assumption(binder.clone(), (**lhs_ty).clone());
                self.is_defeq_with_fuel(&body_ctx, delta, lhs_body, rhs_body, fuel - 1)
            }
            (
                Expr::Lam {
                    binder,
                    ty: lhs_ty,
                    body: lhs_body,
                },
                Expr::Lam {
                    ty: rhs_ty,
                    body: rhs_body,
                    ..
                },
            ) => {
                if !self.is_defeq_with_fuel(ctx, delta, lhs_ty, rhs_ty, fuel - 1)? {
                    return Ok(false);
                }
                let mut body_ctx = ctx.clone();
                body_ctx.push_assumption(binder.clone(), (**lhs_ty).clone());
                self.is_defeq_with_fuel(&body_ctx, delta, lhs_body, rhs_body, fuel - 1)
            }
            _ => Ok(false),
        }
    }
}

fn validate_universe_params(params: &[String]) -> Result<Vec<String>> {
    let mut seen = BTreeSet::new();
    for param in params {
        if !seen.insert(param.clone()) {
            return Err(Error::UnknownUniverseParam(param.clone()));
        }
    }
    Ok(params.to_vec())
}

fn generated_recursor_rules(data: &InductiveDecl) -> RecursorRules {
    let minor_start = data.params.len() + 1;
    RecursorRules::new(minor_start, minor_start + data.constructors.len())
}

fn inductive_type(data: &InductiveDecl) -> Expr {
    let binders = data.params.iter().chain(&data.indices);
    mk_pi_telescope(binders, Expr::sort(data.sort.clone()))
}

fn mk_pi_telescope<'a>(
    binders: impl DoubleEndedIterator<Item = &'a crate::Binder>,
    body: Expr,
) -> Expr {
    binders.rev().fold(body, |body, binder| {
        Expr::pi(binder.name.clone(), binder.ty.clone(), body)
    })
}

fn peel_pi_domains(ty: &Expr) -> (Vec<Expr>, Expr) {
    let mut domains = Vec::new();
    let mut current = ty.clone();
    while let Expr::Pi { ty, body, .. } = current {
        domains.push(*ty);
        current = *body;
    }
    (domains, current)
}

fn check_constructor_domain_positive(
    inductive: &str,
    constructor: &str,
    domain: &Expr,
) -> Result<()> {
    if is_direct_recursive_domain(inductive, domain) {
        return Ok(());
    }
    if contains_const(domain, inductive) {
        return Err(Error::NonPositiveOccurrence {
            inductive: inductive.to_owned(),
            constructor: constructor.to_owned(),
            ty: domain.clone(),
        });
    }
    Ok(())
}

fn is_direct_recursive_domain(inductive: &str, domain: &Expr) -> bool {
    let (head, _) = collect_apps(domain);
    matches!(head, Expr::Const { name, .. } if name == inductive)
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
