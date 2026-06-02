# Phase 2 implementation TODO

この TODO は `develop/phase2.md` を正とし、現在の `crates/npa-cert` 実装との差分を
実装タスクに分解したものです。

調査時点の実装は、既存の Phase 2 certificate verifier として次をかなり満たしています。

```text
- CoreModule から ModuleCert を生成する
- canonical binary encode/decode を行う
- decode 後の再encode一致で canonical bytes を確認する
- name / level / term table の canonical order と reachability を確認する
- import / declaration / export block / axiom report の canonical order を確認する
- level / term / declaration / export / axiom report / module certificate hash を再計算する
- normal / high-trust import policy を検査する
- verified import store から kernel environment を再構成する
- axiom report と axiom policy を保存値ではなく再計算結果から検査する
- inductive constructor / recursor export と generated artifact mismatch を検査する
- Phase 1 Rust kernel に decode 済み declaration を渡して再検査する
```

主な未実装範囲は、`develop/phase2.md` の 7.1 / 11.1.1 / 12.7 で追加された
Human producer / AI producer 分離、AI candidate fast path、checked token、
producer public environment fingerprint、producer separation test です。
加えて、新しい hash payload contract と現在実装の細かいズレもあります。

---

## 0. 実装済みと未実装の境界

### 0.1 実装済みとして扱うもの

以下は今回の producer separation 実装の前提として使ってよいです。

```text
crates/npa-cert/src/lib.rs
- build_module_cert
- encode_module_cert
- decode_module_cert
- verify_module_cert
- term_hash

crates/npa-cert/src/types.rs
- CoreModule
- ModuleCert
- VerifiedModule
- VerifierSession
- AxiomPolicy / TrustMode
- ImportEntry / ImportKey
- DeclCert / DeclPayload
- GlobalRef
- DependencyEntry / AxiomRef
- ExportBlock / AxiomReport

crates/npa-cert/src/canonical.rs
- build_module_cert_impl
- canonical declaration ordering
- import sorting / exact import dedup
- name / level / term table construction
- dependency and axiom dependency construction

crates/npa-cert/src/verify.rs
- canonical round-trip check
- table order / reachability / bvar scope checks
- hash recomputation
- import resolution
- axiom policy enforcement
- kernel recheck
```

### 0.2 実装が存在しないもの

以下は現在のコードに型・API・テストがありません。

```text
ProducerProfile
ProducerLimits
CoreDeclCandidate
CandidateBatch
CheckedDeclCandidate
CandidateHashPreview
CandidateStatus
CandidateBatchResult
check_core_decl_candidates
build_module_cert_from_checked_candidates
ProducerImportEnvKey
ProducerCheckedDeclInterface
ProducerLookupEnv
ProducerPriorChainEntry
producer_limits_hash
stricter_or_equal
producer_env_fingerprint
prior_chain_fingerprint
DuplicateImportEnvKey
```

---

## 1. Hash payload contract のズレを直す

### P2-01: Def の `decl_interface_hash` payload order を `phase2.md` に合わせる

現状:

```text
crates/npa-cert/src/hash.rs の decl_interface_payload(Def):
  kind, name, universe_params, type_hash,
  value_hash if reducible,
  reducibility,
  public_dependency_entries,
  axiom_dependencies
```

`develop/phase2.md` の要求:

```text
Def:
  kind, name, universe_params, type_hash, reducibility,
  public_dependency_entries, axiom_dependencies,
  value_hash only when reducibility = reducible
```

実装タスク:

- `decl_interface_payload` の Def branch を、仕様の field order に変更する。
- `value_hash` は reducible def のときだけ含める方針を維持する。
- 変更後、golden hash fixture を更新する。
- 既存の hash role tests が新 payload order で通ることを確認する。

影響ファイル:

```text
crates/npa-cert/src/hash.rs
crates/npa-cert/src/tests.rs
crates/npa-cert/tests/fixtures/golden_hashes.tsv
```

完了条件:

```text
- Def interface hash の canonical payload order が phase2.md 11.11 と一致する
- binder name stability / transparent body change / opaque body change の既存テストが通る
- golden fixture が新 hash に更新されている
```

### P2-02: Def の `decl_certificate_hash` に常に `value_hash` を入れる

現状:

```text
crates/npa-cert/src/hash.rs の decl_certificate_payload:
- opaque def は value_hash を含める
- reducible def は value_hash を直接含めず、decl_interface_hash 経由でだけ反映される
```

`develop/phase2.md` の要求:

```text
DeclCertificatePayload.Def:
  decl_interface_hash, value_hash, dependency entries, axiom_dependencies
```

実装タスク:

- `decl_certificate_payload` の Def branch を reducible / opaque で分岐させず、
  すべての Def で `value_hash` を明示的に encode する。
- `decl_interface_hash` 経由で value が入る reducible def でも、payload contract として
  `value_hash` を重複して入れる。
- `rehash_cert_after_decl_change` helper と fixture 更新が必要になる。

影響ファイル:

```text
crates/npa-cert/src/hash.rs
crates/npa-cert/src/tests.rs
crates/npa-cert/tests/fixtures/golden_hashes.tsv
```

完了条件:

```text
- transparent def / opaque def のどちらも decl_certificate_hash payload に value_hash が入る
- transparent def body change で decl_certificate_hash / certificate_hash が変わる
- opaque def body change で decl_certificate_hash / certificate_hash が変わり、export_hash は維持される
```

### P2-03: Inductive interface payload の generated artifact hash を明示化する

現状:

```text
crates/npa-cert/src/hash.rs の decl_interface_payload(Inductive):
  constructors の name/type hash と recursor の name/universe_params/type/rules を直接 encode している
```

`develop/phase2.md` の要求:

```text
Inductive:
  kind, name, universe_params, params, indices, sort,
  constructors,
  generated recursor signature hash,
  generated computation rule hash,
  public_dependency_entries,
  axiom_dependencies
```

実装タスク:

- `generated_recursor_signature_hash` と `generated_computation_rule_hash` の canonical payload を定義する。
- 現在直接 encode している recursor type / rules を、上記 hash payload に切り出す。
- constructor 側も、仕様上 `constructors` に直接入れる範囲と、generated artifact hash に入れる範囲を
  コード上で明確に分ける。
- `verify_inductive_generated_artifacts` の再生成ロジックと同じ canonical bytes を hash 計算に使う。

影響ファイル:

```text
crates/npa-cert/src/hash.rs
crates/npa-cert/src/verify.rs
crates/npa-cert/src/tests.rs
crates/npa-cert/tests/fixtures/golden_hashes.tsv
```

完了条件:

```text
- Inductive decl_interface_hash payload が phase2.md 11.11 の field 名と一致する
- recursor type / rule の tamper tests が引き続き失敗する
- generated recursor signature / computation rule hash の安定性テストがある
```

---

## 2. Producer API の型を追加する

### P2-04: `crates/npa-cert` に producer module を追加する

現状:

```text
crates/npa-cert は certificate build / verify API だけを公開している。
AI candidate fast path 用の型・関数はない。
```

実装タスク:

- `crates/npa-cert/src/producer.rs` を追加する。
- `lib.rs` から `mod producer;` と必要な public 型・関数を re-export する。
- producer 型は trusted certificate payload に encode しない。
- `ProducerProfile` は sidecar / audit 用だけにし、`build_module_cert` / `verify_module_cert`
  の引数に入れない。

追加する型:

```rust
pub enum ProducerProfile {
    HumanSurface,
    AiCoreMvp,
}

pub struct ProducerLimits {
    pub max_declarations: u32,
    pub max_expr_nodes: u32,
    pub max_level_nodes: u32,
    pub max_name_components: u32,
    pub max_reduction_steps: u64,
    pub max_conversion_steps: u64,
}

pub struct CoreDeclCandidate {
    pub declaration: npa_kernel::Decl,
}

pub struct CandidateBatch<'a> {
    pub imports: &'a [VerifiedModule],
    pub prior_current_decls: &'a [CheckedDeclCandidate],
    pub candidates: Vec<CoreDeclCandidate>,
    pub limits: ProducerLimits,
}

pub struct CandidateHashPreview {
    pub type_hash: Option<Hash>,
    pub body_hash: Option<Hash>,
    pub decl_interface_hash: Option<Hash>,
    pub decl_certificate_hash: Option<Hash>,
}

pub enum CandidateStatus {
    Accepted(CheckedDeclCandidate),
    Rejected(CertError),
}

pub struct CandidateBatchResult {
    pub statuses: Vec<CandidateStatus>,
}
```

完了条件:

```text
- producer API 型が npa-cert public API から使える
- ProducerProfile は certificate build / verify path に混ざらない
- cargo doc / clippy の missing docs に通る
```

### P2-05: `CheckedDeclCandidate` を opaque token として実装する

現状:

```text
CheckedDeclCandidate 型がない。
任意の caller が raw npa_kernel::Decl を token として扱うことを防ぐ境界もない。
```

実装タスク:

- `CheckedDeclCandidate` の fields を private にする。
- constructor は `check_core_decl_candidates` 内部だけに置く。
- public getter で raw declaration を取り出せるようにしない。
- diagnostic 用 getter を作る場合は、preview hash / interface hash など non-authoritative な値に限定する。

内部 fields:

```rust
declaration: npa_kernel::Decl,
preview_hashes: CandidateHashPreview,
pre_env_fingerprint: Hash,
post_env_fingerprint: Hash,
prior_chain_fingerprint: Hash,
limits: ProducerLimits,
limit_profile_hash: Hash,
decl_interface_hash: Hash,
decl_certificate_hash: Hash,
```

完了条件:

```text
- 外部 crate から CheckedDeclCandidate を直接 construct できない
- token から raw CoreModule を caller 側で組めない
- build_module_cert_from_checked_candidates だけが token 内部の declaration を使える
```

---

## 3. Producer limits と deterministic resource check

### P2-06: `ProducerLimits` canonical bytes / hash を実装する

現状:

```text
ProducerLimits 型も producer_limits_hash もない。
kernel 側には固定 fuel と metered API があるが、candidate fast path から limit profile として使われていない。
```

実装タスク:

- `ProducerLimits` の canonical encode を struct field order 固定で実装する。
- 各 field は minimal ULEB128 で encode する。
- `producer_limits_hash(limits)` を実装する。
- domain separator は `"NPA-PRODUCER-LIMITS-0.1"` に固定する。
- `stricter_or_equal(a, b)` を実装する。

完了条件:

```text
- 同じ limits は同じ producer_limits_hash になる
- field order 変更で hash が変わることがテストされる
- stricter_or_equal は全 field の <= で判定される
```

### P2-07: candidate precheck に deterministic limits を適用する

現状:

```text
npa-kernel::Env には check_with_fuel_metered / infer_with_fuel_metered などがある。
一方、npa-cert の add_decl_to_env / build_module_cert は通常の kernel check を使い、
ProducerLimits を受け取らない。
```

実装タスク:

- producer fast path 専用の declaration precheck 関数を作る。
- `max_reduction_steps` / `max_conversion_steps` を kernel の WHNF / conversion fuel に対応させる。
- `max_declarations` / `max_expr_nodes` / `max_level_nodes` / `max_name_components` を candidate schema validation で検査する。
- limit 超過は per-candidate `Rejected(CertError)` にするか、batch schema 全体に関わる場合は
  batch-level `Err(CertError)` にする。分類をテストで固定する。

完了条件:

```text
- limit profile が token に保存される
- token 作成時 limits と batch.limits の strictness 判定ができる
- limit 超過時の error が deterministic
```

---

## 4. Producer public environment fingerprint

### P2-08: `ProducerImportEnvKey` と import order validation を実装する

現状:

```text
build_module_cert は imports を (module, export_hash, Some(certificate_hash)) で sort / exact dedup する。
CandidateBatch.imports の canonical order 検査や ProducerImportEnvKey(module, export_hash) 重複拒否はない。
CertError::DuplicateImportEnvKey もない。
```

実装タスク:

- `CertError::DuplicateImportEnvKey { module: ModuleName, export_hash: Hash }` を追加する。
- `CandidateBatch.imports` が `ModuleCert.Imports` と同じ canonical import order であることを検査する。
- `ProducerImportEnvKey(module, export_hash)` の重複を batch-level error で拒否する。
- 同じ module / export_hash で certificate_hash だけ異なる import を candidate fast path に同時投入できないようにする。

完了条件:

```text
- noncanonical CandidateBatch.imports は Err(CertError::NonCanonicalEncoding { object: "Imports" })
- duplicate ProducerImportEnvKey は Err(CertError::DuplicateImportEnvKey)
- GlobalRef::Imported(import_index, ...) と batch.imports[import_index] の対応が崩れない
```

### P2-09: `ProducerEnvFingerprintBytes` を実装する

現状:

```text
producer_env_fingerprint はない。
import の public environment 再利用単位は certificate 実装内部の VerifiedModule / ExportBlock に閉じている。
```

実装タスク:

- `ProducerImportEnvKey { module, export_hash }` を定義する。
- `ProducerCheckedDeclInterface { decl_interface_hash, axiom_dependencies }` を定義する。
- `ProducerEnvFingerprintBytes` を fixed record order で canonical encode する。
- `producer_env_fingerprint(env)` を実装する。
- domain separator は `"NPA-PRODUCER-ENV-0.1"` に固定する。

完了条件:

```text
- import 先の proof body だけが変わり module/export_hash が同じなら fingerprint は維持される
- checked_decls の順序変更で fingerprint は変わる
- axiom_dependencies の順序は canonical order に正規化される
```

### P2-10: `ProducerLookupEnv` と axiom dependency recomputation を実装する

現状:

```text
certificate generation は imported ExportBlock と prior declaration から axiom deps を計算している。
producer fast path 用に、fingerprint bytes と lookup environment を分けた API はない。
```

実装タスク:

- `ProducerLookupEnv` を追加する。
- `canonical_import_env_keys(imports)` と `canonical_import_export_views(imports)` が同じ import order を保存するようにする。
- `producer_checked_decl_interface(decl, lookup_env)` を実装する。
- axiom dependencies は AI producer の report や preview hash ではなく、既存 certificate generation と同じ規則で再計算する。
- `ProducerImportEnvKey(module, export_hash)` だけから import 内 axiom deps を lookup しないようにする。

完了条件:

```text
- canonical_import_env_keys と canonical_import_export_views の index が一致する
- GlobalRef::Imported(import_index, ...) が imports / direct_imports / import_exports の同じ import を指す
- imported axiom deps は VerifiedModule.export_block 由来の lookup view から再計算される
```

### P2-11: `post_env_fingerprint` を full recompute で実装する

現状:

```text
incremental producer env fingerprint はない。
```

実装タスク:

- `initial_env_fingerprint(imports)` を実装する。
- `post_env_fingerprint(imports, checked_decls_before, decl)` を実装する。
- 前回 fingerprint に追記するのではなく、imports と checked declaration interface sequence 全体から再計算する。
- incremental cache の有無が fingerprint に影響しないようにする。

完了条件:

```text
- cache enabled / disabled で same accepted module の fingerprint が一致する
- 同じ checked_decls_before と decl から同じ post_env_fingerprint が得られる
```

---

## 5. Prior chain fingerprint と token validation

### P2-12: `ProducerPriorChainEntry` / `prior_chain_fingerprint` を実装する

現状:

```text
current module 内の checked declarations の exact token sequence を固定する hash がない。
```

実装タスク:

- `ProducerPriorChainEntry` を定義する。
- `ProducerPriorChainBytes` を canonical encode する。
- `prior_chain_fingerprint(chain)` を実装する。
- domain separator は `"NPA-PRODUCER-CHAIN-0.1"` に固定する。

Entry fields:

```text
decl_interface_hash
decl_certificate_hash
pre_env_fingerprint
post_env_fingerprint
```

完了条件:

```text
- 空 chain fingerprint が deterministic
- declaration order の変更で prior_chain_fingerprint が変わる
- opaque theorem proof / opaque def body の body-only change で producer public env は維持され、
  prior chain は decl_certificate_hash の差で変わる
```

### P2-13: `prior_current_decls` token validation を実装する

現状:

```text
CandidateBatch.prior_current_decls がない。
過去に検査済みの current module declaration token を batch に安全に再利用する検査もない。
```

実装タスク:

- 最初の token の `pre_env_fingerprint` が initial env fingerprint と一致することを検査する。
- 2個目以降の token の `pre_env_fingerprint` が直前 token の `post_env_fingerprint` と一致することを検査する。
- token の `prior_chain_fingerprint` が、それ以前の accepted prior declarations の chain と一致することを検査する。
- private `decl_interface_hash` / `decl_certificate_hash` を declaration から再計算して照合する。
- `producer_limits_hash(token.limits) == token.limit_profile_hash` を検査する。
- token 作成時 limits が現在 batch limits と同一または stricter_or_equal であることを検査する。
- token の `post_env_fingerprint` を再計算して照合する。

完了条件:

```text
- invalid prior token は per-candidate rejection ではなく batch-level Err(CertError)
- forged token / mismatched chain / mismatched env fingerprint は拒否される
- stricter prior token は再利用でき、looser prior token は拒否される
```

---

## 6. Candidate fast path 本体

### P2-14: `check_core_decl_candidates` を実装する

現状:

```text
候補ごとに certificate を作らずに schema validation / import ref validation / kernel precheck を行う API がない。
```

実装タスク:

- `CandidateBatch` 全体の schema を検査する。
- `imports` の canonical order / duplicate ProducerImportEnvKey を検査する。
- `prior_current_decls` を P2-13 の規則で検証し、環境に追加する。
- `candidates` を input order のまま順に検査する。
- 各 candidate に対して:
  - unresolved metavariable / placeholder 相当が表現されていないことを検査する。
  - name / level / expr node の size limit を検査する。
  - import GlobalRef / local ref / generated ref / builtin ref の解決を検査する。
  - kernel precheck を実行する。
  - dependency / axiom dependency / decl hashes を再計算する。
  - `CheckedDeclCandidate` を作る。
- 失敗した candidate は `Rejected(CertError)` として返す。
- batch-level failure と per-candidate failure の境界をテストで固定する。

完了条件:

```text
- Ok(result) の statuses.len() == candidates.len()
- statuses[i] は candidates[i] の結果で、score/hash/cache 状態で並べ替えない
- Accepted は VerifiedModule として扱えない
- .npcert bytes / certificate_hash はこの API では作らない
```

### P2-15: name-based `npa_kernel::Decl` と hash-bound `GlobalRef` の境界を固定する

現状:

```text
npa_kernel::Expr::Const は name string + levels だけを持つ。
certificate 内では GlobalRef::{Imported, Local, LocalGenerated, Builtin} に解決される。
AI producer MVP のテキストでは pretty-only / fully-qualified name だけを certificate-facing core term として扱わないことを要求している。
```

実装タスク:

- `CoreDeclCandidate { declaration: npa_kernel::Decl }` を維持する場合、
  candidate validation 内で必ず certificate resolver を通して `GlobalRef` に固定する。
- AI producer が渡す preview hash / dependency report / score から `GlobalRef` を補完しない。
- import / prior declaration / builtin のどれにも解決できない name は `Rejected(CertError::UnknownDependency)` にする。
- 必要なら内部用に resolved candidate representation を追加する。ただし public certificate schema には入れない。

完了条件:

```text
- pretty-only name は Accepted token の根拠にならない
- Accepted token 内の private decl hashes は resolved GlobalRef payload から再計算される
- import decl_interface_hash mismatch は拒否される
```

### P2-16: `build_module_cert_from_checked_candidates` を実装する

現状:

```text
Accepted token から最終 ModuleCert を作る API がない。
```

実装タスク:

- `imports` と `checked_decls` の token chain を再検証する。
- `pre_env_fingerprint` / `post_env_fingerprint` / `prior_chain_fingerprint` を再計算して照合する。
- `producer_limits_hash(token.limits) == token.limit_profile_hash` を照合する。
- private `decl_interface_hash` / `decl_certificate_hash` を再計算して照合する。
- chain が完全に一致する場合だけ内部で `CoreModule` を構成する。
- 構成した `CoreModule` を既存 `build_module_cert` に渡す。
- この API では新しい `ProducerLimits` との strictness 判定をしない。

完了条件:

```text
- token chain mismatch を拒否する
- token order mismatch を拒否する
- forged token を拒否する
- 生成された ModuleCert は verify_module_cert を通すまで trusted import store に入らない
```

---

## 7. Producer sidecar と trusted payload 分離

### P2-17: producer sidecar は certificate payload から分離したままにする

現状:

```text
Phase 2 producer sidecar 型はない。
certificate payload には source map / diagnostics / AI trace は入っていない。
```

実装タスク:

- 必要なら `ProducerSidecar` を `npa-cert` ではなく上位 crate か別 artifact として定義する。
- `ProducerProfile` / model name / prompt / score / diagnostics / cache hit は `.npcert` に encode しない。
- sidecar を作る場合も `export_hash` / `axiom_report_hash` / `certificate_hash` の入力にしない。

完了条件:

```text
- sidecar の追加・削除・内容変更で .npcert bytes と各 hash が変わらない
- verify_module_cert は sidecar なしで通る
```

---

## 8. Tests

### P2-18: hash payload contract tests を更新する

実装タスク:

- P2-01 / P2-02 / P2-03 に合わせて golden fixture を更新する。
- `decl_interface_hash(def)` payload order の regression test を追加する。
- reducible def の `decl_certificate_hash` が `value_hash` を直接含むことを mutation test で固定する。
- generated recursor signature / computation rule hash の mutation test を追加する。

完了条件:

```text
- cargo test -p npa-cert golden_certificate_hashes_cover_core_shapes
- cargo test -p npa-cert hash role tests
- cargo test -p npa-cert inductive generated artifact tests
```

### P2-19: producer separation tests を追加する

`develop/phase2.md` 12.7 の項目を、そのまま test checklist として実装します。

必須テスト:

```text
- Human producer 由来 CoreModule と AI producer 由来 CoreModule が同じ core declaration を表す場合、
  .npcert bytes と各 hash が一致する
- producer_profile / producer_run_id / model name / score / diagnostics を sidecar で変えても、
  .npcert bytes と各 hash が変わらない
- check_core_decl_candidates の Accepted をそのまま VerifiedModule として扱えない
- Accepted candidate から build_module_cert した .npcert は verify_module_cert を通すまで trusted import store に入らない
- invalid prior token は per-candidate rejection ではなく batch-level Err(CertError) になる
- CandidateBatch.imports が canonical import order でない場合は batch-level Err(CertError::NonCanonicalEncoding)
- duplicate ProducerImportEnvKey は Err(CertError::DuplicateImportEnvKey)
- CandidateBatchResult.statuses は input candidates と同じ長さ・同じ順序
- build_module_cert_from_checked_candidates は token chain / pre_env_fingerprint / post_env_fingerprint mismatch を拒否
- build_module_cert_from_checked_candidates は token の producer_limits_hash(token.limits) mismatch を拒否
- producer public env / prior chain fingerprint が canonical bytes と domain separator から deterministic に再計算できる
- canonical_import_env_keys と canonical_import_export_views は同じ順序を保持する
- import 先 proof 本体だけが変わり module/export_hash が同じ場合、producer public env fingerprint は維持される
- producer public env fingerprint の axiom dependencies は compute_axiom_deps と同じ規則で再計算する
- opaque theorem proof / opaque def body だけが変わり、公開 interface と axiom dependencies が同じ場合、
  producer public env fingerprint は維持され、prior chain fingerprint は decl_certificate_hash の差で変わる
- ProducerLimits の canonical hash と stricter_or_equal 判定が deterministic
- preview hash が誤っていても token validation / build_module_cert / verify_module_cert は再計算結果だけを採用する
- AI producer 由来 candidate に unresolved metavariable / placeholder / pretty-only GlobalRef がある場合は拒否する
- batch 内で1候補が失敗しても、他候補の結果が失敗順序や cache 状態に依存しない
- cache を有効/無効にしても、同じ accepted module から同じ .npcert bytes が得られる
```

影響ファイル:

```text
crates/npa-cert/src/tests.rs
```

### P2-20: public API compile tests / visibility tests を追加する

実装タスク:

- 外部 crate から `CheckedDeclCandidate` の private fields にアクセスできないことを compile-time に守る。
- `ProducerProfile` が `build_module_cert` / `verify_module_cert` に混ざらないことを API shape として確認する。
- `Accepted(CheckedDeclCandidate)` と `VerifiedModule` が型で混同できないことをテストする。

完了条件:

```text
- cargo check --workspace
- cargo clippy --workspace --all-targets -- -D warnings
```

---

## 9. Integration / rollout

### P2-21: `npa-cert` 内部 helper を producer から再利用できる粒度に整理する

現状:

```text
canonicalize_decl / resolver / dependency construction は canonical.rs の private helper に閉じている。
producer fast path では同じ規則を使う必要がある。
```

実装タスク:

- `canonicalize_decl`、dependency collection、axiom dependency calculation、decl hash calculation を
  producer path から再利用できる internal API に整理する。
- trusted certificate path と producer fast path で hash / dependency / axiom dependency の規則を二重実装しない。
- public API には出さず `pub(crate)` のまま、module boundary を整理する。

完了条件:

```text
- build_module_cert と check_core_decl_candidates が同じ hash/dependency implementation を使う
- 片方だけ更新される drift が起きにくい
```

### P2-22: frontend / API crate への接続は別タスクに分離する

現状:

```text
crates/npa-frontend は human source から CoreModule を作り、build_module_cert / verify_module_cert を呼ぶ。
crates/npa-api の Phase 4/5/7 は tactic/search candidate を持つが、Phase 2 CoreDeclCandidate fast path には接続されていない。
```

実装タスク:

- Phase 2 producer fast path を `npa-cert` に入れたあと、上位 crate で使うかを別設計にする。
- 既存 Phase 4/5/7 の MachineTacticCandidate と Phase 2 `CoreDeclCandidate` を混同しない。
- API 接続時は、最終的に `.npcert` を build/verify する境界を維持する。

完了条件:

```text
- Phase 2 の npa-cert API が単体で完成している
- 上位 crate への接続は trusted payload を広げずに行える
```

---

## 10. Suggested implementation order

実装順は次が安全です。

```text
1. P2-01 / P2-02 / P2-03
   hash payload contract を先に phase2.md と一致させる。

2. P2-04 / P2-05 / P2-06
   producer API 型、opaque token、limits hash を追加する。

3. P2-08 / P2-09 / P2-10 / P2-11 / P2-12
   producer public env fingerprint と prior chain fingerprint を実装する。

4. P2-13 / P2-14 / P2-15
   token validation と candidate fast path 本体を実装する。

5. P2-16
   accepted token から ModuleCert を作る補助 API を実装する。

6. P2-17 / P2-18 / P2-19 / P2-20
   sidecar 分離とテストを固める。

7. P2-21 / P2-22
   internal helper 整理と上位 crate 接続の準備を行う。
```

---

## 11. Done definition

この TODO 全体の完了条件です。

```text
- develop/phase2.md 11.1.1 の producer API が crates/npa-cert に実装されている
- CheckedDeclCandidate が opaque token として実装されている
- ProducerLimits canonical hash と strictness 判定が実装されている
- producer public env fingerprint と prior chain fingerprint が canonical bytes から再計算できる
- check_core_decl_candidates が batch-level failure と per-candidate status を仕様通り返す
- build_module_cert_from_checked_candidates が token chain を再検証してから ModuleCert を作る
- producer sidecar / metadata が .npcert bytes と hash に影響しない
- Def / Inductive の hash payload contract が develop/phase2.md と一致している
- develop/phase2.md 12.7 の producer separation tests が自動テストで通る
- cargo fmt --all
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace
```
