use crate::{
    error::{Error, Result},
    expr::Expr,
    level::{normalize_level, Level},
};

pub fn subst_levels_expr(expr: &Expr, params: &[String], levels: &[Level]) -> Expr {
    match expr {
        Expr::Sort(level) => Expr::sort(subst_level(level, params, levels)),
        Expr::BVar(index) => Expr::bvar(*index),
        Expr::Const { name, levels: us } => Expr::konst(
            name.clone(),
            us.iter()
                .map(|level| subst_level(level, params, levels))
                .collect(),
        ),
        Expr::App(fun, arg) => Expr::app(
            subst_levels_expr(fun, params, levels),
            subst_levels_expr(arg, params, levels),
        ),
        Expr::Lam { binder, ty, body } => Expr::lam(
            binder.clone(),
            subst_levels_expr(ty, params, levels),
            subst_levels_expr(body, params, levels),
        ),
        Expr::Pi { binder, ty, body } => Expr::pi(
            binder.clone(),
            subst_levels_expr(ty, params, levels),
            subst_levels_expr(body, params, levels),
        ),
        Expr::Let {
            binder,
            ty,
            value,
            body,
        } => Expr::let_in(
            binder.clone(),
            subst_levels_expr(ty, params, levels),
            subst_levels_expr(value, params, levels),
            subst_levels_expr(body, params, levels),
        ),
    }
}

fn subst_level(level: &Level, params: &[String], levels: &[Level]) -> Level {
    match level {
        Level::Zero => Level::Zero,
        Level::Succ(level) => Level::succ(subst_level(level, params, levels)),
        Level::Max(lhs, rhs) => Level::max(
            subst_level(lhs, params, levels),
            subst_level(rhs, params, levels),
        ),
        Level::IMax(lhs, rhs) => Level::imax(
            subst_level(lhs, params, levels),
            subst_level(rhs, params, levels),
        ),
        Level::Param(name) => params
            .iter()
            .position(|param| param == name)
            .map(|index| levels[index].clone())
            .unwrap_or_else(|| Level::param(name.clone())),
    }
}

pub fn shift(expr: &Expr, amount: i32, cutoff: u32) -> Result<Expr> {
    match expr {
        Expr::Sort(level) => Ok(Expr::sort(level.clone())),
        Expr::BVar(index) => {
            if *index < cutoff {
                Ok(Expr::bvar(*index))
            } else {
                let shifted = *index as i32 + amount;
                if shifted < 0 {
                    Err(Error::InvalidBVar(*index))
                } else {
                    Ok(Expr::bvar(shifted as u32))
                }
            }
        }
        Expr::Const { name, levels } => Ok(Expr::konst(name.clone(), levels.clone())),
        Expr::App(fun, arg) => Ok(Expr::app(
            shift(fun, amount, cutoff)?,
            shift(arg, amount, cutoff)?,
        )),
        Expr::Lam { binder, ty, body } => Ok(Expr::lam(
            binder.clone(),
            shift(ty, amount, cutoff)?,
            shift(body, amount, cutoff + 1)?,
        )),
        Expr::Pi { binder, ty, body } => Ok(Expr::pi(
            binder.clone(),
            shift(ty, amount, cutoff)?,
            shift(body, amount, cutoff + 1)?,
        )),
        Expr::Let {
            binder,
            ty,
            value,
            body,
        } => Ok(Expr::let_in(
            binder.clone(),
            shift(ty, amount, cutoff)?,
            shift(value, amount, cutoff)?,
            shift(body, amount, cutoff + 1)?,
        )),
    }
}

fn subst(expr: &Expr, target: u32, replacement: &Expr) -> Result<Expr> {
    match expr {
        Expr::Sort(level) => Ok(Expr::sort(level.clone())),
        Expr::BVar(index) if *index == target => shift(replacement, target as i32, 0),
        Expr::BVar(index) if *index > target => Ok(Expr::bvar(index - 1)),
        Expr::BVar(index) => Ok(Expr::bvar(*index)),
        Expr::Const { name, levels } => Ok(Expr::konst(name.clone(), levels.clone())),
        Expr::App(fun, arg) => Ok(Expr::app(
            subst(fun, target, replacement)?,
            subst(arg, target, replacement)?,
        )),
        Expr::Lam { binder, ty, body } => Ok(Expr::lam(
            binder.clone(),
            subst(ty, target, replacement)?,
            subst(body, target + 1, replacement)?,
        )),
        Expr::Pi { binder, ty, body } => Ok(Expr::pi(
            binder.clone(),
            subst(ty, target, replacement)?,
            subst(body, target + 1, replacement)?,
        )),
        Expr::Let {
            binder,
            ty,
            value,
            body,
        } => Ok(Expr::let_in(
            binder.clone(),
            subst(ty, target, replacement)?,
            subst(value, target, replacement)?,
            subst(body, target + 1, replacement)?,
        )),
    }
}

pub fn instantiate(body: &Expr, value: &Expr) -> Result<Expr> {
    subst(body, 0, value)
}

#[allow(dead_code)]
fn _assert_level_normalization_is_linked(level: Level) -> Level {
    normalize_level(level)
}
