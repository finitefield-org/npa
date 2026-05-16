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
enum Phase9AiProfileVersion {
    MvpV1,
}

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

type Phase9Name = Name;
type NameId = Phase9Name;
type GlobalName = Phase9Name;
type NamePrefix = Option<Phase9Name>;
type SortExpr = LevelExpr;

struct MachineSurfaceTerm {
    universe_params: Vec<UniverseParam>,
    term_canonical_bytes: Vec<u8>,
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

enum Phase9AiOptionsVersion {
    MvpV1,
}

struct Phase9AiOptions {
    schema_version: Phase9AiOptionsVersion,
    independent_checker: Phase9IndependentCheckerOptions,
    advanced_inductive: Phase9AdvancedInductiveOptions,
    typeclass: Phase9TypeclassOptions,
    quotient: Option<Phase9QuotientOptions>,
    smt: Option<Phase9SmtOptions>,
    formalization: Option<Phase9FormalizationOptions>,
}

struct Phase9IndependentCheckerOptions {
    profile: Phase9IndependentCheckerProfile,
}

enum Phase9IndependentCheckerProfile {
    Phase8MvpReference,
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
    tactic_budget: TacticBudget,
}
```

`Phase9AiCandidateEnvelope<T>` は task payload を decode した後の semantic view です。
wire-level canonical envelope では、`payload` field は `task_kind` で選ばれる task-specific payload bytes を
length-prefixed opaque bytes として保持します。
validator は common envelope validation step 0 で、この `payload` field の byte range までを一意に確定できれば
task-specific payload を full decode する前に `candidate_hash` を計算できます。
task-specific payload bytes の nested decode、semantic validation、protocol cap、sort order、duplicate check は
各 task の feature-specific validation で行います。
payload bytes を task-specific schema として decode できた場合、decode した typed value を再 serialize した canonical bytes は
wire envelope 内の payload bytes と byte-for-byte に一致しなければなりません。
一致しない、または task-specific schema として decode できない場合は、top-level envelope が decode 済みで
`candidate_hash` を計算できる限り deterministic `Rejected { error = EnvelopeMalformed, ... }` です。
validator は typed payload value から envelope 全体を再 serialize して `candidate_hash` を作ってはいけません。

`Phase9AiProfileVersion`、`Phase9AiOptionsVersion`、`Phase9IndependentCheckerProfile` の canonical bytes は variant tag だけで固定します。
MVP では `Phase9AiProfileVersion` と `Phase9AiOptionsVersion` はどちらも `MvpV1` だけを受け付けます。
`Phase9IndependentCheckerProfile` は MVP では `Phase8MvpReference` だけを受け付けます。
これは Phase 9 success 前に実行する Phase 8 independent checker policy/profile selection を replay input に束縛する field です。
validator は runtime configuration、environment variable、CI policy default、または caller session から checker profile を補完してはいけません。
将来 checker profile を増やす場合は既存 variant tag を変えず、`Phase9IndependentCheckerProfile` 末尾へ追加するか、
別 options schema version を定義します。

MVP の `Phase8MvpReference` checker support matrix は次で固定します。

```text
AdvancedInductive:
  この文書の MVP AdvancedInductive validation を通過し、
  Phase 2 canonical artifact generator / verifier invariant で構成された
  single-declaration inductive certificate package をサポートする。

QuotientConstruction:
  `quotient_v1` feature report を持つ quotient type `DefDecl` certificate package をサポートする。
  `quotient_v1` primitive refs / `eq` public interface mismatch は checker support ではなく
  QuotientConstruction feature-specific validation error として扱う。

SmtCertificate:
  `SmtRuleRegistryProfile::MvpEmptyRegistryV1` では SMT success path が存在しないため、
  Phase 8 checker support check には到達しない。
  future schema / profile で非空 solver-native registry を有効化する場合は、
  その registry profile に対応する checker support entry をこの表へ追加する。

NaturalLanguageFormalization::ProofBridgeChecked:
  Phase 4 proof bridge が生成した ordinary core theorem certificate package をサポートする。
  optional proof candidate や tactic execution log は checker support feature ではなく、
  Phase 4 / Phase 9 validation の入力である。
```

この support matrix は `options.independent_checker.profile` の variant tag が選択する固定仕様です。
profile variant tag は options bytes に含まれる replay input であり、
validator はこれと異なる local allowlist / CI policy / installed checker capability で support 判定を上書きしてはいけません。
未知の version tag は top-level envelope または options bytes の canonical decode failure として扱い、
top-level envelope の場合は `Error::NonCanonicalRequestBytes`、options bytes の場合は deterministic
`Rejected { error = EnvelopeMalformed, feature_error = None }` です。
`Phase9Name` は Phase 2 の canonical `Name` component list です。
この文書の Phase 9 wire schema で unqualified `NameId` と書く field はすべて `Phase9Name` の別名であり、
Phase 2 certificate-local `name_table` index ではありません。
`GlobalName` も `Phase9Name` です。
`NamePrefix` は public name prefix 用の optional `Phase9Name` です。
`NamePrefix = None` は prefix なしを表し、`Some(prefix)` の `prefix` は通常の non-empty `Phase9Name` でなければなりません。
空 component list を `Phase9Name` として encode することはできず、prefix なしを raw empty name で表した request は
`Rejected { error = EnvelopeMalformed, feature_error = None }`
です。
`SortExpr` は Phase 1 / Phase 2 の `LevelExpr` の別名であり、core `Sort(level)` の `level` だけを表します。
`SortExpr` は term-level `CoreExpr` ではないため、`GlobalRef`、`BVar`、`App` などの term node を持ちません。
この文書で `result_sort = Prop` と書く場合は `Sort(0)` ではなく、その sort level `0` を持つ `SortExpr` を意味します。
`MachineSurfaceTerm` は raw source string ではなく、Phase 3 AI の `Machine Surface term-source canonical bytes`
を埋め込む Phase 9 wrapper です。
`term_canonical_bytes` は Phase 3 の `"npa.phase3.machine-term-source.v1"` tag を持つ canonical parsed term AST bytes でなければなりません。
`MachineSurfaceTerm` の canonical bytes は `universe_params`、`term_canonical_bytes` の field order で固定します。
`universe_params` は statement term の free universe parameter context であり、binding order のまま保存し、
同じ `UniverseParam` を重複して含んではいけません。
`MachineSurfaceTerm.universe_params` 内の `UniverseParam` は Phase 2 `Name` と同じ component-list encoding を使いますが、
Phase 3 AI の universe parameter identifier と byte-for-byte に対応させるため、
exactly one UTF-8 component を持つ name だけを許可します。
この単一 component の raw UTF-8 bytes を Phase 3 AI `MachineTermElabContext.universe_params: Vec<String>` の各 entry として使い、
Phase 2 core `LevelExpr::Param` へ lowering する場合も同じ単一 component name を deterministic に intern します。
この単一 component への total mapping を以下 `phase3_universe_param_ident` と呼びます。
`MachineSurfaceTerm.universe_params` 内の multi-component `UniverseParam`、空 component、
または Phase 3 AI の universe parameter identifier として decode できない byte sequence は
`Rejected { error = EnvelopeMalformed, feature_error = None }`
です。
`term_canonical_bytes` が Phase 3 Machine Surface term AST として canonical decode できない場合、
または `universe_params` が重複する場合は
`Rejected { error = EnvelopeMalformed, feature_error = None }`
です。
Phase 9 validator は raw source text、pretty printed theorem statement、diagnostic source span から
`MachineSurfaceTerm` を再構成してはいけません。
Phase 5 wire の dotted `FullyQualifiedName` 文字列や、imported certificate-local `NameId` index を入れてはいけません。
Phase 9 wire payload 内の `CoreExpr` は Phase 1 / Phase 2 の core `Expr` と同じ AST shape を使いますが、
raw `TermId` table や certificate-local `NameId` index は持ちません。
`CoreExpr` 内の imported reference は、canonical sort 後の envelope `imports` に対する `import_index`、
`GlobalName` の canonical bytes、`decl_interface_hash` で encode します。
したがって `candidate_hash`、`goal_fingerprint`、および payload-local hash は `GlobalName` bytes に依存し、
どの certificate の `name_table` index にも依存しません。
Phase 2 certificate や kernel environment へ渡す直前だけ、validator は `GlobalName` を対象 output certificate の
name table に決定的に intern し、その `NameId` を持つ `GlobalRef::Imported` へ lowering します。
この lowering では import export block の entry を decoded `Name` bytes と `decl_interface_hash` で照合し、
imported certificate-local `NameId` の raw 数値を Phase 9 payload identity として使ってはいけません。
Phase 9 wire payload が raw certificate-local numeric `NameId` や `TermId` を含む場合は canonical schema 違反として
`Rejected { error = EnvelopeMalformed, feature_error = None }`
です。
Phase 9 wire `CoreExpr` の imported ref resolution は task 共通の検査です。
validator は kernel type check / definitional equality / proof check へ進む前に、対象 `CoreExpr` 群の全
`Const(GlobalRef::Imported(import_index, name, decl_interface_hash), level_args)` を canonical traversal order で走査します。
`import_index >= envelope.imports.len()`、またはその import の export table に
`(name, decl_interface_hash)` が一意に存在しない場合は
`Rejected { error = ImportClosureMismatch, feature_error = None }`
として拒否します。
この imported ref resolution は、byte-level hash / binding check と wire shape / universe scope check が成功した後、
kernel check より前に実行します。
同じ request に imported ref resolution failure と kernel ill-typedness が同時にある場合は
`ImportClosureMismatch` が `KernelRejected` より優先です。

`candidate_hash` は、provided wire-level envelope の canonical bytes から計算します。
ここでの canonical bytes には top-level field order と length-prefixed payload bytes を含め、
payload typed value の reserialization 結果は使いません。

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
MVP の options bytes cap は、Inline / Artifact とも raw canonical bytes `<= 16_000_000` です。
Inline の場合は `canonical_bytes.len()` を outer framing decode より先に検査します。
Artifact の場合は declared `size_bytes` が cap を超える時点で file read / `file_hash` check へ進まず
`Rejected { error = EnvelopeMalformed, feature_error = None }`
として拒否します。
declared `size_bytes` が cap 内でも実ファイル bytes の長さや `file_hash` が一致しない場合は
`Rejected { error = PayloadHashMismatch, feature_error = None }`
です。
options bytes validation の deterministic order は次で固定します。

```text
0. Inline bytes acquisition and byte cap check, or Artifact path validation /
   declared size cap check / file read / file_hash / size_bytes check
1. Phase9AiOptions outer canonical framing / field order / scalar field decode, including
   approved_nested_type_constructors and class_declarations vector length prefix cap precheck
   before element decode / allocation
2. Phase9AiOptions full canonical decode and common options canonical shape checks
3. options_hash recomputation and comparison
4. common options feature-support checks that are not canonical shape checks
```

step 0 で options bytes cap を超える場合は
`Rejected { error = EnvelopeMalformed, feature_error = None }` です。
step 0 で `Artifact.file_hash` / `size_bytes` が一致しない場合は
`Rejected { error = PayloadHashMismatch, feature_error = None }` です。
path validation を通過した artifact bytes を取得できない場合は `Error::ArtifactUnavailable` です。
step 1 の outer canonical framing / cap precheck failure、または step 2 の canonical decode / common shape failure は
`Rejected { error = EnvelopeMalformed, feature_error = None }`
として拒否し、`options_hash` mismatch より優先します。
step 2 が成功した後に `options_hash` が一致しない場合は
`Rejected { error = PayloadHashMismatch, feature_error = None }` です。
step 4 は step 3 の `options_hash` 一致後にだけ実行します。
`approved_nested_type_constructors` の cap 内 non-empty list のような common options feature-support rejection と
`options_hash` mismatch が同じ request に含まれる場合は、`PayloadHashMismatch` が優先です。
step 2 の common options canonical shape checks は schema version、field order、canonical structural sort order、
duplicate、protocol cap、および task kind に依存しない scalar wire range validation だけです。
ここでいう scalar wire range validation は enum tag、canonical integer encoding、`Option` tag など
`Phase9AiOptions` を一意に decode するための範囲検査であり、nested task option が意味的に受け付ける値かどうかは含みません。
たとえば `options.formalization.tactic_options.max_simp_rewrite_steps = 0` は canonical `u64` としては decode 可能なので、
common options step 2 では拒否しません。
MVP profile が semantic に受け付けない well-formed common option 値の拒否は step 4 で行い、
task kind が要求する nested option の task-specific shape / semantic range validation は、
`options_hash` が一致した後の task options shape check で行います。
`options_hash` は次で固定します。

```text
options_hash =
  sha256("npa.phase9_ai.options.v1" || canonical_bytes(Phase9AiOptions))
```

`Inline.options_hash` または `Artifact.options_hash` が step 3 の再計算値と一致しない request は
`Rejected { error = PayloadHashMismatch, feature_error = None }`
として拒否します。
`Artifact.file_hash` / `size_bytes` が実ファイル bytes と一致しない場合も同じ分類です。
path validation を通過した後に artifact bytes を取得できず hash / size を検査できない場合だけ
`Error::ArtifactUnavailable` です。

`ArtifactPath` は Phase 8 の normal CLI / API artifact path と同じ workspace-root-relative UTF-8 path string です。
canonical bytes には request に現れた UTF-8 path bytes を length-prefixed でそのまま入れます。
validator は environment variable 展開、`~` 展開、case folding、symlink 解決結果による path string 正規化、
または platform path normalization をしてはいけません。
実ファイル I/O では workspace root に対して1回だけ解決します。
ただし Phase 8 と同じ inside-owning-root rule を適用し、解決時に既存 path component または既存 final path が
symlink により workspace root 外へ出る場合は、file bytes を読む前の path validation failure として扱います。
空 path、absolute path、NUL byte、`.` / `..` component、workspace root 外へ出る path shape は
`Rejected { error = EnvelopeMalformed, feature_error = None }`
として拒否します。
symlink escape もこの `EnvelopeMalformed` 分類に含め、`ArtifactUnavailable` にはしません。
path validation を通過した後に、file missing、permission denied、I/O error などで実ファイル bytes を取得できない場合だけ
`Error::ArtifactUnavailable` です。

`imports` は Phase 4 AI と同じく `(module, export_hash, certificate_hash)` の canonical order に sort 済みでなければなりません。
sort order violation は top-level envelope の canonical shape violation として
`Rejected { error = EnvelopeMalformed, feature_error = None }`
で拒否します。
同じ `(module, export_hash, certificate_hash)` を重複して含む request は `ImportClosureMismatch` として拒否します。
これは duplicate import を不足 import と同じ closure identity error として扱う意図的な分類であり、
`Phase9AiGlobalRef` list の duplicate と同じ options shape error ではありません。
ここでの `VerifiedImportRef` は、Phase 2 verifier / Phase 8 import-locked checker workflow が canonical certificate bytes から構成した
opaque verified value を意味します。
public wire adapter は caller が serialized `exports` / `certified_env_decls` を渡しただけの値を
`VerifiedImportRef` として信用してはいけません。
この文書の Phase 9 validator が受け取る canonical request envelope は、すでに
verifier-created `VerifiedImportRef` を含む `Phase9AiCandidateEnvelope<T>` です。
外部 API adapter が import certificate refs / artifact bytes を受け取る場合、その wrapper は
`Phase9AiCandidateEnvelope<T>` の外側の adapter schema であり、wrapper 自体の bytes は
`candidate_hash` / `validation_result_hash` の入力ではありません。
adapter は Phase 9 common envelope validation を呼ぶ前に各 certificate を Phase 2 verifier で検査し、
`module / export_hash / certificate_hash` と verifier 出力から `VerifiedImportRef` を再構成したうえで、
canonical `Phase9AiCandidateEnvelope<T>` を構成しなければなりません。
この envelope を構成できない段階、つまり certificate artifact の取得失敗、certificate decode / verification failure、
recomputed `export_hash` / `certificate_hash` mismatch、dependency closure unverifiable、
または verifier-created `VerifiedImportRef` を取得できない場合は、この文書の `Phase9AiEndpointResponse::Rejected`
を返してはいけません。
これらは adapter 層の error / rejection として扱うか、Phase 9 endpoint としては canonical envelope だけを受け付ける API にします。
Phase 9 `Rejected { error = ImportClosureMismatch, feature_error = None }` は、top-level envelope を canonical decode でき、
`candidate_hash` を計算できた後の duplicate import、import resolution failure、または closure identity mismatch だけに使います。
import certificate refs / artifact bytes そのものを Phase 9 replay input に含めたい future extension では、
それらを含む canonical wrapper schema と `candidate_hash` binding をこの節で先に定義してから有効化しなければなりません。
in-process API では `VerifiedImportRef` の constructor を verifier/session owned にし、Phase 9 validator が raw export block や
caller-provided `certified_env_decls` を検証済みとして扱えない境界にしなければなりません。

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

`target.env_fingerprint` が再計算値と一致しない request は
`Rejected { error = TargetFingerprintMismatch, feature_error = None }`
として拒否します。
`UniverseRepair(TargetFingerprintMismatch)` は `env_fingerprint` mismatch には使わず、UniverseRepair task-local の
`goal_fingerprint` / `target_expr` 束縛 mismatch だけに使います。
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
ここでの `CoreExpr` は Phase 1 / Phase 2 の core `Expr` と同じ AST shape を指す Phase 9 wire alias です。
したがって binder name は `CoreExpr` の canonical bytes に含めず、bound variable は `BVar(index)` で表します。
ただし imported global name は Phase 2 certificate-local `NameId` ではなく、上で定義した `GlobalName` bytes で encode します。
`MachineLocalDecl.ty` / `MachineLocalDecl.value` も同じ core expression canonical bytes として扱います。
`local_context` と `target` に現れる `LevelExpr` は、`Const(imported_ref, level_args)` の explicit level argument として現れる場合も含め、
`Phase9AiGoal.universe_params` に含まれる parameter だけを free universe parameter として参照できます。
imported declaration 側の universe parameter は public interface の level binder order と arity を決めるだけで、
goal 内 `LevelExpr` の free universe parameter scope には入りません。
`local_context` は Phase 4 と同じ context order で保存し、de Bruijn index `0` は `local_context` の最後の binder を指します。
`local_context[i].ty` は `local_context[..i]` の下で well-typed、`value` がある場合も同じ prefix context の下で
`ty` の term として well-typed でなければなりません。`target` は full `local_context` の下で well-typed でなければなりません。
`goal_fingerprint` が一致した後、kernel well-typedness check の前に `Phase9AiGoal` の wire shape /
universe context shape と imported ref resolution を検査します。
`universe_params` が同じ `UniverseParam` を重複して含む場合、または `local_context` / `target` 内の `LevelExpr` が
`Phase9AiGoal.universe_params` に束縛されない universe parameter を参照する場合は
`Rejected { error = EnvelopeMalformed, feature_error = None }`
として拒否します。
同じ shape check では、`local_context[*].ty` / `local_context[*].value` / `target` の全 `CoreExpr` を
canonical order で走査し、`GlobalRef::Local(_)` または `GlobalRef::LocalGenerated` を含む request も
`Rejected { error = EnvelopeMalformed, feature_error = None }`
として拒否します。
これは common `Phase9AiGoal` wire shape violation であり、`ImportClosureMismatch`、`KernelRejected`、
または feature-specific `TargetRefMismatch` には分類しません。
同じ request に `goal_fingerprint` mismatch とこの wire shape / universe context shape violation が同時にある場合は、byte-level binding check である
`TargetFingerprintMismatch` が優先です。
wire shape / universe context shape が通った後、common Phase 9 wire `CoreExpr` imported ref resolution rule に従って、
`local_context[*].ty` / `local_context[*].value` / `target` 内の imported global reference をすべて解決します。
解決できない imported ref は
`Rejected { error = ImportClosureMismatch, feature_error = None }`
として拒否し、`Phase9AiGoal` well-typedness check には進みません。

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
一意に存在しない場合、または `import_index` が canonical sort 後の envelope `imports` 範囲外の場合は、
上の common imported ref resolution rule により
`Rejected { error = ImportClosureMismatch, feature_error = None }`
として拒否します。
Phase 9 AI goal payload では `GlobalRef::Local` / `GlobalRef::LocalGenerated` を使いません。
出現した場合は上の `Phase9AiGoal` wire shape check で
`Rejected { error = EnvelopeMalformed, feature_error = None }`
として拒否します。
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

これらから再計算した値が `target.goal_fingerprint` と一致しない request は次の response で拒否します。
`task_kind = UniverseRepair` の goal_fingerprint mode では
`Rejected { error = TargetFingerprintMismatch, feature_error = Some(UniverseRepair(TargetFingerprintMismatch)) }`
を返します。
その他の task では
`Rejected { error = TargetFingerprintMismatch, feature_error = None }`
です。
`goal_fingerprint` binding check は `Phase9AiGoal` の canonical bytes に対する byte-level check であり、
goal の kernel well-typedness check より先に行います。
同じ request で `goal_fingerprint` mismatch と ill-typed goal が同時に存在する場合は、上の
`TargetFingerprintMismatch` が優先です。
ただしこの check は wire-level envelope の step 0 で opaque payload を解釈するものではありません。
各 task の feature-specific validation 順序で payload outer canonical decode / protocol cap precheck を終え、
`Phase9AiGoal` の canonical bytes が一意に得られた後に実行します。
payload outer decode failure、task-local protocol cap violation、またはその task の順序で
goal binding より前に置かれた payload structural violation は、`TargetFingerprintMismatch` より優先します。
`goal_fingerprint` が一致した後に、`local_context[i].ty` / `local_context[i].value` / `target` の
well-typedness を kernel で検査します。
この well-typedness check が失敗した場合は
`Rejected { error = KernelRejected, feature_error = None }`
として拒否します。

`Phase9AiGlobalRef` の canonical bytes field order と、これを list 内で sort する場合の tuple key は次で固定します。

```text
module
export_hash digest bytes
certificate_hash digest bytes
name
decl_interface_hash digest bytes
```

現在の task の validation で意味的に使用する `Phase9AiGlobalRef` は envelope の `imports` 内の export table から
`module / export_hash / certificate_hash / name / decl_interface_hash` で一意に解決できなければなりません。
解決後に core term へ埋め込むときは、canonical sort 後の envelope `imports` 内の 0-based index を使って
Phase 9 wire `GlobalRef::Imported(import_index, GlobalName, decl_interface_hash)` を作ります。
Phase 2 certificate bytes へ lowering する場合だけ、上で述べた deterministic name table intern によって
`GlobalName` を Phase 2 `NameId` へ変換します。

```text
resolve_imported_ref(ref) =
  Phase9WireImportedRef(import_index(ref), ref.name, ref.decl_interface_hash)
```

`resolve_imported_ref` は Phase 9 AI 全体の共通 helper です。
`import_index(ref)` は、`ref.module / ref.export_hash / ref.certificate_hash` と一致する envelope import の canonical sort 後 index です。
validator は import closure 外の global environment、標準ライブラリの hidden registry、現在編集中 declaration から
`Phase9AiGlobalRef` を補完してはいけません。
解決対象の `Phase9AiGlobalRef` の `module / export_hash / certificate_hash` が envelope imports に存在しない場合、
または `name / decl_interface_hash` がその export table で一意に解決できない場合は `ImportClosureMismatch` として拒否します。
ref は一意に解決できたが、その feature が期待する role、public type、primitive interface、task-local target と合わない場合は
`ImportClosureMismatch` ではなく feature-specific error として拒否します。
たとえば Advanced Inductive の task-local target mismatch は
`Rejected { error = FeatureRejected, feature_error = Some(AdvancedInductive(TargetRefMismatch)) }` です。
現在生成中の declaration は `Phase9AiGlobalRef` では参照せず、`target_decl_hash`、payload-local `expected_decl_hash`、
または payload 内の明示的な local binder で束縛します。
`approved_nested_type_constructors` と `class_declarations` は上の `Phase9AiGlobalRef` tuple key で strictly sorted され、
重複を含んではいけません。
この tuple key 上の sort order violation または duplicate は common options canonical shape violation として
`Rejected { error = EnvelopeMalformed, feature_error = None }`
で拒否し、feature-specific validation へ進みません。
ただし import 解決と public interface 検査を行うのは、現在の task の feature-specific validation が
意味的に参照する list / primitive ref だけです。
MVP v1 の `approved_nested_type_constructors` は、task kind に関係なく空でなければならない list です。
ただし `Phase9AdvancedInductiveOptions.approved_nested_type_constructors.len() <= 65_536` は common options shape cap です。
task kind に関係なく、この cap を超える options は import 解決前に
`Rejected { error = EnvelopeMalformed, feature_error = None }`
として拒否します。
validator は `approved_nested_type_constructors` の vector length prefix を element decode / allocation より前にこの cap と照合しなければなりません。
この照合は options bytes validation order の step 1 に含まれる outer framing precheck であり、
cap 超過時に `approved_nested_type_constructors` の element を decode したり allocation したりしてはいけません。
この list は canonical decode、sort order、重複検査の後、import 解決前に
`Rejected { error = UnsupportedFeature, feature_error = Some(AdvancedInductive(PositivityProfileUnsupported)) }`
として拒否します。
この non-empty list rejection は options bytes validation order の step 4 であり、`options_hash` 一致後にだけ実行します。
したがって cap 内の non-empty list は full canonical decode / sort order / duplicate 検査後に
`UnsupportedFeature` として拒否しますが、cap 超過は list element の shape や sort order より先に
`EnvelopeMalformed` として拒否します。
この common options rejection は task kind に関係なく同じ分類を使います。
ここでの `AdvancedInductive` feature error は、この options field の所有 feature namespace を示すものであり、
endpoint task kind が AdvancedInductive であることを意味しません。
この規則は `class_declarations` には適用しません。`class_declarations` は TypeclassResolution task で意味的に使う list であり、
その task では import 解決と class public interface 検査の対象です。
非 TypeclassResolution task に non-empty `class_declarations` が含まれる場合、validator は common options canonical decode、
sort order、重複検査だけを行い、import 解決や public interface 検査は行いません。
ただし `Phase9TypeclassOptions.class_declarations.len() <= 65_536` は common options shape cap です。
task kind に関係なく、この cap を超える options は import 解決前に
`Rejected { error = EnvelopeMalformed, feature_error = None }`
として拒否します。
validator は `class_declarations` の vector length prefix を element decode / allocation より前にこの cap と照合しなければなりません。
この照合は options bytes validation order の step 1 に含まれる outer framing precheck であり、
cap 超過時に `class_declarations` の element を decode したり allocation したりしてはいけません。
非 TypeclassResolution task では、cap 内の `class_declarations` については上記どおり import 解決や public interface 検査を行いません。
`Phase9QuotientOptions` の quotient primitive refs と `eq` ref は、`QuotientConstruction` task で
`options.quotient = Some(...)` が使われる場合に限り同じ解決規則に従います。
`QuotientConstruction` task では `options.quotient = Some(...)` でなければならず、`SmtCertificate` task では
`options.smt = Some(...)`、`NaturalLanguageFormalization` task では `options.formalization = Some(...)` でなければなりません。
task が必須とする options field が `None` の場合は task options shape violation として
`Rejected { error = EnvelopeMalformed, feature_error = None }`
で拒否します。
その他 task はこれらの `Option` を無視せず
canonical bytes と `options_hash` には含めますが、feature-specific validation には使いません。
task が使わない optional options 内の `Phase9AiGlobalRef` は、canonical decode、sort order、重複検査の対象にはなりますが、
import 解決や public interface 検査の対象にはしません。
たとえば `UniverseRepair` request に `options.smt = Some(...)` が含まれていても、SMT ref の import 解決失敗で
Universe repair を拒否してはいけません。
task が使わない optional options でも、nested canonical bytes の構造条件は満たさなければなりません。
たとえば unused `options.formalization.tactic_options.simp_rules` は Phase 4 `MachineTacticOptions canonical bytes`
と同じ canonical sorted / duplicate-free list でなければなりません。
一方で、task-specific semantic / range validation は行いません。
たとえば unused `options.formalization.tactic_options.max_simp_rewrite_steps = 0` は
NaturalLanguageFormalization task でだけ task options shape violation として拒否し、他 task では
canonical `u64` として `options_hash` と `env_fingerprint` にだけ影響します。

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
  target_decl_hash = None
  goal_fingerprint = None
  生成される declaration hash の束縛は payload 内の MachineQuotientConstructionCandidate.expected_decl_hash で行う

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

上の表にない `target_decl_hash` / `goal_fingerprint` の Some/None 組み合わせは、task target shape violation として
`Rejected { error = EnvelopeMalformed, feature_error = None }`
で拒否します。
これは hash 再計算 mismatch ではないため `TargetFingerprintMismatch` にはしません。
ただし UniverseRepair の `target_decl_hash = Some, goal_fingerprint = None` は future declaration repair extension 用に
shape として予約済みです。この文書の MVP endpoint では payload schema を定義しないため、UniverseRepair 節の規則どおり
`Rejected { error = UnsupportedFeature, feature_error = None }`
として拒否します。

common envelope validation の task 間共通順序は次で固定します。

```text
0. top-level envelope canonical decode and candidate_hash computation
1. imports canonical order / duplicate import validation
2. options bytes validation order from this section
3. target.env_fingerprint recomputation and comparison
4. task target shape validation
5. task-required options field presence and task-specific options shape / semantic range validation
6. task-specific payload bytes decode and feature-specific payload validation
```

step 0 の top-level envelope canonical decode は、profile / task kind / target / imports / options ref /
payload bytes の field boundary を一意に確定し、`candidate_hash` を計算できるところまでを意味します。
task payload の nested semantic decode、protocol cap、sort order、または feature-local canonical shape failure は step 0 ではなく、
step 6 以降の deterministic `Rejected` です。
step 0 で top-level envelope を canonical decode できない場合だけ `Error::NonCanonicalRequestBytes` です。
step 1 の import sort order violation は `EnvelopeMalformed`、duplicate import は `ImportClosureMismatch` です。
step 2 は上で定義した options bytes validation order 全体を意味し、`options_hash` mismatch や
common options feature-support rejection は step 3 以降より優先します。
step 3 で `target.env_fingerprint` が一致しない場合、task target shape や task-required options field は検査せず
`Rejected { error = TargetFingerprintMismatch, feature_error = None }`
として拒否します。
したがって同じ request に `target.env_fingerprint` mismatch と `options.formalization = None`、
または task target shape violation が同時に含まれる場合は、`TargetFingerprintMismatch` が優先です。
`target.env_fingerprint` が一致した後にだけ、step 4 の target Some/None shape と
step 5 の `options.quotient` / `options.smt` / `options.formalization` 必須性、および nested task options の
task-specific shape / semantic range validation を行います。

---

# 3. Advanced Inductive AI

payload schema は indexed / mutual / nested inductive の宣言候補を表現できますが、
MVP validator が受理する範囲は下で定義する non-mutual / non-nested profile だけです。
AI は recursor や computation rule を任意に供給してはいけません。

```rust
struct MachineInductiveProposal {
    block_name: NamePrefix,
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

MVP AdvancedInductive の deterministic protocol cap は次で固定します。

```text
universe_params.len() <= 65_536
inductives.len() <= 65_536
each family params.len() <= 65_536
each family indices.len() <= 65_536
each family constructors.len() <= 65_536
reachable CoreExpr node total across params / indices binder ty and constructor ty <= 1_000_000
reachable LevelExpr node total across result_sort and CoreExpr level_args <= 1_000_000
```

vector length prefix の cap は element decode / allocation の前に検査します。
nested `CoreExpr` / `LevelExpr` node cap は full canonical decode 中に deterministic counter で検査します。
cap violation は
`Rejected { error = EnvelopeMalformed, feature_error = None }`
として拒否します。
この protocol cap check は `inductives.len() == 1` の MVP singleton semantic check より先に実行します。
したがって同じ request に protocol cap violation と `inductives.len() != 1` が同時にある場合は、
cap violation の `EnvelopeMalformed` が優先です。

`block_name` は family declaration name ではなく、public name を作るための optional namespace prefix です。
各 family の宣言名は `family_public_name(i)` だけであり、MVP の単一 family でも
`inductives[0].name` を別の top-level declaration name として扱ってはいけません。
prefix なしで `Vec` を宣言したい場合は `block_name = None`、
`inductives[0].name = Vec` とします。
`block_name = Some(Vec)` かつ `inductives[0].name = Vec` は意図的に `Vec.Vec` を宣言する request であり、
validator が pretty name の重複回避として自動的に短縮してはいけません。

`MachineInductiveProposal` 内で宣言中の inductive family を参照する場合は、
constructor type の `CoreExpr` 内でだけ Phase 2 `GlobalRef::Local(i)` を使います。
ここでの `i` は envelope の module declaration index ではなく、
`MachineInductiveProposal.inductives` 配列の 0-based family index です。
この task-local 解釈は Advanced Inductive payload の validation 中だけ有効です。
`params` / `indices` の binder `ty` に block-local `GlobalRef::Local` が現れた場合は
`Rejected { error = FeatureRejected, feature_error = Some(AdvancedInductive(TargetRefMismatch)) }`
として拒否します。
`result_sort` は `SortExpr = LevelExpr` なので term-level `GlobalRef::Local` を含みません。
`result_sort` 内の universe parameter は `MachineInductiveProposal.universe_params` だけを参照できます。
constructor type 内の imported constant は通常どおり `GlobalRef::Imported(import_index, name, decl_interface_hash)` で表し、
`import_index` は envelope `imports` の canonical sort 後 index です。
`GlobalRef::LocalGenerated` は proposal payload 内では常に
`Rejected { error = FeatureRejected, feature_error = Some(AdvancedInductive(TargetRefMismatch)) }`
として拒否します。
constructor / recursor / iota rule は validator が inductive declaration から生成するため、
AI は生成済み artifact を constructor type の前提として参照できません。

`MachineInductiveProposal.universe_params` は binding order のまま保存し、同じ `UniverseParam` を重複して含んではいけません。
`result_sort`、`params`、`indices`、constructor `ty` に現れる payload-local `LevelExpr` は
`MachineInductiveProposal.universe_params` だけを free universe parameter として参照できます。
`universe_params` の重複、またはこの universe context の外を参照する payload-local `LevelExpr` は
`Rejected { error = EnvelopeMalformed, feature_error = None }`
として拒否します。
この universe parameter well-formedness check は、下の feature-specific validation 順序に従い、
singleton family count check と name uniqueness check の後、constructor type check や positivity check より先に実行します。
同じ request に `inductives.len() != 1` と universe context shape violation が同時にある場合は
`UnsupportedFeature + AdvancedInductive(PositivityProfileUnsupported)` が優先です。
同じ request に name collision と universe context shape violation が同時にある場合は
`FeatureRejected + AdvancedInductive(NameCollision)` が優先です。

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
payload outer decode 後の最初の semantic check として `inductives.len() == 1` を検査し、これを満たさない request は
name uniqueness や constructor type check へ進みません。
`inductives.len() != 1` の empty / mutual block は、Phase 2 に `MutualInductiveBlock` certificate schema または
複数 `InductiveDecl` への lowering rule が追加されるまで
`Rejected { error = UnsupportedFeature, feature_error = Some(AdvancedInductive(PositivityProfileUnsupported)) }`
として拒否します。
AI Machine MVP では nested recursive occurrence も許可しません。
これは Human Profile の「approved strictly-positive functor 越しの nested inductive」を Phase 9 の上位目標として残しつつ、
Machine Profile の最初の実装単位では Phase 2 の既存 `InductiveDecl` と deterministic artifact generator に合わせるためです。
approved nested functor profile は、functoriality 証明、positivity traversal、recursor 生成、certificate hash rule を固定する
後続 profile で有効化します。
したがって AI Machine MVP では `options.advanced_inductive.approved_nested_type_constructors` は空でなければならず、
空でない場合、または constructor type 内に approved constructor 越しの recursive occurrence が現れた場合は
`Rejected { error = UnsupportedFeature, feature_error = Some(AdvancedInductive(PositivityProfileUnsupported)) }`
として拒否します。
large elimination、mutual recursor、nested recursor も MVP では
`Rejected { error = UnsupportedFeature, feature_error = Some(AdvancedInductive(PositivityProfileUnsupported)) }`
です。
MVP payload には recursor / eliminator の requested profile field を置きません。
validator は caller-provided payload field から large elimination request を読み取りません。
MVP で large elimination として拒否するのは、Phase 2 `generate_inductive_artifacts_v1` が
core-spec v0.1 の recursor generation / Prop elimination profile で扱えないと分類した declaration shape だけです。
Phase 9 validator は generator の opaque failure を見てこの分類を推測してはいけません。
generator 呼び出し前に、Phase 2 module が公開する閉じた classifier
`classify_inductive_artifact_profile_v1(base_inductive_decl_for_generation)` を呼びます。
この classifier の result は internal result code であり、wire response enum ではありません。

```text
InductiveArtifactProfileCheckV1:
  SupportedMvpRecursor
  UnsupportedMvpRecursorProfile(LargeEliminationRequired)
  UnsupportedMvpRecursorProfile(MutualOrNestedRecursorRequired)
  UnsupportedMvpRecursorProfile(UnsupportedEliminatorShape)
```

MVP validator は `SupportedMvpRecursor` 以外をすべて
`Rejected { error = UnsupportedFeature, feature_error = Some(AdvancedInductive(PositivityProfileUnsupported)) }`
へ写像し、`generate_inductive_artifacts_v1` へ進みません。
`SupportedMvpRecursor` の後で generator entrypoint だけが存在しない場合に限り
`AdvancedInductive(ArtifactGeneratorUnavailable)` を返します。
特に `result_sort = Prop` 自体は large elimination request ではありません。
Prop-valued inductive の recursor は motive codomain を `Prop` に固定する Phase 2 v0.1 profile だけを使い、
`motive : I ... -> Sort u` のような Prop から Type への elimination を生成・要求する profile は MVP では存在しません。
AI が sidecar や referenced artifact として recursor body、eliminator body、または large-elimination override を添付しても、
MVP validator はそれを feature input として読まず、受理条件にも hash にも入れません。
将来 large elimination profile を有効化する場合は、request payload に elimination universe / motive / recursor profile を
canonical field として追加し、その field から generated artifact hash と rejection category を再定義しなければなりません。

MVP で Phase 2 certificate declaration を作る場合、唯一の family `inductives[0]` から
まず recursor を持たない generation base declaration を決定的に再構成します。
constructor type は hash 計算前に block-local `GlobalRef::Local` を上記の certificate-local `GlobalRef::Local` へ
rewrite したものを使います。
この generation base declaration は generator の内部入力であり、Phase 2 certificate package に出力される declaration ではありません。
validator はこれに対して `decl_interface_hash`、`decl_certificate_hash`、import hash、dependency graph entry を計算してはいけません。
外部に見える declaration identity は、生成済み recursor を持つ `final_inductive_decl` からだけ計算します。
`inductives[0].result_sort` は `SortExpr = LevelExpr` なので、Phase 2 `InductiveDecl.sort` へ lowering する時点で
必ず core sort term `Sort(inductives[0].result_sort)` に包みます。
MVP の `rewrite_block_local_refs` は、constructor type 内の block-local `GlobalRef::Local(0)` だけを
certificate-local self reference `GlobalRef::Local(0)` に置き換えます。
`GlobalRef::Local(i)` where `i != 0` が残る場合は、`inductives.len() == 1` の MVP では必ず
`Rejected { error = FeatureRejected, feature_error = Some(AdvancedInductive(TargetRefMismatch)) }`
として拒否します。

```text
base_inductive_decl_for_generation:
  name = family_public_name(0)
  universe_params = payload.universe_params
  params = inductives[0].params
  indices = inductives[0].indices
  sort = Sort(inductives[0].result_sort)
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

`expected_decl_hash = Some(h)` の場合、validator は下の step 11 で `final_inductive_decl` から Phase 2 の通常規則で
`decl_certificate_hash` を再計算し、`h` と一致しなければ `TargetFingerprintMismatch` として拒否します。
`expected_decl_hash = None` の場合、validator は check success response に再計算した
`decl_interface_hash` と `decl_certificate_hash` を返します。
AdvancedInductive では envelope `target.target_decl_hash` は常に `None` です。
これは既存 declaration を束縛する field と、新規生成される declaration hash の payload-local expectation を混同しないためです。

検査順序は固定します。

```text
0. MachineInductiveProposal payload outer canonical decode / scalar field decode,
   vector length prefix cap precheck, and nested CoreExpr / LevelExpr count cap validation
1. singleton family count check (`inductives.len() == 1`)
2. name uniqueness
3. universe parameter well-formedness
4. parameter / index telescope task-local ref misuse precheck, imported-ref resolution, and type check
5. constructor type task-local `LocalGenerated` precheck, imported-ref resolution, and type check
6. constructor result family check
7. strict positivity check
8. MVP nested occurrence rejection consistency check
9. generated recursor generator/verifier invariant check
10. generated iota rule generator/verifier invariant check
11. expected_decl_hash binding check
```

上の順序は common envelope validation step 5 が終わった後の feature-specific validation 順序です。
step 0 で payload outer canonical decode / scalar field decode または protocol cap check に失敗した場合は、singleton family count、
payload-local name 重複、constructor type、positivity、expected_decl_hash は検査しません。
step 1 で拒否された request では payload-local name 重複、constructor type、positivity、expected_decl_hash は検査しません。
step 2 で拒否された request では universe parameter well-formedness、constructor type、positivity、expected_decl_hash は検査しません。
step 3 で拒否された request では constructor type、positivity、expected_decl_hash は検査しません。
step 9 / 10 で artifact generator unavailable として拒否した request、または generated artifact invariant failure として
`Error::InternalValidatorFailure` になった request では、
step 11 の `expected_decl_hash` binding check へ進みません。
したがって generated artifact availability / invariant failure は `expected_decl_hash` mismatch より優先します。
AdvancedInductive MVP では、non-empty `approved_nested_type_constructors` はこの列に入る前に
`PositivityProfileUnsupported` として拒否されます。
したがって step 8 は approved set で nested occurrence を許可する pass ではなく、
strict positivity traversal が nested / hidden recursive occurrence を通していないことの consistency check です。
future profile で nested inductive を許可する場合だけ、この step を approved nested occurrence check に置き換えます。
step 8 で constructor type 内に、step 7 の direct recursive occurrence rule で許可されない
`GlobalRef::Local(_)` occurrence を検出した場合は、candidate shape が MVP unsupported であるため
`Rejected { error = UnsupportedFeature, feature_error = Some(AdvancedInductive(PositivityProfileUnsupported)) }`
として拒否します。
これは validator invariant failure ではありません。
`Error::InternalValidatorFailure` は、step 8 の deterministic traversal 自体を再現できない、または
step 9 / 10 の generated artifact verifier invariant が破れるなど、validator / generator / verifier の実装不整合に限ります。

name uniqueness check は、payload-local の family / constructor name と、生成される public
family / constructor / recursor name の両方を対象にします。
`inductives[*].name`、同一 family 内の `constructors[*].name`、および
`family_public_name` / `constructor_public_name` / `recursor_public_name` の canonical `Phase9Name` bytes が
重複する candidate は
`Rejected { error = FeatureRejected, feature_error = Some(AdvancedInductive(NameCollision)) }`
として拒否します。

constructor type check の context は次で固定します。
family `i` の constructor `ty` は、universe context `payload.universe_params` と
term context `inductives[i].params` の下で検査します。
family `indices` は constructor `ty` の outer context に暗黙追加しません。
index result に使う local variable は、constructor `ty` 自身の outer `Pi` binder として明示されていなければなりません。
constructor `ty` の outer `Pi` binder を剥がすたびに、その binder を
`inductives[i].params` の後ろへ追加した constructor-local context で以後の binder type と final result を検査します。
したがって constructor result context は常に
`inductives[i].params ++ constructor_outer_pi_binders` です。
parameter / index telescope type check の拒否分類は次で固定します。
step 4 は kernel type inference より前に、`params` / `indices` の全 binder type を canonical order で走査し、
`GlobalRef::Local(_)` または `GlobalRef::LocalGenerated` を含むかを検査します。
該当する occurrence がある場合は、kernel type inference を実行せず
`Rejected { error = FeatureRejected, feature_error = Some(AdvancedInductive(TargetRefMismatch)) }`
として拒否します。
task-local ref misuse precheck が成功した後、step 4 は `params` / `indices` の全 binder type に対して
common Phase 9 wire `CoreExpr` imported ref resolution rule を適用します。
解決できない imported ref は
`Rejected { error = ImportClosureMismatch, feature_error = None }`
として拒否し、kernel type inference には進みません。
同じ request に task-local ref misuse と binder type の kernel type inference failure が同時にある場合は、
この `AdvancedInductive(TargetRefMismatch)` が優先です。
同じ request に binder type の imported ref resolution failure と kernel type inference failure が同時にある場合は、
`ImportClosureMismatch` が優先です。
`params` / `indices` の binder type が、その prefix telescope context の下で kernel type inference に失敗する場合、
または inferred type が `Sort(level)` として認識できず binder type として扱えない場合は
`Rejected { error = KernelRejected, feature_error = None }`
です。
constructor type check では、constructor `ty` 自体と outer `Pi` binder domain / final result を同じ context rule で検査します。
step 5 は constructor `ty` の kernel type inference より前に、全 constructor type を payload order で走査し、
`GlobalRef::LocalGenerated` を含むかを検査します。
該当する occurrence がある場合は、kernel type inference、constructor result family check、strict positivity check へ進まず
`Rejected { error = FeatureRejected, feature_error = Some(AdvancedInductive(TargetRefMismatch)) }`
として拒否します。
constructor type 内の `GlobalRef::Local(i)` は step 5 では拒否せず、宣言中 family への task-local ref として扱い、
step 6 の constructor result family check と step 7 の strict positivity check で scope / target-family / occurrence position を分類します。
`GlobalRef::LocalGenerated` precheck が成功した後、step 5 は全 constructor type に対して
common Phase 9 wire `CoreExpr` imported ref resolution rule を適用します。
解決できない imported ref は
`Rejected { error = ImportClosureMismatch, feature_error = None }`
として拒否し、constructor type の kernel type inference には進みません。
constructor `ty`、outer binder domain、または final result の kernel type inference に失敗する場合、
または type expression として `Sort(level)` を持たない場合は
`Rejected { error = KernelRejected, feature_error = None }`
です。
payload-local `GlobalRef::Local` の scope / target-family mismatch は、
この generic kernel failure へ潰さず、下の constructor result family check / strict positivity check の
`AdvancedInductive(TargetRefMismatch)` 規則で分類します。

future profile の `approved nested occurrence check` が使う approved set は、
`Phase9AiOptions.advanced_inductive.approved_nested_type_constructors` だけから取ります。
実装が hidden builtin list や runtime registry を追加で参照してはいけません。
MVP ではこの list は空でなければならず、nested occurrence はすべて拒否します。
non-empty list を見つけた場合、validator は imported constructor を解決して approved set を作ろうとせず、
上記の `PositivityProfileUnsupported` を返します。
approved set を実際に使うのは、nested inductive を許可する future profile だけです。

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
constructor result family check の拒否分類は次で固定します。
最終 result head が `GlobalRef::Local` でない場合、`GlobalRef::LocalGenerated` の場合、
`family_index` が `inductives` 範囲外または constructor 所属 family と一致しない場合、
`level_args.len()` が `payload.universe_params.len()` と一致しない場合、
または parameter arguments が family params の canonical weakening と byte-for-byte に一致しない場合は
`Rejected { error = FeatureRejected, feature_error = Some(AdvancedInductive(TargetRefMismatch)) }`
として拒否します。
`level_args` 内の out-of-scope universe parameter は step 3 の universe parameter well-formedness check で
`EnvelopeMalformed` として拒否済みであり、ここでは `TargetRefMismatch` へ分類し直しません。
index argument 自体が expected index type の下で kernel type check / defeq に失敗する場合は
`Rejected { error = KernelRejected, feature_error = None }`
です。

strict positivity check では、constructor type 内の任意の `GlobalRef::Local(i)` occurrence を
同一 mutual block 内の recursive occurrence として扱います。
`i` が `inductives` 範囲外の場合は
`Rejected { error = FeatureRejected, feature_error = Some(AdvancedInductive(TargetRefMismatch)) }`
として拒否します。
imported occurrence 自体は recursive occurrence ではありません。
ただし MVP では approved set を参照して nested recursive occurrence を許可する分岐はありません。
imported head、local binder、application、または reducible alias の下に `GlobalRef::Local(_)` が現れる場合は、
下の traversal が direct recursive occurrence として認識する形でない限り nested / hidden recursive occurrence として拒否します。
`approved_nested_type_constructors` を見て一部の nested occurrence を許可するのは future profile だけです。
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

strict positivity check 内の direct recursive occurrence side condition の拒否分類は次で固定します。
`family_index` が `inductives` 範囲外の場合、`level_args.len()` が `payload.universe_params.len()` と一致しない場合、
または parameter args が family params の canonical weakening と byte-for-byte に一致しない場合は
`Rejected { error = FeatureRejected, feature_error = Some(AdvancedInductive(TargetRefMismatch)) }`
です。
`level_args` 内の out-of-scope universe parameter は step 3 の universe parameter well-formedness check で
`EnvelopeMalformed` として拒否済みであり、positivity check では分類し直しません。
index args に `GlobalRef::Local(_)` recursive occurrence が含まれる場合は、recursive occurrence が index に現れる MVP unsupported shape として
`Rejected { error = UnsupportedFeature, feature_error = Some(AdvancedInductive(PositivityProfileUnsupported)) }`
です。
index args が family indices telescope の expected type の下で kernel type check / defeq に失敗する場合は
`Rejected { error = KernelRejected, feature_error = None }`
です。
`otherwise` branch で検出した nested / negative / hidden recursive occurrence は
`Rejected { error = UnsupportedFeature, feature_error = Some(AdvancedInductive(PositivityProfileUnsupported)) }`
です。

`weak_head_normalize_for_inductive_positivity` は β / ζ だけを使います。
imported `DefDecl` の δ unfolding、opaque theorem、axiom、typeclass search、quotient computation rule、
AI hint、または runtime registry は使いません。
したがって `F I`、`I -> X`、`X -> I`、`List I`、reducible alias の中に隠れた `I` は、
MVP では validator の conservative traversal が検出した場合
`Rejected { error = UnsupportedFeature, feature_error = Some(AdvancedInductive(PositivityProfileUnsupported)) }`
として拒否します。
Phase 2 kernel / artifact generator 側の positivity checker が同じ declaration を拒否した場合は
`Rejected { error = KernelRejected, feature_error = None }` です。
将来これらを許可する場合は、nested / higher-order positivity profile を別に定義し、
polarity traversal と approved type constructor の functoriality 証明を certificate に束縛します。

generated artifact の名前と順序は AI から受け取らず、次の `inductive_artifact_profile` で固定します。

```text
inductive_artifact_profile = "npa.phase9_ai.inductive-artifacts.v1"

family_public_name(i) =
  if block_name = None:
    inductives[i].name
  else if block_name = Some(prefix):
    append_name(prefix, inductives[i].name)

constructor_public_name(i, j) =
  append_name(family_public_name(i), inductives[i].constructors[j].name)

recursor_public_name(i) =
  append_name(family_public_name(i), name("rec"))

iota_rule_key(i, j) =
  (recursor_public_name(i), constructor_public_name(i, j))
```

`append_name` は Phase 2 generated artifact name と同じ canonical `NameId` path append で、
左辺と右辺の `Phase9Name` component list を順に連結します。
`name("rec")` は single-component `Phase9Name` です。
生成後の family / constructor / recursor name が重複する candidate は
`Rejected { error = FeatureRejected, feature_error = Some(AdvancedInductive(NameCollision)) }`
として拒否します。
validator は受理済み declaration から `generate_inductive_artifacts_v1` を1回だけ実行し、
constructor specs、recursor specs、Phase 2 `RecursorSpec.rules`、および Phase 2 public interface に入る
generated computation rule hash を family order / constructor order で canonical encode します。
MVP には separate `IotaRuleSpec` canonical object は存在しません。
この節で iota rule と呼ぶものは、Phase 2 verifier が `RecursorSpec.rules` から再生成して public interface hash に含める
generated computation rule hash だけを指します。
この生成関数の入力は proposal payload から再構成した canonical `base_inductive_decl_for_generation`、
`profile_version`、および import closure から解決済みの public interfaces だけです。
`expected_decl_hash`、AI explanation、または response destination は生成入力に含めません。
AI sidecar、runtime registry、現在の IDE state、または caller-provided recursor body は入力にしません。
MVP では `generate_inductive_artifacts_v1` は extension point ではなく、Phase 2 の canonical inductive artifact generator
と同じ実装を呼ぶ alias です。
別実装を Phase 9 validator 内だけに持ってはいけません。
したがって MVP validator は「Phase 2 generator と Phase 9 generator の2実装を比較する」形の
determinism check を持ちません。
generator が返した recursor / iota rule を持つ `final_inductive_decl` は、Phase 2 verifier の通常の
generated artifact check に渡して、同じ `base_inductive_decl_for_generation` から再生成される artifact と一致することを確認します。
この check が失敗する場合は candidate の shape による通常の rejection ではなく、validator / generator / verifier の
implementation invariant 破れなので `Error::InternalValidatorFailure` です。
`AdvancedInductive(GeneratedArtifactMismatch)` は、caller-provided generated artifact や複数 generator profile を
canonical payload として扱う future profile 用の予約 error であり、MVP validator は返してはいけません。
Phase 2 側にこの profile 用 generator がまだ実装されていない場合、Advanced Inductive validator は
candidate を受理せず
`Rejected { error = UnsupportedFeature, feature_error = Some(AdvancedInductive(ArtifactGeneratorUnavailable)) }`
を返します。
ここでいう `ArtifactGeneratorUnavailable` は、singleton / non-nested / non-large-elimination の MVP profile を
feature-specific validation が通過した後、必要な Phase 2 canonical generator だけが存在しない場合に限ります。
mutual block、nested occurrence、higher-order positivity、large elimination など MVP profile 自体が受け付けない shape は、
generator に到達する前に
`Rejected { error = UnsupportedFeature, feature_error = Some(AdvancedInductive(PositivityProfileUnsupported)) }`
として拒否します。
M3 を完了とみなせるのは、non-mutual / non-nested / non-large-elimination のこの profile で
Phase 2 generator の出力と Phase 2 verifier の generated artifact check が同じ canonical bytes に合意する場合だけです。
Phase 9 core がまだ deterministic generator を定義していない組み合わせ
（例: mutual recursor、nested recursor、large elimination profile）は、
validator が独自生成を推測せず、MVP では上の `PositivityProfileUnsupported` として拒否します。
future profile でその shape を supported profile に昇格した後、その profile 用 generator だけが未実装の場合に
`ArtifactGeneratorUnavailable` を使います。

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

enum UniverseConstraintHintReason {
    KernelDiagnostic,
    RepairCandidate,
    MinimizationExplanation,
}

enum UniverseMinimizationHint {
    KernelDefault,
    PreferLowerLevels,
    PreferExistingExplicitArgs,
}
```

`UniverseConstraintHintReason` と `UniverseMinimizationHint` の canonical bytes は variant tag だけで固定します。
MVP では free-form text、source span、score、または model explanation をこれらの enum に入れません。
MVP の UniverseRepair payload deterministic protocol cap は
`instantiations.len() <= 65_536`、`constraint_hints.len() <= 65_536`、
各 `MachineExprOccurrence.path.len() <= 65_536`、および各 `explicit_level_args.len() <= 65_536` です。
これらの vector length prefix は element decode / allocation より前に cap と照合しなければなりません。
cap を超える request は resource guard ではなく payload canonical shape violation として
`Rejected { error = EnvelopeMalformed, feature_error = None }`
で拒否します。
validator は server-local timeout、memory guard、runtime configuration からこの cap を増減してはいけません。
`minimization_hint` は AI repair loop の探索順序や debug 表示のヒントだけです。
採用される level assignment は、canonical universe solver の出力でなければなりません。
MVP で solver が payload に存在しない universe argument を新たに選んで response field として返すことはありません。
ここでの solver output は、`instantiations` を適用した `repaired_expr` と、そこから validator が再導出して
canonical sort した universe constraint set の充足判定です。
unmentioned occurrence の universe args は元の `target_expr` のまま残り、validator が hidden assignment を補完してはいけません。
validator は `minimization_hint` を canonical decode しますが、solver objective や tie-break には渡しません。
`minimization_hint = None`、`KernelDefault`、`PreferLowerLevels`、`PreferExistingExplicitArgs` は
異なる `candidate_hash` と `validation_result_hash` を持てます。
ただし同じ `target_expr` / `instantiations` なら、hint の違いだけで solver input、`repaired_expr`、
`constraint_set_hash`、または rejection category が変わってはいけません。
`MachineUniverseInstantiationPatch` は、`target_expr` 内のどの occurrence に
universe args を与えるかを固定します。対象 ref は `occurrence.expected_ref` です。
複数の polymorphic occurrence がある場合、AI は occurrence ごとに
patch を分けて出します。
`instantiations` は `(occurrence.path canonical bytes, occurrence.expected_ref canonical bytes)` の辞書順で
strictly sorted され、同じ occurrence key を重複して含んではいけません。
同じ path / ref に同じ `explicit_level_args` を2回指定した場合も重複として拒否し、
validator が silently merge してはいけません。
`instantiations` の sort order violation または duplicate occurrence key は payload canonical shape violation として
`Rejected { error = EnvelopeMalformed, feature_error = None }`
で拒否します。
この check は path traversal、`occurrence.expected_ref` の import 解決、`explicit_level_args` の arity / scope validation より先に行います。
ただし UniverseRepair の target mode / target binding check はこの check より先に行います。
同じ request に duplicate occurrence key と `goal = None`、`goal.target` / `target_expr` mismatch、
または `goal_fingerprint` mismatch が同時にある場合は、
この段落の `EnvelopeMalformed` ではなく、下の target binding rejection が優先です。
同じ request に duplicate occurrence key と invalid path / unknown universe parameter が同時にある場合は、
target binding が成功した後に限り `EnvelopeMalformed` が優先です。
同じ path に異なる `expected_ref` を指定した patch も、path traversal 後の ref 一致検査で少なくとも一方が
`Rejected { error = FeatureRejected, feature_error = Some(UniverseRepair(TargetRefMismatch)) }`
になります。
`target.goal_fingerprint` が `Some` の場合、`goal` は `Some(Phase9AiGoal)` でなければなりません。
`target.goal_fingerprint = Some(_)` かつ `goal = None` の request は goal mode payload shape violation として
`Rejected { error = EnvelopeMalformed, feature_error = None }`
で拒否します。これは fingerprint 再計算 mismatch ではないため `TargetFingerprintMismatch` にはしません。
`goal.target` の canonical bytes は `target_expr` と byte-for-byte で一致し、common envelope の式で
`goal.universe_params`、`goal.local_context`、`goal.target` から fingerprint を再計算します。
`goal.target` と `target_expr` が一致しない request は fingerprint 再計算結果とは独立に
`Rejected { error = TargetFingerprintMismatch, feature_error = Some(UniverseRepair(TargetFingerprintMismatch)) }`
として拒否します。
これらの UniverseRepair target mode / target binding check は、`instantiations` と `constraint_hints` の
sort order / duplicate / feature-local level validation より先に実行します。
したがって target binding mismatch と `instantiations` / `constraint_hints` の canonical shape violation が同時にある場合は
`TargetFingerprintMismatch` または goal mode payload shape violation の `EnvelopeMalformed` が優先です。
closed expression repair は
`goal = Some(Phase9AiGoal { universe_params = [], local_context = [], target = target_expr })` として表します。
open goal の universe repair は `goal.universe_params` と `goal.local_context` を明示することで扱います。
`/machine/phase9/universe/repair/check` の MVP は goal mode だけを受け付けます。
`target.target_decl_hash` が `Some` の declaration repair mode は、この文書の MVP では payload schema を定義しないため
`Rejected { error = UnsupportedFeature, feature_error = None }`
として拒否します。将来有効化する場合は、target declaration 全体を request の canonical wrapper から
再構成できる別 payload を定義し、`target_decl_hash` の再計算手順をその payload に固定しなければなりません。
将来の declaration repair mode では `goal = None` でなければなりません。
ただし MVP では `target.target_decl_hash = Some` を見た request の `goal` の Some/None は検査対象にしません。
`target_decl_hash = Some, goal_fingerprint = None, goal = Some(_)` の request も、payload shape violation ではなく
declaration repair mode 全体の未対応として同じ `UnsupportedFeature` で拒否します。
MVP では `target.target_decl_hash = Some` を見た時点で current `MachineUniverseRepairCandidate.target_expr` を
declaration body、declaration type、または repair 対象 expression として解釈してはいけません。
canonical decode と common target shape check、および feature-specific step 0 の payload outer canonical framing /
scalar field decode / length prefix cap precheck の後、`target.target_decl_hash = Some` の request は
goal-mode payload shape、`target_expr` の semantic validation、`instantiations` / `constraint_hints` element decode へ進む前に
上記 `UnsupportedFeature` で拒否します。
したがって current schema 上 `target_expr` が存在していても、declaration repair の witness にはなりません。
`explicit_level_args` は path で到達した `Const` occurrence の universe argument list を置換する patch です。
validator は `occurrence.expected_ref` の public interface から universe parameter order と arity を取得し、
`explicit_level_args.len()` が arity と一致しない場合は
`Rejected { error = FeatureRejected, feature_error = Some(UniverseRepair(IllFormedLevelExpr)) }`
として拒否します。
この段落でいう `LevelExpr` は、`instantiations[*].explicit_level_args` と
`constraint_hints[*].constraint` に含まれる UniverseRepair feature-local level expression です。
`payload.goal.local_context` / `payload.goal.target` / `target_expr` に含まれる `LevelExpr` の universe context shape violation は、
共通 `Phase9AiGoal` validation で
`Rejected { error = EnvelopeMalformed, feature_error = None }`
として拒否済みであり、この `UniverseRepair(UnknownUniverseParam)` 分類には到達しません。
goal mode の feature-local `LevelExpr` は `payload.goal.universe_params` だけを free universe parameter として参照できます。
declaration repair extension では、別 payload に含まれる target declaration の `universe_params` だけを参照できます。
余剰・不足・重複 binder 参照は
`Rejected { error = FeatureRejected, feature_error = Some(UniverseRepair(IllFormedLevelExpr)) }`
として拒否します。
未宣言 parameter 参照は
`Rejected { error = FeatureRejected, feature_error = Some(UniverseRepair(UnknownUniverseParam)) }`
として拒否します。
patch 適用後の `target_expr` から validator が universe constraints を再導出し、canonical solver に渡します。
`MachineExprOccurrence.path` は elaboration 前 source span ではなく、`target_expr` の canonical `CoreExpr` tree path です。
validator は path の到達先が global constant occurrence であり、その core ref が
`resolve_imported_ref(occurrence.expected_ref)` と一致することを確認します。
step 7 の `instantiations` semantic validation は、`instantiations` の canonical order で patch を1つずつ処理し、
最初に失敗した patch の error を返します。
各 patch 内の suborder は次で固定します。

```text
7a. occurrence.path traversal on target_expr
7b. occurrence.expected_ref import resolution
7c. reached Const ref equals resolve_imported_ref(occurrence.expected_ref)
7d. explicit_level_args arity check against the resolved public interface
7e. explicit_level_args universe scope validation, left to right
```

同じ patch に invalid path と import 解決不能が同時にある場合は、7a の
`UniverseRepair(InvalidOccurrencePath)` が優先です。
path は有効だが `occurrence.expected_ref` が envelope imports から一意に解決できない場合は
`Rejected { error = ImportClosureMismatch, feature_error = None }`
として拒否し、explicit level argument の arity / scope validation には進みません。
到達先が `Const` で、ref 解決も成功したが一致しない場合は 7c の
`UniverseRepair(TargetRefMismatch)` が、explicit level argument の検査より優先です。
explicit level argument の arity mismatch と未宣言 parameter 参照が同じ patch にある場合は、
7d の `UniverseRepair(IllFormedLevelExpr)` が優先です。
arity が一致した後、7e では `explicit_level_args` を payload order で左から右へ検査し、
最初の未宣言 parameter 参照を `UniverseRepair(UnknownUniverseParam)` として拒否します。

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
global constant occurrence. Applying an invalid child step to a CoreExpr variant is
`Rejected { error = FeatureRejected, feature_error = Some(UniverseRepair(InvalidOccurrencePath)) }`.
If path traversal reaches `Const` or `BVar`, additional path steps are invalid and use the same rejected result.
MVP の `MachineExprOccurrence.path` は concrete tree path なので、canonical path traversal の到達先は高々1つです。
path が有効でも到達先が `Const` でない場合は
`Rejected { error = FeatureRejected, feature_error = Some(UniverseRepair(InvalidOccurrencePath)) }`
です。
到達先が `Const` だが `occurrence.expected_ref` と一致しない場合は
`Rejected { error = FeatureRejected, feature_error = Some(UniverseRepair(TargetRefMismatch)) }`
です。
`AmbiguousOccurrence` は future selector profile 用の予約 error であり、この MVP concrete path profile では返してはいけません。
MVP の path table はここに列挙した CoreExpr variant に対して閉じています。
validator がこれ以外の CoreExpr variant を含む `target_expr` を受け取った場合、path traversal を推測してはいけません。
その request は、その variant 用の `MachineExprPathStep` が Phase 9 AI Profile に追加されるまで
`Rejected { error = UnsupportedFeature, feature_error = None }`
として拒否します。

`constraint_hints` は AI repair の補助情報です。
canonical solver の入力は、`target_expr` と `instantiations` から validator が導出した制約だけです。
AI が `constraint_hints` で新しい trusted constraint を追加することはできません。
`constraint_hints` は `constraint canonical bytes` の辞書順で strictly sorted され、
同じ constraint を複数含んではいけません。
同じ constraint に複数の説明理由を残したい場合でも、MVP payload では1つの `reason` variant だけを選びます。
`constraint_hints` の sort order violation または duplicate constraint は payload canonical shape violation として
`Rejected { error = EnvelopeMalformed, feature_error = None }`
で拒否します。
この check は `constraint_hints[*].constraint` の feature-local `LevelExpr` scope validation や、
導出済み constraint 集合との照合より先に行います。
ただし UniverseRepair の target mode / target binding check はこの check より先に行います。
同じ request に duplicate constraint と `UnknownUniverseParam` / `ConstraintHintMismatch` が同時にある場合は、
target binding が成功した後に限り `EnvelopeMalformed` が優先です。
各 `constraint_hints[*].constraint` は、patch 適用後の `target_expr` から validator が導出した universe constraint 集合に
canonical bytes で存在しなければなりません。
導出されていない constraint、導出 constraint と矛盾する constraint、または canonical normalize 後に一致しない constraint を含む場合は
`Rejected { error = FeatureRejected, feature_error = Some(UniverseRepair(ConstraintHintMismatch)) }`
として拒否します。
導出 constraint のすべてを `constraint_hints` に含める必要はありません。
missing hint は許可され、solver input、`repaired_expr`、`constraint_set_hash`、または rejection category には影響しません。
patch 適用後に再導出した universe constraints が canonical solver で充足不能な場合は
`Rejected { error = NoSolution, feature_error = Some(UniverseRepair(UnsatisfiedConstraint)) }`
です。
MVP では `repaired_expr` と `constraint_set_hash` は validator がこの節の canonical rule から決定的に計算する
response field であり、caller-provided solution bytes は存在しません。
したがって solver が充足可能と判定した後に、validator が `repaired_expr` または `constraint_set_hash` を
byte-for-byte に再計算できない場合は candidate rejection ではなく implementation invariant 破れとして
`Error::InternalValidatorFailure` です。
`UniverseRepair(NonCanonicalSolution)` は、caller-provided solution bytes や外部 solver certificate を
canonical payload として検査する future profile 用の予約 error であり、MVP validator は返してはいけません。

UniverseRepair の feature-specific validation 順序は固定します。
この順序は common envelope validation step 5 が終わった後に適用します。
`MachineUniverseRepairCandidate` payload bytes の task-specific decode はこの feature-specific validation の step 0 であり、
common envelope validation step 3 の `target.env_fingerprint` mismatch より先に実行してはいけません。

```text
0. MachineUniverseRepairCandidate payload outer canonical framing / scalar field decode,
   and instantiations / constraint_hints length prefix cap precheck
1. declaration repair mode unsupported check (`target.target_decl_hash = Some`)
2. goal-mode payload shape and target binding check
   - `goal = Some`
   - `goal.target` byte-identical to `target_expr`
   - `target.goal_fingerprint` recomputation and comparison
3. Phase9AiGoal wire shape / universe context shape and imported-ref resolution check
4. Phase9AiGoal local_context / target kernel well-typedness check
5. instantiations element canonical decode, occurrence.path / explicit_level_args length prefix cap precheck,
   sort order, and duplicate occurrence key check
6. constraint_hints element canonical decode, sort order, and duplicate constraint check
7. instantiations semantic validation in canonical patch order:
   occurrence path traversal, occurrence.expected_ref import resolution, reached ref check,
   explicit_level_args arity / scope validation
8. apply instantiations, derive canonical universe constraint set, and compare constraint_hints
9. canonical universe solver, repaired_expr / constraint_set_hash construction, and response construction
```

step 0 で payload outer decode や cap check に失敗した場合は、`goal`、target binding、
`instantiations` element、`constraint_hints` element は検査しません。
step 1 の declaration repair mode rejection は、goal-mode payload shape、`goal` の Some/None、
`target_expr` の semantic validation、`instantiations` / `constraint_hints` element decode を行う前に実行します。
step 2 で `goal = None`、`goal.target` / `target_expr` mismatch、または `target.goal_fingerprint` mismatch と判定した場合は、
`Phase9AiGoal` universe context shape、`instantiations` duplicate、`constraint_hints` duplicate には進みません。
step 5 の `instantiations` duplicate は occurrence path traversal や import resolution より前に行います。
step 6 の `constraint_hints` duplicate は feature-local `LevelExpr` scope validation や導出済み constraint 集合との照合より前に行います。
したがって同じ request に duplicate occurrence key と invalid path が同時にある場合は、step 5 の `EnvelopeMalformed` が優先です。
同じ request に duplicate constraint と `UnknownUniverseParam` / `ConstraintHintMismatch` が同時にある場合は、
step 6 の `EnvelopeMalformed` が優先です。

拒否する例:

```text
- undeclared universe parameter を参照する
- `succ u <= u` や `succ u = u` に相当する positive succ cycle など、canonical solver が unsat と判定する
  universe constraint cycle がある
- cumulativity を使って forbidden coercion を通す
- pretty name だけで level を指定する
- target env_fingerprint と違う環境の repair を再利用する
- occurrence path が invalid、または path の到達先が `Const` ではない
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
`ordered_candidates` は重複 candidate target を含んではいけません。
ここでの duplicate key は `MachineInstanceTargetRef` の canonical bytes であり、`priority_hint` は duplicate key に含めません。
同じ target を `priority_hint` だけ変えて複数回入れた request は
`Rejected { error = EnvelopeMalformed, feature_error = None }`
として拒否します。
この duplicate target check は import closure 解決や candidate public type 読み出しより前に行う payload structural validation です。
したがって同じ `MachineInstanceTargetRef` canonical bytes が複数回出現する request は、その ref が後続の import resolution で解決不能であっても
`EnvelopeMalformed` が優先し、`ImportClosureMismatch` には到達しません。
validator は `ordered_candidates` を dedup したり、重複を別 attempt として budget 消費させたりしてはいけません。
MVP の TypeclassResolution payload deterministic protocol cap は `max_depth <= 1024`、`max_nodes <= 1_000_000`、
`ordered_candidates.len() <= 65_536` です。
`Phase9TypeclassOptions.class_declarations.len() <= 65_536` は common options shape cap として上で定義済みです。
`max_depth = 0` / `max_nodes = 0` は下で定義する valid fuel 値ですが、cap を超える値、または `ordered_candidates` cap を超える request は
resource guard ではなく
wire shape violation として
`Rejected { error = EnvelopeMalformed, feature_error = None }`
で拒否します。
validator は server-local timeout、memory guard、runtime configuration からこの cap を増減してはいけません。
validator は `ordered_candidates` の vector length prefix を element decode / allocation より前にこの cap と照合しなければなりません。
TypeclassResolution payload bytes validation は、`MachineTypeclassResolutionPlan` の outer canonical framing と
scalar field decode を先に行い、`max_depth` / `max_nodes` cap と `ordered_candidates` length prefix cap を
candidate element decode / allocation より前に照合します。
この段階で cap を超える場合は、canonical request bytes から `candidate_hash` を計算できる限り通常の
`Rejected { error = EnvelopeMalformed, feature_error = None }` として返します。
top-level request framing が壊れており payload bytes の範囲を一意に確定できない場合だけ、
request 全体の non-canonical bytes failure として扱います。
outer framing precheck を通過した payload では、validator は `ordered_candidates` を materialize しなくても
exact payload bytes から `candidate_hash` を計算できなければなりません。

TypeclassResolution の feature-specific validation 順序は固定します。
この順序は common envelope validation step 5 が終わった後に適用します。
`MachineTypeclassResolutionPlan` payload bytes の task-specific decode はこの feature-specific validation の step 0 であり、
common envelope validation step 3 の `target.env_fingerprint` mismatch より先に実行してはいけません。

```text
0. MachineTypeclassResolutionPlan payload outer canonical framing / scalar field decode,
   max_depth / max_nodes cap check, and ordered_candidates length prefix cap precheck
1. payload.goal and ordered_candidates element canonical decode, and duplicate target check
2. target.goal_fingerprint binding check
3. Phase9AiGoal wire shape / universe context shape and imported-ref resolution check
4. Phase9AiGoal local_context / target kernel well-typedness check
5. options.typeclass.class_declarations import resolution and class declaration public-interface check
6. ordered_candidates imported ref resolution and candidate public-interface decomposition
7. initial obligation class head support check
8. deterministic resolver search, budget accounting, ambiguity / no-solution classification
9. final proof / dictionary term kernel check and response construction
```

step 0 で payload outer decode や cap check に失敗した場合は、`payload.goal`、`ordered_candidates` element、
class declaration import resolution は検査しません。
step 1 の duplicate target check は import closure 解決、candidate public type 読み出し、
および `target.goal_fingerprint` binding check より前に行います。
step 2 で `target.goal_fingerprint` mismatch と判定した場合は、class declaration import resolution には進みません。
したがって同じ request に duplicate target と candidate ref import 解決不能が同時にある場合は
`EnvelopeMalformed` が優先です。
step 5 の class declaration ref 解決不能は、step 6 の candidate ref 解決不能より優先します。

typeclass resolver の deterministic search rule は固定します。

```text
- search state は pending obligations queue、proof_args、visited stack、node_count を持つ
- 初期 queue は goal.target 1件だけ
- 1 branch 内の queue は FIFO で取り出す
- 各 obligation に対し ordered_candidates を配列順に走査する
- candidate entry を検査する直前に `node_count` budget を消費する
- candidate の head が obligation の class head と defeq で一致しない場合は prefilter skip
- candidate を適用して生じる recursive obligations は、candidate telescope の引数順で queue の末尾へ追加する
- 同じ obligation fingerprint と candidate ref が visited stack にある場合は cycle としてその branch を拒否する
- node_count は candidate entry を検査した時点で 1 増やす
- depth は current branch で structural matching と argument classification に成功し child frame を作った
  candidate application 数で数え、`max_depth` を超える child frame 作成は request 全体の `BudgetExceeded`
- `max_nodes = 0` は candidate entry inspection 禁止を意味し、`ordered_candidates` が空でない obligation を処理する時点で `BudgetExceeded`
- `max_depth = 0` は child frame 作成禁止を意味し、structural matching 成功後に child frame が必要になった時点で `BudgetExceeded`
```

direct instance で recursive obligation が発生しない場合でも、structural matching と argument classification に成功した
candidate application は child frame を1つ作ります。
したがって `max_depth = 0` は direct instance も含めて successful candidate application を禁止する budget です。
direct instance を1件だけ許可したい caller は `max_depth >= 1` を指定します。

`proof_args` は候補適用で確定した explicit argument、implicit argument、recursive proof slot の列であり、
`chosen instances` という独立した state は持ちません。
成功時に返す instance chain は `proof_args` と final proof term から導出される debug metadata にすぎず、
search result の canonical identity には final proof term の canonical bytes だけを使います。

`node_count` budget は一意性確認を含む探索全体に適用します。
head defeq mismatch による prefilter skip は candidate application attempt ではありませんが、
candidate list scan そのものを deterministic budget 対象にするため `node_count` を消費します。
各 obligation で `ordered_candidates[i]` を読む直前に `node_count < max_nodes` でなければならず、
満たさない場合は request 全体を `BudgetExceeded` として拒否します。
`node_count` 条件を満たす場合、resolver は head defeq check の前に `node_count += 1` し、
prefilter skip、structural matching failure、argument classification failure、kernel check failure が後で起きても
この `node_count` は戻しません。
candidate の head が一致した後、structural matching と argument classification を始める直前が
candidate application attempt の開始点です。
application attempt の開始時点では `current_depth < max_depth` をまだ検査しません。
candidate application が structural matching と argument classification に成功して child frame を作る直前に、
`current_depth < max_depth` でなければなりません。
満たさない場合は request 全体を `BudgetExceeded` として拒否します。
この depth check は structural matching failure には適用しません。
depth check を通った場合だけ、
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
  current_depth: successful candidate application / child frame count on this branch
```

`search(frame)` は branch-local success enumerator です。
`queue` が空なら success candidate を yield して呼び出し元の continuation へ戻りますが、
endpoint-level resolver はそこで短絡して success を返してはいけません。
top-level resolver は最初の success candidate を保存し、残りの branch を同じ budget 内で探索して
ambiguity と budget exhaustion を確認します。
initial frame は `queue = [goal.target]`、`proof_args = []`、`visited_stack = []`、
`current_depth = 0` で作ります。
そうでなければ queue の先頭 obligation を取り出し、その obligation に対して `ordered_candidates` を配列順に走査します。
candidate application が structural matching で失敗した場合は同じ obligation の次 candidate を試します。
成功した場合は、残り queue の末尾に recursive obligations を candidate telescope order で追加した child frame を作り、
その child frame を先に探索します。
child frame が success candidate を yield した場合、top-level resolver はその proof term を記録したうえで、
child frame と親 frame の continuation を再開し、同じ obligation の後続 candidate も含めて残り branch を探索します。
child frame が no-solution / cycle / kernel rejection で失敗または探索完了した場合、親 frame は同じ obligation の次 candidate を試します。
`BudgetExceeded` と `AmbiguousResolution` は branch failure ではなく request 全体の terminal result です。
`visited_stack` の更新は immutable frame stack として扱います。
candidate application の structural matching と argument classification が成功した後、
resolver は `(obligation_fingerprint_after_matching, resolved_candidate_ref)` を cycle entry として作ります。
この entry が親 frame の `visited_stack` に既に存在する場合、その child は cycle として no-solution branch failure になります。
存在しない場合だけ、child frame の `visited_stack` は
`parent.visited_stack ++ [entry]` です。
親 frame 自体は変更しないため、child 探索から戻った時点の pop は親 frame を再利用することで決定的に表現します。
すべての branch を探索し尽くしても success candidate が1つも得られなかった場合は、terminal result として
`Rejected { error = NoSolution, feature_error = Some(TypeclassResolution(NoSolution)) }` を返します。
これは branch-local no-solution ではなく request 全体の no-solution です。
branch-local kernel rejection は「その candidate application では solution を作れなかった」という探索失敗として扱い、
全 branch が branch-local kernel rejection だけで終わった場合も terminal result は `NoSolution` です。
`KernelRejected` を返すのは、resolver が一意な success proof term を得た後の最終 kernel check で
その proof term が `goal.target` の proof/dictionary term として拒否された場合だけです。

candidate interface は、imported declaration の public type を weak-head normalize し、Pi telescope を剥がした
result type から決定します。
class declaration は `Phase9AiOptions.typeclass.class_declarations` に含まれる `Phase9AiGlobalRef` だけです。
validator は検証開始時に `class_declarations` の各 `Phase9AiGlobalRef` を `resolve_imported_ref` し、
`resolved_class_declarations: Set<GlobalRef::Imported>` を作ります。
解決できない class declaration ref は
`Rejected { error = ImportClosureMismatch, feature_error = None }`
として拒否します。
common options canonical shape validation で tuple key 上の duplicate は既に
`Rejected { error = EnvelopeMalformed, feature_error = None }`
として拒否済みです。
MVP の `resolve_imported_ref` は `GlobalRef::Imported(import_index, name, decl_interface_hash)` を返し、
`import_index` は envelope import tuple から決まるため、tuple key として異なる ref を同じ resolved core ref へ
canonicalize して dedup する追加段階はありません。
class declaration role として使えない public interface は
`Rejected { error = FeatureRejected, feature_error = Some(TypeclassResolution(ClassDeclarationMismatch)) }`
として拒否します。
result type の head がこの `resolved_class_declarations` に含まれる core ref でなければ、その candidate は instance candidate として無効です。
`candidate の head` はこの result type head です。
各 obligation target も weak-head normalize し、head が同じ `resolved_class_declarations` に含まれなければ
`Rejected { error = UnsupportedFeature, feature_error = Some(TypeclassResolution(ClassHeadUnsupported)) }`
として拒否します。
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
`Rejected { error = FeatureRejected, feature_error = Some(TypeclassResolution(CandidateInterfaceInvalid)) }`
として request 全体を拒否します。
candidate ref が解決不能な場合は `ImportClosureMismatch`、candidate public type が Pi telescope と result type へ
決定的に分解できない場合や binder dependency をこの MVP matching 規則で扱えない場合は
`Rejected { error = FeatureRejected, feature_error = Some(TypeclassResolution(CandidateInterfaceInvalid)) }`
です。
result type head が `resolved_class_declarations` に含まれない candidate は request 全体の rejection ではなく、
candidate application attempt 前の deterministic skip です。
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
child frame が success candidate を yield した場合も、top-level resolver はその proof term を記録したうえで
child frame と親 frame の continuation を再開し、同じ obligation の後続 candidate を探索します。
child frame が no-solution / cycle / kernel rejection で失敗した場合、または child frame の探索が完了した場合、
親 frame は同じ obligation の次 candidate を試します。
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
    expected_decl_hash: Option<Hash256>,
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

MVP QuotientConstruction の deterministic protocol cap は次で固定します。

```text
universe_params.len() <= 65_536
params.len() <= 65_536
operations.len() <= 65_536
reachable CoreExpr node total across params binder ty, quotient_type, carrier, relation,
  equivalence_proof, and operation raw_function / compatibility_proof <= 1_000_000
reachable LevelExpr node total across CoreExpr level_args <= 1_000_000
```

vector length prefix の cap は element decode / allocation の前に検査します。
nested `CoreExpr` / `LevelExpr` node cap は full canonical decode 中に deterministic counter で検査します。
cap violation は
`Rejected { error = EnvelopeMalformed, feature_error = None }`
として拒否します。
この protocol cap check は operations sort order / duplicate name check より先に実行します。
したがって同じ request に protocol cap violation と operations sort order / duplicate name violation が同時にある場合は、
cap violation の `EnvelopeMalformed` が優先です。

`operations` は `name` の canonical bytes 昇順で strictly sorted されていなければなりません。
同じ `name` を複数含む candidate は拒否します。
validator は受け取った順序を silently sort せず、順序違反を deterministic error として返します。
`operations` の sort order violation または duplicate name は payload canonical shape violation として
`Rejected { error = EnvelopeMalformed, feature_error = None }`
で拒否します。
`MachineQuotientConstructionCandidate.universe_params` は binding order のまま保存し、同じ `UniverseParam` を重複して含んではいけません。
`params`、`quotient_type`、`carrier`、`relation`、`equivalence_proof`、および各 operation の
`raw_function` / `compatibility_proof` に現れる payload-local `LevelExpr` は
`MachineQuotientConstructionCandidate.universe_params` だけを free universe parameter として参照できます。
`universe_params` の重複、またはこの universe context の外を参照する payload-local `LevelExpr` は
`Rejected { error = EnvelopeMalformed, feature_error = None }`
として拒否します。
同じ shape check では、`params[*].ty`、`quotient_type`、`carrier`、`relation`、`equivalence_proof`、
および各 operation の `raw_function` / `compatibility_proof` の全 `CoreExpr` を canonical order で走査し、
`GlobalRef::Local(_)` または `GlobalRef::LocalGenerated` を含む request も
`Rejected { error = EnvelopeMalformed, feature_error = None }`
として拒否します。
QuotientConstruction MVP では task-local global ref を使わず、params は de Bruijn local binder だけで参照します。
これは payload wire shape violation であり、`KernelRejected` や `QuotientConstruction(TargetRefMismatch)` には分類しません。
`QuotientConstruction(TargetRefMismatch)` は Error Model のとおり future target-bound quotient repair profile 用の予約 variant です。
この universe context / task-local ref shape check は operations sort order / duplicate name check の後、
primitive import 解決や kernel type inference より先に実行します。
同じ request に operations sort order / duplicate name violation と universe context shape violation が同時にある場合は、
operations 側の `EnvelopeMalformed` が優先です。
QuotientConstruction では envelope `target.target_decl_hash` は常に `None` です。
これは既存 declaration を束縛する field と、新規生成される quotient declaration hash の payload-local expectation を
混同しないためです。
`expected_decl_hash = Some(h)` の場合、validator が quotient declaration を再構成した後の `decl_certificate_hash` が
`h` と一致しなければ
`Rejected { error = TargetFingerprintMismatch, feature_error = None }`
として拒否します。
`expected_decl_hash = None` の場合、validator は check success response に再計算した `decl_certificate_hash` を返します。
operation は quotient declaration そのものの `decl_certificate_hash` には入りません。
operation を declaration として出力する場合は、各 operation が別の Phase 2 `decl_certificate_hash` を持つ別 declaration になります。
この candidate では operation ごとの compatibility proof を検査しますが、quotient declaration hash の再構成には
`decl_name / universe_params / params / carrier / relation / equivalence_proof` だけを使います。
payload の `quotient_type` は canonical quotient body との照合にだけ使い、DefDecl の body bytes には使いません。

MVP の quotient primitive は Phase 9 Human Profile の `Setoid` / `Quotient` primitive に合わせます。
AI validator は hidden builtin name table を使わず、`Phase9AiOptions.quotient` に明示された public imported refs だけを
primitive interface として扱います。
`setoid`、`setoid_mk`、`setoid_relation`、`rel_equiv`、`quotient`、`quotient_mk`、`quotient_sound`、
`quotient_lift` は quotient_v1 primitive refs です。
これらの ref は envelope imports から一意に解決でき、かつ解決先 certificate の feature report が `quotient_v1` を含み、
public type が `quotient_v1` の canonical interface と一致しなければなりません。
`options.quotient.eq` は quotient primitive ref ではなく equality family head ref です。
`eq` は envelope imports から一意に解決でき、public type が
`Eq.{u} : Pi A : Sort u, A -> A -> Prop`
と `quotient_public_interface_defeq` で一致しなければなりません。
`eq` の解決先 certificate に `quotient_v1` feature report は要求しません。
core term 内では、これらは通常の `Const(GlobalRef::Imported(...), level_args)` として表します。
`GlobalRef::Primitive` のような隠れた参照 variant は導入しません。
step 3 の quotient_v1 primitive refs and eq ref public interface check は payload の `carrier` や導出後の `u` / `v` に依存しません。
MVP では `quotient_v1` feature report は profile selection bit であり、certificate ごとに custom descriptor bytes を持ちません。
payload-independent primitive descriptor は本節で固定する application builder と expected public type schema そのものです。
validator はこの固定 descriptor を使い、各 ref の level argument arity、term argument order、binder dependency、result type shape を
schematic universe / binder の下で検査します。
feature report 内に caller-defined descriptor、alternate argument order、または additional computation rule が現れても
MVP validator はそれを解釈してはいけません。
それらが必要な場合は `quotient_v2` など別 feature profile と canonical hash rule を定義します。
下の builder は、その descriptor を `carrier : Sort (succ u)` と operation ごとの `result_type : Sort (succ v)` に
具体化した application schema です。
実装が step 3 で payload-specific `carrier`、`relation`、`equivalence_proof`、または operation の `result_type` を先読みして
primitive interface を判定してはいけません。
quotient primitive の core application は、すべて left-associated `App` で作ります。
`u` は carrier universe、`v` は operation result universe です。
下の builder は schema であり、validator は builder を実体化する前に、後述の規則で
`carrier : Sort (succ u)` から `u` を、各 operation の `result_type : Sort (succ v)` から `v` を導出します。
`v` は operation ごとに別々に導出され、operation を持たない quotient type declaration の hash 計算には現れません。

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

quotient_v1 primitive refs と `eq` は、この application builder で使う explicit level arg order と
term arg order の public type を持たなければなりません。
public type が上の固定 schema と definitional equality で一致しない ref は
`Rejected { error = FeatureRejected, feature_error = Some(QuotientConstruction(PrimitiveInterfaceMismatch)) }`
として拒否します。
この public interface check と、この節で `definitional equality` と書く quotient-local check は
`quotient_public_interface_defeq` profile で固定します。
この profile は β / ζ / ι reduction と、Phase 2 `DefDecl.reducibility = Reducible` の δ unfolding だけを許可します。
opaque def、opaque theorem、axiom、typeclass search、implicit insertion、AI hint、
SMT reconstruction result、または quotient_v1 の computation rule は使いません。
実装がより強い equality を使いたい場合は `quotient_v2` など別 feature profile と hash rule を定義してから有効化します。

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

validator は step 8 で `quotient_type` をまず `params` の下で type inference し、
inferred type が `Sort type_level` と `quotient_public_interface_defeq` で一致することを確認してから、
`quotient_type` 本体を `quotient_type_app(setoid_expr)` と照合します。
`quotient_type` の type inference 自体が失敗する場合は
`Rejected { error = KernelRejected, feature_error = None }`
です。
type inference は成功したが inferred type が `Sort type_level` と defeq で一致しない場合は
`Rejected { error = FeatureRejected, feature_error = Some(QuotientConstruction(QuotientTypeMismatch)) }`
です。
これは carrier から `type_level = succ u` を導出した後の quotient type shape mismatch であり、
`UniverseLevelMismatch` には分類しません。
inferred type check が成功した後に、`quotient_type` 本体が `quotient_type_app(setoid_expr)` と defeq で一致しない場合も
`Rejected { error = FeatureRejected, feature_error = Some(QuotientConstruction(QuotientTypeMismatch)) }`
です。

`Type u` は core では `Sort (succ u)` なので、surface の `A : Type u` は上の `carrier : Sort type_level` に対応します。
`u` は `universe_params` 内の level parameter、または validator が `carrier` の inferred sort から導出した level expression です。
validator は `params` の下で `carrier` の type を推論し、その sort level を canonical normalize します。
その level が byte-for-byte に `succ u` へ分解できる場合だけ `u` を採用します。
`succ _` 形へ一意に分解できない sort level、または `payload.universe_params` 外の parameter を含む `u` は
`Rejected { error = FeatureRejected, feature_error = Some(QuotientConstruction(UniverseLevelMismatch)) }`
として拒否します。
`eq_app` の第1引数は `Type` index ではなく、equality の domain type が住む core `Sort` level です。
したがって `quotient_type_app(s) : Sort type_level` に対する equality は `eq_app(type_level, ...)` を使い、
`result_type : Sort (succ v)` に対する equality は `eq_app(succ v, ...)` を使います。
`Phase9QuotientOptions` の quotient primitive refs は `quotient_v1` profile が定義する canonical primitive interface です。
`eq` はこの profile が使う explicit equality head であり、quotient_v1 primitive set には含めません。
AI payload が別の record shape、tuple、自然言語説明で equivalence を表した場合は拒否します。

MVP の `MachineQuotientOperationCandidate` は unary lift だけを表します。
`raw_function` の型は `params` の下で次へ weak-head normalize しなければなりません。

```text
raw_function :
  carrier -> result_type
```

`result_type` は `carrier` binder に依存せず、ある universe level `v` について `result_type : Sort (succ v)` でなければなりません。
ここでの unary lift は、quotient 引数を1つだけ取る operation を意味します。
`result_type` を weak-head normalize した head が `Pi` の場合、その candidate は curried multi-argument operation を表すため
MVP では
`Rejected { error = UnsupportedFeature, feature_error = None }`
として拒否します。
この Pi-head check は `raw_function` の type inference と unary `carrier -> result_type` shape check が成功した直後、
`result_type` の universe level derivation より前に行います。
したがって同じ operation で `result_type` の head が `Pi` かつ `succ v` 分解もできない場合は、
`UniverseLevelMismatch` ではなく `UnsupportedFeature` が優先です。
validator は `params` の下で `result_type` の type を推論し、その sort level を canonical normalize します。
その level が byte-for-byte に `succ v` へ分解できる場合だけ `v` を採用します。
`succ _` 形へ一意に分解できない sort level、または `payload.universe_params` 外の parameter を含む `v` は
`Rejected { error = FeatureRejected, feature_error = Some(QuotientConstruction(UniverseLevelMismatch)) }`
として拒否します。
`compatibility_proof` の期待型は次です。

```text
forall a b : carrier,
  setoid_relation_app(setoid_expr, a, b) ->
  eq_app(succ v, result_type, raw_function a, raw_function b)
```

ここでの relation premise は `payload.relation a b` の pretty form ではなく、
`setoid_expr = setoid_mk_app(carrier, relation, equivalence_proof)` から作る
`setoid_relation_app(setoid_expr, a, b)` の canonical core expression です。
`setoid_relation_app(setoid_expr, a, b)` が kernel の definitional equality で `relation a b` に簡約される場合でも、
validator は compatibility proof の期待型を上の canonical form から作ります。
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

`decl_certificate_hash` はこの `DefDecl` から Phase 2 の通常規則で計算します。
payload の `quotient_type` は採用条件の defeq check にだけ使い、DefDecl の body には入れません。
`equivalence_proof` は canonical body の `setoid_expr` 経由で DefDecl の `value_hash` と `decl_interface_hash` に反映されます。
したがって MVP の quotient declaration identity は proof-sensitive です。
同じ `carrier` と `relation` でも、`equivalence_proof` の canonical bytes が違えば別の quotient declaration hash になります。
validator は `equivalence_proof` を proof irrelevance、record projection、または proof normalization で消してはいけません。
relation だけに依存する proof-irrelevant quotient identity を導入する場合は、
`quotient_v2` のような別 feature profile と hash rule を定義してから有効化します。
`operations` を declaration として出力する future extension でも、各 operation は別 `DefDecl` とし、
`type = close_params_type(params, Pi(quotient_type_app(setoid_expr), result_type))`、
`value = close_params_value(params, quotient_lift_app(setoid_expr, result_type, raw_function, compatibility_proof))`、
`reducibility = Reducible` を使います。
この `Pi` は surface arrow ではなく Phase 2 core AST の binder であり、binder name は canonical bytes に入りません。
de Bruijn 表現では、この `Pi` の codomain は quotient argument binder の下に入るため、
上の `result_type` は `weaken(result_type, 1)` として埋め込む省略表記です。
`setoid_expr`、`result_type`、`raw_function`、`compatibility_proof` はすべて同じ `params` local context の下で検査した
core expression を使い、`close_params_type` / `close_params_value` だけがその local context を top-level declaration へ閉じます。
future extension で dependent operation を許可するまでは、この operation `Pi` の codomain `result_type` は
quotient argument binder を参照してはいけません。
MVP の `/machine/phase9/quotient/check` は operation declaration hash を返さず、compatibility 検査だけを行います。
したがって success response の `QuotientConstruction.decl_certificate_hash` は quotient type declaration だけの identity です。
`operations` が検査済みであることは、その request の `candidate_hash` / `validation_result_hash` と replay input から追跡します。
operation を standalone declaration artifact として保存・参照したい場合は、future extension で operation ごとの
`decl_certificate_hash` を返す response schema を追加してから有効化します。

QuotientConstruction の feature-specific validation 順序は固定します。
この順序は common envelope validation step 5 が終わった後に適用します。
つまり imports / options / `target.env_fingerprint` / task target shape と、
`options.quotient = Some(...)` の task options shape / semantic range validation はこの列より前に完了しています。

```text
0. MachineQuotientConstructionCandidate payload outer canonical decode / scalar field decode,
   vector length prefix cap precheck, and nested CoreExpr / LevelExpr count cap validation
1. operations sort order / duplicate name check
2. universe_params / payload-local LevelExpr universe context shape and task-local global ref absence check
3. payload CoreExpr imported-ref resolution, quotient_v1 primitive refs and eq ref import resolution,
   and public interface check
4. params telescope type check
5. carrier type inference and carrier universe level derivation
6. relation type inference and carrier -> carrier -> Prop check
7. equivalence_proof kernel check against rel_equiv_type(carrier, relation)
8. quotient_type type inference and defeq check against quotient_type_app(setoid_expr)
9. quotient DefDecl reconstruction and expected_decl_hash binding check
10. operations validation in canonical name order
   10a. raw_function type inference and unary carrier -> result_type shape check
   10b. result_type Pi-head unsupported check
   10c. result_type universe level derivation
   10d. compatibility_proof kernel check
```

step 0 で payload outer canonical decode / scalar field decode または protocol cap check に失敗した場合は、operations、
universe_params、import 解決、kernel check、expected_decl_hash は検査しません。
step 1 は import 解決や kernel check より先に実行し、違反した場合はこの節の他の feature-specific validation へ進みません。
step 2 も import 解決や kernel check より先に実行し、違反した場合は step 3 以降へ進みません。
step 3 はまず `params[*].ty`、`quotient_type`、`carrier`、`relation`、`equivalence_proof`、
および各 operation の `raw_function` / `compatibility_proof` に対して common Phase 9 wire `CoreExpr`
imported ref resolution rule を適用します。
payload `CoreExpr` の imported ref が解決できない場合は
`Rejected { error = ImportClosureMismatch, feature_error = None }`
として拒否し、quotient_v1 primitive refs / `eq` ref の public interface check や kernel check へ進みません。
payload `CoreExpr` imported ref resolution が成功した後、`options.quotient` の quotient_v1 primitive refs と
`eq` ref の import resolution / public interface check を実行します。
step 9 は operations を検査する前に実行します。
これは operations が quotient type declaration の `decl_certificate_hash` に入らないためです。
したがって `expected_decl_hash` mismatch と operation-level error が同じ request に含まれる場合は、
`TargetFingerprintMismatch` が優先されます。
step 10 では `operations` を payload の canonical name order で検査し、複数 operation に不整合がある場合は
最初の operation の最初の不整合を返します。

QuotientConstruction の拒否分類は次で固定します。

```text
TargetFingerprintMismatch:
  expected_decl_hash = Some(h) で、再構成した quotient declaration の decl_certificate_hash が h と一致しない。

ImportClosureMismatch:
  payload `CoreExpr` 内の imported ref、または `options.quotient` の quotient_v1 primitive refs / `eq` ref が
  envelope imports から一意に解決できない。
  feature_error は None。

UnsupportedFeature:
  operation.raw_function の result_type を weak-head normalize した head が Pi で、
  curried multi-argument operation を表す。feature_error は None。

FeatureRejected + QuotientConstruction(PrimitiveInterfaceMismatch):
  quotient_v1 primitive refs または `eq` ref は解決できたが、public type がこの節の fixed primitive interface と一致しない。

FeatureRejected + QuotientConstruction(QuotientTypeMismatch):
  payload.quotient_type の inferred type が Sort type_level と一致しない、または
  payload.quotient_type が quotient_type_app(setoid_expr) と quotient_public_interface_defeq で一致しない。

FeatureRejected + QuotientConstruction(RelationTypeMismatch):
  relation は params の下で type inference できたが、carrier -> carrier -> Prop と defeq で一致しない。

FeatureRejected + QuotientConstruction(RawFunctionTypeMismatch):
  operation.raw_function は params の下で type inference できたが、unary lift の carrier -> result_type 形でない、
  または result_type が carrier binder に依存する。

KernelRejected:
  params telescope の binder type が well-typed でない場合。
  carrier / relation / raw_function / quotient_type などの core expression 自体が params の下で type inference できない場合。
  carrier の type inference は成功したが、inferred type が `Sort(level)` として認識できず carrier type として扱えない場合。
  feature_error は None。

FeatureRejected + QuotientConstruction(UniverseLevelMismatch):
  carrier または operation result_type の inferred sort level は得られたが、
  `succ u` / `succ v` へ一意に分解できない、または payload.universe_params 外の parameter を含む場合。

KernelRejected + QuotientConstruction(EquivalenceProofMismatch):
  equivalence_proof が rel_equiv_type(carrier, relation) の proof term として kernel check を通らない。

KernelRejected + QuotientConstruction(CompatibilityProofMismatch):
  operation.compatibility_proof が上の unary lift compatibility proof term として kernel check を通らない。
```

採用条件:

```text
- envelope target.target_decl_hash は None である
- options.quotient が Some であり、すべての primitive refs が envelope imports から一意に解決できる
- validator が decl_name / universe_params / params / carrier / relation / equivalence_proof から
  上記の Phase 2 `DefDecl` を決定的に再構成し、
  `expected_decl_hash = Some(h)` の場合はその `decl_certificate_hash` が `h` と一致する
- quotient_type は params の下で `quotient_type_app(setoid_expr)` の canonical primitive type と definitional equality で一致する
- relation が carrier 上の relation として well-typed
- equivalence_proof が `rel_equiv_type(carrier, relation)` の proof term として kernel check を通る
- quotient primitive の intro / elim / soundness rule だけを使う
- operation ごとの raw_function は params の下で type inference され、unary lift の `carrier -> result_type` 形と
  result_type 条件を満たす
- operation ごとの compatibility_proof が上の unary lift compatibility proof term として kernel check を通る
- resulting certificate の feature report に `quotient_v1` が記録される
- selected independent checker profile が `quotient_v1` 非対応であることを certificate 実行前に判定できる場合は
  `UnsupportedFeature` として拒否する
- `quotient_v1` 対応 profile として実行した independent checker が resulting certificate を拒否した場合は
  `IndependentCheckerRejected` として拒否する
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
    rule_registry_profile: SmtRuleRegistryProfile,
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

enum SmtLogic {
    MvpQfUf,
    MvpQfLia,
    MvpQfBv,
    MvpQfUfLiaBv,
}

enum SmtEncoderVersion {
    MvpNormalizedQfV1,
}

enum SmtCertificateFormat {
    MvpProofNodeTableV1,
}

enum SmtRuleRegistryProfile {
    MvpEmptyRegistryV1,
}

struct SmtSymbol {
    ascii: Vec<u8>,
}

type SmtCommandId = Hash256;
type SmtPayloadNodeId = u32;
type SmtReconstructionStepId = u32;

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
    BuiltinApp {
        op: SmtBuiltinOp,
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

enum SmtBuiltinOp {
    IntNeg,
    IntAdd,
    IntSub,
    IntLe,
    IntLt,
    IntGe,
    IntGt,
    BvNot,
    BvAnd,
    BvOr,
    BvXor,
    BvAdd,
    BvSub,
    BvMul,
    BvUlt,
    BvUle,
    BvConcat,
    BvExtract {
        high: u32,
        low: u32,
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

MVP の encoded problem deterministic protocol cap は
`commands.len() <= 1_000_000`、encoded problem 内で reachable な `CoreExpr` node 総数 `<= 1_000_000`、
reachable な `SmtExpr` node 総数 `<= 1_000_000`、および reachable な `SmtSortExpr` node 総数 `<= 1_000_000` です。
encoded problem raw bytes cap は、Inline / Artifact とも `<= 64_000_000` です。
Inline の場合は `canonical_bytes.len()` を outer framing decode より先に検査します。
Artifact の場合は declared `size_bytes` が cap を超える時点で file read / `file_hash` check へ進まず
`Rejected { error = EnvelopeMalformed, feature_error = Some(SmtCertificate(NonCanonicalPayload)) }`
として拒否します。
declared `size_bytes` が cap 内でも実ファイル bytes の長さや `file_hash` が一致しない場合は
`Rejected { error = PayloadHashMismatch, feature_error = None }`
です。
ここでの `CoreExpr` node は `ContextAssumption.core_expr` と `TargetAssertion.core_expr` から reachable な node です。
`commands` の vector length prefix は command element decode / allocation より前に cap と照合しなければなりません。
`CoreExpr` / `SmtExpr` / `SmtSortExpr` の nested vector も decode 中に deterministic counter を進め、
length prefix を読んだ時点で残り cap を超えることが確定する場合は element decode / allocation より前に拒否します。
これらの cap を超える encoded problem は resource guard ではなく payload canonical shape violation として
`Rejected { error = EnvelopeMalformed, feature_error = Some(SmtCertificate(NonCanonicalPayload)) }`
で拒否します。
validator は server-local timeout、memory guard、runtime configuration からこの cap を増減してはいけません。

`SmtLogic` / `SmtEncoderVersion` / `SmtCommandProfile` / `SmtCertificateFormat` /
`SmtRuleRegistryProfile` canonical bytes は variant tag だけで固定します。
MVP では `SmtEncoderVersion::MvpNormalizedQfV1` と `SmtCommandProfile::MvpNormalizedQf` の組だけを受け付けます。
`SmtCertificateFormat::MvpProofNodeTableV1` はこの文書の `SmtProofNodeTable` canonical bytes だけを表します。
別 solver の native proof format を直接読む場合は、別の `SmtCertificateFormat` variant と rule registry を追加してから有効化します。
`MvpQfBv`、`MvpQfUfLiaBv`、`DatatypeDecl`、および bitvector / datatype syntax は、この MVP では
wire schema と deterministic rejection surface を固定するために存在します。
これらの variant が存在することは、NPA core expression から BV / datatype SMT problem を生成できる、または SMT success を返せることを意味しません。
実際に success path を有効化するには、対応する deterministic encoder mapping table と非空 solver-native rule registry を
同じ profile で定義しなければなりません。
`MvpQfUf` の command-level validation では、non-recursive `DatatypeDecl` は constructor / selector symbol signature を
宣言する normalized IR としてだけ受け付けます。
ここでの datatype は SMT theory axiom や NPA inductive declaration への対応を意味せず、selector / constructor の
signature table 以上の意味論を持ちません。
recursive / mutually-recursive datatype は下の symbol table 規則で `UnsupportedFeature` として拒否します。

`SmtLogic` は使える builtin theory を固定します。

```text
MvpQfUf:
  Bool, User sort, uninterpreted FunctionDecl / non-recursive DatatypeDecl signature only

MvpQfLia:
  MvpQfUf + Int sort / Int literals / FunctionDecl and DatatypeDecl fields that mention Int /
  IntNeg / IntAdd / IntSub / IntLe / IntLt / IntGe / IntGt

MvpQfBv:
  MvpQfUf + BitVec sorts / BitVec literals / FunctionDecl and DatatypeDecl fields that mention BitVec /
  BvNot / BvAnd / BvOr / BvXor / BvAdd / BvSub / BvMul / BvUlt / BvUle / BvConcat / BvExtract

MvpQfUfLiaBv:
  MvpQfUf + MvpQfLia + MvpQfBv
```

`logic` が許可しない builtin sort、literal、または theory operator を含む encoded problem / proof payload は
`Rejected { error = UnsupportedFeature, feature_error = None }`
として拒否します。
encoded problem では command-level validation 中に検査します。
ただし encoded problem の command-level validation は、`MachineSmtEncodedProblem.goal_fingerprint` と
`MachineSmtEncodedProblem.logic` の binding check が成功した後にだけ実行します。
`MachineSmtEncodedProblem.logic` が `MachineSmtCertificateCandidate.logic` と一致しない場合は、
commands 内の builtin が payload selected logic で許可されるかどうかを検査せず、
下の `EncodingMismatch` を返します。
proof payload では `SmtConclusionEncoding.encoded_expr` の pre-registry validation 中に、
request payload の selected logic である `MachineSmtCertificateCandidate.logic` に対して
builtin sort / literal / `SmtBuiltinOp` の allowlist だけを検査します。
pre-registry validation は `SmtConclusionEncoding.logic` を allowlist selector として使いません。
`SmtConclusionEncoding.logic` と encoded problem logic の一致検査は、下の `SmtConclusionEncoding` semantic validation です。
この proof payload 側の logic allowlist violation は、同じ request に `PayloadNode` が存在しても
`RuleRegistryMismatch` より先に `UnsupportedFeature + None` として拒否します。
ここでの theory operator は `SmtBuiltinOp` と builtin sort/literal の許可範囲だけを指します。
solver-native proof rule の可否は `SmtLogic` ではなく下の closed SMT rule registry で判定し、
registry missing / unsupported rule は
`Rejected { error = UnsupportedFeature, feature_error = Some(SmtCertificate(RuleRegistryMismatch)) }`
として拒否します。
`BoolLit`、`Not`、`And`、`Or`、`Imp`、`Eq`、`Ite` は SMT normalized IR の core expression form であり、
operand sort が well-formed であればすべての `SmtLogic` で使えます。
これらは `SmtBuiltinOp` ではなく、`SmtLogic` の theory operator allowlist にも入りません。
ただし `Eq` / `Ite` の operand sort が `Int` や `BitVec` を含む場合、その sort 自体は選択された `SmtLogic` で
許可されていなければなりません。
MVP の builtin theory operator は `SmtExpr::BuiltinApp` だけで表します。
`SmtExpr::App` は `FunctionDecl`、datatype constructor、datatype selector の application 専用であり、
`+`、`<`、bitvector operation などの theory operator を `SmtSymbol` で表してはいけません。
`SmtBuiltinOp` canonical bytes は variant tag と variant field order だけで固定します。
`IntAdd` / `IntSub` は arity 2 以上の Int operands を取り Int を返します。
`IntNeg` は arity 1 の Int operand を取り Int を返します。
`IntLe` / `IntLt` / `IntGe` / `IntGt` は arity 2 の Int operands を取り Bool を返します。
MVP LIA では non-linear multiplication を表す builtin operator を持ちません。
`BvNot` は arity 1、`BvAnd` / `BvOr` / `BvXor` / `BvAdd` / `BvSub` / `BvMul` / `BvUlt` / `BvUle` は
arity 2 の同じ width の BitVec operands を取ります。
`BvNot` / bitvector arithmetic / bitwise operators は同じ width の BitVec を返し、
`BvUlt` / `BvUle` は Bool を返します。
`BvConcat` は arity 2 の BitVec operands を取り、width の和の BitVec を返します。
`BvExtract { high, low }` は arity 1 の BitVec operand を取り、
`0 <= low <= high < operand_width` を満たす場合だけ width `high - low + 1` の BitVec を返します。
`SmtSymbol.ascii` は normalized IR 内の symbol identity であり、raw SMT-LIB identifier ではありません。
MVP では長さ `1..=128` の ASCII bytes で、正規表現 `[A-Za-z_][A-Za-z0-9_.$:-]*` に一致しなければなりません。
canonical bytes は tag `"npa.phase9_ai.smt.symbol.v1"` と raw ASCII bytes の length-prefixed encoding だけです。
Unicode normalization、case folding、SMT-LIB escaping、solver-specific quoting は行いません。
`lc:` と `sk:` で始まる symbols は deterministic encoder 専用 prefix です。
`SmtExpr::Var.symbol` が encoder table から再生成された場合だけこれらの prefix を使えます。
`SortDecl.symbol`、`FunctionDecl.symbol`、`DatatypeDecl.symbol`、constructor / selector symbol、`SmtExpr::App.symbol` は
`lc:` または `sk:` で始まってはいけません。
この prefix violation は reserved name misuse と同じ command canonical shape violation として扱います。
さらに `SmtSymbol.ascii` は次の reserved theory names と bytewise equal であってはいけません。
この list は raw SMT-LIB の完全な予約語表ではなく、MVP normalized IR で builtin と混同しうる pretty/operator alias の閉じた集合です。

```text
Bool
Int
BitVec
true
false
not
and
or
implies
eq
ite
int.neg
int.add
int.sub
int.le
int.lt
int.ge
int.gt
bvnot
bvand
bvor
bvxor
bvadd
bvsub
bvmul
bvult
bvule
bvconcat
bvextract
```

`SmtReconstructionStepId` は `reconstruction_plan` 内だけで有効な payload-local `u32` label です。
MVP では validator は step id を振り直して reconstruction plan を正規化してはいけません。
MVP の reconstruction plan deterministic protocol cap は `steps.len() <= 1_000_000` です。
加えて、`imported_theory_refs.len() <= 65_536`、各 step の `payload_bindings.len() <= 65_536`、
`premises.len() <= 65_536`、`ReorderPremises.permutation.len() <= 65_536`、
`IntroduceTheoryLemma.level_args.len() <= 65_536`、`IntroduceTheoryLemma.term_args.len() <= 65_536`、
`ComposeProof.level_args.len() <= 65_536`、`ComposeProof.term_args.len() <= 65_536` です。
reconstruction plan 内で reachable な `CoreExpr` node 総数は、`final_proof`、全 step の `conclusion` / `proof`、
および LocalBookkeeping の `term_args` を合わせて `<= 1_000_000` でなければなりません。
reconstruction plan 内で reachable な `LevelExpr` node 総数は、LocalBookkeeping の `level_args` と
上記 `CoreExpr` 内の explicit level args を合わせて `<= 1_000_000` でなければなりません。
この cap を超える reconstruction plan は resource guard ではなく payload canonical shape violation として
`Rejected { error = EnvelopeMalformed, feature_error = Some(SmtCertificate(NonCanonicalPayload)) }`
で拒否します。
validator は各 vector length prefix を element decode / allocation より前にこの cap と照合しなければなりません。
nested `CoreExpr` / `LevelExpr` node cap は full canonical decode 中に deterministic counter で検査し、
length prefix または node tag を読んだ時点で残り cap を超えることが確定する場合は nested element allocation より前に拒否します。
ただし任意の sparse id や並び替えによる表現揺れを避けるため、
`steps[k].step_id == k as u32` でなければならず、`step_id` は 0 から `steps.len() - 1` まで連続していなければなりません。
`steps` は `step_id` 昇順で strictly sorted され、重複 step id を含んではいけません。
MVP では `u32::MAX` を valid step id として使わず、許可する step id は `0..steps.len()` です。
`steps.len() > 1_000_000`、または `step_id` が contiguous index と一致しない reconstruction plan は
`Rejected { error = EnvelopeMalformed, feature_error = Some(SmtCertificate(NonCanonicalPayload)) }`
として拒否します。
各 `premises` entry は現在の `step_id` より小さい既出 step だけを参照でき、
`final_step` は `0 <= final_step < steps.len()` を満たさなければなりません。
`steps` の連続性 / sort order violation、未来 step を指す `premises`、または範囲外の `final_step` は
`Rejected { error = EnvelopeMalformed, feature_error = Some(SmtCertificate(NonCanonicalPayload)) }`
として拒否します。
同じ reconstruction DAG でも step id や独立 step の順序が異なる payload は、別の `candidate_hash` を持つ別 candidate として扱います。
reconstruction plan 内の `CoreExpr`、つまり `final_proof`、全 step の `conclusion` / `proof`、
および LocalBookkeeping の `term_args` は、Phase 9 common wire `CoreExpr` として検査します。
これらの `CoreExpr` 内で `GlobalRef::Local(_)` または `GlobalRef::LocalGenerated` を検出した場合は、
SMT reconstruction plan では task-local global ref の意味を定義していないため
`Rejected { error = EnvelopeMalformed, feature_error = Some(SmtCertificate(NonCanonicalPayload)) }`
として拒否します。
この task-local global ref absence check は reconstruction plan canonical shape validation の一部であり、
imported-ref resolution より先に行います。
task-local global ref absence check が通った後、これらの `CoreExpr` 内の `GlobalRef::Imported(import_index, name, decl_interface_hash)` は
common Phase 9 wire `CoreExpr` imported ref resolution rule で解決できなければなりません。
解決できない場合は
`Rejected { error = ImportClosureMismatch, feature_error = None }`
として拒否し、kernel check や proof reconstruction には進みません。
走査順は `final_proof`、各 step を `step_id` 昇順で `conclusion`、`proof`、LocalBookkeeping `term_args` payload order の順です。
同じ request に task-local global ref と unresolved imported ref が同時にある場合は、task-local global ref の
`EnvelopeMalformed + SmtCertificate(NonCanonicalPayload)` が優先です。
`reconstruction_plan.imported_theory_refs` は `Phase9AiGlobalRef` tuple key で strictly sorted され、
重複を含んではいけません。
この list は local bookkeeping step が参照する theorem / combinator の closed allowlist です。
`IntroduceTheoryLemma.lemma` と `ComposeProof.combinator` で実際に使われる ref の集合と
`imported_theory_refs` の集合は byte-for-byte に一致しなければなりません。
重複 ref、sort order violation、および `imported_theory_refs` に含まれるが実際には使われない unused ref は
`Rejected { error = EnvelopeMalformed, feature_error = Some(SmtCertificate(NonCanonicalPayload)) }`
として拒否します。
local bookkeeping step が `imported_theory_refs` に含まれない ref を参照する場合は
`Rejected { error = FeatureRejected, feature_error = Some(SmtCertificate(TheoryRefMismatch)) }`
として拒否します。
`imported_theory_refs` 内の ref が envelope imports から一意に解決できない場合は
`Rejected { error = ImportClosureMismatch, feature_error = None }`
として拒否します。
`imported_theory_refs` exactness / resolution の suborder は次で固定します。

```text
9a. imported_theory_refs sort order / duplicate check
9b. LocalBookkeeping refs membership check, scanning steps in canonical order
9c. unused imported_theory_refs check
9d. imported_theory_refs import resolution, in list order
```

したがって同じ request に `imported_theory_refs` の unused ref と、list に含まれない LocalBookkeeping ref が同時にある場合は、
9b の `TheoryRefMismatch` が優先です。
9b が成功した後にだけ unused ref を検査するため、unused unresolved ref は 9c の
`EnvelopeMalformed + SmtCertificate(NonCanonicalPayload)` として拒否し、9d の `ImportClosureMismatch` には進みません。
この reconstruction plan の step count cap / step id / premises / `final_step` /
reconstruction plan `CoreExpr` ref shape / import 解決、`imported_theory_refs` canonical shape、used-ref exactness / import 解決、
および LocalBookkeeping step が `payload_bindings` を持たないことの検査は、
MVP empty registry の `PayloadNode` 有無による早期 rejection より先に行います。
したがって同じ request に invalid `final_step`、local bookkeeping ref の不整合、LocalBookkeeping の non-empty `payload_bindings`、
および `PayloadNode` が同時に含まれる場合は、この段落の `EnvelopeMalformed` / `TheoryRefMismatch` /
`ImportClosureMismatch` / `PayloadBindingMismatch` が優先されます。

SmtCertificate の feature-specific validation 順序は固定します。
この順序は common envelope validation step 5 が終わった後に適用します。
`MachineSmtCertificateCandidate` payload bytes の task-specific decode はこの feature-specific validation の step 0 であり、
common envelope validation step 3 の `target.env_fingerprint` mismatch より先に実行してはいけません。

```text
0. MachineSmtCertificateCandidate payload outer canonical framing / enum tag / scalar field decode,
   reconstruction_plan vector length prefix cap precheck, and nested reconstruction_plan
   CoreExpr / LevelExpr count cap validation
1. payload.goal canonical decode and target.goal_fingerprint binding check
2. Phase9AiGoal wire shape / universe context shape and imported-ref resolution check
3. Phase9AiGoal local_context / target kernel well-typedness check
4. encoded_problem Inline/Artifact bytes acquisition, raw byte / declared size cap check,
   Artifact file_hash / size_bytes check, outer framing / vector cap precheck,
   full canonical decode, problem_hash / encoding_hash recomputation and comparison
5. encoded_problem goal_fingerprint / logic binding check
6. options.smt import / public interface check
7. encoded problem command-level validation
8. proof_payload validation order
9. reconstruction plan canonical shape / final_step bounds / CoreExpr task-local ref absence and imported-ref resolution /
   imported_theory_refs exactness / import resolution
10. LocalBookkeeping pre-registry structural checks
11. PayloadNode existence and rule registry check
12. non-empty registry reconstruction, final proof kernel check, and response construction
```

step 0 で payload outer decode または reconstruction plan protocol cap check に失敗した場合は、
`payload.goal`、encoded problem artifact、proof payload artifact、reconstruction step の semantic validation は検査しません。
step 1 で `target.goal_fingerprint` mismatch と判定した場合は、encoded problem bytes の取得や hash 検査には進みません。
したがって同じ request に payload goal binding mismatch と `encoded_problem.problem_hash` / `encoding_hash` mismatch が同時にある場合は
`TargetFingerprintMismatch` が優先です。
step 4 の encoded problem bytes validation で `problem_hash` / `encoding_hash` mismatch と判定した場合は、
encoded problem 内の `goal_fingerprint` / logic binding、`options.smt` import / public interface check、
command-level validation、proof payload validation には進みません。
step 5 の encoded problem `goal_fingerprint` mismatch または logic mismatch は、step 6 の `options.smt` import / public interface check より優先します。
この文書の MVP empty registry では step 12 の success path には到達せず、step 11 で
`UnsupportedFeature + SmtCertificate(RuleRegistryMismatch)` または `UnsupportedFeature + None` として拒否します。

`encoded_problem` は replay input の一部です。
validator は `Inline.canonical_bytes` または `Artifact` の `path` / `file_hash` で固定された bytes から
`problem_hash` と `encoding_hash` を再計算します。SMT solver process や solver log から problem を補完してはいけません。
`encoded_problem` bytes は `MachineSmtEncodedProblem` として canonical decode できなければなりません。
encoded problem bytes validation は、Inline bytes acquisition / byte cap check、
または Artifact path / declared size cap / file read / file_hash / size_bytes check の後、
`MachineSmtEncodedProblem` outer canonical framing / scalar field decode と `commands` length prefix cap precheck を行い、
cap を超える場合は command element decode / allocation に進まず
`Rejected { error = EnvelopeMalformed, feature_error = Some(SmtCertificate(NonCanonicalPayload)) }`
として拒否します。
outer framing と cap precheck が成功した後、full canonical decode と `CoreExpr` / `SmtExpr` / `SmtSortExpr` node count cap validation を行い、
decode 中に cap 超過が確定した場合は nested element allocation に進まず拒否します。
full decode と cap validation が成功した後にだけ `problem_hash` / `encoding_hash` を再計算します。
再計算した `problem_hash` または `encoding_hash` が `MachineSmtProblemRef` 内の値と一致しない場合は
`Rejected { error = PayloadHashMismatch, feature_error = None }`
として拒否し、encoded problem の `goal_fingerprint` / logic binding check、`options.smt` import / public interface check、
command-level validation、および proof payload validation より優先します。
outer framing / count cap / full canonical decode failure は hash recomputation に到達しないため、
同じ bytes に hash mismatch も含まれる場合は
`EnvelopeMalformed + SmtCertificate(NonCanonicalPayload)` が `PayloadHashMismatch` より優先します。

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
`MachineSmtEncodedProblem.goal_fingerprint` が一致しない場合は
`Rejected { error = TargetFingerprintMismatch, feature_error = None }`
として拒否します。
`MachineSmtEncodedProblem.logic` が payload の `logic` と一致しない場合は、encoded problem が別 logic 向けに作られているため
`Rejected { error = FeatureRejected, feature_error = Some(SmtCertificate(EncodingMismatch)) }`
として拒否します。
この logic mismatch check は command-level validation より先に行います。
同じ encoded problem が payload selected logic では許可されない builtin sort / literal / operator を含んでいても、
`MachineSmtEncodedProblem.logic` が payload の `logic` と一致しない場合は `EncodingMismatch` が優先です。
logic が一致した後、command-level validation は payload selected logic と同一になった `MachineSmtEncodedProblem.logic` に対して
builtin sort / literal / `SmtBuiltinOp` allowlist を検査します。
command-level validation 内では、selected logic allowlist より先に representation-only canonical shape を検査します。
ここでの representation-only shape は、command phase order、symbol grammar / uniqueness、sort arity、
`SmtExpr` / `SmtSortExpr` の variant field order、`BitVecLit` の fixed-width 表現、datatype constructor / selector signature shape、
および `CoreExpr` / `SmtExpr` / `SmtSortExpr` node count cap です。
同じ command に representation-only shape violation と selected logic unsupported が同時にある場合は、
`Rejected { error = EnvelopeMalformed, feature_error = Some(SmtCertificate(NonCanonicalPayload)) }`
が優先します。
representation-only shape が canonical だが selected `SmtLogic` が builtin sort / literal / operator を許可しない場合だけ
`Rejected { error = UnsupportedFeature, feature_error = None }`
として拒否します。
recursive / mutually-recursive `DatatypeDecl` は symbol / selector signature shape が canonical であることを確認した後の
profile unsupported case なので、`UnsupportedFeature` として拒否します。
`commands` は raw SMT-LIB command 列ではなく、`encoder_version` と `command_profile` が定義する normalized IR の列です。
`SmtEncoderVersion` と `SmtCommandProfile` は validator profile 内の closed deterministic encoder table を指します。
この table は、対応する `CoreExpr` variant、対応する imported constant、SMT symbol derivation、sort mapping、
および theory side condition generation を明示的に定義します。
table にない `CoreExpr` variant、`GlobalRef`、level pattern、または theory mapping を見つけた場合、
validator は AI annotation、pretty name、solver log、または hidden standard library registry から推測せず
`UnsupportedFeature` として拒否します。
`Var.symbol` と、`ContextAssumption` / `TargetAssertion` の `encoded_expr` は deterministic encoder table と
goal / import identity から再生成できなければなりません。
caller が任意に選んだ variable symbol や encoded assertion を受け入れてはいけません。
一方で MVP rejection-surface profile の `SortDecl`、`FunctionDecl`、`DatatypeDecl` は、
normalized IR 内の明示 declaration として grammar、重複、signature、logic allowlist だけを検査します。
これらの declaration symbol は、任意の NPA constant から生成されたことを意味しません。
MVP encoder table に mapping がない core expression がこれらの user sort / function / datatype symbol を必要とする場合、
その command は well-formed な declaration を含んでいても下の `UnsupportedFeature` または `EncodingMismatch` で拒否します。
この文書の `MvpNormalizedQfV1` encoder table は、deterministic rejection surface 用の最小 profile です。
具体的には、この SMT 節で明示している command shape validation、symbol grammar、`options.smt.eq` /
`prop_false` / `prop_not` の recognition、local context assumption と target refutation wrapper の検査だけを持ちます。
任意の NPA constant、Nat-to-Int embedding、LIA/BV operator、datatype projection、または user sort/function symbol を
core term から生成する mapping table は、この MVP schema では未定義です。
そのため `MvpNormalizedQfV1` が明示 mapping を持たない `CoreExpr` / `GlobalRef` / level pattern /
theory side condition を encoding しようとする request は、well-formed な SMT IR であっても
`Rejected { error = UnsupportedFeature, feature_error = None }`
として拒否します。
SMT certificate success を有効化する profile では、非空 rule registry だけでなく、この encoder table の supported mapping も
同じ schema/profile 内で閉じて定義しなければなりません。
MVP の `command_profile = MvpNormalizedQf` では、push/pop、incremental assertion、solver option、named assertion side effect を禁止します。
`SmtEncodedCommand canonical bytes` は `phase / command_id / payload` の順に encoding します。
`SmtEncodedCommand.phase` と `SmtEncodedCommand.payload` の variant は一致しなければなりません。
たとえば `phase = SortDecl` かつ `payload = FunctionDecl { ... }` の command は
`Rejected { error = EnvelopeMalformed, feature_error = Some(SmtCertificate(NonCanonicalPayload)) }`
として拒否します。
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
command list validation の順序は固定します。
まず各 command の canonical decode と `phase` / `payload` variant 一致だけを検査します。
この段階では `command_id_source_key` を取り出せる最小構造だけを要求し、
symbol ASCII grammar、reserved name / prefix、`ContextAssumption.source_local_index` bounds、source key duplicate、`command_id` duplicate、
phase order、phase 内 sort order、symbol table / sort validation、`SmtExpr` の representation-only canonical shape
（`And` / `Or` operand sort order / duplicate、operand count、`BitVecLit` fixed-width 表現など）はまだ検査しません。
次に、input `commands` 配列の順番で各 command の `command_id` を payload から再計算し、
保存されている `command_id` と一致しないものを見つけた時点で
`Rejected { error = PayloadHashMismatch, feature_error = None }`
を返します。
この `command_id` recomputation mismatch は、`ContextAssumption.source_local_index` bounds、source key duplicate、`command_id` duplicate、phase order、
command list sort order violation、symbol / sort / expression validation より優先します。
すべての `command_id` が再計算値と一致した後にだけ、
validator は再計算済み `command_id` を sort key として source key uniqueness、`command_id` uniqueness、
phase order、phase 内 sort order、その後に source binding / symbol table / sort / expression validation を検査します。
この時点では保存された `command_id` と再計算済み `command_id` が byte-for-byte に一致しているため、
`SmtEncodedCommand canonical bytes` 内の `command_id` も stable sort key と同じ値です。

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
mutually-recursive datatype として
`Rejected { error = UnsupportedFeature, feature_error = None }`
で拒否します。

`ContextAssumption.source_local_index` は `goal.local_context` 配列の 0-based index です。
これは de Bruijn index ではありません。`source_local_index = i` が指す local binder は `goal.local_context[i]` であり、
core term 内で同じ binder を参照する de Bruijn index は `goal.local_context.len() - 1 - i` です。
`source_local_index >= goal.local_context.len()` の command は source binding の canonical shape violation として
`Rejected { error = EnvelopeMalformed, feature_error = Some(SmtCertificate(NonCanonicalPayload)) }`
で拒否します。
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
一致しない場合は、deterministic encoder がその assertion command を生成しないため
`Rejected { error = FeatureRejected, feature_error = Some(SmtCertificate(EncodingMismatch)) }`
として拒否します。
同じ command に `command_id` recomputation mismatch もある場合は、上の command list validation order により
`PayloadHashMismatch` が優先です。
`local_type_assumption(i)` を使う場合、その expression は full goal context の下で `Prop` として well-typed でなければなりません。
`decl.ty` が proposition でない local binder は SMT assertion として出してはいけません。
`ContextAssumption.core_expr = local_type_assumption(i)` だが `decl.ty` が full goal context の下で `Prop` として
well-typed でない場合は、deterministic encoder がその assertion command を生成しないため
`Rejected { error = FeatureRejected, feature_error = Some(SmtCertificate(EncodingMismatch)) }`
として拒否します。
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
この `options.smt.eq` / `prop_false` / `prop_not` の public interface check は
`smt_public_interface_defeq` profile で固定します。
SmtCertificate の feature-specific validation order では、payload goal binding と Phase9AiGoal validation、
encoded problem の artifact/hash decode、encoded problem 内の `goal_fingerprint` binding check、
および logic mismatch check が終わった後、
command-level validation、proof payload artifact/hash/decode、proof payload table shape validation、
reconstruction plan validation のいずれにも入る前に
この `options.smt` import / public interface check を実行します。
したがって同じ request に `options.smt.eq` の import 解決失敗または public interface mismatch と
SMT `command_id` recomputation mismatch、proof payload hash / file_hash / size mismatch、
proof payload malformed、または reconstruction plan import failure が同時にある場合は、
`ImportClosureMismatch` または `SmtCertificate(PublicInterfaceMismatch)` が優先です。
encoded problem の `goal_fingerprint` mismatch または logic mismatch が同時にある場合は、それらがこの options check より先です。
これらの ref が envelope imports から一意に解決できない場合は
`Rejected { error = ImportClosureMismatch, feature_error = None }`
として拒否します。
ref は解決できたが public type が上の interface と一致しない場合は
`Rejected { error = FeatureRejected, feature_error = Some(SmtCertificate(PublicInterfaceMismatch)) }`
として拒否します。
この profile は β / ζ / ι reduction と、Phase 2 `DefDecl.reducibility = Reducible` の δ unfolding だけを許可します。
opaque def、opaque theorem、axiom、typeclass search、implicit insertion、AI hint、
SMT solver result、または quotient_v1 の computation rule は使いません。
`options.smt.eq` は `Phase9SmtOptions` の必須 field であり、`SmtCertificate` task で `options.smt = Some(...)` が
存在する限り `None` にはなりません。
`prop_false` / `prop_not` は optional recognition head です。
`options.smt.prop_false = None` または `options.smt.prop_not = None` の profile で、それぞれ core proposition 側の
`False` / `Not` recognition を必要とする encoded conclusion や refutation bridge が現れた場合は
`Rejected { error = UnsupportedFeature, feature_error = None }`
として拒否します。
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
`And` / `Or` の operands は `len >= 2` でなければなりません。
空の conjunction / disjunction と、1要素だけを包む conjunction / disjunction は MVP normalized IR では non-canonical です。
`And` / `Or` operands vector は、encoded problem の `SmtExpr` node count cap の対象であり、
length prefix を読んだ時点で残り node budget を超えることが確定する場合は operand element decode / allocation より前に拒否します。
この arity violation は
`Rejected { error = EnvelopeMalformed, feature_error = Some(SmtCertificate(NonCanonicalPayload)) }`
として拒否します。
`FunctionDecl.args`、`App.args`、datatype constructor list、selector list は source encoding order を持つため、入力順を canonical order とします。
`BitVecLit.value` は最小 byte length ではなく、`width` から決まる fixed byte length の big-endian bytes です。
`width` が 8 の倍数でない場合、先頭 byte の未使用 high bits はすべて 0 でなければなりません。
つまり `BitVecLit.value` が表す unsigned integer は `0 <= value < 2^width` を満たし、
同じ bitvector 値に複数の byte 表現を許してはいけません。
MVP では binder 名、pretty text、solver-generated temporary name を含めません。

validator は `commands` から deterministic symbol table を作り、encoded problem 内のすべての `SmtExpr` / `SmtSortExpr` を検査します。
proof payload 側の `SmtConclusionEncoding.encoded_expr` は、後述する proof payload validation order に従います。

```text
sort symbol table:
  - Bool / Int / BitVec are builtin sorts
  - SortDecl.symbol is unique across SortDecl and distinct from FunctionDecl / DatatypeDecl / constructor / selector symbols
  - SortDecl.arity is the required number of User.args
  - DatatypeDecl.symbol also introduces a User sort with arity 0
  - User(symbol, args) is valid only if symbol exists in SortDecl with args.len = arity,
    or symbol exists in DatatypeDecl with args.len = 0

function symbol table:
  - FunctionDecl.symbol is unique across FunctionDecl and distinct from SortDecl / DatatypeDecl / constructor / selector symbols
  - FunctionDecl args/result sorts are valid
  - App(symbol, args, result_sort) using a function symbol is valid only if
    args length and each arg sort match the declared signature and result_sort matches declared result
  - App(symbol, args, result_sort) using a constructor or selector symbol is checked against the
    signature derived from its DatatypeDecl
  - BuiltinApp(op, args, result_sort) is checked only against the SmtBuiltinOp table and the selected SmtLogic
  - App must not use reserved theory names, reserved encoder prefixes, or pretty names for builtin theory operators

datatype symbol table:
  - DatatypeDecl.symbol is unique across DatatypeDecl and distinct from SortDecl / FunctionDecl / constructor / selector symbols
  - constructor symbols are globally unique across all DatatypeDecl and distinct from SortDecl / FunctionDecl / DatatypeDecl / selector symbols
  - selector symbols are globally unique across all DatatypeDecl and distinct from SortDecl / FunctionDecl / DatatypeDecl / constructor symbols
  - constructor / selector signatures are derived only from DatatypeDecl payload
  - MVP rejects recursive and mutually-recursive datatype declarations as UnsupportedFeature; selector sorts must not refer to any DatatypeDecl symbol

variable symbol table:
  - Var symbols come only from deterministic encoder output for goal.local_context binders and target-local skolem symbols
  - the same Var symbol must always have byte-identical SmtSortExpr
  - caller-provided fresh variable names outside the encoder table are rejected
```

MVP normalized IR は、`SmtExpr::Var.symbol` を除く declaration / application symbols に単一の global namespace を使います。
`SortDecl`、`FunctionDecl`、`DatatypeDecl`、constructor、selector のいずれか2つが bytewise equal な `SmtSymbol` を使う場合は、
variant が違っていても duplicate symbol として
`Rejected { error = EnvelopeMalformed, feature_error = Some(SmtCertificate(NonCanonicalPayload)) }`
で拒否します。
`Var.symbol` は encoder table が作る variable namespace であり、この declaration-symbol namespace には入りません。

MVP の deterministic encoder は local context 由来の variable symbol を次で作ります。

```text
local_var_symbol(i) =
  SmtSymbol(ascii = "lc:" || decimal_u32(i))
```

ここで `i` は `goal.local_context` の 0-based `source_local_index` であり、de Bruijn index ではありません。
`decimal_u32(i)` は leading zero を持たない base-10 ASCII 表現です。`i = 0` の場合だけ `"0"` を使います。
`goal.local_context` から SMT sort を導出できない local declaration は variable symbol table に入りません。
target-local skolem は MVP では導入しません。
encoder が将来 skolemization profile を追加する場合は、`sk:` prefix と source core occurrence hash などから成る
canonical `SmtSymbol` 生成規則をその profile に定義しなければなりません。
MVP で `lc:` prefix 以外の `Var` symbol が現れた場合、または `lc:i` が encoder table に存在しない場合は
`Rejected { error = FeatureRejected, feature_error = Some(SmtCertificate(EncodingMismatch)) }`
として拒否します。

`Not` / `And` / `Or` / `Imp` require Bool operands and return Bool.
`Eq(lhs, rhs)` requires lhs and rhs to have byte-identical sorts and returns Bool.
`Ite` requires Bool condition and byte-identical then/else sorts, and returns that branch sort.
`BitVec.width` and `BitVecLit.width` must be greater than 0, and `BitVecLit.value.len()` must equal `ceil(width / 8)`.
validator は deterministic encoder を再実行し、`encoded_expr` を持つ assertion command だけに expected encoded expression を作ります。
`ContextAssumption` では `expected = encode(ContextAssumption.core_expr)` です。
`TargetAssertion` では `core_expr` が `goal.target` と一致することを先に確認し、
MVP refutation mode の `expected = Not(encode(TargetAssertion.core_expr))` とします。
payload の `encoded_expr` はこの `expected` と canonical bytes 一致しなければなりません。
一致しない command は
`Rejected { error = FeatureRejected, feature_error = Some(SmtCertificate(EncodingMismatch)) }`
として拒否します。
この phase 順を満たさない bytes は canonical decode 後に
`Rejected { error = EnvelopeMalformed, feature_error = Some(SmtCertificate(NonCanonicalPayload)) }`
として拒否します。

SMT command-level validation の拒否分類は次で固定します。

```text
EnvelopeMalformed + SmtCertificate(NonCanonicalPayload):
  - commands / CoreExpr nodes / SmtExpr nodes / SmtSortExpr nodes protocol cap violation
  - phase / payload variant mismatch
  - command list phase order violation
  - command_id_source_key duplicate
  - command_id duplicate
  - ContextAssumption.source_local_index is outside goal.local_context
  - TargetAssertion missing / duplicate
  - FinalCheck missing / duplicate / not last
  - canonical sort order violation in command phases or commutative operands
  - symbol ASCII grammar violation, reserved theory name misuse, or reserved encoder prefix misuse
  - duplicate sort / function / datatype / constructor / selector symbol
  - unknown sort / function / constructor / selector symbol
  - invalid sort arity or function signature mismatch
  - BitVec width = 0, wrong BitVecLit byte length, or non-zero unused high bits
  - Not / And / Or / Imp / Eq / Ite / BuiltinApp operand sort mismatch
  - And / Or operand count is less than 2

PayloadHashMismatch:
  - command_id recomputation mismatch

FeatureRejected + SmtCertificate(EncodingMismatch):
  - deterministic encoder expected expression differs from payload encoded_expr
  - ContextAssumption.core_expr is neither local_type_assumption(i) nor local_value_equality(i)
  - ContextAssumption uses local_type_assumption(i) for a local binder whose type is not Prop
  - TargetAssertion.core_expr is not byte-identical to payload.goal.target
  - deterministic encoder table would generate a different Var symbol or encoded assertion than the payload contains
  - caller-provided Var symbol is not one of the deterministic encoder output symbols

UnsupportedFeature:
  - selected encoder / command_profile intentionally has no mapping for a well-formed core expression,
    theory operator, or refutation bridge shape
  - recursive or mutually-recursive DatatypeDecl selector sort
```

`TargetAssertion` の `Not(...)` は encoded problem command の refutation wrapper であり、
`goal.target` 自体を SMT assertion として証明済みにする規則ではありません。
validator は `TargetAssertion.core_expr = goal.target` を、そのまま `goal.target` の proof premise として扱ってはいけません。
proof payload 側で `TargetAssertion.encoded_expr` と同じ SMT formula を参照する場合、その
`SmtConclusionEncoding.core_expr` は deterministic encoder で `Not(encode(goal.target))` に対応する
well-typed core proposition でなければなりません。
MVP でこの proposition を作る場合、validator は `options.smt.prop_not = Some(ref)` を要求し、
`SmtConclusionEncoding.core_expr` に `weak_head_normalize_for_smt_encoder` を適用した結果が
`App(Const(resolve_imported_ref(ref), []), goal.target)` と canonical bytes で完全一致する場合だけ、
`Not(encode(goal.target))` の対応物として認めます。
ここでは definitional equality search、δ unfolding、ι reduction、quotient computation rule、または typeclass search を使って
`Not goal` を合成してはいけません。
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
MVP の proof payload raw bytes cap は、Inline / Artifact とも `<= 64_000_000` です。
Inline の場合は `canonical_bytes.len()` を outer framing decode より先に検査します。
Artifact の場合は declared `size_bytes` が cap を超える時点で file read / `file_hash` check へ進まず
`Rejected { error = EnvelopeMalformed, feature_error = Some(SmtCertificate(NonCanonicalPayload)) }`
として拒否します。
declared `size_bytes` が cap 内でも実ファイル bytes の長さや `file_hash` が一致しない場合は
`Rejected { error = PayloadHashMismatch, feature_error = None }`
です。
この proof payload bytes validation は、`options.smt` import / public interface check と
command-level validation が成功した後にだけ実行します。
したがって `options.smt` の import 解決失敗または public interface mismatch は、
proof payload の `PayloadHashMismatch` / malformed rejection より優先します。
command-level validation の `EnvelopeMalformed` / `PayloadHashMismatch` も proof payload validation より優先です。
proof payload validation 内の deterministic order は次で固定します。

```text
0. Inline bytes acquisition and byte cap check, or Artifact path validation /
   declared size cap check / file read / file_hash / size_bytes check
1. SmtProofNodeTable outer canonical framing precheck
   - table tag / field boundary / scalar field decode
   - nodes vector length prefix and node count cap
2. payload_hash recomputation and comparison
3. full SmtProofNodeTable canonical decode, nested CoreExpr / SmtExpr / SmtSortExpr count cap validation,
   and table shape validation
```

step 0 で proof payload bytes cap を超える場合は
`Rejected { error = EnvelopeMalformed, feature_error = Some(SmtCertificate(NonCanonicalPayload)) }` です。
step 0 で `Artifact.file_hash` / `size_bytes` が一致しない場合は
`Rejected { error = PayloadHashMismatch, feature_error = None }` です。
path validation を通過した artifact bytes を取得できない場合は `Error::ArtifactUnavailable` です。
step 1 の canonical framing violation、または `nodes.len() > 1_000_000` は
`Rejected { error = EnvelopeMalformed, feature_error = Some(SmtCertificate(NonCanonicalPayload)) }`
として拒否し、`payload_hash` mismatch より優先します。
step 1 が成功した後に `payload_hash` が一致しない場合は
`Rejected { error = PayloadHashMismatch, feature_error = None }` です。
step 2 の hash input は、step 1 の outer canonical framing precheck を通過した provided bytes そのものです。
この時点では nested `SmtConclusionEncoding` まで full decode していないため、validator は table を再serializeして
hash input を作ってはいけません。
下の式の `canonical_bytes(SmtProofNodeTable)` は「step 3 で full canonical decode できなければならない provided bytes」を意味します。
step 2 で `payload_hash` が一致しても、step 3 の full canonical decode / table shape validation が失敗した場合は
`Rejected { error = EnvelopeMalformed, feature_error = Some(SmtCertificate(NonCanonicalPayload)) }`
です。
`SmtProofNodeTable.certificate_format` と `MachineSmtCertificateCandidate.certificate_format` の一致検査は
step 3 の table shape validation に含めます。
したがって同じ proof payload bytes に `payload_hash` mismatch と certificate_format mismatch が同時にある場合は、
step 2 の `PayloadHashMismatch` が優先です。

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
MVP の proof payload deterministic protocol cap は `nodes.len() <= 1_000_000` です。
加えて、proof payload 内の全 `SmtConclusionEncoding` から reachable な `CoreExpr` node 総数 `<= 1_000_000`、
`SmtExpr` node 総数 `<= 1_000_000`、および `SmtSortExpr` node 総数 `<= 1_000_000` です。
これらの cap を超える proof payload は resource guard ではなく payload canonical shape violation として
`Rejected { error = EnvelopeMalformed, feature_error = Some(SmtCertificate(NonCanonicalPayload)) }`
で拒否します。
validator は `nodes` の vector length prefix を element decode / allocation より前にこの cap と照合しなければなりません。
step 3 の full decode 中は nested `CoreExpr` / `SmtExpr` / `SmtSortExpr` counter を deterministic に進め、
length prefix または node tag を読んだ時点で残り cap を超えることが確定する場合は nested element allocation より前に拒否します。
この nested count cap violation は step 3 の failure なので、step 2 の `payload_hash` mismatch がある場合は
`PayloadHashMismatch` が優先します。
ただし任意の sparse id による表現揺れを避けるため、MVP では `nodes[k].node_id == k as u32` でなければならず、
`node_id` は 0 から `nodes.len() - 1` まで連続していなければなりません。
MVP では `u32::MAX` を valid payload node id として使わず、許可する node id は `0..nodes.len()` です。
`nodes.len() > 1_000_000`、または `node_id` が contiguous index と一致しない payload は
`Rejected { error = EnvelopeMalformed, feature_error = Some(SmtCertificate(NonCanonicalPayload)) }`
として拒否します。
`rule_fingerprint` は AI が新しい trusted rule を定義するための field ではありません。
`rule_registry_profile` は solver-native `PayloadNode` rule registry を選ぶ唯一の selector です。
`profile_version` は envelope / options schema の選択に使い、SMT rule registry の選択には使いません。
`certificate_format` と `command_profile` は payload / encoding format を表し、registry profile そのものではありません。
validator は `rule_registry_profile` で closed registry を選び、その registry 内で
`(certificate_format, logic, command_profile, rule_fingerprint)` を key にして rule entry を解決します。
registry entry は premise count、premise order、encoded conclusion check、必要な side condition、
および NPA reconstruction rule への対応を決定的に定義します。
registry entry は validator source 内で `SmtRuleDescriptor` から再計算した `rule_fingerprint` を key として
登録されていなければなりません。
resolved registry entry の descriptor から再計算した fingerprint が registry key と一致しない場合は、
candidate rejection ではなく validator implementation invariant failure です。
candidate payload が持つ `rule_fingerprint` で registry entry を lookup できない場合は、下の
`RuleRegistryMismatch` です。
この schema version で非空 registry profile を定義する場合、registry entry identity は次で固定します。

```text
struct SmtRuleDescriptor {
  certificate_format: SmtCertificateFormat,
  logic: SmtLogic,
  command_profile: SmtCommandProfile,
  rule_name_ascii: Vec<u8>,
  premise_profile: SmtRulePremiseProfile,
  conclusion_profile: SmtRuleConclusionProfile,
  side_condition_profile: SmtRuleSideConditionProfile,
  reconstruction_profile: SmtRuleReconstructionProfile,
}

rule_fingerprint =
  sha256(
    "npa.phase9_ai.smt.rule.v1"
    || canonical_bytes(SmtRuleDescriptor)
  )
```

```text
struct SmtRulePremiseProfile {
  arity: u32,
  order: SmtRulePremiseOrder,
}

enum SmtRulePremiseOrder {
  CertificateOrder,
}

enum SmtRuleConclusionProfile {
  EncodedConclusionChecked,
}

enum SmtRuleSideConditionProfile {
  None,
  RegistryNamed {
    name_ascii: Vec<u8>,
  },
}

enum SmtRuleReconstructionProfile {
  PayloadNodeProof,
  RegistryNamed {
    name_ascii: Vec<u8>,
  },
}
```

`rule_name_ascii` は長さ `1..=128` で `SmtSymbol.ascii` と同じ ASCII 制約に従います。
`RegistryNamed.name_ascii` も同じ制約に従い、validator source 内の closed registry entry 名だけを許可します。
`SmtRulePremiseProfile`、`SmtRuleConclusionProfile`、`SmtRuleSideConditionProfile`、
`SmtRuleReconstructionProfile` は validator source に含まれる closed enum / struct であり、
payload や solver log から読み込んではいけません。
canonical bytes は struct field order と enum variant tag / field order で固定します。
この文書の MVP schema が定義する `rule_registry_profile` variant は
`SmtRuleRegistryProfile::MvpEmptyRegistryV1` だけです。
この schema で未知の `rule_registry_profile` enum tag を受け取った場合は、payload の canonical decode failure として
`Rejected { error = EnvelopeMalformed, feature_error = None }`
で拒否します。
別 schema / profile で `SmtRuleRegistryProfile` variant を追加し、その variant を canonical decode できる validator では、
registry entry が存在しない場合にだけ
`Rejected { error = UnsupportedFeature, feature_error = Some(SmtCertificate(RuleRegistryMismatch)) }`
を返します。
`MvpEmptyRegistryV1` closed registry は
`(SmtCertificateFormat::MvpProofNodeTableV1, *, SmtCommandProfile::MvpNormalizedQf)`
に対して solver-native `PayloadNode` rule entry を1つも定義しません。
したがって MVP で `SmtReconstructionRule::PayloadNode` を含む request は、pre-registry validation を通過した後はすべて
`Rejected { error = UnsupportedFeature, feature_error = Some(SmtCertificate(RuleRegistryMismatch)) }`
として拒否します。
`IntroduceTheoryLemma` と `ComposeProof` はこの SMT rule registry を使わず、明示された imported theorem / combinator を
通常の core proof term として適用する local bookkeeping です。
ただし local bookkeeping は solver-derived proof step を変形・合成するための補助規則であり、
それだけで SMT certificate success を作ってはいけません。
success には少なくとも1つの accepted `PayloadNode` step が必要です。
`reconstruction_plan.steps` に `PayloadNode` が1つもない request は
`Rejected { error = UnsupportedFeature, feature_error = None }`
として拒否します。
solver-native payload node を受理するには、`SmtRuleRegistryProfile` に非空の registry entry 一覧を持つ別 variant
（例: `SmtRuleRegistryProfile::SolverNativeV2`）を追加し、各 rule descriptor と reconstruction rule をこの節と同じ粒度で固定してから有効化します。
非空 registry を持つ profile では、`SmtReconstructionRule::PayloadNode.rule_fingerprint` を使って registry entry を解決します。
registry に存在しない `rule_fingerprint`、または registry が premise order を一意に定義できない rule は
`Rejected { error = UnsupportedFeature, feature_error = Some(SmtCertificate(RuleRegistryMismatch)) }`
として拒否します。
registry entry が解決できた後、PayloadNode step / binding / payload node に現れる fingerprint が
resolved registry fingerprint と一致しない場合は、12p の binding validation または PayloadNode step-local check の
`PayloadBindingMismatch` として扱います。
この `MvpProofNodeTableV1` schema では payload が rule descriptor 本体を持たないため、
registry descriptor 再計算値と caller-provided descriptor fingerprint の不一致としての
`RuleFingerprintMismatch` は到達不能です。
`RuleFingerprintMismatch` は、payload が rule descriptor または rule name binding を canonical data として持つ
future certificate format/profile 用の予約 error です。
この registry は payload、solver log、network lookup、または AI explanation から拡張してはいけません。
上の SmtCertificate feature-specific validation order では、step 10 の LocalBookkeeping pre-registry structural checks までを終えた後、
step 11 で `reconstruction_plan.steps` に `PayloadNode` が存在するかを確認します。
この pre-registry structural checks は、LocalBookkeeping step の `payload_bindings` が空であること、
`rule = LocalBookkeeping { kind = IntroduceTheoryLemma { ... } }` の step では
`MachineSmtReconstructionStep.premises` が空であること、および `ReorderPremises` が使われていないことだけです。
この check に失敗した場合、validator は `PayloadNode` 有無チェックへ進みません。
複数の LocalBookkeeping 違反がある場合は `reconstruction_plan.steps` の canonical order で最初の違反を返し、
同じ step 内では `payload_bindings`、`IntroduceTheoryLemma` step の `premises`、`ReorderPremises` の順で判定します。
`ReorderPremises` が使われている場合は、同じ plan に `PayloadNode` が存在しても
`Rejected { error = UnsupportedFeature, feature_error = None }`
として拒否します。
`PayloadNode` が存在しない場合は local-bookkeeping-only plan として
`Rejected { error = UnsupportedFeature, feature_error = None }`
を返します。
`PayloadNode` が存在する場合はここで
`Rejected { error = UnsupportedFeature, feature_error = Some(SmtCertificate(RuleRegistryMismatch)) }`
を返します。
そのため MVP empty registry では `PayloadNode` 固有の rule fingerprint、conclusion、premise、proof reconstruction check には進みません。
結果として、この文書の `MvpEmptyRegistryV1` では `/machine/phase9/smt/reconstruct` は SMT certificate success を返しません。
success path を有効化するには、非空 solver-native registry を持つ別 `SmtRuleRegistryProfile` variant が必要です。
ここでいう proof payload artifact/file_hash/outer framing and node/nested CoreExpr/SmtExpr/SmtSortExpr count cap/payload_hash/full decode/table shape validation は、
上で定義した proof payload validation order そのものです。
outer framing and node count cap は `payload_hash` recomputation より先に検査し、
nested CoreExpr/SmtExpr/SmtSortExpr count cap は full decode 中に検査し、
full table shape validation は `SmtProofNodeTable` の canonical decode、certificate_format、node order / contiguity、
premise reference、acyclicity、nested `CoreExpr` / `SmtExpr` / `SmtSortExpr` count cap、および
`SmtConclusionEncoding` 内の nested `CoreExpr` / `SmtExpr` を canonical schema として decode できることまでです。
さらに pre-registry table shape validation では、各 `SmtConclusionEncoding.core_expr` を Phase 9 common wire `CoreExpr`
として走査し、`LevelExpr` が `payload.goal.universe_params` の外を参照していないこと、
`GlobalRef::Local(_)` / `GlobalRef::LocalGenerated` を含まないこと、および imported ref が
common Phase 9 wire `CoreExpr` imported ref resolution rule で解決できることまでを検査します。
この `core_expr` wire shape / import validation は `SmtConclusionEncoding` semantic validation ではなく、
proof payload validation order step 3 の table shape validation に含めます。
`core_expr` の universe scope violation または task-local global ref は
`Rejected { error = EnvelopeMalformed, feature_error = Some(SmtCertificate(NonCanonicalPayload)) }`、
imported ref 解決失敗は
`Rejected { error = ImportClosureMismatch, feature_error = None }`
として拒否し、`PayloadNode` 有無チェックや rule registry check へ進みません。
同じ proof payload bytes に `payload_hash` mismatch がある場合は、step 2 の `PayloadHashMismatch` が優先です。
`SmtConclusionEncoding.encoded_expr` については、`SmtExpr` / `SmtSortExpr` の representation-only canonical shape rule
（variant field order、`And` / `Or` の operand sort order / arity / duplicate、`BitVecLit` の fixed-width 表現など）と、
request payload の selected logic である `MachineSmtCertificateCandidate.logic` に対する
builtin sort / literal / `SmtBuiltinOp` allowlist だけを
pre-registry で検査します。
この representation-only violation は、`PayloadNode` が存在する request でも
`Rejected { error = EnvelopeMalformed, feature_error = Some(SmtCertificate(NonCanonicalPayload)) }`
を返し、`RuleRegistryMismatch` には進みません。
logic allowlist violation は、`PayloadNode` が存在する request でも
`Rejected { error = UnsupportedFeature, feature_error = None }`
を返し、`RuleRegistryMismatch` には進みません。
一方で、proof payload 側 `encoded_expr` の symbol resolution / sort checking against `encoded_problem.commands`、
deterministic encoder 再実行結果との一致、`core_expr` の kernel type check / Prop check は
`SmtConclusionEncoding` semantic validation です。
`SmtConclusionEncoding` の semantic validation、PayloadNode step の payload binding 解決、PayloadNode rule fingerprint /
conclusion / premise 照合、LocalBookkeeping の public interface 検査、ComposeProof の premise 使用検査、
proof term reconstruction、kernel check は pre-registry validation に含めません。
MVP empty registry で pre-registry validation を通過した `PayloadNode` request は、これらの post-registry validation に進む前に
`UnsupportedFeature + SmtCertificate(RuleRegistryMismatch)` を返します。
`SmtProofNodeTable.certificate_format` は `MachineSmtCertificateCandidate.certificate_format` と一致しなければなりません。
`nodes` は `node_id` の昇順で strictly sorted され、重複 node id や連番でない node id を含んではいけません。
各 `SmtProofNode.premises` は node table 内の既出 `node_id` だけを参照し、payload table 単体でも acyclic でなければなりません。
`SmtProofNode.premises` は certificate format が定義する proof rule premise order を canonical order とし、node_id で sort してはいけません。
この順序は reconstruction step の `premises` と positional に照合します。
premise order を一意に定義できない certificate format は MVP では
`Rejected { error = UnsupportedFeature, feature_error = Some(SmtCertificate(RuleRegistryMismatch)) }`
です。
以下の `SmtConclusionEncoding` semantic validation は、非空 solver-native registry profile で PayloadNode rule が
registry に受理され、その PayloadNode step の payload binding が一意に解決された後にだけ到達します。
MVP empty registry では到達不能です。
`SmtConclusionEncoding.encoder_version` / `logic` / `command_profile` は `encoded_problem` と一致しなければなりません。
`SmtConclusionEncoding.core_expr` は pre-registry table shape validation で wire shape / import resolution 済みであり、
ここでは同じ `goal.universe_params` と `goal.local_context` の下で well-typed な proposition でなければなりません。
validator は deterministic encoder を再実行し、`SmtConclusionEncoding.core_expr` から得た `SmtExpr` が
同じ `encoder_version` / `logic` / `command_profile` の下で `SmtConclusionEncoding.encoded_expr` と
canonical bytes 一致することを確認します。
`core_expr` が well-typed proposition であっても、selected encoder / command_profile がその core shape、
imported head、level pattern、または required `False` / `Not` recognition をサポートしない場合は、
deterministic encoder replay の unsupported mapping として
`Rejected { error = UnsupportedFeature, feature_error = None }`
で拒否します。
この validation は feature-specific validation order の step 12 内にある PayloadNode conclusion semantic check です。
各 PayloadNode step について、PayloadNode binding / rule fingerprint / node resolution check が成功した後、
payload node conclusion と reconstruction step conclusion を照合する前に、次の `12c` suborder で実行します。

```text
12c-a. encoder_version / logic / command_profile match encoded_problem
12c-b. core_expr kernel type inference and Prop check
12c-c. deterministic encoder support check, replay, and encoded_expr comparison
```

この段落の SMT payload-local validation での拒否分類は次で固定します。
`certificate_format` mismatch、node order / duplicate / non-contiguous id、acyclicity violation、premise が未知 node を指す場合、
payload node table の canonical decode failure、または `SmtConclusionEncoding.encoded_expr` の representation-only
canonical shape violation は
`Rejected { error = EnvelopeMalformed, feature_error = Some(SmtCertificate(NonCanonicalPayload)) }` です。
`SmtConclusionEncoding.encoded_expr` が request payload の selected logic で許可されない builtin sort / literal /
`SmtBuiltinOp` を含む場合は
`Rejected { error = UnsupportedFeature, feature_error = None }` です。
`SmtConclusionEncoding.encoder_version` / `logic` / `command_profile` が encoded problem と一致しない場合、
または `core_expr` は well-typed だが inferred type が `Prop` ではない場合は
`Rejected { error = FeatureRejected, feature_error = Some(SmtCertificate(ConclusionEncodingMismatch)) }` です。
`core_expr` 内の universe scope violation、task-local global ref、または imported ref 解決失敗は
pre-registry table shape validation で拒否済みです。
非空 solver-native registry profile で 12c に到達した時点でこれらが残っている場合は、
candidate rejection ではなく validator implementation invariant failure です。
`core_expr` 自体が goal context の下で kernel type inference に失敗する場合は
`Rejected { error = KernelRejected, feature_error = None }` です。
`core_expr` は well-typed proposition だが selected deterministic encoder table が対応 mapping を持たない場合、
または対応する `False` / `Not` recognition head が `options.smt` にない場合は
`Rejected { error = UnsupportedFeature, feature_error = None }` です。
encoder mapping が存在するにもかかわらず、`core_expr` から deterministic encoder を再実行した結果が
`encoded_expr` と一致しない場合は
`Rejected { error = FeatureRejected, feature_error = Some(SmtCertificate(EncodingMismatch)) }` です。
`SmtConclusionEncoding` では `TargetAssertion` の command-level `Not(...)` wrapper を暗黙に追加しません。
`SmtConclusionEncoding.encoded_expr = Not(encode(goal.target))` を使うなら、対応する
`core_expr` も `encode(core_expr) = Not(encode(goal.target))` を満たす明示 core proposition でなければなりません。
SMT rule validator は payload node の `encoded_expr` と premise payload node の `encoded_expr` を入力にし、
caller-provided `core_expr` を solver rule の根拠として直接信頼してはいけません。
非空 solver-native registry profile で PayloadNode step を受理する場合、
`MachineSmtPayloadBinding.payload_hash` は、同じ `MachineSmtCertificateCandidate.proof_payload` から再計算した
`payload_hash` と一致しなければなりません。各 reconstruction step は、少なくとも1つの payload node に結びつくか、
`SmtReconstructionRule` が明示的に local bookkeeping rule として定義されている必要があります。
MVP empty registry でも、LocalBookkeeping step はこの registry check の前に `payload_bindings.len() == 0`
でなければなりません。
non-empty の場合は
`Rejected { error = FeatureRejected, feature_error = Some(SmtCertificate(PayloadBindingMismatch)) }`
として拒否します。
この node table schema で `PayloadNode` を許可する `SmtRuleRegistryProfile` variant でも、`PayloadNode` step は
`payload_bindings.len() == 1` でなければなりません。
非空 solver-native registry profile の PayloadNode binding validation suborder は、各 `PayloadNode` step を
`reconstruction_plan.steps` の canonical order で処理し、同一 step 内では次で固定します。

```text
12p-a. payload_bindings.len() == 0 check
12p-b. each binding.payload_hash check, in payload order
12p-c. payload_bindings.len() > 1 check
12p-d. unique binding node_id resolution in proof_payload node table
12p-e. unique binding rule_fingerprint check against payload node and step rule
```

`PayloadNode` step の `payload_bindings.len() == 0` は payload node に結びつかない solver-native step なので
`Rejected { error = FeatureRejected, feature_error = Some(SmtCertificate(PayloadBindingMismatch)) }`
として拒否します。
`payload_bindings` が non-empty で、いずれかの binding の `payload_hash` が proof payload から再計算した値と一致しない場合は
`Rejected { error = PayloadHashMismatch, feature_error = None }`
です。
この `PayloadHashMismatch` は `payload_bindings.len() > 1` の `UnsupportedFeature` より優先します。
複数 payload node を1つの reconstruction step にまとめる candidate は `UnsupportedFeature` として拒否します。
binding が1つだけ存在するが node table 内の node に解決できない場合、または binding の
`rule_fingerprint` が対応 node / step rule と一致しない場合は
`Rejected { error = FeatureRejected, feature_error = Some(SmtCertificate(PayloadBindingMismatch)) }`
として拒否します。
非空 solver-native registry を持つ `SmtRuleRegistryProfile` variant では、
`SmtReconstructionRule::PayloadNode.rule_fingerprint` は、すべての `payload_bindings[*].rule_fingerprint` と一致しなければ
なりません。
この step / binding / payload node 間の fingerprint 不一致は `RuleFingerprintMismatch` ではなく
`PayloadBindingMismatch` です。
`LocalBookkeeping` は payload node を持たない step でだけ使えます。
`LocalBookkeeping` step が non-empty `payload_bindings` を持つ場合は
`Rejected { error = FeatureRejected, feature_error = Some(SmtCertificate(PayloadBindingMismatch)) }`
として拒否します。
この schema で proof term reconstruction が定義される local bookkeeping は `IntroduceTheoryLemma` と `ComposeProof` だけです。
MVP empty registry では、これらは local-bookkeeping-only success を許可するための規則ではありません。
`ReorderPremises` は enum に存在しても MVP では拒否します。
`PayloadNode` step の唯一の binding が指す payload node について、非空 solver-native registry を持つ `SmtRuleRegistryProfile` variant の validator は次を確認します。

```text
- payload node の rule_fingerprint が step.rule.rule_fingerprint と一致する
- payload node の conclusion_encoding が持つ core_expr が step.conclusion と definitional equality で一致する
- payload node の conclusion_encoding が持つ encoded_expr が、同じ encoder_version / logic / command_profile での
  deterministic encoder 再実行結果と canonical bytes 一致する
- payload node の premises 長が step.premises 長と一致する
- payload node premises[i] は step.premises[i] が指す prior step の唯一の PayloadNode binding の node_id と一致する
```

この `PayloadNode` step-local check での `definitional equality` は
`smt_reconstruction_defeq` profile で固定します。
この profile は β / ζ / ι reduction と、Phase 2 `DefDecl.reducibility = Reducible` の δ unfolding だけを許可します。
opaque def、opaque theorem、axiom、typeclass search、implicit insertion、AI hint、
SMT solver result、または quotient_v1 の computation rule は使いません。
payload conclusion と step conclusion が一致しない場合は
`Rejected { error = FeatureRejected, feature_error = Some(SmtCertificate(ReconstructionConclusionMismatch)) }` です。
premises 長または positional node binding が一致しない場合は
`Rejected { error = FeatureRejected, feature_error = Some(SmtCertificate(ReconstructionPremiseMismatch)) }` です。
この2つの PayloadNode-specific mismatch は、MVP empty registry では到達不能です。
payload node premise に local bookkeeping step を直接対応させることは MVP では許可しません。
payload proof を local bookkeeping で変形する場合は、payload node step の後に別の `LocalBookkeeping` step を置きます。
`ReorderPremises` は future extension 用の予約 variant です。
MVP では proof-producing structural combinator を hidden builtin として持たないため、`ReorderPremises` は常に
`Rejected { error = UnsupportedFeature, feature_error = None }`
として拒否します。
premise order の変換が必要な場合は、明示 imported combinator を使う `ComposeProof` として表します。
`IntroduceTheoryLemma.lemma` と `ComposeProof.combinator` は `imported_theory_refs` に含まれ、かつ envelope imports から
一意に解決できなければなりません。
`imported_theory_refs` の canonical shape と used-ref exactness は、reconstruction plan validation の先頭で検査します。
`level_args` / `term_args` はその imported declaration の public interface に対する明示 instantiation です。
引数探索、implicit insertion、typeclass search は local bookkeeping の中で実行しません。
非空 solver-native registry profile で local bookkeeping の explicit arguments を検査する場合、validator は
`reconstruction_plan.steps` の canonical order で各 `IntroduceTheoryLemma` / `ComposeProof` step を処理し、
同一 step 内では次の suborder を使います。

```text
12a. lemma / combinator ref has already been resolved from imported_theory_refs
12b. level_args arity check against the resolved public interface universe binder order
12c. level_args universe scope validation, left to right
12d. term_args use the already step-9-resolved CoreExprs, in payload order
12e. term_args kernel check against the instantiated public telescope, left to right
12f. proof term reconstruction and comparison with step.proof
12g. reconstructed proof type defeq check against step.conclusion
```

`level_args.len()` が public interface の universe binder arity と一致しない場合は
`Rejected { error = FeatureRejected, feature_error = Some(SmtCertificate(ReconstructionProofMismatch)) }`
として拒否します。
arity が一致した後、`level_args` 内の `LevelExpr` が `payload.goal.universe_params` の外を参照する場合は
`Rejected { error = EnvelopeMalformed, feature_error = Some(SmtCertificate(NonCanonicalPayload)) }`
として拒否します。
`term_args` 内の task-local global ref absence check と imported-ref resolution は、reconstruction plan validation step 9 で完了済みです。
非空 solver-native registry profile で 12d に到達した時点で未解決 imported ref が残っている場合は、
candidate rejection ではなく validator implementation invariant failure です。
各 `term_args[i]` は、先行する explicit level / term argument と public telescope の依存を反映した expected type に対して
payload goal context の下で kernel check できなければなりません。
この kernel check が失敗する場合は
`Rejected { error = KernelRejected, feature_error = None }`
です。
すべての explicit argument と premise proof を適用した後、proof term を一意に再構成できない、
再構成 proof bytes が `step.proof` と一致しない、または reconstructed proof type が `step.conclusion` と
`smt_reconstruction_defeq` で一致しない場合は
`Rejected { error = FeatureRejected, feature_error = Some(SmtCertificate(ReconstructionProofMismatch)) }`
として拒否します。
非空 solver-native registry profile で accepted PayloadNode を含む reconstruction を検査する場合、validator は
`SmtReconstructionRule`、`payload_bindings`、premise step の `conclusion`、`imported_theory_refs`
から step の proof term を決定的に再構成します。再構成された canonical `CoreExpr` bytes が `step.proof` と
一致しなければ
`Rejected { error = FeatureRejected, feature_error = Some(SmtCertificate(ReconstructionProofMismatch)) }`
として拒否します。`step.proof` を caller-provided trusted proof として扱ってはいけません。
local bookkeeping の再構成は次に限定します。

```text
ReorderPremises:
  returns Rejected { error = UnsupportedFeature, feature_error = None }

IntroduceTheoryLemma:
  lemma に level_args / term_args をその順序で適用した proof term を作る

ComposeProof:
  combinator に level_args / term_args と premise proof を premises の配列順で適用する
```

`IntroduceTheoryLemma` step の `premises` は空でなければなりません。
non-empty の場合は
`Rejected { error = FeatureRejected, feature_error = Some(SmtCertificate(ReconstructionPremiseMismatch)) }`
として拒否します。
`ComposeProof` は `premises` の proof term を配列順ですべて使わなければならず、
premise を無視したり追加の hidden premise を補ったりしてはいけません。
この規則だけで `step.conclusion` の proof term を一意に構成できない場合は
`Rejected { error = FeatureRejected, feature_error = Some(SmtCertificate(ReconstructionProofMismatch)) }`
として拒否します。
MVP empty registry では proof term reconstruction には進みませんが、LocalBookkeeping rule-local structural checks として
`IntroduceTheoryLemma` step の `premises` が空であること、`ReorderPremises` が使われていないことは
`PayloadNode` 有無チェックより先に検査します。
`ComposeProof` の public interface 検査、premise proof term の使用検査、`step.conclusion` との照合は
proof term reconstruction の一部であり、MVP empty registry では到達不能です。

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

非 empty solver-native registry profile での採用条件:

この節の条件は、`SmtRuleRegistryProfile::MvpEmptyRegistryV1` では評価しません。
MVP empty registry では pre-registry validation を通過した後、PayloadNode を含む request は
`UnsupportedFeature + SmtCertificate(RuleRegistryMismatch)` で拒否され、
PayloadNode を含まない request は solver-native SMT certificate success には到達しません。

```text
- target.goal_fingerprint が payload.goal から再計算できる
- encoded_problem.problem_hash / encoding_hash が inline bytes または artifact bytes から再計算できる
- encoded_problem 内の goal_fingerprint / logic が envelope target / payload logic と一致する
- encoded_problem commands が command_profile の canonical phase order を満たす
- Artifact の file_hash / size_bytes が実ファイル bytes と一致する
- proof_payload outer framing / node count cap が proof payload validation order の step 1 を満たす
- proof_payload.payload_hash が inline bytes または artifact bytes から再計算できる
- proof_payload bytes が nested CoreExpr / SmtExpr / SmtSortExpr count cap を含めて certificate_format ごとの
  canonical SMT proof node table として decode できる
- payload の `rule_registry_profile` で選ばれた closed registry に受理済みの PayloadNode step が少なくとも1つ存在する
- PayloadNode step の payload_bindings は proof_payload 内の node に解決でき、rule_fingerprint が payload node の rule と一致する
- LocalBookkeeping step の payload_bindings は空であり、imported_theory_refs / explicit arguments だけから proof term を再構成できる
- SmtReconstructionRule と payload_bindings の rule_fingerprint / local bookkeeping 制約が一致する
- reconstruction_plan.steps が acyclic で、premises が先行 step_id だけを参照する
- 各 step の proof が rule validator の再構成結果と canonical bytes 一致する
- 各 step の proof が、payload.goal.universe_params / payload.goal.local_context の下でその step の conclusion の proof term として kernel check を通る
- final_step が steps 内に存在し、その conclusion が payload.goal.universe_params / payload.goal.local_context の下で payload.goal.target と definitional equality で一致する
- final_proof が final_step の proof と一致し、payload.goal.universe_params / payload.goal.local_context の下で payload.goal.target の proof term として kernel check を通る
- independent checker が resulting certificate を再検査できる
```

次の `final_step` / `final_proof` 検査と error classification も、非 empty solver-native registry profile で
PayloadNode が受理され、proof reconstruction が実行される場合だけ到達します。
ここでいう `final_step` 検査は、pre-registry canonical shape で範囲内と確認済みの step が導く conclusion と
`payload.goal.target` の semantic check です。
`final_step` が `steps` の範囲外である request はこの段落に到達せず、pre-registry で
`Rejected { error = EnvelopeMalformed, feature_error = Some(SmtCertificate(NonCanonicalPayload)) }`
として拒否されます。
`final_step.conclusion` と `payload.goal.target` の defeq check も `smt_reconstruction_defeq` profile を使います。
この defeq mismatch は
`Rejected { error = FeatureRejected, feature_error = Some(SmtCertificate(ReconstructionConclusionMismatch)) }` です。
`final_proof` と `final_step.proof` の canonical bytes mismatch は
`Rejected { error = FeatureRejected, feature_error = Some(SmtCertificate(ReconstructionProofMismatch)) }` です。
step proof または final_proof が対応 conclusion / target の proof term として kernel check を通らない場合は
`Rejected { error = KernelRejected, feature_error = None }` です。

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

enum TheoremGraphExtractorVersion {
    MvpCertificateGraphV1,
}

enum TheoremGraphFeatureSchemaVersion {
    MvpGoalFeaturesV1,
}

enum TheoremGraphEdgeKind {
    ImportsDeclaration,
    UsesConstant,
    MentionsType,
}

struct TheoremGraphFeatureKey {
    namespace_ascii: Vec<u8>,
    name_ascii: Vec<u8>,
}

enum TheoremGraphFeatureValue {
    Bool(bool),
    I64(i64),
    Hash(Hash256),
}
```

`TheoremGraphExtractorVersion` / `TheoremGraphFeatureSchemaVersion` /
`TheoremGraphRankingProfile` / `TheoremGraphEdgeKind` canonical bytes は variant tag だけで固定します。
MVP の theorem graph deterministic protocol cap は
`MachineTheoremGraphSnapshot.nodes.len() <= 1_000_000`、
`MachineTheoremGraphSnapshot.edges.len() <= 1_000_000`、
および `MachineTheoremGraphQueryFeatures.features.len() <= 65_536` です。
snapshot raw bytes cap は Inline / Artifact とも `<= 128_000_000`、
query features raw bytes cap は Inline / Artifact とも `<= 16_000_000` です。
validator はこれらの vector length prefix を element decode / allocation より前に cap と照合しなければなりません。
snapshot の cap violation は
`Rejected { error = EnvelopeMalformed, feature_error = Some(TheoremGraphQuery(SnapshotMalformed)) }`、
query features の cap violation は
`Rejected { error = EnvelopeMalformed, feature_error = Some(TheoremGraphQuery(QueryFeaturesMalformed)) }`
として拒否します。
Inline の raw bytes cap は outer framing decode より先に検査します。
Artifact の場合は declared `size_bytes` が cap を超える時点で file read / `file_hash` check へ進まず、
snapshot なら `TheoremGraphQuery(SnapshotMalformed)`、query features なら `TheoremGraphQuery(QueryFeaturesMalformed)` の
`EnvelopeMalformed` として拒否します。
declared `size_bytes` が cap 内でも実ファイル bytes の長さや `file_hash` が一致しない場合は
`Rejected { error = PayloadHashMismatch, feature_error = None }`
です。
validator は server-local timeout、memory guard、runtime configuration からこの cap を増減してはいけません。
MVP では `TheoremGraphExtractorVersion::MvpCertificateGraphV1` と
`TheoremGraphFeatureSchemaVersion::MvpGoalFeaturesV1` と
`TheoremGraphRankingProfile::MvpTupleOrder` だけを受け付けます。
`MachineTheoremGraphQuery.ranking_profile` がこの schema で decode できない enum tag の場合は、
top-level query payload の canonical schema violation として
`Rejected { error = EnvelopeMalformed, feature_error = None }`
で拒否します。
将来 `TheoremGraphRankingProfile` に別 variant を追加する場合、既存 variant tag を変えず、
その profile の result ordering、score rule、`query_features` の利用範囲、unsupported rejection を同じ節で固定しなければなりません。
`TheoremGraphEdgeKind` の canonical order は enum variant order
`ImportsDeclaration < UsesConstant < MentionsType` です。
別 kind を追加する場合は、既存 kind の order を変えず末尾へ追加するか、別 schema version を定義します。
`TheoremGraphFeatureKey` の各 ASCII field は長さ `1..=64` で、
正規表現 `[A-Za-z_][A-Za-z0-9_.:-]*` に一致しなければなりません。
一致しない query feature bytes は
`Rejected { error = EnvelopeMalformed, feature_error = Some(TheoremGraphQuery(QueryFeaturesMalformed)) }`
として拒否します。
canonical bytes は `namespace_ascii`、`name_ascii` の順の length-prefixed raw ASCII bytes です。
Unicode normalization、case folding、外部 feature registry lookup は行いません。

`score` は対応する `node` にだけ結びつく非信頼 metadata です。
`score` は certificate に入りません。
`MachineTheoremGraphNodeRef` は Phase 5 / Phase 6 と同じ theorem graph node identity を持ちます。
この identity key は次で固定し、`Phase9AiGlobalRef` の tuple key とは field order が異なります。

```text
theorem_graph_node_identity_key(node) =
  module
  name
  export_hash
  certificate_hash
  decl_interface_hash
```

AI premise retrieval が graph node から `Phase9AiGlobalRef` を作る場合は、同じ field 値を
`Phase9AiGlobalRef` の canonical field order
`module / export_hash / certificate_hash / name / decl_interface_hash`
へ並べ替えなければなりません。
validator は theorem graph node identity comparator を `Phase9AiGlobalRef` tuple comparator で代用してはいけません。
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

`source_release_hash` は graph extractor が入力 release を識別するための opaque content-addressed id です。
MVP の Phase 9 validator は `source_release_hash` を envelope imports や filesystem から再計算しません。
validator が行う検査は、`MachineTheoremGraphSnapshotRef.source_release_hash` と
snapshot bytes 内の `MachineTheoremGraphSnapshot.source_release_hash` の bytewise equality だけです。
この値は theorem existence、import closure completeness、または certificate validity の根拠ではありません。
将来 source release 自体を検証したい場合は、release manifest bytes を持つ別 payload を定義し、
その manifest からの hash 再計算規則を追加します。

`MachineTheoremGraphQuery.env_fingerprint` / `goal_fingerprint` は envelope の `target.env_fingerprint` /
`target.goal_fingerprint` と完全一致しなければなりません。
`MachineTheoremGraphQuery.goal` から再計算した `goal_fingerprint` も一致しなければなりません。
これらの target binding が一致しない場合は
`Rejected { error = TargetFingerprintMismatch, feature_error = None }`
として拒否します。
TheoremGraphQuery の feature-specific validation 順序は固定します。
この順序は common envelope validation step 5 が終わった後に適用します。
`MachineTheoremGraphQuery` payload bytes の task-specific decode はこの feature-specific validation の step 0 であり、
common envelope validation step 3 の `target.env_fingerprint` mismatch より先に実行してはいけません。

```text
0. MachineTheoremGraphQuery payload outer canonical decode / scalar field decode
1. query target binding check
2. Phase9AiGoal wire shape / universe context shape and imported-ref resolution check
3. Phase9AiGoal local_context / target kernel well-typedness check
4. ranking_profile is MvpTupleOrder and limit is 0..=256
5. snapshot / query_features artifact bytes validation, vector length cap precheck, embedded hash check,
   and full canonical decode
6. snapshot source_release_hash / extractor_version metadata check
7. query_features env_fingerprint / goal_fingerprint / feature_schema_version metadata check
8. query feature key grammar / sort / duplicate / value kind check
9. snapshot node / edge sort / duplicate / edge target check
10. snapshot node import resolution, decl_certificate_hash / type_hash check, and eligibility classification
11. MvpTupleOrder result construction
```

step 0 で payload outer canonical decode / scalar field decode に失敗した場合は
`Rejected { error = EnvelopeMalformed, feature_error = None }`
として拒否し、query target binding、`limit`、artifact hash、snapshot/query feature bytes は検査しません。
step 1 で拒否される request では `limit`、artifact hash、snapshot/query feature bytes は検査しません。
step 2 / 3 で拒否される request では `limit`、artifact hash、snapshot/query feature bytes は検査しません。
したがって同じ request に Phase9AiGoal wire shape / universe context shape violation、imported ref resolution failure、
または goal ill-typedness と `limit > 256` が同時にある場合は、
step 2 の `EnvelopeMalformed` / `ImportClosureMismatch` または step 3 の `KernelRejected` が優先です。
step 4 で `limit > 256` の request は、snapshot / query_features artifact を読まずに
`Rejected { error = EnvelopeMalformed, feature_error = Some(TheoremGraphQuery(LimitOutOfRange)) }`
として拒否します。
したがって `limit > 256` と artifact hash mismatch / query feature malformed / node resolution mismatch が同時にある場合は
`LimitOutOfRange` が優先です。
`ranking_profile` はこの schema では `MvpTupleOrder` 以外を canonical decode できないため、step 4 の unsupported profile branch はありません。
将来同じ schema version に既知だが未対応の ranking profile を追加する場合は、step 4 で
`Rejected { error = UnsupportedFeature, feature_error = None }`
として拒否するか、別 schema version で順序を再定義します。
validator は snapshot bytes / query feature bytes の outer canonical framing を検査した後、provided bytes そのものから
`graph_snapshot_hash` / `query_features_hash` を再計算します。
full canonical decode は embedded hash check の後に行います。
feature step 5 の artifact-ref validation suborder は、各 ref について次で固定します。
feature step 5 全体では snapshot ref を先にこの suborder で最後まで検査し、snapshot ref が成功した後にだけ
query_features ref を同じ順序で検査します。
したがって snapshot ref の file_hash mismatch / malformed / embedded hash mismatch と
query_features ref の不整合が同じ request にある場合は、snapshot ref の rejection が優先です。

```text
0. Inline bytes acquisition and byte cap check, or Artifact path validation /
   declared size cap check / file read / file_hash / size_bytes check
1. outer canonical framing / schema tag / scalar field decode and vector length cap precheck
2. graph_snapshot_hash or query_features_hash recomputation and comparison
3. full canonical decode
```

substep 0 で snapshot / query features bytes cap を超える場合は、対象 ref に応じて
`Rejected { error = EnvelopeMalformed, feature_error = Some(TheoremGraphQuery(SnapshotMalformed)) }`
または
`Rejected { error = EnvelopeMalformed, feature_error = Some(TheoremGraphQuery(QueryFeaturesMalformed)) }`
です。
substep 0 で `Artifact.file_hash` / `size_bytes` が一致しない場合は
`Rejected { error = PayloadHashMismatch, feature_error = None }` です。
path validation を通過した artifact bytes を取得できない場合は `Error::ArtifactUnavailable` です。
substep 1 の snapshot outer framing / cap precheck failure は
`Rejected { error = EnvelopeMalformed, feature_error = Some(TheoremGraphQuery(SnapshotMalformed)) }`、
query features outer framing / cap precheck failure は
`Rejected { error = EnvelopeMalformed, feature_error = Some(TheoremGraphQuery(QueryFeaturesMalformed)) }`
として拒否し、embedded hash mismatch より優先します。
substep 2 の hash input は、substep 1 の outer canonical framing precheck を通過した provided bytes そのものです。
この時点では full decode していないため、validator は snapshot / query features を再serializeして hash input を作ってはいけません。
再計算した `graph_snapshot_hash` または `query_features_hash` が ref に埋め込まれた hash と一致しない場合は
`Rejected { error = PayloadHashMismatch, feature_error = None }`
です。
substep 2 で embedded hash が一致しても substep 3 の full canonical decode が失敗した場合、snapshot bytes では
`Rejected { error = EnvelopeMalformed, feature_error = Some(TheoremGraphQuery(SnapshotMalformed)) }`、
query features bytes では
`Rejected { error = EnvelopeMalformed, feature_error = Some(TheoremGraphQuery(QueryFeaturesMalformed)) }`
です。
snapshot bytes 内の source release / extractor version metadata は、`source_release_hash` と
`extractor_version` に一致しなければなりません。
query feature bytes 内の `env_fingerprint` / `goal_fingerprint` は request query と一致し、
`feature_schema_version` は `TheoremGraphFeatureSchemaVersion::MvpGoalFeaturesV1` でなければなりません。
snapshot metadata が一致しない場合は
`Rejected { error = EnvelopeMalformed, feature_error = Some(TheoremGraphQuery(SnapshotMalformed)) }`
として拒否します。
query feature bytes 内の `env_fingerprint` / `goal_fingerprint` が request query と一致しない場合、
または `feature_schema_version` が MVP profile と一致しない場合は
`Rejected { error = EnvelopeMalformed, feature_error = Some(TheoremGraphQuery(QueryFeaturesMalformed)) }`
として拒否します。
`MachineTheoremGraphFeature` は `key` の canonical bytes 昇順で strictly sorted され、重複 key を含んではいけません。
feature value は bool、signed 64-bit integer、hash digest だけを許可します。float、embedding vector、implementation-defined
rounding を canonical query feature に入れてはいけません。
snapshot の `nodes` は identity tuple
`module / name / export_hash / certificate_hash / decl_interface_hash` の辞書順で
strictly sorted かつ重複なしでなければなりません。
`decl_certificate_hash` と `type_hash` は sort key に含めません。
`edges` は `from identity tuple / to identity tuple / kind` の辞書順で strictly sorted かつ重複なしでなければなりません。
すべての edge の `from` / `to` は snapshot の `nodes` に exact canonical bytes で存在しなければなりません。
存在しない node を指す edge を含む snapshot は
`Rejected { error = EnvelopeMalformed, feature_error = Some(TheoremGraphQuery(SnapshotMalformed)) }`
として拒否します。
step 10 では snapshot の全 `nodes` を canonical order で走査します。
各 node について、envelope imports 内の export table から
`module / export_hash / certificate_hash / name / decl_interface_hash` で一意に解決できる場合、
validator は解決した Phase 2 export / declaration から `decl_certificate_hash` と `type_hash` を再計算し、
node の値と一致することを確認します。`type_hash` は Phase 2 `ExportEntry.type_hash` です。
`decl_certificate_hash` はその declaration の canonical certificate declaration hash です。
解決できた node の `decl_certificate_hash` または `type_hash` が export table から再計算した値と一致しない場合は、
その node が theorem graph result に入らない kind であっても
`Rejected { error = FeatureRejected, feature_error = Some(TheoremGraphQuery(NodeResolutionMismatch)) }`
として拒否します。
解決でき、かつ hash / type check が一致した node のうち、解決した `ExportEntry.kind` が theorem または axiom の
public export であるものだけを MVP の eligible node とします。
definition、constructor、recursor、generated artifact は、hash / type check が一致しても eligible node にはせず、
theorem graph query result として返してはいけません。
それらを rewrite candidate や constructor hint として検索したい場合は、別の task kind または ranking profile で
期待する `ExportEntry.kind` を明示してから有効化します。
snapshot に含まれるが envelope imports で解決できない node は、hash / type check を行わず、
MVP では result の eligible node にしません。外部 graph store lookup で missing node を補完してはいけません。
MVP の `ranking_profile = MvpTupleOrder` では、validated graph result の `score_microunits` はすべて `0` でなければならず、
result ordering は `module / name / export_hash / certificate_hash / decl_interface_hash` の tuple 辞書順に固定します。
この profile の result entries は、snapshot の sorted `nodes` から eligible node だけを残した列の先頭
`min(limit, eligible_nodes.len())` 件をそのまま返すものに限定します。
したがって MVP の theorem graph query は、goal-aware premise retrieval ではなく、
snapshot / query feature artifact と certificate-bound node ref を deterministic に検証する endpoint です。
`goal` と `query_features` は request binding と future ranking profile の replay input ですが、
`MvpTupleOrder` の ranking では theorem selection に使いません。
`limit` は `0 <= limit <= 256` の `u32` でなければなりません。
`limit = 0` の場合は empty result です。
`limit > 256` の query は implementation-specific resource guard に委ねず、
`Rejected { error = EnvelopeMalformed, feature_error = Some(TheoremGraphQuery(LimitOutOfRange)) }`
として wire validation で deterministic に拒否します。
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

MVP Formalization の deterministic byte cap は次で固定します。

```text
candidate.statement.term_canonical_bytes.len() <= 1_000_000
source document raw UTF-8 bytes <= 16_000_000
rejection reason raw UTF-8 bytes <= 1_000_000
```

`MachineSurfaceTerm.term_canonical_bytes` の length prefix cap は Phase 3 Machine Surface term-source canonical decode より先に検査します。
Inline source / rejection reason の raw bytes length prefix cap も、UTF-8 decode と embedded hash recomputation より先に検査します。
Artifact source / rejection reason は、declared `size_bytes` が cap を超える場合に file read / `file_hash` check へ進まず
`Rejected { error = EnvelopeMalformed, feature_error = None }`
として拒否します。
declared `size_bytes` が cap 内でも実ファイル bytes の長さや `file_hash` が一致しない場合は、従来どおり
`Rejected { error = PayloadHashMismatch, feature_error = None }`
です。

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
一致しない場合は
`Rejected { error = EnvelopeMalformed, feature_error = None }`
として拒否します。
この check の正確な位置は下の Formalization deterministic validation order の step 3 です。
`MachineSurfaceTerm` wrapper shape check と、candidate の source document / claim span、
および current intent status で必要な rejection reason の bytes cap、artifact integrity、
UTF-8 decode、span validation、embedded hash 再計算の後、
`RejectedIntentHasProofCandidate`、`IntentRecordMismatch`、candidate statement elaboration、
または proof bridge validation より先に実行します。
したがって `MachineSurfaceTerm` shape violation、source/rejection bytes cap violation、
artifact の hash mismatch、invalid UTF-8、span violation、embedded hash mismatch は `ReviewerId` regex violation より優先し、
`ReviewerId` regex violation は formalization feature-specific rejection より優先します。
Unicode normalization、case folding、email alias 展開、外部 user directory lookup は行いません。
`ReviewerId::Human.stable_id_ascii`、`ReviewerId::System.system_id_ascii`、`ReviewerId::System.actor_id_ascii` は
canonical bytes の bytewise equality だけで比較します。
同じ人間や外部 system を別 field 値で表した場合、validator は同一 reviewer と推測してはいけません。

Formalization payload の deterministic validation order は次で固定します。
この順序は common envelope validation step 5 が終わった後に適用します。
つまり imports / options / `target.env_fingerprint` / task target shape と、
`options.formalization = Some(...)`、`MachineTacticOptions` canonical bytes precheck、
`simp_rules` sort / duplicate check、tactic budget scalar / range validation を含む
task options shape / semantic range validation はこの列より前に完了しています。

```text
0. MachineFormalizationCheckPayload payload outer canonical decode / scalar field decode
1. candidate.statement MachineSurfaceTerm wrapper shape check
   - universe_params duplicate / Phase 3 identifier compatibility
   - term_canonical_bytes length prefix cap
   - term_canonical_bytes Phase 3 Machine Surface term-source canonical decode
2. candidate.source_document / claim_span は intent_record の有無や status に関係なく常に
   source bytes cap、artifact integrity、UTF-8 decode、span validation、embedded hash recomputation を検査する。
   rejection reason は intent_record.status = Rejected の場合だけ同じ bytes cap、
   artifact integrity、UTF-8 decode、embedded hash recomputation を検査する
3. ReviewerId regex validation
4. RejectedIntentHasProofCandidate check
5. intent_record source_document_hash / claim_span_hash / candidate_statement_hash / rejection_reason_hash consistency check
6. non-rejected status で必要な candidate statement complete-mode elaboration / type inference
7. optional proof bridge validation and certificate binding check
```

step 0 で payload outer canonical decode / scalar field decode に失敗した場合は、candidate.statement、
source / rejection artifact、ReviewerId、intent_record.status、intent_record consistency は検査しません。
step 1 の `MachineSurfaceTerm` shape violation は、source / rejection artifact hash mismatch や
`ReviewerId` regex violation より優先します。
step 2 の source document / rejection reason bytes cap violation、artifact の hash mismatch、invalid UTF-8、span violation、embedded hash mismatch は
step 3 の `ReviewerId` regex violation より優先し、step 3 は step 4 以降の formalization feature-specific rejection より優先します。
`intent_record.status = Rejected` では step 6 / 7 を実行しません。

この節の artifact 参照は、まず declared `size_bytes` が上の byte cap 内にあることを検査し、cap を超える場合は
file read、`Artifact.file_hash` check、UTF-8 decode、content hash 再計算へ進みません。
cap 内の場合だけ、実ファイル bytes に対して `Artifact.file_hash` / `size_bytes` を検査します。
artifact integrity が一致しない場合は UTF-8 decode や content hash 再計算へ進まず、
`Rejected { error = PayloadHashMismatch, feature_error = None }`
として拒否します。
したがって artifact bytes が file_hash mismatch かつ invalid UTF-8 の場合は `PayloadHashMismatch` が優先です。
artifact integrity が一致した後、または inline bytes の場合にだけ、source document / rejection reason bytes を
UTF-8 として decode し、content hash と span を検査します。

`source_document_hash` は raw UTF-8 document bytes から次で再計算します。

```text
source_document_hash =
  sha256("npa.phase9_ai.formalization.source_document.v1" || raw_utf8_document_bytes)
```

`Inline.raw_utf8_bytes` は document wrapper ではなく、文書そのものの raw UTF-8 bytes です。
`Artifact` の場合も file bytes を raw UTF-8 document bytes として扱います。
source document bytes は hash 再計算の前に UTF-8 として decode できなければなりません。
invalid UTF-8 の inline bytes または artifact bytes は
`Rejected { error = EnvelopeMalformed, feature_error = None }`
として拒否します。
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
この range validation に失敗した場合は
`Rejected { error = EnvelopeMalformed, feature_error = None }`
です。

`rejection_reason_hash` は raw UTF-8 reason bytes から次で再計算します。

```text
rejection_reason_hash =
  sha256("npa.phase9_ai.formalization.rejection_reason.v1" || raw_utf8_reason_bytes)
```

`MachineFormalizationRejectionReasonRef::Inline.raw_utf8_bytes` は wrapper ではなく理由本文そのものの raw UTF-8 bytes です。
`Artifact` の場合も file bytes を raw UTF-8 reason bytes として扱います。
rejection reason bytes も hash 再計算の前に UTF-8 として decode できなければなりません。
invalid UTF-8 の inline bytes または artifact bytes は
`Rejected { error = EnvelopeMalformed, feature_error = None }`
として拒否します。
`Artifact.file_hash` / `size_bytes` は実ファイル bytes と一致しなければなりません。
inline bytes または artifact bytes から再計算した hash は、
`MachineFormalizationRejectionReasonRef` 内の `rejection_reason_hash` と一致しなければなりません。
`FormalizationIntentStatus::Rejected.rejection_reason_hash` は artifact / inline ref の embedded hash ではなく、
intent record consistency field です。
これは rejection reason ref の hash binding が成功した後、step 5 で validated rejection reason hash と比較します。
一致しない場合は `PayloadHashMismatch` ではなく
`Rejected { error = FeatureRejected, feature_error = Some(Formalization(IntentRecordMismatch)) }`
です。
`candidate.source_document` 内の `source_document_hash`、`candidate.claim_span.claim_span_hash`、
または `MachineFormalizationRejectionReasonRef` 内の `rejection_reason_hash` の再計算値が
それぞれの embedded hash と一致しない場合は
`Rejected { error = PayloadHashMismatch, feature_error = None }`
として拒否します。
`Artifact.file_hash` / `size_bytes` が実ファイル bytes と一致しない場合も同じ
`Rejected { error = PayloadHashMismatch, feature_error = None }`
です。

`candidate_statement_hash` は、この candidate の `statement: MachineSurfaceTerm` の canonical bytes に
`"npa.phase9_ai.formalization.candidate_statement.v1"` tag を付けて hash した値です。
Phase 9 validator は、candidate statement の complete-mode elaboration が成功した後、
`candidate.statement.universe_params` を byte-for-byte そのまま `accepted_universe_params` として採用します。
Phase 3 AI の term AST elaboration API は `(elaborated_core_term, inferred_type)` だけを返し、
`accepted_universe_params` を別 return value として返しません。
`accepted_statement_hash` は、`target.env_fingerprint`、`accepted_universe_params`、accepted theorem type の canonical `CoreExpr` bytes に
`"npa.phase9_ai.formalization.accepted_statement.v1"` tag を付けて hash した値です。
reviewed intent record を certificate に結びつける場合、certificate を作る request envelope の environment と certificate 内の theorem type から
同じ `accepted_statement_hash` を再計算できなければなりません。
同じ theorem type bytes でも import closure / options が違う場合は別の accepted statement として扱います。

Formalization statement の Phase 3 elaboration context は次で固定します。

```text
MachineCompileOptions:
  mode = MachineSurfaceMode::Complete
  allow_universe_meta = false

MachineTermElabContext:
  global_scope =
    envelope.imports の public export だけから作る Imported entries
    import_index は envelope imports の canonical sort 後 index
    CurrentModule / CurrentGenerated entries は空
  local_context = []
  universe_params =
    candidate.statement.universe_params を phase3_universe_param_ident で Vec<String> に写したもの
  kernel_env = envelope.imports だけから構築した checked environment
```

Phase 9 validator は、Phase 3 の source string 用 `elaborate_machine_term_check(source, expected, ...)` ではなく、
canonical term bytes から直接 decode / infer できる API を使います。
実装名は Phase 3 側に合わせてよいですが、意味論上は次の2段階 API が必要です。

```rust
pub fn decode_machine_term_source_canonical(
    canonical_bytes: &[u8],
) -> Result<MachineTermAst, MachineDiagnostic>;

pub fn elaborate_machine_term_infer_from_ast(
    ast: &MachineTermAst,
    context: &MachineTermElabContext,
    options: &MachineCompileOptions,
) -> Result<(npa_kernel::Expr, npa_kernel::Expr), MachineDiagnostic>;
```

戻り値は `(elaborated_core_term, inferred_type)` です。
validator は `term_canonical_bytes` から source string を復元したり pretty printer を経由したりしてはいけません。

validator は `MachineSurfaceTerm.term_canonical_bytes` を Phase 3 Machine Surface term AST として decode し、
上の context で complete-mode elaboration / type inference を実行します。
その core term は theorem type candidate なので、inferred type が `Sort(level)` でなければなりません。
decode は成功したが elaboration / type inference に失敗した場合、または inferred type が `Sort(level)` でない場合は
`Rejected { error = FeatureRejected, feature_error = Some(Formalization(CandidateStatementElaborationFailed)) }`
です。
成功時の `accepted_universe_params` は `candidate.statement.universe_params` と byte-for-byte に一致し、
accepted theorem type は elaboration result の canonical `CoreExpr` です。
Phase 9 validator は `MachineSurfaceTerm` の外にある source document text、claim span text、pretty theorem statement、
orchestrator state、または hidden current module declaration から Phase 3 scope / universe params を補完してはいけません。

`intent_record.status = Rejected` かつ `candidate.optional_proof_candidate = Some(...)` の場合、
validator は source document / rejection reason の bytes cap、artifact integrity、UTF-8 decode、span validation、
embedded hash 再計算、および `ReviewerId` regex validation をすべて終えた後、
`intent_record.source_document_hash` / `claim_span_hash` / `candidate_statement_hash` /
`FormalizationIntentStatus::Rejected.rejection_reason_hash` の一致検査より先に
`Rejected { error = FeatureRejected, feature_error = Some(Formalization(RejectedIntentHasProofCandidate)) }`
として拒否します。
したがって source document / rejection reason bytes の cap violation、invalid UTF-8、span violation、
candidate.source_document / claim_span / rejection_reason ref 内の embedded hash recomputation mismatch、artifact integrity mismatch、
または `ReviewerId` regex violation は
`RejectedIntentHasProofCandidate` より優先します。
ここでいう embedded hash recomputation mismatch は、inline / artifact bytes から再計算した
`source_document_hash` / `claim_span_hash` / `rejection_reason_hash` が、
candidate.source_document / candidate.claim_span / `MachineFormalizationRejectionReasonRef` 自体に埋め込まれた hash と
一致しない場合です。
intent_record の `source_document_hash` / `claim_span_hash` / `candidate_statement_hash` /
`FormalizationIntentStatus::Rejected.rejection_reason_hash` と candidate / validated rejection reason hash との
consistency mismatch は step 5 の `IntentRecordMismatch` 判定であり、この step 4 より前には返しません。
したがってこの場合、intent record 側の `candidate_statement_hash` が candidate と不一致でも、
または `FormalizationIntentStatus::Rejected.rejection_reason_hash` が validated rejection reason hash と不一致でも、
`IntentRecordMismatch` は返しません。
それ以外で `intent_record` が `Some` の場合、`source_document_hash` / `claim_span_hash` / `candidate_statement_hash` は
payload の `candidate` から再計算した値と一致しなければなりません。
`intent_record.status = Rejected` では、`FormalizationIntentStatus::Rejected.rejection_reason_hash` も
validated rejection reason hash と一致しなければなりません。
一致しない場合は
`Rejected { error = FeatureRejected, feature_error = Some(Formalization(IntentRecordMismatch)) }`
として拒否します。
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
  一致せず、かつ optional_proof_candidate = None の場合だけ、reviewed intent record の整合性だけを検査する intent-record-only result とし、
  accepted theorem type、formalization_proof_root_hash、proof check success、certificate binding を返してはいけない。
  一致せず、かつ optional_proof_candidate = Some(...) の場合は
  Rejected { error = FeatureRejected, feature_error = Some(Formalization(FormalizationProofStatementMismatch)) }
  として拒否する。

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

Phase 3 AI complete mode の elaboration / type check が必要な path で candidate statement の elaboration / type check が失敗した場合、
validator は
`Rejected { error = FeatureRejected, feature_error = Some(Formalization(CandidateStatementElaborationFailed)) }`
として拒否します。
これは `intent_record = None`、`Unreviewed`、`Reviewed` matching、`Reviewed` mismatch の比較用 elaboration、
および `optional_proof_candidate = Some(...)` の proof bridge 前 elaboration すべてに適用します。
`intent_record.status = Rejected` では candidate statement を elaboration / type check しないため、この分類には到達しません。
Phase 3 elaboration / type check が成功した後の Phase 4 proof bridge failure は、下の proof bridge failure 分類に従います。

`intent_record.status = Rejected` かつ `optional_proof_candidate = Some(...)` の場合は、
proof candidate 内の hash や intent record 内の candidate hash が一致するかどうかに関係なく
`Rejected { error = FeatureRejected, feature_error = Some(Formalization(RejectedIntentHasProofCandidate)) }`
として拒否します。
それ以外で `optional_proof_candidate = Some(...)` の場合、validator は先に candidate statement の Phase 3 elaboration /
type check を実行します。
その elaboration / type check が失敗した場合は、`optional_proof_candidate.candidate_statement_hash` が一致しているかどうかに関係なく
`Rejected { error = FeatureRejected, feature_error = Some(Formalization(CandidateStatementElaborationFailed)) }`
として拒否します。
`optional_proof_candidate.candidate_statement_hash` の一致検査は、Rejected intent の proof candidate 禁止検査と、
必要な candidate statement elaboration / type check が成功した後にだけ実行します。
その proof candidate では、`optional_proof_candidate.candidate_statement_hash` は、この candidate の `statement` から再計算した
`candidate_statement_hash` と一致しなければなりません。
一致しない場合は、proof candidate が別 statement に束縛されているため
`Rejected { error = FeatureRejected, feature_error = Some(Formalization(FormalizationProofStatementMismatch)) }`
として拒否します。
`optional_proof_candidate.tactic` は、その `statement` を Phase 3 AI complete mode で elaboration した
core theorem type に対してだけ検査します。
人間レビューで statement が編集され、`accepted_statement_hash` が elaborated candidate statement と一致しない場合、
この optional proof candidate は採用できません。
validator は `optional_proof_candidate = Some(...)` を silently ignore して成功にしてはいけません。
この場合は
`Rejected { error = FeatureRejected, feature_error = Some(Formalization(FormalizationProofStatementMismatch)) }`
として拒否し、caller は `optional_proof_candidate = None` で
statement / intent だけを intent-record-only result として検査するか、
accepted statement を `MachineFormalizationCandidate.statement` にした別 request として作り直します。
`optional_proof_candidate = Some(...)` かつ `accepted_statement_hash` が elaborated candidate statement と一致する場合、
validator は必ず proof bridge を実行します。
proof bridge が失敗した場合は request を拒否し、`CandidateStatementChecked` として downgrade してはいけません。
Phase 4 tactic が deterministic budget を使い切った場合は
`Rejected { error = BudgetExceeded, feature_error = None }` です。
Phase 4 tactic が MVP formalization bridge で許可しない primitive / option / registry を必要とする場合は
`Rejected { error = UnsupportedFeature, feature_error = None }` です。
proof bridge を実行する場合、`options.formalization.tactic_options` 内の imported ref が envelope imports から一意に解決できない場合は
`Rejected { error = ImportClosureMismatch, feature_error = None }` です。
その他の `start_machine_proof` failure、tactic validation failure、tactic execution failure、
open goal が残る success state、または closed proof extraction failure は
`Rejected { error = FeatureRejected, feature_error = Some(Formalization(ProofBridgeFailed)) }`
として拒否します。
closed proof term を accepted theorem type の proof として kernel が拒否した場合は
`Rejected { error = KernelRejected, feature_error = None }`
です。
`CandidateStatementChecked` を返せるのは `optional_proof_candidate = None` で、
かつ `intent_record = None` / `Unreviewed` / `Reviewed` with matching `accepted_statement_hash` の場合だけです。
`Reviewed` intent が candidate statement と異なる accepted statement を指し、
`optional_proof_candidate = None` の場合、validator は `IntentRecordOnly` を返せます。
`Rejected` intent の場合も、`optional_proof_candidate = None` であれば hash 整合性だけを検査する
`IntentRecordOnly` を返します。
`Rejected` intent に `optional_proof_candidate = Some(...)` が付いている場合は
`Rejected { error = FeatureRejected, feature_error = Some(Formalization(RejectedIntentHasProofCandidate)) }`
として拒否します。
ここでの一致判定は、elaborated candidate theorem type の canonical `CoreExpr` bytes に
`accepted_universe_params`、つまり `candidate.statement.universe_params` の canonical bytes と `target.env_fingerprint` を添え、
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
`accepted_statement_hash` という名前は、この request の environment で Machine Surface statement が
well-typed core theorem type として採用可能だった identity を指します。
人間または外部 reviewer が自然言語の意図一致を承認したことは意味しません。
verified mathematical intent と呼べるのは `intent_record.status = Reviewed` で、
その `accepted_statement_hash` が elaborated candidate statement から再計算した値と一致する場合だけです。

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
  この elaboration / type check が失敗した場合、validator は IntentRecordOnly に downgrade せず
  `Rejected { error = FeatureRejected, feature_error = Some(Formalization(CandidateStatementElaborationFailed)) }`
  として request を拒否する。
  intent_record.status = Rejected の場合、candidate.statement は elaboration / type check されず、
  accepted_statement_hash は存在せず、candidate.statement の elaboration result を theorem identity として返してはいけない。
  accepted theorem type / proof root / certificate binding は返さない。

ProofBridgeChecked:
  accepted_statement_hash が elaborated candidate statement と一致し、
  optional_proof_candidate を Phase 4 tactic bridge で検査して proof bridge が成功した。
```

Reviewed intent が candidate statement と異なる accepted statement を指し、
`optional_proof_candidate = None` の場合、MVP で返せる success kind は `IntentRecordOnly` だけです。
`optional_proof_candidate = Some(...)` の場合は
`Rejected { error = FeatureRejected, feature_error = Some(Formalization(FormalizationProofStatementMismatch)) }`
として拒否します。
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
初期 proof state を決定的に作り、`optional_proof_candidate.tactic` を1回だけ Phase 4 tactic として実行します。
MVP formalization bridge は Phase 7 search controller を起動しません。
AI / Phase 7 / 外部探索器を使う場合は、validator の外側で探索し、最終的に検査したい単一の
`MachineTacticCandidate` を `optional_proof_candidate.tactic` に入れて送ります。
複数 tactic の replay plan を formalization payload に直接入れる場合は、別 schema / profile を定義してから有効化します。
Phase 9 validator が `MachineTacticEnv` を手で組み立てて `options_fingerprint` や resolved family を省略してはいけません。
`options.formalization` は `Some` でなければなりません。
これは NaturalLanguageFormalization task の task options shape requirement であり、
`CandidateStatementChecked` や `IntentRecordOnly` でも例外はありません。
NaturalLanguageFormalization task options shape check は proof bridge の有無に関係なく実行します。
この shape check は `MachineTacticOptions` / `TacticBudget` の canonical decode、field order、enum tag、
nested canonical structure / sort / duplicate check、
および import 解決を必要としない scalar/range validation を含みます。
`options.formalization.tactic_options` bytes は、Phase 4 の `MachineTacticOptions canonical bytes`
そのものでなければなりません。
特に `simp_rules` は `name / decl_interface_hash / direction` の canonical order で strictly sorted され、
重複を含んではいけません。
Phase 9 validator は `simp_rules` を sort / dedup してから `options_hash` を計算してはいけません。
`simp_rules` の order violation または duplicate は、proof bridge の有無に関係なく
`Rejected { error = EnvelopeMalformed, feature_error = None }`
として拒否します。
これは Phase 4 の native caller 向け `start_machine_proof` が arbitrary `MachineTacticOptions` を受け取って
`simp_rules` を sort / dedup する canonicalize 挙動より前に行う、Phase 9 request bytes 用の stricter precheck です。
Phase 9 validator は non-canonical `simp_rules` を Phase 4 に渡して正規化させてはいけません。
caller input の sort order violation / duplicate は Phase 4 `InvalidTacticOption` や
`Error::InternalValidatorFailure` ではなく、常にこの段階の `EnvelopeMalformed` です。
この段階では `simp_rules`、`eq_family`、`nat_family` の imported refs は canonical bytes と `options_hash` /
`env_fingerprint` の入力として decode しますが、envelope imports への解決、public interface 検査、
Phase 4 `SimpRegistry` 構築は行いません。
`MachineTacticOptions.max_simp_rewrite_steps = 0`、`max_open_goals = 0`、または `max_metas = 0` は
Phase 4 では `InvalidTacticOption` ですが、Phase 9 options shape check では
`Rejected { error = EnvelopeMalformed, feature_error = None }`
として扱い、proof bridge を実行しない result path でも拒否します。
`TacticBudget` の fuel field は canonical `u64` として decode できれば shape check を通過し、`0` の意味は
proof bridge 実行時の Phase 4 budget rule に従います。
ただし proof bridge を実行しない result path では、`options.formalization.tactic_options` と
`options.formalization.tactic_budget` は上記 shape check と `options_hash` / `env_fingerprint` の入力にだけ使います。
その場合、tactic options 内の imported ref は import 解決や public interface 検査の対象にせず、
その ref が未解決でも `CandidateStatementChecked` / `IntentRecordOnly` を拒否してはいけません。
proof bridge を実行する場合だけ、次の Phase 4 tactic options validation と import 解決を行います。
`options.formalization.tactic_options` は、Phase 9 の precheck で既に Phase 4 `MachineTacticOptions canonical bytes`
と一致している値だけを Phase 4 proof bridge に渡します。
Phase 4 entrypoint が内部で `MachineTacticOptions` を canonicalize する場合、その canonical bytes と hash は
Phase 9 で受け付けた canonical bytes と byte-for-byte に一致することを確認する idempotent check としてだけ使い、
Phase 9 request の `simp_rules` を追加で sort / dedup して別の `options_fingerprint` を作ってはいけません。
一致しない場合は Phase 9 / Phase 4 bridge invariant の破れとして
`Error::InternalValidatorFailure` です。
`options.formalization.tactic_budget` は Phase 4 `TacticBudget` として canonicalize し、
`run_machine_tactic` の deterministic budget にそのまま渡します。
Phase 9 validator は hidden default budget、wall-clock timeout、または runtime configuration から budget を補完してはいけません。
`eq_family = None` や `nat_family = None` の意味も Phase 4 の `start_machine_proof` 規則に従います。
つまり builtin Eq がある profile では、`eq_family = None` でも `MachineTacticEnv.eq_family` は
resolved builtin Eq family の `Some(...)` になりえます。

```text
MachineProofSpec:
  module = formalization_scratch_module(formalization_proof_root_hash)
  theorem_name = formalization_scratch_theorem(formalization_proof_root_hash)
  source_index = 0
  universe_params =
    candidate.statement.universe_params を phase3_universe_param_ident で Vec<String> に写したもの
  theorem_type = elaborated candidate theorem type

start_machine_proof inputs:
  imports = envelope.imports as the checked VerifiedImportRef sequence
  checked_current_decls = []
  options = options.formalization.tactic_options

initial goal:
  universe_params =
    candidate.statement.universe_params を phase3_universe_param_ident で Vec<String> に写したもの
  local_context = []
  target = elaborated candidate theorem type

proof bridge execution:
  1. state0 = start_machine_proof(...)
  2. tactic = validate_machine_tactic_candidate(optional_proof_candidate.tactic)
  3. result = run_machine_tactic(state0, initial_goal_id, tactic, options.formalization.tactic_budget)
  4. result must be Success with no open goals
  5. closed_proof = extract_closed_machine_proof(result.state)
```

ここでの `start_machine_proof` は Phase 4 の `start_machine_proof_from_verified_imports` entrypoint を指します。
Phase 4 `MachineProofSpec.universe_params` と initial goal の universe parameter context は `Vec<String>` なので、
Phase 9 validator は `MachineSurfaceTerm.universe_params` を上で定義した `phase3_universe_param_ident` で変換して渡します。
`accepted_universe_params` と `candidate_statement_hash` / `accepted_statement_hash` の hash input には、
Phase 9 wire の `UniverseParam` sequence を canonical bytes のまま使い、Phase 4 用 `Vec<String>` bytes で代用してはいけません。
Phase 9 envelope の `imports` は `VerifiedImportRef` なので、Phase 4 の `VerifiedModule` 入力 API にそのまま渡してはいけません。
Phase 4 implementation が `VerifiedModule` 入力 API だけを公開している場合、Phase 9 bridge は同じ verified certificate bytes から
`VerifiedModule` を復元して Phase 4 に渡し、Phase 4 が導出した `VerifiedImportRef` の canonical bytes が envelope imports と
byte-for-byte に一致することを検査しなければなりません。
一致しない場合は import closure が replay input と一致しないため
`Rejected { error = ImportClosureMismatch, feature_error = None }` として拒否します。
この復元で workspace / package registry / current IDE session から missing import を補完してはいけません。

scratch module / theorem name は proof candidate ではなく `formalization_proof_root_hash` だけから導出します。
導出規則は次で固定します。

```text
lowerhex(hash):
  32-byte digest を 64 桁 lowercase ASCII hex にする。`sha256:` prefix は付けない。

formalization_scratch_module(hash):
  ModuleName components =
    ["NPA", "Phase9", "FormalizationScratch", lowerhex(hash)]

formalization_scratch_theorem(hash):
  append_name(module_name_as_name(formalization_scratch_module(hash)), name("theorem"))
```

`lowerhex(hash)` は1つの name component として扱い、8文字ごとに分割したり integer として再解釈してはいけません。
`module_name_as_name` は module component list を同じ順序の `Phase9Name` component list として扱う deterministic conversion です。
同じ `target.env_fingerprint` / `candidate_statement_hash` / `accepted_statement_hash` で
`optional_proof_candidate.tactic` だけが違う場合、Phase 4 root module / theorem name は同一でなければなりません。
`optional_proof_candidate.tactic` はこの initial goal からだけ実行します。
別の proof state、IDE session、checked current declaration、追加 simp registry、または
`options.formalization.tactic_options` 以外の tactic options を参照する proof candidate は拒否します。
Phase 4 tactic が `start_machine_proof` で解決された `MachineTacticEnv` 以外の family / registry を必要とする場合は、
MVP formalization proof bridge では `UnsupportedFeature` として拒否します。

採用フロー:

```text
1. AI が Machine Surface statement 候補を出す
2. MachineSurfaceTerm wrapper shape、term_canonical_bytes cap、Phase 3 Machine Surface term-source canonical decode を検査する
3. source document bytes cap と claim span range を hash 固定する
4. source document の declared size cap、artifact integrity、UTF-8、span、embedded hash を常に検査する
   rejection reason は intent_record.status = Rejected の場合だけ declared size cap、artifact integrity、UTF-8、embedded hash を検査する
   intent_record が reviewer を持つ場合は、その後に ReviewerId regex を検査する
5. intent_record.status = Rejected かつ optional_proof_candidate = Some(...) の場合は RejectedIntentHasProofCandidate で拒否する
6. intent_record.status = Rejected かつ optional_proof_candidate = None の場合は candidate / intent hash 整合性だけを検査し、IntentRecordOnly で終了する
7. それ以外の場合は Phase 3 AI complete mode で canonicalize / elaborate / type check し、失敗した場合は CandidateStatementElaborationFailed で拒否する
8. 必要なら人間が intent を確認する
9. proof candidate がある場合は elaboration 成功後に candidate_statement_hash 一致を確認し、elaborated candidate theorem type に対して単一 Phase 4 tactic bridge として検査する
10. certificate には accepted core declaration と proof だけを入れる
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

この文書の MVP では `/machine/phase9/smt/reconstruct` も提供しますが、
`rule_registry_profile = SmtRuleRegistryProfile::MvpEmptyRegistryV1` の間は deterministic rejection surface だけを持ち、
SMT certificate success は返しません。
SMT success path は、非空 solver-native registry を持つ `SmtRuleRegistryProfile` variant を別 schema / profile で定義してから有効化します。
したがって MVP の SMT fixture は success fixture ではなく、canonical schema、artifact hash、encoding binding、
proof payload shape、reconstruction plan pre-registry validation、unsupported registry rejection を再現できる
rejection fixture として用意します。

すべての endpoint は次を満たします。

```text
- request は canonicalizable な structured payload
- response は success / rejected / error を enum で返す
- pretty message は補助情報であり、判定には使わない
- same candidate_hash と same referenced artifact bytes なら same validation result
- time / random seed / network result を validation result hash に入れない
```

`Artifact` 参照を使う request では、`path` だけを replay input と見なしてはいけません。
replay input は request canonical bytes と、`file_hash` / `size_bytes` に一致する artifact bytes の組です。
同じ `candidate_hash` でも、path 上の現在の bytes が指定された `file_hash` / `size_bytes` と一致しない場合は deterministic な
`PayloadHashMismatch` として拒否します。

MVP response envelope は次で固定します。

```rust
enum Phase9AiEndpointResponse {
    Success {
        candidate_hash: Hash256,
        validation_result_hash: Hash256,
        payload: Phase9AiSuccessPayload,
    },
    Rejected {
        candidate_hash: Hash256,
        validation_result_hash: Hash256,
        error: Phase9AiValidationError,
        feature_error: Option<Phase9AiFeatureError>,
    },
    Error {
        error: Phase9AiEndpointError,
    },
}

enum Phase9AiFeatureError {
    AdvancedInductive(AdvancedInductiveError),
    UniverseRepair(UniverseRepairError),
    TypeclassResolution(TypeclassResolutionError),
    QuotientConstruction(QuotientConstructionError),
    SmtCertificate(SmtCertificateError),
    TheoremGraphQuery(TheoremGraphError),
    Formalization(FormalizationError),
}

enum Phase9AiEndpointError {
    NonCanonicalRequestBytes,
    ArtifactUnavailable,
    InternalValidatorFailure,
}
```

`Success` と `Rejected` は canonical request envelope を decode できた後の deterministic validation result です。
どちらも `candidate_hash` を持ちます。
`validation_result_hash` は次で固定します。

```text
validation_result_hash =
  sha256(
    "npa.phase9_ai.validation_result.v1"
    || candidate_hash digest bytes
    || canonical_bytes(success_or_rejected_tag)
    || canonical_bytes(payload_or_error)
  )
```

ここで `payload_or_error` は、`Success` では `Phase9AiSuccessPayload`、
`Rejected` では `{ error, feature_error }` の canonical bytes です。
`Rejected.error` は共通の拒否分類で、必要な場合だけ `feature_error` に task-specific enum を入れます。
`feature_error` は判定用の structured code であり、human-readable message ではありません。
この schema version で許可する `Rejected.error` と `feature_error` の組み合わせは次で固定します。
各 feature の本文で MVP では到達不能と明記した組み合わせは、同じ enum を使う future profile 用の予約組み合わせであり、
MVP validator はそれを validation result として返してはいけません。
ここで `Some(X(Y))` と明示していない場合、`feature_error = None` です。
本文で `UnsupportedFeature`、`KernelRejected`、`ImportClosureMismatch` など共通分類名だけを短縮表記する場合も、
その場で `feature_error = Some(...)` を明示していなければ `feature_error = None` として扱います。

```text
EnvelopeMalformed:
  canonical schema / sort order / range / nested canonical decode failure / task target shape violation / task options shape violation。
  top-level `imports` の sort order violation も feature_error = None。
  Phase9AiGoal / AdvancedInductive / QuotientConstruction の universe_params duplicate、
  またはそれぞれの payload-local LevelExpr の universe context 外参照も feature_error = None。
  Phase9AiGoal 内の `GlobalRef::Local` / `GlobalRef::LocalGenerated`、
  および QuotientConstruction payload 内の task-local `GlobalRef::Local` / `GlobalRef::LocalGenerated` も
  feature_error = None。
  AdvancedInductive / QuotientConstruction の vector length protocol cap、nested CoreExpr / LevelExpr count cap violation も
  feature_error = None。
  ArtifactPath の path shape violation / symlink escape validation failure も feature_error = None。
  options bytes cap violation も feature_error = None。
  UniverseRepair の `instantiations` / `constraint_hints` / `occurrence.path` / `explicit_level_args` protocol cap violation、
  `instantiations` / `constraint_hints` の sort order violation または duplicate も feature_error = None。
  Common options の `approved_nested_type_constructors` / `class_declarations` list length cap violation も feature_error = None。
  TypeclassResolution の `ordered_candidates` duplicate target、`max_depth` / `max_nodes` protocol cap violation、
  `ordered_candidates` list length cap violation も feature_error = None。
  Formalization `MachineSurfaceTerm` の canonical decode failure / universe_params duplicate / Phase 3 identifier incompatibility、
  `term_canonical_bytes` length cap violation、source/rejection reason の byte cap violation / invalid UTF-8、
  claim_span の byte range / UTF-8 boundary violation、
  ReviewerId regex violation は feature_error = None。
  feature_error = Some(SmtCertificate(NonCanonicalPayload))
    SMT payload table の nested decode、certificate_format、step/node count cap、nested CoreExpr / SmtExpr / SmtSortExpr count cap、
    node canonical order、acyclicity、payload table shape、proof payload `SmtConclusionEncoding.encoded_expr` の representation-only canonical shape、
    command / core-expression / expression / sort count cap、command phase/order/id uniqueness/symbol table/sort validation、
    encoded problem / proof payload raw bytes cap、
    proof payload `SmtConclusionEncoding.core_expr` の universe scope / task-local global ref、
    reconstruction plan の steps count cap / nested vector cap / nested CoreExpr / LevelExpr count cap / step id / final_step /
    `CoreExpr` task-local global ref / imported_theory_refs sort order・duplicate・unused ref /
    local bookkeeping explicit level argument scope
    canonical shape の違反。
    proof payload validation では unknown enum tag は step 1 の scalar field decode failure、
    known-but-mismatched `SmtProofNodeTable.certificate_format` は step 3 の table shape validation として扱う。
    同じ proof payload bytes に `payload_hash` mismatch がある場合は step 2 の `PayloadHashMismatch` が優先する。
  feature_error = Some(TheoremGraphQuery(SnapshotMalformed))
    theorem graph snapshot bytes の nested decode、nodes / edges count cap、source_release_hash / extractor_version metadata mismatch、
    raw bytes cap、sort order、edge target 整合性の違反。
  feature_error = Some(TheoremGraphQuery(QueryFeaturesMalformed))
    query feature bytes の nested decode、features count cap、feature_schema_version / target metadata mismatch、
    raw bytes cap、key ASCII grammar、sort order、重複 key、value kind 違反。
  feature_error = Some(TheoremGraphQuery(LimitOutOfRange))
    theorem graph query limit が `0..=256` の wire validation 範囲外。

TargetFingerprintMismatch:
  env_fingerprint / goal_fingerprint / target_decl_hash / expected_decl_hash の再計算 mismatch。
  feature_error = Some(UniverseRepair(TargetFingerprintMismatch))
    universe repair の goal_fingerprint mode で payload.goal / target_expr から再計算した goal binding が request target と一致しない場合。
    env_fingerprint mismatch には使わない。
  その他の target fingerprint mismatch は feature_error = None。
  SMT encoded problem の goal_fingerprint mismatch と theorem graph query の target binding mismatch も feature_error = None。

ImportClosureMismatch:
  imported ref が envelope imports から一意に解決できない、import closure が不足する、
  または同じ `(module, export_hash, certificate_hash)` import tuple が重複している。
  Phase 9 wire `CoreExpr` 内の `GlobalRef::Imported(import_index, name, decl_interface_hash)` で
  `import_index` が範囲外、または指定 import の export table に `(name, decl_interface_hash)` が一意に存在しない場合もここに含む。
  proof payload `SmtConclusionEncoding.core_expr` 内の imported ref 解決失敗、
  SMT reconstruction plan の `final_proof` / step `conclusion` / step `proof` / LocalBookkeeping `term_args` 内の
  imported ref 解決失敗、および `imported_theory_refs` 内の ref 解決失敗もここに含む。
  feature_error = None。

PayloadHashMismatch:
  artifact file_hash / size_bytes / embedded hash / referenced payload hash / options_hash の再計算 mismatch。
  SMT encoded problem の problem_hash / encoding_hash mismatch、command_id recomputation mismatch、
  および PayloadNode binding の payload_hash mismatch もここに含む。
  TheoremGraph snapshot / query_features embedded hash mismatch もここに含む。
  Formalization の candidate.source_document / claim_span / rejection_reason ref 内 embedded hash の再計算 mismatch もここに含む。
  intent_record と candidate の hash consistency mismatch、および
  `FormalizationIntentStatus::Rejected.rejection_reason_hash` と validated rejection reason hash の mismatch は
  Formalization(IntentRecordMismatch) であり、ここには含めない。
  feature_error = None。

KernelRejected:
  core term の type check / defeq / proof check を kernel が拒否した。
  Phase9AiGoal の `local_context` / `target` well-typedness check が、goal_fingerprint binding 成功後に失敗した場合もここに含む。
  feature_error = Some(QuotientConstruction(EquivalenceProofMismatch))
    quotient equivalence_proof が期待型の proof term として kernel check を通らない場合。
  feature_error = Some(QuotientConstruction(CompatibilityProofMismatch))
    quotient operation の compatibility_proof が期待型の proof term として kernel check を通らない場合。
  その他の generic kernel rejection は feature_error = None。
  SmtCertificate(ReconstructionProofMismatch) は SMT proof reconstruction の FeatureRejected にだけ使う。

IndependentCheckerRejected:
  resulting certificate を independent checker が拒否した。
  feature_error = None。

NonDeterministicResult:
  同一 replay input で validation result が安定しないことを validator が検出した。
  feature_error = None。

BudgetExceeded:
  deterministic budget を使い切った。
  feature_error = None。

AmbiguousResolution:
  複数の異なる canonical result が得られた。
  feature_error = None。

NoSolution:
  探索空間を尽くしたが solution が存在しなかった。
  feature_error = Some(TypeclassResolution(NoSolution))。
  feature_error = Some(UniverseRepair(UnsatisfiedConstraint))
    universe constraints が canonical solver で充足不能だった場合。

UnsupportedFeature:
  payload が well-formed でも MVP profile / checker / rule registry がその機能をサポートしない。
  feature_error = Some(AdvancedInductive(PositivityProfileUnsupported))
    requested inductive shape、positivity / nested / higher-order / large-elimination profile を MVP がサポートしない場合。
    common options の `advanced_inductive.approved_nested_type_constructors` が non-empty の場合も、
    task kind に関係なくこの分類を使う。
  feature_error = Some(AdvancedInductive(ArtifactGeneratorUnavailable))
    Phase 2 canonical artifact generator が対象 profile を実装していない場合。
  feature_error = Some(TypeclassResolution(ClassHeadUnsupported))
    obligation target の head が MVP typeclass class declaration として扱えない場合。
  SMT encoded problem / proof payload が selected `SmtLogic` で許可しない builtin sort / literal / operator を含む場合は
    feature_error = None。
  feature_error = Some(SmtCertificate(RuleRegistryMismatch))
    SMT solver-native proof rule registry に rule が存在しない、または premise order を一意に定義できない場合。
  非空 solver-native registry profile で PayloadNode step が複数 payload binding を持つ場合は
    feature_error = None。
  SMT `SmtConclusionEncoding.core_expr` は well-typed proposition だが selected deterministic encoder table が
    対応 mapping を持たない場合、または必要な `prop_false` / `prop_not` recognition head が `options.smt` にない場合は
    feature_error = None。
  その他の profile-wide unsupported case は feature_error = None。

FeatureRejected:
  schema、hash、import、kernel、budget、ambiguity、unsupported のいずれでもない feature 固有の決定的拒否。
  feature_error は必ず Some でなければならない。
  AdvancedInductive:
    TargetRefMismatch, GeneratedArtifactMismatch, NameCollision
    GeneratedArtifactMismatch は future generated-artifact payload/profile 用の予約 variant であり、
    MVP validator は返してはいけない。
  UniverseRepair:
    UnknownUniverseParam, IllFormedLevelExpr, NonCanonicalSolution,
    InvalidOccurrencePath, AmbiguousOccurrence, TargetRefMismatch, ConstraintHintMismatch
    NonCanonicalSolution は caller-provided solution bytes / external solver certificate を検査する future profile 用の
    予約 variant であり、MVP validator は返してはいけない。MVP の validator-derived `repaired_expr` /
    `constraint_set_hash` を再計算できない場合は `Error::InternalValidatorFailure`。
    AmbiguousOccurrence は future selector profile 用の予約 variant であり、
    MVP の concrete path profile は返してはいけない。
  TypeclassResolution:
    ClassDeclarationMismatch, CandidateInterfaceInvalid
  QuotientConstruction:
    PrimitiveInterfaceMismatch, UniverseLevelMismatch,
    QuotientTypeMismatch, RelationTypeMismatch, RawFunctionTypeMismatch。
    TargetRefMismatch は future target-bound quotient repair profile 用の予約 variant であり、
    MVP の new quotient construction validator は返してはいけない。
  SmtCertificate:
    EncodingMismatch, RuleFingerprintMismatch, ConclusionEncodingMismatch,
    PayloadBindingMismatch, ReconstructionConclusionMismatch, ReconstructionPremiseMismatch,
    ReconstructionProofMismatch, PublicInterfaceMismatch, TheoryRefMismatch
    EncodingMismatch は deterministic SMT encoder output / command symbol mismatch に加え、
    encoded problem が payload と異なる logic を持つ場合にも使う。
    deterministic encoder table が対応 mapping を持たない場合は EncodingMismatch ではなく UnsupportedFeature + feature_error = None。
    RuleFingerprintMismatch は、payload が rule descriptor または rule name binding を canonical data として持つ
    future certificate format/profile 用の予約 variant であり、`MvpProofNodeTableV1` schema では返してはいけない。
    registry source 内の descriptor fingerprint が registry key と一致しない場合は candidate rejection ではなく
    Error::InternalValidatorFailure。
    PublicInterfaceMismatch は `options.smt.eq` / `prop_false` / `prop_not` が解決できたが期待 public interface と一致しない場合。
    TheoryRefMismatch は local bookkeeping step が `imported_theory_refs` に含まれない theorem / combinator を参照する場合。
    `imported_theory_refs` に含まれるが未使用の theorem / combinator は TheoryRefMismatch ではなく
    EnvelopeMalformed + SmtCertificate(NonCanonicalPayload)。
    PayloadBindingMismatch は PayloadNode step が payload_bindings を持たない場合、
    payload binding が node table 内の node に解決できない場合、binding fingerprint が一致しない場合、
    または LocalBookkeeping step が payload_bindings を持つ場合に使う。
    PayloadNode step が複数 payload binding を持つ場合は PayloadBindingMismatch ではなく
    UnsupportedFeature + feature_error = None。
    ReconstructionProofMismatch は、再構成 proof bytes が `step.proof` と一致しない場合に加え、
    local bookkeeping explicit level argument arity mismatch、proof term を一意に構成できない場合、
    または reconstructed proof type が `step.conclusion` と一致しない場合にも使う。
    ConclusionEncodingMismatch、ReconstructionProofMismatch、および PayloadNode-specific の
    ReconstructionConclusionMismatch / ReconstructionPremiseMismatch は、
    非空 solver-native registry を持つ `SmtRuleRegistryProfile` variant でだけ到達する。
    ただし LocalBookkeeping の IntroduceTheoryLemma が non-empty premises を持つ場合の
    ReconstructionPremiseMismatch は MVP の pre-registry validation でも到達する。
    PayloadBindingMismatch は MVP でも pre-registry validation で LocalBookkeeping step が payload_bindings を持つ場合に到達する。
    MVP empty registry では、pre-registry validation を通過した PayloadNode request は
    UnsupportedFeature + RuleRegistryMismatch になる。
  TheoremGraphQuery:
    NodeResolutionMismatch
  Formalization:
    IntentRecordMismatch, CandidateStatementElaborationFailed,
    FormalizationProofStatementMismatch, RejectedIntentHasProofCandidate,
    ProofBridgeFailed
```

上の表にない `Rejected.error` / `feature_error` の組み合わせは canonical validation result として出してはいけません。
追加の組み合わせが必要な場合は、この表か各 feature-specific enum を更新してから有効化します。

`Error` は request envelope を canonical decode できない、artifact bytes を取得できない、validator 自体が内部 invariant を破った、
など validation result を構成できない場合だけに使います。
top-level request envelope を canonical decode できない場合だけ `Error::NonCanonicalRequestBytes` です。
top-level envelope を decode でき、`candidate_hash` を計算できた後は、options、payload、inline artifact bytes、
または referenced artifact bytes の nested canonical decode failure は deterministic `Rejected` として返します。
この場合の共通分類は、bytes が期待 schema として decode できないなら `EnvelopeMalformed`、
hash / size / embedded hash が再計算値と一致しないなら `PayloadHashMismatch` です。
`Artifact.file_hash` / `size_bytes` が取得済み bytes と一致しない場合は `Error` ではなく、
canonical request に対する deterministic `Rejected { error = PayloadHashMismatch, ... }` です。

`/machine/phase9/smt/reconstruct` は `Phase9AiTaskKind::SmtCertificate` を検査する endpoint 名です。
API response / error variant は task kind と同じ `SmtCertificate` に揃えます。
`MachineSmtReconstructionPlan`、`SmtReconstructionRule`、`SmtReconstructionStepId` は SMT certificate payload 内の
proof reconstruction substructure の名前であり、別 task kind ではありません。

Phase 9 AI response schema の success payload は endpoint ごとに固定します。
`SmtCertificate` variant は非空 solver-native registry profile で SMT success path を有効化した場合の schema です。
この文書の MVP empty registry では到達不能であり、`/machine/phase9/smt/reconstruct` はこの variant を返しません。

```rust
enum Phase9AiSuccessPayload {
    AdvancedInductive {
        decl_interface_hash: Hash256,
        decl_certificate_hash: Hash256,
    },
    UniverseRepair {
        repaired_expr: CoreExpr,
        constraint_set_hash: Hash256,
    },
    TypeclassResolution {
        proof: CoreExpr,
    },
    QuotientConstruction {
        decl_certificate_hash: Hash256,
    },
    SmtCertificate {
        final_proof: CoreExpr,
    },
    TheoremGraphQuery {
        result: MachineTheoremGraphResult,
    },
    NaturalLanguageFormalization {
        kind: FormalizationSuccessKind,
        accepted_statement_hash: Option<Hash256>,
        formalization_proof_root_hash: Option<Hash256>,
    },
}

enum FormalizationSuccessKind {
    CandidateStatementChecked,
    IntentRecordOnly,
    ProofBridgeChecked,
}
```

`UniverseRepair.repaired_expr` は `target_expr` に `instantiations` を適用した後の canonical `CoreExpr` です。
`constraint_set_hash` は patch 適用後に validator が導出して canonical sort した universe constraint list から
`sha256("npa.phase9_ai.universe.constraints.v1" || canonical_bytes(sorted_constraints))` で計算します。
`TypeclassResolution.proof` と `SmtCertificate.final_proof` は payload goal context の下の open proof term です。
standalone certificate に出す場合は common envelope の `close_goal_proof` 規則で閉じます。
MVP で success 前に independent checker を実行する endpoint は、validator が standalone certificate candidate を
この endpoint 内で deterministic に構成するものだけです。
`AdvancedInductive` は single-declaration inductive certificate package、`QuotientConstruction` は quotient type `DefDecl` certificate package、
`NaturalLanguageFormalization` の `ProofBridgeChecked` は scratch theorem certificate を構成し、success を返す前に
Phase 8 independent checker へ渡します。
`SmtCertificate` は current `SmtRuleRegistryProfile::MvpEmptyRegistryV1` では success path がないため checker 実行に到達しません。
future schema / profile で非空 solver-native registry による success path を有効化する場合は、
`close_goal_proof` で閉じた theorem certificate を構成し、success を返す前に Phase 8 independent checker へ渡します。
これらの endpoint では、common envelope / options validation、task options shape check、feature-specific validation、
artifact hash binding、import resolution、kernel / reconstruction checks、expected hash binding など、
resulting certificate candidate とその feature set を一意に決める検査を先に終えます。
同じ request に payload hash mismatch、import mismatch、kernel rejection、feature-specific rejection などがある場合、
それらが `options.independent_checker.profile` の support check より優先です。
support check は、上記検査を通過して exact resulting certificate feature set が確定した後、checker へ certificate bytes を渡す直前にだけ行います。
この時点で `options.independent_checker.profile` が必要 feature をサポートしないと support matrix から判定できる場合は、
independent checker を実行せず
`Rejected { error = UnsupportedFeature, feature_error = None }`
を返します。
required feature 対応 profile として independent checker を実行し、その checker が resulting certificate を拒否した場合だけ
`Rejected { error = IndependentCheckerRejected, feature_error = None }`
を返します。
checker profile は `options_hash` と `env_fingerprint` に含まれるため、同じ request canonical bytes と同じ artifact bytes で
checker profile だけが runtime により変わって validation result が変化してはいけません。
`IndependentCheckerRejected` は、required feature 対応 profile として checker を実行した後の certificate-level rejection だけに使います。
`UniverseRepair`、`TypeclassResolution`、`TheoremGraphQuery`、および `NaturalLanguageFormalization` の
`CandidateStatementChecked` / `IntentRecordOnly` は MVP response 自体では certificate candidate を構成しないため、
この endpoint 内では `IndependentCheckerRejected` を返してはいけません。
caller が後段で certificate artifact を作る場合、その artifact は Phase 8 の通常の checker workflow で検査します。
SMT endpoint の `Success::SmtCertificate.final_proof` は、非空 solver-native registry profile で success path を有効化した場合、
API response では open proof term だけを返しますが、
validator は success を返す前にこの open proof term を `close_goal_proof` で閉じた certificate candidate を内部で構成し、
independent checker に渡して再検査します。
この closed certificate bytes は MVP response payload には含めません。
caller が certificate artifact を保存したい場合は、同じ closure rule で deterministic に再構成します。
`QuotientConstruction.decl_certificate_hash` は validator が再構成した quotient declaration の
Phase 2 `decl_certificate_hash` です。
payload `expected_decl_hash = Some(h)` の場合は `h` と一致していなければならず、
`expected_decl_hash = None` の場合は caller がこの response field を discovery result として使えます。
`NaturalLanguageFormalization.kind = IntentRecordOnly` の場合、
`accepted_statement_hash` と `formalization_proof_root_hash` は `None` でなければなりません。
`Reviewed` mismatch の input record に `accepted_statement_hash` が存在する場合でも、
MVP response の `NaturalLanguageFormalization.accepted_statement_hash` にはコピーしません。
この response field は、この request で candidate statement から検査済み theorem identity として採用できた
accepted statement だけを返します。
`CandidateStatementChecked` では `accepted_statement_hash = Some(...)`、`formalization_proof_root_hash = None` です。
`ProofBridgeChecked` では両方 `Some(...)` でなければなりません。

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
    NoSolution,
    FeatureRejected,
    UnsupportedFeature,
}
```

feature-specific error はこの enum の下にぶら下げます。
human-readable message は持ってよいですが、hash や replay 判定には使いません。
`Phase9AiEndpointResponse`、`Phase9AiSuccessPayload`、`FormalizationSuccessKind`、
`Phase9AiValidationError`、`Phase9AiFeatureError`、`Phase9AiEndpointError`、
および下の feature-specific error enum の canonical bytes は、すべて enum declaration order の variant tag と
variant field declaration order で固定します。
既存 variant の順序や tag を変更してはいけません。variant を追加する場合は末尾へ追加するか、
別 schema/profile version を定義します。

```rust
enum AdvancedInductiveError {
    TargetRefMismatch,
    PositivityProfileUnsupported,
    ArtifactGeneratorUnavailable,
    GeneratedArtifactMismatch,
    NameCollision,
}

enum TypeclassResolutionError {
    ClassDeclarationMismatch,
    CandidateInterfaceInvalid,
    ClassHeadUnsupported,
    NoSolution,
}

enum QuotientConstructionError {
    TargetRefMismatch,
    PrimitiveInterfaceMismatch,
    UniverseLevelMismatch,
    CompatibilityProofMismatch,
    QuotientTypeMismatch,
    RelationTypeMismatch,
    EquivalenceProofMismatch,
    RawFunctionTypeMismatch,
}

enum SmtCertificateError {
    EncodingMismatch,
    RuleFingerprintMismatch,
    RuleRegistryMismatch,
    NonCanonicalPayload,
    ReconstructionProofMismatch,
    ConclusionEncodingMismatch,
    PayloadBindingMismatch,
    ReconstructionConclusionMismatch,
    ReconstructionPremiseMismatch,
    PublicInterfaceMismatch,
    TheoryRefMismatch,
}

enum TheoremGraphError {
    SnapshotMalformed,
    QueryFeaturesMalformed,
    NodeResolutionMismatch,
    LimitOutOfRange,
}

enum FormalizationError {
    IntentRecordMismatch,
    CandidateStatementElaborationFailed,
    FormalizationProofStatementMismatch,
    RejectedIntentHasProofCandidate,
    ProofBridgeFailed,
}
```

これらの feature-specific enum は repair loop 向けの安定した補助分類です。
branch failure や kernel/checker の詳細をすべて列挙するものではありません。
共通分類だけで十分な rejected result では `feature_error = None` を使ってよく、
本文で `feature-specific ...` と書く場合は、この節の closed enum または
Universe repair 節で定義した `UniverseRepairError` のいずれかに対応させます。

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

# 13. MVP / Follow-up Milestones

推奨実装順序:

```text
M1  common envelope / candidate hash / error model
M2  universe repair candidate validation
M3  advanced inductive proposal validation
M4  theorem graph query with certificate-bound node refs
M5  typeclass resolution plan replay
M6  quotient construction candidate validation
M7a SMT canonical schema / command / proof-payload deterministic rejection validation
M8  natural language formalization statement / proof bridge check
M9  Phase 8 audit integration for all AI sidecars

Post-MVP:
P1  SMT success profile with non-empty encoder table and solver-native rule registry
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
