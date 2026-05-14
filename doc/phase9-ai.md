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
    options_hash: Hash256,
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
- source span
- natural language explanation
- pretty printed theorem statement
```

これらは必要なら sidecar に保存できますが、certificate identity には影響させません。

---

# 3. Advanced Inductive AI

AI は indexed / mutual / nested inductive の宣言候補を出してよいですが、
recursor や computation rule を任意に供給してはいけません。

```rust
struct MachineInductiveProposal {
    block_name: NameId,
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
    target_expr: CoreExpr,
    explicit_level_args: Vec<LevelExpr>,
    proposed_constraints: Vec<UniverseConstraint>,
    minimization_hint: Option<UniverseMinimizationHint>,
}
```

`minimization_hint` は探索順序のヒントだけです。
採用される level assignment は、canonical universe solver の出力でなければなりません。

拒否する例:

```text
- undeclared universe parameter を参照する
- constraint graph に cycle がある
- cumulativity を使って forbidden coercion を通す
- pretty name だけで level を指定する
- target env_fingerprint と違う環境の repair を再利用する
```

AI repair loop には構造化エラーを返します。

```rust
enum UniverseRepairError {
    UnknownUniverseParam,
    IllFormedLevelExpr,
    UnsatisfiedConstraint,
    NonCanonicalSolution,
    TargetFingerprintMismatch,
}
```

---

# 5. Typeclass AI

Typeclass search は core calculus に入りません。
AI は instance search の候補順や resolution plan を提案できますが、最終的な証明は
elaborated core term として kernel が検査します。

```rust
struct MachineTypeclassResolutionPlan {
    class_goal: CoreExpr,
    ordered_candidates: Vec<MachineInstanceCandidateRef>,
    max_depth: u32,
    max_nodes: u32,
}

struct MachineInstanceCandidateRef {
    name: GlobalName,
    decl_interface_hash: Hash256,
    priority_hint: Option<i32>,
}
```

`priority_hint` は candidate hash には含めますが、正しさの根拠ではありません。
`ordered_candidates`、`priority_hint`、`max_depth`、`max_nodes` は executable search plan の一部です。
同じ `class_goal` と import closure でも、AI が違う順序を提案した場合は別 candidate として扱います。

replay invariant は次です。

```text
same candidate_hash
  = same class_goal
  + same import closure
  + same options_hash
  + same ordered_candidates
  + same priority_hint values
  + same budget
  => same resolution result
```

candidate hash が違う resolution plan 同士に、同じ result を要求しません。

採用条件:

```text
- candidate ref が verified import または checked current decl に存在する
- class_goal が well-typed
- search budget 内で一意な solution が得られる
- elaborated instance term が kernel check を通る
- ambiguity がある場合は拒否する
```

拒否するもの:

```text
- AI が選んだ instance を kernel check なしで採用する
- hidden global environment から instance を読む
- import closure 外の instance を暗黙に追加する
- ambiguity を score で解決する
```

---

# 6. Quotient AI

Quotient は trusted base を広げやすい機能です。
AI は quotient construction の補助をしてよいですが、同値関係や lift の well-definedness は
通常の proof obligation として検査します。

```rust
struct MachineQuotientConstructionCandidate {
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

採用条件:

```text
- relation が carrier 上の relation として well-typed
- equivalence_proof が reflexive / symmetric / transitive を証明している
- quotient primitive の intro / elim / soundness rule だけを使う
- operation ごとの compatibility_proof が kernel check を通る
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
    problem_hash: Hash256,
    logic: SmtLogic,
    encoding_hash: Hash256,
    certificate_format: SmtCertificateFormat,
    proof_payload: MachineSmtProofPayloadRef,
    reconstruction_plan: MachineSmtReconstructionPlan,
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
    imported_theory_lemmas: Vec<GlobalRef>,
    generated_core_steps: Vec<CoreExpr>,
}
```

`proof_payload` は replay input の一部です。
validator は `Inline.canonical_bytes` または `Artifact` の `path` / `file_hash` で固定された bytes から
`payload_hash` を再計算します。filesystem discovery や solver log lookup で payload を補完してはいけません。

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
- problem_hash と encoding_hash が現在の goal から再計算できる
- proof_payload.payload_hash が inline bytes または artifact bytes から再計算できる
- Artifact の file_hash / size_bytes が実ファイル bytes と一致する
- reconstruction_plan から生成した proof term が kernel check を通る
- independent checker が resulting certificate を再検査できる
```

---

# 8. Theorem Graph AI

Theorem graph は検索・推薦・学習用の非信頼 index です。
graph の node / edge は verified certificate から抽出された identity に紐づけます。

```rust
struct MachineTheoremGraphNodeRef {
    name: GlobalName,
    declaration_hash: Hash256,
    type_hash: Hash256,
    certificate_hash: Hash256,
}

struct MachineTheoremGraphQuery {
    env_fingerprint: Hash256,
    goal_fingerprint: Hash256,
    query_features_hash: Hash256,
    limit: u32,
}

struct MachineTheoremGraphResult {
    entries: Vec<MachineTheoremGraphResultEntry>,
}

struct MachineTheoremGraphResultEntry {
    node: MachineTheoremGraphNodeRef,
    score: GraphScore,
}
```

`score` は対応する `node` にだけ結びつく非信頼 metadata です。
`score` は certificate に入りません。
AI premise retrieval が graph result を使う場合も、tactic candidate には `GlobalRef` と
`decl_interface_hash` を明示し、Phase 4 AI / Phase 5 AI の通常の検査を通します。

禁止事項:

```text
- graph に存在することを theorem existence の根拠にする
- score が高い theorem を型検査なしで採用する
- graph edge を import dependency として扱う
- AI annotation から declaration_hash を作る
```

---

# 9. Natural Language Formalization AI

Natural language formalization は、自然言語 / LaTeX / コメントから形式命題候補を作る機能です。
AI formalizer の出力は、常に未検証候補として扱います。

```rust
struct MachineFormalizationCandidate {
    source_document_hash: Hash256,
    claim_span_hash: Hash256,
    imports: Vec<VerifiedImportRef>,
    statement: MachineSurfaceTerm,
    optional_proof_candidate: Option<MachineTacticCandidate>,
}

struct FormalizationIntentRecord {
    source_document_hash: Hash256,
    claim_span_hash: Hash256,
    accepted_statement_hash: Hash256,
    reviewer: Option<ReviewerId>,
}
```

採用フロー:

```text
1. source document / claim span を hash 固定する
2. AI が Machine Surface statement 候補を出す
3. Phase 3 AI complete mode で canonicalize / elaborate / type check する
4. 必要なら人間が intent を確認する
5. proof candidate がある場合は Phase 4/7 で通常通り検査する
6. certificate には accepted core declaration と proof だけを入れる
```

`FormalizationIntentRecord` は、自然言語上の意図確認の監査 sidecar です。
それ自体は theorem の正しさを保証しません。

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
- same request hash なら same validation result
- time / random seed / network result を validation result hash に入れない
```

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
M2  theorem graph query with certificate-bound node refs
M3  universe repair candidate validation
M4  typeclass resolution plan replay
M5  natural language formalization statement check
M6  advanced inductive proposal validation
M7  SMT reconstruction candidate validation
M8  quotient construction candidate validation
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
- theorem graph result は certificate-bound node ref だけを返す
- natural language formalization は Machine Surface statement と intent record を分ける
- Phase 8 independent checker が AI sidecar なしで pass/fail を決められる
```

Phase 9 AI Profile は、**高度な自動化・検索・形式化を AI に開放しつつ、
AI を trusted base に入れず、すべてを canonical certificate の検査へ戻すための Machine Profile** です。
