use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use npa_kernel::Decl;

use crate::{
    parse_module, BinderInfo, Diagnostic, DiagnosticKind, DiagnosticSeverity, FileId, ImplicitMode,
    NotationDecl, Result, Span, SurfaceBinder, SurfaceBinderKind, SurfaceCtorDecl, SurfaceDecl,
    SurfaceExpr, SurfaceItem, SurfaceLevel, SurfaceModule, SurfaceName, SurfaceUniverseParam,
};

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
    pub kernel_declarations: Vec<Decl>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportedDeclaration {
    pub name: Name,
    pub decl_interface_hash: String,
    pub binder_infos: Vec<BinderInfo>,
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
    Notation(NotationDecl),
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
    let module = parse_module(file_id, source)?;
    resolve_module(current_module, &module, verified_imports)
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
    diagnostics: Vec<Diagnostic>,
    next_decl_index: usize,
    next_local_id: u32,
}

impl<'a> Resolver<'a> {
    fn new(current_module: Name, verified_imports: &'a [VerifiedImport]) -> Self {
        Self {
            state: FrontendState::new(current_module),
            verified_imports,
            future_globals: BTreeMap::new(),
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
            SurfaceItem::Notation(decl) => Ok(ResolvedItem::Notation(decl.clone())),
            SurfaceItem::Def(decl) => {
                let resolved = self.resolve_value_decl(decl)?;
                self.register_resolved_decl(&resolved.name, resolved.span, false)?;
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

    fn resolve_open_item(&mut self, namespace: &SurfaceName, span: Span) -> Result<ResolvedItem> {
        let resolved = self.resolve_namespace(namespace)?;
        self.state
            .open_scopes
            .last_mut()
            .expect("top-level open scope is always present")
            .namespaces
            .push(resolved.clone());
        Ok(ResolvedItem::Open {
            namespace: resolved,
            span,
        })
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

        let decl_index = self.register_resolved_decl_with_index(&full_name, span, false)?;
        let mut resolved_constructors = Vec::new();
        for constructor in constructors {
            let ctor_name = full_name.push(constructor.name.clone());
            let ctor_ty = self.resolve_expr(&constructor.ty)?;
            resolved_constructors.push(ResolvedCtorDecl {
                name: ctor_name,
                source_name: constructor.name.clone(),
                ty: ctor_ty,
                span: constructor.span,
            });
        }
        for constructor in &resolved_constructors {
            self.register_generated_decl(&constructor.name, constructor.span, decl_index)?;
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
                    implicit_mode: implicit_mode.clone(),
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
            SurfaceExpr::Notation { head, args, span } => Ok(ResolvedExpr::Notation {
                head: head.clone(),
                args: args
                    .iter()
                    .map(|arg| self.resolve_expr(arg))
                    .collect::<Result<_>>()?,
                span: *span,
            }),
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
                constructors,
                span,
                ..
            } => {
                let inductive_name = qualify_name(&namespace_stack, name);
                insert_decl_name(&mut declarations, inductive_name.clone(), *span)?;
                for constructor in constructors {
                    insert_decl_name(
                        &mut declarations,
                        inductive_name.push(constructor.name.clone()),
                        constructor.span,
                    )?;
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
                },
                ImportedDeclaration {
                    name: Name::from_dotted("Nat.zero"),
                    decl_interface_hash: "sha256:Nat.zero".to_owned(),
                    binder_infos: Vec::new(),
                },
                ImportedDeclaration {
                    name: Name::from_dotted("Nat.add"),
                    decl_interface_hash: "sha256:Nat.add".to_owned(),
                    binder_infos: Vec::new(),
                },
            ],
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
            }],
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
            }],
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
                },
                ImportedDeclaration {
                    name: Name::from_dotted("Int"),
                    decl_interface_hash: "sha256:Mixed.Int".to_owned(),
                    binder_infos: Vec::new(),
                },
                ImportedDeclaration {
                    name: Name::from_dotted("zero"),
                    decl_interface_hash: "sha256:Mixed.zero".to_owned(),
                    binder_infos: Vec::new(),
                },
                ImportedDeclaration {
                    name: Name::from_dotted("Nat.zero"),
                    decl_interface_hash: "sha256:Mixed.Nat.zero".to_owned(),
                    binder_infos: Vec::new(),
                },
                ImportedDeclaration {
                    name: Name::from_dotted("Int.zero"),
                    decl_interface_hash: "sha256:Mixed.Int.zero".to_owned(),
                    binder_infos: Vec::new(),
                },
            ],
            kernel_declarations: Vec::new(),
        }
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
    fn surface_name_roundtrips_to_name() {
        let span = Span::new(FileId(0), 0, 3);
        let surface = SurfaceName {
            parts: vec!["A".to_owned(), "B".to_owned()],
            span,
        };
        assert_eq!(Name::from_surface(&surface).to_dotted(), "A.B");
    }
}
