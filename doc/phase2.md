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
Phase 2 certificate verifier + Rust kernel で再検査
  ↓
Phase 8 independent checker でも同じ .npcert を再検査
```

---

# 1. Phase 2 の目的

Phase 2で作るものは、ざっくり言うと `.npcert` のような証明証明書ファイルです。

```text
Std/Nat/Basic.npa      -- 人間が書くソース
Std/Nat/Basic.npcert   -- checker が読む証明証明書。kernel は decode 済み core declaration を検査する
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
  canonical certificate を再検査する Phase 2 certificate verifier
  decode 済み core declaration を検査する kernel
```

ここでいう Phase 2 certificate verifier は、Phase 1 の Rust kernel を呼び出す
同一実装系内の decoder / verifier です。
Phase 8 の independent checker は、同じ `.npcert` を別実装または別プロセスで
再検査する後続成果物です。

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

canonical 形式は：

```text
Const {
  import_index,
  declaration_name,
  decl_interface_hash
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
term_hash(t) = H("NPA-TERM-0.1" || canonical_encode(t))
```

ここで `"NPA-TERM-0.1"` は domain separation 用の固定文字列です。

これにより、別の種類のhashと衝突しても意味的に混同しません。

たとえば：

```text
H("NPA-TERM-0.1" || ...)
H("NPA-DECL-CERT-0.1" || ...)
H("NPA-MODULE-EXPORT-0.1" || ...)
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
  "export_block": [],
  "axiom_report": {
    "module_axioms": [],
    "per_declaration": []
  },
  "hashes": {
    "export_hash": "sha256:...",
    "axiom_report_hash": "sha256:...",
    "certificate_hash": "sha256:..."
  }
}
```

実際の保存形式は JSON ではなく canonical binary にします。
JSONは説明用・デバッグ用で、検査用の本体はbinaryにします。
上の JSON も logical/debug view であり、hash 対象の canonical payload では map を使いません。
`export_block`、`axiom_report`、`hashes` は固定順の record または長さ付き配列として encode します。

### 3.2.1 canonical binary の最低条件

canonical binary は `core-spec-v0.1.md` の canonicalization / binary encoding
条件に従います。Phase 2 実装は、少なくとも次を満たします。

```text
- unsigned integer は minimal ULEB128
- string は byte length + UTF-8 bytes
- enum variant は固定 numeric tag
- record field order は固定
- array は explicit length つき
- map は hash 対象 payload では禁止
- optional field は explicit tag 0/1
- name table は UTF-8 byte lexicographic order
- level / term DAG は topological order、同順位は structural tag order
- import order は module name、export_hash、certificate_hash option/value の辞書順
- declarations は dependency order、同順位は declaration name の UTF-8 byte lexicographic order
- sha256 は canonical byte sequence そのものに対して計算する
```

非最短 ULEB128、未使用 term table entry、順序違反、invalid UTF-8、
hash 対象内 map は `NonCanonicalEncoding` として拒否します。
未知 enum tag は `UnsupportedEncoding` として拒否します。
source map、diagnostics、AI trace、表示名などの metadata は trusted payload と
hash 対象に含めません。

v0.1 の trusted `TermNode` schema には metavariable / hole / placeholder の variant を
持たせません。未解決 metavariable は certificate producer が `.npcert` 生成前に拒否する
対象であり、on-disk certificate では表現不能です。将来 pre-certificate API を追加する場合は
`UnresolvedMetavariable` を返しますが、v0.1 `.npcert` 内で metavariable 相当を unknown tag として
入れた場合は `UnsupportedEncoding` です。

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
  export_hash、axiom_report_hash、certificate_hash
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
  proof本体も含む trusted payload から certificate_hash 自身を除いたhash

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
  proofを含む trusted payload から certificate_hash 自身を除いたもの
```

を分けます。

定義としては、`export_hash` は canonical export block だけに対する
`H("NPA-MODULE-EXPORT-0.1" || export_block)` です。
`certificate_hash` は `certificate_hash` フィールド自身を除いた trusted certificate payload 全体に対する
`H("NPA-MODULE-CERT-0.1" || trusted_payload_without_certificate_hash)` です。
debug metadata、source map、diagnostics、AI trace はどちらの hash にも含めません。

## 4.3 何を export_hash に含めるか

`export_hash` には次を含めます。

```text
- exported declaration interface hashes
- exported inductive declarations
- transparent/reducible definitions の body hash
- opaque theorem の type hash
- exported declaration ごとの axiom dependency summary
- universe declarations
```

`export_hash` の hash 対象は 11.11 の `ExportBlockBytes(export_block)` だけです。
module名、certificate format version、core spec version は `certificate_hash` 側の
`ModuleCertBytesWithoutCertificateHash` に含めます。

注意点：

```text
transparent def の body は export_hash に含める必要がある
```

理由は、transparent definition は δ-reduction で展開され、下流のconversionに影響するからです。

一方：

```text
opaque theorem の proof body は export_hash に含めなくてもよい
```

ただし、その theorem がどのaxiomに依存しているかは export_hash に含めます。

## 4.4 ImportEntry

import entry はこうします。

```rust
struct ImportEntry {
    module_name: ModuleName,
    export_hash: Hash,
    certificate_hash: Option<Hash>,
}
```

これは core-spec の `Import` と同じく、canonical payload 上の import entry には
module name、`export_hash`、optional `certificate_hash` だけを入れるという意味です。

高信頼モードでは `certificate_hash` も必須にします。

```text
通常モード:
  export_hash一致を要求

高信頼モード:
  export_hash一致
  certificate_hash一致
  import先certificateも検査済み
```

import から見える宣言一覧は、import entry に重複して保存しません。
checker は検査済み import certificate の `export_block` から environment を作り、
各 `Const` / dependency entry が持つ `decl_interface_hash` と照合します。
実装上の cache として `imported_decls` 相当を持つ場合も、それは canonical payload と
hash 対象に含めない derived data です。

---

# 5. declaration hash

## 5.1 宣言hashは1種類では足りない

宣言には、少なくとも次の2種類のhashを持たせます。

```text
decl_interface_hash:
  下流から見える意味のhash

decl_certificate_hash:
  proof/value本体まで含む宣言全体のhash
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
      "NPA-DECL-IFACE-0.1",
      kind = def,
      name,
      universe_params,
      type_hash,
      reducibility,
      public_dependency_entries,
      axiom_dependencies,
      value_hash if reducibility = reducible
    )
```

```text
decl_certificate_hash(def)
  = H(
      "NPA-DECL-CERT-0.1",
      decl_interface_hash,
      value_hash,
      dependency_entries,
      axiom_dependencies
    )
```

reducible な `DefDecl` では `value_hash` が interface に入ります。
opaque def では `value_hash` は interface には入らず、`decl_certificate_hash` にだけ入ります。
さらに、公開される type / reducible body に現れる `Const` の `DependencyEntry` も
interface に入れます。これにより local declaration index だけではなく、参照先の
`decl_interface_hash` 変更も downstream の `export_hash` に伝播します。
opaque def では body 自体の non-axiom dependency は公開 interface に入れません。

## 5.3 TheoremDecl のhash

opaque theorem では、proof は下流のconversionに使われません。

そのため：

```text
decl_interface_hash(theorem)
  = H(
      "NPA-DECL-IFACE-0.1",
      kind = theorem,
      name,
      universe_params,
      type_hash,
      opacity = opaque,
      public_dependency_entries,
      axiom_dependencies
    )
```

```text
decl_certificate_hash(theorem)
  = H(
      "NPA-DECL-CERT-0.1",
      decl_interface_hash,
      proof_hash,
      dependency_entries
    )
```

この設計だと、proofだけを変更しても `decl_interface_hash` は変わらず、`decl_certificate_hash` だけが変わります。

ただし、proof変更によって使用axiomが変わる場合は `axiom_dependencies` が変わるので、interface hash も変わります。

この挙動を採用します。

たとえば：

```text
theorem T : P := constructive_proof
```

から：

```text
theorem T : P := proof_using_Classical_choice
```

に変わった場合、下流の論理的信頼性が変わるため、interface hash も変わります。

## 5.4 AxiomDecl のhash

AxiomDecl は、その存在自体がaxiom依存です。

```text
decl_interface_hash(axiom)
  = H(
      "NPA-DECL-IFACE-0.1",
      kind = axiom,
      name,
      universe_params,
      type_hash,
      public_dependency_entries
    )
```

`public_dependency_entries` は axiom の type に現れる direct `Const` 参照から導出します。
この点は 11.11 の `DeclInterfacePayload` と同じ規則です。

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
      "NPA-DECL-IFACE-0.1",
      kind = inductive,
      name,
      universe_params,
      params,
      indices,
      sort,
      constructors,
      generated_recursor_signature_hash,
      generated_computation_rule_hash,
      public_dependency_entries,
      axiom_dependencies
    )
```

InductiveDecl は downstream の型検査・ι-reductionに影響するため、構造全体を interface hash に含めます。
`constructors` は constructor name と constructor type hash を直接入れます。
recursor の signature と computation rule は直接展開せず、それぞれ専用の generated artifact hash に分離します。

---

# 6. axiom report

## 6.1 目的

axiom report は、各定理やモジュールがどのaxiomに依存しているかを明示するものです。

canonical payload では、axiom は後述する `AxiomRef` の canonical order、
per-declaration report は declaration order で保存します。
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
7. 宣言ごとの dependency entries を作る
8. axiom report を作る
9. export block を作る
10. export_hash / certificate_hash / axiom_report_hash を計算する
11. .npcert を書き出す
12. checker が .npcert と import store 内の .npcert を decode し、kernel は decode 済み宣言を再検査する
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

## 7.1 producer 分離方針

Phase 2 の trusted boundary は、producer を分けても変えません。
ここでいう producer は、`.npcert` を作る前に `CoreModule` または core declaration 候補を
生成する非信頼層です。

```text
Human producer:
  human source / notation / display name / source map
  ↓
  CoreModule

AI producer:
  structured AI request / explicit core-like term / batch candidate
  ↓
  CoreModule or CoreDeclCandidate

共通:
  CoreModule
  ↓
  npa-cert build / verify
  ↓
  npa-kernel check
```

重要なのは、`npa-cert` と `npa-kernel` が producer 種別を信用しないことです。
AI 由来か人間由来かは trusted payload に入りません。
同じ `CoreModule` と同じ import set からは、同じ `.npcert` bytes と同じ hash が得られなければいけません。

```text
trusted payload に入れない:
  producer kind
  source text
  source map
  pretty/display name
  elaborator trace
  tactic trace
  AI prompt / completion / score / trace
  model name
  search rank
  cache hit / cache miss
```

producer metadata が必要な場合は、`.npcert` の外側の debug sidecar / audit sidecar として保存します。
sidecar の追加・削除・内容変更で `term_hash`、`decl_interface_hash`、`export_hash`、
`certificate_hash`、`axiom_report_hash` が変わってはいけません。

### 7.1.1 Human producer

Human producer は、Phase 3 以降の人間向け surface language から `CoreModule` を作ります。
ここでは読みやすさを優先してよいです。

```text
許可される非信頼入力:
  source text
  namespace / open
  notation
  implicit arguments
  holes before elaboration
  display binder name
  source location
  diagnostics
```

ただし `.npcert` に渡す直前では、次を満たす必要があります。

```text
- unresolved metavariable / hole がない
- notation / implicit argument / typeclass placeholder が残っていない
- binder name に依存しない de Bruijn 表現に変換できる
- universe level が明示されている
- global constant 参照が import hash と decl_interface_hash で固定されている
```

Human producer が source map や display name を作る場合も、それは sidecar です。
kernel check、certificate hash、export hash の入力にしてはいけません。

### 7.1.2 AI producer

AI producer は、多数の候補を高速に試すため、人間向け surface language を経由しません。
入力はできるだけ canonical core に近い構造化形式にします。

AI producer MVP は、次を使いません。

```text
- notation
- open / namespace に依存する short name
- overload resolution
- implicit argument insertion
- unresolved hole
- tactic script text
- source-level axiom declaration
- source-level inductive syntax
- typeclass search
- numeric literal overload
```

AI producer MVP が生成してよいものは、次に限定します。

```text
- fully qualified global reference as lookup input only
- explicit universe application
- Sort / Pi / Lam / App / Let / Const / BVar
- def / theorem の core declaration
- verified import export_block に存在する GlobalRef
- current module の prior declaration への Local ref
```

ここでいう fully qualified global reference は、AI producer の入力解決用です。
`CoreDeclCandidate` の core term 内に保存される `Const` は、名前だけではなく
`GlobalRef::Imported(import_index, name, decl_interface_hash)`、`GlobalRef::Local(decl_index)`、
`GlobalRef::LocalGenerated(decl_index, name)`、`GlobalRef::Builtin(name, decl_interface_hash)` のような
Phase 2 の hash-bound `GlobalRef` payload でなければいけません。
pretty name / fully qualified name だけを certificate-facing core term として扱ってはいけません。

AI producer の出力は、直接 trusted certificate ではありません。
まず `CoreDeclCandidate` または `CoreModule` として受け取り、`npa-cert` が通常どおり
canonicalization、hash 再計算、axiom dependency 計算、kernel check を行います。

### 7.1.3 AI candidate fast path

AI 探索では、候補ごとに完全な `.npcert` を作ると重いです。
そのため Phase 2 の前段に、certificate を発行しない candidate fast path を置いてよいです。

```text
AI candidate batch
  ↓
schema / size limit check
  ↓
import ref validation
  ↓
kernel precheck
  ↓
candidate accepted?
  ↓ yes
CoreDeclCandidate / CheckedDeclCandidate
  ↓
採用候補だけ build_module_cert
```

fast path が返す成功は、「この候補は現在の verified import environment で kernel precheck に通った」
という意味だけです。証明済み module ではありません。
最終成果物にする場合は、必ず `build_module_cert` で `.npcert` を生成し、
`verify_module_cert` で再検査します。

fast path は次を省略してはいけません。

```text
- core AST の schema validation
- unresolved metavariable / placeholder の拒否
- import GlobalRef の decl_interface_hash 照合
- declaration dependency の well-scoped 性確認
- proof : theorem_type / value : def_type の kernel check
- universe level の well-formedness
```

fast path で省略してよいものは、成果物化に必要な処理だけです。

```text
省略してよい:
  .npcert byte emission
  module certificate_hash 計算
  export_block 全体の確定
  sidecar 生成

省略してはいけない:
  kernel check
  import interface check
  unresolved hole rejection
  resource limit enforcement
```

### 7.1.4 batch / cache

AI producer は batch API を使ってよいです。
batch は同じ import environment と同じ prior declarations に対する候補群をまとめて検査します。

```text
Batch input:
  verified imports
  checked prior current declarations
  candidates[]
  deterministic budget
  resource limit

Batch output:
  per-candidate success / structured error
  optional term_hash / decl hash preview
  optional normalized core size
  no trusted certificate
```

性能のために次の cache を使ってよいです。

```text
- import kernel environment cache keyed by import module + export_hash
- import certificate / high-trust verification cache keyed by import module + export_hash + certificate_hash
- name / level / term hash-consing cache
- WHNF cache
- conversion cache
- declaration dependency cache
```

cache は検査結果の正しさの根拠ではありません。
cache hit / miss は trusted payload、hash、axiom report、certificate identity に入れてはいけません。
cache を無効化しても同じ入力から同じ成功/失敗と同じ certificate bytes が得られる必要があります。

### 7.1.5 producer sidecar

producer は、debug / audit / training 用に sidecar を出してよいです。

```rust
struct ProducerSidecar {
    module: ModuleName,
    producer_profile: ProducerProfile,
    producer_run_id: String,
    candidate_count: u64,
    accepted_candidate_count: u64,
    diagnostics: Vec<ProducerDiagnostic>,
    input_artifact_hashes: Vec<Hash>,
}

enum ProducerProfile {
    HumanSurface,
    AiCoreMvp,
}
```

この sidecar は `.npcert` とは別 artifact です。
`ModuleCertBytes` の trusted payload には含めません。
Phase 2 verifier は sidecar を読まなくても `.npcert` を検査できなければいけません。

### 7.1.6 禁止する shortcut

AI producer 分離で、次の shortcut は禁止します。

```text
- AI producer が計算した hash を trusted hash として採用する
- AI producer が `verified` と言った宣言の kernel check を省く
- candidate fast path の成功を `.npcert` verification success と同一視する
- producer kind によって verifier の検査項目を減らす
- pretty name / source span / model score から GlobalRef を補完する
- import store を producer が暗黙に filesystem / network から補完する
- unresolved metavariable を certificate schema に表現する
```

producer は速度のために分けますが、証明の正本は常に canonical certificate です。

---

# 8. certificate検査パイプライン

Phase 2 の checker は、certificate verifier です。
これは canonical binary を decode し、hash / axiom report / import を再計算し、
decode 済み declaration を Phase 1 Rust kernel に渡して検査します。
source、source map、elaborator trace、tactic trace、AI trace は見ません。

この verifier は Phase 8 の independent checker ではありません。
Phase 8 では、この Phase 2 の `.npcert` 形式を入力として、別実装または別プロセスの
checker が同じ検査を行います。

```text
.npcert
  ↓
parse canonical binary
  ↓
import の export_hash / high-trust 時の certificate_hash 確認
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
  "export_block": [
    {
      "name": "id",
      "decl_interface_hash": "decl-iface:..."
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
    "axiom_report_hash": "sha256:...",
    "certificate_hash": "sha256:..."
  }
}
```

実際には `core` は文字列ではなく、canonical binary term table になります。

---

# 10. エラー条件

Phase 2のcheckerは、次の場合にfailします。

```text
- import の export_hash、または high-trust 時の certificate_hash が一致しない
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

# 11. Phase 2 実装契約

この章は Phase 2 実装時の contract です。
前の章で説明した設計意図を、実装が迷わない粒度まで落とします。
ここにない型検査・conversion・inductive の論理規則は `core-spec-v0.1.md` と
Phase 1 kernel の仕様に従います。

## 11.1 crate / API 境界

Phase 2 では certificate 処理を kernel から分離します。

```text
crates/npa-kernel:
  core Expr / Level / Decl
  type checking
  definitional equality
  reduction
  inductive checking
  no file I/O
  no serialization
  no hashing
  no policy decision

crates/npa-cert:
  canonicalization
  canonical binary encode/decode
  hash calculation
  axiom dependency calculation
  import store / high-trust policy
  certificate build / verify
  calls npa-kernel with decoded declarations
```

Phase 2 で公開する最小 API はこれです。

```rust
pub type Hash = [u8; 32];

pub struct Name(pub Vec<String>);

pub type ModuleName = Name;
pub type AxiomName = Name;

pub struct CoreModule {
    pub name: ModuleName,
    pub declarations: Vec<npa_kernel::Decl>,
}

pub enum TrustMode {
    Normal,
    HighTrust,
}

pub struct AxiomPolicy {
    pub mode: TrustMode,
    pub allowlisted_axioms: BTreeSet<AxiomName>,
    pub deny_sorry: bool,
}

pub struct VerifierSession {
    checked: BTreeMap<ImportKey, VerifiedModule>,
}

pub fn build_module_cert(
    module: CoreModule,
    imports: &[VerifiedModule],
) -> Result<ModuleCert, CertError>;

pub fn encode_module_cert(cert: &ModuleCert) -> Result<Vec<u8>, CertError>;

pub fn decode_module_cert(bytes: &[u8]) -> Result<ModuleCert, CertError>;

pub fn verify_module_cert(
    bytes: &[u8],
    session: &mut VerifierSession,
    policy: &AxiomPolicy,
) -> Result<VerifiedModule, CertError>;
```

`verify_module_cert` は decode、canonical encoding 検査、hash 再計算、
import 解決、axiom policy、kernel check をまとめて行います。
`decode_module_cert` は構文的 decode だけで、trusted module としては扱いません。

`npa-kernel` は path、filesystem、network、時計、乱数、環境変数を見ません。
canonical bytes の生成では `HashMap` / `HashSet` の iteration order に依存してはいけません。
順序が必要な場合は `BTreeMap` / `BTreeSet` または明示 sort 済み `Vec` を使います。

### 11.1.1 producer API 境界

Human producer と AI producer は `npa-cert` の外側に置きます。
`npa-cert` は producer-specific な入力を直接 trusted artifact として受け取りません。
`ProducerProfile` は sidecar / audit 用の分類であり、certificate build / verify API の引数にしてはいけません。

```rust
// sidecar / audit only
pub enum ProducerProfile {
    HumanSurface,
    AiCoreMvp,
}

pub struct ProducerLimits {
    pub max_declarations: u32,
    pub max_expr_nodes: u32,
    pub max_level_nodes: u32,
    pub max_name_components: u32,
    pub max_reduction_steps: u64,
    pub max_conversion_steps: u64,
}

pub struct CoreDeclCandidate {
    pub declaration: npa_kernel::Decl,
}

pub struct CandidateBatch<'a> {
    pub imports: &'a [VerifiedModule],
    pub prior_current_decls: &'a [CheckedDeclCandidate],
    pub candidates: Vec<CoreDeclCandidate>,
    pub limits: ProducerLimits,
}

pub struct CheckedDeclCandidate {
    // private fields; construct only through check_core_decl_candidates
    declaration: npa_kernel::Decl,
    preview_hashes: CandidateHashPreview,
    pre_env_fingerprint: Hash,
    post_env_fingerprint: Hash,
    prior_chain_fingerprint: Hash,
    limits: ProducerLimits,
    limit_profile_hash: Hash,
    decl_interface_hash: Hash,
    decl_certificate_hash: Hash,
}

pub struct CandidateHashPreview {
    pub type_hash: Option<Hash>,
    pub body_hash: Option<Hash>,
    pub decl_interface_hash: Option<Hash>,
    pub decl_certificate_hash: Option<Hash>,
}

pub enum CandidateStatus {
    Accepted(CheckedDeclCandidate),
    Rejected(CertError),
}

pub struct CandidateBatchResult {
    // one status per input candidate, in the same order
    pub statuses: Vec<CandidateStatus>,
}

pub fn check_core_decl_candidates(
    batch: CandidateBatch<'_>,
) -> Result<CandidateBatchResult, CertError>;

pub fn build_module_cert_from_checked_candidates(
    module_name: ModuleName,
    imports: &[VerifiedModule],
    checked_decls: &[CheckedDeclCandidate],
) -> Result<ModuleCert, CertError>;
```

`check_core_decl_candidates` は AI 探索の fast path 用 API です。
この API は `.npcert` を生成せず、`VerifiedModule` も返しません。
`Accepted` は、候補が与えられた import/prior declaration 環境と limit の下で
schema validation と kernel precheck に通ったことだけを意味します。
`CandidateBatch.imports` は `ModuleCert.Imports` と同じ canonical import order で渡します。
`CoreDeclCandidate` 内の `GlobalRef::Imported(import_index, ...)` は、この
`batch.imports[import_index]` を参照します。
`check_core_decl_candidates` は import order が canonical でない場合、または
`ProducerImportEnvKey(module, export_hash)` が重複する場合、batch-level `Err(CertError)` で拒否します。
同じ module / export_hash の異なる certificate hash は producer public environment として同一なので、
candidate fast path の direct imports に同時に入れてはいけません。
`Err(CertError)` は、`prior_current_decls` の token 検証失敗、batch schema 全体の不正、limit profile の不整合など、
候補ごとの `Rejected` に分解できない batch-level failure を表します。
`Ok(result)` の場合、`result.statuses.len() == batch.candidates.len()` であり、`statuses[i]` は
`batch.candidates[i]` の結果です。この対応は入力順そのもので、score、hash、成功/失敗、cache 状態で
並べ替えてはいけません。

`CheckedDeclCandidate` は opaque な checked token です。
caller が任意の `npa_kernel::Decl` から直接作れる型にしてはいけません。
実装では field を private にし、`check_core_decl_candidates` だけが生成します。
`prior_current_decls` に渡された各 token は、次の条件を満たす場合にだけ環境へ追加できます。

```text
- 最初の token の pre_env_fingerprint が batch.imports から再計算した initial env fingerprint と一致する
- 2個目以降の token の pre_env_fingerprint が、直前 token の post_env_fingerprint と一致する
- token の prior_chain_fingerprint が、それ以前の accepted prior declarations の chain と一致する
- token の private `decl_interface_hash` / `decl_certificate_hash` が declaration から再計算した hash と一致する
- token の declaration は同じ limit profile か、より厳しい deterministic limit profile で kernel precheck 済みである
- token の post_env_fingerprint が、pre_env_fingerprint の producer public environment に token.declaration interface を追加して再計算した fingerprint と一致する
```

この検査に失敗した prior token がある場合、batch 全体を deterministic な structured error で拒否します。
unchecked raw declaration を `prior_current_decls` として受け取る API を別に作る場合は、
その API が先頭から順に全 prior declaration を再検査し、同じ `CheckedDeclCandidate` token に変換してから
後続 candidate を検査します。

`ProducerLimits` は canonical bytes を持ちます。
field order は struct 定義順で、各 field は minimal ULEB128 として encode します。

```text
producer_limits_hash(limits) =
  sha256("NPA-PRODUCER-LIMITS-0.1" || canonical_encode(limits))
```

`CheckedDeclCandidate.limit_profile_hash` は、この hash です。
ある prior token を batch の `limits` で再利用できるのは、token を作った limit profile が現在の
`batch.limits` と同一、または現在の `batch.limits` より厳しい場合だけです。
厳しさは全 field の上限で比較します。

```text
stricter_or_equal(a, b) =
  a.max_declarations      <= b.max_declarations
  && a.max_expr_nodes     <= b.max_expr_nodes
  && a.max_level_nodes    <= b.max_level_nodes
  && a.max_name_components <= b.max_name_components
  && a.max_reduction_steps <= b.max_reduction_steps
  && a.max_conversion_steps <= b.max_conversion_steps
```

ここで `a` は token 作成時の limits、`b` は現在の batch limits です。
この比較に必要な元の `ProducerLimits` は、token 内部の private field `limits` に保存します。
`limit_profile_hash` は token の同一性・ログ・diagnostic 用であり、hash だけで厳しさを推測してはいけません。

producer token 用の environment / chain fingerprint は Phase 2 が正本です。
Phase 4 / Phase 5 の proof-state fingerprint と混同してはいけません。
canonical bytes はすべて fixed record order で encode し、hash は次の domain separator を使います。

```text
producer_env_fingerprint(env) =
  sha256("NPA-PRODUCER-ENV-0.1" || ProducerEnvFingerprintBytes(env))

ProducerEnvFingerprintBytes(env):
  direct_imports: vec<ProducerImportEnvKey>      -- canonical import order
  checked_decls: vec<ProducerCheckedDeclInterface>

ProducerImportEnvKey:
  module: ModuleName
  export_hash: hash

ProducerCheckedDeclInterface:
  decl_interface_hash: hash
  axiom_dependencies: vec<AxiomRef>   -- canonical order
```

`producer_env_fingerprint` は pure kernel environment だけの fingerprint ではありません。
AI producer の再利用単位として、import から再構成される kernel environment、current module の
公開 declaration interface、下流の信頼性に影響する axiom trust summary をまとめた
producer public environment fingerprint です。

`ProducerImportEnvKey` は、この producer public environment の import 部分を表します。
import から再構成される kernel environment の同一性だけを固定するため、`certificate_hash` は含めません。
import 先 module の proof 本体だけが変わり、`export_hash` が同じ場合、下流の kernel environment は
変わらないためです。
import certificate identity、高信頼モードの検証済み状態、audit 用の exact import chain は、
`ImportEntry.certificate_hash`、`VerifiedModule.certificate_hash`、または high-trust verification cache で扱います。

`ProducerCheckedDeclInterface` は declaration name を別 field として持ちません。
declaration identity は `decl_interface_hash` に含まれる name / kind / type / public dependency 情報で表します。
また、`decl_certificate_hash` は含めません。
opaque theorem の proof や opaque def の body だけが変わり、公開 interface と axiom dependencies が同じ場合、
producer public environment は変わらないためです。
証明本体・値本体まで含む exact token sequence の固定は、`CheckedDeclCandidate` の private hash と
`ProducerPriorChainEntry.decl_certificate_hash` で行います。
duplicate name、declaration order、module-level visibility の検査は、最終的に `build_module_cert` が
`CoreModule` 全体に対して行います。producer env fingerprint は token chain の producer public environment 同一性を固定するための
補助 hash であり、module validity check の代替ではありません。

`ProducerCheckedDeclInterface.axiom_dependencies` は、certificate generation と同じ規則で計算します。
ここでは fingerprint 用の canonical bytes と、dependency / axiom lookup に使う operational environment を分けます。
fingerprint bytes は deterministic identity のための最小表現です。
operational environment は、`VerifiedModule.export_block` と prior checked declaration interface から作る lookup view で、
hash payload そのものではありません。

```text
ProducerLookupEnv:
  import_exports: vec<ExportBlockView>          -- batch.imports と同じ canonical import order
  checked_decls: vec<ProducerCheckedDeclInterface>

producer_lookup_env(imports, checked_decls):
  import_exports = canonical_import_export_views(imports)
  checked_decls = checked_decls

producer_checked_decl_interface(decl, lookup_env):
  canonical = canonicalize_decl(decl)
  deps = collect_dependencies(canonical)
  axiom_dependencies = compute_axiom_deps(lookup_env, deps, canonical)
  hashes = compute_decl_hashes(canonical, deps, axiom_dependencies)
  return {
    decl_interface_hash = hashes.decl_interface_hash,
    axiom_dependencies = canonical_order(axiom_dependencies)
  }
```

`ProducerEnvFingerprintBytes` と `ProducerLookupEnv` は同じ producer public environment を
別目的で表したものです。前者は hash 用、後者は `compute_axiom_deps` / dependency lookup 用です。
`canonical_import_env_keys(imports)` と `canonical_import_export_views(imports)` は、
同じ canonical import order を保存しなければいけません。
`GlobalRef::Imported(import_index, ...)` は、この順序の `imports[import_index]`、
`direct_imports[import_index]`、`import_exports[import_index]` を同時に指します。
`ProducerImportEnvKey(module, export_hash)` だけから import 内の axiom dependencies を lookup してはいけません。
AI producer が渡した dependency report や preview hash を axiom dependency の根拠にしてはいけません。

initial producer public environment は imports だけを含み、checked declarations は空です。

```text
initial_env_fingerprint(imports) =
  producer_env_fingerprint({
    direct_imports = canonical_import_env_keys(imports),
    checked_decls = []
  })
```

declaration を1つ追加した後の producer public environment fingerprint は、直前の environment bytes を再利用せず、
imports と checked declaration interface sequence 全体から再計算します。
これにより、実装ごとの incremental cache 差が fingerprint に入らないようにします。

```text
post_env_fingerprint(imports, checked_decls_before, decl) =
  pre_env_bytes = {
    direct_imports = canonical_import_env_keys(imports),
    checked_decls = checked_decls_before
  }
  lookup_env = producer_lookup_env(imports, checked_decls_before)
  producer_env_fingerprint({
    direct_imports = pre_env_bytes.direct_imports,
    checked_decls = pre_env_bytes.checked_decls ++ [producer_checked_decl_interface(decl, lookup_env)]
  })
```

prior chain fingerprint は、current module 内の checked declarations の順序を固定するための hash です。
imports は env fingerprint 側に含まれるため、chain fingerprint には含めません。

```text
prior_chain_fingerprint(chain) =
  sha256("NPA-PRODUCER-CHAIN-0.1" || ProducerPriorChainBytes(chain))

ProducerPriorChainBytes(chain):
  checked_decls: vec<ProducerPriorChainEntry>

ProducerPriorChainEntry:
  decl_interface_hash: hash
  decl_certificate_hash: hash
  pre_env_fingerprint: hash
  post_env_fingerprint: hash
```

最初の token の `prior_chain_fingerprint` は空 chain から計算します。
2個目以降の token では、それ以前の accepted prior declarations の
`ProducerPriorChainEntry` sequence から再計算した値と一致しなければいけません。

`preview_hashes` はログ、dedupe、ranking の補助に使ってよいですが、token validation の根拠にしてはいけません。
`CandidateHashPreview` の各 field は optional であり、存在していても non-authoritative です。
accepted token の private `decl_interface_hash` / `decl_certificate_hash` と異なる preview hash がある場合は、
実装は diagnostic として報告してよいですが、trusted hash として採用してはいけません。
最終的な `decl_interface_hash`、`decl_certificate_hash`、`export_hash`、`certificate_hash` は
`build_module_cert` と `verify_module_cert` が再計算した値だけを信用します。

`build_module_cert_from_checked_candidates` は、accepted token だけから最終 `ModuleCert` を作る補助 API です。
この API は各 token の `pre_env_fingerprint`、`post_env_fingerprint`、`prior_chain_fingerprint`、
`producer_limits_hash(token.limits) == token.limit_profile_hash`、private decl hashes を再検証し、
token chain が `imports` と `checked_decls` の順序に完全に一致する場合だけ、内部で `CoreModule` を構成して
`build_module_cert` を呼びます。token から declaration を取り出す public getter を作って caller に
raw `CoreModule` を組ませてはいけません。
この API は新しい `ProducerLimits` との strictness 判定をしません。
strictness 判定は `check_core_decl_candidates` が prior token を現在の `batch.limits` で再利用するときだけ行います。

## 11.2 canonical binary primitive

すべての payload は次の primitive だけで encode します。

```text
uvar:
  minimal unsigned LEB128
  non-minimal form は NonCanonicalEncoding

tag:
  exactly one byte
  unknown tag は UnsupportedEncoding

bytes:
  uvar byte_length + raw bytes

string:
  bytes と同じ形
  content は valid UTF-8
  invalid UTF-8 は NonCanonicalEncoding

vec<T>:
  uvar length + element bytes in order

option<T>:
  tag 0x00 for none
  tag 0x01 + T for some

hash:
  raw 32-byte sha256 digest
  "sha256:..." hex string は debug view のみ

record:
  fields concatenated in the order specified below
  field name は encode しない
```

同じ値は常に同じ bytes に encode されます。
decode 後に再encodeした bytes が入力と一致しない場合、checker は
`NonCanonicalEncoding` として拒否します。

## 11.3 canonical name / id

canonical payload 内の名前は dotted string ではなく component list です。

```text
Name:
  vec<string component>

NameId:
  uvar index into name_table
```

name の順序は、components を左から順に UTF-8 byte lexicographic order で比較します。
片方がもう片方の prefix の場合、短い方を先にします。

Name は non-empty component list で、各 component は空文字列または `.` を含む文字列であってはいけません。
`name_table` は重複なし、上記順序で sort 済みです。
display name や source binder name は trusted payload に入りません。

## 11.4 module certificate byte schema

on-disk `.npcert` の trusted payload は次の順序で encode します。

```text
ModuleCertBytes =
  Header
  Imports
  NameTable
  LevelTable
  TermTable
  Declarations
  ExportBlock
  AxiomReport
  ModuleHashes

Header =
  format: string              -- "NPA-CERT-0.1"
  core_spec: string           -- "NPA-Core-0.1"
  module: Name

Imports =
  vec<ImportEntry>

ImportEntry =
  module: Name
  export_hash: hash
  certificate_hash: option<hash>

NameTable =
  vec<Name>

LevelTable =
  vec<LevelNode>

TermTable =
  vec<TermNode>

Declarations =
  vec<DeclCert>

ModuleHashes =
  export_hash: hash
  axiom_report_hash: hash
  certificate_hash: hash
```

`certificate_hash` の計算時だけは、最後の `ModuleHashes` を次に置き換えます。

```text
ModuleHashesForCertificateHash =
  export_hash: hash
  axiom_report_hash: hash
```

つまり `certificate_hash` は、自分自身の field tag や placeholder bytes を含めません。

`ModuleCertBytes` は trusted payload だけです。
source map、diagnostics、AI trace、elaborator trace、tactic trace、display name などの
metadata field はこの schema に存在しません。したがって v0.1 実装では、metadata を
追加・削除して trusted hash が変わらないことは「metadata を trusted payload に encode
できない」ことによって満たします。将来 debug / audit 用 sidecar を追加する場合も、
sidecar bytes は `export_hash`、`axiom_report_hash`、`certificate_hash` の入力に入れてはいけません。

## 11.5 level schema

`LevelTable` は topological order です。
子 level を持つ node は、自分より小さい `LevelId` だけを参照できます。
同一構造の level node は重複禁止です。

```text
LevelId =
  uvar index into level_table

LevelNode =
  0x00 Zero
  0x01 Succ(inner: LevelId)
  0x02 Max(lhs: LevelId, rhs: LevelId)
  0x03 IMax(lhs: LevelId, rhs: LevelId)
  0x04 Param(name: NameId)
```

`Max` / `IMax` は Phase 1 の `normalize_level` 後の形だけを保存します。
非正規化 level は `NonCanonicalEncoding` として拒否します。

canonical `LevelTable` は、module 内で到達可能な normalized level node を
次の sort key で並べたものです。

```text
LevelSortKey =
  (height, LevelNodeKey)

height(Zero) = 0
height(Param) = 0
height(Succ x) = height(x) + 1
height(Max x y) = max(height(x), height(y)) + 1
height(IMax x y) = max(height(x), height(y)) + 1
```

`LevelNodeKey` は tag と field の canonical bytes です。
`height` を先に比較するため、子 node は必ず親より前に現れます。
同じ `LevelNodeKey` が2回現れたら duplicate として拒否します。

## 11.6 term schema

`TermTable` も topological order です。
子 term を持つ node は、自分より小さい `TermId` だけを参照できます。
同一構造の term node は重複禁止です。

```text
TermId =
  uvar index into term_table

TermNode =
  0x00 Sort(level: LevelId)
  0x01 BVar(index: uvar)
  0x02 Const(global_ref: GlobalRef, levels: vec<LevelId>)
  0x03 App(fun: TermId, arg: TermId)
  0x04 Lam(type: TermId, body: TermId)
  0x05 Pi(type: TermId, body: TermId)
  0x06 Let(type: TermId, value: TermId, body: TermId)

GlobalRef =
  0x00 Imported(import_index: uvar, name: NameId, decl_interface_hash: hash)
  0x01 Local(decl_index: uvar)
  0x02 LocalGenerated(decl_index: uvar, name: NameId)
  0x03 Builtin(name: NameId, decl_interface_hash: hash)
```

`Imported.import_index` は `imports` の index です。
`Imported.name` と `decl_interface_hash` は、その import の `export_block` に存在する
entry と一致しなければいけません。
`Local.decl_index` は現在 module の declaration index です。
`LocalGenerated.decl_index` は現在 module の `InductiveDecl` の declaration index で、
`LocalGenerated.name` はその `InductiveDecl` から生成された constructor または recursor の
`NameId` です。checker は `decl_index` の `InductiveDecl` 内に同じ name の
`ConstructorSpec` / `RecursorSpec` が存在することを確認します。
`Builtin.name` は checker builtin profile が提供する stable name です。
`decl_interface_hash` は builtin interface tag から決定的に再計算できなければならず、
v0.1 では `Nat` / `Nat.zero` / `Nat.succ` / `Nat.rec` / `Eq` / `Eq.refl` / `Eq.rec`
だけを許します。`Eq.rec` は builtin axiom interface なので、これを参照する declaration は
axiom report に `Builtin(Eq.rec)` を含めます。
Phase 2 では mutual declaration を扱わないため、local dependency は現在の declaration より
小さい index だけを許します。
ただし同じ `InductiveDecl` bundle 内の inductive self reference と generated artifact reference は、
declaration graph の cycle とはみなしません。

`Lam` / `Pi` / `Let` には binder name を保存しません。
de Bruijn index が範囲外なら `InvalidBVar` として拒否します。

canonical `TermTable` は、module 内で到達可能な term node を次の sort key で
並べたものです。

```text
TermSortKey =
  (height, TermNodeKey)

height(Sort _) = 0
height(BVar _) = 0
height(Const _ _) = 0
height(App f a) = max(height(f), height(a)) + 1
height(Lam t b) = max(height(t), height(b)) + 1
height(Pi t b) = max(height(t), height(b)) + 1
height(Let t v b) = max(height(t), height(v), height(b)) + 1
```

`TermNodeKey` は tag と field の canonical bytes です。
子 term field は child `term_hash`、level field は `level_hash` を使います。
同じ `TermNodeKey` が2回現れたら duplicate として拒否します。
この規則により、AST走査順や hash map iteration order は table order に影響しません。

## 11.7 declaration schema

Phase 2 certificate の logical declaration は4種類です。
constructor / recursor は独立した source declaration としては保存せず、
`InductiveDecl` から verifier が kernel environment 用の declaration を生成します。

```text
DeclCert =
  decl: DeclPayload
  dependencies: vec<DependencyEntry>
  axiom_dependencies: vec<AxiomRef>
  hashes: DeclHashes

DeclPayload =
  0x00 AxiomDecl
  0x01 DefDecl
  0x02 TheoremDecl
  0x03 InductiveDecl

AxiomDecl =
  name: NameId
  universe_params: vec<NameId>
  type: TermId

DefDecl =
  name: NameId
  universe_params: vec<NameId>
  type: TermId
  value: TermId
  reducibility: Reducibility

TheoremDecl =
  name: NameId
  universe_params: vec<NameId>
  type: TermId
  proof: TermId
  opacity: Opacity

InductiveDecl =
  name: NameId
  universe_params: vec<NameId>
  params: vec<BinderType>
  indices: vec<BinderType>
  sort: LevelId
  constructors: vec<ConstructorSpec>
  recursor: option<RecursorSpec>

BinderType =
  type: TermId

ConstructorSpec =
  name: NameId
  type: TermId

RecursorSpec =
  name: NameId
  universe_params: vec<NameId>
  type: TermId
  rules: RecursorRules

RecursorRules =
  minor_start: uvar
  major_index: uvar

Reducibility =
  0x00 Reducible
  0x01 Opaque

Opacity =
  0x00 Opaque

DeclHashes =
  decl_interface_hash: hash
  decl_certificate_hash: hash
```

Phase 2 theorem は常に opaque です。
transparent proof を downstream conversion に使いたい場合は `DefDecl` として扱います。

`InductiveDecl.recursor` が `some` の場合、verifier は inductive declaration から
recursor type / rules を再生成し、certificate 内の `RecursorSpec` と一致することを確認します。
不一致は `InductiveGeneratedArtifactMismatch` です。

## 11.8 dependency / axiom schema

dependency は declaration の type / value / proof / constructor type / recursor type に
現れる `Const` 参照から作る重複なしの集合です。

```text
DependencyEntry =
  global_ref: GlobalRef
  decl_interface_hash: hash

AxiomRef =
  global_ref: GlobalRef
  name: NameId
  decl_interface_hash: hash
```

`DependencyEntry` は canonical `GlobalRef` bytes の昇順で保存します。
`AxiomRef` も canonical `GlobalRef` bytes の昇順で保存します。
説明用 JSON では axiom name だけを表示してもよいですが、trusted payload では
`GlobalRef` と `decl_interface_hash` を含めます。

dependency graph に cycle がある場合は `DependencyCycle` です。
Phase 2 では mutual declaration を受理しません。
inductive の constructor / recursor は同じ `InductiveDecl` bundle の内部生成物として扱い、
declaration graph の cycle とはみなしません。

## 11.9 export block schema

`ExportBlock` は downstream module が environment を作るための公開 interface です。
entry は `name` の canonical order で保存します。

```text
ExportBlock =
  vec<ExportEntry>

ExportEntry =
  name: NameId
  kind: ExportKind
  universe_params: vec<NameId>
  type: TermId
  body: option<TermId>
  type_hash: hash
  body_hash: option<hash>
  reducibility: option<Reducibility>
  opacity: option<Opacity>
  decl_interface_hash: hash
  axiom_dependencies: vec<AxiomRef>

ExportKind =
  0x00 Axiom
  0x01 Def
  0x02 Theorem
  0x03 Inductive
  0x04 Constructor
  0x05 Recursor
```

`type` は downstream environment を作るための canonical type term です。
`body` は transparent / reducible def の value term だけに使います。
opaque theorem、axiom、inductive、constructor、recursor では `body = none` です。
`type_hash` は `term_hash(type)` と一致しなければいけません。
`body_hash` は `body = some` のとき `term_hash(body)` と一致し、
`body = none` のとき `none` でなければいけません。

`body_hash` は transparent / reducible def の value hash だけに使います。
opaque theorem の proof hash は `ExportBlock` に入れません。
inductive の `type` は `Pi params indices, Sort sort` の full telescope です。
inductive の constructor / recursor は、`InductiveDecl` から生成された interface として
`ExportBlock` に含めます。
import 側 verifier は、検査済み import certificate の `ExportBlock` 内の `type` と
`body` から kernel environment を再構成します。hash だけを使って environment を
作ってはいけません。
imported `ExportBlock` 内の term が `GlobalRef::Local` を含む場合、その参照は
import元 module の declaration index として解釈します。caller module の local declaration
として解釈してはいけません。caller の certificate 内でその宣言を参照する場合は、
caller 側の term に `GlobalRef::Imported(import_index, name, decl_interface_hash)` を
入れます。

## 11.10 axiom report schema

canonical axiom report は audit log ではなく trusted payload です。

```text
AxiomReport =
  per_declaration: vec<DeclAxiomReport>
  module_axioms: vec<AxiomRef>

DeclAxiomReport =
  decl_index: uvar
  direct_axioms: vec<AxiomRef>
  transitive_axioms: vec<AxiomRef>
```

`per_declaration` は declaration order です。
各 axiom list と `module_axioms` は `AxiomRef` の canonical order です。
`safe_for_high_trust`、`contains_sorry`、`custom_axioms`、`standard_axioms` は
trusted payload に入れません。必要なら decode 後の audit view で再計算します。

## 11.11 hash payload table

hash は raw canonical bytes に domain separator を前置して計算します。
domain separator は ASCII string bytes で、length prefix は付けません。

```text
level_hash(level) =
  sha256("NPA-LEVEL-0.1" || LevelHashPayload(level))

term_hash(term) =
  sha256("NPA-TERM-0.1" || TermHashPayload(term))

decl_interface_hash(decl) =
  sha256("NPA-DECL-IFACE-0.1" || DeclInterfacePayload(decl))

decl_certificate_hash(decl_cert) =
  sha256("NPA-DECL-CERT-0.1" || DeclCertificatePayload(decl_cert))

axiom_report_hash(report) =
  sha256("NPA-AXIOM-REPORT-0.1" || AxiomReportBytes(report))

export_hash(export_block) =
  sha256("NPA-MODULE-EXPORT-0.1" || ExportBlockBytes(export_block))

certificate_hash(cert) =
  sha256("NPA-MODULE-CERT-0.1" || ModuleCertBytesWithoutCertificateHash(cert))
```

`TermHashPayload` は term table index ではなく、構造で計算します。
child term は child `term_hash`、level は `level_hash`、global ref は canonical
`GlobalRef` bytes を入れます。これにより table layout の実装差を hash に混ぜません。

```text
TermHashPayload =
  Sort: 0x00 level_hash
  BVar: 0x01 uvar index
  Const: 0x02 GlobalRefBytes vec<level_hash>
  App: 0x03 term_hash(fun) term_hash(arg)
  Lam: 0x04 term_hash(type) term_hash(body)
  Pi: 0x05 term_hash(type) term_hash(body)
  Let: 0x06 term_hash(type) term_hash(value) term_hash(body)
```

`DeclInterfacePayload` は declaration kind ごとに次を入れます。

```text
Axiom:
  kind, name, universe_params, type_hash, public_dependency_entries

Def:
  kind, name, universe_params, type_hash, reducibility,
  public_dependency_entries, axiom_dependencies
  value_hash only when reducibility = reducible

Theorem:
  kind, name, universe_params, type_hash, opacity,
  public_dependency_entries, axiom_dependencies

Inductive:
  kind, name, universe_params, params, indices, sort,
  constructors, generated recursor signature hash, generated computation rule hash,
  public_dependency_entries, axiom_dependencies
```

`public_dependency_entries` は公開 interface に含まれる term から直接導出します。
axiom と theorem では type、reducible def では type と body、opaque def では type、
inductive では params / indices / constructor type / recursor type が対象です。
proof や opaque body の non-axiom dependency は certificate hash 側にだけ入れます。

`DeclCertificatePayload` は次を入れます。

```text
Axiom:
  decl_interface_hash, axiom_dependencies

Def:
  decl_interface_hash, value_hash, dependency entries, axiom_dependencies

Theorem:
  decl_interface_hash, proof_hash, dependency entries

Inductive:
  decl_interface_hash, dependency entries, axiom_dependencies
```

`generated_recursor_signature_hash` と `generated_computation_rule_hash` は、
inductive declaration payload 内の generated artifact を固定する補助 hash です。
どちらも recursor が存在しない場合も absence marker を hash し、field の有無で
`DeclInterfacePayload` の形を変えません。

```text
generated_recursor_signature_hash =
  sha256("NPA-GEN-REC-SIG-0.1" || GeneratedRecursorSignaturePayload)

GeneratedRecursorSignaturePayload =
  None:
    0x00
  Some:
    0x01 recursor_name recursor_universe_params recursor_type_hash

generated_computation_rule_hash =
  sha256("NPA-GEN-COMP-RULE-0.1" || GeneratedComputationRulePayload)

GeneratedComputationRulePayload =
  None:
    0x00
  Some:
    0x01 minor_start major_index
```

`recursor_type_hash` は `TermHashPayload` から計算した recursor type の term hash です。
`minor_start` / `major_index` は verifier が生成規則から再計算して照合する
`RecursorRulesSpec` と同じ canonical uvar encoding で入れます。

## 11.12 import store / high-trust semantics

import は filesystem path ではなく、検査済み module から解決します。

```text
ImportKey =
  module: Name
  export_hash: hash
  certificate_hash: option<hash>

VerifiedModule =
  module: Name
  name_table: vec<Name>
  level_table: vec<LevelNode>
  term_table: vec<TermNode>
  declarations: vec<DeclCert>
  export_hash: hash
  certificate_hash: hash
  export_block: ExportBlock
  axiom_report: AxiomReport
```

`VerifiedModule` は verifier が検査済み canonical payload から作る値です。
Rust の元 `Decl` ベクタを trusted import state として持ち回ってはいけません。
import 側 kernel environment は、`VerifiedModule` 内の canonical tables / declarations から
decode して作ります。将来的に kernel API が interface fragment を直接受け取れるようになったら、
`ExportBlock` だけからの再構成へ縮めます。

通常モードでは、`ImportEntry.module` と `ImportEntry.export_hash` に一致する
`VerifiedModule` が `VerifierSession` にあればよいです。
`ImportEntry.certificate_hash` はあってもなくてもよいですが、ある場合は一致を確認します。

high-trust mode では、次をすべて要求します。

```text
- ImportEntry.certificate_hash が some
- module / export_hash / certificate_hash が VerifiedModule と一致
- その VerifiedModule は現在の VerifierSession が verify_module_cert で検査済み
- policy で forbidden axiom / synthetic sorry が許可されていない
```

外部から直接作った `VerifiedModule` を high-trust import として注入してはいけません。

## 11.13 structured error enum

Phase 2 の失敗は文字列ではなく、少なくとも次の enum で返します。

```rust
pub enum HashObject {
    Level,
    Term,
    DeclInterface,
    DeclCertificate,
    ExportBlock,
    AxiomReport,
    ModuleCertificate,
}

pub enum CertError {
    DecodeError,
    UnsupportedFormat { format: String, core_spec: String },
    UnsupportedEncoding { tag: u8 },
    NonCanonicalEncoding { object: &'static str },
    HashMismatch { object: HashObject, expected: Hash, actual: Hash },
    ImportHashMismatch { module: ModuleName },
    MissingImportCertificateHash { module: ModuleName },
    ImportCertificateHashMismatch { module: ModuleName },
    ImportNotVerifiedInSession { module: ModuleName },
    DuplicateImportEnvKey { module: ModuleName, export_hash: Hash },
    DuplicateName { name: ModuleName },
    UnknownDependency { name: ModuleName },
    DependencyCycle { name: ModuleName },
    AxiomReportMismatch { decl: Option<ModuleName> },
    ForbiddenAxiom { axiom: ModuleName },
    SorryDenied { axiom: ModuleName },
    UnresolvedMetavariable,
    InvalidBVar { index: u32 },
    InductiveGeneratedArtifactMismatch { name: ModuleName },
    Kernel(npa_kernel::Error),
}
```

CLI や diagnostics はこの enum から人間向けメッセージを作ります。
テストは enum variant と主要 field を直接検査します。

---

# 12. Phase 2 のテストケース

Phase 2 では、完了条件を次のテストで確認します。

## 12.1 golden certificate

少なくとも次の core declaration から `.npcert` を生成し、golden fixture として
byte列または各 hash を固定します。

```text
- id
- const
- Nat
- Eq
- Nat.add
- add_zero
```

期待結果：

```text
- 同じ入力から同じ .npcert bytes が得られる
- source なしで Phase 2 certificate verifier が通る
- export_hash / certificate_hash / axiom_report_hash が再計算結果と一致する
```

## 12.2 canonicalization / hash stability

```text
- binder名だけを変えた同一 term は同じ term_hash になる
- input declaration order が同じ依存関係を表す限り、canonical declaration order は安定する
- import 入力順を入れ替えても canonical import order と module hash は安定する
- name / level / term table の生成順が実装内部の走査順に依存しない
```

## 12.3 hash role の差分テスト

```text
- transparent def の body を変えると decl_interface_hash と export_hash が変わる
- opaque def の body だけを変え、type・reducibility・axiom依存が同じなら export_hash は維持される
- その場合でも decl_certificate_hash と certificate_hash は変わる
- opaque theorem の proof だけを変え、type・opacity・axiom依存が同じなら export_hash は維持される
- その場合でも decl_certificate_hash と certificate_hash は変わる
- opaque theorem の proof 変更で axiom依存が変わると export_hash も変わる
- axiom_report_hash は canonical axiom report の変更でだけ変わる
```

## 12.4 mutation / rejection

不正 certificate を作り、構造化エラーで拒否されることを確認します。

```text
- proof body を1 byte変える
- term_hash / decl hash / export_hash / certificate_hash / axiom_report_hash を改ざんする
- axiom report から実際に使っている axiom を削除する
- unresolved metavariable は trusted schema で表現不能であることを確認する
- unknown term tag を入れる
- 非最短 ULEB128 を使う
- term table に未使用 entry を入れる
- table order / import order / declaration order を崩す
- hash 対象 payload に map 相当の非canonical表現を入れる
```

期待結果：

```text
HashMismatch
AxiomReportMismatch
UnresolvedMetavariable
UnsupportedEncoding
NonCanonicalEncoding
Kernel(npa_kernel::Error)
```

など、原因に応じた enum error が返ること。

## 12.5 import / high-trust mode

```text
- 通常モードでは import の export_hash 一致を必須にする
- 通常モードでは import の certificate_hash 欠落を許す
- high-trust mode では certificate_hash 欠落を拒否する
- high-trust mode では certificate_hash mismatch を拒否する
- high-trust mode では同じ verifier が検査済みでない import certificate を拒否する
```

## 12.6 axiom policy / source independence

```text
- forbidden axiom を policy で拒否できる
- deny_sorry policy で synthetic sorry axiom を拒否できる
- source map / diagnostics / AI trace は trusted schema に存在せず、hash 対象に encode 不能である
- source file を消した状態でも .npcert と import store だけで検査できる
```

## 12.7 producer separation

Human producer と AI producer の分離は、次のテストで確認します。

```text
- Human producer 由来の CoreModule と AI producer 由来の CoreModule が同じ core declaration を表す場合、
  生成される .npcert bytes と各 hash が一致する
- producer_profile / producer_run_id / model name / score / diagnostics を sidecar で変えても、
  .npcert bytes と各 hash が変わらない
- check_core_decl_candidates の Accepted をそのまま VerifiedModule として扱えない
- Accepted candidate から build_module_cert した .npcert は verify_module_cert を通すまで trusted import store に入らない
- invalid prior token は per-candidate rejection ではなく batch-level `Err(CertError)` になる
- `CandidateBatch.imports` が canonical import order でない場合は batch-level `Err(CertError::NonCanonicalEncoding)` になる
- batch 内に同じ `ProducerImportEnvKey(module, export_hash)` が複数ある場合は
  `Err(CertError::DuplicateImportEnvKey)` になる
- `CandidateBatchResult.statuses` は input candidates と同じ長さ・同じ順序で返る
- `build_module_cert_from_checked_candidates` は token chain / pre_env_fingerprint / post_env_fingerprint 不一致を拒否する
- `build_module_cert_from_checked_candidates` は token の `producer_limits_hash(token.limits)` 不一致を拒否する
- producer public env / prior chain fingerprint が canonical bytes と domain separator から deterministic に再計算できる
- `canonical_import_env_keys(imports)` と `canonical_import_export_views(imports)` は同じ順序を保持し、
  `GlobalRef::Imported(import_index, ...)` が同じ import を参照する
- import 先の proof 本体だけが変わり、module name と export_hash が同じ場合、producer public env fingerprint は維持される
- producer public env fingerprint の axiom dependencies は certificate generation と同じ `compute_axiom_deps` から再計算する
- opaque theorem の proof / opaque def の body だけが変わり、公開 interface と axiom dependencies が同じ場合、
  producer public env fingerprint は維持され、prior chain fingerprint は `decl_certificate_hash` の差で変わる
- `ProducerLimits` の canonical hash と stricter_or_equal 判定が deterministic である
- AI producer が渡した preview hash が誤っていても、token validation / build_module_cert / verify_module_cert は再計算結果だけを採用する
- AI producer 由来 candidate に unresolved metavariable / placeholder / pretty-only GlobalRef がある場合は拒否する
- batch 内で1候補が失敗しても、他候補の結果が失敗順序や cache 状態に依存しない
- cache を有効/無効にしても、同じ accepted module から同じ .npcert bytes が得られる
```

---

# 13. Phase 2 の完了条件

Phase 2が完了したと言える条件はこれです。

```text
- core term をcanonical binaryにできる
- canonical binary が core-spec v0.1 の byte-level 条件を満たし、非canonical encoding を拒否できる
- binder名を変えても同じterm hashになる
- importに必須のexport_hashと、高信頼モード必須のcertificate_hashを持たせられる
- import entry は module name / export_hash / optional certificate_hash に限定し、宣言一覧は import の export_block から導出できる
- def/theorem/axiom/inductiveにdeclaration hashを持たせられる
- transparent def のbody変更でinterface hashが変わる
- opaque def のbody変更だけで type・reducibility・axiom依存が変わらない場合、export_hashは維持される
- opaque def のbody変更でdecl_certificate_hashとmodule certificate_hashが変わる
- opaque theorem のproof変更でdecl_certificate_hashとmodule certificate_hashが変わる
- opaque theorem のproof変更だけで type・opacity・axiom依存が変わらない場合、export_hashは維持される
- opaque theorem のproof変更でaxiom依存が変わる場合、export_hashも変わる
- axiom依存をdeclarationごとに計算できる
- module全体のaxiom reportをcanonical orderで出せる
- audit用のsafe_for_high_trust等を保存値として信用せず再計算できる
- checker が .npcert と import store 内の .npcert だけを使って再検査でき、kernel は decode 済み宣言だけを検査できる
- Phase 2 の checker は同一 Rust kernel を使う certificate verifier として定義され、Phase 8 independent checker とは責務が分離されている
- source code、source map、AI traceなしで検査が完結する
- Human producer / AI producer は `CoreModule` または `CoreDeclCandidate` までの非信頼層として分離されている
- AI candidate fast path の成功と `.npcert` verification success が型/API上も運用上も区別されている
- producer metadata / sidecar が trusted payload と hash に入らないことをテストで確認している
- 11章の実装契約に沿った API / byte schema / hash payload / error enum を実装している
- 12章の golden / stability / mutation / high-trust / source-independence / producer separation テストが自動テストで通る
```

## 13.1 現在の実装ステータス

`crates/npa-cert` は、Phase 2 の trusted certificate verifier として次を実装済みです。

```text
- CoreModule から ModuleCert を生成する
- canonical binary encode/decode を行う
- decode 後の再encode一致で canonical bytes を確認する
- name / level / term table の canonical order と reachability を確認する
- import / declaration / export block / axiom report の canonical order を確認する
- level / term / declaration / export / axiom report / module certificate hash を再計算する
- normal / high-trust import policy を検査する
- verified import store だけから kernel environment を再構成する
- axiom report と axiom policy を保存値ではなく再計算結果から検査する
- inductive constructor / recursor export と generated artifact mismatch を検査する
- Phase 1 Rust kernel に decode 済み declaration を渡して再検査する
```

v0.1 で意図的に Phase 2 の trusted payload に入れていないもの:

```text
- source map
- diagnostics
- display name
- elaborator trace
- tactic trace
- AI trace
- unresolved metavariable / hole / placeholder
```

これらは trusted hash の対象ではありません。metadata が必要な場合は、`.npcert` の外側の
debug sidecar として扱います。

Phase 8 の independent checker はこの `.npcert` schema を別実装または別プロセスで
再検査する後続成果物であり、Phase 2 には含めません。

7.1 / 11.1.1 / 12.7 で定義した Human producer / AI producer 分離、`CoreDeclCandidate` /
`CheckedDeclCandidate` fast path、producer sidecar、producer separation テストは、現時点では詳細設計です。
これらは `crates/npa-cert` の既存 trusted verifier 実装済み項目には含めません。
実装完了とみなすには、opaque `CheckedDeclCandidate` token、producer public env fingerprint / prior chain 検査、
`check_core_decl_candidates`、`build_module_cert_from_checked_candidates`、`ProducerLimits` canonical hash /
strictness 判定、producer public env / prior chain fingerprint canonical bytes、および 12.7 の producer separation テストを
追加する必要があります。

---

# 14. 設計の要点

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
  trusted payload から certificate_hash 自身を除いたhash

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
