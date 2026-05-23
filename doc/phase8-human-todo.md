# Phase 8 Human Task Breakdown

このタスク分解は `doc/phase8-human.md` を正とし、現在の
`crates/npa-cert` / `crates/npa-kernel` / `crates/npa-api` 実装との差分を、
independent checker / external checker / CI audit の実装マイルストーンに分けたものです。

Phase 8 Human は、source / tactic / elaborator / AI search / theorem search を信用せず、
canonical `.npcert` を本体 kernel とは別経路で再検査するための層です。
`crates/npa-api` の Phase 8 checker audit automation は、checker request / result / audit artifact を
構成・正規化する非信頼 orchestration substrate です。
証明の受理根拠は、canonical certificate と independent checker が返す deterministic result だけです。

重要な制約:

```text
- reference checker / external checker は .npa source、tactic script、AI search trace、theorem index を読まない。
- reference checker は fast kernel / npa-cert verifier と type checker、conversion checker、inductive checker 実装を共有しない。
- `npa_cert::verify_module_cert` を reference checker verdict の代替にしない。
- external checker は別プロセス / 別バイナリとして実行し、network / plugin / source directory access を持たない。
- Phase 8 automation、AI sidecar、challenge generator、audit summary は trusted base に入れない。
- Phase 8 audit は Phase 5 / Phase 7 の AI 候補生成 hot path に同期挿入しない。
- PR mode の required checker profile は `reference` にとどめ、external checker は optional / on-demand とする。
- nightly / release / high-trust mode では external checker と full audit を required にする。
- custom axiom / sorry は拒否する。現在の kernel が `Std.Logic.Eq.rec` を標準 recursor axiom として emit する場合だけ exact exception を許可する。
- kernel crate に I/O、network、plugin loading、AI 呼び出し、checker runner state を入れない。
```

---

## 0. 現在の実装境界

### 0.1 実装済みとして扱うもの

現在の `crates/npa-cert` には、fast verifier と certificate codec があります。
Phase 8 Human 実装では、これを比較対象・fixture 生成元として使ってよいですが、
reference checker の verdict 実装として再利用してはいけません。

```text
crates/npa-cert/src/lib.rs
- build_module_cert / encode_module_cert / decode_module_cert / verify_module_cert
- AxiomPolicy / VerifierSession / VerifiedModule

crates/npa-cert/src/binary.rs
- canonical binary encode / decode

crates/npa-cert/src/hash.rs
- term / declaration / export / certificate / axiom report hash

crates/npa-cert/src/verify.rs
- current fast verifier implementation
```

現在の `crates/npa-kernel` には、fast kernel の型検査・conversion・inductive checker があります。
reference checker の test oracle として比較してよいですが、内部実装をそのまま呼び出して
independent checker と主張してはいけません。

```text
crates/npa-kernel/src/lib.rs
crates/npa-kernel/src/expr.rs
crates/npa-kernel/src/level.rs
crates/npa-kernel/src/env.rs
crates/npa-kernel/src/error.rs
crates/npa-kernel/src/decl.rs
```

現在の `crates/npa-api` には、Phase 8 AI Profile の checker audit automation substrate があります。
これは Human Profile の standalone checker / CI integration が出す request / result / bundle を受ける側です。

```text
crates/npa-api/src/independent_checker.rs
- MachineCheckRequest / MachineCheckResult
- RunnerPolicy / ImportLockManifest / AxiomPolicy TOML
- NormalizedCheckResult / comparison / disagreement
- challenge generation / materialize / replay / coverage summary
- AuxiliaryResult / ReleaseAuditBundleManifest
- AI audit sidecar validation / required sidecar diagnostic gate
- training export labels from checker result only
```

### 0.2 未実装の Phase 8 Human 範囲

`doc/phase8-human.md` が要求する以下の範囲は、まだ full Phase 8 Human implementation として完了していません。

```text
standalone reference checker implementation
source-free certificate decoder inside the reference checker boundary
reference checker hash verifier
reference checker import environment builder
reference checker minimal type checker
reference checker conversion checker
reference checker simple inductive / recursor checker
reference checker axiom report recomputation
external checker binary and process runner
challenge file statement hash enforcement
audit bundle generation / validation through real checker outputs
fast kernel / reference / external checker comparison CI
release / high-trust full independent check gate
performance benchmark policy that does not slow AI candidate hot path
```

### 0.3 AI hot path に入れてはいけないもの

以下は Phase 8 Human の audit / CI / high-trust flow で使ってよいが、
AI の大量候補生成・検索・tactic 実行経路に同期挿入してはいけません。

```text
reference checker process
external checker process
audit bundle generation
challenge mutation / challenge replay
AI sidecar triage / summary / suggested challenge
Human source file lookup
source map and pretty goal rendering
release audit bundle validation
full recursive import certificate recheck
nightly / release benchmark collection
```

AI path は次の形を維持します。

```text
Machine Surface request
  -> Phase 5 machine session / tactic batch / replay / verify
  -> Phase 7 candidate ranking / repair / minimization
  -> closed certificate candidate
  -> optional post-acceptance / CI / release audit
```

---

## 1. AI 向け高速経路を守る設計ルール

Phase 8 Human の各マイルストーンでは、次を acceptance criteria として扱います。

```text
- `/machine/*` endpoint の request / response schema を変更しない。
- Machine `candidate_hash`、`state_fingerprint`、`replay` / `verify` identity hash を Phase 8 audit metadata で変えない。
- tactic candidate expansion ごとに reference / external checker を同期実行しない。
- Phase 5 verify response の前に AI sidecar / challenge generation を必須化しない。
- premise retrieval を未生成の Phase 8 audit result で block しない。
- materialized certificate hash / NormalizedCheckResult / audit summary を使う場合は cache/ranking feature に限定する。
- Phase 8 AI sidecar は checker result と NormalizedCheckResult.comparison を上書きできない。
- PR mode は reference checker の changed-cert check を required とし、external checker は optional / on-demand にする。
- nightly / release / high-trust mode は external checker と full audit を required にする。
- kernel / certificate verifier / independent checker 以外を proof acceptance boundary にしない。
```

---

## 2. 実装順

Phase 8 Human は、reference checker の certificate-only boundary を先に固定し、その後に checker 本体、
external process runner、CI / release audit を積みます。
既存の `crates/npa-api/src/independent_checker.rs` は request / result / audit artifact substrate として再利用します。

```text
1. Phase 8 Human / AI audit boundary と regression guard を固定する
2. reference checker crate / API skeleton を作る
3. source-free canonical certificate decoder を実装する
4. hash verifier を実装する
5. import store / environment builder を実装する
6. minimal type checker を実装する
7. conversion checker を実装する
8. simple inductive / recursor checker を実装する
9. axiom report recomputation / axiom policy を実装する
10. Std.Logic / Std.Nat / Std.List / Std.Algebra.Basic を reference checker で再検査する
11. external checker binary / runner を実装する
12. checker result normalization / disagreement CI を接続する
13. challenge mode / audit bundle を接続する
14. fuzzing / mutation / differential testing を固定する
15. CI modes / performance gates を固定する
16. docs / release completion gate を固定する
```

各段階で少なくとも以下を確認します。

```sh
cargo fmt --all
cargo test -p npa-cert
cargo test -p npa-kernel
cargo test -p npa-api independent_checker
cargo test -p npa-api std_library
cargo test -p npa-api ai_search
```

大きな内部変更後は次も通します。

```sh
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/phase9-regression.sh
```

---

## 3. タスク一覧

### P8H-00: Phase 8 Human / AI audit boundary を固定する

実装タスク:

- [x] `doc/phase8-human.md`、`doc/phase8-ai.md`、README の境界を実装コメントまたは test 名に反映する。
- [x] `crates/npa-api` の checker audit automation が trusted checker ではないことを public API docs に明記する。
- [x] Phase 8 audit が Phase 5 / Phase 7 AI hot path に同期挿入されない regression を追加する。
- [x] PR / nightly / release / high-trust mode の required checker profile 差分を test fixture に固定する。
- [x] `Std.Logic.Eq.rec` の exact standard exception と custom axiom 禁止を Phase 8 policy docs / tests に接続する。

受け入れ条件:

- [x] AI sidecar、challenge generator、audit summary が checker verdict を作れないことが test で固定されている。
- [x] PR mode の required profile が `reference` だけで、external checker が optional / on-demand であることが fixture で固定されている。
- [x] `/machine/*` request / response schema、candidate hash、state fingerprint が Phase 8 境界追加で変わらない。

検証:

```sh
cargo test -p npa-api independent_checker
cargo test -p npa-api ai_search
cargo test -p npa-api phase7
```

依存:

```text
None
```

注意:

```text
この milestone は reference checker 本体を実装しない。境界と regression guard だけを固定する。
```

### P8H-01: reference checker crate / API skeleton を作る

実装タスク:

- [x] reference checker の置き場所を固定する。候補は新規 `crates/npa-checker-ref` または同等の独立 crate。
- [x] public API を `check_certificate(cert_bytes, import_store, policy) -> ReferenceCheckResult` 形で固定する。
- [x] reference checker が `.npa` source、tactic script、AI trace、theorem index を受け取れない型にする。
- [x] fast kernel / `npa_cert::verify_module_cert` を呼ばずに result を返す skeleton を作る。
- [x] error enum を structured / deterministic にし、human string だけに依存しない test を追加する。

受け入れ条件:

- [x] reference checker crate は `npa-api` に依存しない。
- [x] reference checker crate は `npa-tactic` / `npa-frontend` に依存しない。
- [x] skeleton は source-free empty / malformed certificate を deterministic error として返す。
- [x] `unsafe` を使わない。必要になった場合は境界と代替案を文書化する。

検証:

```sh
cargo test -p npa-checker-ref
cargo test -p npa-api independent_checker
cargo clippy --workspace --all-targets -- -D warnings
```

依存:

```text
P8H-00
```

注意:

```text
certificate format の仕様・golden fixtures は共有してよい。type checker / conversion checker / inductive checker 実装は共有しない。
```

### P8H-02: source-free canonical certificate decoder を実装する

実装タスク:

- [x] `.npcert` canonical binary を reference checker boundary 内で decode する。
- [x] magic / format version / core spec version / section order / unknown tag を検査する。
- [x] name / level / term / declaration table の canonical order と dangling reference を検査する。
- [x] unused table entry、duplicate name、non-normalized level entry を deterministic error にする。
- [x] source path、source map、debug JSON なしで decode test fixture を通す。

受け入れ条件:

- [x] valid golden certificate は decode できる。
- [x] noncanonical だが意味的に読める certificate は reject される。
- [x] decode error の kind / section / offset が test で比較可能である。
- [x] decoder は import 解決、type checking、AI sidecar validation をしない。

検証:

```sh
cargo test -p npa-checker-ref decode
cargo test -p npa-cert
```

依存:

```text
P8H-01
```

注意:

```text
P8H-02 は semantic check をしない。decode / canonical shape / table reachability だけを担当する。
```

### P8H-03: reference checker hash verifier を実装する

実装タスク:

- [x] term hash を reference checker 内で再計算する。
- [x] declaration interface / declaration certificate hash を再計算する。
- [x] export hash / certificate hash / axiom report hash を再計算する。
- [x] domain separation tag を `doc/core-spec-v0.1.md` / `doc/phase2.md` と照合する。
- [x] stored hash を信じず、再計算結果との mismatch を structured error にする。

受け入れ条件:

- [x] hash mismatch の対象 object が deterministic に分類される。
- [x] timestamp、path、source text、checker version は certificate hash 入力に入らない。
- [x] fast verifier と同じ golden certificate hash を得るが、fast verifier の hash helper を verdict として使わない。
- [x] hash verifier は type correctness を acceptance 根拠にしない。

検証:

```sh
cargo test -p npa-checker-ref hash
cargo test -p npa-cert golden_hashes
```

依存:

```text
P8H-02
```

注意:

```text
hash helper の機械的な codec fixture 共有は許容するが、fast verifier の pass/fail を reference checker の pass/fail にしない。
```

### P8H-04: import store / environment builder を実装する

実装タスク:

- [x] import certificate store を source-free bytes / checked module interface から構築する。
- [x] normal mode で import `export_hash` を検査する。
- [x] high-trust mode で import `certificate_hash` と same-checker checked status を検査する。
- [x] import public environment を canonical order で現在 module environment に追加する。
- [x] missing import、export hash mismatch、certificate hash mismatch、duplicate import を deterministic error にする。

受け入れ条件:

- [x] import name だけで解決しない。必ず `export_hash` を binding として使う。
- [x] high-trust mode では unchecked imported certificate を使えない。
- [x] import store は network、filesystem package discovery、remote import を実行しない。
- [x] imported axiom dependencies は hidden/private export filtering で消えない。

検証:

```sh
cargo test -p npa-checker-ref import
cargo test -p npa-cert import
cargo test -p npa-api independent_checker
```

依存:

```text
P8H-03
```

注意:

```text
import store は runner が明示的に渡した certificate set だけを見る。自動 import insertion はしない。
```

### P8H-05: minimal type checker と declaration check を実装する

実装タスク:

- [x] Sort / Pi / Lam / App / Let / Const の inference / checking を実装する。
- [x] de Bruijn index / binder scope / context lookup を仕様と対応させる。
- [x] AxiomDecl / DefDecl / TheoremDecl の type : Sort と value/proof : type を検査する。
- [x] declaration order と dependency order を再検査する。
- [x] error は type mismatch、unknown reference、expected function、expected sort などに構造化する。

受け入れ条件:

- [x] well-typed theorem / def は通る。
- [x] ill-typed application / wrong theorem proof は拒否される。
- [x] theorem proof は opaque export として登録され、untrusted theorem unfolding はしない。
- [x] checker は source pretty text、Human name shortening、notation を使わない。

検証:

```sh
cargo test -p npa-checker-ref type_check
cargo test -p npa-kernel checks_
cargo test -p npa-cert
```

依存:

```text
P8H-04
```

注意:

```text
P8H-05 は conversion を最小限の structural equality にしてよい。βδζι の本実装は後続 milestone で固定する。
```

### P8H-06: conversion checker を実装する

実装タスク:

- [x] WHNF を reference checker 内で実装する。
- [x] β reduction を実装する。
- [x] δ reduction を reducibility metadata に従って実装する。
- [x] ζ reduction を実装する。
- [x] Pi / Lam / App / Sort / Const / BVar の definitional equality を実装する。
- [x] fuel / recursion bound を deterministic error として扱う。

受け入れ条件:

- [x] β / δ / ζ の正例と負例が test で固定されている。
- [x] η conversion、proof irrelevance conversion、quotient computation、untrusted theorem unfolding を入れない。
- [x] fast kernel と同じ certificate を受理するが、conversion implementation は共有しない。
- [x] conversion cache は使わないか、使う場合も semantics に影響しない deterministic optimization に限定する。

検証:

```sh
cargo test -p npa-checker-ref conversion
cargo test -p npa-kernel reduces_
cargo test -p npa-kernel rejects_
```

依存:

```text
P8H-05
```

注意:

```text
performance より仕様の読みやすさを優先する。高速化は P8H-14 の benchmark 後に判断する。
```

### P8H-07: simple inductive / recursor checker を実装する

実装タスク:

- [x] inductive parameter / index / result sort / constructor type を検査する。
- [x] constructor result が対象 inductive family を返すことを検査する。
- [x] MVP positivity checker を Nat / Eq / List に十分な保守的仕様で実装する。
- [x] generated recursor type と iota rule が declaration と一致することを検査する。
- [x] WHNF の ι reduction を recursor application に接続する。

受け入れ条件:

- [x] Nat / Eq / List の valid inductive certificate が通る。
- [x] negative occurrence、nested inductive、mutual inductive は MVP で deterministic reject になる。
- [x] recursor result mismatch / constructor result mismatch が structured error になる。
- [x] ι reduction の正例と負例が fast kernel と differential test で比較される。

検証:

```sh
cargo test -p npa-checker-ref inductive
cargo test -p npa-checker-ref iota
cargo test -p npa-kernel inductive
```

依存:

```text
P8H-06
```

注意:

```text
Phase 9 の advanced inductive は対象外。MVP は Nat / Eq / List / simple generated artifacts に限定する。
```

### P8H-08: axiom report recomputation / axiom policy を実装する

実装タスク:

- [x] declaration ごとの direct / transitive axiom dependencies を再計算する。
- [x] module axiom report と certificate 内 report を比較する。
- [x] axiom report hash を再計算し、stale report を拒否する。
- [x] policy file の `deny_sorry` / `deny_custom_axioms` / allowed axiom set を checker boundary に接続する。
- [x] exact `Std.Logic.Eq.rec` standard exception を custom axiom とは別に扱う。

受け入れ条件:

- [x] axiom report から実際の dependency を削った certificate は reject される。
- [x] custom axiom / synthetic sorry は high-trust policy で reject される。
- [x] `Std.Logic.Eq.rec` 以外の classical axiom は MVP standard library policy で reject される。
- [x] axiom policy failure は checker result / auxiliary result のどちらでも deterministic に分類できる。

検証:

```sh
cargo test -p npa-checker-ref axiom
cargo test -p npa-cert axiom
cargo test -p npa-api independent_checker
```

依存:

```text
P8H-07
```

注意:

```text
axiom policy は core typing rule を変えない。proof validity と release/high-trust pass condition を分けて扱う。
```

### P8H-09: standard library certificates を reference checker で再検査する

実装タスク:

- [x] `Std.Logic` / `Std.Nat` / `Std.List` / `Std.Algebra.Basic` の MVP certificate fixture を reference checker に通す。
- [x] import closure と `export_hash` / high-trust `certificate_hash` を reference checker で検査する。
- [x] standard library の theorem index / rewrite profile / simp profile を checker acceptance の根拠にしない regression を追加する。
- [x] Phase 6 release artifacts と reference checker result の hash / axiom report を照合する。
- [x] source package skeleton や Human debug view が checker input に入らないことを test で固定する。

受け入れ条件:

- [x] four MVP release modules は source なしで reference checker OK になる。
- [x] standard library module が custom axiom を含むと reject される。
- [x] theorem index が壊れていても certificate check result は certificate bytes だけから決まる。
- [x] Machine API / Phase 7 retrieval candidate hash は P8H-09 の追加で変わらない。

検証:

```sh
cargo test -p npa-checker-ref std
cargo test -p npa-api std_library
cargo test -p npa-api ai_search
```

依存:

```text
P8H-08
```

注意:

```text
P8H-09 は standard library を拡張しない。既存の Phase 6 artifacts を independent checker の対象にする。
```

### P8H-10: external checker binary / runner を実装する

実装タスク:

- [x] `npa-checker-ref` binary または同等の standalone checker binary を追加する。
- [x] `npa-checker-ext` runner contract を target CLI として実装または wrapper で固定する。
- [x] runner は policy allowlist と runner-owned binary registry だけから checker binary を選ぶ。
- [x] dynamic args は certificate / import dir or import lock / policy / output json に限定する。
- [x] runner sandbox policy として no network、read-only cert dir、no source mount、no plugin を固定する。

受け入れ条件:

- [x] AI / request が arbitrary binary path、extra flags、env vars、cwd override を指定できない。
- [x] raw checker output は MachineCheckResult に保存され、AI sidecar より前に materialize される。
- [x] process launched / exit status / checker id / binary hash が deterministic に記録される。
- [x] malformed raw output は checker success ではなく structured failure になる。

検証:

```sh
cargo test -p npa-checker-ref --bin npa-checker-ref
cargo test -p npa-api independent_checker
cargo clippy --workspace --all-targets -- -D warnings
```

依存:

```text
P8H-09
```

注意:

```text
external checker は運用分離の milestone。reference checker の正しさを AI sidecar や runner policy に委譲しない。
```

### P8H-11: checker result normalization / disagreement CI を接続する

実装タスク:

- [x] fast kernel / reference / external checker raw result を `MachineCheckResult` に変換する。
- [x] `NormalizedCheckResult` を required checker profile order で deterministic に生成する。
- [x] checked / failed / resource exhausted / missing checker result の comparison status を固定する。
- [x] checker disagreement を CI failure として扱う gate を追加する。
- [x] PR mode は required profile `reference`、nightly は `reference, external`、release は `fast-kernel, reference, external` にする。

受け入れ条件:

- [x] checker result の hash / policy hash / request hash が mismatch すると normalization は pass しない。
- [x] optional checker の missing は PR pass condition を壊さず、required checker missing は failure になる。
- [x] AI sidecar は comparison status / result hash を上書きできない。
- [x] disagreement report は module / declaration / checker profile / certificate hash を含む。

検証:

```sh
cargo test -p npa-api independent_checker
cargo test -p npa-api independent_checker_normalized
cargo test -p npa-api phase9
```

依存:

```text
P8H-10
```

注意:

```text
P8H-11 は existing Phase 8 AI substrate と real checker outputs を接続する milestone。
```

### P8H-12: challenge mode / audit bundle を接続する

実装タスク:

- [x] challenge file の statement_core_hash / allowed axioms / import hashes を schema として固定する。
- [x] proof certificate の theorem statement hash が challenge と一致することを検査する。
- [x] audit bundle に proof `.npcert`、imports、policy、checker outputs、hashes、axiom report を materialize する。
- [x] ReleaseAuditBundleManifest に real MachineCheckResult / NormalizedCheckResult / AuxiliaryResult を含める。
- [x] bundle validation は source / tactic / AI trace なしで再実行できるようにする。

受け入れ条件:

- [x] challenge statement mismatch は deterministic failure になる。
- [x] missing import、wrong certificate hash、forbidden axiom は audit bundle validation で failure になる。
- [x] AI sidecar は bundle metadata として含められても pass condition を変えない。
- [x] high-trust audit bundle は source ignored / no network / no plugin の前提で検査できる。

検証:

```sh
cargo test -p npa-api independent_checker_release
cargo test -p npa-api independent_checker_challenge
cargo test -p npa-checker-ref audit
```

依存:

```text
P8H-11
```

注意:

```text
challenge owner が固定した statement hash と allowed axiom policy だけを使う。AI が expected verdict を選ばない。
```

### P8H-13: fuzzing / mutation / differential testing を固定する

実装タスク:

- [x] malformed certificate fuzz cases を reference checker に通し、panic せず reject する。
- [x] proof mutation fixture を作り、fast kernel / reference / external checker が reject することを比較する。
- [x] axiom report mutation、import hash mutation、noncanonical table mutation を challenge corpus に追加する。
- [x] differential test で fast kernel OK / reference NG、reference OK / external NG を failure として保存する。
- [x] challenge replay result を checker result oracle に接続する。

受け入れ条件:

- [x] mutation target が invalid な場合は generator failure と checker reject を混同しない。
- [x] outcome-hint は test helper であり、checker result の代わりにならない。
- [x] accepted mutation は unexpected checker acceptance として CI failure になる。
- [x] fuzz / mutation tests は deterministic seed と artifact hash を記録する。

検証:

```sh
cargo test -p npa-checker-ref fuzz
cargo test -p npa-api independent_checker_challenge
cargo test -p npa-cert mutation
```

依存:

```text
P8H-12
```

注意:

```text
distributed fuzzing や external SMT certificate checker は Phase 8 MVP には入れない。
```

### P8H-14: CI modes / performance gates を固定する

実装タスク:

- [x] PR / nightly / release / high-trust の CI command set を repository scripts または documented workflow に固定する。
- [x] PR mode は changed certs + reverse dependencies + reference checker required にする。
- [x] external checker / full recursive import check / full audit bundle は nightly / release / high-trust required にする。
- [x] performance benchmark を fast kernel、Machine API、theorem index build、AI benchmark、reference/external checker に分ける。
- [x] reference / external checker benchmark を PR の同期必須 job に入れず、別 job または cached audit result として扱う。

受け入れ条件:

- [x] PR の AI candidate hot path latency は Phase 8 audit 追加で増えない。
- [x] nightly / release は reference / external checker の詳細 benchmark を記録する。
- [x] release mode は full independent check と audit bundle generation が通らないと pass しない。
- [x] high-trust mode は all imports recursively checked と at least two independent checkers required を満たす。

検証:

```sh
cargo test -p npa-api independent_checker
cargo test -p npa-api ai_search
./scripts/phase9-regression.sh
```

依存:

```text
P8H-13
```

注意:

```text
benchmark policy は proof acceptance boundary ではない。性能結果は regression gate / release policy として扱う。
```

### P8H-15: docs / release completion gate を固定する

実装タスク:

- [x] README、`doc/phase8-human.md`、`doc/phase8-ai.md` の実装済み境界を更新する。
- [x] standalone checker binary、external checker runner、CI audit の command examples を実コマンドに合わせる。
- [x] Phase 8 completion criteria を test / script / CI workflow 名に紐づける。
- [x] Phase 9 regression と Phase 8 release audit CI の役割差分を文書化する。
- [x] release / high-trust audit artifact の保存場所と generated artifact policy を明記する。

受け入れ条件:

- [x] `.npcert` を source なしで reference / external checker が検査できることが docs と tests で一致している。
- [x] fast kernel / reference / external checker comparison failure が release blocker として文書化されている。
- [x] current repository status と target design の違いが stale になっていない。
- [x] Phase 8 docs は AI sidecar を trust boundary に入れない。

検証:

```sh
git diff --check
rg -n "standalone CLI binary はまだ存在しない|external checker on changed certs|AI sidecar.*pass condition" README.md doc/phase8-human.md doc/phase8-ai.md
cargo test --workspace
./scripts/phase9-regression.sh
```

依存:

```text
P8H-14
```

注意:

```text
P8H-15 は final documentation / gate alignment。未実装機能を README で実装済みと書かない。
```

---

## 4. 完了条件

Phase 8 Human が完了したと言える条件はこれです。

```text
- .npcert を source なしで reference checker が検査できる。
- reference checker が fast kernel / npa-cert verifier と独立した type / conversion / inductive checker を持つ。
- external checker が別プロセス / 別バイナリで動く。
- import の export_hash / high-trust certificate_hash を検査できる。
- declaration hash / export_hash / certificate_hash / axiom_report_hash を再計算できる。
- axiom report を再計算し、custom axiom / sorry を拒否できる。
- exact Std.Logic.Eq.rec standard exception だけを custom axiom と区別して扱える。
- Std.Logic / Std.Nat / Std.List / Std.Algebra.Basic の certificate を source なしで再検査できる。
- fast kernel / reference / external checker が CI で比較される。
- checker disagreement 時に CI / release audit が fail する。
- audit bundle を生成・検査できる。
- release 時に full independent check が走る。
- Phase 8 audit が AI candidate hot path の通常 latency を増やさない。
```

---

## 5. MVP では入れないもの

MVP では次を入れません。

```text
- 形式検証済み checker
- mutual / nested inductive の full support
- quotient computation
- proof irrelevance conversion
- η conversion
- external SMT certificate checker
- distributed certificate verification
- cryptographic signature infrastructure
- AI majority vote over checker disagreement
- AI-selected trusted checker binary
- source re-elaboration as independent verification
```
