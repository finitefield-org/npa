# Phase 9 Human Task Breakdown

このタスク分解は `doc/phase9-human.md` を正とし、現在の
`crates/npa-kernel` / `crates/npa-cert` / `crates/npa-frontend` / `crates/npa-api` /
`crates/npa-checker-ref` 実装との差分を、Phase 9 Human Profile の
kernel-facing / checker-facing / user-facing 高度化マイルストーンに分けたものです。

Phase 9 Human は、advanced inductive、universe polymorphism 強化、typeclass、quotient、
SMT certificates、theorem graph、natural language formalization を、人間が使える高水準機能として
実装する段階です。ただし信頼境界は変えません。

```text
信頼しない:
  parser / elaborator / typeclass search / SMT solver / theorem graph / AI formalizer / automation

信頼する:
  small Rust kernel
  canonical core AST
  canonical certificate
  independent checker
```

`crates/npa-api/src/advanced_ai.rs` の Phase 9 AI deterministic validation / replay substrate と
M9 fixture matrix は実装済みとして扱います。これは高度機能候補を検査境界へ戻す非信頼 automation であり、
Human Profile の kernel / checker-facing trusted rules を置き換えません。

重要な制約:

```text
- kernel に AI 呼び出し、SMT solver process、theorem graph store、network、plugin loading、filesystem discovery を入れない。
- typeclass search、theorem graph ranking、natural language formalizer は kernel / checker の proof acceptance boundary にしない。
- SMT solver の unsat / valid 結果だけで成功扱いしない。最終 NPA proof term を kernel / checker が検査する。
- quotient primitive を入れる場合は fast kernel だけでなく reference checker / external checker profile 側にも同じ規則を追加する。
- certificate には unresolved universe meta、AI trace、typeclass search trace、SMT solver log、natural language confidence を入れない。
- generated recursor / iota rule / theorem graph / SMT reconstruction / intent certificate の hash は deterministic にする。
- AI candidate hot path には full independent checker、external checker、SMT proof reconstruction、release audit、certificate-wide graph extraction を同期挿入しない。
- Phase 9 Human の変更で Phase 9 AI fixture matrix、Phase 5-7 replay / verify identity hash、Phase 8 checker result semantics を壊さない。
```

---

## 0. 現在の実装境界

### 0.1 実装済みとして扱うもの

現在のリポジトリには、Phase 9 Human の土台として使える次の実装があります。

```text
crates/npa-kernel
- core Expr / Level / Env / Decl
- simple inductive / constructor / recursor check
- positivity failure / Prop large elimination restriction
- β / δ / ι / ζ conversion

crates/npa-cert
- canonical .npcert encode / decode
- declaration / export / certificate / axiom report hash
- simple inductive generated recursor artifact hash
- Phase 2 canonical inductive artifact generator

crates/npa-frontend
- Human Surface parser / resolver / elaborator
- simple inductive declaration
- explicit universe argument handling
- Human source interface metadata outside certificate hash

crates/npa-api
- Phase 5 Machine / Human API substrate
- Phase 7 search controller and replay / verify handoff
- Phase 8 independent checker audit automation
- Phase 9 AI advanced automation endpoint substrate and M9 fixture matrix

crates/npa-checker-ref
- source-free reference checker binary
- minimal type / conversion / simple inductive / axiom report checker
```

### 0.2 Phase 9 Human で実装する target scope

```text
- universe meta / constraint solving / canonicalization and optional cumulativity policy
- indexed / mutual / approved nested inductive declarations
- deterministic recursor / induction principle / iota rule generation and checker comparison
- certificate-derived theorem graph extraction, deterministic export, query API, and retrieval integration
- class / instance surface syntax, dictionary elaboration, bounded typeclass search, and notation integration
- quotient_v1 primitive extension, feature flag, reference checker support, and Std.Quotient examples
- SMT certificate schema, small QF fragment encoding, proof reconstruction, and smt tactic
- natural language formalization confirmation flow, reverse translation, and intent certificate
- final documentation / release gate alignment
```

### 0.3 Phase 9 Human では target integration として残すもの

```text
- production LLM / RAG / external SMT solver service operation
- online graph store or embedding index operation
- general nested inductive positivity beyond approved strictly-positive functors
- full nonlinear arithmetic, arrays, bitvectors, datatypes, and quantifier-heavy SMT success
- verified checker implementation beyond current reference checker profile
- AI confidence or theorem graph score as proof acceptance condition
```

---

## 1. AI 向け高速経路を守る設計ルール

Phase 9 Human の各マイルストーンでは、次を acceptance criteria として扱います。

```text
- Phase 9 Human の heavy check は AI candidate enumeration の inner loop に入れない。
- theorem graph は候補ごとに certificate 全体から抽出せず、build / release / index update で作る deterministic snapshot を検索する。
- bounded typeclass search は timeout / max_depth / max_candidates / cycle detection を持つ。
- SMT solver process と proof reconstruction は tactic / adoption / audit 境界で実行し、ranking feature にはしない。
- natural language formalization は proof search より前に formal statement hash を確定する。
- Phase 9 AI deterministic validation / replay fixture は引き続き AI model、network、random seed なしで再現できる。
- Phase 5-7 replay / verify と Phase 8 checker result の deterministic identity hash を Phase 9 Human metadata で変えない。
```

AI path は次の形を維持します。

```text
Machine Surface request
  -> Phase 5 machine session / tactic batch / replay / verify
  -> Phase 7 candidate ranking / repair / minimization
  -> Phase 9 AI bounded candidate validation where applicable
  -> closed certificate candidate
  -> optional post-acceptance / checker / release audit
```

---

## 2. 実装順

Phase 9 Human は `doc/phase9-human.md` の #10 を正として実装します。universe と advanced inductive を先に固め、
その上で theorem graph、typeclass、quotient、SMT、natural language formalization を積みます。

```text
0. Phase 9 Human / AI boundary と performance guard を固定する
1. universe constraint data model / canonical hash を固定する
2. universe meta solver / elaborator integration を実装する
3. universe polymorphic library regression と checker consistency を固定する
4. indexed inductive family core / certificate を実装する
5. mutual inductive block / simultaneous recursor を実装する
6. approved nested inductive / large elimination policy を実装する
7. certificate-derived theorem graph extractor を実装する
8. theorem graph API / retrieval integration / performance guard を実装する
9. class / instance declaration と dictionary elaboration を実装する
10. bounded typeclass search / notation integration を実装する
11. quotient_v1 primitive / certificate feature flag を固定する
12. Std.Quotient / checker support / quotient examples を実装する
13. SMT certificate schema / QF encoding / deterministic checker surface を実装する
14. SMT proof reconstruction / smt tactic を実装する
15. natural language formalization / intent certificate を実装する
16. final docs / release completion gate を固定する
```

各段階で少なくとも以下を確認します。

```sh
cargo fmt --all
cargo test -p npa-kernel
cargo test -p npa-cert
cargo test -p npa-frontend
cargo test -p npa-api advanced_ai
cargo test -p npa-checker-ref
```

大きな内部変更後は次も通します。

```sh
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/phase9-regression.sh
```

---

## 3. タスク一覧

### P9H-00: Phase 9 Human / AI boundary と performance guard を固定する

実装タスク:

- [x] `doc/phase9-human.md`、`doc/phase9-ai.md`、README の Phase 9 実装境界を test 名または public docs に接続する。
- [x] Phase 9 Human の heavy checks が AI candidate hot path に同期挿入されない regression を追加する。
- [x] `crates/npa-api` の Phase 9 AI substrate が trusted checker ではないことを public API docs に明記する。
- [x] Phase 9 Human metadata が Phase 5-7 replay / verify identity hash と Phase 8 checker result を変えない fixture を追加する。
- [x] Phase 9 Regression gate が Phase 9 Human 後も固定ゲートであることを README / docs / workflow 名で確認する。

受け入れ条件:

- [x] AI sidecar、theorem graph score、SMT solver output、formalization confidence が checker verdict を作れないことが test で固定されている。
- [x] full independent checker / external checker / release audit / SMT reconstruction は AI candidate enumeration の inner loop に入らない。
- [x] `/machine/*` request / response schema、candidate hash、state fingerprint が Phase 9 Human 境界追加で変わらない。

検証:

```sh
cargo test -p npa-api advanced_ai
cargo test -p npa-api ai_search
./scripts/phase9-regression.sh
```

依存:

```text
なし
```

注意:

```text
P9H-00 は境界固定と regression guard だけを扱う。高度機能本体は後続 milestone で実装する。
```

### P9H-01: universe constraint data model / canonical hash を固定する

実装タスク:

- [x] `crates/npa-kernel` に universe constraints の構造化型を追加し、`zero / succ / max / imax / param` と整合させる。
- [x] declaration ごとの universe params / constraints を kernel declaration と certificate declaration に表現できるようにする。
- [x] constraint canonicalization と deterministic hash を `crates/npa-cert` に追加する。
- [x] unresolved universe meta を certificate encode / decode / verifier / reference checker が拒否する。
- [x] Option A の equality-only universe policy を MVP の既定として明記し、cumulativity は明示 feature flag なしでは入れない。

受け入れ条件:

- [x] `List.map` 相当の empty constraint と `max u v <= w` 相当の non-empty constraint が canonical hash を持つ。
- [x] universe param の順序、重複、unknown param、non-canonical level expression が deterministic error になる。
- [x] certificate hash / import hash が universe constraints によって安定して変化する。
- [x] reference checker と fast verifier が constraint canonical bytes を同じ意味で検査する。

検証:

```sh
cargo test -p npa-kernel universe
cargo test -p npa-cert universe
cargo test -p npa-checker-ref universe
cargo test --workspace certificate_hash
```

依存:

```text
P9H-00
```

注意:

```text
defeq と cumulativity を混ぜない。cumulativity を入れる場合も別 milestone で subtyping rule として扱う。
```

### P9H-02: universe meta solver / elaborator integration を実装する

実装タスク:

- [x] `crates/npa-frontend` に elaboration-only universe meta を導入する。
- [x] universe meta constraint collection、solving、minimization、failure diagnostics を実装する。
- [x] solved universe args を explicit core term に反映し、certificate には meta が残らないようにする。
- [x] Human Surface の implicit universe inference と Machine Surface の explicit universe fast path を分離して保つ。
- [x] Phase 9 AI `UniverseRepair` fixture と Human elaborator の solver output が同じ canonical constraints に戻ることを確認する。

受け入れ条件:

- [x] polymorphic identity / const / map 相当の theorem が explicit universe args なしの Human Surface から elaboration できる。
- [x] unsolved meta、ambiguous universe、constraint unsatisfied は structured diagnostic になる。
- [x] Machine Surface はこれまで通り explicit universe args を要求し、Human inference で candidate hash が変わらない。
- [x] solver result は deterministic で、同じ source / imports から同じ certificate hash になる。

検証:

```sh
cargo test -p npa-frontend universe
cargo test -p npa-api universe_repair
cargo test -p npa-api human
./scripts/phase9-regression.sh
```

依存:

```text
P9H-01
```

注意:

```text
universe meta は elaboration-only。kernel / certificate / checker の canonical core AST には入れない。
```

### P9H-03: universe polymorphic library regression と checker consistency を固定する

実装タスク:

- [x] `Std.Logic` / `Std.List` / `Std.Algebra.Basic` の polymorphic declarations を universe constraints 付きで再生成する。
- [x] reference checker と fast kernel の universe check / conversion check が一致する fixture を追加する。
- [x] universe-polymorphic theorem reuse の Human examples と Machine API handoff を追加する。
- [x] constraint canonical hash が import / release manifest / theorem index に反映されることを確認する。
- [x] equality-only MVP policy と future cumulativity policy の docs を更新する。

受け入れ条件:

- [x] polymorphic `List` / `Eq` / `Prod` / `Sigma` 相当の declarations が source-free reference checker で検査できる。
- [x] universe constraint の正例と負例が kernel / cert / checker の test にある。
- [x] unresolved meta を含む certificate fixture は fast verifier と reference checker の両方で拒否される。
- [x] Phase 7 retrieval / Phase 9 AI fixtures の candidate hash は Human universe inference 追加で変わらない。

検証:

```sh
cargo test -p npa-api std_library
cargo test -p npa-checker-ref universe
cargo test -p npa-api ai_search
./scripts/phase9-regression.sh
```

依存:

```text
P9H-02
```

注意:

```text
P9H-03 は standard library の universe hardening が主目的。新しい algebraic hierarchy は P9H-09 以降で扱う。
```

### P9H-04: indexed inductive family core / certificate を実装する

実装タスク:

- [x] `InductiveDecl` / certificate schema に params と indices を明確に分けた indexed family 表現を固定する。
- [x] constructor result が対象 family と宣言済み params に一致し、indices が well-typed であることを kernel で検査する。
- [x] `Vec` / `Fin` の certificate fixtures を追加する。
- [x] generated recursor signature hash / iota rules hash を indexed family に対応させる。
- [x] reference checker が fast kernel と独立に indexed family declaration を再検査する。
- [x] `POST /inductive/check` 相当の Human API wrapper を追加し、constructors / recursor / positivity / iota hash を返す。

受け入れ条件:

- [x] `Vec A 0` / `Vec A (succ n)` の constructor result check が通る。
- [x] constructor result family mismatch、param mismatch、bad index type、negative occurrence が deterministic error になる。
- [x] indexed family の recursor / induction principle が checker と fast kernel で同じ hash になる。
- [x] `.npcert` を source なしで検査できる。
- [x] `/inductive/check` response は diagnostic metadata であり、proof acceptance boundary にはならない。

検証:

```sh
cargo test -p npa-kernel inductive
cargo test -p npa-cert inductive
cargo test -p npa-checker-ref inductive
cargo test -p npa-api inductive
```

依存:

```text
P9H-03
```

注意:

```text
AI supplied recursor は採用しない。recursor は declaration から deterministic に生成して照合する。
```

### P9H-05: mutual inductive block / simultaneous recursor を実装する

実装タスク:

- [x] `MutualInductiveBlock` を kernel / certificate / checker の境界に追加する。
- [x] block 全体の name uniqueness、well-typedness、strict positivity、constructor reference scope を検査する。
- [x] `Even` / `Odd` の mutual inductive fixture を追加する。
- [x] simultaneous recursor / induction principles と iota rules の deterministic generation を実装する。
- [x] import / export / theorem index が mutual block の generated declarations を安定順序で扱うようにする。

受け入れ条件:

- [x] mutually recursive `Even` / `Odd` が source-free certificate として検査できる。
- [x] block-local reference scope mismatch、duplicate generated name、non-positive mutual occurrence が拒否される。
- [x] generated recursor artifact hash は declaration order と canonical name order に対して安定する。
- [x] reference checker と fast kernel の iota reduction が一致する。

検証:

```sh
cargo test -p npa-kernel mutual
cargo test -p npa-cert inductive
cargo test -p npa-checker-ref inductive
cargo test -p npa-api std_library
```

依存:

```text
P9H-04
```

注意:

```text
mutual block の採用は kernel trusted base を広げるため、reason / alternative / checker boundary を docs に残す。
```

### P9H-06: approved nested inductive / large elimination policy を実装する

実装タスク:

- [x] approved strictly-positive functor table を kernel / checker の明示 policy として実装する。
- [x] `List` / `Option` / `Prod` 越しの nested recursive occurrence を positivity traversal で扱う。
- [x] `Rose` tree の positive fixture と unknown functor / higher-order negative occurrence の rejection fixture を追加する。
- [x] `Prop` から `Type` への large elimination restriction と例外候補を structured policy にする。
- [x] recursor generation と iota rules hash を approved nested profile に対応させる。

受け入れ条件:

- [x] `List (Rose A)` 相当の approved nested occurrence が通る。
- [x] unknown type constructor、`I -> A`、`I -> I`、higher-order negative occurrence は拒否される。
- [x] `I : Prop` から unrestricted `Type` motive への recursor は拒否される。
- [x] approved functor table が certificate / checker / docs で一致している。

検証:

```sh
cargo test -p npa-kernel positivity
cargo test -p npa-cert inductive
cargo test -p npa-checker-ref positivity
./scripts/phase9-regression.sh
```

依存:

```text
P9H-05
```

注意:

```text
generic positivity checker は future target。P9H-06 は approved functor profile に閉じる。
```

### P9H-07: certificate-derived theorem graph extractor を実装する

実装タスク:

- [x] `crates/npa-api` または専用 module に certificate-derived theorem graph extractor を追加する。
- [x] node schema、edge schema、node identity、edge identity、deterministic graph hash を固定する。
- [x] type / proof / transparent def body / constructor type / axiom deps に現れる `Const` を抽出する。
- [x] source notation、tactic script、AI sidecar を graph extraction input から除外する。
- [x] axiom dependency path と direct / transitive dependency query の fixtures を追加する。

受け入れ条件:

- [x] 同じ `.npcert` input から同じ graph export hash が得られる。
- [x] source text や Human debug metadata を変えても graph hash は変わらない。
- [x] axiom deps、constructor deps、recursor deps が graph に出る。
- [x] import `export_hash` / high-trust `certificate_hash` と graph snapshot の binding が検査できる。

検証:

```sh
cargo test -p npa-api theorem_graph
cargo test -p npa-api std_library
cargo test -p npa-checker-ref
```

依存:

```text
P9H-06
```

注意:

```text
P9H-07 は graph extraction。online graph store、embedding、RAG は実装しない。
```

### P9H-08: theorem graph API / retrieval integration / performance guard を実装する

実装タスク:

- [x] `/graph/dependencies` / `/graph/related` / `/graph/query` 相当の Human API wrapper を `crates/npa-api` に追加する。
- [x] Phase 7 premise retrieval が precomputed theorem graph snapshot を ranking feature として使えるようにする。
- [x] graph proximity score は proof acceptance / checker verdict に影響しないことを test で固定する。
- [x] proof minimization が graph を使って不要 import candidate を提案できるようにする。
- [x] graph extraction を candidate ごとの hot path に入れない performance regression を追加する。

受け入れ条件:

- [x] declaration ごとの direct / transitive dependencies と related theorem query が deterministic に返る。
- [x] graph result node は certificate-bound public export に限定される。
- [x] graph score の有無で final certificate hash と checker result は変わらない。
- [x] Phase 7 retrieval は graph snapshot missing 時も既存の deterministic fallback を持つ。

検証:

```sh
cargo test -p npa-api theorem_graph
cargo test -p npa-api ai_search
./scripts/phase9-regression.sh
```

依存:

```text
P9H-07
```

注意:

```text
theorem graph は AI 探索効率のための sidecar。trusted base に入れない。
```

### P9H-09: class / instance declaration と dictionary elaboration を実装する

実装タスク:

- [ ] Human Surface に `class` / `instance` declaration syntax を追加する。
- [ ] class を structure / record 相当の ordinary core declaration と searchable metadata に elaboration する。
- [ ] instance declaration を ordinary definition として certificate に入れ、metadata は certificate hash の外に分離する。
- [ ] dictionary argument を明示 core term として渡す elaboration path を追加する。
- [ ] `Add` / `Mul` / `Zero` / `One` の minimal examples を追加する。

受け入れ条件:

- [ ] class declaration は kernel が typeclass を知らなくても ordinary declaration として検査できる。
- [ ] instance metadata が壊れても final core term の型検査で拒否できる。
- [ ] certificate には search trace ではなく explicit dictionary term だけが残る。
- [ ] Machine Surface fast path は typeclass metadata を要求しない。

検証:

```sh
cargo test -p npa-frontend typeclass
cargo test -p npa-api human
cargo test -p npa-cert certificate_hash
```

依存:

```text
P9H-08
```

注意:

```text
P9H-09 は declaration と dictionary elaboration。探索アルゴリズムは P9H-10 で扱う。
```

### P9H-10: bounded typeclass search / notation integration を実装する

実装タスク:

- [ ] local / opened namespace / imported global / fallback priority の bounded instance search を実装する。
- [ ] max_depth / max_candidates / timeout / cycle detection / repeated goal cache を policy 化する。
- [ ] ambiguity、no solution、budget exceeded を structured diagnostic にする。
- [ ] `POST /typeclass/search` 相当の Human API wrapper を追加し、instance / core_term / bounded search trace を返す。
- [ ] `+` / `*` / `0` / `1` notation を typeclass 経由で dictionary term に elaboration する。
- [ ] Phase 9 AI TypeclassResolution fixture と Human search behavior の境界を docs / tests に接続する。

受け入れ条件:

- [ ] `Add Nat` の direct instance と recursive instance が budget 内で解決できる。
- [ ] 複数の異なる proof term がある場合は score で選ばず ambiguity error になる。
- [ ] search trace は diagnostic metadata であり certificate hash に入らない。
- [ ] `/typeclass/search` response の `core_term` は kernel-checkable dictionary term で、search trace は proof acceptance boundary ではない。
- [ ] timeout / budget により AI hot path の latency が bounded である。

検証:

```sh
cargo test -p npa-frontend typeclass
cargo test -p npa-api typeclass
cargo test -p npa-api advanced_ai
./scripts/phase9-regression.sh
```

依存:

```text
P9H-09
```

注意:

```text
typeclass search は非信頼 elaborator 機構。kernel / checker に探索器を入れない。
```

### P9H-11: quotient_v1 primitive / certificate feature flag を固定する

実装タスク:

- [ ] `quotient_v1` core feature flag と unsupported feature rejection を certificate / checker に追加する。
- [ ] `Quotient`, `Quotient.mk`, `Quotient.sound`, `Quotient.lift` の primitive interface を kernel policy として固定する。
- [ ] `Quotient.lift` computation rule を definitional equality に入れる範囲を実装する。
- [ ] `Quotient.sound` は proof term として扱い、quotient equality を過剰に正規化しない regression を追加する。
- [ ] axiom report / feature report に quotient 使用が出るようにする。

受け入れ条件:

- [ ] `quotient_v1` 非対応 checker は quotient certificate を deterministic に拒否する。
- [ ] `quotient_v1` 対応 profile では fast kernel と reference checker の primitive interface が一致する。
- [ ] quotient primitive は custom axiom として silently allowed にならない。
- [ ] feature flag、certificate hash、axiom report hash が deterministic に変化する。

検証:

```sh
cargo test -p npa-kernel quotient
cargo test -p npa-cert quotient
cargo test -p npa-checker-ref quotient
cargo test -p npa-api quotient
```

依存:

```text
P9H-10
```

注意:

```text
kernel trusted base を広げる milestone。理由、代替案、checker 境界を docs に残す。
```

### P9H-12: Std.Quotient / checker support / quotient examples を実装する

実装タスク:

- [ ] `Std.Quotient` に `Setoid`、relation notation、quotient helper definitions を追加する。
- [ ] quotient-capable independent checker profile を Phase 8 / Phase 9 API policy に追加する。
- [ ] Phase 9 AI `QuotientConstruction` の deterministic rejection surface を quotient success profile と衝突しないように更新する。
- [ ] `Nat × Nat` から簡易 `Int` を作る example certificate を追加する。
- [ ] `Quotient.lift` の well-defined proof obligation と compatibility proof mismatch の fixtures を追加する。

受け入れ条件:

- [ ] `Setoid`、`Quotient.mk`、`Quotient.sound`、`Quotient.lift` を使った証明が source-free checker で通る。
- [ ] relation equivalence proof や compatibility proof が間違っている場合は kernel / checker が拒否する。
- [ ] Phase 9 AI MVP の `Phase8MvpReference` unsupported fixture は、new profile 追加後も意味が stale にならない。
- [ ] quotient examples は custom axiom / sorry に依存しない。

検証:

```sh
cargo test -p npa-api std_library
cargo test -p npa-checker-ref quotient
cargo test -p npa-api advanced_ai
./scripts/phase9-regression.sh
```

依存:

```text
P9H-11
```

注意:

```text
P9H-12 は quotient-capable profile の導入。production full external checker integration は Phase 8 target integration として扱う。
```

### P9H-13: SMT certificate schema / QF encoding / deterministic checker surface を実装する

実装タスク:

- [ ] SMT certificate schema に format、solver、logic、encoded_goal_hash、smt_problem_hash、proof_hash、reconstruction metadata を追加する。
- [ ] QF propositional / EUF / simple LIA の encoding table と Nat-to-Int side condition 表現を実装する。
- [ ] SMT-LIB problem bytes と encoding hash を deterministic に生成する。
- [ ] Alethe / LFSC 等の proof payload を opaque artifact として hash / size / schema validation できるようにする。
- [ ] unsupported fragment、solver result only、hash mismatch、malformed proof payload を structured error にする。

受け入れ条件:

- [ ] QF propositional / EUF / simple LIA の supported fragment 判定が deterministic である。
- [ ] SMT solver の unsat 結果だけでは success にならない。
- [ ] encoded problem hash、SMT problem hash、proof payload hash が stable である。
- [ ] Phase 9 AI SMT deterministic rejection fixtures と Human SMT schema が矛盾しない。

検証:

```sh
cargo test -p npa-api smt
cargo test -p npa-api advanced_ai
cargo test -p npa-cert certificate_hash
```

依存:

```text
P9H-12
```

注意:

```text
P9H-13 は schema / encoding / deterministic rejection surface。solver-native success は P9H-14 で扱う。
```

### P9H-14: SMT proof reconstruction / smt tactic を実装する

実装タスク:

- [ ] small QF fragment の proof-producing reconstruction を NPA proof term へ変換する。
- [ ] reconstruction rule registry を non-empty profile として固定し、rule descriptor fingerprint を追加する。
- [ ] `smt` / `smt [lemmas]` tactic を Human Surface から呼べるようにする。
- [ ] `POST /smt/prove` 相当の Human API wrapper を追加し、problem hash / proof hash / NPA proof hash / kernel_checked を返す。
- [ ] final NPA proof term を kernel / reference checker で検査してから success にする。
- [ ] failure / unsupported fragment / checker mismatch を structured diagnostic として返す。

受け入れ条件:

- [ ] SMT final proof success は reconstructed NPA proof term が kernel / checker で通る場合だけ返る。
- [ ] solver-native proof rule が未知、premise order が曖昧、final conclusion が target と defeq でない場合は拒否される。
- [ ] `smt` tactic は solver result を trusted input として扱わない。
- [ ] `/smt/prove` は `require_certificate: true` の成功時に `kernel_checked: true` と checked proof hash を返す。
- [ ] SMT reconstruction は AI ranking / candidate enumeration の inner loop に入らない。

検証:

```sh
cargo test -p npa-api smt
cargo test -p npa-tactic smt
cargo test -p npa-checker-ref smt
./scripts/phase9-regression.sh
```

依存:

```text
P9H-13
```

注意:

```text
最初の success fragment は小さく保つ。arrays、bitvectors、nonlinear arithmetic、quantifiers は後続 scope。
```

### P9H-15: natural language formalization / intent certificate を実装する

実装タスク:

- [ ] `/formalize` 相当の Human API wrapper を追加し、複数 formal candidates、reverse translation、ambiguity report を返す。
- [ ] formal statement hash、candidate statement hash、accepted statement hash、intent certificate を分離して保存する。
- [ ] user confirmation / formalization verifier の reviewer identity と status を structured metadata にする。
- [ ] proof search は confirmed formal statement hash の後でのみ起動する flow にする。
- [ ] unconfirmed formalization を verified と呼ばない UI / API / docs regression を追加する。

受け入れ条件:

- [ ] 自然言語 source text や confidence score だけでは theorem statement を定義できない。
- [ ] candidate statement は Machine Surface / Human elaboration を通って canonical core statement hash になる。
- [ ] intent certificate と proof certificate は別 artifact で、kernel proof certificate hash に natural language text は混ざらない。
- [ ] rejected / unreviewed / reviewed formalization の fixtures が deterministic である。

検証:

```sh
cargo test -p npa-api formalization
cargo test -p npa-api advanced_ai
cargo test -p npa-api human
./scripts/phase9-regression.sh
```

依存:

```text
P9H-14
```

注意:

```text
LLM candidate generation の品質評価は対象外。P9H-15 は confirmation と certificate separation を固定する。
```

### P9H-16: final docs / release completion gate を固定する

実装タスク:

- [ ] `doc/phase9-human.md`、`doc/phase9-ai.md`、README、`doc/overall-design.md` の Phase 9 completion status を同期する。
- [ ] Phase 9 Human target scope と Phase 9 AI MVP implemented substrate の違いが stale になっていないか検索で固定する。
- [ ] `./scripts/phase9-regression.sh` が Phase 9 Human 完了後の required gate として必要な test を含むことを確認する。
- [ ] docs に trusted boundary、AI hot path performance boundary、quotient / SMT feature flag boundary を残す。
- [ ] release / high-trust docs に checker result と deterministic artifact だけが pass/fail を決めることを再確認する。

受け入れ条件:

- [ ] Phase 9 Human の完了条件が docs / tests / release gate で一致している。
- [ ] Phase 9 AI sidecar、graph score、formalization confidence、SMT solver output が trusted boundary に入っていない。
- [ ] target integration として残す production AI / graph store / full solver support を実装済みと書いていない。
- [ ] AI candidate hot path の速度を落とす required gate が PR / candidate enumeration に追加されていない。

検証:

```sh
rg -n "Phase 9|advanced inductive|universe|typeclass|quotient|SMT|theorem graph|formalization|AI hot path" README.md doc/phase9-human.md doc/phase9-ai.md doc/overall-design.md
git diff --check
./scripts/phase9-regression.sh
```

依存:

```text
P9H-15
```

注意:

```text
P9H-16 は final documentation / gate alignment。未実装の production integration を実装済みと書かない。
```

---

## 4. 完了条件

Phase 9 Human が完了したと言える条件はこれです。

```text
- universe meta が elaboration-only で解決され、certificate には canonical constraints だけが残る。
- polymorphic List / Eq / Prod / Sigma / Functor / Category 的定義が扱える。
- Vec / Fin / mutual Even/Odd / approved nested Rose が kernel / reference checker で通る。
- recursor / induction principle / iota rules は declaration から deterministic に生成され、hash が checker と fast kernel で一致する。
- theorem graph は certificate から deterministic に抽出され、dependencies / related query / retrieval sidecar として使える。
- typeclass class / instance は ordinary core dictionary term に elaboration され、search trace は certificate に入らない。
- quotient_v1 は feature flag と checker support を持ち、Setoid quotient / Quotient.lift examples が source-free checker で通る。
- SMT certificate は solver result だけで成功せず、reconstructed NPA proof term を kernel / checker が検査する。
- natural language formalization は formal statement hash と intent certificate を proof certificate から分離する。
- Phase 9 Human の heavy checks は AI candidate hot path を遅くしない位置にある。
- `./scripts/phase9-regression.sh` が Phase 9 completion gate として通る。
```

---

## 5. MVP では入れないもの

```text
- AI confidence を proof acceptance に使うこと
- theorem graph score を checker verdict に使うこと
- SMT solver process を kernel / checker 内で起動すること
- typeclass search を kernel / checker に入れること
- natural language text を proof certificate hash に入れること
- production LLM / RAG / online graph store 運用
- full nonlinear arithmetic / arrays / bitvectors / quantifiers の SMT success
- unrestricted generic nested inductive positivity
- quotient を custom axiom として silently allow すること
```
