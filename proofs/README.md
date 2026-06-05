# AI Proof Corpus

Visibility: internal proof-corpus documentation.

This README documents the repository proof corpus and package-fixture
regression data. It is not the public package-author entry path. Public package
authors should start with `README.md`, `docs/README.md`,
`docs/npa-toolchain-reference-v0.1.1.md`, and
`docs/external-theorem-library-ci.md`.

This directory stores proof artifacts intended for AI-facing proof production and regression.
Artifact paths follow the module namespace. For example, module `Proofs.Ai.Basic` lives at
`Proofs/Ai/Basic/`.

The trust boundary follows the repository-wide certificate-first policy:

- `*.npa`, `*.replay.json`, and `*.meta.json` are non-trusted producer sidecars.
- `*.npcert` is the canonical artifact consumed by the certificate verifier and kernel.
- A proof is accepted only when the certificate decodes canonically and `verify_module_cert` succeeds.

Current bundles:

- `Proofs/Ai/Basic/`: small no-import, no-axiom combinator and implication theorem module.
- `Proofs/Ai/Eq/`: equality refl theorem module importing `Std.Logic.Eq` and `Std.Nat.Basic`.
- `Proofs/Ai/EqReasoning/`: equality reasoning module importing `Std.Logic.Eq` and using the
  expected builtin `Eq.rec` axiom interface.
- `Proofs/Ai/Analysis/AbstractMetricTopology/`: predicate-level metric ball, neighborhood, local
  membership, local predicate, local equality, and local uniqueness API for the inverse/implicit
  function route.
- `Proofs/Ai/Analysis/Real/Basic/`: abstract real-analysis foundation over an arbitrary scalar
  carrier, packaging ordered-field laws, field bridge laws, interval APIs, bound/supremum/infimum
  evidence, order completeness, and Archimedean evidence without adding a trusted `Real` primitive.
- `Proofs/Ai/Analysis/AbstractNormedSpace/`: explicit-law normed-space distance, product
  operation, product norm, projection/pairing, and product norm estimate API for the
  inverse/implicit function route.
- `Proofs/Ai/Analysis/AbstractLinearMap/`: bounded linear map, operator-norm bound, linear
  isomorphism, identity/composition/inverse, and block-triangular map API for the inverse/implicit
  function route.
- `Proofs/Ai/Analysis/AbstractDerivative/`: Frechet derivative, differentiability, derivative
  uniqueness, calculus-rule evidence, pairing/projection, composition, and partial-derivative API
  for the inverse/implicit function route.
- `Proofs/Ai/Analysis/AbstractFixedPoint/`: complete metric evidence, contraction/self-map laws,
  fixed-point evidence, uniqueness/stability projections, and Banach fixed-point result API for the
  inverse/implicit function route.
- `Proofs/Ai/Analysis/Sequence/Basic/`: abstract scalar sequence vocabulary over
  `Analysis.Real.Basic`, with convergence, eventuality, subsequence, boundedness, limit-uniqueness
  evidence, monotone convergence evidence, and bridges to `AbstractFixedPoint` `ConvergesTo` and
  `CauchySeq`.
- `Proofs/Ai/Analysis/AbstractInverseFunction/`: residual/Newton-map definitions, local inverse
  evidence/result packaging, uniqueness/differentiability projections, and quantitative inverse
  function theorem API for the implicit-function route.
- `Proofs/Ai/Analysis/AbstractImplicitPhi/`: auxiliary `Phi(x,y)=(x,F(x,y))` definitions,
  base-point equation, derivative law package, and block-triangular linear-isomorphism bridge for
  the implicit-function route.
- `Proofs/Ai/Analysis/AbstractImplicitFunction/`: extraction of the local implicit function from
  explicit local inverse laws for `Phi`, with value membership, zero equation, and local uniqueness
  projections plus the basic public implicit-function theorem evidence wrapper and the separate
  differentiability/derivative-formula evidence wrapper.
- `Proofs/Ai/Algebra/Ring/`: singleton-carrier algebra API and ring-law theorem targets importing
  `Std.Logic.Eq`.
- `Proofs/Ai/Algebra/Square/`: square API and square-expansion theorem targets importing
  `Std.Logic.Eq` and `Proofs.Ai.Algebra.Ring`.
- `Proofs/Ai/Nat/`: Nat smoke theorem module importing `Std.Logic.Eq` and `Std.Nat.Basic`.
- `Proofs/Ai/NumberTheory/Flt/Statement/`: FLT statement-freeze module importing
  `Std.Logic.Eq` and `Std.Nat.Basic`. It exports Prop-valued statement constants for
  `fermat_last_theorem`, natural-number, positive-natural, and integer variants. Current
  `Std.Nat.Basic` provides certified `Nat.zero` and `Nat.succ`; addition, exponentiation, and
  order are explicit statement parameters until the reusable number-theory library materializes
  them. This module has no `Flt.BridgeAxiom.*` dependency.
- `Proofs/Ai/OrderedField/`: order and square-root API theorem targets importing `Std.Logic.Eq`,
  `Proofs.Ai.Algebra.Ring`, and `Proofs.Ai.Algebra.Square`.
- `Proofs/Ai/Prop/`: import-free proposition-only implication search module.
- `Proofs/Ai/Reduction/`: reduction smoke theorem module importing `Std.Nat.Basic`.
- `Proofs/Ai/Vector/Basic/`: vector carrier and basic vector addition theorem targets importing
  `Std.Logic.Eq`.
- `Proofs/Ai/Vector/Dot/`: dot product, squared norm, and squared distance theorem targets
  importing vector, scalar, square, and order corpus layers.
- `Proofs/Ai/Vector/AbstractSpace/`: abstract vector-space theorem targets over the P17-P19
  scalar API layers and explicit vector operation/law assumptions.
- `Proofs/Ai/Vector/AbstractInnerProduct/`: abstract inner-product, squared norm, and vector
  squared-distance theorem targets over explicit scalar, vector, and inner-product law assumptions.
- `Proofs/Ai/Vector/AbstractInnerProductDerive/`: checked norm-expansion, parallelogram,
  polarization, Cauchy-Schwarz, and squared Minkowski derivations from explicit scalar, vector, and
  inner-product law packages.
- `Proofs/Ai/LinearAlgebra/AbstractSpectralTheorem/`: certificate-backed finite-dimensional
  spectral theorem package for normal matrices, exposing unitary diagonalization `A = U D U*` with
  finite-dimensional and complex spectral-field assumptions kept as explicit evidence.
- `Proofs/Ai/FunctionalAnalysis/AbstractHilbertSpaceSpectralTheorem/`: certificate-backed
  Hilbert-space spectral theorem package for bounded normal and self-adjoint operators, exposing
  projection-valued measure representations, multiplication-operator models, and direct-integral
  decompositions with analytic construction evidence kept explicit.
- `Proofs/Ai/Geometry/Affine/`: abstract point, displacement, and point squared-distance theorem
  targets over explicit affine compatibility law assumptions.
- `Proofs/Ai/Geometry/AffineDerive/`: checked affine displacement orientation and point-distance
  bridge derivations from primitive affine and vector law packages.
- `Proofs/Ai/Geometry/AbstractRightTriangle/`: abstract perpendicularity, right-triangle, and
  squared-distance Pythagorean theorem targets over explicit geometry law assumptions.
- `Proofs/Ai/Geometry/AbstractRightTriangleDerive/`: checked right-triangle-to-perpendicular
  bridge derivations for the abstract Pythagorean route.
- `Proofs/Ai/Geometry/AbstractMetric/`: abstract distance, metric law-package, ball API, checked
  distance/squared-distance bridges, squared Minkowski, and checked metric triangle inequality.
- `Proofs/Ai/Geometry/Pythagorean/`: final abstract Pythagorean and law-of-cosines theorem names,
  including checked squared-distance and squared metric-distance derivations from scalar, vector,
  inner-product, affine, right-triangle, and metric bridge law packages.
- `Proofs/Ai/Geometry/RightTriangle/`: right-triangle and squared-distance Pythagoras theorem
  targets importing vector dot and scalar corpus layers.
- `Proofs/Ai/Geometry/Metric/`: distance API and metric theorem targets importing the right-triangle
  and vector dot layers.
- `Proofs/Ai/Logic/Iff/`: first-class logical equivalence, conjunction, disjunction, falsehood, and
  negation theorem targets importing `Std.Logic.Eq`.
- `Proofs/Ai/Category/Classical/`: first-stage classical category-theory law package,
  category-definition introduction, functor-definition introduction, natural-transformation
  definition introduction, adjunction Hom natural-isomorphism law package, unit/counit
  triangle-identity law package, law projections, pointwise Hom functor theorem,
  a pointwise Yoneda lemma, pointwise Yoneda embedding laws,
  sieve, Grothendieck-topology, matching-family, sheaf-condition, and sheafification
  law packages with checked axiom projections, universal-property witnesses, and a
  left-adjoint witness, subobject-classifier law package and classifier witnesses,
  elementary-topos law package with finite-limit, cartesian-closed, and classifier
  projections, Kripke-Joyal forcing semantics over sites with stability, locality,
  propositional connective, and local-disjunction clauses, Giraud axiom and
  Grothendieck-topos representation packages with a checked Giraud theorem witness,
  limit and colimit existence/universal-property packages, adjoint preservation of
  limits and colimits, Freyd-style universal-arrow construction of a left adjoint,
  checked presheaf-category completeness and cocompleteness from pointwise
  construction certificates, and the checked opposite-category law construction.
- `Proofs/Ai/Category/Infinity/SimplicialSet/`: third-stage infinity-category
  entry point for simplicial sets, with the simplex-category law package,
  simplicial-set restriction laws, and a checked presheaf-on-the-simplex-category
  witness, plus a Kan-complex horn-filler law package and checked Kan-complex
  filler projection, a quasicategory inner-horn-filler law package and checked
  quasicategory projection, a homotopy-category law package projecting the
  category laws associated to a quasicategory, a mapping-space law package
  projecting Kan-complex mapping spaces, a Joyal-model-structure law package
  projecting model-structure, cofibration, fibrant-object, and weak-equivalence
  characterization laws, Cartesian and coCartesian fibration law packages with
  simplicial-map, inner-fibration, lift-existence, and lift-stability projections,
  a straightening-unstraightening law package with Cartesian/coCartesian
  functoriality and unit/counit projections, and a nerve-construction law package
  projecting the nerve of a category as a simplicial set.
- `Proofs/Ai/Algebra/AbstractGroup/`: abstract group and homomorphism law packages, group-law
  projections, cancellation, double-inverse, product-reassociation, and reverse-inverse lemmas,
  normal-relation reassociation lemmas, kernel predicate, kernel relation, and checked
  kernel-relation equivalence ingredients for the first-isomorphism route.
- `Proofs/Ai/Algebra/AbstractGroupKernel/`: checked kernel closure under multiplication, inverse,
  and conjugation for the first-isomorphism route.
- `Proofs/Ai/Algebra/AbstractGroupImage/`: Church-encoded image membership, image introduction and
  elimination, and checked image closure under identity, multiplication, and inverse.
- `Proofs/Ai/Algebra/AbstractGroupQuotient/`: quotient-backed kernel relation setoid, quotient
  carrier, canonical representative map into `H`, and checked representative computation and
  multiplication compatibility lemmas.
- `Proofs/Ai/Algebra/AbstractGroupQuotientMul/`: representative multiplication into the kernel
  quotient and checked `KerRel` compatibility for changing both representatives.
- `Proofs/Ai/Algebra/AbstractGroupQuotientGroup/`: quotient-level multiplication, identity,
  inverse, representative computation, and checked quotient-level associativity, identity, and
  inverse laws.
- `Proofs/Ai/Algebra/AbstractGroupQuotientHom/`: quotient induction proof that the canonical map
  from the kernel quotient to `H` preserves quotient-level multiplication for arbitrary quotient
  elements.
- `Proofs/Ai/Algebra/AbstractGroupFirstIsoFull/`: quotient-to-image first-isomorphism facts for
  arbitrary quotient elements: homomorphism, injectivity, image membership, and surjectivity onto
  the Church-encoded image predicate.
- `Proofs/Ai/Algebra/AbstractGroupFirstIsoImage/`: final quotient-to-image bundle for the AI
  route, adding image closure facts and inductive evidence tokens that package the quotient group
  laws together with the canonical map as a homomorphic injection whose image is exactly the
  Church-encoded image predicate.
- `Proofs/Ai/Algebra/AbstractGroupFirstIso/`: representative-level first-isomorphism MVP bundling
  quotient representative computation, multiplication compatibility, representative injectivity,
  and image membership.
- `Proofs/Ai/Algebra/AbstractGroupSubgroup/`: subgroup and normal-subgroup law packages,
  normal conjugation helpers, intersection predicate closure facts, product-subgroup closure
  evidence, normal-relation compatibility facts, and `N h` / `h ~ 1` conversion helpers for the
  second-isomorphism route.
- `Proofs/Ai/Algebra/AbstractGroupSubgroupOrder/`: predicate-level inclusion and equivalence API
  for subgroup-style predicates, including reflexivity/transitivity lemmas and `NormalContains`
  conversions used by later correspondence-order milestones.
- `Proofs/Ai/Algebra/AbstractGroupNormalQuotient/`: quotient setoid, quotient carrier,
  representative injection, and soundness theorem for quotienting by an arbitrary normal subgroup
  predicate.
- `Proofs/Ai/Algebra/AbstractGroupNormalQuotientMul/`: representative multiplication and
  well-definedness for arbitrary normal quotients.
- `Proofs/Ai/Algebra/AbstractGroupNormalQuotientGroup/`: quotient multiplication, identity,
  inverse, and group laws for arbitrary normal quotients.
- `Proofs/Ai/Algebra/AbstractGroupSecondIsoPhi/`: the natural representative map from a
  subgroup predicate `H` to `G/N`, with representative, multiplication, identity, and inverse
  compatibility facts.
- `Proofs/Ai/Algebra/AbstractGroupSecondIsoKernel/`: representative-level kernel predicate for
  the natural map, quotient-identity soundness, and checked conversions between kernel membership
  and `H ∩ N`.
- `Proofs/Ai/Algebra/AbstractGroupSecondIsoImage/`: Church-encoded natural-map image and
  product-quotient predicates, with checked conversions between image membership and `HN / N`
  membership.
- `Proofs/Ai/Algebra/AbstractGroupSecondIsoFinal/`: final AI-facing second-isomorphism evidence
  bundle, packaging kernel identification with image/product-quotient identification for the
  natural map.
- `Proofs/Ai/Algebra/AbstractGroupThirdIso/`: AI-facing third-isomorphism route for normal
  subgroups `N <= H`, defining the canonical map `G/N -> G/H`, representative-level
  multiplication, identity, and inverse compatibility, surjectivity, decomposed `H/N` closure under
  identity, multiplication, inverse, and quotient conjugation, kernel soundness for the `H/N`
  predicate, law-bundle target aliases for `H/N`, and the `ThirdIsoPhi` kernel-relation quotient
  carrier alias.
- `Proofs/Ai/Algebra/AbstractGroupCorrespondence/`: AI-facing correspondence theorem route for a
  normal subgroup `N`, packaging the image map from subgroups `H` with `N <= H` to quotient
  subgroups of `G/N`, the preimage map from quotient subgroups back to subgroups of `G`, subgroup
  closure for both maps, containment of `N` in preimages, quotient-side round-trip membership
  conversions, and subgroup-side `NormalRel` saturation equivalence.
- `Proofs/Ai/Algebra/AbstractGroupCorrespondenceOrder/`: predicate-order monotonicity for the
  correspondence image and preimage maps, plus equivalence-respect lemmas for subgroup-style
  predicates on `G` and `G/N`.
- `Proofs/Ai/Algebra/AbstractGroupCorrespondenceFinal/`: final correspondence theorem evidence
  wrapper importing the correspondence route and exporting direct subgroup law-package
  constructors plus a certificate-backed theorem that collects the checked closure, containment,
  round-trip, and saturation components.
- `Proofs/Ai/Algebra/AbstractGroupCorrespondenceOrderFinal/`: final order-level correspondence
  wrapper packaging image/preimage monotonicity and final correspondence evidence for closure,
  containment, quotient round-trip, and subgroup-side saturation.
- `Proofs/Ai/Algebra/AbstractRing/`: abstract scalar ring theorem targets over explicit carrier,
  operation, and law assumptions importing `Std.Logic.Eq`.
- `Proofs/Ai/Algebra/AbstractField/`: abstract field foundation over `AbstractRing`, adding
  reusable nonzero, inverse-law, and derived-division theorem targets without making division a
  core primitive.
- `Proofs/Ai/Algebra/AbstractRingFirstIsoBase/`: ring homomorphism law package, additive-group
  bridge, image closure, and kernel quotient multiplication well-definedness for the ring first
  isomorphism route.
- `Proofs/Ai/Algebra/AbstractFieldHom/`: field homomorphism law package wrapping the existing
  `RingHomLawArgs` bridge and adding inverse, division, and nonzero-preservation projections.
- `Proofs/Ai/Algebra/AbstractFieldIntegralDomain/`: bridge from field law packages to the
  UFD-style `IntegralDomainLawArgs`, with no-zero-divisor and nonzero-product theorem targets.
- `Proofs/Ai/Algebra/AbstractRingFirstIso/`: certificate-backed ring first isomorphism theorem
  bundle for the canonical map from the kernel quotient to the Church-encoded image, preserving
  zero, one, addition, and multiplication with injectivity and image-surjectivity.
- `Proofs/Ai/Algebra/AbstractRingChineseRemainder/`: certificate-backed ring Chinese remainder
  theorem route for the map `R -> R/I x R/J`, identifying its kernel with the intersection
  predicate and using comaximal combine laws to show the image is the full product.
- `Proofs/Ai/Algebra/AbstractUfdPrimeFactorization/`: certificate-backed UFD prime
  factorization package over an abstract factorization carrier, deriving prime-factor existence
  and uniqueness from the UFD law package without adding unchecked kernel axioms.
- `Proofs/Ai/Algebra/AbstractHilbertBasisTheorem/`: certificate-backed Hilbert basis theorem
  package over abstract ideals, finite generating families, and a polynomial-extension leading
  coefficient construction, deriving Noetherianity of `R[X]` from Noetherianity of `R`.
- `Proofs/Ai/Algebra/AbstractHilbertNullstellensatz/`: certificate-backed Hilbert
  Nullstellensatz package over abstract affine points, polynomial evaluation, vanishing ideals,
  and radical membership evidence.
- `Proofs/Ai/Algebra/AbstractKrullTheorem/`: certificate-backed Krull maximal ideal theorem
  package: every proper ideal is contained in a maximal ideal, with Zorn-style existence kept as
  explicit construction evidence outside the trusted core.
- `Proofs/Ai/Algebra/AbstractFieldIdeal/`: field ideal and quotient bridge packaging
  field-simple-ring evidence and the maximal-ideal quotient-is-field theorem over explicit
  quotient, kernel, and first-isomorphism witnesses.
- `Proofs/Ai/Algebra/AbstractOrderedField/`: abstract scalar order and square-root theorem targets
  over explicit carrier, operation, relation, function, and law assumptions.
- `Proofs/Ai/Algebra/AbstractOrderedFieldFieldBridge/`: split compatibility bridge from
  `OrderedFieldLawArgs` to `FieldLawArgs`, keeping positive/nonzero/inverse/division laws as
  explicit evidence without changing existing ordered-field consumers.
- `Proofs/Ai/Algebra/AbstractSquareNormalize/`: abstract square-normalization theorem targets over
  the P17/P18 scalar APIs and explicit law assumptions.
- `Proofs/Ai/Algebra/AbstractScalarDerive/`: scalar rewrite derivations from `RingLawArgs` and
  equality transport, including the zero cross-term cancellation needed by the abstract
  Pythagorean route.
- `manifest.toml`: legacy `npa-ai-proof-corpus-v0.1` compatibility index for the corpus and
  expected hashes.
- `npa-package.toml`: `npa.package.v0.1` fixture for package-level tooling. CLR-03/CLR-04
  source-free checker import locks are derived from this fixture, not from
  `manifest.toml` or `tools/proof-corpus` Rust constants.
- `generated/package-lock.json`: CLR-03 `npa.package.lock.v0.1` fixture derived from
  `npa-package.toml` and checked-in certificate bytes. It is package metadata, not proof
  evidence; accepted proof evidence remains canonical `.npcert` bytes plus the selected checker
  verdict.
- `generated/axiom-report.json`: CLR-05 `npa.package.axiom_report.v0.1` fixture derived from
  package metadata, the package lock, certificate artifacts, and source-free verifier output. It is
  package metadata, not proof evidence, and it is distinct from
  `npa.independent-checker.axiom_report.v1` and Std-only axiom report schemas.
- `generated/theorem-index.json`: CLR-05 `npa.package.theorem_index.v0.1` fixture derived from
  checked certificates for theorem search, documentation, and later registry metadata. It is
  metadata, not proof evidence, and it is distinct from Std-only theorem index schemas.
- `generated/publish-plan.json`: CLR-06 `npa.package.publish_plan.v0.1` fixture derived from
  the manifest, package lock, axiom report, theorem index, certificate artifacts, and source-free
  checker summaries. It records release artifact hashes, `npa.registry.module.v0.1` theorem
  package module registry seed entries, downstream import bundle data, and checksum-only SHA-256
  signature policy. It is release metadata, not proof evidence, and it is distinct from
  `npa.independent-checker.checker_binary_registry.v1`.

Package fixture handoff data for CLR-03:

- Local `[[modules]]` entries provide module names, package-relative source and certificate paths,
  direct imports, `expected_source_hash`, `expected_certificate_file_hash`,
  `expected_export_hash`, `expected_axiom_report_hash`, and `expected_certificate_hash`.
- Top-level `[[imports]]` entries provide external Std module identity. The current fixture pins
  `Std.Logic.Eq` at `vendor/npa-std/Std/Logic/Eq/certificate.npcert` and `Std.Nat.Basic` at
  `vendor/npa-std/Std/Nat/Basic/certificate.npcert`, with exact `export_hash` and
  `certificate_hash` values. CLR-03 derives external import `axiom_report_hash` values from those
  pinned certificate artifacts when it builds the package lock.
- CLR-03/CLR-04 source-free verification reads certificates through these package-relative paths;
  source, replay, meta, theorem index, AI traces, and out-of-package state are not checker inputs.
- `proofs/generated/package-lock.json` records package graph identity, certificate paths,
  certificate file hashes, export hashes, certificate hashes, direct imports, and axiom report
  hashes. It must not be treated as source-free proof evidence or augmented with source, replay,
  meta, theorem index, AI trace, or out-of-package state.
- `proofs/generated/axiom-report.json` records package policy, module axiom usage, checker
  summaries, and deterministic report hashes. `proofs/generated/theorem-index.json` records
  certificate-derived theorem/axiom entries, statement/interface hashes, dependency constants,
  axiom dependencies, artifact paths, checker summaries, and deterministic index hashes. Neither
  file is checker input.
- Fast and reference source-free library verification examples are
  `cargo test -p npa-api package_fast_verifier`,
  `cargo test -p npa-api package_reference_verifier`,
  `cargo test -p npa-proof-corpus package_fast_source_free`, and
  `cargo test -p npa-proof-corpus package_reference_source_free`.
- CLR-04 repository verification examples for this fixture are:

  ```sh
  cargo run -p npa-cli -- package check --root proofs
  cargo run -p npa-cli -- package build-certs --root proofs --check
  cargo run -p npa-cli -- package verify-certs --root proofs --checker reference
  cargo run -p npa-cli -- package check-hashes --root proofs
  cargo run -p npa-cli -- package axiom-report --root proofs --check
  cargo run -p npa-cli -- package index --root proofs --check
  cargo run -p npa-cli -- package publish-plan --root proofs --check
  ```

- The equivalent contributor-facing commands use the installed `npa` binary:

  ```sh
  npa package check --root .
  npa package build-certs --root . --check
  npa package verify-certs --root . --checker reference
  npa package check-hashes --root .
  npa package axiom-report --root . --check
  npa package index --root . --check
  npa package publish-plan --root . --check
  ```

- `package check` is a manifest metadata gate. `package build-certs` may read local
  `source.npa` and replay/helper data to rebuild certificates. `package check-hashes`
  reads checked-in source, certificate, and generated package-lock bytes to compare
  manifest-pinned hashes. `package verify-certs` is source-free: it reads
  `generated/package-lock.json` and certificate artifacts, not `.npa` source,
  replay, meta, theorem index, AI traces, or out-of-package state.
- `package axiom-report` and `package index` are source-free CLR-05 metadata commands. They read
  package metadata, the package lock, certificate artifacts, and checked generated artifacts in
  `--check` mode. They do not require source, replay, meta, theorem graph score, prompt metadata,
  AI traces, or out-of-package state.
- `package publish-plan` is the CLR-06 release metadata command. It reads package metadata,
  generated package artifacts, certificate artifacts, checker summaries, and the checked
  `generated/publish-plan.json` in `--check` mode. It does not contact a registry server, resolve
  latest versions, read registry URLs, upload release files, or sign artifacts. The signature
  policy is checksum-only SHA-256 in CLR-06; cryptographic signing remains later release workflow
  work.
- CLR-04 `npa package verify-certs` wraps
  `npa_package::build_package_lock_from_package_root`,
  `npa_package::parse_package_lock_json`, `npa_api::verify_package_fast_source_free`,
  `npa_api::verify_package_reference_source_free`,
  `npa_api::materialize_package_phase8_import_locks`, and
  `npa_api::materialize_package_phase8_requests`; it does not redefine graph traversal or
  checker policy mapping.
- Package CLI output and generated package metadata are review/CI diagnostics, not proof
  evidence. Accepted proof evidence remains canonical `.npcert` bytes plus the selected
  checker verdict.
- CLR-06 publish metadata and `npa.registry.module.v0.1` seed entries are helper data for release
  review, search, and downstream hash-pinned imports. They are not checker input and do not replace
  source-free local verification in downstream packages.
- Raw `npa-checker-ref` CLI import scanning is not enough for high-trust package graph verification
  by itself. Package verification also needs the CLR-03 package lock, pinned import identity,
  dependency-topological order, and imports accepted earlier by the same checker run.
- Current package fixture checks are `cargo test -p npa-proof-corpus package_fixture`,
  `cargo test -p npa-proof-corpus package_manifest_parity`, and
  `cargo test -p npa-proof-corpus package_fixture_hashes`.

Planning documents:

- `pythagorean-proof-phases.md`: P26-P34 plan for the checked abstract Pythagorean route.
- `law-of-cosines-proof-phases.md`: LC1-LC8 plan for the checked squared law-of-cosines route.
- `inner-product-to-metric-proof-phases.md`: IPM1-IPM14 plan from parallelogram law through
  polarization, Cauchy-Schwarz, and metric triangle inequality.
- `first-isomorphism-proof-phases.md`: FI0-FI5 plan for the AI-facing group first-isomorphism route.
- `second-isomorphism-proof-phases.md`: SI0-SI7 plan for the AI-facing group
  second-isomorphism route.
- `third-isomorphism-proof-phases.md`: TI0-TI4 plan for the AI-facing group third-isomorphism
  route.
- `correspondence-theorem-proof-phases.md`: CT0-CT8 plan for the AI-facing group correspondence
  theorem route.
- `inverse-implicit-function-proof-phases.md`: IIF0-IIF10 plan for the inverse-function and
  implicit-function theorem route.
- `fermats-last-theorem-proof-phases.md`: FLT0-FLT8 plan for a certificate-first Fermat's Last
  Theorem project, including bridge-axiom policy, elementary reduction, Frey/Ribet/modularity
  layers, and final high-trust audit criteria.

## Completed Inner-Product To Metric Route

The IPM route now has certificate-verified final theorem exports for the standard progression from
inner-product identities to the metric triangle inequality. These final checked theorem names are
separate from older theorem-shaped target wrappers.

| Goal | Final checked theorem | Main checked dependencies |
| --- | --- | --- |
| Parallelogram law | `Proofs.Ai.Vector.AbstractInnerProductDerive.parallelogram_law_from_inner_args` | `RingLawArgs`, `InnerProductLawArgs`, checked norm expansions, scalar normalization |
| Polarization identity | `Proofs.Ai.Vector.AbstractInnerProductDerive.polarization_identity_from_inner_args` | `RingLawArgs`, `InnerProductLawArgs`, checked norm expansion, scalar normalization |
| Cauchy-Schwarz inequality | `Proofs.Ai.Vector.AbstractInnerProductDerive.cauchy_schwarz_from_law_packages` | `RingLawArgs`, `OrderedFieldLawArgs`, `VectorSpaceLawArgs`, `InnerProductLawArgs.quadratic_norm_nonneg_law`, square-completion support |
| Metric triangle inequality | `Proofs.Ai.Geometry.AbstractMetric.triangle_inequality_from_law_packages` | `RingLawArgs`, `OrderedFieldLawArgs`, `VectorSpaceLawArgs`, `InnerProductLawArgs`, `AffineLawArgs`, checked Cauchy-Schwarz, squared Minkowski, and square comparison |

Legacy target / compatibility wrappers remain exported for older callers, but they are not the
completed checked proof paths:

| Area | Legacy wrapper / field | Status |
| --- | --- | --- |
| Abstract inner-product API | `parallelogram_law`, `polarization_identity`, `cauchy_schwarz` | Compatibility theorem targets; prefer the `_from_inner_args` / `_from_law_packages` exports above. |
| Abstract inner-product law package | `parallelogram_law_law`, `polarization_identity_law` | Direct peer theorem fields remain in the package surface, but the final IPM theorem exports do not project them. |
| Abstract metric API | `triangle_inequality` | Compatibility theorem target; prefer `triangle_inequality_from_law_packages`. |
| Abstract metric law package | `triangle_inequality_law` | Direct theorem field remains in `MetricSpaceLawArgs`, but the final IPM triangle proof does not take `MetricSpaceLawArgs`. |

This route proves the checked metric triangle inequality only. It does not claim angle,
trigonometric, completeness, normed-vector-space equivalence, or stronger analytic statements.

## Expansion Plan

Grow the corpus in small, checkable layers. Each layer should keep source, replay, metadata, and
certificate artifacts together, and every checked-in `.npcert` must be covered by an integration
test.

### P0: Basic Combinators

Module: `Proofs.Ai.Basic`

These are the initial no-import, no-axiom examples. They exercise binders, local lookup, direct
application, higher-order arguments, and simple proposition-shaped goals without relying on any
library theorem.

Implemented:

| Theorem | Shape |
| --- | --- |
| `id` | `A -> A` |
| `const_left` | `A -> B -> A` |
| `const_right` | `A -> B -> B` |
| `apply_fn` | `(A -> B) -> A -> B` |
| `compose` | `(B -> C) -> (A -> B) -> A -> C` |
| `flip` | `(A -> B -> C) -> B -> A -> C` |
| `duplicate` | `(A -> A -> B) -> A -> B` |
| `prop_id` | `P -> P` |
| `modus_ponens` | `(P -> Q) -> P -> Q` |
| `imp_trans` | `(P -> Q) -> (Q -> R) -> P -> R` |

### P1: More Basic Search Targets

Module: `Proofs.Ai.Basic`

These extend `Proofs.Ai.Basic` before introducing imports. They give AI search more variation while
staying in the same trusted boundary and proof style.

Implemented:

| Theorem | Shape |
| --- | --- |
| `compose_assoc` | `(C -> D) -> (B -> C) -> (A -> B) -> A -> D` |
| `apply_twice` | `(A -> A) -> A -> A`, with proof `f (f x)` |
| `ignore_middle` | `A -> B -> C -> A` |
| `select_middle` | `A -> B -> C -> B` |
| `select_last` | `A -> B -> C -> C` |
| `imp_swap` | `(P -> Q -> R) -> Q -> P -> R` |
| `imp_compose` | `(Q -> R) -> (P -> Q) -> P -> R` |
| `imp_ignore` | `P -> Q -> P` |
| `imp_duplicate` | `(P -> P -> Q) -> P -> Q` |
| `higher_apply` | `((A -> B) -> C) -> (A -> B) -> C` |

### P2: Equality Refl Corpus

Module: `Proofs.Ai.Eq`

This module imports `Std.Logic.Eq` and keeps the first equality examples refl-only. It checks import
interfaces and builtin equality references without adding rewrite search as a dependency. Later
Eq layers also import `Std.Nat.Basic` for Nat-specialized equality targets.

Implemented:

| Theorem | Shape |
| --- | --- |
| `eq_refl_self` | `x = x` |
| `eq_refl_fn_app` | `f x = f x` |
| `eq_refl_compose` | `f (g x) = f (g x)` |
| `eq_self_imp` | `x = x -> x = x` |
| `eq_refl_prop` | refl over a proposition-shaped term |

### P3: Nat Smoke Corpus

Module: `Proofs.Ai.Nat`

This module imports `Std.Nat.Basic` after P1 and P2 are stable. It also imports `Std.Logic.Eq`
for the refl-only equality smoke tests. Proofs stay closed by locals or refl/reduction so failures
are easy to attribute to import or kernel behavior.

Implemented:

| Theorem | Shape |
| --- | --- |
| `nat_zero_self_eq` | `Nat.zero = Nat.zero` |
| `nat_succ_zero_self_eq` | `Nat.succ Nat.zero = Nat.succ Nat.zero` |
| `nat_id` | `Nat -> Nat` |
| `nat_const_zero` | `Nat -> Nat`, with proof `Nat.zero` |
| `nat_apply_fn` | `(Nat -> Nat) -> Nat -> Nat` |

### P4: More Nat Search Targets

Module: `Proofs.Ai.Nat`

Extend the Nat corpus with closed local/application patterns before introducing recursion or
arithmetic lemmas. These should remain no-axiom proofs over `Std.Nat.Basic` and `Std.Logic.Eq`.

Implemented:

| Theorem | Shape |
| --- | --- |
| `nat_const_succ_zero` | `Nat -> Nat`, with proof `Nat.succ Nat.zero` |
| `nat_apply_twice` | `(Nat -> Nat) -> Nat -> Nat`, with proof `f (f n)` |
| `nat_compose` | `(Nat -> Nat) -> (Nat -> Nat) -> Nat -> Nat` |
| `nat_ignore_middle` | `Nat -> Nat -> Nat -> Nat`, selecting the first argument |
| `nat_select_middle` | `Nat -> Nat -> Nat -> Nat`, selecting the second argument |
| `nat_select_last` | `Nat -> Nat -> Nat -> Nat`, selecting the third argument |
| `nat_succ_self_eq` | `forall (n : Nat), Nat.succ n = Nat.succ n` |

### P5: Equality Shape Expansion

Module: `Proofs.Ai.Eq`

Add more refl-only equality targets with deeper application spines. The goal is to teach producers
to preserve the exact head, universe, and argument structure without relying on rewrite search.

Implemented:

| Theorem | Shape |
| --- | --- |
| `eq_refl_apply_twice` | `f (f x) = f (f x)` |
| `eq_refl_higher_apply` | `h f = h f` |
| `eq_refl_nested_compose` | `f (g (h x)) = f (g (h x))` |
| `eq_refl_prop_apply` | `h p = h p` for proposition-valued functions |
| `eq_local_passthrough` | `(h : x = x) -> x = x` |
| `eq_refl_nat_function` | `f n = f n` specialized to Nat |

### P6: Proposition Search Expansion

Module: `Proofs.Ai.Prop`

Split proposition-only implication patterns out of `Proofs.Ai.Basic` once the Basic module becomes
large. These remain import-free and should exercise binder ordering, argument permutation, and
higher-order implication search.

Implemented:

| Theorem | Shape |
| --- | --- |
| `imp_chain4` | `(P -> Q) -> (Q -> R) -> (R -> S) -> P -> S` |
| `imp_permute3` | `(P -> Q -> R -> S) -> R -> P -> Q -> S` |
| `imp_apply_twice` | `(P -> P) -> P -> P`, with proof `h (h p)` |
| `imp_const3` | `P -> Q -> R -> P` |
| `imp_flip_chain` | `(Q -> R) -> (P -> Q) -> P -> R` |
| `imp_higher_apply` | `((P -> Q) -> R) -> (P -> Q) -> R` |

### P7: Reduction Smoke Corpus

Module: `Proofs.Ai.Reduction`

Introduce small beta/zeta/delta-shaped examples only after the non-reduction corpora are stable.
Items involving named helper definitions may require extending the artifact generator beyond
theorem-only modules.

Implemented:

Definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `reduction_id_nat` | local Nat identity definition used by `delta_id_nat` |

Theorem targets:

| Theorem | Shape |
| --- | --- |
| `beta_id_nat` | `Nat -> Nat`, with proof `(fun x => x) n` |
| `beta_const_nat` | `Nat -> Nat -> Nat`, with proof `(fun x => fun _ => x) n m` |
| `let_id_nat` | `Nat -> Nat`, with proof `let x : Nat := n in x` |
| `let_const_nat` | `Nat -> Nat`, with proof `let z : Nat := Nat.zero in z` |
| `delta_id_nat` | `Nat -> Nat` through a local named identity definition |

### P8: Equality Reasoning Corpus

Module: `Proofs.Ai.EqReasoning`

Introduce equality elimination as an explicit, audited dependency. This layer imports
`Std.Logic.Eq` and intentionally uses the kernel builtin `Eq.rec` axiom interface. `Eq.rec` is
recorded in the certificate axiom report and is checked against the expected axiom list; no
additional module-local axioms are introduced.

Implemented:

Definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| none | Uses imported `Eq`, `Eq.refl`, and builtin `Eq.rec` only |

Theorem targets:

| Theorem | Shape |
| --- | --- |
| `eq_symm` | symmetry of equality |
| `eq_trans` | transitivity of equality |
| `eq_congr_arg` | congruence under a function argument |
| `eq_congr_fun` | congruence of equal functions at an argument |
| `eq_congr2` | congruence for a binary function |
| `eq_subst` | substitution into a proposition family |
| `eq_transport_const` | transport through a constant proposition family |
| `eq_rewrite_left` | left-to-right chained rewrite |
| `eq_rewrite_right` | right-side rewrite through symmetry-shaped input |
| `eq_cast_trans` | composed transport through two equalities |
| `eq_calc3` | three-step equality calculation using transitivity |

### P9: Algebra Ring Corpus

Module: `Proofs.Ai.Algebra.Ring`

Introduce a minimal algebra layer for later square/vector/geometry milestones. This module does
not add abstract ring axioms to the trusted base. Instead it defines a checked singleton carrier
`RingElem` and operation API over that carrier, then proves the selected ring-shaped law targets as
ordinary certificate-checked theorem declarations. The carrier and operations are API declarations,
not proof targets.

Implemented:

Definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `RingElem` | singleton scalar carrier for this corpus layer |
| `zero` | additive identity API |
| `one` | multiplicative identity API |
| `add` | addition API |
| `neg` | additive inverse API |
| `sub` | subtraction API, defined as `add a (neg b)` |
| `mul` | multiplication API |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `sub_eq_add_neg` | `a - b = a + -b` |
| `add_assoc` | `(a + b) + c = a + (b + c)` |
| `add_comm` | `a + b = b + a` |
| `add_zero` | `a + 0 = a` |
| `zero_add` | `0 + a = a` |
| `neg_add_cancel` | `-a + a = 0` |
| `add_neg_cancel` | `a + -a = 0` |
| `sub_self` | `a - a = 0` |
| `mul_assoc` | `(a * b) * c = a * (b * c)` |
| `mul_comm` | `a * b = b * a` |
| `mul_one` | `a * 1 = a` |
| `one_mul` | `1 * a = a` |
| `mul_zero` | `a * 0 = 0` |
| `zero_mul` | `0 * a = 0` |
| `left_distrib` | `a * (b + c) = a * b + a * c` |
| `right_distrib` | `(a + b) * c = a * c + b * c` |
| `add_left_cancel` | `a + b = a + c -> b = c` |
| `mul_add` | multiplication distributes over addition on the right argument |
| `add_mul` | multiplication distributes over addition on the left argument |
| `ring_normalize_add_mul3` | small normalization target for sums/products of three terms |

### P10: Algebra Square Corpus

Module: `Proofs.Ai.Algebra.Square`

Build on `Proofs.Ai.Algebra.Ring` with a small square API and the first square-expansion targets
needed by the coordinate / inner-product route to Pythagoras. As with P9, this is a concrete
singleton-carrier corpus layer rather than an abstract algebraic axiom package. `two` and `sq` are
API declarations; the square identities are proof targets checked through the certificate verifier.

Implemented:

Definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `two` | scalar `2` API, defined as `add one one` |
| `sq` | square operation API, defined as `mul a a` |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `square_def` | `sq a = a * a` |
| `mul_self_eq_square` | `a * a = sq a` |
| `sq_zero` | `sq 0 = 0` |
| `sq_one` | `sq 1 = 1` |
| `sq_neg` | `sq (-a) = sq a` |
| `two_mul` | `2 * a = a + a` |
| `sq_add` | `sq (a + b) = sq a + 2 * a * b + sq b` |
| `sq_sub` | `sq (a - b) = sq a - 2 * a * b + sq b` |
| `sum_two_squares_comm` | `sq a + sq b = sq b + sq a` |
| `sq_eq_sq_of_eq_or_neg_eq` | square equality from an equality-or-negated-equality witness shape |
| `square_nonneg` | predicate-generic bridge `Nonneg 0 -> Nonneg (sq a)`; P11 adds the concrete `le_square_nonneg` version |

### P11: Ordered Field Corpus

Module: `Proofs.Ai.OrderedField`

Build on `Proofs.Ai.Algebra.Ring` and `Proofs.Ai.Algebra.Square` with the first order and
square-root API targets needed by the later metric form of Pythagoras. This layer remains a
concrete singleton-carrier corpus: `le`, `lt`, and `sqrt` are API declarations, while the order
and square-root facts are certificate-checked theorem targets. `le`/`lt` are currently trivial
relations over the singleton scalar carrier; later abstract ordered-field work can replace this
with structure fields without changing the trusted boundary.

Implemented:

Definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `le` | non-strict order relation API |
| `lt` | strict order relation API |
| `sqrt` | square-root API for the later metric form |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `le_refl` | `a <= a` |
| `le_trans` | `a <= b -> b <= c -> a <= c` |
| `add_nonneg` | `0 <= a -> 0 <= b -> 0 <= a + b` |
| `mul_nonneg` | `0 <= a -> 0 <= b -> 0 <= a * b` |
| `le_square_nonneg` | concrete `le` version of `0 <= sq a`; named separately from P10's imported `square_nonneg` |
| `sqrt_nonneg` | `0 <= sqrt a` |
| `sqrt_square_of_nonneg` | `0 <= a -> sqrt (sq a) = a` |
| `sqrt_mul_self` | `0 <= a -> sqrt (a * a) = a` |
| `eq_of_square_eq_square_nonneg` | nonnegative equality from equal squares |

### P12: Vector Basic Corpus

Module: `Proofs.Ai.Vector.Basic`

Add the first vector carrier and additive group-shaped API targets for the coordinate route to
Pythagoras. This module intentionally stays independent of scalar/order APIs and imports only
`Std.Logic.Eq`; `Vector.Dot` is the next layer that combines vectors with the scalar field facts.
As with the earlier algebra layers, this is a concrete singleton-carrier corpus whose declarations
are checked through canonical certificates.

Implemented:

Definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `Vec` | singleton vector or point-difference carrier |
| `vec_zero` | vector zero |
| `vec_add` | vector addition |
| `vec_neg` | vector negation |
| `vec_sub` | vector subtraction, defined as `u + -v` |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `vec_add_assoc` | `(u + v) + w = u + (v + w)` |
| `vec_add_comm` | `u + v = v + u` |
| `vec_zero_add` | `0 + v = v` |
| `vec_add_zero` | `v + 0 = v` |
| `vec_neg_add_cancel` | `-v + v = 0` |
| `vec_add_neg_cancel` | `v + -v = 0` |
| `vec_sub_def` | `u - v = u + -v` |
| `vec_sub_eq_add_neg` | alias-style subtraction rewrite target for AI search |
| `vec_sub_self` | `v - v = 0` |
| `vec_sub_zero` | `v - 0 = v` |
| `vec_add_left_cancel` | `u + v = u + w -> v = w` |
| `sub_sub_sub_cancel` | `(u - w) - (v - w) = u - v` |

### P13: Vector Dot Corpus

Module: `Proofs.Ai.Vector.Dot`

Connect `Proofs.Ai.Vector.Basic` with the scalar corpus by adding dot product, squared norm, and
squared distance APIs. This is still a singleton-carrier corpus, so the theorem statements are
designed as durable targets for later nontrivial instances while the current certificates remain
small and axiom-free.

Implemented:

Definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `dot` | inner product API |
| `normSq` | squared norm, defined as `dot v v` |
| `distSq` | squared distance, defined as `normSq (B - A)` |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `dot_comm` | `dot u v = dot v u` |
| `dot_add_left` | `dot (u + v) w = dot u w + dot v w` |
| `dot_add_right` | `dot u (v + w) = dot u v + dot u w` |
| `dot_neg_left` | `dot (-u) v = -dot u v` |
| `dot_neg_right` | `dot u (-v) = -dot u v` |
| `dot_sub_left` | `dot (u - v) w = dot u w - dot v w` |
| `dot_sub_right` | `dot u (v - w) = dot u v - dot u w` |
| `norm_sq_def` | `normSq v = dot v v` |
| `dist_sq_def` | `distSq A B = normSq (B - A)` |
| `dot_self_eq_norm_sq` | `dot v v = normSq v` |
| `norm_sq_add` | `normSq (u + v) = normSq u + 2 * dot u v + normSq v` |
| `norm_sq_sub` | `normSq (u - v) = normSq u - 2 * dot u v + normSq v` |
| `norm_sq_add_of_dot_zero` | `dot u v = 0 -> normSq (u + v) = normSq u + normSq v` |
| `norm_sq_sub_of_dot_zero` | `dot u v = 0 -> normSq (u - v) = normSq u + normSq v` |
| `parallelogram_law` | `normSq (u + v) + normSq (u - v) = 2 * normSq u + 2 * normSq v` |
| `polarization_identity` | `2 * dot u v = normSq (u + v) - (normSq u + normSq v)` |
| `norm_sq_nonneg` | `0 <= normSq v` |

### P14: Geometry Right Triangle Corpus

Module: `Proofs.Ai.Geometry.RightTriangle`

Add the first geometry layer over `Vec`, `dot`, `normSq`, and `distSq`. The main milestone target is
the squared-distance Pythagorean theorem
`RightTriangle A B C -> distSq B C = distSq A B + distSq A C`, with helper rewrites for the leg and
hypotenuse vectors. `perp_iff_dot_eq_zero` uses a Church-encoded equivalence eliminator because the
corpus does not yet define a first-class `Iff`; the later geometric API placeholders such as
midpoint or altitude foot are passed as predicate parameters rather than new definitions.

Implemented:

Definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `Perp` | perpendicularity predicate, defined as `dot u v = 0` |
| `RightTriangle` | right-triangle predicate over three points, with the right angle at `A` |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `perp_iff_dot_eq_zero` | Church-encoded `Perp u v <-> dot u v = 0` |
| `perp_symm` | `Perp u v -> Perp v u` |
| `right_triangle_legs_perp` | extract perpendicular leg vectors from `RightTriangle A B C` |
| `hypotenuse_vector_eq_sub_legs` | `C - B = (C - A) - (B - A)` |
| `dist_sq_leg_left` | `distSq A B = normSq (B - A)` |
| `dist_sq_leg_right` | `distSq A C = normSq (C - A)` |
| `dist_sq_hypotenuse` | `distSq B C = normSq (C - B)` |
| `pythagorean_distance_sq` | `RightTriangle A B C -> distSq B C = distSq A B + distSq A C` |
| `law_of_cosines` | squared distance with a dot-product correction term |
| `right_triangle_area` | double-area squared target parameterized by a future `Area2` API |
| `median_to_hypotenuse` | midpoint-on-hypotenuse target parameterized by a future midpoint predicate |
| `altitude_on_hypotenuse` | altitude-foot target parameterized by future length and foot predicates |
| `thales_theorem` | circle/diameter-to-right-triangle target parameterized by a future circle predicate |

### P15: Geometry Metric Corpus

Module: `Proofs.Ai.Geometry.Metric`

Add the first distance layer over `distSq`, `sqrt`, and the right-triangle corpus. The main milestone
target is the squared metric Pythagorean statement
`RightTriangle A B C -> sq (dist B C) = sq (dist A B) + sq (dist A C)`. `distance_zero_iff_eq`
uses the same Church-encoded equivalence shape as P14 because the corpus still does not define a
first-class `Iff`.

Implemented:

Definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `dist` | distance API, defined as `sqrt (distSq A B)` |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `dist_def` | `dist A B = sqrt (distSq A B)` |
| `dist_sq_eq_square_dist` | `distSq A B = sq (dist A B)` |
| `dist_nonneg` | `0 <= dist A B` |
| `distance_symm` | `dist A B = dist B A` |
| `distance_zero_iff_eq` | Church-encoded `dist A B = 0 <-> A = B` |
| `pythagorean_distance` | `RightTriangle A B C -> sq (dist B C) = sq (dist A B) + sq (dist A C)` |
| `cauchy_schwarz` | older concrete metric target; not the abstract IPM checked Cauchy-Schwarz route |
| `triangle_inequality` | older concrete metric target; not the abstract IPM checked triangle-inequality route |

### P16: Logic Iff Corpus

Module: `Proofs.Ai.Logic.Iff`

Add first-class logical connectives for later abstract theorem APIs. `Iff`, `And`, `Or`, `False`,
and `Not` are defined as Prop-valued APIs so later modules can stop embedding ad hoc
Church-encoded equivalence shapes in theorem statements. `iff_of_eq` and `iff_congr_arg` use the
same audited `Eq.rec` dependency as P8; the expected axiom report is fixed to `["Eq.rec"]`.

Implemented:

Definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `Iff` | first-class logical equivalence, replacing ad hoc theorem-local equivalence encodings |
| `And` | conjunction API for bundling law hypotheses |
| `Or` | disjunction API for square-root and order case splits |
| `False` | empty proposition API for contradiction and negation eliminators |
| `Not` | negation abbreviation, defined as `P -> False` |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `iff_refl` | `Iff P P` |
| `iff_symm` | `Iff P Q -> Iff Q P` |
| `iff_trans` | `Iff P Q -> Iff Q R -> Iff P R` |
| `iff_mp` | `Iff P Q -> P -> Q` |
| `iff_mpr` | `Iff P Q -> Q -> P` |
| `and_intro` | `P -> Q -> And P Q` |
| `and_left` | `And P Q -> P` |
| `and_right` | `And P Q -> Q` |
| `iff_of_eq` | `P = Q -> Iff P Q` |
| `false_elim` | `False -> P` |
| `not_intro` | `(P -> False) -> Not P` |
| `not_elim` | `Not P -> P -> False` |
| `or_inl` | `P -> Or P Q` |
| `or_inr` | `Q -> Or P Q` |
| `or_elim` | `Or P Q -> (P -> R) -> (Q -> R) -> R` |
| `iff_congr_arg` | `P = Q -> Iff (F P) (F Q)` for Prop-valued contexts |

### General Euclidean Pythagorean Roadmap

Long-term target: prove the Pythagorean theorem as a checked certificate over an abstract Euclidean
space, not only over the current concrete singleton corpus layer. Prefer the coordinate /
inner-product route first:

```text
RightTriangle A B C -> distSqPoints B C = distSqPoints A B + distSqPoints A C
```

Post-P25 implementation phases are tracked in `proofs/pythagorean-proof-phases.md`.

This avoids making the first abstract target depend on square roots. P15 adds the checked bridge to
the squared `dist` form over the current concrete scalar and vector corpus. P17 starts replacing the
singleton scalar layer with explicit carrier, operation, and law assumptions. P18 extends that
abstract scalar layer with order and square-root APIs. P19 supplies the abstract square
normalization layer. P20 supplies the abstract vector-space layer. P21 supplies the abstract
inner-product and squared-norm layer. P22 supplies the affine point/displacement layer. P23 supplies
the abstract right-triangle theorem layer. P24 supplies the abstract metric-distance theorem layer.
P25 supplies the final theorem API names that downstream users can depend on, P31 connects the
squared-distance theorem name to the checked law-package derivation, and P32 connects the squared
metric-distance theorem name to the checked metric bridge. P34 reviews the optional converse and
unsquared-distance strengthenings and keeps them out of completed theorem claims until their
nondegeneracy, angle, and square-root cancellation prerequisites are available.

Planned contents:

The `Definition / API declarations` column lists declarations introduced by `def`, structure
fields, or primitives. They are type-checked declarations, not proof targets. The theorem columns
list declarations that should have checked proof certificates. Definitional rewrite lemmas such as
`sub_eq_add_neg` and `square_def` are theorem targets, although many should close by `Eq.refl`
after unfolding.

Completed prerequisite:

- P8 `Proofs.Ai.EqReasoning` supplies equality symmetry, transitivity, congruence, substitution,
  transport, and calculation lemmas.
- P9 `Proofs.Ai.Algebra.Ring` supplies the first algebra API declarations and certificate-checked
  ring-shaped law targets over a concrete singleton carrier.
- P10 `Proofs.Ai.Algebra.Square` supplies `two`, `sq`, and square-expansion theorem targets over
  the same concrete scalar carrier.
- P11 `Proofs.Ai.OrderedField` supplies `le`, `lt`, `sqrt`, and the nonnegative square-root theorem
  targets needed by later metric statements.
- P12 `Proofs.Ai.Vector.Basic` supplies the first vector carrier and additive vector theorem targets
  used by the dot-product and geometry layers.
- P13 `Proofs.Ai.Vector.Dot` supplies `dot`, `normSq`, `distSq`, and the dot-product expansion
  targets used by the squared-distance Pythagoras route.
- P14 `Proofs.Ai.Geometry.RightTriangle` supplies `Perp`, `RightTriangle`, leg/hypotenuse rewrites,
  and the checked squared-distance Pythagorean theorem target.
- P15 `Proofs.Ai.Geometry.Metric` supplies `dist`, the `distSq = sq dist` bridge, and the checked
  squared metric Pythagorean theorem target.
- P16 `Proofs.Ai.Logic.Iff` supplies first-class `Iff`, `And`, `Or`, `False`, and `Not` APIs for
  later abstract algebra and geometry theorem statements.
- P17 `Proofs.Ai.Algebra.AbstractRing` supplies checked abstract ring theorem targets over explicit
  carrier, operation, and law assumptions without adding unchecked algebra axioms.
- P18 `Proofs.Ai.Algebra.AbstractOrderedField` supplies checked abstract order and square-root
  theorem targets over explicit carrier, operation, relation, function, and law assumptions without
  adding unchecked order or square-root axioms.
- P19 `Proofs.Ai.Algebra.AbstractSquareNormalize` supplies checked abstract square-normalization
  theorem targets over the P17/P18 scalar APIs and explicit law assumptions without adding unchecked
  algebra or order axioms.
- P20 `Proofs.Ai.Vector.AbstractSpace` supplies checked abstract vector-space theorem targets over
  explicit vector carrier, operation, scalar, and law assumptions without adding unchecked vector
  space axioms.
- P21 `Proofs.Ai.Vector.AbstractInnerProduct` supplies checked abstract inner-product, squared norm,
  vector squared-distance, perpendicularity, and norm-expansion theorem targets over explicit law
  assumptions without adding unchecked inner-product or positivity axioms.
- P22 `Proofs.Ai.Geometry.Affine` supplies checked abstract point, displacement, point
  squared-distance, and point extensionality theorem targets over explicit affine law assumptions
  without adding unchecked affine or Euclidean axioms.
- P23 `Proofs.Ai.Geometry.AbstractRightTriangle` supplies checked abstract perpendicularity,
  right-triangle, squared-distance Pythagorean, law-of-cosines, area, and median theorem targets
  over explicit geometry law assumptions without adding unchecked Euclidean axioms.
- P24 `Proofs.Ai.Geometry.AbstractMetric` supplies checked abstract distance, metric law-package,
  ball API, distance/squared-distance bridge, metric Pythagorean, and triangle-inequality theorem
  targets over explicit metric law assumptions without adding unchecked metric axioms.
- P25 `Proofs.Ai.Geometry.Pythagorean` supplies checked final abstract Pythagorean theorem names,
  alias targets, converse target shape, and dependency-package theorem target over the P17-P24
  abstract geometry stack without adding unchecked Euclidean axioms.
- P27 `Proofs.Ai.Algebra.AbstractScalarDerive` supplies checked scalar zero-cross-term derivations
  from `RingLawArgs` and equality transport, without accepting direct theorem-shaped scalar
  normalization law arguments.
- P28 `Proofs.Ai.Vector.AbstractInnerProductDerive` supplies checked norm expansion and
  perpendicular special-case derivations from `InnerProductLawArgs`, P27 scalar rewrites, and
  equality transport, without accepting direct dot-zero or perpendicular norm law arguments.
- P29 `Proofs.Ai.Geometry.AffineDerive` supplies checked affine hypotenuse orientation and
  point-distance/norm bridge derivations from primitive `AffineLawArgs`, `VectorSpaceLawArgs`, and
  equality transport, without accepting direct hypotenuse-vector or point-distance-definition law
  arguments.
- P30 `Proofs.Ai.Geometry.AbstractRightTriangleDerive` supplies checked bridges from
  `RightTriangle A B C` to the exact `PerpVec` / dot-zero premises needed after P29's additive
  hypotenuse orientation, without accepting a direct Pythagorean theorem-shaped argument.
- P31 `Proofs.Ai.Geometry.Pythagorean` supplies the checked squared-distance Pythagorean theorem
  from `RingLawArgs`, `VectorSpaceLawArgs`, `InnerProductLawArgs`, `AffineLawArgs`, and
  `RightTriangle A B C`, without accepting a direct Pythagorean theorem-shaped equality law.
- P32 `Proofs.Ai.Geometry.Pythagorean` supplies the checked squared metric-distance theorem by
  composing P31 with the P32 metric bridge, without accepting a direct metric Pythagorean law.
- P33 refreshes the final public Pythagorean API names and documentation around the completed
  squared and squared-metric theorem claims.
- P34 records the optional-strengthening boundary: no converse or unsquared-distance theorem is
  exported until checked nondegeneracy, angle, first-class `Iff` import, and square-root
  cancellation prerequisites are available.
- LC5 `Proofs.Ai.Geometry.Pythagorean` supplies the checked squared point-distance law of cosines
  as `law_of_cosines_sq_from_law_packages`, using the LC4 affine bridge and law packages instead of
  the legacy `law_of_cosines_general` direct wrapper.
- LC6 supplies `law_of_cosines_right_angle_specialization_from_law_packages` and reconnects the
  public `law_of_cosines_right_angle_specialization` alias to that checked law-of-cosines route.
- LC7 supplies the checked squared metric-distance law of cosines
  `law_of_cosines_dist_sq_from_law_packages` by composing LC5 with the P32 metric square bridge.

Post-P25 policy:

- Keep all algebraic, order, vector-space, and inner-product laws as explicit theorem assumptions or
  checked law-package arguments until NPA has a dedicated structure/class layer.
- Do not introduce module-level unchecked axioms for field, vector, order, real, or Euclidean facts.
- Keep the final theorem independent of the concrete singleton `RingElem` and `Vec` carriers.
- Prefer squared-distance statements first; add square-root distance forms only after the required
  nonnegative square-root and square-cancellation lemmas are available.

The current P17-P34 abstract Pythagorean roadmap now has checked squared-distance and squared
metric-distance theorem names from law packages. The LC5-LC7 law-of-cosines follow-up also has
checked squared point-distance, right-angle specialization, and squared metric-distance theorem
names. Later work can replace explicit law arguments with checked structure/class packages, add
direct first-class `Iff` imports once duplicate `Eq` handoff is resolved, and strengthen the
converse, unsquared-distance, angle, and trigonometric cosine statements as the required
nondegeneracy, angle, and square-root cancellation APIs become available.

The intended dependency order is:

```text
EqReasoning
  -> Algebra.Ring -> Algebra.Square -> OrderedField
  -> Vector.Basic -> Vector.Dot
  -> Geometry.RightTriangle -> Geometry.Metric
  -> Logic.Iff
  -> Algebra.AbstractRing -> Algebra.AbstractField
  -> Algebra.AbstractRingFirstIsoBase -> Algebra.AbstractFieldHom
  -> Algebra.AbstractRingFirstIso -> Algebra.AbstractRingChineseRemainder
  -> Algebra.AbstractUfdPrimeFactorization -> Algebra.AbstractFieldIntegralDomain
  -> Algebra.AbstractHilbertBasisTheorem
  -> Algebra.AbstractHilbertNullstellensatz
  -> Algebra.AbstractKrullTheorem
  -> Algebra.AbstractFieldIdeal
  -> Algebra.AbstractOrderedField -> Algebra.AbstractOrderedFieldFieldBridge
  -> Algebra.AbstractSquareNormalize
  -> Algebra.AbstractScalarDerive
  -> Vector.AbstractSpace -> Vector.AbstractInnerProduct -> Vector.AbstractInnerProductDerive
  -> LinearAlgebra.AbstractSpectralTheorem
  -> FunctionalAnalysis.AbstractHilbertSpaceSpectralTheorem
  -> Geometry.Affine -> Geometry.AffineDerive
  -> Geometry.AbstractRightTriangle -> Geometry.AbstractRightTriangleDerive
  -> Geometry.AbstractMetric
  -> Geometry.Pythagorean
```

Recommended module contents:

#### `Proofs.Ai.EqReasoning`

No new definitions live here; the module builds theorem targets over imported `Eq` and the
expected builtin `Eq.rec` axiom interface.

| Theorem | Shape / purpose |
| --- | --- |
| `eq_symm` | `x = y -> y = x` |
| `eq_trans` | `x = y -> y = z -> x = z` |
| `eq_congr_arg` | `x = y -> f x = f y` |
| `eq_congr_fun` | `f = g -> f x = g x` |
| `eq_congr2` | `a = a' -> b = b' -> f a b = f a' b'` |
| `eq_subst` | transport a proof across equality |
| `eq_transport_const` | transport through a constant family |
| `eq_rewrite_left` | rewrite the left side of an equality target |
| `eq_rewrite_right` | rewrite the right side of an equality target |
| `eq_cast_trans` | compose transports through two equalities |
| `eq_calc3` | three-step equality chaining helper for AI-generated calc blocks |

#### `Proofs.Ai.Algebra.Ring`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `RingElem` | singleton scalar carrier for the current concrete corpus layer |
| `zero` | additive identity API |
| `one` | multiplicative identity API |
| `add` | addition API |
| `neg` | additive inverse API |
| `sub` | subtraction API, normally defined as `a + -b` |
| `mul` | multiplication API |

Definitional rewrite theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `sub_eq_add_neg` | `a - b = a + -b` |

Algebra law theorem targets. If this module later represents an abstract `Ring` structure, these
may be law fields or projections there; concrete scalar instances still need checked certificates
for the laws.

| Theorem | Shape / purpose |
| --- | --- |
| `add_assoc` | `(a + b) + c = a + (b + c)` |
| `add_comm` | `a + b = b + a` |
| `add_zero` | `a + 0 = a` |
| `zero_add` | `0 + a = a` |
| `neg_add_cancel` | `-a + a = 0` |
| `add_neg_cancel` | `a + -a = 0` |
| `sub_self` | `a - a = 0` |
| `mul_assoc` | `(a * b) * c = a * (b * c)` |
| `mul_comm` | `a * b = b * a` |
| `mul_one` | `a * 1 = a` |
| `one_mul` | `1 * a = a` |
| `mul_zero` | `a * 0 = 0` |
| `zero_mul` | `0 * a = 0` |
| `left_distrib` | `a * (b + c) = a * b + a * c` |
| `right_distrib` | `(a + b) * c = a * c + b * c` |
| `add_left_cancel` | `a + b = a + c -> b = c` |
| `mul_add` | `a * (b + c) = a * b + a * c` |
| `add_mul` | `(a + b) * c = a * c + b * c` |
| `ring_normalize_add_mul3` | small normalization target for sums/products of three terms |

#### `Proofs.Ai.Algebra.Square`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `two` | scalar `2`, normally `1 + 1` |
| `sq` | square operation, normally `sq a := a * a` |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `square_def` | `sq a = a * a` |
| `mul_self_eq_square` | `a * a = sq a` |
| `sq_zero` | `sq 0 = 0` |
| `sq_one` | `sq 1 = 1` |
| `sq_neg` | `sq (-a) = sq a` |
| `two_mul` | `2 * a = a + a` |
| `sq_add` | `sq (a + b) = sq a + 2 * a * b + sq b` |
| `sq_sub` | `sq (a - b) = sq a - 2 * a * b + sq b` |
| `sum_two_squares_comm` | `sq a + sq b = sq b + sq a` |
| `sq_eq_sq_of_eq_or_neg_eq` | square equality from an equality-or-negated-equality witness shape |
| `square_nonneg` | predicate-generic bridge to P11's ordered relation work |

#### `Proofs.Ai.OrderedField`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `le` | non-strict order relation |
| `lt` | strict order relation |
| `sqrt` | square-root API for the later metric form |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `le_refl` | `a <= a` |
| `le_trans` | `a <= b -> b <= c -> a <= c` |
| `add_nonneg` | `0 <= a -> 0 <= b -> 0 <= a + b` |
| `mul_nonneg` | `0 <= a -> 0 <= b -> 0 <= a * b` |
| `le_square_nonneg` | `0 <= sq a` over the ordered singleton scalar carrier; avoids colliding with P10's imported `square_nonneg` bridge |
| `sqrt_nonneg` | `0 <= sqrt a` |
| `sqrt_square_of_nonneg` | `0 <= a -> sqrt (sq a) = a` |
| `sqrt_mul_self` | `0 <= a -> sqrt (a * a) = a` |
| `eq_of_square_eq_square_nonneg` | nonnegative equality from equal squares |

#### `Proofs.Ai.Vector.Basic`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `Vec` | vector or point-difference carrier |
| `vec_zero` | vector zero |
| `vec_add` | vector addition |
| `vec_neg` | vector negation |
| `vec_sub` | vector subtraction, normally `u + -v` |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `vec_add_assoc` | `(u + v) + w = u + (v + w)` |
| `vec_add_comm` | `u + v = v + u` |
| `vec_zero_add` | `0 + v = v` |
| `vec_add_zero` | `v + 0 = v` |
| `vec_neg_add_cancel` | `-v + v = 0` |
| `vec_add_neg_cancel` | `v + -v = 0` |
| `vec_sub_def` | `u - v = u + -v` |
| `vec_sub_eq_add_neg` | alias-style subtraction rewrite target for AI search |
| `vec_sub_self` | `v - v = 0` |
| `vec_sub_zero` | `v - 0 = v` |
| `vec_add_left_cancel` | `u + v = u + w -> v = w` |
| `sub_sub_sub_cancel` | `(u - w) - (v - w) = u - v`, used for triangle vertices |

#### `Proofs.Ai.Vector.Dot`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `dot` | inner product API |
| `normSq` | squared norm, normally `dot v v` |
| `distSq` | squared distance, normally `normSq (B - A)` |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `dot_comm` | `dot u v = dot v u` |
| `dot_add_left` | `dot (u + v) w = dot u w + dot v w` |
| `dot_add_right` | `dot u (v + w) = dot u v + dot u w` |
| `dot_neg_left` | `dot (-u) v = -dot u v` |
| `dot_neg_right` | `dot u (-v) = -dot u v` |
| `dot_sub_left` | `dot (u - v) w = dot u w - dot v w` |
| `dot_sub_right` | `dot u (v - w) = dot u v - dot u w` |
| `norm_sq_def` | `normSq v = dot v v` |
| `dist_sq_def` | `distSq A B = normSq (B - A)` |
| `dot_self_eq_norm_sq` | `dot v v = normSq v` |
| `norm_sq_add` | `normSq (u + v) = normSq u + 2 * dot u v + normSq v` |
| `norm_sq_sub` | `normSq (u - v) = normSq u - 2 * dot u v + normSq v` |
| `norm_sq_add_of_dot_zero` | `dot u v = 0 -> normSq (u + v) = normSq u + normSq v` |
| `norm_sq_sub_of_dot_zero` | `dot u v = 0 -> normSq (u - v) = normSq u + normSq v` |
| `parallelogram_law` | `normSq (u + v) + normSq (u - v) = 2 * normSq u + 2 * normSq v` |
| `polarization_identity` | `2 * dot u v = normSq (u + v) - (normSq u + normSq v)` |
| `norm_sq_nonneg` | `0 <= normSq v` |

#### `Proofs.Ai.Geometry.RightTriangle`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `Perp` | perpendicularity predicate |
| `RightTriangle` | right-triangle predicate over three points |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `perp_iff_dot_eq_zero` | Church-encoded `Perp u v <-> dot u v = 0` |
| `perp_symm` | `Perp u v -> Perp v u` |
| `right_triangle_legs_perp` | extract perpendicular leg vectors from `RightTriangle A B C` |
| `hypotenuse_vector_eq_sub_legs` | express the hypotenuse vector through the two leg vectors |
| `dist_sq_leg_left` | rewrite the first leg length as a `distSq` term |
| `dist_sq_leg_right` | rewrite the second leg length as a `distSq` term |
| `dist_sq_hypotenuse` | rewrite the hypotenuse length as a `distSq` term |
| `pythagorean_distance_sq` | `RightTriangle A B C -> distSq B C = distSq A B + distSq A C` |
| `law_of_cosines` | peer theorem: squared distance with a dot-product correction term |
| `right_triangle_area` | double-area squared target parameterized by a future `Area2` API |
| `median_to_hypotenuse` | midpoint-on-hypotenuse target parameterized by a future midpoint predicate |
| `altitude_on_hypotenuse` | altitude-foot target parameterized by future length and foot predicates |
| `thales_theorem` | circle/diameter-to-right-triangle target parameterized by a future circle predicate |

#### `Proofs.Ai.Geometry.Metric`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `dist` | distance API, normally `sqrt (distSq A B)` |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `dist_def` | `dist A B = sqrt (distSq A B)` |
| `dist_sq_eq_square_dist` | `distSq A B = sq (dist A B)` |
| `dist_nonneg` | `0 <= dist A B` |
| `distance_symm` | `dist A B = dist B A` |
| `distance_zero_iff_eq` | Church-encoded `dist A B = 0 <-> A = B` |
| `pythagorean_distance` | `RightTriangle A B C -> sq (dist B C) = sq (dist A B) + sq (dist A C)` |
| `cauchy_schwarz` | older concrete metric target; not the abstract IPM checked Cauchy-Schwarz route |
| `triangle_inequality` | older concrete metric target; not the abstract IPM checked triangle-inequality route |

#### `Proofs.Ai.Logic.Iff`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `Iff` | first-class logical equivalence, replacing ad hoc theorem-local equivalence encodings |
| `And` | conjunction API for bundling law hypotheses when a law-package style is useful |
| `Or` | disjunction API for square-root and order case splits |
| `False` | empty proposition API for contradiction and negation eliminators |
| `Not` | negation abbreviation, normally `P -> False` |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `iff_refl` | `Iff P P` |
| `iff_symm` | `Iff P Q -> Iff Q P` |
| `iff_trans` | `Iff P Q -> Iff Q R -> Iff P R` |
| `iff_mp` | `Iff P Q -> P -> Q` |
| `iff_mpr` | `Iff P Q -> Q -> P` |
| `and_intro` | `P -> Q -> And P Q` |
| `and_left` | `And P Q -> P` |
| `and_right` | `And P Q -> Q` |
| `iff_of_eq` | `P = Q -> Iff P Q` |
| `false_elim` | `False -> P` |
| `not_intro` | `(P -> False) -> Not P` |
| `not_elim` | `Not P -> P -> False` |
| `or_inl`, `or_inr`, `or_elim` | disjunction introduction and elimination helpers |
| `iff_congr_arg` | `P = Q -> Iff (F P) (F Q)` for Prop-valued contexts |

#### `Proofs.Ai.NumberTheory.Flt.Statement`

Implemented definitions / API declarations:

| Declaration | Purpose |
| --- | --- |
| `FltFalse` | FLT-local Church-encoded falsehood used to spell the contradiction target explicitly |
| `FltNot` | negation abbreviation `P -> FltFalse`, used for nonzero hypotheses |
| `FltNatTwo` | certified Nat numeral `2`, defined as `Nat.succ (Nat.succ Nat.zero)` |
| `FltNatNe` | explicit natural-number inequality predicate, defined as negated `Eq` |
| `FltNatEquation` | explicit equation shape `a^n + b^n = c^n` over parameterized `add` and `pow` |
| `fermat_last_theorem` | public final natural-number statement constant parameterized by `add`, `pow`, and `lt` |
| `fermat_last_theorem_nat` | compatibility alias for the public natural-number statement |
| `fermat_last_theorem_positive_nat` | positive-natural statement alias through an explicit `toNat` embedding |
| `fermat_last_theorem_int` | integer statement alias over an explicit integer carrier, zero, addition, and power operation |

The statement module is the FLT-00 contract layer. It freezes names and surface shape only; it does
not prove Fermat's Last Theorem and it does not introduce bridge axioms. The current `Std.Nat.Basic`
fixture supplies the certified Nat carrier plus `Nat.zero` and `Nat.succ`. Because reusable
addition, exponentiation, and order APIs are scheduled for later number-theory milestones, the
statement constants take `add`, `pow`, and `lt` as explicit arguments rather than using notation,
typeclass search, or hidden source sugar.

Bridge policy and library growth policy are deliberately separated:

- `Flt.BridgeAxiom.*` declarations are development-only interfaces for later smoke milestones and
  must not be imported by this statement module.
- Domain milestones that follow FLT-00 are incomplete if they only add hidden FLT glue; they must
  contribute reusable number theory, algebra, elliptic-curve, modular-forms, Galois-representation,
  or modularity theorem surfaces that can be used independently of the final FLT proof route.
- Metadata files such as `manifest.toml`, `npa-package.toml`, generated theorem indexes, and axiom
  reports make the surface discoverable, but proof acceptance remains canonical `.npcert` bytes
  plus checker verdicts.

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `fermat_last_theorem_shape` | reflexive check that the public statement expands to the frozen Nat form |
| `fermat_last_theorem_nat_alias` | reflexive check that the Nat alias is the public statement |
| `fermat_last_theorem_positive_nat_shape` | reflexive check of the positive-natural compatibility shape |
| `fermat_last_theorem_int_shape` | reflexive check of the integer compatibility shape |

#### `Proofs.Ai.Algebra.AbstractRing`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `Scalar` | local carrier parameter used by every abstract ring definition and theorem target |
| `zero`, `one`, `add`, `neg`, `sub`, `mul` | local operation parameters, keeping the module independent of concrete `RingElem` |
| `two` | parametric scalar `2`, defined from an explicit `one` and `add` |
| `sq` | parametric square helper, defined from an explicit `mul` |
| `RingLawArgs` | Church-encoded law package API over the explicit carrier and operations |

The checked theorem targets take the corresponding law as an explicit argument and return it at the
requested variables. This keeps the corpus certificate-first and avoids adding module-level
unchecked ring axioms. Multiplicative nonzero cancellation remains outside `AbstractRing`; the
field route introduces the reusable `Nonzero` API first, and later field-calculation modules derive
the cancellation targets with explicit nonzero evidence.

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `sub_eq_add_neg` | `sub a b = add a (neg b)` |
| `add_assoc`, `add_comm`, `add_zero`, `zero_add` | additive monoid/group laws |
| `neg_add_cancel`, `add_neg_cancel`, `sub_self` | additive inverse and subtraction laws |
| `mul_assoc`, `mul_comm`, `mul_one`, `one_mul` | commutative multiplication laws |
| `left_distrib`, `right_distrib` | distributivity over addition |
| `mul_zero`, `zero_mul`, `add_left_cancel` | cancellation and zero-product helper targets |
| `ring_normalize_add_mul3` | `((a*b)+(b*c))+(a*c) = ((a*b)+(a*c))+(b*c)` normalization target |
| `add_right_cancel` | `b + a = c + a -> b = c` |
| `neg_neg` | `-(-a) = a` |
| `sub_zero`, `zero_sub` | subtraction by zero and from zero |
| `sub_add_cancel`, `add_sub_cancel` | basic subtraction/addition cancellation lemmas |
| `sub_add_sub_cancel` | `(a - c) - (b - c) = a - b`, scalar analogue of vector displacement cancellation |

#### `Proofs.Ai.Algebra.AbstractField`

Implemented definitions / API declarations:

| Declaration | Purpose |
| --- | --- |
| `FieldFalse`, `FieldNot` | local Church-encoded falsehood and negation used only to keep `Nonzero` Prop-level |
| `Nonzero` | reusable predicate `a != 0` over an explicit carrier and zero element |
| `div` | derived operation `a / b := a * inv b`, not a core primitive or law-package field |
| `FieldLawArgs` | Church-encoded field law package separating `RingLawArgs` from field-specific inverse, division, nonzero, cancellation, and zero-product laws |

The foundation imports `AbstractRing` and reuses its `RingLawArgs` instead of restating ring laws.
The division API is deliberately definitional: `field_div_eq_mul_inv` closes by reflexivity after
unfolding `div`, so later modules can search for a theorem name without trusting a new primitive
operation.

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `field_ring_laws` | projects `RingLawArgs` from `FieldLawArgs` for downstream abstract-ring theorem reuse |
| `field_zero_ne_one` | projects `Nonzero one`, the field-level `0 != 1` witness |
| `field_inv_mul_cancel` | `a != 0 -> inv a * a = 1` from the field law package |
| `field_mul_inv_cancel` | `a != 0 -> a * inv a = 1` from the field law package |
| `field_div_eq_mul_inv` | certificate-backed definitional equality for derived division |
| `field_inv_one` | projects `inv 1 = 1` |
| `field_div_one` | projects `a / 1 = a` |
| `field_div_self_nonzero` | projects `a != 0 -> a / a = 1` |
| `field_zero_div` | projects `0 / a = 0` for the total derived division operation |
| `field_mul_left_cancel_nonzero` | projects left multiplication cancellation with explicit nonzero evidence |
| `field_mul_right_cancel_nonzero` | projects right multiplication cancellation with explicit nonzero evidence |
| `field_nonzero_mul_closed` | projects nonzero closure under multiplication |
| `field_mul_eq_zero_cases` | Church-encoded zero-product case split into `a = 0` or `b = 0` continuations |

#### `Proofs.Ai.Algebra.AbstractFieldHom`

Implemented definitions / API declarations:

| Declaration | Purpose |
| --- | --- |
| `FieldHomLawArgs` | Church-encoded field-homomorphism law package containing one `RingHomLawArgs` witness plus inverse, division, and nonzero-preservation laws |

`FieldHomLawArgs` deliberately reuses `RingHomLawArgs` from
`Proofs.Ai.Algebra.AbstractRingFirstIsoBase` instead of duplicating the zero, one, addition,
negation, and multiplication preservation laws. The module imports the existing ring hom bridge and
therefore inherits that bridge's verified import closure, but it introduces no new quotient or image
construction declarations.

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `field_hom_as_ring_hom` | projects the underlying `RingHomLawArgs` witness for downstream ring-first-isomorphism reuse |
| `field_hom_inv_of_nonzero` | projects inverse preservation with explicit source-side `Nonzero` evidence |
| `field_hom_div` | projects division preservation with explicit nonzero evidence for the denominator |
| `field_hom_preserves_nonzero` | projects preservation of source-side `Nonzero` evidence across the homomorphism |

#### `Proofs.Ai.Algebra.AbstractFieldHomKernelImage`

This module is the first direct downstream consumer of `AbstractFieldHom`. It keeps kernel, image,
embedding, and isomorphism construction data explicit instead of introducing concrete quotient or
subtype machinery into the trusted surface.

Implemented definitions / API declarations:

| Declaration | Purpose |
| --- | --- |
| `FieldHomKernelImageArgs` | packages nonzero image preservation and injectivity evidence for a field hom |
| `FieldHomImageFieldArgs` | packages explicit image-carrier field laws plus the image embedding field hom |
| `FieldEmbeddingLawArgs` | packages a field hom together with injectivity evidence |
| `FieldIsoLawArgs` | packages forward/backward field embeddings and inverse laws |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `field_hom_kernel_zero_of_nonzero` | reuses `field_hom_preserves_nonzero` as the kernel-zero exclusion projection |
| `field_hom_injective_of_nonzero` | projects explicit injectivity evidence from `FieldHomKernelImageArgs` |
| `field_hom_image_field_laws` | projects field laws for an explicitly supplied image carrier |
| `field_embedding_as_field_hom` | projects the underlying `FieldHomLawArgs` from an embedding package |
| `field_embedding_comp` | records composition as explicit `FieldEmbeddingLawArgs` evidence |
| `field_iso_symm` | swaps forward and backward embedding evidence for a field isomorphism |
| `field_iso_trans` | records transitivity as explicit composite isomorphism evidence |

#### `Proofs.Ai.Algebra.AbstractFieldIntegralDomain`

This bridge imports `AbstractField` and the existing UFD-style integral-domain API. It does not
make `AbstractField` depend on UFD, and it does not restate field inverse laws in downstream
integral-domain modules.

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `field_no_zero_divisors` | packages `field_mul_eq_zero_cases` as a UFD-style disjunction `a = 0 or b = 0` |
| `field_integral_domain_laws` | builds `IntegralDomainLawArgs` from `FieldLawArgs` for UFD-style downstream reuse |
| `field_nonzero_product_left` | proves product nonzero implies the left factor is nonzero, using ring zero-multiplication laws |
| `field_nonzero_product_right` | proves product nonzero implies the right factor is nonzero, using ring zero-multiplication laws |
| `field_mul_eq_zero_elim` | exposes the field zero-product case split as a direct Prop-level eliminator |

The module uses the repository-wide allowed `Eq.rec` equality-elimination interface through
`EqReasoning` for equality transport in the nonzero-product lemmas.

#### `Proofs.Ai.Algebra.AbstractUfdPrimeFactorization`

Implemented definitions / API declarations:

| Declaration | Purpose |
| --- | --- |
| `UfdFalse`, `UfdNot`, `UfdOr` | UFD-local Church-encoded falsehood, negation, and disjunction |
| `Divides`, `Unit`, `Associate` | Church-encoded divisibility, unit, and associate evidence over explicit multiplication |
| `UfdNonzero`, `Nonunit` | UFD-local element predicates, named separately from field `Nonzero` to keep imports composable |
| `PrimeElement`, `IrreducibleElement` | local element predicates for prime and irreducible behavior |
| `IntegralDomainLawArgs` | integral-domain package extending `RingLawArgs` with `0 != 1` and no zero divisors |
| `FactorizationPred`, `PrimeFactorizationPred` | abstract factorization evidence for an element and its prime-factor refinement |
| `UniqueFactorizationDomainLawArgs` | UFD law package: domain laws, irreducible-factor existence, uniqueness up to the supplied equivalence, and the bridge from irreducible factors to prime factors |
| `UfdPrimeFactorizationTheorem` | bundled theorem shape exposing prime-factor existence, uniqueness, all-prime projection, and erasure back to irreducible factorization |

The factorization object is intentionally abstract because this corpus does not yet define a
canonical list or multiset API. The checked theorem `ufd_prime_factorization_theorem` projects a
UFD law package into a prime-factorization theorem package, so the trusted boundary remains the
canonical certificate and the small checker.

#### `Proofs.Ai.Algebra.AbstractHilbertBasisTheorem`

Implemented definitions / API declarations:

| Declaration | Purpose |
| --- | --- |
| `IdealLawArgs` | ideal closure package over explicit zero, addition, negation, and multiplication |
| `FiniteIdealGeneratingSet`, `FinitelyGeneratedIdeal` | abstract finite generating-family evidence for an ideal |
| `NoetherianRingArgs` | package saying every ideal of the explicit ring is finitely generated |
| `PolynomialExtensionLawArgs` | polynomial-extension law package with polynomial ring laws and constant embedding laws |
| `HilbertBasisConstructionArgs` | leading-coefficient ideal and generator-lifting construction used by the Hilbert basis argument |
| `HilbertBasisTheorem` | bundled theorem exposing base Noetherianity, polynomial ring laws, and Noetherianity of `R[X]` |

The corpus does not yet have canonical polynomial syntax, finite sets, or multiset/list-backed
generating families. This module therefore keeps those objects abstract and certificate-checks the
logical Hilbert-basis extraction: from a Noetherian base ring and a leading-coefficient
construction for polynomial ideals, `hilbert_basis_theorem` returns a checked proof that the
polynomial ring is Noetherian.

#### `Proofs.Ai.Algebra.AbstractHilbertNullstellensatz`

Implemented definitions / API declarations:

| Declaration | Purpose |
| --- | --- |
| `AlgebraicallyClosedFieldArgs` | algebraically closed field law package over an explicit carrier and ring operations |
| `HnsProperIdeal` | Nullstellensatz-local proper-ideal predicate, named separately from Krull's `ProperIdeal` to keep imports composable |
| `ZeroSet`, `HasCommonZero` | common-zero predicate and existence package for an ideal of abstract polynomials |
| `VanishingIdeal` | predicate of polynomials vanishing on every point of `V(I)` |
| `RadicalMember` | abstract positive-power witness package for membership in `sqrt(I)` |
| `PolynomialEvaluationLawArgs` | evaluation law package combining algebraic-closed-field laws, polynomial Noetherianity, and evaluation compatibility |
| `WeakNullstellensatz`, `StrongNullstellensatz` | weak common-zero and strong `I(V(I)) = sqrt(I)` theorem shapes |
| `NullstellensatzConstructionArgs` | construction package for weak common-zero existence and the two radical/vanishing inclusions |
| `HilbertNullstellensatzTheorem` | bundled theorem exposing field laws, polynomial Noetherianity, weak Nullstellensatz, and strong Nullstellensatz |

The corpus still keeps polynomial syntax, affine-space coordinates, and concrete radical powers
outside the trusted core. `hilbert_nullstellensatz_theorem` certificate-checks the extraction from
explicit evaluation laws and a construction package to both the weak proper-ideal common-zero
statement and the strong ideal equality `I(V(I)) = sqrt(I)` encoded by mutual predicate inclusion.

#### `Proofs.Ai.Algebra.AbstractKrullTheorem`

Implemented definitions / API declarations:

| Declaration | Purpose |
| --- | --- |
| `IdealLe` | predicate inclusion between two ideals |
| `ProperIdeal` | properness as non-membership of `1` |
| `MaximalIdeal` | ideal law package, properness, and maximality among proper overideals |
| `MaximalIdealOver` | package saying a maximal ideal contains a given ideal |
| `KrullConstructionArgs` | abstract Zorn-style construction evidence producing a maximal ideal over any proper ideal |
| `KrullTheorem` | bundled theorem exposing the ring laws and the maximal-overideal existence statement |

This module formalizes the common maximal ideal form of Krull's theorem: every proper ideal in a
unital commutative ring is contained in a maximal ideal. Because the corpus has no set-theoretic
chain/Zorn API yet, the Zorn argument is an explicit non-trusted construction package; the checked
certificate verifies only the logical extraction from that construction to the public theorem.

#### `Proofs.Ai.Algebra.AbstractFieldIdeal`

This bridge connects the field law package to ideal and quotient-ring routes while keeping the
non-computational construction evidence explicit. It imports the existing field, ring first
isomorphism, Chinese remainder, Hilbert basis, Hilbert Nullstellensatz, and Krull packages, but it
does not add chain/Zorn machinery, quotient syntax, or hidden global ideal axioms to the trusted
core.

Implemented definitions / API declarations:

| Declaration | Purpose |
| --- | --- |
| `FieldIdealZeroOrTop` | Church-encoded field ideal dichotomy: an ideal is either zero or the whole field |
| `FieldSimpleRingEvidence` | package combining ring laws, zero-or-top ideal laws, and proper-ideal-is-zero evidence |
| `FieldIdealLawArgs` | bridge law package exposing the ideal dichotomy and simple-ring evidence for a field |
| `MaximalIdealQuotientFieldArgs` | explicit quotient/maximal-ideal construction package returning field laws on the quotient |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `field_ideal_zero_or_top` | projects the explicit zero-or-top law for an arbitrary ideal with `IdealLawArgs` |
| `field_simple_ring_evidence` | projects the simple-ring evidence package for downstream ideal arguments |
| `quotient_by_maximal_ideal_is_field` | returns quotient `FieldLawArgs` from explicit `MaximalIdeal`, quotient ring laws, quotient hom, kernel exactness, and `RingFirstIso` evidence |

The module's axiom policy is the existing package-allowed `Eq.rec`, inherited through the imported
equality and quotient/isomorphism route. The theorem statement for
`quotient_by_maximal_ideal_is_field` intentionally exposes all quotient and maximality witnesses so
the Krull and Nullstellensatz trusted boundaries remain unchanged.

#### `Proofs.Ai.Algebra.AbstractOrderedField`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `le`, `lt` | parametric adapters for explicit abstract order relation parameters |
| `sqrt` | parametric adapter for an explicit square-root function parameter |
| `Nonneg` | abbreviation for `le zero a`, useful for square-root APIs |
| `Positive` | abbreviation for `lt zero a`, useful for strict metric statements |
| `OrderedFieldLawArgs` | Church-encoded law package API over the explicit carrier, operations, order, and square root |

The checked theorem targets take the corresponding order, square-root, or compatibility law as an
explicit argument and return it at the requested variables. This keeps P18 independent of concrete
`RingElem`, avoids module-level unchecked ordered-field axioms, and uses P17's parametric `two` and
`sq` APIs. Bundled proposition shapes are Church-encoded locally so P18 does not need an additional
logic-module import.

IPM8 extends `OrderedFieldLawArgs` with `square_completion_bound_law`, a generic scalar/order field
for the quadratic/completed-square step: from nonnegativity of `a * sq t + (2 * b) * t + c` for all
scalar `t`, it yields `sq b <= a * c`. This is deliberately not a vector, inner-product, norm, or
metric theorem. The trusted boundary is unchanged: the source wrapper and replay remain non-trusted
sidecars, and the certificate only verifies that the exported theorem projects this generic scalar
law from the explicit law package supplied by an instantiation.

IPM11 extends the same law package with `le_of_sq_le_sq_nonneg_law`, a generic scalar/order
square-comparison field: from `0 <= a`, `0 <= b`, and `sq a <= sq b`, it yields `a <= b`. The
exported support theorems remain algebraic projections and combinations over the explicit ordered
field package, so they do not depend on vectors, affine points, dot products, Cauchy-Schwarz, or
triangle inequality.

IPM12 adds scalar/order bridges for the squared Minkowski route. `le_mul_sqrt_of_sq_le_mul_nonneg`
turns the squared Cauchy-Schwarz conclusion into a one-sided cross-term bound, and
`add_two_mul_le_sq_add_sqrt` packages the ordered scalar step from
`n = a + 2 * c + b` and `c <= sqrt a * sqrt b` to `n <= sq (sqrt a + sqrt b)`. These are still
generic ordered-field support laws, not vector or metric theorem-shaped assumptions.

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `le_refl`, `le_trans` | order reflexivity and transitivity |
| `add_nonneg`, `mul_nonneg` | nonnegative closure under addition and multiplication |
| `square_nonneg` | `0 <= sq a` |
| `sqrt_nonneg` | `0 <= sqrt a` |
| `sqrt_square_of_nonneg` | `0 <= a -> sqrt (sq a) = a` |
| `sqrt_mul_self` | `0 <= a -> sqrt (a * a) = a` |
| `eq_of_square_eq_square_nonneg` | equality from equal squares under nonnegativity |
| `add_le_add`, `mul_le_mul_nonneg`, `zero_le_two` | order helpers for metric proofs |
| `le_antisymm` | `a <= b -> b <= a -> a = b` |
| `lt_of_le_of_ne` | `0 <= a -> (a = 0 -> False) -> 0 < a`, with `False` Church-encoded |
| `le_of_eq` | equality implies both order directions as a Church-encoded conjunction |
| `sqrt_sq` | `0 <= a -> sq (sqrt a) = a` |
| `sq_eq_zero_iff` | Church-encoded `sq a = 0 <-> a = 0` under the abstract ordered-field assumptions |
| `sum_nonneg_eq_zero` | `0 <= a -> 0 <= b -> a + b = 0 ->` Church-encoded `(a = 0) /\ (b = 0)` |
| `square_completion_bound_from_ordered_args` | projects the generic scalar quadratic/completed-square bound from `OrderedFieldLawArgs` |
| `le_of_sq_le_sq_nonneg_from_ordered_args` | `0 <= a -> 0 <= b -> sq a <= sq b -> a <= b` |
| `add_dist_nonneg_from_ordered_args` | generic nonnegative-sum helper for later metric-distance arguments |
| `sqrt_sum_square_bound_from_ordered_args` | `0 <= a -> 0 <= b -> 0 <= c -> sq a <= sq (b + c) -> a <= b + c` |
| `le_mul_sqrt_of_sq_le_mul_nonneg_from_ordered_args` | `0 <= a -> 0 <= b -> sq c <= a * b -> c <= sqrt a * sqrt b` |
| `add_two_mul_le_sq_add_sqrt_from_ordered_args` | scalar ordered-field step for `a + 2 * c + b <= sq (sqrt a + sqrt b)` |

#### `Proofs.Ai.Algebra.AbstractOrderedFieldFieldBridge`

This split bridge imports `AbstractField` and `AbstractOrderedField` without modifying the existing
`AbstractOrderedField` certificate. Downstream geometry, metric, and inner-product modules can keep
using `OrderedFieldLawArgs`; field-specific consumers may opt into the bridge when they need
`FieldLawArgs` or strict-positivity consequences.

Implemented definitions / API declarations:

| Declaration | Purpose |
| --- | --- |
| `OrderedFieldFieldBridgeArgs` | explicit evidence package containing `FieldLawArgs` plus positive/nonzero, inverse, division, multiplication, and square-positivity laws |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `ordered_field_field_laws` | projects `FieldLawArgs` from the bridge package |
| `ordered_field_nonzero_of_positive` | turns `0 < a` into field-level `Nonzero a` |
| `ordered_field_inv_positive` | projects positivity of inverses |
| `ordered_field_div_positive` | projects positivity of division from positive numerator and denominator |
| `ordered_field_mul_pos` | projects strict positivity under multiplication |
| `ordered_field_sq_pos_of_nonzero` | projects strict positivity of `sq a` from field-level nonzero evidence |

The module introduces no axioms. Its construction evidence remains explicit, and existing
`OrderedFieldLawArgs` theorem names stay in `AbstractOrderedField`.

The field-theory route through `AbstractField`, `AbstractFieldHom`,
`AbstractFieldIntegralDomain`, `AbstractFieldIdeal`, and `AbstractOrderedFieldFieldBridge` is
complete as verified corpus staging. Public `npa-mathlib` materialization is deferred to a separate
closure audit that checks import closure size, axiom policy, statement stability, and compatibility
alias requirements.

#### `Proofs.Ai.Analysis.Sequence.Basic`

This module is the first sequence-convergence layer over `Proofs.Ai.Analysis.Real.Basic`. It stays
abstract over the scalar carrier, complete ordered-field evidence, sequence index type, sequence
function, and the small-radius `NearLimit` predicate. It imports `AbstractFixedPoint` only to expose
checked aliases for the existing `ConvergesTo` and `CauchySeq` surfaces; the fixed-point module is
not modified.

Implemented definitions / API declarations:

| Declaration | Purpose |
| --- | --- |
| `SequenceValue` | Church-encoded evidence that a scalar appears as some term of the sequence |
| `Eventually` | eventual predicate package over the explicit sequence index carrier |
| `Subsequence` | subsequence selector and term-equality predicate |
| `SequenceConvergesTo` | positive-radius convergence predicate using the supplied `NearLimit` relation |
| `SequenceLimit` | public limit vocabulary alias for `SequenceConvergesTo` |
| `BoundedSequenceBy` | lower/upper bound package for all sequence terms |
| `BoundedSequence` | existential-style bounded sequence package hiding the chosen bounds |
| `SequenceBoundedAbove`, `SequenceBoundedBelow` | one-sided boundedness packages for monotone and compactness routes |
| `SequenceMonotoneIncreasing`, `SequenceMonotoneDecreasing` | monotonicity predicates over an explicit index preorder relation |
| `SequenceMonotoneCompletenessEvidence` | explicit package connecting a value set, one-sided boundedness, supremum evidence, and sequence convergence |
| `SequenceSqueezeBounds` | lower/current/upper pointwise bound package for squeeze arguments |
| `SequenceSqueezeConvergenceEvidence` | explicit package containing side-sequence convergence and the squeeze bridge to the middle sequence |
| `NestedIntervalLowerEndpointSet` | lower-endpoint value set used by the interval-nesting completeness route |
| `NestedClosedIntervals` | nested closed-interval package over an explicit index preorder |
| `ShrinkingIntervalLength` | explicit interval-length sequence with convergence to zero |
| `NestedIntervalPoint` | existential-style package for a point contained in every closed interval |
| `NestedIntervalCompletenessEvidence` | explicit package connecting lower endpoint bounds, supremum evidence, shrinking lengths, and an intersection point |
| `SequenceLimitUniquenessEvidence` | local evidence package containing two convergence witnesses and their equality conclusion |
| `FixedPointConvergesToAlias` | bridge alias to `AbstractFixedPoint.ConvergesTo` on the scalar-as-vector instance |
| `FixedPointCauchySeqAlias` | bridge alias to `AbstractFixedPoint.CauchySeq` on the scalar-as-vector instance |
| `SequenceCauchySeq` | positive-radius Cauchy predicate over the explicit sequence and scalar order |
| `SequenceConvergenceChoice` | Church-encoded choice of a sequence limit and convergence witness |
| `SequenceCauchyCompletenessEvidence` | explicit package combining `CompleteOrderedFieldArgs`, fixed-point metric completeness, and bridges back to sequence convergence |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `sequence_value_intro`, `sequence_value_elim` | introduce and eliminate sequence-value evidence |
| `eventually_intro`, `eventually_elim` | package and unpack eventual predicates |
| `subsequence_intro`, `subsequence_term` | build a subsequence witness and project term equality |
| `sequence_converges_to_intro`, `sequence_converges_to_small` | build convergence from positive-radius smallness and project the smallness condition |
| `sequence_limit_def` | definitional equality between `SequenceLimit` and `SequenceConvergesTo` |
| `bounded_sequence_by_intro`, `bounded_sequence_by_lower`, `bounded_sequence_by_upper` | build explicit bounds and project lower/upper inequalities |
| `bounded_sequence_intro`, `bounded_sequence_elim` | hide and recover explicit bound witnesses |
| `sequence_bounded_above_intro`, `sequence_bounded_above_elim` | build and eliminate one-sided upper-bounded sequence evidence |
| `sequence_bounded_below_intro`, `sequence_bounded_below_elim` | build and eliminate one-sided lower-bounded sequence evidence |
| `sequence_bounded_above_from_bounds`, `sequence_bounded_below_from_bounds` | derive one-sided boundedness from a two-sided `BoundedSequenceBy` package |
| `sequence_monotone_increasing_intro`, `sequence_monotone_increasing_apply` | build and apply monotone-increasing sequence evidence |
| `sequence_monotone_decreasing_intro`, `sequence_monotone_decreasing_apply` | build and apply monotone-decreasing sequence evidence |
| `sequence_squeeze_bounds_intro`, `sequence_squeeze_lower_bound`, `sequence_squeeze_upper_bound` | build squeeze bounds and project the lower/upper pointwise inequalities |
| `sequence_squeeze_convergence_evidence_intro` | package side-sequence convergence and the squeeze bridge |
| `sequence_squeeze_converges`, `squeeze_theorem` | derive convergence of the middle sequence from explicit squeeze evidence |
| `nested_interval_lower_endpoint_set_intro`, `nested_interval_lower_endpoint_set_elim` | introduce and eliminate lower-endpoint set membership |
| `nested_closed_intervals_intro`, `nested_closed_intervals_nonempty`, `nested_closed_intervals_contains` | package nested closed intervals and project nonemptiness/nesting |
| `shrinking_interval_length_intro`, `shrinking_interval_length_value`, `shrinking_interval_length_converges` | package interval lengths and project their defining equality/convergence |
| `nested_interval_point_intro`, `nested_interval_point_elim` | package and eliminate a point lying in all closed intervals |
| `nested_interval_completeness_evidence_intro` | package the lower-endpoint nonempty/bounded-above bridges and supremum-to-point bridge |
| `nested_interval_point_from_completeness`, `interval_nesting_theorem` | derive an all-interval point by extracting order completeness and choosing a supremum |
| `sequence_limit_uniqueness_intro` | package local uniqueness evidence from two convergence witnesses and an equality proof |
| `sequence_limit_uniqueness_left`, `sequence_limit_uniqueness_right` | recover the two convergence witnesses from uniqueness evidence |
| `sequence_limit_unique`, `limit_unique` | derive equality of limits from local uniqueness evidence |
| `fixed_point_converges_to_alias_intro`, `fixed_point_converges_to_alias_project` | bridge to and from `AbstractFixedPoint.ConvergesTo` |
| `fixed_point_cauchy_seq_alias_intro`, `fixed_point_cauchy_seq_alias_project` | bridge to and from `AbstractFixedPoint.CauchySeq` |
| `sequence_cauchy_seq_intro`, `sequence_cauchy_seq_small` | build Cauchy evidence from positive-radius smallness and project the smallness condition |
| `sequence_convergence_choice_intro`, `sequence_convergence_choice_elim` | package and eliminate the chosen limit/convergence witness |
| `sequence_cauchy_completeness_evidence_intro` | packages ordered-field completeness, fixed-point metric completeness, and Cauchy/convergence bridge maps |
| `sequence_cauchy_completeness_ordered_field`, `sequence_cauchy_completeness_metric` | project the ordered-field and metric-completeness witnesses from the explicit evidence package |
| `sequence_cauchy_to_fixed_point_cauchy` | converts sequence Cauchy evidence to the fixed-point `CauchySeq` witness through the package bridge |
| `sequence_fixed_point_converges_to_sequence` | converts fixed-point `ConvergesTo` evidence back to `SequenceConvergesTo` |
| `sequence_cauchy_converges_from_completeness` | derives a sequence convergence choice from Cauchy evidence via `CompleteMetricArgs` |
| `sequence_cauchy_convergence_criterion`, `cauchy_convergence_criterion` | stable aliases for later series imports |
| `sequence_monotone_completeness_evidence_intro` | packages the value-set nonempty/bounded-above bridges and supremum-to-limit convergence bridge |
| `sequence_monotone_converges_from_completeness` | derives a sequence convergence choice by extracting order completeness and choosing a supremum |
| `monotone_convergence_theorem` | stable alias for the sequence monotone convergence theorem |

The module has an empty axiom report. Limit uniqueness is not a global law package field: the
checked `sequence_limit_unique` theorem extracts equality from an explicit local evidence package.
The Cauchy criterion is likewise derived through `SequenceCauchyCompletenessEvidence`: the package
must expose the `CompleteOrderedFieldArgs` already in the module context plus a fixed-point
`CompleteMetricArgs` witness and bridge maps, and the checked theorem calls the fixed-point
completeness eliminator rather than assuming the final sequence-convergence choice directly.
The monotone convergence theorem similarly calls `supremum_exists_from_completeness` after
extracting order completeness from `CompleteOrderedFieldArgs`; the theorem takes a separate
`ValueSet` and bridge evidence package instead of assuming supremum existence as a theorem input.
The squeeze theorem is also evidence-driven: `SequenceSqueezeBounds` stores the two pointwise
inequality families, while `SequenceSqueezeConvergenceEvidence` stores convergence of the lower and
upper sequences plus the explicit bridge that turns those bounds into convergence of the middle
sequence.
The interval nesting theorem uses the real `ClosedInterval` API directly. `NestedClosedIntervals`
stores explicit closed-interval nonemptiness and containment maps, `ShrinkingIntervalLength` stores
the length sequence and its convergence to zero, and `nested_interval_point_from_completeness`
chooses a supremum of the lower-endpoint set through `supremum_exists_from_completeness` before
applying the explicit supremum-to-intersection bridge.

#### `Proofs.Ai.Analysis.Sequence.Compactness`

This module is the bounded-sequence compactness layer over
`Proofs.Ai.Analysis.Sequence.Basic`. It keeps the subsequence carrier, selector, extracted
sequence, and extracted convergence predicate explicit, and derives Bolzano-Weierstrass through
the existing interval-nesting theorem rather than introducing a primitive compactness axiom. Its
direct imports are the sequence and real-analysis foundations only; it does not import series or
integration modules.

Implemented definitions / API declarations:

| Declaration | Purpose |
| --- | --- |
| `SubsequenceExtractionEvidence` | Church-encoded package containing the imported `Subsequence` witness for a selector and extracted sequence |
| `ConvergentSubsequenceEvidence` | package combining extraction evidence with convergence of the extracted sequence under its own smallness predicate |
| `BolzanoWeierstrassChoice` | existential-style choice of a convergent subsequence and limit |
| `BolzanoWeierstrassCompletenessEvidence` | explicit bounded-sequence-to-nested-interval route plus a bridge from the interval point to a convergent-subsequence choice |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `subsequence_extraction_evidence_intro`, `subsequence_extraction_evidence_subsequence` | package and recover subsequence extraction witnesses |
| `convergent_subsequence_evidence_intro` | package extraction and convergence witnesses for an extracted sequence |
| `convergent_subsequence_evidence_extraction`, `convergent_subsequence_evidence_converges` | project the extraction and convergence witnesses |
| `bolzano_weierstrass_choice_intro`, `bolzano_weierstrass_choice_elim` | package and eliminate the convergent-subsequence choice |
| `bolzano_weierstrass_completeness_evidence_intro` | package the bounded-sequence interval route and subsequence bridge |
| `bounded_sequence_compactness_from_interval_nesting` | derives a convergent-subsequence choice by calling `interval_nesting_theorem` |
| `bolzano_weierstrass_from_completeness`, `bolzano_weierstrass_theorem` | stable aliases for bounded-sequence Bolzano-Weierstrass imports |

The module has an empty axiom report. The compactness proof path requires explicit evidence that
boundedness supplies nested closed intervals, shrinking lengths, and interval-completeness data;
the checked theorem then calls `interval_nesting_theorem` and passes its intersection point to the
subsequence bridge.

#### `Proofs.Ai.Analysis.Series.Basic`

This module is the first series-convergence layer over `Proofs.Ai.Analysis.Sequence.Basic`.
It keeps the partial-sum construction abstract as an explicit relation between a term sequence
and its partial-sum sequence. Series convergence is therefore not a separate limit theory: the
checked API reduces it to `SequenceConvergesTo` for the partial sums. The module imports sequence
and real-analysis foundations, but does not import continuity, compactness criteria, or integration
modules.

Implemented definitions / API declarations:

| Declaration | Purpose |
| --- | --- |
| `SeriesPartialSums` | abstract relation witnessing that a sequence is the partial-sum sequence of the ambient term sequence |
| `SeriesConvergesTo` | packages partial-sum evidence and convergence of that partial-sum sequence to a limit |
| `SeriesConverges` | existential-style choice of a series limit |
| `SeriesAbsoluteTerms` | abstract relation assigning absolute-value terms without adding a primitive absolute-value function |
| `SeriesAbsolutelyConverges` | convergence of the series formed from explicit absolute-value terms |
| `SeriesTail` | abstract tail relation from a starting index to a tail term sequence |
| `SeriesCauchy` | packages partial-sum evidence and Cauchy evidence for the partial-sum sequence |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `series_partial_sums_intro`, `series_partial_sums_project` | introduce and project partial-sum evidence |
| `series_converges_to_intro` | builds series convergence-to evidence from partial-sum evidence and sequence convergence |
| `series_converges_to_partial_sums`, `series_converges_to_sequence_converges` | project the partial-sum and sequence-convergence witnesses |
| `series_convergence_is_partial_sum_sequence_convergence` | stable reduction theorem from series convergence to sequence convergence of partial sums |
| `series_converges_intro`, `series_converges_elim` | package and eliminate the chosen series limit |
| `series_absolute_terms_intro`, `series_absolute_terms_project` | package and project absolute-term evidence |
| `series_absolutely_converges_intro`, `series_absolutely_converges_terms`, `series_absolutely_converges_series` | package and project absolute convergence |
| `series_tail_intro`, `series_tail_project` | package and project tail evidence |
| `series_cauchy_intro`, `series_cauchy_partial_sums`, `series_cauchy_sequence_cauchy` | package and project the series Cauchy criterion hypotheses |
| `series_cauchy_converges_from_sequence_criterion` | derives series convergence by applying `sequence_cauchy_converges_from_completeness` to partial sums |
| `series_cauchy_convergence_criterion`, `cauchy_series_criterion` | stable aliases for later series criteria modules |

The module has an empty axiom report. Absolute values, finite sums, and tails remain explicit
relations supplied by callers, so later comparison-test modules can choose concrete law packages
without changing the core convergence statements.

#### `Proofs.Ai.Analysis.Series.Criteria`

This module adds the first certificate-backed series criteria layer on top of
`Proofs.Ai.Analysis.Series.Basic`. It keeps absolute values and order assumptions explicit:
there is no primitive absolute-value function and no typeclass search.

Implemented definitions / API declarations:

| Declaration | Purpose |
| --- | --- |
| `SeriesNonnegativeTerms` | explicit nonnegativity evidence for a term sequence |
| `SeriesTermwiseDomination` | explicit pointwise domination relation between two term sequences |
| `SeriesAbsoluteValueTerms` | packages absolute-term evidence with nonnegativity evidence |
| `AbsoluteConvergenceCauchyEvidence` | law package turning absolute convergence evidence into Cauchy evidence for the original partial sums |
| `SeriesComparisonCauchyEvidence` | law package turning explicit nonnegative comparison hypotheses into Cauchy evidence |
| `SeriesAbsoluteComparisonEvidence` | law package turning absolute domination by a convergent majorant into absolute convergence evidence |

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `series_nonnegative_terms_intro`, `series_nonnegative_terms_project` | introduce and project nonnegative-term evidence |
| `series_termwise_domination_intro`, `series_termwise_domination_project` | introduce and project pointwise domination evidence |
| `series_absolute_value_terms_intro`, `series_absolute_value_terms_absolute_terms`, `series_absolute_value_terms_nonnegative` | package and project explicit absolute-value evidence |
| `absolute_convergence_cauchy_evidence_intro`, `absolute_convergence_cauchy_evidence_apply` | package and apply the absolute-convergence-to-Cauchy bridge |
| `absolute_convergence_implies_cauchy` | derives the original series Cauchy criterion from absolute convergence via the explicit bridge |
| `absolute_convergence_implies_convergence` | proves ANQ-009 by applying `cauchy_series_criterion` to the Cauchy evidence obtained from absolute convergence |
| `absolute_convergent_series_converges` | stable alias for downstream criteria modules |
| `series_comparison_cauchy_evidence_intro`, `series_comparison_cauchy_evidence_apply` | package and apply the explicit nonnegative comparison-to-Cauchy bridge |
| `series_absolute_comparison_evidence_intro`, `series_absolute_comparison_evidence_apply` | package and apply the absolute-domination-to-absolute-convergence bridge |
| `comparison_test_nonnegative`, `nonnegative_series_comparison_test` | prove ANQ-010 nonnegative comparison by applying `cauchy_series_criterion` to packaged comparison Cauchy evidence |
| `comparison_test_absolutely_dominated`, `absolutely_dominated_series_comparison_test` | prove ANQ-010 absolute domination comparison via the ANQ-009 absolute-convergence theorem |

The module has an empty axiom report and imports `Series.Basic`, sequence, real, normed-space,
and algebra foundations only. ANQ-009 and ANQ-010 keep all order, absolute-value, and comparison
assumptions in explicit law packages; ratio and root tests are intentionally left for the next
criteria theorem batch.

#### `Proofs.Ai.Algebra.AbstractSquareNormalize`

No new carrier or operation definition lives here. This implemented module provides checked
normalization theorem targets over the P17/P18 abstract scalar APIs.

The checked theorem targets either close by definitional equality (`square_def`,
`mul_self_eq_square`) or take the corresponding normalization/order law as an explicit argument and
return it at the requested variables. This avoids adding unchecked algebra or order axioms while
giving later vector and norm layers stable target names.

| Theorem | Shape / purpose |
| --- | --- |
| `square_def` | `sq a = a * a` in the abstract scalar layer |
| `mul_self_eq_square` | `a * a = sq a` |
| `sq_add` | expansion of `sq (a + b)` |
| `sq_sub` | expansion of `sq (a - b)` |
| `sum_two_squares_comm` | commutation of a sum of two squares |
| `cancel_double_zero_term` | remove a `2 * x` cross term under `x = 0` |
| `sq_zero`, `sq_one`, `sq_neg`, `two_mul` | square and scalar-2 helper lemmas |
| `sq_eq_sq_of_eq_or_neg_eq` | bridge for later square-root equality arguments |
| `sq_add_eq_add_sq_add_two_mul` | normalization-oriented alias for `sq_add` |
| `sq_sub_eq_add_sq_sub_two_mul` | normalization-oriented alias for `sq_sub` |
| `add_sq_eq_zero_iff` | sum of nonnegative squares is zero only when both terms are zero |
| `mul_two_zero_term` | `x = 0 -> 2 * x = 0`, used by later norm expansion |
| `normalize_add_with_zero_cross_term` | scalar-only normal form used by `norm_sq_add_of_dot_zero` |

#### `Proofs.Ai.Algebra.AbstractScalarDerive`

No new carrier or operation definition lives here. This implemented module derives scalar
normalization helpers from the P17 `RingLawArgs` package and `Std.Logic.Eq` equality transport,
while importing the P19 square-normalization layer for the Pythagorean scalar stack.

The checked theorem targets use equality transport through `Proofs.Ai.EqReasoning`, so the module
records the expected `Eq.rec` dependency. They do not accept
`normalize_add_with_zero_cross_term_law`, parallelogram, or polarization laws as direct
theorem-shaped arguments.

| Theorem | Shape / purpose |
| --- | --- |
| `mul_two_zero_term_from_ring_args` | `x = 0 -> 2 * x = 0`, derived from `RingLawArgs` |
| `cancel_double_zero_term_from_ring_args` | `x = 0 -> a + 2 * x = a` |
| `normalize_add_with_zero_cross_term_from_ring_args` | `x = 0 -> (a + 2 * x) + b = a + b` |
| `mul_two_neg_from_ring_args` | `2 * (-x) = -(2 * x)`, used by the law-of-cosines scalar route |
| `add_neg_cross_term_to_sub_sum_from_ring_args` | `(a + -t) + b = (a + b) - t` |
| `law_of_cosines_scalar_rhs_from_ring_args` | `(a + 2 * -x) + b = (a + b) - 2 * x` |
| `two_mul_from_ring_args` | `2 * a = a + a`, derived from distributivity and `1 * a = a` |
| `add_sub_cross_cancel_from_ring_args` | `x + (a - x) = a` |
| `add_pairwise_commute_from_ring_args` | `(a + b) + (c + d) = (a + c) + (b + d)` |
| `add_cross_and_sub_cross_cancel_from_ring_args` | cancels opposite cross terms in the parallelogram scalar sum |
| `parallelogram_scalar_rhs_from_ring_args` | `(a + x + b) + (a - x + b) = 2 * a + 2 * b` |
| `add_middle_to_front_from_ring_args` | `(a + x) + b = x + (a + b)`, the scalar rearrangement used by polarization |
| `polarization_scalar_rhs_from_ring_args` | `2 * d = (nx + 2 * d + ny) - (nx + ny)` for the checked polarization route |

#### `Proofs.Ai.Vector.AbstractSpace`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `Vector` | local abstract vector carrier parameter over the scalar API |
| `vzero`, `vadd`, `vneg`, `smul` | local vector operation parameters |
| `vsub` | parametric vector subtraction helper, defined as `vadd x (vneg y)` |
| `linear_comb2` | helper API for two-term linear combinations, useful for generated proof terms |
| `linear_comb3` | helper API for three-term linear combinations in affine point proofs |
| `VectorSpaceLawArgs` | Church-encoded law package API over the explicit scalar/vector operations |

The checked theorem targets either close by definitional equality (`vec_sub_def`,
`vec_sub_eq_add_neg`, `linear_comb2_ext`, `linear_comb3_ext`) or take the corresponding vector-space
law as an explicit argument and return it at the requested variables. The vector subtraction alias
uses the `vec_` prefix to avoid colliding with P17's scalar `sub_eq_add_neg` declaration in the
kernel import environment.

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `vec_sub_def` | `u - v = u + -v` |
| `vec_add_assoc`, `vec_add_comm`, `vec_add_zero`, `vec_zero_add` | additive vector laws |
| `vec_neg_add_cancel`, `vec_add_neg_cancel` | vector inverse laws |
| `sub_sub_sub_cancel` | `(u - w) - (v - w) = u - v`, used for triangle vertices |
| `vec_sub_self`, `vec_sub_zero`, `vec_add_left_cancel` | vector subtraction and cancellation helpers |
| `smul_add`, `add_smul`, `one_smul`, `mul_smul` | scalar multiplication laws |
| `zero_smul`, `smul_zero` | zero scalar/vector multiplication |
| `neg_smul`, `smul_neg` | scalar multiplication and negation interaction |
| `vec_sub_eq_add_neg` | vector subtraction rewrite alias for search consistency |
| `sub_add_sub_cancel_left` | `(u - w) + (w - v) = u - v` displacement-style cancellation |
| `linear_comb2_ext` | expansion theorem for `linear_comb2` |
| `linear_comb3_ext` | expansion theorem for `linear_comb3` |

#### `Proofs.Ai.Vector.AbstractInnerProduct`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `dot` | parametric wrapper for a local abstract inner-product operation |
| `normSq` | squared norm, defined as `dot v v` |
| `distSq` | vector-level squared distance, defined as `normSq (vsub B A)`; P22 adds point-level `distSqPoints` |
| `PerpVec` | vector-level perpendicular predicate, defined as `dot u v = 0` |
| `InnerProductLawArgs` | Church-encoded law package API for symmetry, bilinearity, norm expansion, and positivity hypotheses |

The checked theorem targets either close by definitional equality (`norm_sq_def`,
`dist_sq_def`, `dot_self_eq_norm_sq`) or take the corresponding inner-product, norm-expansion, or
positivity law as an explicit argument and return it at the requested variables.
`perp_vec_iff_dot_eq_zero` and `norm_sq_zero_iff` use the same Church-encoded iff shape as P16's
`Iff` API without importing `Proofs.Ai.Logic.Iff`, because the current source handoff cannot combine
that module with the abstract algebra imports without duplicating the imported `Eq` declaration.
`cauchy_schwarz` uses the squared form `sq (dot u v) <= normSq u * normSq v`, avoiding a square-root
dependency at this layer.

The theorem rows below are the law-package API surface. Rows that take the corresponding law and
return it at requested variables are legacy target/compatibility wrappers, not the completed checked
derivation paths. The completed checked parallelogram and polarization exports live in
`Proofs.Ai.Vector.AbstractInnerProductDerive` as `parallelogram_law_from_inner_args` and
`polarization_identity_from_inner_args`. IPM10 also adds the checked Cauchy-Schwarz route as
`cauchy_schwarz_from_law_packages`, using generic quadratic nonnegativity support rather than a
direct Cauchy-Schwarz law-package field.

Theorem targets / compatibility wrappers:

| Theorem | Shape / purpose |
| --- | --- |
| `dot_comm` | inner-product symmetry |
| `dot_add_left`, `dot_add_right` | additivity in each argument |
| `dot_neg_left`, `dot_neg_right` | negation in each argument |
| `dot_sub_left`, `dot_sub_right` | subtraction expansion in each argument |
| `norm_sq_def`, `dot_self_eq_norm_sq` | squared norm definition and reverse rewrite |
| `dist_sq_def` | squared distance definition after affine displacement is available |
| `norm_sq_add`, `norm_sq_sub` | squared norm expansion |
| `norm_sq_add_of_dot_zero`, `norm_sq_sub_of_dot_zero` | Pythagorean norm steps under perpendicularity |
| `norm_sq_nonneg` | positivity target |
| `quadratic_norm_nonneg` | generic all-`t` quadratic nonnegativity support used by the checked Cauchy-Schwarz route |
| `cauchy_schwarz` | legacy squared Cauchy-Schwarz target wrapper; the checked route is `cauchy_schwarz_from_law_packages` |
| `parallelogram_law`, `polarization_identity` | legacy peer theorem target wrappers; completed checked exports are `parallelogram_law_from_inner_args` and `polarization_identity_from_inner_args` |
| `perp_vec_iff_dot_eq_zero` | iff-shaped equivalence between `PerpVec u v` and `dot u v = 0` |
| `perp_vec_symm` | vector-level perpendicularity symmetry |
| `norm_sq_zero_iff` | iff-shaped `normSq v = 0 <-> v = 0` under positive-definiteness |
| `dist_sq_nonneg` | `0 <= distSq A B` after affine distance is connected |
| `norm_sq_add_of_perp` | `PerpVec u v -> normSq (u + v) = normSq u + normSq v` |
| `norm_sq_sub_of_perp` | `PerpVec u v -> normSq (u - v) = normSq u + normSq v` |

#### `Proofs.Ai.Vector.AbstractInnerProductDerive`

No new vector or scalar operation definition lives here. This implemented module derives the
norm-expansion path needed by the abstract Pythagorean route and the checked parallelogram route
from `InnerProductLawArgs`, `Proofs.Ai.Algebra.AbstractScalarDerive`, `Proofs.Ai.EqReasoning`, and
`Std.Logic.Eq` equality transport. IPM9 also adds reusable zero-norm Cauchy-Schwarz degenerate
helpers, using `OrderedFieldLawArgs` only to turn equality of scalar endpoints into the requested
order conclusion. IPM10 adds the full squared Cauchy-Schwarz theorem by applying
`square_completion_bound_from_ordered_args` to `InnerProductLawArgs.quadratic_norm_nonneg_law`;
this uniform quadratic route covers both zero and nonzero cases without projecting a direct
Cauchy-Schwarz law.

The checked theorem targets record the expected `Eq.rec` dependency. They do not accept
`norm_sq_add_of_dot_zero_law`, `norm_sq_add_of_perp_law`, `parallelogram_law_law`, or
`polarization_identity_law` as direct theorem-shaped arguments. The IPM9 degenerate
Cauchy-Schwarz helpers use positive-definiteness through `norm_sq_zero_iff_law`. The IPM10 full
Cauchy-Schwarz proof uses `quadratic_norm_nonneg_law`, not a direct Cauchy-Schwarz field. The
IPM12 vector bounds use the checked Cauchy-Schwarz export and scalar square-root/order bridges to
derive the cross-term and squared Minkowski core. The manifest and metadata for this module list
the checked exports with the expected `Eq.rec` axiom report; they do not list any metric triangle
inequality theorem yet.

| Theorem | Shape / purpose |
| --- | --- |
| `norm_sq_add_from_inner_args` | projects the primitive `normSq (x + y)` expansion from `InnerProductLawArgs` |
| `norm_sq_sub_from_inner_args` | projects the primitive `normSq (x - y)` expansion from `InnerProductLawArgs` |
| `parallelogram_law_from_inner_args` | derives `normSq (x + y) + normSq (x - y) = 2 * normSq x + 2 * normSq y` from checked norm expansions and scalar normalization |
| `polarization_identity_from_inner_args` | derives `2 * dot x y = normSq (x + y) - (normSq x + normSq y)` from checked norm expansion and IPM4 scalar normalization |
| `norm_sq_add_of_dot_zero_from_args` | `dot x y = 0 -> normSq (x + y) = normSq x + normSq y` using the P27 scalar rewrite |
| `norm_sq_add_of_perp_from_args` | `PerpVec x y -> normSq (x + y) = normSq x + normSq y` |
| `dot_zero_left_from_law_packages` | derives `dot 0 y = 0` from vector-zero additivity, dot additivity, and scalar cancellation |
| `dot_zero_right_from_law_packages` | derives `dot x 0 = 0` from symmetry plus the left-zero helper |
| `dot_eq_zero_of_norm_sq_zero_left_from_inner_args` | derives `dot x y = 0` from `normSq x = 0` via positive-definiteness |
| `dot_eq_zero_of_norm_sq_zero_right_from_inner_args` | derives `dot x y = 0` from `normSq y = 0` via positive-definiteness |
| `cauchy_schwarz_zero_left_from_law_packages` | proves the Cauchy-Schwarz inequality when the left norm square is zero, without using the direct Cauchy-Schwarz law |
| `cauchy_schwarz_zero_right_from_law_packages` | proves the Cauchy-Schwarz inequality when the right norm square is zero, without using the direct Cauchy-Schwarz law |
| `cauchy_schwarz_from_law_packages` | proves `sq (dot x y) <= normSq x * normSq y` from ordered-field square completion and quadratic inner-product nonnegativity |
| `norm_sq_nonneg_from_inner_args` | projects `0 <= normSq x` from `InnerProductLawArgs` for later order proofs |
| `dot_le_mul_sqrt_norm_sq_from_cauchy` | derives `dot x y <= sqrt(normSq x) * sqrt(normSq y)` from checked Cauchy-Schwarz and scalar square-root order support |
| `norm_sq_add_le_square_sum_norms_from_cauchy` | derives `normSq (x + y) <= sq (sqrt(normSq x) + sqrt(normSq y))` without using triangle inequality |

#### `Proofs.Ai.LinearAlgebra.AbstractSpectralTheorem`

This module records the finite-dimensional spectral theorem in the normal-matrix form: a normal
matrix over a finite-dimensional complex spectral setting has a unitary diagonalization
`A = U D U*`. It is intentionally matrix-level rather than tied to a concrete `n x n` representation:
the finite-dimensional index/basis evidence and the complex eigenvalue construction are explicit
proof inputs, and the kernel only checks the canonical certificate that packages and projects them.

| Declaration | Purpose |
| --- | --- |
| `MatrixMulAssocLaw`, `MatrixLeftUnitLaw`, `MatrixRightUnitLaw` | matrix multiplication associativity and identity laws |
| `MatrixAdjointMulLaw`, `MatrixAdjointInvolutiveLaw` | adjoint compatibility laws used by the star-algebra package |
| `MatrixStarAlgebraLawArgs` | Church-encoded star-algebra laws for matrix multiplication, identity, and adjoint |
| `NormalMatrix` | normality predicate `A A* = A* A` |
| `UnitaryMatrix` | unitary predicate with both `U* U = I` and `U U* = I` |
| `DiagonalMatrix` | abstract diagonal-matrix predicate supplied by the finite-dimensional model |
| `DiagonalizationEquation` | matrix equation `A = U D U*` |
| `UnitaryDiagonalization` | packaged unitary `U`, diagonal `D`, and diagonalization equation |
| `NormalMatrixDiagonalizes` | theorem-shaped choice principle assigning a unitary diagonalization to each normal matrix |
| `SpectralConstructionArgs` | explicit finite-dimensional, complex spectral-field, and normal-to-diagonalization construction evidence |
| `FiniteDimensionalSpectralTheorem` | public theorem package exposing the spectral theorem data |

The final theorem export is `finite_dimensional_normal_matrix_unitarily_diagonalizable`, which
returns a Church-encoded choice of `U` and `D` from a normal matrix, and
`finite_dimensional_spectral_theorem`, which bundles the same result with the finite-dimensional and
complex-field evidence. The module has an empty axiom report; the non-kernel mathematical
construction is represented by `SpectralConstructionArgs` rather than a trusted axiom.

#### `Proofs.Ai.FunctionalAnalysis.AbstractHilbertSpaceSpectralTheorem`

This module records the general Hilbert-space spectral theorem for bounded normal and self-adjoint
operators. For a bounded normal operator it exposes a projection-valued measure `E`, the spectral
integral equation `T = spectral_integral E`, a multiplication-operator model, and a direct-integral
decomposition. For a bounded self-adjoint operator it exposes the same data plus real spectral
support for the projection-valued measure. The analytic measure-theoretic construction is explicit
evidence in `HilbertSpaceSpectralConstructionArgs`; the trusted artifact is the canonical certificate
that packages and projects the theorem data.

| Declaration | Purpose |
| --- | --- |
| `BoundedHilbertOperator` | boundedness predicate wrapper for Hilbert-space operators |
| `NormalHilbertOperator` | normality equation `T T* = T* T` |
| `SelfAdjointHilbertOperator` | self-adjointness equation `T* = T` |
| `ProjectionValuedMeasure` | projection-valued measure law package supplied by the analytic model |
| `RealSupportedProjectionValuedMeasure` | real spectral support predicate for self-adjoint spectral measures |
| `SpectralIntegralEquation` | equation `T = spectral_integral E` abstracting `T = integral z dE(z)` |
| `MultiplicationOperatorModel` | predicate that `T` is represented as multiplication on the spectral model |
| `DirectIntegralDecomposition` | predicate that `T` has the associated direct-integral decomposition |
| `BoundedNormalSpectralData` | Church-encoded package of PVM, spectral integral equation, multiplication model, and direct integral for a bounded normal operator |
| `BoundedSelfAdjointSpectralData` | bounded self-adjoint spectral package, adding real support for the PVM |
| `BoundedNormalSpectralResolution` | choice-style resolution assigning spectral data to every bounded normal operator |
| `BoundedSelfAdjointSpectralResolution` | choice-style resolution assigning real-supported spectral data to every bounded self-adjoint operator |
| `HilbertSpaceSpectralConstructionArgs` | explicit Hilbert/operator laws and analytic spectral construction evidence |
| `HilbertSpaceSpectralTheorem` | public package bundling the normal and self-adjoint spectral resolutions |
| `bounded_normal_operator_spectral_theorem` | final bounded-normal theorem returning spectral data with PVM, multiplication model, and direct integral |
| `bounded_self_adjoint_operator_spectral_theorem` | final bounded-self-adjoint theorem returning real-supported spectral data |
| `hilbert_space_spectral_theorem` | bundled general Hilbert-space spectral theorem package |

Projection lemmas expose the normal and self-adjoint data fields separately, including the PVM,
spectral-integral equation, multiplication model, direct integral, and self-adjoint real-support
field.

The module has an empty axiom report. It deliberately keeps measure theory, functional calculus, and
direct-integral construction evidence outside the trusted kernel while still certificate-checking the
general theorem interface requested by downstream modules.

#### `Proofs.Ai.Geometry.Affine`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `Point` | parametric wrapper for an abstract point carrier |
| `disp` | parametric wrapper for a displacement vector from one point to another |
| `distSqPoints` | point-level squared distance, defined as `normSq (disp A B)` |
| `translate` | point translation API wrapper for later affine law statements |
| `midpoint` | midpoint API wrapper for later right-triangle geometry |
| `collinear` | collinearity predicate wrapper for later geometric sanity lemmas |
| `AffineLawArgs` | Church-encoded law package API for point/vector compatibility hypotheses |

`distSqPoints` is intentionally separate from P21's vector-level `distSq`, so the affine layer can
state point-distance lemmas without colliding with the imported vector-distance declaration. The
checked theorem targets either close by definitional equality (`dist_sq_points_def`) or take the
corresponding affine compatibility law as an explicit argument and return it at the requested
points. `AffineLawArgs` itself keeps only the primitive affine compatibility fields needed by later
derivation layers; the theorem-shaped hypotenuse-vector and point-distance-definition fields were
removed from the law package in P29.

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `disp_self` | `disp A A = 0` |
| `disp_reverse` | `disp B A = -disp A B` |
| `disp_comp` | `disp A C = disp A B + disp B C` |
| `hypotenuse_vector_eq_sub_legs` | express the hypotenuse displacement through two leg displacements |
| `dist_sq_points_def` | `distSqPoints A B = normSq (disp A B)` |
| `point_ext_of_zero_disp` | zero displacement implies point equality |
| `dist_sq_symm` | point squared-distance symmetry |
| `dist_sq_zero_iff_eq` | iff-shaped point squared-distance nondegeneracy |

#### `Proofs.Ai.Geometry.AffineDerive`

No new point, vector, or scalar operation definition lives here. This implemented module derives
the affine orientation path needed by the abstract Pythagorean route from primitive
`AffineLawArgs`, `VectorSpaceLawArgs`, and `Std.Logic.Eq` equality transport.

The checked theorem targets record the expected `Eq.rec` dependency. They do not accept
`hypotenuse_vector_eq_sub_legs_law` or `dist_sq_points_def_law` as direct theorem-shaped
arguments; the hypotenuse orientation is built from `disp_comp`, `disp_reverse`, vector addition
commutativity, and the definitional `vsub` / `distSqPoints` expansions.

| Theorem | Shape / purpose |
| --- | --- |
| `vec_add_comm_from_vector_args` | projects vector addition commutativity from `VectorSpaceLawArgs` |
| `disp_reverse_from_affine_args` | projects the primitive reverse displacement law from `AffineLawArgs` |
| `disp_comp_from_affine_args` | projects the primitive displacement composition law from `AffineLawArgs` |
| `dist_sq_points_def_from_args` | `distSqPoints X Y = normSq (disp X Y)` by definition |
| `hypotenuse_vector_eq_neg_left_add_right_from_args` | `disp B C = vadd (vneg (disp A B)) (disp A C)` |
| `hypotenuse_vector_eq_sub_legs_from_args` | `disp B C = vsub (disp A C) (disp A B)` |
| `dist_sq_hypotenuse_norm_neg_left_add_right_from_args` | rewrites `distSqPoints B C` to the norm of the additive hypotenuse orientation |
| `dist_sq_hypotenuse_norm_sub_legs_from_args` | rewrites `distSqPoints B C` to the norm of the subtraction hypotenuse orientation |

#### `Proofs.Ai.Geometry.AbstractRightTriangle`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `Perp` | vector-level perpendicularity predicate, defined through P21 `PerpVec` |
| `RightTriangle` | point-level right-triangle predicate, with the right angle at `A` |
| `AngleRight` | angle-level predicate wrapper for later APIs that separate angle from triangle |
| `Area2` | doubled-area API wrapper for right-triangle area theorem targets |
| `FootOnHypotenuse` | altitude-foot predicate wrapper for later classical right-triangle targets |

The checked theorem targets either close by definition (`perp_iff_dot_eq_zero`,
`right_triangle_legs_perp`) or take the corresponding perpendicularity, Pythagorean,
law-of-cosines, area, or median law as an explicit argument and return it at the requested points.
`perp_iff_dot_eq_zero` uses the same iff-shaped Church encoding as P21's perpendicularity theorem
target without importing `Proofs.Ai.Logic.Iff`.

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `perp_iff_dot_eq_zero` | iff-shaped equivalence between `Perp u v` and `dot u v = 0` |
| `perp_symm` | `Perp u v -> Perp v u` |
| `right_triangle_legs_perp` | extract perpendicular leg displacement vectors |
| `pythagorean_distance_sq_general` | `RightTriangle A B C -> distSqPoints B C = distSqPoints A B + distSqPoints A C` |
| `law_of_cosines_general` | squared-distance law of cosines over the abstract inner product |
| `right_triangle_area_general`, `median_to_hypotenuse_general` | same-level classical right-triangle targets |

#### `Proofs.Ai.Geometry.AbstractRightTriangleDerive`

No new geometric definition lives here. This implemented module derives the bridge needed by the
abstract Pythagorean route while keeping `RightTriangle A B C` as the public geometric hypothesis.
The key final premise matches P29's additive hypotenuse orientation:
`PerpVec (vneg (disp A B)) (disp A C)`.

The checked theorem targets record the expected `Eq.rec` dependency for equality transport. They do
not accept a theorem argument whose conclusion is already the Pythagorean equality.

| Theorem | Shape / purpose |
| --- | --- |
| `neg_zero_from_ring_args` | derives `-0 = 0` from `RingLawArgs` |
| `dot_neg_left_from_inner_args` | projects `dot (-x) y = -(dot x y)` from `InnerProductLawArgs` |
| `right_triangle_legs_perp_vec_from_rt` | `RightTriangle A B C -> PerpVec (disp A B) (disp A C)` by unfolding |
| `right_triangle_legs_dot_zero_from_rt` | `RightTriangle A B C -> dot (disp A B) (disp A C) = 0` by unfolding |
| `right_triangle_neg_left_dot_zero_from_rt` | derives `dot (-(disp A B)) (disp A C) = 0` |
| `right_triangle_neg_left_perp_vec_from_rt` | final P28 premise `PerpVec (-(disp A B)) (disp A C)` |
| `right_triangle_affine_additive_perp_bridge_from_rt` | packages P29's additive hypotenuse orientation with the matching P28 perpendicular premise |

#### `Proofs.Ai.Geometry.AbstractMetric`

Implemented definitions / API declarations, not proof targets:

| Declaration | Purpose |
| --- | --- |
| `dist` | abstract distance API, defined as `sqrt (distSqPoints A B)` |
| `MetricSpaceLawArgs` | Church-encoded law package for distance definition, nonnegativity, symmetry, zero-distance equivalence, and triangle inequality exports |
| `Ball` | closed-ball API, defined through `dist center x <= radius` |

`MetricSpaceLawArgs` no longer carries a direct `distSqPoints = sq dist` bridge or a direct
metric Pythagorean field. `dist_def` closes by definitional equality. The P32 metric bridge derives
`sq (dist A B) = distSqPoints A B` from the primitive ordered-field `sqrt_sq` field and
`distSqPoints A B >= 0`, with the nonnegativity proof projected from `InnerProductLawArgs`; the
reverse bridge uses the audited `Eq.rec` equality transport. IPM12 adds a checked squared affine
distance bound by transporting the vector squared Minkowski core through affine displacement
composition. IPM13 applies the IPM11 square-comparison helper to that squared bound, using checked
distance nonnegativity from the ordered square-root API, to export the public law-package triangle
inequality. The existing `triangle_inequality` theorem is still an explicit compatibility wrapper.
`distance_zero_iff_eq` uses the same iff-shaped Church encoding as earlier geometry targets rather
than importing `Proofs.Ai.Logic.Iff` directly into this metric layer.

Theorem targets:

| Theorem | Shape / purpose |
| --- | --- |
| `dist_def` | `dist A B = sqrt (distSqPoints A B)` |
| `point_dist_sq_nonneg_from_inner_args` | derives `0 <= distSqPoints A B` from `InnerProductLawArgs` |
| `square_dist_eq_dist_sq_from_law_packages` | derives `sq (dist A B) = distSqPoints A B` from `sqrt_sq` and nonnegativity |
| `dist_sq_eq_square_dist_from_law_packages` | reverses the bridge to `distSqPoints A B = sq (dist A B)` |
| `dist_sq_eq_square_dist` | public bridge alias backed by P32 law-package derivation |
| `dist_sq_points_le_square_sum_dist_from_law_packages` | derives the squared affine bound `distSqPoints A C <= sq (dist A B + dist B C)` from Cauchy-Schwarz, scalar square-root/order support, and `disp_comp` |
| `dist_nonneg_from_ordered_args` | derives `0 <= dist A B` from the ordered-field `sqrt_nonneg` law and the definition of `dist` |
| `triangle_inequality_from_law_packages` | derives `dist A C <= dist A B + dist B C` from IPM12 squared bound and IPM11 square comparison, without using `triangle_inequality_law` |
| `dist_nonneg` | `0 <= dist A B` |
| `distance_symm` | `dist A B = dist B A` |
| `distance_zero_iff_eq` | iff-shaped equivalence between `dist A B = 0` and `A = B` |
| `pythagorean_distance_general` | legacy explicit metric Pythagorean wrapper, not used by the final P32 path |
| `triangle_inequality` | legacy explicit metric-law wrapper; prefer `triangle_inequality_from_law_packages` for the checked route |

#### `Proofs.Ai.Geometry.Pythagorean`

No new API declarations live here. This final module collects the abstract prerequisites and
exports theorem names that users can depend on.

Implemented imports:

| Import | Purpose |
| --- | --- |
| `Std.Logic.Eq` | equality target statements and explicit certificate dependency |
| `Proofs.Ai.EqReasoning` | equality transitivity, symmetry, congruence, and transport helpers |
| `Proofs.Ai.Algebra.AbstractRing`, `Proofs.Ai.Algebra.AbstractOrderedField`, `Proofs.Ai.Algebra.AbstractSquareNormalize`, `Proofs.Ai.Algebra.AbstractScalarDerive` | scalar law packages and checked zero-cross-term normalization |
| `Proofs.Ai.Vector.AbstractSpace`, `Proofs.Ai.Vector.AbstractInnerProduct`, `Proofs.Ai.Vector.AbstractInnerProductDerive` | vector-space, inner-product, and checked perpendicular norm-addition derivations |
| `Proofs.Ai.Geometry.Affine`, `Proofs.Ai.Geometry.AffineDerive` | point displacement API, hypotenuse orientation, and point-distance/norm bridges |
| `Proofs.Ai.Geometry.AbstractRightTriangle`, `Proofs.Ai.Geometry.AbstractRightTriangleDerive` | right-triangle hypotheses and checked perpendicular bridge |
| `Proofs.Ai.Geometry.AbstractMetric` | distance API and metric theorem bridge |

The P31 squared-distance theorem is `pythagorean_distance_sq_from_law_packages`. It composes P29's
hypotenuse distance/norm bridge, P30's `RightTriangle` to perpendicular bridge, P28's perpendicular
norm-addition derivation, and small affine symmetry/reversal bridges in this module. It takes
`RingLawArgs`, `VectorSpaceLawArgs`, `InnerProductLawArgs`, `AffineLawArgs`, and
`RightTriangle A B C`; it does not accept a direct Pythagorean equality law.

`law_of_cosines_sq_from_law_packages` is the completed squared point-distance law of cosines. It
uses the LC4 affine bridge `dist_sq_law_of_cosines_rhs_from_law_packages`, with `RingLawArgs`,
`VectorSpaceLawArgs`, `InnerProductLawArgs`, and `AffineLawArgs`; it does not accept
`law_of_cosines_general` or any theorem-shaped direct law-of-cosines equality argument.
`law_of_cosines_dist_sq_from_law_packages` is the completed squared metric-distance variant. It
composes the squared point-distance theorem with P32's `square_dist_eq_dist_sq_from_law_packages`
and `dist_sq_eq_square_dist_from_law_packages`; it remains a squared statement and does not claim
an unsquared distance theorem or square-root cancellation for the law-of-cosines right-hand side.

`pythagorean_theorem_sq` and `pythagorean_theorem_api_alias` delegate to the checked P31 derivation.
`pythagorean_theorem_dist_sq` now composes P31's squared-distance theorem with P32's metric bridge,
so it no longer accepts a direct metric Pythagorean law. The public
`law_of_cosines_right_angle_specialization` compatibility alias now delegates to
`law_of_cosines_right_angle_specialization_from_law_packages`, which specializes the checked LC5
law-of-cosines theorem through `right_triangle_legs_dot_zero_from_rt` and scalar zero-cross-term
normalization. The converse remains an explicit target until the nondegeneracy and angle APIs are
strong enough. P34 and LC8 leave the unsquared distance and angle-cosine forms unexported: the
current ordered-field and metric layers can justify squared metric-distance equality, but they do
not yet provide a checked square-root cancellation path for the full Pythagorean or
law-of-cosines right-hand side.
The module axiom report is `["Eq.rec"]`; this is the documented equality-recursion exception
inherited from imported equality reasoning and transport lemmas, not a geometry or metric axiom.
`Proofs.Ai.Logic.Iff` is not directly imported here because the current source handoff cannot
combine that module with the abstract geometry imports without duplicating the imported `Eq`
declaration.

LC8 manifest/metadata review: `proofs/manifest.toml` and
`Proofs/Ai/Geometry/Pythagorean/meta.json` list `law_of_cosines_sq_from_law_packages`,
`law_of_cosines_dist_sq_from_law_packages`, and
`law_of_cosines_right_angle_specialization_from_law_packages` as certificate-verified theorem
exports. The Pythagorean module's axiom report remains `["Eq.rec"]`. The legacy direct wrappers
`Proofs.Ai.Geometry.RightTriangle.law_of_cosines` and
`Proofs.Ai.Geometry.AbstractRightTriangle.law_of_cosines_general` remain older theorem targets or
compatibility wrappers, not the completed checked law-of-cosines proof path.

| Theorem | Shape / purpose |
| --- | --- |
| `pythagorean_dist_sq_symm_from_affine_args` | extracts point squared-distance symmetry from `AffineLawArgs` |
| `pythagorean_dist_sq_reverse_norm_neg_from_law_packages` | rewrites `distSqPoints B A` to `normSq (vneg (disp A B))` |
| `pythagorean_left_leg_norm_neg_from_law_packages` | identifies `normSq (vneg (disp A B))` with `distSqPoints A B` |
| `dist_sq_law_of_cosines_rhs_from_law_packages` | checked LC4 affine bridge from point squared distance to the scalar law-of-cosines RHS using `RingLawArgs`, `InnerProductLawArgs`, and `AffineLawArgs` |
| `law_of_cosines_sq_from_law_packages` | completed squared point-distance law of cosines using `RingLawArgs`, `VectorSpaceLawArgs`, `InnerProductLawArgs`, and `AffineLawArgs`, without a direct law-of-cosines wrapper |
| `law_of_cosines_dist_sq_from_law_packages` | completed squared metric-distance law of cosines using LC5 plus P32's `OrderedFieldLawArgs` / `InnerProductLawArgs` metric square bridge |
| `pythagorean_distance_sq_from_law_packages` | checked squared-distance Pythagorean theorem from law packages and `RightTriangle A B C` |
| `pythagorean_theorem_sq` | public squared-distance theorem delegating to the checked P31 derivation |
| `pythagorean_theorem_dist_sq` | squared metric-distance theorem derived from P31 plus the P32 metric bridge |
| `pythagorean_converse_sq` | explicit converse target, not a completed theorem derivation, until the required nondegeneracy and angle API are available |
| `law_of_cosines_right_angle_specialization_from_law_packages` | checked right-angle specialization of LC5 using `RightTriangle` dot-zero and scalar zero-cross-term normalization |
| `law_of_cosines_right_angle_specialization` | public compatibility alias backed by `law_of_cosines_right_angle_specialization_from_law_packages` |
| `pythagorean_theorem_api_alias` | stable alias backed by the checked squared-distance theorem |
| `pythagorean_theorem_dependencies` | documentation theorem or metadata target listing required law packages |

Regenerate the corpus:

```sh
cargo run -p npa-proof-corpus
```

Verify the checked-in corpus:

```sh
cargo test -p npa-proof-corpus
```
