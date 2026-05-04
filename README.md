# NPA

NPA は **Neuro-symbolic Proof Assistant** の実験的な設計・実装リポジトリです。

目標は Lean や Rocq から学びつつ、AI 時代向けに最初から設計された
**proof certificate first** な依存型証明支援系を作ることです。

```text
AI が証明候補を探し、
人間が形式化の意図を確認し、
Rust 製 kernel と独立 checker が proof certificate だけを検査する。
```

## 基本方針

- kernel は Rust で実装する。
- AI、tactic、elaborator、parser、theorem search は trusted base に入れない。
- 最終的な正しさは source script ではなく canonical proof certificate で保証する。
- kernel は小さく、監査しやすく、決定的に動くものにする。
- certificate は再検査可能で、import hash、declaration hash、axiom report を含む。

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
| 3 | 表層言語、parser、name resolution、notation、elaboration |
| 4 | tactic 層: `intro`, `exact`, `apply`, `rw`, `simp-lite`, `induction` |
| 5 | IDE / API: proof state、tactic execution、theorem search、goal display |
| 6 | 小さく堅い標準ライブラリ: `Std.Logic`, `Std.Nat`, `Std.List`, `Std.Algebra.Basic` |
| 7 | AI 証明探索: premise retrieval、tactic generation、search、repair |
| 8 | 独立 checker、external checker、CI audit |
| 9 | advanced inductive、quotient、typeclass、SMT certificates、theorem graph |

## リポジトリ構成

```text
.
├── Cargo.toml
├── README.md
├── AGENTS.md
├── crates/
│   └── npa-kernel/
│       └── src/lib.rs
└── doc/
    ├── overall-design.md
    ├── phase0.md
    ├── phase1.md
    └── ...
```

現時点では Rust kernel の最小実装と設計資料が中心です。certificate checker、surface language などは
今後この設計に沿って追加します。

## 開発メモ

少なくとも次を通す方針です。

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## 参考資料

- [NPA Core Specification v0.1](doc/core-spec-v0.1.md)
- [全体設計](doc/overall-design.md)
- [Phase 0: Core Spec](doc/phase0.md)
- [Phase 1: Kernel](doc/phase1.md)
- [Phase 2: Certificate](doc/phase2.md)
