use std::collections::{BTreeMap, BTreeSet};

use npa_kernel::{Decl, Env, Expr, Level};

use crate::*;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CanonLevel {
    Zero,
    Succ(Box<CanonLevel>),
    Max(Box<CanonLevel>, Box<CanonLevel>),
    IMax(Box<CanonLevel>, Box<CanonLevel>),
    Param(NameId),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum CanonTerm {
    Sort(CanonLevel),
    BVar(u32),
    Const {
        global_ref: GlobalRef,
        levels: Vec<CanonLevel>,
    },
    App(Box<CanonTerm>, Box<CanonTerm>),
    Lam {
        ty: Box<CanonTerm>,
        body: Box<CanonTerm>,
    },
    Pi {
        ty: Box<CanonTerm>,
        body: Box<CanonTerm>,
    },
    Let {
        ty: Box<CanonTerm>,
        value: Box<CanonTerm>,
        body: Box<CanonTerm>,
    },
}

#[derive(Clone)]
pub(crate) struct CanonDecl {
    decl: CanonDeclPayload,
    dependencies: Vec<DependencyEntry>,
}

#[derive(Clone)]
pub(crate) enum CanonDeclPayload {
    Axiom {
        name: NameId,
        universe_params: Vec<NameId>,
        ty: CanonTerm,
    },
    Def {
        name: NameId,
        universe_params: Vec<NameId>,
        ty: CanonTerm,
        value: CanonTerm,
        reducibility: CertReducibility,
    },
    Theorem {
        name: NameId,
        universe_params: Vec<NameId>,
        ty: CanonTerm,
        proof: CanonTerm,
    },
    Inductive {
        name: NameId,
        universe_params: Vec<NameId>,
        params: Vec<CanonTerm>,
        indices: Vec<CanonTerm>,
        sort: CanonLevel,
        constructors: Vec<(NameId, CanonTerm)>,
        recursor: Option<(NameId, Vec<NameId>, CanonTerm, RecursorRulesSpec)>,
    },
}

pub(crate) fn build_module_cert_impl(
    module: CoreModule,
    imports: &[VerifiedModule],
) -> Result<ModuleCert> {
    let mut module = module;
    module.declarations = canonical_declaration_order(module.declarations)?;

    let mut imports: Vec<_> = imports.iter().collect();
    imports.sort_by_key(|module| {
        (
            module.module.clone(),
            module.export_hash,
            Some(module.certificate_hash),
        )
    });
    imports.dedup_by(|lhs, rhs| {
        lhs.module == rhs.module
            && lhs.export_hash == rhs.export_hash
            && lhs.certificate_hash == rhs.certificate_hash
    });

    let local_names: Vec<Name> = module
        .declarations
        .iter()
        .map(|decl| Name::from_dotted(decl.name()))
        .collect();
    let mut local_public_names = local_names.clone();
    let mut local_generated_name_to_index = BTreeMap::new();
    for (decl_index, decl) in module.declarations.iter().enumerate() {
        if let Decl::Inductive { data, .. } = decl {
            for constructor in &data.constructors {
                let name = Name::from_dotted(&constructor.name);
                local_generated_name_to_index.insert(name.clone(), decl_index);
                local_public_names.push(name);
            }
            if let Some(recursor) = &data.recursor {
                let name = Name::from_dotted(&recursor.name);
                local_generated_name_to_index.insert(name.clone(), decl_index);
                local_public_names.push(name);
            }
        }
    }
    ensure_unique_names(&local_public_names)?;
    let local_name_to_index: BTreeMap<_, _> = local_names
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, name)| (name, index))
        .collect();

    let mut names = BTreeSet::new();
    collect_name(&mut names, &module.name);
    for import in &imports {
        collect_name(&mut names, &import.module);
    }
    for decl in &module.declarations {
        collect_names_from_decl(&mut names, decl);
    }
    let directly_referenced_names =
        referenced_imported_export_names(&module.declarations, &imports, &local_public_names)?;
    collect_imported_axiom_names_for_referenced_exports(
        &mut names,
        &imports,
        &directly_referenced_names,
    )?;
    let name_table: Vec<_> = names.into_iter().collect();
    ensure_canonical_names(&name_table)?;
    let name_index: BTreeMap<_, _> = name_table
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, name)| (name, index))
        .collect();

    let imports_entries: Vec<_> = imports
        .iter()
        .map(|module| ImportEntry {
            module: module.module.clone(),
            export_hash: module.export_hash,
            certificate_hash: Some(module.certificate_hash),
        })
        .collect();
    let imported_decls = imported_decl_map(&imports, &name_index, &directly_referenced_names)?;
    let referenced_builtins =
        referenced_builtin_names(&module.declarations, &imports, &local_public_names)?;

    let mut env = Env::new();
    add_referenced_builtins_to_env(&mut env, &referenced_builtins)?;
    add_imports_to_env(&mut env, &imports)?;

    let mut canon_decls = Vec::new();
    for (decl_index, decl) in module.declarations.iter().cloned().enumerate() {
        add_decl_to_env(&mut env, decl.clone())?;
        let allow_self = matches!(decl, Decl::Inductive { .. } | Decl::Axiom { .. });
        let resolver = Resolver {
            current_decl_index: decl_index,
            allow_self,
            local_name_to_index: &local_name_to_index,
            local_generated_name_to_index: &local_generated_name_to_index,
            imported_decls: &imported_decls,
            name_index: &name_index,
        };
        let (canon_decl, _) = canonicalize_decl(decl.clone(), decl_index, &resolver, &[])?;
        canon_decls.push(canon_decl);
    }

    let mut levels = BTreeSet::new();
    let mut terms = BTreeSet::new();
    for decl in &canon_decls {
        collect_canon_decl_nodes(decl, &mut levels, &mut terms);
    }
    let (level_table, level_ids) = build_level_table(levels, &name_table)?;
    let (term_table, term_ids) = build_term_table(terms, &level_ids, &name_table)?;

    let level_hashes = compute_level_hashes(&level_table, &name_table)?;
    let term_hashes = compute_term_hashes(&term_table, &level_hashes)?;

    let payloads: Vec<_> = canon_decls
        .iter()
        .map(|canon_decl| materialize_decl_payload(&canon_decl.decl, &level_ids, &term_ids))
        .collect();
    let mut declarations: Vec<DeclCert> = Vec::new();
    let mut per_declaration = Vec::new();
    let mut previous_axioms: Vec<Vec<AxiomRef>> = Vec::new();
    let mut interface_hashes: Vec<Hash> = Vec::new();
    for (decl_index, (payload, canon_decl)) in payloads.iter().zip(&canon_decls).enumerate() {
        let dependencies =
            fill_local_dependency_hashes(&canon_decl.dependencies, &interface_hashes)?;
        let mut axiom_dependencies = axiom_dependencies_from_final_deps(
            &dependencies,
            &previous_axioms,
            &imported_decls,
            &name_table,
        )?;
        let mut direct_axioms = direct_axioms_from_final_deps(
            &dependencies,
            &previous_axioms,
            &imported_decls,
            &name_table,
        )?;

        if let DeclPayload::Axiom { name, .. } = payload {
            let preliminary = compute_decl_hashes(
                payload,
                &dependencies,
                &[],
                &term_table,
                &level_hashes,
                &term_hashes,
                &name_table,
            )?;
            let self_ref = AxiomRef {
                global_ref: GlobalRef::Local { decl_index },
                name: *name,
                decl_interface_hash: preliminary.decl_interface_hash,
            };
            axiom_dependencies =
                union_axioms(axiom_dependencies.into_iter().chain([self_ref.clone()]));
            direct_axioms = union_axioms(direct_axioms.into_iter().chain([self_ref]));
        }

        let hashes = compute_decl_hashes(
            payload,
            &dependencies,
            &axiom_dependencies,
            &term_table,
            &level_hashes,
            &term_hashes,
            &name_table,
        )?;
        interface_hashes.push(hashes.decl_interface_hash);
        previous_axioms.push(axiom_dependencies.clone());
        declarations.push(DeclCert {
            decl: payload.clone(),
            dependencies,
            axiom_dependencies: axiom_dependencies.clone(),
            hashes,
        });
        per_declaration.push(DeclAxiomReport {
            decl_index,
            direct_axioms,
            transitive_axioms: axiom_dependencies,
        });
    }

    let export_block = build_export_block(&declarations, &term_table, &term_hashes)?;
    let module_axioms = union_axioms(
        per_declaration
            .iter()
            .flat_map(|report| report.transitive_axioms.iter().cloned()),
    );
    let axiom_report = AxiomReport {
        per_declaration,
        module_axioms,
    };

    let export_hash = hash_with_domain(
        b"NPA-MODULE-EXPORT-0.1",
        &encode_export_block(&export_block),
    );
    let axiom_report_hash =
        hash_with_domain(b"NPA-AXIOM-REPORT-0.1", &encode_axiom_report(&axiom_report));

    let mut cert = ModuleCert {
        header: CertHeader {
            format: FORMAT.to_owned(),
            core_spec: CORE_SPEC.to_owned(),
            module: module.name,
        },
        imports: imports_entries,
        name_table,
        level_table,
        term_table,
        declarations,
        export_block,
        axiom_report,
        hashes: ModuleHashes {
            export_hash,
            axiom_report_hash,
            certificate_hash: [0; 32],
        },
    };
    cert.hashes.certificate_hash = hash_with_domain(
        b"NPA-MODULE-CERT-0.1",
        &encode_module_cert_without_certificate_hash(&cert),
    );
    Ok(cert)
}

fn canonicalize_decl(
    decl: Decl,
    decl_index: usize,
    resolver: &Resolver<'_>,
    previous_axioms: &[Vec<AxiomRef>],
) -> Result<(CanonDecl, Vec<AxiomRef>)> {
    match decl {
        Decl::Axiom {
            name,
            universe_params,
            ty,
        } => {
            let name_id = resolver.name_id(&Name::from_dotted(&name))?;
            let ty = canonicalize_expr(&ty, resolver)?;
            let deps = dependencies_from_terms([&ty]);
            let ax = axiom_dependencies_from_deps(&deps, previous_axioms, resolver);
            let self_ref = AxiomRef {
                global_ref: GlobalRef::Local { decl_index },
                name: name_id,
                decl_interface_hash: [0; 32],
            };
            let axiom_dependencies = union_axioms(ax.into_iter().chain([self_ref.clone()]));
            Ok((
                CanonDecl {
                    decl: CanonDeclPayload::Axiom {
                        name: name_id,
                        universe_params: universe_param_ids(&universe_params, resolver)?,
                        ty,
                    },
                    dependencies: deps,
                },
                axiom_dependencies,
            ))
        }
        Decl::Def {
            name,
            universe_params,
            ty,
            value,
            reducibility,
        } => {
            let ty = canonicalize_expr(&ty, resolver)?;
            let value = canonicalize_expr(&value, resolver)?;
            let deps = dependencies_from_terms([&ty, &value]);
            let ax = axiom_dependencies_from_deps(&deps, previous_axioms, resolver);
            Ok((
                CanonDecl {
                    decl: CanonDeclPayload::Def {
                        name: resolver.name_id(&Name::from_dotted(&name))?,
                        universe_params: universe_param_ids(&universe_params, resolver)?,
                        ty,
                        value,
                        reducibility: CertReducibility::from(&reducibility),
                    },
                    dependencies: deps,
                },
                ax,
            ))
        }
        Decl::Theorem {
            name,
            universe_params,
            ty,
            proof,
        } => {
            let ty = canonicalize_expr(&ty, resolver)?;
            let proof = canonicalize_expr(&proof, resolver)?;
            let deps = dependencies_from_terms([&ty, &proof]);
            let ax = axiom_dependencies_from_deps(&deps, previous_axioms, resolver);
            Ok((
                CanonDecl {
                    decl: CanonDeclPayload::Theorem {
                        name: resolver.name_id(&Name::from_dotted(&name))?,
                        universe_params: universe_param_ids(&universe_params, resolver)?,
                        ty,
                        proof,
                    },
                    dependencies: deps,
                },
                ax,
            ))
        }
        Decl::Inductive {
            name,
            universe_params,
            ty,
            data,
        } => {
            if name != data.name || universe_params != data.universe_params {
                return Err(CertError::InductiveWrapperMismatch {
                    name: Name::from_dotted(&name),
                });
            }
            let mut terms = Vec::new();
            let ty = canonicalize_expr(&ty, resolver)?;
            let params = data
                .params
                .iter()
                .map(|binder| canonicalize_expr(&binder.ty, resolver))
                .collect::<Result<Vec<_>>>()?;
            terms.extend(params.iter().cloned());
            let indices = data
                .indices
                .iter()
                .map(|binder| canonicalize_expr(&binder.ty, resolver))
                .collect::<Result<Vec<_>>>()?;
            terms.extend(indices.iter().cloned());
            let constructors = data
                .constructors
                .iter()
                .map(|constructor| {
                    let ty = canonicalize_expr(&constructor.ty, resolver)?;
                    terms.push(ty.clone());
                    Ok((resolver.name_id(&Name::from_dotted(&constructor.name))?, ty))
                })
                .collect::<Result<Vec<_>>>()?;
            let recursor = data
                .recursor
                .as_ref()
                .map(|recursor| {
                    let ty = canonicalize_expr(&recursor.ty, resolver)?;
                    terms.push(ty.clone());
                    Ok::<_, CertError>((
                        resolver.name_id(&Name::from_dotted(&recursor.name))?,
                        universe_param_ids(&recursor.universe_params, resolver)?,
                        ty,
                        recursor
                            .rules
                            .as_ref()
                            .map(|rules| RecursorRulesSpec {
                                minor_start: rules.minor_start,
                                major_index: rules.major_index,
                            })
                            .unwrap_or_else(|| RecursorRulesSpec {
                                minor_start: data.params.len() + 1,
                                major_index: data.params.len() + 1 + data.constructors.len(),
                            }),
                    ))
                })
                .transpose()?;
            let sort = canonicalize_level(&data.sort, resolver)?;
            if ty != inductive_type_canon_term(&params, &indices, &sort) {
                return Err(CertError::InductiveWrapperMismatch {
                    name: Name::from_dotted(&name),
                });
            }
            let mut deps = dependencies_from_terms(terms.iter());
            remove_self_dependency(&mut deps, decl_index);
            let ax = axiom_dependencies_from_deps(&deps, previous_axioms, resolver);
            Ok((
                CanonDecl {
                    decl: CanonDeclPayload::Inductive {
                        name: resolver.name_id(&Name::from_dotted(&name))?,
                        universe_params: universe_param_ids(&universe_params, resolver)?,
                        params,
                        indices,
                        sort,
                        constructors,
                        recursor,
                    },
                    dependencies: deps,
                },
                ax,
            ))
        }
        Decl::Constructor { name, .. } | Decl::Recursor { name, .. } => {
            Err(CertError::UnknownDependency {
                name: Name::from_dotted(name),
            })
        }
    }
}

pub(crate) fn canonical_producer_checked_decl_interface(
    decl: &Decl,
    lookup_env: &ProducerLookupEnv,
) -> Result<ProducerCheckedDeclInterface> {
    Ok(canonical_producer_checked_decl_hashes(decl, lookup_env)?.0)
}

pub(crate) fn canonical_producer_checked_decl_hashes(
    decl: &Decl,
    lookup_env: &ProducerLookupEnv,
) -> Result<(ProducerCheckedDeclInterface, DeclHashes)> {
    let current_decl_index = lookup_env.checked_decls.len();
    let mut names = BTreeSet::new();
    for import in &lookup_env.import_exports {
        collect_name(&mut names, &import.module);
    }
    collect_names_from_decl(&mut names, decl);
    let referenced_imports =
        producer_referenced_imported_export_names(decl, &lookup_env.import_exports)?;
    producer_collect_imported_axiom_names_for_referenced_exports(
        &mut names,
        &lookup_env.import_exports,
        &referenced_imports,
    )?;
    let name_table: Vec<_> = names.into_iter().collect();
    ensure_canonical_names(&name_table)?;
    let name_index: BTreeMap<_, _> = name_table
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, name)| (name, index))
        .collect();
    let imported_decls =
        producer_imported_decl_map(&lookup_env.import_exports, &name_index, &referenced_imports)?;
    let local_name_to_index: BTreeMap<_, _> = lookup_env
        .checked_decl_names
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, name)| (name, index))
        .collect();
    let resolver = Resolver {
        current_decl_index,
        allow_self: matches!(decl, Decl::Inductive { .. } | Decl::Axiom { .. }),
        local_name_to_index: &local_name_to_index,
        local_generated_name_to_index: &lookup_env.checked_generated_name_to_index,
        imported_decls: &imported_decls,
        name_index: &name_index,
    };
    let previous_axioms: Vec<_> = lookup_env
        .checked_decls
        .iter()
        .map(|interface| union_axioms(interface.axiom_dependencies.iter().cloned()))
        .collect();
    let (canon_decl, _) = canonicalize_decl(
        decl.clone(),
        current_decl_index,
        &resolver,
        &previous_axioms,
    )?;

    let mut levels = BTreeSet::new();
    let mut terms = BTreeSet::new();
    collect_canon_decl_nodes(&canon_decl, &mut levels, &mut terms);
    let (level_table, level_ids) = build_level_table(levels, &name_table)?;
    let (term_table, term_ids) = build_term_table(terms, &level_ids, &name_table)?;
    let level_hashes = compute_level_hashes(&level_table, &name_table)?;
    let term_hashes = compute_term_hashes(&term_table, &level_hashes)?;
    let payload = materialize_decl_payload(&canon_decl.decl, &level_ids, &term_ids);
    let interface_hashes: Vec<_> = lookup_env
        .checked_decls
        .iter()
        .map(|interface| interface.decl_interface_hash)
        .collect();
    let dependencies = fill_local_dependency_hashes(&canon_decl.dependencies, &interface_hashes)?;
    let mut axiom_dependencies = axiom_dependencies_from_final_deps(
        &dependencies,
        &previous_axioms,
        &imported_decls,
        &name_table,
    )?;

    if let DeclPayload::Axiom { name, .. } = &payload {
        let preliminary = compute_decl_hashes(
            &payload,
            &dependencies,
            &[],
            &term_table,
            &level_hashes,
            &term_hashes,
            &name_table,
        )?;
        let self_ref = AxiomRef {
            global_ref: GlobalRef::Local {
                decl_index: current_decl_index,
            },
            name: *name,
            decl_interface_hash: preliminary.decl_interface_hash,
        };
        axiom_dependencies = union_axioms(axiom_dependencies.into_iter().chain([self_ref]));
    }

    let hashes = compute_decl_hashes(
        &payload,
        &dependencies,
        &axiom_dependencies,
        &term_table,
        &level_hashes,
        &term_hashes,
        &name_table,
    )?;
    let interface = ProducerCheckedDeclInterface {
        decl_interface_hash: hashes.decl_interface_hash,
        axiom_dependencies,
    };
    Ok((interface, hashes))
}

struct Resolver<'a> {
    current_decl_index: usize,
    allow_self: bool,
    local_name_to_index: &'a BTreeMap<Name, usize>,
    local_generated_name_to_index: &'a BTreeMap<Name, usize>,
    imported_decls: &'a BTreeMap<Name, ImportedDeclInfo>,
    name_index: &'a BTreeMap<Name, usize>,
}

#[derive(Clone, Debug)]
struct ImportedDeclInfo {
    import_index: usize,
    decl_interface_hash: Hash,
    kind: ExportKind,
    axiom_dependencies: Vec<AxiomRef>,
}

pub(crate) fn canonical_declaration_order(declarations: Vec<Decl>) -> Result<Vec<Decl>> {
    let local_names: Vec<_> = declarations
        .iter()
        .map(|decl| Name::from_dotted(decl.name()))
        .collect();
    ensure_unique_names(&local_names)?;
    let local_name_to_index: BTreeMap<_, _> = local_names
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, name)| (name, index))
        .collect();

    let mut generated_name_to_index = BTreeMap::new();
    let mut public_names = local_names.clone();
    for (decl_index, decl) in declarations.iter().enumerate() {
        if let Decl::Inductive { data, .. } = decl {
            for constructor in &data.constructors {
                let name = Name::from_dotted(&constructor.name);
                generated_name_to_index.insert(name.clone(), decl_index);
                public_names.push(name);
            }
            if let Some(recursor) = &data.recursor {
                let name = Name::from_dotted(&recursor.name);
                generated_name_to_index.insert(name.clone(), decl_index);
                public_names.push(name);
            }
        }
    }
    ensure_unique_names(&public_names)?;

    let dependencies = declarations
        .iter()
        .enumerate()
        .map(|(decl_index, decl)| {
            let mut names = BTreeSet::new();
            collect_const_names_from_decl(&mut names, decl);
            Ok(names
                .into_iter()
                .filter_map(|name| {
                    local_name_to_index
                        .get(&name)
                        .or_else(|| generated_name_to_index.get(&name))
                        .copied()
                })
                .filter(|dependency| *dependency != decl_index)
                .collect::<BTreeSet<_>>())
        })
        .collect::<Result<Vec<_>>>()?;

    let mut emitted = BTreeSet::new();
    let mut remaining: BTreeSet<_> = (0..declarations.len()).collect();
    let mut ordered = Vec::with_capacity(declarations.len());
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
            ordered.push(declarations[index].clone());
        }
    }

    Ok(ordered)
}

impl Resolver<'_> {
    fn name_id(&self, name: &Name) -> Result<NameId> {
        self.name_index
            .get(name)
            .copied()
            .ok_or_else(|| CertError::UnknownDependency { name: name.clone() })
    }

    fn resolve_const(&self, name: &Name) -> Result<GlobalRef> {
        if let Some(index) = self.local_name_to_index.get(name).copied() {
            if index < self.current_decl_index
                || (self.allow_self && index == self.current_decl_index)
            {
                return Ok(GlobalRef::Local { decl_index: index });
            }
            return Err(CertError::DependencyCycle { name: name.clone() });
        }
        if let Some(index) = self.local_generated_name_to_index.get(name).copied() {
            if index < self.current_decl_index
                || (self.allow_self && index == self.current_decl_index)
            {
                return Ok(GlobalRef::LocalGenerated {
                    decl_index: index,
                    name: self.name_id(name)?,
                });
            }
            return Err(CertError::DependencyCycle { name: name.clone() });
        }
        if let Some(info) = self.imported_decls.get(name) {
            return Ok(GlobalRef::Imported {
                import_index: info.import_index,
                name: self.name_id(name)?,
                decl_interface_hash: info.decl_interface_hash,
            });
        }
        if let Some(decl_interface_hash) = builtin_decl_interface_hash(name) {
            return Ok(GlobalRef::Builtin {
                name: self.name_id(name)?,
                decl_interface_hash,
            });
        }
        Err(CertError::UnknownDependency { name: name.clone() })
    }
}

fn universe_param_ids(params: &[String], resolver: &Resolver<'_>) -> Result<Vec<NameId>> {
    params
        .iter()
        .map(|param| resolver.name_id(&Name::from_dotted(param)))
        .collect()
}

fn canonicalize_expr(expr: &Expr, resolver: &Resolver<'_>) -> Result<CanonTerm> {
    Ok(match expr {
        Expr::Sort(level) => CanonTerm::Sort(canonicalize_level(level, resolver)?),
        Expr::BVar(index) => CanonTerm::BVar(*index),
        Expr::Const { name, levels } => {
            let name = Name::from_dotted(name);
            CanonTerm::Const {
                global_ref: resolver.resolve_const(&name)?,
                levels: levels
                    .iter()
                    .map(|level| canonicalize_level(level, resolver))
                    .collect::<Result<Vec<_>>>()?,
            }
        }
        Expr::App(fun, arg) => CanonTerm::App(
            Box::new(canonicalize_expr(fun, resolver)?),
            Box::new(canonicalize_expr(arg, resolver)?),
        ),
        Expr::Lam { ty, body, .. } => CanonTerm::Lam {
            ty: Box::new(canonicalize_expr(ty, resolver)?),
            body: Box::new(canonicalize_expr(body, resolver)?),
        },
        Expr::Pi { ty, body, .. } => CanonTerm::Pi {
            ty: Box::new(canonicalize_expr(ty, resolver)?),
            body: Box::new(canonicalize_expr(body, resolver)?),
        },
        Expr::Let {
            ty, value, body, ..
        } => CanonTerm::Let {
            ty: Box::new(canonicalize_expr(ty, resolver)?),
            value: Box::new(canonicalize_expr(value, resolver)?),
            body: Box::new(canonicalize_expr(body, resolver)?),
        },
    })
}

fn canonicalize_level(level: &Level, resolver: &Resolver<'_>) -> Result<CanonLevel> {
    Ok(match npa_kernel::level::normalize_level(level.clone()) {
        Level::Zero => CanonLevel::Zero,
        Level::Succ(inner) => CanonLevel::Succ(Box::new(canonicalize_level(&inner, resolver)?)),
        Level::Max(lhs, rhs) => CanonLevel::Max(
            Box::new(canonicalize_level(&lhs, resolver)?),
            Box::new(canonicalize_level(&rhs, resolver)?),
        ),
        Level::IMax(lhs, rhs) => CanonLevel::IMax(
            Box::new(canonicalize_level(&lhs, resolver)?),
            Box::new(canonicalize_level(&rhs, resolver)?),
        ),
        Level::Param(name) => CanonLevel::Param(resolver.name_id(&Name::from_dotted(name))?),
    })
}

fn dependencies_from_terms<'a>(
    terms: impl IntoIterator<Item = &'a CanonTerm>,
) -> Vec<DependencyEntry> {
    let mut deps = BTreeSet::new();
    for term in terms {
        collect_dependencies(term, &mut deps);
    }
    deps.into_iter().collect()
}

fn remove_self_dependency(deps: &mut Vec<DependencyEntry>, current_decl_index: usize) {
    deps.retain(|dependency| {
        !matches!(
            dependency.global_ref,
            GlobalRef::Local { decl_index } | GlobalRef::LocalGenerated { decl_index, .. }
                if decl_index == current_decl_index
        )
    });
}

fn collect_dependencies(term: &CanonTerm, deps: &mut BTreeSet<DependencyEntry>) {
    match term {
        CanonTerm::Sort(_) | CanonTerm::BVar(_) => {}
        CanonTerm::Const { global_ref, .. } => {
            let decl_interface_hash = match global_ref {
                GlobalRef::Builtin {
                    decl_interface_hash,
                    ..
                } => *decl_interface_hash,
                GlobalRef::Imported {
                    decl_interface_hash,
                    ..
                } => *decl_interface_hash,
                GlobalRef::Local { .. } | GlobalRef::LocalGenerated { .. } => [0; 32],
            };
            deps.insert(DependencyEntry {
                global_ref: global_ref.clone(),
                decl_interface_hash,
            });
        }
        CanonTerm::App(fun, arg) => {
            collect_dependencies(fun, deps);
            collect_dependencies(arg, deps);
        }
        CanonTerm::Lam { ty, body } | CanonTerm::Pi { ty, body } => {
            collect_dependencies(ty, deps);
            collect_dependencies(body, deps);
        }
        CanonTerm::Let { ty, value, body } => {
            collect_dependencies(ty, deps);
            collect_dependencies(value, deps);
            collect_dependencies(body, deps);
        }
    }
}

fn axiom_dependencies_from_deps(
    deps: &[DependencyEntry],
    previous_axioms: &[Vec<AxiomRef>],
    resolver: &Resolver<'_>,
) -> Vec<AxiomRef> {
    let mut axioms = BTreeSet::new();
    for dep in deps {
        match &dep.global_ref {
            GlobalRef::Builtin {
                name,
                decl_interface_hash,
            } => {
                let is_builtin_axiom = resolver
                    .name_index
                    .iter()
                    .any(|(candidate, index)| *index == *name && builtin_is_axiom(candidate));
                if is_builtin_axiom {
                    axioms.insert(AxiomRef {
                        global_ref: dep.global_ref.clone(),
                        name: *name,
                        decl_interface_hash: *decl_interface_hash,
                    });
                }
            }
            GlobalRef::Local { decl_index } | GlobalRef::LocalGenerated { decl_index, .. } => {
                if let Some(dep_axioms) = previous_axioms.get(*decl_index) {
                    axioms.extend(dep_axioms.iter().cloned());
                }
            }
            GlobalRef::Imported { name, .. } => {
                let name = resolver
                    .name_index
                    .iter()
                    .find_map(|(candidate, index)| (*index == *name).then(|| candidate.clone()));
                if let Some(name) = name {
                    if let Some(info) = resolver.imported_decls.get(&name) {
                        axioms.extend(info.axiom_dependencies.iter().cloned());
                    }
                }
            }
        }
    }
    axioms.into_iter().collect()
}

pub(crate) fn fill_local_dependency_hashes(
    dependencies: &[DependencyEntry],
    interface_hashes: &[Hash],
) -> Result<Vec<DependencyEntry>> {
    dependencies
        .iter()
        .map(|dependency| {
            let decl_interface_hash = match &dependency.global_ref {
                GlobalRef::Local { decl_index } => {
                    *interface_hashes
                        .get(*decl_index)
                        .ok_or(CertError::DependencyCycle {
                            name: Name::from_dotted(format!("local.{decl_index}")),
                        })?
                }
                GlobalRef::LocalGenerated { decl_index, .. } => *interface_hashes
                    .get(*decl_index)
                    .ok_or(CertError::DependencyCycle {
                        name: Name::from_dotted(format!("local.{decl_index}")),
                    })?,
                GlobalRef::Imported {
                    decl_interface_hash,
                    ..
                } => *decl_interface_hash,
                GlobalRef::Builtin {
                    decl_interface_hash,
                    ..
                } => *decl_interface_hash,
            };
            Ok(DependencyEntry {
                global_ref: dependency.global_ref.clone(),
                decl_interface_hash,
            })
        })
        .collect()
}

fn axiom_dependencies_from_final_deps(
    dependencies: &[DependencyEntry],
    previous_axioms: &[Vec<AxiomRef>],
    imported_decls: &BTreeMap<Name, ImportedDeclInfo>,
    name_table: &[Name],
) -> Result<Vec<AxiomRef>> {
    let mut axioms = BTreeSet::new();
    for dependency in dependencies {
        match &dependency.global_ref {
            GlobalRef::Builtin {
                name,
                decl_interface_hash,
            } => {
                let name_value = name_table.get(*name).ok_or(CertError::DecodeError)?;
                if builtin_is_axiom(name_value) {
                    axioms.insert(AxiomRef {
                        global_ref: dependency.global_ref.clone(),
                        name: *name,
                        decl_interface_hash: *decl_interface_hash,
                    });
                }
            }
            GlobalRef::Local { decl_index } | GlobalRef::LocalGenerated { decl_index, .. } => {
                if let Some(dep_axioms) = previous_axioms.get(*decl_index) {
                    axioms.extend(dep_axioms.iter().cloned());
                }
            }
            GlobalRef::Imported {
                import_index,
                name,
                decl_interface_hash,
            } => {
                let name = name_table.get(*name).ok_or(CertError::DecodeError)?;
                let info = imported_decls
                    .get(name)
                    .filter(|info| {
                        info.import_index == *import_index
                            && info.decl_interface_hash == *decl_interface_hash
                    })
                    .ok_or_else(|| CertError::UnknownDependency { name: name.clone() })?;
                axioms.extend(info.axiom_dependencies.iter().cloned());
            }
        }
    }
    Ok(axioms.into_iter().collect())
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

fn direct_axioms_from_final_deps(
    dependencies: &[DependencyEntry],
    previous_axioms: &[Vec<AxiomRef>],
    imported_decls: &BTreeMap<Name, ImportedDeclInfo>,
    name_table: &[Name],
) -> Result<Vec<AxiomRef>> {
    let mut axioms = BTreeSet::new();
    for dependency in dependencies {
        match &dependency.global_ref {
            GlobalRef::Builtin {
                name,
                decl_interface_hash,
            } => {
                let name_value = name_table.get(*name).ok_or(CertError::DecodeError)?;
                if builtin_is_axiom(name_value) {
                    axioms.insert(AxiomRef {
                        global_ref: dependency.global_ref.clone(),
                        name: *name,
                        decl_interface_hash: *decl_interface_hash,
                    });
                }
            }
            GlobalRef::Local { decl_index } => {
                if let Some(axiom) = previous_axioms
                    .get(*decl_index)
                    .and_then(|dep_axioms| local_axiom_ref_for_decl(*decl_index, dep_axioms))
                {
                    axioms.insert(axiom);
                }
            }
            GlobalRef::LocalGenerated { .. } => {}
            GlobalRef::Imported {
                import_index,
                name,
                decl_interface_hash,
            } => {
                let imported_name = name_table.get(*name).ok_or(CertError::DecodeError)?;
                let info = imported_decls
                    .get(imported_name)
                    .filter(|info| {
                        info.import_index == *import_index
                            && info.decl_interface_hash == *decl_interface_hash
                    })
                    .ok_or_else(|| CertError::UnknownDependency {
                        name: imported_name.clone(),
                    })?;
                if info.kind == ExportKind::Axiom {
                    axioms.insert(AxiomRef {
                        global_ref: dependency.global_ref.clone(),
                        name: *name,
                        decl_interface_hash: *decl_interface_hash,
                    });
                }
            }
        }
    }
    Ok(axioms.into_iter().collect())
}

pub(crate) fn collect_canon_decl_nodes(
    decl: &CanonDecl,
    levels: &mut BTreeSet<CanonLevel>,
    terms: &mut BTreeSet<CanonTerm>,
) {
    match &decl.decl {
        CanonDeclPayload::Axiom { ty, .. } => collect_term_nodes(ty, levels, terms),
        CanonDeclPayload::Def { ty, value, .. } => {
            collect_term_nodes(ty, levels, terms);
            collect_term_nodes(value, levels, terms);
        }
        CanonDeclPayload::Theorem { ty, proof, .. } => {
            collect_term_nodes(ty, levels, terms);
            collect_term_nodes(proof, levels, terms);
        }
        CanonDeclPayload::Inductive {
            params,
            indices,
            sort,
            constructors,
            recursor,
            ..
        } => {
            collect_level_nodes(sort, levels);
            collect_term_nodes(
                &inductive_type_canon_term(params, indices, sort),
                levels,
                terms,
            );
            for term in params.iter().chain(indices) {
                collect_term_nodes(term, levels, terms);
            }
            for (_, term) in constructors {
                collect_term_nodes(term, levels, terms);
            }
            if let Some((_, _, ty, _)) = recursor {
                collect_term_nodes(ty, levels, terms);
            }
        }
    }
}

fn inductive_type_canon_term(
    params: &[CanonTerm],
    indices: &[CanonTerm],
    sort: &CanonLevel,
) -> CanonTerm {
    params
        .iter()
        .chain(indices)
        .rev()
        .fold(CanonTerm::Sort(sort.clone()), |body, ty| CanonTerm::Pi {
            ty: Box::new(ty.clone()),
            body: Box::new(body),
        })
}

fn collect_term_nodes(
    term: &CanonTerm,
    levels: &mut BTreeSet<CanonLevel>,
    terms: &mut BTreeSet<CanonTerm>,
) {
    terms.insert(term.clone());
    match term {
        CanonTerm::Sort(level) => collect_level_nodes(level, levels),
        CanonTerm::BVar(_) => {}
        CanonTerm::Const { levels: ls, .. } => {
            for level in ls {
                collect_level_nodes(level, levels);
            }
        }
        CanonTerm::App(fun, arg) => {
            collect_term_nodes(fun, levels, terms);
            collect_term_nodes(arg, levels, terms);
        }
        CanonTerm::Lam { ty, body } | CanonTerm::Pi { ty, body } => {
            collect_term_nodes(ty, levels, terms);
            collect_term_nodes(body, levels, terms);
        }
        CanonTerm::Let { ty, value, body } => {
            collect_term_nodes(ty, levels, terms);
            collect_term_nodes(value, levels, terms);
            collect_term_nodes(body, levels, terms);
        }
    }
}

fn collect_level_nodes(level: &CanonLevel, levels: &mut BTreeSet<CanonLevel>) {
    levels.insert(level.clone());
    match level {
        CanonLevel::Zero | CanonLevel::Param(_) => {}
        CanonLevel::Succ(inner) => collect_level_nodes(inner, levels),
        CanonLevel::Max(lhs, rhs) | CanonLevel::IMax(lhs, rhs) => {
            collect_level_nodes(lhs, levels);
            collect_level_nodes(rhs, levels);
        }
    }
}

pub(crate) fn build_level_table(
    levels: BTreeSet<CanonLevel>,
    names: &[Name],
) -> Result<(Vec<LevelNode>, BTreeMap<CanonLevel, LevelId>)> {
    let mut keyed_levels: Vec<_> = levels
        .into_iter()
        .map(|level| {
            Ok((
                (level_height(&level), canon_level_key(&level, names)?),
                level,
            ))
        })
        .collect::<Result<_>>()?;
    keyed_levels.sort_by(|(lhs_key, _), (rhs_key, _)| lhs_key.cmp(rhs_key));
    let levels: Vec<_> = keyed_levels.into_iter().map(|(_, level)| level).collect();
    let ids: BTreeMap<_, _> = levels
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, level)| (level, index))
        .collect();
    let table = levels
        .iter()
        .map(|level| materialize_level_node(level, &ids))
        .collect();
    Ok((table, ids))
}

pub(crate) fn build_term_table(
    terms: BTreeSet<CanonTerm>,
    level_ids: &BTreeMap<CanonLevel, LevelId>,
    names: &[Name],
) -> Result<(Vec<TermNode>, BTreeMap<CanonTerm, TermId>)> {
    let mut keyed_terms: Vec<_> = terms
        .into_iter()
        .map(|term| Ok(((term_height(&term), canon_term_key(&term, names)?), term)))
        .collect::<Result<_>>()?;
    keyed_terms.sort_by(|(lhs_key, _), (rhs_key, _)| lhs_key.cmp(rhs_key));
    let terms: Vec<_> = keyed_terms.into_iter().map(|(_, term)| term).collect();
    let ids: BTreeMap<_, _> = terms
        .iter()
        .cloned()
        .enumerate()
        .map(|(index, term)| (term, index))
        .collect();
    let table = terms
        .iter()
        .map(|term| materialize_term_node(term, level_ids, &ids))
        .collect();
    Ok((table, ids))
}

fn materialize_level_node(level: &CanonLevel, ids: &BTreeMap<CanonLevel, LevelId>) -> LevelNode {
    match level {
        CanonLevel::Zero => LevelNode::Zero,
        CanonLevel::Succ(inner) => LevelNode::Succ(ids[inner.as_ref()]),
        CanonLevel::Max(lhs, rhs) => LevelNode::Max(ids[lhs.as_ref()], ids[rhs.as_ref()]),
        CanonLevel::IMax(lhs, rhs) => LevelNode::IMax(ids[lhs.as_ref()], ids[rhs.as_ref()]),
        CanonLevel::Param(name) => LevelNode::Param(*name),
    }
}

fn materialize_term_node(
    term: &CanonTerm,
    level_ids: &BTreeMap<CanonLevel, LevelId>,
    term_ids: &BTreeMap<CanonTerm, TermId>,
) -> TermNode {
    match term {
        CanonTerm::Sort(level) => TermNode::Sort(level_ids[level]),
        CanonTerm::BVar(index) => TermNode::BVar(*index),
        CanonTerm::Const { global_ref, levels } => TermNode::Const {
            global_ref: global_ref.clone(),
            levels: levels.iter().map(|level| level_ids[level]).collect(),
        },
        CanonTerm::App(fun, arg) => TermNode::App(term_ids[fun.as_ref()], term_ids[arg.as_ref()]),
        CanonTerm::Lam { ty, body } => TermNode::Lam {
            ty: term_ids[ty.as_ref()],
            body: term_ids[body.as_ref()],
        },
        CanonTerm::Pi { ty, body } => TermNode::Pi {
            ty: term_ids[ty.as_ref()],
            body: term_ids[body.as_ref()],
        },
        CanonTerm::Let { ty, value, body } => TermNode::Let {
            ty: term_ids[ty.as_ref()],
            value: term_ids[value.as_ref()],
            body: term_ids[body.as_ref()],
        },
    }
}

pub(crate) fn materialize_decl_payload(
    decl: &CanonDeclPayload,
    level_ids: &BTreeMap<CanonLevel, LevelId>,
    term_ids: &BTreeMap<CanonTerm, TermId>,
) -> DeclPayload {
    match decl {
        CanonDeclPayload::Axiom {
            name,
            universe_params,
            ty,
        } => DeclPayload::Axiom {
            name: *name,
            universe_params: universe_params.clone(),
            ty: term_ids[ty],
        },
        CanonDeclPayload::Def {
            name,
            universe_params,
            ty,
            value,
            reducibility,
        } => DeclPayload::Def {
            name: *name,
            universe_params: universe_params.clone(),
            ty: term_ids[ty],
            value: term_ids[value],
            reducibility: *reducibility,
        },
        CanonDeclPayload::Theorem {
            name,
            universe_params,
            ty,
            proof,
        } => DeclPayload::Theorem {
            name: *name,
            universe_params: universe_params.clone(),
            ty: term_ids[ty],
            proof: term_ids[proof],
            opacity: Opacity::Opaque,
        },
        CanonDeclPayload::Inductive {
            name,
            universe_params,
            params,
            indices,
            sort,
            constructors,
            recursor,
        } => DeclPayload::Inductive {
            name: *name,
            universe_params: universe_params.clone(),
            params: params
                .iter()
                .map(|ty| BinderType { ty: term_ids[ty] })
                .collect(),
            indices: indices
                .iter()
                .map(|ty| BinderType { ty: term_ids[ty] })
                .collect(),
            sort: level_ids[sort],
            constructors: constructors
                .iter()
                .map(|(name, ty)| ConstructorSpec {
                    name: *name,
                    ty: term_ids[ty],
                })
                .collect(),
            recursor: recursor
                .as_ref()
                .map(|(name, params, ty, rules)| RecursorSpec {
                    name: *name,
                    universe_params: params.clone(),
                    ty: term_ids[ty],
                    rules: *rules,
                }),
        },
    }
}

fn collect_name(names: &mut BTreeSet<Name>, name: &Name) {
    names.insert(name.clone());
}

fn collect_names_from_decl(names: &mut BTreeSet<Name>, decl: &Decl) {
    collect_name(names, &Name::from_dotted(decl.name()));
    for param in decl.universe_params() {
        collect_name(names, &Name::from_dotted(param));
    }
    collect_names_from_expr(names, decl.ty());
    match decl {
        Decl::Def { value, .. } => collect_names_from_expr(names, value),
        Decl::Theorem { proof, .. } => collect_names_from_expr(names, proof),
        Decl::Inductive { data, .. } => {
            for param in &data.params {
                collect_names_from_expr(names, &param.ty);
            }
            for index in &data.indices {
                collect_names_from_expr(names, &index.ty);
            }
            collect_name(names, &Name::from_dotted(&data.name));
            for constructor in &data.constructors {
                collect_name(names, &Name::from_dotted(&constructor.name));
                collect_names_from_expr(names, &constructor.ty);
            }
            if let Some(recursor) = &data.recursor {
                collect_name(names, &Name::from_dotted(&recursor.name));
                for param in &recursor.universe_params {
                    collect_name(names, &Name::from_dotted(param));
                }
                collect_names_from_expr(names, &recursor.ty);
            }
        }
        _ => {}
    }
}

fn collect_const_names_from_decl(names: &mut BTreeSet<Name>, decl: &Decl) {
    collect_const_names_from_expr(names, decl.ty());
    match decl {
        Decl::Def { value, .. } => collect_const_names_from_expr(names, value),
        Decl::Theorem { proof, .. } => collect_const_names_from_expr(names, proof),
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
        Decl::Axiom { .. } | Decl::Constructor { .. } | Decl::Recursor { .. } => {}
    }
}

fn collect_const_names_from_expr(names: &mut BTreeSet<Name>, expr: &Expr) {
    match expr {
        Expr::Sort(_) | Expr::BVar(_) => {}
        Expr::Const { name, .. } => {
            collect_name(names, &Name::from_dotted(name));
        }
        Expr::App(fun, arg) => {
            collect_const_names_from_expr(names, fun);
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

fn collect_names_from_expr(names: &mut BTreeSet<Name>, expr: &Expr) {
    match expr {
        Expr::Sort(level) => collect_names_from_level(names, level),
        Expr::BVar(_) => {}
        Expr::Const { name, levels } => {
            collect_name(names, &Name::from_dotted(name));
            for level in levels {
                collect_names_from_level(names, level);
            }
        }
        Expr::App(fun, arg) => {
            collect_names_from_expr(names, fun);
            collect_names_from_expr(names, arg);
        }
        Expr::Lam { ty, body, .. } | Expr::Pi { ty, body, .. } => {
            collect_names_from_expr(names, ty);
            collect_names_from_expr(names, body);
        }
        Expr::Let {
            ty, value, body, ..
        } => {
            collect_names_from_expr(names, ty);
            collect_names_from_expr(names, value);
            collect_names_from_expr(names, body);
        }
    }
}

fn collect_names_from_level(names: &mut BTreeSet<Name>, level: &Level) {
    match level {
        Level::Zero => {}
        Level::Succ(inner) => collect_names_from_level(names, inner),
        Level::Max(lhs, rhs) | Level::IMax(lhs, rhs) => {
            collect_names_from_level(names, lhs);
            collect_names_from_level(names, rhs);
        }
        Level::Param(name) => collect_name(names, &Name::from_dotted(name)),
    }
}

pub(crate) fn ensure_unique_names(names: &[Name]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for name in names {
        if !seen.insert(name.clone()) {
            return Err(CertError::DuplicateName { name: name.clone() });
        }
    }
    Ok(())
}

fn ensure_canonical_names(names: &[Name]) -> Result<()> {
    if names.iter().all(Name::is_canonical) {
        Ok(())
    } else {
        Err(CertError::NonCanonicalEncoding { object: "Name" })
    }
}

fn imported_decl_map(
    imports: &[&VerifiedModule],
    name_index: &BTreeMap<Name, usize>,
    referenced_names: &BTreeSet<Name>,
) -> Result<BTreeMap<Name, ImportedDeclInfo>> {
    let mut map = BTreeMap::new();
    for (import_index, import) in imports.iter().enumerate() {
        for entry in &import.export_block {
            let name = import
                .name_table
                .get(entry.name)
                .ok_or(CertError::DecodeError)?;
            if !referenced_names.contains(name) || !name_index.contains_key(name) {
                continue;
            }
            let axiom_dependencies = entry
                .axiom_dependencies
                .iter()
                .map(|axiom| remap_imported_axiom_ref(imports, import, axiom, name_index))
                .collect::<Result<Vec<_>>>()?;
            let old = map.insert(
                name.clone(),
                ImportedDeclInfo {
                    import_index,
                    decl_interface_hash: entry.decl_interface_hash,
                    kind: entry.kind,
                    axiom_dependencies,
                },
            );
            if old.is_some() {
                return Err(CertError::DuplicateName { name: name.clone() });
            }
        }
    }
    Ok(map)
}

fn producer_imported_decl_map(
    imports: &[ProducerImportExportView],
    name_index: &BTreeMap<Name, usize>,
    referenced_names: &BTreeSet<Name>,
) -> Result<BTreeMap<Name, ImportedDeclInfo>> {
    let mut map = BTreeMap::new();
    for (import_index, import) in imports.iter().enumerate() {
        for entry in &import.exports {
            let name = import
                .name_table
                .get(entry.name)
                .ok_or(CertError::DecodeError)?;
            if !referenced_names.contains(name) || !name_index.contains_key(name) {
                continue;
            }
            let axiom_dependencies = entry
                .axiom_dependencies
                .iter()
                .map(|axiom| remap_producer_imported_axiom_ref(imports, import, axiom, name_index))
                .collect::<Result<Vec<_>>>()?;
            let old = map.insert(
                name.clone(),
                ImportedDeclInfo {
                    import_index,
                    decl_interface_hash: entry.decl_interface_hash,
                    kind: entry.kind,
                    axiom_dependencies,
                },
            );
            if old.is_some() {
                return Err(CertError::DuplicateName { name: name.clone() });
            }
        }
    }
    Ok(map)
}

fn remap_imported_axiom_ref(
    imports: &[&VerifiedModule],
    import: &VerifiedModule,
    axiom: &AxiomRef,
    name_index: &BTreeMap<Name, usize>,
) -> Result<AxiomRef> {
    let axiom_name = import
        .name_table
        .get(axiom.name)
        .ok_or(CertError::DecodeError)?;
    let name = *name_index.get(axiom_name).ok_or(CertError::DecodeError)?;
    if let GlobalRef::Builtin {
        decl_interface_hash,
        ..
    } = &axiom.global_ref
    {
        if builtin_decl_interface_hash(axiom_name) != Some(*decl_interface_hash) {
            return Err(CertError::UnknownDependency {
                name: axiom_name.clone(),
            });
        }
        return Ok(AxiomRef {
            global_ref: GlobalRef::Builtin {
                name,
                decl_interface_hash: *decl_interface_hash,
            },
            name,
            decl_interface_hash: *decl_interface_hash,
        });
    }
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

fn remap_producer_imported_axiom_ref(
    imports: &[ProducerImportExportView],
    import: &ProducerImportExportView,
    axiom: &AxiomRef,
    name_index: &BTreeMap<Name, usize>,
) -> Result<AxiomRef> {
    let axiom_name = import
        .name_table
        .get(axiom.name)
        .ok_or(CertError::DecodeError)?;
    let name = *name_index.get(axiom_name).ok_or(CertError::DecodeError)?;
    if let GlobalRef::Builtin {
        decl_interface_hash,
        ..
    } = &axiom.global_ref
    {
        if builtin_decl_interface_hash(axiom_name) != Some(*decl_interface_hash) {
            return Err(CertError::UnknownDependency {
                name: axiom_name.clone(),
            });
        }
        return Ok(AxiomRef {
            global_ref: GlobalRef::Builtin {
                name,
                decl_interface_hash: *decl_interface_hash,
            },
            name,
            decl_interface_hash: *decl_interface_hash,
        });
    }
    let import_index =
        producer_import_index_exporting_axiom(imports, axiom_name, axiom.decl_interface_hash)?;
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

fn referenced_imported_export_names(
    declarations: &[Decl],
    imports: &[&VerifiedModule],
    local_public_names: &[Name],
) -> Result<BTreeSet<Name>> {
    let mut referenced_names = BTreeSet::new();
    for decl in declarations {
        collect_const_names_from_decl(&mut referenced_names, decl);
    }

    let mut local_public_names = local_public_names.iter().cloned().collect::<BTreeSet<_>>();
    for decl in declarations {
        if let Decl::Inductive { data, .. } = decl {
            local_public_names.insert(Name::from_dotted(&data.name));
        }
    }
    referenced_names.retain(|name| !local_public_names.contains(name));

    let mut imported_exports = BTreeSet::new();
    for import in imports {
        for entry in &import.export_block {
            imported_exports.insert(
                import
                    .name_table
                    .get(entry.name)
                    .cloned()
                    .ok_or(CertError::DecodeError)?,
            );
        }
    }
    referenced_names.retain(|name| imported_exports.contains(name));

    Ok(referenced_names)
}

fn producer_referenced_imported_export_names(
    decl: &Decl,
    imports: &[ProducerImportExportView],
) -> Result<BTreeSet<Name>> {
    let mut referenced_names = BTreeSet::new();
    collect_const_names_from_decl(&mut referenced_names, decl);

    let mut imported_exports = BTreeSet::new();
    for import in imports {
        for entry in &import.exports {
            imported_exports.insert(
                import
                    .name_table
                    .get(entry.name)
                    .cloned()
                    .ok_or(CertError::DecodeError)?,
            );
        }
    }
    referenced_names.retain(|name| imported_exports.contains(name));

    Ok(referenced_names)
}

fn referenced_builtin_names(
    declarations: &[Decl],
    imports: &[&VerifiedModule],
    local_public_names: &[Name],
) -> Result<BTreeSet<Name>> {
    let mut referenced_names = BTreeSet::new();
    for decl in declarations {
        collect_const_names_from_decl(&mut referenced_names, decl);
    }

    let mut local_public_names = local_public_names.iter().cloned().collect::<BTreeSet<_>>();
    for decl in declarations {
        if let Decl::Inductive { data, .. } = decl {
            local_public_names.insert(Name::from_dotted(&data.name));
        }
    }
    referenced_names.retain(|name| !local_public_names.contains(name));

    let mut imported_exports = BTreeSet::new();
    for import in imports {
        for entry in &import.export_block {
            imported_exports.insert(
                import
                    .name_table
                    .get(entry.name)
                    .cloned()
                    .ok_or(CertError::DecodeError)?,
            );
        }
    }
    referenced_names.retain(|name| {
        !imported_exports.contains(name) && builtin_decl_interface_hash(name).is_some()
    });

    Ok(referenced_names)
}

fn collect_imported_axiom_names_for_referenced_exports(
    names: &mut BTreeSet<Name>,
    imports: &[&VerifiedModule],
    referenced_names: &BTreeSet<Name>,
) -> Result<()> {
    for import in imports {
        for entry in &import.export_block {
            let entry_name = import
                .name_table
                .get(entry.name)
                .ok_or(CertError::DecodeError)?;
            if !referenced_names.contains(entry_name) {
                continue;
            }
            for axiom in &entry.axiom_dependencies {
                let axiom_name = import
                    .name_table
                    .get(axiom.name)
                    .ok_or(CertError::DecodeError)?;
                collect_name(names, axiom_name);
            }
        }
    }
    Ok(())
}

fn producer_collect_imported_axiom_names_for_referenced_exports(
    names: &mut BTreeSet<Name>,
    imports: &[ProducerImportExportView],
    referenced_names: &BTreeSet<Name>,
) -> Result<()> {
    for import in imports {
        for entry in &import.exports {
            let entry_name = import
                .name_table
                .get(entry.name)
                .ok_or(CertError::DecodeError)?;
            if !referenced_names.contains(entry_name) {
                continue;
            }
            for axiom in &entry.axiom_dependencies {
                let axiom_name = import
                    .name_table
                    .get(axiom.name)
                    .ok_or(CertError::DecodeError)?;
                collect_name(names, axiom_name);
            }
        }
    }
    Ok(())
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

fn producer_import_index_exporting_axiom(
    imports: &[ProducerImportExportView],
    axiom_name: &Name,
    decl_interface_hash: Hash,
) -> Result<usize> {
    imports
        .iter()
        .enumerate()
        .find_map(|(import_index, import)| {
            import
                .exports
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

pub(crate) fn union_axioms(axioms: impl IntoIterator<Item = AxiomRef>) -> Vec<AxiomRef> {
    axioms
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
