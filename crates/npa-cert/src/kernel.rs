use npa_kernel::{
    Binder, ConstructorDecl, Decl, Env, Error, Expr, InductiveDecl, Level, RecursorDecl,
    RecursorRules,
};

use crate::types::{
    CertError, CertHeader, DeclPayload, GlobalRef, LevelId, LevelNode, ModuleCert, ModuleHashes,
    Name, NameId, Result, TermId, TermNode, VerifiedModule,
};
use crate::{CORE_SPEC, FORMAT};

pub(crate) fn cert_to_kernel_decls(cert: &ModuleCert) -> Result<Vec<Decl>> {
    let mut decls = Vec::new();
    for decl in &cert.declarations {
        decls.push(match &decl.decl {
            DeclPayload::Axiom {
                name,
                universe_params,
                ty,
            } => Decl::Axiom {
                name: name_to_string(cert, *name)?,
                universe_params: universe_names(cert, universe_params)?,
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
            DeclPayload::Inductive {
                name,
                universe_params,
                params,
                indices,
                sort,
                constructors,
                recursor,
            } => Decl::Inductive {
                name: name_to_string(cert, *name)?,
                universe_params: universe_names(cert, universe_params)?,
                ty: Expr::sort(level_from_node(cert, *sort)?),
                data: Box::new(InductiveDecl::new(
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
                )),
            },
        });
    }
    Ok(decls)
}

pub(crate) fn verified_module_to_kernel_decls(module: &VerifiedModule) -> Result<Vec<Decl>> {
    let cert = ModuleCert {
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
    };
    cert_to_kernel_decls(&cert)
}

fn expr_from_term(cert: &ModuleCert, term: TermId) -> Result<Expr> {
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

fn level_from_node(cert: &ModuleCert, level: LevelId) -> Result<Level> {
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
        | DeclPayload::Def { name, .. }
        | DeclPayload::Theorem { name, .. }
        | DeclPayload::Inductive { name, .. } => *name,
    };
    name_to_string(cert, name)
}

fn name_to_string(cert: &ModuleCert, name: NameId) -> Result<String> {
    Ok(cert
        .name_table
        .get(name)
        .ok_or(CertError::DecodeError)?
        .as_dotted())
}

fn universe_names(cert: &ModuleCert, names: &[NameId]) -> Result<Vec<String>> {
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
        Decl::Def {
            name,
            universe_params,
            ty,
            value,
            reducibility,
        } => env.add_def(name, universe_params, ty, value, reducibility)?,
        Decl::Theorem {
            name,
            universe_params,
            ty,
            proof,
        } => env.add_theorem(name, universe_params, ty, proof)?,
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
        Decl::Constructor { .. } | Decl::Recursor { .. } => {
            return Err(CertError::UnknownDependency {
                name: Name::from_dotted(decl.name()),
            });
        }
    }
    Ok(())
}
