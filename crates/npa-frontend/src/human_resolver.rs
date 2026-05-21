use std::collections::{BTreeMap, BTreeSet};

use crate::resolver::{
    find_unique_verified_import_by_module, verified_import_identity, VerifiedImportIdentity,
    VerifiedImportLookupError,
};
use crate::{
    HumanAxiomDecl, HumanBinder, HumanBinderKind, HumanCompileOptions, HumanDecl, HumanDiagnostic,
    HumanDiagnosticKind, HumanDiagnosticPayload, HumanDiagnosticPhase, HumanExpr,
    HumanFrontendState, HumanGeneratedDeclarationKind, HumanGeneratedDeclarationMetadata,
    HumanImportedSourceInterface, HumanInductiveDecl, HumanItem, HumanModule, HumanName,
    HumanNotationAssociativity, HumanNotationHead, HumanNotationKind, HumanOpenScope,
    HumanOpenScopeFrame, HumanResult, HumanSourceBinderMetadata, HumanSourceDeclarationKind,
    HumanSourceDeclarationMetadata, HumanSourceInterface, HumanSourceNotationMetadata, Span,
    VerifiedImport,
};

const MAX_HUMAN_NAME_CANDIDATES: usize = 32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedHumanModule {
    pub module: HumanModule,
    pub state: HumanFrontendState,
    pub global_scope: HumanGlobalScope,
    pub resolved_names: Vec<HumanResolvedNameUse>,
    pub notation_table: Vec<HumanResolvedNotationEntry>,
    pub resolved_notations: Vec<HumanResolvedNotationUse>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HumanGlobalScope {
    pub current: Vec<HumanGlobalScopeEntry>,
    pub imported: Vec<HumanGlobalScopeEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanGlobalScopeEntry {
    pub name: HumanName,
    pub reference: HumanGlobalRef,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HumanGlobalRef {
    Imported {
        module: npa_cert::ModuleName,
        name: npa_cert::Name,
        decl_interface_hash: npa_cert::Hash,
    },
    Local {
        index: usize,
        name: npa_cert::Name,
    },
    LocalGenerated {
        index: usize,
        name: npa_cert::Name,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanResolvedNameUse {
    pub source: HumanName,
    pub resolved: HumanResolvedName,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HumanResolvedName {
    Local {
        name: HumanName,
        de_bruijn_index: usize,
    },
    Global(HumanGlobalRef),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanResolvedNotationEntry {
    pub kind: HumanNotationKind,
    pub associativity: HumanNotationAssociativity,
    pub precedence: u16,
    pub token: String,
    pub target: HumanGlobalRef,
    pub namespace: Vec<String>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanResolvedNotationUse {
    pub head: HumanNotationHead,
    pub candidates: Vec<HumanGlobalRef>,
}

pub fn resolve_human_module(
    module_name: npa_cert::ModuleName,
    module: HumanModule,
    verified_imports: &[VerifiedImport],
    options: &HumanCompileOptions,
) -> HumanResult<ResolvedHumanModule> {
    HumanResolver::new(module_name, verified_imports, options)
        .resolve_module(module)
        .map_err(|diagnostic| diagnostic.with_default_phase(HumanDiagnosticPhase::Resolver))
}

struct HumanResolver<'a> {
    verified_imports: &'a [VerifiedImport],
    max_notation_candidates: usize,
    state: HumanFrontendState,
    global_scope: HumanGlobalScope,
    resolved_names: Vec<HumanResolvedNameUse>,
    notation_table: Vec<HumanResolvedNotationEntry>,
    resolved_notations: Vec<HumanResolvedNotationUse>,
    notation_scopes: Vec<HumanNotationScope>,
    namespace_notations: BTreeMap<Vec<String>, Vec<HumanResolvedNotationEntry>>,
    seen_imports: BTreeSet<VerifiedImportIdentity>,
    pending_current_names: BTreeSet<npa_cert::Name>,
    temporary_globals: Vec<HumanGlobalScopeEntry>,
}

impl<'a> HumanResolver<'a> {
    fn new(
        module_name: npa_cert::ModuleName,
        verified_imports: &'a [VerifiedImport],
        options: &HumanCompileOptions,
    ) -> Self {
        Self {
            verified_imports,
            max_notation_candidates: options.max_notation_candidates,
            state: HumanFrontendState::new(module_name),
            global_scope: HumanGlobalScope::default(),
            resolved_names: Vec::new(),
            notation_table: Vec::new(),
            resolved_notations: Vec::new(),
            notation_scopes: vec![HumanNotationScope::default()],
            namespace_notations: BTreeMap::new(),
            seen_imports: BTreeSet::new(),
            pending_current_names: BTreeSet::new(),
            temporary_globals: Vec::new(),
        }
    }

    fn resolve_module(mut self, module: HumanModule) -> HumanResult<ResolvedHumanModule> {
        self.pending_current_names = planned_current_names(&module);

        for item in &module.items {
            if let HumanItem::Import { module, span } = item {
                self.add_import(module, *span)?;
            }
        }

        for item in &module.items {
            self.resolve_item(item)?;
        }

        Ok(ResolvedHumanModule {
            module,
            state: self.state,
            global_scope: self.global_scope,
            resolved_names: self.resolved_names,
            notation_table: self.notation_table,
            resolved_notations: self.resolved_notations,
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
            self.add_imported_globals(import);
        }

        Ok(())
    }

    fn close_namespace(&mut self, name: Option<&HumanName>, span: Span) -> HumanResult<()> {
        let Some(top) = self.state.namespace_stack.pop() else {
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::NamespaceMismatch,
                span,
                "end has no matching namespace",
            ));
        };

        if let Some(expected) = name {
            if top.parts != expected.parts {
                return Err(HumanDiagnostic::error(
                    HumanDiagnosticKind::NamespaceMismatch,
                    span,
                    format!(
                        "end {} does not match namespace {}",
                        expected.as_dotted(),
                        top.as_dotted()
                    ),
                ));
            }
        }

        if self.state.open_scopes.len() > 1 {
            self.state.open_scopes.pop();
        }

        Ok(())
    }

    fn resolve_decl_terms(&mut self, decl: &HumanDecl) -> HumanResult<()> {
        let mut locals = HumanLocalScope::default();
        self.resolve_binders(&decl.binders, &mut locals)?;
        self.resolve_expr(&decl.ty, &mut locals)?;
        self.resolve_expr(&decl.value, &mut locals)
    }

    fn resolve_axiom_terms(&mut self, decl: &HumanAxiomDecl) -> HumanResult<()> {
        let mut locals = HumanLocalScope::default();
        self.resolve_binders(&decl.binders, &mut locals)?;
        self.resolve_expr(&decl.ty, &mut locals)
    }

    fn resolve_inductive_terms(
        &mut self,
        decl: &HumanInductiveDecl,
        head_name: &HumanName,
        local_index: usize,
    ) -> HumanResult<()> {
        let mut locals = HumanLocalScope::default();
        self.resolve_binders(&decl.binders, &mut locals)?;
        self.resolve_expr(&decl.ty, &mut locals)?;

        let temporary = HumanGlobalScopeEntry {
            name: head_name.clone(),
            reference: HumanGlobalRef::Local {
                index: local_index,
                name: name_from_human(head_name),
            },
            span: decl.span,
        };
        self.temporary_globals.push(temporary);
        let result = decl
            .constructors
            .iter()
            .try_for_each(|constructor| self.resolve_expr(&constructor.ty, &mut locals));
        self.temporary_globals.pop();
        result
    }

    fn resolve_binders(
        &mut self,
        binders: &[HumanBinder],
        locals: &mut HumanLocalScope,
    ) -> HumanResult<()> {
        for binder in binders {
            if let Some(ty) = &binder.ty {
                self.resolve_expr(ty, locals)?;
            }
            if let HumanBinderKind::Named(name) = &binder.kind {
                locals.push(name.clone());
            }
        }

        Ok(())
    }

    fn resolve_expr(&mut self, expr: &HumanExpr, locals: &mut HumanLocalScope) -> HumanResult<()> {
        match expr {
            HumanExpr::Ident { name, span, .. } => {
                let resolved = self.resolve_name(name, locals, *span)?;
                self.resolved_names.push(HumanResolvedNameUse {
                    source: name.clone(),
                    resolved,
                });
            }
            HumanExpr::Sort { .. } | HumanExpr::Hole { .. } => {}
            HumanExpr::App { func, arg, .. } => {
                self.resolve_expr(func, locals)?;
                self.resolve_expr(arg, locals)?;
            }
            HumanExpr::Lam { binders, body, .. } | HumanExpr::Pi { binders, body, .. } => {
                let mut nested = locals.clone();
                self.resolve_binders(binders, &mut nested)?;
                self.resolve_expr(body, &mut nested)?;
            }
            HumanExpr::Let {
                name,
                ty,
                value,
                body,
                ..
            } => {
                if let Some(ty) = ty {
                    self.resolve_expr(ty, locals)?;
                }
                self.resolve_expr(value, locals)?;
                let mut nested = locals.clone();
                nested.push(name.clone());
                self.resolve_expr(body, &mut nested)?;
            }
            HumanExpr::Annot { expr, ty, .. } => {
                self.resolve_expr(expr, locals)?;
                self.resolve_expr(ty, locals)?;
            }
            HumanExpr::Arrow {
                domain, codomain, ..
            } => {
                self.resolve_expr(domain, locals)?;
                self.resolve_expr(codomain, locals)?;
            }
            HumanExpr::NotationApp { head, args, .. } => {
                for arg in args {
                    self.resolve_expr(arg, locals)?;
                }
                let candidates = self.resolve_notation_candidates(head)?;
                self.resolved_notations.push(HumanResolvedNotationUse {
                    head: head.clone(),
                    candidates,
                });
            }
        }

        Ok(())
    }

    fn resolve_name(
        &self,
        name: &HumanName,
        locals: &HumanLocalScope,
        span: Span,
    ) -> HumanResult<HumanResolvedName> {
        if name.parts.len() == 1 {
            if let Some((local_name, de_bruijn_index)) = locals.lookup(&name.parts[0]) {
                return Ok(HumanResolvedName::Local {
                    name: local_name,
                    de_bruijn_index,
                });
            }
        }

        if let Some(resolved) = self.resolve_global_name(name)? {
            return Ok(HumanResolvedName::Global(resolved));
        }

        let forward_candidates = self.forward_reference_candidates(name);
        if !forward_candidates.is_empty() {
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::ForwardReference,
                span,
                format!("{} refers to a later declaration", name.as_dotted()),
            )
            .with_payload(candidate_payload(forward_candidates)));
        }

        Err(HumanDiagnostic::error(
            HumanDiagnosticKind::UnknownIdentifier,
            span,
            format!("unknown identifier {}", name.as_dotted()),
        ))
    }

    fn resolve_namespace(&self, name: &HumanName) -> HumanResult<HumanName> {
        let candidates = self.namespace_candidates(name);
        match candidates.len() {
            0 => Err(HumanDiagnostic::error(
                HumanDiagnosticKind::UnknownNamespace,
                name.span,
                format!("unknown namespace {}", name.as_dotted()),
            )),
            1 => Ok(HumanName::new(candidates[0].0.clone(), name.span)),
            _ => Err(HumanDiagnostic::error(
                HumanDiagnosticKind::AmbiguousName,
                name.span,
                format!("ambiguous namespace {}", name.as_dotted()),
            )
            .with_payload(candidate_payload(
                candidates
                    .into_iter()
                    .map(|candidate| candidate.as_dotted())
                    .collect(),
            ))),
        }
    }

    fn resolve_global_name(&self, name: &HumanName) -> HumanResult<Option<HumanGlobalRef>> {
        for candidates in self.global_candidate_levels(name) {
            let mut candidates = dedupe_and_sort_candidates(candidates);
            if candidates.is_empty() {
                continue;
            }
            if candidates.len() == 1 {
                return Ok(Some(candidates.remove(0).reference));
            }

            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::AmbiguousName,
                name.span,
                format!("ambiguous name {}", name.as_dotted()),
            )
            .with_payload(candidate_payload(
                candidates
                    .into_iter()
                    .map(|candidate| candidate.key)
                    .collect(),
            )));
        }

        Ok(None)
    }

    fn resolve_notation_candidates(
        &self,
        head: &HumanNotationHead,
    ) -> HumanResult<Vec<HumanGlobalRef>> {
        let mut candidates = BTreeMap::new();
        for entry in self
            .notation_scopes
            .iter()
            .flat_map(|scope| scope.entries.iter())
            .filter(|entry| {
                entry.token == head.token
                    && entry.kind == head.kind
                    && entry.precedence == head.precedence
                    && entry.associativity == head.associativity
            })
        {
            candidates.insert(global_ref_sort_key(&entry.target), entry.target.clone());
            if candidates.len() > self.max_notation_candidates {
                if let Some(last_key) = candidates.keys().next_back().cloned() {
                    candidates.remove(&last_key);
                }
            }
        }

        let active_candidate_count = self.active_notation_candidate_count(head);
        if active_candidate_count > self.max_notation_candidates {
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::TooManyNotationCandidates,
                head.span,
                format!("notation {} has too many candidates", head.token),
            )
            .with_payload(candidate_payload(self.active_notation_candidate_keys(head))));
        }
        if candidates.is_empty() {
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::AmbiguousNotation,
                head.span,
                format!("notation {} has no resolved candidates", head.token),
            ));
        }

        Ok(candidates.into_values().collect())
    }

    fn active_notation_candidate_count(&self, head: &HumanNotationHead) -> usize {
        let mut seen = BTreeSet::new();
        for entry in self
            .notation_scopes
            .iter()
            .flat_map(|scope| scope.entries.iter())
        {
            if entry.token == head.token
                && entry.kind == head.kind
                && entry.precedence == head.precedence
                && entry.associativity == head.associativity
            {
                seen.insert(global_ref_sort_key(&entry.target));
                if seen.len() > self.max_notation_candidates {
                    break;
                }
            }
        }
        seen.len()
    }
    fn active_notation_candidate_keys(&self, head: &HumanNotationHead) -> Vec<String> {
        let mut keys = BTreeSet::new();
        for entry in self
            .notation_scopes
            .iter()
            .flat_map(|scope| scope.entries.iter())
        {
            if entry.token == head.token
                && entry.kind == head.kind
                && entry.precedence == head.precedence
                && entry.associativity == head.associativity
            {
                keys.insert(global_ref_sort_key(&entry.target));
                if keys.len() > self.max_notation_candidates {
                    if let Some(last) = keys.iter().next_back().cloned() {
                        keys.remove(&last);
                    }
                }
            }
        }
        keys.into_iter().collect()
    }

    fn global_candidate_levels(&self, name: &HumanName) -> Vec<Vec<HumanNameCandidate>> {
        if name.parts.len() == 1 {
            vec![
                self.lookup_exact_candidates(&self.relative_to_current_namespace(name)),
                self.opened_namespace_candidates(name),
                self.short_name_candidates(&name.parts[0]),
            ]
        } else {
            let mut levels = vec![self.lookup_exact_candidates(&name_from_human(name))];
            let current_relative = self.relative_to_current_namespace(name);
            if current_relative != name_from_human(name) {
                levels.push(self.lookup_exact_candidates(&current_relative));
            }
            levels.push(self.opened_namespace_candidates(name));
            levels.push(self.suffix_candidates(&name.parts));
            levels
        }
    }

    fn lookup_exact_candidates(&self, name: &npa_cert::Name) -> Vec<HumanNameCandidate> {
        let mut local_candidates = BoundedHumanNameCandidates::default();
        for entry in self
            .temporary_globals
            .iter()
            .chain(self.global_scope.current.iter())
            .filter(|entry| name_from_human(&entry.name) == *name)
        {
            local_candidates.insert(candidate_from_entry(entry));
        }

        if !local_candidates.is_empty() {
            return local_candidates.into_vec();
        }

        let mut imported_candidates = BoundedHumanNameCandidates::default();
        for entry in self
            .global_scope
            .imported
            .iter()
            .filter(|entry| name_from_human(&entry.name) == *name)
        {
            imported_candidates.insert(candidate_from_entry(entry));
        }
        imported_candidates.into_vec()
    }

    fn opened_namespace_candidates(&self, name: &HumanName) -> Vec<HumanNameCandidate> {
        let mut candidates = BoundedHumanNameCandidates::default();
        for frame in &self.state.open_scopes {
            for open in &frame.opens {
                let mut parts = open.namespace.parts.clone();
                parts.extend(name.parts.iter().cloned());
                let full_name = npa_cert::Name(parts);
                candidates.extend(self.lookup_exact_candidates(&full_name));
            }
        }

        candidates.into_vec()
    }

    fn short_name_candidates(&self, short_name: &str) -> Vec<HumanNameCandidate> {
        let mut local_candidates = BoundedHumanNameCandidates::default();
        for entry in self
            .temporary_globals
            .iter()
            .chain(self.global_scope.current.iter())
        {
            if entry
                .name
                .parts
                .last()
                .is_some_and(|part| part == short_name)
            {
                local_candidates.insert(candidate_from_entry(entry));
            }
        }

        if !local_candidates.is_empty() {
            return local_candidates.into_vec();
        }

        let mut imported_candidates = BoundedHumanNameCandidates::default();
        for entry in &self.global_scope.imported {
            if entry
                .name
                .parts
                .last()
                .is_some_and(|part| part == short_name)
            {
                imported_candidates.insert(candidate_from_entry(entry));
            }
        }

        imported_candidates.into_vec()
    }

    fn suffix_candidates(&self, suffix: &[String]) -> Vec<HumanNameCandidate> {
        let mut local_candidates = BoundedHumanNameCandidates::default();
        for entry in self
            .temporary_globals
            .iter()
            .chain(self.global_scope.current.iter())
            .filter(|entry| name_has_suffix(&entry.name.parts, suffix))
        {
            local_candidates.insert(candidate_from_entry(entry));
        }

        if !local_candidates.is_empty() {
            return local_candidates.into_vec();
        }

        let mut imported_candidates = BoundedHumanNameCandidates::default();
        for entry in self
            .global_scope
            .imported
            .iter()
            .filter(|entry| name_has_suffix(&entry.name.parts, suffix))
        {
            imported_candidates.insert(candidate_from_entry(entry));
        }

        imported_candidates.into_vec()
    }

    fn forward_reference_candidates(&self, name: &HumanName) -> Vec<String> {
        let mut candidates = BoundedStrings::default();
        if name.parts.len() == 1 {
            let current = self.relative_to_current_namespace(name);
            if self.pending_current_names.contains(&current) {
                candidates.insert(current.as_dotted());
            }
            for frame in &self.state.open_scopes {
                for open in &frame.opens {
                    let mut parts = open.namespace.parts.clone();
                    parts.extend(name.parts.iter().cloned());
                    let opened = npa_cert::Name(parts);
                    if self.pending_current_names.contains(&opened) {
                        candidates.insert(opened.as_dotted());
                    }
                }
            }
            for candidate in &self.pending_current_names {
                if candidate
                    .0
                    .last()
                    .is_some_and(|part| part == &name.parts[0])
                {
                    candidates.insert(candidate.as_dotted());
                }
            }
        } else {
            let exact = name_from_human(name);
            if self.pending_current_names.contains(&exact) {
                candidates.insert(exact.as_dotted());
            }
            let current = self.relative_to_current_namespace(name);
            if self.pending_current_names.contains(&current) {
                candidates.insert(current.as_dotted());
            }
            for frame in &self.state.open_scopes {
                for open in &frame.opens {
                    let mut parts = open.namespace.parts.clone();
                    parts.extend(name.parts.iter().cloned());
                    let opened = npa_cert::Name(parts);
                    if self.pending_current_names.contains(&opened) {
                        candidates.insert(opened.as_dotted());
                    }
                }
            }
            for candidate in &self.pending_current_names {
                if name_has_suffix(&candidate.0, &name.parts) {
                    candidates.insert(candidate.as_dotted());
                }
            }
        }

        candidates.into_vec()
    }

    fn namespace_candidates(&self, name: &HumanName) -> Vec<npa_cert::Name> {
        for candidates in [
            self.exact_namespace_candidates(&name_from_human(name)),
            self.exact_namespace_candidates(&self.relative_to_current_namespace(name)),
            self.opened_namespace_prefix_candidates(name),
        ] {
            let candidates = dedupe_names(candidates);
            if !candidates.is_empty() {
                return candidates;
            }
        }

        Vec::new()
    }

    fn exact_namespace_candidates(&self, namespace: &npa_cert::Name) -> Vec<npa_cert::Name> {
        let has_local_candidate = self
            .temporary_globals
            .iter()
            .chain(self.global_scope.current.iter())
            .any(|entry| name_has_strict_prefix(&entry.name.parts, &namespace.0));

        if has_local_candidate {
            return vec![namespace.clone()];
        }

        if self
            .global_scope
            .imported
            .iter()
            .any(|entry| name_has_strict_prefix(&entry.name.parts, &namespace.0))
        {
            return vec![namespace.clone()];
        }

        Vec::new()
    }

    fn opened_namespace_prefix_candidates(&self, name: &HumanName) -> Vec<npa_cert::Name> {
        let mut candidates = BoundedNames::default();
        for frame in &self.state.open_scopes {
            for open in &frame.opens {
                let mut parts = open.namespace.parts.clone();
                parts.extend(name.parts.iter().cloned());
                candidates.extend(self.exact_namespace_candidates(&npa_cert::Name(parts)));
            }
        }
        candidates.into_vec()
    }

    fn relative_to_current_namespace(&self, name: &HumanName) -> npa_cert::Name {
        let mut parts = self.current_namespace_parts();
        parts.extend(name.parts.iter().cloned());
        npa_cert::Name(parts)
    }

    fn ensure_current_name_is_available(&self, name: &HumanName, span: Span) -> HumanResult<()> {
        let full_name = name_from_human(name);
        if self
            .global_scope
            .current
            .iter()
            .chain(self.temporary_globals.iter())
            .any(|entry| name_from_human(&entry.name) == full_name)
        {
            return Err(HumanDiagnostic::error(
                HumanDiagnosticKind::DuplicateDeclaration,
                span,
                format!("duplicate declaration {}", name.as_dotted()),
            ));
        }

        Ok(())
    }

    fn add_current_global(
        &mut self,
        name: HumanName,
        _kind: HumanSourceDeclarationKind,
        span: Span,
    ) -> HumanResult<usize> {
        let index = self.next_local_index();
        let full_name = name_from_human(&name);
        self.pending_current_names.remove(&full_name);
        self.global_scope.current.push(HumanGlobalScopeEntry {
            name,
            reference: HumanGlobalRef::Local {
                index,
                name: full_name,
            },
            span,
        });
        Ok(index)
    }

    fn next_local_index(&self) -> usize {
        self.global_scope
            .current
            .iter()
            .filter(|entry| matches!(entry.reference, HumanGlobalRef::Local { .. }))
            .count()
    }

    fn add_current_generated_global(
        &mut self,
        entry: HumanGeneratedDeclarationMetadata,
        index: usize,
    ) -> HumanResult<()> {
        let full_name = name_from_human(&entry.name);
        self.pending_current_names.remove(&full_name);
        self.global_scope.current.push(HumanGlobalScopeEntry {
            name: entry.name.clone(),
            reference: HumanGlobalRef::LocalGenerated {
                index,
                name: full_name,
            },
            span: entry.span,
        });
        Ok(())
    }

    fn add_imported_globals(&mut self, import: &VerifiedImport) {
        for export in &import.exports {
            self.global_scope.imported.push(HumanGlobalScopeEntry {
                name: HumanName::new(export.name.0.clone(), Span::empty(crate::FileId(0))),
                reference: HumanGlobalRef::Imported {
                    module: import.module.clone(),
                    name: export.name.clone(),
                    decl_interface_hash: export.decl_interface_hash,
                },
                span: Span::empty(crate::FileId(0)),
            });
        }
    }

    fn generated_inductive_entries(
        &self,
        decl: &HumanInductiveDecl,
        parent: &HumanName,
    ) -> Vec<HumanGeneratedDeclarationMetadata> {
        decl.constructors
            .iter()
            .map(|constructor| HumanGeneratedDeclarationMetadata {
                kind: HumanGeneratedDeclarationKind::Constructor,
                parent: parent.clone(),
                name: relative_child_name(parent, &constructor.name),
                decl_interface_hash: None,
                span: constructor.span,
            })
            .chain(std::iter::once(HumanGeneratedDeclarationMetadata {
                kind: HumanGeneratedDeclarationKind::Recursor,
                parent: parent.clone(),
                name: generated_recursor_name(parent),
                decl_interface_hash: None,
                span: decl.span,
            }))
            .collect()
    }

    fn resolve_notation_target(
        &self,
        decl: &crate::HumanNotationDecl,
    ) -> HumanResult<HumanGlobalRef> {
        match self.resolve_global_name(&decl.target)? {
            Some(target) => Ok(target),
            None => {
                let forward_candidates = self.forward_reference_candidates(&decl.target);
                if !forward_candidates.is_empty() {
                    return Err(HumanDiagnostic::error(
                        HumanDiagnosticKind::ForwardReference,
                        decl.target.span,
                        format!(
                            "notation target {} refers to a later declaration",
                            decl.target.as_dotted()
                        ),
                    )
                    .with_payload(candidate_payload(forward_candidates)));
                }
                Err(HumanDiagnostic::error(
                    HumanDiagnosticKind::UnknownIdentifier,
                    decl.target.span,
                    format!("unknown notation target {}", decl.target.as_dotted()),
                ))
            }
        }
    }

    fn register_notation_entry(
        &mut self,
        decl: &crate::HumanNotationDecl,
        target: HumanGlobalRef,
        metadata: &HumanSourceNotationMetadata,
    ) -> HumanResult<()> {
        if notation_fixity(decl.kind).is_none() {
            return Ok(());
        }
        let entry = HumanResolvedNotationEntry {
            kind: decl.kind,
            associativity: metadata.associativity,
            precedence: decl.precedence,
            token: decl.token.clone(),
            target,
            namespace: metadata.namespace.clone(),
            span: decl.span,
        };
        self.ensure_notation_compatible(&entry)?;
        self.current_notation_scope().entries.push(entry.clone());
        self.namespace_notations
            .entry(entry.namespace.clone())
            .or_default()
            .push(entry.clone());
        self.notation_table.push(entry);
        Ok(())
    }

    fn ensure_notation_compatible(&self, entry: &HumanResolvedNotationEntry) -> HumanResult<()> {
        let Some(fixity) = notation_fixity(entry.kind) else {
            return Ok(());
        };
        for existing in self.active_notation_entries(&entry.token, fixity) {
            if existing.precedence != entry.precedence
                || existing.associativity != entry.associativity
            {
                return Err(HumanDiagnostic::error(
                    HumanDiagnosticKind::NotationConflict,
                    entry.span,
                    format!("conflicting notation declaration for {}", entry.token),
                )
                .with_payload(candidate_payload(vec![
                    resolved_notation_sort_key(&existing),
                    resolved_notation_sort_key(entry),
                ])));
            }
        }

        Ok(())
    }

    fn activate_open_notations(&mut self, namespace: &HumanName) -> HumanResult<()> {
        if let Some(entries) = self.namespace_notations.get(&namespace.parts).cloned() {
            for entry in &entries {
                self.ensure_notation_compatible(entry)?;
            }
            self.current_notation_scope().entries.extend(entries);
        }
        Ok(())
    }

    fn active_notation_entries(
        &self,
        token: &str,
        fixity: HumanNotationFixity,
    ) -> Vec<HumanResolvedNotationEntry> {
        let mut entries: Vec<_> = self
            .notation_scopes
            .iter()
            .flat_map(|scope| scope.entries.iter())
            .filter(|entry| entry.token == token && notation_fixity(entry.kind) == Some(fixity))
            .cloned()
            .collect();
        entries.sort_by_key(resolved_notation_sort_key);
        entries.dedup_by_key(|entry| resolved_notation_sort_key(entry));
        entries
    }

    fn resolve_item(&mut self, item: &HumanItem) -> HumanResult<()> {
        match item {
            HumanItem::Import { .. } => {}
            HumanItem::Open { namespace, span } => {
                let namespace = self.resolve_namespace(namespace)?;
                let open = HumanOpenScope {
                    namespace: namespace.clone(),
                    span: *span,
                };
                self.current_open_frame().opens.push(open);
                self.activate_open_notations(&namespace)?;
            }
            HumanItem::NamespaceStart { name, .. } => {
                let namespace = self.qualify_name(name);
                self.state.namespace_stack.push(name.clone());
                self.state.open_scopes.push(HumanOpenScopeFrame {
                    namespace: Some(namespace),
                    opens: Vec::new(),
                });
                self.notation_scopes.push(HumanNotationScope::default());
            }
            HumanItem::NamespaceEnd { name, span } => {
                self.close_namespace(name.as_ref(), *span)?;
                if self.notation_scopes.len() > 1 {
                    self.notation_scopes.pop();
                }
            }
            HumanItem::Def(decl) => {
                let name = self.qualify_name(&decl.name);
                self.ensure_current_name_is_available(&name, decl.span)?;
                self.resolve_decl_terms(decl)?;
                let metadata = self.decl_metadata(HumanSourceDeclarationKind::Def, decl);
                self.add_current_global(name, HumanSourceDeclarationKind::Def, decl.span)?;
                self.state
                    .source_interfaces
                    .current
                    .declarations
                    .push(metadata);
            }
            HumanItem::Theorem(decl) => {
                let name = self.qualify_name(&decl.name);
                self.ensure_current_name_is_available(&name, decl.span)?;
                self.resolve_decl_terms(decl)?;
                let metadata = self.decl_metadata(HumanSourceDeclarationKind::Theorem, decl);
                self.add_current_global(name, HumanSourceDeclarationKind::Theorem, decl.span)?;
                self.state
                    .source_interfaces
                    .current
                    .declarations
                    .push(metadata);
            }
            HumanItem::Axiom(decl) => {
                let name = self.qualify_name(&decl.name);
                self.ensure_current_name_is_available(&name, decl.span)?;
                self.resolve_axiom_terms(decl)?;
                let metadata = self.axiom_metadata(decl);
                self.add_current_global(name, HumanSourceDeclarationKind::Axiom, decl.span)?;
                self.state
                    .source_interfaces
                    .current
                    .declarations
                    .push(metadata);
            }
            HumanItem::Inductive(decl) => {
                let name = self.qualify_name(&decl.name);
                let generated = self.generated_inductive_entries(decl, &name);
                self.ensure_current_name_is_available(&name, decl.span)?;
                let mut generated_names = BTreeSet::new();
                for generated_entry in &generated {
                    if !generated_names.insert(name_from_human(&generated_entry.name)) {
                        return Err(HumanDiagnostic::error(
                            HumanDiagnosticKind::DuplicateDeclaration,
                            generated_entry.span,
                            format!("duplicate declaration {}", generated_entry.name.as_dotted()),
                        ));
                    }
                    self.ensure_current_name_is_available(
                        &generated_entry.name,
                        generated_entry.span,
                    )?;
                }
                let index = self.next_local_index();
                self.resolve_inductive_terms(decl, &name, index)?;
                let metadata = self.inductive_metadata(decl);
                let added_index = self.add_current_global(
                    name,
                    HumanSourceDeclarationKind::Inductive,
                    decl.span,
                )?;
                debug_assert_eq!(added_index, index);
                for generated_entry in generated {
                    self.add_current_generated_global(generated_entry, index)?;
                }
                self.state
                    .source_interfaces
                    .current
                    .declarations
                    .push(metadata);
                self.record_generated_inductive_metadata(decl);
            }
            HumanItem::Notation(decl) => {
                let target = self.resolve_notation_target(decl)?;
                let metadata = HumanSourceNotationMetadata {
                    kind: decl.kind,
                    associativity: notation_associativity(decl.kind),
                    precedence: decl.precedence,
                    token: decl.token.clone(),
                    target: decl.target.clone(),
                    namespace: self.current_namespace_parts(),
                    span: decl.span,
                };
                self.register_notation_entry(decl, target, &metadata)?;
                self.state.notation_table.push(metadata.clone());
                self.state
                    .source_interfaces
                    .current
                    .notations
                    .push(metadata);
            }
        }

        Ok(())
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

    fn current_notation_scope(&mut self) -> &mut HumanNotationScope {
        if self.notation_scopes.is_empty() {
            self.notation_scopes.push(HumanNotationScope::default());
        }
        self.notation_scopes
            .last_mut()
            .expect("notation scope stack has a top-level frame")
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

#[derive(Clone, Debug, Default)]
struct HumanLocalScope {
    names: Vec<HumanName>,
}

impl HumanLocalScope {
    fn push(&mut self, name: HumanName) {
        self.names.push(name);
    }

    fn lookup(&self, name: &str) -> Option<(HumanName, usize)> {
        self.names
            .iter()
            .rev()
            .enumerate()
            .find(|(_, local)| local.parts.len() == 1 && local.parts[0] == name)
            .map(|(index, local)| (local.clone(), index))
    }
}

#[derive(Clone, Debug, Default)]
struct HumanNotationScope {
    entries: Vec<HumanResolvedNotationEntry>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HumanNotationFixity {
    Prefix,
    Postfix,
    Infix,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HumanNameCandidate {
    key: String,
    reference: HumanGlobalRef,
}

fn candidate_from_entry(entry: &HumanGlobalScopeEntry) -> HumanNameCandidate {
    HumanNameCandidate {
        key: global_ref_sort_key(&entry.reference),
        reference: entry.reference.clone(),
    }
}

#[derive(Default)]
struct BoundedHumanNameCandidates {
    candidates: BTreeMap<String, HumanNameCandidate>,
}

impl BoundedHumanNameCandidates {
    fn insert(&mut self, candidate: HumanNameCandidate) {
        if self.candidates.contains_key(&candidate.key) {
            return;
        }
        self.candidates.insert(candidate.key.clone(), candidate);
        self.trim();
    }

    fn extend<I>(&mut self, candidates: I)
    where
        I: IntoIterator<Item = HumanNameCandidate>,
    {
        for candidate in candidates {
            self.insert(candidate);
        }
    }

    fn is_empty(&self) -> bool {
        self.candidates.is_empty()
    }

    fn into_vec(self) -> Vec<HumanNameCandidate> {
        self.candidates.into_values().collect()
    }

    fn trim(&mut self) {
        if self.candidates.len() <= MAX_HUMAN_NAME_CANDIDATES {
            return;
        }
        if let Some(last_key) = self.candidates.keys().next_back().cloned() {
            self.candidates.remove(&last_key);
        }
    }
}

#[derive(Default)]
struct BoundedNames {
    names: BTreeSet<npa_cert::Name>,
}

impl BoundedNames {
    fn insert(&mut self, name: npa_cert::Name) {
        self.names.insert(name);
        self.trim();
    }

    fn extend<I>(&mut self, names: I)
    where
        I: IntoIterator<Item = npa_cert::Name>,
    {
        for name in names {
            self.insert(name);
        }
    }

    fn into_vec(self) -> Vec<npa_cert::Name> {
        self.names.into_iter().collect()
    }

    fn trim(&mut self) {
        if self.names.len() <= MAX_HUMAN_NAME_CANDIDATES {
            return;
        }
        if let Some(last) = self.names.iter().next_back().cloned() {
            self.names.remove(&last);
        }
    }
}

#[derive(Default)]
struct BoundedStrings {
    strings: BTreeSet<String>,
}

impl BoundedStrings {
    fn insert(&mut self, value: String) {
        self.strings.insert(value);
        self.trim();
    }

    fn into_vec(self) -> Vec<String> {
        self.strings.into_iter().collect()
    }

    fn trim(&mut self) {
        if self.strings.len() <= MAX_HUMAN_NAME_CANDIDATES {
            return;
        }
        if let Some(last) = self.strings.iter().next_back().cloned() {
            self.strings.remove(&last);
        }
    }
}

fn dedupe_and_sort_candidates(candidates: Vec<HumanNameCandidate>) -> Vec<HumanNameCandidate> {
    let mut bounded = BoundedHumanNameCandidates::default();
    for candidate in candidates {
        bounded.insert(candidate);
    }
    bounded.into_vec()
}

fn dedupe_names(names: Vec<npa_cert::Name>) -> Vec<npa_cert::Name> {
    let mut names: Vec<_> = names
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    names.truncate(MAX_HUMAN_NAME_CANDIDATES);
    names
}

fn candidate_payload(mut candidates: Vec<String>) -> HumanDiagnosticPayload {
    let mut bounded = BoundedStrings::default();
    for candidate in candidates.drain(..) {
        bounded.insert(candidate);
    }
    HumanDiagnosticPayload {
        candidates: bounded.into_vec(),
        ..HumanDiagnosticPayload::default()
    }
}

fn global_ref_sort_key(reference: &HumanGlobalRef) -> String {
    match reference {
        HumanGlobalRef::Imported {
            module,
            name,
            decl_interface_hash,
        } => format!(
            "imported:{}:{}:{}",
            module.as_dotted(),
            name.as_dotted(),
            hash_hex(decl_interface_hash)
        ),
        HumanGlobalRef::Local { index, name } => {
            format!("local:{index:08}:{}", name.as_dotted())
        }
        HumanGlobalRef::LocalGenerated { index, name } => {
            format!("local-generated:{index:08}:{}", name.as_dotted())
        }
    }
}

fn notation_fixity(kind: HumanNotationKind) -> Option<HumanNotationFixity> {
    match kind {
        HumanNotationKind::Notation => None,
        HumanNotationKind::Prefix => Some(HumanNotationFixity::Prefix),
        HumanNotationKind::Postfix => Some(HumanNotationFixity::Postfix),
        HumanNotationKind::Infix | HumanNotationKind::Infixl | HumanNotationKind::Infixr => {
            Some(HumanNotationFixity::Infix)
        }
    }
}

fn notation_associativity(kind: HumanNotationKind) -> HumanNotationAssociativity {
    match kind {
        HumanNotationKind::Infixl => HumanNotationAssociativity::Left,
        HumanNotationKind::Infixr => HumanNotationAssociativity::Right,
        HumanNotationKind::Notation
        | HumanNotationKind::Prefix
        | HumanNotationKind::Postfix
        | HumanNotationKind::Infix => HumanNotationAssociativity::NonAssoc,
    }
}

fn resolved_notation_sort_key(entry: &HumanResolvedNotationEntry) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        entry.token,
        notation_kind_sort_key(entry.kind),
        entry.precedence,
        notation_associativity_sort_key(entry.associativity),
        global_ref_sort_key(&entry.target)
    )
}

fn notation_kind_sort_key(kind: HumanNotationKind) -> u8 {
    match kind {
        HumanNotationKind::Notation => 0,
        HumanNotationKind::Prefix => 1,
        HumanNotationKind::Postfix => 2,
        HumanNotationKind::Infix => 3,
        HumanNotationKind::Infixl => 4,
        HumanNotationKind::Infixr => 5,
    }
}

fn notation_associativity_sort_key(associativity: HumanNotationAssociativity) -> u8 {
    match associativity {
        HumanNotationAssociativity::NonAssoc => 0,
        HumanNotationAssociativity::Left => 1,
        HumanNotationAssociativity::Right => 2,
    }
}

fn hash_hex(hash: &npa_cert::Hash) -> String {
    hash.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn name_has_suffix(name: &[String], suffix: &[String]) -> bool {
    name.len() >= suffix.len() && &name[(name.len() - suffix.len())..] == suffix
}

fn name_has_strict_prefix(name: &[String], prefix: &[String]) -> bool {
    name.len() > prefix.len() && name.starts_with(prefix)
}

fn planned_current_names(module: &HumanModule) -> BTreeSet<npa_cert::Name> {
    let mut names = BTreeSet::new();
    let mut namespace_stack: Vec<HumanName> = Vec::new();

    for item in &module.items {
        match item {
            HumanItem::NamespaceStart { name, .. } => {
                namespace_stack.push(name.clone());
            }
            HumanItem::NamespaceEnd { .. } => {
                namespace_stack.pop();
            }
            HumanItem::Def(decl) | HumanItem::Theorem(decl) => {
                names.insert(name_from_parts(&namespace_stack, &decl.name));
            }
            HumanItem::Axiom(decl) => {
                names.insert(name_from_parts(&namespace_stack, &decl.name));
            }
            HumanItem::Inductive(decl) => {
                let parent = HumanName::new(
                    name_from_parts(&namespace_stack, &decl.name).0,
                    decl.name.span,
                );
                names.insert(name_from_human(&parent));
                for constructor in &decl.constructors {
                    names.insert(name_from_human(&relative_child_name(
                        &parent,
                        &constructor.name,
                    )));
                }
                names.insert(name_from_human(&generated_recursor_name(&parent)));
            }
            HumanItem::Import { .. } | HumanItem::Open { .. } | HumanItem::Notation(_) => {}
        }
    }

    names
}

fn name_from_parts(namespace_stack: &[HumanName], name: &HumanName) -> npa_cert::Name {
    let mut parts = namespace_stack
        .iter()
        .flat_map(|namespace| namespace.parts.iter().cloned())
        .collect::<Vec<_>>();
    parts.extend(name.parts.iter().cloned());
    npa_cert::Name(parts)
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
    use crate::{
        parse_human_module, HumanBinderInfo, HumanDiagnosticKind, HumanNotationAssociativity,
        HumanNotationKind,
    };

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
        resolve_source_with_options(source, imports, &crate::HumanCompileOptions::default())
    }

    fn resolve_source_with_options(
        source: &str,
        imports: &[VerifiedImport],
        options: &crate::HumanCompileOptions,
    ) -> HumanResult<ResolvedHumanModule> {
        let module = parse_human_module(crate::FileId(0), source).expect("source should parse");
        resolve_human_module(
            npa_cert::Name::from_dotted("Current.Module"),
            module,
            imports,
            options,
        )
    }

    #[test]
    fn human_frontend_state_records_module_namespace_and_lexical_open_scopes() {
        let import = verified_import("Std.Basic", &["Std.foo", "Nat.zero"]);
        let resolved = resolve_source(
            "\
import Std.Basic
open Std
namespace Demo
open Nat
def id {A : Type} (x : A) : A := x
end Demo",
            &[import],
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
def zero : Type := Type
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

    #[test]
    fn namespace_declaration_registers_fully_qualified_global_name() {
        let resolved = resolve_source(
            "\
namespace Nat
def zero : Type := Type
end Nat",
            &[],
        )
        .expect("namespace declaration should resolve");

        assert_eq!(resolved.global_scope.current.len(), 1);
        assert_eq!(
            resolved.global_scope.current[0].name.as_dotted(),
            "Nat.zero"
        );
        assert_eq!(
            resolved.state.source_interfaces.current.declarations[0]
                .name
                .as_dotted(),
            "Nat.zero"
        );
    }

    #[test]
    fn open_scope_resolves_unqualified_name_to_imported_namespace_member() {
        let import = verified_import("Std.Nat.Basic", &["Std.Nat.zero"]);
        let resolved = resolve_source(
            "\
import Std.Nat.Basic
open Std.Nat
def use_zero : Type := zero",
            &[import],
        )
        .expect("opened namespace member should resolve");

        assert_eq!(resolved.resolved_names.len(), 1);
        assert_eq!(resolved.resolved_names[0].source.as_dotted(), "zero");
        let HumanResolvedName::Global(HumanGlobalRef::Imported { module, name, .. }) =
            &resolved.resolved_names[0].resolved
        else {
            panic!("zero should resolve to imported global");
        };
        assert_eq!(module, &npa_cert::Name::from_dotted("Std.Nat.Basic"));
        assert_eq!(name, &npa_cert::Name::from_dotted("Std.Nat.zero"));
    }

    #[test]
    fn notation_target_resolves_to_global_ref_and_use_records_candidate() {
        let resolved = resolve_source(
            "\
def add (n m : Type) : Type := n
infixl:65 \" + \" => add
def use (n : Type) : Type := n + Type",
            &[],
        )
        .expect("notation target and use should resolve");

        assert_eq!(resolved.notation_table.len(), 1);
        let HumanGlobalRef::Local { name, .. } = &resolved.notation_table[0].target else {
            panic!("notation target should resolve to current local declaration");
        };
        assert_eq!(name, &npa_cert::Name::from_dotted("add"));
        assert_eq!(resolved.resolved_notations.len(), 1);
        assert_eq!(resolved.resolved_notations[0].head.token, "+");
        assert_eq!(resolved.resolved_notations[0].candidates.len(), 1);
        assert_eq!(
            resolved.resolved_notations[0].candidates[0],
            resolved.notation_table[0].target
        );
    }

    #[test]
    fn open_namespace_activates_namespaced_notation() {
        let resolved = resolve_source(
            "\
namespace Nat
def add (n m : Type) : Type := n
infixl:65 \" + \" => add
end Nat
open Nat
def use (n : Type) : Type := n + Type",
            &[],
        )
        .expect("open should activate namespace notation");

        assert_eq!(resolved.resolved_notations.len(), 1);
        let HumanGlobalRef::Local { name, .. } = &resolved.resolved_notations[0].candidates[0]
        else {
            panic!("notation candidate should resolve to current local declaration");
        };
        assert_eq!(name, &npa_cert::Name::from_dotted("Nat.add"));
    }

    #[test]
    fn resolver_rejects_open_namespace_notation_conflict() {
        let imports: Vec<VerifiedImport> = Vec::new();
        let mut resolver = HumanResolver::new(
            npa_cert::Name::from_dotted("Current.Module"),
            &imports,
            &crate::HumanCompileOptions::default(),
        );
        let left = HumanResolvedNotationEntry {
            kind: HumanNotationKind::Infixl,
            associativity: HumanNotationAssociativity::Left,
            precedence: 65,
            token: "+".to_owned(),
            target: HumanGlobalRef::Local {
                index: 0,
                name: npa_cert::Name::from_dotted("A.add"),
            },
            namespace: vec!["A".to_owned()],
            span: Span::empty(crate::FileId(0)),
        };
        let right = HumanResolvedNotationEntry {
            kind: HumanNotationKind::Infixr,
            associativity: HumanNotationAssociativity::Right,
            precedence: 70,
            token: "+".to_owned(),
            target: HumanGlobalRef::Local {
                index: 1,
                name: npa_cert::Name::from_dotted("B.add"),
            },
            namespace: vec!["B".to_owned()],
            span: Span::empty(crate::FileId(0)),
        };

        resolver.current_notation_scope().entries.push(left);
        resolver
            .namespace_notations
            .insert(vec!["B".to_owned()], vec![right]);
        let err = resolver
            .activate_open_notations(&HumanName::new(
                vec!["B".to_owned()],
                Span::empty(crate::FileId(0)),
            ))
            .expect_err("open should reject active notation conflicts");

        assert_eq!(err.kind, HumanDiagnosticKind::NotationConflict);
    }

    #[test]
    fn overloaded_notation_candidates_are_deterministically_sorted() {
        let resolved = resolve_source(
            "\
def add_a (n m : Type) : Type := n
def add_b (n m : Type) : Type := m
infixl:65 \" + \" => add_b
infixl:65 \" + \" => add_a
def use (n : Type) : Type := n + Type",
            &[],
        )
        .expect("overloaded notation should resolve to a bounded candidate set");

        assert_eq!(resolved.resolved_notations.len(), 1);
        assert_eq!(resolved.resolved_notations[0].candidates.len(), 2);
        let names: Vec<_> = resolved.resolved_notations[0]
            .candidates
            .iter()
            .map(|candidate| match candidate {
                HumanGlobalRef::Local { name, .. } => name.as_dotted(),
                other => panic!("unexpected notation candidate: {other:?}"),
            })
            .collect();
        assert_eq!(names, vec!["add_a".to_owned(), "add_b".to_owned()]);
    }

    #[test]
    fn too_many_notation_candidates_is_rejected() {
        let options = crate::HumanCompileOptions {
            max_notation_candidates: 1,
        };
        let err = resolve_source_with_options(
            "\
def add_a (n m : Type) : Type := n
def add_b (n m : Type) : Type := m
infixl:65 \" + \" => add_a
infixl:65 \" + \" => add_b
def use (n : Type) : Type := n + Type",
            &[],
            &options,
        )
        .expect_err("candidate count above the configured limit should fail");

        assert_eq!(err.kind, HumanDiagnosticKind::TooManyNotationCandidates);
        let payload = err
            .payload
            .expect("too many candidates should carry a bounded payload");
        assert_eq!(payload.candidates.len(), 1);
    }

    #[test]
    fn ambiguous_unqualified_name_returns_deterministic_payload() {
        let left = verified_import("Std.Nat.Basic", &["Std.Nat.zero"]);
        let right = verified_import("Other.Nat.Basic", &["Other.Nat.zero"]);
        let err = resolve_source(
            "\
import Std.Nat.Basic
import Other.Nat.Basic
def use_zero : Type := zero",
            &[left, right],
        )
        .expect_err("ambiguous short name should fail");

        assert_eq!(err.kind, HumanDiagnosticKind::AmbiguousName);
        let payload = err.payload.expect("ambiguous name should carry candidates");
        assert_eq!(payload.phase, Some(HumanDiagnosticPhase::Resolver));
        assert_eq!(payload.candidates.len(), 2);
        assert!(payload.candidates[0].contains("Other.Nat.zero"));
        assert!(payload.candidates[1].contains("Std.Nat.zero"));
    }

    #[test]
    fn ambiguous_name_payload_is_bounded_and_deterministically_sorted() {
        let import_specs: Vec<_> = (0..40)
            .map(|index| (format!("M{index:02}"), format!("M{index:02}.zero")))
            .collect();
        let imports: Vec<_> = import_specs
            .iter()
            .map(|(module, export)| verified_import(module, &[export.as_str()]))
            .collect();
        let mut source = String::new();
        for (module, _) in &import_specs {
            source.push_str(&format!("import {module}\n"));
        }
        source.push_str("def use_zero : Type := zero");

        let err = resolve_source(&source, &imports).expect_err("ambiguous short name should fail");

        assert_eq!(err.kind, HumanDiagnosticKind::AmbiguousName);
        let payload = err.payload.expect("ambiguous name should carry candidates");
        assert_eq!(payload.candidates.len(), MAX_HUMAN_NAME_CANDIDATES);
        assert!(payload.candidates[0].starts_with("imported:M00:M00.zero:"));
        assert!(payload.candidates[31].starts_with("imported:M31:M31.zero:"));
    }

    #[test]
    fn forward_reference_is_rejected_before_later_declaration_is_registered() {
        let err = resolve_source(
            "\
def first : Type := later
def later : Type := Type",
            &[],
        )
        .expect_err("forward reference should fail");

        assert_eq!(err.kind, HumanDiagnosticKind::ForwardReference);
        let payload = err
            .payload
            .expect("forward reference should identify later declaration");
        assert_eq!(payload.phase, Some(HumanDiagnosticPhase::Resolver));
        assert_eq!(payload.candidates, vec!["later".to_owned()]);
    }

    #[test]
    fn current_declaration_wins_over_imported_short_name() {
        let import = verified_import("Std.Basic", &["zero"]);
        let resolved = resolve_source(
            "\
import Std.Basic
def zero : Type := Type
def use_zero : Type := zero",
            &[import],
        )
        .expect("current declaration should shadow imported declaration");

        let HumanResolvedName::Global(HumanGlobalRef::Local { name, .. }) =
            &resolved.resolved_names[0].resolved
        else {
            panic!("zero should resolve to current module global");
        };
        assert_eq!(name, &npa_cert::Name::from_dotted("zero"));
    }

    #[test]
    fn local_context_is_separate_and_shadows_global_names() {
        let import = verified_import("Std.Basic", &["Nat"]);
        let resolved = resolve_source(
            "\
import Std.Basic
def id (Nat : Type) (x : Nat) : Nat := x",
            &[import],
        )
        .expect("local names should resolve independently from globals");

        assert!(matches!(
            resolved.resolved_names[0].resolved,
            HumanResolvedName::Local { .. }
        ));
        assert!(matches!(
            resolved.resolved_names[1].resolved,
            HumanResolvedName::Local { .. }
        ));
        assert!(matches!(
            resolved.resolved_names[2].resolved,
            HumanResolvedName::Local { .. }
        ));
    }

    #[test]
    fn unknown_open_namespace_is_rejected() {
        let err = resolve_source("open Missing", &[]).expect_err("unknown namespace should fail");

        assert_eq!(err.kind, HumanDiagnosticKind::UnknownNamespace);
    }

    #[test]
    fn open_requires_exact_visible_namespace_prefix_not_suffix_only() {
        let import = verified_import("Std.Nat.Basic", &["Std.Nat.zero"]);
        let err = resolve_source(
            "\
import Std.Nat.Basic
open Nat",
            &[import],
        )
        .expect_err("suffix-only namespace should not be opened");

        assert_eq!(err.kind, HumanDiagnosticKind::UnknownNamespace);
    }
}
