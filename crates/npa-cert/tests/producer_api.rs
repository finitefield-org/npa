use npa_cert::{
    build_module_cert, canonical_import_env_keys, canonical_import_export_views,
    encode_module_cert, initial_env_fingerprint, post_env_fingerprint,
    precheck_core_decl_candidate, prior_chain_fingerprint, prior_chain_fingerprint_canonical_bytes,
    producer_checked_decl_interface, producer_env_fingerprint,
    producer_env_fingerprint_canonical_bytes, producer_import_env_key,
    producer_limits_canonical_bytes, producer_limits_hash, producer_lookup_env, stricter_or_equal,
    validate_candidate_batch_imports, verify_module_cert, AxiomPolicy, AxiomRef, CandidateBatch,
    CandidateBatchResult, CandidateHashPreview, CandidateStatus, CertError, CheckedDeclCandidate,
    CoreDeclCandidate, CoreModule, GlobalRef, Name, ProducerCheckedDeclInterface,
    ProducerEnvFingerprintBytes, ProducerImportEnvKey, ProducerLimitKind, ProducerLimits,
    ProducerPriorChainBytes, ProducerPriorChainEntry, ProducerProfile, VerifiedModule,
    VerifierSession,
};
use npa_kernel::{Decl, Env, Error as KernelError, Expr, Level, Reducibility, ResourceLimitKind};

fn trivial_axiom(name: &str) -> Decl {
    Decl::Axiom {
        name: name.to_owned(),
        universe_params: vec![],
        ty: Expr::sort(Level::zero()),
    }
}

fn generous_limits() -> ProducerLimits {
    ProducerLimits {
        max_declarations: 1,
        max_expr_nodes: 64,
        max_level_nodes: 64,
        max_name_components: 8,
        max_reduction_steps: 64,
        max_conversion_steps: 64,
    }
}

fn hash(byte: u8) -> [u8; 32] {
    [byte; 32]
}

fn verify_module(module: CoreModule) -> VerifiedModule {
    let cert = build_module_cert(module, &[]).unwrap();
    let bytes = encode_module_cert(&cert).unwrap();
    let mut session = VerifierSession::new();
    verify_module_cert(&bytes, &mut session, &AxiomPolicy::normal()).unwrap()
}

fn producer_env(
    direct_imports: Vec<ProducerImportEnvKey>,
    checked_decls: Vec<ProducerCheckedDeclInterface>,
) -> ProducerEnvFingerprintBytes {
    ProducerEnvFingerprintBytes {
        direct_imports,
        checked_decls,
    }
}

fn checked_decl_interface(byte: u8) -> ProducerCheckedDeclInterface {
    ProducerCheckedDeclInterface {
        decl_interface_hash: hash(byte),
        axiom_dependencies: vec![],
    }
}

fn local_axiom_ref(decl_index: usize, name: usize, byte: u8) -> AxiomRef {
    AxiomRef {
        global_ref: GlobalRef::Local { decl_index },
        name,
        decl_interface_hash: hash(byte),
    }
}

fn empty_batch(imports: &[VerifiedModule]) -> CandidateBatch<'_> {
    CandidateBatch {
        imports,
        prior_current_decls: &[],
        candidates: vec![],
        limits: generous_limits(),
    }
}

fn axiom_module(module_name: &str, decl_name: &str) -> CoreModule {
    CoreModule {
        name: Name::from_dotted(module_name),
        declarations: vec![trivial_axiom(decl_name)],
    }
}

fn opaque_def_module(module_name: &str, value: Expr) -> CoreModule {
    CoreModule {
        name: Name::from_dotted(module_name),
        declarations: vec![Decl::Def {
            name: "carrier".to_owned(),
            universe_params: vec![],
            ty: Expr::sort(Level::succ(Level::zero())),
            value,
            reducibility: Reducibility::Opaque,
        }],
    }
}

fn id_type(a: &str, x: &str) -> Expr {
    Expr::pi(
        a,
        Expr::sort(Level::param("u")),
        Expr::pi(x, Expr::bvar(0), Expr::bvar(1)),
    )
}

fn id_value(a: &str, x: &str) -> Expr {
    Expr::lam(
        a,
        Expr::sort(Level::param("u")),
        Expr::lam(x, Expr::bvar(0), Expr::bvar(0)),
    )
}

fn id_value_with_beta_redex() -> Expr {
    Expr::lam(
        "A",
        Expr::sort(Level::param("u")),
        Expr::lam(
            "x",
            Expr::bvar(0),
            Expr::app(Expr::lam("y", Expr::bvar(1), Expr::bvar(0)), Expr::bvar(0)),
        ),
    )
}

fn id_theorem_module(module_name: &str, proof: Expr) -> CoreModule {
    CoreModule {
        name: Name::from_dotted(module_name),
        declarations: vec![Decl::Theorem {
            name: "id_thm".to_owned(),
            universe_params: vec!["u".to_owned()],
            ty: id_type("A", "x"),
            proof,
        }],
    }
}

fn imported_theorem(name: &str, imported_name: &str) -> Decl {
    Decl::Theorem {
        name: name.to_owned(),
        universe_params: vec![],
        ty: Expr::konst(imported_name, vec![]),
        proof: Expr::konst(imported_name, vec![]),
    }
}

fn prior_entry(
    decl_interface_hash: [u8; 32],
    decl_certificate_hash: [u8; 32],
    pre_env_fingerprint: [u8; 32],
    post_env_fingerprint: [u8; 32],
) -> ProducerPriorChainEntry {
    ProducerPriorChainEntry {
        decl_interface_hash,
        decl_certificate_hash,
        pre_env_fingerprint,
        post_env_fingerprint,
    }
}

#[test]
fn producer_types_are_available_from_public_api() {
    let limits = ProducerLimits {
        max_declarations: 1,
        max_expr_nodes: 8,
        max_level_nodes: 2,
        max_name_components: 4,
        max_reduction_steps: 16,
        max_conversion_steps: 16,
    };
    let candidate = CoreDeclCandidate {
        declaration: trivial_axiom("P"),
    };
    let imports: &[VerifiedModule] = &[];
    let prior: &[CheckedDeclCandidate] = &[];

    let batch = CandidateBatch {
        imports,
        prior_current_decls: prior,
        candidates: vec![candidate],
        limits,
    };

    assert_eq!(batch.imports.len(), 0);
    assert_eq!(batch.prior_current_decls.len(), 0);
    assert_eq!(batch.candidates.len(), 1);
    assert_eq!(batch.limits, limits);

    let preview = CandidateHashPreview {
        type_hash: None,
        body_hash: None,
        decl_interface_hash: None,
        decl_certificate_hash: None,
    };
    assert_eq!(preview.type_hash, None);

    let result = CandidateBatchResult {
        statuses: vec![CandidateStatus::Rejected(CertError::DecodeError)],
    };
    assert!(matches!(
        result.statuses.as_slice(),
        [CandidateStatus::Rejected(CertError::DecodeError)]
    ));
}

#[test]
fn validate_candidate_batch_imports_preserves_canonical_import_index_order() {
    let import_a = verify_module(axiom_module("Lib.A", "A"));
    let import_b = verify_module(axiom_module("Lib.B", "B"));

    let imports = [import_a.clone(), import_b.clone()];
    let keys = validate_candidate_batch_imports(&empty_batch(&imports)).unwrap();
    assert_eq!(
        keys,
        vec![
            ProducerImportEnvKey {
                module: import_a.module().clone(),
                export_hash: import_a.export_hash(),
            },
            ProducerImportEnvKey {
                module: import_b.module().clone(),
                export_hash: import_b.export_hash(),
            },
        ]
    );

    let reversed_imports = [import_b, import_a];
    assert_eq!(
        validate_candidate_batch_imports(&empty_batch(&reversed_imports)).unwrap_err(),
        CertError::NonCanonicalEncoding { object: "Imports" }
    );
}

#[test]
fn validate_candidate_batch_imports_rejects_duplicate_env_key_before_certificate_hash() {
    let import_a = verify_module(opaque_def_module(
        "Lib.SameExport",
        Expr::sort(Level::zero()),
    ));
    let import_b = verify_module(opaque_def_module(
        "Lib.SameExport",
        Expr::pi("x", Expr::sort(Level::zero()), Expr::sort(Level::zero())),
    ));

    assert_eq!(import_a.export_hash(), import_b.export_hash());
    assert_ne!(import_a.certificate_hash(), import_b.certificate_hash());

    let imports = [import_a.clone(), import_b];
    assert_eq!(
        validate_candidate_batch_imports(&empty_batch(&imports)).unwrap_err(),
        CertError::DuplicateImportEnvKey {
            module: import_a.module().clone(),
            export_hash: import_a.export_hash(),
        }
    );
}

#[test]
fn producer_env_fingerprint_canonical_bytes_fix_record_order() {
    let env = producer_env(
        vec![ProducerImportEnvKey {
            module: Name::from_dotted("Lib.A"),
            export_hash: hash(0x11),
        }],
        vec![ProducerCheckedDeclInterface {
            decl_interface_hash: hash(0x22),
            axiom_dependencies: vec![],
        }],
    );

    let mut expected = vec![0x01, 0x02, 0x03, b'L', b'i', b'b', 0x01, b'A'];
    expected.extend(hash(0x11));
    expected.push(0x01);
    expected.extend(hash(0x22));
    expected.push(0x00);

    assert_eq!(producer_env_fingerprint_canonical_bytes(&env), expected);
    assert_eq!(
        producer_env_fingerprint(&env),
        [
            0x1c, 0xa5, 0xe5, 0xaa, 0xa4, 0x39, 0xec, 0x3e, 0x99, 0xc2, 0xc5, 0xe9, 0xff, 0x9d,
            0xaa, 0xda, 0xfd, 0x73, 0xff, 0x9f, 0x43, 0x57, 0xf5, 0x4c, 0x83, 0xda, 0xf6, 0x74,
            0x9e, 0x4a, 0xc4, 0x15,
        ]
    );
}

#[test]
fn producer_env_fingerprint_ignores_import_certificate_hash() {
    let import_a = verify_module(opaque_def_module(
        "Lib.SameExportForEnv",
        Expr::sort(Level::zero()),
    ));
    let import_b = verify_module(opaque_def_module(
        "Lib.SameExportForEnv",
        Expr::pi("x", Expr::sort(Level::zero()), Expr::sort(Level::zero())),
    ));

    assert_eq!(import_a.export_hash(), import_b.export_hash());
    assert_ne!(import_a.certificate_hash(), import_b.certificate_hash());

    let env_a = producer_env(vec![producer_import_env_key(&import_a)], vec![]);
    let env_b = producer_env(vec![producer_import_env_key(&import_b)], vec![]);

    assert_eq!(
        producer_env_fingerprint(&env_a),
        producer_env_fingerprint(&env_b)
    );
}

#[test]
fn producer_env_fingerprint_preserves_checked_decl_order() {
    let first = checked_decl_interface(0x31);
    let second = checked_decl_interface(0x32);

    let env_ab = producer_env(vec![], vec![first.clone(), second.clone()]);
    let env_ba = producer_env(vec![], vec![second, first]);

    assert_ne!(
        producer_env_fingerprint(&env_ab),
        producer_env_fingerprint(&env_ba)
    );
}

#[test]
fn producer_env_fingerprint_sorts_axiom_dependencies_canonically() {
    let axiom_a = local_axiom_ref(0, 1, 0x41);
    let axiom_b = local_axiom_ref(1, 0, 0x42);

    let env_ab = producer_env(
        vec![],
        vec![ProducerCheckedDeclInterface {
            decl_interface_hash: hash(0x51),
            axiom_dependencies: vec![axiom_a.clone(), axiom_b.clone()],
        }],
    );
    let env_ba = producer_env(
        vec![],
        vec![ProducerCheckedDeclInterface {
            decl_interface_hash: hash(0x51),
            axiom_dependencies: vec![axiom_b, axiom_a],
        }],
    );

    assert_eq!(
        producer_env_fingerprint_canonical_bytes(&env_ab),
        producer_env_fingerprint_canonical_bytes(&env_ba)
    );
    assert_eq!(
        producer_env_fingerprint(&env_ab),
        producer_env_fingerprint(&env_ba)
    );
}

#[test]
fn canonical_import_keys_and_export_views_preserve_same_indices() {
    let import_a = verify_module(axiom_module("Lib.IndexA", "IndexA"));
    let import_b = verify_module(axiom_module("Lib.IndexB", "IndexB"));
    let imports = [import_a, import_b];

    let keys = canonical_import_env_keys(&imports).unwrap();
    let views = canonical_import_export_views(&imports).unwrap();

    assert_eq!(keys.len(), views.len());
    for (key, view) in keys.iter().zip(&views) {
        assert_eq!(key.module, view.module);
        assert_eq!(key.export_hash, view.export_hash);
    }
}

#[test]
fn producer_checked_decl_interface_uses_import_export_view_indices() {
    let import_a = verify_module(axiom_module("Lib.LookupA", "LookupA"));
    let import_b = verify_module(axiom_module("Lib.LookupB", "LookupB"));
    let imports = [import_a, import_b];
    let lookup = producer_lookup_env(&imports, &[]).unwrap();

    let interface =
        producer_checked_decl_interface(&imported_theorem("UsesLookupB", "LookupB"), &lookup)
            .unwrap();

    assert_eq!(interface.axiom_dependencies.len(), 1);
    assert!(matches!(
        interface.axiom_dependencies[0].global_ref,
        GlobalRef::Imported {
            import_index: 1,
            ..
        }
    ));
}

#[test]
fn producer_checked_decl_interface_recomputes_axioms_from_export_view_not_key() {
    let import = verify_module(axiom_module("Lib.LookupSource", "LookupSource"));
    let imports = [import];
    let lookup = producer_lookup_env(&imports, &[]).unwrap();
    let decl = imported_theorem("UsesLookupSource", "LookupSource");

    let from_verified_view = producer_checked_decl_interface(&decl, &lookup).unwrap();

    let mut tampered_lookup = lookup.clone();
    tampered_lookup.import_exports[0].exports[0]
        .axiom_dependencies
        .clear();
    let from_tampered_view = producer_checked_decl_interface(&decl, &tampered_lookup).unwrap();

    assert_eq!(from_verified_view.axiom_dependencies.len(), 1);
    assert!(from_tampered_view.axiom_dependencies.is_empty());
    assert_ne!(
        from_verified_view.decl_interface_hash,
        from_tampered_view.decl_interface_hash
    );
}

#[test]
fn producer_checked_decl_interface_matches_certificate_generation_for_imported_axiom() {
    let import = verify_module(axiom_module("Lib.MatchP", "MatchP"));
    let imports = [import.clone()];
    let decl = Decl::Axiom {
        name: "MatchQ".to_owned(),
        universe_params: vec![],
        ty: Expr::konst("MatchP", vec![]),
    };
    let cert = build_module_cert(
        CoreModule {
            name: Name::from_dotted("MatchQ"),
            declarations: vec![decl.clone()],
        },
        &imports,
    )
    .unwrap();
    let lookup = producer_lookup_env(&imports, &[]).unwrap();

    let interface = producer_checked_decl_interface(&decl, &lookup).unwrap();

    assert_eq!(
        interface.decl_interface_hash,
        cert.declarations[0].hashes.decl_interface_hash
    );
    assert_eq!(
        interface.axiom_dependencies,
        cert.declarations[0].axiom_dependencies
    );
}

#[test]
fn initial_env_fingerprint_matches_explicit_full_recompute() {
    let import_a = verify_module(axiom_module("Lib.InitialA", "InitialA"));
    let import_b = verify_module(axiom_module("Lib.InitialB", "InitialB"));
    let imports = [import_a, import_b];

    let expected = producer_env_fingerprint(&ProducerEnvFingerprintBytes {
        direct_imports: canonical_import_env_keys(&imports).unwrap(),
        checked_decls: vec![],
    });

    assert_eq!(initial_env_fingerprint(&imports).unwrap(), expected);
}

#[test]
fn post_env_fingerprint_matches_explicit_full_recompute() {
    let import = verify_module(axiom_module("Lib.PostSource", "PostSource"));
    let imports = [import];
    let prior = vec![checked_decl_interface(0x61)];
    let decl = imported_theorem("UsesPostSource", "PostSource");
    let lookup = producer_lookup_env(&imports, &prior).unwrap();
    let mut expected_checked = prior.clone();
    expected_checked.push(producer_checked_decl_interface(&decl, &lookup).unwrap());
    let expected = producer_env_fingerprint(&ProducerEnvFingerprintBytes {
        direct_imports: canonical_import_env_keys(&imports).unwrap(),
        checked_decls: expected_checked,
    });

    assert_eq!(
        post_env_fingerprint(&imports, &prior, &decl).unwrap(),
        expected
    );
}

#[test]
fn post_env_fingerprint_is_deterministic_for_same_inputs() {
    let import = verify_module(axiom_module("Lib.PostDeterministic", "PostDeterministic"));
    let imports = [import];
    let prior = vec![ProducerCheckedDeclInterface {
        decl_interface_hash: hash(0x62),
        axiom_dependencies: vec![local_axiom_ref(0, 0, 0x63)],
    }];
    let decl = imported_theorem("UsesPostDeterministic", "PostDeterministic");

    assert_eq!(
        post_env_fingerprint(&imports, &prior, &decl).unwrap(),
        post_env_fingerprint(&imports, &prior, &decl).unwrap()
    );
}

#[test]
fn post_env_fingerprint_changes_when_checked_decl_sequence_changes() {
    let import = verify_module(axiom_module("Lib.PostSequence", "PostSequence"));
    let imports = [import];
    let decl = imported_theorem("UsesPostSequence", "PostSequence");
    let prior_a = vec![checked_decl_interface(0x71)];
    let prior_b = vec![checked_decl_interface(0x72)];

    assert_ne!(
        post_env_fingerprint(&imports, &prior_a, &decl).unwrap(),
        post_env_fingerprint(&imports, &prior_b, &decl).unwrap()
    );
}

#[test]
fn post_env_fingerprint_uses_import_public_environment_not_certificate_identity() {
    let import_a = verify_module(opaque_def_module(
        "Lib.SamePostExport",
        Expr::sort(Level::zero()),
    ));
    let import_b = verify_module(opaque_def_module(
        "Lib.SamePostExport",
        Expr::pi("x", Expr::sort(Level::zero()), Expr::sort(Level::zero())),
    ));
    let decl = Decl::Axiom {
        name: "UsesSamePostExport".to_owned(),
        universe_params: vec![],
        ty: Expr::konst("carrier", vec![]),
    };

    assert_eq!(import_a.export_hash(), import_b.export_hash());
    assert_ne!(import_a.certificate_hash(), import_b.certificate_hash());
    assert_eq!(
        post_env_fingerprint(&[import_a], &[], &decl).unwrap(),
        post_env_fingerprint(&[import_b], &[], &decl).unwrap()
    );
}

#[test]
fn prior_chain_fingerprint_canonical_bytes_fix_record_order() {
    let chain = ProducerPriorChainBytes {
        checked_decls: vec![prior_entry(hash(0x11), hash(0x22), hash(0x33), hash(0x44))],
    };
    let mut expected = vec![0x01];
    expected.extend(hash(0x11));
    expected.extend(hash(0x22));
    expected.extend(hash(0x33));
    expected.extend(hash(0x44));

    assert_eq!(prior_chain_fingerprint_canonical_bytes(&chain), expected);
}

#[test]
fn prior_chain_fingerprint_empty_chain_is_deterministic() {
    let chain = ProducerPriorChainBytes {
        checked_decls: vec![],
    };

    assert_eq!(prior_chain_fingerprint_canonical_bytes(&chain), vec![0x00]);
    assert_eq!(
        prior_chain_fingerprint(&chain),
        [
            0x81, 0x78, 0xcf, 0xcd, 0xe5, 0xe9, 0x89, 0x13, 0x8f, 0x61, 0x8d, 0x11, 0x02, 0x0f,
            0xef, 0xd3, 0x68, 0xde, 0x61, 0x5a, 0x69, 0xb2, 0x3e, 0x45, 0x06, 0x0e, 0xac, 0xa9,
            0xb8, 0xeb, 0x8b, 0xa4,
        ]
    );
}

#[test]
fn prior_chain_fingerprint_preserves_declaration_order() {
    let first = prior_entry(hash(0x11), hash(0x12), hash(0x13), hash(0x14));
    let second = prior_entry(hash(0x21), hash(0x22), hash(0x23), hash(0x24));
    let chain_ab = ProducerPriorChainBytes {
        checked_decls: vec![first.clone(), second.clone()],
    };
    let chain_ba = ProducerPriorChainBytes {
        checked_decls: vec![second, first],
    };

    assert_ne!(
        prior_chain_fingerprint(&chain_ab),
        prior_chain_fingerprint(&chain_ba)
    );
}

#[test]
fn prior_chain_fingerprint_changes_for_body_only_certificate_hash_change() {
    let cert_a = build_module_cert(
        id_theorem_module("Test.PriorBodyOnly", id_value("A", "x")),
        &[],
    )
    .unwrap();
    let cert_b = build_module_cert(
        id_theorem_module("Test.PriorBodyOnly", id_value_with_beta_redex()),
        &[],
    )
    .unwrap();
    let decl_a = &cert_a.declarations[0];
    let decl_b = &cert_b.declarations[0];

    assert_eq!(
        decl_a.hashes.decl_interface_hash,
        decl_b.hashes.decl_interface_hash
    );
    assert_ne!(
        decl_a.hashes.decl_certificate_hash,
        decl_b.hashes.decl_certificate_hash
    );

    let pre_env = initial_env_fingerprint(&[]).unwrap();
    let post_env_a = producer_env_fingerprint(&ProducerEnvFingerprintBytes {
        direct_imports: vec![],
        checked_decls: vec![ProducerCheckedDeclInterface {
            decl_interface_hash: decl_a.hashes.decl_interface_hash,
            axiom_dependencies: decl_a.axiom_dependencies.clone(),
        }],
    });
    let post_env_b = producer_env_fingerprint(&ProducerEnvFingerprintBytes {
        direct_imports: vec![],
        checked_decls: vec![ProducerCheckedDeclInterface {
            decl_interface_hash: decl_b.hashes.decl_interface_hash,
            axiom_dependencies: decl_b.axiom_dependencies.clone(),
        }],
    });

    assert_eq!(post_env_a, post_env_b);
    assert_ne!(
        prior_chain_fingerprint(&ProducerPriorChainBytes {
            checked_decls: vec![prior_entry(
                decl_a.hashes.decl_interface_hash,
                decl_a.hashes.decl_certificate_hash,
                pre_env,
                post_env_a,
            )],
        }),
        prior_chain_fingerprint(&ProducerPriorChainBytes {
            checked_decls: vec![prior_entry(
                decl_b.hashes.decl_interface_hash,
                decl_b.hashes.decl_certificate_hash,
                pre_env,
                post_env_b,
            )],
        })
    );
}

#[test]
fn producer_profile_is_not_part_of_certificate_build_path() {
    let _profile = ProducerProfile::AiCoreMvp;
    let module = CoreModule {
        name: Name::from_dotted("Test.ProducerProfileOutOfBand"),
        declarations: vec![trivial_axiom("P")],
    };

    let cert = build_module_cert(module, &[]).unwrap();
    let bytes = encode_module_cert(&cert).unwrap();
    assert!(!bytes.is_empty());

    let mut session = VerifierSession::new();
    verify_module_cert(&bytes, &mut session, &AxiomPolicy::normal()).unwrap();
}

#[test]
fn producer_limits_canonical_bytes_fix_field_order_and_minimal_uleb128() {
    let limits = ProducerLimits {
        max_declarations: 1,
        max_expr_nodes: 128,
        max_level_nodes: 16_384,
        max_name_components: 4,
        max_reduction_steps: 16,
        max_conversion_steps: 65_536,
    };

    assert_eq!(
        producer_limits_canonical_bytes(&limits),
        vec![0x01, 0x80, 0x01, 0x80, 0x80, 0x01, 0x04, 0x10, 0x80, 0x80, 0x04]
    );
}

#[test]
fn producer_limits_hash_is_deterministic_and_field_order_sensitive() {
    let limits = ProducerLimits {
        max_declarations: 1,
        max_expr_nodes: 2,
        max_level_nodes: 3,
        max_name_components: 4,
        max_reduction_steps: 5,
        max_conversion_steps: 6,
    };
    let swapped_first_two_fields = ProducerLimits {
        max_declarations: 2,
        max_expr_nodes: 1,
        ..limits
    };

    assert_eq!(producer_limits_hash(&limits), producer_limits_hash(&limits));
    assert_eq!(
        producer_limits_hash(&limits),
        [
            0xc9, 0x89, 0x1e, 0x26, 0xbb, 0x98, 0x9d, 0x9d, 0x53, 0x3b, 0x83, 0x1c, 0x08, 0xbb,
            0x86, 0xf6, 0x3b, 0xcd, 0x28, 0x4f, 0x60, 0x7d, 0xb6, 0xe4, 0x1f, 0x27, 0x5d, 0x55,
            0xb8, 0x8e, 0x18, 0x4f,
        ]
    );
    assert_ne!(
        producer_limits_hash(&limits),
        producer_limits_hash(&swapped_first_two_fields)
    );
}

#[test]
fn stricter_or_equal_compares_every_limit_field() {
    let baseline = ProducerLimits {
        max_declarations: 10,
        max_expr_nodes: 20,
        max_level_nodes: 30,
        max_name_components: 40,
        max_reduction_steps: 50,
        max_conversion_steps: 60,
    };
    let stricter = ProducerLimits {
        max_declarations: 9,
        max_expr_nodes: 19,
        max_level_nodes: 29,
        max_name_components: 39,
        max_reduction_steps: 49,
        max_conversion_steps: 59,
    };

    assert!(stricter_or_equal(&baseline, &baseline));
    assert!(stricter_or_equal(&stricter, &baseline));

    let looser_profiles = [
        ProducerLimits {
            max_declarations: baseline.max_declarations + 1,
            ..baseline
        },
        ProducerLimits {
            max_expr_nodes: baseline.max_expr_nodes + 1,
            ..baseline
        },
        ProducerLimits {
            max_level_nodes: baseline.max_level_nodes + 1,
            ..baseline
        },
        ProducerLimits {
            max_name_components: baseline.max_name_components + 1,
            ..baseline
        },
        ProducerLimits {
            max_reduction_steps: baseline.max_reduction_steps + 1,
            ..baseline
        },
        ProducerLimits {
            max_conversion_steps: baseline.max_conversion_steps + 1,
            ..baseline
        },
    ];

    for looser in looser_profiles {
        assert!(!stricter_or_equal(&looser, &baseline));
    }
}

#[test]
fn precheck_core_decl_candidate_accepts_simple_axiom_under_limits() {
    let candidate = CoreDeclCandidate {
        declaration: trivial_axiom("P"),
    };

    precheck_core_decl_candidate(&Env::new(), &candidate, &generous_limits()).unwrap();
}

#[test]
fn precheck_core_decl_candidate_rejects_schema_limit_excess_deterministically() {
    let candidate = CoreDeclCandidate {
        declaration: trivial_axiom("A.B"),
    };

    let mut limits = generous_limits();
    limits.max_declarations = 0;
    assert_eq!(
        precheck_core_decl_candidate(&Env::new(), &candidate, &limits).unwrap_err(),
        CertError::ProducerLimitExceeded {
            limit: ProducerLimitKind::MaxDeclarations
        }
    );

    let mut limits = generous_limits();
    limits.max_expr_nodes = 0;
    assert_eq!(
        precheck_core_decl_candidate(&Env::new(), &candidate, &limits).unwrap_err(),
        CertError::ProducerLimitExceeded {
            limit: ProducerLimitKind::MaxExprNodes
        }
    );

    let mut limits = generous_limits();
    limits.max_level_nodes = 0;
    assert_eq!(
        precheck_core_decl_candidate(&Env::new(), &candidate, &limits).unwrap_err(),
        CertError::ProducerLimitExceeded {
            limit: ProducerLimitKind::MaxLevelNodes
        }
    );

    let mut limits = generous_limits();
    limits.max_name_components = 1;
    assert_eq!(
        precheck_core_decl_candidate(&Env::new(), &candidate, &limits).unwrap_err(),
        CertError::ProducerLimitExceeded {
            limit: ProducerLimitKind::MaxNameComponents
        }
    );
}

#[test]
fn precheck_core_decl_candidate_maps_reduction_and_conversion_limits_to_kernel_fuel() {
    let axiom = CoreDeclCandidate {
        declaration: trivial_axiom("P"),
    };
    let mut limits = generous_limits();
    limits.max_reduction_steps = 0;
    assert_eq!(
        precheck_core_decl_candidate(&Env::new(), &axiom, &limits).unwrap_err(),
        CertError::Kernel(KernelError::ResourceLimit {
            kind: ResourceLimitKind::Whnf
        })
    );

    let definition = CoreDeclCandidate {
        declaration: Decl::Def {
            name: "P".to_owned(),
            universe_params: vec![],
            ty: Expr::sort(Level::succ(Level::zero())),
            value: Expr::sort(Level::zero()),
            reducibility: Reducibility::Reducible,
        },
    };
    let mut limits = generous_limits();
    limits.max_conversion_steps = 0;
    assert_eq!(
        precheck_core_decl_candidate(&Env::new(), &definition, &limits).unwrap_err(),
        CertError::Kernel(KernelError::ResourceLimit {
            kind: ResourceLimitKind::Conversion
        })
    );
}
