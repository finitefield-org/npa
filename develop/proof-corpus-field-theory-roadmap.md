# Proof Corpus Field Theory Roadmap

Date: 2026-06-03

この文書は、proof corpus に体論の定理を追加していくためのロードマップです。
これは計画文書であり、証明受理の根拠ではありません。
公開 `npa-mathlib` closure 全体の次リリース順を上書きするものではなく、
体論ルートを増やす場合の局所的な追加順を記録します。

NPA の信頼境界は変えません。

```text
信頼しない:
  この文書
  source.npa
  replay.json
  meta.json
  theorem index
  AI が生成した証明候補

信頼する:
  canonical .npcert
  deterministic hash
  kernel / certificate verifier verdict
  source-free independent checker verdict
```

## 1. 現状

既存 corpus では、群論と環論の基礎はすでに厚くなっています。
特に次の層は体論の土台として使えます。

```text
Proofs.Ai.Algebra.AbstractGroup
Proofs.Ai.Algebra.AbstractGroupImage
Proofs.Ai.Algebra.AbstractGroupQuotient
Proofs.Ai.Algebra.AbstractGroupQuotientMul
Proofs.Ai.Algebra.AbstractGroupQuotientGroup
Proofs.Ai.Algebra.AbstractGroupQuotientHom
Proofs.Ai.Algebra.AbstractRing
Proofs.Ai.Algebra.AbstractRingFirstIsoBase
Proofs.Ai.Algebra.AbstractRingFirstIso
Proofs.Ai.Algebra.AbstractRingChineseRemainder
Proofs.Ai.Algebra.AbstractOrderedField
```

FT-01 以前は、`AbstractOrderedField` が順序・平方根 law bundle 中心であり、
体そのものの逆元構造は共通モジュールとして切り出されていませんでした。
現在は FT-01 から FT-07 までで、`AbstractField` foundation、field hom、
field-to-integral-domain、field ideal / quotient、ordered-field bridge までを
corpus 側の verified staging として追加済みです。
この文書の前半は完了済みの基礎 route を、後半はその上に積む高度な体論 route を扱います。

## 2. 基本方針

基礎 route では、いきなり代数閉体・Nullstellensatz・拡大体へ進めず、
`AbstractRing` と `AbstractOrderedField` の間に小さく再利用可能な層を作りました。
高度な体論 route でも、existence theorem を trusted axiom として増やさず、
明示的な evidence package と projection theorem を段階的に追加します。

設計方針:

- `FieldLawArgs` は明示的な law package として持つ。
- `inv` / `div` / `Nonzero` を core calculus に入れず、通常の carrier 上の演算・述語として扱う。
- `zero_ne_one` や `a != 0` は決定可能 bool にせず、既存 corpus と同じく Prop-level の否定証拠で表す。
- `div` は primitive にせず、まず `mul a (inv b)` への projection theorem を置く。
- proof corpus authoring では便利な source / replay を使ってよいが、受理根拠は checked certificate に限定する。
- 既存の `AbstractOrderedField` はすぐに破壊的に作り替えず、下流 module への影響を見ながら `AbstractField` への橋渡しを追加する。

## 3. 基礎 route の優先順位

### 3.1 AbstractField foundation

実装済み module:

```text
Proofs.Ai.Algebra.AbstractField
```

実装済み public surface:

```text
Nonzero
div
FieldLawArgs
field_ring_laws
field_zero_ne_one
field_inv_mul_cancel
field_mul_inv_cancel
field_div_eq_mul_inv
```

目的:

- `AbstractRing` の law package を前提に、体固有の law だけを追加する。
- 逆元と除法の基本 projection theorem を theorem search しやすい名前で固定する。
- 後続の線形代数、幾何、解析、環論上位定理が「体」を明示的に要求できるようにする。

### 3.2 Basic field calculation lemmas

定理探索と rewrite に効く小さい補題を追加済みです。

実装済み theorem:

```text
field_inv_one
field_div_one
field_div_self_nonzero
field_zero_div
field_mul_left_cancel_nonzero
field_mul_right_cancel_nonzero
field_nonzero_mul_closed
field_mul_eq_zero_cases
```

目的:

- 除法・逆元を含む等式変形を、後続 module が毎回 law argument から展開しなくてよいようにする。
- `Nonzero` の閉性と cancellation を定理検索可能にする。
- `field_div_self_nonzero` と cancellation を、線形代数のスカラー正規化に使える形で置く。

### 3.3 Field homomorphism bridge

既存の `RingHomLawArgs` と接続する module を追加済みです。

実装済み module:

```text
Proofs.Ai.Algebra.AbstractFieldHom
```

実装済み theorem:

```text
FieldHomLawArgs
field_hom_as_ring_hom
field_hom_inv_of_nonzero
field_hom_div
field_hom_preserves_nonzero
```

目的:

- 環準同型の first isomorphism route と体準同型を接続する。
- 逆元保存と除法保存を、環準同型の乗法保存から毎回作らずに再利用できるようにする。
- 後続の体同型・埋め込み・部分体 API の足場にする。

### 3.4 Field as integral domain

体から整域性を取り出す層を追加済みです。

実装済み module:

```text
Proofs.Ai.Algebra.AbstractFieldIntegralDomain
```

実装済み theorem:

```text
field_no_zero_divisors
field_integral_domain_laws
field_nonzero_product_left
field_nonzero_product_right
field_mul_eq_zero_elim
```

目的:

- `AbstractUfdPrimeFactorization` など環論上位層へ、体を整域として渡せるようにする。
- `a * b = 0` から片方が 0 であることを使う証明を共通化する。

### 3.5 Field ideals and quotient bridge

環論の商・イデアル定理と接続する上位層です。

実装済み module:

```text
Proofs.Ai.Algebra.AbstractFieldIdeal
```

実装済み theorem:

```text
field_ideal_zero_or_top
field_simple_ring_evidence
quotient_by_maximal_ideal_is_field
```

目的:

- `AbstractKrullTheorem` や `AbstractHilbertNullstellensatz` の前提をより自然な体論 API と接続する。
- maximal ideal 商が体になる標準ルートを、証明 corpus 上で再利用可能にする。

## 4. OrderedField との接続

`AbstractOrderedField` は現状、順序・平方根・平方の単調性を bundle として持っています。
`AbstractField` 追加後の互換 bridge として、次の module を追加済みです。

実装済み module:

```text
Proofs.Ai.Algebra.AbstractOrderedFieldFieldBridge
```

実装済み theorem:

```text
ordered_field_field_laws
ordered_field_nonzero_of_positive
ordered_field_inv_positive
ordered_field_div_positive
ordered_field_mul_pos
ordered_field_sq_pos_of_nonzero
```

既存の `OrderedFieldLawArgs` は削除または全面置換していません。split bridge module で
`FieldLawArgs` と order/sqrt laws の間に projection theorem を追加し、既存
`AbstractOrderedField` consumer の certificate / export hash への影響を局所化しています。

## 5. 基礎 route の実装単位

実装単位は、次の順で完了しています。

1. `Proofs.Ai.Algebra.AbstractField`
2. `Proofs.Ai.Algebra.AbstractFieldHom`
3. `Proofs.Ai.Algebra.AbstractFieldIntegralDomain`
4. `Proofs.Ai.Algebra.AbstractFieldIdeal`
5. `Proofs.Ai.Algebra.AbstractOrderedFieldFieldBridge`

各 module は小さく分け、import は必要最小限にしています。
最初の `AbstractField` は `AbstractRing` と `Std.Logic.Eq` だけから始め、
群商・環商・CRT への依存は後続 bridge module に閉じ込めています。

## 6. npa-mathlib materialization policy

この field-theory route は corpus 側の verified staging として完了しています。公開
`npa-mathlib` への materialization はこの roadmap では実行せず、別の closure audit で
import closure、axiom policy、statement stability、compatibility alias の要否を確認してから
判断します。

## 7. 高度な体論の追加計画

FT-01 から FT-07 までで、`AbstractField` foundation、field hom、field-to-integral-domain、
field ideal / quotient、ordered-field bridge までの verified staging は完了しています。
次の追加では、すぐに `npa-mathlib` へ一括 materialize するのではなく、corpus 側で
downstream 利用実績を増やし、import closure が小さい層から public package 候補にします。

高度 route の優先順位は次の通りです。

### 7.1 Field hom kernel / image / embedding

最初に追加する候補:

```text
Proofs.Ai.Algebra.AbstractFieldHomKernelImage
```

候補 theorem / API:

```text
field_hom_kernel_zero_of_nonzero
field_hom_injective_of_nonzero
field_hom_image_field_laws
field_embedding_as_field_hom
field_embedding_comp
field_iso_symm
field_iso_trans
```

目的:

- `AbstractFieldHom` の direct downstream を増やし、promotion 判断を強くする。
- field hom の kernel / image / injectivity を、既存 `RingHomLawArgs` と field-level `Nonzero`
  から再利用できる形にする。
- 後続の field extension / embedding / isomorphism API の土台にする。

注意:

- zero map 排除や injectivity は、`1` 保存、`zero_ne_one`、kernel triviality などの証拠を
  明示引数として扱う。
- まずは concrete kernel quotient を作らず、explicit evidence package と projection theorem に留める。

### 7.2 Polynomial quotient over a field

次に追加する候補:

```text
Proofs.Ai.Algebra.AbstractPolynomialFieldQuotient
```

候補 theorem / API:

```text
PolynomialFieldQuotientArgs
irreducible_polynomial_generates_maximal_ideal
quotient_by_irreducible_polynomial_is_field
polynomial_eval_kernel_contains_minimal_polynomial
simple_algebraic_extension_as_polynomial_quotient
```

目的:

- `F[x] / (p)` が体になる標準ルートを corpus に置く。
- field extension、finite field、minimal polynomial、splitting field の土台を作る。
- 既存の abstract Hilbert / Nullstellensatz / polynomial-extension style と接続する。

注意:

- まだ concrete polynomial syntax や polynomial evaluator を trusted base に入れない。
- `IrreduciblePolynomial`、`PrincipalIdealGeneratedBy`、`PolynomialQuotientFieldArgs` などの
  evidence package を明示し、証明受理は certificate に限定する。
- `AbstractFieldIdeal` の大きい closure をそのまま public 化しないよう、quotient field に必要な
  最小 bridge を分割する。

### 7.3 Field extension law package

候補 module:

```text
Proofs.Ai.Algebra.AbstractFieldExtension
```

候補 theorem / API:

```text
FieldExtensionLawArgs
field_extension_base_embedding
field_extension_as_field
field_extension_restrict_scalars
field_extension_tower
field_embedding_compose
```

目的:

- base field `K`、extension field `L`、embedding `K -> L` を明示的な law package として扱う。
- algebraic extension、finite extension、splitting field、Galois theory へ進む入口にする。
- 既存 vector / linear algebra corpus と接続できる形で scalar restriction を用意する。

注意:

- module name と statement は変わりやすいため、最初は corpus staging に留める。
- `FieldHomLawArgs` と `field_hom_injective_of_nonzero` を下流利用する設計にする。

### 7.4 Algebraic elements and minimal polynomial

候補 module:

```text
Proofs.Ai.Algebra.AbstractAlgebraicExtension
```

候補 theorem / API:

```text
AlgebraicElement
MinimalPolynomial
minimal_polynomial_divides_annihilating_polynomial
minimal_polynomial_irreducible
degree_one_algebraic_element_in_base
field_adjoin_algebraic_element_is_finite_extension
```

目的:

- algebraic extension と finite extension の橋渡しを作る。
- polynomial quotient route と field extension route を接続する。
- 後続の splitting field / algebraic closure の statement を小さくする。

注意:

- minimal polynomial の uniqueness / monic / irreducible 条件は statement が揺れやすい。
  まずは `MinimalPolynomial` evidence package に閉じ込める。

### 7.5 Finite extension

候補 module:

```text
Proofs.Ai.Algebra.AbstractFiniteFieldExtension
```

候補 theorem / API:

```text
FiniteExtensionLawArgs
finite_extension_is_algebraic
extension_degree_tower
finite_dimensional_vector_space_bridge
finite_extension_embedding_preserves_degree
```

目的:

- `[L : K]` 型の degree 証拠を explicit package として扱う。
- tower law と finite-dimensional vector-space bridge を corpus に置く。
- finite field / Galois theory の依存を整理する。

注意:

- 自然数 arithmetic や dimension API が重い場合は、最初は degree law を Prop-level evidence として扱う。

### 7.6 Finite fields and Frobenius

候補 module:

```text
Proofs.Ai.Algebra.AbstractFiniteField
```

候補 theorem / API:

```text
FiniteFieldLawArgs
field_characteristic_prime_or_zero
finite_field_characteristic_prime
frobenius_is_field_hom
finite_field_pow_card_eq_self
finite_field_roots_x_pow_q_minus_x
```

目的:

- finite field route を作り、Frobenius、cardinality、root characterization を theorem search 可能にする。
- later finite-field specific corpus と `npa-mathlib` promotion 候補を作る。

注意:

- cardinality、power、polynomial roots の API は依存が重くなりやすい。
  まずは `FiniteFieldLawArgs` と Frobenius homomorphism から始める。

### 7.7 Splitting field / algebraic closure

候補 module:

```text
Proofs.Ai.Algebra.AbstractSplittingField
Proofs.Ai.Algebra.AbstractAlgebraicClosure
```

候補 theorem / API:

```text
SplittingFieldLawArgs
splitting_field_contains_all_roots
splitting_field_generated_by_roots
splitting_field_unique_up_to_field_iso
AlgebraicClosureLawArgs
algebraic_closure_is_algebraic
algebraic_closure_polynomial_has_root
```

目的:

- Galois theory へ進む前に、root existence と uniqueness-up-to-isomorphism の evidence package を作る。
- existence theorem を trusted axiom にせず、明示的な construction evidence として扱う。

注意:

- existence は重いので、最初は「given splitting-field evidence」から始める。

### 7.8 Galois theory starter

候補 module:

```text
Proofs.Ai.Algebra.AbstractGaloisStarter
```

候補 theorem / API:

```text
FieldAutomorphismGroupArgs
fixed_field_laws
galois_extension_args
automorphism_group_laws
fixed_field_is_field
galois_correspondence_order_bridge
```

目的:

- field automorphism group、fixed field、Galois correspondence の前段階を作る。
- 既存 group correspondence theorem と field extension route を接続する。

注意:

- 依存が最も重い層なので、field extension、finite extension、splitting field が corpus 側で固まってから着手する。

## 8. 検証

proof corpus に体論 module を追加する作業では、package/full corpus gate を毎回走らせず、
局所確認を優先します。

例:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractField
cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractField --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
```

複数 module の export hash に影響する変更、certificate encode / decode / hash、
kernel semantics、independent checker、package verifier に関わる変更では、
最後に package/full corpus gate を明示的に実行します。

```sh
./scripts/check-corpus-full.sh
```

通常の code / doc 変更だけなら、まず fast gate を使います。

```sh
./scripts/check-fast.sh
```

## 9. 完了条件

FT-01 から FT-07 までの基礎 route の完了条件:

- `FieldLawArgs` が `RingLawArgs` と体固有 law を明確に分けている。
- `inv` / `div` / `Nonzero` の theorem 名が後続 module から検索しやすい。
- `field_ring_laws` により、既存の `AbstractRing` theorem を再利用できる。
- `field_inv_mul_cancel` / `field_mul_inv_cancel` / `field_div_eq_mul_inv` が certificate-backed theorem として検査される。
- generated `.npcert` が source-free verifier で通る。
- axiom report が意図せず増えない。

FT-03 から FT-07 までの基礎 bridge 層の完了条件:

- field hom が既存の `RingHomLawArgs` に橋渡しされる。
- 体から整域性を取り出す theorem がある。
- イデアル・商環ルートで体を使う定理が、`FieldLawArgs` を前提として再利用できる。
- `AbstractOrderedField` との bridge が追加され、既存の順序・平方根 corpus を壊さない。

FT-08 以降の高度な体論 route の完了条件:

- 各 module が explicit evidence package を使い、存在定理を hidden axiom として増やさない。
- 新しい theorem / definition 名と statement が、少なくとも直接 downstream で使える程度に安定している。
- source、certificate、replay、meta、manifest、package metadata、AI theorem index の更新が deterministic である。
- 局所 authoring では対象 module の `--build-module` / `--module` / `--changed-only` を通す。
- public `npa-mathlib` へ promotion する前に、import closure、axiom policy、statement stability、
  compatibility alias の要否を別途 audit する。
- promotion 候補は source-free verifier と package hash / index / axiom report が通っている。
