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
    theorems: &'static [TheoremArtifact],
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
}

const BASIC_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Basic",
    source_path: "Proofs/Ai/Basic/source.npa",
    certificate_path: "Proofs/Ai/Basic/certificate.npcert",
    meta_path: "Proofs/Ai/Basic/meta.json",
    replay_path: "Proofs/Ai/Basic/replay.json",
    imports: &[],
    theorems: BASIC_THEOREMS,
};

const EQ_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Eq",
    source_path: "Proofs/Ai/Eq/source.npa",
    certificate_path: "Proofs/Ai/Eq/certificate.npcert",
    meta_path: "Proofs/Ai/Eq/meta.json",
    replay_path: "Proofs/Ai/Eq/replay.json",
    imports: &["Std.Logic.Eq", "Std.Nat.Basic"],
    theorems: EQ_THEOREMS,
};

const NAT_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Nat",
    source_path: "Proofs/Ai/Nat/source.npa",
    certificate_path: "Proofs/Ai/Nat/certificate.npcert",
    meta_path: "Proofs/Ai/Nat/meta.json",
    replay_path: "Proofs/Ai/Nat/replay.json",
    imports: &["Std.Logic.Eq", "Std.Nat.Basic"],
    theorems: NAT_THEOREMS,
};

const PROP_MODULE: ModuleArtifact = ModuleArtifact {
    module: "Proofs.Ai.Prop",
    source_path: "Proofs/Ai/Prop/source.npa",
    certificate_path: "Proofs/Ai/Prop/certificate.npcert",
    meta_path: "Proofs/Ai/Prop/meta.json",
    replay_path: "Proofs/Ai/Prop/replay.json",
    imports: &[],
    theorems: PROP_THEOREMS,
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

    let (eq_import, eq_source_interface) = verified_human_import("Std.Logic.Eq", EQ_IMPORT_SOURCE)?;
    let (nat_import, nat_source_interface) =
        verified_human_import("Std.Nat.Basic", NAT_IMPORT_SOURCE)?;
    let eq_imports = vec![eq_import.clone(), nat_import.clone()];
    let eq_source_interfaces = vec![eq_source_interface.clone(), nat_source_interface.clone()];
    let nat_imports = vec![eq_import, nat_import];
    let nat_source_interfaces = vec![eq_source_interface, nat_source_interface];

    let basic = build_and_write_module(&proof_root, &BASIC_MODULE, &[], &[])?;
    let eq = build_and_write_module(&proof_root, &EQ_MODULE, &eq_imports, &eq_source_interfaces)?;
    let nat = build_and_write_module(
        &proof_root,
        &NAT_MODULE,
        &nat_imports,
        &nat_source_interfaces,
    )?;
    let prop = build_and_write_module(&proof_root, &PROP_MODULE, &[], &[])?;

    write(
        proof_root.join(MANIFEST_PATH),
        manifest_toml(&[basic, eq, nat, prop]).as_bytes(),
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
    if !verified.axiom_report().module_axioms.is_empty() {
        return Err(format!(
            "generated AI proof corpus module {} unexpectedly depends on axioms",
            config.module
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

fn verified_human_import(
    module: &str,
    source: &str,
) -> Result<
    (
        npa_cert::VerifiedModule,
        npa_frontend::HumanImportedSourceInterface,
    ),
    String,
> {
    let output = npa_frontend::compile_human_source_to_certificate_output_with_source_interfaces(
        npa_frontend::FileId(0),
        npa_cert::Name::from_dotted(module),
        source,
        &[],
        &[],
        &npa_frontend::HumanCompileOptions::default(),
    )
    .map_err(|err| format!("failed to compile import {module}: {err:?}"))?;
    let bytes = npa_cert::encode_module_cert(&output.certificate)
        .map_err(|err| format!("failed to encode import {module}: {err:?}"))?;
    let verified = npa_cert::verify_module_cert(
        &bytes,
        &mut npa_cert::VerifierSession::new(),
        &npa_cert::AxiomPolicy::normal(),
    )
    .map_err(|err| format!("import {module} did not verify: {err:?}"))?;
    let import = npa_frontend::VerifiedImport::from(&verified);
    let source_interface = npa_frontend::HumanImportedSourceInterface {
        module: import.module,
        export_hash: import.export_hash,
        certificate_hash: import.certificate_hash,
        source_interface: output.source_interface,
    };

    Ok((verified, source_interface))
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
        manifest.push_str("trusted_status = \"verified_by_phase2_certificate\"\n");
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
        manifest.push_str("axioms = []\n");
    }
    manifest
}

fn meta_json(module: &GeneratedModule) -> String {
    let declarations = module
        .config
        .theorems
        .iter()
        .map(|theorem| {
            format!(
                "    {{ \"name\": \"{}\", \"kind\": \"theorem\" }}",
                theorem.name
            )
        })
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
  \"trusted_status\": \"verified_by_phase2_certificate\",
  \"source_sha256\": \"{}\",
  \"certificate_file_sha256\": \"{}\",
  \"export_hash\": \"{}\",
  \"axiom_report_hash\": \"{}\",
  \"certificate_hash\": \"{}\",
  \"imports\": [{}],
  \"axioms\": [],
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
        declarations
    )
}

fn replay_json(config: &ModuleArtifact) -> String {
    let steps = config
        .theorems
        .iter()
        .map(|theorem| {
            format!(
                "    {{
      \"declaration\": \"{}\",
      \"source_kind\": \"explicit_term\",
      \"term\": \"{}\"
    }}",
                theorem.name, theorem.proof
            )
        })
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

fn quoted_items(items: &[&str]) -> String {
    items
        .iter()
        .map(|item| format!("\"{item}\""))
        .collect::<Vec<_>>()
        .join(", ")
}
