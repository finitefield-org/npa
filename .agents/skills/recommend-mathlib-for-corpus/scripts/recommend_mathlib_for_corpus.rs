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
    imports: Vec<Entry>,
    modules: Vec<Entry>,
}

#[derive(Debug)]
struct Options {
    mathlib_root: Option<String>,
    prefix: String,
    limit: usize,
    include_pinned: bool,
}

#[derive(Debug)]
struct Site {
    file: String,
    line: usize,
    corpus_module: String,
    old_import: String,
}

#[derive(Debug)]
struct Recommendation {
    mathlib_module: String,
    counterpart: String,
    score: i64,
    pinned: bool,
    sites: Vec<Site>,
    direct_corpus_imports: usize,
    mathlib_imports: Vec<String>,
    theorem_count: usize,
    definition_count: usize,
    certificate: String,
    export_hash: String,
    certificate_hash: String,
}

fn usage() {
    println!(
        "Usage: recommend_mathlib_for_corpus.sh [options]\n\nOptions:\n  --mathlib-root PATH   npa-mathlib checkout path (default: ../npa-mathlib)\n  --prefix MODULE       corpus source prefix to inspect (default: Proofs.Ai.)\n  --limit N             maximum recommendations to print (default: 12)\n  --include-pinned      include modules already imported by proofs/npa-package.toml\n  -h, --help            show this help"
    );
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut options = Options {
        mathlib_root: None,
        prefix: "Proofs.Ai.".to_string(),
        limit: 12,
        include_pinned: false,
    };

    let mut index = 1;
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
            "--limit" => {
                index += 1;
                let raw = args
                    .get(index)
                    .ok_or_else(|| "--limit requires a number".to_string())?;
                options.limit = raw
                    .parse::<usize>()
                    .map_err(|_| format!("invalid --limit value: {raw}"))?;
            }
            "--include-pinned" => options.include_pinned = true,
            "-h" | "--help" => {
                usage();
                return Err("__HELP__".to_string());
            }
            other => return Err(format!("unknown option: {other}")),
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
    Some((key, value.trim_matches('"').to_string()))
}

fn parse_manifest(path: &Path) -> io::Result<Manifest> {
    let text = fs::read_to_string(path)?;
    let mut manifest = Manifest::default();
    let mut section = "";
    let mut current = Entry::default();
    let mut has_current = false;

    fn flush(section: &str, current: &mut Entry, has_current: &mut bool, manifest: &mut Manifest) {
        if !*has_current || current.module.is_empty() {
            *current = Entry::default();
            *has_current = false;
            return;
        }
        if section == "imports" {
            manifest.imports.push(current.clone());
        } else if section == "modules" {
            manifest.modules.push(current.clone());
        }
        *current = Entry::default();
        *has_current = false;
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

fn collect_sites(
    npa_root: &Path,
    prefix: &str,
    counterparts: &BTreeMap<String, String>,
) -> io::Result<BTreeMap<String, Vec<Site>>> {
    let mut sources = Vec::new();
    walk_sources(&npa_root.join("proofs/Proofs/Ai"), &mut sources)?;
    let mut out: BTreeMap<String, Vec<Site>> = BTreeMap::new();

    for source in sources {
        let corpus_module = module_from_source(npa_root, &source);
        if !corpus_module.starts_with(prefix) {
            continue;
        }
        let rel = source
            .strip_prefix(npa_root)
            .unwrap_or(&source)
            .to_string_lossy()
            .to_string();
        for (line_index, raw_line) in fs::read_to_string(&source)?.lines().enumerate() {
            let line = raw_line.trim();
            let Some(imported) = line.strip_prefix("import ") else {
                continue;
            };
            let imported = imported.trim();
            let Some(mathlib_module) = counterparts.get(imported) else {
                continue;
            };
            out.entry(mathlib_module.clone()).or_default().push(Site {
                file: rel.clone(),
                line: line_index + 1,
                corpus_module: corpus_module.clone(),
                old_import: imported.to_string(),
            });
        }
    }
    Ok(out)
}

fn direct_corpus_import_count(corpus: &Manifest, counterpart: &str) -> usize {
    corpus
        .modules
        .iter()
        .filter(|entry| entry.imports.iter().any(|import| import == counterpart))
        .count()
}

fn recommend(
    npa_root: &Path,
    mathlib_root: &Path,
    corpus: &Manifest,
    mathlib: &Manifest,
    options: &Options,
) -> io::Result<Vec<Recommendation>> {
    let counterparts = build_counterpart_map(corpus, mathlib);
    let reverse_counterparts: BTreeMap<String, String> = counterparts
        .iter()
        .map(|(corpus_module, mathlib_module)| (mathlib_module.clone(), corpus_module.clone()))
        .collect();
    let sites_by_mathlib = collect_sites(npa_root, &options.prefix, &counterparts)?;
    let pinned: BTreeSet<String> = corpus
        .imports
        .iter()
        .map(|entry| entry.module.clone())
        .collect();
    let mathlib_by_module: BTreeMap<String, Entry> = mathlib
        .modules
        .iter()
        .map(|entry| (entry.module.clone(), entry.clone()))
        .collect();
    let mut recommendations = Vec::new();

    for (mathlib_module, sites) in sites_by_mathlib {
        let is_pinned = pinned.contains(&mathlib_module);
        if is_pinned && !options.include_pinned {
            continue;
        }
        let Some(entry) = mathlib_by_module.get(&mathlib_module) else {
            continue;
        };
        let counterpart = reverse_counterparts
            .get(&mathlib_module)
            .cloned()
            .unwrap_or_else(|| "(unknown)".to_string());
        let unique_importers: BTreeSet<String> = sites
            .iter()
            .map(|site| site.corpus_module.clone())
            .collect();
        let direct_imports = direct_corpus_import_count(corpus, &counterpart);
        let mut score = (sites.len() as i64 * 5)
            + (unique_importers.len() as i64 * 7)
            + (direct_imports as i64 * 3)
            + (entry.theorems.len() as i64 / 4)
            + (entry.definitions.len() as i64 / 2);
        if !is_pinned {
            score += 12;
        }
        if mathlib_module.starts_with("Mathlib.Logic.")
            || mathlib_module.starts_with("Mathlib.Algebra.")
            || mathlib_module.starts_with("Mathlib.LinearAlgebra.")
        {
            score += 4;
        }
        recommendations.push(Recommendation {
            mathlib_module,
            counterpart,
            score,
            pinned: is_pinned,
            sites,
            direct_corpus_imports: direct_imports,
            mathlib_imports: entry.imports.clone(),
            theorem_count: entry.theorems.len(),
            definition_count: entry.definitions.len(),
            certificate: entry.certificate.clone(),
            export_hash: entry.expected_export_hash.clone(),
            certificate_hash: entry.expected_certificate_hash.clone(),
        });
    }

    recommendations.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.mathlib_module.cmp(&right.mathlib_module))
    });

    println!("# npa-mathlib modules recommended for proof corpus use");
    println!("npa_root: {}", npa_root.display());
    println!("mathlib_root: {}", mathlib_root.display());
    println!("prefix: {}", options.prefix);
    println!("include_pinned: {}", options.include_pinned);
    println!("detected_counterparts: {}", counterparts.len());
    println!(
        "candidate_modules: {}",
        recommendations.len().min(options.limit)
    );
    println!("note: scores rank migration payoff; they are not proof evidence.");

    Ok(recommendations)
}

fn print_recommendations(recommendations: &[Recommendation], limit: usize) {
    for (index, item) in recommendations.iter().take(limit).enumerate() {
        let status = if item.pinned { "pinned" } else { "not-pinned" };
        let unique_importers: BTreeSet<String> = item
            .sites
            .iter()
            .map(|site| site.corpus_module.clone())
            .collect();
        println!(
            "\n{}. {} score={}",
            index + 1,
            item.mathlib_module,
            item.score
        );
        println!("  status: {status}");
        println!("  counterpart: {}", item.counterpart);
        println!("  replacement_sites: {}", item.sites.len());
        println!("  unique_corpus_importers: {}", unique_importers.len());
        println!(
            "  direct_corpus_imports_of_counterpart: {}",
            item.direct_corpus_imports
        );
        println!(
            "  public_surface: definitions={} theorems={}",
            item.definition_count, item.theorem_count
        );
        println!(
            "  public_imports: {}",
            if item.mathlib_imports.is_empty() {
                "none".to_string()
            } else {
                item.mathlib_imports.join(",")
            }
        );
        println!("  certificate: {}", item.certificate);
        println!("  export_hash: {}", item.export_hash);
        println!("  certificate_hash: {}", item.certificate_hash);
        println!("  suggested_pin: .agents/skills/use-mathlib-in-corpus/scripts/use_mathlib_in_corpus.sh pin {} --mathlib-root ../npa-mathlib --apply", item.mathlib_module);
        println!("  example_sites:");
        for site in item.sites.iter().take(6) {
            println!(
                "    {}:{} {} -> {}",
                site.file, site.line, site.old_import, item.mathlib_module
            );
        }
        if item.sites.len() > 6 {
            println!("    ... {} more", item.sites.len() - 6);
        }
    }

    if recommendations.is_empty() {
        println!("\nNo recommendation candidates found.");
        println!("Possible causes: no local npa-mathlib, no detected promoted counterparts, all candidates already pinned, or the prefix has no matching source imports.");
    }
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
    let options = match parse_args(&shifted_args) {
        Ok(options) => options,
        Err(message) if message == "__HELP__" => return Ok(ExitCode::SUCCESS),
        Err(message) => return Err(message),
    };

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
    let recommendations = recommend(&npa_root, &mathlib_root, &corpus, &mathlib, &options)
        .map_err(|err| format!("recommendation failed: {err}"))?;
    print_recommendations(&recommendations, options.limit);

    Ok(ExitCode::SUCCESS)
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
