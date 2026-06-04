# Number Theory Theorem Cards

Source roadmap: `proofs/number-theory-theorem-proof-roadmap.md`

This file is the `NT-T00` theorem-card inventory for the number-theory proof
roadmap. It is a planning sidecar only. It does not add trusted proof evidence,
axioms, source-free certificate verdicts, or package verification claims.

Proof acceptance remains based only on canonical `.npcert` bytes,
deterministic hashes, and source-free checker verdicts. Source files, replay
files, metadata, theorem indexes, theorem-search sidecars, this document,
roadmaps, tactics, plugins, and AI output are untrusted.

## Card Legend

| Field | Meaning |
| --- | --- |
| Card | Primary roadmap theorem family. |
| Stable id | English identifier used for later source/module naming. |
| Level | Initial target level from the roadmap: `L0 Statement`, `L1 Evidence package`, `L2 Derived certificate`, or `L3 Public closure`. |
| Primary home | Namespace or roadmap that owns the first formalization. |
| Labels | `derived-target`, `bridge-interface`, `conditional`, `conjecture`, `duplicate-alias`, or `promotion`. |
| Gate | First acceptance gate for the card. |

Conjectures and broad program statements are never marked `L2`. They remain
`L0 Statement` cards, or appear only as explicitly conditional assumptions for
future theorem forms.

## Namespace Contract

Concrete entry point: `Proofs.Ai.NumberTheory.Inventory`.

This module is a certificate-backed policy entry point, not a mathematical
number-theory foundation. Its checked declarations are projection theorems over
explicit evidence:

| Checked theorem | Contract evidence it preserves |
| --- | --- |
| `arithmetic_object_structure_policy` | arithmetic objects are ordinary proof-corpus structures supplied by later modules |
| `external_owner_alias_policy` | number-theory aliases point to external owner namespaces instead of duplicating them |
| `bridge_assumption_named_policy` | each `BridgeAxiom`-style development assumption is named explicitly |
| `conjecture_assumption_explicit_policy` | conjectures appear only as explicit statement or conditional-assumption evidence |
| `derived_target_certificate_policy` | derived theorem targets require source-free certificate-verdict evidence |

Namespace ownership rules:

- Arithmetic-owned modules live under `Proofs.Ai.NumberTheory.*` only when the
  roadmap owns the definitions or theorem specialization.
- `Proofs.Ai.EllipticCurve.*`, `Proofs.Ai.ModularForms.*`,
  `Proofs.Ai.Modularity.*`, `Proofs.Ai.GaloisRepresentation.*`,
  `Proofs.Ai.AlgebraicGeometry.*`, and
  `Proofs.Ai.Algebra.AbstractFiniteField` remain external owner namespaces.
- Number-theory modules may import or alias external owner results, but must
  not re-create private duplicates under `Proofs.Ai.NumberTheory`.
- `Nat`, `Int`, divisibility, primality, congruence, residue rings, ideals,
  fields, elliptic curves, modular forms, Galois representations, and
  cryptographic assumptions are ordinary library structures or explicit law
  packages. They are not kernel primitives.
- Bridge assumptions must use localized names, such as
  `ClassFieldBridgeAxiom.*`, and may not be hidden behind `L2` labels.

## Elementary Divisibility Interface

Concrete entry points: `Proofs.Ai.NumberTheory.Elementary` and
`Proofs.Ai.NumberTheory.Divisibility`.

The elementary module records checked projection theorems for explicit `Int`
carriers, `Nat`-to-`Int` translation evidence, positivity evidence, nonzero
evidence, and ordinary arithmetic theorem targets. The divisibility module
records checked theorem targets for `Divides`, sign-normalized divisibility,
normalized antisymmetry evidence, divisor/multiple projections, sign rules,
right-multiplication closure, `Nat`/`Int` translation with positivity
hypotheses, and divisibility simplification as an ordinary theorem target.

These modules remain `L1 Evidence package` interfaces. They do not add
arithmetic automation to the trusted core and do not import prime
factorization, elliptic-curve, or modularity modules.

## Euclidean Division And Descent Interface

Concrete entry points: `Proofs.Ai.NumberTheory.EuclideanDivision` and
`Proofs.Ai.NumberTheory.Descent`.

The Euclidean-division module records checked theorem targets for
quotient/remainder existence, the identity
`dividend = quotient * divisor + remainder`, explicit nonzero divisor
hypotheses, remainder sign and bound hypotheses, quotient and remainder
uniqueness under those bounds, normalized Euclidean division, `Nat`/`Int`
translation, and the separation between mathematical existence and algorithm
extraction. The descent module records finite-descent, well-founded
minimization, no-infinite-descent contradiction, gcd-measure, continued
fraction, Diophantine, and extraction-boundary surfaces.

These declarations are still `L1 Evidence package` interfaces. They do not
assume gcd, prime factorization, elliptic curves, or modularity.

## Gcd And Lcm Normal-Form Interface

Concrete entry points: `Proofs.Ai.NumberTheory.Gcd` and
`Proofs.Ai.NumberTheory.Lcm`.

The gcd module records checked theorem targets for Euclidean-division-backed
gcd existence, left and right divisor projections, greatest-common-divisor
characterization, normalized uniqueness, symmetry, normalized sign convention,
and the normal forms consumed by congruence and Diophantine reduction. The lcm
module records lcm existence from gcd evidence, left and right multiple
projections, least-common-multiple characterization, normalized uniqueness,
normalized sign convention, an explicit `gcd_lcm` product-formula hypothesis,
and the matching congruence and Diophantine normal forms.

These modules still do not assume Bezout, Euclid's lemma, Gauss's lemma, prime
factorization, CRT, elliptic curves, or modularity.

## Euclid Algorithm And Bezout Interface

Concrete entry points: `Proofs.Ai.NumberTheory.EuclideanAlgorithm` and
`Proofs.Ai.NumberTheory.Bezout`.

The Euclidean-algorithm module records checked theorem targets for descent-
based termination, remainder-step preservation of gcd evidence, algorithmic gcd
correctness, extended Euclidean algorithm correctness, and an explicit boundary
separating correctness statements from runtime-complexity claims. The Bezout
module records Bezout identities from extended Euclid evidence, linear-
combination witnesses for gcd, the gcd linear-combination characterization,
integer and natural `Coprime` iff linear-combination-equals-one surfaces, and
`Nat`/`Int` coprimality translation.

These modules import the gcd/Euclidean-algorithm closure only. They do not
import prime factorization, CRT, Euclid's lemma, Gauss's lemma, elliptic curves,
or modularity.

## Prime And Composite Predicate Interface

Concrete entry points: `Proofs.Ai.NumberTheory.Prime` and
`Proofs.Ai.NumberTheory.Composite`.

The prime module records checked theorem targets for natural-number and integer
prime predicate surfaces, integer unit and associated predicates, sign-
normalized primes, `1`-is-not-prime surfaces, trivial-divisor characterizations,
`Nat`/`Int` prime translation, and the terminology boundary with UFD-local
`PrimeElement`. The composite module records natural-number and integer
composite predicate surfaces, nontrivial-divisor projections, sign-normalized
composite forms, `Nat`/`Int` composite translation, and factor-extraction input
surfaces for later `UfdBridge` and `Factorization` modules.

These modules do not assume unique factorization, Euclid's lemma, prime
factorization, CRT, elliptic curves, analytic prime distribution, or modularity.

## Theorem Cards

| Card | Stable id | Level | Primary home | Labels | Gate |
| --- | --- | --- | --- | --- | --- |
| `NT-00` inventory and statement policy | `number_theory_inventory_policy` | `L0 Statement` | `Proofs.Ai.NumberTheory.Inventory` | statement-policy | `rg -n "NT-00|NT-24|Riemann hypothesis|Birch|Langlands|sidecar" proofs`; `git diff --check` |
| `NT-01` integers, divisibility, and Euclidean division | `integer_divisibility_euclidean_division` | `L1 Evidence package` until `Nat`/`Int` APIs are stable, then `L2 Derived certificate` | `Proofs.Ai.NumberTheory.Elementary`, `Proofs.Ai.NumberTheory.Divisibility`, `Proofs.Ai.NumberTheory.EuclideanDivision` | derived-target | `--build-module Proofs.Ai.NumberTheory.EuclideanDivision`; `--module Proofs.Ai.NumberTheory.EuclideanDivision` |
| `NT-02` gcd, lcm, Euclid algorithm, and Bezout | `gcd_lcm_euclid_bezout` | `L1 Evidence package` until `NT-01` lands, then `L2 Derived certificate` | `Proofs.Ai.NumberTheory.Gcd`, `Proofs.Ai.NumberTheory.Bezout`, `Proofs.Ai.NumberTheory.EuclideanAlgorithm` | derived-target | `--build-module Proofs.Ai.NumberTheory.Bezout`; `--module Proofs.Ai.NumberTheory.Bezout` |
| `NT-03` primes and unique factorization | `prime_unique_factorization` | `L2 Derived certificate` after `NT-02` and UFD bridge | `Proofs.Ai.NumberTheory.Prime`, `Proofs.Ai.NumberTheory.Factorization`, `Proofs.Ai.NumberTheory.UfdBridge` | derived-target | `--build-module Proofs.Ai.NumberTheory.Factorization`; `--module Proofs.Ai.NumberTheory.Factorization` |
| `NT-04` congruences, residue rings, and Chinese remainder | `congruence_residue_ring_chinese_remainder` | `L2 Derived certificate` after quotient and CRT prerequisites | `Proofs.Ai.NumberTheory.Congruence`, `Proofs.Ai.NumberTheory.ResidueRing`, `Proofs.Ai.NumberTheory.ChineseRemainder` | derived-target | `--build-module Proofs.Ai.NumberTheory.ChineseRemainder`; `--module Proofs.Ai.NumberTheory.ChineseRemainder` |
| `NT-05` Fermat, Euler, Wilson, Carmichael, and RSA | `finite_unit_group_fermat_euler_wilson_rsa` | `L2 Derived certificate` for algebraic correctness; security claims stay `L0` | `Proofs.Ai.NumberTheory.ModularGroup`, `Proofs.Ai.NumberTheory.FermatEulerWilson`, `Proofs.Ai.NumberTheory.Carmichael`, `Proofs.Ai.NumberTheory.Rsa` | derived-target, conditional | `--build-module Proofs.Ai.NumberTheory.Rsa`; `--module Proofs.Ai.NumberTheory.Rsa` |
| `NT-06` primitive roots, characters, and Gauss sums | `primitive_roots_characters_gauss_sums` | `L1 Evidence package`, promoting bounded cyclic-group facts to `L2` | `Proofs.Ai.NumberTheory.PrimitiveRoot`, `Proofs.Ai.NumberTheory.Character`, `Proofs.Ai.NumberTheory.GaussSum` | bridge-interface, derived-target | `--build-module Proofs.Ai.NumberTheory.Character`; `--module Proofs.Ai.NumberTheory.Character` |
| `NT-07` quadratic residues and reciprocity | `quadratic_residue_reciprocity` | `L2 Derived certificate` after finite cyclic group and Legendre/Jacobi APIs | `Proofs.Ai.NumberTheory.QuadraticResidue`, `Proofs.Ai.NumberTheory.Legendre`, `Proofs.Ai.NumberTheory.Jacobi`, `Proofs.Ai.NumberTheory.QuadraticReciprocity` | derived-target | `--build-module Proofs.Ai.NumberTheory.QuadraticReciprocity`; `--module Proofs.Ai.NumberTheory.QuadraticReciprocity` |
| `NT-08` arithmetic functions and Dirichlet convolution | `arithmetic_functions_dirichlet_convolution` | `L2 Derived certificate` for finite divisor algebra; analytic Euler products start `L1` | `Proofs.Ai.NumberTheory.ArithmeticFunction`, `Proofs.Ai.NumberTheory.DirichletConvolution`, `Proofs.Ai.NumberTheory.Mobius` | derived-target, bridge-interface | `--build-module Proofs.Ai.NumberTheory.Mobius`; `--module Proofs.Ai.NumberTheory.Mobius` |
| `NT-09` continued fractions, Pell, and Diophantine approximation | `continued_fraction_pell_approximation` | `L2 Derived certificate` for rational/Pell core; metric approximation starts `L1` | `Proofs.Ai.NumberTheory.ContinuedFraction`, `Proofs.Ai.NumberTheory.Pell`, `Proofs.Ai.NumberTheory.DiophantineApproximation` | derived-target, bridge-interface | `--build-module Proofs.Ai.NumberTheory.Pell`; `--module Proofs.Ai.NumberTheory.Pell` |
| `NT-10` Diophantine equations and additive number theory | `diophantine_additive_number_theory` | `L2 Derived certificate` for small classifications; Waring and additive-combinatorics surfaces start `L1` | `Proofs.Ai.NumberTheory.Diophantine`, `Proofs.Ai.NumberTheory.SumsOfSquares`, `Proofs.Ai.NumberTheory.Additive` | derived-target, bridge-interface | `--build-module Proofs.Ai.NumberTheory.Additive`; `--module Proofs.Ai.NumberTheory.Additive` |
| `NT-11` analytic number theory foundations | `analytic_number_theory_foundations` | `L1 Evidence package` for analytic continuation, zero-free regions, and PNT interfaces | `Proofs.Ai.NumberTheory.DirichletSeries`, `Proofs.Ai.NumberTheory.Zeta`, `Proofs.Ai.NumberTheory.DirichletL`, `Proofs.Ai.NumberTheory.PrimeNumberTheorem` | bridge-interface, conditional | `--build-module Proofs.Ai.NumberTheory.Zeta`; `--module Proofs.Ai.NumberTheory.Zeta` |
| `NT-12` sieve methods and circle method | `sieve_circle_method` | `L1 Evidence package` until finite-sum and asymptotic APIs are certified | `Proofs.Ai.NumberTheory.Sieve`, `Proofs.Ai.NumberTheory.CircleMethod`, `Proofs.Ai.NumberTheory.AdditivePrime` | bridge-interface | `--build-module Proofs.Ai.NumberTheory.Sieve`; `--module Proofs.Ai.NumberTheory.Sieve` |
| `NT-13` algebraic number theory | `algebraic_number_theory` | `L1 Evidence package` first, with algebraic sublemmas promoted to `L2` | `Proofs.Ai.NumberTheory.AlgebraicInteger`, `Proofs.Ai.NumberTheory.NumberField`, `Proofs.Ai.NumberTheory.DedekindDomain`, `Proofs.Ai.NumberTheory.ClassGroup` | bridge-interface, derived-target | `--build-module Proofs.Ai.NumberTheory.AlgebraicInteger`; `--module Proofs.Ai.NumberTheory.AlgebraicInteger` |
| `NT-14` local fields and p-adic analysis | `local_fields_padic_analysis` | `L1 Evidence package`; valuation algebra can become `L2` before completions | `Proofs.Ai.NumberTheory.Valuation`, `Proofs.Ai.NumberTheory.Padic`, `Proofs.Ai.NumberTheory.LocalField`, `Proofs.Ai.NumberTheory.Hensel` | bridge-interface, derived-target | `--build-module Proofs.Ai.NumberTheory.Hensel`; `--module Proofs.Ai.NumberTheory.Hensel` |
| `NT-15` class field theory | `class_field_theory` | `L1 Evidence package` with named reciprocity bridge assumptions | `Proofs.Ai.NumberTheory.ClassField.Local`, `Proofs.Ai.NumberTheory.ClassField.Global`, `Proofs.Ai.NumberTheory.ArtinReciprocity` | bridge-interface, conditional | `--build-module Proofs.Ai.NumberTheory.ClassField.Global`; `--module Proofs.Ai.NumberTheory.ClassField.Global` |
| `NT-16` elliptic curves | `elliptic_curve_number_theory` | `L1 Evidence package` first; bounded group-law lemmas may become `L2`; BSD stays `L0` | `Proofs.Ai.EllipticCurve.*` | bridge-interface, conditional, conjecture | `--build-module Proofs.Ai.EllipticCurve.Basic`; `--module Proofs.Ai.EllipticCurve.Basic` |
| `NT-17` modular forms and modularity | `modular_forms_modularity` | `L1 Evidence package`; modularity-lifting surfaces are explicit interfaces | `Proofs.Ai.ModularForms.*`, `Proofs.Ai.Modularity.*` | bridge-interface, conditional | `--build-module Proofs.Ai.ModularForms.Basic`; `--module Proofs.Ai.ModularForms.Basic` |
| `NT-18` L-functions and Langlands interfaces | `l_functions_langlands_interfaces` | `L1 Evidence package` for proved interface fragments; broad Langlands functoriality stays `L0` | `Proofs.Ai.NumberTheory.LFunction`, `Proofs.Ai.NumberTheory.ArtinL`, `Proofs.Ai.NumberTheory.HeckeL`, `Proofs.Ai.NumberTheory.AutomorphicL`, `Proofs.Ai.Langlands.Interface` | bridge-interface, conjecture, conditional | `--build-module Proofs.Ai.Langlands.Interface`; `--module Proofs.Ai.Langlands.Interface` |
| `NT-19` arithmetic geometry | `arithmetic_geometry_number_theory` | `L1 Evidence package`; Weil and Faltings-level results are interfaces until closure exists | `Proofs.Ai.ArithmeticGeometry.*` | bridge-interface, conditional | `--build-module Proofs.Ai.ArithmeticGeometry.Schemes`; `--module Proofs.Ai.ArithmeticGeometry.Schemes` |
| `NT-20` Iwasawa theory | `iwasawa_theory` | `L1 Evidence package`; main conjectures are `L0` or conditional theorem assumptions | `Proofs.Ai.NumberTheory.Iwasawa.Basic`, `Proofs.Ai.NumberTheory.Iwasawa.MainConjecture`, `Proofs.Ai.NumberTheory.Iwasawa.EulerSystem` | bridge-interface, conjecture, conditional | `--build-module Proofs.Ai.NumberTheory.Iwasawa.Basic`; `--module Proofs.Ai.NumberTheory.Iwasawa.Basic` |
| `NT-21` Galois representations and density theorems | `galois_representations_density` | `L1 Evidence package`; Chebotarev is an interface until algebraic-number closure is certified | `Proofs.Ai.GaloisRepresentation.*`, `Proofs.Ai.NumberTheory.Chebotarev`, `Proofs.Ai.NumberTheory.Frobenius`, `Proofs.Ai.GaloisCohomology.Basic` | bridge-interface, duplicate-alias | `--build-module Proofs.Ai.NumberTheory.Chebotarev`; `--module Proofs.Ai.NumberTheory.Chebotarev` |
| `NT-22` computational number theory and cryptography | `computational_number_theory_cryptography` | `L2 Derived certificate` for algebraic correctness where algorithms exist; hardness/security assumptions stay `L0` | `Proofs.Ai.NumberTheory.Algorithm`, `Proofs.Ai.NumberTheory.PrimalityTest`, `Proofs.Ai.Cryptography.NumberTheory`, `Proofs.Ai.Cryptography.EllipticCurve` | derived-target, conditional, duplicate-alias | `--build-module Proofs.Ai.Cryptography.NumberTheory`; `--module Proofs.Ai.Cryptography.NumberTheory` |
| `NT-23` finite fields and combinatorial number theory | `finite_field_applications_combinatorial_number_theory` | `L1 Evidence package` for applications; finite-field core aliases point to field theory | `Proofs.Ai.Algebra.AbstractFiniteField`, `Proofs.Ai.NumberTheory.FiniteFieldApplications`, `Proofs.Ai.NumberTheory.ExponentialSum`, `Proofs.Ai.NumberTheory.Combinatorial` | duplicate-alias, bridge-interface, derived-target | `--build-module Proofs.Ai.NumberTheory.FiniteFieldApplications`; `--module Proofs.Ai.NumberTheory.FiniteFieldApplications` |
| `NT-24` packaging and promotion | `number_theory_packaging_promotion` | `L3 Public closure` only after local closure, package checks, and promotion audit | `Proofs.Ai.NumberTheory.*`, selected external owner namespaces, and `npa-mathlib` closure sidecars | promotion | `./scripts/check-corpus-authoring.sh`; package/full gates only for promotion or high-trust release work |

## Duplicate-Home Map

| Theorem family | Primary home | Number-theory role | Decision |
| --- | --- | --- | --- |
| Abstract group, ring, field, and module laws | Existing `Proofs.Ai.Algebra.*`, `Proofs.Ai.Vector.*`, and linear-algebra roadmap modules | Import and specialize to arithmetic structures | Do not duplicate algebra foundations under `Proofs.Ai.NumberTheory`. |
| Abstract Chinese remainder theorem | `Proofs.Ai.Algebra.AbstractRingChineseRemainder` | Specialize to integer residue rings in `NT-04` | Number theory owns integer/residue-ring specialization only. |
| UFD prime factorization | `Proofs.Ai.Algebra.AbstractUfdPrimeFactorization` plus field-theory roadmap where needed | Bridge to natural/integer factorization in `NT-03` | Fundamental theorem of arithmetic must be derived from explicit bridge data. |
| Finite-field core construction, Frobenius, roots, and subfields | `Proofs.Ai.Algebra.AbstractFiniteField` and `develop/proof-corpus-field-theory-roadmap.md` | Import or alias for primitive roots, exponential sums, and finite-field applications | Number theory may own applications, not the finite-field core. |
| Finite cyclic groups and unit-group order facts | Existing/future algebra group modules | Specialize to units modulo `n` in `NT-05` and `NT-06` | Keep cyclic-group facts abstract before arithmetic specialization. |
| Real/complex analysis, series, integration, Tauberian theorems | Analysis, topology, and measure roadmaps | Supply analytic prerequisites for zeta, `L`-functions, PNT, sieve, and circle method | Number theory records analytic theorem interfaces until dependencies are certified. |
| Chebotarev density and Frobenius data | `Proofs.Ai.NumberTheory.Chebotarev`, `Proofs.Ai.NumberTheory.Frobenius`, and `Proofs.Ai.GaloisRepresentation.*` | Used later for Dirichlet-from-Chebotarev and density routes | Chebotarev does not prove elementary prime facts or FTA. |
| Elliptic-curve foundations | `Proofs.Ai.EllipticCurve.*` | Number-theory roadmap tracks dependencies and arithmetic applications | Do not re-home group law, height, or reduction APIs under number theory. |
| Modularity and modular forms | `Proofs.Ai.ModularForms.*` and `Proofs.Ai.Modularity.*` | Number theory consumes modularity surfaces for advanced arithmetic results | Keep Wiles/Taylor-Wiles/Ribet interfaces explicit and localized. |
| Langlands interfaces | `Proofs.Ai.Langlands.Interface` plus automorphic and `L`-function namespaces | Number theory records `L`-function and reciprocity connections | Broad functoriality stays an interface/conjecture graph, not an `L2` target. |
| Algebraic and arithmetic geometry | Existing `Proofs.Ai.AlgebraicGeometry.*` and `Proofs.Ai.ArithmeticGeometry.*` | Number theory imports scheme/cohomology/rational-point interfaces | Weil and Faltings-level theorem families remain interface-level until their closure is audited. |
| Cryptographic security assumptions | `Proofs.Ai.Cryptography.*` or future cryptography roadmap | Number theory proves algebraic correctness only | Hardness/security assumptions stay assumptions or `L0` statement cards. |
| Additive combinatorics over finite groups/fields | Number theory with algebra and finite-field imports; future combinatorics roadmap may own generic variants | Number theory owns arithmetic applications | Ambient group/field assumptions must be explicit. |

## Conjecture And Conditional Status Map

| Statement family | Card | Status | Rule |
| --- | --- | --- | --- |
| Riemann hypothesis | `NT-11`, `NT-18` | `L0 Statement` or explicit conditional assumption | Never mark as `L2`; only conditional theorem forms may assume it. |
| Generalized Riemann hypothesis | `NT-11`, `NT-18` | `L0 Statement` or explicit conditional assumption | Keep separate from proved zero-free-region interfaces. |
| Birch and Swinnerton-Dyer conjecture | `NT-16` | `L0 Statement` or explicit conditional assumption | BSD is not evidence for elliptic-curve group law, height, or modularity surfaces. |
| Broad Langlands functoriality and correspondence statements | `NT-18` | `L0 Statement` or `L1` named interface fragments | Individual proved fragments may be promoted only after source-free closure. |
| Iwasawa main conjectures before certified route | `NT-20` | `L0 Statement`, `L1 Evidence package`, or conditional theorem assumption by exact formulation | Do not export conjecture assumptions from derived public closures. |
| Artin conjecture and Fontaine-Mazur conjecture | `NT-18`, `NT-21` | `L0 Statement` or explicit conditional assumption | Keep distinct from Artin reciprocity and verified representation lemmas. |
| Twin prime, Goldbach, and broad Hardy-Littlewood conjectures | `NT-12` | `L0 Statement` unless replaced by proved bounded-gap or weak-Goldbach interfaces | Do not treat sieve/circle-method interfaces as conjecture proofs. |
| Cryptographic hardness assumptions | `NT-22` | `L0 Assumption` for security statements | Algebraic correctness theorems may be `L2`; security claims may not. |

## Acceptance Checklist

- Every roadmap family `NT-00` through `NT-24` has one primary card above.
- Finite fields, Chebotarev, modularity, Langlands, elliptic curves,
  algebraic geometry, cryptography, and analytic number theory have
  duplicate-home decisions.
- Conjectures, conditional theorem forms, bridge interfaces, and derived
  theorem targets are labeled separately.
- No card treats source, replay, theorem indexes, AI output, metadata,
  sidecars, or this document as proof evidence.
- The namespace contract above names the checked entry point and keeps
  arithmetic structures, external owner aliases, bridge assumptions, and
  conjectures explicit.
