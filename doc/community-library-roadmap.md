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

# 4. 先に作るべきもの

## 4.1 `npa-package.toml`

外部 theorem library は、root に `npa-package.toml` を置きます。
これは source layout、certificate artifact、import lock、axiom policy、生成物を記述する package manifest です。

草案:

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
module = "Math.Basic"
source = "Math/Basic/source.npa"
certificate = "Math/Basic/certificate.npcert"
meta = "Math/Basic/meta.json"
replay = "Math/Basic/replay.json"
producer_profile = "human-surface-explicit-term"

imports = []
expected_export_hash = "sha256:..."
expected_certificate_hash = "sha256:..."
expected_axiom_report_hash = "sha256:..."

definitions = []
theorems = ["id", "compose"]
axioms = []

[[imports]]
module = "Std.Logic.Eq"
package = "npa-std"
version = "0.1.0"
export_hash = "sha256:..."
certificate_hash = "sha256:..."
certificate = "vendor/npa-std/Std/Logic/Eq/certificate.npcert"
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

禁止すること:

```text
- import を module name だけで解決する
- package manager が registry から暗黙に最新 certificate を補完する
- source file だけを見て verified と扱う
- expected hash 不一致を warning に落とす
```

## 4.2 package CLI

この `npa` リポジトリ側に、外部 repo から使える CLI を用意します。
名前は仮に `npa` または `npa-package` とします。

必要な command:

```sh
npa package check
npa package build-certs
npa package verify-certs
npa package check-hashes
npa package axiom-report
npa package index
npa package publish-plan
```

各 command の責務:

```text
npa package check
  manifest schema、module graph、import lock、path、policy を検査する

npa package build-certs
  source.npa から certificate.npcert を再生成する
  replay.json は使ってよいが、trusted input にはしない

npa package verify-certs
  certificate.npcert を source なしで検査する
  fast verifier と reference checker の両方を実行できるようにする

npa package check-hashes
  expected_export_hash / expected_certificate_hash / expected_axiom_report_hash と
  実際の artifact を比較する

npa package axiom-report
  package 全体と module ごとの axiom report を生成する

npa package index
  theorem search / documentation / registry 用の theorem index を生成する

npa package publish-plan
  registry に送る metadata と artifact list を出す
```

CLI は package repo の source を読んでよいですが、checker verdict の根拠は `.npcert` に限定します。
kernel crate に filesystem、network、registry lookup を入れてはいけません。

## 4.3 CI contract

PR mode では、contributor が追加した module と影響範囲を中心に検査します。

```sh
npa package check
npa package build-certs
npa package check-hashes
npa package verify-certs --checker reference --changed
npa package axiom-report --check
npa package index --check
```

release / high-trust mode では、全 package を source-free に再検査します。

```sh
npa package check
npa package build-certs --all
npa package check-hashes --all
npa package verify-certs --checker fast --all
npa package verify-certs --checker reference --all
npa package verify-certs --checker external --all
npa package axiom-report --check --all
npa package index --check --all
npa package publish-plan --check
```

PR mode で external checker を required にする必要はありません。
ただし release / high-trust mode では required にします。

## 4.4 artifact layout

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
    theorem-index.json
    axiom-report.json
    package-lock.json
```

`replay.json` は任意です。
AI proof search や tactic replay の再現性には有用ですが、checker は読まない前提にします。

## 4.5 review policy

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

## 4.6 registry

最初は registry service を作らず、Git tag と release artifact だけでもよいです。
ただし registry に将来移行できる metadata は初期から固定します。

registry entry の最小単位:

```json
{
  "schema": "npa.registry.module.v0.1",
  "package": "npa-mathlib",
  "package_version": "0.1.0",
  "module": "Math.Algebra.Group.Basic",
  "core_spec": "npa.core.v0.1",
  "kernel_profile": "npa.kernel.v0.1",
  "certificate_format": "npa.certificate.canonical.v0.1",
  "export_hash": "sha256:...",
  "certificate_hash": "sha256:...",
  "axiom_report_hash": "sha256:...",
  "imports": [
    {
      "module": "Std.Logic.Eq",
      "export_hash": "sha256:...",
      "certificate_hash": "sha256:..."
    }
  ],
  "checker_results": [
    {
      "checker": "npa-checker-ref",
      "profile": "npa.checker.reference.v0.1",
      "status": "accepted"
    }
  ]
}
```

registry は便利な配布・検索の層であり、trusted base ではありません。
registry metadata は、local checker が certificate を再検査するための入力補助として扱います。

---

# 5. 別 repo 化の完了条件

`npa-mathlib` を安全に外へ出す条件は次です。

```text
- 外部 repo root の npa-package.toml だけで package graph を読める
- source から certificate を再生成できる
- checked-in certificate と再生成 certificate の hash が一致する
- source-free reference checker が全 certificate を検査できる
- import closure が module / export_hash / certificate_hash で固定されている
- axiom report が deterministic で、policy 違反を CI failure にできる
- theorem index / documentation index が deterministic に生成できる
- fresh checkout の CI で registry や local machine state に依存せず通る
- `npa` 本体の kernel / certificate / checker 変更なしに theorem-only PR を受け入れられる
```

これを満たすまでは、`proofs/` をこの repo 内の seed corpus として運用する方が安全です。

---

# 6. 実装マイルストーン

## M0: 現 proof corpus の package 化

目的:

```text
proofs/manifest.toml を npa-package.toml 草案に近づける。
```

作業:

```text
- 現行 manifest schema と npa.package.v0.1 の差分を整理する
- module entry に expected_* hash 名を揃える
- import lock 情報を明示する
- axiom policy を package 単位で表現する
```

完了条件:

```text
tools/proof-corpus が出す manifest から、外部 package manifest に必要な情報が欠けていない。
```

## M1: package manifest validator

目的:

```text
npa-package.toml を parse し、module graph と import lock を検査する。
```

作業:

```text
- schema version validation
- path validation
- duplicate module detection
- import cycle detection
- unknown import detection
- expected hash grammar validation
- axiom policy validation
```

完了条件:

```text
manifest だけの不整合を certificate build 前に構造化 error として返せる。
```

## M2: generic package build / verify CLI

目的:

```text
tools/proof-corpus の repo 固定ロジックを、外部 repo でも使える CLI に分離する。
```

作業:

```text
- package root を引数で受け取る
- module source から certificate を build する
- checked-in certificate を verify する
- expected hash と比較する
- source-free checker を実行する
```

完了条件:

```text
この repo の proofs/ を package として扱い、既存の proof corpus と同じ certificate/hash を再現できる。
```

## M3: CI template

目的:

```text
外部 theorem library がそのまま使える CI workflow を用意する。
```

作業:

```text
- PR mode の changed-module check
- release mode の full package check
- checker result / axiom report / theorem index artifact upload
- failure message を contributor 向けに構造化する
```

完了条件:

```text
npa-mathlib-seed repo を fresh checkout して、CI が package check を完走できる。
```

## M4: npa-mathlib-seed

目的:

```text
別 repo 化を小さく dogfood する。
```

作業:

```text
- Basic / Eq / Nat / 小さな Algebra module を移す
- npa-package.toml を置く
- checked-in certificate と axiom report を置く
- CI を通す
- npa 本体側から外部 package を import する fixture を作る
```

完了条件:

```text
npa repo に theorem を追加せず、外部 repo 側だけの PR で theorem library を更新できる。
```

## M5: publish metadata / registry seed

目的:

```text
registry service なしでも、将来 registry に載せる metadata を release artifact として出す。
```

作業:

```text
- module registry entry JSON を生成する
- package release manifest を生成する
- checker result summary を添付する
- artifact signature / checksum policy を決める
```

完了条件:

```text
GitHub release artifact だけで、別 package が hash 固定 import を解決できる。
```

---

# 7. 初期 contributor workflow

最初の外部 library で contributor に見せる流れは、なるべく短くします。

```text
1. source.npa に theorem を追加する
2. npa package build-certs を実行する
3. npa package check を実行する
4. certificate.npcert / meta.json / manifest hash 更新を commit する
5. PR を出す
6. CI が source-free checker と axiom report を検査する
7. reviewer は statement / naming / dependency / documentation を見る
```

AI assistant や tactic は contributor の作業を助けてよいですが、PR の pass/fail は certificate と checker で決めます。

---

# 8. 非目標

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

# 9. 直近の実装順

今この `npa` repo で始めるなら、順番は次です。

```text
1. `proofs/manifest.toml` を npa-package.toml 草案に対応づける設計差分を書く。
2. `npa.package.v0.1` の Rust data model と validator を追加する。
3. `tools/proof-corpus` の hard-coded module list と package CLI の責務を分ける。
4. 現在の `proofs/` を package CLI で build / verify できるようにする。
5. source-free checker を package CLI の required gate にする。
6. theorem index / axiom report / publish-plan を deterministic artifact にする。
7. `npa-mathlib-seed` を別 repo として作り、小さな module で CI を通す。
```

この順序なら、別 repo を作る前に必要な信頼境界と contributor experience をこの repo 内で検証できます。
