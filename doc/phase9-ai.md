# Phase 9 AI Profile: Advanced Automation

この文書は、NPA の **AI 向け Phase 9** の仕様です。

Phase 9 Human Profile は、advanced inductive、universe polymorphism 強化、
typeclass、quotient、SMT certificates、theorem graph、natural language formalization を
人間が使える高水準機能として整理します。

Phase 9 AI Profile は、それらを AI 探索器・形式化器・修復器から使うための
**非信頼 Machine Profile** です。AI は候補と補助情報を出しますが、正しさの根拠にはしません。
最終的に採用されるのは、kernel と independent checker が検査できる canonical certificate だけです。

```text
信頼しない:
  AI model
  prompt / completion
  theorem graph score
  typeclass search heuristic
  SMT solver process
  natural language formalizer
  repair suggestion

信頼する:
  small Rust kernel
  canonical core AST
  canonical certificate
  independent checker
```

Phase 9 AI Profile の目的は次です。

```text
- AI が高度機能の候補を構造化形式で出せるようにする
- すべての候補を deterministic validation / replay に通す
- AI trace や score を certificate hash に混ぜない
- theorem graph と自然言語情報を、探索効率のための sidecar に限定する
- SMT や quotient など trusted base を広げやすい機能の検査境界を固定する
```

---

# 1. 全体アーキテクチャ

AI 向け Phase 9 は、Phase 3 AI、Phase 4 AI、Phase 5 AI、Phase 7、Phase 8 AI の上に乗る
上位 profile です。

```text
AI Orchestrator
  ↓ untrusted proposals
Phase 9 AI Machine Profile
  ↓ validation / replay
Phase 3 AI Machine Surface
Phase 4 AI Machine Tactics
Phase 5 AI Machine API
Phase 7 Search
  ↓ checked proof term / declaration / certificate
Rust kernel
  ↓ canonical certificate
Phase 8 independent checker
```

Phase 9 AI は kernel に AI 呼び出しを追加しません。
AI 呼び出し、RAG、embedding、graph ranking、SMT solver execution はすべて trusted base の外側に置きます。

---

# 2. 共通 Candidate Envelope

Phase 9 AI の各機能は、自由文ではなく共通 envelope に包んだ構造化候補として扱います。

```rust
struct Phase9AiCandidateEnvelope<T> {
    profile_version: Phase9AiProfileVersion,
    task_kind: Phase9AiTaskKind,
    target: Phase9AiTarget,
    imports: Vec<VerifiedImportRef>,
    options: Phase9AiOptionsRef,
    payload: T,
}

enum Phase9AiTaskKind {
    AdvancedInductive,
    UniverseRepair,
    TypeclassResolution,
    QuotientConstruction,
    SmtCertificate,
    TheoremGraphQuery,
    NaturalLanguageFormalization,
}

struct Phase9AiTarget {
    env_fingerprint: Hash256,
    target_decl_hash: Option<Hash256>,
    goal_fingerprint: Option<Hash256>,
}

struct Phase9AiGoal {
    universe_params: Vec<UniverseParam>,
    local_context: Vec<MachineLocalDecl>,
    target: CoreExpr,
}

type Telescope = Vec<MachineTelescopeBinder>;

struct MachineTelescopeBinder {
    ty: CoreExpr,
}

struct Phase9AiGlobalRef {
    module: ModuleName,
    export_hash: Hash256,
    certificate_hash: Hash256,
    name: GlobalName,
    decl_interface_hash: Hash256,
}

enum Phase9AiOptionsRef {
    Inline {
        options_hash: Hash256,
        canonical_bytes: Vec<u8>,
    },
    Artifact {
        path: ArtifactPath,
        file_hash: Hash256,
        options_hash: Hash256,
        size_bytes: u64,
    },
}

struct Phase9AiOptions {
    schema_version: Phase9AiOptionsVersion,
    advanced_inductive: Phase9AdvancedInductiveOptions,
    typeclass: Phase9TypeclassOptions,
    quotient: Option<Phase9QuotientOptions>,
    smt: Option<Phase9SmtOptions>,
    formalization: Option<Phase9FormalizationOptions>,
}

struct Phase9AdvancedInductiveOptions {
    approved_nested_type_constructors: Vec<Phase9AiGlobalRef>,
}

struct Phase9TypeclassOptions {
    class_declarations: Vec<Phase9AiGlobalRef>,
}

struct Phase9QuotientOptions {
    setoid: Phase9AiGlobalRef,
    setoid_mk: Phase9AiGlobalRef,
    setoid_relation: Phase9AiGlobalRef,
    rel_equiv: Phase9AiGlobalRef,
    quotient: Phase9AiGlobalRef,
    quotient_mk: Phase9AiGlobalRef,
    quotient_sound: Phase9AiGlobalRef,
    quotient_lift: Phase9AiGlobalRef,
    eq: Phase9AiGlobalRef,
}

struct Phase9SmtOptions {
    eq: Phase9AiGlobalRef,
    prop_false: Option<Phase9AiGlobalRef>,
    prop_not: Option<Phase9AiGlobalRef>,
}

struct Phase9FormalizationOptions {
    tactic_options: MachineTacticOptions,
}
```

`candidate_hash` は envelope の canonical bytes から計算します。

```text
candidate_hash =
  sha256("npa.phase9_ai.candidate.v1" || canonical_bytes(envelope))
```

canonical bytes には次を入れません。

```text
- prompt
- completion
- model name
- model score
- sampling parameter
- latency
- diagnostic source span
- natural language explanation
- pretty printed theorem statement
```

これらは必要なら sidecar に保存できますが、certificate identity には影響させません。
ただし、formalization の `claim_span` のように候補の意味を固定する span は diagnostic source span ではありません。
semantic span は payload の canonical bytes と `candidate_hash` に含めます。

`options` は replay input の一部です。
validator は `Inline.canonical_bytes` または `Artifact` の `path` / `file_hash` で固定された bytes から
`options_hash` を再計算します。hash だけを渡して、validator が外部 store から options を補完してはいけません。
`options` bytes は `Phase9AiOptions` として canonical decode できなければなりません。
`options_hash` は次で固定します。

```text
options_hash =
  sha256("npa.phase9_ai.options.v1" || canonical_bytes(Phase9AiOptions))
```

`imports` は Phase 4 AI と同じく `(module, export_hash, certificate_hash)` の canonical order に sort 済みでなければなりません。
同じ `(module, export_hash, certificate_hash)` を重複して含む request は `ImportClosureMismatch` として拒否します。

`target.env_fingerprint` は validator が次から再計算します。

```text
env_fingerprint =
  sha256(
    "npa.phase9_ai.env.v1"
    || profile_version canonical bytes
    || task_kind canonical bytes
    || imports canonical bytes
    || options_hash digest bytes
  )
```

`target.env_fingerprint` が再計算値と一致しない request は `TargetFingerprintMismatch` として拒否します。
goal や target declaration の内容は `env_fingerprint` には入れず、`goal_fingerprint` / `target_decl_hash` で別に束縛します。

`target_decl_hash` は Phase 2 の `decl_certificate_hash` そのものです。
`decl_interface_hash` や `type_hash` で代用してはいけません。
target declaration を束縛する endpoint は、payload から決定的に再構成できる declaration、または request の canonical wrapper に
明示された declaration だけを使って再計算します。hidden session state や IDE 上の現在カーソル位置から補完してはいけません。

`goal_fingerprint` は checked goal の universe parameter context、local context、target から次で再計算します。
`universe_params` はその goal 内の `LevelExpr` が参照できる universe parameter context です。
`universe_params` は binding order のまま保存し、同じ `UniverseParam` を重複して含んではいけません。
`local_context` は Phase 4 AI の `MachineLocalContext canonical bytes` と同じ規則で encoding し、
closed goal の場合は空配列にします。
Phase 4 の `MachineLocalDecl canonical bytes` は UTF-8 local name bytes を含むため、
Phase 9 の `goal_fingerprint` も local name に敏感です。
ここでの local name は tactic / API が local binder を安定参照するための machine identifier であり、
core binder name ではありません。
同じ `ty` / `value` / `target` でも local name が違えば別 goal として扱います。
name-insensitive な identity が必要な caller は、`Phase9AiGoal` を作る前に local name を決定的に正規化しなければなりません。
ここでの `CoreExpr` は Phase 1 / Phase 2 の core `Expr` と同じ AST を指す alias です。
したがって binder name は `CoreExpr` の canonical bytes に含めず、bound variable は `BVar(index)` で表します。
`MachineLocalDecl.ty` / `MachineLocalDecl.value` も同じ core expression canonical bytes として扱います。
`local_context` と `target` に現れる `LevelExpr` は、imported declaration の explicit level argument か、
`Phase9AiGoal.universe_params` に含まれる parameter だけを参照できます。
`local_context` は Phase 4 と同じ context order で保存し、de Bruijn index `0` は `local_context` の最後の binder を指します。
`local_context[i].ty` は `local_context[..i]` の下で well-typed、`value` がある場合も同じ prefix context の下で
`ty` の term として well-typed でなければなりません。`target` は full `local_context` の下で well-typed でなければなりません。

`Telescope` は Phase 9 AI 内の inductive `params` / `indices` と quotient `params` に共通して使う
machine-level binder list です。
`Telescope` は `MachineTelescopeBinder` の context order 配列で、各 binder は type だけを持ちます。
`MachineLocalDecl.value = Some(...)` に相当する let binder は `Telescope` では許可しません。
local let を扱う必要がある task は `Telescope` ではなく `MachineLocalDecl` を使う別 payload を定義します。
`Telescope` canonical bytes は次で固定します。

```text
MachineTelescopeBinder canonical bytes:
  - tag "npa.phase9_ai.telescope-binder.v1"
  - ty CoreExpr canonical bytes

Telescope canonical bytes:
  - tag "npa.phase9_ai.telescope.v1"
  - MachineTelescopeBinder list in context order
```

`Telescope[i].ty` は `Telescope[..i]` の下で well-typed でなければなりません。
`Telescope` の body に現れる de Bruijn index `0` は、その body から見て最後の telescope binder を指します。
binder name は diagnostic metadata として sidecar に保存してよいですが、`Telescope` canonical bytes、
`candidate_hash`、`target_decl_hash` には入りません。

`Phase9AiGoal` 内の imported global reference は、Phase 2 の `GlobalRef::Imported(import_index, name, decl_interface_hash)`
として表し、`import_index` は canonical sort 後の envelope `imports` の 0-based index です。
`name` / `decl_interface_hash` はその import の export table に一意に存在しなければなりません。
Phase 9 AI goal payload では `GlobalRef::Local` / `GlobalRef::LocalGenerated` を使いません。
現在生成中の declaration や current module declaration は、goal 内 global ref ではなく local binder または
task-specific payload で明示します。

`Phase9AiGoal` の検査結果を standalone certificate に出す場合、validator は local context を決定的に閉じます。
standalone declaration の `universe_params` は `goal.universe_params` をそのまま使い、closure は local binders だけに適用します。
`local_context = [d0, d1, ..., d(n-1)]` のとき、closure は次の再帰定義で固定します。

```text
close_goal_type(goal) = close_type(0, goal.target)

close_type(i, body):
  if i == n:
    body
  else if di.value = None:
    Pi(di.ty, close_type(i + 1, body))
  else if di.value = Some(v):
    Let(di.ty, v, close_type(i + 1, body))

close_goal_proof(goal, proof) = close_proof(0, proof)

close_proof(i, proof_body):
  if i == n:
    proof_body
  else if di.value = None:
    Lam(di.ty, close_proof(i + 1, proof_body))
  else if di.value = Some(v):
    Let(di.ty, v, close_proof(i + 1, proof_body))
```

実装としては `target` / `proof` から `local_context` を末尾から先頭へ fold して同じ AST を作ってもよいですが、
結果は上の再帰定義と byte-for-byte に一致しなければなりません。
`MachineLocalDecl` が diagnostic name を持つ場合でも、上の `Pi` / `Lam` / `Let` node には name を入れません。
tactic 内の一時 goal として使う場合は closure を行わず、Phase 4/5 の proof state が context を保持します。

```text
goal_fingerprint =
  sha256(
    "npa.phase9_ai.goal.v1"
    || env_fingerprint digest bytes
    || canonical_bytes(Phase9AiGoal.universe_params)
    || canonical_bytes(Phase9AiGoal.local_context)
    || canonical_bytes(Phase9AiGoal.target)
  )
```

task ごとの `Phase9AiGoal` source は固定します。

```text
UniverseRepair:
  goal_fingerprint mode では payload.goal
  future target_decl_hash mode では goal_fingerprint = None

TypeclassResolution:
  payload.goal

SmtCertificate:
  payload.goal

TheoremGraphQuery:
  payload.goal
```

これらから再計算した値が `target.goal_fingerprint` と一致しない request は `TargetFingerprintMismatch` として拒否します。

`Phase9AiGlobalRef` の canonical bytes field order と、これを list 内で sort する場合の tuple key は次で固定します。

```text
module
export_hash digest bytes
certificate_hash digest bytes
name
decl_interface_hash digest bytes
```

すべての `Phase9AiGlobalRef` は envelope の `imports` 内の export table から
`module / export_hash / certificate_hash / name / decl_interface_hash` で一意に解決できなければなりません。
解決後に core term へ埋め込むときは、canonical sort 後の envelope `imports` 内の 0-based index を使って
`GlobalRef::Imported(import_index, name, decl_interface_hash)` を作ります。

```text
resolve_imported_ref(ref) =
  GlobalRef::Imported(import_index(ref), ref.name, ref.decl_interface_hash)
```

`resolve_imported_ref` は Phase 9 AI 全体の共通 helper です。
`import_index(ref)` は、`ref.module / ref.export_hash / ref.certificate_hash` と一致する envelope import の canonical sort 後 index です。
validator は import closure 外の global environment、標準ライブラリの hidden registry、現在編集中 declaration から
`Phase9AiGlobalRef` を補完してはいけません。
解決不能または複数解決になる ref は `ImportClosureMismatch` または feature-specific な `TargetRefMismatch` として拒否します。
現在生成中の declaration は `Phase9AiGlobalRef` では参照せず、`target_decl_hash` または payload 内の明示的な local binder で束縛します。
`approved_nested_type_constructors` と `class_declarations` は上の `Phase9AiGlobalRef` tuple key で strictly sorted され、
重複を含んではいけません。`Phase9QuotientOptions` の各 primitive ref も同じ解決規則に従います。
`QuotientConstruction` task では `options.quotient = Some(...)` でなければならず、`SmtCertificate` task では
`options.smt = Some(...)`、`NaturalLanguageFormalization` task では `options.formalization = Some(...)` でなければなりません。
その他 task はこれらの `Option` を無視せず
canonical bytes と `options_hash` には含めますが、feature-specific validation には使いません。

task ごとの `target` 必須条件は固定します。

```text
AdvancedInductive:
  target_decl_hash = None
  goal_fingerprint = None
  生成される declaration hash の束縛は payload 内の MachineInductiveProposal.expected_decl_hash で行う

UniverseRepair:
  MVP /machine/phase9/universe/repair/check:
    target_decl_hash = None
    goal_fingerprint = Some
  future declaration repair extension:
    target_decl_hash = Some
    goal_fingerprint = None

TypeclassResolution:
  target_decl_hash = None
  goal_fingerprint = Some

QuotientConstruction:
  target_decl_hash = Some
  goal_fingerprint = None

SmtCertificate:
  target_decl_hash = None
  goal_fingerprint = Some

TheoremGraphQuery:
  target_decl_hash = None
  goal_fingerprint = Some

NaturalLanguageFormalization:
  target_decl_hash = None
  goal_fingerprint = None
```

---

# 3. Advanced Inductive AI

payload schema は indexed / mutual / nested inductive の宣言候補を表現できますが、
MVP validator が受理する範囲は下で定義する non-mutual / non-nested profile だけです。
AI は recursor や computation rule を任意に供給してはいけません。

```rust
struct MachineInductiveProposal {
    block_name: NameId,
    expected_decl_hash: Option<Hash256>,
    universe_params: Vec<UniverseParam>,
    inductives: Vec<MachineInductiveFamilyProposal>,
}

struct MachineInductiveFamilyProposal {
    name: NameId,
    params: Telescope,
    indices: Telescope,
    result_sort: SortExpr,
    constructors: Vec<MachineConstructorProposal>,
}

struct MachineConstructorProposal {
    name: NameId,
    ty: CoreExpr,
}
```

`block_name` は family declaration name ではなく、public name を作るための namespace prefix です。
各 family の宣言名は `family_public_name(i)` だけであり、MVP の単一 family でも
`inductives[0].name` を別の top-level declaration name として扱ってはいけません。
prefix なしで `Vec` を宣言したい場合は canonical empty prefix を `block_name` に入れ、
`inductives[0].name = Vec` とします。
`block_name = Vec` かつ `inductives[0].name = Vec` は意図的に `Vec.Vec` を宣言する request であり、
validator が pretty name の重複回避として自動的に短縮してはいけません。

`MachineInductiveProposal` 内で宣言中の inductive family を参照する場合は、
constructor type の `CoreExpr` 内でだけ Phase 2 `GlobalRef::Local(i)` を使います。
ここでの `i` は envelope の module declaration index ではなく、
`MachineInductiveProposal.inductives` 配列の 0-based family index です。
この task-local 解釈は Advanced Inductive payload の validation 中だけ有効です。
`params` / `indices` / `result_sort` に block-local `GlobalRef::Local` が現れた場合は拒否します。
constructor type 内の imported constant は通常どおり `GlobalRef::Imported(import_index, name, decl_interface_hash)` で表し、
`import_index` は envelope `imports` の canonical sort 後 index です。
`GlobalRef::LocalGenerated` は proposal payload 内では常に拒否します。
constructor / recursor / iota rule は validator が inductive declaration から生成するため、
AI は生成済み artifact を constructor type の前提として参照できません。

validator が Phase 2 certificate declaration package を作る場合、
この block-local `GlobalRef::Local(i)` は受理済み inductive family の certificate-local declaration index へ
family 配列順で決定的に rewrite します。
rewrite 前後で constructor type、positivity check、generated artifact check が同じ family 対応を使わなければなりません。
hidden current module state や既存 declaration index から `Local(i)` の意味を補完してはいけません。
MVP の `/machine/phase9/inductive/check` が返す declaration hash は、生成される certificate package が
この inductive declaration 1件だけを含むものとして計算します。
したがって `inductives[0]` 内の block-local `GlobalRef::Local(0)` は、hash 計算前に
Phase 2 `GlobalRef::Local(0)` へ rewrite します。
将来、既存 module の途中へ挿入する profile を追加する場合は、その module declaration order を request payload に含め、
rewrite 後の index と `decl_certificate_hash` をその profile で再定義しなければなりません。

Phase 9 AI MVP の Advanced Inductive validator は、Phase 2 の既存 `InductiveDecl` schema に合わせ、
`inductives.len() == 1` の non-mutual declaration だけを受け付けます。
`inductives.len() != 1` の mutual block は、Phase 2 に `MutualInductiveBlock` certificate schema または
複数 `InductiveDecl` への lowering rule が追加されるまで `UnsupportedFeature` として拒否します。
MVP では nested recursive occurrence も許可しません。
したがって `options.advanced_inductive.approved_nested_type_constructors` は空でなければならず、
空でない場合、または constructor type 内に approved constructor 越しの recursive occurrence が現れた場合は
`UnsupportedFeature` として拒否します。
large elimination、mutual recursor、nested recursor も MVP では `UnsupportedFeature` です。

MVP で Phase 2 certificate declaration を作る場合、唯一の family `inductives[0]` から
まず recursor を持たない generation base declaration を決定的に再構成します。
constructor type は hash 計算前に block-local `GlobalRef::Local` を上記の certificate-local `GlobalRef::Local` へ
rewrite したものを使います。
この generation base declaration は generator の内部入力であり、Phase 2 certificate package に出力される declaration ではありません。
validator はこれに対して `decl_interface_hash`、`decl_certificate_hash`、import hash、dependency graph entry を計算してはいけません。
外部に見える declaration identity は、生成済み recursor を持つ `final_inductive_decl` からだけ計算します。
MVP の `rewrite_block_local_refs` は、constructor type 内の block-local `GlobalRef::Local(0)` だけを
certificate-local self reference `GlobalRef::Local(0)` に置き換えます。
`GlobalRef::Local(i)` where `i != 0` が残る場合は、`inductives.len() == 1` の MVP では必ず `TargetRefMismatch` として拒否します。

```text
base_inductive_decl_for_generation:
  name = family_public_name(0)
  universe_params = payload.universe_params
  params = inductives[0].params
  indices = inductives[0].indices
  sort = inductives[0].result_sort
  constructors =
    for each constructor j in payload order:
      ConstructorSpec {
        name = constructor_public_name(0, j)
        type = rewrite_block_local_refs(inductives[0].constructors[j].ty)
      }
  recursor = None

final_inductive_decl:
  same fields as base_inductive_decl_for_generation
  recursor = generate_inductive_artifacts_v1(base_inductive_decl_for_generation, ...).recursor
```

`expected_decl_hash = Some(h)` の場合、validator は `final_inductive_decl` から Phase 2 の通常規則で
`decl_certificate_hash` を再計算し、`h` と一致しなければ `TargetFingerprintMismatch` として拒否します。
`expected_decl_hash = None` の場合、validator は check success response に再計算した
`decl_interface_hash` と `decl_certificate_hash` を返します。
AdvancedInductive では envelope `target.target_decl_hash` は常に `None` です。
これは既存 declaration を束縛する field と、新規生成される declaration hash の payload-local expectation を混同しないためです。

検査順序は固定します。

```text
1. name uniqueness
2. universe parameter well-formedness
3. parameter / index telescope type check
4. constructor type check
5. constructor result family check
6. strict positivity check
7. approved nested occurrence check
8. generated recursor determinism check
9. generated iota rule determinism check
```

constructor type check の context は次で固定します。
family `i` の constructor `ty` は、universe context `payload.universe_params` と
term context `inductives[i].params` の下で検査します。
family `indices` は constructor `ty` の outer context に暗黙追加しません。
index result に使う local variable は、constructor `ty` 自身の outer `Pi` binder として明示されていなければなりません。
constructor `ty` の outer `Pi` binder を剥がすたびに、その binder を
`inductives[i].params` の後ろへ追加した constructor-local context で以後の binder type と final result を検査します。
したがって constructor result context は常に
`inductives[i].params ++ constructor_outer_pi_binders` です。

`approved nested occurrence check` が使う approved set は、`Phase9AiOptions.advanced_inductive.approved_nested_type_constructors`
だけから取ります。実装が hidden builtin list や runtime registry を追加で参照してはいけません。
MVP で nested inductive を許可しない場合は、この list を空にし、nested occurrence をすべて拒否します。

constructor result family check は次で固定します。
constructor type の outer `Pi` binders を左から右へ剥がした後、最終 result は
`Const(GlobalRef::Local(family_index), level_args)` を head とする application でなければなりません。
`family_index` はその constructor が属する `MachineInductiveFamilyProposal` の index と一致しなければなりません。
`level_args.len()` は proposal の `universe_params.len()` と一致し、各 level はその universe parameter context だけを参照できます。
result application の parameter arguments は、family `params` を constructor result context へ weakening したものと
byte-for-byte に一致しなければなりません。
これは de Bruijn 表現上、constructor-local binder 数だけ持ち上げた family parameter variable が
declaration order で並ぶことを意味します。
index arguments は family `indices` telescope を parameter arguments と先行 index arguments で左から右へ instantiation した型の下で
well-typed でなければなりません。
この check で別 family への result、imported constant head、`LocalGenerated` head、または pretty name による補完を許してはいけません。

strict positivity check では、constructor type 内の任意の `GlobalRef::Local(i)` occurrence を
同一 mutual block 内の recursive occurrence として扱います。
`i` が `inductives` 範囲外の場合は `TargetRefMismatch` として拒否します。
imported occurrence は recursive occurrence ではありませんが、nested occurrence の head が
`approved_nested_type_constructors` にない場合は nested recursive occurrence を通してはいけません。
MVP の positivity traversal は保守的に次へ固定します。

```text
check_constructor_positivity(constructor_ty):
  1. constructor_ty の outer Pi binders を constructor result family check と同じ方法で剥がす
  2. 各 outer Pi binder domain を constructor argument type として check_strictly_positive_arg する
  3. final result は positivity traversal の対象にせず、constructor result family check だけで検査する

check_strictly_positive_arg(arg_ty):
  let arg0 = weak_head_normalize_for_inductive_positivity(arg_ty)

  if arg0 is App*(Const(GlobalRef::Local(family_index), level_args), args):
    - family_index は inductives 範囲内でなければならない
    - level_args は payload.universe_params だけを参照し、arity が一致しなければならない
    - parameter args は family params を現在の constructor-local context へ weakening したものと
      byte-for-byte 一致しなければならない
    - index args は recursive occurrence を含まず、family indices telescope の下で well-typed でなければならない
    - この形だけを direct recursive occurrence として許可する

  otherwise:
    - arg0 内に GlobalRef::Local(_) が現れなければ許可する
    - arg0 内に GlobalRef::Local(_) が現れた場合は nested / negative / hidden recursive occurrence として拒否する
```

`weak_head_normalize_for_inductive_positivity` は β / ζ だけを使います。
imported `DefDecl` の δ unfolding、opaque theorem、axiom、typeclass search、quotient computation rule、
AI hint、または runtime registry は使いません。
したがって `F I`、`I -> X`、`X -> I`、`List I`、reducible alias の中に隠れた `I` は、
MVP ではすべて `UnsupportedFeature` または kernel の positivity error として拒否します。
将来これらを許可する場合は、nested / higher-order positivity profile を別に定義し、
polarity traversal と approved type constructor の functoriality 証明を certificate に束縛します。

generated artifact の名前と順序は AI から受け取らず、次の `inductive_artifact_profile` で固定します。

```text
inductive_artifact_profile = "npa.phase9_ai.inductive-artifacts.v1"

family_public_name(i) =
  append_name(block_name, inductives[i].name)

constructor_public_name(i, j) =
  append_name(family_public_name(i), inductives[i].constructors[j].name)

recursor_public_name(i) =
  append_name(family_public_name(i), "rec")

iota_rule_key(i, j) =
  (recursor_public_name(i), constructor_public_name(i, j))
```

`append_name` は Phase 2 generated artifact name と同じ canonical `NameId` path append です。
生成後の family / constructor / recursor name が重複する candidate は拒否します。
validator は受理済み declaration から `generate_inductive_artifacts_v1` を1回だけ実行し、
constructor specs、recursor specs、iota rule specs を family order / constructor order で canonical encode します。
この生成関数の入力は proposal payload から再構成した canonical `base_inductive_decl_for_generation`、
`profile_version`、および import closure から解決済みの public interfaces だけです。
`expected_decl_hash`、AI explanation、または response destination は生成入力に含めません。
AI sidecar、runtime registry、現在の IDE state、または caller-provided recursor body は入力にしません。
MVP では `generate_inductive_artifacts_v1` は extension point ではなく、Phase 2 の canonical inductive artifact generator
と同じ実装を呼ぶ alias です。
別実装を Phase 9 validator 内だけに持ってはいけません。
Phase 2 側にこの profile 用 generator がまだ実装されていない場合、Advanced Inductive validator は
candidate を受理せず `UnsupportedFeature` を返します。
M3 を完了とみなせるのは、non-mutual / non-nested / non-large-elimination のこの profile が
Phase 2 generator と Phase 9 validator の両方から同じ canonical bytes を生成できる場合だけです。
Phase 9 core がまだ deterministic generator を定義していない組み合わせ
（例: mutual recursor、nested recursor、large elimination profile）は、
validator が独自生成を推測せず `UnsupportedFeature` として拒否します。

AI が出してよいもの:

```text
- inductive family declaration candidate
- constructor type candidate
- universe parameter candidate
- positivity failure repair suggestion
```

AI が出してはいけないもの:

```text
- trusted recursor body
- trusted eliminator axiom
- positivity override
- large elimination override
- constructor hash の手入力
```

kernel / checker は inductive declaration から recursor と computation rule を決定的に生成します。
AI 生成 recursor は、デバッグ用 sidecar として保存しても採用しません。

---

# 4. Universe Polymorphism AI

AI は universe error の修復候補を出せます。
ただし universe constraint の充足性は kernel 側の決定的な solver が判定します。

```rust
struct MachineUniverseRepairCandidate {
    goal: Option<Phase9AiGoal>,
    target_expr: CoreExpr,
    instantiations: Vec<MachineUniverseInstantiationPatch>,
    constraint_hints: Vec<UniverseConstraintHint>,
    minimization_hint: Option<UniverseMinimizationHint>,
}

struct MachineUniverseInstantiationPatch {
    occurrence: MachineExprOccurrence,
    explicit_level_args: Vec<LevelExpr>,
}

struct MachineExprOccurrence {
    path: Vec<MachineExprPathStep>,
    expected_ref: Phase9AiGlobalRef,
}

enum MachineExprPathStep {
    AppFun,
    AppArg,
    LamType,
    LamBody,
    PiDomain,
    PiCodomain,
    LetType,
    LetValue,
    LetBody,
}

struct UniverseConstraintHint {
    constraint: UniverseConstraint,
    reason: UniverseConstraintHintReason,
}
```

`minimization_hint` は探索順序のヒントだけです。
採用される level assignment は、canonical universe solver の出力でなければなりません。
`MachineUniverseInstantiationPatch` は、`target_expr` 内のどの occurrence に
universe args を与えるかを固定します。対象 ref は `occurrence.expected_ref` です。
複数の polymorphic occurrence がある場合、AI は occurrence ごとに
patch を分けて出します。
`instantiations` は `(occurrence.path canonical bytes, occurrence.expected_ref canonical bytes)` の辞書順で
strictly sorted され、同じ occurrence key を重複して含んではいけません。
同じ path / ref に同じ `explicit_level_args` を2回指定した場合も重複として拒否し、
validator が silently merge してはいけません。
同じ path に異なる `expected_ref` を指定した patch も、path traversal 後の ref 一致検査で少なくとも一方が
`TargetRefMismatch` になります。
`target.goal_fingerprint` が `Some` の場合、`goal` は `Some(Phase9AiGoal)` でなければなりません。
`goal.target` の canonical bytes は `target_expr` と byte-for-byte で一致し、common envelope の式で
`goal.universe_params`、`goal.local_context`、`goal.target` から fingerprint を再計算します。
closed expression repair は
`goal = Some(Phase9AiGoal { universe_params = [], local_context = [], target = target_expr })` として表します。
open goal の universe repair は `goal.universe_params` と `goal.local_context` を明示することで扱います。
`/machine/phase9/universe/repair/check` の MVP は goal mode だけを受け付けます。
`target.target_decl_hash` が `Some` の declaration repair mode は、この文書の MVP では payload schema を定義しないため
`UnsupportedFeature` として拒否します。将来有効化する場合は、target declaration 全体を request の canonical wrapper から
再構成できる別 payload を定義し、`target_decl_hash` の再計算手順をその payload に固定しなければなりません。
declaration repair mode では `goal = None` でなければなりません。
`explicit_level_args` は path で到達した `Const` occurrence の universe argument list を置換する patch です。
validator は `occurrence.expected_ref` の public interface から universe parameter order と arity を取得し、
`explicit_level_args.len()` が arity と一致しない場合は拒否します。
goal mode の各 `LevelExpr` は `payload.goal.universe_params` だけを free universe parameter として参照できます。
declaration repair extension では、別 payload に含まれる target declaration の `universe_params` だけを参照できます。
余剰・不足・重複 binder 参照・未宣言 parameter 参照は `IllFormedLevelExpr` または `UnknownUniverseParam` として拒否します。
patch 適用後の `target_expr` から validator が universe constraints を再導出し、canonical solver に渡します。
`MachineExprOccurrence.path` は elaboration 前 source span ではなく、`target_expr` の canonical `CoreExpr` tree path です。
validator は path の到達先が global constant occurrence であり、その core ref が
`resolve_imported_ref(occurrence.expected_ref)` と一致することを確認します。

path step の意味は CoreExpr variant ごとに固定します。

```text
App(f, a):
  AppFun -> f
  AppArg -> a

Lam(ty, body):
  LamType -> ty
  LamBody -> body

Pi(domain, codomain):
  PiDomain -> domain
  PiCodomain -> codomain

Let(ty, value, body):
  LetType -> ty
  LetValue -> value
  LetBody -> body

Sort(_):
  no child steps

Const(ref, level_args):
  no child steps

BVar(index):
  no child steps
```

empty path means `target_expr` itself. For universe instantiation, empty path is valid only when `target_expr` itself is a
global constant occurrence. Applying an invalid child step to a CoreExpr variant is `InvalidOccurrencePath`.
If path traversal reaches `Const` or `BVar`, additional path steps are invalid.
MVP の path table はここに列挙した CoreExpr variant に対して閉じています。
validator がこれ以外の CoreExpr variant を含む `target_expr` を受け取った場合、path traversal を推測してはいけません。
その request は、その variant 用の `MachineExprPathStep` が Phase 9 AI Profile に追加されるまで `UnsupportedFeature` として拒否します。

`constraint_hints` は AI repair の補助情報です。
canonical solver の入力は、`target_expr` と `instantiations` から validator が導出した制約だけです。
AI が `constraint_hints` で新しい trusted constraint を追加することはできません。

拒否する例:

```text
- undeclared universe parameter を参照する
- constraint graph に cycle がある
- cumulativity を使って forbidden coercion を通す
- pretty name だけで level を指定する
- target env_fingerprint と違う環境の repair を再利用する
- occurrence が target_expr 内で一意に解決できない
- path の到達先 ref と occurrence.expected_ref が一致しない
- constraint_hints が validator 導出 constraint と矛盾する
```

AI repair loop には構造化エラーを返します。

```rust
enum UniverseRepairError {
    UnknownUniverseParam,
    IllFormedLevelExpr,
    UnsatisfiedConstraint,
    NonCanonicalSolution,
    TargetFingerprintMismatch,
    InvalidOccurrencePath,
    AmbiguousOccurrence,
    TargetRefMismatch,
    ConstraintHintMismatch,
}
```

---

# 5. Typeclass AI

Typeclass search は core calculus に入りません。
AI は instance search の候補順や resolution plan を提案できますが、最終的な証明は
elaborated core term として kernel が検査します。

```rust
struct MachineTypeclassResolutionPlan {
    goal: Phase9AiGoal,
    ordered_candidates: Vec<MachineInstanceCandidateRef>,
    max_depth: u32,
    max_nodes: u32,
}

struct MachineInstanceCandidateRef {
    target: MachineInstanceTargetRef,
    priority_hint: Option<i32>,
}

enum MachineInstanceTargetRef {
    Imported {
        global_ref: Phase9AiGlobalRef,
    },
}
```

`priority_hint` は candidate hash には含めますが、正しさの根拠ではありません。
`ordered_candidates`、`priority_hint`、`max_depth`、`max_nodes` は executable search plan の一部です。
同じ `goal` と import closure でも、AI が違う順序を提案した場合は別 candidate として扱います。
Search order は `ordered_candidates` の配列順だけで決まります。
`priority_hint` は response/debug/training 用 metadata であり、resolver が candidate を再ソートしたり tie-break したりする入力に
使ってはいけません。

typeclass resolver の deterministic search rule は固定します。

```text
- search state は pending obligations queue、proof_args、visited stack、node_count を持つ
- 初期 queue は goal.target 1件だけ
- 1 branch 内の queue は FIFO で取り出す
- 各 obligation に対し ordered_candidates を配列順に走査する
- candidate の head が obligation の class head と defeq で一致しない場合は prefilter skip
- candidate を適用して生じる recursive obligations は、candidate telescope の引数順で queue の末尾へ追加する
- 同じ obligation fingerprint と candidate ref が visited stack にある場合は cycle としてその branch を拒否する
- node_count は candidate application を試した時点で 1 増やす
- depth は current branch の candidate application 数で数え、`max_depth` を超える candidate application は request 全体の `BudgetExceeded`
- `max_nodes = 0` または `max_depth = 0` は candidate application 禁止を意味し、非空の初期 queue では `BudgetExceeded`
```

`proof_args` は候補適用で確定した explicit argument、implicit argument、recursive proof slot の列であり、
`chosen instances` という独立した state は持ちません。
成功時に返す instance chain は `proof_args` と final proof term から導出される debug metadata にすぎず、
search result の canonical identity には final proof term の canonical bytes だけを使います。

`node_count` budget は一意性確認を含む探索全体に適用します。
head defeq mismatch による prefilter skip は candidate application attempt ではなく、
`node_count` と `current_depth` を消費しません。
candidate の head が一致した後、structural matching と argument classification を始める直前が
candidate application attempt の開始点です。
その時点で `node_count < max_nodes` でなければならず、満たさない場合は
request 全体を `BudgetExceeded` として拒否します。
同じく `current_depth < max_depth` でなければならず、満たさない場合はその時点で
request 全体を `BudgetExceeded` として拒否します。
両条件を満たす場合、resolver は candidate application attempt を開始する直前に `node_count += 1` し、
structural matching や kernel check が後で失敗してもこの `node_count` は戻しません。
candidate application が structural matching と argument classification に成功して child frame を作る場合だけ、
child frame の `current_depth` を 1 増やします。
recursive obligations が空で、残り queue だけを引き継ぐ child frame でも `current_depth` は増えます。
structural matching failure で child frame が作られない場合、親 frame の `current_depth` は変わりません。
1つ目の成功 proof term が見つかった後も、ambiguity を排除するために同じ budget 内で残りの探索を続けます。
2つ目の異なる successful proof term を見つけた場合は直ちに `AmbiguousResolution` です。
1つ目の成功がある状態で、残り探索中に node または depth budget を使い切った場合は success ではなく `BudgetExceeded` です。
つまり success を返してよいのは、budget 内で探索空間を尽くし、成功 proof term が canonical bytes で1種類だけだった場合に限ります。

DFS/backtracking の branch frame は次で固定します。

```text
BranchFrame:
  queue: pending obligations FIFO queue
  proof_args: chosen argument / recursive proof slots for all candidates on this branch
  visited_stack: obligation fingerprint + candidate ref entries on the current recursion path
  current_depth: candidate application count on this branch
```

`search(frame)` は `queue` が空なら success candidate を返します。
initial frame は `queue = [goal.target]`、`proof_args = []`、`visited_stack = []`、
`current_depth = 0` で作ります。
そうでなければ queue の先頭 obligation を取り出し、その obligation に対して `ordered_candidates` を配列順に走査します。
candidate application が structural matching で失敗した場合は同じ obligation の次 candidate を試します。
成功した場合は、残り queue の末尾に recursive obligations を candidate telescope order で追加した child frame を作り、
その child frame を先に探索します。
child frame が no-solution / cycle / kernel rejection で失敗した場合だけ、親 frame は同じ obligation の次 candidate を試します。
`BudgetExceeded` と `AmbiguousResolution` は branch failure ではなく request 全体の terminal result です。
`visited_stack` の更新は immutable frame stack として扱います。
candidate application の structural matching と argument classification が成功した後、
resolver は `(obligation_fingerprint_after_matching, resolved_candidate_ref)` を cycle entry として作ります。
この entry が親 frame の `visited_stack` に既に存在する場合、その child は cycle として no-solution branch failure になります。
存在しない場合だけ、child frame の `visited_stack` は
`parent.visited_stack ++ [entry]` です。
親 frame 自体は変更しないため、child 探索から戻った時点の pop は親 frame を再利用することで決定的に表現します。

candidate interface は、imported declaration の public type を weak-head normalize し、Pi telescope を剥がした
result type から決定します。
class declaration は `Phase9AiOptions.typeclass.class_declarations` に含まれる `Phase9AiGlobalRef` だけです。
validator は検証開始時に `class_declarations` の各 `Phase9AiGlobalRef` を `resolve_imported_ref` し、
`resolved_class_declarations: Set<GlobalRef::Imported>` を作ります。
解決できない class declaration ref、または同じ `GlobalRef::Imported` に解決される重複 ref は拒否します。
result type の head がこの `resolved_class_declarations` に含まれる core ref でなければ、その candidate は instance candidate として無効です。
`candidate の head` はこの result type head です。
各 obligation target も weak-head normalize し、head が同じ `resolved_class_declarations` に含まれなければ
`UnsupportedFeature` として拒否します。
`Phase9AiGlobalRef` の tuple / canonical bytes は request identity と import 解決のためだけに使い、
WHNF 後の head 比較は必ず解決済み core ref 同士で行います。
ここで使う weak-head normalize / definitional equality の transparency profile は固定です。
β / ζ / ι reduction と、Phase 2 `DefDecl.reducibility = Reducible` の δ unfolding だけを許可します。
opaque def、opaque theorem、axiom、typeclass search、implicit insertion、AI hint による unfold は使いません。
quotient_v1 の computation rule が有効な environment でも、typeclass head 判定では quotient lift の特殊 reduction を使いません。

MVP の matching は first-order な structural matching だけです。
imported candidate の public type は closed declaration interface から取り出し、
candidate 自身の universe parameter context と Pi telescope context の下で扱います。
validator は matching 前に次の task-local pattern context を作ります。

```text
candidate universe pattern:
  U_j for candidate universe_params[j], in declaration order

candidate term pattern:
  P_i for candidate telescope binder i, in Pi order
```

Pi telescope を剥がした result type 内で candidate telescope binder を指す `BVar` は、
その binder に対応する `P_i` として扱います。
candidate public type は closed interface なので、Pi telescope binder 以外を指す free `BVar` が result type に残る場合は
candidate interface invalid として拒否します。
structural matching は `goal.universe_params` / `goal.local_context` の下で、
candidate result type pattern と obligation target を左から右へ同時走査して行います。
同じ `P_i` または `U_j` に2回以上 assignment する場合、既存 assignment と新しい assignment の canonical bytes が一致しなければ
その candidate application は失敗です。
`P_i` に入る term は `goal.local_context` の下で well-scoped であり、candidate telescope の前方 binder assignment を
`P_0..P_(i-1)` に代入した binder type の term として kernel check できなければなりません。
`U_j` に入る level は `goal.universe_params` だけを free universe parameter として参照できます。
higher-order unification、backtracking unification、implicit argument insertion、typeclass search を matching の中で実行してはいけません。
universe assignment は candidate の declared universe parameter order に従って structural matching から一意に決まる場合だけ許可します。
未解決 universe parameter または term parameter が残る場合、その candidate application は失敗です。

candidate application の引数分類は次で固定します。

```text
1. imported candidate の public type を weak-head normalize し、Pi telescope と result type に分ける
2. result type と obligation target の structural matching で universe args と telescope binder assignment を埋める
3. telescope を左から右へ走査する
4. binder に assignment がある場合、その term を candidate application argument に入れる
5. binder に assignment がない場合、これまでの assignment を binder type に代入する
6. 代入後 binder type の weak-head head が class declaration set に含まれる場合、
   その binder type を recursive obligation とし、解けた dictionary proof term を同じ argument slot に入れる
7. 代入後 binder type が class obligation でない場合、その candidate application は失敗する
```

したがって最終 proof/dictionary term は
`candidate.target = Imported { global_ref }` の `global_ref` を
`resolve_imported_ref(global_ref)` した `Const` に、`matched_universe_args` と telescope order の arguments をすべて適用した term です。
recursive obligation の proof term もこの argument list 内の元 binder 位置に入ります。

branch 探索の主仕様は上の `BranchFrame` / `search(frame)` です。
実装はその frame stack を depth-first に処理し、同じ obligation の candidate cursor だけを backtracking point とします。
親 frame へ戻るのは child frame が no-solution / cycle / kernel rejection で失敗した場合だけです。
`BudgetExceeded` と `AmbiguousResolution` は backtracking 対象ではなく request 全体の terminal result です。
resolver は budget 内で探索空間を最後まで調べ、canonical bytes が異なる2つ目の成功 proof term を見つけた時点で
`AmbiguousResolution` として拒否します。

obligation fingerprint は `goal.universe_params`、`goal.local_context`、obligation target、
解決済み pattern assignment の canonical bytes から計算します。
pattern assignment は universe assignments `U_j` を declaration order、term assignments `P_i` を Pi order で encoding します。
ここでの assignment は typeclass resolver 内部の first-order matching result であり、
Phase 4 metavariable store や unification metavariable を作ってはいけません。
複数 branch が成功した場合は、成功 proof term の canonical bytes が完全一致する場合だけ同一解として扱います。
異なる canonical proof term が複数ある場合は `AmbiguousResolution` です。

replay invariant は次です。

```text
same candidate_hash
  = same goal
  + same import closure
  + same options_hash recomputed from Phase9AiOptionsRef
  + same ordered_candidates
  + same priority_hint values
  + same budget
  => same resolution result
```

candidate hash が違う resolution plan 同士に、同じ result を要求しません。
`ordered_candidates` は closed allowlist です。resolver はこの list に含まれない import instance を探索してはいけません。
未列挙 instance を探索する場合は、別の candidate hash を持つ新しい resolution plan として明示します。
Phase 9 AI MVP では checked current declaration を instance candidate として参照しません。

採用条件:

```text
- candidate ref が verified import に存在する
- imported candidate ref は `module / export_hash / certificate_hash / name / decl_interface_hash` で一意に解決できる
- goal.universe_params が重複なしで well-formed
- goal.target が goal.universe_params / goal.local_context の下で well-typed
- search budget 内で一意な solution が得られる
- elaborated instance term が goal.universe_params / goal.local_context の下で goal.target の proof/dictionary term として kernel check を通る
- ambiguity がある場合は拒否する
```

拒否するもの:

```text
- AI が選んだ instance を kernel check なしで採用する
- hidden global environment から instance を読む
- import closure 外の instance を暗黙に追加する
- checked current declaration を instance candidate として参照する
- ambiguity を score で解決する
```

---

# 6. Quotient AI

Quotient は trusted base を広げやすい機能です。
AI は quotient construction の補助をしてよいですが、同値関係や lift の well-definedness は
通常の proof obligation として検査します。

```rust
struct MachineQuotientConstructionCandidate {
    target_decl_hash: Hash256,
    decl_name: NameId,
    universe_params: Vec<UniverseParam>,
    params: Telescope,
    quotient_type: CoreExpr,
    carrier: CoreExpr,
    relation: CoreExpr,
    equivalence_proof: CoreExpr,
    operations: Vec<MachineQuotientOperationCandidate>,
}

struct MachineQuotientOperationCandidate {
    name: NameId,
    raw_function: CoreExpr,
    compatibility_proof: CoreExpr,
}
```

`operations` は `name` の canonical bytes 昇順で strictly sorted されていなければなりません。
同じ `name` を複数含む candidate は拒否します。
validator は受け取った順序を silently sort せず、順序違反を deterministic error として返します。
operation は quotient declaration そのものの `target_decl_hash` には入りません。
operation を declaration として出力する場合は、各 operation が別の Phase 2 `decl_certificate_hash` を持つ別 declaration になります。
この candidate では operation ごとの compatibility proof を検査しますが、`target_decl_hash` の再構成には
`decl_name / universe_params / params / carrier / relation / equivalence_proof` だけを使います。
payload の `quotient_type` は canonical quotient body との照合にだけ使い、`target_decl_hash` の body bytes には使いません。

MVP の quotient primitive は Phase 9 Human Profile の `Setoid` / `Quotient` primitive に合わせます。
AI validator は hidden builtin name table を使わず、`Phase9AiOptions.quotient` に明示された public imported refs だけを
primitive interface として扱います。
これらの ref は envelope imports から一意に解決でき、かつ解決先 certificate の feature report が `quotient_v1` を含み、
public type が `quotient_v1` の canonical interface と一致しなければなりません。
core term 内では、これらは通常の `Const(GlobalRef::Imported(...), level_args)` として表します。
`GlobalRef::Primitive` のような隠れた参照 variant は導入しません。
quotient primitive の core application は、すべて left-associated `App` で作ります。
`u` は carrier universe、`v` は operation result universe です。

```text
qconst(ref, levels) =
  Const(resolve_imported_ref(ref), levels)

setoid_type(carrier) =
  App(qconst(options.quotient.setoid, [u]), carrier)

rel_equiv_type(carrier, relation) =
  App(App(qconst(options.quotient.rel_equiv, [u]), carrier), relation)

setoid_mk_app(carrier, relation, equivalence_proof) =
  App(
    App(
      App(qconst(options.quotient.setoid_mk, [u]), carrier),
      relation),
    equivalence_proof)

setoid_relation_app(setoid_expr, a, b) =
  App(
    App(
      App(qconst(options.quotient.setoid_relation, [u]), setoid_expr),
      a),
    b)

eq_app(sort_level, result_type, lhs, rhs) =
  App(
    App(
      App(qconst(options.quotient.eq, [sort_level]), result_type),
      lhs),
    rhs)

quotient_type_app(setoid_expr) =
  App(qconst(options.quotient.quotient, [u]), setoid_expr)

quotient_mk_app(setoid_expr, a) =
  App(
    App(qconst(options.quotient.quotient_mk, [u]), setoid_expr),
    a)

quotient_sound_app(setoid_expr, a, b, relation_proof) =
  App(
    App(
      App(
        App(qconst(options.quotient.quotient_sound, [u]), setoid_expr),
        a),
      b),
    relation_proof)

quotient_lift_app(setoid_expr, result_type, raw_function, compatibility_proof) =
  App(
    App(
      App(
        App(qconst(options.quotient.quotient_lift, [u, v]), setoid_expr),
        result_type),
      raw_function),
    compatibility_proof)
```

`Phase9QuotientOptions` の各 ref は、この application builder で使う explicit level arg order と
term arg order の public type を持たなければなりません。public type がこの schema と definitional equality で
一致しない ref は `TargetRefMismatch` として拒否します。

`params` の下で期待型は次です。

```text
type_level = succ u

carrier : Sort type_level

relation :
  carrier -> carrier -> Prop

equivalence_proof :
  rel_equiv_type(carrier, relation)
```

`rel_equiv_type(carrier, relation)` は equivalence proof object の canonical type です。
validator は `rel_equiv` の内部 field / projection を hidden rule で読んではいけません。
`rel_equiv` が record として実装される場合でも、refl / symm / trans の存在は imported certificate の public type と
kernel check によって検査され、Phase 9 AI validator の追加 introspection 対象にはしません。

```text
setoid_expr =
  setoid_mk_app(carrier, relation, equivalence_proof)

quotient_type :
  Sort type_level

quotient_type defeq
  quotient_type_app(setoid_expr)

options.quotient.quotient_mk :
  (s : setoid_type(carrier)) ->
  carrier -> quotient_type_app(s)

options.quotient.quotient_sound :
  forall s : setoid_type(carrier),
  forall a b : carrier,
  setoid_relation_app(s, a, b) ->
    eq_app
      type_level
      (quotient_type_app(s))
      (quotient_mk_app(s, a))
      (quotient_mk_app(s, b))

options.quotient.quotient_lift :
  forall s : setoid_type(carrier),
  forall result_type : Sort (succ v),
  (f : carrier -> result_type) ->
  (forall a b : carrier,
    setoid_relation_app(s, a, b) ->
    eq_app(succ v, result_type, f a, f b)) ->
  quotient_type_app(s) ->
  result_type
```

`Type u` は core では `Sort (succ u)` なので、surface の `A : Type u` は上の `carrier : Sort type_level` に対応します。
`u` は `universe_params` 内の level parameter、または validator が `carrier` の inferred sort から導出した level expression です。
validator は `params` の下で `carrier` の type を推論し、その sort level を canonical normalize します。
その level が byte-for-byte に `succ u` へ分解できる場合だけ `u` を採用します。
`succ _` 形へ一意に分解できない sort level、または `payload.universe_params` 外の parameter を含む `u` は `TargetRefMismatch` または
feature-specific な universe mismatch として拒否します。
`eq_app` の第1引数は `Type` index ではなく、equality の domain type が住む core `Sort` level です。
したがって `quotient_type_app(s) : Sort type_level` に対する equality は `eq_app(type_level, ...)` を使い、
`result_type : Sort (succ v)` に対する equality は `eq_app(succ v, ...)` を使います。
`Phase9QuotientOptions` の全 ref は `quotient_v1` profile が定義する canonical primitive interface です。
AI payload が別の record shape、tuple、自然言語説明で equivalence を表した場合は拒否します。

MVP の `MachineQuotientOperationCandidate` は unary lift だけを表します。
`raw_function` の型は `params` の下で次へ weak-head normalize しなければなりません。

```text
raw_function :
  carrier -> result_type
```

`result_type` は `carrier` binder に依存せず、ある universe level `v` について `result_type : Sort (succ v)` でなければなりません。
validator は `params` の下で `result_type` の type を推論し、その sort level を canonical normalize します。
その level が byte-for-byte に `succ v` へ分解できる場合だけ `v` を採用します。
`succ _` 形へ一意に分解できない sort level、または `payload.universe_params` 外の parameter を含む `v` は `TargetRefMismatch` または
feature-specific な universe mismatch として拒否します。
`compatibility_proof` の期待型は次です。

```text
forall a b : carrier,
  relation a b ->
  eq_app(succ v, result_type, raw_function a, raw_function b)
```

ここでの equality は `options.quotient.eq` で固定します。typeclass search や notation 解決で補完してはいけません。

operation declaration を出力する場合、validator は
`quotient_lift_app(setoid_expr, result_type, raw_function, compatibility_proof)` から決定的に body を作ります。
multi-argument lift、dependent motive、quotient-to-quotient operation の追加自動展開は MVP では `UnsupportedFeature` です。

quotient target declaration は Phase 2 `DefDecl` として固定します。
`params = [p0, p1, ..., p(n-1)]` の下で作った `quotient_type` は、target declaration へ入れる前に
local binders を決定的に閉じます。

```text
close_params_type(params, body):
  if params is empty:
    body
  else:
    Pi(p0.ty, close_params_type(params[1..], body))

close_params_value(params, body):
  if params is empty:
    body
  else:
    Lam(p0.ty, close_params_value(params[1..], body))
```

実装は body 側から逆順 fold してもよいですが、結果 AST は上の再帰定義と byte-for-byte に一致しなければなりません。
`params` の binder name は diagnostic metadata として保存してよいですが、Phase 2 `Pi` / `Lam` node の canonical bytes には入りません。

```text
DefDecl:
  name = decl_name
  universe_params = payload.universe_params
  type = close_params_type(params, Sort type_level)
  value = close_params_value(params, quotient_type_app(setoid_expr))
  reducibility = Reducible
```

`target_decl_hash` はこの `DefDecl` から Phase 2 の通常規則で計算した `decl_certificate_hash` です。
payload の `quotient_type` は採用条件の defeq check にだけ使い、DefDecl の body には入れません。
`equivalence_proof` は canonical body の `setoid_expr` 経由で DefDecl の `value_hash` と `decl_interface_hash` に反映されます。
したがって MVP の quotient declaration identity は proof-sensitive です。
同じ `carrier` と `relation` でも、`equivalence_proof` の canonical bytes が違えば別の quotient declaration hash になります。
validator は `equivalence_proof` を proof irrelevance、record projection、または proof normalization で消してはいけません。
relation だけに依存する proof-irrelevant quotient identity を導入する場合は、
`quotient_v2` のような別 feature profile と hash rule を定義してから有効化します。
`operations` を declaration として出力する future extension でも、各 operation は別 `DefDecl` とし、
`type = close_params_type(params, quotient_type_app(setoid_expr) -> result_type)`、
`value = close_params_value(params, quotient_lift_app(setoid_expr, result_type, raw_function, compatibility_proof))`、
`reducibility = Reducible` を使います。
MVP の `/machine/phase9/quotient/check` は operation declaration hash を返さず、compatibility 検査だけを行います。

採用条件:

```text
- payload.target_decl_hash が envelope target.target_decl_hash と一致する
- options.quotient が Some であり、すべての primitive refs が envelope imports から一意に解決できる
- target_decl_hash は quotient construction で新規生成される quotient declaration の Phase 2 `decl_certificate_hash` である
- validator が decl_name / universe_params / params / carrier / relation / equivalence_proof から
  上記の Phase 2 `DefDecl` を決定的に再構成し、
  その `decl_certificate_hash` が target_decl_hash と一致する
- quotient_type は params の下で `quotient_type_app(setoid_expr)` の canonical primitive type と definitional equality で一致する
- relation が carrier 上の relation として well-typed
- equivalence_proof が `rel_equiv_type(carrier, relation)` の proof term として kernel check を通る
- quotient primitive の intro / elim / soundness rule だけを使う
- operation ごとの raw_function と compatibility_proof が上の unary lift interface で kernel check を通る
- resulting certificate の feature report に `quotient_v1` が記録される
- independent checker が `quotient_v1` をサポートしない profile では `UnsupportedFeature` として拒否する
- certificate には natural language explanation を入れない
```

AI が「同値関係らしい」と説明しても、それは証明ではありません。
`equivalence_proof` と `compatibility_proof` の core term が検査されるまで採用しません。

---

# 7. SMT Certificates AI

SMT solver 本体と AI は trusted base に入りません。
Phase 9 AI で扱う SMT 結果は、証明再構成できる certificate candidate だけです。

```rust
struct MachineSmtCertificateCandidate {
    goal: Phase9AiGoal,
    logic: SmtLogic,
    encoded_problem: MachineSmtProblemRef,
    certificate_format: SmtCertificateFormat,
    proof_payload: MachineSmtProofPayloadRef,
    reconstruction_plan: MachineSmtReconstructionPlan,
}

enum MachineSmtProblemRef {
    Inline {
        problem_hash: Hash256,
        encoding_hash: Hash256,
        canonical_bytes: Vec<u8>,
    },
    Artifact {
        path: ArtifactPath,
        file_hash: Hash256,
        problem_hash: Hash256,
        encoding_hash: Hash256,
        size_bytes: u64,
    },
}

struct MachineSmtEncodedProblem {
    encoder_version: SmtEncoderVersion,
    goal_fingerprint: Hash256,
    logic: SmtLogic,
    command_profile: SmtCommandProfile,
    commands: Vec<SmtEncodedCommand>,
}

enum SmtCommandProfile {
    MvpNormalizedQf,
}

struct SmtEncodedCommand {
    phase: SmtCommandPhase,
    command_id: SmtCommandId,
    payload: SmtCommandPayload,
}

enum SmtCommandPhase {
    SortDecl,
    DatatypeDecl,
    FunctionDecl,
    ContextAssumption,
    TargetAssertion,
    FinalCheck,
}

enum SmtCommandPayload {
    SortDecl {
        symbol: SmtSymbol,
        arity: u32,
    },
    FunctionDecl {
        symbol: SmtSymbol,
        args: Vec<SmtSortExpr>,
        result: SmtSortExpr,
    },
    DatatypeDecl {
        symbol: SmtSymbol,
        constructors: Vec<SmtDatatypeConstructor>,
    },
    ContextAssumption {
        source_local_index: u32,
        core_expr: CoreExpr,
        encoded_expr: SmtExpr,
    },
    TargetAssertion {
        core_expr: CoreExpr,
        encoded_expr: SmtExpr,
    },
    FinalCheck,
}

enum SmtSortExpr {
    Bool,
    Int,
    BitVec {
        width: u32,
    },
    User {
        symbol: SmtSymbol,
        args: Vec<SmtSortExpr>,
    },
}

struct SmtDatatypeConstructor {
    constructor: SmtSymbol,
    selectors: Vec<SmtDatatypeSelector>,
}

struct SmtDatatypeSelector {
    selector: SmtSymbol,
    sort: SmtSortExpr,
}

enum SmtExpr {
    Var {
        symbol: SmtSymbol,
        sort: SmtSortExpr,
    },
    BoolLit(bool),
    IntLit(i128),
    BitVecLit {
        width: u32,
        value: Vec<u8>,
    },
    App {
        symbol: SmtSymbol,
        args: Vec<SmtExpr>,
        result_sort: SmtSortExpr,
    },
    Not(Box<SmtExpr>),
    And(Vec<SmtExpr>),
    Or(Vec<SmtExpr>),
    Eq(Box<SmtExpr>, Box<SmtExpr>),
    Imp(Box<SmtExpr>, Box<SmtExpr>),
    Ite {
        cond: Box<SmtExpr>,
        then_expr: Box<SmtExpr>,
        else_expr: Box<SmtExpr>,
    },
}

struct SmtProofNodeTable {
    certificate_format: SmtCertificateFormat,
    nodes: Vec<SmtProofNode>,
}

struct SmtProofNode {
    node_id: SmtPayloadNodeId,
    rule_fingerprint: Hash256,
    premises: Vec<SmtPayloadNodeId>,
    conclusion_encoding: SmtConclusionEncoding,
}

struct SmtConclusionEncoding {
    encoder_version: SmtEncoderVersion,
    logic: SmtLogic,
    command_profile: SmtCommandProfile,
    core_expr: CoreExpr,
    encoded_expr: SmtExpr,
}

enum MachineSmtProofPayloadRef {
    Inline {
        payload_hash: Hash256,
        canonical_bytes: Vec<u8>,
    },
    Artifact {
        path: ArtifactPath,
        file_hash: Hash256,
        payload_hash: Hash256,
        size_bytes: u64,
    },
}

struct MachineSmtReconstructionPlan {
    imported_theory_refs: Vec<Phase9AiGlobalRef>,
    steps: Vec<MachineSmtReconstructionStep>,
    final_step: SmtReconstructionStepId,
    final_proof: CoreExpr,
}

struct MachineSmtReconstructionStep {
    step_id: SmtReconstructionStepId,
    rule: SmtReconstructionRule,
    payload_bindings: Vec<MachineSmtPayloadBinding>,
    premises: Vec<SmtReconstructionStepId>,
    conclusion: CoreExpr,
    proof: CoreExpr,
}

struct MachineSmtPayloadBinding {
    payload_hash: Hash256,
    node_id: SmtPayloadNodeId,
    rule_fingerprint: Hash256,
}

enum SmtReconstructionRule {
    PayloadNode {
        certificate_format: SmtCertificateFormat,
        rule_fingerprint: Hash256,
    },
    LocalBookkeeping {
        kind: SmtLocalBookkeepingRule,
    },
}

enum SmtLocalBookkeepingRule {
    ReorderPremises {
        permutation: Vec<u32>,
    },
    IntroduceTheoryLemma {
        lemma: Phase9AiGlobalRef,
        level_args: Vec<LevelExpr>,
        term_args: Vec<CoreExpr>,
    },
    ComposeProof {
        combinator: Phase9AiGlobalRef,
        level_args: Vec<LevelExpr>,
        term_args: Vec<CoreExpr>,
    },
}
```

`encoded_problem` は replay input の一部です。
validator は `Inline.canonical_bytes` または `Artifact` の `path` / `file_hash` で固定された bytes から
`problem_hash` と `encoding_hash` を再計算します。SMT solver process や solver log から problem を補完してはいけません。
`encoded_problem` bytes は `MachineSmtEncodedProblem` として canonical decode できなければなりません。

```text
problem_hash =
  sha256("npa.phase9_ai.smt.problem.v1" || canonical_bytes(MachineSmtEncodedProblem))

encoding_hash =
  sha256(
    "npa.phase9_ai.smt.encoding.v1"
    || encoder_version canonical bytes
    || logic canonical bytes
    || command_profile canonical bytes
    || goal_fingerprint digest bytes
    || problem_hash digest bytes
  )
```

`MachineSmtEncodedProblem.goal_fingerprint` は、`payload.goal` から再計算した envelope の
`target.goal_fingerprint` と一致しなければなりません。
`MachineSmtEncodedProblem.logic` は `MachineSmtCertificateCandidate.logic` と一致しなければなりません。
`commands` は raw SMT-LIB command 列ではなく、`encoder_version` と `command_profile` が定義する normalized IR の列です。
`SmtEncoderVersion` と `SmtCommandProfile` は validator profile 内の closed deterministic encoder table を指します。
この table は、対応する `CoreExpr` variant、対応する imported constant、SMT symbol derivation、sort mapping、
および theory side condition generation を明示的に定義します。
table にない `CoreExpr` variant、`GlobalRef`、level pattern、または theory mapping を見つけた場合、
validator は AI annotation、pretty name、solver log、または hidden standard library registry から推測せず
`UnsupportedFeature` として拒否します。
`FunctionDecl.symbol`、`SortDecl.symbol`、datatype constructor / selector symbol、`Var.symbol` は
この deterministic encoder table と goal / import identity から再生成できなければなりません。
caller が任意に選んだ SMT symbol を受け入れてはいけません。
MVP の `command_profile = MvpNormalizedQf` では、push/pop、incremental assertion、solver option、named assertion side effect を禁止します。
`SmtEncodedCommand canonical bytes` は `phase / command_id / payload` の順に encoding します。
`SmtEncodedCommand.phase` と `SmtEncodedCommand.payload` の variant は一致しなければなりません。
たとえば `phase = SortDecl` かつ `payload = FunctionDecl { ... }` の command は `PayloadHashMismatch` として拒否します。
payload variant 内の list は表示順ではなく、各 variant が定める canonical order で保存します。
`FunctionDecl.args` は関数 signature の意味を持つため source encoding order を canonical order とし、sort してはいけません。
`DatatypeDecl.constructors` と `SmtDatatypeConstructor.selectors` も source encoding order を canonical order とします。
`command_id` は caller が任意に選ぶ識別子ではなく、validator が payload から再生成します。
`SmtCommandId` の canonical bytes は次で固定します。

```text
command_id(command) =
  sha256(
    "npa.phase9_ai.smt.command_id.v1"
    || command.phase canonical bytes
    || command_id_source_key(command.payload)
  )

command_id_source_key:
  SortDecl { symbol, ... }       -> canonical_bytes(symbol)
  DatatypeDecl { symbol, ... }   -> canonical_bytes(symbol)
  FunctionDecl { symbol, ... }   -> canonical_bytes(symbol)
  ContextAssumption { source_local_index, ... }
                                  -> canonical_u32(source_local_index)
                                     || canonical_bytes(core_expr)
  TargetAssertion { ... }        -> empty bytes
  FinalCheck                     -> empty bytes
```

payload に入っている `command_id` がこの再計算値と一致しない場合は `PayloadHashMismatch` として拒否します。
同じ phase で `command_id_source_key` が重複する command は source key uniqueness 違反として拒否します。
`commands` は次の phase 順で並べ、各 phase 内だけを `(command_id, SmtEncodedCommand canonical bytes)` の辞書順に sort します。
`command_id` は command list 全体で重複してはいけません。

```text
1. SortDecl
2. DatatypeDecl
3. FunctionDecl
4. ContextAssumption
5. TargetAssertion
6. FinalCheck
```

`FunctionDecl` は `DatatypeDecl` phase の後に置くため、function signature は user sort と datatype sort の両方を参照できます。
MVP の `DatatypeDecl` payload 内の selector sort は builtin sort と `SortDecl` の user sort だけを参照できます。
別の `DatatypeDecl` symbol、同じ declaration の symbol、または後続 declaration の symbol を参照した場合は recursive /
mutually-recursive datatype として拒否します。

`ContextAssumption.source_local_index` は `goal.local_context` 配列の 0-based index です。
これは de Bruijn index ではありません。`source_local_index = i` が指す local binder は `goal.local_context[i]` であり、
core term 内で同じ binder を参照する de Bruijn index は `goal.local_context.len() - 1 - i` です。
`ContextAssumption.core_expr` はその local declaration の type、または value 付き local declaration から生じる equality assumption だけを許可します。
local declaration の type から生じる assumption と、value 付き local declaration から生じる equality assumption は次で固定します。

```text
let decl = goal.local_context[i]
let k = goal.local_context.len() - 1 - i

local_type_assumption(i) =
  weaken(decl.ty, k)

if decl.value = Some(v):
  local_value_equality(i) =
    EqApp(
      weaken(decl.ty, k),
      BVar(k),
      weaken(v, k)
    )
```

`weaken(expr, k)` は `decl` より後ろにある `k` 個の local binder の下へ `expr` を持ち上げる標準 weakening です。
`ContextAssumption.core_expr` は `local_type_assumption(i)` または `local_value_equality(i)` のどちらかと
canonical bytes が完全一致しなければなりません。
`local_type_assumption(i)` を使う場合、その expression は full goal context の下で `Prop` として well-typed でなければなりません。
`decl.ty` が proposition でない local binder は SMT assertion として出してはいけません。
`EqApp(T, lhs, rhs)` は、validator が full goal context の下で `T : Sort eq_sort_level` を推論し、
`Const(canonical_eq_ref, [eq_sort_level]) T lhs rhs` へ展開する left-associated core application です。
`canonical_eq_ref` は `resolve_imported_ref(options.smt.eq)` で得る Phase 1 `Eq` の exact `GlobalRef::Imported` です。
`options.smt.eq` は envelope imports から一意に解決でき、public type が
`Eq.{u} : Pi A : Sort u, A -> A -> Prop` と definitional equality で一致しなければなりません。
SMT payload、encoder hidden table、または kernel builtin registry から補完してはいけません。
MVP encoder profile が equality constant と level derivation rule を一意に持たない場合、value 付き local declaration の
equality assumption は `UnsupportedFeature` として拒否します。
`options.smt.prop_false = Some(ref)` の場合、その ref は envelope imports から一意に解決でき、public type が
`False : Prop` と definitional equality で一致しなければなりません。
`options.smt.prop_not = Some(ref)` の場合、その ref は envelope imports から一意に解決でき、public type が
`Not : Prop -> Prop` と definitional equality で一致しなければなりません。
これらが `None` の profile では、core proposition 側の `False` / `Not` を必要とする encoded conclusion や
refutation bridge は `UnsupportedFeature` として拒否します。
Boolean connective、contradiction elimination、または refutation bridge に必要な theorem / combinator は、
hidden builtin registry から補完しません。
それらが NPA core proof reconstruction に必要な場合は、
`reconstruction_plan.imported_theory_refs` と `IntroduceTheoryLemma` / `ComposeProof` の explicit arguments で渡します。
`prop_false` / `prop_not` は deterministic encoder が core proposition と SMT Bool connective を対応づけるための
recognition head だけであり、単独では proof rule を追加しません。
MVP deterministic encoder は core proposition の `False` / `Not` を次の形だけで認識します。

```text
encode_prop(p):
  let p0 = weak_head_normalize_for_smt_encoder(p)

  if options.smt.prop_false = Some(false_ref)
     and p0 is exactly Const(resolve_imported_ref(false_ref), []):
       return BoolLit(false)

  if options.smt.prop_not = Some(not_ref)
     and p0 is exactly App(Const(resolve_imported_ref(not_ref), []), q):
       return Not(encode_prop(q))

  otherwise:
       continue with equality / local assumption / supported theory encoding rules,
       or reject as UnsupportedFeature
```

`BoolLit(true)` に対応する core proposition head は MVP では定義しません。
`weak_head_normalize_for_smt_encoder` は `MvpNormalizedQf` では β / ζ のみを使います。
imported `DefDecl` の δ unfolding、recursor ι reduction、opaque theorem、axiom、typeclass search、
quotient_v1 computation rule、SMT solver result、または AI hint を使いません。
`Not` の argument `q` は normalized core expression の syntactic application shape からだけ取り出し、
definitional equality search から合成してはいけません。
`False` / `Not` の recognition は `resolve_imported_ref` 済み core ref と canonical core shape だけで行い、
pretty name、notation、hidden builtin、または SMT solver の builtin symbol 名から補完してはいけません。
MVP の `command_profile = MvpNormalizedQf` は refutation mode で固定します。
`TargetAssertion` はちょうど1つだけ存在し、その `core_expr` は `goal.target` と canonical bytes が完全一致しなければなりません。
`TargetAssertion.encoded_expr` は deterministic encoder が `goal.target` から生成した target expression を SMT の `Not(...)` で包んだ
canonical refutation assertion でなければなりません。
target を否定せずに直接 assertion として入れる profile は MVP では `UnsupportedFeature` です。
`FinalCheck` は payload を持たず、command list にちょうど1つだけ存在し、最後の phase に置かなければなりません。
MVP の `SmtSortExpr` / `SmtDatatypeConstructor` / `SmtExpr` canonical bytes は上の enum variant と field order で固定します。
`And` / `Or` の operands は `SmtExpr` canonical bytes 昇順で strictly sorted し、重複 operand を拒否します。
`FunctionDecl.args`、`App.args`、datatype constructor list、selector list は source encoding order を持つため、入力順を canonical order とします。
`BitVecLit.value` は最小 byte length ではなく、`width` から決まる fixed byte length の big-endian bytes です。
`width` が 8 の倍数でない場合、先頭 byte の未使用 high bits はすべて 0 でなければなりません。
つまり `BitVecLit.value` が表す unsigned integer は `0 <= value < 2^width` を満たし、
同じ bitvector 値に複数の byte 表現を許してはいけません。
MVP では binder 名、pretty text、solver-generated temporary name を含めません。

validator は `commands` から deterministic symbol table を作り、すべての `SmtExpr` / `SmtSortExpr` を検査します。

```text
sort symbol table:
  - Bool / Int / BitVec are builtin sorts
  - SortDecl.symbol is unique across SortDecl
  - SortDecl.arity is the required number of User.args
  - DatatypeDecl.symbol also introduces a User sort with arity 0
  - User(symbol, args) is valid only if symbol exists in SortDecl with args.len = arity,
    or symbol exists in DatatypeDecl with args.len = 0

function symbol table:
  - FunctionDecl.symbol is unique across FunctionDecl
  - FunctionDecl args/result sorts are valid
  - App(symbol, args, result_sort) using a function symbol is valid only if
    args length and each arg sort match the declared signature and result_sort matches declared result
  - App(symbol, args, result_sort) using a constructor or selector symbol is checked against the
    signature derived from its DatatypeDecl

datatype symbol table:
  - DatatypeDecl.symbol is unique across DatatypeDecl and distinct from SortDecl symbols
  - constructor symbols are globally unique across all DatatypeDecl and distinct from FunctionDecl symbols
  - selector symbols are globally unique across all DatatypeDecl and distinct from FunctionDecl / constructor symbols
  - constructor / selector signatures are derived only from DatatypeDecl payload
  - MVP rejects recursive and mutually-recursive datatype declarations; selector sorts must not refer to any DatatypeDecl symbol

variable symbol table:
  - Var symbols come only from deterministic encoder output for goal.local_context binders and target-local skolem symbols
  - the same Var symbol must always have byte-identical SmtSortExpr
  - caller-provided fresh variable names outside the encoder table are rejected
```

`Not` / `And` / `Or` / `Imp` require Bool operands and return Bool.
`Eq(lhs, rhs)` requires lhs and rhs to have byte-identical sorts and returns Bool.
`Ite` requires Bool condition and byte-identical then/else sorts, and returns that branch sort.
`BitVec.width` and `BitVecLit.width` must be greater than 0, and `BitVecLit.value.len()` must equal `ceil(width / 8)`.
validator は deterministic encoder を再実行し、command kind ごとに expected encoded expression を作ります。
`ContextAssumption` では `expected = encode(ContextAssumption.core_expr)` です。
`TargetAssertion` では `core_expr` が `goal.target` と一致することを先に確認し、
MVP refutation mode の `expected = Not(encode(TargetAssertion.core_expr))` とします。
payload の `encoded_expr` はこの `expected` と canonical bytes 一致しなければなりません。
一致しない command は `PayloadHashMismatch` または feature-specific encoding mismatch として拒否します。
この phase 順を満たさない bytes は canonical decode 後に拒否します。

`TargetAssertion` の `Not(...)` は encoded problem command の refutation wrapper であり、
`goal.target` 自体を SMT assertion として証明済みにする規則ではありません。
validator は `TargetAssertion.core_expr = goal.target` を、そのまま `goal.target` の proof premise として扱ってはいけません。
proof payload 側で `TargetAssertion.encoded_expr` と同じ SMT formula を参照する場合、その
`SmtConclusionEncoding.core_expr` は deterministic encoder で `Not(encode(goal.target))` に対応する
well-typed core proposition でなければなりません。
MVP でこの proposition を作る場合、validator は `options.smt.prop_not = Some(ref)` を要求し、
`App(Const(resolve_imported_ref(ref), []), goal.target)` と defeq で一致する core proposition だけを
`Not(encode(goal.target))` の対応物として認めます。
そのような core proposition を request 内の imports と explicit `CoreExpr` から構成できない場合、MVP では `UnsupportedFeature` として拒否します。
solver refutation から `goal.target` への橋渡しは、`ComposeProof` などの local bookkeeping step と
明示 imported combinator によって行い、final step の `conclusion` は必ず `payload.goal.target` と defeq で一致させます。
たとえば refutation proof が `not_goal -> False` 型の conclusion へ再構成される profile では、
`goal.target` への変換は imported theorem
`by_contradiction` や profile-specific contradiction eliminator を `ComposeProof` で明示適用する形にします。
validator は refutation mode であることだけを理由に `goal.target` の proof を暗黙生成してはいけません。

`proof_payload` は replay input の一部です。
validator は `Inline.canonical_bytes` または `Artifact` の `path` / `file_hash` で固定された bytes から
`payload_hash` を再計算します。filesystem discovery や solver log lookup で payload を補完してはいけません。

```text
payload_hash =
  sha256("npa.phase9_ai.smt.proof_payload.v1" || canonical_bytes(SmtProofNodeTable))
```

`proof_payload` bytes は `certificate_format` ごとの canonical SMT proof node table として decode できなければなりません。
MVP では、payload node table は `(node_id, rule_fingerprint, premises, conclusion_encoding)` の canonical order
で一意に decode できる形式だけを受け付けます。
`SmtPayloadNodeId` は payload-local `u32` label であり、semantic proof DAG の同型性に対する canonical renumbering は
MVP では行いません。
validator は node id を振り直して payload を正規化してはいけません。
同じ proof DAG でも node id や独立 node の順序が異なる payload は、別の `payload_hash` を持つ別 certificate として扱います。
ただし任意の sparse id による表現揺れを避けるため、MVP では `nodes[k].node_id == k as u32` でなければならず、
`node_id` は 0 から `nodes.len() - 1` まで連続していなければなりません。
`nodes.len() > u32::MAX` の payload は canonical decode error として拒否します。
`rule_fingerprint` は AI が新しい trusted rule を定義するための field ではありません。
validator は `(certificate_format, logic, command_profile, rule_fingerprint)` を key にする closed SMT rule registry を
validator profile 内に持ちます。
registry entry は premise count、premise order、encoded conclusion check、必要な side condition、
および NPA reconstruction rule への対応を決定的に定義します。
registry に存在しない `rule_fingerprint`、または registry が premise order を一意に定義できない rule は
`UnsupportedFeature` として拒否します。
この registry は payload、solver log、network lookup、または AI explanation から拡張してはいけません。
`SmtProofNodeTable.certificate_format` は `MachineSmtCertificateCandidate.certificate_format` と一致しなければなりません。
`nodes` は `node_id` の昇順で strictly sorted され、重複 node id や連番でない node id を含んではいけません。
各 `SmtProofNode.premises` は node table 内の既出 `node_id` だけを参照し、payload table 単体でも acyclic でなければなりません。
`SmtProofNode.premises` は certificate format が定義する proof rule premise order を canonical order とし、node_id で sort してはいけません。
この順序は reconstruction step の `premises` と positional に照合します。
premise order を一意に定義できない certificate format は MVP では `UnsupportedFeature` です。
`SmtConclusionEncoding.encoder_version` / `logic` / `command_profile` は `encoded_problem` と一致しなければなりません。
`SmtConclusionEncoding.core_expr` は、同じ `goal.universe_params` と `goal.local_context` の下で
well-typed な proposition でなければなりません。
validator は deterministic encoder を再実行し、`SmtConclusionEncoding.core_expr` から得た `SmtExpr` が
同じ `encoder_version` / `logic` / `command_profile` の下で `SmtConclusionEncoding.encoded_expr` と
canonical bytes 一致することを確認します。
`SmtConclusionEncoding` では `TargetAssertion` の command-level `Not(...)` wrapper を暗黙に追加しません。
`SmtConclusionEncoding.encoded_expr = Not(encode(goal.target))` を使うなら、対応する
`core_expr` も `encode(core_expr) = Not(encode(goal.target))` を満たす明示 core proposition でなければなりません。
SMT rule validator は payload node の `encoded_expr` と premise payload node の `encoded_expr` を入力にし、
caller-provided `core_expr` を solver rule の根拠として直接信頼してはいけません。
`MachineSmtPayloadBinding.payload_hash` は、同じ `MachineSmtCertificateCandidate.proof_payload` から再計算した
`payload_hash` と一致しなければなりません。各 reconstruction step は、少なくとも1つの payload node に結びつくか、
`SmtReconstructionRule` が明示的に local bookkeeping rule として定義されている必要があります。
MVP の `PayloadNode` step は `payload_bindings.len() == 1` でなければなりません。
複数 payload node を1つの reconstruction step にまとめる candidate は `UnsupportedFeature` として拒否します。
`SmtReconstructionRule::PayloadNode.rule_fingerprint` は、すべての `payload_bindings[*].rule_fingerprint` と一致しなければ
なりません。`LocalBookkeeping` は payload node を持たない step でだけ使えます。
MVP で proof-producing rule として使える local bookkeeping は `IntroduceTheoryLemma` と `ComposeProof` だけです。
`ReorderPremises` は enum に存在しても MVP では拒否します。
`PayloadNode` step の唯一の binding が指す payload node について、validator は次を確認します。

```text
- payload node の rule_fingerprint が step.rule.rule_fingerprint と一致する
- payload node の conclusion_encoding が持つ core_expr が step.conclusion と definitional equality で一致する
- payload node の conclusion_encoding が持つ encoded_expr が、同じ encoder_version / logic / command_profile での
  deterministic encoder 再実行結果と canonical bytes 一致する
- payload node の premises 長が step.premises 長と一致する
- payload node premises[i] は step.premises[i] が指す prior step の唯一の PayloadNode binding の node_id と一致する
```

payload node premise に local bookkeeping step を直接対応させることは MVP では許可しません。
payload proof を local bookkeeping で変形する場合は、payload node step の後に別の `LocalBookkeeping` step を置きます。
`ReorderPremises` は future extension 用の予約 variant です。
MVP では proof-producing structural combinator を hidden builtin として持たないため、`ReorderPremises` は常に `UnsupportedFeature` として拒否します。
premise order の変換が必要な場合は、明示 imported combinator を使う `ComposeProof` として表します。
`IntroduceTheoryLemma.lemma` と `ComposeProof.combinator` は `imported_theory_refs` に含まれ、かつ envelope imports から
一意に解決できなければなりません。
`level_args` / `term_args` はその imported declaration の public interface に対する明示 instantiation です。
引数探索、implicit insertion、typeclass search は local bookkeeping の中で実行しません。
validator は `SmtReconstructionRule`、`payload_bindings`、premise step の `conclusion`、`imported_theory_refs`
から step の proof term を決定的に再構成します。再構成された canonical `CoreExpr` bytes が `step.proof` と
一致しなければ、その step は拒否します。`step.proof` を caller-provided trusted proof として扱ってはいけません。
local bookkeeping の再構成は次に限定します。

```text
ReorderPremises:
  MVP rejects this variant as UnsupportedFeature

IntroduceTheoryLemma:
  lemma に level_args / term_args をその順序で適用した proof term を作る

ComposeProof:
  combinator に level_args / term_args と premise proof を premises の配列順で適用する
```

この規則だけで `step.conclusion` の proof term を一意に構成できない場合は `UnsupportedFeature` として拒否します。

AI が使ってよい用途:

```text
- SMT に渡す前の theory selection
- encoding の候補作成
- failed proof の原因分類
- reconstruction_plan の候補作成
```

AI が使ってはいけない用途:

```text
- unsat / sat の trusted 判定
- unsat core だけを proof として採用する
- solver log を certificate として採用する
- reconstruction failure を自然文 explanation で上書きする
```

採用条件:

```text
- target.goal_fingerprint が payload.goal から再計算できる
- encoded_problem.problem_hash / encoding_hash が inline bytes または artifact bytes から再計算できる
- encoded_problem 内の goal_fingerprint / logic が envelope target / payload logic と一致する
- encoded_problem commands が command_profile の canonical phase order を満たす
- proof_payload.payload_hash が inline bytes または artifact bytes から再計算できる
- Artifact の file_hash / size_bytes が実ファイル bytes と一致する
- proof_payload bytes が certificate_format ごとの canonical SMT proof node table として decode できる
- 各 step の payload_bindings が proof_payload 内の node に解決でき、rule_fingerprint が payload node の rule と一致する
- SmtReconstructionRule と payload_bindings の rule_fingerprint / local bookkeeping 制約が一致する
- reconstruction_plan.steps が acyclic で、premises が先行 step_id だけを参照する
- 各 step の proof が rule validator の再構成結果と canonical bytes 一致する
- 各 step の proof が、payload.goal.universe_params / payload.goal.local_context の下でその step の conclusion の proof term として kernel check を通る
- final_step が steps 内に存在し、その conclusion が payload.goal.universe_params / payload.goal.local_context の下で payload.goal.target と definitional equality で一致する
- final_proof が final_step の proof と一致し、payload.goal.universe_params / payload.goal.local_context の下で payload.goal.target の proof term として kernel check を通る
- independent checker が resulting certificate を再検査できる
```

---

# 8. Theorem Graph AI

Theorem graph は検索・推薦・学習用の非信頼 index です。
graph の node / edge は verified certificate から抽出された identity に紐づけます。

```rust
struct MachineTheoremGraphNodeRef {
    module: ModuleName,
    name: GlobalName,
    export_hash: Hash256,
    decl_certificate_hash: Hash256,
    type_hash: Hash256,
    certificate_hash: Hash256,
    decl_interface_hash: Hash256,
}

struct MachineTheoremGraphQuery {
    env_fingerprint: Hash256,
    goal_fingerprint: Hash256,
    goal: Phase9AiGoal,
    snapshot: MachineTheoremGraphSnapshotRef,
    query_features: MachineTheoremGraphQueryFeaturesRef,
    ranking_profile: TheoremGraphRankingProfile,
    limit: u32,
}

struct MachineTheoremGraphSnapshotRef {
    source_release_hash: Hash256,
    extractor_version: TheoremGraphExtractorVersion,
    source: MachineTheoremGraphSnapshotSource,
}

enum MachineTheoremGraphSnapshotSource {
    Inline {
        graph_snapshot_hash: Hash256,
        canonical_bytes: Vec<u8>,
    },
    Artifact {
        path: ArtifactPath,
        file_hash: Hash256,
        graph_snapshot_hash: Hash256,
        size_bytes: u64,
    },
}

enum MachineTheoremGraphQueryFeaturesRef {
    Inline {
        query_features_hash: Hash256,
        canonical_bytes: Vec<u8>,
    },
    Artifact {
        path: ArtifactPath,
        file_hash: Hash256,
        query_features_hash: Hash256,
        size_bytes: u64,
    },
}

struct MachineTheoremGraphResult {
    entries: Vec<MachineTheoremGraphResultEntry>,
}

struct MachineTheoremGraphResultEntry {
    node: MachineTheoremGraphNodeRef,
    score: GraphScore,
}

struct GraphScore {
    score_microunits: i64,
}

enum TheoremGraphRankingProfile {
    MvpTupleOrder,
}

struct MachineTheoremGraphSnapshot {
    source_release_hash: Hash256,
    extractor_version: TheoremGraphExtractorVersion,
    nodes: Vec<MachineTheoremGraphNodeRef>,
    edges: Vec<MachineTheoremGraphEdge>,
}

struct MachineTheoremGraphEdge {
    from: MachineTheoremGraphNodeRef,
    to: MachineTheoremGraphNodeRef,
    kind: TheoremGraphEdgeKind,
}

struct MachineTheoremGraphQueryFeatures {
    env_fingerprint: Hash256,
    goal_fingerprint: Hash256,
    feature_schema_version: TheoremGraphFeatureSchemaVersion,
    features: Vec<MachineTheoremGraphFeature>,
}

struct MachineTheoremGraphFeature {
    key: TheoremGraphFeatureKey,
    value: TheoremGraphFeatureValue,
}

enum TheoremGraphFeatureValue {
    Bool(bool),
    I64(i64),
    Hash(Hash256),
}
```

`score` は対応する `node` にだけ結びつく非信頼 metadata です。
`score` は certificate に入りません。
`MachineTheoremGraphNodeRef` は Phase 5 / Phase 6 と同じ `module / name / export_hash / certificate_hash / decl_interface_hash`
identity を持つため、AI premise retrieval はここから `GlobalRef` を一意に作れます。
`decl_certificate_hash` と `type_hash` は identity ではなく、解決した export / declaration と一致することを確認する verification field です。
同じ identity tuple を持つ node が複数ある snapshot は、`decl_certificate_hash` / `type_hash` が違っていても重複として拒否します。
graph query の replay identity は `snapshot` と `query_features` を含みます。同じ goal でも、
`graph_snapshot_hash` または `query_features_hash` が違う場合は別 request として扱います。
`TheoremGraphQuery` task の envelope payload は `MachineTheoremGraphQuery` だけです。
`MachineTheoremGraphResult` は validator が query から決定的に計算して返す response であり、caller-provided payload ではありません。
precomputed graph result を検証する future endpoint を追加する場合は、query と result を両方持つ別 task kind を定義します。

```text
graph_snapshot_hash =
  sha256("npa.phase9_ai.theorem_graph.snapshot.v1" || canonical_bytes(MachineTheoremGraphSnapshot))

query_features_hash =
  sha256("npa.phase9_ai.theorem_graph.query_features.v1" || canonical_bytes(MachineTheoremGraphQueryFeatures))
```

`MachineTheoremGraphQuery.env_fingerprint` / `goal_fingerprint` は envelope の `target.env_fingerprint` /
`target.goal_fingerprint` と完全一致しなければなりません。
`MachineTheoremGraphQuery.goal` から再計算した `goal_fingerprint` も一致しなければなりません。
validator は snapshot bytes を `MachineTheoremGraphSnapshot` として canonical decode し、そこから `graph_snapshot_hash` を
再計算します。query feature bytes は `MachineTheoremGraphQueryFeatures` として canonical decode し、そこから
`query_features_hash` を再計算します。
snapshot bytes 内の source release / extractor version metadata は、`source_release_hash` と
`extractor_version` に一致しなければなりません。
query feature bytes 内の `env_fingerprint` / `goal_fingerprint` は request query と一致しなければなりません。
`MachineTheoremGraphFeature` は `key` の canonical bytes 昇順で strictly sorted され、重複 key を含んではいけません。
feature value は bool、signed 64-bit integer、hash digest だけを許可します。float、embedding vector、implementation-defined
rounding を canonical query feature に入れてはいけません。
snapshot の `nodes` は identity tuple
`module / name / export_hash / certificate_hash / decl_interface_hash` の辞書順で
strictly sorted かつ重複なしでなければなりません。
`decl_certificate_hash` と `type_hash` は sort key に含めません。
`edges` は `from identity tuple / to identity tuple / kind` の辞書順で strictly sorted かつ重複なしでなければなりません。
すべての edge の `from` / `to` は snapshot の `nodes` に exact canonical bytes で存在しなければなりません。
存在しない node を指す edge を含む snapshot は `PayloadHashMismatch` として拒否します。
graph result の各 `node` は snapshot 内に存在し、かつ envelope imports 内の export table から
`module / export_hash / certificate_hash / name / decl_interface_hash` で一意に解決できなければなりません。
validator は解決した Phase 2 export / declaration から `decl_certificate_hash` と `type_hash` を再計算し、
node の値と一致することを確認します。`type_hash` は Phase 2 `ExportEntry.type_hash` です。
`decl_certificate_hash` はその declaration の canonical certificate declaration hash です。
MVP の eligible node は、解決した `ExportEntry.kind` が theorem または axiom の public export であるものだけです。
definition、constructor、recursor、generated artifact を theorem graph query result として返してはいけません。
それらを rewrite candidate や constructor hint として検索したい場合は、別の task kind または ranking profile で
期待する `ExportEntry.kind` を明示してから有効化します。
snapshot に含まれるが envelope imports で解決できない node は、
MVP では result の eligible node にしません。外部 graph store lookup で missing node を補完してはいけません。
MVP の `ranking_profile = MvpTupleOrder` では、validated graph result の `score_microunits` はすべて `0` でなければならず、
result ordering は `module / name / export_hash / certificate_hash / decl_interface_hash` の tuple 辞書順に固定します。
この profile の result entries は、snapshot の sorted `nodes` から eligible node だけを残した列の先頭
`min(limit, eligible_nodes.len())` 件をそのまま返すものに限定します。
`limit` は `0 <= limit <= 256` の `u32` でなければなりません。
`limit = 0` の場合は empty result です。
`limit > 256` の query は implementation-specific resource guard に委ねず、wire validation で deterministic error として拒否します。
learned ranker や embedding score は sidecar に保存してよいですが、`MvpTupleOrder` の validated result には入れません。
`GraphScore` は signed 64-bit integer microunit だけを使います。float、NaN、infinity、implementation-defined rounding を
wire payload や canonical ordering に入れてはいけません。
AI premise retrieval が graph result を使う場合も、tactic candidate には `GlobalRef` と
`decl_interface_hash` を明示し、Phase 4 AI / Phase 5 AI の通常の検査を通します。

禁止事項:

```text
- graph に存在することを theorem existence の根拠にする
- score が高い theorem を型検査なしで採用する
- graph edge を import dependency として扱う
- AI annotation から decl_certificate_hash を作る
```

---

# 9. Natural Language Formalization AI

Natural language formalization は、自然言語 / LaTeX / コメントから形式命題候補を作る機能です。
AI formalizer の出力は、常に未検証候補として扱います。

```rust
struct MachineFormalizationCheckPayload {
    candidate: MachineFormalizationCandidate,
    intent_record: Option<FormalizationIntentRecord>,
}

struct MachineFormalizationCandidate {
    source_document: MachineFormalizationSourceDocumentRef,
    claim_span: MachineFormalizationClaimSpan,
    statement: MachineSurfaceTerm,
    optional_proof_candidate: Option<MachineFormalizationProofCandidate>,
}

struct MachineFormalizationProofCandidate {
    candidate_statement_hash: Hash256,
    tactic: MachineTacticCandidate,
}

enum MachineFormalizationSourceDocumentRef {
    Inline {
        source_document_hash: Hash256,
        raw_utf8_bytes: Vec<u8>,
    },
    Artifact {
        path: ArtifactPath,
        file_hash: Hash256,
        source_document_hash: Hash256,
        size_bytes: u64,
    },
}

struct MachineFormalizationClaimSpan {
    start_byte: u64,
    end_byte: u64,
    claim_span_hash: Hash256,
}

enum ReviewerId {
    Human {
        stable_id_ascii: Vec<u8>,
    },
    System {
        system_id_ascii: Vec<u8>,
        actor_id_ascii: Vec<u8>,
    },
}

struct FormalizationIntentRecord {
    source_document_hash: Hash256,
    claim_span_hash: Hash256,
    candidate_statement_hash: Hash256,
    status: FormalizationIntentStatus,
}

enum FormalizationIntentStatus {
    Unreviewed,
    Reviewed {
        reviewer: ReviewerId,
        accepted_statement_hash: Hash256,
    },
    Rejected {
        reviewer: ReviewerId,
        rejection_reason: MachineFormalizationRejectionReasonRef,
        rejection_reason_hash: Hash256,
    },
}

enum MachineFormalizationRejectionReasonRef {
    Inline {
        rejection_reason_hash: Hash256,
        raw_utf8_bytes: Vec<u8>,
    },
    Artifact {
        path: ArtifactPath,
        file_hash: Hash256,
        rejection_reason_hash: Hash256,
        size_bytes: u64,
    },
}
```

`NaturalLanguageFormalization` task の envelope payload は `MachineFormalizationCheckPayload` です。
AI が生成する部分は `candidate` であり、`intent_record` は人間レビューや監査用の review record です。
`intent_record = None` は candidate-only の未レビュー検査を意味します。
`intent_record` は certificate には入りませんが、`/machine/phase9/formalize/check` request の canonical payload には入ります。
したがって `intent_record` を追加・削除・更新すると envelope の `candidate_hash` は変わります。
未レビューの形式化候補そのものを追跡する identity には、`candidate_hash` ではなく
`source_document_hash` / `claim_span_hash` / `candidate_statement_hash` の組を使います。
formalization check payload の import closure は envelope の `imports` だけを authoritative にします。
`MachineFormalizationCandidate` は別の `imports` field を持ちません。

`ReviewerId` は reviewer DB lookup や表示名解決の key ではなく、request payload 内で完結する canonical identity です。
MVP では各 ASCII field は正規表現 `[A-Za-z0-9._@:-]{1,128}` に一致しなければなりません。
Unicode normalization、case folding、email alias 展開、外部 user directory lookup は行いません。
`ReviewerId::Human.stable_id_ascii`、`ReviewerId::System.system_id_ascii`、`ReviewerId::System.actor_id_ascii` は
canonical bytes の bytewise equality だけで比較します。
同じ人間や外部 system を別 field 値で表した場合、validator は同一 reviewer と推測してはいけません。

`source_document_hash` は raw UTF-8 document bytes から次で再計算します。

```text
source_document_hash =
  sha256("npa.phase9_ai.formalization.source_document.v1" || raw_utf8_document_bytes)
```

`Inline.raw_utf8_bytes` は document wrapper ではなく、文書そのものの raw UTF-8 bytes です。
`Artifact` の場合も file bytes を raw UTF-8 document bytes として扱います。
`claim_span` は UTF-8 source document bytes の byte range `[start_byte, end_byte)` です。
`claim_span_hash` は次で計算します。

```text
claim_span_hash =
  sha256(
    "npa.phase9_ai.formalization.claim_span.v1"
    || source_document_hash
    || start_byte as canonical u64
    || end_byte as canonical u64
    || source_document_bytes[start_byte..end_byte]
  )
```

`start_byte > end_byte`、document length を超える range、UTF-8 codepoint boundary でない range は拒否します。

`rejection_reason_hash` は raw UTF-8 reason bytes から次で再計算します。

```text
rejection_reason_hash =
  sha256("npa.phase9_ai.formalization.rejection_reason.v1" || raw_utf8_reason_bytes)
```

`MachineFormalizationRejectionReasonRef::Inline.raw_utf8_bytes` は wrapper ではなく理由本文そのものの raw UTF-8 bytes です。
`Artifact` の場合も file bytes を raw UTF-8 reason bytes として扱います。
`Artifact.file_hash` / `size_bytes` は実ファイル bytes と一致しなければなりません。
inline bytes または artifact bytes から再計算した hash は、
`MachineFormalizationRejectionReasonRef` 内の `rejection_reason_hash` と
`FormalizationIntentStatus::Rejected.rejection_reason_hash` の両方に一致しなければなりません。

`candidate_statement_hash` は、この candidate の `statement: MachineSurfaceTerm` の canonical bytes に
`"npa.phase9_ai.formalization.candidate_statement.v1"` tag を付けて hash した値です。
Phase 3 AI complete mode は、candidate statement から `accepted_universe_params` と accepted theorem type の
canonical `CoreExpr` を返します。
`accepted_statement_hash` は、`target.env_fingerprint`、`accepted_universe_params`、accepted theorem type の canonical `CoreExpr` bytes に
`"npa.phase9_ai.formalization.accepted_statement.v1"` tag を付けて hash した値です。
reviewed intent record を certificate に結びつける場合、certificate を作る request envelope の environment と certificate 内の theorem type から
同じ `accepted_statement_hash` を再計算できなければなりません。
同じ theorem type bytes でも import closure / options が違う場合は別の accepted statement として扱います。

`intent_record` が `Some` の場合、`source_document_hash` / `claim_span_hash` / `candidate_statement_hash` は
payload の `candidate` から再計算した値と一致しなければなりません。
validator は reviewer DB や外部監査 log を lookup して `intent_record` を補完してはいけません。
proof bridge で使う `accepted_statement_hash` の入力元は次で固定します。

```text
intent_record = None:
  elaborated candidate statement から accepted_statement_hash を再計算する。
  これは未レビュー proof candidate の scratch identity にだけ使い、verified mathematical intent とは呼ばない。

intent_record.status = Unreviewed:
  None と同じく elaborated candidate statement から accepted_statement_hash を再計算する。
  record は監査 sidecar であり、reviewed intent の根拠にはならない。

intent_record.status = Reviewed { accepted_statement_hash, ... }:
  record 内の accepted_statement_hash を proof bridge の accepted_statement_hash として使う。
  MVP では accepted statement body を別 field として持たないため、この hash が elaborated candidate statement から
  再計算した accepted_statement_hash と一致する場合だけ proof bridge / certificate binding を実行できる。
  一致しない場合は reviewed intent record の整合性だけを検査する intent-record-only result とし、
  accepted theorem type、formalization_proof_root_hash、proof check success、certificate binding を返してはいけない。

intent_record.status = Rejected { ... }:
  accepted_statement_hash は未定義であり、optional_proof_candidate は None でなければならない。
  validator は source_document_hash / claim_span_hash / candidate_statement_hash / rejection_reason_hash の
  canonical 整合性だけを検査し、candidate.statement を Phase 3 elaboration しない。
  candidate.statement は MachineSurfaceTerm として canonical decode できなければならず、
  candidate_statement_hash はその canonical bytes から再計算する。
  ただし rejected status ではその MachineSurfaceTerm に対して name resolution、implicit insertion、
  elaboration、type check、normalization を実行してはいけない。
  rejection_reason_hash は `rejection_reason` の inline bytes または artifact bytes から必ず再計算する。
  proof bridge は実行しない。
```

`optional_proof_candidate.candidate_statement_hash` は、この candidate の `statement` から再計算した
`candidate_statement_hash` と一致しなければなりません。
`optional_proof_candidate.tactic` は、その `statement` を Phase 3 AI complete mode で elaboration した
core theorem type に対してだけ検査します。
人間レビューで statement が編集され、`accepted_statement_hash` が elaborated candidate statement と一致しない場合、
この optional proof candidate は採用できません。
validator は `optional_proof_candidate = Some(...)` を silently ignore して成功にしてはいけません。
この場合は `FormalizationProofStatementMismatch` として拒否し、caller は `optional_proof_candidate = None` で
statement / intent だけを intent-record-only result として検査するか、
accepted statement を `MachineFormalizationCandidate.statement` にした別 request として作り直します。
`optional_proof_candidate = Some(...)` かつ `accepted_statement_hash` が elaborated candidate statement と一致する場合、
validator は必ず proof bridge を実行します。
proof bridge が失敗した場合は request を拒否し、`CandidateStatementChecked` として downgrade してはいけません。
`CandidateStatementChecked` を返せるのは `optional_proof_candidate = None` で、
かつ `intent_record = None` / `Unreviewed` / `Reviewed` with matching `accepted_statement_hash` の場合だけです。
`Reviewed` intent が candidate statement と異なる accepted statement を指し、
`optional_proof_candidate = None` の場合、validator は `IntentRecordOnly` を返せます。
`Rejected` intent の場合も、`optional_proof_candidate = None` であれば hash 整合性だけを検査する
`IntentRecordOnly` を返します。
`Rejected` intent に `optional_proof_candidate = Some(...)` が付いている場合は
`RejectedIntentHasProofCandidate` として拒否します。
ここでの一致判定は、elaborated candidate theorem type の canonical `CoreExpr` bytes に
elaborated candidate universe params と `target.env_fingerprint` を添え、
`"npa.phase9_ai.formalization.accepted_statement.v1"` tag を付けて hash した値と `accepted_statement_hash` の比較です。

```text
accepted_statement_hash =
  sha256(
    "npa.phase9_ai.formalization.accepted_statement.v1"
    || target.env_fingerprint digest bytes
    || canonical_bytes(accepted_universe_params)
    || canonical_bytes(accepted_theorem_type)
  )
```

proof bridge 用の scratch identity は proof candidate 自体ではなく、statement / accepted theorem / environment に束縛します。
`accepted_statement_hash` と `target.env_fingerprint` には imports と `options_hash` が含まれるため、同じ表層 statement でも
import closure や formalization options が違う場合は別 root になります。

`/machine/phase9/formalize/check` の success kind は次の3つだけです。

```text
CandidateStatementChecked:
  candidate.statement が Phase 3 AI complete mode で elaboration / type check された。
  optional_proof_candidate は None であり、intent_record は None / Unreviewed / Reviewed matching のいずれかである。
  Reviewed matching の場合、accepted_statement_hash は elaborated candidate statement から再計算した値と一致している。
  proof bridge は実行していない。

IntentRecordOnly:
  intent_record と candidate の hash 整合性、および intent status 判定に必要な candidate statement 比較だけを検査した。
  intent_record.status = Reviewed mismatch の場合、candidate.statement は accepted_statement_hash との比較に必要な範囲で
  elaboration / type check された。
  intent_record.status = Rejected の場合、candidate.statement は elaboration / type check されず、
  accepted_statement_hash は存在せず、candidate.statement の elaboration result を theorem identity として返してはいけない。
  accepted theorem type / proof root / certificate binding は返さない。

ProofBridgeChecked:
  accepted_statement_hash が elaborated candidate statement と一致し、
  optional_proof_candidate を Phase 4/7 で検査して proof bridge が成功した。
```

Reviewed intent が candidate statement と異なる accepted statement を指す場合、MVP で返せる success kind は
`IntentRecordOnly` だけです。
accepted statement 自体を検査・証明したい caller は、その accepted statement を新しい
`MachineFormalizationCandidate.statement` として送る必要があります。

```text
formalization_proof_root_hash =
  sha256(
    "npa.phase9_ai.formalization.proof_root.v1"
    || target.env_fingerprint digest bytes
    || candidate_statement_hash digest bytes
    || accepted_statement_hash digest bytes
  )
```

MVP の proof bridge は、elaborated candidate theorem type から Phase 4 AI の `start_machine_proof` を呼び、
初期 proof state を決定的に作ります。
Phase 9 validator が `MachineTacticEnv` を手で組み立てて `options_fingerprint` や resolved family を省略してはいけません。
`options.formalization` は `Some` でなければなりません。
`options.formalization.tactic_options` は Phase 4 `MachineTacticOptions` として canonicalize し、
その canonical bytes と hash が Phase 4 `MachineTacticEnv.options` / `options_fingerprint` に保存されます。
`eq_family = None` や `nat_family = None` の意味も Phase 4 の `start_machine_proof` 規則に従います。
つまり builtin Eq がある profile では、`eq_family = None` でも `MachineTacticEnv.eq_family` は
resolved builtin Eq family の `Some(...)` になりえます。

```text
MachineProofSpec:
  module = deterministic scratch module name derived from formalization_proof_root_hash
  theorem_name = deterministic scratch theorem name derived from formalization_proof_root_hash
  source_index = 0
  universe_params = elaborated candidate universe_params
  theorem_type = elaborated candidate theorem type

start_machine_proof inputs:
  imports = envelope.imports
  checked_current_decls = []
  options = options.formalization.tactic_options

initial goal:
  universe_params = elaborated candidate universe_params
  local_context = []
  target = elaborated candidate theorem type
```

scratch module / theorem name は proof candidate ではなく `formalization_proof_root_hash` だけから導出します。
同じ `target.env_fingerprint` / `candidate_statement_hash` / `accepted_statement_hash` で
`optional_proof_candidate.tactic` だけが違う場合、Phase 4 root module / theorem name は同一でなければなりません。
`optional_proof_candidate.tactic` はこの initial goal からだけ実行します。
別の proof state、IDE session、checked current declaration、追加 simp registry、または
`options.formalization.tactic_options` 以外の tactic options を参照する proof candidate は拒否します。
Phase 4 tactic が `start_machine_proof` で解決された `MachineTacticEnv` 以外の family / registry を必要とする場合は、
MVP formalization proof bridge では `UnsupportedFeature` として拒否します。

採用フロー:

```text
1. source document bytes と claim span range を hash 固定する
2. AI が Machine Surface statement 候補を出す
3. intent_record.status = Rejected の場合は candidate / intent hash 整合性だけを検査し、IntentRecordOnly で終了する
4. それ以外の場合は Phase 3 AI complete mode で canonicalize / elaborate / type check する
5. 必要なら人間が intent を確認する
6. proof candidate がある場合は candidate_statement_hash 一致を確認し、elaborated candidate theorem type に対して Phase 4/7 で通常通り検査する
7. certificate には accepted core declaration と proof だけを入れる
```

`FormalizationIntentRecord` は、自然言語上の意図確認の監査 record です。
ここでの record は certificate 外の metadata という意味では sidecar ですが、
`/machine/phase9/formalize/check` の payload に含める場合は canonical request bytes と `candidate_hash` に入ります。
それ自体は theorem の正しさを保証しません。
`status = Unreviewed` の record は、well-typed な形式命題候補を保存するためだけに使えます。
`status = Reviewed` でなければ verified mathematical intent と呼んではいけません。

拒否するもの:

```text
- natural language explanation を theorem statement と同一視する
- LaTeX 表記を parse しただけで well-typed と扱う
- confidence score で採用可否を決める
- reviewer なしの formalization を verified mathematical intent と呼ぶ
```

---

# 10. API Surface

Phase 9 AI MVP の Machine API は、少なくとも次を提供します。

```text
POST /machine/phase9/inductive/check
POST /machine/phase9/universe/repair/check
POST /machine/phase9/typeclass/resolve
POST /machine/phase9/quotient/check
POST /machine/phase9/smt/reconstruct
POST /machine/phase9/theorem-graph/query
POST /machine/phase9/formalize/check
```

すべての endpoint は次を満たします。

```text
- request は canonicalizable な structured payload
- response は success / rejected / error を enum で返す
- pretty message は補助情報であり、判定には使わない
- same request hash と same referenced artifact bytes なら same validation result
- time / random seed / network result を validation result hash に入れない
```

`Artifact` 参照を使う request では、`path` だけを replay input と見なしてはいけません。
replay input は request canonical bytes と、`file_hash` / `size_bytes` に一致する artifact bytes の組です。
同じ request hash でも、path 上の現在の bytes が指定された `file_hash` / `size_bytes` と一致しない場合は deterministic な
`PayloadHashMismatch` として拒否します。

---

# 11. Error Model

AI repair loop のため、エラーは自然文ではなく enum 中心にします。

```rust
enum Phase9AiValidationError {
    EnvelopeMalformed,
    TargetFingerprintMismatch,
    ImportClosureMismatch,
    PayloadHashMismatch,
    KernelRejected,
    IndependentCheckerRejected,
    NonDeterministicResult,
    BudgetExceeded,
    AmbiguousResolution,
    UnsupportedFeature,
}
```

feature-specific error はこの enum の下にぶら下げます。
human-readable message は持ってよいですが、hash や replay 判定には使いません。

```rust
enum FormalizationError {
    IntentRecordMismatch,
    FormalizationProofStatementMismatch,
    RejectedIntentHasProofCandidate,
}
```

---

# 12. Security and Sandboxing

Phase 9 AI は外部 solver、embedding index、LLM、RAG store を扱うため、境界を固定します。

```text
kernel:
  no network
  no filesystem discovery
  no plugin loading
  no AI call
  no SMT solver process execution

orchestrator:
  may call AI / solver / graph index
  must serialize candidates before validation
  must pass explicit imports and options
  must not mutate trusted env during failed candidate validation

checker:
  reads only canonical certificate and declared imports
  ignores AI sidecars for pass/fail
```

AI sidecar は監査・学習・デバッグ用です。
CI の pass/fail は Phase 8 の checker result と policy で決めます。

---

# 13. MVP Milestones

推奨実装順序:

```text
M1  common envelope / candidate hash / error model
M2  universe repair candidate validation
M3  advanced inductive proposal validation
M4  theorem graph query with certificate-bound node refs
M5  typeclass resolution plan replay
M6  quotient construction candidate validation
M7  SMT reconstruction candidate validation
M8  natural language formalization statement / proof bridge check
M9  Phase 8 audit integration for all AI sidecars
```

MVP では、AI モデルがなくても deterministic fixtures で全 endpoint を検査できるようにします。
LLM や embedding retriever は、その後で差し替え可能な caller として接続します。

---

# 14. 完了条件

Phase 9 AI Profile が完了したと言える条件:

```text
- すべての Phase 9 AI candidate が canonical hash を持つ
- AI trace / score / prompt が certificate hash に入らない
- advanced inductive は AI 生成 recursor を採用しない
- universe repair は deterministic solver の結果だけを採用する
- typeclass resolution は ambiguity を score で解決しない
- quotient construction は equivalence / compatibility proof を kernel check する
- SMT result は reconstruction proof term なしで成功扱いしない
- theorem graph result は snapshot 固定された certificate-bound node ref だけを返す
- natural language formalization は Machine Surface statement と intent review state を分ける
- Phase 8 independent checker が AI sidecar なしで pass/fail を決められる
```

Phase 9 AI Profile は、**高度な自動化・検索・形式化を AI に開放しつつ、
AI を trusted base に入れず、すべてを canonical certificate の検査へ戻すための Machine Profile** です。
