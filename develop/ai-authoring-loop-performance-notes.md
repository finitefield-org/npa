# AI Authoring Loop Performance Notes

Date: 2026-06-12 (round 3; rounds 1-2 summarized for context)

This document records the optimization work on the AI proof-authoring loop
(`npa-proof-corpus --build-module` / `--module`), what each round changed,
which approaches were measured and rejected, and the candidate levers for the
next round. None of this work changes the certificate-first trust boundary:
every round is gated on byte-identical certificates
(`certificate_file_sha256` / `export_hash` unchanged after a module rebuild)
plus `./scripts/check-fast.sh` and `./scripts/check-corpus-package.sh`.

## Timeline of results

Benchmark module: `Proofs.Ai.Category.Classical` (largest corpus module),
dev profile with the workspace `opt-level = 3` overrides, Apple Silicon.

| Round | Branch / PR | `--build-module --checked-in-imports` | Main changes |
| --- | --- | --- | --- |
| baseline | pre-#35 | 36.6 s | — |
| 1 | claude0612 / PR #35 | 6.0 s | `quick_syntactic_eq` defeq fast path, memoized cert hashing, `npa-frontend` dev opt-level 3 |
| 2 | claude0612-2 / PR #36 | ~3.3-4.0 s user | elaborator Pi quadratic fix, verify height O(n²) fix, canonicalization memo sharing |
| 3 | claude0612-3 / 5c701a2 | 24.0G → 21.1G instructions (−12%); ~2.0 → ~1.8 s user (load-noisy) | hash-keyed canonical tables, level hash memo, byte-scan `json_escape` |

`--module` verify is unaffected by round 3 (those changes sit on the
certificate *production* path) and stays at ~0.2 s user.

## Round 3: what changed

1. **Hash-keyed canonical certificate tables** (`npa-cert/src/canonical.rs`,
   `hash.rs`). `CanonTerm::cmp` was the largest single self-time symbol
   (13.8%): the `BTreeSet<CanonTerm>` dedup set and `BTreeMap<CanonTerm,
   TermId>` id maps performed deep structural comparisons (plus a `clone()`
   per node occurrence) on every insert and lookup. They are now
   `CanonNodeCollector` / `CanonNodeIds`, keyed by the domain-separated
   sha256 node hashes, with one pointer-keyed `TermHashMemo` and one
   value-keyed `LevelHashMemo` threaded through collect → table build →
   payload materialization, so every node is hashed once per certificate
   build.

   Why this is byte-safe: the canonical table order never came from the
   BTree ordering — it is the stable sort by `(height, key bytes)`, and the
   key bytes embed child hashes (or the full leaf encoding), so two distinct
   nodes can only tie by colliding sha256 hashes, which the certificate
   format already assumes away (the same hashes are the Merkle identity of
   the tables). `CanonTerm` no longer implements `Ord`.

2. **Canonical level hash memo.** `canon_level_hash` was unmemoized and
   recursively rehashed the level tree for every `Sort` / `Const` occurrence
   in key computation. `LevelHashMemo` (value-keyed; levels are tiny)
   removes the repeated sha256 work.

3. **`json_escape` in `tools/proof-corpus`.** `write_ai_theorem_index`
   regenerates the whole-corpus theorem index on every `--build-module`,
   escaping every theorem statement; the char-by-char `String::push` loop
   plus UTF-8 decoding was ~10% of the run. It now scans bytes and
   bulk-copies maximal clean runs. The output is byte-identical, including
   `\uXXXX` escapes for the `char::is_control` set (C0, DEL, and C1 — the
   C1 range is detected as the two-byte UTF-8 sequence `0xC2 0x80..=0x9F`).
   An equivalence test against the old char-wise implementation
   (`json_escape_matches_char_wise_escaping`) pins the behavior.

## Round 3: negative result (measured, reverted — do not retry)

**Inline `Expr` metadata.** The "remaining big lever" hypothesis from rounds
1-2 was: store the loose-bvar upper bound and a has-level-params bit inline
in every kernel `Expr` node (struct `{ meta: u32, kind: ExprKind }`), so that
`subst` / `shift` / `subst_levels_expr` can skip subtrees they provably
cannot change. This was implemented to completion — constructors computing
metadata shallowly, early-outs in all three traversals, the whole workspace
migrated, all crate tests green, certificates byte-identical — and then
measured:

| Workload | Effect of inline metadata |
| --- | --- |
| `--build-module` (dev) | **+1.8%** instructions |
| `--module` verify (dev) | **+2.4%** instructions |
| `bench_package_verifier fast` (release, full corpus) | **+2.9%** instructions (94.7G → 97.2G) |

It lost on every workload, so it was reverted. The root cause is worth
remembering: the copy-on-write `subst`/`shift` already return
`None`/shared `Arc`s for unchanged subtrees, so **the metadata can only skip
the walk, never any allocation**. Against that bounded saving, the change
pays per-node metadata computation at every construction site (including a
level scan in every `Expr::konst`) and grows every node by 8 bytes in a
profile that is already allocation-bound (~40-45% of self time in
malloc/free/Arc). Combined with the round-2 postmortem (all side-table memo
variants for the same skip also lost: insert churn, refcount-1 misses, or
probe cost exceeding walk savings), the loose-bvar-skip idea is now closed
in both its known forms — side table and inline.

## What we learned

- **Dedup/id maps over canonical terms do not need ordered comparisons.**
  Whenever the final order is a sort over collision-resistant keys, BTree
  containers are pure overhead; hash-keying by the canonical sha256 is
  byte-compatible by construction. The same pattern may apply to other
  `BTreeMap`s keyed by structural values on hot paths.
- **CoW sharing already captured the allocation win.** Optimizations that
  only avoid re-walking shared/unchanged subtrees have a hard ceiling: the
  walk itself. Measure that ceiling (e.g. self time of `subst_changed`,
  ~7-8%) before paying any per-node cost for it.
- **The tool layer is part of the loop.** ~10% of `--build-module` was JSON
  string escaping in the corpus tool, invisible until the kernel-side noise
  above it was removed. Profile the whole binary, not just the kernel.
- **Measure with instructions retired, not wall time.** This machine's
  background load and E/P-core scheduling skew single runs by ±40%.
  Interleave A/B binaries copied to `/tmp`, take ≥6 runs, and prefer
  `/usr/bin/time -l` "instructions retired" for small deltas; `sample <pid>`
  for profiles.
- **Verification discipline pays.** Byte-identity of certificates plus the
  two gate scripts caught nothing this round because every risky step was
  checked before proceeding — keep that order (identity check before
  timing, gates before commit).

## Candidate levers for next time

In rough order of expected value:

1. **Arena / pool allocation for `Expr` nodes.** malloc/free + `Arc`
   drop_slow remain ~40-45% of build-module self time after round 3. A
   bump-arena per module build (or a size-class pool reused across builds)
   attacks the dominant cost directly. Hard parts: `Arc` crosses threads in
   the parallel package verifier (`Send` required), and lifetimes would
   ripple through public APIs. A contained first step: arena-allocate only
   inside `whnf`/`is_defeq` scratch reductions where results never escape.
2. **Producer-token per-decl rebuild cost** (`canonical.rs` per-decl
   pipeline). Each declaration builds its own name table, so term/level hash
   memos cannot be shared across declarations naively (keys embed resolved
   names). Options: split the key into a name-table-independent core plus a
   name binding, or batch producer-token checks per module so one collector
   serves all decls.
3. **sha2 cost in cert production.** After round 3, sha256 is a visible
   slice (~5-8%). Most of it is inherent (Merkle identity), but check for
   remaining duplicate hashing (e.g. `materialize` recomputing root-field
   keys per decl payload lookup) before assuming it is floor.
4. **Elaborator allocation behavior.** `infer_human_expr` recursion remains
   hot; the next win there is likely reusing `Ctx`/local-decl buffers across
   recursion rather than algorithmic change (the Pi quadratic is already
   fixed).
5. **Whole-corpus index regeneration.** `write_ai_theorem_index` rebuilds
   the index from the static `MODULES` table on every build. With
   `json_escape` fixed the remaining cost is mostly per-statement sha256
   (`tagged_sha256`); if it shows up again, cache statement hashes or write
   the index only when its byte content changes.

Bench commands:

```sh
# AI loop (dev profile; interleave with a baseline binary copied to /tmp)
cargo build -p npa-proof-corpus
/usr/bin/time -l target/debug/npa-proof-corpus \
  --build-module Proofs.Ai.Category.Classical --checked-in-imports

# Verifier core (release, full corpus)
cargo run --release -p npa-api --example bench_package_verifier -- fast
```

Known quirk: rebuilding a module rewrites `source.npa` without a trailing
newline, dirtying only `source_sha256` in `meta.json`; the tracked cache
under `crates/npa-api/target/npa-package-audit-cache/` churns transiently
while package gates run — re-check `git status` after the gate finishes.
