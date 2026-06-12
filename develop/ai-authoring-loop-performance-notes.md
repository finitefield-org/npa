# AI Authoring Loop Performance Notes

Date: 2026-06-12 (rounds 3-4; rounds 1-2 summarized for context)

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
| 4 | claude0612-3 / 5c79b64 | 21.1G → 13.1G instructions (−37%); ~1.44 → ~0.95 s user | incremental kernel `Ctx` in the elaborator, `MachineTerm` clone elimination in lowering, O(n) cert reachability/bvar verify |

`--module` verify is mostly production-path-independent and stays at
~0.2 s user (round 4's verify rewrite trims it ~3%).

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

## Round 4: what changed

1. **Incremental kernel `Ctx` in the elaborator**
   (`human_elaborator.rs`). `HumanLocalContext::to_kernel_ctx` rebuilt a
   kernel `Ctx` — one `String` clone plus one `Arc<LocalDecl>` allocation
   per local — on *every* kernel `infer`/`whnf`/`is_defeq` call, i.e.
   O(locals) work per call and O(nodes × locals) per declaration. The
   context now mirrors its locals into an embedded kernel `Ctx` at push
   time and `to_kernel_ctx` returns a borrow. Locals are `Arc` so the
   ~20 nested-scope `locals.clone()` sites are refcount bumps. This was
   the single largest win of the day (17.4G → 13.1G instructions; the
   kernel ignores local names, so pushing `""` mirrors the old rebuild
   exactly).

2. **`MachineTerm` deep-clone elimination in Human→Machine lowering.**
   Profiling showed `MachineTerm::clone` + drop as the top identifiable
   cost (~10%): `lower_lambda_binders` cloned the whole expected term per
   lambda group, `rename_machine_local` rebuilt (reallocated) every node
   even for no-op renames, and `lower_expr` cloned the lowering context —
   deep-cloning every local's `MachineTerm` type — at every Lam/Pi/Let.
   Fixes: thread the expected term as `Cow` (one clone at the first Pi
   decomposition; owned leftovers flow through nested lambdas), rename in
   place (touches only matching `Local` name strings; no-op when
   `from == to`), and scope mark/truncate on the shared lowering context
   instead of cloning (sound because lookups only scan `locals` and an
   error aborts the whole declaration).

3. **O(n) certificate reachability/bvar verification**
   (`npa-cert/src/verify.rs`). `verify_term_scope` memoized on
   `(TermId, depth)` pairs in a `BTreeSet`, so shared table subtrees were
   re-walked once per distinct depth with O(log n) probes.
   Children-precede-parents is verified by the encoding pass that runs
   first, so the rewrite computes per-node loose-bvar bounds in one
   forward pass (the same trick as the round-2 height fix), checks roots
   in O(1), and marks reachability with a single-visit bitset walk; level
   reachability also uses a bitset. A failing root replays the original
   depth-tracking search so the reported `InvalidBVar` error is
   byte-identical. Equivalence argument: bound(root) > 0 ⟺ some path
   reaches a bvar with index ≥ binders crossed ⟺ the old DFS errors.

## Round 4: negative result (measured, reverted)

**mimalloc as the tool-binary allocator.** With malloc/free at ~40% of
self time, swapping the proof-corpus binary's global allocator looked
promising. Measured: only ~2% faster than the macOS system allocator
(median over 8 interleaved runs, after adding dev-profile opt-level
overrides for the allocator crates — without them the allocator's debug
checks made it 3× *slower*). Not worth a native C dependency in this
repo. Conclusion: the allocator is fine; the win is in not allocating —
which rounds 3-4 then delivered by removing clones and rebuilds.

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

## What we learned (round 4 additions)

- **Hidden O(per-call) context rebuilds dwarf everything else.** A
  "convert my context to the kernel's context" helper called inside the
  per-node recursion was ~25% of the whole loop, invisible as a single
  symbol because the cost spread across malloc/String/Arc. When a hot
  recursion calls into another layer, audit what gets re-materialized per
  call and maintain it incrementally instead.
- **Allocator swaps don't fix allocation-bound code.** mimalloc bought
  ~2%; removing the allocations bought ~45%.
- **`(node, depth)` DFS memos over shared DAGs are a smell.** If a prior
  pass guarantees topological table order, per-node facts (heights —
  round 2; loose-bvar bounds — round 4) come from one forward pass.
  Keep the old search as a cold-path replayer when error payloads must
  stay identical.

## Candidate levers for next time

After round 4 the profile is flat: sha2 (~7%, mostly inherent Merkle
identity), residual malloc/free (~15%), kernel `subst_changed` walk
(~4%), `json_escape`/theorem-index regen (~2%). In rough order:

1. **sha2 in cert production.** Check for remaining duplicate hashing in
   the canonical pipeline (e.g. `CanonNodeIds::term_id` recomputing
   root-field keys per decl payload lookup) before assuming it is floor.
2. **Theorem-index regeneration.** Content depends only on the static
   `MODULES` table; generate once per binary (lazy static) or skip the
   write when bytes are unchanged.
3. **Kernel `subst` walk.** Only ~4% now; the inline-metadata and
   side-memo routes are both measured losers (see round 3) — any further
   win must come from the elaborator calling `subst`/`whnf` less, not
   from making the walk cheaper.
4. **Arena / pool allocation for `Expr` nodes.** Still plausible for the
   residual malloc share, but the share is now ~15%, the refactor is
   large (Send across the parallel verifier, lifetime ripple), and the
   mimalloc data point says the allocator itself is not slow. Revisit
   only if a profile shows allocation dominating again.

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
