use std::collections::BTreeSet;

use npa_kernel::Env;

use crate::*;

pub(crate) fn verify_module_cert_impl(
    bytes: &[u8],
    session: &mut VerifierSession,
    policy: &AxiomPolicy,
) -> Result<VerifiedModule> {
    let cert = decode_module_cert(bytes)?;
    let canonical = encode_module_cert_full(&cert);
    if canonical != bytes {
        return Err(CertError::NonCanonicalEncoding {
            object: "ModuleCert",
        });
    }
    verify_header(&cert.header)?;
    verify_tables(&cert)?;
    verify_hashes(&cert)?;
    verify_declaration_order(&cert)?;
    verify_inductive_generated_artifacts(&cert)?;

    let imports = resolve_imports(&cert, session, policy)?;
    verify_dependencies_and_axioms(&cert, &imports)?;
    enforce_axiom_policy(&cert, policy)?;
    enforce_import_axiom_policy(&imports, policy)?;

    let mut env = Env::new();
    add_imports_to_env(&mut env, &imports)?;

    for decl in cert_to_kernel_decls(&cert)? {
        add_decl_to_env(&mut env, decl.clone())?;
    }

    let verified = VerifiedModule {
        module: cert.header.module.clone(),
        name_table: cert.name_table.clone(),
        level_table: cert.level_table.clone(),
        term_table: cert.term_table.clone(),
        declarations: cert.declarations.clone(),
        export_hash: cert.hashes.export_hash,
        certificate_hash: cert.hashes.certificate_hash,
        export_block: cert.export_block.clone(),
        axiom_report: cert.axiom_report.clone(),
    };
    session.insert_verified(verified.clone());
    Ok(verified)
}
fn verify_header(header: &CertHeader) -> Result<()> {
    if header.format != FORMAT || header.core_spec != CORE_SPEC {
        return Err(CertError::UnsupportedFormat {
            format: header.format.clone(),
            core_spec: header.core_spec.clone(),
        });
    }
    Ok(())
}

fn verify_tables(cert: &ModuleCert) -> Result<()> {
    if !cert.imports.windows(2).all(|pair| {
        (
            pair[0].module.clone(),
            pair[0].export_hash,
            pair[0].certificate_hash,
        ) < (
            pair[1].module.clone(),
            pair[1].export_hash,
            pair[1].certificate_hash,
        )
    }) {
        return Err(CertError::NonCanonicalEncoding { object: "Imports" });
    }
    if !cert.name_table.windows(2).all(|pair| pair[0] < pair[1]) {
        return Err(CertError::NonCanonicalEncoding {
            object: "NameTable",
        });
    }
    for (index, level) in cert.level_table.iter().enumerate() {
        let ok = match level {
            LevelNode::Zero | LevelNode::Param(_) => true,
            LevelNode::Succ(inner) => *inner < index,
            LevelNode::Max(lhs, rhs) | LevelNode::IMax(lhs, rhs) => *lhs < index && *rhs < index,
        };
        let name_ok = match level {
            LevelNode::Param(name) => *name < cert.name_table.len(),
            _ => true,
        };
        if !ok || !name_ok {
            return Err(CertError::NonCanonicalEncoding {
                object: "LevelTable",
            });
        }
    }
    let level_hashes = compute_level_hashes(&cert.level_table, &cert.name_table)?;
    let level_keys = cert
        .level_table
        .iter()
        .enumerate()
        .map(|(index, _)| {
            Ok((
                level_node_height(&cert.level_table, index)?,
                level_hashes[index],
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    if !level_keys.windows(2).all(|pair| pair[0] < pair[1]) {
        return Err(CertError::NonCanonicalEncoding {
            object: "LevelTable",
        });
    }

    for (index, term) in cert.term_table.iter().enumerate() {
        let ok = match term {
            TermNode::Sort(_) | TermNode::BVar(_) | TermNode::Const { .. } => true,
            TermNode::App(fun, arg) => *fun < index && *arg < index,
            TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => *ty < index && *body < index,
            TermNode::Let { ty, value, body } => *ty < index && *value < index && *body < index,
        };
        let refs_ok = match term {
            TermNode::Sort(level) => *level < cert.level_table.len(),
            TermNode::Const { global_ref, levels } => {
                global_ref_is_in_range(cert, global_ref)
                    && levels.iter().all(|level| *level < cert.level_table.len())
            }
            _ => true,
        };
        if !ok || !refs_ok {
            return Err(CertError::NonCanonicalEncoding {
                object: "TermTable",
            });
        }
    }
    let term_hashes = compute_term_hashes(&cert.term_table, &level_hashes)?;
    let term_keys = cert
        .term_table
        .iter()
        .enumerate()
        .map(|(index, _)| {
            Ok((
                term_node_height(&cert.term_table, index)?,
                term_hashes[index],
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    if !term_keys.windows(2).all(|pair| pair[0] < pair[1]) {
        return Err(CertError::NonCanonicalEncoding {
            object: "TermTable",
        });
    }
    verify_reachable_tables_and_bvars(cert)?;
    verify_name_table_reachable(cert)?;
    Ok(())
}

fn verify_name_table_reachable(cert: &ModuleCert) -> Result<()> {
    let mut names = BTreeSet::new();
    names.insert(cert.header.module.clone());
    for import in &cert.imports {
        names.insert(import.module.clone());
    }

    for level in &cert.level_table {
        collect_level_node_names(cert, level, &mut names)?;
    }
    for term in &cert.term_table {
        collect_term_node_names(cert, term, &mut names)?;
    }
    for decl in &cert.declarations {
        collect_decl_payload_names(cert, &decl.decl, &mut names)?;
        collect_dependency_entry_names(cert, &decl.dependencies, &mut names)?;
        collect_axiom_ref_names(cert, &decl.axiom_dependencies, &mut names)?;
    }
    for entry in &cert.export_block {
        collect_name_id(cert, entry.name, &mut names)?;
        collect_name_ids(cert, &entry.universe_params, &mut names)?;
        collect_axiom_ref_names(cert, &entry.axiom_dependencies, &mut names)?;
    }
    for report in &cert.axiom_report.per_declaration {
        collect_axiom_ref_names(cert, &report.direct_axioms, &mut names)?;
        collect_axiom_ref_names(cert, &report.transitive_axioms, &mut names)?;
    }
    collect_axiom_ref_names(cert, &cert.axiom_report.module_axioms, &mut names)?;

    let expected = names.into_iter().collect::<Vec<_>>();
    if expected != cert.name_table {
        return Err(CertError::NonCanonicalEncoding {
            object: "NameTable",
        });
    }
    Ok(())
}

fn collect_level_node_names(
    cert: &ModuleCert,
    level: &LevelNode,
    names: &mut BTreeSet<Name>,
) -> Result<()> {
    if let LevelNode::Param(name) = level {
        collect_name_id(cert, *name, names)?;
    }
    Ok(())
}

fn collect_term_node_names(
    cert: &ModuleCert,
    term: &TermNode,
    names: &mut BTreeSet<Name>,
) -> Result<()> {
    if let TermNode::Const { global_ref, .. } = term {
        collect_global_ref_names(cert, global_ref, names)?;
    }
    Ok(())
}

fn collect_decl_payload_names(
    cert: &ModuleCert,
    decl: &DeclPayload,
    names: &mut BTreeSet<Name>,
) -> Result<()> {
    match decl {
        DeclPayload::Axiom {
            name,
            universe_params,
            ..
        }
        | DeclPayload::Def {
            name,
            universe_params,
            ..
        }
        | DeclPayload::Theorem {
            name,
            universe_params,
            ..
        } => {
            collect_name_id(cert, *name, names)?;
            collect_name_ids(cert, universe_params, names)?;
        }
        DeclPayload::Inductive {
            name,
            universe_params,
            constructors,
            recursor,
            ..
        } => {
            collect_name_id(cert, *name, names)?;
            collect_name_ids(cert, universe_params, names)?;
            for constructor in constructors {
                collect_name_id(cert, constructor.name, names)?;
            }
            if let Some(recursor) = recursor {
                collect_name_id(cert, recursor.name, names)?;
                collect_name_ids(cert, &recursor.universe_params, names)?;
            }
        }
    }
    Ok(())
}

fn collect_dependency_entry_names(
    cert: &ModuleCert,
    dependencies: &[DependencyEntry],
    names: &mut BTreeSet<Name>,
) -> Result<()> {
    for dependency in dependencies {
        collect_global_ref_names(cert, &dependency.global_ref, names)?;
    }
    Ok(())
}

fn collect_axiom_ref_names(
    cert: &ModuleCert,
    axioms: &[AxiomRef],
    names: &mut BTreeSet<Name>,
) -> Result<()> {
    for axiom in axioms {
        collect_global_ref_names(cert, &axiom.global_ref, names)?;
        collect_name_id(cert, axiom.name, names)?;
    }
    Ok(())
}

fn collect_global_ref_names(
    cert: &ModuleCert,
    global_ref: &GlobalRef,
    names: &mut BTreeSet<Name>,
) -> Result<()> {
    match global_ref {
        GlobalRef::Imported { name, .. } | GlobalRef::LocalGenerated { name, .. } => {
            collect_name_id(cert, *name, names)?;
        }
        GlobalRef::Local { .. } => {}
    }
    Ok(())
}

fn collect_name_ids(cert: &ModuleCert, ids: &[NameId], names: &mut BTreeSet<Name>) -> Result<()> {
    for id in ids {
        collect_name_id(cert, *id, names)?;
    }
    Ok(())
}

fn collect_name_id(cert: &ModuleCert, id: NameId, names: &mut BTreeSet<Name>) -> Result<()> {
    names.insert(
        cert.name_table
            .get(id)
            .cloned()
            .ok_or(CertError::DecodeError)?,
    );
    Ok(())
}

fn verify_reachable_tables_and_bvars(cert: &ModuleCert) -> Result<()> {
    let mut reachable_terms = BTreeSet::new();
    let mut seen_term_depths = BTreeSet::new();

    for decl in &cert.declarations {
        match &decl.decl {
            DeclPayload::Axiom { ty, .. } => {
                verify_term_scope(cert, *ty, 0, &mut seen_term_depths, &mut reachable_terms)?;
            }
            DeclPayload::Def { ty, value, .. } => {
                verify_term_scope(cert, *ty, 0, &mut seen_term_depths, &mut reachable_terms)?;
                verify_term_scope(cert, *value, 0, &mut seen_term_depths, &mut reachable_terms)?;
            }
            DeclPayload::Theorem { ty, proof, .. } => {
                verify_term_scope(cert, *ty, 0, &mut seen_term_depths, &mut reachable_terms)?;
                verify_term_scope(cert, *proof, 0, &mut seen_term_depths, &mut reachable_terms)?;
            }
            DeclPayload::Inductive {
                params,
                indices,
                sort,
                constructors,
                recursor,
                ..
            } => {
                let ty = inductive_export_type_term_id(&cert.term_table, params, indices, *sort)?;
                verify_term_scope(cert, ty, 0, &mut seen_term_depths, &mut reachable_terms)?;
                for constructor in constructors {
                    verify_term_scope(
                        cert,
                        constructor.ty,
                        0,
                        &mut seen_term_depths,
                        &mut reachable_terms,
                    )?;
                }
                if let Some(recursor) = recursor {
                    verify_term_scope(
                        cert,
                        recursor.ty,
                        0,
                        &mut seen_term_depths,
                        &mut reachable_terms,
                    )?;
                }
            }
        }
    }

    if reachable_terms.len() != cert.term_table.len() {
        return Err(CertError::NonCanonicalEncoding {
            object: "TermTable",
        });
    }

    let mut reachable_levels = BTreeSet::new();
    for term in &reachable_terms {
        collect_levels_from_term_node(cert, *term, &mut reachable_levels)?;
    }
    if reachable_levels.len() != cert.level_table.len() {
        return Err(CertError::NonCanonicalEncoding {
            object: "LevelTable",
        });
    }

    Ok(())
}

fn verify_term_scope(
    cert: &ModuleCert,
    term: TermId,
    depth: u32,
    seen: &mut BTreeSet<(TermId, u32)>,
    reachable_terms: &mut BTreeSet<TermId>,
) -> Result<()> {
    if !seen.insert((term, depth)) {
        reachable_terms.insert(term);
        return Ok(());
    }
    reachable_terms.insert(term);
    match cert.term_table.get(term).ok_or(CertError::DecodeError)? {
        TermNode::Sort(_) | TermNode::Const { .. } => {}
        TermNode::BVar(index) => {
            if *index >= depth {
                return Err(CertError::InvalidBVar { index: *index });
            }
        }
        TermNode::App(fun, arg) => {
            verify_term_scope(cert, *fun, depth, seen, reachable_terms)?;
            verify_term_scope(cert, *arg, depth, seen, reachable_terms)?;
        }
        TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => {
            verify_term_scope(cert, *ty, depth, seen, reachable_terms)?;
            verify_term_scope(cert, *body, depth + 1, seen, reachable_terms)?;
        }
        TermNode::Let { ty, value, body } => {
            verify_term_scope(cert, *ty, depth, seen, reachable_terms)?;
            verify_term_scope(cert, *value, depth, seen, reachable_terms)?;
            verify_term_scope(cert, *body, depth + 1, seen, reachable_terms)?;
        }
    }
    Ok(())
}

fn collect_levels_from_term_node(
    cert: &ModuleCert,
    term: TermId,
    reachable_levels: &mut BTreeSet<LevelId>,
) -> Result<()> {
    match cert.term_table.get(term).ok_or(CertError::DecodeError)? {
        TermNode::Sort(level) => collect_level_reachable(cert, *level, reachable_levels)?,
        TermNode::Const { levels, .. } => {
            for level in levels {
                collect_level_reachable(cert, *level, reachable_levels)?;
            }
        }
        TermNode::BVar(_)
        | TermNode::App(_, _)
        | TermNode::Lam { .. }
        | TermNode::Pi { .. }
        | TermNode::Let { .. } => {}
    }
    Ok(())
}

fn collect_level_reachable(
    cert: &ModuleCert,
    level: LevelId,
    reachable_levels: &mut BTreeSet<LevelId>,
) -> Result<()> {
    if !reachable_levels.insert(level) {
        return Ok(());
    }
    match cert.level_table.get(level).ok_or(CertError::DecodeError)? {
        LevelNode::Zero | LevelNode::Param(_) => {}
        LevelNode::Succ(inner) => collect_level_reachable(cert, *inner, reachable_levels)?,
        LevelNode::Max(lhs, rhs) | LevelNode::IMax(lhs, rhs) => {
            collect_level_reachable(cert, *lhs, reachable_levels)?;
            collect_level_reachable(cert, *rhs, reachable_levels)?;
        }
    }
    Ok(())
}

fn verify_hashes(cert: &ModuleCert) -> Result<()> {
    let level_hashes = compute_level_hashes(&cert.level_table, &cert.name_table)?;
    let term_hashes = compute_term_hashes(&cert.term_table, &level_hashes)?;
    for decl in &cert.declarations {
        let expected = compute_decl_hashes(
            &decl.decl,
            &decl.dependencies,
            &decl.axiom_dependencies,
            &level_hashes,
            &term_hashes,
            &cert.name_table,
        )?;
        if expected.decl_interface_hash != decl.hashes.decl_interface_hash {
            return Err(CertError::HashMismatch {
                object: HashObject::DeclInterface,
                expected: expected.decl_interface_hash,
                actual: decl.hashes.decl_interface_hash,
            });
        }
        if expected.decl_certificate_hash != decl.hashes.decl_certificate_hash {
            return Err(CertError::HashMismatch {
                object: HashObject::DeclCertificate,
                expected: expected.decl_certificate_hash,
                actual: decl.hashes.decl_certificate_hash,
            });
        }
    }

    let expected_export_block =
        build_export_block(&cert.declarations, &cert.term_table, &term_hashes)?;
    let expected_export = hash_with_domain(
        b"NPA-MODULE-EXPORT-0.1",
        &encode_export_block(&expected_export_block),
    );
    if expected_export_block != cert.export_block || expected_export != cert.hashes.export_hash {
        return Err(CertError::HashMismatch {
            object: HashObject::ExportBlock,
            expected: expected_export,
            actual: cert.hashes.export_hash,
        });
    }

    let expected_axioms = hash_with_domain(
        b"NPA-AXIOM-REPORT-0.1",
        &encode_axiom_report(&cert.axiom_report),
    );
    if expected_axioms != cert.hashes.axiom_report_hash {
        return Err(CertError::HashMismatch {
            object: HashObject::AxiomReport,
            expected: expected_axioms,
            actual: cert.hashes.axiom_report_hash,
        });
    }

    let expected_cert = hash_with_domain(
        b"NPA-MODULE-CERT-0.1",
        &encode_module_cert_without_certificate_hash(cert),
    );
    if expected_cert != cert.hashes.certificate_hash {
        return Err(CertError::HashMismatch {
            object: HashObject::ModuleCertificate,
            expected: expected_cert,
            actual: cert.hashes.certificate_hash,
        });
    }

    Ok(())
}

fn verify_declaration_order(cert: &ModuleCert) -> Result<()> {
    let local_names = (0..cert.declarations.len())
        .map(|index| decl_name_as_name(cert, index))
        .collect::<Result<Vec<_>>>()?;
    ensure_unique_names(&local_names)?;

    let dependencies = cert
        .declarations
        .iter()
        .enumerate()
        .map(|(decl_index, decl)| {
            let mut deps = BTreeSet::new();
            for dependency in &decl.dependencies {
                match &dependency.global_ref {
                    GlobalRef::Local {
                        decl_index: dependency_index,
                    } => {
                        if *dependency_index >= decl_index {
                            return Err(CertError::DependencyCycle {
                                name: local_names[decl_index].clone(),
                            });
                        }
                        deps.insert(*dependency_index);
                    }
                    GlobalRef::LocalGenerated {
                        decl_index: dependency_index,
                        name,
                    } => {
                        if *dependency_index >= decl_index {
                            return Err(CertError::DependencyCycle {
                                name: local_names[decl_index].clone(),
                            });
                        }
                        if !local_generated_entry_exists(cert, *dependency_index, *name)? {
                            return Err(CertError::UnknownDependency {
                                name: cert
                                    .name_table
                                    .get(*name)
                                    .cloned()
                                    .ok_or(CertError::DecodeError)?,
                            });
                        }
                        deps.insert(*dependency_index);
                    }
                    GlobalRef::Imported { .. } => {}
                }
            }
            Ok(deps)
        })
        .collect::<Result<Vec<_>>>()?;

    let mut emitted = BTreeSet::new();
    let mut remaining: BTreeSet<_> = (0..cert.declarations.len()).collect();
    let mut expected = Vec::with_capacity(cert.declarations.len());
    while !remaining.is_empty() {
        let mut ready: Vec<_> = remaining
            .iter()
            .copied()
            .filter(|index| dependencies[*index].is_subset(&emitted))
            .collect();
        if ready.is_empty() {
            let index = *remaining.iter().next().ok_or(CertError::DecodeError)?;
            return Err(CertError::DependencyCycle {
                name: local_names[index].clone(),
            });
        }
        ready.sort_by_key(|index| local_names[*index].clone());
        for index in ready {
            remaining.remove(&index);
            emitted.insert(index);
            expected.push(index);
        }
    }

    if expected != (0..cert.declarations.len()).collect::<Vec<_>>() {
        return Err(CertError::NonCanonicalEncoding {
            object: "Declarations",
        });
    }

    Ok(())
}

fn global_ref_is_in_range(cert: &ModuleCert, global_ref: &GlobalRef) -> bool {
    match global_ref {
        GlobalRef::Imported {
            import_index, name, ..
        } => *import_index < cert.imports.len() && *name < cert.name_table.len(),
        GlobalRef::Local { decl_index } => *decl_index < cert.declarations.len(),
        GlobalRef::LocalGenerated { decl_index, name } => {
            *decl_index < cert.declarations.len() && *name < cert.name_table.len()
        }
    }
}

fn level_node_height(levels: &[LevelNode], index: usize) -> Result<usize> {
    Ok(match levels.get(index).ok_or(CertError::DecodeError)? {
        LevelNode::Zero | LevelNode::Param(_) => 0,
        LevelNode::Succ(inner) => level_node_height(levels, *inner)? + 1,
        LevelNode::Max(lhs, rhs) | LevelNode::IMax(lhs, rhs) => {
            level_node_height(levels, *lhs)?.max(level_node_height(levels, *rhs)?) + 1
        }
    })
}

fn term_node_height(terms: &[TermNode], index: usize) -> Result<usize> {
    Ok(match terms.get(index).ok_or(CertError::DecodeError)? {
        TermNode::Sort(_) | TermNode::BVar(_) | TermNode::Const { .. } => 0,
        TermNode::App(fun, arg) => {
            term_node_height(terms, *fun)?.max(term_node_height(terms, *arg)?) + 1
        }
        TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => {
            term_node_height(terms, *ty)?.max(term_node_height(terms, *body)?) + 1
        }
        TermNode::Let { ty, value, body } => {
            term_node_height(terms, *ty)?
                .max(term_node_height(terms, *value)?)
                .max(term_node_height(terms, *body)?)
                + 1
        }
    })
}

fn resolve_imports<'a>(
    cert: &ModuleCert,
    session: &'a VerifierSession,
    policy: &AxiomPolicy,
) -> Result<Vec<&'a VerifiedModule>> {
    let mut imports = Vec::new();
    for entry in &cert.imports {
        if policy.mode == TrustMode::HighTrust && entry.certificate_hash.is_none() {
            return Err(CertError::MissingImportCertificateHash {
                module: entry.module.clone(),
            });
        }
        imports.push(session.find_import(entry, policy.mode)?);
    }
    Ok(imports)
}

pub(crate) fn add_imports_to_env(env: &mut Env, imports: &[&VerifiedModule]) -> Result<()> {
    for import in import_kernel_order(imports)? {
        for decl in verified_module_to_kernel_decls(import)? {
            add_decl_to_env(env, decl)?;
        }
    }
    Ok(())
}

fn import_kernel_order<'a>(imports: &[&'a VerifiedModule]) -> Result<Vec<&'a VerifiedModule>> {
    let mut added = vec![false; imports.len()];
    let mut order = Vec::with_capacity(imports.len());

    while order.len() < imports.len() {
        let mut progressed = false;
        for (index, import) in imports.iter().enumerate() {
            if added[index] || !import_dependencies_satisfied(import, imports, &added)? {
                continue;
            }
            added[index] = true;
            order.push(*import);
            progressed = true;
        }

        if !progressed {
            let name = imports
                .iter()
                .enumerate()
                .find_map(|(index, import)| (!added[index]).then(|| import.module.clone()))
                .ok_or(CertError::DecodeError)?;
            return Err(CertError::DependencyCycle { name });
        }
    }

    Ok(order)
}

fn import_dependencies_satisfied(
    import: &VerifiedModule,
    imports: &[&VerifiedModule],
    added: &[bool],
) -> Result<bool> {
    for (dep_name, decl_interface_hash) in imported_dependency_targets(import)? {
        let mut found = false;
        let mut satisfied = false;
        for (index, candidate) in imports.iter().enumerate() {
            if module_exports_dependency(candidate, &dep_name, decl_interface_hash)? {
                found = true;
                satisfied |= added[index];
            }
        }
        if !found {
            return Err(CertError::UnknownDependency { name: dep_name });
        }
        if !satisfied {
            return Ok(false);
        }
    }
    Ok(true)
}

fn imported_dependency_targets(module: &VerifiedModule) -> Result<BTreeSet<(Name, Hash)>> {
    let mut deps = BTreeSet::new();
    for decl in &module.declarations {
        for dependency in &decl.dependencies {
            if let GlobalRef::Imported {
                name,
                decl_interface_hash,
                ..
            } = &dependency.global_ref
            {
                let name = module
                    .name_table
                    .get(*name)
                    .ok_or(CertError::DecodeError)?
                    .clone();
                deps.insert((name, *decl_interface_hash));
            }
        }
    }
    Ok(deps)
}

fn module_exports_dependency(
    module: &VerifiedModule,
    name: &Name,
    decl_interface_hash: Hash,
) -> Result<bool> {
    for entry in &module.export_block {
        let entry_name = module
            .name_table
            .get(entry.name)
            .ok_or(CertError::DecodeError)?;
        if entry_name == name && entry.decl_interface_hash == decl_interface_hash {
            return Ok(true);
        }
    }
    Ok(false)
}

fn verify_dependencies_and_axioms(cert: &ModuleCert, imports: &[&VerifiedModule]) -> Result<()> {
    let mut previous_axioms: Vec<Vec<AxiomRef>> = Vec::new();
    let mut expected_reports = Vec::new();

    for (decl_index, decl) in cert.declarations.iter().enumerate() {
        let expected_deps = expected_dependencies_for_decl(cert, imports, decl_index, &decl.decl)?;
        if expected_deps != decl.dependencies {
            return Err(CertError::AxiomReportMismatch {
                decl: Some(decl_name_as_name(cert, decl_index)?),
            });
        }

        let (direct_axioms, transitive_axioms) = expected_axioms_for_decl(
            cert,
            imports,
            decl_index,
            &decl.decl,
            &expected_deps,
            &previous_axioms,
        )?;
        if transitive_axioms != decl.axiom_dependencies {
            return Err(CertError::AxiomReportMismatch {
                decl: Some(decl_name_as_name(cert, decl_index)?),
            });
        }

        let expected_report = DeclAxiomReport {
            decl_index,
            direct_axioms,
            transitive_axioms,
        };
        if cert.axiom_report.per_declaration.get(decl_index) != Some(&expected_report) {
            return Err(CertError::AxiomReportMismatch {
                decl: Some(decl_name_as_name(cert, decl_index)?),
            });
        }

        previous_axioms.push(expected_report.transitive_axioms.clone());
        expected_reports.push(expected_report);
    }

    if cert.axiom_report.per_declaration.len() != expected_reports.len() {
        return Err(CertError::AxiomReportMismatch { decl: None });
    }

    let expected_module_axioms = union_axioms(
        expected_reports
            .iter()
            .flat_map(|report| report.transitive_axioms.iter().cloned()),
    );
    if expected_module_axioms != cert.axiom_report.module_axioms {
        return Err(CertError::AxiomReportMismatch { decl: None });
    }

    Ok(())
}

fn verify_inductive_generated_artifacts(cert: &ModuleCert) -> Result<()> {
    for decl in &cert.declarations {
        let DeclPayload::Inductive {
            name,
            params,
            constructors,
            recursor: Some(recursor),
            ..
        } = &decl.decl
        else {
            continue;
        };

        let expected_rules = RecursorRulesSpec {
            minor_start: params.len() + 1,
            major_index: params.len() + 1 + constructors.len(),
        };
        if recursor.rules != expected_rules {
            return Err(CertError::InductiveGeneratedArtifactMismatch {
                name: cert
                    .name_table
                    .get(*name)
                    .ok_or(CertError::DecodeError)?
                    .clone(),
            });
        }
    }
    Ok(())
}

pub(crate) fn expected_dependencies_for_decl(
    cert: &ModuleCert,
    imports: &[&VerifiedModule],
    decl_index: usize,
    decl: &DeclPayload,
) -> Result<Vec<DependencyEntry>> {
    let mut refs = BTreeSet::new();
    for term in decl_term_ids(decl) {
        collect_global_refs_from_term(cert, term, &mut refs)?;
    }

    let current_decl_index = decl_index;
    let allow_self_reference = matches!(decl, DeclPayload::Inductive { .. });
    refs.into_iter()
        .filter(|global_ref| {
            !matches!(
                global_ref,
                GlobalRef::Local {
                    decl_index: referenced_decl_index,
                } | GlobalRef::LocalGenerated {
                    decl_index: referenced_decl_index,
                    ..
                } if allow_self_reference && *referenced_decl_index == current_decl_index
            )
        })
        .map(|global_ref| {
            let decl_interface_hash =
                interface_hash_for_global_ref(cert, imports, decl_index, &global_ref)?;
            Ok(DependencyEntry {
                global_ref,
                decl_interface_hash,
            })
        })
        .collect()
}

pub(crate) fn expected_axioms_for_decl(
    cert: &ModuleCert,
    imports: &[&VerifiedModule],
    decl_index: usize,
    decl: &DeclPayload,
    dependencies: &[DependencyEntry],
    previous_axioms: &[Vec<AxiomRef>],
) -> Result<(Vec<AxiomRef>, Vec<AxiomRef>)> {
    let mut direct = BTreeSet::new();
    let mut transitive = BTreeSet::new();
    for dependency in dependencies {
        match &dependency.global_ref {
            GlobalRef::Local { decl_index } => {
                if let Some(dep_axioms) = previous_axioms.get(*decl_index) {
                    if let Some(axiom) = local_axiom_ref_for_decl(*decl_index, dep_axioms) {
                        direct.insert(axiom);
                    }
                    transitive.extend(dep_axioms.iter().cloned());
                }
            }
            GlobalRef::LocalGenerated { decl_index, .. } => {
                if let Some(dep_axioms) = previous_axioms.get(*decl_index) {
                    transitive.extend(dep_axioms.iter().cloned());
                }
            }
            GlobalRef::Imported {
                import_index,
                name,
                decl_interface_hash,
            } => {
                let entry =
                    imported_export_entry_for_global_ref(cert, imports, &dependency.global_ref)?;
                if entry.kind == ExportKind::Axiom {
                    direct.insert(AxiomRef {
                        global_ref: dependency.global_ref.clone(),
                        name: *name,
                        decl_interface_hash: *decl_interface_hash,
                    });
                }
                let import = imports.get(*import_index).ok_or(CertError::DecodeError)?;
                for axiom in &entry.axiom_dependencies {
                    transitive.insert(remap_axiom_ref_from_cert_import(
                        cert, imports, import, axiom,
                    )?);
                }
            }
        }
    }
    if let DeclPayload::Axiom { name, .. } = decl {
        let self_ref = AxiomRef {
            global_ref: GlobalRef::Local { decl_index },
            name: *name,
            decl_interface_hash: cert
                .declarations
                .get(decl_index)
                .ok_or(CertError::DecodeError)?
                .hashes
                .decl_interface_hash,
        };
        direct.insert(self_ref.clone());
        transitive.insert(self_ref);
    }
    Ok((
        direct.into_iter().collect(),
        transitive.into_iter().collect(),
    ))
}

fn remap_axiom_ref_from_cert_import(
    cert: &ModuleCert,
    imports: &[&VerifiedModule],
    import: &VerifiedModule,
    axiom: &AxiomRef,
) -> Result<AxiomRef> {
    let axiom_name = import
        .name_table
        .get(axiom.name)
        .ok_or(CertError::DecodeError)?;
    let name = cert
        .name_table
        .iter()
        .position(|candidate| candidate == axiom_name)
        .ok_or(CertError::DecodeError)?;
    let import_index =
        import_index_exporting_axiom(imports, axiom_name, axiom.decl_interface_hash)?;
    Ok(AxiomRef {
        global_ref: GlobalRef::Imported {
            import_index,
            name,
            decl_interface_hash: axiom.decl_interface_hash,
        },
        name,
        decl_interface_hash: axiom.decl_interface_hash,
    })
}

fn import_index_exporting_axiom(
    imports: &[&VerifiedModule],
    axiom_name: &Name,
    decl_interface_hash: Hash,
) -> Result<usize> {
    imports
        .iter()
        .enumerate()
        .find_map(|(import_index, import)| {
            import
                .export_block
                .iter()
                .any(|entry| {
                    entry.kind == ExportKind::Axiom
                        && entry.decl_interface_hash == decl_interface_hash
                        && import
                            .name_table
                            .get(entry.name)
                            .is_some_and(|name| name == axiom_name)
                })
                .then_some(import_index)
        })
        .ok_or_else(|| CertError::UnknownDependency {
            name: axiom_name.clone(),
        })
}

fn local_axiom_ref_for_decl(decl_index: usize, dep_axioms: &[AxiomRef]) -> Option<AxiomRef> {
    dep_axioms
        .iter()
        .find(|axiom| {
            matches!(
                axiom.global_ref,
                GlobalRef::Local { decl_index: axiom_index } if axiom_index == decl_index
            )
        })
        .cloned()
}

fn decl_term_ids(decl: &DeclPayload) -> Vec<TermId> {
    match decl {
        DeclPayload::Axiom { ty, .. } => vec![*ty],
        DeclPayload::Def { ty, value, .. } => vec![*ty, *value],
        DeclPayload::Theorem { ty, proof, .. } => vec![*ty, *proof],
        DeclPayload::Inductive {
            params,
            indices,
            constructors,
            recursor,
            ..
        } => params
            .iter()
            .map(|param| param.ty)
            .chain(indices.iter().map(|index| index.ty))
            .chain(constructors.iter().map(|constructor| constructor.ty))
            .chain(recursor.iter().map(|recursor| recursor.ty))
            .collect(),
    }
}

fn collect_global_refs_from_term(
    cert: &ModuleCert,
    term: TermId,
    refs: &mut BTreeSet<GlobalRef>,
) -> Result<()> {
    match cert.term_table.get(term).ok_or(CertError::DecodeError)? {
        TermNode::Sort(_) | TermNode::BVar(_) => {}
        TermNode::Const { global_ref, .. } => {
            refs.insert(global_ref.clone());
        }
        TermNode::App(fun, arg) => {
            collect_global_refs_from_term(cert, *fun, refs)?;
            collect_global_refs_from_term(cert, *arg, refs)?;
        }
        TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => {
            collect_global_refs_from_term(cert, *ty, refs)?;
            collect_global_refs_from_term(cert, *body, refs)?;
        }
        TermNode::Let { ty, value, body } => {
            collect_global_refs_from_term(cert, *ty, refs)?;
            collect_global_refs_from_term(cert, *value, refs)?;
            collect_global_refs_from_term(cert, *body, refs)?;
        }
    }
    Ok(())
}

fn interface_hash_for_global_ref(
    cert: &ModuleCert,
    imports: &[&VerifiedModule],
    current_decl_index: usize,
    global_ref: &GlobalRef,
) -> Result<Hash> {
    match global_ref {
        GlobalRef::Local { decl_index } => {
            if *decl_index >= current_decl_index {
                return Err(CertError::DependencyCycle {
                    name: Name::from_dotted(format!("local.{decl_index}")),
                });
            }
            Ok(cert
                .declarations
                .get(*decl_index)
                .ok_or(CertError::DecodeError)?
                .hashes
                .decl_interface_hash)
        }
        GlobalRef::LocalGenerated { decl_index, name } => {
            if *decl_index >= current_decl_index {
                return Err(CertError::DependencyCycle {
                    name: cert
                        .name_table
                        .get(*name)
                        .cloned()
                        .unwrap_or_else(|| Name::from_dotted(format!("local.{decl_index}"))),
                });
            }
            if !local_generated_entry_exists(cert, *decl_index, *name)? {
                return Err(CertError::UnknownDependency {
                    name: cert
                        .name_table
                        .get(*name)
                        .cloned()
                        .ok_or(CertError::DecodeError)?,
                });
            }
            Ok(cert
                .declarations
                .get(*decl_index)
                .ok_or(CertError::DecodeError)?
                .hashes
                .decl_interface_hash)
        }
        GlobalRef::Imported {
            decl_interface_hash,
            ..
        } => {
            let entry = imported_export_entry_for_global_ref(cert, imports, global_ref)?;
            if entry.decl_interface_hash != *decl_interface_hash {
                return Err(CertError::ImportHashMismatch {
                    module: imported_module_name_for_global_ref(imports, global_ref)?,
                });
            }
            Ok(*decl_interface_hash)
        }
    }
}

fn local_generated_entry_exists(
    cert: &ModuleCert,
    decl_index: usize,
    name: NameId,
) -> Result<bool> {
    let decl = cert
        .declarations
        .get(decl_index)
        .ok_or(CertError::DecodeError)?;
    Ok(match &decl.decl {
        DeclPayload::Inductive {
            constructors,
            recursor,
            ..
        } => {
            constructors
                .iter()
                .any(|constructor| constructor.name == name)
                || recursor
                    .as_ref()
                    .is_some_and(|recursor| recursor.name == name)
        }
        _ => false,
    })
}

fn imported_export_entry_for_global_ref<'a>(
    cert: &ModuleCert,
    imports: &'a [&'a VerifiedModule],
    global_ref: &GlobalRef,
) -> Result<&'a ExportEntry> {
    let GlobalRef::Imported {
        import_index,
        name,
        decl_interface_hash,
    } = global_ref
    else {
        return Err(CertError::DecodeError);
    };
    let imported = imports.get(*import_index).ok_or(CertError::DecodeError)?;
    let wanted_name = cert.name_table.get(*name).ok_or(CertError::DecodeError)?;
    imported
        .export_block
        .iter()
        .find(|entry| {
            imported
                .name_table
                .get(entry.name)
                .is_some_and(|candidate| candidate == wanted_name)
                && entry.decl_interface_hash == *decl_interface_hash
        })
        .ok_or_else(|| CertError::ImportHashMismatch {
            module: imported.module.clone(),
        })
}

fn imported_module_name_for_global_ref(
    imports: &[&VerifiedModule],
    global_ref: &GlobalRef,
) -> Result<ModuleName> {
    let GlobalRef::Imported { import_index, .. } = global_ref else {
        return Err(CertError::DecodeError);
    };
    Ok(imports
        .get(*import_index)
        .ok_or(CertError::DecodeError)?
        .module
        .clone())
}

fn decl_name_as_name(cert: &ModuleCert, decl_index: usize) -> Result<Name> {
    let decl = cert
        .declarations
        .get(decl_index)
        .ok_or(CertError::DecodeError)?;
    let name = match &decl.decl {
        DeclPayload::Axiom { name, .. }
        | DeclPayload::Def { name, .. }
        | DeclPayload::Theorem { name, .. }
        | DeclPayload::Inductive { name, .. } => *name,
    };
    cert.name_table
        .get(name)
        .cloned()
        .ok_or(CertError::DecodeError)
}

fn enforce_axiom_policy(cert: &ModuleCert, policy: &AxiomPolicy) -> Result<()> {
    enforce_axiom_policy_for_report(&cert.name_table, &cert.axiom_report, policy)
}

fn enforce_import_axiom_policy(imports: &[&VerifiedModule], policy: &AxiomPolicy) -> Result<()> {
    for import in imports {
        enforce_axiom_policy_for_report(&import.name_table, &import.axiom_report, policy)?;
    }
    Ok(())
}

fn enforce_axiom_policy_for_report(
    name_table: &[Name],
    axiom_report: &AxiomReport,
    policy: &AxiomPolicy,
) -> Result<()> {
    for axiom in &axiom_report.module_axioms {
        let name = name_table.get(axiom.name).ok_or(CertError::DecodeError)?;
        let dotted = name.as_dotted();
        if policy.deny_sorry && dotted.contains("sorry") {
            return Err(CertError::SorryDenied {
                axiom: name.clone(),
            });
        }
        if policy.mode == TrustMode::HighTrust && !policy.allowlisted_axioms.contains(name) {
            return Err(CertError::ForbiddenAxiom {
                axiom: name.clone(),
            });
        }
    }
    Ok(())
}
