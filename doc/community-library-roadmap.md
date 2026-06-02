# NPA Community Theorem Library Roadmap

この文書は、NPA を公開し、多くの人が定理を追加できる
Lean mathlib のような集合知を作るために、現在の `npa` リポジトリで先に固めるべき仕組みをまとめます。

目標は、便利な theorem contribution workflow を作りつつ、NPA の信頼境界を崩さないことです。

```text
信頼しない:
  source parser / elaborator / tactic / AI / theorem search / API orchestration / package registry

信頼する:
  canonical certificate
  Rust kernel verdict
  source-free independent checker verdict
  deterministic hash / axiom report
```

---

# 1. 目標状態

最終的には、実装本体と定理ライブラリを分けます。

```text
finitefield-org/npa
  kernel / certificate format / checker / frontend / tactic / package CLI

finitefield-org/npa-std
  小さく堅い標準ライブラリ
  Std.Logic / Std.Nat / Std.List / Std.Algebra.Basic など

finitefield-org/npa-mathlib
  community theorem library
  algebra / order / topology / analysis / geometry など

NPA package registry
  published package metadata
  module export_hash / certificate_hash / axiom report
  source-free checker result
```

最初からすべてを分離する必要はありません。
当面は、この `npa` リポジトリにある `proofs/` と `tools/proof-corpus` を seed として使い、
外部リポジトリでも同じ build / verify ができる package contract を先に作ります。

---

# 2. 基本方針

## 2.1 定理ライブラリは source ではなく certificate を公開単位にする

人間が review する主対象は `.npa` source、命名、statement、依存関係、ドキュメントです。
しかし公開 artifact として信用する対象は `.npcert` です。

```text
source.npa
  -> frontend / elaborator / tactic / AI assistant
  -> canonical certificate
  -> kernel verify
  -> independent checker verify
  -> published theorem artifact
```

`source.npa`、`replay.json`、tactic trace、AI trace は便利な補助情報です。
それらが存在しても、証明済みと呼ぶ根拠にはしません。

## 2.2 import は名前だけでなく hash で固定する

外部 library では、同じ module 名が将来別の中身を持つ可能性があります。
そのため import は少なくとも次の組で固定します。

```text
module
export_hash
certificate_hash
```

`export_hash` は公開 interface の同一性を固定します。
`certificate_hash` は high-trust mode で、依存先 certificate 本体も同一であることを固定します。

## 2.3 API endpoint は proof acceptance boundary にしない

`crates/npa-api` には `/machine/tactics/batch` や `/machine/verify` 相当の library API があります。
これは proof state、tactic execution、search、replay、verify handoff の非信頼 orchestration です。

公開 package の採用条件は、API response ではなく次にします。

```text
checked certificate
  + deterministic certificate_hash
  + deterministic export_hash
  + deterministic axiom_report_hash
  + source-free independent checker success
```

## 2.4 大勢が触る repo と trusted base を分ける

`npa-mathlib` の PR は定理追加、命名、抽象化、依存関係の改善が中心になります。
`npa` 本体の PR は kernel、certificate format、checker、frontend、package CLI の変更が中心になります。

この分離により、community theorem contribution が trusted base の変更と混ざるのを避けます。

---

# 3. 現在の出発点

現在のリポジトリには、外部 library 化の seed になる部品があります。

```text
proofs/manifest.toml
  proof module ごとの source / certificate / replay / meta / hash / axiom 一覧

tools/proof-corpus
  現リポジトリ固定の proof corpus generator
  source 生成、certificate encode、verify、manifest 更新を担う

crates/npa-cert
  canonical certificate encode / decode / verify
  export_hash / certificate_hash / axiom_report_hash

crates/npa-checker-ref
  source-free reference checker binary

crates/npa-api
  Machine API substrate
  checker audit automation substrate
  theorem index / search / replay / verify handoff
```

不足しているのは、外部 repo でもそのまま使える package-level contract です。
特に `tools/proof-corpus` は現在の proof corpus layout と Rust source 内の module list に強く結びついています。
これを一般化する必要があります。

---

# 4. 現時点の未完了点

この節は、registry に進む前に残っている実装 gap を明示します。
ここにある項目は、registry server を作る前にこの `npa` リポジトリ側で片付けるか、
少なくとも外部 theorem library が依存できる contract として固定します。

## 4.1 NPA 本体側の未完了 / target integration

すでに certificate / fast verifier / reference checker / Machine API substrate はあります。
CLR-06 完了時点では、package manifest、package CLI、hash-pinned imports、
source-free package verification、axiom report、theorem index、publish metadata も
local package contract として実装済みです。

公開 ecosystem の残り target integration は次です。

```text
- pinned built `npa-checker-ext` release/high-trust binary evidence
```

`crates/npa-checker-ref` の source-free reference checker binary はあります。
OCaml clean-room `npa-checker-ext` source は `checkers/npa-checker-ext/` にあり、
package command `npa package verify-certs --checker external` も runner policy と
checker registry を必須にして実装済みです。`package high-trust` も
`verified_high_trust` artifact generator として実装済みです。ただし
`npa-checker-ext` が release/high-trust
evidence として存在すると扱うのは、build 済み executable が fresh checkout または
documented CI environment で用意され、runner-owned registry / policy が binary identity と
hash を検証し、package external-mode integration が通った場合だけです。copyable
high-trust CI template は `ci-templates/github-actions/npa-package-high-trust.yml` に
ありますが、実際の high-trust release evidence には外部 repo 側の pinned binary と
release audit bundle が必要です。
External checker benchmark / release audit collection の deterministic summary
contract は実装済みです。Benchmark row は checker identity、certificate hash、
module、time、timeout、memory limit、checker result hash を release audit bundle
内の machine result と結びますが、これは regression evidence であり proof validity
を変える proof input ではありません。reference / external checker benchmark は PR hot
path の同期必須 job ではなく、release/high-trust policy が必要な場合だけ release audit
diagnostic として fail できます。

この repository の GitHub Actions workflow は現状では削除済みで、
`scripts/phase8-release-audit.sh` と `scripts/phase9-regression.sh` を必要に応じて
ローカル実行する状態です。外部 theorem library 用の copyable workflow template は
`ci-templates/github-actions/npa-package-pr.yml` と
`ci-templates/github-actions/npa-package-release.yml` にありますが、この repository の
active gate ではありません。
opt-in high-trust release template は
`ci-templates/github-actions/npa-package-high-trust.yml` にあり、base template とは分離されています。

## 4.2 Registry 前の blocker

registry server より先に必要な blocker は次です。

```text
1. package manifest
   `npa.package.v0.1` で module graph、source path、certificate path、imports、
   expected hashes、axiom policy を宣言する標準形式は固定済み。external dogfood repo
   で使うことが次の検証対象。

2. package CLI
   `npa package check`、`build-certs`、`verify-certs`、`check-hashes`、
   `axiom-report`、`index`、`publish-plan` は実装済み。外部 library CI template は
   CLR-07 で `ci-templates/github-actions/` に固定済み。

3. CI contract
   theorem-only PR で何を required にし、release / high-trust で何を required にするかを
   `doc/external-theorem-library-ci.md` と `doc/community-library-roadmap-clr-07-todo.md` に固定済み。

4. external package import resolution
   package 間 import は `module + export_hash + certificate_hash` で lock する。
   CLR-09 で seed release artifact から downstream fixture に取り込む実績を作る。

5. source-free package verification
   package graph 全体を dependency-topological order で source なし検査する CLI contract は
   `npa package verify-certs` で実装済み。external repo CI template への組み込みは CLR-07 で完了。

6. deterministic public artifacts
   theorem index、axiom report、package lock、publish metadata を registry / docs / downstream package 用に
   deterministic artifact として固定する。CLR-06 で publish metadata は
   `generated/publish-plan.json` として固定済み。

7. publish metadata
   module ごとの `export_hash`、`certificate_hash`、`axiom_report_hash`、checker result、
   import closure を registry entry として出す schema / generator は CLR-06 が提供する。
   registry server と dependency solver はまだ作らない。

8. external dogfood repo
   CLR-09 で `fixtures/npa-mathlib-seed` が reference-checker-only seed release
   fixture として固定済み。fresh checkout 相当の package command、release artifact、
   downstream import fixture、workflow template は検証済み。standalone repo activation と
   CLR-08 high-trust evidence は CLR-10 以降の readiness / follow-up 判断に残す。
```

残りの CI / dogfood / registry integration を飛ばして registry server だけを作ると、
registry が「最新 source や最新 package を便利に配る層」になり、NPA の
certificate-first な信頼境界が曖昧になります。

## 4.3 Registry 前に blocker ではないもの

次は重要ですが、registry seed の前提にはしません。

```text
- production LLM / RAG integration
- online theorem graph store
- external SMT solver service
- browser IDE
- package dependency solver
- binary cache service
- proof search marketplace
```

これらは package contract と source-free verification が固まった後で追加します。

---

# 5. 先に作るべきもの

## 5.1 `npa-package.toml`

外部 theorem library は、root に `npa-package.toml` を置きます。
これは source layout、certificate artifact、import lock、axiom policy、生成物を記述する package manifest です。

構造の草案です。実際の manifest では hash fields に、package command が生成した
正確な SHA-256 文字列を入れます。この文書では疑似 hash literal は置きません。

```toml
schema = "npa.package.v0.1"
package = "npa-mathlib-seed"
version = "0.1.0"
license = "Apache-2.0 OR MIT"

core_spec = "npa.core.v0.1"
kernel_profile = "npa.kernel.v0.1"
certificate_format = "npa.certificate.canonical.v0.1"
checker_profile = "npa.checker.reference.v0.1"

[policy]
allow_custom_axioms = false
allowed_axioms = []

[[modules]]
module = "Proofs.Ai.Basic"
source = "Proofs/Ai/Basic/source.npa"
certificate = "Proofs/Ai/Basic/certificate.npcert"
meta = "Proofs/Ai/Basic/meta.json"
replay = "Proofs/Ai/Basic/replay.json"
producer_profile = "human-surface-explicit-term"

imports = []
# expected_source_hash, expected_certificate_file_hash,
# expected_export_hash, expected_certificate_hash, and
# expected_axiom_report_hash are required exact SHA-256 values
# in a real manifest.

definitions = []
theorems = ["id", "compose"]
axioms = []

[[imports]]
module = "Std.Logic.Eq"
package = "npa-std"
version = "0.1.0"
certificate = "vendor/npa-std/Std/Logic/Eq/certificate.npcert"
# export_hash and certificate_hash are required exact SHA-256 values
# in a real manifest.
```

manifest の役割:

```text
- module graph を明示する
- source path と certificate path を対応させる
- import を hash 固定する
- expected hash を CI で比較できるようにする
- axiom policy を package 単位で固定する
- registry publish に必要な metadata を出す
```

Import resolution rule:

```text
- module-level `imports = [...]` strings first resolve to same-package `[[modules]]`
  entries by exact module name.
- otherwise they must resolve to hash-pinned top-level `[[imports]]` entries.
- external import by module name alone is invalid.
- local/external module name collision is invalid.
- registry lookup, network fetch, package-cache fallback, and implicit latest-version
  resolution are forbidden.
```

CLR-00 で固定する schema constants:

| Constant | Schema string | Artifact |
| --- | --- | --- |
| `PACKAGE_MANIFEST_SCHEMA` | `npa.package.v0.1` | `npa-package.toml` |
| `PACKAGE_LOCK_SCHEMA` | `npa.package.lock.v0.1` | `generated/package-lock.json` |
| `PACKAGE_AXIOM_REPORT_SCHEMA` | `npa.package.axiom_report.v0.1` | `generated/axiom-report.json` |
| `PACKAGE_THEOREM_INDEX_SCHEMA` | `npa.package.theorem_index.v0.1` | `generated/theorem-index.json` |
| `PACKAGE_PUBLISH_PLAN_SCHEMA` | `npa.package.publish_plan.v0.1` | `generated/publish-plan.json` |
| `REGISTRY_MODULE_SCHEMA` | `npa.registry.module.v0.1` | module registry entry |

`npa.package.lock.v0.1` は package-level artifact です。
Phase 8 の `npa.independent-checker.import_lock_manifest.v1` は、checker run ごとに
package metadata から導出される source-free checker input であり、同じ schema ではありません。
`generated/package-lock.json`、`generated/axiom-report.json`、`generated/theorem-index.json`、
`generated/publish-plan.json`、registry module entry は review、search、publish、
CI freshness check のための metadata です。これらは checker evidence ではなく、
証明受理の根拠は canonical certificate と kernel / source-free checker verdict だけです。

禁止すること:

```text
- import を module name だけで解決する
- package manager が registry から暗黙に最新 certificate を補完する
- source file だけを見て verified と扱う
- expected hash 不一致を warning に落とす
```

## 5.2 package CLI

この `npa` リポジトリ側に、外部 repo から使える CLI を用意します。
CLR-00 で、contributor-facing command は installed binary `npa`、Cargo package は
`npa-cli` に固定します。repo 内の検証では `cargo run -p npa-cli -- package ...` を使い、
外部 contributor 向け docs では `npa package ...` を使います。

CLR-04 で実装済みの基本 command:

```sh
npa package check
npa package build-certs
npa package verify-certs
npa package check-hashes
```

CLR-05 までに実装済みの generated metadata command:

```sh
npa package axiom-report
npa package index
```

CLR-06 の release metadata command:

```sh
npa package publish-plan
```

各 command の責務:

```text
npa package check
  manifest schema、module graph、path、policy を検査する metadata gate。

npa package build-certs
  source.npa から certificate.npcert を再生成する。
  replay.json / helper data は読んでよいが、trusted input にはしない。
  --check は差分検査のみで checked-in artifact を書き換えない。

npa package verify-certs
  generated/package-lock.json と certificate artifacts から source-free に検査する。
  .npa source、replay、meta、theorem index、AI trace、out-of-package state は
  checker input にしない。
  fast verifier と reference checker の両方を実行できるが、fast result は
  reference checker verdict として扱わない。

npa package check-hashes
  expected_export_hash / expected_certificate_hash / expected_axiom_report_hash と
  実際の artifact を比較する。expected_source_hash と certificate file hash も
  checked-in bytes に対して検査する。

npa package axiom-report
  package 全体と module ごとの `npa.package.axiom_report.v0.1` metadata を生成または --check する。
  package axiom report schema は `npa.independent-checker.axiom_report.v1` や
  Std-only axiom report schema とは別物。

npa package index
  theorem search / documentation / future registry metadata 用の
  `npa.package.theorem_index.v0.1` metadata を生成または --check する。
  package theorem index schema は Std-only theorem index schema とは別物。

npa package publish-plan
  CLR-06。`npa.package.publish_plan.v0.1` の publish metadata と artifact list を出す。
  `npa.registry.module.v0.1` theorem package module registry seed entries、
  downstream import bundle、checksum-only SHA-256 signature policy を含む。
  registry server、registry URL、network fetch、latest-version resolution、upload、signing は行わない。
```

CLI は非信頼 orchestration layer です。CLI output、diagnostics、package lock、generated
axiom report、generated theorem index、generated publish plan は review / CI / search / release 用の deterministic
metadata, not proof evidence です。
証明受理の根拠は canonical `.npcert` と kernel / source-free checker verdict に限定します。
`npa.registry.module.v0.1` は theorem package registry metadata であり、
`npa.independent-checker.checker_binary_registry.v1` とは別物です。registry metadata は
checker input ではありません。
kernel crate に filesystem、network、registry lookup を入れてはいけません。package commands は
explicit local package root だけを対象にし、network access や binary cache lookup を行いません。

## 5.3 CI contract

PR mode では、現時点の base contract として full-package reference check を使います。
changed-module selection は便利ですが、package command の必須 contract にはまだ入れません。
Copyable templates は `ci-templates/github-actions/` にあり、詳細 task breakdown は
`doc/community-library-roadmap-clr-07-todo.md` です。

```sh
npa package check --root . --json
npa package build-certs --root . --check --json
npa package check-hashes --root . --json
npa package verify-certs --root . --checker reference --json
npa package axiom-report --root . --check --json
npa package index --root . --check --json
```

`axiom-report --check` と `index --check` は CLR-05 の full-package freshness gate です。
どちらも source-free artifact generation boundary を保ち、source、replay、meta、theorem graph
score、prompt metadata、AI traces を required input にしません。CLR-06 の
`publish-plan --check` は release artifact の freshness gate に含めます。

```sh
npa package check --root . --json
npa package build-certs --root . --check --json
npa package check-hashes --root . --json
npa package verify-certs --root . --checker fast --json
npa package verify-certs --root . --checker reference --json
npa package axiom-report --root . --check --json
npa package index --root . --check --json
```

CLR-06 release gate extension:

```sh
npa package publish-plan --root . --check --json
```

PR mode で external checker を required にする必要はありません。
base release mode でも external checker は required ではありません。`--checker external`
と `package high-trust` は、runner policy と checker binary registry を明示し、
source-free package artifacts と build 済み `npa-checker-ext` executable だけを読む
high-trust extension です。`verified_high_trust` は external checker と
high-trust-reference を含む required evidence が揃った後だけ生成できます。reference-only
evidence から `verified_high_trust` を生成してはいけません。
`--changed`、`--all`、`--registry`、`--network`、`--latest` は base contract には入りません。
`npa-package-pr.yml` と `npa-package-release.yml` は外部 repository が copy または reference
する template であり、この repository の `.github/workflows` として有効化するものではありません。
`npa-package-high-trust.yml` は pinned external checker binary と release audit evidence を
同じ external repo が用意した場合だけ copy する opt-in template です。
`npa-mathlib-seed` は CLR-09 でこれらの template と
`summarize-npa-diagnostics.py` / `validate-workflows.py` を取り込みます。

## 5.4 artifact layout

外部 theorem library の layout は、source と checked artifact の対応が機械的に分かる形にします。

```text
npa-mathlib/
  npa-package.toml
  Math/
    Basic/
      source.npa
      certificate.npcert
      replay.json
      meta.json
    Algebra/
      Group/
        Basic/
          source.npa
          certificate.npcert
          replay.json
          meta.json
  generated/
    package-lock.json
    theorem-index.json
    axiom-report.json
    publish-plan.json
```

`replay.json` は任意です。
AI proof search や tactic replay の再現性には有用ですが、checker は読まない前提にします。
`generated/axiom-report.json` と `generated/theorem-index.json` も checker input ではありません。
これらの生成・検査は source-free で、manifest、package lock、certificate artifacts、
source-free verifier output、check mode の checked generated JSON だけを必要入力にします。

## 5.5 review policy

community theorem library の review は、証明の正しさを人間が手で検査する場ではありません。
正しさは certificate と checker が検査します。

人間 review で見るべきもの:

```text
- theorem statement が意図した数学を表しているか
- namespace と theorem name が検索しやすいか
- 既存定理と重複していないか
- 依存関係が重すぎないか
- axiom 使用が増えていないか
- 抽象化が後続 library に使いやすいか
- source が保守可能か
- documentation と tags が十分か
```

CI で見るべきもの:

```text
- certificate が再生成可能か
- source-free checker が通るか
- expected hash と一致するか
- axiom report が policy に合うか
- import closure が hash 固定されているか
- theorem index が deterministic か
```

## 5.6 CLR-06 publish-plan handoff

CLR-06 `publish-plan` は CLR-05 が固定した package metadata を消費できます。ただし
publish metadata も registry entry も trusted base ではなく、証明受理の根拠は canonical
certificate と source-free checker verdict のままです。

CLR-06 が使ってよい CLR-05 artifact fields:

```text
generated/axiom-report.json
  schema = npa.package.axiom_report.v0.1
  package, version, manifest, package_lock
  policy.allow_custom_axioms, policy.allowed_axioms
  modules[].module, origin, export_hash, certificate_hash, axiom_report_hash,
  certificate_file_hash, direct_axioms, transitive_axioms, policy_status
  checker_summaries[]
  summary.*
  package_axiom_report_hash

generated/theorem-index.json
  schema = npa.package.theorem_index.v0.1
  package, version, manifest, package_lock, index_profile
  entries[].global_ref, kind, statement.core_hash, statement.head,
  statement.constants, modes, tags, axiom_dependencies,
  module_axiom_report_hash, artifact.certificate
  checker_summaries[]
  summary.*
  theorem_index_hash
```

CLR-06 may copy artifact paths, hashes, policy status, checker summaries, and theorem index
entries into publish metadata. It must not require source, replay, meta, theorem graph score,
prompt metadata, or AI traces to validate CLR-05 artifacts.

CLR-06 output is helper metadata, not proof evidence. `generated/publish-plan.json` uses
`npa.package.publish_plan.v0.1`, embeds `npa.registry.module.v0.1` theorem package module seed
entries, and records a checksum-only SHA-256 MVP signature policy. It must remain distinct from
Phase 8 independent checker binary registry metadata such as
`npa.independent-checker.checker_binary_registry.v1`. Downstream packages may use the
downstream import bundle to copy package, version, module, path, `export_hash`, and
`certificate_hash` pins, but they still rerun source-free local verification from certificate
bytes.

Handoff boundaries:

```text
CLR-07 may add an optional publish-plan check step to release-template variants.
Base CLR-07 templates still avoid registry/network/latest/upload/sign behavior.
CLR-09 consumes the publish plan, release artifact list, registry seed entries,
and downstream import bundle for the seed library release flow.
```

## 5.7 registry

最初は registry service を作らず、Git tag と release artifact だけでもよいです。
ただし registry に将来移行できる metadata は初期から固定します。

registry entry の最小単位:

```text
schema = npa.registry.module.v0.1
package = npa-mathlib
package_version = 0.1.0
module = Math.Algebra.Group.Basic
core_spec = npa.core.v0.1
kernel_profile = npa.kernel.v0.1
certificate_format = npa.certificate.canonical.v0.1
export_hash = exact SHA-256 export hash
certificate_hash = exact SHA-256 certificate hash
axiom_report_hash = exact SHA-256 axiom report hash
imports = direct imports with module, export_hash, certificate_hash
checker_results = source-free checker summaries, for example npa-checker-ref accepted
artifact_hashes = release artifact file hashes
```

registry は便利な配布・検索の層であり、trusted base ではありません。
registry metadata は、local checker が certificate を再検査するための入力補助として扱います。
`npa.registry.module.v0.1` は theorem package module metadata であり、
`npa.independent-checker.checker_binary_registry.v1` は external checker binary selection /
runner policy metadata です。この 2 つは相互に代替できません。registry seed だけで
import を承認せず、downstream package は source-free checker をローカルに再実行します。

---

# 6. 別 repo 化の完了条件

`npa-mathlib` を安全に外へ出す条件は次です。

```text
- 外部 repo root の npa-package.toml だけで package graph を読める
- source から certificate を再生成できる
- checked-in certificate と再生成 certificate の hash が一致する
- source-free reference checker が全 certificate を検査できる
- import closure が module / export_hash / certificate_hash で固定されている
- axiom report が deterministic で、policy 違反を CI failure にできる
- package lock / axiom report / theorem index / publish-plan が deterministic に生成できる
- fresh checkout の CI で registry や local machine state に依存せず通る
- `npa` 本体の kernel / certificate / checker 変更なしに theorem-only PR を受け入れられる
```

これを満たすまでは、`proofs/` をこの repo 内の seed corpus として運用する方が安全です。

---

# 7. 実装マイルストーン

詳細な実装単位は `doc/community-library-roadmap-todo.md` と
`doc/community-library-roadmap-clr-00-todo.md` から
`doc/community-library-roadmap-clr-10-todo.md` に分解済みです。
この章では、元の M0-M5 を現在の CLR sequence に対応づけます。

```text
M0: 現 proof corpus の package 化
  -> CLR-00, CLR-01, CLR-02
     CLI / schema 決定、`npa.package.v0.1` validator、
     `proofs/` を package fixture として表現する。

M1: package manifest validator
  -> CLR-01
     manifest parse、closed schema、path/hash/axiom/import graph validation。

M2: generic package build / verify CLI
  -> CLR-03, CLR-04
     import lock、source-free package graph verification、
     `npa package check/build-certs/verify-certs/check-hashes`。
     CLR-04 の詳細 breakdown は `doc/community-library-roadmap-clr-04-todo.md`。

M3: deterministic public artifacts and CI template
  -> CLR-05, CLR-07
     axiom report / theorem index、external theorem library CI templates。
     Base CI は full-package reference check を使い、changed-module selection は後続。
     CLR-05 の詳細 breakdown は `doc/community-library-roadmap-clr-05-todo.md`。
     CLR-07 の詳細 breakdown は `doc/community-library-roadmap-clr-07-todo.md`。

M4: publish metadata / registry seed
  -> CLR-06
     `generated/publish-plan.json`、`npa.registry.module.v0.1`、
     downstream import bundle、checksum-only MVP policy。

M5: npa-mathlib-seed dogfood
  -> CLR-09
     `Proofs.Ai.Basic`, `Proofs.Ai.Prop`, `Proofs.Ai.Eq`,
     `Proofs.Ai.Nat`, `Proofs.Ai.Reduction` から始める。
     CLR-07 の `npa-package-pr.yml` / `npa-package-release.yml` を copy または reference する。
     大きな algebra / geometry / analysis corpus は package ergonomics 確認後。

Registry readiness
  -> CLR-10
     section 4.2 blocker の pass/fail evidence を揃え、
     Git-release-based registry seed を続ける判断を記録する。
     public `npa-mathlib` Layer 0 は `fixtures/npa-mathlib/` と
     `fixtures/npa-mathlib-downstream/` で local baseline 化済み。
     次の作業は `doc/npa-mathlib-public-release-plan.md` に従って
     `doc/npa-standalone-repo-activation.md` の手順で standalone repo
     activation を進め、その後 larger theorem layer を追加する。
```

CLR-08 は high-trust external checker integration の独立 milestone です。
package external checker command、`verified_high_trust` generator、opt-in high-trust CI
template、external checker benchmark / audit summary contract は実装済みですが、
`npa-mathlib-seed` と registry readiness は、external / high-trust-reference
evidence がまだ無い場合でも reference-checker-only release として進められます。

---

# 8. 初期 contributor workflow

最初の外部 library で contributor に見せる流れは、なるべく短くします。

```text
1. source.npa に theorem を追加する
2. npa package check --root . --json を実行する
3. npa package build-certs --root . --check --json を実行する
4. npa package check-hashes --root . --json を実行する
5. npa package verify-certs --root . --checker reference --json を実行する
6. npa package axiom-report --root . --check --json を実行する
7. npa package index --root . --check --json を実行する
8. CLR-06 後、release 前には npa package publish-plan --root . --check --json を実行する
9. source / certificate / replay / meta / generated artifact の必要な差分を commit する
10. PR を出す
11. CI が同じ package command を fresh checkout で検査する
12. reviewer は statement / naming / dependency / axiom change / documentation を見る
```

AI assistant や tactic は contributor の作業を助けてよいですが、PR の pass/fail は certificate と checker で決めます。

---

# 9. 非目標

このロードマップで今すぐ作らないもの:

```text
- online theorem proving service
- registry server
- package dependency solver
- binary cache service
- proof search marketplace
- browser IDE
- production LLM / RAG integration
- external SMT solver service
```

これらは有用ですが、先に package contract、source-free checker CI、hash 固定 import を完成させます。

---

# 10. 直近の実装順

今この `npa` repo で始めるなら、順番は次です。

```text
1. `proofs/manifest.toml` を npa-package.toml 草案に対応づける設計差分を書く。
2. `npa.package.v0.1` の Rust data model と validator を追加する。
3. `tools/proof-corpus` の hard-coded module list と package CLI の責務を分ける。
4. 現在の `proofs/` を package CLI で build / verify できるようにする。
5. source-free checker を package CLI の required gate にする。
6. theorem index / axiom report / publish-plan を deterministic artifact にする。
7. 外部 theorem library 用 CI template を作る。
8. `npa-mathlib-seed` で reference-checker-only dogfood release artifact と downstream fixture を固定する。
9. registry readiness review で、server 実装ではなく Git release artifact 運用を続ける判断を記録する。
10. `fixtures/npa-mathlib/` と `fixtures/npa-mathlib-downstream/` を public Layer 0 baseline として安定させる。
11. `doc/npa-standalone-repo-activation.md` に従って `npa`, `npa-std`, `npa-mathlib` を activation し、larger theorem layer 追加へ進む。
```

この順序なら、別 repo を作る前に必要な信頼境界と contributor experience をこの repo 内で検証できます。
