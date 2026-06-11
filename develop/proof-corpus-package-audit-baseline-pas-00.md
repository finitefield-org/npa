# Proof Corpus Package Audit Baseline PAS-00

Source: `develop/proof-corpus-package-audit-speed-plan.md` PAS-00

This document records the package audit baseline before adding package audit
cache, selection, export-summary, or parallel verification code. The timing
record is not proof evidence. Proof acceptance still depends on canonical
certificate bytes, deterministic hashes, and source-free checker / verifier
verdicts.

## Environment

- Measurement start: `2026-06-08 03:03:14 JST (+0900)`
- Measurement end: `2026-06-08 08:10:19 JST (+0900)`
- Repository path: `/Users/kazuyoshitoshiya/ff/npa-package-audit-speed-plan`
- Commit: `23d322e2c47bbf7f6bfc5f47c9945ce568cc6f93`
- Branch: `codex/package-audit-speed-plan`
- Worktree status before measurement: clean
- Worktree status after measurement, before PAS-00 document edits: clean
- Worktree status after PAS-00 document edits: this baseline document plus the
  PAS-00 status update in `develop/proof-corpus-package-audit-speed-plan-todo.md`
- OS: `macOS 26.5.1 25F80`
- Kernel: `Darwin 25.5.0 RELEASE_ARM64_T6000 arm64`
- Machine model: `MacBookPro18,4`
- CPU: `Apple M1 Max`
- CPU thread count: `10`
- Memory: `68719476736` bytes
- Rust: `rustc 1.95.0 (59807616e 2026-04-14)`
- Cargo: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- Cargo target cache before full gate: absent in this worktree
- Cargo target cache for individual subcommand timings: warm, after the full
  package gate populated `target/`

Measurement caveat:

- A separate unrelated checkout under `/Users/kazuyoshitoshiya/ff/npa` was
  observed running `cargo test -p npa-cli package_cli_examples_pass_on_proof_corpus`
  with `npa package build-certs --root proofs --check` during this baseline.
  That process was not touched. Treat these timings as a real local baseline
  with concurrent CPU load, not as an isolated benchmark.

## Full Package Gate Timing

Command:

```sh
/usr/bin/time -p ./scripts/check-corpus-package.sh
```

Result:

| Status | real | user | sys |
| --- | ---: | ---: | ---: |
| pass | 9158.73s | 10459.77s | 57.29s |

The full gate was run with a cold Cargo target cache in this worktree. It
compiled the workspace test artifacts and then ran all eight package gate
steps.

## Per-Step Timings

These commands were run after the full package gate, so Cargo test artifacts
were warm. Cargo output was captured under `/tmp/npa-pas00-step*.log`.

| Step | Command | Status | real | user | sys |
| --- | --- | --- | ---: | ---: | ---: |
| 1 | `cargo test -p npa-proof-corpus --test manifest_package_audit` | pass | 486.46s | 754.96s | 3.47s |
| 2 | `cargo test --workspace --exclude npa-proof-corpus proof_package` | pass | 484.54s | 750.71s | 3.68s |
| 3 | `cargo test -p npa-api package_axiom_report_projection_proof_corpus_fixture_passes_eq_rec_policy` | pass | 469.47s | 466.43s | 2.15s |
| 4 | `cargo test -p npa-api --lib package_verifier` | pass | 514.86s | 1291.42s | 4.79s |
| 5 | `cargo test -p npa-cli package_cli_examples_pass_on_proof_corpus` | pass | 4998.20s | 4966.39s | 21.64s |
| 6 | `cargo test -p npa-cli package_axiom_report_proof_corpus_check_mode_succeeds_without_mutating_generated_artifacts` | pass | 470.60s | 467.66s | 2.13s |
| 7 | `cargo test -p npa-cli package_index_theorem_index_proof_corpus_check_keeps_generated_artifacts_clean` | pass | 475.09s | 472.20s | 2.14s |
| 8 | `cargo test -p npa-cli package_publish_plan_proof_corpus_check_mode_succeeds_with_checked_in_artifact` | pass | 1200.55s | 1192.53s | 5.25s |

The sum of the individual warm-cache real timings is `9099.77s`, close to the
cold full-gate real timing once script overhead and cold compilation are taken
into account.

## Package Graph Inventory

Inventory source:

```text
proofs/generated/package-lock.json
```

Temporary local command:

```sh
ruby -rjson -rset -e 'lock=JSON.parse(File.read(ARGV.fetch(0))); entries=lock.fetch("entries"); modules=entries.map{|e| e.fetch("module")}; module_set=modules.to_set; local_set=entries.select{|e| e.fetch("origin")=="local"}.map{|e| e.fetch("module")}.to_set; imports_by=entries.to_h{|e| [e.fetch("module"), (e["imports"]||[]).map{|i| i.fetch("module")}.select{|m| module_set.include?(m)}]}; direct_import_edges=entries.sum{|e| (e["imports"]||[]).length}; local_reverse_edges=entries.select{|e| e.fetch("origin")=="local"}.sum{|e| (e["imports"]||[]).count{|i| local_set.include?(i.fetch("module"))}}; remaining=modules.to_set; done=Set.new; layers=[]; until remaining.empty?; layer=modules.select{|m| remaining.include?(m) && imports_by.fetch(m).all?{|dep| done.include?(dep)}}; raise "cycle_or_missing_progress" if layer.empty?; layers << layer; layer.each{|m| remaining.delete(m); done.add(m)}; end; puts "local_module_count=#{local_set.length}"; puts "external_import_count=#{entries.count{|e| e.fetch("origin")=="external"}}"; puts "lock_entry_count=#{entries.length}"; puts "direct_import_edge_count=#{direct_import_edges}"; puts "local_reverse_edge_count=#{local_reverse_edges}"; puts "topological_layer_count=#{layers.length}"' proofs/generated/package-lock.json
```

Counts:

| Metric | Count |
| --- | ---: |
| Package module count | 228 |
| External import count | 2 |
| Package-lock entry count | 230 |
| Direct import edge count | 1206 |
| Local reverse edge count | 1001 |
| Topological layer count | 26 |

Definitions used for this baseline:

- Package module count counts `origin = "local"` package-lock entries.
- External import count counts `origin = "external"` package-lock entries.
- Direct import edge count sums every `imports[]` entry in the checked package
  lock.
- Local reverse edge count counts direct import edges from local entries to
  local entries, which are the edges that create local reverse-dependency work.
- Topological layer count is computed over all checked package-lock entries
  using only imports that are also present in the checked lock.

## Top Bottlenecks

Top three individual subcommands by real time:

1. `cargo test -p npa-cli package_cli_examples_pass_on_proof_corpus`:
   `4998.20s`
2. `cargo test -p npa-cli package_publish_plan_proof_corpus_check_mode_succeeds_with_checked_in_artifact`:
   `1200.55s`
3. `cargo test -p npa-api --lib package_verifier`: `514.86s`

The next cluster is close behind:

- `cargo test -p npa-proof-corpus --test manifest_package_audit`: `486.46s`
- `cargo test --workspace --exclude npa-proof-corpus proof_package`: `484.54s`
- `cargo test -p npa-cli package_index_theorem_index_proof_corpus_check_keeps_generated_artifacts_clean`:
  `475.09s`
- `cargo test -p npa-cli package_axiom_report_proof_corpus_check_mode_succeeds_without_mutating_generated_artifacts`:
  `470.60s`
- `cargo test -p npa-api package_axiom_report_projection_proof_corpus_fixture_passes_eq_rec_policy`:
  `469.47s`

Operational observation:

- The slowest CLI example step spends most of its time in source-free package
  commands, including `npa package build-certs --root proofs --check` and
  `npa package verify-certs --root proofs --checker reference`.
- The package gate is dominated by repeated source-free work over the same
  checked certificate graph, which is the optimization target for PAS-01 and
  later milestones.

## Verification Notes

PAS-00 intentionally did not add Rust helpers. The graph inventory was collected
with a temporary one-liner that reads only the checked package lock.

Before this document was written, the following scope check produced no paths:

```sh
git diff --name-only -- proofs tools/proof-corpus scripts crates
```

This milestone leaves proof source, certificates, package manifests, generated
package artifacts, gate scripts, and Rust code unchanged.
