use crate::expr::Expr;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceLimitKind {
    Whnf,
    Conversion,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    UnknownConstant(String),
    UnknownUniverseParam(String),
    BadUniverseArity {
        name: String,
        expected: usize,
        actual: usize,
    },
    InvalidBVar(u32),
    ExpectedSort {
        actual: Expr,
    },
    ExpectedPi {
        actual: Expr,
    },
    TypeMismatch {
        expected: Expr,
        actual: Expr,
    },
    NotDefEq {
        lhs: Expr,
        rhs: Expr,
    },
    DuplicateDecl(String),
    InvalidInductive(String),
    NonPositiveOccurrence {
        inductive: String,
        constructor: String,
        ty: Expr,
    },
    BadConstructorResult {
        inductive: String,
        constructor: String,
        result: Expr,
    },
    ResourceLimit {
        kind: ResourceLimitKind,
    },
}
