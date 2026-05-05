use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use npa_kernel::{subst::instantiate, Decl, Expr, Level, Reducibility};

use crate::parser::{parse_module_with_imports, ParserImport, ParserImportedNotation};
use crate::{
    BinderInfo, Diagnostic, DiagnosticKind, DiagnosticSeverity, FileId, ImplicitMode, NotationDecl,
    NotationKind, Result, Span, SurfaceBinder, SurfaceBinderKind, SurfaceCtorDecl, SurfaceDecl,
    SurfaceExpr, SurfaceItem, SurfaceLevel, SurfaceModule, SurfaceName, SurfaceUniverseParam,
};

const TYPE_ALIAS_SHAPE_FUEL: usize = 128;
const TYPE_ALIAS_SHAPE_MAX_NUMERIC_LEVEL: u64 = 1024;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name {
    pub parts: Vec<String>,
}

impl Name {
    pub fn new(parts: Vec<String>) -> Self {
        Self { parts }
    }

    pub fn from_dotted(name: &str) -> Self {
        let parts = name
            .split('.')
            .filter(|part| !part.is_empty())
            .map(str::to_owned)
            .collect();
        Self { parts }
    }

    pub fn from_surface(name: &SurfaceName) -> Self {
        Self {
            parts: name.parts.clone(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    pub fn push(&self, part: impl Into<String>) -> Self {
        let mut parts = self.parts.clone();
        parts.push(part.into());
        Self { parts }
    }

    pub fn append(&self, suffix: &Name) -> Self {
        let mut parts = self.parts.clone();
        parts.extend(suffix.parts.iter().cloned());
        Self { parts }
    }

    pub fn starts_with(&self, prefix: &Name) -> bool {
        self.parts.len() >= prefix.parts.len()
            && self.parts[..prefix.parts.len()] == prefix.parts[..]
    }

    pub fn ends_with(&self, suffix: &Name) -> bool {
        self.parts.len() >= suffix.parts.len()
            && self.parts[self.parts.len() - suffix.parts.len()..] == suffix.parts[..]
    }

    pub fn to_dotted(&self) -> String {
        self.parts.join(".")
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_dotted())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedImport {
    pub module: Name,
    pub export_hash: String,
    pub declarations: Vec<ImportedDeclaration>,
    pub notations: Vec<ImportedNotation>,
    pub kernel_declarations: Vec<Decl>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportedDeclaration {
    pub name: Name,
    pub decl_interface_hash: String,
    pub binder_infos: Vec<BinderInfo>,
    pub domain_infos: Vec<ImportedTypeMetadata>,
    pub type_value_metadata: Option<ImportedTypeMetadata>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportedNotation {
    pub kind: NotationKind,
    pub precedence: u32,
    pub symbol: String,
    pub target: Name,
    pub decl_interface_hash: String,
    pub namespace: Option<Name>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImportedTypeMetadata {
    pub binder_infos: Vec<BinderInfo>,
    pub domain_infos: Vec<ImportedTypeMetadata>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedModule {
    pub current_module: Name,
    pub items: Vec<ResolvedItem>,
    pub diagnostics: Vec<Diagnostic>,
    pub state: FrontendState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendState {
    pub current_module: Name,
    pub namespace_stack: Vec<String>,
    pub open_scopes: Vec<OpenScope>,
    pub globals: GlobalScope,
    pub locals: LocalScopeStack,
    pub imports: Vec<ResolvedImport>,
    pub notations: Vec<ResolvedNotationDecl>,
}

impl FrontendState {
    pub fn new(current_module: Name) -> Self {
        Self {
            current_module,
            namespace_stack: Vec::new(),
            open_scopes: vec![OpenScope::default()],
            globals: GlobalScope::default(),
            locals: LocalScopeStack::default(),
            imports: Vec::new(),
            notations: Vec::new(),
        }
    }

    pub fn current_namespace(&self) -> Name {
        Name::new(self.namespace_stack.clone())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OpenScope {
    pub namespaces: Vec<Name>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GlobalScope {
    pub current: Vec<GlobalDeclaration>,
    pub imported: Vec<GlobalDeclaration>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlobalDeclaration {
    pub name: Name,
    pub origin: GlobalOrigin,
    pub span: Option<Span>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GlobalOrigin {
    Local {
        decl_index: usize,
    },
    LocalGenerated {
        decl_index: usize,
    },
    Imported {
        module: Name,
        decl_interface_hash: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedImport {
    pub module: Name,
    pub export_hash: String,
    pub declarations: Vec<ImportedDeclaration>,
    pub kernel_declarations: Vec<Decl>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LocalScopeStack {
    pub bindings: Vec<LocalBinding>,
}

impl LocalScopeStack {
    fn len(&self) -> usize {
        self.bindings.len()
    }

    fn truncate(&mut self, len: usize) {
        self.bindings.truncate(len);
    }

    fn push(&mut self, binding: LocalBinding) {
        self.bindings.push(binding);
    }

    fn lookup(&self, name: &str) -> Option<LocalRef> {
        self.bindings
            .iter()
            .rev()
            .enumerate()
            .find_map(|(de_bruijn_index, binding)| {
                let binding_name = binding.name.as_deref()?;
                if binding_name == name {
                    Some(LocalRef {
                        id: binding.id,
                        name: binding_name.to_owned(),
                        de_bruijn_index: de_bruijn_index as u32,
                    })
                } else {
                    None
                }
            })
    }

    fn contains_named(&self, name: &str) -> bool {
        self.bindings
            .iter()
            .any(|binding| binding.name.as_deref() == Some(name))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalBinding {
    pub id: LocalId,
    pub name: Option<String>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocalId(pub u32);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalRef {
    pub id: LocalId,
    pub name: String,
    pub de_bruijn_index: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ElabGlobalRef {
    Local {
        decl_index: usize,
        name: Name,
    },
    LocalGenerated {
        decl_index: usize,
        name: Name,
    },
    Imported {
        module: Name,
        name: Name,
        decl_interface_hash: String,
    },
}

impl GlobalDeclaration {
    fn to_ref(&self) -> ElabGlobalRef {
        match &self.origin {
            GlobalOrigin::Local { decl_index } => ElabGlobalRef::Local {
                decl_index: *decl_index,
                name: self.name.clone(),
            },
            GlobalOrigin::LocalGenerated { decl_index } => ElabGlobalRef::LocalGenerated {
                decl_index: *decl_index,
                name: self.name.clone(),
            },
            GlobalOrigin::Imported {
                module,
                decl_interface_hash,
            } => ElabGlobalRef::Imported {
                module: module.clone(),
                name: self.name.clone(),
                decl_interface_hash: decl_interface_hash.clone(),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedName {
    Local(LocalRef),
    Global(ElabGlobalRef),
    Overloaded(Vec<ElabGlobalRef>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedItem {
    Import {
        module: Name,
        export_hash: String,
        duplicate: bool,
        span: Span,
    },
    Open {
        namespace: Name,
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
    Notation(ResolvedNotationDecl),
    Def(ResolvedDecl),
    Theorem(ResolvedDecl),
    Axiom(ResolvedDecl),
    Inductive {
        name: Name,
        universe_params: Vec<SurfaceUniverseParam>,
        binders: Vec<ResolvedBinder>,
        ty: ResolvedExpr,
        constructors: Vec<ResolvedCtorDecl>,
        span: Span,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedNotationDecl {
    pub kind: NotationKind,
    pub precedence: u32,
    pub symbol: String,
    pub target: ElabGlobalRef,
    pub namespace: Option<Name>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedDecl {
    pub name: Name,
    pub source_name: String,
    pub universe_params: Vec<SurfaceUniverseParam>,
    pub binders: Vec<ResolvedBinder>,
    pub ty: ResolvedExpr,
    pub value: Option<ResolvedExpr>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedCtorDecl {
    pub name: Name,
    pub source_name: String,
    pub ty: ResolvedExpr,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedBinder {
    pub kind: SurfaceBinderKind,
    pub local_id: LocalId,
    pub ty: Option<Box<ResolvedExpr>>,
    pub binder_info: BinderInfo,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedExpr {
    Ident {
        name: SurfaceName,
        resolved: ResolvedName,
        universe_args: Option<Vec<SurfaceLevel>>,
        implicit_mode: ImplicitMode,
        span: Span,
    },
    Sort {
        level: SurfaceLevel,
        span: Span,
    },
    App {
        func: Box<ResolvedExpr>,
        arg: Box<ResolvedExpr>,
        span: Span,
    },
    Lam {
        binders: Vec<ResolvedBinder>,
        body: Box<ResolvedExpr>,
        span: Span,
    },
    Pi {
        binders: Vec<ResolvedBinder>,
        body: Box<ResolvedExpr>,
        span: Span,
    },
    Let {
        name: SurfaceName,
        local_id: LocalId,
        ty: Option<Box<ResolvedExpr>>,
        value: Box<ResolvedExpr>,
        body: Box<ResolvedExpr>,
        span: Span,
    },
    Annot {
        expr: Box<ResolvedExpr>,
        ty: Box<ResolvedExpr>,
        span: Span,
    },
    Hole {
        name: Option<SurfaceName>,
        span: Span,
    },
    Notation {
        head: crate::NotationHead,
        candidates: Vec<ElabGlobalRef>,
        args: Vec<ResolvedExpr>,
        span: Span,
    },
}

pub fn resolve_source(
    file_id: FileId,
    current_module: Name,
    source: &str,
    verified_imports: &[VerifiedImport],
) -> Result<ResolvedModule> {
    let module = parse_module_with_verified_imports(file_id, source, verified_imports)?;
    resolve_module(current_module, &module, verified_imports)
}

pub fn parse_module_with_verified_imports(
    file_id: FileId,
    source: &str,
    verified_imports: &[VerifiedImport],
) -> Result<SurfaceModule> {
    parse_module_with_imports(
        file_id,
        source,
        &parser_imports_from_verified(verified_imports),
    )
}

pub fn resolve_module(
    current_module: Name,
    module: &SurfaceModule,
    verified_imports: &[VerifiedImport],
) -> Result<ResolvedModule> {
    Resolver::new(current_module, verified_imports).resolve_module(module)
}

struct Resolver<'a> {
    state: FrontendState,
    verified_imports: &'a [VerifiedImport],
    future_globals: BTreeMap<Name, Span>,
    type_alias_values: BTreeMap<Name, TypeAliasValue>,
    notation_scopes: Vec<Vec<ResolvedNotationDecl>>,
    namespace_notations: BTreeMap<Name, Vec<ResolvedNotationDecl>>,
    diagnostics: Vec<Diagnostic>,
    next_decl_index: usize,
    next_local_id: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InductiveResultShape {
    Sort,
    Indexed,
}

#[derive(Clone, Debug)]
enum TypeAliasValue {
    Resolved(ResolvedExpr),
    Core(Expr),
}

#[derive(Clone)]
struct ResolvedLamClosure {
    binders: Vec<ResolvedBinder>,
    body: ResolvedExpr,
    locals: BTreeMap<LocalId, ResolvedExpr>,
}

enum ResolvedApplied {
    Expr {
        expr: ResolvedExpr,
        locals: BTreeMap<LocalId, ResolvedExpr>,
    },
    Lam(ResolvedLamClosure),
}

impl<'a> Resolver<'a> {
    fn new(current_module: Name, verified_imports: &'a [VerifiedImport]) -> Self {
        Self {
            state: FrontendState::new(current_module),
            verified_imports,
            future_globals: BTreeMap::new(),
            type_alias_values: BTreeMap::new(),
            notation_scopes: vec![Vec::new()],
            namespace_notations: BTreeMap::new(),
            diagnostics: Vec::new(),
            next_decl_index: 0,
            next_local_id: 0,
        }
    }

    fn resolve_module(mut self, module: &SurfaceModule) -> Result<ResolvedModule> {
        self.future_globals = collect_current_module_declarations(module)?;

        let mut items = Vec::new();
        for item in &module.items {
            items.push(self.resolve_item(item)?);
        }

        if let Some(open_namespace) = self.state.namespace_stack.last() {
            return Err(Diagnostic::error(
                DiagnosticKind::NamespaceMismatch,
                module.span,
                format!("namespace `{open_namespace}` is not closed"),
            ));
        }

        debug_assert_eq!(self.state.locals.len(), 0);

        Ok(ResolvedModule {
            current_module: self.state.current_module.clone(),
            items,
            diagnostics: self.diagnostics,
            state: self.state,
        })
    }

    fn resolve_item(&mut self, item: &SurfaceItem) -> Result<ResolvedItem> {
        match item {
            SurfaceItem::Import { module, span } => self.resolve_import_item(module, *span),
            SurfaceItem::Open { namespace, span } => self.resolve_open_item(namespace, *span),
            SurfaceItem::Namespace { name, span } => {
                self.state.namespace_stack.push(name.clone());
                self.state.open_scopes.push(OpenScope::default());
                self.notation_scopes.push(Vec::new());
                self.activate_current_namespace_notations(*span)?;
                Ok(ResolvedItem::Namespace {
                    name: name.clone(),
                    span: *span,
                })
            }
            SurfaceItem::End { name, span } => {
                self.resolve_end_item(name.as_deref(), *span)?;
                Ok(ResolvedItem::End {
                    name: name.clone(),
                    span: *span,
                })
            }
            SurfaceItem::Notation(decl) => {
                let resolved = self.resolve_notation_decl(decl)?;
                self.register_notation(resolved.clone())?;
                Ok(ResolvedItem::Notation(resolved))
            }
            SurfaceItem::Def(decl) => {
                let resolved = self.resolve_value_decl(decl)?;
                self.register_resolved_decl(&resolved.name, resolved.span, false)?;
                self.record_local_type_alias_value(&resolved);
                Ok(ResolvedItem::Def(resolved))
            }
            SurfaceItem::Theorem(decl) => {
                let resolved = self.resolve_value_decl(decl)?;
                self.register_resolved_decl(&resolved.name, resolved.span, false)?;
                Ok(ResolvedItem::Theorem(resolved))
            }
            SurfaceItem::Axiom(decl) => {
                let resolved = self.resolve_value_decl(decl)?;
                self.register_resolved_decl(&resolved.name, resolved.span, false)?;
                Ok(ResolvedItem::Axiom(resolved))
            }
            SurfaceItem::Inductive {
                name,
                universe_params,
                binders,
                ty,
                constructors,
                span,
            } => {
                self.resolve_inductive_item(name, universe_params, binders, ty, constructors, *span)
            }
        }
    }

    fn resolve_import_item(&mut self, module: &SurfaceName, span: Span) -> Result<ResolvedItem> {
        let module_name = Name::from_surface(module);
        let import = self.lookup_verified_import(&module_name, span)?;

        if let Some(existing) = self
            .state
            .imports
            .iter()
            .find(|existing| existing.module == module_name)
        {
            self.diagnostics.push(Diagnostic {
                kind: DiagnosticKind::DuplicateImportWarning,
                severity: DiagnosticSeverity::Warning,
                primary_span: span,
                message: format!("duplicate import `{module_name}` ignored"),
            });
            return Ok(ResolvedItem::Import {
                module: module_name,
                export_hash: existing.export_hash.clone(),
                duplicate: true,
                span,
            });
        }

        self.state.imports.push(ResolvedImport {
            module: module_name.clone(),
            export_hash: import.export_hash.clone(),
            declarations: import.declarations.clone(),
            kernel_declarations: import.kernel_declarations.clone(),
        });
        self.record_imported_type_alias_values(import);

        let mut imported = import.declarations.clone();
        imported.sort_by(|lhs, rhs| {
            lhs.name
                .cmp(&rhs.name)
                .then(lhs.decl_interface_hash.cmp(&rhs.decl_interface_hash))
        });
        imported.dedup_by(|lhs, rhs| {
            lhs.name == rhs.name && lhs.decl_interface_hash == rhs.decl_interface_hash
        });

        for decl in imported {
            self.state.globals.imported.push(GlobalDeclaration {
                name: decl.name,
                origin: GlobalOrigin::Imported {
                    module: module_name.clone(),
                    decl_interface_hash: decl.decl_interface_hash,
                },
                span: None,
            });
        }
        self.register_imported_notations(&module_name, &import.notations, span)?;

        Ok(ResolvedItem::Import {
            module: module_name,
            export_hash: import.export_hash.clone(),
            duplicate: false,
            span,
        })
    }

    fn lookup_verified_import(&self, module: &Name, span: Span) -> Result<&'a VerifiedImport> {
        let matches: Vec<_> = self
            .verified_imports
            .iter()
            .filter(|import| &import.module == module)
            .collect();
        if matches.is_empty() {
            return Err(Diagnostic::error(
                DiagnosticKind::ImportResolutionError,
                span,
                format!("import `{module}` was not provided as a verified import"),
            ));
        }

        let hashes: BTreeSet<_> = matches.iter().map(|import| &import.export_hash).collect();
        if hashes.len() > 1 {
            return Err(Diagnostic::error(
                DiagnosticKind::ImportResolutionError,
                span,
                format!("verified imports contain multiple hashes for `{module}`"),
            ));
        }

        Ok(matches[0])
    }

    fn register_imported_notations(
        &mut self,
        module: &Name,
        notations: &[ImportedNotation],
        span: Span,
    ) -> Result<()> {
        let mut notations = notations.to_vec();
        notations.sort_by(|lhs, rhs| {
            lhs.namespace
                .cmp(&rhs.namespace)
                .then(lhs.symbol.cmp(&rhs.symbol))
                .then(lhs.kind.cmp(&rhs.kind))
                .then(lhs.precedence.cmp(&rhs.precedence))
                .then(lhs.target.cmp(&rhs.target))
                .then(lhs.decl_interface_hash.cmp(&rhs.decl_interface_hash))
        });
        notations.dedup();

        for notation in notations {
            if !self.imported_decl_exists(module, &notation.target, &notation.decl_interface_hash) {
                return Err(Diagnostic::error(
                    DiagnosticKind::ImportResolutionError,
                    span,
                    format!(
                        "imported notation `{}` targets missing declaration `{}`",
                        notation.symbol, notation.target
                    ),
                ));
            }
            let resolved = ResolvedNotationDecl {
                kind: notation.kind,
                precedence: notation.precedence,
                symbol: notation.symbol,
                target: ElabGlobalRef::Imported {
                    module: module.clone(),
                    name: notation.target,
                    decl_interface_hash: notation.decl_interface_hash,
                },
                namespace: notation.namespace,
                span,
            };
            self.register_imported_notation(resolved)?;
        }
        Ok(())
    }

    fn imported_decl_exists(&self, module: &Name, target: &Name, hash: &str) -> bool {
        self.state.globals.imported.iter().any(|decl| {
            &decl.name == target
                && matches!(
                    &decl.origin,
                    GlobalOrigin::Imported {
                        module: origin_module,
                        decl_interface_hash,
                    } if origin_module == module && decl_interface_hash == hash
                )
        })
    }

    fn register_imported_notation(&mut self, notation: ResolvedNotationDecl) -> Result<()> {
        if let Some(namespace) = &notation.namespace {
            self.namespace_notations
                .entry(namespace.clone())
                .or_default()
                .push(notation.clone());
        } else {
            self.add_active_notation(notation.clone())?;
        }
        self.state.notations.push(notation);
        Ok(())
    }

    fn resolve_open_item(&mut self, namespace: &SurfaceName, span: Span) -> Result<ResolvedItem> {
        let resolved = self.resolve_namespace(namespace)?;
        self.state
            .open_scopes
            .last_mut()
            .expect("top-level open scope is always present")
            .namespaces
            .push(resolved.clone());
        self.activate_namespace_notations(&resolved, span)?;
        Ok(ResolvedItem::Open {
            namespace: resolved,
            span,
        })
    }

    fn resolve_notation_decl(&self, decl: &NotationDecl) -> Result<ResolvedNotationDecl> {
        let resolved = self.resolve_name(&decl.target)?;
        let target = match resolved {
            ResolvedName::Global(global @ ElabGlobalRef::Local { .. })
            | ResolvedName::Global(global @ ElabGlobalRef::Imported { .. }) => global,
            ResolvedName::Global(ElabGlobalRef::LocalGenerated { .. }) => {
                return Err(Diagnostic::error(
                    DiagnosticKind::InvalidNotation,
                    decl.target.span,
                    "notation target must be a source declaration or imported declaration",
                ));
            }
            ResolvedName::Local(_) => {
                return Err(Diagnostic::error(
                    DiagnosticKind::InvalidNotation,
                    decl.target.span,
                    "notation target must be a global declaration",
                ));
            }
            ResolvedName::Overloaded(_) => {
                return Err(Diagnostic::error(
                    DiagnosticKind::AmbiguousName,
                    decl.target.span,
                    format!(
                        "notation target `{}` is ambiguous",
                        decl.target.parts.join(".")
                    ),
                ));
            }
        };
        let namespace = {
            let current = self.state.current_namespace();
            (!current.is_empty()).then_some(current)
        };
        Ok(ResolvedNotationDecl {
            kind: decl.kind.clone(),
            precedence: decl.precedence,
            symbol: decl.symbol.clone(),
            target,
            namespace,
            span: decl.span,
        })
    }

    fn register_notation(&mut self, notation: ResolvedNotationDecl) -> Result<()> {
        self.add_active_notation(notation.clone())?;
        if let Some(namespace) = &notation.namespace {
            self.namespace_notations
                .entry(namespace.clone())
                .or_default()
                .push(notation.clone());
        }
        self.state.notations.push(notation);
        Ok(())
    }

    fn activate_current_namespace_notations(&mut self, span: Span) -> Result<()> {
        let namespace = self.state.current_namespace();
        if namespace.is_empty() {
            return Ok(());
        }
        for end in 1..=namespace.parts.len() {
            self.activate_namespace_notations(&Name::new(namespace.parts[..end].to_vec()), span)?;
        }
        Ok(())
    }

    fn activate_namespace_notations(&mut self, namespace: &Name, span: Span) -> Result<()> {
        let entries = self
            .namespace_notations
            .get(namespace)
            .cloned()
            .unwrap_or_default();
        for mut entry in entries {
            entry.span = span;
            self.add_active_notation(entry)?;
        }
        Ok(())
    }

    fn add_active_notation(&mut self, notation: ResolvedNotationDecl) -> Result<()> {
        for active in self.active_notations() {
            if active.symbol == notation.symbol
                && (active.kind != notation.kind || active.precedence != notation.precedence)
            {
                return Err(Diagnostic::error(
                    DiagnosticKind::NotationConflict,
                    notation.span,
                    format!("conflicting notation declaration for `{}`", notation.symbol),
                ));
            }
        }
        self.notation_scopes
            .last_mut()
            .expect("top-level notation scope exists")
            .push(notation);
        Ok(())
    }

    fn notation_candidates(
        &self,
        head: &crate::NotationHead,
        span: Span,
    ) -> Result<Vec<ElabGlobalRef>> {
        let mut candidates: Vec<_> = self
            .active_notations()
            .into_iter()
            .filter(|entry| entry.kind == head.kind && entry.symbol == head.symbol)
            .map(|entry| entry.target)
            .collect();
        candidates.sort_by(global_ref_cmp);
        candidates.dedup();
        if candidates.is_empty() {
            return Err(Diagnostic::error(
                DiagnosticKind::AmbiguousNotation,
                span,
                format!("notation `{}` has no active target", head.symbol),
            ));
        }
        Ok(candidates)
    }

    fn active_notations(&self) -> Vec<ResolvedNotationDecl> {
        self.notation_scopes
            .iter()
            .flat_map(|scope| scope.iter().cloned())
            .collect()
    }

    fn resolve_end_item(&mut self, expected: Option<&str>, span: Span) -> Result<()> {
        let Some(actual) = self.state.namespace_stack.pop() else {
            return Err(Diagnostic::error(
                DiagnosticKind::NamespaceMismatch,
                span,
                "`end` without an open namespace",
            ));
        };

        if let Some(expected) = expected {
            if expected != actual {
                return Err(Diagnostic::error(
                    DiagnosticKind::NamespaceMismatch,
                    span,
                    format!("expected `end {actual}`, found `end {expected}`"),
                ));
            }
        }

        self.state
            .open_scopes
            .pop()
            .expect("namespace open scope must exist");
        self.notation_scopes
            .pop()
            .expect("namespace notation scope must exist");
        Ok(())
    }

    fn resolve_value_decl(&mut self, decl: &SurfaceDecl) -> Result<ResolvedDecl> {
        let name = self.qualify_decl_name(&decl.name);
        let local_len = self.state.locals.len();

        let binders = self.resolve_binders(&decl.binders)?;
        let ty = self.resolve_expr(&decl.ty)?;
        let value = decl
            .value
            .as_ref()
            .map(|value| self.resolve_expr(value))
            .transpose()?;

        self.state.locals.truncate(local_len);
        Ok(ResolvedDecl {
            name,
            source_name: decl.name.clone(),
            universe_params: decl.universe_params.clone(),
            binders,
            ty,
            value,
            span: decl.span,
        })
    }

    fn resolve_inductive_item(
        &mut self,
        name: &str,
        universe_params: &[SurfaceUniverseParam],
        binders: &[SurfaceBinder],
        ty: &SurfaceExpr,
        constructors: &[SurfaceCtorDecl],
        span: Span,
    ) -> Result<ResolvedItem> {
        let full_name = self.qualify_decl_name(name);
        let local_len = self.state.locals.len();
        let binders = self.resolve_binders(binders)?;
        let ty = self.resolve_expr(ty)?;
        let generate_recursor = self.inductive_generates_recursor(&ty);

        let decl_index = self.register_resolved_decl_with_index(&full_name, span, false)?;
        let mut resolved_constructors = Vec::new();
        for constructor in constructors {
            let ctor_suffix = Name::from_surface(&constructor.name);
            let ctor_name = full_name.append(&ctor_suffix);
            let ctor_ty = self.resolve_expr(&constructor.ty)?;
            resolved_constructors.push(ResolvedCtorDecl {
                name: ctor_name,
                source_name: constructor.name.parts.join("."),
                ty: ctor_ty,
                span: constructor.span,
            });
        }
        for constructor in &resolved_constructors {
            self.register_generated_decl(&constructor.name, constructor.span, decl_index)?;
        }
        if generate_recursor {
            self.register_generated_decl(&full_name.push("rec"), span, decl_index)?;
        }
        self.state.locals.truncate(local_len);

        Ok(ResolvedItem::Inductive {
            name: full_name,
            universe_params: universe_params.to_vec(),
            binders,
            ty,
            constructors: resolved_constructors,
            span,
        })
    }

    fn resolve_expr(&mut self, expr: &SurfaceExpr) -> Result<ResolvedExpr> {
        match expr {
            SurfaceExpr::Ident {
                name,
                universe_args,
                implicit_mode,
                span,
            } => {
                let resolved = self.resolve_name(name)?;
                Ok(ResolvedExpr::Ident {
                    name: name.clone(),
                    resolved,
                    universe_args: universe_args.clone(),
                    implicit_mode: *implicit_mode,
                    span: *span,
                })
            }
            SurfaceExpr::Sort { level, span } => Ok(ResolvedExpr::Sort {
                level: level.clone(),
                span: *span,
            }),
            SurfaceExpr::App { func, arg, span } => Ok(ResolvedExpr::App {
                func: Box::new(self.resolve_expr(func)?),
                arg: Box::new(self.resolve_expr(arg)?),
                span: *span,
            }),
            SurfaceExpr::Lam {
                binders,
                body,
                span,
            } => {
                let local_len = self.state.locals.len();
                let binders = self.resolve_binders(binders)?;
                let body = self.resolve_expr(body)?;
                self.state.locals.truncate(local_len);
                Ok(ResolvedExpr::Lam {
                    binders,
                    body: Box::new(body),
                    span: *span,
                })
            }
            SurfaceExpr::Pi {
                binders,
                body,
                span,
            } => {
                let local_len = self.state.locals.len();
                let binders = self.resolve_binders(binders)?;
                let body = self.resolve_expr(body)?;
                self.state.locals.truncate(local_len);
                Ok(ResolvedExpr::Pi {
                    binders,
                    body: Box::new(body),
                    span: *span,
                })
            }
            SurfaceExpr::Let {
                name,
                ty,
                value,
                body,
                span,
            } => {
                let ty = ty
                    .as_ref()
                    .map(|ty| self.resolve_expr(ty))
                    .transpose()?
                    .map(Box::new);
                let value = self.resolve_expr(value)?;
                let local_id = self.push_local_for_name(name, name.span);
                let body = self.resolve_expr(body)?;
                self.state.locals.bindings.pop();
                Ok(ResolvedExpr::Let {
                    name: name.clone(),
                    local_id,
                    ty,
                    value: Box::new(value),
                    body: Box::new(body),
                    span: *span,
                })
            }
            SurfaceExpr::Annot { expr, ty, span } => Ok(ResolvedExpr::Annot {
                expr: Box::new(self.resolve_expr(expr)?),
                ty: Box::new(self.resolve_expr(ty)?),
                span: *span,
            }),
            SurfaceExpr::Hole { name, span } => Ok(ResolvedExpr::Hole {
                name: name.clone(),
                span: *span,
            }),
            SurfaceExpr::Notation { head, args, span } => {
                let candidates = self.notation_candidates(head, *span)?;
                Ok(ResolvedExpr::Notation {
                    head: head.clone(),
                    candidates,
                    args: args
                        .iter()
                        .map(|arg| self.resolve_expr(arg))
                        .collect::<Result<_>>()?,
                    span: *span,
                })
            }
        }
    }

    fn resolve_binders(&mut self, binders: &[SurfaceBinder]) -> Result<Vec<ResolvedBinder>> {
        let mut resolved = Vec::new();
        let mut index = 0;
        while index < binders.len() {
            let group_span = binders[index].span;
            let group_start = index;
            while index < binders.len() && binders[index].span == group_span {
                index += 1;
            }

            let mut group = Vec::new();
            for binder in &binders[group_start..index] {
                let ty = binder
                    .ty
                    .as_ref()
                    .map(|ty| self.resolve_expr(ty))
                    .transpose()?
                    .map(Box::new);
                group.push((binder, ty));
            }

            for (binder, ty) in group {
                let local_id = self.push_local_for_binder(binder);
                resolved.push(ResolvedBinder {
                    kind: binder.kind.clone(),
                    local_id,
                    ty,
                    binder_info: binder.binder_info.clone(),
                    span: binder.span,
                });
            }
        }
        Ok(resolved)
    }

    fn push_local_for_binder(&mut self, binder: &SurfaceBinder) -> LocalId {
        match &binder.kind {
            SurfaceBinderKind::Named(name) => self.push_local_for_name(name, binder.span),
            SurfaceBinderKind::Anonymous => {
                let id = self.alloc_local_id();
                self.state.locals.push(LocalBinding {
                    id,
                    name: None,
                    span: binder.span,
                });
                id
            }
        }
    }

    fn push_local_for_name(&mut self, name: &SurfaceName, span: Span) -> LocalId {
        let local_name = name.parts.join(".");
        if self.state.locals.contains_named(&local_name) || !self.global_candidates(name).is_empty()
        {
            self.diagnostics.push(Diagnostic {
                kind: DiagnosticKind::ShadowingWarning,
                severity: DiagnosticSeverity::Warning,
                primary_span: span,
                message: format!("local name `{local_name}` shadows an existing name"),
            });
        }

        let id = self.alloc_local_id();
        self.state.locals.push(LocalBinding {
            id,
            name: Some(local_name),
            span,
        });
        id
    }

    fn alloc_local_id(&mut self) -> LocalId {
        let id = LocalId(self.next_local_id);
        self.next_local_id += 1;
        id
    }

    fn resolve_name(&self, name: &SurfaceName) -> Result<ResolvedName> {
        if name.parts.len() == 1 {
            if let Some(local_ref) = self.state.locals.lookup(&name.parts[0]) {
                return Ok(ResolvedName::Local(local_ref));
            }
        }

        let candidates = self.global_candidates(name);
        if !candidates.is_empty() {
            return Ok(global_candidates_to_resolved(candidates));
        }

        if self.has_future_candidate(name) {
            return Err(Diagnostic::error(
                DiagnosticKind::ForwardReference,
                name.span,
                format!(
                    "`{}` refers to a declaration that is not available yet",
                    name.parts.join(".")
                ),
            ));
        }

        Err(Diagnostic::error(
            DiagnosticKind::UnknownIdentifier,
            name.span,
            format!("unknown identifier `{}`", name.parts.join(".")),
        ))
    }

    fn global_candidates(&self, source: &SurfaceName) -> Vec<ElabGlobalRef> {
        let suffix = Name::from_surface(source);
        for level in self.global_lookup_priority_levels(&suffix, source.parts.len() == 1) {
            let candidates = self.find_global_candidates(&level);
            if !candidates.is_empty() {
                return candidates;
            }
        }
        Vec::new()
    }

    fn find_global_candidates(&self, level: &[LookupPriority]) -> Vec<ElabGlobalRef> {
        let mut current = Vec::new();
        let mut imported = Vec::new();

        for decl in &self.state.globals.current {
            if level
                .iter()
                .filter(|priority| !priority.is_suffix())
                .any(|priority| priority.matches(&decl.name))
            {
                current.push(decl.to_ref());
            }
        }
        if !current.is_empty() {
            current.sort_by(global_ref_cmp);
            current.dedup();
            return current;
        }

        for decl in &self.state.globals.imported {
            if level.iter().any(|priority| priority.matches(&decl.name)) {
                imported.push(decl.to_ref());
            }
        }
        imported.sort_by(global_ref_cmp);
        imported.dedup();
        imported
    }

    fn has_future_candidate(&self, source: &SurfaceName) -> bool {
        let suffix = Name::from_surface(source);
        self.global_lookup_priority_levels(&suffix, source.parts.len() == 1)
            .into_iter()
            .any(|level| {
                level
                    .iter()
                    .filter(|priority| !priority.is_suffix())
                    .any(|priority| {
                        self.future_globals
                            .keys()
                            .any(|future| priority.matches(future))
                    })
            })
    }

    fn global_lookup_priority_levels(
        &self,
        suffix: &Name,
        unqualified: bool,
    ) -> Vec<Vec<LookupPriority>> {
        let mut levels = Vec::new();
        let current = self.state.current_namespace();
        if unqualified {
            if !current.is_empty() {
                levels.push(vec![LookupPriority::Exact(current.append(suffix))]);
            }
            let opened: Vec<_> = self
                .opened_namespaces()
                .into_iter()
                .map(|namespace| LookupPriority::Exact(namespace.append(suffix)))
                .collect();
            if !opened.is_empty() {
                levels.push(opened);
            }
            levels.push(vec![
                LookupPriority::Exact(suffix.clone()),
                LookupPriority::Suffix(suffix.clone()),
            ]);
        } else {
            levels.push(vec![LookupPriority::Exact(suffix.clone())]);
            if !current.is_empty() {
                levels.push(vec![LookupPriority::Exact(current.append(suffix))]);
            }
            let opened: Vec<_> = self
                .opened_namespaces()
                .into_iter()
                .map(|namespace| LookupPriority::Exact(namespace.append(suffix)))
                .collect();
            if !opened.is_empty() {
                levels.push(opened);
            }
            levels.push(vec![LookupPriority::Suffix(suffix.clone())]);
        }
        levels
    }

    fn opened_namespaces(&self) -> Vec<Name> {
        self.state
            .open_scopes
            .iter()
            .flat_map(|scope| scope.namespaces.iter().cloned())
            .collect()
    }

    fn resolve_namespace(&self, namespace: &SurfaceName) -> Result<Name> {
        let suffix = Name::from_surface(namespace);
        let mut candidates = Vec::new();

        let mut candidate_names = vec![suffix.clone()];
        let current = self.state.current_namespace();
        if !current.is_empty() {
            candidate_names.push(current.append(&suffix));
        }
        candidate_names.extend(
            self.opened_namespaces()
                .into_iter()
                .map(|opened| opened.append(&suffix)),
        );

        for candidate in candidate_names {
            if self.namespace_exists(&candidate) {
                candidates.push(candidate);
            }
        }

        candidates.sort();
        candidates.dedup();

        match candidates.as_slice() {
            [] => Err(Diagnostic::error(
                DiagnosticKind::UnknownNamespace,
                namespace.span,
                format!("unknown namespace `{suffix}`"),
            )),
            [candidate] => Ok(candidate.clone()),
            _ => Err(Diagnostic::error(
                DiagnosticKind::AmbiguousName,
                namespace.span,
                format!("ambiguous namespace `{suffix}`"),
            )),
        }
    }

    fn namespace_exists(&self, namespace: &Name) -> bool {
        self.all_visible_global_names()
            .into_iter()
            .any(|name| name.starts_with(namespace) && name.parts.len() > namespace.parts.len())
            || self
                .state
                .notations
                .iter()
                .filter_map(|notation| notation.namespace.as_ref())
                .any(|name| name == namespace || name.starts_with(namespace))
    }

    fn all_visible_global_names(&self) -> Vec<Name> {
        self.state
            .globals
            .current
            .iter()
            .chain(self.state.globals.imported.iter())
            .map(|decl| decl.name.clone())
            .collect()
    }

    fn qualify_decl_name(&self, name: &str) -> Name {
        self.state.current_namespace().push(name)
    }

    fn register_resolved_decl(
        &mut self,
        name: &Name,
        span: Span,
        generated: bool,
    ) -> Result<usize> {
        self.register_resolved_decl_with_index(name, span, generated)
    }

    fn register_resolved_decl_with_index(
        &mut self,
        name: &Name,
        span: Span,
        generated: bool,
    ) -> Result<usize> {
        if self
            .state
            .globals
            .current
            .iter()
            .any(|decl| &decl.name == name)
        {
            return Err(Diagnostic::error(
                DiagnosticKind::DuplicateDeclaration,
                span,
                format!("duplicate declaration `{name}`"),
            ));
        }

        if self
            .state
            .globals
            .imported
            .iter()
            .any(|decl| &decl.name == name)
        {
            self.diagnostics.push(Diagnostic {
                kind: DiagnosticKind::ShadowingWarning,
                severity: DiagnosticSeverity::Warning,
                primary_span: span,
                message: format!("declaration `{name}` shadows an imported declaration"),
            });
        }

        self.future_globals.remove(name);
        let decl_index = self.next_decl_index;
        self.next_decl_index += 1;
        self.state.globals.current.push(GlobalDeclaration {
            name: name.clone(),
            origin: if generated {
                GlobalOrigin::LocalGenerated { decl_index }
            } else {
                GlobalOrigin::Local { decl_index }
            },
            span: Some(span),
        });
        Ok(decl_index)
    }

    fn register_generated_decl(
        &mut self,
        name: &Name,
        span: Span,
        decl_index: usize,
    ) -> Result<()> {
        if self
            .state
            .globals
            .current
            .iter()
            .any(|decl| &decl.name == name)
        {
            return Err(Diagnostic::error(
                DiagnosticKind::DuplicateDeclaration,
                span,
                format!("duplicate declaration `{name}`"),
            ));
        }
        self.future_globals.remove(name);
        self.state.globals.current.push(GlobalDeclaration {
            name: name.clone(),
            origin: GlobalOrigin::LocalGenerated { decl_index },
            span: Some(span),
        });
        Ok(())
    }

    fn record_local_type_alias_value(&mut self, decl: &ResolvedDecl) {
        let Some(value) = &decl.value else {
            return;
        };
        let value = if decl.binders.is_empty() {
            value.clone()
        } else {
            ResolvedExpr::Lam {
                binders: decl.binders.clone(),
                body: Box::new(value.clone()),
                span: decl.span,
            }
        };
        self.type_alias_values
            .insert(decl.name.clone(), TypeAliasValue::Resolved(value));
    }

    fn record_imported_type_alias_values(&mut self, import: &VerifiedImport) {
        for decl in &import.kernel_declarations {
            let Decl::Def {
                name,
                value,
                reducibility: Reducibility::Reducible,
                ..
            } = decl
            else {
                continue;
            };
            self.type_alias_values
                .insert(Name::from_dotted(name), TypeAliasValue::Core(value.clone()));
        }
    }

    fn inductive_generates_recursor(&self, ty: &ResolvedExpr) -> bool {
        self.resolved_inductive_result_shape(ty) == Some(InductiveResultShape::Sort)
    }

    fn resolved_inductive_result_shape(&self, expr: &ResolvedExpr) -> Option<InductiveResultShape> {
        self.resolved_inductive_result_shape_with(expr, &BTreeMap::new(), TYPE_ALIAS_SHAPE_FUEL)
    }

    fn resolved_inductive_result_shape_with(
        &self,
        expr: &ResolvedExpr,
        locals: &BTreeMap<LocalId, ResolvedExpr>,
        fuel: usize,
    ) -> Option<InductiveResultShape> {
        if fuel == 0 {
            return None;
        }

        match expr {
            ResolvedExpr::Sort { .. } => Some(InductiveResultShape::Sort),
            ResolvedExpr::Pi { .. } => Some(InductiveResultShape::Indexed),
            ResolvedExpr::Annot { expr, .. } => {
                self.resolved_inductive_result_shape_with(expr, locals, fuel - 1)
            }
            ResolvedExpr::Let {
                local_id,
                value,
                body,
                ..
            } => {
                let mut body_locals = locals.clone();
                body_locals.insert(*local_id, (**value).clone());
                self.resolved_inductive_result_shape_with(body, &body_locals, fuel - 1)
            }
            ResolvedExpr::Ident {
                resolved: ResolvedName::Local(local),
                ..
            } => locals.get(&local.id).and_then(|value| {
                self.resolved_inductive_result_shape_with(value, locals, fuel - 1)
            }),
            ResolvedExpr::Ident {
                resolved: ResolvedName::Global(global),
                ..
            } => self
                .type_alias_values
                .get(&global_ref_display_name(global))
                .and_then(|value| self.type_alias_value_shape(value, locals, fuel - 1)),
            ResolvedExpr::App { func, arg, .. } => {
                self.resolved_app_result_shape(func, arg, locals, fuel - 1)
            }
            _ => None,
        }
    }

    fn core_inductive_result_shape_with(
        &self,
        expr: &Expr,
        fuel: usize,
    ) -> Option<InductiveResultShape> {
        if fuel == 0 {
            return None;
        }

        match expr {
            Expr::Sort(_) => Some(InductiveResultShape::Sort),
            Expr::Pi { .. } => Some(InductiveResultShape::Indexed),
            Expr::Let { value, body, .. } => instantiate(body, value)
                .ok()
                .and_then(|body| self.core_inductive_result_shape_with(&body, fuel - 1)),
            Expr::Const { name, .. } => self
                .type_alias_values
                .get(&Name::from_dotted(name))
                .and_then(|value| match value {
                    TypeAliasValue::Core(value) => {
                        self.core_inductive_result_shape_with(value, fuel - 1)
                    }
                    TypeAliasValue::Resolved(_) => None,
                }),
            Expr::App(fun, arg) => self
                .core_reduce_app(fun, arg, fuel - 1)
                .and_then(|reduced| self.core_inductive_result_shape_with(&reduced, fuel - 1)),
            _ => None,
        }
    }

    fn type_alias_value_shape(
        &self,
        value: &TypeAliasValue,
        locals: &BTreeMap<LocalId, ResolvedExpr>,
        fuel: usize,
    ) -> Option<InductiveResultShape> {
        match value {
            TypeAliasValue::Resolved(value) => {
                self.resolved_inductive_result_shape_with(value, locals, fuel)
            }
            TypeAliasValue::Core(value) => self.core_inductive_result_shape_with(value, fuel),
        }
    }

    fn resolved_app_result_shape(
        &self,
        func: &ResolvedExpr,
        arg: &ResolvedExpr,
        locals: &BTreeMap<LocalId, ResolvedExpr>,
        fuel: usize,
    ) -> Option<InductiveResultShape> {
        if let Some(result) = self.resolved_apply_one(func, arg, locals, fuel) {
            return match result {
                ResolvedApplied::Expr { expr, locals } => self
                    .resolved_inductive_result_shape_with(&expr, &locals, fuel.saturating_sub(1)),
                ResolvedApplied::Lam(_) => None,
            };
        }

        self.resolved_core_app_result_shape(func, arg, locals, fuel)
    }

    fn resolved_core_app_result_shape(
        &self,
        func: &ResolvedExpr,
        arg: &ResolvedExpr,
        locals: &BTreeMap<LocalId, ResolvedExpr>,
        fuel: usize,
    ) -> Option<InductiveResultShape> {
        let fun = self.resolved_core_lam(func, locals, fuel)?;
        let Expr::Lam { body, .. } = fun else {
            return None;
        };
        self.core_shape_after_resolved_arg(&body, arg, locals, fuel.saturating_sub(1))
    }

    fn resolved_core_lam(
        &self,
        expr: &ResolvedExpr,
        locals: &BTreeMap<LocalId, ResolvedExpr>,
        fuel: usize,
    ) -> Option<Expr> {
        if fuel == 0 {
            return None;
        }

        match expr {
            ResolvedExpr::Annot { expr, .. } => self.resolved_core_lam(expr, locals, fuel - 1),
            ResolvedExpr::Ident {
                resolved: ResolvedName::Global(global),
                ..
            } => {
                let value = self
                    .type_alias_values
                    .get(&global_ref_display_name(global))?;
                match value {
                    TypeAliasValue::Core(value) => self.core_reduce_to_lam(value, fuel - 1),
                    TypeAliasValue::Resolved(_) => None,
                }
            }
            ResolvedExpr::App { func, arg, .. } => {
                let fun = self.resolved_core_lam(func, locals, fuel - 1)?;
                let arg = self.resolved_expr_as_core(arg, locals, fuel - 1)?;
                self.core_reduce_app(&fun, &arg, fuel - 1)
                    .and_then(|reduced| self.core_reduce_to_lam(&reduced, fuel - 1))
            }
            _ => None,
        }
    }

    fn resolved_expr_as_core(
        &self,
        expr: &ResolvedExpr,
        locals: &BTreeMap<LocalId, ResolvedExpr>,
        fuel: usize,
    ) -> Option<Expr> {
        if fuel == 0 {
            return None;
        }

        match expr {
            ResolvedExpr::Sort { level, .. } => Some(Expr::sort(surface_level_as_core(level)?)),
            ResolvedExpr::Ident {
                resolved: ResolvedName::Local(local),
                ..
            } => locals
                .get(&local.id)
                .and_then(|value| self.resolved_expr_as_core(value, locals, fuel - 1))
                .or_else(|| Some(Expr::bvar(local.de_bruijn_index))),
            ResolvedExpr::Ident {
                resolved: ResolvedName::Global(global),
                universe_args,
                ..
            } => Some(Expr::konst(
                global_ref_display_name(global).to_dotted(),
                surface_levels_as_core(universe_args.as_deref())?,
            )),
            ResolvedExpr::App { func, arg, .. } => Some(Expr::app(
                self.resolved_expr_as_core(func, locals, fuel - 1)?,
                self.resolved_expr_as_core(arg, locals, fuel - 1)?,
            )),
            ResolvedExpr::Lam { .. } => self.resolved_lam_as_core_lam(expr, locals, fuel - 1),
            ResolvedExpr::Pi { binders, body, .. } => {
                let body = self.resolved_expr_as_core(body, locals, fuel - 1)?;
                binders.iter().rev().try_fold(body, |body, binder| {
                    let ty = binder.ty.as_ref()?;
                    Some(Expr::pi(
                        resolved_binder_name(binder),
                        self.resolved_expr_as_core(ty, locals, fuel - 1)?,
                        body,
                    ))
                })
            }
            ResolvedExpr::Annot { expr, .. } => self.resolved_expr_as_core(expr, locals, fuel - 1),
            ResolvedExpr::Let {
                local_id,
                value,
                body,
                ..
            } => {
                let mut body_locals = locals.clone();
                body_locals.insert(*local_id, (**value).clone());
                self.resolved_expr_as_core(body, &body_locals, fuel - 1)
            }
            _ => None,
        }
    }

    fn resolved_lam_as_core_lam(
        &self,
        expr: &ResolvedExpr,
        locals: &BTreeMap<LocalId, ResolvedExpr>,
        fuel: usize,
    ) -> Option<Expr> {
        let closure = self.resolved_as_lam(expr, locals, fuel)?;
        self.resolved_lam_closure_as_core_lam(closure, fuel.saturating_sub(1))
    }

    fn resolved_lam_closure_as_core_lam(
        &self,
        closure: ResolvedLamClosure,
        fuel: usize,
    ) -> Option<Expr> {
        if fuel == 0 {
            return None;
        }

        let body = self.resolved_expr_as_core(&closure.body, &closure.locals, fuel - 1)?;
        closure.binders.iter().rev().try_fold(body, |body, binder| {
            let ty = binder.ty.as_ref()?;
            Some(Expr::lam(
                resolved_binder_name(binder),
                self.resolved_expr_as_core(ty, &closure.locals, fuel - 1)?,
                body,
            ))
        })
    }

    fn core_shape_after_resolved_arg(
        &self,
        expr: &Expr,
        arg: &ResolvedExpr,
        arg_locals: &BTreeMap<LocalId, ResolvedExpr>,
        fuel: usize,
    ) -> Option<InductiveResultShape> {
        if fuel == 0 {
            return None;
        }

        match expr {
            Expr::Sort(_) => Some(InductiveResultShape::Sort),
            Expr::Pi { .. } => Some(InductiveResultShape::Indexed),
            Expr::BVar(0) => self.resolved_inductive_result_shape_with(arg, arg_locals, fuel - 1),
            Expr::Let { value, body, .. } => instantiate(body, value).ok().and_then(|body| {
                self.core_shape_after_resolved_arg(&body, arg, arg_locals, fuel - 1)
            }),
            Expr::Const { name, .. } => self
                .type_alias_values
                .get(&Name::from_dotted(name))
                .and_then(|value| match value {
                    TypeAliasValue::Core(value) => {
                        self.core_shape_after_resolved_arg(value, arg, arg_locals, fuel - 1)
                    }
                    TypeAliasValue::Resolved(value) => {
                        self.resolved_inductive_result_shape_with(value, arg_locals, fuel - 1)
                    }
                }),
            Expr::App(fun, core_arg) => {
                if let Some(shape) =
                    self.core_resolved_app_result_shape(fun, core_arg, arg, arg_locals, fuel - 1)
                {
                    return Some(shape);
                }
                self.core_reduce_app(fun, core_arg, fuel - 1)
                    .and_then(|reduced| {
                        self.core_shape_after_resolved_arg(&reduced, arg, arg_locals, fuel - 1)
                    })
            }
            _ => None,
        }
    }

    fn core_resolved_app_result_shape(
        &self,
        fun: &Expr,
        core_arg: &Expr,
        arg: &ResolvedExpr,
        arg_locals: &BTreeMap<LocalId, ResolvedExpr>,
        fuel: usize,
    ) -> Option<InductiveResultShape> {
        if fuel == 0 || !matches!(core_arg, Expr::BVar(0)) {
            return None;
        }

        let Expr::Const { name, .. } = fun else {
            return None;
        };
        let TypeAliasValue::Resolved(value) =
            self.type_alias_values.get(&Name::from_dotted(name))?
        else {
            return None;
        };

        match self.resolved_apply_one(value, arg, arg_locals, fuel - 1)? {
            ResolvedApplied::Expr { expr, locals } => {
                self.resolved_inductive_result_shape_with(&expr, &locals, fuel - 1)
            }
            ResolvedApplied::Lam(_) => None,
        }
    }

    fn resolved_apply_one(
        &self,
        func: &ResolvedExpr,
        arg: &ResolvedExpr,
        locals: &BTreeMap<LocalId, ResolvedExpr>,
        fuel: usize,
    ) -> Option<ResolvedApplied> {
        let mut closure = self.resolved_as_lam(func, locals, fuel)?;
        let (binder, rest) = closure.binders.split_first()?;
        for (id, value) in locals {
            closure.locals.entry(*id).or_insert_with(|| value.clone());
        }
        closure.locals.insert(binder.local_id, arg.clone());
        if rest.is_empty() {
            Some(ResolvedApplied::Expr {
                expr: closure.body,
                locals: closure.locals,
            })
        } else {
            Some(ResolvedApplied::Lam(ResolvedLamClosure {
                binders: rest.to_vec(),
                body: closure.body,
                locals: closure.locals,
            }))
        }
    }

    fn resolved_as_lam(
        &self,
        expr: &ResolvedExpr,
        locals: &BTreeMap<LocalId, ResolvedExpr>,
        fuel: usize,
    ) -> Option<ResolvedLamClosure> {
        if fuel == 0 {
            return None;
        }

        match expr {
            ResolvedExpr::Annot { expr, .. } => self.resolved_as_lam(expr, locals, fuel - 1),
            ResolvedExpr::Let {
                local_id,
                value,
                body,
                ..
            } => {
                let mut body_locals = locals.clone();
                body_locals.insert(*local_id, (**value).clone());
                self.resolved_as_lam(body, &body_locals, fuel - 1)
            }
            ResolvedExpr::Ident {
                resolved: ResolvedName::Local(local),
                ..
            } => locals
                .get(&local.id)
                .and_then(|value| self.resolved_as_lam(value, locals, fuel - 1)),
            ResolvedExpr::Ident {
                resolved: ResolvedName::Global(global),
                ..
            } => {
                let value = self
                    .type_alias_values
                    .get(&global_ref_display_name(global))?;
                match value {
                    TypeAliasValue::Resolved(value) => {
                        self.resolved_as_lam(value, locals, fuel - 1)
                    }
                    TypeAliasValue::Core(_) => None,
                }
            }
            ResolvedExpr::App { func, arg, .. } => {
                match self.resolved_apply_one(func, arg, locals, fuel - 1)? {
                    ResolvedApplied::Expr { expr, locals } => {
                        self.resolved_as_lam(&expr, &locals, fuel - 1)
                    }
                    ResolvedApplied::Lam(closure) => Some(closure),
                }
            }
            ResolvedExpr::Lam { binders, body, .. } => Some(ResolvedLamClosure {
                binders: binders.clone(),
                body: (**body).clone(),
                locals: locals.clone(),
            }),
            _ => None,
        }
    }

    fn core_reduce_app(&self, fun: &Expr, arg: &Expr, fuel: usize) -> Option<Expr> {
        let fun = self.core_reduce_to_lam(fun, fuel)?;
        let Expr::Lam { body, .. } = fun else {
            return None;
        };
        instantiate(&body, arg).ok()
    }

    fn core_reduce_to_lam(&self, expr: &Expr, fuel: usize) -> Option<Expr> {
        if fuel == 0 {
            return None;
        }

        match expr {
            Expr::Lam { .. } => Some(expr.clone()),
            Expr::Let { value, body, .. } => instantiate(body, value)
                .ok()
                .and_then(|body| self.core_reduce_to_lam(&body, fuel - 1)),
            Expr::Const { name, .. } => {
                let value = self.type_alias_values.get(&Name::from_dotted(name))?;
                match value {
                    TypeAliasValue::Core(value) => self.core_reduce_to_lam(value, fuel - 1),
                    TypeAliasValue::Resolved(value) => {
                        self.resolved_lam_as_core_lam(value, &BTreeMap::new(), fuel - 1)
                    }
                }
            }
            Expr::App(fun, arg) => self
                .core_reduce_app(fun, arg, fuel - 1)
                .and_then(|reduced| self.core_reduce_to_lam(&reduced, fuel - 1)),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
enum LookupPriority {
    Exact(Name),
    Suffix(Name),
}

impl LookupPriority {
    fn is_suffix(&self) -> bool {
        matches!(self, Self::Suffix(_))
    }

    fn matches(&self, name: &Name) -> bool {
        match self {
            Self::Exact(expected) => name == expected,
            Self::Suffix(suffix) => name.ends_with(suffix),
        }
    }
}

fn global_candidates_to_resolved(candidates: Vec<ElabGlobalRef>) -> ResolvedName {
    match candidates.as_slice() {
        [candidate] => ResolvedName::Global(candidate.clone()),
        _ => ResolvedName::Overloaded(candidates),
    }
}

fn parser_imports_from_verified(imports: &[VerifiedImport]) -> Vec<ParserImport> {
    imports
        .iter()
        .map(|import| ParserImport {
            module: import.module.parts.clone(),
            notations: import
                .notations
                .iter()
                .map(|notation| ParserImportedNotation {
                    kind: notation.kind.clone(),
                    precedence: notation.precedence,
                    symbol: notation.symbol.clone(),
                    namespace: notation
                        .namespace
                        .as_ref()
                        .map(|namespace| namespace.parts.clone()),
                })
                .collect(),
        })
        .collect()
}

fn global_ref_cmp(lhs: &ElabGlobalRef, rhs: &ElabGlobalRef) -> std::cmp::Ordering {
    global_ref_display_name(lhs)
        .cmp(&global_ref_display_name(rhs))
        .then(global_ref_kind_rank(lhs).cmp(&global_ref_kind_rank(rhs)))
        .then_with(|| match (lhs, rhs) {
            (
                ElabGlobalRef::Local {
                    decl_index: lhs_index,
                    ..
                },
                ElabGlobalRef::Local {
                    decl_index: rhs_index,
                    ..
                },
            )
            | (
                ElabGlobalRef::LocalGenerated {
                    decl_index: lhs_index,
                    ..
                },
                ElabGlobalRef::LocalGenerated {
                    decl_index: rhs_index,
                    ..
                },
            ) => lhs_index.cmp(rhs_index),
            (
                ElabGlobalRef::Imported {
                    module: lhs_module,
                    decl_interface_hash: lhs_hash,
                    ..
                },
                ElabGlobalRef::Imported {
                    module: rhs_module,
                    decl_interface_hash: rhs_hash,
                    ..
                },
            ) => lhs_module.cmp(rhs_module).then(lhs_hash.cmp(rhs_hash)),
            _ => std::cmp::Ordering::Equal,
        })
}

fn global_ref_display_name(global_ref: &ElabGlobalRef) -> Name {
    match global_ref {
        ElabGlobalRef::Local { name, .. }
        | ElabGlobalRef::LocalGenerated { name, .. }
        | ElabGlobalRef::Imported { name, .. } => name.clone(),
    }
}

fn global_ref_kind_rank(global_ref: &ElabGlobalRef) -> u8 {
    match global_ref {
        ElabGlobalRef::Local { .. } => 0,
        ElabGlobalRef::LocalGenerated { .. } => 1,
        ElabGlobalRef::Imported { .. } => 2,
    }
}

fn resolved_binder_name(binder: &ResolvedBinder) -> String {
    match &binder.kind {
        SurfaceBinderKind::Named(name) => name.parts.join("."),
        SurfaceBinderKind::Anonymous => "_".to_owned(),
    }
}

fn surface_levels_as_core(levels: Option<&[SurfaceLevel]>) -> Option<Vec<Level>> {
    levels
        .unwrap_or_default()
        .iter()
        .map(surface_level_as_core)
        .collect()
}

fn surface_level_as_core(level: &SurfaceLevel) -> Option<Level> {
    match level {
        SurfaceLevel::Nat { value, .. } => {
            if *value > TYPE_ALIAS_SHAPE_MAX_NUMERIC_LEVEL {
                return None;
            }
            Some((0..*value).fold(Level::zero(), |level, _| Level::succ(level)))
        }
        SurfaceLevel::Param { name, .. } => Some(Level::param(name.clone())),
        SurfaceLevel::Succ { level, .. } => Some(Level::succ(surface_level_as_core(level)?)),
        SurfaceLevel::Max { lhs, rhs, .. } => Some(Level::max(
            surface_level_as_core(lhs)?,
            surface_level_as_core(rhs)?,
        )),
        SurfaceLevel::IMax { lhs, rhs, .. } => Some(Level::imax(
            surface_level_as_core(lhs)?,
            surface_level_as_core(rhs)?,
        )),
    }
}

fn collect_current_module_declarations(module: &SurfaceModule) -> Result<BTreeMap<Name, Span>> {
    let mut namespace_stack = Vec::new();
    let mut declarations = BTreeMap::new();

    for item in &module.items {
        match item {
            SurfaceItem::Namespace { name, .. } => namespace_stack.push(name.clone()),
            SurfaceItem::End { name, span } => {
                let Some(actual) = namespace_stack.pop() else {
                    return Err(Diagnostic::error(
                        DiagnosticKind::NamespaceMismatch,
                        *span,
                        "`end` without an open namespace",
                    ));
                };
                if let Some(expected) = name {
                    if expected != &actual {
                        return Err(Diagnostic::error(
                            DiagnosticKind::NamespaceMismatch,
                            *span,
                            format!("expected `end {actual}`, found `end {expected}`"),
                        ));
                    }
                }
            }
            SurfaceItem::Def(decl) | SurfaceItem::Theorem(decl) | SurfaceItem::Axiom(decl) => {
                insert_decl_name(
                    &mut declarations,
                    qualify_name(&namespace_stack, &decl.name),
                    decl.span,
                )?;
            }
            SurfaceItem::Inductive {
                name,
                ty,
                constructors,
                span,
                ..
            } => {
                let inductive_name = qualify_name(&namespace_stack, name);
                insert_decl_name(&mut declarations, inductive_name.clone(), *span)?;
                for constructor in constructors {
                    insert_decl_name(
                        &mut declarations,
                        inductive_name.append(&Name::from_surface(&constructor.name)),
                        constructor.span,
                    )?;
                }
                if surface_inductive_generates_recursor(ty) {
                    insert_decl_name(&mut declarations, inductive_name.push("rec"), *span)?;
                }
            }
            SurfaceItem::Import { .. } | SurfaceItem::Open { .. } | SurfaceItem::Notation(_) => {}
        }
    }

    if let Some(open_namespace) = namespace_stack.last() {
        return Err(Diagnostic::error(
            DiagnosticKind::NamespaceMismatch,
            module.span,
            format!("namespace `{open_namespace}` is not closed"),
        ));
    }

    Ok(declarations)
}

fn qualify_name(namespace_stack: &[String], name: &str) -> Name {
    let mut parts = namespace_stack.to_vec();
    parts.push(name.to_owned());
    Name::new(parts)
}

fn insert_decl_name(declarations: &mut BTreeMap<Name, Span>, name: Name, span: Span) -> Result<()> {
    if declarations.insert(name.clone(), span).is_some() {
        return Err(Diagnostic::error(
            DiagnosticKind::DuplicateDeclaration,
            span,
            format!("duplicate declaration `{name}`"),
        ));
    }
    Ok(())
}

fn surface_inductive_generates_recursor(ty: &SurfaceExpr) -> bool {
    match ty {
        SurfaceExpr::Sort { .. } => true,
        SurfaceExpr::Annot { expr, .. } => surface_inductive_generates_recursor(expr),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SurfaceName;

    fn std_nat_import() -> VerifiedImport {
        VerifiedImport {
            module: Name::from_dotted("Std.Nat"),
            export_hash: "sha256:nat".to_owned(),
            declarations: vec![
                ImportedDeclaration {
                    name: Name::from_dotted("Nat"),
                    decl_interface_hash: "sha256:Nat".to_owned(),
                    binder_infos: Vec::new(),
                    domain_infos: Vec::new(),
                    type_value_metadata: None,
                },
                ImportedDeclaration {
                    name: Name::from_dotted("Nat.zero"),
                    decl_interface_hash: "sha256:Nat.zero".to_owned(),
                    binder_infos: Vec::new(),
                    domain_infos: Vec::new(),
                    type_value_metadata: None,
                },
                ImportedDeclaration {
                    name: Name::from_dotted("Nat.add"),
                    decl_interface_hash: "sha256:Nat.add".to_owned(),
                    binder_infos: Vec::new(),
                    domain_infos: Vec::new(),
                    type_value_metadata: None,
                },
            ],
            notations: Vec::new(),
            kernel_declarations: Vec::new(),
        }
    }

    fn other_add_import() -> VerifiedImport {
        VerifiedImport {
            module: Name::from_dotted("Other"),
            export_hash: "sha256:other".to_owned(),
            declarations: vec![ImportedDeclaration {
                name: Name::from_dotted("Int.add"),
                decl_interface_hash: "sha256:Int.add".to_owned(),
                binder_infos: Vec::new(),
                domain_infos: Vec::new(),
                type_value_metadata: None,
            }],
            notations: Vec::new(),
            kernel_declarations: Vec::new(),
        }
    }

    fn nested_namespace_import() -> VerifiedImport {
        VerifiedImport {
            module: Name::from_dotted("Nested"),
            export_hash: "sha256:nested".to_owned(),
            declarations: vec![ImportedDeclaration {
                name: Name::from_dotted("A.B.x"),
                decl_interface_hash: "sha256:A.B.x".to_owned(),
                binder_infos: Vec::new(),
                domain_infos: Vec::new(),
                type_value_metadata: None,
            }],
            notations: Vec::new(),
            kernel_declarations: Vec::new(),
        }
    }

    fn mixed_zero_import() -> VerifiedImport {
        VerifiedImport {
            module: Name::from_dotted("Mixed"),
            export_hash: "sha256:mixed".to_owned(),
            declarations: vec![
                ImportedDeclaration {
                    name: Name::from_dotted("Nat"),
                    decl_interface_hash: "sha256:Mixed.Nat".to_owned(),
                    binder_infos: Vec::new(),
                    domain_infos: Vec::new(),
                    type_value_metadata: None,
                },
                ImportedDeclaration {
                    name: Name::from_dotted("Int"),
                    decl_interface_hash: "sha256:Mixed.Int".to_owned(),
                    binder_infos: Vec::new(),
                    domain_infos: Vec::new(),
                    type_value_metadata: None,
                },
                ImportedDeclaration {
                    name: Name::from_dotted("zero"),
                    decl_interface_hash: "sha256:Mixed.zero".to_owned(),
                    binder_infos: Vec::new(),
                    domain_infos: Vec::new(),
                    type_value_metadata: None,
                },
                ImportedDeclaration {
                    name: Name::from_dotted("Nat.zero"),
                    decl_interface_hash: "sha256:Mixed.Nat.zero".to_owned(),
                    binder_infos: Vec::new(),
                    domain_infos: Vec::new(),
                    type_value_metadata: None,
                },
                ImportedDeclaration {
                    name: Name::from_dotted("Int.zero"),
                    decl_interface_hash: "sha256:Mixed.Int.zero".to_owned(),
                    binder_infos: Vec::new(),
                    domain_infos: Vec::new(),
                    type_value_metadata: None,
                },
            ],
            notations: Vec::new(),
            kernel_declarations: Vec::new(),
        }
    }

    fn std_nat_notation_import() -> VerifiedImport {
        let mut import = std_nat_import();
        import.notations = vec![ImportedNotation {
            kind: NotationKind::Infixl,
            precedence: 65,
            symbol: "+".to_owned(),
            target: Name::from_dotted("Nat.add"),
            decl_interface_hash: "sha256:Nat.add".to_owned(),
            namespace: Some(Name::from_dotted("Nat")),
        }];
        import
    }

    fn resolve(source: &str, imports: &[VerifiedImport]) -> Result<ResolvedModule> {
        resolve_source(FileId(0), Name::from_dotted("Scratch"), source, imports)
    }

    fn ident_resolution(expr: &ResolvedExpr) -> &ResolvedName {
        let ResolvedExpr::Ident { resolved, .. } = expr else {
            panic!("expected identifier");
        };
        resolved
    }

    #[test]
    fn matches_verified_imports_and_warns_on_duplicate_source_import() {
        let resolved = resolve(
            r#"
import Std.Nat
import Std.Nat
axiom x : Nat
"#,
            &[std_nat_import()],
        )
        .expect("module should resolve");

        assert_eq!(resolved.state.imports.len(), 1);
        assert_eq!(resolved.state.globals.imported.len(), 3);
        assert!(matches!(
            resolved.items[1],
            ResolvedItem::Import {
                duplicate: true,
                ..
            }
        ));
        assert_eq!(resolved.diagnostics.len(), 1);
        assert_eq!(
            resolved.diagnostics[0].kind,
            DiagnosticKind::DuplicateImportWarning
        );
        assert_eq!(
            resolved.diagnostics[0].severity,
            DiagnosticSeverity::Warning
        );
    }

    #[test]
    fn rejects_missing_verified_import() {
        let err = resolve("import Std.Nat\naxiom x : Nat", &[])
            .expect_err("missing verified import must fail");
        assert_eq!(err.kind, DiagnosticKind::ImportResolutionError);
    }

    #[test]
    fn rejects_opening_leaf_declaration_name() {
        let err = resolve("import Std.Nat\nopen zero", &[std_nat_import()])
            .expect_err("leaf declaration names are not namespaces");
        assert_eq!(err.kind, DiagnosticKind::UnknownNamespace);
    }

    #[test]
    fn rejects_opening_unopened_namespace_suffix() {
        let err = resolve("import Nested\nopen B", &[nested_namespace_import()])
            .expect_err("namespace suffix without visible prefix must not open");
        assert_eq!(err.kind, DiagnosticKind::UnknownNamespace);
    }

    #[test]
    fn namespace_open_scope_is_lexical() {
        let resolved = resolve(
            r#"
import Std.Nat
namespace Demo
open Nat
def z : Nat := zero
end Demo
"#,
            &[std_nat_import()],
        )
        .expect("module should resolve");
        assert_eq!(resolved.state.open_scopes.len(), 1);
        assert!(resolved.state.open_scopes[0].namespaces.is_empty());
    }

    #[test]
    fn closed_current_namespace_does_not_leak_short_names() {
        let err = resolve(
            r#"
axiom Nat : Type
namespace Nat
axiom zero : Nat
end Nat
def bad : Nat := zero
"#,
            &[],
        )
        .expect_err("closed namespace member must not resolve by short name");
        assert_eq!(err.kind, DiagnosticKind::UnknownIdentifier);
        assert!(err.message.contains("zero"));
    }

    #[test]
    fn future_closed_namespace_member_is_not_a_forward_reference() {
        let err = resolve(
            r#"
axiom Nat : Type
def bad : Nat := zero
namespace Nat
axiom zero : Nat
end Nat
"#,
            &[],
        )
        .expect_err("future closed namespace member must not be visible by short name");
        assert_eq!(err.kind, DiagnosticKind::UnknownIdentifier);
        assert!(err.message.contains("zero"));
    }

    #[test]
    fn local_declaration_takes_priority_over_imported_short_name() {
        let resolved = resolve(
            r#"
import Std.Nat
def zero : Nat := Nat.zero
def use : Nat := zero
"#,
            &[std_nat_import()],
        )
        .expect("module should resolve");

        let ResolvedItem::Def(use_decl) = &resolved.items[2] else {
            panic!("expected second def");
        };
        let value = use_decl.value.as_ref().expect("def value");
        match ident_resolution(value) {
            ResolvedName::Global(ElabGlobalRef::Local { decl_index, name }) => {
                assert_eq!(*decl_index, 0);
                assert_eq!(name.to_dotted(), "zero");
            }
            other => panic!("expected local global, got {other:?}"),
        }
    }

    #[test]
    fn ambiguous_short_global_name_is_kept_as_overloaded() {
        let resolved = resolve(
            r#"
import Std.Nat
import Other
def use : Nat := add
"#,
            &[std_nat_import(), other_add_import()],
        )
        .expect("module should resolve");

        let ResolvedItem::Def(use_decl) = &resolved.items[2] else {
            panic!("expected def");
        };
        let value = use_decl.value.as_ref().expect("def value");
        match ident_resolution(value) {
            ResolvedName::Overloaded(candidates) => assert_eq!(candidates.len(), 2),
            other => panic!("expected overloaded name, got {other:?}"),
        }
    }

    #[test]
    fn opened_namespace_short_names_share_one_priority_level() {
        let resolved = resolve(
            r#"
import Mixed
open Nat
open Int
def use : Nat := zero
"#,
            &[mixed_zero_import()],
        )
        .expect("module should resolve");

        let ResolvedItem::Def(use_decl) = &resolved.items[3] else {
            panic!("expected def");
        };
        let value = use_decl.value.as_ref().expect("def value");
        match ident_resolution(value) {
            ResolvedName::Overloaded(candidates) => assert_eq!(candidates.len(), 2),
            other => panic!("expected overloaded opened name, got {other:?}"),
        }
    }

    #[test]
    fn imported_root_and_suffix_short_names_share_one_priority_level() {
        let resolved = resolve(
            r#"
import Mixed
def use : Nat := zero
"#,
            &[mixed_zero_import()],
        )
        .expect("module should resolve");

        let ResolvedItem::Def(use_decl) = &resolved.items[1] else {
            panic!("expected def");
        };
        let value = use_decl.value.as_ref().expect("def value");
        match ident_resolution(value) {
            ResolvedName::Overloaded(candidates) => assert_eq!(candidates.len(), 3),
            other => panic!("expected overloaded imported short name, got {other:?}"),
        }
    }

    #[test]
    fn rejects_forward_reference_to_later_declaration() {
        let err = resolve(
            r#"
import Std.Nat
def f : Nat := g
def g : Nat := Nat.zero
"#,
            &[std_nat_import()],
        )
        .expect_err("forward reference must fail");
        assert_eq!(err.kind, DiagnosticKind::ForwardReference);
    }

    #[test]
    fn rejects_duplicate_current_module_declarations() {
        let err = resolve(
            r#"
import Std.Nat
def x : Nat := Nat.zero
def x : Nat := Nat.zero
"#,
            &[std_nat_import()],
        )
        .expect_err("duplicate declaration must fail");
        assert_eq!(err.kind, DiagnosticKind::DuplicateDeclaration);
    }

    #[test]
    fn rejects_same_block_constructor_reference() {
        let err = resolve(
            r#"
inductive T : Type where
| c1 : T
| c2 : T.c1
"#,
            &[],
        )
        .expect_err("constructors in the same block must not be visible");
        assert_eq!(err.kind, DiagnosticKind::ForwardReference);
    }

    #[test]
    fn rejects_namespace_mismatch() {
        let err = resolve(
            r#"
namespace A
end B
"#,
            &[],
        )
        .expect_err("namespace mismatch must fail");
        assert_eq!(err.kind, DiagnosticKind::NamespaceMismatch);
    }

    #[test]
    fn local_context_shadows_global_names() {
        let resolved = resolve(
            r#"
import Std.Nat
def id (Nat : Type) (x : Nat) : Nat := x
"#,
            &[std_nat_import()],
        )
        .expect("module should resolve");

        let ResolvedItem::Def(id_decl) = &resolved.items[1] else {
            panic!("expected def");
        };
        let second_binder = &id_decl.binders[1];
        let binder_ty = second_binder.ty.as_deref().expect("binder type");
        assert!(matches!(
            ident_resolution(binder_ty),
            ResolvedName::Local(_)
        ));

        let value = id_decl.value.as_ref().expect("def value");
        assert!(matches!(ident_resolution(value), ResolvedName::Local(_)));
        assert!(resolved
            .diagnostics
            .iter()
            .any(|diag| diag.kind == DiagnosticKind::ShadowingWarning));
    }

    #[test]
    fn qualified_names_prefer_exact_global_name_inside_namespace() {
        let resolved = resolve(
            r#"
import Std.Nat
namespace Nat
def one : Nat := Nat.zero
end Nat
"#,
            &[std_nat_import()],
        )
        .expect("module should resolve");

        let ResolvedItem::Def(one_decl) = &resolved.items[2] else {
            panic!("expected def");
        };
        let value = one_decl.value.as_ref().expect("def value");
        match ident_resolution(value) {
            ResolvedName::Global(ElabGlobalRef::Imported { name, .. }) => {
                assert_eq!(name.to_dotted(), "Nat.zero");
            }
            other => panic!("expected imported Nat.zero, got {other:?}"),
        }
    }

    #[test]
    fn open_uses_resolved_namespace_name() {
        let resolved = resolve(
            r#"
import Std.Nat
open Nat
def z : Nat := zero
"#,
            &[std_nat_import()],
        )
        .expect("module should resolve");

        let ResolvedItem::Open { namespace, .. } = &resolved.items[1] else {
            panic!("expected open item");
        };
        assert_eq!(namespace.to_dotted(), "Nat");

        let ResolvedItem::Def(decl) = &resolved.items[2] else {
            panic!("expected def");
        };
        let value = decl.value.as_ref().expect("def value");
        match ident_resolution(value) {
            ResolvedName::Global(ElabGlobalRef::Imported { name, .. }) => {
                assert_eq!(name.to_dotted(), "Nat.zero");
            }
            other => panic!("expected imported Nat.zero, got {other:?}"),
        }
    }

    #[test]
    fn resolves_namespaced_notation_after_open() {
        let resolved = resolve(
            r#"
import Std.Nat
namespace Nat
infixl:65 " + " => add
end Nat
open Nat
def z : Nat := Nat.zero + Nat.zero
"#,
            &[std_nat_import()],
        )
        .expect("namespaced notation should resolve after open");

        let ResolvedItem::Notation(notation) = &resolved.items[2] else {
            panic!("expected notation item");
        };
        assert_eq!(
            notation.namespace.as_ref().map(Name::to_dotted).as_deref(),
            Some("Nat")
        );
        match &notation.target {
            ElabGlobalRef::Imported { name, .. } => assert_eq!(name.to_dotted(), "Nat.add"),
            other => panic!("expected imported Nat.add target, got {other:?}"),
        }

        let ResolvedItem::Def(decl) = &resolved.items[5] else {
            panic!("expected def");
        };
        let value = decl.value.as_ref().expect("def value");
        let ResolvedExpr::Notation {
            candidates, args, ..
        } = value
        else {
            panic!("expected notation expression");
        };
        assert_eq!(args.len(), 2);
        assert_eq!(candidates.len(), 1);
    }

    #[test]
    fn resolves_imported_notation_after_open() {
        let resolved = resolve(
            r#"
import Std.Nat
open Nat
def z : Nat := Nat.zero + Nat.zero
"#,
            &[std_nat_notation_import()],
        )
        .expect("imported notation should resolve after open");

        let ResolvedItem::Def(decl) = &resolved.items[2] else {
            panic!("expected def");
        };
        let value = decl.value.as_ref().expect("def value");
        let ResolvedExpr::Notation { candidates, .. } = value else {
            panic!("expected notation expression");
        };
        assert_eq!(candidates.len(), 1);
        match &candidates[0] {
            ElabGlobalRef::Imported { name, .. } => assert_eq!(name.to_dotted(), "Nat.add"),
            other => panic!("expected imported Nat.add target, got {other:?}"),
        }
    }

    #[test]
    fn parses_imported_notation_through_public_staged_api() {
        let imports = [std_nat_notation_import()];
        let module = parse_module_with_verified_imports(
            FileId(0),
            r#"
import Std.Nat
open Nat
def z : Nat := Nat.zero + Nat.zero
"#,
            &imports,
        )
        .expect("public parser should use verified import notation metadata");

        let resolved = resolve_module(Name::from_dotted("Scratch"), &module, &imports)
            .expect("parsed module should resolve through the staged API");

        let ResolvedItem::Def(decl) = &resolved.items[2] else {
            panic!("expected def");
        };
        let value = decl.value.as_ref().expect("def value");
        assert!(matches!(value, ResolvedExpr::Notation { .. }));
    }

    #[test]
    fn surface_name_roundtrips_to_name() {
        let span = Span::new(FileId(0), 0, 3);
        let surface = SurfaceName {
            parts: vec!["A".to_owned(), "B".to_owned()],
            span,
        };
        assert_eq!(Name::from_surface(&surface).to_dotted(), "A.B");
    }
}
