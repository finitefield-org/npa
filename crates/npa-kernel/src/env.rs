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
        for (domain_index, domain) in domains.iter().enumerate() {
            check_constructor_domain_positive(data, &constructor.name, domain_index, domain)?;
        }

        let result = self.whnf(&Ctx::new(), delta, &result)?;
        self.check_constructor_result(data, constructor, domains.len(), result)
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
        if domains.len() != rules.major_index + 1 {
            return Err(Error::InvalidInductive(format!(
                "{} recursor major premise must be the final binder in Phase 1",
                recursor.name
            )));
        }

        self.check_recursor_params(data, recursor, &domains, delta)?;

        let motive_domain = domains.get(data.params.len()).ok_or_else(|| {
            Error::InvalidInductive(format!("{} recursor is missing motive", recursor.name))
        })?;
        self.check_motive_domain(data, recursor, motive_domain)?;

        let major_domain = &domains[rules.major_index];
        self.check_recursor_target(data, recursor, major_domain, "major premise")?;
        self.check_recursor_result(data, recursor, &result)?;

        for (constructor_index, constructor) in data.constructors.iter().enumerate() {
            let minor_index = rules.minor_start + constructor_index;
            let minor_domain = &domains[rules.minor_start + constructor_index];
            let expected_minor = expected_minor_type(data, constructor, constructor_index)?;
            let prefix_ctx = recursor_prefix_ctx(&domains[..minor_index]);
            if !self.is_defeq(&prefix_ctx, delta, minor_domain, &expected_minor)? {
                return Err(Error::InvalidInductive(format!(
                    "{} minor premise for {} does not match constructor",
                    recursor.name, constructor.name
                )));
            }
        }

        Ok(())
    }

    fn check_constructor_result(
        &self,
        data: &InductiveDecl,
        constructor: &ConstructorDecl,
        domain_count: usize,
        result: Expr,
    ) -> Result<()> {
        let (head, args) = collect_apps(&result);
        let levels = match head {
            Expr::Const { name, levels } if name == data.name => levels,
            _ => {
                return Err(Error::BadConstructorResult {
                    inductive: data.name.clone(),
                    constructor: constructor.name.clone(),
                    result,
                })
            }
        };

        let expected_levels: Vec<_> = data
            .universe_params
            .iter()
            .map(|param| Level::param(param.clone()))
            .collect();
        if !levels_eq(&levels, &expected_levels)
            || args.len() != data.params.len() + data.indices.len()
            || domain_count < data.params.len()
        {
            return Err(Error::BadConstructorResult {
                inductive: data.name.clone(),
                constructor: constructor.name.clone(),
                result,
            });
        }

        for (param_index, arg) in args.iter().take(data.params.len()).enumerate() {
            let expected = Expr::bvar((domain_count - 1 - param_index) as u32);
            if arg != &expected {
                return Err(Error::BadConstructorResult {
                    inductive: data.name.clone(),
                    constructor: constructor.name.clone(),
                    result,
                });
            }
        }

        Ok(())
    }

    fn check_recursor_params(
        &self,
        data: &InductiveDecl,
        recursor: &RecursorDecl,
        domains: &[Expr],
        delta: &[String],
    ) -> Result<()> {
        if domains.len() < data.params.len() {
            return Err(Error::InvalidInductive(format!(
                "{} recursor is missing parameter binders",
                recursor.name
            )));
        }

        let mut ctx = Ctx::new();
        for (param_index, param) in data.params.iter().enumerate() {
            self.expect_sort(&ctx, delta, &param.ty)?;
            if !self.is_defeq(&ctx, delta, &domains[param_index], &param.ty)? {
                return Err(Error::InvalidInductive(format!(
                    "{} recursor parameter {} does not match inductive",
                    recursor.name, param.name
                )));
            }
            ctx.push_assumption(param.name.clone(), param.ty.clone());
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
        match motive_result {
            Expr::Sort(level) => {
                if level_eq(&data.sort, &Level::zero()) && !level_eq(&level, &Level::zero()) {
                    return Err(Error::InvalidInductive(format!(
                        "{} Prop recursor motive must return Prop",
                        recursor.name
                    )));
                }
            }
            _ => {
                return Err(Error::InvalidInductive(format!(
                    "{} motive must return a Sort",
                    recursor.name
                )))
            }
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
        for (field_index, (field_arg, field_domain)) in
            field_args.iter().zip(field_domains).enumerate()
        {
            reduced = Expr::app(reduced, field_arg.clone());
            if is_direct_recursive_domain(data, field_domain, param_count + field_index) {
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

fn recursor_prefix_ctx(domains: &[Expr]) -> Ctx {
    let mut ctx = Ctx::new();
    for (index, domain) in domains.iter().enumerate() {
        ctx.push_assumption(format!("_rec_arg_{index}"), domain.clone());
    }
    ctx
}

fn expected_minor_type(
    data: &InductiveDecl,
    constructor: &ConstructorDecl,
    constructor_index: usize,
) -> Result<Expr> {
    let (domains, _) = peel_pi_domains(&constructor.ty);
    let param_count = data.params.len();
    if domains.len() < param_count {
        return Err(Error::InvalidInductive(format!(
            "{} constructor is missing parameter binders",
            constructor.name
        )));
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
        )?);

        source_to_target.push(target_ctx_len);
        field_abs.push(target_ctx_len);
        target_ctx_len += 1;

        if is_direct_recursive_domain(data, field_domain, source_ctx_len) {
            expected_domains.push(motive_app(target_ctx_len, motive_abs, Expr::bvar(0))?);
            target_ctx_len += 1;
        }
    }

    let mut constructor_args = Vec::with_capacity(param_count + field_abs.len());
    for param_abs in 0..param_count {
        constructor_args.push(bvar_for_abs(target_ctx_len, param_abs)?);
    }
    for field_abs in field_abs {
        constructor_args.push(bvar_for_abs(target_ctx_len, field_abs)?);
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
    let result = motive_app(target_ctx_len, motive_abs, constructor_value)?;

    Ok(mk_pi_from_domains(expected_domains, result))
}

fn motive_app(ctx_len: usize, motive_abs: usize, target: Expr) -> Result<Expr> {
    Ok(Expr::app(bvar_for_abs(ctx_len, motive_abs)?, target))
}

fn bvar_for_abs(ctx_len: usize, abs: usize) -> Result<Expr> {
    if abs >= ctx_len {
        return Err(Error::InvalidInductive(format!(
            "binder index {abs} escapes context of length {ctx_len}"
        )));
    }
    Ok(Expr::bvar((ctx_len - 1 - abs) as u32))
}

fn mk_pi_from_domains(domains: Vec<Expr>, body: Expr) -> Expr {
    domains
        .into_iter()
        .rev()
        .fold(body, |body, domain| Expr::pi("_", domain, body))
}

fn remap_bvars(
    expr: &Expr,
    source_ctx_len: usize,
    target_ctx_len: usize,
    source_to_target: &[usize],
) -> Result<Expr> {
    match expr {
        Expr::Sort(level) => Ok(Expr::sort(level.clone())),
        Expr::BVar(index) => {
            let index = *index as usize;
            if index >= source_ctx_len {
                return Err(Error::InvalidInductive(format!(
                    "binder index {index} escapes context of length {source_ctx_len}"
                )));
            }
            let source_abs = source_ctx_len - 1 - index;
            let Some(target_abs) = source_to_target.get(source_abs).copied() else {
                return Err(Error::InvalidInductive(format!(
                    "binder index {index} has no target in recursor minor"
                )));
            };
            bvar_for_abs(target_ctx_len, target_abs)
        }
        Expr::Const { name, levels } => Ok(Expr::konst(name.clone(), levels.clone())),
        Expr::App(fun, arg) => Ok(Expr::app(
            remap_bvars(fun, source_ctx_len, target_ctx_len, source_to_target)?,
            remap_bvars(arg, source_ctx_len, target_ctx_len, source_to_target)?,
        )),
        Expr::Lam { binder, ty, body } => {
            let mut body_map = source_to_target.to_vec();
            body_map.push(target_ctx_len);
            Ok(Expr::lam(
                binder.clone(),
                remap_bvars(ty, source_ctx_len, target_ctx_len, source_to_target)?,
                remap_bvars(body, source_ctx_len + 1, target_ctx_len + 1, &body_map)?,
            ))
        }
        Expr::Pi { binder, ty, body } => {
            let mut body_map = source_to_target.to_vec();
            body_map.push(target_ctx_len);
            Ok(Expr::pi(
                binder.clone(),
                remap_bvars(ty, source_ctx_len, target_ctx_len, source_to_target)?,
                remap_bvars(body, source_ctx_len + 1, target_ctx_len + 1, &body_map)?,
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
                remap_bvars(ty, source_ctx_len, target_ctx_len, source_to_target)?,
                remap_bvars(value, source_ctx_len, target_ctx_len, source_to_target)?,
                remap_bvars(body, source_ctx_len + 1, target_ctx_len + 1, &body_map)?,
            ))
        }
    }
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
    data: &InductiveDecl,
    constructor: &str,
    domain_index: usize,
    domain: &Expr,
) -> Result<()> {
    if domain_index >= data.params.len() && is_direct_recursive_domain(data, domain, domain_index) {
        return Ok(());
    }
    if contains_const(domain, &data.name) {
        return Err(Error::NonPositiveOccurrence {
            inductive: data.name.clone(),
            constructor: constructor.to_owned(),
            ty: domain.clone(),
        });
    }
    Ok(())
}

fn is_direct_recursive_domain(data: &InductiveDecl, domain: &Expr, ctx_len: usize) -> bool {
    let (head, args) = collect_apps(domain);
    let levels = match head {
        Expr::Const { name, levels } if name == data.name => levels,
        _ => return false,
    };

    let expected_levels: Vec<_> = data
        .universe_params
        .iter()
        .map(|param| Level::param(param.clone()))
        .collect();
    if !levels_eq(&levels, &expected_levels) || args.len() != data.params.len() + data.indices.len()
    {
        return false;
    }

    for (param_index, arg) in args.iter().take(data.params.len()).enumerate() {
        let Ok(expected) = bvar_for_abs(ctx_len, param_index) else {
            return false;
        };
        if arg != &expected {
            return false;
        }
    }

    args.iter().all(|arg| !contains_const(arg, &data.name))
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
