use std::collections::BTreeSet;

use crate::resolver::{
    find_unique_verified_import_by_module, verified_import_identity, VerifiedImportIdentity,
    VerifiedImportLookupError,
};
use crate::{
    HumanAxiomDecl, HumanBinder, HumanBinderKind, HumanCompileOptions, HumanDecl, HumanDiagnostic,
    HumanDiagnosticKind, HumanFrontendState, HumanGeneratedDeclarationKind,
    HumanGeneratedDeclarationMetadata, HumanImportedSourceInterface, HumanInductiveDecl, HumanItem,
    HumanModule, HumanName, HumanOpenScope, HumanOpenScopeFrame, HumanResult,
    HumanSourceBinderMetadata, HumanSourceDeclarationKind, HumanSourceDeclarationMetadata,
    HumanSourceInterface, HumanSourceNotationMetadata, VerifiedImport,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedHumanModule {
    pub module: HumanModule,
    pub state: HumanFrontendState,
}

pub fn resolve_human_module(
    module_name: npa_cert::ModuleName,
    module: HumanModule,
    verified_imports: &[VerifiedImport],
    _options: &HumanCompileOptions,
) -> HumanResult<ResolvedHumanModule> {
    HumanResolver::new(module_name, verified_imports).resolve_module(module)
}

struct HumanResolver<'a> {
    verified_imports: &'a [VerifiedImport],
    state: HumanFrontendState,
    seen_imports: BTreeSet<VerifiedImportIdentity>,
}

impl<'a> HumanResolver<'a> {
    fn new(module_name: npa_cert::ModuleName, verified_imports: &'a [VerifiedImport]) -> Self {
        Self {
            verified_imports,
            state: HumanFrontendState::new(module_name),
            seen_imports: BTreeSet::new(),
        }
    }

    fn resolve_module(mut self, module: HumanModule) -> HumanResult<ResolvedHumanModule> {
        for item in &module.items {
            if let HumanItem::Import { module, span } = item {
                self.add_import(module, *span)?;
            }
        }

        for item in &module.items {
            self.record_item_metadata(item);
        }

        Ok(ResolvedHumanModule {
            module,
            state: self.state,
        })
    }

    fn add_import(&mut self, module: &HumanName, span: crate::Span) -> HumanResult<()> {
        let import_module = name_from_human(module);
        let import =
            match find_unique_verified_import_by_module(self.verified_imports, &import_module) {
                Ok(import) => import,
                Err(VerifiedImportLookupError::Missing) => {
                    return Err(HumanDiagnostic::error(
                        HumanDiagnosticKind::MissingVerifiedImport,
                        span,
                        format!(
                            "import {} is not present in the verified import set",
                            module.as_dotted()
                        ),
                    ));
                }
                Err(VerifiedImportLookupError::Ambiguous) => {
                    return Err(HumanDiagnostic::error(
                        HumanDiagnosticKind::ImportResolutionError,
                        span,
                        format!(
                            "import {} has multiple verified interfaces",
                            module.as_dotted()
                        ),
                    ));
                }
            };

        if self.seen_imports.insert(verified_import_identity(import)) {
            self.state
                .source_interfaces
                .imports
                .push(imported_source_interface(import));
        }

        Ok(())
    }

    fn record_item_metadata(&mut self, item: &HumanItem) {
        match item {
            HumanItem::Import { .. } => {}
            HumanItem::Open { namespace, span } => {
                let open = HumanOpenScope {
                    namespace: self.qualify_name(namespace),
                    span: *span,
                };
                self.current_open_frame().opens.push(open);
            }
            HumanItem::NamespaceStart { name, .. } => {
                let namespace = self.qualify_name(name);
                self.state.namespace_stack.push(name.clone());
                self.state.open_scopes.push(HumanOpenScopeFrame {
                    namespace: Some(namespace),
                    opens: Vec::new(),
                });
            }
            HumanItem::NamespaceEnd { .. } => {
                if !self.state.namespace_stack.is_empty() {
                    self.state.namespace_stack.pop();
                }
                if self.state.open_scopes.len() > 1 {
                    self.state.open_scopes.pop();
                }
            }
            HumanItem::Def(decl) => {
                let metadata = self.decl_metadata(HumanSourceDeclarationKind::Def, decl);
                self.state
                    .source_interfaces
                    .current
                    .declarations
                    .push(metadata);
            }
            HumanItem::Theorem(decl) => {
                let metadata = self.decl_metadata(HumanSourceDeclarationKind::Theorem, decl);
                self.state
                    .source_interfaces
                    .current
                    .declarations
                    .push(metadata);
            }
            HumanItem::Axiom(decl) => {
                let metadata = self.axiom_metadata(decl);
                self.state
                    .source_interfaces
                    .current
                    .declarations
                    .push(metadata);
            }
            HumanItem::Inductive(decl) => {
                let metadata = self.inductive_metadata(decl);
                self.state
                    .source_interfaces
                    .current
                    .declarations
                    .push(metadata);
                self.record_generated_inductive_metadata(decl);
            }
            HumanItem::Notation(decl) => {
                let metadata = HumanSourceNotationMetadata {
                    kind: decl.kind,
                    precedence: decl.precedence,
                    token: decl.token.clone(),
                    target: decl.target.clone(),
                    namespace: self.current_namespace_parts(),
                    span: decl.span,
                };
                self.state.notation_table.push(metadata.clone());
                self.state
                    .source_interfaces
                    .current
                    .notations
                    .push(metadata);
            }
        }
    }

    fn decl_metadata(
        &self,
        kind: HumanSourceDeclarationKind,
        decl: &HumanDecl,
    ) -> HumanSourceDeclarationMetadata {
        HumanSourceDeclarationMetadata {
            kind,
            name: self.qualify_name(&decl.name),
            universe_params: decl.universe_params.clone(),
            binders: binder_metadata(&decl.binders),
            decl_interface_hash: None,
            span: decl.span,
        }
    }

    fn axiom_metadata(&self, decl: &HumanAxiomDecl) -> HumanSourceDeclarationMetadata {
        HumanSourceDeclarationMetadata {
            kind: HumanSourceDeclarationKind::Axiom,
            name: self.qualify_name(&decl.name),
            universe_params: decl.universe_params.clone(),
            binders: binder_metadata(&decl.binders),
            decl_interface_hash: None,
            span: decl.span,
        }
    }

    fn inductive_metadata(&self, decl: &HumanInductiveDecl) -> HumanSourceDeclarationMetadata {
        HumanSourceDeclarationMetadata {
            kind: HumanSourceDeclarationKind::Inductive,
            name: self.qualify_name(&decl.name),
            universe_params: decl.universe_params.clone(),
            binders: binder_metadata(&decl.binders),
            decl_interface_hash: None,
            span: decl.span,
        }
    }

    fn record_generated_inductive_metadata(&mut self, decl: &HumanInductiveDecl) {
        let parent = self.qualify_name(&decl.name);
        let generated: Vec<_> = decl
            .constructors
            .iter()
            .map(|constructor| HumanGeneratedDeclarationMetadata {
                kind: HumanGeneratedDeclarationKind::Constructor,
                parent: parent.clone(),
                name: relative_child_name(&parent, &constructor.name),
                decl_interface_hash: None,
                span: constructor.span,
            })
            .chain(std::iter::once(HumanGeneratedDeclarationMetadata {
                kind: HumanGeneratedDeclarationKind::Recursor,
                parent: parent.clone(),
                name: generated_recursor_name(&parent),
                decl_interface_hash: None,
                span: decl.span,
            }))
            .collect();

        self.state
            .source_interfaces
            .current
            .generated_declarations
            .extend(generated);
    }

    fn current_open_frame(&mut self) -> &mut HumanOpenScopeFrame {
        if self.state.open_scopes.is_empty() {
            self.state.open_scopes.push(HumanOpenScopeFrame {
                namespace: None,
                opens: Vec::new(),
            });
        }
        self.state
            .open_scopes
            .last_mut()
            .expect("open scope stack has a top-level frame")
    }

    fn qualify_name(&self, name: &HumanName) -> HumanName {
        let mut parts = self.current_namespace_parts();
        parts.extend(name.parts.iter().cloned());
        HumanName::new(parts, name.span)
    }

    fn current_namespace_parts(&self) -> Vec<String> {
        self.state
            .namespace_stack
            .iter()
            .flat_map(|name| name.parts.iter().cloned())
            .collect()
    }
}

fn binder_metadata(binders: &[HumanBinder]) -> Vec<HumanSourceBinderMetadata> {
    binders
        .iter()
        .map(|binder| HumanSourceBinderMetadata {
            name: match &binder.kind {
                HumanBinderKind::Named(name) => Some(name.clone()),
                HumanBinderKind::Anonymous => None,
            },
            binder_info: binder.binder_info,
            span: binder.span,
        })
        .collect()
}

fn imported_source_interface(import: &VerifiedImport) -> HumanImportedSourceInterface {
    let mut source_interface = HumanSourceInterface::new(import.module.clone());
    source_interface.declarations = import
        .exports
        .iter()
        .map(|export| HumanSourceDeclarationMetadata {
            kind: HumanSourceDeclarationKind::Imported,
            name: HumanName::new(export.name.0.clone(), crate::Span::empty(crate::FileId(0))),
            universe_params: export
                .universe_params
                .iter()
                .cloned()
                .map(|name| crate::HumanUniverseParam {
                    name,
                    span: crate::Span::empty(crate::FileId(0)),
                })
                .collect(),
            binders: Vec::new(),
            decl_interface_hash: Some(export.decl_interface_hash),
            span: crate::Span::empty(crate::FileId(0)),
        })
        .collect();

    HumanImportedSourceInterface {
        module: import.module.clone(),
        export_hash: import.export_hash,
        certificate_hash: import.certificate_hash,
        source_interface,
    }
}

fn relative_child_name(parent: &HumanName, child: &HumanName) -> HumanName {
    let mut parts = parent.parts.clone();
    parts.extend(child.parts.iter().cloned());
    HumanName::new(parts, child.span)
}

fn generated_recursor_name(parent: &HumanName) -> HumanName {
    let mut parts = parent.parts.clone();
    parts.push("rec".to_owned());
    HumanName::new(parts, parent.span)
}

fn name_from_human(name: &HumanName) -> npa_cert::Name {
    npa_cert::Name(name.parts.clone())
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::*;
    use crate::{parse_human_module, HumanBinderInfo, HumanDiagnosticKind, HumanNotationKind};

    fn hash(seed: u8) -> npa_cert::Hash {
        [seed; 32]
    }

    fn verified_import(module: &str, exports: &[&str]) -> VerifiedImport {
        let exports: Vec<_> = exports
            .iter()
            .enumerate()
            .map(|(index, name)| crate::VerifiedExport {
                name: npa_cert::Name::from_dotted(name),
                universe_params: Vec::new(),
                ty: npa_kernel::Expr::sort(npa_kernel::Level::zero()),
                decl_interface_hash: hash(index as u8 + 2),
            })
            .collect();
        let kernel_decls = exports
            .iter()
            .map(|export| npa_kernel::Decl::Axiom {
                name: export.name.as_dotted(),
                universe_params: export.universe_params.clone(),
                ty: export.ty.clone(),
            })
            .collect();

        VerifiedImport {
            module: npa_cert::Name::from_dotted(module),
            export_hash: hash(1),
            certificate_hash: None,
            decl_interface_hashes: exports
                .iter()
                .map(|export| (export.name.clone(), export.decl_interface_hash))
                .collect(),
            exports,
            kernel_decls,
            kernel_decl_dependencies: BTreeMap::new(),
        }
    }

    fn resolve_source(
        source: &str,
        imports: &[VerifiedImport],
    ) -> HumanResult<ResolvedHumanModule> {
        let module = parse_human_module(crate::FileId(0), source).expect("source should parse");
        resolve_human_module(
            npa_cert::Name::from_dotted("Current.Module"),
            module,
            imports,
            &crate::HumanCompileOptions::default(),
        )
    }

    #[test]
    fn human_frontend_state_records_module_namespace_and_lexical_open_scopes() {
        let resolved = resolve_source(
            "\
open Std
namespace Demo
open Nat
def id {A : Type} (x : A) : A := x
end Demo",
            &[],
        )
        .expect("source should resolve to metadata");

        assert_eq!(
            resolved.state.current_module,
            npa_cert::Name::from_dotted("Current.Module")
        );
        assert!(resolved.state.namespace_stack.is_empty());
        assert_eq!(resolved.state.open_scopes.len(), 1);
        assert_eq!(resolved.state.open_scopes[0].opens.len(), 1);
        assert_eq!(
            resolved.state.open_scopes[0].opens[0].namespace.as_dotted(),
            "Std"
        );

        let decl = &resolved.state.source_interfaces.current.declarations[0];
        assert_eq!(decl.name.as_dotted(), "Demo.id");
        assert_eq!(decl.binders.len(), 2);
        assert_eq!(decl.binders[0].binder_info, HumanBinderInfo::Implicit);
        assert_eq!(decl.binders[1].binder_info, HumanBinderInfo::Explicit);
    }

    #[test]
    fn human_imports_are_checked_against_verified_imports_and_deduped() {
        let import = verified_import("Std.Nat.Basic", &["Nat", "Nat.zero"]);
        let resolved = resolve_source(
            "\
import Std.Nat.Basic
import Std.Nat.Basic",
            std::slice::from_ref(&import),
        )
        .expect("duplicate same source import should resolve deterministically");

        assert_eq!(resolved.state.source_interfaces.imports.len(), 1);
        let imported = &resolved.state.source_interfaces.imports[0];
        assert_eq!(
            imported.module,
            npa_cert::Name::from_dotted("Std.Nat.Basic")
        );
        assert_eq!(imported.export_hash, hash(1));
        assert_eq!(imported.source_interface.declarations.len(), 2);
        assert_eq!(
            imported.source_interface.declarations[1].decl_interface_hash,
            Some(hash(3))
        );
    }

    #[test]
    fn human_resolver_rejects_missing_verified_import() {
        let err = resolve_source("import Std.Nat.Basic", &[])
            .expect_err("missing import should fail")
            .kind;

        assert_eq!(err, HumanDiagnosticKind::MissingVerifiedImport);
    }

    #[test]
    fn human_resolver_rejects_ambiguous_verified_import_interfaces() {
        let first = verified_import("Std.Nat.Basic", &["Nat"]);
        let mut second = verified_import("Std.Nat.Basic", &["Nat.zero"]);
        second.export_hash = hash(9);
        let err = resolve_source("import Std.Nat.Basic", &[first, second])
            .expect_err("ambiguous import should fail")
            .kind;

        assert_eq!(err, HumanDiagnosticKind::ImportResolutionError);
    }

    #[test]
    fn human_source_interface_records_notation_and_generated_display_metadata() {
        let resolved = resolve_source(
            "\
namespace Nat
notation \"zero\" => Nat.zero
inductive List : Type where
| nil : List
| cons : List -> List",
            &[],
        )
        .expect("source should resolve to metadata");

        assert_eq!(resolved.state.notation_table.len(), 1);
        let notation = &resolved.state.source_interfaces.current.notations[0];
        assert_eq!(notation.kind, HumanNotationKind::Notation);
        assert_eq!(notation.token, "zero");
        assert_eq!(notation.namespace, vec!["Nat".to_owned()]);

        let generated = &resolved
            .state
            .source_interfaces
            .current
            .generated_declarations;
        assert_eq!(generated.len(), 3);
        assert_eq!(generated[0].name.as_dotted(), "Nat.List.nil");
        assert_eq!(generated[1].name.as_dotted(), "Nat.List.cons");
        assert_eq!(generated[2].kind, HumanGeneratedDeclarationKind::Recursor);
        assert_eq!(generated[2].name.as_dotted(), "Nat.List.rec");
    }

    #[test]
    fn human_metadata_is_frontend_only_and_core_certificates_do_not_require_it() {
        let module = npa_cert::CoreModule {
            name: npa_cert::Name::from_dotted("Meta.Free"),
            declarations: Vec::new(),
        };
        let cert = npa_cert::build_module_cert(module, &[]).expect("core cert should build");
        let bytes = npa_cert::encode_module_cert(&cert).expect("cert should encode");
        let mut session = npa_cert::VerifierSession::new();
        let verified =
            npa_cert::verify_module_cert(&bytes, &mut session, &npa_cert::AxiomPolicy::normal())
                .expect("cert should verify without Human metadata");

        assert_eq!(verified.module(), &npa_cert::Name::from_dotted("Meta.Free"));
    }

    #[test]
    fn duplicate_identical_verified_imports_are_accepted_for_human_resolution() {
        let import = verified_import("Std.Nat.Basic", &["Nat"]);
        resolve_source("import Std.Nat.Basic", &[import.clone(), import])
            .expect("identical verified import entries should be accepted");
    }

    #[test]
    fn seen_import_identity_order_is_deterministic() {
        let left = verified_import("A", &["A.x"]);
        let right = verified_import("B", &["B.x"]);
        let resolved = resolve_source(
            "\
import B
import A
import B",
            &[left, right],
        )
        .expect("imports should resolve");
        let imported_modules: BTreeSet<_> = resolved
            .state
            .source_interfaces
            .imports
            .iter()
            .map(|import| import.module.as_dotted())
            .collect();

        assert_eq!(
            imported_modules,
            BTreeSet::from(["A".to_owned(), "B".to_owned()])
        );
        assert_eq!(resolved.state.source_interfaces.imports.len(), 2);
        assert_eq!(
            resolved.state.source_interfaces.imports[0]
                .module
                .as_dotted(),
            "B"
        );
        assert_eq!(
            resolved.state.source_interfaces.imports[1]
                .module
                .as_dotted(),
            "A"
        );
    }
}
