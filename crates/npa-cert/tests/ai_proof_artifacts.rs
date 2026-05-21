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
    theorems: &'static [&'static str],
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

const EXPECTED_MODULES: &[ExpectedModule] = &[
    ExpectedModule {
        module: "Proofs.Ai.Basic",
        source: "Proofs/Ai/Basic/source.npa",
        certificate: "Proofs/Ai/Basic/certificate.npcert",
        meta: "Proofs/Ai/Basic/meta.json",
        replay: "Proofs/Ai/Basic/replay.json",
        imports: &[],
        theorems: BASIC_THEOREMS,
    },
    ExpectedModule {
        module: "Proofs.Ai.Eq",
        source: "Proofs/Ai/Eq/source.npa",
        certificate: "Proofs/Ai/Eq/certificate.npcert",
        meta: "Proofs/Ai/Eq/meta.json",
        replay: "Proofs/Ai/Eq/replay.json",
        imports: &["Std.Logic.Eq", "Std.Nat.Basic"],
        theorems: EQ_THEOREMS,
    },
    ExpectedModule {
        module: "Proofs.Ai.Nat",
        source: "Proofs/Ai/Nat/source.npa",
        certificate: "Proofs/Ai/Nat/certificate.npcert",
        meta: "Proofs/Ai/Nat/meta.json",
        replay: "Proofs/Ai/Nat/replay.json",
        imports: &["Std.Logic.Eq", "Std.Nat.Basic"],
        theorems: NAT_THEOREMS,
    },
    ExpectedModule {
        module: "Proofs.Ai.Prop",
        source: "Proofs/Ai/Prop/source.npa",
        certificate: "Proofs/Ai/Prop/certificate.npcert",
        meta: "Proofs/Ai/Prop/meta.json",
        replay: "Proofs/Ai/Prop/replay.json",
        imports: &[],
        theorems: PROP_THEOREMS,
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

    for expected in EXPECTED_MODULES {
        let block = manifest_block(&manifest, expected.module);
        assert_eq!(
            quoted_value(block, "trusted_status"),
            "verified_by_phase2_certificate"
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
        assert!(decoded.axiom_report.module_axioms.is_empty());
        assert_imports(&decoded, expected.imports);

        let mut session = VerifierSession::new();
        register_expected_imports(&mut session, expected.imports, &eq_import, &nat_import);
        let verified = verify_module_cert(&certificate_bytes, &mut session, &AxiomPolicy::normal())
            .expect("AI corpus certificate verifies");
        assert_eq!(verified.module(), &Name::from_dotted(expected.module));
        assert_eq!(tagged_hash(verified.export_hash()), export_hash);
        assert_eq!(tagged_hash(verified.certificate_hash()), certificate_hash);
        assert!(verified.axiom_report().module_axioms.is_empty());

        assert_theorem_exports(&decoded, expected.theorems);
        assert_theorem_declarations(&decoded, expected.theorems);

        let meta = read_to_string(root.join(expected.meta));
        assert!(meta.contains(&format!("\"certificate_hash\": \"{certificate_hash}\"")));
        assert!(meta.contains("\"trusted_status\": \"verified_by_phase2_certificate\""));
        for import in expected.imports {
            assert!(meta.contains(&format!("\"{import}\"")));
        }
        for theorem in expected.theorems {
            assert!(meta.contains(&format!("\"name\": \"{theorem}\"")));
            assert!(block.contains(&format!("\"{theorem}\"")));
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
    eq_import: &VerifiedModule,
    nat_import: &VerifiedModule,
) {
    for import in imports {
        match *import {
            "Std.Logic.Eq" => session.register_verified_module(eq_import.clone()),
            "Std.Nat.Basic" => session.register_verified_module(nat_import.clone()),
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
