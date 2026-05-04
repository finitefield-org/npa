use crate::error::{Error, Result};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    Zero,
    Succ(Box<Level>),
    Max(Box<Level>, Box<Level>),
    IMax(Box<Level>, Box<Level>),
    Param(String),
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

pub fn ensure_level_wf(delta: &[String], level: &Level) -> Result<()> {
    match level {
        Level::Zero => Ok(()),
        Level::Succ(level) => ensure_level_wf(delta, level),
        Level::Max(lhs, rhs) | Level::IMax(lhs, rhs) => {
            ensure_level_wf(delta, lhs)?;
            ensure_level_wf(delta, rhs)
        }
        Level::Param(name) => {
            if delta.iter().any(|param| param == name) {
                Ok(())
            } else {
                Err(Error::UnknownUniverseParam(name.clone()))
            }
        }
    }
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
}
