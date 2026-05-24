use std::collections::BTreeSet;

use crate::error::{Error, Result};

const HUMAN_UNIVERSE_META_PREFIX: &str = "__npa_internal_human_universe_meta#";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    Zero,
    Succ(Box<Level>),
    Max(Box<Level>, Box<Level>),
    IMax(Box<Level>, Box<Level>),
    Param(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UniverseConstraintRelation {
    Le,
    Eq,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UniverseConstraint {
    pub lhs: Level,
    pub relation: UniverseConstraintRelation,
    pub rhs: Level,
}

impl UniverseConstraint {
    pub fn le(lhs: Level, rhs: Level) -> Self {
        Self {
            lhs,
            relation: UniverseConstraintRelation::Le,
            rhs,
        }
    }

    pub fn eq(lhs: Level, rhs: Level) -> Self {
        Self {
            lhs,
            relation: UniverseConstraintRelation::Eq,
            rhs,
        }
    }
}

impl Level {
    pub fn zero() -> Self {
        Self::Zero
    }

    pub fn succ(level: Self) -> Self {
        Self::Succ(Box::new(level))
    }

    pub fn max(lhs: Self, rhs: Self) -> Self {
        normalize_level(Self::Max(Box::new(lhs), Box::new(rhs)))
    }

    pub fn imax(lhs: Self, rhs: Self) -> Self {
        normalize_level(Self::IMax(Box::new(lhs), Box::new(rhs)))
    }

    pub fn param(name: impl Into<String>) -> Self {
        Self::Param(name.into())
    }
}

pub fn validate_universe_params(params: &[String]) -> Result<Vec<String>> {
    let mut seen = BTreeSet::new();
    for param in params {
        if is_unresolved_universe_meta_param(param) {
            return Err(Error::UnresolvedUniverseMeta(param.clone()));
        }
        if !seen.insert(param.clone()) {
            return Err(Error::DuplicateUniverseParam(param.clone()));
        }
    }
    if !params.windows(2).all(|pair| pair[0] < pair[1]) {
        return Err(Error::NonCanonicalUniverseParams(params.to_vec()));
    }
    Ok(params.to_vec())
}

pub fn ensure_level_wf(delta: &[String], level: &Level) -> Result<()> {
    match level {
        Level::Zero => Ok(()),
        Level::Succ(level) => ensure_level_wf(delta, level),
        Level::Max(lhs, rhs) | Level::IMax(lhs, rhs) => {
            ensure_level_wf(delta, lhs)?;
            ensure_level_wf(delta, rhs)
        }
        Level::Param(name) => {
            if is_unresolved_universe_meta_param(name) {
                return Err(Error::UnresolvedUniverseMeta(name.clone()));
            }
            if delta.iter().any(|param| param == name) {
                Ok(())
            } else {
                Err(Error::UnknownUniverseParam(name.clone()))
            }
        }
    }
}

fn is_unresolved_universe_meta_param(param: &str) -> bool {
    param.starts_with(HUMAN_UNIVERSE_META_PREFIX) || param.contains('?')
}

pub fn ensure_universe_constraints_wf(
    delta: &[String],
    constraints: &[UniverseConstraint],
) -> Result<()> {
    let mut canonical = constraints.to_vec();
    canonical.sort();
    for constraint in &canonical {
        ensure_canonical_level(delta, &constraint.lhs)?;
        ensure_canonical_level(delta, &constraint.rhs)?;
    }
    if constraints != canonical.as_slice() {
        return Err(Error::NonCanonicalUniverseConstraints);
    }
    if canonical.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(Error::DuplicateUniverseConstraint);
    }
    Ok(())
}

pub fn normalize_level(level: Level) -> Level {
    match level {
        Level::Zero | Level::Param(_) => level,
        Level::Succ(level) => Level::Succ(Box::new(normalize_level(*level))),
        Level::Max(lhs, rhs) => {
            let lhs = normalize_level(*lhs);
            let rhs = normalize_level(*rhs);
            if lhs == rhs {
                return lhs;
            }
            if lhs == Level::Zero {
                return rhs;
            }
            if rhs == Level::Zero {
                return lhs;
            }
            match (level_as_nat(&lhs), level_as_nat(&rhs)) {
                (Some(lhs_nat), Some(rhs_nat)) => level_from_nat(lhs_nat.max(rhs_nat)),
                _ if rhs < lhs => Level::Max(Box::new(rhs), Box::new(lhs)),
                _ => Level::Max(Box::new(lhs), Box::new(rhs)),
            }
        }
        Level::IMax(lhs, rhs) => {
            let lhs = normalize_level(*lhs);
            let rhs = normalize_level(*rhs);
            match rhs {
                Level::Zero => Level::Zero,
                Level::Succ(inner) => {
                    normalize_level(Level::Max(Box::new(lhs), Box::new(Level::Succ(inner))))
                }
                rhs => Level::IMax(Box::new(lhs), Box::new(rhs)),
            }
        }
    }
}

fn ensure_canonical_level(delta: &[String], level: &Level) -> Result<()> {
    ensure_level_wf(delta, level)?;
    let normalized = normalize_level(level.clone());
    if normalized == *level {
        Ok(())
    } else {
        Err(Error::NonCanonicalUniverseLevel {
            level: level.clone(),
        })
    }
}

pub fn level_eq(lhs: &Level, rhs: &Level) -> bool {
    normalize_level(lhs.clone()) == normalize_level(rhs.clone())
}

pub fn levels_eq(lhs: &[Level], rhs: &[Level]) -> bool {
    lhs.len() == rhs.len() && lhs.iter().zip(rhs).all(|(lhs, rhs)| level_eq(lhs, rhs))
}

fn level_as_nat(level: &Level) -> Option<u32> {
    match level {
        Level::Zero => Some(0),
        Level::Succ(level) => Some(level_as_nat(level)? + 1),
        _ => None,
    }
}

fn level_from_nat(n: u32) -> Level {
    (0..n).fold(Level::Zero, |level, _| Level::succ(level))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_zero_normalizes_to_other_level() {
        let u = Level::succ(Level::param("u"));

        assert!(level_eq(&Level::max(Level::zero(), u.clone()), &u));
        assert!(level_eq(&Level::max(u.clone(), Level::zero()), &u));
        assert!(level_eq(&Level::imax(Level::zero(), u.clone()), &u));
    }

    #[test]
    fn universe_constraints_accept_empty_and_max_le() {
        let delta =
            validate_universe_params(&["u".to_owned(), "v".to_owned(), "w".to_owned()]).unwrap();
        let constraint = UniverseConstraint::le(
            Level::max(Level::param("u"), Level::param("v")),
            Level::param("w"),
        );

        ensure_universe_constraints_wf(&delta, &[]).unwrap();
        ensure_universe_constraints_wf(&delta, &[constraint]).unwrap();
    }

    #[test]
    fn universe_params_reject_duplicate_and_noncanonical_order() {
        assert_eq!(
            validate_universe_params(&["u".to_owned(), "u".to_owned()]),
            Err(Error::DuplicateUniverseParam("u".to_owned()))
        );
        assert_eq!(
            validate_universe_params(&["v".to_owned(), "u".to_owned()]),
            Err(Error::NonCanonicalUniverseParams(vec![
                "v".to_owned(),
                "u".to_owned()
            ]))
        );
    }

    #[test]
    fn universe_params_reject_unresolved_meta_names() {
        assert_eq!(
            validate_universe_params(&["?u".to_owned()]),
            Err(Error::UnresolvedUniverseMeta("?u".to_owned()))
        );
        assert_eq!(
            ensure_level_wf(&["u".to_owned()], &Level::param("z?meta")),
            Err(Error::UnresolvedUniverseMeta("z?meta".to_owned()))
        );
        assert_eq!(
            validate_universe_params(&[format!("{HUMAN_UNIVERSE_META_PREFIX}0")]),
            Err(Error::UnresolvedUniverseMeta(format!(
                "{HUMAN_UNIVERSE_META_PREFIX}0"
            )))
        );
    }

    #[test]
    fn universe_constraints_reject_unknown_and_noncanonical_levels() {
        let delta = validate_universe_params(&["u".to_owned(), "v".to_owned()]).unwrap();
        let unknown = UniverseConstraint::le(Level::param("u"), Level::param("w"));
        assert_eq!(
            ensure_universe_constraints_wf(&delta, &[unknown]),
            Err(Error::UnknownUniverseParam("w".to_owned()))
        );

        let noncanonical = UniverseConstraint::le(
            Level::Max(Box::new(Level::param("v")), Box::new(Level::param("u"))),
            Level::param("v"),
        );
        assert_eq!(
            ensure_universe_constraints_wf(&delta, std::slice::from_ref(&noncanonical)),
            Err(Error::NonCanonicalUniverseLevel {
                level: noncanonical.lhs,
            })
        );
    }
}
