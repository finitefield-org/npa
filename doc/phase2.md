この5項目は **Phase 2: certificate** の中核です。
Phase 1では「kernelがcore termを検査できる」ことが目標でした。Phase 2では、それを一段進めて、**ソースコード・tactic・AI出力から独立した再検査可能な成果物**を作ります。

目標はこれです。

```text
source / tactic / AI / elaborator
  ↓
canonical core AST
  ↓
module certificate
  ↓
hashes + axiom report
  ↓
kernel / independent checker で再検査
```

---

# 1. Phase 2 の目的

Phase 2で作るものは、ざっくり言うと `.npcert` のような証明証明書ファイルです。

```text
Std/Nat/Basic.npa      -- 人間が書くソース
Std/Nat/Basic.npcert   -- kernel/checkerが読む証明証明書
```

重要なのは、checker が読むのはソースではなく certificate という点です。

```text
信頼しない:
  parser
  notation
  elaborator
  tactic
  AI
  source-level macro

信頼する:
  canonical certificate を読む kernel / checker
```

Phase 2の成果物は、最低限次を持ちます。

```text
- canonical core AST
- module certificate
- import hash
- declaration hash
- axiom report
```

---

# 2. canonical core AST

## 2.1 目的

canonical core AST は、表層構文を完全に消した、kernel用の標準表現です。

表層では：

```text
theorem id (A : Type) (x : A) : A := x
```

のように書けますが、certificate内では：

```text
Lam A : Sort u,
  Lam x : BVar 0,
    BVar 0
```

のような完全に明示的なtermとして保存します。

canonical core ASTの条件は次です。

```text
- notationなし
- implicit argumentsなし
- typeclass placeholderなし
- unresolved metavariableなし
- tacticなし
- macroなし
- source-level matchなし
- binder nameに依存しない
- universe levelが明示されている
- global constant参照が一意
```

つまり、次の2つは同じcanonical ASTになります。

```text
λ x : Nat, x
λ y : Nat, y
```

binder名は意味を持たないためです。

## 2.2 Core AST の形

Phase 2では、Phase 1で決めたこのtermをcanonical化します。

```text
Term ::=
  Sort Level
| BVar Index
| Const GlobalRef [Level]
| App Term Term
| Lam Type Body
| Pi  Type Body
| Let Type Value Body
```

binder名はcertificateの本体には入れません。
デバッグ用に source map や display name を別領域に入れるのはよいですが、kernel検査やhash計算には使いません。

## 2.3 Canonical encoding

概念的にはこういうタグ付き表現にします。

```text
00 Sort(level)
01 BVar(index)
02 Const(global_ref, levels)
03 App(fn, arg)
04 Lam(type, body)
05 Pi(type, body)
06 Let(type, value, body)
```

たとえば：

```text
Π A : Sort u, A → A
```

は内部的には：

```text
Pi
  type = Sort u
  body =
    Pi
      type = BVar 0
      body = BVar 1
```

になります。

de Bruijn index を使うので、binder名には依存しません。

## 2.4 GlobalRef

`Const` の参照は、名前だけでは危険です。

悪い例：

```text
Const "Nat.add"
```

同じ名前で違う中身のimportが存在する可能性があります。

望ましい形式は：

```text
Const {
  import_index,
  declaration_name,
  declaration_interface_hash
}
```

または、現在のモジュール内なら：

```text
Const {
  local_declaration_index
}
```

つまり `Nat.add` という名前だけでなく、「どのimportの、どのhashを持つ宣言か」まで固定します。

## 2.5 Term hash

各termにはhashを与えます。

```text
term_hash(t) = H("NPA_TERM_V1" || canonical_encode(t))
```

ここで `"NPA_TERM_V1"` は domain separation 用の固定文字列です。

これにより、別の種類のhashと衝突しても意味的に混同しません。

たとえば：

```text
H("NPA_TERM_V1" || ...)
H("NPA_DECL_CERT_V1" || ...)
H("NPA_MODULE_EXPORT_V1" || ...)
```

のように分けます。

---

# 3. module certificate

## 3.1 目的

module certificate は、1つのモジュール全体を検査するための成果物です。

たとえば：

```text
Std.Nat.Basic
```

というモジュールには、次のようなcertificateを作ります。

```text
Std.Nat.Basic.npcert
```

この中に、import、宣言、hash、axiom report が入ります。

## 3.2 大枠

概念的にはこうです。

```json
{
  "format": "NPA-CERT-0.1",
  "core_spec": "NPA-Core-0.1",
  "module": "Std.Nat.Basic",
  "imports": [],
  "declarations": [],
  "exports": [],
  "axiom_report": {
    "module_axioms": [],
    "per_declaration": []
  },
  "hashes": {
    "export_hash": "sha256:...",
    "certificate_hash": "sha256:...",
    "axiom_report_hash": "sha256:..."
  }
}
```

実際の保存形式は JSON ではなく canonical binary にします。
JSONは説明用・デバッグ用で、検査用の本体はbinaryにします。
上の JSON も logical/debug view であり、hash 対象の canonical payload では map を使いません。
`exports`、`axiom_report`、`hashes` は固定順の record または長さ付き配列として encode します。

理由は：

```text
- JSONは空白・順序・エスケープなどでcanonical化が面倒
- binaryの方が高速
- hash対象を固定しやすい
- term table / name table / DAG共有を入れやすい
```

## 3.3 ModuleCert の構造

Rust風にはこうです。

```rust
struct ModuleCert {
    header: CertHeader,
    imports: Vec<ImportEntry>,
    name_table: NameTable,
    level_table: LevelTable,
    term_table: TermTable,
    declarations: Vec<DeclCert>,
    export_block: ExportBlock,
    axiom_report: AxiomReport,
    hashes: ModuleHashes,
}
```

各要素の役割は：

```text
header:
  certificate形式、core仕様、module名など

imports:
  依存モジュールとそのhash

name_table:
  global nameのcanonical encoding

level_table:
  universe levelの共有テーブル

term_table:
  core termのDAG表現

declarations:
  このモジュールで定義されるdef/theorem/axiom/inductive

export_block:
  downstream moduleが使う公開インターフェース

axiom_report:
  使用されたaxiom一覧

hashes:
  export hash、full certificate hash、axiom report hash
```

## 3.4 Declaration の種類

Phase 2でcertificate化する宣言は最低限この4種類です。

```text
AxiomDecl
DefDecl
TheoremDecl
InductiveDecl
```

### AxiomDecl

```json
{
  "kind": "axiom",
  "name": "Classical.choice",
  "universe_params": ["u"],
  "type": "..."
}
```

AxiomDecl は原則として危険です。
高信頼モードでは allowlist 制にします。

### DefDecl

```json
{
  "kind": "def",
  "name": "Nat.add",
  "universe_params": [],
  "type": "...",
  "value": "...",
  "reducibility": "reducible"
}
```

`DefDecl` は、kernel が次を検査します。

```text
value : type
```

さらに、`reducible` な定義なら δ-reduction の対象になります。

### TheoremDecl

```json
{
  "kind": "theorem",
  "name": "Nat.add_zero",
  "universe_params": [],
  "type": "Π n : Nat, Eq Nat (Nat.add n Nat.zero) n",
  "proof": "...",
  "opacity": "opaque"
}
```

kernel は：

```text
proof : type
```

を検査します。

ただし theorem は通常 opaque なので、conversion checker は proof を展開しません。

### InductiveDecl

```json
{
  "kind": "inductive",
  "name": "Nat",
  "universe_params": [],
  "params": [],
  "indices": [],
  "sort": "Sort 1",
  "constructors": [
    {
      "name": "Nat.zero",
      "type": "Nat"
    },
    {
      "name": "Nat.succ",
      "type": "Nat → Nat"
    }
  ]
}
```

kernel は次を検査します。

```text
- constructor type が正しい
- 戻り値が対象inductiveである
- strict positivity を満たす
- universe制約が矛盾しない
- recursorを正しく生成できる
```

---

# 4. import hash

## 4.1 なぜ必要か

importを名前だけで管理すると危険です。

```text
import Std.Nat.Basic
```

と書いてあっても、実際に読み込まれた `Std.Nat.Basic` が期待したものか分かりません。

そこでimportにはhashを持たせます。

```json
{
  "module": "Std.Nat.Basic",
  "export_hash": "sha256:...",
  "certificate_hash": "sha256:..."
}
```

`export_hash` は常に必須です。
`certificate_hash` は通常モードでは省略可能ですが、高信頼モードでは必須にします。

## 4.2 export_hash と certificate_hash を分ける

ここは重要です。

module certificate には、最低限次の hash を持たせます。

```text
export_hash:
  downstream moduleが型検査・conversionに必要とする公開情報のhash

certificate_hash:
  proof本体も含むcertificate全体のhash

axiom_report_hash:
  canonical axiom report のhash
```

なぜ分けるかというと、opaque theorem の証明本体は下流の型検査には不要だからです。

たとえば：

```text
theorem T : P := proof1
```

を

```text
theorem T : P := proof2
```

に変えても、`T` が opaque なら、下流から見える型は同じです。

しかし監査上は proof が変わったことを知りたい。

そのため：

```text
export_hash:
  下流の型検査に影響するもの

certificate_hash:
  proofを含む完全な成果物
```

を分けます。

定義としては、`export_hash` は canonical export block だけに対する
`H("NPA_MODULE_EXPORT_V1" || export_block)` です。
`certificate_hash` は `certificate_hash` フィールド自身を除いた trusted certificate payload 全体に対する
`H("NPA_MODULE_CERT_V1" || full_certificate_payload_without_certificate_hash)` です。
debug metadata、source map、diagnostics、AI trace はどちらの hash にも含めません。

## 4.3 何を export_hash に含めるか

`export_hash` には次を含めます。

```text
- module名
- certificate format version
- core spec version
- exported declaration interface hashes
- exported inductive declarations
- transparent/reducible definitions の body hash
- opaque theorem の type hash
- theoremごとの axiom dependency summary
- universe declarations
```

注意点：

```text
transparent def の body は export_hash に含める必要がある
```

理由は、transparent definition は δ-reduction で展開され、下流のconversionに影響するからです。

一方：

```text
opaque theorem の proof body は export_hash に含めなくてもよい
```

ただし、その theorem がどのaxiomに依存しているかは export_hash に含めるべきです。

## 4.4 ImportEntry

import entry はこうします。

```rust
struct ImportEntry {
    module_name: ModuleName,
    export_hash: Hash,
    certificate_hash: Option<Hash>,
    imported_decls: Vec<DeclInterfaceHash>,
}
```

高信頼モードでは `certificate_hash` も必須にします。

```text
通常モード:
  export_hash一致を要求

高信頼モード:
  export_hash一致
  certificate_hash一致
  import先certificateも検査済み
```

---

# 5. declaration hash

## 5.1 宣言hashは1種類では足りない

宣言には、少なくとも次の2種類のhashを持たせるとよいです。

```text
decl_interface_hash:
  下流から見える意味のhash

decl_certificate_hash:
  proof/value本体を含む完全なhash
```

さらに実装上は、補助的に：

```text
type_hash
value_hash
proof_hash
```

も持つと便利です。

## 5.2 DefDecl のhash

transparent/reducibleな定義では、body が下流のconversionに影響します。

したがって：

```text
decl_interface_hash(def)
  = H(
      "NPA_DECL_IFACE_V1",
      kind = def,
      name,
      universe_params,
      type_hash,
      value_hash,
      reducibility
    )
```

```text
decl_certificate_hash(def)
  = H(
      "NPA_DECL_CERT_V1",
      decl_interface_hash,
      dependency_hashes,
      axiom_dependencies
    )
```

`DefDecl` では `value_hash` が interface に入ります。

## 5.3 TheoremDecl のhash

opaque theorem では、proof は下流のconversionに使われません。

そのため：

```text
decl_interface_hash(theorem)
  = H(
      "NPA_DECL_IFACE_V1",
      kind = theorem,
      name,
      universe_params,
      type_hash,
      opacity = opaque,
      axiom_dependencies
    )
```

```text
decl_certificate_hash(theorem)
  = H(
      "NPA_DECL_CERT_V1",
      decl_interface_hash,
      proof_hash,
      dependency_hashes
    )
```

この設計だと、proofだけを変更しても `decl_interface_hash` は変わらず、`decl_certificate_hash` だけが変わります。

ただし、proof変更によって使用axiomが変わる場合は `axiom_dependencies` が変わるので、interface hash も変わります。

これは望ましいです。

たとえば：

```text
theorem T : P := constructive_proof
```

から：

```text
theorem T : P := proof_using_Classical_choice
```

に変わった場合、下流の論理的信頼性が変わるため、interface hash も変わるべきです。

## 5.4 AxiomDecl のhash

AxiomDecl は、その存在自体がaxiom依存です。

```text
decl_interface_hash(axiom)
  = H(
      "NPA_DECL_IFACE_V1",
      kind = axiom,
      name,
      universe_params,
      type_hash
    )
```

axiom report では：

```text
axiom_dependencies(axiom) = { axiom.name }
```

になります。

## 5.5 InductiveDecl のhash

InductiveDecl は、constructors と recursor の計算規則に影響します。

```text
decl_interface_hash(inductive)
  = H(
      "NPA_DECL_IFACE_V1",
      kind = inductive,
      name,
      universe_params,
      params,
      indices,
      sort,
      constructors,
      generated_recursor_signature,
      computation_rules
    )
```

InductiveDecl は downstream の型検査・ι-reductionに影響するため、構造全体を interface hash に含めます。

---

# 6. axiom report

## 6.1 目的

axiom report は、各定理やモジュールがどのaxiomに依存しているかを明示するものです。

canonical payload では、axiom は canonical name order、per-declaration report は declaration order で保存します。
説明用 JSON では宣言名を key にした map のように書くことがありますが、hash 対象では map を使いません。

たとえば：

```json
{
  "module": "Std.Nat.Basic",
  "axioms_used": [],
  "declarations": {
    "Nat.add_zero": {
      "axioms_used": []
    }
  }
}
```

一方、古典論理を使う定理なら：

```json
{
  "declarations": {
    "Classical.some_theorem": {
      "axioms_used": [
        "Classical.choice",
        "Propext"
      ]
    }
  }
}
```

## 6.2 sorry は axiom として扱う

`sorry` や `admit` を許す場合、それは内部的には axiom として扱います。

```text
sorry : P
```

は事実上：

```text
axiom sorry_123 : P
```

と同じです。

したがって axiom report には必ず出します。

```json
{
  "axioms_used": [
    {
      "name": "synthetic.sorry.Std.Nat.Basic.add_zero",
      "kind": "sorry",
      "allowed": false
    }
  ]
}
```

高信頼モードでは、これは即failにします。

## 6.3 per-declaration report

各宣言ごとにaxiom依存を記録します。

```json
{
  "name": "Nat.add_zero",
  "kind": "theorem",
  "direct_axioms": [],
  "transitive_axioms": [],
  "status": "constructive"
}
```

古典公理を使う場合：

```json
{
  "name": "Classical.choice_example",
  "kind": "theorem",
  "direct_axioms": ["Classical.choice"],
  "transitive_axioms": ["Classical.choice"],
  "status": "uses_allowed_axioms"
}
```

## 6.4 axiom dependency の計算

宣言を依存関係順に並べ、順番に計算します。

```text
axioms(AxiomDecl a)
  = {a}

axioms(DefDecl d)
  = direct_axioms(type(d))
    ∪ direct_axioms(value(d))
    ∪ ⋃ axioms(dep) for dep in dependencies(d)

axioms(TheoremDecl t)
  = direct_axioms(type(t))
    ∪ direct_axioms(proof(t))
    ∪ ⋃ axioms(dep) for dep in dependencies(t)

axioms(InductiveDecl I)
  = direct_axioms(types in declaration)
    ∪ ⋃ axioms(dep) for dep in dependencies(I)
```

ここで `dependencies(d)` は、型や値や証明に現れる `Const` 参照です。

## 6.5 module-level report

モジュール全体では、各宣言のaxiom集合をunionします。

```json
{
  "module": "Std.Nat.Basic",
  "module_axioms": [],
  "per_declaration": [],
  "custom_axioms": [],
  "standard_axioms": [],
  "contains_sorry": false,
  "safe_for_high_trust": true
}
```

`custom_axioms`、`standard_axioms`、`contains_sorry`、`safe_for_high_trust` は audit/policy view です。
checker は保存された boolean を信用せず、canonical `module_axioms` と policy から毎回再計算します。

高信頼モードの判定：

```text
safe_for_high_trust =
  contains_sorry == false
  && custom_axioms == []
  && all standard_axioms are allowlisted
```

---

# 7. certificate生成パイプライン

Phase 2では、次のパイプラインを実装します。

```text
1. core declarations を受け取る
2. kernelで各宣言を検査する
3. canonical core AST に変換する
4. name/level/term table を canonical order で作る
5. term hash を計算する
6. declaration hash を計算する
7. dependency graph を作る
8. axiom report を作る
9. export block を作る
10. export_hash / certificate_hash / axiom_report_hash を計算する
11. .npcert を書き出す
12. checkerで .npcert と import store 内の .npcert だけを再検査する
```

疑似コード：

```rust
fn build_certificate(module: CoreModule, imports: Vec<VerifiedImport>) -> Result<ModuleCert> {
    let mut env = Env::from_imports(&imports);
    let mut decl_certs = Vec::new();

    for decl in module.declarations {
        check_declaration(&env, &decl)?;

        let canonical = canonicalize_decl(&decl)?;
        let deps = collect_dependencies(&canonical);
        let axiom_deps = compute_axiom_deps(&env, &deps, &canonical);

        let hashes = compute_decl_hashes(&canonical, &deps, &axiom_deps);

        let cert = DeclCert {
            canonical,
            deps,
            axiom_deps,
            hashes,
        };

        env.add_decl_interface(&cert)?;
        decl_certs.push(cert);
    }

    let export_block = build_export_block(&decl_certs);
    let axiom_report = build_axiom_report(&decl_certs);
    let tables = build_canonical_tables(&decl_certs)?;
    let hashes = compute_certificate_hashes(&imports, &tables, &export_block, &decl_certs, &axiom_report);

    Ok(ModuleCert {
        header: make_header(),
        imports,
        name_table: tables.names,
        level_table: tables.levels,
        term_table: tables.terms,
        declarations: decl_certs,
        export_block,
        axiom_report,
        hashes,
    })
}
```

---

# 8. certificate検査パイプライン

checker側は、sourceを見ません。

```text
.npcert
  ↓
parse canonical binary
  ↓
import hash確認
  ↓
declaration hash再計算
  ↓
kernel check
  ↓
axiom report再計算
  ↓
export_hash / certificate_hash / axiom_report_hash 再計算
  ↓
合格
```

疑似コード：

```rust
fn check_certificate(cert: ModuleCert, import_store: ImportStore) -> Result<VerifiedModule> {
    verify_header(&cert)?;

    let imports = load_and_check_imports(&cert.imports, import_store)?;
    let mut env = Env::from_imports(&imports);

    verify_canonical_tables(&cert.name_table, &cert.level_table, &cert.term_table)?;

    for decl in &cert.declarations {
        verify_canonical_encoding(decl)?;
        verify_decl_hashes(decl)?;

        check_declaration(&env, &decl.canonical)?;

        let deps = collect_dependencies(&decl.canonical);
        let ax = compute_axiom_deps(&env, &deps, &decl.canonical);

        if ax != decl.axiom_deps {
            return Err(Error::AxiomReportMismatch);
        }

        env.add_decl_interface(decl)?;
    }

    verify_module_axiom_report(&cert)?;
    verify_axiom_report_hash(&cert)?;
    verify_export_hash(&cert)?;
    verify_module_certificate_hash(&cert)?;

    Ok(VerifiedModule::new(cert))
}
```

---

# 9. 最小certificate例

たとえば `id` だけを含むモジュールを考えます。

```text
def id.{u} : Π A : Sort u, A → A :=
  λ A : Sort u, λ x : A, x
```

概念的なcertificateは：

```json
{
  "format": "NPA-CERT-0.1",
  "core_spec": "NPA-Core-0.1",
  "module": "Std.Logic.Id",
  "imports": [],
  "declarations": [
    {
      "kind": "def",
      "name": "id",
      "universe_params": ["u"],
      "type": {
        "core": "Pi (Sort u) (Pi (BVar 0) (BVar 1))",
        "hash": "term:..."
      },
      "value": {
        "core": "Lam (Sort u) (Lam (BVar 0) (BVar 0))",
        "hash": "term:..."
      },
      "reducibility": "reducible",
      "decl_interface_hash": "decl-iface:...",
      "decl_certificate_hash": "decl-cert:...",
      "axioms_used": []
    }
  ],
  "axiom_report": {
    "module_axioms": [],
    "per_declaration": [
      {
        "decl": "id",
        "axioms": []
      }
    ]
  },
  "hashes": {
    "export_hash": "sha256:...",
    "certificate_hash": "sha256:...",
    "axiom_report_hash": "sha256:..."
  }
}
```

実際には `core` は文字列ではなく、canonical binary term table になります。

---

# 10. エラー条件

Phase 2のcheckerは、次の場合にfailします。

```text
- import hash が一致しない
- certificate format version が非対応
- core AST がcanonicalでない
- unresolved metavariable が残っている
- declaration hash が再計算結果と違う
- term hash が再計算結果と違う
- proof : theorem_type が成り立たない
- def value : def_type が成り立たない
- inductive declaration がpositivityを満たさない
- axiom report が再計算結果と違う
- policy で禁止された axiom が含まれる
- policy で `deny_sorry` のとき sorry が含まれる
- export_hash が再計算結果と違う
- certificate_hash が再計算結果と違う
```

---

# 11. Phase 2 の完了条件

Phase 2が完了したと言える条件はこれです。

```text
- core term をcanonical binaryにできる
- binder名を変えても同じterm hashになる
- importに必須のexport_hashと、高信頼モード必須のcertificate_hashを持たせられる
- def/theorem/axiom/inductiveにdeclaration hashを持たせられる
- transparent def のbody変更でinterface hashが変わる
- opaque theorem のproof変更でdecl_certificate_hashとmodule certificate_hashが変わる
- opaque theorem のproof変更だけで type・opacity・axiom依存が変わらない場合、export_hashは維持される
- opaque theorem のproof変更でaxiom依存が変わる場合、export_hashも変わる
- axiom依存をdeclarationごとに計算できる
- module全体のaxiom reportをcanonical orderで出せる
- audit用のsafe_for_high_trust等を保存値として信用せず再計算できる
- .npcertとimport store内の.npcertだけを使ってkernel/checkerが再検査できる
- source code、source map、AI traceなしで検査が完結する
```

---

# 12. 設計の要点

Phase 2で一番大事なのは、hashの役割を分けることです。

```text
term_hash:
  core termそのもののhash

decl_interface_hash:
  下流から見える宣言の意味のhash

decl_certificate_hash:
  proof/value本体まで含む宣言全体のhash

export_hash:
  moduleの公開インターフェースのhash

certificate_hash:
  module certificate全体のhash

axiom_report_hash:
  canonical axiom report のhash
```

そして axiom report は、単なるログではなく、hashと同じくらい重要な検証対象にします。

最終的なPhase 2の一文要約はこれです。

```text
canonical core AST を binary certificate として固定し、
import・declaration・moduleにhashを与え、
各宣言のaxiom依存を再計算可能にし、
sourceなしでkernelが再検査できる形式にする。
```
