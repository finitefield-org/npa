# Phase 4 AI Profile: Machine Tactics

この文書は、NPA の **AI 向け Phase 4** の設計案です。

従来の `doc/phase4-human.md` は、人間が `by ...` block と tactic script を書き、proof state を
小さな命令で進めるための設計です。一方、AI 証明探索では、多数の tactic 候補を生成し、失敗を
前提に高速・決定的・transactional に検査します。

そのため AI 向け Phase 4 では、人間向け tactic syntax を直接信用せず、構造化された
**Machine Tactic** を受け取り、通ったものだけ proof term / certificate に接続します。

---

# 1. 目的

Machine Tactic の目的は、AI が出す tactic 候補を小さい信頼境界の外側で試し、成功した候補だけを
kernel が検査できる proof term に変換することです。

```text
AI candidate
  ↓ parse / validate Machine Tactic
structured tactic AST
  ↓ transactional tactic execution
new proof state + proof delta
  ↓ unresolved goal check
core proof term
  ↓ kernel check
  ↓ certificate generation / verification
canonical certificate
```

優先する性質は次です。

```text
- tactic text を trusted payload に入れない
- tactic trace / score / prompt / model metadata を certificate に入れない
- proof state の更新は transactional
- failure は structured error
- 同じ state + 同じ tactic + 同じ deterministic budget から同じ result / error
- term payload は Phase 3 Machine Surface term-level API で検査する
```

---

# 2. 信頼境界

Machine Tactic は信用しません。

```text
信頼しない:
  AI output
  tactic parser
  tactic selection / ranking
  repair suggestion
  proof search trace
  tactic execution log

信頼する:
  canonical core proof term
  Phase 1 Rust kernel
  Phase 2 certificate verifier
  Phase 8 independent checker
```

AI tactic が成功したように見えても、それは証明ではありません。最終的な proof term が kernel check と
certificate verify に通った場合だけ採用します。

---

# 3. Human Tactic との差分

人間向け Phase 4 は読みやすさを優先します。

```npa
by
  intro n
  rw [Nat.add_zero]
  exact Eq.refl n
```

AI 向け Phase 4 は、曖昧な syntax よりも構造化入力を優先します。

```json
{
  "kind": "exact",
  "term": "@Eq.refl.{1} Nat n"
}
```

AI 向け Phase 4 の構造化 tactic 層では次を使いません。

```text
- open / namespace に依存する short name
- notation / infix / numeric literal overload
- implicit term argument insertion
- tactic macro
- arbitrary tactic script
- backtracking tactic language
- user-defined tactic
- plugin execution
- IO / network / file access
```

Phase 4 full target で扱う tactic / API は次です。
最初の AI MVP completion scope は Section 13 の `exact` / `intro` / `apply` に限定します。

```text
- exact
- intro
- apply
- rw
- simp-lite
- induction-nat
- Phase 3 Machine Surface term payload
- verified import / export_hash
```

---

# 4. 前提条件

Machine Tactic は Phase 3 AI の term-level API に依存します。

```text
required before Phase 4 AI M2:
  - elaborate_machine_term_check
  - canonicalize_machine_term_source
  - local context import
  - expected type check
  - constants / core_hash extraction
```

`exact`, `apply`, `rw`, `simp-lite` はいずれも term の型検査や rule interface の検査を必要とします。
そのため Phase 3 AI M7 が未実装の状態では、Phase 4 AI は MachineProofState skeleton と parser /
validator までに留め、proof-producing tactic は始めません。

---

# 5. Proof State

Proof state は AI に渡せる形では構造化します。ただし表示用 state は trusted payload ではありません。

```rust
struct MachineProofState {
    state_id: StateId,
    root: ProofRoot,
    open_goals: Vec<GoalId>,
    metavars: MetaVarStore,
    env: MachineTacticEnv,
    reserved_local_names: Vec<String>,
    fingerprint: Hash,
}

struct ProofRoot {
    module: ModuleName,
    theorem_name: Name,
    source_index: u64,
    universe_params: Vec<String>,
    theorem_type: Expr,
    body: ProofExpr,
}

struct MachineTacticEnv {
    imports: Vec<VerifiedImportRef>,
    checked_current_decls: Vec<CheckedCurrentDecl>,
    simp_registry: SimpRegistry,
    eq_family: Option<ResolvedEqFamily>,
    nat_family: Option<ResolvedNatFamily>,
    options: MachineTacticOptions,
    options_fingerprint: Hash,
}

struct VerifiedImportRef {
    module: ModuleName,
    export_hash: Hash,
    certificate_hash: Hash,
    exports: Vec<CheckedDeclSignature>,
    certified_env_decls: Vec<Decl>,
}

struct CheckedCurrentDecl {
    // 実装では private field にし、専用 constructor 以外から作らせない。
    source_index: u64,
    signature: CheckedDeclSignature,
    core_decl: Decl,
    prior_chain_fingerprint: Hash,
    checked_env_fingerprint: Hash,
}

struct CheckedDeclSignature {
    name: Name,
    universe_params: Vec<String>,
    ty: Expr,
    decl_interface_hash: Hash,
}

enum ProofExpr {
    Core(Expr),
    Meta(MetaVarId),
    App(Box<ProofExpr>, Box<ProofExpr>),
    Lam { name: String, ty: Expr, body: Box<ProofExpr> },
    Let { name: String, ty: Expr, value: Box<ProofExpr>, body: Box<ProofExpr> },
}

struct MetaVarStore {
    metas: BTreeMap<MetaVarId, MachineMetaVar>,
    goal_to_meta: BTreeMap<GoalId, MetaVarId>,
    next_id: u64,
}

struct MachineGoal {
    goal_id: GoalId,
    meta_id: MetaVarId,
    context: Vec<MachineLocalDecl>,
    context_hash: Hash,
    target: Expr,
    target_hash: Hash,
}

struct MachineMetaVar {
    id: MetaVarId,
    goal_id: GoalId,
    context: Vec<MachineLocalDecl>,
    target: Expr,
    assignment: Option<ProofExpr>,
}

struct MachineProofDelta {
    previous_state_fingerprint: Hash,
    assigned_goal: GoalId,
    assigned_meta: MetaVarId,
    assigned_proof_expr_hash: Hash,
    new_goals: Vec<GoalId>,
    new_metas: Vec<MachineNewMetaDelta>,
    next_state_fingerprint: Hash,
    proof_delta_hash: Hash,
}

struct MachineNewMetaDelta {
    meta_id: MetaVarId,
    goal_id: GoalId,
    context_hash: Hash,
    target_hash: Hash,
}

struct MachineLocalDecl {
    name: String,
    ty: Expr,
    value: Option<Expr>,
}
```

`ProofExpr canonical bytes` は proof skeleton hash、metavariable assignment hash、
`MachineProofDelta.assigned_proof_expr_hash` に共通して使います。

```text
ProofExpr canonical bytes:
  - tag "npa.phase4.proof-expr.v1"
  - variant tag:
      Core / Meta / App / Lam / Let
  - Core:
      Phase 1 Expr canonical bytes
  - Meta:
      MetaVarId canonical bytes
  - App:
      function ProofExpr canonical bytes
      argument ProofExpr canonical bytes
  - Lam:
      UTF-8 binder name bytes
      ty Expr canonical bytes
      body ProofExpr canonical bytes
  - Let:
      UTF-8 binder name bytes
      ty Expr canonical bytes
      value ProofExpr canonical bytes
      body ProofExpr canonical bytes
```

`ProofExpr::Core` は pretty text ではなく、Phase 1 の core `Expr` canonical bytes を埋め込みます。
source span、AI trace、diagnostic message、display-only goal text は `ProofExpr canonical bytes` に入りません。
`proof skeleton with MetaVarId references`、metavariable `assignment hash`、
`assigned_proof_expr_hash` はすべてこの bytes の hash です。

Phase 4 local id の canonical bytes は Phase 2 と同じ minimal unsigned LEB128 整数 encoding を使い、
id 種別ごとに domain tag を分けます。

```text
MetaVarId canonical bytes:
  - tag "npa.phase4.meta-var-id.v1"
  - numeric id as minimal unsigned LEB128

GoalId canonical bytes:
  - tag "npa.phase4.goal-id.v1"
  - numeric id as minimal unsigned LEB128
```

`GoalId(n)` は `MetaVarId(n)` から導出しますが、canonical bytes の tag は共有しません。

Phase 4 独自 canonical bytes で使う primitive encoding は、別途明記がない限り次で固定します。

```text
Phase 4 canonical primitive encoding:
  - unsigned integer / u64 / list length:
      Phase 2 と同じ minimal unsigned LEB128
  - UTF-8 string:
      byte length as minimal unsigned LEB128, followed by UTF-8 bytes
  - list:
      length as minimal unsigned LEB128, followed by elements in the specified order
  - option:
      none tag 0x00, or some tag 0x01 followed by payload
  - enum variant:
      variant tag as the documented stable tag string unless a numeric tag is explicitly specified
  - hash:
      exactly 32 raw bytes
```

`source_index`、`TacticBudget` fields、`MachineTacticOptions` numeric fields、`MetaVarStore.next_id`、
`StateId` derivation input、diagnostic hash payload の count はすべてこの encoding を使います。
`HashMap` / `HashSet` iteration order、platform endianness、Rust enum discriminant は canonical bytes に
使ってはいけません。

`CheckedDeclSignature canonical bytes` は次で固定します。

```text
CheckedDeclSignature canonical bytes:
  - tag "npa.phase4.checked-decl-signature.v1"
  - name canonical bytes
  - universe_params in kernel Decl order, as UTF-8 bytes
  - ty Expr canonical bytes
  - decl_interface_hash
```

この hash は `signature hash` と書かれている箇所、diagnostic の
`CurrentDeclSignatureMismatch.expected_hash / actual_hash`、state fingerprint の export signature hashes に
共通して使います。

`MachineLocalDecl canonical bytes` と context hash は次で固定します。

```text
MachineLocalDecl canonical bytes:
  - tag "npa.phase4.machine-local-decl.v1"
  - UTF-8 local name bytes
  - ty Expr canonical bytes
  - value:
      none tag, or some tag + value Expr canonical bytes

MachineLocalContext canonical bytes:
  - tag "npa.phase4.machine-local-context.v1"
  - MachineLocalDecl list in context order
```

`context hash` は `MachineLocalContext canonical bytes` の hash です。`target_hash`、`theorem type hash`、
`eq_type hash`、`theorem_lhs hash`、`theorem_rhs hash`、`from_pattern hash`、`to_pattern hash` は
Phase 1 `Expr` canonical bytes の hash です。
`core declaration hash` は Phase 2 certificate と同じ canonical core `Decl` hash です。

AI へ返す goal 表示には human-friendly text を含めてもよいですが、tactic 実行時には `goal_id` と
内部 core target を基準にします。

`MachineLocalDecl` は Phase 3 の `MachineLocalDecl` と同じ構造を使います。`value = None` は local
assumption、`value = Some(expr)` は local let value です。context prefix 判定では `name`, `ty`, `value` の
canonical hash がすべて一致することを要求し、表示名だけでは比較しません。

`reserved_local_names` は binding environment ではなく、deterministic fresh name 生成と fingerprint 用の
derived field です。真の local binding は各 `MachineMetaVar.context` の `MachineLocalDecl` だけで表します。
`reserved_local_names` はすべての open / assigned meta context、`ProofRoot.body`、およびすべての assigned
metavariable の `ProofExpr` を再帰走査して見つかる binder 名から canonical order で再計算します。
`intro`, `apply`, `rw`, `simp-lite`, `induction-nat` が作る synthetic local name はこの集合と現在 goal context
の名前を避けます。実装が field として保持せず都度計算してもよいですが、state fingerprint に入れる
canonical bytes はこの規則で固定します。

`Name` はすべて fully qualified canonical name とします。`VerifiedImportRef.exports` は tactic head /
simp rule 解決用の public signature index であり、kernel 環境を作る情報ではありません。
この文書の `universe_params: Vec<String>` はすべて、parser/resolver 後に kernel `Decl` が保持する ordered
universe parameter 名列をそのまま表します。Phase 4 が universe parameter を rename、sort、deduplicate、
または constraints 付き context に変換することはありません。
`certified_env_decls` には certificate verification 後の canonical core declaration を保持し、
exported declaration と、その transparent body / recursor / type を downstream conversion で使うために必要な
verified dependency closure を入れます。非 export の dependency は kernel 環境には入りますが、tactic head や
simp rule としては `exports` に存在しない限り参照できません。reducible definition、inductive、recursor の
body / reducibility metadata は axiom に潰さず Phase 3 term check と kernel conversion へ渡します。

`CheckedCurrentDecl` は current module 内で既に kernel check 済みの declaration だけを表します。
`signature` は tactic head / simp rule 解決用、`core_decl` は Phase 3 term check と kernel conversion 用です。
current module の reducible def を後続 tactic で展開できるよう、`core_decl` には body / reducibility metadata を
保持します。opaque theorem は kernel 側の通常規則に従い、proof body を conversion で展開しません。
実装では `CheckedCurrentDecl` の fields を公開せず、`check_current_decl_for_machine_tactic` だけが生成します。
この constructor は current module 内の parser/resolver source order index、import 環境、先行 current
declarations を受け取り、`source_index` が prior declarations より大きいことを確認してから `core_decl` を
kernel check します。さらに prior declarations から `prior_chain_fingerprint` を計算し、kernel environment から
`checked_env_fingerprint` を計算して記録します。`start_machine_proof` は `checked_current_decls` を state に
取り込む前に、`source_index` が strictly increasing であること、各 `prior_chain_fingerprint` がその位置より前の
checked current declarations から再計算した値と一致すること、各 `checked_env_fingerprint` が同じ imports と
その declaration より前の checked current declarations から再計算した値と一致することを確認します。
一致しない場合は `InvalidCurrentDeclOrder` または `UncheckedCurrentDecl` として reject します。

`prior_chain_fingerprint` の canonical bytes は次だけをこの順序で含めます。

```text
prior_chain_fingerprint includes:
  - tag "npa.phase4.current.prior-chain.v1"
  - prior checked current declarations in source order:
      source_index / signature hash / core declaration hash /
      prior_chain_fingerprint / checked_env_fingerprint

prior_chain_fingerprint excludes:
  - imports
  - declaration being checked
  - future current declarations
  - tactic options / simp rules / eq_family / nat_family
  - display text / source span / diagnostics
```

`checked_env_fingerprint` の canonical bytes は次だけをこの順序で含めます。

```text
checked_env_fingerprint includes:
  - tag "npa.phase4.current.checked-env.v1"
  - verified imports in canonical order by (module, export_hash, certificate_hash):
      module / export_hash / certificate_hash
      exports signature hashes in canonical order
      certified_env_decls in verifier dependency-topological order:
        canonical core declaration hash
  - prior_chain_fingerprint
  - prior checked current declarations in source order:
      source_index / signature hash / core declaration hash / checked_env_fingerprint
  - kernel_check_profile_hash

checked_env_fingerprint excludes:
  - declaration being checked
  - future current declarations
  - tactic options / simp rules / eq_family / nat_family
  - display text / source span / diagnostics
```

`kernel_check_profile_hash` は Phase 1 kernel の checking profile を表す固定 canonical hash です。Phase 4 API の
入力ではありません。kernel の conversion/reduction/universe checking 規則や builtins profile が変わる場合は、
この profile hash を変えます。

```text
kernel_check_profile_hash canonical bytes:
  - tag "npa.phase4.kernel-check-profile.v1"
  - core spec id: "core-spec-v0.1"
  - kernel semantics profile id: "npa-kernel.phase1.v0.1"
  - reduction profile id: "beta-delta-iota-zeta.v0.1"
  - universe profile id: "levels-imax-v0.1"
  - builtin profile id:
      "builtin-eq-nat-v0.1" if the kernel exposes Eq/Nat builtins for this run,
      otherwise "builtin-none-v0.1"
```

Phase 1 kernel が同等の `kernel_check_profile_bytes()` API を公開している場合は、その bytes と上の field
集合が一致することを Phase 4 側で確認して使います。crate package version、build timestamp、feature flag 名、
CPU/OS 情報はこの hash に入れてはいけません。kernel の検査意味論が変わる場合だけ profile id を bump します。

`signature` は手書き値を信用せず、Phase 2 と同じ declaration interface hash 実装で `core_decl` から再計算します。
再計算した `name`, `universe_params`, `ty`, `decl_interface_hash` のいずれかが一致しなければ
`CurrentDeclSignatureMismatch` として state 作成前に reject します。def / theorem / axiom / inductive /
recursor ごとの差分は Phase 2 の canonical interface hash rule を唯一の基準にし、Phase 4 独自の hash
アルゴリズムを持ちません。

`MachineGoal` は `MetaVarStore` から導出できる表示・実行用 view です。mutable な真の状態は
`ProofRoot.body`, `open_goals`, `MetaVarStore`, `MachineTacticEnv` に限定し、同じ情報を二重管理しません。
`MachineProofDelta.new_metas` は diagnostic payload 用の summary です。以後 `assign_goal` の規則で
`new_metas` とだけ書く場合は、この summary ではなく今回作った fresh `MetaVarId` 集合を指します。

すべての tactic は次の共通 primitive で state を更新します。

```text
assign_goal(
  state,
  goal_id,
  proof_expr,
  new_goal_specs,
) -> (new_state, MachineProofDelta)
```

`assign_goal` の規則:

```text
1. goal_id が state.open_goals に存在し、goal_to_meta に対応する meta が未代入であることを確認する
2. new_goal_specs を左から順に fresh MetaVarId / GoalId に変換し、MetaVarStore に追加する
3. proof_expr が assigned meta の context で well-scoped であることを確認する
4. proof_expr の meta dependency を再帰検査し、未代入 Meta は今回作った new_metas だけに限定する
5. proof_expr の型が assigned meta の target になることを、Phase 4 の proof skeleton checker で検査する
6. assigned meta の assignment に proof_expr を入れる
7. open_goals 内の goal_id を new_goals で置き換える
8. state_fingerprint と proof_delta_hash を canonical bytes から再計算する
```

途中で失敗した場合は state を変更しません。`exact` は `new_goal_specs = []`、`intro` は body goal を 1 つ、
`apply` は premise goals、`rw` は premise goals と rewritten target goal を渡します。`simp-lite` は
Eq.refl closure で閉じる場合は `new_goal_specs = []`、固定点で止まる場合は rewritten target goal を 1 つ渡します。

`proof_expr` は今回作った `new_metas` と、既に assignment を持つ既存 meta を参照してよいです。ただし
`proof_expr` を再帰展開した dependency graph は必ず DAG でなければなりません。`assign_goal` は
`proof_expr` から到達できる既存 assignment を再帰的にたどり、次のいずれかに当たれば state 更新前に
`InvalidMetaDependency` として reject します。

```text
- assigned_meta 自身に到達する
- assignment を持たない既存 meta に到達し、その meta が今回作った new_metas に含まれない
- dependency walk 中に同じ MetaVarId へ戻る cycle を検出する
```

この検査では、今回作った `new_metas` は leaf として扱い、その assignment の有無はまだ要求しません。
したがって `apply` / `rw` のように、親 goal の proof skeleton が新しい subgoal meta を参照する形は許されます。
一方、後続 goal が自分を待っている親 assignment を参照して cycle を作ることはできません。

`assign_goal` は kernel に metavariable を追加しません。Phase 4 は trusted でない補助 checker として
`check_proof_expr_with_metas(env, context, proof_expr, expected, meta_store)` を実装します。この checker は:

```text
- Core(expr) は meta を含まない core Expr として Phase 3 / kernel check に委譲する
- Meta(id) は id の context が現在 context と同一か prefix の場合だけ使える
- Meta(id) の target を必要な分だけ lift し、expected と kernel conversion で一致することを確認する
- App / Lam / Let は ProofExpr 構造をたどって型を合成または check する
- 参照できる未代入 meta は今回の assign_goal で作った new_metas だけに限定する
- 代入済み meta を参照する場合は assignment を再帰展開し、cycle と old open meta 参照を拒否する
```

この checker の成功は certificate の根拠ではありません。closed proof extraction 後に meta を完全消去した core
`Expr` を kernel check し、そこだけを Phase 2 certificate に渡します。

metavariable は MachineProofState 内だけの非 trusted 表現です。kernel / certificate に渡す core AST には
`MetaVarId` や `ProofExpr::Meta` を残してはいけません。`extract_closed_machine_proof` は全
metavariable が assignment を持つことを確認し、完全代入済みの core `Expr` だけを返します。

metavariable assignment は、その metavariable が作られた `context` に対して check します。
`ProofExpr::Meta(id)` を展開する時点の context は、meta の context と同一か、その extension で
なければなりません。extension の下で展開する場合は、assignment の de Bruijn index を extension
分だけ lift します。context が prefix 関係にない場合は `InvalidMetaContext` として reject します。

`ProofExpr` と core `Expr` の binder 境界は次の規則で固定します。

```text
- MachineMetaVar.context を Γ とする
- ProofExpr::Lam / Let が導入した未抽出 binder を Δ とする
- ProofExpr::Core(expr) は Γ, Δ に相対的な通常の core Expr であり、ProofExpr::Meta を内部に含めない
- Lam.ty と Let.ty は導入前の Γ, Δ で well-scoped な core Expr である
- Let.value は導入前の Γ, Δ で check し、Let.body は Γ, Δ, name := value : ty で check する
- ProofExpr::Meta(id) は Expr 内には埋め込まず、ProofExpr node としてだけ現れる
```

`extract_closed_machine_proof` は `ProofExpr` を core `Expr` に落とすとき、`Lam` / `Let` ごとに通常の core
binder を生成し、`Core(expr)` は現在の Γ, Δ depth に合わせてそのまま埋め込みます。`Meta(id)` の assignment を
展開する場合だけ、context extension 分の lift を行います。`Core(expr)` の de Bruijn index が現在の Γ, Δ を
越えていれば `ProofExprScopeError` として reject します。

closed proof extraction:

```text
1. root.body を現在 context = [] で走査する
2. ProofExpr::Meta(id) は assignment を再帰的に展開する
3. 未代入 meta があれば UnresolvedGoal
4. 展開後の core Expr を theorem_type に対して kernel check する
5. tactic / meta / display metadata を含まない core Expr だけ返す
```

`start_machine_proof` は `VerifiedModule` から `VerifiedImportRef` を作るとき、import 環境を canonical
order に正規化します。parser/resolver は current module の top-level declaration に source order の
0-based gapless index を割り当てます。`MachineProofSpec.source_index` は証明対象 theorem の index です。
`checked_current_decls` は同じ index 体系で source order に渡す必要があり、この順序は prior declaration chain の
検証と state fingerprint にそのまま使います。Phase 4 AI では任意 subset ではなく、証明対象の直前までの
完全 prefix を要求します。

`start_machine_proof` が作る初期 state は次で固定します。

```text
- root.body = ProofExpr::Meta(m0)
- metas = { m0 => MachineMetaVar {
      id: m0,
      goal_id: g0,
      context: [],
      target: MachineProofSpec.theorem_type,
      assignment: None
  } }
- goal_to_meta = { g0 => m0 }
- open_goals = [g0]
- next_id = 1
```

`m0` は最初に割り当てる `MetaVarId(0)`、`g0` は `m0` から決定的に導出する `GoalId(0)` です。Phase 4 は
`GoalId` 用の独立 counter を持ちません。以後も `new_goal_specs` の左から順に fresh `MetaVarId` を割り当て、
対応する `GoalId` は同じ番号から導出します。`start_machine_proof` は state 作成前に
`MachineProofSpec.theorem_type` を `MachineProofSpec.universe_params` の下、empty context で kernel infer し、
その型が sort に WHNF しなければ `TypeMismatch` として reject します。

`source_index` は parser/resolver が読んだ source-level top-level declaration だけに付きます。inductive から生成される
recursor、constructor signature、private helper、certificate verification 時に復元される dependency declaration は
別の `source_index` を持ちません。これらの生成・依存 declaration は、対応する source declaration の
`core_decl` / certified dependency closure に含め、`CheckedCurrentDecl.source_index` の連番には数えません。
そのため `0..MachineProofSpec.source_index` の完全 prefix は「証明対象より前に source に現れた declaration」の
完全 prefix だけを意味します。

次の入力は実行前に reject します。

```text
- 同じ module name に複数の export_hash / certificate_hash が対応する
- 複数 import が同じ fully qualified export name を公開する
- checked_current_decls が source_index 0..MachineProofSpec.source_index の完全 prefix でない
- checked_current_decls の source_index が strictly increasing でない
- checked_current_decls の prior_chain_fingerprint が prior declarations と一致しない
- checked_current_decls に同じ signature.name が複数回現れる
- checked_current_decls に kernel check 済みでない current declaration が混ざる
- checked_current_decls の checked_env_fingerprint が imports + prior checked current declarations と一致しない
- checked_current_decls の signature と core_decl から再計算した signature が一致しない
- certified_env_decls が exported declaration または必要な dependency closure を欠いている
- certified_env_decls と exports の public signature が一致しない
- certified_env_decls が dependency-topological canonical order でない
- imports / checked_current_decls から作る kernel environment に、canonical dedup 後も同じ name で異なる decl hash が残る
```

これにより、import head 解決と simp registry は caller の import slice order に依存しません。
`checked_current_decls` は `0..MachineProofSpec.source_index` の完全 prefix かつ strictly increasing な
`source_index` と prior chain が一致する入力だけを有効にし、それ以外は `InvalidCurrentDeclOrder` として reject
するため、current declaration の依存順序が暗黙に入れ替わることはありません。
`start_machine_proof` は raw export block を検証済みとして扱ってはいけません。`VerifiedModule` は Phase 2
の verifier/session から返った値だけを受け取り、ここでは重複と signature/core declaration の整合性だけを
再確認します。

kernel environment は現行 kernel と同じく name keyed です。非 export dependency closure の private helper も
canonical name で追加します。同じ name かつ同じ canonical core declaration hash の entry は dependency
closure の重複として canonical dedup し、kernel environment には 1 entry だけ入れます。同じ name で異なる
core declaration / hash が複数 import または current declaration から来た場合は `AmbiguousKernelEnvDecl` として
reject します。Phase 4 AI では private helper の自動 rename はしません。

kernel environment の構築順は次で固定します。`VerifiedImportRef` は `(module, export_hash, certificate_hash)` の
canonical order に sort します。各 `VerifiedImportRef.certified_env_decls` は Phase 2 verifier が返す
dependency-topological canonical order のまま追加し、Phase 4 で並べ替えません。重複 declaration は
`name + canonical core declaration hash` が一致する場合だけ deterministic に skip します。全 import を追加した後、
`checked_current_decls` を source order で追加します。`certified_env_decls` が dependency-topological でないために
kernel env へ追加できない場合は、verified import payload が Phase 4 の前提を満たしていないものとして
`InvalidVerifiedImport` に写像します。

```json
{
  "goal_id": "g0",
  "context": [
    {"name": "n", "type": "Nat"}
  ],
  "target": "Eq.{0} Nat n n",
  "target_hash": "..."
}
```

---

# 6. Machine Tactic AST

MVP の tactic AST は固定 enum にします。

```rust
enum MachineTacticCandidate {
    Exact { term: RawMachineTerm },
    Intro { name: String },
    Apply { head: TacticHead, universe_args: Vec<Level>, args: Vec<CandidateApplyArg> },
    Rewrite { rule: CandidateRewriteRuleRef, direction: RewriteDirection, site: RewriteSite },
    SimpLite { rules: Vec<SimpRuleRef> },
    InductionNat { local_name: String },
}

struct RawMachineTerm {
    source: String,
}

enum CandidateApplyArg {
    Term(RawMachineTerm),
    Subgoal { name_hint: Option<String> },
    InferFromTarget,
}

struct CandidateRewriteRuleRef {
    head: TacticHead,
    universe_args: Vec<Level>,
    args: Vec<CandidateApplyArg>,
}

struct MachineTermSource {
    source: String,
    canonical_hash: Hash,
}

enum MachineTactic {
    Exact { term: MachineTermSource },
    Intro { name: String },
    Apply { head: TacticHead, universe_args: Vec<Level>, args: Vec<ApplyArg> },
    Rewrite { rule: RewriteRuleRef, direction: RewriteDirection, site: RewriteSite },
    SimpLite { rules: Vec<SimpRuleRef> },
    InductionNat { local_name: String },
}

enum TacticHead {
    Imported { name: Name, decl_interface_hash: Hash },
    CurrentModule { name: Name, decl_interface_hash: Hash },
    Local { name: String },
}

enum ApplyArg {
    Term(MachineTermSource),
    Subgoal { name_hint: Option<String> },
    InferFromTarget,
}

struct RewriteRuleRef {
    head: TacticHead,
    universe_args: Vec<Level>,
    args: Vec<ApplyArg>,
}

struct SimpRuleRef {
    name: Name,
    decl_interface_hash: Hash,
    direction: RewriteDirection,
}

struct SimpRegistry {
    rules: BTreeMap<SimpRuleKey, ResolvedSimpRule>,
}

struct SimpRuleKey {
    name: Name,
    decl_interface_hash: Hash,
    direction: RewriteDirection,
}

struct ResolvedSimpRule {
    key: SimpRuleKey,
    source: TacticHead,
    signature: CheckedDeclSignature,
    core_decl: Decl,
    theorem_ty: Expr,
    universe_params: Vec<String>,
    rule_telescope: Vec<ResolvedRuleParam>,
    eq_type: Expr,
    theorem_lhs: Expr,
    theorem_rhs: Expr,
    from_pattern: Expr,
    to_pattern: Expr,
}

struct ResolvedRuleParam {
    name: String,
    ty: Expr,
}

struct ResolvedEqFamily {
    origin: FamilyOrigin,
    eq: ResolvedFamilyHead,
    refl: ResolvedFamilyHead,
    rec: ResolvedFamilyHead,
}

struct ResolvedNatFamily {
    origin: FamilyOrigin,
    nat: ResolvedFamilyHead,
    zero: ResolvedFamilyHead,
    succ: ResolvedFamilyHead,
    rec: ResolvedFamilyHead,
}

enum ResolvedFamilyHead {
    Builtin { builtin_profile_id: String, name: Name, interface_tag: String },
    Decl { head: TacticHead, signature: CheckedDeclSignature, core_declaration_hash: Hash },
}

enum FamilyOrigin {
    Builtin { builtin_profile_id: String },
    Imported { module: ModuleName, export_hash: Hash, certificate_hash: Hash },
    CurrentSourceDecl { module: ModuleName, source_index: u64, core_declaration_hash: Hash },
}

enum RuleParamClassification {
    InferableTerm { name: String, ty: Expr },
    RejectedProofPremise,
    RejectedUninferableTerm,
}

enum RewriteSite {
    EqTargetLeft,
    EqTargetRight,
}

enum RewriteDirection {
    Forward,
    Backward,
}

struct MachineTacticOptions {
    simp_rules: Vec<SimpRuleRef>,
    eq_family: Option<EqFamilyRef>,
    nat_family: Option<NatFamilyRef>,
    max_simp_rewrite_steps: u64,
    max_open_goals: u64,
    max_metas: u64,
}

struct EqFamilyRef {
    eq_name: Name,
    eq_interface_hash: Hash,
    refl_name: Name,
    refl_interface_hash: Hash,
    rec_name: Name,
    rec_interface_hash: Hash,
}

struct NatFamilyRef {
    nat_name: Name,
    nat_interface_hash: Hash,
    zero_name: Name,
    zero_interface_hash: Hash,
    succ_name: Name,
    succ_interface_hash: Hash,
    rec_name: Name,
    rec_interface_hash: Hash,
}
```

`MachineTacticCandidate` は外部 JSON / parser 直後の非 canonical 入力です。candidate は raw term text だけを
持ち、term hash を受け取りません。外部 JSON に `canonical_hash` 風の field が含まれる場合、validator は
それを無視せず `InvalidMachineTactic` として reject します。term hash は必ず Phase 4 内部で
`canonicalize_machine_term_source` から計算します。

`MachineTermSource` は Phase 4 の内部 tactic AST が持つ term payload です。外部 JSON は raw Machine
Surface term text を渡してよいですが、`validate_machine_tactic_candidate` は実行前に Phase 3 AI M7 の
`canonicalize_machine_term_source(source)` を呼び、`source`、Phase 3 `canonical_bytes`、Phase 3
`canonical_hash = hash(canonical_bytes)` を得ます。Phase 4 は Phase 3 `canonical_hash` をそのまま
`MachineTermSource.canonical_hash` にしてはいけません。必ず下の Phase 4 wrapper bytes を組み立て、
`MachineTermSource.canonical_hash = hash(MachineTermSource canonical bytes)` とします。
whitespace、source span、pretty text、AI trace はこの hash に入りません。

```text
MachineTermSource canonical bytes:
  - tag "npa.phase4.machine-term-source.v1"
  - Phase 3 Machine Surface term-source canonical bytes

MachineTermSource canonical hash:
  - hash(MachineTermSource canonical bytes)
```

`run_machine_tactic` は `MachineTermSource.source` を Phase 3 の `elaborate_machine_term_check` に渡して
expected type に対して check しますが、tactic hash / cache key には `canonical_hash` だけを使います。
`canonicalize_machine_term_source` が失敗した candidate は `InvalidMachineTermSource` として reject します。
`MachineTermSource` の fields は public API では公開せず、`validate_machine_tactic_candidate` または
`MachineTermSource::new_checked(source)` だけが作れます。serialization boundary から既に canonicalized された
`MachineTactic` を受け取る debug/FFI API を作る場合でも、`run_machine_tactic` validation step 4 で
`source` を再 canonicalize して Phase 4 wrapper hash を再計算し、それが `canonical_hash` と一致しなければ
`InvalidMachineTermSource` として reject します。

`MachineTactic canonical bytes` は cache key と deterministic same-result 判定に使います。variant と field は
次の順で encode します。

```text
MachineTactic canonical bytes:
  - tag "npa.phase4.machine-tactic.v1"
  - variant tag:
      Exact / Intro / Apply / Rewrite / SimpLite / InductionNat
  - Exact:
      MachineTermSource canonical hash
  - Intro:
      UTF-8 name bytes
  - Apply:
      TacticHead canonical bytes
      universe_args in input order, using Phase 1 Level canonical bytes
      ApplyArg list in input order
  - Rewrite:
      RewriteRuleRef canonical bytes
      direction
      site
  - SimpLite:
      SimpRuleRef list after canonical sort and dedup by SimpRuleKey
  - InductionNat:
      UTF-8 local_name bytes

TacticHead canonical bytes:
  - Imported: name canonical bytes / decl_interface_hash
  - CurrentModule: name canonical bytes / decl_interface_hash
  - Local: UTF-8 local name bytes

ApplyArg canonical bytes:
  - Term: MachineTermSource canonical hash
  - Subgoal: variant tag only; name_hint is display-only and excluded
  - InferFromTarget

RewriteRuleRef canonical bytes:
  - TacticHead canonical bytes
  - universe_args in input order, using Phase 1 Level canonical bytes
  - ApplyArg list in input order

SimpRuleRef canonical bytes:
  - name canonical bytes / decl_interface_hash / direction
```

`MachineTermSource canonical hash` は上で定義した Phase 4 wrapper hash であり、その payload は Phase 3 AI M7 の
Machine Surface term-source canonical bytes です。Phase 4 は source text、pretty-printed term、parse span、
AI trace から tactic hash を作ってはいけません。
`SimpLite { rules }` の tactic hash は execution semantics と同じ canonicalized allowlist を使うため、
input order と duplicate count が違っても同じ canonical tactic hash になります。
`ApplyArg::Subgoal.name_hint` は UI / debug 表示だけの hint であり、fresh `MetaVarId`、`GoalId`、goal
context、target、proof skeleton、`proof_delta_hash`、`state_fingerprint`、canonical tactic hash には
入りません。subgoal の deterministic 名は `MetaVarId` から導出し、hint が違うだけの tactic は同じ canonical
tactic hash と同じ semantic result を持ちます。

外部 API では JSON schema にします。

```json
{
  "kind": "apply",
  "head": {
    "imported": {
      "name": "Eq.trans",
      "decl_interface_hash": "..."
    }
  },
  "universe_args": ["0"],
  "args": [
    {"mode": "infer_from_target"},
    {"mode": "subgoal", "name_hint": "h"}
  ]
}
```

term を含む tactic は、必ず Phase 3 の `elaborate_machine_term_check` を使って expected type に
対して check します。

`MachineTacticOptions` は `start_machine_proof` で canonicalize します。`simp_rules` は
`Name + decl_interface_hash + direction` の canonical order に sort して重複を落とし、参照先が
import / checked current declaration のどちらにも存在しなければ
`UnknownSimpRule` として reject します。`SimpRuleRef` は `TacticHead` を持たないため、同じ
`name + decl_interface_hash` が import と checked current declaration の両方、または複数 import に見つかる場合は
source を推測せず `AmbiguousSimpRule` として reject します。current declaration を優先したり、同 hash の候補を
自動 dedup したりしてはいけません。`eq_family = None` は kernel builtin の Eq head / primitives がある場合だけそれを使い、
なければ resolved Eq family を `None` にします。
`eq_family = Some(ref)` は builtin を指せず、`EqFamilyRef` のすべての head が verified import または checked
current declaration に `name + decl_interface_hash` で一意に存在することを確認します。`nat_family = None` は
`induction-nat` を無効にします。`nat_family = Some(ref)` は builtin を指せず、`NatFamilyRef` のすべての head が
verified import または checked current declaration に
`name + decl_interface_hash` で一意に存在することを確認します。`max_simp_rewrite_steps`,
`max_open_goals`, `max_metas` は 0 を許さず、実行中に超えた場合はそれぞれ
`SimpStepLimitExceeded` / `GoalLimitExceeded` / `MetaLimitExceeded` を返します。
canonicalize 後の `MachineTacticOptions` 本体は `MachineTacticEnv.options` に保存し、その canonical bytes hash を
`options_fingerprint` に保存します。`run_machine_tactic` は `MachineTacticEnv.options` だけを読み、呼び出し時に
別 options を受け取りません。
`options_fingerprint` は derived field です。`start_machine_proof` は必ず
`hash(MachineTacticOptions canonical bytes)` を保存し、`run_machine_tactic` の state validation は
`state.env.options_fingerprint` を再計算して一致しなければ `InvalidMachineProofState` として reject します。
`start_machine_proof` は options から Eq / Nat family を一度だけ解決し、結果を
`MachineTacticEnv.eq_family` / `MachineTacticEnv.nat_family` に保存します。`run_machine_tactic` は
`MachineTacticOptions.eq_family` / `nat_family` を再解決せず、resolved family fields だけを使います。
`eq_family = None` で kernel builtin Eq head / primitives がある場合、`MachineTacticEnv.eq_family` は builtin
profile に由来する `Some(ResolvedEqFamily)` です。`eq_family = None` かつ builtin Eq がない場合、
`MachineTacticEnv.eq_family` は `None` であり、`rw` / `simp-lite` は `TacticPrimitiveUnavailable` を返します。
`nat_family = None` の場合、`MachineTacticEnv.nat_family` は `None` であり、`induction-nat` は
`TacticPrimitiveUnavailable` を返します。

Eq は target shape recognition と proof primitive availability を分けて扱います。

```text
Eq target recognition head:
  - start_machine_proof 時:
      options.eq_family = Some(ref) の場合は ref.eq_name を resolved Eq head にする
      options.eq_family = None かつ builtin Eq がある場合は kernel builtin Eq head を resolved Eq head にする
      options.eq_family = None かつ builtin Eq がない場合は resolved Eq family を None にする
  - run_machine_tactic 時:
      MachineTacticEnv.eq_family が Some の場合だけその eq head を使う

Eq proof primitives:
  - MachineTacticEnv.eq_family が Some の場合だけ、その refl / rec を使う
```

`rw` / `simp-lite` は tactic-specific head / rule lookup が成功した後に Eq target recognition head を解決します。
recognition head が存在しない場合は target shape を判定できないため `TacticPrimitiveUnavailable` を返します。
recognition head が存在する場合だけ
current target を WHNF して Eq target かを判定し、head が一致しなければ `ExpectedEqTarget` を返します。
Eq target と判定できた後で、実際に必要になった resolved family の refl / rec availability を検査します。
`run_machine_tactic` の validation order では、state / goal / tactic AST validation の後、tactic-specific head /
rule lookup を Eq target recognition より先に行います。したがって `rw` の rule head が不正な場合、または
`SimpLite { rules }` の allowlist に未知 rule がある場合は、`MachineTacticEnv.eq_family = None` でも
`UnknownTacticHead` / `AmbiguousTacticHead` / `UnknownSimpRule` / `AmbiguousSimpRule` を先に返します。
head / rule lookup がすべて成功してから `MachineTacticEnv.eq_family = None` を検査し、その場合だけ
`TacticPrimitiveUnavailable` を返します。

`max_simp_rewrite_steps = 0`、`max_open_goals = 0`、`max_metas = 0` は `start_machine_proof` の
options canonicalize 時点で `InvalidTacticOption` として reject します。これは milestone 未実装を表す
`UnsupportedTacticOption` とは別であり、完成版 Phase 4 でも同じ診断を使います。

`max_open_goals` / `max_metas` は `assign_goal` の state 置換後の個数で判定します。`assign_goal` は mutation 前に
次を計算し、上限を超える場合は fresh meta allocation や assignment を行う前に reject します。

```text
final_open_goal_count =
  state.open_goals.len() - 1 + new_goal_specs.len()

final_meta_count =
  state.metavars.metas.len() + new_goal_specs.len()
```

`final_open_goal_count` は未代入 open goal 数だけを数えます。`final_meta_count` は代入済み meta も含む
`MetaVarStore` 内の総 meta 数を数えます。`max_meta_allocations` fuel とは別です。同時に複数の limit が
失敗し得る場合、`assign_goal` は mutation 前に次の順序で検査し、最初の error だけを返します。

```text
1. new_goal_specs.len() > remaining max_meta_allocations:
     TacticFuelExhausted { MetaAllocation }
2. final_open_goal_count > MachineTacticOptions.max_open_goals:
     GoalLimitExceeded
3. final_meta_count > MachineTacticOptions.max_metas:
     MetaLimitExceeded
```

`MachineTacticEnv.simp_registry` は実行時の authoritative simp registry です。これは
`MachineTacticEnv.options.simp_rules` を解決・canonicalize・deduplicate した結果であり、すべての entry が
verified import または checked current declaration に解決済みでなければなりません。各 `ResolvedSimpRule` は
rule の `CheckedDeclSignature`、kernel environment に入れる `core_decl`、WHNF した theorem type、universe
parameters、equality conclusion 前の `rule_telescope`、`eq_type`、theorem の元の `lhs` / `rhs`、direction 適用後の
`from_pattern` / `to_pattern` を保持します。`theorem_lhs`, `theorem_rhs`, `from_pattern`, `to_pattern` は
`rule_telescope` の context に相対的な core `Expr` です。`simp-lite` は実行時に `theorem_ty` を再解析せず、
`ResolvedSimpRule` の telescope と pattern だけから target への deterministic instantiation を行います。
`MachineTacticEnv.options.simp_rules` は caller が指定した canonical allowlist と state fingerprint 用に保持します。
`run_machine_tactic` は simp 実行時に `MachineTacticEnv.simp_registry` だけを registry として使い、
`options.simp_rules` を再解決しません。
`MachineTacticEnv.eq_family = None` の場合でも、`start_machine_proof` は `simp_rules` の head resolution、
signature / core declaration validation、`SimpRegistry` canonicalization を通常通り行います。非空 `simp_rules` を
reject してはいけません。`simp-lite` 実行時は allowlist 検査を先に行い、登録済み rule 参照であることを確認した後、
Eq family がないため `TacticPrimitiveUnavailable` を返します。

`SimpRegistry canonical bytes` は次で固定します。`rules` は `SimpRuleKey` の canonical order
(`name`, `decl_interface_hash`, `direction`) で encode し、`BTreeMap` の実装詳細や insertion order に依存してはいけません。

```text
SimpRuleKey canonical bytes:
  - name canonical bytes
  - decl_interface_hash
  - direction

SimpRegistry canonical bytes:
  - tag "npa.phase4.simp-registry.v1"
  - rule count
  - each ResolvedSimpRule in SimpRuleKey canonical order:
      SimpRuleKey canonical bytes
      source TacticHead canonical bytes
      CheckedDeclSignature canonical bytes
      core_declaration_hash
      theorem_ty Expr hash
      universe_params as ordered string list
      rule_telescope hash
      eq_type Expr hash
      theorem_lhs Expr hash
      theorem_rhs Expr hash
      from_pattern Expr hash
      to_pattern Expr hash
```

`SimpRegistry canonical hash` はこの bytes の hash です。state validation は `MachineTacticEnv.simp_registry` から
この bytes を再計算し、state fingerprint に入っている registry 内容と一致することを確認します。

`ResolvedSimpRule.universe_params` は `signature.universe_params` および `core_decl.universe_params()` から
再計算した、kernel `Decl` の ordered universe parameter 名列そのものです。Phase 4 は universe constraint を
独自に持たず、現行 core/kernel と同じく `Vec<String>` を canonical source とします。将来 core が
`UniverseContext { params, constraints }` を public declaration interface に入れる場合は、
`CheckedDeclSignature`、`ResolvedSimpRule`、state fingerprint、`SimpRuleRef` の interface hash 検査を同じ
変更単位で拡張します。Phase 4 だけで constraints を補ったり、string list から別順序へ並べ替えたりしてはいけません。

`RuleParamClassification` は simp registry 作成時に、theorem conclusion 前 telescope の各 term binder を
分類した結果です。universe binder は `ResolvedRuleParam` に入れず、`ResolvedSimpRule.universe_params` と
tactic 側 `universe_args` だけで扱います。

```text
InferableTerm:
  term binder で、型が proof sort ではなく、from_pattern または to_pattern に出現し、
  target 側 pattern match から一意に決まる可能性がある。実際に一意に決まるかは per-target で判定する。

RejectedProofPremise:
  proof sort の binder、または Eq conclusion 以外の premise。
  MVP の simp-lite は premise subgoal を生成しないため InvalidSimpRule として registry 作成時に reject する。

RejectedUninferableTerm:
  equality conclusion の lhs / rhs / type から原理的に推論できない term binder、または pattern に出現しない term binder。
  MVP では caller から simp rule 用 term 引数を受け取らないため InvalidSimpRule として reject する。
```

`rule_telescope` に残せる entry は `InferableTerm` だけです。`RejectedProofPremise` または
`RejectedUninferableTerm` に分類される binder を持つ rule は、`start_machine_proof` の simp registry 作成時点で
`InvalidSimpRule` として reject します。これにより MVP の `simp-lite` は proof premise subgoal や caller 指定の
rule-specific term argument を必要としません。

`eq_family = Some(ref)` の検査は head の存在確認だけでは足りません。`start_machine_proof` は
`EqFamilyRef` が一つの coherent family であることを次で確認します。

```text
- eq_name は verified import または checked current declaration として解決された Eq 型である
- Eq universe argument order は `[u]`
- eq_name の type は forall (A : Sort u), A -> A -> Sort 0 と conversion で一致する
- refl_name の type は次の Eq.refl interface と conversion で一致する:
    universe arguments: [u]
    term arguments:
      (A : Sort u) ->
      (x : A) ->
      eq_name.{u} A x x
- rec_name の type は次の Eq.rec interface と conversion で一致する:
    universe arguments: [u, v]
    term arguments:
      (A : Sort u) ->
      (a : A) ->
      (motive : forall (b : A), eq_name.{u} A a b -> Sort v) ->
      motive a (refl_name.{u} A a) ->
      (b : A) ->
      (h : eq_name.{u} A a b) ->
      motive b h
- refl / rec の declaration は eq_name と同じ verified import または checked current family に属する
```

最後の条件により、別 module の `Eq`, `Eq.refl`, `Eq.rec` を混ぜた `EqFamilyRef` は
`InvalidEqFamily` として reject します。

family origin は head 解決時に次のどれかとして記録します。

```text
FamilyOrigin:
  Builtin { builtin_profile_id }
  Imported { module, export_hash, certificate_hash }
  CurrentSourceDecl { module, source_index, core_declaration_hash }
```

`EqFamilyRef` の `eq_name` / `refl_name` / `rec_name` は、解決後の `FamilyOrigin` がすべて完全一致する場合だけ
「同じ family」とみなします。`Builtin` と imported/current declaration を混ぜてはいけません。
`FamilyOrigin::Builtin` は `eq_family = None` の default Eq recognition / primitive resolution にだけ使い、
`eq_family = Some(ref)` の `EqFamilyRef` 入力には使いません。
`ResolvedFamilyHead::Builtin` は `name` と stable `interface_tag` を保持し、`core_declaration_hash` を持ちません。
`ResolvedFamilyHead::Decl` は resolved `TacticHead`、再計算済み `CheckedDeclSignature`、Phase 2 core declaration
hash を保持します。resolved family は start 時に構成した後、run 時に name / hash から再解決してはいけません。
imported family の同一性は同じ `VerifiedImportRef` の `(module, export_hash, certificate_hash)` で判定します。
checked current family の同一性は、同じ source-level declaration から作られた `core_decl` とその generated
companion declaration closure に属すること、すなわち同じ
`CurrentSourceDecl { module, source_index, core_declaration_hash }` で判定します。current module で Eq / refl /
rec を別々の source declaration として定義したものは、Phase 4 MVP では同じ family として扱わず
`InvalidEqFamily` として reject します。

`nat_family = Some(ref)` の検査は head の存在確認だけでは足りません。`start_machine_proof` は
`NatFamilyRef` が一つの coherent family であることを次で確認します。

```text
- nat_name は verified import または checked current declaration として解決された inductive Nat 型である
- zero_name の type は nat_name と conversion で一致する
- succ_name の type は forall (_ : nat_name), nat_name と conversion で一致する
- rec_name の type は次の Nat.rec interface と conversion で一致する:
    universe arguments: [u]
    term arguments:
      (motive : nat_name -> Sort u) ->
      motive zero_name ->
      (forall (n : nat_name), motive n -> motive (succ_name n)) ->
      (n : nat_name) ->
      motive n
- zero / succ / rec の declaration は nat_name と同じ verified import または checked current family に属する
```

最後の条件により、別 module の `Nat`, `zero`, `succ`, `rec` を混ぜた `NatFamilyRef` は
`InvalidNatFamily` として reject します。

`NatFamilyRef` の `nat_name` / `zero_name` / `succ_name` / `rec_name` も `EqFamilyRef` と同じ
`FamilyOrigin` 完全一致規則を使います。imported Nat family は同じ `VerifiedImportRef` から来る必要があり、
checked current Nat family は同じ source-level inductive declaration の generated closure に属する必要があります。
`FamilyOrigin::Builtin` は `nat_family = Some(ref)` の `NatFamilyRef` 入力には使いません。
別々の current declarations で Nat / zero / succ / rec 風の head を用意したものは、Phase 4 MVP では
`InvalidNatFamily` として reject します。

resolved family canonical bytes は次で固定します。

```text
FamilyOrigin canonical bytes:
  - Builtin:
      tag "builtin"
      builtin_profile_id string
  - Imported:
      tag "imported"
      module name canonical bytes
      export_hash
      certificate_hash
  - CurrentSourceDecl:
      tag "current-source-decl"
      module name canonical bytes
      source_index
      core_declaration_hash

ResolvedFamilyHead canonical bytes:
  - Builtin:
      tag "builtin"
      builtin_profile_id string
      name canonical bytes
      interface_tag string
  - Decl:
      tag "decl"
      TacticHead canonical bytes
      CheckedDeclSignature canonical bytes
      core_declaration_hash

Builtin family interface_tag values:
  - Eq:
      "npa.phase4.builtin.eq.v1"
  - Eq.refl:
      "npa.phase4.builtin.eq.refl.v1"
  - Eq.rec:
      "npa.phase4.builtin.eq.rec.v1"
  - Nat:
      "npa.phase4.builtin.nat.v1"
  - Nat.zero:
      "npa.phase4.builtin.nat.zero.v1"
  - Nat.succ:
      "npa.phase4.builtin.nat.succ.v1"
  - Nat.rec:
      "npa.phase4.builtin.nat.rec.v1"

Builtin family stable Name values:
  - Eq:
      `NPA.Builtin.Eq`
  - Eq.refl:
      `NPA.Builtin.Eq.refl`
  - Eq.rec:
      `NPA.Builtin.Eq.rec`
  - Nat:
      `NPA.Builtin.Nat`
  - Nat.zero:
      `NPA.Builtin.Nat.zero`
  - Nat.succ:
      `NPA.Builtin.Nat.succ`
  - Nat.rec:
      `NPA.Builtin.Nat.rec`

これらの builtin stable `Name` は `ResolvedFamilyHead::Builtin` の canonical bytes 専用です。
import / current declaration の `TacticHead` 解決には参加せず、source declaration が同名を定義しても
`ResolvedFamilyHead::Builtin` として扱ってはいけません。

ResolvedEqFamily canonical bytes:
  - tag "npa.phase4.resolved-eq-family.v1"
  - FamilyOrigin canonical bytes
  - eq / refl / rec ResolvedFamilyHead canonical bytes in this order

ResolvedNatFamily canonical bytes:
  - tag "npa.phase4.resolved-nat-family.v1"
  - FamilyOrigin canonical bytes
  - nat / zero / succ / rec ResolvedFamilyHead canonical bytes in this order

MachineTacticEnv resolved family bytes:
  - eq_family option: none tag, or some tag + ResolvedEqFamily canonical bytes
  - nat_family option: none tag, or some tag + ResolvedNatFamily canonical bytes
```

`UnsupportedTacticOption` は milestone 実装中の一時的な診断です。M1-only implementation は
`eq_family = Some(_)` または `nat_family = Some(_)` を `UnsupportedTacticOption` として reject してよいですが、
M4 完了後の `eq_family = Some(_)` は必ず coherent family 検査まで進め、失敗時は `InvalidEqFamily` を返します。
M5 完了後の `nat_family = Some(_)` も同様に `InvalidNatFamily` を返します。完成版 Phase 4 では
`UnsupportedTacticOption` を `eq_family` / `nat_family` の代替エラーとして使ってはいけません。

head 解決は次の通り固定します。

```text
Imported:
  verified import の export block に name + decl_interface_hash がちょうど1つある場合だけ成功。
  同名別hash、同hash別module、複数一致は AmbiguousTacticHead。

CurrentModule:
  state.env.checked_current_decls の signature に name + decl_interface_hash がちょうど1つある場合だけ成功。
  現在の compilation unit で既に kernel check 済みの declaration だけ参照できる。
  current module の未検査 declaration や future declaration は参照しない。

Local:
  goal context 内の local name に完全一致する場合だけ成功。
  0 件なら UnknownLocalName、local name が重複していれば AmbiguousLocalName。
```

`TacticHead::Local` は name cardinality だけを解決し、local let を proof head として暗黙展開しません。
`apply` / `rw` の local head は `MachineLocalDecl.value = None` の local assumption だけを許し、
`value = Some(_)` の local let が一意に見つかった場合は `InvalidLocalHead` を返します。local let の value は
kernel conversion / WHNF が context を読む範囲では使えますが、tactic head としては使いません。
`induction-nat` は local let を tactic-specific に `InvalidInductionTarget` へ写像します。

`parse_machine_tactic_candidate` のような文字列 parser は補助 API として持ってよいですが、AI 向けの
主入口は structured JSON / AST validation です。文字列 tactic を直接 trusted execution に渡しては
いけません。

---

# 7. Tactic Semantics

## 7.1 exact

`exact` は現在 goal の target に対して term を check し、通れば goal を閉じます。

```text
goal:
  Γ ⊢ T

exact t:
  check Γ ⊢ t : T
```

completion:

```text
- Machine Surface term が expected target に check される
- 成功時に new_goals が空
- 失敗時に state が変わらない
```

## 7.2 intro

`intro` は target の WHNF が Pi の場合だけ通します。

```text
Γ ⊢ forall (x : A), B
↓ intro x
Γ, x : A ⊢ B
```

completion:

```text
- Pi target だけ受け付ける
- local name が既存 local / global root を shadow しない
- 作られる proof term は lambda
```

## 7.3 apply

`apply` は theorem / local hypothesis の型を WHNF し、結論が target と conversion で合う場合に
premise を subgoal にします。

```text
h : forall (x : A), P x -> Q x
goal: Q a
apply h with [InferFromTarget, Subgoal]
new goals:
  P a
```

AI MVP では implicit insertion をしません。ただし `apply` の使い勝手を保つため、binder ごとに
argument policy を明示します。

```text
ApplyArg::Term(t):
  既に消費した universe / term argument を代入した binder domain に対して t を check して使う。

ApplyArg::Subgoal:
  既に消費した universe / term argument を代入した binder domain を新しい goal にする。
  proof term には対応する metavariable を入れる。

ApplyArg::InferFromTarget:
  target との conversion / first-order pattern match から一意に決まる場合だけ使う。
  一意でなければ AmbiguousApplyArgument。
```

ルール:

```text
- AI が指定していない explicit binder を勝手に埋めない
- universe_args の個数が head の universe parameter 数と一致しない場合は UniverseArgumentMismatch
- Subgoal は binder domain が Prop / proposition と判定できる場合だけ許す
- TacticHead::Local は local assumption だけを proof head にでき、local let は InvalidLocalHead
- target から一意に決まる index / parameter だけ InferFromTarget を許す
- 複数候補または未決定なら MissingExplicitArgument / AmbiguousApplyArgument
- 作られる subgoal の順序は binder order
```

`Subgoal` の許可条件:

```text
- binder domain の型を kernel で infer し、WHNF が Prop または Sort 0 であること
- domain が Type / Sort u (u > 0) のデータ引数なら SubgoalDataArgument として reject
- type parameter / index parameter は Term または InferFromTarget で解決する
```

`args` は WHNF した head type の Pi binder を左から順に消費します。MVP では named argument は
ありません。

```text
while head type WHNF is Pi:
  if current result type has no unresolved PatternMetaId
     and is convertible to target:
    if next ApplyArg exists:
      TooManyApplyArguments
    stop and close the original goal
  else if next ApplyArg exists:
    consume that binder with Term / Subgoal / InferFromTarget
  else:
    break and proceed to after args are exhausted

after args are exhausted:
  if result type has unresolved PatternMetaId:
    run InferFromTarget batch matcher
    substitute solved PatternMetaId into all pending args and result type
  if result type is convertible to target:
    close the original goal
  else if result type WHNF is still Pi:
    TooFewApplyArguments
  else:
    TypeMismatch
```

`TooFewApplyArguments` は、caller-supplied `args` をすべて消費した後も result type の WHNF が Pi の場合に返します。
`MissingExplicitArgument` は、caller が `InferFromTarget` を指定したがその parameter が result pattern に
出現せず target から推論できない場合に返します。単に `args` が足りないだけのケースを
`MissingExplicitArgument` にしてはいけません。

`InferFromTarget` が 1 つでもある場合、binder 消費中に即時確定せず、まず temporary `PatternMetaId` を
その binder の引数位置に入れた spine を作ります。`Term` / `Subgoal` の expected domain が未解決
`PatternMetaId` を含む場合、その term check や fresh meta allocation は batch inference の後まで遅延します。
すべての caller-supplied `args` を消費した後、WHNF した result pattern と target を 1 回だけ match し、
すべての `PatternMetaId` を同時に解決します。解決後、binder order で各 argument を再検査します。

```text
InferFromTarget batch order:
  1. Term / Subgoal / InferFromTarget を left-to-right に spine へ入れる
  2. InferFromTarget は unresolved PatternMetaId として記録する
  3. args 消費後の result pattern を WHNF する
  4. result pattern と goal target を 1 回だけ match し、全 PatternMetaId を解決する
  5. 解決した term を binder order で domain check する
  6. Subgoal の fresh meta allocation はすべての PatternMetaId 解決後に行う
```

`PatternMetaId` が result pattern に出現しない場合は `MissingExplicitArgument` を返します。
result pattern には出現するが一意の core term に決まらない場合、または同じ `PatternMetaId` の複数出現が
kernel conversion で一致しない場合は `AmbiguousApplyArgument` を返します。解決した term が binder domain に
check できない場合は `TypeMismatch` を返します。

`InferFromTarget` は探索ではなく保守的な matcher です。

```text
- matcher input は WHNF した expected result pattern と target
- pattern variable は InferFromTarget が作った temporary PatternMetaId のみ
- pattern variable は `Const c args` または local rigid head の引数位置にだけ出てよい
- pattern variable が関数 head、binder の下、let body の下、または別の pattern variable の中に出たら reject
- 同じ pattern variable が複数回出た場合は kernel conversion で同じ term に一致する必要がある
- matcher は left-to-right に1回だけ走り、backtracking しない
- β/ζ/δ/ι conversion は比較時に kernel conversion を呼ぶが、候補探索には使わない
```

`InferFromTarget` の `PatternMetaId` は matcher 内だけの一時変数であり、`MetaVarStore` に登録される
open goal ではありません。すべての `PatternMetaId` はその場で一意の core term に解決され、代入後の
binder domain に対して kernel check されます。未解決の `PatternMetaId` が残る場合は
`AmbiguousApplyArgument` または `MissingExplicitArgument` として reject します。

`Subgoal` が作る metavariable は MachineProofState の `MetaVarStore` にだけ置かれます。closed proof
extraction は未解決 metavariable が 1 つでも残っていれば `UnresolvedGoal` を返します。

これにより、`apply h` 相当を使いたい場合でも、AI は「どの引数を subgoal にするか」を構造化して
明示します。

## 7.4 rw

`rw` は Eq theorem による、等式 target の片側全体の rewrite に限定します。

```text
rule : Eq A lhs rhs
direction: forward | backward
```

rule は `Name` だけでは不十分です。多相 theorem や `forall` を持つ rewrite rule は、rewrite 前に
rule term を instantiate して、最終的に次の形へ WHNF できる必要があります。

```text
Eq.{u} A lhs rhs
```

`RewriteRuleRef` は `apply` と同じ `universe_args` / `args` policy を使います。

```text
- universe args は明示
- theorem parameters は Term / InferFromTarget / Subgoal のいずれか
- lhs/rhs が target から一意に決まらない場合は AmbiguousRewriteRule
- proof-relevant premise が残る rule は、その premise を subgoal にするか reject
- premise subgoal は rule binder order で生成する
```

`RewriteRuleRef` instantiation の diagnostic mapping は固定します。

```text
head resolution failure:
  UnknownTacticHead / AmbiguousTacticHead

universe argument count mismatch:
  UniverseArgumentMismatch

Term argument domain check failure:
  TypeMismatch

Subgoal on data/type/index argument:
  SubgoalDataArgument

extra argument after Eq conclusion is reached:
  TooManyApplyArguments

caller args are exhausted before Eq conclusion and remaining binders cannot be inferred from target:
  AmbiguousRewriteRule

InferFromTarget is missing from lhs/rhs pattern:
  AmbiguousRewriteRule

InferFromTarget has multiple non-convertible target matches:
  AmbiguousRewriteRule

instantiated theorem type does not WHNF to Eq after all accepted arguments:
  AmbiguousRewriteRule
```

`rw` は `TooFewApplyArguments` / `MissingExplicitArgument` を返しません。apply と同じ binder walking
subroutine を使う実装でも、rewrite rule の lhs/rhs instantiation に関する不足・曖昧さはすべて
`AmbiguousRewriteRule` に写像します。

`rw` の new goals は常に次の順序にします。

```text
1. RewriteRuleRef.args の Subgoal が作る premise goals, binder order
2. rewritten target goal
```

old goal の assignment は `mk_rewrite_transport(rule_proof, direction, site, old_target, new_target, p_new)` です。
`rule_proof` は premise goals の metavariable を含んでよく、`p_new` は最後に生成される rewritten target goal の
metavariable です。

rewrite site は MVP でも固定 enum にします。

```rust
enum RewriteSite {
    EqTargetLeft,
    EqTargetRight,
}
```

```text
target:
  Eq.{v} B target_lhs target_rhs

EqTargetLeft:
  selected_side = target_lhs

EqTargetRight:
  selected_side = target_rhs

direction forward:
  from = lhs
  to = rhs

direction backward:
  from = rhs
  to = lhs

rewrite succeeds only if:
  - target WHNF が Eq target である
  - rule instantiated type WHNF が Eq A lhs rhs である
  - A と B が kernel conversion で一致する
  - selected_side と from が kernel conversion で一致する

new target:
  Eq B new_lhs new_rhs
  where:
    - Eq head / universe / type argument B は old_eq_target 由来のものを保持する
    - selected_side が left なら new_lhs = to, new_rhs = target_rhs
    - selected_side が right なら new_lhs = target_lhs, new_rhs = to
```

`rw` は元の target 構文を保持しません。まず assigned goal の `old_target` を WHNF して
`old_eq_target = Eq B target_lhs target_rhs` を得ます。`selected_side` と `new_target` はこの
`old_eq_target` からだけ作ります。transparent alias や reducible definition 経由で Eq target になる場合でも、
rewritten target goal は WHNF 後の Eq target の片側を置換した canonical shape になります。
rule 側の type argument `A` と target 側の `B` が conversion で一致する場合でも、`new_target` は target 側の
`Eq B ...` を保持し、rule 側の `Eq A ...` へ置き換えません。`to` は `B` の要素として kernel conversion で
check できる必要があります。
`mk_rewrite_transport` が作る proof skeleton は `old_eq_target` を expected type として
`check_proof_expr_with_metas` で検査し、その結果が元の `old_target` と conversion で一致することを確認します。
metavariable を含まない closed transport でも、`run_machine_tactic` の成功/失敗は `assign_goal` の
proof skeleton check にだけ基づけます。実装が debug assertion として kernel check する場合でも、その結果で
diagnostic、state、hash を変えてはいけません。正式な kernel check は `extract_closed_machine_proof` で行います。

MVP では selected side の内部部分式 rewrite、target が Eq でない命題の rewrite、hypotheses の rewrite、
binder の下の rewrite、dependent rewrite、setoid rewrite は拒否します。rewrite proof は selected side を
抽象化した motive に対する Eq.rec の単一 step として生成します。congruence proof builder と
occurrence index selector は future work です。

M4 の `rw` / `simp-lite` は Eq target recognition head と Eq proof primitives を使います。
利用可能な family は次のどちらかです。

```text
- kernel builtin / generated recursor として EqFamilyRef と同じ Eq / Eq.refl / Eq.rec interface がある
- MachineTacticOptions.eq_family に coherent EqFamilyRef が固定されている
```

どちらの Eq target recognition head もない environment では、`start_machine_proof` は
`MachineTacticEnv.eq_family = None` を保存し、`rw` / `simp-lite` は target shape を判定できないため
`TacticPrimitiveUnavailable` を返します。recognition head がある場合、`start_machine_proof` は
`MachineTacticEnv.eq_family = Some(resolved family)` を保存し、`rw` / `simp-lite` は current target を
WHNF して Eq target かを判定し、target head が recognition head と一致しなければ `ExpectedEqTarget` を返します。
Eq target であることが確定した後、実際に必要になった resolved family の refl / rec availability を検査し、欠けていれば
`TacticPrimitiveUnavailable` を返します。

`rw` は current goal を `old_target` から `new_target` に置き換えます。proof delta は、new goal の proof
`p_new : new_target` と rule proof から `p_old : old_target` を作る transport です。

```text
mk_rewrite_transport(
  rule_proof,
  direction,
  site,
  old_target,
  new_target,
  p_new,
) : old_target
```

この helper は resolved `family.rec` の motive を内部で構成し、`p_old` の proof skeleton を生成します。`rule_proof` や
`p_new` が metavariable を含む場合、生成直後に kernel check するのではなく、`assign_goal` の
`check_proof_expr_with_metas` で expected type に対して skeleton check します。metavariable がすべて閉じた後、
`extract_closed_machine_proof` が meta を消去した core `Expr` を theorem type に対して kernel check します。
metavariable を含まない closed transport を生成する場合でも、tactic execution 中の diagnostic は
proof skeleton check の結果に固定します。kernel check の失敗を tactic execution の別 error にしてはいけません。
MVP の rewrite proof generation は `Eq.trans` / `Eq.symm` のような helper theorem を使いません。
それらが verified import / checked current declaration として存在しても、Phase 4 は resolved `family.rec` から直接
proof を構成します。将来 helper theorem を使う場合は、`MachineTacticOptions`、state fingerprint、
canonical proof generation rule を同じ変更単位で拡張します。

MVP の transport は常に次の Eq.rec 正規形で生成します。ここで `Eq`, `Eq.refl`, `Eq.rec` は
literal builtin 名ではなく、必ず `MachineTacticEnv.eq_family = Some(family)` の
`family.eq` / `family.refl` / `family.rec` を表す schematic notation です。`family` が builtin 由来なら builtin head を、
`EqFamilyRef` 由来なら verified import / checked current declaration の resolved head を使います。
`MachineTacticEnv.eq_family = None` の状態でこの proof generation に入ってはいけません。

```text
1. rule_proof_B を作る
   - B は old_eq_target 由来の target 側 type argument
   - rule proof の元の型 Eq A lhs rhs は、A と B の conversion により
     同じ proof expression を Eq B lhs rhs として検査したもの
   - proof expression が metavariable を含む場合は skeleton check、closed の場合は kernel check を使う
   - proof term に coercion node は挿入しない
2. rule proof を direction に従って h_from_to : Eq B from to に正規化する
   - forward:  h_from_to = rule_proof_B
   - backward: h_from_to = mk_eq_symm(rule_proof_B)
3. p_new : P to から p_old : P from を作るため、h_to_from : Eq B to from を用意する
   - forward:  h_to_from = mk_eq_symm(rule_proof_B)
   - backward: h_to_from = rule_proof_B
4. P(x) は old_eq_target の selected side を x に置き換えた motive
5. p_old = family.rec B to (fun x _ => P(x)) p_new from h_to_from
```

`old_target` 引数は assigned goal の元 target を保持しますが、Eq.rec の motive は常に WHNF 後の
`old_eq_target` から作ります。元の `old_target` と `old_eq_target` の変換可能性は kernel conversion で確認し、
transport 全体の proof skeleton が元の goal target に check されることで alias 経由の Eq target も扱います。
canonical proof generation は target 側の `Eq B ...` に揃え、rule 側の `Eq A ...` を transport の主 family として
使いません。

`mk_eq_symm` も resolved family の rec / refl から直接作ります。

```text
mk_eq_symm(h : Eq C a b) : Eq C b a =
  family.rec.{u, 0} C a
    (fun b _ => family.eq.{u} C b a)
    (family.refl.{u} C a)
    b h
```

ここで `Eq.rec` の second universe argument `0` は、`EqFamilyRef` coherent family 検査で
`Eq A b a : Sort 0` と固定しているためです。Phase 4 MVP の equality は Prop-valued Eq だけを扱い、
`Eq : Sort u -> A -> A -> Sort v` のような proof universe-polymorphic equality family は
`InvalidEqFamily` として reject します。

生成した `h_from_to`, `h_to_from`, `p_old` は、metavariable を含む間は proof skeleton として検査し、closed になった後で
kernel check します。MVP の canonical proof generation は常に上の Eq.rec 由来の構成だけを使います。

## 7.5 simp-lite

`simp-lite` は証明生成する小さい rewrite engine です。

MVP:

```text
- registered simp rule のみ使用
- β/ζ/δ/ι WHNF と Eq.refl closure
- equality target の左辺/右辺全体の rewrite のみ
- rewrite step 数に上限
- 成功時は proof term を生成
```

rule registry は deterministic にします。

```rust
MachineTacticEnv.simp_registry: SimpRegistry
```

登録元:

```text
- MachineTacticOptions.simp_rules
- verified import の export block で公開され、SimpRuleRef と一致する rule
```

standard library index は Phase 4 API の入力ではありません。Phase 5 / Phase 7 が standard library index を
lookup accelerator として使う場合でも、Phase 4 に渡す前に対応 module を Phase 2 verifier で
`VerifiedModule` にし、使う rule を `MachineTacticOptions.simp_rules` に `Name + decl_interface_hash +
direction` として固定します。`start_machine_proof` 後に `run_machine_tactic` が file / network lookup や
未検証 certificate の追加を行うことはありません。

Machine Surface MVP には source-level simp rule 登録 syntax を入れません。current module の theorem を
simp rule として使いたい場合は、Phase 5 / Phase 7 の呼び出し側が `MachineTacticOptions.simp_rules`
に `Name + decl_interface_hash + direction` を渡します。

実行順:

```text
1. SimpRuleKey の canonical order
2. 同じ key の重複は deduplicate
3. step limit を超えたら SimpStepLimitExceeded
```

`simp-lite` の tactic input は rule name だけでなく `decl_interface_hash` を持ちます。これにより、
同名 rule や import slice order によって結果が変わることを防ぎます。

`SimpLite { rules }` の `rules` は tactic ごとの allowlist です。空の場合は `state.env.simp_registry` の全 rule を
使います。空でない場合、各 `SimpRuleRef` は `state.env.simp_registry` に登録済みでなければ
`UnknownSimpRule` として reject します。実行時は allowlist を `SimpRuleKey` の canonical order に並べ替え、
同じ `SimpRuleKey` が複数回出る場合は 1 件に canonical dedup します。重複を error にしてはいけません。
input order と input duplicate count には依存しません。

`simp-lite` は rule に `ApplyArg` を持たせません。rule instantiation は次だけを許します。

```text
- universe parameter は selected side / target から一意に推論できる場合だけ使う
- forall parameter は selected side / target から一意に推論できる場合だけ使う
- proof-relevant premise が equality conclusion 前に残る rule は InvalidSimpRule
- 推論できない parameter がある rule は、その target には not applicable として次の rule を試す
```

`simp-lite` の per-target instantiation algorithm は次の順序で固定します。

```text
1. current target を WHNF し、Eq target recognition head に一致する Eq A target_lhs target_rhs だけを扱う
2. RewriteSite と rule direction から selected_side と from_pattern / to_pattern を決める
3. rule_telescope の各 term binder に PatternMetaId を作り、universe_params の各 entry に UniversePatternId を作る
4. rule の eq_type と target の A、from_pattern と selected_side を左から右へ matching する
5. matching は backtracking しない。binder 下、let 下、function head 位置の PatternMetaId 出現は not applicable
6. 同じ PatternMetaId / UniversePatternId の複数出現は kernel conversion / level equality で一致する場合だけ受理する
7. 未解決の universe parameter、または conflicting universe solution があれば not applicable
8. 未解決の term parameter、または conflicting / non-convertible term solution があれば not applicable
9. telescope binder order で term solution を domain に対して skeleton check / kernel check する
10. 解決済み universe / term solution を theorem_lhs / theorem_rhs / to_pattern に代入して candidate target を作る
```

この matcher は rule ごとの hidden search を持ちません。WHNF と conversion は `TacticBudget` の
`max_whnf_steps` / `max_conversion_steps` を消費し、尽きた場合は `TacticFuelExhausted` を返します。
not applicable は rewrite count、`max_simp_rewrite_steps`、`max_rewrite_steps` fuel を消費しません。

conditional simp rule、premise subgoal を生成する simp rule、rule-specific explicit arguments は MVP では扱いません。

`InvalidSimpRule` と not applicable の境界は次で固定します。

```text
registration-time InvalidSimpRule:
  - rule head が theorem / reducible proof declaration として解決できない
  - conclusion を WHNF しても Eq 型にならない
  - equality conclusion より前に proof-relevant premise が残る
  - explicit premise subgoal なしでは使えない前提を持つ
  - equality conclusion の lhs / rhs / type から原理的に推論できない parameter を持つ

per-target not applicable:
  - rule 自体は有効だが、現在 target の selected side と pattern が一致しない
  - parameter は rule の equality conclusion に現れるが、この target からは一意に決まらない
  - universe parameter がこの target では一意に決まらない
```

`registration-time` 検査は `start_machine_proof` で registry を作る時点で行います。`per-target not applicable` は
error ではなく次の canonical ordered rule / site を試します。rewrite count が 0 のまま、すべての rule / site が
not applicable の場合だけ、実行結果として `SimpNoProgress` を返します。

MVP の `simp-lite` は target 内部分式を直接書き換えません。現在 target を WHNF し、target が
`Eq A lhs rhs` であれば、登録 rule を canonical order で試し、各 rule について `EqTargetLeft`、
`EqTargetRight` の順に片側全体の rewrite を試します。各 step は `rw` と同じ proof-producing primitive に
落とし、複数 step の証明は nested `mk_rewrite_transport` として構成します。
1 回の rewrite に成功したら、更新後 target を WHNF し、次の探索 iteration では必ず canonical ordered rule / site の
先頭から再走査します。直前に成功した rule の次から続けてはいけません。

`simp-lite` で rewrite candidate が applicable とみなされるのは、`rw` と同じ条件を満たし、かつ
`current_whnf_eq_target_hash` と `candidate_whnf_new_target_hash` が異なる場合だけです。
`current_whnf_eq_target_hash` は、その探索 iteration の先頭で current target を WHNF して得た
`Eq A lhs rhs` の canonical core hash です。`candidate_whnf_new_target_hash` は、candidate から作った
`new_target` を WHNF した canonical core hash です。元 goal の target hash や alias 展開前の target hash と
比較してはいけません。`x = x` のように selected side と `to` が conversion で一致しても
WHNF 後 target hash が変わらない rule は per-target not applicable として扱い、rewrite count、
`max_simp_rewrite_steps`、`max_rewrite_steps` fuel を消費しません。

`simp-lite` は rule を試す前と各 rewrite step 後に target を WHNF し、まず target が `Eq A t u` かを
判定します。Eq でなければ `ExpectedEqTarget` を返し、この場合は Eq primitive availability を調べません。
Eq target で `t` と `u` が kernel conversion で一致すれば `Eq.refl` で閉じます。初期 target がすでに
Eq.refl closure で閉じる場合は rewrite count 0 の成功であり、`new_goal_specs = []`、`SimpNoProgress` では
ありません。この 0 rewrite closure には `Eq.refl` だけが必要で、`Eq.rec` は要求しません。`Eq.refl` が
verified import または kernel builtin として利用できない場合は `TacticPrimitiveUnavailable` を返します。
proof-producing rewrite candidate が progress-making と判定された時点で、`rw` と同じく `Eq.rec` / `Eq.refl` の
availability を検査し、利用できなければ `TacticPrimitiveUnavailable` を返します。progress-making candidate が
1 つもない full scan が終わった場合、`Eq.rec` が利用できない environment でも次の分岐だけを使います。

```text
- rewrite_count == 0:
    SimpNoProgress
- rewrite_count > 0:
    success; last rewritten target を 1 つの new goal として返す
```

つまり、1 回以上 rewrite した後、Eq.refl closure では閉じられないが canonical ordered rule / site をすべて試して
次の applicable candidate がない場合だけ、`simp-lite` は fixed point success になります。
old goal の assignment は nested `mk_rewrite_transport` で、その new goal の proof metavariable から old target の
proof を構成します。

`MachineTacticOptions.max_simp_rewrite_steps` 回の proof-producing rewrite に成功しても Eq.refl closure で
閉じられない場合、追加の applicability scan は行わず、ただちに `SimpStepLimitExceeded` を返します。
これは semantic limit であり、`TacticBudget.max_rewrite_steps` の fuel exhaustion とは別です。fuel が先に尽きた場合は常に
`TacticFuelExhausted { kind: Rewrite }` を返し、`SimpStepLimitExceeded` には写像しません。binder の下、hypothesis、setoid
relation、congruence による subterm rewrite は Phase 4 AI MVP の外です。

## 7.6 induction-nat

`induction-nat` は local context にある `Nat` 変数だけを対象にし、`Nat.rec` proof term を構成します。
`InductionNat { local_name }` は `TacticHead::Local` と同じ local 解決規則を使います。goal context 内の
`MachineLocalDecl.name` に完全一致する local がちょうど 1 件だけある場合に成功し、0 件なら `UnknownLocalName`、
2 件以上なら `AmbiguousLocalName` を返します。対象 local は `value = None` の assumption でなければならず、
local let declaration は induction target にできません。

MVP:

```text
- target motive を構造的に生成できる場合だけ
- base / step の 2 goal を生成
- generalization はしない
- induction 対象 local は MVP では context の最後の assumption に限定する
- 使う Nat family は MachineTacticEnv.nat_family で固定する
```

generalization と context reordering をしないため、MVP では次を拒否します。

```text
- induction target より後ろに local declaration がある
- target motive を `fun n => target[n]` として構造的に抽出できない
- target variable が local context に直接ない
- target variable name が複数 local に一致する
- target variable が local let declaration である
- MachineTacticEnv.nat_family が None
- target variable の型が ResolvedNatFamily.nat と conversion で一致しない
- ResolvedNatFamily の zero / succ / rec head が壊れている
```

`induction-nat` の static validation order と diagnostic mapping は次で固定します。

```text
1. local_name が goal context に 0 件:
     UnknownLocalName
2. local_name が goal context に 2 件以上:
     AmbiguousLocalName
3. resolved local が value = Some(_) の let declaration:
     InvalidInductionTarget
4. resolved local より後ろに local declaration がある:
     InvalidInductionTarget
5. MachineTacticEnv.nat_family が None:
     TacticPrimitiveUnavailable
6. resolved local の型が ResolvedNatFamily.nat と conversion で一致しない:
     InvalidInductionTarget
7. target motive を対象 local の de Bruijn occurrence から構造的に抽出できない:
     InvalidInductionTarget
8. Nat.zero / Nat.succ / Nat.rec resolved family head が missing または壊れている:
     InvalidMachineProofState
```

step 8 は public constructor から作った state では発生しない防御用です。`start_machine_proof` で
`NatFamilyRef` の coherent family 検査に失敗した場合は `InvalidNatFamily` を返し、壊れた resolved family が
`run_machine_tactic` に到達した場合だけ `InvalidMachineProofState` に写像します。

term 生成規則:

```text
入力 goal:
  Γ, n : Nat ⊢ target

motive:
  P := fun (n_ind : Nat) => target[n_ind / n]

base goal:
  Γ ⊢ P Nat.zero

step goal:
  Γ, n : Nat, n_i : Nat, ih_i : P n_i ⊢ P (Nat.succ n_i)

assignment:
  Nat.rec P base (fun (n_i : Nat) => fun (ih_i : P n_i) => step) n : target
```

`Nat`, `Nat.zero`, `Nat.succ`, `Nat.rec` は `start_machine_proof` で
`MachineTacticEnv.nat_family` に固定された resolved family head だけを使います。複数の Nat 風 inductive が
import されていても short name や shape から推測しません。`nat_family = None` の state で `InductionNat` を実行した場合は
`TacticPrimitiveUnavailable` を返します。

`P` の universe level は `target` を kernel infer して得た sort level から作ります。`target[n_ind / n]` は
対象 local の de Bruijn occurrence だけを抽象化し、同名表示名による置換はしません。`base` と `step` は
それぞれ fresh meta として作り、`assign_goal` には `new_goal_specs = [base_goal, step_goal]` の順で渡します。
`base` meta の context は `Γ`、`step` meta の context は `Γ, n : Nat, n_i : Nat, ih_i : P n_i` です。
assignment では `step` meta を直接 `Nat.rec` に渡さず、`fun n_i => fun ih_i => Meta(step)` の
ProofExpr wrapper を作ります。MVP は context projection を持たないため、step goal には元の induction target
`n` を残します。

step goal に残る元の `n` は ordinary local assumption として扱います。つまり step proof は外側の
`n` を参照してもよく、その場合でも生成される proof term は `Γ, n : Nat` の下の
`Nat.rec P base step n` として kernel check されます。Phase 4 MVP は「一般化された induction principle」を
生成しているのではなく、現在 goal の context にある `n` に対する recursor application を生成しているだけです。
AI / caller は step goal が外側の `n` から独立していると仮定してはいけません。`P` と step target 自体は
元の `n` に依存しない shape ですが、step proof term が外側の `n` を参照することは許されます。
元の `n` を step context から落とす強い induction step は future work です。
生成した `Nat.rec P base (fun n_i => fun ih_i => step) n` は、`base` / `step` meta が open の間は
assigned goal の context で proof skeleton として検査し、closed extraction 後に kernel check します。

生成する base / step goal の local name は deterministic にします。ユーザー名と衝突する場合は
`n0`, `ih0`, `n1`, `ih1` のように番号を増やし、同じ context から同じ名前列を作ります。

---

# 8. Transaction と Budget

AI 探索では失敗が通常なので、tactic execution は常に transaction にします。

```rust
fn run_machine_tactic_transactional(
    state: &MachineProofState,
    goal: GoalId,
    tactic: MachineTactic,
    budget: TacticBudget,
) -> MachineTacticResult;

struct TacticBudget {
    max_tactic_steps: u64,
    max_whnf_steps: u64,
    max_conversion_steps: u64,
    max_rewrite_steps: u64,
    max_meta_allocations: u64,
    max_expr_nodes: u64,
}
```

```text
- 成功時だけ new_state を返す
- 失敗時は元 state を変更しない
- deterministic budget は step / reduction / rewrite / allocation fuel で表す
- fuel 超過は structured error にする
- wall-clock timeout は外側の scheduler が扱い、deterministic cache key / same-result 判定には入れない
- tactic trace は diagnostic には出してよいが certificate には入れない
```

`TacticBudget` は `run_machine_tactic` 1 回ごとの入力であり、`MachineProofState` の永続フィールドでは
ありません。成功後の `new_state.state_fingerprint` は、残 fuel や未使用 fuel ではなく、できあがった
proof state だけから計算します。各 field は finite fuel で、無制限を表す sentinel は持ちません。
0 は「その操作に使える fuel がない」という意味で、操作が必要になった時点で対応する
`TacticFuelExhausted { kind }` を返します。

`run_machine_tactic` は `TacticBudget` から run-local remaining fuel counters を作ります。semantic
transition、kernel wrapper call、fresh meta allocation、ProofExpr / Expr node generation はこの counters を
減らし、次の操作には返された remaining fuel を渡します。remaining fuel は
`MachineProofState`、`MachineProofDelta`、`state_fingerprint`、`proof_delta_hash`、search cache key に
入れません。失敗時も caller に remaining fuel を返さず、caller は入力 state と元の budget だけを保持します。

`run_machine_tactic` の error priority は、まず state/tactic 入力 validation、次に deterministic fuel 消費です。
`goal_id` が存在しない、goal が既に assigned、tactic head が解決できない、AST validation が失敗する、といった
fuel を使わない前提違反は `max_tactic_steps = 0` より先に対応する structured error を返します。
validation がすべて通った後、最初の semantic transition の直前に `max_tactic_steps` を消費します。

`run_machine_tactic` の validation order は次で固定します。複数の前提違反が同時にある場合は、この順序で
最初に見つかった error だけを返します。

```text
1. state derived fields を再計算し、state_fingerprint / state_id / reserved_local_names /
   options_fingerprint が stored fields と一致し、
   each metavariable の context hash / target hash / assignment hash、各 derived MachineGoal の
   context_hash / target_hash、MachineTacticOptions canonical bytes、SimpRegistry canonical bytes、
   resolved family canonical bytes を含む
   再計算 state_fingerprint が stored state_fingerprint と一致しなければ InvalidMachineProofState
2. goal_id が state.open_goals / goal_to_meta に存在しなければ UnknownGoal
3. 対応 meta が既に assignment を持つ場合は GoalAlreadyAssigned
4. MachineTactic AST の canonical validation:
     invalid enum field / invalid Level / invalid field shape -> InvalidMachineTactic
     invalid MachineTermSource -> InvalidMachineTermSource
5. state.env は start_machine_proof 済みのものとして扱い、options / family / simp_registry を再解決しない。
   ただし step 1 で env derived hashes と resolved entries の canonical bytes は再検査済みでなければならない
6. tactic-specific static validation:
     head / local / rule lookup、universe arg count、argument shape、site validation
7. 上がすべて成功した後、最初の semantic transition 直前に max_tactic_steps を 1 消費する
8. semantic execution 中の error は各 tactic section の mapping に従う
```

`InvalidMachineProofState` は通常の public constructor からは発生しません。serialization boundary、FFI、
debug tool などから壊れた state が渡された場合の防御用 diagnostic です。

fuel は deterministic な境界で消費します。

```text
max_tactic_steps:
  tactic semantic 内の fixed transition ごとに消費する。
  goal lookup と MachineTactic AST validation の後、各 transition を実行する直前に 1 消費する。
  fuel が残っていなければ TacticFuelExhausted { TacticStep } を返し、state は変えない。

  common:
    - run_machine_tactic dispatch
  exact:
    - exact proof_expr construction
  intro:
    - Pi target body goal construction
    - lambda proof skeleton construction
  apply:
    - each Pi binder consumption
    - InferFromTarget batch matcher execution
    - final application proof skeleton construction
  rw:
    - rewrite rule instantiation
    - target rewrite construction
    - rewrite transport proof skeleton construction
  simp-lite:
    - each iteration Eq.refl closure / applicability scan
    - each proof-producing rewrite transport nesting
    - fixed-point rewritten target goal construction
  induction-nat:
    - motive extraction
    - base / step goal construction
    - Nat.rec proof skeleton construction

max_whnf_steps:
  kernel WHNF/reduction step ごとに消費する

max_conversion_steps:
  kernel conversion comparison step ごとに消費する

max_rewrite_steps:
  rw/simp-lite の proof-producing rewrite step ごとに消費する。
  0 で rewrite が必要になった場合は TacticFuelExhausted { Rewrite }。
  simp-lite の semantic limit 到達は MachineTacticOptions.max_simp_rewrite_steps で判定し、
  fuel exhaustion とは区別する。

max_meta_allocations:
  new_goal_specs から fresh MetaVarId を 1 つ作るたびに消費する。
  対象は ApplyArg::Subgoal だけでなく、intro の body goal、rw の rewritten target goal、
  simp-lite fixed point の rewritten target goal、induction-nat の base / step goal を含む。
  必要な fresh meta 数が残 fuel を超える場合は allocation 前に TacticFuelExhausted { MetaAllocation }。

max_expr_nodes:
  tactic engine が新しい ProofExpr / Expr node を作るたびに消費する。
  対象には intro の lambda / body goal skeleton、apply の application / inserted Core wrapper、
  rw/simp-lite の Eq.rec transport、mk_eq_symm、Eq.refl proof、nested transport、
  rewritten target Expr、induction-nat の motive / Nat.rec proof skeleton / base target / step target を含む。
  入力 MachineTermSource の parser AST node はここでは数えない。ただし elaborated core Expr を
  ProofExpr::Core として proof state に保存する場合は、その core Expr node を数える。
  proof-producing rewrite では max_rewrite_steps を先に消費し、その後に生成 node 数を検査する。
  new_goal_specs では max_meta_allocations、max_open_goals、max_metas を allocation 前に検査し、
  その後に generated proof/target node 数を検査する。
  node fuel が足りなければ TacticFuelExhausted { ExprNode } を返す。
```

Machine Surface term elaboration と `max_expr_nodes` の優先順位は固定します。

```text
term payload tactics:
  1. Phase 3 elaborate_machine_term_check を expected type に対して実行する
  2. elaboration が TypeMismatch などで失敗した場合は、その Phase 3 由来 error を返す
  3. elaboration が成功した場合だけ、生成された core Expr node 数を max_expr_nodes に対して検査する
  4. node fuel が足りなければ TacticFuelExhausted { ExprNode }
```

これにより、同じ term payload が型エラーでもあり巨大な term でもある場合は、型エラーが先に返ります。

Phase 4 は kernel の fuel 消費を推測してはいけません。deterministic budget を厳密に使う実装では、kernel 側に
次の public wrapper と同じ入力、出力、remaining fuel contract を持つ API を用意します。

```text
kernel_infer_with_fuel(ctx, delta, expr, whnf_fuel, conversion_fuel)
  -> (ty, remaining_whnf_fuel, remaining_conversion_fuel)
kernel_check_with_fuel(ctx, delta, expr, expected, whnf_fuel, conversion_fuel)
  -> (remaining_whnf_fuel, remaining_conversion_fuel)
kernel_whnf_with_fuel(ctx, delta, expr, whnf_fuel)
  -> (expr, remaining_whnf_fuel)
kernel_is_defeq_with_fuel(ctx, delta, lhs, rhs, conversion_fuel)
  -> (bool, remaining_conversion_fuel)
```

kernel wrapper に渡す fuel と exhaustion mapping は固定します。

```text
kernel_whnf_with_fuel:
  consumes TacticBudget.max_whnf_steps
  FuelExhausted -> TacticFuelExhausted { Whnf }

kernel_is_defeq_with_fuel:
  consumes TacticBudget.max_conversion_steps
  FuelExhausted -> TacticFuelExhausted { Conversion }

kernel_infer_with_fuel / kernel_check_with_fuel:
  consumes both TacticBudget.max_whnf_steps and TacticBudget.max_conversion_steps.
  If the exhausted internal fuel kind is known, map to Whnf or Conversion respectively.
  If both Whnf and Conversion exhaustion are reported for one wrapper call, Whnf takes priority.
  If the kernel can only report a single checker FuelExhausted, map to TacticFuelExhausted { Conversion }.
```

kernel が `FuelExhausted` を返した場合は、上の規則で Phase 4 の structured error に写像します。既存の固定 fuel
kernel API だけを使う暫定実装は M2/M3 の開発には使ってよいですが、M7 の deterministic search integration
完了条件には含めません。固定 fuel kernel API を使う M2/M3 暫定実装では tactic search cache を無効にし、
same-result 判定の対象にしてはいけません。M7 で cache を有効化するには、上の fuel-aware API か、同じ
入出力 contract を持つ wrapper を使う必要があります。

探索キャッシュの key は次です。

```text
MachineTacticCacheKey =
  state_fingerprint
  + goal_id
  + canonical tactic hash
  + deterministic budget fingerprint
```

`deterministic budget fingerprint` は `TacticBudget canonical bytes` の hash です。

```text
TacticBudget canonical bytes:
  - tag "npa.phase4.tactic-budget.v1"
  - max_tactic_steps
  - max_whnf_steps
  - max_conversion_steps
  - max_rewrite_steps
  - max_meta_allocations
  - max_expr_nodes
```

---

# 9. Structured Result

成功:

```json
{
  "status": "success",
  "new_state_id": "s1",
  "state_fingerprint": "...",
  "closed_goals": ["g0"],
  "new_goals": ["g1", "g2"],
  "proof_delta_hash": "..."
}
```

失敗:

```json
{
  "status": "error",
  "error_kind": "missing_explicit_argument",
  "goal_id": "g0",
  "tactic_kind": "apply",
  "message": "apply requires explicit argument for A"
}
```

API 内部の最小構造は次で固定します。JSON 表示はこの構造から決定的に生成します。

```rust
enum MachineTacticResult {
    Success {
        state: MachineProofState,
        delta: MachineProofDelta,
    },
    Error {
        diagnostic: MachineTacticDiagnostic,
    },
}

struct MachineTacticDiagnostic {
    kind: MachineTacticDiagnosticKind,
    goal_id: Option<GoalId>,
    tactic_kind: Option<String>,
    primary_name: Option<Name>,
    expected_hash: Option<Hash>,
    actual_hash: Option<Hash>,
}

enum TacticFuelKind {
    TacticStep,
    Whnf,
    Conversion,
    Rewrite,
    MetaAllocation,
    ExprNode,
}

enum MachineTacticDiagnosticKind {
    InvalidMachineProofState,
    InvalidMachineTactic,
    InvalidMachineTermSource,
    UnknownGoal,
    GoalAlreadyAssigned,
    UnknownMeta,
    InvalidMetaContext,
    InvalidMetaDependency,
    ProofExprScopeError,
    ProofExprTypeMismatch,
    UnresolvedGoal,
    UnknownTacticHead,
    AmbiguousTacticHead,
    UnknownLocalName,
    AmbiguousLocalName,
    InvalidLocalHead,
    UnknownSimpRule,
    AmbiguousSimpRule,
    InvalidSimpRule,
    SimpNoProgress,
    SimpStepLimitExceeded,
    ExpectedFunctionType,
    ExpectedPiTarget,
    ExpectedEqTarget,
    UniverseArgumentMismatch,
    MissingExplicitArgument,
    AmbiguousApplyArgument,
    TooManyApplyArguments,
    TooFewApplyArguments,
    TypeMismatch,
    SubgoalDataArgument,
    AmbiguousRewriteRule,
    TacticPrimitiveUnavailable,
    InvalidInductionTarget,
    GoalLimitExceeded,
    MetaLimitExceeded,
    TacticFuelExhausted { kind: TacticFuelKind },
    InvalidCurrentDeclOrder,
    UncheckedCurrentDecl,
    CurrentDeclSignatureMismatch,
    AmbiguousKernelEnvDecl,
    InvalidVerifiedImport,
    InvalidTacticOption,
    UnsupportedTacticOption,
    InvalidEqFamily,
    InvalidNatFamily,
}
```

成功時は `MachineTacticResult::Success { state, delta }` を返します。失敗時は
`MachineTacticResult::Error { diagnostic }` を返し、呼び出し元は入力 state をそのまま保持します。

`MachineTacticDiagnostic` の optional fields は deterministic に埋めます。

```text
goal_id:
  run_machine_tactic の input goal_id が parse/validation 済みなら常に Some(input goal_id)。
  start_machine_proof / options validation のように goal がまだない error では None。

tactic_kind:
  run_machine_tactic が受け取った MachineTactic variant 名を kebab-case で入れる。
  start_machine_proof / check_current_decl_for_machine_tactic / extract_closed_machine_proof の error では None。

primary_name:
  fully qualified Name を持つ name 解決 error では、問題になった Name を Some にする。
  複数 name が同時に問題になる場合は、canonical input order で最初の name。
  local String は fully qualified Name ではないため primary_name に入れず、UnknownLocalName /
  AmbiguousLocalName / InvalidLocalHead では None。
  name に関係しない error でも None。

expected_hash / actual_hash:
  TypeMismatch / ProofExprTypeMismatch / UniverseArgumentMismatch / CurrentDeclSignatureMismatch など、
  expected と actual が両方 deterministic に得られる場合だけ Some/Some。
  片方でも構成できない場合は両方 None。
  rendered text、source span、diagnostic message から hash を作ってはいけない。

expected_hash / actual_hash payload:
  expected_hash / actual_hash は次の bytes の hash。
    - tag "npa.phase4.diagnostic.expected-actual.v1"
    - diagnostic kind
    - side: expected または actual
    - payload kind
    - payload bytes
  Expr または type の mismatch:
    payload kind = Expr。
    payload bytes = Phase 1 Expr canonical bytes。
  UniverseArgumentMismatch:
    expected payload kind = UniverseParamList。
    expected payload bytes = ordered universe parameter name list と expected count の canonical bytes。
    actual payload kind = LevelArgList。
    actual payload bytes = input Level argument list の Phase 1 Level canonical bytes。
  CurrentDeclSignatureMismatch:
    payload kind = CheckedDeclSignature。
    payload bytes = CheckedDeclSignature canonical bytes。
  上に列挙されていない diagnostic:
    その diagnostic kind が別途 canonical payload を定義しない限り None/None。
```

主な validation / execution error の対応は次で固定します。

```text
universe_args count mismatch:
  UniverseArgumentMismatch

proof_expr meta dependency reaches old open meta or cycle:
  InvalidMetaDependency

verified import certified_env_decls missing dependency or not topological:
  InvalidVerifiedImport

SimpRuleRef resolves to multiple import/current candidates:
  AmbiguousSimpRule

MachineTacticOptions numeric limit is 0:
  InvalidTacticOption

state fingerprint does not match state payload:
  InvalidMachineProofState

MachineTactic enum / Level / field shape is invalid:
  InvalidMachineTactic

MachineTermSource canonicalization fails:
  InvalidMachineTermSource

apply/rw TacticHead::Local resolves to a local let declaration:
  InvalidLocalHead

checked_current_decls source_index not strictly increasing:
  InvalidCurrentDeclOrder

checked_current_decls not complete prefix before theorem source_index:
  InvalidCurrentDeclOrder

checked_current_decls prior_chain_fingerprint mismatch:
  InvalidCurrentDeclOrder

checked_current_decls checked_env_fingerprint mismatch:
  UncheckedCurrentDecl

EqFamilyRef head missing or not unique:
  InvalidEqFamily

EqFamilyRef type/interface/coherence mismatch:
  InvalidEqFamily

NatFamilyRef head missing or not unique:
  InvalidNatFamily

NatFamilyRef type/interface/coherence mismatch:
  InvalidNatFamily

partial M1 implementation before M4/M5 receives eq_family = Some or nat_family = Some:
  UnsupportedTacticOption

nat_family = None when running InductionNat:
  TacticPrimitiveUnavailable

InductionNat local is a let, not the last assumption, not Nat, or motive extraction fails:
  InvalidInductionTarget

InductionNat resolved Nat family is inconsistent with state payload:
  InvalidMachineProofState

rw/simp-lite has no Eq target recognition head:
  TacticPrimitiveUnavailable, after tactic-specific head / rule lookup succeeds

rw/simp-lite target is not an Eq target:
  ExpectedEqTarget

rw/simp-lite target is Eq but required Eq primitive is unavailable:
  TacticPrimitiveUnavailable

generated ProofExpr / Expr nodes exceed max_expr_nodes:
  TacticFuelExhausted { ExprNode }

simp-lite rewrite count is 0, initial Eq.refl closure fails, and every rule/site is not applicable:
  SimpNoProgress

simp-lite reaches rewrite step limit:
  SimpStepLimitExceeded
```

error は AI repair に使える程度に構造化しますが、repair suggestion 自体は trusted payload ではありません。

`state_id`, `state_fingerprint`, `target_hash`, `proof_delta_hash` は display text を含めない canonical
bytes から計算します。
`MachineProofState.fingerprint` field はこの文書で `state_fingerprint` と呼ぶ hash そのものです。
`state_id` は display / API handle であり、`state_fingerprint` の入力には含めません。

`proof_delta_hash` は `MachineProofDelta` の canonical bytes から計算する non-trusted diagnostic hash です。
certificate や checker input には入りません。hash 対象は `MachineProofDelta` から
`proof_delta_hash` field 自体を除いた payload です。

```text
proof_delta_hash includes:
  - previous state_fingerprint
  - assigned_goal / assigned_meta
  - assigned proof_expr skeleton hash
  - new_goals order
  - new_metas order:
      meta_id / goal_id / context hash / target hash
  - next state_fingerprint

proof_delta_hash excludes:
  - rendered proof text
  - tactic text
  - diagnostic message text
  - AI trace / score
```

```text
state_fingerprint includes:
  - root theorem type hash
  - root module / theorem_name / source_index / universe_params
  - kernel_check_profile_hash
  - proof skeleton with MetaVarId references
  - open_goals order
  - each metavariable id / context / target / assignment hash
  - verified import module / export_hash / certificate_hash / exports signature hashes /
      certified env declaration hashes
  - checked_current_decls in source order:
      source_index / signature hash / core declaration hash /
      prior_chain_fingerprint / checked_env_fingerprint
  - SimpRegistry canonical hash
  - MachineTacticEnv resolved family bytes
  - reserved_local_names
  - MachineTacticOptions canonical bytes

state_fingerprint excludes:
  - rendered goal text
  - source span
  - tactic text
  - AI score / trace
  - diagnostic message text
  - deterministic budget / fuel counters
  - wall-clock timeout / scheduler metadata
```

`exports signature hashes` は `VerifiedImportRef.exports` の canonical ordered `CheckedDeclSignature` hashes です。
`start_machine_proof` は `VerifiedModule` 由来の export block から `exports` を再構成し、`export_hash` と
一致することを検証します。実装が `export_hash` を cache key として使う場合でも、state fingerprint には
resolved exports の signature hashes を入れ、同じ `export_hash` 文字列だけで tactic head 解決結果を
同一視してはいけません。

`telescope hash` は `rule_telescope` の ordered entries を `name / ty hash / tag "InferableTerm"` の順で
encode した hash です。valid registry に入る entry は target から推論できる term binder だけですが、固定 tag も
必ず encode し、将来 premise subgoal 対応を足す場合に既存 fingerprint と衝突しないようにします。

`MachineTacticOptions canonical bytes` は次をこの順序で encode します。

```text
- tag "npa.phase4.tactic-options.v1"
- canonical sorted simp_rules:
    name / decl_interface_hash / direction
- eq_family:
    none, or eq/refl/rec name + interface hash in field order
- nat_family:
    none, or nat/zero/succ/rec name + interface hash in field order
- max_simp_rewrite_steps
- max_open_goals
- max_metas
```

`state_id` は `state_fingerprint` から deterministically 導出します。

```text
StateId canonical/display form:
  - prefix "s:"
  - lowercase hex encoding of the full state_fingerprint bytes
```

`state_id` は `state_fingerprint` の入力には含めません。乱数、作成時刻、process-local counter、
wall-clock timeout、scheduler metadata に依存させてはいけません。same `state_fingerprint` は必ず same
`state_id` を持ちます。

---

# 10. Certificate Handoff

すべての goal が閉じたら、proof state から theorem body を取り出し、Phase 1 / Phase 2 へ渡します。

```text
closed MachineProofState
  ↓ extract proof term
kernel check theorem proof : theorem type
  ↓ CoreModule
build_module_cert
encode_module_cert
verify_module_cert
```

certificate に残してよいもの:

```text
- canonical core declarations
- canonical term / level / name tables
- import entries
- hashes
```

certificate に残さないもの:

```text
- tactic text
- AI prompt / completion
- tactic ranking score
- failed tactic list
- source span
- proof state display text
```

---

# 11. API Sketch

Phase 5 / Phase 7 からは次の API を呼びます。

```rust
struct MachineProofSpec {
    module: ModuleName,
    theorem_name: Name,
    source_index: u64,
    universe_params: Vec<String>,
    theorem_type: Expr,
}

pub fn start_machine_proof(
    spec: MachineProofSpec,
    imports: &[VerifiedModule],
    checked_current_decls: &[CheckedCurrentDecl],
    options: &MachineTacticOptions,
) -> Result<MachineProofState, MachineTacticDiagnostic>;

pub fn start_machine_proof_from_verified_imports(
    spec: MachineProofSpec,
    imports: &[VerifiedImportRef],
    checked_current_decls: &[CheckedCurrentDecl],
    options: &MachineTacticOptions,
) -> Result<MachineProofState, MachineTacticDiagnostic>;

pub fn check_current_decl_for_machine_tactic(
    imports: &[VerifiedModule],
    checked_prior_current_decls: &[CheckedCurrentDecl],
    source_index: u64,
    decl: Decl,
) -> Result<CheckedCurrentDecl, MachineTacticDiagnostic>;

pub fn validate_machine_tactic_candidate(
    candidate: MachineTacticCandidate,
) -> Result<MachineTactic, MachineTacticDiagnostic>;

pub fn parse_machine_tactic_candidate_text(
    input: &str,
) -> Result<MachineTacticCandidate, MachineTacticDiagnostic>;

pub fn run_machine_tactic(
    state: &MachineProofState,
    goal: GoalId,
    tactic: MachineTactic,
    budget: TacticBudget,
) -> MachineTacticResult;

pub fn extract_closed_machine_proof(
    state: &MachineProofState,
) -> Result<Expr, MachineTacticDiagnostic>;

pub fn extract_closed_machine_theorem_decl(
    state: &MachineProofState,
) -> Result<Decl, MachineTacticDiagnostic>;
```

`validate_machine_tactic_candidate` が主入口です。`parse_machine_tactic_candidate_text` は CLI /
debug / compatibility 用の補助入口であり、parse 後に必ず同じ validator を通します。

`exact`, `apply`, `rw`, `simp-lite` が term や theorem interface を扱う場合は、Phase 3 AI の
term-level API と verified import metadata を使います。

imports、checked current declarations、simp rules、options は `start_machine_proof` で state に固定します。
Phase 2 verifier 出力として `VerifiedImportRef` をすでに保持している caller は
`start_machine_proof_from_verified_imports` を使います。
この entrypoint の意味論は、`start_machine_proof` が `VerifiedModule` から import 環境を canonical order に正規化した後の状態と
byte-for-byte に一致しなければなりません。
`start_machine_proof_from_verified_imports` は filesystem、network、package registry、または current IDE session から
import を補完してはいけません。
`imports` は `(module, export_hash, certificate_hash)` の canonical order でなければならず、違反した場合は
`MachineTacticDiagnostic` として deterministic に拒否します。
`run_machine_tactic` は state 内の environment だけを使い、別の imports/options を受け取りません。
呼び出し側が environment を変えたい場合は、新しい state を作り直します。

`CheckedCurrentDecl` は `check_current_decl_for_machine_tactic` でだけ生成します。呼び出し側は current module の
declaration を parser/resolver が付けた `source_index` とともに source order で 1 つずつ渡し、constructor は
imports と既に checked 済みの prior declarations だけを kernel environment に入れて検査します。
`start_machine_proof` に渡す `checked_current_decls` も同じ source order でなければなりません。future declaration
を環境に入れてはいけません。

`extract_closed_machine_proof` は proof term だけを返します。certificate handoff で theorem declaration
まで作る場合は `extract_closed_machine_theorem_decl` を使い、`MachineProofSpec` の
`module / theorem_name / universe_params / theorem_type` と抽出した proof term から kernel `Decl` を作ります。

---

# 12. Milestones

## M0: Baseline split

```text
- doc/phase4-human.md と doc/phase4-ai.md を分ける
- trusted payload に tactic trace を入れない方針を明記
```

## M1: MachineProofState core

```text
- MachineProofState / MachineGoal / MetaVarStore
- ProofRoot / ProofExpr skeleton
- ProofExpr と core Expr の binder 境界
- Phase 4 proof skeleton checker
- MachineLocalDecl context hash / prefix comparison
- MachineLocalDecl / MachineLocalContext canonical bytes を固定する
- reserved_local_names の deterministic 再計算
- assign_goal / MachineProofDelta
- max_meta_allocations はすべての new_goal_specs 由来 fresh meta に適用
- max_open_goals は置換後 open_goals 数、max_metas は MetaVarStore 総 meta 数に適用
- MachineProofSpec / extract_closed_machine_theorem_decl
- start_machine_proof の initial m0/g0 state は deterministic
- deterministic goal order
- GoalId は MetaVarId から deterministic に導出する
- MetaVarId / GoalId canonical bytes を domain tag + minimal ULEB128 で固定する
- Phase 4 canonical primitive encoding を固定する
- transactional state clone
- run_machine_tactic validation order を固定する
- run_machine_tactic は env derived fields も state validation で再検査する
- structured tactic error
- MachineTacticDiagnostic optional fields are deterministic
- expected_hash / actual_hash の canonical payload を固定する
- MachineTacticOptions の numeric limit 0 は InvalidTacticOption
- options_fingerprint は MachineTacticOptions canonical bytes からの derived field として検証する
- resolved Eq/Nat family を MachineTacticEnv に保存し、run 時に再解決しない
- Eq family がない environment は MachineTacticEnv.eq_family = None で表す
- Builtin family interface_tag の stable values を固定する
- Builtin family stable Name values を固定する
- resolved family canonical bytes を state_fingerprint に含める
- canonical state_fingerprint
- state_fingerprint は kernel_check_profile_hash を直接含める
- state validation は stored fields と derived view hash を混同しない
- ProofExpr canonical bytes / assignment hash / assigned_proof_expr_hash を固定する
- CheckedDeclSignature canonical bytes を固定する
- MachineProofDelta payload と proof_delta_hash input を一致させる
- state_id は full state_fingerprint hex から導出する
- start_machine_proof
- current declaration signature は Phase 2 interface hash rule で再計算
- checked current declarations は source_index / prior_chain_fingerprint で順序を検証
- checked current declarations は theorem source_index 直前までの完全 prefix
- source_index は source-level top-level declaration だけに付け、生成 declaration は親 closure に含める
- kernel environment は canonical dedup し、同名異hashを reject
- imports/options/current checked declarations are fixed in state
- verified imports keep certified dependency closure for conversion
- checked current declarations keep kernel-checked core declarations for conversion
- M1 では eq_family / nat_family が Some の場合だけ UnsupportedTacticOption で reject してよい
- meta dependency graph は DAG に限定し、old open meta / cycle を reject
- import kernel environment は sorted imports + dependency-topological certified_env_decls + checked current source order
- state_fingerprint は verified import exports signature hashes を含む
- SimpRegistry canonical bytes / hash を固定する
- remaining fuel counters は run-local で state / delta / cache key に入れない
- Phase 3 AI M7 dependency を API 境界に明示
```

## M2: exact / intro

```text
- Phase 3 AI M7 が完了していること
- exact は Machine Surface term check で goal を閉じる
- MachineTermSource / canonicalize_machine_term_source を固定する
- MachineTacticCandidate の raw term を MachineTermSource に checked constructor 経由で変換する
- run_machine_tactic は MachineTermSource.source と Phase 4 wrapper canonical_hash の一致を再検査する
- term elaboration error は max_expr_nodes より先に返す
- intro は Pi target だけ lambda を作る
- local shadowing rule を Phase 3 と揃える
```

## M3: apply

```text
- fully qualified theorem / local hypothesis application
- local let declaration は apply の proof head にせず InvalidLocalHead
- imported head は name + decl_interface_hash 必須
- explicit universe args
- ApplyArg は binder order で positional に消費
- ApplyArg policy に従って Term / Subgoal / InferFromTarget を処理
- InferFromTarget は conservative first-order matcher に限定
- 複数 InferFromTarget は batch matcher で同時解決する
- missing / ambiguous explicit arg を structured error
- MachineTactic canonical bytes を cache key 用に固定する
- generated subgoal の deterministic ordering
- Subgoal.name_hint は display-only で tactic hash / state fingerprint に入れない
```

## M4: rw / simp-lite

```text
- RewriteRuleRef の universe_args / args を検査
- local let declaration は rw の rule head にせず InvalidLocalHead
- RewriteDirection は Forward / Backward の固定 enum とする
- RewriteRuleRef instantiation の diagnostic mapping を固定する
- Eq theorem rewrite は instantiated rule と EqTargetLeft / EqTargetRight だけ許す
- Eq eliminator primitive availability を検査
- Eq target recognition head と resolved refl/rec availability を分ける
- EqFamilyRef の coherent family 検査
- EqFamilyRef の FamilyOrigin 一致規則を固定する
- premise goals first, rewritten target goal last
- SimpRuleRef は name + decl_interface_hash + direction
- resolved SimpRegistry を実行時 registry とする
- SimpRuleRef が import/current の複数候補へ解決される場合は AmbiguousSimpRule
- SimpLite tactic-local allowlist の duplicate は canonical dedup
- SimpLite canonical tactic hash は canonicalized allowlist を使う
- simp rule instantiation は target からの deterministic inference のみ
- simp-lite per-target instantiation algorithm を固定する
- simp rule universe_params は kernel Decl の ordered universe parameter 名列をそのまま使う
- simp rule telescope は InferableTerm だけを残し、proof premise / uninferable term binder は InvalidSimpRule
- InvalidSimpRule と per-target not applicable の境界を固定
- deterministic simp registry
- RewriteSite は EqTargetLeft / EqTargetRight のみ
- rw / simp-lite の rewritten target は WHNF 後の Eq target から作る
- rw / simp-lite は tactic-specific head / rule lookup 後に Eq target recognition head 解決、target Eq 判定、resolved refl/rec availability 検査の順で行う
- rw / simp-lite は head / rule lookup error を Eq family unavailable より先に返す
- rw は target 側 Eq type argument を保持し、rule 側 Eq type argument に置き換えない
- rw transport の Eq.rec は target 側 Eq family に揃える
- rw transport / mk_eq_symm は literal builtin ではなく resolved Eq family の eq/refl/rec を使う
- simp-lite は rewrite 成功ごとに canonical rule/site order の先頭から再走査
- simp-lite は WHNF current target hash と WHNF candidate new target hash が変わる rewrite だけを progress として数える
- simp-lite の 0 rewrite Eq.refl closure は Eq.rec を要求しない
- simp-lite fixed point は rewritten target goal 1 つで成功できる
- max_simp_rewrite_steps 到達時は追加 scan せず SimpStepLimitExceeded
- max_simp_rewrite_steps は semantic limit、TacticBudget.max_rewrite_steps は fuel
- source-level simp rule 登録 syntax は入れない
- step limit
```

## M5: induction-nat

```text
- local Nat variable only
- induction 対象 local は MVP では context の最後に限定
- induction 対象 local_name は完全一致 1 件だけ許し、重複は AmbiguousLocalName
- induction 対象は local assumption に限定し、local let declaration は拒否
- Nat.zero / Nat.succ / Nat.rec を verified head として解決
- NatFamilyRef の coherent family 検査
- NatFamilyRef の FamilyOrigin 一致規則を固定する
- Nat.rec proof skeleton
- step meta は lambda wrapper 経由で Nat.rec に渡す
- step goal に外側 induction target local が残ることを明記する
- base / step goal generation
- induction-nat validation order と InvalidInductionTarget mapping を固定する
- later local declaration / motive extraction failure を reject
- generated local names are deterministic
```

## M6: Certificate handoff

```text
- closed MachineProofState から theorem body extraction
- kernel check
- CoreModule -> certificate -> verify
- tactic metadata が certificate に入らないことをテスト
```

## M7: AI search integration gate

```text
- Phase 7 が candidate を複数投げられる
- kernel fuel-aware public API を使う
- fuel-based failure が deterministic
- kernel FuelExhausted は Whnf / Conversion へ deterministic に写像する
- kernel infer/check で Whnf と Conversion が同時 exhaustion の場合は Whnf を優先する
- fixed fuel kernel API の暫定実装は search cache / same-result 判定の対象外
- TacticBudget canonical bytes を cache key 用に固定する
- max_tactic_steps の fixed transition 消費点が deterministic
- max_expr_nodes の対象 node と他 limit との優先順位が deterministic
- wall-clock timeout は外側 scheduler の non-canonical result
- same state + same tactic + same deterministic budget から same result
- failed tactic log は non-trusted artifact
```

---

# 13. MVP Completion

AI 向け Phase 4 MVP が完了したと言える条件:

```text
- exact / intro / apply が structured Machine Tactic として動く
- tactic execution が transactional
- assign_goal がすべての state update を担う
- term payload は Phase 3 Machine Surface term-level API で check される
- imported tactic head は name + decl_interface_hash で解決される
- current module head は checked signature + decl_interface_hash で解決される
- current module の reducible def は checked core declaration から conversion で使える
- apply の argument policy が deterministic
- Subgoal は Prop/proposition binder に限定される
- unresolved metavariable は core proof extraction で reject される
- state_fingerprint が display text / trace に依存しない
- state_fingerprint が imports / checked current core decls / simp rules を含む
- tactic cache key が deterministic budget を含む
- failed tactic が structured error を返す
- unresolved goal が残る proof は certificate 化されない
- closed proof term を kernel が検査する
- certificate に tactic / AI metadata が入らない
- same state + same tactic + same deterministic budget から same proof term hash または same structured error
```

`rw`, `simp-lite`, `induction-nat` は Phase 4 full target として実装しますが、最初の AI MVP は
`exact`, `intro`, `apply` だけでも Phase 7 の探索基盤になります。

---

# 14. 一文でまとめると

AI 向け Phase 4 は、**AI が大量に出す tactic 候補を、曖昧な script ではなく構造化された
transactional command として試し、成功したものだけを kernel が検査できる proof term に変換する層**
です。
