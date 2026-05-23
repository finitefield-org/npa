use std::collections::BTreeSet;

use sha2::{Digest, Sha256};

use crate::{
    ReferenceAxiomDependency, ReferenceCertificateHeader, ReferenceCertificateSection,
    ReferenceCheckError, ReferenceCheckReason, ReferenceCheckedModule, ReferenceCheckerPolicy,
    ReferenceCoreExpr, ReferenceCoreGlobalRef, ReferenceCoreLevel, ReferenceDecodedCertificate,
    ReferenceDecodedCertificateCounts, ReferenceExportKind, ReferenceHash, ReferenceHashObject,
    ReferenceImportEntry, ReferenceImportEnvironment, ReferenceImportStore, ReferenceModuleHashes,
    ReferenceModuleName, ReferencePublicEnvironment, ReferencePublicExport,
    ReferenceResolvedImport, ReferenceTrustMode, REFERENCE_CERTIFICATE_FORMAT, REFERENCE_CORE_SPEC,
};

type DecodeResult<T> = Result<T, ReferenceCheckError>;

pub(crate) fn decode_certificate_impl(bytes: &[u8]) -> DecodeResult<ReferenceDecodedCertificate> {
    decode_module_certificate(bytes).map(DecodedModuleCertificate::summary)
}

pub(crate) fn verify_certificate_hashes_impl(
    bytes: &[u8],
) -> DecodeResult<ReferenceDecodedCertificate> {
    let cert = decode_module_certificate(bytes)?;
    cert.verify_hashes(bytes)?;
    Ok(cert.summary())
}

pub(crate) fn import_entry_from_source_free_certificate_impl(
    bytes: &[u8],
) -> DecodeResult<ReferenceImportEntry> {
    let cert = decode_module_certificate(bytes)?;
    cert.verify_hashes(bytes)?;
    cert.import_entry(false)
}

pub(crate) fn build_import_environment_impl(
    bytes: &[u8],
    import_store: &ReferenceImportStore,
    policy: &ReferenceCheckerPolicy,
) -> DecodeResult<ReferenceImportEnvironment> {
    let cert = decode_module_certificate(bytes)?;
    cert.verify_hashes(bytes)?;
    cert.build_import_environment(import_store, policy)
}

pub(crate) fn check_certificate_impl(
    bytes: &[u8],
    import_store: &ReferenceImportStore,
    policy: &ReferenceCheckerPolicy,
) -> DecodeResult<ReferenceCheckedModule> {
    let cert = decode_module_certificate(bytes)?;
    cert.verify_hashes(bytes)?;
    let imports = cert.build_import_environment(import_store, policy)?;
    cert.type_check(&imports)
}

fn decode_module_certificate(bytes: &[u8]) -> DecodeResult<DecodedModuleCertificate> {
    if bytes.is_empty() {
        return Err(ReferenceCheckError::empty());
    }

    let mut decoder = Decoder::new(bytes);
    let cert = decoder.module_certificate()?;
    if !decoder.is_done() {
        return Err(ReferenceCheckError::malformed(
            ReferenceCertificateSection::FullCertificate,
            decoder.offset(),
            ReferenceCheckReason::TrailingBytes,
        ));
    }
    cert.validate()?;
    Ok(cert)
}

#[derive(Clone, Debug)]
struct Located<T> {
    value: T,
    offset: usize,
}

#[derive(Clone, Debug)]
struct DecodedModuleCertificate {
    header: ReferenceCertificateHeader,
    imports: Vec<Located<ImportEntry>>,
    name_table: Vec<Located<ReferenceModuleName>>,
    level_table: Vec<Located<LevelNode>>,
    term_table: Vec<Located<TermNode>>,
    declarations: Vec<Located<DeclCert>>,
    export_block: Vec<Located<ExportEntry>>,
    axiom_report: AxiomReport,
    hashes: ReferenceModuleHashes,
    hash_offsets: ModuleHashOffsets,
}

impl DecodedModuleCertificate {
    fn validate(&self) -> DecodeResult<()> {
        self.validate_import_order()?;
        self.validate_name_table_order()?;
        let level_hashes = self.validate_level_table()?;
        self.validate_term_table(&level_hashes)?;
        let mut used = UsedTables::new();
        self.collect_name_roots(&mut used)?;
        self.collect_decl_roots(&mut used)?;
        self.collect_export_roots(&mut used)?;
        self.collect_axiom_report_roots(&mut used)?;
        self.collect_reachable_terms(&mut used)?;
        self.collect_reachable_levels(&mut used)?;
        self.validate_declaration_order()?;
        self.validate_vector_orders()?;
        self.validate_used_names(&used.names)?;
        self.validate_used_levels(&used.levels)?;
        self.validate_used_terms(&used.terms)?;
        Ok(())
    }

    fn summary(self) -> ReferenceDecodedCertificate {
        ReferenceDecodedCertificate::new(
            self.header,
            ReferenceDecodedCertificateCounts {
                imports_len: self.imports.len(),
                name_table_len: self.name_table.len(),
                level_table_len: self.level_table.len(),
                term_table_len: self.term_table.len(),
                declarations_len: self.declarations.len(),
                export_block_len: self.export_block.len(),
            },
            self.hashes,
        )
    }

    fn import_entry(
        &self,
        checked_by_reference_checker: bool,
    ) -> DecodeResult<ReferenceImportEntry> {
        Ok(ReferenceImportEntry::new(
            self.header.module.clone(),
            self.hashes.export_hash,
            self.hashes.axiom_report_hash,
            self.hashes.certificate_hash,
            self.public_environment()?,
            checked_by_reference_checker,
        ))
    }

    fn checked_module(&self) -> DecodeResult<ReferenceCheckedModule> {
        Ok(ReferenceCheckedModule::new(
            self.header.module.clone(),
            self.hashes.export_hash,
            self.hashes.axiom_report_hash,
            self.hashes.certificate_hash,
            self.public_environment()?,
        ))
    }

    fn public_environment(&self) -> DecodeResult<ReferencePublicEnvironment> {
        let core_levels = self.core_levels()?;
        let core_terms = self.core_terms(&core_levels)?;
        let exports = self
            .export_block
            .iter()
            .map(|entry| {
                let entry = &entry.value;
                ReferencePublicExport {
                    name: self.name_table[entry.name].value.clone(),
                    kind: match entry.kind {
                        ExportKind::Axiom => ReferenceExportKind::Axiom,
                        ExportKind::Def => ReferenceExportKind::Def,
                        ExportKind::Theorem => ReferenceExportKind::Theorem,
                        ExportKind::Inductive => ReferenceExportKind::Inductive,
                        ExportKind::Constructor => ReferenceExportKind::Constructor,
                        ExportKind::Recursor => ReferenceExportKind::Recursor,
                    },
                    decl_interface_hash: entry.decl_interface_hash,
                    axiom_dependencies: self.public_axiom_dependencies(&entry.axiom_dependencies),
                    universe_params: self.name_ids_to_names(&entry.universe_params),
                    ty: core_terms[entry.ty].clone(),
                    body: entry.body.map(|body| core_terms[body].clone()),
                }
            })
            .collect();
        Ok(ReferencePublicEnvironment::new(
            exports,
            self.public_axiom_dependencies(&self.axiom_report.module_axioms),
        ))
    }

    fn public_axiom_dependencies(&self, axioms: &[AxiomRef]) -> Vec<ReferenceAxiomDependency> {
        axioms
            .iter()
            .map(|axiom| ReferenceAxiomDependency {
                name: self.name_table[axiom.name].value.clone(),
                decl_interface_hash: axiom.decl_interface_hash,
            })
            .collect()
    }

    fn name_ids_to_names(&self, names: &[usize]) -> Vec<ReferenceModuleName> {
        names
            .iter()
            .map(|name| self.name_table[*name].value.clone())
            .collect()
    }

    fn build_import_environment(
        &self,
        import_store: &ReferenceImportStore,
        policy: &ReferenceCheckerPolicy,
    ) -> DecodeResult<ReferenceImportEnvironment> {
        let mut resolved = Vec::with_capacity(self.imports.len());
        for requested in &self.imports {
            let entry = resolve_import(requested, import_store, policy)?;
            resolved.push(ReferenceResolvedImport {
                module: entry.module().clone(),
                export_hash: *entry.export_hash(),
                certificate_hash: *entry.certificate_hash(),
                public_environment: entry.public_environment().clone(),
            });
        }
        Ok(ReferenceImportEnvironment::new(resolved))
    }

    fn type_check(
        &self,
        imports: &ReferenceImportEnvironment,
    ) -> DecodeResult<ReferenceCheckedModule> {
        TypeChecker::new(self, imports)?.check_declarations()?;
        self.checked_module()
    }

    fn verify_hashes(&self, bytes: &[u8]) -> DecodeResult<()> {
        let level_hashes = self.compute_level_hashes()?;
        let term_hashes = self.compute_term_hashes(&level_hashes)?;
        for declaration in &self.declarations {
            let expected = compute_decl_hashes(
                &declaration.value.decl,
                &declaration.value.dependencies,
                &declaration.value.axiom_dependencies,
                &self.term_table,
                &level_hashes,
                &term_hashes,
                &self.name_table,
            )?;
            if expected.decl_interface_hash != declaration.value.hashes.decl_interface_hash {
                return Err(ReferenceCheckError::hash_mismatch(
                    ReferenceCertificateSection::Declarations,
                    declaration.value.hashes.decl_interface_hash_offset,
                    ReferenceHashObject::DeclInterface,
                ));
            }
            if expected.decl_certificate_hash != declaration.value.hashes.decl_certificate_hash {
                return Err(ReferenceCheckError::hash_mismatch(
                    ReferenceCertificateSection::Declarations,
                    declaration.value.hashes.decl_certificate_hash_offset,
                    ReferenceHashObject::DeclCertificate,
                ));
            }
        }

        let expected_export_block = self.build_export_block(&term_hashes)?;
        let actual_export_block = self
            .export_block
            .iter()
            .map(|entry| entry.value.clone())
            .collect::<Vec<_>>();
        let expected_export_hash = hash_with_domain(
            b"NPA-MODULE-EXPORT-0.1",
            &encode_export_block(&expected_export_block),
        );
        if expected_export_block != actual_export_block
            || expected_export_hash != self.hashes.export_hash
        {
            return Err(ReferenceCheckError::hash_mismatch(
                ReferenceCertificateSection::Hashes,
                self.hash_offsets.export_hash_offset,
                ReferenceHashObject::ExportBlock,
            ));
        }

        let expected_axiom_report_hash = hash_with_domain(
            b"NPA-AXIOM-REPORT-0.1",
            &encode_axiom_report(&self.axiom_report),
        );
        if expected_axiom_report_hash != self.hashes.axiom_report_hash {
            return Err(ReferenceCheckError::hash_mismatch(
                ReferenceCertificateSection::Hashes,
                self.hash_offsets.axiom_report_hash_offset,
                ReferenceHashObject::AxiomReport,
            ));
        }

        let hash_input = bytes
            .get(..self.hash_offsets.certificate_hash_offset)
            .ok_or_else(|| {
                ReferenceCheckError::malformed(
                    ReferenceCertificateSection::Hashes,
                    self.hash_offsets.certificate_hash_offset,
                    ReferenceCheckReason::UnexpectedEof,
                )
            })?;
        let expected_certificate_hash = hash_with_domain(b"NPA-MODULE-CERT-0.1", hash_input);
        if expected_certificate_hash != self.hashes.certificate_hash {
            return Err(ReferenceCheckError::hash_mismatch(
                ReferenceCertificateSection::Hashes,
                self.hash_offsets.certificate_hash_offset,
                ReferenceHashObject::ModuleCertificate,
            ));
        }

        Ok(())
    }

    fn compute_level_hashes(&self) -> DecodeResult<Vec<ReferenceHash>> {
        let mut hashes = Vec::with_capacity(self.level_table.len());
        for located in &self.level_table {
            let key = level_node_key(&located.value, &hashes, &self.name_table)?;
            hashes.push(hash_with_domain(b"NPA-LEVEL-0.1", &key));
        }
        Ok(hashes)
    }

    fn compute_term_hashes(
        &self,
        level_hashes: &[ReferenceHash],
    ) -> DecodeResult<Vec<ReferenceHash>> {
        let mut hashes = Vec::with_capacity(self.term_table.len());
        for located in &self.term_table {
            let key = term_node_key(&located.value, &hashes, level_hashes)?;
            hashes.push(hash_with_domain(b"NPA-TERM-0.1", &key));
        }
        Ok(hashes)
    }

    fn core_levels(&self) -> DecodeResult<Vec<ReferenceCoreLevel>> {
        let mut levels: Vec<ReferenceCoreLevel> = Vec::with_capacity(self.level_table.len());
        for located in &self.level_table {
            levels.push(match &located.value {
                LevelNode::Zero => ReferenceCoreLevel::Zero,
                LevelNode::Succ(inner) => {
                    ReferenceCoreLevel::Succ(Box::new(levels[*inner].clone()))
                }
                LevelNode::Max(lhs, rhs) => ReferenceCoreLevel::Max(
                    Box::new(levels[*lhs].clone()),
                    Box::new(levels[*rhs].clone()),
                ),
                LevelNode::IMax(lhs, rhs) => ReferenceCoreLevel::IMax(
                    Box::new(levels[*lhs].clone()),
                    Box::new(levels[*rhs].clone()),
                ),
                LevelNode::Param(name) => {
                    ReferenceCoreLevel::Param(self.name_table[*name].value.clone())
                }
            });
        }
        Ok(levels)
    }

    fn core_terms(
        &self,
        core_levels: &[ReferenceCoreLevel],
    ) -> DecodeResult<Vec<ReferenceCoreExpr>> {
        let mut terms: Vec<ReferenceCoreExpr> = Vec::with_capacity(self.term_table.len());
        for located in &self.term_table {
            terms.push(match &located.value {
                TermNode::Sort(level) => ReferenceCoreExpr::Sort(core_levels[*level].clone()),
                TermNode::BVar(index) => ReferenceCoreExpr::BVar(*index),
                TermNode::Const {
                    global_ref,
                    levels: level_ids,
                } => ReferenceCoreExpr::Const {
                    global_ref: self.core_global_ref(global_ref),
                    levels: level_ids
                        .iter()
                        .map(|level| core_levels[*level].clone())
                        .collect(),
                },
                TermNode::App(fun, arg) => ReferenceCoreExpr::App(
                    Box::new(terms[*fun].clone()),
                    Box::new(terms[*arg].clone()),
                ),
                TermNode::Lam { ty, body } => ReferenceCoreExpr::Lam {
                    ty: Box::new(terms[*ty].clone()),
                    body: Box::new(terms[*body].clone()),
                },
                TermNode::Pi { ty, body } => ReferenceCoreExpr::Pi {
                    ty: Box::new(terms[*ty].clone()),
                    body: Box::new(terms[*body].clone()),
                },
                TermNode::Let { ty, value, body } => ReferenceCoreExpr::Let {
                    ty: Box::new(terms[*ty].clone()),
                    value: Box::new(terms[*value].clone()),
                    body: Box::new(terms[*body].clone()),
                },
            });
        }
        Ok(terms)
    }

    fn core_global_ref(&self, global_ref: &GlobalRef) -> ReferenceCoreGlobalRef {
        match global_ref {
            GlobalRef::Builtin {
                name,
                decl_interface_hash,
            } => ReferenceCoreGlobalRef::Builtin {
                name: self.name_table[*name].value.clone(),
                decl_interface_hash: *decl_interface_hash,
            },
            GlobalRef::Imported {
                import_index,
                name,
                decl_interface_hash,
            } => ReferenceCoreGlobalRef::Imported {
                import_index: *import_index,
                name: self.name_table[*name].value.clone(),
                decl_interface_hash: *decl_interface_hash,
            },
            GlobalRef::Local { decl_index } => ReferenceCoreGlobalRef::Local {
                decl_index: *decl_index,
            },
            GlobalRef::LocalGenerated { decl_index, name } => {
                ReferenceCoreGlobalRef::LocalGenerated {
                    decl_index: *decl_index,
                    name: self.name_table[*name].value.clone(),
                }
            }
        }
    }

    fn build_export_block(&self, term_hashes: &[ReferenceHash]) -> DecodeResult<Vec<ExportEntry>> {
        let mut entries = Vec::new();
        for located in &self.declarations {
            let decl = &located.value;
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
                } => {
                    let ty =
                        inductive_export_type_term_id(&self.term_table, params, indices, *sort)?;
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

    fn validate_import_order(&self) -> DecodeResult<()> {
        let mut seen = BTreeSet::new();
        for import in &self.imports {
            if !seen.insert((import.value.module.clone(), import.value.export_hash)) {
                return Err(ReferenceCheckError::import_resolution(
                    ReferenceCertificateSection::Imports,
                    import.offset,
                    ReferenceCheckReason::DuplicateImport,
                ));
            }
        }
        for pair in self.imports.windows(2) {
            let previous = import_order_key(&pair[0].value);
            let current = import_order_key(&pair[1].value);
            if previous >= current {
                return Err(ReferenceCheckError::malformed(
                    ReferenceCertificateSection::Imports,
                    pair[1].offset,
                    ReferenceCheckReason::NonCanonicalOrder,
                ));
            }
        }
        Ok(())
    }

    fn validate_name_table_order(&self) -> DecodeResult<()> {
        for pair in self.name_table.windows(2) {
            if pair[0].value == pair[1].value {
                return Err(ReferenceCheckError::malformed(
                    ReferenceCertificateSection::NameTable,
                    pair[1].offset,
                    ReferenceCheckReason::DuplicateName,
                ));
            }
            if pair[0].value > pair[1].value {
                return Err(ReferenceCheckError::malformed(
                    ReferenceCertificateSection::NameTable,
                    pair[1].offset,
                    ReferenceCheckReason::NonCanonicalOrder,
                ));
            }
        }
        Ok(())
    }

    fn validate_level_table(&self) -> DecodeResult<Vec<ReferenceHash>> {
        let mut hashes = Vec::with_capacity(self.level_table.len());
        let mut keys = Vec::with_capacity(self.level_table.len());
        let mut raw_levels = Vec::with_capacity(self.level_table.len());
        for (index, located) in self.level_table.iter().enumerate() {
            self.validate_level_refs(index, located)?;
            let raw = raw_level_from_node(&located.value, &raw_levels, &self.name_table)?;
            if normalize_level(raw.clone()) != raw {
                return Err(ReferenceCheckError::malformed(
                    ReferenceCertificateSection::LevelTable,
                    located.offset,
                    ReferenceCheckReason::NonNormalizedLevel,
                ));
            }
            let key = level_node_key(&located.value, &hashes, &self.name_table)?;
            let hash = hash_with_domain(b"NPA-LEVEL-0.1", &key);
            keys.push((level_node_height(&located.value, &self.level_table)?, key));
            hashes.push(hash);
            raw_levels.push(raw);
        }
        for (index, pair) in keys.windows(2).enumerate() {
            if pair[0] >= pair[1] {
                return Err(ReferenceCheckError::malformed(
                    ReferenceCertificateSection::LevelTable,
                    self.level_table[index + 1].offset,
                    ReferenceCheckReason::NonCanonicalOrder,
                ));
            }
        }
        Ok(hashes)
    }

    fn validate_level_refs(&self, index: usize, located: &Located<LevelNode>) -> DecodeResult<()> {
        match &located.value {
            LevelNode::Zero => Ok(()),
            LevelNode::Succ(inner) => self.require_previous_level(index, *inner, located.offset),
            LevelNode::Max(lhs, rhs) | LevelNode::IMax(lhs, rhs) => {
                self.require_previous_level(index, *lhs, located.offset)?;
                self.require_previous_level(index, *rhs, located.offset)
            }
            LevelNode::Param(name) => self.require_name(
                *name,
                ReferenceCertificateSection::LevelTable,
                located.offset,
            ),
        }
    }

    fn validate_term_table(&self, level_hashes: &[ReferenceHash]) -> DecodeResult<()> {
        let mut hashes = Vec::with_capacity(self.term_table.len());
        let mut keys = Vec::with_capacity(self.term_table.len());
        for (index, located) in self.term_table.iter().enumerate() {
            self.validate_term_refs(index, located)?;
            let key = term_node_key(&located.value, &hashes, level_hashes)?;
            keys.push((
                term_node_height(&located.value, &self.term_table)?,
                key.clone(),
            ));
            hashes.push(hash_with_domain(b"NPA-TERM-0.1", &key));
        }
        for (index, pair) in keys.windows(2).enumerate() {
            if pair[0] >= pair[1] {
                return Err(ReferenceCheckError::malformed(
                    ReferenceCertificateSection::TermTable,
                    self.term_table[index + 1].offset,
                    ReferenceCheckReason::NonCanonicalOrder,
                ));
            }
        }
        Ok(())
    }

    fn validate_term_refs(&self, index: usize, located: &Located<TermNode>) -> DecodeResult<()> {
        match &located.value {
            TermNode::Sort(level) => self.require_level(
                *level,
                ReferenceCertificateSection::TermTable,
                located.offset,
            ),
            TermNode::BVar(_) => Ok(()),
            TermNode::Const { global_ref, levels } => {
                self.require_global_ref(
                    global_ref,
                    ReferenceCertificateSection::TermTable,
                    located.offset,
                )?;
                for level in levels {
                    self.require_level(
                        *level,
                        ReferenceCertificateSection::TermTable,
                        located.offset,
                    )?;
                }
                Ok(())
            }
            TermNode::App(fun, arg) => {
                self.require_previous_term(index, *fun, located.offset)?;
                self.require_previous_term(index, *arg, located.offset)
            }
            TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => {
                self.require_previous_term(index, *ty, located.offset)?;
                self.require_previous_term(index, *body, located.offset)
            }
            TermNode::Let { ty, value, body } => {
                self.require_previous_term(index, *ty, located.offset)?;
                self.require_previous_term(index, *value, located.offset)?;
                self.require_previous_term(index, *body, located.offset)
            }
        }
    }

    fn collect_name_roots(&self, used: &mut UsedTables) -> DecodeResult<()> {
        used.names.insert(self.header.module.clone());
        for import in &self.imports {
            used.names.insert(import.value.module.clone());
        }
        Ok(())
    }

    fn collect_decl_roots(&self, used: &mut UsedTables) -> DecodeResult<()> {
        for located in &self.declarations {
            self.collect_decl_payload(&located.value.decl, used, located.offset)?;
            self.collect_dependency_entries(&located.value.dependencies, used, located.offset)?;
            self.collect_axiom_refs(
                &located.value.axiom_dependencies,
                used,
                ReferenceCertificateSection::Declarations,
                located.offset,
            )?;
        }
        Ok(())
    }

    fn collect_export_roots(&self, used: &mut UsedTables) -> DecodeResult<()> {
        for located in &self.export_block {
            let entry = &located.value;
            self.collect_name_id(
                entry.name,
                used,
                ReferenceCertificateSection::ExportBlock,
                located.offset,
            )?;
            self.collect_name_ids(
                &entry.universe_params,
                used,
                ReferenceCertificateSection::ExportBlock,
                located.offset,
            )?;
            self.collect_term_root(
                entry.ty,
                ReferenceCertificateSection::ExportBlock,
                located.offset,
            )?;
            used.terms.insert(entry.ty);
            if let Some(body) = entry.body {
                self.collect_term_root(
                    body,
                    ReferenceCertificateSection::ExportBlock,
                    located.offset,
                )?;
                used.terms.insert(body);
            }
            self.collect_axiom_refs(
                &entry.axiom_dependencies,
                used,
                ReferenceCertificateSection::ExportBlock,
                located.offset,
            )?;
        }
        Ok(())
    }

    fn collect_axiom_report_roots(&self, used: &mut UsedTables) -> DecodeResult<()> {
        for report in &self.axiom_report.per_declaration {
            if report.decl_index >= self.declarations.len() {
                return Err(ReferenceCheckError::malformed(
                    ReferenceCertificateSection::AxiomReport,
                    report.offset,
                    ReferenceCheckReason::DanglingReference,
                ));
            }
            self.collect_axiom_refs(
                &report.direct_axioms,
                used,
                ReferenceCertificateSection::AxiomReport,
                report.offset,
            )?;
            self.collect_axiom_refs(
                &report.transitive_axioms,
                used,
                ReferenceCertificateSection::AxiomReport,
                report.offset,
            )?;
        }
        self.collect_axiom_refs(
            &self.axiom_report.module_axioms,
            used,
            ReferenceCertificateSection::AxiomReport,
            self.axiom_report.module_axioms_offset,
        )
    }

    fn collect_decl_payload(
        &self,
        decl: &DeclPayload,
        used: &mut UsedTables,
        offset: usize,
    ) -> DecodeResult<()> {
        match decl {
            DeclPayload::Axiom {
                name,
                universe_params,
                ty,
            } => {
                self.collect_name_id(
                    *name,
                    used,
                    ReferenceCertificateSection::Declarations,
                    offset,
                )?;
                self.collect_name_ids(
                    universe_params,
                    used,
                    ReferenceCertificateSection::Declarations,
                    offset,
                )?;
                self.collect_term_root(*ty, ReferenceCertificateSection::Declarations, offset)?;
                used.terms.insert(*ty);
            }
            DeclPayload::Def {
                name,
                universe_params,
                ty,
                value,
                reducibility: _,
            } => {
                self.collect_name_id(
                    *name,
                    used,
                    ReferenceCertificateSection::Declarations,
                    offset,
                )?;
                self.collect_name_ids(
                    universe_params,
                    used,
                    ReferenceCertificateSection::Declarations,
                    offset,
                )?;
                self.collect_term_root(*ty, ReferenceCertificateSection::Declarations, offset)?;
                self.collect_term_root(*value, ReferenceCertificateSection::Declarations, offset)?;
                used.terms.insert(*ty);
                used.terms.insert(*value);
            }
            DeclPayload::Theorem {
                name,
                universe_params,
                ty,
                proof,
                opacity: _,
            } => {
                self.collect_name_id(
                    *name,
                    used,
                    ReferenceCertificateSection::Declarations,
                    offset,
                )?;
                self.collect_name_ids(
                    universe_params,
                    used,
                    ReferenceCertificateSection::Declarations,
                    offset,
                )?;
                self.collect_term_root(*ty, ReferenceCertificateSection::Declarations, offset)?;
                self.collect_term_root(*proof, ReferenceCertificateSection::Declarations, offset)?;
                used.terms.insert(*ty);
                used.terms.insert(*proof);
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
                self.collect_name_id(
                    *name,
                    used,
                    ReferenceCertificateSection::Declarations,
                    offset,
                )?;
                self.collect_name_ids(
                    universe_params,
                    used,
                    ReferenceCertificateSection::Declarations,
                    offset,
                )?;
                self.require_level(*sort, ReferenceCertificateSection::Declarations, offset)?;
                used.levels.insert(*sort);
                for binder in params.iter().chain(indices) {
                    self.collect_term_root(
                        binder.ty,
                        ReferenceCertificateSection::Declarations,
                        offset,
                    )?;
                    used.terms.insert(binder.ty);
                }
                for constructor in constructors {
                    self.collect_name_id(
                        constructor.name,
                        used,
                        ReferenceCertificateSection::Declarations,
                        offset,
                    )?;
                    self.collect_term_root(
                        constructor.ty,
                        ReferenceCertificateSection::Declarations,
                        offset,
                    )?;
                    used.terms.insert(constructor.ty);
                }
                if let Some(recursor) = recursor {
                    self.collect_name_id(
                        recursor.name,
                        used,
                        ReferenceCertificateSection::Declarations,
                        offset,
                    )?;
                    self.collect_name_ids(
                        &recursor.universe_params,
                        used,
                        ReferenceCertificateSection::Declarations,
                        offset,
                    )?;
                    self.collect_term_root(
                        recursor.ty,
                        ReferenceCertificateSection::Declarations,
                        offset,
                    )?;
                    used.terms.insert(recursor.ty);
                    let _ = recursor.rules;
                }
            }
        }
        Ok(())
    }

    fn collect_dependency_entries(
        &self,
        entries: &[DependencyEntry],
        used: &mut UsedTables,
        offset: usize,
    ) -> DecodeResult<()> {
        for entry in entries {
            self.collect_global_ref(
                &entry.global_ref,
                used,
                ReferenceCertificateSection::Declarations,
                offset,
            )?;
        }
        Ok(())
    }

    fn collect_axiom_refs(
        &self,
        axioms: &[AxiomRef],
        used: &mut UsedTables,
        section: ReferenceCertificateSection,
        offset: usize,
    ) -> DecodeResult<()> {
        for axiom in axioms {
            self.collect_global_ref(&axiom.global_ref, used, section, offset)?;
            self.collect_name_id(axiom.name, used, section, offset)?;
        }
        Ok(())
    }

    fn collect_reachable_terms(&self, used: &mut UsedTables) -> DecodeResult<()> {
        let mut stack = used.terms.iter().copied().collect::<Vec<_>>();
        while let Some(term_id) = stack.pop() {
            let located = self.term_table.get(term_id).ok_or_else(|| {
                ReferenceCheckError::malformed(
                    ReferenceCertificateSection::TermTable,
                    self.term_table.last().map_or(0, |entry| entry.offset),
                    ReferenceCheckReason::DanglingReference,
                )
            })?;
            let term = &located.value;
            match term {
                TermNode::Sort(level) => {
                    used.levels.insert(*level);
                }
                TermNode::BVar(_) => {}
                TermNode::Const { global_ref, levels } => {
                    self.collect_global_ref(
                        global_ref,
                        used,
                        ReferenceCertificateSection::TermTable,
                        located.offset,
                    )?;
                    used.levels.extend(levels.iter().copied());
                }
                TermNode::App(fun, arg) => {
                    push_term(*fun, used, &mut stack);
                    push_term(*arg, used, &mut stack);
                }
                TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => {
                    push_term(*ty, used, &mut stack);
                    push_term(*body, used, &mut stack);
                }
                TermNode::Let { ty, value, body } => {
                    push_term(*ty, used, &mut stack);
                    push_term(*value, used, &mut stack);
                    push_term(*body, used, &mut stack);
                }
            }
        }
        Ok(())
    }

    fn collect_reachable_levels(&self, used: &mut UsedTables) -> DecodeResult<()> {
        let mut stack = used.levels.iter().copied().collect::<Vec<_>>();
        while let Some(level_id) = stack.pop() {
            let level = &self
                .level_table
                .get(level_id)
                .ok_or_else(|| {
                    ReferenceCheckError::malformed(
                        ReferenceCertificateSection::LevelTable,
                        self.level_table.last().map_or(0, |entry| entry.offset),
                        ReferenceCheckReason::DanglingReference,
                    )
                })?
                .value;
            match level {
                LevelNode::Zero => {}
                LevelNode::Succ(inner) => push_level(*inner, used, &mut stack),
                LevelNode::Max(lhs, rhs) | LevelNode::IMax(lhs, rhs) => {
                    push_level(*lhs, used, &mut stack);
                    push_level(*rhs, used, &mut stack);
                }
                LevelNode::Param(name) => {
                    self.collect_name_id(*name, used, ReferenceCertificateSection::LevelTable, 0)?;
                }
            }
        }
        Ok(())
    }

    fn validate_declaration_order(&self) -> DecodeResult<()> {
        let mut local_names = Vec::with_capacity(self.declarations.len());
        let mut seen = BTreeSet::new();
        for located in &self.declarations {
            let name_id = located.value.decl.name_id();
            self.require_name(
                name_id,
                ReferenceCertificateSection::Declarations,
                located.offset,
            )?;
            let name = self.name_table[name_id].value.clone();
            if !seen.insert(name.clone()) {
                return Err(ReferenceCheckError::malformed(
                    ReferenceCertificateSection::Declarations,
                    located.offset,
                    ReferenceCheckReason::DuplicateDeclarationName,
                ));
            }
            local_names.push(name);
        }

        let dependencies = self
            .declarations
            .iter()
            .enumerate()
            .map(|(decl_index, located)| {
                let mut deps = BTreeSet::new();
                for dependency in &located.value.dependencies {
                    match &dependency.global_ref {
                        GlobalRef::Local {
                            decl_index: dependency_index,
                        }
                        | GlobalRef::LocalGenerated {
                            decl_index: dependency_index,
                            ..
                        } => {
                            if *dependency_index >= decl_index {
                                return Err(ReferenceCheckError::malformed(
                                    ReferenceCertificateSection::Declarations,
                                    located.offset,
                                    ReferenceCheckReason::NonCanonicalOrder,
                                ));
                            }
                            deps.insert(*dependency_index);
                        }
                        GlobalRef::Builtin { .. } | GlobalRef::Imported { .. } => {}
                    }
                }
                Ok(deps)
            })
            .collect::<DecodeResult<Vec<_>>>()?;

        let mut emitted = BTreeSet::new();
        let mut remaining = (0..self.declarations.len()).collect::<BTreeSet<_>>();
        let mut expected = Vec::with_capacity(self.declarations.len());
        while !remaining.is_empty() {
            let mut ready = remaining
                .iter()
                .copied()
                .filter(|index| dependencies[*index].is_subset(&emitted))
                .collect::<Vec<_>>();
            if ready.is_empty() {
                return Err(ReferenceCheckError::malformed(
                    ReferenceCertificateSection::Declarations,
                    self.declarations.first().map_or(0, |entry| entry.offset),
                    ReferenceCheckReason::NonCanonicalOrder,
                ));
            }
            ready.sort_by_key(|index| local_names[*index].clone());
            for index in ready {
                remaining.remove(&index);
                emitted.insert(index);
                expected.push(index);
            }
        }
        if expected != (0..self.declarations.len()).collect::<Vec<_>>() {
            let bad_index = expected
                .iter()
                .zip(0..self.declarations.len())
                .find_map(|(actual, expected)| (*actual != expected).then_some(expected))
                .unwrap_or(0);
            return Err(ReferenceCheckError::malformed(
                ReferenceCertificateSection::Declarations,
                self.declarations
                    .get(bad_index)
                    .map_or(0, |entry| entry.offset),
                ReferenceCheckReason::NonCanonicalOrder,
            ));
        }
        Ok(())
    }

    fn validate_vector_orders(&self) -> DecodeResult<()> {
        for located in &self.declarations {
            ensure_strict_order(
                &located.value.dependencies,
                ReferenceCertificateSection::Declarations,
                located.offset,
            )?;
            ensure_strict_order(
                &located.value.axiom_dependencies,
                ReferenceCertificateSection::Declarations,
                located.offset,
            )?;
        }
        for located in &self.export_block {
            ensure_strict_order(
                &located.value.axiom_dependencies,
                ReferenceCertificateSection::ExportBlock,
                located.offset,
            )?;
        }
        for report in &self.axiom_report.per_declaration {
            ensure_strict_order(
                &report.direct_axioms,
                ReferenceCertificateSection::AxiomReport,
                report.offset,
            )?;
            ensure_strict_order(
                &report.transitive_axioms,
                ReferenceCertificateSection::AxiomReport,
                report.offset,
            )?;
        }
        ensure_strict_order(
            &self.axiom_report.module_axioms,
            ReferenceCertificateSection::AxiomReport,
            self.axiom_report.module_axioms_offset,
        )
    }

    fn validate_used_names(&self, used_names: &BTreeSet<ReferenceModuleName>) -> DecodeResult<()> {
        let actual = self
            .name_table
            .iter()
            .map(|entry| entry.value.clone())
            .collect::<Vec<_>>();
        let expected = used_names.iter().cloned().collect::<Vec<_>>();
        if actual == expected {
            return Ok(());
        }
        for entry in &self.name_table {
            if !used_names.contains(&entry.value) {
                return Err(ReferenceCheckError::malformed(
                    ReferenceCertificateSection::NameTable,
                    entry.offset,
                    ReferenceCheckReason::UnusedTableEntry,
                ));
            }
        }
        Err(ReferenceCheckError::malformed(
            ReferenceCertificateSection::NameTable,
            self.name_table.first().map_or(0, |entry| entry.offset),
            ReferenceCheckReason::NonCanonicalOrder,
        ))
    }

    fn validate_used_levels(&self, used_levels: &BTreeSet<usize>) -> DecodeResult<()> {
        for (index, entry) in self.level_table.iter().enumerate() {
            if !used_levels.contains(&index) {
                return Err(ReferenceCheckError::malformed(
                    ReferenceCertificateSection::LevelTable,
                    entry.offset,
                    ReferenceCheckReason::UnusedTableEntry,
                ));
            }
        }
        Ok(())
    }

    fn validate_used_terms(&self, used_terms: &BTreeSet<usize>) -> DecodeResult<()> {
        for (index, entry) in self.term_table.iter().enumerate() {
            if !used_terms.contains(&index) {
                return Err(ReferenceCheckError::malformed(
                    ReferenceCertificateSection::TermTable,
                    entry.offset,
                    ReferenceCheckReason::UnusedTableEntry,
                ));
            }
        }
        Ok(())
    }

    fn collect_name_id(
        &self,
        id: usize,
        used: &mut UsedTables,
        section: ReferenceCertificateSection,
        offset: usize,
    ) -> DecodeResult<()> {
        let name = self
            .name_table
            .get(id)
            .ok_or_else(|| {
                ReferenceCheckError::malformed(
                    section,
                    offset,
                    ReferenceCheckReason::DanglingReference,
                )
            })?
            .value
            .clone();
        used.names.insert(name);
        Ok(())
    }

    fn collect_name_ids(
        &self,
        ids: &[usize],
        used: &mut UsedTables,
        section: ReferenceCertificateSection,
        offset: usize,
    ) -> DecodeResult<()> {
        for id in ids {
            self.collect_name_id(*id, used, section, offset)?;
        }
        Ok(())
    }

    fn collect_term_root(
        &self,
        id: usize,
        section: ReferenceCertificateSection,
        offset: usize,
    ) -> DecodeResult<()> {
        self.require_term(id, section, offset)
    }

    fn collect_global_ref(
        &self,
        global_ref: &GlobalRef,
        used: &mut UsedTables,
        section: ReferenceCertificateSection,
        offset: usize,
    ) -> DecodeResult<()> {
        self.require_global_ref(global_ref, section, offset)?;
        match global_ref {
            GlobalRef::Builtin { name, .. }
            | GlobalRef::Imported { name, .. }
            | GlobalRef::LocalGenerated { name, .. } => {
                self.collect_name_id(*name, used, section, offset)?;
            }
            GlobalRef::Local { .. } => {}
        }
        Ok(())
    }

    fn require_name(
        &self,
        id: usize,
        section: ReferenceCertificateSection,
        offset: usize,
    ) -> DecodeResult<()> {
        if id < self.name_table.len() {
            Ok(())
        } else {
            Err(ReferenceCheckError::malformed(
                section,
                offset,
                ReferenceCheckReason::DanglingReference,
            ))
        }
    }

    fn require_level(
        &self,
        id: usize,
        section: ReferenceCertificateSection,
        offset: usize,
    ) -> DecodeResult<()> {
        if id < self.level_table.len() {
            Ok(())
        } else {
            Err(ReferenceCheckError::malformed(
                section,
                offset,
                ReferenceCheckReason::DanglingReference,
            ))
        }
    }

    fn require_term(
        &self,
        id: usize,
        section: ReferenceCertificateSection,
        offset: usize,
    ) -> DecodeResult<()> {
        if id < self.term_table.len() {
            Ok(())
        } else {
            Err(ReferenceCheckError::malformed(
                section,
                offset,
                ReferenceCheckReason::DanglingReference,
            ))
        }
    }

    fn require_previous_level(&self, index: usize, id: usize, offset: usize) -> DecodeResult<()> {
        if id < index {
            Ok(())
        } else {
            Err(ReferenceCheckError::malformed(
                ReferenceCertificateSection::LevelTable,
                offset,
                ReferenceCheckReason::DanglingReference,
            ))
        }
    }

    fn require_previous_term(&self, index: usize, id: usize, offset: usize) -> DecodeResult<()> {
        if id < index {
            Ok(())
        } else {
            Err(ReferenceCheckError::malformed(
                ReferenceCertificateSection::TermTable,
                offset,
                ReferenceCheckReason::DanglingReference,
            ))
        }
    }

    fn require_global_ref(
        &self,
        global_ref: &GlobalRef,
        section: ReferenceCertificateSection,
        offset: usize,
    ) -> DecodeResult<()> {
        match global_ref {
            GlobalRef::Builtin { name, .. } => self.require_name(*name, section, offset),
            GlobalRef::Imported {
                import_index, name, ..
            } => {
                if *import_index >= self.imports.len() {
                    return Err(ReferenceCheckError::malformed(
                        section,
                        offset,
                        ReferenceCheckReason::DanglingReference,
                    ));
                }
                self.require_name(*name, section, offset)
            }
            GlobalRef::Local { decl_index } => self.require_decl(*decl_index, section, offset),
            GlobalRef::LocalGenerated { decl_index, name } => {
                self.require_decl(*decl_index, section, offset)?;
                self.require_name(*name, section, offset)
            }
        }
    }

    fn require_decl(
        &self,
        id: usize,
        section: ReferenceCertificateSection,
        offset: usize,
    ) -> DecodeResult<()> {
        if id < self.declarations.len() {
            Ok(())
        } else {
            Err(ReferenceCheckError::malformed(
                section,
                offset,
                ReferenceCheckReason::DanglingReference,
            ))
        }
    }
}

fn resolve_import<'a>(
    requested: &Located<ImportEntry>,
    import_store: &'a ReferenceImportStore,
    policy: &ReferenceCheckerPolicy,
) -> DecodeResult<&'a ReferenceImportEntry> {
    let same_module = import_store
        .entries()
        .iter()
        .filter(|entry| entry.module() == &requested.value.module)
        .collect::<Vec<_>>();
    if same_module.is_empty() {
        return Err(ReferenceCheckError::import_resolution(
            ReferenceCertificateSection::Imports,
            requested.offset,
            ReferenceCheckReason::MissingImport,
        ));
    }

    let same_export = same_module
        .into_iter()
        .filter(|entry| *entry.export_hash() == requested.value.export_hash)
        .collect::<Vec<_>>();
    if same_export.is_empty() {
        return Err(ReferenceCheckError::import_resolution(
            ReferenceCertificateSection::Imports,
            requested.offset,
            ReferenceCheckReason::ImportExportHashMismatch,
        ));
    }
    if same_export.len() > 1 {
        return Err(ReferenceCheckError::import_resolution(
            ReferenceCertificateSection::Imports,
            requested.offset,
            ReferenceCheckReason::DuplicateImport,
        ));
    }

    let entry = same_export[0];
    if let Some(certificate_hash) = requested.value.certificate_hash {
        if *entry.certificate_hash() != certificate_hash {
            return Err(ReferenceCheckError::import_resolution(
                ReferenceCertificateSection::Imports,
                requested.offset,
                ReferenceCheckReason::ImportCertificateHashMismatch,
            ));
        }
    }

    if policy.trust_mode == ReferenceTrustMode::HighTrust {
        let Some(certificate_hash) = requested.value.certificate_hash else {
            return Err(ReferenceCheckError::import_resolution(
                ReferenceCertificateSection::Imports,
                requested.offset,
                ReferenceCheckReason::MissingImportCertificateHash,
            ));
        };
        if *entry.certificate_hash() != certificate_hash {
            return Err(ReferenceCheckError::import_resolution(
                ReferenceCertificateSection::Imports,
                requested.offset,
                ReferenceCheckReason::ImportCertificateHashMismatch,
            ));
        }
        if !entry.checked_by_reference_checker() {
            return Err(ReferenceCheckError::import_resolution(
                ReferenceCertificateSection::Imports,
                requested.offset,
                ReferenceCheckReason::UncheckedImport,
            ));
        }
    }

    Ok(entry)
}

struct TypeChecker<'a> {
    cert: &'a DecodedModuleCertificate,
    imports: &'a ReferenceImportEnvironment,
    terms: Vec<ReferenceCoreExpr>,
    locals: Vec<TypeSignature>,
}

impl<'a> TypeChecker<'a> {
    fn new(
        cert: &'a DecodedModuleCertificate,
        imports: &'a ReferenceImportEnvironment,
    ) -> DecodeResult<Self> {
        let levels = cert.core_levels()?;
        let terms = cert.core_terms(&levels)?;
        Ok(Self {
            cert,
            imports,
            terms,
            locals: Vec::new(),
        })
    }

    fn check_declarations(&mut self) -> DecodeResult<()> {
        for located in &self.cert.declarations {
            let delta = self.declaration_universe_params(&located.value.decl);
            let ctx = TypeContext::default();
            match &located.value.decl {
                DeclPayload::Axiom { ty, .. } => {
                    self.expect_sort(&ctx, &delta, &self.terms[*ty], located.offset)?;
                    self.locals.push(self.signature_for_decl(&located.value)?);
                }
                DeclPayload::Def { ty, value, .. } => {
                    self.expect_sort(&ctx, &delta, &self.terms[*ty], located.offset)?;
                    self.check(
                        &ctx,
                        &delta,
                        &self.terms[*value],
                        &self.terms[*ty],
                        located.offset,
                    )?;
                    self.locals.push(self.signature_for_decl(&located.value)?);
                }
                DeclPayload::Theorem { ty, proof, .. } => {
                    self.expect_sort(&ctx, &delta, &self.terms[*ty], located.offset)?;
                    self.check(
                        &ctx,
                        &delta,
                        &self.terms[*proof],
                        &self.terms[*ty],
                        located.offset,
                    )?;
                    self.locals.push(self.signature_for_decl(&located.value)?);
                }
                DeclPayload::Inductive { .. } => {
                    return Err(ReferenceCheckError::unsupported(located.offset));
                }
            }
        }
        Ok(())
    }

    fn declaration_universe_params(&self, decl: &DeclPayload) -> Vec<ReferenceModuleName> {
        let params = match decl {
            DeclPayload::Axiom {
                universe_params, ..
            }
            | DeclPayload::Def {
                universe_params, ..
            }
            | DeclPayload::Theorem {
                universe_params, ..
            }
            | DeclPayload::Inductive {
                universe_params, ..
            } => universe_params,
        };
        self.cert.name_ids_to_names(params)
    }

    fn signature_for_decl(&self, decl: &DeclCert) -> DecodeResult<TypeSignature> {
        Ok(match &decl.decl {
            DeclPayload::Axiom {
                name: _,
                universe_params,
                ty,
            } => TypeSignature {
                universe_params: self.cert.name_ids_to_names(universe_params),
                ty: self.terms[*ty].clone(),
            },
            DeclPayload::Def {
                universe_params,
                ty,
                ..
            } => TypeSignature {
                universe_params: self.cert.name_ids_to_names(universe_params),
                ty: self.terms[*ty].clone(),
            },
            DeclPayload::Theorem {
                universe_params,
                ty,
                ..
            } => TypeSignature {
                universe_params: self.cert.name_ids_to_names(universe_params),
                ty: self.terms[*ty].clone(),
            },
            DeclPayload::Inductive { .. } => {
                return Err(ReferenceCheckError::unsupported(
                    self.cert
                        .declarations
                        .first()
                        .map_or(0, |entry| entry.offset),
                ));
            }
        })
    }

    fn infer(
        &self,
        ctx: &TypeContext,
        delta: &[ReferenceModuleName],
        term: &ReferenceCoreExpr,
        offset: usize,
    ) -> DecodeResult<ReferenceCoreExpr> {
        match term {
            ReferenceCoreExpr::Sort(level) => {
                ensure_level_wf(level, delta, offset)?;
                Ok(ReferenceCoreExpr::Sort(ReferenceCoreLevel::Succ(Box::new(
                    level.clone(),
                ))))
            }
            ReferenceCoreExpr::BVar(index) => ctx.lookup_type(*index, offset),
            ReferenceCoreExpr::Const { global_ref, levels } => {
                for level in levels {
                    ensure_level_wf(level, delta, offset)?;
                }
                let signature = self.resolve_signature(global_ref, offset)?;
                if signature.universe_params.len() != levels.len() {
                    return Err(ReferenceCheckError::type_check(
                        ReferenceCertificateSection::Declarations,
                        offset,
                        ReferenceCheckReason::BadUniverseArity,
                    ));
                }
                Ok(subst_levels_expr(
                    &signature.ty,
                    &signature.universe_params,
                    levels,
                ))
            }
            ReferenceCoreExpr::Pi { ty, body } => {
                let domain_sort = self.expect_sort(ctx, delta, ty, offset)?;
                let mut body_ctx = ctx.clone();
                body_ctx.push_assumption((**ty).clone());
                let body_sort = self.expect_sort(&body_ctx, delta, body, offset)?;
                Ok(ReferenceCoreExpr::Sort(ReferenceCoreLevel::IMax(
                    Box::new(domain_sort),
                    Box::new(body_sort),
                )))
            }
            ReferenceCoreExpr::Lam { ty, body } => {
                self.expect_sort(ctx, delta, ty, offset)?;
                let mut body_ctx = ctx.clone();
                body_ctx.push_assumption((**ty).clone());
                let body_ty = self.infer(&body_ctx, delta, body, offset)?;
                Ok(ReferenceCoreExpr::Pi {
                    ty: ty.clone(),
                    body: Box::new(body_ty),
                })
            }
            ReferenceCoreExpr::App(fun, arg) => {
                let fun_ty = self.infer(ctx, delta, fun, offset)?;
                match fun_ty {
                    ReferenceCoreExpr::Pi { ty, body } => {
                        self.check(ctx, delta, arg, &ty, offset)?;
                        instantiate(&body, arg, offset)
                    }
                    _ => Err(ReferenceCheckError::type_check(
                        ReferenceCertificateSection::Declarations,
                        offset,
                        ReferenceCheckReason::ExpectedFunction,
                    )),
                }
            }
            ReferenceCoreExpr::Let { ty, value, body } => {
                self.expect_sort(ctx, delta, ty, offset)?;
                self.check(ctx, delta, value, ty, offset)?;
                let mut body_ctx = ctx.clone();
                body_ctx.push_definition((**ty).clone(), (**value).clone());
                let body_ty = self.infer(&body_ctx, delta, body, offset)?;
                instantiate(&body_ty, value, offset)
            }
        }
    }

    fn check(
        &self,
        ctx: &TypeContext,
        delta: &[ReferenceModuleName],
        term: &ReferenceCoreExpr,
        expected: &ReferenceCoreExpr,
        offset: usize,
    ) -> DecodeResult<()> {
        let actual = self.infer(ctx, delta, term, offset)?;
        if actual == *expected {
            Ok(())
        } else {
            Err(ReferenceCheckError::type_check(
                ReferenceCertificateSection::Declarations,
                offset,
                ReferenceCheckReason::TypeMismatch,
            ))
        }
    }

    fn expect_sort(
        &self,
        ctx: &TypeContext,
        delta: &[ReferenceModuleName],
        term: &ReferenceCoreExpr,
        offset: usize,
    ) -> DecodeResult<ReferenceCoreLevel> {
        match self.infer(ctx, delta, term, offset)? {
            ReferenceCoreExpr::Sort(level) => Ok(level),
            _ => Err(ReferenceCheckError::type_check(
                ReferenceCertificateSection::Declarations,
                offset,
                ReferenceCheckReason::ExpectedSort,
            )),
        }
    }

    fn resolve_signature(
        &self,
        global_ref: &ReferenceCoreGlobalRef,
        offset: usize,
    ) -> DecodeResult<TypeSignature> {
        match global_ref {
            ReferenceCoreGlobalRef::Builtin { .. }
            | ReferenceCoreGlobalRef::LocalGenerated { .. } => {
                Err(ReferenceCheckError::type_check(
                    ReferenceCertificateSection::Declarations,
                    offset,
                    ReferenceCheckReason::UnknownReference,
                ))
            }
            ReferenceCoreGlobalRef::Imported {
                import_index,
                name,
                decl_interface_hash,
            } => {
                let import = self.imports.imports().get(*import_index).ok_or_else(|| {
                    ReferenceCheckError::type_check(
                        ReferenceCertificateSection::Declarations,
                        offset,
                        ReferenceCheckReason::UnknownReference,
                    )
                })?;
                let export = import
                    .public_environment
                    .exports()
                    .iter()
                    .find(|export| {
                        export.name == *name && export.decl_interface_hash == *decl_interface_hash
                    })
                    .ok_or_else(|| {
                        ReferenceCheckError::type_check(
                            ReferenceCertificateSection::Declarations,
                            offset,
                            ReferenceCheckReason::UnknownReference,
                        )
                    })?;
                Ok(TypeSignature {
                    universe_params: export.universe_params.clone(),
                    ty: export.ty.clone(),
                })
            }
            ReferenceCoreGlobalRef::Local { decl_index } => {
                self.locals.get(*decl_index).cloned().ok_or_else(|| {
                    ReferenceCheckError::type_check(
                        ReferenceCertificateSection::Declarations,
                        offset,
                        ReferenceCheckReason::UnknownReference,
                    )
                })
            }
        }
    }
}

#[derive(Clone, Debug)]
struct TypeSignature {
    universe_params: Vec<ReferenceModuleName>,
    ty: ReferenceCoreExpr,
}

#[derive(Clone, Debug, Default)]
struct TypeContext {
    locals: Vec<LocalType>,
}

impl TypeContext {
    fn push_assumption(&mut self, ty: ReferenceCoreExpr) {
        self.locals.push(LocalType { ty, _value: None });
    }

    fn push_definition(&mut self, ty: ReferenceCoreExpr, value: ReferenceCoreExpr) {
        self.locals.push(LocalType {
            ty,
            _value: Some(value),
        });
    }

    fn lookup_type(&self, index: u32, offset: usize) -> DecodeResult<ReferenceCoreExpr> {
        let index = index as usize;
        let local = self
            .locals
            .get(self.locals.len().checked_sub(index + 1).ok_or_else(|| {
                ReferenceCheckError::type_check(
                    ReferenceCertificateSection::Declarations,
                    offset,
                    ReferenceCheckReason::InvalidBVar,
                )
            })?)
            .ok_or_else(|| {
                ReferenceCheckError::type_check(
                    ReferenceCertificateSection::Declarations,
                    offset,
                    ReferenceCheckReason::InvalidBVar,
                )
            })?;
        shift(&local.ty, index as i32 + 1, 0, offset)
    }
}

#[derive(Clone, Debug)]
struct LocalType {
    ty: ReferenceCoreExpr,
    _value: Option<ReferenceCoreExpr>,
}

fn ensure_level_wf(
    level: &ReferenceCoreLevel,
    delta: &[ReferenceModuleName],
    offset: usize,
) -> DecodeResult<()> {
    match level {
        ReferenceCoreLevel::Zero => Ok(()),
        ReferenceCoreLevel::Succ(inner) => ensure_level_wf(inner, delta, offset),
        ReferenceCoreLevel::Max(lhs, rhs) | ReferenceCoreLevel::IMax(lhs, rhs) => {
            ensure_level_wf(lhs, delta, offset)?;
            ensure_level_wf(rhs, delta, offset)
        }
        ReferenceCoreLevel::Param(name) => {
            if delta.contains(name) {
                Ok(())
            } else {
                Err(ReferenceCheckError::type_check(
                    ReferenceCertificateSection::Declarations,
                    offset,
                    ReferenceCheckReason::UnknownReference,
                ))
            }
        }
    }
}

fn subst_levels_expr(
    expr: &ReferenceCoreExpr,
    params: &[ReferenceModuleName],
    levels: &[ReferenceCoreLevel],
) -> ReferenceCoreExpr {
    match expr {
        ReferenceCoreExpr::Sort(level) => {
            ReferenceCoreExpr::Sort(subst_level(level, params, levels))
        }
        ReferenceCoreExpr::BVar(index) => ReferenceCoreExpr::BVar(*index),
        ReferenceCoreExpr::Const {
            global_ref,
            levels: expr_levels,
        } => ReferenceCoreExpr::Const {
            global_ref: global_ref.clone(),
            levels: expr_levels
                .iter()
                .map(|level| subst_level(level, params, levels))
                .collect(),
        },
        ReferenceCoreExpr::App(fun, arg) => ReferenceCoreExpr::App(
            Box::new(subst_levels_expr(fun, params, levels)),
            Box::new(subst_levels_expr(arg, params, levels)),
        ),
        ReferenceCoreExpr::Lam { ty, body } => ReferenceCoreExpr::Lam {
            ty: Box::new(subst_levels_expr(ty, params, levels)),
            body: Box::new(subst_levels_expr(body, params, levels)),
        },
        ReferenceCoreExpr::Pi { ty, body } => ReferenceCoreExpr::Pi {
            ty: Box::new(subst_levels_expr(ty, params, levels)),
            body: Box::new(subst_levels_expr(body, params, levels)),
        },
        ReferenceCoreExpr::Let { ty, value, body } => ReferenceCoreExpr::Let {
            ty: Box::new(subst_levels_expr(ty, params, levels)),
            value: Box::new(subst_levels_expr(value, params, levels)),
            body: Box::new(subst_levels_expr(body, params, levels)),
        },
    }
}

fn subst_level(
    level: &ReferenceCoreLevel,
    params: &[ReferenceModuleName],
    levels: &[ReferenceCoreLevel],
) -> ReferenceCoreLevel {
    match level {
        ReferenceCoreLevel::Zero => ReferenceCoreLevel::Zero,
        ReferenceCoreLevel::Succ(inner) => {
            ReferenceCoreLevel::Succ(Box::new(subst_level(inner, params, levels)))
        }
        ReferenceCoreLevel::Max(lhs, rhs) => ReferenceCoreLevel::Max(
            Box::new(subst_level(lhs, params, levels)),
            Box::new(subst_level(rhs, params, levels)),
        ),
        ReferenceCoreLevel::IMax(lhs, rhs) => ReferenceCoreLevel::IMax(
            Box::new(subst_level(lhs, params, levels)),
            Box::new(subst_level(rhs, params, levels)),
        ),
        ReferenceCoreLevel::Param(name) => params
            .iter()
            .position(|param| param == name)
            .map(|index| levels[index].clone())
            .unwrap_or_else(|| ReferenceCoreLevel::Param(name.clone())),
    }
}

fn shift(
    expr: &ReferenceCoreExpr,
    amount: i32,
    cutoff: u32,
    offset: usize,
) -> DecodeResult<ReferenceCoreExpr> {
    match expr {
        ReferenceCoreExpr::Sort(level) => Ok(ReferenceCoreExpr::Sort(level.clone())),
        ReferenceCoreExpr::BVar(index) => {
            if *index < cutoff {
                Ok(ReferenceCoreExpr::BVar(*index))
            } else {
                let shifted = *index as i32 + amount;
                if shifted < 0 {
                    Err(ReferenceCheckError::type_check(
                        ReferenceCertificateSection::Declarations,
                        offset,
                        ReferenceCheckReason::InvalidBVar,
                    ))
                } else {
                    Ok(ReferenceCoreExpr::BVar(shifted as u32))
                }
            }
        }
        ReferenceCoreExpr::Const { global_ref, levels } => Ok(ReferenceCoreExpr::Const {
            global_ref: global_ref.clone(),
            levels: levels.clone(),
        }),
        ReferenceCoreExpr::App(fun, arg) => Ok(ReferenceCoreExpr::App(
            Box::new(shift(fun, amount, cutoff, offset)?),
            Box::new(shift(arg, amount, cutoff, offset)?),
        )),
        ReferenceCoreExpr::Lam { ty, body } => Ok(ReferenceCoreExpr::Lam {
            ty: Box::new(shift(ty, amount, cutoff, offset)?),
            body: Box::new(shift(body, amount, cutoff + 1, offset)?),
        }),
        ReferenceCoreExpr::Pi { ty, body } => Ok(ReferenceCoreExpr::Pi {
            ty: Box::new(shift(ty, amount, cutoff, offset)?),
            body: Box::new(shift(body, amount, cutoff + 1, offset)?),
        }),
        ReferenceCoreExpr::Let { ty, value, body } => Ok(ReferenceCoreExpr::Let {
            ty: Box::new(shift(ty, amount, cutoff, offset)?),
            value: Box::new(shift(value, amount, cutoff, offset)?),
            body: Box::new(shift(body, amount, cutoff + 1, offset)?),
        }),
    }
}

fn substitute(
    expr: &ReferenceCoreExpr,
    target: u32,
    replacement: &ReferenceCoreExpr,
    offset: usize,
) -> DecodeResult<ReferenceCoreExpr> {
    match expr {
        ReferenceCoreExpr::Sort(level) => Ok(ReferenceCoreExpr::Sort(level.clone())),
        ReferenceCoreExpr::BVar(index) if *index == target => {
            shift(replacement, target as i32, 0, offset)
        }
        ReferenceCoreExpr::BVar(index) if *index > target => Ok(ReferenceCoreExpr::BVar(index - 1)),
        ReferenceCoreExpr::BVar(index) => Ok(ReferenceCoreExpr::BVar(*index)),
        ReferenceCoreExpr::Const { global_ref, levels } => Ok(ReferenceCoreExpr::Const {
            global_ref: global_ref.clone(),
            levels: levels.clone(),
        }),
        ReferenceCoreExpr::App(fun, arg) => Ok(ReferenceCoreExpr::App(
            Box::new(substitute(fun, target, replacement, offset)?),
            Box::new(substitute(arg, target, replacement, offset)?),
        )),
        ReferenceCoreExpr::Lam { ty, body } => Ok(ReferenceCoreExpr::Lam {
            ty: Box::new(substitute(ty, target, replacement, offset)?),
            body: Box::new(substitute(body, target + 1, replacement, offset)?),
        }),
        ReferenceCoreExpr::Pi { ty, body } => Ok(ReferenceCoreExpr::Pi {
            ty: Box::new(substitute(ty, target, replacement, offset)?),
            body: Box::new(substitute(body, target + 1, replacement, offset)?),
        }),
        ReferenceCoreExpr::Let { ty, value, body } => Ok(ReferenceCoreExpr::Let {
            ty: Box::new(substitute(ty, target, replacement, offset)?),
            value: Box::new(substitute(value, target, replacement, offset)?),
            body: Box::new(substitute(body, target + 1, replacement, offset)?),
        }),
    }
}

fn instantiate(
    body: &ReferenceCoreExpr,
    value: &ReferenceCoreExpr,
    offset: usize,
) -> DecodeResult<ReferenceCoreExpr> {
    substitute(body, 0, value, offset)
}

#[derive(Default)]
struct UsedTables {
    names: BTreeSet<ReferenceModuleName>,
    levels: BTreeSet<usize>,
    terms: BTreeSet<usize>,
}

impl UsedTables {
    fn new() -> Self {
        Self::default()
    }
}

fn push_term(term: usize, used: &mut UsedTables, stack: &mut Vec<usize>) {
    if used.terms.insert(term) {
        stack.push(term);
    }
}

fn push_level(level: usize, used: &mut UsedTables, stack: &mut Vec<usize>) {
    if used.levels.insert(level) {
        stack.push(level);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ImportEntry {
    module: ReferenceModuleName,
    export_hash: ReferenceHash,
    certificate_hash: Option<ReferenceHash>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum LevelNode {
    Zero,
    Succ(usize),
    Max(usize, usize),
    IMax(usize, usize),
    Param(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum TermNode {
    Sort(usize),
    BVar(u32),
    Const {
        global_ref: GlobalRef,
        levels: Vec<usize>,
    },
    App(usize, usize),
    Lam {
        ty: usize,
        body: usize,
    },
    Pi {
        ty: usize,
        body: usize,
    },
    Let {
        ty: usize,
        value: usize,
        body: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum GlobalRef {
    Builtin {
        name: usize,
        decl_interface_hash: ReferenceHash,
    },
    Imported {
        import_index: usize,
        name: usize,
        decl_interface_hash: ReferenceHash,
    },
    Local {
        decl_index: usize,
    },
    LocalGenerated {
        decl_index: usize,
        name: usize,
    },
}

#[derive(Clone, Debug)]
struct DeclCert {
    decl: DeclPayload,
    dependencies: Vec<DependencyEntry>,
    axiom_dependencies: Vec<AxiomRef>,
    hashes: DeclHashes,
}

#[derive(Clone, Debug)]
enum DeclPayload {
    Axiom {
        name: usize,
        universe_params: Vec<usize>,
        ty: usize,
    },
    Def {
        name: usize,
        universe_params: Vec<usize>,
        ty: usize,
        value: usize,
        reducibility: CertReducibility,
    },
    Theorem {
        name: usize,
        universe_params: Vec<usize>,
        ty: usize,
        proof: usize,
        opacity: Opacity,
    },
    Inductive {
        name: usize,
        universe_params: Vec<usize>,
        params: Vec<BinderType>,
        indices: Vec<BinderType>,
        sort: usize,
        constructors: Vec<ConstructorSpec>,
        recursor: Option<RecursorSpec>,
    },
}

impl DeclPayload {
    fn name_id(&self) -> usize {
        match self {
            Self::Axiom { name, .. }
            | Self::Def { name, .. }
            | Self::Theorem { name, .. }
            | Self::Inductive { name, .. } => *name,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct BinderType {
    ty: usize,
}

#[derive(Clone, Copy, Debug)]
struct ConstructorSpec {
    name: usize,
    ty: usize,
}

#[derive(Clone, Debug)]
struct RecursorSpec {
    name: usize,
    universe_params: Vec<usize>,
    ty: usize,
    rules: RecursorRulesSpec,
}

#[derive(Clone, Copy, Debug)]
struct RecursorRulesSpec {
    minor_start: usize,
    major_index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CertReducibility {
    Reducible,
    Opaque,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Opacity {
    Opaque,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct DependencyEntry {
    global_ref: GlobalRef,
    decl_interface_hash: ReferenceHash,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct AxiomRef {
    global_ref: GlobalRef,
    name: usize,
    decl_interface_hash: ReferenceHash,
}

#[derive(Clone, Debug)]
struct DeclHashes {
    decl_interface_hash: ReferenceHash,
    decl_certificate_hash: ReferenceHash,
    decl_interface_hash_offset: usize,
    decl_certificate_hash_offset: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExportEntry {
    name: usize,
    kind: ExportKind,
    universe_params: Vec<usize>,
    ty: usize,
    body: Option<usize>,
    type_hash: ReferenceHash,
    body_hash: Option<ReferenceHash>,
    reducibility: Option<CertReducibility>,
    opacity: Option<Opacity>,
    decl_interface_hash: ReferenceHash,
    axiom_dependencies: Vec<AxiomRef>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExportKind {
    Axiom,
    Def,
    Theorem,
    Inductive,
    Constructor,
    Recursor,
}

#[derive(Clone, Debug)]
struct AxiomReport {
    per_declaration: Vec<DeclAxiomReport>,
    module_axioms: Vec<AxiomRef>,
    module_axioms_offset: usize,
}

#[derive(Clone, Debug)]
struct DeclAxiomReport {
    decl_index: usize,
    direct_axioms: Vec<AxiomRef>,
    transitive_axioms: Vec<AxiomRef>,
    offset: usize,
}

#[derive(Clone, Copy, Debug)]
struct ModuleHashOffsets {
    export_hash_offset: usize,
    axiom_report_hash_offset: usize,
    certificate_hash_offset: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum RawLevel {
    Zero,
    Succ(Box<RawLevel>),
    Max(Box<RawLevel>, Box<RawLevel>),
    IMax(Box<RawLevel>, Box<RawLevel>),
    Param(String),
}

fn normalize_level(level: RawLevel) -> RawLevel {
    match level {
        RawLevel::Zero | RawLevel::Param(_) => level,
        RawLevel::Succ(inner) => RawLevel::Succ(Box::new(normalize_level(*inner))),
        RawLevel::Max(lhs, rhs) => {
            let lhs = normalize_level(*lhs);
            let rhs = normalize_level(*rhs);
            if lhs == rhs {
                return lhs;
            }
            if lhs == RawLevel::Zero {
                return rhs;
            }
            if rhs == RawLevel::Zero {
                return lhs;
            }
            match (level_as_nat(&lhs), level_as_nat(&rhs)) {
                (Some(lhs_nat), Some(rhs_nat)) => level_from_nat(lhs_nat.max(rhs_nat)),
                _ if rhs < lhs => RawLevel::Max(Box::new(rhs), Box::new(lhs)),
                _ => RawLevel::Max(Box::new(lhs), Box::new(rhs)),
            }
        }
        RawLevel::IMax(lhs, rhs) => {
            let lhs = normalize_level(*lhs);
            let rhs = normalize_level(*rhs);
            match rhs {
                RawLevel::Zero => RawLevel::Zero,
                RawLevel::Succ(inner) => normalize_level(RawLevel::Max(
                    Box::new(lhs),
                    Box::new(RawLevel::Succ(inner)),
                )),
                rhs => RawLevel::IMax(Box::new(lhs), Box::new(rhs)),
            }
        }
    }
}

fn level_as_nat(level: &RawLevel) -> Option<u32> {
    match level {
        RawLevel::Zero => Some(0),
        RawLevel::Succ(inner) => Some(level_as_nat(inner)? + 1),
        RawLevel::Max(_, _) | RawLevel::IMax(_, _) | RawLevel::Param(_) => None,
    }
}

fn level_from_nat(n: u32) -> RawLevel {
    (0..n).fold(RawLevel::Zero, |level, _| RawLevel::Succ(Box::new(level)))
}

fn raw_level_from_node(
    node: &LevelNode,
    previous: &[RawLevel],
    names: &[Located<ReferenceModuleName>],
) -> DecodeResult<RawLevel> {
    Ok(match node {
        LevelNode::Zero => RawLevel::Zero,
        LevelNode::Succ(inner) => RawLevel::Succ(Box::new(previous[*inner].clone())),
        LevelNode::Max(lhs, rhs) => RawLevel::Max(
            Box::new(previous[*lhs].clone()),
            Box::new(previous[*rhs].clone()),
        ),
        LevelNode::IMax(lhs, rhs) => RawLevel::IMax(
            Box::new(previous[*lhs].clone()),
            Box::new(previous[*rhs].clone()),
        ),
        LevelNode::Param(name) => RawLevel::Param(
            names
                .get(*name)
                .ok_or_else(|| {
                    ReferenceCheckError::malformed(
                        ReferenceCertificateSection::LevelTable,
                        0,
                        ReferenceCheckReason::DanglingReference,
                    )
                })?
                .value
                .dotted(),
        ),
    })
}

struct Decoder<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Decoder<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    const fn offset(&self) -> usize {
        self.offset
    }

    fn is_done(&self) -> bool {
        self.offset == self.bytes.len()
    }

    fn module_certificate(&mut self) -> DecodeResult<DecodedModuleCertificate> {
        let header = self.header()?;
        let imports = self.imports()?;
        let name_table = self.name_table()?;
        let level_table = self.level_table()?;
        let term_table = self.term_table()?;
        let declarations = self.declarations()?;
        let export_block = self.export_block()?;
        let axiom_report = self.axiom_report()?;
        let export_hash_offset = self.offset;
        let export_hash = self.hash(ReferenceCertificateSection::Hashes)?;
        let axiom_report_hash_offset = self.offset;
        let axiom_report_hash = self.hash(ReferenceCertificateSection::Hashes)?;
        let certificate_hash_offset = self.offset;
        let certificate_hash = self.hash(ReferenceCertificateSection::Hashes)?;
        let hashes = ReferenceModuleHashes {
            export_hash,
            axiom_report_hash,
            certificate_hash,
        };
        let hash_offsets = ModuleHashOffsets {
            export_hash_offset,
            axiom_report_hash_offset,
            certificate_hash_offset,
        };
        Ok(DecodedModuleCertificate {
            header,
            imports,
            name_table,
            level_table,
            term_table,
            declarations,
            export_block,
            axiom_report,
            hashes,
            hash_offsets,
        })
    }

    fn header(&mut self) -> DecodeResult<ReferenceCertificateHeader> {
        let format = self.string(ReferenceCertificateSection::HeaderFormat)?;
        if format != REFERENCE_CERTIFICATE_FORMAT {
            return Err(ReferenceCheckError::malformed(
                ReferenceCertificateSection::HeaderFormat,
                self.offset,
                ReferenceCheckReason::FormatMismatch,
            ));
        }
        let core_spec = self.string(ReferenceCertificateSection::HeaderCoreSpec)?;
        if core_spec != REFERENCE_CORE_SPEC {
            return Err(ReferenceCheckError::malformed(
                ReferenceCertificateSection::HeaderCoreSpec,
                self.offset,
                ReferenceCheckReason::CoreSpecMismatch,
            ));
        }
        let module = self.name(ReferenceCertificateSection::HeaderModule)?;
        Ok(ReferenceCertificateHeader {
            format,
            core_spec,
            module,
        })
    }

    fn imports(&mut self) -> DecodeResult<Vec<Located<ImportEntry>>> {
        let len = self.bounded_len(ReferenceCertificateSection::Imports)?;
        let mut imports = Vec::with_capacity(len);
        for _ in 0..len {
            let offset = self.offset;
            imports.push(Located {
                value: ImportEntry {
                    module: self.name(ReferenceCertificateSection::Imports)?,
                    export_hash: self.hash(ReferenceCertificateSection::Imports)?,
                    certificate_hash: self.option_hash(ReferenceCertificateSection::Imports)?,
                },
                offset,
            });
        }
        Ok(imports)
    }

    fn name_table(&mut self) -> DecodeResult<Vec<Located<ReferenceModuleName>>> {
        let len = self.bounded_len(ReferenceCertificateSection::NameTable)?;
        let mut names = Vec::with_capacity(len);
        for _ in 0..len {
            let offset = self.offset;
            names.push(Located {
                value: self.name(ReferenceCertificateSection::NameTable)?,
                offset,
            });
        }
        Ok(names)
    }

    fn level_table(&mut self) -> DecodeResult<Vec<Located<LevelNode>>> {
        let len = self.bounded_len(ReferenceCertificateSection::LevelTable)?;
        let mut levels = Vec::with_capacity(len);
        for _ in 0..len {
            let offset = self.offset;
            let tag = self.byte(ReferenceCertificateSection::LevelTable)?;
            let value = match tag {
                0x00 => LevelNode::Zero,
                0x01 => LevelNode::Succ(self.usize(ReferenceCertificateSection::LevelTable)?),
                0x02 => LevelNode::Max(
                    self.usize(ReferenceCertificateSection::LevelTable)?,
                    self.usize(ReferenceCertificateSection::LevelTable)?,
                ),
                0x03 => LevelNode::IMax(
                    self.usize(ReferenceCertificateSection::LevelTable)?,
                    self.usize(ReferenceCertificateSection::LevelTable)?,
                ),
                0x04 => LevelNode::Param(self.usize(ReferenceCertificateSection::LevelTable)?),
                tag => {
                    return Err(ReferenceCheckError::malformed(
                        ReferenceCertificateSection::LevelTable,
                        offset,
                        ReferenceCheckReason::UnknownTag { tag },
                    ));
                }
            };
            levels.push(Located { value, offset });
        }
        Ok(levels)
    }

    fn term_table(&mut self) -> DecodeResult<Vec<Located<TermNode>>> {
        let len = self.bounded_len(ReferenceCertificateSection::TermTable)?;
        let mut terms = Vec::with_capacity(len);
        for _ in 0..len {
            let offset = self.offset;
            let tag = self.byte(ReferenceCertificateSection::TermTable)?;
            let value = match tag {
                0x00 => TermNode::Sort(self.usize(ReferenceCertificateSection::TermTable)?),
                0x01 => TermNode::BVar(self.u32(ReferenceCertificateSection::TermTable)?),
                0x02 => TermNode::Const {
                    global_ref: self.global_ref(ReferenceCertificateSection::TermTable)?,
                    levels: self.usize_vec(ReferenceCertificateSection::TermTable)?,
                },
                0x03 => TermNode::App(
                    self.usize(ReferenceCertificateSection::TermTable)?,
                    self.usize(ReferenceCertificateSection::TermTable)?,
                ),
                0x04 => TermNode::Lam {
                    ty: self.usize(ReferenceCertificateSection::TermTable)?,
                    body: self.usize(ReferenceCertificateSection::TermTable)?,
                },
                0x05 => TermNode::Pi {
                    ty: self.usize(ReferenceCertificateSection::TermTable)?,
                    body: self.usize(ReferenceCertificateSection::TermTable)?,
                },
                0x06 => TermNode::Let {
                    ty: self.usize(ReferenceCertificateSection::TermTable)?,
                    value: self.usize(ReferenceCertificateSection::TermTable)?,
                    body: self.usize(ReferenceCertificateSection::TermTable)?,
                },
                tag => {
                    return Err(ReferenceCheckError::malformed(
                        ReferenceCertificateSection::TermTable,
                        offset,
                        ReferenceCheckReason::UnknownTag { tag },
                    ));
                }
            };
            terms.push(Located { value, offset });
        }
        Ok(terms)
    }

    fn declarations(&mut self) -> DecodeResult<Vec<Located<DeclCert>>> {
        let len = self.bounded_len(ReferenceCertificateSection::Declarations)?;
        let mut declarations = Vec::with_capacity(len);
        for _ in 0..len {
            let offset = self.offset;
            let decl = self.decl_payload()?;
            let dependencies =
                self.dependency_entries(ReferenceCertificateSection::Declarations)?;
            let axiom_dependencies = self.axiom_refs(ReferenceCertificateSection::Declarations)?;
            let decl_interface_hash_offset = self.offset;
            let decl_interface_hash = self.hash(ReferenceCertificateSection::Declarations)?;
            let decl_certificate_hash_offset = self.offset;
            let decl_certificate_hash = self.hash(ReferenceCertificateSection::Declarations)?;
            declarations.push(Located {
                value: DeclCert {
                    decl,
                    dependencies,
                    axiom_dependencies,
                    hashes: DeclHashes {
                        decl_interface_hash,
                        decl_certificate_hash,
                        decl_interface_hash_offset,
                        decl_certificate_hash_offset,
                    },
                },
                offset,
            });
        }
        Ok(declarations)
    }

    fn decl_payload(&mut self) -> DecodeResult<DeclPayload> {
        let offset = self.offset;
        let tag = self.byte(ReferenceCertificateSection::Declarations)?;
        Ok(match tag {
            0x00 => DeclPayload::Axiom {
                name: self.usize(ReferenceCertificateSection::Declarations)?,
                universe_params: self.usize_vec(ReferenceCertificateSection::Declarations)?,
                ty: self.usize(ReferenceCertificateSection::Declarations)?,
            },
            0x01 => DeclPayload::Def {
                name: self.usize(ReferenceCertificateSection::Declarations)?,
                universe_params: self.usize_vec(ReferenceCertificateSection::Declarations)?,
                ty: self.usize(ReferenceCertificateSection::Declarations)?,
                value: self.usize(ReferenceCertificateSection::Declarations)?,
                reducibility: self.reducibility(ReferenceCertificateSection::Declarations)?,
            },
            0x02 => DeclPayload::Theorem {
                name: self.usize(ReferenceCertificateSection::Declarations)?,
                universe_params: self.usize_vec(ReferenceCertificateSection::Declarations)?,
                ty: self.usize(ReferenceCertificateSection::Declarations)?,
                proof: self.usize(ReferenceCertificateSection::Declarations)?,
                opacity: self.opacity(ReferenceCertificateSection::Declarations)?,
            },
            0x03 => {
                let name = self.usize(ReferenceCertificateSection::Declarations)?;
                let universe_params = self.usize_vec(ReferenceCertificateSection::Declarations)?;
                let params = self.binder_types()?;
                let indices = self.binder_types()?;
                let sort = self.usize(ReferenceCertificateSection::Declarations)?;
                let constructors_len =
                    self.bounded_len(ReferenceCertificateSection::Declarations)?;
                let mut constructors = Vec::with_capacity(constructors_len);
                for _ in 0..constructors_len {
                    constructors.push(ConstructorSpec {
                        name: self.usize(ReferenceCertificateSection::Declarations)?,
                        ty: self.usize(ReferenceCertificateSection::Declarations)?,
                    });
                }
                let recursor_offset = self.offset;
                let recursor = match self.byte(ReferenceCertificateSection::Declarations)? {
                    0x00 => None,
                    0x01 => Some(RecursorSpec {
                        name: self.usize(ReferenceCertificateSection::Declarations)?,
                        universe_params: self
                            .usize_vec(ReferenceCertificateSection::Declarations)?,
                        ty: self.usize(ReferenceCertificateSection::Declarations)?,
                        rules: RecursorRulesSpec {
                            minor_start: self.usize(ReferenceCertificateSection::Declarations)?,
                            major_index: self.usize(ReferenceCertificateSection::Declarations)?,
                        },
                    }),
                    tag => {
                        return Err(ReferenceCheckError::malformed(
                            ReferenceCertificateSection::Declarations,
                            recursor_offset,
                            ReferenceCheckReason::UnknownTag { tag },
                        ));
                    }
                };
                DeclPayload::Inductive {
                    name,
                    universe_params,
                    params,
                    indices,
                    sort,
                    constructors,
                    recursor,
                }
            }
            tag => {
                return Err(ReferenceCheckError::malformed(
                    ReferenceCertificateSection::Declarations,
                    offset,
                    ReferenceCheckReason::UnknownTag { tag },
                ));
            }
        })
    }

    fn binder_types(&mut self) -> DecodeResult<Vec<BinderType>> {
        let len = self.bounded_len(ReferenceCertificateSection::Declarations)?;
        let mut binders = Vec::with_capacity(len);
        for _ in 0..len {
            binders.push(BinderType {
                ty: self.usize(ReferenceCertificateSection::Declarations)?,
            });
        }
        Ok(binders)
    }

    fn export_block(&mut self) -> DecodeResult<Vec<Located<ExportEntry>>> {
        let len = self.bounded_len(ReferenceCertificateSection::ExportBlock)?;
        let mut exports = Vec::with_capacity(len);
        for _ in 0..len {
            let offset = self.offset;
            let name = self.usize(ReferenceCertificateSection::ExportBlock)?;
            let kind_offset = self.offset;
            let kind = match self.byte(ReferenceCertificateSection::ExportBlock)? {
                0x00 => ExportKind::Axiom,
                0x01 => ExportKind::Def,
                0x02 => ExportKind::Theorem,
                0x03 => ExportKind::Inductive,
                0x04 => ExportKind::Constructor,
                0x05 => ExportKind::Recursor,
                tag => {
                    return Err(ReferenceCheckError::malformed(
                        ReferenceCertificateSection::ExportBlock,
                        kind_offset,
                        ReferenceCheckReason::UnknownTag { tag },
                    ));
                }
            };
            exports.push(Located {
                value: ExportEntry {
                    name,
                    kind,
                    universe_params: self.usize_vec(ReferenceCertificateSection::ExportBlock)?,
                    ty: self.usize(ReferenceCertificateSection::ExportBlock)?,
                    body: self.option_usize(ReferenceCertificateSection::ExportBlock)?,
                    type_hash: self.hash(ReferenceCertificateSection::ExportBlock)?,
                    body_hash: self.option_hash(ReferenceCertificateSection::ExportBlock)?,
                    reducibility: self
                        .option_reducibility(ReferenceCertificateSection::ExportBlock)?,
                    opacity: self.option_opacity(ReferenceCertificateSection::ExportBlock)?,
                    decl_interface_hash: self.hash(ReferenceCertificateSection::ExportBlock)?,
                    axiom_dependencies: self
                        .axiom_refs(ReferenceCertificateSection::ExportBlock)?,
                },
                offset,
            });
        }
        Ok(exports)
    }

    fn axiom_report(&mut self) -> DecodeResult<AxiomReport> {
        let len = self.bounded_len(ReferenceCertificateSection::AxiomReport)?;
        let mut per_declaration = Vec::with_capacity(len);
        for _ in 0..len {
            let offset = self.offset;
            per_declaration.push(DeclAxiomReport {
                decl_index: self.usize(ReferenceCertificateSection::AxiomReport)?,
                direct_axioms: self.axiom_refs(ReferenceCertificateSection::AxiomReport)?,
                transitive_axioms: self.axiom_refs(ReferenceCertificateSection::AxiomReport)?,
                offset,
            });
        }
        let module_axioms_offset = self.offset;
        let module_axioms = self.axiom_refs(ReferenceCertificateSection::AxiomReport)?;
        Ok(AxiomReport {
            per_declaration,
            module_axioms,
            module_axioms_offset,
        })
    }

    fn dependency_entries(
        &mut self,
        section: ReferenceCertificateSection,
    ) -> DecodeResult<Vec<DependencyEntry>> {
        let len = self.bounded_len(section)?;
        let mut entries = Vec::with_capacity(len);
        for _ in 0..len {
            entries.push(DependencyEntry {
                global_ref: self.global_ref(section)?,
                decl_interface_hash: self.hash(section)?,
            });
        }
        Ok(entries)
    }

    fn axiom_refs(&mut self, section: ReferenceCertificateSection) -> DecodeResult<Vec<AxiomRef>> {
        let len = self.bounded_len(section)?;
        let mut axioms = Vec::with_capacity(len);
        for _ in 0..len {
            axioms.push(AxiomRef {
                global_ref: self.global_ref(section)?,
                name: self.usize(section)?,
                decl_interface_hash: self.hash(section)?,
            });
        }
        Ok(axioms)
    }

    fn global_ref(&mut self, section: ReferenceCertificateSection) -> DecodeResult<GlobalRef> {
        let offset = self.offset;
        let tag = self.byte(section)?;
        Ok(match tag {
            0x03 => GlobalRef::Builtin {
                name: self.usize(section)?,
                decl_interface_hash: self.hash(section)?,
            },
            0x00 => GlobalRef::Imported {
                import_index: self.usize(section)?,
                name: self.usize(section)?,
                decl_interface_hash: self.hash(section)?,
            },
            0x01 => GlobalRef::Local {
                decl_index: self.usize(section)?,
            },
            0x02 => GlobalRef::LocalGenerated {
                decl_index: self.usize(section)?,
                name: self.usize(section)?,
            },
            tag => {
                return Err(ReferenceCheckError::malformed(
                    section,
                    offset,
                    ReferenceCheckReason::UnknownTag { tag },
                ));
            }
        })
    }

    fn reducibility(
        &mut self,
        section: ReferenceCertificateSection,
    ) -> DecodeResult<CertReducibility> {
        let offset = self.offset;
        Ok(match self.byte(section)? {
            0x00 => CertReducibility::Reducible,
            0x01 => CertReducibility::Opaque,
            tag => {
                return Err(ReferenceCheckError::malformed(
                    section,
                    offset,
                    ReferenceCheckReason::UnknownTag { tag },
                ));
            }
        })
    }

    fn option_reducibility(
        &mut self,
        section: ReferenceCertificateSection,
    ) -> DecodeResult<Option<CertReducibility>> {
        let offset = self.offset;
        match self.byte(section)? {
            0x00 => Ok(None),
            0x01 => Ok(Some(self.reducibility(section)?)),
            tag => Err(ReferenceCheckError::malformed(
                section,
                offset,
                ReferenceCheckReason::UnknownTag { tag },
            )),
        }
    }

    fn opacity(&mut self, section: ReferenceCertificateSection) -> DecodeResult<Opacity> {
        let offset = self.offset;
        Ok(match self.byte(section)? {
            0x00 => Opacity::Opaque,
            tag => {
                return Err(ReferenceCheckError::malformed(
                    section,
                    offset,
                    ReferenceCheckReason::UnknownTag { tag },
                ));
            }
        })
    }

    fn option_opacity(
        &mut self,
        section: ReferenceCertificateSection,
    ) -> DecodeResult<Option<Opacity>> {
        let offset = self.offset;
        match self.byte(section)? {
            0x00 => Ok(None),
            0x01 => Ok(Some(self.opacity(section)?)),
            tag => Err(ReferenceCheckError::malformed(
                section,
                offset,
                ReferenceCheckReason::UnknownTag { tag },
            )),
        }
    }

    fn name(&mut self, section: ReferenceCertificateSection) -> DecodeResult<ReferenceModuleName> {
        let len = self.bounded_len(section)?;
        if len == 0 {
            return Err(ReferenceCheckError::malformed(
                section,
                self.offset,
                ReferenceCheckReason::EmptyModuleName,
            ));
        }
        let mut components = Vec::with_capacity(len);
        for _ in 0..len {
            let component = self.string(section)?;
            if component.is_empty() {
                return Err(ReferenceCheckError::malformed(
                    section,
                    self.offset,
                    ReferenceCheckReason::EmptyModuleNameComponent,
                ));
            }
            if component.contains('.') {
                return Err(ReferenceCheckError::malformed(
                    section,
                    self.offset,
                    ReferenceCheckReason::DottedNameComponent,
                ));
            }
            components.push(component);
        }
        ReferenceModuleName::new(components).map_err(|_| {
            ReferenceCheckError::malformed(
                section,
                self.offset,
                ReferenceCheckReason::EmptyModuleName,
            )
        })
    }

    fn string(&mut self, section: ReferenceCertificateSection) -> DecodeResult<String> {
        let len = self.usize(section)?;
        let start = self.offset;
        let bytes = self.take(len, section)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| {
            ReferenceCheckError::malformed(section, start, ReferenceCheckReason::InvalidUtf8)
        })
    }

    fn usize_vec(&mut self, section: ReferenceCertificateSection) -> DecodeResult<Vec<usize>> {
        let len = self.bounded_len(section)?;
        let mut values = Vec::with_capacity(len);
        for _ in 0..len {
            values.push(self.usize(section)?);
        }
        Ok(values)
    }

    fn option_usize(
        &mut self,
        section: ReferenceCertificateSection,
    ) -> DecodeResult<Option<usize>> {
        let offset = self.offset;
        match self.byte(section)? {
            0x00 => Ok(None),
            0x01 => Ok(Some(self.usize(section)?)),
            tag => Err(ReferenceCheckError::malformed(
                section,
                offset,
                ReferenceCheckReason::UnknownTag { tag },
            )),
        }
    }

    fn option_hash(
        &mut self,
        section: ReferenceCertificateSection,
    ) -> DecodeResult<Option<ReferenceHash>> {
        let offset = self.offset;
        match self.byte(section)? {
            0x00 => Ok(None),
            0x01 => Ok(Some(self.hash(section)?)),
            tag => Err(ReferenceCheckError::malformed(
                section,
                offset,
                ReferenceCheckReason::UnknownTag { tag },
            )),
        }
    }

    fn hash(&mut self, section: ReferenceCertificateSection) -> DecodeResult<ReferenceHash> {
        let bytes = self.take(32, section)?;
        let mut hash = [0; 32];
        hash.copy_from_slice(bytes);
        Ok(hash)
    }

    fn bounded_len(&mut self, section: ReferenceCertificateSection) -> DecodeResult<usize> {
        let len = self.usize(section)?;
        let remaining = self.bytes.len().saturating_sub(self.offset);
        if len > remaining {
            return Err(ReferenceCheckError::malformed(
                section,
                self.offset,
                ReferenceCheckReason::UnexpectedEof,
            ));
        }
        Ok(len)
    }

    fn u32(&mut self, section: ReferenceCertificateSection) -> DecodeResult<u32> {
        let offset = self.offset;
        let value = self.uvar(section)?;
        u32::try_from(value).map_err(|_| {
            ReferenceCheckError::malformed(section, offset, ReferenceCheckReason::LengthOverflow)
        })
    }

    fn usize(&mut self, section: ReferenceCertificateSection) -> DecodeResult<usize> {
        let offset = self.offset;
        let value = self.uvar(section)?;
        usize::try_from(value).map_err(|_| {
            ReferenceCheckError::malformed(section, offset, ReferenceCheckReason::LengthOverflow)
        })
    }

    fn uvar(&mut self, section: ReferenceCertificateSection) -> DecodeResult<u64> {
        let start = self.offset;
        let mut shift = 0u32;
        let mut value = 0u64;
        loop {
            let byte = self.byte(section)?;
            value |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                if encode_uvar(value) != self.bytes[start..self.offset] {
                    return Err(ReferenceCheckError::malformed(
                        section,
                        start,
                        ReferenceCheckReason::NonCanonicalUvar,
                    ));
                }
                return Ok(value);
            }
            shift += 7;
            if shift >= 64 {
                return Err(ReferenceCheckError::malformed(
                    section,
                    start,
                    ReferenceCheckReason::UvarOverflow,
                ));
            }
        }
    }

    fn byte(&mut self, section: ReferenceCertificateSection) -> DecodeResult<u8> {
        let byte = *self.bytes.get(self.offset).ok_or_else(|| {
            ReferenceCheckError::malformed(
                section,
                self.offset,
                ReferenceCheckReason::UnexpectedEof,
            )
        })?;
        self.offset += 1;
        Ok(byte)
    }

    fn take(&mut self, len: usize, section: ReferenceCertificateSection) -> DecodeResult<&'a [u8]> {
        let end = self.offset.checked_add(len).ok_or_else(|| {
            ReferenceCheckError::malformed(
                section,
                self.offset,
                ReferenceCheckReason::LengthOverflow,
            )
        })?;
        let bytes = self.bytes.get(self.offset..end).ok_or_else(|| {
            ReferenceCheckError::malformed(
                section,
                self.offset,
                ReferenceCheckReason::UnexpectedEof,
            )
        })?;
        self.offset = end;
        Ok(bytes)
    }
}

fn import_order_key(
    import: &ImportEntry,
) -> (ReferenceModuleName, ReferenceHash, Option<ReferenceHash>) {
    (
        import.module.clone(),
        import.export_hash,
        import.certificate_hash,
    )
}

fn ensure_strict_order<T: Ord>(
    values: &[T],
    section: ReferenceCertificateSection,
    offset: usize,
) -> DecodeResult<()> {
    if values.windows(2).all(|pair| pair[0] < pair[1]) {
        Ok(())
    } else {
        Err(ReferenceCheckError::malformed(
            section,
            offset,
            ReferenceCheckReason::NonCanonicalOrder,
        ))
    }
}

fn level_node_height(node: &LevelNode, levels: &[Located<LevelNode>]) -> DecodeResult<usize> {
    Ok(match node {
        LevelNode::Zero | LevelNode::Param(_) => 0,
        LevelNode::Succ(inner) => level_node_height(&levels[*inner].value, levels)? + 1,
        LevelNode::Max(lhs, rhs) | LevelNode::IMax(lhs, rhs) => {
            level_node_height(&levels[*lhs].value, levels)?
                .max(level_node_height(&levels[*rhs].value, levels)?)
                + 1
        }
    })
}

fn term_node_height(node: &TermNode, terms: &[Located<TermNode>]) -> DecodeResult<usize> {
    Ok(match node {
        TermNode::Sort(_) | TermNode::BVar(_) | TermNode::Const { .. } => 0,
        TermNode::App(fun, arg) => {
            term_node_height(&terms[*fun].value, terms)?
                .max(term_node_height(&terms[*arg].value, terms)?)
                + 1
        }
        TermNode::Lam { ty, body } | TermNode::Pi { ty, body } => {
            term_node_height(&terms[*ty].value, terms)?
                .max(term_node_height(&terms[*body].value, terms)?)
                + 1
        }
        TermNode::Let { ty, value, body } => {
            term_node_height(&terms[*ty].value, terms)?
                .max(term_node_height(&terms[*value].value, terms)?)
                .max(term_node_height(&terms[*body].value, terms)?)
                + 1
        }
    })
}

fn level_node_key(
    level: &LevelNode,
    child_hashes: &[ReferenceHash],
    names: &[Located<ReferenceModuleName>],
) -> DecodeResult<Vec<u8>> {
    let mut payload = Vec::new();
    match level {
        LevelNode::Zero => payload.push(0x00),
        LevelNode::Succ(inner) => {
            payload.push(0x01);
            payload.extend(child_hashes[*inner]);
        }
        LevelNode::Max(lhs, rhs) => {
            payload.push(0x02);
            payload.extend(child_hashes[*lhs]);
            payload.extend(child_hashes[*rhs]);
        }
        LevelNode::IMax(lhs, rhs) => {
            payload.push(0x03);
            payload.extend(child_hashes[*lhs]);
            payload.extend(child_hashes[*rhs]);
        }
        LevelNode::Param(name) => {
            payload.push(0x04);
            encode_name_to(&mut payload, &names[*name].value);
        }
    }
    Ok(payload)
}

fn term_node_key(
    term: &TermNode,
    child_hashes: &[ReferenceHash],
    level_hashes: &[ReferenceHash],
) -> DecodeResult<Vec<u8>> {
    let mut payload = Vec::new();
    match term {
        TermNode::Sort(level) => {
            payload.push(0x00);
            payload.extend(level_hashes[*level]);
        }
        TermNode::BVar(index) => {
            payload.push(0x01);
            encode_uvar_to(&mut payload, u64::from(*index));
        }
        TermNode::Const { global_ref, levels } => {
            payload.push(0x02);
            encode_global_ref_to(&mut payload, global_ref);
            encode_uvar_to(&mut payload, levels.len() as u64);
            for level in levels {
                payload.extend(level_hashes[*level]);
            }
        }
        TermNode::App(fun, arg) => {
            payload.push(0x03);
            payload.extend(child_hashes[*fun]);
            payload.extend(child_hashes[*arg]);
        }
        TermNode::Lam { ty, body } => {
            payload.push(0x04);
            payload.extend(child_hashes[*ty]);
            payload.extend(child_hashes[*body]);
        }
        TermNode::Pi { ty, body } => {
            payload.push(0x05);
            payload.extend(child_hashes[*ty]);
            payload.extend(child_hashes[*body]);
        }
        TermNode::Let { ty, value, body } => {
            payload.push(0x06);
            payload.extend(child_hashes[*ty]);
            payload.extend(child_hashes[*value]);
            payload.extend(child_hashes[*body]);
        }
    }
    Ok(payload)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ComputedDeclHashes {
    decl_interface_hash: ReferenceHash,
    decl_certificate_hash: ReferenceHash,
}

fn compute_decl_hashes(
    decl: &DeclPayload,
    dependencies: &[DependencyEntry],
    axiom_dependencies: &[AxiomRef],
    term_table: &[Located<TermNode>],
    level_hashes: &[ReferenceHash],
    term_hashes: &[ReferenceHash],
    names: &[Located<ReferenceModuleName>],
) -> DecodeResult<ComputedDeclHashes> {
    let interface_dependencies = interface_dependencies_for_decl(decl, dependencies, term_table)?;
    let interface_hash = hash_with_domain(
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
    let certificate_hash = hash_with_domain(
        b"NPA-DECL-CERT-0.1",
        &decl_certificate_payload(
            decl,
            interface_hash,
            dependencies,
            axiom_dependencies,
            term_hashes,
        )?,
    );
    Ok(ComputedDeclHashes {
        decl_interface_hash: interface_hash,
        decl_certificate_hash: certificate_hash,
    })
}

fn decl_interface_payload(
    decl: &DeclPayload,
    interface_dependencies: &[DependencyEntry],
    axiom_dependencies: &[AxiomRef],
    level_hashes: &[ReferenceHash],
    term_hashes: &[ReferenceHash],
    names: &[Located<ReferenceModuleName>],
) -> DecodeResult<Vec<u8>> {
    let mut out = Vec::new();
    match decl {
        DeclPayload::Axiom {
            name,
            universe_params,
            ty,
        } => {
            out.push(0x00);
            encode_name_id_to(&mut out, names, *name);
            encode_name_ids_to(&mut out, names, universe_params);
            out.extend(term_hashes[*ty]);
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
            encode_name_id_to(&mut out, names, *name);
            encode_name_ids_to(&mut out, names, universe_params);
            out.extend(term_hashes[*ty]);
            encode_reducibility_to(&mut out, *reducibility);
            encode_dependency_entries_to(&mut out, interface_dependencies);
            encode_axiom_refs_to(&mut out, axiom_dependencies);
            if *reducibility == CertReducibility::Reducible {
                out.extend(term_hashes[*value]);
            }
        }
        DeclPayload::Theorem {
            name,
            universe_params,
            ty,
            opacity,
            ..
        } => {
            out.push(0x02);
            encode_name_id_to(&mut out, names, *name);
            encode_name_ids_to(&mut out, names, universe_params);
            out.extend(term_hashes[*ty]);
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
            encode_name_id_to(&mut out, names, *name);
            encode_name_ids_to(&mut out, names, universe_params);
            encode_uvar_to(&mut out, params.len() as u64);
            for param in params {
                out.extend(term_hashes[param.ty]);
            }
            encode_uvar_to(&mut out, indices.len() as u64);
            for index in indices {
                out.extend(term_hashes[index.ty]);
            }
            out.extend(level_hashes[*sort]);
            encode_constructor_specs_to(&mut out, constructors, term_hashes, names);
            out.extend(generated_recursor_signature_hash(
                recursor.as_ref(),
                term_hashes,
                names,
            ));
            out.extend(generated_computation_rule_hash(recursor.as_ref()));
            encode_dependency_entries_to(&mut out, interface_dependencies);
            encode_axiom_refs_to(&mut out, axiom_dependencies);
        }
    }
    Ok(out)
}

fn decl_certificate_payload(
    decl: &DeclPayload,
    interface_hash: ReferenceHash,
    dependencies: &[DependencyEntry],
    axiom_dependencies: &[AxiomRef],
    term_hashes: &[ReferenceHash],
) -> DecodeResult<Vec<u8>> {
    let mut out = Vec::new();
    out.extend(interface_hash);
    match decl {
        DeclPayload::Axiom { .. } => encode_axiom_refs_to(&mut out, axiom_dependencies),
        DeclPayload::Def { value, .. } => {
            out.extend(term_hashes[*value]);
            encode_dependency_entries_to(&mut out, dependencies);
            encode_axiom_refs_to(&mut out, axiom_dependencies);
        }
        DeclPayload::Inductive { .. } => {
            encode_dependency_entries_to(&mut out, dependencies);
            encode_axiom_refs_to(&mut out, axiom_dependencies);
        }
        DeclPayload::Theorem { proof, .. } => {
            out.extend(term_hashes[*proof]);
            encode_dependency_entries_to(&mut out, dependencies);
        }
    }
    Ok(out)
}

fn interface_dependencies_for_decl(
    decl: &DeclPayload,
    dependencies: &[DependencyEntry],
    term_table: &[Located<TermNode>],
) -> DecodeResult<Vec<DependencyEntry>> {
    let mut refs = BTreeSet::new();
    for term in interface_term_ids(decl) {
        collect_global_refs_from_term(term_table, term, &mut refs)?;
    }
    Ok(dependencies
        .iter()
        .filter(|dependency| refs.contains(&dependency.global_ref))
        .cloned()
        .collect())
}

fn interface_term_ids(decl: &DeclPayload) -> Vec<usize> {
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
    terms: &[Located<TermNode>],
    term: usize,
    refs: &mut BTreeSet<GlobalRef>,
) -> DecodeResult<()> {
    match &terms[term].value {
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

fn inductive_export_type_term_id(
    term_table: &[Located<TermNode>],
    params: &[BinderType],
    indices: &[BinderType],
    sort: usize,
) -> DecodeResult<usize> {
    let mut body = term_table
        .iter()
        .position(|term| matches!(term.value, TermNode::Sort(level) if level == sort))
        .ok_or_else(|| {
            ReferenceCheckError::malformed(
                ReferenceCertificateSection::TermTable,
                term_table.last().map_or(0, |entry| entry.offset),
                ReferenceCheckReason::DanglingReference,
            )
        })?;
    for binder in params.iter().chain(indices).rev() {
        body = term_table
            .iter()
            .position(|term| {
                matches!(
                    term.value,
                    TermNode::Pi { ty, body: pi_body } if ty == binder.ty && pi_body == body
                )
            })
            .ok_or_else(|| {
                ReferenceCheckError::malformed(
                    ReferenceCertificateSection::TermTable,
                    term_table.last().map_or(0, |entry| entry.offset),
                    ReferenceCheckReason::DanglingReference,
                )
            })?;
    }
    Ok(body)
}

fn encode_constructor_specs_to(
    out: &mut Vec<u8>,
    constructors: &[ConstructorSpec],
    term_hashes: &[ReferenceHash],
    names: &[Located<ReferenceModuleName>],
) {
    encode_uvar_to(out, constructors.len() as u64);
    for constructor in constructors {
        encode_name_id_to(out, names, constructor.name);
        out.extend(term_hashes[constructor.ty]);
    }
}

fn generated_recursor_signature_hash(
    recursor: Option<&RecursorSpec>,
    term_hashes: &[ReferenceHash],
    names: &[Located<ReferenceModuleName>],
) -> ReferenceHash {
    hash_with_domain(
        b"NPA-GEN-REC-SIG-0.1",
        &generated_recursor_signature_payload(recursor, term_hashes, names),
    )
}

fn generated_recursor_signature_payload(
    recursor: Option<&RecursorSpec>,
    term_hashes: &[ReferenceHash],
    names: &[Located<ReferenceModuleName>],
) -> Vec<u8> {
    let mut out = Vec::new();
    match recursor {
        Some(recursor) => {
            out.push(0x01);
            encode_name_id_to(&mut out, names, recursor.name);
            encode_name_ids_to(&mut out, names, &recursor.universe_params);
            out.extend(term_hashes[recursor.ty]);
        }
        None => out.push(0x00),
    }
    out
}

fn generated_computation_rule_hash(recursor: Option<&RecursorSpec>) -> ReferenceHash {
    hash_with_domain(
        b"NPA-GEN-COMP-RULE-0.1",
        &generated_computation_rule_payload(recursor),
    )
}

fn generated_computation_rule_payload(recursor: Option<&RecursorSpec>) -> Vec<u8> {
    let mut out = Vec::new();
    match recursor {
        Some(recursor) => {
            out.push(0x01);
            encode_recursor_rules_to(&mut out, &recursor.rules);
        }
        None => out.push(0x00),
    }
    out
}

fn encode_recursor_rules_to(out: &mut Vec<u8>, rules: &RecursorRulesSpec) {
    encode_uvar_to(out, rules.minor_start as u64);
    encode_uvar_to(out, rules.major_index as u64);
}

fn encode_export_block(block: &[ExportEntry]) -> Vec<u8> {
    let mut out = Vec::new();
    encode_uvar_to(&mut out, block.len() as u64);
    for entry in block {
        encode_uvar_to(&mut out, entry.name as u64);
        out.push(match entry.kind {
            ExportKind::Axiom => 0x00,
            ExportKind::Def => 0x01,
            ExportKind::Theorem => 0x02,
            ExportKind::Inductive => 0x03,
            ExportKind::Constructor => 0x04,
            ExportKind::Recursor => 0x05,
        });
        encode_usize_vec(&mut out, &entry.universe_params);
        encode_uvar_to(&mut out, entry.ty as u64);
        encode_option_usize_to(&mut out, entry.body);
        out.extend(entry.type_hash);
        encode_option_hash_to(&mut out, entry.body_hash.as_ref());
        encode_option_reducibility_to(&mut out, entry.reducibility);
        encode_option_opacity_to(&mut out, entry.opacity);
        out.extend(entry.decl_interface_hash);
        encode_axiom_refs_to(&mut out, &entry.axiom_dependencies);
    }
    out
}

fn encode_axiom_report(report: &AxiomReport) -> Vec<u8> {
    let mut out = Vec::new();
    encode_uvar_to(&mut out, report.per_declaration.len() as u64);
    for entry in &report.per_declaration {
        encode_uvar_to(&mut out, entry.decl_index as u64);
        encode_axiom_refs_to(&mut out, &entry.direct_axioms);
        encode_axiom_refs_to(&mut out, &entry.transitive_axioms);
    }
    encode_axiom_refs_to(&mut out, &report.module_axioms);
    out
}

fn encode_dependency_entries_to(out: &mut Vec<u8>, entries: &[DependencyEntry]) {
    encode_uvar_to(out, entries.len() as u64);
    for entry in entries {
        encode_global_ref_to(out, &entry.global_ref);
        out.extend(entry.decl_interface_hash);
    }
}

fn encode_axiom_refs_to(out: &mut Vec<u8>, axioms: &[AxiomRef]) {
    encode_uvar_to(out, axioms.len() as u64);
    for axiom in axioms {
        encode_global_ref_to(out, &axiom.global_ref);
        encode_uvar_to(out, axiom.name as u64);
        out.extend(axiom.decl_interface_hash);
    }
}

fn encode_name_id_to(out: &mut Vec<u8>, names: &[Located<ReferenceModuleName>], name: usize) {
    encode_name_to(out, &names[name].value);
}

fn encode_name_ids_to(out: &mut Vec<u8>, names: &[Located<ReferenceModuleName>], values: &[usize]) {
    encode_uvar_to(out, values.len() as u64);
    for value in values {
        encode_name_id_to(out, names, *value);
    }
}

fn encode_usize_vec(out: &mut Vec<u8>, values: &[usize]) {
    encode_uvar_to(out, values.len() as u64);
    for value in values {
        encode_uvar_to(out, *value as u64);
    }
}

fn encode_reducibility_to(out: &mut Vec<u8>, value: CertReducibility) {
    out.push(match value {
        CertReducibility::Reducible => 0x00,
        CertReducibility::Opaque => 0x01,
    });
}

fn encode_option_reducibility_to(out: &mut Vec<u8>, value: Option<CertReducibility>) {
    match value {
        Some(value) => {
            out.push(0x01);
            encode_reducibility_to(out, value);
        }
        None => out.push(0x00),
    }
}

fn encode_opacity_to(out: &mut Vec<u8>, value: Opacity) {
    match value {
        Opacity::Opaque => out.push(0x00),
    }
}

fn encode_option_opacity_to(out: &mut Vec<u8>, value: Option<Opacity>) {
    match value {
        Some(value) => {
            out.push(0x01);
            encode_opacity_to(out, value);
        }
        None => out.push(0x00),
    }
}

fn encode_option_usize_to(out: &mut Vec<u8>, value: Option<usize>) {
    match value {
        Some(value) => {
            out.push(0x01);
            encode_uvar_to(out, value as u64);
        }
        None => out.push(0x00),
    }
}

fn encode_option_hash_to(out: &mut Vec<u8>, hash: Option<&ReferenceHash>) {
    match hash {
        Some(hash) => {
            out.push(0x01);
            out.extend(hash);
        }
        None => out.push(0x00),
    }
}

fn encode_global_ref_to(out: &mut Vec<u8>, global_ref: &GlobalRef) {
    match global_ref {
        GlobalRef::Builtin {
            name,
            decl_interface_hash,
        } => {
            out.push(0x03);
            encode_uvar_to(out, *name as u64);
            out.extend(decl_interface_hash);
        }
        GlobalRef::Imported {
            import_index,
            name,
            decl_interface_hash,
        } => {
            out.push(0x00);
            encode_uvar_to(out, *import_index as u64);
            encode_uvar_to(out, *name as u64);
            out.extend(decl_interface_hash);
        }
        GlobalRef::Local { decl_index } => {
            out.push(0x01);
            encode_uvar_to(out, *decl_index as u64);
        }
        GlobalRef::LocalGenerated { decl_index, name } => {
            out.push(0x02);
            encode_uvar_to(out, *decl_index as u64);
            encode_uvar_to(out, *name as u64);
        }
    }
}

fn encode_name_to(out: &mut Vec<u8>, name: &ReferenceModuleName) {
    encode_uvar_to(out, name.components().len() as u64);
    for component in name.components() {
        encode_uvar_to(out, component.len() as u64);
        out.extend(component.as_bytes());
    }
}

fn hash_with_domain(domain: &[u8], payload: &[u8]) -> ReferenceHash {
    let mut hasher = Sha256::new();
    hasher.update(domain);
    hasher.update(payload);
    hasher.finalize().into()
}

fn encode_uvar(value: u64) -> Vec<u8> {
    let mut out = Vec::new();
    encode_uvar_to(&mut out, value);
    out
}

fn encode_uvar_to(out: &mut Vec<u8>, mut value: u64) {
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
