# Proof Corpus Package Audit PAS-08 Measurement

Source: `develop/proof-corpus-package-audit-speed-plan.md` PAS-08

This document records the final PAS-08 package audit speed measurements. Timing
logs, local audit cache entries, and this document are not proof evidence.
Proof acceptance still depends on canonical certificate bytes, deterministic
hashes, and source-free checker verdicts. Local audit cache hits are local
iteration acceleration only.

## Baseline Reference

- Baseline document: `develop/proof-corpus-package-audit-baseline-pas-00.md`
- Baseline commit: `23d322e2c47bbf7f6bfc5f47c9945ce568cc6f93`
- Baseline measurement window: `2026-06-08 03:03:14 JST (+0900)` to
  `2026-06-08 08:10:19 JST (+0900)`
- Baseline machine context: macOS 26.5.1 25F80, Darwin 25.5.0,
  MacBookPro18,4, Apple M1 Max, 10 CPU threads, 68,719,476,736 bytes memory,
  Rust `rustc 1.95.0 (59807616e 2026-04-14)`, Cargo
  `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- Baseline full package gate: `./scripts/check-corpus-package.sh` passed in
  `real 9158.73s`, `user 10459.77s`, `sys 57.29s`

## Environment

- Measurement date: `2026-06-08`
- Measurement repository: `/Users/kazuyoshitoshiya/ff/npa-package-audit-speed-plan`
- Measurement commit before PAS-08 document edits:
  `0967e62d68e891724de0d32765ad347e0446a3fd`
- Branch: `codex/package-audit-speed-plan`
- OS: macOS 26.5.1 25F80
- Kernel: Darwin 25.5.0 `RELEASE_ARM64_T6000`
- Machine model: MacBookPro18,4
- CPU thread count: 10
- Memory: 68,719,476,736 bytes
- Rust: `rustc 1.95.0 (59807616e 2026-04-14)`
- Cargo: `cargo 1.95.0 (f2d3ce0bd 2026-03-21)`
- Worktree status before measurement: clean

## Full Package Gate

Command:

```sh
/usr/bin/time -p ./scripts/check-corpus-package.sh
```

Result:

| Status | real | user | sys | Notes |
| --- | ---: | ---: | ---: | --- |
| not completed | n/a | n/a | n/a | A PAS-08 run was started but interrupted before a final `/usr/bin/time -p` line was produced. No complete full-gate timing is claimed for PAS-08. |

The final package gate remains required for promotion, release, package
tooling, certificate, checker, and high-trust boundaries. PAS-08 does not relax
`check-corpus-package.sh` or `check-corpus-full.sh`.

## Proof Corpus Package Verification

Before the read-through run, `target/npa-package-audit-cache` was removed to
measure a cold local audit cache. The package contained 230 modules.

| Command | Status | real | user | sys | Cache mode | Hits | Misses | Stale | Written | Live checker | Cached | Proof evidence |
| --- | --- | ---: | ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `cargo run -p npa-cli -- package verify-certs --root proofs --checker reference --audit-cache off --json` | pass | 253.88s | 249.11s | 1.85s | off | n/a | n/a | n/a | n/a | 230 | 0 | yes |
| `cargo run -p npa-cli -- package verify-certs --root proofs --checker reference --audit-cache read-through --json` | pass | 258.59s | 252.97s | 2.09s | read-through | 0 | 230 | 0 | 230 | 230 | 0 | yes, live checker |
| `cargo run -p npa-cli -- package verify-certs --root proofs --checker reference --audit-cache local-hit --json` | pass | 1.38s | 0.81s | 0.08s | local-hit | 230 | 0 | 0 | 0 | 0 | 230 | no |

Observations:

- The cache-off reference run is the final source-free verifier evidence for
  this measurement.
- The read-through run intentionally did not accelerate the verdict; it still
  live-checked 230 modules and wrote 230 local cache entries.
- The local-hit run was about 184x faster than the cache-off reference run, but
  every module reported `evidence=local-audit-cache;proof_evidence=false`.
- Deleting the local cache affects only acceleration counters; it does not
  change the cache-off verifier verdict.

## Fast Verifier Jobs

| Command | Status | real | user | sys | Normalized comparison |
| --- | --- | ---: | ---: | ---: | --- |
| `cargo run -p npa-cli -- package verify-certs --root proofs --checker fast --jobs 1 --json` | pass | 472.78s | 465.70s | 3.07s | baseline output for fast checker |
| `cargo run -p npa-cli -- package verify-certs --root proofs --checker fast --jobs 4 --json` | fail | 37.32s | 41.29s | 0.22s | not comparable; process aborted with stack overflow before JSON output |

`--jobs 4` failed with:

```text
thread '<unknown>' (1143800) has overflowed its stack
fatal runtime error: stack overflow, aborting
```

The targeted unit test
`cargo test -p npa-api --lib package_verifier_parallel_fast_jobs_four_matches_jobs_one_normalized`
passed, but the full proof corpus package run still aborted. This is a
remaining implementation issue for the fast parallel path. PAS-08 does not
treat the failed `--jobs 4` run as evidence of speedup or correctness.

## Representative Closure Audit Loop

The sibling `../npa-mathlib` checkout existed and was clean:

```text
## main...origin/main
```

The representative loop used the existing public layer containing
`Mathlib.Logic.Basic`, which corresponds to the `Proofs.Ai.Basic` seed route.
Before the read-through run, `target/npa-package-audit-cache` was removed to
measure a cold local audit cache for this package. The package contained 66
modules.

| Command | Status | real | user | sys | Cache mode | Hits | Misses | Stale | Written | Live checker | Cached | Proof evidence |
| --- | --- | ---: | ---: | ---: | --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| `cargo run -p npa-cli -- package verify-certs --root ../npa-mathlib --checker reference --audit-cache off --json` | pass | 51.54s | 50.64s | 0.42s | off | n/a | n/a | n/a | n/a | 66 | 0 | yes |
| `cargo run -p npa-cli -- package verify-certs --root ../npa-mathlib --checker reference --audit-cache read-through --json` | pass | 51.71s | 50.85s | 0.42s | read-through | 0 | 66 | 0 | 66 | 66 | 0 | yes, live checker |
| `cargo run -p npa-cli -- package verify-certs --root ../npa-mathlib --checker reference --audit-cache local-hit --json` | pass | 2.28s | 0.28s | 0.03s | local-hit | 66 | 0 | 0 | 0 | 0 | 66 | no |

Closure audit recording rules from PAS-07 remain in force:

- `local_audit_cache_mode`: local-hit is allowed only for local iteration.
- `selected_modules`: selection output is non-evidence and must be recorded
  separately when used.
- `selection_reasons`: selection reasons do not replace verifier commands.
- `cache_hits`: local-hit produced 66 hits for this representative package.
- `live_checker_count`: final cache-off verification live-checked 66 modules.
- `skipped_by_stable_export`: not used in this representative full-package
  loop.
- `final_cache_off_verification`: the cache-off reference command above passed.
- `trust_boundary_note`: local cache entries and timing logs are not proof
  evidence.

## Gate Policy Result

No default operator guidance changed in PAS-08:

- `README.md`, `CONTRIBUTING.md`, `AGENTS.md`, and
  `develop/internal-readme-notes-ja.md` already preserve explicit package/full
  gates for promotion, release, checker, certificate, and high-trust
  boundaries.
- `local-hit` was not added to `check-corpus-package.sh`,
  `check-corpus-full.sh`, release scripts, or high-trust scripts.
- `local-hit` remains an optional local iteration tool only.

Verification cleanup:

- `check-fast.sh` initially failed on clippy-only issues:
  `cloned_ref_to_slice_refs` in `crates/npa-cert/src/tests.rs`,
  `manual_contains` in `crates/npa-package/src/audit_selection.rs`, and
  `large_enum_variant` in `crates/npa-api/src/package_verifier.rs` and
  `crates/npa-cli/src/package_verify.rs`.
- PAS-08 includes only those mechanical clippy fixes outside documentation.
  They do not change package audit policy, proof source, certificates, package
  manifests, generated artifacts, or gate scripts.

## Remaining Bottlenecks

1. The full package gate remains intentionally expensive. PAS-08 has no
   completed full-gate timing, and the PAS-00 full gate remains the last
   complete end-to-end package gate timing.
2. Fast verifier `--jobs 4` currently aborts with stack overflow, so parallel
   fast verification is not yet a usable speedup path for the proof corpus
   package.
3. Cache-off and read-through reference verification still require live checker
   work over every module. Local-hit removes that work for repeated local
   iteration, but it is explicitly not proof evidence.

## Conclusion

PAS-08 shows a large local iteration speedup when an exact local audit cache hit
is available: `253.88s` cache-off reference verification versus `1.38s`
local-hit on the proof corpus package, and `51.54s` versus `2.28s` on the
representative `npa-mathlib` package. The speedup does not weaken the final
package gate policy because final readiness still requires cache-off source-free
verification and the explicit package/full gates.
