use npa_cert::{
    build_module_cert, decode_module_cert, encode_module_cert, verify_module_cert, AxiomPolicy,
    CoreModule, DeclPayload, ExportKind, Name, VerifiedModule, VerifierSession,
};
use npa_kernel::{Decl, Level};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

struct ExpectedModule {
    module: &'static str,
    source: &'static str,
    certificate: &'static str,
    meta: &'static str,
    replay: &'static str,
    imports: &'static [&'static str],
    inductives: &'static [&'static str],
    definitions: &'static [&'static str],
    theorems: &'static [&'static str],
    axioms: &'static [&'static str],
}

struct VerifiedCorpusImports<'a> {
    eq: &'a VerifiedModule,
    nat: &'a VerifiedModule,
    ring: &'a VerifiedModule,
    square: &'a VerifiedModule,
    ordered_field: &'a VerifiedModule,
    vector_basic: &'a VerifiedModule,
    vector_dot: &'a VerifiedModule,
    right_triangle: &'a VerifiedModule,
    abstract_ring: &'a VerifiedModule,
    abstract_ordered_field: &'a VerifiedModule,
    abstract_square_normalize: &'a VerifiedModule,
}

const BASIC_THEOREMS: &[&str] = &[
    "id",
    "const_left",
    "const_right",
    "apply_fn",
    "compose",
    "flip",
    "duplicate",
    "prop_id",
    "modus_ponens",
    "imp_trans",
    "compose_assoc",
    "apply_twice",
    "ignore_middle",
    "select_middle",
    "select_last",
    "imp_swap",
    "imp_compose",
    "imp_ignore",
    "imp_duplicate",
    "higher_apply",
];

const EQ_THEOREMS: &[&str] = &[
    "eq_refl_self",
    "eq_refl_fn_app",
    "eq_refl_compose",
    "eq_self_imp",
    "eq_refl_prop",
    "eq_refl_apply_twice",
    "eq_refl_higher_apply",
    "eq_refl_nested_compose",
    "eq_refl_prop_apply",
    "eq_local_passthrough",
    "eq_refl_nat_function",
];

const NAT_THEOREMS: &[&str] = &[
    "nat_zero_self_eq",
    "nat_succ_zero_self_eq",
    "nat_id",
    "nat_const_zero",
    "nat_apply_fn",
    "nat_const_succ_zero",
    "nat_apply_twice",
    "nat_compose",
    "nat_ignore_middle",
    "nat_select_middle",
    "nat_select_last",
    "nat_succ_self_eq",
];

const PROP_THEOREMS: &[&str] = &[
    "imp_chain4",
    "imp_permute3",
    "imp_apply_twice",
    "imp_const3",
    "imp_flip_chain",
    "imp_higher_apply",
];

const REDUCTION_DEFINITIONS: &[&str] = &["reduction_id_nat"];

const REDUCTION_THEOREMS: &[&str] = &[
    "beta_id_nat",
    "beta_const_nat",
    "let_id_nat",
    "let_const_nat",
    "delta_id_nat",
];

const EQ_REASONING_THEOREMS: &[&str] = &[
    "eq_symm",
    "eq_trans",
    "eq_congr_arg",
    "eq_congr_fun",
    "eq_congr2",
    "eq_subst",
    "eq_transport_const",
    "eq_rewrite_left",
    "eq_rewrite_right",
    "eq_cast_trans",
    "eq_calc3",
];

const RING_INDUCTIVES: &[&str] = &["RingElem"];

const RING_DEFINITIONS: &[&str] = &["zero", "one", "add", "neg", "sub", "mul"];

const RING_THEOREMS: &[&str] = &[
    "sub_eq_add_neg",
    "add_assoc",
    "add_comm",
    "add_zero",
    "zero_add",
    "neg_add_cancel",
    "add_neg_cancel",
    "sub_self",
    "mul_assoc",
    "mul_comm",
    "mul_one",
    "one_mul",
    "mul_zero",
    "zero_mul",
    "left_distrib",
    "right_distrib",
    "add_left_cancel",
    "mul_add",
    "add_mul",
    "ring_normalize_add_mul3",
];

const SQUARE_DEFINITIONS: &[&str] = &["two", "sq"];

const SQUARE_THEOREMS: &[&str] = &[
    "square_def",
    "mul_self_eq_square",
    "sq_zero",
    "sq_one",
    "sq_neg",
    "two_mul",
    "sq_add",
    "sq_sub",
    "sum_two_squares_comm",
    "sq_eq_sq_of_eq_or_neg_eq",
    "square_nonneg",
];

const ORDERED_FIELD_DEFINITIONS: &[&str] = &["le", "lt", "sqrt"];

const ORDERED_FIELD_THEOREMS: &[&str] = &[
    "le_refl",
    "le_trans",
    "add_nonneg",
    "mul_nonneg",
    "le_square_nonneg",
    "sqrt_nonneg",
    "sqrt_square_of_nonneg",
    "sqrt_mul_self",
    "eq_of_square_eq_square_nonneg",
];

const VECTOR_BASIC_INDUCTIVES: &[&str] = &["Vec"];

const VECTOR_BASIC_DEFINITIONS: &[&str] = &["vec_zero", "vec_add", "vec_neg", "vec_sub"];

const VECTOR_BASIC_THEOREMS: &[&str] = &[
    "vec_add_assoc",
    "vec_add_comm",
    "vec_zero_add",
    "vec_add_zero",
    "vec_neg_add_cancel",
    "vec_add_neg_cancel",
    "vec_sub_def",
    "vec_sub_eq_add_neg",
    "vec_sub_self",
    "vec_sub_zero",
    "vec_add_left_cancel",
    "sub_sub_sub_cancel",
];

const VECTOR_DOT_DEFINITIONS: &[&str] = &["dot", "normSq", "distSq"];

const VECTOR_DOT_THEOREMS: &[&str] = &[
    "dot_comm",
    "dot_add_left",
    "dot_add_right",
    "dot_neg_left",
    "dot_neg_right",
    "dot_sub_left",
    "dot_sub_right",
    "norm_sq_def",
    "dist_sq_def",
    "dot_self_eq_norm_sq",
    "norm_sq_add",
    "norm_sq_sub",
    "norm_sq_add_of_dot_zero",
    "norm_sq_sub_of_dot_zero",
    "parallelogram_law",
    "polarization_identity",
    "norm_sq_nonneg",
];

const RIGHT_TRIANGLE_DEFINITIONS: &[&str] = &["Perp", "RightTriangle"];

const RIGHT_TRIANGLE_THEOREMS: &[&str] = &[
    "perp_iff_dot_eq_zero",
    "perp_symm",
    "right_triangle_legs_perp",
    "hypotenuse_vector_eq_sub_legs",
    "dist_sq_leg_left",
    "dist_sq_leg_right",
    "dist_sq_hypotenuse",
    "pythagorean_distance_sq",
    "law_of_cosines",
    "right_triangle_area",
    "median_to_hypotenuse",
    "altitude_on_hypotenuse",
    "thales_theorem",
];

const METRIC_DEFINITIONS: &[&str] = &["dist"];

const METRIC_THEOREMS: &[&str] = &[
    "dist_def",
    "dist_sq_eq_square_dist",
    "dist_nonneg",
    "distance_symm",
    "distance_zero_iff_eq",
    "pythagorean_distance",
    "cauchy_schwarz",
    "triangle_inequality",
];

const IFF_DEFINITIONS: &[&str] = &["Iff", "And", "Or", "False", "Not"];

const IFF_THEOREMS: &[&str] = &[
    "iff_refl",
    "iff_symm",
    "iff_trans",
    "iff_mp",
    "iff_mpr",
    "and_intro",
    "and_left",
    "and_right",
    "iff_of_eq",
    "false_elim",
    "not_intro",
    "not_elim",
    "or_inl",
    "or_inr",
    "or_elim",
    "iff_congr_arg",
];

const ABSTRACT_RING_DEFINITIONS: &[&str] = &["two", "sq", "RingLawArgs"];

const ABSTRACT_RING_THEOREMS: &[&str] = &[
    "sub_eq_add_neg",
    "add_assoc",
    "add_comm",
    "add_zero",
    "zero_add",
    "neg_add_cancel",
    "add_neg_cancel",
    "sub_self",
    "mul_assoc",
    "mul_comm",
    "mul_one",
    "one_mul",
    "left_distrib",
    "right_distrib",
    "mul_zero",
    "zero_mul",
    "add_left_cancel",
    "ring_normalize_add_mul3",
    "add_right_cancel",
    "neg_neg",
    "sub_zero",
    "zero_sub",
    "sub_add_cancel",
    "add_sub_cancel",
    "sub_add_sub_cancel",
];

const ABSTRACT_ORDERED_FIELD_DEFINITIONS: &[&str] = &[
    "le",
    "lt",
    "sqrt",
    "Nonneg",
    "Positive",
    "OrderedFieldLawArgs",
];

const ABSTRACT_ORDERED_FIELD_THEOREMS: &[&str] = &[
    "le_refl",
    "le_trans",
    "add_nonneg",
    "mul_nonneg",
    "square_nonneg",
    "sqrt_nonneg",
    "sqrt_square_of_nonneg",
    "sqrt_mul_self",
    "eq_of_square_eq_square_nonneg",
    "add_le_add",
    "mul_le_mul_nonneg",
    "zero_le_two",
    "le_antisymm",
    "lt_of_le_of_ne",
    "le_of_eq",
    "sqrt_sq",
    "sq_eq_zero_iff",
    "sum_nonneg_eq_zero",
];

const ABSTRACT_SQUARE_NORMALIZE_THEOREMS: &[&str] = &[
    "square_def",
    "mul_self_eq_square",
    "sq_add",
    "sq_sub",
    "sum_two_squares_comm",
    "cancel_double_zero_term",
    "sq_zero",
    "sq_one",
    "sq_neg",
    "two_mul",
    "sq_eq_sq_of_eq_or_neg_eq",
    "sq_add_eq_add_sq_add_two_mul",
    "sq_sub_eq_add_sq_sub_two_mul",
    "add_sq_eq_zero_iff",
    "mul_two_zero_term",
    "normalize_add_with_zero_cross_term",
];

const ABSTRACT_VECTOR_SPACE_DEFINITIONS: &[&str] =
    &["vsub", "linear_comb2", "linear_comb3", "VectorSpaceLawArgs"];

const ABSTRACT_VECTOR_SPACE_THEOREMS: &[&str] = &[
    "vec_sub_def",
    "vec_add_assoc",
    "vec_add_comm",
    "vec_add_zero",
    "vec_zero_add",
    "vec_neg_add_cancel",
    "vec_add_neg_cancel",
    "sub_sub_sub_cancel",
    "vec_sub_self",
    "vec_sub_zero",
    "vec_add_left_cancel",
    "smul_add",
    "add_smul",
    "one_smul",
    "mul_smul",
    "zero_smul",
    "smul_zero",
    "neg_smul",
    "smul_neg",
    "vec_sub_eq_add_neg",
    "sub_add_sub_cancel_left",
    "linear_comb2_ext",
    "linear_comb3_ext",
];

const EXPECTED_MODULES: &[ExpectedModule] = &[
    ExpectedModule {
        module: "Proofs.Ai.Basic",
        source: "Proofs/Ai/Basic/source.npa",
        certificate: "Proofs/Ai/Basic/certificate.npcert",
        meta: "Proofs/Ai/Basic/meta.json",
        replay: "Proofs/Ai/Basic/replay.json",
        imports: &[],
        inductives: &[],
        definitions: &[],
        theorems: BASIC_THEOREMS,
        axioms: &[],
    },
    ExpectedModule {
        module: "Proofs.Ai.Eq",
        source: "Proofs/Ai/Eq/source.npa",
        certificate: "Proofs/Ai/Eq/certificate.npcert",
        meta: "Proofs/Ai/Eq/meta.json",
        replay: "Proofs/Ai/Eq/replay.json",
        imports: &["Std.Logic.Eq", "Std.Nat.Basic"],
        inductives: &[],
        definitions: &[],
        theorems: EQ_THEOREMS,
        axioms: &[],
    },
    ExpectedModule {
        module: "Proofs.Ai.Nat",
        source: "Proofs/Ai/Nat/source.npa",
        certificate: "Proofs/Ai/Nat/certificate.npcert",
        meta: "Proofs/Ai/Nat/meta.json",
        replay: "Proofs/Ai/Nat/replay.json",
        imports: &["Std.Logic.Eq", "Std.Nat.Basic"],
        inductives: &[],
        definitions: &[],
        theorems: NAT_THEOREMS,
        axioms: &[],
    },
    ExpectedModule {
        module: "Proofs.Ai.Prop",
        source: "Proofs/Ai/Prop/source.npa",
        certificate: "Proofs/Ai/Prop/certificate.npcert",
        meta: "Proofs/Ai/Prop/meta.json",
        replay: "Proofs/Ai/Prop/replay.json",
        imports: &[],
        inductives: &[],
        definitions: &[],
        theorems: PROP_THEOREMS,
        axioms: &[],
    },
    ExpectedModule {
        module: "Proofs.Ai.Reduction",
        source: "Proofs/Ai/Reduction/source.npa",
        certificate: "Proofs/Ai/Reduction/certificate.npcert",
        meta: "Proofs/Ai/Reduction/meta.json",
        replay: "Proofs/Ai/Reduction/replay.json",
        imports: &["Std.Nat.Basic"],
        inductives: &[],
        definitions: REDUCTION_DEFINITIONS,
        theorems: REDUCTION_THEOREMS,
        axioms: &[],
    },
    ExpectedModule {
        module: "Proofs.Ai.EqReasoning",
        source: "Proofs/Ai/EqReasoning/source.npa",
        certificate: "Proofs/Ai/EqReasoning/certificate.npcert",
        meta: "Proofs/Ai/EqReasoning/meta.json",
        replay: "Proofs/Ai/EqReasoning/replay.json",
        imports: &["Std.Logic.Eq"],
        inductives: &[],
        definitions: &[],
        theorems: EQ_REASONING_THEOREMS,
        axioms: &["Eq.rec"],
    },
    ExpectedModule {
        module: "Proofs.Ai.Algebra.Ring",
        source: "Proofs/Ai/Algebra/Ring/source.npa",
        certificate: "Proofs/Ai/Algebra/Ring/certificate.npcert",
        meta: "Proofs/Ai/Algebra/Ring/meta.json",
        replay: "Proofs/Ai/Algebra/Ring/replay.json",
        imports: &["Std.Logic.Eq"],
        inductives: RING_INDUCTIVES,
        definitions: RING_DEFINITIONS,
        theorems: RING_THEOREMS,
        axioms: &[],
    },
    ExpectedModule {
        module: "Proofs.Ai.Algebra.Square",
        source: "Proofs/Ai/Algebra/Square/source.npa",
        certificate: "Proofs/Ai/Algebra/Square/certificate.npcert",
        meta: "Proofs/Ai/Algebra/Square/meta.json",
        replay: "Proofs/Ai/Algebra/Square/replay.json",
        imports: &["Proofs.Ai.Algebra.Ring", "Std.Logic.Eq"],
        inductives: &[],
        definitions: SQUARE_DEFINITIONS,
        theorems: SQUARE_THEOREMS,
        axioms: &[],
    },
    ExpectedModule {
        module: "Proofs.Ai.OrderedField",
        source: "Proofs/Ai/OrderedField/source.npa",
        certificate: "Proofs/Ai/OrderedField/certificate.npcert",
        meta: "Proofs/Ai/OrderedField/meta.json",
        replay: "Proofs/Ai/OrderedField/replay.json",
        imports: &[
            "Proofs.Ai.Algebra.Ring",
            "Proofs.Ai.Algebra.Square",
            "Std.Logic.Eq",
        ],
        inductives: &[],
        definitions: ORDERED_FIELD_DEFINITIONS,
        theorems: ORDERED_FIELD_THEOREMS,
        axioms: &[],
    },
    ExpectedModule {
        module: "Proofs.Ai.Vector.Basic",
        source: "Proofs/Ai/Vector/Basic/source.npa",
        certificate: "Proofs/Ai/Vector/Basic/certificate.npcert",
        meta: "Proofs/Ai/Vector/Basic/meta.json",
        replay: "Proofs/Ai/Vector/Basic/replay.json",
        imports: &["Std.Logic.Eq"],
        inductives: VECTOR_BASIC_INDUCTIVES,
        definitions: VECTOR_BASIC_DEFINITIONS,
        theorems: VECTOR_BASIC_THEOREMS,
        axioms: &[],
    },
    ExpectedModule {
        module: "Proofs.Ai.Vector.Dot",
        source: "Proofs/Ai/Vector/Dot/source.npa",
        certificate: "Proofs/Ai/Vector/Dot/certificate.npcert",
        meta: "Proofs/Ai/Vector/Dot/meta.json",
        replay: "Proofs/Ai/Vector/Dot/replay.json",
        imports: &[
            "Proofs.Ai.Algebra.Ring",
            "Proofs.Ai.Algebra.Square",
            "Proofs.Ai.OrderedField",
            "Proofs.Ai.Vector.Basic",
            "Std.Logic.Eq",
        ],
        inductives: &[],
        definitions: VECTOR_DOT_DEFINITIONS,
        theorems: VECTOR_DOT_THEOREMS,
        axioms: &[],
    },
    ExpectedModule {
        module: "Proofs.Ai.Geometry.RightTriangle",
        source: "Proofs/Ai/Geometry/RightTriangle/source.npa",
        certificate: "Proofs/Ai/Geometry/RightTriangle/certificate.npcert",
        meta: "Proofs/Ai/Geometry/RightTriangle/meta.json",
        replay: "Proofs/Ai/Geometry/RightTriangle/replay.json",
        imports: &[
            "Proofs.Ai.Algebra.Ring",
            "Proofs.Ai.Algebra.Square",
            "Proofs.Ai.OrderedField",
            "Proofs.Ai.Vector.Basic",
            "Proofs.Ai.Vector.Dot",
            "Std.Logic.Eq",
        ],
        inductives: &[],
        definitions: RIGHT_TRIANGLE_DEFINITIONS,
        theorems: RIGHT_TRIANGLE_THEOREMS,
        axioms: &[],
    },
    ExpectedModule {
        module: "Proofs.Ai.Geometry.Metric",
        source: "Proofs/Ai/Geometry/Metric/source.npa",
        certificate: "Proofs/Ai/Geometry/Metric/certificate.npcert",
        meta: "Proofs/Ai/Geometry/Metric/meta.json",
        replay: "Proofs/Ai/Geometry/Metric/replay.json",
        imports: &[
            "Proofs.Ai.Algebra.Ring",
            "Proofs.Ai.Algebra.Square",
            "Proofs.Ai.Geometry.RightTriangle",
            "Proofs.Ai.OrderedField",
            "Proofs.Ai.Vector.Basic",
            "Proofs.Ai.Vector.Dot",
            "Std.Logic.Eq",
        ],
        inductives: &[],
        definitions: METRIC_DEFINITIONS,
        theorems: METRIC_THEOREMS,
        axioms: &[],
    },
    ExpectedModule {
        module: "Proofs.Ai.Logic.Iff",
        source: "Proofs/Ai/Logic/Iff/source.npa",
        certificate: "Proofs/Ai/Logic/Iff/certificate.npcert",
        meta: "Proofs/Ai/Logic/Iff/meta.json",
        replay: "Proofs/Ai/Logic/Iff/replay.json",
        imports: &["Std.Logic.Eq"],
        inductives: &[],
        definitions: IFF_DEFINITIONS,
        theorems: IFF_THEOREMS,
        axioms: &["Eq.rec"],
    },
    ExpectedModule {
        module: "Proofs.Ai.Algebra.AbstractRing",
        source: "Proofs/Ai/Algebra/AbstractRing/source.npa",
        certificate: "Proofs/Ai/Algebra/AbstractRing/certificate.npcert",
        meta: "Proofs/Ai/Algebra/AbstractRing/meta.json",
        replay: "Proofs/Ai/Algebra/AbstractRing/replay.json",
        imports: &["Std.Logic.Eq"],
        inductives: &[],
        definitions: ABSTRACT_RING_DEFINITIONS,
        theorems: ABSTRACT_RING_THEOREMS,
        axioms: &[],
    },
    ExpectedModule {
        module: "Proofs.Ai.Algebra.AbstractOrderedField",
        source: "Proofs/Ai/Algebra/AbstractOrderedField/source.npa",
        certificate: "Proofs/Ai/Algebra/AbstractOrderedField/certificate.npcert",
        meta: "Proofs/Ai/Algebra/AbstractOrderedField/meta.json",
        replay: "Proofs/Ai/Algebra/AbstractOrderedField/replay.json",
        imports: &["Proofs.Ai.Algebra.AbstractRing", "Std.Logic.Eq"],
        inductives: &[],
        definitions: ABSTRACT_ORDERED_FIELD_DEFINITIONS,
        theorems: ABSTRACT_ORDERED_FIELD_THEOREMS,
        axioms: &[],
    },
    ExpectedModule {
        module: "Proofs.Ai.Algebra.AbstractSquareNormalize",
        source: "Proofs/Ai/Algebra/AbstractSquareNormalize/source.npa",
        certificate: "Proofs/Ai/Algebra/AbstractSquareNormalize/certificate.npcert",
        meta: "Proofs/Ai/Algebra/AbstractSquareNormalize/meta.json",
        replay: "Proofs/Ai/Algebra/AbstractSquareNormalize/replay.json",
        imports: &[
            "Proofs.Ai.Algebra.AbstractOrderedField",
            "Proofs.Ai.Algebra.AbstractRing",
            "Std.Logic.Eq",
        ],
        inductives: &[],
        definitions: &[],
        theorems: ABSTRACT_SQUARE_NORMALIZE_THEOREMS,
        axioms: &[],
    },
    ExpectedModule {
        module: "Proofs.Ai.Vector.AbstractSpace",
        source: "Proofs/Ai/Vector/AbstractSpace/source.npa",
        certificate: "Proofs/Ai/Vector/AbstractSpace/certificate.npcert",
        meta: "Proofs/Ai/Vector/AbstractSpace/meta.json",
        replay: "Proofs/Ai/Vector/AbstractSpace/replay.json",
        imports: &[
            "Proofs.Ai.Algebra.AbstractOrderedField",
            "Proofs.Ai.Algebra.AbstractRing",
            "Proofs.Ai.Algebra.AbstractSquareNormalize",
            "Std.Logic.Eq",
        ],
        inductives: &[],
        definitions: ABSTRACT_VECTOR_SPACE_DEFINITIONS,
        theorems: ABSTRACT_VECTOR_SPACE_THEOREMS,
        axioms: &[],
    },
];

#[test]
fn ai_certificates_match_manifest_and_verify() {
    let root = corpus_root();
    let manifest = read_to_string(root.join("manifest.toml"));
    assert_eq!(
        quoted_value(&manifest, "schema"),
        "npa-ai-proof-corpus-v0.1"
    );
    let eq_import = verified_eq_import_module();
    let nat_import = verified_nat_import_module();
    let ring_import = verified_ring_import_module(&root, &eq_import);
    let square_import = verified_square_import_module(&root, &eq_import, &ring_import);
    let ordered_field_import =
        verified_ordered_field_import_module(&root, &eq_import, &ring_import, &square_import);
    let vector_basic_import = verified_vector_basic_import_module(&root, &eq_import);
    let vector_dot_import = verified_vector_dot_import_module(
        &root,
        &eq_import,
        &ring_import,
        &square_import,
        &ordered_field_import,
        &vector_basic_import,
    );
    let right_triangle_import = verified_right_triangle_import_module(
        &root,
        &eq_import,
        &ring_import,
        &square_import,
        &ordered_field_import,
        &vector_basic_import,
        &vector_dot_import,
    );
    let abstract_ring_import = verified_abstract_ring_import_module(&root, &eq_import);
    let abstract_ordered_field_import =
        verified_abstract_ordered_field_import_module(&root, &eq_import, &abstract_ring_import);
    let abstract_square_normalize_import = verified_abstract_square_normalize_import_module(
        &root,
        &eq_import,
        &abstract_ring_import,
        &abstract_ordered_field_import,
    );
    let verified_imports = VerifiedCorpusImports {
        eq: &eq_import,
        nat: &nat_import,
        ring: &ring_import,
        square: &square_import,
        ordered_field: &ordered_field_import,
        vector_basic: &vector_basic_import,
        vector_dot: &vector_dot_import,
        right_triangle: &right_triangle_import,
        abstract_ring: &abstract_ring_import,
        abstract_ordered_field: &abstract_ordered_field_import,
        abstract_square_normalize: &abstract_square_normalize_import,
    };

    for expected in EXPECTED_MODULES {
        let block = manifest_block(&manifest, expected.module);
        assert_eq!(
            quoted_value(block, "trusted_status"),
            "verified_by_certificate"
        );
        assert_eq!(quoted_value(block, "source"), expected.source);
        assert_eq!(quoted_value(block, "certificate"), expected.certificate);
        assert_eq!(quoted_value(block, "meta"), expected.meta);
        assert_eq!(quoted_value(block, "replay"), expected.replay);
        for import in expected.imports {
            assert!(block.contains(&format!("\"{import}\"")));
        }

        let source_sha256 = quoted_value(block, "source_sha256");
        let certificate_file_sha256 = quoted_value(block, "certificate_file_sha256");
        let export_hash = quoted_value(block, "export_hash");
        let axiom_report_hash = quoted_value(block, "axiom_report_hash");
        let certificate_hash = quoted_value(block, "certificate_hash");

        let source_bytes = read(root.join(expected.source));
        assert_eq!(tagged_sha256(&source_bytes), source_sha256);

        let certificate_bytes = read(root.join(expected.certificate));
        assert_eq!(tagged_sha256(&certificate_bytes), certificate_file_sha256);

        let decoded =
            decode_module_cert(&certificate_bytes).expect("AI corpus certificate decodes");
        assert_eq!(decoded.header.module, Name::from_dotted(expected.module));
        assert_eq!(tagged_hash(decoded.hashes.export_hash), export_hash);
        assert_eq!(
            tagged_hash(decoded.hashes.axiom_report_hash),
            axiom_report_hash
        );
        assert_eq!(
            tagged_hash(decoded.hashes.certificate_hash),
            certificate_hash
        );
        assert_axioms(&decoded, expected.axioms);
        assert_imports(&decoded, expected.imports);

        let mut session = VerifierSession::new();
        register_expected_imports(&mut session, expected.imports, &verified_imports);
        let verified = verify_module_cert(&certificate_bytes, &mut session, &AxiomPolicy::normal())
            .expect("AI corpus certificate verifies");
        assert_eq!(verified.module(), &Name::from_dotted(expected.module));
        assert_eq!(tagged_hash(verified.export_hash()), export_hash);
        assert_eq!(tagged_hash(verified.certificate_hash()), certificate_hash);
        let verified_axioms = verified
            .axiom_report()
            .module_axioms
            .iter()
            .map(|axiom| verified.name_table()[axiom.name].as_dotted())
            .collect::<Vec<_>>();
        assert_eq!(verified_axioms, expected.axioms);

        assert_definition_exports(&decoded, expected.definitions);
        assert_inductive_exports(&decoded, expected.inductives);
        assert_theorem_exports(&decoded, expected.theorems);
        if expected.module == "Proofs.Ai.Algebra.Ring" {
            assert_export(&decoded, "RingElem.unit", ExportKind::Constructor);
            assert_export(&decoded, "RingElem.rec", ExportKind::Recursor);
        }
        if expected.module == "Proofs.Ai.Vector.Basic" {
            assert_export(&decoded, "Vec.unit", ExportKind::Constructor);
            assert_export(&decoded, "Vec.rec", ExportKind::Recursor);
        }
        assert_declarations(
            &decoded,
            expected.inductives,
            expected.definitions,
            expected.theorems,
        );

        let meta = read_to_string(root.join(expected.meta));
        assert!(meta.contains(&format!("\"certificate_hash\": \"{certificate_hash}\"")));
        assert!(meta.contains("\"trusted_status\": \"verified_by_certificate\""));
        for import in expected.imports {
            assert!(meta.contains(&format!("\"{import}\"")));
        }
        for inductive in expected.inductives {
            assert!(meta.contains(&format!("\"name\": \"{inductive}\"")));
            assert!(block.contains(&format!("\"{inductive}\"")));
        }
        for definition in expected.definitions {
            assert!(meta.contains(&format!("\"name\": \"{definition}\"")));
            assert!(block.contains(&format!("\"{definition}\"")));
        }
        for theorem in expected.theorems {
            assert!(meta.contains(&format!("\"name\": \"{theorem}\"")));
            assert!(block.contains(&format!("\"{theorem}\"")));
        }
        for axiom in expected.axioms {
            assert!(meta.contains(&format!("\"{axiom}\"")));
            assert!(block.contains(&format!("\"{axiom}\"")));
        }

        let replay = read_to_string(root.join(expected.replay));
        assert!(replay.contains("\"trusted\": false"));
        assert!(replay.contains(&format!(
            "\"accepted_artifact\": \"{}\"",
            expected.certificate
        )));
    }
}

fn register_expected_imports(
    session: &mut VerifierSession,
    imports: &[&str],
    verified_imports: &VerifiedCorpusImports<'_>,
) {
    for import in imports {
        match *import {
            "Std.Logic.Eq" => session.register_verified_module(verified_imports.eq.clone()),
            "Std.Nat.Basic" => session.register_verified_module(verified_imports.nat.clone()),
            "Proofs.Ai.Algebra.Ring" => {
                session.register_verified_module(verified_imports.ring.clone())
            }
            "Proofs.Ai.Algebra.Square" => {
                session.register_verified_module(verified_imports.square.clone())
            }
            "Proofs.Ai.OrderedField" => {
                session.register_verified_module(verified_imports.ordered_field.clone())
            }
            "Proofs.Ai.Vector.Basic" => {
                session.register_verified_module(verified_imports.vector_basic.clone())
            }
            "Proofs.Ai.Vector.Dot" => {
                session.register_verified_module(verified_imports.vector_dot.clone())
            }
            "Proofs.Ai.Geometry.RightTriangle" => {
                session.register_verified_module(verified_imports.right_triangle.clone())
            }
            "Proofs.Ai.Algebra.AbstractRing" => {
                session.register_verified_module(verified_imports.abstract_ring.clone())
            }
            "Proofs.Ai.Algebra.AbstractOrderedField" => {
                session.register_verified_module(verified_imports.abstract_ordered_field.clone())
            }
            "Proofs.Ai.Algebra.AbstractSquareNormalize" => {
                session.register_verified_module(verified_imports.abstract_square_normalize.clone())
            }
            other => panic!("unexpected AI proof corpus import {other}"),
        }
    }
}

fn verified_eq_import_module() -> VerifiedModule {
    verified_core_module(CoreModule {
        name: Name::from_dotted("Std.Logic.Eq"),
        declarations: vec![Decl::Inductive {
            name: "Eq".to_owned(),
            universe_params: vec!["u".to_owned()],
            ty: npa_kernel::eq_type(Level::param("u")),
            data: Box::new(npa_kernel::eq_inductive()),
        }],
    })
}

fn verified_nat_import_module() -> VerifiedModule {
    verified_core_module(CoreModule {
        name: Name::from_dotted("Std.Nat.Basic"),
        declarations: vec![Decl::Inductive {
            name: "Nat".to_owned(),
            universe_params: Vec::new(),
            ty: npa_kernel::Expr::sort(npa_kernel::type0()),
            data: Box::new(npa_kernel::nat_inductive()),
        }],
    })
}

fn verified_ring_import_module(root: &Path, eq_import: &VerifiedModule) -> VerifiedModule {
    let bytes = read(root.join("Proofs/Ai/Algebra/Ring/certificate.npcert"));
    let mut session = VerifierSession::new();
    session.register_verified_module(eq_import.clone());
    verify_module_cert(&bytes, &mut session, &AxiomPolicy::normal())
        .expect("Ring corpus certificate should verify for downstream imports")
}

fn verified_square_import_module(
    root: &Path,
    eq_import: &VerifiedModule,
    ring_import: &VerifiedModule,
) -> VerifiedModule {
    let bytes = read(root.join("Proofs/Ai/Algebra/Square/certificate.npcert"));
    let mut session = VerifierSession::new();
    session.register_verified_module(eq_import.clone());
    session.register_verified_module(ring_import.clone());
    verify_module_cert(&bytes, &mut session, &AxiomPolicy::normal())
        .expect("Square corpus certificate should verify for downstream imports")
}

fn verified_ordered_field_import_module(
    root: &Path,
    eq_import: &VerifiedModule,
    ring_import: &VerifiedModule,
    square_import: &VerifiedModule,
) -> VerifiedModule {
    let bytes = read(root.join("Proofs/Ai/OrderedField/certificate.npcert"));
    let mut session = VerifierSession::new();
    session.register_verified_module(eq_import.clone());
    session.register_verified_module(ring_import.clone());
    session.register_verified_module(square_import.clone());
    verify_module_cert(&bytes, &mut session, &AxiomPolicy::normal())
        .expect("OrderedField corpus certificate should verify for downstream imports")
}

fn verified_vector_basic_import_module(root: &Path, eq_import: &VerifiedModule) -> VerifiedModule {
    let bytes = read(root.join("Proofs/Ai/Vector/Basic/certificate.npcert"));
    let mut session = VerifierSession::new();
    session.register_verified_module(eq_import.clone());
    verify_module_cert(&bytes, &mut session, &AxiomPolicy::normal())
        .expect("Vector.Basic corpus certificate should verify for downstream imports")
}

fn verified_vector_dot_import_module(
    root: &Path,
    eq_import: &VerifiedModule,
    ring_import: &VerifiedModule,
    square_import: &VerifiedModule,
    ordered_field_import: &VerifiedModule,
    vector_basic_import: &VerifiedModule,
) -> VerifiedModule {
    let bytes = read(root.join("Proofs/Ai/Vector/Dot/certificate.npcert"));
    let mut session = VerifierSession::new();
    session.register_verified_module(eq_import.clone());
    session.register_verified_module(ring_import.clone());
    session.register_verified_module(square_import.clone());
    session.register_verified_module(ordered_field_import.clone());
    session.register_verified_module(vector_basic_import.clone());
    verify_module_cert(&bytes, &mut session, &AxiomPolicy::normal())
        .expect("Vector.Dot corpus certificate should verify for downstream imports")
}

fn verified_right_triangle_import_module(
    root: &Path,
    eq_import: &VerifiedModule,
    ring_import: &VerifiedModule,
    square_import: &VerifiedModule,
    ordered_field_import: &VerifiedModule,
    vector_basic_import: &VerifiedModule,
    vector_dot_import: &VerifiedModule,
) -> VerifiedModule {
    let bytes = read(root.join("Proofs/Ai/Geometry/RightTriangle/certificate.npcert"));
    let mut session = VerifierSession::new();
    session.register_verified_module(eq_import.clone());
    session.register_verified_module(ring_import.clone());
    session.register_verified_module(square_import.clone());
    session.register_verified_module(ordered_field_import.clone());
    session.register_verified_module(vector_basic_import.clone());
    session.register_verified_module(vector_dot_import.clone());
    verify_module_cert(&bytes, &mut session, &AxiomPolicy::normal())
        .expect("RightTriangle corpus certificate should verify for downstream imports")
}

fn verified_abstract_ring_import_module(root: &Path, eq_import: &VerifiedModule) -> VerifiedModule {
    let bytes = read(root.join("Proofs/Ai/Algebra/AbstractRing/certificate.npcert"));
    let mut session = VerifierSession::new();
    session.register_verified_module(eq_import.clone());
    verify_module_cert(&bytes, &mut session, &AxiomPolicy::normal())
        .expect("AbstractRing corpus certificate should verify for downstream imports")
}

fn verified_abstract_ordered_field_import_module(
    root: &Path,
    eq_import: &VerifiedModule,
    abstract_ring_import: &VerifiedModule,
) -> VerifiedModule {
    let bytes = read(root.join("Proofs/Ai/Algebra/AbstractOrderedField/certificate.npcert"));
    let mut session = VerifierSession::new();
    session.register_verified_module(abstract_ring_import.clone());
    session.register_verified_module(eq_import.clone());
    verify_module_cert(&bytes, &mut session, &AxiomPolicy::normal())
        .expect("AbstractOrderedField corpus certificate should verify for downstream imports")
}

fn verified_abstract_square_normalize_import_module(
    root: &Path,
    eq_import: &VerifiedModule,
    abstract_ring_import: &VerifiedModule,
    abstract_ordered_field_import: &VerifiedModule,
) -> VerifiedModule {
    let bytes = read(root.join("Proofs/Ai/Algebra/AbstractSquareNormalize/certificate.npcert"));
    let mut session = VerifierSession::new();
    session.register_verified_module(abstract_ordered_field_import.clone());
    session.register_verified_module(abstract_ring_import.clone());
    session.register_verified_module(eq_import.clone());
    verify_module_cert(&bytes, &mut session, &AxiomPolicy::normal())
        .expect("AbstractSquareNormalize corpus certificate should verify for downstream imports")
}

fn verified_core_module(module: CoreModule) -> VerifiedModule {
    let cert = build_module_cert(module, &[]).expect("import certificate should build");
    let bytes = encode_module_cert(&cert).expect("import certificate should encode");
    verify_module_cert(&bytes, &mut VerifierSession::new(), &AxiomPolicy::normal())
        .expect("import certificate should verify")
}

fn corpus_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("npa-cert crate lives under crates/")
        .join("proofs")
}

fn read(path: PathBuf) -> Vec<u8> {
    fs::read(&path).unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn read_to_string(path: PathBuf) -> String {
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn manifest_block<'a>(manifest: &'a str, module: &str) -> &'a str {
    manifest
        .split("[[proof_modules]]")
        .skip(1)
        .find(|block| quoted_value(block, "module") == module)
        .unwrap_or_else(|| panic!("manifest block for {module} not found"))
}

fn quoted_value(text: &str, key: &str) -> String {
    let prefix = format!("{key} = ");
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix(&prefix) {
            return value
                .trim()
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .unwrap_or_else(|| panic!("manifest value for {key} is not quoted"))
                .to_owned();
        }
    }
    panic!("manifest key {key} not found");
}

fn assert_imports(cert: &npa_cert::ModuleCert, expected: &[&str]) {
    let actual = cert
        .imports
        .iter()
        .map(|import| import.module.as_dotted())
        .collect::<Vec<_>>();
    let expected = expected
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

fn assert_axioms(cert: &npa_cert::ModuleCert, expected: &[&str]) {
    let actual = cert
        .axiom_report
        .module_axioms
        .iter()
        .map(|axiom| cert.name_table[axiom.name].as_dotted())
        .collect::<Vec<_>>();
    let expected = expected
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}

fn assert_definition_exports(cert: &npa_cert::ModuleCert, expected: &[&str]) {
    let mut actual = cert
        .export_block
        .iter()
        .filter(|entry| entry.kind == ExportKind::Def)
        .map(|entry| cert.name_table[entry.name].as_dotted())
        .collect::<Vec<_>>();
    actual.sort();

    let mut expected = expected
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<Vec<_>>();
    expected.sort();

    assert_eq!(actual, expected);
}

fn assert_inductive_exports(cert: &npa_cert::ModuleCert, expected: &[&str]) {
    let mut actual = cert
        .export_block
        .iter()
        .filter(|entry| entry.kind == ExportKind::Inductive)
        .map(|entry| cert.name_table[entry.name].as_dotted())
        .collect::<Vec<_>>();
    actual.sort();

    let mut expected = expected
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<Vec<_>>();
    expected.sort();

    assert_eq!(actual, expected);
}

fn assert_theorem_exports(cert: &npa_cert::ModuleCert, expected: &[&str]) {
    let mut actual = cert
        .export_block
        .iter()
        .filter(|entry| entry.kind == ExportKind::Theorem)
        .map(|entry| cert.name_table[entry.name].as_dotted())
        .collect::<Vec<_>>();
    actual.sort();

    let mut expected = expected
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<Vec<_>>();
    expected.sort();

    assert_eq!(actual, expected);
}

fn assert_export(cert: &npa_cert::ModuleCert, expected_name: &str, expected_kind: ExportKind) {
    assert!(
        cert.export_block.iter().any(|entry| {
            entry.kind == expected_kind && cert.name_table[entry.name].as_dotted() == expected_name
        }),
        "expected export {expected_name} with kind {expected_kind:?}"
    );
}

fn assert_declarations(
    cert: &npa_cert::ModuleCert,
    inductives: &[&str],
    definitions: &[&str],
    theorems: &[&str],
) {
    let mut actual = cert
        .declarations
        .iter()
        .map(|decl| match &decl.decl {
            DeclPayload::Inductive { name, .. } => {
                (cert.name_table[*name].as_dotted(), "inductive")
            }
            DeclPayload::Def { name, .. } => (cert.name_table[*name].as_dotted(), "def"),
            DeclPayload::Theorem { name, .. } => (cert.name_table[*name].as_dotted(), "theorem"),
            other => {
                panic!(
                    "AI proof corpus should contain only inductive/def/theorem declarations: {other:?}"
                )
            }
        })
        .collect::<Vec<_>>();
    actual.sort();

    let mut expected = inductives
        .iter()
        .map(|name| ((*name).to_owned(), "inductive"))
        .chain(definitions.iter().map(|name| ((*name).to_owned(), "def")))
        .chain(theorems.iter().map(|name| ((*name).to_owned(), "theorem")))
        .collect::<Vec<_>>();
    expected.sort();

    assert_eq!(actual, expected);
}

fn tagged_sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("sha256:{}", hex_bytes(&digest))
}

fn tagged_hash(hash: npa_cert::Hash) -> String {
    format!("sha256:{}", hex_bytes(&hash))
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}
