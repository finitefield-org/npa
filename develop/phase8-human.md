This document is the detailed design for **Phase 8: independent checker**.
The purpose of Phase 8 is to build a mechanism that rechecks proof certificates through an independent path from the fast kernel, elaborator, tactics, and AI search produced in Phases 1-7.

The scope is these three components.

```text
- reference checker
- external checker
- CI integration
```

Implementation notes (2026-05-21):

```text
- This document describes the final target design for the Phase 8 Human Profile.
- The current repository already implements the standalone reference checker binary
  in crates/npa-checker-ref, the Phase 8 checker audit automation library substrate
  in crates/npa-api, the OCaml clean-room source / build scripts in
  checkers/npa-checker-ext/, the source-free runner path for
  `npa package verify-certs --checker external`, and the Phase 8 Release Audit
  fixture gate.
- `npa-checker-ext` is treated as existing release evidence only when a built
  executable is resolved from the runner-owned checker registry and package
  external mode passes runner policy, binary hash, and identity validation.
- The `verified_high_trust` artifact generator is implemented as
  `npa package high-trust`, but full external-checker release CI is still treated
  as a high-trust integration target. Do not generate the artifact from
  reference-checker-only evidence.
- GitHub Actions workflows have been removed from the current repository, and
  the Phase 9 Regression script is not a replacement for the Phase 8 Release
  Audit fixture gate.
- Even when Phase 8 automation lives in crates/npa-api, the trusted boundary is
  the canonical certificate and checker result. Audit automation itself is not
  part of the trusted base.
```

The main principle is:

```text
Do not stop just because the main kernel says OK.
Trust neither source, tactics, AI, elaboration, nor proof search;
recheck only the canonical certificate with independent checkers.
```

---

# 1. Role of Phase 8

The flow through Phase 7 was:

```text
source / tactic / AI search
  ↓
elaboration
  ↓
core proof term
  ↓
fast kernel check
  ↓
certificate generation
```

Phase 8 extends it to:

```text
certificate
  ↓
reference checker
  ↓
external checker
  ↓
CI / release audit
  ↓
verified_high_trust artifact
```

This is the Phase 8 target architecture. In the current repository,
`npa package high-trust` is the `verified_high_trust` artifact generator, but it
can generate an artifact only when the required evidence includes the external
checker and high-trust reference result. It cannot generate one from
reference-checker-only evidence. The final deliverable is not plain `.npa`
source, a tactic script, or an AI search log. It is a `.npcert` rechecked by
multiple checkers.

---

# 2. Checker Types

Phase 8 assumes at least three checker types.

```text
1. fast kernel
   The fast kernel used during normal development, IDE work, and proof search.

2. reference checker
   A small, readable, specification-faithful checker.
   It may be slow.

3. external checker
   A checker that runs as a separate implementation, process, and build
   environment from the main system.
   Used in high-trust modes and CI.
```

Ideally, a fourth checker will be added in the future.

```text
4. verified checker
   A checker whose correctness is formally verified in Lean, Rocq, NPA itself,
   or a similar system.
```

For the Phase 8 MVP, the following is enough.

```text
fast kernel             : Rust
reference checker       : small separate Rust implementation in crates/npa-checker-ref
external checker profile: runner / comparison fixtures in crates/npa-api
target external checker : OCaml clean-room npa-checker-ext
```

---

# 3. Trust Boundary

Phase 8 does not trust:

```text
- source parser
- notation parser
- elaborator
- implicit argument inference
- tactic engine
- simp-lite
- induction tactic
- AI premise retrieval
- AI tactic generation
- best-first search
- proof minimizer
- theorem search index
- IDE display
- generated proof script
```

The checker reads:

```text
- canonical core AST
- module certificate
- import export_hash, and certificate_hash in high-trust mode
- declaration hash
- axiom report
- declaration dependency entries inside the certificate
```

The checker does not read:

```text
- .npa source
- tactic script
- AI search trace
- pretty printed goal
- theorem search index
- source map
- externally emitted dependency graph artifact
```

In principle, the checker input is only `.npcert`.

Challenge, source, and display examples in this Phase 8 document sometimes use
`0` for readability. The checker compares `statement_core_hash` and the
canonical core AST, not display strings. In an accepted certificate, `0` must
already have been elaborated to the canonical `Const` reference to `Nat.zero`.

## 3.1 Performance Boundary with the AI Fast Path

Phase 8 independent checking and audit work must not be inserted synchronously
into the Phase 5 / Phase 7 AI candidate generation hot path.

The fast path for AI remains:

```text
Machine Surface request
  ↓
Phase 5 machine session / tactic batch / replay / verify
  ↓
Phase 7 candidate ranking / repair / minimization
  ↓
closed certificate candidate
```

Phase 8 reference / external checking, audit bundles, challenge replay, and AI
sidecar triage run at these points:

```text
- CI / nightly / release / high-trust audit
- explicit post-acceptance audit
- background or cached audit sidecar generation
- deterministic benchmark job
```

Forbidden:

```text
- synchronously running the reference / external checker for each tactic candidate expansion
- making AI sidecar / challenge generation mandatory before a Phase 5 verify response
- reading Human source, audit bundles, or external checker output inside the search loop
- blocking premise retrieval on a Phase 8 audit result that has not been generated
```

Allowed:

```text
- using an already materialized certificate hash / NormalizedCheckResult / audit summary as a cache key or ranking feature
- handing a closed fast-path candidate to a later audit job
- making a Phase 8 audit disagreement fail in release / high-trust mode
```

This boundary strengthens guarantees for accepted artifacts without increasing
the normal latency of AI candidate generation, batch replay, or verify.

---

# 4. Reference Checker

## 4.1 Purpose

The reference checker is a simple, audit-friendly checker that stays close to
the specification itself.

The fast kernel becomes complex for performance.

```text
- arena allocation
- hash-consing
- WHNF cache
- conversion cache
- parallel checking
- optimized substitution
- compact term encoding
```

Those choices matter for performance, but they are also sources of bugs. The
reference checker intentionally goes the other direction.

```text
- slow is acceptable
- cache as little as possible
- do not parallelize
- avoid over-optimization
- do not use unsafe code
- stay faithful to the specification
- prioritize correctness over error message quality
```

---

## 4.2 What the Reference Checker Checks

Given a `.npcert`, it checks:

```text
1. certificate header
2. import export_hash, and certificate_hash in high-trust mode
3. canonical encoding
4. term hash
5. declaration hash
6. declaration dependency
7. type correctness
8. conversion correctness
9. universe consistency
10. inductive declaration validity
11. axiom report correctness
12. axiom report hash
13. export hash
14. certificate hash
```

On success:

```json
{
  "status": "checked",
  "checker": "npa-checker-ref",
  "module": "Std.Nat",
  "export_hash": "sha256:...",
  "certificate_hash": "sha256:...",
  "axiom_report_hash": "sha256:...",
  "axioms_used": [],
  "declarations_checked": 128
}
```

On failure:

```json
{
  "status": "failed",
  "checker": "npa-checker-ref",
  "error_kind": "type_mismatch",
  "declaration": "Nat.add_zero",
  "expected": "Eq Nat (Nat.add n Nat.zero) n",
  "actual": "Eq Nat n n",
  "note": "expected and actual were not definitionally equal under this checker"
}
```

---

## 4.3 Non-Goals of the Reference Checker

The reference checker does not do:

```text
- source parse
- elaboration
- tactic execution
- proof search
- theorem search
- AI calls
- source map interpretation
- processing that depends on pretty printing
- network access for import resolution
```

The especially important rule is:

```text
The reference checker does not read .npa source.
```

If checking elaborated source again were enough, an elaborator bug could be
reused in the check. Phase 8 is about checking `.npcert` independently of the
elaborator.

---

# 5. Reference Checker Structure

## 5.1 Overall Structure

```text
npa-checker-ref
  ├── canonical decoder
  ├── name table checker
  ├── level checker
  ├── term checker
  ├── environment builder
  ├── type checker
  ├── conversion checker
  ├── universe checker
  ├── inductive checker
  ├── hash verifier
  ├── axiom report verifier
  └── module verifier
```

A Rust-style API would be:

```rust
fn check_certificate(cert: ModuleCert, imports: ImportStore) -> Result<CheckedModule>;

fn check_declaration(env: &mut Env, decl: &DeclCert) -> Result<()>;

fn infer(env: &Env, ctx: &Ctx, term: TermId) -> Result<TermId>;

fn is_defeq(env: &Env, ctx: &Ctx, a: TermId, b: TermId) -> Result<bool>;
```

The reference checker can use simple data structures.

```text
fast kernel:
  arena + ExprId + hash-consing + cache

reference checker:
  immutable AST + plain recursion + explicit environment
```

---

## 5.2 Certificate Decode

The reference checker first decodes the canonical binary.

It checks:

```text
- magic number is correct
- certificate format version is supported
- core spec version is supported
- section order is canonical
- term table is canonical
- name table is canonical
- declaration order is dependency order
- there is no unknown tag
- there is no duplicate name
- there is no dangling reference
```

Example error:

```json
{
  "error_kind": "non_canonical_encoding",
  "message": "term table contains unused entry",
  "term_id": 42
}
```

The Phase 8 checker rejects certificates that are noncanonical even if they are
semantically readable, because accepting them breaks hashing and reproducibility.

---

## 5.3 Import Verification

Imports are checked by hash, not only by name.

```json
{
  "module": "Std.Logic",
  "export_hash": "sha256:...",
  "certificate_hash": "sha256:..."
}
```

The reference checker:

```text
1. finds the import module certificate
2. rechecks the import certificate, or confirms it is in the checked cache
3. confirms that export_hash matches
4. in high-trust mode, also confirms that certificate_hash matches
5. adds the public environment of the import to the current environment
```

Normal mode:

```text
require export_hash
```

High-trust mode:

```text
require export_hash
require certificate_hash
require imported certificate already checked by same checker
```

This import identity follows the SHA-256 collision threat model in
`develop/core-spec-v0.1.md`. The reference checker rechecks the import
certificate and checks the downstream certificate against the public environment
that was actually resolved, but hashes alone do not prove byte-for-byte artifact
identity. Pinning with `export_hash`, `certificate_hash`, and package artifact
hashes assumes SHA-256 collision resistance cryptographically.

---

## 5.4 Declaration Check

The checker verifies each declaration.

```text
AxiomDecl:
  confirm type : Sort u
  add it to the axiom report

DefDecl:
  confirm type : Sort u
  confirm value : type
  register reducibility in the environment

TheoremDecl:
  confirm type : Sort u
  confirm proof : type
  register the proof as opaque

InductiveDecl:
  check parameters / indices / constructors / positivity / recursor
```

Pseudocode:

```rust
fn check_decl(env: &mut Env, decl: &DeclCert) -> Result<()> {
    verify_decl_hash(decl)?;

    match decl.kind {
        DeclKind::Axiom => {
            check_is_sort(env, decl.ty)?;
            env.add_axiom(decl.interface())?;
        }

        DeclKind::Def => {
            check_is_sort(env, decl.ty)?;
            check(env, Ctx::empty(), decl.value, decl.ty)?;
            env.add_def(decl.interface())?;
        }

        DeclKind::Theorem => {
            check_is_sort(env, decl.ty)?;
            check(env, Ctx::empty(), decl.proof, decl.ty)?;
            env.add_theorem_opaque(decl.interface())?;
        }

        DeclKind::Inductive => {
            check_inductive(env, decl.inductive)?;
            env.add_inductive_family(decl.interface())?;
        }
    }

    Ok(())
}
```

---

# 6. Conversion Checker in the Reference Checker

## 6.1 Policy

The reference checker implements the same conversion specification as the fast
kernel.

Phase 1 specifies:

```text
β reduction
δ reduction
ι reduction
ζ reduction
```

Excluded:

```text
η conversion
proof irrelevance conversion
quotient computation in Phase8MvpReference / default profile
untrusted theorem unfolding
```

After P9H-12, quotient-capable profiles that explicitly allow `quotient_v1`
implement `Setoid.r` projection and `Quotient.lift` computation in both the
fast kernel and reference checker. Profiles that explicitly allow `quotient_v2`
implement the binary computation rule for `Quotient.lift2` at the same boundary.
Profiles that explicitly allow `quotient_v3` implement the proposition-valued
induction computation rule for `Quotient.indProp` at the same boundary. The
Phase 8 MVP reference profile still rejects quotient certificates as
`UnsupportedCoreFeature` and does not enter the default latency of the AI fast
path or normal verify response.

---

## 6.2 WHNF

The reference checker also needs WHNF.

```text
whnf(t):
  - App (Lam A body) arg       → β
  - Let A value body           → ζ
  - Const c                    → δ if reducible
  - Recursor applied to ctor   → ι
```

Pseudocode:

```rust
fn whnf(env: &Env, ctx: &Ctx, t: Term) -> Result<Term> {
    match t {
        App(f, a) => {
            let f_nf = whnf(env, ctx, f)?;
            match f_nf {
                Lam { body, .. } => whnf(env, ctx, subst(body, a)),
                _ => Ok(App(f_nf, a)),
            }
        }

        Let { value, body, .. } => {
            whnf(env, ctx, subst(body, value))
        }

        Const(c, levels) if env.is_reducible(c) => {
            let value = env.def_value(c, levels)?;
            whnf(env, ctx, value)
        }

        RecursorApp(rec, args) if can_iota_reduce(env, rec, args) => {
            let reduced = iota_reduce(env, rec, args)?;
            whnf(env, ctx, reduced)
        }

        _ => Ok(t),
    }
}
```

---

## 6.3 Definitional Equality

```rust
fn is_defeq(env: &Env, ctx: &Ctx, a: Term, b: Term) -> Result<bool> {
    let a = whnf(env, ctx, a)?;
    let b = whnf(env, ctx, b)?;

    if alpha_equal(&a, &b) {
        return Ok(true);
    }

    match (a, b) {
        (Sort(u), Sort(v)) => level_equal(env, u, v),

        (BVar(i), BVar(j)) => Ok(i == j),

        (Const(c1, us1), Const(c2, us2)) => {
            Ok(c1 == c2 && levels_equal(env, us1, us2)?)
        }

        (App(f1, x1), App(f2, x2)) => {
            Ok(
                is_defeq(env, ctx, *f1, *f2)?
                && is_defeq(env, ctx, *x1, *x2)?
            )
        }

        (Pi { ty: a1, body: b1, .. },
         Pi { ty: a2, body: b2, .. }) => {
            Ok(
                is_defeq(env, ctx, *a1, *a2)?
                && is_defeq_under_binder(env, ctx, *b1, *b2)?
            )
        }

        (Lam { ty: a1, body: b1, .. },
         Lam { ty: a2, body: b2, .. }) => {
            Ok(
                is_defeq(env, ctx, *a1, *a2)?
                && is_defeq_under_binder(env, ctx, *b1, *b2)?
            )
        }

        (Let { .. }, _) | (_, Let { .. }) => {
            unreachable!("whnf should reduce let at head")
        }

        _ => Ok(false),
    }
}
```

The reference checker prioritizes clarity over performance.

---

# 7. Inductive Checking

## 7.1 Checked Content

The reference checker also checks inductive declarations.

```text
- parameters are well-typed
- indices are well-typed
- result sort is valid
- constructors are well-typed
- constructor result targets the inductive being declared
- recursive occurrences are strictly positive
- generated recursor type is correct
- iota rules match the declaration
```

---

## 7.2 Strict Positivity

The Phase 8 MVP positivity checker may be conservative.

Allow:

```text
Nat
List A
A -> I
I as a constructor argument, not I -> I
```

Reject:

```text
(I -> A) -> I
negative occurrence
nested inductive
mutual inductive
```

If Phases 1-6 handle only `Nat`, `Eq`, and `List`, checking that range reliably
is enough at first.

---

# 8. Hash Verification

## 8.1 The Checker Does Not Trust Hashes

Hashes stored in a certificate are not trusted. The reference checker always
recomputes them.

Checked hashes:

```text
- term_hash
- decl_interface_hash
- decl_certificate_hash
- export_hash
- certificate_hash
- axiom_report_hash
```

Flow:

```text
decode certificate
  ↓
canonical encode again
  ↓
recompute hashes
  ↓
compare with stored hashes
```

A mismatch fails.

```json
{
  "error_kind": "hash_mismatch",
  "expected": "sha256:abc...",
  "actual": "sha256:def...",
  "object": "decl_certificate_hash",
  "declaration": "Nat.add_zero"
}
```

---

## 8.2 Hash Policy

Domain separation is mandatory.

```text
H("NPA-TERM-0.1" || term_encoding)
H("NPA-DECL-IFACE-0.1" || decl_interface)
H("NPA-DECL-CERT-0.1" || decl_certificate)
H("NPA-MODULE-EXPORT-0.1" || export_block)
H("NPA-MODULE-CERT-0.1" || trusted_payload_without_certificate_hash)
H("NPA-AXIOM-REPORT-0.1" || axiom_report)
```

This reduces the risk of accidentally reusing different data kinds as the same
hash.

---

# 9. Axiom Report Verification

## 9.1 Recompute the Axiom Report

The axiom report inside a certificate is not a log. It is checked data.

The reference checker recomputes the axiom set from declaration dependencies.

```text
axioms(AxiomDecl a)
  = {a}

axioms(DefDecl d)
  = axioms(type(d)) ∪ axioms(value(d)) ∪ axioms(dependencies)

axioms(TheoremDecl t)
  = axioms(type(t)) ∪ axioms(proof(t)) ∪ axioms(dependencies)

axioms(InductiveDecl I)
  = axioms(all constructor types and parameter types)
```

It then confirms that the recomputed result matches the certificate axiom
report.

---

## 9.2 Trust Policy

The checker accepts a policy file.

```json
{
  "deny_sorry": true,
  "deny_custom_axioms": true,
  "allowed_axioms": [
    "Classical.choice",
    "Propext"
  ]
}
```

For the high-trust standard library:

```json
{
  "deny_sorry": true,
  "deny_custom_axioms": true,
  "allowed_axioms": []
}
```

If the current kernel represents `Std.Logic.Eq.rec` as a standard recursor axiom
with `AxiomDecl`, the high-trust standard-library policy may allow exactly
`Std.Logic.Eq.rec` as a standard exception. This does not allow custom axioms.

Result:

```json
{
  "axioms_used": [],
  "contains_sorry": false,
  "safe_for_high_trust": true
}
```

In this example, `axioms_used = []` is the minimal no-custom-axiom form. When
the current kernel emits standard `Eq.rec` as an `AxiomDecl`, `axioms_used` may
contain exactly `Std.Logic.Eq.rec`; any other axiom is a failure.

With a forbidden axiom:

```json
{
  "status": "failed",
  "error_kind": "forbidden_axiom",
  "axiom": "synthetic.sorry.Std.Nat.add_zero",
  "declaration": "Nat.add_zero"
}
```

---

# 10. External Checker

## 10.1 Purpose

The external checker runs independently of the main build system.

The reference checker is an implementation close to the specification. The
external checker is operationally separated from the main system.

Ideal properties:

```text
- separate binary
- separate process
- separate build configuration
- separate language or implementation
- does not read source code
- no network access
- no plugins
- no tactic execution
- no AI
```

---

## 10.2 External Checker CLI

The source for the standalone external checker binary lives in
`checkers/npa-checker-ext/`. This binary is treated as release evidence only
when the built executable is resolved from the runner-owned checker registry and
the package `--checker external` integration passes runner policy, binary hash,
and checker identity validation. The target binary specification is defined as
an OCaml clean-room implementation in `develop/npa-checker-ext-ocaml.md`.

```bash
npa-checker-ext \
  --cert build/Std/Nat.npcert \
  --import-dir build/certs \
  --policy policies/high_trust.toml \
  --output json
```

Checker raw result output:

```json
{
  "schema": "npa.independent-checker.checker_raw_result.v1",
  "checker_id": "npa-checker-ext",
  "checker_version": "0.1.0",
  "checker_build_hash": "sha256:...",
  "status": "checked",
  "module": "Std.Nat",
  "certificate_hash": "sha256:...",
  "export_hash": "sha256:...",
  "axiom_report_hash": "sha256:..."
}
```

Process metadata, resource usage, and diagnostics belong in the runner-owned
`MachineCheckResult`. They are not part of the semantic identity of the checker
raw result.

---

## 10.3 External Checker Input

External checker input is restricted.

```text
Allowed as input:
  - .npcert
  - import certificate directory
  - policy file
  - optional expected statement hash
  - optional expected export hash

Not allowed as input:
  - source .npa
  - tactic script
  - generated theorem search index
  - proof search trace
  - untrusted plugin
  - remote import
```

---

## 10.4 Challenge Mode

For high-trust use, fix only the proposition to be proved in a separate file.

```json
{
  "challenge": "theorem add_zero : ∀ n : Nat, n + 0 = n",
  "statement_core_hash": "sha256:...",
  "allowed_axioms": [],
  "imports": [
    {
      "module": "Std.Nat",
      "export_hash": "sha256:...",
      "certificate_hash": "sha256:..."
    }
  ]
}
```

`allowed_axioms` is the policy fixed by the challenge owner. If a kernel
represents standard `Eq.rec` as an axiom and the certificate depends on
`Eq.rec`, this field may also contain exactly `Std.Logic.Eq.rec`.

The external checker verifies that the theorem statement in the proof
certificate matches the challenge.

```text
certificate theorem statement hash
  ==
challenge statement hash
```

This prevents AI or proof search from silently proving a similar but different
theorem and treating it as success.

---

## 10.5 Audit Bundle

For releases, papers, and contest submissions, produce an audit bundle.

```text
audit/
  challenge.json
  proof.npcert
  imports/
    Std.Logic.npcert
    Std.Nat.npcert
  policy.json
  checker-output-fast.json
  checker-output-ref.json
  checker-output-ext.json
```

The audit runner can build the external checker input from this bundle alone.

However, the mandatory CLI contract for `npa-checker-ext` itself is `--cert` /
`--import-dir` / `--policy` / `--output json`. The audit runner materializes
this source-free checker invocation from the bundle. Bundle validation and
challenge coverage are the responsibility of the runner / audit command layer.

---

# 11. Checker Disagreement

## 11.1 Disagreement Is Always a Failure

If the fast kernel, reference checker, and external checker disagree, CI fails.

Cases:

```text
fast kernel OK, reference checker FAIL
  → possible fast kernel bug or certificate generator bug

fast kernel FAIL, reference checker OK
  → possible overstrict fast kernel, or too-permissive reference checker

reference checker OK, external checker FAIL
  → checker implementation difference, or environment difference

hashes match but axiom reports disagree
  → serious bug in certificate generation or report computation
```

No such case may be released.

---

## 11.2 Disagreement Report

On disagreement, emit minimal reproduction information.

```json
{
  "status": "checker_disagreement",
  "module": "Std.Nat",
  "declaration": "Nat.add_zero",
  "fast_kernel": {
    "status": "ok"
  },
  "reference_checker": {
    "status": "failed",
    "error_kind": "conversion_failed"
  },
  "external_checker": {
    "status": "failed",
    "error_kind": "conversion_failed"
  },
  "artifact": {
    "certificate_hash": "sha256:...",
    "decl_certificate_hash": "sha256:..."
  }
}
```

---

# 12. CI Integration

## 12.1 Purpose of CI

CI automatically guarantees:

```text
- source builds
- certificates are generated
- certificates are checked by the fast kernel
- certificates are rechecked by the reference checker
- certificates are rechecked by the external checker
- import export_hash, and certificate_hash in high-trust mode, match
- declaration hashes match
- axiom reports are correct
- there is no forbidden axiom or sorry
- theorem index matches the certificate
- proof minimization has not changed the theorem
- there is no performance regression
```

---

## 12.2 Overall CI Pipeline

```text
Stage 1: source lint
Stage 2: build certificates
Stage 3: fast kernel check
Stage 4: reference checker check
Stage 5: external checker check
Stage 6: axiom policy check
Stage 7: hash reproducibility check
Stage 8: theorem index validation
Stage 9: tactic/search regression tests
Stage 10: performance benchmarks
Stage 11: audit bundle generation
```

---

# 13. CI Stage Details

## Stage 1: Source Lint

Checks:

```text
- forbidden tokens
- naming convention
- namespace convention
- duplicate names
- suspicious shadowing
- unsafe declarations
- unapproved axiom
- sorry/admit
```

Example:

```text
fail:
  theorem Nat.add_zero ... := sorry
```

---

## Stage 2: Build Certificates

Generate `.npcert` files from source.

```bash
npa build --emit-cert --locked
```

Artifacts:

```text
build/
  Std/Logic.npcert
  Std/Nat.npcert
  Std/List.npcert
  Std/Algebra/Basic.npcert
```

The fast kernel is used in this stage.

---

## Stage 3: Fast Kernel Check

```bash
npa check build/Std/Nat.npcert
```

Checks:

```text
- core term type check
- conversion
- inductive rules
- declaration hash
- export_hash / certificate_hash / axiom_report_hash
```

---

## Stage 4: Reference Checker Check

```bash
cargo run -p npa-checker-ref -- \
  --cert build/Std/Nat.npcert \
  --import-dir build \
  --policy policies/std.json \
  --output json
```

PRs check changed modules and their reverse dependencies.

```text
changed module:
  Std.Nat

also check reverse dependencies:
  Std.List
```

Nightly checks every module.

---

## Stage 5: External Checker Check

The standalone external checker command is:

```bash
npa-checker-ext \
  --cert build/Std/Nat.npcert \
  --import-dir build \
  --policy policies/std.toml \
  --output json
```

This invocation is release evidence only when the `npa-checker-ext` binary has
been built and selected from the runner-owned checker registry. In the current
repository's CI fixtures, the `external` profile `MachineCheckResult` is fixed
by the normalization, comparison, and release bundle tests in `crates/npa-api`.

When possible, run the external checker in a separate container.

```text
- no network
- read-only cert directory
- no source directory mounted
- no build scripts
- no plugin
```

This makes it easier to guarantee that the external checker really checks only
certificates.

---

## Stage 6: Axiom Policy Check

```bash
npa audit axioms build/Std/Nat.npcert --policy policies/std.json
```

For the standard library:

```text
allowed axioms = []
or exactly [Std.Logic.Eq.rec] when the current kernel emits Eq.rec as the standard recursor axiom
```

Failure conditions:

```text
- there is a sorry
- there is a custom axiom
- there is an axiom not in the allowlist
- the axiom report differs from the recomputed result
```

Output:

```json
{
  "module": "Std.Nat",
  "axioms_used": [],
  "contains_sorry": false,
  "safe_for_high_trust": true
}
```

This output example is also the minimal no-custom-axiom form. In a kernel that
emits standard `Eq.rec` as an `AxiomDecl`, exactly `Std.Logic.Eq.rec` may appear
in `axioms_used` as the standard exception.

---

## Stage 7: Hash Reproducibility Check

Confirm that the same source and lockfile produce the same certificate hash.

```bash
npa clean
npa build --emit-cert --locked
hash1=$(npa cert-hash build/Std/Nat.npcert)

npa clean
npa build --emit-cert --locked
hash2=$(npa cert-hash build/Std/Nat.npcert)

test "$hash1" = "$hash2"
```

This detects:

```text
- nondeterministic name allocation
- nondeterministic declaration order
- timestamp contamination
- random seed contamination
- dependency on hash table iteration order
```

---

## Stage 8: Theorem Index Validation

Validate that the Phase 6/7 theorem search index matches the certificate.

```bash
npa index validate \
  --index build/Std/Nat.index.json \
  --cert build/Std/Nat.npcert
```

Checks:

```text
- each theorem in the index exists in the certificate
- statement hash matches
- attributes match declaration metadata
- rewrite lhs/rhs matches the theorem statement
- axiom_deps matches the axiom report
```

AI search depends on the theorem index, so this matters. The theorem index
itself is not trusted. If it is wrong, the final proof check still fails or
passes independently, but search quality and safety policy can be affected.

---

## Stage 9: Tactic/Search Regression Tests

Test Phase 4/7 tactics and AI search.

```bash
npa test tactics
npa test search
```

Examples:

```text
intro/exact:
  theorem id_nat : Nat -> Nat

rw:
  theorem rw_test (a b : Nat) (h : a = b) : a = a

simp-lite:
  theorem add_zero (n : Nat) : n + 0 = n

induction:
  theorem zero_add (n : Nat) : 0 + n = n

AI search:
  automatically prove List.append_nil
```

Important:

```text
tactic/search regression is a convenience test.
It is not a replacement for the checker.
```

Always run certificate checking at the end.

---

## Stage 10: Performance Benchmarks

Detect speed regressions.

Measured values:

```text
- fast kernel check time
- reference checker time
- external checker time
- conversion checker time
- largest declaration check time
- certificate decode time
- memory usage
- theorem index build time
- AI search success/time on benchmark set
```

Example:

```json
{
  "module": "Std.Nat",
  "fast_kernel_ms": 120,
  "reference_checker_ms": 950,
  "external_checker_ms": 1100,
  "memory_mb": 42
}
```

CI policy:

```text
PR:
  warn or fail on major fast kernel / Machine API / theorem index build / AI benchmark regressions
  do not put reference / external checker benchmarks in the synchronous required PR job;
  treat them as separate jobs or cached audit results

nightly:
  record detailed benchmarks including reference / external checker

release:
  fail if thresholds are exceeded
```

---

## Stage 11: Audit Bundle Generation

Generate audit bundles for releases and high-trust verification.

```bash
npa audit bundle \
  --module Std.Nat \
  --cert build/Std/Nat.npcert \
  --imports build \
  --policy policies/std.json \
  --out audit/Std.Nat/
```

Contents:

```text
audit/Std.Nat/
  proof.npcert
  imports/
  policy.json
  checker-fast.json
  checker-ref.json
  checker-ext.json
  hashes.json
  axiom-report.json
```

---

# 14. CI Modes

## 14.1 Pull Request Mode

Prioritize speed.

```text
- changed modules
- reverse dependencies
- fast kernel check
- reference checker on changed certs
- external checker optional / on-demand only
- axiom policy
- basic tactic regression
```

The required checker profile for PR mode is speed-oriented and requires only
`reference`. The external checker is required in nightly / release / high-trust
mode.

## 14.2 Nightly Mode

Prioritize coverage.

```text
- full library check
- full reference checker
- full external checker
- fuzz tests
- theorem index validation
- AI search benchmark
- performance benchmark
```

## 14.3 Release Mode

High trust.

```text
- clean build
- locked dependencies
- deterministic rebuild
- full fast kernel check
- full reference checker
- full external checker
- import recursive verification
- audit bundle generation
- signed release artifacts
```

## 14.4 High-Trust Mode

For papers, contests, and safety-sensitive use.

```text
- challenge file required
- proof certificate required
- source ignored
- no network
- no plugin
- no custom axiom; exact standard `Std.Logic.Eq.rec` exception only when the kernel emits it as a recursor axiom
- no sorry
- all imports recursively checked
- at least two independent checkers required
```

## 14.5 Implementation Fixed Points

The CI workflow treats the sequences returned by
`IndependentCheckerTrustMode::ci_commands()` and
`IndependentCheckerTrustMode::ci_pass_requirements()` as authoritative. PR mode
requires changed certificate / reverse dependency selection and the reference
checker. External checker, full recursive import checking, and audit artifact
coverage are required in nightly / release / high-trust mode and above.

Performance benchmark classification is fixed by
`independent_checker_performance_gates()`. Synchronous required PR benchmarks
are limited to the fast kernel, Machine API, theorem index build, and AI
benchmark. Reference / external checker benchmarks are background jobs or
cached audit results. These performance gates are regression / release policy
gates, not proof acceptance boundaries.

The current repository has these local gates.

```text
scripts/phase8-release-audit.sh:
  Run scripts/phase8-release-audit.sh. It fixes the source-free reference
  checker binary, independent checker audit substrate, standard-library release
  audit fixture, and AI fast path boundary.

scripts/phase9-regression.sh:
  Run scripts/phase9-regression.sh. It fixes the Phase 9 fixtures, fmt, clippy,
  and workspace-wide regression checks.
```

GitHub Actions workflows have been removed from the current repository.
External theorem-library CI is integration scope after the package contract is
fixed.

Phase 8 Release Audit is a narrow gate for the release audit fixture. Phase 9
Regression is a broader regression gate that includes later phases; it is not a
replacement for full external-checker release audit.

---

# 15. Fuzzing and Mutation Tests

Phase 8 also needs checker robustness tests.

## 15.1 Certificate Fuzzing

Generate many invalid certificates and confirm that the checker rejects them
safely.

Example mutations:

```text
- corrupt term tag
- dangling term reference
- wrong binder index
- wrong universe level
- wrong declaration hash
- reordered declaration
- missing import
- changed proof term
- axiom report falsification
- noncanonical name table
```

Expected:

```text
checker rejects clearly without panic
```

## 15.2 Proof Mutation

Slightly corrupt a valid proof.

```text
Eq.refl n
  ↓
Eq.refl m
```

Or:

```text
Nat.add_zero proof
  ↓
change part of the proof term to another term
```

Expected:

```text
fast kernel / reference checker / external checker all reject
```

## 15.3 Differential Testing

Feed the same certificate to the fast kernel and reference checker, then compare
results.

```text
same input
  ↓
fast kernel result
reference checker result
external checker result
  ↓
must agree
```

Any disagreement fails.

---

# 16. Recommended Implementation Languages

In Phase 8, the fast kernel and reference checker should not share too much of
the same design or code.

Recommended:

```text
fast kernel:
  Rust

reference checker:
  small separate Rust implementation in crates/npa-checker-ref in the current repository
  can later be replaced with OCaml / Haskell / another implementation

external checker:
  OCaml clean-room npa-checker-ext
  separate process / build implementation that does not depend on Rust workspace crates

future verified checker:
  NPA itself / Lean / Rocq
```

Avoid:

```text
fast kernel and reference checker almost entirely sharing the same internal library
```

That risks sharing the same bug.

Acceptable sharing:

```text
- certificate format specification
- test cases
- golden certificates
```

Avoid sharing:

```text
- conversion checker implementation
- type checker implementation
- inductive checker implementation
- positivity checker implementation
```

---

# 17. Checker API

## 17.1 `/check/certificate`

This may be provided as a local API.

```json
POST /check/certificate
{
  "certificate_path": "build/Std/Nat.npcert",
  "checker": "reference",
  "policy": {
    "deny_sorry": true,
    "deny_custom_axioms": true,
    "allowed_axioms": []
  }
}
```

When checking a `Std.Nat` certificate in a kernel that represents standard
`Eq.rec` as an axiom, this policy may also allow exactly `Std.Logic.Eq.rec`.

Response:

```json
{
  "status": "checked",
  "checker": "reference",
  "module": "Std.Nat",
  "certificate_hash": "sha256:...",
  "export_hash": "sha256:...",
  "axiom_report_hash": "sha256:...",
  "axioms_used": [],
  "time_ms": 950
}
```

## 17.2 `/check/audit_bundle`

```json
POST /check/audit_bundle
{
  "bundle_path": "audit/Std.Nat",
  "checker": "external"
}
```

Response:

```json
{
  "status": "verified_audit_bundle",
  "challenge_statement_match": true,
  "imports_checked": true,
  "policy_satisfied": true
}
```

---

# 18. Minimal Command Set

Phase 8 gates available in the current repository:

```bash
cargo test -p npa-checker-ref
cargo test -p npa-api independent_checker
cargo test -p npa-api ai_search
./scripts/phase8-release-audit.sh
```

Source-free check command for the standalone reference checker binary:

```bash
cargo run -p npa-checker-ref -- \
  --cert build/certs/Std/Nat.npcert \
  --import-dir build/certs \
  --policy policies/std.json \
  --output json
```

External checker runner / release blocker fixtures are fixed in `crates/npa-api`.
The target specification for the OCaml clean-room external checker itself is
`develop/npa-checker-ext-ocaml.md`.

```bash
cargo test -p npa-api independent_checker::tests::p8h00_pr_mode_requires_reference_and_keeps_external_on_demand_only
cargo test -p npa-api independent_checker::tests::independent_checker_challenge_p8h13_differential_disagreements_fail_ci
cargo test -p npa-api independent_checker::tests::m12_release_bundle_generates_manifest_and_validation_auxiliary_passes
```

`npa-check ...`, `npa cert ...`, and `npa audit ...` are target command
contracts from the Phase 8 AI document. In the current repository, the
standalone `npa-check` CLI and audit-bundle CLI are still target integration.
The same semantics are fixed through library APIs, the `npa-checker-ref` binary,
the `npa-checker-ext` package runner path, deterministic tests, and CI fixture
workflow. `npa-checker-ext` itself is not release-gate evidence until a built
binary is resolved from the registry.

Storage locations and generated artifact policy for release / high-trust audit
artifacts:

```text
bundle root:
  build/release-audit/<module>/

manifest:
  build/release-audit/<module>/manifest.json

bundle-local artifact:
  build/release-audit/<module>/artifacts/<kind>/<file_hash_without_sha256_prefix>.json

post-bundle audit result:
  build/aux/<module>.audit-bundle.json
```

`ReleaseAuditBundleManifest` records only workspace-relative paths from
`bundle_root`. Artifact files are content-addressed; the bundle is invalid if
the filename hash and file byte hash differ. Generated artifacts are not the
committed source of truth; they are regenerated from source fixtures, Rust
builders, deterministic tests, and CI. The basis for a release / high-trust pass
is the canonical certificate, checker result, NormalizedCheckResult comparison,
required AuxiliaryResult, and ReleaseAuditBundleManifest validation, not the
mere existence of generated artifacts. AI sidecars may be stored as metadata /
diagnostic artifacts, but they are outside the trust boundary.

---

# 19. Phase 8 Implementation Order

Recommended order:

```text
1. Certificate decoder for reference checker
   Read .npcert without source.

2. Hash verifier
   Recompute term / decl / export / certificate / axiom_report hashes.

3. Environment builder
   Build an environment from import certificates.

4. Minimal type checker
   Sort / Pi / Lambda / App / Let / Const.

5. Conversion checker
   Start with βδ, then ζ, then ι.

6. Def / theorem check
   Confirm value : type and proof : type.

7. Axiom report recomputation
   Compare against the report inside the certificate.

8. Inductive checker
   Check simple inductives for Nat / Eq / List.

9. External checker CLI
   Source-free, cert-only, with policy.

10. CI integration
    PR / nightly / release pipeline.

11. Differential testing
    fast kernel vs reference vs external.

12. Fuzzing / mutation testing
    Confirm invalid certificates are rejected.

13. Audit bundle
    Artifact for high-trust mode.
```

---

# 20. Phase 8 Test Cases

## 20.1 Happy Path

```text
Std.Logic.npcert
Std.Nat.npcert
Std.List.npcert
Std.Algebra.Basic.npcert
```

Expected:

```text
fast kernel OK
reference checker OK
external checker OK
axioms_used = [] or exact standard Std.Logic.Eq.rec only
custom axioms = []
```

## 20.2 Hash Tampering

Change one byte of the proof term for `Nat.add_zero`.

Expected:

```text
hash mismatch
or type check failure
```

## 20.3 Axiom Report Tampering

Remove an actually used axiom from the axiom report.

Expected:

```text
AxiomReportMismatch
```

## 20.4 Import Hash Mismatch

Change the export_hash of `Std.Logic`, which `Std.Nat` depends on.

Expected:

```text
ImportHashMismatch
```

## 20.5 Theorem Statement Mismatch

Challenge:

```text
∀ n : Nat, n + 0 = n
```

Proof certificate:

```text
∀ n : Nat, n = n
```

Expected:

```text
ChallengeStatementMismatch
```

## 20.6 Noncanonical Certificate

Add an unused entry to the term table.

Expected:

```text
NonCanonicalEncoding
```

## 20.7 Forbidden Axiom

Check a certificate that uses `Classical.choice` with an empty allowlist.

Expected:

```text
ForbiddenAxiom
```

---

# 21. Items Excluded from Phase 8

Items that can be deferred in the MVP:

```text
- formally verified checker
- complex mutual/nested inductive checking
- quotient computation in Phase8MvpReference / default profile
- proof irrelevance conversion
- η conversion
- external SMT certificate checker
- distributed certificate verification
- cryptographic signature infrastructure
```

The top priority is independently rechecking:

```text
Nat / Eq / List / basic theorem
```

without source.

---

# 22. Phase 8 Completion Criteria

The conditions for considering Phase 8 complete, and the gates fixed in the
current repository, are:

| Condition | Fixed Point in the Current Repository |
| --- | --- |
| The reference checker can check `.npcert` without source | `cargo test -p npa-checker-ref`, `cargo run -p npa-checker-ref -- --cert ... --output json` |
| The external checker runner does not read source / tactics / AI traces | runner policy / forbidden input tests in `cargo test -p npa-api independent_checker` |
| import `export_hash` / high-trust `certificate_hash` can be checked | `npa-cert` / `npa-checker-ref` high-trust import tests in `cargo test --workspace` |
| declaration / export / certificate / axiom report hashes can be recomputed | `cargo test -p npa-checker-ref` and `cargo test -p npa-api independent_checker` |
| forbidden axioms / sorry can be rejected | `cargo test -p npa-checker-ref`, `cargo test -p npa-api independent_checker` |
| `Std.Logic` / `Std.Nat` / `Std.List` / `Std.Algebra.Basic` can be rechecked without source | `cargo test -p npa-api --lib std_library::tests::audits_mvp_release_artifacts_for_independent_checker` |
| fast kernel / reference / external profile comparison disagreement becomes a release blocker | `cargo test -p npa-api independent_checker::tests::independent_checker_challenge_p8h13_differential_disagreements_fail_ci` |
| audit bundles can be generated and validated | `cargo test -p npa-api independent_checker::tests::m12_release_bundle_generates_manifest_and_validation_auxiliary_passes` |
| release profile full independent-check requirements are fixed | `IndependentCheckerTrustMode::Release.ci_commands()` / `ci_pass_requirements()`, P8H-14 release/high-trust tests, `./scripts/phase8-release-audit.sh` |
| Phase 8 audit does not increase normal AI candidate hot-path latency | `cargo test -p npa-api ai_search` and step 4 of Phase 8 Release Audit |

The current repository's `scripts/phase8-release-audit.sh` is a fixture gate.
Full external-checker release audit CI and the CI connection for
`package high-trust` remain target integration work. `npa-checker-ext` is not
release evidence until a built binary is verified through the runner-owned
registry and package external mode. AI sidecars are diagnostic / metadata and
are not part of Phase 8 completion criteria or release-blocker evidence.

---

# 23. One-Sentence Summary

Phase 8 is the stage where proofs produced by the prover are rechecked through a
path independent from the prover itself.

The core flow is:

```text
.npcert
  ↓
reference checker
  ↓
external checker
  ↓
axiom/hash/import policy check
  ↓
enforced by CI
  ↓
verified_high_trust artifact
```

This is the target high-trust flow. It does not mean that a
`verified_high_trust` artifact may currently be generated from reference-only
evidence, or that the external checker is required in PR mode. With this flow,
even if AI search, tactics, the elaborator, or the fast kernel has a bug, the
independent checker can reject the final certificate.

The final ideal is to call something high-trust verified only when:

```text
"A proof was found" is not enough.
"Multiple independent checkers checked the same certificate,
 and the import export_hash / high-trust certificate_hash and axiom policy
 were also satisfied" is the high-trust verified result.
```
