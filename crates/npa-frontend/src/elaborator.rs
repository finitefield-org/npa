use std::collections::{BTreeMap, BTreeSet};

use crate::{
    parse_machine_module, resolve_machine_module, MachineBinder, MachineCompileOptions,
    MachineDecl, MachineDiagnostic, MachineDiagnosticKind, MachineItem, MachineLevel,
    MachineLocalDecl, MachineTerm, ResolvedMachineModule, Result, VerifiedImport,
};
use npa_kernel::{Ctx, Decl, Env, Expr, Level, Reducibility};

pub fn elaborate_machine_module(
    module_name: npa_cert::ModuleName,
    module: ResolvedMachineModule,
    verified_imports: &[VerifiedImport],
    _options: &MachineCompileOptions,
) -> Result<npa_cert::CoreModule> {
    let active_imports = active_verified_imports(&module.module.items, verified_imports)?;
    let kernel_env = kernel_env_from_imports(
        active_imports.iter().copied(),
        verified_imports,
        module.module.span,
    )?;
    let mut elaborator = Elaborator::new(active_imports.iter().copied(), kernel_env);
    let mut declarations = Vec::new();

    for item in module.module.items {
        match item {
            MachineItem::Import { .. } => {}
            MachineItem::Def(decl) => {
                let span = decl.span;
                let decl = elaborator.elaborate_decl(decl, DeclKind::Def)?;
                elaborator.add_decl_to_kernel_env(&decl, span)?;
                elaborator.add_decl_signature(&decl);
                declarations.push(decl);
            }
            MachineItem::Theorem(decl) => {
                let span = decl.span;
                let decl = elaborator.elaborate_decl(decl, DeclKind::Theorem)?;
                elaborator.add_decl_to_kernel_env(&decl, span)?;
                elaborator.add_decl_signature(&decl);
                declarations.push(decl);
            }
        }
    }

    Ok(npa_cert::CoreModule {
        name: module_name,
        declarations,
    })
}

pub fn compile_machine_source_to_core(
    file_id: crate::FileId,
    module_name: npa_cert::ModuleName,
    source: &str,
    verified_imports: &[VerifiedImport],
    options: &MachineCompileOptions,
) -> Result<npa_cert::CoreModule> {
    let module = parse_machine_module(file_id, source)?;
    let resolved = resolve_machine_module(module, verified_imports)?;
    elaborate_machine_module(module_name, resolved, verified_imports, options)
}

pub fn compile_machine_source_to_certificate(
    file_id: crate::FileId,
    module_name: npa_cert::ModuleName,
    source: &str,
    verified_modules: &[npa_cert::VerifiedModule],
    options: &MachineCompileOptions,
) -> Result<npa_cert::ModuleCert> {
    let verified_imports: Vec<_> = verified_modules.iter().map(VerifiedImport::from).collect();
    let parsed = parse_machine_module(file_id, source)?;
    let resolved = resolve_machine_module(parsed, &verified_imports)?;
    let active_import_indices =
        active_verified_import_indices(&resolved.module.items, &verified_imports)?;
    let module = elaborate_machine_module(module_name, resolved, &verified_imports, options)?;
    let certificate_imports =
        certificate_imports_for_module(&module, &active_import_indices, verified_modules, file_id)?;
    npa_cert::build_module_cert(module, &certificate_imports).map_err(|err| {
        MachineDiagnostic::error(
            MachineDiagnosticKind::CertificateRejected,
            crate::Span::empty(file_id),
            format!("certificate construction failed: {err:?}"),
        )
    })
}

pub fn elaborate_machine_term_check(
    source: &str,
    _local_context: &[MachineLocalDecl],
    _expected: &npa_kernel::Expr,
    _verified_imports: &[VerifiedImport],
    _options: &MachineCompileOptions,
) -> Result<npa_kernel::Expr> {
    Err(MachineDiagnostic::unsupported_syntax(
        crate::Span::new(crate::FileId(0), 0, source.len() as u32),
        "term-level Machine Surface elaboration is implemented in M7",
    ))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DeclKind {
    Def,
    Theorem,
}

const MAX_NUMERIC_UNIVERSE_LEVEL: u64 = 1024;

#[derive(Clone, Debug)]
struct ElaboratedBinder {
    name: String,
    ty: Expr,
}

#[derive(Clone, Debug)]
struct GlobalSignature {
    universe_params: Vec<String>,
}

#[derive(Clone, Debug)]
struct LocalDecl {
    name: String,
    ty: Expr,
    value: Option<Expr>,
}

#[derive(Clone, Debug, Default)]
struct LocalContext {
    locals: Vec<LocalDecl>,
}

impl LocalContext {
    fn push_assumption(&mut self, name: String, ty: Expr) {
        self.locals.push(LocalDecl {
            name,
            ty,
            value: None,
        });
    }

    fn push_definition(&mut self, name: String, ty: Expr, value: Expr) {
        self.locals.push(LocalDecl {
            name,
            ty,
            value: Some(value),
        });
    }

    fn lookup_bvar(&self, name: &str) -> Option<u32> {
        self.locals
            .iter()
            .rev()
            .position(|local| local.name == name)
            .map(|index| index as u32)
    }

    fn to_kernel_ctx(&self) -> Ctx {
        let mut ctx = Ctx::new();
        for local in &self.locals {
            match &local.value {
                Some(value) => {
                    ctx.push_definition(local.name.clone(), local.ty.clone(), value.clone())
                }
                None => ctx.push_assumption(local.name.clone(), local.ty.clone()),
            }
        }
        ctx
    }
}

struct Elaborator {
    globals: BTreeMap<String, GlobalSignature>,
    kernel_env: Env,
}

impl Elaborator {
    fn new<'a>(
        verified_imports: impl IntoIterator<Item = &'a VerifiedImport>,
        kernel_env: Env,
    ) -> Self {
        let mut elaborator = Self {
            globals: BTreeMap::new(),
            kernel_env,
        };

        for import in verified_imports {
            for export in &import.exports {
                elaborator
                    .add_global_signature(export.name.as_dotted(), export.universe_params.clone());
            }
        }

        elaborator
    }

    fn add_decl_signature(&mut self, decl: &Decl) {
        self.add_global_signature(decl.name().to_owned(), decl.universe_params().to_vec());
    }

    fn add_global_signature(&mut self, name: String, universe_params: Vec<String>) {
        self.globals
            .insert(name, GlobalSignature { universe_params });
    }

    fn elaborate_decl(&self, decl: MachineDecl, kind: DeclKind) -> Result<Decl> {
        let name = decl.name.as_dotted();
        let universe_params: Vec<_> = decl
            .universe_params
            .into_iter()
            .map(|param| param.name)
            .collect();
        let mut locals = LocalContext::default();
        let mut binders = Vec::with_capacity(decl.binders.len());

        for binder in decl.binders {
            let binder = self.elaborate_binder(binder, &mut locals, &universe_params)?;
            binders.push(binder);
        }

        let ty = close_pi(
            &binders,
            self.elaborate_term(decl.ty, &mut locals, &universe_params)?,
        );
        let value = close_lam(
            &binders,
            self.elaborate_term(decl.value, &mut locals, &universe_params)?,
        );

        match kind {
            DeclKind::Def => Ok(Decl::Def {
                name,
                universe_params,
                ty,
                value,
                reducibility: Reducibility::Reducible,
            }),
            DeclKind::Theorem => Ok(Decl::Theorem {
                name,
                universe_params,
                ty,
                proof: value,
            }),
        }
    }

    fn elaborate_binder(
        &self,
        binder: MachineBinder,
        locals: &mut LocalContext,
        delta: &[String],
    ) -> Result<ElaboratedBinder> {
        let ty = self.elaborate_term(binder.ty, locals, delta)?;
        locals.push_assumption(binder.name.clone(), ty.clone());

        Ok(ElaboratedBinder {
            name: binder.name,
            ty,
        })
    }

    fn elaborate_term(
        &self,
        term: MachineTerm,
        locals: &mut LocalContext,
        delta: &[String],
    ) -> Result<Expr> {
        match term {
            MachineTerm::Ident {
                name,
                universe_args,
                explicit_mode,
                span,
            } => {
                let name = name.as_dotted();
                let universe_param_count = self.universe_param_count(&name);
                let levels = self.elaborate_universe_args(
                    universe_args,
                    universe_param_count,
                    explicit_mode,
                    span,
                    &name,
                )?;

                Ok(Expr::konst(name, levels))
            }
            MachineTerm::Local { name, span } => {
                locals.lookup_bvar(&name).map(Expr::bvar).ok_or_else(|| {
                    MachineDiagnostic::error(
                        MachineDiagnosticKind::UnknownLocalName,
                        span,
                        format!("unknown local name {name}"),
                    )
                })
            }
            MachineTerm::Sort { level, .. } => Ok(Expr::sort(elaborate_level(level)?)),
            MachineTerm::App { func, arg, .. } => Ok(Expr::app(
                self.elaborate_term(*func, locals, delta)?,
                self.elaborate_term(*arg, locals, delta)?,
            )),
            MachineTerm::Lam { binders, body, .. } => {
                let mut nested_locals = locals.clone();
                let mut elaborated_binders = Vec::with_capacity(binders.len());
                for binder in binders {
                    elaborated_binders.push(self.elaborate_binder(
                        binder,
                        &mut nested_locals,
                        delta,
                    )?);
                }
                let body = self.elaborate_term(*body, &mut nested_locals, delta)?;
                Ok(close_lam(&elaborated_binders, body))
            }
            MachineTerm::Pi { binders, body, .. } => {
                let mut nested_locals = locals.clone();
                let mut elaborated_binders = Vec::with_capacity(binders.len());
                for binder in binders {
                    elaborated_binders.push(self.elaborate_binder(
                        binder,
                        &mut nested_locals,
                        delta,
                    )?);
                }
                let body = self.elaborate_term(*body, &mut nested_locals, delta)?;
                Ok(close_pi(&elaborated_binders, body))
            }
            MachineTerm::Let {
                name,
                ty,
                value,
                body,
                ..
            } => {
                let ty = self.elaborate_term(*ty, locals, delta)?;
                let value = self.elaborate_term(*value, locals, delta)?;
                let mut nested_locals = locals.clone();
                nested_locals.push_definition(name.clone(), ty.clone(), value.clone());
                let body = self.elaborate_term(*body, &mut nested_locals, delta)?;
                Ok(Expr::let_in(name, ty, value, body))
            }
            MachineTerm::Annot { expr, ty, span } => {
                let expr = self.elaborate_term(*expr, locals, delta)?;
                let ty = self.elaborate_term(*ty, locals, delta)?;
                self.ensure_type(&ty, locals, delta, span)?;
                self.check_expr(&expr, &ty, locals, delta, span)?;
                Ok(expr)
            }
        }
    }

    fn universe_param_count(&self, name: &str) -> usize {
        self.globals
            .get(name)
            .map(|signature| signature.universe_params.len())
            .unwrap_or(0)
    }

    fn ensure_type(
        &self,
        expr: &Expr,
        locals: &LocalContext,
        delta: &[String],
        span: crate::Span,
    ) -> Result<Level> {
        let ctx = locals.to_kernel_ctx();
        let inferred = self
            .kernel_env
            .infer(&ctx, delta, expr)
            .map_err(|err| kernel_expr_diagnostic(span, err))?;
        match self
            .kernel_env
            .whnf(&ctx, delta, &inferred)
            .map_err(|err| kernel_expr_diagnostic(span, err))?
        {
            Expr::Sort(level) => Ok(level),
            actual => Err(MachineDiagnostic::error(
                MachineDiagnosticKind::ExpectedSort,
                span,
                format!("expected a type annotation, got {actual:?}"),
            )),
        }
    }

    fn check_expr(
        &self,
        expr: &Expr,
        expected: &Expr,
        locals: &LocalContext,
        delta: &[String],
        span: crate::Span,
    ) -> Result<()> {
        let ctx = locals.to_kernel_ctx();
        self.kernel_env
            .check(&ctx, delta, expr, expected)
            .map_err(|err| kernel_expr_diagnostic(span, err))
    }

    fn add_decl_to_kernel_env(&mut self, decl: &Decl, span: crate::Span) -> Result<()> {
        add_kernel_decl_to_env(&mut self.kernel_env, decl.clone()).map_err(|err| {
            MachineDiagnostic::error(
                MachineDiagnosticKind::KernelRejected,
                span,
                format!("kernel rejected elaborated declaration: {err:?}"),
            )
        })
    }

    fn elaborate_universe_args(
        &self,
        args: Option<Vec<MachineLevel>>,
        expected: usize,
        explicit_mode: bool,
        span: crate::Span,
        name: &str,
    ) -> Result<Vec<Level>> {
        match args {
            Some(args) => {
                if args.len() != expected {
                    return Err(MachineDiagnostic::error(
                        MachineDiagnosticKind::MissingExplicitUniverse,
                        span,
                        format!(
                            "global name {name} expects {expected} explicit universe arguments"
                        ),
                    ));
                }

                args.into_iter().map(elaborate_level).collect()
            }
            None if expected == 0 => Ok(Vec::new()),
            None if explicit_mode => Err(MachineDiagnostic::error(
                MachineDiagnosticKind::MissingExplicitUniverse,
                span,
                format!("global name {name} requires explicit universe arguments"),
            )),
            None => Err(MachineDiagnostic::error(
                MachineDiagnosticKind::ImplicitArgumentRequired,
                span,
                format!("global name {name} requires explicit arguments"),
            )),
        }
    }
}

fn active_verified_imports<'a>(
    items: &[MachineItem],
    verified_imports: &'a [VerifiedImport],
) -> Result<Vec<&'a VerifiedImport>> {
    active_verified_import_indices(items, verified_imports).map(|indices| {
        indices
            .into_iter()
            .map(|index| &verified_imports[index])
            .collect()
    })
}

fn active_verified_import_indices(
    items: &[MachineItem],
    verified_imports: &[VerifiedImport],
) -> Result<Vec<usize>> {
    let mut seen = BTreeSet::new();
    let mut imports = Vec::new();

    for item in items {
        let MachineItem::Import { module, span } = item else {
            continue;
        };
        let module_name = npa_cert::Name(module.parts.clone());
        if seen.insert(module_name.clone()) {
            imports.push(find_verified_import_index(
                verified_imports,
                &module_name,
                *span,
            )?);
        }
    }

    Ok(imports)
}

fn find_verified_import_index(
    verified_imports: &[VerifiedImport],
    module_name: &npa_cert::ModuleName,
    span: crate::Span,
) -> Result<usize> {
    let mut matches = verified_imports
        .iter()
        .enumerate()
        .filter(|(_, import)| &import.module == module_name);
    let Some((first_index, first)) = matches.next() else {
        return Err(MachineDiagnostic::error(
            MachineDiagnosticKind::MissingVerifiedImport,
            span,
            format!(
                "import {} is not present in the verified import set",
                module_name.as_dotted()
            ),
        ));
    };

    if matches.any(|(_, import)| import != first) {
        return Err(MachineDiagnostic::error(
            MachineDiagnosticKind::ImportResolutionError,
            span,
            format!(
                "import {} has multiple verified interfaces",
                module_name.as_dotted()
            ),
        ));
    }

    Ok(first_index)
}

fn certificate_imports_for_module(
    module: &npa_cert::CoreModule,
    active_import_indices: &[usize],
    verified_modules: &[npa_cert::VerifiedModule],
    file_id: crate::FileId,
) -> Result<Vec<npa_cert::VerifiedModule>> {
    let span = crate::Span::empty(file_id);
    let referenced_exports = referenced_import_names(module);
    let mut selected = BTreeSet::new();
    let mut pending: Vec<_> = active_import_indices.to_vec();

    while let Some(index) = pending.pop() {
        if !selected.insert(index) {
            continue;
        }

        let import = verified_modules.get(index).ok_or_else(|| {
            import_resolution_diagnostic(span, "verified import index is out of bounds")
        })?;
        for dependency in import_interface_dependency_targets(import, span)? {
            let dependency_index = find_verified_module_export(
                verified_modules,
                &dependency.name,
                dependency.hash,
                span,
            )?;
            if !selected.contains(&dependency_index) {
                pending.push(dependency_index);
            }
        }

        for dependency in referenced_axiom_dependency_targets(import, &referenced_exports, span)? {
            let dependency_index = find_verified_module_axiom_export(
                verified_modules,
                &dependency.name,
                dependency.hash,
                span,
            )?;
            if !selected.contains(&dependency_index) {
                pending.push(dependency_index);
            }
        }
    }

    Ok(selected
        .into_iter()
        .map(|index| verified_modules[index].clone())
        .collect())
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ImportDependencyTarget {
    name: npa_cert::Name,
    hash: npa_cert::Hash,
}

fn import_interface_dependency_targets(
    module: &npa_cert::VerifiedModule,
    span: crate::Span,
) -> Result<BTreeSet<ImportDependencyTarget>> {
    let mut dependencies = BTreeSet::new();
    for entry in &module.export_block {
        collect_imported_dependency_targets_from_term(module, entry.ty, &mut dependencies, span)?;
        if let Some(body) = entry.body {
            collect_imported_dependency_targets_from_term(module, body, &mut dependencies, span)?;
        }
    }
    Ok(dependencies)
}

fn collect_imported_dependency_targets_from_term(
    module: &npa_cert::VerifiedModule,
    term: npa_cert::TermId,
    dependencies: &mut BTreeSet<ImportDependencyTarget>,
    span: crate::Span,
) -> Result<()> {
    match module
        .term_table
        .get(term)
        .ok_or_else(|| import_resolution_diagnostic(span, "verified import term is missing"))?
    {
        npa_cert::TermNode::Sort(_) | npa_cert::TermNode::BVar(_) => {}
        npa_cert::TermNode::Const { global_ref, .. } => {
            if let npa_cert::GlobalRef::Imported {
                name,
                decl_interface_hash,
                ..
            } = global_ref
            {
                dependencies.insert(ImportDependencyTarget {
                    name: module
                        .name_table
                        .get(*name)
                        .ok_or_else(|| {
                            import_resolution_diagnostic(span, "verified import name is missing")
                        })?
                        .clone(),
                    hash: *decl_interface_hash,
                });
            }
        }
        npa_cert::TermNode::App(func, arg) => {
            collect_imported_dependency_targets_from_term(module, *func, dependencies, span)?;
            collect_imported_dependency_targets_from_term(module, *arg, dependencies, span)?;
        }
        npa_cert::TermNode::Lam { ty, body } | npa_cert::TermNode::Pi { ty, body } => {
            collect_imported_dependency_targets_from_term(module, *ty, dependencies, span)?;
            collect_imported_dependency_targets_from_term(module, *body, dependencies, span)?;
        }
        npa_cert::TermNode::Let { ty, value, body } => {
            collect_imported_dependency_targets_from_term(module, *ty, dependencies, span)?;
            collect_imported_dependency_targets_from_term(module, *value, dependencies, span)?;
            collect_imported_dependency_targets_from_term(module, *body, dependencies, span)?;
        }
    }
    Ok(())
}

fn referenced_axiom_dependency_targets(
    module: &npa_cert::VerifiedModule,
    referenced_exports: &BTreeSet<npa_cert::Name>,
    span: crate::Span,
) -> Result<BTreeSet<ImportDependencyTarget>> {
    let mut dependencies = BTreeSet::new();
    for entry in &module.export_block {
        let Some(entry_name) = module.name_table.get(entry.name) else {
            return Err(import_resolution_diagnostic(
                span,
                "verified import export name is missing",
            ));
        };
        if !referenced_exports.contains(entry_name) {
            continue;
        }

        for axiom in &entry.axiom_dependencies {
            dependencies.insert(ImportDependencyTarget {
                name: module
                    .name_table
                    .get(axiom.name)
                    .ok_or_else(|| {
                        import_resolution_diagnostic(span, "verified import axiom name is missing")
                    })?
                    .clone(),
                hash: axiom.decl_interface_hash,
            });
        }
    }
    Ok(dependencies)
}

fn find_verified_module_export(
    verified_modules: &[npa_cert::VerifiedModule],
    name: &npa_cert::Name,
    decl_interface_hash: npa_cert::Hash,
    span: crate::Span,
) -> Result<usize> {
    find_verified_module_export_by(verified_modules, name, decl_interface_hash, None, span)
}

fn find_verified_module_axiom_export(
    verified_modules: &[npa_cert::VerifiedModule],
    name: &npa_cert::Name,
    decl_interface_hash: npa_cert::Hash,
    span: crate::Span,
) -> Result<usize> {
    find_verified_module_export_by(
        verified_modules,
        name,
        decl_interface_hash,
        Some(npa_cert::ExportKind::Axiom),
        span,
    )
}

fn find_verified_module_export_by(
    verified_modules: &[npa_cert::VerifiedModule],
    name: &npa_cert::Name,
    decl_interface_hash: npa_cert::Hash,
    kind: Option<npa_cert::ExportKind>,
    span: crate::Span,
) -> Result<usize> {
    verified_modules
        .iter()
        .enumerate()
        .find_map(|(index, module)| {
            module
                .export_block
                .iter()
                .any(|entry| {
                    kind.is_none_or(|kind| entry.kind == kind)
                        && entry.decl_interface_hash == decl_interface_hash
                        && module
                            .name_table
                            .get(entry.name)
                            .is_some_and(|entry_name| entry_name == name)
                })
                .then_some(index)
        })
        .ok_or_else(|| {
            import_resolution_diagnostic(
                span,
                format!(
                    "verified dependency {} is not present in the verified import set",
                    name.as_dotted()
                ),
            )
        })
}

fn referenced_import_names(module: &npa_cert::CoreModule) -> BTreeSet<npa_cert::Name> {
    let mut names = BTreeSet::new();
    for decl in &module.declarations {
        collect_const_names_from_decl(&mut names, decl);
    }

    for name in local_public_names(module) {
        names.remove(&name);
    }

    names
}

fn local_public_names(module: &npa_cert::CoreModule) -> Vec<npa_cert::Name> {
    let mut names = Vec::new();
    for decl in &module.declarations {
        names.push(npa_cert::Name::from_dotted(decl.name()));
        if let Decl::Inductive { data, .. } = decl {
            names.extend(
                data.constructors
                    .iter()
                    .map(|constructor| npa_cert::Name::from_dotted(&constructor.name)),
            );
            if let Some(recursor) = &data.recursor {
                names.push(npa_cert::Name::from_dotted(&recursor.name));
            }
        }
    }
    names
}

fn collect_const_names_from_decl(names: &mut BTreeSet<npa_cert::Name>, decl: &Decl) {
    match decl {
        Decl::Axiom { ty, .. } => collect_const_names_from_expr(names, ty),
        Decl::Def { ty, value, .. } => {
            collect_const_names_from_expr(names, ty);
            collect_const_names_from_expr(names, value);
        }
        Decl::Theorem { ty, proof, .. } => {
            collect_const_names_from_expr(names, ty);
            collect_const_names_from_expr(names, proof);
        }
        Decl::Inductive { data, .. } => {
            for param in &data.params {
                collect_const_names_from_expr(names, &param.ty);
            }
            for index in &data.indices {
                collect_const_names_from_expr(names, &index.ty);
            }
            for constructor in &data.constructors {
                collect_const_names_from_expr(names, &constructor.ty);
            }
            if let Some(recursor) = &data.recursor {
                collect_const_names_from_expr(names, &recursor.ty);
            }
        }
        Decl::Constructor { ty, .. } | Decl::Recursor { ty, .. } => {
            collect_const_names_from_expr(names, ty);
        }
    }
}

fn collect_const_names_from_expr(names: &mut BTreeSet<npa_cert::Name>, expr: &Expr) {
    match expr {
        Expr::Sort(_) | Expr::BVar(_) => {}
        Expr::Const { name, .. } => {
            names.insert(npa_cert::Name::from_dotted(name));
        }
        Expr::App(func, arg) => {
            collect_const_names_from_expr(names, func);
            collect_const_names_from_expr(names, arg);
        }
        Expr::Lam { ty, body, .. } | Expr::Pi { ty, body, .. } => {
            collect_const_names_from_expr(names, ty);
            collect_const_names_from_expr(names, body);
        }
        Expr::Let {
            ty, value, body, ..
        } => {
            collect_const_names_from_expr(names, ty);
            collect_const_names_from_expr(names, value);
            collect_const_names_from_expr(names, body);
        }
    }
}

fn import_resolution_diagnostic(
    span: crate::Span,
    message: impl Into<String>,
) -> MachineDiagnostic {
    MachineDiagnostic::error(MachineDiagnosticKind::ImportResolutionError, span, message)
}

fn kernel_env_from_imports<'a>(
    active_imports: impl IntoIterator<Item = &'a VerifiedImport>,
    available_imports: &'a [VerifiedImport],
    span: crate::Span,
) -> Result<Env> {
    let available_decls = collect_import_decls(available_imports);
    let mut pending: Vec<_> = collect_import_decls(active_imports)
        .into_values()
        .flatten()
        .collect();
    let mut queued: BTreeSet<_> = pending
        .iter()
        .map(|decl: &Decl| decl.name().to_owned())
        .collect();
    let mut env = Env::new();

    while !pending.is_empty() {
        let mut progressed = false;
        let mut remaining = Vec::new();

        for decl in pending {
            if env.decl(decl.name()).is_some() {
                progressed = true;
                continue;
            }

            match add_kernel_decl_to_env(&mut env, decl.clone()) {
                Ok(()) => {
                    progressed = true;
                }
                Err(npa_kernel::Error::UnknownConstant(name)) => {
                    if !queued.contains(&name) {
                        if let Some(Some(dependency)) = available_decls.get(&name) {
                            queued.insert(name.clone());
                            remaining.push(dependency.clone());
                            progressed = true;
                        }
                    }
                    remaining.push(decl);
                }
                Err(err) => {
                    return Err(MachineDiagnostic::error(
                        MachineDiagnosticKind::KernelRejected,
                        span,
                        format!("kernel rejected verified import interface: {err:?}"),
                    ));
                }
            }
        }

        if !progressed {
            break;
        }

        pending = remaining;
    }

    Ok(env)
}

fn collect_import_decls<'a>(
    imports: impl IntoIterator<Item = &'a VerifiedImport>,
) -> BTreeMap<String, Option<Decl>> {
    let mut decls: BTreeMap<String, Option<Decl>> = BTreeMap::new();

    for import in imports {
        for decl in kernel_decls_for_import(import) {
            for name in kernel_decl_lookup_names(&decl) {
                match decls.get_mut(&name) {
                    Some(existing) if existing.as_ref() == Some(&decl) => {}
                    Some(existing) => {
                        *existing = None;
                    }
                    None => {
                        decls.insert(name, Some(decl.clone()));
                    }
                }
            }
        }
    }

    decls
}

fn kernel_decl_lookup_names(decl: &Decl) -> Vec<String> {
    let mut names = vec![decl.name().to_owned()];

    if let Decl::Inductive { data, .. } = decl {
        names.extend(
            data.constructors
                .iter()
                .map(|constructor| constructor.name.clone()),
        );
        if let Some(recursor) = &data.recursor {
            names.push(recursor.name.clone());
        }
    }

    names
}

fn kernel_decls_for_import(import: &VerifiedImport) -> Vec<Decl> {
    if import.kernel_decls.is_empty() {
        return fallback_kernel_decls_for_import(import);
    }

    import.kernel_decls.clone()
}

fn fallback_kernel_decls_for_import(import: &VerifiedImport) -> Vec<Decl> {
    import
        .exports
        .iter()
        .map(|export| Decl::Axiom {
            name: export.name.as_dotted(),
            universe_params: export.universe_params.clone(),
            ty: export.ty.clone(),
        })
        .collect()
}

fn add_kernel_decl_to_env(env: &mut Env, decl: Decl) -> npa_kernel::Result<()> {
    match decl {
        Decl::Axiom {
            name,
            universe_params,
            ty,
        } => env.add_axiom(name, universe_params, ty),
        Decl::Def {
            name,
            universe_params,
            ty,
            value,
            reducibility,
        } => env.add_def(name, universe_params, ty, value, reducibility),
        Decl::Theorem {
            name,
            universe_params,
            ty,
            proof,
        } => env.add_theorem(name, universe_params, ty, proof),
        Decl::Inductive { data, .. } => env.add_inductive(*data),
        Decl::Constructor { .. } | Decl::Recursor { .. } => {
            Err(npa_kernel::Error::InvalidInductive(
                "generated declarations cannot be added directly".to_owned(),
            ))
        }
    }
}

fn kernel_expr_diagnostic(span: crate::Span, err: npa_kernel::Error) -> MachineDiagnostic {
    match err {
        npa_kernel::Error::ExpectedPi { actual } => MachineDiagnostic::error(
            MachineDiagnosticKind::ExpectedFunctionType,
            span,
            format!("application head is not a function: {actual:?}"),
        ),
        npa_kernel::Error::ExpectedSort { actual } => MachineDiagnostic::error(
            MachineDiagnosticKind::ExpectedSort,
            span,
            format!("expected a type annotation, got {actual:?}"),
        ),
        npa_kernel::Error::TypeMismatch { expected, actual } => MachineDiagnostic::error(
            MachineDiagnosticKind::TypeMismatch,
            span,
            format!("type annotation mismatch: expected {expected:?}, got {actual:?}"),
        ),
        err => MachineDiagnostic::error(
            MachineDiagnosticKind::KernelRejected,
            span,
            format!("kernel rejected elaborated expression: {err:?}"),
        ),
    }
}

fn close_lam(binders: &[ElaboratedBinder], mut body: Expr) -> Expr {
    for binder in binders.iter().rev() {
        body = Expr::lam(binder.name.clone(), binder.ty.clone(), body);
    }
    body
}

fn close_pi(binders: &[ElaboratedBinder], mut body: Expr) -> Expr {
    for binder in binders.iter().rev() {
        body = Expr::pi(binder.name.clone(), binder.ty.clone(), body);
    }
    body
}

fn elaborate_level(level: MachineLevel) -> Result<Level> {
    match level {
        MachineLevel::Nat { value, span } => level_from_nat(value, span),
        MachineLevel::Param { name, .. } => Ok(Level::param(name)),
        MachineLevel::Succ { level, .. } => Ok(Level::succ(elaborate_level(*level)?)),
        MachineLevel::Max { lhs, rhs, .. } => {
            Ok(Level::max(elaborate_level(*lhs)?, elaborate_level(*rhs)?))
        }
        MachineLevel::IMax { lhs, rhs, .. } => {
            Ok(Level::imax(elaborate_level(*lhs)?, elaborate_level(*rhs)?))
        }
    }
}

fn level_from_nat(value: u64, span: crate::Span) -> Result<Level> {
    if value > MAX_NUMERIC_UNIVERSE_LEVEL {
        return Err(MachineDiagnostic::error(
            MachineDiagnosticKind::UniverseLevelTooLarge,
            span,
            format!(
                "numeric universe level {value} exceeds the maximum supported level {MAX_NUMERIC_UNIVERSE_LEVEL}"
            ),
        ));
    }

    Ok((0..value).fold(Level::zero(), |level, _| Level::succ(level)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FileId;

    fn hash(seed: u8) -> npa_cert::Hash {
        [seed; 32]
    }

    fn type0() -> Level {
        Level::succ(Level::zero())
    }

    fn nat() -> Expr {
        Expr::konst("Nat", vec![])
    }

    fn verified_import(module: &str, exports: &[(&str, &[&str])]) -> VerifiedImport {
        let exports: Vec<_> = exports
            .iter()
            .enumerate()
            .map(|(index, (name, universe_params))| crate::VerifiedExport {
                name: npa_cert::Name::from_dotted(name),
                universe_params: universe_params
                    .iter()
                    .map(|param| param.to_string())
                    .collect(),
                ty: export_ty(name),
                decl_interface_hash: hash(index as u8 + 2),
            })
            .collect();
        let kernel_decls = exports
            .iter()
            .map(|export| Decl::Axiom {
                name: export.name.as_dotted(),
                universe_params: export.universe_params.clone(),
                ty: export.ty.clone(),
            })
            .collect();

        VerifiedImport {
            module: npa_cert::Name::from_dotted(module),
            export_hash: hash(1),
            certificate_hash: None,
            exports,
            kernel_decls,
        }
    }

    fn nat_import() -> VerifiedImport {
        verified_import("Std.Nat.Basic", &[("Nat", &[])])
    }

    fn eq_import() -> VerifiedImport {
        verified_import("Std.Logic.Eq", &[("Eq", &["u"]), ("Eq.refl", &["u"])])
    }

    fn export_ty(name: &str) -> Expr {
        match name {
            "Nat" => Expr::sort(type0()),
            "Eq" => npa_kernel::eq_type(Level::param("u")),
            "Eq.refl" => npa_kernel::eq_refl_type(Level::param("u")),
            _ => Expr::sort(Level::zero()),
        }
    }

    fn verified_core_module(module: npa_cert::CoreModule) -> npa_cert::VerifiedModule {
        let cert = npa_cert::build_module_cert(module, &[]).expect("core module should certify");
        let bytes = npa_cert::encode_module_cert(&cert).expect("certificate should encode");
        let mut session = npa_cert::VerifierSession::new();
        npa_cert::verify_module_cert(&bytes, &mut session, &npa_cert::AxiomPolicy::normal())
            .expect("certificate should verify")
    }

    fn verified_core_module_in_session(
        module: npa_cert::CoreModule,
        imports: &[npa_cert::VerifiedModule],
        session: &mut npa_cert::VerifierSession,
    ) -> npa_cert::VerifiedModule {
        let cert =
            npa_cert::build_module_cert(module, imports).expect("core module should certify");
        let bytes = npa_cert::encode_module_cert(&cert).expect("certificate should encode");
        npa_cert::verify_module_cert(&bytes, session, &npa_cert::AxiomPolicy::normal())
            .expect("certificate should verify")
    }

    fn alias_import() -> VerifiedImport {
        let u = Level::param("u");
        let module = verified_core_module(npa_cert::CoreModule {
            name: npa_cert::Name::from_dotted("Std.Alias"),
            declarations: vec![
                Decl::Def {
                    name: "Alias.IdTy".to_owned(),
                    universe_params: vec!["u".to_owned()],
                    ty: Expr::pi(
                        "A",
                        Expr::sort(u.clone()),
                        Expr::sort(Level::imax(u.clone(), u.clone())),
                    ),
                    value: Expr::lam(
                        "A",
                        Expr::sort(u.clone()),
                        Expr::pi("x", Expr::bvar(0), Expr::bvar(1)),
                    ),
                    reducibility: Reducibility::Reducible,
                },
                Decl::Def {
                    name: "Alias.id".to_owned(),
                    universe_params: vec!["u".to_owned()],
                    ty: Expr::pi(
                        "A",
                        Expr::sort(u.clone()),
                        Expr::app(Expr::konst("Alias.IdTy", vec![u.clone()]), Expr::bvar(0)),
                    ),
                    value: Expr::lam(
                        "A",
                        Expr::sort(u),
                        Expr::lam("x", Expr::bvar(0), Expr::bvar(0)),
                    ),
                    reducibility: Reducibility::Reducible,
                },
            ],
        });

        VerifiedImport::from(&module)
    }

    fn unary_expr() -> Expr {
        Expr::konst("Unary", vec![])
    }

    fn unary_zero_expr() -> Expr {
        Expr::konst("Unary.zero", vec![])
    }

    fn unary_succ_expr(arg: Expr) -> Expr {
        Expr::app(Expr::konst("Unary.succ", vec![]), arg)
    }

    fn unary_rec_type_expr(level: Level) -> Expr {
        let motive_ty = Expr::pi("_", unary_expr(), Expr::sort(level));
        let z_ty = Expr::app(Expr::bvar(0), unary_zero_expr());
        let s_ty = Expr::pi(
            "n",
            unary_expr(),
            Expr::pi(
                "ih",
                Expr::app(Expr::bvar(2), Expr::bvar(0)),
                Expr::app(Expr::bvar(3), unary_succ_expr(Expr::bvar(1))),
            ),
        );

        Expr::pi(
            "motive",
            motive_ty,
            Expr::pi(
                "z",
                z_ty,
                Expr::pi(
                    "s",
                    s_ty,
                    Expr::pi("n", unary_expr(), Expr::app(Expr::bvar(3), Expr::bvar(0))),
                ),
            ),
        )
    }

    fn unary_rec_core_module() -> npa_cert::CoreModule {
        let data = npa_kernel::InductiveDecl::new(
            "Unary",
            vec![],
            vec![],
            vec![],
            type0(),
            vec![
                npa_kernel::ConstructorDecl::new("Unary.zero", unary_expr()),
                npa_kernel::ConstructorDecl::new(
                    "Unary.succ",
                    Expr::pi("_", unary_expr(), unary_expr()),
                ),
            ],
            Some(npa_kernel::RecursorDecl::new(
                "Unary.rec",
                vec!["u".to_owned()],
                unary_rec_type_expr(Level::param("u")),
            )),
        );
        npa_cert::CoreModule {
            name: npa_cert::Name::from_dotted("Test.Unary"),
            declarations: vec![Decl::Inductive {
                name: "Unary".to_owned(),
                universe_params: vec![],
                ty: Expr::sort(type0()),
                data: Box::new(data),
            }],
        }
    }

    fn unary_rec_import() -> VerifiedImport {
        VerifiedImport {
            module: npa_cert::Name::from_dotted("Test.Unary"),
            export_hash: hash(30),
            certificate_hash: None,
            exports: Vec::new(),
            kernel_decls: unary_rec_core_module().declarations,
        }
    }

    fn recursor_generated_type() -> Expr {
        let motive_level = Level::succ(type0());
        let motive = Expr::lam("_", unary_expr(), Expr::sort(type0()));
        let zero_case = Expr::sort(Level::zero());
        let succ_case = Expr::lam(
            "n",
            unary_expr(),
            Expr::lam("ih", Expr::sort(type0()), Expr::sort(Level::zero())),
        );

        Expr::apps(
            Expr::konst("Unary.rec", vec![motive_level]),
            vec![motive, zero_case, succ_case, unary_zero_expr()],
        )
    }

    fn generated_dependency_core_module() -> npa_cert::CoreModule {
        let p_ty = recursor_generated_type();

        npa_cert::CoreModule {
            name: npa_cert::Name::from_dotted("Test.UseRec"),
            declarations: vec![
                Decl::Axiom {
                    name: "UseRec.P".to_owned(),
                    universe_params: Vec::new(),
                    ty: p_ty,
                },
                Decl::Axiom {
                    name: "UseRec.w".to_owned(),
                    universe_params: Vec::new(),
                    ty: Expr::konst("UseRec.P", vec![]),
                },
            ],
        }
    }

    fn generated_dependency_import() -> VerifiedImport {
        let p_ty = recursor_generated_type();

        VerifiedImport {
            module: npa_cert::Name::from_dotted("Test.UseRec"),
            export_hash: hash(31),
            certificate_hash: None,
            exports: vec![
                crate::VerifiedExport {
                    name: npa_cert::Name::from_dotted("UseRec.P"),
                    universe_params: Vec::new(),
                    ty: p_ty.clone(),
                    decl_interface_hash: hash(32),
                },
                crate::VerifiedExport {
                    name: npa_cert::Name::from_dotted("UseRec.w"),
                    universe_params: Vec::new(),
                    ty: Expr::konst("UseRec.P", vec![]),
                    decl_interface_hash: hash(33),
                },
            ],
            kernel_decls: vec![
                Decl::Axiom {
                    name: "UseRec.P".to_owned(),
                    universe_params: Vec::new(),
                    ty: p_ty,
                },
                Decl::Axiom {
                    name: "UseRec.w".to_owned(),
                    universe_params: Vec::new(),
                    ty: Expr::konst("UseRec.P", vec![]),
                },
            ],
        }
    }

    #[test]
    fn compiles_empty_machine_module_to_empty_core_module() {
        let module = compile_machine_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test.Empty"),
            "",
            &[],
            &MachineCompileOptions::default(),
        )
        .expect("empty module should compile in M1");

        assert_eq!(module.name, npa_cert::Name::from_dotted("Test.Empty"));
        assert!(module.declarations.is_empty());
    }

    #[test]
    fn loads_transitive_import_needed_by_generated_inductive_name() {
        let imports = [generated_dependency_import(), unary_rec_import()];
        compile_machine_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Test.UseRec
def Test.copy : UseRec.P := UseRec.w",
            &imports,
            &MachineCompileOptions::default(),
        )
        .expect("generated inductive dependency should queue its wrapper declaration");
    }

    #[test]
    fn certificate_ignores_unimported_verified_modules() {
        let mut session = npa_cert::VerifierSession::new();
        let unused = verified_core_module_in_session(
            npa_cert::CoreModule {
                name: npa_cert::Name::from_dotted("Unused.Module"),
                declarations: vec![Decl::Axiom {
                    name: "Test.ok".to_owned(),
                    universe_params: vec![],
                    ty: Expr::sort(type0()),
                }],
            },
            &[],
            &mut session,
        );

        compile_machine_source_to_certificate(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "def Test.ok : Sort 2 := Type",
            &[unused],
            &MachineCompileOptions::default(),
        )
        .expect("unimported verified modules should not be passed to certificate construction");
    }

    #[test]
    fn certificate_includes_transitive_import_needed_by_generated_inductive_name() {
        let mut session = npa_cert::VerifierSession::new();
        let unary = verified_core_module_in_session(unary_rec_core_module(), &[], &mut session);
        let use_rec = verified_core_module_in_session(
            generated_dependency_core_module(),
            std::slice::from_ref(&unary),
            &mut session,
        );

        compile_machine_source_to_certificate(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Test.UseRec
def Test.copy : UseRec.P := UseRec.w",
            &[use_rec, unary],
            &MachineCompileOptions::default(),
        )
        .expect("certificate construction should receive transitive import dependencies");
    }

    #[test]
    fn ignores_unimported_verified_interfaces_when_checking_local_decls() {
        let imports = [verified_import("Unused.Module", &[("Test.ok", &[])])];
        compile_machine_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "def Test.ok : Sort 2 := Type",
            &imports,
            &MachineCompileOptions::default(),
        )
        .expect("unimported verified interfaces should not populate the kernel env");
    }

    #[test]
    fn elaborates_explicit_id_to_core_def() {
        let module = compile_machine_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "def Test.id.{u} (A : Sort u) (x : A) : A := x",
            &[],
            &MachineCompileOptions::default(),
        )
        .expect("explicit id should elaborate");

        let u = Level::param("u");
        assert_eq!(
            module.declarations,
            vec![Decl::Def {
                name: "Test.id".to_owned(),
                universe_params: vec!["u".to_owned()],
                ty: Expr::pi(
                    "A",
                    Expr::sort(u.clone()),
                    Expr::pi("x", Expr::bvar(0), Expr::bvar(1)),
                ),
                value: Expr::lam(
                    "A",
                    Expr::sort(u),
                    Expr::lam("x", Expr::bvar(0), Expr::bvar(0)),
                ),
                reducibility: Reducibility::Reducible,
            }]
        );
    }

    #[test]
    fn elaborates_explicit_eq_refl_to_core_theorem() {
        let imports = [nat_import(), eq_import()];
        let module = compile_machine_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Nat.Basic
import Std.Logic.Eq
theorem Test.self_eq (n : Nat) : Eq.{1} Nat n n := @Eq.refl.{1} Nat n",
            &imports,
            &MachineCompileOptions::default(),
        )
        .expect("explicit Eq.refl theorem should elaborate");

        let eq_nn = Expr::apps(
            Expr::konst("Eq", vec![type0()]),
            vec![nat(), Expr::bvar(0), Expr::bvar(0)],
        );
        let proof = Expr::apps(
            Expr::konst("Eq.refl", vec![type0()]),
            vec![nat(), Expr::bvar(0)],
        );

        assert_eq!(
            module.declarations,
            vec![Decl::Theorem {
                name: "Test.self_eq".to_owned(),
                universe_params: Vec::new(),
                ty: Expr::pi("n", nat(), eq_nn),
                proof: Expr::lam("n", nat(), proof),
            }]
        );
    }

    #[test]
    fn core_module_erases_machine_surface_import_items() {
        let imports = [nat_import()];
        let module = compile_machine_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Nat.Basic
def Test.id_nat (n : Nat) : Nat := n",
            &imports,
            &MachineCompileOptions::default(),
        )
        .expect("imported Nat declaration should elaborate and kernel-check");

        assert_eq!(module.name, npa_cert::Name::from_dotted("Test"));
        assert_eq!(
            module.declarations,
            vec![Decl::Def {
                name: "Test.id_nat".to_owned(),
                universe_params: Vec::new(),
                ty: Expr::pi("n", nat(), nat()),
                value: Expr::lam("n", nat(), Expr::bvar(0)),
                reducibility: Reducibility::Reducible,
            }]
        );
    }

    #[test]
    fn rejects_eq_refl_without_explicit_arguments() {
        let imports = [nat_import(), eq_import()];
        let err = compile_machine_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Nat.Basic
import Std.Logic.Eq
theorem Test.bad (n : Nat) : Eq.{1} Nat n n := Eq.refl n",
            &imports,
            &MachineCompileOptions::default(),
        )
        .expect_err("implicit Eq.refl should be rejected");

        assert_eq!(err.kind, MachineDiagnosticKind::ImplicitArgumentRequired);
    }

    #[test]
    fn rejects_ill_typed_theorem_during_kernel_handoff() {
        let err = compile_machine_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
theorem Test.bad (A : Type) (x : A) : A := fun (y : A) => y",
            &[],
            &MachineCompileOptions::default(),
        )
        .expect_err("kernel handoff should reject an ill-typed theorem proof");

        assert_eq!(err.kind, MachineDiagnosticKind::KernelRejected);
    }

    #[test]
    fn elaborates_lambda_pi_let_and_annotation() {
        compile_machine_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
def Test.term.{u} (A : Sort u) : (forall (x : A), A) :=
  fun (x : A) => let y : A := x in (y : A)",
            &[],
            &MachineCompileOptions::default(),
        )
        .expect("lambda, Pi, let, and annotation should elaborate");
    }

    #[test]
    fn accepts_alpha_equivalent_annotation() {
        compile_machine_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
def Test.alpha.{u} (A : Sort u) : (forall (z : A), A) :=
  ((fun (x : A) => x) : forall (y : A), A)",
            &[],
            &MachineCompileOptions::default(),
        )
        .expect("alpha-equivalent annotation should elaborate");
    }

    #[test]
    fn accepts_beta_equivalent_annotation() {
        compile_machine_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
def Test.beta.{u} (A : Sort u) (x : A) : A :=
  (x : (fun (T : Sort u) => T) A)",
            &[],
            &MachineCompileOptions::default(),
        )
        .expect("beta-equivalent annotation should elaborate");
    }

    #[test]
    fn rejects_large_numeric_universe_before_expansion() {
        let err = compile_machine_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "def Test.bad : Sort 1025 := Sort 0",
            &[],
            &MachineCompileOptions::default(),
        )
        .expect_err("oversized numeric universe should be rejected");

        assert_eq!(err.kind, MachineDiagnosticKind::UniverseLevelTooLarge);
    }

    #[test]
    fn elaborates_application_through_reducible_function_type_alias() {
        compile_machine_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
def Test.IdTy.{u} (A : Sort u) : Sort imax u u := forall (x : A), A
def Test.id.{u} (A : Sort u) : Test.IdTy.{u} A := fun (x : A) => x
def Test.use.{u} (A : Sort u) (x : A) : A := Test.id.{u} A x",
            &[],
            &MachineCompileOptions::default(),
        )
        .expect("reducible function type alias should expose the Pi");
    }

    #[test]
    fn elaborates_application_through_reducible_imported_function_type_alias() {
        let imports = [alias_import()];
        compile_machine_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Alias
def Test.use.{u} (A : Sort u) (x : A) : A := Alias.id.{u} A x",
            &imports,
            &MachineCompileOptions::default(),
        )
        .expect("imported reducible definitions should remain reducible");
    }

    #[test]
    fn rejects_incorrect_annotation_before_erasing_it() {
        let imports = [nat_import()];
        let err = compile_machine_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
import Std.Nat.Basic
def Test.bad.{u} (A : Sort u) (x : A) : A := (x : Nat)",
            &imports,
            &MachineCompileOptions::default(),
        )
        .expect_err("incorrect annotation should be rejected");

        assert_eq!(err.kind, MachineDiagnosticKind::TypeMismatch);
    }

    #[test]
    fn rejects_kernel_invalid_declaration_before_exporting_signature() {
        let err = compile_machine_source_to_core(
            FileId(0),
            npa_cert::Name::from_dotted("Test"),
            "\
def Test.bad (A : Type) (x : A) : A := x x
def Test.use (A : Type) (x : A) : A := Test.bad A x",
            &[],
            &MachineCompileOptions::default(),
        )
        .expect_err("ill-typed declaration should be rejected before it is exported");

        assert_eq!(err.kind, MachineDiagnosticKind::KernelRejected);
    }
}
