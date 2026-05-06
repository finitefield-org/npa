use crate::{FileId, Span};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineModule {
    pub file_id: FileId,
    pub items: Vec<MachineItem>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MachineItem {
    Import { module: MachineName, span: Span },
    Def(MachineDecl),
    Theorem(MachineDecl),
}

impl MachineItem {
    pub fn span(&self) -> Span {
        match self {
            Self::Import { span, .. } => *span,
            Self::Def(decl) | Self::Theorem(decl) => decl.span,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineDecl {
    pub name: MachineName,
    pub universe_params: Vec<MachineUniverseParam>,
    pub binders: Vec<MachineBinder>,
    pub ty: MachineTerm,
    pub value: MachineTerm,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineUniverseParam {
    pub name: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineName {
    pub parts: Vec<String>,
    pub span: Span,
}

impl MachineName {
    pub fn new(parts: Vec<String>, span: Span) -> Self {
        Self { parts, span }
    }

    pub fn as_dotted(&self) -> String {
        self.parts.join(".")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineBinder {
    pub name: String,
    pub ty: MachineTerm,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MachineLevel {
    Nat {
        value: u64,
        span: Span,
    },
    Param {
        name: String,
        span: Span,
    },
    Succ {
        level: Box<MachineLevel>,
        span: Span,
    },
    Max {
        lhs: Box<MachineLevel>,
        rhs: Box<MachineLevel>,
        span: Span,
    },
    IMax {
        lhs: Box<MachineLevel>,
        rhs: Box<MachineLevel>,
        span: Span,
    },
}

impl MachineLevel {
    pub fn span(&self) -> Span {
        match self {
            Self::Nat { span, .. }
            | Self::Param { span, .. }
            | Self::Succ { span, .. }
            | Self::Max { span, .. }
            | Self::IMax { span, .. } => *span,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MachineTerm {
    Ident {
        name: MachineName,
        universe_args: Option<Vec<MachineLevel>>,
        explicit_mode: bool,
        span: Span,
    },
    Local {
        name: String,
        span: Span,
    },
    Sort {
        level: MachineLevel,
        span: Span,
    },
    App {
        func: Box<MachineTerm>,
        arg: Box<MachineTerm>,
        span: Span,
    },
    Lam {
        binders: Vec<MachineBinder>,
        body: Box<MachineTerm>,
        span: Span,
    },
    Pi {
        binders: Vec<MachineBinder>,
        body: Box<MachineTerm>,
        span: Span,
    },
    Let {
        name: String,
        ty: Box<MachineTerm>,
        value: Box<MachineTerm>,
        body: Box<MachineTerm>,
        span: Span,
    },
    Annot {
        expr: Box<MachineTerm>,
        ty: Box<MachineTerm>,
        span: Span,
    },
}

impl MachineTerm {
    pub fn span(&self) -> Span {
        match self {
            Self::Ident { span, .. }
            | Self::Local { span, .. }
            | Self::Sort { span, .. }
            | Self::App { span, .. }
            | Self::Lam { span, .. }
            | Self::Pi { span, .. }
            | Self::Let { span, .. }
            | Self::Annot { span, .. } => *span,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineSurfaceMode {
    Complete,
    Repair,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineCompileOptions {
    pub mode: MachineSurfaceMode,
    pub allow_universe_meta: bool,
}

impl Default for MachineCompileOptions {
    fn default() -> Self {
        Self {
            mode: MachineSurfaceMode::Complete,
            allow_universe_meta: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineLocalDecl {
    pub name: String,
    pub ty: npa_kernel::Expr,
    pub value: Option<npa_kernel::Expr>,
}
