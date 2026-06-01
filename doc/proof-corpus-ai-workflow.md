# Proof Corpus AI Workflow

この文書は、proof corpus を拡大するときに AI が時間をかけすぎずに定理を追加するための
運用方針です。NPA の信頼境界は変えません。AI、replay、metadata、theorem index はすべて
未信頼 sidecar であり、受理根拠は canonical certificate と checker / kernel の検査結果だけです。

## 基本方針

AI は証明を「信用される成果物」として出すのではなく、安い候補を大量に出します。
各候補は Machine Surface / tactic API / certificate verifier に即座に通し、失敗したら structured
diagnostic を AI に戻して修正します。

探索順は原則として安い順にします。

```text
exact local hypothesis
exact known theorem
rw / simp-lite
apply theorem + subgoal generation
induction-nat
explicit proof term
new lemma
```

Human Surface の便利機能は corpus authoring には使ってよいですが、AI 探索の hot path では
Machine Surface、tactic candidate、source-free certificate verification を優先します。

## 通常の追加ループ

1. 追加する定理を小さい module に入れる。
2. import を最小化する。
3. AI 用 theorem index を必要に応じて更新する。
4. 追加した module と import closure だけを source から再生成する。
5. 追加した module だけ source-free に検査する。
6. 失敗した declaration だけ focused replay に切り出して AI repair に戻す。
7. まとまったところで changed-only / corpus gate を実行する。

よく使うコマンド:

```sh
cargo run -p npa-proof-corpus -- --build-module Proofs.Ai.Basic
cargo run -p npa-proof-corpus -- --write-ai-index
cargo run -p npa-proof-corpus -- --module Proofs.Ai.Basic
cargo run -p npa-proof-corpus -- --changed-only
cargo run -p npa-proof-corpus -- --changed-only --failures-out proofs/generated/failed-corpus-replay.json
cargo run -p npa-proof-corpus -- --write-replay Proofs.Ai.Basic::id proofs/generated/replay-basic-id.json
```

`--build-module MODULE` は authoring 用の高速補助です。指定 module とその import closure だけを
Human Surface source から compile し、`source.npa`、`certificate.npcert`、`meta.json`、`replay.json`、
`manifest.toml`、`npa-package.toml`、`generated/package-lock.json`、AI theorem index を更新します。
下流 module は再生成しないため、基礎 module の export hash が変わった場合は、必要な下流 module も
順に `--build-module` するか、最後に full corpus gate で検出します。

`--module` と `--changed-only` は checked-in certificate を source-free に検査します。依存 module は
再帰的に読み込まれ、同一プロセス内で verified module / decoded certificate が cache されます。

## Shard

大きめの確認を分割したいときは zero-based shard を使います。

```sh
cargo run -p npa-proof-corpus -- --verify --shard 0/4
cargo run -p npa-proof-corpus -- --verify --shard 1/4
cargo run -p npa-proof-corpus -- --verify --shard 2/4
cargo run -p npa-proof-corpus -- --verify --shard 3/4
```

`--changed-only --shard 0/2` のように changed set に対しても使えます。

## AI Theorem Index

`--write-ai-index` は `proofs/generated/ai-theorem-index.json` を生成します。
これは AI retrieval 用の軽量 index です。定理名、statement、import、certificate path、replay path、
focused replay spec を含みますが、trusted artifact ではありません。

既存の package theorem index は certificate-derived な広い release artifact です。
AI 作業中は軽量 index を先に使い、必要になったときだけ重い package / corpus gate を実行します。

## Focused Replay

失敗した declaration だけを AI に戻すには `--write-replay MODULE::DECL PATH` を使います。

```sh
cargo run -p npa-proof-corpus -- \
  --write-replay Proofs.Ai.Basic::id proofs/generated/replay-basic-id.json
```

focused replay は未信頼 sidecar です。再投入された候補は、通った場合だけ certificate handoff に進めます。

## Gate

通常開発では `./scripts/check-fast.sh` を使います。

proof corpus を変更した場合、または certificate / kernel / checker / package verification の互換性に
関わる変更では `./scripts/check-corpus.sh` を実行します。作業中に毎回 full corpus gate を回すのではなく、
`--module`、`--changed-only`、`--shard` で局所確認してから最後に gate を通します。
