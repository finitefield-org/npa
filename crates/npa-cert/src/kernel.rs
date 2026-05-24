use std::collections::BTreeSet;

use npa_kernel::{
    eq_inductive, eq_rec_type, nat_inductive, Binder, ConstructorDecl, Decl, Env, Error, Expr,
    InductiveDecl, Level, MutualInductiveBlock, RecursorDecl, RecursorRules, Reducibility,
    UniverseConstraint,
};

use crate::types::{
    CertError, CertHeader, CertReducibility, DeclPayload, ExportEntry, ExportKind, GlobalRef, Hash,
    LevelId, LevelNode, ModuleCert, ModuleHashes, Name, NameId, Result, TermId, TermNode,
    UniverseConstraintSpec, VerifiedModule,
};
use crate::{hash_with_domain, CORE_SPEC, FORMAT};

const BUILTIN_NAT: &str = "Nat";
const BUILTIN_NAT_ZERO: &str = "Nat.zero";
const BUILTIN_NAT_SUCC: &str = "Nat.succ";
const BUILTIN_NAT_REC: &str = "Nat.rec";
const BUILTIN_EQ: &str = "Eq";
const BUILTIN_EQ_REFL: &str = "Eq.refl";
const BUILTIN_EQ_REC: &str = "Eq.rec";

pub(crate) fn cert_to_kernel_decls(cert: &ModuleCert) -> Result<Vec<Decl>> {
    cert.declarations
        .iter()
        .map(|decl| decl_payload_to_kernel_decl(cert, &decl.decl))
        .collect()
}

/// Reconstruct kernel declarations exported by a verified module for downstream checking.
///
/// Transparent definitions keep their bodies and reducibility metadata; opaque definitions and
/// theorem exports are reconstructed as axioms because their bodies are not part of the public
/// downstream interface.
pub fn verified_module_to_kernel_decls(module: &VerifiedModule) -> Result<Vec<Decl>> {
    let cert = module_cert_from_verified_module(module);
    let mut decls = Vec::new();
    for decl in &cert.declarations {
        decls.push(match &decl.decl {
            DeclPayload::Axiom { name, .. } | DeclPayload::AxiomConstrained { name, .. } => {
                let entry = export_entry_for_decl(&cert, *name, ExportKind::Axiom)?;
                let universe_constraints =
                    universe_constraints_from_decl_payload(&cert, &decl.decl)?;
                if universe_constraints.is_empty() {
                    Decl::Axiom {
                        name: name_to_string(&cert, entry.name)?,
                        universe_params: universe_names(&cert, &entry.universe_params)?,
                        ty: expr_from_term(&cert, entry.ty)?,
                    }
                } else {
                    Decl::AxiomConstrained {
                        name: name_to_string(&cert, entry.name)?,
                        universe_params: universe_names(&cert, &entry.universe_params)?,
                        universe_constraints,
                        ty: expr_from_term(&cert, entry.ty)?,
                    }
                }
            }
            DeclPayload::Def { name, .. } | DeclPayload::DefConstrained { name, .. } => {
                let entry = export_entry_for_decl(&cert, *name, ExportKind::Def)?;
                let ty = expr_from_term(&cert, entry.ty)?;
                let universe_constraints =
                    universe_constraints_from_decl_payload(&cert, &decl.decl)?;
                match entry.reducibility.ok_or(CertError::DecodeError)? {
                    CertReducibility::Reducible if universe_constraints.is_empty() => Decl::Def {
                        name: name_to_string(&cert, entry.name)?,
                        universe_params: universe_names(&cert, &entry.universe_params)?,
                        ty,
                        value: expr_from_term(&cert, entry.body.ok_or(CertError::DecodeError)?)?,
                        reducibility: Reducibility::Reducible,
                    },
                    CertReducibility::Reducible => Decl::DefConstrained {
                        name: name_to_string(&cert, entry.name)?,
                        universe_params: universe_names(&cert, &entry.universe_params)?,
                        universe_constraints,
                        ty,
                        value: expr_from_term(&cert, entry.body.ok_or(CertError::DecodeError)?)?,
                        reducibility: Reducibility::Reducible,
                    },
                    CertReducibility::Opaque if universe_constraints.is_empty() => Decl::Axiom {
                        name: name_to_string(&cert, entry.name)?,
                        universe_params: universe_names(&cert, &entry.universe_params)?,
                        ty,
                    },
                    CertReducibility::Opaque => Decl::AxiomConstrained {
                        name: name_to_string(&cert, entry.name)?,
                        universe_params: universe_names(&cert, &entry.universe_params)?,
                        universe_constraints,
                        ty,
                    },
                }
            }
            DeclPayload::Theorem { name, .. } | DeclPayload::TheoremConstrained { name, .. } => {
                let entry = export_entry_for_decl(&cert, *name, ExportKind::Theorem)?;
                let universe_constraints =
                    universe_constraints_from_decl_payload(&cert, &decl.decl)?;
                if universe_constraints.is_empty() {
                    Decl::Axiom {
                        name: name_to_string(&cert, entry.name)?,
                        universe_params: universe_names(&cert, &entry.universe_params)?,
                        ty: expr_from_term(&cert, entry.ty)?,
                    }
                } else {
                    Decl::AxiomConstrained {
                        name: name_to_string(&cert, entry.name)?,
                        universe_params: universe_names(&cert, &entry.universe_params)?,
                        universe_constraints,
                        ty: expr_from_term(&cert, entry.ty)?,
                    }
                }
            }
            DeclPayload::Inductive { .. } | DeclPayload::InductiveConstrained { .. } => {
                normalize_builtin_import_decl(decl_payload_to_kernel_decl(&cert, &decl.decl)?)
            }
            DeclPayload::MutualInductiveBlock { .. } => {
                decl_payload_to_kernel_decl(&cert, &decl.decl)?
            }
        });
    }
    Ok(decls)
}

fn normalize_builtin_import_decl(decl: Decl) -> Decl {
    match decl {
        Decl::Inductive {
            name,
            universe_params,
            ty,
            mut data,
        } if name == BUILTIN_EQ => {
            data.recursor = None;
            Decl::Inductive {
                name,
                universe_params,
                ty,
                data,
            }
        }
        decl => decl,
    }
}

fn module_cert_from_verified_module(module: &VerifiedModule) -> ModuleCert {
    ModuleCert {
        header: CertHeader {
            format: FORMAT.to_owned(),
            core_spec: CORE_SPEC.to_owned(),
            module: module.module.clone(),
        },
        imports: Vec::new(),
        name_table: module.name_table.clone(),
        level_table: module.level_table.clone(),
        term_table: module.term_table.clone(),
        declarations: module.declarations.clone(),
        export_block: module.export_block.clone(),
        axiom_report: module.axiom_report.clone(),
        hashes: ModuleHashes {
            export_hash: module.export_hash,
            axiom_report_hash: [0; 32],
            certificate_hash: module.certificate_hash,
        },
    }
}

fn export_entry_for_decl(
    cert: &ModuleCert,
    name: NameId,
    kind: ExportKind,
) -> Result<&ExportEntry> {
    cert.export_block
        .iter()
        .find(|entry| entry.name == name && entry.kind == kind)
        .ok_or(CertError::DecodeError)
}

fn decl_payload_to_kernel_decl(cert: &ModuleCert, decl: &DeclPayload) -> Result<Decl> {
    Ok(match decl {
        DeclPayload::Axiom {
            name,
            universe_params,
            ty,
        } => Decl::Axiom {
            name: name_to_string(cert, *name)?,
            universe_params: universe_names(cert, universe_params)?,
            ty: expr_from_term(cert, *ty)?,
        },
        DeclPayload::AxiomConstrained {
            name,
            universe_params,
            universe_constraints,
            ty,
        } => Decl::AxiomConstrained {
            name: name_to_string(cert, *name)?,
            universe_params: universe_names(cert, universe_params)?,
            universe_constraints: universe_constraints_from_specs(cert, universe_constraints)?,
            ty: expr_from_term(cert, *ty)?,
        },
        DeclPayload::Def {
            name,
            universe_params,
            ty,
            value,
            reducibility,
        } => Decl::Def {
            name: name_to_string(cert, *name)?,
            universe_params: universe_names(cert, universe_params)?,
            ty: expr_from_term(cert, *ty)?,
            value: expr_from_term(cert, *value)?,
            reducibility: (*reducibility).into(),
        },
        DeclPayload::DefConstrained {
            name,
            universe_params,
            universe_constraints,
            ty,
            value,
            reducibility,
        } => Decl::DefConstrained {
            name: name_to_string(cert, *name)?,
            universe_params: universe_names(cert, universe_params)?,
            universe_constraints: universe_constraints_from_specs(cert, universe_constraints)?,
            ty: expr_from_term(cert, *ty)?,
            value: expr_from_term(cert, *value)?,
            reducibility: (*reducibility).into(),
        },
        DeclPayload::Theorem {
            name,
            universe_params,
            ty,
            proof,
            ..
        } => Decl::Theorem {
            name: name_to_string(cert, *name)?,
            universe_params: universe_names(cert, universe_params)?,
            ty: expr_from_term(cert, *ty)?,
            proof: expr_from_term(cert, *proof)?,
        },
        DeclPayload::TheoremConstrained {
            name,
            universe_params,
            universe_constraints,
            ty,
            proof,
            ..
        } => Decl::TheoremConstrained {
            name: name_to_string(cert, *name)?,
            universe_params: universe_names(cert, universe_params)?,
            universe_constraints: universe_constraints_from_specs(cert, universe_constraints)?,
            ty: expr_from_term(cert, *ty)?,
            proof: expr_from_term(cert, *proof)?,
        },
        DeclPayload::Inductive {
            name,
            universe_params,
            params,
            indices,
            sort,
            constructors,
            recursor,
        }
        | DeclPayload::InductiveConstrained {
            name,
            universe_params,
            params,
            indices,
            sort,
            constructors,
            recursor,
            ..
        } => Decl::Inductive {
            name: name_to_string(cert, *name)?,
            universe_params: universe_names(cert, universe_params)?,
            ty: Expr::sort(level_from_node(cert, *sort)?),
            data: Box::new(
                InductiveDecl::new(
                    name_to_string(cert, *name)?,
                    universe_names(cert, universe_params)?,
                    params
                        .iter()
                        .enumerate()
                        .map(|(index, binder)| {
                            Ok(Binder::new(
                                format!("p{index}"),
                                expr_from_term(cert, binder.ty)?,
                            ))
                        })
                        .collect::<Result<Vec<_>>>()?,
                    indices
                        .iter()
                        .enumerate()
                        .map(|(index, binder)| {
                            Ok(Binder::new(
                                format!("i{index}"),
                                expr_from_term(cert, binder.ty)?,
                            ))
                        })
                        .collect::<Result<Vec<_>>>()?,
                    level_from_node(cert, *sort)?,
                    constructors
                        .iter()
                        .map(|constructor| {
                            Ok(ConstructorDecl::new(
                                name_to_string(cert, constructor.name)?,
                                expr_from_term(cert, constructor.ty)?,
                            ))
                        })
                        .collect::<Result<Vec<_>>>()?,
                    recursor
                        .as_ref()
                        .map(|recursor| {
                            Ok::<_, CertError>(RecursorDecl::with_rules(
                                name_to_string(cert, recursor.name)?,
                                universe_names(cert, &recursor.universe_params)?,
                                expr_from_term(cert, recursor.ty)?,
                                RecursorRules::new(
                                    recursor.rules.minor_start,
                                    recursor.rules.major_index,
                                ),
                            ))
                        })
                        .transpose()?,
                )
                .with_universe_constraints(universe_constraints_from_decl_payload(cert, decl)?),
            ),
        },
        DeclPayload::MutualInductiveBlock {
            name,
            universe_params,
            universe_constraints,
            inductives,
        } => Decl::MutualInductiveBlock {
            name: name_to_string(cert, *name)?,
            universe_params: universe_names(cert, universe_params)?,
            data: Box::new(
                MutualInductiveBlock::new(
                    name_to_string(cert, *name)?,
                    universe_names(cert, universe_params)?,
                    inductives
                        .iter()
                        .map(|inductive| {
                            Ok(InductiveDecl::new(
                                name_to_string(cert, inductive.name)?,
                                universe_names(cert, universe_params)?,
                                inductive
                                    .params
                                    .iter()
                                    .enumerate()
                                    .map(|(index, binder)| {
                                        Ok(Binder::new(
                                            format!("p{index}"),
                                            expr_from_term(cert, binder.ty)?,
                                        ))
                                    })
                                    .collect::<Result<Vec<_>>>()?,
                                inductive
                                    .indices
                                    .iter()
                                    .enumerate()
                                    .map(|(index, binder)| {
                                        Ok(Binder::new(
                                            format!("i{index}"),
                                            expr_from_term(cert, binder.ty)?,
                                        ))
                                    })
                                    .collect::<Result<Vec<_>>>()?,
                                level_from_node(cert, inductive.sort)?,
                                inductive
                                    .constructors
                                    .iter()
                                    .map(|constructor| {
                                        Ok(ConstructorDecl::new(
                                            name_to_string(cert, constructor.name)?,
                                            expr_from_term(cert, constructor.ty)?,
                                        ))
                                    })
                                    .collect::<Result<Vec<_>>>()?,
                                inductive
                                    .recursor
                                    .as_ref()
                                    .map(|recursor| {
                                        Ok::<_, CertError>(RecursorDecl::with_rules(
                                            name_to_string(cert, recursor.name)?,
                                            universe_names(cert, &recursor.universe_params)?,
                                            expr_from_term(cert, recursor.ty)?,
                                            RecursorRules::new(
                                                recursor.rules.minor_start,
                                                recursor.rules.major_index,
                                            ),
                                        ))
                                    })
                                    .transpose()?,
                            ))
                        })
                        .collect::<Result<Vec<_>>>()?,
                )
                .with_universe_constraints(universe_constraints_from_specs(
                    cert,
                    universe_constraints,
                )?),
            ),
        },
    })
}

fn universe_constraints_from_decl_payload(
    cert: &ModuleCert,
    decl: &DeclPayload,
) -> Result<Vec<UniverseConstraint>> {
    match decl {
        DeclPayload::AxiomConstrained {
            universe_constraints,
            ..
        }
        | DeclPayload::DefConstrained {
            universe_constraints,
            ..
        }
        | DeclPayload::TheoremConstrained {
            universe_constraints,
            ..
        }
        | DeclPayload::InductiveConstrained {
            universe_constraints,
            ..
        }
        | DeclPayload::MutualInductiveBlock {
            universe_constraints,
            ..
        } => universe_constraints_from_specs(cert, universe_constraints),
        DeclPayload::Axiom { .. }
        | DeclPayload::Def { .. }
        | DeclPayload::Theorem { .. }
        | DeclPayload::Inductive { .. } => Ok(Vec::new()),
    }
}

fn universe_constraints_from_specs(
    cert: &ModuleCert,
    constraints: &[UniverseConstraintSpec],
) -> Result<Vec<UniverseConstraint>> {
    constraints
        .iter()
        .map(|constraint| {
            Ok(UniverseConstraint {
                lhs: level_from_node(cert, constraint.lhs)?,
                relation: constraint.relation,
                rhs: level_from_node(cert, constraint.rhs)?,
            })
        })
        .collect()
}

pub(crate) fn expr_from_term(cert: &ModuleCert, term: TermId) -> Result<Expr> {
    Ok(
        match cert.term_table.get(term).ok_or(CertError::DecodeError)? {
            TermNode::Sort(level) => Expr::sort(level_from_node(cert, *level)?),
            TermNode::BVar(index) => Expr::bvar(*index),
            TermNode::Const { global_ref, levels } => Expr::konst(
                global_ref_name(cert, global_ref)?,
                levels
                    .iter()
                    .map(|level| level_from_node(cert, *level))
                    .collect::<Result<Vec<_>>>()?,
            ),
            TermNode::App(fun, arg) => {
                Expr::app(expr_from_term(cert, *fun)?, expr_from_term(cert, *arg)?)
            }
            TermNode::Lam { ty, body } => Expr::lam(
                "_",
                expr_from_term(cert, *ty)?,
                expr_from_term(cert, *body)?,
            ),
            TermNode::Pi { ty, body } => Expr::pi(
                "_",
                expr_from_term(cert, *ty)?,
                expr_from_term(cert, *body)?,
            ),
            TermNode::Let { ty, value, body } => Expr::let_in(
                "_",
                expr_from_term(cert, *ty)?,
                expr_from_term(cert, *value)?,
                expr_from_term(cert, *body)?,
            ),
        },
    )
}

pub(crate) fn level_from_node(cert: &ModuleCert, level: LevelId) -> Result<Level> {
    Ok(
        match cert.level_table.get(level).ok_or(CertError::DecodeError)? {
            LevelNode::Zero => Level::zero(),
            LevelNode::Succ(inner) => Level::succ(level_from_node(cert, *inner)?),
            LevelNode::Max(lhs, rhs) => {
                Level::max(level_from_node(cert, *lhs)?, level_from_node(cert, *rhs)?)
            }
            LevelNode::IMax(lhs, rhs) => {
                Level::imax(level_from_node(cert, *lhs)?, level_from_node(cert, *rhs)?)
            }
            LevelNode::Param(name) => Level::param(name_to_string(cert, *name)?),
        },
    )
}

fn global_ref_name(cert: &ModuleCert, global_ref: &GlobalRef) -> Result<String> {
    match global_ref {
        GlobalRef::Builtin { name, .. } => name_to_string(cert, *name),
        GlobalRef::Imported { name, .. } => name_to_string(cert, *name),
        GlobalRef::Local { decl_index } => decl_name(cert, *decl_index),
        GlobalRef::LocalGenerated { name, .. } => name_to_string(cert, *name),
    }
}

fn decl_name(cert: &ModuleCert, decl_index: usize) -> Result<String> {
    let decl = cert
        .declarations
        .get(decl_index)
        .ok_or(CertError::DecodeError)?;
    let name = match &decl.decl {
        DeclPayload::Axiom { name, .. }
        | DeclPayload::AxiomConstrained { name, .. }
        | DeclPayload::Def { name, .. }
        | DeclPayload::DefConstrained { name, .. }
        | DeclPayload::Theorem { name, .. }
        | DeclPayload::TheoremConstrained { name, .. }
        | DeclPayload::Inductive { name, .. }
        | DeclPayload::InductiveConstrained { name, .. }
        | DeclPayload::MutualInductiveBlock { name, .. } => *name,
    };
    name_to_string(cert, name)
}

pub(crate) fn name_to_string(cert: &ModuleCert, name: NameId) -> Result<String> {
    Ok(cert
        .name_table
        .get(name)
        .ok_or(CertError::DecodeError)?
        .as_dotted())
}

pub(crate) fn universe_names(cert: &ModuleCert, names: &[NameId]) -> Result<Vec<String>> {
    names
        .iter()
        .map(|name| name_to_string(cert, *name))
        .collect()
}

pub(crate) fn add_decl_to_env(env: &mut Env, decl: Decl) -> Result<()> {
    match decl {
        Decl::Axiom {
            name,
            universe_params,
            ty,
        } => env.add_axiom(name, universe_params, ty)?,
        Decl::AxiomConstrained {
            name,
            universe_params,
            universe_constraints,
            ty,
        } => env.add_axiom_with_universe_constraints(
            name,
            universe_params,
            universe_constraints,
            ty,
        )?,
        Decl::Def {
            name,
            universe_params,
            ty,
            value,
            reducibility,
        } => env.add_def(name, universe_params, ty, value, reducibility)?,
        Decl::DefConstrained {
            name,
            universe_params,
            universe_constraints,
            ty,
            value,
            reducibility,
        } => env.add_def_with_universe_constraints(
            name,
            universe_params,
            universe_constraints,
            ty,
            value,
            reducibility,
        )?,
        Decl::Theorem {
            name,
            universe_params,
            ty,
            proof,
        } => env.add_theorem(name, universe_params, ty, proof)?,
        Decl::TheoremConstrained {
            name,
            universe_params,
            universe_constraints,
            ty,
            proof,
        } => env.add_theorem_with_universe_constraints(
            name,
            universe_params,
            universe_constraints,
            ty,
            proof,
        )?,
        Decl::Inductive { data, .. } => {
            let name = Name::from_dotted(&data.name);
            match env.add_inductive(*data) {
                Ok(()) => {}
                Err(Error::InvalidInductive(message)) if message.contains("recursor") => {
                    return Err(CertError::InductiveGeneratedArtifactMismatch { name });
                }
                Err(err) => return Err(CertError::Kernel(err)),
            }
        }
        Decl::MutualInductiveBlock { data, .. } => {
            let name = Name::from_dotted(&data.name);
            match env.add_mutual_inductive(*data) {
                Ok(()) => {}
                Err(Error::InvalidInductive(message)) if message.contains("recursor") => {
                    return Err(CertError::InductiveGeneratedArtifactMismatch { name });
                }
                Err(err) => return Err(CertError::Kernel(err)),
            }
        }
        Decl::Constructor { .. } | Decl::Recursor { .. } => {
            return Err(CertError::UnknownDependency {
                name: Name::from_dotted(decl.name()),
            });
        }
    }
    Ok(())
}

/// Return the canonical interface hash for a declaration supplied by the builtin checker profile.
pub fn builtin_decl_interface_hash(name: &Name) -> Option<Hash> {
    let tag = match name.as_dotted().as_str() {
        BUILTIN_NAT => "npa.machine-tactic.builtin.nat.v1",
        BUILTIN_NAT_ZERO => "npa.machine-tactic.builtin.nat.zero.v1",
        BUILTIN_NAT_SUCC => "npa.machine-tactic.builtin.nat.succ.v1",
        BUILTIN_NAT_REC => "npa.machine-tactic.builtin.nat.rec.v1",
        BUILTIN_EQ => "npa.machine-tactic.builtin.eq.v1",
        BUILTIN_EQ_REFL => "npa.machine-tactic.builtin.eq.refl.v1",
        BUILTIN_EQ_REC => "npa.machine-tactic.builtin.eq.rec.v1",
        _ => return None,
    };
    Some(hash_with_domain(
        b"NPA-BUILTIN-INTERFACE-0.1",
        tag.as_bytes(),
    ))
}

pub(crate) fn builtin_is_axiom(name: &Name) -> bool {
    name.as_dotted() == BUILTIN_EQ_REC
}

pub(crate) fn add_referenced_builtins_to_env(
    env: &mut Env,
    referenced: &BTreeSet<Name>,
) -> Result<()> {
    let needs_nat = referenced.iter().any(|name| {
        matches!(
            name.as_dotted().as_str(),
            BUILTIN_NAT | BUILTIN_NAT_ZERO | BUILTIN_NAT_SUCC | BUILTIN_NAT_REC
        )
    });
    let needs_eq = referenced.iter().any(|name| {
        matches!(
            name.as_dotted().as_str(),
            BUILTIN_EQ | BUILTIN_EQ_REFL | BUILTIN_EQ_REC
        )
    });
    let needs_eq_rec = referenced
        .iter()
        .any(|name| name.as_dotted() == BUILTIN_EQ_REC);

    if needs_nat && env.decl(BUILTIN_NAT).is_none() {
        env.add_inductive(nat_inductive())?;
    }
    if needs_eq && env.decl(BUILTIN_EQ).is_none() {
        env.add_inductive(eq_inductive())?;
    }
    if needs_eq_rec && env.decl(BUILTIN_EQ_REC).is_none() {
        env.add_axiom(
            BUILTIN_EQ_REC,
            vec!["u".to_owned(), "v".to_owned()],
            eq_rec_type(Level::param("u"), Level::param("v")),
        )?;
    }
    Ok(())
}

pub(crate) fn verified_module_referenced_builtin_names(
    module: &VerifiedModule,
) -> Result<BTreeSet<Name>> {
    let mut names = BTreeSet::new();
    for term in &module.term_table {
        if let TermNode::Const {
            global_ref:
                GlobalRef::Builtin {
                    name,
                    decl_interface_hash,
                },
            ..
        } = term
        {
            let name_value = module.name_table.get(*name).ok_or(CertError::DecodeError)?;
            if builtin_decl_interface_hash(name_value) != Some(*decl_interface_hash) {
                return Err(CertError::UnknownDependency {
                    name: name_value.clone(),
                });
            }
            names.insert(name_value.clone());
        }
    }
    Ok(names)
}
