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
- field hom kernel / image / embedding layer
- polynomial quotient over field bridge
- field extension law package
- algebraic element / minimal polynomial layer
- finite extension layer
- finite field / Frobenius layer
- splitting field / algebraic closure evidence layers
- Galois theory starter
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

existing abstract polynomial / Hilbert / Nullstellensatz style modules
  use explicit polynomial-extension and algebraically-closed-field evidence packages, but do not
  replace the reusable field-extension, minimal-polynomial, or algebraic-closure layers planned here

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
- Advanced field-theory milestones may reuse existing abstract polynomial and algebraically-closed
  evidence styles, but should not claim concrete polynomial syntax, evaluator, or algebraic closure
  construction is trusted infrastructure unless a later design document adds it explicitly.

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
  - `./scripts/check-corpus-full.sh`
  - `cargo run -p npa-proof-corpus -- --write-ai-index`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "AbstractField|FieldLawArgs|FieldHomLawArgs|field_integral_domain_laws" proofs develop`
  - `git diff --check`
- Notes:
  - This is the only milestone that should normally run the package/full corpus gate for the field route.
    Earlier milestones should prefer `--build-module`, `--module`, and `--changed-only`.
  - Completed as the final field-theory corpus pass after FT-01 through FT-06.
  - `proofs/manifest.toml`, `proofs/npa-package.toml`, `proofs/generated/package-lock.json`, and
    `proofs/generated/ai-theorem-index.json` include the new field modules in deterministic corpus
    order: `AbstractField`, `AbstractFieldHom`, `AbstractFieldIntegralDomain`,
    `AbstractFieldIdeal`, and `AbstractOrderedFieldFieldBridge`.
  - Public `npa-mathlib` materialization is explicitly deferred to a separate closure audit. That
    audit should re-check import closure size, axiom policy, statement stability, and compatibility
    alias requirements before any promotion.

### FT-08 Add Field Hom Kernel / Image / Embedding Layer

- Status: Completed
- Depends on: FT-03
- Inputs:
  - `Proofs.Ai.Algebra.AbstractField`
  - `Proofs.Ai.Algebra.AbstractFieldHom`
  - `Proofs.Ai.Algebra.AbstractRingFirstIsoBase`
- Code or documentation areas:
  - `tools/proof-corpus/src/main.rs`
  - `proofs/Proofs/Ai/Algebra/AbstractFieldHomKernelImage/`
  - `proofs/README.md`
  - package / manifest / generated proof-corpus metadata
- Tasks:
  - Add an explicit field hom kernel / image / embedding evidence layer.
  - Add theorem targets:
    - `field_hom_kernel_zero_of_nonzero`
    - `field_hom_injective_of_nonzero`
    - `field_hom_image_field_laws`
    - `field_embedding_as_field_hom`
    - `field_embedding_comp`
    - `field_iso_symm`
    - `field_iso_trans`
  - Reuse `FieldHomLawArgs` and `RingHomLawArgs` instead of duplicating homomorphism laws.
- Deliverables:
  - A source-free verified `AbstractFieldHomKernelImage` module.
  - README notes explaining how this layer strengthens the promotion case for `AbstractFieldHom`.
- Acceptance criteria:
  - Kernel / image / injectivity statements keep all construction evidence explicit.
  - The module does not introduce concrete quotient construction machinery beyond existing imported
    ring hom APIs.
  - `AbstractFieldHom` gains at least one direct downstream module in package metadata.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractFieldHomKernelImage`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractFieldHomKernelImage`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "field_hom_injective_of_nonzero|field_hom_image_field_laws|field_embedding_comp" proofs tools`
- Notes:
  - This is the preferred next implementation milestone because it is small and creates immediate
    downstream evidence for the existing field hom layer.
  - Completed with `Proofs.Ai.Algebra.AbstractFieldHomKernelImage`, which adds
    `FieldHomKernelImageArgs`, `FieldHomImageFieldArgs`, `FieldEmbeddingLawArgs`,
    `FieldIsoLawArgs`, and the seven planned theorem targets. Construction-heavy kernel, image,
    embedding composition, and isomorphism transitivity facts remain explicit evidence rather than
    trusted infrastructure.

### FT-09 Add Polynomial Quotient Over Field Bridge

- Status: Completed
- Depends on: FT-05, FT-08
- Inputs:
  - `Proofs.Ai.Algebra.AbstractField`
  - `Proofs.Ai.Algebra.AbstractFieldIdeal`
  - existing abstract polynomial / Hilbert / Nullstellensatz style modules
- Code or documentation areas:
  - `tools/proof-corpus/src/main.rs`
  - `proofs/Proofs/Ai/Algebra/AbstractPolynomialFieldQuotient/`
  - `proofs/README.md`
  - package / manifest / generated proof-corpus metadata
- Tasks:
  - Add evidence packages such as `PolynomialFieldQuotientArgs`, `IrreduciblePolynomial`, and
    `PrincipalIdealGeneratedBy` if not already available in reusable form.
  - Add theorem targets:
    - `irreducible_polynomial_generates_maximal_ideal`
    - `quotient_by_irreducible_polynomial_is_field`
    - `polynomial_eval_kernel_contains_minimal_polynomial`
    - `simple_algebraic_extension_as_polynomial_quotient`
  - Keep concrete polynomial syntax and evaluation machinery outside the trusted base.
- Deliverables:
  - A verified bridge showing that quotient by an irreducible polynomial can be treated as a field
    from explicit evidence.
  - Documentation of which polynomial and quotient facts are assumed as evidence packages.
- Acceptance criteria:
  - The module does not force all of `AbstractFieldIdeal` into a future public closure unless the
    promotion audit explicitly accepts that import closure.
  - Irreducibility, principal ideal generation, quotient ring laws, and kernel exactness are explicit
    hypotheses.
  - The theorem names line up with later field extension / minimal polynomial milestones.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractPolynomialFieldQuotient`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractPolynomialFieldQuotient`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "quotient_by_irreducible_polynomial_is_field|PolynomialFieldQuotientArgs" proofs tools`
- Notes:
  - If the import closure is too large, split a narrow `AbstractMaximalIdealQuotientField` adapter
    before adding the polynomial-facing theorem names.
  - Completed with `Proofs.Ai.Algebra.AbstractPolynomialFieldQuotient`, which adds
    `IrreduciblePolynomial`, `PrincipalIdealGeneratedBy`, `PolynomialFieldQuotientArgs`, and
    `SimpleAlgebraicExtensionQuotientArgs`.
  - The bridge keeps irreducibility, principal ideal generation, quotient ring laws, quotient hom
    evidence, evaluation-kernel exactness, and simple-extension comparison explicit. It imports the
    current `AbstractFieldIdeal` route for staging, so promotion should audit this closure before
    making the polynomial quotient layer public.

### FT-10 Add Field Extension Law Package

- Status: Completed
- Depends on: FT-08
- Inputs:
  - `Proofs.Ai.Algebra.AbstractField`
  - `Proofs.Ai.Algebra.AbstractFieldHom`
  - `Proofs.Ai.Algebra.AbstractFieldHomKernelImage`
- Code or documentation areas:
  - `tools/proof-corpus/src/main.rs`
  - `proofs/Proofs/Ai/Algebra/AbstractFieldExtension/`
  - `proofs/README.md`
  - package / manifest / generated proof-corpus metadata
- Tasks:
  - Add `FieldExtensionLawArgs` for base field `K`, extension field `L`, and an embedding `K -> L`.
  - Add theorem targets:
    - `field_extension_base_embedding`
    - `field_extension_as_field`
    - `field_extension_restrict_scalars`
    - `field_extension_tower`
    - `field_embedding_compose`
  - Keep tower composition and scalar restriction as explicit projection theorems.
- Deliverables:
  - A verified field extension law package suitable for algebraic, finite, and splitting field layers.
  - README section documenting the explicit evidence model.
- Acceptance criteria:
  - `FieldExtensionLawArgs` reuses field hom / embedding evidence instead of restating all field laws.
  - The module does not depend on polynomial quotient, finite-dimensional vector spaces, or Galois
    machinery.
  - Downstream algebraic / finite extension milestones can import this module without circularity.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractFieldExtension`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractFieldExtension`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "FieldExtensionLawArgs|field_extension_tower|field_embedding_compose" proofs tools`
- Notes:
  - Module and theorem names are likely to become public later, so keep statements conservative.
  - Completed with `Proofs.Ai.Algebra.AbstractFieldExtension`, adding
    `FieldExtensionLawArgs`, `FieldExtensionRestrictScalarsArgs`, and
    `FieldExtensionTowerArgs`.
  - The theorem targets remain projection-style evidence theorems:
    `field_extension_base_embedding`, `field_extension_as_field`,
    `field_extension_restrict_scalars`, `field_extension_tower`, and
    `field_embedding_compose`.
  - The module reuses `FieldEmbeddingLawArgs` from `AbstractFieldHomKernelImage` and does not
    import polynomial quotient, finite-dimensional vector-space, or Galois machinery.

### FT-11 Add Algebraic Element And Minimal Polynomial Layer

- Status: Completed
- Depends on: FT-09, FT-10
- Inputs:
  - `Proofs.Ai.Algebra.AbstractFieldExtension`
  - `Proofs.Ai.Algebra.AbstractPolynomialFieldQuotient`
- Code or documentation areas:
  - `tools/proof-corpus/src/main.rs`
  - `proofs/Proofs/Ai/Algebra/AbstractAlgebraicExtension/`
  - `proofs/README.md`
  - package / manifest / generated proof-corpus metadata
- Tasks:
  - Add evidence packages:
    - `AlgebraicElement`
    - `MinimalPolynomial`
  - Add theorem targets:
    - `minimal_polynomial_divides_annihilating_polynomial`
    - `minimal_polynomial_irreducible`
    - `degree_one_algebraic_element_in_base`
    - `field_adjoin_algebraic_element_is_finite_extension`
  - Connect minimal-polynomial evidence with polynomial quotient evidence.
- Deliverables:
  - A verified algebraic-extension bridge with explicit minimal polynomial evidence.
  - Documentation of monic / irreducible / uniqueness assumptions.
- Acceptance criteria:
  - Minimal polynomial uniqueness and monicity are explicit evidence fields or explicit theorem
    hypotheses.
  - No global algebraic-closure existence axiom is introduced.
  - The module remains usable by finite extension and splitting field milestones.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractAlgebraicExtension`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractAlgebraicExtension`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "AlgebraicElement|MinimalPolynomial|minimal_polynomial_irreducible" proofs tools`
- Notes:
  - If statement stability is weak, keep this layer in corpus staging and do not promote until at
    least one finite-extension consumer exists.
  - Completed with `Proofs.Ai.Algebra.AbstractAlgebraicExtension`, adding explicit
    `AlgebraicElement`, `MinimalPolynomial`, and `FieldAdjoinAlgebraicElementArgs` packages.
  - Monicity, irreducibility, minimal-polynomial uniqueness, degree-one base membership, and
    finite-extension output remain explicit evidence fields or theorem hypotheses.
  - The bridge connects `MinimalPolynomial` to `SimpleAlgebraicExtensionQuotientArgs` without
    adding any algebraic-closure existence axiom.

### FT-12 Add Finite Extension Layer

- Status: Completed
- Depends on: FT-10, FT-11
- Inputs:
  - `Proofs.Ai.Algebra.AbstractFieldExtension`
  - `Proofs.Ai.Algebra.AbstractAlgebraicExtension`
  - existing vector / linear algebra law packages when needed
- Code or documentation areas:
  - `tools/proof-corpus/src/main.rs`
  - `proofs/Proofs/Ai/Algebra/AbstractFiniteFieldExtension/`
  - `proofs/README.md`
  - package / manifest / generated proof-corpus metadata
- Tasks:
  - Add `FiniteExtensionLawArgs`.
  - Add theorem targets:
    - `finite_extension_is_algebraic`
    - `extension_degree_tower`
    - `finite_dimensional_vector_space_bridge`
    - `finite_extension_embedding_preserves_degree`
  - Decide whether degree is represented by Nat data or by Prop-level degree evidence for this
    corpus layer.
- Deliverables:
  - A verified finite-extension module that can feed finite fields and Galois theory.
  - Documentation of degree evidence and vector-space dependency choices.
- Acceptance criteria:
  - The finite extension layer depends on field extension / algebraic extension, not conversely.
  - If vector-space imports make the closure large, degree facts are kept as explicit evidence.
  - Tower law statement is stable enough to be reused by later modules.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractFiniteFieldExtension`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractFiniteFieldExtension`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "FiniteExtensionLawArgs|extension_degree_tower|finite_extension_is_algebraic" proofs tools`
- Notes:
  - This layer should not be promoted until the dependency cost of vector-space bridge imports is
    measured.
  - Completed with `Proofs.Ai.Algebra.AbstractFiniteFieldExtension`, adding
    `FiniteExtensionLawArgs`, `FiniteExtensionTowerDegreeArgs`, and
    `FiniteExtensionEmbeddingDegreeArgs`.
  - Degree is represented by Prop-level `ExtensionDegreeEvidence` in this corpus layer rather than
    concrete Nat data.
  - Finite-dimensional vector-space structure is kept behind
    `FiniteDimensionalVectorSpaceBridge` to avoid importing a large vector-space basis API before
    downstream finite-field and Galois-theory statements stabilize.
  - The finite-implies-algebraic, tower-degree, vector-space bridge, and embedding-degree theorem
    targets are projection theorems over explicit evidence packages; no algebraic-closure or
    basis-existence axiom is introduced.

### FT-13 Add Finite Field And Frobenius Layer

- Status: Completed
- Depends on: FT-10, FT-12
- Inputs:
  - `Proofs.Ai.Algebra.AbstractField`
  - `Proofs.Ai.Algebra.AbstractFieldHom`
  - `Proofs.Ai.Algebra.AbstractFiniteFieldExtension`
- Code or documentation areas:
  - `tools/proof-corpus/src/main.rs`
  - `proofs/Proofs/Ai/Algebra/AbstractFiniteField/`
  - `proofs/README.md`
  - package / manifest / generated proof-corpus metadata
- Tasks:
  - Add `FiniteFieldLawArgs`.
  - Add theorem targets:
    - `field_characteristic_prime_or_zero`
    - `finite_field_characteristic_prime`
    - `frobenius_is_field_hom`
    - `finite_field_pow_card_eq_self`
    - `finite_field_roots_x_pow_q_minus_x`
  - Start with Frobenius and characteristic facts before adding root-counting facts if imports get
    too large.
- Deliverables:
  - A verified finite-field staging module.
  - README notes separating cardinality / power / polynomial-root evidence from trusted proof
    certificates.
- Acceptance criteria:
  - Characteristic, cardinality, power, and root APIs are explicit and do not require hidden runtime
    computation.
  - Frobenius uses the existing field hom route where possible.
  - Heavy root-counting results may be split into a later module if needed.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractFiniteField`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractFiniteField`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "FiniteFieldLawArgs|frobenius_is_field_hom|finite_field_pow_card_eq_self" proofs tools`
- Notes:
  - This is a high-value future `npa-mathlib` candidate, but only after cardinality and polynomial
    APIs stabilize.
  - Completed with `Proofs.Ai.Algebra.AbstractFiniteField`, adding `FiniteFieldLawArgs` and
    characteristic, Frobenius, power-cardinality, and root-predicate projection theorem targets.
  - Frobenius is represented through the existing `FieldHomLawArgs` route.
  - Cardinality, power, and roots of `x^q - x` remain explicit evidence fields; no hidden finite
    enumeration, cardinality computation, or root-counting axiom is introduced.
  - The module imports `AbstractFiniteFieldExtension` so later finite fields can connect back to
    finite-extension degree evidence, but keeps concrete polynomial-root APIs staged until those
    statements stabilize.

### FT-14 Add Splitting Field And Algebraic Closure Evidence Layers

- Status: Completed
- Depends on: FT-11, FT-12
- Inputs:
  - `Proofs.Ai.Algebra.AbstractAlgebraicExtension`
  - `Proofs.Ai.Algebra.AbstractFiniteFieldExtension`
  - polynomial quotient / root evidence packages
- Code or documentation areas:
  - `tools/proof-corpus/src/main.rs`
  - `proofs/Proofs/Ai/Algebra/AbstractSplittingField/`
  - `proofs/Proofs/Ai/Algebra/AbstractAlgebraicClosure/`
  - `proofs/README.md`
  - package / manifest / generated proof-corpus metadata
- Tasks:
  - Add evidence packages:
    - `SplittingFieldLawArgs`
    - `AlgebraicClosureLawArgs`
  - Add theorem targets:
    - `splitting_field_contains_all_roots`
    - `splitting_field_generated_by_roots`
    - `splitting_field_unique_up_to_field_iso`
    - `algebraic_closure_is_algebraic`
    - `algebraic_closure_polynomial_has_root`
  - Keep existence as explicit evidence, not as a hidden axiom.
- Deliverables:
  - Verified staging modules for splitting fields and algebraic closure evidence.
  - Documentation of which existence claims are assumed as construction evidence.
- Acceptance criteria:
  - No algebraic-closure existence theorem is stated without explicit construction evidence.
  - Uniqueness is up to field isomorphism and reuses the field iso API.
  - Galois starter can import splitting field evidence without circular dependencies.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractSplittingField`
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractAlgebraicClosure`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractSplittingField`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractAlgebraicClosure`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "SplittingFieldLawArgs|AlgebraicClosureLawArgs|splitting_field_unique" proofs tools`
- Notes:
  - Split the two modules if algebraic closure statements remain unstable.
  - Completed with `Proofs.Ai.Algebra.AbstractSplittingField` and
    `Proofs.Ai.Algebra.AbstractAlgebraicClosure`.
  - Splitting-field construction, all-roots containment, generation by roots, and uniqueness up to
    field isomorphism are explicit evidence fields or `FieldIsoLawArgs` witnesses.
  - Algebraic-closure construction, element algebraicity, and polynomial root existence are explicit
    evidence fields; no algebraic-closure existence axiom or hidden root-finding procedure is added.
  - The algebraic-closure `HasRoot` predicate remains abstract so downstream modules can encode
    nonconstant or positive-degree side conditions before the polynomial API stabilizes.

### FT-15 Add Galois Theory Starter

- Status: Completed
- Depends on: FT-10, FT-12, FT-14
- Inputs:
  - `Proofs.Ai.Algebra.AbstractFieldExtension`
  - `Proofs.Ai.Algebra.AbstractFiniteFieldExtension`
  - `Proofs.Ai.Algebra.AbstractSplittingField`
  - existing group correspondence modules
- Code or documentation areas:
  - `tools/proof-corpus/src/main.rs`
  - `proofs/Proofs/Ai/Algebra/AbstractGaloisStarter/`
  - `proofs/README.md`
  - package / manifest / generated proof-corpus metadata
- Tasks:
  - Add evidence packages:
    - `FieldAutomorphismGroupArgs`
    - `GaloisExtensionArgs`
  - Add theorem targets:
    - `automorphism_group_laws`
    - `fixed_field_laws`
    - `fixed_field_is_field`
    - `galois_correspondence_order_bridge`
  - Reuse existing group correspondence theorem targets rather than restating group theory.
- Deliverables:
  - A verified Galois starter module that connects field extensions to automorphism groups and
    fixed fields.
  - Documentation of import closure and why this layer remains corpus staging.
- Acceptance criteria:
  - The module does not pull Galois correspondence into public `npa-mathlib` without a separate
    closure audit.
  - Fixed-field and automorphism-group construction evidence remains explicit.
  - The dependency graph is acyclic and does not force earlier field-extension modules to import
    group correspondence.
- Verification:
  - `cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractGaloisStarter`
  - `cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractGaloisStarter`
  - `cargo run -p npa-proof-corpus -- --changed-only`
  - `rg -n "FieldAutomorphismGroupArgs|fixed_field_is_field|galois_correspondence_order_bridge" proofs tools`
- Notes:
  - Added `Proofs.Ai.Algebra.AbstractGaloisStarter` as a corpus staging layer above field
    extensions, finite extensions, splitting fields, and the existing group-correspondence modules.
  - The module keeps automorphism-group and fixed-field construction evidence explicit, and the
    correspondence theorem target is a bridge to existing group correspondence order evidence.
  - Expected axiom policy is `Eq.rec`, inherited from the existing group-correspondence closure.
  - This is not promoted to `npa-mathlib`; promotion still requires a separate closure audit,
    package hash/index/axiom report verification, and an alias decision.

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
