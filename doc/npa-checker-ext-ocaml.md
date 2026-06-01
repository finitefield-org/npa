# OCaml clean-room npa-checker-ext specification

この文書は、Phase 8 / CLR-08 の target integration として追加する
外部 checker `npa-checker-ext` を **OCaml clean-room 実装**にするための仕様です。

現状、この文書は実装済み宣言ではありません。`crates/npa-checker-ref` はすでに
source-free reference checker として存在しますが、`npa-checker-ext` は別実装・別プロセス・
別ビルド環境で動く外部 checker として新規に作ります。

---

## 1. 決定

`npa-checker-ext` の初期実装は OCaml で行います。
OCaml project はこの `npa` repository 内の `checkers/npa-checker-ext/` に置き、
外部 repository には分離しません。
ただし Rust workspace crate として扱わず、OCaml project 側から `crates/*` にリンクしない
clean-room 境界を維持します。
SHA-256 は pinned external library ではなく、この repository 内の vendored implementation を使います。
First release では quotient feature profile を実装せず、quotient certificate は
`unsupported_core_feature` で拒否します。
First release では checker identity manifest の暗号署名を必須にしません。
runner policy による manifest hash pinning と checker binary hash pinning を必須にし、
署名、key rotation、revocation は後続 hardening scope とします。

```text
checker id:
  npa-checker-ext

implementation profile:
  ocaml-clean-room

input:
  canonical .npcert bytes
  explicit import certificate store
  axiom/checker policy

output:
  deterministic checker_raw_result JSON
```

OCaml を選ぶ理由は次です。

- pattern matching により canonical AST / declaration / error classification を小さく書ける。
- fast kernel の Rust 実装と実装言語、runtime、dependency graph が分かれる。
- reference checker と比較しやすいが、Rust crate を共有しない実装にできる。
- 将来、より形式化された checker へ移行する前の監査可能な外部実装として扱いやすい。

---

## 2. 信頼境界

`npa-checker-ext` が読んでよいものは次だけです。

```text
- --cert で指定された canonical .npcert
- --import-dir または --imports で runner が明示した import certificate inputs
- --policy で指定された axiom/checker policy
- checker binary 自身に埋め込まれた version / build identity
```

読んではいけないものは次です。

```text
- .npa source
- replay.json
- meta.json
- tactic trace
- AI trace / prompt / sidecar
- theorem index
- package registry network data
- hidden package cache
- plugin output
- unchecked source-derived environment
```

外部 checker は source を再 elaboration してはいけません。受理根拠は canonical certificate bytes と
明示 import certificate bytes に対する deterministic check result だけです。

---

## 3. Clean-room 制約

`npa-checker-ext` は NPA Rust workspace の内部 crate にリンクしてはいけません。

禁止:

```text
- npa-kernel
- npa-cert
- npa-api
- npa-frontend
- npa-tactic
- Rust reference checker code の移植的コピー
- source parser / elaborator / tactic の再利用
```

許可:

```text
- doc/core-spec-v0.1.md
- doc/phase0.md
- doc/phase2.md
- doc/phase8-human.md
- canonical certificate fixtures
- public CLI / JSON schema contract
- golden hash fixtures
- differential test result
```

clean-room の意味は「同じ仕様を別実装で検査する」ことです。Rust 実装の関数構造をなぞるのではなく、
公開仕様、canonical byte format、golden/mutation corpus から実装を作ります。

---

## 4. CLI contract

target command:

```sh
npa-checker-ext \
  --cert path/to/module.npcert \
  --import-dir path/to/import-certs \
  --policy path/to/axiom-policy.toml \
  --output json
```

必須要件:

- `--output json` 以外は拒否する。
- `.npa` extension の入力 path は拒否する。
- `--cert` は単一 certificate file の exact bytes として読む。
- import resolution は runner が明示した import store だけを使う。
- network access、package discovery、registry lookup は行わない。
- `LC_ALL=C.UTF-8`, `LANG=C.UTF-8`, `TZ=UTC` 相当の deterministic environment で動く。
- stdout は raw result JSON だけを出す。
- stderr は人間向け diagnostic のみで、proof evidence として扱わない。

将来 `--imports` / `--imports-hash` / `--policy-hash` を受ける場合も、意味は
Phase 8 runner contract と一致させます。AI 出力や package metadata が checker executable を
選択・上書きしてはいけません。

---

## 5. Raw result JSON

`npa-checker-ext` は `npa.independent-checker.checker_raw_result.v1` を出力します。

checked result:

```json
{
  "schema": "npa.independent-checker.checker_raw_result.v1",
  "checker_id": "npa-checker-ext",
  "checker_version": "0.1.0",
  "checker_build_hash": "sha256:...",
  "status": "checked",
  "module": "Std.Nat.Basic",
  "certificate_hash": "sha256:...",
  "export_hash": "sha256:...",
  "axiom_report_hash": "sha256:..."
}
```

failed result:

```json
{
  "schema": "npa.independent-checker.checker_raw_result.v1",
  "checker_id": "npa-checker-ext",
  "checker_version": "0.1.0",
  "checker_build_hash": "sha256:...",
  "status": "failed",
  "module": "Std.Nat.Basic",
  "certificate_hash": "sha256:...",
  "error": {
    "kind": "type_mismatch",
    "section": "declarations",
    "offset": 123
  }
}
```

JSON は deterministic にします。

- object key order は固定する。
- integer は decimal canonical form にする。
- hash は lowercase `sha256:<64 hex>` にする。
- timestamp、host path、absolute path、locale-dependent message は出力しない。
- human-readable error string は raw result identity に入れない。

error kind は Phase 8 raw result normalizer が扱う安定分類に合わせます。

```text
certificate_decode_error
noncanonical_encoding
declaration_hash_mismatch
dependency_hash_mismatch
export_hash_mismatch
axiom_report_mismatch
certificate_hash_mismatch
import_not_found
import_hash_mismatch
forbidden_axiom
type_mismatch
conversion_failure
universe_inconsistency
positivity_failure
inductive_invalid
unsupported_core_feature
unsupported_schema_version
checker_internal_error
```

---

## 6. Certificate decoding

checker は canonical binary `.npcert` だけを受け付けます。

検査対象:

```text
- header format = NPA-CERT-0.1
- core spec = NPA-Core-0.1
- module name grammar
- import table
- name table
- universe level table
- term table
- declaration table
- export block
- axiom report block
- stored module hashes
```

decoder は次を拒否します。

```text
- empty input
- unknown tag
- invalid UTF-8
- non-canonical varint
- table order violation
- duplicate name / declaration / import binding
- dangling reference
- unused canonical table entry
- non-normalized level / term table entry
- trailing bytes
```

OCaml 実装では decoded AST を文字列ではなく algebraic data type として保持します。
de Bruijn index、level expression、global reference、declaration payload はすべて構造化して扱います。
module decode の受理前に、header / imports / declarations / export block /
axiom report から reachability root を作り、term / level を構造的に走査します。
name / level / term table に root から到達不能な entry が残る場合は
`noncanonical_encoding` として拒否します。level / term DAG の order 検査は
canonical payload と domain-separated SHA-256 hash を用いた deterministic order で行い、
stored module hash trailer の後ろに byte が残る場合は `certificate_decode_error` として拒否します。

---

## 7. Hash verification

`npa-checker-ext` は stored hash を信用せず、certificate bytes から再計算します。
hash input の canonical encoder は checker 内部の source-free decoded AST だけを入力にし、
pretty printer、JSON renderer、filesystem path、source span、debug sidecar は参照しません。
domain label は Rust `npa-cert` と byte-for-byte で一致する固定文字列として実装します。
level / term hash recomputation は canonical table order に従い、child hash は既に解決済みの
table entry からだけ取得します。

必須再計算:

```text
- level hash
- term hash
- declaration interface hash
- declaration certificate hash
- export hash
- axiom report hash
- module certificate hash
```

hash domain separation は Rust 実装の結果と bit-for-bit で一致させます。ただし実装は Rust crate を呼ばず、
OCaml 側で canonical encoder と SHA-256 入力を再構成します。

SHA-256 実装はこの repository 内の vendored implementation を使います。
implementation source、test vector fixture、checker build hash への反映方法は
OCaml project skeleton 作成時に固定します。

```text
vendored implementation:
  small OCaml SHA-256 implementation
  no transitive runtime dependency
  standard SHA-256 test vectors required
  Rust sha2 differential fixtures required
```

vendored SHA-256 source identity と build hash は checker identity manifest に固定します。

---

## 8. Import resolution

import resolution は explicit import store だけを使います。

normal mode:

```text
- requested module name と export_hash が一致する import を探す
- certificate_hash が certificate に存在する場合は一致を要求する
- missing import / export hash mismatch は deterministic error
```

high-trust mode:

```text
- import certificate_hash を必須にする
- import certificate bytes を先に external checker で checked にする
- unchecked source-free import を high-trust import として扱わない
- import closure は topological order で検査する
```

外部 checker は filesystem を探索して import を発見してはいけません。`--import-dir` は runner が構成した
source-free import store としてだけ扱います。

---

## 9. Type checking scope

初期 `npa-checker-ext` は `npa-checker-ref` Phase 8 MVP と同じ semantic scope を目標にします。

必須:

```text
- sort / universe level validation
- Pi / Lam / App / Let
- local de Bruijn scope check
- builtin / imported / local global reference resolution
- axiom declaration check
- reducible definition check
- theorem proof type check
- declaration dependency check
- universe parameter arity check
- unresolved universe metavariable rejection
```

conversion:

```text
- alpha-equivalence through de Bruijn representation
- beta reduction
- delta reduction for reducible definitions only
- zeta reduction for let
- iota reduction for supported recursors
- opaque theorem unfolding forbidden
- deterministic fuel / step bound
```

inductive / recursor:

```text
- constructor result targets declared family
- conservative strict positivity check
- generated constructor / recursor interface validation
- recursor parameter / motive / major / minor / result shape validation
- unsupported inductive skeleton rejected with structured error
```

初期実装で対応しない core feature は `unsupported_core_feature` として拒否します。
First release では quotient feature profile を実装せず、quotient certificate は拒否します。
feature gate を増やす場合は、fast kernel、reference checker、external checker の3者で
golden corpus を追加してから有効化します。

M0-05 では first-release supported core feature set を空集合として実装します。
このため canonical certificate の feature report に `quotient_v1`, `quotient_v2`,
`quotient_v3` のいずれかが現れた時点で、外部 checker は
`checker_raw.error.kind = unsupported_core_feature` を返します。空の feature report を持つ
MVP certificate はこの gate では拒否しません。feature policy の入力は canonical certificate
feature report のみであり、AI sidecar、package metadata、source-derived environment は
feature enablement に使いません。quotient support を導入する場合は、fast kernel /
reference checker / external checker の golden corpus を同時に拡張してから supported set に追加します。

---

## 10. Axiom report and policy

外部 checker は axiom report を certificate から再計算します。

必須:

```text
- declaration ごとの direct axiom set
- declaration ごとの transitive axiom set
- module-level transitive axiom set
- import 由来 axiom dependencies
- export block の axiom dependencies
- axiom_report_hash
```

policy:

```text
- deny_sorry = true を default にする
- custom axiom は allowlist にない限り拒否できる
- Std.Logic.Eq.rec の standard exception は exact name/hash でのみ許可する
- axiom policy parse error は checker_internal_error ではなく policy input error として runner 側で扱う
```

checker は axiom の説明文や source span を信用してはいけません。判定は canonical name と
decl_interface_hash に基づけます。

---

## 11. Resource and determinism rules

`npa-checker-ext` は deterministic resource bound を持ちます。

```text
- max_steps
- max_memory_mb
- timeout_ms
- max_term_depth
- max_table_entries
- max_imports
```

runner が強制した timeout / resource exhaustion は checker raw result ではなく
runner-owned `MachineCheckResult` の `timeout` / `resource_exhausted` として扱います。
`npa-checker-ext` が raw result を自力で出す場合、`resource_exhausted` や `timeout` を
`checker_raw.error.kind` に入れてはいけません。semantic checker 内部の deterministic fuel failure は
発生箇所に応じて `conversion_failure`、`type_mismatch`、または
`checker_internal_error` に分類します。OCaml exception backtrace や host-specific message を
raw result に入れてはいけません。

並列化する場合も result order は certificate order / import topological order に固定します。

---

## 12. Implementation layout

推奨 module 分割:

```text
ext_cli.ml
  argv validation, file input, stdout JSON

ext_bytes.ml
  byte reader, canonical varint, offset tracking

ext_name.ml
  module/declaration name grammar

ext_level.ml
  universe level AST, normalization, hashing

ext_term.ml
  core term AST, de Bruijn utilities, hashing

ext_cert.ml
  certificate decoder, table validation, root reachability

ext_hash.ml
  domain-separated SHA-256 input construction

ext_import.ml
  source-free import store, normal/high-trust resolution

ext_axiom.ml
  axiom report recomputation and policy gates

ext_env.ml
  checked environment and public environment

ext_reduce.ml
  whnf, beta/delta/iota/zeta reduction with fuel

ext_typecheck.ml
  inference, checking, definitional equality

ext_inductive.ml
  positivity and recursor shape checks

ext_result.ml
  deterministic checker_raw_result JSON
```

module 間の依存は一方向にします。

```text
bytes/name/level/term
  -> cert/hash
  -> import/env
  -> reduce/typecheck/inductive/axiom
  -> cli/result
```

`ext_cli` 以外は filesystem に触れない設計にします。

---

## 13. Differential testing

最小 test set:

```text
- valid golden certificates accepted by npa-checker-ref and npa-checker-ext
- malformed binary corpus rejected without crash
- hash mutation corpus rejected with matching stable error class
- ill-typed theorem proof rejected
- bad de Bruijn index rejected
- wrong universe arity rejected
- import export_hash mismatch rejected
- high-trust missing certificate_hash rejected
- forbidden custom axiom rejected
- synthetic sorry rejected
- unsupported core feature rejected
- quotient feature profile rejected as unsupported_core_feature in first release
```

比較対象:

```text
fast-kernel:
  acceptance baseline for generated certificates

npa-checker-ref:
  source-free reference baseline

npa-checker-ext:
  clean-room external verdict
```

release / high-trust では、checked / failed status、module name、export_hash、certificate_hash、
axiom_report_hash が不一致なら release blocker にします。error message の自然言語一致は要求しません。

---

## 14. Milestones

M0: repository and build identity

```text
- OCaml project skeleton
- in-repository OCaml project placement
- vendored OCaml SHA-256 implementation
- quotient feature profile rejected for first release
- checker_id = npa-checker-ext
- manifest hash pinning and checker binary hash pinning required
- checker identity manifest signature not required for first release
- deterministic --version / build hash
- --output json only
```

M1: source-free decoder

```text
- .npcert decode
- canonical table validation
- offset-preserving structured errors
```

M1-01 で decoder の基礎として immutable byte reader を追加します。
reader は construction 時点で入力 bytes を immutable string にコピーし、read 操作は reader を
破壊せず `(value, next_reader)` を返します。すべての decode error は certificate section、
byte offset、reason code を持ちます。canonical unsigned varint は minimal ULEB128 のみを許可し、
unexpected EOF、non-minimal encoding、u64 overflow、host length overflow を拒否します。
この層は filesystem、source parser、JSON rendering を参照しません。

M1-02 では header と name grammar を source-free に decode します。
header は `NPA-CERT-0.1` と `NPA-Core-0.1` を必須とし、module name と name table entry は
`Ext_name.t` の structured component list として保持します。empty name、empty component、
dotted component、invalid UTF-8、duplicate name table entry は reason code 付きの decode error
として拒否します。

M1-03 では `LevelTable` と `TermTable` を source-free に decode します。
level は `Zero` / `Succ` / `Max` / `Imax` / `Param`、term は `Sort` / `BVar` /
`Const` / `App` / `Lam` / `Pi` / `Let` の OCaml algebraic data type として保持し、
source text へ戻さない形で後続 checker に渡します。level child と term child は table の
topological order に従い、自分より前の entry だけを参照できます。`Sort` と `Const` の
universe level reference、`Param` と global reference の name reference は、該当 table 内に
存在しなければ `dangling_reference` として拒否します。unknown tag は section と byte offset
付きの deterministic error にし、`Max Zero u` など normalize 後に変化する level entry、
duplicate term entry、`?` を含む unresolved universe metavariable name は semantic trust の前に
拒否します。

M1-04 では header 以降の remaining top-level sections として、imports、declarations、
export block、axiom report、optional core feature report、module hash trailer を source-free に
decode します。declaration payload は axiom / definition / theorem / inductive /
constrained variants / mutual inductive block を structured OCaml values として保持し、
dependency entry と axiom reference は `GlobalRef`、canonical name、hash bytes の構造を
保ったまま decode します。export entry は name、kind、universe params、type、optional body、
type/body hash、optional reducibility/opacity、interface hash、axiom dependencies を保持します。
duplicate declaration name、export block 内の dangling term reference、export axiom dependency の
dangling local declaration reference は deterministic decode error にします。一方、
axiom report の declaration count mismatch は M1-04 では拒否せず、decoded value に preserved
state として残し、M1-05 以降の axiom-report validation に渡します。

M2: hash verifier

```text
- declaration/export/axiom/certificate hash recomputation
- golden hash parity with npa-checker-ref
```

M3: import store

```text
- normal import resolution
- high-trust import certificate hash policy
- topological import checking harness
```

M4: minimal type checker

```text
- sort/Pi/Lam/App/Let
- local/imported/global references
- theorem and definition check
```

M5: conversion

```text
- beta/delta/iota/zeta
- opaque theorem unfolding boundary
- deterministic fuel
```

M6: inductive / recursor

```text
- conservative positivity
- simple inductive declarations
- generated constructor and recursor checks
```

M7: axiom report / policy

```text
- axiom report recomputation
- deny_sorry
- allowed axioms
- exact Std.Logic.Eq.rec exception
```

M8: runner integration

```text
- CheckerBinaryRegistry identity
- MachineCheckResult adoption
- normalized comparison with fast/reference/external
```

M9: release gate

```text
- npa package verify-certs --checker external
- release/high-trust comparison gate
- benchmark and audit bundle collection
```

---

## 15. Acceptance criteria

`npa-checker-ext` を release/high-trust の external checker として使える条件:

```text
- source, tactic, replay, AI trace を読まないことを tests で固定している
- valid Phase 8 MVP certificate corpus を source なしで accept する
- required mutation corpus を deterministic structured error で reject する
- npa-checker-ref と checked module identity が一致する
- high-trust import closure を external checker result から構成できる
- forbidden axiom / sorry を policy で reject できる
- checker binary hash と identity manifest が runner policy に固定されている
- checker identity manifest signature がなくても first release では pass/fail が定義されている
- missing external checker では verified_high_trust artifact を生成しない
```

この条件を満たすまでは、`npa-checker-ext` は target integration であり、
証明受理の必須根拠としては扱いません。

---

## 16. Directory decision and open decisions

M0-01 で OCaml project directory を次のように固定します。

```text
OCaml project directory:
  checkers/npa-checker-ext/

Rust workspace membership:
  not a Cargo workspace member
  do not add this path to Cargo.toml workspace.members
  do not link from the OCaml project to crates/*
```

M0-02 以降の project skeleton は、この directory の下で次の subdirectory を使います。

```text
checkers/npa-checker-ext/src/
checkers/npa-checker-ext/test/fixtures/
checkers/npa-checker-ext/test/golden/
checkers/npa-checker-ext/scripts/
```

M0-02 で skeleton build / test command を次のように固定します。

```sh
checkers/npa-checker-ext/scripts/build.sh
checkers/npa-checker-ext/_build/npa-checker-ext --version
checkers/npa-checker-ext/scripts/test.sh
```

M0-03 で vendored SHA-256 layout と test command を次のように固定します。

```text
implementation:
  checkers/npa-checker-ext/src/ext_sha256.ml

adapter:
  checkers/npa-checker-ext/src/ext_hash.ml

fixtures:
  checkers/npa-checker-ext/test/golden/sha256_vectors.tsv

test:
  checkers/npa-checker-ext/scripts/test.sh sha256
```

`Ext_sha256.source_identity` は checker build hash material に含めます。
source file 全体の hash pinning は runner policy の checker binary hash / manifest hash pinning で扱います。

M0-04 で first-release CLI boundary と build identity material を次のように固定します。

```text
accepted CLI:
  --cert path
  --import-dir path
  --policy path
  --output json
  --version

--version:
  must be used alone
  prints checker_id, checker_version, checker_build_hash, certificate_format,
  core_spec, implementation_profile, project_directory,
  vendored_sha256_source_identity, and
  checker_identity_manifest_signature_required

checker_build_hash material:
  checker_id
  checker_version
  certificate_format
  core_spec
  implementation_profile
  project_directory
  CLI contract version
  feature policy contract version
  vendored SHA-256 source identity
```

First release では checker identity manifest signature は required identity material に含めず、
`checker_identity_manifest_signature_required false` として version output に固定します。

M0-05 で first-release feature policy を次のように固定します。

```text
supported_core_features:
  []

rejected quotient feature profiles:
  quotient_v1
  quotient_v2
  quotient_v3

error kind:
  unsupported_core_feature

policy input:
  canonical certificate feature report only

build identity material:
  feature_policy_contract = m0-05:first-release-empty-core-feature-set
```

この配置は clean-room 境界を狭く保つためのものです。OCaml project は同一 repository 内の
公開仕様、canonical certificate fixture、JSON schema contract、差分 test result を入力としてよい一方、
Rust workspace crate を build dependency として参照してはいけません。

未決定:

```text
- verified checker へ進むときに Lean / Rocq / NPA 自身のどれを優先するか
```

これらは `npa-checker-ext` の trust boundary を広げる判断ではありません。
決定時は Phase 8 / CLR-08 docs と runner policy tests を更新します。
