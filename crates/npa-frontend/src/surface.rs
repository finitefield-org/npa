use crate::Span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurfaceModule {
    pub file_id: crate::FileId,
    pub items: Vec<SurfaceItem>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SurfaceItem {
    Import {
        module: SurfaceName,
        span: Span,
    },
    Open {
        namespace: SurfaceName,
        span: Span,
    },
    Namespace {
        name: String,
        span: Span,
    },
    End {
        name: Option<String>,
        span: Span,
    },
    Notation(NotationDecl),
    Def(SurfaceDecl),
    Theorem(SurfaceDecl),
    Axiom(SurfaceDecl),
    Inductive {
        name: String,
        universe_params: Vec<SurfaceUniverseParam>,
        binders: Vec<SurfaceBinder>,
        ty: SurfaceExpr,
        constructors: Vec<SurfaceCtorDecl>,
        span: Span,
    },
}

impl SurfaceItem {
    pub fn span(&self) -> Span {
        match self {
            Self::Import { span, .. }
            | Self::Open { span, .. }
            | Self::Namespace { span, .. }
            | Self::End { span, .. }
            | Self::Inductive { span, .. } => *span,
            Self::Notation(decl) => decl.span,
            Self::Def(decl) | Self::Theorem(decl) | Self::Axiom(decl) => decl.span,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurfaceDecl {
    pub name: String,
    pub universe_params: Vec<SurfaceUniverseParam>,
    pub binders: Vec<SurfaceBinder>,
    pub ty: SurfaceExpr,
    pub value: Option<SurfaceExpr>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurfaceCtorDecl {
    pub name: String,
    pub ty: SurfaceExpr,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurfaceUniverseParam {
    pub name: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurfaceName {
    pub parts: Vec<String>,
    pub span: Span,
}

impl SurfaceName {
    pub fn single(name: impl Into<String>, span: Span) -> Self {
        Self {
            parts: vec![name.into()],
            span,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BinderInfo {
    Explicit,
    Implicit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImplicitMode {
    Insert,
    Explicit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SurfaceBinderKind {
    Named(SurfaceName),
    Anonymous,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurfaceBinder {
    pub kind: SurfaceBinderKind,
    pub ty: Option<Box<SurfaceExpr>>,
    pub binder_info: BinderInfo,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SurfaceLevel {
    Nat {
        value: u64,
        span: Span,
    },
    Param {
        name: String,
        span: Span,
    },
    Succ {
        level: Box<SurfaceLevel>,
        span: Span,
    },
    Max {
        lhs: Box<SurfaceLevel>,
        rhs: Box<SurfaceLevel>,
        span: Span,
    },
    IMax {
        lhs: Box<SurfaceLevel>,
        rhs: Box<SurfaceLevel>,
        span: Span,
    },
}

impl SurfaceLevel {
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
pub enum SurfaceExpr {
    Ident {
        name: SurfaceName,
        universe_args: Option<Vec<SurfaceLevel>>,
        implicit_mode: ImplicitMode,
        span: Span,
    },
    Sort {
        level: SurfaceLevel,
        span: Span,
    },
    App {
        func: Box<SurfaceExpr>,
        arg: Box<SurfaceExpr>,
        span: Span,
    },
    Lam {
        binders: Vec<SurfaceBinder>,
        body: Box<SurfaceExpr>,
        span: Span,
    },
    Pi {
        binders: Vec<SurfaceBinder>,
        body: Box<SurfaceExpr>,
        span: Span,
    },
    Let {
        name: SurfaceName,
        ty: Option<Box<SurfaceExpr>>,
        value: Box<SurfaceExpr>,
        body: Box<SurfaceExpr>,
        span: Span,
    },
    Annot {
        expr: Box<SurfaceExpr>,
        ty: Box<SurfaceExpr>,
        span: Span,
    },
    Hole {
        name: Option<SurfaceName>,
        span: Span,
    },
    Notation {
        head: NotationHead,
        args: Vec<SurfaceExpr>,
        span: Span,
    },
}

impl SurfaceExpr {
    pub fn span(&self) -> Span {
        match self {
            Self::Ident { span, .. }
            | Self::Sort { span, .. }
            | Self::App { span, .. }
            | Self::Lam { span, .. }
            | Self::Pi { span, .. }
            | Self::Let { span, .. }
            | Self::Annot { span, .. }
            | Self::Hole { span, .. }
            | Self::Notation { span, .. } => *span,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotationHead {
    pub kind: NotationKind,
    pub symbol: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotationDecl {
    pub kind: NotationKind,
    pub precedence: u32,
    pub symbol: String,
    pub target: SurfaceName,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotationKind {
    Prefix,
    Postfix,
    Infix,
    Infixl,
    Infixr,
}
