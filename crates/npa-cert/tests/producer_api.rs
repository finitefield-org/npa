use npa_cert::{
    build_module_cert, encode_module_cert, verify_module_cert, AxiomPolicy, CandidateBatch,
    CandidateBatchResult, CandidateHashPreview, CandidateStatus, CertError, CheckedDeclCandidate,
    CoreDeclCandidate, CoreModule, Name, ProducerLimits, ProducerProfile, VerifiedModule,
    VerifierSession,
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
