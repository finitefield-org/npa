# Nano Proof Auditor (NPA)
### ~ A High-Throughput Verification Environment for AI Provers ~

NPA は **Nano Proof Auditor** の実験的な設計・実装リポジトリです。

目標は Lean や Rocq から学びつつ、AI 時代向けに最初から設計された
**proof certificate first** な依存型証明支援系を作ることです。

```text
AI が証明候補を探し、
人間が形式化の意図を確認し、
Rust 製 kernel と独立 checker が proof certificate だけを検査する。
```

## 基本方針

- kernel は Rust で実装する。
- AI、automation、solver、tactic、elaborator、parser、theorem search、API orchestration は trusted base に入れない。
- 最終的な正しさは source script ではなく canonical proof certificate で保証する。
- kernel は小さく、監査しやすく、決定的に動くものにする。
- certificate は再検査可能で、import の `export_hash` と高信頼モード用の `certificate_hash`、declaration hash、`axiom_report_hash` を含む。

文書内の数学例では、読みやすさのため `0`, `1`, `2` を使うことがあります。
これは自然数の表示用省略で、Phase 3 MVP の実入力では数値リテラルを入れるまで
`Nat.zero` / `Nat.succ ...` か、開いた namespace 内の `zero` / `succ ...` と書ければ十分です。
certificate に残るのは canonical `Const` 参照です。

## アーキテクチャ

```text
User / IDE / Web UI / API
AI Proof Orchestrator
Automation / Solvers
Tactic / Metaprogramming
Elaborator / Surface Language
Core Language
Proof Certificate Format
Trusted Kernel
Independent Checkers / Audit Layer
```

上位層は便利だが信用しません。下位層ほど小さくし、最終的には kernel と独立 checker が
canonical certificate を検査します。

## 実装ロードマップ

設計資料は `doc/` に phase ごとに整理されています。

| Phase | 内容 |
| --- | --- |
| 0 | core calculus、typing、conversion、universe、inductive、certificate の仕様固定 |
| 1 | Rust kernel による core term / simple inductive / reduction の検査 |
| 2 | `.npcert` 形式、canonical core AST、hash、axiom report |
| 3 Human | 人間向け Human Surface: parser、name resolution、notation、simple inductive declaration、elaboration |
| 3 AI | AI 向け Machine Surface: 高速で明示的な Phase 3 fast path |
| 4 | tactic 層: `intro`, `exact`, `apply`, `rw`, `simp-lite`, `induction` |
| 5 | IDE / API: proof state、tactic execution、theorem search、goal display |
| 6 | 小さく堅い標準ライブラリ: `Std.Logic`, `Std.Nat`, `Std.List`, `Std.Algebra.Basic` |
| 7 | AI 証明探索: premise retrieval、tactic generation、search、repair |
| 8 | 独立 checker、external checker、CI audit |
| 9 Human | advanced inductive、universe polymorphism強化、quotient、typeclass、SMT certificates、theorem graph、natural language formalization |
| 9 AI | 高度機能向け Machine Profile: AI 候補、SMT 再構成、theorem graph、自然言語形式化の非信頼仕様 |

公開後に theorem library を別リポジトリとして育てるための package / CI / registry への移行計画は
`doc/community-library-roadmap.md` にまとめています。

## Package CLI

外部 theorem library 用の contributor-facing command は、インストール済み binary `npa`
の `package` サブコマンドです。CLR-04 時点で実装済みの基本 gate は次です。

```sh
npa package check --root .
npa package build-certs --root . --check
npa package verify-certs --root . --checker reference
npa package check-hashes --root .
```

このリポジトリ内での開発・検証では、Cargo package `npa-cli` から同じ command family を
実行します。`npa-cli` が提供する installed binary name は `npa` です。repository
verification examples は次です。

```sh
cargo run -p npa-cli -- package check --root proofs
cargo run -p npa-cli -- package build-certs --root proofs --check
cargo run -p npa-cli -- package verify-certs --root proofs --checker reference
cargo run -p npa-cli -- package check-hashes --root proofs
cargo run -p npa-cli -- package axiom-report --root proofs --check
cargo run -p npa-cli -- package index --root proofs --check
cargo run -p npa-cli -- package publish-plan --root proofs --check
```

CLR-08 以降の external checker gate は、runner policy と checker binary registry を明示した
場合だけ有効です。

```sh
npa package verify-certs --root . --checker external \
  --runner-policy ci/runner.release.json \
  --runner-policy-hash "$NPA_RUNNER_POLICY_HASH" \
  --checker-registry ci/checker-binaries.json \
  --json
```

Contributor-facing examples use the installed `npa` binary:

```sh
npa package axiom-report --root . --check
npa package index --root . --check
npa package publish-plan --root . --check
```

source-reading boundary は command ごとに違います。

```text
package check
  `npa-package.toml` の schema / graph / policy を検査する metadata gate。

package build-certs
  local module の `source.npa` と必要な replay/helper data を読んで certificate を再生成する。
  `--check` は差分検査のみで、checked-in artifact を書き換えない。

package check-hashes
  manifest が pin する source / certificate / generated lock hash と checked-in bytes を比較する。

package verify-certs
  source-free verification path。`generated/package-lock.json` と certificate artifacts を読み、
  `.npa` source、replay、meta、theorem index、AI trace、out-of-package state は checker input にしない。
  `--checker external` は `--runner-policy`、`--runner-policy-hash`、
  `--checker-registry` を必須にし、MachineCheckResult JSON diagnostics を
  `generated/checker-results/.../external/result.json` に保存する。

package axiom-report
  `npa.package.axiom_report.v0.1` の package metadata を生成または `--check` する。
  manifest、package lock、certificate artifacts、source-free verifier output から導出し、
  `.npa` source、replay、meta、theorem graph score、prompt metadata、AI trace は必要入力にしない。

package index
  `npa.package.theorem_index.v0.1` の theorem search / documentation metadata を生成または `--check` する。
  certificate-derived data だけを使い、pretty source statement、replay、meta、prompt metadata、
  theorem graph score、AI trace から定理情報を推測しない。

package publish-plan
  `npa.package.publish_plan.v0.1` の release metadata を生成または `--check` する。
  artifact file hashes、`npa.registry.module.v0.1` module registry seed entries、
  downstream import bundle、checksum-only SHA-256 signature policy を記録する。
  registry server、registry URL、network fetch、latest-version resolution、upload、signing は行わない。
```

CLI output、package lock、diagnostics、`generated/axiom-report.json`、
`generated/theorem-index.json`、`generated/publish-plan.json` は CI / review / search / release 用の deterministic metadata,
not proof evidence です。証明の受理根拠は canonical `.npcert` bytes と、
選択された source-free checker / kernel verifier の deterministic verdict です。
`npa.package.axiom_report.v0.1` は package-level schema であり、
`npa.independent-checker.axiom_report.v1` や Std-only axiom report schema とは別物です。
`npa.package.theorem_index.v0.1` も Std-only theorem index schema とは別物です。
`npa.registry.module.v0.1` は theorem package module metadata であり、
`npa.independent-checker.checker_binary_registry.v1` とは別物です。registry seed entries
は checker input ではなく、downstream package は certificate bytes と hash pins を取得した後も
source-free local verification を再実行します。

`npa package publish-plan` は CLR-06 の release metadata command です。package commands は
explicit local package root だけを対象にし、network access や binary cache lookup を行いません。

外部 theorem library 用の copyable CI template は `ci-templates/github-actions/` にあります。
`npa-package-pr.yml` は PR 用に次を実行します。

```sh
npa package check --root . --json
npa package build-certs --root . --check --json
npa package check-hashes --root . --json
npa package verify-certs --root . --checker reference --json
npa package axiom-report --root . --check --json
npa package index --root . --check --json
```

`npa-package-release.yml` は release/high-trust base 用に同じ artifact checks に加えて、
fast-kernel と reference checker の source-free verification を分けて実行します。
CLR-06 の `publish-plan` check は gated optional step です。外部 checker required mode と
`verified_high_trust` は CLR-08 まで入りません。これらの template はこの repo の
`.github/workflows` ではなく、`npa-mathlib-seed` などの外部 repo が copy または reference
するためのものです。この repo の local gate は引き続き `scripts/phase8-release-audit.sh` と
`scripts/phase9-regression.sh` です。

`npa-kernel`、`npa-cert`、`npa-checker-ref` は package CLI の責務を持ちません。
package command の filesystem access、registry metadata handling、CI orchestration は
非信頼層に閉じ、trusted base は canonical certificate、Rust kernel verdict、
source-free checker verdict のままにします。

## リポジトリ構成

```text
.
├── Cargo.toml
├── README.md
├── AGENTS.md
├── crates/
│   ├── npa-kernel/
│   │   └── src/
│   ├── npa-cert/
│   │   ├── src/
│   │   └── tests/
│   ├── npa-frontend/
│   │   └── src/
│   ├── npa-tactic/
│   │   └── src/
│   ├── npa-checker-ref/
│   │   └── src/
│   └── npa-api/
│       └── src/
├── scripts/
│   ├── phase8-release-audit.sh
│   └── phase9-regression.sh
└── doc/
    ├── core-spec-v0.1.md
    ├── overall-design.md
    ├── phase0.md
    ├── phase1.md
    └── ...
```

現時点では Rust kernel と Phase 2 の certificate verifier は実装済みです。
`crates/npa-cert` は `.npcert` の canonical encode/decode、hash 再計算、import 検査、
axiom report 検査、Rust kernel への再検査ハンドオフを担当します。
Phase 3 は `crates/npa-frontend` で Human Surface と Machine Surface を分けて実装しています。
Phase 3 Human は、`parse_human_*` / `compile_human_source_to_*` から使う人間向け convenience layer です。
`open` / `namespace` / notation / implicit argument / hole / simple inductive などを扱えますが、
parser、resolver、elaborator、metadata は trusted base に入りません。
Phase 3 AI は、`parse_machine_*` / Machine Surface term API から使う explicit fast path です。
AI 候補生成と tactic / search / replay / verify は Human Surface を経由せず、notation table、
open scope、overload transaction、hole を持たない Machine Surface request を検査します。
Phase 4 Human は `crates/npa-api` の Human API wrapper と `crates/npa-tactic` の
proof-state primitive を接続して、`by` proof block の `intro` / `exact` / `apply` /
`rw` / `simp-lite` / `induction` を kernel が検査できる proof term に変換します。
`rw` / `induction` を含む certificate-compatible な Human examples を、Machine Surface fixture hash を
変えない regression として固定しています。この Human parser / bridge は AI 向け Machine API の既定経路には入りません。
AI 向け Phase 4 M1/M2/M3/M4/M5/M6/M7 の tactic proof-state core と `exact` /
`intro` / `apply` / `rw` / `simp-lite` / `induction-nat` は `crates/npa-tactic`
で実装されています。closed proof state から canonical certificate へ渡す handoff API と、
AI 探索向けの deterministic budget hash / tactic cache key / batch 実行 gate も同 crate で実装されています。
Phase 5 AI の substrate は `crates/npa-api` で進めており、lossless JSON request decoder、
import/current projection、Phase 4 adapter boundary、Machine Surface callable interface table builder、
owner-aware MachineExprRenderer v1 / renderer QA substrate、MachineApiDiagnostic canonicalization
に加えて、M2 の Machine API types / ID・HashString wire grammar / endpoint envelope validation
と、M5 `/machine/snapshots/get`、M6 `/machine/tactics/run`、M7 `/machine/tactics/batch`、
M8 `/machine/search/for_goal`、M9 `/machine/replay`、M10 `/machine/verify`、
M11 `/machine/prompt_payload` の library API を含みます。
Phase 5 Human の IDE/API profile も `crates/npa-api` に実装済みで、Human session、
structured proof state、transactional `/tactic/run`、theorem search、goal display、
verify / certificate handoff、document incremental cache、LSP-facing payload、
optional assistant payload を提供します。Phase 5 Human の統合 fixture は
session create、state lookup、tactic run、search、display、verify を通し、同時に
Human path が Phase 7 Machine API の candidate hash / state fingerprint を変えないことを
regression として固定しています。
Phase 6 Human / AI の標準ライブラリ handoff も `crates/npa-api` で実装済みです。
Human 側は `Std.Logic` / `Std.Nat` / `Std.List` / `Std.Algebra.Basic` の source package layout と
certificate build boundary を固定し、AI 側は同じ raw `.npcert` から release manifest、
import bundles、theorem index、rewrite / simp profiles、axiom report を再生成します。
`std.nat.mvp` / `std.list.mvp` / `std.all.mvp` は Phase 5 `/machine/sessions` 相当の request に展開して
再検証され、Phase 7 retrieval 候補は必ず Phase 5 batch / replay / verify に戻してから採用する
regression として固定されています。生成される `.npcert` と `Std.machine-*.json` は release/build artifact であり、
このリポジトリでは source layout fixtures、Rust builders、tests を正本として temp package 上で再生成します。
同じ `crates/npa-api` に Phase 7 search controller、Phase 8 checker audit automation、
Phase 9 advanced automation endpoint substrate も実装されています。
Phase 9 Human の target scope は、advanced inductive、universe polymorphism 強化、
typeclass、quotient_v1、SMT certificate surface / reconstruction boundary、theorem graph、
natural language formalization confirmation flow まで実装済みです。
Phase 9 Human / AI 境界は `p9h00_advanced_ai_sidecars_scores_and_smt_outputs_stay_untrusted` と
`p9h00_ai_fast_path_request_shapes_exclude_phase9_human_heavy_checks` で固定し、
高度機能の sidecar / score / solver output / confidence と重い audit は AI 候補 hot path や
checker verdict の根拠に入れません。
Phase 9 AI は deterministic validation / replay substrate と M9 fixture matrix を実装済みですが、
production LLM / RAG、online theorem graph store、external SMT solver service、非空 solver-native
SMT success profile は target integration であり、このリポジトリでは実装済みとは扱いません。
Phase 8 では `crates/npa-checker-ref` の `npa-checker-ref` binary が `.npcert` を
source なしで検査し、`crates/npa-api` が checker request / result の正規化、
release audit bundle、challenge replay、AI sidecar validation の非信頼 orchestration を固定します。
standalone `npa-checker-ext` binary と full external-checker release audit workflow は target integration で、
現リポジトリでは external checker profile と disagreement gate を deterministic tests で固定しています。
これらの `npa-api` automation / library API は候補生成、検査要求の構成、
監査 artifact の正規化、回帰 fixture の実行を担う非信頼層です。
trusted base は広げません。証明の受理根拠は引き続き canonical certificate と、
Rust kernel / 独立 checker が返す deterministic result だけです。

## 開発メモ

少なくとも次を通す方針です。

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Phase 9 Human 完了後の required release completion gate は次です。

```sh
./scripts/phase9-regression.sh
```

このゲートは Phase 9 AI M9 deterministic fixture matrix を先に実行し、その後 `fmt --check`、
`clippy -D warnings`、workspace 全体の test を通します。release / high-trust の pass/fail は
checker result と deterministic artifact で決まり、AI sidecar、theorem graph score、
formalization confidence、SMT solver output は trusted boundary に入りません。
GitHub Actions workflow は削除済みです。このゲートは必要に応じてローカルで実行します。

Phase 8 の release audit fixture gate は次です。

```sh
./scripts/phase8-release-audit.sh
```

このゲートは `cargo test -p npa-checker-ref`、`cargo test -p npa-api independent_checker`、
標準ライブラリ release audit fixture、`cargo test -p npa-api ai_search` を実行します。
GitHub Actions workflow は削除済みです。このゲートは必要に応じてローカルで実行します。
Phase 8 gate は source-free checker / release audit / AI fast path 境界を確認する狭い gate で、
Phase 9 Regression は workspace 全体の後続機能まで含む広い回帰 gate です。

## 参考資料

- [NPA Core Specification v0.1](doc/core-spec-v0.1.md)
- [全体設計](doc/overall-design.md)
- [Phase 0: Core Spec](doc/phase0.md)
- [Phase 1: Kernel](doc/phase1.md)
- [Phase 2: Certificate](doc/phase2.md)
- [Phase 3: Human Surface Language](doc/phase3-human.md)
- [Phase 3 AI Profile: Machine Surface](doc/phase3-ai.md)
- [Phase 4: Human Tactic](doc/phase4-human.md)
- [Phase 4 AI Profile: Machine Tactics](doc/phase4-ai.md)
- [Phase 5: Human IDE/API](doc/phase5-human.md)
- [Phase 5 AI Profile: Machine IDE/API](doc/phase5-ai.md)
- [Phase 6 Human Profile: Library](doc/phase6-human.md)
- [Phase 6 AI Profile: Machine Standard Library](doc/phase6-ai.md)
- [Phase 7: AI Search](doc/phase7-ai.md)
- [Phase 8 Human Profile: Independent Checker](doc/phase8-human.md)
- [Phase 8 AI Profile: Checker Audit Automation](doc/phase8-ai.md)
- [Phase 9 Human Profile: Advanced Features](doc/phase9-human.md)
- [Phase 9 AI Profile: Advanced Automation](doc/phase9-ai.md)
