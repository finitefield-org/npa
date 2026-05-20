use npa_cert::{
    build_module_cert, encode_module_cert, producer_limits_canonical_bytes, producer_limits_hash,
    stricter_or_equal, verify_module_cert, AxiomPolicy, CandidateBatch, CandidateBatchResult,
    CandidateHashPreview, CandidateStatus, CertError, CheckedDeclCandidate, CoreDeclCandidate,
    CoreModule, Name, ProducerLimits, ProducerProfile, VerifiedModule, VerifierSession,
};
use npa_kernel::{Decl, Expr, Level};

fn trivial_axiom(name: &str) -> Decl {
    Decl::Axiom {
        name: name.to_owned(),
        universe_params: vec![],
        ty: Expr::sort(Level::zero()),
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
