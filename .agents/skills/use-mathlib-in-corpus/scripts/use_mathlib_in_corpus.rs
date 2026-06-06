use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

#[derive(Clone, Debug, Default)]
struct Entry {
    module: String,
    package: String,
    version: String,
    certificate: String,
    expected_export_hash: String,
    expected_certificate_hash: String,
    imports: Vec<String>,
    definitions: Vec<String>,
    theorems: Vec<String>,
    axioms: Vec<String>,
}

#[derive(Debug, Default)]
struct Manifest {
    version: String,
    imports: Vec<Entry>,
    modules: Vec<Entry>,
}

#[derive(Debug)]
struct Options {
    command: String,
    mathlib_root: Option<String>,
    prefix: String,
    apply: bool,
    positional: Vec<String>,
}

fn usage() {
    println!(
        "Usage:\n  use_mathlib_in_corpus.sh scan [options]\n  use_mathlib_in_corpus.sh pin Mathlib.Module... [options]\n\nOptions:\n  --mathlib-root PATH   npa-mathlib checkout path (default: ../npa-mathlib)\n  --prefix MODULE       scan corpus source prefix (default: Proofs.Ai.)\n  --apply               write changes for pin\n  -h, --help            show this help\n\nCommands:\n  scan                  report Proofs.Ai.* imports with likely Mathlib.* counterparts\n  pin                   vendor npa-mathlib certificates and add missing package imports"
    );
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let command = args.get(1).cloned().unwrap_or_else(|| "help".to_string());
    if command == "help" || command == "-h" || command == "--help" {
        return Ok(Options {
            command: "help".to_string(),
            mathlib_root: None,
            prefix: "Proofs.Ai.".to_string(),
            apply: false,
            positional: Vec::new(),
        });
    }

    let mut options = Options {
        command,
        mathlib_root: None,
        prefix: "Proofs.Ai.".to_string(),
        apply: false,
        positional: Vec::new(),
    };

    let mut index = 2;
    while index < args.len() {
        match args[index].as_str() {
            "--mathlib-root" => {
                index += 1;
                options.mathlib_root = args.get(index).cloned();
                if options.mathlib_root.is_none() {
                    return Err("--mathlib-root requires a path".to_string());
                }
            }
            "--prefix" => {
                index += 1;
                options.prefix = args
                    .get(index)
                    .cloned()
                    .ok_or_else(|| "--prefix requires a module prefix".to_string())?;
            }
            "--apply" => options.apply = true,
            "-h" | "--help" => {
                options.command = "help".to_string();
                return Ok(options);
            }
            other if other.starts_with("--") => return Err(format!("unknown option: {other}")),
            other => options.positional.push(other.to_string()),
        }
        index += 1;
    }

    Ok(options)
}

fn repo_root(script_dir: &Path) -> PathBuf {
    let output = Command::new("git")
        .arg("-C")
        .arg(script_dir)
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output();
    if let Ok(output) = output {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !text.is_empty() {
                return PathBuf::from(text);
            }
        }
    }
    script_dir
        .join("../../../..")
        .canonicalize()
        .unwrap_or_else(|_| script_dir.to_path_buf())
}

fn parse_list(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(start) = rest.find('"') {
        rest = &rest[start + 1..];
        if let Some(end) = rest.find('"') {
            out.push(rest[..end].to_string());
            rest = &rest[end + 1..];
        } else {
            break;
        }
    }
    out
}

fn parse_string_assignment(line: &str) -> Option<(&str, String)> {
    let (key, raw_value) = line.split_once('=')?;
    let key = key.trim();
    let value = raw_value.trim();
    if !value.starts_with('"') {
        return None;
    }
    let value = value.trim_matches('"').to_string();
    Some((key, value))
}

fn parse_manifest(path: &Path) -> io::Result<Manifest> {
    let text = fs::read_to_string(path)?;
    let mut manifest = Manifest::default();
    let mut section = "";
    let mut current = Entry::default();
    let mut has_current = false;

    fn flush(section: &str, current: &mut Entry, has_current: &mut bool, manifest: &mut Manifest) {
        if !*has_current || current.module.is_empty() {
            *has_current = false;
            *current = Entry::default();
            return;
        }
        if section == "imports" {
            manifest.imports.push(current.clone());
        } else if section == "modules" {
            manifest.modules.push(current.clone());
        }
        *has_current = false;
        *current = Entry::default();
    }

    for raw_line in text.lines() {
        let line = raw_line.trim();
        match line {
            "[[imports]]" => {
                flush(section, &mut current, &mut has_current, &mut manifest);
                section = "imports";
                has_current = true;
            }
            "[[modules]]" => {
                flush(section, &mut current, &mut has_current, &mut manifest);
                section = "modules";
                has_current = true;
            }
            _ => {
                if let Some((key, value)) = parse_string_assignment(line) {
                    if section.is_empty() && key == "version" {
                        manifest.version = value.clone();
                    }
                    match key {
                        "module" => current.module = value,
                        "package" => current.package = value,
                        "version" => current.version = value,
                        "certificate" => current.certificate = value,
                        "expected_export_hash" | "export_hash" => {
                            current.expected_export_hash = value
                        }
                        "expected_certificate_hash" | "certificate_hash" => {
                            current.expected_certificate_hash = value
                        }
                        _ => {}
                    }
                } else if line.starts_with("imports") {
                    current.imports = parse_list(line);
                } else if line.starts_with("definitions") {
                    current.definitions = parse_list(line);
                } else if line.starts_with("theorems") {
                    current.theorems = parse_list(line);
                } else if line.starts_with("axioms") {
                    current.axioms = parse_list(line);
                }
            }
        }
    }
    flush(section, &mut current, &mut has_current, &mut manifest);
    Ok(manifest)
}

fn signature(entry: &Entry) -> String {
    format!(
        "{}|{}",
        entry.definitions.join(","),
        entry.theorems.join(",")
    )
}

fn build_counterpart_map(corpus: &Manifest, mathlib: &Manifest) -> BTreeMap<String, String> {
    let mut by_export = BTreeMap::new();
    let mut by_signature = BTreeMap::new();
    for entry in &mathlib.modules {
        if !entry.expected_export_hash.is_empty() {
            by_export.insert(entry.expected_export_hash.clone(), entry.module.clone());
        }
        let sig = signature(entry);
        if sig != "|" {
            by_signature.insert(sig, entry.module.clone());
        }
    }

    let mut out = BTreeMap::new();
    for entry in &corpus.modules {
        if !entry.module.starts_with("Proofs.Ai.") {
            continue;
        }
        let target = if !entry.expected_export_hash.is_empty() {
            by_export.get(&entry.expected_export_hash).cloned()
        } else {
            None
        }
        .or_else(|| by_signature.get(&signature(entry)).cloned());

        if let Some(target) = target {
            out.insert(entry.module.clone(), target);
        }
    }
    out
}

fn walk_sources(root: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_sources(&path, out)?;
        } else if path.file_name().and_then(|name| name.to_str()) == Some("source.npa") {
            out.push(path);
        }
    }
    out.sort();
    Ok(())
}

fn module_from_source(npa_root: &Path, source: &Path) -> String {
    let rel = source
        .strip_prefix(npa_root.join("proofs"))
        .unwrap_or(source);
    rel.to_string_lossy()
        .trim_end_matches("/source.npa")
        .replace('/', ".")
}

fn scan(
    npa_root: &Path,
    mathlib_root: &Path,
    corpus: &Manifest,
    mathlib: &Manifest,
    prefix: &str,
) -> io::Result<()> {
    let counterparts = build_counterpart_map(corpus, mathlib);
    let mut sources = Vec::new();
    walk_sources(&npa_root.join("proofs/Proofs/Ai"), &mut sources)?;

    let mut grouped: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for source in sources {
        let module = module_from_source(npa_root, &source);
        if !module.starts_with(prefix) {
            continue;
        }
        let rel = source
            .strip_prefix(npa_root)
            .unwrap_or(&source)
            .to_string_lossy();
        for (line_index, raw_line) in fs::read_to_string(&source)?.lines().enumerate() {
            let line = raw_line.trim();
            if let Some(imported) = line.strip_prefix("import ") {
                if let Some(target) = counterparts.get(imported.trim()) {
                    let key = format!("{} -> {}", imported.trim(), target);
                    grouped.entry(key).or_default().push(format!(
                        "{}:{} ({})",
                        rel,
                        line_index + 1,
                        module
                    ));
                }
            }
        }
    }

    let count: usize = grouped.values().map(Vec::len).sum();
    println!("# Proof corpus npa-mathlib migration scan");
    println!("npa_root: {}", npa_root.display());
    println!("mathlib_root: {}", mathlib_root.display());
    println!("prefix: {prefix}");
    println!("counterparts: {}", counterparts.len());
    println!("findings: {count}");
    println!("note: scan output is a migration hint; verify source-free after editing.");
    for (mapping, files) in grouped {
        println!("\n{mapping}");
        for file in files.iter().take(12) {
            println!("  {file}");
        }
        if files.len() > 12 {
            println!("  ... {} more", files.len() - 12);
        }
    }
    if count == 0 {
        println!("\nNo Proofs.Ai.* imports with detected Mathlib.* counterparts.");
    }
    Ok(())
}

fn module_cert_rel(module: &str) -> String {
    format!("{}/certificate.npcert", module.replace('.', "/"))
}

fn import_stanza(entry: &Entry, mathlib_version: &str) -> String {
    format!(
        "[[imports]]\nmodule = \"{}\"\npackage = \"npa-mathlib\"\nversion = \"{}\"\ncertificate = \"vendor/npa-mathlib/{}\"\nexport_hash = \"{}\"\ncertificate_hash = \"{}\"\n",
        entry.module,
        mathlib_version,
        module_cert_rel(&entry.module),
        entry.expected_export_hash,
        entry.expected_certificate_hash
    )
}

fn insert_import_stanzas(text: &str, stanzas: &[String]) -> String {
    let insertion = format!("{}\n", stanzas.join("\n"));
    if let Some(index) = text.find("\n[[modules]]") {
        let before = text[..index].trim_end();
        let after = &text[index + 1..];
        format!("{before}\n\n{insertion}{after}")
    } else {
        format!("{}\n\n{insertion}", text.trim_end())
    }
}

fn pin(
    npa_root: &Path,
    mathlib_root: &Path,
    corpus_manifest_path: &Path,
    corpus: &Manifest,
    mathlib: &Manifest,
    requested: &[String],
    apply: bool,
) -> io::Result<ExitCode> {
    if requested.is_empty() {
        eprintln!("pin requires at least one Mathlib.* module");
        return Ok(ExitCode::from(2));
    }

    let mathlib_by_module: BTreeMap<_, _> = mathlib
        .modules
        .iter()
        .map(|entry| (entry.module.as_str(), entry))
        .collect();
    let existing_imports: BTreeSet<_> = corpus
        .imports
        .iter()
        .map(|entry| entry.module.as_str())
        .collect();
    let mut stanzas = Vec::new();
    let mut copies = Vec::new();

    for module in requested {
        let Some(entry) = mathlib_by_module.get(module.as_str()) else {
            eprintln!("npa-mathlib module not found: {module}");
            return Ok(ExitCode::from(1));
        };
        if entry.expected_export_hash.is_empty() || entry.expected_certificate_hash.is_empty() {
            eprintln!("npa-mathlib module lacks expected hashes: {module}");
            return Ok(ExitCode::from(1));
        }
        let source_cert = mathlib_root.join(if entry.certificate.is_empty() {
            module_cert_rel(module)
        } else {
            entry.certificate.clone()
        });
        if !source_cert.exists() {
            eprintln!(
                "certificate not found for {module}: {}",
                source_cert.display()
            );
            return Ok(ExitCode::from(1));
        }
        let dest_rel = format!("vendor/npa-mathlib/{}", module_cert_rel(module));
        let dest = npa_root.join("proofs").join(&dest_rel);
        copies.push((module.clone(), source_cert, dest, dest_rel));
        if !existing_imports.contains(module.as_str()) {
            stanzas.push(import_stanza(entry, &mathlib.version));
        }
    }

    println!("# npa-mathlib corpus import pin plan");
    println!("npa_root: {}", npa_root.display());
    println!("mathlib_root: {}", mathlib_root.display());
    println!("apply: {}", if apply { "true" } else { "false" });
    for (module, source, _dest, dest_rel) in &copies {
        println!("\ncopy {module}");
        println!("  from: {}", source.display());
        println!("  to:   proofs/{dest_rel}");
    }
    if stanzas.is_empty() {
        println!("\nmanifest stanzas: all requested imports already exist");
    } else {
        println!("\nmanifest stanzas:");
        println!("{}", stanzas.join("\n"));
    }

    if !apply {
        println!("\ndry-run only; rerun with --apply to write files");
        return Ok(ExitCode::SUCCESS);
    }

    for (_module, source, dest, _dest_rel) in &copies {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source, dest)?;
    }
    if !stanzas.is_empty() {
        let text = fs::read_to_string(corpus_manifest_path)?;
        fs::write(corpus_manifest_path, insert_import_stanzas(&text, &stanzas))?;
    }
    println!("\napplied");
    Ok(ExitCode::SUCCESS)
}

fn run() -> Result<ExitCode, String> {
    let args: Vec<String> = env::args().collect();
    let script_dir = PathBuf::from(
        args.get(1)
            .ok_or_else(|| "missing script_dir argument".to_string())?,
    );
    let shifted_args: Vec<String> = std::iter::once(args[0].clone())
        .chain(args.iter().skip(2).cloned())
        .collect();
    let options = parse_args(&shifted_args)?;
    if options.command == "help" {
        usage();
        return Ok(ExitCode::SUCCESS);
    }

    let npa_root = repo_root(&script_dir);
    let mathlib_root = npa_root.join(options.mathlib_root.as_deref().unwrap_or("../npa-mathlib"));
    let corpus_manifest_path = npa_root.join("proofs/npa-package.toml");
    let mathlib_manifest_path = mathlib_root.join("npa-package.toml");
    if !corpus_manifest_path.exists() {
        return Err(format!(
            "missing corpus manifest: {}",
            corpus_manifest_path.display()
        ));
    }
    if !mathlib_manifest_path.exists() {
        return Err(format!(
            "missing npa-mathlib manifest: {}",
            mathlib_manifest_path.display()
        ));
    }
    let corpus = parse_manifest(&corpus_manifest_path)
        .map_err(|err| format!("failed to read corpus manifest: {err}"))?;
    let mathlib = parse_manifest(&mathlib_manifest_path)
        .map_err(|err| format!("failed to read npa-mathlib manifest: {err}"))?;

    match options.command.as_str() {
        "scan" => {
            scan(&npa_root, &mathlib_root, &corpus, &mathlib, &options.prefix)
                .map_err(|err| format!("scan failed: {err}"))?;
            Ok(ExitCode::SUCCESS)
        }
        "pin" => pin(
            &npa_root,
            &mathlib_root,
            &corpus_manifest_path,
            &corpus,
            &mathlib,
            &options.positional,
            options.apply,
        )
        .map_err(|err| format!("pin failed: {err}")),
        other => Err(format!("unknown command: {other}")),
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
    }
}
