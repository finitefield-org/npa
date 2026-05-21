use npa_cert::{
    decode_module_cert, verify_module_cert, AxiomPolicy, DeclPayload, ExportKind, Name,
    VerifierSession,
};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

const EXPECTED_THEOREMS: &[&str] = &[
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

#[test]
fn ai_basic_certificate_matches_manifest_and_verifies() {
    let root = corpus_root();
    let manifest = read_to_string(root.join("manifest.toml"));
    assert_eq!(
        quoted_value(&manifest, "schema"),
        "npa-ai-proof-corpus-v0.1"
    );

    let module = quoted_value(&manifest, "module");
    let source = quoted_value(&manifest, "source");
    let certificate = quoted_value(&manifest, "certificate");
    let meta = quoted_value(&manifest, "meta");
    let replay = quoted_value(&manifest, "replay");
    let source_sha256 = quoted_value(&manifest, "source_sha256");
    let certificate_file_sha256 = quoted_value(&manifest, "certificate_file_sha256");
    let export_hash = quoted_value(&manifest, "export_hash");
    let axiom_report_hash = quoted_value(&manifest, "axiom_report_hash");
    let certificate_hash = quoted_value(&manifest, "certificate_hash");

    assert_eq!(module, "Proofs.Ai.Basic");
    assert_eq!(
        quoted_value(&manifest, "trusted_status"),
        "verified_by_phase2_certificate"
    );
    assert_eq!(source, "Proofs/Ai/Basic/source.npa");
    assert_eq!(certificate, "Proofs/Ai/Basic/certificate.npcert");
    assert_eq!(meta, "Proofs/Ai/Basic/meta.json");
    assert_eq!(replay, "Proofs/Ai/Basic/replay.json");

    let source_bytes = read(root.join(source));
    assert_eq!(tagged_sha256(&source_bytes), source_sha256);

    let certificate_bytes = read(root.join(certificate));
    assert_eq!(tagged_sha256(&certificate_bytes), certificate_file_sha256);

    let decoded = decode_module_cert(&certificate_bytes).expect("AI corpus certificate decodes");
    assert_eq!(decoded.header.module, Name::from_dotted(&module));
    assert_eq!(tagged_hash(decoded.hashes.export_hash), export_hash);
    assert_eq!(
        tagged_hash(decoded.hashes.axiom_report_hash),
        axiom_report_hash
    );
    assert_eq!(
        tagged_hash(decoded.hashes.certificate_hash),
        certificate_hash
    );
    assert!(decoded.axiom_report.module_axioms.is_empty());

    let mut session = VerifierSession::new();
    let verified = verify_module_cert(&certificate_bytes, &mut session, &AxiomPolicy::normal())
        .expect("AI corpus certificate verifies");
    assert_eq!(verified.module(), &Name::from_dotted(&module));
    assert_eq!(tagged_hash(verified.export_hash()), export_hash);
    assert_eq!(tagged_hash(verified.certificate_hash()), certificate_hash);
    assert!(verified.axiom_report().module_axioms.is_empty());

    assert_theorem_exports(&decoded, EXPECTED_THEOREMS);
    assert_theorem_declarations(&decoded, EXPECTED_THEOREMS);

    let meta = read_to_string(root.join(meta));
    assert!(meta.contains(&format!("\"certificate_hash\": \"{certificate_hash}\"")));
    assert!(meta.contains("\"trusted_status\": \"verified_by_phase2_certificate\""));
    for theorem in EXPECTED_THEOREMS {
        assert!(meta.contains(&format!("\"name\": \"{theorem}\"")));
        assert!(manifest.contains(&format!("\"{theorem}\"")));
    }

    let replay = read_to_string(root.join(replay));
    assert!(replay.contains("\"trusted\": false"));
    assert!(replay.contains("\"accepted_artifact\": \"Proofs/Ai/Basic/certificate.npcert\""));
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

fn assert_theorem_declarations(cert: &npa_cert::ModuleCert, expected: &[&str]) {
    let mut actual = cert
        .declarations
        .iter()
        .map(|decl| match &decl.decl {
            DeclPayload::Theorem { name, .. } => cert.name_table[*name].as_dotted(),
            other => panic!("AI proof corpus should contain only theorem declarations: {other:?}"),
        })
        .collect::<Vec<_>>();
    actual.sort();

    let mut expected = expected
        .iter()
        .map(|name| (*name).to_owned())
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
