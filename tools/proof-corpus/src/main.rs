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

const ABSTRACT_GROUP_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Algebra.AbstractGroup",
    source_path: "Proofs/Ai/Algebra/AbstractGroup/source.npa",
    certificate_path: "Proofs/Ai/Algebra/AbstractGroup/certificate.npcert",
    meta_path: "Proofs/Ai/Algebra/AbstractGroup/meta.json",
    replay_path: "Proofs/Ai/Algebra/AbstractGroup/replay.json",
    imports: &["Std.Logic.Eq", "Proofs.Ai.EqReasoning"],
    inductives: &[],
    definitions: ABSTRACT_GROUP_DEFINITIONS,
    theorems: ABSTRACT_GROUP_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const ABSTRACT_GROUP_KERNEL_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Algebra.AbstractGroupKernel",
    source_path: "Proofs/Ai/Algebra/AbstractGroupKernel/source.npa",
    certificate_path: "Proofs/Ai/Algebra/AbstractGroupKernel/certificate.npcert",
    meta_path: "Proofs/Ai/Algebra/AbstractGroupKernel/meta.json",
    replay_path: "Proofs/Ai/Algebra/AbstractGroupKernel/replay.json",
    imports: &[
        "Std.Logic.Eq",
        "Proofs.Ai.EqReasoning",
        "Proofs.Ai.Algebra.AbstractGroup",
    ],
    inductives: &[],
    definitions: &[],
    theorems: ABSTRACT_GROUP_KERNEL_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const ABSTRACT_GROUP_IMAGE_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Algebra.AbstractGroupImage",
    source_path: "Proofs/Ai/Algebra/AbstractGroupImage/source.npa",
    certificate_path: "Proofs/Ai/Algebra/AbstractGroupImage/certificate.npcert",
    meta_path: "Proofs/Ai/Algebra/AbstractGroupImage/meta.json",
    replay_path: "Proofs/Ai/Algebra/AbstractGroupImage/replay.json",
    imports: &[
        "Std.Logic.Eq",
        "Proofs.Ai.EqReasoning",
        "Proofs.Ai.Algebra.AbstractGroup",
    ],
    inductives: &[],
    definitions: ABSTRACT_GROUP_IMAGE_DEFINITIONS,
    theorems: ABSTRACT_GROUP_IMAGE_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const ABSTRACT_GROUP_QUOTIENT_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Algebra.AbstractGroupQuotient",
    source_path: "Proofs/Ai/Algebra/AbstractGroupQuotient/source.npa",
    certificate_path: "Proofs/Ai/Algebra/AbstractGroupQuotient/certificate.npcert",
    meta_path: "Proofs/Ai/Algebra/AbstractGroupQuotient/meta.json",
    replay_path: "Proofs/Ai/Algebra/AbstractGroupQuotient/replay.json",
    imports: &["Std.Logic.Eq", "Proofs.Ai.Algebra.AbstractGroup"],
    inductives: &[],
    definitions: ABSTRACT_GROUP_QUOTIENT_DEFINITIONS,
    theorems: ABSTRACT_GROUP_QUOTIENT_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const ABSTRACT_GROUP_QUOTIENT_MUL_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Algebra.AbstractGroupQuotientMul",
    source_path: "Proofs/Ai/Algebra/AbstractGroupQuotientMul/source.npa",
    certificate_path: "Proofs/Ai/Algebra/AbstractGroupQuotientMul/certificate.npcert",
    meta_path: "Proofs/Ai/Algebra/AbstractGroupQuotientMul/meta.json",
    replay_path: "Proofs/Ai/Algebra/AbstractGroupQuotientMul/replay.json",
    imports: &[
        "Std.Logic.Eq",
        "Proofs.Ai.EqReasoning",
        "Proofs.Ai.Algebra.AbstractGroup",
        "Proofs.Ai.Algebra.AbstractGroupQuotient",
    ],
    inductives: &[],
    definitions: ABSTRACT_GROUP_QUOTIENT_MUL_DEFINITIONS,
    theorems: ABSTRACT_GROUP_QUOTIENT_MUL_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const ABSTRACT_GROUP_QUOTIENT_GROUP_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Algebra.AbstractGroupQuotientGroup",
    source_path: "Proofs/Ai/Algebra/AbstractGroupQuotientGroup/source.npa",
    certificate_path: "Proofs/Ai/Algebra/AbstractGroupQuotientGroup/certificate.npcert",
    meta_path: "Proofs/Ai/Algebra/AbstractGroupQuotientGroup/meta.json",
    replay_path: "Proofs/Ai/Algebra/AbstractGroupQuotientGroup/replay.json",
    imports: &[
        "Std.Logic.Eq",
        "Proofs.Ai.EqReasoning",
        "Proofs.Ai.Algebra.AbstractGroup",
        "Proofs.Ai.Algebra.AbstractGroupQuotient",
        "Proofs.Ai.Algebra.AbstractGroupQuotientMul",
    ],
    inductives: &[],
    definitions: ABSTRACT_GROUP_QUOTIENT_GROUP_DEFINITIONS,
    theorems: ABSTRACT_GROUP_QUOTIENT_GROUP_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const ABSTRACT_GROUP_QUOTIENT_HOM_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Algebra.AbstractGroupQuotientHom",
    source_path: "Proofs/Ai/Algebra/AbstractGroupQuotientHom/source.npa",
    certificate_path: "Proofs/Ai/Algebra/AbstractGroupQuotientHom/certificate.npcert",
    meta_path: "Proofs/Ai/Algebra/AbstractGroupQuotientHom/meta.json",
    replay_path: "Proofs/Ai/Algebra/AbstractGroupQuotientHom/replay.json",
    imports: &[
        "Std.Logic.Eq",
        "Proofs.Ai.EqReasoning",
        "Proofs.Ai.Algebra.AbstractGroup",
        "Proofs.Ai.Algebra.AbstractGroupQuotient",
        "Proofs.Ai.Algebra.AbstractGroupQuotientMul",
        "Proofs.Ai.Algebra.AbstractGroupQuotientGroup",
    ],
    inductives: &[],
    definitions: &[],
    theorems: ABSTRACT_GROUP_QUOTIENT_HOM_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const ABSTRACT_GROUP_FIRST_ISO_FULL_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Algebra.AbstractGroupFirstIsoFull",
    source_path: "Proofs/Ai/Algebra/AbstractGroupFirstIsoFull/source.npa",
    certificate_path: "Proofs/Ai/Algebra/AbstractGroupFirstIsoFull/certificate.npcert",
    meta_path: "Proofs/Ai/Algebra/AbstractGroupFirstIsoFull/meta.json",
    replay_path: "Proofs/Ai/Algebra/AbstractGroupFirstIsoFull/replay.json",
    imports: &[
        "Std.Logic.Eq",
        "Proofs.Ai.EqReasoning",
        "Proofs.Ai.Algebra.AbstractGroup",
        "Proofs.Ai.Algebra.AbstractGroupImage",
        "Proofs.Ai.Algebra.AbstractGroupQuotient",
        "Proofs.Ai.Algebra.AbstractGroupQuotientMul",
        "Proofs.Ai.Algebra.AbstractGroupQuotientGroup",
        "Proofs.Ai.Algebra.AbstractGroupQuotientHom",
    ],
    inductives: &[],
    definitions: &[],
    theorems: ABSTRACT_GROUP_FIRST_ISO_FULL_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const ABSTRACT_GROUP_FIRST_ISO_IMAGE_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Algebra.AbstractGroupFirstIsoImage",
    source_path: "Proofs/Ai/Algebra/AbstractGroupFirstIsoImage/source.npa",
    certificate_path: "Proofs/Ai/Algebra/AbstractGroupFirstIsoImage/certificate.npcert",
    meta_path: "Proofs/Ai/Algebra/AbstractGroupFirstIsoImage/meta.json",
    replay_path: "Proofs/Ai/Algebra/AbstractGroupFirstIsoImage/replay.json",
    imports: &[
        "Std.Logic.Eq",
        "Proofs.Ai.EqReasoning",
        "Proofs.Ai.Algebra.AbstractGroup",
        "Proofs.Ai.Algebra.AbstractGroupImage",
        "Proofs.Ai.Algebra.AbstractGroupQuotient",
        "Proofs.Ai.Algebra.AbstractGroupQuotientMul",
        "Proofs.Ai.Algebra.AbstractGroupQuotientGroup",
        "Proofs.Ai.Algebra.AbstractGroupFirstIsoFull",
    ],
    inductives: ABSTRACT_GROUP_FIRST_ISO_IMAGE_INDUCTIVES,
    definitions: ABSTRACT_GROUP_FIRST_ISO_IMAGE_DEFINITIONS,
    theorems: ABSTRACT_GROUP_FIRST_ISO_IMAGE_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const ABSTRACT_GROUP_FIRST_ISO_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Algebra.AbstractGroupFirstIso",
    source_path: "Proofs/Ai/Algebra/AbstractGroupFirstIso/source.npa",
    certificate_path: "Proofs/Ai/Algebra/AbstractGroupFirstIso/certificate.npcert",
    meta_path: "Proofs/Ai/Algebra/AbstractGroupFirstIso/meta.json",
    replay_path: "Proofs/Ai/Algebra/AbstractGroupFirstIso/replay.json",
    imports: &[
        "Std.Logic.Eq",
        "Proofs.Ai.Algebra.AbstractGroup",
        "Proofs.Ai.Algebra.AbstractGroupImage",
        "Proofs.Ai.Algebra.AbstractGroupQuotient",
    ],
    inductives: &[],
    definitions: ABSTRACT_GROUP_FIRST_ISO_DEFINITIONS,
    theorems: ABSTRACT_GROUP_FIRST_ISO_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const ABSTRACT_GROUP_SUBGROUP_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Algebra.AbstractGroupSubgroup",
    source_path: "Proofs/Ai/Algebra/AbstractGroupSubgroup/source.npa",
    certificate_path: "Proofs/Ai/Algebra/AbstractGroupSubgroup/certificate.npcert",
    meta_path: "Proofs/Ai/Algebra/AbstractGroupSubgroup/meta.json",
    replay_path: "Proofs/Ai/Algebra/AbstractGroupSubgroup/replay.json",
    imports: &[
        "Std.Logic.Eq",
        "Proofs.Ai.Algebra.AbstractGroup",
        "Proofs.Ai.EqReasoning",
    ],
    inductives: &[],
    definitions: ABSTRACT_GROUP_SUBGROUP_DEFINITIONS,
    theorems: ABSTRACT_GROUP_SUBGROUP_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const ABSTRACT_GROUP_NORMAL_QUOTIENT_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Algebra.AbstractGroupNormalQuotient",
    source_path: "Proofs/Ai/Algebra/AbstractGroupNormalQuotient/source.npa",
    certificate_path: "Proofs/Ai/Algebra/AbstractGroupNormalQuotient/certificate.npcert",
    meta_path: "Proofs/Ai/Algebra/AbstractGroupNormalQuotient/meta.json",
    replay_path: "Proofs/Ai/Algebra/AbstractGroupNormalQuotient/replay.json",
    imports: &[
        "Std.Logic.Eq",
        "Proofs.Ai.Algebra.AbstractGroup",
        "Proofs.Ai.Algebra.AbstractGroupSubgroup",
    ],
    inductives: &[],
    definitions: ABSTRACT_GROUP_NORMAL_QUOTIENT_DEFINITIONS,
    theorems: ABSTRACT_GROUP_NORMAL_QUOTIENT_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const ABSTRACT_GROUP_NORMAL_QUOTIENT_MUL_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul",
    source_path: "Proofs/Ai/Algebra/AbstractGroupNormalQuotientMul/source.npa",
    certificate_path: "Proofs/Ai/Algebra/AbstractGroupNormalQuotientMul/certificate.npcert",
    meta_path: "Proofs/Ai/Algebra/AbstractGroupNormalQuotientMul/meta.json",
    replay_path: "Proofs/Ai/Algebra/AbstractGroupNormalQuotientMul/replay.json",
    imports: &[
        "Std.Logic.Eq",
        "Proofs.Ai.Algebra.AbstractGroup",
        "Proofs.Ai.Algebra.AbstractGroupSubgroup",
        "Proofs.Ai.Algebra.AbstractGroupNormalQuotient",
    ],
    inductives: &[],
    definitions: ABSTRACT_GROUP_NORMAL_QUOTIENT_MUL_DEFINITIONS,
    theorems: ABSTRACT_GROUP_NORMAL_QUOTIENT_MUL_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const ABSTRACT_GROUP_NORMAL_QUOTIENT_GROUP_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup",
    source_path: "Proofs/Ai/Algebra/AbstractGroupNormalQuotientGroup/source.npa",
    certificate_path: "Proofs/Ai/Algebra/AbstractGroupNormalQuotientGroup/certificate.npcert",
    meta_path: "Proofs/Ai/Algebra/AbstractGroupNormalQuotientGroup/meta.json",
    replay_path: "Proofs/Ai/Algebra/AbstractGroupNormalQuotientGroup/replay.json",
    imports: &[
        "Std.Logic.Eq",
        "Proofs.Ai.Algebra.AbstractGroup",
        "Proofs.Ai.Algebra.AbstractGroupSubgroup",
        "Proofs.Ai.Algebra.AbstractGroupNormalQuotient",
        "Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul",
    ],
    inductives: &[],
    definitions: ABSTRACT_GROUP_NORMAL_QUOTIENT_GROUP_DEFINITIONS,
    theorems: ABSTRACT_GROUP_NORMAL_QUOTIENT_GROUP_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const ABSTRACT_GROUP_SECOND_ISO_PHI_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Algebra.AbstractGroupSecondIsoPhi",
    source_path: "Proofs/Ai/Algebra/AbstractGroupSecondIsoPhi/source.npa",
    certificate_path: "Proofs/Ai/Algebra/AbstractGroupSecondIsoPhi/certificate.npcert",
    meta_path: "Proofs/Ai/Algebra/AbstractGroupSecondIsoPhi/meta.json",
    replay_path: "Proofs/Ai/Algebra/AbstractGroupSecondIsoPhi/replay.json",
    imports: &[
        "Std.Logic.Eq",
        "Proofs.Ai.Algebra.AbstractGroup",
        "Proofs.Ai.Algebra.AbstractGroupSubgroup",
        "Proofs.Ai.Algebra.AbstractGroupNormalQuotient",
        "Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul",
        "Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup",
    ],
    inductives: &[],
    definitions: ABSTRACT_GROUP_SECOND_ISO_PHI_DEFINITIONS,
    theorems: ABSTRACT_GROUP_SECOND_ISO_PHI_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const ABSTRACT_GROUP_SECOND_ISO_KERNEL_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Algebra.AbstractGroupSecondIsoKernel",
    source_path: "Proofs/Ai/Algebra/AbstractGroupSecondIsoKernel/source.npa",
    certificate_path: "Proofs/Ai/Algebra/AbstractGroupSecondIsoKernel/certificate.npcert",
    meta_path: "Proofs/Ai/Algebra/AbstractGroupSecondIsoKernel/meta.json",
    replay_path: "Proofs/Ai/Algebra/AbstractGroupSecondIsoKernel/replay.json",
    imports: &[
        "Std.Logic.Eq",
        "Proofs.Ai.Algebra.AbstractGroup",
        "Proofs.Ai.Algebra.AbstractGroupSubgroup",
        "Proofs.Ai.Algebra.AbstractGroupNormalQuotient",
        "Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul",
        "Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup",
        "Proofs.Ai.Algebra.AbstractGroupSecondIsoPhi",
    ],
    inductives: &[],
    definitions: ABSTRACT_GROUP_SECOND_ISO_KERNEL_DEFINITIONS,
    theorems: ABSTRACT_GROUP_SECOND_ISO_KERNEL_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const ABSTRACT_GROUP_SECOND_ISO_IMAGE_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Algebra.AbstractGroupSecondIsoImage",
    source_path: "Proofs/Ai/Algebra/AbstractGroupSecondIsoImage/source.npa",
    certificate_path: "Proofs/Ai/Algebra/AbstractGroupSecondIsoImage/certificate.npcert",
    meta_path: "Proofs/Ai/Algebra/AbstractGroupSecondIsoImage/meta.json",
    replay_path: "Proofs/Ai/Algebra/AbstractGroupSecondIsoImage/replay.json",
    imports: &[
        "Std.Logic.Eq",
        "Proofs.Ai.EqReasoning",
        "Proofs.Ai.Algebra.AbstractGroup",
        "Proofs.Ai.Algebra.AbstractGroupSubgroup",
        "Proofs.Ai.Algebra.AbstractGroupNormalQuotient",
        "Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul",
        "Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup",
        "Proofs.Ai.Algebra.AbstractGroupSecondIsoPhi",
    ],
    inductives: &[],
    definitions: ABSTRACT_GROUP_SECOND_ISO_IMAGE_DEFINITIONS,
    theorems: ABSTRACT_GROUP_SECOND_ISO_IMAGE_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const ABSTRACT_GROUP_SECOND_ISO_FINAL_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Algebra.AbstractGroupSecondIsoFinal",
    source_path: "Proofs/Ai/Algebra/AbstractGroupSecondIsoFinal/source.npa",
    certificate_path: "Proofs/Ai/Algebra/AbstractGroupSecondIsoFinal/certificate.npcert",
    meta_path: "Proofs/Ai/Algebra/AbstractGroupSecondIsoFinal/meta.json",
    replay_path: "Proofs/Ai/Algebra/AbstractGroupSecondIsoFinal/replay.json",
    imports: &[
        "Std.Logic.Eq",
        "Proofs.Ai.Algebra.AbstractGroup",
        "Proofs.Ai.Algebra.AbstractGroupSubgroup",
        "Proofs.Ai.Algebra.AbstractGroupNormalQuotient",
        "Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul",
        "Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup",
        "Proofs.Ai.Algebra.AbstractGroupSecondIsoPhi",
        "Proofs.Ai.Algebra.AbstractGroupSecondIsoKernel",
        "Proofs.Ai.Algebra.AbstractGroupSecondIsoImage",
    ],
    inductives: &[],
    definitions: ABSTRACT_GROUP_SECOND_ISO_FINAL_DEFINITIONS,
    theorems: ABSTRACT_GROUP_SECOND_ISO_FINAL_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const ABSTRACT_GROUP_THIRD_ISO_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Algebra.AbstractGroupThirdIso",
    source_path: "Proofs/Ai/Algebra/AbstractGroupThirdIso/source.npa",
    certificate_path: "Proofs/Ai/Algebra/AbstractGroupThirdIso/certificate.npcert",
    meta_path: "Proofs/Ai/Algebra/AbstractGroupThirdIso/meta.json",
    replay_path: "Proofs/Ai/Algebra/AbstractGroupThirdIso/replay.json",
    imports: &[
        "Std.Logic.Eq",
        "Proofs.Ai.EqReasoning",
        "Proofs.Ai.Algebra.AbstractGroup",
        "Proofs.Ai.Algebra.AbstractGroupQuotient",
        "Proofs.Ai.Algebra.AbstractGroupSubgroup",
        "Proofs.Ai.Algebra.AbstractGroupNormalQuotient",
        "Proofs.Ai.Algebra.AbstractGroupNormalQuotientMul",
        "Proofs.Ai.Algebra.AbstractGroupNormalQuotientGroup",
    ],
    inductives: &[],
    definitions: ABSTRACT_GROUP_THIRD_ISO_DEFINITIONS,
    theorems: ABSTRACT_GROUP_THIRD_ISO_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const ABSTRACT_RING_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Algebra.AbstractRing",
    source_path: "Proofs/Ai/Algebra/AbstractRing/source.npa",
    certificate_path: "Proofs/Ai/Algebra/AbstractRing/certificate.npcert",
    meta_path: "Proofs/Ai/Algebra/AbstractRing/meta.json",
    replay_path: "Proofs/Ai/Algebra/AbstractRing/replay.json",
    imports: &["Std.Logic.Eq"],
    inductives: &[],
    definitions: ABSTRACT_RING_DEFINITIONS,
    theorems: ABSTRACT_RING_THEOREMS,
    expected_axioms: &[],
};

const ABSTRACT_ORDERED_FIELD_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Algebra.AbstractOrderedField",
    source_path: "Proofs/Ai/Algebra/AbstractOrderedField/source.npa",
    certificate_path: "Proofs/Ai/Algebra/AbstractOrderedField/certificate.npcert",
    meta_path: "Proofs/Ai/Algebra/AbstractOrderedField/meta.json",
    replay_path: "Proofs/Ai/Algebra/AbstractOrderedField/replay.json",
    imports: &["Proofs.Ai.Algebra.AbstractRing", "Std.Logic.Eq"],
    inductives: &[],
    definitions: ABSTRACT_ORDERED_FIELD_DEFINITIONS,
    theorems: ABSTRACT_ORDERED_FIELD_THEOREMS,
    expected_axioms: &[],
};

const ABSTRACT_SQUARE_NORMALIZE_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Algebra.AbstractSquareNormalize",
    source_path: "Proofs/Ai/Algebra/AbstractSquareNormalize/source.npa",
    certificate_path: "Proofs/Ai/Algebra/AbstractSquareNormalize/certificate.npcert",
    meta_path: "Proofs/Ai/Algebra/AbstractSquareNormalize/meta.json",
    replay_path: "Proofs/Ai/Algebra/AbstractSquareNormalize/replay.json",
    imports: &[
        "Proofs.Ai.Algebra.AbstractOrderedField",
        "Proofs.Ai.Algebra.AbstractRing",
        "Std.Logic.Eq",
    ],
    inductives: &[],
    definitions: &[],
    theorems: ABSTRACT_SQUARE_NORMALIZE_THEOREMS,
    expected_axioms: &[],
};

const ABSTRACT_SCALAR_DERIVE_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Algebra.AbstractScalarDerive",
    source_path: "Proofs/Ai/Algebra/AbstractScalarDerive/source.npa",
    certificate_path: "Proofs/Ai/Algebra/AbstractScalarDerive/certificate.npcert",
    meta_path: "Proofs/Ai/Algebra/AbstractScalarDerive/meta.json",
    replay_path: "Proofs/Ai/Algebra/AbstractScalarDerive/replay.json",
    imports: &[
        "Proofs.Ai.Algebra.AbstractRing",
        "Proofs.Ai.Algebra.AbstractSquareNormalize",
        "Proofs.Ai.EqReasoning",
        "Std.Logic.Eq",
    ],
    inductives: &[],
    definitions: &[],
    theorems: ABSTRACT_SCALAR_DERIVE_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const ABSTRACT_VECTOR_SPACE_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Vector.AbstractSpace",
    source_path: "Proofs/Ai/Vector/AbstractSpace/source.npa",
    certificate_path: "Proofs/Ai/Vector/AbstractSpace/certificate.npcert",
    meta_path: "Proofs/Ai/Vector/AbstractSpace/meta.json",
    replay_path: "Proofs/Ai/Vector/AbstractSpace/replay.json",
    imports: &[
        "Proofs.Ai.Algebra.AbstractOrderedField",
        "Proofs.Ai.Algebra.AbstractRing",
        "Proofs.Ai.Algebra.AbstractSquareNormalize",
        "Std.Logic.Eq",
    ],
    inductives: &[],
    definitions: ABSTRACT_VECTOR_SPACE_DEFINITIONS,
    theorems: ABSTRACT_VECTOR_SPACE_THEOREMS,
    expected_axioms: &[],
};

const ABSTRACT_INNER_PRODUCT_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Vector.AbstractInnerProduct",
    source_path: "Proofs/Ai/Vector/AbstractInnerProduct/source.npa",
    certificate_path: "Proofs/Ai/Vector/AbstractInnerProduct/certificate.npcert",
    meta_path: "Proofs/Ai/Vector/AbstractInnerProduct/meta.json",
    replay_path: "Proofs/Ai/Vector/AbstractInnerProduct/replay.json",
    imports: &[
        "Proofs.Ai.Algebra.AbstractOrderedField",
        "Proofs.Ai.Algebra.AbstractRing",
        "Proofs.Ai.Algebra.AbstractSquareNormalize",
        "Proofs.Ai.Vector.AbstractSpace",
        "Std.Logic.Eq",
    ],
    inductives: &[],
    definitions: ABSTRACT_INNER_PRODUCT_DEFINITIONS,
    theorems: ABSTRACT_INNER_PRODUCT_THEOREMS,
    expected_axioms: &[],
};

const ABSTRACT_INNER_PRODUCT_DERIVE_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Vector.AbstractInnerProductDerive",
    source_path: "Proofs/Ai/Vector/AbstractInnerProductDerive/source.npa",
    certificate_path: "Proofs/Ai/Vector/AbstractInnerProductDerive/certificate.npcert",
    meta_path: "Proofs/Ai/Vector/AbstractInnerProductDerive/meta.json",
    replay_path: "Proofs/Ai/Vector/AbstractInnerProductDerive/replay.json",
    imports: &[
        "Proofs.Ai.Algebra.AbstractOrderedField",
        "Proofs.Ai.Algebra.AbstractRing",
        "Proofs.Ai.Algebra.AbstractScalarDerive",
        "Proofs.Ai.EqReasoning",
        "Proofs.Ai.Vector.AbstractInnerProduct",
        "Proofs.Ai.Vector.AbstractSpace",
        "Std.Logic.Eq",
    ],
    inductives: &[],
    definitions: &[],
    theorems: ABSTRACT_INNER_PRODUCT_DERIVE_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const AFFINE_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Geometry.Affine",
    source_path: "Proofs/Ai/Geometry/Affine/source.npa",
    certificate_path: "Proofs/Ai/Geometry/Affine/certificate.npcert",
    meta_path: "Proofs/Ai/Geometry/Affine/meta.json",
    replay_path: "Proofs/Ai/Geometry/Affine/replay.json",
    imports: &[
        "Proofs.Ai.Algebra.AbstractOrderedField",
        "Proofs.Ai.Algebra.AbstractRing",
        "Proofs.Ai.Algebra.AbstractSquareNormalize",
        "Proofs.Ai.Vector.AbstractInnerProduct",
        "Proofs.Ai.Vector.AbstractSpace",
        "Std.Logic.Eq",
    ],
    inductives: &[],
    definitions: AFFINE_DEFINITIONS,
    theorems: AFFINE_THEOREMS,
    expected_axioms: &[],
};

const AFFINE_DERIVE_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Geometry.AffineDerive",
    source_path: "Proofs/Ai/Geometry/AffineDerive/source.npa",
    certificate_path: "Proofs/Ai/Geometry/AffineDerive/certificate.npcert",
    meta_path: "Proofs/Ai/Geometry/AffineDerive/meta.json",
    replay_path: "Proofs/Ai/Geometry/AffineDerive/replay.json",
    imports: &[
        "Proofs.Ai.Algebra.AbstractOrderedField",
        "Proofs.Ai.Algebra.AbstractRing",
        "Proofs.Ai.Algebra.AbstractSquareNormalize",
        "Proofs.Ai.Geometry.Affine",
        "Proofs.Ai.Vector.AbstractInnerProduct",
        "Proofs.Ai.Vector.AbstractSpace",
        "Std.Logic.Eq",
    ],
    inductives: &[],
    definitions: &[],
    theorems: AFFINE_DERIVE_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const ABSTRACT_RIGHT_TRIANGLE_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Geometry.AbstractRightTriangle",
    source_path: "Proofs/Ai/Geometry/AbstractRightTriangle/source.npa",
    certificate_path: "Proofs/Ai/Geometry/AbstractRightTriangle/certificate.npcert",
    meta_path: "Proofs/Ai/Geometry/AbstractRightTriangle/meta.json",
    replay_path: "Proofs/Ai/Geometry/AbstractRightTriangle/replay.json",
    imports: &[
        "Proofs.Ai.Algebra.AbstractOrderedField",
        "Proofs.Ai.Algebra.AbstractRing",
        "Proofs.Ai.Algebra.AbstractSquareNormalize",
        "Proofs.Ai.Geometry.Affine",
        "Proofs.Ai.Vector.AbstractInnerProduct",
        "Proofs.Ai.Vector.AbstractSpace",
        "Std.Logic.Eq",
    ],
    inductives: &[],
    definitions: ABSTRACT_RIGHT_TRIANGLE_DEFINITIONS,
    theorems: ABSTRACT_RIGHT_TRIANGLE_THEOREMS,
    expected_axioms: &[],
};

const ABSTRACT_RIGHT_TRIANGLE_DERIVE_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Geometry.AbstractRightTriangleDerive",
    source_path: "Proofs/Ai/Geometry/AbstractRightTriangleDerive/source.npa",
    certificate_path: "Proofs/Ai/Geometry/AbstractRightTriangleDerive/certificate.npcert",
    meta_path: "Proofs/Ai/Geometry/AbstractRightTriangleDerive/meta.json",
    replay_path: "Proofs/Ai/Geometry/AbstractRightTriangleDerive/replay.json",
    imports: &[
        "Proofs.Ai.Algebra.AbstractOrderedField",
        "Proofs.Ai.Algebra.AbstractRing",
        "Proofs.Ai.Algebra.AbstractSquareNormalize",
        "Proofs.Ai.Geometry.AbstractRightTriangle",
        "Proofs.Ai.Geometry.Affine",
        "Proofs.Ai.Geometry.AffineDerive",
        "Proofs.Ai.Vector.AbstractInnerProduct",
        "Proofs.Ai.Vector.AbstractInnerProductDerive",
        "Proofs.Ai.Vector.AbstractSpace",
        "Std.Logic.Eq",
    ],
    inductives: &[],
    definitions: &[],
    theorems: ABSTRACT_RIGHT_TRIANGLE_DERIVE_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const ABSTRACT_METRIC_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Geometry.AbstractMetric",
    source_path: "Proofs/Ai/Geometry/AbstractMetric/source.npa",
    certificate_path: "Proofs/Ai/Geometry/AbstractMetric/certificate.npcert",
    meta_path: "Proofs/Ai/Geometry/AbstractMetric/meta.json",
    replay_path: "Proofs/Ai/Geometry/AbstractMetric/replay.json",
    imports: &[
        "Proofs.Ai.Algebra.AbstractOrderedField",
        "Proofs.Ai.Algebra.AbstractRing",
        "Proofs.Ai.Algebra.AbstractSquareNormalize",
        "Proofs.Ai.Geometry.AbstractRightTriangle",
        "Proofs.Ai.Geometry.Affine",
        "Proofs.Ai.Geometry.AffineDerive",
        "Proofs.Ai.Vector.AbstractInnerProduct",
        "Proofs.Ai.Vector.AbstractInnerProductDerive",
        "Proofs.Ai.Vector.AbstractSpace",
        "Std.Logic.Eq",
    ],
    inductives: &[],
    definitions: ABSTRACT_METRIC_DEFINITIONS,
    theorems: ABSTRACT_METRIC_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

const PYTHAGOREAN_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Geometry.Pythagorean",
    source_path: "Proofs/Ai/Geometry/Pythagorean/source.npa",
    certificate_path: "Proofs/Ai/Geometry/Pythagorean/certificate.npcert",
    meta_path: "Proofs/Ai/Geometry/Pythagorean/meta.json",
    replay_path: "Proofs/Ai/Geometry/Pythagorean/replay.json",
    imports: &[
        "Proofs.Ai.Algebra.AbstractOrderedField",
        "Proofs.Ai.Algebra.AbstractRing",
        "Proofs.Ai.Algebra.AbstractScalarDerive",
        "Proofs.Ai.Algebra.AbstractSquareNormalize",
        "Proofs.Ai.EqReasoning",
        "Proofs.Ai.Geometry.AbstractMetric",
        "Proofs.Ai.Geometry.AbstractRightTriangle",
        "Proofs.Ai.Geometry.AbstractRightTriangleDerive",
        "Proofs.Ai.Geometry.Affine",
        "Proofs.Ai.Geometry.AffineDerive",
        "Proofs.Ai.Vector.AbstractInnerProduct",
        "Proofs.Ai.Vector.AbstractInnerProductDerive",
        "Proofs.Ai.Vector.AbstractSpace",
        "Std.Logic.Eq",
    ],
    inductives: &[],
    definitions: &[],
    theorems: PYTHAGOREAN_THEOREMS,
    expected_axioms: &["Eq.rec"],
};

macro_rules! abstract_ring_params {
    ($tail:literal) => {
        concat!(
            "forall (Scalar : Sort u), ",
            "forall (zero : Scalar), ",
            "forall (one : Scalar), ",
            "forall (add : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            "forall (neg : forall (a : Scalar), Scalar), ",
            "forall (sub : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            "forall (mul : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            $tail
        )
    };
}

macro_rules! abstract_ring_abs {
    (concat!($($tail:literal),+ $(,)?)) => {
        concat!(
            "fun Scalar => fun zero => fun one => fun add => fun neg => fun sub => fun mul => ",
            $($tail),+
        )
    };
    ($tail:literal) => {
        concat!(
            "fun Scalar => fun zero => fun one => fun add => fun neg => fun sub => fun mul => ",
            $tail
        )
    };
}

macro_rules! abstract_ordered_field_params {
    ($tail:literal) => {
        concat!(
            "forall (Scalar : Sort u), ",
            "forall (zero : Scalar), ",
            "forall (one : Scalar), ",
            "forall (add : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            "forall (neg : forall (a : Scalar), Scalar), ",
            "forall (sub : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            "forall (mul : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            "forall (le_rel : forall (a : Scalar), forall (b : Scalar), Prop), ",
            "forall (lt_rel : forall (a : Scalar), forall (b : Scalar), Prop), ",
            "forall (sqrt_fn : forall (a : Scalar), Scalar), ",
            $tail
        )
    };
}

macro_rules! abstract_ordered_field_abs {
    (concat!($($tail:tt)+)) => {
        concat!(
            "fun Scalar => fun zero => fun one => fun add => fun neg => fun sub => fun mul => fun le_rel => fun lt_rel => fun sqrt_fn => ",
            $($tail)+
        )
    };
    ($tail:literal) => {
        concat!(
            "fun Scalar => fun zero => fun one => fun add => fun neg => fun sub => fun mul => fun le_rel => fun lt_rel => fun sqrt_fn => ",
            $tail
        )
    };
}

macro_rules! abstract_vector_space_params {
    ($tail:literal) => {
        concat!(
            "forall (Scalar : Sort u), ",
            "forall (zero : Scalar), ",
            "forall (one : Scalar), ",
            "forall (add : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            "forall (neg : forall (a : Scalar), Scalar), ",
            "forall (sub : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            "forall (mul : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            "forall (Vector : Sort v), ",
            "forall (vzero : Vector), ",
            "forall (vadd : forall (x : Vector), forall (y : Vector), Vector), ",
            "forall (vneg : forall (x : Vector), Vector), ",
            "forall (smul : forall (a : Scalar), forall (x : Vector), Vector), ",
            $tail
        )
    };
}

macro_rules! abstract_vector_space_abs {
    (concat!($($tail:literal),+ $(,)?)) => {
        concat!(
            "fun Scalar => fun zero => fun one => fun add => fun neg => fun sub => fun mul => fun Vector => fun vzero => fun vadd => fun vneg => fun smul => ",
            $($tail),+
        )
    };
    ($tail:literal) => {
        concat!(
            "fun Scalar => fun zero => fun one => fun add => fun neg => fun sub => fun mul => fun Vector => fun vzero => fun vadd => fun vneg => fun smul => ",
            $tail
        )
    };
}

macro_rules! abstract_group_params {
    ($tail:literal) => {
        concat!(
            "forall (G : Sort u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            $tail
        )
    };
}

macro_rules! abstract_group_abs {
    (concat!($($tail:literal),+ $(,)?)) => {
        concat!(
            "fun G => fun one => fun mul => fun inv => ",
            $($tail),+
        )
    };
    ($tail:literal) => {
        concat!("fun G => fun one => fun mul => fun inv => ", $tail)
    };
}

macro_rules! abstract_group_hom_params {
    ($tail:literal) => {
        concat!(
            "forall (G : Sort u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            $tail
        )
    };
}

macro_rules! abstract_group_hom_abs {
    (concat!($($tail:literal),+ $(,)?)) => {
        concat!(
            "fun G => fun oneG => fun mulG => fun invG => fun H => fun oneH => fun mulH => fun invH => fun f => ",
            $($tail),+
        )
    };
    ($tail:literal) => {
        concat!(
            "fun G => fun oneG => fun mulG => fun invG => fun H => fun oneH => fun mulH => fun invH => fun f => ",
            $tail
        )
    };
}

macro_rules! abstract_inner_product_params {
    ($tail:literal) => {
        concat!(
            "forall (Scalar : Sort u), ",
            "forall (zero : Scalar), ",
            "forall (one : Scalar), ",
            "forall (add : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            "forall (neg : forall (a : Scalar), Scalar), ",
            "forall (sub : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            "forall (mul : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            "forall (le_rel : forall (a : Scalar), forall (b : Scalar), Prop), ",
            "forall (Vector : Sort v), ",
            "forall (vzero : Vector), ",
            "forall (vadd : forall (x : Vector), forall (y : Vector), Vector), ",
            "forall (vneg : forall (x : Vector), Vector), ",
            "forall (smul : forall (a : Scalar), forall (x : Vector), Vector), ",
            "forall (inner : forall (x : Vector), forall (y : Vector), Scalar), ",
            $tail
        )
    };
}

macro_rules! abstract_inner_product_abs {
    (concat!($($tail:tt)+)) => {
        concat!(
            "fun Scalar => fun zero => fun one => fun add => fun neg => fun sub => fun mul => fun le_rel => fun Vector => fun vzero => fun vadd => fun vneg => fun smul => fun inner => ",
            $($tail)+
        )
    };
    ($tail:literal) => {
        concat!(
            "fun Scalar => fun zero => fun one => fun add => fun neg => fun sub => fun mul => fun le_rel => fun Vector => fun vzero => fun vadd => fun vneg => fun smul => fun inner => ",
            $tail
        )
    };
}

macro_rules! abstract_ordered_inner_product_params {
    ($tail:literal) => {
        concat!(
            "forall (Scalar : Sort u), ",
            "forall (zero : Scalar), ",
            "forall (one : Scalar), ",
            "forall (add : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            "forall (neg : forall (a : Scalar), Scalar), ",
            "forall (sub : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            "forall (mul : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            "forall (le_rel : forall (a : Scalar), forall (b : Scalar), Prop), ",
            "forall (lt_rel : forall (a : Scalar), forall (b : Scalar), Prop), ",
            "forall (sqrt_fn : forall (a : Scalar), Scalar), ",
            "forall (Vector : Sort v), ",
            "forall (vzero : Vector), ",
            "forall (vadd : forall (x : Vector), forall (y : Vector), Vector), ",
            "forall (vneg : forall (x : Vector), Vector), ",
            "forall (smul : forall (a : Scalar), forall (x : Vector), Vector), ",
            "forall (inner : forall (x : Vector), forall (y : Vector), Scalar), ",
            $tail
        )
    };
}

macro_rules! abstract_ordered_inner_product_abs {
    (concat!($($tail:tt)+)) => {
        concat!(
            "fun Scalar => fun zero => fun one => fun add => fun neg => fun sub => fun mul => fun le_rel => fun lt_rel => fun sqrt_fn => fun Vector => fun vzero => fun vadd => fun vneg => fun smul => fun inner => ",
            $($tail)+
        )
    };
    ($tail:literal) => {
        concat!(
            "fun Scalar => fun zero => fun one => fun add => fun neg => fun sub => fun mul => fun le_rel => fun lt_rel => fun sqrt_fn => fun Vector => fun vzero => fun vadd => fun vneg => fun smul => fun inner => ",
            $tail
        )
    };
}

macro_rules! ring_args_elim {
    ($target:literal, $($tail:tt)+) => {
        concat!(
            "ring_args ",
            $target,
            " ",
            "(fun (sub_eq_add_neg_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub a b) (add a (neg b))) => ",
            "fun (add_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add a b) c) (add a (add b c))) => ",
            "fun (add_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add a b) (add b a)) => ",
            "fun (add_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (add a zero) a) => ",
            "fun (zero_add_arg : forall (a : Scalar), @Eq.{u} Scalar (add zero a) a) => ",
            "fun (neg_add_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add (neg a) a) zero) => ",
            "fun (add_neg_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add a (neg a)) zero) => ",
            "fun (sub_self_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a a) zero) => ",
            "fun (mul_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (mul a b) c) (mul a (mul b c))) => ",
            "fun (mul_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (mul a b) (mul b a)) => ",
            "fun (mul_one_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a one) a) => ",
            "fun (one_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul one a) a) => ",
            "fun (left_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul a (add b c)) (add (mul a b) (mul a c))) => ",
            "fun (right_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (add a b) c) (add (mul a c) (mul b c))) => ",
            "fun (mul_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a zero) zero) => ",
            "fun (zero_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul zero a) zero) => ",
            "fun (add_left_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add a b) (add a c)), @Eq.{u} Scalar b c) => ",
            "fun (ring_normalize_add_mul3_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add (mul a b) (mul b c)) (mul a c)) (add (add (mul a b) (mul a c)) (mul b c))) => ",
            "fun (add_right_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add b a) (add c a)), @Eq.{u} Scalar b c) => ",
            "fun (neg_neg_arg : forall (a : Scalar), @Eq.{u} Scalar (neg (neg a)) a) => ",
            "fun (sub_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a zero) a) => ",
            "fun (zero_sub_arg : forall (a : Scalar), @Eq.{u} Scalar (sub zero a) (neg a)) => ",
            "fun (sub_add_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add (sub a b) b) a) => ",
            "fun (add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub (add a b) b) a) => ",
            "fun (sub_add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (sub (sub a c) (sub b c)) (sub a b)) => ",
            $($tail)+,
            ")"
        )
    };
}

macro_rules! ordered_args_elim {
    ($target:literal, $($tail:tt)+) => {
        concat!(
            "ordered_args ",
            $target,
            " ",
            "(fun (le_refl_arg : forall (a : Scalar), le_rel a a) => ",
            "fun (le_trans_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (hab : le_rel a b), forall (hbc : le_rel b c), le_rel a c) => ",
            "fun (add_nonneg_arg : forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), le_rel zero (add a b)) => ",
            "fun (mul_nonneg_arg : forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), le_rel zero (mul a b)) => ",
            "fun (square_nonneg_arg : forall (a : Scalar), le_rel zero (@sq.{u} Scalar mul a)) => ",
            "fun (sqrt_nonneg_arg : forall (a : Scalar), le_rel zero (sqrt_fn a)) => ",
            "fun (sqrt_square_of_nonneg_arg : forall (a : Scalar), forall (ha : le_rel zero a), @Eq.{u} Scalar (sqrt_fn (@sq.{u} Scalar mul a)) a) => ",
            "fun (sqrt_mul_self_arg : forall (a : Scalar), forall (ha : le_rel zero a), @Eq.{u} Scalar (sqrt_fn (mul a a)) a) => ",
            "fun (eq_of_square_eq_square_nonneg_arg : forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hsq : @Eq.{u} Scalar (@sq.{u} Scalar mul a) (@sq.{u} Scalar mul b)), @Eq.{u} Scalar a b) => ",
            "fun (add_le_add_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (d : Scalar), forall (hab : le_rel a b), forall (hcd : le_rel c d), le_rel (add a c) (add b d)) => ",
            "fun (mul_le_mul_nonneg_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (d : Scalar), forall (ha : le_rel zero a), forall (hab : le_rel a b), forall (hc : le_rel zero c), forall (hcd : le_rel c d), le_rel (mul a c) (mul b d)) => ",
            "fun (zero_le_two_arg : le_rel zero (@two.{u} Scalar one add)) => ",
            "fun (le_antisymm_arg : forall (a : Scalar), forall (b : Scalar), forall (hab : le_rel a b), forall (hba : le_rel b a), @Eq.{u} Scalar a b) => ",
            "fun (lt_of_le_of_ne_arg : forall (a : Scalar), forall (ha : le_rel zero a), forall (hne : forall (haz : @Eq.{u} Scalar a zero), forall (P : Prop), P), lt_rel zero a) => ",
            "fun (le_of_eq_arg : forall (a : Scalar), forall (b : Scalar), forall (hab : @Eq.{u} Scalar a b), forall (P : Prop), forall (mk : forall (hab_le : le_rel a b), forall (hba_le : le_rel b a), P), P) => ",
            "fun (sqrt_sq_arg : forall (a : Scalar), forall (ha : le_rel zero a), @Eq.{u} Scalar (@sq.{u} Scalar mul (sqrt_fn a)) a) => ",
            "fun (sq_eq_zero_iff_arg : forall (a : Scalar), forall (R : Prop), forall (mk : forall (forward : forall (hsqz : @Eq.{u} Scalar (@sq.{u} Scalar mul a) zero), @Eq.{u} Scalar a zero), forall (backward : forall (haz : @Eq.{u} Scalar a zero), @Eq.{u} Scalar (@sq.{u} Scalar mul a) zero), R), R) => ",
            "fun (sum_nonneg_eq_zero_arg : forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hsum : @Eq.{u} Scalar (add a b) zero), forall (R : Prop), forall (mk : forall (haz : @Eq.{u} Scalar a zero), forall (hbz : @Eq.{u} Scalar b zero), R), R) => ",
            "fun (square_completion_bound_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (hquadratic : forall (t : Scalar), le_rel zero (add (add (mul a (@sq.{u} Scalar mul t)) (mul (mul (@two.{u} Scalar one add) b) t)) c)), le_rel (@sq.{u} Scalar mul b) (mul a c)) => ",
            "fun (le_of_sq_le_sq_nonneg_arg : forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hsq : le_rel (@sq.{u} Scalar mul a) (@sq.{u} Scalar mul b)), le_rel a b) => ",
            "fun (le_mul_sqrt_of_sq_le_mul_nonneg_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hsq : le_rel (@sq.{u} Scalar mul c) (mul a b)), le_rel c (mul (sqrt_fn a) (sqrt_fn b))) => ",
            "fun (add_two_mul_le_sq_add_sqrt_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (n : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hn : @Eq.{u} Scalar n (add (add a (mul (@two.{u} Scalar one add) c)) b)), forall (hc : le_rel c (mul (sqrt_fn a) (sqrt_fn b))), le_rel n (@sq.{u} Scalar mul (add (sqrt_fn a) (sqrt_fn b)))) => ",
            $($tail)+,
            ")"
        )
    };
}

macro_rules! vector_args_elim {
    ($target:literal, $($tail:tt)+) => {
        concat!(
            "vector_args ",
            $target,
            " ",
            "(fun (vec_sub_def_arg : forall (x : Vector), forall (y : Vector), @Eq.{v} Vector (@vsub.{v} Vector vadd vneg x y) (vadd x (vneg y))) => ",
            "fun (vec_add_assoc_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{v} Vector (vadd (vadd x y) z) (vadd x (vadd y z))) => ",
            "fun (vec_add_comm_arg : forall (x : Vector), forall (y : Vector), @Eq.{v} Vector (vadd x y) (vadd y x)) => ",
            "fun (vec_add_zero_arg : forall (x : Vector), @Eq.{v} Vector (vadd x vzero) x) => ",
            "fun (vec_zero_add_arg : forall (x : Vector), @Eq.{v} Vector (vadd vzero x) x) => ",
            "fun (vec_neg_add_cancel_arg : forall (x : Vector), @Eq.{v} Vector (vadd (vneg x) x) vzero) => ",
            "fun (vec_add_neg_cancel_arg : forall (x : Vector), @Eq.{v} Vector (vadd x (vneg x)) vzero) => ",
            "fun (sub_sub_sub_cancel_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{v} Vector (@vsub.{v} Vector vadd vneg (@vsub.{v} Vector vadd vneg x z) (@vsub.{v} Vector vadd vneg y z)) (@vsub.{v} Vector vadd vneg x y)) => ",
            "fun (vec_sub_self_arg : forall (x : Vector), @Eq.{v} Vector (@vsub.{v} Vector vadd vneg x x) vzero) => ",
            "fun (vec_sub_zero_arg : forall (x : Vector), @Eq.{v} Vector (@vsub.{v} Vector vadd vneg x vzero) x) => ",
            "fun (vec_add_left_cancel_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), forall (h : @Eq.{v} Vector (vadd x y) (vadd x z)), @Eq.{v} Vector y z) => ",
            "fun (smul_add_arg : forall (a : Scalar), forall (b : Scalar), forall (x : Vector), @Eq.{v} Vector (smul (add a b) x) (vadd (smul a x) (smul b x))) => ",
            "fun (add_smul_arg : forall (a : Scalar), forall (x : Vector), forall (y : Vector), @Eq.{v} Vector (smul a (vadd x y)) (vadd (smul a x) (smul a y))) => ",
            "fun (one_smul_arg : forall (x : Vector), @Eq.{v} Vector (smul one x) x) => ",
            "fun (mul_smul_arg : forall (a : Scalar), forall (b : Scalar), forall (x : Vector), @Eq.{v} Vector (smul (mul a b) x) (smul a (smul b x))) => ",
            "fun (zero_smul_arg : forall (x : Vector), @Eq.{v} Vector (smul zero x) vzero) => ",
            "fun (smul_zero_arg : forall (a : Scalar), @Eq.{v} Vector (smul a vzero) vzero) => ",
            "fun (neg_smul_arg : forall (a : Scalar), forall (x : Vector), @Eq.{v} Vector (smul (neg a) x) (vneg (smul a x))) => ",
            "fun (smul_neg_arg : forall (a : Scalar), forall (x : Vector), @Eq.{v} Vector (smul a (vneg x)) (vneg (smul a x))) => ",
            "fun (vec_sub_eq_add_neg_arg : forall (x : Vector), forall (y : Vector), @Eq.{v} Vector (@vsub.{v} Vector vadd vneg x y) (vadd x (vneg y))) => ",
            "fun (sub_add_sub_cancel_left_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{v} Vector (vadd (@vsub.{v} Vector vadd vneg x z) (@vsub.{v} Vector vadd vneg z y)) (@vsub.{v} Vector vadd vneg x y)) => ",
            "fun (linear_comb2_ext_arg : forall (a : Scalar), forall (x : Vector), forall (b : Scalar), forall (y : Vector), @Eq.{v} Vector (@linear_comb2.{u,v} Scalar Vector vadd smul a x b y) (vadd (smul a x) (smul b y))) => ",
            "fun (linear_comb3_ext_arg : forall (a : Scalar), forall (x : Vector), forall (b : Scalar), forall (y : Vector), forall (c : Scalar), forall (z : Vector), @Eq.{v} Vector (@linear_comb3.{u,v} Scalar Vector vadd smul a x b y c z) (vadd (vadd (smul a x) (smul b y)) (smul c z))) => ",
            $($tail)+,
            ")"
        )
    };
}

macro_rules! inner_args_elim {
    ($target:literal, $($tail:tt)+) => {
        concat!(
            "inner_args ",
            $target,
            " ",
            "(fun (dot_comm_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner y x)) => ",
            "fun (dot_add_left_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vadd x y) z) (add (@dot.{u,v} Scalar Vector inner x z) (@dot.{u,v} Scalar Vector inner y z))) => ",
            "fun (dot_add_right_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (vadd y z)) (add (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner x z))) => ",
            "fun (dot_neg_left_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vneg x) y) (neg (@dot.{u,v} Scalar Vector inner x y))) => ",
            "fun (dot_neg_right_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (vneg y)) (neg (@dot.{u,v} Scalar Vector inner x y))) => ",
            "fun (dot_sub_left_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y) z) (sub (@dot.{u,v} Scalar Vector inner x z) (@dot.{u,v} Scalar Vector inner y z))) => ",
            "fun (dot_sub_right_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (@vsub.{v} Vector vadd vneg y z)) (sub (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner x z))) => ",
            "fun (norm_sq_def_arg : forall (x : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) (@dot.{u,v} Scalar Vector inner x x)) => ",
            "fun (dist_sq_def_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@distSq.{u,v} Scalar Vector vadd vneg inner x y) (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg y x))) => ",
            "fun (dot_self_eq_norm_sq_arg : forall (x : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x x) (@normSq.{u,v} Scalar Vector inner x)) => ",
            "fun (norm_sq_add_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (add (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (norm_sq_sub_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (sub (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (inner_field13_arg : forall (x : Vector), forall (y : Vector), forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (inner_field14_arg : forall (x : Vector), forall (y : Vector), forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (norm_sq_nonneg_arg : forall (x : Vector), le_rel zero (@normSq.{u,v} Scalar Vector inner x)) => ",
            "fun (inner_field16_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (add (@normSq.{u,v} Scalar Vector inner (vadd x y)) (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y))) (add (mul (@two.{u} Scalar one add) (@normSq.{u,v} Scalar Vector inner x)) (mul (@two.{u} Scalar one add) (@normSq.{u,v} Scalar Vector inner y)))) => ",
            "fun (inner_field17_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y)) (sub (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)))) => ",
            "fun (perp_vec_iff_dot_eq_zero_arg : forall (x : Vector), forall (y : Vector), forall (R : Prop), forall (mk : forall (forward : forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), forall (backward : forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @PerpVec.{u,v} Scalar zero Vector inner x y), R), R) => ",
            "fun (perp_vec_symm_arg : forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @PerpVec.{u,v} Scalar zero Vector inner y x) => ",
            "fun (norm_sq_zero_iff_arg : forall (x : Vector), forall (R : Prop), forall (mk : forall (forward : forall (h : @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) zero), @Eq.{v} Vector x vzero), forall (backward : forall (h : @Eq.{v} Vector x vzero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) zero), R), R) => ",
            "fun (dist_sq_nonneg_arg : forall (x : Vector), forall (y : Vector), le_rel zero (@distSq.{u,v} Scalar Vector vadd vneg inner x y)) => ",
            "fun (inner_field23_arg : forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (inner_field24_arg : forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (quadratic_norm_nonneg_arg : forall (x : Vector), forall (y : Vector), forall (t : Scalar), le_rel zero (add (add (mul (@normSq.{u,v} Scalar Vector inner x) (@sq.{u} Scalar mul t)) (mul (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y)) t)) (@normSq.{u,v} Scalar Vector inner y))) => ",
            $($tail)+,
            ")"
        )
    };
}

macro_rules! affine_params {
    ($tail:literal) => {
        concat!(
            "forall (Scalar : Sort u), ",
            "forall (zero : Scalar), ",
            "forall (one : Scalar), ",
            "forall (add : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            "forall (neg : forall (a : Scalar), Scalar), ",
            "forall (sub : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            "forall (mul : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            "forall (le_rel : forall (a : Scalar), forall (b : Scalar), Prop), ",
            "forall (Vector : Sort v), ",
            "forall (vzero : Vector), ",
            "forall (vadd : forall (x : Vector), forall (y : Vector), Vector), ",
            "forall (vneg : forall (x : Vector), Vector), ",
            "forall (smul : forall (a : Scalar), forall (x : Vector), Vector), ",
            "forall (inner : forall (x : Vector), forall (y : Vector), Scalar), ",
            "forall (PointCarrier : Sort p), ",
            "forall (disp_op : forall (A : PointCarrier), forall (B : PointCarrier), Vector), ",
            $tail
        )
    };
}

macro_rules! affine_abs {
    (concat!($($tail:literal),+ $(,)?)) => {
        concat!(
            "fun Scalar => fun zero => fun one => fun add => fun neg => fun sub => fun mul => fun le_rel => fun Vector => fun vzero => fun vadd => fun vneg => fun smul => fun inner => fun PointCarrier => fun disp_op => ",
            $($tail),+
        )
    };
    ($tail:literal) => {
        concat!(
            "fun Scalar => fun zero => fun one => fun add => fun neg => fun sub => fun mul => fun le_rel => fun Vector => fun vzero => fun vadd => fun vneg => fun smul => fun inner => fun PointCarrier => fun disp_op => ",
            $tail
        )
    };
}

macro_rules! abstract_right_triangle_params {
    ($tail:literal) => {
        concat!(
            "forall (Scalar : Sort u), ",
            "forall (zero : Scalar), ",
            "forall (one : Scalar), ",
            "forall (add : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            "forall (neg : forall (a : Scalar), Scalar), ",
            "forall (sub : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            "forall (mul : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            "forall (le_rel : forall (a : Scalar), forall (b : Scalar), Prop), ",
            "forall (Vector : Sort v), ",
            "forall (vzero : Vector), ",
            "forall (vadd : forall (x : Vector), forall (y : Vector), Vector), ",
            "forall (vneg : forall (x : Vector), Vector), ",
            "forall (smul : forall (a : Scalar), forall (x : Vector), Vector), ",
            "forall (inner : forall (x : Vector), forall (y : Vector), Scalar), ",
            "forall (PointCarrier : Sort p), ",
            "forall (disp_op : forall (A : PointCarrier), forall (B : PointCarrier), Vector), ",
            $tail
        )
    };
}

macro_rules! abstract_right_triangle_abs {
    (concat!($($tail:literal),+ $(,)?)) => {
        concat!(
            "fun Scalar => fun zero => fun one => fun add => fun neg => fun sub => fun mul => fun le_rel => fun Vector => fun vzero => fun vadd => fun vneg => fun smul => fun inner => fun PointCarrier => fun disp_op => ",
            $($tail),+
        )
    };
    ($tail:literal) => {
        concat!(
            "fun Scalar => fun zero => fun one => fun add => fun neg => fun sub => fun mul => fun le_rel => fun Vector => fun vzero => fun vadd => fun vneg => fun smul => fun inner => fun PointCarrier => fun disp_op => ",
            $tail
        )
    };
}

macro_rules! abstract_metric_params {
    ($tail:literal) => {
        concat!(
            "forall (Scalar : Sort u), ",
            "forall (zero : Scalar), ",
            "forall (one : Scalar), ",
            "forall (add : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            "forall (neg : forall (a : Scalar), Scalar), ",
            "forall (sub : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            "forall (mul : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            "forall (le_rel : forall (a : Scalar), forall (b : Scalar), Prop), ",
            "forall (lt_rel : forall (a : Scalar), forall (b : Scalar), Prop), ",
            "forall (sqrt_fn : forall (a : Scalar), Scalar), ",
            "forall (Vector : Sort v), ",
            "forall (vzero : Vector), ",
            "forall (vadd : forall (x : Vector), forall (y : Vector), Vector), ",
            "forall (vneg : forall (x : Vector), Vector), ",
            "forall (smul : forall (a : Scalar), forall (x : Vector), Vector), ",
            "forall (inner : forall (x : Vector), forall (y : Vector), Scalar), ",
            "forall (PointCarrier : Sort p), ",
            "forall (disp_op : forall (A : PointCarrier), forall (B : PointCarrier), Vector), ",
            $tail
        )
    };
}

macro_rules! abstract_metric_abs {
    (concat!($($tail:tt)+)) => {
        concat!(
            "fun Scalar => fun zero => fun one => fun add => fun neg => fun sub => fun mul => fun le_rel => fun lt_rel => fun sqrt_fn => fun Vector => fun vzero => fun vadd => fun vneg => fun smul => fun inner => fun PointCarrier => fun disp_op => ",
            $($tail)+
        )
    };
    ($tail:literal) => {
        concat!(
            "fun Scalar => fun zero => fun one => fun add => fun neg => fun sub => fun mul => fun le_rel => fun lt_rel => fun sqrt_fn => fun Vector => fun vzero => fun vadd => fun vneg => fun smul => fun inner => fun PointCarrier => fun disp_op => ",
            $tail
        )
    };
}

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

const ABSTRACT_GROUP_DEFINITIONS: &[DefinitionArtifact] = &[
    DefinitionArtifact {
        name: "GroupLawArgs",
        universe_params: &["u"],
        ty: abstract_group_params!("Prop"),
        value: abstract_group_abs!(concat!(
            "forall (P : Prop), forall (mk : ",
            "forall (mul_assoc_law : forall (a : G), forall (b : G), forall (c : G), @Eq.{u} G (mul (mul a b) c) (mul a (mul b c))), ",
            "forall (one_mul_law : forall (a : G), @Eq.{u} G (mul one a) a), ",
            "forall (mul_one_law : forall (a : G), @Eq.{u} G (mul a one) a), ",
            "forall (inv_mul_law : forall (a : G), @Eq.{u} G (mul (inv a) a) one), ",
            "forall (mul_inv_law : forall (a : G), @Eq.{u} G (mul a (inv a)) one), ",
            "P), P"
        )),
    },
    DefinitionArtifact {
        name: "GroupHomLawArgs",
        universe_params: &["u", "v"],
        ty: abstract_group_hom_params!("Prop"),
        value: abstract_group_hom_abs!(concat!(
            "forall (P : Prop), forall (mk : ",
            "forall (hom_mul_law : forall (a : G), forall (b : G), @Eq.{v} H (f (mulG a b)) (mulH (f a) (f b))), ",
            "forall (hom_one_law : @Eq.{v} H (f oneG) oneH), ",
            "forall (hom_inv_law : forall (a : G), @Eq.{v} H (f (invG a)) (invH (f a))), ",
            "P), P"
        )),
    },
    DefinitionArtifact {
        name: "KernelPred",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (G : Sort u), ",
            "forall (H : Sort v), ",
            "forall (oneH : H), ",
            "forall (f : forall (x : G), H), ",
            "forall (a : G), ",
            "Prop"
        ),
        value:
            "fun G => fun H => fun oneH => fun f => fun a => @Eq.{v} H (f a) oneH",
    },
    DefinitionArtifact {
        name: "KerRel",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (G : Sort u), ",
            "forall (H : Sort v), ",
            "forall (f : forall (x : G), H), ",
            "forall (a : G), ",
            "forall (b : G), ",
            "Prop"
        ),
        value:
            "fun G => fun H => fun f => fun a => fun b => @Eq.{v} H (f a) (f b)",
    },
];

const ABSTRACT_GROUP_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "group_mul_assoc",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (a : G), forall (b : G), forall (c : G), @Eq.{u} G (mul (mul a b) c) (mul a (mul b c))"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun a => fun b => fun c => ",
            "group_args (@Eq.{u} G (mul (mul a b) c) (mul a (mul b c))) ",
            "(fun (mul_assoc_arg : forall (a : G), forall (b : G), forall (c : G), @Eq.{u} G (mul (mul a b) c) (mul a (mul b c))) => ",
            "fun (one_mul_arg : forall (a : G), @Eq.{u} G (mul one a) a) => ",
            "fun (mul_one_arg : forall (a : G), @Eq.{u} G (mul a one) a) => ",
            "fun (inv_mul_arg : forall (a : G), @Eq.{u} G (mul (inv a) a) one) => ",
            "fun (mul_inv_arg : forall (a : G), @Eq.{u} G (mul a (inv a)) one) => ",
            "mul_assoc_arg a b c)"
        )),
    },
    TheoremArtifact {
        name: "group_one_mul",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (a : G), @Eq.{u} G (mul one a) a"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun a => ",
            "group_args (@Eq.{u} G (mul one a) a) ",
            "(fun (mul_assoc_arg : forall (a : G), forall (b : G), forall (c : G), @Eq.{u} G (mul (mul a b) c) (mul a (mul b c))) => ",
            "fun (one_mul_arg : forall (a : G), @Eq.{u} G (mul one a) a) => ",
            "fun (mul_one_arg : forall (a : G), @Eq.{u} G (mul a one) a) => ",
            "fun (inv_mul_arg : forall (a : G), @Eq.{u} G (mul (inv a) a) one) => ",
            "fun (mul_inv_arg : forall (a : G), @Eq.{u} G (mul a (inv a)) one) => ",
            "one_mul_arg a)"
        )),
    },
    TheoremArtifact {
        name: "group_mul_one",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (a : G), @Eq.{u} G (mul a one) a"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun a => ",
            "group_args (@Eq.{u} G (mul a one) a) ",
            "(fun (mul_assoc_arg : forall (a : G), forall (b : G), forall (c : G), @Eq.{u} G (mul (mul a b) c) (mul a (mul b c))) => ",
            "fun (one_mul_arg : forall (a : G), @Eq.{u} G (mul one a) a) => ",
            "fun (mul_one_arg : forall (a : G), @Eq.{u} G (mul a one) a) => ",
            "fun (inv_mul_arg : forall (a : G), @Eq.{u} G (mul (inv a) a) one) => ",
            "fun (mul_inv_arg : forall (a : G), @Eq.{u} G (mul a (inv a)) one) => ",
            "mul_one_arg a)"
        )),
    },
    TheoremArtifact {
        name: "group_inv_mul",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (a : G), @Eq.{u} G (mul (inv a) a) one"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun a => ",
            "group_args (@Eq.{u} G (mul (inv a) a) one) ",
            "(fun (mul_assoc_arg : forall (a : G), forall (b : G), forall (c : G), @Eq.{u} G (mul (mul a b) c) (mul a (mul b c))) => ",
            "fun (one_mul_arg : forall (a : G), @Eq.{u} G (mul one a) a) => ",
            "fun (mul_one_arg : forall (a : G), @Eq.{u} G (mul a one) a) => ",
            "fun (inv_mul_arg : forall (a : G), @Eq.{u} G (mul (inv a) a) one) => ",
            "fun (mul_inv_arg : forall (a : G), @Eq.{u} G (mul a (inv a)) one) => ",
            "inv_mul_arg a)"
        )),
    },
    TheoremArtifact {
        name: "group_mul_inv",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (a : G), @Eq.{u} G (mul a (inv a)) one"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun a => ",
            "group_args (@Eq.{u} G (mul a (inv a)) one) ",
            "(fun (mul_assoc_arg : forall (a : G), forall (b : G), forall (c : G), @Eq.{u} G (mul (mul a b) c) (mul a (mul b c))) => ",
            "fun (one_mul_arg : forall (a : G), @Eq.{u} G (mul one a) a) => ",
            "fun (mul_one_arg : forall (a : G), @Eq.{u} G (mul a one) a) => ",
            "fun (inv_mul_arg : forall (a : G), @Eq.{u} G (mul (inv a) a) one) => ",
            "fun (mul_inv_arg : forall (a : G), @Eq.{u} G (mul a (inv a)) one) => ",
            "mul_inv_arg a)"
        )),
    },
    TheoremArtifact {
        name: "group_left_cancel",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (a : G), forall (b : G), forall (c : G), forall (h : @Eq.{u} G (mul a b) (mul a c)), @Eq.{u} G b c"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun a => fun b => fun c => fun h => ",
            "@eq_trans.{u} G b (mul one b) c ",
            "(@eq_symm.{u} G (mul one b) b (@group_one_mul.{u} G one mul inv group_args b)) ",
            "(@eq_trans.{u} G (mul one b) (mul (mul (inv a) a) b) c ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul z b) one (mul (inv a) a) ",
            "(@eq_symm.{u} G (mul (inv a) a) one (@group_inv_mul.{u} G one mul inv group_args a))) ",
            "(@eq_trans.{u} G (mul (mul (inv a) a) b) (mul (inv a) (mul a b)) c ",
            "(@group_mul_assoc.{u} G one mul inv group_args (inv a) a b) ",
            "(@eq_trans.{u} G (mul (inv a) (mul a b)) (mul (inv a) (mul a c)) c ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul (inv a) z) (mul a b) (mul a c) h) ",
            "(@eq_trans.{u} G (mul (inv a) (mul a c)) (mul (mul (inv a) a) c) c ",
            "(@eq_symm.{u} G (mul (mul (inv a) a) c) (mul (inv a) (mul a c)) ",
            "(@group_mul_assoc.{u} G one mul inv group_args (inv a) a c)) ",
            "(@eq_trans.{u} G (mul (mul (inv a) a) c) (mul one c) c ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul z c) (mul (inv a) a) one ",
            "(@group_inv_mul.{u} G one mul inv group_args a)) ",
            "(@group_one_mul.{u} G one mul inv group_args c))))))"
        )),
    },
    TheoremArtifact {
        name: "group_inv_inv",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (a : G), @Eq.{u} G (inv (inv a)) a"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun a => ",
            "@group_left_cancel.{u} G one mul inv group_args (inv a) (inv (inv a)) a ",
            "(@eq_trans.{u} G (mul (inv a) (inv (inv a))) one (mul (inv a) a) ",
            "(@group_mul_inv.{u} G one mul inv group_args (inv a)) ",
            "(@eq_symm.{u} G (mul (inv a) a) one (@group_inv_mul.{u} G one mul inv group_args a)))"
        )),
    },
    TheoremArtifact {
        name: "group_inv_mul_left_reassoc",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (a : G), forall (b : G), @Eq.{u} G (mul (inv a) (mul a b)) b"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun a => fun b => ",
            "@eq_trans.{u} G (mul (inv a) (mul a b)) (mul (mul (inv a) a) b) b ",
            "(@eq_symm.{u} G (mul (mul (inv a) a) b) (mul (inv a) (mul a b)) ",
            "(@group_mul_assoc.{u} G one mul inv group_args (inv a) a b)) ",
            "(@eq_trans.{u} G (mul (mul (inv a) a) b) (mul one b) b ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul z b) (mul (inv a) a) one ",
            "(@group_inv_mul.{u} G one mul inv group_args a)) ",
            "(@group_one_mul.{u} G one mul inv group_args b))"
        )),
    },
    TheoremArtifact {
        name: "group_conj_slide",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (k : G), forall (n : G), @Eq.{u} G (mul k (mul (mul (inv k) n) k)) (mul n k)"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun k => fun n => ",
            "@eq_trans.{u} G (mul k (mul (mul (inv k) n) k)) (mul (mul k (mul (inv k) n)) k) (mul n k) ",
            "(@eq_symm.{u} G (mul (mul k (mul (inv k) n)) k) (mul k (mul (mul (inv k) n) k)) ",
            "(@group_mul_assoc.{u} G one mul inv group_args k (mul (inv k) n) k)) ",
            "(@eq_trans.{u} G (mul (mul k (mul (inv k) n)) k) (mul (mul (mul k (inv k)) n) k) (mul n k) ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul z k) (mul k (mul (inv k) n)) (mul (mul k (inv k)) n) ",
            "(@eq_symm.{u} G (mul (mul k (inv k)) n) (mul k (mul (inv k) n)) ",
            "(@group_mul_assoc.{u} G one mul inv group_args k (inv k) n))) ",
            "(@eq_trans.{u} G (mul (mul (mul k (inv k)) n) k) (mul (mul one n) k) (mul n k) ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul (mul z n) k) (mul k (inv k)) one ",
            "(@group_mul_inv.{u} G one mul inv group_args k)) ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul z k) (mul one n) n ",
            "(@group_one_mul.{u} G one mul inv group_args n))))"
        )),
    },
    TheoremArtifact {
        name: "group_product_mul_reassoc",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (h : G), forall (n : G), forall (k : G), forall (m : G), @Eq.{u} G (mul (mul h k) (mul (mul (mul (inv k) n) k) m)) (mul (mul h n) (mul k m))"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun h => fun n => fun k => fun m => ",
            "@eq_trans.{u} G (mul (mul h k) (mul (mul (mul (inv k) n) k) m)) (mul (mul (mul h k) (mul (mul (inv k) n) k)) m) (mul (mul h n) (mul k m)) ",
            "(@eq_symm.{u} G (mul (mul (mul h k) (mul (mul (inv k) n) k)) m) (mul (mul h k) (mul (mul (mul (inv k) n) k) m)) ",
            "(@group_mul_assoc.{u} G one mul inv group_args (mul h k) (mul (mul (inv k) n) k) m)) ",
            "(@eq_trans.{u} G (mul (mul (mul h k) (mul (mul (inv k) n) k)) m) (mul (mul h (mul k (mul (mul (inv k) n) k))) m) (mul (mul h n) (mul k m)) ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul z m) (mul (mul h k) (mul (mul (inv k) n) k)) (mul h (mul k (mul (mul (inv k) n) k))) ",
            "(@group_mul_assoc.{u} G one mul inv group_args h k (mul (mul (inv k) n) k))) ",
            "(@eq_trans.{u} G (mul (mul h (mul k (mul (mul (inv k) n) k))) m) (mul (mul h (mul n k)) m) (mul (mul h n) (mul k m)) ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul (mul h z) m) (mul k (mul (mul (inv k) n) k)) (mul n k) ",
            "(@group_conj_slide.{u} G one mul inv group_args k n)) ",
            "(@eq_trans.{u} G (mul (mul h (mul n k)) m) (mul (mul (mul h n) k) m) (mul (mul h n) (mul k m)) ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul z m) (mul h (mul n k)) (mul (mul h n) k) ",
            "(@eq_symm.{u} G (mul (mul h n) k) (mul h (mul n k)) ",
            "(@group_mul_assoc.{u} G one mul inv group_args h n k))) ",
            "(@group_mul_assoc.{u} G one mul inv group_args (mul h n) k m))))"
        )),
    },
    TheoremArtifact {
        name: "group_mul_inv_rev",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (a : G), forall (b : G), @Eq.{u} G (mul (inv b) (inv a)) (inv (mul a b))"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun a => fun b => ",
            "@group_left_cancel.{u} G one mul inv group_args (mul a b) (mul (inv b) (inv a)) (inv (mul a b)) ",
            "(@eq_trans.{u} G (mul (mul a b) (mul (inv b) (inv a))) one (mul (mul a b) (inv (mul a b))) ",
            "(@eq_trans.{u} G (mul (mul a b) (mul (inv b) (inv a))) (mul (mul (mul a b) (inv b)) (inv a)) one ",
            "(@eq_symm.{u} G (mul (mul (mul a b) (inv b)) (inv a)) (mul (mul a b) (mul (inv b) (inv a))) ",
            "(@group_mul_assoc.{u} G one mul inv group_args (mul a b) (inv b) (inv a))) ",
            "(@eq_trans.{u} G (mul (mul (mul a b) (inv b)) (inv a)) (mul (mul a (mul b (inv b))) (inv a)) one ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul z (inv a)) (mul (mul a b) (inv b)) (mul a (mul b (inv b))) ",
            "(@group_mul_assoc.{u} G one mul inv group_args a b (inv b))) ",
            "(@eq_trans.{u} G (mul (mul a (mul b (inv b))) (inv a)) (mul (mul a one) (inv a)) one ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul (mul a z) (inv a)) (mul b (inv b)) one ",
            "(@group_mul_inv.{u} G one mul inv group_args b)) ",
            "(@eq_trans.{u} G (mul (mul a one) (inv a)) (mul a (inv a)) one ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul z (inv a)) (mul a one) a ",
            "(@group_mul_one.{u} G one mul inv group_args a)) ",
            "(@group_mul_inv.{u} G one mul inv group_args a))))) ",
            "(@eq_symm.{u} G (mul (mul a b) (inv (mul a b))) one ",
            "(@group_mul_inv.{u} G one mul inv group_args (mul a b))))"
        )),
    },
    TheoremArtifact {
        name: "group_product_inv_reassoc",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (h : G), forall (n : G), @Eq.{u} G (mul (inv h) (mul (mul h (inv n)) (inv h))) (inv (mul h n))"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun h => fun n => ",
            "@eq_trans.{u} G (mul (inv h) (mul (mul h (inv n)) (inv h))) (mul (mul (inv h) (mul h (inv n))) (inv h)) (inv (mul h n)) ",
            "(@eq_symm.{u} G (mul (mul (inv h) (mul h (inv n))) (inv h)) (mul (inv h) (mul (mul h (inv n)) (inv h))) ",
            "(@group_mul_assoc.{u} G one mul inv group_args (inv h) (mul h (inv n)) (inv h))) ",
            "(@eq_trans.{u} G (mul (mul (inv h) (mul h (inv n))) (inv h)) (mul (mul (mul (inv h) h) (inv n)) (inv h)) (inv (mul h n)) ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul z (inv h)) (mul (inv h) (mul h (inv n))) (mul (mul (inv h) h) (inv n)) ",
            "(@eq_symm.{u} G (mul (mul (inv h) h) (inv n)) (mul (inv h) (mul h (inv n))) ",
            "(@group_mul_assoc.{u} G one mul inv group_args (inv h) h (inv n)))) ",
            "(@eq_trans.{u} G (mul (mul (mul (inv h) h) (inv n)) (inv h)) (mul (mul one (inv n)) (inv h)) (inv (mul h n)) ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul (mul z (inv n)) (inv h)) (mul (inv h) h) one ",
            "(@group_inv_mul.{u} G one mul inv group_args h)) ",
            "(@eq_trans.{u} G (mul (mul one (inv n)) (inv h)) (mul (inv n) (inv h)) (inv (mul h n)) ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul z (inv h)) (mul one (inv n)) (inv n) ",
            "(@group_one_mul.{u} G one mul inv group_args (inv n))) ",
            "(@group_mul_inv_rev.{u} G one mul inv group_args h n))))"
        )),
    },
    TheoremArtifact {
        name: "group_inv_rel_symm_reassoc",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (a : G), forall (b : G), @Eq.{u} G (inv (mul (inv a) b)) (mul (inv b) a)"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun a => fun b => ",
            "@eq_trans.{u} G (inv (mul (inv a) b)) (mul (inv b) (inv (inv a))) (mul (inv b) a) ",
            "(@eq_symm.{u} G (mul (inv b) (inv (inv a))) (inv (mul (inv a) b)) ",
            "(@group_mul_inv_rev.{u} G one mul inv group_args (inv a) b)) ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul (inv b) z) (inv (inv a)) a ",
            "(@group_inv_inv.{u} G one mul inv group_args a))"
        )),
    },
    TheoremArtifact {
        name: "group_rel_trans_reassoc",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (a : G), forall (b : G), forall (c : G), @Eq.{u} G (mul (mul (inv a) b) (mul (inv b) c)) (mul (inv a) c)"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun a => fun b => fun c => ",
            "@eq_trans.{u} G (mul (mul (inv a) b) (mul (inv b) c)) (mul (mul (mul (inv a) b) (inv b)) c) (mul (inv a) c) ",
            "(@eq_symm.{u} G (mul (mul (mul (inv a) b) (inv b)) c) (mul (mul (inv a) b) (mul (inv b) c)) ",
            "(@group_mul_assoc.{u} G one mul inv group_args (mul (inv a) b) (inv b) c)) ",
            "(@eq_trans.{u} G (mul (mul (mul (inv a) b) (inv b)) c) (mul (mul (inv a) (mul b (inv b))) c) (mul (inv a) c) ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul z c) (mul (mul (inv a) b) (inv b)) (mul (inv a) (mul b (inv b))) ",
            "(@group_mul_assoc.{u} G one mul inv group_args (inv a) b (inv b))) ",
            "(@eq_trans.{u} G (mul (mul (inv a) (mul b (inv b))) c) (mul (mul (inv a) one) c) (mul (inv a) c) ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul (mul (inv a) z) c) (mul b (inv b)) one ",
            "(@group_mul_inv.{u} G one mul inv group_args b)) ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul z c) (mul (inv a) one) (inv a) ",
            "(@group_mul_one.{u} G one mul inv group_args (inv a)))))"
        )),
    },
    TheoremArtifact {
        name: "group_rel_mul_reassoc",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (a : G), forall (a2 : G), forall (b : G), forall (b2 : G), @Eq.{u} G (mul (mul (mul (inv b) (mul (inv a) a2)) b) (mul (inv b) b2)) (mul (inv (mul a b)) (mul a2 b2))"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun a => fun a2 => fun b => fun b2 => ",
            "@eq_trans.{u} G ",
            "(mul (mul (mul (inv b) (mul (inv a) a2)) b) (mul (inv b) b2)) ",
            "(mul (mul (inv b) (mul (inv a) a2)) (mul b (mul (inv b) b2))) ",
            "(mul (inv (mul a b)) (mul a2 b2)) ",
            "(@group_mul_assoc.{u} G one mul inv group_args (mul (inv b) (mul (inv a) a2)) b (mul (inv b) b2)) ",
            "(@eq_trans.{u} G ",
            "(mul (mul (inv b) (mul (inv a) a2)) (mul b (mul (inv b) b2))) ",
            "(mul (mul (inv b) (mul (inv a) a2)) (mul (mul b (inv b)) b2)) ",
            "(mul (inv (mul a b)) (mul a2 b2)) ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul (mul (inv b) (mul (inv a) a2)) z) ",
            "(mul b (mul (inv b) b2)) (mul (mul b (inv b)) b2) ",
            "(@eq_symm.{u} G (mul (mul b (inv b)) b2) (mul b (mul (inv b) b2)) ",
            "(@group_mul_assoc.{u} G one mul inv group_args b (inv b) b2))) ",
            "(@eq_trans.{u} G ",
            "(mul (mul (inv b) (mul (inv a) a2)) (mul (mul b (inv b)) b2)) ",
            "(mul (mul (inv b) (mul (inv a) a2)) (mul one b2)) ",
            "(mul (inv (mul a b)) (mul a2 b2)) ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul (mul (inv b) (mul (inv a) a2)) (mul z b2)) ",
            "(mul b (inv b)) one ",
            "(@group_mul_inv.{u} G one mul inv group_args b)) ",
            "(@eq_trans.{u} G ",
            "(mul (mul (inv b) (mul (inv a) a2)) (mul one b2)) ",
            "(mul (mul (inv b) (mul (inv a) a2)) b2) ",
            "(mul (inv (mul a b)) (mul a2 b2)) ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul (mul (inv b) (mul (inv a) a2)) z) ",
            "(mul one b2) b2 ",
            "(@group_one_mul.{u} G one mul inv group_args b2)) ",
            "(@eq_trans.{u} G ",
            "(mul (mul (inv b) (mul (inv a) a2)) b2) ",
            "(mul (mul (mul (inv b) (inv a)) a2) b2) ",
            "(mul (inv (mul a b)) (mul a2 b2)) ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul z b2) ",
            "(mul (inv b) (mul (inv a) a2)) (mul (mul (inv b) (inv a)) a2) ",
            "(@eq_symm.{u} G (mul (mul (inv b) (inv a)) a2) (mul (inv b) (mul (inv a) a2)) ",
            "(@group_mul_assoc.{u} G one mul inv group_args (inv b) (inv a) a2))) ",
            "(@eq_trans.{u} G ",
            "(mul (mul (mul (inv b) (inv a)) a2) b2) ",
            "(mul (mul (inv b) (inv a)) (mul a2 b2)) ",
            "(mul (inv (mul a b)) (mul a2 b2)) ",
            "(@group_mul_assoc.{u} G one mul inv group_args (mul (inv b) (inv a)) a2 b2) ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul z (mul a2 b2)) ",
            "(mul (inv b) (inv a)) (inv (mul a b)) ",
            "(@group_mul_inv_rev.{u} G one mul inv group_args a b)))))))"
        )),
    },
    TheoremArtifact {
        name: "group_rel_inv_reassoc",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (a : G), forall (b : G), @Eq.{u} G (mul (mul a (mul (inv b) a)) (inv a)) (mul (inv (inv a)) (inv b))"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun a => fun b => ",
            "@eq_trans.{u} G ",
            "(mul (mul a (mul (inv b) a)) (inv a)) ",
            "(mul (mul (mul a (inv b)) a) (inv a)) ",
            "(mul (inv (inv a)) (inv b)) ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul z (inv a)) ",
            "(mul a (mul (inv b) a)) (mul (mul a (inv b)) a) ",
            "(@eq_symm.{u} G (mul (mul a (inv b)) a) (mul a (mul (inv b) a)) ",
            "(@group_mul_assoc.{u} G one mul inv group_args a (inv b) a))) ",
            "(@eq_trans.{u} G ",
            "(mul (mul (mul a (inv b)) a) (inv a)) ",
            "(mul (mul a (inv b)) (mul a (inv a))) ",
            "(mul (inv (inv a)) (inv b)) ",
            "(@group_mul_assoc.{u} G one mul inv group_args (mul a (inv b)) a (inv a)) ",
            "(@eq_trans.{u} G ",
            "(mul (mul a (inv b)) (mul a (inv a))) ",
            "(mul (mul a (inv b)) one) ",
            "(mul (inv (inv a)) (inv b)) ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul (mul a (inv b)) z) ",
            "(mul a (inv a)) one ",
            "(@group_mul_inv.{u} G one mul inv group_args a)) ",
            "(@eq_trans.{u} G ",
            "(mul (mul a (inv b)) one) ",
            "(mul a (inv b)) ",
            "(mul (inv (inv a)) (inv b)) ",
            "(@group_mul_one.{u} G one mul inv group_args (mul a (inv b))) ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul z (inv b)) ",
            "a (inv (inv a)) ",
            "(@eq_symm.{u} G (inv (inv a)) a ",
            "(@group_inv_inv.{u} G one mul inv group_args a))))))"
        )),
    },
    TheoremArtifact {
        name: "hom_mul",
        universe_params: &["u", "v"],
        statement: abstract_group_hom_params!(
            "forall (hom_args : @GroupHomLawArgs.{u,v} G oneG mulG invG H oneH mulH invH f), forall (a : G), forall (b : G), @Eq.{v} H (f (mulG a b)) (mulH (f a) (f b))"
        ),
        proof: abstract_group_hom_abs!(concat!(
            "fun hom_args => fun a => fun b => ",
            "hom_args (@Eq.{v} H (f (mulG a b)) (mulH (f a) (f b))) ",
            "(fun (hom_mul_arg : forall (a : G), forall (b : G), @Eq.{v} H (f (mulG a b)) (mulH (f a) (f b))) => ",
            "fun (hom_one_arg : @Eq.{v} H (f oneG) oneH) => ",
            "fun (hom_inv_arg : forall (a : G), @Eq.{v} H (f (invG a)) (invH (f a))) => ",
            "hom_mul_arg a b)"
        )),
    },
    TheoremArtifact {
        name: "hom_one",
        universe_params: &["u", "v"],
        statement: abstract_group_hom_params!(
            "forall (hom_args : @GroupHomLawArgs.{u,v} G oneG mulG invG H oneH mulH invH f), @Eq.{v} H (f oneG) oneH"
        ),
        proof: abstract_group_hom_abs!(concat!(
            "fun hom_args => ",
            "hom_args (@Eq.{v} H (f oneG) oneH) ",
            "(fun (hom_mul_arg : forall (a : G), forall (b : G), @Eq.{v} H (f (mulG a b)) (mulH (f a) (f b))) => ",
            "fun (hom_one_arg : @Eq.{v} H (f oneG) oneH) => ",
            "fun (hom_inv_arg : forall (a : G), @Eq.{v} H (f (invG a)) (invH (f a))) => ",
            "hom_one_arg)"
        )),
    },
    TheoremArtifact {
        name: "hom_inv",
        universe_params: &["u", "v"],
        statement: abstract_group_hom_params!(
            "forall (hom_args : @GroupHomLawArgs.{u,v} G oneG mulG invG H oneH mulH invH f), forall (a : G), @Eq.{v} H (f (invG a)) (invH (f a))"
        ),
        proof: abstract_group_hom_abs!(concat!(
            "fun hom_args => fun a => ",
            "hom_args (@Eq.{v} H (f (invG a)) (invH (f a))) ",
            "(fun (hom_mul_arg : forall (a : G), forall (b : G), @Eq.{v} H (f (mulG a b)) (mulH (f a) (f b))) => ",
            "fun (hom_one_arg : @Eq.{v} H (f oneG) oneH) => ",
            "fun (hom_inv_arg : forall (a : G), @Eq.{v} H (f (invG a)) (invH (f a))) => ",
            "hom_inv_arg a)"
        )),
    },
    TheoremArtifact {
        name: "kernel_one",
        universe_params: &["u", "v"],
        statement: abstract_group_hom_params!(
            "forall (hom_args : @GroupHomLawArgs.{u,v} G oneG mulG invG H oneH mulH invH f), @KernelPred.{u,v} G H oneH f oneG"
        ),
        proof: abstract_group_hom_abs!(
            "fun hom_args => @hom_one.{u,v} G oneG mulG invG H oneH mulH invH f hom_args"
        ),
    },
    TheoremArtifact {
        name: "ker_rel_refl",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort u), forall (H : Sort v), ",
            "forall (f : forall (x : G), H), ",
            "forall (a : G), ",
            "@KerRel.{u,v} G H f a a"
        ),
        proof: "fun G => fun H => fun f => fun a => @Eq.refl.{v} H (f a)",
    },
    TheoremArtifact {
        name: "ker_rel_symm",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort u), forall (H : Sort v), ",
            "forall (f : forall (x : G), H), ",
            "forall (a : G), forall (b : G), ",
            "forall (h : @KerRel.{u,v} G H f a b), ",
            "@KerRel.{u,v} G H f b a"
        ),
        proof:
            "fun G => fun H => fun f => fun a => fun b => fun h => @eq_symm.{v} H (f a) (f b) h",
    },
    TheoremArtifact {
        name: "ker_rel_trans",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort u), forall (H : Sort v), ",
            "forall (f : forall (x : G), H), ",
            "forall (a : G), forall (b : G), forall (c : G), ",
            "forall (hab : @KerRel.{u,v} G H f a b), ",
            "forall (hbc : @KerRel.{u,v} G H f b c), ",
            "@KerRel.{u,v} G H f a c"
        ),
        proof:
            "fun G => fun H => fun f => fun a => fun b => fun c => fun hab => fun hbc => @eq_trans.{v} H (f a) (f b) (f c) hab hbc",
    },
];

const ABSTRACT_GROUP_KERNEL_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "kernel_mul_closed",
        universe_params: &["u", "v"],
        statement: abstract_group_hom_params!(
            "forall (groupH_args : @GroupLawArgs.{v} H oneH mulH invH), forall (hom_args : @GroupHomLawArgs.{u,v} G oneG mulG invG H oneH mulH invH f), forall (a : G), forall (b : G), forall (ha : @KernelPred.{u,v} G H oneH f a), forall (hb : @KernelPred.{u,v} G H oneH f b), @KernelPred.{u,v} G H oneH f (mulG a b)"
        ),
        proof: abstract_group_hom_abs!(concat!(
            "fun groupH_args => fun hom_args => fun a => fun b => fun ha => fun hb => ",
            "@eq_trans.{v} H (f (mulG a b)) (mulH (f a) (f b)) oneH ",
            "(@hom_mul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args a b) ",
            "(@eq_trans.{v} H (mulH (f a) (f b)) (mulH oneH oneH) oneH ",
            "(@eq_congr2.{v,v,v} H H H mulH (f a) oneH (f b) oneH ha hb) ",
            "(@group_one_mul.{v} H oneH mulH invH groupH_args oneH))"
        )),
    },
    TheoremArtifact {
        name: "kernel_inv_closed",
        universe_params: &["u", "v"],
        statement: abstract_group_hom_params!(
            "forall (groupH_args : @GroupLawArgs.{v} H oneH mulH invH), forall (hom_args : @GroupHomLawArgs.{u,v} G oneG mulG invG H oneH mulH invH f), forall (a : G), forall (ha : @KernelPred.{u,v} G H oneH f a), @KernelPred.{u,v} G H oneH f (invG a)"
        ),
        proof: abstract_group_hom_abs!(concat!(
            "fun groupH_args => fun hom_args => fun a => fun ha => ",
            "@eq_trans.{v} H (f (invG a)) (invH (f a)) oneH ",
            "(@hom_inv.{u,v} G oneG mulG invG H oneH mulH invH f hom_args a) ",
            "(@eq_trans.{v} H (invH (f a)) (invH oneH) oneH ",
            "(@eq_congr_arg.{v,v} H H invH (f a) oneH ha) ",
            "(@eq_trans.{v} H (invH oneH) (mulH (invH oneH) oneH) oneH ",
            "(@eq_symm.{v} H (mulH (invH oneH) oneH) (invH oneH) ",
            "(@group_mul_one.{v} H oneH mulH invH groupH_args (invH oneH))) ",
            "(@group_inv_mul.{v} H oneH mulH invH groupH_args oneH)))"
        )),
    },
    TheoremArtifact {
        name: "kernel_conj_closed",
        universe_params: &["u", "v"],
        statement: abstract_group_hom_params!(
            "forall (groupH_args : @GroupLawArgs.{v} H oneH mulH invH), forall (hom_args : @GroupHomLawArgs.{u,v} G oneG mulG invG H oneH mulH invH f), forall (g : G), forall (a : G), forall (ha : @KernelPred.{u,v} G H oneH f a), @KernelPred.{u,v} G H oneH f (mulG (mulG g a) (invG g))"
        ),
        proof: abstract_group_hom_abs!(concat!(
            "fun groupH_args => fun hom_args => fun g => fun a => fun ha => ",
            "@eq_trans.{v} H (f (mulG (mulG g a) (invG g))) (mulH (f (mulG g a)) (f (invG g))) oneH ",
            "(@hom_mul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args (mulG g a) (invG g)) ",
            "(@eq_trans.{v} H (mulH (f (mulG g a)) (f (invG g))) (mulH (mulH (f g) (f a)) (invH (f g))) oneH ",
            "(@eq_congr2.{v,v,v} H H H mulH (f (mulG g a)) (mulH (f g) (f a)) (f (invG g)) (invH (f g)) ",
            "(@hom_mul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args g a) ",
            "(@hom_inv.{u,v} G oneG mulG invG H oneH mulH invH f hom_args g)) ",
            "(@eq_trans.{v} H (mulH (mulH (f g) (f a)) (invH (f g))) (mulH (mulH (f g) oneH) (invH (f g))) oneH ",
            "(@eq_congr_arg.{v,v} H H (fun (z : H) => mulH (mulH (f g) z) (invH (f g))) (f a) oneH ha) ",
            "(@eq_trans.{v} H (mulH (mulH (f g) oneH) (invH (f g))) (mulH (f g) (mulH oneH (invH (f g)))) oneH ",
            "(@group_mul_assoc.{v} H oneH mulH invH groupH_args (f g) oneH (invH (f g))) ",
            "(@eq_trans.{v} H (mulH (f g) (mulH oneH (invH (f g)))) (mulH (f g) (invH (f g))) oneH ",
            "(@eq_congr_arg.{v,v} H H (fun (z : H) => mulH (f g) z) (mulH oneH (invH (f g))) (invH (f g)) ",
            "(@group_one_mul.{v} H oneH mulH invH groupH_args (invH (f g)))) ",
            "(@group_mul_inv.{v} H oneH mulH invH groupH_args (f g))))))"
        )),
    },
];

const ABSTRACT_GROUP_IMAGE_DEFINITIONS: &[DefinitionArtifact] = &[DefinitionArtifact {
    name: "ImagePred",
    universe_params: &["u", "v"],
    ty: concat!(
        "forall (G : Sort u), ",
        "forall (H : Sort v), ",
        "forall (f : forall (x : G), H), ",
        "forall (y : H), ",
        "Prop"
    ),
    value: concat!(
        "fun G => fun H => fun f => fun y => ",
        "forall (P : Prop), ",
        "forall (mk : forall (a : G), forall (h : @Eq.{v} H (f a) y), P), ",
        "P"
    ),
}];

const ABSTRACT_GROUP_IMAGE_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "image_intro",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort u), ",
            "forall (H : Sort v), ",
            "forall (f : forall (x : G), H), ",
            "forall (a : G), ",
            "@ImagePred.{u,v} G H f (f a)"
        ),
        proof: concat!(
            "fun G => fun H => fun f => fun a => ",
            "fun (P : Prop) => ",
            "fun (mk : forall (b : G), forall (h : @Eq.{v} H (f b) (f a)), P) => ",
            "mk a (@Eq.refl.{v} H (f a))"
        ),
    },
    TheoremArtifact {
        name: "image_elim",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort u), ",
            "forall (H : Sort v), ",
            "forall (f : forall (x : G), H), ",
            "forall (y : H), ",
            "forall (img : @ImagePred.{u,v} G H f y), ",
            "forall (P : Prop), ",
            "forall (mk : forall (a : G), forall (h : @Eq.{v} H (f a) y), P), ",
            "P"
        ),
        proof: "fun G => fun H => fun f => fun y => fun img => fun P => fun mk => img P mk",
    },
    TheoremArtifact {
        name: "image_one",
        universe_params: &["u", "v"],
        statement: abstract_group_hom_params!(
            "forall (hom_args : @GroupHomLawArgs.{u,v} G oneG mulG invG H oneH mulH invH f), @ImagePred.{u,v} G H f oneH"
        ),
        proof: abstract_group_hom_abs!(concat!(
            "fun hom_args => fun (P : Prop) => ",
            "fun (mk : forall (a : G), forall (h : @Eq.{v} H (f a) oneH), P) => ",
            "mk oneG (@hom_one.{u,v} G oneG mulG invG H oneH mulH invH f hom_args)"
        )),
    },
    TheoremArtifact {
        name: "image_mul_closed",
        universe_params: &["u", "v"],
        statement: abstract_group_hom_params!(
            "forall (hom_args : @GroupHomLawArgs.{u,v} G oneG mulG invG H oneH mulH invH f), forall (x : H), forall (y : H), forall (hx : @ImagePred.{u,v} G H f x), forall (hy : @ImagePred.{u,v} G H f y), @ImagePred.{u,v} G H f (mulH x y)"
        ),
        proof: abstract_group_hom_abs!(concat!(
            "fun hom_args => fun x => fun y => fun hx => fun hy => ",
            "hx (@ImagePred.{u,v} G H f (mulH x y)) ",
            "(fun (a : G) => fun (hax : @Eq.{v} H (f a) x) => ",
            "hy (@ImagePred.{u,v} G H f (mulH x y)) ",
            "(fun (b : G) => fun (hby : @Eq.{v} H (f b) y) => ",
            "fun (P : Prop) => ",
            "fun (mk : forall (c : G), forall (h : @Eq.{v} H (f c) (mulH x y)), P) => ",
            "mk (mulG a b) ",
            "(@eq_trans.{v} H (f (mulG a b)) (mulH (f a) (f b)) (mulH x y) ",
            "(@hom_mul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args a b) ",
            "(@eq_congr2.{v,v,v} H H H mulH (f a) x (f b) y hax hby))))"
        )),
    },
    TheoremArtifact {
        name: "image_inv_closed",
        universe_params: &["u", "v"],
        statement: abstract_group_hom_params!(
            "forall (hom_args : @GroupHomLawArgs.{u,v} G oneG mulG invG H oneH mulH invH f), forall (y : H), forall (hy : @ImagePred.{u,v} G H f y), @ImagePred.{u,v} G H f (invH y)"
        ),
        proof: abstract_group_hom_abs!(concat!(
            "fun hom_args => fun y => fun hy => ",
            "hy (@ImagePred.{u,v} G H f (invH y)) ",
            "(fun (a : G) => fun (hay : @Eq.{v} H (f a) y) => ",
            "fun (P : Prop) => ",
            "fun (mk : forall (c : G), forall (h : @Eq.{v} H (f c) (invH y)), P) => ",
            "mk (invG a) ",
            "(@eq_trans.{v} H (f (invG a)) (invH (f a)) (invH y) ",
            "(@hom_inv.{u,v} G oneG mulG invG H oneH mulH invH f hom_args a) ",
            "(@eq_congr_arg.{v,v} H H invH (f a) y hay)))"
        )),
    },
];

const ABSTRACT_GROUP_QUOTIENT_DEFINITIONS: &[DefinitionArtifact] = &[
    DefinitionArtifact {
        name: "KerSetoid",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (H : Sort succ v), ",
            "forall (f : forall (x : G), H), ",
            "@Setoid.{u} G"
        ),
        value: concat!(
            "fun G => fun H => fun f => ",
            "@Setoid.mk.{u} G (@KerRel.{succ u,succ v} G H f) ",
            "(fun (P : Prop) => ",
            "fun (mk_equiv : ",
            "forall (refl_arg : forall (x : G), @KerRel.{succ u,succ v} G H f x x), ",
            "forall (symm_arg : forall (x : G), forall (y : G), forall (p : @KerRel.{succ u,succ v} G H f x y), @KerRel.{succ u,succ v} G H f y x), ",
            "forall (trans_arg : forall (x : G), forall (y : G), forall (z : G), forall (p : @KerRel.{succ u,succ v} G H f x y), forall (q : @KerRel.{succ u,succ v} G H f y z), @KerRel.{succ u,succ v} G H f x z), ",
            "P) => ",
            "mk_equiv ",
            "(@ker_rel_refl.{succ u,succ v} G H f) ",
            "(@ker_rel_symm.{succ u,succ v} G H f) ",
            "(@ker_rel_trans.{succ u,succ v} G H f))"
        ),
    },
    DefinitionArtifact {
        name: "KerQuot",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (H : Sort succ v), ",
            "forall (f : forall (x : G), H), ",
            "Sort succ u"
        ),
        value:
            "fun G => fun H => fun f => @Quotient.{u} G (@KerSetoid.{u,v} G H f)",
    },
    DefinitionArtifact {
        name: "KerQuotMk",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (H : Sort succ v), ",
            "forall (f : forall (x : G), H), ",
            "forall (a : G), ",
            "@KerQuot.{u,v} G H f"
        ),
        value: concat!(
            "fun G => fun H => fun f => fun a => ",
            "@Quotient.mk.{u} G (@KerSetoid.{u,v} G H f) a"
        ),
    },
    DefinitionArtifact {
        name: "KerQuotToH",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (H : Sort succ v), ",
            "forall (f : forall (x : G), H), ",
            "forall (q : @KerQuot.{u,v} G H f), ",
            "H"
        ),
        value: concat!(
            "fun G => fun H => fun f => ",
            "@Quotient.lift.{u,v} G H (@KerSetoid.{u,v} G H f) f ",
            "(fun (a : G) => fun (b : G) => ",
            "fun (h : @Setoid.r.{u} G (@KerSetoid.{u,v} G H f) a b) => h)"
        ),
    },
];

const ABSTRACT_GROUP_QUOTIENT_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "ker_quot_sound",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (H : Sort succ v), ",
            "forall (f : forall (x : G), H), ",
            "forall (a : G), forall (b : G), ",
            "forall (h : @KerRel.{succ u,succ v} G H f a b), ",
            "@Eq.{succ u} (@KerQuot.{u,v} G H f) (@KerQuotMk.{u,v} G H f a) (@KerQuotMk.{u,v} G H f b)"
        ),
        proof: concat!(
            "fun G => fun H => fun f => fun a => fun b => fun h => ",
            "@Quotient.sound.{u} G (@KerSetoid.{u,v} G H f) a b h"
        ),
    },
    TheoremArtifact {
        name: "ker_quot_to_h_mk",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (H : Sort succ v), ",
            "forall (f : forall (x : G), H), ",
            "forall (a : G), ",
            "@Eq.{succ v} H (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f a)) (f a)"
        ),
        proof: "fun G => fun H => fun f => fun a => @Eq.refl.{succ v} H (f a)",
    },
    TheoremArtifact {
        name: "ker_quot_to_h_mul_mk",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "forall (a : G), forall (b : G), ",
            "@Eq.{succ v} H ",
            "(@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f (mulG a b))) ",
            "(mulH (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f a)) (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f b)))"
        ),
        proof: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => ",
            "fun hom_args => fun a => fun b => ",
            "@hom_mul.{succ u,succ v} G oneG mulG invG H oneH mulH invH f hom_args a b"
        ),
    },
];

const ABSTRACT_GROUP_QUOTIENT_MUL_DEFINITIONS: &[DefinitionArtifact] = &[DefinitionArtifact {
    name: "KerQuotMulRep",
    universe_params: &["u", "v"],
    ty: concat!(
        "forall (G : Sort succ u), ",
        "forall (mulG : forall (a : G), forall (b : G), G), ",
        "forall (H : Sort succ v), ",
        "forall (f : forall (x : G), H), ",
        "forall (a : G), forall (b : G), ",
        "@KerQuot.{u,v} G H f"
    ),
    value: concat!(
        "fun G => fun mulG => fun H => fun f => fun a => fun b => ",
        "@KerQuotMk.{u,v} G H f (mulG a b)"
    ),
}];

const ABSTRACT_GROUP_QUOTIENT_MUL_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "ker_quot_mul_rep_compat",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "forall (a : G), forall (a2 : G), ",
            "forall (b : G), forall (b2 : G), ",
            "forall (ha : @KerRel.{succ u,succ v} G H f a a2), ",
            "forall (hb : @KerRel.{succ u,succ v} G H f b b2), ",
            "@Eq.{succ u} (@KerQuot.{u,v} G H f) ",
            "(@KerQuotMulRep.{u,v} G mulG H f a b) ",
            "(@KerQuotMulRep.{u,v} G mulG H f a2 b2)"
        ),
        proof: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => ",
            "fun hom_args => fun a => fun a2 => fun b => fun b2 => fun ha => fun hb => ",
            "@ker_quot_sound.{u,v} G H f (mulG a b) (mulG a2 b2) ",
            "(@eq_trans.{succ v} H (f (mulG a b)) (mulH (f a) (f b)) (f (mulG a2 b2)) ",
            "(@hom_mul.{succ u,succ v} G oneG mulG invG H oneH mulH invH f hom_args a b) ",
            "(@eq_trans.{succ v} H (mulH (f a) (f b)) (mulH (f a2) (f b2)) (f (mulG a2 b2)) ",
            "(@eq_congr2.{succ v,succ v,succ v} H H H mulH (f a) (f a2) (f b) (f b2) ha hb) ",
            "(@eq_symm.{succ v} H (f (mulG a2 b2)) (mulH (f a2) (f b2)) ",
            "(@hom_mul.{succ u,succ v} G oneG mulG invG H oneH mulH invH f hom_args a2 b2))))"
        ),
    },
];

const ABSTRACT_GROUP_QUOTIENT_GROUP_DEFINITIONS: &[DefinitionArtifact] = &[
    DefinitionArtifact {
        name: "KerQuotMul",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "forall (q1 : @KerQuot.{u,v} G H f), ",
            "forall (q2 : @KerQuot.{u,v} G H f), ",
            "@KerQuot.{u,v} G H f"
        ),
        value: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => fun hom_args => ",
            "@Quotient.lift2.{u,u} G (@KerQuot.{u,v} G H f) (@KerSetoid.{u,v} G H f) ",
            "(@KerQuotMulRep.{u,v} G mulG H f) ",
            "(fun (a : G) => fun (a2 : G) => fun (b : G) => fun (b2 : G) => ",
            "fun (ha : @Setoid.r.{u} G (@KerSetoid.{u,v} G H f) a a2) => ",
            "fun (hb : @Setoid.r.{u} G (@KerSetoid.{u,v} G H f) b b2) => ",
            "@ker_quot_mul_rep_compat.{u,v} G oneG mulG invG H oneH mulH invH f hom_args a a2 b b2 ha hb)"
        ),
    },
    DefinitionArtifact {
        name: "KerQuotOne",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (H : Sort succ v), ",
            "forall (f : forall (x : G), H), ",
            "@KerQuot.{u,v} G H f"
        ),
        value: concat!(
            "fun G => fun oneG => fun H => fun f => ",
            "@KerQuotMk.{u,v} G H f oneG"
        ),
    },
    DefinitionArtifact {
        name: "KerQuotInv",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "forall (q : @KerQuot.{u,v} G H f), ",
            "@KerQuot.{u,v} G H f"
        ),
        value: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => fun hom_args => ",
            "@Quotient.lift.{u,u} G (@KerQuot.{u,v} G H f) (@KerSetoid.{u,v} G H f) ",
            "(fun (a : G) => @KerQuotMk.{u,v} G H f (invG a)) ",
            "(fun (a : G) => fun (b : G) => ",
            "fun (h : @Setoid.r.{u} G (@KerSetoid.{u,v} G H f) a b) => ",
            "@ker_quot_sound.{u,v} G H f (invG a) (invG b) ",
            "(@eq_trans.{succ v} H (f (invG a)) (invH (f a)) (f (invG b)) ",
            "(@hom_inv.{succ u,succ v} G oneG mulG invG H oneH mulH invH f hom_args a) ",
            "(@eq_trans.{succ v} H (invH (f a)) (invH (f b)) (f (invG b)) ",
            "(@eq_congr_arg.{succ v,succ v} H H invH (f a) (f b) h) ",
            "(@eq_symm.{succ v} H (f (invG b)) (invH (f b)) ",
            "(@hom_inv.{succ u,succ v} G oneG mulG invG H oneH mulH invH f hom_args b)))))"
        ),
    },
];

const ABSTRACT_GROUP_QUOTIENT_GROUP_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "ker_quot_mul_mk",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "forall (a : G), forall (b : G), ",
            "@Eq.{succ u} (@KerQuot.{u,v} G H f) ",
            "(@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args ",
            "(@KerQuotMk.{u,v} G H f a) (@KerQuotMk.{u,v} G H f b)) ",
            "(@KerQuotMk.{u,v} G H f (mulG a b))"
        ),
        proof: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => ",
            "fun hom_args => fun a => fun b => ",
            "@Eq.refl.{succ u} (@KerQuot.{u,v} G H f) (@KerQuotMk.{u,v} G H f (mulG a b))"
        ),
    },
    TheoremArtifact {
        name: "ker_quot_inv_mk",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "forall (a : G), ",
            "@Eq.{succ u} (@KerQuot.{u,v} G H f) ",
            "(@KerQuotInv.{u,v} G oneG mulG invG H oneH mulH invH f hom_args (@KerQuotMk.{u,v} G H f a)) ",
            "(@KerQuotMk.{u,v} G H f (invG a))"
        ),
        proof: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => ",
            "fun hom_args => fun a => ",
            "@Eq.refl.{succ u} (@KerQuot.{u,v} G H f) (@KerQuotMk.{u,v} G H f (invG a))"
        ),
    },
    TheoremArtifact {
        name: "ker_quot_mul_assoc",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (group_args : @GroupLawArgs.{succ u} G oneG mulG invG), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "forall (q1 : @KerQuot.{u,v} G H f), ",
            "forall (q2 : @KerQuot.{u,v} G H f), ",
            "forall (q3 : @KerQuot.{u,v} G H f), ",
            "@Eq.{succ u} (@KerQuot.{u,v} G H f) ",
            "(@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args ",
            "(@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args q1 q2) q3) ",
            "(@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args ",
            "q1 (@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args q2 q3))"
        ),
        proof: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => fun group_args => fun hom_args => ",
            "@Quotient.indProp.{u} G (@KerSetoid.{u,v} G H f) ",
            "(fun (q1 : @KerQuot.{u,v} G H f) => ",
            "forall (q2 : @KerQuot.{u,v} G H f), forall (q3 : @KerQuot.{u,v} G H f), ",
            "@Eq.{succ u} (@KerQuot.{u,v} G H f) ",
            "(@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args ",
            "(@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args q1 q2) q3) ",
            "(@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args ",
            "q1 (@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args q2 q3))) ",
            "(fun (a : G) => ",
            "@Quotient.indProp.{u} G (@KerSetoid.{u,v} G H f) ",
            "(fun (q2 : @KerQuot.{u,v} G H f) => ",
            "forall (q3 : @KerQuot.{u,v} G H f), ",
            "@Eq.{succ u} (@KerQuot.{u,v} G H f) ",
            "(@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args ",
            "(@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args (@KerQuotMk.{u,v} G H f a) q2) q3) ",
            "(@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args ",
            "(@KerQuotMk.{u,v} G H f a) (@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args q2 q3))) ",
            "(fun (b : G) => ",
            "@Quotient.indProp.{u} G (@KerSetoid.{u,v} G H f) ",
            "(fun (q3 : @KerQuot.{u,v} G H f) => ",
            "@Eq.{succ u} (@KerQuot.{u,v} G H f) ",
            "(@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args ",
            "(@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args (@KerQuotMk.{u,v} G H f a) (@KerQuotMk.{u,v} G H f b)) q3) ",
            "(@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args ",
            "(@KerQuotMk.{u,v} G H f a) (@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args (@KerQuotMk.{u,v} G H f b) q3))) ",
            "(fun (c : G) => ",
            "@ker_quot_sound.{u,v} G H f (mulG (mulG a b) c) (mulG a (mulG b c)) ",
            "(@eq_congr_arg.{succ u,succ v} G H f (mulG (mulG a b) c) (mulG a (mulG b c)) ",
            "(@group_mul_assoc.{succ u} G oneG mulG invG group_args a b c)))))"
        ),
    },
    TheoremArtifact {
        name: "ker_quot_one_mul",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (group_args : @GroupLawArgs.{succ u} G oneG mulG invG), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "forall (q : @KerQuot.{u,v} G H f), ",
            "@Eq.{succ u} (@KerQuot.{u,v} G H f) ",
            "(@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args (@KerQuotOne.{u,v} G oneG H f) q) q"
        ),
        proof: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => fun group_args => fun hom_args => ",
            "@Quotient.indProp.{u} G (@KerSetoid.{u,v} G H f) ",
            "(fun (q : @KerQuot.{u,v} G H f) => ",
            "@Eq.{succ u} (@KerQuot.{u,v} G H f) ",
            "(@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args (@KerQuotOne.{u,v} G oneG H f) q) q) ",
            "(fun (a : G) => ",
            "@ker_quot_sound.{u,v} G H f (mulG oneG a) a ",
            "(@eq_congr_arg.{succ u,succ v} G H f (mulG oneG a) a ",
            "(@group_one_mul.{succ u} G oneG mulG invG group_args a)))"
        ),
    },
    TheoremArtifact {
        name: "ker_quot_mul_one",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (group_args : @GroupLawArgs.{succ u} G oneG mulG invG), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "forall (q : @KerQuot.{u,v} G H f), ",
            "@Eq.{succ u} (@KerQuot.{u,v} G H f) ",
            "(@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args q (@KerQuotOne.{u,v} G oneG H f)) q"
        ),
        proof: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => fun group_args => fun hom_args => ",
            "@Quotient.indProp.{u} G (@KerSetoid.{u,v} G H f) ",
            "(fun (q : @KerQuot.{u,v} G H f) => ",
            "@Eq.{succ u} (@KerQuot.{u,v} G H f) ",
            "(@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args q (@KerQuotOne.{u,v} G oneG H f)) q) ",
            "(fun (a : G) => ",
            "@ker_quot_sound.{u,v} G H f (mulG a oneG) a ",
            "(@eq_congr_arg.{succ u,succ v} G H f (mulG a oneG) a ",
            "(@group_mul_one.{succ u} G oneG mulG invG group_args a)))"
        ),
    },
    TheoremArtifact {
        name: "ker_quot_inv_mul",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (group_args : @GroupLawArgs.{succ u} G oneG mulG invG), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "forall (q : @KerQuot.{u,v} G H f), ",
            "@Eq.{succ u} (@KerQuot.{u,v} G H f) ",
            "(@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args ",
            "(@KerQuotInv.{u,v} G oneG mulG invG H oneH mulH invH f hom_args q) q) ",
            "(@KerQuotOne.{u,v} G oneG H f)"
        ),
        proof: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => fun group_args => fun hom_args => ",
            "@Quotient.indProp.{u} G (@KerSetoid.{u,v} G H f) ",
            "(fun (q : @KerQuot.{u,v} G H f) => ",
            "@Eq.{succ u} (@KerQuot.{u,v} G H f) ",
            "(@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args ",
            "(@KerQuotInv.{u,v} G oneG mulG invG H oneH mulH invH f hom_args q) q) ",
            "(@KerQuotOne.{u,v} G oneG H f)) ",
            "(fun (a : G) => ",
            "@ker_quot_sound.{u,v} G H f (mulG (invG a) a) oneG ",
            "(@eq_congr_arg.{succ u,succ v} G H f (mulG (invG a) a) oneG ",
            "(@group_inv_mul.{succ u} G oneG mulG invG group_args a)))"
        ),
    },
    TheoremArtifact {
        name: "ker_quot_mul_inv",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (group_args : @GroupLawArgs.{succ u} G oneG mulG invG), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "forall (q : @KerQuot.{u,v} G H f), ",
            "@Eq.{succ u} (@KerQuot.{u,v} G H f) ",
            "(@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args ",
            "q (@KerQuotInv.{u,v} G oneG mulG invG H oneH mulH invH f hom_args q)) ",
            "(@KerQuotOne.{u,v} G oneG H f)"
        ),
        proof: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => fun group_args => fun hom_args => ",
            "@Quotient.indProp.{u} G (@KerSetoid.{u,v} G H f) ",
            "(fun (q : @KerQuot.{u,v} G H f) => ",
            "@Eq.{succ u} (@KerQuot.{u,v} G H f) ",
            "(@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args ",
            "q (@KerQuotInv.{u,v} G oneG mulG invG H oneH mulH invH f hom_args q)) ",
            "(@KerQuotOne.{u,v} G oneG H f)) ",
            "(fun (a : G) => ",
            "@ker_quot_sound.{u,v} G H f (mulG a (invG a)) oneG ",
            "(@eq_congr_arg.{succ u,succ v} G H f (mulG a (invG a)) oneG ",
            "(@group_mul_inv.{succ u} G oneG mulG invG group_args a)))"
        ),
    },
];

const ABSTRACT_GROUP_QUOTIENT_HOM_THEOREMS: &[TheoremArtifact] = &[TheoremArtifact {
    name: "ker_quot_to_h_mul",
    universe_params: &["u", "v"],
    statement: concat!(
        "forall (G : Sort succ u), ",
        "forall (oneG : G), ",
        "forall (mulG : forall (a : G), forall (b : G), G), ",
        "forall (invG : forall (a : G), G), ",
        "forall (H : Sort succ v), ",
        "forall (oneH : H), ",
        "forall (mulH : forall (a : H), forall (b : H), H), ",
        "forall (invH : forall (a : H), H), ",
        "forall (f : forall (x : G), H), ",
        "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
        "forall (q1 : @KerQuot.{u,v} G H f), ",
        "forall (q2 : @KerQuot.{u,v} G H f), ",
        "@Eq.{succ v} H ",
        "(@KerQuotToH.{u,v} G H f (@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args q1 q2)) ",
        "(mulH (@KerQuotToH.{u,v} G H f q1) (@KerQuotToH.{u,v} G H f q2))"
    ),
    proof: concat!(
        "fun G => fun oneG => fun mulG => fun invG => ",
        "fun H => fun oneH => fun mulH => fun invH => fun f => fun hom_args => ",
        "@Quotient.indProp.{u} G (@KerSetoid.{u,v} G H f) ",
        "(fun (q1 : @KerQuot.{u,v} G H f) => ",
        "forall (q2 : @KerQuot.{u,v} G H f), ",
        "@Eq.{succ v} H ",
        "(@KerQuotToH.{u,v} G H f (@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args q1 q2)) ",
        "(mulH (@KerQuotToH.{u,v} G H f q1) (@KerQuotToH.{u,v} G H f q2))) ",
        "(fun (a : G) => ",
        "@Quotient.indProp.{u} G (@KerSetoid.{u,v} G H f) ",
        "(fun (q2 : @KerQuot.{u,v} G H f) => ",
        "@Eq.{succ v} H ",
        "(@KerQuotToH.{u,v} G H f (@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args (@KerQuotMk.{u,v} G H f a) q2)) ",
        "(mulH (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f a)) (@KerQuotToH.{u,v} G H f q2))) ",
        "(fun (b : G) => ",
        "@hom_mul.{succ u,succ v} G oneG mulG invG H oneH mulH invH f hom_args a b))"
    ),
}];

const ABSTRACT_GROUP_FIRST_ISO_FULL_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "first_iso_phi_mul",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "forall (q1 : @KerQuot.{u,v} G H f), ",
            "forall (q2 : @KerQuot.{u,v} G H f), ",
            "@Eq.{succ v} H ",
            "(@KerQuotToH.{u,v} G H f (@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args q1 q2)) ",
            "(mulH (@KerQuotToH.{u,v} G H f q1) (@KerQuotToH.{u,v} G H f q2))"
        ),
        proof: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => fun hom_args => ",
            "@ker_quot_to_h_mul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args"
        ),
    },
    TheoremArtifact {
        name: "first_iso_phi_injective",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (H : Sort succ v), ",
            "forall (f : forall (x : G), H), ",
            "forall (q1 : @KerQuot.{u,v} G H f), ",
            "forall (q2 : @KerQuot.{u,v} G H f), ",
            "forall (h : @Eq.{succ v} H (@KerQuotToH.{u,v} G H f q1) (@KerQuotToH.{u,v} G H f q2)), ",
            "@Eq.{succ u} (@KerQuot.{u,v} G H f) q1 q2"
        ),
        proof: concat!(
            "fun G => fun H => fun f => ",
            "@Quotient.indProp.{u} G (@KerSetoid.{u,v} G H f) ",
            "(fun (q1 : @KerQuot.{u,v} G H f) => ",
            "forall (q2 : @KerQuot.{u,v} G H f), ",
            "forall (h : @Eq.{succ v} H (@KerQuotToH.{u,v} G H f q1) (@KerQuotToH.{u,v} G H f q2)), ",
            "@Eq.{succ u} (@KerQuot.{u,v} G H f) q1 q2) ",
            "(fun (a : G) => ",
            "@Quotient.indProp.{u} G (@KerSetoid.{u,v} G H f) ",
            "(fun (q2 : @KerQuot.{u,v} G H f) => ",
            "forall (h : @Eq.{succ v} H (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f a)) (@KerQuotToH.{u,v} G H f q2)), ",
            "@Eq.{succ u} (@KerQuot.{u,v} G H f) (@KerQuotMk.{u,v} G H f a) q2) ",
            "(fun (b : G) => fun (h : @Eq.{succ v} H (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f a)) (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f b))) => ",
            "@ker_quot_sound.{u,v} G H f a b h))"
        ),
    },
    TheoremArtifact {
        name: "first_iso_phi_hits_image",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (H : Sort succ v), ",
            "forall (f : forall (x : G), H), ",
            "forall (q : @KerQuot.{u,v} G H f), ",
            "@ImagePred.{succ u,succ v} G H f (@KerQuotToH.{u,v} G H f q)"
        ),
        proof: concat!(
            "fun G => fun H => fun f => ",
            "@Quotient.indProp.{u} G (@KerSetoid.{u,v} G H f) ",
            "(fun (q : @KerQuot.{u,v} G H f) => ",
            "@ImagePred.{succ u,succ v} G H f (@KerQuotToH.{u,v} G H f q)) ",
            "(fun (a : G) => @image_intro.{succ u,succ v} G H f a)"
        ),
    },
    TheoremArtifact {
        name: "first_iso_phi_surj_image",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (H : Sort succ v), ",
            "forall (f : forall (x : G), H), ",
            "forall (y : H), ",
            "forall (hy : @ImagePred.{succ u,succ v} G H f y), ",
            "forall (P : Prop), ",
            "forall (mk : forall (q : @KerQuot.{u,v} G H f), forall (h : @Eq.{succ v} H (@KerQuotToH.{u,v} G H f q) y), P), ",
            "P"
        ),
        proof: concat!(
            "fun G => fun H => fun f => fun y => fun hy => fun P => fun mk => ",
            "@image_elim.{succ u,succ v} G H f y hy P ",
            "(fun (a : G) => fun (h : @Eq.{succ v} H (f a) y) => ",
            "mk (@KerQuotMk.{u,v} G H f a) ",
            "(@eq_trans.{succ v} H (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f a)) (f a) y ",
            "(@ker_quot_to_h_mk.{u,v} G H f a) h))"
        ),
    },
    TheoremArtifact {
        name: "first_isomorphism_image_facts",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "forall (P : Prop), ",
            "forall (mk : ",
            "forall (mul_compat : forall (q1 : @KerQuot.{u,v} G H f), forall (q2 : @KerQuot.{u,v} G H f), @Eq.{succ v} H (@KerQuotToH.{u,v} G H f (@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args q1 q2)) (mulH (@KerQuotToH.{u,v} G H f q1) (@KerQuotToH.{u,v} G H f q2))), ",
            "forall (injective : forall (q1 : @KerQuot.{u,v} G H f), forall (q2 : @KerQuot.{u,v} G H f), forall (h : @Eq.{succ v} H (@KerQuotToH.{u,v} G H f q1) (@KerQuotToH.{u,v} G H f q2)), @Eq.{succ u} (@KerQuot.{u,v} G H f) q1 q2), ",
            "forall (hits_image : forall (q : @KerQuot.{u,v} G H f), @ImagePred.{succ u,succ v} G H f (@KerQuotToH.{u,v} G H f q)), ",
            "forall (surj_image : forall (y : H), forall (hy : @ImagePred.{succ u,succ v} G H f y), forall (Q : Prop), forall (mk_surj : forall (q : @KerQuot.{u,v} G H f), forall (h : @Eq.{succ v} H (@KerQuotToH.{u,v} G H f q) y), Q), Q), ",
            "P), ",
            "P"
        ),
        proof: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => fun hom_args => ",
            "fun (P : Prop) => fun mk => ",
            "mk ",
            "(@first_iso_phi_mul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args) ",
            "(@first_iso_phi_injective.{u,v} G H f) ",
            "(@first_iso_phi_hits_image.{u,v} G H f) ",
            "(@first_iso_phi_surj_image.{u,v} G H f)"
        ),
    },
];

const FIRST_ISO_QUOTIENT_ASSOC_EVIDENCE_CONSTRUCTORS: &[ConstructorArtifact] =
    &[ConstructorArtifact {
        name: "mk",
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "forall (law : forall (q1 : @KerQuot.{u,v} G H f), forall (q2 : @KerQuot.{u,v} G H f), forall (q3 : @KerQuot.{u,v} G H f), @Eq.{succ u} (@KerQuot.{u,v} G H f) (@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args (@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args q1 q2) q3) (@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args q1 (@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args q2 q3))), ",
            "@FirstIsoQuotientAssocEvidence.{u,v} G oneG mulG invG H oneH mulH invH f hom_args"
        ),
    }];

const FIRST_ISO_QUOTIENT_ONE_MUL_EVIDENCE_CONSTRUCTORS: &[ConstructorArtifact] =
    &[ConstructorArtifact {
        name: "mk",
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "forall (law : forall (q : @KerQuot.{u,v} G H f), @Eq.{succ u} (@KerQuot.{u,v} G H f) (@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args (@KerQuotOne.{u,v} G oneG H f) q) q), ",
            "@FirstIsoQuotientOneMulEvidence.{u,v} G oneG mulG invG H oneH mulH invH f hom_args"
        ),
    }];

const FIRST_ISO_QUOTIENT_MUL_ONE_EVIDENCE_CONSTRUCTORS: &[ConstructorArtifact] =
    &[ConstructorArtifact {
        name: "mk",
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "forall (law : forall (q : @KerQuot.{u,v} G H f), @Eq.{succ u} (@KerQuot.{u,v} G H f) (@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args q (@KerQuotOne.{u,v} G oneG H f)) q), ",
            "@FirstIsoQuotientMulOneEvidence.{u,v} G oneG mulG invG H oneH mulH invH f hom_args"
        ),
    }];

const FIRST_ISO_QUOTIENT_INV_MUL_EVIDENCE_CONSTRUCTORS: &[ConstructorArtifact] =
    &[ConstructorArtifact {
        name: "mk",
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "forall (law : forall (q : @KerQuot.{u,v} G H f), @Eq.{succ u} (@KerQuot.{u,v} G H f) (@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args (@KerQuotInv.{u,v} G oneG mulG invG H oneH mulH invH f hom_args q) q) (@KerQuotOne.{u,v} G oneG H f)), ",
            "@FirstIsoQuotientInvMulEvidence.{u,v} G oneG mulG invG H oneH mulH invH f hom_args"
        ),
    }];

const FIRST_ISO_QUOTIENT_MUL_INV_EVIDENCE_CONSTRUCTORS: &[ConstructorArtifact] =
    &[ConstructorArtifact {
        name: "mk",
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "forall (law : forall (q : @KerQuot.{u,v} G H f), @Eq.{succ u} (@KerQuot.{u,v} G H f) (@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args q (@KerQuotInv.{u,v} G oneG mulG invG H oneH mulH invH f hom_args q)) (@KerQuotOne.{u,v} G oneG H f)), ",
            "@FirstIsoQuotientMulInvEvidence.{u,v} G oneG mulG invG H oneH mulH invH f hom_args"
        ),
    }];

const FIRST_ISO_QUOTIENT_GROUP_EVIDENCE_CONSTRUCTORS: &[ConstructorArtifact] =
    &[ConstructorArtifact {
        name: "mk",
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "forall (assoc : @FirstIsoQuotientAssocEvidence.{u,v} G oneG mulG invG H oneH mulH invH f hom_args), ",
            "forall (one_mul : @FirstIsoQuotientOneMulEvidence.{u,v} G oneG mulG invG H oneH mulH invH f hom_args), ",
            "forall (mul_one : @FirstIsoQuotientMulOneEvidence.{u,v} G oneG mulG invG H oneH mulH invH f hom_args), ",
            "forall (inv_mul : @FirstIsoQuotientInvMulEvidence.{u,v} G oneG mulG invG H oneH mulH invH f hom_args), ",
            "forall (mul_inv : @FirstIsoQuotientMulInvEvidence.{u,v} G oneG mulG invG H oneH mulH invH f hom_args), ",
            "@FirstIsoQuotientGroupEvidence.{u,v} G oneG mulG invG H oneH mulH invH f hom_args"
        ),
    }];

const FIRST_ISO_IMAGE_GROUP_EVIDENCE_CONSTRUCTORS: &[ConstructorArtifact] =
    &[ConstructorArtifact {
        name: "mk",
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "forall (one_member : @ImagePred.{succ u,succ v} G H f oneH), ",
            "forall (mul_closed : forall (x : H), forall (y : H), forall (hx : @ImagePred.{succ u,succ v} G H f x), forall (hy : @ImagePred.{succ u,succ v} G H f y), @ImagePred.{succ u,succ v} G H f (mulH x y)), ",
            "forall (inv_closed : forall (y : H), forall (hy : @ImagePred.{succ u,succ v} G H f y), @ImagePred.{succ u,succ v} G H f (invH y)), ",
            "@FirstIsoImageGroupEvidence.{u,v} G oneG mulG invG H oneH mulH invH f hom_args"
        ),
    }];

const FIRST_ISO_THEOREM_EVIDENCE_CONSTRUCTORS: &[ConstructorArtifact] =
    &[ConstructorArtifact {
        name: "mk",
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "forall (quotient_group : @FirstIsoQuotientGroupEvidence.{u,v} G oneG mulG invG H oneH mulH invH f hom_args), ",
            "forall (image_group : @FirstIsoImageGroupEvidence.{u,v} G oneG mulG invG H oneH mulH invH f hom_args), ",
            "forall (mul_compat : forall (q1 : @KerQuot.{u,v} G H f), forall (q2 : @KerQuot.{u,v} G H f), @Eq.{succ v} H (@KerQuotToH.{u,v} G H f (@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args q1 q2)) (mulH (@KerQuotToH.{u,v} G H f q1) (@KerQuotToH.{u,v} G H f q2))), ",
            "forall (injective : forall (q1 : @KerQuot.{u,v} G H f), forall (q2 : @KerQuot.{u,v} G H f), forall (h : @Eq.{succ v} H (@KerQuotToH.{u,v} G H f q1) (@KerQuotToH.{u,v} G H f q2)), @Eq.{succ u} (@KerQuot.{u,v} G H f) q1 q2), ",
            "forall (hits_image : forall (q : @KerQuot.{u,v} G H f), @ImagePred.{succ u,succ v} G H f (@KerQuotToH.{u,v} G H f q)), ",
            "forall (surj_image : forall (y : H), forall (hy : @ImagePred.{succ u,succ v} G H f y), forall (Q : Prop), forall (mk_surj : forall (q : @KerQuot.{u,v} G H f), forall (h : @Eq.{succ v} H (@KerQuotToH.{u,v} G H f q) y), Q), Q), ",
            "@FirstIsoTheoremEvidence.{u,v} G oneG mulG invG H oneH mulH invH f hom_args"
        ),
    }];

const ABSTRACT_GROUP_FIRST_ISO_IMAGE_INDUCTIVES: &[InductiveArtifact] = &[
    InductiveArtifact {
        name: "FirstIsoQuotientAssocEvidence",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "Prop"
        ),
        constructors: FIRST_ISO_QUOTIENT_ASSOC_EVIDENCE_CONSTRUCTORS,
    },
    InductiveArtifact {
        name: "FirstIsoQuotientOneMulEvidence",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "Prop"
        ),
        constructors: FIRST_ISO_QUOTIENT_ONE_MUL_EVIDENCE_CONSTRUCTORS,
    },
    InductiveArtifact {
        name: "FirstIsoQuotientMulOneEvidence",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "Prop"
        ),
        constructors: FIRST_ISO_QUOTIENT_MUL_ONE_EVIDENCE_CONSTRUCTORS,
    },
    InductiveArtifact {
        name: "FirstIsoQuotientInvMulEvidence",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "Prop"
        ),
        constructors: FIRST_ISO_QUOTIENT_INV_MUL_EVIDENCE_CONSTRUCTORS,
    },
    InductiveArtifact {
        name: "FirstIsoQuotientMulInvEvidence",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "Prop"
        ),
        constructors: FIRST_ISO_QUOTIENT_MUL_INV_EVIDENCE_CONSTRUCTORS,
    },
    InductiveArtifact {
        name: "FirstIsoQuotientGroupEvidence",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "Prop"
        ),
        constructors: FIRST_ISO_QUOTIENT_GROUP_EVIDENCE_CONSTRUCTORS,
    },
    InductiveArtifact {
        name: "FirstIsoImageGroupEvidence",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "Prop"
        ),
        constructors: FIRST_ISO_IMAGE_GROUP_EVIDENCE_CONSTRUCTORS,
    },
    InductiveArtifact {
        name: "FirstIsoTheoremEvidence",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "Prop"
        ),
        constructors: FIRST_ISO_THEOREM_EVIDENCE_CONSTRUCTORS,
    },
];

const ABSTRACT_GROUP_FIRST_ISO_IMAGE_DEFINITIONS: &[DefinitionArtifact] = &[
    DefinitionArtifact {
        name: "FirstIsoImageGroupFacts",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "Prop"
        ),
        value: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => fun hom_args => ",
            "forall (P : Prop), ",
            "forall (mk : ",
            "forall (one_member : @ImagePred.{succ u,succ v} G H f oneH), ",
            "forall (mul_closed : forall (x : H), forall (y : H), forall (hx : @ImagePred.{succ u,succ v} G H f x), forall (hy : @ImagePred.{succ u,succ v} G H f y), @ImagePred.{succ u,succ v} G H f (mulH x y)), ",
            "forall (inv_closed : forall (y : H), forall (hy : @ImagePred.{succ u,succ v} G H f y), @ImagePred.{succ u,succ v} G H f (invH y)), ",
            "P), ",
            "P"
        ),
    },
    DefinitionArtifact {
        name: "FirstIsoImage",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (group_args : @GroupLawArgs.{succ u} G oneG mulG invG), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "Prop"
        ),
        value: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => fun group_args => fun hom_args => ",
            "forall (P : Prop), ",
            "forall (mk : ",
            "forall (image_group : @FirstIsoImageGroupFacts.{u,v} G oneG mulG invG H oneH mulH invH f hom_args), ",
            "forall (mul_compat : forall (q1 : @KerQuot.{u,v} G H f), forall (q2 : @KerQuot.{u,v} G H f), @Eq.{succ v} H (@KerQuotToH.{u,v} G H f (@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args q1 q2)) (mulH (@KerQuotToH.{u,v} G H f q1) (@KerQuotToH.{u,v} G H f q2))), ",
            "forall (injective : forall (q1 : @KerQuot.{u,v} G H f), forall (q2 : @KerQuot.{u,v} G H f), forall (h : @Eq.{succ v} H (@KerQuotToH.{u,v} G H f q1) (@KerQuotToH.{u,v} G H f q2)), @Eq.{succ u} (@KerQuot.{u,v} G H f) q1 q2), ",
            "forall (hits_image : forall (q : @KerQuot.{u,v} G H f), @ImagePred.{succ u,succ v} G H f (@KerQuotToH.{u,v} G H f q)), ",
            "forall (surj_image : forall (y : H), forall (hy : @ImagePred.{succ u,succ v} G H f y), forall (Q : Prop), forall (mk_surj : forall (q : @KerQuot.{u,v} G H f), forall (h : @Eq.{succ v} H (@KerQuotToH.{u,v} G H f q) y), Q), Q), ",
            "P), ",
            "P"
        ),
    },
];

const ABSTRACT_GROUP_FIRST_ISO_IMAGE_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "first_iso_quotient_assoc_evidence",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (group_args : @GroupLawArgs.{succ u} G oneG mulG invG), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "@FirstIsoQuotientAssocEvidence.{u,v} G oneG mulG invG H oneH mulH invH f hom_args"
        ),
        proof: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => fun group_args => fun hom_args => ",
            "@FirstIsoQuotientAssocEvidence.mk.{u,v} G oneG mulG invG H oneH mulH invH f hom_args ",
            "(@ker_quot_mul_assoc.{u,v} G oneG mulG invG H oneH mulH invH f group_args hom_args)"
        ),
    },
    TheoremArtifact {
        name: "first_iso_quotient_one_mul_evidence",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (group_args : @GroupLawArgs.{succ u} G oneG mulG invG), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "@FirstIsoQuotientOneMulEvidence.{u,v} G oneG mulG invG H oneH mulH invH f hom_args"
        ),
        proof: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => fun group_args => fun hom_args => ",
            "@FirstIsoQuotientOneMulEvidence.mk.{u,v} G oneG mulG invG H oneH mulH invH f hom_args ",
            "(@ker_quot_one_mul.{u,v} G oneG mulG invG H oneH mulH invH f group_args hom_args)"
        ),
    },
    TheoremArtifact {
        name: "first_iso_quotient_mul_one_evidence",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (group_args : @GroupLawArgs.{succ u} G oneG mulG invG), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "@FirstIsoQuotientMulOneEvidence.{u,v} G oneG mulG invG H oneH mulH invH f hom_args"
        ),
        proof: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => fun group_args => fun hom_args => ",
            "@FirstIsoQuotientMulOneEvidence.mk.{u,v} G oneG mulG invG H oneH mulH invH f hom_args ",
            "(@ker_quot_mul_one.{u,v} G oneG mulG invG H oneH mulH invH f group_args hom_args)"
        ),
    },
    TheoremArtifact {
        name: "first_iso_quotient_inv_mul_evidence",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (group_args : @GroupLawArgs.{succ u} G oneG mulG invG), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "@FirstIsoQuotientInvMulEvidence.{u,v} G oneG mulG invG H oneH mulH invH f hom_args"
        ),
        proof: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => fun group_args => fun hom_args => ",
            "@FirstIsoQuotientInvMulEvidence.mk.{u,v} G oneG mulG invG H oneH mulH invH f hom_args ",
            "(@ker_quot_inv_mul.{u,v} G oneG mulG invG H oneH mulH invH f group_args hom_args)"
        ),
    },
    TheoremArtifact {
        name: "first_iso_quotient_mul_inv_evidence",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (group_args : @GroupLawArgs.{succ u} G oneG mulG invG), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "@FirstIsoQuotientMulInvEvidence.{u,v} G oneG mulG invG H oneH mulH invH f hom_args"
        ),
        proof: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => fun group_args => fun hom_args => ",
            "@FirstIsoQuotientMulInvEvidence.mk.{u,v} G oneG mulG invG H oneH mulH invH f hom_args ",
            "(@ker_quot_mul_inv.{u,v} G oneG mulG invG H oneH mulH invH f group_args hom_args)"
        ),
    },
    TheoremArtifact {
        name: "first_iso_quotient_group_evidence",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (group_args : @GroupLawArgs.{succ u} G oneG mulG invG), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "@FirstIsoQuotientGroupEvidence.{u,v} G oneG mulG invG H oneH mulH invH f hom_args"
        ),
        proof: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => fun group_args => fun hom_args => ",
            "@FirstIsoQuotientGroupEvidence.mk.{u,v} G oneG mulG invG H oneH mulH invH f hom_args ",
            "(@first_iso_quotient_assoc_evidence.{u,v} G oneG mulG invG H oneH mulH invH f group_args hom_args) ",
            "(@first_iso_quotient_one_mul_evidence.{u,v} G oneG mulG invG H oneH mulH invH f group_args hom_args) ",
            "(@first_iso_quotient_mul_one_evidence.{u,v} G oneG mulG invG H oneH mulH invH f group_args hom_args) ",
            "(@first_iso_quotient_inv_mul_evidence.{u,v} G oneG mulG invG H oneH mulH invH f group_args hom_args) ",
            "(@first_iso_quotient_mul_inv_evidence.{u,v} G oneG mulG invG H oneH mulH invH f group_args hom_args)"
        ),
    },
    TheoremArtifact {
        name: "first_iso_image_group_evidence",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "@FirstIsoImageGroupEvidence.{u,v} G oneG mulG invG H oneH mulH invH f hom_args"
        ),
        proof: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => fun hom_args => ",
            "@FirstIsoImageGroupEvidence.mk.{u,v} G oneG mulG invG H oneH mulH invH f hom_args ",
            "(@image_one.{succ u,succ v} G oneG mulG invG H oneH mulH invH f hom_args) ",
            "(@image_mul_closed.{succ u,succ v} G oneG mulG invG H oneH mulH invH f hom_args) ",
            "(@image_inv_closed.{succ u,succ v} G oneG mulG invG H oneH mulH invH f hom_args)"
        ),
    },
    TheoremArtifact {
        name: "first_iso_image_group_facts",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "forall (P : Prop), ",
            "forall (mk : ",
            "forall (one_member : @ImagePred.{succ u,succ v} G H f oneH), ",
            "forall (mul_closed : forall (x : H), forall (y : H), forall (hx : @ImagePred.{succ u,succ v} G H f x), forall (hy : @ImagePred.{succ u,succ v} G H f y), @ImagePred.{succ u,succ v} G H f (mulH x y)), ",
            "forall (inv_closed : forall (y : H), forall (hy : @ImagePred.{succ u,succ v} G H f y), @ImagePred.{succ u,succ v} G H f (invH y)), ",
            "P), ",
            "P"
        ),
        proof: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => fun hom_args => ",
            "fun (P : Prop) => fun mk => ",
            "mk ",
            "(@image_one.{succ u,succ v} G oneG mulG invG H oneH mulH invH f hom_args) ",
            "(@image_mul_closed.{succ u,succ v} G oneG mulG invG H oneH mulH invH f hom_args) ",
            "(@image_inv_closed.{succ u,succ v} G oneG mulG invG H oneH mulH invH f hom_args)"
        ),
    },
    TheoremArtifact {
        name: "first_isomorphism_theorem_evidence",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (group_args : @GroupLawArgs.{succ u} G oneG mulG invG), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "@FirstIsoTheoremEvidence.{u,v} G oneG mulG invG H oneH mulH invH f hom_args"
        ),
        proof: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => fun group_args => fun hom_args => ",
            "@FirstIsoTheoremEvidence.mk.{u,v} G oneG mulG invG H oneH mulH invH f hom_args ",
            "(@first_iso_quotient_group_evidence.{u,v} G oneG mulG invG H oneH mulH invH f group_args hom_args) ",
            "(@first_iso_image_group_evidence.{u,v} G oneG mulG invG H oneH mulH invH f hom_args) ",
            "(@first_iso_phi_mul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args) ",
            "(@first_iso_phi_injective.{u,v} G H f) ",
            "(@first_iso_phi_hits_image.{u,v} G H f) ",
            "(@first_iso_phi_surj_image.{u,v} G H f)"
        ),
    },
    TheoremArtifact {
        name: "first_isomorphism_to_image",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (group_args : @GroupLawArgs.{succ u} G oneG mulG invG), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "forall (P : Prop), ",
            "forall (mk : ",
            "forall (image_group : @FirstIsoImageGroupFacts.{u,v} G oneG mulG invG H oneH mulH invH f hom_args), ",
            "forall (mul_compat : forall (q1 : @KerQuot.{u,v} G H f), forall (q2 : @KerQuot.{u,v} G H f), @Eq.{succ v} H (@KerQuotToH.{u,v} G H f (@KerQuotMul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args q1 q2)) (mulH (@KerQuotToH.{u,v} G H f q1) (@KerQuotToH.{u,v} G H f q2))), ",
            "forall (injective : forall (q1 : @KerQuot.{u,v} G H f), forall (q2 : @KerQuot.{u,v} G H f), forall (h : @Eq.{succ v} H (@KerQuotToH.{u,v} G H f q1) (@KerQuotToH.{u,v} G H f q2)), @Eq.{succ u} (@KerQuot.{u,v} G H f) q1 q2), ",
            "forall (hits_image : forall (q : @KerQuot.{u,v} G H f), @ImagePred.{succ u,succ v} G H f (@KerQuotToH.{u,v} G H f q)), ",
            "forall (surj_image : forall (y : H), forall (hy : @ImagePred.{succ u,succ v} G H f y), forall (Q : Prop), forall (mk_surj : forall (q : @KerQuot.{u,v} G H f), forall (h : @Eq.{succ v} H (@KerQuotToH.{u,v} G H f q) y), Q), Q), ",
            "P), ",
            "P"
        ),
        proof: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => fun group_args => fun hom_args => ",
            "fun (P : Prop) => fun mk => ",
            "mk ",
            "(@first_iso_image_group_facts.{u,v} G oneG mulG invG H oneH mulH invH f hom_args) ",
            "(@first_iso_phi_mul.{u,v} G oneG mulG invG H oneH mulH invH f hom_args) ",
            "(@first_iso_phi_injective.{u,v} G H f) ",
            "(@first_iso_phi_hits_image.{u,v} G H f) ",
            "(@first_iso_phi_surj_image.{u,v} G H f)"
        ),
    },
];

const ABSTRACT_GROUP_FIRST_ISO_DEFINITIONS: &[DefinitionArtifact] = &[DefinitionArtifact {
    name: "FirstIsoRepMvp",
    universe_params: &["u", "v"],
    ty: concat!(
        "forall (G : Sort succ u), ",
        "forall (oneG : G), ",
        "forall (mulG : forall (a : G), forall (b : G), G), ",
        "forall (invG : forall (a : G), G), ",
        "forall (H : Sort succ v), ",
        "forall (oneH : H), ",
        "forall (mulH : forall (a : H), forall (b : H), H), ",
        "forall (invH : forall (a : H), H), ",
        "forall (f : forall (x : G), H), ",
        "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
        "Prop"
    ),
    value: concat!(
        "fun G => fun oneG => fun mulG => fun invG => ",
        "fun H => fun oneH => fun mulH => fun invH => fun f => fun hom_args => ",
        "forall (P : Prop), ",
        "forall (mk : ",
        "forall (mk_compute : forall (a : G), @Eq.{succ v} H (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f a)) (f a)), ",
        "forall (mul_compat : forall (a : G), forall (b : G), @Eq.{succ v} H (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f (mulG a b))) (mulH (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f a)) (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f b)))), ",
        "forall (rep_injective : forall (a : G), forall (b : G), forall (h : @Eq.{succ v} H (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f a)) (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f b))), @KerRel.{succ u,succ v} G H f a b), ",
        "forall (hits_image : forall (a : G), @ImagePred.{succ u,succ v} G H f (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f a))), ",
        "P), ",
        "P"
    ),
}];

const ABSTRACT_GROUP_FIRST_ISO_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "first_iso_phi_mk",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (H : Sort succ v), ",
            "forall (f : forall (x : G), H), ",
            "forall (a : G), ",
            "@Eq.{succ v} H (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f a)) (f a)"
        ),
        proof: concat!(
            "fun G => fun H => fun f => fun a => ",
            "@ker_quot_to_h_mk.{u,v} G H f a"
        ),
    },
    TheoremArtifact {
        name: "first_iso_phi_mul_mk",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "forall (a : G), forall (b : G), ",
            "@Eq.{succ v} H ",
            "(@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f (mulG a b))) ",
            "(mulH (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f a)) (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f b)))"
        ),
        proof: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => ",
            "fun hom_args => fun a => fun b => ",
            "@ker_quot_to_h_mul_mk.{u,v} G oneG mulG invG H oneH mulH invH f hom_args a b"
        ),
    },
    TheoremArtifact {
        name: "first_iso_rep_injective",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (H : Sort succ v), ",
            "forall (f : forall (x : G), H), ",
            "forall (a : G), forall (b : G), ",
            "forall (h : @Eq.{succ v} H ",
            "(@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f a)) ",
            "(@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f b))), ",
            "@KerRel.{succ u,succ v} G H f a b"
        ),
        proof: "fun G => fun H => fun f => fun a => fun b => fun h => h",
    },
    TheoremArtifact {
        name: "first_iso_rep_hits_image",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (H : Sort succ v), ",
            "forall (f : forall (x : G), H), ",
            "forall (a : G), ",
            "@ImagePred.{succ u,succ v} G H f (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f a))"
        ),
        proof: concat!(
            "fun G => fun H => fun f => fun a => ",
            "@image_intro.{succ u,succ v} G H f a"
        ),
    },
    TheoremArtifact {
        name: "first_isomorphism_rep_mvp",
        universe_params: &["u", "v"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (oneG : G), ",
            "forall (mulG : forall (a : G), forall (b : G), G), ",
            "forall (invG : forall (a : G), G), ",
            "forall (H : Sort succ v), ",
            "forall (oneH : H), ",
            "forall (mulH : forall (a : H), forall (b : H), H), ",
            "forall (invH : forall (a : H), H), ",
            "forall (f : forall (x : G), H), ",
            "forall (hom_args : @GroupHomLawArgs.{succ u,succ v} G oneG mulG invG H oneH mulH invH f), ",
            "@FirstIsoRepMvp.{u,v} G oneG mulG invG H oneH mulH invH f hom_args"
        ),
        proof: concat!(
            "fun G => fun oneG => fun mulG => fun invG => ",
            "fun H => fun oneH => fun mulH => fun invH => fun f => fun hom_args => ",
            "fun (P : Prop) => ",
            "fun (mk : ",
            "forall (mk_compute : forall (a : G), @Eq.{succ v} H (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f a)) (f a)), ",
            "forall (mul_compat : forall (a : G), forall (b : G), @Eq.{succ v} H (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f (mulG a b))) (mulH (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f a)) (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f b)))), ",
            "forall (rep_injective : forall (a : G), forall (b : G), forall (h : @Eq.{succ v} H (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f a)) (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f b))), @KerRel.{succ u,succ v} G H f a b), ",
            "forall (hits_image : forall (a : G), @ImagePred.{succ u,succ v} G H f (@KerQuotToH.{u,v} G H f (@KerQuotMk.{u,v} G H f a))), ",
            "P) => ",
            "mk ",
            "(@first_iso_phi_mk.{u,v} G H f) ",
            "(@first_iso_phi_mul_mk.{u,v} G oneG mulG invG H oneH mulH invH f hom_args) ",
            "(@first_iso_rep_injective.{u,v} G H f) ",
            "(@first_iso_rep_hits_image.{u,v} G H f)"
        ),
    },
];

const ABSTRACT_GROUP_SUBGROUP_DEFINITIONS: &[DefinitionArtifact] = &[
    DefinitionArtifact {
        name: "SubgroupLawArgs",
        universe_params: &["u"],
        ty: abstract_group_params!(
            "forall (S : forall (x : G), Prop), Prop"
        ),
        value: abstract_group_abs!(concat!(
            "fun S => ",
            "forall (P : Prop), forall (mk : ",
            "forall (one_mem : S one), ",
            "forall (mul_closed : forall (a : G), forall (b : G), forall (ha : S a), forall (hb : S b), S (mul a b)), ",
            "forall (inv_closed : forall (a : G), forall (ha : S a), S (inv a)), ",
            "P), P"
        )),
    },
    DefinitionArtifact {
        name: "NormalSubgroupLawArgs",
        universe_params: &["u"],
        ty: abstract_group_params!(
            "forall (N : forall (x : G), Prop), Prop"
        ),
        value: abstract_group_abs!(concat!(
            "fun N => ",
            "forall (P : Prop), forall (mk : ",
            "forall (subgroup_laws : @SubgroupLawArgs.{u} G one mul inv N), ",
            "forall (conj_closed : forall (g : G), forall (n : G), forall (hn : N n), N (mul (mul g n) (inv g))), ",
            "P), P"
        )),
    },
    DefinitionArtifact {
        name: "SubgroupInterPred",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort u), ",
            "forall (S : forall (x : G), Prop), ",
            "forall (T : forall (x : G), Prop), ",
            "forall (x : G), ",
            "Prop"
        ),
        value: concat!(
            "fun G => fun S => fun T => fun x => ",
            "forall (P : Prop), forall (mk : forall (hs : S x), forall (ht : T x), P), P"
        ),
    },
    DefinitionArtifact {
        name: "SubgroupProductPred",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort u), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (S : forall (x : G), Prop), ",
            "forall (T : forall (x : G), Prop), ",
            "forall (x : G), ",
            "Prop"
        ),
        value: concat!(
            "fun G => fun mul => fun S => fun T => fun x => ",
            "forall (P : Prop), ",
            "forall (mk : forall (s : G), forall (t : G), forall (hs : S s), forall (ht : T t), forall (h : @Eq.{u} G (mul s t) x), P), ",
            "P"
        ),
    },
    DefinitionArtifact {
        name: "NormalRel",
        universe_params: &["u"],
        ty: abstract_group_params!(
            "forall (N : forall (x : G), Prop), forall (a : G), forall (b : G), Prop"
        ),
        value: abstract_group_abs!(concat!(
            "fun N => fun a => fun b => ",
            "N (mul (inv a) b)"
        )),
    },
];

const ABSTRACT_GROUP_SUBGROUP_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "subgroup_one",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (S : forall (x : G), Prop), forall (subgroup_args : @SubgroupLawArgs.{u} G one mul inv S), S one"
        ),
        proof: abstract_group_abs!(concat!(
            "fun S => fun subgroup_args => ",
            "subgroup_args (S one) ",
            "(fun (one_mem : S one) => ",
            "fun (mul_closed : forall (a : G), forall (b : G), forall (ha : S a), forall (hb : S b), S (mul a b)) => ",
            "fun (inv_closed : forall (a : G), forall (ha : S a), S (inv a)) => ",
            "one_mem)"
        )),
    },
    TheoremArtifact {
        name: "subgroup_mul_closed",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (S : forall (x : G), Prop), forall (subgroup_args : @SubgroupLawArgs.{u} G one mul inv S), forall (a : G), forall (b : G), forall (ha : S a), forall (hb : S b), S (mul a b)"
        ),
        proof: abstract_group_abs!(concat!(
            "fun S => fun subgroup_args => fun a => fun b => fun ha => fun hb => ",
            "subgroup_args (S (mul a b)) ",
            "(fun (one_mem : S one) => ",
            "fun (mul_closed : forall (a : G), forall (b : G), forall (ha : S a), forall (hb : S b), S (mul a b)) => ",
            "fun (inv_closed : forall (a : G), forall (ha : S a), S (inv a)) => ",
            "mul_closed a b ha hb)"
        )),
    },
    TheoremArtifact {
        name: "subgroup_inv_closed",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (S : forall (x : G), Prop), forall (subgroup_args : @SubgroupLawArgs.{u} G one mul inv S), forall (a : G), forall (ha : S a), S (inv a)"
        ),
        proof: abstract_group_abs!(concat!(
            "fun S => fun subgroup_args => fun a => fun ha => ",
            "subgroup_args (S (inv a)) ",
            "(fun (one_mem : S one) => ",
            "fun (mul_closed : forall (a : G), forall (b : G), forall (ha : S a), forall (hb : S b), S (mul a b)) => ",
            "fun (inv_closed : forall (a : G), forall (ha : S a), S (inv a)) => ",
            "inv_closed a ha)"
        )),
    },
    TheoremArtifact {
        name: "normal_subgroup_laws",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (N : forall (x : G), Prop), forall (normal_args : @NormalSubgroupLawArgs.{u} G one mul inv N), @SubgroupLawArgs.{u} G one mul inv N"
        ),
        proof: abstract_group_abs!(concat!(
            "fun N => fun normal_args => ",
            "normal_args (@SubgroupLawArgs.{u} G one mul inv N) ",
            "(fun (subgroup_laws : @SubgroupLawArgs.{u} G one mul inv N) => ",
            "fun (conj_closed : forall (g : G), forall (n : G), forall (hn : N n), N (mul (mul g n) (inv g))) => ",
            "subgroup_laws)"
        )),
    },
    TheoremArtifact {
        name: "normal_conj_closed",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (N : forall (x : G), Prop), forall (normal_args : @NormalSubgroupLawArgs.{u} G one mul inv N), forall (g : G), forall (n : G), forall (hn : N n), N (mul (mul g n) (inv g))"
        ),
        proof: abstract_group_abs!(concat!(
            "fun N => fun normal_args => fun g => fun n => fun hn => ",
            "normal_args (N (mul (mul g n) (inv g))) ",
            "(fun (subgroup_laws : @SubgroupLawArgs.{u} G one mul inv N) => ",
            "fun (conj_closed : forall (g : G), forall (n : G), forall (hn : N n), N (mul (mul g n) (inv g))) => ",
            "conj_closed g n hn)"
        )),
    },
    TheoremArtifact {
        name: "normal_inv_conj_closed",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (N : forall (x : G), Prop), forall (normal_args : @NormalSubgroupLawArgs.{u} G one mul inv N), forall (g : G), forall (n : G), forall (hn : N n), N (mul (mul (inv g) n) g)"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun N => fun normal_args => fun g => fun n => fun hn => ",
            "@eq_subst.{u} G N (mul (mul (inv g) n) (inv (inv g))) (mul (mul (inv g) n) g) ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul (mul (inv g) n) z) (inv (inv g)) g ",
            "(@group_inv_inv.{u} G one mul inv group_args g)) ",
            "(@normal_conj_closed.{u} G one mul inv N normal_args (inv g) n hn)"
        )),
    },
    TheoremArtifact {
        name: "subgroup_inter_intro",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort u), forall (S : forall (x : G), Prop), forall (T : forall (x : G), Prop), ",
            "forall (x : G), forall (hs : S x), forall (ht : T x), ",
            "@SubgroupInterPred.{u} G S T x"
        ),
        proof: concat!(
            "fun G => fun S => fun T => fun x => fun hs => fun ht => ",
            "fun (P : Prop) => fun (mk : forall (hs : S x), forall (ht : T x), P) => mk hs ht"
        ),
    },
    TheoremArtifact {
        name: "subgroup_inter_left",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort u), forall (S : forall (x : G), Prop), forall (T : forall (x : G), Prop), ",
            "forall (x : G), forall (h : @SubgroupInterPred.{u} G S T x), S x"
        ),
        proof: concat!(
            "fun G => fun S => fun T => fun x => fun h => ",
            "h (S x) (fun (hs : S x) => fun (ht : T x) => hs)"
        ),
    },
    TheoremArtifact {
        name: "subgroup_inter_right",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort u), forall (S : forall (x : G), Prop), forall (T : forall (x : G), Prop), ",
            "forall (x : G), forall (h : @SubgroupInterPred.{u} G S T x), T x"
        ),
        proof: concat!(
            "fun G => fun S => fun T => fun x => fun h => ",
            "h (T x) (fun (hs : S x) => fun (ht : T x) => ht)"
        ),
    },
    TheoremArtifact {
        name: "subgroup_inter_one",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (S : forall (x : G), Prop), forall (T : forall (x : G), Prop), forall (s_args : @SubgroupLawArgs.{u} G one mul inv S), forall (t_args : @SubgroupLawArgs.{u} G one mul inv T), @SubgroupInterPred.{u} G S T one"
        ),
        proof: abstract_group_abs!(concat!(
            "fun S => fun T => fun s_args => fun t_args => ",
            "@subgroup_inter_intro.{u} G S T one ",
            "(@subgroup_one.{u} G one mul inv S s_args) ",
            "(@subgroup_one.{u} G one mul inv T t_args)"
        )),
    },
    TheoremArtifact {
        name: "subgroup_inter_mul_closed",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (S : forall (x : G), Prop), forall (T : forall (x : G), Prop), forall (s_args : @SubgroupLawArgs.{u} G one mul inv S), forall (t_args : @SubgroupLawArgs.{u} G one mul inv T), forall (a : G), forall (b : G), forall (ha : @SubgroupInterPred.{u} G S T a), forall (hb : @SubgroupInterPred.{u} G S T b), @SubgroupInterPred.{u} G S T (mul a b)"
        ),
        proof: abstract_group_abs!(concat!(
            "fun S => fun T => fun s_args => fun t_args => fun a => fun b => fun ha => fun hb => ",
            "ha (@SubgroupInterPred.{u} G S T (mul a b)) ",
            "(fun (haS : S a) => fun (haT : T a) => ",
            "hb (@SubgroupInterPred.{u} G S T (mul a b)) ",
            "(fun (hbS : S b) => fun (hbT : T b) => ",
            "@subgroup_inter_intro.{u} G S T (mul a b) ",
            "(@subgroup_mul_closed.{u} G one mul inv S s_args a b haS hbS) ",
            "(@subgroup_mul_closed.{u} G one mul inv T t_args a b haT hbT)))"
        )),
    },
    TheoremArtifact {
        name: "subgroup_inter_inv_closed",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (S : forall (x : G), Prop), forall (T : forall (x : G), Prop), forall (s_args : @SubgroupLawArgs.{u} G one mul inv S), forall (t_args : @SubgroupLawArgs.{u} G one mul inv T), forall (a : G), forall (ha : @SubgroupInterPred.{u} G S T a), @SubgroupInterPred.{u} G S T (inv a)"
        ),
        proof: abstract_group_abs!(concat!(
            "fun S => fun T => fun s_args => fun t_args => fun a => fun ha => ",
            "ha (@SubgroupInterPred.{u} G S T (inv a)) ",
            "(fun (haS : S a) => fun (haT : T a) => ",
            "@subgroup_inter_intro.{u} G S T (inv a) ",
            "(@subgroup_inv_closed.{u} G one mul inv S s_args a haS) ",
            "(@subgroup_inv_closed.{u} G one mul inv T t_args a haT))"
        )),
    },
    TheoremArtifact {
        name: "subgroup_inter_normal_in_left",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (Hpred : forall (x : G), Prop), forall (N : forall (x : G), Prop), forall (h_args : @SubgroupLawArgs.{u} G one mul inv Hpred), forall (n_normal : @NormalSubgroupLawArgs.{u} G one mul inv N), forall (h : G), forall (n : G), forall (hh : Hpred h), forall (hn : @SubgroupInterPred.{u} G Hpred N n), @SubgroupInterPred.{u} G Hpred N (mul (mul h n) (inv h))"
        ),
        proof: abstract_group_abs!(concat!(
            "fun Hpred => fun N => fun h_args => fun n_normal => fun h => fun n => fun hh => fun hn => ",
            "hn (@SubgroupInterPred.{u} G Hpred N (mul (mul h n) (inv h))) ",
            "(fun (hnH : Hpred n) => fun (hnN : N n) => ",
            "@subgroup_inter_intro.{u} G Hpred N (mul (mul h n) (inv h)) ",
            "(@subgroup_mul_closed.{u} G one mul inv Hpred h_args (mul h n) (inv h) ",
            "(@subgroup_mul_closed.{u} G one mul inv Hpred h_args h n hh hnH) ",
            "(@subgroup_inv_closed.{u} G one mul inv Hpred h_args h hh)) ",
            "(@normal_conj_closed.{u} G one mul inv N n_normal h n hnN))"
        )),
    },
    TheoremArtifact {
        name: "subgroup_product_intro",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort u), forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (S : forall (x : G), Prop), forall (T : forall (x : G), Prop), ",
            "forall (x : G), forall (s : G), forall (t : G), ",
            "forall (hs : S s), forall (ht : T t), forall (h : @Eq.{u} G (mul s t) x), ",
            "@SubgroupProductPred.{u} G mul S T x"
        ),
        proof: concat!(
            "fun G => fun mul => fun S => fun T => fun x => fun s => fun t => fun hs => fun ht => fun h => ",
            "fun (P : Prop) => ",
            "fun (mk : forall (s : G), forall (t : G), forall (hs : S s), forall (ht : T t), forall (h : @Eq.{u} G (mul s t) x), P) => ",
            "mk s t hs ht h"
        ),
    },
    TheoremArtifact {
        name: "subgroup_product_elim",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort u), forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (S : forall (x : G), Prop), forall (T : forall (x : G), Prop), ",
            "forall (x : G), forall (prod : @SubgroupProductPred.{u} G mul S T x), ",
            "forall (P : Prop), ",
            "forall (mk : forall (s : G), forall (t : G), forall (hs : S s), forall (ht : T t), forall (h : @Eq.{u} G (mul s t) x), P), ",
            "P"
        ),
        proof: concat!(
            "fun G => fun mul => fun S => fun T => fun x => fun prod => fun P => fun mk => ",
            "prod P mk"
        ),
    },
    TheoremArtifact {
        name: "subgroup_product_one",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (S : forall (x : G), Prop), forall (T : forall (x : G), Prop), forall (s_args : @SubgroupLawArgs.{u} G one mul inv S), forall (t_args : @SubgroupLawArgs.{u} G one mul inv T), @SubgroupProductPred.{u} G mul S T one"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun S => fun T => fun s_args => fun t_args => ",
            "@subgroup_product_intro.{u} G mul S T one one one ",
            "(@subgroup_one.{u} G one mul inv S s_args) ",
            "(@subgroup_one.{u} G one mul inv T t_args) ",
            "(@group_one_mul.{u} G one mul inv group_args one)"
        )),
    },
    TheoremArtifact {
        name: "subgroup_product_mul_closed",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (Hpred : forall (x : G), Prop), forall (N : forall (x : G), Prop), forall (h_args : @SubgroupLawArgs.{u} G one mul inv Hpred), forall (n_normal : @NormalSubgroupLawArgs.{u} G one mul inv N), forall (x : G), forall (y : G), forall (hx : @SubgroupProductPred.{u} G mul Hpred N x), forall (hy : @SubgroupProductPred.{u} G mul Hpred N y), @SubgroupProductPred.{u} G mul Hpred N (mul x y)"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun Hpred => fun N => fun h_args => fun n_normal => fun x => fun y => fun hx => fun hy => ",
            "hx (@SubgroupProductPred.{u} G mul Hpred N (mul x y)) ",
            "(fun (h1 : G) => fun (n1 : G) => fun (hh1 : Hpred h1) => fun (hn1 : N n1) => fun (hex : @Eq.{u} G (mul h1 n1) x) => ",
            "hy (@SubgroupProductPred.{u} G mul Hpred N (mul x y)) ",
            "(fun (h2 : G) => fun (n2 : G) => fun (hh2 : Hpred h2) => fun (hn2 : N n2) => fun (hey : @Eq.{u} G (mul h2 n2) y) => ",
            "@subgroup_product_intro.{u} G mul Hpred N (mul x y) (mul h1 h2) (mul (mul (mul (inv h2) n1) h2) n2) ",
            "(@subgroup_mul_closed.{u} G one mul inv Hpred h_args h1 h2 hh1 hh2) ",
            "(@subgroup_mul_closed.{u} G one mul inv N (@normal_subgroup_laws.{u} G one mul inv N n_normal) (mul (mul (inv h2) n1) h2) n2 ",
            "(@normal_inv_conj_closed.{u} G one mul inv group_args N n_normal h2 n1 hn1) ",
            "hn2) ",
            "(@eq_trans.{u} G (mul (mul h1 h2) (mul (mul (mul (inv h2) n1) h2) n2)) (mul (mul h1 n1) (mul h2 n2)) (mul x y) ",
            "(@group_product_mul_reassoc.{u} G one mul inv group_args h1 n1 h2 n2) ",
            "(@eq_congr2.{u,u,u} G G G mul (mul h1 n1) x (mul h2 n2) y hex hey))))"
        )),
    },
    TheoremArtifact {
        name: "subgroup_product_inv_closed",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (Hpred : forall (x : G), Prop), forall (N : forall (x : G), Prop), forall (h_args : @SubgroupLawArgs.{u} G one mul inv Hpred), forall (n_normal : @NormalSubgroupLawArgs.{u} G one mul inv N), forall (x : G), forall (hx : @SubgroupProductPred.{u} G mul Hpred N x), @SubgroupProductPred.{u} G mul Hpred N (inv x)"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun Hpred => fun N => fun h_args => fun n_normal => fun x => fun hx => ",
            "hx (@SubgroupProductPred.{u} G mul Hpred N (inv x)) ",
            "(fun (h : G) => fun (n : G) => fun (hh : Hpred h) => fun (hn : N n) => fun (hex : @Eq.{u} G (mul h n) x) => ",
            "@subgroup_product_intro.{u} G mul Hpred N (inv x) (inv h) (mul (mul h (inv n)) (inv h)) ",
            "(@subgroup_inv_closed.{u} G one mul inv Hpred h_args h hh) ",
            "(@normal_conj_closed.{u} G one mul inv N n_normal h (inv n) ",
            "(@subgroup_inv_closed.{u} G one mul inv N (@normal_subgroup_laws.{u} G one mul inv N n_normal) n hn)) ",
            "(@eq_trans.{u} G (mul (inv h) (mul (mul h (inv n)) (inv h))) (inv (mul h n)) (inv x) ",
            "(@group_product_inv_reassoc.{u} G one mul inv group_args h n) ",
            "(@eq_congr_arg.{u,u} G G inv (mul h n) x hex)))"
        )),
    },
    TheoremArtifact {
        name: "subgroup_product_laws",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (Hpred : forall (x : G), Prop), forall (N : forall (x : G), Prop), forall (h_args : @SubgroupLawArgs.{u} G one mul inv Hpred), forall (n_normal : @NormalSubgroupLawArgs.{u} G one mul inv N), @SubgroupLawArgs.{u} G one mul inv (@SubgroupProductPred.{u} G mul Hpred N)"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun Hpred => fun N => fun h_args => fun n_normal => ",
            "fun (P : Prop) => ",
            "fun (mk : forall (one_mem : @SubgroupProductPred.{u} G mul Hpred N one), forall (mul_closed : forall (a : G), forall (b : G), forall (ha : @SubgroupProductPred.{u} G mul Hpred N a), forall (hb : @SubgroupProductPred.{u} G mul Hpred N b), @SubgroupProductPred.{u} G mul Hpred N (mul a b)), forall (inv_closed : forall (a : G), forall (ha : @SubgroupProductPred.{u} G mul Hpred N a), @SubgroupProductPred.{u} G mul Hpred N (inv a)), P) => ",
            "mk ",
            "(@subgroup_product_one.{u} G one mul inv group_args Hpred N h_args (@normal_subgroup_laws.{u} G one mul inv N n_normal)) ",
            "(@subgroup_product_mul_closed.{u} G one mul inv group_args Hpred N h_args n_normal) ",
            "(@subgroup_product_inv_closed.{u} G one mul inv group_args Hpred N h_args n_normal)"
        )),
    },
    TheoremArtifact {
        name: "normal_rel_refl",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (N : forall (x : G), Prop), forall (normal_args : @NormalSubgroupLawArgs.{u} G one mul inv N), forall (a : G), @NormalRel.{u} G one mul inv N a a"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun N => fun normal_args => fun a => ",
            "@eq_subst.{u} G N one (mul (inv a) a) ",
            "(@eq_symm.{u} G (mul (inv a) a) one ",
            "(@group_inv_mul.{u} G one mul inv group_args a)) ",
            "(@subgroup_one.{u} G one mul inv N ",
            "(@normal_subgroup_laws.{u} G one mul inv N normal_args))"
        )),
    },
    TheoremArtifact {
        name: "normal_rel_symm",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (N : forall (x : G), Prop), forall (normal_args : @NormalSubgroupLawArgs.{u} G one mul inv N), forall (a : G), forall (b : G), forall (h : @NormalRel.{u} G one mul inv N a b), @NormalRel.{u} G one mul inv N b a"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun N => fun normal_args => fun a => fun b => fun h => ",
            "@eq_subst.{u} G N (inv (mul (inv a) b)) (mul (inv b) a) ",
            "(@group_inv_rel_symm_reassoc.{u} G one mul inv group_args a b) ",
            "(@subgroup_inv_closed.{u} G one mul inv N ",
            "(@normal_subgroup_laws.{u} G one mul inv N normal_args) ",
            "(mul (inv a) b) h)"
        )),
    },
    TheoremArtifact {
        name: "normal_rel_trans",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (N : forall (x : G), Prop), forall (normal_args : @NormalSubgroupLawArgs.{u} G one mul inv N), forall (a : G), forall (b : G), forall (c : G), forall (hab : @NormalRel.{u} G one mul inv N a b), forall (hbc : @NormalRel.{u} G one mul inv N b c), @NormalRel.{u} G one mul inv N a c"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun N => fun normal_args => fun a => fun b => fun c => fun hab => fun hbc => ",
            "@eq_subst.{u} G N (mul (mul (inv a) b) (mul (inv b) c)) (mul (inv a) c) ",
            "(@group_rel_trans_reassoc.{u} G one mul inv group_args a b c) ",
            "(@subgroup_mul_closed.{u} G one mul inv N ",
            "(@normal_subgroup_laws.{u} G one mul inv N normal_args) ",
            "(mul (inv a) b) (mul (inv b) c) hab hbc)"
        )),
    },
    TheoremArtifact {
        name: "normal_rel_of_eq",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (N : forall (x : G), Prop), forall (normal_args : @NormalSubgroupLawArgs.{u} G one mul inv N), forall (a : G), forall (b : G), forall (h : @Eq.{u} G a b), @NormalRel.{u} G one mul inv N a b"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun N => fun normal_args => fun a => fun b => fun h => ",
            "@eq_subst.{u} G (fun (z : G) => @NormalRel.{u} G one mul inv N a z) a b h ",
            "(@normal_rel_refl.{u} G one mul inv group_args N normal_args a)"
        )),
    },
    TheoremArtifact {
        name: "normal_rel_mul_compat",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (N : forall (x : G), Prop), forall (normal_args : @NormalSubgroupLawArgs.{u} G one mul inv N), forall (a : G), forall (a2 : G), forall (b : G), forall (b2 : G), forall (ha : @NormalRel.{u} G one mul inv N a a2), forall (hb : @NormalRel.{u} G one mul inv N b b2), @NormalRel.{u} G one mul inv N (mul a b) (mul a2 b2)"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun N => fun normal_args => fun a => fun a2 => fun b => fun b2 => fun ha => fun hb => ",
            "@eq_subst.{u} G N ",
            "(mul (mul (mul (inv b) (mul (inv a) a2)) b) (mul (inv b) b2)) ",
            "(mul (inv (mul a b)) (mul a2 b2)) ",
            "(@group_rel_mul_reassoc.{u} G one mul inv group_args a a2 b b2) ",
            "(@subgroup_mul_closed.{u} G one mul inv N ",
            "(@normal_subgroup_laws.{u} G one mul inv N normal_args) ",
            "(mul (mul (inv b) (mul (inv a) a2)) b) (mul (inv b) b2) ",
            "(@normal_inv_conj_closed.{u} G one mul inv group_args N normal_args b (mul (inv a) a2) ha) ",
            "hb)"
        )),
    },
    TheoremArtifact {
        name: "normal_rel_inv_compat",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (N : forall (x : G), Prop), forall (normal_args : @NormalSubgroupLawArgs.{u} G one mul inv N), forall (a : G), forall (b : G), forall (h : @NormalRel.{u} G one mul inv N a b), @NormalRel.{u} G one mul inv N (inv a) (inv b)"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun N => fun normal_args => fun a => fun b => fun h => ",
            "@eq_subst.{u} G N ",
            "(mul (mul a (mul (inv b) a)) (inv a)) ",
            "(mul (inv (inv a)) (inv b)) ",
            "(@group_rel_inv_reassoc.{u} G one mul inv group_args a b) ",
            "(@normal_conj_closed.{u} G one mul inv N normal_args a (mul (inv b) a) ",
            "(@normal_rel_symm.{u} G one mul inv group_args N normal_args a b h))"
        )),
    },
    TheoremArtifact {
        name: "normal_rel_one_of_mem",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (N : forall (x : G), Prop), forall (normal_args : @NormalSubgroupLawArgs.{u} G one mul inv N), forall (h : G), forall (hn : N h), @NormalRel.{u} G one mul inv N h one"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun N => fun normal_args => fun h => fun hn => ",
            "@eq_subst.{u} G N (inv h) (mul (inv h) one) ",
            "(@eq_symm.{u} G (mul (inv h) one) (inv h) ",
            "(@group_mul_one.{u} G one mul inv group_args (inv h))) ",
            "(@subgroup_inv_closed.{u} G one mul inv N ",
            "(@normal_subgroup_laws.{u} G one mul inv N normal_args) h hn)"
        )),
    },
    TheoremArtifact {
        name: "normal_rel_one_to_mem",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (N : forall (x : G), Prop), forall (normal_args : @NormalSubgroupLawArgs.{u} G one mul inv N), forall (h : G), forall (hk : @NormalRel.{u} G one mul inv N h one), N h"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun N => fun normal_args => fun h => fun hk => ",
            "@eq_subst.{u} G N (inv (inv h)) h ",
            "(@group_inv_inv.{u} G one mul inv group_args h) ",
            "(@subgroup_inv_closed.{u} G one mul inv N ",
            "(@normal_subgroup_laws.{u} G one mul inv N normal_args) (inv h) ",
            "(@eq_subst.{u} G N (mul (inv h) one) (inv h) ",
            "(@group_mul_one.{u} G one mul inv group_args (inv h)) hk))"
        )),
    },
    TheoremArtifact {
        name: "normal_rel_product_right",
        universe_params: &["u"],
        statement: abstract_group_params!(
            "forall (group_args : @GroupLawArgs.{u} G one mul inv), forall (N : forall (x : G), Prop), forall (h : G), forall (n : G), forall (x : G), forall (hn : N n), forall (hx : @Eq.{u} G (mul h n) x), @NormalRel.{u} G one mul inv N h x"
        ),
        proof: abstract_group_abs!(concat!(
            "fun group_args => fun N => fun h => fun n => fun x => fun hn => fun hx => ",
            "@eq_subst.{u} G N n (mul (inv h) x) ",
            "(@eq_trans.{u} G n (mul (inv h) (mul h n)) (mul (inv h) x) ",
            "(@eq_symm.{u} G (mul (inv h) (mul h n)) n ",
            "(@group_inv_mul_left_reassoc.{u} G one mul inv group_args h n)) ",
            "(@eq_congr_arg.{u,u} G G (fun (z : G) => mul (inv h) z) (mul h n) x hx)) ",
            "hn"
        )),
    },
];

const ABSTRACT_GROUP_NORMAL_QUOTIENT_DEFINITIONS: &[DefinitionArtifact] = &[
    DefinitionArtifact {
        name: "NormalSetoid",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "@Setoid.{u} G"
        ),
        value: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => ",
            "@Setoid.mk.{u} G (@NormalRel.{succ u} G one mul inv N) ",
            "(fun (P : Prop) => ",
            "fun (mk_equiv : ",
            "forall (refl_arg : forall (x : G), @NormalRel.{succ u} G one mul inv N x x), ",
            "forall (symm_arg : forall (x : G), forall (y : G), forall (p : @NormalRel.{succ u} G one mul inv N x y), @NormalRel.{succ u} G one mul inv N y x), ",
            "forall (trans_arg : forall (x : G), forall (y : G), forall (z : G), forall (p : @NormalRel.{succ u} G one mul inv N x y), forall (q : @NormalRel.{succ u} G one mul inv N y z), @NormalRel.{succ u} G one mul inv N x z), ",
            "P) => ",
            "mk_equiv ",
            "(@normal_rel_refl.{succ u} G one mul inv group_args N normal_args) ",
            "(@normal_rel_symm.{succ u} G one mul inv group_args N normal_args) ",
            "(@normal_rel_trans.{succ u} G one mul inv group_args N normal_args))"
        ),
    },
    DefinitionArtifact {
        name: "NormalQuot",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "Sort succ u"
        ),
        value: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => ",
            "@Quotient.{u} G (@NormalSetoid.{u} G one mul inv N group_args normal_args)"
        ),
    },
    DefinitionArtifact {
        name: "NormalQuotMk",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (a : G), ",
            "@NormalQuot.{u} G one mul inv N group_args normal_args"
        ),
        value: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => fun a => ",
            "@Quotient.mk.{u} G (@NormalSetoid.{u} G one mul inv N group_args normal_args) a"
        ),
    },
];

const ABSTRACT_GROUP_NORMAL_QUOTIENT_THEOREMS: &[TheoremArtifact] = &[TheoremArtifact {
    name: "normal_quot_sound",
    universe_params: &["u"],
    statement: concat!(
        "forall (G : Sort succ u), ",
        "forall (one : G), ",
        "forall (mul : forall (a : G), forall (b : G), G), ",
        "forall (inv : forall (a : G), G), ",
        "forall (N : forall (x : G), Prop), ",
        "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
        "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
        "forall (a : G), forall (b : G), ",
        "forall (h : @NormalRel.{succ u} G one mul inv N a b), ",
        "@Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
        "(@NormalQuotMk.{u} G one mul inv N group_args normal_args a) ",
        "(@NormalQuotMk.{u} G one mul inv N group_args normal_args b)"
    ),
    proof: concat!(
        "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => fun a => fun b => fun h => ",
        "@Quotient.sound.{u} G (@NormalSetoid.{u} G one mul inv N group_args normal_args) a b h"
    ),
}];

const ABSTRACT_GROUP_NORMAL_QUOTIENT_MUL_DEFINITIONS: &[DefinitionArtifact] =
    &[DefinitionArtifact {
        name: "NormalQuotMulRep",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (a : G), forall (b : G), ",
            "@NormalQuot.{u} G one mul inv N group_args normal_args"
        ),
        value: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => fun a => fun b => ",
            "@NormalQuotMk.{u} G one mul inv N group_args normal_args (mul a b)"
        ),
    }];

const ABSTRACT_GROUP_NORMAL_QUOTIENT_MUL_THEOREMS: &[TheoremArtifact] = &[TheoremArtifact {
    name: "normal_quot_mul_rep_compat",
    universe_params: &["u"],
    statement: concat!(
        "forall (G : Sort succ u), ",
        "forall (one : G), ",
        "forall (mul : forall (a : G), forall (b : G), G), ",
        "forall (inv : forall (a : G), G), ",
        "forall (N : forall (x : G), Prop), ",
        "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
        "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
        "forall (a : G), forall (a2 : G), ",
        "forall (b : G), forall (b2 : G), ",
        "forall (ha : @NormalRel.{succ u} G one mul inv N a a2), ",
        "forall (hb : @NormalRel.{succ u} G one mul inv N b b2), ",
        "@Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
        "(@NormalQuotMulRep.{u} G one mul inv N group_args normal_args a b) ",
        "(@NormalQuotMulRep.{u} G one mul inv N group_args normal_args a2 b2)"
    ),
    proof: concat!(
        "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => ",
        "fun a => fun a2 => fun b => fun b2 => fun ha => fun hb => ",
        "@normal_quot_sound.{u} G one mul inv N group_args normal_args (mul a b) (mul a2 b2) ",
        "(@normal_rel_mul_compat.{succ u} G one mul inv group_args N normal_args a a2 b b2 ha hb)"
    ),
}];

const ABSTRACT_GROUP_NORMAL_QUOTIENT_GROUP_DEFINITIONS: &[DefinitionArtifact] = &[
    DefinitionArtifact {
        name: "NormalQuotMul",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (q1 : @NormalQuot.{u} G one mul inv N group_args normal_args), ",
            "forall (q2 : @NormalQuot.{u} G one mul inv N group_args normal_args), ",
            "@NormalQuot.{u} G one mul inv N group_args normal_args"
        ),
        value: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => ",
            "@Quotient.lift2.{u,u} G (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalSetoid.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotMulRep.{u} G one mul inv N group_args normal_args) ",
            "(fun (a : G) => fun (a2 : G) => fun (b : G) => fun (b2 : G) => ",
            "fun (ha : @Setoid.r.{u} G (@NormalSetoid.{u} G one mul inv N group_args normal_args) a a2) => ",
            "fun (hb : @Setoid.r.{u} G (@NormalSetoid.{u} G one mul inv N group_args normal_args) b b2) => ",
            "@normal_quot_mul_rep_compat.{u} G one mul inv N group_args normal_args a a2 b b2 ha hb)"
        ),
    },
    DefinitionArtifact {
        name: "NormalQuotOne",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "@NormalQuot.{u} G one mul inv N group_args normal_args"
        ),
        value: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => ",
            "@NormalQuotMk.{u} G one mul inv N group_args normal_args one"
        ),
    },
    DefinitionArtifact {
        name: "NormalQuotInv",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (q : @NormalQuot.{u} G one mul inv N group_args normal_args), ",
            "@NormalQuot.{u} G one mul inv N group_args normal_args"
        ),
        value: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => ",
            "@Quotient.lift.{u,u} G (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalSetoid.{u} G one mul inv N group_args normal_args) ",
            "(fun (a : G) => @NormalQuotMk.{u} G one mul inv N group_args normal_args (inv a)) ",
            "(fun (a : G) => fun (b : G) => ",
            "fun (h : @Setoid.r.{u} G (@NormalSetoid.{u} G one mul inv N group_args normal_args) a b) => ",
            "@normal_quot_sound.{u} G one mul inv N group_args normal_args (inv a) (inv b) ",
            "(@normal_rel_inv_compat.{succ u} G one mul inv group_args N normal_args a b h))"
        ),
    },
];

const ABSTRACT_GROUP_NORMAL_QUOTIENT_GROUP_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "normal_quot_mul_mk",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (a : G), forall (b : G), ",
            "@Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotMul.{u} G one mul inv N group_args normal_args ",
            "(@NormalQuotMk.{u} G one mul inv N group_args normal_args a) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args normal_args b)) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args normal_args (mul a b))"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => fun a => fun b => ",
            "@Eq.refl.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args normal_args (mul a b))"
        ),
    },
    TheoremArtifact {
        name: "normal_quot_inv_mk",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (a : G), ",
            "@Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotInv.{u} G one mul inv N group_args normal_args ",
            "(@NormalQuotMk.{u} G one mul inv N group_args normal_args a)) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args normal_args (inv a))"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => fun a => ",
            "@Eq.refl.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args normal_args (inv a))"
        ),
    },
    TheoremArtifact {
        name: "normal_quot_mul_assoc",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (q1 : @NormalQuot.{u} G one mul inv N group_args normal_args), ",
            "forall (q2 : @NormalQuot.{u} G one mul inv N group_args normal_args), ",
            "forall (q3 : @NormalQuot.{u} G one mul inv N group_args normal_args), ",
            "@Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotMul.{u} G one mul inv N group_args normal_args ",
            "(@NormalQuotMul.{u} G one mul inv N group_args normal_args q1 q2) q3) ",
            "(@NormalQuotMul.{u} G one mul inv N group_args normal_args ",
            "q1 (@NormalQuotMul.{u} G one mul inv N group_args normal_args q2 q3))"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => ",
            "@Quotient.indProp.{u} G (@NormalSetoid.{u} G one mul inv N group_args normal_args) ",
            "(fun (q1 : @NormalQuot.{u} G one mul inv N group_args normal_args) => ",
            "forall (q2 : @NormalQuot.{u} G one mul inv N group_args normal_args), ",
            "forall (q3 : @NormalQuot.{u} G one mul inv N group_args normal_args), ",
            "@Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotMul.{u} G one mul inv N group_args normal_args ",
            "(@NormalQuotMul.{u} G one mul inv N group_args normal_args q1 q2) q3) ",
            "(@NormalQuotMul.{u} G one mul inv N group_args normal_args ",
            "q1 (@NormalQuotMul.{u} G one mul inv N group_args normal_args q2 q3))) ",
            "(fun (a : G) => ",
            "@Quotient.indProp.{u} G (@NormalSetoid.{u} G one mul inv N group_args normal_args) ",
            "(fun (q2 : @NormalQuot.{u} G one mul inv N group_args normal_args) => ",
            "forall (q3 : @NormalQuot.{u} G one mul inv N group_args normal_args), ",
            "@Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotMul.{u} G one mul inv N group_args normal_args ",
            "(@NormalQuotMul.{u} G one mul inv N group_args normal_args ",
            "(@NormalQuotMk.{u} G one mul inv N group_args normal_args a) q2) q3) ",
            "(@NormalQuotMul.{u} G one mul inv N group_args normal_args ",
            "(@NormalQuotMk.{u} G one mul inv N group_args normal_args a) ",
            "(@NormalQuotMul.{u} G one mul inv N group_args normal_args q2 q3))) ",
            "(fun (b : G) => ",
            "@Quotient.indProp.{u} G (@NormalSetoid.{u} G one mul inv N group_args normal_args) ",
            "(fun (q3 : @NormalQuot.{u} G one mul inv N group_args normal_args) => ",
            "@Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotMul.{u} G one mul inv N group_args normal_args ",
            "(@NormalQuotMul.{u} G one mul inv N group_args normal_args ",
            "(@NormalQuotMk.{u} G one mul inv N group_args normal_args a) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args normal_args b)) q3) ",
            "(@NormalQuotMul.{u} G one mul inv N group_args normal_args ",
            "(@NormalQuotMk.{u} G one mul inv N group_args normal_args a) ",
            "(@NormalQuotMul.{u} G one mul inv N group_args normal_args ",
            "(@NormalQuotMk.{u} G one mul inv N group_args normal_args b) q3))) ",
            "(fun (c : G) => ",
            "@normal_quot_sound.{u} G one mul inv N group_args normal_args (mul (mul a b) c) (mul a (mul b c)) ",
            "(@normal_rel_of_eq.{succ u} G one mul inv group_args N normal_args (mul (mul a b) c) (mul a (mul b c)) ",
            "(@group_mul_assoc.{succ u} G one mul inv group_args a b c)))))"
        ),
    },
    TheoremArtifact {
        name: "normal_quot_one_mul",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (q : @NormalQuot.{u} G one mul inv N group_args normal_args), ",
            "@Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotMul.{u} G one mul inv N group_args normal_args ",
            "(@NormalQuotOne.{u} G one mul inv N group_args normal_args) q) q"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => ",
            "@Quotient.indProp.{u} G (@NormalSetoid.{u} G one mul inv N group_args normal_args) ",
            "(fun (q : @NormalQuot.{u} G one mul inv N group_args normal_args) => ",
            "@Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotMul.{u} G one mul inv N group_args normal_args ",
            "(@NormalQuotOne.{u} G one mul inv N group_args normal_args) q) q) ",
            "(fun (a : G) => ",
            "@normal_quot_sound.{u} G one mul inv N group_args normal_args (mul one a) a ",
            "(@normal_rel_of_eq.{succ u} G one mul inv group_args N normal_args (mul one a) a ",
            "(@group_one_mul.{succ u} G one mul inv group_args a)))"
        ),
    },
    TheoremArtifact {
        name: "normal_quot_mul_one",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (q : @NormalQuot.{u} G one mul inv N group_args normal_args), ",
            "@Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotMul.{u} G one mul inv N group_args normal_args ",
            "q (@NormalQuotOne.{u} G one mul inv N group_args normal_args)) q"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => ",
            "@Quotient.indProp.{u} G (@NormalSetoid.{u} G one mul inv N group_args normal_args) ",
            "(fun (q : @NormalQuot.{u} G one mul inv N group_args normal_args) => ",
            "@Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotMul.{u} G one mul inv N group_args normal_args ",
            "q (@NormalQuotOne.{u} G one mul inv N group_args normal_args)) q) ",
            "(fun (a : G) => ",
            "@normal_quot_sound.{u} G one mul inv N group_args normal_args (mul a one) a ",
            "(@normal_rel_of_eq.{succ u} G one mul inv group_args N normal_args (mul a one) a ",
            "(@group_mul_one.{succ u} G one mul inv group_args a)))"
        ),
    },
    TheoremArtifact {
        name: "normal_quot_inv_mul",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (q : @NormalQuot.{u} G one mul inv N group_args normal_args), ",
            "@Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotMul.{u} G one mul inv N group_args normal_args ",
            "(@NormalQuotInv.{u} G one mul inv N group_args normal_args q) q) ",
            "(@NormalQuotOne.{u} G one mul inv N group_args normal_args)"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => ",
            "@Quotient.indProp.{u} G (@NormalSetoid.{u} G one mul inv N group_args normal_args) ",
            "(fun (q : @NormalQuot.{u} G one mul inv N group_args normal_args) => ",
            "@Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotMul.{u} G one mul inv N group_args normal_args ",
            "(@NormalQuotInv.{u} G one mul inv N group_args normal_args q) q) ",
            "(@NormalQuotOne.{u} G one mul inv N group_args normal_args)) ",
            "(fun (a : G) => ",
            "@normal_quot_sound.{u} G one mul inv N group_args normal_args (mul (inv a) a) one ",
            "(@normal_rel_of_eq.{succ u} G one mul inv group_args N normal_args (mul (inv a) a) one ",
            "(@group_inv_mul.{succ u} G one mul inv group_args a)))"
        ),
    },
    TheoremArtifact {
        name: "normal_quot_mul_inv",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (q : @NormalQuot.{u} G one mul inv N group_args normal_args), ",
            "@Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotMul.{u} G one mul inv N group_args normal_args ",
            "q (@NormalQuotInv.{u} G one mul inv N group_args normal_args q)) ",
            "(@NormalQuotOne.{u} G one mul inv N group_args normal_args)"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => ",
            "@Quotient.indProp.{u} G (@NormalSetoid.{u} G one mul inv N group_args normal_args) ",
            "(fun (q : @NormalQuot.{u} G one mul inv N group_args normal_args) => ",
            "@Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotMul.{u} G one mul inv N group_args normal_args ",
            "q (@NormalQuotInv.{u} G one mul inv N group_args normal_args q)) ",
            "(@NormalQuotOne.{u} G one mul inv N group_args normal_args)) ",
            "(fun (a : G) => ",
            "@normal_quot_sound.{u} G one mul inv N group_args normal_args (mul a (inv a)) one ",
            "(@normal_rel_of_eq.{succ u} G one mul inv group_args N normal_args (mul a (inv a)) one ",
            "(@group_mul_inv.{succ u} G one mul inv group_args a)))"
        ),
    },
];

const ABSTRACT_GROUP_SECOND_ISO_PHI_DEFINITIONS: &[DefinitionArtifact] = &[DefinitionArtifact {
    name: "SecondIsoPhi",
    universe_params: &["u"],
    ty: concat!(
        "forall (G : Sort succ u), ",
        "forall (one : G), ",
        "forall (mul : forall (a : G), forall (b : G), G), ",
        "forall (inv : forall (a : G), G), ",
        "forall (N : forall (x : G), Prop), ",
        "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
        "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
        "forall (Hpred : forall (x : G), Prop), ",
        "forall (h : G), ",
        "forall (hh : Hpred h), ",
        "@NormalQuot.{u} G one mul inv N group_args normal_args"
    ),
    value: concat!(
        "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => fun Hpred => fun h => fun hh => ",
        "@NormalQuotMk.{u} G one mul inv N group_args normal_args h"
    ),
}];

const ABSTRACT_GROUP_SECOND_ISO_PHI_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "second_iso_phi_mk",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (h : G), ",
            "forall (hh : Hpred h), ",
            "@Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@SecondIsoPhi.{u} G one mul inv N group_args normal_args Hpred h hh) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args normal_args h)"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => fun Hpred => fun h => fun hh => ",
            "@Eq.refl.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args normal_args h)"
        ),
    },
    TheoremArtifact {
        name: "second_iso_phi_mul",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (h_args : @SubgroupLawArgs.{succ u} G one mul inv Hpred), ",
            "forall (a : G), forall (b : G), ",
            "forall (ha : Hpred a), forall (hb : Hpred b), ",
            "@Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotMul.{u} G one mul inv N group_args normal_args ",
            "(@SecondIsoPhi.{u} G one mul inv N group_args normal_args Hpred a ha) ",
            "(@SecondIsoPhi.{u} G one mul inv N group_args normal_args Hpred b hb)) ",
            "(@SecondIsoPhi.{u} G one mul inv N group_args normal_args Hpred (mul a b) ",
            "(@subgroup_mul_closed.{succ u} G one mul inv Hpred h_args a b ha hb))"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => fun Hpred => fun h_args => fun a => fun b => fun ha => fun hb => ",
            "@Eq.refl.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args normal_args (mul a b))"
        ),
    },
    TheoremArtifact {
        name: "second_iso_phi_one",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (h_args : @SubgroupLawArgs.{succ u} G one mul inv Hpred), ",
            "@Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@SecondIsoPhi.{u} G one mul inv N group_args normal_args Hpred one ",
            "(@subgroup_one.{succ u} G one mul inv Hpred h_args)) ",
            "(@NormalQuotOne.{u} G one mul inv N group_args normal_args)"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => fun Hpred => fun h_args => ",
            "@Eq.refl.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args normal_args one)"
        ),
    },
    TheoremArtifact {
        name: "second_iso_phi_inv",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (h_args : @SubgroupLawArgs.{succ u} G one mul inv Hpred), ",
            "forall (h : G), ",
            "forall (hh : Hpred h), ",
            "@Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotInv.{u} G one mul inv N group_args normal_args ",
            "(@SecondIsoPhi.{u} G one mul inv N group_args normal_args Hpred h hh)) ",
            "(@SecondIsoPhi.{u} G one mul inv N group_args normal_args Hpred (inv h) ",
            "(@subgroup_inv_closed.{succ u} G one mul inv Hpred h_args h hh))"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => fun Hpred => fun h_args => fun h => fun hh => ",
            "@Eq.refl.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args normal_args (inv h))"
        ),
    },
];

const ABSTRACT_GROUP_SECOND_ISO_KERNEL_DEFINITIONS: &[DefinitionArtifact] = &[DefinitionArtifact {
    name: "SecondIsoKernelPred",
    universe_params: &["u"],
    ty: concat!(
        "forall (G : Sort succ u), ",
        "forall (one : G), ",
        "forall (mul : forall (a : G), forall (b : G), G), ",
        "forall (inv : forall (a : G), G), ",
        "forall (N : forall (x : G), Prop), ",
        "forall (h : G), ",
        "Prop"
    ),
    value: concat!(
        "fun G => fun one => fun mul => fun inv => fun N => fun h => ",
        "@NormalRel.{succ u} G one mul inv N h one"
    ),
}];

const ABSTRACT_GROUP_SECOND_ISO_KERNEL_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "second_iso_kernel_sound",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (h : G), ",
            "forall (hh : Hpred h), ",
            "forall (hk : @SecondIsoKernelPred.{u} G one mul inv N h), ",
            "@Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@SecondIsoPhi.{u} G one mul inv N group_args normal_args Hpred h hh) ",
            "(@NormalQuotOne.{u} G one mul inv N group_args normal_args)"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => fun Hpred => fun h => fun hh => fun hk => ",
            "@normal_quot_sound.{u} G one mul inv N group_args normal_args h one hk"
        ),
    },
    TheoremArtifact {
        name: "second_iso_kernel_to_inter",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (h : G), ",
            "forall (hh : Hpred h), ",
            "forall (hk : @SecondIsoKernelPred.{u} G one mul inv N h), ",
            "@SubgroupInterPred.{succ u} G Hpred N h"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => fun Hpred => fun h => fun hh => fun hk => ",
            "@subgroup_inter_intro.{succ u} G Hpred N h hh ",
            "(@normal_rel_one_to_mem.{succ u} G one mul inv group_args N normal_args h hk)"
        ),
    },
    TheoremArtifact {
        name: "second_iso_inter_to_kernel",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (h : G), ",
            "forall (hi : @SubgroupInterPred.{succ u} G Hpred N h), ",
            "@SecondIsoKernelPred.{u} G one mul inv N h"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => fun Hpred => fun h => fun hi => ",
            "@normal_rel_one_of_mem.{succ u} G one mul inv group_args N normal_args h ",
            "(@subgroup_inter_right.{succ u} G Hpred N h hi)"
        ),
    },
];

const ABSTRACT_GROUP_SECOND_ISO_IMAGE_DEFINITIONS: &[DefinitionArtifact] = &[
    DefinitionArtifact {
        name: "SecondIsoImagePred",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (q : @NormalQuot.{u} G one mul inv N group_args normal_args), ",
            "Prop"
        ),
        value: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => fun Hpred => fun q => ",
            "forall (P : Prop), ",
            "forall (mk : forall (h : G), forall (hh : Hpred h), ",
            "forall (eq_phi : @Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@SecondIsoPhi.{u} G one mul inv N group_args normal_args Hpred h hh) q), P), ",
            "P"
        ),
    },
    DefinitionArtifact {
        name: "SecondIsoProductQuotPred",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (q : @NormalQuot.{u} G one mul inv N group_args normal_args), ",
            "Prop"
        ),
        value: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => fun Hpred => fun q => ",
            "forall (P : Prop), ",
            "forall (mk : forall (x : G), ",
            "forall (hx : @SubgroupProductPred.{succ u} G mul Hpred N x), ",
            "forall (eq_q : @Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args normal_args x) q), P), ",
            "P"
        ),
    },
];

const ABSTRACT_GROUP_SECOND_ISO_IMAGE_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "second_iso_image_intro",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (h : G), ",
            "forall (hh : Hpred h), ",
            "@SecondIsoImagePred.{u} G one mul inv N group_args normal_args Hpred ",
            "(@SecondIsoPhi.{u} G one mul inv N group_args normal_args Hpred h hh)"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => fun Hpred => fun h => fun hh => ",
            "fun (P : Prop) => fun (mk : forall (h2 : G), forall (hh2 : Hpred h2), forall (eq_phi : @Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) (@SecondIsoPhi.{u} G one mul inv N group_args normal_args Hpred h2 hh2) (@SecondIsoPhi.{u} G one mul inv N group_args normal_args Hpred h hh)), P) => ",
            "mk h hh (@Eq.refl.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@SecondIsoPhi.{u} G one mul inv N group_args normal_args Hpred h hh))"
        ),
    },
    TheoremArtifact {
        name: "second_iso_image_elim",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (q : @NormalQuot.{u} G one mul inv N group_args normal_args), ",
            "forall (img : @SecondIsoImagePred.{u} G one mul inv N group_args normal_args Hpred q), ",
            "forall (P : Prop), ",
            "forall (mk : forall (h : G), forall (hh : Hpred h), forall (eq_phi : @Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) (@SecondIsoPhi.{u} G one mul inv N group_args normal_args Hpred h hh) q), P), ",
            "P"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => fun Hpred => fun q => fun img => fun P => fun mk => ",
            "img P mk"
        ),
    },
    TheoremArtifact {
        name: "second_iso_product_quot_intro",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (q : @NormalQuot.{u} G one mul inv N group_args normal_args), ",
            "forall (x : G), ",
            "forall (hx : @SubgroupProductPred.{succ u} G mul Hpred N x), ",
            "forall (eq_q : @Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args normal_args x) q), ",
            "@SecondIsoProductQuotPred.{u} G one mul inv N group_args normal_args Hpred q"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => fun Hpred => fun q => fun x => fun hx => fun eq_q => ",
            "fun (P : Prop) => fun (mk : forall (x2 : G), forall (hx2 : @SubgroupProductPred.{succ u} G mul Hpred N x2), forall (eq_q2 : @Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) (@NormalQuotMk.{u} G one mul inv N group_args normal_args x2) q), P) => ",
            "mk x hx eq_q"
        ),
    },
    TheoremArtifact {
        name: "second_iso_product_quot_elim",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (q : @NormalQuot.{u} G one mul inv N group_args normal_args), ",
            "forall (prod : @SecondIsoProductQuotPred.{u} G one mul inv N group_args normal_args Hpred q), ",
            "forall (P : Prop), ",
            "forall (mk : forall (x : G), forall (hx : @SubgroupProductPred.{succ u} G mul Hpred N x), forall (eq_q : @Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) (@NormalQuotMk.{u} G one mul inv N group_args normal_args x) q), P), ",
            "P"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => fun Hpred => fun q => fun prod => fun P => fun mk => ",
            "prod P mk"
        ),
    },
    TheoremArtifact {
        name: "second_iso_image_to_product_quot",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (q : @NormalQuot.{u} G one mul inv N group_args normal_args), ",
            "forall (img : @SecondIsoImagePred.{u} G one mul inv N group_args normal_args Hpred q), ",
            "@SecondIsoProductQuotPred.{u} G one mul inv N group_args normal_args Hpred q"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => fun Hpred => fun q => fun img => ",
            "img (@SecondIsoProductQuotPred.{u} G one mul inv N group_args normal_args Hpred q) ",
            "(fun (h : G) => fun (hh : Hpred h) => fun (eq_phi : @Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) (@SecondIsoPhi.{u} G one mul inv N group_args normal_args Hpred h hh) q) => ",
            "@second_iso_product_quot_intro.{u} G one mul inv N group_args normal_args Hpred q h ",
            "(@subgroup_product_intro.{succ u} G mul Hpred N h h one hh ",
            "(@subgroup_one.{succ u} G one mul inv N (@normal_subgroup_laws.{succ u} G one mul inv N normal_args)) ",
            "(@group_mul_one.{succ u} G one mul inv group_args h)) ",
            "(@eq_trans.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args normal_args h) ",
            "(@SecondIsoPhi.{u} G one mul inv N group_args normal_args Hpred h hh) q ",
            "(@eq_symm.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@SecondIsoPhi.{u} G one mul inv N group_args normal_args Hpred h hh) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args normal_args h) ",
            "(@second_iso_phi_mk.{u} G one mul inv N group_args normal_args Hpred h hh)) ",
            "eq_phi))"
        ),
    },
    TheoremArtifact {
        name: "second_iso_product_quot_to_image",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (q : @NormalQuot.{u} G one mul inv N group_args normal_args), ",
            "forall (prod : @SecondIsoProductQuotPred.{u} G one mul inv N group_args normal_args Hpred q), ",
            "@SecondIsoImagePred.{u} G one mul inv N group_args normal_args Hpred q"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => fun Hpred => fun q => fun prod => ",
            "prod (@SecondIsoImagePred.{u} G one mul inv N group_args normal_args Hpred q) ",
            "(fun (x : G) => fun (hx : @SubgroupProductPred.{succ u} G mul Hpred N x) => ",
            "fun (eq_q : @Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) (@NormalQuotMk.{u} G one mul inv N group_args normal_args x) q) => ",
            "hx (@SecondIsoImagePred.{u} G one mul inv N group_args normal_args Hpred q) ",
            "(fun (h : G) => fun (n : G) => fun (hh : Hpred h) => fun (hn : N n) => fun (hx_eq : @Eq.{succ u} G (mul h n) x) => ",
            "fun (P : Prop) => fun (mk : forall (h2 : G), forall (hh2 : Hpred h2), forall (eq_phi : @Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) (@SecondIsoPhi.{u} G one mul inv N group_args normal_args Hpred h2 hh2) q), P) => mk h hh ",
            "(@eq_trans.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@SecondIsoPhi.{u} G one mul inv N group_args normal_args Hpred h hh) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args normal_args h) q ",
            "(@second_iso_phi_mk.{u} G one mul inv N group_args normal_args Hpred h hh) ",
            "(@eq_trans.{succ u} (@NormalQuot.{u} G one mul inv N group_args normal_args) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args normal_args h) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args normal_args x) q ",
            "(@normal_quot_sound.{u} G one mul inv N group_args normal_args h x ",
            "(@normal_rel_product_right.{succ u} G one mul inv group_args N h n x hn hx_eq)) ",
            "eq_q))))"
        ),
    },
];

const ABSTRACT_GROUP_SECOND_ISO_FINAL_DEFINITIONS: &[DefinitionArtifact] = &[
    DefinitionArtifact {
        name: "SecondIsoKernelEvidence",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "Prop"
        ),
        value: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => fun Hpred => ",
            "forall (P : Prop), ",
            "forall (mk : ",
            "forall (to_inter : forall (h : G), forall (hh : Hpred h), forall (hk : @SecondIsoKernelPred.{u} G one mul inv N h), @SubgroupInterPred.{succ u} G Hpred N h), ",
            "forall (to_kernel : forall (h : G), forall (hi : @SubgroupInterPred.{succ u} G Hpred N h), @SecondIsoKernelPred.{u} G one mul inv N h), ",
            "P), ",
            "P"
        ),
    },
    DefinitionArtifact {
        name: "SecondIsoImageEvidence",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "Prop"
        ),
        value: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => fun Hpred => ",
            "forall (P : Prop), ",
            "forall (mk : ",
            "forall (image_to_product : forall (q : @NormalQuot.{u} G one mul inv N group_args normal_args), forall (img : @SecondIsoImagePred.{u} G one mul inv N group_args normal_args Hpred q), @SecondIsoProductQuotPred.{u} G one mul inv N group_args normal_args Hpred q), ",
            "forall (product_to_image : forall (q : @NormalQuot.{u} G one mul inv N group_args normal_args), forall (prod : @SecondIsoProductQuotPred.{u} G one mul inv N group_args normal_args Hpred q), @SecondIsoImagePred.{u} G one mul inv N group_args normal_args Hpred q), ",
            "P), ",
            "P"
        ),
    },
    DefinitionArtifact {
        name: "SecondIsoTheoremEvidence",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (h_args : @SubgroupLawArgs.{succ u} G one mul inv Hpred), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "Prop"
        ),
        value: concat!(
            "fun G => fun one => fun mul => fun inv => fun Hpred => fun N => fun group_args => fun h_args => fun normal_args => ",
            "forall (P : Prop), ",
            "forall (mk : ",
            "forall (kernel_evidence : @SecondIsoKernelEvidence.{u} G one mul inv N group_args normal_args Hpred), ",
            "forall (image_evidence : @SecondIsoImageEvidence.{u} G one mul inv N group_args normal_args Hpred), ",
            "P), ",
            "P"
        ),
    },
];

const ABSTRACT_GROUP_SECOND_ISO_FINAL_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "second_iso_kernel_evidence",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "@SecondIsoKernelEvidence.{u} G one mul inv N group_args normal_args Hpred"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => fun Hpred => ",
            "fun (P : Prop) => ",
            "fun (mk : forall (to_inter : forall (h : G), forall (hh : Hpred h), forall (hk : @SecondIsoKernelPred.{u} G one mul inv N h), @SubgroupInterPred.{succ u} G Hpred N h), forall (to_kernel : forall (h : G), forall (hi : @SubgroupInterPred.{succ u} G Hpred N h), @SecondIsoKernelPred.{u} G one mul inv N h), P) => ",
            "mk ",
            "(@second_iso_kernel_to_inter.{u} G one mul inv N group_args normal_args Hpred) ",
            "(@second_iso_inter_to_kernel.{u} G one mul inv N group_args normal_args Hpred)"
        ),
    },
    TheoremArtifact {
        name: "second_iso_image_evidence",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "@SecondIsoImageEvidence.{u} G one mul inv N group_args normal_args Hpred"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun normal_args => fun Hpred => ",
            "fun (P : Prop) => ",
            "fun (mk : forall (image_to_product : forall (q : @NormalQuot.{u} G one mul inv N group_args normal_args), forall (img : @SecondIsoImagePred.{u} G one mul inv N group_args normal_args Hpred q), @SecondIsoProductQuotPred.{u} G one mul inv N group_args normal_args Hpred q), forall (product_to_image : forall (q : @NormalQuot.{u} G one mul inv N group_args normal_args), forall (prod : @SecondIsoProductQuotPred.{u} G one mul inv N group_args normal_args Hpred q), @SecondIsoImagePred.{u} G one mul inv N group_args normal_args Hpred q), P) => ",
            "mk ",
            "(@second_iso_image_to_product_quot.{u} G one mul inv N group_args normal_args Hpred) ",
            "(@second_iso_product_quot_to_image.{u} G one mul inv N group_args normal_args Hpred)"
        ),
    },
    TheoremArtifact {
        name: "second_isomorphism_theorem_evidence",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (h_args : @SubgroupLawArgs.{succ u} G one mul inv Hpred), ",
            "forall (normal_args : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "@SecondIsoTheoremEvidence.{u} G one mul inv Hpred N group_args h_args normal_args"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun Hpred => fun N => fun group_args => fun h_args => fun normal_args => ",
            "fun (P : Prop) => ",
            "fun (mk : forall (kernel_evidence : @SecondIsoKernelEvidence.{u} G one mul inv N group_args normal_args Hpred), forall (image_evidence : @SecondIsoImageEvidence.{u} G one mul inv N group_args normal_args Hpred), P) => ",
            "mk ",
            "(@second_iso_kernel_evidence.{u} G one mul inv N group_args normal_args Hpred) ",
            "(@second_iso_image_evidence.{u} G one mul inv N group_args normal_args Hpred)"
        ),
    },
];

const ABSTRACT_GROUP_THIRD_ISO_DEFINITIONS: &[DefinitionArtifact] = &[
    DefinitionArtifact {
        name: "ThirdIsoGN",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "Sort succ u"
        ),
        value: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun n_normal => ",
            "@NormalQuot.{u} G one mul inv N group_args n_normal"
        ),
    },
    DefinitionArtifact {
        name: "ThirdIsoGNOne",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "@ThirdIsoGN.{u} G one mul inv N group_args n_normal"
        ),
        value: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun n_normal => ",
            "@NormalQuotOne.{u} G one mul inv N group_args n_normal"
        ),
    },
    DefinitionArtifact {
        name: "ThirdIsoGNMul",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (q1 : @ThirdIsoGN.{u} G one mul inv N group_args n_normal), ",
            "forall (q2 : @ThirdIsoGN.{u} G one mul inv N group_args n_normal), ",
            "@ThirdIsoGN.{u} G one mul inv N group_args n_normal"
        ),
        value: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun n_normal => ",
            "@NormalQuotMul.{u} G one mul inv N group_args n_normal"
        ),
    },
    DefinitionArtifact {
        name: "ThirdIsoGNInv",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (q : @ThirdIsoGN.{u} G one mul inv N group_args n_normal), ",
            "@ThirdIsoGN.{u} G one mul inv N group_args n_normal"
        ),
        value: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun group_args => fun n_normal => ",
            "@NormalQuotInv.{u} G one mul inv N group_args n_normal"
        ),
    },
    DefinitionArtifact {
        name: "ThirdIsoHNPred",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (q : @ThirdIsoGN.{u} G one mul inv N group_args n_normal), ",
            "Prop"
        ),
        value: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun Hpred => fun group_args => fun n_normal => fun q => ",
            "forall (P : Prop), ",
            "forall (mk : forall (h : G), forall (hh : Hpred h), forall (eq_q : @Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args n_normal) (@NormalQuotMk.{u} G one mul inv N group_args n_normal h) q), P), ",
            "P"
        ),
    },
    DefinitionArtifact {
        name: "ThirdIsoHNSubgroupLawArgs",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "Prop"
        ),
        value: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun Hpred => fun group_args => fun n_normal => ",
            "@SubgroupLawArgs.{succ u} ",
            "(@ThirdIsoGN.{u} G one mul inv N group_args n_normal) ",
            "(@ThirdIsoGNOne.{u} G one mul inv N group_args n_normal) ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal) ",
            "(@ThirdIsoGNInv.{u} G one mul inv N group_args n_normal) ",
            "(@ThirdIsoHNPred.{u} G one mul inv N Hpred group_args n_normal)"
        ),
    },
    DefinitionArtifact {
        name: "ThirdIsoHNNormalSubgroupLawArgs",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "Prop"
        ),
        value: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun Hpred => fun group_args => fun n_normal => ",
            "@NormalSubgroupLawArgs.{succ u} ",
            "(@ThirdIsoGN.{u} G one mul inv N group_args n_normal) ",
            "(@ThirdIsoGNOne.{u} G one mul inv N group_args n_normal) ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal) ",
            "(@ThirdIsoGNInv.{u} G one mul inv N group_args n_normal) ",
            "(@ThirdIsoHNPred.{u} G one mul inv N Hpred group_args n_normal)"
        ),
    },
    DefinitionArtifact {
        name: "ThirdIsoPhi",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (h_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv Hpred), ",
            "forall (n_le_h : forall (x : G), forall (hn : N x), Hpred x), ",
            "forall (q : @NormalQuot.{u} G one mul inv N group_args n_normal), ",
            "@NormalQuot.{u} G one mul inv Hpred group_args h_normal"
        ),
        value: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun Hpred => fun group_args => fun n_normal => fun h_normal => fun n_le_h => ",
            "@Quotient.lift.{u,u} G (@NormalQuot.{u} G one mul inv Hpred group_args h_normal) ",
            "(@NormalSetoid.{u} G one mul inv N group_args n_normal) ",
            "(fun (a : G) => @NormalQuotMk.{u} G one mul inv Hpred group_args h_normal a) ",
            "(fun (a : G) => fun (b : G) => fun (h : @Setoid.r.{u} G (@NormalSetoid.{u} G one mul inv N group_args n_normal) a b) => ",
            "@normal_quot_sound.{u} G one mul inv Hpred group_args h_normal a b ",
            "(n_le_h (mul (inv a) b) h))"
        ),
    },
    DefinitionArtifact {
        name: "ThirdIsoPhiKernelQuot",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (h_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv Hpred), ",
            "forall (n_le_h : forall (x : G), forall (hn : N x), Hpred x), ",
            "Sort succ u"
        ),
        value: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun Hpred => fun group_args => fun n_normal => fun h_normal => fun n_le_h => ",
            "@KerQuot.{u,u} ",
            "(@ThirdIsoGN.{u} G one mul inv N group_args n_normal) ",
            "(@NormalQuot.{u} G one mul inv Hpred group_args h_normal) ",
            "(@ThirdIsoPhi.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h)"
        ),
    },
    DefinitionArtifact {
        name: "ThirdIsoKernelPred",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (q : @NormalQuot.{u} G one mul inv N group_args n_normal), ",
            "Prop"
        ),
        value: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun Hpred => fun group_args => fun n_normal => fun q => ",
            "@ThirdIsoHNPred.{u} G one mul inv N Hpred group_args n_normal q"
        ),
    },
    DefinitionArtifact {
        name: "ThirdIsoKernelEvidence",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (h_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv Hpred), ",
            "forall (n_le_h : forall (x : G), forall (hn : N x), Hpred x), ",
            "Prop"
        ),
        value: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun Hpred => fun group_args => fun n_normal => fun h_normal => fun n_le_h => ",
            "forall (P : Prop), ",
            "forall (mk : ",
            "forall (kernel_sound : forall (q : @NormalQuot.{u} G one mul inv N group_args n_normal), forall (hk : @ThirdIsoKernelPred.{u} G one mul inv N Hpred group_args n_normal q), @Eq.{succ u} (@NormalQuot.{u} G one mul inv Hpred group_args h_normal) (@ThirdIsoPhi.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h q) (@NormalQuotOne.{u} G one mul inv Hpred group_args h_normal)), ",
            "forall (kernel_intro : forall (a : G), forall (ha : Hpred a), @ThirdIsoKernelPred.{u} G one mul inv N Hpred group_args n_normal (@NormalQuotMk.{u} G one mul inv N group_args n_normal a)), ",
            "P), ",
            "P"
        ),
    },
    DefinitionArtifact {
        name: "ThirdIsoTheoremEvidence",
        universe_params: &["u"],
        ty: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (h_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv Hpred), ",
            "forall (n_le_h : forall (x : G), forall (hn : N x), Hpred x), ",
            "Prop"
        ),
        value: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun Hpred => fun group_args => fun n_normal => fun h_normal => fun n_le_h => ",
            "forall (P : Prop), ",
            "forall (mk : ",
            "forall (rel_lift : forall (a : G), forall (b : G), forall (h : @NormalRel.{succ u} G one mul inv N a b), @NormalRel.{succ u} G one mul inv Hpred a b), ",
            "forall (surjective : forall (q : @NormalQuot.{u} G one mul inv Hpred group_args h_normal), forall (Q : Prop), forall (mk_surj : forall (src : @NormalQuot.{u} G one mul inv N group_args n_normal), forall (h : @Eq.{succ u} (@NormalQuot.{u} G one mul inv Hpred group_args h_normal) (@ThirdIsoPhi.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h src) q), Q), Q), ",
            "forall (kernel_evidence : @ThirdIsoKernelEvidence.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h), ",
            "P), ",
            "P"
        ),
    },
];

const ABSTRACT_GROUP_THIRD_ISO_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "third_iso_rel_lift",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (n_le_h : forall (x : G), forall (hn : N x), Hpred x), ",
            "forall (a : G), forall (b : G), ",
            "forall (h : @NormalRel.{succ u} G one mul inv N a b), ",
            "@NormalRel.{succ u} G one mul inv Hpred a b"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun Hpred => fun n_le_h => fun a => fun b => fun h => ",
            "n_le_h (mul (inv a) b) h"
        ),
    },
    TheoremArtifact {
        name: "third_iso_hn_intro",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (q : @NormalQuot.{u} G one mul inv N group_args n_normal), ",
            "forall (h : G), forall (hh : Hpred h), ",
            "forall (eq_q : @Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args n_normal) (@NormalQuotMk.{u} G one mul inv N group_args n_normal h) q), ",
            "@ThirdIsoHNPred.{u} G one mul inv N Hpred group_args n_normal q"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun Hpred => fun group_args => fun n_normal => fun q => fun h => fun hh => fun eq_q => ",
            "fun (P : Prop) => fun (mk : forall (h2 : G), forall (hh2 : Hpred h2), forall (eq_q2 : @Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args n_normal) (@NormalQuotMk.{u} G one mul inv N group_args n_normal h2) q), P) => ",
            "mk h hh eq_q"
        ),
    },
    TheoremArtifact {
        name: "third_iso_hn_elim",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (q : @NormalQuot.{u} G one mul inv N group_args n_normal), ",
            "forall (hn : @ThirdIsoHNPred.{u} G one mul inv N Hpred group_args n_normal q), ",
            "forall (P : Prop), ",
            "forall (mk : forall (h : G), forall (hh : Hpred h), forall (eq_q : @Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args n_normal) (@NormalQuotMk.{u} G one mul inv N group_args n_normal h) q), P), ",
            "P"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun Hpred => fun group_args => fun n_normal => fun q => fun hn => fun P => fun mk => ",
            "hn P mk"
        ),
    },
    TheoremArtifact {
        name: "third_iso_hn_one",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (h_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv Hpred), ",
            "@ThirdIsoHNPred.{u} G one mul inv N Hpred group_args n_normal ",
            "(@ThirdIsoGNOne.{u} G one mul inv N group_args n_normal)"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun Hpred => fun group_args => fun n_normal => fun h_normal => ",
            "@third_iso_hn_intro.{u} G one mul inv N Hpred group_args n_normal ",
            "(@ThirdIsoGNOne.{u} G one mul inv N group_args n_normal) ",
            "one ",
            "(@subgroup_one.{succ u} G one mul inv Hpred (@normal_subgroup_laws.{succ u} G one mul inv Hpred h_normal)) ",
            "(@Eq.refl.{succ u} (@ThirdIsoGN.{u} G one mul inv N group_args n_normal) (@ThirdIsoGNOne.{u} G one mul inv N group_args n_normal))"
        ),
    },
    TheoremArtifact {
        name: "third_iso_hn_mul_closed",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (h_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv Hpred), ",
            "forall (q1 : @ThirdIsoGN.{u} G one mul inv N group_args n_normal), ",
            "forall (q2 : @ThirdIsoGN.{u} G one mul inv N group_args n_normal), ",
            "forall (hq1 : @ThirdIsoHNPred.{u} G one mul inv N Hpred group_args n_normal q1), ",
            "forall (hq2 : @ThirdIsoHNPred.{u} G one mul inv N Hpred group_args n_normal q2), ",
            "@ThirdIsoHNPred.{u} G one mul inv N Hpred group_args n_normal ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal q1 q2)"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun Hpred => fun group_args => fun n_normal => fun h_normal => fun q1 => fun q2 => fun hq1 => fun hq2 => ",
            "hq1 (@ThirdIsoHNPred.{u} G one mul inv N Hpred group_args n_normal (@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal q1 q2)) ",
            "(fun (a : G) => fun (ha : Hpred a) => fun (eqa : @Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args n_normal) (@NormalQuotMk.{u} G one mul inv N group_args n_normal a) q1) => ",
            "hq2 (@ThirdIsoHNPred.{u} G one mul inv N Hpred group_args n_normal (@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal q1 q2)) ",
            "(fun (b : G) => fun (hb : Hpred b) => fun (eqb : @Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args n_normal) (@NormalQuotMk.{u} G one mul inv N group_args n_normal b) q2) => ",
            "@third_iso_hn_intro.{u} G one mul inv N Hpred group_args n_normal ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal q1 q2) ",
            "(mul a b) ",
            "(@subgroup_mul_closed.{succ u} G one mul inv Hpred (@normal_subgroup_laws.{succ u} G one mul inv Hpred h_normal) a b ha hb) ",
            "(@eq_trans.{succ u} (@NormalQuot.{u} G one mul inv N group_args n_normal) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args n_normal (mul a b)) ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal (@NormalQuotMk.{u} G one mul inv N group_args n_normal a) (@NormalQuotMk.{u} G one mul inv N group_args n_normal b)) ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal q1 q2) ",
            "(@eq_symm.{succ u} (@NormalQuot.{u} G one mul inv N group_args n_normal) ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal (@NormalQuotMk.{u} G one mul inv N group_args n_normal a) (@NormalQuotMk.{u} G one mul inv N group_args n_normal b)) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args n_normal (mul a b)) ",
            "(@normal_quot_mul_mk.{u} G one mul inv N group_args n_normal a b)) ",
            "(@eq_congr2.{succ u,succ u,succ u} (@NormalQuot.{u} G one mul inv N group_args n_normal) (@NormalQuot.{u} G one mul inv N group_args n_normal) (@NormalQuot.{u} G one mul inv N group_args n_normal) ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args n_normal a) q1 ",
            "(@NormalQuotMk.{u} G one mul inv N group_args n_normal b) q2 eqa eqb))))"
        ),
    },
    TheoremArtifact {
        name: "third_iso_hn_inv_closed",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (h_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv Hpred), ",
            "forall (q : @ThirdIsoGN.{u} G one mul inv N group_args n_normal), ",
            "forall (hq : @ThirdIsoHNPred.{u} G one mul inv N Hpred group_args n_normal q), ",
            "@ThirdIsoHNPred.{u} G one mul inv N Hpred group_args n_normal ",
            "(@ThirdIsoGNInv.{u} G one mul inv N group_args n_normal q)"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun Hpred => fun group_args => fun n_normal => fun h_normal => fun q => fun hq => ",
            "hq (@ThirdIsoHNPred.{u} G one mul inv N Hpred group_args n_normal (@ThirdIsoGNInv.{u} G one mul inv N group_args n_normal q)) ",
            "(fun (a : G) => fun (ha : Hpred a) => fun (eqa : @Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args n_normal) (@NormalQuotMk.{u} G one mul inv N group_args n_normal a) q) => ",
            "@third_iso_hn_intro.{u} G one mul inv N Hpred group_args n_normal ",
            "(@ThirdIsoGNInv.{u} G one mul inv N group_args n_normal q) ",
            "(inv a) ",
            "(@subgroup_inv_closed.{succ u} G one mul inv Hpred (@normal_subgroup_laws.{succ u} G one mul inv Hpred h_normal) a ha) ",
            "(@eq_trans.{succ u} (@NormalQuot.{u} G one mul inv N group_args n_normal) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args n_normal (inv a)) ",
            "(@ThirdIsoGNInv.{u} G one mul inv N group_args n_normal (@NormalQuotMk.{u} G one mul inv N group_args n_normal a)) ",
            "(@ThirdIsoGNInv.{u} G one mul inv N group_args n_normal q) ",
            "(@eq_symm.{succ u} (@NormalQuot.{u} G one mul inv N group_args n_normal) ",
            "(@ThirdIsoGNInv.{u} G one mul inv N group_args n_normal (@NormalQuotMk.{u} G one mul inv N group_args n_normal a)) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args n_normal (inv a)) ",
            "(@normal_quot_inv_mk.{u} G one mul inv N group_args n_normal a)) ",
            "(@eq_congr_arg.{succ u,succ u} (@NormalQuot.{u} G one mul inv N group_args n_normal) (@NormalQuot.{u} G one mul inv N group_args n_normal) ",
            "(@ThirdIsoGNInv.{u} G one mul inv N group_args n_normal) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args n_normal a) q eqa)))"
        ),
    },
    TheoremArtifact {
        name: "third_iso_hn_conj_closed",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (h_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv Hpred), ",
            "forall (gq : @ThirdIsoGN.{u} G one mul inv N group_args n_normal), ",
            "forall (nq : @ThirdIsoGN.{u} G one mul inv N group_args n_normal), ",
            "forall (hnq : @ThirdIsoHNPred.{u} G one mul inv N Hpred group_args n_normal nq), ",
            "@ThirdIsoHNPred.{u} G one mul inv N Hpred group_args n_normal ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal gq nq) ",
            "(@ThirdIsoGNInv.{u} G one mul inv N group_args n_normal gq))"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun Hpred => fun group_args => fun n_normal => fun h_normal => ",
            "@Quotient.indProp.{u} G (@NormalSetoid.{u} G one mul inv N group_args n_normal) ",
            "(fun (gq : @NormalQuot.{u} G one mul inv N group_args n_normal) => ",
            "forall (nq : @ThirdIsoGN.{u} G one mul inv N group_args n_normal), ",
            "forall (hnq : @ThirdIsoHNPred.{u} G one mul inv N Hpred group_args n_normal nq), ",
            "@ThirdIsoHNPred.{u} G one mul inv N Hpred group_args n_normal ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal gq nq) ",
            "(@ThirdIsoGNInv.{u} G one mul inv N group_args n_normal gq))) ",
            "(fun (g : G) => fun (nq : @ThirdIsoGN.{u} G one mul inv N group_args n_normal) => fun (hnq : @ThirdIsoHNPred.{u} G one mul inv N Hpred group_args n_normal nq) => ",
            "hnq (@ThirdIsoHNPred.{u} G one mul inv N Hpred group_args n_normal ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal (@NormalQuotMk.{u} G one mul inv N group_args n_normal g) nq) ",
            "(@ThirdIsoGNInv.{u} G one mul inv N group_args n_normal (@NormalQuotMk.{u} G one mul inv N group_args n_normal g)))) ",
            "(fun (h : G) => fun (hh : Hpred h) => fun (eqh : @Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args n_normal) (@NormalQuotMk.{u} G one mul inv N group_args n_normal h) nq) => ",
            "@third_iso_hn_intro.{u} G one mul inv N Hpred group_args n_normal ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal (@NormalQuotMk.{u} G one mul inv N group_args n_normal g) nq) ",
            "(@ThirdIsoGNInv.{u} G one mul inv N group_args n_normal (@NormalQuotMk.{u} G one mul inv N group_args n_normal g))) ",
            "(mul (mul g h) (inv g)) ",
            "(@normal_conj_closed.{succ u} G one mul inv Hpred h_normal g h hh) ",
            "(@eq_trans.{succ u} (@NormalQuot.{u} G one mul inv N group_args n_normal) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args n_normal (mul (mul g h) (inv g))) ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal (@NormalQuotMk.{u} G one mul inv N group_args n_normal (mul g h)) (@NormalQuotMk.{u} G one mul inv N group_args n_normal (inv g))) ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal (@NormalQuotMk.{u} G one mul inv N group_args n_normal g) nq) ",
            "(@ThirdIsoGNInv.{u} G one mul inv N group_args n_normal (@NormalQuotMk.{u} G one mul inv N group_args n_normal g))) ",
            "(@eq_symm.{succ u} (@NormalQuot.{u} G one mul inv N group_args n_normal) ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal (@NormalQuotMk.{u} G one mul inv N group_args n_normal (mul g h)) (@NormalQuotMk.{u} G one mul inv N group_args n_normal (inv g))) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args n_normal (mul (mul g h) (inv g))) ",
            "(@normal_quot_mul_mk.{u} G one mul inv N group_args n_normal (mul g h) (inv g))) ",
            "(@eq_congr2.{succ u,succ u,succ u} (@NormalQuot.{u} G one mul inv N group_args n_normal) (@NormalQuot.{u} G one mul inv N group_args n_normal) (@NormalQuot.{u} G one mul inv N group_args n_normal) ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args n_normal (mul g h)) ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal (@NormalQuotMk.{u} G one mul inv N group_args n_normal g) nq) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args n_normal (inv g)) ",
            "(@ThirdIsoGNInv.{u} G one mul inv N group_args n_normal (@NormalQuotMk.{u} G one mul inv N group_args n_normal g)) ",
            "(@eq_trans.{succ u} (@NormalQuot.{u} G one mul inv N group_args n_normal) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args n_normal (mul g h)) ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal (@NormalQuotMk.{u} G one mul inv N group_args n_normal g) (@NormalQuotMk.{u} G one mul inv N group_args n_normal h)) ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal (@NormalQuotMk.{u} G one mul inv N group_args n_normal g) nq) ",
            "(@eq_symm.{succ u} (@NormalQuot.{u} G one mul inv N group_args n_normal) ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal (@NormalQuotMk.{u} G one mul inv N group_args n_normal g) (@NormalQuotMk.{u} G one mul inv N group_args n_normal h)) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args n_normal (mul g h)) ",
            "(@normal_quot_mul_mk.{u} G one mul inv N group_args n_normal g h)) ",
            "(@eq_congr2.{succ u,succ u,succ u} (@NormalQuot.{u} G one mul inv N group_args n_normal) (@NormalQuot.{u} G one mul inv N group_args n_normal) (@NormalQuot.{u} G one mul inv N group_args n_normal) ",
            "(@ThirdIsoGNMul.{u} G one mul inv N group_args n_normal) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args n_normal g) (@NormalQuotMk.{u} G one mul inv N group_args n_normal g) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args n_normal h) nq ",
            "(@Eq.refl.{succ u} (@NormalQuot.{u} G one mul inv N group_args n_normal) (@NormalQuotMk.{u} G one mul inv N group_args n_normal g)) eqh)) ",
            "(@eq_symm.{succ u} (@NormalQuot.{u} G one mul inv N group_args n_normal) ",
            "(@ThirdIsoGNInv.{u} G one mul inv N group_args n_normal (@NormalQuotMk.{u} G one mul inv N group_args n_normal g)) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args n_normal (inv g)) ",
            "(@normal_quot_inv_mk.{u} G one mul inv N group_args n_normal g))))))"
        ),
    },
    TheoremArtifact {
        name: "third_iso_phi_mk",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (h_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv Hpred), ",
            "forall (n_le_h : forall (x : G), forall (hn : N x), Hpred x), ",
            "forall (a : G), ",
            "@Eq.{succ u} (@NormalQuot.{u} G one mul inv Hpred group_args h_normal) ",
            "(@ThirdIsoPhi.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h (@NormalQuotMk.{u} G one mul inv N group_args n_normal a)) ",
            "(@NormalQuotMk.{u} G one mul inv Hpred group_args h_normal a)"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun Hpred => fun group_args => fun n_normal => fun h_normal => fun n_le_h => fun a => ",
            "@Eq.refl.{succ u} (@NormalQuot.{u} G one mul inv Hpred group_args h_normal) (@NormalQuotMk.{u} G one mul inv Hpred group_args h_normal a)"
        ),
    },
    TheoremArtifact {
        name: "third_iso_phi_mul",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (h_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv Hpred), ",
            "forall (n_le_h : forall (x : G), forall (hn : N x), Hpred x), ",
            "forall (a : G), forall (b : G), ",
            "@Eq.{succ u} (@NormalQuot.{u} G one mul inv Hpred group_args h_normal) ",
            "(@ThirdIsoPhi.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h (@NormalQuotMk.{u} G one mul inv N group_args n_normal (mul a b))) ",
            "(@NormalQuotMul.{u} G one mul inv Hpred group_args h_normal (@ThirdIsoPhi.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h (@NormalQuotMk.{u} G one mul inv N group_args n_normal a)) (@ThirdIsoPhi.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h (@NormalQuotMk.{u} G one mul inv N group_args n_normal b)))"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun Hpred => fun group_args => fun n_normal => fun h_normal => fun n_le_h => fun a => fun b => ",
            "@Eq.refl.{succ u} (@NormalQuot.{u} G one mul inv Hpred group_args h_normal) (@NormalQuotMk.{u} G one mul inv Hpred group_args h_normal (mul a b))"
        ),
    },
    TheoremArtifact {
        name: "third_iso_phi_one",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (h_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv Hpred), ",
            "forall (n_le_h : forall (x : G), forall (hn : N x), Hpred x), ",
            "@Eq.{succ u} (@NormalQuot.{u} G one mul inv Hpred group_args h_normal) ",
            "(@ThirdIsoPhi.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h (@NormalQuotOne.{u} G one mul inv N group_args n_normal)) ",
            "(@NormalQuotOne.{u} G one mul inv Hpred group_args h_normal)"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun Hpred => fun group_args => fun n_normal => fun h_normal => fun n_le_h => ",
            "@Eq.refl.{succ u} (@NormalQuot.{u} G one mul inv Hpred group_args h_normal) (@NormalQuotOne.{u} G one mul inv Hpred group_args h_normal)"
        ),
    },
    TheoremArtifact {
        name: "third_iso_phi_inv",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (h_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv Hpred), ",
            "forall (n_le_h : forall (x : G), forall (hn : N x), Hpred x), ",
            "forall (a : G), ",
            "@Eq.{succ u} (@NormalQuot.{u} G one mul inv Hpred group_args h_normal) ",
            "(@ThirdIsoPhi.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h (@NormalQuotMk.{u} G one mul inv N group_args n_normal (inv a))) ",
            "(@NormalQuotInv.{u} G one mul inv Hpred group_args h_normal (@ThirdIsoPhi.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h (@NormalQuotMk.{u} G one mul inv N group_args n_normal a)))"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun Hpred => fun group_args => fun n_normal => fun h_normal => fun n_le_h => fun a => ",
            "@Eq.refl.{succ u} (@NormalQuot.{u} G one mul inv Hpred group_args h_normal) (@NormalQuotMk.{u} G one mul inv Hpred group_args h_normal (inv a))"
        ),
    },
    TheoremArtifact {
        name: "third_iso_phi_surjective",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (h_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv Hpred), ",
            "forall (n_le_h : forall (x : G), forall (hn : N x), Hpred x), ",
            "forall (q : @NormalQuot.{u} G one mul inv Hpred group_args h_normal), ",
            "forall (P : Prop), ",
            "forall (mk : forall (src : @NormalQuot.{u} G one mul inv N group_args n_normal), forall (h : @Eq.{succ u} (@NormalQuot.{u} G one mul inv Hpred group_args h_normal) (@ThirdIsoPhi.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h src) q), P), ",
            "P"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun Hpred => fun group_args => fun n_normal => fun h_normal => fun n_le_h => ",
            "@Quotient.indProp.{u} G (@NormalSetoid.{u} G one mul inv Hpred group_args h_normal) ",
            "(fun (q : @NormalQuot.{u} G one mul inv Hpred group_args h_normal) => forall (P : Prop), forall (mk : forall (src : @NormalQuot.{u} G one mul inv N group_args n_normal), forall (h : @Eq.{succ u} (@NormalQuot.{u} G one mul inv Hpred group_args h_normal) (@ThirdIsoPhi.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h src) q), P), P) ",
            "(fun (a : G) => fun (P : Prop) => fun (mk : forall (src : @NormalQuot.{u} G one mul inv N group_args n_normal), forall (h : @Eq.{succ u} (@NormalQuot.{u} G one mul inv Hpred group_args h_normal) (@ThirdIsoPhi.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h src) (@NormalQuotMk.{u} G one mul inv Hpred group_args h_normal a)), P) => ",
            "mk (@NormalQuotMk.{u} G one mul inv N group_args n_normal a) (@third_iso_phi_mk.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h a))"
        ),
    },
    TheoremArtifact {
        name: "third_iso_hn_to_kernel_sound",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (h_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv Hpred), ",
            "forall (n_le_h : forall (x : G), forall (hn : N x), Hpred x), ",
            "forall (q : @NormalQuot.{u} G one mul inv N group_args n_normal), ",
            "forall (hk : @ThirdIsoKernelPred.{u} G one mul inv N Hpred group_args n_normal q), ",
            "@Eq.{succ u} (@NormalQuot.{u} G one mul inv Hpred group_args h_normal) ",
            "(@ThirdIsoPhi.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h q) ",
            "(@NormalQuotOne.{u} G one mul inv Hpred group_args h_normal)"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun Hpred => fun group_args => fun n_normal => fun h_normal => fun n_le_h => fun q => fun hk => ",
            "hk (@Eq.{succ u} (@NormalQuot.{u} G one mul inv Hpred group_args h_normal) (@ThirdIsoPhi.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h q) (@NormalQuotOne.{u} G one mul inv Hpred group_args h_normal)) ",
            "(fun (h : G) => fun (hh : Hpred h) => fun (eq_q : @Eq.{succ u} (@NormalQuot.{u} G one mul inv N group_args n_normal) (@NormalQuotMk.{u} G one mul inv N group_args n_normal h) q) => ",
            "@eq_trans.{succ u} (@NormalQuot.{u} G one mul inv Hpred group_args h_normal) ",
            "(@ThirdIsoPhi.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h q) ",
            "(@ThirdIsoPhi.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h (@NormalQuotMk.{u} G one mul inv N group_args n_normal h)) ",
            "(@NormalQuotOne.{u} G one mul inv Hpred group_args h_normal) ",
            "(@eq_symm.{succ u} (@NormalQuot.{u} G one mul inv Hpred group_args h_normal) ",
            "(@ThirdIsoPhi.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h (@NormalQuotMk.{u} G one mul inv N group_args n_normal h)) ",
            "(@ThirdIsoPhi.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h q) ",
            "(@eq_congr_arg.{succ u,succ u} (@NormalQuot.{u} G one mul inv N group_args n_normal) (@NormalQuot.{u} G one mul inv Hpred group_args h_normal) ",
            "(@ThirdIsoPhi.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h) ",
            "(@NormalQuotMk.{u} G one mul inv N group_args n_normal h) q eq_q)) ",
            "(@eq_trans.{succ u} (@NormalQuot.{u} G one mul inv Hpred group_args h_normal) ",
            "(@ThirdIsoPhi.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h (@NormalQuotMk.{u} G one mul inv N group_args n_normal h)) ",
            "(@NormalQuotMk.{u} G one mul inv Hpred group_args h_normal h) ",
            "(@NormalQuotOne.{u} G one mul inv Hpred group_args h_normal) ",
            "(@third_iso_phi_mk.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h h) ",
            "(@normal_quot_sound.{u} G one mul inv Hpred group_args h_normal h one (@normal_rel_one_of_mem.{succ u} G one mul inv group_args Hpred h_normal h hh))))"
        ),
    },
    TheoremArtifact {
        name: "third_iso_kernel_intro",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (a : G), forall (ha : Hpred a), ",
            "@ThirdIsoKernelPred.{u} G one mul inv N Hpred group_args n_normal (@NormalQuotMk.{u} G one mul inv N group_args n_normal a)"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun Hpred => fun group_args => fun n_normal => fun a => fun ha => ",
            "@third_iso_hn_intro.{u} G one mul inv N Hpred group_args n_normal (@NormalQuotMk.{u} G one mul inv N group_args n_normal a) a ha (@Eq.refl.{succ u} (@NormalQuot.{u} G one mul inv N group_args n_normal) (@NormalQuotMk.{u} G one mul inv N group_args n_normal a))"
        ),
    },
    TheoremArtifact {
        name: "third_iso_kernel_evidence",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (h_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv Hpred), ",
            "forall (n_le_h : forall (x : G), forall (hn : N x), Hpred x), ",
            "@ThirdIsoKernelEvidence.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun Hpred => fun group_args => fun n_normal => fun h_normal => fun n_le_h => ",
            "fun (P : Prop) => ",
            "fun (mk : forall (kernel_sound : forall (q : @NormalQuot.{u} G one mul inv N group_args n_normal), forall (hk : @ThirdIsoKernelPred.{u} G one mul inv N Hpred group_args n_normal q), @Eq.{succ u} (@NormalQuot.{u} G one mul inv Hpred group_args h_normal) (@ThirdIsoPhi.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h q) (@NormalQuotOne.{u} G one mul inv Hpred group_args h_normal)), forall (kernel_intro : forall (a : G), forall (ha : Hpred a), @ThirdIsoKernelPred.{u} G one mul inv N Hpred group_args n_normal (@NormalQuotMk.{u} G one mul inv N group_args n_normal a)), P) => ",
            "mk ",
            "(@third_iso_hn_to_kernel_sound.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h) ",
            "(@third_iso_kernel_intro.{u} G one mul inv N Hpred group_args n_normal)"
        ),
    },
    TheoremArtifact {
        name: "third_isomorphism_theorem_evidence",
        universe_params: &["u"],
        statement: concat!(
            "forall (G : Sort succ u), ",
            "forall (one : G), ",
            "forall (mul : forall (a : G), forall (b : G), G), ",
            "forall (inv : forall (a : G), G), ",
            "forall (N : forall (x : G), Prop), ",
            "forall (Hpred : forall (x : G), Prop), ",
            "forall (group_args : @GroupLawArgs.{succ u} G one mul inv), ",
            "forall (n_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv N), ",
            "forall (h_normal : @NormalSubgroupLawArgs.{succ u} G one mul inv Hpred), ",
            "forall (n_le_h : forall (x : G), forall (hn : N x), Hpred x), ",
            "@ThirdIsoTheoremEvidence.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h"
        ),
        proof: concat!(
            "fun G => fun one => fun mul => fun inv => fun N => fun Hpred => fun group_args => fun n_normal => fun h_normal => fun n_le_h => ",
            "fun (P : Prop) => ",
            "fun (mk : forall (rel_lift : forall (a : G), forall (b : G), forall (h : @NormalRel.{succ u} G one mul inv N a b), @NormalRel.{succ u} G one mul inv Hpred a b), forall (surjective : forall (q : @NormalQuot.{u} G one mul inv Hpred group_args h_normal), forall (Q : Prop), forall (mk_surj : forall (src : @NormalQuot.{u} G one mul inv N group_args n_normal), forall (h : @Eq.{succ u} (@NormalQuot.{u} G one mul inv Hpred group_args h_normal) (@ThirdIsoPhi.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h src) q), Q), Q), forall (kernel_evidence : @ThirdIsoKernelEvidence.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h), P) => ",
            "mk ",
            "(@third_iso_rel_lift.{u} G one mul inv N Hpred n_le_h) ",
            "(@third_iso_phi_surjective.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h) ",
            "(@third_iso_kernel_evidence.{u} G one mul inv N Hpred group_args n_normal h_normal n_le_h)"
        ),
    },
];

const ABSTRACT_RING_DEFINITIONS: &[DefinitionArtifact] = &[
    DefinitionArtifact {
        name: "two",
        universe_params: &["u"],
        ty: concat!(
            "forall (Scalar : Sort u), ",
            "forall (one : Scalar), ",
            "forall (add : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            "Scalar"
        ),
        value: "fun Scalar => fun one => fun add => add one one",
    },
    DefinitionArtifact {
        name: "sq",
        universe_params: &["u"],
        ty: concat!(
            "forall (Scalar : Sort u), ",
            "forall (mul : forall (a : Scalar), forall (b : Scalar), Scalar), ",
            "forall (a : Scalar), ",
            "Scalar"
        ),
        value: "fun Scalar => fun mul => fun a => mul a a",
    },
    DefinitionArtifact {
        name: "RingLawArgs",
        universe_params: &["u"],
        ty: abstract_ring_params!("Prop"),
        value: abstract_ring_abs!(concat!(
            "forall (P : Prop), forall (mk : ",
            "forall (sub_eq_add_neg_law : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub a b) (add a (neg b))), ",
            "forall (add_assoc_law : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add a b) c) (add a (add b c))), ",
            "forall (add_comm_law : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add a b) (add b a)), ",
            "forall (add_zero_law : forall (a : Scalar), @Eq.{u} Scalar (add a zero) a), ",
            "forall (zero_add_law : forall (a : Scalar), @Eq.{u} Scalar (add zero a) a), ",
            "forall (neg_add_cancel_law : forall (a : Scalar), @Eq.{u} Scalar (add (neg a) a) zero), ",
            "forall (add_neg_cancel_law : forall (a : Scalar), @Eq.{u} Scalar (add a (neg a)) zero), ",
            "forall (sub_self_law : forall (a : Scalar), @Eq.{u} Scalar (sub a a) zero), ",
            "forall (mul_assoc_law : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (mul a b) c) (mul a (mul b c))), ",
            "forall (mul_comm_law : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (mul a b) (mul b a)), ",
            "forall (mul_one_law : forall (a : Scalar), @Eq.{u} Scalar (mul a one) a), ",
            "forall (one_mul_law : forall (a : Scalar), @Eq.{u} Scalar (mul one a) a), ",
            "forall (left_distrib_law : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul a (add b c)) (add (mul a b) (mul a c))), ",
            "forall (right_distrib_law : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (add a b) c) (add (mul a c) (mul b c))), ",
            "forall (mul_zero_law : forall (a : Scalar), @Eq.{u} Scalar (mul a zero) zero), ",
            "forall (zero_mul_law : forall (a : Scalar), @Eq.{u} Scalar (mul zero a) zero), ",
            "forall (add_left_cancel_law : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add a b) (add a c)), @Eq.{u} Scalar b c), ",
            "forall (ring_normalize_add_mul3_law : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add (mul a b) (mul b c)) (mul a c)) (add (add (mul a b) (mul a c)) (mul b c))), ",
            "forall (add_right_cancel_law : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add b a) (add c a)), @Eq.{u} Scalar b c), ",
            "forall (neg_neg_law : forall (a : Scalar), @Eq.{u} Scalar (neg (neg a)) a), ",
            "forall (sub_zero_law : forall (a : Scalar), @Eq.{u} Scalar (sub a zero) a), ",
            "forall (zero_sub_law : forall (a : Scalar), @Eq.{u} Scalar (sub zero a) (neg a)), ",
            "forall (sub_add_cancel_law : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add (sub a b) b) a), ",
            "forall (add_sub_cancel_law : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub (add a b) b) a), ",
            "forall (sub_add_sub_cancel_law : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (sub (sub a c) (sub b c)) (sub a b)), P), P"
        )),
    },
];

const ABSTRACT_RING_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "sub_eq_add_neg",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub a b) (add a (neg b))), forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub a b) (add a (neg b))"
        ),
        proof: abstract_ring_abs!("fun law => fun a => fun b => law a b"),
    },
    TheoremArtifact {
        name: "add_assoc",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add a b) c) (add a (add b c))), forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add a b) c) (add a (add b c))"
        ),
        proof: abstract_ring_abs!("fun law => fun a => fun b => fun c => law a b c"),
    },
    TheoremArtifact {
        name: "add_comm",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add a b) (add b a)), forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add a b) (add b a)"
        ),
        proof: abstract_ring_abs!("fun law => fun a => fun b => law a b"),
    },
    TheoremArtifact {
        name: "add_zero",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (law : forall (a : Scalar), @Eq.{u} Scalar (add a zero) a), forall (a : Scalar), @Eq.{u} Scalar (add a zero) a"
        ),
        proof: abstract_ring_abs!("fun law => fun a => law a"),
    },
    TheoremArtifact {
        name: "zero_add",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (law : forall (a : Scalar), @Eq.{u} Scalar (add zero a) a), forall (a : Scalar), @Eq.{u} Scalar (add zero a) a"
        ),
        proof: abstract_ring_abs!("fun law => fun a => law a"),
    },
    TheoremArtifact {
        name: "neg_add_cancel",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (law : forall (a : Scalar), @Eq.{u} Scalar (add (neg a) a) zero), forall (a : Scalar), @Eq.{u} Scalar (add (neg a) a) zero"
        ),
        proof: abstract_ring_abs!("fun law => fun a => law a"),
    },
    TheoremArtifact {
        name: "add_neg_cancel",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (law : forall (a : Scalar), @Eq.{u} Scalar (add a (neg a)) zero), forall (a : Scalar), @Eq.{u} Scalar (add a (neg a)) zero"
        ),
        proof: abstract_ring_abs!("fun law => fun a => law a"),
    },
    TheoremArtifact {
        name: "sub_self",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (law : forall (a : Scalar), @Eq.{u} Scalar (sub a a) zero), forall (a : Scalar), @Eq.{u} Scalar (sub a a) zero"
        ),
        proof: abstract_ring_abs!("fun law => fun a => law a"),
    },
    TheoremArtifact {
        name: "mul_assoc",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (mul a b) c) (mul a (mul b c))), forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (mul a b) c) (mul a (mul b c))"
        ),
        proof: abstract_ring_abs!("fun law => fun a => fun b => fun c => law a b c"),
    },
    TheoremArtifact {
        name: "mul_comm",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (mul a b) (mul b a)), forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (mul a b) (mul b a)"
        ),
        proof: abstract_ring_abs!("fun law => fun a => fun b => law a b"),
    },
    TheoremArtifact {
        name: "mul_one",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (law : forall (a : Scalar), @Eq.{u} Scalar (mul a one) a), forall (a : Scalar), @Eq.{u} Scalar (mul a one) a"
        ),
        proof: abstract_ring_abs!("fun law => fun a => law a"),
    },
    TheoremArtifact {
        name: "one_mul",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (law : forall (a : Scalar), @Eq.{u} Scalar (mul one a) a), forall (a : Scalar), @Eq.{u} Scalar (mul one a) a"
        ),
        proof: abstract_ring_abs!("fun law => fun a => law a"),
    },
    TheoremArtifact {
        name: "left_distrib",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul a (add b c)) (add (mul a b) (mul a c))), forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul a (add b c)) (add (mul a b) (mul a c))"
        ),
        proof: abstract_ring_abs!("fun law => fun a => fun b => fun c => law a b c"),
    },
    TheoremArtifact {
        name: "right_distrib",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (add a b) c) (add (mul a c) (mul b c))), forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (add a b) c) (add (mul a c) (mul b c))"
        ),
        proof: abstract_ring_abs!("fun law => fun a => fun b => fun c => law a b c"),
    },
    TheoremArtifact {
        name: "mul_zero",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (law : forall (a : Scalar), @Eq.{u} Scalar (mul a zero) zero), forall (a : Scalar), @Eq.{u} Scalar (mul a zero) zero"
        ),
        proof: abstract_ring_abs!("fun law => fun a => law a"),
    },
    TheoremArtifact {
        name: "zero_mul",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (law : forall (a : Scalar), @Eq.{u} Scalar (mul zero a) zero), forall (a : Scalar), @Eq.{u} Scalar (mul zero a) zero"
        ),
        proof: abstract_ring_abs!("fun law => fun a => law a"),
    },
    TheoremArtifact {
        name: "add_left_cancel",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add a b) (add a c)), @Eq.{u} Scalar b c), forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add a b) (add a c)), @Eq.{u} Scalar b c"
        ),
        proof: abstract_ring_abs!("fun law => fun a => fun b => fun c => fun h => law a b c h"),
    },
    TheoremArtifact {
        name: "ring_normalize_add_mul3",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add (mul a b) (mul b c)) (mul a c)) (add (add (mul a b) (mul a c)) (mul b c))), forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add (mul a b) (mul b c)) (mul a c)) (add (add (mul a b) (mul a c)) (mul b c))"
        ),
        proof: abstract_ring_abs!("fun law => fun a => fun b => fun c => law a b c"),
    },
    TheoremArtifact {
        name: "add_right_cancel",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add b a) (add c a)), @Eq.{u} Scalar b c), forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add b a) (add c a)), @Eq.{u} Scalar b c"
        ),
        proof: abstract_ring_abs!("fun law => fun a => fun b => fun c => fun h => law a b c h"),
    },
    TheoremArtifact {
        name: "neg_neg",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (law : forall (a : Scalar), @Eq.{u} Scalar (neg (neg a)) a), forall (a : Scalar), @Eq.{u} Scalar (neg (neg a)) a"
        ),
        proof: abstract_ring_abs!("fun law => fun a => law a"),
    },
    TheoremArtifact {
        name: "sub_zero",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (law : forall (a : Scalar), @Eq.{u} Scalar (sub a zero) a), forall (a : Scalar), @Eq.{u} Scalar (sub a zero) a"
        ),
        proof: abstract_ring_abs!("fun law => fun a => law a"),
    },
    TheoremArtifact {
        name: "zero_sub",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (law : forall (a : Scalar), @Eq.{u} Scalar (sub zero a) (neg a)), forall (a : Scalar), @Eq.{u} Scalar (sub zero a) (neg a)"
        ),
        proof: abstract_ring_abs!("fun law => fun a => law a"),
    },
    TheoremArtifact {
        name: "sub_add_cancel",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add (sub a b) b) a), forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add (sub a b) b) a"
        ),
        proof: abstract_ring_abs!("fun law => fun a => fun b => law a b"),
    },
    TheoremArtifact {
        name: "add_sub_cancel",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub (add a b) b) a), forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub (add a b) b) a"
        ),
        proof: abstract_ring_abs!("fun law => fun a => fun b => law a b"),
    },
    TheoremArtifact {
        name: "sub_add_sub_cancel",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (sub (sub a c) (sub b c)) (sub a b)), forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (sub (sub a c) (sub b c)) (sub a b)"
        ),
        proof: abstract_ring_abs!("fun law => fun a => fun b => fun c => law a b c"),
    },
];

const ABSTRACT_ORDERED_FIELD_DEFINITIONS: &[DefinitionArtifact] = &[
    DefinitionArtifact {
        name: "le",
        universe_params: &["u"],
        ty: concat!(
            "forall (Scalar : Sort u), ",
            "forall (le_rel : forall (a : Scalar), forall (b : Scalar), Prop), ",
            "forall (a : Scalar), forall (b : Scalar), Prop"
        ),
        value: "fun Scalar => fun le_rel => fun a => fun b => le_rel a b",
    },
    DefinitionArtifact {
        name: "lt",
        universe_params: &["u"],
        ty: concat!(
            "forall (Scalar : Sort u), ",
            "forall (lt_rel : forall (a : Scalar), forall (b : Scalar), Prop), ",
            "forall (a : Scalar), forall (b : Scalar), Prop"
        ),
        value: "fun Scalar => fun lt_rel => fun a => fun b => lt_rel a b",
    },
    DefinitionArtifact {
        name: "sqrt",
        universe_params: &["u"],
        ty: concat!(
            "forall (Scalar : Sort u), ",
            "forall (sqrt_fn : forall (a : Scalar), Scalar), ",
            "forall (a : Scalar), Scalar"
        ),
        value: "fun Scalar => fun sqrt_fn => fun a => sqrt_fn a",
    },
    DefinitionArtifact {
        name: "Nonneg",
        universe_params: &["u"],
        ty: concat!(
            "forall (Scalar : Sort u), ",
            "forall (zero : Scalar), ",
            "forall (le_rel : forall (a : Scalar), forall (b : Scalar), Prop), ",
            "forall (a : Scalar), Prop"
        ),
        value: "fun Scalar => fun zero => fun le_rel => fun a => le_rel zero a",
    },
    DefinitionArtifact {
        name: "Positive",
        universe_params: &["u"],
        ty: concat!(
            "forall (Scalar : Sort u), ",
            "forall (zero : Scalar), ",
            "forall (lt_rel : forall (a : Scalar), forall (b : Scalar), Prop), ",
            "forall (a : Scalar), Prop"
        ),
        value: "fun Scalar => fun zero => fun lt_rel => fun a => lt_rel zero a",
    },
    DefinitionArtifact {
        name: "OrderedFieldLawArgs",
        universe_params: &["u"],
        ty: abstract_ordered_field_params!("Prop"),
        value: abstract_ordered_field_abs!(concat!(
            "forall (P : Prop), forall (mk : ",
            "forall (le_refl_law : forall (a : Scalar), le_rel a a), ",
            "forall (le_trans_law : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (hab : le_rel a b), forall (hbc : le_rel b c), le_rel a c), ",
            "forall (add_nonneg_law : forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), le_rel zero (add a b)), ",
            "forall (mul_nonneg_law : forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), le_rel zero (mul a b)), ",
            "forall (square_nonneg_law : forall (a : Scalar), le_rel zero (@sq.{u} Scalar mul a)), ",
            "forall (sqrt_nonneg_law : forall (a : Scalar), le_rel zero (sqrt_fn a)), ",
            "forall (sqrt_square_of_nonneg_law : forall (a : Scalar), forall (ha : le_rel zero a), @Eq.{u} Scalar (sqrt_fn (@sq.{u} Scalar mul a)) a), ",
            "forall (sqrt_mul_self_law : forall (a : Scalar), forall (ha : le_rel zero a), @Eq.{u} Scalar (sqrt_fn (mul a a)) a), ",
            "forall (eq_of_square_eq_square_nonneg_law : forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hsq : @Eq.{u} Scalar (@sq.{u} Scalar mul a) (@sq.{u} Scalar mul b)), @Eq.{u} Scalar a b), ",
            "forall (add_le_add_law : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (d : Scalar), forall (hab : le_rel a b), forall (hcd : le_rel c d), le_rel (add a c) (add b d)), ",
            "forall (mul_le_mul_nonneg_law : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (d : Scalar), forall (ha : le_rel zero a), forall (hab : le_rel a b), forall (hc : le_rel zero c), forall (hcd : le_rel c d), le_rel (mul a c) (mul b d)), ",
            "forall (zero_le_two_law : le_rel zero (@two.{u} Scalar one add)), ",
            "forall (le_antisymm_law : forall (a : Scalar), forall (b : Scalar), forall (hab : le_rel a b), forall (hba : le_rel b a), @Eq.{u} Scalar a b), ",
            "forall (lt_of_le_of_ne_law : forall (a : Scalar), forall (ha : le_rel zero a), forall (hne : forall (haz : @Eq.{u} Scalar a zero), forall (P : Prop), P), lt_rel zero a), ",
            "forall (le_of_eq_law : forall (a : Scalar), forall (b : Scalar), forall (hab : @Eq.{u} Scalar a b), forall (P : Prop), forall (mk : forall (hab_le : le_rel a b), forall (hba_le : le_rel b a), P), P), ",
            "forall (sqrt_sq_law : forall (a : Scalar), forall (ha : le_rel zero a), @Eq.{u} Scalar (@sq.{u} Scalar mul (sqrt_fn a)) a), ",
            "forall (sq_eq_zero_iff_law : forall (a : Scalar), forall (R : Prop), forall (mk : forall (forward : forall (hsqz : @Eq.{u} Scalar (@sq.{u} Scalar mul a) zero), @Eq.{u} Scalar a zero), forall (backward : forall (haz : @Eq.{u} Scalar a zero), @Eq.{u} Scalar (@sq.{u} Scalar mul a) zero), R), R), ",
            "forall (sum_nonneg_eq_zero_law : forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hsum : @Eq.{u} Scalar (add a b) zero), forall (R : Prop), forall (mk : forall (haz : @Eq.{u} Scalar a zero), forall (hbz : @Eq.{u} Scalar b zero), R), R), ",
            "forall (square_completion_bound_law : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (hquadratic : forall (t : Scalar), le_rel zero (add (add (mul a (@sq.{u} Scalar mul t)) (mul (mul (@two.{u} Scalar one add) b) t)) c)), le_rel (@sq.{u} Scalar mul b) (mul a c)), ",
            "forall (le_of_sq_le_sq_nonneg_law : forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hsq : le_rel (@sq.{u} Scalar mul a) (@sq.{u} Scalar mul b)), le_rel a b), ",
            "forall (le_mul_sqrt_of_sq_le_mul_nonneg_law : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hsq : le_rel (@sq.{u} Scalar mul c) (mul a b)), le_rel c (mul (sqrt_fn a) (sqrt_fn b))), ",
            "forall (add_two_mul_le_sq_add_sqrt_law : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (n : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hn : @Eq.{u} Scalar n (add (add a (mul (@two.{u} Scalar one add) c)) b)), forall (hc : le_rel c (mul (sqrt_fn a) (sqrt_fn b))), le_rel n (@sq.{u} Scalar mul (add (sqrt_fn a) (sqrt_fn b)))), P), P"
        )),
    },
];

const ABSTRACT_ORDERED_FIELD_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "le_refl",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), le_rel a a), forall (a : Scalar), le_rel a a"
        ),
        proof: abstract_ordered_field_abs!("fun law => fun a => law a"),
    },
    TheoremArtifact {
        name: "le_trans",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (hab : le_rel a b), forall (hbc : le_rel b c), le_rel a c), forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (hab : le_rel a b), forall (hbc : le_rel b c), le_rel a c"
        ),
        proof: abstract_ordered_field_abs!(
            "fun law => fun a => fun b => fun c => fun hab => fun hbc => law a b c hab hbc"
        ),
    },
    TheoremArtifact {
        name: "add_nonneg",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), le_rel zero (add a b)), forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), le_rel zero (add a b)"
        ),
        proof: abstract_ordered_field_abs!(
            "fun law => fun a => fun b => fun ha => fun hb => law a b ha hb"
        ),
    },
    TheoremArtifact {
        name: "mul_nonneg",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), le_rel zero (mul a b)), forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), le_rel zero (mul a b)"
        ),
        proof: abstract_ordered_field_abs!(
            "fun law => fun a => fun b => fun ha => fun hb => law a b ha hb"
        ),
    },
    TheoremArtifact {
        name: "square_nonneg",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), le_rel zero (@sq.{u} Scalar mul a)), forall (a : Scalar), le_rel zero (@sq.{u} Scalar mul a)"
        ),
        proof: abstract_ordered_field_abs!("fun law => fun a => law a"),
    },
    TheoremArtifact {
        name: "sqrt_nonneg",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), le_rel zero (sqrt_fn a)), forall (a : Scalar), le_rel zero (sqrt_fn a)"
        ),
        proof: abstract_ordered_field_abs!("fun law => fun a => law a"),
    },
    TheoremArtifact {
        name: "sqrt_square_of_nonneg",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), forall (ha : le_rel zero a), @Eq.{u} Scalar (sqrt_fn (@sq.{u} Scalar mul a)) a), forall (a : Scalar), forall (ha : le_rel zero a), @Eq.{u} Scalar (sqrt_fn (@sq.{u} Scalar mul a)) a"
        ),
        proof: abstract_ordered_field_abs!("fun law => fun a => fun ha => law a ha"),
    },
    TheoremArtifact {
        name: "sqrt_mul_self",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), forall (ha : le_rel zero a), @Eq.{u} Scalar (sqrt_fn (mul a a)) a), forall (a : Scalar), forall (ha : le_rel zero a), @Eq.{u} Scalar (sqrt_fn (mul a a)) a"
        ),
        proof: abstract_ordered_field_abs!("fun law => fun a => fun ha => law a ha"),
    },
    TheoremArtifact {
        name: "eq_of_square_eq_square_nonneg",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hsq : @Eq.{u} Scalar (@sq.{u} Scalar mul a) (@sq.{u} Scalar mul b)), @Eq.{u} Scalar a b), forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hsq : @Eq.{u} Scalar (@sq.{u} Scalar mul a) (@sq.{u} Scalar mul b)), @Eq.{u} Scalar a b"
        ),
        proof: abstract_ordered_field_abs!(
            "fun law => fun a => fun b => fun ha => fun hb => fun hsq => law a b ha hb hsq"
        ),
    },
    TheoremArtifact {
        name: "add_le_add",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (d : Scalar), forall (hab : le_rel a b), forall (hcd : le_rel c d), le_rel (add a c) (add b d)), forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (d : Scalar), forall (hab : le_rel a b), forall (hcd : le_rel c d), le_rel (add a c) (add b d)"
        ),
        proof: abstract_ordered_field_abs!(
            "fun law => fun a => fun b => fun c => fun d => fun hab => fun hcd => law a b c d hab hcd"
        ),
    },
    TheoremArtifact {
        name: "mul_le_mul_nonneg",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (d : Scalar), forall (ha : le_rel zero a), forall (hab : le_rel a b), forall (hc : le_rel zero c), forall (hcd : le_rel c d), le_rel (mul a c) (mul b d)), forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (d : Scalar), forall (ha : le_rel zero a), forall (hab : le_rel a b), forall (hc : le_rel zero c), forall (hcd : le_rel c d), le_rel (mul a c) (mul b d)"
        ),
        proof: abstract_ordered_field_abs!(
            "fun law => fun a => fun b => fun c => fun d => fun ha => fun hab => fun hc => fun hcd => law a b c d ha hab hc hcd"
        ),
    },
    TheoremArtifact {
        name: "zero_le_two",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : le_rel zero (@two.{u} Scalar one add)), le_rel zero (@two.{u} Scalar one add)"
        ),
        proof: abstract_ordered_field_abs!("fun law => law"),
    },
    TheoremArtifact {
        name: "le_antisymm",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), forall (hab : le_rel a b), forall (hba : le_rel b a), @Eq.{u} Scalar a b), forall (a : Scalar), forall (b : Scalar), forall (hab : le_rel a b), forall (hba : le_rel b a), @Eq.{u} Scalar a b"
        ),
        proof: abstract_ordered_field_abs!(
            "fun law => fun a => fun b => fun hab => fun hba => law a b hab hba"
        ),
    },
    TheoremArtifact {
        name: "lt_of_le_of_ne",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), forall (ha : le_rel zero a), forall (hne : forall (haz : @Eq.{u} Scalar a zero), forall (P : Prop), P), lt_rel zero a), forall (a : Scalar), forall (ha : le_rel zero a), forall (hne : forall (haz : @Eq.{u} Scalar a zero), forall (P : Prop), P), lt_rel zero a"
        ),
        proof: abstract_ordered_field_abs!(
            "fun law => fun a => fun ha => fun hne => law a ha hne"
        ),
    },
    TheoremArtifact {
        name: "le_of_eq",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), forall (hab : @Eq.{u} Scalar a b), forall (P : Prop), forall (mk : forall (hab_le : le_rel a b), forall (hba_le : le_rel b a), P), P), forall (a : Scalar), forall (b : Scalar), forall (hab : @Eq.{u} Scalar a b), forall (P : Prop), forall (mk : forall (hab_le : le_rel a b), forall (hba_le : le_rel b a), P), P"
        ),
        proof: abstract_ordered_field_abs!(
            "fun law => fun a => fun b => fun hab => fun P => fun mk => law a b hab P mk"
        ),
    },
    TheoremArtifact {
        name: "sqrt_sq",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), forall (ha : le_rel zero a), @Eq.{u} Scalar (@sq.{u} Scalar mul (sqrt_fn a)) a), forall (a : Scalar), forall (ha : le_rel zero a), @Eq.{u} Scalar (@sq.{u} Scalar mul (sqrt_fn a)) a"
        ),
        proof: abstract_ordered_field_abs!("fun law => fun a => fun ha => law a ha"),
    },
    TheoremArtifact {
        name: "sq_eq_zero_iff",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), forall (R : Prop), forall (mk : forall (forward : forall (hsqz : @Eq.{u} Scalar (@sq.{u} Scalar mul a) zero), @Eq.{u} Scalar a zero), forall (backward : forall (haz : @Eq.{u} Scalar a zero), @Eq.{u} Scalar (@sq.{u} Scalar mul a) zero), R), R), forall (a : Scalar), forall (R : Prop), forall (mk : forall (forward : forall (hsqz : @Eq.{u} Scalar (@sq.{u} Scalar mul a) zero), @Eq.{u} Scalar a zero), forall (backward : forall (haz : @Eq.{u} Scalar a zero), @Eq.{u} Scalar (@sq.{u} Scalar mul a) zero), R), R"
        ),
        proof: abstract_ordered_field_abs!(
            "fun law => fun a => fun R => fun mk => law a R mk"
        ),
    },
    TheoremArtifact {
        name: "sum_nonneg_eq_zero",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hsum : @Eq.{u} Scalar (add a b) zero), forall (R : Prop), forall (mk : forall (haz : @Eq.{u} Scalar a zero), forall (hbz : @Eq.{u} Scalar b zero), R), R), forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hsum : @Eq.{u} Scalar (add a b) zero), forall (R : Prop), forall (mk : forall (haz : @Eq.{u} Scalar a zero), forall (hbz : @Eq.{u} Scalar b zero), R), R"
        ),
        proof: abstract_ordered_field_abs!(
            "fun law => fun a => fun b => fun ha => fun hb => fun hsum => fun R => fun mk => law a b ha hb hsum R mk"
        ),
    },
    TheoremArtifact {
        name: "square_completion_bound_from_ordered_args",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (ordered_args : @OrderedFieldLawArgs.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn), forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (hquadratic : forall (t : Scalar), le_rel zero (add (add (mul a (@sq.{u} Scalar mul t)) (mul (mul (@two.{u} Scalar one add) b) t)) c)), le_rel (@sq.{u} Scalar mul b) (mul a c)"
        ),
        proof: abstract_ordered_field_abs!(concat!(
            "fun ordered_args => fun a => fun b => fun c => fun hquadratic => ",
            "ordered_args (le_rel (@sq.{u} Scalar mul b) (mul a c)) ",
            "(fun (le_refl_arg : forall (a : Scalar), le_rel a a) => ",
            "fun (le_trans_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (hab : le_rel a b), forall (hbc : le_rel b c), le_rel a c) => ",
            "fun (add_nonneg_arg : forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), le_rel zero (add a b)) => ",
            "fun (mul_nonneg_arg : forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), le_rel zero (mul a b)) => ",
            "fun (square_nonneg_arg : forall (a : Scalar), le_rel zero (@sq.{u} Scalar mul a)) => ",
            "fun (sqrt_nonneg_arg : forall (a : Scalar), le_rel zero (sqrt_fn a)) => ",
            "fun (sqrt_square_of_nonneg_arg : forall (a : Scalar), forall (ha : le_rel zero a), @Eq.{u} Scalar (sqrt_fn (@sq.{u} Scalar mul a)) a) => ",
            "fun (sqrt_mul_self_arg : forall (a : Scalar), forall (ha : le_rel zero a), @Eq.{u} Scalar (sqrt_fn (mul a a)) a) => ",
            "fun (eq_of_square_eq_square_nonneg_arg : forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hsq : @Eq.{u} Scalar (@sq.{u} Scalar mul a) (@sq.{u} Scalar mul b)), @Eq.{u} Scalar a b) => ",
            "fun (add_le_add_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (d : Scalar), forall (hab : le_rel a b), forall (hcd : le_rel c d), le_rel (add a c) (add b d)) => ",
            "fun (mul_le_mul_nonneg_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (d : Scalar), forall (ha : le_rel zero a), forall (hab : le_rel a b), forall (hc : le_rel zero c), forall (hcd : le_rel c d), le_rel (mul a c) (mul b d)) => ",
            "fun (zero_le_two_arg : le_rel zero (@two.{u} Scalar one add)) => ",
            "fun (le_antisymm_arg : forall (a : Scalar), forall (b : Scalar), forall (hab : le_rel a b), forall (hba : le_rel b a), @Eq.{u} Scalar a b) => ",
            "fun (lt_of_le_of_ne_arg : forall (a : Scalar), forall (ha : le_rel zero a), forall (hne : forall (haz : @Eq.{u} Scalar a zero), forall (P : Prop), P), lt_rel zero a) => ",
            "fun (le_of_eq_arg : forall (a : Scalar), forall (b : Scalar), forall (hab : @Eq.{u} Scalar a b), forall (P : Prop), forall (mk : forall (hab_le : le_rel a b), forall (hba_le : le_rel b a), P), P) => ",
            "fun (sqrt_sq_arg : forall (a : Scalar), forall (ha : le_rel zero a), @Eq.{u} Scalar (@sq.{u} Scalar mul (sqrt_fn a)) a) => ",
            "fun (sq_eq_zero_iff_arg : forall (a : Scalar), forall (R : Prop), forall (mk : forall (forward : forall (hsqz : @Eq.{u} Scalar (@sq.{u} Scalar mul a) zero), @Eq.{u} Scalar a zero), forall (backward : forall (haz : @Eq.{u} Scalar a zero), @Eq.{u} Scalar (@sq.{u} Scalar mul a) zero), R), R) => ",
            "fun (sum_nonneg_eq_zero_arg : forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hsum : @Eq.{u} Scalar (add a b) zero), forall (R : Prop), forall (mk : forall (haz : @Eq.{u} Scalar a zero), forall (hbz : @Eq.{u} Scalar b zero), R), R) => ",
            "fun (square_completion_bound_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (hquadratic : forall (t : Scalar), le_rel zero (add (add (mul a (@sq.{u} Scalar mul t)) (mul (mul (@two.{u} Scalar one add) b) t)) c)), le_rel (@sq.{u} Scalar mul b) (mul a c)) => ",
            "fun (le_of_sq_le_sq_nonneg_arg : forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hsq : le_rel (@sq.{u} Scalar mul a) (@sq.{u} Scalar mul b)), le_rel a b) => ",
            "fun (le_mul_sqrt_of_sq_le_mul_nonneg_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hsq : le_rel (@sq.{u} Scalar mul c) (mul a b)), le_rel c (mul (sqrt_fn a) (sqrt_fn b))) => ",
            "fun (add_two_mul_le_sq_add_sqrt_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (n : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hn : @Eq.{u} Scalar n (add (add a (mul (@two.{u} Scalar one add) c)) b)), forall (hc : le_rel c (mul (sqrt_fn a) (sqrt_fn b))), le_rel n (@sq.{u} Scalar mul (add (sqrt_fn a) (sqrt_fn b)))) => ",
            "square_completion_bound_arg a b c hquadratic)"
        )),
    },
    TheoremArtifact {
        name: "le_of_sq_le_sq_nonneg_from_ordered_args",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (ordered_args : @OrderedFieldLawArgs.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn), forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hsq : le_rel (@sq.{u} Scalar mul a) (@sq.{u} Scalar mul b)), le_rel a b"
        ),
        proof: abstract_ordered_field_abs!(concat!(
            "fun ordered_args => fun a => fun b => fun ha => fun hb => fun hsq => ",
            ordered_args_elim!(
                "(le_rel a b)",
                "le_of_sq_le_sq_nonneg_arg a b ha hb hsq"
            )
        )),
    },
    TheoremArtifact {
        name: "add_dist_nonneg_from_ordered_args",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (ordered_args : @OrderedFieldLawArgs.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn), forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), le_rel zero (add a b)"
        ),
        proof: abstract_ordered_field_abs!(concat!(
            "fun ordered_args => fun a => fun b => fun ha => fun hb => ",
            ordered_args_elim!(
                "(le_rel zero (add a b))",
                "add_nonneg_arg a b ha hb"
            )
        )),
    },
    TheoremArtifact {
        name: "sqrt_sum_square_bound_from_ordered_args",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (ordered_args : @OrderedFieldLawArgs.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn), forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hc : le_rel zero c), forall (hsq : le_rel (@sq.{u} Scalar mul a) (@sq.{u} Scalar mul (add b c))), le_rel a (add b c)"
        ),
        proof: abstract_ordered_field_abs!(concat!(
            "fun ordered_args => fun a => fun b => fun c => fun ha => fun hb => fun hc => fun hsq => ",
            "@le_of_sq_le_sq_nonneg_from_ordered_args.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn ordered_args ",
            "a (add b c) ha ",
            "(@add_dist_nonneg_from_ordered_args.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn ordered_args b c hb hc) ",
            "hsq"
        )),
    },
    TheoremArtifact {
        name: "le_mul_sqrt_of_sq_le_mul_nonneg_from_ordered_args",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (ordered_args : @OrderedFieldLawArgs.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn), forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hsq : le_rel (@sq.{u} Scalar mul c) (mul a b)), le_rel c (mul (sqrt_fn a) (sqrt_fn b))"
        ),
        proof: abstract_ordered_field_abs!(concat!(
            "fun ordered_args => fun a => fun b => fun c => fun ha => fun hb => fun hsq => ",
            ordered_args_elim!(
                "(le_rel c (mul (sqrt_fn a) (sqrt_fn b)))",
                "le_mul_sqrt_of_sq_le_mul_nonneg_arg a b c ha hb hsq"
            )
        )),
    },
    TheoremArtifact {
        name: "add_two_mul_le_sq_add_sqrt_from_ordered_args",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (ordered_args : @OrderedFieldLawArgs.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn), forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (n : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hn : @Eq.{u} Scalar n (add (add a (mul (@two.{u} Scalar one add) c)) b)), forall (hc : le_rel c (mul (sqrt_fn a) (sqrt_fn b))), le_rel n (@sq.{u} Scalar mul (add (sqrt_fn a) (sqrt_fn b)))"
        ),
        proof: abstract_ordered_field_abs!(concat!(
            "fun ordered_args => fun a => fun b => fun c => fun n => fun ha => fun hb => fun hn => fun hc => ",
            ordered_args_elim!(
                "(le_rel n (@sq.{u} Scalar mul (add (sqrt_fn a) (sqrt_fn b))))",
                "add_two_mul_le_sq_add_sqrt_arg a b c n ha hb hn hc"
            )
        )),
    },
];

const ABSTRACT_SQUARE_NORMALIZE_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "square_def",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (a : Scalar), @Eq.{u} Scalar (@sq.{u} Scalar mul a) (mul a a)"
        ),
        proof: abstract_ordered_field_abs!(
            "fun a => @Eq.refl.{u} Scalar (@sq.{u} Scalar mul a)"
        ),
    },
    TheoremArtifact {
        name: "mul_self_eq_square",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (a : Scalar), @Eq.{u} Scalar (mul a a) (@sq.{u} Scalar mul a)"
        ),
        proof: abstract_ordered_field_abs!("fun a => @Eq.refl.{u} Scalar (mul a a)"),
    },
    TheoremArtifact {
        name: "sq_add",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (@sq.{u} Scalar mul (add a b)) (add (add (@sq.{u} Scalar mul a) (mul (mul (@two.{u} Scalar one add) a) b)) (@sq.{u} Scalar mul b))), forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (@sq.{u} Scalar mul (add a b)) (add (add (@sq.{u} Scalar mul a) (mul (mul (@two.{u} Scalar one add) a) b)) (@sq.{u} Scalar mul b))"
        ),
        proof: abstract_ordered_field_abs!("fun law => fun a => fun b => law a b"),
    },
    TheoremArtifact {
        name: "sq_sub",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (@sq.{u} Scalar mul (sub a b)) (add (sub (@sq.{u} Scalar mul a) (mul (mul (@two.{u} Scalar one add) a) b)) (@sq.{u} Scalar mul b))), forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (@sq.{u} Scalar mul (sub a b)) (add (sub (@sq.{u} Scalar mul a) (mul (mul (@two.{u} Scalar one add) a) b)) (@sq.{u} Scalar mul b))"
        ),
        proof: abstract_ordered_field_abs!("fun law => fun a => fun b => law a b"),
    },
    TheoremArtifact {
        name: "sum_two_squares_comm",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (x : Scalar), forall (y : Scalar), @Eq.{u} Scalar (add x y) (add y x)), forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add (@sq.{u} Scalar mul a) (@sq.{u} Scalar mul b)) (add (@sq.{u} Scalar mul b) (@sq.{u} Scalar mul a))"
        ),
        proof: abstract_ordered_field_abs!(
            "fun law => fun a => fun b => law (@sq.{u} Scalar mul a) (@sq.{u} Scalar mul b)"
        ),
    },
    TheoremArtifact {
        name: "cancel_double_zero_term",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), forall (x : Scalar), forall (hzero : @Eq.{u} Scalar x zero), @Eq.{u} Scalar (add a (mul (@two.{u} Scalar one add) x)) a), forall (a : Scalar), forall (x : Scalar), forall (hzero : @Eq.{u} Scalar x zero), @Eq.{u} Scalar (add a (mul (@two.{u} Scalar one add) x)) a"
        ),
        proof: abstract_ordered_field_abs!(
            "fun law => fun a => fun x => fun hzero => law a x hzero"
        ),
    },
    TheoremArtifact {
        name: "sq_zero",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), @Eq.{u} Scalar (mul a zero) zero), @Eq.{u} Scalar (@sq.{u} Scalar mul zero) zero"
        ),
        proof: abstract_ordered_field_abs!("fun law => law zero"),
    },
    TheoremArtifact {
        name: "sq_one",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), @Eq.{u} Scalar (mul a one) a), @Eq.{u} Scalar (@sq.{u} Scalar mul one) one"
        ),
        proof: abstract_ordered_field_abs!("fun law => law one"),
    },
    TheoremArtifact {
        name: "sq_neg",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), @Eq.{u} Scalar (mul (neg a) (neg a)) (mul a a)), forall (a : Scalar), @Eq.{u} Scalar (@sq.{u} Scalar mul (neg a)) (@sq.{u} Scalar mul a)"
        ),
        proof: abstract_ordered_field_abs!("fun law => fun a => law a"),
    },
    TheoremArtifact {
        name: "two_mul",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), @Eq.{u} Scalar (mul (@two.{u} Scalar one add) a) (add a a)), forall (a : Scalar), @Eq.{u} Scalar (mul (@two.{u} Scalar one add) a) (add a a)"
        ),
        proof: abstract_ordered_field_abs!("fun law => fun a => law a"),
    },
    TheoremArtifact {
        name: "sq_eq_sq_of_eq_or_neg_eq",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), forall (h : forall (R : Prop), forall (eq_case : forall (hab : @Eq.{u} Scalar a b), R), forall (neg_case : forall (hanb : @Eq.{u} Scalar a (neg b)), R), R), @Eq.{u} Scalar (@sq.{u} Scalar mul a) (@sq.{u} Scalar mul b)), forall (a : Scalar), forall (b : Scalar), forall (h : forall (R : Prop), forall (eq_case : forall (hab : @Eq.{u} Scalar a b), R), forall (neg_case : forall (hanb : @Eq.{u} Scalar a (neg b)), R), R), @Eq.{u} Scalar (@sq.{u} Scalar mul a) (@sq.{u} Scalar mul b)"
        ),
        proof: abstract_ordered_field_abs!("fun law => fun a => fun b => fun h => law a b h"),
    },
    TheoremArtifact {
        name: "sq_add_eq_add_sq_add_two_mul",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (@sq.{u} Scalar mul (add a b)) (add (add (@sq.{u} Scalar mul a) (mul (mul (@two.{u} Scalar one add) a) b)) (@sq.{u} Scalar mul b))), forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (@sq.{u} Scalar mul (add a b)) (add (add (@sq.{u} Scalar mul a) (mul (mul (@two.{u} Scalar one add) a) b)) (@sq.{u} Scalar mul b))"
        ),
        proof: abstract_ordered_field_abs!("fun law => fun a => fun b => law a b"),
    },
    TheoremArtifact {
        name: "sq_sub_eq_add_sq_sub_two_mul",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (@sq.{u} Scalar mul (sub a b)) (add (sub (@sq.{u} Scalar mul a) (mul (mul (@two.{u} Scalar one add) a) b)) (@sq.{u} Scalar mul b))), forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (@sq.{u} Scalar mul (sub a b)) (add (sub (@sq.{u} Scalar mul a) (mul (mul (@two.{u} Scalar one add) a) b)) (@sq.{u} Scalar mul b))"
        ),
        proof: abstract_ordered_field_abs!("fun law => fun a => fun b => law a b"),
    },
    TheoremArtifact {
        name: "add_sq_eq_zero_iff",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), forall (R : Prop), forall (mk : forall (forward : forall (hsum : @Eq.{u} Scalar (add (@sq.{u} Scalar mul a) (@sq.{u} Scalar mul b)) zero), forall (S : Prop), forall (mk_pair : forall (haz : @Eq.{u} Scalar a zero), forall (hbz : @Eq.{u} Scalar b zero), S), S), forall (backward : forall (hpair : forall (S : Prop), forall (mk_pair : forall (haz : @Eq.{u} Scalar a zero), forall (hbz : @Eq.{u} Scalar b zero), S), S), @Eq.{u} Scalar (add (@sq.{u} Scalar mul a) (@sq.{u} Scalar mul b)) zero), R), R), forall (a : Scalar), forall (b : Scalar), forall (R : Prop), forall (mk : forall (forward : forall (hsum : @Eq.{u} Scalar (add (@sq.{u} Scalar mul a) (@sq.{u} Scalar mul b)) zero), forall (S : Prop), forall (mk_pair : forall (haz : @Eq.{u} Scalar a zero), forall (hbz : @Eq.{u} Scalar b zero), S), S), forall (backward : forall (hpair : forall (S : Prop), forall (mk_pair : forall (haz : @Eq.{u} Scalar a zero), forall (hbz : @Eq.{u} Scalar b zero), S), S), @Eq.{u} Scalar (add (@sq.{u} Scalar mul a) (@sq.{u} Scalar mul b)) zero), R), R"
        ),
        proof: abstract_ordered_field_abs!(
            "fun law => fun a => fun b => fun R => fun mk => law a b R mk"
        ),
    },
    TheoremArtifact {
        name: "mul_two_zero_term",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (x : Scalar), forall (hzero : @Eq.{u} Scalar x zero), @Eq.{u} Scalar (mul (@two.{u} Scalar one add) x) zero), forall (x : Scalar), forall (hzero : @Eq.{u} Scalar x zero), @Eq.{u} Scalar (mul (@two.{u} Scalar one add) x) zero"
        ),
        proof: abstract_ordered_field_abs!("fun law => fun x => fun hzero => law x hzero"),
    },
    TheoremArtifact {
        name: "normalize_add_with_zero_cross_term",
        universe_params: &["u"],
        statement: abstract_ordered_field_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), forall (x : Scalar), forall (hzero : @Eq.{u} Scalar x zero), @Eq.{u} Scalar (add (add a (mul (@two.{u} Scalar one add) x)) b) (add a b)), forall (a : Scalar), forall (b : Scalar), forall (x : Scalar), forall (hzero : @Eq.{u} Scalar x zero), @Eq.{u} Scalar (add (add a (mul (@two.{u} Scalar one add) x)) b) (add a b)"
        ),
        proof: abstract_ordered_field_abs!(
            "fun law => fun a => fun b => fun x => fun hzero => law a b x hzero"
        ),
    },
];

const ABSTRACT_SCALAR_DERIVE_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "mul_two_zero_term_from_ring_args",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (x : Scalar), forall (hzero : @Eq.{u} Scalar x zero), @Eq.{u} Scalar (mul (@two.{u} Scalar one add) x) zero"
        ),
        proof: abstract_ring_abs!(concat!(
            "fun ring_args => fun x => fun hzero => ",
            "ring_args (@Eq.{u} Scalar (mul (@two.{u} Scalar one add) x) zero) ",
            "(fun (sub_eq_add_neg_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub a b) (add a (neg b))) => ",
            "fun (add_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add a b) c) (add a (add b c))) => ",
            "fun (add_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add a b) (add b a)) => ",
            "fun (add_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (add a zero) a) => ",
            "fun (zero_add_arg : forall (a : Scalar), @Eq.{u} Scalar (add zero a) a) => ",
            "fun (neg_add_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add (neg a) a) zero) => ",
            "fun (add_neg_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add a (neg a)) zero) => ",
            "fun (sub_self_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a a) zero) => ",
            "fun (mul_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (mul a b) c) (mul a (mul b c))) => ",
            "fun (mul_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (mul a b) (mul b a)) => ",
            "fun (mul_one_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a one) a) => ",
            "fun (one_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul one a) a) => ",
            "fun (left_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul a (add b c)) (add (mul a b) (mul a c))) => ",
            "fun (right_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (add a b) c) (add (mul a c) (mul b c))) => ",
            "fun (mul_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a zero) zero) => ",
            "fun (zero_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul zero a) zero) => ",
            "fun (add_left_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add a b) (add a c)), @Eq.{u} Scalar b c) => ",
            "fun (ring_normalize_add_mul3_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add (mul a b) (mul b c)) (mul a c)) (add (add (mul a b) (mul a c)) (mul b c))) => ",
            "fun (add_right_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add b a) (add c a)), @Eq.{u} Scalar b c) => ",
            "fun (neg_neg_arg : forall (a : Scalar), @Eq.{u} Scalar (neg (neg a)) a) => ",
            "fun (sub_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a zero) a) => ",
            "fun (zero_sub_arg : forall (a : Scalar), @Eq.{u} Scalar (sub zero a) (neg a)) => ",
            "fun (sub_add_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add (sub a b) b) a) => ",
            "fun (add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub (add a b) b) a) => ",
            "fun (sub_add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (sub (sub a c) (sub b c)) (sub a b)) => ",
            "@Eq.rec.{u,0} Scalar zero ",
            "(fun (y : Scalar) => fun (hy : @Eq.{u} Scalar zero y) => @Eq.{u} Scalar (mul (@two.{u} Scalar one add) y) zero) ",
            "(mul_zero_arg (@two.{u} Scalar one add)) x ",
            "(@Eq.rec.{u,0} Scalar x ",
            "(fun (y : Scalar) => fun (hy : @Eq.{u} Scalar x y) => @Eq.{u} Scalar y x) ",
            "(@Eq.refl.{u} Scalar x) zero hzero))"
        )),
    },
    TheoremArtifact {
        name: "cancel_double_zero_term_from_ring_args",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (a : Scalar), forall (x : Scalar), forall (hzero : @Eq.{u} Scalar x zero), @Eq.{u} Scalar (add a (mul (@two.{u} Scalar one add) x)) a"
        ),
        proof: abstract_ring_abs!(concat!(
            "fun ring_args => fun a => fun x => fun hzero => ",
            "ring_args (@Eq.{u} Scalar (add a (mul (@two.{u} Scalar one add) x)) a) ",
            "(fun (sub_eq_add_neg_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub a b) (add a (neg b))) => ",
            "fun (add_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add a b) c) (add a (add b c))) => ",
            "fun (add_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add a b) (add b a)) => ",
            "fun (add_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (add a zero) a) => ",
            "fun (zero_add_arg : forall (a : Scalar), @Eq.{u} Scalar (add zero a) a) => ",
            "fun (neg_add_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add (neg a) a) zero) => ",
            "fun (add_neg_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add a (neg a)) zero) => ",
            "fun (sub_self_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a a) zero) => ",
            "fun (mul_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (mul a b) c) (mul a (mul b c))) => ",
            "fun (mul_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (mul a b) (mul b a)) => ",
            "fun (mul_one_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a one) a) => ",
            "fun (one_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul one a) a) => ",
            "fun (left_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul a (add b c)) (add (mul a b) (mul a c))) => ",
            "fun (right_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (add a b) c) (add (mul a c) (mul b c))) => ",
            "fun (mul_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a zero) zero) => ",
            "fun (zero_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul zero a) zero) => ",
            "fun (add_left_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add a b) (add a c)), @Eq.{u} Scalar b c) => ",
            "fun (ring_normalize_add_mul3_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add (mul a b) (mul b c)) (mul a c)) (add (add (mul a b) (mul a c)) (mul b c))) => ",
            "fun (add_right_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add b a) (add c a)), @Eq.{u} Scalar b c) => ",
            "fun (neg_neg_arg : forall (a : Scalar), @Eq.{u} Scalar (neg (neg a)) a) => ",
            "fun (sub_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a zero) a) => ",
            "fun (zero_sub_arg : forall (a : Scalar), @Eq.{u} Scalar (sub zero a) (neg a)) => ",
            "fun (sub_add_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add (sub a b) b) a) => ",
            "fun (add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub (add a b) b) a) => ",
            "fun (sub_add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (sub (sub a c) (sub b c)) (sub a b)) => ",
            "@Eq.rec.{u,0} Scalar zero ",
            "(fun (y : Scalar) => fun (hy : @Eq.{u} Scalar zero y) => @Eq.{u} Scalar (add a y) a) ",
            "(add_zero_arg a) (mul (@two.{u} Scalar one add) x) ",
            "(@Eq.rec.{u,0} Scalar (mul (@two.{u} Scalar one add) x) ",
            "(fun (y : Scalar) => fun (hy : @Eq.{u} Scalar (mul (@two.{u} Scalar one add) x) y) => @Eq.{u} Scalar y (mul (@two.{u} Scalar one add) x)) ",
            "(@Eq.refl.{u} Scalar (mul (@two.{u} Scalar one add) x)) zero ",
            "(@mul_two_zero_term_from_ring_args.{u} Scalar zero one add neg sub mul ring_args x hzero)))"
        )),
    },
    TheoremArtifact {
        name: "normalize_add_with_zero_cross_term_from_ring_args",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (a : Scalar), forall (b : Scalar), forall (x : Scalar), forall (hzero : @Eq.{u} Scalar x zero), @Eq.{u} Scalar (add (add a (mul (@two.{u} Scalar one add) x)) b) (add a b)"
        ),
        proof: abstract_ring_abs!(concat!(
            "fun ring_args => fun a => fun b => fun x => fun hzero => ",
            "@Eq.rec.{u,0} Scalar (add a (mul (@two.{u} Scalar one add) x)) ",
            "(fun (y : Scalar) => fun (hy : @Eq.{u} Scalar (add a (mul (@two.{u} Scalar one add) x)) y) => @Eq.{u} Scalar (add (add a (mul (@two.{u} Scalar one add) x)) b) (add y b)) ",
            "(@Eq.refl.{u} Scalar (add (add a (mul (@two.{u} Scalar one add) x)) b)) ",
            "a (@cancel_double_zero_term_from_ring_args.{u} Scalar zero one add neg sub mul ring_args a x hzero)"
        )),
    },
    TheoremArtifact {
        name: "mul_two_neg_from_ring_args",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (x : Scalar), @Eq.{u} Scalar (mul (@two.{u} Scalar one add) (neg x)) (neg (mul (@two.{u} Scalar one add) x))"
        ),
        proof: abstract_ring_abs!(concat!(
            "fun ring_args => fun x => ",
            "ring_args (@Eq.{u} Scalar (mul (@two.{u} Scalar one add) (neg x)) (neg (mul (@two.{u} Scalar one add) x))) ",
            "(fun (sub_eq_add_neg_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub a b) (add a (neg b))) => ",
            "fun (add_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add a b) c) (add a (add b c))) => ",
            "fun (add_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add a b) (add b a)) => ",
            "fun (add_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (add a zero) a) => ",
            "fun (zero_add_arg : forall (a : Scalar), @Eq.{u} Scalar (add zero a) a) => ",
            "fun (neg_add_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add (neg a) a) zero) => ",
            "fun (add_neg_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add a (neg a)) zero) => ",
            "fun (sub_self_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a a) zero) => ",
            "fun (mul_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (mul a b) c) (mul a (mul b c))) => ",
            "fun (mul_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (mul a b) (mul b a)) => ",
            "fun (mul_one_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a one) a) => ",
            "fun (one_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul one a) a) => ",
            "fun (left_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul a (add b c)) (add (mul a b) (mul a c))) => ",
            "fun (right_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (add a b) c) (add (mul a c) (mul b c))) => ",
            "fun (mul_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a zero) zero) => ",
            "fun (zero_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul zero a) zero) => ",
            "fun (add_left_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add a b) (add a c)), @Eq.{u} Scalar b c) => ",
            "fun (ring_normalize_add_mul3_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add (mul a b) (mul b c)) (mul a c)) (add (add (mul a b) (mul a c)) (mul b c))) => ",
            "fun (add_right_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add b a) (add c a)), @Eq.{u} Scalar b c) => ",
            "fun (neg_neg_arg : forall (a : Scalar), @Eq.{u} Scalar (neg (neg a)) a) => ",
            "fun (sub_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a zero) a) => ",
            "fun (zero_sub_arg : forall (a : Scalar), @Eq.{u} Scalar (sub zero a) (neg a)) => ",
            "fun (sub_add_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add (sub a b) b) a) => ",
            "fun (add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub (add a b) b) a) => ",
            "fun (sub_add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (sub (sub a c) (sub b c)) (sub a b)) => ",
            "add_left_cancel_arg (mul (@two.{u} Scalar one add) x) (mul (@two.{u} Scalar one add) (neg x)) (neg (mul (@two.{u} Scalar one add) x)) ",
            "(@Eq.rec.{u,0} Scalar zero ",
            "(fun (y : Scalar) => fun (hy : @Eq.{u} Scalar zero y) => @Eq.{u} Scalar (add (mul (@two.{u} Scalar one add) x) (mul (@two.{u} Scalar one add) (neg x))) y) ",
            "(@Eq.rec.{u,0} Scalar (mul (@two.{u} Scalar one add) (add x (neg x))) ",
            "(fun (y : Scalar) => fun (hy : @Eq.{u} Scalar (mul (@two.{u} Scalar one add) (add x (neg x))) y) => @Eq.{u} Scalar y zero) ",
            "(@Eq.rec.{u,0} Scalar (mul (@two.{u} Scalar one add) zero) ",
            "(fun (y : Scalar) => fun (hy : @Eq.{u} Scalar (mul (@two.{u} Scalar one add) zero) y) => @Eq.{u} Scalar (mul (@two.{u} Scalar one add) (add x (neg x))) y) ",
            "(@Eq.rec.{u,0} Scalar (add x (neg x)) ",
            "(fun (y : Scalar) => fun (hy : @Eq.{u} Scalar (add x (neg x)) y) => @Eq.{u} Scalar (mul (@two.{u} Scalar one add) (add x (neg x))) (mul (@two.{u} Scalar one add) y)) ",
            "(@Eq.refl.{u} Scalar (mul (@two.{u} Scalar one add) (add x (neg x)))) zero (add_neg_cancel_arg x)) ",
            "zero (mul_zero_arg (@two.{u} Scalar one add))) ",
            "(add (mul (@two.{u} Scalar one add) x) (mul (@two.{u} Scalar one add) (neg x))) ",
            "(left_distrib_arg (@two.{u} Scalar one add) x (neg x))) ",
            "(add (mul (@two.{u} Scalar one add) x) (neg (mul (@two.{u} Scalar one add) x))) ",
            "(@Eq.rec.{u,0} Scalar (add (mul (@two.{u} Scalar one add) x) (neg (mul (@two.{u} Scalar one add) x))) ",
            "(fun (y : Scalar) => fun (hy : @Eq.{u} Scalar (add (mul (@two.{u} Scalar one add) x) (neg (mul (@two.{u} Scalar one add) x))) y) => @Eq.{u} Scalar y (add (mul (@two.{u} Scalar one add) x) (neg (mul (@two.{u} Scalar one add) x)))) ",
            "(@Eq.refl.{u} Scalar (add (mul (@two.{u} Scalar one add) x) (neg (mul (@two.{u} Scalar one add) x)))) zero ",
            "(add_neg_cancel_arg (mul (@two.{u} Scalar one add) x)))))"
        )),
    },
    TheoremArtifact {
        name: "add_neg_cross_term_to_sub_sum_from_ring_args",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (a : Scalar), forall (b : Scalar), forall (t : Scalar), @Eq.{u} Scalar (add (add a (neg t)) b) (sub (add a b) t)"
        ),
        proof: abstract_ring_abs!(concat!(
            "fun ring_args => fun a => fun b => fun t => ",
            "ring_args (@Eq.{u} Scalar (add (add a (neg t)) b) (sub (add a b) t)) ",
            "(fun (sub_eq_add_neg_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub a b) (add a (neg b))) => ",
            "fun (add_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add a b) c) (add a (add b c))) => ",
            "fun (add_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add a b) (add b a)) => ",
            "fun (add_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (add a zero) a) => ",
            "fun (zero_add_arg : forall (a : Scalar), @Eq.{u} Scalar (add zero a) a) => ",
            "fun (neg_add_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add (neg a) a) zero) => ",
            "fun (add_neg_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add a (neg a)) zero) => ",
            "fun (sub_self_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a a) zero) => ",
            "fun (mul_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (mul a b) c) (mul a (mul b c))) => ",
            "fun (mul_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (mul a b) (mul b a)) => ",
            "fun (mul_one_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a one) a) => ",
            "fun (one_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul one a) a) => ",
            "fun (left_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul a (add b c)) (add (mul a b) (mul a c))) => ",
            "fun (right_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (add a b) c) (add (mul a c) (mul b c))) => ",
            "fun (mul_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a zero) zero) => ",
            "fun (zero_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul zero a) zero) => ",
            "fun (add_left_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add a b) (add a c)), @Eq.{u} Scalar b c) => ",
            "fun (ring_normalize_add_mul3_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add (mul a b) (mul b c)) (mul a c)) (add (add (mul a b) (mul a c)) (mul b c))) => ",
            "fun (add_right_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add b a) (add c a)), @Eq.{u} Scalar b c) => ",
            "fun (neg_neg_arg : forall (a : Scalar), @Eq.{u} Scalar (neg (neg a)) a) => ",
            "fun (sub_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a zero) a) => ",
            "fun (zero_sub_arg : forall (a : Scalar), @Eq.{u} Scalar (sub zero a) (neg a)) => ",
            "fun (sub_add_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add (sub a b) b) a) => ",
            "fun (add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub (add a b) b) a) => ",
            "fun (sub_add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (sub (sub a c) (sub b c)) (sub a b)) => ",
            "@Eq.rec.{u,0} Scalar (add a (add (neg t) b)) ",
            "(fun (y : Scalar) => fun (hy : @Eq.{u} Scalar (add a (add (neg t) b)) y) => @Eq.{u} Scalar (add (add a (neg t)) b) y) ",
            "(add_assoc_arg a (neg t) b) (sub (add a b) t) ",
            "(@Eq.rec.{u,0} Scalar (add a (add b (neg t))) ",
            "(fun (y : Scalar) => fun (hy : @Eq.{u} Scalar (add a (add b (neg t))) y) => @Eq.{u} Scalar (add a (add (neg t) b)) y) ",
            "(@Eq.rec.{u,0} Scalar (add (neg t) b) ",
            "(fun (y : Scalar) => fun (hy : @Eq.{u} Scalar (add (neg t) b) y) => @Eq.{u} Scalar (add a (add (neg t) b)) (add a y)) ",
            "(@Eq.refl.{u} Scalar (add a (add (neg t) b))) (add b (neg t)) (add_comm_arg (neg t) b)) ",
            "(sub (add a b) t) ",
            "(@Eq.rec.{u,0} Scalar (add (add a b) (neg t)) ",
            "(fun (y : Scalar) => fun (hy : @Eq.{u} Scalar (add (add a b) (neg t)) y) => @Eq.{u} Scalar (add a (add b (neg t))) y) ",
            "(@Eq.rec.{u,0} Scalar (add (add a b) (neg t)) ",
            "(fun (y : Scalar) => fun (hy : @Eq.{u} Scalar (add (add a b) (neg t)) y) => @Eq.{u} Scalar y (add (add a b) (neg t))) ",
            "(@Eq.refl.{u} Scalar (add (add a b) (neg t))) (add a (add b (neg t))) (add_assoc_arg a b (neg t))) ",
            "(sub (add a b) t) ",
            "(@Eq.rec.{u,0} Scalar (sub (add a b) t) ",
            "(fun (y : Scalar) => fun (hy : @Eq.{u} Scalar (sub (add a b) t) y) => @Eq.{u} Scalar y (sub (add a b) t)) ",
            "(@Eq.refl.{u} Scalar (sub (add a b) t)) (add (add a b) (neg t)) (sub_eq_add_neg_arg (add a b) t)))))"
        )),
    },
    TheoremArtifact {
        name: "law_of_cosines_scalar_rhs_from_ring_args",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (a : Scalar), forall (b : Scalar), forall (x : Scalar), @Eq.{u} Scalar (add (add a (mul (@two.{u} Scalar one add) (neg x))) b) (sub (add a b) (mul (@two.{u} Scalar one add) x))"
        ),
        proof: abstract_ring_abs!(concat!(
            "fun ring_args => fun a => fun b => fun x => ",
            "@Eq.rec.{u,0} Scalar (add (add a (neg (mul (@two.{u} Scalar one add) x))) b) ",
            "(fun (y : Scalar) => fun (hy : @Eq.{u} Scalar (add (add a (neg (mul (@two.{u} Scalar one add) x))) b) y) => @Eq.{u} Scalar (add (add a (mul (@two.{u} Scalar one add) (neg x))) b) y) ",
            "(@Eq.rec.{u,0} Scalar (mul (@two.{u} Scalar one add) (neg x)) ",
            "(fun (y : Scalar) => fun (hy : @Eq.{u} Scalar (mul (@two.{u} Scalar one add) (neg x)) y) => @Eq.{u} Scalar (add (add a (mul (@two.{u} Scalar one add) (neg x))) b) (add (add a y) b)) ",
            "(@Eq.refl.{u} Scalar (add (add a (mul (@two.{u} Scalar one add) (neg x))) b)) ",
            "(neg (mul (@two.{u} Scalar one add) x)) ",
            "(@mul_two_neg_from_ring_args.{u} Scalar zero one add neg sub mul ring_args x)) ",
            "(sub (add a b) (mul (@two.{u} Scalar one add) x)) ",
            "(@add_neg_cross_term_to_sub_sum_from_ring_args.{u} Scalar zero one add neg sub mul ring_args a b (mul (@two.{u} Scalar one add) x))"
        )),
    },
    TheoremArtifact {
        name: "two_mul_from_ring_args",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (a : Scalar), @Eq.{u} Scalar (mul (@two.{u} Scalar one add) a) (add a a)"
        ),
        proof: abstract_ring_abs!(concat!(
            "fun ring_args => fun a => ",
            "ring_args (@Eq.{u} Scalar (mul (@two.{u} Scalar one add) a) (add a a)) ",
            "(fun (sub_eq_add_neg_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub a b) (add a (neg b))) => ",
            "fun (add_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add a b) c) (add a (add b c))) => ",
            "fun (add_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add a b) (add b a)) => ",
            "fun (add_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (add a zero) a) => ",
            "fun (zero_add_arg : forall (a : Scalar), @Eq.{u} Scalar (add zero a) a) => ",
            "fun (neg_add_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add (neg a) a) zero) => ",
            "fun (add_neg_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add a (neg a)) zero) => ",
            "fun (sub_self_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a a) zero) => ",
            "fun (mul_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (mul a b) c) (mul a (mul b c))) => ",
            "fun (mul_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (mul a b) (mul b a)) => ",
            "fun (mul_one_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a one) a) => ",
            "fun (one_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul one a) a) => ",
            "fun (left_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul a (add b c)) (add (mul a b) (mul a c))) => ",
            "fun (right_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (add a b) c) (add (mul a c) (mul b c))) => ",
            "fun (mul_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a zero) zero) => ",
            "fun (zero_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul zero a) zero) => ",
            "fun (add_left_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add a b) (add a c)), @Eq.{u} Scalar b c) => ",
            "fun (ring_normalize_add_mul3_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add (mul a b) (mul b c)) (mul a c)) (add (add (mul a b) (mul a c)) (mul b c))) => ",
            "fun (add_right_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add b a) (add c a)), @Eq.{u} Scalar b c) => ",
            "fun (neg_neg_arg : forall (a : Scalar), @Eq.{u} Scalar (neg (neg a)) a) => ",
            "fun (sub_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a zero) a) => ",
            "fun (zero_sub_arg : forall (a : Scalar), @Eq.{u} Scalar (sub zero a) (neg a)) => ",
            "fun (sub_add_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add (sub a b) b) a) => ",
            "fun (add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub (add a b) b) a) => ",
            "fun (sub_add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (sub (sub a c) (sub b c)) (sub a b)) => ",
            "@eq_trans.{u} Scalar (mul (@two.{u} Scalar one add) a) (add (mul one a) (mul one a)) (add a a) ",
            "(right_distrib_arg one one a) ",
            "(@eq_congr2.{u,u,u} Scalar Scalar Scalar add (mul one a) a (mul one a) a (one_mul_arg a) (one_mul_arg a)))"
        )),
    },
    TheoremArtifact {
        name: "add_sub_cross_cancel_from_ring_args",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (a : Scalar), forall (x : Scalar), @Eq.{u} Scalar (add x (sub a x)) a"
        ),
        proof: abstract_ring_abs!(concat!(
            "fun ring_args => fun a => fun x => ",
            "ring_args (@Eq.{u} Scalar (add x (sub a x)) a) ",
            "(fun (sub_eq_add_neg_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub a b) (add a (neg b))) => ",
            "fun (add_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add a b) c) (add a (add b c))) => ",
            "fun (add_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add a b) (add b a)) => ",
            "fun (add_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (add a zero) a) => ",
            "fun (zero_add_arg : forall (a : Scalar), @Eq.{u} Scalar (add zero a) a) => ",
            "fun (neg_add_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add (neg a) a) zero) => ",
            "fun (add_neg_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add a (neg a)) zero) => ",
            "fun (sub_self_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a a) zero) => ",
            "fun (mul_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (mul a b) c) (mul a (mul b c))) => ",
            "fun (mul_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (mul a b) (mul b a)) => ",
            "fun (mul_one_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a one) a) => ",
            "fun (one_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul one a) a) => ",
            "fun (left_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul a (add b c)) (add (mul a b) (mul a c))) => ",
            "fun (right_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (add a b) c) (add (mul a c) (mul b c))) => ",
            "fun (mul_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a zero) zero) => ",
            "fun (zero_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul zero a) zero) => ",
            "fun (add_left_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add a b) (add a c)), @Eq.{u} Scalar b c) => ",
            "fun (ring_normalize_add_mul3_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add (mul a b) (mul b c)) (mul a c)) (add (add (mul a b) (mul a c)) (mul b c))) => ",
            "fun (add_right_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add b a) (add c a)), @Eq.{u} Scalar b c) => ",
            "fun (neg_neg_arg : forall (a : Scalar), @Eq.{u} Scalar (neg (neg a)) a) => ",
            "fun (sub_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a zero) a) => ",
            "fun (zero_sub_arg : forall (a : Scalar), @Eq.{u} Scalar (sub zero a) (neg a)) => ",
            "fun (sub_add_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add (sub a b) b) a) => ",
            "fun (add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub (add a b) b) a) => ",
            "fun (sub_add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (sub (sub a c) (sub b c)) (sub a b)) => ",
            "@eq_trans.{u} Scalar (add x (sub a x)) (add (sub a x) x) a ",
            "(add_comm_arg x (sub a x)) ",
            "(sub_add_cancel_arg a x))"
        )),
    },
    TheoremArtifact {
        name: "add_pairwise_commute_from_ring_args",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (d : Scalar), @Eq.{u} Scalar (add (add a b) (add c d)) (add (add a c) (add b d))"
        ),
        proof: abstract_ring_abs!(concat!(
            "fun ring_args => fun a => fun b => fun c => fun d => ",
            "ring_args (@Eq.{u} Scalar (add (add a b) (add c d)) (add (add a c) (add b d))) ",
            "(fun (sub_eq_add_neg_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub a b) (add a (neg b))) => ",
            "fun (add_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add a b) c) (add a (add b c))) => ",
            "fun (add_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add a b) (add b a)) => ",
            "fun (add_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (add a zero) a) => ",
            "fun (zero_add_arg : forall (a : Scalar), @Eq.{u} Scalar (add zero a) a) => ",
            "fun (neg_add_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add (neg a) a) zero) => ",
            "fun (add_neg_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add a (neg a)) zero) => ",
            "fun (sub_self_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a a) zero) => ",
            "fun (mul_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (mul a b) c) (mul a (mul b c))) => ",
            "fun (mul_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (mul a b) (mul b a)) => ",
            "fun (mul_one_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a one) a) => ",
            "fun (one_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul one a) a) => ",
            "fun (left_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul a (add b c)) (add (mul a b) (mul a c))) => ",
            "fun (right_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (add a b) c) (add (mul a c) (mul b c))) => ",
            "fun (mul_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a zero) zero) => ",
            "fun (zero_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul zero a) zero) => ",
            "fun (add_left_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add a b) (add a c)), @Eq.{u} Scalar b c) => ",
            "fun (ring_normalize_add_mul3_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add (mul a b) (mul b c)) (mul a c)) (add (add (mul a b) (mul a c)) (mul b c))) => ",
            "fun (add_right_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add b a) (add c a)), @Eq.{u} Scalar b c) => ",
            "fun (neg_neg_arg : forall (a : Scalar), @Eq.{u} Scalar (neg (neg a)) a) => ",
            "fun (sub_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a zero) a) => ",
            "fun (zero_sub_arg : forall (a : Scalar), @Eq.{u} Scalar (sub zero a) (neg a)) => ",
            "fun (sub_add_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add (sub a b) b) a) => ",
            "fun (add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub (add a b) b) a) => ",
            "fun (sub_add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (sub (sub a c) (sub b c)) (sub a b)) => ",
            "@eq_calc3.{u} Scalar (add (add a b) (add c d)) (add a (add b (add c d))) (add a (add c (add b d))) (add (add a c) (add b d)) ",
            "(add_assoc_arg a b (add c d)) ",
            "(@eq_congr_arg.{u,u} Scalar Scalar (fun (z : Scalar) => add a z) (add b (add c d)) (add c (add b d)) ",
            "(@eq_calc3.{u} Scalar (add b (add c d)) (add (add b c) d) (add (add c b) d) (add c (add b d)) ",
            "(@eq_symm.{u} Scalar (add (add b c) d) (add b (add c d)) (add_assoc_arg b c d)) ",
            "(@eq_congr_arg.{u,u} Scalar Scalar (fun (z : Scalar) => add z d) (add b c) (add c b) (add_comm_arg b c)) ",
            "(add_assoc_arg c b d))) ",
            "(@eq_symm.{u} Scalar (add (add a c) (add b d)) (add a (add c (add b d))) (add_assoc_arg a c (add b d))))"
        )),
    },
    TheoremArtifact {
        name: "add_cross_and_sub_cross_cancel_from_ring_args",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (a : Scalar), forall (b : Scalar), forall (x : Scalar), @Eq.{u} Scalar (add (add (add a x) b) (add (sub a x) b)) (add (add a a) (add b b))"
        ),
        proof: abstract_ring_abs!(concat!(
            "fun ring_args => fun a => fun b => fun x => ",
            "@eq_trans.{u} Scalar (add (add (add a x) b) (add (sub a x) b)) (add (add (add a x) (sub a x)) (add b b)) (add (add a a) (add b b)) ",
            "(@add_pairwise_commute_from_ring_args.{u} Scalar zero one add neg sub mul ring_args (add a x) b (sub a x) b) ",
            "(@eq_congr2.{u,u,u} Scalar Scalar Scalar add (add (add a x) (sub a x)) (add a a) (add b b) (add b b) ",
            "(@eq_trans.{u} Scalar (add (add a x) (sub a x)) (add a (add x (sub a x))) (add a a) ",
            "(ring_args (@Eq.{u} Scalar (add (add a x) (sub a x)) (add a (add x (sub a x)))) ",
            "(fun (sub_eq_add_neg_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub a b) (add a (neg b))) => ",
            "fun (add_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add a b) c) (add a (add b c))) => ",
            "fun (add_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add a b) (add b a)) => ",
            "fun (add_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (add a zero) a) => ",
            "fun (zero_add_arg : forall (a : Scalar), @Eq.{u} Scalar (add zero a) a) => ",
            "fun (neg_add_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add (neg a) a) zero) => ",
            "fun (add_neg_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add a (neg a)) zero) => ",
            "fun (sub_self_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a a) zero) => ",
            "fun (mul_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (mul a b) c) (mul a (mul b c))) => ",
            "fun (mul_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (mul a b) (mul b a)) => ",
            "fun (mul_one_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a one) a) => ",
            "fun (one_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul one a) a) => ",
            "fun (left_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul a (add b c)) (add (mul a b) (mul a c))) => ",
            "fun (right_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (add a b) c) (add (mul a c) (mul b c))) => ",
            "fun (mul_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a zero) zero) => ",
            "fun (zero_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul zero a) zero) => ",
            "fun (add_left_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add a b) (add a c)), @Eq.{u} Scalar b c) => ",
            "fun (ring_normalize_add_mul3_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add (mul a b) (mul b c)) (mul a c)) (add (add (mul a b) (mul a c)) (mul b c))) => ",
            "fun (add_right_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add b a) (add c a)), @Eq.{u} Scalar b c) => ",
            "fun (neg_neg_arg : forall (a : Scalar), @Eq.{u} Scalar (neg (neg a)) a) => ",
            "fun (sub_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a zero) a) => ",
            "fun (zero_sub_arg : forall (a : Scalar), @Eq.{u} Scalar (sub zero a) (neg a)) => ",
            "fun (sub_add_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add (sub a b) b) a) => ",
            "fun (add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub (add a b) b) a) => ",
            "fun (sub_add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (sub (sub a c) (sub b c)) (sub a b)) => add_assoc_arg a x (sub a x))) ",
            "(@eq_congr_arg.{u,u} Scalar Scalar (fun (z : Scalar) => add a z) (add x (sub a x)) a ",
            "(@add_sub_cross_cancel_from_ring_args.{u} Scalar zero one add neg sub mul ring_args a x))) ",
            "(@Eq.refl.{u} Scalar (add b b)))"
        )),
    },
    TheoremArtifact {
        name: "parallelogram_scalar_rhs_from_ring_args",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (a : Scalar), forall (b : Scalar), forall (x : Scalar), @Eq.{u} Scalar (add (add (add a x) b) (add (sub a x) b)) (add (mul (@two.{u} Scalar one add) a) (mul (@two.{u} Scalar one add) b))"
        ),
        proof: abstract_ring_abs!(concat!(
            "fun ring_args => fun a => fun b => fun x => ",
            "@eq_calc3.{u} Scalar (add (add (add a x) b) (add (sub a x) b)) (add (add a a) (add b b)) (add (mul (@two.{u} Scalar one add) a) (add b b)) (add (mul (@two.{u} Scalar one add) a) (mul (@two.{u} Scalar one add) b)) ",
            "(@add_cross_and_sub_cross_cancel_from_ring_args.{u} Scalar zero one add neg sub mul ring_args a b x) ",
            "(@eq_congr2.{u,u,u} Scalar Scalar Scalar add (add a a) (mul (@two.{u} Scalar one add) a) (add b b) (add b b) ",
            "(@eq_symm.{u} Scalar (mul (@two.{u} Scalar one add) a) (add a a) (@two_mul_from_ring_args.{u} Scalar zero one add neg sub mul ring_args a)) ",
            "(@Eq.refl.{u} Scalar (add b b))) ",
            "(@eq_congr2.{u,u,u} Scalar Scalar Scalar add (mul (@two.{u} Scalar one add) a) (mul (@two.{u} Scalar one add) a) (add b b) (mul (@two.{u} Scalar one add) b) ",
            "(@Eq.refl.{u} Scalar (mul (@two.{u} Scalar one add) a)) ",
            "(@eq_symm.{u} Scalar (mul (@two.{u} Scalar one add) b) (add b b) (@two_mul_from_ring_args.{u} Scalar zero one add neg sub mul ring_args b)))"
        )),
    },
    TheoremArtifact {
        name: "add_middle_to_front_from_ring_args",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (a : Scalar), forall (x : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add (add a x) b) (add x (add a b))"
        ),
        proof: abstract_ring_abs!(concat!(
            "fun ring_args => fun a => fun x => fun b => ",
            "ring_args (@Eq.{u} Scalar (add (add a x) b) (add x (add a b))) ",
            "(fun (sub_eq_add_neg_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub a b) (add a (neg b))) => ",
            "fun (add_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add a b) c) (add a (add b c))) => ",
            "fun (add_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add a b) (add b a)) => ",
            "fun (add_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (add a zero) a) => ",
            "fun (zero_add_arg : forall (a : Scalar), @Eq.{u} Scalar (add zero a) a) => ",
            "fun (neg_add_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add (neg a) a) zero) => ",
            "fun (add_neg_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add a (neg a)) zero) => ",
            "fun (sub_self_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a a) zero) => ",
            "fun (mul_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (mul a b) c) (mul a (mul b c))) => ",
            "fun (mul_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (mul a b) (mul b a)) => ",
            "fun (mul_one_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a one) a) => ",
            "fun (one_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul one a) a) => ",
            "fun (left_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul a (add b c)) (add (mul a b) (mul a c))) => ",
            "fun (right_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (add a b) c) (add (mul a c) (mul b c))) => ",
            "fun (mul_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a zero) zero) => ",
            "fun (zero_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul zero a) zero) => ",
            "fun (add_left_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add a b) (add a c)), @Eq.{u} Scalar b c) => ",
            "fun (ring_normalize_add_mul3_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add (mul a b) (mul b c)) (mul a c)) (add (add (mul a b) (mul a c)) (mul b c))) => ",
            "fun (add_right_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add b a) (add c a)), @Eq.{u} Scalar b c) => ",
            "fun (neg_neg_arg : forall (a : Scalar), @Eq.{u} Scalar (neg (neg a)) a) => ",
            "fun (sub_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a zero) a) => ",
            "fun (zero_sub_arg : forall (a : Scalar), @Eq.{u} Scalar (sub zero a) (neg a)) => ",
            "fun (sub_add_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add (sub a b) b) a) => ",
            "fun (add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub (add a b) b) a) => ",
            "fun (sub_add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (sub (sub a c) (sub b c)) (sub a b)) => ",
            "@eq_calc3.{u} Scalar (add (add a x) b) (add a (add x b)) (add a (add b x)) (add x (add a b)) ",
            "(add_assoc_arg a x b) ",
            "(@eq_congr_arg.{u,u} Scalar Scalar (fun (z : Scalar) => add a z) (add x b) (add b x) (add_comm_arg x b)) ",
            "(@eq_trans.{u} Scalar (add a (add b x)) (add (add a b) x) (add x (add a b)) ",
            "(@eq_symm.{u} Scalar (add (add a b) x) (add a (add b x)) (add_assoc_arg a b x)) ",
            "(add_comm_arg (add a b) x)))"
        )),
    },
    TheoremArtifact {
        name: "polarization_scalar_rhs_from_ring_args",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (nx : Scalar), forall (ny : Scalar), forall (d : Scalar), @Eq.{u} Scalar (mul (@two.{u} Scalar one add) d) (sub (add (add nx (mul (@two.{u} Scalar one add) d)) ny) (add nx ny))"
        ),
        proof: abstract_ring_abs!(concat!(
            "fun ring_args => fun nx => fun ny => fun d => ",
            "ring_args (@Eq.{u} Scalar (mul (@two.{u} Scalar one add) d) (sub (add (add nx (mul (@two.{u} Scalar one add) d)) ny) (add nx ny))) ",
            "(fun (sub_eq_add_neg_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub a b) (add a (neg b))) => ",
            "fun (add_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add a b) c) (add a (add b c))) => ",
            "fun (add_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add a b) (add b a)) => ",
            "fun (add_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (add a zero) a) => ",
            "fun (zero_add_arg : forall (a : Scalar), @Eq.{u} Scalar (add zero a) a) => ",
            "fun (neg_add_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add (neg a) a) zero) => ",
            "fun (add_neg_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add a (neg a)) zero) => ",
            "fun (sub_self_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a a) zero) => ",
            "fun (mul_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (mul a b) c) (mul a (mul b c))) => ",
            "fun (mul_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (mul a b) (mul b a)) => ",
            "fun (mul_one_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a one) a) => ",
            "fun (one_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul one a) a) => ",
            "fun (left_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul a (add b c)) (add (mul a b) (mul a c))) => ",
            "fun (right_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (add a b) c) (add (mul a c) (mul b c))) => ",
            "fun (mul_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a zero) zero) => ",
            "fun (zero_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul zero a) zero) => ",
            "fun (add_left_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add a b) (add a c)), @Eq.{u} Scalar b c) => ",
            "fun (ring_normalize_add_mul3_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add (mul a b) (mul b c)) (mul a c)) (add (add (mul a b) (mul a c)) (mul b c))) => ",
            "fun (add_right_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add b a) (add c a)), @Eq.{u} Scalar b c) => ",
            "fun (neg_neg_arg : forall (a : Scalar), @Eq.{u} Scalar (neg (neg a)) a) => ",
            "fun (sub_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a zero) a) => ",
            "fun (zero_sub_arg : forall (a : Scalar), @Eq.{u} Scalar (sub zero a) (neg a)) => ",
            "fun (sub_add_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add (sub a b) b) a) => ",
            "fun (add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub (add a b) b) a) => ",
            "fun (sub_add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (sub (sub a c) (sub b c)) (sub a b)) => ",
            "@eq_symm.{u} Scalar ",
            "(sub (add (add nx (mul (@two.{u} Scalar one add) d)) ny) (add nx ny)) ",
            "(mul (@two.{u} Scalar one add) d) ",
            "(@eq_trans.{u} Scalar ",
            "(sub (add (add nx (mul (@two.{u} Scalar one add) d)) ny) (add nx ny)) ",
            "(sub (add (mul (@two.{u} Scalar one add) d) (add nx ny)) (add nx ny)) ",
            "(mul (@two.{u} Scalar one add) d) ",
            "(@eq_congr2.{u,u,u} Scalar Scalar Scalar sub ",
            "(add (add nx (mul (@two.{u} Scalar one add) d)) ny) ",
            "(add (mul (@two.{u} Scalar one add) d) (add nx ny)) ",
            "(add nx ny) ",
            "(add nx ny) ",
            "(@add_middle_to_front_from_ring_args.{u} Scalar zero one add neg sub mul ring_args nx (mul (@two.{u} Scalar one add) d) ny) ",
            "(@Eq.refl.{u} Scalar (add nx ny))) ",
            "(add_sub_cancel_arg (mul (@two.{u} Scalar one add) d) (add nx ny))))"
        )),
    },
];

const ABSTRACT_VECTOR_SPACE_DEFINITIONS: &[DefinitionArtifact] = &[
    DefinitionArtifact {
        name: "vsub",
        universe_params: &["v"],
        ty: concat!(
            "forall (Vector : Sort v), ",
            "forall (vadd : forall (x : Vector), forall (y : Vector), Vector), ",
            "forall (vneg : forall (x : Vector), Vector), ",
            "forall (x : Vector), forall (y : Vector), Vector"
        ),
        value: "fun Vector => fun vadd => fun vneg => fun x => fun y => vadd x (vneg y)",
    },
    DefinitionArtifact {
        name: "linear_comb2",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (Scalar : Sort u), ",
            "forall (Vector : Sort v), ",
            "forall (vadd : forall (x : Vector), forall (y : Vector), Vector), ",
            "forall (smul : forall (a : Scalar), forall (x : Vector), Vector), ",
            "forall (a : Scalar), forall (x : Vector), ",
            "forall (b : Scalar), forall (y : Vector), Vector"
        ),
        value:
            "fun Scalar => fun Vector => fun vadd => fun smul => fun a => fun x => fun b => fun y => vadd (smul a x) (smul b y)",
    },
    DefinitionArtifact {
        name: "linear_comb3",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (Scalar : Sort u), ",
            "forall (Vector : Sort v), ",
            "forall (vadd : forall (x : Vector), forall (y : Vector), Vector), ",
            "forall (smul : forall (a : Scalar), forall (x : Vector), Vector), ",
            "forall (a : Scalar), forall (x : Vector), ",
            "forall (b : Scalar), forall (y : Vector), ",
            "forall (c : Scalar), forall (z : Vector), Vector"
        ),
        value:
            "fun Scalar => fun Vector => fun vadd => fun smul => fun a => fun x => fun b => fun y => fun c => fun z => vadd (vadd (smul a x) (smul b y)) (smul c z)",
    },
    DefinitionArtifact {
        name: "VectorSpaceLawArgs",
        universe_params: &["u", "v"],
        ty: abstract_vector_space_params!("Prop"),
        value: abstract_vector_space_abs!(concat!(
            "forall (P : Prop), forall (mk : ",
            "forall (vec_sub_def_law : forall (x : Vector), forall (y : Vector), @Eq.{v} Vector (@vsub.{v} Vector vadd vneg x y) (vadd x (vneg y))), ",
            "forall (vec_add_assoc_law : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{v} Vector (vadd (vadd x y) z) (vadd x (vadd y z))), ",
            "forall (vec_add_comm_law : forall (x : Vector), forall (y : Vector), @Eq.{v} Vector (vadd x y) (vadd y x)), ",
            "forall (vec_add_zero_law : forall (x : Vector), @Eq.{v} Vector (vadd x vzero) x), ",
            "forall (vec_zero_add_law : forall (x : Vector), @Eq.{v} Vector (vadd vzero x) x), ",
            "forall (vec_neg_add_cancel_law : forall (x : Vector), @Eq.{v} Vector (vadd (vneg x) x) vzero), ",
            "forall (vec_add_neg_cancel_law : forall (x : Vector), @Eq.{v} Vector (vadd x (vneg x)) vzero), ",
            "forall (sub_sub_sub_cancel_law : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{v} Vector (@vsub.{v} Vector vadd vneg (@vsub.{v} Vector vadd vneg x z) (@vsub.{v} Vector vadd vneg y z)) (@vsub.{v} Vector vadd vneg x y)), ",
            "forall (vec_sub_self_law : forall (x : Vector), @Eq.{v} Vector (@vsub.{v} Vector vadd vneg x x) vzero), ",
            "forall (vec_sub_zero_law : forall (x : Vector), @Eq.{v} Vector (@vsub.{v} Vector vadd vneg x vzero) x), ",
            "forall (vec_add_left_cancel_law : forall (x : Vector), forall (y : Vector), forall (z : Vector), forall (h : @Eq.{v} Vector (vadd x y) (vadd x z)), @Eq.{v} Vector y z), ",
            "forall (smul_add_law : forall (a : Scalar), forall (b : Scalar), forall (x : Vector), @Eq.{v} Vector (smul (add a b) x) (vadd (smul a x) (smul b x))), ",
            "forall (add_smul_law : forall (a : Scalar), forall (x : Vector), forall (y : Vector), @Eq.{v} Vector (smul a (vadd x y)) (vadd (smul a x) (smul a y))), ",
            "forall (one_smul_law : forall (x : Vector), @Eq.{v} Vector (smul one x) x), ",
            "forall (mul_smul_law : forall (a : Scalar), forall (b : Scalar), forall (x : Vector), @Eq.{v} Vector (smul (mul a b) x) (smul a (smul b x))), ",
            "forall (zero_smul_law : forall (x : Vector), @Eq.{v} Vector (smul zero x) vzero), ",
            "forall (smul_zero_law : forall (a : Scalar), @Eq.{v} Vector (smul a vzero) vzero), ",
            "forall (neg_smul_law : forall (a : Scalar), forall (x : Vector), @Eq.{v} Vector (smul (neg a) x) (vneg (smul a x))), ",
            "forall (smul_neg_law : forall (a : Scalar), forall (x : Vector), @Eq.{v} Vector (smul a (vneg x)) (vneg (smul a x))), ",
            "forall (vec_sub_eq_add_neg_law : forall (x : Vector), forall (y : Vector), @Eq.{v} Vector (@vsub.{v} Vector vadd vneg x y) (vadd x (vneg y))), ",
            "forall (sub_add_sub_cancel_left_law : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{v} Vector (vadd (@vsub.{v} Vector vadd vneg x z) (@vsub.{v} Vector vadd vneg z y)) (@vsub.{v} Vector vadd vneg x y)), ",
            "forall (linear_comb2_ext_law : forall (a : Scalar), forall (x : Vector), forall (b : Scalar), forall (y : Vector), @Eq.{v} Vector (@linear_comb2.{u,v} Scalar Vector vadd smul a x b y) (vadd (smul a x) (smul b y))), ",
            "forall (linear_comb3_ext_law : forall (a : Scalar), forall (x : Vector), forall (b : Scalar), forall (y : Vector), forall (c : Scalar), forall (z : Vector), @Eq.{v} Vector (@linear_comb3.{u,v} Scalar Vector vadd smul a x b y c z) (vadd (vadd (smul a x) (smul b y)) (smul c z))), P), P"
        )),
    },
];

const ABSTRACT_VECTOR_SPACE_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "vec_sub_def",
        universe_params: &["u", "v"],
        statement: abstract_vector_space_params!(
            "forall (x : Vector), forall (y : Vector), @Eq.{v} Vector (@vsub.{v} Vector vadd vneg x y) (vadd x (vneg y))"
        ),
        proof: abstract_vector_space_abs!(
            "fun x => fun y => @Eq.refl.{v} Vector (@vsub.{v} Vector vadd vneg x y)"
        ),
    },
    TheoremArtifact {
        name: "vec_add_assoc",
        universe_params: &["u", "v"],
        statement: abstract_vector_space_params!(
            "forall (law : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{v} Vector (vadd (vadd x y) z) (vadd x (vadd y z))), forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{v} Vector (vadd (vadd x y) z) (vadd x (vadd y z))"
        ),
        proof: abstract_vector_space_abs!("fun law => fun x => fun y => fun z => law x y z"),
    },
    TheoremArtifact {
        name: "vec_add_comm",
        universe_params: &["u", "v"],
        statement: abstract_vector_space_params!(
            "forall (law : forall (x : Vector), forall (y : Vector), @Eq.{v} Vector (vadd x y) (vadd y x)), forall (x : Vector), forall (y : Vector), @Eq.{v} Vector (vadd x y) (vadd y x)"
        ),
        proof: abstract_vector_space_abs!("fun law => fun x => fun y => law x y"),
    },
    TheoremArtifact {
        name: "vec_add_zero",
        universe_params: &["u", "v"],
        statement: abstract_vector_space_params!(
            "forall (law : forall (x : Vector), @Eq.{v} Vector (vadd x vzero) x), forall (x : Vector), @Eq.{v} Vector (vadd x vzero) x"
        ),
        proof: abstract_vector_space_abs!("fun law => fun x => law x"),
    },
    TheoremArtifact {
        name: "vec_zero_add",
        universe_params: &["u", "v"],
        statement: abstract_vector_space_params!(
            "forall (law : forall (x : Vector), @Eq.{v} Vector (vadd vzero x) x), forall (x : Vector), @Eq.{v} Vector (vadd vzero x) x"
        ),
        proof: abstract_vector_space_abs!("fun law => fun x => law x"),
    },
    TheoremArtifact {
        name: "vec_neg_add_cancel",
        universe_params: &["u", "v"],
        statement: abstract_vector_space_params!(
            "forall (law : forall (x : Vector), @Eq.{v} Vector (vadd (vneg x) x) vzero), forall (x : Vector), @Eq.{v} Vector (vadd (vneg x) x) vzero"
        ),
        proof: abstract_vector_space_abs!("fun law => fun x => law x"),
    },
    TheoremArtifact {
        name: "vec_add_neg_cancel",
        universe_params: &["u", "v"],
        statement: abstract_vector_space_params!(
            "forall (law : forall (x : Vector), @Eq.{v} Vector (vadd x (vneg x)) vzero), forall (x : Vector), @Eq.{v} Vector (vadd x (vneg x)) vzero"
        ),
        proof: abstract_vector_space_abs!("fun law => fun x => law x"),
    },
    TheoremArtifact {
        name: "sub_sub_sub_cancel",
        universe_params: &["u", "v"],
        statement: abstract_vector_space_params!(
            "forall (law : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{v} Vector (@vsub.{v} Vector vadd vneg (@vsub.{v} Vector vadd vneg x z) (@vsub.{v} Vector vadd vneg y z)) (@vsub.{v} Vector vadd vneg x y)), forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{v} Vector (@vsub.{v} Vector vadd vneg (@vsub.{v} Vector vadd vneg x z) (@vsub.{v} Vector vadd vneg y z)) (@vsub.{v} Vector vadd vneg x y)"
        ),
        proof: abstract_vector_space_abs!("fun law => fun x => fun y => fun z => law x y z"),
    },
    TheoremArtifact {
        name: "vec_sub_self",
        universe_params: &["u", "v"],
        statement: abstract_vector_space_params!(
            "forall (law : forall (x : Vector), @Eq.{v} Vector (@vsub.{v} Vector vadd vneg x x) vzero), forall (x : Vector), @Eq.{v} Vector (@vsub.{v} Vector vadd vneg x x) vzero"
        ),
        proof: abstract_vector_space_abs!("fun law => fun x => law x"),
    },
    TheoremArtifact {
        name: "vec_sub_zero",
        universe_params: &["u", "v"],
        statement: abstract_vector_space_params!(
            "forall (law : forall (x : Vector), @Eq.{v} Vector (@vsub.{v} Vector vadd vneg x vzero) x), forall (x : Vector), @Eq.{v} Vector (@vsub.{v} Vector vadd vneg x vzero) x"
        ),
        proof: abstract_vector_space_abs!("fun law => fun x => law x"),
    },
    TheoremArtifact {
        name: "vec_add_left_cancel",
        universe_params: &["u", "v"],
        statement: abstract_vector_space_params!(
            "forall (law : forall (x : Vector), forall (y : Vector), forall (z : Vector), forall (h : @Eq.{v} Vector (vadd x y) (vadd x z)), @Eq.{v} Vector y z), forall (x : Vector), forall (y : Vector), forall (z : Vector), forall (h : @Eq.{v} Vector (vadd x y) (vadd x z)), @Eq.{v} Vector y z"
        ),
        proof: abstract_vector_space_abs!(
            "fun law => fun x => fun y => fun z => fun h => law x y z h"
        ),
    },
    TheoremArtifact {
        name: "smul_add",
        universe_params: &["u", "v"],
        statement: abstract_vector_space_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), forall (x : Vector), @Eq.{v} Vector (smul (add a b) x) (vadd (smul a x) (smul b x))), forall (a : Scalar), forall (b : Scalar), forall (x : Vector), @Eq.{v} Vector (smul (add a b) x) (vadd (smul a x) (smul b x))"
        ),
        proof: abstract_vector_space_abs!("fun law => fun a => fun b => fun x => law a b x"),
    },
    TheoremArtifact {
        name: "add_smul",
        universe_params: &["u", "v"],
        statement: abstract_vector_space_params!(
            "forall (law : forall (a : Scalar), forall (x : Vector), forall (y : Vector), @Eq.{v} Vector (smul a (vadd x y)) (vadd (smul a x) (smul a y))), forall (a : Scalar), forall (x : Vector), forall (y : Vector), @Eq.{v} Vector (smul a (vadd x y)) (vadd (smul a x) (smul a y))"
        ),
        proof: abstract_vector_space_abs!("fun law => fun a => fun x => fun y => law a x y"),
    },
    TheoremArtifact {
        name: "one_smul",
        universe_params: &["u", "v"],
        statement: abstract_vector_space_params!(
            "forall (law : forall (x : Vector), @Eq.{v} Vector (smul one x) x), forall (x : Vector), @Eq.{v} Vector (smul one x) x"
        ),
        proof: abstract_vector_space_abs!("fun law => fun x => law x"),
    },
    TheoremArtifact {
        name: "mul_smul",
        universe_params: &["u", "v"],
        statement: abstract_vector_space_params!(
            "forall (law : forall (a : Scalar), forall (b : Scalar), forall (x : Vector), @Eq.{v} Vector (smul (mul a b) x) (smul a (smul b x))), forall (a : Scalar), forall (b : Scalar), forall (x : Vector), @Eq.{v} Vector (smul (mul a b) x) (smul a (smul b x))"
        ),
        proof: abstract_vector_space_abs!("fun law => fun a => fun b => fun x => law a b x"),
    },
    TheoremArtifact {
        name: "zero_smul",
        universe_params: &["u", "v"],
        statement: abstract_vector_space_params!(
            "forall (law : forall (x : Vector), @Eq.{v} Vector (smul zero x) vzero), forall (x : Vector), @Eq.{v} Vector (smul zero x) vzero"
        ),
        proof: abstract_vector_space_abs!("fun law => fun x => law x"),
    },
    TheoremArtifact {
        name: "smul_zero",
        universe_params: &["u", "v"],
        statement: abstract_vector_space_params!(
            "forall (law : forall (a : Scalar), @Eq.{v} Vector (smul a vzero) vzero), forall (a : Scalar), @Eq.{v} Vector (smul a vzero) vzero"
        ),
        proof: abstract_vector_space_abs!("fun law => fun a => law a"),
    },
    TheoremArtifact {
        name: "neg_smul",
        universe_params: &["u", "v"],
        statement: abstract_vector_space_params!(
            "forall (law : forall (a : Scalar), forall (x : Vector), @Eq.{v} Vector (smul (neg a) x) (vneg (smul a x))), forall (a : Scalar), forall (x : Vector), @Eq.{v} Vector (smul (neg a) x) (vneg (smul a x))"
        ),
        proof: abstract_vector_space_abs!("fun law => fun a => fun x => law a x"),
    },
    TheoremArtifact {
        name: "smul_neg",
        universe_params: &["u", "v"],
        statement: abstract_vector_space_params!(
            "forall (law : forall (a : Scalar), forall (x : Vector), @Eq.{v} Vector (smul a (vneg x)) (vneg (smul a x))), forall (a : Scalar), forall (x : Vector), @Eq.{v} Vector (smul a (vneg x)) (vneg (smul a x))"
        ),
        proof: abstract_vector_space_abs!("fun law => fun a => fun x => law a x"),
    },
    TheoremArtifact {
        name: "vec_sub_eq_add_neg",
        universe_params: &["u", "v"],
        statement: abstract_vector_space_params!(
            "forall (x : Vector), forall (y : Vector), @Eq.{v} Vector (@vsub.{v} Vector vadd vneg x y) (vadd x (vneg y))"
        ),
        proof: abstract_vector_space_abs!(
            "fun x => fun y => @Eq.refl.{v} Vector (@vsub.{v} Vector vadd vneg x y)"
        ),
    },
    TheoremArtifact {
        name: "sub_add_sub_cancel_left",
        universe_params: &["u", "v"],
        statement: abstract_vector_space_params!(
            "forall (law : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{v} Vector (vadd (@vsub.{v} Vector vadd vneg x z) (@vsub.{v} Vector vadd vneg z y)) (@vsub.{v} Vector vadd vneg x y)), forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{v} Vector (vadd (@vsub.{v} Vector vadd vneg x z) (@vsub.{v} Vector vadd vneg z y)) (@vsub.{v} Vector vadd vneg x y)"
        ),
        proof: abstract_vector_space_abs!("fun law => fun x => fun y => fun z => law x y z"),
    },
    TheoremArtifact {
        name: "linear_comb2_ext",
        universe_params: &["u", "v"],
        statement: abstract_vector_space_params!(
            "forall (a : Scalar), forall (x : Vector), forall (b : Scalar), forall (y : Vector), @Eq.{v} Vector (@linear_comb2.{u,v} Scalar Vector vadd smul a x b y) (vadd (smul a x) (smul b y))"
        ),
        proof: abstract_vector_space_abs!(
            "fun a => fun x => fun b => fun y => @Eq.refl.{v} Vector (@linear_comb2.{u,v} Scalar Vector vadd smul a x b y)"
        ),
    },
    TheoremArtifact {
        name: "linear_comb3_ext",
        universe_params: &["u", "v"],
        statement: abstract_vector_space_params!(
            "forall (a : Scalar), forall (x : Vector), forall (b : Scalar), forall (y : Vector), forall (c : Scalar), forall (z : Vector), @Eq.{v} Vector (@linear_comb3.{u,v} Scalar Vector vadd smul a x b y c z) (vadd (vadd (smul a x) (smul b y)) (smul c z))"
        ),
        proof: abstract_vector_space_abs!(
            "fun a => fun x => fun b => fun y => fun c => fun z => @Eq.refl.{v} Vector (@linear_comb3.{u,v} Scalar Vector vadd smul a x b y c z)"
        ),
    },
];

const ABSTRACT_INNER_PRODUCT_DEFINITIONS: &[DefinitionArtifact] = &[
    DefinitionArtifact {
        name: "dot",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (Scalar : Sort u), ",
            "forall (Vector : Sort v), ",
            "forall (inner : forall (x : Vector), forall (y : Vector), Scalar), ",
            "forall (x : Vector), forall (y : Vector), Scalar"
        ),
        value: "fun Scalar => fun Vector => fun inner => fun x => fun y => inner x y",
    },
    DefinitionArtifact {
        name: "normSq",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (Scalar : Sort u), ",
            "forall (Vector : Sort v), ",
            "forall (inner : forall (x : Vector), forall (y : Vector), Scalar), ",
            "forall (x : Vector), Scalar"
        ),
        value: "fun Scalar => fun Vector => fun inner => fun x => @dot.{u,v} Scalar Vector inner x x",
    },
    DefinitionArtifact {
        name: "distSq",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (Scalar : Sort u), ",
            "forall (Vector : Sort v), ",
            "forall (vadd : forall (x : Vector), forall (y : Vector), Vector), ",
            "forall (vneg : forall (x : Vector), Vector), ",
            "forall (inner : forall (x : Vector), forall (y : Vector), Scalar), ",
            "forall (x : Vector), forall (y : Vector), Scalar"
        ),
        value:
            "fun Scalar => fun Vector => fun vadd => fun vneg => fun inner => fun x => fun y => @normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg y x)",
    },
    DefinitionArtifact {
        name: "PerpVec",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (Scalar : Sort u), ",
            "forall (zero : Scalar), ",
            "forall (Vector : Sort v), ",
            "forall (inner : forall (x : Vector), forall (y : Vector), Scalar), ",
            "forall (x : Vector), forall (y : Vector), Prop"
        ),
        value:
            "fun Scalar => fun zero => fun Vector => fun inner => fun x => fun y => @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero",
    },
    DefinitionArtifact {
        name: "InnerProductLawArgs",
        universe_params: &["u", "v"],
        ty: abstract_inner_product_params!("Prop"),
        value: abstract_inner_product_abs!(concat!(
            "forall (P : Prop), forall (mk : ",
            "forall (dot_comm_law : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner y x)), ",
            "forall (dot_add_left_law : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vadd x y) z) (add (@dot.{u,v} Scalar Vector inner x z) (@dot.{u,v} Scalar Vector inner y z))), ",
            "forall (dot_add_right_law : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (vadd y z)) (add (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner x z))), ",
            "forall (dot_neg_left_law : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vneg x) y) (neg (@dot.{u,v} Scalar Vector inner x y))), ",
            "forall (dot_neg_right_law : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (vneg y)) (neg (@dot.{u,v} Scalar Vector inner x y))), ",
            "forall (dot_sub_left_law : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y) z) (sub (@dot.{u,v} Scalar Vector inner x z) (@dot.{u,v} Scalar Vector inner y z))), ",
            "forall (dot_sub_right_law : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (@vsub.{v} Vector vadd vneg y z)) (sub (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner x z))), ",
            "forall (norm_sq_def_law : forall (x : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) (@dot.{u,v} Scalar Vector inner x x)), ",
            "forall (dist_sq_def_law : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@distSq.{u,v} Scalar Vector vadd vneg inner x y) (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg y x))), ",
            "forall (dot_self_eq_norm_sq_law : forall (x : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x x) (@normSq.{u,v} Scalar Vector inner x)), ",
            "forall (norm_sq_add_law : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (add (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y))), ",
            "forall (norm_sq_sub_law : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (sub (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y))), ",
            "forall (norm_sq_add_of_dot_zero_law : forall (x : Vector), forall (y : Vector), forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))), ",
            "forall (norm_sq_sub_of_dot_zero_law : forall (x : Vector), forall (y : Vector), forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))), ",
            "forall (norm_sq_nonneg_law : forall (x : Vector), le_rel zero (@normSq.{u,v} Scalar Vector inner x)), ",
            "forall (parallelogram_law_law : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (add (@normSq.{u,v} Scalar Vector inner (vadd x y)) (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y))) (add (mul (@two.{u} Scalar one add) (@normSq.{u,v} Scalar Vector inner x)) (mul (@two.{u} Scalar one add) (@normSq.{u,v} Scalar Vector inner y)))), ",
            "forall (polarization_identity_law : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y)) (sub (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)))), ",
            "forall (perp_vec_iff_dot_eq_zero_law : forall (x : Vector), forall (y : Vector), forall (R : Prop), forall (mk : forall (forward : forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), forall (backward : forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @PerpVec.{u,v} Scalar zero Vector inner x y), R), R), ",
            "forall (perp_vec_symm_law : forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @PerpVec.{u,v} Scalar zero Vector inner y x), ",
            "forall (norm_sq_zero_iff_law : forall (x : Vector), forall (R : Prop), forall (mk : forall (forward : forall (h : @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) zero), @Eq.{v} Vector x vzero), forall (backward : forall (h : @Eq.{v} Vector x vzero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) zero), R), R), ",
            "forall (dist_sq_nonneg_law : forall (x : Vector), forall (y : Vector), le_rel zero (@distSq.{u,v} Scalar Vector vadd vneg inner x y)), ",
            "forall (norm_sq_add_of_perp_law : forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))), ",
            "forall (norm_sq_sub_of_perp_law : forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))), ",
            "forall (quadratic_norm_nonneg_law : forall (x : Vector), forall (y : Vector), forall (t : Scalar), le_rel zero (add (add (mul (@normSq.{u,v} Scalar Vector inner x) (@sq.{u} Scalar mul t)) (mul (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y)) t)) (@normSq.{u,v} Scalar Vector inner y))), P), P"
        )),
    },
];

const ABSTRACT_INNER_PRODUCT_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "dot_comm",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (law : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner y x)), forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner y x)"
        ),
        proof: abstract_inner_product_abs!("fun law => fun x => fun y => law x y"),
    },
    TheoremArtifact {
        name: "dot_add_left",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (law : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vadd x y) z) (add (@dot.{u,v} Scalar Vector inner x z) (@dot.{u,v} Scalar Vector inner y z))), forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vadd x y) z) (add (@dot.{u,v} Scalar Vector inner x z) (@dot.{u,v} Scalar Vector inner y z))"
        ),
        proof: abstract_inner_product_abs!("fun law => fun x => fun y => fun z => law x y z"),
    },
    TheoremArtifact {
        name: "dot_add_right",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (law : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (vadd y z)) (add (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner x z))), forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (vadd y z)) (add (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner x z))"
        ),
        proof: abstract_inner_product_abs!("fun law => fun x => fun y => fun z => law x y z"),
    },
    TheoremArtifact {
        name: "dot_neg_left",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (law : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vneg x) y) (neg (@dot.{u,v} Scalar Vector inner x y))), forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vneg x) y) (neg (@dot.{u,v} Scalar Vector inner x y))"
        ),
        proof: abstract_inner_product_abs!("fun law => fun x => fun y => law x y"),
    },
    TheoremArtifact {
        name: "dot_neg_right",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (law : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (vneg y)) (neg (@dot.{u,v} Scalar Vector inner x y))), forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (vneg y)) (neg (@dot.{u,v} Scalar Vector inner x y))"
        ),
        proof: abstract_inner_product_abs!("fun law => fun x => fun y => law x y"),
    },
    TheoremArtifact {
        name: "dot_sub_left",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (law : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y) z) (sub (@dot.{u,v} Scalar Vector inner x z) (@dot.{u,v} Scalar Vector inner y z))), forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y) z) (sub (@dot.{u,v} Scalar Vector inner x z) (@dot.{u,v} Scalar Vector inner y z))"
        ),
        proof: abstract_inner_product_abs!("fun law => fun x => fun y => fun z => law x y z"),
    },
    TheoremArtifact {
        name: "dot_sub_right",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (law : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (@vsub.{v} Vector vadd vneg y z)) (sub (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner x z))), forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (@vsub.{v} Vector vadd vneg y z)) (sub (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner x z))"
        ),
        proof: abstract_inner_product_abs!("fun law => fun x => fun y => fun z => law x y z"),
    },
    TheoremArtifact {
        name: "norm_sq_def",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (x : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) (@dot.{u,v} Scalar Vector inner x x)"
        ),
        proof: abstract_inner_product_abs!(
            "fun x => @Eq.refl.{u} Scalar (@normSq.{u,v} Scalar Vector inner x)"
        ),
    },
    TheoremArtifact {
        name: "dist_sq_def",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@distSq.{u,v} Scalar Vector vadd vneg inner x y) (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg y x))"
        ),
        proof: abstract_inner_product_abs!(
            "fun x => fun y => @Eq.refl.{u} Scalar (@distSq.{u,v} Scalar Vector vadd vneg inner x y)"
        ),
    },
    TheoremArtifact {
        name: "dot_self_eq_norm_sq",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (x : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x x) (@normSq.{u,v} Scalar Vector inner x)"
        ),
        proof: abstract_inner_product_abs!(
            "fun x => @Eq.refl.{u} Scalar (@dot.{u,v} Scalar Vector inner x x)"
        ),
    },
    TheoremArtifact {
        name: "norm_sq_add",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (law : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (add (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y))), forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (add (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y))"
        ),
        proof: abstract_inner_product_abs!("fun law => fun x => fun y => law x y"),
    },
    TheoremArtifact {
        name: "norm_sq_sub",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (law : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (sub (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y))), forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (sub (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y))"
        ),
        proof: abstract_inner_product_abs!("fun law => fun x => fun y => law x y"),
    },
    TheoremArtifact {
        name: "norm_sq_add_of_dot_zero",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (law : forall (x : Vector), forall (y : Vector), forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))), forall (x : Vector), forall (y : Vector), forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))"
        ),
        proof: abstract_inner_product_abs!("fun law => fun x => fun y => fun h => law x y h"),
    },
    TheoremArtifact {
        name: "norm_sq_sub_of_dot_zero",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (law : forall (x : Vector), forall (y : Vector), forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))), forall (x : Vector), forall (y : Vector), forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))"
        ),
        proof: abstract_inner_product_abs!("fun law => fun x => fun y => fun h => law x y h"),
    },
    TheoremArtifact {
        name: "norm_sq_nonneg",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (law : forall (x : Vector), le_rel zero (@normSq.{u,v} Scalar Vector inner x)), forall (x : Vector), le_rel zero (@normSq.{u,v} Scalar Vector inner x)"
        ),
        proof: abstract_inner_product_abs!("fun law => fun x => law x"),
    },
    TheoremArtifact {
        name: "parallelogram_law",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (law : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (add (@normSq.{u,v} Scalar Vector inner (vadd x y)) (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y))) (add (mul (@two.{u} Scalar one add) (@normSq.{u,v} Scalar Vector inner x)) (mul (@two.{u} Scalar one add) (@normSq.{u,v} Scalar Vector inner y)))), forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (add (@normSq.{u,v} Scalar Vector inner (vadd x y)) (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y))) (add (mul (@two.{u} Scalar one add) (@normSq.{u,v} Scalar Vector inner x)) (mul (@two.{u} Scalar one add) (@normSq.{u,v} Scalar Vector inner y)))"
        ),
        proof: abstract_inner_product_abs!("fun law => fun x => fun y => law x y"),
    },
    TheoremArtifact {
        name: "polarization_identity",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (law : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y)) (sub (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)))), forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y)) (sub (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)))"
        ),
        proof: abstract_inner_product_abs!("fun law => fun x => fun y => law x y"),
    },
    TheoremArtifact {
        name: "cauchy_schwarz",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (law : forall (x : Vector), forall (y : Vector), le_rel (@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y)) (mul (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))), forall (x : Vector), forall (y : Vector), le_rel (@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y)) (mul (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))"
        ),
        proof: abstract_inner_product_abs!("fun law => fun x => fun y => law x y"),
    },
    TheoremArtifact {
        name: "perp_vec_iff_dot_eq_zero",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (x : Vector), forall (y : Vector), forall (R : Prop), forall (mk : forall (forward : forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), forall (backward : forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @PerpVec.{u,v} Scalar zero Vector inner x y), R), R"
        ),
        proof: abstract_inner_product_abs!(
            "fun x => fun y => fun (R : Prop) => fun (mk : forall (forward : forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), forall (backward : forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @PerpVec.{u,v} Scalar zero Vector inner x y), R) => mk (fun (h : @PerpVec.{u,v} Scalar zero Vector inner x y) => h) (fun (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero) => h)"
        ),
    },
    TheoremArtifact {
        name: "perp_vec_symm",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (law : forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @PerpVec.{u,v} Scalar zero Vector inner y x), forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @PerpVec.{u,v} Scalar zero Vector inner y x"
        ),
        proof: abstract_inner_product_abs!("fun law => fun x => fun y => fun h => law x y h"),
    },
    TheoremArtifact {
        name: "norm_sq_zero_iff",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (law : forall (x : Vector), forall (R : Prop), forall (mk : forall (forward : forall (h : @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) zero), @Eq.{v} Vector x vzero), forall (backward : forall (h : @Eq.{v} Vector x vzero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) zero), R), R), forall (x : Vector), forall (R : Prop), forall (mk : forall (forward : forall (h : @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) zero), @Eq.{v} Vector x vzero), forall (backward : forall (h : @Eq.{v} Vector x vzero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) zero), R), R"
        ),
        proof: abstract_inner_product_abs!(
            "fun law => fun x => fun (R : Prop) => fun (mk : forall (forward : forall (h : @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) zero), @Eq.{v} Vector x vzero), forall (backward : forall (h : @Eq.{v} Vector x vzero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) zero), R) => law x R mk"
        ),
    },
    TheoremArtifact {
        name: "dist_sq_nonneg",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (law : forall (x : Vector), forall (y : Vector), le_rel zero (@distSq.{u,v} Scalar Vector vadd vneg inner x y)), forall (x : Vector), forall (y : Vector), le_rel zero (@distSq.{u,v} Scalar Vector vadd vneg inner x y)"
        ),
        proof: abstract_inner_product_abs!("fun law => fun x => fun y => law x y"),
    },
    TheoremArtifact {
        name: "norm_sq_add_of_perp",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (law : forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))), forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))"
        ),
        proof: abstract_inner_product_abs!("fun law => fun x => fun y => fun h => law x y h"),
    },
    TheoremArtifact {
        name: "norm_sq_sub_of_perp",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (law : forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))), forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))"
        ),
        proof: abstract_inner_product_abs!("fun law => fun x => fun y => fun h => law x y h"),
    },
    TheoremArtifact {
        name: "quadratic_norm_nonneg",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (law : forall (x : Vector), forall (y : Vector), forall (t : Scalar), le_rel zero (add (add (mul (@normSq.{u,v} Scalar Vector inner x) (@sq.{u} Scalar mul t)) (mul (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y)) t)) (@normSq.{u,v} Scalar Vector inner y))), forall (x : Vector), forall (y : Vector), forall (t : Scalar), le_rel zero (add (add (mul (@normSq.{u,v} Scalar Vector inner x) (@sq.{u} Scalar mul t)) (mul (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y)) t)) (@normSq.{u,v} Scalar Vector inner y))"
        ),
        proof: abstract_inner_product_abs!("fun law => fun x => fun y => fun t => law x y t"),
    },
];

const ABSTRACT_INNER_PRODUCT_DERIVE_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "norm_sq_add_from_inner_args",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (add (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y))"
        ),
        proof: abstract_inner_product_abs!(concat!(
            "fun inner_args => fun x => fun y => ",
            "inner_args (@Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (add (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y))) ",
            "(fun (dot_comm_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner y x)) => ",
            "fun (dot_add_left_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vadd x y) z) (add (@dot.{u,v} Scalar Vector inner x z) (@dot.{u,v} Scalar Vector inner y z))) => ",
            "fun (dot_add_right_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (vadd y z)) (add (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner x z))) => ",
            "fun (dot_neg_left_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vneg x) y) (neg (@dot.{u,v} Scalar Vector inner x y))) => ",
            "fun (dot_neg_right_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (vneg y)) (neg (@dot.{u,v} Scalar Vector inner x y))) => ",
            "fun (dot_sub_left_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y) z) (sub (@dot.{u,v} Scalar Vector inner x z) (@dot.{u,v} Scalar Vector inner y z))) => ",
            "fun (dot_sub_right_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (@vsub.{v} Vector vadd vneg y z)) (sub (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner x z))) => ",
            "fun (norm_sq_def_arg : forall (x : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) (@dot.{u,v} Scalar Vector inner x x)) => ",
            "fun (dist_sq_def_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@distSq.{u,v} Scalar Vector vadd vneg inner x y) (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg y x))) => ",
            "fun (dot_self_eq_norm_sq_arg : forall (x : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x x) (@normSq.{u,v} Scalar Vector inner x)) => ",
            "fun (norm_sq_add_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (add (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (norm_sq_sub_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (sub (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (inner_field13_arg : forall (x : Vector), forall (y : Vector), forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (inner_field14_arg : forall (x : Vector), forall (y : Vector), forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (norm_sq_nonneg_arg : forall (x : Vector), le_rel zero (@normSq.{u,v} Scalar Vector inner x)) => ",
            "fun (parallelogram_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (add (@normSq.{u,v} Scalar Vector inner (vadd x y)) (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y))) (add (mul (@two.{u} Scalar one add) (@normSq.{u,v} Scalar Vector inner x)) (mul (@two.{u} Scalar one add) (@normSq.{u,v} Scalar Vector inner y)))) => ",
            "fun (polarization_identity_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y)) (sub (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)))) => ",
            "fun (perp_vec_iff_dot_eq_zero_arg : forall (x : Vector), forall (y : Vector), forall (R : Prop), forall (mk : forall (forward : forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), forall (backward : forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @PerpVec.{u,v} Scalar zero Vector inner x y), R), R) => ",
            "fun (perp_vec_symm_arg : forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @PerpVec.{u,v} Scalar zero Vector inner y x) => ",
            "fun (norm_sq_zero_iff_arg : forall (x : Vector), forall (R : Prop), forall (mk : forall (forward : forall (h : @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) zero), @Eq.{v} Vector x vzero), forall (backward : forall (h : @Eq.{v} Vector x vzero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) zero), R), R) => ",
            "fun (dist_sq_nonneg_arg : forall (x : Vector), forall (y : Vector), le_rel zero (@distSq.{u,v} Scalar Vector vadd vneg inner x y)) => ",
            "fun (inner_field23_arg : forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (inner_field24_arg : forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (quadratic_norm_nonneg_arg : forall (x : Vector), forall (y : Vector), forall (t : Scalar), le_rel zero (add (add (mul (@normSq.{u,v} Scalar Vector inner x) (@sq.{u} Scalar mul t)) (mul (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y)) t)) (@normSq.{u,v} Scalar Vector inner y))) => norm_sq_add_arg x y)"
        )),
    },
    TheoremArtifact {
        name: "norm_sq_sub_from_inner_args",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (sub (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y))"
        ),
        proof: abstract_inner_product_abs!(concat!(
            "fun inner_args => fun x => fun y => ",
            "inner_args (@Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (sub (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y))) ",
            "(fun (dot_comm_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner y x)) => ",
            "fun (dot_add_left_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vadd x y) z) (add (@dot.{u,v} Scalar Vector inner x z) (@dot.{u,v} Scalar Vector inner y z))) => ",
            "fun (dot_add_right_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (vadd y z)) (add (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner x z))) => ",
            "fun (dot_neg_left_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vneg x) y) (neg (@dot.{u,v} Scalar Vector inner x y))) => ",
            "fun (dot_neg_right_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (vneg y)) (neg (@dot.{u,v} Scalar Vector inner x y))) => ",
            "fun (dot_sub_left_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y) z) (sub (@dot.{u,v} Scalar Vector inner x z) (@dot.{u,v} Scalar Vector inner y z))) => ",
            "fun (dot_sub_right_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (@vsub.{v} Vector vadd vneg y z)) (sub (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner x z))) => ",
            "fun (norm_sq_def_arg : forall (x : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) (@dot.{u,v} Scalar Vector inner x x)) => ",
            "fun (dist_sq_def_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@distSq.{u,v} Scalar Vector vadd vneg inner x y) (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg y x))) => ",
            "fun (dot_self_eq_norm_sq_arg : forall (x : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x x) (@normSq.{u,v} Scalar Vector inner x)) => ",
            "fun (norm_sq_add_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (add (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (norm_sq_sub_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (sub (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (inner_field13_arg : forall (x : Vector), forall (y : Vector), forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (inner_field14_arg : forall (x : Vector), forall (y : Vector), forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (norm_sq_nonneg_arg : forall (x : Vector), le_rel zero (@normSq.{u,v} Scalar Vector inner x)) => ",
            "fun (parallelogram_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (add (@normSq.{u,v} Scalar Vector inner (vadd x y)) (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y))) (add (mul (@two.{u} Scalar one add) (@normSq.{u,v} Scalar Vector inner x)) (mul (@two.{u} Scalar one add) (@normSq.{u,v} Scalar Vector inner y)))) => ",
            "fun (polarization_identity_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y)) (sub (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)))) => ",
            "fun (perp_vec_iff_dot_eq_zero_arg : forall (x : Vector), forall (y : Vector), forall (R : Prop), forall (mk : forall (forward : forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), forall (backward : forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @PerpVec.{u,v} Scalar zero Vector inner x y), R), R) => ",
            "fun (perp_vec_symm_arg : forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @PerpVec.{u,v} Scalar zero Vector inner y x) => ",
            "fun (norm_sq_zero_iff_arg : forall (x : Vector), forall (R : Prop), forall (mk : forall (forward : forall (h : @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) zero), @Eq.{v} Vector x vzero), forall (backward : forall (h : @Eq.{v} Vector x vzero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) zero), R), R) => ",
            "fun (dist_sq_nonneg_arg : forall (x : Vector), forall (y : Vector), le_rel zero (@distSq.{u,v} Scalar Vector vadd vneg inner x y)) => ",
            "fun (inner_field23_arg : forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (inner_field24_arg : forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (quadratic_norm_nonneg_arg : forall (x : Vector), forall (y : Vector), forall (t : Scalar), le_rel zero (add (add (mul (@normSq.{u,v} Scalar Vector inner x) (@sq.{u} Scalar mul t)) (mul (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y)) t)) (@normSq.{u,v} Scalar Vector inner y))) => norm_sq_sub_arg x y)"
        )),
    },
    TheoremArtifact {
        name: "parallelogram_law_from_inner_args",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (add (@normSq.{u,v} Scalar Vector inner (vadd x y)) (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y))) (add (mul (@two.{u} Scalar one add) (@normSq.{u,v} Scalar Vector inner x)) (mul (@two.{u} Scalar one add) (@normSq.{u,v} Scalar Vector inner y)))"
        ),
        proof: abstract_inner_product_abs!(concat!(
            "fun ring_args => fun inner_args => fun x => fun y => ",
            "@eq_trans.{u} Scalar ",
            "(add (@normSq.{u,v} Scalar Vector inner (vadd x y)) (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y))) ",
            "(add (add (add (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y)) (add (sub (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y))) ",
            "(add (mul (@two.{u} Scalar one add) (@normSq.{u,v} Scalar Vector inner x)) (mul (@two.{u} Scalar one add) (@normSq.{u,v} Scalar Vector inner y))) ",
            "(@eq_congr2.{u,u,u} Scalar Scalar Scalar add ",
            "(@normSq.{u,v} Scalar Vector inner (vadd x y)) ",
            "(add (add (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y)) ",
            "(@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) ",
            "(add (sub (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y)) ",
            "(@norm_sq_add_from_inner_args.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner inner_args x y) ",
            "(@norm_sq_sub_from_inner_args.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner inner_args x y)) ",
            "(@parallelogram_scalar_rhs_from_ring_args.{u} Scalar zero one add neg sub mul ring_args (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y)))"
        )),
    },
    TheoremArtifact {
        name: "polarization_identity_from_inner_args",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y)) (sub (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)))"
        ),
        proof: abstract_inner_product_abs!(concat!(
            "fun ring_args => fun inner_args => fun x => fun y => ",
            "@eq_trans.{u} Scalar ",
            "(mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y)) ",
            "(sub (add (add (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) ",
            "(sub (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) ",
            "(@polarization_scalar_rhs_from_ring_args.{u} Scalar zero one add neg sub mul ring_args (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y) (@dot.{u,v} Scalar Vector inner x y)) ",
            "(@eq_congr2.{u,u,u} Scalar Scalar Scalar sub ",
            "(add (add (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y)) ",
            "(@normSq.{u,v} Scalar Vector inner (vadd x y)) ",
            "(add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)) ",
            "(add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)) ",
            "(@eq_symm.{u} Scalar ",
            "(@normSq.{u,v} Scalar Vector inner (vadd x y)) ",
            "(add (add (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y)) ",
            "(@norm_sq_add_from_inner_args.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner inner_args x y)) ",
            "(@Eq.refl.{u} Scalar (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))))"
        )),
    },
    TheoremArtifact {
        name: "dot_neg_left_from_inner_args",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vneg x) y) (neg (@dot.{u,v} Scalar Vector inner x y))"
        ),
        proof: abstract_inner_product_abs!(concat!(
            "fun inner_args => fun x => fun y => ",
            "inner_args (@Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vneg x) y) (neg (@dot.{u,v} Scalar Vector inner x y))) ",
            "(fun (dot_comm_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner y x)) => ",
            "fun (dot_add_left_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vadd x y) z) (add (@dot.{u,v} Scalar Vector inner x z) (@dot.{u,v} Scalar Vector inner y z))) => ",
            "fun (dot_add_right_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (vadd y z)) (add (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner x z))) => ",
            "fun (dot_neg_left_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vneg x) y) (neg (@dot.{u,v} Scalar Vector inner x y))) => ",
            "fun (dot_neg_right_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (vneg y)) (neg (@dot.{u,v} Scalar Vector inner x y))) => ",
            "fun (dot_sub_left_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y) z) (sub (@dot.{u,v} Scalar Vector inner x z) (@dot.{u,v} Scalar Vector inner y z))) => ",
            "fun (dot_sub_right_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (@vsub.{v} Vector vadd vneg y z)) (sub (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner x z))) => ",
            "fun (norm_sq_def_arg : forall (x : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) (@dot.{u,v} Scalar Vector inner x x)) => ",
            "fun (dist_sq_def_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@distSq.{u,v} Scalar Vector vadd vneg inner x y) (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg y x))) => ",
            "fun (dot_self_eq_norm_sq_arg : forall (x : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x x) (@normSq.{u,v} Scalar Vector inner x)) => ",
            "fun (norm_sq_add_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (add (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (norm_sq_sub_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (sub (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (inner_field13_arg : forall (x : Vector), forall (y : Vector), forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (inner_field14_arg : forall (x : Vector), forall (y : Vector), forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (norm_sq_nonneg_arg : forall (x : Vector), le_rel zero (@normSq.{u,v} Scalar Vector inner x)) => ",
            "fun (parallelogram_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (add (@normSq.{u,v} Scalar Vector inner (vadd x y)) (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y))) (add (mul (@two.{u} Scalar one add) (@normSq.{u,v} Scalar Vector inner x)) (mul (@two.{u} Scalar one add) (@normSq.{u,v} Scalar Vector inner y)))) => ",
            "fun (polarization_identity_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y)) (sub (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)))) => ",
            "fun (perp_vec_iff_dot_eq_zero_arg : forall (x : Vector), forall (y : Vector), forall (R : Prop), forall (mk : forall (forward : forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), forall (backward : forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @PerpVec.{u,v} Scalar zero Vector inner x y), R), R) => ",
            "fun (perp_vec_symm_arg : forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @PerpVec.{u,v} Scalar zero Vector inner y x) => ",
            "fun (norm_sq_zero_iff_arg : forall (x : Vector), forall (R : Prop), forall (mk : forall (forward : forall (h : @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) zero), @Eq.{v} Vector x vzero), forall (backward : forall (h : @Eq.{v} Vector x vzero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) zero), R), R) => ",
            "fun (dist_sq_nonneg_arg : forall (x : Vector), forall (y : Vector), le_rel zero (@distSq.{u,v} Scalar Vector vadd vneg inner x y)) => ",
            "fun (inner_field23_arg : forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (inner_field24_arg : forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (quadratic_norm_nonneg_arg : forall (x : Vector), forall (y : Vector), forall (t : Scalar), le_rel zero (add (add (mul (@normSq.{u,v} Scalar Vector inner x) (@sq.{u} Scalar mul t)) (mul (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y)) t)) (@normSq.{u,v} Scalar Vector inner y))) => dot_neg_left_arg x y)"
        )),
    },
    TheoremArtifact {
        name: "norm_sq_neg_from_inner_args",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (x : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vneg x)) (@normSq.{u,v} Scalar Vector inner x)"
        ),
        proof: abstract_inner_product_abs!(concat!(
            "fun ring_args => fun inner_args => fun x => ",
            "ring_args (@Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vneg x)) (@normSq.{u,v} Scalar Vector inner x)) ",
            "(fun (sub_eq_add_neg_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub a b) (add a (neg b))) => ",
            "fun (add_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add a b) c) (add a (add b c))) => ",
            "fun (add_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add a b) (add b a)) => ",
            "fun (add_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (add a zero) a) => ",
            "fun (zero_add_arg : forall (a : Scalar), @Eq.{u} Scalar (add zero a) a) => ",
            "fun (neg_add_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add (neg a) a) zero) => ",
            "fun (add_neg_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add a (neg a)) zero) => ",
            "fun (sub_self_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a a) zero) => ",
            "fun (mul_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (mul a b) c) (mul a (mul b c))) => ",
            "fun (mul_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (mul a b) (mul b a)) => ",
            "fun (mul_one_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a one) a) => ",
            "fun (one_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul one a) a) => ",
            "fun (left_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul a (add b c)) (add (mul a b) (mul a c))) => ",
            "fun (right_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (add a b) c) (add (mul a c) (mul b c))) => ",
            "fun (mul_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a zero) zero) => ",
            "fun (zero_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul zero a) zero) => ",
            "fun (add_left_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add a b) (add a c)), @Eq.{u} Scalar b c) => ",
            "fun (ring_normalize_add_mul3_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add (mul a b) (mul b c)) (mul a c)) (add (add (mul a b) (mul a c)) (mul b c))) => ",
            "fun (add_right_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add b a) (add c a)), @Eq.{u} Scalar b c) => ",
            "fun (neg_neg_arg : forall (a : Scalar), @Eq.{u} Scalar (neg (neg a)) a) => ",
            "fun (sub_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a zero) a) => ",
            "fun (zero_sub_arg : forall (a : Scalar), @Eq.{u} Scalar (sub zero a) (neg a)) => ",
            "fun (sub_add_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add (sub a b) b) a) => ",
            "fun (add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub (add a b) b) a) => ",
            "fun (sub_add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (sub (sub a c) (sub b c)) (sub a b)) => ",
            "inner_args (@Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vneg x)) (@normSq.{u,v} Scalar Vector inner x)) ",
            "(fun (dot_comm_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner y x)) => ",
            "fun (dot_add_left_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vadd x y) z) (add (@dot.{u,v} Scalar Vector inner x z) (@dot.{u,v} Scalar Vector inner y z))) => ",
            "fun (dot_add_right_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (vadd y z)) (add (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner x z))) => ",
            "fun (dot_neg_left_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vneg x) y) (neg (@dot.{u,v} Scalar Vector inner x y))) => ",
            "fun (dot_neg_right_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (vneg y)) (neg (@dot.{u,v} Scalar Vector inner x y))) => ",
            "fun (dot_sub_left_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y) z) (sub (@dot.{u,v} Scalar Vector inner x z) (@dot.{u,v} Scalar Vector inner y z))) => ",
            "fun (dot_sub_right_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (@vsub.{v} Vector vadd vneg y z)) (sub (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner x z))) => ",
            "fun (norm_sq_def_arg : forall (x : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) (@dot.{u,v} Scalar Vector inner x x)) => ",
            "fun (dist_sq_def_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@distSq.{u,v} Scalar Vector vadd vneg inner x y) (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg y x))) => ",
            "fun (dot_self_eq_norm_sq_arg : forall (x : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x x) (@normSq.{u,v} Scalar Vector inner x)) => ",
            "fun (norm_sq_add_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (add (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (norm_sq_sub_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (sub (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (inner_field13_arg : forall (x : Vector), forall (y : Vector), forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (inner_field14_arg : forall (x : Vector), forall (y : Vector), forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (norm_sq_nonneg_arg : forall (x : Vector), le_rel zero (@normSq.{u,v} Scalar Vector inner x)) => ",
            "fun (parallelogram_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (add (@normSq.{u,v} Scalar Vector inner (vadd x y)) (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y))) (add (mul (@two.{u} Scalar one add) (@normSq.{u,v} Scalar Vector inner x)) (mul (@two.{u} Scalar one add) (@normSq.{u,v} Scalar Vector inner y)))) => ",
            "fun (polarization_identity_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y)) (sub (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)))) => ",
            "fun (perp_vec_iff_dot_eq_zero_arg : forall (x : Vector), forall (y : Vector), forall (R : Prop), forall (mk : forall (forward : forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), forall (backward : forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @PerpVec.{u,v} Scalar zero Vector inner x y), R), R) => ",
            "fun (perp_vec_symm_arg : forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @PerpVec.{u,v} Scalar zero Vector inner y x) => ",
            "fun (norm_sq_zero_iff_arg : forall (x : Vector), forall (R : Prop), forall (mk : forall (forward : forall (h : @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) zero), @Eq.{v} Vector x vzero), forall (backward : forall (h : @Eq.{v} Vector x vzero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) zero), R), R) => ",
            "fun (dist_sq_nonneg_arg : forall (x : Vector), forall (y : Vector), le_rel zero (@distSq.{u,v} Scalar Vector vadd vneg inner x y)) => ",
            "fun (inner_field23_arg : forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (inner_field24_arg : forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (quadratic_norm_nonneg_arg : forall (x : Vector), forall (y : Vector), forall (t : Scalar), le_rel zero (add (add (mul (@normSq.{u,v} Scalar Vector inner x) (@sq.{u} Scalar mul t)) (mul (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y)) t)) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "@Eq.rec.{u,0} Scalar (@dot.{u,v} Scalar Vector inner (vneg x) (vneg x)) ",
            "(fun (z : Scalar) => fun (hz : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vneg x) (vneg x)) z) => @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vneg x)) z) ",
            "(norm_sq_def_arg (vneg x)) (@normSq.{u,v} Scalar Vector inner x) ",
            "(@Eq.rec.{u,0} Scalar (neg (@dot.{u,v} Scalar Vector inner x (vneg x))) ",
            "(fun (z : Scalar) => fun (hz : @Eq.{u} Scalar (neg (@dot.{u,v} Scalar Vector inner x (vneg x))) z) => @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vneg x) (vneg x)) z) ",
            "(@dot_neg_left_from_inner_args.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner inner_args x (vneg x)) ",
            "(@normSq.{u,v} Scalar Vector inner x) ",
            "(@Eq.rec.{u,0} Scalar (neg (neg (@dot.{u,v} Scalar Vector inner x x))) ",
            "(fun (z : Scalar) => fun (hz : @Eq.{u} Scalar (neg (neg (@dot.{u,v} Scalar Vector inner x x))) z) => @Eq.{u} Scalar (neg (@dot.{u,v} Scalar Vector inner x (vneg x))) z) ",
            "(@Eq.rec.{u,0} Scalar (@dot.{u,v} Scalar Vector inner x (vneg x)) ",
            "(fun (z : Scalar) => fun (hz : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (vneg x)) z) => @Eq.{u} Scalar (neg (@dot.{u,v} Scalar Vector inner x (vneg x))) (neg z)) ",
            "(@Eq.refl.{u} Scalar (neg (@dot.{u,v} Scalar Vector inner x (vneg x)))) (neg (@dot.{u,v} Scalar Vector inner x x)) (dot_neg_right_arg x x)) ",
            "(@normSq.{u,v} Scalar Vector inner x) ",
            "(@Eq.rec.{u,0} Scalar (@dot.{u,v} Scalar Vector inner x x) ",
            "(fun (z : Scalar) => fun (hz : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x x) z) => @Eq.{u} Scalar (neg (neg (@dot.{u,v} Scalar Vector inner x x))) z) ",
            "(neg_neg_arg (@dot.{u,v} Scalar Vector inner x x)) (@normSq.{u,v} Scalar Vector inner x) (dot_self_eq_norm_sq_arg x))))))"
        )),
    },
    TheoremArtifact {
        name: "norm_sq_add_of_dot_zero_from_args",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (x : Vector), forall (y : Vector), forall (hzero : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))"
        ),
        proof: abstract_inner_product_abs!(concat!(
            "fun ring_args => fun inner_args => fun x => fun y => fun hzero => ",
            "@Eq.rec.{u,0} Scalar (add (add (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y)) ",
            "(fun (z : Scalar) => fun (hz : @Eq.{u} Scalar (add (add (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y)) z) => @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) z) ",
            "(@norm_sq_add_from_inner_args.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner inner_args x y) ",
            "(add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)) ",
            "(@normalize_add_with_zero_cross_term_from_ring_args.{u} Scalar zero one add neg sub mul ring_args (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y) (@dot.{u,v} Scalar Vector inner x y) hzero)"
        )),
    },
    TheoremArtifact {
        name: "norm_sq_add_of_perp_from_args",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (x : Vector), forall (y : Vector), forall (hperp : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))"
        ),
        proof: abstract_inner_product_abs!(
            "fun ring_args => fun inner_args => fun x => fun y => fun hperp => @norm_sq_add_of_dot_zero_from_args.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner ring_args inner_args x y hperp"
        ),
    },
    TheoremArtifact {
        name: "norm_sq_add_neg_left_from_inner_args",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd (vneg x) y)) (sub (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y)))"
        ),
        proof: abstract_inner_product_abs!(concat!(
            "fun ring_args => fun inner_args => fun x => fun y => ",
            "@Eq.rec.{u,0} Scalar (add (add (@normSq.{u,v} Scalar Vector inner (vneg x)) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (vneg x) y))) (@normSq.{u,v} Scalar Vector inner y)) ",
            "(fun (z : Scalar) => fun (hz : @Eq.{u} Scalar (add (add (@normSq.{u,v} Scalar Vector inner (vneg x)) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (vneg x) y))) (@normSq.{u,v} Scalar Vector inner y)) z) => @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd (vneg x) y)) z) ",
            "(@norm_sq_add_from_inner_args.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner inner_args (vneg x) y) ",
            "(sub (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) ",
            "(@Eq.rec.{u,0} Scalar (add (add (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (vneg x) y))) (@normSq.{u,v} Scalar Vector inner y)) ",
            "(fun (z : Scalar) => fun (hz : @Eq.{u} Scalar (add (add (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (vneg x) y))) (@normSq.{u,v} Scalar Vector inner y)) z) => @Eq.{u} Scalar (add (add (@normSq.{u,v} Scalar Vector inner (vneg x)) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (vneg x) y))) (@normSq.{u,v} Scalar Vector inner y)) z) ",
            "(@Eq.rec.{u,0} Scalar (@normSq.{u,v} Scalar Vector inner (vneg x)) ",
            "(fun (z : Scalar) => fun (hz : @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vneg x)) z) => @Eq.{u} Scalar (add (add (@normSq.{u,v} Scalar Vector inner (vneg x)) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (vneg x) y))) (@normSq.{u,v} Scalar Vector inner y)) (add (add z (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (vneg x) y))) (@normSq.{u,v} Scalar Vector inner y))) ",
            "(@Eq.refl.{u} Scalar (add (add (@normSq.{u,v} Scalar Vector inner (vneg x)) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (vneg x) y))) (@normSq.{u,v} Scalar Vector inner y))) ",
            "(@normSq.{u,v} Scalar Vector inner x) ",
            "(@norm_sq_neg_from_inner_args.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner ring_args inner_args x)) ",
            "(sub (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) ",
            "(@Eq.rec.{u,0} Scalar (add (add (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (neg (@dot.{u,v} Scalar Vector inner x y)))) (@normSq.{u,v} Scalar Vector inner y)) ",
            "(fun (z : Scalar) => fun (hz : @Eq.{u} Scalar (add (add (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (neg (@dot.{u,v} Scalar Vector inner x y)))) (@normSq.{u,v} Scalar Vector inner y)) z) => @Eq.{u} Scalar (add (add (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (vneg x) y))) (@normSq.{u,v} Scalar Vector inner y)) z) ",
            "(@Eq.rec.{u,0} Scalar (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (vneg x) y)) ",
            "(fun (z : Scalar) => fun (hz : @Eq.{u} Scalar (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (vneg x) y)) z) => @Eq.{u} Scalar (add (add (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (vneg x) y))) (@normSq.{u,v} Scalar Vector inner y)) (add (add (@normSq.{u,v} Scalar Vector inner x) z) (@normSq.{u,v} Scalar Vector inner y))) ",
            "(@Eq.refl.{u} Scalar (add (add (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (vneg x) y))) (@normSq.{u,v} Scalar Vector inner y))) ",
            "(mul (@two.{u} Scalar one add) (neg (@dot.{u,v} Scalar Vector inner x y))) ",
            "(@Eq.rec.{u,0} Scalar (@dot.{u,v} Scalar Vector inner (vneg x) y) ",
            "(fun (z : Scalar) => fun (hz : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vneg x) y) z) => @Eq.{u} Scalar (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (vneg x) y)) (mul (@two.{u} Scalar one add) z)) ",
            "(@Eq.refl.{u} Scalar (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (vneg x) y))) ",
            "(neg (@dot.{u,v} Scalar Vector inner x y)) ",
            "(@dot_neg_left_from_inner_args.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner inner_args x y))) ",
            "(sub (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) ",
            "(@law_of_cosines_scalar_rhs_from_ring_args.{u} Scalar zero one add neg sub mul ring_args (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y) (@dot.{u,v} Scalar Vector inner x y))))"
        )),
    },
    TheoremArtifact {
        name: "dot_zero_left_from_law_packages",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (vector_args : @VectorSpaceLawArgs.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner vzero y) zero"
        ),
        proof: concat!(
            "fun Scalar => fun zero => fun one => fun add => fun neg => fun sub => fun mul => fun le_rel => fun Vector => fun vzero => fun vadd => fun vneg => fun smul => fun inner => ",
            "fun ring_args => fun vector_args => fun inner_args => fun y => ",
            ring_args_elim!(
                "(@Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner vzero y) zero)",
                vector_args_elim!(
                    "(@Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner vzero y) zero)",
                    inner_args_elim!(
                        "(@Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner vzero y) zero)",
                        "add_right_cancel_arg (@dot.{u,v} Scalar Vector inner vzero y) (@dot.{u,v} Scalar Vector inner vzero y) zero ",
                        "(@eq_trans.{u} Scalar ",
                        "(add (@dot.{u,v} Scalar Vector inner vzero y) (@dot.{u,v} Scalar Vector inner vzero y)) ",
                        "(@dot.{u,v} Scalar Vector inner vzero y) ",
                        "(add zero (@dot.{u,v} Scalar Vector inner vzero y)) ",
                        "(@eq_trans.{u} Scalar ",
                        "(add (@dot.{u,v} Scalar Vector inner vzero y) (@dot.{u,v} Scalar Vector inner vzero y)) ",
                        "(@dot.{u,v} Scalar Vector inner (vadd vzero vzero) y) ",
                        "(@dot.{u,v} Scalar Vector inner vzero y) ",
                        "(@eq_symm.{u} Scalar ",
                        "(@dot.{u,v} Scalar Vector inner (vadd vzero vzero) y) ",
                        "(add (@dot.{u,v} Scalar Vector inner vzero y) (@dot.{u,v} Scalar Vector inner vzero y)) ",
                        "(dot_add_left_arg vzero vzero y)) ",
                        "(@eq_congr_arg.{v,u} Vector Scalar ",
                        "(fun (z : Vector) => @dot.{u,v} Scalar Vector inner z y) ",
                        "(vadd vzero vzero) vzero (vec_zero_add_arg vzero))) ",
                        "(@eq_symm.{u} Scalar ",
                        "(add zero (@dot.{u,v} Scalar Vector inner vzero y)) ",
                        "(@dot.{u,v} Scalar Vector inner vzero y) ",
                        "(zero_add_arg (@dot.{u,v} Scalar Vector inner vzero y))))"
                    )
                )
            )
        ),
    },
    TheoremArtifact {
        name: "dot_zero_right_from_law_packages",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (vector_args : @VectorSpaceLawArgs.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (x : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x vzero) zero"
        ),
        proof: concat!(
            "fun Scalar => fun zero => fun one => fun add => fun neg => fun sub => fun mul => fun le_rel => fun Vector => fun vzero => fun vadd => fun vneg => fun smul => fun inner => ",
            "fun ring_args => fun vector_args => fun inner_args => fun x => ",
            inner_args_elim!(
                "(@Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x vzero) zero)",
                "@eq_trans.{u} Scalar ",
                "(@dot.{u,v} Scalar Vector inner x vzero) ",
                "(@dot.{u,v} Scalar Vector inner vzero x) ",
                "zero ",
                "(dot_comm_arg x vzero) ",
                "(@dot_zero_left_from_law_packages.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner ring_args vector_args inner_args x)"
            )
        ),
    },
    TheoremArtifact {
        name: "dot_eq_zero_of_norm_sq_zero_left_from_inner_args",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (vector_args : @VectorSpaceLawArgs.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (x : Vector), forall (y : Vector), forall (hzero : @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) zero), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero"
        ),
        proof: concat!(
            "fun Scalar => fun zero => fun one => fun add => fun neg => fun sub => fun mul => fun le_rel => fun Vector => fun vzero => fun vadd => fun vneg => fun smul => fun inner => ",
            "fun ring_args => fun vector_args => fun inner_args => fun x => fun y => fun hzero => ",
            inner_args_elim!(
                "(@Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero)",
                "@eq_trans.{u} Scalar ",
                "(@dot.{u,v} Scalar Vector inner x y) ",
                "(@dot.{u,v} Scalar Vector inner vzero y) ",
                "zero ",
                "(@eq_congr_arg.{v,u} Vector Scalar ",
                "(fun (z : Vector) => @dot.{u,v} Scalar Vector inner z y) ",
                "x vzero ",
                "(norm_sq_zero_iff_arg x (@Eq.{v} Vector x vzero) ",
                "(fun (forward : forall (h : @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) zero), @Eq.{v} Vector x vzero) => ",
                "fun (backward : forall (h : @Eq.{v} Vector x vzero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) zero) => forward hzero))) ",
                "(@dot_zero_left_from_law_packages.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner ring_args vector_args inner_args y)"
            )
        ),
    },
    TheoremArtifact {
        name: "dot_eq_zero_of_norm_sq_zero_right_from_inner_args",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (vector_args : @VectorSpaceLawArgs.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (x : Vector), forall (y : Vector), forall (hzero : @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner y) zero), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero"
        ),
        proof: concat!(
            "fun Scalar => fun zero => fun one => fun add => fun neg => fun sub => fun mul => fun le_rel => fun Vector => fun vzero => fun vadd => fun vneg => fun smul => fun inner => ",
            "fun ring_args => fun vector_args => fun inner_args => fun x => fun y => fun hzero => ",
            inner_args_elim!(
                "(@Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero)",
                "@eq_trans.{u} Scalar ",
                "(@dot.{u,v} Scalar Vector inner x y) ",
                "(@dot.{u,v} Scalar Vector inner x vzero) ",
                "zero ",
                "(@eq_congr_arg.{v,u} Vector Scalar ",
                "(fun (z : Vector) => @dot.{u,v} Scalar Vector inner x z) ",
                "y vzero ",
                "(norm_sq_zero_iff_arg y (@Eq.{v} Vector y vzero) ",
                "(fun (forward : forall (h : @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner y) zero), @Eq.{v} Vector y vzero) => ",
                "fun (backward : forall (h : @Eq.{v} Vector y vzero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner y) zero) => forward hzero))) ",
                "(@dot_zero_right_from_law_packages.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner ring_args vector_args inner_args x)"
            )
        ),
    },
    TheoremArtifact {
        name: "cauchy_schwarz_zero_left_from_law_packages",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (lt_rel : forall (a : Scalar), forall (b : Scalar), Prop), forall (sqrt_fn : forall (a : Scalar), Scalar), forall (ordered_args : @OrderedFieldLawArgs.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn), forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (vector_args : @VectorSpaceLawArgs.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (x : Vector), forall (y : Vector), forall (hzero : @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) zero), le_rel (@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y)) (mul (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))"
        ),
        proof: concat!(
            "fun Scalar => fun zero => fun one => fun add => fun neg => fun sub => fun mul => fun le_rel => fun Vector => fun vzero => fun vadd => fun vneg => fun smul => fun inner => ",
            "fun lt_rel => fun sqrt_fn => fun ordered_args => fun ring_args => fun vector_args => fun inner_args => fun x => fun y => fun hzero => ",
            ordered_args_elim!(
                "(le_rel (@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y)) (mul (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)))",
                ring_args_elim!(
                    "(le_rel (@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y)) (mul (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)))",
                    "le_of_eq_arg ",
                    "(@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y)) ",
                    "(mul (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)) ",
                    "(@eq_trans.{u} Scalar ",
                    "(@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y)) ",
                    "zero ",
                    "(mul (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)) ",
                    "(sq_eq_zero_iff_arg (@dot.{u,v} Scalar Vector inner x y) ",
                    "(@Eq.{u} Scalar (@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y)) zero) ",
                    "(fun (forward : forall (hsqz : @Eq.{u} Scalar (@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y)) zero), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero) => ",
                    "fun (backward : forall (haz : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @Eq.{u} Scalar (@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y)) zero) => ",
                    "backward (@dot_eq_zero_of_norm_sq_zero_left_from_inner_args.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner ring_args vector_args inner_args x y hzero))) ",
                    "(@eq_symm.{u} Scalar ",
                    "(mul (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)) ",
                    "zero ",
                    "(@eq_trans.{u} Scalar ",
                    "(mul (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)) ",
                    "(mul zero (@normSq.{u,v} Scalar Vector inner y)) ",
                    "zero ",
                    "(@eq_congr_arg.{u,u} Scalar Scalar ",
                    "(fun (z : Scalar) => mul z (@normSq.{u,v} Scalar Vector inner y)) ",
                    "(@normSq.{u,v} Scalar Vector inner x) zero hzero) ",
                    "(zero_mul_arg (@normSq.{u,v} Scalar Vector inner y))))) ",
                    "(le_rel (@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y)) (mul (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) ",
                    "(fun (hab_le : le_rel (@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y)) (mul (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
                    "fun (hba_le : le_rel (mul (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)) (@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y))) => hab_le)"
                )
            )
        ),
    },
    TheoremArtifact {
        name: "cauchy_schwarz_zero_right_from_law_packages",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (lt_rel : forall (a : Scalar), forall (b : Scalar), Prop), forall (sqrt_fn : forall (a : Scalar), Scalar), forall (ordered_args : @OrderedFieldLawArgs.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn), forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (vector_args : @VectorSpaceLawArgs.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (x : Vector), forall (y : Vector), forall (hzero : @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner y) zero), le_rel (@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y)) (mul (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))"
        ),
        proof: concat!(
            "fun Scalar => fun zero => fun one => fun add => fun neg => fun sub => fun mul => fun le_rel => fun Vector => fun vzero => fun vadd => fun vneg => fun smul => fun inner => ",
            "fun lt_rel => fun sqrt_fn => fun ordered_args => fun ring_args => fun vector_args => fun inner_args => fun x => fun y => fun hzero => ",
            ordered_args_elim!(
                "(le_rel (@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y)) (mul (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)))",
                ring_args_elim!(
                    "(le_rel (@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y)) (mul (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)))",
                    "le_of_eq_arg ",
                    "(@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y)) ",
                    "(mul (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)) ",
                    "(@eq_trans.{u} Scalar ",
                    "(@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y)) ",
                    "zero ",
                    "(mul (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)) ",
                    "(sq_eq_zero_iff_arg (@dot.{u,v} Scalar Vector inner x y) ",
                    "(@Eq.{u} Scalar (@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y)) zero) ",
                    "(fun (forward : forall (hsqz : @Eq.{u} Scalar (@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y)) zero), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero) => ",
                    "fun (backward : forall (haz : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @Eq.{u} Scalar (@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y)) zero) => ",
                    "backward (@dot_eq_zero_of_norm_sq_zero_right_from_inner_args.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner ring_args vector_args inner_args x y hzero))) ",
                    "(@eq_symm.{u} Scalar ",
                    "(mul (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)) ",
                    "zero ",
                    "(@eq_trans.{u} Scalar ",
                    "(mul (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)) ",
                    "(mul (@normSq.{u,v} Scalar Vector inner x) zero) ",
                    "zero ",
                    "(@eq_congr_arg.{u,u} Scalar Scalar ",
                    "(fun (z : Scalar) => mul (@normSq.{u,v} Scalar Vector inner x) z) ",
                    "(@normSq.{u,v} Scalar Vector inner y) zero hzero) ",
                    "(mul_zero_arg (@normSq.{u,v} Scalar Vector inner x))))) ",
                    "(le_rel (@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y)) (mul (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) ",
                    "(fun (hab_le : le_rel (@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y)) (mul (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
                    "fun (hba_le : le_rel (mul (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)) (@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y))) => hab_le)"
                )
            )
        ),
    },
    TheoremArtifact {
        name: "cauchy_schwarz_from_law_packages",
        universe_params: &["u", "v"],
        statement: abstract_ordered_inner_product_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (ordered_args : @OrderedFieldLawArgs.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn), forall (vector_args : @VectorSpaceLawArgs.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (x : Vector), forall (y : Vector), le_rel (@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y)) (mul (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))"
        ),
        proof: abstract_ordered_inner_product_abs!(concat!(
            "fun ring_args => fun ordered_args => fun vector_args => fun inner_args => fun x => fun y => ",
            inner_args_elim!(
                "(le_rel (@sq.{u} Scalar mul (@dot.{u,v} Scalar Vector inner x y)) (mul (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)))",
                "@square_completion_bound_from_ordered_args.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn ordered_args ",
                "(@normSq.{u,v} Scalar Vector inner x) ",
                "(@dot.{u,v} Scalar Vector inner x y) ",
                "(@normSq.{u,v} Scalar Vector inner y) ",
                "(quadratic_norm_nonneg_arg x y)"
            )
        )),
    },
    TheoremArtifact {
        name: "norm_sq_nonneg_from_inner_args",
        universe_params: &["u", "v"],
        statement: abstract_inner_product_params!(
            "forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (x : Vector), le_rel zero (@normSq.{u,v} Scalar Vector inner x)"
        ),
        proof: abstract_inner_product_abs!(concat!(
            "fun inner_args => fun x => ",
            inner_args_elim!(
                "(le_rel zero (@normSq.{u,v} Scalar Vector inner x))",
                "norm_sq_nonneg_arg x"
            )
        )),
    },
    TheoremArtifact {
        name: "dot_le_mul_sqrt_norm_sq_from_cauchy",
        universe_params: &["u", "v"],
        statement: abstract_ordered_inner_product_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (ordered_args : @OrderedFieldLawArgs.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn), forall (vector_args : @VectorSpaceLawArgs.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (x : Vector), forall (y : Vector), le_rel (@dot.{u,v} Scalar Vector inner x y) (mul (sqrt_fn (@normSq.{u,v} Scalar Vector inner x)) (sqrt_fn (@normSq.{u,v} Scalar Vector inner y)))"
        ),
        proof: abstract_ordered_inner_product_abs!(concat!(
            "fun ring_args => fun ordered_args => fun vector_args => fun inner_args => fun x => fun y => ",
            "@le_mul_sqrt_of_sq_le_mul_nonneg_from_ordered_args.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn ordered_args ",
            "(@normSq.{u,v} Scalar Vector inner x) ",
            "(@normSq.{u,v} Scalar Vector inner y) ",
            "(@dot.{u,v} Scalar Vector inner x y) ",
            "(@norm_sq_nonneg_from_inner_args.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner inner_args x) ",
            "(@norm_sq_nonneg_from_inner_args.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner inner_args y) ",
            "(@cauchy_schwarz_from_law_packages.{u,v} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn Vector vzero vadd vneg smul inner ring_args ordered_args vector_args inner_args x y)"
        )),
    },
    TheoremArtifact {
        name: "norm_sq_add_le_square_sum_norms_from_cauchy",
        universe_params: &["u", "v"],
        statement: abstract_ordered_inner_product_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (ordered_args : @OrderedFieldLawArgs.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn), forall (vector_args : @VectorSpaceLawArgs.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (x : Vector), forall (y : Vector), le_rel (@normSq.{u,v} Scalar Vector inner (vadd x y)) (@sq.{u} Scalar mul (add (sqrt_fn (@normSq.{u,v} Scalar Vector inner x)) (sqrt_fn (@normSq.{u,v} Scalar Vector inner y))))"
        ),
        proof: abstract_ordered_inner_product_abs!(concat!(
            "fun ring_args => fun ordered_args => fun vector_args => fun inner_args => fun x => fun y => ",
            "@add_two_mul_le_sq_add_sqrt_from_ordered_args.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn ordered_args ",
            "(@normSq.{u,v} Scalar Vector inner x) ",
            "(@normSq.{u,v} Scalar Vector inner y) ",
            "(@dot.{u,v} Scalar Vector inner x y) ",
            "(@normSq.{u,v} Scalar Vector inner (vadd x y)) ",
            "(@norm_sq_nonneg_from_inner_args.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner inner_args x) ",
            "(@norm_sq_nonneg_from_inner_args.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner inner_args y) ",
            "(@norm_sq_add_from_inner_args.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner inner_args x y) ",
            "(@dot_le_mul_sqrt_norm_sq_from_cauchy.{u,v} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn Vector vzero vadd vneg smul inner ring_args ordered_args vector_args inner_args x y)"
        )),
    },
];

const AFFINE_DEFINITIONS: &[DefinitionArtifact] = &[
    DefinitionArtifact {
        name: "Point",
        universe_params: &["p"],
        ty: "forall (Carrier : Sort p), Sort p",
        value: "fun Carrier => Carrier",
    },
    DefinitionArtifact {
        name: "disp",
        universe_params: &["p", "v"],
        ty: concat!(
            "forall (PointCarrier : Sort p), ",
            "forall (Vector : Sort v), ",
            "forall (disp_op : forall (A : PointCarrier), forall (B : PointCarrier), Vector), ",
            "forall (A : PointCarrier), forall (B : PointCarrier), Vector"
        ),
        value: "fun PointCarrier => fun Vector => fun disp_op => fun A => fun B => disp_op A B",
    },
    DefinitionArtifact {
        name: "distSqPoints",
        universe_params: &["p", "u", "v"],
        ty: concat!(
            "forall (Scalar : Sort u), ",
            "forall (Vector : Sort v), ",
            "forall (inner : forall (x : Vector), forall (y : Vector), Scalar), ",
            "forall (PointCarrier : Sort p), ",
            "forall (disp_op : forall (A : PointCarrier), forall (B : PointCarrier), Vector), ",
            "forall (A : PointCarrier), forall (B : PointCarrier), Scalar"
        ),
        value:
            "fun Scalar => fun Vector => fun inner => fun PointCarrier => fun disp_op => fun A => fun B => @normSq.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B)",
    },
    DefinitionArtifact {
        name: "translate",
        universe_params: &["p", "v"],
        ty: concat!(
            "forall (PointCarrier : Sort p), ",
            "forall (Vector : Sort v), ",
            "forall (translate_op : forall (A : PointCarrier), forall (v : Vector), PointCarrier), ",
            "forall (A : PointCarrier), forall (v : Vector), PointCarrier"
        ),
        value:
            "fun PointCarrier => fun Vector => fun translate_op => fun A => fun v => translate_op A v",
    },
    DefinitionArtifact {
        name: "midpoint",
        universe_params: &["p"],
        ty: concat!(
            "forall (PointCarrier : Sort p), ",
            "forall (midpoint_op : forall (A : PointCarrier), forall (B : PointCarrier), PointCarrier), ",
            "forall (A : PointCarrier), forall (B : PointCarrier), PointCarrier"
        ),
        value: "fun PointCarrier => fun midpoint_op => fun A => fun B => midpoint_op A B",
    },
    DefinitionArtifact {
        name: "collinear",
        universe_params: &["p"],
        ty: concat!(
            "forall (PointCarrier : Sort p), ",
            "forall (collinear_rel : forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), Prop), ",
            "forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), Prop"
        ),
        value:
            "fun PointCarrier => fun collinear_rel => fun A => fun B => fun C => collinear_rel A B C",
    },
    DefinitionArtifact {
        name: "AffineLawArgs",
        universe_params: &["p", "u", "v"],
        ty: affine_params!("Prop"),
        value: affine_abs!(concat!(
            "forall (P : Prop), forall (mk : ",
            "forall (disp_self_law : forall (A : PointCarrier), @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op A A) vzero), ",
            "forall (disp_reverse_law : forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op B A) (vneg (@disp.{p,v} PointCarrier Vector disp_op A B))), ",
            "forall (disp_comp_law : forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op A C) (vadd (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op B C))), ",
            "forall (point_ext_of_zero_disp_law : forall (A : PointCarrier), forall (B : PointCarrier), forall (h : @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op A B) vzero), @Eq.{p} PointCarrier A B), ",
            "forall (dist_sq_symm_law : forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B A)), ",
            "forall (dist_sq_zero_iff_eq_law : forall (A : PointCarrier), forall (B : PointCarrier), forall (R : Prop), forall (mk : forall (forward : forall (h : @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) zero), @Eq.{p} PointCarrier A B), forall (backward : forall (h : @Eq.{p} PointCarrier A B), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) zero), R), R), P), P"
        )),
    },
];

const AFFINE_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "disp_self",
        universe_params: &["p", "u", "v"],
        statement: affine_params!(
            "forall (law : forall (A : PointCarrier), @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op A A) vzero), forall (A : PointCarrier), @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op A A) vzero"
        ),
        proof: affine_abs!("fun law => fun A => law A"),
    },
    TheoremArtifact {
        name: "disp_reverse",
        universe_params: &["p", "u", "v"],
        statement: affine_params!(
            "forall (law : forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op B A) (vneg (@disp.{p,v} PointCarrier Vector disp_op A B))), forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op B A) (vneg (@disp.{p,v} PointCarrier Vector disp_op A B))"
        ),
        proof: affine_abs!("fun law => fun A => fun B => law A B"),
    },
    TheoremArtifact {
        name: "disp_comp",
        universe_params: &["p", "u", "v"],
        statement: affine_params!(
            "forall (law : forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op A C) (vadd (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op B C))), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op A C) (vadd (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op B C))"
        ),
        proof: affine_abs!("fun law => fun A => fun B => fun C => law A B C"),
    },
    TheoremArtifact {
        name: "hypotenuse_vector_eq_sub_legs",
        universe_params: &["p", "u", "v"],
        statement: affine_params!(
            "forall (law : forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op B C) (@vsub.{v} Vector vadd vneg (@disp.{p,v} PointCarrier Vector disp_op A C) (@disp.{p,v} PointCarrier Vector disp_op A B))), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op B C) (@vsub.{v} Vector vadd vneg (@disp.{p,v} PointCarrier Vector disp_op A C) (@disp.{p,v} PointCarrier Vector disp_op A B))"
        ),
        proof: affine_abs!("fun law => fun A => fun B => fun C => law A B C"),
    },
    TheoremArtifact {
        name: "dist_sq_points_def",
        universe_params: &["p", "u", "v"],
        statement: affine_params!(
            "forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@normSq.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B))"
        ),
        proof: affine_abs!(
            "fun A => fun B => @Eq.refl.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B)"
        ),
    },
    TheoremArtifact {
        name: "point_ext_of_zero_disp",
        universe_params: &["p", "u", "v"],
        statement: affine_params!(
            "forall (law : forall (A : PointCarrier), forall (B : PointCarrier), forall (h : @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op A B) vzero), @Eq.{p} PointCarrier A B), forall (A : PointCarrier), forall (B : PointCarrier), forall (h : @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op A B) vzero), @Eq.{p} PointCarrier A B"
        ),
        proof: affine_abs!("fun law => fun A => fun B => fun h => law A B h"),
    },
    TheoremArtifact {
        name: "dist_sq_symm",
        universe_params: &["p", "u", "v"],
        statement: affine_params!(
            "forall (law : forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B A)), forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B A)"
        ),
        proof: affine_abs!("fun law => fun A => fun B => law A B"),
    },
    TheoremArtifact {
        name: "dist_sq_zero_iff_eq",
        universe_params: &["p", "u", "v"],
        statement: affine_params!(
            "forall (law : forall (A : PointCarrier), forall (B : PointCarrier), forall (R : Prop), forall (mk : forall (forward : forall (h : @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) zero), @Eq.{p} PointCarrier A B), forall (backward : forall (h : @Eq.{p} PointCarrier A B), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) zero), R), R), forall (A : PointCarrier), forall (B : PointCarrier), forall (R : Prop), forall (mk : forall (forward : forall (h : @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) zero), @Eq.{p} PointCarrier A B), forall (backward : forall (h : @Eq.{p} PointCarrier A B), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) zero), R), R"
        ),
        proof: affine_abs!(
            "fun law => fun A => fun B => fun (R : Prop) => fun (mk : forall (forward : forall (h : @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) zero), @Eq.{p} PointCarrier A B), forall (backward : forall (h : @Eq.{p} PointCarrier A B), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) zero), R) => law A B R mk"
        ),
    },
];

const AFFINE_DERIVE_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "vec_add_comm_from_vector_args",
        universe_params: &["u", "v"],
        statement: abstract_vector_space_params!(
            "forall (vector_args : @VectorSpaceLawArgs.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul), forall (x : Vector), forall (y : Vector), @Eq.{v} Vector (vadd x y) (vadd y x)"
        ),
        proof: abstract_vector_space_abs!(concat!(
            "fun vector_args => fun x => fun y => ",
            "vector_args (@Eq.{v} Vector (vadd x y) (vadd y x)) ",
            "(fun (vec_sub_def_arg : forall (x : Vector), forall (y : Vector), @Eq.{v} Vector (@vsub.{v} Vector vadd vneg x y) (vadd x (vneg y))) => ",
            "fun (vec_add_assoc_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{v} Vector (vadd (vadd x y) z) (vadd x (vadd y z))) => ",
            "fun (vec_add_comm_arg : forall (x : Vector), forall (y : Vector), @Eq.{v} Vector (vadd x y) (vadd y x)) => ",
            "fun (vec_add_zero_arg : forall (x : Vector), @Eq.{v} Vector (vadd x vzero) x) => ",
            "fun (vec_zero_add_arg : forall (x : Vector), @Eq.{v} Vector (vadd vzero x) x) => ",
            "fun (vec_neg_add_cancel_arg : forall (x : Vector), @Eq.{v} Vector (vadd (vneg x) x) vzero) => ",
            "fun (vec_add_neg_cancel_arg : forall (x : Vector), @Eq.{v} Vector (vadd x (vneg x)) vzero) => ",
            "fun (sub_sub_sub_cancel_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{v} Vector (@vsub.{v} Vector vadd vneg (@vsub.{v} Vector vadd vneg x z) (@vsub.{v} Vector vadd vneg y z)) (@vsub.{v} Vector vadd vneg x y)) => ",
            "fun (vec_sub_self_arg : forall (x : Vector), @Eq.{v} Vector (@vsub.{v} Vector vadd vneg x x) vzero) => ",
            "fun (vec_sub_zero_arg : forall (x : Vector), @Eq.{v} Vector (@vsub.{v} Vector vadd vneg x vzero) x) => ",
            "fun (vec_add_left_cancel_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), forall (h : @Eq.{v} Vector (vadd x y) (vadd x z)), @Eq.{v} Vector y z) => ",
            "fun (smul_add_arg : forall (a : Scalar), forall (b : Scalar), forall (x : Vector), @Eq.{v} Vector (smul (add a b) x) (vadd (smul a x) (smul b x))) => ",
            "fun (add_smul_arg : forall (a : Scalar), forall (x : Vector), forall (y : Vector), @Eq.{v} Vector (smul a (vadd x y)) (vadd (smul a x) (smul a y))) => ",
            "fun (one_smul_arg : forall (x : Vector), @Eq.{v} Vector (smul one x) x) => ",
            "fun (mul_smul_arg : forall (a : Scalar), forall (b : Scalar), forall (x : Vector), @Eq.{v} Vector (smul (mul a b) x) (smul a (smul b x))) => ",
            "fun (zero_smul_arg : forall (x : Vector), @Eq.{v} Vector (smul zero x) vzero) => ",
            "fun (smul_zero_arg : forall (a : Scalar), @Eq.{v} Vector (smul a vzero) vzero) => ",
            "fun (neg_smul_arg : forall (a : Scalar), forall (x : Vector), @Eq.{v} Vector (smul (neg a) x) (vneg (smul a x))) => ",
            "fun (smul_neg_arg : forall (a : Scalar), forall (x : Vector), @Eq.{v} Vector (smul a (vneg x)) (vneg (smul a x))) => ",
            "fun (vec_sub_eq_add_neg_arg : forall (x : Vector), forall (y : Vector), @Eq.{v} Vector (@vsub.{v} Vector vadd vneg x y) (vadd x (vneg y))) => ",
            "fun (sub_add_sub_cancel_left_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{v} Vector (vadd (@vsub.{v} Vector vadd vneg x z) (@vsub.{v} Vector vadd vneg z y)) (@vsub.{v} Vector vadd vneg x y)) => ",
            "fun (linear_comb2_ext_arg : forall (a : Scalar), forall (x : Vector), forall (b : Scalar), forall (y : Vector), @Eq.{v} Vector (@linear_comb2.{u,v} Scalar Vector vadd smul a x b y) (vadd (smul a x) (smul b y))) => ",
            "fun (linear_comb3_ext_arg : forall (a : Scalar), forall (x : Vector), forall (b : Scalar), forall (y : Vector), forall (c : Scalar), forall (z : Vector), @Eq.{v} Vector (@linear_comb3.{u,v} Scalar Vector vadd smul a x b y c z) (vadd (vadd (smul a x) (smul b y)) (smul c z))) => vec_add_comm_arg x y)"
        )),
    },
    TheoremArtifact {
        name: "disp_reverse_from_affine_args",
        universe_params: &["p", "u", "v"],
        statement: affine_params!(
            "forall (affine_args : @AffineLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op), forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op B A) (vneg (@disp.{p,v} PointCarrier Vector disp_op A B))"
        ),
        proof: affine_abs!(concat!(
            "fun affine_args => fun A => fun B => ",
            "affine_args (@Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op B A) (vneg (@disp.{p,v} PointCarrier Vector disp_op A B))) ",
            "(fun (disp_self_arg : forall (A : PointCarrier), @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op A A) vzero) => ",
            "fun (disp_reverse_arg : forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op B A) (vneg (@disp.{p,v} PointCarrier Vector disp_op A B))) => ",
            "fun (disp_comp_arg : forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op A C) (vadd (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op B C))) => ",
            "fun (point_ext_arg : forall (A : PointCarrier), forall (B : PointCarrier), forall (h : @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op A B) vzero), @Eq.{p} PointCarrier A B) => ",
            "fun (dist_sq_symm_arg : forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B A)) => ",
            "fun (dist_sq_zero_iff_eq_arg : forall (A : PointCarrier), forall (B : PointCarrier), forall (R : Prop), forall (mk : forall (forward : forall (h : @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) zero), @Eq.{p} PointCarrier A B), forall (backward : forall (h : @Eq.{p} PointCarrier A B), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) zero), R), R) => disp_reverse_arg A B)"
        )),
    },
    TheoremArtifact {
        name: "disp_comp_from_affine_args",
        universe_params: &["p", "u", "v"],
        statement: affine_params!(
            "forall (affine_args : @AffineLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op A C) (vadd (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op B C))"
        ),
        proof: affine_abs!(concat!(
            "fun affine_args => fun A => fun B => fun C => ",
            "affine_args (@Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op A C) (vadd (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op B C))) ",
            "(fun (disp_self_arg : forall (A : PointCarrier), @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op A A) vzero) => ",
            "fun (disp_reverse_arg : forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op B A) (vneg (@disp.{p,v} PointCarrier Vector disp_op A B))) => ",
            "fun (disp_comp_arg : forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op A C) (vadd (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op B C))) => ",
            "fun (point_ext_arg : forall (A : PointCarrier), forall (B : PointCarrier), forall (h : @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op A B) vzero), @Eq.{p} PointCarrier A B) => ",
            "fun (dist_sq_symm_arg : forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B A)) => ",
            "fun (dist_sq_zero_iff_eq_arg : forall (A : PointCarrier), forall (B : PointCarrier), forall (R : Prop), forall (mk : forall (forward : forall (h : @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) zero), @Eq.{p} PointCarrier A B), forall (backward : forall (h : @Eq.{p} PointCarrier A B), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) zero), R), R) => disp_comp_arg A B C)"
        )),
    },
    TheoremArtifact {
        name: "dist_sq_points_def_from_args",
        universe_params: &["p", "u", "v"],
        statement: affine_params!(
            "forall (affine_args : @AffineLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op), forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@normSq.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B))"
        ),
        proof: affine_abs!(
            "fun affine_args => fun A => fun B => @Eq.refl.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B)"
        ),
    },
    TheoremArtifact {
        name: "hypotenuse_vector_eq_neg_left_add_right_from_args",
        universe_params: &["p", "u", "v"],
        statement: affine_params!(
            "forall (affine_args : @AffineLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op B C) (vadd (vneg (@disp.{p,v} PointCarrier Vector disp_op A B)) (@disp.{p,v} PointCarrier Vector disp_op A C))"
        ),
        proof: affine_abs!(concat!(
            "fun affine_args => fun A => fun B => fun C => ",
            "@Eq.rec.{v,0} Vector (vadd (@disp.{p,v} PointCarrier Vector disp_op B A) (@disp.{p,v} PointCarrier Vector disp_op A C)) ",
            "(fun (z : Vector) => fun (hz : @Eq.{v} Vector (vadd (@disp.{p,v} PointCarrier Vector disp_op B A) (@disp.{p,v} PointCarrier Vector disp_op A C)) z) => @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op B C) z) ",
            "(@disp_comp_from_affine_args.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op affine_args B A C) ",
            "(vadd (vneg (@disp.{p,v} PointCarrier Vector disp_op A B)) (@disp.{p,v} PointCarrier Vector disp_op A C)) ",
            "(@Eq.rec.{v,0} Vector (@disp.{p,v} PointCarrier Vector disp_op B A) ",
            "(fun (q : Vector) => fun (hq : @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op B A) q) => @Eq.{v} Vector (vadd (@disp.{p,v} PointCarrier Vector disp_op B A) (@disp.{p,v} PointCarrier Vector disp_op A C)) (vadd q (@disp.{p,v} PointCarrier Vector disp_op A C))) ",
            "(@Eq.refl.{v} Vector (vadd (@disp.{p,v} PointCarrier Vector disp_op B A) (@disp.{p,v} PointCarrier Vector disp_op A C))) ",
            "(vneg (@disp.{p,v} PointCarrier Vector disp_op A B)) ",
            "(@disp_reverse_from_affine_args.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op affine_args A B))"
        )),
    },
    TheoremArtifact {
        name: "hypotenuse_vector_eq_sub_legs_from_args",
        universe_params: &["p", "u", "v"],
        statement: affine_params!(
            "forall (vector_args : @VectorSpaceLawArgs.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul), forall (affine_args : @AffineLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op B C) (@vsub.{v} Vector vadd vneg (@disp.{p,v} PointCarrier Vector disp_op A C) (@disp.{p,v} PointCarrier Vector disp_op A B))"
        ),
        proof: affine_abs!(concat!(
            "fun vector_args => fun affine_args => fun A => fun B => fun C => ",
            "@Eq.rec.{v,0} Vector (vadd (vneg (@disp.{p,v} PointCarrier Vector disp_op A B)) (@disp.{p,v} PointCarrier Vector disp_op A C)) ",
            "(fun (z : Vector) => fun (hz : @Eq.{v} Vector (vadd (vneg (@disp.{p,v} PointCarrier Vector disp_op A B)) (@disp.{p,v} PointCarrier Vector disp_op A C)) z) => @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op B C) z) ",
            "(@hypotenuse_vector_eq_neg_left_add_right_from_args.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op affine_args A B C) ",
            "(@vsub.{v} Vector vadd vneg (@disp.{p,v} PointCarrier Vector disp_op A C) (@disp.{p,v} PointCarrier Vector disp_op A B)) ",
            "(@vec_add_comm_from_vector_args.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul vector_args (vneg (@disp.{p,v} PointCarrier Vector disp_op A B)) (@disp.{p,v} PointCarrier Vector disp_op A C))"
        )),
    },
    TheoremArtifact {
        name: "dist_sq_hypotenuse_norm_neg_left_add_right_from_args",
        universe_params: &["p", "u", "v"],
        statement: affine_params!(
            "forall (affine_args : @AffineLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C) (@normSq.{u,v} Scalar Vector inner (vadd (vneg (@disp.{p,v} PointCarrier Vector disp_op A B)) (@disp.{p,v} PointCarrier Vector disp_op A C)))"
        ),
        proof: affine_abs!(concat!(
            "fun affine_args => fun A => fun B => fun C => ",
            "@Eq.rec.{v,0} Vector (@disp.{p,v} PointCarrier Vector disp_op B C) ",
            "(fun (z : Vector) => fun (hz : @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op B C) z) => @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C) (@normSq.{u,v} Scalar Vector inner z)) ",
            "(@dist_sq_points_def_from_args.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op affine_args B C) ",
            "(vadd (vneg (@disp.{p,v} PointCarrier Vector disp_op A B)) (@disp.{p,v} PointCarrier Vector disp_op A C)) ",
            "(@hypotenuse_vector_eq_neg_left_add_right_from_args.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op affine_args A B C)"
        )),
    },
    TheoremArtifact {
        name: "dist_sq_hypotenuse_norm_sub_legs_from_args",
        universe_params: &["p", "u", "v"],
        statement: affine_params!(
            "forall (vector_args : @VectorSpaceLawArgs.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul), forall (affine_args : @AffineLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C) (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg (@disp.{p,v} PointCarrier Vector disp_op A C) (@disp.{p,v} PointCarrier Vector disp_op A B)))"
        ),
        proof: affine_abs!(concat!(
            "fun vector_args => fun affine_args => fun A => fun B => fun C => ",
            "@Eq.rec.{v,0} Vector (@disp.{p,v} PointCarrier Vector disp_op B C) ",
            "(fun (z : Vector) => fun (hz : @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op B C) z) => @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C) (@normSq.{u,v} Scalar Vector inner z)) ",
            "(@dist_sq_points_def_from_args.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op affine_args B C) ",
            "(@vsub.{v} Vector vadd vneg (@disp.{p,v} PointCarrier Vector disp_op A C) (@disp.{p,v} PointCarrier Vector disp_op A B)) ",
            "(@hypotenuse_vector_eq_sub_legs_from_args.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op vector_args affine_args A B C)"
        )),
    },
];

const ABSTRACT_RIGHT_TRIANGLE_DEFINITIONS: &[DefinitionArtifact] = &[
    DefinitionArtifact {
        name: "Perp",
        universe_params: &["u", "v"],
        ty: concat!(
            "forall (Scalar : Sort u), ",
            "forall (zero : Scalar), ",
            "forall (Vector : Sort v), ",
            "forall (inner : forall (x : Vector), forall (y : Vector), Scalar), ",
            "forall (x : Vector), forall (y : Vector), Prop"
        ),
        value:
            "fun Scalar => fun zero => fun Vector => fun inner => fun x => fun y => @PerpVec.{u,v} Scalar zero Vector inner x y",
    },
    DefinitionArtifact {
        name: "RightTriangle",
        universe_params: &["p", "u", "v"],
        ty: concat!(
            "forall (Scalar : Sort u), ",
            "forall (zero : Scalar), ",
            "forall (Vector : Sort v), ",
            "forall (inner : forall (x : Vector), forall (y : Vector), Scalar), ",
            "forall (PointCarrier : Sort p), ",
            "forall (disp_op : forall (A : PointCarrier), forall (B : PointCarrier), Vector), ",
            "forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), Prop"
        ),
        value:
            "fun Scalar => fun zero => fun Vector => fun inner => fun PointCarrier => fun disp_op => fun A => fun B => fun C => @Perp.{u,v} Scalar zero Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C)",
    },
    DefinitionArtifact {
        name: "AngleRight",
        universe_params: &["p"],
        ty: concat!(
            "forall (PointCarrier : Sort p), ",
            "forall (angle_right_rel : forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), Prop), ",
            "forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), Prop"
        ),
        value:
            "fun PointCarrier => fun angle_right_rel => fun A => fun B => fun C => angle_right_rel A B C",
    },
    DefinitionArtifact {
        name: "Area2",
        universe_params: &["p", "u"],
        ty: concat!(
            "forall (Scalar : Sort u), ",
            "forall (PointCarrier : Sort p), ",
            "forall (area2_op : forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), Scalar), ",
            "forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), Scalar"
        ),
        value:
            "fun Scalar => fun PointCarrier => fun area2_op => fun A => fun B => fun C => area2_op A B C",
    },
    DefinitionArtifact {
        name: "FootOnHypotenuse",
        universe_params: &["p"],
        ty: concat!(
            "forall (PointCarrier : Sort p), ",
            "forall (foot_rel : forall (H : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), Prop), ",
            "forall (H : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), Prop"
        ),
        value:
            "fun PointCarrier => fun foot_rel => fun H => fun B => fun C => foot_rel H B C",
    },
];

const ABSTRACT_RIGHT_TRIANGLE_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "perp_iff_dot_eq_zero",
        universe_params: &["p", "u", "v"],
        statement: abstract_right_triangle_params!(
            "forall (x : Vector), forall (y : Vector), forall (R : Prop), forall (mk : forall (forward : forall (h : @Perp.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), forall (backward : forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @Perp.{u,v} Scalar zero Vector inner x y), R), R"
        ),
        proof: abstract_right_triangle_abs!(
            "fun x => fun y => fun (R : Prop) => fun (mk : forall (forward : forall (h : @Perp.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), forall (backward : forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @Perp.{u,v} Scalar zero Vector inner x y), R) => mk (fun (h : @Perp.{u,v} Scalar zero Vector inner x y) => h) (fun (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero) => h)"
        ),
    },
    TheoremArtifact {
        name: "perp_symm",
        universe_params: &["p", "u", "v"],
        statement: abstract_right_triangle_params!(
            "forall (law : forall (x : Vector), forall (y : Vector), forall (h : @Perp.{u,v} Scalar zero Vector inner x y), @Perp.{u,v} Scalar zero Vector inner y x), forall (x : Vector), forall (y : Vector), forall (h : @Perp.{u,v} Scalar zero Vector inner x y), @Perp.{u,v} Scalar zero Vector inner y x"
        ),
        proof: abstract_right_triangle_abs!("fun law => fun x => fun y => fun h => law x y h"),
    },
    TheoremArtifact {
        name: "right_triangle_legs_perp",
        universe_params: &["p", "u", "v"],
        statement: abstract_right_triangle_params!(
            "forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), forall (h : @RightTriangle.{p,u,v} Scalar zero Vector inner PointCarrier disp_op A B C), @Perp.{u,v} Scalar zero Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C)"
        ),
        proof: abstract_right_triangle_abs!("fun A => fun B => fun C => fun h => h"),
    },
    TheoremArtifact {
        name: "pythagorean_distance_sq_general",
        universe_params: &["p", "u", "v"],
        statement: abstract_right_triangle_params!(
            "forall (pythagorean_sq_target : forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), forall (h : @RightTriangle.{p,u,v} Scalar zero Vector inner PointCarrier disp_op A B C), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C) (add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C))), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), forall (h : @RightTriangle.{p,u,v} Scalar zero Vector inner PointCarrier disp_op A B C), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C) (add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C))"
        ),
        proof: abstract_right_triangle_abs!(
            "fun pythagorean_sq_target => fun A => fun B => fun C => fun h => pythagorean_sq_target A B C h"
        ),
    },
    TheoremArtifact {
        name: "law_of_cosines_general",
        universe_params: &["p", "u", "v"],
        statement: abstract_right_triangle_params!(
            "forall (law : forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C) (sub (add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C)) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C))))), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C) (sub (add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C)) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C))))"
        ),
        proof: abstract_right_triangle_abs!("fun law => fun A => fun B => fun C => law A B C"),
    },
    TheoremArtifact {
        name: "right_triangle_area_general",
        universe_params: &["p", "u", "v"],
        statement: abstract_right_triangle_params!(
            "forall (area2_op : forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), Scalar), forall (area_target : forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), forall (h : @RightTriangle.{p,u,v} Scalar zero Vector inner PointCarrier disp_op A B C), @Eq.{u} Scalar (@sq.{u} Scalar mul (@Area2.{p,u} Scalar PointCarrier area2_op A B C)) (mul (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C))), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), forall (h : @RightTriangle.{p,u,v} Scalar zero Vector inner PointCarrier disp_op A B C), @Eq.{u} Scalar (@sq.{u} Scalar mul (@Area2.{p,u} Scalar PointCarrier area2_op A B C)) (mul (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C))"
        ),
        proof: abstract_right_triangle_abs!(
            "fun area2_op => fun area_target => fun A => fun B => fun C => fun h => area_target A B C h"
        ),
    },
    TheoremArtifact {
        name: "median_to_hypotenuse_general",
        universe_params: &["p", "u", "v"],
        statement: abstract_right_triangle_params!(
            "forall (midpoint_op : forall (A : PointCarrier), forall (B : PointCarrier), PointCarrier), forall (median_target : forall (M : PointCarrier), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), forall (h : @RightTriangle.{p,u,v} Scalar zero Vector inner PointCarrier disp_op A B C), forall (hm : @Eq.{p} PointCarrier M (@midpoint.{p} PointCarrier midpoint_op B C)), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A M) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B M)), forall (M : PointCarrier), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), forall (h : @RightTriangle.{p,u,v} Scalar zero Vector inner PointCarrier disp_op A B C), forall (hm : @Eq.{p} PointCarrier M (@midpoint.{p} PointCarrier midpoint_op B C)), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A M) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B M)"
        ),
        proof: abstract_right_triangle_abs!(
            "fun midpoint_op => fun median_target => fun M => fun A => fun B => fun C => fun h => fun hm => median_target M A B C h hm"
        ),
    },
];

const ABSTRACT_RIGHT_TRIANGLE_DERIVE_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "neg_zero_from_ring_args",
        universe_params: &["u"],
        statement: abstract_ring_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), @Eq.{u} Scalar (neg zero) zero"
        ),
        proof: abstract_ring_abs!(concat!(
            "fun ring_args => ",
            "ring_args (@Eq.{u} Scalar (neg zero) zero) ",
            "(fun (sub_eq_add_neg_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub a b) (add a (neg b))) => ",
            "fun (add_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add a b) c) (add a (add b c))) => ",
            "fun (add_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add a b) (add b a)) => ",
            "fun (add_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (add a zero) a) => ",
            "fun (zero_add_arg : forall (a : Scalar), @Eq.{u} Scalar (add zero a) a) => ",
            "fun (neg_add_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add (neg a) a) zero) => ",
            "fun (add_neg_cancel_arg : forall (a : Scalar), @Eq.{u} Scalar (add a (neg a)) zero) => ",
            "fun (sub_self_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a a) zero) => ",
            "fun (mul_assoc_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (mul a b) c) (mul a (mul b c))) => ",
            "fun (mul_comm_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (mul a b) (mul b a)) => ",
            "fun (mul_one_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a one) a) => ",
            "fun (one_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul one a) a) => ",
            "fun (left_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul a (add b c)) (add (mul a b) (mul a c))) => ",
            "fun (right_distrib_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (mul (add a b) c) (add (mul a c) (mul b c))) => ",
            "fun (mul_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (mul a zero) zero) => ",
            "fun (zero_mul_arg : forall (a : Scalar), @Eq.{u} Scalar (mul zero a) zero) => ",
            "fun (add_left_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add a b) (add a c)), @Eq.{u} Scalar b c) => ",
            "fun (ring_normalize_add_mul3_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (add (add (mul a b) (mul b c)) (mul a c)) (add (add (mul a b) (mul a c)) (mul b c))) => ",
            "fun (add_right_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (h : @Eq.{u} Scalar (add b a) (add c a)), @Eq.{u} Scalar b c) => ",
            "fun (neg_neg_arg : forall (a : Scalar), @Eq.{u} Scalar (neg (neg a)) a) => ",
            "fun (sub_zero_arg : forall (a : Scalar), @Eq.{u} Scalar (sub a zero) a) => ",
            "fun (zero_sub_arg : forall (a : Scalar), @Eq.{u} Scalar (sub zero a) (neg a)) => ",
            "fun (sub_add_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (add (sub a b) b) a) => ",
            "fun (add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), @Eq.{u} Scalar (sub (add a b) b) a) => ",
            "fun (sub_add_sub_cancel_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), @Eq.{u} Scalar (sub (sub a c) (sub b c)) (sub a b)) => ",
            "@Eq.rec.{u,0} Scalar (add (neg zero) zero) ",
            "(fun (y : Scalar) => fun (hy : @Eq.{u} Scalar (add (neg zero) zero) y) => @Eq.{u} Scalar y zero) ",
            "(neg_add_cancel_arg zero) ",
            "(neg zero) ",
            "(add_zero_arg (neg zero)))"
        )),
    },
    TheoremArtifact {
        name: "right_triangle_legs_perp_vec_from_rt",
        universe_params: &["p", "u", "v"],
        statement: abstract_right_triangle_params!(
            "forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), forall (h : @RightTriangle.{p,u,v} Scalar zero Vector inner PointCarrier disp_op A B C), @PerpVec.{u,v} Scalar zero Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C)"
        ),
        proof: abstract_right_triangle_abs!("fun A => fun B => fun C => fun h => h"),
    },
    TheoremArtifact {
        name: "right_triangle_legs_dot_zero_from_rt",
        universe_params: &["p", "u", "v"],
        statement: abstract_right_triangle_params!(
            "forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), forall (h : @RightTriangle.{p,u,v} Scalar zero Vector inner PointCarrier disp_op A B C), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C)) zero"
        ),
        proof: abstract_right_triangle_abs!("fun A => fun B => fun C => fun h => h"),
    },
    TheoremArtifact {
        name: "right_triangle_neg_left_dot_zero_from_rt",
        universe_params: &["p", "u", "v"],
        statement: abstract_right_triangle_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), forall (h : @RightTriangle.{p,u,v} Scalar zero Vector inner PointCarrier disp_op A B C), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vneg (@disp.{p,v} PointCarrier Vector disp_op A B)) (@disp.{p,v} PointCarrier Vector disp_op A C)) zero"
        ),
        proof: abstract_right_triangle_abs!(concat!(
            "fun ring_args => fun inner_args => fun A => fun B => fun C => fun h => ",
            "@Eq.rec.{u,0} Scalar (neg zero) ",
            "(fun (z : Scalar) => fun (hz : @Eq.{u} Scalar (neg zero) z) => @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vneg (@disp.{p,v} PointCarrier Vector disp_op A B)) (@disp.{p,v} PointCarrier Vector disp_op A C)) z) ",
            "(@Eq.rec.{u,0} Scalar (neg (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C))) ",
            "(fun (z : Scalar) => fun (hz : @Eq.{u} Scalar (neg (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C))) z) => @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vneg (@disp.{p,v} PointCarrier Vector disp_op A B)) (@disp.{p,v} PointCarrier Vector disp_op A C)) z) ",
            "(@dot_neg_left_from_inner_args.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner inner_args (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C)) ",
            "(neg zero) ",
            "(@Eq.rec.{u,0} Scalar (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C)) ",
            "(fun (q : Scalar) => fun (hq : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C)) q) => @Eq.{u} Scalar (neg (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C))) (neg q)) ",
            "(@Eq.refl.{u} Scalar (neg (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C)))) ",
            "zero ",
            "(@right_triangle_legs_dot_zero_from_rt.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op A B C h))) ",
            "zero ",
            "(@neg_zero_from_ring_args.{u} Scalar zero one add neg sub mul ring_args)"
        )),
    },
    TheoremArtifact {
        name: "right_triangle_neg_left_perp_vec_from_rt",
        universe_params: &["p", "u", "v"],
        statement: abstract_right_triangle_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), forall (h : @RightTriangle.{p,u,v} Scalar zero Vector inner PointCarrier disp_op A B C), @PerpVec.{u,v} Scalar zero Vector inner (vneg (@disp.{p,v} PointCarrier Vector disp_op A B)) (@disp.{p,v} PointCarrier Vector disp_op A C)"
        ),
        proof: abstract_right_triangle_abs!(
            "fun ring_args => fun inner_args => fun A => fun B => fun C => fun h => @right_triangle_neg_left_dot_zero_from_rt.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op ring_args inner_args A B C h"
        ),
    },
    TheoremArtifact {
        name: "right_triangle_affine_additive_perp_bridge_from_rt",
        universe_params: &["p", "u", "v"],
        statement: abstract_right_triangle_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (affine_args : @AffineLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), forall (h : @RightTriangle.{p,u,v} Scalar zero Vector inner PointCarrier disp_op A B C), forall (R : Prop), forall (mk : forall (hypotenuse_orientation : @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op B C) (vadd (vneg (@disp.{p,v} PointCarrier Vector disp_op A B)) (@disp.{p,v} PointCarrier Vector disp_op A C))), forall (perp_premise : @PerpVec.{u,v} Scalar zero Vector inner (vneg (@disp.{p,v} PointCarrier Vector disp_op A B)) (@disp.{p,v} PointCarrier Vector disp_op A C)), R), R"
        ),
        proof: abstract_right_triangle_abs!(
            "fun ring_args => fun inner_args => fun affine_args => fun A => fun B => fun C => fun h => fun R => fun mk => mk (@hypotenuse_vector_eq_neg_left_add_right_from_args.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op affine_args A B C) (@right_triangle_neg_left_perp_vec_from_rt.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op ring_args inner_args A B C h)"
        ),
    },
];

const ABSTRACT_METRIC_DEFINITIONS: &[DefinitionArtifact] = &[
    DefinitionArtifact {
        name: "dist",
        universe_params: &["p", "u", "v"],
        ty: concat!(
            "forall (Scalar : Sort u), ",
            "forall (sqrt_fn : forall (a : Scalar), Scalar), ",
            "forall (Vector : Sort v), ",
            "forall (inner : forall (x : Vector), forall (y : Vector), Scalar), ",
            "forall (PointCarrier : Sort p), ",
            "forall (disp_op : forall (A : PointCarrier), forall (B : PointCarrier), Vector), ",
            "forall (A : PointCarrier), forall (B : PointCarrier), Scalar"
        ),
        value:
            "fun Scalar => fun sqrt_fn => fun Vector => fun inner => fun PointCarrier => fun disp_op => fun A => fun B => @sqrt.{u} Scalar sqrt_fn (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B)",
    },
    DefinitionArtifact {
        name: "MetricSpaceLawArgs",
        universe_params: &["p", "u", "v"],
        ty: abstract_metric_params!("Prop"),
        value: abstract_metric_abs!(concat!(
            "forall (P : Prop), forall (mk : ",
            "forall (dist_def_law : forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{u} Scalar (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) (@sqrt.{u} Scalar sqrt_fn (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B))), ",
            "forall (dist_nonneg_law : forall (A : PointCarrier), forall (B : PointCarrier), le_rel zero (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B)), ",
            "forall (distance_symm_law : forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{u} Scalar (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op B A)), ",
            "forall (distance_zero_iff_eq_law : forall (A : PointCarrier), forall (B : PointCarrier), forall (R : Prop), forall (mk : forall (forward : forall (h : @Eq.{u} Scalar (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) zero), @Eq.{p} PointCarrier A B), forall (backward : forall (h : @Eq.{p} PointCarrier A B), @Eq.{u} Scalar (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) zero), R), R), ",
            "forall (triangle_inequality_law : forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), le_rel (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A C) (add (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op B C))), P), P"
        )),
    },
    DefinitionArtifact {
        name: "Ball",
        universe_params: &["p", "u", "v"],
        ty: abstract_metric_params!(
            "forall (center : PointCarrier), forall (radius : Scalar), forall (x : PointCarrier), Prop"
        ),
        value: abstract_metric_abs!(
            "fun center => fun radius => fun x => le_rel (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op center x) radius"
        ),
    },
];

const ABSTRACT_METRIC_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "dist_def",
        universe_params: &["p", "u", "v"],
        statement: abstract_metric_params!(
            "forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{u} Scalar (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) (@sqrt.{u} Scalar sqrt_fn (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B))"
        ),
        proof: abstract_metric_abs!(
            "fun A => fun B => @Eq.refl.{u} Scalar (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B)"
        ),
    },
    TheoremArtifact {
        name: "point_dist_sq_nonneg_from_inner_args",
        universe_params: &["p", "u", "v"],
        statement: abstract_metric_params!(
            "forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (A : PointCarrier), forall (B : PointCarrier), le_rel zero (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B)"
        ),
        proof: abstract_metric_abs!(concat!(
            "fun inner_args => fun A => fun B => ",
            "inner_args (le_rel zero (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B)) ",
            "(fun (dot_comm_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner y x)) => ",
            "fun (dot_add_left_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vadd x y) z) (add (@dot.{u,v} Scalar Vector inner x z) (@dot.{u,v} Scalar Vector inner y z))) => ",
            "fun (dot_add_right_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (vadd y z)) (add (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner x z))) => ",
            "fun (dot_neg_left_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (vneg x) y) (neg (@dot.{u,v} Scalar Vector inner x y))) => ",
            "fun (dot_neg_right_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (vneg y)) (neg (@dot.{u,v} Scalar Vector inner x y))) => ",
            "fun (dot_sub_left_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y) z) (sub (@dot.{u,v} Scalar Vector inner x z) (@dot.{u,v} Scalar Vector inner y z))) => ",
            "fun (dot_sub_right_arg : forall (x : Vector), forall (y : Vector), forall (z : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x (@vsub.{v} Vector vadd vneg y z)) (sub (@dot.{u,v} Scalar Vector inner x y) (@dot.{u,v} Scalar Vector inner x z))) => ",
            "fun (norm_sq_def_arg : forall (x : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) (@dot.{u,v} Scalar Vector inner x x)) => ",
            "fun (dist_sq_def_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@distSq.{u,v} Scalar Vector vadd vneg inner x y) (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg y x))) => ",
            "fun (dot_self_eq_norm_sq_arg : forall (x : Vector), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x x) (@normSq.{u,v} Scalar Vector inner x)) => ",
            "fun (norm_sq_add_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (add (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (norm_sq_sub_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (sub (@normSq.{u,v} Scalar Vector inner x) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y))) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (inner_field13_arg : forall (x : Vector), forall (y : Vector), forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (inner_field14_arg : forall (x : Vector), forall (y : Vector), forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (norm_sq_nonneg_arg : forall (x : Vector), le_rel zero (@normSq.{u,v} Scalar Vector inner x)) => ",
            "fun (parallelogram_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (add (@normSq.{u,v} Scalar Vector inner (vadd x y)) (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y))) (add (mul (@two.{u} Scalar one add) (@normSq.{u,v} Scalar Vector inner x)) (mul (@two.{u} Scalar one add) (@normSq.{u,v} Scalar Vector inner y)))) => ",
            "fun (polarization_identity_arg : forall (x : Vector), forall (y : Vector), @Eq.{u} Scalar (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y)) (sub (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y)))) => ",
            "fun (perp_vec_iff_dot_eq_zero_arg : forall (x : Vector), forall (y : Vector), forall (R : Prop), forall (mk : forall (forward : forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), forall (backward : forall (h : @Eq.{u} Scalar (@dot.{u,v} Scalar Vector inner x y) zero), @PerpVec.{u,v} Scalar zero Vector inner x y), R), R) => ",
            "fun (perp_vec_symm_arg : forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @PerpVec.{u,v} Scalar zero Vector inner y x) => ",
            "fun (norm_sq_zero_iff_arg : forall (x : Vector), forall (R : Prop), forall (mk : forall (forward : forall (h : @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) zero), @Eq.{v} Vector x vzero), forall (backward : forall (h : @Eq.{v} Vector x vzero), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner x) zero), R), R) => ",
            "fun (dist_sq_nonneg_arg : forall (x : Vector), forall (y : Vector), le_rel zero (@distSq.{u,v} Scalar Vector vadd vneg inner x y)) => ",
            "fun (inner_field23_arg : forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vadd x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (inner_field24_arg : forall (x : Vector), forall (y : Vector), forall (h : @PerpVec.{u,v} Scalar zero Vector inner x y), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (@vsub.{v} Vector vadd vneg x y)) (add (@normSq.{u,v} Scalar Vector inner x) (@normSq.{u,v} Scalar Vector inner y))) => ",
            "fun (quadratic_norm_nonneg_arg : forall (x : Vector), forall (y : Vector), forall (t : Scalar), le_rel zero (add (add (mul (@normSq.{u,v} Scalar Vector inner x) (@sq.{u} Scalar mul t)) (mul (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner x y)) t)) (@normSq.{u,v} Scalar Vector inner y))) => norm_sq_nonneg_arg (@disp.{p,v} PointCarrier Vector disp_op A B))"
        )),
    },
    TheoremArtifact {
        name: "square_dist_eq_dist_sq_from_law_packages",
        universe_params: &["p", "u", "v"],
        statement: abstract_metric_params!(
            "forall (ordered_args : @OrderedFieldLawArgs.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{u} Scalar (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B)) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B)"
        ),
        proof: abstract_metric_abs!(concat!(
            "fun ordered_args => fun inner_args => fun A => fun B => ",
            "ordered_args (@Eq.{u} Scalar (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B)) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B)) ",
            "(fun (le_refl_arg : forall (a : Scalar), le_rel a a) => ",
            "fun (le_trans_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (hab : le_rel a b), forall (hbc : le_rel b c), le_rel a c) => ",
            "fun (add_nonneg_arg : forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), le_rel zero (add a b)) => ",
            "fun (mul_nonneg_arg : forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), le_rel zero (mul a b)) => ",
            "fun (square_nonneg_arg : forall (a : Scalar), le_rel zero (@sq.{u} Scalar mul a)) => ",
            "fun (sqrt_nonneg_arg : forall (a : Scalar), le_rel zero (sqrt_fn a)) => ",
            "fun (sqrt_square_of_nonneg_arg : forall (a : Scalar), forall (ha : le_rel zero a), @Eq.{u} Scalar (sqrt_fn (@sq.{u} Scalar mul a)) a) => ",
            "fun (sqrt_mul_self_arg : forall (a : Scalar), forall (ha : le_rel zero a), @Eq.{u} Scalar (sqrt_fn (mul a a)) a) => ",
            "fun (eq_of_square_eq_square_nonneg_arg : forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hsq : @Eq.{u} Scalar (@sq.{u} Scalar mul a) (@sq.{u} Scalar mul b)), @Eq.{u} Scalar a b) => ",
            "fun (add_le_add_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (d : Scalar), forall (hab : le_rel a b), forall (hcd : le_rel c d), le_rel (add a c) (add b d)) => ",
            "fun (mul_le_mul_nonneg_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (d : Scalar), forall (ha : le_rel zero a), forall (hab : le_rel a b), forall (hc : le_rel zero c), forall (hcd : le_rel c d), le_rel (mul a c) (mul b d)) => ",
            "fun (zero_le_two_arg : le_rel zero (@two.{u} Scalar one add)) => ",
            "fun (le_antisymm_arg : forall (a : Scalar), forall (b : Scalar), forall (hab : le_rel a b), forall (hba : le_rel b a), @Eq.{u} Scalar a b) => ",
            "fun (lt_of_le_of_ne_arg : forall (a : Scalar), forall (ha : le_rel zero a), forall (hne : forall (haz : @Eq.{u} Scalar a zero), forall (P : Prop), P), lt_rel zero a) => ",
            "fun (le_of_eq_arg : forall (a : Scalar), forall (b : Scalar), forall (hab : @Eq.{u} Scalar a b), forall (P : Prop), forall (mk : forall (hab_le : le_rel a b), forall (hba_le : le_rel b a), P), P) => ",
            "fun (sqrt_sq_arg : forall (a : Scalar), forall (ha : le_rel zero a), @Eq.{u} Scalar (@sq.{u} Scalar mul (sqrt_fn a)) a) => ",
            "fun (sq_eq_zero_iff_arg : forall (a : Scalar), forall (R : Prop), forall (mk : forall (forward : forall (hsqz : @Eq.{u} Scalar (@sq.{u} Scalar mul a) zero), @Eq.{u} Scalar a zero), forall (backward : forall (haz : @Eq.{u} Scalar a zero), @Eq.{u} Scalar (@sq.{u} Scalar mul a) zero), R), R) => ",
            "fun (sum_nonneg_eq_zero_arg : forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hsum : @Eq.{u} Scalar (add a b) zero), forall (R : Prop), forall (mk : forall (haz : @Eq.{u} Scalar a zero), forall (hbz : @Eq.{u} Scalar b zero), R), R) => ",
            "fun (square_completion_bound_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (hquadratic : forall (t : Scalar), le_rel zero (add (add (mul a (@sq.{u} Scalar mul t)) (mul (mul (@two.{u} Scalar one add) b) t)) c)), le_rel (@sq.{u} Scalar mul b) (mul a c)) => ",
            "fun (le_of_sq_le_sq_nonneg_arg : forall (a : Scalar), forall (b : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hsq : le_rel (@sq.{u} Scalar mul a) (@sq.{u} Scalar mul b)), le_rel a b) => ",
            "fun (le_mul_sqrt_of_sq_le_mul_nonneg_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hsq : le_rel (@sq.{u} Scalar mul c) (mul a b)), le_rel c (mul (sqrt_fn a) (sqrt_fn b))) => ",
            "fun (add_two_mul_le_sq_add_sqrt_arg : forall (a : Scalar), forall (b : Scalar), forall (c : Scalar), forall (n : Scalar), forall (ha : le_rel zero a), forall (hb : le_rel zero b), forall (hn : @Eq.{u} Scalar n (add (add a (mul (@two.{u} Scalar one add) c)) b)), forall (hc : le_rel c (mul (sqrt_fn a) (sqrt_fn b))), le_rel n (@sq.{u} Scalar mul (add (sqrt_fn a) (sqrt_fn b)))) => ",
            "sqrt_sq_arg (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@point_dist_sq_nonneg_from_inner_args.{p,u,v} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn Vector vzero vadd vneg smul inner PointCarrier disp_op inner_args A B))"
        )),
    },
    TheoremArtifact {
        name: "dist_sq_eq_square_dist_from_law_packages",
        universe_params: &["p", "u", "v"],
        statement: abstract_metric_params!(
            "forall (ordered_args : @OrderedFieldLawArgs.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B))"
        ),
        proof: abstract_metric_abs!(concat!(
            "fun ordered_args => fun inner_args => fun A => fun B => ",
            "@Eq.rec.{u,0} Scalar ",
            "(@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B)) ",
            "(fun (q : Scalar) => fun (hq : @Eq.{u} Scalar (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B)) q) => @Eq.{u} Scalar q (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B))) ",
            "(@Eq.refl.{u} Scalar (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B))) ",
            "(@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) ",
            "(@square_dist_eq_dist_sq_from_law_packages.{p,u,v} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn Vector vzero vadd vneg smul inner PointCarrier disp_op ordered_args inner_args A B)"
        )),
    },
    TheoremArtifact {
        name: "dist_sq_eq_square_dist",
        universe_params: &["p", "u", "v"],
        statement: abstract_metric_params!(
            "forall (ordered_args : @OrderedFieldLawArgs.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B))"
        ),
        proof: abstract_metric_abs!(
            "fun ordered_args => fun inner_args => fun A => fun B => @dist_sq_eq_square_dist_from_law_packages.{p,u,v} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn Vector vzero vadd vneg smul inner PointCarrier disp_op ordered_args inner_args A B"
        ),
    },
    TheoremArtifact {
        name: "dist_sq_points_le_square_sum_dist_from_law_packages",
        universe_params: &["p", "u", "v"],
        statement: abstract_metric_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (ordered_args : @OrderedFieldLawArgs.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn), forall (vector_args : @VectorSpaceLawArgs.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (affine_args : @AffineLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), le_rel (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C) (@sq.{u} Scalar mul (add (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op B C)))"
        ),
        proof: abstract_metric_abs!(concat!(
            "fun ring_args => fun ordered_args => fun vector_args => fun inner_args => fun affine_args => fun A => fun B => fun C => ",
            "@Eq.rec.{u,0} Scalar ",
            "(@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C) ",
            "(fun (q : Scalar) => fun (hq : @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C) q) => forall (hbound : le_rel q (@sq.{u} Scalar mul (add (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op B C)))), le_rel (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C) (@sq.{u} Scalar mul (add (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op B C)))) ",
            "(fun (hbound : le_rel (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C) (@sq.{u} Scalar mul (add (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op B C)))) => hbound) ",
            "(@normSq.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A C)) ",
            "(@dist_sq_points_def_from_args.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op affine_args A C) ",
            "(@Eq.rec.{v,0} Vector ",
            "(@disp.{p,v} PointCarrier Vector disp_op A C) ",
            "(fun (z : Vector) => fun (hz : @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op A C) z) => forall (hbound : le_rel (@normSq.{u,v} Scalar Vector inner z) (@sq.{u} Scalar mul (add (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op B C)))), le_rel (@normSq.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A C)) (@sq.{u} Scalar mul (add (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op B C)))) ",
            "(fun (hbound : le_rel (@normSq.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A C)) (@sq.{u} Scalar mul (add (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op B C)))) => hbound) ",
            "(vadd (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op B C)) ",
            "(@disp_comp_from_affine_args.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op affine_args A B C) ",
            "(@norm_sq_add_le_square_sum_norms_from_cauchy.{u,v} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn Vector vzero vadd vneg smul inner ring_args ordered_args vector_args inner_args ",
            "(@disp.{p,v} PointCarrier Vector disp_op A B) ",
            "(@disp.{p,v} PointCarrier Vector disp_op B C)))"
        )),
    },
    TheoremArtifact {
        name: "dist_nonneg_from_ordered_args",
        universe_params: &["p", "u", "v"],
        statement: abstract_metric_params!(
            "forall (ordered_args : @OrderedFieldLawArgs.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn), forall (A : PointCarrier), forall (B : PointCarrier), le_rel zero (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B)"
        ),
        proof: abstract_metric_abs!(concat!(
            "fun ordered_args => fun A => fun B => ",
            ordered_args_elim!(
                "(le_rel zero (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B))",
                "sqrt_nonneg_arg (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B)"
            )
        )),
    },
    TheoremArtifact {
        name: "triangle_inequality_from_law_packages",
        universe_params: &["p", "u", "v"],
        statement: abstract_metric_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (ordered_args : @OrderedFieldLawArgs.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn), forall (vector_args : @VectorSpaceLawArgs.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (affine_args : @AffineLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), le_rel (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A C) (add (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op B C))"
        ),
        proof: abstract_metric_abs!(concat!(
            "fun ring_args => fun ordered_args => fun vector_args => fun inner_args => fun affine_args => fun A => fun B => fun C => ",
            "@sqrt_sum_square_bound_from_ordered_args.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn ordered_args ",
            "(@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A C) ",
            "(@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) ",
            "(@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op B C) ",
            "(@dist_nonneg_from_ordered_args.{p,u,v} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn Vector vzero vadd vneg smul inner PointCarrier disp_op ordered_args A C) ",
            "(@dist_nonneg_from_ordered_args.{p,u,v} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn Vector vzero vadd vneg smul inner PointCarrier disp_op ordered_args A B) ",
            "(@dist_nonneg_from_ordered_args.{p,u,v} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn Vector vzero vadd vneg smul inner PointCarrier disp_op ordered_args B C) ",
            "(@Eq.rec.{u,0} Scalar ",
            "(@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A C)) ",
            "(fun (q : Scalar) => fun (hq : @Eq.{u} Scalar (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A C)) q) => forall (hbound : le_rel q (@sq.{u} Scalar mul (add (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op B C)))), le_rel (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A C)) (@sq.{u} Scalar mul (add (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op B C)))) ",
            "(fun (hbound : le_rel (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A C)) (@sq.{u} Scalar mul (add (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op B C)))) => hbound) ",
            "(@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C) ",
            "(@square_dist_eq_dist_sq_from_law_packages.{p,u,v} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn Vector vzero vadd vneg smul inner PointCarrier disp_op ordered_args inner_args A C) ",
            "(@dist_sq_points_le_square_sum_dist_from_law_packages.{p,u,v} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn Vector vzero vadd vneg smul inner PointCarrier disp_op ring_args ordered_args vector_args inner_args affine_args A B C))"
        )),
    },
    TheoremArtifact {
        name: "dist_nonneg",
        universe_params: &["p", "u", "v"],
        statement: abstract_metric_params!(
            "forall (law : forall (A : PointCarrier), forall (B : PointCarrier), le_rel zero (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B)), forall (A : PointCarrier), forall (B : PointCarrier), le_rel zero (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B)"
        ),
        proof: abstract_metric_abs!("fun law => fun A => fun B => law A B"),
    },
    TheoremArtifact {
        name: "distance_symm",
        universe_params: &["p", "u", "v"],
        statement: abstract_metric_params!(
            "forall (law : forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{u} Scalar (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op B A)), forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{u} Scalar (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op B A)"
        ),
        proof: abstract_metric_abs!("fun law => fun A => fun B => law A B"),
    },
    TheoremArtifact {
        name: "distance_zero_iff_eq",
        universe_params: &["p", "u", "v"],
        statement: abstract_metric_params!(
            "forall (law : forall (A : PointCarrier), forall (B : PointCarrier), forall (R : Prop), forall (mk : forall (forward : forall (h : @Eq.{u} Scalar (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) zero), @Eq.{p} PointCarrier A B), forall (backward : forall (h : @Eq.{p} PointCarrier A B), @Eq.{u} Scalar (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) zero), R), R), forall (A : PointCarrier), forall (B : PointCarrier), forall (R : Prop), forall (mk : forall (forward : forall (h : @Eq.{u} Scalar (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) zero), @Eq.{p} PointCarrier A B), forall (backward : forall (h : @Eq.{p} PointCarrier A B), @Eq.{u} Scalar (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) zero), R), R"
        ),
        proof: abstract_metric_abs!(
            "fun law => fun A => fun B => fun (R : Prop) => fun (mk : forall (forward : forall (h : @Eq.{u} Scalar (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) zero), @Eq.{p} PointCarrier A B), forall (backward : forall (h : @Eq.{p} PointCarrier A B), @Eq.{u} Scalar (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) zero), R) => law A B R mk"
        ),
    },
    TheoremArtifact {
        name: "pythagorean_distance_general",
        universe_params: &["p", "u", "v"],
        statement: abstract_metric_params!(
            "forall (metric_pythagorean_target : forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), forall (h : @RightTriangle.{p,u,v} Scalar zero Vector inner PointCarrier disp_op A B C), @Eq.{u} Scalar (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op B C)) (add (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B)) (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A C)))), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), forall (h : @RightTriangle.{p,u,v} Scalar zero Vector inner PointCarrier disp_op A B C), @Eq.{u} Scalar (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op B C)) (add (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B)) (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A C)))"
        ),
        proof: abstract_metric_abs!(
            "fun metric_pythagorean_target => fun A => fun B => fun C => fun h => metric_pythagorean_target A B C h"
        ),
    },
    TheoremArtifact {
        name: "triangle_inequality",
        universe_params: &["p", "u", "v"],
        statement: abstract_metric_params!(
            "forall (law : forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), le_rel (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A C) (add (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op B C))), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), le_rel (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A C) (add (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B) (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op B C))"
        ),
        proof: abstract_metric_abs!("fun law => fun A => fun B => fun C => law A B C"),
    },
];

const PYTHAGOREAN_THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "pythagorean_dist_sq_symm_from_affine_args",
        universe_params: &["p", "u", "v"],
        statement: affine_params!(
            "forall (affine_args : @AffineLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op), forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B A)"
        ),
        proof: affine_abs!(concat!(
            "fun affine_args => fun A => fun B => ",
            "affine_args (@Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B A)) ",
            "(fun (disp_self_arg : forall (A : PointCarrier), @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op A A) vzero) => ",
            "fun (disp_reverse_arg : forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op B A) (vneg (@disp.{p,v} PointCarrier Vector disp_op A B))) => ",
            "fun (disp_comp_arg : forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op A C) (vadd (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op B C))) => ",
            "fun (point_ext_arg : forall (A : PointCarrier), forall (B : PointCarrier), forall (h : @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op A B) vzero), @Eq.{p} PointCarrier A B) => ",
            "fun (dist_sq_symm_arg : forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B A)) => ",
            "fun (dist_sq_zero_iff_eq_arg : forall (A : PointCarrier), forall (B : PointCarrier), forall (R : Prop), forall (mk : forall (forward : forall (h : @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) zero), @Eq.{p} PointCarrier A B), forall (backward : forall (h : @Eq.{p} PointCarrier A B), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) zero), R), R) => dist_sq_symm_arg A B)"
        )),
    },
    TheoremArtifact {
        name: "pythagorean_dist_sq_reverse_norm_neg_from_law_packages",
        universe_params: &["p", "u", "v"],
        statement: affine_params!(
            "forall (affine_args : @AffineLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op), forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B A) (@normSq.{u,v} Scalar Vector inner (vneg (@disp.{p,v} PointCarrier Vector disp_op A B)))"
        ),
        proof: affine_abs!(concat!(
            "fun affine_args => fun A => fun B => ",
            "@eq_trans.{u} Scalar ",
            "(@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B A) ",
            "(@normSq.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op B A)) ",
            "(@normSq.{u,v} Scalar Vector inner (vneg (@disp.{p,v} PointCarrier Vector disp_op A B))) ",
            "(@dist_sq_points_def_from_args.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op affine_args B A) ",
            "(@eq_congr_arg.{v,u} Vector Scalar (fun (x : Vector) => @normSq.{u,v} Scalar Vector inner x) ",
            "(@disp.{p,v} PointCarrier Vector disp_op B A) ",
            "(vneg (@disp.{p,v} PointCarrier Vector disp_op A B)) ",
            "(@disp_reverse_from_affine_args.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op affine_args A B))"
        )),
    },
    TheoremArtifact {
        name: "pythagorean_left_leg_norm_neg_from_law_packages",
        universe_params: &["p", "u", "v"],
        statement: affine_params!(
            "forall (affine_args : @AffineLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op), forall (A : PointCarrier), forall (B : PointCarrier), @Eq.{u} Scalar (@normSq.{u,v} Scalar Vector inner (vneg (@disp.{p,v} PointCarrier Vector disp_op A B))) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B)"
        ),
        proof: affine_abs!(concat!(
            "fun affine_args => fun A => fun B => ",
            "@eq_trans.{u} Scalar ",
            "(@normSq.{u,v} Scalar Vector inner (vneg (@disp.{p,v} PointCarrier Vector disp_op A B))) ",
            "(@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B A) ",
            "(@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) ",
            "(@eq_symm.{u} Scalar ",
            "(@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B A) ",
            "(@normSq.{u,v} Scalar Vector inner (vneg (@disp.{p,v} PointCarrier Vector disp_op A B))) ",
            "(@pythagorean_dist_sq_reverse_norm_neg_from_law_packages.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op affine_args A B)) ",
            "(@pythagorean_dist_sq_symm_from_affine_args.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op affine_args B A)"
        )),
    },
    TheoremArtifact {
        name: "dist_sq_law_of_cosines_rhs_from_law_packages",
        universe_params: &["p", "u", "v"],
        statement: affine_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (affine_args : @AffineLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C) (sub (add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C)) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C))))"
        ),
        proof: affine_abs!(concat!(
            "fun ring_args => fun inner_args => fun affine_args => fun A => fun B => fun C => ",
            "@eq_calc3.{u} Scalar ",
            "(@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C) ",
            "(@normSq.{u,v} Scalar Vector inner (vadd (vneg (@disp.{p,v} PointCarrier Vector disp_op A B)) (@disp.{p,v} PointCarrier Vector disp_op A C))) ",
            "(sub (add (@normSq.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B)) (@normSq.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A C))) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C)))) ",
            "(sub (add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C)) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C)))) ",
            "(@eq_trans.{u} Scalar ",
            "(@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C) ",
            "(@normSq.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op B C)) ",
            "(@normSq.{u,v} Scalar Vector inner (vadd (vneg (@disp.{p,v} PointCarrier Vector disp_op A B)) (@disp.{p,v} PointCarrier Vector disp_op A C))) ",
            "(@dist_sq_points_def_from_args.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op affine_args B C) ",
            "(@eq_congr_arg.{v,u} Vector Scalar (fun (x : Vector) => @normSq.{u,v} Scalar Vector inner x) ",
            "(@disp.{p,v} PointCarrier Vector disp_op B C) ",
            "(vadd (vneg (@disp.{p,v} PointCarrier Vector disp_op A B)) (@disp.{p,v} PointCarrier Vector disp_op A C)) ",
            "(@hypotenuse_vector_eq_neg_left_add_right_from_args.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op affine_args A B C))) ",
            "(@norm_sq_add_neg_left_from_inner_args.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner ring_args inner_args (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C)) ",
            "(@eq_congr2.{u,u,u} Scalar Scalar Scalar sub ",
            "(add (@normSq.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B)) (@normSq.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A C))) ",
            "(add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C)) ",
            "(mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C))) ",
            "(mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C))) ",
            "(@eq_congr2.{u,u,u} Scalar Scalar Scalar add ",
            "(@normSq.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B)) ",
            "(@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) ",
            "(@normSq.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A C)) ",
            "(@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C) ",
            "(@eq_symm.{u} Scalar ",
            "(@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) ",
            "(@normSq.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B)) ",
            "(@dist_sq_points_def_from_args.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op affine_args A B)) ",
            "(@eq_symm.{u} Scalar ",
            "(@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C) ",
            "(@normSq.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A C)) ",
            "(@dist_sq_points_def_from_args.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op affine_args A C))) ",
            "(@Eq.refl.{u} Scalar (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C)))))"
        )),
    },
    TheoremArtifact {
        name: "law_of_cosines_sq_from_law_packages",
        universe_params: &["p", "u", "v"],
        statement: affine_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (vector_args : @VectorSpaceLawArgs.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (affine_args : @AffineLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C) (sub (add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C)) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C))))"
        ),
        proof: affine_abs!(
            "fun ring_args => fun vector_args => fun inner_args => fun affine_args => fun A => fun B => fun C => @dist_sq_law_of_cosines_rhs_from_law_packages.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op ring_args inner_args affine_args A B C"
        ),
    },
    TheoremArtifact {
        name: "law_of_cosines_dist_sq_from_law_packages",
        universe_params: &["p", "u", "v"],
        statement: abstract_metric_params!(
            "forall (ordered_args : @OrderedFieldLawArgs.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn), forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (vector_args : @VectorSpaceLawArgs.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (affine_args : @AffineLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), @Eq.{u} Scalar (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op B C)) (sub (add (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B)) (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A C))) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C))))"
        ),
        proof: abstract_metric_abs!(concat!(
            "fun ordered_args => fun ring_args => fun vector_args => fun inner_args => fun affine_args => fun A => fun B => fun C => ",
            "@eq_calc3.{u} Scalar ",
            "(@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op B C)) ",
            "(@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C) ",
            "(sub (add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C)) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C)))) ",
            "(sub (add (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B)) (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A C))) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C)))) ",
            "(@square_dist_eq_dist_sq_from_law_packages.{p,u,v} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn Vector vzero vadd vneg smul inner PointCarrier disp_op ordered_args inner_args B C) ",
            "(@law_of_cosines_sq_from_law_packages.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op ring_args vector_args inner_args affine_args A B C) ",
            "(@eq_congr2.{u,u,u} Scalar Scalar Scalar sub ",
            "(add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C)) ",
            "(add (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B)) (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A C))) ",
            "(mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C))) ",
            "(mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C))) ",
            "(@eq_congr2.{u,u,u} Scalar Scalar Scalar add ",
            "(@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) ",
            "(@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B)) ",
            "(@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C) ",
            "(@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A C)) ",
            "(@dist_sq_eq_square_dist_from_law_packages.{p,u,v} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn Vector vzero vadd vneg smul inner PointCarrier disp_op ordered_args inner_args A B) ",
            "(@dist_sq_eq_square_dist_from_law_packages.{p,u,v} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn Vector vzero vadd vneg smul inner PointCarrier disp_op ordered_args inner_args A C)) ",
            "(@Eq.refl.{u} Scalar (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C)))))"
        )),
    },
    TheoremArtifact {
        name: "pythagorean_distance_sq_from_law_packages",
        universe_params: &["p", "u", "v"],
        statement: affine_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (vector_args : @VectorSpaceLawArgs.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (affine_args : @AffineLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), forall (h : @RightTriangle.{p,u,v} Scalar zero Vector inner PointCarrier disp_op A B C), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C) (add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C))"
        ),
        proof: affine_abs!(concat!(
            "fun ring_args => fun vector_args => fun inner_args => fun affine_args => fun A => fun B => fun C => fun h => ",
            "@right_triangle_affine_additive_perp_bridge_from_rt.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op ring_args inner_args affine_args A B C h ",
            "(@Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C) (add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C))) ",
            "(fun (hypotenuse_orientation : @Eq.{v} Vector (@disp.{p,v} PointCarrier Vector disp_op B C) (vadd (vneg (@disp.{p,v} PointCarrier Vector disp_op A B)) (@disp.{p,v} PointCarrier Vector disp_op A C))) => ",
            "fun (perp_premise : @PerpVec.{u,v} Scalar zero Vector inner (vneg (@disp.{p,v} PointCarrier Vector disp_op A B)) (@disp.{p,v} PointCarrier Vector disp_op A C)) => ",
            "@eq_calc3.{u} Scalar ",
            "(@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C) ",
            "(@normSq.{u,v} Scalar Vector inner (vadd (vneg (@disp.{p,v} PointCarrier Vector disp_op A B)) (@disp.{p,v} PointCarrier Vector disp_op A C))) ",
            "(add (@normSq.{u,v} Scalar Vector inner (vneg (@disp.{p,v} PointCarrier Vector disp_op A B))) (@normSq.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A C))) ",
            "(add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C)) ",
            "(@eq_trans.{u} Scalar ",
            "(@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C) ",
            "(@normSq.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op B C)) ",
            "(@normSq.{u,v} Scalar Vector inner (vadd (vneg (@disp.{p,v} PointCarrier Vector disp_op A B)) (@disp.{p,v} PointCarrier Vector disp_op A C))) ",
            "(@dist_sq_points_def_from_args.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op affine_args B C) ",
            "(@eq_congr_arg.{v,u} Vector Scalar (fun (x : Vector) => @normSq.{u,v} Scalar Vector inner x) ",
            "(@disp.{p,v} PointCarrier Vector disp_op B C) ",
            "(vadd (vneg (@disp.{p,v} PointCarrier Vector disp_op A B)) (@disp.{p,v} PointCarrier Vector disp_op A C)) ",
            "hypotenuse_orientation)) ",
            "(@norm_sq_add_of_perp_from_args.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner ring_args inner_args (vneg (@disp.{p,v} PointCarrier Vector disp_op A B)) (@disp.{p,v} PointCarrier Vector disp_op A C) perp_premise) ",
            "(@eq_congr2.{u,u,u} Scalar Scalar Scalar add ",
            "(@normSq.{u,v} Scalar Vector inner (vneg (@disp.{p,v} PointCarrier Vector disp_op A B))) ",
            "(@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) ",
            "(@normSq.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A C)) ",
            "(@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C) ",
            "(@pythagorean_left_leg_norm_neg_from_law_packages.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op affine_args A B) ",
            "(@Eq.refl.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C))))"
        )),
    },
    TheoremArtifact {
        name: "pythagorean_theorem_sq",
        universe_params: &["p", "u", "v"],
        statement: abstract_metric_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (vector_args : @VectorSpaceLawArgs.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (affine_args : @AffineLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), forall (h : @RightTriangle.{p,u,v} Scalar zero Vector inner PointCarrier disp_op A B C), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C) (add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C))"
        ),
        proof: abstract_metric_abs!(
            "fun ring_args => fun vector_args => fun inner_args => fun affine_args => fun A => fun B => fun C => fun h => @pythagorean_distance_sq_from_law_packages.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op ring_args vector_args inner_args affine_args A B C h"
        ),
    },
    TheoremArtifact {
        name: "pythagorean_theorem_dist_sq",
        universe_params: &["p", "u", "v"],
        statement: abstract_metric_params!(
            "forall (ordered_args : @OrderedFieldLawArgs.{u} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn), forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (vector_args : @VectorSpaceLawArgs.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (affine_args : @AffineLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), forall (h : @RightTriangle.{p,u,v} Scalar zero Vector inner PointCarrier disp_op A B C), @Eq.{u} Scalar (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op B C)) (add (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B)) (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A C)))"
        ),
        proof: abstract_metric_abs!(concat!(
            "fun ordered_args => fun ring_args => fun vector_args => fun inner_args => fun affine_args => fun A => fun B => fun C => fun h => ",
            "@eq_calc3.{u} Scalar ",
            "(@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op B C)) ",
            "(@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C) ",
            "(add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C)) ",
            "(add (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B)) (@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A C))) ",
            "(@square_dist_eq_dist_sq_from_law_packages.{p,u,v} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn Vector vzero vadd vneg smul inner PointCarrier disp_op ordered_args inner_args B C) ",
            "(@pythagorean_distance_sq_from_law_packages.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op ring_args vector_args inner_args affine_args A B C h) ",
            "(@eq_congr2.{u,u,u} Scalar Scalar Scalar add ",
            "(@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) ",
            "(@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A B)) ",
            "(@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C) ",
            "(@sq.{u} Scalar mul (@dist.{p,u,v} Scalar sqrt_fn Vector inner PointCarrier disp_op A C)) ",
            "(@dist_sq_eq_square_dist_from_law_packages.{p,u,v} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn Vector vzero vadd vneg smul inner PointCarrier disp_op ordered_args inner_args A B) ",
            "(@dist_sq_eq_square_dist_from_law_packages.{p,u,v} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn Vector vzero vadd vneg smul inner PointCarrier disp_op ordered_args inner_args A C))"
        )),
    },
    TheoremArtifact {
        name: "pythagorean_converse_sq",
        universe_params: &["p", "u", "v"],
        statement: abstract_metric_params!(
            "forall (converse_target : forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), forall (h : @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C) (add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C))), @RightTriangle.{p,u,v} Scalar zero Vector inner PointCarrier disp_op A B C), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), forall (h : @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C) (add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C))), @RightTriangle.{p,u,v} Scalar zero Vector inner PointCarrier disp_op A B C"
        ),
        proof: abstract_metric_abs!(
            "fun converse_target => fun A => fun B => fun C => fun h => converse_target A B C h"
        ),
    },
    TheoremArtifact {
        name: "law_of_cosines_right_angle_specialization_from_law_packages",
        universe_params: &["p", "u", "v"],
        statement: affine_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (vector_args : @VectorSpaceLawArgs.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (affine_args : @AffineLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), forall (h : @RightTriangle.{p,u,v} Scalar zero Vector inner PointCarrier disp_op A B C), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C) (add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C))"
        ),
        proof: affine_abs!(concat!(
            "fun ring_args => fun vector_args => fun inner_args => fun affine_args => fun A => fun B => fun C => fun h => ",
            "@eq_trans.{u} Scalar ",
            "(@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C) ",
            "(sub (add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C)) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C)))) ",
            "(add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C)) ",
            "(@law_of_cosines_sq_from_law_packages.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op ring_args vector_args inner_args affine_args A B C) ",
            "(@eq_trans.{u} Scalar ",
            "(sub (add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C)) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C)))) ",
            "(add (add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (mul (@two.{u} Scalar one add) (neg (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C))))) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C)) ",
            "(add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C)) ",
            "(@eq_symm.{u} Scalar ",
            "(add (add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (mul (@two.{u} Scalar one add) (neg (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C))))) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C)) ",
            "(sub (add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C)) (mul (@two.{u} Scalar one add) (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C)))) ",
            "(@law_of_cosines_scalar_rhs_from_ring_args.{u} Scalar zero one add neg sub mul ring_args (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C) (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C)))) ",
            "(@normalize_add_with_zero_cross_term_from_ring_args.{u} Scalar zero one add neg sub mul ring_args (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C) (neg (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C))) ",
            "(@eq_trans.{u} Scalar ",
            "(neg (@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C))) ",
            "(neg zero) zero ",
            "(@eq_congr_arg.{u,u} Scalar Scalar (fun (x : Scalar) => neg x) ",
            "(@dot.{u,v} Scalar Vector inner (@disp.{p,v} PointCarrier Vector disp_op A B) (@disp.{p,v} PointCarrier Vector disp_op A C)) zero ",
            "(@right_triangle_legs_dot_zero_from_rt.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op A B C h)) ",
            "(@neg_zero_from_ring_args.{u} Scalar zero one add neg sub mul ring_args))))"
        )),
    },
    TheoremArtifact {
        name: "law_of_cosines_right_angle_specialization",
        universe_params: &["p", "u", "v"],
        statement: abstract_metric_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (vector_args : @VectorSpaceLawArgs.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (affine_args : @AffineLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), forall (h : @RightTriangle.{p,u,v} Scalar zero Vector inner PointCarrier disp_op A B C), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C) (add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C))"
        ),
        proof: abstract_metric_abs!(
            "fun ring_args => fun vector_args => fun inner_args => fun affine_args => fun A => fun B => fun C => fun h => @law_of_cosines_right_angle_specialization_from_law_packages.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op ring_args vector_args inner_args affine_args A B C h"
        ),
    },
    TheoremArtifact {
        name: "pythagorean_theorem_api_alias",
        universe_params: &["p", "u", "v"],
        statement: abstract_metric_params!(
            "forall (ring_args : @RingLawArgs.{u} Scalar zero one add neg sub mul), forall (vector_args : @VectorSpaceLawArgs.{u,v} Scalar zero one add neg sub mul Vector vzero vadd vneg smul), forall (inner_args : @InnerProductLawArgs.{u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner), forall (affine_args : @AffineLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel Vector vzero vadd vneg smul inner PointCarrier disp_op), forall (A : PointCarrier), forall (B : PointCarrier), forall (C : PointCarrier), forall (h : @RightTriangle.{p,u,v} Scalar zero Vector inner PointCarrier disp_op A B C), @Eq.{u} Scalar (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op B C) (add (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A B) (@distSqPoints.{p,u,v} Scalar Vector inner PointCarrier disp_op A C))"
        ),
        proof: abstract_metric_abs!(
            "fun ring_args => fun vector_args => fun inner_args => fun affine_args => fun A => fun B => fun C => fun h => @pythagorean_theorem_sq.{p,u,v} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn Vector vzero vadd vneg smul inner PointCarrier disp_op ring_args vector_args inner_args affine_args A B C h"
        ),
    },
    TheoremArtifact {
        name: "pythagorean_theorem_dependencies",
        universe_params: &["p", "u", "v"],
        statement: abstract_metric_params!(
            "forall (laws : @MetricSpaceLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn Vector vzero vadd vneg smul inner PointCarrier disp_op), @MetricSpaceLawArgs.{p,u,v} Scalar zero one add neg sub mul le_rel lt_rel sqrt_fn Vector vzero vadd vneg smul inner PointCarrier disp_op"
        ),
        proof: abstract_metric_abs!("fun laws => laws"),
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
    let abstract_group_imports = vec![eq_import.clone(), eq_reasoning.verified_module.clone()];
    let abstract_group_source_interfaces = vec![
        eq_source_interface.clone(),
        eq_reasoning.source_interface.clone(),
    ];
    let abstract_group = build_and_write_module(
        &proof_root,
        &ABSTRACT_GROUP_MODULE,
        &abstract_group_imports,
        &abstract_group_source_interfaces,
    )?;
    let abstract_group_kernel_imports = vec![
        eq_import.clone(),
        eq_reasoning.verified_module.clone(),
        abstract_group.verified_module.clone(),
    ];
    let abstract_group_kernel_source_interfaces = vec![
        eq_source_interface.clone(),
        eq_reasoning.source_interface.clone(),
        abstract_group.source_interface.clone(),
    ];
    let abstract_group_kernel = build_and_write_module(
        &proof_root,
        &ABSTRACT_GROUP_KERNEL_MODULE,
        &abstract_group_kernel_imports,
        &abstract_group_kernel_source_interfaces,
    )?;
    let abstract_group_image_imports = vec![
        eq_import.clone(),
        eq_reasoning.verified_module.clone(),
        abstract_group.verified_module.clone(),
    ];
    let abstract_group_image_source_interfaces = vec![
        eq_source_interface.clone(),
        eq_reasoning.source_interface.clone(),
        abstract_group.source_interface.clone(),
    ];
    let abstract_group_image = build_and_write_module(
        &proof_root,
        &ABSTRACT_GROUP_IMAGE_MODULE,
        &abstract_group_image_imports,
        &abstract_group_image_source_interfaces,
    )?;
    let abstract_group_quotient_imports =
        vec![eq_import.clone(), abstract_group.verified_module.clone()];
    let abstract_group_quotient_source_interfaces = vec![
        eq_source_interface.clone(),
        abstract_group.source_interface.clone(),
    ];
    let abstract_group_quotient = build_and_write_module(
        &proof_root,
        &ABSTRACT_GROUP_QUOTIENT_MODULE,
        &abstract_group_quotient_imports,
        &abstract_group_quotient_source_interfaces,
    )?;
    let abstract_group_quotient_mul_imports = vec![
        eq_import.clone(),
        eq_reasoning.verified_module.clone(),
        abstract_group.verified_module.clone(),
        abstract_group_quotient.verified_module.clone(),
    ];
    let abstract_group_quotient_mul_source_interfaces = vec![
        eq_source_interface.clone(),
        eq_reasoning.source_interface.clone(),
        abstract_group.source_interface.clone(),
        abstract_group_quotient.source_interface.clone(),
    ];
    let abstract_group_quotient_mul = build_and_write_module(
        &proof_root,
        &ABSTRACT_GROUP_QUOTIENT_MUL_MODULE,
        &abstract_group_quotient_mul_imports,
        &abstract_group_quotient_mul_source_interfaces,
    )?;
    let abstract_group_quotient_group_imports = vec![
        eq_import.clone(),
        eq_reasoning.verified_module.clone(),
        abstract_group.verified_module.clone(),
        abstract_group_quotient.verified_module.clone(),
        abstract_group_quotient_mul.verified_module.clone(),
    ];
    let abstract_group_quotient_group_source_interfaces = vec![
        eq_source_interface.clone(),
        eq_reasoning.source_interface.clone(),
        abstract_group.source_interface.clone(),
        abstract_group_quotient.source_interface.clone(),
        abstract_group_quotient_mul.source_interface.clone(),
    ];
    let abstract_group_quotient_group = build_and_write_module(
        &proof_root,
        &ABSTRACT_GROUP_QUOTIENT_GROUP_MODULE,
        &abstract_group_quotient_group_imports,
        &abstract_group_quotient_group_source_interfaces,
    )?;
    let abstract_group_quotient_hom_imports = vec![
        eq_import.clone(),
        eq_reasoning.verified_module.clone(),
        abstract_group.verified_module.clone(),
        abstract_group_quotient.verified_module.clone(),
        abstract_group_quotient_mul.verified_module.clone(),
        abstract_group_quotient_group.verified_module.clone(),
    ];
    let abstract_group_quotient_hom_source_interfaces = vec![
        eq_source_interface.clone(),
        eq_reasoning.source_interface.clone(),
        abstract_group.source_interface.clone(),
        abstract_group_quotient.source_interface.clone(),
        abstract_group_quotient_mul.source_interface.clone(),
        abstract_group_quotient_group.source_interface.clone(),
    ];
    let abstract_group_quotient_hom = build_and_write_module(
        &proof_root,
        &ABSTRACT_GROUP_QUOTIENT_HOM_MODULE,
        &abstract_group_quotient_hom_imports,
        &abstract_group_quotient_hom_source_interfaces,
    )?;
    let abstract_group_first_iso_full_imports = vec![
        eq_import.clone(),
        eq_reasoning.verified_module.clone(),
        abstract_group.verified_module.clone(),
        abstract_group_image.verified_module.clone(),
        abstract_group_quotient.verified_module.clone(),
        abstract_group_quotient_mul.verified_module.clone(),
        abstract_group_quotient_group.verified_module.clone(),
        abstract_group_quotient_hom.verified_module.clone(),
    ];
    let abstract_group_first_iso_full_source_interfaces = vec![
        eq_source_interface.clone(),
        eq_reasoning.source_interface.clone(),
        abstract_group.source_interface.clone(),
        abstract_group_image.source_interface.clone(),
        abstract_group_quotient.source_interface.clone(),
        abstract_group_quotient_mul.source_interface.clone(),
        abstract_group_quotient_group.source_interface.clone(),
        abstract_group_quotient_hom.source_interface.clone(),
    ];
    let abstract_group_first_iso_full = build_and_write_module(
        &proof_root,
        &ABSTRACT_GROUP_FIRST_ISO_FULL_MODULE,
        &abstract_group_first_iso_full_imports,
        &abstract_group_first_iso_full_source_interfaces,
    )?;
    let abstract_group_first_iso_image_imports = vec![
        eq_import.clone(),
        eq_reasoning.verified_module.clone(),
        abstract_group.verified_module.clone(),
        abstract_group_image.verified_module.clone(),
        abstract_group_quotient.verified_module.clone(),
        abstract_group_quotient_mul.verified_module.clone(),
        abstract_group_quotient_group.verified_module.clone(),
        abstract_group_first_iso_full.verified_module.clone(),
    ];
    let abstract_group_first_iso_image_source_interfaces = vec![
        eq_source_interface.clone(),
        eq_reasoning.source_interface.clone(),
        abstract_group.source_interface.clone(),
        abstract_group_image.source_interface.clone(),
        abstract_group_quotient.source_interface.clone(),
        abstract_group_quotient_mul.source_interface.clone(),
        abstract_group_quotient_group.source_interface.clone(),
        abstract_group_first_iso_full.source_interface.clone(),
    ];
    let abstract_group_first_iso_image = build_and_write_module(
        &proof_root,
        &ABSTRACT_GROUP_FIRST_ISO_IMAGE_MODULE,
        &abstract_group_first_iso_image_imports,
        &abstract_group_first_iso_image_source_interfaces,
    )?;
    let abstract_group_first_iso_imports = vec![
        eq_import.clone(),
        abstract_group.verified_module.clone(),
        abstract_group_image.verified_module.clone(),
        abstract_group_quotient.verified_module.clone(),
    ];
    let abstract_group_first_iso_source_interfaces = vec![
        eq_source_interface.clone(),
        abstract_group.source_interface.clone(),
        abstract_group_image.source_interface.clone(),
        abstract_group_quotient.source_interface.clone(),
    ];
    let abstract_group_first_iso = build_and_write_module(
        &proof_root,
        &ABSTRACT_GROUP_FIRST_ISO_MODULE,
        &abstract_group_first_iso_imports,
        &abstract_group_first_iso_source_interfaces,
    )?;
    let abstract_group_subgroup_imports = vec![
        eq_import.clone(),
        eq_reasoning.verified_module.clone(),
        abstract_group.verified_module.clone(),
    ];
    let abstract_group_subgroup_source_interfaces = vec![
        eq_source_interface.clone(),
        eq_reasoning.source_interface.clone(),
        abstract_group.source_interface.clone(),
    ];
    let abstract_group_subgroup = build_and_write_module(
        &proof_root,
        &ABSTRACT_GROUP_SUBGROUP_MODULE,
        &abstract_group_subgroup_imports,
        &abstract_group_subgroup_source_interfaces,
    )?;
    let abstract_group_normal_quotient_imports = vec![
        eq_import.clone(),
        abstract_group.verified_module.clone(),
        abstract_group_subgroup.verified_module.clone(),
    ];
    let abstract_group_normal_quotient_source_interfaces = vec![
        eq_source_interface.clone(),
        abstract_group.source_interface.clone(),
        abstract_group_subgroup.source_interface.clone(),
    ];
    let abstract_group_normal_quotient = build_and_write_module(
        &proof_root,
        &ABSTRACT_GROUP_NORMAL_QUOTIENT_MODULE,
        &abstract_group_normal_quotient_imports,
        &abstract_group_normal_quotient_source_interfaces,
    )?;
    let abstract_group_normal_quotient_mul_imports = vec![
        eq_import.clone(),
        abstract_group.verified_module.clone(),
        abstract_group_subgroup.verified_module.clone(),
        abstract_group_normal_quotient.verified_module.clone(),
    ];
    let abstract_group_normal_quotient_mul_source_interfaces = vec![
        eq_source_interface.clone(),
        abstract_group.source_interface.clone(),
        abstract_group_subgroup.source_interface.clone(),
        abstract_group_normal_quotient.source_interface.clone(),
    ];
    let abstract_group_normal_quotient_mul = build_and_write_module(
        &proof_root,
        &ABSTRACT_GROUP_NORMAL_QUOTIENT_MUL_MODULE,
        &abstract_group_normal_quotient_mul_imports,
        &abstract_group_normal_quotient_mul_source_interfaces,
    )?;
    let abstract_group_normal_quotient_group_imports = vec![
        eq_import.clone(),
        abstract_group.verified_module.clone(),
        abstract_group_subgroup.verified_module.clone(),
        abstract_group_normal_quotient.verified_module.clone(),
        abstract_group_normal_quotient_mul.verified_module.clone(),
    ];
    let abstract_group_normal_quotient_group_source_interfaces = vec![
        eq_source_interface.clone(),
        abstract_group.source_interface.clone(),
        abstract_group_subgroup.source_interface.clone(),
        abstract_group_normal_quotient.source_interface.clone(),
        abstract_group_normal_quotient_mul.source_interface.clone(),
    ];
    let abstract_group_normal_quotient_group = build_and_write_module(
        &proof_root,
        &ABSTRACT_GROUP_NORMAL_QUOTIENT_GROUP_MODULE,
        &abstract_group_normal_quotient_group_imports,
        &abstract_group_normal_quotient_group_source_interfaces,
    )?;
    let abstract_group_second_iso_phi_imports = vec![
        eq_import.clone(),
        abstract_group.verified_module.clone(),
        abstract_group_subgroup.verified_module.clone(),
        abstract_group_normal_quotient.verified_module.clone(),
        abstract_group_normal_quotient_mul.verified_module.clone(),
        abstract_group_normal_quotient_group.verified_module.clone(),
    ];
    let abstract_group_second_iso_phi_source_interfaces = vec![
        eq_source_interface.clone(),
        abstract_group.source_interface.clone(),
        abstract_group_subgroup.source_interface.clone(),
        abstract_group_normal_quotient.source_interface.clone(),
        abstract_group_normal_quotient_mul.source_interface.clone(),
        abstract_group_normal_quotient_group
            .source_interface
            .clone(),
    ];
    let abstract_group_second_iso_phi = build_and_write_module(
        &proof_root,
        &ABSTRACT_GROUP_SECOND_ISO_PHI_MODULE,
        &abstract_group_second_iso_phi_imports,
        &abstract_group_second_iso_phi_source_interfaces,
    )?;
    let abstract_group_second_iso_kernel_imports = vec![
        eq_import.clone(),
        abstract_group.verified_module.clone(),
        abstract_group_subgroup.verified_module.clone(),
        abstract_group_normal_quotient.verified_module.clone(),
        abstract_group_normal_quotient_mul.verified_module.clone(),
        abstract_group_normal_quotient_group.verified_module.clone(),
        abstract_group_second_iso_phi.verified_module.clone(),
    ];
    let abstract_group_second_iso_kernel_source_interfaces = vec![
        eq_source_interface.clone(),
        abstract_group.source_interface.clone(),
        abstract_group_subgroup.source_interface.clone(),
        abstract_group_normal_quotient.source_interface.clone(),
        abstract_group_normal_quotient_mul.source_interface.clone(),
        abstract_group_normal_quotient_group
            .source_interface
            .clone(),
        abstract_group_second_iso_phi.source_interface.clone(),
    ];
    let abstract_group_second_iso_kernel = build_and_write_module(
        &proof_root,
        &ABSTRACT_GROUP_SECOND_ISO_KERNEL_MODULE,
        &abstract_group_second_iso_kernel_imports,
        &abstract_group_second_iso_kernel_source_interfaces,
    )?;
    let abstract_group_second_iso_image_imports = vec![
        eq_import.clone(),
        eq_reasoning.verified_module.clone(),
        abstract_group.verified_module.clone(),
        abstract_group_subgroup.verified_module.clone(),
        abstract_group_normal_quotient.verified_module.clone(),
        abstract_group_normal_quotient_mul.verified_module.clone(),
        abstract_group_normal_quotient_group.verified_module.clone(),
        abstract_group_second_iso_phi.verified_module.clone(),
    ];
    let abstract_group_second_iso_image_source_interfaces = vec![
        eq_source_interface.clone(),
        eq_reasoning.source_interface.clone(),
        abstract_group.source_interface.clone(),
        abstract_group_subgroup.source_interface.clone(),
        abstract_group_normal_quotient.source_interface.clone(),
        abstract_group_normal_quotient_mul.source_interface.clone(),
        abstract_group_normal_quotient_group
            .source_interface
            .clone(),
        abstract_group_second_iso_phi.source_interface.clone(),
    ];
    let abstract_group_second_iso_image = build_and_write_module(
        &proof_root,
        &ABSTRACT_GROUP_SECOND_ISO_IMAGE_MODULE,
        &abstract_group_second_iso_image_imports,
        &abstract_group_second_iso_image_source_interfaces,
    )?;
    let abstract_group_second_iso_final_imports = vec![
        eq_import.clone(),
        abstract_group.verified_module.clone(),
        abstract_group_subgroup.verified_module.clone(),
        abstract_group_normal_quotient.verified_module.clone(),
        abstract_group_normal_quotient_mul.verified_module.clone(),
        abstract_group_normal_quotient_group.verified_module.clone(),
        abstract_group_second_iso_phi.verified_module.clone(),
        abstract_group_second_iso_kernel.verified_module.clone(),
        abstract_group_second_iso_image.verified_module.clone(),
    ];
    let abstract_group_second_iso_final_source_interfaces = vec![
        eq_source_interface.clone(),
        abstract_group.source_interface.clone(),
        abstract_group_subgroup.source_interface.clone(),
        abstract_group_normal_quotient.source_interface.clone(),
        abstract_group_normal_quotient_mul.source_interface.clone(),
        abstract_group_normal_quotient_group
            .source_interface
            .clone(),
        abstract_group_second_iso_phi.source_interface.clone(),
        abstract_group_second_iso_kernel.source_interface.clone(),
        abstract_group_second_iso_image.source_interface.clone(),
    ];
    let abstract_group_second_iso_final = build_and_write_module(
        &proof_root,
        &ABSTRACT_GROUP_SECOND_ISO_FINAL_MODULE,
        &abstract_group_second_iso_final_imports,
        &abstract_group_second_iso_final_source_interfaces,
    )?;
    let abstract_group_third_iso_imports = vec![
        eq_import.clone(),
        eq_reasoning.verified_module.clone(),
        abstract_group.verified_module.clone(),
        abstract_group_quotient.verified_module.clone(),
        abstract_group_subgroup.verified_module.clone(),
        abstract_group_normal_quotient.verified_module.clone(),
        abstract_group_normal_quotient_mul.verified_module.clone(),
        abstract_group_normal_quotient_group.verified_module.clone(),
    ];
    let abstract_group_third_iso_source_interfaces = vec![
        eq_source_interface.clone(),
        eq_reasoning.source_interface.clone(),
        abstract_group.source_interface.clone(),
        abstract_group_quotient.source_interface.clone(),
        abstract_group_subgroup.source_interface.clone(),
        abstract_group_normal_quotient.source_interface.clone(),
        abstract_group_normal_quotient_mul.source_interface.clone(),
        abstract_group_normal_quotient_group
            .source_interface
            .clone(),
    ];
    let abstract_group_third_iso = build_and_write_module(
        &proof_root,
        &ABSTRACT_GROUP_THIRD_ISO_MODULE,
        &abstract_group_third_iso_imports,
        &abstract_group_third_iso_source_interfaces,
    )?;
    let abstract_ring_imports = vec![eq_import.clone()];
    let abstract_ring_source_interfaces = vec![eq_source_interface.clone()];
    let abstract_ring = build_and_write_module(
        &proof_root,
        &ABSTRACT_RING_MODULE,
        &abstract_ring_imports,
        &abstract_ring_source_interfaces,
    )?;
    let abstract_ordered_field_imports =
        vec![eq_import.clone(), abstract_ring.verified_module.clone()];
    let abstract_ordered_field_source_interfaces = vec![abstract_ring.source_interface.clone()];
    let abstract_ordered_field = build_and_write_module(
        &proof_root,
        &ABSTRACT_ORDERED_FIELD_MODULE,
        &abstract_ordered_field_imports,
        &abstract_ordered_field_source_interfaces,
    )?;
    let abstract_square_normalize_imports = vec![
        eq_import.clone(),
        abstract_ring.verified_module.clone(),
        abstract_ordered_field.verified_module.clone(),
    ];
    let abstract_square_normalize_source_interfaces = vec![
        abstract_ring.source_interface.clone(),
        abstract_ordered_field.source_interface.clone(),
    ];
    let abstract_square_normalize = build_and_write_module(
        &proof_root,
        &ABSTRACT_SQUARE_NORMALIZE_MODULE,
        &abstract_square_normalize_imports,
        &abstract_square_normalize_source_interfaces,
    )?;
    let abstract_scalar_derive_imports = vec![
        eq_import.clone(),
        eq_reasoning.verified_module.clone(),
        abstract_ring.verified_module.clone(),
        abstract_square_normalize.verified_module.clone(),
    ];
    let abstract_scalar_derive_source_interfaces = vec![
        eq_reasoning.source_interface.clone(),
        abstract_ring.source_interface.clone(),
        abstract_square_normalize.source_interface.clone(),
    ];
    let abstract_scalar_derive = build_and_write_module(
        &proof_root,
        &ABSTRACT_SCALAR_DERIVE_MODULE,
        &abstract_scalar_derive_imports,
        &abstract_scalar_derive_source_interfaces,
    )?;
    let abstract_vector_space_imports = vec![
        eq_import.clone(),
        abstract_ring.verified_module.clone(),
        abstract_ordered_field.verified_module.clone(),
        abstract_square_normalize.verified_module.clone(),
    ];
    let abstract_vector_space_source_interfaces = vec![
        abstract_ring.source_interface.clone(),
        abstract_ordered_field.source_interface.clone(),
        abstract_square_normalize.source_interface.clone(),
    ];
    let abstract_vector_space = build_and_write_module(
        &proof_root,
        &ABSTRACT_VECTOR_SPACE_MODULE,
        &abstract_vector_space_imports,
        &abstract_vector_space_source_interfaces,
    )?;
    let abstract_inner_product_imports = vec![
        eq_import.clone(),
        abstract_ring.verified_module.clone(),
        abstract_ordered_field.verified_module.clone(),
        abstract_square_normalize.verified_module.clone(),
        abstract_vector_space.verified_module.clone(),
    ];
    let abstract_inner_product_source_interfaces = vec![
        abstract_ring.source_interface.clone(),
        abstract_ordered_field.source_interface.clone(),
        abstract_square_normalize.source_interface.clone(),
        abstract_vector_space.source_interface.clone(),
    ];
    let abstract_inner_product = build_and_write_module(
        &proof_root,
        &ABSTRACT_INNER_PRODUCT_MODULE,
        &abstract_inner_product_imports,
        &abstract_inner_product_source_interfaces,
    )?;
    let abstract_inner_product_derive_imports = vec![
        eq_import.clone(),
        eq_reasoning.verified_module.clone(),
        abstract_ring.verified_module.clone(),
        abstract_ordered_field.verified_module.clone(),
        abstract_square_normalize.verified_module.clone(),
        abstract_scalar_derive.verified_module.clone(),
        abstract_vector_space.verified_module.clone(),
        abstract_inner_product.verified_module.clone(),
    ];
    let abstract_inner_product_derive_source_interfaces = vec![
        eq_reasoning.source_interface.clone(),
        abstract_ring.source_interface.clone(),
        abstract_ordered_field.source_interface.clone(),
        abstract_square_normalize.source_interface.clone(),
        abstract_scalar_derive.source_interface.clone(),
        abstract_vector_space.source_interface.clone(),
        abstract_inner_product.source_interface.clone(),
    ];
    let abstract_inner_product_derive = build_and_write_module(
        &proof_root,
        &ABSTRACT_INNER_PRODUCT_DERIVE_MODULE,
        &abstract_inner_product_derive_imports,
        &abstract_inner_product_derive_source_interfaces,
    )?;
    let affine_imports = vec![
        eq_import.clone(),
        abstract_ring.verified_module.clone(),
        abstract_ordered_field.verified_module.clone(),
        abstract_square_normalize.verified_module.clone(),
        abstract_vector_space.verified_module.clone(),
        abstract_inner_product.verified_module.clone(),
    ];
    let affine_source_interfaces = vec![
        abstract_ring.source_interface.clone(),
        abstract_ordered_field.source_interface.clone(),
        abstract_square_normalize.source_interface.clone(),
        abstract_vector_space.source_interface.clone(),
        abstract_inner_product.source_interface.clone(),
    ];
    let affine = build_and_write_module(
        &proof_root,
        &AFFINE_MODULE,
        &affine_imports,
        &affine_source_interfaces,
    )?;
    let affine_derive_imports = vec![
        eq_import.clone(),
        abstract_ring.verified_module.clone(),
        abstract_ordered_field.verified_module.clone(),
        abstract_square_normalize.verified_module.clone(),
        abstract_vector_space.verified_module.clone(),
        abstract_inner_product.verified_module.clone(),
        affine.verified_module.clone(),
    ];
    let affine_derive_source_interfaces = vec![
        abstract_ring.source_interface.clone(),
        abstract_ordered_field.source_interface.clone(),
        abstract_square_normalize.source_interface.clone(),
        abstract_vector_space.source_interface.clone(),
        abstract_inner_product.source_interface.clone(),
        affine.source_interface.clone(),
    ];
    let affine_derive = build_and_write_module(
        &proof_root,
        &AFFINE_DERIVE_MODULE,
        &affine_derive_imports,
        &affine_derive_source_interfaces,
    )?;
    let abstract_right_triangle_imports = vec![
        eq_import.clone(),
        abstract_ring.verified_module.clone(),
        abstract_ordered_field.verified_module.clone(),
        abstract_square_normalize.verified_module.clone(),
        abstract_vector_space.verified_module.clone(),
        abstract_inner_product.verified_module.clone(),
        affine.verified_module.clone(),
    ];
    let abstract_right_triangle_source_interfaces = vec![
        abstract_ring.source_interface.clone(),
        abstract_ordered_field.source_interface.clone(),
        abstract_square_normalize.source_interface.clone(),
        abstract_vector_space.source_interface.clone(),
        abstract_inner_product.source_interface.clone(),
        affine.source_interface.clone(),
    ];
    let abstract_right_triangle = build_and_write_module(
        &proof_root,
        &ABSTRACT_RIGHT_TRIANGLE_MODULE,
        &abstract_right_triangle_imports,
        &abstract_right_triangle_source_interfaces,
    )?;
    let abstract_right_triangle_derive_imports = vec![
        eq_import.clone(),
        abstract_ring.verified_module.clone(),
        abstract_ordered_field.verified_module.clone(),
        abstract_square_normalize.verified_module.clone(),
        abstract_vector_space.verified_module.clone(),
        abstract_inner_product.verified_module.clone(),
        abstract_inner_product_derive.verified_module.clone(),
        affine.verified_module.clone(),
        affine_derive.verified_module.clone(),
        abstract_right_triangle.verified_module.clone(),
    ];
    let abstract_right_triangle_derive_source_interfaces = vec![
        abstract_ring.source_interface.clone(),
        abstract_ordered_field.source_interface.clone(),
        abstract_square_normalize.source_interface.clone(),
        abstract_vector_space.source_interface.clone(),
        abstract_inner_product.source_interface.clone(),
        abstract_inner_product_derive.source_interface.clone(),
        affine.source_interface.clone(),
        affine_derive.source_interface.clone(),
        abstract_right_triangle.source_interface.clone(),
    ];
    let abstract_right_triangle_derive = build_and_write_module(
        &proof_root,
        &ABSTRACT_RIGHT_TRIANGLE_DERIVE_MODULE,
        &abstract_right_triangle_derive_imports,
        &abstract_right_triangle_derive_source_interfaces,
    )?;
    let abstract_metric_imports = vec![
        eq_import.clone(),
        abstract_ring.verified_module.clone(),
        abstract_ordered_field.verified_module.clone(),
        abstract_square_normalize.verified_module.clone(),
        abstract_vector_space.verified_module.clone(),
        abstract_inner_product.verified_module.clone(),
        abstract_inner_product_derive.verified_module.clone(),
        affine.verified_module.clone(),
        affine_derive.verified_module.clone(),
        abstract_right_triangle.verified_module.clone(),
    ];
    let abstract_metric_source_interfaces = vec![
        abstract_ring.source_interface.clone(),
        abstract_ordered_field.source_interface.clone(),
        abstract_square_normalize.source_interface.clone(),
        abstract_vector_space.source_interface.clone(),
        abstract_inner_product.source_interface.clone(),
        abstract_inner_product_derive.source_interface.clone(),
        affine.source_interface.clone(),
        affine_derive.source_interface.clone(),
        abstract_right_triangle.source_interface.clone(),
    ];
    let abstract_metric = build_and_write_module(
        &proof_root,
        &ABSTRACT_METRIC_MODULE,
        &abstract_metric_imports,
        &abstract_metric_source_interfaces,
    )?;
    let pythagorean_imports = vec![
        eq_import.clone(),
        eq_reasoning.verified_module.clone(),
        abstract_ring.verified_module.clone(),
        abstract_ordered_field.verified_module.clone(),
        abstract_square_normalize.verified_module.clone(),
        abstract_scalar_derive.verified_module.clone(),
        abstract_vector_space.verified_module.clone(),
        abstract_inner_product.verified_module.clone(),
        abstract_inner_product_derive.verified_module.clone(),
        affine.verified_module.clone(),
        affine_derive.verified_module.clone(),
        abstract_right_triangle.verified_module.clone(),
        abstract_right_triangle_derive.verified_module.clone(),
        abstract_metric.verified_module.clone(),
    ];
    let pythagorean_source_interfaces = vec![
        eq_reasoning.source_interface.clone(),
        abstract_ring.source_interface.clone(),
        abstract_ordered_field.source_interface.clone(),
        abstract_square_normalize.source_interface.clone(),
        abstract_scalar_derive.source_interface.clone(),
        abstract_vector_space.source_interface.clone(),
        abstract_inner_product.source_interface.clone(),
        abstract_inner_product_derive.source_interface.clone(),
        affine.source_interface.clone(),
        affine_derive.source_interface.clone(),
        abstract_right_triangle.source_interface.clone(),
        abstract_right_triangle_derive.source_interface.clone(),
        abstract_metric.source_interface.clone(),
    ];
    let pythagorean = build_and_write_module(
        &proof_root,
        &PYTHAGOREAN_MODULE,
        &pythagorean_imports,
        &pythagorean_source_interfaces,
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
            abstract_group,
            abstract_group_kernel,
            abstract_group_image,
            abstract_group_quotient,
            abstract_group_quotient_mul,
            abstract_group_quotient_group,
            abstract_group_quotient_hom,
            abstract_group_first_iso_full,
            abstract_group_first_iso_image,
            abstract_group_first_iso,
            abstract_group_subgroup,
            abstract_group_normal_quotient,
            abstract_group_normal_quotient_mul,
            abstract_group_normal_quotient_group,
            abstract_group_second_iso_phi,
            abstract_group_second_iso_kernel,
            abstract_group_second_iso_image,
            abstract_group_second_iso_final,
            abstract_group_third_iso,
            abstract_ring,
            abstract_ordered_field,
            abstract_square_normalize,
            abstract_scalar_derive,
            abstract_vector_space,
            abstract_inner_product,
            abstract_inner_product_derive,
            affine,
            affine_derive,
            abstract_right_triangle,
            abstract_right_triangle_derive,
            abstract_metric,
            pythagorean,
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
    let axiom_policy = axiom_policy_for_module(config.module);
    let output = npa_frontend::compile_human_source_to_certificate_output_with_source_interfaces_and_axiom_policy(
        npa_frontend::FileId(0),
        npa_cert::Name::from_dotted(config.module),
        &source,
        verified_modules,
        imported_source_interfaces,
        &npa_frontend::HumanCompileOptions::default(),
        &axiom_policy,
    )
    .map_err(|err| format!("failed to compile {}: {err:?}", config.source_path))?;
    let certificate_bytes = npa_cert::encode_module_cert(&output.certificate)
        .map_err(|err| format!("failed to encode {}: {err:?}", config.certificate_path))?;

    let mut session = npa_cert::VerifierSession::new();
    for import in verified_modules {
        session.register_verified_module(import.clone());
    }
    let verified = npa_cert::verify_module_cert(&certificate_bytes, &mut session, &axiom_policy)
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
    let expected_features = expected_core_features_for_module(config.module);
    if verified.axiom_report().core_features != expected_features {
        return Err(format!(
            "generated AI proof corpus module {} has core features {:?}, expected {:?}",
            config.module,
            verified.axiom_report().core_features,
            expected_features
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

fn axiom_policy_for_module(module: &str) -> npa_cert::AxiomPolicy {
    let mut policy = npa_cert::AxiomPolicy::normal();
    for feature in supported_core_features_for_module(module) {
        policy = policy.with_core_feature(feature);
    }
    policy
}

fn supported_core_features_for_module(module: &str) -> Vec<npa_cert::CoreFeature> {
    if module == ABSTRACT_GROUP_QUOTIENT_MODULE.module
        || module == ABSTRACT_GROUP_QUOTIENT_MUL_MODULE.module
        || module == ABSTRACT_GROUP_NORMAL_QUOTIENT_MODULE.module
        || module == ABSTRACT_GROUP_NORMAL_QUOTIENT_MUL_MODULE.module
        || module == ABSTRACT_GROUP_FIRST_ISO_MODULE.module
    {
        vec![npa_cert::CoreFeature::QuotientV1]
    } else if module == ABSTRACT_GROUP_QUOTIENT_GROUP_MODULE.module
        || module == ABSTRACT_GROUP_QUOTIENT_HOM_MODULE.module
        || module == ABSTRACT_GROUP_FIRST_ISO_FULL_MODULE.module
        || module == ABSTRACT_GROUP_FIRST_ISO_IMAGE_MODULE.module
        || module == ABSTRACT_GROUP_NORMAL_QUOTIENT_GROUP_MODULE.module
        || module == ABSTRACT_GROUP_SECOND_ISO_PHI_MODULE.module
        || module == ABSTRACT_GROUP_SECOND_ISO_KERNEL_MODULE.module
        || module == ABSTRACT_GROUP_SECOND_ISO_IMAGE_MODULE.module
        || module == ABSTRACT_GROUP_SECOND_ISO_FINAL_MODULE.module
        || module == ABSTRACT_GROUP_THIRD_ISO_MODULE.module
    {
        vec![
            npa_cert::CoreFeature::QuotientV1,
            npa_cert::CoreFeature::QuotientV2,
            npa_cert::CoreFeature::QuotientV3,
        ]
    } else {
        Vec::new()
    }
}

fn expected_core_features_for_module(module: &str) -> Vec<npa_cert::CoreFeature> {
    if module == ABSTRACT_GROUP_QUOTIENT_MODULE.module
        || module == ABSTRACT_GROUP_NORMAL_QUOTIENT_MODULE.module
    {
        vec![npa_cert::CoreFeature::QuotientV1]
    } else if module == ABSTRACT_GROUP_QUOTIENT_GROUP_MODULE.module
        || module == ABSTRACT_GROUP_NORMAL_QUOTIENT_GROUP_MODULE.module
    {
        vec![
            npa_cert::CoreFeature::QuotientV1,
            npa_cert::CoreFeature::QuotientV2,
            npa_cert::CoreFeature::QuotientV3,
        ]
    } else if module == ABSTRACT_GROUP_QUOTIENT_HOM_MODULE.module
        || module == ABSTRACT_GROUP_FIRST_ISO_FULL_MODULE.module
        || module == ABSTRACT_GROUP_THIRD_ISO_MODULE.module
    {
        vec![
            npa_cert::CoreFeature::QuotientV1,
            npa_cert::CoreFeature::QuotientV3,
        ]
    } else {
        Vec::new()
    }
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
    let imports = source_imports(config);
    for import in imports {
        source.push_str("import ");
        source.push_str(import);
        source.push('\n');
    }
    if !imports.is_empty() {
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
    if config.module == ABSTRACT_RIGHT_TRIANGLE_DERIVE_MODULE.module {
        source.truncate(source.trim_end_matches('\n').len() + 1);
    }
    source
}

fn source_imports(config: &ModuleArtifact) -> &'static [&'static str] {
    if config.module == ABSTRACT_ORDERED_FIELD_MODULE.module {
        // Eq is verified as a transitive AbstractRing dependency; importing it directly here
        // duplicates the kernel Eq declaration during certificate handoff.
        &["Proofs.Ai.Algebra.AbstractRing"]
    } else if config.module == ABSTRACT_SQUARE_NORMALIZE_MODULE.module {
        &[
            "Proofs.Ai.Algebra.AbstractRing",
            "Proofs.Ai.Algebra.AbstractOrderedField",
        ]
    } else if config.module == ABSTRACT_SCALAR_DERIVE_MODULE.module {
        &[
            "Proofs.Ai.Algebra.AbstractRing",
            "Proofs.Ai.Algebra.AbstractSquareNormalize",
            "Proofs.Ai.EqReasoning",
        ]
    } else if config.module == ABSTRACT_VECTOR_SPACE_MODULE.module {
        &[
            "Proofs.Ai.Algebra.AbstractRing",
            "Proofs.Ai.Algebra.AbstractOrderedField",
            "Proofs.Ai.Algebra.AbstractSquareNormalize",
        ]
    } else if config.module == ABSTRACT_INNER_PRODUCT_MODULE.module {
        &[
            "Proofs.Ai.Algebra.AbstractRing",
            "Proofs.Ai.Algebra.AbstractOrderedField",
            "Proofs.Ai.Algebra.AbstractSquareNormalize",
            "Proofs.Ai.Vector.AbstractSpace",
        ]
    } else if config.module == ABSTRACT_INNER_PRODUCT_DERIVE_MODULE.module {
        &[
            "Proofs.Ai.Algebra.AbstractRing",
            "Proofs.Ai.Algebra.AbstractOrderedField",
            "Proofs.Ai.Algebra.AbstractScalarDerive",
            "Proofs.Ai.EqReasoning",
            "Proofs.Ai.Vector.AbstractSpace",
            "Proofs.Ai.Vector.AbstractInnerProduct",
        ]
    } else if config.module == AFFINE_MODULE.module {
        &[
            "Proofs.Ai.Algebra.AbstractRing",
            "Proofs.Ai.Algebra.AbstractOrderedField",
            "Proofs.Ai.Algebra.AbstractSquareNormalize",
            "Proofs.Ai.Vector.AbstractSpace",
            "Proofs.Ai.Vector.AbstractInnerProduct",
        ]
    } else if config.module == AFFINE_DERIVE_MODULE.module
        || config.module == ABSTRACT_RIGHT_TRIANGLE_MODULE.module
    {
        &[
            "Proofs.Ai.Algebra.AbstractRing",
            "Proofs.Ai.Algebra.AbstractOrderedField",
            "Proofs.Ai.Algebra.AbstractSquareNormalize",
            "Proofs.Ai.Vector.AbstractSpace",
            "Proofs.Ai.Vector.AbstractInnerProduct",
            "Proofs.Ai.Geometry.Affine",
        ]
    } else if config.module == ABSTRACT_RIGHT_TRIANGLE_DERIVE_MODULE.module
        || config.module == ABSTRACT_METRIC_MODULE.module
    {
        &[
            "Proofs.Ai.Algebra.AbstractRing",
            "Proofs.Ai.Algebra.AbstractOrderedField",
            "Proofs.Ai.Algebra.AbstractSquareNormalize",
            "Proofs.Ai.Vector.AbstractSpace",
            "Proofs.Ai.Vector.AbstractInnerProduct",
            "Proofs.Ai.Vector.AbstractInnerProductDerive",
            "Proofs.Ai.Geometry.Affine",
            "Proofs.Ai.Geometry.AffineDerive",
            "Proofs.Ai.Geometry.AbstractRightTriangle",
        ]
    } else if config.module == PYTHAGOREAN_MODULE.module {
        &[
            "Proofs.Ai.EqReasoning",
            "Proofs.Ai.Algebra.AbstractRing",
            "Proofs.Ai.Algebra.AbstractOrderedField",
            "Proofs.Ai.Algebra.AbstractSquareNormalize",
            "Proofs.Ai.Algebra.AbstractScalarDerive",
            "Proofs.Ai.Vector.AbstractSpace",
            "Proofs.Ai.Vector.AbstractInnerProduct",
            "Proofs.Ai.Vector.AbstractInnerProductDerive",
            "Proofs.Ai.Geometry.Affine",
            "Proofs.Ai.Geometry.AffineDerive",
            "Proofs.Ai.Geometry.AbstractRightTriangle",
            "Proofs.Ai.Geometry.AbstractRightTriangleDerive",
            "Proofs.Ai.Geometry.AbstractMetric",
        ]
    } else {
        config.imports
    }
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
