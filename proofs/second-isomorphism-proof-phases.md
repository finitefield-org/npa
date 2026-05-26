# Second Isomorphism Proof Phases

This plan tracks the AI-facing route to the group second isomorphism theorem. As with the first
isomorphism route, source, replay, and metadata are non-trusted sidecars; completed layers are
accepted only through canonical certificates and the certificate verifier.

Target theorem shape:

```text
second_isomorphism:
  for H <= G and N normal in G,
  H / (H ∩ N) is isomorphic to HN / N
```

The current proof route uses predicate-shaped subgroups and inductive/evidence packages rather
than native subtype or record carriers. The intended proof is the standard first-isomorphism route:
define the natural map from `H` into `G / N`, identify its kernel with `H ∩ N`, identify its image
with `HN / N`, and package the resulting isomorphism evidence.

## SI0: Subgroup And Normal Base

Module: `Proofs.Ai.Algebra.AbstractGroupSubgroup`

Status: certificate generated.

Purpose:

- define explicit subgroup and normal-subgroup law packages
- expose projection theorems for identity, multiplication closure, inverse closure, and conjugation
  closure
- keep the statements over predicate-shaped subgroups `S : G -> Prop`

Completed exports:

| Export | Role |
| --- | --- |
| `SubgroupLawArgs` | explicit subgroup law package over a predicate |
| `NormalSubgroupLawArgs` | explicit normal-subgroup law package with subgroup laws and conjugation closure |
| `subgroup_one` | subgroup identity membership projection |
| `subgroup_mul_closed` | subgroup multiplication closure projection |
| `subgroup_inv_closed` | subgroup inverse closure projection |
| `normal_subgroup_laws` | normal subgroup contains subgroup laws |
| `normal_conj_closed` | normal subgroup conjugation closure projection |
| `normal_inv_conj_closed` | inverse-sided conjugation closure, obtained from normality and `inv (inv g) = g` |

Supporting group-algebra exports in `Proofs.Ai.Algebra.AbstractGroup`:

| Export | Role |
| --- | --- |
| `group_left_cancel` | left cancellation derived from the group laws |
| `group_inv_inv` | double-inverse law used to rewrite inverse-sided conjugation |
| `group_conj_slide` | rewrites `k * ((k^-1 * n) * k)` into `n * k` |
| `group_inv_mul_left_reassoc` | rewrites `h^-1 * (h * n)` into `n` |
| `group_product_mul_reassoc` | rewrites product witnesses into canonical `h * n` form for multiplication |
| `group_mul_inv_rev` | derives `(a * b)^-1 = b^-1 * a^-1` in certificate form |
| `group_product_inv_reassoc` | rewrites inverse product witnesses into canonical `h * n` form |
| `group_inv_rel_symm_reassoc` | rewrites inverse normal-relation witnesses for symmetry |
| `group_rel_trans_reassoc` | rewrites composed normal-relation witnesses for transitivity |

## SI1: Intersection Layer

Module: `Proofs.Ai.Algebra.AbstractGroupSubgroup`

Status: certificate generated for the first intersection facts.

Purpose:

- define `H ∩ N` as a Church-encoded predicate
- prove identity, multiplication, and inverse closure for intersections
- prove the key fact that `H ∩ N` is normal in `H` in predicate form: conjugation by an element of
  `H` preserves `H ∩ N`

Completed exports:

| Export | Role |
| --- | --- |
| `SubgroupInterPred` | Church-encoded intersection predicate |
| `subgroup_inter_intro` | introduction rule for intersection membership |
| `subgroup_inter_left`, `subgroup_inter_right` | elimination projections |
| `subgroup_inter_one` | identity lies in the intersection |
| `subgroup_inter_mul_closed` | intersection is closed under multiplication |
| `subgroup_inter_inv_closed` | intersection is closed under inverse |
| `subgroup_inter_normal_in_left` | `H ∩ N` is normal under conjugation by elements of `H` |

## SI2: Product Subgroup Predicate

Module: `Proofs.Ai.Algebra.AbstractGroupSubgroup`

Status: predicate-level certificate generated.

Purpose:

- define `HN` as a Church-encoded product predicate
- expose introduction and elimination for product witnesses
- prove the identity, multiplication, and inverse closure facts for `HN`
- package the closure facts as subgroup law evidence

Completed exports:

| Export | Role |
| --- | --- |
| `SubgroupProductPred` | Church-encoded predicate for elements equal to `h * n` |
| `subgroup_product_intro` | introduction rule with explicit witnesses and equality |
| `subgroup_product_elim` | eliminator for product witnesses |
| `subgroup_product_one` | identity lies in the product predicate |
| `subgroup_product_mul_closed` | product predicate is closed under multiplication when `N` is normal |
| `subgroup_product_inv_closed` | product predicate is closed under inverse when `N` is normal |
| `subgroup_product_laws` | `HN` packaged as `SubgroupLawArgs` |
| `normal_rel_product_right` | product witness `x = h * n` gives the quotient relation `h ~ x` |

Remaining for SI2: no predicate-level closure work remains. Later quotient and image-identification
layers may still require additional compatibility lemmas over this predicate.

## SI3: General Normal Quotient

Status: certificate generated through quotient operations and quotient group laws.

Goal:

- define a quotient carrier for an arbitrary normal subgroup predicate `N`
- prove quotient operation well-definedness, representative computation, and group laws
- keep the quotient primitives under the same `CoreFeature` gates already used by the
  first-isomorphism route

Completed exports:

| Export | Role |
| --- | --- |
| `NormalRel` | relation `a ~ b` induced by `a^-1 * b ∈ N` |
| `normal_rel_refl`, `normal_rel_symm`, `normal_rel_trans` | equivalence proof ingredients for `NormalRel` |
| `normal_rel_of_eq` | equality of representatives gives a `NormalRel` witness |
| `normal_rel_mul_compat`, `normal_rel_inv_compat` | multiplication and inverse compatibility for `NormalRel` |
| `normal_rel_one_of_mem`, `normal_rel_one_to_mem` | conversion between `N h` and the representative-level kernel relation `h ~ 1` |
| `NormalSetoid` | quotient setoid built from `NormalRel` equivalence evidence |
| `NormalQuot`, `NormalQuotMk` | quotient carrier and representative injection |
| `normal_quot_sound` | relation witnesses identify representatives in `NormalQuot` |
| `NormalQuotMul`, `NormalQuotOne`, `NormalQuotInv` | quotient group operations |
| `normal_quot_mul_assoc`, `normal_quot_one_mul`, `normal_quot_mul_one`, `normal_quot_inv_mul`, `normal_quot_mul_inv` | quotient group laws |

Remaining exports: none for the standalone normal quotient layer. The next phase uses this quotient
to build the natural map from `H` into `G/N`.

## SI4: Natural Map From H To G/N

Status: certificate generated for the representative-level natural map.

Goal:

- define the natural map `phi(h) = hN`
- prove representative computation and homomorphism facts in the predicate/evidence style

Completed exports:

| Export | Role |
| --- | --- |
| `SecondIsoPhi` | canonical representative map from `H` evidence to `NormalQuot` |
| `second_iso_phi_mk` | representative computation |
| `second_iso_phi_mul` | multiplication compatibility |
| `second_iso_phi_one`, `second_iso_phi_inv` | identity and inverse compatibility |

Remaining exports: none for the representative-level map. The next phase identifies the kernel of
this map with `H ∩ N`.

## SI5: Kernel Identification

Module: `Proofs.Ai.Algebra.AbstractGroupSecondIsoKernel`

Status: certificate generated for representative-level kernel identification.

Goal:

- define a representative-level kernel predicate for the natural map
- prove that this predicate is sound as equality to the quotient identity in `G/N`
- prove that this kernel predicate is exactly `H ∩ N`

Completed exports:

| Export | Role |
| --- | --- |
| `SecondIsoKernelPred` | representative-level kernel condition, defined as `h ~ 1` for `NormalRel` |
| `second_iso_kernel_sound` | kernel-relation evidence identifies `phi(h)` with the quotient identity |
| `second_iso_kernel_to_inter` | kernel membership implies intersection membership |
| `second_iso_inter_to_kernel` | intersection membership implies kernel membership |

Remaining exports: none for the representative-level kernel predicate. A later packaging phase may
wrap these facts into final second-isomorphism evidence after the image side has been identified.

## SI6: Image Identification

Module: `Proofs.Ai.Algebra.AbstractGroupSecondIsoImage`

Status: certificate generated for representative-level image identification.

Goal:

- prove the image of the natural map is exactly the product quotient `HN / N`

Completed exports:

| Export | Role |
| --- | --- |
| `SecondIsoImagePred` | Church-encoded image predicate for the natural map `H -> G/N` |
| `SecondIsoProductQuotPred` | Church-encoded predicate for quotient elements represented by `HN` |
| `second_iso_image_intro`, `second_iso_image_elim` | introduction and elimination rules for image membership |
| `second_iso_product_quot_intro`, `second_iso_product_quot_elim` | introduction and elimination rules for product-quotient membership |
| `second_iso_image_to_product_quot` | image membership implies product quotient membership |
| `second_iso_product_quot_to_image` | product quotient membership implies image membership |

Remaining exports: none for the predicate-level image/product-quotient identification. The final
phase packages this together with the kernel identification evidence.

## SI7: Final Second-Isomorphism Evidence

Module: `Proofs.Ai.Algebra.AbstractGroupSecondIsoFinal`

Status: certificate generated.

Goal:

- package the certified kernel identification and image/product-quotient identification for the
  natural map `H -> G/N`
- expose a final AI-facing second-isomorphism evidence theorem without adding subtype quotient
  carriers to the kernel surface

Completed exports:

| Export | Role |
| --- | --- |
| `SecondIsoKernelEvidence` | packaged equivalence between the natural-map kernel and `H ∩ N` |
| `SecondIsoImageEvidence` | packaged equivalence between the natural-map image and `HN / N` |
| `SecondIsoTheoremEvidence` | final AI-route evidence target |
| `second_iso_kernel_evidence` | certificate-backed packaged kernel evidence |
| `second_iso_image_evidence` | certificate-backed packaged image evidence |
| `second_isomorphism_theorem_evidence` | certificate-backed second-isomorphism evidence theorem |

Scope note: this is the current AI-facing theorem shape for the second-isomorphism route. It
connects the natural-map kernel with `H ∩ N` and its image with the product quotient predicate
`HN / N`; it does not yet introduce native subgroup quotient carriers or a record-shaped
isomorphism object.

## Completion Evidence

The route is complete when these checks pass:

- generated `.npcert` artifacts for every second-isomorphism module
- `tools/proof-corpus` manifest entries for those modules
- `cargo run -p npa-proof-corpus`
- `cargo test -p npa-proof-corpus`
- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
