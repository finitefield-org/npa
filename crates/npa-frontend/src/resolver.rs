use std::collections::{BTreeMap, BTreeSet};

use crate::{
    MachineBinder, MachineDecl, MachineDiagnostic, MachineDiagnosticKind, MachineItem,
    MachineLevel, MachineModule, MachineName, MachineTerm, Result, Span,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedExport {
    pub name: npa_cert::Name,
    pub decl_interface_hash: npa_cert::Hash,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifiedImport {
    pub module: npa_cert::ModuleName,
    pub export_hash: npa_cert::Hash,
    pub certificate_hash: Option<npa_cert::Hash>,
    pub exports: Vec<VerifiedExport>,
}

impl From<&npa_cert::VerifiedModule> for VerifiedImport {
    fn from(module: &npa_cert::VerifiedModule) -> Self {
        let exports = module
            .export_block
            .iter()
            .map(|entry| VerifiedExport {
                name: module.name_table[entry.name].clone(),
                decl_interface_hash: entry.decl_interface_hash,
            })
            .collect();

        Self {
            module: module.module.clone(),
            export_hash: module.export_hash,
            certificate_hash: Some(module.certificate_hash),
            exports,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedMachineModule {
    pub module: MachineModule,
}

pub fn resolve_machine_module(
    module: MachineModule,
    verified_imports: &[VerifiedImport],
) -> Result<ResolvedMachineModule> {
    Resolver::new(verified_imports).resolve_module(module)
}

struct Resolver<'a> {
    verified_imports: &'a [VerifiedImport],
    globals: GlobalTable,
}

impl<'a> Resolver<'a> {
    fn new(verified_imports: &'a [VerifiedImport]) -> Self {
        Self {
            verified_imports,
            globals: GlobalTable::default(),
        }
    }

    fn resolve_module(mut self, module: MachineModule) -> Result<ResolvedMachineModule> {
        let mut seen_imports = BTreeSet::new();
        for item in &module.items {
            if let MachineItem::Import {
                module: import_name,
                span,
            } = item
            {
                let import = self.find_verified_import(import_name, *span)?;
                let import_key = (
                    import.module.clone(),
                    import.export_hash,
                    import.certificate_hash,
                );
                if seen_imports.insert(import_key) {
                    self.globals.add_import(import)?;
                }
            }
        }
        for item in &module.items {
            match item {
                MachineItem::Def(decl) | MachineItem::Theorem(decl) => {
                    self.globals.add_decl_root(&decl.name);
                }
                MachineItem::Import { .. } => {}
            }
        }

        let mut resolved_items = Vec::with_capacity(module.items.len());
        for item in module.items {
            let resolved_item = match item {
                MachineItem::Import { .. } => item,
                MachineItem::Def(decl) => {
                    let resolved = self.resolve_decl(decl)?;
                    self.globals.add_local_decl(&resolved.name)?;
                    MachineItem::Def(resolved)
                }
                MachineItem::Theorem(decl) => {
                    let resolved = self.resolve_decl(decl)?;
                    self.globals.add_local_decl(&resolved.name)?;
                    MachineItem::Theorem(resolved)
                }
            };
            resolved_items.push(resolved_item);
        }

        Ok(ResolvedMachineModule {
            module: MachineModule {
                file_id: module.file_id,
                items: resolved_items,
                span: module.span,
            },
        })
    }

    fn find_verified_import(
        &self,
        import_name: &MachineName,
        span: Span,
    ) -> Result<&'a VerifiedImport> {
        let import_module = name_from_machine(import_name);
        let mut matches = self
            .verified_imports
            .iter()
            .filter(|import| import.module == import_module);
        let Some(first) = matches.next() else {
            return Err(MachineDiagnostic::error(
                MachineDiagnosticKind::MissingVerifiedImport,
                span,
                format!(
                    "import {} is not present in the verified import set",
                    import_name.as_dotted()
                ),
            ));
        };

        if matches.any(|import| import != first) {
            return Err(MachineDiagnostic::error(
                MachineDiagnosticKind::ImportResolutionError,
                span,
                format!(
                    "import {} has multiple verified interfaces",
                    import_name.as_dotted()
                ),
            ));
        }

        Ok(first)
    }

    fn resolve_decl(&self, decl: MachineDecl) -> Result<MachineDecl> {
        let universe_params = self.resolve_universe_params(decl.universe_params)?;
        let universe_param_names: BTreeSet<_> = universe_params
            .iter()
            .map(|param| param.name.clone())
            .collect();
        let mut locals = LocalScope::default();
        let mut binders = Vec::with_capacity(decl.binders.len());

        for binder in decl.binders {
            let binder = self.resolve_binder(binder, &mut locals, &universe_param_names)?;
            binders.push(binder);
        }

        let ty = self.resolve_term(decl.ty, &locals, &universe_param_names)?;
        let value = self.resolve_term(decl.value, &locals, &universe_param_names)?;

        Ok(MachineDecl {
            name: decl.name,
            universe_params,
            binders,
            ty,
            value,
            span: decl.span,
        })
    }

    fn resolve_universe_params(
        &self,
        params: Vec<crate::MachineUniverseParam>,
    ) -> Result<Vec<crate::MachineUniverseParam>> {
        let mut seen = BTreeSet::new();

        for param in &params {
            if !seen.insert(param.name.clone()) {
                return Err(MachineDiagnostic::error(
                    MachineDiagnosticKind::DuplicateUniverseParam,
                    param.span,
                    format!("duplicate universe parameter {}", param.name),
                ));
            }
        }

        Ok(params)
    }

    fn resolve_binder(
        &self,
        binder: MachineBinder,
        locals: &mut LocalScope,
        universe_params: &BTreeSet<String>,
    ) -> Result<MachineBinder> {
        self.ensure_local_does_not_shadow_global(&binder.name, binder.span)?;
        let ty = self.resolve_term(binder.ty, locals, universe_params)?;
        locals.push(binder.name.clone());

        Ok(MachineBinder {
            name: binder.name,
            ty,
            span: binder.span,
        })
    }

    fn resolve_term(
        &self,
        term: MachineTerm,
        locals: &LocalScope,
        universe_params: &BTreeSet<String>,
    ) -> Result<MachineTerm> {
        match term {
            MachineTerm::Ident {
                name,
                universe_args,
                explicit_mode,
                span,
            } => self.resolve_ident(
                name,
                universe_args,
                explicit_mode,
                span,
                locals,
                universe_params,
            ),
            MachineTerm::Local { .. } => Ok(term),
            MachineTerm::Sort { level, span } => Ok(MachineTerm::Sort {
                level: self.resolve_level(level, universe_params)?,
                span,
            }),
            MachineTerm::App { func, arg, span } => Ok(MachineTerm::App {
                func: Box::new(self.resolve_term(*func, locals, universe_params)?),
                arg: Box::new(self.resolve_term(*arg, locals, universe_params)?),
                span,
            }),
            MachineTerm::Lam {
                binders,
                body,
                span,
            } => {
                let mut nested_locals = locals.clone();
                let mut resolved_binders = Vec::with_capacity(binders.len());
                for binder in binders {
                    let binder =
                        self.resolve_binder(binder, &mut nested_locals, universe_params)?;
                    resolved_binders.push(binder);
                }
                Ok(MachineTerm::Lam {
                    binders: resolved_binders,
                    body: Box::new(self.resolve_term(*body, &nested_locals, universe_params)?),
                    span,
                })
            }
            MachineTerm::Pi {
                binders,
                body,
                span,
            } => {
                let mut nested_locals = locals.clone();
                let mut resolved_binders = Vec::with_capacity(binders.len());
                for binder in binders {
                    let binder =
                        self.resolve_binder(binder, &mut nested_locals, universe_params)?;
                    resolved_binders.push(binder);
                }
                Ok(MachineTerm::Pi {
                    binders: resolved_binders,
                    body: Box::new(self.resolve_term(*body, &nested_locals, universe_params)?),
                    span,
                })
            }
            MachineTerm::Let {
                name,
                ty,
                value,
                body,
                span,
            } => {
                self.ensure_local_does_not_shadow_global(&name, span)?;
                let ty = self.resolve_term(*ty, locals, universe_params)?;
                let value = self.resolve_term(*value, locals, universe_params)?;
                let mut nested_locals = locals.clone();
                nested_locals.push(name.clone());
                Ok(MachineTerm::Let {
                    name,
                    ty: Box::new(ty),
                    value: Box::new(value),
                    body: Box::new(self.resolve_term(*body, &nested_locals, universe_params)?),
                    span,
                })
            }
            MachineTerm::Annot { expr, ty, span } => Ok(MachineTerm::Annot {
                expr: Box::new(self.resolve_term(*expr, locals, universe_params)?),
                ty: Box::new(self.resolve_term(*ty, locals, universe_params)?),
                span,
            }),
        }
    }

    fn resolve_ident(
        &self,
        name: MachineName,
        universe_args: Option<Vec<MachineLevel>>,
        explicit_mode: bool,
        span: Span,
        locals: &LocalScope,
        universe_params: &BTreeSet<String>,
    ) -> Result<MachineTerm> {
        if name.parts.len() == 1 && locals.contains(&name.parts[0]) {
            if explicit_mode || universe_args.is_some() {
                return Err(MachineDiagnostic::unsupported_syntax(
                    span,
                    "local names cannot use @ or explicit universe arguments",
                ));
            }

            return Ok(MachineTerm::Local {
                name: name.parts[0].clone(),
                span,
            });
        }

        let universe_args = universe_args
            .map(|args| {
                args.into_iter()
                    .map(|arg| self.resolve_level(arg, universe_params))
                    .collect::<Result<Vec<_>>>()
            })
            .transpose()?;

        match self.globals.lookup(&name) {
            GlobalLookup::Resolved => Ok(MachineTerm::Ident {
                name,
                universe_args,
                explicit_mode,
                span,
            }),
            GlobalLookup::Ambiguous => Err(MachineDiagnostic::error(
                MachineDiagnosticKind::AmbiguousGlobalName,
                name.span,
                format!(
                    "global name {} is exported more than once",
                    name.as_dotted()
                ),
            )),
            GlobalLookup::ShortGlobal => Err(MachineDiagnostic::error(
                MachineDiagnosticKind::ShortGlobalName,
                name.span,
                format!(
                    "global name {} must be written as a fully qualified exact name",
                    name.as_dotted()
                ),
            )),
            GlobalLookup::UnknownLocal => Err(MachineDiagnostic::error(
                MachineDiagnosticKind::UnknownLocalName,
                name.span,
                format!("unknown local name {}", name.as_dotted()),
            )),
            GlobalLookup::UnknownGlobal => Err(MachineDiagnostic::error(
                MachineDiagnosticKind::UnknownGlobalName,
                name.span,
                format!("unknown global name {}", name.as_dotted()),
            )),
        }
    }

    fn resolve_level(
        &self,
        level: MachineLevel,
        universe_params: &BTreeSet<String>,
    ) -> Result<MachineLevel> {
        match level {
            MachineLevel::Nat { .. } => Ok(level),
            MachineLevel::Param { name, span } => {
                if universe_params.contains(&name) {
                    Ok(MachineLevel::Param { name, span })
                } else {
                    Err(MachineDiagnostic::error(
                        MachineDiagnosticKind::UnknownUniverseParam,
                        span,
                        format!("unknown universe parameter {name}"),
                    ))
                }
            }
            MachineLevel::Succ { level, span } => Ok(MachineLevel::Succ {
                level: Box::new(self.resolve_level(*level, universe_params)?),
                span,
            }),
            MachineLevel::Max { lhs, rhs, span } => Ok(MachineLevel::Max {
                lhs: Box::new(self.resolve_level(*lhs, universe_params)?),
                rhs: Box::new(self.resolve_level(*rhs, universe_params)?),
                span,
            }),
            MachineLevel::IMax { lhs, rhs, span } => Ok(MachineLevel::IMax {
                lhs: Box::new(self.resolve_level(*lhs, universe_params)?),
                rhs: Box::new(self.resolve_level(*rhs, universe_params)?),
                span,
            }),
        }
    }

    fn ensure_local_does_not_shadow_global(&self, name: &str, span: Span) -> Result<()> {
        if self.globals.has_root(name) {
            return Err(MachineDiagnostic::error(
                MachineDiagnosticKind::GlobalShadowedByLocal,
                span,
                format!("local name {name} shadows a global namespace root"),
            ));
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
struct LocalScope {
    names: Vec<String>,
}

impl LocalScope {
    fn push(&mut self, name: String) {
        self.names.push(name);
    }

    fn contains(&self, name: &str) -> bool {
        self.names.iter().rev().any(|local| local == name)
    }
}

#[derive(Clone, Debug, Default)]
struct GlobalTable {
    names: BTreeMap<String, GlobalEntry>,
    roots: BTreeSet<String>,
    suffixes: BTreeSet<String>,
}

impl GlobalTable {
    fn add_import(&mut self, import: &VerifiedImport) -> Result<()> {
        for export in &import.exports {
            self.add_global_name(&export.name, Some(export.decl_interface_hash), None)?;
        }

        Ok(())
    }

    fn add_local_decl(&mut self, name: &MachineName) -> Result<()> {
        self.add_global_name(&name_from_machine(name), None, Some(name.span))
    }

    fn add_decl_root(&mut self, name: &MachineName) {
        if let Some(root) = name.parts.first() {
            self.roots.insert(root.clone());
        }
    }

    fn add_global_name(
        &mut self,
        name: &npa_cert::Name,
        interface_hash: Option<npa_cert::Hash>,
        duplicate_span: Option<Span>,
    ) -> Result<()> {
        let dotted = name.as_dotted();
        let first = name.0.first().cloned();
        let last = name.0.last().cloned();

        match self.names.get_mut(&dotted) {
            Some(_) if duplicate_span.is_some() => {
                let span = duplicate_span.expect("local duplicate has a source span");
                return Err(MachineDiagnostic::error(
                    MachineDiagnosticKind::DuplicateDeclaration,
                    span,
                    format!("duplicate declaration {}", name.as_dotted()),
                ));
            }
            Some(GlobalEntry::Resolved { .. }) if duplicate_span.is_none() => {
                self.names.insert(dotted.clone(), GlobalEntry::Ambiguous);
            }
            Some(GlobalEntry::Resolved { .. } | GlobalEntry::Ambiguous) => {}
            None => {
                self.names
                    .insert(dotted, GlobalEntry::Resolved { interface_hash });
            }
        }

        if let Some(first) = first {
            self.roots.insert(first);
        }
        if let Some(last) = last {
            self.suffixes.insert(last);
        }

        Ok(())
    }

    fn lookup(&self, name: &MachineName) -> GlobalLookup {
        let dotted = name.as_dotted();
        match self.names.get(&dotted) {
            Some(GlobalEntry::Resolved { .. }) => GlobalLookup::Resolved,
            Some(GlobalEntry::Ambiguous) => GlobalLookup::Ambiguous,
            None if name.parts.len() == 1 && self.suffixes.contains(&name.parts[0]) => {
                GlobalLookup::ShortGlobal
            }
            None if name.parts.len() == 1 => GlobalLookup::UnknownLocal,
            None => GlobalLookup::UnknownGlobal,
        }
    }

    fn has_root(&self, name: &str) -> bool {
        self.roots.contains(name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum GlobalEntry {
    Resolved {
        interface_hash: Option<npa_cert::Hash>,
    },
    Ambiguous,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GlobalLookup {
    Resolved,
    Ambiguous,
    ShortGlobal,
    UnknownLocal,
    UnknownGlobal,
}

fn name_from_machine(name: &MachineName) -> npa_cert::Name {
    npa_cert::Name(name.parts.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse_machine_module, FileId};

    fn hash(seed: u8) -> npa_cert::Hash {
        [seed; 32]
    }

    fn verified_import(module: &str, exports: &[&str]) -> VerifiedImport {
        VerifiedImport {
            module: npa_cert::Name::from_dotted(module),
            export_hash: hash(1),
            certificate_hash: None,
            exports: exports
                .iter()
                .enumerate()
                .map(|(index, name)| VerifiedExport {
                    name: npa_cert::Name::from_dotted(name),
                    decl_interface_hash: hash(index as u8 + 2),
                })
                .collect(),
        }
    }

    fn resolve_source(source: &str, imports: &[VerifiedImport]) -> Result<ResolvedMachineModule> {
        let module = parse_machine_module(FileId(0), source).expect("source should parse");
        resolve_machine_module(module, imports)
    }

    fn resolve_err(source: &str, imports: &[VerifiedImport]) -> MachineDiagnosticKind {
        resolve_source(source, imports)
            .expect_err("source should fail resolution")
            .kind
    }

    fn nat_import() -> VerifiedImport {
        verified_import("Std.Nat.Basic", &["Nat", "Nat.zero", "Nat.add"])
    }

    #[test]
    fn resolves_imported_globals_and_locals() {
        let imports = [nat_import()];
        let resolved = resolve_source(
            "import Std.Nat.Basic\ndef Test.use (n : Nat) : Nat := Nat.add n Nat.zero",
            &imports,
        )
        .expect("source should resolve");

        let MachineItem::Def(decl) = &resolved.module.items[1] else {
            panic!("expected resolved def");
        };
        let MachineTerm::App { func, arg, .. } = &decl.value else {
            panic!("expected outer application");
        };
        let MachineTerm::Ident { name, .. } = arg.as_ref() else {
            panic!("expected Nat.zero global");
        };
        assert_eq!(name.as_dotted(), "Nat.zero");

        let MachineTerm::App { func, arg, .. } = func.as_ref() else {
            panic!("expected inner application");
        };
        let MachineTerm::Local { name, .. } = arg.as_ref() else {
            panic!("expected local n");
        };
        assert_eq!(name, "n");

        let MachineTerm::Ident { name, .. } = func.as_ref() else {
            panic!("expected Nat.add global");
        };
        assert_eq!(name.as_dotted(), "Nat.add");
    }

    #[test]
    fn resolves_previous_declaration_by_exact_name() {
        let resolved = resolve_source(
            "\
def Test.id (A : Type) (x : A) : A := x
def Test.use (A : Type) (x : A) : A := Test.id A x",
            &[],
        )
        .expect("source should resolve");

        let MachineItem::Def(decl) = &resolved.module.items[1] else {
            panic!("expected second def");
        };
        let MachineTerm::App { func, .. } = &decl.value else {
            panic!("expected application");
        };
        let MachineTerm::App { func, .. } = func.as_ref() else {
            panic!("expected application");
        };
        let MachineTerm::Ident { name, .. } = func.as_ref() else {
            panic!("expected Test.id global");
        };
        assert_eq!(name.as_dotted(), "Test.id");
    }

    #[test]
    fn rejects_missing_verified_import() {
        assert_eq!(
            resolve_err("import Std.Nat.Basic", &[]),
            MachineDiagnosticKind::MissingVerifiedImport
        );
    }

    #[test]
    fn rejects_short_global_suffix() {
        let imports = [nat_import()];
        assert_eq!(
            resolve_err(
                "import Std.Nat.Basic\ndef Test.bad (n : Nat) : Nat := add n Nat.zero",
                &imports,
            ),
            MachineDiagnosticKind::ShortGlobalName
        );
    }

    #[test]
    fn rejects_unknown_global_name() {
        let imports = [nat_import()];
        assert_eq!(
            resolve_err(
                "import Std.Nat.Basic\ndef Test.bad (n : Nat) : Nat := Nat.mul n Nat.zero",
                &imports,
            ),
            MachineDiagnosticKind::UnknownGlobalName
        );
    }

    #[test]
    fn rejects_ambiguous_imported_exact_name() {
        let imports = [
            verified_import("Std.Nat.Left", &["Nat", "Nat.zero"]),
            verified_import("Std.Nat.Right", &["Nat", "Nat.zero"]),
        ];
        assert_eq!(
            resolve_err(
                "\
import Std.Nat.Left
import Std.Nat.Right
def Test.bad : Nat := Nat.zero",
                &imports,
            ),
            MachineDiagnosticKind::AmbiguousGlobalName
        );
    }

    #[test]
    fn repeated_same_import_does_not_make_exports_ambiguous() {
        let imports = [nat_import()];
        resolve_source(
            "\
import Std.Nat.Basic
import Std.Nat.Basic
def Test.ok : Nat := Nat.zero",
            &imports,
        )
        .expect("duplicate source import should resolve to the same verified import");
    }

    #[test]
    fn repeated_identical_verified_import_is_accepted() {
        let import = nat_import();
        let imports = [import.clone(), import];
        resolve_source(
            "\
import Std.Nat.Basic
def Test.ok : Nat := Nat.zero",
            &imports,
        )
        .expect("identical verified imports should be order independent");
    }

    #[test]
    fn rejects_duplicate_verified_import_interfaces_for_one_module() {
        let first = nat_import();
        let mut second = verified_import("Std.Nat.Basic", &["Nat", "Nat.succ"]);
        second.export_hash = hash(9);
        let imports = [first, second];

        assert_eq!(
            resolve_err("import Std.Nat.Basic", &imports),
            MachineDiagnosticKind::ImportResolutionError
        );
    }

    #[test]
    fn rejects_global_shadowed_by_local() {
        let imports = [nat_import()];
        assert_eq!(
            resolve_err(
                "import Std.Nat.Basic\ndef Test.bad (Nat : Type) : Nat := Nat",
                &imports,
            ),
            MachineDiagnosticKind::GlobalShadowedByLocal
        );
    }

    #[test]
    fn rejects_current_declaration_root_shadowed_by_local() {
        assert_eq!(
            resolve_err("def Test.bad (Test : Type) : Test := Test", &[]),
            MachineDiagnosticKind::GlobalShadowedByLocal
        );
    }

    #[test]
    fn rejects_current_declaration_root_shadowed_by_let() {
        assert_eq!(
            resolve_err(
                "def Test.bad : Type := let Test : Type := Type in Test",
                &[],
            ),
            MachineDiagnosticKind::GlobalShadowedByLocal
        );
    }

    #[test]
    fn rejects_future_declaration_root_shadowed_by_local() {
        assert_eq!(
            resolve_err(
                "\
def A.f (B : Type) : B := B
def B.g : Type := Type",
                &[],
            ),
            MachineDiagnosticKind::GlobalShadowedByLocal
        );
    }

    #[test]
    fn rejects_duplicate_declaration() {
        assert_eq!(
            resolve_err(
                "\
def Test.x : Type := Type
def Test.x : Type := Type",
                &[],
            ),
            MachineDiagnosticKind::DuplicateDeclaration
        );
    }

    #[test]
    fn rejects_duplicate_universe_param() {
        assert_eq!(
            resolve_err("def Test.bad.{u,u} : Sort u := Sort u", &[]),
            MachineDiagnosticKind::DuplicateUniverseParam
        );
    }

    #[test]
    fn rejects_unknown_universe_param() {
        assert_eq!(
            resolve_err("def Test.bad.{u} : Sort v := Sort u", &[]),
            MachineDiagnosticKind::UnknownUniverseParam
        );
    }
}
