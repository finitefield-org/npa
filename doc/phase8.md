以下は **Phase 8: independent checker** の詳細設計です。
Phase 8 の目的は、Phase 1〜7 で作った高速 kernel・elaborator・tactic・AI探索に対して、**別経路で証明証明書を再検査する仕組み**を作ることです。

対象はこの3つです。

```text
- reference checker
- external checker
- CI integration
```

大原則はこれです。

```text
本体 kernel が OK と言っても、それだけで満足しない。
source / tactic / AI / elaborator / proof search を一切信用せず、
canonical certificate だけを独立 checker で再検査する。
```

---

# 1. Phase 8 の位置づけ

Phase 7 までの流れはこうでした。

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

Phase 8 では、さらにこうします。

```text
certificate
  ↓
reference checker
  ↓
external checker
  ↓
CI / release audit
  ↓
verified artifact
```

つまり、最終成果物は単なる `.npa` ソースでも、tactic script でも、AI探索ログでもなく、**複数 checker で再検査済みの `.npcert`** です。

---

# 2. checker の種類

Phase 8 では、少なくとも3種類の checker を想定します。

```text
1. fast kernel
   普段の開発・IDE・証明探索で使う高速 kernel。

2. reference checker
   小さく、読みやすく、仕様に忠実な checker。
   遅くてもよい。

3. external checker
   本体とは別実装・別プロセス・別ビルド環境で動く checker。
   高信頼モードやCIで使う。
```

理想的には、将来4つ目も追加します。

```text
4. verified checker
   Lean / Rocq / NPA自身などで正当性を形式検証した checker。
```

ただし Phase 8 MVP では、まず次で十分です。

```text
fast kernel      : Rust
reference checker: OCaml / Haskell / 小さなRust別実装
external checker : reference checker を独立バイナリとして運用
```

---

# 3. 信頼境界

Phase 8 で信用しないもの：

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
- IDE表示
- generated proof script
```

Phase 8 で checker が読むもの：

```text
- canonical core AST
- module certificate
- import hash
- declaration hash
- axiom report
- dependency graph
```

checker が読まないもの：

```text
- .npa source
- tactic script
- AI search trace
- pretty printed goal
- theorem search index
- source map
```

つまり、checker の入力は原則として `.npcert` だけです。

---

# 4. Reference checker

## 4.1 目的

reference checker は、**仕様そのものに近い、単純で監査しやすい checker** です。

fast kernel は高速化のために複雑になります。

```text
- arena allocation
- hash-consing
- WHNF cache
- conversion cache
- parallel checking
- optimized substitution
- compact term encoding
```

これらは性能には重要ですが、バグの温床にもなります。
reference checker は逆に、なるべく単純にします。

```text
- 遅くてよい
- キャッシュは最小限
- 並列化しない
- 最適化しすぎない
- unsafe code を使わない
- 仕様に忠実
- エラーメッセージより正確性重視
```

---

## 4.2 Reference checker が検査するもの

`.npcert` を受け取り、次を検査します。

```text
1. certificate header
2. import hash
3. canonical encoding
4. term hash
5. declaration hash
6. declaration dependency
7. type correctness
8. conversion correctness
9. universe consistency
10. inductive declaration validity
11. axiom report correctness
12. export hash
13. certificate hash
```

成功時：

```json
{
  "status": "checked",
  "checker": "npa-checker-ref",
  "module": "Std.Nat",
  "export_hash": "sha256:...",
  "certificate_hash": "sha256:...",
  "axioms_used": [],
  "declarations_checked": 128
}
```

失敗時：

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

## 4.3 Reference checker の非目標

reference checker は次をしません。

```text
- source parse
- elaboration
- tactic execution
- proof search
- theorem search
- AI呼び出し
- source map 解釈
- pretty printing 依存の処理
- import解決のためのネットワークアクセス
```

特に重要なのは：

```text
reference checker は .npa source を読まない。
```

です。

もし source からもう一度 elaboration して検査すると、elaborator のバグを再利用してしまう可能性があります。
Phase 8 の目的は、elaborator とは独立に `.npcert` を検査することです。

---

# 5. Reference checker の構造

## 5.1 全体構成

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

Rust風のAPIなら：

```rust
fn check_certificate(cert: ModuleCert, imports: ImportStore) -> Result<CheckedModule>;

fn check_declaration(env: &mut Env, decl: &DeclCert) -> Result<()>;

fn infer(env: &Env, ctx: &Ctx, term: TermId) -> Result<TermId>;

fn is_defeq(env: &Env, ctx: &Ctx, a: TermId, b: TermId) -> Result<bool>;
```

reference checker では、データ構造は単純でよいです。

```text
fast kernel:
  arena + ExprId + hash-consing + cache

reference checker:
  immutable AST + plain recursion + explicit environment
```

---

## 5.2 Certificate decode

reference checker はまず canonical binary を decode します。

検査すること：

```text
- magic number が正しい
- certificate format version が対応範囲内
- core spec version が対応範囲内
- section order が canonical
- term table が canonical
- name table が canonical
- declaration order が dependency order
- unknown tag がない
- duplicate name がない
- dangling reference がない
```

エラー例：

```json
{
  "error_kind": "non_canonical_encoding",
  "message": "term table contains unused entry",
  "term_id": 42
}
```

Phase 8 の checker は、**非canonicalだが意味的には読める certificate** も拒否します。
理由は、hashと再現性を壊すからです。

---

## 5.3 Import verification

import は名前だけではなく hash で検査します。

```json
{
  "module": "Std.Logic",
  "export_hash": "sha256:...",
  "certificate_hash": "sha256:..."
}
```

reference checker は：

```text
1. import module の certificate を探す
2. import certificate を再検査する、または検査済みcacheを確認する
3. export_hash が一致するか確認する
4. high-trust mode なら certificate_hash も一致確認する
5. import の public environment を現在の environment に追加する
```

通常モード：

```text
require export_hash
```

高信頼モード：

```text
require export_hash
require certificate_hash
require imported certificate already checked by same checker
```

---

## 5.4 Declaration check

宣言ごとに検査します。

```text
AxiomDecl:
  type : Sort u を確認
  axiom report に追加

DefDecl:
  type : Sort u を確認
  value : type を確認
  reducibility を environment に登録

TheoremDecl:
  type : Sort u を確認
  proof : type を確認
  proof は opaque として登録

InductiveDecl:
  parameters / indices / constructors / positivity / recursor を検査
```

疑似コード：

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

# 6. Conversion checker in reference checker

## 6.1 方針

reference checker の conversion は、fast kernel と同じ仕様を実装します。

Phase 1 の仕様：

```text
β reduction
δ reduction
ι reduction
ζ reduction
```

入れないもの：

```text
η conversion
proof irrelevance conversion
quotient computation
untrusted theorem unfolding
```

---

## 6.2 WHNF

reference checker でも WHNF は必要です。

```text
whnf(t):
  - App (Lam A body) arg       → β
  - Let A value body           → ζ
  - Const c                    → δ if reducible
  - Recursor applied to ctor   → ι
```

疑似コード：

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

## 6.3 Definitional equality

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

reference checker では、性能よりも明快さを優先します。

---

# 7. Inductive checking

## 7.1 検査対象

reference checker は inductive declaration も検査します。

```text
- parameters が well-typed
- indices が well-typed
- result sort が valid
- constructors が well-typed
- constructor result が対象 inductive
- recursive occurrence が strictly positive
- generated recursor type が正しい
- iota rules が宣言と一致する
```

---

## 7.2 Strict positivity

Phase 8 MVP の positivity checker は保守的でよいです。

許す：

```text
Nat
List A
A -> I
I -> I   ではなく、constructor argument としての I
```

禁止：

```text
(I -> A) -> I
negative occurrence
nested inductive
mutual inductive
```

もし Phase 1〜6 で `Nat`, `Eq`, `List` しか扱わないなら、まずはこの範囲を確実に検査できれば十分です。

---

# 8. Hash verification

## 8.1 checker は hash を信じない

certificate に保存されている hash は信用しません。
reference checker は必ず再計算します。

検査対象：

```text
- term_hash
- decl_interface_hash
- decl_certificate_hash
- export_hash
- certificate_hash
- axiom_report_hash
```

流れ：

```text
decode certificate
  ↓
canonical encode again
  ↓
recompute hashes
  ↓
compare with stored hashes
```

不一致なら失敗です。

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

## 8.2 Hash policy

Domain separation を必須にします。

```text
H("NPA_TERM_V1" || term_encoding)
H("NPA_DECL_IFACE_V1" || decl_interface)
H("NPA_DECL_CERT_V1" || decl_certificate)
H("NPA_MODULE_EXPORT_V1" || export_block)
H("NPA_MODULE_CERT_V1" || full_certificate)
H("NPA_AXIOM_REPORT_V1" || axiom_report)
```

こうすることで、異なる種類のデータを同じ hash として誤用する事故を減らします。

---

# 9. Axiom report verification

## 9.1 axiom report は再計算する

certificate 内の axiom report はログではありません。
検証対象です。

reference checker は各宣言の依存関係から、axiom 集合を再計算します。

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

そして certificate の axiom report と一致するか確認します。

---

## 9.2 Trust policy

checker は policy file を受け取ります。

```json
{
  "deny_sorry": true,
  "deny_custom_axioms": true,
  "allow_axioms": [
    "Classical.choice",
    "Propext"
  ]
}
```

高信頼標準ライブラリでは：

```json
{
  "deny_sorry": true,
  "deny_custom_axioms": true,
  "allow_axioms": []
}
```

検査結果：

```json
{
  "axioms_used": [],
  "contains_sorry": false,
  "safe_for_high_trust": true
}
```

禁止公理がある場合：

```json
{
  "status": "failed",
  "error_kind": "forbidden_axiom",
  "axiom": "synthetic.sorry.Std.Nat.add_zero",
  "declaration": "Nat.add_zero"
}
```

---

# 10. External checker

## 10.1 目的

external checker は、**本体ビルドシステムから独立して動く checker** です。

reference checker は「仕様に近い実装」という意味です。
external checker は「運用上、本体から切り離されている検査器」という意味です。

理想：

```text
- 別バイナリ
- 別プロセス
- 別ビルド設定
- 別言語または別実装
- source code を読まない
- network access なし
- plugin なし
- tactic 実行なし
- AI なし
```

---

## 10.2 external checker のCLI

```bash
npa-checker-ext \
  --cert build/Std/Nat.npcert \
  --import-dir build/certs \
  --policy policies/high_trust.json \
  --output json
```

出力：

```json
{
  "status": "checked",
  "checker": "npa-checker-ext",
  "checker_version": "0.1.0",
  "core_spec": "NPA-Core-0.1",
  "certificate_format": "NPA-CERT-1",
  "module": "Std.Nat",
  "export_hash": "sha256:...",
  "certificate_hash": "sha256:...",
  "axioms_used": [],
  "checked_declarations": 84,
  "time_ms": 913
}
```

---

## 10.3 external checker の入力

external checker の入力は限定します。

```text
入力として許す:
  - .npcert
  - import certificate directory
  - policy file
  - optional expected statement hash
  - optional expected export hash

入力として許さない:
  - source .npa
  - tactic script
  - generated theorem search index
  - proof search trace
  - untrusted plugin
  - remote import
```

---

## 10.4 Challenge mode

高信頼用途では、証明対象の命題だけを別ファイルとして固定します。

```json
{
  "challenge": "theorem add_zero : ∀ n : Nat, n + 0 = n",
  "statement_core_hash": "sha256:...",
  "allowed_axioms": [],
  "imports": [
    {
      "module": "Std.Nat",
      "export_hash": "sha256:..."
    }
  ]
}
```

external checker は、証明 certificate の theorem statement が challenge と一致するか検査します。

```text
certificate theorem statement hash
  ==
challenge statement hash
```

これにより、AIやproof searchが勝手に「似ているが違う定理」を証明して成功扱いすることを防げます。

---

## 10.5 Audit bundle

release や論文・コンテスト提出用には、audit bundle を作ります。

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

external checker はこの bundle だけで検査できます。

```bash
npa-checker-ext --audit-bundle audit/
```

成功時：

```json
{
  "status": "verified_audit_bundle",
  "challenge_statement_match": true,
  "imports_checked": true,
  "proof_checked": true,
  "policy_satisfied": true,
  "axioms_used": []
}
```

---

# 11. Checker disagreement

## 11.1 不一致は必ず failure

fast kernel, reference checker, external checker の結果が不一致なら、CI は fail します。

ケース：

```text
fast kernel OK, reference checker NG
  → fast kernel のバグまたは certificate generator のバグの可能性

fast kernel NG, reference checker OK
  → fast kernel が厳しすぎる、または reference checker が緩い可能性

reference checker OK, external checker NG
  → checker実装差異、または環境差異

hash一致だが axiom report 不一致
  → certificate生成またはreport計算の重大バグ
```

どの場合も release してはいけません。

---

## 11.2 disagreement report

不一致時には、最小限の再現情報を出します。

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

# 12. CI integration

## 12.1 CI の目的

CI は、次を自動で保証します。

```text
- source が build できる
- certificate が生成される
- fast kernel で検査される
- reference checker で再検査される
- external checker で再検査される
- import hash が固定されている
- declaration hash が一致する
- axiom report が正しい
- forbidden axiom / sorry がない
- theorem index が certificate と一致する
- proof minimization が定理を壊していない
- performance regression がない
```

---

## 12.2 CI pipeline 全体

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

# 13. CI Stage 詳細

## Stage 1: source lint

検査：

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

例：

```text
fail:
  theorem Nat.add_zero ... := sorry
```

---

## Stage 2: build certificates

ソースから `.npcert` を生成します。

```bash
npa build --emit-cert --locked
```

生成物：

```text
build/
  Std/Logic.npcert
  Std/Nat.npcert
  Std/List.npcert
  Std/Algebra/Basic.npcert
```

この段階では fast kernel が使われます。

---

## Stage 3: fast kernel check

```bash
npa check build/Std/Nat.npcert
```

検査：

```text
- core term type check
- conversion
- inductive rules
- declaration hash
- module hash
```

---

## Stage 4: reference checker check

```bash
npa-checker-ref \
  --cert build/Std/Nat.npcert \
  --import-dir build \
  --policy policies/std.json
```

PR では変更モジュールとその依存先を検査します。

```text
changed module:
  Std.Nat

also check reverse dependencies:
  Std.List
```

nightly では全モジュールを検査します。

---

## Stage 5: external checker check

```bash
npa-checker-ext \
  --cert build/Std/Nat.npcert \
  --import-dir build \
  --policy policies/std.json
```

external checker は、できれば別 container で実行します。

```text
- no network
- read-only cert directory
- no source directory mounted
- no build scripts
- no plugin
```

これにより、external checker が本当に certificate だけで検査していることを保証しやすくなります。

---

## Stage 6: axiom policy check

```bash
npa audit axioms build/Std/Nat.npcert --policy policies/std.json
```

標準ライブラリでは：

```text
allowed axioms = []
```

fail 条件：

```text
- sorry がある
- custom axiom がある
- allowlist にない axiom がある
- axiom report と再計算結果が違う
```

出力：

```json
{
  "module": "Std.Nat",
  "axioms_used": [],
  "contains_sorry": false,
  "safe_for_high_trust": true
}
```

---

## Stage 7: hash reproducibility check

同じ source と lockfile から、同じ certificate hash が得られるか確認します。

```bash
npa clean
npa build --emit-cert --locked
hash1=$(npa cert-hash build/Std/Nat.npcert)

npa clean
npa build --emit-cert --locked
hash2=$(npa cert-hash build/Std/Nat.npcert)

test "$hash1" = "$hash2"
```

これにより：

```text
- 非決定的なname allocation
- 非決定的なdeclaration order
- timestamp混入
- random seed混入
- hash table iteration order依存
```

を検出できます。

---

## Stage 8: theorem index validation

Phase 6/7 の theorem search index が certificate と一致するか検査します。

```bash
npa index validate \
  --index build/Std/Nat.index.json \
  --cert build/Std/Nat.npcert
```

検査：

```text
- index内の定理がcertificateに存在する
- statement hash が一致する
- attributes が宣言metadataと一致する
- rewrite lhs/rhs が theorem statement と一致する
- axiom_deps が axiom report と一致する
```

AI探索は theorem index に依存するため、ここも重要です。
ただし theorem index 自体は trusted ではありません。間違っていても最終的な proof check は通りませんが、探索品質や安全ポリシーに影響します。

---

## Stage 9: tactic/search regression tests

Phase 4/7 の tactic とAI探索をテストします。

```bash
npa test tactics
npa test search
```

例：

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

重要：

```text
tactic/search regression は convenience test。
checker の代わりではない。
```

必ず最後に certificate check します。

---

## Stage 10: performance benchmarks

速度劣化を検出します。

測るもの：

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

例：

```json
{
  "module": "Std.Nat",
  "fast_kernel_ms": 120,
  "reference_checker_ms": 950,
  "external_checker_ms": 1100,
  "memory_mb": 42
}
```

CI policy：

```text
PR:
  大幅な性能劣化があれば警告またはfail

nightly:
  詳細ベンチマークを記録

release:
  基準値を超えたらfail
```

---

## Stage 11: audit bundle generation

release や高信頼検証用に audit bundle を生成します。

```bash
npa audit bundle \
  --module Std.Nat \
  --cert build/Std/Nat.npcert \
  --imports build \
  --policy policies/std.json \
  --out audit/Std.Nat/
```

中身：

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

# 14. CI モード

## 14.1 Pull request mode

速さ重視。

```text
- changed modules
- reverse dependencies
- fast kernel check
- reference checker on changed certs
- external checker on changed certs
- axiom policy
- basic tactic regression
```

## 14.2 Nightly mode

網羅性重視。

```text
- full library check
- full reference checker
- full external checker
- fuzz tests
- theorem index validation
- AI search benchmark
- performance benchmark
```

## 14.3 Release mode

高信頼。

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

## 14.4 High-trust mode

論文・コンテスト・安全性重視用途。

```text
- challenge file required
- proof certificate required
- source ignored
- no network
- no plugin
- no custom axiom
- no sorry
- all imports recursively checked
- at least two independent checkers required
```

---

# 15. Fuzzing and mutation tests

Phase 8 では、checker の堅牢性テストも重要です。

## 15.1 Certificate fuzzing

不正 certificate を大量生成して、checker が安全に拒否するか確認します。

変異例：

```text
- term tag を壊す
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

期待：

```text
checker は panic せず、明確に reject する。
```

## 15.2 Proof mutation

正しい証明を少し壊します。

```text
Eq.refl n
  ↓
Eq.refl m
```

または：

```text
Nat.add_zero proof
  ↓
proof term の一部を別termに変更
```

期待：

```text
fast kernel / reference checker / external checker がすべて reject する。
```

## 15.3 Differential testing

fast kernel と reference checker に同じ certificate を食わせ、結果を比較します。

```text
same input
  ↓
fast kernel result
reference checker result
external checker result
  ↓
must agree
```

不一致なら fail。

---

# 16. 実装言語の推奨

Phase 8 では、fast kernel と reference checker を同じ設計・同じコード共有にしすぎない方がよいです。

おすすめ：

```text
fast kernel:
  Rust

reference checker:
  OCaml / Haskell / 別Rust実装

external checker:
  reference checker の独立バイナリ
  または別言語実装

future verified checker:
  NPA自身 / Lean / Rocq
```

避けたい構成：

```text
fast kernel と reference checker が同じ内部ライブラリをほぼ共有
```

これだと、同じバグを共有する危険があります。

許容できる共有：

```text
- certificate format の仕様書
- テストケース
- golden certificates
```

避けたい共有：

```text
- conversion checker 実装
- type checker 実装
- inductive checker 実装
- positivity checker 実装
```

---

# 17. Checker API

## 17.1 `/check/certificate`

ローカルAPIとして提供してもよいです。

```json
POST /check/certificate
{
  "certificate_path": "build/Std/Nat.npcert",
  "checker": "reference",
  "policy": {
    "deny_sorry": true,
    "deny_custom_axioms": true,
    "allow_axioms": []
  }
}
```

レスポンス：

```json
{
  "status": "checked",
  "checker": "reference",
  "module": "Std.Nat",
  "certificate_hash": "sha256:...",
  "export_hash": "sha256:...",
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

レスポンス：

```json
{
  "status": "verified_audit_bundle",
  "challenge_statement_match": true,
  "imports_checked": true,
  "policy_satisfied": true
}
```

---

# 18. 最小コマンド群

Phase 8 MVP で用意するCLI：

```bash
npa cert check build/Std/Nat.npcert
npa cert hash build/Std/Nat.npcert
npa cert axioms build/Std/Nat.npcert
npa audit bundle --module Std.Nat --out audit/Std.Nat

npa-checker-ref --cert build/Std/Nat.npcert --import-dir build
npa-checker-ext --cert build/Std/Nat.npcert --import-dir build
```

CIでは：

```bash
npa build --emit-cert --locked
npa-checker-ref --cert build/Std/Nat.npcert --import-dir build --policy policies/std.json
npa-checker-ext --cert build/Std/Nat.npcert --import-dir build --policy policies/std.json
npa audit axioms build/Std/Nat.npcert --policy policies/std.json
```

---

# 19. Phase 8 の実装順序

おすすめ順はこれです。

```text
1. Certificate decoder for reference checker
   .npcert を source なしで読めるようにする

2. Hash verifier
   term / decl / module hash を再計算する

3. Environment builder
   import certificate から environment を作る

4. Minimal type checker
   Sort / Pi / Lambda / App / Let / Const

5. Conversion checker
   βδ から始め、次に ζ、最後に ι

6. Def / theorem check
   value : type, proof : type を確認

7. Axiom report recomputation
   certificate内 report と比較

8. Inductive checker
   Nat / Eq / List の simple inductive を確認

9. External checker CLI
   sourceなし、certのみ、policy付き

10. CI integration
    PR / nightly / release pipeline

11. Differential testing
    fast kernel vs reference vs external

12. Fuzzing / mutation testing
    不正certificateのreject確認

13. Audit bundle
    high-trust mode 用成果物
```

---

# 20. Phase 8 のテストケース

## 20.1 正常系

```text
Std.Logic.npcert
Std.Nat.npcert
Std.List.npcert
Std.Algebra.Basic.npcert
```

期待：

```text
fast kernel OK
reference checker OK
external checker OK
axioms_used = []
```

## 20.2 hash 改ざん

`Nat.add_zero` の proof term を1 byte変える。

期待：

```text
hash mismatch
or type check failure
```

## 20.3 axiom report 改ざん

実際には axiom を使っているのに、axiom report から削除。

期待：

```text
AxiomReportMismatch
```

## 20.4 import hash mismatch

`Std.Nat` が依存する `Std.Logic` の export_hash を変更。

期待：

```text
ImportHashMismatch
```

## 20.5 theorem statement mismatch

challenge では：

```text
∀ n : Nat, n + 0 = n
```

証明 certificate では：

```text
∀ n : Nat, n = n
```

期待：

```text
ChallengeStatementMismatch
```

## 20.6 noncanonical certificate

term table に未使用項目を追加。

期待：

```text
NonCanonicalEncoding
```

## 20.7 forbidden axiom

`Classical.choice` を使った certificate を、allowlist 空で検査。

期待：

```text
ForbiddenAxiom
```

---

# 21. Phase 8 でまだ入れないもの

MVPでは後回しでよいもの：

```text
- 形式検証済み checker
- 複雑な mutual/nested inductive の検査
- quotient computation
- proof irrelevance conversion
- η conversion
- external SMT certificate checker
- distributed certificate verification
- cryptographic signature infrastructure
```

まずは：

```text
Nat / Eq / List / basic theorem
```

を source なしで独立再検査できることが最優先です。

---

# 22. Phase 8 の完了条件

Phase 8 が完了したと言える条件はこれです。

```text
- .npcert を source なしで検査できる
- reference checker が fast kernel と独立している
- external checker が別プロセスで動く
- import hash を検査できる
- declaration hash を再計算できる
- certificate hash を再計算できる
- axiom report を再計算できる
- forbidden axiom / sorry を拒否できる
- Nat / Eq / List / Std.Algebra.Basic の証明を再検査できる
- fast kernel / reference / external checker がCIで比較される
- checker不一致時にCIがfailする
- audit bundle を生成・検査できる
- release時に full independent check が走る
```

---

# 23. 一文でまとめると

Phase 8 は、**証明器本体が作った証明を、証明器本体から独立した経路で再検査する段階**です。

中核はこの流れです。

```text
.npcert
  ↓
reference checker
  ↓
external checker
  ↓
axiom/hash/import policy check
  ↓
CIで強制
  ↓
verified artifact
```

これにより、AI探索・tactic・elaborator・fast kernel のどこかにバグがあっても、最終的な certificate を独立 checker が拒否できます。

最終的な理想は：

```text
「証明が見つかった」ではなく、
「複数の独立 checker が同じ certificate を検査し、
 import hash と axiom policy も満たした」
ことを verified と呼ぶ。
```

