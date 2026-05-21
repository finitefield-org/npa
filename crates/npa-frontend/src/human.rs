use crate::{FileId, Span};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanModule {
    pub file_id: FileId,
    pub items: Vec<HumanItem>,
    pub span: Span,
}

impl HumanModule {
    pub fn empty(file_id: FileId, source_len: u32) -> Self {
        Self {
            file_id,
            items: Vec::new(),
            span: Span::new(file_id, 0, source_len),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HumanItem {
    Import { module: HumanName, span: Span },
    Open { namespace: HumanName, span: Span },
    NamespaceStart { name: HumanName, span: Span },
    NamespaceEnd { name: Option<HumanName>, span: Span },
    Def(HumanDecl),
    Theorem(HumanDecl),
    Axiom(HumanAxiomDecl),
    Inductive(HumanInductiveDecl),
    Notation(HumanNotationDecl),
}

impl HumanItem {
    pub fn span(&self) -> Span {
        match self {
            Self::Import { span, .. }
            | Self::Open { span, .. }
            | Self::NamespaceStart { span, .. }
            | Self::NamespaceEnd { span, .. } => *span,
            Self::Def(decl) | Self::Theorem(decl) => decl.span,
            Self::Axiom(decl) => decl.span,
            Self::Inductive(decl) => decl.span,
            Self::Notation(decl) => decl.span,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanDecl {
    pub name: HumanName,
    pub universe_params: Vec<HumanUniverseParam>,
    pub binders: Vec<HumanBinder>,
    pub ty: HumanExpr,
    pub value: HumanExpr,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanAxiomDecl {
    pub name: HumanName,
    pub universe_params: Vec<HumanUniverseParam>,
    pub binders: Vec<HumanBinder>,
    pub ty: HumanExpr,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanInductiveDecl {
    pub name: HumanName,
    pub universe_params: Vec<HumanUniverseParam>,
    pub binders: Vec<HumanBinder>,
    pub ty: HumanExpr,
    pub constructors: Vec<HumanConstructorDecl>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanConstructorDecl {
    pub name: HumanName,
    pub ty: HumanExpr,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanNotationDecl {
    pub kind: HumanNotationKind,
    pub precedence: u16,
    pub token: String,
    pub target: HumanName,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HumanNotationKind {
    Notation,
    Prefix,
    Postfix,
    Infix,
    Infixl,
    Infixr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanName {
    pub parts: Vec<String>,
    pub span: Span,
}

impl HumanName {
    pub fn new(parts: Vec<String>, span: Span) -> Self {
        Self { parts, span }
    }

    pub fn as_dotted(&self) -> String {
        self.parts.join(".")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanUniverseParam {
    pub name: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanBinder {
    pub kind: HumanBinderKind,
    pub ty: Option<Box<HumanExpr>>,
    pub binder_info: HumanBinderInfo,
    pub span: Span,
}

impl HumanBinder {
    pub fn named(
        name: HumanName,
        ty: Option<HumanExpr>,
        binder_info: HumanBinderInfo,
        span: Span,
    ) -> Self {
        Self {
            kind: HumanBinderKind::Named(name),
            ty: ty.map(Box::new),
            binder_info,
            span,
        }
    }

    pub fn anonymous(ty: Option<HumanExpr>, span: Span) -> Self {
        Self {
            kind: HumanBinderKind::Anonymous,
            ty: ty.map(Box::new),
            binder_info: HumanBinderInfo::Explicit,
            span,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HumanBinderKind {
    Named(HumanName),
    Anonymous,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HumanBinderInfo {
    Explicit,
    Implicit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HumanImplicitMode {
    Insert,
    Explicit,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HumanLevel {
    Nat {
        value: u64,
        span: Span,
    },
    Param {
        name: String,
        span: Span,
    },
    Succ {
        level: Box<HumanLevel>,
        span: Span,
    },
    Max {
        lhs: Box<HumanLevel>,
        rhs: Box<HumanLevel>,
        span: Span,
    },
    IMax {
        lhs: Box<HumanLevel>,
        rhs: Box<HumanLevel>,
        span: Span,
    },
}

impl HumanLevel {
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
pub enum HumanExpr {
    Ident {
        name: HumanName,
        universe_args: Option<Vec<HumanLevel>>,
        implicit_mode: HumanImplicitMode,
        span: Span,
    },
    Sort {
        level: HumanLevel,
        span: Span,
    },
    App {
        func: Box<HumanExpr>,
        arg: Box<HumanExpr>,
        span: Span,
    },
    Lam {
        binders: Vec<HumanBinder>,
        body: Box<HumanExpr>,
        span: Span,
    },
    Pi {
        binders: Vec<HumanBinder>,
        body: Box<HumanExpr>,
        span: Span,
    },
    Let {
        name: HumanName,
        ty: Option<Box<HumanExpr>>,
        value: Box<HumanExpr>,
        body: Box<HumanExpr>,
        span: Span,
    },
    Annot {
        expr: Box<HumanExpr>,
        ty: Box<HumanExpr>,
        span: Span,
    },
    Arrow {
        domain: Box<HumanExpr>,
        codomain: Box<HumanExpr>,
        span: Span,
    },
    Hole {
        name: Option<HumanName>,
        span: Span,
    },
    NotationApp {
        head: HumanNotationHead,
        args: Vec<HumanExpr>,
        span: Span,
    },
}

impl HumanExpr {
    pub fn span(&self) -> Span {
        match self {
            Self::Ident { span, .. }
            | Self::Sort { span, .. }
            | Self::App { span, .. }
            | Self::Lam { span, .. }
            | Self::Pi { span, .. }
            | Self::Let { span, .. }
            | Self::Annot { span, .. }
            | Self::Arrow { span, .. }
            | Self::Hole { span, .. }
            | Self::NotationApp { span, .. } => *span,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanNotationHead {
    pub token: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanFrontendState {
    pub current_module: npa_cert::ModuleName,
    pub namespace_stack: Vec<HumanName>,
    pub open_scopes: Vec<HumanOpenScopeFrame>,
    pub notation_table: Vec<HumanSourceNotationMetadata>,
    pub source_interfaces: HumanSourceInterfaceStore,
}

impl HumanFrontendState {
    pub fn new(current_module: npa_cert::ModuleName) -> Self {
        Self {
            source_interfaces: HumanSourceInterfaceStore {
                current: HumanSourceInterface::new(current_module.clone()),
                imports: Vec::new(),
            },
            current_module,
            namespace_stack: Vec::new(),
            open_scopes: vec![HumanOpenScopeFrame {
                namespace: None,
                opens: Vec::new(),
            }],
            notation_table: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanOpenScopeFrame {
    pub namespace: Option<HumanName>,
    pub opens: Vec<HumanOpenScope>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanOpenScope {
    pub namespace: HumanName,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanSourceInterfaceStore {
    pub current: HumanSourceInterface,
    pub imports: Vec<HumanImportedSourceInterface>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanSourceInterface {
    pub module: npa_cert::ModuleName,
    pub declarations: Vec<HumanSourceDeclarationMetadata>,
    pub notations: Vec<HumanSourceNotationMetadata>,
    pub generated_declarations: Vec<HumanGeneratedDeclarationMetadata>,
}

impl HumanSourceInterface {
    pub fn new(module: npa_cert::ModuleName) -> Self {
        Self {
            module,
            declarations: Vec::new(),
            notations: Vec::new(),
            generated_declarations: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanImportedSourceInterface {
    pub module: npa_cert::ModuleName,
    pub export_hash: npa_cert::Hash,
    pub certificate_hash: Option<npa_cert::Hash>,
    pub source_interface: HumanSourceInterface,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanSourceDeclarationMetadata {
    pub kind: HumanSourceDeclarationKind,
    pub name: HumanName,
    pub universe_params: Vec<HumanUniverseParam>,
    pub binders: Vec<HumanSourceBinderMetadata>,
    pub decl_interface_hash: Option<npa_cert::Hash>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HumanSourceDeclarationKind {
    Def,
    Theorem,
    Axiom,
    Inductive,
    Imported,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanSourceBinderMetadata {
    pub name: Option<HumanName>,
    pub binder_info: HumanBinderInfo,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanSourceNotationMetadata {
    pub kind: HumanNotationKind,
    pub precedence: u16,
    pub token: String,
    pub target: HumanName,
    pub namespace: Vec<String>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanGeneratedDeclarationMetadata {
    pub kind: HumanGeneratedDeclarationKind,
    pub parent: HumanName,
    pub name: HumanName,
    pub decl_interface_hash: Option<npa_cert::Hash>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HumanGeneratedDeclarationKind {
    Constructor,
    Recursor,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HumanCompileOptions {}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(start: u32, end: u32) -> Span {
        Span::new(FileId(0), start, end)
    }

    fn name(value: &str, start: u32, end: u32) -> HumanName {
        HumanName::new(vec![value.to_owned()], span(start, end))
    }

    fn sort_universe(value: u64, start: u32, end: u32) -> HumanExpr {
        HumanExpr::Sort {
            level: HumanLevel::Nat {
                value,
                span: span(start, end),
            },
            span: span(start, end),
        }
    }

    fn ident(value: &str, start: u32, end: u32) -> HumanExpr {
        HumanExpr::Ident {
            name: name(value, start, end),
            universe_args: None,
            implicit_mode: HumanImplicitMode::Insert,
            span: span(start, end),
        }
    }

    #[test]
    fn human_module_is_distinct_from_machine_module() {
        let module = HumanModule::empty(FileId(3), 11);

        assert_eq!(module.file_id, FileId(3));
        assert!(module.items.is_empty());
        assert_eq!(module.span, Span::new(FileId(3), 0, 11));
    }

    #[test]
    fn human_ast_can_model_explicit_id_declaration() {
        let type_expr = sort_universe(1, 10, 14);
        let binder_a = HumanBinder::named(
            name("A", 8, 9),
            Some(type_expr.clone()),
            HumanBinderInfo::Explicit,
            span(7, 15),
        );
        let binder_x = HumanBinder::named(
            name("x", 17, 18),
            Some(ident("A", 21, 22)),
            HumanBinderInfo::Explicit,
            span(16, 23),
        );
        let decl = HumanDecl {
            name: name("id", 4, 6),
            universe_params: Vec::new(),
            binders: vec![binder_a, binder_x],
            ty: ident("A", 26, 27),
            value: ident("x", 31, 32),
            span: span(0, 32),
        };
        let item = HumanItem::Def(decl);

        assert_eq!(item.span(), span(0, 32));
        let HumanItem::Def(decl) = item else {
            panic!("expected def item");
        };
        assert_eq!(decl.name.as_dotted(), "id");
        assert_eq!(decl.binders.len(), 2);
        assert!(decl
            .binders
            .iter()
            .all(|binder| binder.binder_info == HumanBinderInfo::Explicit));
        assert_eq!(decl.ty.span(), span(26, 27));
        assert_eq!(decl.value.span(), span(31, 32));
    }

    #[test]
    fn human_binder_info_distinguishes_explicit_and_implicit() {
        let ty = sort_universe(1, 3, 7);
        let explicit = HumanBinder::named(
            name("A", 1, 2),
            Some(ty.clone()),
            HumanBinderInfo::Explicit,
            span(0, 8),
        );
        let implicit = HumanBinder::named(
            name("A", 10, 11),
            Some(ty),
            HumanBinderInfo::Implicit,
            span(9, 17),
        );

        assert_ne!(explicit, implicit);
        assert_eq!(explicit.binder_info, HumanBinderInfo::Explicit);
        assert_eq!(implicit.binder_info, HumanBinderInfo::Implicit);
    }

    #[test]
    fn grouped_binders_are_represented_as_expanded_binder_lists() {
        let ty = ident("A", 7, 8);
        let expanded = [
            HumanBinder::named(
                name("x", 1, 2),
                Some(ty.clone()),
                HumanBinderInfo::Explicit,
                span(0, 9),
            ),
            HumanBinder::named(
                name("y", 3, 4),
                Some(ty),
                HumanBinderInfo::Explicit,
                span(0, 9),
            ),
        ];

        assert_eq!(expanded.len(), 2);
        assert!(expanded.iter().all(|binder| binder.ty.is_some()));
        assert_eq!(expanded[0].span, expanded[1].span);
    }

    #[test]
    fn human_holes_preserve_anonymous_and_named_forms() {
        let anonymous = HumanExpr::Hole {
            name: None,
            span: span(0, 1),
        };
        let named = HumanExpr::Hole {
            name: Some(name("m", 2, 4)),
            span: span(2, 4),
        };

        assert_eq!(anonymous.span(), span(0, 1));
        assert_eq!(named.span(), span(2, 4));
        assert!(matches!(anonymous, HumanExpr::Hole { name: None, .. }));
        assert!(matches!(named, HumanExpr::Hole { name: Some(_), .. }));
    }
}
