use crate::{expr::Expr, level::Level};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reducibility {
    Reducible,
    Opaque,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decl {
    Axiom {
        name: String,
        universe_params: Vec<String>,
        ty: Expr,
    },
    Def {
        name: String,
        universe_params: Vec<String>,
        ty: Expr,
        value: Expr,
        reducibility: Reducibility,
    },
    Theorem {
        name: String,
        universe_params: Vec<String>,
        ty: Expr,
        proof: Expr,
    },
    Inductive {
        name: String,
        universe_params: Vec<String>,
        ty: Expr,
        data: Box<InductiveDecl>,
    },
    Constructor {
        name: String,
        universe_params: Vec<String>,
        ty: Expr,
        inductive: String,
    },
    Recursor {
        name: String,
        universe_params: Vec<String>,
        ty: Expr,
        inductive: String,
        rules: RecursorRules,
    },
}

impl Decl {
    pub fn name(&self) -> &str {
        match self {
            Self::Axiom { name, .. } | Self::Def { name, .. } | Self::Theorem { name, .. } => name,
            Self::Inductive { name, .. }
            | Self::Constructor { name, .. }
            | Self::Recursor { name, .. } => name,
        }
    }

    pub fn universe_params(&self) -> &[String] {
        match self {
            Self::Axiom {
                universe_params, ..
            }
            | Self::Def {
                universe_params, ..
            }
            | Self::Theorem {
                universe_params, ..
            }
            | Self::Inductive {
                universe_params, ..
            }
            | Self::Constructor {
                universe_params, ..
            }
            | Self::Recursor {
                universe_params, ..
            } => universe_params,
        }
    }

    pub fn ty(&self) -> &Expr {
        match self {
            Self::Axiom { ty, .. } | Self::Def { ty, .. } | Self::Theorem { ty, .. } => ty,
            Self::Inductive { ty, .. }
            | Self::Constructor { ty, .. }
            | Self::Recursor { ty, .. } => ty,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Binder {
    pub name: String,
    pub ty: Expr,
}

impl Binder {
    pub fn new(name: impl Into<String>, ty: Expr) -> Self {
        Self {
            name: name.into(),
            ty,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstructorDecl {
    pub name: String,
    pub ty: Expr,
}

impl ConstructorDecl {
    pub fn new(name: impl Into<String>, ty: Expr) -> Self {
        Self {
            name: name.into(),
            ty,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecursorRules {
    pub minor_start: usize,
    pub major_index: usize,
}

impl RecursorRules {
    pub fn new(minor_start: usize, major_index: usize) -> Self {
        Self {
            minor_start,
            major_index,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecursorDecl {
    pub name: String,
    pub universe_params: Vec<String>,
    pub ty: Expr,
    pub rules: Option<RecursorRules>,
}

impl RecursorDecl {
    pub fn new(name: impl Into<String>, universe_params: Vec<String>, ty: Expr) -> Self {
        Self {
            name: name.into(),
            universe_params,
            ty,
            rules: None,
        }
    }

    pub fn with_rules(
        name: impl Into<String>,
        universe_params: Vec<String>,
        ty: Expr,
        rules: RecursorRules,
    ) -> Self {
        Self {
            name: name.into(),
            universe_params,
            ty,
            rules: Some(rules),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InductiveDecl {
    pub name: String,
    pub universe_params: Vec<String>,
    pub params: Vec<Binder>,
    pub indices: Vec<Binder>,
    pub sort: Level,
    pub constructors: Vec<ConstructorDecl>,
    pub recursor: Option<RecursorDecl>,
}

impl InductiveDecl {
    pub fn new(
        name: impl Into<String>,
        universe_params: Vec<String>,
        params: Vec<Binder>,
        indices: Vec<Binder>,
        sort: Level,
        constructors: Vec<ConstructorDecl>,
        recursor: Option<RecursorDecl>,
    ) -> Self {
        Self {
            name: name.into(),
            universe_params,
            params,
            indices,
            sort,
            constructors,
            recursor,
        }
    }
}
