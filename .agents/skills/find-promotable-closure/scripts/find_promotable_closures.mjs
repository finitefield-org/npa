#!/usr/bin/env node
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));

const options = {
  mathlibRoot: null,
  prefix: "Proofs.Ai.",
  limit: 12,
  maxClosure: 8,
  includeDefer: false,
  includeSourceFallback: false,
  withAxiomReport: false,
};

function usage() {
  console.log(`Usage: find_promotable_closures.sh [options]

Options:
  --mathlib-root PATH     npa-mathlib checkout path (default: ../npa-mathlib)
  --prefix MODULE         corpus module prefix (default: Proofs.Ai.)
  --limit N               maximum rows to print (default: 12)
  --max-closure N         maximum unpromoted closure size (default: 8)
  --include-defer         include near misses that fail the ready heuristic
  --include-source-fallback
                           include proofs/Proofs/Ai/**/source.npa modules absent from manifest
  --with-axiom-report     read proofs/generated/axiom-report.json
  -h, --help              show this help

This is a read-only heuristic scanner for the local npa proof corpus.`);
}

for (let i = 2; i < process.argv.length; i += 1) {
  const arg = process.argv[i];
  if (arg === "--mathlib-root") options.mathlibRoot = process.argv[++i];
  else if (arg === "--prefix") options.prefix = process.argv[++i];
  else if (arg === "--limit") options.limit = Number(process.argv[++i]);
  else if (arg === "--max-closure") options.maxClosure = Number(process.argv[++i]);
  else if (arg === "--include-defer") options.includeDefer = true;
  else if (arg === "--include-source-fallback") options.includeSourceFallback = true;
  else if (arg === "--with-axiom-report") options.withAxiomReport = true;
  else if (arg === "-h" || arg === "--help") {
    usage();
    process.exit(0);
  } else {
    console.error(`unknown option: ${arg}`);
    usage();
    process.exit(2);
  }
}

function repoRoot() {
  try {
    return execFileSync("git", ["-C", scriptDir, "rev-parse", "--show-toplevel"], {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    }).trim();
  } catch {
    return path.resolve(scriptDir, "../../../..");
  }
}

const npaRoot = repoRoot();
const mathlibRoot = path.resolve(npaRoot, options.mathlibRoot ?? "../npa-mathlib");
const corpusManifest = path.join(npaRoot, "proofs", "npa-package.toml");
const mathlibManifest = path.join(mathlibRoot, "npa-package.toml");

if (!fs.existsSync(corpusManifest)) {
  console.error(`missing corpus manifest: ${corpusManifest}`);
  process.exit(1);
}
if (!fs.existsSync(mathlibManifest)) {
  console.error(`missing npa-mathlib manifest: ${mathlibManifest}`);
  console.error("pass --mathlib-root PATH if the checkout is not a sibling of npa");
  process.exit(1);
}

function parseList(line) {
  return [...line.matchAll(/"([^"]+)"/g)].map((match) => match[1]);
}

function parseManifest(filePath, metadata) {
  const modules = [];
  let current = null;

  function flush() {
    if (!current?.module) return;
    current.metadata = metadata;
    current.imports ??= [];
    current.definitions ??= [];
    current.theorems ??= [];
    current.axioms ??= [];
    modules.push(current);
  }

  for (const rawLine of fs.readFileSync(filePath, "utf8").split(/\r?\n/)) {
    const line = rawLine.trim();
    if (line === "[[modules]]") {
      flush();
      current = {};
      continue;
    }
    if (!current) continue;
    const stringMatch = line.match(/^([A-Za-z0-9_]+)\s*=\s*"([^"]*)"/);
    if (stringMatch) {
      const [, key, value] = stringMatch;
      current[key] = value;
      continue;
    }
    const listMatch = line.match(/^(imports|definitions|theorems|axioms)\s*=/);
    if (listMatch) current[listMatch[1]] = parseList(line);
  }
  flush();
  return modules;
}

function walkSourceFiles(root) {
  const out = [];
  if (!fs.existsSync(root)) return out;
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    const full = path.join(root, entry.name);
    if (entry.isDirectory()) out.push(...walkSourceFiles(full));
    else if (entry.isFile() && entry.name === "source.npa") out.push(full);
  }
  return out.sort();
}

function cleanDeclName(name) {
  return name.replace(/[.:].*$/, "");
}

function parseSourceFallbacks() {
  const sourceRoot = path.join(npaRoot, "proofs", "Proofs", "Ai");
  const files = walkSourceFiles(sourceRoot);
  return files.map((sourcePath) => {
    const rel = path.relative(path.join(npaRoot, "proofs"), sourcePath);
    const module = rel.replace(/\/source\.npa$/, "").split(path.sep).join(".");
    const dir = path.dirname(rel);
    const mod = {
      module,
      source: rel,
      certificate: path.join(dir, "certificate.npcert"),
      meta: path.join(dir, "meta.json"),
      replay: path.join(dir, "replay.json"),
      imports: [],
      definitions: [],
      theorems: [],
      axioms: [],
      metadata: "source-fallback",
    };
    for (const rawLine of fs.readFileSync(sourcePath, "utf8").split(/\r?\n/)) {
      const line = rawLine.trim();
      const importMatch = line.match(/^import\s+(\S+)/);
      if (importMatch) mod.imports.push(importMatch[1]);
      const declMatch = line.match(/^(def|theorem|axiom)\s+(\S+)/);
      if (declMatch) {
        const name = cleanDeclName(declMatch[2]);
        if (declMatch[1] === "def") mod.definitions.push(name);
        if (declMatch[1] === "theorem") mod.theorems.push(name);
        if (declMatch[1] === "axiom") mod.axioms.push(name);
      }
    }
    return mod;
  });
}

function signature(mod) {
  return `${(mod.definitions ?? []).join(",")}|${(mod.theorems ?? []).join(",")}`;
}

function unique(items) {
  return [...new Set(items.filter(Boolean))];
}

const corpus = parseManifest(corpusManifest, "manifest");
const mathlib = parseManifest(mathlibManifest, "manifest");
const byModule = new Map(corpus.map((mod) => [mod.module, mod]));

if (options.includeSourceFallback) {
  for (const mod of parseSourceFallbacks()) {
    if (!byModule.has(mod.module)) {
      corpus.push(mod);
      byModule.set(mod.module, mod);
    }
  }
}

const mathlibByExport = new Map();
const mathlibBySignature = new Map();
for (const mod of mathlib) {
  if (mod.expected_export_hash) mathlibByExport.set(mod.expected_export_hash, mod.module);
  const sig = signature(mod);
  if (sig !== "|") mathlibBySignature.set(sig, mod.module);
}

const axiomReport = new Map();
if (options.withAxiomReport) {
  const reportPath = path.join(npaRoot, "proofs", "generated", "axiom-report.json");
  if (fs.existsSync(reportPath)) {
    const report = JSON.parse(fs.readFileSync(reportPath, "utf8"));
    for (const mod of report.modules ?? []) {
      axiomReport.set(mod.module, {
        direct: unique((mod.direct_axioms ?? []).map((axiom) => axiom.name)),
        transitive: unique((mod.transitive_axioms ?? []).map((axiom) => axiom.name)),
        status: mod.policy_status?.status ?? "unknown",
      });
    }
  }
}

const promotedCache = new Map();
function promoted(mod) {
  if (promotedCache.has(mod.module)) return promotedCache.get(mod.module);
  let reason = null;
  if (mod.expected_export_hash && mathlibByExport.has(mod.expected_export_hash)) {
    reason = `export-hash:${mathlibByExport.get(mod.expected_export_hash)}`;
  } else {
    const sig = signature(mod);
    if (sig !== "|" && mathlibBySignature.has(sig)) {
      reason = `declaration-signature:${mathlibBySignature.get(sig)}`;
    }
  }
  promotedCache.set(mod.module, reason);
  return reason;
}

function artifactIssues(mod) {
  const issues = [];
  for (const [key, rel] of [
    ["source", mod.source],
    ["certificate", mod.certificate],
    ["meta", mod.meta],
    ["replay", mod.replay],
  ]) {
    if (!rel || !fs.existsSync(path.join(npaRoot, "proofs", rel))) {
      issues.push(`${mod.module}:${key}`);
    }
  }
  return issues;
}

function closureFor(root) {
  const seen = new Set();
  const closure = [];
  const promotedDeps = new Set();
  const publicDeps = new Set();
  const missingDeps = new Set();
  let truncated = false;

  function dfs(moduleName) {
    if (seen.has(moduleName)) return;
    const mod = byModule.get(moduleName);
    if (!mod) {
      missingDeps.add(moduleName);
      return;
    }
    seen.add(moduleName);
    closure.push(moduleName);
    if (closure.length > options.maxClosure) {
      truncated = true;
      return;
    }
    for (const imp of mod.imports ?? []) {
      if (imp.startsWith("Proofs.Ai.")) {
        const dep = byModule.get(imp);
        if (!dep) missingDeps.add(imp);
        else if (promoted(dep)) promotedDeps.add(imp);
        else dfs(imp);
      } else {
        publicDeps.add(imp);
      }
    }
  }

  dfs(root.module);
  return { closure, promotedDeps, publicDeps, missingDeps, truncated };
}

function directImportCount(moduleName) {
  let count = 0;
  for (const mod of corpus) {
    if (mod.module !== moduleName && (mod.imports ?? []).includes(moduleName)) count += 1;
  }
  return count;
}

function closureAxioms(closure) {
  const axioms = new Set();
  let policy = "ok";
  for (const moduleName of closure) {
    const mod = byModule.get(moduleName);
    const report = axiomReport.get(moduleName);
    const names = report?.transitive?.length ? report.transitive : (mod?.axioms ?? []);
    for (const name of names) axioms.add(name);
    if (report?.status && report.status !== "ok") policy = report.status;
  }
  return { names: [...axioms], policy };
}

function publicGuess(moduleName) {
  return moduleName.replace(/^Proofs\.Ai\./, "Mathlib.");
}

const candidates = [];
for (const root of corpus) {
  if (!root.module.startsWith(options.prefix)) continue;
  if (promoted(root)) continue;

  const closure = closureFor(root);
  const artifacts = unique(closure.closure.flatMap((moduleName) => artifactIssues(byModule.get(moduleName))));
  const ax = closureAxioms(closure.closure);
  const policyOk = ax.names.every((name) => name === "Eq.rec") && ax.policy === "ok";
  const artifactOk = artifacts.length === 0;
  const direct = directImportCount(root.module);

  let status = "candidate";
  let reason = root.metadata === "source-fallback" ? "ready-heuristic-source-fallback" : "ready-heuristic";
  if (closure.closure.length > options.maxClosure || closure.truncated) {
    status = "defer";
    reason = "closure-too-large";
  } else if (closure.missingDeps.size > 0) {
    status = "defer";
    reason = "missing-corpus-dependency";
  } else if (!artifactOk) {
    status = "defer";
    reason = "missing-sidecar";
  } else if (!policyOk) {
    status = "defer";
    reason = "axiom-policy";
  }

  if (status !== "candidate" && !options.includeDefer) continue;

  let score = direct * 4 + (options.maxClosure - closure.closure.length);
  if (direct >= 2) score += 4;
  if (artifactOk) score += 2;
  else score -= 10;
  if (policyOk) score += 2;
  else score -= 10;
  if (closure.missingDeps.size === 0) score += 1;
  else score -= 5;
  if (root.metadata === "source-fallback") score -= 2;

  candidates.push({
    root,
    status,
    reason,
    score,
    direct,
    closure,
    artifacts,
    axioms: ax.names,
    publicModule: publicGuess(root.module),
  });
}

candidates.sort((a, b) => b.score - a.score || a.root.module.localeCompare(b.root.module));
const top = candidates.slice(0, Math.max(0, options.limit));

console.log("# NPA promotable proof-corpus closure candidates");
console.log(`npa_root: ${npaRoot}`);
console.log(`mathlib_root: ${mathlibRoot}`);
console.log(`prefix: ${options.prefix}`);
console.log(`max_closure: ${options.maxClosure}`);
console.log(`with_axiom_report: ${options.withAxiomReport ? "true" : "false"}`);
console.log(`include_source_fallback: ${options.includeSourceFallback ? "true" : "false"}`);
console.log(`corpus_modules: ${corpus.length}`);
console.log(`mathlib_modules: ${mathlib.length}`);
console.log("note: candidate is a heuristic; use judge-promote-to-mathlib before recommending promotion.");

for (const item of top) {
  const axiomText = item.axioms.length ? item.axioms.join(",") : "none";
  const deps = [...item.closure.promotedDeps, ...item.closure.publicDeps];
  console.log("");
  console.log(`${item.status} score=${item.score} root=${item.root.module}`);
  console.log(`  suggested_public_module: ${item.publicModule}`);
  console.log(`  metadata: ${item.root.metadata}`);
  console.log(`  reason: ${item.reason}`);
  console.log(`  closure_size: ${item.closure.closure.length}`);
  console.log(`  closure: ${item.closure.closure.join(",")}`);
  console.log(`  direct_downstream_importers: ${item.direct}`);
  console.log(`  axioms: ${axiomText}`);
  console.log(`  promoted_or_public_deps: ${deps.length ? deps.join(",") : "none"}`);
  if (item.closure.missingDeps.size) console.log(`  missing_deps: ${[...item.closure.missingDeps].join(",")}`);
  if (item.artifacts.length) console.log(`  artifact_issues: ${item.artifacts.join(",")}`);
  if (item.closure.truncated) console.log("  closure_note: truncated-at-max-closure");
  console.log(`  next_verify: cargo run -q -p npa-proof-corpus -- --module ${item.root.module} --verified-cache authoring`);
}

if (top.length === 0) {
  console.log("");
  console.log("No ready candidates matched the current heuristic.");
  console.log("Rerun with --include-defer to inspect near misses.");
}
