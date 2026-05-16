# Phase 5 AI Profile: Machine IDE/API

この文書は、NPA の **AI 向け Phase 5** の設計案です。

従来の `doc/phase5-human.md` は、人間が IDE / Web UI / CLI から goal を読み、tactic を試し、
定理を検索するための API です。一方、AI 証明探索では、多数の候補を生成し、失敗を前提に、
同じ入力から同じ結果を再現できる形で API を呼びます。

そのため AI 向け Phase 5 では、人間向け pretty text や tactic script を中心にせず、構造化された
**Machine Proof Session API** を公開します。

---

# 1. 目的

Phase 5 AI Profile の目的は、Phase 3 AI / Phase 4 AI / Phase 7 AI 探索をつなぐ、
決定的で再実行可能な proof-server API を定義することです。

```text
Machine Surface request
  ↓ Phase 3 AI
fully explicit term / structured diagnostic
  ↓
Machine Tactic candidate
  ↓ Phase 4 AI
transactional proof state transition
  ↓
Machine IDE/API
batch execution / theorem retrieval / replay / verify handoff
  ↓
kernel check + certificate generation
```

優先する性質は次です。

```text
- pretty string を trusted payload にしない
- tactic text を主入力にしない
- AI prompt / completion / score / trace を certificate に入れない
- state_fingerprint は state の canonical bytes から決定的に導出する
- snapshot_id は full state_fingerprint から決定的に導出する
- 同じ state + scheduler_limits を除く同じ deterministic request + 同じ deterministic budget から同じ result / error を返す
- scheduler_limits 由来の timeout / memory stop は deterministic result ではなく retryable artifact とする
- batch 実行は各候補を transactional に扱う
- search result は verified import / decl_interface_hash に紐づける
- verify 成功までは証明済みと呼ばない
```

---

# 2. 信頼境界

AI 向け API は信用しません。

```text
信頼しない:
  AI output
  API client
  request ordering
  tactic ranking
  theorem search ranking
  proof search trace
  repair suggestion
  pretty printer
  cache hit / cache miss

信頼する:
  canonical core AST
  Phase 1 Rust kernel
  Phase 2 certificate verifier
  Phase 8 independent checker
```

Machine IDE/API が `success` を返しても、それは「state transition が非信頼層で成功した」という意味だけです。
証明として採用できるのは、閉じた proof term を kernel check し、certificate checker に通した後だけです。

---

# 3. Human API との差分

人間向け Phase 5 は読みやすさと対話性を優先します。

```json
{
  "state_id": "st_100",
  "goal_id": "g1",
  "tactic": "exact Eq.refl n"
}
```

AI 向け Phase 5 は、曖昧な文字列よりも構造化入力を優先します。

```json
{
  "state_fingerprint": "sha256:...",
  "goal_id": "g1",
  "candidate": {
    "kind": "exact",
    "term": {
      "source": "@Eq.refl.{1} Nat n"
    }
  },
  "deterministic_budget": {
    "max_tactic_steps": 64,
    "max_whnf_steps": 10000,
    "max_conversion_steps": 10000,
    "max_rewrite_steps": 100,
    "max_meta_allocations": 16,
    "max_expr_nodes": 20000
  },
  "scheduler_limits": {
    "timeout_ms": 100
  }
}
```

AI Profile MVP では次を主 API にしません。

```text
- arbitrary tactic script text
- notation に依存する tactic payload
- source cursor position に依存する state selection
- open / namespace に依存する short name
- LSP display option
- natural language instruction
- model score を使った server-side 採用判定
```

使うものは次です。

```text
- MachineProofSession
- MachineProofSnapshot
- MachineGoalView
- MachineTacticCandidate
- MachineTheoremQuery
- MachineTacticBatch
- MachineReplayPlan
- verified import / export_hash / decl_interface_hash
```

---

# 4. 前提条件

AI 向け Phase 5 は次に依存します。

```text
Phase 2:
  verified module import
  export_hash
  certificate_hash
  decl_interface_hash
  axiom report

Phase 3 AI:
  Machine Surface parser / resolver / elaborator
  elaborate_machine_term_check
  canonicalize_machine_term_source
  structured MachineDiagnostic

Phase 4 AI:
  MachineProofState
  MachineTactic
  run_machine_tactic
  MachineProofDelta
  extract_closed_machine_theorem_decl

Phase 6:
  verified standard-library certificates
  optional theorem-search metadata only for non-MVP extensions
```

Phase 5 AI は探索戦略そのものを持ちません。best-first search、premise retrieval の高度化、
proof minimization は Phase 7 の責務です。
Phase 5 AI は、Phase 7 が安全に探索できる API contract を提供します。

---

# 5. Machine Proof Session

## 5.1 セッションの役割

`MachineProofSession` は、AI 探索器が多数の候補を投げるための非信頼セッションです。
セッションは filesystem や network から import を解決しません。呼び出し側が verified import set を明示します。

```rust
struct MachineProofSession {
    session_id: SessionId,
    protocol_version: MachineApiVersion,
    root: CheckedMachineProofRoot,
    imports: Vec<VerifiedImportRef>,
    import_certificate_context: MachineImportCertificateContext,
    machine_surface_callable_interface_table: MachineSurfaceCallableInterfaceTable,
    checked_current_decls: Vec<CheckedCurrentDecl>,
    options: MachineApiOptions,
    initial_snapshot: MachineProofSnapshot,
    snapshots: MachineSnapshotStore,
}
```

`imports` は Phase 4 AI の `VerifiedImportRef` payload そのものを保持します。
ただし request は `VerifiedImportRef` の derived fields を信用して渡すのではなく、検証元になる
canonical certificate bytes と、その certificate を Phase 2 verifier で検査するための transitive closure を渡します。
`module` / hash だけを渡して server が filesystem や network から補完する mode は MVP には入れません。
server-local verified module store を将来追加する場合は、`imports` ではなく別 field
`import_store_refs` として明示し、store hit 後に得られた full `VerifiedImportRef` canonical bytes を
`session_root_hash` と `state_fingerprint` に含めます。

MVP の session create request は `import_closure` と `imports` を分けます。
`import_closure` は direct import とその transitive dependency certificate をすべて含む certificate payload set です。
`imports` は、その closure のうち current proof state の direct imports として Phase 4 に渡す root key list です。
`import_closure` にあるが `imports` にない module は、Phase 2 verifier session を構築するためだけに使い、
Phase 4 `imports`、theorem search premise、AI candidate の direct tactic head にはなりません。
MVP では、`import_closure` wire payload 内に同じ closure key
`(module, expected_export_hash, expected_certificate_hash)` を持つ item が複数ある場合、
certificate bytes が同一でも `InvalidVerifiedImport` として拒否します。
その後に作る canonical closure set は、`imports` root key から各 certificate の `ImportEntry` をたどって到達する
最小 transitive closure と完全一致しなければなりません。
到達不能な extra certificate、または必要な dependency の欠落は `InvalidVerifiedImport` として拒否します。

import payload は canonical object として decode します。
wire JSON の object field order は canonical bytes に影響しません。
unknown field はすべて `InvalidVerifiedImport` として拒否します。
Proof server は `import_closure` の各 `certificate.bytes` を Phase 2 certificate verifier に渡します。
Phase 5 AI MVP は high-trust import mode だけを使うため、closure 内 certificate のすべての `ImportEntry` は
`certificate_hash = some(hash)` を持たなければなりません。
`certificate_hash = none` の `ImportEntry` は module / export_hash が一致していても `InvalidVerifiedImport` です。
各 certificate の `ImportEntry` は同じ `import_closure` 内の verified module key に
`module` / `export_hash` / `certificate_hash` の完全一致で解決できなければなりません。
dependency が欠けている、module / export_hash / certificate_hash が一致しない、または certificate が dependency-topological
order で verifier session に追加できない場合は `InvalidVerifiedImport` です。
request order は検証順ではありません。server は certificate import graph から dependency-topological order を決定し、
cycle、同じ `module` に対する異なる `export_hash` / `certificate_hash`、`import_closure` 内の duplicate closure key、
または `imports` root key が `import_closure` に存在しない場合を `InvalidVerifiedImport` として拒否します。
server は検証済み closure から `exports`, `certified_env_decls`, `axiom_report` payload,
`axiom_report_hash` と certificate-local context table を導出します。
client が `exports` や `certified_env_decls` を直接指定することは MVP では許しません。
certificate bytes から再計算した `certificate_hash` / `export_hash` が request の expected hash と一致しない場合は
`InvalidVerifiedImport` として拒否します。
certificate 内の module name と export block の module origin が request の `module` と一致しない場合も
`InvalidVerifiedImport` として拒否します。

```rust
struct CanonicalCertificateBytesWire {
    encoding: String,
    bytes: String,
}

struct VerifiedModuleCertificateRequest {
    module: ModuleName,
    expected_export_hash: HashString,
    expected_certificate_hash: HashString,
    certificate: CanonicalCertificateBytesWire,
}

struct VerifiedImportRequest {
    module: ModuleName,
    expected_export_hash: HashString,
    expected_certificate_hash: HashString,
}

struct MachineImportCertificateContext {
    verified_modules: Vec<VerifiedModuleContextEntry>,
    direct_import_keys: Vec<VerifiedImportRequest>,
}

struct VerifiedModuleContextEntry {
    module: ModuleName,
    export_hash: HashString,
    certificate_hash: HashString,
    certificate_import_table: Vec<VerifiedImportRequest>,
    decoded_name_table: Vec<Phase5Name>,
    decl_index_table: VerifiedImportDeclIndexTable,
    generated_decl_table: VerifiedImportGeneratedDeclTable,
}
```

`certificate.encoding` は `"npa.certificate.canonical.v0.1.hex"` だけを許します。
`certificate.bytes` は Phase 2 canonical certificate bytes の lowercase hex です。
`certificate` object の unknown field、`bytes` の non-lowercase hex、奇数長、canonical certificate decoder が拒否する
bytes は `InvalidVerifiedImport` です。

session 内部の `VerifiedImportRef` は、`imports` root key が指す verified closure entry の Phase 2 verifier 出力から構築します。
`MachineImportCertificateContext.verified_modules` は direct imports と transitive dependencies を
`(module, export_hash, certificate_hash)` の canonical order で保持します。
各 `VerifiedModuleContextEntry` は、後続の axiom/head normalization が使う certificate-local import table、
decoded name table、declaration index table、generated declaration table を再構築できる context です。
これらの table は process-local pointer ではなく、verified certificate bytes から決定的に再構築した canonical payload /
hash だけを保持します。
owner-aware renderer QA では、同じ verified certificate bytes から各 declaration / generated artifact の
checked declaration view も再取得できなければなりません。
ここでいう checked declaration view は、declaration kind、interface type、body option、reducibility / opacity、
universe parameter context、generated constructor / recursor の reconstructed interface を含む、
Phase 2 verifier が kernel environment を再構成するために使う情報です。
これは新しい trusted input ではなく、`certificate_hash` で固定された certificate bytes と Phase 2 verifier output からの
決定的な派生 payload です。
実装はこの view を `VerifiedModuleContextEntry` 内に cache しても、必要時に certificate bytes から再構築してもよいですが、
name / hash table だけから型や body を推測してはいけません。
`session_root_hash` には table payload 全体ではなく下で定義する table hash を入れますが、session 内部は
`NameId` / `decl_index` 解決に必要な payload を保持しなければなりません。
session 内部の `imports` は、検証後の direct `VerifiedImportRef` を `(module, export_hash, certificate_hash)` の
canonical order に sort して保存します。request order は `session_root_hash` に入りません。
同じ `(module, export_hash, certificate_hash)` の重複は 1 件に dedup します。
同じ `module` で異なる `export_hash` または `certificate_hash` が複数ある場合は、kernel environment の head 解決が
曖昧になるため `InvalidVerifiedImport` として拒否します。
dedup 後の direct verified imports から `DirectPublicExportNameTable` を構築し、public `ExportEntry.name` が複数 direct import に
現れる場合も `InvalidVerifiedImport` として拒否します。
この拒否は `decl_interface_hash` や `export_hash` が同じ場合でも行います。
Phase 4 `TacticHead::Imported` は `name + decl_interface_hash` だけで解決され、`module` や `export_hash` を
持たないため、複数 import が同じ fully-qualified export name を公開する session は MVP では許しません。
単一 import の `ExportBlock` 内に同じ public name が重複する verifier 出力も `InvalidVerifiedImport` です。
`DirectPublicExportNameTable` は request order ではなく dedup 後の canonical direct imports から作ります。
direct import の public `ExportEntry.name` は、後続の session semantic validation で root theorem name と
checked current declaration name との衝突検査にも使います。
`MachineImportCertificateContext.verified_modules` に含まれる direct import または transitive dependency の
`module` は、session `root.module` と一致してはいけません。
同じ module name を import closure 側とこれから生成する current module certificate 側の両方に置くと、
verify 成功後の `dependency_import_closure + [import_payload]` が同名 module を含み、後続 session の closure 検査と
certificate import table が曖昧になります。
この衝突は root/import semantic conflict として `InvalidSessionRequest` です。
Machine Surface source は global name に imported/current の variant tag を持たないため、同じ fully-qualified name が
direct import と current module の両方に存在する session は MVP では許しません。
`import_closure` にだけ含まれる transitive dependency module の public export は、その module 自身では `public_export = true`
でも Phase 4 `TacticHead::Imported` の解決対象ではありません。
Phase 5 view ではこの差を `MachineGlobalRefView.tactic_head_visible` で明示します。

Import certificate context table hash は次で固定します。

```text
DecodedNameTableHash:
  sha256(
    tag "npa.phase5.decoded-name-table.v1"
    certificate module / export_hash / certificate_hash
    names in certificate NameId index order as Phase5Name canonical bytes
  )

VerifiedImportDeclIndexTableHash:
  sha256(
    tag "npa.phase5.import-decl-index-table.v1"
    certificate module / export_hash / certificate_hash
    entries sorted by Phase 2 declaration index:
      decl_index as minimal unsigned LEB128 u64
      fully-qualified name canonical bytes
      decl_interface_hash as HashString digest bytes
      public_export as 0x00 | 0x01
  )

VerifiedImportGeneratedDeclTableHash:
  sha256(
    tag "npa.phase5.import-generated-decl-table.v1"
    certificate module / export_hash / certificate_hash
    entries sorted by (parent_decl_index, generated fully-qualified name canonical bytes):
      parent_decl_index as minimal unsigned LEB128 u64
      parent fully-qualified name canonical bytes
      generated fully-qualified name canonical bytes
      parent_decl_interface_hash as HashString digest bytes
      generated_decl_interface_hash as HashString digest bytes (= parent_decl_interface_hash)
      public_export as 0x00 | 0x01
  )
```

Phase 5 が Phase 4 `start_machine_proof` へ渡す `VerifiedImportRef` と、Phase 4
`state_fingerprint` に入る verified import projection は、上の同じ Phase 2 verifier output だけから作ります。
`state_fingerprint` 用の import projection は次で固定します。

```text
Phase 4 verified import projection used by Phase 5:
  - direct imports in session canonical order by (module, export_hash, certificate_hash)
  - for each direct import:
      module / export_hash / certificate_hash
      exports signature hashes:
        Phase 2 ExportBlock canonical order の各 ExportEntry から
        ExportEntry.name / universe_params / type / decl_interface_hash を使って
        Phase 4 CheckedDeclSignature canonical bytes を再構築して hash した list。
        `universe_params` は 5.3 の ExportSignatureSummary.universe_params と同じ
        single-component string projection を使い、sort / rename / dedup しない。
        その projection ができない ExportEntry は InvalidVerifiedImport。
        ここで `ExportEntry.type` は `CheckedDeclSignature.ty` に対応します。
      certified_env_decls hashes:
        Phase 4 certified_env_decls in verifier dependency-topological order:
          canonical core declaration hash
```

`exports signature hashes` は 5.3 の `export_signature_summary_hash` そのものではありません。
`certified_env_decls hashes` も 5.3 の `certified_env_decl_hashes_summary_hash` そのものではありません。
5.3 の summary hash は `session_root_hash` 用の Phase 5 projection であり、Phase 4 `state_fingerprint` には
Phase 4 が定義する native projection を渡します。
ただし、どちらも同じ accepted certificate bytes と Phase 2 verifier output だけから再構築しなければなりません。
Phase 4 `state_fingerprint` のために request JSON、server-local verified module store、pretty metadata、
または table hash 文字列から別の import projection を作ってはいけません。

`checked_current_decls` は、Phase 4 AI の `CheckedCurrentDecl` payload を復元できる
Phase 5 `CheckedCurrentDeclPackage` として渡します。
hash だけの参照や process-local pointer は使いません。
session create request で current module の prior declaration を渡す場合も、
`check_current_decl_for_machine_tactic` で作られた Phase 4 fields と一致する package を含めます。
Proof server が内部 store を使う場合でも、session の canonical root hash は
`CheckedCurrentDeclPackage canonical bytes` から計算し、store key や挿入順には依存させません。
Phase 4 `state_fingerprint` は package bytes ではなく、復元した Phase 4 `CheckedCurrentDecl`
fields だけから Phase 4 AI の規則で計算します。
`CurrentDeclDependencyReport` は `state_fingerprint` には直接入らず、session root、allow_axioms validation、
後続 verify artifact construction の入力です。
session create では、`checked_current_decls` を source order で 1 件ずつ再検証します。

外部 JSON では `CheckedCurrentDecl` の内部 field を展開せず、次の canonical bytes wrapper だけを許します。
server は `bytes` を decode して `CheckedCurrentDeclPackage` を復元し、そこから Phase 4
`CheckedCurrentDecl` payload と Phase 5 `CurrentDeclDependencyReport` を作って下の validation を実行します。
wrapper の `encoding` や JSON field order は session hash に入りません。

```rust
struct CheckedCurrentDeclWire {
    encoding: String,
    bytes: String,
}
```

```text
CheckedCurrentDeclWire:
  encoding = "npa.phase5.checked-current-decl-package.canonical.v5.hex"
  bytes = lowercase hex of CheckedCurrentDeclPackage canonical bytes

CheckedCurrentDeclPackage canonical bytes:
  - tag "npa.phase5.checked-current-decl-package.v5"
  - source_index as minimal unsigned LEB128 u64
  - CheckedDeclSignature canonical bytes
  - CoreDeclPackage canonical bytes
  - CurrentDeclDependencyReport canonical bytes
  - prior_chain_fingerprint as HashString digest bytes
  - checked_env_fingerprint as HashString digest bytes

CoreDeclPackage canonical bytes:
  - tag "npa.phase5.current-core-decl-package.v1"
  - name_table:
      tag "npa.phase5.current-core-decl-package.name-table.v1"
      Phase 2 NameTable canonical payload bytes
  - level_table:
      tag "npa.phase5.current-core-decl-package.level-table.v1"
      Phase 2 LevelTable canonical payload bytes
  - term_table:
      tag "npa.phase5.current-core-decl-package.term-table.v1"
      Phase 2 TermTable canonical payload bytes
  - root_decl:
      tag "npa.phase5.current-core-decl-package.root-decl.v1"
      exactly one Phase 2 DeclPayload encoded against the three tables above

CurrentDeclDependencyReport canonical bytes:
  - tag "npa.phase5.current-decl-dependency-report.v4"
  - direct_dependency_entries in CurrentDeclDependencyEntry canonical order
  - axiom_dependencies as MachineAxiomRefWire canonical bytes in canonical order

CurrentDeclDependencyEntry canonical bytes:
  - tag "npa.phase5.current-decl-dependency-entry.v1"
  - dependency_ref as MachineDependencyRefWire canonical bytes
  - decl_interface_hash as HashString digest bytes

MachineDependencyRefWire canonical bytes:
  - tag "npa.phase5.dependency-ref-wire.v2"
  - variant tag:
      0x00 imported
      0x01 current_module
      0x02 current_generated
  - imported:
      module canonical bytes
      fully-qualified name canonical bytes
      export_hash as HashString digest bytes
  - current_module:
      module canonical bytes
      fully-qualified name canonical bytes
      source_index as minimal unsigned LEB128 u64
  - current_generated:
      module canonical bytes
      generated fully-qualified name canonical bytes
      parent_source_index as minimal unsigned LEB128 u64
```

`prior_chain_fingerprint` と `checked_env_fingerprint` は Phase 4 AI の `CheckedCurrentDecl` が定義する
同名 fingerprint と完全に同じ canonical bytes から計算します。
Phase 5 はこの 2 つを再定義せず、`CheckedCurrentDeclPackage` にはその 32-byte digest だけを入れます。
したがって Phase 4 の fingerprint schema version を変える場合は、Phase 5 の
`CheckedCurrentDeclPackage` encoding version も上げなければなりません。

`CoreDeclPackage` は current declaration 1 件を structural に standalone decode できる table-inclusive package です。
`DeclPayload` だけ、または process-local / import certificate-local table を参照する bytes は wire payload として許しません。
`CoreDeclPackage` は Phase 2 module certificate ではありません。
module header、import table、`DeclHashes`、`DependencyEntry`、`AxiomRef`、`ExportBlock`、`AxiomReport`、
certificate sidecar、または `certificate_hash` placeholder を含めてはいけません。
Phase 2 table payload bytes は、Phase 2 canonical table payload そのものを上の Phase 5 wrapper tag の内側に
1 回だけ入れます。Phase 2 module / certificate outer field tag を再利用したり、Phase 5 wrapper tag を省略したりしてはいけません。
`root_decl` の `NameId` / `LevelId` / `TermId` は同じ `CoreDeclPackage` 内の table だけを参照します。
name table / level table / term DAG は Phase 2 canonical table order と同じ規則を使い、root declaration から到達可能な
entry だけを含む最小 package でなければなりません。
unused table entry、duplicate node、未参照 id、table 範囲外 id、Phase 2 canonical order 違反は
canonical decoder failure として `InvalidCheckedCurrentDecl` です。
server は `CoreDeclPackage` から structural core `Decl` を復元し、それを Phase 4 `CheckedCurrentDecl.core_decl` として扱います。
`CoreDeclPackage` 内の Phase 2 `GlobalRef` は package-local table ではなく、次の session semantic context でだけ解釈します。
この scope を変える option はありません。

```text
CoreDeclPackage GlobalRef scope in current module context:
  Imported(import_index, name, decl_interface_hash):
    import_index indexes the deduped direct session imports sorted by
    (module, export_hash, certificate_hash) canonical order.
    The referenced name + decl_interface_hash must be a public ExportEntry of that direct import.
    A transitive-closure-only module is not addressable from current core_decl.

  Local(decl_index):
    decl_index is a Phase 5 session current-module source_index coordinate, not the final
    Phase 2 certificate declaration index.
    MVP requires decl_index = source_index for each checked current source declaration.
    Ordinary local references must satisfy decl_index < package.source_index.
    Verify certificate construction rewrites this source_index coordinate to the
    certificate-local declaration index after Phase 2 declaration ordering is known.

  LocalGenerated(decl_index, generated_name):
    decl_index is the parent source declaration index in the same Phase 5 session coordinate system.
    Ordinary generated references must satisfy decl_index < package.source_index and resolve
    in CurrentGeneratedDeclTable.
    Verify certificate construction rewrites the parent source_index to the parent
    certificate-local declaration index.
```

`InductiveDecl` package の constructor type / recursor type / generated computation rule でだけ、
`Local(package.source_index)` と `LocalGenerated(package.source_index, generated_name)` を同一 inductive bundle 内参照として許します。
この bundle-internal ref は kernel environment generation の整合性検査に使いますが、
`CurrentDeclDependencyEntry` には出力せず、dependency cycle ともみなしません。
verify artifact construction で Phase 2 `DeclCertificatePayload.dependency entries` を作る場合も、同じ
bundle-internal ref は dependency entry から除外します。
つまり Phase 2 の「constructor / recursor は同じ `InductiveDecl` bundle の内部生成物」という例外を
Phase 5 current declaration package にも適用し、core term 内の ref は残すが dependency set には入れません。
Phase 5 report と Phase 2 certificate dependency payload は、この除外後の同じ dependency set から作ります。
同じ例外を `AxiomDecl` / `DefDecl` / `TheoremDecl` には適用しません。
また、inductive bundle 内参照でない self / future local ref は常に `InvalidCheckedCurrentDecl` です。
`CurrentDeclDependencyReport` は Phase 2 raw `NameId` や certificate-local table を含めません。
`MachineDependencyRefWire` と `MachineAxiomRefWire` は、5.1 の `Phase5Name canonical bytes`、
`HashString`、`source_index` / `parent_source_index` integer、固定 variant tag だけで encode される
table-free payload です。
したがって report bytes は `CoreDeclPackage` の name table、import certificate の name table、または
Phase 2 `NameId` numbering に依存しません。
`CurrentDeclDependencyReport` は `core_decl` と session context から Phase 2 の dependency / axiom derivation rule に
対応する Phase 5 wire report として再計算できる deterministic report です。
request payload の report と再計算 report が byte-for-byte で一致しない場合は `InvalidCheckedCurrentDecl` です。
`allow_axioms` 判定で使う checked current decl の axiom dependencies は、この再計算済み
`CurrentDeclDependencyReport.axiom_dependencies` だけです。
decoder は report bytes の structural canonicality だけを先に確認します。
`MachineDependencyRefWire` / `MachineAxiomRefWire` が session context に存在する import/current declaration へ
解決できるかは checked_current_decls validation step 6 で検査します。
解決できない ref、または session context に対して再導出できない dependency report は `InvalidCheckedCurrentDecl` です。

current decl の direct dependency report は、`core_decl` の type / value / proof / constructor type / recursor type /
generated computation rule に現れる direct `Const` を Phase 2 と同じ対象範囲で走査し、
同一 `InductiveDecl` bundle 内参照を除外したうえで、
各 `GlobalRef` を 5.1 の `DependencyEntry to MachineDependencyRefWire` 規則で table-free ref に変換して作ります。
current decl の transitive axiom dependencies は次の deterministic closure で作ります。

```text
CurrentDecl axiom dependency derivation:
  1. direct_dependency_entries を core_decl から導出し、同一 InductiveDecl bundle 内参照を除外し、
     CurrentDeclDependencyEntry canonical bytes で sort/dedup する
  2. current core_decl が AxiomDecl の場合:
       self を MachineAxiomRefWire::CurrentModule {
         module = root.module,
         name = checked signature.name,
         source_index = package.source_index,
         decl_interface_hash = checked signature.decl_interface_hash
       } として axiom_dependencies に入れる
       この場合、direct dependency entry は report に残すが、それらの axiom_dependencies は union しない
       これは Phase 2 の axiom_dependencies(axiom) = {self} と同じ規則である
  3. current core_decl が AxiomDecl ではない場合、各 direct dependency entry について:
       imported ref:
         direct import ExportEntry の axiom_dependencies を 5.1 の AxiomRef to MachineAxiomRefWire で変換して追加する
       current_module ref:
         source_index < current package.source_index の prior CurrentDeclDependencyReport.axiom_dependencies を追加する
       current_generated ref:
         parent_source_index < current package.source_index の prior CurrentDeclDependencyReport.axiom_dependencies を追加する
         generated artifact 自体は独立した axiom report を持たず、axiom closure は parent declaration と同じものとして扱う
  4. MachineAxiomRefWire canonical bytes で sort/dedup する
```

current decl が future declaration、同一 `InductiveDecl` bundle 例外ではない自分自身、または source_index prefix 外の
current dependency を参照する場合は `InvalidCheckedCurrentDecl` です。
imported dependency の `module/name/export_hash/decl_interface_hash` が direct import の public `ExportEntry` に
存在しない場合も `InvalidCheckedCurrentDecl` です。

`checked_current_decls` item は `encoding` と `bytes` だけを持つ object でなければなりません。
item non-object、duplicate key、unknown field、required field omitted、`null`、未知 `encoding`、
`bytes` の non-string、non-lowercase hex、奇数長、canonical decoder failure は
`InvalidCheckedCurrentDecl` です。
decode 後の canonical bytes を再 encode した結果が input `bytes` と byte-for-byte で一致しない payload も
`InvalidCheckedCurrentDecl` です。

```text
checked_current_decls validation:
  1. imports から構築した verified kernel environment を初期環境にする
  2. source_index が重複する payload は InvalidCheckedCurrentDecl として拒否する
  3. checked_current_decls を source_index 昇順で処理する
  4. root.source_index = k とし、source_index が 0, 1, ..., k-1 の完全 prefix であることを確認する
  5. source_index >= root.source_index、gap、任意 subset、未来 declaration は InvalidCheckedCurrentDecl として拒否する
  6. 各 package canonical bytes を再 encode し、CurrentDeclDependencyReport の refs が
     session imports / prior current declarations / self axiom ref の許可範囲で解決できることを確認する
  7. prior_chain_fingerprint が直前までの checked decl chain と一致することを確認する
  8. checked_env_fingerprint が imports + prior checked decls の kernel environment fingerprint と一致することを確認する
  9. core_decl を kernel check し、CheckedDeclSignature と CurrentDeclDependencyReport を再計算する。
     signature と decl_interface_hash が Phase 2 rule と一致し、再計算 report が package report と一致することを確認する
  10. 失敗した場合は InvalidCheckedCurrentDecl として拒否する
```

`session_id` は便利な handle です。信頼できる識別子ではありません。
state の同一性は `state_fingerprint` だけで判断します。
`snapshot_id` は `state_fingerprint` から導出される API handle であり、fingerprint そのものではありません。
`session_id` は server-local fresh handle であり、決定性 contract の対象ではありません。
同じ session create request を 2 回投げた場合、`session_root_hash` と `initial_snapshot.state_fingerprint` は同じでなければ
なりませんが、`session_id` は異なってよいです。
server は `session_root_hash` から `session_id` を導出してはいけません。

```text
SessionId wire:
  regex = ^msess_[A-Za-z0-9._-]{1,64}$
  canonical bytes = Phase5 UTF-8 string primitive bytes
```

`session_id` は JSON string 必須です。
空 suffix、control character、whitespace、`/`、`\`、percent escape 後に上の regex へ一致しない path segment、
64 文字を超える suffix、JSON number / object / array は invalid `SessionId` grammar です。
request body 内の invalid `SessionId` grammar は endpoint ごとの request validation error に写します。
`DELETE /machine/sessions/{id}` の path id でも同じ grammar を使い、違反は `InvalidSessionRequest` です。

`root.theorem_type` は request の raw Machine Surface text のまま hash しません。
session create 時に Phase 3 AI Complete mode で canonicalize / elaborate し、kernel が type として受理することを
確認した `CheckedMachineProofRoot` を作ります。
root theorem type は tactic candidate ではないため、Phase 4 `MachineTermSource` wrapper hash を使いません。
`theorem_type_source` は Phase 3 `MachineTermSourceCanonical` から作る root 専用 payload で、
`phase3_canonical_hash = hash(Phase 3 Machine Surface term-source canonical bytes)` を保持します。
`MachineRootTermSource.source` は debug / replay-adjacent display 用に保持してよい raw source text ですが、
`session_root_hash`、Phase 4 `state_fingerprint`、certificate payload、candidate hash には入りません。
同じ Phase 3 canonical term-source bytes を持つ root theorem type は、raw whitespace や parentheses だけが違っても
同じ `phase3_canonical_hash` を使います。
`root.theorem_name` は session 内では予約済みの出力名であり、root theorem type、tactic candidate の raw term、
theorem search、prompt payload の premise scope から参照可能な declaration ではありません。
verify 成功後に生成された certificate を後続 session の direct import にした場合だけ、その root theorem は
imported public declaration として参照可能になります。
証明中に `root.theorem_name` を global name として解決しようとする Machine Surface term は `UnknownName` です。

```rust
struct CheckedMachineProofRoot {
    module: ModuleName,
    theorem_name: Name,
    source_index: u64,
    universe_params: Vec<String>,
    theorem_type_source: MachineRootTermSource,
    theorem_type_core_hash: Hash,
}

struct MachineRootTermSource {
    source: String,
    phase3_canonical_hash: Hash,
}
```

`CheckedMachineProofRoot canonical bytes` は次です。

```text
CheckedMachineProofRoot canonical bytes:
  - tag "npa.phase5.checked-machine-proof-root.v1"
  - module as Phase5Name canonical bytes
  - theorem_name as Phase5Name canonical bytes
  - source_index as minimal unsigned LEB128 u64
  - universe_params in request order:
      each MachineUniverseParamName as Phase5 UTF-8 string primitive bytes
  - theorem_type_source.phase3_canonical_hash as HashString digest bytes
  - theorem_type_core_hash as HashString digest bytes
```

`universe_params` は request order が declaration order です。sort / rename / dedup せず、
`Phase5Name canonical bytes` でも dotted name でも encode しません。

`MachineApiOptions` は profile 名だけではなく、Phase 4 の tactic environment を作る入力を明示します。
unknown field と省略 field は `InvalidMachineApiOptions` として拒否します。

```rust
struct MachineApiOptions {
    kernel_check_profile: KernelCheckProfileId,
    allow_axioms: Vec<MachineAxiomRefWire>,
    tactic_options: MachineTacticOptionsRequest,
}

enum MachineAxiomRefWire {
    Imported {
        module: ModuleName,
        name: FullyQualifiedName,
        export_hash: HashString,
        decl_interface_hash: HashString,
    },
    CurrentModule {
        module: ModuleName,
        name: FullyQualifiedName,
        source_index: u64,
        decl_interface_hash: HashString,
    },
}

struct MachineTacticOptionsRequest {
    simp_rules: Vec<SimpRuleRef>,
    eq_family: Option<EqFamilyRef>,
    nat_family: Option<NatFamilyRef>,
    max_simp_rewrite_steps: u64,
    max_open_goals: u64,
    max_metas: u64,
}
```

`MachineTacticOptionsRequest` の wire schema は Phase 4 AI の `MachineTacticOptions` / `SimpRuleRef` /
`EqFamilyRef` / `NatFamilyRef` と同じ field names に固定します。
`tactic_options` object 自体、`simp_rules`、`eq_family`、`nat_family`、
`max_simp_rewrite_steps`、`max_open_goals`、`max_metas` はすべて必須 field です。
`eq_family` と `nat_family` は `null` または object だけを許します。

```text
SimpRuleRef wire:
  {
    "name": FullyQualifiedName,
    "decl_interface_hash": HashString,
    "direction": "forward" | "backward"
  }

EqFamilyRef wire:
  {
    "eq_name": FullyQualifiedName,
    "eq_interface_hash": HashString,
    "refl_name": FullyQualifiedName,
    "refl_interface_hash": HashString,
    "rec_name": FullyQualifiedName,
    "rec_interface_hash": HashString
  }

NatFamilyRef wire:
  {
    "nat_name": FullyQualifiedName,
    "nat_interface_hash": HashString,
    "zero_name": FullyQualifiedName,
    "zero_interface_hash": HashString,
    "succ_name": FullyQualifiedName,
    "succ_interface_hash": HashString,
    "rec_name": FullyQualifiedName,
    "rec_interface_hash": HashString
  }
```

各 nested object の unknown field、省略 field、`null`、`HashString` でない hash、fully-qualified canonical name でない
name、`MachineSurfaceRenderableName` でない `SimpRuleRef` / `EqFamilyRef` / `NatFamilyRef` の name、
未知 `direction` は `InvalidMachineApiOptions` です。
`simp_rules` は array 必須で、各 item を Phase 4 `SimpRuleKey` canonical order に sort/dedup します。
`eq_family` / `nat_family` が object の場合は上の field order で Phase 4 canonical bytes に encode し、sort しません。
`max_simp_rewrite_steps`、`max_open_goals`、`max_metas` は unsigned integer 必須で、0、負数、float、`null`、
型幅を超える整数は `InvalidMachineApiOptions` です。
Phase 4 coherent family validation、simp registry validation、unknown/ambiguous rule handling は Phase 4 AI の
`start_machine_proof` と同じ規則で実行し、失敗は `InvalidMachineApiOptions` に写します。

`allow_axioms` と `simp_rules` は canonical order に sort し、完全一致重複だけ dedup します。
`allow_axioms` は session 内で許可する axiom dependency の閉集合です。
verified import の `axiom_report` payload に含まれる axiom dependencies、checked current decl の axiom dependencies、
verify 対象 theorem の axiom dependencies は、すべて `allow_axioms` の subset でなければなりません。
verify 対象 theorem については、root theorem type から導出できる type-level axiom dependencies を session create 中に検査し、
final proof body / generated certificate からしか確定できない axiom dependencies を `/machine/verify` 中に検査します。
外部 JSON の `allow_axioms` は Phase 2 `AxiomRef` をそのまま出さず、上の `MachineAxiomRefWire` を使います。
`allow_axioms` item は semantic validation で実在する axiom declaration だけを指さなければなりません。
`Imported` item は `MachineImportCertificateContext.verified_modules` 内の matching `module` / `export_hash`
を持つ verified module で、`name` / `decl_interface_hash` が一致する declaration kind `AxiomDecl` に
解決できる場合だけ有効です。public `ExportEntry` であることは要求しません。
`CurrentModule` item は `CurrentDeclIndexTable[source_index]` の checked current declaration が
`AxiomDecl` で、`module` / `name` / `decl_interface_hash` がその table entry と checked signature に一致する場合だけ有効です。
MVP ではこの `module` は必ず `root.module` です。`module` が一致しない item は subset mismatch ではなく
`InvalidMachineApiOptions` です。
def / theorem / inductive / constructor / recursor / generated artifact を指す `allow_axioms` item は
`InvalidMachineApiOptions` です。
同じ axiom kind check は request `allow_axioms` だけでなく、verified import の `axiom_report`、
checked current declaration の再計算済み axiom dependencies、verify で生成した certificate の axiom report を
`MachineAxiomRefWire` へ変換する前にも必ず適用します。
axiom dependency が `AxiomDecl` 以外、または constructor / recursor などの generated artifact を指す場合は
malformed report です。verified import 由来なら `InvalidVerifiedImport`、checked current declaration 由来なら
`InvalidCheckedCurrentDecl`、verify で生成した certificate 由来なら `VerifyFailed` として拒否します。
`allow_axioms` は array 必須で、item は object でなければなりません。
wire encoding では各 item に `kind = "imported" | "current_module"` を必ず入れます。
item object の duplicate key、unknown field、required field omitted、`null` は `InvalidMachineApiOptions` です。
`kind` が non-string、未知 string、または variant と field set が一致しない場合も `InvalidMachineApiOptions` です。

```text
MachineAxiomRefWire item schema:
  imported:
    required fields = kind, module, name, export_hash, decl_interface_hash
    forbidden fields = source_index
  current_module:
    required fields = kind, module, name, source_index, decl_interface_hash
    forbidden fields = export_hash
```

`module` は JSON string で `ModuleName` grammar を満たさなければなりません。
`name` は JSON string で `FullyQualifiedName` grammar を満たさなければなりません。
`export_hash` / `decl_interface_hash` は `HashString` でなければなりません。
`source_index` は unsigned u64 integer 必須で、`null`、負数、float、型幅を超える整数は
`InvalidMachineApiOptions` です。

`kernel_check_profile` の MVP allowed values は次だけです。

```text
KernelCheckProfileId:
  - "npa.kernel.v0.1.builtin-none"
  - "npa.kernel.v0.1.builtin-eq-nat"

kernel_check_profile canonical bytes:
  - tag "npa.phase4.kernel-check-profile.v1"
  - core spec id: "core-spec-v0.1"
  - kernel semantics profile id: "npa-kernel.phase1.v0.1"
  - reduction profile id: "beta-delta-iota-zeta.v0.1"
  - universe profile id: "levels-imax-v0.1"
  - builtin profile id:
      "builtin-none-v0.1" for "npa.kernel.v0.1.builtin-none"
      "builtin-eq-nat-v0.1" for "npa.kernel.v0.1.builtin-eq-nat"
```

`kernel_check_profile` omitted、`null`、上の allowed values 以外の string は `InvalidMachineApiOptions` です。
server は request id から上の bytes を再構築し、実行中 kernel が公開する Phase 4
`kernel_check_profile_hash` と一致しない場合も `InvalidMachineApiOptions` として拒否します。
`session_root_hash` と Phase 4 `state_fingerprint` には request string ではなく、この canonical bytes の hash を入れます。

`MachineAxiomRefWire canonical bytes` は次です。

```text
MachineAxiomRefWire canonical bytes:
  - tag "npa.phase5.axiom-ref-wire.v1"
  - variant tag:
      0x00 imported
      0x01 current_module
  - imported:
      module canonical bytes
      fully-qualified name canonical bytes
      export_hash as HashString digest bytes
      decl_interface_hash as HashString digest bytes
  - current_module:
      module canonical bytes
      fully-qualified name canonical bytes
      source_index as minimal unsigned LEB128 u64
      decl_interface_hash as HashString digest bytes
```

`HashString` は `sha256:` prefix を除いた 32-byte digest だけを encode します。
Phase 5 で `SimpRuleRef canonical bytes` / `SimpRuleRef canonical order` と書く場合は、独自の wire order ではなく
Phase 4 `SimpRuleKey canonical bytes` / `SimpRuleKey canonical order` をそのまま使います。
入力は `SimpRuleRef.name`、`SimpRuleRef.decl_interface_hash`、`SimpRuleRef.direction` の 3 field だけです。
JSON field order、request order、duplicate count、または resolved rule table の insertion order は使いません。
Phase 5 wire の `ModuleName` / `FullyQualifiedName` は JSON では dotted UTF-8 string ですが、hash 入力では
Phase 2 certificate 内の `NameId` index を使いません。
`NameId` は certificate-local name table の index なので、Phase 5 wire / cache / replay 用の独立した名前 bytes
には使えません。

```text
Phase5Name canonical bytes:
  - component_count as minimal unsigned LEB128 u32
  - for each component in order:
      UTF-8 byte length as minimal unsigned LEB128 u32
      UTF-8 bytes exactly as decoded from JSON
```

Phase 5 は `ModuleName` / `FullyQualifiedName` の JSON string を `.` で分割して component list にします。
空文字列、先頭 / 末尾の `.`, 連続する `.`, JSON escape 展開後に `.` を含む component は拒否します。
Unicode normalization は行わず、JSON decode 後の UTF-8 byte sequence が完全一致する場合だけ同じ名前とみなします。
比較と sort は `Phase5Name canonical bytes` の辞書順です。
Phase 2 `NameId` から Phase 5 wire 名を作る場合は、必ず certificate の `name_table` で `Name` component list に
解決してから `Phase5Name canonical bytes` に変換します。

この文書で `FullyQualifiedName` と書く場合、それは dotted component list として一意な Phase 5 wire name を
意味し、必ず module name を prefix に持つという意味ではありません。
Phase 2 `ExportEntry.name` が `Eq` や `Nat.add_zero` なら、それがそのまま exported declaration の
`FullyQualifiedName` です。
`module` field と `ExportEntry.name` を連結して synthetic fully-qualified name を作ってはいけません。
一方、`root.theorem_name` と current module の checked declaration name は current module 内に新しく作る
declaration name なので、5.2 の規則どおり `root.module` を prefix に持たなければなりません。
以後、この区別が重要な箇所では `exported declaration name` と `current module declaration name` と明記します。

Machine Surface source に出力する名前は、より狭い `MachineSurfaceRenderableName` でなければなりません。
MVP では escaping / quoting を持たないため、renderable でない Phase5Name を Machine Surface source に出してはいけません。

```text
MachineSurfaceNameComponent:
  regex ^[A-Za-z_][A-Za-z0-9_']{0,63}$
  and not "_"

MachineSurfaceTermHeadComponent:
  MachineSurfaceNameComponent
  and not one of the MVP term-head reserved components:
    "import", "def", "theorem",
    "forall", "fun", "let", "in",
    "Prop", "Type", "Sort",
    "succ", "max", "imax",
    "open", "namespace", "match", "with"

MachineSurfaceRenderableName:
  first component is MachineSurfaceTermHeadComponent
  followed by zero or more "." + MachineSurfaceNameComponent

MachineUniverseParamName:
  MachineSurfaceNameComponent
  and not one of the MVP universe-param reserved components:
    "import", "def", "theorem",
    "forall", "fun", "let", "in",
    "Prop", "Type", "Sort",
    "succ", "max", "imax",
    "open", "namespace", "match", "with"
```

Phase 3 が keyword / reserved token を増やす場合は、Phase 5 protocol version も上げて、
`MachineSurfaceTermHeadComponent` または `MachineUniverseParamName` の reserved set を同時に更新します。
reserved spelling は dotted name の 2 component 目以降では使えます。たとえば `Nat.succ` は
`MachineSurfaceRenderableName` ですが、単独の `Prop` は `MachineSurfaceRenderableName` ではありません。
`MachineLocalName` は dotted name ではなく、1 つの `MachineSurfaceTermHeadComponent` だけからなる JSON string です。
`intro.name`、`induction-nat.local_name`、`TacticHead.Local.name`、`CandidateApplyArg.Subgoal.name_hint` に
string を入れる場合は `MachineLocalName` を満たさなければなりません。
`name_hint = null` は許しますが、string が `MachineLocalName` でない場合は candidate schema violation として拒否します。
`MachineExprRenderer.machine`、`MachineTacticCandidate` 内の `TacticHead` / raw term source、
`global_ref.name` から生成する suggested candidate、`SimpRuleRef`、`EqFamilyRef`、`NatFamilyRef` に入る
wire name はすべて `MachineSurfaceRenderableName` を満たす必要があります。
MVP の session create は、direct import の public `ExportEntry.name` が renderable でない場合は
`InvalidVerifiedImport`、public simp / family rule source は `InvalidMachineApiOptions`、root `theorem_name` は
`InvalidSessionRequest`、checked current declaration の public name と current generated constructor / recursor name は
`InvalidCheckedCurrentDecl` として拒否します。
さらに MVP session create は、direct import から theorem index entry になり得る public `TheoremDecl` / `AxiomDecl`
export について、`ExportEntry.name`、`ExportEntry.universe_params`、および `statement.machine` rendering に必要な
display render scope name が renderable であることを preflight します。
この preflight に失敗する import は、search endpoint が必ず `InvalidTheoremIndex` になる session を作らないため
`InvalidVerifiedImport` として拒否します。
theorem index entry にならない private / transitive dependency name は certificate context 内に保持できます。
ただし後続 endpoint がそれを snapshot / prompt の Machine Surface source として実際に表示する必要がある場合は
`InvalidMachineProofState`、theorem index construction 中に新たに必要になった場合は `InvalidTheoremIndex` です。
renderer は non-renderable name を silently drop、rename、または pretty-only alias に置換してはいけません。

この文書の Phase 5 canonical bytes block は、別途明記しない限り次の primitive encoding を使います。
Prompt payload、session root、theorem index、query、filters、batch policy は同じ primitive を共有します。
Diagnostic は 11 の `MachineApiDiagnostic canonical bytes` で明示する専用 encoding を使います。

```text
Phase5 canonical primitive encoding:
  - domain tag / protocol_version / profile id / output_schema / enum wire name / UTF-8 string:
      byte length as minimal unsigned LEB128 u64
      UTF-8 bytes exactly as decoded from JSON or defined by the protocol
  - bool:
      0x00 false
      0x01 true
  - unsigned integer:
      minimal unsigned LEB128 with the field's declared width; if no width is declared, u64
  - list / vec:
      length as minimal unsigned LEB128 u64
      elements in the documented order
  - option:
      0x00 none
      0x01 some followed by payload
  - enum variant:
      documented numeric tag if one is specified; otherwise enum wire name as UTF-8 string
  - HashString / Hash:
      exactly 32 digest bytes without "sha256:"
  - ModuleName / FullyQualifiedName:
      Phase5Name canonical bytes
```

Phase 5 wire JSON の unsigned integer は、JSON number token のうち `0|[1-9][0-9]*` だけを許します。
`+` / `-` sign、leading zero、fraction、exponent、string number、または implementation-defined な float-to-int coercion は
すべて invalid integer です。
値は各 field の declared width に収まらなければならず、0 を許すかどうかは field ごとの規則で決めます。
実装は `1e3` や `1.0` を JSON parser の数値変換後に `1000` / `1` として受理してはいけません。

JSON object field order、whitespace、escape の書き方、request insertion order、Rust enum discriminant、
HashMap / HashSet iteration order は Phase 5 canonical bytes に使ってはいけません。
すべての Phase 5 JSON object は duplicate key を許しません。
同じ object 内に同じ decoded UTF-8 key が複数回現れた request は、endpoint ごとの request validation error
として拒否します。
実装は duplicate key を検出できる JSON decoder を使い、first-wins / last-wins / merge の挙動に依存してはいけません。
server が返す response object も duplicate key を含めてはいけません。
ただし `/machine/tactics/run` の embedded candidate、`/machine/tactics/batch` の各 item の inner `candidate` payload、
`/machine/replay` の plan / step / embedded candidate / budget のように、
この文書で明示的に delayed validation とした embedded payload は、その validation stage まで object として decode
しません。
この delayed payload は raw JSON slice、または duplicate key を保持できる lossless JSON representation として保持します。
Phase 5 server は delayed payload を `serde_json::Value` のような first-wins / last-wins map に先に変換してはいけません。
ただし delayed payload も request body の一部なので、HTTP body 全体は syntactically valid JSON でなければなりません。
delayed validation で遅らせるのは、valid JSON value としては切り出せるが object schema / candidate schema としては
まだ decode しない payload です。
JSON 構文エラーは delayed validation の対象ではなく、その endpoint の request parse / request validation error です。
duplicate key prohibition は、各 validation stage で実際に decode する object に対して適用します。
したがって batch の inner candidate payload に duplicate key が含まれていても、それだけで batch request 全体を
拒否してはいけません。prefix 内で評価対象になった candidate は、candidate object として decode された時点で
`InvalidCandidate` result として拒否します。prefix 外 candidate は評価されない限り candidate object として decode せず、
response にも診断にも含めません。

MVP 実装は、top-level / envelope object の duplicate key を検出しつつ、delayed validation payload については
raw JSON slice または duplicate-key-aware syntax tree を保持できる decoder を用意しなければなりません。
`serde_json::Value` のように object を map へ正規化して duplicate key や original JSON value boundary を失う表現だけで
request body を受ける実装は Phase 5 AI MVP v1 と互換ではありません。
この lossless request decoding layer は `/machine/tactics/batch` の inner `candidate`、`/machine/replay` の
embedded `candidate` / `deterministic_budget`、および将来 delayed validation と明記する payload の前提条件です。
`allow_axioms` は `MachineAxiomRefWire canonical bytes` の辞書順に sort/dedup します。
Phase 2 verifier 由来の dependency `AxiomRef` は、session の canonical import/current-decl table を使って
`MachineAxiomRefWire` に変換してから比較します。
変換は Phase 2 `GlobalRef` が出現した certificate / current-module context に依存します。

Phase 5 は `decl_index` を `ExportBlock` や `checked_current_decls` の配列 index として扱いません。
session create / verify は次の lookup table を検証済み payload から構築します。

```text
VerifiedImportDeclIndexTable for import C:
  - keyed by Phase 2 declaration index from C certificate declaration order
  - value:
      module = C.module
      export_hash = C.export_hash
      fully-qualified declaration name
      decl_interface_hash
      public_export = whether the same name/hash appears in C.export_block
      pointer/key to verified checked declaration view derived from C certificate bytes

VerifiedImportGeneratedDeclTable for import C:
  - keyed by (parent Phase 2 declaration index, generated fully-qualified name)
  - value:
      module = C.module
      export_hash = C.export_hash
      parent fully-qualified declaration name
      generated fully-qualified declaration name
      parent decl_interface_hash
      generated decl_interface_hash (= parent decl_interface_hash)
      public_export = whether the generated name/hash appears in C.export_block
      pointer/key to reconstructed generated declaration view derived from C certificate bytes

CurrentDeclIndexTable:
  - keyed by Phase 5 session current-module source_index
  - value is the unique CheckedCurrentDecl whose source_index equals the key
  - verify additionally builds SourceIndexToCertificateDeclIndex after Phase 2 declaration ordering

CurrentGeneratedDeclTable:
  - keyed by (parent source_index, generated fully-qualified name)
  - value:
      module = current module
      parent_source_index
      parent fully-qualified declaration name
      parent_decl_interface_hash
      generated fully-qualified name
      generated decl_interface_hash (= parent_decl_interface_hash)
      generated interface reconstructed from the parent CheckedCurrentDecl.core_decl
      generated declaration view used by owner-aware renderer QA
```

`VerifiedImportDeclIndexTable` と `VerifiedImportGeneratedDeclTable` は Phase 2 verifier が certificate bytes から
再構築した declaration order、generated constructor / recursor interface、`decl_interface_hash` だけを入力にします。
`ExportBlock` の name 順 Vec position、request order、process-local pointer は使いません。
上の pointer/key は implementation-local handle であり、canonical bytes や table hash には入りません。
handle が指す checked declaration view は certificate bytes または checked current declaration package から
決定的に再構築できなければならず、server-local registry や pretty metadata から補完してはいけません。
`CurrentDeclIndexTable` は `checked_current_decls` が `0..root.source_index` の完全 prefix で、`source_index` が重複しない
場合だけ作れます。current module の non-source private helper を Phase 2 `GlobalRef::Local` として外へ出す payload は
MVP では `InvalidCheckedCurrentDecl` です。
`CurrentGeneratedDeclTable` の `parent fully-qualified declaration name` と `parent_decl_interface_hash` は、
必ず `CurrentDeclIndexTable[parent_source_index]` の checked signature から取ります。
parent entry が存在しない、parent name/hash が reconstructed generated interface の parent と一致しない、
または generated interface hash を再構築できない場合は `InvalidCheckedCurrentDecl` です。
Phase 2 の constructor / recursor export と `GlobalRef::LocalGenerated` dependency は parent `InductiveDecl` の
`decl_interface_hash` を使うため、`generated_decl_interface_hash` は独立値ではなく
`parent_decl_interface_hash` と byte-for-byte で一致しなければなりません。
`MachineGlobalRefView::LocalGenerated` を current generated artifact から materialize する場合も、この parent name/hash を使い、
renderer output、pretty metadata、または generated name の文字列から parent を推測してはいけません。

```text
AxiomRef to MachineAxiomRefWire:
  precondition:
    ref target must resolve to declaration kind AxiomDecl.
    GlobalRef::LocalGenerated is valid for dependency refs, but malformed for axiom refs.

  in verified import context C for module M:
    GlobalRef::Imported(import_index, name, decl_interface_hash):
      C.imports[import_index] の module / export_hash と name / decl_interface_hash から Imported を作る
    GlobalRef::Local(decl_index):
      C.VerifiedImportDeclIndexTable[decl_index] の fully-qualified name / decl_interface_hash と C.module / C.export_hash から
      Imported を作る
    GlobalRef::LocalGenerated(decl_index, name):
      axiom ref としては不正。verified import axiom_report では InvalidVerifiedImport

  in current module context:
    GlobalRef::Imported(import_index, name, decl_interface_hash):
      session imports[import_index] の module / export_hash と name / decl_interface_hash から Imported を作る
    GlobalRef::Local(decl_index):
      CurrentDeclIndexTable[decl_index] の module / fully-qualified name / source_index /
      decl_interface_hash から CurrentModule を作る
    GlobalRef::LocalGenerated(decl_index, name):
      axiom ref としては不正。checked current declaration axiom report では InvalidCheckedCurrentDecl

  in generated certificate current-module context:
    GlobalRef::Imported(import_index, name, decl_interface_hash):
      certificate import table は session direct imports と同じ順序なので、
      session imports[import_index] の module / export_hash と name / decl_interface_hash から Imported を作る
    GlobalRef::Local(certificate_decl_index):
      CertificateDeclIndexToSourceIndex[certificate_decl_index] を引き、
      CurrentDeclIndexTable[source_index] の module / fully-qualified name / source_index /
      decl_interface_hash から CurrentModule を作る
    GlobalRef::LocalGenerated(parent_certificate_decl_index, name):
      axiom ref としては不正。verify で生成した certificate axiom report では VerifyFailed
```

`in current module context` は SourceIndexToCertificateDeclIndex rewrite 前の session / checked current report 用です。
生成 certificate や Phase 2 verifier 出力から `root_axioms_used` / `module_axioms_used` を作る場合は、
`generated certificate current-module context` を使い、certificate-local declaration index を
`CertificateDeclIndexToSourceIndex` で Phase 5 source_index へ戻してから `MachineAxiomRefWire` に変換します。
逆写像できない certificate-local axiom ref、または `AxiomDecl` 以外を指す axiom ref は `VerifyFailed` です。

`DependencyEntry to MachineDependencyRefWire` は、同じ context 規則で `DependencyEntry.global_ref` を解決します。
`DependencyEntry.decl_interface_hash` は `CurrentDeclDependencyEntry.decl_interface_hash` に入れ、
`MachineDependencyRefWire` 自体には重複して入れません。
imported dependency は direct import の public `ExportEntry` に存在する `name + decl_interface_hash` だけを許します。
current-module ordinary dependency は `GlobalRef::Local(source_index)` を `CurrentDeclIndexTable[source_index]` で
解決し、`MachineDependencyRefWire::current_module` に変換します。
current-module generated dependency は `GlobalRef::LocalGenerated(parent_source_index, generated_name)` を
`CurrentGeneratedDeclTable[(parent_source_index, generated_name)]` で解決し、
`MachineDependencyRefWire::current_generated` に変換します。
`DependencyEntry.decl_interface_hash` は解決先の checked signature hash と一致しなければなりません。
generated artifact の場合も parent checked signature hash を使い、独立した generated hash を使ってはいけません。
`LocalGenerated` を `current_module` に潰してはいけません。
current-module dependency は上の table で一意に解決できる prior current declaration / generated artifact
だけを許します。
同一 `InductiveDecl` bundle 内の `Local(package.source_index)` / `LocalGenerated(package.source_index, generated_name)` は
`MachineDependencyRefWire` に変換せず、direct dependency entry から除外します。
それ以外の self / future current-module dependency は `InvalidCheckedCurrentDecl` です。

import certificate 内の `GlobalRef::Local` は、その imported module 自身の declaration であり、current module の
`CurrentModule` に変換してはいけません。
current module の generated constructor / recursor は独立した `source_index` を持ちません。
dependency ref としてだけ `MachineDependencyRefWire::current_generated` に変換し、parent declaration の
`source_index` と generated artifact 自身の fully-qualified name を入れます。
generated constructor / recursor を `MachineAxiomRefWire` に変換してはいけません。
axiom report に generated artifact が現れた場合は、上の `AxiomRef to MachineAxiomRefWire` の規則どおり malformed report
として扱います。
変換できない dependency ref や table 再構築不能な payload は `InvalidCheckedCurrentDecl`、import certificate 内なら `InvalidVerifiedImport`、
verify 時なら `VerifyFailed` です。
比較時は `MachineAxiomRefWire canonical bytes` が完全一致する場合だけ同じ axiom とみなします。
許可されていない axiom が見つかった場合は `DisallowedAxiom` として拒否します。
`eq_family` / `nat_family` は request option そのものではなく、Phase 4 AI の coherent family validation と
builtin family resolution を通した後に `MachineTacticEnv` へ保存された resolved family bytes を
`session_root_hash` と Phase 4 `state_fingerprint` に含めます。
`tactic_options_fingerprint` は Phase 4 AI の `MachineTacticOptions canonical bytes` だけの hash です。
Phase 5 の `MachineApiOptions` 全体に対する hash が必要な場合は `machine_api_options_hash` と呼び、
`tactic_options_fingerprint` と混同してはいけません。

`MachineApiOptions canonical bytes` は、raw request JSON ではなく session create semantic validation 後の
canonicalized options projection です。次で固定します。

```text
MachineApiOptions canonical bytes:
  - tag "npa.phase5.machine-api-options.v1"
  - kernel_check_profile_hash:
      sha256(kernel_check_profile canonical bytes), encoded as HashString digest bytes
  - allow_axioms:
      MachineAxiomRefWire canonical bytes in MachineAxiomRefWire canonical order after sort/dedup
  - Phase 4 MachineTacticOptions canonical bytes:
      exactly the bytes Phase 4 uses for MachineTacticOptions after simp_rules sort/dedup,
      eq_family / nat_family request option encoding, and numeric option validation
  - Phase 4 MachineTacticEnv resolved family option bytes:
      exactly the Phase 4 MachineTacticEnv resolved family bytes after start_machine_proof:
        eq_family field: none tag if MachineTacticEnv.eq_family = None,
          or some tag + ResolvedEqFamily canonical bytes if Some
        nat_family field: none tag if MachineTacticEnv.nat_family = None,
          or some tag + ResolvedNatFamily canonical bytes if Some
  - SimpRegistry canonical hash:
      sha256(Phase 4 SimpRegistry canonical bytes), encoded as HashString digest bytes

machine_api_options_hash:
  sha256(MachineApiOptions canonical bytes)
```

`MachineApiOptions canonical bytes` には tagged bytes 全体を使います。
同じ field list を tag なしで inline encode した bytes や、raw request JSON field order から作った bytes は使いません。
`kernel_check_profile_hash`、resolved family option bytes、`SimpRegistry canonical hash` は、Phase 4 `start_machine_proof`
へ渡す tactic environment から再計算した値と byte-for-byte に対応していなければなりません。
resolved family option bytes は request JSON の `eq_family = null` / `nat_family = null` をそのまま encode するものではありません。
`eq_family = null` でも Phase 4 が builtin Eq head / primitives から
`MachineTacticEnv.eq_family = Some(ResolvedEqFamily)` を作る場合は `some` tag を encode します。
`null` request option が `none` tag になるのは、対応する `MachineTacticEnv` field が実際に `None` の場合だけです。

## 5.2 session create

```json
POST /machine/sessions
{
  "protocol_version": "npa.machine-api.v1",
  "root": {
    "module": "Scratch",
    "theorem_name": "Scratch.t",
    "source_index": 0,
    "universe_params": [],
    "theorem_type": {
      "format": "machine_surface_v1",
      "source": "forall (n : Nat), Eq.{1} Nat n n"
    }
  },
  "import_closure": [
    {
      "module": "Std.Init",
      "expected_export_hash": "sha256:...",
      "expected_certificate_hash": "sha256:...",
      "certificate": {
        "encoding": "npa.certificate.canonical.v0.1.hex",
        "bytes": "..."
      }
    },
    {
      "module": "Std.Nat.Basic",
      "expected_export_hash": "sha256:...",
      "expected_certificate_hash": "sha256:...",
      "certificate": {
        "encoding": "npa.certificate.canonical.v0.1.hex",
        "bytes": "..."
      }
    }
  ],
  "imports": [
    {
      "module": "Std.Init",
      "expected_export_hash": "sha256:...",
      "expected_certificate_hash": "sha256:..."
    },
    {
      "module": "Std.Nat.Basic",
      "expected_export_hash": "sha256:...",
      "expected_certificate_hash": "sha256:..."
    }
  ],
  "checked_current_decls": [],
  "options": {
    "kernel_check_profile": "npa.kernel.v0.1.builtin-none",
    "allow_axioms": [],
    "tactic_options": {
      "simp_rules": [],
      "eq_family": null,
      "nat_family": null,
      "max_simp_rewrite_steps": 100,
      "max_open_goals": 32,
      "max_metas": 64
    }
  }
}
```

`/machine/sessions` request object は次の field だけを持ちます。

```text
required:
  protocol_version
  root
  import_closure
  imports
  checked_current_decls
  options
optional:
  none
```

top-level unknown field、duplicate key、required field omitted、`null`、`protocol_version` が
`"npa.machine-api.v1"` 以外、`root` / `options` の non-object、`import_closure` / `imports` /
`checked_current_decls` の non-array は `InvalidSessionRequest` です。
`root` object の unknown field、省略 field、`module` / `theorem_name` の non-string、`module` の invalid
`ModuleName` grammar、`theorem_name` の invalid `FullyQualifiedName` grammar または
non-renderable `MachineSurfaceRenderableName`、`source_index` の
invalid integer、`universe_params` の non-array、`universe_params` item の `null` / non-string、
invalid universe identifier grammar、duplicate universe parameter、`theorem_type` の non-object は
`InvalidSessionRequest` です。
`theorem_type` object は `format` と `source` だけを持ちます。
`theorem_type` object の duplicate key、unknown field、`format` / `source` omitted、`null`、non-string は
`InvalidSessionRequest` です。
`theorem_type.format` は `"machine_surface_v1"`、`theorem_type.source` は string 必須です。
`root.universe_params` item は `MachineUniverseParamName` でなければなりません。
`.` を含む name、empty string、`_`、reserved keyword spelling は universe parameter として使えません。
`universe_params` は request order を declaration order として保存し、sort / rename / dedup しません。
`source_index` は unsigned u64 integer 必須で、`null`、負数、float、型幅を超える整数は invalid integer です。
`root.theorem_name` は `root.module` の component list を prefix に持ち、かつ declaration component を少なくとも 1 つ
追加した `FullyQualifiedName` でなければなりません。
たとえば `module = "Scratch"` の root theorem は `"Scratch.t"` や `"Scratch.Ns.t"` を許し、`"Other.t"` や
`"Scratch"` は `InvalidSessionRequest` です。
root theorem type の Machine Surface parse/check 失敗は Phase 3 mapping に従い、
`MachineTermParseError` / `MachineTermElaborationError` / `TypeMismatch` などへ写します。
この error は session root context の diagnostic override を使い、`goal_id` と `tactic_kind` は常に none です。
`import_closure` / `imports` の item wire shape error は `InvalidVerifiedImport`、
`checked_current_decls` の item wire shape error は `InvalidCheckedCurrentDecl`、
`options` 以下の wire shape error は `InvalidMachineApiOptions` です。
これらの nested wire shape error は request body の schema validation なので、diagnostic phase は
`request_validation` に固定します。
同じ error kind でも certificate verification、current decl 再検証、options semantic validation の失敗は
後述の semantic validation stage で発生し、diagnostic phase は `session_create` です。

`/machine/sessions` の request wire validation priority は次に固定します。
同じ request が複数の wire shape failure を含む場合でも、最初に失敗した stage の error kind だけを返します。

```text
SessionCreate request wire validation order:
  1. top-level object / duplicate key / unknown field / required field presence / null /
     protocol_version / root object / options object / import_closure array / imports array /
     checked_current_decls array を検査する。
     失敗は InvalidSessionRequest。
  2. root object の field set、module / theorem_name / source_index / universe_params /
     theorem_type object / theorem_type.format / theorem_type.source primitive shape を検査する。
     root.theorem_type.source の Machine Surface parse/check は実行しない。
     失敗は InvalidSessionRequest。
  3. import_closure と imports の item object shape、duplicate key、unknown field、
     required field、certificate wrapper encoding/hex shape を検査する。
     certificate bytes の Phase 2 semantic verification は実行しない。
     失敗は InvalidVerifiedImport。
  4. checked_current_decls item の CheckedCurrentDeclWire shape と canonical decoder を検査する。
     source_index prefix、prior_chain、kernel check は実行しない。
     失敗は InvalidCheckedCurrentDecl。
  5. options object 以下の wire shape、allow_axioms item schema、tactic_options nested object、
     HashString/name/integer primitive shape を検査する。
     import/session-dependent name resolution や family coherence は実行しない。
     失敗は InvalidMachineApiOptions。
```

session create の semantic validation は次の順序で固定します。
同じ request が複数の失敗を含む場合でも、先に失敗した stage の error だけを返します。

```text
SessionCreate semantic validation order:
  1. Request envelope / root primitive / import_closure / imports / checked_current_decls / options の
     wire shape を検査する。
     この段階では root.theorem_type.source の parse/check は実行しない。
     失敗の diagnostic phase は request_validation。

  2. options.kernel_check_profile を検証し、Phase 4 kernel_check_profile_hash を確定する。
     失敗は InvalidMachineApiOptions。

  3. 確定済み kernel_check_profile 上で import_closure と imports を検証し、
     最小 transitive closure と certificate context table を作る。
     失敗は InvalidVerifiedImport。

  4. root.module が MachineImportCertificateContext.verified_modules 内の direct import または
     transitive dependency module と一致しないことを確認する。
     root.theorem_name が DirectPublicExportNameTable の public ExportEntry.name と衝突しないことも確認する。
     いずれかが衝突する場合は InvalidSessionRequest。

  5. verified imports、prior checked declarations、確定済み kernel_check_profile 上で
     checked_current_decls を完全 prefix として再検証し、current decl / generated decl table を作る。
     各 checked_current_decls.signature.name は root.module の component list を prefix に持たなければならない。
     checked_current_decls 内の signature.name 重複、または root.theorem_name と同じ signature.name は
     InvalidCheckedCurrentDecl。
     各 checked_current_decls.signature.name が DirectPublicExportNameTable の public ExportEntry.name と
     衝突する場合も InvalidCheckedCurrentDecl。
     CurrentGeneratedDeclTable に入る generated name は renderable でなければならず、
     root.module の component list を prefix に持たなければならない。
     DirectPublicExportNameTable、root.theorem_name、checked_current_decls.signature.name、または他の generated name と
     衝突してはいけない。衝突する場合は InvalidCheckedCurrentDecl。
     失敗は InvalidCheckedCurrentDecl。

  6. verified imports + checked_current_decls table 上で options の残りを semantic validation する。
     allow_axioms、simp_rules、eq_family、nat_family の解決失敗は InvalidMachineApiOptions。
     allow_axioms item が axiom declaration 以外を指す場合も InvalidMachineApiOptions。
     verified import と checked current decl の axiom dependencies が allow_axioms の subset でない場合は
     DisallowedAxiom。
     sub-order は次に固定する:
       6.1 allow_axioms を MachineAxiomRefWire canonical bytes で sort/dedup し、その順序で実在 axiom へ解決する。
           最初の解決失敗、module/name/hash mismatch、または axiom declaration 以外への参照は InvalidMachineApiOptions。
       6.2 simp_rules を Phase 4 SimpRuleKey canonical order で sort/dedup し、その順序で rule 参照を解決する。
           ここでは name / decl_interface_hash / direction から一意な rule source を見つけるだけで、
           Eq family に依存する rewrite rule shape validation や SimpRegistry 構築はまだ行わない。
           最初の unknown / ambiguous rule reference は InvalidMachineApiOptions。
       6.3 eq_family が object の場合、eq, refl, rec の順に head を解決し、coherent family validation を行う。
           最初の失敗は InvalidMachineApiOptions。
       6.4 nat_family が object の場合、nat, zero, succ, rec の順に head を解決し、coherent family validation を行う。
           最初の失敗は InvalidMachineApiOptions。
       6.5 resolved Eq / Nat family bytes を固定した後、6.2 の順序で Phase 4 SimpRegistry validation を行う。
           Eq family mismatch、conclusion shape error、unsupported rule form などの最初の invalid simp rule は
           InvalidMachineApiOptions。成功した場合だけ Phase 4 MachineTacticOptions と SimpRegistry を確定する。
       6.6 verified import axiom dependencies と checked current decl axiom dependencies の union を
           MachineAxiomRefWire canonical bytes で sort/dedup し、allow_axioms subset か確認する。
           最初の許可されていない axiom を DisallowedAxiom の primary failed axiom とする。

     `simp_rules`、`eq_family`、`nat_family` の name / decl_interface_hash は、次の Phase 5 option head
     resolution scope でだけ解決します。
     Phase 4 の `start_machine_proof` へ渡す前に、Phase 5 adapter は同じ scope で unknown / ambiguous /
     unsupported current-generated reference を検出し、すべて InvalidMachineApiOptions として扱います。

     Phase5 option head resolution:
       - direct import public ExportEntry.name と decl_interface_hash が一致する場合:
           Phase 4 `TacticHead::Imported { name, decl_interface_hash }` にする。
           ordinary declaration だけでなく、direct import public generated constructor / recursor もここに含む。
       - checked current declaration signature.name と decl_interface_hash が一致する場合:
           Phase 4 `TacticHead::CurrentModule { name, decl_interface_hash }` にする。
       - CurrentGeneratedDeclTable にだけ一致する current generated constructor / recursor:
           Phase 4 external option schema に `CurrentGenerated` variant がないため InvalidMachineApiOptions。
           raw Machine Surface term source では参照できても、SimpRuleRef / EqFamilyRef / NatFamilyRef には使えない。
       - transitive dependency、imported private declaration、display render scope にだけ存在する declaration:
           option head resolution scope 外なので InvalidMachineApiOptions。
       - direct import public と checked current declaration の両方、または複数 direct import に一致する場合:
           session create の collision validation を通っていれば発生しないが、発見した場合は
           InvalidMachineApiOptions。

     `SimpRuleRef` は上の head resolution 後、direction を加えた Phase 4 `SimpRuleKey` として検証します。
     `EqFamilyRef` は eq / refl / rec、`NatFamilyRef` は nat / zero / succ / rec の順でこの head resolution を使い、
     その後に Phase 4 の coherent family validation を実行します。

  7. verified imports、checked_current_decls table、CurrentGeneratedDeclTable から
     MachineSurfaceCallableInterfaceTable を決定的に構築する。
     この table の entry 集合は後述の display render scope 全体ではなく、root theorem type / candidate validation /
     replay validation で Machine Surface elaborator が参照できる callable だけを対象にする。
     direct import public ExportEntry に含まれる generated constructor / recursor も、ordinary declaration と同じく
     direct import public callable として table に入れる。
     imported generated callable の key は LocalGenerated ではなく、
     ExportEntry.name / ExportEntry.decl_interface_hash を使った MachineSurfaceCallableRef::Imported とする。
     imported callable の table 構築失敗は InvalidVerifiedImport、checked current / current generated callable の
     table 構築失敗は InvalidCheckedCurrentDecl。
     同じ MachineSurfaceCallableRef canonical bytes を持つ entry が複数生成された場合は、
     implicit_profile が同一でも重複として拒否する。
     imported callable の duplicate は InvalidVerifiedImport、checked current / current generated callable の
     duplicate は InvalidCheckedCurrentDecl。
     server-local registry、source span、package cache、UI 設定、HashMap iteration order に依存した profile は
     この段階で拒否する。
     続けて direct import の theorem-index-visible public `TheoremDecl` / `AxiomDecl` export について、
     `ExportEntry.name`、`ExportEntry.universe_params`、statement rendering に必要な display render scope name を
     preflight する。
     この preflight は theorem index を response として返す処理ではないが、同じ verified payload から
     `statement.machine` を deterministic に作れることを確認する。
     失敗は imported certificate が Phase 5 AI MVP の Machine API profile に適合しないものとして
     InvalidVerifiedImport、diagnostic phase session_create です。

  8. 1-7 が成功した後だけ、root.universe_params を request order の level context として導入し、
     verified imports + checked_current_decls prefix の checked environment で
     root.theorem_type.source を Phase 3 AI Complete mode で parse/canonicalize/check する。
     この Phase 3 term check には、stage 7 で構築した `MachineSurfaceCallableInterfaceTable` を
     callable-interface input として必ず渡す。
     Phase 3 の実装 API が `MachineTermElabContext` にこの field をまだ持たない場合でも、
     Phase 5 adapter は wrapper context または明示引数でこの table を渡さなければならない。
     この table を渡せない実装は Phase 5 MVP v1 と互換ではなく、session create では
     InvalidMachineProofState、diagnostic phase session_create として扱う。
     root theorem type の Machine Surface global scope は次の exact-name map だけです:
       direct import public ExportEntry.name
       checked_current_decls prefix に含まれる checked current declaration signature.name
       checked_current_decls prefix に含まれる current generated constructor / recursor name
     この scope には root theorem 自身、transitive dependency の public export、imported private declaration、
     display render scope にだけ存在する constant を入れてはいけません。
     この scope で解決できない名前は stage 8 の UnknownName / machine_term_check であり、
     stage 9 の RootTheoremTypeDependencyReport 変換失敗として遅延させてはいけません。
     stage 8 が成功した後に stage 9 でこの scope 外の GlobalRef が見つかった場合だけ、
     Phase 5 / Phase 3 adapter invariant failure として InvalidMachineProofState、diagnostic phase session_create です。
     root.module / theorem_name / source_index は MachineProofSpec と later certificate construction の metadata として
     記録するだけで、この Phase 3 `MachineTermElabContext` 内部の global scope には root theorem 自身を入れない。
     失敗は MachineTermParseError / MachineTermElaborationError / UnknownName /
     ImplicitArgumentRequired / TypeMismatch / ExpectedPiType。

  9. checked root theorem type から RootTheoremTypeDependencyReport を再計算する。
     これは verify handoff の RootTheoremDependencyReport と同じ dependency derivation rule の type-only subset であり、
     final proof body はまだ存在しないため proof dependency は含めない。
     この report は session create 中の一時検査 artifact であり、wire response、CheckedMachineProofRoot canonical bytes、
     SessionRoot canonical bytes、Phase 4 state_fingerprint、certificate payload には入れない。
     direct_dependency_entries は checked root theorem type の core Expr に syntactically 出現する global Const だけから作る。
     root theorem 自身は stage 8 の global_scope に入らないため、self dependency は存在してはならない。
     各 GlobalRef は 5.1 の DependencyEntry to MachineDependencyRefWire と同じ session context 規則で変換する。
     imported dependency は direct import の public ExportEntry だけを許し、current-module dependency は
     checked_current_decls の完全 prefix に含まれる source_index だけを許す。
     current-generated dependency は parent_source_index が checked_current_decls prefix にある generated artifact だけを許す。
     RootTheoremTypeDependencyReport は DependencyEntry payload を経由しないため、各 CurrentDeclDependencyEntry の
     decl_interface_hash は参照種別ごとに次から取る:
       imported ref は matching direct import public ExportEntry.decl_interface_hash を入れ、GlobalRef::Imported が持つ
       decl_interface_hash と一致しなければならない。
       current_module ref は checked_current_decls prefix の対応 source_index にある checked signature.decl_interface_hash を入れる。
       current_generated ref は CurrentGeneratedDeclTable[(parent_source_index, generated_name)] の
       parent checked signature.decl_interface_hash を入れる。
     変換後の direct_dependency_entries は CurrentDeclDependencyEntry canonical bytes と同じ順序で sort/dedup する。
     axiom_dependencies は theorem type の direct_dependency_entries から次の closure で作る:
       imported ref は direct import ExportEntry の axiom_dependencies を MachineAxiomRefWire に変換して追加する。
       current_module ref は対応する prior CurrentDeclDependencyReport.axiom_dependencies を追加する。
       current_generated ref は parent declaration の prior CurrentDeclDependencyReport.axiom_dependencies を追加する。
       root theorem type 自身は axiom declaration ではないため self axiom は追加しない。
       最後に MachineAxiomRefWire canonical bytes で sort/dedup する。
     GlobalRef / dependency / axiom ref をこの規則で変換できない場合、または stage 8 の checked Expr から
     session context に存在しない ref が見つかった場合は session invariant failure として
     InvalidMachineProofState、diagnostic phase session_create です。
     report の axiom_dependencies を MachineAxiomRefWire canonical bytes で sort/dedup し、allow_axioms subset か確認する。
     最初の許可されていない axiom を DisallowedAxiom の primary failed axiom とする。

  10. Phase 4 start_machine_proof と初期 MachineProofState 構築を行う。
     失敗は InvalidMachineProofState。

  11. session_root_hash、initial snapshot、session_id を確定する。
      response を返す前に initial snapshot を session snapshot store へ保存しなければならない。
      保存 entry は 6.4 の Stored snapshot entry であり、executable_state_payload は stage 10 の
      初期 MachineProofState を lossless に復元できる payload、materialized_view_payload は
      その初期 state から materialize した StoredSnapshotView canonical bytes です。
      response の initial_snapshot.snapshot_id はこの保存 entry を指す。
      保存できない、または保存直後の self-check が失敗する場合は InvalidMachineProofState、
      diagnostic phase session_create です。
```

import certificate verification は selected `kernel_check_profile` の core spec / kernel semantics / reduction /
universe profile と互換でなければなりません。
MVP の Phase 2 certificate canonical bytes、`export_hash`、`certificate_hash` は
`kernel_check_profile` 全体ではなく、certificate format version と core spec version にだけ profile 情報を持ちます。
`builtin-none` / `builtin-eq-nat` の違いは imported certificate bytes、`export_hash`、`certificate_hash` の入力に入れません。
したがって MVP の import certificate compatibility check は次だけです。

```text
Import certificate / kernel profile compatibility:
  - certificate format version is supported by the Phase 2 verifier
  - certificate embedded core spec id equals selected kernel_check_profile core spec id
  - selected kernel semantics profile id equals "npa-kernel.phase1.v0.1"
  - selected reduction profile id equals "beta-delta-iota-zeta.v0.1"
  - selected universe profile id equals "levels-imax-v0.1"
  - verifier recomputed export_hash / certificate_hash equal request expected hashes
```

上のいずれかが certificate payload または recomputed hash と矛盾する場合は `InvalidVerifiedImport` です。
selected builtin profile id は import certificate compatibility check では比較せず、Phase 4 tactic environment、
`state_fingerprint`、root theorem / checked current declaration の kernel check profile としてだけ使います。
将来 builtin profile が certificate canonical bytes や hash に影響する場合は、Phase 2 certificate format version と
Phase 5 `protocol_version` を同時に上げ、その profile id を certificate hash 入力に入れる規則を追加します。
profile id そのものが未知または現在の kernel が公開する profile hash と一致しない場合は、import bytes を検証する前に
`InvalidMachineApiOptions` を返します。
したがって、profile が valid で import が壊れていて theorem_type も壊れている request は必ず
`InvalidVerifiedImport` を返します。
`root.theorem_type` の check は import / options / current decl が確定した環境だけを入力にし、
session_root_hash は 5.3 の `SessionRoot canonical bytes` に列挙した canonicalized successful inputs から計算します。
stage 9 の `RootTheoremTypeDependencyReport` は `theorem_type_core_hash`、imports、checked current decls、`allow_axioms` からの
派生検査であり、独立した bytes として `session_root_hash` に追加しません。

レスポンス:

```json
{
  "status": "ok",
  "session_id": "msess_001",
  "session_root_hash": "sha256:...",
  "initial_snapshot": {
    "session_id": "msess_001",
    "snapshot_id": "mst_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "state_fingerprint": "sha256:...",
    "tactic_options_fingerprint": "sha256:...",
    "open_goals": ["g0"],
    "goals": [
      {
        "goal_id": "g0",
        "meta_id": "m0",
        "context_hash": "sha256:...",
        "local_name_map_hash": "sha256:...",
        "context": [],
        "target": {
          "core_hash": "sha256:...",
          "head": null,
          "constants": [
            {
              "kind": "imported",
              "module": "Std.Init",
              "name": "Eq",
              "export_hash": "sha256:...",
              "decl_interface_hash": "sha256:...",
              "public_export": true,
              "tactic_head_visible": true
            },
            {
              "kind": "imported",
              "module": "Std.Nat.Basic",
              "name": "Nat",
              "export_hash": "sha256:...",
              "decl_interface_hash": "sha256:...",
              "public_export": true,
              "tactic_head_visible": true
            }
          ],
          "free_locals": [],
          "size": 8,
          "machine": "forall (n : Nat), Eq.{1} Nat n n"
        },
        "target_hash": "sha256:...",
        "goal_fingerprint": "sha256:...",
        "allowed_tactics": ["intro", "exact", "apply"]
      }
    ],
    "proof_skeleton_hash": "sha256:..."
  }
}
```

`initial_snapshot` は `/machine/snapshots/get` が返す `MachineProofSnapshot` wire payload と同じ shape です。
ただし `/machine/sessions` request は `include_pretty` を持たないため、session create response の
`initial_snapshot` は常に `include_pretty = false` 相当です。
`MachineExprView.pretty` などの pretty-only field は omit し、`pretty: null` を返してはいけません。
pretty 付きの initial snapshot が必要な client は、返却された `snapshot_id` / `state_fingerprint` で
`/machine/snapshots/get` を `include_pretty = true` として呼び直します。
session create が success response を返した時点で、この `snapshot_id` / `state_fingerprint` は
同じ session の snapshot store から取得可能でなければなりません。

session 作成直後に top-level binder を自動で `intro` しません。
初期 goal の target は `root.theorem_type` そのものです。

```text
initial goal:
  context = []
  target  = forall (n : Nat), Eq.{1} Nat n n
```

`n : Nat` を context に持つ goal は、`intro n` の成功後に作られる次の snapshot で表します。

## 5.3 session_root_hash

`session_root_hash` は次の canonical bytes から計算します。

```text
SessionRoot canonical bytes:
  - tag "npa.phase5.session-root.v1"
  - protocol_version
  - CheckedMachineProofRoot canonical bytes:
      module / theorem_name / source_index / universe_params
      theorem_type_source.phase3_canonical_hash
      theorem_type_core_hash
  - import certificate context after validation, sort, and dedup:
      verified module keys in canonical order:
        module / export_hash / certificate_hash
      each module certificate import table key list in certificate order
      decoded_name_table_hash
      decl_index_table_hash
      generated_decl_table_hash
  - machine_surface_callable_interface_table_hash
  - imports after validation, sort, and dedup:
      module / export_hash / certificate_hash
      export_signature_summary_hash
      certified_env_decl_hashes_summary_hash
      axiom_report_hash
  - checked_current_decls in source_index order:
      decoded CheckedCurrentDeclPackage canonical bytes
  - MachineApiOptions canonical bytes
```

`SessionRoot canonical bytes` が取り込む `MachineApiOptions canonical bytes` 内の
resolved family option bytes は、Phase 4 `MachineTacticEnv resolved family bytes` と
byte-for-byte 同じです。これは request JSON の `eq_family` / `nat_family` value ではなく、
Phase 4 `start_machine_proof` が作った `MachineTacticEnv.eq_family` / `MachineTacticEnv.nat_family`
field を encode します。request が `null` でも builtin Eq などにより `MachineTacticEnv.eq_family = Some(_)`
になる場合は `some` tag を encode し、`none` tag は対応する resolved field が実際に `None` の場合だけ使います。

raw request JSON の field order、import request order、pretty text、source span、server-local store key、
session_id は `session_root_hash` に入りません。
`export_signature_summary_hash` と `certified_env_decl_hashes_summary_hash` は client から受け取る hash ではなく、
Phase 2 verifier output から Phase 5 が再計算する direct import ごとの summary hash です。
`certificate_hash` が full certificate bytes を固定していても、Phase 5 / Phase 4 adapter が使う import projection を
実装差なく固定するために、この 2 つを明示して `session_root_hash` に入れます。

```text
ExportSignatureSummary canonical bytes:
  - tag "npa.phase5.export-signature-summary.v1"
  - module / export_hash / certificate_hash
  - ExportEntry summaries in Phase 2 ExportBlock canonical order:
      name as Phase5Name canonical bytes
      kind as Phase 2 ExportKind numeric tag
      universe_params in ExportEntry order:
        each NameId must decode to a single-component Phase 2 Name
        encode that component as Phase5 UTF-8 string primitive bytes
      type_hash as HashString digest bytes
      body_hash option as HashString digest bytes
      reducibility option as Phase 2 numeric tag
      opacity option as Phase 2 numeric tag
      decl_interface_hash as HashString digest bytes
      axiom_dependencies_hash =
        sha256(Phase 2 AxiomRef canonical bytes list in Phase 2 canonical order)

export_signature_summary_hash:
  sha256(ExportSignatureSummary canonical bytes)

CertifiedEnvDeclHashesSummary canonical bytes:
  - tag "npa.phase5.certified-env-decl-hashes-summary.v1"
  - module / export_hash / certificate_hash
  - declarations in Phase 2 certificate declaration order:
      decl_index as minimal unsigned LEB128 u64
      DeclHashes.decl_interface_hash as HashString digest bytes
      DeclHashes.decl_certificate_hash as HashString digest bytes

certified_env_decl_hashes_summary_hash:
  sha256(CertifiedEnvDeclHashesSummary canonical bytes)
```

`ExportSignatureSummary.universe_params` は `Phase5Name canonical bytes` ではなく、Phase 4
`CheckedDeclSignature.universe_params` と同じ string list へ投影して encode します。
`ExportEntry.universe_params` の NameId が single-component Name に decode できない場合、Phase 4
`CheckedDeclSignature canonical bytes` を再構築できないため `InvalidVerifiedImport` です。
この summary hash では sort / rename / dedup を行いません。
`MachineUniverseParamName` validation は Machine Surface へ render する endpoint の別条件であり、
summary hash の encoding rule ではありません。

`ExportEntry.type` / `body` term bytes、proof body bytes、certificate table bytes をここへ再コピーしてはいけません。
それらは `export_hash` / `certificate_hash` と verifier output の自己整合性検査で固定します。
summary hash の入力は、Phase 2 verifier が受理した canonical payload から再構築した値だけです。
request JSON、Phase 4 cache、server-local verified module store、または pretty metadata から補完してはいけません。

## 5.4 session delete

`DELETE /machine/sessions/{id}` は resource cleanup だけを行う API です。
proof state の deterministic fingerprint や replay plan には入りません。

```http
DELETE /machine/sessions/msess_001
```

レスポンス:

```json
{
  "status": "deleted",
  "session_id": "msess_001"
}
```

存在しない session は `UnknownSession` を返します。
request body は持ちません。
body が存在する、path id が UTF-8 でない、または path id が `SessionId wire` grammar に一致しない場合は
`InvalidSessionRequest` です。

---

# 6. Machine Proof Snapshot

## 6.0 Hash wire format

API 上の hash は `HashString` として固定します。

```text
HashString:
  "sha256:" + 64 lowercase hex chars

SnapshotId wire:
  "mst_" + 64 lowercase hex chars
```

MVP では hash algorithm は `sha256` だけを許します。
`snapshot_id` は `state_fingerprint` の digest hex だけから導出し、`sha256:` prefix は入れません。
将来 algorithm を増やす場合は `snapshot_id = "mst_" + algorithm + "_" + digest_hex` のような
別 protocol version にします。

## 6.1 Snapshot の構造

```rust
struct MachineProofSnapshot {
    snapshot_id: SnapshotId,
    session_id: SessionId,
    state_fingerprint: Hash,
    tactic_options_fingerprint: Hash,
    open_goals: Vec<GoalId>,
    goals: Vec<MachineGoalView>,
    proof_skeleton_hash: Hash,
}
```

`open_goals` は current snapshot の open goal list の authoritative order です。
wire response の `goals` は `open_goals` と同じ goal set を同じ順序で含めなければなりません。
closed / assigned goal、prefix 外の archived goal、または debug 用 hidden goal は `goals` に含めません。
`goals[*].goal_id` の重複、`open_goals` にない goal、`open_goals` にある goal の欠落、順序不一致は
snapshot materialization failure とし、endpoint ごとの `InvalidMachineProofState` に写します。

`tactic_options_fingerprint` は Phase 4 AI の `MachineTacticOptions canonical bytes` から再計算される hash です。
Phase 5 snapshot は `env_fingerprint` という別 hash を返しません。
verified imports、checked current declarations、SimpRegistry、resolved Eq / Nat family bytes は Phase 4
`state_fingerprint` に含まれます。
verified imports の projection は 5.1 の `Phase 4 verified import projection used by Phase 5` に固定し、
Phase 5 の summary hash や request/store 由来の hash で代用してはいけません。
`tactic_options_fingerprint` は resolved family bytes を直接含めません。

`proof_skeleton_hash` は Phase 4 `ProofRoot.body` の `ProofExpr canonical bytes` から計算します。
Phase 4 AI で定義済みの `sha256(ProofExpr canonical bytes)` と同じ hash family を使い、Phase 5 独自の追加 tag は
重ねません。
`ProofRoot.body` 内の `ProofExpr::Meta(id)` は meta id 参照としてそのまま encode し、対応する metavariable の
`assignment`、`context`、`target`、open goal order、pretty text、tactic trace、scheduler artifact は
`proof_skeleton_hash` には入りません。
metavariable assignment や open goal set の変化は Phase 4 `state_fingerprint` に含まれるため、snapshot の同一性確認は
常に `state_fingerprint` で行います。
stored snapshot の `proof_skeleton_hash` が `ProofRoot.body` から再計算した値と一致しない場合は
`InvalidMachineProofState` です。

`snapshot_id` は full `state_fingerprint` から導出します。

```text
snapshot_id = "mst_" + digest_hex(state_fingerprint)
```

prefix 省略形は wire format では使いません。
request の `snapshot_id` が `SnapshotId wire` grammar に合わない場合は endpoint ごとの request validation error として拒否します。
`snapshot_id` は content-addressed handle ですが、lookup authority は常に request の `session_id` が指す
current session の snapshot store です。
別 session に同じ `snapshot_id` が存在する場合や、request の `state_fingerprint` から導出した digest と
`snapshot_id` が一致する場合でも、current session の snapshot store に entry がなければ `UnknownSnapshot` です。
session-scoped lookup に成功した後、まず stored snapshot 自身の identity invariant を検査します。
stored snapshot 自身の `snapshot_id` が stored `state_fingerprint` から導出した値と一致しない場合は
store invariant failure として `InvalidMachineProofState` です。
続いて stored snapshot entry 全体の materialization self-check を実行します。
`/machine/snapshots/get`、search、prompt のように response construction では view だけを読む endpoint でも、
stored snapshot entry は `executable_state_payload` と `materialized_view_payload` の両方を持たなければなりません。
`executable_state_payload` を復元し、復元 state から再計算した `state_fingerprint`、`proof_skeleton_hash`、
各 goal の `context_hash` / `target_hash` が stored view と一致することを検査します。
さらに `materialized_view_payload` の `StoredSnapshotView canonical bytes` が復元 state から再 materialize した bytes と
byte-for-byte に一致することを検査します。
どちらかの payload 欠落、復元不能、view projection との矛盾、または stored view canonical bytes と復元 state の不一致は
request の `state_fingerprint` と比較する前に `InvalidMachineProofState` として拒否します。
この stored snapshot self-check に成功した後で、stored snapshot の `state_fingerprint` と request の
`state_fingerprint` を比較し、一致しない場合は `StateFingerprintMismatch` です。
`/machine/snapshots/get`, `/machine/tactics/run`, `/machine/tactics/batch`, `/machine/search/for_goal`,
`/machine/prompt_payload`, `/machine/verify` は
`snapshot_id` に加えて request の `state_fingerprint` を必須にします。
これらの endpoint は session-scoped lookup、stored snapshot self-check、stored / request
`state_fingerprint` 比較をこの順序で実行します。

## 6.2 MachineGoalView

AI に渡す goal は、pretty 表示と機械表現を明確に分けます。

```rust
struct MachineGoalView {
    goal_id: GoalId,
    meta_id: MetaVarId,
    context_hash: Hash,
    local_name_map_hash: Hash,
    context: Vec<MachineLocalView>,
    target: MachineExprView,
    target_hash: Hash,
    goal_fingerprint: Hash,
    allowed_tactics: Vec<MachineTacticKind>,
}

struct MachineLocalView {
    local_id: LocalId,
    machine_name: String,
    display_name: String,
    ty: MachineExprView,
    value: Option<MachineExprView>,
    depends_on: Vec<LocalId>,
    binder_index: u32,
}

struct MachineExprView {
    core_hash: Hash,
    head: Option<MachineGlobalRefView>,
    constants: Vec<MachineGlobalRefView>,
    free_locals: Vec<LocalId>,
    size: u32,
    machine: String,
    pretty: Option<String>,
}

enum MachineGlobalRefView {
    Imported {
        module: ModuleName,
        name: FullyQualifiedName,
        export_hash: HashString,
        decl_interface_hash: HashString,
        public_export: bool,
        tactic_head_visible: bool,
    },
    CurrentModule {
        module: ModuleName,
        name: FullyQualifiedName,
        decl_interface_hash: HashString,
        source_index: u64,
    },
    LocalGenerated {
        module: ModuleName,
        export_hash: Option<HashString>,
        parent_name: FullyQualifiedName,
        name: FullyQualifiedName,
        parent_decl_interface_hash: HashString,
        decl_interface_hash: HashString,
        public_export: bool,
        tactic_head_visible: bool,
    },
}
```

`GoalId` / `MetaVarId` / `LocalId` の wire grammar は次で固定します。

```text
GoalId wire:
  "g" + decimal_u64
  canonical bytes = Phase 4 GoalId(decimal_u64) canonical bytes

MetaVarId wire:
  "m" + decimal_u64
  canonical bytes = Phase 4 MetaVarId(decimal_u64) canonical bytes

LocalId wire:
  "l" + decimal_u32
  meaning = 0-based index into this MachineGoalView.context
  canonical bytes:
    - tag "npa.phase5.local-id.v1"
    - context index as minimal unsigned LEB128 u32
```

`MachineExprView.core_hash` は Phase 1 `Expr` canonical payload を
`NPA-PHASE1-EXPR-0.1` domain で hash した値です。
`machine`、`pretty`、`head`、`constants`、`free_locals`、`size` は派生 view であり、`core_hash` の入力ではありません。
Phase 3 term 単体 API の `MachineTermCheckResult.contextual_core_hash` は owner / import context 固定用の別 hash であり、
`MachineExprView.core_hash`、`MachineGoalView.target_hash`、`theorem_type_core_hash` の代わりに使ってはいけません。
`MachineGoalView.target_hash` は Phase 4 `target_hash` と同じ hash family で、必ず
`MachineGoalView.target.core_hash` と byte-for-byte で一致しなければなりません。
一致しない stored snapshot は `InvalidMachineProofState` です。
`MachineLocalView.ty.core_hash` と `MachineLocalView.value.core_hash` も同じ Phase 1 `Expr` structural hash rule で計算します。

`MachineExprView.head`、`MachineExprView.constants`、`MachineExprView.machine` の materialization は、
core `Expr` に加えて、その `Expr` 内の `GlobalRef::Local` / `GlobalRef::LocalGenerated` を解釈する
owner context を入力にします。
current proof state の goal target / local type / local value / root theorem type は
`Phase5ResolvedDisplayCoreRefOwner::CurrentSessionRootModule { module = session.root.module }` を使います。
imported theorem search result、prompt premise `statement.machine`、imported `ExportEntry.type` 由来の expression は
その export entry が属する verified certificate の
`Phase5ResolvedDisplayCoreRefOwner::VerifiedImportedModule { owner_module = module, owner_export_hash = export_hash }`
を使います。
`MachineExprView.core_hash` は従来どおり owner を含まない Phase 1 `Expr` canonical bytes の hash ですが、
`head` / `constants` / `machine` を作るときは owner context なしで `GlobalRef` を解釈してはいけません。
同じ `GlobalRef::Local(0)` でも owner module が異なれば別 declaration です。

`MachineExprView.head` は core `Expr` の syntactic application head だけを表します。
renderer は WHNF、δ/β/ι/ζ reduction、conversion、let unfolding、Pi/forall binder peeling、annotation peeling を
行ってはいけません。

```text
MachineExprView.head extraction:
  1. input expr を Phase 1 core Expr のまま見る
  2. expr が App(f, a) である間、f へ進む
  3. 到達した head が global constant の場合だけ、materialization owner context と一緒に
     6.2 の MachineGlobalRefView へ正規化して some を返す
  4. head が local variable、Sort/Type/Prop、Pi/forall、Lambda、Let、またはその他の non-global non-Meta node なら none
```

`MachineExprView.constants` は同じ core `Expr` 全体を syntactic に走査し、binder body、local let の type/value/body、
application の function/argument などに現れる global constant をすべて `MachineGlobalRefView` に変換して
canonical order で sort/dedup します。
この変換も `MachineExprView.head` と同じ materialization owner context で行います。
`MachineExprView.head` は theorem index の `head symbol option` とは別規則です。
theorem index は leading syntactic `Pi` を peel しますが、一般の `MachineExprView.head` は peel しません。

owner-aware `GlobalRef` から `MachineGlobalRefView` を作るとき、public generated constructor / recursor は
candidate source では `GlobalRef::Imported(import_index, generated_name, generated_decl_interface_hash)` として
現れ得ますが、view では `MachineGlobalRefView::LocalGenerated` に戻します。
`MachineGlobalRefView::Imported` は ordinary declaration だけを表します。

この文書で `tactic_head_visible` を決める場合は、次の predicate だけを使います。

```text
direct_public_tactic_head_visible(module, name, export_hash, decl_interface_hash):
  DirectPublicExportNameTable[name] を exact lookup し、
  unique direct public ExportEntry が存在し、
  その ExportEntry の module / export_hash / name / decl_interface_hash がすべて一致する場合だけ true。

  generated constructor / recursor の場合、name と decl_interface_hash は
  generated ExportEntry.name / generated ExportEntry.decl_interface_hash を使う。
  parent_name や parent_decl_interface_hash では判定しない。
```

`tactic_head_visible = true` を module が direct import かどうか、`public_export` flag、または import order から
別々に推測してはいけません。
この predicate に失敗した imported / transitive / private head は、表示や fingerprint には残せますが
Phase 4 external `TacticHead::Imported` へ変換してはいけません。

```text
Owner-aware GlobalRef to MachineGlobalRefView:
  owner_context = CurrentSessionRootModule:
    GlobalRef::Imported(import_index, name, decl_interface_hash):
      session.imports[import_index] の direct public ExportEntry を name / decl_interface_hash で引く。
      ExportEntry が ordinary declaration に対応する場合:
        Imported { module, name, export_hash, decl_interface_hash,
        public_export = true,
        tactic_head_visible = direct_public_tactic_head_visible(module, name, export_hash, decl_interface_hash)
        } にする。
      ExportEntry が generated constructor / recursor に対応する場合:
        VerifiedImportGeneratedDeclTable から parent_name / parent_decl_interface_hash を取得し、
        LocalGenerated { module, export_hash = some(export_hash), parent_name, name,
        parent_decl_interface_hash, decl_interface_hash,
        public_export = true,
        tactic_head_visible = direct_public_tactic_head_visible(module, name, export_hash, decl_interface_hash)
        } にする。
      ordinary と generated の両方に一致する、またはどちらにも一致しない場合は materialization failure。

    GlobalRef::Local(source_index):
      CurrentDeclIndexTable[source_index] から CurrentModule view を作る。

    GlobalRef::LocalGenerated(parent_source_index, generated_name):
      CurrentGeneratedDeclTable[(parent_source_index, generated_name)] から
      LocalGenerated { module = session.root.module, export_hash = none, ...,
      public_export = false, tactic_head_visible = false } を作る。

  owner_context = VerifiedImportedModule { owner_module, owner_export_hash }:
    GlobalRef::Imported(import_index, name, decl_interface_hash):
      owner verified module の certificate_import_table[import_index] を引き、
      dependency verified module { dependency_module, dependency_export_hash } の public ExportEntry を
      name / decl_interface_hash で引く。
      dependency ExportEntry が ordinary declaration に対応する場合は Imported view を作る。
      generated constructor / recursor に対応する場合は VerifiedImportGeneratedDeclTable から
      parent_name / parent_decl_interface_hash を取得し、
      LocalGenerated { module = dependency_module, export_hash = some(dependency_export_hash), parent_name, name,
      parent_decl_interface_hash, decl_interface_hash, ... } を作る。
      tactic_head_visible は
      direct_public_tactic_head_visible(dependency_module, name, dependency_export_hash, decl_interface_hash)。

    GlobalRef::Local(decl_index):
      owner verified module の VerifiedImportDeclIndexTable[decl_index] から Imported view を作る。
      public_export は table.public_export。
      tactic_head_visible は table.public_export かつ
      direct_public_tactic_head_visible(owner_module, name, owner_export_hash, decl_interface_hash)。

    GlobalRef::LocalGenerated(parent_decl_index, generated_name):
      owner verified module の VerifiedImportGeneratedDeclTable[(parent_decl_index, generated_name)] から
      LocalGenerated view を作る。
      public_export は table.public_export。
      tactic_head_visible は table.public_export かつ
      direct_public_tactic_head_visible(owner_module, generated_name, owner_export_hash, generated_decl_interface_hash)。
```

`MachineExprView.machine` を作る対象の core `Expr` は Meta-free でなければなりません。
Phase 4 proof state の未解決 metavariable は open goal / metavariable store で表し、goal target、local type、
local value、theorem index statement の中へ `Meta` node として埋め込んではいけません。
renderer は `Meta` の surface syntax を持たず、input expr のどこかに `Meta` がある場合は
`MachineExprView.head` / `constants` / `machine` の partial projection を返してはいけません。
snapshot / prompt rendering では `InvalidMachineProofState`、theorem index / search rendering では
`InvalidTheoremIndex` とし、diagnostic phase は下の renderer caller table に従います。

`decimal_u64` / `decimal_u32` は ASCII digits だけで、0 は `"0"`、0 以外は leading zero を許しません。
JSON number、empty suffix、`+` / `-`、underscore、whitespace、型幅を超える値、未知 prefix は invalid wire id です。
request の `goal_id` が grammar に合わない場合は `InvalidSnapshotRequest` / `InvalidTacticRunRequest` /
`InvalidTheoremQuery` /
`InvalidReplayPlan` など、その endpoint の request validation error として拒否します。
grammar は正しいが current snapshot の open goal に存在しない `goal_id` は `GoalNotOpen` です。
`LocalId` は Phase 4 state の永続 id ではなく、Phase 5 view 内だけの安定 handle です。
context order は Phase 4 `MachineMetaVar.context` の Vec order そのものです。
index 0 はその goal context で最も古い / 外側の local declaration、`context.len - 1` は最も新しい / 内側の
local declaration です。
`MachineLocalView.local_id` は必ずその local の context index と一致します。
`MachineLocalView.binder_index` も同じ context index を u32 として返し、de Bruijn index ではありません。
追加 binder のない `MachineExprView.machine` 内で de Bruijn index に換算する必要がある場合は
`de_bruijn_index = context.len - 1 - binder_index` ですが、この換算値を wire payload や hash には入れません。
`free_locals` / `depends_on` は同じ context 内の valid `LocalId` だけを参照します。
context 外 index、重複 `local_id`、context order と一致しない `local_id`、または
`local_id` と `binder_index` の数値不一致は `InvalidMachineProofState` です。

`pretty` は debug / prompt 用の補助情報です。
`machine` は Phase 3 AI Machine Surface の canonical source です。
どちらも certificate payload には入りません。
wire response では `pretty` 以外の field は必須です。
Snapshot / goal view の `MachineLocalView` wire object は、Rust-like struct と同じ
`local_id`, `machine_name`, `display_name`, `ty`, `value`, `depends_on`, `binder_index` だけを持ちます。
これらはすべて必須 field です。
`ty` は `MachineExprView` object、`value` は required option field で local assumption なら JSON `null`、
local let なら `MachineExprView` object です。
Snapshot / goal view では prompt 用の `type_machine` / `value_machine` field 名を使ってはいけません。
`display_name` は snapshot / goal view では `include_pretty` に関係なく必須で、
MVP では常に `machine_name` と同じ string です。
`include_pretty = false` の場合、`pretty` field は omit し、`pretty: null` は返しません。
Machine snapshot / goal view response の `Option<T>` field は、別途明記しない限り field を必須で返し、
`none` を JSON `null`、`some` を通常の payload として encode します。
MVP でこの required-null rule を使う field は `MachineLocalView.value`、`MachineExprView.head`、
`MachineGlobalRefView.LocalGenerated.export_hash` です。
したがって local assumption は `"value": null`、local let は `"value": MachineExprView object` として返します。
`value` 内の nested `MachineExprView.pretty` は他の pretty field と同じく `include_pretty = false` では omit します。
pretty-only field だけは exception で、`include_pretty = false` の response では field ごと omit し、`null` を返しません。
MVP の pretty / display profile は `MachineDisplayProfileId = "npa.phase5.display.v1"` に固定し、
この profile id は `protocol_version` の一部として扱います。
`MachineExprView.pretty`、prompt の `target_pretty`、local `value_pretty` は、この fixed display profile で
core expr から決定的に生成した UTF-8 string だけです。
locale、terminal width、color、font、UI setting、source span、user alias、server config、HashMap iteration order に依存してはいけません。
同じ `protocol_version` で pretty renderer 出力を変える場合は protocol version を上げます。
MVP AI Profile の `MachineLocalView.display_name` は常に `MachineLocalView.machine_name` と同じ UTF-8 string です。
user-facing alias、source display name、pretty printer の name shortening を `display_name` に入れてはいけません。

`MachineExprView.machine` は Phase 5 MachineExprRenderer v1 で core `Expr` から決定的に生成します。
`MachineExprView.machine` と premise `statement.machine` は canonical display Machine Surface source であり、
Phase 3 Machine Surface grammar に従う canonical source ですが、その global name scope は tactic candidate の
validation / execution scope とは別です。
field 名の `machine` は「Machine Surface grammar で parse できる canonical display source」を意味し、
`/machine/tactics/run` の candidate `source` として受理されることを意味しません。
candidate として使えるかどうかの正本は、candidate validation / execution scope で再度
`RawMachineTerm` prepass と Phase 4 candidate canonicalization を通した結果だけです。
AI 向けに自動生成してよい executable candidate は、`/machine/search/for_goal` の
`suggested_candidates` のように candidate validation / execution scope で lightweight validation した payload だけです。
renderer は display render scope で exact match 解決できる fully-qualified declaration name、explicit universe argument、
Phase 4 `MachineLocalDecl.name` だけを使います。
notation、short name、implicit argument insertion / omission、pretty-only display name に依存した source を
出力してはいけません。
renderer version は `protocol_version` に含まれ、同じ `protocol_version` で renderer 出力を変えてはいけません。
この文書の `machine` / `source` / `statement.machine` 例は wire contract として normative です。
global name はすべて Phase 3 Machine Surface が exact match で解決する canonical declaration name で書きます。
`Eq`, `Nat`, `Nat.add_zero` が Phase 2 `ExportEntry.name` ならそれらは canonical name です。
`refl`, `zero`, `add_zero` のような suffix-only name や、`ExportEntry.name` に存在しない module-prefixed synthetic name は
Machine Surface canonical source では使いません。

renderer caller は expression ごとに `renderer_base_context` を明示してから `MachineExprRenderer v1` を呼びます。
これは BVar を goal local へ戻すための Phase 4 `MachineLocalDecl` prefix であり、常に full goal context とは限りません。
renderer caller は同時に `renderer_level_context` も明示します。
これは rendered level parameter name を解決するための ordered `MachineUniverseParamName` list です。

```text
renderer_base_context:
  goal target:
    the full MachineGoal.context
  MachineLocalView.context[i].ty:
    MachineGoal.context[0..i], excluding the local itself and all later locals
  MachineLocalView.context[i].value:
    MachineGoal.context[0..i], excluding the local itself and all later locals
  theorem index / search / prompt premise statement:
    []

renderer_level_context:
  goal target:
    session.root.universe_params in request order
  MachineLocalView.context[i].ty:
    session.root.universe_params in request order
  MachineLocalView.context[i].value:
    session.root.universe_params in request order
  theorem index / search / prompt premise statement:
    premise ExportEntry.universe_params in ExportEntry order
```

local declaration type/value が自分自身または後続 local を参照している場合、snapshot materialization では
`InvalidMachineProofState`、theorem index / premise construction では `InvalidTheoremIndex` です。
`MachineLocalView.depends_on` と local type/value の `MachineExprView.free_locals` も同じ
`renderer_base_context` に対して計算し、local declaration `i` の type/value では `LocalId < i` だけを返します。
`MachineLocalView.depends_on` は direct dependency だけです。
具体的には local declaration `i` について、`ty.free_locals` と、`value = Some(_)` の場合の
`value.free_locals` の union を goal context order で sort/dedup したものです。
他 local の `depends_on` を再帰的にたどる transitive closure ではありません。
local assumption の `depends_on` は `ty.free_locals` だけから作ります。
core `Level::Param` が `renderer_level_context` に存在しない、または `MachineUniverseParamName` として renderable でない場合は、
snapshot / prompt rendering では `InvalidMachineProofState`、theorem index / search rendering では
`InvalidTheoremIndex` です。

MachineExprRenderer v1 の token / parenthesis rule は次で固定します。

```text
MachineExprRenderer v1:
  output:
    UTF-8 string using ASCII syntax tokens only.
    Rendered global names must satisfy MachineSurfaceRenderableName; rendered local names must
    satisfy MachineLocalName. MVP renderer output has no escaping / quoting form.

  whitespace:
    exactly one ASCII space between adjacent keywords / identifiers / terms where needed.
    no leading whitespace, no trailing whitespace, no newline.

  precedence:
    atom = 100
    application = 80, left associative
    binder / let = 10
    top-level caller required precedence = 0

  parenthesize:
    render child expression recursively, then wrap in "(" + child + ")" when the child precedence
    is lower than the required precedence for that syntactic position.
    For positions that explicitly say "parenthesize binder/let child", also wrap when child
    precedence equals binder / let.

  global constant:
    render the display-scope fully-qualified name, with the explicit-head marker rule below.
    if universe argument list is non-empty, append ".{" + comma-separated rendered levels + "}".
    no spaces are inserted inside ".{...}" except those required inside prefix level syntax.

  level:
    zero as "0"; a closed numeral level made only from zero and succ renders as decimal without leading zero;
    param as MachineUniverseParamName;
    succ u as "succ " + level(u), only when u is not a closed numeral level;
    max u v as "max " + level(u) + " " + level(v);
    imax u v as "imax " + level(u) + " " + level(v).

  Sort:
    Sort 0 renders as "Prop".
    Sort (succ u) renders as "Type " + level(u).
    every other Sort u renders as "Sort " + level(u).

  level canonicalization:
    level(u) always returns canonical level source.
    The renderer must not output "succ 0", "succ 1", or "succ succ 0" when the level can be
    folded to a closed decimal numeral; it outputs "1", "2", and "2" respectively.
    The "succ " prefix form is used only when the operand is not a closed numeral level.
    max / imax operands are recursively rendered with this same canonical level rule.
    This canonical source rule performs only syntactic closed-numeral folding for chains of
    succ over zero. It does not perform algebraic normalization for max / imax; for example
    max 0 0 renders as "max 0 0", imax 0 0 renders as "imax 0 0", and
    succ (max 0 0) renders as "succ max 0 0", not "1".

  BVar:
    resolve against the innermost renderer binder stack first, then the renderer_base_context.
    base-context variables render as Phase 4 MachineLocalDecl.name.
    For BVar i under renderer binder_stack length b:
      if i < b, use binder_stack[b - 1 - i].
      otherwise let j = i - b.
      If j < renderer_base_context.len, use renderer_base_context[renderer_base_context.len - 1 - j].
      Otherwise the BVar is out of scope.
    unresolved / out-of-scope BVar is InvalidMachineProofState or InvalidTheoremIndex,
    depending on the renderer caller.

  App:
    flatten the left spine and render as head arg1 arg2 ...
    if the flattened head is a global constant, render that head with "@" exactly when the
    explicit-head marker rule below requires it.
    parenthesize the head if its precedence is lower than application.
    parenthesize an argument unless it is an atom.

  Lam:
    render binder_type with top-level required precedence; if binder_type is Lam / Pi / Let,
    parenthesize it inside the binder annotation.
    render body with top-level required precedence after pushing binder_name onto binder_stack.
    render "fun (" + binder_name + " : " + binder_type + ") => " + body.
    nested lambdas are rendered one binder at a time; grouping consecutive binders is forbidden.

  Pi:
    render binder_type with top-level required precedence; if binder_type is Lam / Pi / Let,
    parenthesize it inside the binder annotation.
    render body with top-level required precedence after pushing binder_name onto binder_stack.
    render "forall (" + binder_name + " : " + binder_type + "), " + body.
    nested Pis are rendered one binder at a time; grouping consecutive binders is forbidden.

  Let:
    render type with top-level required precedence; if type is Lam / Pi / Let, parenthesize it.
    render value with application required precedence and parenthesize binder/let child.
    render body with top-level required precedence after pushing binder_name onto binder_stack.
    render "let " + binder_name + " : " + type + " := " + value + " in " + body.
```

`binder_name` は core binder debug name をそのまま信用しません。
debug name が `MachineLocalName` で、現在の renderer binder stack、`renderer_base_context` の
`MachineLocalDecl.name`、display render scope にある全 `MachineGlobalRefView` の fully-qualified name の第 1 component の
いずれにも衝突しない場合だけ使います。
それ以外の場合は base name `x` を使い、衝突するたびに `x_0`, `x_1`, ... を decimal suffix 昇順で試して
最初の未使用 `MachineLocalName` を選びます。
ここで未使用とは、現在の renderer binder stack、`renderer_base_context` の `MachineLocalDecl.name`、
display render scope にある全 global name の第 1 component のいずれにも一致しないことです。
suffix 付き candidate が `MachineLocalName` grammar または 64-byte length limit を満たさなくなった時点で探索を止めます。
その時点までに未使用名が見つからない場合、base name を変えたり suffix を truncate したりせず、renderer failure として扱います。
この fresh-name rule は renderer-local であり、Phase 4 `MachineLocalDecl.name` や `state_fingerprint` を変更しません。

renderer の explicit-head marker rule は Phase 3 Machine Surface の `@` marker と同じ意味です。
flattened application head が global constant で、その declaration の Machine Surface callable interface に
implicit term binder があり、かつ rendered application spine がその implicit binder 位置へ明示 term argument を渡す場合だけ、
head の直前に `@` を出力します。
implicit term binder が存在しない、または implicit binder 位置へ term argument を渡していない global constant には `@` を出力しません。
この判定は display render scope の `MachineGlobalRefView` や server-local declaration registry から推測してはいけません。
renderer caller は `MachineSurfaceCallableInterfaceTable` から、Machine Surface elaborator が使う
implicit binder profile と同じ情報を renderer に渡さなければなりません。
implicit binder profile は、その declaration の term binder telescope を先頭から順に見た
`explicit | implicit` の固定 list です。
これは Phase 3 が `ImplicitArgumentRequired` を出す判定と同じ metadata であり、core certificate の信頼境界には入りません。
renderer は flattened application spine の term argument をこの profile の先頭から対応付け、
対応した profile prefix のうち 1 つでも `implicit` があれば `@` を付けます。
application spine が profile より長い場合、profile 外の term argument はこの `@` 判定には使いません。
その source が実際に elaboration できるかどうかは後続の renderer QA で検査します。
存在する `MachineSurfaceCallableInterfaceTable` entry の implicit profile を decode できない global constant を
`machine` source に出力する場合は renderer failure です。
`@` の有無は Phase 3 term-source canonical bytes の explicit-at marker に入るため、
renderer QA と prompt / snapshot fixture はこの rule に従って固定します。

`MachineSurfaceCallableInterfaceTable` は renderer と Phase 3 Machine Surface elaborator が共有する
非信頼 metadata artifact です。
この table は canonical core AST、certificate payload、kernel check の入力ではありませんが、
Phase 5 の `machine` / `statement.machine` source と renderer QA の結果を決定するため、
`machine_surface_callable_interface_table_hash` として `session_root_hash` に入れます。
同じ `protocol_version` と同じ `session_root_hash` では、この table の内容が同一でなければなりません。
Phase 5 renderer と Phase 3 `elaborate_machine_term_check` は、implicit binder 判定でこの table 以外の
source span、pretty metadata、server-local registry、package cache、UI 設定を読んではいけません。

Phase 5 から呼ぶ term elaboration の実効 context は、Phase 3 の `MachineTermElabContext` に
`MachineSurfaceCallableInterfaceTable` を加えたものです。
Phase 3 文書の struct shape と実装 API がまだこの field を直接持たない場合、Phase 5 adapter は
次と同等の wrapper を作って `elaborate_machine_term_check` へ渡さなければなりません。

```rust
enum MachineSurfaceCallableRef {
    Imported {
        module: ModuleName,
        name: FullyQualifiedName,
        export_hash: HashString,
        decl_interface_hash: HashString,
    },
    CurrentModule {
        module: ModuleName,
        name: FullyQualifiedName,
        source_index: u64,
        decl_interface_hash: HashString,
    },
    CurrentGenerated {
        module: ModuleName,
        name: FullyQualifiedName,
        parent_source_index: u64,
        decl_interface_hash: HashString,
    },
}

struct Phase5MachineTermElabContext {
    // phase3_context's internal global scope is caller-specific:
    // root/candidate/replay use the candidate validation scope,
    // renderer QA uses the display render scope.
    phase3_context: MachineTermElabContext,
    phase5_global_scope: Phase5MachineTermGlobalScope,
    callable_interface_table: MachineSurfaceCallableInterfaceTable,
}

struct Phase5MachineTermGlobalScope {
    entries: Vec<Phase5MachineTermGlobalScopeEntry>,
}

struct Phase5MachineTermGlobalScopeEntry {
    name: FullyQualifiedName,
    candidate_resolution: Option<MachineGlobalScopeEntry>,
    display_core_ref: Option<Phase5ResolvedDisplayCoreRef>,
    callable_ref: MachineSurfaceCallableRef,
}

struct Phase5ResolvedDisplayCoreRef {
    view: MachineGlobalRefView,
    owner_context: Phase5ResolvedDisplayCoreRefOwner,
    global_ref: GlobalRef,
}

enum Phase5ResolvedDisplayCoreRefOwner {
    CurrentSessionRootModule {
        module: ModuleName,
    },
    VerifiedImportedModule {
        owner_module: ModuleName,
        owner_export_hash: HashString,
    },
}
```

`MachineSurfaceCallableRef` は `MachineSurfaceCallableInterfaceEntry canonical bytes` の
`callable_ref` variant と 1 対 1 に対応します。
equality、sort、dedup、table lookup key はすべて下で定義する `callable_ref` canonical bytes で行い、
Rust enum discriminant、HashMap order、pretty name、source span を読んではいけません。
`MachineSurfaceCallableInterfaceTable` 内の `callable_ref` は一意でなければなりません。
同じ `callable_ref` canonical bytes を持つ entry が複数ある table は invalid です。
implicit profile が byte-for-byte に同一でも duplicate entry として拒否し、最初の entry を採用してはいけません。
session create stage 7 でこの重複を検出した場合、imported callable 由来なら `InvalidVerifiedImport`、
checked current / current generated callable 由来なら `InvalidCheckedCurrentDecl` です。
renderer QA 用に display render scope から追加される display-only callable は table entry を追加しないため、
この uniqueness rule の対象ではありません。
`Phase5MachineTermGlobalScopeEntry.callable_ref` は必須です。
global scope entry を作る時点で `MachineSurfaceCallableRef` に正規化できない global name は、
missing-entry rule ではなく、root / candidate / replay なら Phase 5 adapter invariant failure、
renderer なら renderer failure / renderer QA failure として扱います。

Phase 5 term elaboration では `phase5_global_scope` を唯一の authoritative exact-name map とします。
`phase3_context` 内部の global scope は Phase 3 API compatibility 用の projection であり、
`phase5_global_scope` に存在しない名前を追加してはいけません。
root theorem type、candidate validation、replay execution では、両方とも 5.2 / 6.2 の
candidate validation / execution scope から構築します。
この scope の各 entry は `candidate_resolution = some(...)` でなければならず、
`display_core_ref` は `none` でよいです。
renderer QA では `phase5_global_scope` を display render scope から構築し、display render scope にだけ存在する
private / transitive constant も入れます。
renderer QA で name resolution の候補として使う entry は `display_core_ref = some(...)` でなければなりません。
同じ name が candidate validation / execution scope でも解決可能な場合は、
同じ entry に `candidate_resolution = some(...)` も保持します。
display render scope にだけ存在する private / transitive constant は
`candidate_resolution = none` です。
display render scope は candidate scope の name を local/global collision check のためにも保持しますが、
表示対象 expression に出現せず `display_core_ref = none` の candidate-scope-only entry は
renderer QA の name resolution 候補にしてはいけません。
実装がそのような entry を `phase5_global_scope.entries` に保持する場合でも、
renderer QA resolver は `display_core_ref = none` の entry を UnknownName 相当として扱い、
owner-aware expression へ戻してはいけません。
既存の Phase 3 API が `MachineTermElabContext` 内部の global scope だけで名前解決する実装の場合、
Phase 5 adapter は display render scope を同じ exact-name map として projection するか、
Phase 3 resolver の前後で `phase5_global_scope` による解決を挟まなければなりません。
renderer QA 用に `phase3_context` 内部の global scope へ projection する名前は、
`display_core_ref = some(...)` の entry に限定します。
`display_core_ref = none` の candidate-scope-only entry は collision check には使えますが、
renderer QA の parse / elaborate で解決可能な global name として渡してはいけません。
renderer QA 用の global scope を candidate validation / execution scope で代用してはいけません。
どの実装形でも、最終的な global head resolution と callable-interface lookup は
`phase5_global_scope` の entry だけを入力にします。
candidate validation / replay / root theorem type check で `candidate_resolution = none` の entry が使われた場合は
UnknownName / machine_term_check です。
renderer QA では `candidate_resolution` が存在しても owner-aware 比較には使わず、
常に `display_core_ref` を使って global head を戻します。
これにより、direct import public name として candidate-admissible な declaration でも、
元の imported theorem statement 内では `VerifiedImportedModule` owner の `GlobalRef::Local` だった場合に
owner を失わず round-trip できます。
`Phase5ResolvedDisplayCoreRef.global_ref` は、display render scope の構築時に元の core expression と verified context から得た
exact core global reference であり、`owner_context` に対して解釈します。
`Phase5MachineTermGlobalScopeEntry` は `candidate_resolution` と `display_core_ref` の少なくとも一方を
持たなければなりません。両方が `none` の entry は invalid です。
`display_core_ref.view` は同じ entry の `name` と同じ fully-qualified name を持たなければなりません。
`display_core_ref` が存在する場合、`callable_ref` は `display_core_ref.view` を下の
display render scope / renderer QA normalization rule で正規化した結果と byte-for-byte に一致しなければなりません。
`candidate_resolution` が存在する場合、`callable_ref` は `candidate_resolution` の core `GlobalRef` を
root theorem type / candidate validation / replay execution normalization rule で正規化した結果とも一致しなければなりません。
`candidate_resolution` が存在する場合、その exact-name lookup key も `entry.name` と一致しなければなりません。
`candidate_resolution` と `display_core_ref` の両方が存在する entry は、同じ `entry.name` で解決され、
かつ両者が同じ `callable_ref` canonical bytes に正規化される場合だけ結合できます。
public generated constructor / recursor では、candidate 側が `GlobalRef::Imported` / `MachineSurfaceCallableRef::Imported`、
display 側が `MachineGlobalRefView::LocalGenerated` になり得ます。
この組み合わせは、generated name / generated decl_interface_hash / export_hash が一致し、
上の 2 つの normalization 結果が同じ `callable_ref` になる場合だけ同じ declaration とみなします。
これらの不一致は、root / candidate / replay では Phase 5 adapter invariant failure、
renderer では renderer failure / renderer QA failure です。
ここで `GlobalRef` は Phase 2 2.4 の core `GlobalRef` payload であり、standalone な fully-qualified reference ではありません。
`CurrentSessionRootModule` の場合、`Imported` は `session.imports`、`Local` / `LocalGenerated` は
`CurrentDeclIndexTable` / `CurrentGeneratedDeclTable` に対する参照です。
`VerifiedImportedModule` の場合、`Imported` はその verified module certificate の import table、
`Local` / `LocalGenerated` はその verified module の `VerifiedImportDeclIndexTable` /
`VerifiedImportGeneratedDeclTable` に対する参照です。
imported module 内の `GlobalRef::Local` を current session の local declaration として解釈してはいけません。
この参照は renderer output の文字列、pretty metadata、または `kernel_env` lookup から再推測してはいけません。
renderer QA の比較に使う display expression は、各 `Const(GlobalRef, levels)` に
`Phase5ResolvedDisplayCoreRefOwner` を添えた owner-aware core expression として扱います。
owner-aware expression は certificate payload、Phase 1 canonical Expr、Phase 4 state fingerprint には入れない
Phase 5 renderer QA 専用の派生 artifact です。
ordinary `Expr` の binder、application、level、local index、sort、let の構造はそのまま比較し、
global constant だけを `(owner_context, global_ref, levels)` で比較します。
`GlobalRef::Local` / `LocalGenerated` の owner が異なる場合は、fully-qualified name と hash が同じでも別 constant です。
renderer QA は parsed / elaborated source から得た global head を display render scope の entry に戻し、
entry の `Phase5ResolvedDisplayCoreRef` を使って owner-aware expression を構築してから、元の owner-aware expression と比較します。
display-only constant を current session の `GlobalRef` へ rewrite してから比較してはいけません。
candidate validation / replay / verify handoff では owner-aware expression を使わず、通常の Phase 1 / Phase 2 `Expr` だけを使います。

renderer QA の type checking、WHNF、conversion、application spine check は、通常の `kernel_env` global lookup ではなく
次の owner-aware declaration lookup を使います。
lookup result は checked declaration view と、その view 内の `GlobalRef::Local` / `LocalGenerated` を解釈する
owner context の pair です。
checked declaration view は interface type、declaration kind、universe parameters、body option、
reducibility / opacity、generated artifact の reconstructed interface を含み、Phase 1/2 の evaluator が
型検査、WHNF、conversion、δ reduction 可否判定に必要な情報をすべて持ちます。

```text
lookup_owner_aware_decl(owner_context, global_ref):
  owner_context = CurrentSessionRootModule:
    GlobalRef::Imported(import_index, name, decl_interface_hash):
      session.imports[import_index] の direct import public ExportEntry を引く。
      name / decl_interface_hash が一致しなければ renderer failure / renderer QA failure。
      direct_module / direct_export_hash は session.imports[import_index] の module / export_hash。
      result owner_context は VerifiedImportedModule { owner_module = direct_module,
      owner_export_hash = direct_export_hash }。
      ExportEntry が ordinary declaration に対応する場合、
      result declaration view はその ExportEntry に対応する verified checked declaration view。
      ExportEntry が generated constructor / recursor に対応する場合、
      result declaration view は VerifiedImportGeneratedDeclTable から得る reconstructed generated declaration view。
      ordinary と generated の両方に一致する、またはどちらにも一致しない場合は renderer failure / renderer QA failure。

    GlobalRef::Local(source_index):
      CurrentDeclIndexTable[source_index] を引く。
      result owner_context は CurrentSessionRootModule。
      result declaration view は checked current declaration view。

    GlobalRef::LocalGenerated(parent_source_index, generated_name):
      CurrentGeneratedDeclTable[(parent_source_index, generated_name)] を引く。
      result owner_context は CurrentSessionRootModule。
      result declaration view は generated constructor / recursor の reconstructed declaration view。

  owner_context = VerifiedImportedModule { owner_module, owner_export_hash }:
    GlobalRef::Imported(import_index, name, decl_interface_hash):
      owner verified module の certificate_import_table[import_index] を引き、
      dependency verified module { dependency_module, dependency_export_hash } の public ExportEntry を引く。
      name / decl_interface_hash が一致しなければ renderer failure / renderer QA failure。
      result owner_context は VerifiedImportedModule { owner_module = dependency_module,
      owner_export_hash = dependency_export_hash }。
      dependency ExportEntry が ordinary declaration に対応する場合、
      result declaration view はその dependency ExportEntry に対応する verified checked declaration view。
      dependency ExportEntry が generated constructor / recursor に対応する場合、
      result declaration view は dependency の VerifiedImportGeneratedDeclTable から得る
      reconstructed generated declaration view。
      ordinary と generated の両方に一致する、またはどちらにも一致しない場合は renderer failure / renderer QA failure。

    GlobalRef::Local(decl_index):
      owner verified module の VerifiedImportDeclIndexTable[decl_index] を引く。
      result owner_context は同じ VerifiedImportedModule。
      result declaration view はその verified checked declaration view。

    GlobalRef::LocalGenerated(parent_decl_index, generated_name):
      owner verified module の VerifiedImportGeneratedDeclTable[(parent_decl_index, generated_name)] を引く。
      result owner_context は同じ VerifiedImportedModule。
      result declaration view は generated constructor / recursor の reconstructed declaration view。
```

上の lookup に失敗した display render scope entry は renderer failure / renderer QA failure です。
owner-aware lookup は renderer QA 専用です。
candidate validation / replay / verify handoff は従来どおり current session の checked `kernel_env` を使い、
display render scope にだけ存在する private / transitive constant をその `kernel_env` に追加してはいけません。
`phase5_global_scope.entries` は exact name map であり、同じ `name` を持つ複数 entry は許しません。
duplicate name が必要になる display render scope は renderer failure / renderer QA failure として扱います。
root theorem type / candidate validation / replay execution の checked `kernel_env` は
同じ verified imports / checked current declarations から作った checked environment です。
renderer QA 実装が Phase 3 API compatibility のために同じ checked `kernel_env` を渡す場合でも、
display render scope の global name lookup、owner-aware declaration lookup、または display-only constant の補完に
その `kernel_env` を使ってはいけません。

Phase 3 elaborator は global head を `callable_ref` に正規化してこの table を引き、
entry がある場合はその `implicit_profile` だけを使います。
entry がない global constant は下の MVP missing-entry rule に従います。
`kernel_env`、declaration registry、pretty metadata、source span から implicit profile を復元してはいけません。
この正規化は table lookup 専用であり、kernel に渡す core `GlobalRef` の表現を変更しません。

```text
Phase5CallableRef normalization for callable-interface lookup:
  root theorem type / candidate validation / replay execution:
    Phase 2 GlobalRef::Imported(import_index, name, decl_interface_hash):
      session.imports[import_index] を引き、
      imported(module, name, export_hash, decl_interface_hash) にする。
      import_index が session.imports 範囲外、または name / decl_interface_hash が
      対応する direct public ExportEntry と一致しない場合は Phase 5 adapter invariant failure。

    Phase 5 source-indexed GlobalRef::Local(source_index):
      CurrentDeclIndexTable[source_index] を引き、
      current_module(root.module, checked signature.name, source_index, checked signature.decl_interface_hash) にする。
      table entry がない、または checked signature と一致しない場合は Phase 5 adapter invariant failure。

    Phase 5 source-indexed GlobalRef::LocalGenerated(parent_source_index, generated_name):
      CurrentGeneratedDeclTable[(parent_source_index, generated_name)] を引き、
      current_generated(root.module, generated_name, parent_source_index, parent decl_interface_hash) にする。
      table entry がない、または generated decl_interface_hash が parent decl_interface_hash と一致しない場合は
      Phase 5 adapter invariant failure。

  display render scope / renderer QA:
    MachineGlobalRefView::Imported { module, name, export_hash, decl_interface_hash, ... }:
      module / export_hash が MachineImportCertificateContext.verified_modules に存在しない場合は
      renderer failure / renderer QA failure。
      VerifiedModuleContextEntry(module, export_hash) の VerifiedImportDeclIndexTable に
      name / decl_interface_hash が一致する unique declaration が存在しなければ
      renderer failure / renderer QA failure。
      generated declaration を Imported variant として正規化してはいけません。
      imported(module, name, export_hash, decl_interface_hash) にする。

    MachineGlobalRefView::CurrentModule { module, name, source_index, decl_interface_hash }:
      module が session.root.module と一致しなければ renderer failure / renderer QA failure。
      CurrentDeclIndexTable[source_index] と照合し、
      table entry がない、または checked signature.name / checked signature.decl_interface_hash が
      name / decl_interface_hash と一致しない場合は renderer failure / renderer QA failure。
      current_module(module, name, source_index, decl_interface_hash) にする。

    MachineGlobalRefView::LocalGenerated { module, export_hash = some(export_hash), parent_name, name,
    parent_decl_interface_hash, decl_interface_hash, ... }:
      module / export_hash が MachineImportCertificateContext.verified_modules に存在しなければ
      renderer failure / renderer QA failure。
      VerifiedImportGeneratedDeclTable に parent_name / parent_decl_interface_hash / name /
      decl_interface_hash が一致する unique generated declaration が存在しなければ
      renderer failure / renderer QA failure。
      imported(module, name, export_hash, decl_interface_hash) にする。
      これは imported generated constructor / recursor を imported callable として扱う規則です。

    MachineGlobalRefView::LocalGenerated { module, export_hash = none, parent_name, name, parent_decl_interface_hash,
    decl_interface_hash, ... }:
      module が session.root.module と一致しなければ renderer failure / renderer QA failure。
      CurrentDeclIndexTable から parent_name / parent_decl_interface_hash に一致する unique parent source_index を見つけ、
      CurrentGeneratedDeclTable[(parent_source_index, name)] と照合し、
      current_generated(session.root.module, name, parent_source_index, decl_interface_hash) にする。
      unique parent がない、または generated table と一致しない場合は renderer failure / renderer QA failure。
```

display render scope の `MachineGlobalRefView::Imported` は ordinary declaration 専用です。
direct import public generated constructor / recursor は、candidate core では `GlobalRef::Imported` として現れても、
`MachineGlobalRefView` では `LocalGenerated { export_hash = some(_) }` に戻します。
その `LocalGenerated` を callable-interface lookup に使うときだけ、上の規則で
`MachineSurfaceCallableRef::Imported(module, generated_name, export_hash, generated_decl_interface_hash)` に正規化します。
したがって `MachineSurfaceCallableRef::Imported` は ordinary imported declaration と public imported generated artifact の
両方を含み得ますが、`MachineGlobalRefView::Imported` は generated artifact を含みません。

上の正規化に失敗した global head は missing-entry rule に進めてはいけません。
missing-entry rule は、正規化済み `callable_ref` が存在するが
`MachineSurfaceCallableInterfaceTable` に entry がない場合だけに適用します。

MVP の table は session create stage 7 で、direct import public ExportEntry、checked current declaration、
current generated constructor / recursor のうち Machine Surface で参照可能な callable だけを entry にします。
direct import public ExportEntry には、ordinary declaration だけでなく `ExportBlock` に公開された
generated constructor / recursor も含みます。
public generated ExportEntry の table key は `MachineSurfaceCallableRef::Imported` であり、
`ExportEntry.name` と `ExportEntry.decl_interface_hash` を使います。
`MachineSurfaceCallableRef::CurrentGenerated` は current module の generated constructor / recursor 専用であり、
imported generated callable には使いません。
direct import public generated callable が table から欠けている session は `InvalidVerifiedImport` です。
MVP v1 の implicit profile 生成元は、この文書で定義された canonical session input だけです。
Phase 2 certificate payload、Phase 4 `CheckedDeclSignature`、`CheckedCurrentDeclPackage` v5 は
implicit argument metadata を持たないため、MVP v1 の imported / current / current-generated callable は
すべて all-explicit profile として構築します。
all-explicit profile の長さは declaration interface type を reduction せず syntactic `Expr::Pi` head から
左から順に剥がして得た term binder 数です。
各要素は `explicit` で、binder name、source span、display metadata、pretty metadata は読みません。
syntactic `Expr::Pi` でない head に達した時点で telescope は終端します。
entry が存在しない global constant は、Phase 3 elaboration と renderer の `@` marker 判定の両方で、
implicit term binder を持たない empty all-explicit profile として扱います。
この missing-entry rule は display render scope にだけ存在する private / transitive constant の
renderer QA にも適用します。
root theorem type / candidate validation / replay execution の application 型検査、WHNF、conversion は
従来どおり current session の checked `kernel_env` の declaration type を使います。
renderer QA の application 型検査、WHNF、conversion は上の `lookup_owner_aware_decl` を使います。
missing-entry profile は implicit binder 判定にだけ使います。
MVP v1 の elaborator は、all-explicit profile の global head に付いた `@` marker を許可します。
この場合 `@` は Phase 3 の global exact-match marker と term-source canonical bytes の at-form marker としてだけ残り、
implicit binder 消費は発生しません。
したがって `@Eq.refl.{1} Nat n` のような既存の normative example は MVP v1 でも有効です。
一方、all-explicit profile では implicit term binder 不足による `ImplicitArgumentRequired` は発生しません。
MVP v1 で `ImplicitArgumentRequired` が返り得るのは、MissingExplicitUniverse など Phase 3 の別エラーを
11 の error mapping で同じ API error kind に写す場合だけです。
これは Phase 5 AI MVP v1 の callable table が implicit metadata を持たないことによる明示的な profile choice です。
Phase 3 Machine Surface の一般規則で `ImplicitArgumentRequired` になる例は、caller が implicit binder を含む
callable profile を渡した場合の挙動であり、Phase 5 AI MVP v1 の all-explicit profile ではその前提を満たしません。
non-MVP で `implicit` を含む profile を導入した場合は、`@` なしで implicit binder 位置へ term argument を渡そうとする source を
`ImplicitArgumentRequired` として拒否します。
MVP v1 で `implicit` を含む profile を server-local registry、元 source、package cache、UI 設定から
生成してはいけません。
将来 non-all-explicit profile や imported private / transitive display-only constant の profile を許す場合は、
metadata の request / package schema、canonical bytes、`session_root_hash` 入力を protocol version と一緒に更新します。

```text
MachineSurfaceCallableInterfaceTable canonical bytes:
  - tag "npa.phase5.machine-surface-callable-interface-table.v1"
  - entries in MachineSurfaceCallableInterfaceEntry canonical order

MachineSurfaceCallableInterfaceEntry canonical bytes:
  - callable_ref:
      imported:
        variant tag 0x00
        module canonical bytes
        fully-qualified name canonical bytes
        export_hash as HashString digest bytes
        decl_interface_hash as HashString digest bytes
      current_module:
        variant tag 0x01
        module canonical bytes
        fully-qualified name canonical bytes
        source_index as minimal unsigned LEB128 u64
        decl_interface_hash as HashString digest bytes
      current_generated:
        variant tag 0x02
        module canonical bytes
        generated fully-qualified name canonical bytes
        parent_source_index as minimal unsigned LEB128 u64
        decl_interface_hash as HashString digest bytes
  - implicit_profile list in telescope order:
      0x00 explicit
      0x01 implicit

MachineSurfaceCallableInterfaceEntry canonical order:
  lexicographic order of MachineSurfaceCallableInterfaceEntry canonical bytes

machine_surface_callable_interface_table_hash:
  sha256(MachineSurfaceCallableInterfaceTable canonical bytes)
```

MVP session create は、この table を deterministic に構築できない場合、
imported callable なら `InvalidVerifiedImport`、checked current / current generated callable なら
`InvalidCheckedCurrentDecl` として拒否します。
non-MVP で implicit profile を user-facing source metadata から受け取る場合も、受け取った metadata を信用せず、
この canonical table に固定したうえで、最終的な proof / certificate は core term と checker だけで検査します。

この renderer で生成した `machine` source は、まず context-free
`canonicalize_machine_term_source(source)` に成功しなければなりません。
この API が返す canonical bytes は Phase 3 parsed term AST bytes であり、UTF-8 source string ではありません。
したがって renderer output string と Phase 3 canonical bytes を byte-for-byte 比較してはいけません。
renderer source string の byte-for-byte 正規形を検査したい場合は、将来の explicit canonical-source-string API を
別途定義してから使います。
この context-free canonicalization は name resolution や型検査を意味しません。
さらに renderer QA として、同じ `renderer_base_context`、`renderer_level_context`、renderer binder scope、
display render scope でその renderer output source を parse / elaborate し、owner-aware core expression に戻します。
戻した owner-aware expression が元の expression から display render scope 構築時に得た owner-aware expression と一致しなければ
renderer bug として扱います。
通常の Phase 1 canonical bytes だけで比較すると、imported module 内の `GlobalRef::Local` と current session の
`GlobalRef::Local` のような owner-dependent reference を取り違えるため、renderer QA では使ってはいけません。
ただし owner-aware expression で owner を消去し、すべての global ref が current session context で意味を持つ
candidate-admissible expression だけを扱う caller は、追加の sanity check として Phase 1 canonical bytes を比較してよいです。
この追加比較は renderer QA の合否を置き換えません。
renderer QA の parse / elaborate は、display render scope と session の `MachineSurfaceCallableInterfaceTable` から作った
`Phase5MachineTermElabContext` を使います。
display render scope にだけ存在し table entry がない private / transitive constant は、上の missing-entry rule で
empty all-explicit profile として扱い、renderer QA 用に ad hoc な table entry を追加してはいけません。
renderer failure または renderer QA failure を外部 response に出す場合は、caller の materialization failure と同じ error kind / phase に写します。
`renderer_bug` のような独立 error kind は MVP には入れません。

```text
Renderer failure / QA failure mapping:
  session initial snapshot materialization:
    InvalidMachineProofState / session_create
  /machine/snapshots/get stored snapshot materialization:
    InvalidMachineProofState / snapshot_lookup
  /machine/tactics/run next snapshot materialization after logical success:
    InvalidMachineProofState / tactic_execution
  /machine/tactics/batch per-candidate next snapshot materialization after logical success:
    per-candidate invalid_machine_proof_state / tactic_execution
  /machine/replay final snapshot materialization:
    InvalidMachineProofState / replay_execution
  /machine/verify closed snapshot extraction:
    InvalidMachineProofState / snapshot_lookup
  /machine/search/for_goal theorem index or search-result statement rendering:
    InvalidTheoremIndex / theorem_search
  /machine/prompt_payload goal/context rendering:
    InvalidMachineProofState / snapshot_lookup
  /machine/prompt_payload selected premise statement rendering:
    InvalidTheoremIndex / theorem_search
  /machine/prompt_payload PromptRenderedContent assembly after goal and premise rendering succeeded:
    InvalidMachineProofState / prompt_payload
```

Phase 5 は Machine Surface global scope を用途で分けます。
candidate validation / execution scope は tactic raw term source、suggested candidate validation、replay step validation にだけ使う
exact-name map です。
session create の name collision checks が成功していることを前提に、次の順序ではなく集合として一意に解決します。

```text
Phase5MachineSurfaceCandidateScope:
  - direct import public ExportEntry.name:
      resolves to Phase 2 GlobalRef::Imported(import_index, name, decl_interface_hash)
  - checked current declaration signature.name:
      resolves to Phase 5 source-indexed GlobalRef::Local(source_index)
  - current generated constructor / recursor name in CurrentGeneratedDeclTable:
      resolves to Phase 5 source-indexed GlobalRef::LocalGenerated(parent_source_index, generated_name)
```

この candidate scope に同じ name が複数存在する session は作れません。
session が open な間、root theorem 自身はこの candidate scope に入りません。
raw Machine Surface term の elaboration はこの candidate scope を使って current checked declaration と current generated artifact を
core `GlobalRef` へ戻せなければなりません。
Phase 5 はこの candidate scope、current goal local context、root universe parameter context、同じ verified imports /
checked current declarations から作った checked kernel environment、session の
`MachineSurfaceCallableInterfaceTable` から Phase 5 term elaboration context を構築し、
`elaborate_machine_term_check` に渡します。
candidate validation / execution scope では、display render scope にだけ存在する constant の callable entry を
この context に追加してはいけません。

`Phase5MachineSurfaceGlobalRootSet` は local name collision check だけに使う集合です。
candidate scope に含まれる各 `MachineSurfaceRenderableName` を `.` で component 分割し、その第 1 component だけを
`MachineLocalName` として集め、canonical string order で sort/dedup します。
この集合には direct import public、checked current declaration、current generated constructor / recursor の
candidate scope 名だけを入れます。
root theorem 自身、transitive import の非 public 名、private dependency 名、theorem index / search / prompt の
display-render-only 名は入れません。
Phase 4 state construction / tactic execution が fresh local を作る場合、`Phase5MachineSurfaceGlobalRootSet` と
同名の `MachineLocalName` を選んではいけません。
stored state に含まれる local 名がこの集合に含まれる場合は `InvalidMachineProofState` です。
この集合は Phase 4 `reserved_local_names` に追加してはいけません。
Phase 4 `reserved_local_names` は Phase 4 の規則どおり proof state 内の binder 名から再計算され、
Phase 4 `state_fingerprint` に入ります。
`Phase5MachineSurfaceGlobalRootSet` は Phase 5 adapter が Phase 4 fresh-name generator / candidate validation に渡す
out-of-state forbidden local root set です。
この set 自体は Phase 4 state payload と `reserved_local_names` には保存しませんが、session input から決定的に再構築し、
同じ `session_root_hash` では同じ set でなければなりません。
Phase 4 実装 API がこの forbidden set を受け取れない場合、Phase 5 adapter は tactic success 後の
next snapshot materialization 前に全 open/assigned meta context と proof body binder 名を検査し、
衝突があれば success を返さず `InvalidMachineProofState` / `tactic_execution` とします。
`intro.name` が `Phase5MachineSurfaceGlobalRootSet` または current goal context 内の既存
`MachineLocalDecl.name` と同名の場合、wire grammar には成功しても state-dependent candidate validation failure です。
この失敗は Phase 4 `MachineTactic canonical bytes` 構築後に検出する post-canonical `InvalidCandidate` とし、
diagnostic phase は `candidate_validation`、`candidate_hash` は返します。
`intro.name` の `MachineLocalName` grammar 違反はこれとは別に pre-canonical `InvalidCandidate` であり、
`candidate_hash` を返しません。
`canonicalize_machine_term_source` は context-free に raw term source だけを受け取り、name resolution や型検査を行いません。
Phase 3 の term-level API は `verified_imports` だけを見て global name を解決してはいけません。
`CurrentGeneratedDeclTable` の generated artifact は raw term source では参照できますが、Phase 4 external
`TacticHead` には対応 variant がないため、`apply.head` / `rw.rule.head` / `SimpRuleRef` / family ref には使えません。
AI が current generated artifact を証明 head として使いたい場合は、`exact` の raw term など、Machine Surface term 内の
global ref として明示します。

Display render scope は `MachineExprView.machine` と premise `statement.machine` だけに使います。
display render scope は candidate scope に加えて、表示対象の core `Expr` に実際に現れる verified import / current-module
constant のうち、6.2 の `MachineGlobalRefView` へ正規化できるものを含めます。
これには transitive dependency module の public export、imported module 内の private declaration、
imported generated constructor / recursor のように `public_export = false` または `tactic_head_visible = false` になる
constant も含まれ得ます。
ただし renderer が `machine` / `statement.machine` に出力してよいのは、その fully-qualified name が
`MachineSurfaceRenderableName` であり、同じ display render scope 内で 1 つの `MachineGlobalRefView` にだけ対応する場合に
限ります。
さらに、その fully-qualified name の第 1 component は、現在の `renderer_base_context` と renderer binder stack の
どの `MachineLocalName` とも一致してはいけません。
これは display render scope にだけ存在する private / transitive constant にも適用します。
local / binder が global root を shadow する name を renderer が出力すると、escaping なしの Machine Surface source では
round-trip scope が一意にならないためです。
同じ name が複数の distinct `MachineGlobalRefView` に対応する、name が renderable でない、または第 1 component が
in-scope local / binder と衝突する場合は、
snapshot / prompt rendering では `InvalidMachineProofState`、theorem index / search rendering では
`InvalidTheoremIndex` とします。
display render scope にだけ存在する constant は prompt / debug 表示用であり、`/machine/tactics/run`、
`/machine/tactics/batch`、`/machine/replay` の candidate source として受理される保証はありません。
AI が display source を candidate に再利用する場合も、candidate validation / execution scope で改めて validation しなければなりません。
`suggested_candidates` は display-scope-only constant を raw candidate にコピーしてはいけません。
この節でいう display-only text / name は、pretty alias、short name、name shortening、user-facing alias のように
canonical fully-qualified declaration name ではない表示文字列を指します。
display render scope にだけ存在し candidate scope では解決できない private / transitive dependency であっても、
上の条件を満たす fully-qualified declaration name として出力する場合、それは canonical display source の一部です。
そのような display-scope-only constant を含む `machine` / `statement.machine` は parse / renderer QA 用には
display render scope で解釈しますが、candidate-admissible source であることは意味しません。

`MachineExprView.head` と `MachineExprView.constants` は display-only string ではなく `MachineGlobalRefView` で返します。
wire encoding では各 variant に `kind = "imported" | "current_module" | "local_generated"` を必ず入れます。
`Imported.public_export = true` は、その declaration が所属 module の `ExportBlock` で公開されていることだけを意味します。
`Imported.tactic_head_visible = true` は、その module が current session の direct import であり、Phase 4
`TacticHead::Imported { name, decl_interface_hash }` として解決できることを意味します。
AI が `MachineExprView.head` / `constants` から Phase 4 `TacticHead::Imported { name, decl_interface_hash }` を作ってよいのは
`tactic_head_visible = true` の item だけです。
`public_export = false` は verified import の kernel environment には存在するが `ExportBlock` では公開されない
dependency を表す表示 / fingerprint 用 metadata であり、tactic head、search premise、suggested candidate へ
変換してはいけません。
`CurrentModule` variant は current session の `checked_current_decls` 完全 prefix に含まれる checked declaration だけを表します。
valid な `CurrentModule` view は常に Phase 4
`TacticHead::CurrentModule { name, decl_interface_hash }` として解決可能でなければなりません。
そのため `CurrentModule` には `tactic_head_visible` field を持たせません。
`source_index` が `CurrentDeclIndexTable` に存在しない、`module` / `name` / `decl_interface_hash` が table entry と
checked declaration signature に一致しない、または name が Machine Surface source として renderable でない
`CurrentModule` view は
`InvalidMachineProofState` です。
`LocalGenerated.public_export = true` かつ `export_hash = some(_)` は、その generated constructor / recursor が
所属 module の imported `ExportBlock` に公開されていることを意味します。
AI は `LocalGenerated.tactic_head_visible = true` の場合だけ、`name` と `decl_interface_hash` から Phase 4
`TacticHead::Imported` を作ってよいです。
`LocalGenerated.public_export = false`、`export_hash = none`、または `tactic_head_visible = false` の場合は
`MachineExprView` / fingerprint / raw term round-trip 用の参照としてだけ扱い、Phase 4 external tactic head にしてはいけません。
valid な `LocalGenerated` field combination は次だけです。

```text
LocalGenerated field invariants:
  imported generated, public in its owner module:
    export_hash = some(owner_export_hash)
    public_export = true
    tactic_head_visible =
      direct_public_tactic_head_visible(module, name, owner_export_hash, decl_interface_hash)

  imported generated, private in its owner module:
    export_hash = some(owner_export_hash)
    public_export = false
    tactic_head_visible = false

  current generated:
    export_hash = none
    public_export = false
    tactic_head_visible = false
```

`export_hash = none` で `public_export = true`、`export_hash = none` で `tactic_head_visible = true`、
または `public_export = false` で `tactic_head_visible = true` の組み合わせは常に invalid です。
`export_hash = some(_)` かつ `tactic_head_visible = true` の場合は、上の
`direct_public_tactic_head_visible` predicate が true でなければなりません。
この invariant に反する stored snapshot は `InvalidMachineProofState`、theorem index construction 中なら
`InvalidTheoremIndex` です。
`MachineExprView.constants` は core expr 内に出現する global ref を `MachineGlobalRefView` canonical order に
sort/dedup したものです。
`MachineGlobalRefView canonical bytes` は次です。

```text
MachineGlobalRefView canonical bytes:
  - tag "npa.phase5.global-ref-view.v2"
  - variant tag:
      0x00 imported
      0x01 current_module
      0x02 local_generated
  - imported:
      module canonical bytes
      fully-qualified name canonical bytes
      export_hash as HashString digest bytes
      decl_interface_hash as HashString digest bytes
      public_export as 0x00 | 0x01
      tactic_head_visible as 0x00 | 0x01
  - current_module:
      module canonical bytes
      fully-qualified name canonical bytes
      decl_interface_hash as HashString digest bytes
      source_index as minimal unsigned LEB128 u64
  - local_generated:
      module canonical bytes
      export_hash option:
        0x00 none
        0x01 some + HashString digest bytes
      parent_name fully-qualified canonical bytes
      name fully-qualified canonical bytes
      parent_decl_interface_hash as HashString digest bytes
      decl_interface_hash as HashString digest bytes
      public_export as 0x00 | 0x01
      tactic_head_visible as 0x00 | 0x01
```

`HashString digest bytes` は `sha256:` prefix を除いた 32-byte digest です。
MVP では `HashString` の algorithm は `sha256` だけなので、algorithm name はこの canonical bytes に重複して入れません。
`module canonical bytes` と `fully-qualified name canonical bytes` は 5.1 の `Phase5Name canonical bytes` です。
`MachineGlobalRefView` canonical order は `MachineGlobalRefView canonical bytes` の辞書順です。
`MachineExprView.free_locals` と `MachineLocalView.depends_on` は goal context order に sort/dedup します。
`MachineExprView.size` は Phase 1 core `Expr` を tree として見た occurrence count です。
hash-cons / arena / DAG sharing は無視し、同じ subexpression が複数箇所から参照される場合も出現箇所ごとに数えます。
各 `Expr` constructor occurrence を 1 node とし、binder type/body、let type/value/body、
application function/argument はそれぞれ再帰的に数えます。
`Sort` や universe arguments を持つ global constant は `Expr` constructor としては 1 node だけ数え、
Phase 1 `Level` の内部 node、name component、source span、cached hash、type annotation cache、pretty-only wrapper は
`MachineExprView.size` に数えません。
count が `u32::MAX` を超える expression は materialization failure です。
snapshot / prompt rendering では `InvalidMachineProofState`、theorem index / search rendering では
`InvalidTheoremIndex` とします。

`machine_name` は Machine Surface term で使う canonical local name です。
同じ context 内で一意でなければならず、外部から渡された display name をそのまま信用しません。
Phase 5 は local alias を後段で書き換えません。`machine_name` は Phase 4 proof state の
`MachineLocalDecl.name` と完全一致し、`/machine/tactics/run` と `/machine/replay` は candidate を Phase 4 へ
無変換で渡します。
生成規則は Phase 4 state construction / tactic execution の時点で deterministic に適用し、生成された
`local_id -> machine_name` map から `local_name_map_hash` を計算します。
`local_name_map_hash` は Phase 5 view / query integrity 用の派生 hash であり、Phase 4 の `context_hash` と
`state_fingerprint` には追加しません。
Phase 5 `MachineGoalFingerprint` には下で定義する明示入力として含めます。
Phase 4 の state fingerprint は Phase 4 AI の定義をそのまま使います。
`machine_name` は Phase 4 `MachineLocalDecl.name` と同一なので、同じ Phase 4 `state_fingerprint` を持つ
snapshot は必ず同じ Machine Surface local name を返します。
`/machine/tactics/run`、`/machine/tactics/batch`、`/machine/replay` の state identity check は
`state_fingerprint` だけで行い、`local_name_map_hash` を request に要求しません。

```text
machine_name:
  value = Phase 4 MachineLocalDecl.name
  Phase 5 never renames or aliases it
```

Phase 4 state construction / tactic execution が fresh local を作る時点で deterministic fresh-name rule を適用し、
保存される `MachineLocalDecl.name` を一意な Machine Surface local identifier にします。
Phase 5 は表示時に別名を作らず、candidate 内の local 名も rewrite しません。
state に保存された local 名が Machine Surface identifier として無効、`Phase5MachineSurfaceGlobalRootSet` と衝突、
または同一 context 内で重複している場合は `InvalidMachineProofState` として拒否します。
`machine_name` は user-facing `display_name` から生成しません。表示名だけを変えても `machine_name` は変わりません。
`display_name` は人間向け表示用で、`state_fingerprint`, `candidate_hash`, `Machine Surface` term には入りません。
AI が tactic term を返すときは `machine_name` だけを参照します。

`local_name_map_hash` と `goal_fingerprint` は Phase 5 view / query 用の hash です。
どちらも Phase 4 `state_fingerprint` の入力ではありません。

```text
LocalNameMap canonical bytes:
  - tag "npa.phase5.local-name-map.v1"
  - entries in context order:
      local_id canonical bytes
      binder_index as minimal unsigned LEB128 u32
      machine_name as Phase5 UTF-8 string primitive bytes

local_name_map_hash:
  sha256(LocalNameMap canonical bytes)

MachineGoalFingerprint canonical bytes:
  - tag "npa.phase5.goal-fingerprint.v1"
  - goal_id canonical bytes
  - meta_id canonical bytes
  - context_hash
  - target_hash
  - local_name_map_hash

goal_fingerprint:
  sha256(MachineGoalFingerprint canonical bytes)
```

`allowed_tactics` の canonical order は `intro`, `exact`, `apply`, `rw`, `simp-lite`, `induction-nat` です。
`allowed_tactics` は Phase 5 protocol が受け付け、現在の session の resolved tactic environment で primitive が利用可能な
tactic constructor の deterministic capability hint であり、goal に対する成功可能性の予測ではありません。
MVP では同じ session の全 open goal に同じ canonical subset を返します。
`intro` / `exact` / `apply` は常に含めます。
`rw` / `simp-lite` は `MachineTacticEnv.eq_family = Some(_)` の場合だけ含めます。
`induction-nat` は `MachineTacticEnv.nat_family = Some(_)` の場合だけ含めます。
`allowed_tactics` は `goal_fingerprint`、`state_fingerprint`、search cache key には入りません。
unknown tactic kind と duplicate tactic kind は `InvalidMachineProofState` です。

## 6.3 Snapshot 取得

```json
POST /machine/snapshots/get
{
  "session_id": "msess_001",
  "snapshot_id": "mst_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  "state_fingerprint": "sha256:...",
  "include_pretty": false
}
```

`/machine/snapshots/get` request object は次の field だけを持ちます。

```text
required:
  session_id
  snapshot_id
  state_fingerprint
  include_pretty
optional:
  none
```

top-level unknown field、duplicate key、required field omitted、`null`、`session_id` / `snapshot_id` /
`state_fingerprint` の non-string、invalid `SessionId` grammar、invalid `SnapshotId` grammar、invalid `HashString`、
`include_pretty` の non-bool は `InvalidSnapshotRequest` として拒否します。
request envelope validation 後、`session_id` が存在しない場合は `UnknownSession` です。
その後の snapshot lookup は 6.1 の session-scoped lookup order に従い、missing entry は `UnknownSnapshot`、
stored snapshot self-check 失敗は `InvalidMachineProofState`、stored snapshot の `state_fingerprint` と
request の `state_fingerprint` が一致しない場合は `StateFingerprintMismatch` です。

レスポンス:

```json
{
  "status": "ok",
  "snapshot": {
    "session_id": "msess_001",
    "snapshot_id": "mst_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "state_fingerprint": "sha256:...",
    "tactic_options_fingerprint": "sha256:...",
    "open_goals": ["g0"],
    "goals": [
      {
        "goal_id": "g0",
        "meta_id": "m0",
        "context_hash": "sha256:...",
        "local_name_map_hash": "sha256:...",
        "context": [],
        "target": {
          "core_hash": "sha256:...",
          "head": null,
          "constants": [
            {
              "kind": "imported",
              "module": "Std.Init",
              "name": "Eq",
              "export_hash": "sha256:...",
              "decl_interface_hash": "sha256:...",
              "public_export": true,
              "tactic_head_visible": true
            },
            {
              "kind": "imported",
              "module": "Std.Nat.Basic",
              "name": "Nat",
              "export_hash": "sha256:...",
              "decl_interface_hash": "sha256:...",
              "public_export": true,
              "tactic_head_visible": true
            }
          ],
          "free_locals": [],
          "size": 8,
          "machine": "forall (n : Nat), Eq.{1} Nat n n"
        },
        "target_hash": "sha256:...",
        "goal_fingerprint": "sha256:...",
        "allowed_tactics": ["intro", "exact", "apply"]
      }
    ],
    "proof_skeleton_hash": "sha256:..."
  }
}
```

`/machine/snapshots/get` は summary ではなく、`MachineProofSnapshot` の wire payload を返します。
`session_id`、`snapshot_id`、`state_fingerprint`、`tactic_options_fingerprint`、`open_goals`、
`goals`、`proof_skeleton_hash` は必須 field です。
`include_pretty` は `goals[*].context[*].ty.pretty`、`goals[*].context[*].value.pretty`、
`goals[*].target.pretty` のような nested display-only field の有無だけを変え、
snapshot identity / fingerprint / proof skeleton の field は省略しません。

## 6.4 Snapshot store lifetime / quota

MVP の session snapshot store は、session が open の間は content-addressed append-only store として扱います。
`/machine/sessions` が success response で返した initial snapshot、および
`/machine/tactics/run`、`/machine/tactics/batch`、`/machine/replay` が success response で返した snapshot は、
同じ `session_id`、`snapshot_id`、`state_fingerprint` で `/machine/snapshots/get` できなければなりません。
server は session が open の間、response 済み snapshot を silently evict してはいけません。
snapshot cleanup は `DELETE /machine/sessions/{id}`、server process restart、または transport/session layer の明示的な
session expiration だけで行います。
session expiration は Machine API deterministic diagnostic ではなく、以後の request は `UnknownSession` です。

snapshot store が保存する正本は response JSON ではありません。
MVP の snapshot entry は次の 2 つを区別します。

```text
Stored snapshot entry:
  executable_state_payload:
    Phase 4 MachineProofState を lossless に保持する implementation-local payload。
    run / batch / replay / verify は必ずこの payload から Phase 4 state を復元して実行する。
  materialized_view_payload:
    下で定義する StoredSnapshotView canonical bytes。
    Phase 4 state、proof body、metavariable assignment、MachineProofDelta の復元には使わない。
    /machine/snapshots/get、search、prompt の response projection、snapshot reuse、
    および 6.1 の stored snapshot entry self-check で view integrity を照合するために使う。
```

`executable_state_payload` は Phase 4 `MachineProofState` の full proof state を保持しなければなりません。
少なくとも `ProofRoot.body`、open / assigned metavariable store、各 metavariable の context / target / assignment、
goal-to-meta mapping、open goal order、fresh-name state、Phase 4 state validation に必要な derived fields を
失わずに復元できる必要があります。
この payload は Rust object、arena、private binary serialization など implementation-local 形式でよいですが、
復元後に Phase 4 rule で `state_fingerprint`、`proof_skeleton_hash`、各 goal の `context_hash` / `target_hash` を
再計算して stored view と照合できなければなりません。
`StoredSnapshotView canonical bytes` だけから Phase 4 `MachineProofState`、proof body、metavariable assignment、
または `MachineProofDelta` を復元してはいけません。
executable payload が欠けている、または復元 state と view projection が矛盾する stored snapshot は
`InvalidMachineProofState` です。
run / batch / replay / verify は logical execution の入力として `materialized_view_payload` を使ってはいけません。
ただし、それらの endpoint も 6.1 の stored snapshot entry self-check では `materialized_view_payload` を読み、
復元した executable state から再 materialize した `StoredSnapshotView canonical bytes` と byte-for-byte に照合します。

`materialized_view_payload` は pretty-only field を含みません。
`/machine/snapshots/get` の `include_pretty = false` response は `materialized_view_payload` から作る projection です。
`include_pretty = true` response は、同じ projection に対して、復元済み executable state と 6.2 の fixed display profile から
deterministic renderer で再計算した pretty-only fields を response-only field として追加します。
pretty-only fields は `StoredSnapshotView canonical bytes`、`state_fingerprint`、`proof_skeleton_hash`、
snapshot reuse check、replay plan、certificate payload には入りません。
したがって同じ stored snapshot から `include_pretty = false` と `include_pretty = true` の response JSON が
byte-for-byte に異なることは正常です。
下の byte-for-byte 一致条件は `StoredSnapshotView canonical bytes` に対する条件であり、
HTTP / JSON response body や executable state の private serialization に対する条件ではありません。

`StoredSnapshotView canonical bytes` は、`include_pretty = false` の `MachineProofSnapshot` materialization を
JSON ではなく 5.1 の Phase 5 canonical primitive encoding で encode したものです。
実装が view payload を cache せず毎回 executable state から再 materialize する場合でも、次の canonical bytes を
計算できなければなりません。

```text
StoredSnapshotView canonical bytes:
  - tag "npa.phase5.stored-snapshot-view.v1"
  - state_fingerprint
  - tactic_options_fingerprint
  - open_goals in response order
  - goals in open_goals order:
      goal_id
      meta_id
      context_hash
      local_name_map_hash
      context list in context order:
        local_id
        machine_name
        display_name
        ty as StoredExprView canonical bytes
        value option StoredExprView canonical bytes
        depends_on in context order
        binder_index
      target as StoredExprView canonical bytes
      target_hash
      goal_fingerprint
      allowed_tactics in MachineTacticKind canonical order
  - proof_skeleton_hash

StoredExprView canonical bytes:
  - tag "npa.phase5.stored-expr-view.v1"
  - core_hash
  - head option MachineGlobalRefView canonical bytes
  - constants in MachineGlobalRefView canonical order
  - free_locals in context order
  - size as minimal unsigned LEB128 u32
  - machine string
```

StoredSnapshotView canonical bytes には `session_id`、pretty-only field、source span、diagnostic text、
`proof_delta_hash` や predecessor snapshot id のような path-dependent lineage metadata、server-local store key、
wall-clock timestamp、cache hit metadata を入れてはいけません。
`snapshot_id` は `state_fingerprint` だけから導出されるため、MVP の `MachineProofSnapshot` は直前 delta を表す
`prior_delta_hash` field を持ちません。
delta chain の監査に必要な `proof_delta_hash` は `/machine/tactics/run`、`/machine/tactics/batch`、
または `/machine/replay` の plan / success response から取得し、`/machine/snapshots/get` の state payload に
後付けしてはいけません。

同じ `snapshot_id` は full `state_fingerprint` から導出されるため、既存 snapshot と同じ state を再生成した場合は
次をすべて確認して既存 entry を再利用してよいです。

```text
Snapshot entry reuse check:
  - newly generated executable_state_payload recomputes to the generated state_fingerprint
  - StoredSnapshotView canonical bytes newly materialized from the newly generated executable_state_payload
    have state_fingerprint equal to the generated state_fingerprint
  - existing executable_state_payload is present and recomputes to the same generated state_fingerprint
  - existing materialized_view_payload.state_fingerprint equals the same generated state_fingerprint
  - StoredSnapshotView canonical bytes newly materialized from the existing executable_state_payload are
    byte-for-byte equal to the existing materialized_view_payload bytes
  - StoredSnapshotView canonical bytes newly materialized from the newly generated executable_state_payload are
    byte-for-byte equal to the existing materialized_view_payload bytes
```

既存 entry の `executable_state_payload` が欠けている、復元できない、または既存 `materialized_view_payload` と矛盾する場合、
新しく生成した payload で上書きしてはいけません。
同じ `snapshot_id` に異なる materialized payload が存在する場合、または既存 executable payload と既存 materialized payload が
矛盾する場合は store corruption として
`InvalidMachineProofState` です。

server が `max snapshots per session` などの quota を持つ場合、その quota は trusted proof payload、cache key、
replay plan、diagnostic hash には入りません。
候補実行が論理的には success でも snapshot store quota / resource guard により next snapshot を保存できない場合、
server は success を返してはいけません。
その場合だけ retryable scheduler artifact として扱います。
next snapshot の materialization、`StoredSnapshotView canonical bytes` の consistency check、または既存 entry との
byte-for-byte 照合が失敗した場合は quota stop ではなく、各 endpoint で明記する `InvalidMachineProofState` です。

```text
snapshot store quota stop uses the same response shape as each endpoint's scheduler stop:
  /machine/tactics/run:
    status = "scheduler_stopped"
    include previous_state_fingerprint and deterministic_budget_hash exactly as 7.1
    scheduler_artifact.kind = "resource_limit_exceeded"
    scheduler_artifact.scope = "candidate"
    scheduler_artifact.retryable = true

  /machine/tactics/batch:
    status = "partial_resource_limit"
    include previous_state_fingerprint, deterministic_budget_hash, completed_prefix_len,
    and the completed prefix results exactly as 7.2
    completed_prefix_len = number of result items whose snapshots, if any, were already stored
    scheduler_artifact.kind = "resource_limit_exceeded"
    scheduler_artifact.scope = "batch"
    scheduler_artifact.retryable = true

  /machine/replay:
    status = "scheduler_stopped"
    use the replay scheduler stop response shape from 12.1
    scheduler_artifact.kind = "resource_limit_exceeded"
    scheduler_artifact.scope = "replay"
    scheduler_artifact.retryable = true
```

quota stop で確定しなかった candidate / replay final snapshot は response、deterministic cache、replay plan に入れません。
Batch では、snapshot store quota により stop した candidate は `results` に含めず、success / failure count にも数えません。

---

# 7. Machine Tactic Execution API

## 7.0 MachineTacticCandidate wire schema

`candidate` field は Phase 4 AI の external `MachineTacticCandidate` wire schema と同じ raw payload だけを受け取ります。
`candidate_hash`、`deterministic_budget_hash`、`proof_delta_hash`、`next_state_fingerprint`、`diagnostic_hash`、
`checked_tactic`、`checked_term`、`score`、`metadata` など、7.0 の variant schema に定義されていない field は
`candidate` の内側に入れてはいけません。
これらは特別な metadata として扱わず、すべて unknown field として拒否します。
Raw Machine term は `source` だけを持ちます。

```json
{
  "kind": "exact",
  "term": {
    "source": "@Eq.refl.{1} Nat n"
  }
}
```

`RawMachineTerm` wire object は次の field だけを持ちます。

```text
RawMachineTerm:
  required fields = source
  optional fields = none
```

`source` は JSON string 必須です。
`RawMachineTerm` の non-object、duplicate key、unknown field、`source` omitted、`source = null`、
`source` non-string は `InvalidCandidate` です。
空文字列や Machine Surface として parse できない文字列は wire shape ではなく Phase 3 Machine Surface
parse/check の失敗として扱い、`MachineTermParseError` または `MachineTermElaborationError` に写します。
ただし Phase 3 `canonicalize_machine_term_source(source)` が成功するまでは Phase 4 `MachineTactic canonical bytes` を
構築できないため、この段階の term-source parse / canonicalization failure では `candidate_hash` を返しません。
この failure の diagnostic phase は 11 の matrix に従い、`MachineTermParseError` は `machine_term_parse`、
`MachineTermElaborationError` は `machine_term_check` とします。
Phase 5 `RawMachineTerm` prepass の parse / canonicalization failure を
`InvalidCandidate` や `candidate_validation` phase に写してはいけません。
この規則は 11 の Phase 4 `InvalidMachineTermSource` mapping より優先します。
Phase 5 endpoint adapter は、評価対象になった candidate を Phase 4
`validate_machine_tactic_candidate` に渡す前に、その candidate に含まれるすべての `RawMachineTerm.source` へ
Phase 3 `canonicalize_machine_term_source(source)` を直接実行しなければなりません。
この prepass は context-free source canonicalization だけを行い、name resolution、type checking、WHNF、
conversion、tactic execution は実行しません。
以降、この文書で `Phase 5 RawMachineTerm prepass` と書く場合は、この context-free
`canonicalize_machine_term_source(source)` 呼び出しだけを指します。
`Phase 4 candidate canonicalization` は Phase 4 `validate_machine_tactic_candidate` が
`MachineTactic canonical bytes` を構築し、そこから `candidate_hash` を計算できる状態を指します。
`RawMachineTerm` prepass 成功だけを `candidate_hash` inclusion threshold として扱ってはいけません。
prepass が失敗した場合、Phase 5 はその Phase 3 diagnostic を 11 の Phase 3 mapping に従って
`MachineTermParseError` または `MachineTermElaborationError` に写し、`candidate_hash` を返しません。
prepass 成功後も、Phase 4 が Phase 4 `MachineTactic canonical bytes` と `candidate_hash` の唯一の作成元です。
Phase 5 は prepass の canonical bytes だけから `candidate_hash` を作ってはいけません。
prepass 成功後に Phase 4 から `InvalidMachineTermSource` が返った場合は、source diagnostic ではなく
adapter invariant failure として扱い、run / batch では `InvalidMachineProofState`、phase `candidate_validation` に写し、
`candidate_hash` は返しません。
`candidate_hash` を返せる error は、candidate schema validation、Phase 5 `RawMachineTerm` prepass、
Phase 4 candidate canonicalization がすべて成功し、Phase 4 `MachineTactic canonical bytes` を構築できた後の
post-canonical `InvalidCandidate`、term elaboration / type check / tactic execution failure だけです。
`deterministic_budget.max_expr_nodes` はこの pre-canonical Phase 5 `RawMachineTerm` prepass の
parse / canonicalization には適用しません。
この段階で巨大 payload を止める必要がある場合は endpoint 契約を持たない transport / resource layer guard とし、
`MachineApiDiagnostic`、`diagnostic_hash`、`candidate_hash`、`deterministic_budget_hash` を生成してはいけません。
Phase 4 `TacticFuelExhausted { kind: ExprNode }` を `TooLargeTerm` に写せるのは、Phase 4 candidate canonicalization が成功して
`candidate_hash` を計算できた後の term elaboration / type check / tactic execution 中に、
proof / core expression generation budget として `max_expr_nodes` が消費された場合だけです。

MVP の variant schema:

```text
Exact:
  { "kind": "exact", "term": RawMachineTerm }

Intro:
  { "kind": "intro", "name": String }

Apply:
  { "kind": "apply", "head": TacticHead, "universe_args": [Level], "args": [CandidateApplyArg] }

Rw:
  {
    "kind": "rw",
    "rule": { "head": TacticHead, "universe_args": [Level], "args": [CandidateApplyArg] },
    "direction": "forward" | "backward",
    "site": "eq_target_left" | "eq_target_right"
  }

SimpLite:
  { "kind": "simp-lite", "rules": [SimpRuleRef] }

InductionNat:
  { "kind": "induction-nat", "local_name": String }
```

`Intro.name` と `InductionNat.local_name` は `MachineLocalName` でなければなりません。
`Intro.name` が valid `MachineLocalName` だが current goal context または `Phase5MachineSurfaceGlobalRootSet` と衝突する場合は、
6.2 の state-dependent candidate validation rule に従う post-canonical `InvalidCandidate` です。
`Apply.universe_args`、`Apply.args`、`Rw.rule.universe_args`、`Rw.rule.args`、`SimpLite.rules` は array 必須で、
omitted / `null` / non-array は `InvalidCandidate` です。
`kind` は JSON string 必須で、MVP allowed values は `intro`, `exact`, `apply`, `rw`, `simp-lite`,
`induction-nat` です。
candidate variant object の unknown field、duplicate key、required field omitted、`null` は
`InvalidCandidate` です。
ただし field ごとに nullable と明記した場合だけ例外とします。
MVP の candidate schema で nullable な field は `CandidateApplyArg.subgoal.name_hint` だけです。

`TacticHead` は Phase 4 の `Imported` / `CurrentModule` / `Local` に対応します。
Phase 5 tactic candidate 内では、`MachineGlobalRefView` とは違い `kind` field を使いません。
Phase 4 external schema と同じ singleton-object variant だけを受け付けます。

```text
Imported:
  { "imported": { "name": FullyQualifiedName, "decl_interface_hash": HashString } }

CurrentModule:
  { "current_module": { "name": FullyQualifiedName, "decl_interface_hash": HashString } }

Local:
  { "local": { "name": MachineLocalName } }
```

`TacticHead` object は top-level key をちょうど 1 つだけ持つ singleton object でなければなりません。
`imported` / `current_module` の inner object は `name` と `decl_interface_hash` だけを持ち、
`local` の inner object は `name` だけを持ちます。
inner object の duplicate key、unknown field、required field omitted、`null` は `InvalidCandidate` です。
`TacticHead.imported.name` と `TacticHead.current_module.name` は `FullyQualifiedName` かつ
`MachineSurfaceRenderableName` でなければならず、広い `FullyQualifiedName` だけを満たす名前は candidate schema violation です。
`imported` / `current_module` の `name` non-string、invalid `FullyQualifiedName` grammar、
non-renderable `MachineSurfaceRenderableName`、`decl_interface_hash` non-string または invalid `HashString` は
`InvalidCandidate` です。
`local.name` は `MachineLocalName` でなければならず、non-string または invalid `MachineLocalName` は
`InvalidCandidate` です。

`CandidateApplyArg` は `{"mode": "term", "term": RawMachineTerm}`、
`{"mode": "subgoal", "name_hint": String | null}`、`{"mode": "infer_from_target"}` のいずれかです。
`kind` field を使った `TacticHead` / `CandidateApplyArg` は Phase 4 external schema と一致しないため
`InvalidCandidate` として拒否します。
`SimpRuleRef` は `{"name": FullyQualifiedName, "decl_interface_hash": HashString, "direction": "forward" | "backward"}`
です。
`CandidateApplyArg` object は `mode` を必須 string とし、`mode` ごとの field set は次に固定します。

```text
CandidateApplyArg:
  term:
    required fields = mode, term
    forbidden fields = name_hint
  subgoal:
    required fields = mode, name_hint
    forbidden fields = term
  infer_from_target:
    required fields = mode
    forbidden fields = term, name_hint
```

`mode` が未知 string、`term` の value が `RawMachineTerm` でない、`subgoal.name_hint` が `null` でも
`MachineLocalName` string でもない、または object に余分な field がある場合は `InvalidCandidate` です。
`term` mode と `infer_from_target` mode に `name_hint = null` を置くことは forbidden field なので
`InvalidCandidate` です。
`SimpRuleRef.name` も `FullyQualifiedName` かつ `MachineSurfaceRenderableName` でなければなりません。
`SimpRuleRef` object の duplicate key、unknown field、required field omitted、`null`、invalid name/hash/direction も
`InvalidCandidate` です。

`Level` は JSON string で、Phase 3 Machine Surface の level grammar を次の canonical source subset に固定します。
JSON number や object は許しません。

```text
Level wire:
  natural:
    "0" or non-zero ASCII decimal without leading zero
  param:
    MachineUniverseParamName
  succ:
    "succ " + Level wire source
  max:
    "max " + Level wire source + " " + Level wire source
  imax:
    "imax " + Level wire source + " " + Level wire source
```

`succ` / `max` / `imax` の separator は single ASCII space だけです。
`Level wire` は 6.2 `MachineExprRenderer v1` の canonical level source と同じ正規化規則を使います。
`succ 0`、`succ 1`、`succ succ 0` のように closed numeral level として decimal に畳める source は
canonical ではないため `InvalidCandidate` です。
client はそれぞれ `"1"`、`"2"`、`"2"` を送らなければなりません。
`Level wire` も max / imax の algebraic normalization は行いません。
したがって `"max 0 0"`、`"imax 0 0"`、`"succ max 0 0"` は、recursive operand が canonical であれば
source として canonical です。
server はこれらを `"0"` や `"1"` に畳んではいけません。
server は Level source を parse した後に canonical level source を再生成し、input bytes と byte-for-byte に一致しない場合は
candidate schema violation として拒否します。
`Level` param として許す name は current root theorem の `root.universe_params` に含まれる
`MachineUniverseParamName` だけです。
premise の `universe_params` は AI が `universe_args` の arity / order を読むための label であり、
candidate validation の level parameter scope には追加しません。
余分な whitespace、parentheses、negative number、leading zero、unknown level parameter、non-renderable parameter name は
candidate schema violation として `InvalidCandidate` です。
Phase 4 validation は `universe_args` を input order のまま Phase 1 `Level` canonical bytes へ変換します。
search / prompt response の `universe_params` は AI が polymorphic premise の `universe_args` を作るための
ExportEntry order です。

## 7.1 単一候補実行

主 API は tactic text ではなく `MachineTacticCandidate` を受け取ります。
次の `exact` 例は、初期 goal に `intro n` を実行した後の snapshot に対するものです。

```json
POST /machine/tactics/run
{
  "session_id": "msess_001",
  "snapshot_id": "mst_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  "state_fingerprint": "sha256:...",
  "goal_id": "g1",
  "candidate": {
    "kind": "exact",
    "term": {
      "source": "@Eq.refl.{1} Nat n"
    }
  },
  "deterministic_budget": {
    "max_tactic_steps": 64,
    "max_whnf_steps": 10000,
    "max_conversion_steps": 10000,
    "max_rewrite_steps": 100,
    "max_meta_allocations": 8,
    "max_expr_nodes": 20000
  },
  "scheduler_limits": {
    "timeout_ms": 100
  }
}
```

`/machine/tactics/run` request object は次の field だけを持ちます。

```text
required:
  session_id
  snapshot_id
  state_fingerprint
  goal_id
  candidate
  deterministic_budget
optional:
  scheduler_limits
```

top-level non-object、top-level unknown field、duplicate key、`session_id` / `snapshot_id` /
`state_fingerprint` / `goal_id` の omitted / `null` / non-string、invalid `SessionId` grammar、
invalid `SnapshotId` grammar、invalid `HashString`、invalid `GoalId` grammar は
`InvalidTacticRunRequest` です。
`candidate` omitted / `null` / non-object は `InvalidTacticRunRequest` です。
`deterministic_budget` omitted / `null` / non-object、budget object 内の unknown field や invalid integer は
`InvalidBudget` です。
`scheduler_limits` omitted は no scheduler hint です。
`scheduler_limits = {}` は valid で、omitted と同じく scheduler hint なしを意味します。
`scheduler_limits = null`、non-object、unknown field、endpoint で許可されない field、invalid integer は
`InvalidSchedulerLimits` です。
`batch_policy` や `expected_target_hash` のような run request に存在しない top-level field は
`InvalidTacticRunRequest` として拒否し、無視してはいけません。

`/machine/tactics/run` の request validation priority は次に固定します。
同じ request が複数の failure を含む場合でも、最初に失敗した stage の error kind だけを返します。

```text
TacticRun request validation order:
  1. top-level object / duplicate key / unknown field を検査する。
     失敗は InvalidTacticRunRequest。
  2. session_id / snapshot_id / state_fingerprint / goal_id の presence、primitive type、wire grammar を検査する。
     失敗は InvalidTacticRunRequest。
  3. candidate の presence と object type だけを検査する。
     omitted / null / non-object は InvalidTacticRunRequest。
     candidate の semantic schema はこの stage では検査しない。
  4. deterministic_budget の presence、object type、field set、integer range を検査する。
     失敗は InvalidBudget。
  5. scheduler_limits が present の場合だけ object type、field set、integer range を検査する。
     失敗は InvalidSchedulerLimits。
  6. session / snapshot / state / goal lookup を実行する。
```

request envelope validation 後、`session_id` が存在しない場合は `UnknownSession` です。
その後の snapshot lookup は 6.1 の session-scoped lookup order に従い、missing entry は `UnknownSnapshot`、
stored snapshot self-check 失敗は `InvalidMachineProofState`、stored snapshot の `state_fingerprint` と
request の `state_fingerprint` が一致しない場合は `StateFingerprintMismatch` です。
grammar は正しいが current snapshot の open goal に存在しない `goal_id` は `GoalNotOpen` です。
`candidate` object の内側は、request envelope / budget / scheduler validation と
session / snapshot / state / goal lookup がすべて成功した後だけ validation します。
`candidate` object 内の unknown field、invalid variant、7.0 variant schema に定義されていない correlation/hash field、
`RawMachineTerm` wire object shape violation、
invalid `Level` / `MachineLocalName` wire grammar は `InvalidCandidate` です。
したがって、snapshot lookup と candidate schema の両方が壊れている request では、必ず lookup error を先に返します。

成功レスポンス:

```json
{
  "status": "success",
  "result": {
    "kind": "closed",
    "previous_state_fingerprint": "sha256:...",
    "candidate_hash": "sha256:...",
    "deterministic_budget_hash": "sha256:...",
    "next_snapshot_id": "mst_cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
    "next_state_fingerprint": "sha256:...",
    "closed_goals": ["g1"],
    "new_goals": [],
    "delta": {
      "proof_delta_hash": "sha256:...",
      "assigned_goal": "g1",
      "assigned_proof_expr_hash": "sha256:..."
    }
  }
}
```

`/machine/tactics/run` の success `result` object は上の field だけを返します。
`result.kind` の MVP allowed values は `"closed"` と `"expanded"` だけです。
`kind = "closed"` は `delta.assigned_goal` が解決され、`new_goals = []` の場合です。
`kind = "expanded"` は `delta.assigned_goal` が解決され、その代わりに 1 件以上の new open goal が生成された場合です。
MVP の tactic は 1 件の selected goal に対する transition なので、`closed_goals` は常に
`[delta.assigned_goal]` で、長さは 1 です。
`new_goals` は Phase 4 `MachineProofDelta.new_goals` の goal id を next snapshot の `open_goals` order で返します。
`new_goals` に含まれる goal id は `next_snapshot_id` が指す snapshot の `open_goals` に存在しなければなりません。
`new_goals = []` なら `kind` は必ず `"closed"`、`new_goals` が空でなければ `kind` は必ず `"expanded"` です。

Phase 5 tactic success response は compact execution summary であり、Phase 4 `MachineProofDelta` の full wire payload ではありません。
`delta.proof_delta_hash` は Phase 4 `MachineProofDelta canonical bytes` から計算した hash ですが、
response の `delta` object だけから client が再計算できるとは限りません。
next state payload は session snapshot store に保持し、
success response には debugging / correlation に必要な summary field だけを返します。
`assigned_meta`、`new_metas`、full context / target hash list などの full `MachineProofDelta` payload は
MVP の response / store contract には含めません。
`/machine/tactics/run` は success response を返す前に next snapshot payload を session snapshot store に保存し、
`next_snapshot_id` と `next_state_fingerprint` で取得できるようにしなければなりません。
client が full `MachineProofDelta` を監査する API は non-MVP とし、追加する場合は full delta wire schema、
canonical bytes、snapshot store lifetime、redaction rule をこの文書で固定します。

Phase 4 tactic transition が logical success を返した後で、next snapshot の `StoredSnapshotView canonical bytes`
materialization、`proof_skeleton_hash` / `open_goals` / `goals` invariant validation、
または同じ `snapshot_id` の既存 entry との consistency check に失敗した場合、success response を返してはいけません。
この failure は snapshot quota / server resource stop ではなく、`InvalidMachineProofState`、
diagnostic phase `tactic_execution` です。
この場合 `next_snapshot_id` / `next_state_fingerprint` / `delta` は返さず、`candidate_hash` と
`deterministic_budget_hash` は canonicalization 済み candidate と accepted budget の correlation field として返します。
top-level `unchanged_state_fingerprint` は request の `state_fingerprint` と同じ値を返します。
snapshot store quota により保存できない場合だけ、6.4 の scheduler artifact response を使います。

失敗レスポンス:

```json
{
  "status": "error",
  "error": {
    "kind": "type_mismatch",
    "phase": "machine_term_check",
    "goal_id": "g1",
    "tactic_kind": "exact",
    "candidate_hash": "sha256:...",
    "deterministic_budget_hash": "sha256:...",
    "diagnostic_hash": "sha256:...",
    "expected_hash": "sha256:...",
    "actual_hash": "sha256:...",
    "retryable": false
  },
  "unchanged_state_fingerprint": "sha256:..."
}
```

失敗時、元 snapshot は変更しません。
top-level `unchanged_state_fingerprint` は、request validation、budget validation、scheduler validation、
session lookup、snapshot lookup、state fingerprint check、goal lookup がすべて成功した後の
candidate validation / term parse-check / tactic execution error response にだけ必須で返します。
値は request の `state_fingerprint` と同じ `HashString` です。
`InvalidTacticRunRequest`、`InvalidBudget`、`InvalidSchedulerLimits`、`UnknownSession`、`UnknownSnapshot`、
`StateFingerprintMismatch`、`GoalNotOpen`、snapshot materialization 中の `InvalidMachineProofState` では
`unchanged_state_fingerprint` を omit します。
ここでいう snapshot materialization 中の `InvalidMachineProofState` は、入力 snapshot を lookup / materialize する段階の
failure だけです。
logical success 後の next snapshot materialization / store consistency failure は上の規則どおり
`unchanged_state_fingerprint` を返します。
`exact` / `apply` / `rw` などが必要とする expected target は、server が `snapshot_id`,
`state_fingerprint`, `goal_id` から取得した open goal の `target` から常に導出します。
外部 request は expected target hash を渡しません。
stale request 検出は `state_fingerprint` と `goal_id` の照合で行います。
`candidate_hash` は Phase 4 の `validate_machine_tactic_candidate` が Phase 4 candidate canonicalization を完了した後に
server が `MachineTactic canonical bytes` から計算した値です。
`deterministic_budget_hash` も server が accepted budget payload から計算します。
candidate schema validation、または Phase 5 `RawMachineTerm` prepass の parse / canonicalization に失敗した error では
`candidate_hash` を返しません。
`MachineTermElaborationError` / `UnknownName` / `ImplicitArgumentRequired` / `TypeMismatch` / `ExpectedPiType` などでも、
その前に Phase 4 candidate canonicalization が成功していない場合は `candidate_hash` を返しません。
`InvalidCandidate` には pre-canonical failure と post-canonical failure があります。
candidate object の wire schema、variant field set、`TacticHead` / `SimpRuleRef` / `Level` / `MachineLocalName`
schema violation、7.0 variant schema に定義されていない correlation/hash field、`RawMachineTerm` object shape violation は pre-canonical failure であり、
`phase = "candidate_validation"` ですが `candidate_hash` を返しません。
Phase 4 `MachineTactic canonical bytes` を構築できた後に、current state / goal に対する head / rule / local /
argument validation が `UnknownTacticHead`、`AmbiguousTacticHead`、`UnknownSimpRule`、`InvalidLocalHead` などで
失敗する場合、または `Intro.name` collision validation が失敗して `InvalidCandidate` に写る場合は
post-canonical failure であり、`candidate_hash` を返します。
Phase 4 `run_machine_tactic` またはその前段の Phase 4 state/candidate semantic validation が、入力 snapshot
materialization ではなく実行中の proof state invariant failure として `InvalidMachineProofState` を返した場合は、
deterministic tactic execution failure として扱います。
この failure の diagnostic phase は `tactic_execution` で、`candidate_hash`、`deterministic_budget_hash`、
top-level `unchanged_state_fingerprint` を返します。
`GoalNotOpen` や stale snapshot として再分類してはいけません。
budget validation に失敗した error では `deterministic_budget_hash` を返しません。
request envelope / budget / scheduler validation を通過した後の tactic error は、canonical diagnostic bytes から計算した
`diagnostic_hash` を必ず返します。

`/machine/tactics/run` の `deterministic_budget_hash` response population は次で固定します。

```text
/machine/tactics/run deterministic_budget_hash:
  omit:
    - InvalidTacticRunRequest
    - InvalidBudget
    - InvalidSchedulerLimits
    - UnknownSession
    - UnknownSnapshot
    - StateFingerprintMismatch
    - GoalNotOpen
    - InvalidMachineProofState while loading or materializing the input snapshot

  include in scheduler_stopped:
    - timeout / resource_limit_exceeded after deterministic_budget validation,
      session lookup, input snapshot lookup/materialization, state_fingerprint check,
      and open goal lookup have all succeeded

  include in error object:
    - InvalidCandidate after snapshot lookup and open goal lookup have succeeded, including pre-canonical candidate schema failure
    - MachineTermParseError / MachineTermElaborationError / UnknownName /
      ImplicitArgumentRequired / TypeMismatch / ExpectedPiType while checking candidate terms
    - InvalidMachineProofState caused by a Phase 5 / Phase 4 adapter invariant failure after
      RawMachineTerm prepass, snapshot lookup, and open goal lookup have succeeded
    - UnsupportedTactic / RewriteRuleInvalid / SimpNoProgress / InductionTargetNotNat /
      BudgetExceeded / TooManyGoals / TooLargeTerm
    - InvalidMachineProofState produced by Phase 4 tactic semantic validation or run_machine_tactic after Phase 4 candidate canonicalization
    - InvalidMachineProofState while materializing or storing the next snapshot after logical tactic success
```

scheduler stop response:

```json
{
  "status": "scheduler_stopped",
  "previous_state_fingerprint": "sha256:...",
  "deterministic_budget_hash": "sha256:...",
  "scheduler_artifact": {
    "kind": "timeout",
    "scope": "candidate",
    "retryable": true
  }
}
```

`scheduler_artifact.kind` は `"timeout"` または `"resource_limit_exceeded"` です。
`previous_state_fingerprint` は request の `state_fingerprint` と同じ値で、state_fingerprint check 成功後にだけ返せます。
session lookup、input snapshot lookup/materialization、state_fingerprint check、または open goal lookup が終わる前に
server-local guard が発火した場合は Machine API の `scheduler_stopped` ではなく transport / resource layer error として扱い、
`deterministic_budget_hash`、`previous_state_fingerprint`、`diagnostic_hash` は返しません。

`/machine/tactics/run` の scheduler stop observation rule は次に固定します。

```text
TacticRun scheduler observation:
  1. request envelope、deterministic_budget、scheduler_limits、session / snapshot / state / goal lookup が成功するまでは
     Machine API scheduler artifact を返してはいけない。
  2. lookup 成功後、candidate schema validation / Phase 5 RawMachineTerm prepass / Phase 4 candidate canonicalization /
     term elaboration / tactic execution / next snapshot materialization の中断点で accepted scheduler_limits を観測してよい。
  3. 各 observation point では、すでに deterministic candidate result が確定しているかを先に判定する。
     確定している場合は deterministic success または deterministic error を返し、scheduler_stopped にしてはいけない。
  4. deterministic candidate result がまだ確定していない場合だけ、
     accepted scheduler_limits または 6.4 snapshot store quota / resource guard stop を scheduler_stopped として返してよい。
  5. deterministic candidate result の確定とは、pre-canonical candidate error、term parse/check error、
     post-canonical InvalidCandidate、tactic deterministic error、logical success 後の
     next snapshot materialization / store consistency InvalidMachineProofState、または next snapshot store / reuse 済み
     success response payload が確定した時点をいう。
```

同じ observation point で deterministic candidate result と timeout / resource limit が両方成立し得る場合は、
deterministic candidate result を優先します。
deterministic result 確定前に timeout と resource limit が同時に観測された場合は、resource limit を優先し、
`scheduler_artifact.kind = "resource_limit_exceeded"`、`scope = "candidate"` を返します。
この response は deterministic tactic error ではないため、`candidate_hash` / `diagnostic_hash` は返さず、
deterministic cache や replay plan に入れません。

## 7.2 Batch 実行

AI 探索では、同じ state に対して多数の候補を試します。
batch API は各候補を独立 transaction として扱い、成功候補ごとに別 snapshot を返します。

```json
POST /machine/tactics/batch
{
  "session_id": "msess_001",
  "snapshot_id": "mst_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  "state_fingerprint": "sha256:...",
  "goal_id": "g1",
  "candidates": [
    {
      "candidate_id": "c0",
      "candidate": {
        "kind": "exact",
        "term": {
          "source": "@Eq.refl.{1} Nat n"
        }
      }
    },
    {
      "candidate_id": "c1",
      "candidate": {
        "kind": "simp-lite",
        "rules": []
      }
    }
  ],
  "deterministic_budget": {
    "max_tactic_steps": 64,
    "max_whnf_steps": 10000,
    "max_conversion_steps": 10000,
    "max_rewrite_steps": 100,
    "max_meta_allocations": 8,
    "max_expr_nodes": 20000
  },
  "scheduler_limits": {
    "per_candidate_timeout_ms": 100,
    "batch_timeout_ms": 1000
  },
  "batch_policy": {
    "max_evaluated_candidates": 128,
    "stop_after_successes": 8,
    "stop_after_failures": 128
  }
}
```

`/machine/tactics/batch` request object は次の field だけを持ちます。

```text
required:
  session_id
  snapshot_id
  state_fingerprint
  goal_id
  candidates
  deterministic_budget
  batch_policy
optional:
  scheduler_limits
```

top-level non-object、top-level unknown field、duplicate key、`session_id` / `snapshot_id` /
`state_fingerprint` / `goal_id` の omitted / `null` / non-string、invalid `SessionId` grammar、
invalid `SnapshotId` grammar、invalid `HashString`、invalid `GoalId` grammar は
`InvalidBatchPolicy` です。
`candidates` omitted / `null` / non-array、empty array、256 件を超える array、duplicate `candidate_id` は
`InvalidBatchPolicy` です。
`deterministic_budget` omitted / `null` / non-object、budget object 内の unknown field や invalid integer は
`InvalidBudget` です。
`batch_policy` omitted / `null` / non-object、policy object 内の unknown field、省略 field、invalid integer、
256 を超える field value は
`InvalidBatchPolicy` です。
`scheduler_limits` omitted は no scheduler hint です。
`scheduler_limits = {}` は valid で、omitted と同じく scheduler hint なしを意味します。
`scheduler_limits = null`、non-object、unknown field、endpoint で許可されない field、invalid integer は
`InvalidSchedulerLimits` です。

`/machine/tactics/batch` の request validation priority は次に固定します。
同じ request が複数の failure を含む場合でも、最初に失敗した stage の error kind だけを返します。

```text
TacticBatch request validation order:
  1. top-level object / duplicate key / unknown field を検査する。
     失敗は InvalidBatchPolicy。
  2. session_id / snapshot_id / state_fingerprint / goal_id の presence、primitive type、wire grammar を検査する。
     失敗は InvalidBatchPolicy。
  3. candidates の presence、array type、non-empty、length <= 256、item object、item duplicate key、
     candidate_id presence / grammar / uniqueness、candidate field presence だけを検査する。
     失敗は InvalidBatchPolicy。
     candidate field の値が null / non-object / invalid variant かどうかはこの stage では検査しない。
  4. deterministic_budget の presence、object type、field set、integer range を検査する。
     失敗は InvalidBudget。
  5. batch_policy の presence、object type、field set、integer range、各 field <= 256 を検査する。
     失敗は InvalidBatchPolicy。
  6. scheduler_limits が present の場合だけ object type、field set、integer range を検査する。
     失敗は InvalidSchedulerLimits。
  7. session / snapshot / state / goal lookup を実行する。
```

request envelope validation 後、`session_id` が存在しない場合は `UnknownSession` です。
その後の snapshot lookup は 6.1 の session-scoped lookup order に従い、missing entry は `UnknownSnapshot`、
stored snapshot self-check 失敗は `InvalidMachineProofState`、stored snapshot の `state_fingerprint` と
request の `state_fingerprint` が一致しない場合は `StateFingerprintMismatch` です。
grammar は正しいが current snapshot の open goal に存在しない `goal_id` は `GoalNotOpen` です。

レスポンス:

```json
{
  "status": "ok",
  "previous_state_fingerprint": "sha256:...",
  "deterministic_budget_hash": "sha256:...",
  "results": [
    {
      "candidate_id": "c0",
      "status": "success",
      "candidate_hash": "sha256:...",
      "next_snapshot_id": "mst_cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
      "next_state_fingerprint": "sha256:...",
      "proof_delta_hash": "sha256:..."
    },
    {
      "candidate_id": "c1",
      "status": "error",
      "candidate_hash": "sha256:...",
      "error_kind": "simp_no_progress",
      "phase": "tactic_execution",
      "goal_id": "g1",
      "tactic_kind": "simp-lite",
      "diagnostic_hash": "sha256:...",
      "retryable": false
    }
  ]
}
```

Batch の per-candidate success result も compact summary です。
`proof_delta_hash` は Phase 4 full `MachineProofDelta` から計算しますが、batch result item だけから再計算できるとは限りません。
success result ごとに `next_snapshot_id` が指す next snapshot payload を session snapshot store に保存してから response に含めます。
batch response に full `MachineProofDelta` payload を含める mode は MVP には入れません。
evaluated candidate が logical success を返した後で、その candidate の next snapshot materialization /
store consistency check に失敗した場合、その candidate は success result ではなく
`status = "error"`, `error_kind = "invalid_machine_proof_state"`, `phase = "tactic_execution"` の
per-candidate error result として返します。
この result は `candidate_hash` を返し、failure count に数えます。
accepted budget の `deterministic_budget_hash` は通常の batch response と同じ top-level field として返します。
`next_snapshot_id`、`next_state_fingerprint`、`proof_delta_hash` は返しません。
logical success candidate は、必要な next snapshot の保存または既存 snapshot reuse check が完了し、
success result の response fields が確定するまで success count に加算してはいけません。
snapshot store quota により保存できない場合はこの per-candidate error にせず、6.4 の batch scheduler artifact
response を使います。
Batch の per-candidate error result も compact item ですが、`diagnostic_hash` を監査できるように
`MachineApiErrorWire` と同じ canonical diagnostic fields を flattened field として返します。
`kind` field 名だけは top-level error object と区別するため `error_kind` に置き換えます。
`phase`、`diagnostic_hash`、`retryable` は必須です。
`goal_id`、`tactic_kind`、`primary_name`、`primary_axiom_ref`、`expected_hash`、`actual_hash` は
`MachineApiDiagnostic` の option population で `some` になる場合だけ返し、`none` の field は omit します。
`candidate_hash` は candidate schema validation、Phase 5 `RawMachineTerm` prepass、
Phase 4 candidate canonicalization がすべて成功し、Phase 4 `MachineTactic canonical bytes` を構築できた後に
発生した error result だけに返します。
candidate schema validation failure result は
`phase = "candidate_validation"` とし、recognized tactic kind なら `tactic_kind` も返しますが、
`candidate_hash` は返しません。
Phase 5 `RawMachineTerm` prepass の parse / canonicalization failure result は `MachineTermParseError` または
`MachineTermElaborationError` として返し、phase はそれぞれ `machine_term_parse` / `machine_term_check` です。
この result も `candidate_hash` は返しません。
Phase 5 prepass 成功後に Phase 4 が `InvalidMachineTermSource` を返す adapter invariant failure は、
per-candidate `status = "error"`、`error_kind = "invalid_machine_proof_state"`、
`phase = "candidate_validation"` として返します。
この result は `candidate_hash` を返さず、failure count に数えます。
accepted budget の `deterministic_budget_hash` は通常の batch response と同じ top-level field として返します。

`/machine/tactics/batch` の `deterministic_budget_hash` response population は次で固定します。
batch result item には per-candidate `deterministic_budget_hash` field を持たせず、返す場合は常に
top-level field として返します。

```text
/machine/tactics/batch deterministic_budget_hash:
  omit:
    - InvalidBatchPolicy
    - InvalidBudget
    - InvalidSchedulerLimits
    - UnknownSession
    - UnknownSnapshot
    - StateFingerprintMismatch
    - GoalNotOpen
    - InvalidMachineProofState while loading or materializing the input snapshot

  include as top-level field in normal response:
    - status = "ok", including responses whose results contain per-candidate
      candidate_validation, machine_term_parse, machine_term_check, tactic_execution,
      or invalid_machine_proof_state errors

  include as top-level field in partial scheduler response:
    - partial_timeout / partial_resource_limit after session, input snapshot, state fingerprint,
      and open goal lookup have succeeded
    - snapshot store quota / resource guard stop after the already completed prefix, possibly empty,
      has been finalized and before adding the current candidate to results
```

Batch の意味は、request の `candidates` 順に候補を 1 件ずつ評価した場合と同じにします。
各候補は同じ入力 snapshot に対する独立 transaction です。
`candidate_id` は batch request 内で一意の correlation id です。
grammar は `^[A-Za-z0-9._-]{1,64}$` に固定します。
`candidate_id` は `candidate_hash`、`proof_delta_hash`、`state_fingerprint` には入りません。

`batch_policy` は deterministic request の一部ですが、個別 tactic の `deterministic_budget_hash` には入りません。
wire request では `batch_policy` の全 field を明示します。
unknown field、omitted field、`null`、負数、float、0、256 を超える値、型幅を超える整数、
duplicate `candidate_id`、空 `candidates`、256 件を超える `candidates` は `InvalidBatchPolicy` として拒否します。
MVP の protocol-level batch cap は `candidates.length <= 256` かつ
`1 <= max_evaluated_candidates, stop_after_successes, stop_after_failures <= 256` です。
`batch_policy` の値が `candidates.length` を超えることは許します。
その場合も deterministic stop rule の「candidates をすべて評価済み」が先に発火します。

```text
MachineBatchPolicy canonical bytes:
  - tag "npa.phase5.batch-policy.v1"
  - max_evaluated_candidates as minimal unsigned LEB128 u32
  - stop_after_successes as minimal unsigned LEB128 u32
  - stop_after_failures as minimal unsigned LEB128 u32
```

deterministic な stop rule は次です。

```text
1. request envelope, deterministic_budget, scheduler_limits, batch_policy を先に validate する
2. index 0 から順に各 candidate を validate / execute し、必要な next snapshot の materialization /
   store / reuse check まで終えて candidate result を確定する
3. candidate result が確定した直後に、scheduler wall-clock / resource stop を観測する前に deterministic policy stop を判定する
4. 次のいずれかで prefix を止める:
   - evaluated count == max_evaluated_candidates
   - success count == stop_after_successes
   - failure count == stop_after_failures
   - candidates をすべて評価済み
5. deterministic policy stop が成立しない場合だけ、次 candidate に進む前、または実行中 candidate の中断点で
   accepted scheduler_limits / snapshot store quota stop を観測してよい
6. response.results は確定した prefix だけを request order で返す
```

`success count` / `failure count` は確定済み candidate result だけから更新します。
logical success 後の snapshot materialization / store consistency failure は success count に入れず、
上の per-candidate error result として failure count に入れます。
quota / resource guard による scheduler artifact stop は、実行中 candidate を results に含めず、
success / failure count にも入れません。
`max_evaluated_candidates`、`stop_after_successes`、`stop_after_failures`、または candidates 全件評価済みによる
deterministic policy stop は scheduler artifact ではありません。
この場合の response は `status = "ok"` で、`results` は確定済み prefix だけを request order で返します。
`completed_prefix_len` と `scheduler_artifact` は返しません。
candidate result 確定直後に deterministic policy stop と scheduler timeout / resource stop の両方が成立し得る場合は、
deterministic policy stop を優先し、`status = "ok"` を返します。

candidate semantic validation failure は batch 全体の request rejection ではなく、その candidate の
`status = "error"` result です。
candidate schema validation、または Phase 5 `RawMachineTerm` prepass の parse / canonicalization に失敗した candidate は
failure count に数え、`candidate_hash` を返しません。
candidate schema validation failure の phase は `candidate_validation`、Phase 5 `RawMachineTerm` prepass の
parse / canonicalization failure の phase は 11 の matrix に従い `machine_term_parse` または
`machine_term_check` です。
Phase 5 prepass 成功後に Phase 4 が `InvalidMachineTermSource` を返す adapter invariant failure は、
`phase = "candidate_validation"` の per-candidate `InvalidMachineProofState` であり、
`candidate_hash` を返さず、failure count に数えます。
post-canonical `InvalidCandidate` は、7.1 と同じく `phase = "candidate_validation"` で
`candidate_hash` を返します。
Phase 4 candidate canonicalization 成功後の term elaboration / type check / tactic execution error は、7.1 と同じく
`candidate_hash` を返します。
Phase 4 実行中の proof state invariant failure が `InvalidMachineProofState` に写る場合も、7.1 と同じく
per-candidate `error_kind = "invalid_machine_proof_state"`, `phase = "tactic_execution"` とし、
`candidate_hash` を返して failure count に数えます。
batch の error result は、candidate schema validation failure と Phase 5 `RawMachineTerm` prepass の
parse / canonicalization failure を含めて
`diagnostic_hash` を必ず返します。
prefix 外 candidate の inner `candidate` payload は semantic validation してはいけません。
request envelope validation は `candidates` が array であること、各 item が `candidate_id` と `candidate` だけを持つ
object であること、`candidate_id` が一意であることを batch 全体に対して確認します。
candidate item object の unknown field、duplicate key、`candidate_id` omitted、`candidate` omitted、
`candidate_id` が `^[A-Za-z0-9._-]{1,64}$` に一致しない場合は `InvalidBatchPolicy` として batch 全体を拒否します。
inner `candidate` payload は delayed payload として raw JSON のまま保持し、prefix 内で評価対象になるまで object として
decode してはいけません。
実装は inner `candidate` を raw byte slice、または duplicate-key-aware JSON syntax tree として保持しなければなりません。
`serde_json::Value` のような duplicate key を失う map representation へ prefix 評価前に正規化してはいけません。
inner `candidate` payload の duplicate key、`null` / non-object / unknown field / invalid variant /
`RawMachineTerm` wire object shape violation / invalid `Level` or `MachineLocalName` wire grammar は、
その candidate が prefix 内で評価対象になった時だけ `InvalidCandidate` result として返します。
prefix 外 candidate の `candidate` payload は、valid JSON value として切り出せる限り、duplicate key、`null`、non-object、
candidate schema としての invalid shape であっても検査してはいけません。
JSON 構文エラーは batch request 全体の parse / request validation error であり、prefix 外 delayed payload として
無視してはいけません。ただし outer candidate item object 自体は request envelope なので、上の item object rule に従って
prefix 外でも検査します。

server 内部で並列実行しても、prefix 外の先行計算結果は response / cache / replay に入れてはいけません。
`scheduler_limits.batch_timeout_ms` または `scheduler_limits.per_candidate_timeout_ms` により prefix の途中で
止まった場合は `status = "partial_timeout"` とし、`completed_prefix_len` を返します。
`scheduler_limits.max_memory_mb` により止まった場合は `status = "partial_resource_limit"` とします。
これらの result は retryable な scheduler artifact であり、deterministic cache key や replay plan には入れません。
batch scheduler stop は deterministic policy stop が成立していない observation point でだけ返せます。
candidate result が確定した後に policy stop が成立した場合、同じ observation point で timeout / resource limit も成立していても
partial scheduler response にしてはいけません。
batch response の `candidate_hash` は、prefix 内で candidate schema validation、Phase 5 `RawMachineTerm` prepass、
Phase 4 candidate canonicalization がすべて成功し、Phase 4 `MachineTactic canonical bytes` を構築できた
candidate ごとに返します。
candidate schema validation failure、または Phase 5 `RawMachineTerm` prepass の parse / canonicalization failure result には
`candidate_hash` を含めません。

`partial_timeout` / `partial_resource_limit` response は、scheduler stop 前に論理結果が確定した prefix だけを返します。
stop 中に実行中だった candidate、prefix 外 candidate、未確定 diagnostic は response に含めません。
実行中 candidate で `per_candidate_timeout_ms` または `max_memory_mb` が発火した場合、その candidate は
`results` に含めず、success / failure count にも数えません。

```json
{
  "status": "partial_timeout",
  "previous_state_fingerprint": "sha256:...",
  "deterministic_budget_hash": "sha256:...",
  "completed_prefix_len": 1,
  "results": [
    {
      "candidate_id": "c0",
      "status": "success",
      "candidate_hash": "sha256:...",
      "next_snapshot_id": "mst_cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
      "next_state_fingerprint": "sha256:...",
      "proof_delta_hash": "sha256:..."
    }
  ],
  "scheduler_artifact": {
    "kind": "timeout",
    "scope": "batch",
    "retryable": true
  }
}
```

`completed_prefix_len` は `results.length` と一致しなければなりません。
`partial_timeout` / `partial_resource_limit` の `deterministic_budget_hash` は accepted budget から計算した値です。
budget validation が終わる前に拒否される場合は partial scheduler response ではなく `InvalidBudget` を返します。
`scheduler_artifact.kind` は単発 run と batch で共通に `"timeout"` または `"resource_limit_exceeded"` です。
`scheduler_artifact.scope` は `"batch"` または `"candidate"` です。
batch scheduler stop classification は次の優先順位で固定します。
1. `max_memory_mb` または snapshot store quota / resource guard stop:
   `kind = "resource_limit_exceeded", scope = "batch"`
2. `batch_timeout_ms`:
   `kind = "timeout", scope = "batch"`
3. `per_candidate_timeout_ms`:
   `kind = "timeout", scope = "candidate"`
したがって batch timeout と per-candidate timeout が同じ observation point で同時に成立する場合は
`kind = "timeout", scope = "batch"` を返します。
`max_memory_mb` による resource stop は、`/machine/tactics/run` では常に
`kind = "resource_limit_exceeded", scope = "candidate"` です。
`/machine/tactics/batch` では process / batch 全体の guard として扱い、実行中 candidate の途中で発火した場合でも
常に `kind = "resource_limit_exceeded", scope = "batch"` を返します。
この場合、実行中 candidate は `results` に含めず、success / failure count にも数えません。

## 7.3 Text compatibility

debug / CLI 互換用に text tactic を受ける補助 API は MVP には入れません。
追加する場合も、`/machine/tactics/run` や `/machine/tactics/batch` の代替実行経路にしてはいけません。

```json
POST /machine/tactics/parse
{
  "input": "exact @Eq.refl.{1} Nat n"
}
```

この API は `MachineTacticCandidate` を返すだけです。
実行時は必ず `validate_machine_tactic_candidate` と Machine Surface check を通します。

---

# 8. Deterministic Budget

```rust
struct MachineDeterministicBudget {
    max_tactic_steps: u64,
    max_whnf_steps: u64,
    max_conversion_steps: u64,
    max_rewrite_steps: u64,
    max_meta_allocations: u64,
    max_expr_nodes: u64,
}

struct MachineSchedulerLimits {
    timeout_ms: Option<u64>,
    per_candidate_timeout_ms: Option<u64>,
    batch_timeout_ms: Option<u64>,
    max_memory_mb: Option<u64>,
}
```

`MachineDeterministicBudget` は Phase 4 AI の `TacticBudget` と同じ field set / semantics を持つ wire type です。
Phase 5 は budget field を追加したり、Phase 4 と異なる意味に再解釈してはいけません。
AI 向け API では、`MachineDeterministicBudget` だけを candidate hash / cache key / replay plan の
`deterministic_budget_hash` に入れます。`MachineSchedulerLimits` はプロセス保護用の外側制限であり、
deterministic fingerprint には入れません。
wire request では `MachineDeterministicBudget` の全 field を明示します。
server-side default によって hash が変わるのを避けるため、省略 field は `InvalidBudget` として拒否します。
unknown field、`null`、負数、float、型幅を超える整数も `InvalidBudget` として拒否します。
0 は Phase 4 `TacticBudget` と同じく有効な fuel 値です。
0 の field は canonical bytes にそのまま encode し、必要な操作に対応する fuel が 0 の時点で Phase 4 の
deterministic `TacticFuelExhausted` 系 error を返します。
Phase 5 は 0 を request validation で拒否してはいけません。

`deterministic_budget_hash` は次の canonical bytes から計算します。

```text
MachineDeterministicBudget canonical bytes:
  - tag "npa.phase4.tactic-budget.v1"
  - max_tactic_steps as minimal unsigned LEB128 u64
  - max_whnf_steps as minimal unsigned LEB128 u64
  - max_conversion_steps as minimal unsigned LEB128 u64
  - max_rewrite_steps as minimal unsigned LEB128 u64
  - max_meta_allocations as minimal unsigned LEB128 u64
  - max_expr_nodes as minimal unsigned LEB128 u64

deterministic_budget_hash:
  sha256(MachineDeterministicBudget canonical bytes)
```

wire JSON の field order は hash に影響しません。
`MachineSchedulerLimits`、wall-clock timeout、memory limit、rate limit はこの canonical bytes に入りません。
`scheduler_limits` object は endpoint ごとに許す field を固定します。

```text
/machine/tactics/run:
  allowed scheduler fields = timeout_ms, max_memory_mb

/machine/tactics/batch:
  allowed scheduler fields = per_candidate_timeout_ms, batch_timeout_ms, max_memory_mb
```

`scheduler_limits` omitted は scheduler hint なしを意味します。
`scheduler_limits = {}` は valid で、omitted と同じく scheduler hint なしを意味します。
present object の field はすべて optional ですが、存在する field は endpoint allowed field かつ positive u64 でなければなりません。
`scheduler_limits` present で object 以外、unknown field、endpoint で許されない field、`null`、負数、float、0、
型幅を超える整数は `InvalidSchedulerLimits` として拒否します。
`MachineSchedulerLimits` は retryable scheduler artifact の生成だけに使い、deterministic cache key、
`candidate_hash`、`deterministic_budget_hash`、`proof_delta_hash`、`state_fingerprint` には入れません。
`/machine/tactics/run` と `/machine/tactics/batch` で Machine API scheduler artifact を返してよいのは、
accepted `scheduler_limits` の明示 field が発火した場合、または 6.4 の snapshot store quota / resource guard が
発火した場合だけです。
`scheduler_limits` omitted または `{}` の場合、server-local default wall-clock / memory guard を
Machine API scheduler artifact として返してはいけません。
そのような guard は endpoint 契約を持たない transport / resource layer error であり、`MachineApiDiagnostic`、
`diagnostic_hash`、`deterministic_budget_hash` を生成しません。
`/machine/replay` の server process guard は 12.1 で別に固定した replay scheduler stop contract だけを使います。
`scheduler_artifact.scope` の MVP values は `"candidate"`, `"batch"`, `"replay"` です。
`/machine/tactics/run` は `"candidate"`、`/machine/tactics/batch` は `"candidate"` または `"batch"`、
`/machine/replay` は `"replay"` だけを返します。

deterministic budget に含めるもの:

```text
- tactic semantic transition count, capped by max_tactic_steps
- generated proof/core expression node count after Phase 4 candidate canonicalization, capped by max_expr_nodes
- metavariable allocation count, capped by max_meta_allocations
- proof-producing rewrite step count, capped by max_rewrite_steps
- WHNF step count, capped by max_whnf_steps
- kernel conversion step count, capped by max_conversion_steps
```

`max_open_goals`、`max_metas`、`max_simp_rewrite_steps` は session の `MachineTacticOptions` に属する semantic
limit であり、per-run deterministic budget には入れません。
`max_unification_steps` は Phase 4 `TacticBudget` に存在しないため Phase 5 wire schema にも入れません。
`max_expr_nodes` は raw request JSON、candidate schema validation、または Phase 5 `RawMachineTerm` prepass の
parse / canonicalization の入力サイズ制限ではありません。
Phase 4 candidate canonicalization 前の payload-size / parser resource guard は Machine API deterministic error ではなく、
transport / resource layer error として扱います。

accepted `MachineSchedulerLimits` による wall-clock timeout や memory limit の停止は
`scheduler_artifact.kind = timeout` / `resource_limit_exceeded` の retryable scheduler artifact として扱います。
この result は certificate payload に残さず、deterministic cache にも保存しません。
accepted `MachineSchedulerLimits` 以外の server-local timeout / memory guard はこの rule の対象外です。

---

# 9. Theorem Retrieval API

## 9.1 目的

Machine theorem retrieval は、AI に渡す premise 候補を verified metadata に固定して返します。
ランキングは非信頼です。候補の定理参照は `decl_interface_hash` と import の `export_hash` で固定します。
`global_ref` object 自体には `certificate_hash` を重複して入れません。
Phase 5 response 内の premise identity は、それを包む `session_root_hash` / `theorem_index_fingerprint` /
`query_fingerprint` が direct import の `(module, export_hash, certificate_hash)` を既に束縛している場合だけ有効な
session-local locator です。
この `global_ref` だけを Phase 6 `MachineStdGlobalRef` のような cross-session / release artifact identity として
使ってはいけません。
search / prompt response の `global_ref.name` は、Phase 2 `ExportEntry.name` そのものです。
`module` はその declaration を export した module を別 field として持つだけで、`global_ref.name` に
export module name を prefix として合成してはいけません。
たとえば `Std.Nat.Basic` が public declaration `Nat.add_zero` を export する場合、
wire は `module = "Std.Nat.Basic"` かつ `name = "Nat.add_zero"` です。
AI が search result から Phase 4 `TacticHead::Imported` を作る場合は、`global_ref.name` と
`global_ref.decl_interface_hash` をそのまま使います。

## 9.2 theorem index fingerprint

Phase 5 AI MVP の theorem index entry set は、current session の direct verified imports だけから構築します。
MVP の entry は direct import の public `ExportEntry` のうち、declaration kind が `TheoremDecl` または
`AxiomDecl` のものだけです。
`DefDecl`、`InductiveDecl`、constructor / recursor などの generated artifact、private dependency は theorem index
entry に含めません。
filesystem、network、mutable global cache から追加 premise を混ぜません。
`checked_current_decls` は tactic execution の環境には入りますが、MVP の theorem index には入れません。
current module declaration を premise search に含める場合は、`Imported` / `CurrentModule` を分ける
premise ref enum と canonical ordering を追加してから non-MVP endpoint として扱います。

```text
TheoremIndexFingerprint canonical bytes:
  - tag "npa.phase5.theorem-index.v1"
  - protocol_version
  - session_root_hash
  - theorem_index_schema_version
  - entries in canonical order:
      TheoremIndexEntry canonical bytes

TheoremIndexEntry canonical bytes:
  - tag "npa.phase5.theorem-index-entry.v1"
  - global_ref.module as Phase5Name canonical bytes
  - global_ref.name as Phase5Name canonical bytes
  - global_ref.export_hash as HashString digest bytes
  - global_ref.decl_interface_hash as HashString digest bytes
  - universe_params in ExportEntry order:
      each MachineUniverseParamName as Phase5 UTF-8 string primitive bytes
  - statement core hash as HashString digest bytes
  - head symbol option:
      none tag, or some tag + MachineGlobalRefView canonical bytes
  - axiom dependencies canonical hash as HashString digest bytes
  - searchable modes in fixed MachineTheoremMode order:
      each mode wire name as Phase5 UTF-8 string primitive bytes
```

`MachineTheoremMode` MVP enum は wire name `exact`, `apply`, `rw`, `simp` だけです。
unknown mode は `InvalidTheoremQuery` です。
`theorem_index_schema_version` MVP value は `"mvp-export-entry-v4-entry-bytes-visible-heads-universe-params"` です。
9.4 の scoring contract でいう `theorem index entry canonical bytes` も上の
`TheoremIndexEntry canonical bytes` そのものです。
search response JSON、`premise_id`、score、`suggested_candidates`、pretty text から別の entry bytes を作ってはいけません。
MVP の theorem index は Phase 6 の theorem search metadata、attribute、pretty statement、rewrite hint を読みません。
Phase 6 theorem index を search ranking に使う場合は、canonical artifact と verifier / hash rule を別に定義するまで
non-MVP とします。
`universe_params` は Phase 2 `ExportEntry.universe_params` を certificate name table から
`MachineUniverseParamName` に decode した list で、ExportEntry order をそのまま使います。
sort / rename / dedup してはいけません。
direct import の theorem-index-visible public export entry が `MachineUniverseParamName` として valid でない
universe parameter name を持つ場合は、5.2 stage 7 の session create preflight で `InvalidVerifiedImport` として拒否します。
theorem index construction 中にこの条件違反が見つかった場合は session invariant failure として
`InvalidTheoremIndex` です。

searchable mode は、Phase 2 verifier が certificate から導出した `ExportBlock` / `certified_env_decls` と、
session 作成時に Phase 4 が検証した `MachineTacticEnv.simp_registry` だけを入力にして決定します。
Phase 5 は user-facing notation や pretty text を解析しません。
`statement core hash` は imported `ExportEntry.type_hash` です。
`axiom dependencies canonical hash` は imported `ExportEntry.axiom_dependencies` を Phase 2 canonical order で
encode した bytes の hash です。
response の `axioms_used` は、同じ dependency list を 5.1 の `AxiomRef to MachineAxiomRefWire` 規則で
`MachineAxiomRefWire` に変換し、`MachineAxiomRefWire canonical bytes` の辞書順に sort/dedup した配列です。
search response と prompt response では `axioms_used` にこの JSON wire schema をそのまま返します。
Theorem index construction は `ExportEntry.type` を reduction しません。
WHNF、δ/β/ι/ζ reduction、conversion、unification、tactic execution は theorem index construction では禁止です。
`head symbol option` は canonical `ExportEntry.type` の leading syntactic `Pi` nodes を peel した後の conclusion を
構文的に見て、outermost head が global constant の場合だけその head にします。それ以外は `none` です。
head は display name や Phase 2 raw `import_index` ではなく、6.2 の `MachineGlobalRefView` canonical bytes へ
変換して保存します。
`MachineGlobalRefView` への変換は次です。

```text
Theorem head global ref normalization:
  - Phase 2 GlobalRef::Imported(import_index, name, decl_interface_hash):
      import_index をその certificate / checked env の canonical import table で解決し、
      public ExportEntry を name / decl_interface_hash で引く。
      ExportEntry が ordinary declaration に対応する場合:
        Imported {
          module,
          fully-qualified name,
          export_hash,
          decl_interface_hash,
          public_export = true,
          tactic_head_visible =
            direct_public_tactic_head_visible(module, name, export_hash, decl_interface_hash)
        } にする
      ExportEntry が generated constructor / recursor に対応する場合:
        VerifiedImportGeneratedDeclTable から parent fully-qualified name / parent decl_interface_hash を取得し、
        LocalGenerated {
          module,
          export_hash = some(export_hash),
          parent fully-qualified name,
          fully-qualified generated name,
          parent decl_interface_hash,
          generated decl_interface_hash,
          public_export = true,
          tactic_head_visible =
            direct_public_tactic_head_visible(module, generated_name, export_hash, generated_decl_interface_hash)
        } にする
      ordinary と generated の両方に一致する、またはどちらにも一致しない場合は InvalidTheoremIndex。
  - Phase 2 GlobalRef::Local(decl_index):
      theorem entry と同じ imported module の VerifiedImportDeclIndexTable[decl_index] で解決し、
      resolved_module = theorem entry の imported module、
      resolved_export_hash = theorem entry の current import export_hash として、
      Imported { module = resolved_module, fully-qualified name, export_hash = resolved_export_hash,
      decl_interface_hash, public_export = table.public_export,
      tactic_head_visible = table.public_export and
        direct_public_tactic_head_visible(resolved_module, resolved_name, resolved_export_hash, decl_interface_hash) } にする
  - Phase 2 GlobalRef::LocalGenerated(decl_index, name):
      theorem entry と同じ imported module の VerifiedImportGeneratedDeclTable[(decl_index, name)] で解決し、
      resolved_module = theorem entry の imported module、
      resolved_export_hash = theorem entry の current import export_hash として、
      LocalGenerated { module = resolved_module, export_hash = some(resolved_export_hash), parent fully-qualified name,
      fully-qualified generated name, parent decl_interface_hash, generated decl_interface_hash,
      public_export = table.public_export,
      tactic_head_visible = table.public_export and
        direct_public_tactic_head_visible(resolved_module, resolved_generated_name,
        resolved_export_hash, generated_decl_interface_hash) } にする
```

この正規化は `ExportBlock` の Vec position を使いません。
`GlobalRef::Local(decl_index)` が private dependency を指す場合でも、verified declaration table に fully-qualified
name と `decl_interface_hash` があれば
`MachineGlobalRefView::Imported { public_export = false, tactic_head_visible = false }` として正規化できます。
ただし theorem search の `global_ref` は direct import の public `ExportEntry` に限定し、tactic candidate の
`TacticHead::Imported` は `tactic_head_visible = true` の `MachineGlobalRefView` からだけ生成します。
private dependency だけを head に持つ entry は search result の suggested candidate を生成してはいけません。
`LocalGenerated` の `generated decl_interface_hash` は Phase 2 verifier が parent `InductiveDecl` に付与した
`decl_interface_hash` と同一であり、constructor / recursor から独立に作った hash や ExportBlock index から
推測した値を使ってはいけません。
public generated constructor / recursor は
`MachineGlobalRefView::LocalGenerated { public_export = true, export_hash = some(_), ... }` として表示してよく、
`tactic_head_visible = true` の場合だけ、tactic candidate を作るときに Phase 4 external schema に合わせて
`TacticHead::Imported { name = generated name, decl_interface_hash = generated decl_interface_hash }` に変換します。
current module の generated artifact は MVP では `public_export = false` とし、`CurrentModule` tactic head にはしません。
table lookup が失敗した場合、theorem index construction は `InvalidTheoremIndex` です。

変換後の `MachineGlobalRefView` canonical bytes が theorem index fingerprint の `head symbol option` 入力です。
`rw` mode の Eq family head との比較は、まず session の `MachineTacticEnv.eq_family` から
`ResolvedEqFamily.eq` を次の MVP 規則で public imported head に解決します。

```text
Resolved Eq head for theorem index:
  - ResolvedFamilyHead::Decl { head = TacticHead::Imported { name, decl_interface_hash }, ... }:
      DirectPublicExportNameTable[name] を引き、decl_interface_hash が一致する唯一の direct public ExportEntry かつ
      ordinary declaration なら
      MachineGlobalRefView::Imported {
        module,
        name,
        export_hash,
        decl_interface_hash,
        public_export = true,
        tactic_head_visible = direct_public_tactic_head_visible(module, name, export_hash, decl_interface_hash)
      } にする
      generated constructor / recursor の ExportEntry は Eq family head として扱わず、
      no theorem-index Eq head in MVP にする。
  - ResolvedFamilyHead::Builtin:
      no theorem-index Eq head in MVP
  - head = TacticHead::CurrentModule or TacticHead::Local:
      no theorem-index Eq head in MVP
  - name lookup failure, duplicate public name, decl_interface_hash mismatch, generated ExportEntry:
      no theorem-index Eq head in MVP
```

`DirectPublicExportNameTable` は session create 時に重複 direct public export name を拒否しているため、成功時の lookup は
request order や import order に依存しません。
`rw` mode 判定は、theorem entry の `head symbol option` とこの resolved Eq head の
`MachineGlobalRefView` canonical bytes を完全一致で比較します。
`MachineTacticEnv.eq_family = None`、builtin Eq、current module Eq、local Eq、または public imported Eq head に
一意解決できない session では、MVP theorem index は `rw` mode を付けません。
MVP の導出規則は次に固定します。

```text
mode exact:
  every theorem index entry
mode apply:
  theorem index entry の imported ExportEntry type が 1 個以上の leading syntactic Pi node を持つ場合
mode rw:
  theorem index entry の leading syntactic Pi nodes を peel した conclusion の outermost global head が current session の resolved public imported Eq head と一致する場合
mode simp:
  state.env.simp_registry に、その source TacticHead が同じ theorem index entry の imported ExportEntry を指す ResolvedSimpRule が 1 件以上ある場合
```

`rw` / `simp` の導出に失敗した entry はその mode だけを落とし、entry 全体は `exact` / `apply` 用に残してよいです。
導出結果の `searchable modes` が theorem index canonical bytes に入ります。
`simp` mode は user request の `options.tactic_options.simp_rules` と Phase 4 simp registry validation に依存します。
同じ import でも `simp_rules` が違う session では `session_root_hash` と `theorem_index_fingerprint` が変わります。
`same imported ExportEntry` は `(module, name, export_hash, decl_interface_hash)` の完全一致です。
`ResolvedSimpRule.source` が `CurrentModule` または `Local` の場合、MVP theorem index の imported entry に
`simp` mode を付けてはいけません。
`filters.exclude_axioms = true` の query では、axiom dependency list が空でない entry を eligible entries から除外します。
entry canonical order は `(module, name, export_hash, decl_interface_hash)` の tuple 辞書順です。
tuple 比較では `module` と `name` は 5.1 の `Phase5Name canonical bytes`、`export_hash` と
`decl_interface_hash` は `HashString` の 32-byte digest bytes を使います。
JSON string 表現、`sha256:` prefix、locale、Unicode normalization、request order は比較に使いません。
session create は同じ public fully-qualified export name を複数 import から公開する入力を拒否します。
theorem index construction 中に同じ public `name` が複数 entry として見つかる場合は、session invariant 破損として
`InvalidTheoremIndex` を返します。
`theorem_index_fingerprint = sha256(TheoremIndexFingerprint canonical bytes)` です。

search request の `modes` は set として扱い、canonical order は `exact`, `apply`, `rw`, `simp` です。
`modes` omitted、`null`、non-array、空 array、item の `null` / non-string、重複 mode、未知 mode は
`InvalidTheoremQuery` として拒否します。
`filters` は次の schema に固定します。

```text
MachineTheoremFilters:
  exclude_axioms: bool
  allowed_modules: omitted | Vec<ModuleName>
```

`filters` omitted、`null`、non-object、unknown field、`exclude_axioms` omitted、`exclude_axioms = null`、
`exclude_axioms` の non-bool、`allowed_modules = null` は `InvalidTheoremQuery` として拒否します。
`allowed_modules` が存在する場合は array でなければならず、item の `null` / non-string、
invalid `ModuleName` grammar は `InvalidTheoremQuery` です。
`filters.allowed_modules` は dedup 済み direct session imports の module name だけを許し、
module name の canonical order に sort/dedup します。
`allowed_modules` omitted は session 内の全 direct imported modules を許すという意味です。
`allowed_modules = []` は有効ですが、eligible entries は空になります。
dedup 済み direct session imports に存在しない module name が含まれる場合は `InvalidTheoremQuery` として拒否します。
`import_closure` にだけ存在する transitive dependency module は、theorem index entry set に入らず、
`allowed_modules` に指定された場合も session-dependent validation で拒否します。
explicit `allowed_modules` が sort/dedup 後に current session の direct imported module set 全体と一致する場合、
canonical filter では omitted と同じ all-direct form に正規化します。
ただし current session の direct imported module set が空の場合、explicit `allowed_modules = []` は
empty strict subset ではなく all-direct form です。
direct import が 1 件以上ある session では、explicit `allowed_modules = []` と strict subset は explicit-list form のままです。
Theorem query validation は pure wire validation と session-dependent validation に分けます。
pure wire validation は `modes` / `filters` / `limit` の object shape、duplicate key、omitted / null、
primitive type、mode enum、integer range、`ModuleName` grammar だけを検査し、session / snapshot lookup より先に実行します。
pure wire validation の失敗は `InvalidTheoremQuery` で、diagnostic phase は `request_validation` です。
session-dependent validation は、`filters.allowed_modules` が current session の dedup 済み direct imports に
存在するかどうかだけを検査します。
これは `session_id` lookup 成功後、snapshot / state / goal lookup より先に実行します。
session-dependent validation の失敗も、query request が session import set と矛盾しているという扱いで
diagnostic phase は `request_validation` に固定します。
したがって session が存在しない request は `UnknownSession` を返し、session が存在して
`allowed_modules` に direct import でない module がある request は、snapshot が壊れていても
`InvalidTheoremQuery` を先に返します。

`limit` は必須の unsigned integer で、MVP では `1 <= limit <= 256` に固定します。
`limit` omitted、`null`、0、負数、float、型幅を超える整数、256 を超える値は `InvalidTheoremQuery` です。
`limit` は canonical query bytes では minimal unsigned LEB128 u32 として encode し、score 計算には入れず、
deterministic ordering 後の truncation だけに使います。
`max theorem results` はこの `limit <= 256` による deterministic protocol cap です。
`/machine/search/for_goal` は MVP では `scheduler_limits`、partial result、timeout artifact を持ちません。
server-local response byte cap、CPU guard、memory guard、rate limit などで search を中断する場合は、
transport / resource layer error として扱い、`MachineApiDiagnostic`、`diagnostic_hash`、`query_fingerprint`、
partial `results`、deterministic cache entry を生成してはいけません。
deterministic Machine API response として返す search result は、必ず query 全体の theorem index construction、
ordering、truncation、suggestion lightweight validation が完了したものだけです。

```text
MachineTheoremFilters canonical bytes:
  - tag "npa.phase5.theorem-filters.v1"
  - exclude_axioms as 0x00 | 0x01
  - allowed_modules:
      0x00 for canonical all-direct form:
        request omitted, or explicit list equal to all direct imported modules after session-dependent sort/dedup
        including explicit empty list when the session has zero direct imports
      0x01 + list length + module names in canonical order for explicit strict subset
        including explicit empty list only when the session has one or more direct imports
```

## 9.3 goal 用検索

次の response 例は、context に `n : Nat` があり、target が
`Eq.{1} Nat (Nat.add n Nat.zero) n` の goal に対するものです。
この例の `simp` mode は、`Nat.add_zero` が session の Phase 4 `SimpRegistry` に登録済みである場合だけ返ります。
この例の `rw` mode は、session の `options.tactic_options.eq_family` が Phase 4
coherent family validation により public imported `Eq` head へ一意解決済みである場合だけ返ります。
この例の validated `rw` candidate は、さらに `Nat.add_zero` を source に持つ matching forward
`ResolvedSimpRule` が同じ `SimpRegistry` にある場合だけ返ります。
5.2 の最小 session create 例のように `eq_family = null` かつ builtin Eq head も使えない session では、
この theorem entry に `rw` mode を付けず、`rw` suggested candidate も返しません。
`rw` mode が付くが matching forward `ResolvedSimpRule` がない session では、`rw` mode は残して
`rw` suggested candidate だけを返しません。

```json
POST /machine/search/for_goal
{
  "session_id": "msess_001",
  "snapshot_id": "mst_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  "state_fingerprint": "sha256:...",
  "goal_id": "g1",
  "modes": ["exact", "apply", "rw", "simp"],
  "limit": 20,
  "filters": {
    "exclude_axioms": true,
    "allowed_modules": ["Std.Nat.Basic"]
  }
}
```

`/machine/search/for_goal` request object は次の field だけを持ちます。

```text
required:
  session_id
  snapshot_id
  state_fingerprint
  goal_id
  modes
  limit
  filters
optional:
  none
```

top-level unknown field、duplicate key、required field omitted、`null`、`session_id` / `snapshot_id` /
`state_fingerprint` / `goal_id` の non-string、invalid `SessionId` grammar、invalid `SnapshotId` grammar、
invalid `HashString`、invalid `GoalId` grammar は
`InvalidTheoremQuery` です。
`modes` / `filters` / `limit` の pure wire validation も session lookup 前に実行し、失敗は `InvalidTheoremQuery` です。
request envelope と pure query validation が成功した後、`session_id` が存在しない場合は `UnknownSession` です。
`filters.allowed_modules` の session-dependent validation は session lookup 後、snapshot lookup 前に実行します。
その後の snapshot lookup は 6.1 の session-scoped lookup order に従い、missing entry は `UnknownSnapshot`、
stored snapshot self-check 失敗は `InvalidMachineProofState`、stored snapshot の `state_fingerprint` と
request の `state_fingerprint` が一致しない場合は `StateFingerprintMismatch` です。
grammar は正しいが current snapshot の open goal に存在しない `goal_id` は `GoalNotOpen` です。

レスポンス:

```json
{
  "status": "ok",
  "query_fingerprint": "sha256:...",
  "theorem_index_fingerprint": "sha256:...",
  "search_profile_version": "mvp-zero-score-v1",
  "suggestion_profile_version": "mvp-suggested-candidates-v1",
  "results": [
    {
      "premise_id": "prem_0",
      "global_ref": {
        "module": "Std.Nat.Basic",
        "name": "Nat.add_zero",
        "export_hash": "sha256:...",
        "decl_interface_hash": "sha256:..."
      },
      "universe_params": [],
      "statement": {
        "core_hash": "sha256:...",
        "head": {
          "kind": "imported",
          "module": "Std.Init",
          "name": "Eq",
          "export_hash": "sha256:...",
          "decl_interface_hash": "sha256:...",
          "public_export": true,
          "tactic_head_visible": true
        },
        "machine": "forall (n : Nat), Eq.{1} Nat (Nat.add n Nat.zero) n"
      },
      "modes": ["exact", "apply", "rw", "simp"],
      "suggested_candidates": [
        {
          "status": "validated",
          "candidate_hash": "sha256:...",
          "candidate": {
            "kind": "rw",
            "rule": {
              "head": {
                "imported": {
                  "name": "Nat.add_zero",
                  "decl_interface_hash": "sha256:..."
                }
              },
              "universe_args": [],
              "args": [
                {"mode": "infer_from_target"}
              ]
            },
            "direction": "forward",
            "site": "eq_target_left"
          }
        },
        {
          "status": "validated",
          "candidate_hash": "sha256:...",
          "candidate": {
            "kind": "simp-lite",
            "rules": [
              {
                "name": "Nat.add_zero",
                "decl_interface_hash": "sha256:...",
                "direction": "forward"
              }
            ]
          }
        }
      ],
      "score": 0,
      "axioms_used": []
    }
  ]
}
```

`score` は探索のヒントです。server は score によって候補を verified とみなしません。
MVP の `score` wire value は JSON integer `0` に固定します。
JSON float、string、`NaN` / `Infinity` 相当の非 JSON 数値、または implementation-defined な丸めを response に使ってはいけません。
将来 non-zero ranking を導入する場合は、`search_profile_version` とともに score の wire 型
（例: 有理数文字列または固定小数 decimal）と canonical comparison rule を定義します。
search result の `universe_params` は必須 field で、Phase 2 `ExportEntry.universe_params` の order をそのまま返します。
空の場合も `[]` を返し、omitted や `null` にはしません。
search result の `suggested_candidates` は必須 field で、候補がない場合も JSON array `[]` を返します。
server は `suggested_candidates` を omit したり `null` にしたりしてはいけません。
search / prompt response の premise `statement` object は `core_hash`, `head`, `machine` を必須 field として返します。
`statement.head` は option field ですが omit してはいけません。
head がない場合は JSON `null`、head がある場合は `MachineGlobalRefView` object を返します。
premise `statement.machine` は、その premise の `universe_params` を level context として解釈する canonical
display Machine Surface source です。
この context は statement rendering / prompt 表示用であり、`/machine/tactics/run` の candidate `Level` scope には入りません。
`statement.machine` は 6.2 の display render scope で作ります。
pretty-only alias、short name、name shortening、user-facing alias のような canonical fully-qualified declaration name ではない
display-only text は `pretty` 系 field だけに出し、`statement.machine` と fingerprint 入力には入れません。
`statement.machine` は display render scope でだけ解決できる private / transitive dependency の fully-qualified name を
含む場合がありますが、それは candidate validation / execution scope で同じ source が受理されることを意味しません。
search / prompt の `suggested_candidates` は `statement.machine` から term をコピーせず、candidate validation / execution scope で
validation できる payload だけを返します。
`modes` は theorem index entry の検索可能 mode 全体であり、`suggested_candidates` は current goal に対して
MVP template validation まで通った候補だけの subset です。
上の例では `exact` / `apply` mode は entry に残りますが、current goal では validated suggestion になりません。
`rw` と `simp-lite` はこの順で `candidate_hash` を持つ suggested candidate として返ります。
`suggestion_profile_version` は `suggested_candidates` 生成規則の明示 version です。
MVP は `"mvp-suggested-candidates-v1"` に固定し、`query_fingerprint` に含めます。
同じ `query_fingerprint` から返る `suggested_candidates` の空 / 非空、順序、`candidate_hash` は同一でなければなりません。
MVP の suggestion generation は、request `modes` と result `modes` の intersection を `MachineTheoremMode` canonical order
`exact`, `apply`, `rw`, `simp` で処理し、各 mode 最大 1 件だけを同じ order で返します。
ただし MVP の `exact` / `apply` suggestion は無効です。
`exact` / `apply` mode は theorem index entry の検索可能 mode として残りますが、型照合 / instantiation / unification を
使わずに goal-specific candidate 成功可能性を判定する固定規則をまだ持たないため、`suggested_candidates` には出しません。
premise の `ExportEntry.universe_params` が空でない場合、MVP はその premise から suggested candidate を生成しません。
MVP の suggested candidate validation は proof-producing execution ではありません。
Phase 5 は template から raw `MachineTacticCandidate` を作り、次だけを実行します。

```text
MVP suggested candidate validation:
  1. external MachineTacticCandidate wire schema validation
  2. RawMachineTerm がある場合は Phase 5 RawMachineTerm prepass
  3. Phase 4 candidate canonicalization による MachineTactic canonical bytes の構築
  4. non-execution resolution subset:
     - TacticHead::Imported / CurrentModule は candidate execution scope で name + decl_interface_hash が一意に解決できること
     - TacticHead::Local を template が使う場合は selected goal context にその MachineLocalName が一意に存在すること
     - SimpRuleRef は session の Phase 4 SimpRegistry に exact SimpRuleKey で存在すること
     - rw template の rule は direct-import premise_head と matching ResolvedSimpRule から作られていること
     - rw template の args は chosen ResolvedSimpRule.rule_telescope の長さと一致し、各 item が infer_from_target であること
  5. candidate_hash = hash(Phase 4 MachineTactic canonical bytes)
```

この validation は `elaborate_machine_term_check`、WHNF、conversion、unification、rewrite lhs/rhs target matching、
rule telescope instantiation、`run_machine_tactic`、proof term construction、または kernel check を呼びません。
`rw` suggestion は current goal target が実際に rewrite 可能かを判定しません。
したがって `/machine/tactics/run` に同じ candidate を渡したとき、同じ `candidate_hash` を得たうえで
`RewriteRuleInvalid`、`UnsupportedTactic`、`BudgetExceeded` などの deterministic tactic error になる場合があります。
したがって `/machine/tactics/run` の `deterministic_budget` と `scheduler_limits` は suggestion generation に使いません。
template が型検査や tactic execution をしないと安全に候補化できない場合、MVP はその `suggested_candidate` を返しません。
下で定義する MVP deterministic templates は `RawMachineTerm` を生成しません。
将来の suggestion template が `RawMachineTerm` を生成する場合は、Machine Surface Complete mode を呼ばないまま
完全明示形式であることを判定する fixed syntactic rule、または別 profile の validation rule を
`suggestion_profile_version` と `query_fingerprint` に追加してから有効化します。
`suggested_candidates.status = "validated"` は「上の lightweight validation を通り、同じ raw candidate を
`/machine/tactics/run` に渡したとき同じ `candidate_hash` が得られる」という意味だけです。
その candidate が goal を閉じること、または tactic execution に成功することは意味しません。
将来 suggestion validation に Machine Surface Complete mode、tactic dry-run、別 fuel / timeout を入れる場合は
`suggestion_profile_version` を変え、その canonical budget / stop rule / failure omission rule を
この query fingerprint contract に追加します。

MVP の deterministic templates は次です。

```text
common:
  premise_head =
    { "imported": { "name": global_ref.name, "decl_interface_hash": global_ref.decl_interface_hash } }
  universe_args = []

exact:
  no suggested candidate in MVP

apply:
  no suggested candidate in MVP

rw:
  require result mode contains "rw"
  collect ResolvedSimpRule entries whose source points to the same imported ExportEntry
    and whose SimpRuleKey.direction is forward
  if none, do not emit rw suggestion
  choose the first rule by SimpRuleRef canonical order
  args =
    one { "mode": "infer_from_target" } for each InferableTerm parameter in chosen ResolvedSimpRule.rule_telescope order
    without checking whether those parameters are inferable from the current goal target
  candidate =
    {
      "kind": "rw",
      "rule": { "head": premise_head, "universe_args": [], "args": args },
      "direction": "forward",
      "site": "eq_target_left"
    }

simp:
  require result mode contains "simp"
  collect ResolvedSimpRule entries whose source points to the same imported ExportEntry
  if none, do not emit simp suggestion
  choose the first rule by SimpRuleRef canonical order
  candidate =
    { "kind": "simp-lite", "rules": [chosen SimpRuleRef wire] }
```

MVP は rw suggestion のために theorem type を別途 WHNF / binder-walk して rewrite analysis を作りません。
`rw` suggestion は session create 時点で Phase 4 が検証し、`SimpRegistry canonical bytes` と
`session_root_hash` / `theorem_index_fingerprint` に反映済みの `ResolvedSimpRule` だけを使います。
`rw` mode がある theorem entry でも、matching forward `ResolvedSimpRule` がなければ `rw` suggested candidate は返しません。
将来 simp registry に入っていない theorem から rw suggestion を作る場合は、canonical rewrite-analysis profile、
fuel / reduction rule、failure rule、fingerprint input を `suggestion_profile_version` と一緒に追加します。

各 template で作った raw candidate は、上で定義した MVP suggested candidate validation に成功した場合だけ
`suggested_candidates` に入れます。失敗した template の fallback candidate は作りません。
`suggested_candidates.status = "validated"` の候補は、返却前に candidate schema validation、
candidate が `RawMachineTerm` を含む場合の Phase 5 `RawMachineTerm` prepass、
Phase 4 `MachineTactic canonical bytes` 構築、head / rule / local 解決を
通したものだけにします。
MVP deterministic templates は `RawMachineTerm` を生成しないため、この prepass 条件は MVP では vacuous です。
Machine Surface Complete mode、tactic execution、kernel check は MVP suggested candidate validation では実行しません。
`suggested_candidates[*]` の MVP field は `status`, `candidate_hash`, `candidate` だけです。
MVP の `suggested_candidates[*].status` は必須 literal string `"validated"` だけです。
server は omitted / null / non-string / `"validated"` 以外の string を返してはいけません。
将来 `"repaired"` や `"unvalidated"` などを追加する場合は、response schema version、
canonical ordering、client compatibility rule をこの文書で固定してから有効化します。
`candidate` field は `/machine/tactics/run` にそのまま渡せる raw `MachineTacticCandidate` だけを含みます。
`candidate_hash` は Phase 4 `MachineTactic canonical bytes` の hash で、`/machine/tactics/run` が同じ raw
candidate を validation したときに返す `candidate_hash` と一致しなければなりません。
MVP search response は candidate-level metadata として `candidate_hash` 以外を返しません。
特に `canonical_hash`、`checked_expected_type_hash`、Phase 4 内部 `MachineTermSource.canonical_hash` は返しません。
result-level の `score` は candidate-level metadata ではなく、9.4 の search scoring contract に従います。
将来の suggestion template が term source を生成する場合、implicit argument や universe argument が必要な source は
`@Name.{u} ...` 形式で完全明示させる validation rule を追加します。
候補を validation できない場合は `suggested_candidates` には入れません。
server は省略した template failure を wire response に入れません。
ただし実装は debug / audit 用に、`query_fingerprint`、premise `global_ref`、mode、template kind、
省略理由の enum を server-local non-wire log として記録してよいです。
この log は response、`query_fingerprint`、`payload_fingerprint`、cache key、certificate payload に入れてはいけません。
MVP の search response は `repair_hints` を返しません。
repair hint を追加する場合は non-MVP response field として、wire schema、ordering、fingerprint input、
信頼境界をこの文書で固定してから有効化します。

## 9.4 query fingerprint

`query_fingerprint` は次から計算します。

```text
- tag "npa.phase5.theorem-query.v1"
- protocol version
- state_fingerprint
- goal_id
- goal_fingerprint
- theorem index fingerprint
- modes in fixed canonical order
- filters after canonicalization
- search_profile_version
- suggestion_profile_version
- limit as minimal unsigned LEB128 u32
```

`search_profile_version` は match score algorithm の明示 version です。
MVP は `"mvp-zero-score-v1"` に固定します。
将来 ranking algorithm を変える場合は、`search_profile_version` を変え、old profile で作った
`query_fingerprint` と cache artifact を再利用してはいけません。

score 計算には `limit` を入れません。
server は次の canonical bytes から `search_score_key_fingerprint` を計算します。

```text
SearchScoreKey canonical bytes:
  - tag "npa.phase5.search-score-key.v1"
  - protocol version
  - state_fingerprint
  - goal_id
  - goal_fingerprint
  - theorem index fingerprint
  - modes in fixed canonical order
  - filters after canonicalization
  - search_profile_version
```

同じ query fingerprint からは同じ result ordering と truncation を返します。
eligible entries は theorem index entries から `modes` と `filters` に合うものだけを取り、goal との deterministic
match score を計算します。
match score は `search_score_key_fingerprint` と `TheoremIndexEntry canonical bytes` だけの関数です。
`search_profile_version = "mvp-zero-score-v1"` では score を全 entry で `0` とします。
semantic score を導入する場合は別 `search_profile_version` と canonical scoring contract を定義します。
ordering は `score` 降順、同点なら `module`, `name`, `export_hash`, `decl_interface_hash` の tuple 辞書順に固定します。
この tuple 比較でも `module` / `name` は `Phase5Name canonical bytes`、hash は 32-byte digest bytes を使います。
response の `results` はこの ordering の先頭 `limit` 件だけです。
`premise_id` は response order の 0-based index から決定的に作ります。
format は `"prem_" + decimal(index)` で、decimal は ASCII digits、leading zero なしです。
同じ query fingerprint で同じ ordering / truncation の response は必ず同じ `premise_id` を返します。
`premise_id` は cache key や theorem index fingerprint には入れず、search / prompt response 内の相関 id としてだけ使います。

---

# 10. Prompt Payload API

Phase 5 AI は LLM を呼びません。
ただし Phase 7 が LLM に渡すための、決定的に整形された prompt payload を生成できます。

```json
POST /machine/prompt_payload
{
  "session_id": "msess_001",
  "snapshot_id": "mst_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  "state_fingerprint": "sha256:...",
  "goal_id": "g1",
  "include_pretty": true,
  "include_failed_candidates": true,
  "premise_selection": {
    "modes": ["exact", "apply", "rw", "simp"],
    "limit": 16,
    "filters": {
      "exclude_axioms": true,
      "allowed_modules": ["Std.Init"]
    }
  },
  "failed_candidates": [
    {
      "candidate_hash": "sha256:...",
      "error_kind": "type_mismatch",
      "diagnostic_hash": "sha256:..."
    }
  ]
}
```

レスポンス:

```json
{
  "status": "ok",
  "payload_fingerprint": "sha256:...",
  "premise_query_fingerprint": "sha256:...",
  "theorem_index_fingerprint": "sha256:...",
  "search_profile_version": "mvp-zero-score-v1",
  "suggestion_profile_version": "mvp-suggested-candidates-v1",
  "goal": {
    "target_machine": "Eq.{1} Nat n n",
    "target_pretty": "n = n",
    "context": [
      {
        "machine_name": "n",
        "display_name": "n",
        "type_machine": "Nat"
      }
    ]
  },
  "premises": [
    {
      "premise_id": "prem_0",
      "global_ref": {
        "module": "Std.Init",
        "name": "Eq.refl",
        "export_hash": "sha256:...",
        "decl_interface_hash": "sha256:..."
      },
      "universe_params": ["u"],
      "statement": {
        "core_hash": "sha256:...",
        "head": {
          "kind": "imported",
          "module": "Std.Init",
          "name": "Eq",
          "export_hash": "sha256:...",
          "decl_interface_hash": "sha256:...",
          "public_export": true,
          "tactic_head_visible": true
        },
        "machine": "forall (A : Sort u), forall (x : A), Eq.{u} A x x"
      },
      "modes": ["exact", "apply"],
      "axioms_used": []
    }
  ],
  "failed_candidates": [
    {
      "candidate_hash": "sha256:...",
      "error_kind": "type_mismatch",
      "diagnostic_hash": "sha256:..."
    }
  ],
  "allowed_tactics": ["intro", "exact", "apply"],
  "output_schema": "npa.machine_tactic_candidate.v1"
}
```

この payload は非信頼です。
LLM の返答は必ず `/machine/tactics/run` または `/machine/tactics/batch` に再投入します。
MVP の `output_schema` は固定文字列 `"npa.machine_tactic_candidate.v1"` です。
これは Phase 7 / LLM に期待する tactic candidate response schema を識別するための prompt payload field であり、
request で変更できません。
server は prompt payload を組み立てるとき常にこの値を使わなければならず、別の値を使う実装は
`PromptRenderedContent canonical bytes` と `payload_fingerprint` の互換性を失います。
schema を変える場合はこの文字列を変更し、同時に prompt payload version / canonical bytes rule の更新として扱います。
`/machine/prompt_payload` request object は次の field だけを持ちます。

```text
required:
  session_id
  snapshot_id
  state_fingerprint
  goal_id
  include_pretty
  include_failed_candidates
  premise_selection
  failed_candidates
optional:
  none
```

top-level unknown field、duplicate key、required field omitted、`null`、`session_id` / `snapshot_id` /
`state_fingerprint` / `goal_id` の non-string、invalid `SessionId` grammar、invalid `SnapshotId` grammar、
invalid `HashString`、invalid `GoalId` grammar、
`include_pretty` / `include_failed_candidates` の non-bool、`premise_selection` の non-object、
`failed_candidates` の non-array は `InvalidPromptPayloadRequest` です。
`premise_selection.modes` / `filters` / `limit` の pure wire validation は、9.2 の theorem query pure wire validation と
同じ規則を使い、session lookup 前に `InvalidPromptPayloadRequest` として拒否します。
`failed_candidates` は必須 array です。
空の場合も `[]` を明示します。
`include_failed_candidates = false` の場合、`failed_candidates` は `[]` でなければ `InvalidPromptPayloadRequest` です。
`failed_candidates` item は次の field だけを持つ object でなければなりません。

```text
FailedCandidatePromptItem:
  candidate_hash: HashString
  error_kind: FailedCandidateErrorKind
  diagnostic_hash: HashString
```

item が object でない、duplicate key を持つ、unknown field を持つ、required field omitted、`null`、
`candidate_hash` / `diagnostic_hash` の non-string または invalid `HashString`、
`error_kind` の non-string または `FailedCandidateErrorKind` 以外の場合は
`InvalidPromptPayloadRequest` です。
`failed_candidates` item schema validation と `include_failed_candidates` consistency validation は
session / snapshot lookup より前に実行し、失敗時の diagnostic phase は `request_validation` です。
request envelope、pure premise selection validation、`failed_candidates` item schema validation、
`include_failed_candidates` consistency validation がすべて成功した後、`session_id` が存在しない場合は
`UnknownSession` です。
`premise_selection.filters.allowed_modules` の session-dependent validation は session lookup 後、snapshot lookup 前に実行し、
失敗は `InvalidPromptPayloadRequest` です。
この validation でも 9.2 と同じく dedup 済み direct session imports だけを許し、`import_closure` にだけ存在する
transitive dependency module は `InvalidPromptPayloadRequest` として拒否します。
この失敗も prompt request が session import set と矛盾しているという扱いで、diagnostic phase は
`request_validation` に固定します。
その後の snapshot lookup は 6.1 の session-scoped lookup order に従い、missing entry は `UnknownSnapshot`、
stored snapshot self-check 失敗は `InvalidMachineProofState`、stored snapshot の `state_fingerprint` と
request の `state_fingerprint` が一致しない場合は `StateFingerprintMismatch` です。
grammar は正しいが current snapshot の open goal に存在しない `goal_id` は `GoalNotOpen` です。
`failed_candidates` に入れるべきものは、`/machine/tactics/run` または `/machine/tactics/batch` の error result が
`candidate_hash` と `diagnostic_hash` の両方を返した accepted candidate failure だけです。
ここでいう accepted candidate failure は、`candidate_hash` を伴い、かつ下の `FailedCandidateErrorKind` に含まれる
error kind だけです。
post-canonical `InvalidCandidate` は `candidate_hash` を返す場合がありますが、MVP の prompt repair context には入れません。
ただし MVP server は session history や cache を使って provenance を検証しません。
`failed_candidates` は非信頼 prompt context であり、server は schema、`HashString` grammar、allowed `error_kind`、
request order だけを検査します。
過去の response に存在しない、または別 session 由来に見える `candidate_hash` / `diagnostic_hash` であっても、
wire schema が正しければ provenance 不一致だけを理由に拒否してはいけません。
provenance 監査を行う endpoint を追加する場合は non-MVP とし、history store、session binding、diagnostic payload store、
fingerprint input、拒否 error を別途固定します。
candidate schema validation failure、Phase 5 RawMachineTerm prepass parse / canonicalization failure、
request envelope validation failure、
budget validation failure、goal lookup failure、scheduler artifact は `candidate_hash` を持たないため、
MVP の `failed_candidates` には入れません。
`machine_term_parse_error` は Phase 5 MVP では `candidate_hash` と一緒に返らないため、
`FailedCandidateErrorKind` には含めません。
`invalid_candidate` は pre-canonical failure では `candidate_hash` を持たず、post-canonical failure でも
repair 用の型エラー / 実行エラーとしては粗すぎるため、MVP の `FailedCandidateErrorKind` には含めません。
`failed_candidates.error_kind` の MVP allowed values は次だけです。

```text
FailedCandidateErrorKind:
  unsupported_tactic
  machine_term_elaboration_error
  unknown_name
  implicit_argument_required
  type_mismatch
  expected_pi_type
  rewrite_rule_invalid
  simp_no_progress
  induction_target_not_nat
  budget_exceeded
  too_many_goals
  too_large_term
```

上記以外の `error_kind`、特に `invalid_candidate`、`invalid_budget`、`goal_not_open`、
request validation 系 error、verify/session/search/prompt/replay 系 error は
`InvalidPromptPayloadRequest` として拒否します。

`payload_fingerprint` は次の canonical payload から計算します。

```text
PromptPayloadFingerprint input:
  - tag "npa.phase5.prompt-payload.v1"
  - protocol_version
  - session_root_hash
  - state_fingerprint
  - goal_id
  - include_pretty
  - include_failed_candidates
  - theorem_index_fingerprint
  - premise_query_fingerprint
  - rendered_prompt_content canonical bytes
```

`premise_selection` の canonicalized `modes` / `filters` / `limit` は `premise_query_fingerprint` に含まれるため、
`payload_fingerprint` へ別 field として二重に入れません。
`failed_candidates` は `rendered_prompt_content` の typed field として入るため、`payload_fingerprint` へ
別 field として二重に入れません。
raw `premise_selection` JSON の field order や spelling difference は hash 入力に使いません。
`rendered_prompt_content` は response JSON body 全体ではなく、次の typed canonical object です。
JSON object の field order、whitespace、escape の書き方は hash に影響しません。
`status`, `payload_fingerprint`, `theorem_index_fingerprint`, `premise_query_fingerprint`, `search_profile_version`,
`suggestion_profile_version` は除外します。
`theorem_index_fingerprint` と `premise_query_fingerprint` は上の explicit input として 1 回だけ hash に入ります。
`search_profile_version` と `suggestion_profile_version` は `premise_query_fingerprint` の入力として含まれるため、
payload fingerprint へ直接は入れません。
`payload_fingerprint` 自身は絶対に hash 入力へ入れません。

`PromptRenderedContent canonical bytes` は 5.1 の Phase 5 canonical primitive encoding を使います。

`rendered_prompt_content` に入る typed field は次だけです。

```text
PromptRenderedContent canonical bytes:
  - tag "npa.phase5.prompt-rendered-content.v1"
  - goal:
      target_machine string
      target_pretty option string
      context list in response order:
        machine_name string
        display_name option string
        type_machine string
        value_machine option string
        value_pretty option string
  - premises list in response order:
      premise_id string
      global_ref:
        module Phase5Name
        name Phase5Name
        export_hash HashString digest bytes
        decl_interface_hash HashString digest bytes
      universe_params list in ExportEntry order as MachineUniverseParamName strings
      statement:
        core_hash HashString digest bytes
        head option MachineGlobalRefView canonical bytes
        machine string
      modes list in MachineTheoremMode canonical order
      axioms_used list in response order as MachineAxiomRefWire canonical bytes
        response order MUST be MachineAxiomRefWire canonical order
  - failed_candidates list in request order:
      candidate_hash HashString digest bytes
      error_kind enum wire name
      diagnostic_hash HashString digest bytes
  - allowed_tactics list in MachineTacticKind canonical order
  - output_schema fixed string bytes: "npa.machine_tactic_candidate.v1"
```

実装は schema version を変えずに `rendered_prompt_content` へ追加 field を入れてはいけません。
`include_pretty = true` の場合、`target_pretty` と local `display_name` は `some` として payload fingerprint に含めます。
`include_pretty = false` の場合、`target_pretty`、local `display_name`、その他 pretty-only field は response から omit し、
canonical bytes では対応する option を `none` として encode します。
local `value_machine` option は `include_pretty` に依存せず、`MachineLocalView.value = Some(_)` の場合だけ `some` です。
local `value_pretty` option は pretty-only field なので、`include_pretty = true` かつ
`MachineLocalView.value = Some(_)` の場合だけ `some` です。
`include_pretty = true` の場合でも pretty text は Machine Surface の入力として信用してはいけません。
MVP の pretty printer version / options は 6.2 の fixed display profile と `protocol_version` で固定します。
将来 explicit display profile を request field にする場合は、その profile id と canonical bytes を
`payload_fingerprint` の入力に追加します。
`premise_selection` は `/machine/search/for_goal` の `modes` / `filters` / `limit` と同じ validation と
canonicalization を使います。
prompt payload に入る selected premises は、同じ `snapshot_id` / `state_fingerprint` / `goal_id` と
`premise_selection` で `/machine/search/for_goal` を呼んだ場合の `results` ordering / truncation と同じ premise set から作ります。
この一致は premise identity と verified metadata に限定します。
prompt response は search-only field である `score` と `suggested_candidates` を返さず、`PromptRenderedContent` にも含めません。
`premise_query_fingerprint` は 9.4 の search query fingerprint そのものを返し、payload fingerprint に含めます。
そのため `suggestion_profile_version` は premise query fingerprint の互換性入力ですが、prompt rendered content の field ではありません。
将来 prompt に suggested candidate を含める場合は、prompt response schema、`PromptRenderedContent canonical bytes`、
`payload_fingerprint` input を同時に更新しなければなりません。
response の `premises` は search result の `premise_id`、`global_ref`、`universe_params`、`statement`、`modes`、
`axioms_used` と同じ verified metadata を含みます。
`name` だけの premise 表現は返しません。
search / prompt response の各 premise は `universe_params` を必須 field として返します。
空の場合も `[]` を返し、omitted や `null` にはしません。
response の `allowed_tactics` は 6.2 の session capability subset を canonical `MachineTacticKind` order で返します。
prompt response の `goal.context` item は、対応する `MachineLocalView` から次の wire field を返します。
`machine_name` と `type_machine` は常に必須です。
`value_machine` は local `value = Some(_)` の場合だけ返し、`value = None` では omit します。
`display_name` は `include_pretty = true` の場合だけ返し、MVP では `machine_name` と同じ string です。
`value_pretty` は `include_pretty = true` かつ `value = Some(_)` の場合だけ返し、それ以外では omit します。
prompt は local let を暗黙展開してはいけません。AI が local let の定義を使う必要がある場合は、
`value_machine` を読んだうえで通常の Machine Surface term として候補を作り、run/batch に再投入します。
`include_failed_candidates = true` の場合、server は request の `failed_candidates` だけを payload に含めます。
session-local history、cache、server が過去に見た失敗結果から自動収集してはいけません。
`failed_candidates` は request order を維持し、各 item の `candidate_hash` / `error_kind` / `diagnostic_hash` を
`PromptRenderedContent canonical bytes` 経由で payload fingerprint に含めます。
`include_failed_candidates = false` の場合、`failed_candidates` は空でなければ拒否します。
prompt response の `failed_candidates` field は MVP では常に必須 array として返します。
`include_failed_candidates = false` の場合、response では `failed_candidates: []` を返し、
field を omit したり `null` を返したりしてはいけません。

`/machine/prompt_payload` は MVP では `scheduler_limits`、partial payload、timeout artifact を持ちません。
`max prompt payload bytes` は deterministic fingerprint input ではなく、server / transport layer の response-size guard です。
この guard、CPU guard、memory guard、rate limit などで prompt construction を中断する場合は、
transport / resource layer error として扱い、`MachineApiDiagnostic`、`diagnostic_hash`、`payload_fingerprint`、
partial `goal` / `premises` / `failed_candidates`、deterministic cache entry を生成してはいけません。
deterministic Machine API response として返す prompt payload は、`PromptRenderedContent canonical bytes` 全体を
構築し終え、`payload_fingerprint` を計算できたものだけです。
将来 prompt byte budget を deterministic request field にする場合は、request schema、fingerprint input、
truncation / rejection rule、error kind を同時に version up します。

---

# 11. Error Taxonomy

AI 修復に使うため、エラーは enum 中心にします。

```rust
enum MachineApiErrorKind {
    UnknownSession,
    UnknownSnapshot,
    StateFingerprintMismatch,
    SessionRootHashMismatch,
    InvalidVerifiedImport,
    InvalidCheckedCurrentDecl,
    InvalidMachineApiOptions,
    InvalidMachineProofState,
    InvalidSessionRequest,
    InvalidSnapshotRequest,
    InvalidTacticRunRequest,
    InvalidTheoremIndex,
    InvalidTheoremQuery,
    InvalidPromptPayloadRequest,
    InvalidBatchPolicy,
    InvalidSchedulerLimits,
    InvalidReplayPlan,
    InvalidVerifyRequest,
    ReplayHashMismatch,
    DisallowedAxiom,
    GoalNotOpen,
    InvalidCandidate,
    InvalidBudget,
    UnsupportedTactic,
    MachineTermParseError,
    MachineTermElaborationError,
    UnknownName,
    ImplicitArgumentRequired,
    TypeMismatch,
    ExpectedPiType,
    RewriteRuleInvalid,
    SimpNoProgress,
    InductionTargetNotNat,
    BudgetExceeded,
    TooManyGoals,
    TooLargeTerm,
    VerifyFailed,
}
```

wire JSON の `error.kind` は enum variant の lower_snake_case 名です。
例: `TypeMismatch` は `"type_mismatch"`、`InvalidSchedulerLimits` は `"invalid_scheduler_limits"` と encode します。
unknown `error.kind` は protocol violation として扱い、request payload 内で参照された場合はその request の
validation error に写します。

route dispatch 後に JSON parse failure、top-level non-object、または duplicate key decoder failure が起きた場合の
`error.kind` は、その endpoint の request envelope validation kind に固定します。

```text
Endpoint request-parse failure kind:
  POST /machine/sessions:
    InvalidSessionRequest
  DELETE /machine/sessions/{id}:
    InvalidSessionRequest
  POST /machine/snapshots/get:
    InvalidSnapshotRequest
  POST /machine/tactics/run:
    InvalidTacticRunRequest
  POST /machine/tactics/batch:
    InvalidBatchPolicy
  POST /machine/search/for_goal:
    InvalidTheoremQuery
  POST /machine/prompt_payload:
    InvalidPromptPayloadRequest
  POST /machine/replay:
    InvalidReplayPlan
  POST /machine/verify:
    InvalidVerifyRequest
```

route が存在しない、HTTP method が違う、body size limit に達した、認証 / rate limit で拒否した、などの
transport-level error は Machine API deterministic diagnostic ではありません。
それらを HTTP / RPC layer で返す場合、`MachineApiDiagnostic` と `diagnostic_hash` を生成してはいけません。

Phase 5 は Phase 3 / Phase 4 の structured error kind を、次の表で必ず `MachineApiErrorKind` へ写します。
この表にない upstream error kind を Machine API response として返してはいけません。
Phase 3 / Phase 4 に新しい structured error kind を追加する場合は、この表を更新するか protocol version を
上げてから Phase 5 に接続します。

```text
Phase 3 MachineErrorKind -> MachineApiErrorKind:
  ParseError:
    MachineTermParseError
  UnsupportedSyntax / UnsupportedItem / ImportAfterItem / ImportResolutionError / MissingVerifiedImport /
  DuplicateDeclaration / DuplicateUniverseParam / UnknownUniverseParam / UnannotatedBinder / UnannotatedLet /
  HoleNotAllowed / UnsolvedUniverseMeta / KernelRejected:
    MachineTermElaborationError
  UnknownGlobalName / ShortGlobalName / AmbiguousGlobalName / GlobalShadowedByLocal / UnknownLocalName:
    UnknownName
  ImplicitArgumentRequired / MissingExplicitUniverse:
    ImplicitArgumentRequired
  ExpectedFunctionType:
    ExpectedPiType
  ExpectedSort / TooManyArguments / TooFewArguments:
    MachineTermElaborationError
  TypeMismatch with both expected_hash and actual_hash in the source diagnostic:
    TypeMismatch
  TypeMismatch without both expected_hash and actual_hash in the source diagnostic:
    MachineTermElaborationError
  CertificateRejected:
    VerifyFailed

Phase 4 MachineTacticDiagnosticKind -> MachineApiErrorKind:
  InvalidMachineProofState / UnknownMeta / InvalidMetaContext / InvalidMetaDependency /
  ProofExprScopeError / UnresolvedGoal / AmbiguousKernelEnvDecl:
    InvalidMachineProofState
  InvalidMachineTactic:
    InvalidCandidate
  InvalidMachineTermSource returned by Phase 4 after Phase 5 RawMachineTerm prepass has succeeded:
    InvalidMachineProofState
  UnknownGoal / GoalAlreadyAssigned:
    GoalNotOpen
  UnknownTacticHead / AmbiguousTacticHead / UnknownLocalName / AmbiguousLocalName /
  InvalidLocalHead / UnknownSimpRule / AmbiguousSimpRule / InvalidSimpRule /
  AmbiguousApplyArgument / TooManyApplyArguments / TooFewApplyArguments / SubgoalDataArgument:
    InvalidCandidate
  ExpectedFunctionType / ExpectedPiTarget:
    ExpectedPiType
  ExpectedEqTarget / AmbiguousRewriteRule:
    RewriteRuleInvalid
  UniverseArgumentMismatch with both expected_hash and actual_hash in the source diagnostic:
    TypeMismatch
  UniverseArgumentMismatch without both expected_hash and actual_hash in the source diagnostic:
    MachineTermElaborationError
  TypeMismatch with both expected_hash and actual_hash in the source diagnostic:
    TypeMismatch
  TypeMismatch without both expected_hash and actual_hash in the source diagnostic:
    MachineTermElaborationError
  ProofExprTypeMismatch with both expected_hash and actual_hash in the source diagnostic:
    TypeMismatch
  ProofExprTypeMismatch without both expected_hash and actual_hash in the source diagnostic:
    InvalidMachineProofState
  MissingExplicitArgument:
    ImplicitArgumentRequired
  SimpNoProgress:
    SimpNoProgress
  SimpStepLimitExceeded:
    BudgetExceeded
  TacticPrimitiveUnavailable:
    UnsupportedTactic
  InvalidInductionTarget:
    InductionTargetNotNat
  GoalLimitExceeded:
    TooManyGoals
  MetaLimitExceeded:
    BudgetExceeded
  TacticFuelExhausted { kind: ExprNode }:
    TooLargeTerm
  TacticFuelExhausted { kind: TacticStep | Whnf | Conversion | Rewrite | MetaAllocation }:
    BudgetExceeded
  InvalidCurrentDeclOrder / UncheckedCurrentDecl / CurrentDeclSignatureMismatch:
    InvalidCheckedCurrentDecl
  InvalidVerifiedImport:
    InvalidVerifiedImport
  InvalidTacticOption / UnsupportedTacticOption / InvalidEqFamily / InvalidNatFamily:
    InvalidMachineApiOptions
```

上の mapping は error kind の写像だけを固定します。
`phase`、`goal_id`、`tactic_kind`、`primary_name`、`primary_axiom_ref`、`expected_hash`、`actual_hash` は、mapping 後の
`MachineApiErrorKind` と source diagnostic の structured field から後述の option population rule で埋めます。

Diagnostic には、自然文よりも hash と enum を優先して入れます。
`diagnostic_hash` は response JSON 全体の hash ではありません。
Phase 5 は Phase 3 / Phase 4 / verify diagnostic を次の canonical object に写してから hash します。
この diagnostic canonical bytes は 5.1 の汎用 `Phase5 canonical primitive encoding` をそのまま使う payload ではなく、
下で明記する専用 schema です。
特に `error.kind`、`phase`、`tactic_kind` の string length は u32 byte length で encode し、5.1 の
UTF-8 string primitive が使う u64 byte length と混同してはいけません。
diagnostic schema version を変えずに u32/u64 のどちらかへ実装ごとに寄せることは禁止です。

```text
MachineApiDiagnostic canonical bytes:
  - tag "npa.phase5.api-diagnostic.v1"
  - error.kind lower_snake_case as UTF-8 bytes with u32 byte length
  - phase option
  - goal_id option
  - tactic_kind option
  - primary_name option
  - primary_axiom_ref option
  - expected_hash option
  - actual_hash option
  - retryable as 0x00 | 0x01
```

通常の deterministic top-level error response は、scheduler artifact response と batch per-candidate compact error
result を除き、必ず次の wrapper を使います。

```text
MachineApiErrorResponse:
  {
    "status": "error",
    "error": MachineApiErrorWire,
    ... endpoint-specific top-level fields explicitly allowed by that endpoint
  }
```

bare `MachineApiErrorWire` を top-level response として返してはいけません。
`MachineApiErrorWire` は wrapper 内の `error` object の shape です。
`phase`、`diagnostic_hash`、`retryable` は `error` object 内で必須です。
option field は canonical diagnostic で `some` の場合だけ `error` object に出し、`none` の field は omit します。
`candidate_hash` や `deterministic_budget_hash` のような endpoint-specific correlation field は同じ `error`
object に追加してよいですが、`MachineApiDiagnostic canonical bytes` には入りません。
`unchanged_state_fingerprint` のような endpoint-specific top-level field は、その endpoint が明示した場合だけ wrapper に追加します。

```text
MachineApiErrorWire:
  {
    "kind": MachineApiErrorKind lower_snake_case,
    "phase": MachineApiDiagnostic.phase,
    "diagnostic_hash": HashString,
    "retryable": false,
    "goal_id"?: GoalId,
    "tactic_kind"?: MachineTacticKind wire name,
    "primary_name"?: FullyQualifiedName,
    "primary_axiom_ref"?: MachineAxiomRefWire,
    "expected_hash"?: HashString,
    "actual_hash"?: HashString
  }
```

Batch per-candidate `status = "error"` result is a compact result item, not a top-level error response.
It must still expose the same canonical diagnostic fields as `MachineApiErrorWire`, with `kind` renamed to
`error_kind`, so that clients can recompute `diagnostic_hash` from the response item.

Option encoding はすべて `0x00 none` または `0x01 some + payload` です。
`phase` some payload は lower_snake_case UTF-8 bytes with u32 byte length です。
MVP の known phase は `request_validation`, `session_lookup`, `session_create`, `snapshot_lookup`, `candidate_validation`,
`machine_term_parse`, `machine_term_check`, `tactic_execution`, `theorem_search`,
`prompt_payload`, `replay_validation`, `replay_execution`, `kernel_check`, `certificate_generation`,
`certificate_verify` です。
unknown phase を diagnostic に入れる場合は protocol version を変えます。
`goal_id` some payload は Phase 4 `GoalId canonical bytes` です。
`tactic_kind` some payload は Phase 5 `MachineTacticKind` wire name の UTF-8 bytes with u32 byte length です。
MVP の allowed values と canonical order は `intro`, `exact`, `apply`, `rw`, `simp-lite`, `induction-nat` です。
Phase 4 Rust variant 名から機械変換してはいけません。
特に `MachineTacticCandidate::Rewrite` の diagnostic wire name は `"rewrite"` ではなく、candidate schema と同じ `"rw"` です。
`primary_name` some payload は fully-qualified canonical name bytes です。
`primary_axiom_ref` some payload は 5.1 の `MachineAxiomRefWire canonical bytes` です。
`expected_hash` / `actual_hash` some payload は `HashString` digest bytes です。
length は minimal unsigned LEB128 u32 で encode します。

`phase` は error kind だけから推測せず、次の endpoint/stage matrix で固定します。
同じ bad request に複数の失敗がある場合でも、各 endpoint の request validation 順序を先に適用し、
最初に確定した failure stage の phase を使います。
この matrix は failure が確定した後の phase mapping であり、endpoint 内の validation 実行順ではありません。
validation 実行順は各 endpoint の request / semantic validation order に従います。
request envelope の JSON object shape、duplicate key、required/optional field、primitive type、`HashString` grammar、
id grammar の検査は session / snapshot lookup より先に行います。
ここでいう request envelope は top-level object と、その endpoint が lookup 前 validation と明記した field だけです。
`/machine/tactics/run` の embedded candidate、batch の prefix 内 candidate、`/machine/replay` の plan / step / embedded
candidate / budget のように明示的に delayed validation とした payload は、この共通 envelope rule の対象外です。
session imports や current snapshot が必要な semantic validation は lookup 後にだけ行い、lookup が失敗した場合は
session-dependent validation を実行しません。
特に `POST /machine/sessions` では 5.2 の `SessionCreate semantic validation order` を優先し、
`root.theorem_type` の parse/check 行は stage 1-7 がすべて成功した後にだけ到達します。
import / callable-interface semantic validation と `root.theorem_type` の両方が不正な request では、
`root.theorem_type` を parse/check せず、先に失敗した stage の
`InvalidVerifiedImport` または `InvalidCheckedCurrentDecl` / `session_create` を返します。

```text
MachineApiDiagnostic phase matrix:
  JSON parse failure:
    request_validation
  request envelope duplicate key / top-level unknown field / required field omitted / null / primitive type error:
    request_validation

  UnknownSession on any endpoint:
    session_lookup

  POST /machine/sessions:
    InvalidSessionRequest caused by root request wire shape:
      request_validation
    InvalidVerifiedImport / InvalidCheckedCurrentDecl / InvalidMachineApiOptions caused by nested request wire shape:
      request_validation
    InvalidMachineApiOptions caused by kernel_check_profile validation or options semantic validation:
      session_create
    InvalidVerifiedImport caused by certificate / import closure semantic validation
    or imported callable interface table construction:
      session_create
    InvalidSessionRequest caused by root.module collision with import closure module,
    or root.theorem_name collision with direct import public ExportEntry.name:
      session_create
    InvalidCheckedCurrentDecl caused by checked current declaration semantic validation
    or checked current / current generated callable interface table construction:
      session_create
    DisallowedAxiom:
      session_create
    MachineTermParseError while checking root.theorem_type after stages 1-7 in 5.2 have succeeded:
      machine_term_parse
    MachineTermElaborationError / UnknownName / ImplicitArgumentRequired / TypeMismatch / ExpectedPiType
    while checking root.theorem_type after stages 1-7 in 5.2 have succeeded:
      machine_term_check
    InvalidMachineProofState while deriving RootTheoremTypeDependencyReport after root.theorem_type check:
      session_create
    InvalidMachineProofState:
      session_create

  DELETE /machine/sessions/{id}:
    invalid path id or non-empty body:
      request_validation
    UnknownSession:
      session_lookup

  POST /machine/snapshots/get:
    InvalidSnapshotRequest:
      request_validation
    UnknownSnapshot / StateFingerprintMismatch / InvalidMachineProofState while materializing stored snapshot:
      snapshot_lookup

  POST /machine/tactics/run:
    InvalidTacticRunRequest / InvalidBudget / InvalidSchedulerLimits:
      request_validation
    UnknownSnapshot / StateFingerprintMismatch / GoalNotOpen /
    InvalidMachineProofState while loading or validating stored snapshot:
      snapshot_lookup
    InvalidCandidate:
      candidate_validation
    InvalidMachineProofState caused by Phase 5 / Phase 4 adapter invariant failure after
    RawMachineTerm prepass has succeeded:
      candidate_validation
    MachineTermParseError:
      machine_term_parse
    MachineTermElaborationError / UnknownName / ImplicitArgumentRequired / TypeMismatch / ExpectedPiType:
      machine_term_check
    InvalidMachineProofState produced by Phase 4 tactic semantic validation or run_machine_tactic after Phase 4 candidate canonicalization:
      tactic_execution
    UnsupportedTactic / RewriteRuleInvalid / SimpNoProgress / InductionTargetNotNat /
    BudgetExceeded / TooManyGoals / TooLargeTerm:
      tactic_execution
    InvalidMachineProofState while materializing or storing the next snapshot after logical tactic success:
      tactic_execution

  POST /machine/tactics/batch:
    InvalidBatchPolicy / InvalidBudget / InvalidSchedulerLimits:
      request_validation
    UnknownSnapshot / StateFingerprintMismatch / GoalNotOpen /
    InvalidMachineProofState while loading or validating stored snapshot:
      snapshot_lookup
    per-candidate InvalidCandidate:
      candidate_validation
    per-candidate InvalidMachineProofState caused by Phase 5 / Phase 4 adapter invariant failure after
    RawMachineTerm prepass has succeeded:
      candidate_validation
    per-candidate MachineTermParseError:
      machine_term_parse
    per-candidate MachineTermElaborationError / UnknownName / ImplicitArgumentRequired /
    TypeMismatch / ExpectedPiType:
      machine_term_check
    per-candidate InvalidMachineProofState produced by Phase 4 tactic semantic validation or run_machine_tactic after Phase 4 candidate canonicalization:
      tactic_execution
    per-candidate tactic semantic errors:
      tactic_execution
    per-candidate InvalidMachineProofState while materializing or storing that candidate's next snapshot:
      tactic_execution

  POST /machine/search/for_goal:
    InvalidTheoremQuery caused by pure wire validation or allowed_modules session-dependent validation:
      request_validation
    UnknownSnapshot / StateFingerprintMismatch / GoalNotOpen /
    InvalidMachineProofState while materializing goal view:
      snapshot_lookup
    InvalidTheoremIndex:
      theorem_search

  POST /machine/prompt_payload:
    InvalidPromptPayloadRequest caused by pure wire validation, failed_candidates item schema validation,
    include_failed_candidates consistency validation, or allowed_modules session-dependent validation:
      request_validation
    UnknownSnapshot / StateFingerprintMismatch / GoalNotOpen /
    InvalidMachineProofState while materializing goal view:
      snapshot_lookup
    InvalidTheoremIndex produced by premise selection or selected premise statement rendering:
      theorem_search
    InvalidMachineProofState caused by PromptRenderedContent assembly after goal view and premise selection succeeded:
      prompt_payload

  POST /machine/replay:
    InvalidReplayPlan caused by request envelope JSON shape / top-level unknown field /
    required field omitted / null / primitive type error / id grammar / plan non-object:
      request_validation
    InvalidReplayPlan caused by plan object / step object structure, embedded candidate/budget wire shape,
    plan-internal hash-chain validation, or step deterministic_budget_hash validation:
      replay_validation
    SessionRootHashMismatch / StateFingerprintMismatch after plan wire / chain validation succeeds:
      replay_validation
    ReplayHashMismatch:
      replay_execution
    InvalidMachineProofState caused by Phase 5 / Phase 4 adapter invariant failure while replaying a step candidate:
      replay_execution
    InvalidMachineProofState while materializing final replay snapshot:
      replay_execution

  POST /machine/verify:
    InvalidVerifyRequest caused by request envelope, id/hash grammar, or unsupported mode:
      request_validation
    UnknownSnapshot / StateFingerprintMismatch /
    InvalidMachineProofState while extracting closed snapshot:
      snapshot_lookup
    InvalidVerifyRequest caused by non-empty open_goals, unresolved meta, or unresolved goal in the target snapshot:
      snapshot_lookup
    VerifyFailed during final kernel check before certificate construction:
      kernel_check
    VerifyFailed during certificate construction / dependency projection / source_index rewrite / canonical serialization:
      certificate_generation
    VerifyFailed during certificate verifier:
      certificate_verify
    DisallowedAxiom:
      certificate_verify
```

`MachineApiDiagnostic` の option population は次で固定します。
Per-kind overrides は Common より優先し、物理的に request body をどの順序で parse / pre-validate したかで
option field を変えてはいけません。
Context-specific overrides は、同じ error kind に対する Per-kind overrides より優先します。

```text
Common:
  phase:
    always some known phase after request dispatch
  retryable:
    false for every MachineApiDiagnostic in MVP
    accepted explicit scheduler_limits stops, 6.4 snapshot store quota/resource guard stops,
    and /machine/replay 12.1 scheduler stops are scheduler_artifact responses, not MachineApiDiagnostic
    server-local guards outside those endpoint contracts are transport/resource layer errors, not MachineApiDiagnostic
    scheduler_artifact responses MUST NOT set error.kind and MUST NOT produce diagnostic_hash
  goal_id:
    some iff the request or replay step contains a syntactically valid GoalId and the failing operation is tied to that goal
    none for invalid GoalId grammar, session/import/options validation, theorem index build not tied to one goal, and verify whole-proof failures
  tactic_kind:
    some iff the logical failing operation is candidate semantic validation / tactic execution for a recognized tactic kind
    none for request envelope errors, invalid / unknown tactic kind, search/prompt/session/verify errors
  primary_name:
    some iff the deterministic primary failed name is present in the structured source error
    and that name is a fully-qualified declaration name:
      UnknownName, RewriteRuleInvalid, Unknown/ambiguous tactic head mapped to InvalidCandidate, invalid simp/family reference mapped to InvalidMachineApiOptions
    none for MachineLocalName, local binder names, universe parameter names, invalid local names, and display-only names
    none otherwise
  primary_axiom_ref:
    some iff the error is DisallowedAxiom
    MVP DisallowedAxiom must always have a deterministic primary failed MachineAxiomRefWire;
    if an axiom dependency cannot be converted to MachineAxiomRefWire, return the corresponding malformed-report error instead
    none otherwise
  expected_hash / actual_hash:
    both some for TypeMismatch
    none for ExpectedPiType in MVP, even if an implementation can compute an expected or actual type hash
    none for both in all other MVP errors

Context-specific overrides:
  POST /machine/sessions root.theorem_type Machine Surface parse/check errors:
    applies to MachineTermParseError / MachineTermElaborationError / UnknownName /
    ImplicitArgumentRequired / TypeMismatch / ExpectedPiType emitted while building CheckedMachineProofRoot
    goal_id none; tactic_kind none
    primary_name follows the source diagnostic only for a fully-qualified declaration name
    expected_hash and actual_hash both some for TypeMismatch
    expected/actual none for ExpectedPiType and the other root theorem_type errors
  POST /machine/tactics/run Phase 4 tactic semantic validation / run_machine_tactic InvalidMachineProofState:
    applies after open goal lookup and Phase 4 candidate canonicalization, before logical success is accepted
    goal_id some; tactic_kind some iff candidate kind is recognized; primary_name none; expected/actual none
  POST /machine/tactics/run RawMachineTerm prepass / Phase 4 adapter invariant failure:
    applies after open goal lookup but before candidate_hash can be computed
    goal_id some; tactic_kind some iff candidate kind is recognized; primary_name none; expected/actual none
  POST /machine/tactics/run next snapshot materialization / store consistency InvalidMachineProofState:
    applies after Phase 4 candidate canonicalization and logical tactic success
    goal_id some; tactic_kind some iff candidate kind is recognized; primary_name none; expected/actual none
  POST /machine/tactics/batch per-candidate Phase 4 tactic semantic validation / run_machine_tactic InvalidMachineProofState:
    applies only to the evaluated candidate after open goal lookup and Phase 4 candidate canonicalization
    goal_id some; tactic_kind some iff candidate kind is recognized; primary_name none; expected/actual none
  POST /machine/tactics/batch per-candidate RawMachineTerm prepass / Phase 4 adapter invariant failure:
    applies only to the evaluated candidate after open goal lookup but before candidate_hash can be computed
    goal_id some; tactic_kind some iff candidate kind is recognized; primary_name none; expected/actual none
  POST /machine/tactics/batch per-candidate next snapshot materialization / store consistency InvalidMachineProofState:
    applies only to the evaluated candidate whose logical tactic transition succeeded
    goal_id some; tactic_kind some iff candidate kind is recognized; primary_name none; expected/actual none
  POST /machine/replay step candidate Phase 4 adapter invariant failure:
    applies after ReplayPlan validation accepts a syntactically valid step.goal_id but before
    replay can recompute candidate_hash for that step
    goal_id some(step.goal_id); tactic_kind some iff step.candidate kind is recognized;
    primary_name none; expected/actual none
  POST /machine/search/for_goal goal view materialization InvalidMachineProofState:
    applies after request validation and open goal lookup tie the failure to the requested goal
    goal_id some; tactic_kind none; primary_name none; expected/actual none
  POST /machine/prompt_payload goal/context rendering or PromptRenderedContent assembly InvalidMachineProofState:
    applies after request validation and open goal lookup tie the failure to the requested goal
    goal_id some; tactic_kind none; primary_name none; expected/actual none
  DisallowedAxiom from session_create import/current-decl subset check,
  session_create root theorem type subset check, or verify certificate axiom subset check:
    primary failed axiom is the first disallowed MachineAxiomRefWire in MachineAxiomRefWire canonical order
    goal_id none; tactic_kind none; primary_name some(primary failed axiom fully-qualified name);
    primary_axiom_ref some(primary failed axiom MachineAxiomRefWire); expected/actual none

Per-kind overrides:
  Request / lookup / non-tactic validation:
    UnknownSession / UnknownSnapshot / StateFingerprintMismatch / SessionRootHashMismatch /
    InvalidVerifiedImport / InvalidCheckedCurrentDecl / InvalidMachineApiOptions / InvalidSessionRequest /
    InvalidSnapshotRequest / InvalidMachineProofState / InvalidTacticRunRequest / InvalidTheoremIndex / InvalidTheoremQuery /
    InvalidPromptPayloadRequest / InvalidBatchPolicy /
    InvalidSchedulerLimits / InvalidReplayPlan / InvalidVerifyRequest /
    InvalidBudget / VerifyFailed:
      goal_id none; tactic_kind none; primary_name none unless the structured source error explicitly names a fully-qualified declaration;
      expected/actual none
  GoalNotOpen:
    goal_id some; tactic_kind none; primary_name none; expected/actual none
  ReplayHashMismatch:
    replay step execution mismatch after ReplayPlan validation succeeds:
      goal_id some(step.goal_id); tactic_kind some iff step.candidate kind is recognized before failure;
      primary_name none; expected/actual none
    replay plan structure / hash-chain validation failure is not ReplayHashMismatch;
    it is InvalidReplayPlan and is evaluated before SessionRootHashMismatch / StateFingerprintMismatch;
    only well-formed plan session-binding failures use SessionRootHashMismatch / StateFingerprintMismatch
    ReplayHashMismatch never copies source diagnostic primary_name / expected_hash / actual_hash
    from the underlying candidate error in MVP
  InvalidCandidate:
    goal_id some iff candidate validation is reached after open goal lookup; tactic_kind some iff candidate kind is recognized;
    primary_name follows the source diagnostic only if it names a fully-qualified declaration; local head/local-name failures omit primary_name;
    expected/actual none
  MachineTermParseError:
    outside session root override: goal_id some; tactic_kind some if the enclosing candidate kind is recognized; expected/actual none
  MachineTermElaborationError / ImplicitArgumentRequired / UnknownName:
    outside session root override: goal_id some; tactic_kind some if recognized;
    primary_name follows the source diagnostic only if it names a fully-qualified declaration;
    UnknownLocalName and other local-name failures omit primary_name; expected/actual none
  TypeMismatch:
    outside session root override: goal_id some; tactic_kind some; primary_name none; expected_hash some; actual_hash some
  ExpectedPiType:
    outside session root override: goal_id some; tactic_kind some; primary_name none; expected/actual none
  UnsupportedTactic / RewriteRuleInvalid / SimpNoProgress / InductionTargetNotNat / BudgetExceeded /
  TooManyGoals / TooLargeTerm:
    goal_id some, tactic_kind some, primary_name none, expected/actual none
```

If Phase 3 / Phase 4 / verify adds a new structured error field that would change any option above, Phase 5 must either
map it according to this table without changing existing hashes or bump the diagnostic schema version.

`diagnostic_hash = sha256(MachineApiDiagnostic canonical bytes)` です。
`diagnostic_hash` 自身、`candidate_hash`、`deterministic_budget_hash`、pretty/machine rendered text、
source span、natural language message、AI trace、score、`suggestions` は hash 入力に含めません。
Phase 3 / Phase 4 diagnostic に上の field へ写せない structured data が増えた場合は、
schema version を変えるか、hash 入力に入れない display-only diagnostic field として明記してから追加します。
次は display-only source diagnostic payload の例であり、top-level `MachineApiErrorWire` ではありません。

```json
{
  "kind": "type_mismatch",
  "phase": "machine_term_check",
  "expected": {
    "core_hash": "sha256:...",
    "machine": "Eq.{1} Nat n n"
  },
  "actual": {
    "core_hash": "sha256:...",
    "machine": "Nat"
  },
  "suggestions": [
    {
      "kind": "insert_explicit_argument",
      "replacement_machine": "@Eq.refl.{1} Nat n"
    }
  ]
}
```

`suggestions` は信用しません。
修復候補として再投入し、Machine Surface Complete mode と tactic execution に通った場合だけ採用します。

---

# 12. Replay Contract

## 12.1 Replay plan

Phase 7 の探索結果は、後から replay できる必要があります。
Phase 5 AI は、成功した delta chain を表す wire contract として `MachineReplayPlan` を定義します。
MVP では `MachineReplayPlan` を自動生成して返す専用 endpoint は持ちません。
Phase 7 / client は `/machine/tactics/run` または `/machine/tactics/batch` の success response と、
自分が送った `goal_id`、wire `candidate`、`deterministic_budget` payload を保存して
`MachineReplayPlan` を組み立てます。
server-side plan export を追加する場合は non-MVP endpoint とし、どの success history を保存するか、
candidate payload をどこから復元するか、plan canonical bytes と store lifetime をこの文書に追記してから実装します。

```rust
struct MachineReplayPlan {
    protocol_version: MachineApiVersion,
    session_root_hash: Hash,
    initial_state_fingerprint: Hash,
    steps: Vec<MachineReplayStep>,
    final_state_fingerprint: Hash,
}

struct MachineReplayStep {
    previous_state_fingerprint: Hash,
    goal_id: GoalId,
    candidate: MachineTacticCandidate,
    deterministic_budget: MachineDeterministicBudget,
    candidate_hash: Hash,
    deterministic_budget_hash: Hash,
    proof_delta_hash: Hash,
    next_state_fingerprint: Hash,
}
```

Replay plan は tactic 再実行に必要な step payload について自己完結していなければなりません。
ただし MVP replay は `session_root_hash` が一致する current session を要求し、その session の initial snapshot を
開始 state とします。
`candidate_hash` と `deterministic_budget_hash` は照合用であり、再実行に必要な wire `candidate` と
`deterministic_budget` payload も step に含めます。
`step.candidate` は `/machine/tactics/run` の `candidate` と同じ external raw `MachineTacticCandidate` wire payload です。
Phase 4 内部の checked `MachineTactic`、`MachineTermSource.canonical_hash`、score、metadata を replay plan に入れてはいけません。
JSON field order は replay plan の意味に影響せず、replay execution 時に validation / canonicalization をやり直してから
`candidate_hash` を照合します。
`session_root_hash` は 5.3 の `SessionRoot canonical bytes` から計算します。
MVP の replay は current session の `initial_snapshot` からだけ開始します。
`/machine/replay` は request envelope validation と session lookup の後、まず plan wire / chain validation を実行し、
その後に session binding validation を実行します。
plan wire / chain validation が失敗した場合は、plan の `session_root_hash` や `initial_state_fingerprint` が
current session と一致しない場合でも `InvalidReplayPlan` を優先します。

replay plan wire / chain validation は再実行前に次を検査します。

```text
ReplayPlan wire / chain validation:
  - protocol_version equals current MachineApiVersion
  - steps length <= 4096
  - if steps is empty:
      final_state_fingerprint equals initial_state_fingerprint
  - if steps is non-empty:
      steps[0].previous_state_fingerprint equals initial_state_fingerprint
      for every adjacent pair i, i+1:
        steps[i].next_state_fingerprint equals steps[i+1].previous_state_fingerprint
      final_state_fingerprint equals last step next_state_fingerprint
  - every step deterministic_budget_hash equals hash(step.deterministic_budget canonical bytes)
```

plan wire / chain validation に成功した後で、次の session binding validation を実行します。

```text
ReplayPlan session binding validation:
  - plan.session_root_hash equals current session recomputed session_root_hash
  - plan.initial_state_fingerprint equals current session initial snapshot state_fingerprint
```

session binding validation の `session_root_hash` 不一致は `SessionRootHashMismatch`、
`initial_state_fingerprint` 不一致は `StateFingerprintMismatch` です。
それ以外の plan object / step object / embedded payload / hash-chain validation failure は `InvalidReplayPlan` です。
`candidate_hash` は pre-validation では照合しません。
replay executor は current replay state を持ちながら step を順に処理し、各 step で次を実行します。

```text
Replay step execution:
  1. current_state_fingerprint == step.previous_state_fingerprint を確認する
  2. current replay state から step.goal_id の open goal を取得する
  3. step.candidate に Phase 5 RawMachineTerm prepass を適用する
  4. step.candidate を current replay state / goal に対して validate_machine_tactic_candidate する
  5. 再計算 candidate_hash が step.candidate_hash と一致することを確認する
  6. step.deterministic_budget で tactic を実行する
  7. 実行結果が success であることを確認する
  8. 再計算 proof_delta_hash と next_state_fingerprint が step の hash と一致することを確認する
```

Replay step execution step 3 の prepass failure は、plan に保存された candidate payload では同じ
`candidate_hash` を再構成できないため `ReplayHashMismatch` です。
この場合も Phase 3 source diagnostic payload を外部 response の error として返しません。
Replay step execution の hash 不一致、candidate validation result 不一致、goal lookup failure、
tactic execution が deterministic error を返した場合は、明示的な Phase 5 / Phase 4 adapter invariant failure を除き
`ReplayHashMismatch` です。
replay は成功 delta chain の検証 API なので、step 実行中に返った元の `MachineApiErrorKind` を外部 response の
`error.kind` として返しません。
MVP replay error response に step 実行中の source diagnostic / debug diagnostic payload を添付してはいけません。
将来 debug field を追加する場合は、field name、wire shape、ordering、redaction、hash 除外規則をこの文書で固定します。
`ReplayHashMismatch` response の `diagnostic_hash` は常に `ReplayHashMismatch` の `MachineApiDiagnostic` から計算します。
Phase 5 / Phase 4 adapter invariant failure を `InvalidMachineProofState` として返す場合は、その
`InvalidMachineProofState` diagnostic から `diagnostic_hash` を計算します。
各 step を再実行した後の `proof_delta_hash` または `next_state_fingerprint` が plan と一致しない場合は
`ReplayHashMismatch` です。
`/machine/replay` は MVP では request `scheduler_limits` を受け取りません。
server process guard の wall-clock timeout、memory limit、または resource stop が replay 中に発生した場合は
deterministic replay error ではなく、次の retryable scheduler stop response を返します。
この response は `error.kind` と `diagnostic_hash` を持たず、`ReplayHashMismatch` に写してはいけません。

```json
{
  "status": "scheduler_stopped",
  "scheduler_artifact": {
    "kind": "timeout",
    "scope": "replay",
    "retryable": true
  }
}
```

任意の中間 snapshot から replay を開始する API は non-MVP とし、追加する場合は initial snapshot payload または
session snapshot store 依存を wire contract に明記します。
server が content-addressed store を追加する場合でも、MVP の wire format は payload 埋め込みを必須にします。

## 12.2 replay API

```json
POST /machine/replay
{
  "session_id": "msess_001",
  "plan": {
    "protocol_version": "npa.machine-api.v1",
    "session_root_hash": "sha256:...",
    "initial_state_fingerprint": "sha256:...",
    "steps": [
      {
        "previous_state_fingerprint": "sha256:...",
        "goal_id": "g0",
        "candidate": {
          "kind": "intro",
          "name": "n"
        },
        "deterministic_budget": {
          "max_tactic_steps": 64,
          "max_whnf_steps": 10000,
          "max_conversion_steps": 10000,
          "max_rewrite_steps": 100,
          "max_meta_allocations": 8,
          "max_expr_nodes": 20000
        },
        "candidate_hash": "sha256:...",
        "deterministic_budget_hash": "sha256:...",
        "proof_delta_hash": "sha256:...",
        "next_state_fingerprint": "sha256:..."
      }
    ],
    "final_state_fingerprint": "sha256:..."
  }
}
```

`/machine/replay` request object は次の field だけを持ちます。

```text
required:
  session_id
  plan
optional:
  none
```

top-level unknown field、duplicate key、required field omitted、`null`、`session_id` の non-string、
invalid `SessionId` grammar、
`plan` の non-object は `InvalidReplayPlan` です。
この段階の `InvalidReplayPlan` diagnostic phase は `request_validation` です。
request envelope validation 後、`session_id` が存在しない場合は `UnknownSession` です。
`plan` object は `protocol_version`, `session_root_hash`, `initial_state_fingerprint`, `steps`,
`final_state_fingerprint` だけを持ちます。
plan object の unknown field、duplicate key、required field omitted、`null`、`protocol_version` が
`"npa.machine-api.v1"` 以外、invalid `HashString`、`steps` の non-array、`steps.length > 4096` は
`InvalidReplayPlan` です。
MVP の protocol-level replay cap は `steps.length <= 4096` です。
これを超える replay は deterministic replay validation error であり、transport / resource layer stop ではありません。
各 step object は `previous_state_fingerprint`, `goal_id`, `candidate`, `deterministic_budget`,
`candidate_hash`, `deterministic_budget_hash`, `proof_delta_hash`, `next_state_fingerprint` だけを持ちます。
step object の unknown field、duplicate key、required field omitted、`null`、invalid `HashString`、
invalid `GoalId` grammar、`candidate` の non-object、`deterministic_budget` の non-object は
`InvalidReplayPlan` です。
replay plan 内の embedded `candidate` と `deterministic_budget` は wire shape だけを pre-validate します。
embedded candidate pre-validation は session binding より前に行うため、TacticHead / SimpRule / local name の解決や
Level parameter が current root `universe_params` に含まれるかどうかの検査は実行しません。
この段階で `InvalidReplayPlan` になる embedded candidate failure は、candidate non-object、duplicate key、
unknown field、required field omitted、field value `null` except `CandidateApplyArg.subgoal.name_hint`、
unknown `kind`、variant field set mismatch、7.0 variant schema に定義されていない correlation/hash field、
`RawMachineTerm` wire object shape violation、invalid `HashString` grammar、
invalid `FullyQualifiedName` grammar、non-renderable `MachineSurfaceRenderableName`、
invalid `SimpRuleRef.direction`、unknown `CandidateApplyArg.mode`、session-independent な invalid
`Level` source syntax、invalid `MachineLocalName` wire grammar に限ります。
replay embedded candidate は 7.1 の candidate schema と同じ nullable field exception を使い、
`CandidateApplyArg.subgoal.name_hint = null` は `InvalidReplayPlan` にしてはいけません。
TacticHead / SimpRuleRef の name と hash が wire grammar と renderability check には通るが current replay state で
unknown / ambiguous / scope 外になる場合、または `Level` param が current root `universe_params` に存在しない場合は、
plan wire shape failure ではありません。
plan wire / chain / budget-hash validation と session binding が成功した後、replay execution stage で
current replay state に対して candidate canonicalization をやり直したときの mismatch として `ReplayHashMismatch` です。
embedded budget の field set / integer shape failure も `InvalidReplayPlan` です。
plan object / step object / embedded candidate / embedded budget の validation は request envelope validation と
session lookup の後に実行し、この段階の `InvalidReplayPlan` diagnostic phase は `replay_validation` です。
この replay validation stage の優先順位は、plan object / step object / embedded candidate / embedded budget の
wire shape、plan-internal hash chain、各 step の `deterministic_budget_hash`、session_root_hash binding、
initial_state_fingerprint binding の順です。
前段の wire shape / hash-chain / budget-hash validation が失敗した場合は、後段の
`SessionRootHashMismatch` / `StateFingerprintMismatch` ではなく `InvalidReplayPlan` を返します。
Phase 5 `RawMachineTerm` prepass、Phase 4 candidate canonicalization 後の TacticHead / SimpRule / local /
argument validation、term elaboration / type check、tactic execution は replay execution stage で
current replay state に対して実行します。
Replay execution でも 7.0 と同じ Phase 5 `RawMachineTerm` prepass を各 step candidate に適用します。
この prepass で Phase 3 `canonicalize_machine_term_source(source)` が失敗した場合は、
plan に保存された candidate payload では同じ `candidate_hash` を再構成できないため `ReplayHashMismatch` です。
この場合、Phase 3 の source diagnostic は response error payload として外へ出しません。
prepass 成功後に Phase 4 が `InvalidMachineTermSource` を返す場合は plan mismatch ではなく
Phase 5 / Phase 4 adapter invariant failure として `InvalidMachineProofState`、phase `replay_execution` です。
この stage の post-canonical `InvalidCandidate`、term elaboration / type check error、tactic deterministic error は
すべて `ReplayHashMismatch` です。
`/machine/replay` は MVP では `scheduler_limits` field を受け取らず、存在すれば `InvalidReplayPlan` です。

レスポンス:

```json
{
  "status": "ok",
  "final_snapshot_id": "mst_dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
  "final_state_fingerprint": "sha256:..."
}
```

`/machine/replay` success は replay で得た final proof state を session snapshot store に materialize してから返します。
`final_snapshot_id` は `final_state_fingerprint` から 6.0 の `snapshot_id` 規則で導出した handle です。
response を返した後、同じ `session_id`、`final_snapshot_id`、`final_state_fingerprint` で
`/machine/snapshots/get` を呼べなければなりません。
`steps = []` の場合は session の `initial_snapshot` をそのまま final snapshot として返します。
`steps` が non-empty の場合でも、保存される final snapshot は path-dependent な `prior_delta_hash` を持ちません。
最後の step の `proof_delta_hash` は replay plan 検証用の値であり、snapshot identity や
`/machine/snapshots/get` response には入りません。
MVP replay が新規に session snapshot store へ作成または更新してよい entry は final snapshot だけです。
中間 step の snapshot payload は replay executor の in-memory state としてだけ保持し、replay の副作用として
`/machine/snapshots/get` 可能な handle を新規作成してはいけません。
中間 snapshot が replay 前から store に存在していた場合でも、replay はそれを削除、上書き、照合しません。
final snapshot と同じ `snapshot_id` で既存 payload がある場合は、replay final executable state から再計算した
`state_fingerprint` が `final_state_fingerprint` と一致し、既存 entry の `executable_state_payload` から再計算した
`state_fingerprint` も `final_state_fingerprint` と一致し、既存 entry の executable state から newly materialized した
`StoredSnapshotView canonical bytes` が既存 entry の `materialized_view_payload` bytes と byte-for-byte に一致し、
かつ replay final state から newly materialized した `StoredSnapshotView canonical bytes` も同じ
`materialized_view_payload` bytes と byte-for-byte に一致する場合だけ再利用できます。
既存 final snapshot の executable payload が欠けている、既存 executable state と既存 materialized view が矛盾する、
または replay で得た final payload が既存 final snapshot の `StoredSnapshotView canonical bytes` と矛盾する場合は
session store corruption として
`InvalidMachineProofState` を返し、diagnostic phase は `replay_execution` です。

Replay は proof trace を信用するのではなく、各 step を再実行して hash を照合します。

---

# 13. Verify Handoff

open goals が空になっても、それだけでは verified ではありません。

```text
open_goals = []
  ↓
extract_closed_machine_theorem_decl
  ↓
kernel check
  ↓
certificate generation
  ↓
certificate verifier
  ↓
verified
```

API:

```json
POST /machine/verify
{
  "session_id": "msess_001",
  "snapshot_id": "mst_dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
  "state_fingerprint": "sha256:...",
  "mode": "certificate"
}
```

`/machine/verify` request object は次の field だけを持ちます。

```text
required:
  session_id
  snapshot_id
  state_fingerprint
  mode
optional:
  none
```

top-level unknown field、duplicate key、required field omitted、`null`、`session_id` / `snapshot_id` /
`state_fingerprint` / `mode` の non-string、invalid `SessionId` grammar、invalid `SnapshotId` grammar、
invalid `HashString` は `InvalidVerifyRequest` です。
MVP の `mode` は `"certificate"` だけです。
それ以外の mode string は `InvalidVerifyRequest` として拒否します。
request envelope validation 後、`session_id` が存在しない場合は `UnknownSession` です。
その後の snapshot lookup は 6.1 の session-scoped lookup order に従い、missing entry は `UnknownSnapshot`、
stored snapshot self-check 失敗は `InvalidMachineProofState`、stored snapshot の `state_fingerprint` と
request の `state_fingerprint` が一致しない場合は `StateFingerprintMismatch` です。
対象 snapshot に open goals が残っている場合、または `extract_closed_machine_theorem_decl` が未解決 meta /
未解決 goal を検出した場合は `InvalidVerifyRequest` です。
request validation が通った後の kernel check、certificate generation、certificate verifier、axiom subset check の失敗は
`VerifyFailed` または `DisallowedAxiom` です。
request envelope、id/hash grammar、unsupported mode による `InvalidVerifyRequest` の diagnostic phase は
`request_validation` です。
snapshot lookup 後に判明する non-empty `open_goals`、未解決 meta、未解決 goal による
`InvalidVerifyRequest` の diagnostic phase は `snapshot_lookup` です。
`VerifyFailed` は失敗箇所に応じて `kernel_check`、`certificate_generation`、または `certificate_verify` を使います。
final proof term / prior declaration の kernel check が失敗した場合は `kernel_check` です。
SourceIndexToCertificateDeclIndex 構築、current-module ref rewrite、dependency / axiom ref の certificate-local 逆写像、
Phase 2 certificate payload projection、canonical serialization、生成 certificate hash の自己整合性確認が失敗した場合は
`certificate_generation` です。
生成済み certificate bytes を Phase 2 certificate verifier に渡した後の拒否は `certificate_verify` です。
Phase 2 verifier が accepted response を返した後の verifier output projection failure は次で固定します。
verifier output 自体から root theorem declaration の axiom dependencies または module-level `axiom_report` を
一意に取得できない、verifier output の certificate-local axiom ref が malformed である、または verifier output の
axiom ref を generated certificate current-module context で `MachineAxiomRefWire` に変換できない場合は
`VerifyFailed`、phase `certificate_verify` です。
一方、verifier output から変換した axiom set と、Phase 5 が certificate projection に使った
`RootTheoremDependencyReport` または全 declaration report union が一致しない場合は、
certificate construction / projection の自己整合性 failure として `VerifyFailed`、phase `certificate_generation` です。

verify が生成する Phase 2 module certificate は、current session の root module に対する 1 つの canonical module です。
root theorem だけを単独 certificate にしてはいけません。
root theorem の proof term と prior checked current declaration は、Phase 5 session 内では source_index 座標の
`GlobalRef::Local` / `LocalGenerated` を使います。
Phase 2 certificate では declaration index が Phase 2 canonical declaration order の index なので、
verify handoff は source_index 座標を certificate-local index へ rewrite してから certificate bytes を作ります。
certificate import table と declaration order は次に固定します。

```text
Verify certificate construction:
  1. certificate module name = session.root.module
  2. certificate import table =
       session direct imports in canonical order by (module, export_hash, certificate_hash)
       This is the same order used by CoreDeclPackage GlobalRef::Imported(import_index, ...).
       The certificate builder MUST NOT reorder imports unless it also rewrites every GlobalRef::Imported
       in checked_current_decls and extracted root theorem before hashing.
       MVP chooses no rewrite and therefore uses the session canonical direct import order as-is.
  3. build a source-indexed declaration set =
       checked_current_decls.core_decl with source_index 0, 1, ..., root.source_index - 1,
       plus the extracted root theorem declaration with source_index root.source_index.
  4. project each declaration's dependency report to a temporary dependency graph in source_index
     coordinates, excluding the same InductiveDecl bundle-internal refs as session validation.
     Graph edge rule:
       imported dependencies do not create current-module ordering edges
       MachineDependencyRefWire::current_module(source_index) creates an edge to that source_index
       MachineDependencyRefWire::current_generated(parent_source_index, generated_name)
         creates an edge to parent_source_index; generated artifacts are not graph nodes
  5. compute Phase 2 certificate declaration order with the Phase 2 rule:
       dependency order, and same-rank tie-break by declaration name canonical bytes.
     This order may differ from source_index order.
  6. construct SourceIndexToCertificateDeclIndex:
       every source_index in the declaration set maps to its 0-based index in the Phase 2
       certificate declaration order.
       Define internal root_decl_index =
       SourceIndexToCertificateDeclIndex[root.source_index], not root.source_index itself.
       This value is used for certificate-local rewrite and certificate hash construction.
       The MVP verify response does not expose root_decl_index; if a later API adds that field,
       it must return this certificate declaration index, not the Phase 5 source_index.
  7. construct CertificateDeclIndexToSourceIndex as the exact inverse:
       every certificate declaration index in the generated Phase 2 declaration order maps back
       to exactly one Phase 5 source_index from the declaration set.
       The inverse is used only when converting verifier output, certificate-local axiom refs,
       or diagnostic dependency refs back to API wire responses. It is not serialized into the
       certificate and must not be replaced by source_index identity assumptions.
       A missing, duplicate, or out-of-range inverse entry is VerifyFailed.
  8. rewrite every current-module reference in checked_current_decls, extracted root theorem,
     projected dependency entries, and projected axiom refs:
       GlobalRef::Local(source_index)
         -> GlobalRef::Local(SourceIndexToCertificateDeclIndex[source_index])
       GlobalRef::LocalGenerated(parent_source_index, generated_name)
         -> GlobalRef::LocalGenerated(SourceIndexToCertificateDeclIndex[parent_source_index],
                                      generated_name)
     Imported refs are unchanged because the certificate import table order is unchanged.
  9. after rewrite, every Phase 2 current-module dependency must refer to an earlier
     certificate declaration index unless it is the allowed same-InductiveDecl bundle-internal ref.
  10. generated constructor / recursor artifacts do not receive independent source_index values.
     They remain generated artifacts of their parent InductiveDecl, exactly as in session validation.
  11. no future current declaration, unchecked declaration, tactic metadata, prompt, search score,
     scheduler artifact, or failed candidate list is included in the certificate payload.
```

`extract_closed_machine_theorem_decl` は root theorem name / universe_params / theorem type と final proof body から
Phase 5 source_index 座標の root `TheoremDecl` を作ります。
MVP の root theorem `opacity` は常に Phase 2 `opaque` とします。
request は root theorem opacity を指定できず、この protocol version で transparent / reducible theorem を生成してはいけません。
将来 root theorem の opacity を選べるようにする場合は、request schema、`session_root_hash`、verify response、
certificate generation rule を同時に version up します。
verify handoff は SourceIndexToCertificateDeclIndex による current-module ref rewrite 後の Phase 2 `TheoremDecl` を、
canonical certificate import table order と certificate-local declaration index scope で kernel check します。
checked current declarations の package report は certificate payload には直接入れません。
ただし certificate generation は、session create で検証済みの `CurrentDeclDependencyReport` と同じ
dependency / axiom derivation rule を使って prior declarations と root theorem の Phase 2 dependency payload を作ります。
再計算した dependency / axiom payload が session に保存された checked current report と矛盾する場合は
`VerifyFailed` です。

Phase 5 の `CurrentDeclDependencyReport` は Phase 2 payload そのものではありません。
certificate generation では、declaration kind ごとに次の投影だけを使います。
root theorem については、final proof extraction 後に `RootTheoremDependencyReport` を一時的に作ります。
これは `CurrentDeclDependencyReport` と同じ dependency / axiom fields を持ちますが、kind は常に `TheoremDecl`、
name / universe_params / type は session root、proof は final snapshot から抽出した proof body です。
`RootTheoremDependencyReport` は session_root_hash には入りません。verify response と certificate payload を作るためだけに使います。

```text
CurrentDeclDependencyReport -> Phase 2 payload projection:
  common:
    - direct_dependency_entries は bundle-internal Local / LocalGenerated ref を除外済みでなければならない
    - report.direct_dependency_entries は Phase 5 wire refs のまま certificate payload にコピーしない
    - imported dependency は verify certificate import table の import_index を使って
      Phase 2 GlobalRef::Imported(import_index, name, decl_interface_hash) へ写す
    - current-module ordinary dependency は
      MachineDependencyRefWire::current_module.source_index を SourceIndexToCertificateDeclIndex で引き、
      Phase 2 certificate declaration index として GlobalRef::Local(decl_index) へ写す
    - current-module generated dependency は
      MachineDependencyRefWire::current_generated.parent_source_index と generated name から
      parent certificate declaration index を引き、
      GlobalRef::LocalGenerated(parent_decl_index, generated_name) へ写す
    - dependency entry list は Phase 2 DependencyEntry canonical bytes で sort / dedup する
    - axiom_dependencies は 5.1 の MachineAxiomRefWire canonical order と同じ集合から
      Phase 2 certificate-local AxiomRef list へ逆写像して作り、Phase 2 AxiomRef canonical bytes で sort / dedup する
    - dependency または axiom ref を certificate-local Phase 2 ref へ一意に写せない場合は VerifyFailed
    - 上の projection と SourceIndexToCertificateDeclIndex rewrite 後の
      Phase 2 DeclInterfacePayload から decl_interface_hash を Phase 2 rule で再計算する
    - every DeclCertificatePayload.decl_interface_hash =
      this recomputed certificate-local decl_interface_hash
    - Phase 5 checked signature.decl_interface_hash は session validation 用の
      照合値であり、generated certificate payload にはそのままコピーしてはならない

  AxiomDecl:
    - DeclInterfacePayload.public_dependency_entries =
        dependencies appearing in the axiom type only
    - DeclCertificatePayload.axiom_dependencies =
        { self axiom } only
    - DeclCertificatePayload.dependency entries =
        none

  DefDecl:
    - DeclInterfacePayload.public_dependency_entries =
        Phase 2 public dependency entries for the definition interface
    - DeclCertificatePayload.dependency entries =
        converted Phase 2 DependencyEntry list from report.direct_dependency_entries
    - DeclCertificatePayload.axiom_dependencies =
        converted Phase 2 AxiomRef list from report.axiom_dependencies

  TheoremDecl:
    - DeclInterfacePayload.opacity =
        opaque
    - DeclInterfacePayload.public_dependency_entries =
        dependencies appearing in the theorem type only
    - DeclInterfacePayload.axiom_dependencies =
        converted Phase 2 AxiomRef list from report.axiom_dependencies
    - DeclCertificatePayload.proof_hash =
        hash of the checked proof body; for the root theorem this is the extracted final proof,
        and for prior checked theorem declarations this is the proof already present in core_decl
    - DeclCertificatePayload.dependency entries =
        converted Phase 2 DependencyEntry list from report.direct_dependency_entries
    - DeclCertificatePayload does not carry a separate axiom_dependencies field for theorem;
      theorem axiom dependencies enter through DeclInterfacePayload as specified by Phase 2

  InductiveDecl:
    - DeclInterfacePayload.public_dependency_entries =
        dependencies from parameters, indices, constructor types, recursor types, and generated
        computation rules, excluding bundle-internal refs
    - DeclCertificatePayload.dependency entries =
        converted Phase 2 DependencyEntry list from report.direct_dependency_entries
    - DeclCertificatePayload.axiom_dependencies =
        converted Phase 2 AxiomRef list from report.axiom_dependencies
```

`report.direct_dependency_entries` をそのまま `DeclInterfacePayload.public_dependency_entries` として使ってはいけません。
Theorem proof dependency や opaque body dependency は certificate payload には必要でも public interface ではないため、
Phase 2 の kind 別 public dependency rule で再計算します。

生成 certificate の `ExportBlock` は Phase 2 canonical rule で、`checked_current_decls.core_decl` と
root theorem declaration から作ります。
どの current declaration が export entry になるかは Phase 2 `ExportBlock` rule だけで決め、
Phase 5 独自の public/private flag は持ちません。
後続 session で `import_payload` を direct import にした場合、Phase 2 が export entry にした prior checked current declaration と
root theorem は、その module の verified environment / export block 経由で参照可能でなければなりません。

成功:

```json
{
  "status": "verified",
  "root_decl_interface_hash": "sha256:...",
  "root_decl_certificate_hash": "sha256:...",
  "root_axioms_used": [],
  "module_export_hash": "sha256:...",
  "module_certificate_hash": "sha256:...",
  "module_axioms_used": [],
  "certificate": {
    "encoding": "npa.certificate.canonical.v0.1.hex",
    "bytes": "..."
  },
  "dependency_import_closure": [
    {
      "module": "Std.Init",
      "expected_export_hash": "sha256:...",
      "expected_certificate_hash": "sha256:...",
      "certificate": {
        "encoding": "npa.certificate.canonical.v0.1.hex",
        "bytes": "..."
      }
    },
    {
      "module": "Std.Nat.Basic",
      "expected_export_hash": "sha256:...",
      "expected_certificate_hash": "sha256:...",
      "certificate": {
        "encoding": "npa.certificate.canonical.v0.1.hex",
        "bytes": "..."
      }
    }
  ],
  "import_payload": {
    "module": "Scratch",
    "expected_export_hash": "sha256:...",
    "expected_certificate_hash": "sha256:...",
    "certificate": {
      "encoding": "npa.certificate.canonical.v0.1.hex",
      "bytes": "..."
    }
  }
}
```

`root_*` fields は extracted root theorem declaration だけを指します。
`root_decl_interface_hash` / `root_decl_certificate_hash` は、generated module certificate 内で
`SourceIndexToCertificateDeclIndex[root.source_index]` に置かれる root theorem declaration の Phase 2 hash です。
`root_axioms_used` は、生成した `certificate.bytes` を Phase 2 verifier に渡して得た verifier output から、
root theorem declaration の axiom dependencies を 5.1 の `MachineAxiomRefWire` JSON schema に変換し、
canonical order で並べたものです。
Phase 5 が certificate construction 前に作る `RootTheoremDependencyReport.axiom_dependencies` は照合用の入力であり、
成功レスポンスの正本ではありません。
この変換は generated certificate current-module context を使い、certificate-local declaration index を
`CertificateDeclIndexToSourceIndex` で Phase 5 source_index に戻してから行います。
Phase 2 verifier output から root theorem declaration の axiom dependencies を一意に取得できない場合、
または verifier output の axiom ref を `MachineAxiomRefWire` へ変換できない場合は
`VerifyFailed`、phase `certificate_verify` です。
Phase 5 の `RootTheoremDependencyReport.axiom_dependencies` と集合が一致しない場合は
`VerifyFailed`、phase `certificate_generation` です。

`module_*` fields は生成された Phase 2 module certificate 全体を指します。
`module_export_hash` は生成 certificate の `ExportBlock` hash、`module_certificate_hash` は
`certificate.bytes` の canonical certificate hash です。
`module_axioms_used` は、生成した `certificate.bytes` を Phase 2 verifier に渡して得た module-level
`axiom_report` payload を 5.1 の `AxiomRef to MachineAxiomRefWire` 規則で変換し、
`MachineAxiomRefWire` canonical order で sort/dedup したものです。
`module_axioms_used` の正本は Phase 2 verifier output であり、Phase 5 が certificate construction 前に持つ
`checked_current_decls` / root theorem の report union ではありません。
ただし verifier output の `module_axioms_used` と、Phase 5 が projection に使った全 declaration report の
axiom dependency union が一致しない場合は `VerifyFailed`、phase `certificate_generation` です。
`module_axioms_used` も同じ generated certificate current-module context で変換します。
session `allow_axioms` の subset check は verifier output 由来の `module_axioms_used` に対して行い、
root theorem だけを見てはいけません。
root theorem type だけから導出できる axiom dependencies は session create stage 9 ですでに検査済みですが、
verify では final proof body と generated module certificate 全体を含む verifier output に対して再度 subset check を行います。
`import_payload.expected_export_hash` は `module_export_hash`、`import_payload.expected_certificate_hash` は
`module_certificate_hash` と完全一致しなければなりません。

失敗:

```json
{
  "status": "error",
  "error": {
    "kind": "verify_failed",
    "phase": "kernel_check",
    "diagnostic_hash": "sha256:...",
    "retryable": false
  }
}
```

`status = verified` 以外は証明として保存してはいけません。
`mode = "certificate"` の成功レスポンスは canonical certificate bytes を必ず返します。
`import_payload` は後続 session の `import_closure` にそのまま渡せる `VerifiedModuleCertificateRequest` wire payload です。
`dependency_import_closure` は、`import_payload.certificate` を Phase 2 verifier で検査するために必要な
最小 transitive dependency closure です。
response order は `(module, export_hash, certificate_hash)` canonical order で、`import_payload` 自身は含めません。
依存がない場合は `[]` を返します。
後続 session で direct import にする場合は、この payload の `module` / `expected_export_hash` /
`expected_certificate_hash` から `imports` root key を作ります。
後続 session の `import_closure` は `dependency_import_closure + [import_payload]` の dedup 後 set を使います。
Phase 2 verifier output 由来の `module_axioms_used` が session `allow_axioms` の subset でない場合は
`DisallowedAxiom` として拒否します。
`certificate.bytes` から再計算した `module_certificate_hash` と `module_export_hash` は response の hash と一致しなければ
実装 bug として扱います。
将来 server-local verified certificate store を追加する場合でも、MVP の wire response は certificate bytes を省略しません。

---

# 14. Cache / Store

Machine API は大量候補を扱うため cache が必要です。
ただし cache hit は信頼根拠にしません。
MVP の cache は server-local advisory index であり、外部 response を生成する canonical artifact store ではありません。
MVP で cache lookup を実行の代替に使ってよい endpoint はありません。
`/machine/tactics/run` は cache hit を観測しても、response を返す前に必ず current snapshot payload に対して
deterministic tactic execution を再実行し、その実行で得た `MachineProofState`、`MachineProofDelta`、
`MachineApiDiagnostic` から response を作ります。
`/machine/tactics/batch` と `/machine/replay` は MVP では cache lookup を行いません。
batch は prefix 外 candidate の validation / execution を禁止するため、cache hit によって prefix rule、
stop rule、partial scheduler response を変えてはいけません。
batch は確定済み prefix result を response 生成後に advisory cache へ書き込んでもよいですが、その cache entry は
同一 request 内の後続 candidate や partial response 生成には使いません。

```rust
struct MachineApiCacheKey {
    protocol_version: MachineApiVersion,
    state_fingerprint: Hash,
    goal_id: GoalId,
    candidate_hash: Hash,
    deterministic_budget_hash: Hash,
    tactic_options_fingerprint: Hash,
}
```

cache value:

```rust
enum MachineCachedResult {
    Success {
        next_state_fingerprint: Hash,
        proof_delta_hash: Hash,
    },
    Error {
        diagnostic_hash: Hash,
        error_kind: MachineApiErrorKind,
    },
}
```

`MachineCachedResult` は「同じ deterministic input を過去に観測した」という hint だけです。
`Success` の `next_state_fingerprint` と `proof_delta_hash` から next snapshot payload を復元してはいけません。
`Error` の `diagnostic_hash` と `error_kind` から response error payload を復元してはいけません。
cache hit と live re-execution の結果が一致しない場合は cache entry を破棄し、live result だけを返します。
この不一致は trusted proof failure ではなく、server-local cache corruption / version mismatch として扱います。

high-trust replay は cache value を使わず、必ず replay plan の payload から各 step を再実行して hash を照合します。
`MachineSchedulerLimits` 由来の timeout / memory error はこの cache に保存しません。
full response を cache から返す artifact cache を追加する場合は non-MVP とし、
`MachineProofState` payload、`MachineProofDelta` payload、`MachineApiDiagnostic` payload の content-addressed store、
canonical bytes、eviction 後の fallback、versioning をこの文書で固定してから使います。

---

# 15. Security

Machine API は外部探索器から大量入力を受ける前提です。

server がしてはいけないこと:

```text
- AI candidate を trusted proof として保存する
- unchecked theorem を search result に混ぜる
- import の export_hash 不一致を無視する
- certificate_hash が必要な mode で certificate_hash を省略する
- batch の途中成功で元 snapshot を破壊する
- request に含まれる pretty string から core term を復元する
- request が指定した axiom report を信用する
- server-side LLM 呼び出しを kernel / checker 経路に混ぜる
```

resource 制限:

```text
- max sessions
- max snapshots per session
- max batch candidates
- max candidate bytes
- max theorem results
- max prompt payload bytes
- max replay steps
- per-client rate limit
```

上のうち endpoint schema で deterministic に固定されているものは、その endpoint の request validation /
scheduler artifact 契約に従います。
たとえば `max theorem results` は `/machine/search/for_goal.limit <= 256`、
`max batch candidates` は `/machine/tactics/batch.candidates.length <= 256`、
`max replay steps` は `/machine/replay.plan.steps.length <= 4096` として固定済みです。
`max candidate bytes` のように endpoint schema 側でまだ上限を固定していない guard を Machine API error として
返したい場合は、implementation cap ではなく protocol が許す request validation rule として明記してから有効化します。
MVP で endpoint 契約を持たない server-local guard、response-size guard、認証、rate limit、process kill は
transport / resource layer error であり、`MachineApiDiagnostic` と `diagnostic_hash` を生成してはいけません。
これらの guard は certificate payload、replay plan、deterministic cache key にも入れません。

MVP の kernel crate には、HTTP server、network、plugin loading、AI 呼び出しを入れません。
Proof server は kernel API を呼ぶ非信頼アプリケーションです。

---

# 16. 最小 API 一覧

MVP に入れる API は、この文書で request / response schema と fingerprint rule を固定したものだけにします。
ここにない endpoint を追加する場合は、MVP に昇格する前に schema-level contract をこの文書へ追記します。

Session:

```text
POST /machine/sessions
DELETE /machine/sessions/{id}
```

Snapshot:

```text
POST /machine/snapshots/get
```

Tactic:

```text
POST /machine/tactics/run
POST /machine/tactics/batch
```

Search:

```text
POST /machine/search/for_goal
```

Prompt:

```text
POST /machine/prompt_payload
```

Replay / Verify:

```text
POST /machine/replay
POST /machine/verify
```

Later / non-MVP:

```text
POST /machine/snapshots/list_open
POST /machine/tactics/parse
POST /machine/search/name
POST /machine/search/by_type
POST /machine/search/rewrite
```

non-MVP endpoint は、request / response shape、snapshot identity check、cache key、fingerprint input、
error taxonomy を固定してから実装します。

---

# 17. 実装順序

Phase 5 AI の endpoint を実装する前に、次の adapter / substrate を先に用意します。
これらがない状態で `/machine/sessions` から作り始めると、session は作れるが snapshot / search / verify が
決定的に materialize できない実装になりやすいです。

```text
Phase 5 AI substrate:
  - duplicate-key-aware / lossless JSON request decoder
    delayed validation payload を raw slice または syntax tree として保持できること
  - Phase 2 verifier output projection
    VerifiedModuleContextEntry、decl/generated table、axiom report projection を certificate bytes から再構築できること
  - CheckedCurrentDeclPackage decoder / revalidator
    source_index prefix、prior_chain_fingerprint、checked_env_fingerprint、dependency report を再検証できること
  - Phase 4 adapter boundary
    start_machine_proof、validate_machine_tactic_candidate、run_machine_tactic、
    extract_closed_machine_theorem_decl との hash / error mapping を固定すること
  - MachineSurfaceCallableInterfaceTable builder
    Phase 3 elaborator と renderer が同じ all-explicit profile table を読むこと
  - owner-aware MachineExprRenderer v1 と renderer QA
    snapshot / theorem statement / prompt の machine source を deterministic に round-trip できること
  - MachineApiDiagnostic canonicalization
    endpoint/stage ごとの phase と option population から diagnostic_hash を作れること
```

おすすめの順番はこれです。

```text
1. substrate
   lossless JSON decoder、Phase 2 projection、CheckedCurrentDeclPackage decoder、
   callable interface table、owner-aware renderer QA、diagnostic canonicalization を用意する

2. Machine API types
   MachineProofSession / MachineProofSnapshot / MachineGoalView / MachineApiErrorWire を定義する

3. Snapshot fingerprint
   state_fingerprint 由来の snapshot_id、StoredSnapshotView canonical bytes、store self-check を固定する

4. /machine/sessions
   verified imports、checked_current_decls、MachineProofSpec、MachineApiOptions から初期 snapshot を作る

5. /machine/snapshots/get
   pretty なしの machine goal view を返す

6. /machine/tactics/run
   Phase 4 AI run_machine_tactic を 1 候補実行で包む

7. /machine/tactics/batch
   同一 snapshot への候補を独立 transaction として試す

8. /machine/search/for_goal
   verified imports と Phase 4 simp registry から premise と MachineTacticCandidate を返す

9. replay plan
   delta chain を replay して state_fingerprint / proof_delta_hash を照合する

10. /machine/verify
   closed snapshot を kernel check / certificate generation に渡す

11. prompt payload
   Phase 7 用に deterministic prompt payload を生成する
```

---

# 18. テスト例

## 18.1 snapshot が決定的

同じ root / imports / options から session を作ると、同じ `initial_state_fingerprint` を返す。
session create response は replay 用の `session_root_hash` も返す。
imports は request order に依存せず `(module, export_hash, certificate_hash)` で sort/dedup され、
`root.theorem_type` は Phase 3 canonical Machine Surface payload と core hash に変換された後で
`session_root_hash` に入る。
`import_closure` は direct imports から到達する最小 transitive dependency certificate set と完全一致しなければならず、
closure に欠けた dependency、到達不能な extra certificate、または `certificate_hash = none` の dependency は
`InvalidVerifiedImport` として拒否される。
import payload に unknown field がある場合、certificate bytes が Phase 2 verifier に通らない場合、または
certificate bytes から再計算した hash が expected hash と一致しない場合は `InvalidVerifiedImport` として拒否する。
certificate encoding が literal value と違う場合、または certificate 内 module / export origin が request `module` と
一致しない場合も `InvalidVerifiedImport` として拒否する。
MachineApiOptions に unknown field や省略 field がある場合は `InvalidMachineApiOptions` として拒否する。
import / checked current declaration / root theorem type / verify final certificate の axiom dependencies が
`allow_axioms` の subset でない場合は `DisallowedAxiom` として拒否する。
root theorem type の axiom subset は session create 中、final proof / module certificate の axiom subset は verify 中に検査する。
`root.theorem_name` は `root.module` 配下の fully-qualified name でなければならず、`checked_current_decls.signature.name` と
重複してはいけない。
`root.theorem_name` または `checked_current_decls.signature.name` が direct import の public `ExportEntry.name` と
衝突する session は拒否する。
`checked_current_decls` は `source_index = 0..root.source_index-1` の完全 prefix であること、source order、
`prior_chain_fingerprint`、`checked_env_fingerprint` を再検証し、
imports + prior checked decls に対して kernel check できない payload は `InvalidCheckedCurrentDecl` として拒否する。
`checked_current_decls` の `source_index` 重複は `InvalidCheckedCurrentDecl` として拒否する。
`tactic_options_fingerprint` は Phase 4 `MachineTacticOptions canonical bytes` だけの hash で、resolved family bytes は
`state_fingerprint` と `session_root_hash` 側で照合する。
初期 snapshot は top-level binder を自動で開かず、`context = []` と `target = root.theorem_type` を返す。
`MachineGoalView.target_hash` は `target.core_hash` と一致し、どちらも
`NPA-PHASE1-EXPR-0.1` domain の Phase 1 `Expr` structural hash である。
`snapshot_fingerprint` という別名 field は返さない。
`snapshot_id` は `sha256:` prefix を除いた digest hex から導出される。
`local_name_map_hash` は Phase 5 view integrity 用の派生 hash であり、Phase 4 の `context_hash` と
`state_fingerprint` には追加しない。
表示名だけを変えても `machine_name` は変わらない。
`machine_name` は Phase 4 `MachineLocalDecl.name` と一致し、candidate を Phase 4 に渡す前に Phase 5 が local 名を
rewrite しない。

## 18.2 tactic failure が state を壊さない

`intro` できない goal に `intro` を実行した場合:

```json
{
  "status": "error",
  "unchanged_state_fingerprint": "sha256:old"
}
```

元 snapshot を再取得すると、同じ `state_fingerprint` と open goals を返す。
request は expected target hash を渡さず、server は goal の target から expected type を導出する。
candidate schema validation、Phase 5 `RawMachineTerm` prepass、Phase 4 candidate canonicalization がすべて成功し、
Phase 4 `MachineTactic canonical bytes` を構築できた candidate では response に `candidate_hash` と
`deterministic_budget_hash` を返す。
candidate schema validation、または Phase 5 `RawMachineTerm` prepass の parse / canonicalization failure では
`candidate_hash` を返さない。
Phase 5 `RawMachineTerm` prepass 成功後に Phase 4 が `InvalidMachineTermSource` を返す場合は
adapter invariant failure として `InvalidMachineProofState`、phase `candidate_validation` を返し、
`candidate_hash` を返さない。
Phase 4 candidate canonicalization 成功後の elaboration / type check / tactic execution error では
`candidate_hash` を返す。
Phase 4 candidate canonicalization 成功後の tactic error は `diagnostic_hash` を返し、`candidate_hash` を伴い、かつ
`FailedCandidateErrorKind` に含まれる error kind の場合だけ prompt payload の `failed_candidates` に再利用できる。
batch の per-candidate error item は compact summary だが、`error_kind`、`phase`、`diagnostic_hash`、`retryable` と
diagnostic option field を返すため、client は item だけから `diagnostic_hash` を監査できる。
`MachineDeterministicBudget` の JSON field order を変えても同じ `deterministic_budget_hash` になり、
unknown field や `null` は `InvalidBudget` になる。
0 fuel は `InvalidBudget` ではなく accepted budget として hash 化され、Phase 4 の deterministic fuel error を返す。

## 18.3 batch ordering

server 内部で並列実行しても、batch response の `results` は request の `candidates` array order で返る。
`candidate_id` は correlation id であり、辞書順 sort や numeric sort には使わない。
`stop_after_successes` などの stop rule は request order の prefix にだけ適用される。
`scheduler_limits` による `partial_timeout` / `partial_resource_limit` は retryable で、deterministic cache に入らない。
`partial_timeout` response は `previous_state_fingerprint`、`deterministic_budget_hash`、`completed_prefix_len`、
確定済み prefix results、`scheduler_artifact` だけを返し、未確定 candidate diagnostic は含めない。
per-candidate timeout / memory stop が発火した candidate は `results` に含めず、success / failure count にも数えない。

## 18.4 search result が import hash に固定される

`Nat.add_zero` の検索結果には次が含まれる。

```text
- module
- name
- export_hash
- decl_interface_hash
- axiom dependencies
```

export_hash が request imports と一致しない場合は search result に出さない。
MVP の theorem index は direct verified imports だけを対象にし、`checked_current_decls` は検索結果に含めない。
theorem index の `modes` は Phase 2 verifier が導出した `ExportBlock` / `certified_env_decls` と
Phase 4 の検証済み `SimpRegistry` だけから Phase 5 MVP enum と固定 mapping で導出する。
Phase 6 theorem index metadata、attribute、rewrite hint は MVP の theorem index fingerprint に入れない。
`filters.exclude_axioms` は必須 bool で、unknown filter field、`null`、direct session import にない `allowed_modules` は
`InvalidTheoremQuery` として拒否する。
`limit` は必須で `1 <= limit <= 256`、0 / `null` / float / 範囲外は `InvalidTheoremQuery` として拒否する。
`/machine/search/for_goal` request は `state_fingerprint` を含み、current session の snapshot store で
`snapshot_id` を lookup した後、6.1 と同じ stored snapshot self-check を先に行う。
self-check 失敗は request の `state_fingerprint` と比較する前に `InvalidMachineProofState` として拒否し、
self-check 成功後に stored `state_fingerprint` と request の `state_fingerprint` が一致しない場合は
`StateFingerprintMismatch` として拒否する。
top-level unknown field、duplicate key、required field omitted、`null`、id/hash/goal field の型不一致は
`InvalidTheoremQuery` として拒否する。
同じ theorem index canonical bytes から同じ `theorem_index_fingerprint` を返し、`modes` / `filters` /
`limit` の canonical query から同じ `query_fingerprint` と truncation を返す。
match score は `limit` を含まない `search_score_key_fingerprint` から計算する。
`suggested_candidates` は search result の必須 JSON array field で、候補がない場合も `[]` を返す。
`suggested_candidates` に入る候補は MVP suggested candidate validation 済みで、`candidate_hash` と raw `candidate` payload を持つ。
この validation は Machine Surface Complete mode、tactic execution、kernel check を呼ばず、`deterministic_budget` も要求しない。
MVP deterministic templates は Machine Surface term source を含む candidate を生成しない。
将来 candidate 内に Machine Surface term source を含む suggested candidate を追加する場合、その term source が
完全明示形式であることの判定規則を `suggestion_profile_version` と `query_fingerprint` に追加する。
`suggested_candidates.candidate` は Phase 4 external `MachineTacticCandidate` wire schema と同じ raw payload であり、
`rw` は `rule.head` / `rule.universe_args` / `rule.args` / `direction` / `site` を持つ。
`suggestion_profile_version` を変える場合は `query_fingerprint` も変わる。
validation できない候補は `suggested_candidates` に入れず、MVP response では `repair_hints` も返さない。

## 18.5 prompt payload fingerprint が入力に固定される

同じ `session_root_hash` / `state_fingerprint` / `goal_id` / theorem index / prompt options から作った
prompt payload は同じ `payload_fingerprint` を返す。
`include_pretty`、`include_failed_candidates`、`premise_selection`、selected premise、`allowed_tactics`、
または rendered payload field が変わる場合は `payload_fingerprint` も変わる。
`payload_fingerprint` 自身は hash 入力に含めず、`theorem_index_fingerprint` と `premise_query_fingerprint` は
explicit input として 1 回だけ含める。
prompt response の `premises` は `global_ref` / `decl_interface_hash` / statement core hash を含む search result と
同じ verified metadata を返し、name-only premise は返さない。
prompt response は search response の `score` / `suggested_candidates` を含めず、premise metadata だけを
`PromptRenderedContent` に入れる。
failed candidates は request の `failed_candidates` だけから取り、session history や cache から自動収集しない。

## 18.6 replay step payload が自己完結している

各 replay step は `candidate_hash` / `deterministic_budget_hash` だけでなく、wire `candidate` と
`deterministic_budget` payload を含む。
replay は payload から候補を再実行し、hash と `proof_delta_hash` を照合する。
well-formed plan の `session_root_hash` が current session と一致しない場合は `SessionRootHashMismatch` を返す。
MVP replay は current session の initial snapshot から開始する。
plan structure / chain が壊れている場合は `SessionRootHashMismatch` より先に `InvalidReplayPlan` を返し、
再実行結果の hash が plan と違う場合は
`ReplayHashMismatch` を返す。
replay step の Phase 5 `RawMachineTerm` prepass failure は `ReplayHashMismatch` であり、
Phase 3 source diagnostic を response error として返さない。
prepass 成功後に Phase 4 が `InvalidMachineTermSource` を返す adapter invariant failure は
`InvalidMachineProofState`、phase `replay_execution` であり、`ReplayHashMismatch` ではない。

## 18.7 verify まで証明扱いしない

open goals が空の snapshot に対して `/machine/verify` を呼び、kernel check と certificate check に通った場合だけ
`status = verified` を返す。
verify 成功レスポンスは canonical certificate bytes、`dependency_import_closure`、後続 session の
`import_closure` に渡せる `import_payload` を返す。
response は root theorem 単体の `root_decl_interface_hash` / `root_decl_certificate_hash` / `root_axioms_used` と、
module certificate 全体の `module_export_hash` / `module_certificate_hash` / `module_axioms_used` を分けて返す。
`import_payload.expected_export_hash` は `module_export_hash`、`import_payload.expected_certificate_hash` は
`module_certificate_hash` と一致する。
生成 certificate は session direct imports を canonical order の import table とし、
`checked_current_decls` の完全 prefix と root theorem から Phase 2 canonical declaration order を作り、
source_index 座標の current-module refs を certificate-local declaration index へ rewrite してから保存する。

---

# 19. Phase 5 AI でまだ入れないもの

MVP では次を入れません。

```text
- server-side LLM 呼び出し
- embedding semantic search
- natural language formalization
- automatic theorem invention
- global best-first search scheduler
- proof minimization
- distributed worker orchestration
- plugin tactic execution
- arbitrary user-defined tactic
- unverified external theorem database
```

これらは Phase 7 以降、または別の非信頼サービスとして追加します。

---

# 20. 完了条件

Phase 5 AI Profile が完了したと言える条件はこれです。

```text
- verified imports から MachineProofSession を作れる
- MachineProofSnapshot が deterministic state_fingerprint を持つ
- AI 向け goal view が pretty と machine 表現を分けて返せる
- MachineTacticCandidate を 1 件実行できる
- batch tactic execution が transactional で順序決定的である
- tactic error が MachineApiErrorKind と hash 付き diagnostic で返る
- theorem search が verified metadata と suggested MachineTacticCandidate を返す
- replay plan を再実行して delta hash を照合できる
- closed snapshot を kernel check / certificate generation に渡せる
- verify 成功まで certificate / theorem を verified と扱わない
```

---

# 21. 一文でまとめると

Phase 5 AI Profile は、**AI 証明探索器が proof state・tactic execution・theorem retrieval を大量かつ決定的に扱うための、非信頼 Machine API 層**です。

中核はこの4つです。

```text
machine proof snapshot:
  goal/context/target を canonical hash と Machine Surface で返す

batch tactic execution:
  AI 候補を transactional に試し、成功 delta と構造化 error を返す

verified theorem retrieval:
  premise を export_hash / decl_interface_hash に固定して返す

replay / verify handoff:
  delta chain を再実行し、最後は kernel / certificate checker に渡す
```
