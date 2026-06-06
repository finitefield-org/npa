# AGENTS.md

このリポジトリで作業するエージェント向けの作業指針です。

## プロジェクトの目的

NPA は certificate first な依存型証明支援系です。便利な上位機能ではなく、
最終的に検査される canonical proof certificate を中心に設計します。

最重要の信頼境界は次です。

```text
信頼しない:
  parser / elaborator / tactic / automation / AI / plugin / theorem search

信頼する:
  小さい Rust kernel
  canonical certificate
  独立 checker
```

## 実装方針

- kernel 部分は Rust で実装する。
- kernel は小さく保ち、I/O、ネットワーク、plugin loading、AI 呼び出しを入れない。
- tactic や elaborator は proof term / certificate を生成するだけで、正しさの根拠にしない。
- certificate checker が読む表現は canonical core AST に限定する。
- 表層構文、notation、implicit arguments、typeclass search、holes は core calculus に入れない。
- hash、serialization、error reporting は決定的にする。
- `unsafe` Rust は原則使わない。必要な場合は理由と境界を明記する。

## 作業前に読む資料

大きな実装変更の前に、該当する phase の資料を確認してください。

- 実装基準: `develop/core-spec-v0.1.md`
- 全体像: `develop/overall-design.md`
- kernel / core calculus: `develop/phase0.md`, `develop/phase1.md`
- certificate: `develop/phase2.md`
- 表層言語 / elaborator: `develop/phase3-human.md`
- tactic: `develop/phase4-human.md`, `develop/phase4-ai.md`
- IDE / API: `develop/phase5-human.md`, `develop/phase5-ai.md`
- 標準ライブラリ: `develop/phase6-human.md`, `develop/phase6-ai.md`
- AI 探索: `develop/phase7-ai.md`
- 独立 checker: `develop/phase8-human.md`, `develop/phase8-ai.md`
- 高度化: `develop/phase9-human.md`, `develop/phase9-ai.md`
- proof corpus を AI で拡大する作業: `develop/proof-corpus-ai-workflow.md`
- 定理証明用 repo-local skill: `.agents/skills/prove-theorem/SKILL.md`

## Rust kernel の設計ルール

- 型検査、definitional equality、reduction、universe constraint、inductive check を明確に分ける。
- AST は文字列処理ではなく構造化データとして扱う。
- de Bruijn index / level などの束縛表現は仕様と実装の対応を崩さない。
- β / δ / ι / ζ reduction の責務と停止性を明確にする。
- エラーは人間向け文字列だけでなく、テスト可能な構造化 enum として返す。
- kernel API はテストから直接呼べるようにし、CLI や server には依存させない。

## テスト方針

通常の開発では proof corpus を hot path に入れません。corpus 以外の変更では、まず
短時間で終わる fast gate を使います。

```sh
./scripts/check-fast.sh
```

これは内部で次を実行します。

```sh
cargo fmt --all -- --check
cargo clippy --workspace --exclude npa-proof-corpus --all-targets -- -D warnings
cargo test --workspace --exclude npa-proof-corpus -- \
  --skip proof_corpus \
  --skip proof_package \
  --skip package_fast_verifier_ \
  --skip package_reference_verifier_ \
  --skip package_phase8_ \
  --skip package_source_free_
```

proof corpus は作業用 staging space であり、公開用 package ではありません。
`proofs/**` に定理を追加・修正する通常 authoring では、package-wide な検査を hot path に
入れず、局所 build / source-free verify と軽量 authoring gate を使います。
通常の `--build-module` / `--build-modules` は公開用 package metadata を生成せず、
source / certificate / meta / replay と未信頼 AI theorem index だけを更新します。
`manifest.toml`、`npa-package.toml`、`generated/package-lock.json`、axiom-report、
theorem-index、publish-plan は `npa-mathlib` への promote / release handoff など
公開境界で明示的に確認・生成します。

proof corpus authoring の通常確認:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
./scripts/check-corpus-authoring.sh
```

`check-corpus-authoring.sh` は、changed proof corpus modules の source-free 検査だけを
`--verified-cache authoring` 付きで実行する軽量 gate です。既存の
`./scripts/check-corpus.sh` も互換 wrapper としてこの軽量 authoring gate を実行します。

重い proof corpus package gate は、次のいずれかに該当する場合だけ実行します。

- `npa-mathlib` への promote 直前、または promote materialize / closure audit の完了確認。
- `tools/proof-corpus/**` の promotion、package lock、artifact 生成に関わる変更。
- promote / release handoff のために `proofs/npa-package.toml`、`proofs/generated/package-lock.json`、
  axiom-report、theorem-index、publish-plan など package generated artifacts を意図的に更新する場合。
- certificate の canonical encode / decode / hash / import / axiom report に関わる変更。
- kernel の core semantics、typecheck、reduction、universe、inductive に関わる変更。
- independent checker、package verifier、package lock、artifact validation に関わる変更。
- `.npcert` の生成・検査互換性に関わる変更。
- release / high-trust gate を実行する場合。

該当する場合は、変更の性質に応じて split corpus gate を明示的に実行します。

```sh
./scripts/check-corpus-package.sh
./scripts/check-corpus-full.sh
```

`check-corpus-package.sh` は package verifier、package CLI examples、axiom-report、index、
publish-plan の package-wide 回帰に使います。`check-corpus-full.sh` は軽量 authoring gate と
package gate をまとめた promote / release / high-trust 手前の full gate です。

proof corpus に定理を追加している authoring 中は、毎回 package/full gate を走らせず、
`develop/proof-corpus-ai-workflow.md` の局所確認コマンドを優先します。

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.X
cargo run -p npa-proof-corpus -- --module Proofs.Ai.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
cargo run -p npa-proof-corpus -- --write-replay Proofs.Ai.X::theorem_name proofs/generated/replay-X-theorem.json
```

`--build-module` は指定 module と import closure だけを source から再生成する authoring 用補助です。
公開用 package metadata も併せて更新する必要がある promote 準備では、明示的に
`--package-metadata` を付けます。
`--module` / `--changed-only` は checked-in certificate を source-free に検査する補助であり、
source 変更を certificate に反映する用途には `--build-module` を先に使います。

AI theorem index が必要な場合は次で更新します。この index は未信頼 sidecar であり、
証明の受理根拠にはしません。

```sh
cargo run -p npa-proof-corpus -- --write-ai-index
```

kernel 周辺では、少なくとも次のケースを追加してください。

- well-typed term が通ること
- ill-typed term が拒否されること
- definitional equality の正例と負例
- universe constraint の正例と負例
- certificate hash / import hash が決定的であること
- axiom report が意図せず増えないこと

## 変更時の注意

- unrelated な設計文書やユーザー変更を巻き戻さない。
- phase の責務をまたぐ変更は、README または該当する `develop/phase*.md` も更新する。
- kernel の trusted base を広げる変更は、必ず理由、代替案、検査境界を文書化する。
- 標準ライブラリでは `sorry` 相当や未許可 axiom を前提にしない。
