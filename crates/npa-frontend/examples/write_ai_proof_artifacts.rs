use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const MODULE: &str = "Proofs.Ai.Basic";
const SOURCE_PATH: &str = "Proofs/Ai/Basic/source.npa";
const CERTIFICATE_PATH: &str = "Proofs/Ai/Basic/certificate.npcert";
const META_PATH: &str = "Proofs/Ai/Basic/meta.json";
const REPLAY_PATH: &str = "Proofs/Ai/Basic/replay.json";
const MANIFEST_PATH: &str = "manifest.toml";

struct TheoremArtifact {
    name: &'static str,
    statement: &'static str,
    proof: &'static str,
}

const THEOREMS: &[TheoremArtifact] = &[
    TheoremArtifact {
        name: "id",
        statement: "forall (A : Type), forall (x : A), A",
        proof: "fun A => fun x => x",
    },
    TheoremArtifact {
        name: "const_left",
        statement: "forall (A : Type), forall (B : Type), forall (x : A), forall (y : B), A",
        proof: "fun A => fun B => fun x => fun y => x",
    },
    TheoremArtifact {
        name: "const_right",
        statement: "forall (A : Type), forall (B : Type), forall (x : A), forall (y : B), B",
        proof: "fun A => fun B => fun x => fun y => y",
    },
    TheoremArtifact {
        name: "apply_fn",
        statement:
            "forall (A : Type), forall (B : Type), forall (f : forall (x : A), B), forall (x : A), B",
        proof: "fun A => fun B => fun f => fun x => f x",
    },
    TheoremArtifact {
        name: "compose",
        statement:
            "forall (A : Type), forall (B : Type), forall (C : Type), forall (f : forall (x : B), C), forall (g : forall (x : A), B), forall (x : A), C",
        proof: "fun A => fun B => fun C => fun f => fun g => fun x => f (g x)",
    },
    TheoremArtifact {
        name: "flip",
        statement:
            "forall (A : Type), forall (B : Type), forall (C : Type), forall (f : forall (x : A), forall (y : B), C), forall (y : B), forall (x : A), C",
        proof: "fun A => fun B => fun C => fun f => fun y => fun x => f x y",
    },
    TheoremArtifact {
        name: "duplicate",
        statement:
            "forall (A : Type), forall (B : Type), forall (f : forall (x : A), forall (y : A), B), forall (x : A), B",
        proof: "fun A => fun B => fun f => fun x => f x x",
    },
    TheoremArtifact {
        name: "prop_id",
        statement: "forall (P : Prop), forall (p : P), P",
        proof: "fun P => fun p => p",
    },
    TheoremArtifact {
        name: "modus_ponens",
        statement:
            "forall (P : Prop), forall (Q : Prop), forall (h : forall (p : P), Q), forall (p : P), Q",
        proof: "fun P => fun Q => fun h => fun p => h p",
    },
    TheoremArtifact {
        name: "imp_trans",
        statement:
            "forall (P : Prop), forall (Q : Prop), forall (R : Prop), forall (pq : forall (p : P), Q), forall (qr : forall (q : Q), R), forall (p : P), R",
        proof: "fun P => fun Q => fun R => fun pq => fun qr => fun p => qr (pq p)",
    },
    TheoremArtifact {
        name: "compose_assoc",
        statement:
            "forall (A : Type), forall (B : Type), forall (C : Type), forall (D : Type), forall (h : forall (x : C), D), forall (g : forall (x : B), C), forall (f : forall (x : A), B), forall (x : A), D",
        proof: "fun A => fun B => fun C => fun D => fun h => fun g => fun f => fun x => h (g (f x))",
    },
    TheoremArtifact {
        name: "apply_twice",
        statement:
            "forall (A : Type), forall (f : forall (x : A), A), forall (x : A), A",
        proof: "fun A => fun f => fun x => f (f x)",
    },
    TheoremArtifact {
        name: "ignore_middle",
        statement:
            "forall (A : Type), forall (B : Type), forall (C : Type), forall (x : A), forall (y : B), forall (z : C), A",
        proof: "fun A => fun B => fun C => fun x => fun y => fun z => x",
    },
    TheoremArtifact {
        name: "select_middle",
        statement:
            "forall (A : Type), forall (B : Type), forall (C : Type), forall (x : A), forall (y : B), forall (z : C), B",
        proof: "fun A => fun B => fun C => fun x => fun y => fun z => y",
    },
    TheoremArtifact {
        name: "select_last",
        statement:
            "forall (A : Type), forall (B : Type), forall (C : Type), forall (x : A), forall (y : B), forall (z : C), C",
        proof: "fun A => fun B => fun C => fun x => fun y => fun z => z",
    },
    TheoremArtifact {
        name: "imp_swap",
        statement:
            "forall (P : Prop), forall (Q : Prop), forall (R : Prop), forall (h : forall (p : P), forall (q : Q), R), forall (q : Q), forall (p : P), R",
        proof: "fun P => fun Q => fun R => fun h => fun q => fun p => h p q",
    },
    TheoremArtifact {
        name: "imp_compose",
        statement:
            "forall (P : Prop), forall (Q : Prop), forall (R : Prop), forall (qr : forall (q : Q), R), forall (pq : forall (p : P), Q), forall (p : P), R",
        proof: "fun P => fun Q => fun R => fun qr => fun pq => fun p => qr (pq p)",
    },
    TheoremArtifact {
        name: "imp_ignore",
        statement: "forall (P : Prop), forall (Q : Prop), forall (p : P), forall (q : Q), P",
        proof: "fun P => fun Q => fun p => fun q => p",
    },
    TheoremArtifact {
        name: "imp_duplicate",
        statement:
            "forall (P : Prop), forall (Q : Prop), forall (h : forall (p1 : P), forall (p2 : P), Q), forall (p : P), Q",
        proof: "fun P => fun Q => fun h => fun p => h p p",
    },
    TheoremArtifact {
        name: "higher_apply",
        statement:
            "forall (A : Type), forall (B : Type), forall (C : Type), forall (h : forall (f : forall (x : A), B), C), forall (f : forall (x : A), B), C",
        proof: "fun A => fun B => fun C => fun h => fun f => h f",
    },
];

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
    let source = basic_source();

    let certificate = npa_frontend::compile_human_source_to_certificate(
        npa_frontend::FileId(0),
        npa_cert::Name::from_dotted(MODULE),
        &source,
        &[],
        &npa_frontend::HumanCompileOptions::default(),
    )
    .map_err(|err| format!("failed to compile {SOURCE_PATH}: {err:?}"))?;
    let certificate_bytes = npa_cert::encode_module_cert(&certificate)
        .map_err(|err| format!("failed to encode {CERTIFICATE_PATH}: {err:?}"))?;

    let mut session = npa_cert::VerifierSession::new();
    let verified = npa_cert::verify_module_cert(
        &certificate_bytes,
        &mut session,
        &npa_cert::AxiomPolicy::normal(),
    )
    .map_err(|err| format!("generated certificate did not verify: {err:?}"))?;
    if !verified.axiom_report().module_axioms.is_empty() {
        return Err("generated AI proof corpus unexpectedly depends on axioms".to_owned());
    }

    write(proof_root.join(SOURCE_PATH), source.as_bytes())?;
    write(proof_root.join(CERTIFICATE_PATH), &certificate_bytes)?;

    let source_sha256 = tagged_sha256(source.as_bytes());
    let certificate_file_sha256 = tagged_sha256(&certificate_bytes);
    let export_hash = tagged_hash(certificate.hashes.export_hash);
    let axiom_report_hash = tagged_hash(certificate.hashes.axiom_report_hash);
    let certificate_hash = tagged_hash(certificate.hashes.certificate_hash);

    write(
        proof_root.join(META_PATH),
        meta_json(
            &source_sha256,
            &certificate_file_sha256,
            &export_hash,
            &axiom_report_hash,
            &certificate_hash,
        )
        .as_bytes(),
    )?;
    write(proof_root.join(REPLAY_PATH), replay_json().as_bytes())?;
    write(
        proof_root.join(MANIFEST_PATH),
        manifest_toml(
            &source_sha256,
            &certificate_file_sha256,
            &export_hash,
            &axiom_report_hash,
            &certificate_hash,
        )
        .as_bytes(),
    )?;

    Ok(())
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

fn basic_source() -> String {
    let mut source = String::new();
    for theorem in THEOREMS {
        source.push_str("theorem ");
        source.push_str(theorem.name);
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

fn manifest_toml(
    source_sha256: &str,
    certificate_file_sha256: &str,
    export_hash: &str,
    axiom_report_hash: &str,
    certificate_hash: &str,
) -> String {
    let theorem_names = THEOREMS
        .iter()
        .map(|theorem| format!("\"{}\"", theorem.name))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "\
schema = \"npa-ai-proof-corpus-v0.1\"

[[proof_modules]]
module = \"{MODULE}\"
source = \"{SOURCE_PATH}\"
certificate = \"{CERTIFICATE_PATH}\"
meta = \"{META_PATH}\"
replay = \"{REPLAY_PATH}\"
producer_profile = \"human-surface-explicit-term\"
trusted_status = \"verified_by_phase2_certificate\"
source_sha256 = \"{source_sha256}\"
certificate_file_sha256 = \"{certificate_file_sha256}\"
export_hash = \"{export_hash}\"
axiom_report_hash = \"{axiom_report_hash}\"
certificate_hash = \"{certificate_hash}\"
theorems = [{theorem_names}]
axioms = []
"
    )
}

fn meta_json(
    source_sha256: &str,
    certificate_file_sha256: &str,
    export_hash: &str,
    axiom_report_hash: &str,
    certificate_hash: &str,
) -> String {
    let declarations = THEOREMS
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
  \"module\": \"{MODULE}\",
  \"source\": \"{SOURCE_PATH}\",
  \"certificate\": \"{CERTIFICATE_PATH}\",
  \"producer_profile\": \"human-surface-explicit-term\",
  \"trusted_status\": \"verified_by_phase2_certificate\",
  \"source_sha256\": \"{source_sha256}\",
  \"certificate_file_sha256\": \"{certificate_file_sha256}\",
  \"export_hash\": \"{export_hash}\",
  \"axiom_report_hash\": \"{axiom_report_hash}\",
  \"certificate_hash\": \"{certificate_hash}\",
  \"axioms\": [],
  \"declarations\": [
{declarations}
  ],
  \"trust_boundary\": \"source, replay, and metadata are non-trusted sidecars; only the canonical certificate verified by npa-cert is accepted\"
}}
"
    )
}

fn replay_json() -> String {
    let steps = THEOREMS
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
  \"module\": \"{MODULE}\",
  \"trusted\": false,
  \"profile\": \"explicit_term_source_certificate_handoff\",
  \"steps\": [
{steps}
  ],
  \"acceptance\": {{
    \"required\": [\"decode_module_cert\", \"verify_module_cert\"],
    \"accepted_artifact\": \"{CERTIFICATE_PATH}\"
  }}
}}
"
    )
}
