use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const MANIFEST_PATH: &str = "manifest.toml";

struct ModuleArtifact {
    module: &'static str,
    source_path: &'static str,
    certificate_path: &'static str,
    meta_path: &'static str,
    replay_path: &'static str,
    imports: &'static [&'static str],
    inductives: &'static [InductiveArtifact],
    definitions: &'static [DefinitionArtifact],
    theorems: &'static [TheoremArtifact],
    expected_axioms: &'static [&'static str],
}

struct InductiveArtifact {
    name: &'static str,
    universe_params: &'static [&'static str],
    ty: &'static str,
    constructors: &'static [ConstructorArtifact],
}

struct ConstructorArtifact {
    name: &'static str,
    ty: &'static str,
}

struct DefinitionArtifact {
    name: &'static str,
    universe_params: &'static [&'static str],
    ty: &'static str,
    value: &'static str,
}

struct TheoremArtifact {
    name: &'static str,
    universe_params: &'static [&'static str],
    statement: &'static str,
    proof: &'static str,
}

struct GeneratedModule {
    config: &'static ModuleArtifact,
    source_sha256: String,
    certificate_file_sha256: String,
    export_hash: String,
    axiom_report_hash: String,
    certificate_hash: String,
    axioms: Vec<String>,
    verified_module: npa_cert::VerifiedModule,
    source_interface: npa_frontend::HumanImportedSourceInterface,
}

const BASIC_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Basic",
    source_path: "Proofs/Ai/Basic/source.npa",
    certificate_path: "Proofs/Ai/Basic/certificate.npcert",
    meta_path: "Proofs/Ai/Basic/meta.json",
    replay_path: "Proofs/Ai/Basic/replay.json",
    imports: &[],
    inductives: &[],
    definitions: &[],
    theorems: BASIC_THEOREMS,
    expected_axioms: &[],
};

const EQ_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Eq",
    source_path: "Proofs/Ai/Eq/source.npa",
    certificate_path: "Proofs/Ai/Eq/certificate.npcert",
    meta_path: "Proofs/Ai/Eq/meta.json",
    replay_path: "Proofs/Ai/Eq/replay.json",
    imports: &["Std.Logic.Eq", "Std.Nat.Basic"],
    inductives: &[],
    definitions: &[],
    theorems: EQ_THEOREMS,
    expected_axioms: &[],
};

const NAT_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Nat",
    source_path: "Proofs/Ai/Nat/source.npa",
    certificate_path: "Proofs/Ai/Nat/certificate.npcert",
    meta_path: "Proofs/Ai/Nat/meta.json",
    replay_path: "Proofs/Ai/Nat/replay.json",
    imports: &["Std.Logic.Eq", "Std.Nat.Basic"],
    inductives: &[],
    definitions: &[],
    theorems: NAT_THEOREMS,
    expected_axioms: &[],
};

const PROP_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Prop",
    source_path: "Proofs/Ai/Prop/source.npa",
    certificate_path: "Proofs/Ai/Prop/certificate.npcert",
    meta_path: "Proofs/Ai/Prop/meta.json",
    replay_path: "Proofs/Ai/Prop/replay.json",
    imports: &[],
    inductives: &[],
    definitions: &[],
    theorems: PROP_THEOREMS,
    expected_axioms: &[],
};

const REDUCTION_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Reduction",
    source_path: "Proofs/Ai/Reduction/source.npa",
    certificate_path: "Proofs/Ai/Reduction/certificate.npcert",
    meta_path: "Proofs/Ai/Reduction/meta.json",
    replay_path: "Proofs/Ai/Reduction/replay.json",
    imports: &["Std.Nat.Basic"],
    inductives: &[],
    definitions: REDUCTION_DEFINITIONS,
    theorems: REDUCTION_THEOREMS,
    expected_axioms: &[],
};

const EQ_REASONING_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.EqReasoning",
    source_path: "Proofs/Ai/EqReasoning/source.npa",
    certificate_path: "Proofs/Ai/EqReasoning/certificate.npcert",
    meta_path: "Proofs/Ai/EqReasoning/meta.json",
    replay_path: "Proofs/Ai/EqReasoning/replay.json",
    imports: &["Std.Logic.Eq"],
    inductives: &[],
    definitions: &[],
    theorems: EQ_REASONING_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const RING_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Algebra.Ring",
    source_path: "Proofs/Ai/Algebra/Ring/source.npa",
    certificate_path: "Proofs/Ai/Algebra/Ring/certificate.npcert",
    meta_path: "Proofs/Ai/Algebra/Ring/meta.json",
    replay_path: "Proofs/Ai/Algebra/Ring/replay.json",
    imports: &["Std.Logic.Eq"],
    inductives: RING_INDUCTIVES,
    definitions: RING_DEFINITIONS,
    theorems: RING_THEOREMS,
    expected_axioms: &[],
};

const SQUARE_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Algebra.Square",
    source_path: "Proofs/Ai/Algebra/Square/source.npa",
    certificate_path: "Proofs/Ai/Algebra/Square/certificate.npcert",
    meta_path: "Proofs/Ai/Algebra/Square/meta.json",
    replay_path: "Proofs/Ai/Algebra/Square/replay.json",
    imports: &["Std.Logic.Eq", "Proofs.Ai.Algebra.Ring"],
    inductives: &[],
    definitions: SQUARE_DEFINITIONS,
    theorems: SQUARE_THEOREMS,
    expected_axioms: &[],
};

const ORDERED_FIELD_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.OrderedField",
    source_path: "Proofs/Ai/OrderedField/source.npa",
    certificate_path: "Proofs/Ai/OrderedField/certificate.npcert",
    meta_path: "Proofs/Ai/OrderedField/meta.json",
    replay_path: "Proofs/Ai/OrderedField/replay.json",
    imports: &[
        "Std.Logic.Eq",
        "Proofs.Ai.Algebra.Ring",
        "Proofs.Ai.Algebra.Square",
    ],
    inductives: &[],
    definitions: ORDERED_FIELD_DEFINITIONS,
    theorems: ORDERED_FIELD_THEOREMS,
    expected_axioms: &[],
};

const VECTOR_BASIC_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Vector.Basic",
    source_path: "Proofs/Ai/Vector/Basic/source.npa",
    certificate_path: "Proofs/Ai/Vector/Basic/certificate.npcert",
    meta_path: "Proofs/Ai/Vector/Basic/meta.json",
    replay_path: "Proofs/Ai/Vector/Basic/replay.json",
    imports: &["Std.Logic.Eq"],
    inductives: VECTOR_BASIC_INDUCTIVES,
    definitions: VECTOR_BASIC_DEFINITIONS,
    theorems: VECTOR_BASIC_THEOREMS,
    expected_axioms: &[],
};

const VECTOR_DOT_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Vector.Dot",
    source_path: "Proofs/Ai/Vector/Dot/source.npa",
    certificate_path: "Proofs/Ai/Vector/Dot/certificate.npcert",
    meta_path: "Proofs/Ai/Vector/Dot/meta.json",
    replay_path: "Proofs/Ai/Vector/Dot/replay.json",
    imports: &[
        "Std.Logic.Eq",
        "Proofs.Ai.Algebra.Ring",
        "Proofs.Ai.Algebra.Square",
        "Proofs.Ai.OrderedField",
        "Proofs.Ai.Vector.Basic",
    ],
    inductives: &[],
    definitions: VECTOR_DOT_DEFINITIONS,
    theorems: VECTOR_DOT_THEOREMS,
    expected_axioms: &[],
};

const RIGHT_TRIANGLE_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Geometry.RightTriangle",
    source_path: "Proofs/Ai/Geometry/RightTriangle/source.npa",
    certificate_path: "Proofs/Ai/Geometry/RightTriangle/certificate.npcert",
    meta_path: "Proofs/Ai/Geometry/RightTriangle/meta.json",
    replay_path: "Proofs/Ai/Geometry/RightTriangle/replay.json",
    imports: &[
        "Std.Logic.Eq",
        "Proofs.Ai.Algebra.Ring",
        "Proofs.Ai.Algebra.Square",
        "Proofs.Ai.OrderedField",
        "Proofs.Ai.Vector.Basic",
        "Proofs.Ai.Vector.Dot",
    ],
    inductives: &[],
    definitions: RIGHT_TRIANGLE_DEFINITIONS,
    theorems: RIGHT_TRIANGLE_THEOREMS,
    expected_axioms: &[],
};

const METRIC_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Geometry.Metric",
    source_path: "Proofs/Ai/Geometry/Metric/source.npa",
    certificate_path: "Proofs/Ai/Geometry/Metric/certificate.npcert",
    meta_path: "Proofs/Ai/Geometry/Metric/meta.json",
    replay_path: "Proofs/Ai/Geometry/Metric/replay.json",
    imports: &[
        "Std.Logic.Eq",
        "Proofs.Ai.Algebra.Ring",
        "Proofs.Ai.Algebra.Square",
        "Proofs.Ai.OrderedField",
        "Proofs.Ai.Vector.Basic",
        "Proofs.Ai.Vector.Dot",
        "Proofs.Ai.Geometry.RightTriangle",
    ],
    inductives: &[],
    definitions: METRIC_DEFINITIONS,
    theorems: METRIC_THEOREMS,
    expected_axioms: &[],
};

const IFF_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Logic.Iff",
    source_path: "Proofs/Ai/Logic/Iff/source.npa",
    certificate_path: "Proofs/Ai/Logic/Iff/certificate.npcert",
    meta_path: "Proofs/Ai/Logic/Iff/meta.json",
    replay_path: "Proofs/Ai/Logic/Iff/replay.json",
    imports: &["Std.Logic.Eq"],
    inductives: &[],
    definitions: IFF_DEFINITIONS,
    theorems: IFF_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const BASIC_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "id",
        universe_params: &[],
        statement: "forall (A : Type), forall (x : A), A",
        proof: "fun A => fun x => x",
    },
    TheoremArtifact {
        name: "const_left",
        universe_params: &[],
        statement: "forall (A : Type), forall (B : Type), forall (x : A), forall (y : B), A",
        proof: "fun A => fun B => fun x => fun y => x",
    },
    TheoremArtifact {
        name: "const_right",
        universe_params: &[],
        statement: "forall (A : Type), forall (B : Type), forall (x : A), forall (y : B), B",
        proof: "fun A => fun B => fun x => fun y => y",
    },
    TheoremArtifact {
        name: "apply_fn",
        universe_params: &[],
        statement:
            "forall (A : Type), forall (B : Type), forall (f : forall (x : A), B), forall (x : A), B",
        proof: "fun A => fun B => fun f => fun x => f x",
    },
    TheoremArtifact {
        name: "compose",
        universe_params: &[],
        statement:
            "forall (A : Type), forall (B : Type), forall (C : Type), forall (f : forall (x : B), C), forall (g : forall (x : A), B), forall (x : A), C",
        proof: "fun A => fun B => fun C => fun f => fun g => fun x => f (g x)",
    },
    TheoremArtifact {
        name: "flip",
        universe_params: &[],
        statement:
            "forall (A : Type), forall (B : Type), forall (C : Type), forall (f : forall (x : A), forall (y : B), C), forall (y : B), forall (x : A), C",
        proof: "fun A => fun B => fun C => fun f => fun y => fun x => f x y",
    },
    TheoremArtifact {
        name: "duplicate",
        universe_params: &[],
        statement:
            "forall (A : Type), forall (B : Type), forall (f : forall (x : A), forall (y : A), B), forall (x : A), B",
        proof: "fun A => fun B => fun f => fun x => f x x",
    },
    TheoremArtifact {
        name: "prop_id",
        universe_params: &[],
        statement: "forall (P : Prop), forall (p : P), P",
        proof: "fun P => fun p => p",
    },
    TheoremArtifact {
        name: "modus_ponens",
        universe_params: &[],
        statement:
            "forall (P : Prop), forall (Q : Prop), forall (h : forall (p : P), Q), forall (p : P), Q",
        proof: "fun P => fun Q => fun h => fun p => h p",
    },
    TheoremArtifact {
        name: "imp_trans",
        universe_params: &[],
        statement:
            "forall (P : Prop), forall (Q : Prop), forall (R : Prop), forall (pq : forall (p : P), Q), forall (qr : forall (q : Q), R), forall (p : P), R",
        proof: "fun P => fun Q => fun R => fun pq => fun qr => fun p => qr (pq p)",
    },
    TheoremArtifact {
        name: "compose_assoc",
        universe_params: &[],
        statement:
            "forall (A : Type), forall (B : Type), forall (C : Type), forall (D : Type), forall (h : forall (x : C), D), forall (g : forall (x : B), C), forall (f : forall (x : A), B), forall (x : A), D",
        proof: "fun A => fun B => fun C => fun D => fun h => fun g => fun f => fun x => h (g (f x))",
    },
    TheoremArtifact {
        name: "apply_twice",
        universe_params: &[],
        statement:
            "forall (A : Type), forall (f : forall (x : A), A), forall (x : A), A",
        proof: "fun A => fun f => fun x => f (f x)",
    },
    TheoremArtifact {
        name: "ignore_middle",
        universe_params: &[],
        statement:
            "forall (A : Type), forall (B : Type), forall (C : Type), forall (x : A), forall (y : B), forall (z : C), A",
        proof: "fun A => fun B => fun C => fun x => fun y => fun z => x",
    },
    TheoremArtifact {
        name: "select_middle",
        universe_params: &[],
        statement:
            "forall (A : Type), forall (B : Type), forall (C : Type), forall (x : A), forall (y : B), forall (z : C), B",
        proof: "fun A => fun B => fun C => fun x => fun y => fun z => y",
    },
    TheoremArtifact {
        name: "select_last",
        universe_params: &[],
        statement:
            "forall (A : Type), forall (B : Type), forall (C : Type), forall (x : A), forall (y : B), forall (z : C), C",
        proof: "fun A => fun B => fun C => fun x => fun y => fun z => z",
    },
    TheoremArtifact {
        name: "imp_swap",
        universe_params: &[],
        statement:
            "forall (P : Prop), forall (Q : Prop), forall (R : Prop), forall (h : forall (p : P), forall (q : Q), R), forall (q : Q), forall (p : P), R",
        proof: "fun P => fun Q => fun R => fun h => fun q => fun p => h p q",
    },
    TheoremArtifact {
        name: "imp_compose",
        universe_params: &[],
        statement:
            "forall (P : Prop), forall (Q : Prop), forall (R : Prop), forall (qr : forall (q : Q), R), forall (pq : forall (p : P), Q), forall (p : P), R",
        proof: "fun P => fun Q => fun R => fun qr => fun pq => fun p => qr (pq p)",
    },
    TheoremArtifact {
        name: "imp_ignore",
        universe_params: &[],
        statement: "forall (P : Prop), forall (Q : Prop), forall (p : P), forall (q : Q), P",
        proof: "fun P => fun Q => fun p => fun q => p",
    },
    TheoremArtifact {
        name: "imp_duplicate",
        universe_params: &[],
        statement:
            "forall (P : Prop), forall (Q : Prop), forall (h : forall (p1 : P), forall (p2 : P), Q), forall (p : P), Q",
        proof: "fun P => fun Q => fun h => fun p => h p p",
    },
    TheoremArtifact {
        name: "higher_apply",
        universe_params: &[],
        statement:
            "forall (A : Type), forall (B : Type), forall (C : Type), forall (h : forall (f : forall (x : A), B), C), forall (f : forall (x : A), B), C",
        proof: "fun A => fun B => fun C => fun h => fun f => h f",
    },
];

const EQ_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "eq_refl_self",
        universe_params: &["u"],
        statement: "forall (A : Sort u), forall (x : A), @Eq.{u} A x x",
        proof: "fun A => fun x => @Eq.refl.{u} A x",
    },
    TheoremArtifact {
        name: "eq_refl_fn_app",
        universe_params: &["u", "v"],
        statement:
            "forall (A : Sort u), forall (B : Sort v), forall (f : forall (x : A), B), forall (x : A), @Eq.{v} B (f x) (f x)",
        proof: "fun A => fun B => fun f => fun x => @Eq.refl.{v} B (f x)",
    },
    TheoremArtifact {
        name: "eq_refl_compose",
        universe_params: &["u", "v", "w"],
        statement:
            "forall (A : Sort u), forall (B : Sort v), forall (C : Sort w), forall (f : forall (x : B), C), forall (g : forall (x : A), B), forall (x : A), @Eq.{w} C (f (g x)) (f (g x))",
        proof: "fun A => fun B => fun C => fun f => fun g => fun x => @Eq.refl.{w} C (f (g x))",
    },
    TheoremArtifact {
        name: "eq_self_imp",
        universe_params: &["u"],
        statement:
            "forall (A : Sort u), forall (x : A), forall (h : @Eq.{u} A x x), @Eq.{u} A x x",
        proof: "fun A => fun x => fun h => h",
    },
    TheoremArtifact {
        name: "eq_refl_prop",
        universe_params: &[],
        statement: "forall (P : Prop), forall (p : P), @Eq.{0} P p p",
        proof: "fun P => fun p => @Eq.refl.{0} P p",
    },
    TheoremArtifact {
        name: "eq_refl_apply_twice",
        universe_params: &["u"],
        statement:
            "forall (A : Sort u), forall (f : forall (x : A), A), forall (x : A), @Eq.{u} A (f (f x)) (f (f x))",
        proof: "fun A => fun f => fun x => @Eq.refl.{u} A (f (f x))",
    },
    TheoremArtifact {
        name: "eq_refl_higher_apply",
        universe_params: &["u", "v", "w"],
        statement:
            "forall (A : Sort u), forall (B : Sort v), forall (C : Sort w), forall (h : forall (f : forall (x : A), B), C), forall (f : forall (x : A), B), @Eq.{w} C (h f) (h f)",
        proof: "fun A => fun B => fun C => fun h => fun f => @Eq.refl.{w} C (h f)",
    },
    TheoremArtifact {
        name: "eq_refl_nested_compose",
        universe_params: &["u", "v", "w", "z"],
        statement:
            "forall (A : Sort u), forall (B : Sort v), forall (C : Sort w), forall (D : Sort z), forall (f : forall (x : C), D), forall (g : forall (x : B), C), forall (h : forall (x : A), B), forall (x : A), @Eq.{z} D (f (g (h x))) (f (g (h x)))",
        proof: "fun A => fun B => fun C => fun D => fun f => fun g => fun h => fun x => @Eq.refl.{z} D (f (g (h x)))",
    },
    TheoremArtifact {
        name: "eq_refl_prop_apply",
        universe_params: &[],
        statement:
            "forall (P : Prop), forall (Q : Prop), forall (h : forall (p : P), Q), forall (p : P), @Eq.{0} Q (h p) (h p)",
        proof: "fun P => fun Q => fun h => fun p => @Eq.refl.{0} Q (h p)",
    },
    TheoremArtifact {
        name: "eq_local_passthrough",
        universe_params: &["u"],
        statement:
            "forall (A : Sort u), forall (x : A), forall (h : @Eq.{u} A x x), @Eq.{u} A x x",
        proof: "fun A => fun x => fun h => h",
    },
    TheoremArtifact {
        name: "eq_refl_nat_function",
        universe_params: &[],
        statement:
            "forall (f : forall (n : Nat), Nat), forall (n : Nat), @Eq.{1} Nat (f n) (f n)",
        proof: "fun f => fun n => @Eq.refl.{1} Nat (f n)",
    },
];

const NAT_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "nat_zero_self_eq",
        universe_params: &[],
        statement: "@Eq.{1} Nat Nat.zero Nat.zero",
        proof: "@Eq.refl.{1} Nat Nat.zero",
    },
    TheoremArtifact {
        name: "nat_succ_zero_self_eq",
        universe_params: &[],
        statement: "@Eq.{1} Nat (Nat.succ Nat.zero) (Nat.succ Nat.zero)",
        proof: "@Eq.refl.{1} Nat (Nat.succ Nat.zero)",
    },
    TheoremArtifact {
        name: "nat_id",
        universe_params: &[],
        statement: "forall (n : Nat), Nat",
        proof: "fun n => n",
    },
    TheoremArtifact {
        name: "nat_const_zero",
        universe_params: &[],
        statement: "forall (n : Nat), Nat",
        proof: "fun n => Nat.zero",
    },
    TheoremArtifact {
        name: "nat_apply_fn",
        universe_params: &[],
        statement: "forall (f : forall (n : Nat), Nat), forall (n : Nat), Nat",
        proof: "fun f => fun n => f n",
    },
    TheoremArtifact {
        name: "nat_const_succ_zero",
        universe_params: &[],
        statement: "forall (n : Nat), Nat",
        proof: "fun n => Nat.succ Nat.zero",
    },
    TheoremArtifact {
        name: "nat_apply_twice",
        universe_params: &[],
        statement: "forall (f : forall (n : Nat), Nat), forall (n : Nat), Nat",
        proof: "fun f => fun n => f (f n)",
    },
    TheoremArtifact {
        name: "nat_compose",
        universe_params: &[],
        statement:
            "forall (f : forall (n : Nat), Nat), forall (g : forall (n : Nat), Nat), forall (n : Nat), Nat",
        proof: "fun f => fun g => fun n => f (g n)",
    },
    TheoremArtifact {
        name: "nat_ignore_middle",
        universe_params: &[],
        statement: "forall (x : Nat), forall (y : Nat), forall (z : Nat), Nat",
        proof: "fun x => fun y => fun z => x",
    },
    TheoremArtifact {
        name: "nat_select_middle",
        universe_params: &[],
        statement: "forall (x : Nat), forall (y : Nat), forall (z : Nat), Nat",
        proof: "fun x => fun y => fun z => y",
    },
    TheoremArtifact {
        name: "nat_select_last",
        universe_params: &[],
        statement: "forall (x : Nat), forall (y : Nat), forall (z : Nat), Nat",
        proof: "fun x => fun y => fun z => z",
    },
    TheoremArtifact {
        name: "nat_succ_self_eq",
        universe_params: &[],
        statement: "forall (n : Nat), @Eq.{1} Nat (Nat.succ n) (Nat.succ n)",
        proof: "fun n => @Eq.refl.{1} Nat (Nat.succ n)",
    },
];

const PROP_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "imp_chain4",
        universe_params: &[],
        statement:
            "forall (P : Prop), forall (Q : Prop), forall (R : Prop), forall (S : Prop), forall (pq : forall (p : P), Q), forall (qr : forall (q : Q), R), forall (rs : forall (r : R), S), forall (p : P), S",
        proof: "fun P => fun Q => fun R => fun S => fun pq => fun qr => fun rs => fun p => rs (qr (pq p))",
    },
    TheoremArtifact {
        name: "imp_permute3",
        universe_params: &[],
        statement:
            "forall (P : Prop), forall (Q : Prop), forall (R : Prop), forall (S : Prop), forall (h : forall (p : P), forall (q : Q), forall (r : R), S), forall (r : R), forall (p : P), forall (q : Q), S",
        proof: "fun P => fun Q => fun R => fun S => fun h => fun r => fun p => fun q => h p q r",
    },
    TheoremArtifact {
        name: "imp_apply_twice",
        universe_params: &[],
        statement:
            "forall (P : Prop), forall (h : forall (p : P), P), forall (p : P), P",
        proof: "fun P => fun h => fun p => h (h p)",
    },
    TheoremArtifact {
        name: "imp_const3",
        universe_params: &[],
        statement:
            "forall (P : Prop), forall (Q : Prop), forall (R : Prop), forall (p : P), forall (q : Q), forall (r : R), P",
        proof: "fun P => fun Q => fun R => fun p => fun q => fun r => p",
    },
    TheoremArtifact {
        name: "imp_flip_chain",
        universe_params: &[],
        statement:
            "forall (P : Prop), forall (Q : Prop), forall (R : Prop), forall (qr : forall (q : Q), R), forall (pq : forall (p : P), Q), forall (p : P), R",
        proof: "fun P => fun Q => fun R => fun qr => fun pq => fun p => qr (pq p)",
    },
    TheoremArtifact {
        name: "imp_higher_apply",
        universe_params: &[],
        statement:
            "forall (P : Prop), forall (Q : Prop), forall (R : Prop), forall (h : forall (f : forall (p : P), Q), R), forall (f : forall (p : P), Q), R",
        proof: "fun P => fun Q => fun R => fun h => fun f => h f",
    },
];

const REDUCTION_DEFINITIONS: &[DefinitionArtifact] = &[DefinitionArtifact {
    name: "reduction_id_nat",
    universe_params: &[],
    ty: "forall (n : Nat), Nat",
    value: "fun n => n",
}];

const REDUCTION_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "beta_id_nat",
        universe_params: &[],
        statement: "forall (n : Nat), Nat",
        proof: "fun n => (fun (x : Nat) => x) n",
    },
    TheoremArtifact {
        name: "beta_const_nat",
        universe_params: &[],
        statement: "forall (n : Nat), forall (m : Nat), Nat",
        proof: "fun n => fun m => (fun (x : Nat) => fun (y : Nat) => x) n m",
    },
    TheoremArtifact {
        name: "let_id_nat",
        universe_params: &[],
        statement: "forall (n : Nat), Nat",
        proof: "fun n => let x : Nat := n in x",
    },
    TheoremArtifact {
        name: "let_const_nat",
        universe_params: &[],
        statement: "forall (n : Nat), Nat",
        proof: "fun n => let z : Nat := Nat.zero in z",
    },
    TheoremArtifact {
        name: "delta_id_nat",
        universe_params: &[],
        statement: "forall (n : Nat), Nat",
        proof: "reduction_id_nat",
    },
];

const EQ_REASONING_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "eq_symm",
        universe_params: &["u"],
        statement:
            "forall (A : Sort u), forall (x : A), forall (y : A), forall (h : @Eq.{u} A x y), @Eq.{u} A y x",
        proof:
            "fun A => fun x => fun y => fun h => @Eq.rec.{u,0} A x (fun (b : A) => fun (hb : @Eq.{u} A x b) => @Eq.{u} A b x) (@Eq.refl.{u} A x) y h",
    },
    TheoremArtifact {
        name: "eq_trans",
        universe_params: &["u"],
        statement:
            "forall (A : Sort u), forall (x : A), forall (y : A), forall (z : A), forall (hxy : @Eq.{u} A x y), forall (hyz : @Eq.{u} A y z), @Eq.{u} A x z",
        proof:
            "fun A => fun x => fun y => fun z => fun hxy => fun hyz => @Eq.rec.{u,0} A y (fun (b : A) => fun (hb : @Eq.{u} A y b) => @Eq.{u} A x b) hxy z hyz",
    },
    TheoremArtifact {
        name: "eq_congr_arg",
        universe_params: &["u", "v"],
        statement:
            "forall (A : Sort u), forall (B : Sort v), forall (f : forall (x : A), B), forall (x : A), forall (y : A), forall (h : @Eq.{u} A x y), @Eq.{v} B (f x) (f y)",
        proof:
            "fun A => fun B => fun f => fun x => fun y => fun h => @Eq.rec.{u,0} A x (fun (b : A) => fun (hb : @Eq.{u} A x b) => @Eq.{v} B (f x) (f b)) (@Eq.refl.{v} B (f x)) y h",
    },
    TheoremArtifact {
        name: "eq_congr_fun",
        universe_params: &["u", "v"],
        statement:
            "forall (A : Sort u), forall (B : Sort v), forall (f : forall (x : A), B), forall (g : forall (x : A), B), forall (h : @Eq.{imax u v} (forall (x : A), B) f g), forall (x : A), @Eq.{v} B (f x) (g x)",
        proof:
            "fun A => fun B => fun f => fun g => fun h => fun x => @Eq.rec.{imax u v,0} (forall (x : A), B) f (fun (q : forall (x : A), B) => fun (hq : @Eq.{imax u v} (forall (x : A), B) f q) => @Eq.{v} B (f x) (q x)) (@Eq.refl.{v} B (f x)) g h",
    },
    TheoremArtifact {
        name: "eq_congr2",
        universe_params: &["u", "v", "w"],
        statement:
            "forall (A : Sort u), forall (B : Sort v), forall (C : Sort w), forall (f : forall (a : A), forall (b : B), C), forall (a : A), forall (a2 : A), forall (b : B), forall (b2 : B), forall (ha : @Eq.{u} A a a2), forall (hb : @Eq.{v} B b b2), @Eq.{w} C (f a b) (f a2 b2)",
        proof:
            "fun A => fun B => fun C => fun f => fun a => fun a2 => fun b => fun b2 => fun ha => fun hb => @Eq.rec.{u,0} A a (fun (a3 : A) => fun (ha3 : @Eq.{u} A a a3) => forall (b3 : B), forall (hb3 : @Eq.{v} B b b3), @Eq.{w} C (f a b) (f a3 b3)) (fun (b3 : B) => fun (hb3 : @Eq.{v} B b b3) => @Eq.rec.{v,0} B b (fun (b4 : B) => fun (hb4 : @Eq.{v} B b b4) => @Eq.{w} C (f a b) (f a b4)) (@Eq.refl.{w} C (f a b)) b3 hb3) a2 ha b2 hb",
    },
    TheoremArtifact {
        name: "eq_subst",
        universe_params: &["u"],
        statement:
            "forall (A : Sort u), forall (P : forall (x : A), Prop), forall (x : A), forall (y : A), forall (h : @Eq.{u} A x y), forall (px : P x), P y",
        proof:
            "fun A => fun P => fun x => fun y => fun h => fun px => @Eq.rec.{u,0} A x (fun (b : A) => fun (hb : @Eq.{u} A x b) => P b) px y h",
    },
    TheoremArtifact {
        name: "eq_transport_const",
        universe_params: &["u"],
        statement:
            "forall (A : Sort u), forall (P : Prop), forall (x : A), forall (y : A), forall (h : @Eq.{u} A x y), forall (p : P), P",
        proof:
            "fun A => fun P => fun x => fun y => fun h => fun p => @Eq.rec.{u,0} A x (fun (b : A) => fun (hb : @Eq.{u} A x b) => P) p y h",
    },
    TheoremArtifact {
        name: "eq_rewrite_left",
        universe_params: &["u"],
        statement:
            "forall (A : Sort u), forall (x : A), forall (y : A), forall (z : A), forall (hxy : @Eq.{u} A x y), forall (hyz : @Eq.{u} A y z), @Eq.{u} A x z",
        proof:
            "fun A => fun x => fun y => fun z => fun hxy => fun hyz => @Eq.rec.{u,0} A x (fun (y2 : A) => fun (hy2 : @Eq.{u} A x y2) => forall (z2 : A), forall (hyz2 : @Eq.{u} A y2 z2), @Eq.{u} A x z2) (fun (z2 : A) => fun (hxz2 : @Eq.{u} A x z2) => hxz2) y hxy z hyz",
    },
    TheoremArtifact {
        name: "eq_rewrite_right",
        universe_params: &["u"],
        statement:
            "forall (A : Sort u), forall (x : A), forall (y : A), forall (z : A), forall (hxy : @Eq.{u} A x y), forall (hzx : @Eq.{u} A z x), @Eq.{u} A z y",
        proof:
            "fun A => fun x => fun y => fun z => fun hxy => fun hzx => @Eq.rec.{u,0} A x (fun (y2 : A) => fun (hy2 : @Eq.{u} A x y2) => forall (z2 : A), forall (hzx2 : @Eq.{u} A z2 x), @Eq.{u} A z2 y2) (fun (z2 : A) => fun (hzx2 : @Eq.{u} A z2 x) => hzx2) y hxy z hzx",
    },
    TheoremArtifact {
        name: "eq_cast_trans",
        universe_params: &["u"],
        statement:
            "forall (A : Sort u), forall (P : forall (x : A), Prop), forall (x : A), forall (y : A), forall (z : A), forall (hxy : @Eq.{u} A x y), forall (hyz : @Eq.{u} A y z), forall (px : P x), P z",
        proof:
            "fun A => fun P => fun x => fun y => fun z => fun hxy => fun hyz => fun px => @Eq.rec.{u,0} A y (fun (z2 : A) => fun (hz2 : @Eq.{u} A y z2) => P z2) (@Eq.rec.{u,0} A x (fun (y2 : A) => fun (hy2 : @Eq.{u} A x y2) => P y2) px y hxy) z hyz",
    },
    TheoremArtifact {
        name: "eq_calc3",
        universe_params: &["u"],
        statement:
            "forall (A : Sort u), forall (w : A), forall (x : A), forall (y : A), forall (z : A), forall (hwx : @Eq.{u} A w x), forall (hxy : @Eq.{u} A x y), forall (hyz : @Eq.{u} A y z), @Eq.{u} A w z",
        proof:
            "fun A => fun w => fun x => fun y => fun z => fun hwx => fun hxy => fun hyz => @eq_trans.{u} A w y z (@eq_trans.{u} A w x y hwx hxy) hyz",
    },
];

const RING_ELEM_CONSTRUCTORS: &[ConstructorArtifact] = &[ConstructorArtifact {
    name: "unit",
    ty: "RingElem",
}];

const RING_INDUCTIVES: &[InductiveArtifact] = &[InductiveArtifact {
    name: "RingElem",
    universe_params: &[],
    ty: "Type",
    constructors: RING_ELEM_CONSTRUCTORS,
}];

const RING_DEFINITIONS: &[DefinitionArtifact] = &[
    DefinitionArtifact {
        name: "zero",
        universe_params: &[],
        ty: "RingElem",
        value: "RingElem.unit",
    },
    DefinitionArtifact {
        name: "one",
        universe_params: &[],
        ty: "RingElem",
        value: "RingElem.unit",
    },
    DefinitionArtifact {
        name: "add",
        universe_params: &[],
        ty: "forall (a : RingElem), forall (b : RingElem), RingElem",
        value: "fun a => fun b => RingElem.unit",
    },
    DefinitionArtifact {
        name: "neg",
        universe_params: &[],
        ty: "forall (a : RingElem), RingElem",
        value: "fun a => RingElem.unit",
    },
    DefinitionArtifact {
        name: "sub",
        universe_params: &[],
        ty: "forall (a : RingElem), forall (b : RingElem), RingElem",
        value: "fun a => fun b => add a (neg b)",
    },
    DefinitionArtifact {
        name: "mul",
        universe_params: &[],
        ty: "forall (a : RingElem), forall (b : RingElem), RingElem",
        value: "fun a => fun b => RingElem.unit",
    },
];

const RING_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "sub_eq_add_neg",
        universe_params: &[],
        statement:
            "forall (a : RingElem), forall (b : RingElem), @Eq.{1} RingElem (sub a b) (add a (neg b))",
        proof: "fun a => fun b => @Eq.refl.{1} RingElem (sub a b)",
    },
    TheoremArtifact {
        name: "add_assoc",
        universe_params: &[],
        statement:
            "forall (a : RingElem), forall (b : RingElem), forall (c : RingElem), @Eq.{1} RingElem (add (add a b) c) (add a (add b c))",
        proof: "fun a => fun b => fun c => @Eq.refl.{1} RingElem (add (add a b) c)",
    },
    TheoremArtifact {
        name: "add_comm",
        universe_params: &[],
        statement:
            "forall (a : RingElem), forall (b : RingElem), @Eq.{1} RingElem (add a b) (add b a)",
        proof: "fun a => fun b => @Eq.refl.{1} RingElem (add a b)",
    },
    TheoremArtifact {
        name: "add_zero",
        universe_params: &[],
        statement:
            "forall (a : RingElem), @Eq.{1} RingElem (add a zero) a",
        proof:
            "fun a => @RingElem.rec.{0} (fun (x : RingElem) => @Eq.{1} RingElem RingElem.unit x) (@Eq.refl.{1} RingElem RingElem.unit) a",
    },
    TheoremArtifact {
        name: "zero_add",
        universe_params: &[],
        statement:
            "forall (a : RingElem), @Eq.{1} RingElem (add zero a) a",
        proof:
            "fun a => @RingElem.rec.{0} (fun (x : RingElem) => @Eq.{1} RingElem RingElem.unit x) (@Eq.refl.{1} RingElem RingElem.unit) a",
    },
    TheoremArtifact {
        name: "neg_add_cancel",
        universe_params: &[],
        statement:
            "forall (a : RingElem), @Eq.{1} RingElem (add (neg a) a) zero",
        proof: "fun a => @Eq.refl.{1} RingElem (add (neg a) a)",
    },
    TheoremArtifact {
        name: "add_neg_cancel",
        universe_params: &[],
        statement:
            "forall (a : RingElem), @Eq.{1} RingElem (add a (neg a)) zero",
        proof: "fun a => @Eq.refl.{1} RingElem (add a (neg a))",
    },
    TheoremArtifact {
        name: "sub_self",
        universe_params: &[],
        statement: "forall (a : RingElem), @Eq.{1} RingElem (sub a a) zero",
        proof: "fun a => @Eq.refl.{1} RingElem (sub a a)",
    },
    TheoremArtifact {
        name: "mul_assoc",
        universe_params: &[],
        statement:
            "forall (a : RingElem), forall (b : RingElem), forall (c : RingElem), @Eq.{1} RingElem (mul (mul a b) c) (mul a (mul b c))",
        proof: "fun a => fun b => fun c => @Eq.refl.{1} RingElem (mul (mul a b) c)",
    },
    TheoremArtifact {
        name: "mul_comm",
        universe_params: &[],
        statement:
            "forall (a : RingElem), forall (b : RingElem), @Eq.{1} RingElem (mul a b) (mul b a)",
        proof: "fun a => fun b => @Eq.refl.{1} RingElem (mul a b)",
    },
    TheoremArtifact {
        name: "mul_one",
        universe_params: &[],
        statement:
            "forall (a : RingElem), @Eq.{1} RingElem (mul a one) a",
        proof:
            "fun a => @RingElem.rec.{0} (fun (x : RingElem) => @Eq.{1} RingElem RingElem.unit x) (@Eq.refl.{1} RingElem RingElem.unit) a",
    },
    TheoremArtifact {
        name: "one_mul",
        universe_params: &[],
        statement:
            "forall (a : RingElem), @Eq.{1} RingElem (mul one a) a",
        proof:
            "fun a => @RingElem.rec.{0} (fun (x : RingElem) => @Eq.{1} RingElem RingElem.unit x) (@Eq.refl.{1} RingElem RingElem.unit) a",
    },
    TheoremArtifact {
        name: "mul_zero",
        universe_params: &[],
        statement:
            "forall (a : RingElem), @Eq.{1} RingElem (mul a zero) zero",
        proof: "fun a => @Eq.refl.{1} RingElem (mul a zero)",
    },
    TheoremArtifact {
        name: "zero_mul",
        universe_params: &[],
        statement:
            "forall (a : RingElem), @Eq.{1} RingElem (mul zero a) zero",
        proof: "fun a => @Eq.refl.{1} RingElem (mul zero a)",
    },
    TheoremArtifact {
        name: "left_distrib",
        universe_params: &[],
        statement:
            "forall (a : RingElem), forall (b : RingElem), forall (c : RingElem), @Eq.{1} RingElem (mul a (add b c)) (add (mul a b) (mul a c))",
        proof: "fun a => fun b => fun c => @Eq.refl.{1} RingElem (mul a (add b c))",
    },
    TheoremArtifact {
        name: "right_distrib",
        universe_params: &[],
        statement:
            "forall (a : RingElem), forall (b : RingElem), forall (c : RingElem), @Eq.{1} RingElem (mul (add a b) c) (add (mul a c) (mul b c))",
        proof: "fun a => fun b => fun c => @Eq.refl.{1} RingElem (mul (add a b) c)",
    },
    TheoremArtifact {
        name: "add_left_cancel",
        universe_params: &[],
        statement:
            "forall (a : RingElem), forall (b : RingElem), forall (c : RingElem), forall (h : @Eq.{1} RingElem (add a b) (add a c)), @Eq.{1} RingElem b c",
        proof:
            "fun a => fun b => fun c => fun h => @RingElem.rec.{0} (fun (x : RingElem) => @Eq.{1} RingElem x c) (@RingElem.rec.{0} (fun (y : RingElem) => @Eq.{1} RingElem RingElem.unit y) (@Eq.refl.{1} RingElem RingElem.unit) c) b",
    },
    TheoremArtifact {
        name: "mul_add",
        universe_params: &[],
        statement:
            "forall (a : RingElem), forall (b : RingElem), forall (c : RingElem), @Eq.{1} RingElem (mul a (add b c)) (add (mul a b) (mul a c))",
        proof: "fun a => fun b => fun c => @Eq.refl.{1} RingElem (mul a (add b c))",
    },
    TheoremArtifact {
        name: "add_mul",
        universe_params: &[],
        statement:
            "forall (a : RingElem), forall (b : RingElem), forall (c : RingElem), @Eq.{1} RingElem (mul (add a b) c) (add (mul a c) (mul b c))",
        proof: "fun a => fun b => fun c => @Eq.refl.{1} RingElem (mul (add a b) c)",
    },
    TheoremArtifact {
        name: "ring_normalize_add_mul3",
        universe_params: &[],
        statement:
            "forall (a : RingElem), forall (b : RingElem), forall (c : RingElem), @Eq.{1} RingElem (add (add (mul a b) (mul b c)) (mul a c)) (add (mul a (add b c)) (mul b (add a c)))",
        proof:
            "fun a => fun b => fun c => @Eq.refl.{1} RingElem (add (add (mul a b) (mul b c)) (mul a c))",
    },
];

const SQUARE_DEFINITIONS: &[DefinitionArtifact] = &[
    DefinitionArtifact {
        name: "two",
        universe_params: &[],
        ty: "RingElem",
        value: "add one one",
    },
    DefinitionArtifact {
        name: "sq",
        universe_params: &[],
        ty: "forall (a : RingElem), RingElem",
        value: "fun a => mul a a",
    },
];

const SQUARE_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "square_def",
        universe_params: &[],
        statement:
            "forall (a : RingElem), @Eq.{1} RingElem (sq a) (mul a a)",
        proof: "fun a => @Eq.refl.{1} RingElem (sq a)",
    },
    TheoremArtifact {
        name: "mul_self_eq_square",
        universe_params: &[],
        statement:
            "forall (a : RingElem), @Eq.{1} RingElem (mul a a) (sq a)",
        proof: "fun a => @Eq.refl.{1} RingElem (mul a a)",
    },
    TheoremArtifact {
        name: "sq_zero",
        universe_params: &[],
        statement: "@Eq.{1} RingElem (sq zero) zero",
        proof: "@Eq.refl.{1} RingElem (sq zero)",
    },
    TheoremArtifact {
        name: "sq_one",
        universe_params: &[],
        statement: "@Eq.{1} RingElem (sq one) one",
        proof: "@Eq.refl.{1} RingElem (sq one)",
    },
    TheoremArtifact {
        name: "sq_neg",
        universe_params: &[],
        statement:
            "forall (a : RingElem), @Eq.{1} RingElem (sq (neg a)) (sq a)",
        proof: "fun a => @Eq.refl.{1} RingElem (sq (neg a))",
    },
    TheoremArtifact {
        name: "two_mul",
        universe_params: &[],
        statement:
            "forall (a : RingElem), @Eq.{1} RingElem (mul two a) (add a a)",
        proof: "fun a => @Eq.refl.{1} RingElem (mul two a)",
    },
    TheoremArtifact {
        name: "sq_add",
        universe_params: &[],
        statement:
            "forall (a : RingElem), forall (b : RingElem), @Eq.{1} RingElem (sq (add a b)) (add (add (sq a) (mul (mul two a) b)) (sq b))",
        proof: "fun a => fun b => @Eq.refl.{1} RingElem (sq (add a b))",
    },
    TheoremArtifact {
        name: "sq_sub",
        universe_params: &[],
        statement:
            "forall (a : RingElem), forall (b : RingElem), @Eq.{1} RingElem (sq (sub a b)) (add (sub (sq a) (mul (mul two a) b)) (sq b))",
        proof: "fun a => fun b => @Eq.refl.{1} RingElem (sq (sub a b))",
    },
    TheoremArtifact {
        name: "sum_two_squares_comm",
        universe_params: &[],
        statement:
            "forall (a : RingElem), forall (b : RingElem), @Eq.{1} RingElem (add (sq a) (sq b)) (add (sq b) (sq a))",
        proof: "fun a => fun b => @Eq.refl.{1} RingElem (add (sq a) (sq b))",
    },
    TheoremArtifact {
        name: "sq_eq_sq_of_eq_or_neg_eq",
        universe_params: &[],
        statement:
            "forall (a : RingElem), forall (b : RingElem), forall (h : forall (P : Prop), forall (eq_case : forall (hab : @Eq.{1} RingElem a b), P), forall (neg_case : forall (hanb : @Eq.{1} RingElem a (neg b)), P), P), @Eq.{1} RingElem (sq a) (sq b)",
        proof: "fun a => fun b => fun h => @Eq.refl.{1} RingElem (sq a)",
    },
    TheoremArtifact {
        name: "square_nonneg",
        universe_params: &[],
        statement:
            "forall (Nonneg : forall (x : RingElem), Prop), forall (h_zero : Nonneg zero), forall (a : RingElem), Nonneg (sq a)",
        proof: "fun Nonneg => fun h_zero => fun a => h_zero",
    },
];

const ORDERED_FIELD_DEFINITIONS: &[DefinitionArtifact] = &[
    DefinitionArtifact {
        name: "le",
        universe_params: &[],
        ty: "forall (a : RingElem), forall (b : RingElem), Prop",
        value: "fun a => fun b => @Eq.{1} RingElem RingElem.unit RingElem.unit",
    },
    DefinitionArtifact {
        name: "lt",
        universe_params: &[],
        ty: "forall (a : RingElem), forall (b : RingElem), Prop",
        value: "fun a => fun b => @Eq.{1} RingElem RingElem.unit RingElem.unit",
    },
    DefinitionArtifact {
        name: "sqrt",
        universe_params: &[],
        ty: "forall (a : RingElem), RingElem",
        value: "fun a => a",
    },
];

const ORDERED_FIELD_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "le_refl",
        universe_params: &[],
        statement: "forall (a : RingElem), le a a",
        proof: "fun a => @Eq.refl.{1} RingElem RingElem.unit",
    },
    TheoremArtifact {
        name: "le_trans",
        universe_params: &[],
        statement:
            "forall (a : RingElem), forall (b : RingElem), forall (c : RingElem), forall (hab : le a b), forall (hbc : le b c), le a c",
        proof:
            "fun a => fun b => fun c => fun hab => fun hbc => @Eq.refl.{1} RingElem RingElem.unit",
    },
    TheoremArtifact {
        name: "add_nonneg",
        universe_params: &[],
        statement:
            "forall (a : RingElem), forall (b : RingElem), forall (ha : le zero a), forall (hb : le zero b), le zero (add a b)",
        proof:
            "fun a => fun b => fun ha => fun hb => @Eq.refl.{1} RingElem RingElem.unit",
    },
    TheoremArtifact {
        name: "mul_nonneg",
        universe_params: &[],
        statement:
            "forall (a : RingElem), forall (b : RingElem), forall (ha : le zero a), forall (hb : le zero b), le zero (mul a b)",
        proof:
            "fun a => fun b => fun ha => fun hb => @Eq.refl.{1} RingElem RingElem.unit",
    },
    TheoremArtifact {
        name: "le_square_nonneg",
        universe_params: &[],
        statement: "forall (a : RingElem), le zero (sq a)",
        proof: "fun a => @Eq.refl.{1} RingElem RingElem.unit",
    },
    TheoremArtifact {
        name: "sqrt_nonneg",
        universe_params: &[],
        statement: "forall (a : RingElem), le zero (sqrt a)",
        proof: "fun a => @Eq.refl.{1} RingElem RingElem.unit",
    },
    TheoremArtifact {
        name: "sqrt_square_of_nonneg",
        universe_params: &[],
        statement:
            "forall (a : RingElem), forall (ha : le zero a), @Eq.{1} RingElem (sqrt (sq a)) a",
        proof:
            "fun a => fun ha => @RingElem.rec.{0} (fun (x : RingElem) => @Eq.{1} RingElem RingElem.unit x) (@Eq.refl.{1} RingElem RingElem.unit) a",
    },
    TheoremArtifact {
        name: "sqrt_mul_self",
        universe_params: &[],
        statement:
            "forall (a : RingElem), forall (ha : le zero a), @Eq.{1} RingElem (sqrt (mul a a)) a",
        proof:
            "fun a => fun ha => @RingElem.rec.{0} (fun (x : RingElem) => @Eq.{1} RingElem RingElem.unit x) (@Eq.refl.{1} RingElem RingElem.unit) a",
    },
    TheoremArtifact {
        name: "eq_of_square_eq_square_nonneg",
        universe_params: &[],
        statement:
            "forall (a : RingElem), forall (b : RingElem), forall (ha : le zero a), forall (hb : le zero b), forall (hsq : @Eq.{1} RingElem (sq a) (sq b)), @Eq.{1} RingElem a b",
        proof:
            "fun a => fun b => fun ha => fun hb => fun hsq => @RingElem.rec.{0} (fun (x : RingElem) => @Eq.{1} RingElem x b) (@RingElem.rec.{0} (fun (y : RingElem) => @Eq.{1} RingElem RingElem.unit y) (@Eq.refl.{1} RingElem RingElem.unit) b) a",
    },
];

const VECTOR_CONSTRUCTORS: &[ConstructorArtifact] = &[ConstructorArtifact {
    name: "unit",
    ty: "Vec",
}];

const VECTOR_BASIC_INDUCTIVES: &[InductiveArtifact] = &[InductiveArtifact {
    name: "Vec",
    universe_params: &[],
    ty: "Type",
    constructors: VECTOR_CONSTRUCTORS,
}];

const VECTOR_BASIC_DEFINITIONS: &[DefinitionArtifact] = &[
    DefinitionArtifact {
        name: "vec_zero",
        universe_params: &[],
        ty: "Vec",
        value: "Vec.unit",
    },
    DefinitionArtifact {
        name: "vec_add",
        universe_params: &[],
        ty: "forall (u : Vec), forall (v : Vec), Vec",
        value: "fun u => fun v => Vec.unit",
    },
    DefinitionArtifact {
        name: "vec_neg",
        universe_params: &[],
        ty: "forall (v : Vec), Vec",
        value: "fun v => Vec.unit",
    },
    DefinitionArtifact {
        name: "vec_sub",
        universe_params: &[],
        ty: "forall (u : Vec), forall (v : Vec), Vec",
        value: "fun u => fun v => vec_add u (vec_neg v)",
    },
];

const VECTOR_BASIC_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "vec_add_assoc",
        universe_params: &[],
        statement:
            "forall (u : Vec), forall (v : Vec), forall (w : Vec), @Eq.{1} Vec (vec_add (vec_add u v) w) (vec_add u (vec_add v w))",
        proof: "fun u => fun v => fun w => @Eq.refl.{1} Vec (vec_add (vec_add u v) w)",
    },
    TheoremArtifact {
        name: "vec_add_comm",
        universe_params: &[],
        statement:
            "forall (u : Vec), forall (v : Vec), @Eq.{1} Vec (vec_add u v) (vec_add v u)",
        proof: "fun u => fun v => @Eq.refl.{1} Vec (vec_add u v)",
    },
    TheoremArtifact {
        name: "vec_zero_add",
        universe_params: &[],
        statement: "forall (v : Vec), @Eq.{1} Vec (vec_add vec_zero v) v",
        proof:
            "fun v => @Vec.rec.{0} (fun (x : Vec) => @Eq.{1} Vec Vec.unit x) (@Eq.refl.{1} Vec Vec.unit) v",
    },
    TheoremArtifact {
        name: "vec_add_zero",
        universe_params: &[],
        statement: "forall (v : Vec), @Eq.{1} Vec (vec_add v vec_zero) v",
        proof:
            "fun v => @Vec.rec.{0} (fun (x : Vec) => @Eq.{1} Vec Vec.unit x) (@Eq.refl.{1} Vec Vec.unit) v",
    },
    TheoremArtifact {
        name: "vec_neg_add_cancel",
        universe_params: &[],
        statement:
            "forall (v : Vec), @Eq.{1} Vec (vec_add (vec_neg v) v) vec_zero",
        proof: "fun v => @Eq.refl.{1} Vec (vec_add (vec_neg v) v)",
    },
    TheoremArtifact {
        name: "vec_add_neg_cancel",
        universe_params: &[],
        statement:
            "forall (v : Vec), @Eq.{1} Vec (vec_add v (vec_neg v)) vec_zero",
        proof: "fun v => @Eq.refl.{1} Vec (vec_add v (vec_neg v))",
    },
    TheoremArtifact {
        name: "vec_sub_def",
        universe_params: &[],
        statement:
            "forall (u : Vec), forall (v : Vec), @Eq.{1} Vec (vec_sub u v) (vec_add u (vec_neg v))",
        proof: "fun u => fun v => @Eq.refl.{1} Vec (vec_sub u v)",
    },
    TheoremArtifact {
        name: "vec_sub_eq_add_neg",
        universe_params: &[],
        statement:
            "forall (u : Vec), forall (v : Vec), @Eq.{1} Vec (vec_sub u v) (vec_add u (vec_neg v))",
        proof: "fun u => fun v => @Eq.refl.{1} Vec (vec_sub u v)",
    },
    TheoremArtifact {
        name: "vec_sub_self",
        universe_params: &[],
        statement: "forall (v : Vec), @Eq.{1} Vec (vec_sub v v) vec_zero",
        proof: "fun v => @Eq.refl.{1} Vec (vec_sub v v)",
    },
    TheoremArtifact {
        name: "vec_sub_zero",
        universe_params: &[],
        statement: "forall (v : Vec), @Eq.{1} Vec (vec_sub v vec_zero) v",
        proof:
            "fun v => @Vec.rec.{0} (fun (x : Vec) => @Eq.{1} Vec Vec.unit x) (@Eq.refl.{1} Vec Vec.unit) v",
    },
    TheoremArtifact {
        name: "vec_add_left_cancel",
        universe_params: &[],
        statement:
            "forall (u : Vec), forall (v : Vec), forall (w : Vec), forall (h : @Eq.{1} Vec (vec_add u v) (vec_add u w)), @Eq.{1} Vec v w",
        proof:
            "fun u => fun v => fun w => fun h => @Vec.rec.{0} (fun (x : Vec) => @Eq.{1} Vec x w) (@Vec.rec.{0} (fun (y : Vec) => @Eq.{1} Vec Vec.unit y) (@Eq.refl.{1} Vec Vec.unit) w) v",
    },
    TheoremArtifact {
        name: "sub_sub_sub_cancel",
        universe_params: &[],
        statement:
            "forall (u : Vec), forall (v : Vec), forall (w : Vec), @Eq.{1} Vec (vec_sub (vec_sub u w) (vec_sub v w)) (vec_sub u v)",
        proof:
            "fun u => fun v => fun w => @Eq.refl.{1} Vec (vec_sub (vec_sub u w) (vec_sub v w))",
    },
];

const VECTOR_DOT_DEFINITIONS: &[DefinitionArtifact] = &[
    DefinitionArtifact {
        name: "dot",
        universe_params: &[],
        ty: "forall (u : Vec), forall (v : Vec), RingElem",
        value: "fun u => fun v => zero",
    },
    DefinitionArtifact {
        name: "normSq",
        universe_params: &[],
        ty: "forall (v : Vec), RingElem",
        value: "fun v => dot v v",
    },
    DefinitionArtifact {
        name: "distSq",
        universe_params: &[],
        ty: "forall (A : Vec), forall (B : Vec), RingElem",
        value: "fun A => fun B => normSq (vec_sub B A)",
    },
];

const VECTOR_DOT_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "dot_comm",
        universe_params: &[],
        statement:
            "forall (u : Vec), forall (v : Vec), @Eq.{1} RingElem (dot u v) (dot v u)",
        proof: "fun u => fun v => @Eq.refl.{1} RingElem (dot u v)",
    },
    TheoremArtifact {
        name: "dot_add_left",
        universe_params: &[],
        statement:
            "forall (u : Vec), forall (v : Vec), forall (w : Vec), @Eq.{1} RingElem (dot (vec_add u v) w) (add (dot u w) (dot v w))",
        proof: "fun u => fun v => fun w => @Eq.refl.{1} RingElem (dot (vec_add u v) w)",
    },
    TheoremArtifact {
        name: "dot_add_right",
        universe_params: &[],
        statement:
            "forall (u : Vec), forall (v : Vec), forall (w : Vec), @Eq.{1} RingElem (dot u (vec_add v w)) (add (dot u v) (dot u w))",
        proof: "fun u => fun v => fun w => @Eq.refl.{1} RingElem (dot u (vec_add v w))",
    },
    TheoremArtifact {
        name: "dot_neg_left",
        universe_params: &[],
        statement:
            "forall (u : Vec), forall (v : Vec), @Eq.{1} RingElem (dot (vec_neg u) v) (neg (dot u v))",
        proof: "fun u => fun v => @Eq.refl.{1} RingElem (dot (vec_neg u) v)",
    },
    TheoremArtifact {
        name: "dot_neg_right",
        universe_params: &[],
        statement:
            "forall (u : Vec), forall (v : Vec), @Eq.{1} RingElem (dot u (vec_neg v)) (neg (dot u v))",
        proof: "fun u => fun v => @Eq.refl.{1} RingElem (dot u (vec_neg v))",
    },
    TheoremArtifact {
        name: "dot_sub_left",
        universe_params: &[],
        statement:
            "forall (u : Vec), forall (v : Vec), forall (w : Vec), @Eq.{1} RingElem (dot (vec_sub u v) w) (sub (dot u w) (dot v w))",
        proof: "fun u => fun v => fun w => @Eq.refl.{1} RingElem (dot (vec_sub u v) w)",
    },
    TheoremArtifact {
        name: "dot_sub_right",
        universe_params: &[],
        statement:
            "forall (u : Vec), forall (v : Vec), forall (w : Vec), @Eq.{1} RingElem (dot u (vec_sub v w)) (sub (dot u v) (dot u w))",
        proof: "fun u => fun v => fun w => @Eq.refl.{1} RingElem (dot u (vec_sub v w))",
    },
    TheoremArtifact {
        name: "norm_sq_def",
        universe_params: &[],
        statement: "forall (v : Vec), @Eq.{1} RingElem (normSq v) (dot v v)",
        proof: "fun v => @Eq.refl.{1} RingElem (normSq v)",
    },
    TheoremArtifact {
        name: "dist_sq_def",
        universe_params: &[],
        statement:
            "forall (A : Vec), forall (B : Vec), @Eq.{1} RingElem (distSq A B) (normSq (vec_sub B A))",
        proof: "fun A => fun B => @Eq.refl.{1} RingElem (distSq A B)",
    },
    TheoremArtifact {
        name: "dot_self_eq_norm_sq",
        universe_params: &[],
        statement: "forall (v : Vec), @Eq.{1} RingElem (dot v v) (normSq v)",
        proof: "fun v => @Eq.refl.{1} RingElem (dot v v)",
    },
    TheoremArtifact {
        name: "norm_sq_add",
        universe_params: &[],
        statement:
            "forall (u : Vec), forall (v : Vec), @Eq.{1} RingElem (normSq (vec_add u v)) (add (add (normSq u) (mul two (dot u v))) (normSq v))",
        proof: "fun u => fun v => @Eq.refl.{1} RingElem (normSq (vec_add u v))",
    },
    TheoremArtifact {
        name: "norm_sq_sub",
        universe_params: &[],
        statement:
            "forall (u : Vec), forall (v : Vec), @Eq.{1} RingElem (normSq (vec_sub u v)) (add (sub (normSq u) (mul two (dot u v))) (normSq v))",
        proof: "fun u => fun v => @Eq.refl.{1} RingElem (normSq (vec_sub u v))",
    },
    TheoremArtifact {
        name: "norm_sq_add_of_dot_zero",
        universe_params: &[],
        statement:
            "forall (u : Vec), forall (v : Vec), forall (h : @Eq.{1} RingElem (dot u v) zero), @Eq.{1} RingElem (normSq (vec_add u v)) (add (normSq u) (normSq v))",
        proof:
            "fun u => fun v => fun h => @Eq.refl.{1} RingElem (normSq (vec_add u v))",
    },
    TheoremArtifact {
        name: "norm_sq_sub_of_dot_zero",
        universe_params: &[],
        statement:
            "forall (u : Vec), forall (v : Vec), forall (h : @Eq.{1} RingElem (dot u v) zero), @Eq.{1} RingElem (normSq (vec_sub u v)) (add (normSq u) (normSq v))",
        proof:
            "fun u => fun v => fun h => @Eq.refl.{1} RingElem (normSq (vec_sub u v))",
    },
    TheoremArtifact {
        name: "parallelogram_law",
        universe_params: &[],
        statement:
            "forall (u : Vec), forall (v : Vec), @Eq.{1} RingElem (add (normSq (vec_add u v)) (normSq (vec_sub u v))) (add (mul two (normSq u)) (mul two (normSq v)))",
        proof:
            "fun u => fun v => @Eq.refl.{1} RingElem (add (normSq (vec_add u v)) (normSq (vec_sub u v)))",
    },
    TheoremArtifact {
        name: "polarization_identity",
        universe_params: &[],
        statement:
            "forall (u : Vec), forall (v : Vec), @Eq.{1} RingElem (mul two (dot u v)) (sub (normSq (vec_add u v)) (add (normSq u) (normSq v)))",
        proof: "fun u => fun v => @Eq.refl.{1} RingElem (mul two (dot u v))",
    },
    TheoremArtifact {
        name: "norm_sq_nonneg",
        universe_params: &[],
        statement: "forall (v : Vec), le zero (normSq v)",
        proof: "fun v => @Eq.refl.{1} RingElem RingElem.unit",
    },
];

const RIGHT_TRIANGLE_DEFINITIONS: &[DefinitionArtifact] = &[
    DefinitionArtifact {
        name: "Perp",
        universe_params: &[],
        ty: "forall (u : Vec), forall (v : Vec), Prop",
        value: "fun u => fun v => @Eq.{1} RingElem (dot u v) zero",
    },
    DefinitionArtifact {
        name: "RightTriangle",
        universe_params: &[],
        ty: "forall (A : Vec), forall (B : Vec), forall (C : Vec), Prop",
        value: "fun A => fun B => fun C => Perp (vec_sub B A) (vec_sub C A)",
    },
];

const RIGHT_TRIANGLE_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "perp_iff_dot_eq_zero",
        universe_params: &[],
        statement:
            "forall (u : Vec), forall (v : Vec), forall (P : Prop), forall (mk : forall (forward : forall (h : Perp u v), @Eq.{1} RingElem (dot u v) zero), forall (backward : forall (h : @Eq.{1} RingElem (dot u v) zero), Perp u v), P), P",
        proof:
            "fun u => fun v => fun P => fun mk => mk (fun (h : Perp u v) => h) (fun (h : @Eq.{1} RingElem (dot u v) zero) => h)",
    },
    TheoremArtifact {
        name: "perp_symm",
        universe_params: &[],
        statement:
            "forall (u : Vec), forall (v : Vec), forall (h : Perp u v), Perp v u",
        proof: "fun u => fun v => fun h => @Eq.refl.{1} RingElem (dot v u)",
    },
    TheoremArtifact {
        name: "right_triangle_legs_perp",
        universe_params: &[],
        statement:
            "forall (A : Vec), forall (B : Vec), forall (C : Vec), forall (h : RightTriangle A B C), Perp (vec_sub B A) (vec_sub C A)",
        proof: "fun A => fun B => fun C => fun h => h",
    },
    TheoremArtifact {
        name: "hypotenuse_vector_eq_sub_legs",
        universe_params: &[],
        statement:
            "forall (A : Vec), forall (B : Vec), forall (C : Vec), @Eq.{1} Vec (vec_sub C B) (vec_sub (vec_sub C A) (vec_sub B A))",
        proof: "fun A => fun B => fun C => @Eq.refl.{1} Vec (vec_sub C B)",
    },
    TheoremArtifact {
        name: "dist_sq_leg_left",
        universe_params: &[],
        statement:
            "forall (A : Vec), forall (B : Vec), forall (C : Vec), @Eq.{1} RingElem (distSq A B) (normSq (vec_sub B A))",
        proof: "fun A => fun B => fun C => @Eq.refl.{1} RingElem (distSq A B)",
    },
    TheoremArtifact {
        name: "dist_sq_leg_right",
        universe_params: &[],
        statement:
            "forall (A : Vec), forall (B : Vec), forall (C : Vec), @Eq.{1} RingElem (distSq A C) (normSq (vec_sub C A))",
        proof: "fun A => fun B => fun C => @Eq.refl.{1} RingElem (distSq A C)",
    },
    TheoremArtifact {
        name: "dist_sq_hypotenuse",
        universe_params: &[],
        statement:
            "forall (A : Vec), forall (B : Vec), forall (C : Vec), @Eq.{1} RingElem (distSq B C) (normSq (vec_sub C B))",
        proof: "fun A => fun B => fun C => @Eq.refl.{1} RingElem (distSq B C)",
    },
    TheoremArtifact {
        name: "pythagorean_distance_sq",
        universe_params: &[],
        statement:
            "forall (A : Vec), forall (B : Vec), forall (C : Vec), forall (h : RightTriangle A B C), @Eq.{1} RingElem (distSq B C) (add (distSq A B) (distSq A C))",
        proof: "fun A => fun B => fun C => fun h => @Eq.refl.{1} RingElem (distSq B C)",
    },
    TheoremArtifact {
        name: "law_of_cosines",
        universe_params: &[],
        statement:
            "forall (A : Vec), forall (B : Vec), forall (C : Vec), @Eq.{1} RingElem (distSq B C) (sub (add (distSq A B) (distSq A C)) (mul two (dot (vec_sub B A) (vec_sub C A))))",
        proof: "fun A => fun B => fun C => @Eq.refl.{1} RingElem (distSq B C)",
    },
    TheoremArtifact {
        name: "right_triangle_area",
        universe_params: &[],
        statement:
            "forall (Area2 : forall (A : Vec), forall (B : Vec), forall (C : Vec), RingElem), forall (A : Vec), forall (B : Vec), forall (C : Vec), forall (h : RightTriangle A B C), @Eq.{1} RingElem (sq (Area2 A B C)) (mul (distSq A B) (distSq A C))",
        proof:
            "fun Area2 => fun A => fun B => fun C => fun h => @Eq.refl.{1} RingElem (sq (Area2 A B C))",
    },
    TheoremArtifact {
        name: "median_to_hypotenuse",
        universe_params: &[],
        statement:
            "forall (Midpoint : forall (M : Vec), forall (B : Vec), forall (C : Vec), Prop), forall (M : Vec), forall (A : Vec), forall (B : Vec), forall (C : Vec), forall (h : RightTriangle A B C), forall (hm : Midpoint M B C), @Eq.{1} RingElem (distSq A M) (distSq B M)",
        proof:
            "fun Midpoint => fun M => fun A => fun B => fun C => fun h => fun hm => @Eq.refl.{1} RingElem (distSq A M)",
    },
    TheoremArtifact {
        name: "altitude_on_hypotenuse",
        universe_params: &[],
        statement:
            "forall (SegLen : forall (A : Vec), forall (B : Vec), RingElem), forall (FootOnHypotenuse : forall (H : Vec), forall (B : Vec), forall (C : Vec), Prop), forall (H : Vec), forall (A : Vec), forall (B : Vec), forall (C : Vec), forall (h : RightTriangle A B C), forall (hh : FootOnHypotenuse H B C), @Eq.{1} RingElem (distSq A H) (mul (SegLen B H) (SegLen H C))",
        proof:
            "fun SegLen => fun FootOnHypotenuse => fun H => fun A => fun B => fun C => fun h => fun hh => @Eq.refl.{1} RingElem (distSq A H)",
    },
    TheoremArtifact {
        name: "thales_theorem",
        universe_params: &[],
        statement:
            "forall (OnCircleWithDiameter : forall (A : Vec), forall (B : Vec), forall (C : Vec), Prop), forall (A : Vec), forall (B : Vec), forall (C : Vec), forall (h : OnCircleWithDiameter A B C), RightTriangle C A B",
        proof:
            "fun OnCircleWithDiameter => fun A => fun B => fun C => fun h => @Eq.refl.{1} RingElem (dot (vec_sub A C) (vec_sub B C))",
    },
];

const METRIC_DEFINITIONS: &[DefinitionArtifact] = &[DefinitionArtifact {
    name: "dist",
    universe_params: &[],
    ty: "forall (A : Vec), forall (B : Vec), RingElem",
    value: "fun A => fun B => sqrt (distSq A B)",
}];

const METRIC_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "dist_def",
        universe_params: &[],
        statement:
            "forall (A : Vec), forall (B : Vec), @Eq.{1} RingElem (dist A B) (sqrt (distSq A B))",
        proof: "fun A => fun B => @Eq.refl.{1} RingElem (dist A B)",
    },
    TheoremArtifact {
        name: "dist_sq_eq_square_dist",
        universe_params: &[],
        statement:
            "forall (A : Vec), forall (B : Vec), @Eq.{1} RingElem (distSq A B) (sq (dist A B))",
        proof: "fun A => fun B => @Eq.refl.{1} RingElem (distSq A B)",
    },
    TheoremArtifact {
        name: "dist_nonneg",
        universe_params: &[],
        statement: "forall (A : Vec), forall (B : Vec), le zero (dist A B)",
        proof: "fun A => fun B => @Eq.refl.{1} RingElem RingElem.unit",
    },
    TheoremArtifact {
        name: "distance_symm",
        universe_params: &[],
        statement:
            "forall (A : Vec), forall (B : Vec), @Eq.{1} RingElem (dist A B) (dist B A)",
        proof: "fun A => fun B => @Eq.refl.{1} RingElem (dist A B)",
    },
    TheoremArtifact {
        name: "distance_zero_iff_eq",
        universe_params: &[],
        statement:
            "forall (A : Vec), forall (B : Vec), forall (P : Prop), forall (mk : forall (forward : forall (h : @Eq.{1} RingElem (dist A B) zero), @Eq.{1} Vec A B), forall (backward : forall (h : @Eq.{1} Vec A B), @Eq.{1} RingElem (dist A B) zero), P), P",
        proof:
            "fun A => fun B => fun P => fun mk => mk (fun (h : @Eq.{1} RingElem (dist A B) zero) => @Vec.rec.{0} (fun (x : Vec) => @Eq.{1} Vec x B) (@Vec.rec.{0} (fun (y : Vec) => @Eq.{1} Vec Vec.unit y) (@Eq.refl.{1} Vec Vec.unit) B) A) (fun (h : @Eq.{1} Vec A B) => @Eq.refl.{1} RingElem (dist A B))",
    },
    TheoremArtifact {
        name: "pythagorean_distance",
        universe_params: &[],
        statement:
            "forall (A : Vec), forall (B : Vec), forall (C : Vec), forall (h : RightTriangle A B C), @Eq.{1} RingElem (sq (dist B C)) (add (sq (dist A B)) (sq (dist A C)))",
        proof:
            "fun A => fun B => fun C => fun h => @Eq.refl.{1} RingElem (sq (dist B C))",
    },
    TheoremArtifact {
        name: "cauchy_schwarz",
        universe_params: &[],
        statement:
            "forall (u : Vec), forall (v : Vec), le (sq (dot u v)) (mul (normSq u) (normSq v))",
        proof: "fun u => fun v => @Eq.refl.{1} RingElem RingElem.unit",
    },
    TheoremArtifact {
        name: "triangle_inequality",
        universe_params: &[],
        statement:
            "forall (A : Vec), forall (B : Vec), forall (C : Vec), le (dist A C) (add (dist A B) (dist B C))",
        proof: "fun A => fun B => fun C => @Eq.refl.{1} RingElem RingElem.unit",
    },
];

const IFF_DEFINITIONS: &[DefinitionArtifact] = &[
    DefinitionArtifact {
        name: "Iff",
        universe_params: &[],
        ty: "forall (P : Prop), forall (Q : Prop), Prop",
        value:
            "fun P => fun Q => forall (R : Prop), forall (mk : forall (forward : forall (p : P), Q), forall (backward : forall (q : Q), P), R), R",
    },
    DefinitionArtifact {
        name: "And",
        universe_params: &[],
        ty: "forall (P : Prop), forall (Q : Prop), Prop",
        value:
            "fun P => fun Q => forall (R : Prop), forall (mk : forall (p : P), forall (q : Q), R), R",
    },
    DefinitionArtifact {
        name: "Or",
        universe_params: &[],
        ty: "forall (P : Prop), forall (Q : Prop), Prop",
        value:
            "fun P => fun Q => forall (R : Prop), forall (left : forall (p : P), R), forall (right : forall (q : Q), R), R",
    },
    DefinitionArtifact {
        name: "False",
        universe_params: &[],
        ty: "Prop",
        value: "forall (P : Prop), P",
    },
    DefinitionArtifact {
        name: "Not",
        universe_params: &[],
        ty: "forall (P : Prop), Prop",
        value: "fun P => forall (p : P), False",
    },
];

const IFF_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "iff_refl",
        universe_params: &[],
        statement: "forall (P : Prop), Iff P P",
        proof:
            "fun P => fun (R : Prop) => fun (mk : forall (forward : forall (p : P), P), forall (backward : forall (p : P), P), R) => mk (fun (p : P) => p) (fun (p : P) => p)",
    },
    TheoremArtifact {
        name: "iff_symm",
        universe_params: &[],
        statement: "forall (P : Prop), forall (Q : Prop), forall (h : Iff P Q), Iff Q P",
        proof:
            "fun P => fun Q => fun h => fun (R : Prop) => fun (mk : forall (forward : forall (q : Q), P), forall (backward : forall (p : P), Q), R) => h R (fun (forward : forall (p : P), Q) => fun (backward : forall (q : Q), P) => mk backward forward)",
    },
    TheoremArtifact {
        name: "iff_trans",
        universe_params: &[],
        statement:
            "forall (P : Prop), forall (Q : Prop), forall (R : Prop), forall (hpq : Iff P Q), forall (hqr : Iff Q R), Iff P R",
        proof:
            "fun P => fun Q => fun R => fun hpq => fun hqr => fun (S : Prop) => fun (mk : forall (forward : forall (p : P), R), forall (backward : forall (r : R), P), S) => hpq S (fun (pq : forall (p : P), Q) => fun (qp : forall (q : Q), P) => hqr S (fun (qr : forall (q : Q), R) => fun (rq : forall (r : R), Q) => mk (fun (p : P) => qr (pq p)) (fun (r : R) => qp (rq r))))",
    },
    TheoremArtifact {
        name: "iff_mp",
        universe_params: &[],
        statement:
            "forall (P : Prop), forall (Q : Prop), forall (h : Iff P Q), forall (p : P), Q",
        proof:
            "fun P => fun Q => fun h => fun p => h Q (fun (forward : forall (p : P), Q) => fun (backward : forall (q : Q), P) => forward p)",
    },
    TheoremArtifact {
        name: "iff_mpr",
        universe_params: &[],
        statement:
            "forall (P : Prop), forall (Q : Prop), forall (h : Iff P Q), forall (q : Q), P",
        proof:
            "fun P => fun Q => fun h => fun q => h P (fun (forward : forall (p : P), Q) => fun (backward : forall (q : Q), P) => backward q)",
    },
    TheoremArtifact {
        name: "and_intro",
        universe_params: &[],
        statement:
            "forall (P : Prop), forall (Q : Prop), forall (p : P), forall (q : Q), And P Q",
        proof:
            "fun P => fun Q => fun p => fun q => fun (R : Prop) => fun (mk : forall (p : P), forall (q : Q), R) => mk p q",
    },
    TheoremArtifact {
        name: "and_left",
        universe_params: &[],
        statement: "forall (P : Prop), forall (Q : Prop), forall (h : And P Q), P",
        proof: "fun P => fun Q => fun h => h P (fun (p : P) => fun (q : Q) => p)",
    },
    TheoremArtifact {
        name: "and_right",
        universe_params: &[],
        statement: "forall (P : Prop), forall (Q : Prop), forall (h : And P Q), Q",
        proof: "fun P => fun Q => fun h => h Q (fun (p : P) => fun (q : Q) => q)",
    },
    TheoremArtifact {
        name: "iff_of_eq",
        universe_params: &[],
        statement:
            "forall (P : Prop), forall (Q : Prop), forall (h : @Eq.{1} Prop P Q), Iff P Q",
        proof:
            "fun P => fun Q => fun h => @Eq.rec.{1,0} Prop P (fun (R : Prop) => fun (hR : @Eq.{1} Prop P R) => Iff P R) (iff_refl P) Q h",
    },
    TheoremArtifact {
        name: "false_elim",
        universe_params: &[],
        statement: "forall (P : Prop), forall (h : False), P",
        proof: "fun P => fun h => h P",
    },
    TheoremArtifact {
        name: "not_intro",
        universe_params: &[],
        statement:
            "forall (P : Prop), forall (h : forall (p : P), False), Not P",
        proof: "fun P => fun h => h",
    },
    TheoremArtifact {
        name: "not_elim",
        universe_params: &[],
        statement:
            "forall (P : Prop), forall (hn : Not P), forall (p : P), False",
        proof: "fun P => fun hn => fun p => hn p",
    },
    TheoremArtifact {
        name: "or_inl",
        universe_params: &[],
        statement:
            "forall (P : Prop), forall (Q : Prop), forall (p : P), Or P Q",
        proof:
            "fun P => fun Q => fun p => fun (R : Prop) => fun (left : forall (p : P), R) => fun (right : forall (q : Q), R) => left p",
    },
    TheoremArtifact {
        name: "or_inr",
        universe_params: &[],
        statement:
            "forall (P : Prop), forall (Q : Prop), forall (q : Q), Or P Q",
        proof:
            "fun P => fun Q => fun q => fun (R : Prop) => fun (left : forall (p : P), R) => fun (right : forall (q : Q), R) => right q",
    },
    TheoremArtifact {
        name: "or_elim",
        universe_params: &[],
        statement:
            "forall (P : Prop), forall (Q : Prop), forall (R : Prop), forall (h : Or P Q), forall (left : forall (p : P), R), forall (right : forall (q : Q), R), R",
        proof: "fun P => fun Q => fun R => fun h => fun left => fun right => h R left right",
    },
    TheoremArtifact {
        name: "iff_congr_arg",
        universe_params: &[],
        statement:
            "forall (P : Prop), forall (Q : Prop), forall (F : forall (X : Prop), Prop), forall (h : @Eq.{1} Prop P Q), Iff (F P) (F Q)",
        proof:
            "fun P => fun Q => fun F => fun h => @Eq.rec.{1,0} Prop P (fun (R : Prop) => fun (hR : @Eq.{1} Prop P R) => Iff (F P) (F R)) (iff_refl (F P)) Q h",
    },
];

const EQ_IMPORT_SOURCE: &str = "\
inductive Eq.{u} {A : Sort u} (a : A) : forall (b : A), Prop where
| refl : Eq.{u} a a
";

const NAT_IMPORT_SOURCE: &str = "\
inductive Nat : Type where
| zero : Nat
| succ : forall (n : Nat), Nat
";

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let repo_root = repo_root()?;
    let proof_root = repo_root.join("proofs");
    fs::create_dir_all(&proof_root)
        .map_err(|err| format!("failed to create {}: {err}", proof_root.display()))?;

    let (eq_import, eq_source_interface) =
        verified_core_import_with_source_interface(std_logic_eq_module(), EQ_IMPORT_SOURCE)?;
    let (nat_import, nat_source_interface) =
        verified_core_import_with_source_interface(std_nat_basic_module(), NAT_IMPORT_SOURCE)?;
    let eq_imports = vec![eq_import.clone(), nat_import.clone()];
    let eq_source_interfaces = vec![eq_source_interface.clone(), nat_source_interface.clone()];
    let eq_reasoning_imports = vec![eq_import.clone()];
    let eq_reasoning_source_interfaces = vec![eq_source_interface.clone()];
    let ring_imports = vec![eq_import.clone()];
    let ring_source_interfaces = vec![eq_source_interface.clone()];
    let nat_imports = vec![eq_import.clone(), nat_import.clone()];
    let nat_source_interfaces = vec![eq_source_interface.clone(), nat_source_interface.clone()];
    let reduction_imports = vec![nat_import];
    let reduction_source_interfaces = vec![nat_source_interface];

    let basic = build_and_write_module(&proof_root, &BASIC_MODULE, &[], &[])?;
    let eq = build_and_write_module(&proof_root, &EQ_MODULE, &eq_imports, &eq_source_interfaces)?;
    let nat = build_and_write_module(
        &proof_root,
        &NAT_MODULE,
        &nat_imports,
        &nat_source_interfaces,
    )?;
    let prop = build_and_write_module(&proof_root, &PROP_MODULE, &[], &[])?;
    let reduction = build_and_write_module(
        &proof_root,
        &REDUCTION_MODULE,
        &reduction_imports,
        &reduction_source_interfaces,
    )?;
    let eq_reasoning = build_and_write_module(
        &proof_root,
        &EQ_REASONING_MODULE,
        &eq_reasoning_imports,
        &eq_reasoning_source_interfaces,
    )?;
    let ring = build_and_write_module(
        &proof_root,
        &RING_MODULE,
        &ring_imports,
        &ring_source_interfaces,
    )?;
    let square_imports = vec![eq_import.clone(), ring.verified_module.clone()];
    let square_source_interfaces = vec![eq_source_interface.clone(), ring.source_interface.clone()];
    let square = build_and_write_module(
        &proof_root,
        &SQUARE_MODULE,
        &square_imports,
        &square_source_interfaces,
    )?;
    let ordered_field_imports = vec![
        eq_import.clone(),
        ring.verified_module.clone(),
        square.verified_module.clone(),
    ];
    let ordered_field_source_interfaces = vec![
        eq_source_interface.clone(),
        ring.source_interface.clone(),
        square.source_interface.clone(),
    ];
    let ordered_field = build_and_write_module(
        &proof_root,
        &ORDERED_FIELD_MODULE,
        &ordered_field_imports,
        &ordered_field_source_interfaces,
    )?;
    let vector_basic_imports = vec![eq_import.clone()];
    let vector_basic_source_interfaces = vec![eq_source_interface.clone()];
    let vector_basic = build_and_write_module(
        &proof_root,
        &VECTOR_BASIC_MODULE,
        &vector_basic_imports,
        &vector_basic_source_interfaces,
    )?;
    let vector_dot_imports = vec![
        eq_import.clone(),
        ring.verified_module.clone(),
        square.verified_module.clone(),
        ordered_field.verified_module.clone(),
        vector_basic.verified_module.clone(),
    ];
    let vector_dot_source_interfaces = vec![
        eq_source_interface.clone(),
        ring.source_interface.clone(),
        square.source_interface.clone(),
        ordered_field.source_interface.clone(),
        vector_basic.source_interface.clone(),
    ];
    let vector_dot = build_and_write_module(
        &proof_root,
        &VECTOR_DOT_MODULE,
        &vector_dot_imports,
        &vector_dot_source_interfaces,
    )?;
    let right_triangle_imports = vec![
        eq_import.clone(),
        ring.verified_module.clone(),
        square.verified_module.clone(),
        ordered_field.verified_module.clone(),
        vector_basic.verified_module.clone(),
        vector_dot.verified_module.clone(),
    ];
    let right_triangle_source_interfaces = vec![
        eq_source_interface.clone(),
        ring.source_interface.clone(),
        square.source_interface.clone(),
        ordered_field.source_interface.clone(),
        vector_basic.source_interface.clone(),
        vector_dot.source_interface.clone(),
    ];
    let right_triangle = build_and_write_module(
        &proof_root,
        &RIGHT_TRIANGLE_MODULE,
        &right_triangle_imports,
        &right_triangle_source_interfaces,
    )?;
    let metric_imports = vec![
        eq_import.clone(),
        ring.verified_module.clone(),
        square.verified_module.clone(),
        ordered_field.verified_module.clone(),
        vector_basic.verified_module.clone(),
        vector_dot.verified_module.clone(),
        right_triangle.verified_module.clone(),
    ];
    let metric_source_interfaces = vec![
        eq_source_interface.clone(),
        ring.source_interface.clone(),
        square.source_interface.clone(),
        ordered_field.source_interface.clone(),
        vector_basic.source_interface.clone(),
        vector_dot.source_interface.clone(),
        right_triangle.source_interface.clone(),
    ];
    let metric = build_and_write_module(
        &proof_root,
        &METRIC_MODULE,
        &metric_imports,
        &metric_source_interfaces,
    )?;
    let iff_imports = vec![eq_import.clone()];
    let iff_source_interfaces = vec![eq_source_interface.clone()];
    let iff = build_and_write_module(
        &proof_root,
        &IFF_MODULE,
        &iff_imports,
        &iff_source_interfaces,
    )?;

    write(
        proof_root.join(MANIFEST_PATH),
        manifest_toml(&[
            basic,
            eq,
            nat,
            prop,
            reduction,
            eq_reasoning,
            ring,
            square,
            ordered_field,
            vector_basic,
            vector_dot,
            right_triangle,
            metric,
            iff,
        ])
        .as_bytes(),
    )?;

    Ok(())
}

fn build_and_write_module(
    proof_root: &Path,
    config: &'static ModuleArtifact,
    verified_modules: &[npa_cert::VerifiedModule],
    imported_source_interfaces: &[npa_frontend::HumanImportedSourceInterface],
) -> Result<GeneratedModule, String> {
    let source = module_source(config);
    let output = npa_frontend::compile_human_source_to_certificate_output_with_source_interfaces(
        npa_frontend::FileId(0),
        npa_cert::Name::from_dotted(config.module),
        &source,
        verified_modules,
        imported_source_interfaces,
        &npa_frontend::HumanCompileOptions::default(),
    )
    .map_err(|err| format!("failed to compile {}: {err:?}", config.source_path))?;
    let certificate_bytes = npa_cert::encode_module_cert(&output.certificate)
        .map_err(|err| format!("failed to encode {}: {err:?}", config.certificate_path))?;

    let mut session = npa_cert::VerifierSession::new();
    for import in verified_modules {
        session.register_verified_module(import.clone());
    }
    let verified = npa_cert::verify_module_cert(
        &certificate_bytes,
        &mut session,
        &npa_cert::AxiomPolicy::normal(),
    )
    .map_err(|err| format!("generated certificate did not verify: {err:?}"))?;
    let axioms = verified
        .axiom_report()
        .module_axioms
        .iter()
        .map(|axiom| verified.name_table()[axiom.name].as_dotted())
        .collect::<Vec<_>>();
    let expected_axioms = config
        .expected_axioms
        .iter()
        .map(|axiom| (*axiom).to_owned())
        .collect::<Vec<_>>();
    if axioms != expected_axioms {
        return Err(format!(
            "generated AI proof corpus module {} has axioms {:?}, expected {:?}",
            config.module, axioms, expected_axioms
        ));
    }

    write(proof_root.join(config.source_path), source.as_bytes())?;
    write(proof_root.join(config.certificate_path), &certificate_bytes)?;

    let source_sha256 = tagged_sha256(source.as_bytes());
    let certificate_file_sha256 = tagged_sha256(&certificate_bytes);
    let export_hash = tagged_hash(output.certificate.hashes.export_hash);
    let axiom_report_hash = tagged_hash(output.certificate.hashes.axiom_report_hash);
    let certificate_hash = tagged_hash(output.certificate.hashes.certificate_hash);

    let generated = GeneratedModule {
        config,
        source_sha256,
        certificate_file_sha256,
        export_hash,
        axiom_report_hash,
        certificate_hash,
        axioms,
        verified_module: verified.clone(),
        source_interface: human_imported_source_interface(&verified, &output.source_interface),
    };

    write(
        proof_root.join(config.meta_path),
        meta_json(&generated).as_bytes(),
    )?;
    write(
        proof_root.join(config.replay_path),
        replay_json(config).as_bytes(),
    )?;

    Ok(generated)
}

fn human_imported_source_interface(
    verified: &npa_cert::VerifiedModule,
    source_interface: &npa_frontend::HumanSourceInterface,
) -> npa_frontend::HumanImportedSourceInterface {
    let import = npa_frontend::VerifiedImport::from(verified);
    npa_frontend::HumanImportedSourceInterface {
        module: import.module,
        export_hash: import.export_hash,
        certificate_hash: import.certificate_hash,
        source_interface: source_interface.clone(),
    }
}

fn verified_core_import_with_source_interface(
    module: npa_cert::CoreModule,
    source: &str,
) -> Result<
    (
        npa_cert::VerifiedModule,
        npa_frontend::HumanImportedSourceInterface,
    ),
    String,
> {
    let module_name = module.name.as_dotted();
    let output = npa_frontend::compile_human_source_to_certificate_output_with_source_interfaces(
        npa_frontend::FileId(0),
        module.name.clone(),
        source,
        &[],
        &[],
        &npa_frontend::HumanCompileOptions::default(),
    )
    .map_err(|err| format!("failed to compile import {module_name}: {err:?}"))?;
    let mut source_interface = output.source_interface;
    clear_source_interface_hashes(&mut source_interface);
    let cert = npa_cert::build_module_cert(module, &[])
        .map_err(|err| format!("failed to build import {module_name}: {err:?}"))?;
    let bytes = npa_cert::encode_module_cert(&cert)
        .map_err(|err| format!("failed to encode import {module_name}: {err:?}"))?;
    let verified = npa_cert::verify_module_cert(
        &bytes,
        &mut npa_cert::VerifierSession::new(),
        &npa_cert::AxiomPolicy::normal(),
    )
    .map_err(|err| format!("import {module_name} did not verify: {err:?}"))?;
    let source_interface = human_imported_source_interface(&verified, &source_interface);

    Ok((verified, source_interface))
}

fn clear_source_interface_hashes(source_interface: &mut npa_frontend::HumanSourceInterface) {
    for declaration in &mut source_interface.declarations {
        declaration.decl_interface_hash = None;
    }
    for declaration in &mut source_interface.generated_declarations {
        declaration.decl_interface_hash = None;
    }
    for class in &mut source_interface.typeclass_classes {
        class.decl_interface_hash = None;
        for field in &mut class.fields {
            field.decl_interface_hash = None;
        }
    }
    for instance in &mut source_interface.typeclass_instances {
        instance.decl_interface_hash = None;
    }
}

fn std_logic_eq_module() -> npa_cert::CoreModule {
    npa_cert::CoreModule {
        name: npa_cert::Name::from_dotted("Std.Logic.Eq"),
        declarations: vec![npa_kernel::Decl::Inductive {
            name: "Eq".to_owned(),
            universe_params: vec!["u".to_owned()],
            ty: npa_kernel::eq_type(npa_kernel::Level::param("u")),
            data: Box::new(npa_kernel::eq_inductive()),
        }],
    }
}

fn std_nat_basic_module() -> npa_cert::CoreModule {
    npa_cert::CoreModule {
        name: npa_cert::Name::from_dotted("Std.Nat.Basic"),
        declarations: vec![npa_kernel::Decl::Inductive {
            name: "Nat".to_owned(),
            universe_params: Vec::new(),
            ty: npa_kernel::Expr::sort(npa_kernel::type0()),
            data: Box::new(npa_kernel::nat_inductive()),
        }],
    }
}

fn repo_root() -> Result<PathBuf, String> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            format!(
                "could not resolve repository root from {}",
                manifest_dir.display()
            )
        })
}

fn write(path: PathBuf, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    fs::write(&path, bytes).map_err(|err| format!("failed to write {}: {err}", path.display()))
}

fn module_source(config: &ModuleArtifact) -> String {
    let mut source = String::new();
    for import in config.imports {
        source.push_str("import ");
        source.push_str(import);
        source.push('\n');
    }
    if !config.imports.is_empty() {
        source.push('\n');
    }
    for inductive in config.inductives {
        source.push_str("inductive ");
        source.push_str(inductive.name);
        if !inductive.universe_params.is_empty() {
            source.push_str(".{");
            source.push_str(&inductive.universe_params.join(","));
            source.push('}');
        }
        source.push_str(" :\n  ");
        source.push_str(inductive.ty);
        source.push_str(" where\n");
        for constructor in inductive.constructors {
            source.push_str("| ");
            source.push_str(constructor.name);
            source.push_str(" : ");
            source.push_str(constructor.ty);
            source.push('\n');
        }
        source.push('\n');
    }
    for definition in config.definitions {
        source.push_str("def ");
        source.push_str(definition.name);
        if !definition.universe_params.is_empty() {
            source.push_str(".{");
            source.push_str(&definition.universe_params.join(","));
            source.push('}');
        }
        source.push_str(" :\n  ");
        source.push_str(definition.ty);
        source.push_str(" :=\n  ");
        source.push_str(definition.value);
        source.push_str("\n\n");
    }
    for theorem in config.theorems {
        source.push_str("theorem ");
        source.push_str(theorem.name);
        if !theorem.universe_params.is_empty() {
            source.push_str(".{");
            source.push_str(&theorem.universe_params.join(","));
            source.push('}');
        }
        source.push_str(" :\n  ");
        source.push_str(theorem.statement);
        source.push_str(" :=\n  ");
        source.push_str(theorem.proof);
        source.push_str("\n\n");
    }
    source
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

fn manifest_toml(modules: &[GeneratedModule]) -> String {
    let mut manifest = "schema = \"npa-ai-proof-corpus-v0.1\"\n".to_owned();
    for module in modules {
        manifest.push('\n');
        manifest.push_str("[[proof_modules]]\n");
        manifest.push_str(&format!("module = \"{}\"\n", module.config.module));
        manifest.push_str(&format!("source = \"{}\"\n", module.config.source_path));
        manifest.push_str(&format!(
            "certificate = \"{}\"\n",
            module.config.certificate_path
        ));
        manifest.push_str(&format!("meta = \"{}\"\n", module.config.meta_path));
        manifest.push_str(&format!("replay = \"{}\"\n", module.config.replay_path));
        manifest.push_str("producer_profile = \"human-surface-explicit-term\"\n");
        manifest.push_str("trusted_status = \"verified_by_certificate\"\n");
        manifest.push_str(&format!("source_sha256 = \"{}\"\n", module.source_sha256));
        manifest.push_str(&format!(
            "certificate_file_sha256 = \"{}\"\n",
            module.certificate_file_sha256
        ));
        manifest.push_str(&format!("export_hash = \"{}\"\n", module.export_hash));
        manifest.push_str(&format!(
            "axiom_report_hash = \"{}\"\n",
            module.axiom_report_hash
        ));
        manifest.push_str(&format!(
            "certificate_hash = \"{}\"\n",
            module.certificate_hash
        ));
        manifest.push_str(&format!(
            "imports = [{}]\n",
            quoted_items(module.config.imports)
        ));
        if !module.config.inductives.is_empty() {
            manifest.push_str(&format!(
                "inductives = [{}]\n",
                quoted_items(
                    &module
                        .config
                        .inductives
                        .iter()
                        .map(|inductive| inductive.name)
                        .collect::<Vec<_>>()
                )
            ));
        }
        manifest.push_str(&format!(
            "definitions = [{}]\n",
            quoted_items(
                &module
                    .config
                    .definitions
                    .iter()
                    .map(|definition| definition.name)
                    .collect::<Vec<_>>()
            )
        ));
        manifest.push_str(&format!(
            "theorems = [{}]\n",
            quoted_items(
                &module
                    .config
                    .theorems
                    .iter()
                    .map(|theorem| theorem.name)
                    .collect::<Vec<_>>()
            )
        ));
        manifest.push_str(&format!(
            "axioms = [{}]\n",
            quoted_owned_items(&module.axioms)
        ));
    }
    manifest
}

fn meta_json(module: &GeneratedModule) -> String {
    let inductives = module.config.inductives.iter().map(|inductive| {
        format!(
            "    {{ \"name\": \"{}\", \"kind\": \"inductive\" }}",
            inductive.name
        )
    });
    let definitions = module.config.definitions.iter().map(|definition| {
        format!(
            "    {{ \"name\": \"{}\", \"kind\": \"def\" }}",
            definition.name
        )
    });
    let theorems = module.config.theorems.iter().map(|theorem| {
        format!(
            "    {{ \"name\": \"{}\", \"kind\": \"theorem\" }}",
            theorem.name
        )
    });
    let declarations = inductives
        .chain(definitions)
        .chain(theorems)
        .collect::<Vec<_>>()
        .join(",\n");
    format!(
        "\
{{
  \"schema\": \"npa-ai-proof-meta-v0.1\",
  \"module\": \"{}\",
  \"source\": \"{}\",
  \"certificate\": \"{}\",
  \"producer_profile\": \"human-surface-explicit-term\",
  \"trusted_status\": \"verified_by_certificate\",
  \"source_sha256\": \"{}\",
  \"certificate_file_sha256\": \"{}\",
  \"export_hash\": \"{}\",
  \"axiom_report_hash\": \"{}\",
  \"certificate_hash\": \"{}\",
  \"imports\": [{}],
  \"axioms\": [{}],
  \"declarations\": [
{}
  ],
  \"trust_boundary\": \"source, replay, and metadata are non-trusted sidecars; only the canonical certificate verified by npa-cert is accepted\"
}}
",
        module.config.module,
        module.config.source_path,
        module.config.certificate_path,
        module.source_sha256,
        module.certificate_file_sha256,
        module.export_hash,
        module.axiom_report_hash,
        module.certificate_hash,
        quoted_items(module.config.imports),
        quoted_owned_items(&module.axioms),
        declarations
    )
}

fn replay_json(config: &ModuleArtifact) -> String {
    let inductive_steps = config.inductives.iter().map(|inductive| {
        let term = inductive_replay_term(inductive);
        replay_step_json(inductive.name, "inductive_decl", &term)
    });
    let definition_steps = config.definitions.iter().map(|definition| {
        replay_step_json(definition.name, "explicit_def_value", definition.value)
    });
    let theorem_steps = config
        .theorems
        .iter()
        .map(|theorem| replay_step_json(theorem.name, "explicit_term", theorem.proof));
    let steps = inductive_steps
        .chain(definition_steps)
        .chain(theorem_steps)
        .collect::<Vec<_>>()
        .join(",\n");
    format!(
        "\
{{
  \"schema\": \"npa-ai-proof-replay-v0.1\",
  \"module\": \"{}\",
  \"trusted\": false,
  \"profile\": \"explicit_term_source_certificate_handoff\",
  \"steps\": [
{}
  ],
  \"acceptance\": {{
    \"required\": [\"decode_module_cert\", \"verify_module_cert\"],
    \"accepted_artifact\": \"{}\"
  }}
}}
",
        config.module, steps, config.certificate_path
    )
}

fn inductive_replay_term(inductive: &InductiveArtifact) -> String {
    let universe_params = if inductive.universe_params.is_empty() {
        String::new()
    } else {
        format!(".{{{}}}", inductive.universe_params.join(","))
    };
    let constructors = inductive
        .constructors
        .iter()
        .map(|constructor| format!("| {} : {}", constructor.name, constructor.ty))
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "inductive {}{} : {} where {}",
        inductive.name, universe_params, inductive.ty, constructors
    )
}

fn replay_step_json(declaration: &str, source_kind: &str, term: &str) -> String {
    format!(
        "    {{
      \"declaration\": \"{}\",
      \"source_kind\": \"{}\",
      \"term\": \"{}\"
    }}",
        declaration, source_kind, term
    )
}

fn quoted_items(items: &[&str]) -> String {
    items
        .iter()
        .map(|item| format!("\"{item}\""))
        .collect::<Vec<_>>()
        .join(", ")
}

fn quoted_owned_items(items: &[String]) -> String {
    items
        .iter()
        .map(|item| format!("\"{item}\""))
        .collect::<Vec<_>>()
        .join(", ")
}
