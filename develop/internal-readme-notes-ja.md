# Internal README Notes

この文書は、PUB-01 で root `README.md` を英語の公開入口に整理した際に、
旧 README から外した内部向けの日本語メモを保存する場所です。

公開ユーザー向けの入口は root `README.md` と `docs/README.md` です。この文書は
maintainer / development agent 向けの内部メモであり、proof evidence ではありません。

## 旧 README の実装状況メモ

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
OCaml clean-room `npa-checker-ext` source は `checkers/npa-checker-ext/` にあります。
release/high-trust evidence として存在すると扱うのは、build 済み binary が runner-owned
checker registry から解決され、package `--checker external` integration と binary hash /
identity validation が通った場合だけです。`package high-trust` は
`verified_high_trust` artifact generator として実装済みで、copyable opt-in
high-trust CI template は `ci-templates/github-actions/npa-package-high-trust.yml` に
あります。ただし reference-only evidence から artifact を生成しません。

External checker benchmark summaries are release audit metadata linked to
checker result hashes. They may fail release/high-trust policy as regression
evidence, but they are not checker verdicts and do not affect proof validity.

これらの `npa-api` automation / library API は候補生成、検査要求の構成、
監査 artifact の正規化、回帰 fixture の実行を担う非信頼層です。
trusted base は広げません。証明の受理根拠は引き続き canonical certificate と、
Rust kernel / 独立 checker が返す deterministic result だけです。

## 旧 README の開発メモ

通常開発では proof corpus を hot path に入れず、まず短時間の fast gate を通します。

```sh
./scripts/check-fast.sh
```

`./scripts/check-fast.sh` は `npa-proof-corpus` と proof-corpus-backed package verifier / CLI fixture
tests を除外して、format / clippy / workspace tests を実行します。
proof corpus gate は次の条件に該当する変更だけで実行します。

- `proofs/**` または `tools/proof-corpus/**` の変更
- certificate の canonical encode / decode / hash / import / axiom report に関わる変更
- kernel の core semantics、typecheck、reduction、universe、inductive に関わる変更
- independent checker、package verifier、package lock、artifact validation に関わる変更
- `.npcert` の生成・検査互換性に関わる変更
- release / high-trust gate

該当する場合は次を実行します。

```sh
./scripts/check-corpus.sh
```

proof corpus に定理を追加している間は、毎回 full corpus gate を回さず、対象 module だけを
再生成・検査します。

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.X
cargo run -p npa-proof-corpus -- --changed-only
```

`--build-module` は source から指定 module と import closure だけを再生成する authoring 用補助です。
`--module` / `--changed-only` は checked-in certificate の source-free 検査です。詳しい AI 向け手順は
`develop/proof-corpus-ai-workflow.md` を参照してください。

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
