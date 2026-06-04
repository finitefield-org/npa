# Proof Corpus Tooling Improvement Plan

Date: 2026-06-03

この文書は、proof corpus authoring を速くするための tooling 改善計画と仕様です。
実装計画であり、証明受理の根拠ではありません。

## 1. 目的

proof corpus は、AI が定理を多数試し、固まった証明を `npa-mathlib` へ移すための
staging 環境です。authoring 中は局所検証を速くし、release / promotion / package
compatibility の確認は明示的な gate に寄せます。

この計画の対象:

- 複数 module を batch build して、authoring index 更新を最後にまとめる。公開用 package
  metadata は promote / release handoff で明示的に生成する。
- corpus authoring が読む `npa-mathlib` / external package の verified certificate cache を
  process 間で再利用する。
- corpus authoring 用の gate から package-wide CLI examples を外し、daily / PR gate 側へ寄せる。
- corpus module から `npa-mathlib` package への promotion を定型化する command / skill を作る。

## 2. 信頼境界

NPA の信頼境界は変えません。

```text
信頼しない:
  AI / tactic / replay / metadata / theorem index / promotion plan / cache file

信頼する:
  canonical certificate
  deterministic hash
  small Rust kernel
  source-free checker / verifier verdict
```

特に process 間 cache は performance 補助です。cache hit は authoring fast path の
時間短縮には使えますが、release verdict、public `npa-mathlib` promotion verdict、
high-trust audit の根拠にしてはいけません。最終判断では source-free verifier と
package artifact checks を cache なし、または cache を証明根拠にしない mode で通します。

## 3. 改善 1: Batch Module Build

### 3.1 背景

`--build-module MODULE` は指定 module と import closure を rebuild します。通常 authoring では
公開用 `manifest.toml`、`npa-package.toml`、`generated/package-lock.json` は更新せず、
module artifacts と AI theorem index だけを更新します。複数 module を連続で追加する場合、
下流依存の rebuild 順が手作業になりやすいため、batch build を使います。

### 3.2 CLI 仕様

追加する authoring command:

```sh
cargo run -p npa-proof-corpus -- --build-modules Proofs.Ai.X Proofs.Ai.Y
cargo run -p npa-proof-corpus -- --build-modules-file proofs/generated/build-batch.txt
```

オプション:

```text
--build-modules <MODULE>...
  指定 module 群と、それらに必要な import closure を一度だけ topological order で build する。

--build-modules-file <PATH>
  1 行 1 module の batch spec を読む。空行と # comment は無視する。

--package-metadata
  promote / release handoff 用。module artifacts をすべて生成した後で、manifest / package /
  package lock / AI index を 1 回だけ更新する。

--metadata-once
  `--package-metadata` の互換 alias。

--failures-out <PATH>
  失敗 module / declaration / diagnostic を JSON sidecar として出す。
```

`--build-module MODULE` は既存互換のため残し、内部的には 1 要素 batch として扱います。

### 3.3 動作

1. 入力 module 名を検証する。
2. import closure を計算する。
3. closure を topological order に並べる。
4. すでに hash が一致する module は build を skip できる。
5. dirty / changed / explicitly requested module を build する。
6. すべて成功した場合だけ、AI index を更新する。
7. `--package-metadata` が指定された場合だけ、manifest / package metadata / lock をまとめて更新する。
8. 一部失敗した場合は、成功 module の certificate は残してよいが、metadata は更新しない。

### 3.4 完了条件

- `--build-modules A B` が `--build-module A` と `--build-module B` の連続実行より rebuild 手順を減らす。
- batch 内の共有 import closure は 1 回だけ build / verify される。
- 失敗時に stale package metadata を書かない。
- `--build-module` の既存挙動が壊れない。

## 4. 改善 2: Verified Certificate Cache

### 4.1 背景

`--module`、`--changed-only`、package verifier tests は、同じ checked-in certificate と
同じ import certificates を process ごとに再度 decode / verify します。特に corpus が
`npa-mathlib` や外部 package の verified certificate を import する authoring では、これが
反復時間を押し上げます。

### 4.2 Scope

cache は authoring fast path 用です。`npa-mathlib` の public release verdict を短縮する
仕組みではありません。次では既定無効にします。

- `./scripts/check-corpus.sh`
- release / publish-plan / public package verification
- independent checker / high-trust audit
- `npa-mathlib` release handoff の最終 gate

### 4.3 Cache key

cache key は少なくとも次を含めます。

```text
core_spec
certificate_format
kernel_profile
verifier_profile
npa binary build identity
certificate_hash
certificate_file_hash
direct import module names
direct import export hashes
direct import certificate hashes
import closure certificate file hashes
axiom policy fingerprint
enabled core features
```

cache entry は content-addressed path に置きます。

```text
target/npa-proof-cache/verified-v0.1/<cache-key>.json
```

PCT-05 では、authoring cache の data model として
`npa-proof-corpus.verified-cache.v0.1` schema、content-addressed key material、
entry JSON、schema version mismatch を miss として扱う判定を実装済みです。
PCT-06 では `--module` / `--changed-only` に対して lookup / write / hit reporting を実装済みです。
cache key には direct import identity に加えて import closure の certificate file hash も入れるため、
依存 certificate file が変わった場合は authoring hit にならず live verifier に戻ります。
gate scripts は `--verified-cache authoring` を渡さないため、release-like path の既定は cache off です。

### 4.4 CLI 仕様

```sh
cargo run -p npa-proof-corpus -- --module Proofs.Ai.X --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache authoring
cargo run -p npa-proof-corpus -- --changed-only --verified-cache off
```

mode:

```text
off
  cache を使わない。release / corpus gate の既定。

authoring
  cache hit を authoring verification の短縮に使う。出力に cached verdict であることを明示する。

read-through
  cache lookup はするが、最終的に verifier を再実行して cache と比較する。debug 用。
  cache entry が live verifier の結果と一致しない場合は stale として破棄し、live result を再書き込みする。
```

cache mode が有効な場合、deterministic text output に次の形で status を出します。

```text
verified Proofs.Ai.X cache_status = "hit" cache_mode = "authoring"
verified Proofs.Ai.X cache_status = "stale" cache_mode = "read-through"
```

### 4.5 完了条件

- cache を削除しても受理結果が変わらない。
- cache hit は machine-readable output に `cache_status = "hit"` として出る。
- `check-corpus.sh` は cache なしで通る。
- cache entry の schema version mismatch は安全に miss として扱う。

## 5. 改善 3: Gate Split For Package-Wide CLI Examples

### 5.1 背景

以前の `check-corpus.sh` には package-wide CLI examples が含まれ、authoring 直後の feedback loop には重すぎました。
これらは package CLI の end-to-end 回帰として重要ですが、個々の theorem authoring の毎回確認には
過剰です。

### 5.2 Gate 分類

PCT-03 で追加された gate:

```sh
./scripts/check-corpus-authoring.sh
./scripts/check-corpus-package.sh
./scripts/check-corpus-full.sh
```

役割:

```text
check-corpus-authoring.sh
  changed proof corpus modules の source-free 検査だけを authoring cache 付きで実行する。
  package-wide CLI examples、axiom-report、index、publish-plan は含めない。

check-corpus-package.sh
  package verifier、package CLI examples、publish-plan、index、axiom-report の package-wide 回帰。
  npa-mathlib promotion、package tooling、release/high-trust 境界で実行する。

check-corpus-full.sh
  authoring + package をまとめた promotion / release / high-trust 手前の full gate。
```

既存の `./scripts/check-corpus.sh` は互換性のため残し、軽量
`check-corpus-authoring.sh` を呼ぶ alias とします。重い gate は
`check-corpus-package.sh` / `check-corpus-full.sh` を明示的に呼びます。
script 分割前の古い案内では `./scripts/check-corpus.sh` が full corpus gate でしたが、
現在の AGENTS.md / CONTRIBUTING.md / README.md は staging corpus の通常 authoring を軽量 gate に寄せます。

### 5.3 完了条件

- theorem authoring の通常終了時に `check-corpus-authoring.sh` だけを案内できる。
- package-wide CLI examples は PR / daily gate で残る。
- `AGENTS.md` と `develop/proof-corpus-ai-workflow.md` の gate 方針が新しい script 名に追随する。

## 6. 改善 4: Promotion Command / Skill

### 6.1 背景

固まった corpus module は `npa-mathlib` に移したい一方、namespace 変換、import mapping、
package metadata 更新、downstream smoke が手作業になっています。

既に判定用 skill として `judge-promote-to-mathlib` を用意しています。次は promotion plan の生成と
materialization を command 化します。

### 6.2 CLI 仕様

PCT-04 で実装済み:

```sh
cargo run -p npa-proof-corpus -- \
  --promote-plan Proofs.Ai.Algebra.AbstractField \
  --mathlib-root ../npa-mathlib \
  --to-module Mathlib.Algebra.Field.Basic \
  --out develop/npa-mathlib-field-closure-audit.md
```

`--promote-plan` は `--mathlib-root` 配下を読み取り専用の evidence source として扱います。
`--out` が `--mathlib-root` 配下を指す場合は、plan 生成前に deterministic diagnostic で失敗します。

PCT-07 で実装済みの materialize command:

```sh
cargo run -p npa-proof-corpus -- \
  --promote-materialize develop/npa-mathlib-field-closure-audit.md \
  --mathlib-root ../npa-mathlib \
  --dry-run \
  --compat-alias none
```

既定は dry-run です。`--apply` を指定した場合だけ target package の source、
certificate、meta、replay、`npa-package.toml` を書きます。git staging は行わず、
書いた path を deterministic text output に列挙します。PCT-04 plan 内で import mapping、
axiom policy、compatibility alias decision が未解決のままなら拒否します。ただし
`--compat-alias none` は operator が「互換 alias なし」と明示判断するための option です。

### 6.3 Promotion plan 内容

promotion plan は次を含みます。

- corpus source module と target `Mathlib.*` module の対応。
- direct import mapping。
- import closure と public package へ入る module set。
- axiom policy の差分。
- theorem / definition / inductive export list。
- compatibility alias が必要かどうか。
- `npa-mathlib` package gate と downstream smoke command。
- source-free verification evidence 欄。

### 6.4 完了条件

- plan 生成だけなら `npa-mathlib` repo を変更しない。
- materialize は dry-run / apply を分ける。
- dry-run は intended file / manifest / package metadata / namespace change を表示し、
  `npa-mathlib` repo を変更しない。
- apply は target package artifacts と manifest だけを書き、git stage は行わない。
- materialize 後に package check、build-certs --check、verify-certs --checker reference、
  check-hashes、axiom-report --check、index --check が案内される。
- source-free downstream smoke が promotion checklist に含まれる。

## 7. 実装順

推奨順:

1. Batch module build を入れる。
2. Gate split を入れて authoring loop の既定を軽くする。
3. Promotion plan command を入れる。
4. Verified certificate cache を authoring-only として入れる。
5. Promotion materialize command を入れる。

cache は効果が大きい一方で信頼境界の説明が難しいため、release path と切り離した後に入れます。

## 8. 計測指標

各 milestone で次を記録します。

- single module build time
- batch build time
- changed-only verification time
- authoring gate time
- package gate time
- cache hit ratio
- cache disabled full gate time

目標は、theorem authoring の通常 loop を full corpus gate ではなく局所 build / verify の時間に
近づけることです。

PCT-08 の最終計測は `develop/proof-corpus-tooling-pct-08-measurement.md` に記録します。
PCT-00 baseline の full corpus gate は 1059.81s でした。PCT-08 では clean small-module
authoring loop、つまり `--build-module Proofs.Ai.Basic`、selected module source-free verification、
`--changed-only` の合計が 2.69s でした。これは baseline full gate より約 394 倍短いです。

PCT-08 時点の `./scripts/check-corpus-authoring.sh` は 115.21s で通りました。現在の authoring gate は
さらに changed-only source-free 検査へ絞り、proof corpus staging の通常 batch boundary 用 gate として使います。
`./scripts/check-corpus-package.sh` は 1122.39s で通り、
package verifier、package CLI examples、axiom-report、index、publish-plan の回帰を含むため、
npa-mathlib promotion / release handoff / compatibility changes の境界に寄せます。

cache、promotion plan、promotion dry-run、theorem index、replay、metadata、CI status、timing log は
すべて未信頼 sidecar です。これらは作業効率や audit の入力には使えますが、証明受理の根拠にはしません。
