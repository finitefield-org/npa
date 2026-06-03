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

一方で、`AbstractOrderedField` は順序・平方根 law bundle が中心であり、
体そのものの逆元構造はまだ共通モジュールとして切り出されていません。
そのため、体論ルートで最初に追加すべき主対象は `AbstractField` 層です。

## 2. 基本方針

体論の追加は、いきなり代数閉体・Nullstellensatz・拡大体へ進めず、
`AbstractRing` と `AbstractOrderedField` の間に小さく再利用可能な層を作ります。

設計方針:

- `FieldLawArgs` は明示的な law package として持つ。
- `inv` / `div` / `Nonzero` を core calculus に入れず、通常の carrier 上の演算・述語として扱う。
- `zero_ne_one` や `a != 0` は決定可能 bool にせず、既存 corpus と同じく Prop-level の否定証拠で表す。
- `div` は primitive にせず、まず `mul a (inv b)` への projection theorem を置く。
- proof corpus authoring では便利な source / replay を使ってよいが、受理根拠は checked certificate に限定する。
- 既存の `AbstractOrderedField` はすぐに破壊的に作り替えず、下流 module への影響を見ながら `AbstractField` への橋渡しを追加する。

## 3. 優先順位

### 3.1 AbstractField foundation

最初に追加する候補 module:

```text
Proofs.Ai.Algebra.AbstractField
```

最小の public surface 候補:

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

次に、定理探索と rewrite に効く小さい補題を追加します。

候補:

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

既存の `RingHomLawArgs` と接続する module を追加します。

候補 module:

```text
Proofs.Ai.Algebra.AbstractFieldHom
```

候補 theorem:

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

体から整域性を取り出す層を追加します。

候補 module:

```text
Proofs.Ai.Algebra.AbstractFieldIntegralDomain
```

候補 theorem:

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

候補 module:

```text
Proofs.Ai.Algebra.AbstractFieldIdeal
```

候補 theorem:

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
`AbstractField` 追加後は、互換性を保ちながら次の橋渡しを追加します。

候補:

```text
ordered_field_field_laws
ordered_field_nonzero_of_positive
ordered_field_inv_positive
ordered_field_div_positive
ordered_field_mul_pos
ordered_field_sq_pos_of_nonzero
```

この段階では、既存の `OrderedFieldLawArgs` を直ちに削除または全面置換しません。
まず `FieldLawArgs` と order/sqrt laws の間に projection theorem を追加し、
依存 module の certificate / export hash への影響を局所化します。

## 5. 実装単位

最初の実装単位は、次の順がよいです。

1. `Proofs.Ai.Algebra.AbstractField`
2. `Proofs.Ai.Algebra.AbstractFieldHom`
3. `Proofs.Ai.Algebra.AbstractFieldIntegralDomain`
4. `Proofs.Ai.Algebra.AbstractFieldIdeal`
5. `Proofs.Ai.Algebra.AbstractOrderedField` への bridge theorem 追加

各 module は小さく分け、import は必要最小限にします。
特に最初の `AbstractField` は `AbstractRing` と `Std.Logic.Eq` だけから始め、
群商・環商・CRT への依存は後続 bridge module に閉じ込めます。

## 6. 検証

proof corpus に体論 module を追加する作業では、full corpus gate を毎回走らせず、
局所確認を優先します。

例:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Algebra.AbstractField
cargo run -p npa-proof-corpus -- --module Proofs.Ai.Algebra.AbstractField
cargo run -p npa-proof-corpus -- --changed-only
```

複数 module の export hash に影響する変更、certificate encode / decode / hash、
kernel semantics、independent checker、package verifier に関わる変更では、
最後に corpus gate を実行します。

```sh
./scripts/check-corpus.sh
```

通常の code / doc 変更だけなら、まず fast gate を使います。

```sh
./scripts/check-fast.sh
```

## 7. 完了条件

`AbstractField` foundation の完了条件:

- `FieldLawArgs` が `RingLawArgs` と体固有 law を明確に分けている。
- `inv` / `div` / `Nonzero` の theorem 名が後続 module から検索しやすい。
- `field_ring_laws` により、既存の `AbstractRing` theorem を再利用できる。
- `field_inv_mul_cancel` / `field_mul_inv_cancel` / `field_div_eq_mul_inv` が certificate-backed theorem として検査される。
- generated `.npcert` が source-free verifier で通る。
- axiom report が意図せず増えない。

上位層の完了条件:

- field hom が既存の `RingHomLawArgs` に橋渡しされる。
- 体から整域性を取り出す theorem がある。
- イデアル・商環ルートで体を使う定理が、`FieldLawArgs` を前提として再利用できる。
- `AbstractOrderedField` との bridge が追加され、既存の順序・平方根 corpus を壊さない。
