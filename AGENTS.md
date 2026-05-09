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

- 実装基準: `doc/core-spec-v0.1.md`
- 全体像: `doc/overall-design.md`
- kernel / core calculus: `doc/phase0.md`, `doc/phase1.md`
- certificate: `doc/phase2.md`
- 表層言語 / elaborator: `doc/phase3-human.md`
- tactic: `doc/phase4-human.md`, `doc/phase4-ai.md`
- IDE / API: `doc/phase5-human.md`, `doc/phase5-ai.md`
- 標準ライブラリ: `doc/phase6-human.md`, `doc/phase6-ai.md`
- AI 探索: `doc/phase7.md`
- 独立 checker: `doc/phase8.md`
- 高度化: `doc/phase9.md`

## Rust kernel の設計ルール

- 型検査、definitional equality、reduction、universe constraint、inductive check を明確に分ける。
- AST は文字列処理ではなく構造化データとして扱う。
- de Bruijn index / level などの束縛表現は仕様と実装の対応を崩さない。
- β / δ / ι / ζ reduction の責務と停止性を明確にする。
- エラーは人間向け文字列だけでなく、テスト可能な構造化 enum として返す。
- kernel API はテストから直接呼べるようにし、CLI や server には依存させない。

## テスト方針

変更に応じて以下を使います。

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
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
- phase の責務をまたぐ変更は、README または該当する `doc/phase*.md` も更新する。
- kernel の trusted base を広げる変更は、必ず理由、代替案、検査境界を文書化する。
- 標準ライブラリでは `sorry` 相当や未許可 axiom を前提にしない。
