# Proof Corpus Field Theory Roadmap Todo

Source: `develop/proof-corpus-field-theory-roadmap.md`

このタスク分解は、proof corpus に体論ルートを追加する作業を、
後続の実装エージェントが 1 milestone ずつ進められる単位に分けたものです。
公開 `npa-mathlib` closure 全体のリリース順は
`develop/npa-mathlib-next-closure-roadmap.md` を優先し、この文書は体論ルート内の
局所的な追加順だけを扱います。

---

## Scope

対象:

```text
- `Proofs.Ai.Algebra.AbstractField` foundation
- 逆元・除法・Nonzero の基本計算補題
- field hom と既存 `RingHomLawArgs` の bridge
- field から integral domain への bridge
- field ideal / quotient bridge
- `AbstractOrderedField` との互換 bridge theorem
- proof corpus package metadata / AI theorem index / README などの非信頼 sidecar 更新
```

非対象:

```text
- core calculus への field / inv / div primitive 追加
- typeclass search、implicit arguments、overloaded notation、ring tactic の導入
- kernel、certificate format、independent checker の trusted base 拡張
- 公開 `npa-mathlib` closure 全体の優先順位変更
- registry server、online theorem search、LLM / RAG runtime integration
```

信頼境界:

```text
信頼しない:
  source.npa / replay.json / meta.json / theorem index / roadmap / todo / AI proof candidate

信頼する:
  canonical .npcert
  deterministic export_hash / certificate_hash / axiom_report_hash
  kernel / certificate verifier verdict
  source-free independent checker verdict
```

---

## Current Implementation Facts

```text
Proofs.Ai.Algebra.AbstractRing
  RingLawArgs, two, sq, ring projection theorems, cancellation helpers

Proofs.Ai.Algebra.AbstractRingFirstIsoBase
  RingHomLawArgs, RingImagePred, RingKerQuot, RingKerQuotMk, RingKerQuotToS,
  RingKerQuotAdd, RingKerQuotZero, RingKerQuotNeg, and RingKerQuotMulRep

Proofs.Ai.Algebra.AbstractUfdPrimeFactorization
  local UfdNonzero and IntegralDomainLawArgs already exist for UFD, but not as a reusable field layer

Proofs.Ai.Algebra.AbstractOrderedField
  OrderedFieldLawArgs, le / lt / sqrt / Nonneg / Positive, square and sqrt order projections

tools/proof-corpus
  currently owns source / certificate / meta / replay generation for proof-corpus modules

proofs/npa-package.toml and proofs/manifest.toml
  package fixture and legacy corpus manifest must stay deterministic when new modules are added
```

Implication:

- `AbstractField` should not reuse the UFD-local `UfdNonzero` by import at first. It should define the
  field-level API deliberately, then later bridges can connect it to UFD / integral-domain APIs.
- `AbstractOrderedField` should not be destructively rewritten in the first milestones. Add bridge
  theorems only after the standalone field layer is certificate-backed.

---

## Milestones

### FT-00 Fix Field API Shape And Corpus Insertion Points

- Status: Completed
- Depends on: None
- Inputs:
  - `develop/proof-corpus-field-theory-roadmap.md`
  - `proofs/README.md`
  - `proofs/npa-package.toml`
  - `proofs/manifest.toml`
  - `tools/proof-corpus/src/main.rs`
- Code or documentation areas:
  - `develop/proof-corpus-field-theory-roadmap-todo.md`
  - `tools/proof-corpus/src/main.rs`
  - `proofs/README.md`
- Tasks:
  - Confirm final theorem and definition names for `Nonzero`, `div`, and `FieldLawArgs`.
  - Decide whether `Nonzero` is introduced in `AbstractField` even though UFD has a local
    `UfdNonzero` definition.
  - Identify the exact generator insertion point and import order for the new algebra modules.
  - Record the planned module order in proof-corpus documentation before adding certificates.
- Deliverables:
  - A short implementation note or README update that fixes field API names and module order.
  - A generator insertion plan that keeps `AbstractField` dependent only on `Std.Logic.Eq` and
    `Proofs.Ai.Algebra.AbstractRing`.
- Acceptance criteria:
  - Later milestones do not need to guess whether `Nonzero` is reused, renamed, or bridge-only.
  - The planned import order is acyclic and keeps quotient / CRT dependencies out of the
    foundation module.
  - No public closure order in `develop/npa-mathlib-next-closure-roadmap.md` is changed by this
    planning step.
- Verification:
  - `rg -n "AbstractField|FieldLawArgs|Nonzero|FieldHomLawArgs" develop proofs tools/proof-corpus/src/main.rs`
  - `git diff --check`
- Notes:
  - This milestone may be documentation-only if the implementation agent can infer the insertion
    point from nearby generated algebra modules.
  - Completed before FT-02: `AbstractField` uses its own `Nonzero`, `div`, and `FieldLawArgs`,
    imports only `Std.Logic.Eq` and `Proofs.Ai.Algebra.AbstractRing`, and is documented in
    `proofs/README.md`.

### FT-01 Add `Proofs.Ai.Algebra.AbstractField` Foundation

- Status: Completed
- Depends on: FT-00
- Inputs:
  - `Proofs.Ai.Algebra.AbstractRing`
  - `Std.Logic.Eq`
  - `develop/proof-corpus-ai-workflow.md`
- Code or documentation areas:
  - `tools/proof-corpus/src/main.rs`
  - `proofs/Proofs/Ai/Algebra/AbstractField/`
  - `proofs/manifest.toml`
  - `proofs/npa-package.toml`
  - `proofs/generated/package-lock.json`
  - `proofs/generated/ai-theorem-index.json`
  - `proofs/README.md`
- Tasks:
  - Add `Nonzero`, `div`, and `FieldLawArgs`.
  - Add projection theorem targets:
    - `field_ring_laws`
    - `field_zero_ne_one`
    - `field_inv_mul_cancel`
    - `field_mul_inv_cancel`
    - `field_div_eq_mul_inv`
  - Generate source, certificate, replay, meta, package, manifest, and AI index artifacts.
  - Document the module in `proofs/README.md`.
- Deliverables:
  - Checked-in `Proofs/Ai/Algebra/AbstractField/{source.npa,replay.json,meta.json,certificate.npcert}`.
  - Updated package / manifest / generated metadata for the new module.
  - README entry describing the foundation module and its trusted-boundary role.
- Acceptance criteria:
  - `FieldLawArgs` separates `RingLawArgs` from field-specific inverse / nonzero laws.
  - `div` is definitionally or theorem-level connected to `mul a (inv b)` and is not a core
    primitive.
  - No new unchecked custom axiom is introduced beyond existing corpus policy.
  - `field_ring_laws` lets downstream modules reuse `AbstractRing` theorem targets.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractField`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractField`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `git diff --check`
- Notes:
  - Keep the module small. Do not import quotient, first-isomorphism, CRT, UFD, or ordered-field
    modules in this milestone.
  - Completed before FT-02 with checked-in source, replay, meta, certificate, manifest, package,
    package-lock, and AI theorem index artifacts for `Proofs.Ai.Algebra.AbstractField`.

### FT-02 Add Basic Field Calculation Lemmas

- Status: Completed
- Depends on: FT-01
- Inputs:
  - `Proofs.Ai.Algebra.AbstractField`
  - `Proofs.Ai.Algebra.AbstractRing`
  - equality reasoning helpers when needed
- Code or documentation areas:
  - `tools/proof-corpus/src/main.rs`
  - `proofs/Proofs/Ai/Algebra/AbstractField/` or a split calculation module if the foundation
    module becomes too large
  - `proofs/README.md`
  - package / manifest / generated proof-corpus metadata
- Tasks:
  - Add theorem targets:
    - `field_inv_one`
    - `field_div_one`
    - `field_div_self_nonzero`
    - `field_zero_div`
    - `field_mul_left_cancel_nonzero`
    - `field_mul_right_cancel_nonzero`
    - `field_nonzero_mul_closed`
    - `field_mul_eq_zero_cases`
  - Reuse `RingLawArgs` projections from FT-01 instead of duplicating ring assumptions.
  - Keep `Nonzero` proofs Prop-level and avoid decidable equality requirements.
- Deliverables:
  - Certificate-backed calculation lemmas available to theorem search.
  - Updated README theorem table or module description.
- Acceptance criteria:
  - Cancellation lemmas require explicit `Nonzero` evidence for the cancelled factor.
  - `field_div_self_nonzero` does not state an unconditional `a / a = 1`.
  - `field_mul_eq_zero_cases` exposes a proposition-level case split compatible with existing
    Church-encoded proposition style.
  - The module remains source-free verifiable.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractField`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractField`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "field_div_self_nonzero|field_mul_left_cancel_nonzero|field_mul_eq_zero_cases" proofs tools`
- Notes:
  - If the foundation module becomes hard to review, split calculation lemmas into
    `Proofs.Ai.Algebra.AbstractFieldBasic` and make the dependency explicit in package metadata.
  - If the split module is chosen, run `--build-module` and `--module` for
    `Proofs.Ai.Algebra.AbstractFieldBasic` and still run `--module Proofs.Ai.Algebra.AbstractField`
    to verify the foundation dependency.
  - Completed in `Proofs.Ai.Algebra.AbstractField` by extending `FieldLawArgs` with the basic
    calculation / cancellation / zero-product laws and projecting them as theorem targets.
  - The split module was not needed; the foundation module remains source-free verifiable.

### FT-03 Add Field Homomorphism Bridge

- Status: Completed
- Depends on: FT-02
- Inputs:
  - `Proofs.Ai.Algebra.AbstractField`
  - `Proofs.Ai.Algebra.AbstractRingFirstIsoBase`
  - `RingHomLawArgs`
- Code or documentation areas:
  - `tools/proof-corpus/src/main.rs`
  - `proofs/Proofs/Ai/Algebra/AbstractFieldHom/`
  - `proofs/README.md`
  - package / manifest / generated proof-corpus metadata
- Tasks:
  - Add `FieldHomLawArgs`.
  - Add bridge theorem targets:
    - `field_hom_as_ring_hom`
    - `field_hom_inv_of_nonzero`
    - `field_hom_div`
    - `field_hom_preserves_nonzero`
  - Verify that field hom reuse does not duplicate ring hom laws already owned by
    `AbstractRingFirstIsoBase`.
- Deliverables:
  - A separate `AbstractFieldHom` module with source, replay, meta, and certificate artifacts.
  - README entry explaining the relationship to `RingHomLawArgs`.
- Acceptance criteria:
  - `field_hom_as_ring_hom` is the only route needed by downstream ring first-isomorphism code to
    consume a field hom as a ring hom.
  - Inverse and division preservation require explicit nonzero-side hypotheses where needed.
  - The module does not introduce quotient or image constructions beyond importing the ring hom
    bridge that already exists.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractFieldHom`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractFieldHom`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "FieldHomLawArgs|field_hom_as_ring_hom|field_hom_div" proofs tools`
- Notes:
  - This milestone prepares for field isomorphism and embedding APIs, but does not need to define
    those APIs yet.
  - Completed in `Proofs.Ai.Algebra.AbstractFieldHom` with `FieldHomLawArgs`,
    `field_hom_as_ring_hom`, `field_hom_inv_of_nonzero`, `field_hom_div`, and
    `field_hom_preserves_nonzero`.
  - `FieldHomLawArgs` stores a single `RingHomLawArgs` witness from
    `AbstractRingFirstIsoBase`, so zero / one / addition / negation / multiplication preservation
    laws are not duplicated in the field-hom package.
  - The verified import closure includes the existing ring first-isomorphism base dependencies
    needed by `RingHomLawArgs`; the new module itself adds no quotient or image construction
    declarations.
  - Inverse and division preservation projections require explicit source-side `Nonzero`
    hypotheses.

### FT-04 Add Field To Integral Domain Bridge

- Status: Completed
- Depends on: FT-02
- Inputs:
  - `Proofs.Ai.Algebra.AbstractField`
  - `Proofs.Ai.Algebra.AbstractUfdPrimeFactorization`
  - existing UFD-local `IntegralDomainLawArgs`
- Code or documentation areas:
  - `tools/proof-corpus/src/main.rs`
  - `proofs/Proofs/Ai/Algebra/AbstractFieldIntegralDomain/`
  - `proofs/README.md`
  - package / manifest / generated proof-corpus metadata
- Tasks:
  - Add theorem targets:
    - `field_no_zero_divisors`
    - `field_integral_domain_laws`
    - `field_nonzero_product_left`
    - `field_nonzero_product_right`
    - `field_mul_eq_zero_elim`
  - Bridge the field-level `Nonzero` API to the existing integral-domain package shape.
  - Avoid making UFD factorization a dependency of `AbstractField`.
- Deliverables:
  - A source-free verified module that exports integral-domain evidence from field evidence.
  - README notes explaining how this bridge relates to `AbstractUfdPrimeFactorization`.
- Acceptance criteria:
  - `AbstractFieldIntegralDomain` imports `AbstractField` and the existing integral-domain API,
    not the other way around.
  - `field_integral_domain_laws` can be used by UFD-style modules without restating field inverse
    laws.
  - Zero-divisor elimination remains proposition-level and does not require decidable equality.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractFieldIntegralDomain`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractFieldIntegralDomain`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "field_integral_domain_laws|field_no_zero_divisors|IntegralDomainLawArgs" proofs tools`
- Notes:
  - If the existing UFD-local `IntegralDomainLawArgs` is too specialized, document the mismatch
    and introduce a narrow adapter instead of changing UFD theorem statements broadly.
  - Completed in `Proofs.Ai.Algebra.AbstractFieldIntegralDomain` with
    `field_no_zero_divisors`, `field_integral_domain_laws`, `field_nonzero_product_left`,
    `field_nonzero_product_right`, and `field_mul_eq_zero_elim`.
  - The bridge imports `AbstractField` and `AbstractUfdPrimeFactorization`; `AbstractField` does
    not import the UFD layer.
  - UFD's local nonzero predicate is named `UfdNonzero`, so the field `Nonzero` API and the
    integral-domain bridge can be imported together without declaration-name collision.
  - The module's axiom policy is the existing package-allowed `Eq.rec`, required only through
    `EqReasoning` equality transport in the nonzero-product factor lemmas.

### FT-05 Add Field Ideal And Quotient Bridge

- Status: Completed
- Depends on: FT-03, FT-04
- Inputs:
  - `Proofs.Ai.Algebra.AbstractField`
  - `Proofs.Ai.Algebra.AbstractRingFirstIso`
  - `Proofs.Ai.Algebra.AbstractRingChineseRemainder`
  - `Proofs.Ai.Algebra.AbstractKrullTheorem`
  - `Proofs.Ai.Algebra.AbstractHilbertNullstellensatz`
- Code or documentation areas:
  - `tools/proof-corpus/src/main.rs`
  - `proofs/Proofs/Ai/Algebra/AbstractFieldIdeal/`
  - `proofs/README.md`
  - package / manifest / generated proof-corpus metadata
- Tasks:
  - Add theorem targets:
    - `field_ideal_zero_or_top`
    - `field_simple_ring_evidence`
    - `quotient_by_maximal_ideal_is_field`
  - Keep ideal membership and maximality evidence explicit, matching the existing abstract Krull
    and Nullstellensatz style.
  - Avoid adding set-theoretic chain, Zorn, or concrete polynomial syntax machinery.
- Deliverables:
  - A verified bridge module connecting field evidence to ideal / quotient ring routes.
  - Documentation explaining which construction evidence remains explicit and untrusted.
- Acceptance criteria:
  - `quotient_by_maximal_ideal_is_field` is stated over explicit quotient / maximality evidence,
    not hidden global axioms.
  - The module does not alter the existing trusted boundary of Krull or Nullstellensatz packages.
  - Public theorem names do not conflict with existing ring CRT / first-isomorphism names.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractFieldIdeal`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractFieldIdeal`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "field_ideal_zero_or_top|quotient_by_maximal_ideal_is_field|MaximalIdeal" proofs tools`
- Notes:
  - This milestone is the first one allowed to depend on heavier ring quotient and ideal modules.
  - Completed in `Proofs.Ai.Algebra.AbstractFieldIdeal` with `FieldIdealZeroOrTop`,
    `FieldSimpleRingEvidence`, `FieldIdealLawArgs`, `MaximalIdealQuotientFieldArgs`, and theorem
    projections for the three target names.
  - `quotient_by_maximal_ideal_is_field` is intentionally stated over explicit `MaximalIdeal`,
    quotient ring laws, quotient hom, kernel exactness, and `RingFirstIso` evidence; no hidden
    quotient or maximality axiom was added.
  - `AbstractHilbertNullstellensatz` now uses the local name `HnsProperIdeal` so it can be imported
    together with Krull's `ProperIdeal` without public-name conflicts.
  - The module's axiom policy is the existing package-allowed `Eq.rec`, inherited through the
    equality/quotient import route.

### FT-06 Add OrderedField Compatibility Bridge

- Status: Completed
- Depends on: FT-02
- Inputs:
  - `Proofs.Ai.Algebra.AbstractField`
  - `Proofs.Ai.Algebra.AbstractOrderedField`
  - downstream geometry / inner-product modules using `OrderedFieldLawArgs`
- Code or documentation areas:
  - `tools/proof-corpus/src/main.rs`
  - `proofs/Proofs/Ai/Algebra/AbstractOrderedField/` or a split bridge module
  - `proofs/README.md`
  - package / manifest / generated proof-corpus metadata
- Tasks:
  - Add bridge theorem targets:
    - `ordered_field_field_laws`
    - `ordered_field_nonzero_of_positive`
    - `ordered_field_inv_positive`
    - `ordered_field_div_positive`
    - `ordered_field_mul_pos`
    - `ordered_field_sq_pos_of_nonzero`
  - Preserve existing `OrderedFieldLawArgs` consumers while adding field connectivity.
  - Verify that Pythagorean / metric / inner-product routes remain source-free valid.
- Deliverables:
  - Compatibility theorem artifacts that connect ordered-field law packages to field API.
  - Documentation clarifying whether the bridge lives inside `AbstractOrderedField` or a split
    module.
- Acceptance criteria:
  - Existing `OrderedFieldLawArgs` theorem names remain available.
  - Existing geometry and analysis modules are not forced to adopt `FieldLawArgs` in the same
    milestone.
  - Certificate / export hash changes are localized and any affected downstream modules are
    rebuilt deliberately.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractOrderedFieldFieldBridge`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractOrderedFieldFieldBridge`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractOrderedField`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Geometry.Pythagorean`
  - `rg -n "ordered_field_field_laws|OrderedFieldLawArgs|FieldLawArgs" proofs tools`
- Notes:
  - If adding these theorems to `AbstractOrderedField` changes too much downstream metadata,
    prefer a split `Proofs.Ai.Algebra.AbstractOrderedFieldFieldBridge` module.
  - If the split module is chosen, substitute
    `Proofs.Ai.Algebra.AbstractOrderedFieldFieldBridge` for the first two verification commands and
    still run `--module Proofs.Ai.Algebra.AbstractOrderedField` to verify the existing base module.
  - Completed as the split module `Proofs.Ai.Algebra.AbstractOrderedFieldFieldBridge` so existing
    `OrderedFieldLawArgs` consumers keep their imports and theorem names unchanged.
  - Added `OrderedFieldFieldBridgeArgs` as explicit bridge evidence containing `FieldLawArgs` and
    the positive/nonzero, inverse-positive, division-positive, multiplication-positive, and
    square-positive laws.
  - The bridge introduces no axioms and does not force geometry, metric, or inner-product modules
    to adopt `FieldLawArgs` in this milestone.

### FT-07 Final Corpus Gate And Documentation Pass

- Status: Completed
- Depends on: FT-01, FT-02, FT-03, FT-04, FT-05, FT-06
- Inputs:
  - all field-theory modules added by FT-01 through FT-06
  - `proofs/README.md`
  - `proofs/npa-package.toml`
  - `proofs/generated/ai-theorem-index.json`
  - `develop/proof-corpus-field-theory-roadmap.md`
  - this task document
- Code or documentation areas:
  - `proofs/**`
  - `tools/proof-corpus/**`
  - `develop/proof-corpus-field-theory-roadmap.md`
  - `develop/proof-corpus-field-theory-roadmap-todo.md`
- Tasks:
  - Run the appropriate proof-corpus gate after all field modules are in place.
  - Refresh documentation so module descriptions, command examples, and trusted-boundary notes
    agree.
  - Confirm the AI theorem index and package metadata include the new field modules in deterministic
    order.
  - Record that public `npa-mathlib` materialization is deferred to a separate closure audit.
- Deliverables:
  - A reviewed field-theory corpus route with generated artifacts checked in.
  - Updated roadmap / README notes reflecting completed milestones.
  - A follow-up closure-audit recommendation if public materialization is ready.
- Acceptance criteria:
  - Full relevant corpus verification passes after all certificate-affecting changes.
  - Axiom report does not grow unexpectedly.
  - The roadmap and todo document no longer describe completed work as unimplemented work.
  - Any public `npa-mathlib` materialization is explicitly deferred to a separate closure audit.
- Verification:
  - `./scripts/check-corpus.sh`
  - `cargo run -p npa-proof-corpus -- --write-ai-index`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "AbstractField|FieldLawArgs|FieldHomLawArgs|field_integral_domain_laws" proofs develop`
  - `git diff --check`
- Notes:
  - This is the only milestone that should normally run the full corpus gate for the field route.
    Earlier milestones should prefer `--build-module`, `--module`, and `--changed-only`.
  - Completed as the final field-theory corpus pass after FT-01 through FT-06.
  - `proofs/manifest.toml`, `proofs/npa-package.toml`, `proofs/generated/package-lock.json`, and
    `proofs/generated/ai-theorem-index.json` include the new field modules in deterministic corpus
    order: `AbstractField`, `AbstractFieldHom`, `AbstractFieldIntegralDomain`,
    `AbstractFieldIdeal`, and `AbstractOrderedFieldFieldBridge`.
  - Public `npa-mathlib` materialization is explicitly deferred to a separate closure audit. That
    audit should re-check import closure size, axiom policy, statement stability, and compatibility
    alias requirements before any promotion.

---

## Review Checklist

Before marking any milestone complete, check:

- Trusted base did not grow: no kernel I/O, network, plugin loading, AI call, or new core field
  primitive.
- Each new module has deterministic source, certificate, replay, meta, manifest, package, and index
  updates where applicable.
- New theorem statements use explicit carrier, operation, relation, and law arguments.
- Nonzero and negation evidence remain Prop-level and do not require decidable equality unless a
  later design document explicitly introduces it.
- Verification commands check the module being changed, not only unrelated fast tests.
- Public closure ordering remains scoped to `develop/npa-mathlib-next-closure-roadmap.md`.
