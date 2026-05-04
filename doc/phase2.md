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
      value_hash,
      reducibility
    )
```

```text
decl_certificate_hash(def)
  = H(
      "NPA-DECL-CERT-0.1",
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
      "NPA-DECL-IFACE-0.1",
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
      "NPA-DECL-CERT-0.1",
      decl_interface_hash,
      proof_hash,
      dependency_hashes
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
      "NPA-DECL-IFACE-0.1",
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
```

`Imported.import_index` は `imports` の index です。
`Imported.name` と `decl_interface_hash` は、その import の `export_block` に存在する
entry と一致しなければいけません。
`Local.decl_index` は現在 module の declaration index です。
`LocalGenerated.decl_index` は現在 module の `InductiveDecl` の declaration index で、
`LocalGenerated.name` はその `InductiveDecl` から生成された constructor または recursor の
`NameId` です。checker は `decl_index` の `InductiveDecl` 内に同じ name の
`ConstructorSpec` / `RecursorSpec` が存在することを確認します。
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
  kind, name, universe_params, type_hash

Def:
  kind, name, universe_params, type_hash, reducibility, axiom_dependencies
  value_hash only when reducibility = reducible

Theorem:
  kind, name, universe_params, type_hash, opacity, axiom_dependencies

Inductive:
  kind, name, universe_params, params, indices, sort,
  constructors, generated recursor signature hash, generated computation rule hash,
  axiom_dependencies
```

`DeclCertificatePayload` は次を入れます。

```text
Axiom:
  decl_interface_hash, axiom_dependencies

Def:
  decl_interface_hash, dependency entries, axiom_dependencies

Theorem:
  decl_interface_hash, proof_hash, dependency entries

Inductive:
  decl_interface_hash, dependency entries, axiom_dependencies
```

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
- opaque theorem のproof変更でdecl_certificate_hashとmodule certificate_hashが変わる
- opaque theorem のproof変更だけで type・opacity・axiom依存が変わらない場合、export_hashは維持される
- opaque theorem のproof変更でaxiom依存が変わる場合、export_hashも変わる
- axiom依存をdeclarationごとに計算できる
- module全体のaxiom reportをcanonical orderで出せる
- audit用のsafe_for_high_trust等を保存値として信用せず再計算できる
- checker が .npcert と import store 内の .npcert だけを使って再検査でき、kernel は decode 済み宣言だけを検査できる
- Phase 2 の checker は同一 Rust kernel を使う certificate verifier として定義され、Phase 8 independent checker とは責務が分離されている
- source code、source map、AI traceなしで検査が完結する
- 11章の実装契約に沿った API / byte schema / hash payload / error enum を実装している
- 12章の golden / stability / mutation / high-trust / source-independence テストが自動テストで通る
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
