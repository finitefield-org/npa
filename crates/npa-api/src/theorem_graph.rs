use std::collections::{BTreeMap, BTreeSet};

use npa_cert::{
    decode_module_cert, term_hash, AxiomRef, CertReducibility, DeclCert, DeclPayload, ExportEntry,
    ExportKind, GlobalRef, Hash, ModuleCert, Name, NameId, TermId, TermNode, VerifiedModule,
};
use sha2::{Digest, Sha256};

const CERTIFICATE_THEOREM_GRAPH_HASH_TAG: &[u8] = b"npa.certificate-theorem-graph.v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CertificateTheoremGraphOptions {
    pub require_import_certificate_hashes: bool,
}

impl CertificateTheoremGraphOptions {
    pub fn export_hash_bound() -> Self {
        Self {
            require_import_certificate_hashes: false,
        }
    }

    pub fn high_trust_certificate_hash_bound() -> Self {
        Self {
            require_import_certificate_hashes: true,
        }
    }
}

impl Default for CertificateTheoremGraphOptions {
    fn default() -> Self {
        Self::export_hash_bound()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CertificateTheoremGraphError {
    DecodeCertificate,
    MissingImportBinding { module: Name },
    DuplicateImportBinding { module: Name },
    ImportExportHashMismatch { module: Name },
    ImportCertificateHashMissing { module: Name },
    ImportCertificateHashMismatch { module: Name },
    MissingName { name_id: NameId },
    MissingTerm { term_id: TermId },
    MissingDeclaration { decl_index: usize },
    MissingExport { name: Name },
    MissingImportedExport { module: Name, name: Name },
    DeclInterfaceHashMismatch { name: Name },
    TermHash { term_id: TermId },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CertificateTheoremGraphSnapshot {
    pub source_module: Name,
    pub source_export_hash: Hash,
    pub source_certificate_hash: Hash,
    pub extractor_version: CertificateTheoremGraphExtractorVersion,
    pub imports: Vec<CertificateTheoremGraphImportBinding>,
    pub nodes: Vec<CertificateTheoremGraphNode>,
    pub edges: Vec<CertificateTheoremGraphEdge>,
    pub graph_hash: Hash,
}

impl CertificateTheoremGraphSnapshot {
    pub fn node(&self, id: &CertificateTheoremGraphNodeId) -> Option<&CertificateTheoremGraphNode> {
        self.nodes.iter().find(|node| &node.id == id)
    }

    pub fn outgoing_edges(
        &self,
        id: &CertificateTheoremGraphNodeId,
    ) -> Vec<CertificateTheoremGraphEdge> {
        self.edges
            .iter()
            .filter(|edge| &edge.from == id)
            .cloned()
            .collect()
    }

    pub fn direct_axiom_dependencies(
        &self,
        id: &CertificateTheoremGraphNodeId,
    ) -> Vec<CertificateTheoremGraphNodeId> {
        self.edge_targets(id, CertificateTheoremGraphEdgeKind::DependsOnDirectAxiom)
    }

    pub fn transitive_axiom_dependencies(
        &self,
        id: &CertificateTheoremGraphNodeId,
    ) -> Vec<CertificateTheoremGraphNodeId> {
        self.edge_targets(
            id,
            CertificateTheoremGraphEdgeKind::DependsOnTransitiveAxiom,
        )
    }

    pub fn direct_dependency_targets(
        &self,
        id: &CertificateTheoremGraphNodeId,
    ) -> Vec<CertificateTheoremGraphNodeId> {
        self.edge_targets(id, CertificateTheoremGraphEdgeKind::ImportsDeclaration)
    }

    fn edge_targets(
        &self,
        id: &CertificateTheoremGraphNodeId,
        kind: CertificateTheoremGraphEdgeKind,
    ) -> Vec<CertificateTheoremGraphNodeId> {
        self.edges
            .iter()
            .filter(|edge| &edge.from == id && edge.kind == kind)
            .map(|edge| edge.to.clone())
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CertificateTheoremGraphImportBinding {
    pub module: Name,
    pub export_hash: Hash,
    pub certificate_hash: Option<Hash>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CertificateTheoremGraphExtractorVersion {
    CertificateGraphV1,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CertificateTheoremGraphNodeId {
    pub scope: CertificateTheoremGraphNodeScope,
    pub module: Name,
    pub name: Name,
    pub decl_interface_hash: Hash,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CertificateTheoremGraphNodeScope {
    Builtin,
    Imported {
        import_export_hash: Hash,
        import_certificate_hash: Option<Hash>,
    },
    Local,
    LocalGenerated {
        source_decl_index: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CertificateTheoremGraphNode {
    pub id: CertificateTheoremGraphNodeId,
    pub kind: CertificateTheoremGraphNodeKind,
    pub type_hash: Option<Hash>,
    pub proof_hash: Option<Hash>,
    pub body_hash: Option<Hash>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CertificateTheoremGraphNodeKind {
    Axiom,
    Definition,
    Theorem,
    Inductive,
    Constructor,
    Recursor,
    Builtin,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CertificateTheoremGraphEdge {
    pub from: CertificateTheoremGraphNodeId,
    pub to: CertificateTheoremGraphNodeId,
    pub kind: CertificateTheoremGraphEdgeKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CertificateTheoremGraphEdgeKind {
    ImportsDeclaration,
    MentionsType,
    UsesConstant,
    GeneratedDeclaration,
    DependsOnDirectAxiom,
    DependsOnTransitiveAxiom,
}

pub fn extract_certificate_theorem_graph(
    certificate_bytes: &[u8],
    imports: &[VerifiedModule],
    options: CertificateTheoremGraphOptions,
) -> Result<CertificateTheoremGraphSnapshot, CertificateTheoremGraphError> {
    let cert = decode_module_cert(certificate_bytes)
        .map_err(|_| CertificateTheoremGraphError::DecodeCertificate)?;
    extract_certificate_theorem_graph_from_cert(&cert, imports, options)
}

pub fn extract_certificate_theorem_graph_from_cert(
    cert: &ModuleCert,
    imports: &[VerifiedModule],
    options: CertificateTheoremGraphOptions,
) -> Result<CertificateTheoremGraphSnapshot, CertificateTheoremGraphError> {
    let import_modules = validate_import_bindings(cert, imports, options)?;
    let export_by_name = export_entries_by_name(cert)?;
    let direct_axioms_by_decl = direct_axioms_by_decl(cert);
    let transitive_axioms_by_decl = transitive_axioms_by_decl(cert);
    let mut state = GraphExtractionState {
        cert,
        export_by_name,
        import_modules,
        nodes: BTreeMap::new(),
        edges: BTreeSet::new(),
    };

    for (decl_index, decl) in cert.declarations.iter().enumerate() {
        let source = local_decl_node_id(cert, decl_index)?;
        insert_node(
            &mut state.nodes,
            local_decl_node(cert, &state.export_by_name, decl_index, decl)?,
        );

        for dependency in &decl.dependencies {
            let target = graph_node_for_global_ref(
                cert,
                &state.export_by_name,
                &state.import_modules,
                &dependency.global_ref,
                Some(dependency.decl_interface_hash),
                None,
            )?;
            insert_node(&mut state.nodes, target.clone());
            state.edges.insert(CertificateTheoremGraphEdge {
                from: source.clone(),
                to: target.id,
                kind: CertificateTheoremGraphEdgeKind::ImportsDeclaration,
            });
        }

        for axiom in direct_axioms_by_decl.get(&decl_index).into_iter().flatten() {
            add_axiom_edge(
                &mut state,
                &source,
                axiom,
                CertificateTheoremGraphEdgeKind::DependsOnDirectAxiom,
            )?;
        }
        for axiom in transitive_axioms_by_decl
            .get(&decl_index)
            .into_iter()
            .flatten()
        {
            add_axiom_edge(
                &mut state,
                &source,
                axiom,
                CertificateTheoremGraphEdgeKind::DependsOnTransitiveAxiom,
            )?;
        }

        add_decl_payload_edges(&mut state, decl_index, decl, &source)?;
    }

    let imports = cert
        .imports
        .iter()
        .map(|import| CertificateTheoremGraphImportBinding {
            module: import.module.clone(),
            export_hash: import.export_hash,
            certificate_hash: import.certificate_hash,
        })
        .collect();
    let mut snapshot = CertificateTheoremGraphSnapshot {
        source_module: cert.header.module.clone(),
        source_export_hash: cert.hashes.export_hash,
        source_certificate_hash: cert.hashes.certificate_hash,
        extractor_version: CertificateTheoremGraphExtractorVersion::CertificateGraphV1,
        imports,
        nodes: state.nodes.into_values().collect(),
        edges: state.edges.into_iter().collect(),
        graph_hash: [0; 32],
    };
    snapshot.graph_hash = certificate_theorem_graph_snapshot_hash(&snapshot);
    Ok(snapshot)
}

pub fn certificate_theorem_graph_snapshot_canonical_bytes(
    snapshot: &CertificateTheoremGraphSnapshot,
) -> Vec<u8> {
    let mut out = Vec::new();
    encode_name(&mut out, &snapshot.source_module);
    out.extend(snapshot.source_export_hash);
    out.extend(snapshot.source_certificate_hash);
    out.push(extractor_version_tag(snapshot.extractor_version));
    encode_uvar(&mut out, snapshot.imports.len() as u64);
    for import in &snapshot.imports {
        encode_name(&mut out, &import.module);
        out.extend(import.export_hash);
        encode_optional_hash(&mut out, import.certificate_hash);
    }
    let mut nodes = snapshot.nodes.clone();
    nodes.sort_by(|lhs, rhs| lhs.id.cmp(&rhs.id));
    encode_uvar(&mut out, nodes.len() as u64);
    for node in &nodes {
        encode_node(&mut out, node);
    }
    let mut edges = snapshot.edges.clone();
    edges.sort();
    encode_uvar(&mut out, edges.len() as u64);
    for edge in &edges {
        encode_node_id(&mut out, &edge.from);
        encode_node_id(&mut out, &edge.to);
        out.push(edge_kind_tag(edge.kind));
    }
    out
}

pub fn certificate_theorem_graph_snapshot_hash(snapshot: &CertificateTheoremGraphSnapshot) -> Hash {
    hash_with_domain(
        CERTIFICATE_THEOREM_GRAPH_HASH_TAG,
        &certificate_theorem_graph_snapshot_canonical_bytes(snapshot),
    )
}

fn validate_import_bindings<'a>(
    cert: &ModuleCert,
    imports: &'a [VerifiedModule],
    options: CertificateTheoremGraphOptions,
) -> Result<BTreeMap<Name, &'a VerifiedModule>, CertificateTheoremGraphError> {
    let mut by_module = BTreeMap::new();
    for import in imports {
        if by_module.insert(import.module().clone(), import).is_some() {
            return Err(CertificateTheoremGraphError::DuplicateImportBinding {
                module: import.module().clone(),
            });
        }
    }
    for required in &cert.imports {
        let Some(verified) = by_module.get(&required.module) else {
            return Err(CertificateTheoremGraphError::MissingImportBinding {
                module: required.module.clone(),
            });
        };
        if verified.export_hash() != required.export_hash {
            return Err(CertificateTheoremGraphError::ImportExportHashMismatch {
                module: required.module.clone(),
            });
        }
        if let Some(certificate_hash) = required.certificate_hash {
            if verified.certificate_hash() != certificate_hash {
                return Err(
                    CertificateTheoremGraphError::ImportCertificateHashMismatch {
                        module: required.module.clone(),
                    },
                );
            }
        } else if options.require_import_certificate_hashes {
            return Err(CertificateTheoremGraphError::ImportCertificateHashMissing {
                module: required.module.clone(),
            });
        }
    }
    Ok(by_module)
}

fn export_entries_by_name(
    cert: &ModuleCert,
) -> Result<BTreeMap<Name, &ExportEntry>, CertificateTheoremGraphError> {
    let mut exports = BTreeMap::new();
    for entry in &cert.export_block {
        exports.insert(name(cert, entry.name)?, entry);
    }
    Ok(exports)
}

fn direct_axioms_by_decl(cert: &ModuleCert) -> BTreeMap<usize, Vec<AxiomRef>> {
    cert.axiom_report
        .per_declaration
        .iter()
        .map(|report| (report.decl_index, report.direct_axioms.clone()))
        .collect()
}

fn transitive_axioms_by_decl(cert: &ModuleCert) -> BTreeMap<usize, Vec<AxiomRef>> {
    cert.axiom_report
        .per_declaration
        .iter()
        .map(|report| (report.decl_index, report.transitive_axioms.clone()))
        .collect()
}

struct GraphExtractionState<'a> {
    cert: &'a ModuleCert,
    export_by_name: BTreeMap<Name, &'a ExportEntry>,
    import_modules: BTreeMap<Name, &'a VerifiedModule>,
    nodes: BTreeMap<CertificateTheoremGraphNodeId, CertificateTheoremGraphNode>,
    edges: BTreeSet<CertificateTheoremGraphEdge>,
}

fn add_decl_payload_edges(
    state: &mut GraphExtractionState<'_>,
    decl_index: usize,
    decl: &DeclCert,
    source: &CertificateTheoremGraphNodeId,
) -> Result<(), CertificateTheoremGraphError> {
    match &decl.decl {
        DeclPayload::Axiom { ty, .. } | DeclPayload::AxiomConstrained { ty, .. } => {
            add_term_edges(
                state,
                source,
                *ty,
                CertificateTheoremGraphEdgeKind::MentionsType,
            )?;
        }
        DeclPayload::Def {
            ty,
            value,
            reducibility,
            ..
        }
        | DeclPayload::DefConstrained {
            ty,
            value,
            reducibility,
            ..
        } => {
            add_term_edges(
                state,
                source,
                *ty,
                CertificateTheoremGraphEdgeKind::MentionsType,
            )?;
            if *reducibility == CertReducibility::Reducible {
                add_term_edges(
                    state,
                    source,
                    *value,
                    CertificateTheoremGraphEdgeKind::UsesConstant,
                )?;
            }
        }
        DeclPayload::Theorem { ty, proof, .. }
        | DeclPayload::TheoremConstrained { ty, proof, .. } => {
            add_term_edges(
                state,
                source,
                *ty,
                CertificateTheoremGraphEdgeKind::MentionsType,
            )?;
            add_term_edges(
                state,
                source,
                *proof,
                CertificateTheoremGraphEdgeKind::UsesConstant,
            )?;
        }
        DeclPayload::Inductive {
            params,
            indices,
            constructors,
            recursor,
            ..
        }
        | DeclPayload::InductiveConstrained {
            params,
            indices,
            constructors,
            recursor,
            ..
        } => {
            for binder in params.iter().chain(indices) {
                add_term_edges(
                    state,
                    source,
                    binder.ty,
                    CertificateTheoremGraphEdgeKind::MentionsType,
                )?;
            }
            for constructor in constructors {
                add_generated_node_and_type_edges(
                    state,
                    decl_index,
                    source,
                    constructor.name,
                    constructor.ty,
                )?;
            }
            if let Some(recursor) = recursor {
                add_generated_node_and_type_edges(
                    state,
                    decl_index,
                    source,
                    recursor.name,
                    recursor.ty,
                )?;
            }
        }
        DeclPayload::MutualInductiveBlock { inductives, .. } => {
            for inductive in inductives {
                for binder in inductive.params.iter().chain(&inductive.indices) {
                    add_term_edges(
                        state,
                        source,
                        binder.ty,
                        CertificateTheoremGraphEdgeKind::MentionsType,
                    )?;
                }
                for constructor in &inductive.constructors {
                    add_generated_node_and_type_edges(
                        state,
                        decl_index,
                        source,
                        constructor.name,
                        constructor.ty,
                    )?;
                }
                if let Some(recursor) = &inductive.recursor {
                    add_generated_node_and_type_edges(
                        state,
                        decl_index,
                        source,
                        recursor.name,
                        recursor.ty,
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn add_generated_node_and_type_edges(
    state: &mut GraphExtractionState<'_>,
    decl_index: usize,
    source: &CertificateTheoremGraphNodeId,
    generated_name: NameId,
    generated_ty: TermId,
) -> Result<(), CertificateTheoremGraphError> {
    let generated = local_generated_node(
        state.cert,
        &state.export_by_name,
        decl_index,
        generated_name,
    )?;
    state.edges.insert(CertificateTheoremGraphEdge {
        from: source.clone(),
        to: generated.id.clone(),
        kind: CertificateTheoremGraphEdgeKind::GeneratedDeclaration,
    });
    insert_node(&mut state.nodes, generated.clone());
    add_term_edges(
        state,
        &generated.id,
        generated_ty,
        CertificateTheoremGraphEdgeKind::MentionsType,
    )
}

fn add_axiom_edge(
    state: &mut GraphExtractionState<'_>,
    source: &CertificateTheoremGraphNodeId,
    axiom: &AxiomRef,
    kind: CertificateTheoremGraphEdgeKind,
) -> Result<(), CertificateTheoremGraphError> {
    let target = graph_node_for_global_ref(
        state.cert,
        &state.export_by_name,
        &state.import_modules,
        &axiom.global_ref,
        Some(axiom.decl_interface_hash),
        Some(CertificateTheoremGraphNodeKind::Axiom),
    )?;
    insert_node(&mut state.nodes, target.clone());
    state.edges.insert(CertificateTheoremGraphEdge {
        from: source.clone(),
        to: target.id,
        kind,
    });
    Ok(())
}

fn add_term_edges(
    state: &mut GraphExtractionState<'_>,
    source: &CertificateTheoremGraphNodeId,
    term_id: TermId,
    kind: CertificateTheoremGraphEdgeKind,
) -> Result<(), CertificateTheoremGraphError> {
    let mut refs = BTreeSet::new();
    let mut visited = BTreeSet::new();
    collect_const_refs(state.cert, term_id, &mut visited, &mut refs)?;
    for global_ref in refs {
        let target = graph_node_for_global_ref(
            state.cert,
            &state.export_by_name,
            &state.import_modules,
            &global_ref,
            None,
            None,
        )?;
        insert_node(&mut state.nodes, target.clone());
        state.edges.insert(CertificateTheoremGraphEdge {
            from: source.clone(),
            to: target.id,
            kind,
        });
    }
    Ok(())
}

fn collect_const_refs(
    cert: &ModuleCert,
    term_id: TermId,
    visited: &mut BTreeSet<TermId>,
    refs: &mut BTreeSet<GlobalRef>,
) -> Result<(), CertificateTheoremGraphError> {
    if !visited.insert(term_id) {
        return Ok(());
    }
    match term(cert, term_id)? {
        TermNode::Sort(_) | TermNode::BVar(_) => {}
        TermNode::Const { global_ref, .. } => {
            refs.insert(global_ref.clone());
        }
        TermNode::App(fun, arg) => {
            collect_const_refs(cert, *fun, visited, refs)?;
            collect_const_refs(cert, *arg, visited, refs)?;
        }
        TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => {
            collect_const_refs(cert, *ty, visited, refs)?;
            collect_const_refs(cert, *body, visited, refs)?;
        }
        TermNode::Let { ty, value, body } => {
            collect_const_refs(cert, *ty, visited, refs)?;
            collect_const_refs(cert, *value, visited, refs)?;
            collect_const_refs(cert, *body, visited, refs)?;
        }
    }
    Ok(())
}

fn local_decl_node(
    cert: &ModuleCert,
    export_by_name: &BTreeMap<Name, &ExportEntry>,
    decl_index: usize,
    decl: &DeclCert,
) -> Result<CertificateTheoremGraphNode, CertificateTheoremGraphError> {
    let name = decl_payload_name(cert, &decl.decl)?;
    let export = export_by_name.get(&name);
    let type_hash = export.map(|entry| entry.type_hash);
    let proof_hash = match &decl.decl {
        DeclPayload::Theorem { proof, .. } | DeclPayload::TheoremConstrained { proof, .. } => Some(
            term_hash(cert, *proof)
                .map_err(|_| CertificateTheoremGraphError::TermHash { term_id: *proof })?,
        ),
        _ => None,
    };
    let body_hash = match &decl.decl {
        DeclPayload::Def {
            value,
            reducibility,
            ..
        }
        | DeclPayload::DefConstrained {
            value,
            reducibility,
            ..
        } if *reducibility == CertReducibility::Reducible => Some(
            term_hash(cert, *value)
                .map_err(|_| CertificateTheoremGraphError::TermHash { term_id: *value })?,
        ),
        _ => None,
    };
    Ok(CertificateTheoremGraphNode {
        id: local_decl_node_id(cert, decl_index)?,
        kind: decl_payload_node_kind(&decl.decl),
        type_hash,
        proof_hash,
        body_hash,
    })
}

fn local_generated_node(
    cert: &ModuleCert,
    export_by_name: &BTreeMap<Name, &ExportEntry>,
    decl_index: usize,
    generated_name: NameId,
) -> Result<CertificateTheoremGraphNode, CertificateTheoremGraphError> {
    let name = name(cert, generated_name)?;
    let entry = export_by_name
        .get(&name)
        .ok_or_else(|| CertificateTheoremGraphError::MissingExport { name: name.clone() })?;
    Ok(CertificateTheoremGraphNode {
        id: CertificateTheoremGraphNodeId {
            scope: CertificateTheoremGraphNodeScope::LocalGenerated {
                source_decl_index: decl_index,
            },
            module: cert.header.module.clone(),
            name,
            decl_interface_hash: entry.decl_interface_hash,
        },
        kind: export_kind_to_node_kind(entry.kind),
        type_hash: Some(entry.type_hash),
        proof_hash: None,
        body_hash: entry.body_hash,
    })
}

fn graph_node_for_global_ref(
    cert: &ModuleCert,
    export_by_name: &BTreeMap<Name, &ExportEntry>,
    import_modules: &BTreeMap<Name, &VerifiedModule>,
    global_ref: &GlobalRef,
    decl_interface_hash: Option<Hash>,
    override_kind: Option<CertificateTheoremGraphNodeKind>,
) -> Result<CertificateTheoremGraphNode, CertificateTheoremGraphError> {
    match global_ref {
        GlobalRef::Builtin {
            name: name_id,
            decl_interface_hash,
        } => {
            let name = name(cert, *name_id)?;
            Ok(CertificateTheoremGraphNode {
                id: CertificateTheoremGraphNodeId {
                    scope: CertificateTheoremGraphNodeScope::Builtin,
                    module: Name::from_dotted("builtin"),
                    name,
                    decl_interface_hash: *decl_interface_hash,
                },
                kind: override_kind.unwrap_or(CertificateTheoremGraphNodeKind::Builtin),
                type_hash: None,
                proof_hash: None,
                body_hash: None,
            })
        }
        GlobalRef::Imported {
            import_index,
            name: name_id,
            decl_interface_hash: ref_interface_hash,
        } => {
            let import = cert.imports.get(*import_index).ok_or_else(|| {
                CertificateTheoremGraphError::MissingImportBinding {
                    module: Name::from_dotted("<invalid-import-index>"),
                }
            })?;
            let name = name(cert, *name_id)?;
            let verified = import_modules.get(&import.module).ok_or_else(|| {
                CertificateTheoremGraphError::MissingImportBinding {
                    module: import.module.clone(),
                }
            })?;
            let import_export = imported_export(verified, &import.module, &name)?;
            let expected_interface_hash = decl_interface_hash.unwrap_or(*ref_interface_hash);
            if expected_interface_hash != *ref_interface_hash
                || expected_interface_hash != import_export.decl_interface_hash
            {
                return Err(CertificateTheoremGraphError::DeclInterfaceHashMismatch { name });
            }
            Ok(CertificateTheoremGraphNode {
                id: CertificateTheoremGraphNodeId {
                    scope: CertificateTheoremGraphNodeScope::Imported {
                        import_export_hash: import.export_hash,
                        import_certificate_hash: import.certificate_hash,
                    },
                    module: import.module.clone(),
                    name,
                    decl_interface_hash: expected_interface_hash,
                },
                kind: override_kind.unwrap_or_else(|| export_kind_to_node_kind(import_export.kind)),
                type_hash: Some(import_export.type_hash),
                proof_hash: None,
                body_hash: import_export.body_hash,
            })
        }
        GlobalRef::Local { decl_index } => {
            let decl = cert.declarations.get(*decl_index).ok_or(
                CertificateTheoremGraphError::MissingDeclaration {
                    decl_index: *decl_index,
                },
            )?;
            let mut node = local_decl_node(cert, export_by_name, *decl_index, decl)?;
            if let Some(expected) = decl_interface_hash {
                if expected != node.id.decl_interface_hash {
                    return Err(CertificateTheoremGraphError::DeclInterfaceHashMismatch {
                        name: node.id.name,
                    });
                }
                node.id.decl_interface_hash = expected;
            }
            Ok(node)
        }
        GlobalRef::LocalGenerated {
            decl_index,
            name: name_id,
        } => {
            let mut node = local_generated_node(cert, export_by_name, *decl_index, *name_id)?;
            if let Some(expected) = decl_interface_hash {
                if expected != node.id.decl_interface_hash {
                    return Err(CertificateTheoremGraphError::DeclInterfaceHashMismatch {
                        name: node.id.name,
                    });
                }
                node.id.decl_interface_hash = expected;
            }
            Ok(node)
        }
    }
}

fn imported_export<'a>(
    import: &'a VerifiedModule,
    module: &Name,
    target: &Name,
) -> Result<&'a ExportEntry, CertificateTheoremGraphError> {
    for entry in import.export_block() {
        let Some(entry_name) = import.name_table().get(entry.name) else {
            return Err(CertificateTheoremGraphError::MissingImportedExport {
                module: module.clone(),
                name: target.clone(),
            });
        };
        if entry_name == target {
            return Ok(entry);
        }
    }
    Err(CertificateTheoremGraphError::MissingImportedExport {
        module: module.clone(),
        name: target.clone(),
    })
}

fn insert_node(
    nodes: &mut BTreeMap<CertificateTheoremGraphNodeId, CertificateTheoremGraphNode>,
    node: CertificateTheoremGraphNode,
) {
    nodes
        .entry(node.id.clone())
        .and_modify(|existing| {
            if existing.kind == CertificateTheoremGraphNodeKind::Unknown
                && node.kind != CertificateTheoremGraphNodeKind::Unknown
            {
                existing.kind = node.kind;
            }
            existing.type_hash = existing.type_hash.or(node.type_hash);
            existing.proof_hash = existing.proof_hash.or(node.proof_hash);
            existing.body_hash = existing.body_hash.or(node.body_hash);
        })
        .or_insert(node);
}

fn local_decl_node_id(
    cert: &ModuleCert,
    decl_index: usize,
) -> Result<CertificateTheoremGraphNodeId, CertificateTheoremGraphError> {
    let decl = cert
        .declarations
        .get(decl_index)
        .ok_or(CertificateTheoremGraphError::MissingDeclaration { decl_index })?;
    Ok(CertificateTheoremGraphNodeId {
        scope: CertificateTheoremGraphNodeScope::Local,
        module: cert.header.module.clone(),
        name: decl_payload_name(cert, &decl.decl)?,
        decl_interface_hash: decl.hashes.decl_interface_hash,
    })
}

fn decl_payload_name(
    cert: &ModuleCert,
    decl: &DeclPayload,
) -> Result<Name, CertificateTheoremGraphError> {
    name(cert, decl_payload_name_id(decl))
}

fn decl_payload_name_id(decl: &DeclPayload) -> NameId {
    match decl {
        DeclPayload::Axiom { name, .. }
        | DeclPayload::AxiomConstrained { name, .. }
        | DeclPayload::Def { name, .. }
        | DeclPayload::DefConstrained { name, .. }
        | DeclPayload::Theorem { name, .. }
        | DeclPayload::TheoremConstrained { name, .. }
        | DeclPayload::Inductive { name, .. }
        | DeclPayload::InductiveConstrained { name, .. }
        | DeclPayload::MutualInductiveBlock { name, .. } => *name,
    }
}

fn decl_payload_node_kind(decl: &DeclPayload) -> CertificateTheoremGraphNodeKind {
    match decl {
        DeclPayload::Axiom { .. } | DeclPayload::AxiomConstrained { .. } => {
            CertificateTheoremGraphNodeKind::Axiom
        }
        DeclPayload::Def { .. } | DeclPayload::DefConstrained { .. } => {
            CertificateTheoremGraphNodeKind::Definition
        }
        DeclPayload::Theorem { .. } | DeclPayload::TheoremConstrained { .. } => {
            CertificateTheoremGraphNodeKind::Theorem
        }
        DeclPayload::Inductive { .. }
        | DeclPayload::InductiveConstrained { .. }
        | DeclPayload::MutualInductiveBlock { .. } => CertificateTheoremGraphNodeKind::Inductive,
    }
}

fn export_kind_to_node_kind(kind: ExportKind) -> CertificateTheoremGraphNodeKind {
    match kind {
        ExportKind::Axiom => CertificateTheoremGraphNodeKind::Axiom,
        ExportKind::Def => CertificateTheoremGraphNodeKind::Definition,
        ExportKind::Theorem => CertificateTheoremGraphNodeKind::Theorem,
        ExportKind::Inductive => CertificateTheoremGraphNodeKind::Inductive,
        ExportKind::Constructor => CertificateTheoremGraphNodeKind::Constructor,
        ExportKind::Recursor => CertificateTheoremGraphNodeKind::Recursor,
    }
}

fn name(cert: &ModuleCert, name_id: NameId) -> Result<Name, CertificateTheoremGraphError> {
    cert.name_table
        .get(name_id)
        .cloned()
        .ok_or(CertificateTheoremGraphError::MissingName { name_id })
}

fn term(cert: &ModuleCert, term_id: TermId) -> Result<&TermNode, CertificateTheoremGraphError> {
    cert.term_table
        .get(term_id)
        .ok_or(CertificateTheoremGraphError::MissingTerm { term_id })
}

fn encode_node(out: &mut Vec<u8>, node: &CertificateTheoremGraphNode) {
    encode_node_id(out, &node.id);
    out.push(node_kind_tag(node.kind));
    encode_optional_hash(out, node.type_hash);
    encode_optional_hash(out, node.proof_hash);
    encode_optional_hash(out, node.body_hash);
}

fn encode_node_id(out: &mut Vec<u8>, id: &CertificateTheoremGraphNodeId) {
    encode_node_scope(out, &id.scope);
    encode_name(out, &id.module);
    encode_name(out, &id.name);
    out.extend(id.decl_interface_hash);
}

fn encode_node_scope(out: &mut Vec<u8>, scope: &CertificateTheoremGraphNodeScope) {
    match scope {
        CertificateTheoremGraphNodeScope::Builtin => out.push(0),
        CertificateTheoremGraphNodeScope::Imported {
            import_export_hash,
            import_certificate_hash,
        } => {
            out.push(1);
            out.extend(import_export_hash);
            encode_optional_hash(out, *import_certificate_hash);
        }
        CertificateTheoremGraphNodeScope::Local => out.push(2),
        CertificateTheoremGraphNodeScope::LocalGenerated { source_decl_index } => {
            out.push(3);
            encode_uvar(out, *source_decl_index as u64);
        }
    }
}

fn encode_name(out: &mut Vec<u8>, name: &Name) {
    encode_uvar(out, name.0.len() as u64);
    for component in &name.0 {
        encode_bytes(out, component.as_bytes());
    }
}

fn encode_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    encode_uvar(out, bytes.len() as u64);
    out.extend(bytes);
}

fn encode_optional_hash(out: &mut Vec<u8>, hash: Option<Hash>) {
    match hash {
        Some(hash) => {
            out.push(1);
            out.extend(hash);
        }
        None => out.push(0),
    }
}

fn encode_uvar(out: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn node_kind_tag(kind: CertificateTheoremGraphNodeKind) -> u8 {
    match kind {
        CertificateTheoremGraphNodeKind::Axiom => 0,
        CertificateTheoremGraphNodeKind::Definition => 1,
        CertificateTheoremGraphNodeKind::Theorem => 2,
        CertificateTheoremGraphNodeKind::Inductive => 3,
        CertificateTheoremGraphNodeKind::Constructor => 4,
        CertificateTheoremGraphNodeKind::Recursor => 5,
        CertificateTheoremGraphNodeKind::Builtin => 6,
        CertificateTheoremGraphNodeKind::Unknown => 7,
    }
}

fn extractor_version_tag(version: CertificateTheoremGraphExtractorVersion) -> u8 {
    match version {
        CertificateTheoremGraphExtractorVersion::CertificateGraphV1 => 0,
    }
}

fn edge_kind_tag(kind: CertificateTheoremGraphEdgeKind) -> u8 {
    match kind {
        CertificateTheoremGraphEdgeKind::ImportsDeclaration => 0,
        CertificateTheoremGraphEdgeKind::MentionsType => 1,
        CertificateTheoremGraphEdgeKind::UsesConstant => 2,
        CertificateTheoremGraphEdgeKind::GeneratedDeclaration => 3,
        CertificateTheoremGraphEdgeKind::DependsOnDirectAxiom => 4,
        CertificateTheoremGraphEdgeKind::DependsOnTransitiveAxiom => 5,
    }
}

fn hash_with_domain(domain: &[u8], payload: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update([0]);
    hasher.update(payload);
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use npa_cert::{
        build_module_cert, encode_module_cert, verify_module_cert, AxiomPolicy, CoreModule,
        VerifierSession,
    };
    use npa_kernel::{nat_inductive, Decl, Expr, Level, Reducibility};

    struct FixtureModule {
        verified: VerifiedModule,
        bytes: Vec<u8>,
    }

    fn fixture_module(module: CoreModule, imports: &[VerifiedModule]) -> FixtureModule {
        let cert = build_module_cert(module, imports).unwrap();
        let bytes = encode_module_cert(&cert).unwrap();
        let mut session = VerifierSession::new();
        for import in imports {
            session.register_verified_module(import.clone());
        }
        let verified = verify_module_cert(&bytes, &mut session, &AxiomPolicy::normal()).unwrap();
        FixtureModule { verified, bytes }
    }

    fn p() -> Expr {
        Expr::konst("Base.P", vec![])
    }

    fn id_p_type() -> Expr {
        Expr::pi("h", p(), p())
    }

    fn id_p_proof() -> Expr {
        Expr::lam("h", p(), Expr::bvar(0))
    }

    fn base_fixture() -> FixtureModule {
        fixture_module(
            CoreModule {
                name: Name::from_dotted("Base"),
                declarations: vec![Decl::Axiom {
                    name: "Base.P".to_owned(),
                    universe_params: Vec::new(),
                    ty: Expr::sort(Level::zero()),
                }],
            },
            &[],
        )
    }

    fn alternate_base_fixture() -> FixtureModule {
        fixture_module(
            CoreModule {
                name: Name::from_dotted("Base"),
                declarations: vec![Decl::Axiom {
                    name: "Base.Q".to_owned(),
                    universe_params: Vec::new(),
                    ty: Expr::sort(Level::zero()),
                }],
            },
            &[],
        )
    }

    fn client_fixture(base: &VerifiedModule) -> FixtureModule {
        fixture_module(
            CoreModule {
                name: Name::from_dotted("Client"),
                declarations: vec![
                    Decl::Def {
                        name: "Client.idP".to_owned(),
                        universe_params: Vec::new(),
                        ty: id_p_type(),
                        value: id_p_proof(),
                        reducibility: Reducibility::Reducible,
                    },
                    Decl::Theorem {
                        name: "Client.thmP".to_owned(),
                        universe_params: Vec::new(),
                        ty: id_p_type(),
                        proof: id_p_proof(),
                    },
                    Decl::Inductive {
                        name: "Nat".to_owned(),
                        universe_params: Vec::new(),
                        ty: Expr::sort(Level::succ(Level::zero())),
                        data: Box::new(nat_inductive()),
                    },
                ],
            },
            std::slice::from_ref(base),
        )
    }

    fn node_id_by_name(
        snapshot: &CertificateTheoremGraphSnapshot,
        name: &str,
    ) -> CertificateTheoremGraphNodeId {
        snapshot
            .nodes
            .iter()
            .find(|node| node.id.name.as_dotted() == name)
            .map(|node| node.id.clone())
            .unwrap_or_else(|| panic!("missing node {name}"))
    }

    #[test]
    fn theorem_graph_extracts_certificate_edges_and_hash_deterministically() {
        let base = base_fixture();
        let client = client_fixture(&base.verified);
        let options = CertificateTheoremGraphOptions::high_trust_certificate_hash_bound();

        let first = extract_certificate_theorem_graph(
            &client.bytes,
            std::slice::from_ref(&base.verified),
            options,
        )
        .unwrap();
        let second = extract_certificate_theorem_graph(
            &client.bytes,
            std::slice::from_ref(&base.verified),
            options,
        )
        .unwrap();

        assert_eq!(first.graph_hash, second.graph_hash);
        assert_eq!(
            first.graph_hash,
            certificate_theorem_graph_snapshot_hash(&first)
        );
        assert_eq!(first.imports[0].module, Name::from_dotted("Base"));
        assert_eq!(first.imports[0].export_hash, base.verified.export_hash());
        assert_eq!(
            first.imports[0].certificate_hash,
            Some(base.verified.certificate_hash())
        );

        let theorem = node_id_by_name(&first, "Client.thmP");
        let imported_axiom = node_id_by_name(&first, "Base.P");
        assert!(first
            .direct_axiom_dependencies(&theorem)
            .contains(&imported_axiom));
        assert!(first
            .transitive_axiom_dependencies(&theorem)
            .contains(&imported_axiom));
        assert!(first
            .direct_dependency_targets(&theorem)
            .contains(&imported_axiom));

        let def = first.node(&node_id_by_name(&first, "Client.idP")).unwrap();
        assert!(def.body_hash.is_some());
        assert!(first.edges.iter().any(|edge| {
            edge.from == def.id
                && edge.to == imported_axiom
                && edge.kind == CertificateTheoremGraphEdgeKind::UsesConstant
        }));

        let nat = node_id_by_name(&first, "Nat");
        let nat_zero = node_id_by_name(&first, "Nat.zero");
        let nat_rec = node_id_by_name(&first, "Nat.rec");
        assert!(first.nodes.iter().any(|node| {
            node.id == nat_zero && node.kind == CertificateTheoremGraphNodeKind::Constructor
        }));
        assert!(first.nodes.iter().any(|node| {
            node.id == nat_rec && node.kind == CertificateTheoremGraphNodeKind::Recursor
        }));
        assert!(first.edges.iter().any(|edge| {
            edge.from == nat
                && edge.to == nat_zero
                && edge.kind == CertificateTheoremGraphEdgeKind::GeneratedDeclaration
        }));
        assert!(first.edges.iter().any(|edge| {
            edge.from == nat
                && edge.to == nat_rec
                && edge.kind == CertificateTheoremGraphEdgeKind::GeneratedDeclaration
        }));
        assert!(first.edges.iter().any(|edge| {
            edge.from == nat_rec && edge.kind == CertificateTheoremGraphEdgeKind::MentionsType
        }));
    }

    #[test]
    fn theorem_graph_hash_ignores_source_text_and_human_debug_metadata() {
        let base = base_fixture();
        let client = client_fixture(&base.verified);
        let source_text = "theorem thmP := by intro h; exact h";
        let human_debug_metadata = "pretty names, spans, tactic trace";
        let first = extract_certificate_theorem_graph(
            &client.bytes,
            std::slice::from_ref(&base.verified),
            CertificateTheoremGraphOptions::default(),
        )
        .unwrap();

        let changed_source_text = format!("{source_text}\n-- edited comment");
        let changed_human_debug_metadata = format!("{human_debug_metadata}\nspan:changed");
        let second = extract_certificate_theorem_graph(
            &client.bytes,
            std::slice::from_ref(&base.verified),
            CertificateTheoremGraphOptions::default(),
        )
        .unwrap();

        assert_ne!(source_text, changed_source_text);
        assert_ne!(human_debug_metadata, changed_human_debug_metadata);
        assert_eq!(first.graph_hash, second.graph_hash);
    }

    #[test]
    fn theorem_graph_checks_import_export_and_high_trust_certificate_bindings() {
        let base = base_fixture();
        let alternate = alternate_base_fixture();
        let client = client_fixture(&base.verified);

        let err = extract_certificate_theorem_graph(
            &client.bytes,
            &[alternate.verified],
            CertificateTheoremGraphOptions::default(),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            CertificateTheoremGraphError::ImportExportHashMismatch { .. }
        ));

        let mut cert = decode_module_cert(&client.bytes).unwrap();
        cert.imports[0].certificate_hash = None;
        let err = extract_certificate_theorem_graph_from_cert(
            &cert,
            std::slice::from_ref(&base.verified),
            CertificateTheoremGraphOptions::high_trust_certificate_hash_bound(),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            CertificateTheoremGraphError::ImportCertificateHashMissing { .. }
        ));

        let mut cert = decode_module_cert(&client.bytes).unwrap();
        cert.declarations[0].dependencies[0].decl_interface_hash = [7; 32];
        let err = extract_certificate_theorem_graph_from_cert(
            &cert,
            std::slice::from_ref(&base.verified),
            CertificateTheoremGraphOptions::default(),
        )
        .unwrap_err();
        assert!(matches!(
            err,
            CertificateTheoremGraphError::DeclInterfaceHashMismatch { .. }
        ));
    }
}
