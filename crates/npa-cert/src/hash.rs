use sha2::{Digest, Sha256};

use crate::*;

pub(crate) fn term_hash_impl(cert: &ModuleCert, term: TermId) -> Result<Hash> {
    let level_hashes = compute_level_hashes(&cert.level_table, &cert.name_table)?;
    let term_hashes = compute_term_hashes(&cert.term_table, &level_hashes)?;
    term_hashes.get(term).copied().ok_or(CertError::DecodeError)
}
pub(crate) fn build_export_block(
    declarations: &[DeclCert],
    term_table: &[TermNode],
    term_hashes: &[Hash],
) -> Result<ExportBlock> {
    let mut entries = Vec::new();
    for decl in declarations {
        match &decl.decl {
            DeclPayload::Axiom {
                name,
                universe_params,
                ty,
            } => entries.push(ExportEntry {
                name: *name,
                kind: ExportKind::Axiom,
                universe_params: universe_params.clone(),
                ty: *ty,
                body: None,
                type_hash: term_hashes[*ty],
                body_hash: None,
                reducibility: None,
                opacity: None,
                decl_interface_hash: decl.hashes.decl_interface_hash,
                axiom_dependencies: decl.axiom_dependencies.clone(),
            }),
            DeclPayload::Def {
                name,
                universe_params,
                ty,
                value,
                reducibility,
            } => entries.push(ExportEntry {
                name: *name,
                kind: ExportKind::Def,
                universe_params: universe_params.clone(),
                ty: *ty,
                body: (*reducibility == CertReducibility::Reducible).then_some(*value),
                type_hash: term_hashes[*ty],
                body_hash: (*reducibility == CertReducibility::Reducible)
                    .then_some(term_hashes[*value]),
                reducibility: Some(*reducibility),
                opacity: None,
                decl_interface_hash: decl.hashes.decl_interface_hash,
                axiom_dependencies: decl.axiom_dependencies.clone(),
            }),
            DeclPayload::Theorem {
                name,
                universe_params,
                ty,
                ..
            } => entries.push(ExportEntry {
                name: *name,
                kind: ExportKind::Theorem,
                universe_params: universe_params.clone(),
                ty: *ty,
                body: None,
                type_hash: term_hashes[*ty],
                body_hash: None,
                reducibility: None,
                opacity: Some(Opacity::Opaque),
                decl_interface_hash: decl.hashes.decl_interface_hash,
                axiom_dependencies: decl.axiom_dependencies.clone(),
            }),
            DeclPayload::Inductive {
                name,
                universe_params,
                params,
                indices,
                sort,
                constructors,
                recursor,
                ..
            } => {
                let ty = inductive_export_type_term_id(term_table, params, indices, *sort)?;
                entries.push(ExportEntry {
                    name: *name,
                    kind: ExportKind::Inductive,
                    universe_params: universe_params.clone(),
                    ty,
                    body: None,
                    type_hash: term_hashes[ty],
                    body_hash: None,
                    reducibility: None,
                    opacity: None,
                    decl_interface_hash: decl.hashes.decl_interface_hash,
                    axiom_dependencies: decl.axiom_dependencies.clone(),
                });
                for constructor in constructors {
                    entries.push(ExportEntry {
                        name: constructor.name,
                        kind: ExportKind::Constructor,
                        universe_params: universe_params.clone(),
                        ty: constructor.ty,
                        body: None,
                        type_hash: term_hashes[constructor.ty],
                        body_hash: None,
                        reducibility: None,
                        opacity: None,
                        decl_interface_hash: decl.hashes.decl_interface_hash,
                        axiom_dependencies: decl.axiom_dependencies.clone(),
                    });
                }
                if let Some(recursor) = recursor {
                    entries.push(ExportEntry {
                        name: recursor.name,
                        kind: ExportKind::Recursor,
                        universe_params: recursor.universe_params.clone(),
                        ty: recursor.ty,
                        body: None,
                        type_hash: term_hashes[recursor.ty],
                        body_hash: None,
                        reducibility: None,
                        opacity: None,
                        decl_interface_hash: decl.hashes.decl_interface_hash,
                        axiom_dependencies: decl.axiom_dependencies.clone(),
                    });
                }
            }
        }
    }
    entries.sort_by_key(|entry| entry.name);
    Ok(entries)
}

pub(crate) fn inductive_export_type_term_id(
    term_table: &[TermNode],
    params: &[BinderType],
    indices: &[BinderType],
    sort: LevelId,
) -> Result<TermId> {
    let mut body = term_table
        .iter()
        .position(|term| matches!(term, TermNode::Sort(level) if *level == sort))
        .ok_or(CertError::DecodeError)?;
    for binder in params.iter().chain(indices).rev() {
        body = term_table
            .iter()
            .position(|term| {
                matches!(
                    term,
                    TermNode::Pi { ty, body: pi_body } if *ty == binder.ty && *pi_body == body
                )
            })
            .ok_or(CertError::DecodeError)?;
    }
    Ok(body)
}

pub(crate) fn compute_decl_hashes(
    decl: &DeclPayload,
    dependencies: &[DependencyEntry],
    axiom_dependencies: &[AxiomRef],
    term_table: &[TermNode],
    level_hashes: &[Hash],
    term_hashes: &[Hash],
    names: &[Name],
) -> Result<DeclHashes> {
    let interface_dependencies = interface_dependencies_for_decl(decl, dependencies, term_table)?;
    let iface = hash_with_domain(
        b"NPA-DECL-IFACE-0.1",
        &decl_interface_payload(
            decl,
            &interface_dependencies,
            axiom_dependencies,
            level_hashes,
            term_hashes,
            names,
        )?,
    );
    let cert = hash_with_domain(
        b"NPA-DECL-CERT-0.1",
        &decl_certificate_payload(decl, iface, dependencies, axiom_dependencies, term_hashes)?,
    );
    Ok(DeclHashes {
        decl_interface_hash: iface,
        decl_certificate_hash: cert,
    })
}

fn decl_interface_payload(
    decl: &DeclPayload,
    interface_dependencies: &[DependencyEntry],
    axiom_dependencies: &[AxiomRef],
    level_hashes: &[Hash],
    term_hashes: &[Hash],
    names: &[Name],
) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    match decl {
        DeclPayload::Axiom {
            name,
            universe_params,
            ty,
        } => {
            out.push(0x00);
            encode_name_id_to(&mut out, names, *name)?;
            encode_name_ids_to(&mut out, names, universe_params)?;
            out.extend(term_hashes.get(*ty).ok_or(CertError::DecodeError)?);
            encode_dependency_entries_to(&mut out, interface_dependencies);
        }
        DeclPayload::Def {
            name,
            universe_params,
            ty,
            value,
            reducibility,
        } => {
            out.push(0x01);
            encode_name_id_to(&mut out, names, *name)?;
            encode_name_ids_to(&mut out, names, universe_params)?;
            out.extend(term_hashes.get(*ty).ok_or(CertError::DecodeError)?);
            if *reducibility == CertReducibility::Reducible {
                out.extend(term_hashes.get(*value).ok_or(CertError::DecodeError)?);
            }
            encode_reducibility_to(&mut out, *reducibility);
            encode_dependency_entries_to(&mut out, interface_dependencies);
            encode_axiom_refs_to(&mut out, axiom_dependencies);
        }
        DeclPayload::Theorem {
            name,
            universe_params,
            ty,
            opacity,
            ..
        } => {
            out.push(0x02);
            encode_name_id_to(&mut out, names, *name)?;
            encode_name_ids_to(&mut out, names, universe_params)?;
            out.extend(term_hashes.get(*ty).ok_or(CertError::DecodeError)?);
            encode_opacity_to(&mut out, *opacity);
            encode_dependency_entries_to(&mut out, interface_dependencies);
            encode_axiom_refs_to(&mut out, axiom_dependencies);
        }
        DeclPayload::Inductive {
            name,
            universe_params,
            params,
            indices,
            sort,
            constructors,
            recursor,
        } => {
            out.push(0x03);
            encode_name_id_to(&mut out, names, *name)?;
            encode_name_ids_to(&mut out, names, universe_params)?;
            encode_uvar_to(&mut out, params.len() as u64);
            for param in params {
                out.extend(term_hashes.get(param.ty).ok_or(CertError::DecodeError)?);
            }
            encode_uvar_to(&mut out, indices.len() as u64);
            for index in indices {
                out.extend(term_hashes.get(index.ty).ok_or(CertError::DecodeError)?);
            }
            out.extend(level_hashes.get(*sort).ok_or(CertError::DecodeError)?);
            encode_uvar_to(&mut out, constructors.len() as u64);
            for constructor in constructors {
                encode_name_id_to(&mut out, names, constructor.name)?;
                out.extend(
                    term_hashes
                        .get(constructor.ty)
                        .ok_or(CertError::DecodeError)?,
                );
            }
            match recursor {
                Some(recursor) => {
                    out.push(0x01);
                    encode_name_id_to(&mut out, names, recursor.name)?;
                    encode_name_ids_to(&mut out, names, &recursor.universe_params)?;
                    out.extend(term_hashes.get(recursor.ty).ok_or(CertError::DecodeError)?);
                    encode_uvar_to(&mut out, recursor.rules.minor_start as u64);
                    encode_uvar_to(&mut out, recursor.rules.major_index as u64);
                }
                None => out.push(0x00),
            }
            encode_dependency_entries_to(&mut out, interface_dependencies);
            encode_axiom_refs_to(&mut out, axiom_dependencies);
        }
    }
    Ok(out)
}

fn interface_dependencies_for_decl(
    decl: &DeclPayload,
    dependencies: &[DependencyEntry],
    term_table: &[TermNode],
) -> Result<Vec<DependencyEntry>> {
    let mut refs = std::collections::BTreeSet::new();
    for term in interface_term_ids(decl) {
        collect_global_refs_from_term(term_table, term, &mut refs)?;
    }
    Ok(dependencies
        .iter()
        .filter(|dependency| refs.contains(&dependency.global_ref))
        .cloned()
        .collect())
}

fn interface_term_ids(decl: &DeclPayload) -> Vec<TermId> {
    match decl {
        DeclPayload::Axiom { ty, .. } => vec![*ty],
        DeclPayload::Def {
            ty,
            value,
            reducibility,
            ..
        } => {
            let mut terms = vec![*ty];
            if *reducibility == CertReducibility::Reducible {
                terms.push(*value);
            }
            terms
        }
        DeclPayload::Theorem { ty, .. } => vec![*ty],
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
    terms: &[TermNode],
    term: TermId,
    refs: &mut std::collections::BTreeSet<GlobalRef>,
) -> Result<()> {
    match terms.get(term).ok_or(CertError::DecodeError)? {
        TermNode::Sort(_) | TermNode::BVar(_) => {}
        TermNode::Const { global_ref, .. } => {
            refs.insert(global_ref.clone());
        }
        TermNode::App(fun, arg) => {
            collect_global_refs_from_term(terms, *fun, refs)?;
            collect_global_refs_from_term(terms, *arg, refs)?;
        }
        TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => {
            collect_global_refs_from_term(terms, *ty, refs)?;
            collect_global_refs_from_term(terms, *body, refs)?;
        }
        TermNode::Let { ty, value, body } => {
            collect_global_refs_from_term(terms, *ty, refs)?;
            collect_global_refs_from_term(terms, *value, refs)?;
            collect_global_refs_from_term(terms, *body, refs)?;
        }
    }
    Ok(())
}

fn decl_certificate_payload(
    decl: &DeclPayload,
    interface_hash: Hash,
    dependencies: &[DependencyEntry],
    axiom_dependencies: &[AxiomRef],
    term_hashes: &[Hash],
) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    out.extend(interface_hash);
    match decl {
        DeclPayload::Axiom { .. } => encode_axiom_refs_to(&mut out, axiom_dependencies),
        DeclPayload::Def {
            value,
            reducibility: CertReducibility::Opaque,
            ..
        } => {
            out.extend(term_hashes.get(*value).ok_or(CertError::DecodeError)?);
            encode_dependency_entries_to(&mut out, dependencies);
            encode_axiom_refs_to(&mut out, axiom_dependencies);
        }
        DeclPayload::Def { .. } | DeclPayload::Inductive { .. } => {
            encode_dependency_entries_to(&mut out, dependencies);
            encode_axiom_refs_to(&mut out, axiom_dependencies);
        }
        DeclPayload::Theorem { proof, .. } => {
            out.extend(term_hashes.get(*proof).ok_or(CertError::DecodeError)?);
            encode_dependency_entries_to(&mut out, dependencies);
        }
    }
    Ok(out)
}

fn encode_name_id_to(out: &mut Vec<u8>, names: &[Name], name: NameId) -> Result<()> {
    encode_name_to(out, names.get(name).ok_or(CertError::DecodeError)?);
    Ok(())
}

fn encode_name_ids_to(out: &mut Vec<u8>, names: &[Name], values: &[NameId]) -> Result<()> {
    encode_uvar_to(out, values.len() as u64);
    for value in values {
        encode_name_id_to(out, names, *value)?;
    }
    Ok(())
}

pub(crate) fn compute_level_hashes(levels: &[LevelNode], names: &[Name]) -> Result<Vec<Hash>> {
    let mut hashes = Vec::with_capacity(levels.len());
    for level in levels {
        let payload = level_node_key(level, &hashes, names)?;
        hashes.push(hash_with_domain(b"NPA-LEVEL-0.1", &payload));
    }
    Ok(hashes)
}

pub(crate) fn compute_term_hashes(terms: &[TermNode], level_hashes: &[Hash]) -> Result<Vec<Hash>> {
    let mut hashes = Vec::with_capacity(terms.len());
    for term in terms {
        let payload = term_node_key(term, &hashes, level_hashes)?;
        hashes.push(hash_with_domain(b"NPA-TERM-0.1", &payload));
    }
    Ok(hashes)
}

pub(crate) fn level_node_key(
    level: &LevelNode,
    child_hashes: &[Hash],
    names: &[Name],
) -> Result<Vec<u8>> {
    let mut payload = Vec::new();
    match level {
        LevelNode::Zero => payload.push(0x00),
        LevelNode::Succ(inner) => {
            payload.push(0x01);
            payload.extend(child_hashes.get(*inner).ok_or(CertError::DecodeError)?);
        }
        LevelNode::Max(lhs, rhs) => {
            payload.push(0x02);
            payload.extend(child_hashes.get(*lhs).ok_or(CertError::DecodeError)?);
            payload.extend(child_hashes.get(*rhs).ok_or(CertError::DecodeError)?);
        }
        LevelNode::IMax(lhs, rhs) => {
            payload.push(0x03);
            payload.extend(child_hashes.get(*lhs).ok_or(CertError::DecodeError)?);
            payload.extend(child_hashes.get(*rhs).ok_or(CertError::DecodeError)?);
        }
        LevelNode::Param(name) => {
            payload.push(0x04);
            encode_name_to(
                &mut payload,
                names.get(*name).ok_or(CertError::DecodeError)?,
            );
        }
    }
    Ok(payload)
}

pub(crate) fn term_node_key(
    term: &TermNode,
    child_hashes: &[Hash],
    level_hashes: &[Hash],
) -> Result<Vec<u8>> {
    let mut payload = Vec::new();
    match term {
        TermNode::Sort(level) => {
            payload.push(0x00);
            payload.extend(level_hashes.get(*level).ok_or(CertError::DecodeError)?);
        }
        TermNode::BVar(index) => {
            payload.push(0x01);
            encode_uvar_to(&mut payload, *index as u64);
        }
        TermNode::Const { global_ref, levels } => {
            payload.push(0x02);
            encode_global_ref_to(&mut payload, global_ref);
            encode_uvar_to(&mut payload, levels.len() as u64);
            for level in levels {
                payload.extend(level_hashes.get(*level).ok_or(CertError::DecodeError)?);
            }
        }
        TermNode::App(fun, arg) => {
            payload.push(0x03);
            payload.extend(child_hashes.get(*fun).ok_or(CertError::DecodeError)?);
            payload.extend(child_hashes.get(*arg).ok_or(CertError::DecodeError)?);
        }
        TermNode::Lam { ty, body } => {
            payload.push(0x04);
            payload.extend(child_hashes.get(*ty).ok_or(CertError::DecodeError)?);
            payload.extend(child_hashes.get(*body).ok_or(CertError::DecodeError)?);
        }
        TermNode::Pi { ty, body } => {
            payload.push(0x05);
            payload.extend(child_hashes.get(*ty).ok_or(CertError::DecodeError)?);
            payload.extend(child_hashes.get(*body).ok_or(CertError::DecodeError)?);
        }
        TermNode::Let { ty, value, body } => {
            payload.push(0x06);
            payload.extend(child_hashes.get(*ty).ok_or(CertError::DecodeError)?);
            payload.extend(child_hashes.get(*value).ok_or(CertError::DecodeError)?);
            payload.extend(child_hashes.get(*body).ok_or(CertError::DecodeError)?);
        }
    }
    Ok(payload)
}

pub(crate) fn canon_level_key(level: &CanonLevel, names: &[Name]) -> Result<Vec<u8>> {
    let mut payload = Vec::new();
    match level {
        CanonLevel::Zero => payload.push(0x00),
        CanonLevel::Succ(inner) => {
            payload.push(0x01);
            payload.extend(canon_level_hash(inner, names)?);
        }
        CanonLevel::Max(lhs, rhs) => {
            payload.push(0x02);
            payload.extend(canon_level_hash(lhs, names)?);
            payload.extend(canon_level_hash(rhs, names)?);
        }
        CanonLevel::IMax(lhs, rhs) => {
            payload.push(0x03);
            payload.extend(canon_level_hash(lhs, names)?);
            payload.extend(canon_level_hash(rhs, names)?);
        }
        CanonLevel::Param(name) => {
            payload.push(0x04);
            encode_name_to(
                &mut payload,
                names.get(*name).ok_or(CertError::DecodeError)?,
            );
        }
    }
    Ok(payload)
}

pub(crate) fn canon_level_hash(level: &CanonLevel, names: &[Name]) -> Result<Hash> {
    Ok(hash_with_domain(
        b"NPA-LEVEL-0.1",
        &canon_level_key(level, names)?,
    ))
}

pub(crate) fn canon_term_key(term: &CanonTerm, names: &[Name]) -> Result<Vec<u8>> {
    let mut payload = Vec::new();
    match term {
        CanonTerm::Sort(level) => {
            payload.push(0x00);
            payload.extend(canon_level_hash(level, names)?);
        }
        CanonTerm::BVar(index) => {
            payload.push(0x01);
            encode_uvar_to(&mut payload, *index as u64);
        }
        CanonTerm::Const { global_ref, levels } => {
            payload.push(0x02);
            encode_global_ref_to(&mut payload, global_ref);
            encode_uvar_to(&mut payload, levels.len() as u64);
            for level in levels {
                payload.extend(canon_level_hash(level, names)?);
            }
        }
        CanonTerm::App(fun, arg) => {
            payload.push(0x03);
            payload.extend(canon_term_hash(fun, names)?);
            payload.extend(canon_term_hash(arg, names)?);
        }
        CanonTerm::Lam { ty, body } => {
            payload.push(0x04);
            payload.extend(canon_term_hash(ty, names)?);
            payload.extend(canon_term_hash(body, names)?);
        }
        CanonTerm::Pi { ty, body } => {
            payload.push(0x05);
            payload.extend(canon_term_hash(ty, names)?);
            payload.extend(canon_term_hash(body, names)?);
        }
        CanonTerm::Let { ty, value, body } => {
            payload.push(0x06);
            payload.extend(canon_term_hash(ty, names)?);
            payload.extend(canon_term_hash(value, names)?);
            payload.extend(canon_term_hash(body, names)?);
        }
    }
    Ok(payload)
}

pub(crate) fn canon_term_hash(term: &CanonTerm, names: &[Name]) -> Result<Hash> {
    let payload = canon_term_key(term, names)?;
    Ok(hash_with_domain(b"NPA-TERM-0.1", &payload))
}

pub(crate) fn level_height(level: &CanonLevel) -> usize {
    match level {
        CanonLevel::Zero | CanonLevel::Param(_) => 0,
        CanonLevel::Succ(inner) => level_height(inner) + 1,
        CanonLevel::Max(lhs, rhs) | CanonLevel::IMax(lhs, rhs) => {
            level_height(lhs).max(level_height(rhs)) + 1
        }
    }
}

pub(crate) fn term_height(term: &CanonTerm) -> usize {
    match term {
        CanonTerm::Sort(_) | CanonTerm::BVar(_) | CanonTerm::Const { .. } => 0,
        CanonTerm::App(fun, arg) => term_height(fun).max(term_height(arg)) + 1,
        CanonTerm::Lam { ty, body } | CanonTerm::Pi { ty, body } => {
            term_height(ty).max(term_height(body)) + 1
        }
        CanonTerm::Let { ty, value, body } => {
            term_height(ty)
                .max(term_height(value))
                .max(term_height(body))
                + 1
        }
    }
}
pub(crate) fn hash_with_domain(domain: &[u8], payload: &[u8]) -> Hash {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(payload);
    hasher.finalize().into()
}
