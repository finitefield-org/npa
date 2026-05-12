以下は **Phase 8 AI Profile: Checker Audit Automation** の詳細設計です。
Phase 8 Human Profile は `doc/phase8-human.md` で定義し、この文書はその上に乗る
AI / machine client 向けの実行・監査・差分整理プロファイルを定義します。

Phase 8 AI Profile の目的は、AI が `.npcert` の正しさを判断することではありません。
AI は checker を起動し、結果を正規化し、失敗を整理し、追加テストを提案できます。
しかし、証明の受理根拠は常に independent checker が出した deterministic result だけです。

対象はこの6つです。

```text
- machine check orchestration
- checker result normalization
- checker disagreement triage
- adversarial challenge generation
- CI / release audit summarization
- training and evaluation sidecars
```

大原則はこれです。

```text
AI は checker ではない。
AI は verdict を作らない。
AI は checker result を説明・分類・再実行するだけ。
accepted proof の根拠は canonical certificate と independent checker result だけ。
```

---

# 1. Phase 8 AI Profile の位置づけ

Phase 8 Human Profile の流れはこうです。

```text
certificate
  ↓
reference checker
  ↓
external checker
  ↓
CI / release audit
  ↓
verified_high_trust artifact
```

AI Profile はこの流れの横に置く sidecar です。

```text
certificate manifest
  ↓
machine check runner
  ↓
normalized checker results
  ↓
AI triage / summary / challenge proposal
  ↓
human-readable audit report
```

重要なのは、AI Profile が trust chain の中に入らないことです。

```text
trusted path:
  .npcert
    → reference checker
    → external checker
    → deterministic check result

untrusted sidecar:
  check result
    → AI summary
    → repair suggestion
    → challenge proposal
```

AI summary がどれだけ正確に見えても、`checked` status の代わりにはなりません。

---

# 2. 非信頼境界

Phase 8 AI Profile で信用しないもの：

```text
- LLM output
- AI agent plan
- AI generated challenge
- AI generated certificate
- AI generated proof script
- AI explanation of checker errors
- AI selected imports
- AI selected checker binary
- AI modified checker config
- AI generated CI summary
- training labels inferred by AI
```

Phase 8 AI Profile で信用できるもの：

```text
- canonical .npcert bytes
- checker binary identity selected by policy, not by AI
- checker result produced by that binary
- checker version / build hash recorded by CI
- export_hash / certificate_hash recomputed by checker
- axiom report recomputed by checker
```

AI が参照してよいが信用してはいけないもの：

```text
- .npa source
- tactic script
- AI search trace
- pretty printed goal
- source map
- theorem search index
- previous failure summary
- human-written PR comment
```

AI はこれらを説明や修正候補の材料にできます。
ただし checker request の正本には入れません。

---

# 3. AI Profile の入力と出力

## 3.1 入力

AI Profile の入力は、checker に渡す artifact と、AI sidecar が読む補助情報に分けます。

checker に渡す入力：

```text
- certificate bytes
- import certificates or trusted import store references
- explicit checker profile
- explicit trust mode
- deterministic budget
- axiom policy file
```

AI sidecar が読んでよい補助入力：

```text
- module name
- PR / commit metadata
- source file path
- previous check results
- failed declaration name
- source map, if available
- tactic / AI search trace, if available
```

この2つを混ぜてはいけません。
AI sidecar の補助入力が変わっても、checker result は変化してはいけません。

## 3.2 出力

Phase 8 AI Profile の出力は、保存 artifact、untrusted sidecar、transient response に分けます。

```text
saved artifact:
1. MachineCheckResult
   runner が checker raw result と process / policy metadata から生成する正本 envelope。

2. NormalizedCheckResult
   複数 checker の結果を比較しやすくする正規化表現。
   verdict は checker result から機械的に写すだけ。

3. MachineCheckRequestErrorResult
   request JSON を MachineCheckRequest として読み込めない場合の pipeline error artifact。
   checker verdict ではなく、checker は起動されない。

4. NormalizeErrorResult
   request store や policy 解決に失敗し、NormalizedCheckResult を作れない場合の error artifact。
   checker verdict ではなく pipeline error として扱う。

5. AuxiliaryResult
   CI pass condition に使う axiom policy / reproducibility / audit bundle などの deterministic 補助結果。

6. ChallengeReplayResult
   adversarial challenge replay の保存 summary。
   checker oracle は参照先 MachineCheckResult。

7. ChallengeCoverageSummary
   challenge manifest と replay result から作る deterministic coverage summary。
   checker verdict の代替ではなく release audit 用の補助 artifact。

8. AxiomReport
   checker が生成する used axiom の canonical report。
   MachineCheckResult には `axiom_report_hash` だけを写し、report 本体は別 artifact として保存する。
   `AxiomReport` は Phase 8 の saved artifact ですが、release audit bundle artifact kind には含めない。
   release bundle には `axiom_policy` の passed `AuxiliaryResult` だけを含め、report 本体は bundle 生成前の deterministic CI step の input として扱う。

untrusted sidecar:
9. AiAuditSidecar
   AI が生成する説明・分類・修正候補。
   verdict として扱ってはいけない。

transient response:
10. CompareValidationResult
   保存済み NormalizedCheckResult.comparison の integrity validation response。
   保存正本 artifact ではなく result_hash を持たない。

11. AuditSidecarValidationResult
   AiAuditSidecar の schema-only / cross-artifact validation response。
   保存正本 artifact ではなく result_hash を持たない。

12. NormalizationWriteResult
   normalize-results が `--out` 指定時に返す書き込み summary。
   保存正本 artifact ではなく result_hash を持たない。

13. ChallengeRequestMaterializationResult
   challenge replay request materialization の書き込み summary。
   保存正本 artifact ではなく result_hash を持たない。

14. ChallengeGenerationResult
   challenge generation が ChallengeManifest、mutated certificate、challenge output store を
   書き込んだ summary。
   保存正本 artifact ではなく result_hash を持たない。

15. ReleaseBundleStagingResult
   release bundle staging が staged artifact と bundle-local store manifest を
   書き込んだ summary。
   保存正本 artifact ではなく result_hash を持たない。

16. CommandError
   challenge generate / challenge materialize-requests / challenge replay /
   release stage-bundle-inputs などが
   成功 response または saved artifact を作れない場合の transient diagnostic。
   normalize-results では使わず、normalize pipeline / write-stage failure は
   `NormalizeErrorResult` で返す。
   保存正本 artifact ではなく result_hash を持たない。

17. ApiError
   machine API の wrapper object schema violation、workspace path validation failure、
   HTTP method / endpoint validation failure を表す transient transport diagnostic。
   endpoint 固有の artifact / response ではなく、result_hash を持たない。
```

`AiAuditSidecar` は `checked`、`accepted`、`verified` という status を持ってはいけません。
使える status は、AI の作業状態に限定します。

```text
- summarized
- triaged
- suggested_fix
- suggested_challenge
- inconclusive
```

## 3.3 Canonical serialization and hashes

この文書の `*_hash` は、特に断らない限り `sha256:<lower-hex>` 形式です。
hash 対象は schema ごとの canonical serialization です。

MVP の canonical serialization rule は RFC 8785 JSON Canonicalization Scheme を基準にし、
次の追加制約を置きます。

```text
- UTF-8 without BOM only
- JSON object keys are sorted by RFC 8785 ordering
- arrays preserve order
- strings use RFC 8785 escaping
- `/` is not escaped
- non-ASCII scalar values are emitted as UTF-8, not `\u` escapes
- surrogate code points are invalid
- control characters use the shortest RFC 8785 JSON escape
- integers are decimal without leading zeros
- floats are forbidden
- duplicate object keys are forbidden
- absent field and explicit null are different
- optional field absence is represented by omitting the field
- null is invalid unless the schema explicitly marks that field nullable
- unknown fields are invalid under each closed-world schema
- paths are workspace-relative, use `/`, and contain no `.` / `..` segment
- hashes over files use exact file bytes, not parsed JSON values
```

この文書の schema / domain validation で array / manifest entry が `昇順` または
`ascending` と書かれている場合、別途 `strict` と明記しない限り、
隣接する sort key が `previous <= current` であることを意味します。
隣接 sort key が等しいだけでは `order_violation` にせず、その key が unique 制約にも違反する場合は
対応する duplicate reason で報告します。
`order_violation` は、最初に `current < previous` となる後続 element の concrete 0-based index を
報告対象にします。

Unicode string values are treated as UTF-8 bytes after parsing.
Hashing code must not apply locale-dependent case folding or path normalization.
If a schema needs human text normalization, that schema must say so explicitly.
Path string identity and filesystem safety are separate rules.
Artifact hashes, manifest entries, and bundle paths use the bytewise path string that appears in
the JSON / CLI input; producers and validators must not rewrite that string by symlink resolution,
case folding, environment expansion, or platform path normalization.
When a workspace-relative path is used for actual file IO, the implementation resolves it exactly
once against the owning root:

```text
- normal CLI / API artifact paths: repository workspace root
- bundle-local ReleaseAuditBundleManifest artifact paths: bundle root
- checker binary registry paths: runner install root or repository workspace root, as configured in 4.2
```

For all workspace / bundle artifact reads and writes, every existing path component, including the
final path when it already exists, must resolve to a target inside that owning root.
A symlink that escapes the owning root is a path validation failure before file bytes are read or
written.  CLI / artifact validation reports this with the caller's `invalid_path` field shape;
machine API wrapper path validation reports it as `ApiError.reason_code = api_path_outside_workspace`.
If a path belongs to an inline command request object that has already passed API wrapper validation,
symlink escape is not reported as an `ApiError`; the command maps it to the same command-specific
unreadable / write failure reason that would be used for an inaccessible file at that field.
Checker binary registry path resolution applies the same inside-owning-root rule to executable
resolution, even though checker binaries are not artifact reads / writes; 4.2 defines the runner
pre-check error mapping for that case.
This safety check does not change the stored path string and is not a hash input.
Phase 8 AI Profile の JSON artifact hash は、特にその schema が別の domain separation
string を定義しない限り、canonical serialization bytes そのものの SHA-256 です。
`NPA-TERM-0.1`、`NPA-MODULE-CERT-0.1` などの domain-separated binary certificate hash とは
同じ `sha256:<lower-hex>` 形式でも別 namespace として扱います。
JSON artifact hash を certificate hash、term hash、decl hash の代替として比較してはいけません。
canonical schema を読むすべての JSON loader は duplicate-aware でなければなりません。
object を map に変換して duplicate key を破棄する parser、last-write-wins parser、
first-write-wins parser は file loader でも禁止です。
non-nullable field の explicit null は missing や wrong type として扱わず、
schema / canonical decode failure の `actual_value = "null_not_allowed"` として返します。
required field の explicit null は dedicated `*_missing` reason code ではなく、
対応する `*_schema_invalid` または `api_request_schema_invalid` に分類します。
UTF-8 / JSON syntax として不正な入力だけを `*_json_invalid` にし、
duplicate object key は JSON parse failure ではなく schema / canonical decode failure の
`duplicate_field` として返します。
duplicate object key の `expected_value` は、別途 field shape が明記されていない限り
`unique_object_keys` です。
duplicate object key の `error.field` は、同じ object 内で同名 key が2回目以降に出現した
後続 member の JSON path です。
duplicate object key と、同じ logical field の value schema failure が同時に成立する場合は、
値の内容を検査せず duplicate object key の `duplicate_field` を報告します。
同じ object 内に複数種類の duplicate key がある場合の tie-break は、artifact ごとの
validation order が明記する field order または bytewise field name order に従います。
たとえば audit sidecar file 内の duplicate key は `sidecar_json_invalid` ではなく
`sidecar_schema_invalid`、input policy file 内なら `input_policy_schema_invalid`、
result / normalized store manifest 内なら各 `*_manifest_invalid` です。
API request body の duplicate key は machine API の `ApiError` 規則に従います。
top-level `schema` mismatch の `actual_value` は artifact ごとの field shape に従います。
この文書で `actual_value = "invalid_enum"` と明記する artifact では固定値 `invalid_enum` を使い、
入力の `schema` 文字列を copy すると明記する artifact ではその文字列を使います。
producer / validator は一方の convention を別 artifact に暗黙適用してはいけません。

Phase 8 JSON artifact 内で Phase 2 の `Name` / `ModuleName` / `AxiomName` を表す field は、
schema boundary では dotted UTF-8 string として表します。
JSON decode 後の string を `.` で分割して Phase 2 `Name` component list に変換し、
空 component、先頭 / 末尾の `.`, 連続する `.`, JSON decode 後の空 string を拒否します。
Unicode normalization は行わず、JSON decode 後の UTF-8 byte sequence をそのまま canonical name bytes に使います。
この規則は少なくとも次の Phase 8 artifact / selector field に適用します。

```text
- MachineCheckRequest.module
- ImportLockManifest.imports[].module
- CheckerRawResult.module
- CheckerRawResult.error.declaration, if present
- MachineCheckResult.module
- MachineCheckResult.error.declaration, if present
- NormalizedCheckResult.artifact.module
- NormalizedCheckResult.results[].error.declaration, if present
- NormalizedCheckResult.results[].failure_key.declaration, if present
- artifact_selector.module
- ChallengeGenerationRequest.module
- ChallengeManifest.module
- ChallengeGenerationRequest.mutation.target / ChallengeManifest.mutation.target,
  when the target rule is declaration target, import target, or axiom target
- AxiomReport.module
- AxiomReport.axioms[].name
```

MVP known mutation kind で Phase 2 `Name` として decode しない `mutation.target` は、
whole certificate target の `"$whole_certificate"` だけです。
informational `ChallengeManifest.mutation.kind` の `mutation.target` は 10 の opaque
informational target label grammar に従い、Phase 2 `Name` として decode しません。
`ChallengeGenerationRequest` は informational kind を受け付けないため、この例外は
保存済み / 読み込み済みの `ChallengeManifest` manifest-local validation だけに適用します。
name grammar violation は schema / domain failure とし、`actual_value = "invalid_name_format"` を使います。
module / name の deterministic order は、別途 file path や raw string order と明記していない限り、
decoded Phase 2 `Name` の canonical name order です。

`request_hash` は `request_id` と `request_hash` field を除いた
`MachineCheckRequest` の canonical hash です。
MVP の保存済み `MachineCheckRequest` は `request_hash` field を required にします。
request generator はまず `request_id` と `request_hash` を omit した object から
`request_hash` を計算し、その値を object に書き込みます。
runner と request store loader は同じ規則で再計算し、保存値と一致しない request を拒否します。
`request_hash` は semantic request identity であり、保存 file bytes の hash ではありません。
request store manifest の `file_hash` は `request_id` と書き込まれた `request_hash` を含む
exact file bytes の sha256 です。
したがって `request_id` だけを変えた場合、`request_hash` は変わらず `file_hash` は変わります。
`result_hash` は deterministic runner-envelope verdict の hash です。
MVP では、runner が policy と照合した後に正本として保存する次の field を
hash 対象に含めます。

```text
- schema
- policy
- runner
- checker
- status
- module
- error, if present
- certificate_hash, if present
- export_hash, if present
- axiom_report_hash, if present
```

同じ checker raw result でも、policy hash、runner build hash、checker binary hash が違えば
別の `result_hash` になります。

`result_hash` から除外する field は次です。

```text
- request_id
- result_id
- request_hash
- result_hash
- run_artifact_hash
- checker.version
- attempt
- process
- resource_usage
- diagnostics
- axioms_used
- declarations_checked
```

`axioms_used` と `declarations_checked` は summary / instrumentation metadata です。
axiom list の正本性は `axiom_report_hash` と別途保存される axiom report artifact で検査します。
MVP の `AxiomReport` artifact schema：

```json
{
  "schema": "npa.phase8.axiom_report.v1",
  "axiom_report_hash": "sha256:...",
  "module": "Std.Nat",
  "certificate_hash": "sha256:...",
  "axioms": [
    { "name": "Classical.choice" }
  ]
}
```

`AxiomReport` は closed-world JSON object です。
top-level required field は `schema`、`axiom_report_hash`、`module`、
`certificate_hash`、`axioms` です。
`schema` が `npa.phase8.axiom_report.v1` 以外の string の場合は
`actual_value = "invalid_enum"` の schema failure とします。
`axiom_report_hash` は `axiom_report_hash` field 自身を除いた `AxiomReport`
object の canonical serialization hash です。
`module` は canonical module name、`certificate_hash` は checked certificate の
canonical certificate hash です。
`module` と `axioms[].name` は 3.3 の Phase 8 name JSON representation に従います。
`module` は Phase 2 の `ModuleName`、`axioms[].name` は Phase 2 の `AxiomName` として扱います。
grammar violation は schema / domain failure とし、`actual_value = "invalid_name_format"` を使います。
この schema 節で書く field は `AxiomReport` root からの local field path であり、
standalone loader では `field = "module"` または `field = "axioms[<i>].name"` を使います。
`AuxiliaryResult.error` へ写す場合は `field = "axiom_report.module"` または
`field = "axiom_report.axioms[<i>].name"` のように `axiom_report.` prefix を付けます。
`module` と `certificate_hash` は metadata ではなく、`axiom_policy` oracle が
selector で選んだ `NormalizedCheckResult` / result entry と照合する binding field です。
`axioms` は used axiom set であり、重複を禁止します。
各 entry は closed-world object で、MVP では `name` だけを required にします。
`axioms` は decoded Phase 2 `AxiomName` の canonical name order で昇順に並べます。
`AxiomReport` loader は duplicate key、unknown field、wrong type、explicit null、
invalid hash format、invalid name format、order violation、duplicate axiom name を
schema / domain failure として拒否します。
duplicate axiom name の `actual_value` は `duplicate_axiom_name` です。
AxiomReport validation は schema failure を domain failure より先に報告します。
複数の schema failure が同時に存在する場合は、top-level non-object、`schema`、
`axiom_report_hash`、`module`、`certificate_hash`、`axioms` array、
`axioms[]` entry object by smaller index、`axioms[].name` by smaller index、
その後 unknown field の順で最初の1件だけを返します。
known field の duplicate object key は、その field の schema order 位置で
`expected_value = "unique_object_keys"`、`actual_value = "duplicate_field"` として報告します。
unknown field の duplicate object key は unknown field の位置で
`expected_value = "unique_object_keys"`、`actual_value = "duplicate_field"` を返し、
`field` は重複した後続 unknown field の AxiomReport-local JSON path にします。
unknown field が複数ある場合は top-level object、次に `axioms[]` entry の小さい index の順で
object を選び、同じ object 内では field name の bytewise lexicographic order で最初の field を返します。
複数の domain failure が同時に存在する場合は、`module` name grammar、
`axioms[].name` grammar by smaller index、`axioms` order violation、
duplicate axiom name の順で最初の1件だけを返します。
`axioms` order violation の field は `axioms[<i>]` で、
`<i>` は最初に decoded AxiomName が直前 entry より小さくなる後続 index です。
duplicate axiom name の field は `axioms[<i>].name`、
`expected_value = "unique_axiom_names"`、`actual_value = "duplicate_axiom_name"` とし、
`<i>` は同じ AxiomName がすでに出現している最小の後続 index です。
schema / domain validation 後、validator は `axiom_report_hash` を再計算します。
再計算値と field 値が一致しない場合は schema / domain failure ではなく、
`axiom_policy` oracle の dedicated self-hash mismatch failure として扱います。
`MachineCheckResult.axiom_report_hash` はこの canonical `axiom_report_hash` と一致しなければなりません。
`checker.version` は audit / display 用 metadata であり、checker identity ではありません。
`result_hash` は `checker.profile`、`checker.id`、`checker.binary_id`、`checker.binary_hash`、
`checker.build_hash` を含めますが、`checker.version` は含めません。
`checker.version` が変わっただけの同一 binary / build / verdict は同じ `result_hash` を持ちます。
ただし `checker.version` は `run_artifact_hash` には含まれるため、保存 artifact の完全性検査では
version metadata の変更も検出されます。
将来 `result_hash` の対象 field を増やす場合は schema version を上げます。

`run_artifact_hash` は `run_artifact_hash` field 自身だけを除いた
`MachineCheckResult` object 全体の canonical hash です。
`run_artifact_hash` は canonicalized object hash であり、保存ファイル bytes の hash ではありません。
`run_artifact_hash` は exact saved artifact integrity 用なので、`result_hash` から除外される次の field も含めます。

```text
- request_id
- result_id
- request_hash
- result_hash
- attempt
- process
- resource_usage
- diagnostics
- axioms_used
- declarations_checked
```

`run_artifact_hash` の計算順序は固定します。
runner はまず `result_hash` と `run_artifact_hash` を omit した object から
`result_hash` を計算して object に書き込みます。
次に `run_artifact_hash` だけを omit し、書き込まれた `result_hash` を含む object から
`run_artifact_hash` を計算します。
検証側も同じ順序で再計算します。
`result_hash` が欠けている、または再計算値と一致しない artifact では、
`run_artifact_hash` を正当な integrity hash として扱ってはいけません。

したがって diagnostics や `result_id` が変わると `run_artifact_hash` は変わります。
verdict identity には `result_hash` を使い、保存 artifact の改ざん検出には `run_artifact_hash` を使います。
CI / release の artifact object 改ざん検出では `run_artifact_hash` を使い、
正本 result identity や sidecar 参照では `result_hash` を使います。
保存ファイル bytes の完全性も検査する場合は、audit bundle manifest に
別途 `file_hash` を記録します。

`normalized_result_hash` は `normalized_result_id`、`normalized_result_hash`、
`results[*].result_id` field を除いた `NormalizedCheckResult` の canonical hash です。
normalization は volatile run metadata を入れないため、同じ certificate / policy /
runner-envelope verdict からは同じ `normalized_result_hash` が得られます。

`error` object に入れてよいのは deterministic な structured fields だけです。
自然言語の説明、OS error text、stderr excerpt、human-facing hint を `error` に入れてはいけません。
MVP の canonical artifact では、それらは下の fixed diagnostics token、または artifact 外ログに分離します。
`diagnostics` は `result_hash` から除外されるため、文言変更で verdict identity が変わりません。
この文書で定義する `diagnostics` field は optional array of string です。
診断がない場合は `diagnostics` field を omit し、empty array を canonical output として書いてはいけません。
`diagnostics` を書く場合は non-empty で、fixed diagnostics token だけを入れます。
MVP の fixed diagnostics token は
`^[A-Za-z0-9_.-]+:[a-z][a-z0-9_]{0,63}$` に一致する ASCII string です。
左辺は source field / source component、右辺は diagnostic code です。
この文書の共通規則または command-specific section が具体的な token を定義していない場合は、
その diagnostic を canonical artifact に書かず omit します。
複数 token を書く場合は duplicate を除去し、bytewise lexicographic order で昇順に並べます。
free-form text、redacted path、raw stderr / stdout excerpt、OS error text、byte dump を
canonical artifact の `diagnostics` に入れてはいけません。
stderr / stdout が存在したことだけを示す場合は、command-specific section が許すときだけ
`checker_process:stderr_present` または `checker_process:stdout_present` を使います。
verdict、pass/fail、retry、分類、identity の判定に `diagnostics` を使ってはいけません。

MVP の structured error fields：

```text
- kind
- reason_code, if applicable
- field, if applicable
- declaration, if applicable
- core_path, if applicable
- section, if applicable
- offset, if applicable
- expected_hash, if applicable
- actual_hash, if applicable
- expected_value, if applicable
- actual_value, if applicable
```

MVP の structured error field value shape は次で固定します。
`reason_code` は各 error kind ごとの closed enum です。
`field` はこの文書で field shape が明記された deterministic field path string だけを使います。
`declaration` は Phase 2 `Name` の dotted JSON 表現で、3.3 の name grammar に従います。
`core_path` は non-empty JSON array で、各 component は non-negative i64 array index、
または `^[A-Za-z_][A-Za-z0-9_]*$` に一致する core AST field label string です。
`section` は `^[A-Za-z][A-Za-z0-9._-]{0,63}$` に一致する certificate / artifact section token です。
`offset` は対象 file bytes の先頭からの byte offset を表す non-negative i64 です。
`expected_hash` と `actual_hash` は `sha256:<lower-hex>` string です。
`expected_value` と `actual_value` は deterministic JSON scalar です。
値は、この文書の各 reason code / schema failure table で固定された ASCII token、
schema requirement 名、enum / mode / path / name などの deterministic string、
または比較対象 schema field の canonical integer / boolean に限定します。
float と null は禁止します。
object と array を `expected_value` / `actual_value` に直接入れてはいけません。
entry object など構造値を比較結果に入れると明記された箇所では、
RFC 8785 canonical JSON bytes を UTF-8 string として入れます。

---

# 4. MachineCheckRequest

AI agent や CI bot は checker を直接自由に呼び出すのではなく、
policy で固定された runner に request を渡します。

```json
{
  "schema": "npa.phase8.machine_check_request.v1",
  "request_id": "mchkreq_001",
  "request_hash": "sha256:...",
  "module": "Std.Nat",
  "policy": {
    "id": "phase8-pr",
    "version": 1,
    "hash": "sha256:..."
  },
  "certificate": {
    "kind": "path",
    "path": "build/certs/Std/Nat.npcert",
    "file_hash": "sha256:...",
    "expected_certificate_hash": "sha256:..."
  },
  "imports": {
    "mode": "locked_store",
    "manifest": "build/certs/import-lock.json",
    "manifest_hash": "sha256:..."
  },
  "checker_profile": "reference",
  "trust_mode": "pr",
  "axiom_policy": "ci/axiom-policy.toml",
  "budget": {
    "max_steps": 10000000,
    "max_memory_mb": 2048,
    "timeout_ms": 60000
  }
}
```

`checker_profile`、`trust_mode`、`axiom_policy`、`budget`、`imports.mode` は
request にも policy にも現れます。
request 側の値は「この request が policy から選択した値」を記録するためのものであり、
policy を上書きするためのものではありません。
`certificate.file_hash` は input file bytes の hash で、`expected_certificate_hash` は
checker が再計算すべき canonical certificate hash です。
ordinary check request では `certificate.expected_certificate_hash` は required です。
`MachineCheckRequest.policy` は provenance であり、runner が policy file を探すための
path ではありません。
`MachineCheckRequest.policy.id` は 4.1 の `RunnerPolicy.id` と同じ grammar、
`MachineCheckRequest.policy.version` は 4.1 の `RunnerPolicy.version` と同じ positive i64 domain を使います。
この2 field の grammar / domain violation は request load validation の
`request_schema_invalid` であり、runner pre-check の
`request_policy_hash_mismatch` の id / version subcase には到達しません。
`policy.id` violation では `error.field = "policy.id"`、
`expected_value = "runner_policy_id"`、`actual_value = "invalid_name_format"` にします。
`policy.version` violation では `error.field = "policy.version"`、
`expected_value = "positive_i64"`、
`actual_value = "non_positive_integer"` または `"integer_out_of_range"` にします。
MVP の `npa-check run` は `--policy` で `RunnerPolicy` file を明示的に受け取ります。
API では `/machine/check/certificate` request body が `MachineCheckRequest` と
`RunnerPolicyReference` を両方含む wrapper object になります。
runner は request 内の `policy.hash` だけを根拠に policy file を選んではいけません。

MVP の import lock manifest schema：

```json
{
  "schema": "npa.phase8.import_lock_manifest.v1",
  "imports": [
    {
      "module": "Std.Logic",
      "export_hash": "sha256:...",
      "certificate": {
        "kind": "path",
        "path": "build/certs/Std/Logic.npcert",
        "file_hash": "sha256:...",
        "certificate_hash": "sha256:..."
      }
    }
  ]
}
```

`MachineCheckRequest.imports.manifest_hash` は、この manifest file bytes の SHA-256 です。
import lock manifest object 自身に self hash field は持たせません。
`imports` は decoded Phase 2 `ModuleName` の canonical name order、
次に `export_hash`、次に `certificate.certificate_hash` の bytewise lexicographic order で昇順に並べます。
MVP v1 ではすべての entry で `certificate.certificate_hash` を required とし、
欠落した entry は import lock manifest schema invalid です。
したがって sort key に absent value は存在しません。
`module` と `certificate.path` はそれぞれ unique です。
`certificate.kind = path` だけを MVP で許可します。
MVP の top-level required field は `schema`、`imports` です。
manifest object、`imports[]` entry object、`certificate` object は closed-world object で、
unknown field と duplicate key を禁止します。
`schema` が `npa.phase8.import_lock_manifest.v1` 以外の string の場合、
および `certificate.kind` が `path` 以外の string の場合は
`actual_value = "invalid_enum"` の schema / domain failure とします。
`certificate.path` は workspace-relative path で、runner / checker は HTTP URL、
directory scan、database lookup、network import resolution に fallback してはいけません。
`certificate.file_hash` は referenced `.npcert` file bytes の SHA-256 です。
`certificate.certificate_hash` は referenced certificate の claimed certificate hash ではなく、
checker がその import certificate を検査して再計算すべき canonical certificate hash です。

通常 trust mode でも import entry の `export_hash` と `certificate.certificate_hash` は required です。
これは Phase 8 AI runner の import lock manifest に対する追加制約であり、
Phase 2 / core certificate payload の `ImportEntry.certificate_hash` optional semantics を
変更しません。
通常 mode の `.npcert` 内 import entry が `certificate_hash` を持たないことだけを理由に
checker が certificate payload を拒否してはいけません。
Phase 8 import lock は、runner が import certificate file identity と deterministic replay identity を
固定するための外側 manifest です。
checker は manifest から import certificate bytes を解決しますが、certificate payload 自体の
normal / high-trust import semantics は Phase 2 / core-spec に従って検査します。
将来 schema で `certificate.certificate_hash` を optional にする場合は、schema version を上げ、
absent value の sort rule を同時に定義しなければなりません。
`trust_mode = high-trust` では、manifest に書かれた全 import certificate file identity と
canonical certificate hash を `import_certificate_hash` `AuxiliaryResult` で検証することが
pass condition に追加されます。
runner は request pre-check で import lock manifest file bytes hash と
import lock manifest の JSON / schema / domain validation だけを検査します。
import certificate の full semantic verification、`export_hash` validation、依存関係検査は
checker が import certificate を実際に読むときに行います。
`import_certificate_hash` `AuxiliaryResult` の deterministic oracle は、import certificate file bytes を
canonical decode し、canonical certificate hash が import lock の `certificate.certificate_hash` と
一致することだけを検査します。
`npa-check run` / `/machine/check/certificate` で import lock manifest が unreadable、
hash mismatch、invalid JSON、schema invalid、domain invalid の場合は checker を起動しません。
unreadable は `request_import_manifest_file_unreadable`、hash mismatch は
`request_import_manifest_hash_mismatch`、JSON / schema / domain invalid は
`request_import_manifest_invalid` の `policy_failure` result にします。
`request_import_manifest_invalid` では JSON parse failure の `error.field` は
`imports.manifest`、schema / domain root-level failure の `error.field` も `imports.manifest`、
nested field failure の `error.field` は `imports.manifest.<JSON path>` とします。
JSON parse failure では `expected_value = "valid_json"`、`actual_value = "invalid_json"` とします。
schema / domain failure では `expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_enum`、
`invalid_hash_format`、`invalid_name_format`、`invalid_path`、`null_not_allowed`、`order_violation`、
`duplicate_field`、`duplicate_module`、`duplicate_path` のいずれかを入れます。
ImportLockManifest validation は schema failure を domain failure より先に報告します。
複数の schema failure が同時に存在する場合は、次の順で最初の1件だけを返します。

```text
1. top-level JSON value is not object
2. top-level schema
3. top-level imports
4. imports[] entry object, by smaller array index
5. imports[].module, by smaller array index
6. imports[].export_hash, by smaller array index
7. imports[].certificate object, by smaller array index
8. imports[].certificate.kind, by smaller array index
9. imports[].certificate.path, by smaller array index
10. imports[].certificate.file_hash, by smaller array index
11. imports[].certificate.certificate_hash, by smaller array index
12. unknown field, by the containing object order above and then bytewise field name
```

known field の duplicate object key は、その field の schema order 位置で
`expected_value = "unique_object_keys"`、`actual_value = "duplicate_field"` として報告します。
unknown field の duplicate object key は item 12 の位置で
`expected_value = "unique_object_keys"`、`actual_value = "duplicate_field"` を返し、
`error.field` は重複した後続 unknown field の JSON path にします。
複数の domain failure が同時に存在する場合は、
`imports` order violation、`module` duplicate、`certificate.path` duplicate の順で最初の1件だけを返します。
duplicate は、同じ key がすでに出現している最小の後続 entry index を報告対象にします。
`imports` order violation の field は `imports.manifest.imports[<i>]` で、
`<i>` は最初に sort key が直前 entry より小さくなる後続 entry index です。
`module` duplicate の field は `imports.manifest.imports[<i>].module`、
`certificate.path` duplicate の field は `imports.manifest.imports[<i>].certificate.path` です。
`auxiliary import-certificate-hash` で同じ invalid import lock manifest を検出した場合は
`AuxiliaryResult.status = inconclusive`、`error.reason_code = import_certificate_hash_inconclusive` にします。
この場合の `AuxiliaryResult.error.field` / `expected_value` / `actual_value` は
`import_certificate_hash` oracle の field shape に従います。
release bundle validation で included `import_lock` が invalid な場合は bundle invalid です。
checker は import lock manifest を読む場合も、そこに書かれた hash を信用せず、
import certificate bytes から `export_hash` / `certificate_hash` を再計算します。

runner はまず request load validation を行います。

```text
- request file bytes を読める
- JSON として parse できる
- top-level schema が npa.phase8.machine_check_request.v1
- request_hash field が存在する
- request_hash field を含む full MachineCheckRequest schema / domain validation を通る
- request.request_hash が 3.3 の規則で再計算した hash と一致する
```

full schema / domain validation では `request_hash` の `sha256:<lower-hex>` 形式、
3.3 の name / path / hash field shape、`MachineCheckRequest.policy.id` / `version` domain、
`MachineCheckRequest.budget` positive integer domain などを検査します。
`request_hash` が missing の場合だけ専用 reason `request_hash_missing` を使い、
`request_hash` が present だが wrong type / explicit null / invalid hash format の場合は
`request_schema_invalid` です。
複数の request schema / domain failure が同時に存在する場合は、schema 定義の field 出現順を
深さ優先でたどり、object member は schema に書かれた順、array element は小さい index 順で
最初の failure を返します。
この schema / domain validation が通った後でだけ request self-hash を再計算します。
したがって `policy.version` domain failure と `request_hash` mismatch が同時に存在する場合は
`request_schema_invalid` が先です。

request load validation に失敗した場合、checker は起動せず、
`MachineCheckResult` も作りません。
この場合は `MachineCheckRequestErrorResult` を保存または返します。
`MachineCheckResult` は valid な `request_hash` に紐づく runner envelope だけに限定します。
API の inline `check_request` object で request load validation に失敗した場合も
同じ `MachineCheckRequestErrorResult` を返しますが、file input ではないため
`request_path` と `request_file_hash` は omit します。
この場合の `error.field` は wrapper path ではなく `MachineCheckRequest` 内の artifact-local path にします。
たとえば self-hash mismatch は `check_request.request_hash` ではなく
`request_hash` です。
API request body 自体の JSON parse failure や `check_request` wrapper field の
missing / wrong type / explicit null は `ApiError` であり、
`MachineCheckRequestErrorResult` を返しません。
inline object では `request_file_unreadable` と `request_json_invalid` を使いません。

MVP の `MachineCheckRequestErrorResult`：

```json
{
  "schema": "npa.phase8.machine_check_request_error_result.v1",
  "result_id": "mchkreqerr_001",
  "result_hash": "sha256:...",
  "status": "failed",
  "request_path": "build/check-requests/Std.Nat.reference.json",
  "request_file_hash": "sha256:...",
  "error": {
    "kind": "request_load_failure",
    "reason_code": "request_hash_missing",
    "field": "request_hash"
  }
}
```

`MachineCheckRequestErrorResult.result_hash` は `result_id` と `result_hash` field を除いた
canonical hash です。
`request_path` は request file path が分かる場合だけ required です。
`request_file_hash` は request file bytes を読めた場合だけ required です。
JSON parse 前に失敗した場合、`request_file_hash` は file bytes の sha256 です。
file を読めない場合は `request_file_hash` を omit します。
MVP の `MachineCheckRequestErrorResult.error.reason_code` は次に限定します。

```text
- request_file_unreadable
- request_json_invalid
- request_schema_invalid
- request_hash_missing
- request_hash_mismatch
```

`request_file_unreadable` では `error.field = "request_path"`、
`actual_value = "unreadable"` にし、`request_file_hash` は omit します。
`request_json_invalid` では `error.field = "request"`、
`actual_value = "invalid_json"` にし、`request_file_hash` は required です。
`request_schema_invalid` では `error.field` に invalid field の JSON path を入れ、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、
`invalid_enum`、`invalid_path`、`invalid_hash_format`、`invalid_name_format`、`null_not_allowed`、
`non_positive_integer`、`integer_out_of_range`、`duplicate_field` のいずれかを入れます。
top-level `schema` が `npa.phase8.machine_check_request.v1` でない場合も
`request_schema_invalid` です。
この場合は `error.field = "schema"`、
`expected_value = "npa.phase8.machine_check_request.v1"`、
`actual_value = "missing"`、`"null_not_allowed"`、`"wrong_type"`、
または入力 artifact の `schema` 文字列を入れます。
top-level schema mismatch では `actual_value = "wrong_schema"` を使いません。
top-level JSON value が object でない場合は `error.field = "$"`、
`expected_value = "object"`、`actual_value = "wrong_type"` または `"null_not_allowed"` にします。
`request_hash_missing` では `error.field = "request_hash"`、
`expected_value = "sha256:<lower-hex>"`、`actual_value = "missing"` にします。
`request_hash_mismatch` では `error.field = "request_hash"`、
`error.expected_hash` に再計算した request hash、
`error.actual_hash` に request file 内の `request_hash` を入れます。
`MachineCheckRequestErrorResult` は checker verdict ではなく、normalization / comparison に渡してはいけません。

runner は実行前に次を検査します。

```text
- CLI/API から明示された RunnerPolicyReference を解決できる
- request.policy.id / version / hash が読み込んだ RunnerPolicy の id / version / canonical hash と一致する
- request.trust_mode が RunnerPolicy.trust_mode と一致する
- request.checker_profile が RunnerPolicy.required_checker_profiles または
  explicitly allowed optional profiles に含まれる
- request.axiom_policy が RunnerPolicy.axiom_policy.path と一致する
- RunnerPolicy.axiom_policy.hash が axiom policy file bytes の hash と一致する
- request.budget が RunnerPolicy.budgets[checker_profile] と一致する
- request.imports.mode が RunnerPolicy.import_policy.mode と一致する
- request.imports.manifest_hash が import lock file bytes の hash と一致する
- import lock manifest が JSON / schema / domain validation を通る
- request.certificate.file_hash が input certificate file bytes の hash と一致する
```

一致しない場合、runner は checker を起動せず `policy_failure` result を保存します。
「policy を優先して request を黙って修正する」動作は禁止です。

MVP の runner pre-check / policy failure reason code：

```text
- runner_policy_reference_invalid
- runner_policy_file_unreadable
- runner_policy_hash_mismatch
- runner_policy_invalid
- request_policy_hash_mismatch
- request_trust_mode_mismatch
- request_checker_profile_not_allowed
- request_axiom_policy_mismatch
- request_axiom_policy_file_unreadable
- request_axiom_policy_hash_mismatch
- request_budget_mismatch
- request_import_mode_mismatch
- request_import_manifest_file_unreadable
- request_import_manifest_hash_mismatch
- request_import_manifest_invalid
- request_certificate_file_unreadable
- request_certificate_file_hash_mismatch
- checker_binary_file_unreadable
- checker_binary_hash_mismatch
- checker_identity_manifest_file_unreadable
- checker_identity_manifest_hash_mismatch
- checker_identity_manifest_invalid
- checker_build_hash_mismatch
- checker_identity_mismatch
- checker_identity_missing
```

runner pre-check failure では `error.field` に不一致 field path を入れます。
hash 不一致では `error.expected_hash` / `error.actual_hash` を使い、
enum / path / profile 不一致では `error.expected_value` / `error.actual_value` を使います。
checker を起動していない failure は `process.launched = false` にします。
policy file が valid に解決できた後の request pre-check failure では、
required field である `MachineCheckResult.policy` には loaded `RunnerPolicy` の
`id`、`version`、canonical hash を入れます。
request 側 metadata と一致しない場合でも、`MachineCheckResult.policy` に
`MachineCheckRequest.policy` を copy してはいけません。
request load validation 後に `runner_policy_*` failure で `MachineCheckResult` を返す場合、
required field である `MachineCheckResult.policy` は
読み込めた policy file や malformed `RunnerPolicyReference` から合成せず、
valid な `MachineCheckRequest.policy` をそのまま copy します。
`MachineCheckRequest.policy` 自体を読めない場合は request load failure なので、
`MachineCheckRequestErrorResult` を返し、`MachineCheckResult` は作りません。
`runner_policy_hash_mismatch` などの expected / actual は `error` field にだけ記録します。
この境界は、`RunnerPolicyReference` が valid で、policy file が readable / parseable / schema-valid で、
かつその canonical hash が `RunnerPolicyReference.hash` と一致した時点です。
その時点より前の `runner_policy_reference_invalid`、`runner_policy_file_unreadable`、
`runner_policy_hash_mismatch`、`runner_policy_invalid` では、runner は trusted `RunnerPolicy`
を持たないため `MachineCheckRequest.policy` を provenance として copy します。
この copy は policy validation の成功を意味せず、`error.reason_code` が failure の正本です。
その時点以後の `request_policy_hash_mismatch`、`request_trust_mode_mismatch`、
`request_budget_mismatch` などでは、`MachineCheckResult.policy` は loaded `RunnerPolicy` の
`id`、`version`、canonical hash です。
たとえば request の `policy.hash` が stale でも、policy reference と file が一致して読めたなら、
`MachineCheckResult.policy.hash` は loaded policy hash になり、stale request hash は
`error.actual_hash` にだけ記録します。
runner pre-check reason の field shape は次で固定します。
複数の pre-check failure が同時に存在する場合は、この table の順序で最初の1件だけを報告します。
この順序は `runner_policy_*`、`request_*`、checker identity failure を含む runner pre-check
全体の優先順です。
ただし checker executable / identity validation では、4.1 の checker identity validation order を
この table 内の checker 関連 row より優先します。
これは同じ reason code が executable bytes check と identity manifest check の両方で使われるためです。
この table で `SelectedCheckerPolicy` は
`RunnerPolicy.checker_allowlist` のうち
`profile = MachineCheckRequest.checker_profile` の unique entry を指します。
`request_checker_profile_not_allowed` より後の checker identity check では、
この entry が存在することを前提にします。
`CheckerIdentityManifestEntry` は起動前 identity manifest を使う場合に、
同じ `profile` で照合される manifest entry を指します。

```text
runner_policy_reference_invalid:
  field = "policy" or invalid RunnerPolicyReference member path
  expected_value = "RunnerPolicyReference" or member-specific expected value
  actual_value = missing | wrong_type | unknown_field | invalid_enum | invalid_path |
                 invalid_hash_format | null_not_allowed | order_violation | duplicate_field

runner_policy_file_unreadable:
  field = "policy.path"
  expected_value = "readable_file"
  actual_value = "unreadable"

runner_policy_hash_mismatch:
  field = "policy.hash"
  expected_hash = RunnerPolicyReference.hash
  actual_hash = loaded RunnerPolicy canonical hash

runner_policy_invalid:
  JSON parse failure:
    field = "policy.path"
    expected_value = "valid_json"
    actual_value = "invalid_json"
  schema / domain validation failure:
    field = RunnerPolicy-local invalid field path
    expected_value / actual_value = RunnerPolicy schema / domain validation field shape

request_policy_hash_mismatch:
  policy.id mismatch:
    field = "policy.id"
    expected_value = RunnerPolicy.id
    actual_value = MachineCheckRequest.policy.id
  policy.version mismatch:
    field = "policy.version"
    expected_value = RunnerPolicy.version
    actual_value = MachineCheckRequest.policy.version
  policy.hash mismatch:
    field = "policy.hash"
    expected_hash = loaded RunnerPolicy canonical hash
    actual_hash = MachineCheckRequest.policy.hash
  If multiple policy metadata members mismatch, report id, then version, then hash.

request_trust_mode_mismatch:
  field = "trust_mode"
  expected_value = RunnerPolicy.trust_mode
  actual_value = MachineCheckRequest.trust_mode

request_checker_profile_not_allowed:
  field = "checker_profile"
  expected_value = "required_or_optional_checker_profile"
  actual_value = MachineCheckRequest.checker_profile

request_axiom_policy_mismatch:
  field = "axiom_policy"
  expected_value = RunnerPolicy.axiom_policy.path
  actual_value = MachineCheckRequest.axiom_policy

request_axiom_policy_file_unreadable:
  field = "axiom_policy"
  expected_value = "readable_file"
  actual_value = "unreadable"

request_axiom_policy_hash_mismatch:
  field = "axiom_policy.hash"
  expected_hash = RunnerPolicy.axiom_policy.hash
  actual_hash = axiom policy file bytes sha256

request_budget_mismatch:
  field = "budget.<member>"
  expected_value = RunnerPolicy.budgets[MachineCheckRequest.checker_profile].<member>
  actual_value = MachineCheckRequest.budget.<member>
  Check members in this order: max_steps, max_memory_mb, timeout_ms.

request_import_mode_mismatch:
  field = "imports.mode"
  expected_value = RunnerPolicy.import_policy.mode
  actual_value = MachineCheckRequest.imports.mode

request_import_manifest_file_unreadable:
  field = "imports.manifest"
  expected_value = "readable_file"
  actual_value = "unreadable"

request_import_manifest_hash_mismatch:
  field = "imports.manifest_hash"
  expected_hash = MachineCheckRequest.imports.manifest_hash
  actual_hash = import lock file bytes sha256

request_import_manifest_invalid:
  JSON parse failure:
    field = "imports.manifest"
    expected_value = "valid_json"
    actual_value = "invalid_json"
  schema / domain validation failure:
    field = "imports.manifest" for root-level failure,
            otherwise "imports.manifest.<invalid import lock field path>"
    expected_value = import lock schema / domain requirement
    actual_value = missing | wrong_type | unknown_field | invalid_enum |
                   invalid_hash_format | invalid_name_format | invalid_path | null_not_allowed |
                   order_violation | duplicate_field | duplicate_module |
                   duplicate_path

request_certificate_file_unreadable:
  field = "certificate.path"
  expected_value = "readable_file"
  actual_value = "unreadable"

request_certificate_file_hash_mismatch:
  field = "certificate.file_hash"
  expected_hash = MachineCheckRequest.certificate.file_hash
  actual_hash = certificate file bytes sha256

checker_binary_file_unreadable:
  field = "checker.binary_id"
  expected_value = "readable_executable"
  actual_value = "unreadable"

checker_binary_hash_mismatch:
  executable bytes mismatch:
    field = "checker.binary_hash"
    expected_hash = SelectedCheckerPolicy.binary_hash
    actual_hash = checker binary file bytes sha256
  pre-launch identity manifest binary_hash mismatch:
    field = "checker.binary_hash"
    expected_hash = SelectedCheckerPolicy.binary_hash
    actual_hash = CheckerIdentityManifestEntry.binary_hash
  If both would fail, report executable bytes mismatch first.

checker_identity_manifest_file_unreadable:
  field = "checker_identity_manifest.path"
  expected_value = "readable_file"
  actual_value = "unreadable"

checker_identity_manifest_hash_mismatch:
  field = "checker_identity_manifest.manifest_hash"
  expected_hash = RunnerPolicy.checker_identity_manifest.manifest_hash
  actual_hash = checker identity manifest file bytes sha256

checker_identity_manifest_invalid:
  JSON parse failure:
    field = "checker_identity_manifest.path"
    expected_value = "valid_json"
    actual_value = "invalid_json"
  schema / domain validation failure:
    field = "checker_identity_manifest.<invalid CheckerIdentityManifest field path>"
    expected_value / actual_value = CheckerIdentityManifest schema / domain validation field shape

checker_build_hash_mismatch:
  pre-launch identity manifest build_hash mismatch:
    field = "checker.build_hash"
    expected_hash = SelectedCheckerPolicy.build_hash
    actual_hash = CheckerIdentityManifestEntry.build_hash
  post-launch raw result build_hash mismatch:
    field = "checker.build_hash"
    expected_hash = SelectedCheckerPolicy.build_hash
    actual_hash = CheckerRawResult.checker_build_hash

checker_identity_mismatch:
  pre-launch identity manifest checker_id mismatch:
    field = "checker.id"
    expected_value = SelectedCheckerPolicy.checker_id
    actual_value = CheckerIdentityManifestEntry.checker_id
  post-launch raw result checker_id mismatch:
    field = "checker.id"
    expected_value = SelectedCheckerPolicy.checker_id
    actual_value = CheckerRawResult.checker_id
  pre-launch identity manifest binary_id mismatch:
    field = "checker.binary_id"
    expected_value = SelectedCheckerPolicy.binary_id
    actual_value = CheckerIdentityManifestEntry.binary_id

checker_identity_missing:
  missing manifest entry:
    field = "checker_identity_manifest.checkers[]"
    expected_value = "entry_for_selected_checker_profile"
    actual_value = "missing"
  missing checker_id:
    field = "checker.id"
    expected_value = "checker_id"
    actual_value = "missing"
  malformed checker_id:
    field = "checker.id"
    expected_value = "non_empty_string"
    actual_value = wrong_type | null_not_allowed | empty_string
  missing checker_build_hash:
    field = "checker.build_hash"
    expected_value = "sha256:<lower-hex>"
    actual_value = "missing"
  malformed checker_build_hash type:
    field = "checker.build_hash"
    expected_value = "sha256:<lower-hex>"
    actual_value = wrong_type | null_not_allowed | invalid_hash_format
```

`runner_policy_reference_invalid` では、reference object 自体が missing / wrong type / explicit null の場合
`error.field = "policy"`、`expected_value = "RunnerPolicyReference"`、
`actual_value` に `missing`、`wrong_type`、または `null_not_allowed` を入れます。
reference object が存在し、その member が不正な場合は
`error.field` に invalid member の JSON path を入れます。
既知 member では `policy.kind`、`policy.path`、`policy.hash` のいずれか、
unknown field では `policy.<unknown_field_name>` です。
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_enum`、
`invalid_path`、`invalid_hash_format`、`null_not_allowed`、`order_violation`、
`duplicate_field` のいずれかを入れます。
`runner_policy_file_unreadable` では `error.field = "policy.path"`、
`expected_value = "readable_file"`、`actual_value = "unreadable"` にします。
`runner_policy_hash_mismatch` では `error.field = "policy.hash"`、
`error.expected_hash` に `RunnerPolicyReference.hash`、
`error.actual_hash` に読み込んだ `RunnerPolicy` の canonical hash を入れます。
`runner_policy_invalid` では、policy file の JSON parse failure の場合
`error.field = "policy.path"`、`expected_value = "valid_json"`、
`actual_value = "invalid_json"` にします。
この場合、invalid policy field の JSON path は存在しないため、policy file path を field として使います。
`RunnerPolicy` schema / domain validation failure では
`error.field` に RunnerPolicy-local invalid field の JSON path を入れます。
top-level `schema` が `npa.phase8.runner_policy.v1` でない場合は、
`error.field = "schema"`、
`expected_value = "npa.phase8.runner_policy.v1"`、
`actual_value = "missing"`、`"null_not_allowed"`、`"wrong_type"`、
または入力 policy の `schema` 文字列にします。
top-level schema mismatch では `actual_value = "wrong_schema"` を使いません。
top-level JSON value が object でない場合は `error.field = "$"`、
`expected_value = "object"`、`actual_value = "wrong_type"` または `"null_not_allowed"` にします。
それ以外の field schema failure の `actual_value` には `missing`、`wrong_type`、
`unknown_field`、`invalid_enum`、`invalid_path`、`invalid_hash_format`、
`null_not_allowed`、`order_violation`、`duplicate_field` のいずれかを入れます。
policy domain failure の `expected_value` / `actual_value` は 4.1 の
RunnerPolicy schema / domain validation field shape に従います。

request の禁止事項：

```text
- AI が任意の checker binary path を指定する
- AI が import をネットワークから解決する
- AI が source file を checker input として渡す
- AI が expected result を checker に渡す
- AI が hash mismatch を許容する flag を立てる
- AI が noncanonical certificate を受理する互換 mode を選ぶ
```

ここでいう `expected result` は checker verdict や theorem statement / export expectation を
AI が任意に与えることです。
`MachineCheckRequest.certificate.expected_certificate_hash` は expected verdict ではなく、
runner が入力 artifact identity を固定し、checker が再計算した canonical certificate hash と
照合するための deterministic binding field です。
Phase 8 Human Profile の external checker challenge mode にある optional expected statement hash /
expected export hash は、Phase 8 AI Profile MVP の `MachineCheckRequest` には入りません。
将来それらを machine runner に入れる場合は、AI / sidecar / request が選ぶ値ではなく、
policy-owned challenge artifact から runner が固定順序の dynamic arg として渡す別 schema にします。

`checker_profile` は allowlist から選びます。

```text
allowed checker profiles:
  - fast-kernel
  - reference
  - external
  - high-trust-reference
```

AI が profile を提案することはできます。
しかし実際に使う profile は CI policy / release policy が決めます。

## 4.1 Runner policy

runner policy は、AI agent ではなく repository / CI / release process が所有する
deterministic config です。
`MachineCheckRequest.policy.hash` は、この policy file の canonical serialization hash です。
policy file の解決は request ではなく runner invocation が所有します。

MVP の `RunnerPolicyReference` schema：

```json
{
  "kind": "file",
  "path": "ci/phase8-pr-policy.json",
  "hash": "sha256:..."
}
```

`kind = file` だけを MVP で許可します。
`path` は workspace-relative path です。
runner は file bytes を読み、canonical `RunnerPolicy` として parse し、
再計算した canonical hash が `RunnerPolicyReference.hash` と一致することを検査します。
その後、同じ hash を `MachineCheckRequest.policy.hash` と照合します。
`RunnerPolicyReference.hash` と file hash は別物です。
`RunnerPolicyReference.hash` は parsed `RunnerPolicy` の canonical hash であり、
policy file bytes の sha256 ではありません。
CLI では `--policy <path>` と `--policy-hash <sha256:...>` の組を
`RunnerPolicyReference` として扱います。
`--policy` だけで暗黙に reference hash を補ってはいけません。
`--policy-hash` がない invocation は CLI argument validation error であり、
`MachineCheckResult` や validation response を作りません。
`--policy` と `--policy-hash` の両方が存在した後の malformed reference
（invalid hash format、invalid path、unknown field 相当）は、CLI argument error ではなく
command-specific な policy reference failure として扱います。
API は同じ `RunnerPolicyReference` object を request body に含めます。
API では `RunnerPolicyReference` は endpoint wrapper field なので、
`policy` object の missing / wrong type / explicit null、`policy.kind` の invalid enum、
`policy.hash` の invalid hash format、unknown field、duplicate field は
endpoint 固有 body ではなく `ApiError.reason_code = api_request_schema_invalid` です。
`policy.path` が workspace path validation に失敗した場合は
`ApiError.reason_code = api_path_outside_workspace` です。
API wrapper validation を通過した後の policy file unreadable、policy hash mismatch、
policy file JSON / schema / domain failure だけを endpoint 固有の policy failure にします。
`/machine/check/certificate` ではそれぞれ `MachineCheckResult.error.reason_code` の
`runner_policy_file_unreadable`、`runner_policy_hash_mismatch`、
`runner_policy_invalid` に写します。
`runner_policy_hash_mismatch` は、`RunnerPolicyReference.hash` と parsed `RunnerPolicy`
canonical hash が一致しない場合に使います。

MVP の policy schema：

```json
{
  "schema": "npa.phase8.runner_policy.v1",
  "id": "phase8-pr",
  "version": 1,
  "trust_mode": "pr",
  "required_checker_profiles": ["reference"],
  "optional_checker_profiles": [],
  "checker_allowlist": [
    {
      "profile": "reference",
      "checker_id": "npa-checker-ref",
      "binary_id": "npa-checker-ref-macos-aarch64",
      "binary_hash": "sha256:...",
      "build_hash": "sha256:...",
      "allowed_args": ["--json", "--no-network", "--canonical-only"]
    }
  ],
  "checker_identity_manifest": {
    "kind": "file",
    "path": "ci/checker-identity-manifest.json",
    "manifest_hash": "sha256:..."
  },
  "import_policy": {
    "mode": "locked_store",
    "network": "forbidden",
    "require_import_lock_hash": true
  },
  "axiom_policy": {
    "path": "ci/axiom-policy.toml",
    "hash": "sha256:..."
  },
  "budgets": {
    "reference": {
      "max_steps": 10000000,
      "max_memory_mb": 2048,
      "timeout_ms": 60000
    }
  },
  "on_resource_exhausted": "fail",
  "on_missing_required_checker": "fail",
  "on_profile_requested_by_ai": "ignore_unless_policy_allows"
}
```

MVP では `on_resource_exhausted`、`on_missing_required_checker`、
`on_profile_requested_by_ai` は上記の固定値だけを許可します。
これらは将来の policy schema 拡張用 field であり、MVP の comparison rule を変えません。
別値が入っている `RunnerPolicy` は malformed policy として扱います。
`npa-check run` / `/machine/check/certificate` では checker を起動せず
`MachineCheckResult.status = failed`、`error.kind = policy_failure`、
`error.reason_code = runner_policy_invalid` を返します。
normalize / compare / challenge command が `RunnerPolicyReference` から解決した policy file で
同じ malformed policy を検出した場合は、`MachineCheckResult` ではなく各 command の
policy reference failure として返します。
具体的には normalize は `NormalizeErrorResult.error.reason_code = policy_reference_invalid`、
compare は `CompareValidationResult.error.kind = policy_failure` かつ
`error.reason_code = policy_reference_invalid`、challenge 系 command は
`CommandError.reason_code = policy_reference_invalid` です。

`RunnerPolicy.id` は `^[a-z][a-z0-9-]{0,63}$` に固定します。
`RunnerPolicy.version` は JSON integer で、`1 <= version <= 9223372036854775807` です。
`checker_allowlist[].checker_id` と `checker_allowlist[].binary_id` は
`^[a-z][a-z0-9._-]{0,127}$` に固定します。
slash、backslash、colon、空白、control character、uppercase は forbidden です。
`RunnerPolicy.budgets[*].max_steps`、`max_memory_mb`、`timeout_ms` と
`MachineCheckRequest.budget` の同名 member は同じ positive integer domain を使い、
`1 <= value <= 9223372036854775807` でなければなりません。
zero は「無制限」や「未設定」を表しません。
budget を無効化する互換 mode は MVP にはありません。
wrong type / explicit null は schema failure、0 以下は `actual_value = "non_positive_integer"`、
範囲超過は `actual_value = "integer_out_of_range"` です。

`trust_mode` ごとの MVP 必須 profile：

```text
pr:
  required_checker_profiles = [reference]

nightly:
  required_checker_profiles = [reference, external]

release:
  required_checker_profiles = [fast-kernel, reference, external]

high-trust:
  required_checker_profiles = [fast-kernel, reference, external, high-trust-reference]
```

MVP では `RunnerPolicy.required_checker_profiles` は `trust_mode` ごとの表と
同じ profile 集合・同じ順序でなければなりません。
policy file 側の値でこの表を上書きしてはいけません。
一致しない policy は `runner_policy_invalid` として扱い、checker を起動しません。
checker profile name grammar は `^[a-z][a-z0-9-]{0,63}$` に固定します。
`.`、`..`、slash、backslash、colon、空白、control character、uppercase は forbidden です。
この grammar は `required_checker_profiles[]`、`optional_checker_profiles[]`、
`checker_allowlist[].profile`、`budgets` object key、および `MachineCheckRequest.checker_profile` に適用します。
grammar violation は schema / domain failure として `actual_value = "invalid_name_format"` を使います。
RunnerPolicy では profile grammar validation を profile 集合・順序 validation より先に行います。
複数の invalid profile がある場合は、`required_checker_profiles[]`、
`optional_checker_profiles[]`、`checker_allowlist[].profile`、`budgets` key の順で最初の failure を返します。
同じ array 内では小さい index を先に、`budgets` key では bytewise lexicographic order で最初の key を返します。
`optional_checker_profiles` は required profile を含んではいけません。
`checker_allowlist` と `budgets` の profile 集合は、required / optional profile の和集合と
完全一致しなければなりません。
`optional_checker_profiles` は重複を許さず、配列順は semantic order です。
MVP では generator が bytewise lexicographic order で書き出すことを推奨しますが、
comparison / replay の optional profile order は policy file に保存された配列順を使います。
`checker_allowlist` は `profile` の bytewise lexicographic order で昇順に並べます。
`checker_allowlist.profile` と `checker_allowlist.binary_id` はそれぞれ unique です。
`allowed_args` の配列順は checker command identity の一部なので、sort してはいけません。
MVP の `allowed_args` は checker-owned static option だけを表します。
各 element は1つの argv element です。
flag の値が必要な static option は `--flag=value` の1 element として表し、
`["--flag", "value"]` のような separate value argv は forbidden です。
各 element は `--` で始まる non-empty visible ASCII string でなければならず、
ここで visible ASCII は各 character が `U+0021` から `U+007E` の範囲にあることです。
空白、NUL、control character、non-ASCII character、`--` 単体は forbidden です。
また runner-owned dynamic flag と bytewise に一致する element、および
`<runner-owned dynamic flag>=` で始まる element は forbidden です。
runner-owned dynamic flag は 4.2 の dynamic args block に列挙された
`--imports`、`--imports-hash`、`--trust-mode`、`--axiom-policy`、
`--axiom-policy-hash`、`--max-steps`、`--max-memory-mb`、`--timeout-ms` です。
この rule により、policy static args が runner-owned request / policy binding を
上書きしたり、`--` によって dynamic args を positional argument 化したりすることを禁止します。
複数の `allowed_args` domain failure がある場合は、`checker_allowlist` の配列順、
次に `allowed_args` の小さい index の順で最初の failure を返します。
`RunnerPolicy.axiom_policy.hash` は `axiom_policy.path` が指す file bytes の sha256 です。
`checker_identity_manifest` は optional です。
存在する場合、`kind = file` だけを許可し、`path` は workspace-relative path、
`manifest_hash` は referenced checker identity manifest file bytes の sha256 です。
request や AI sidecar は checker identity manifest を指定できません。
manifest を使うかどうかは `RunnerPolicy.checker_identity_manifest` の有無だけで決まります。
runner invocation は別 manifest reference、別 hash、または manifest check を無効化する flag を
指定してはいけません。
runner invocation が所有してよいのは workspace root / file access policy だけであり、
identity 判定に使う manifest identity は policy hash に含まれる `RunnerPolicy` 内 reference に限定します。

RunnerPolicy schema / domain validation failure の field shape は、
RunnerPolicy-local field shape と caller prefix rule に分けて定義します。
この 4.1 の table は RunnerPolicy-local field path です。
`npa-check run` / `/machine/check/certificate` の `runner_policy_invalid` では
local path をそのまま使い、top-level non-object は `field = "$"`、
top-level `schema` failure は `field = "schema"` にします。
normalize / compare / challenge 系 command の `policy_reference_invalid` では
caller prefix rule を適用し、local `$` は `policy`、local `schema` は `policy.schema`、
その他の local path は `policy.<RunnerPolicy JSON path>` に変換します。
この prefix 変換は `field` だけに適用し、`expected_value` / `actual_value` は
RunnerPolicy-local field shape の値をそのまま使います。
それ以外の schema failure では local invalid field の JSON path、schema requirement 名、
上記の field schema failure `actual_value` を使います。
domain failure では次の table の local `field`、`expected_value`、`actual_value` を使います。

```text
RunnerPolicy.id grammar violation:
  field = "id"
  expected_value = "runner_policy_id"
  actual_value = "invalid_name_format"

RunnerPolicy.version domain violation:
  field = "version"
  expected_value = "positive_i64"
  actual_value = "non_positive_integer" | "integer_out_of_range"

profile name grammar violation in required_checker_profiles:
  field = "required_checker_profiles[<i>]"
  expected_value = "checker_profile_name"
  actual_value = "invalid_name_format"

profile name grammar violation in optional_checker_profiles:
  field = "optional_checker_profiles[<i>]"
  expected_value = "checker_profile_name"
  actual_value = "invalid_name_format"

profile name grammar violation in checker_allowlist:
  field = "checker_allowlist[<i>].profile"
  expected_value = "checker_profile_name"
  actual_value = "invalid_name_format"

profile name grammar violation in budgets key:
  field = "budgets.<profile>"
  expected_value = "checker_profile_name"
  actual_value = "invalid_name_format"

checker_allowlist.checker_id grammar violation:
  field = "checker_allowlist[<i>].checker_id"
  expected_value = "checker_id"
  actual_value = "invalid_name_format"

checker_allowlist.binary_id grammar violation:
  field = "checker_allowlist[<i>].binary_id"
  expected_value = "checker_binary_id"
  actual_value = "invalid_name_format"

required_checker_profiles が trust_mode 表と一致しない:
  field = "required_checker_profiles"
  expected_value = "profiles_for_trust_mode:<trust_mode>"
  actual_value = "profile_set_mismatch"

required_checker_profiles の順序だけが trust_mode 表と一致しない:
  field = "required_checker_profiles"
  expected_value = "profiles_for_trust_mode:<trust_mode>"
  actual_value = "profile_order_mismatch"

optional_checker_profiles が required profile を含む:
  field = "optional_checker_profiles[<i>]"
  expected_value = "exclude_required_checker_profiles"
  actual_value = "required_profile_in_optional"

optional_checker_profiles が重複 profile を含む:
  field = "optional_checker_profiles[<i>]"
  expected_value = "unique_profiles"
  actual_value = "duplicate_profile"

checker_allowlist に required / optional profile の entry がない:
  field = "checker_allowlist"
  expected_value = "entry_for_each_required_and_optional_profile"
  actual_value = "missing_checker_allowlist_entry"

checker_allowlist に required / optional profile 以外の entry がある:
  field = "checker_allowlist[<i>].profile"
  expected_value = "only_required_and_optional_profiles"
  actual_value = "unexpected_checker_allowlist_entry"

checker_allowlist が profile 昇順でない:
  field = "checker_allowlist[<i>]"
  expected_value = "profile_bytewise_ascending"
  actual_value = "order_violation"

checker_allowlist.profile が重複する:
  field = "checker_allowlist[<i>].profile"
  expected_value = "unique_profiles"
  actual_value = "duplicate_profile"

checker_allowlist.binary_id が重複する:
  field = "checker_allowlist[<i>].binary_id"
  expected_value = "unique_binary_ids"
  actual_value = "duplicate_binary_id"

checker_allowlist.allowed_args が static option rule に違反:
  field = "checker_allowlist[<i>].allowed_args[<j>]"
  expected_value = "static_checker_option_without_runner_owned_dynamic_args"
  actual_value = "positional_arg" | "end_of_options_marker" | "reserved_dynamic_arg" |
                 "invalid_arg_text"

budgets に required / optional profile の entry がない:
  field = "budgets"
  expected_value = "budget_for_each_required_and_optional_profile"
  actual_value = "missing_budget_entry"

budgets に required / optional profile 以外の entry がある:
  field = "budgets.<profile>"
  expected_value = "only_required_and_optional_profiles"
  actual_value = "unexpected_budget_entry"

budgets member domain violation:
  field = "budgets.<profile>.<member>"
  expected_value = "positive_i64"
  actual_value = "non_positive_integer" | "integer_out_of_range"

on_resource_exhausted が固定値 fail でない:
  field = "on_resource_exhausted"
  expected_value = "fail"
  actual_value = "invalid_fixed_value"

on_missing_required_checker が固定値 fail でない:
  field = "on_missing_required_checker"
  expected_value = "fail"
  actual_value = "invalid_fixed_value"

on_profile_requested_by_ai が固定値 ignore_unless_policy_allows でない:
  field = "on_profile_requested_by_ai"
  expected_value = "ignore_unless_policy_allows"
  actual_value = "invalid_fixed_value"
```

RunnerPolicy domain failure table の `<i>` / `<j>` は 0-based array index です。
invalid grammar / static option rule では最初に失敗した element の index、
duplicate では同じ key がすでに出現している最小の後続 index、
order violation では最初に `current < previous` となる後続 index を使います。
`budgets` key の profile grammar violation では bytewise lexicographic order で最初の
invalid key を `budgets.<profile>` に入れます。
`checker_allowlist` の missing entry failure は対応する concrete entry がないため
`field = "checker_allowlist"` のままにします。
`checker_allowlist` unexpected entry failure は最小の unexpected entry index を使います。
`budgets` の missing entry failure は concrete key を持たないため `field = "budgets"` のままにし、
unexpected entry failure は bytewise lexicographic order で最初の unexpected key を
`budgets.<profile>` に入れます。

RunnerPolicy validation は schema failure を domain failure より先に報告します。
複数の schema failure が同時に存在する場合は、top-level non-object、`schema`、
`id`、`version`、`trust_mode`、`required_checker_profiles` array、
`required_checker_profiles[]` element by smaller index、`optional_checker_profiles` array、
`optional_checker_profiles[]` element by smaller index、`checker_allowlist` array、
`checker_allowlist[]` entry object by smaller index、`checker_allowlist[].profile` by smaller index、
`checker_allowlist[].checker_id` by smaller index、`checker_allowlist[].binary_id` by smaller index、
`checker_allowlist[].binary_hash` by smaller index、`checker_allowlist[].build_hash` by smaller index、
`checker_allowlist[].allowed_args` array by smaller index、
`checker_allowlist[].allowed_args[]` element by checker entry index then arg index、
`checker_identity_manifest` object、`checker_identity_manifest.kind`、
`checker_identity_manifest.path`、`checker_identity_manifest.manifest_hash`、
`import_policy` object、`import_policy.mode`、`import_policy.network`、
`import_policy.require_import_lock_hash`、`axiom_policy` object、`axiom_policy.path`、
`axiom_policy.hash`、`budgets` object、`budgets.<profile>` object by bytewise profile key、
`budgets.<profile>.max_steps`、`budgets.<profile>.max_memory_mb`、
`budgets.<profile>.timeout_ms`、`on_resource_exhausted`、
`on_missing_required_checker`、`on_profile_requested_by_ai`、
その後 unknown field の順で最初の1件だけを返します。
known field の duplicate object key は、その field の schema order 位置で
`expected_value = "unique_object_keys"`、`actual_value = "duplicate_field"` として報告します。
unknown field の duplicate object key は unknown field の位置で
`expected_value = "unique_object_keys"`、`actual_value = "duplicate_field"` を返し、
`field` は重複した後続 unknown field の RunnerPolicy-local JSON path にします。
unknown field が複数ある場合は、top-level object、`checker_allowlist[]` entry の小さい index、
`checker_identity_manifest`、`import_policy`、`axiom_policy`、`budgets.<profile>` の bytewise profile key 順で
object を選び、同じ object 内では field name の bytewise lexicographic order で最初の field を返します。
複数の domain failure が同時に存在する場合は、上の table の順序で最初の1件だけを報告します。
required profile の集合が一致しない場合は、順序不一致も同時に起きていても
`profile_set_mismatch` を報告します。

`binary_hash` は実行ファイル bytes の hash です。
`build_hash` は checker build identity です。
runner は `MachineCheckRequest.checker_profile` に対応する
`RunnerPolicy.checker_allowlist` entry を `SelectedCheckerPolicy` として選びます。
checker executable / identity validation order は次で固定します。
各 pre-launch step が failure を返した場合、後続 step と checker launch は行いません。
pre-launch step が成功した場合は、identity manifest の有無に関係なく checker を起動します。

```text
1. SelectedCheckerPolicy.binary_id から executable を解決し、file bytes を読む
   failure: checker_binary_file_unreadable

2. executable file bytes sha256 と SelectedCheckerPolicy.binary_hash を照合する
   failure: checker_binary_hash_mismatch

3. RunnerPolicy.checker_identity_manifest が存在する場合、
   checker identity manifest file を読み、manifest_hash と schema / domain を検査する
   unreadable failure: checker_identity_manifest_file_unreadable
   hash failure: checker_identity_manifest_hash_mismatch
   JSON / schema / domain failure: checker_identity_manifest_invalid

4. checker identity manifest を使う場合、profile entry の存在を確認する
   failure: checker_identity_missing

5. checker identity manifest entry の checker_id / binary_id を照合する
   failure: checker_identity_mismatch

6. checker identity manifest entry の binary_hash を照合する
   failure: checker_binary_hash_mismatch

7. checker identity manifest entry の build_hash を照合する
   failure: checker_build_hash_mismatch

8. checker を起動し、process status と CheckerRawResult JSON を処理する
   failure: checker raw output / process convention の規則に従う

9. CheckerRawResult.checker_id の presence / shape / value を照合する
   missing / wrong type / null / empty string failure: checker_identity_missing
   mismatch failure: checker_identity_mismatch

10. CheckerRawResult.checker_build_hash の presence / shape / value を照合する
   missing / wrong type / null / invalid hash format failure: checker_identity_missing
   mismatch failure: checker_build_hash_mismatch
```

identity manifest は起動前に executable identity を固定する補助入力であり、
起動後の raw identity check を省略する根拠にはしません。
起動後の checker id / build hash mismatch は checker verdict として扱わず、
`error.kind = policy_failure` の `MachineCheckResult` として保存します。
post-launch identity mismatch result の `checker.id` / `checker.build_hash` には、
checker が実際に報告した actual value を記録します。
checker が該当 field を報告できなかった場合は、その field を omit します。

resource limit に到達した場合、checker result は `status = failed`、
`error.kind = resource_exhausted` として保存します。
timeout や memory limit を「flaky」として自動 retry で成功扱いにしてはいけません。
retry を行う場合も、各 attempt を独立した `MachineCheckResult` として保存します。

## 4.2 Runner command construction

runner は policy から checker command を構成します。
request や AI agent が command line を直接構成してはいけません。

MVP の command construction rule：

```text
- cwd は repository root に固定する
- executable は checker_allowlist.binary_id / binary_hash で解決する
- argv[0] は allowlist で解決した executable path
- argv[1..] は checker_allowlist.allowed_args の順序どおり
- その後に runner-owned dynamic args を固定順序で追加する
- certificate path は runner が最後の positional argument として追加する
- stdin は empty
- stdout は CheckerRawResult JSON 専用
- stdout raw bytes は artifact に写さない。malformed output presence は 4.2 の stdout rule だけで token 化する
- stderr は fixed diagnostics token の入力にだけ使い、raw text を verdict / artifact に写さない
- environment は fixed allowlist のみ渡す
- locale は C / UTF-8 fixed
- network access は runner sandbox で禁止する
- extra flags, env vars, cwd override, shell expansion は禁止する
```

allowed_args の順序は semantic identity の一部です。
同じ flag set でも順序が違う command は別 policy として扱い、policy hash も変わります。

`checker_allowlist.allowed_args` は policy が所有する static args です。
import store、trust mode、axiom policy、budget、certificate path は request / policy validation 後に
runner が次の fixed order で追加する dynamic args です。
AI、request、checker raw output がこの順序や flag 名を変えてはいけません。

```text
runner-owned dynamic args:
  --imports <MachineCheckRequest.imports.manifest>
  --imports-hash <MachineCheckRequest.imports.manifest_hash>
  --trust-mode <MachineCheckRequest.trust_mode>
  --axiom-policy <MachineCheckRequest.axiom_policy>
  --axiom-policy-hash <RunnerPolicy.axiom_policy.hash>
  --max-steps <MachineCheckRequest.budget.max_steps>
  --max-memory-mb <MachineCheckRequest.budget.max_memory_mb>
  --timeout-ms <MachineCheckRequest.budget.timeout_ms>
  <MachineCheckRequest.certificate.path>
```

runner は dynamic args の元になる request field を 4 の pre-check で policy と照合済みにします。
checker はこの dynamic args を唯一の import / axiom / budget 入力として扱い、source file、
network、environment variable、current directory scan から追加 input を発見してはいけません。
timeout と memory limit は runner が OS / sandbox でも enforcement します。
`--max-steps` は checker 側の deterministic step budget で、checker が step count を実装できない場合でも
flag は受け取り、resource usage の `steps = 0` として報告します。
MVP runner が checker process に渡す environment は次の3つだけです。

```text
LC_ALL=C.UTF-8
LANG=C.UTF-8
TZ=UTC
```

`PATH`、`HOME`、`TMPDIR`、proxy variable、logging variable、CI variable、user shell environment は
checker process に渡してはいけません。
checker executable は registry で解決した `argv[0]` から直接起動し、`PATH` lookup に依存してはいけません。

MVP の checker executable resolution は runner-owned `CheckerBinaryRegistry` で行います。
registry は AI、request、sidecar から指定できず、`binary_id` から runner-controlled executable path を
一意に返します。
registry entry の path は workspace-relative または runner install root relative のどちらかですが、
どちらの root を使うかは runner registry configuration で決まり、`RunnerPolicy`、request、
sidecar は変更できません。
どちらの場合も shell expansion、`PATH` search、current directory search は使いません。
runner は registry entry path を選択された root に対して1回だけ解決し、全 path component と
symlink 解決後の最終 executable target がその root の内側にあることを確認します。
root 外へ解決される registry entry は `checker_binary_file_unreadable` として扱い、
`field = "checker.binary_id"`、`expected_value = "readable_executable"`、
`actual_value = "unreadable"` を返します。
この場合、runner は executable bytes hash を計算せず、checker を起動してはいけません。
`SelectedCheckerPolicy.binary_hash` と最終 target bytes が一致しうる場合でも、root escape は許可されません。
runner は symlink を解決した最終 target file bytes を読み、
その SHA-256 が `SelectedCheckerPolicy.binary_hash` と一致する場合だけ実行します。
registry path は checker identity ではありません。
identity は `binary_id`、実行 file bytes の `binary_hash`、post-launch `checker_id`、
`checker_build_hash`、および optional checker identity manifest で決まります。

---

# 5. MachineCheckResult

checker が返す結果は、AI が読む前に保存します。
AI が結果本文を書き換えた場合は別 artifact として扱い、正本にはしません。

厳密には、`MachineCheckResult` は checker ではなく runner が作る envelope です。
checker は `CheckerRawResult` 相当の structured JSON を stdout に出してもよいですが、
runner はそれを policy / binary identity / process result と照合し、必ず
`MachineCheckResult` に包んで保存します。

MVP の `CheckerRawResult` schema：

```json
{
  "schema": "npa.phase8.checker_raw_result.v1",
  "checker_id": "npa-checker-ref",
  "checker_version": "0.8.0",
  "checker_build_hash": "sha256:...",
  "status": "failed",
  "module": "Std.Nat",
  "certificate_hash": "sha256:...",
  "export_hash": "sha256:...",
  "axiom_report_hash": "sha256:...",
  "error": {
    "kind": "type_mismatch",
    "declaration": "Nat.add_zero",
    "core_path": ["declarations", 17, "body"],
    "expected_hash": "sha256:...",
    "actual_hash": "sha256:..."
  }
}
```

`CheckerRawResult` は checker の主張であり、正本ではありません。
runner は identity manifest を使う場合でも、起動後に `checker_id` /
`checker_build_hash` を `SelectedCheckerPolicy` と照合し、
process status と矛盾せず identity check も通った場合だけ raw verdict を
`MachineCheckResult` に写します。
`checker_id` mismatch は `policy_failure` であり、
`error.reason_code = checker_identity_mismatch`、
`error.field = "checker.id"` として保存します。
allowlist 側の expected id は `error.expected_value`、checker が報告した actual id は
`error.actual_value` に記録します。

MVP の `CheckerRawResult` は closed-world JSON object です。
top-level `schema = npa.phase8.checker_raw_result.v1` はすべての raw result で required です。
unknown field、duplicate object key、explicit null in non-nullable field、wrong type、
top-level schema mismatch は raw schema failure です。
`CheckerRawResult.module` と `CheckerRawResult.error.declaration` の name grammar violation も
raw schema failure であり、`checker_module_mismatch` ではありません。
ただし `checker_id`、`checker_build_hash`、`checker_version` は下の identity / metadata
例外規則を優先します。
duplicate object key と unknown field はこの例外対象ではなく、常に raw schema failure です。
raw schema failure は checker exit code に応じて `malformed_success_output`、
`malformed_rejection_output`、または `malformed_internal_error_output` に分類します。
invalid JSON の raw schema failure では `MachineCheckResult.error.field = "checker_raw"`、
`expected_value = "valid_json"`、`actual_value = "invalid_json"` を入れます。
JSON parse 後の raw schema failure では `MachineCheckResult.error.field` に
`checker_raw.<CheckerRawResult JSON path>` を入れ、root-level failure では
`checker_raw` を使います。
`expected_value` には raw schema requirement 名、
`actual_value` には `missing`、`wrong_type`、`unknown_field`、`invalid_enum`、
`invalid_hash_format`、`invalid_name_format`、`invalid_path`、`integer_out_of_range`、
`null_not_allowed`、`duplicate_field`、`forbidden_field` のいずれかを入れます。
複数の raw schema failure が同時に存在する場合は、top-level non-object、`schema`、
`status`、valid な `status` から決まる minimal fields、status-dependent forbidden fields、
status-dependent optional top-level hash fields、status-dependent `error` members、unknown field の順で
最初の1件だけを返します。
valid な `status` から決まる minimal fields は次の順です。
`status = checked` では `module`、`certificate_hash`、`export_hash`、`axiom_report_hash` の順です。
`status = failed` では、まず `error` object、`error.kind` を検査します。
failed raw result では valid な `error.kind` を読めるまで `module` missing を報告してはいけません。
`CheckerRawResult.error.kind` は checker-originated failure kind だけを許可する closed enum です。
raw checker が直接出してよい kind group は次に限定します。

```text
decode / schema / noncanonical failure:
  certificate_decode_error
  noncanonical_encoding
  unsupported_schema_version

ordinary failure kind:
  import_not_found
  import_hash_mismatch
  certificate_hash_mismatch
  axiom_report_mismatch
  export_hash_mismatch
  type_mismatch
  conversion_failure
  universe_inconsistency
  inductive_invalid
  positivity_failure
  declaration_hash_mismatch
  dependency_hash_mismatch
  forbidden_axiom

checker internal error:
  checker_internal_error
```

`policy_failure`、`resource_exhausted`、`timeout` は runner-owned `MachineCheckResult.error.kind`
であり、`CheckerRawResult.error.kind` としては forbidden です。
raw output がこれら、または上の一覧外の kind を出した場合は raw schema failure とし、
`field = "checker_raw.error.kind"`、`expected_value = "checker_raw_error_kind"`、
`actual_value = "invalid_enum"` を返します。
`error.kind` が schema-valid に読めた後でだけ、その kind group に応じた non-error required fields を検査します。
ordinary failure kind では `module`、`certificate_hash` の順で検査し、checker internal error では
`error.reason_code` を検査します。
decode / schema / noncanonical failure では追加の required field はありません。
decode / schema / noncanonical failure と checker internal error の `module` は optional です。
存在する場合だけ name grammar を検査し、missing を raw schema failure として報告してはいけません。
したがって `status = failed` で `error.kind` と `module` / `certificate_hash` が同時に missing の場合は
`checker_raw.error.kind` を先に返します。
`error.kind` が ordinary failure kind として schema-valid に読めた後で `module` と
`certificate_hash` が両方 missing の場合は `checker_raw.module` を先に返します。
`error` member 自体が missing / null / wrong type の場合は `checker_raw.error` を返し、
その後に `checker_raw.error.kind` missing を合成してはいけません。
`error` が object として valid な場合だけ、`error.kind` と、
checker internal error に限り required `error.reason_code` を検査します。
raw checker internal error の `error.reason_code` は checker-originated internal error を表す
closed enum で、MVP では `checker_reported_internal_error` だけを許可します。
missing / null / wrong type の場合は `field = "checker_raw.error.reason_code"`、
`expected_value = "checker_raw_internal_reason_code"`、
`actual_value = "missing"` / `"null_not_allowed"` / `"wrong_type"` を返します。
`malformed_success_output`、`malformed_rejection_output`、`malformed_internal_error_output`、
`process_exit_failure`、`checker_module_mismatch` などの runner-owned reason code、
または一覧外の string を raw checker が出した場合は
`field = "checker_raw.error.reason_code"`、
`expected_value = "checker_raw_internal_reason_code"`、
`actual_value = "invalid_enum"` の raw schema failure です。
raw `CheckerRawResult` は checker-originated kind について 5 の
`MVP の error kind ごとの field requirement` と同じ kind-specific required error member を要求します。
この検査は raw schema priority の `status-dependent error members` phase で行い、
status-dependent optional top-level hash fields の検査より前に繰り上げません。
`import_hash_mismatch`、`certificate_hash_mismatch`、`axiom_report_mismatch`、
`export_hash_mismatch` では `error.expected_hash`、`error.actual_hash` の順で required です。
これらが missing / null / wrong type / invalid hash format の場合は raw schema failure とし、
`field = "checker_raw.error.expected_hash"` または `"checker_raw.error.actual_hash"`、
`expected_value = "sha256:<lower-hex>"`、
`actual_value = "missing"` / `"null_not_allowed"` / `"wrong_type"` /
`"invalid_hash_format"` を返します。
`import_not_found` の `error.expected_hash` と、その他の ordinary failure kind の
`error.expected_hash` / `error.actual_hash` は required ではなく optional です。
status-dependent optional top-level hash fields は `certificate_hash`、`export_hash`、
`axiom_report_hash` の順で検査します。
その status / kind group で optional と明記された hash field だけをここで許可し、
それ以外の top-level hash field が存在する場合は status-dependent forbidden field として先に返します。
forbidden field は値の shape を検査せず、`field = "checker_raw.<field>"`、
`expected_value = "absent_for_status_kind"`、`actual_value = "forbidden_field"` とします。
複数の status-dependent forbidden fields がある場合は `error`、`certificate_hash`、
`export_hash`、`axiom_report_hash` の順で最初の field を返します。
したがって raw `checker_internal_error` に malformed な `export_hash` が存在しても、
`invalid_hash_format` ではなく `checker_raw.export_hash` の `forbidden_field` を返します。
`checker_id`、`checker_build_hash`、`checker_version` は identity / metadata 例外規則に従い、
この raw schema priority の optional top-level hash fields には含めません。
status-dependent `error` member validation では、上記の kind-specific required error member を
optional member より先に検査し、その後 `error.reason_code`、`error.declaration`、
`error.core_path`、`error.expected_hash`、`error.actual_hash`、`error.section`、`error.offset`
の順で forbidden member と allowed optional member の value shape を検査します。
現在の `error.kind` group で required / optional と明記されていない `error` member は forbidden です。
forbidden nested error member は値の shape を検査せず、
`field = "checker_raw.error.<member>"`、`expected_value = "absent_for_error_kind"`、
`actual_value = "forbidden_field"` とします。
ordinary failure kind では `error.declaration`、`error.core_path`、`error.expected_hash`、
`error.actual_hash` だけが optional で、`error.reason_code`、`error.section`、`error.offset`
は forbidden です。
decode / schema / noncanonical failure では `error.section`、`error.offset` だけが optional で、
`error.reason_code`、`error.declaration`、`error.core_path`、`error.expected_hash`、
`error.actual_hash` は forbidden です。
checker internal error では `error.reason_code` だけが required で、その他の `error` member は forbidden です。
allowed optional `error` member の value shape は structured error field value shape に従います。
malformed な allowed optional `error` member の `expected_value` / `actual_value` は次で固定します。

```text
error.declaration:
  expected_value = "phase2_name"
  actual_value = wrong_type | null_not_allowed | invalid_name_format

error.core_path:
  expected_value = "core_path"
  actual_value = wrong_type | null_not_allowed | invalid_path

error.expected_hash / error.actual_hash:
  expected_value = "sha256:<lower-hex>"
  actual_value = wrong_type | null_not_allowed | invalid_hash_format

error.section:
  expected_value = "section_token"
  actual_value = wrong_type | null_not_allowed | invalid_name_format

error.offset:
  expected_value = "non_negative_i64"
  actual_value = wrong_type | null_not_allowed | integer_out_of_range
```

allowed optional `error` member が複数 malformed な場合は、上の順で最初の field を返します。
known field の duplicate object key は、その field の raw schema order 位置で
`expected_value = "unique_object_keys"`、`actual_value = "duplicate_field"` として報告します。
unknown field の duplicate object key は unknown field の位置で報告し、
同じ object 内では field name の bytewise lexicographic order で最初の field を返します。
`CheckerRawResult` 自体は保存正本 artifact ではないため `result_hash` を持ちません。

`CheckerRawResult` の required / optional field：

```text
status = checked:
  required:
    - schema
    - status
    - module
    - certificate_hash
    - export_hash
    - axiom_report_hash
  identity-checked:
    - checker_id
    - checker_build_hash
  optional metadata:
    - checker_version
  forbidden:
    - error

ordinary status = failed:
  required:
    - schema
    - status
    - module
    - certificate_hash
    - error.kind
  kind-dependent error fields:
    - see MVP error kind field requirement table
  identity-checked:
    - checker_id
    - checker_build_hash
  optional:
    - checker_version
    - export_hash
    - axiom_report_hash
    - error.declaration
    - error.core_path
    - error.expected_hash, when not kind-required
    - error.actual_hash, when not kind-required

decode / schema / noncanonical failure:
  required:
    - schema
    - status = failed
    - error.kind
  identity-checked:
    - checker_id
    - checker_build_hash
  optional:
    - checker_version
    - certificate_hash
    - module
    - error.section
    - error.offset

checker internal error:
  required:
    - schema
    - status = failed
    - error.kind = checker_internal_error
    - error.reason_code
  identity-checked:
    - checker_id
    - checker_build_hash
  optional:
    - checker_version
    - certificate_hash
    - module
```

`identity-checked` field は generic raw schema required field ではありません。
raw JSON が parse でき、`status` / `error.kind` などの minimal raw shape を読めた後、
runner が 4.1 の checker executable / identity validation order で検査します。
`checker_id` が missing / wrong type / null / empty string の場合、または
`checker_build_hash` が missing / wrong type / null / invalid hash format の場合、
generic `malformed_*` ではなく `policy_failure` の `checker_identity_missing` に分類します。
`checker_id` が valid string だが allowlist と一致しない場合は
`checker_identity_mismatch`、`checker_build_hash` が valid hash だが allowlist と一致しない場合は
`checker_build_hash_mismatch` に分類します。
`checker_version` は optional metadata です。
missing の場合は `MachineCheckResult.checker.version` を omit し、
wrong type / null の場合は `diagnostics` にだけ fixed token として記録して raw verdict 採用の可否に使いません。
wrong type では `checker_raw.checker_version:wrong_type`、
null では `checker_raw.checker_version:null_not_allowed` を使います。
raw `checker_internal_error` に schema-valid な `certificate_hash` が存在し、
process convention と checker identity check も通った場合、runner は
`MachineCheckResult.certificate_hash` にその値を写してよいです。
存在しない場合は omit します。
raw `checker_internal_error` の `certificate_hash` は required ではありませんが、
存在して malformed な場合は raw schema failure です。
MVP の ordinary failure kind はすべて canonical certificate hash recomputation 後の failure として扱うため、
raw `certificate_hash` は常に required です。
将来 before-recompute ordinary failure kind を追加する場合は schema version を上げ、
kind group と required field rule を明示します。

`MachineCheckResult.module` はすべての result で required です。
runner が常に `MachineCheckRequest.module` から埋め、checker raw output の `module` を
正本 source として使ってはいけません。
`CheckerRawResult.module` が存在し、`MachineCheckRequest.module` と一致しない場合、
runner は raw output を正本 verdict として写さず、
`status = failed`、`error.kind = checker_internal_error`、
`error.reason_code = checker_module_mismatch` の `MachineCheckResult` を保存します。
この判定は `CheckerRawResult.module` が 3.3 の name grammar validation を通った場合だけ行います。
name grammar validation に失敗した raw output は raw schema failure として扱います。
このとき `error.field = "checker_raw.module"`、
`error.expected_value = "module_name"`、
`error.actual_value = "invalid_name_format"` にします。
`CheckerRawResult.module` が schema-valid だが request module と bytewise に一致しない場合だけ、
`checker_module_mismatch` として `error.field = "module"`、
`error.expected_value = MachineCheckRequest.module`、
`error.actual_value = CheckerRawResult.module` を使います。
checker identity が allowlist と一致する場合は `checker.id`、`checker.build_hash`、
`checker.binary_id`、`checker.binary_hash`、`checker.profile` を記録します。
raw `checker_version` が valid string の場合だけ `checker.version` も記録します。
checker identity が allowlist と一致しない場合は identity mismatch の `policy_failure` を優先します。
`checker_module_mismatch` validation は raw schema validation と checker identity validation の後、
exit-code convention による raw checker verdict 採用の前に行います。
exit 2 の raw checker internal error がそれ以外は valid で、schema-valid な `module` が
`MachineCheckRequest.module` と異なる場合は、`checker_reported_internal_error` の採用より
`checker_module_mismatch` を優先します。
raw `module` が missing の場合、module mismatch validation は行いません。
`checker_module_mismatch` では raw verdict に含まれる `certificate_hash`、
`export_hash`、`axiom_report_hash` は `MachineCheckResult` に写しません。

MVP の `MachineCheckResult` top-level required field：

```text
- schema
- request_id
- request_hash
- result_id
- result_hash
- run_artifact_hash
- policy
- runner
- checker
- attempt
- status
- module
- process
- resource_usage
```

`checker.profile` は常に required です。
`checker.binary_id` と `checker.binary_hash` は、checker process を起動した場合に required です。
`checker.id` と `checker.build_hash` は、それぞれ対応する raw identity field を
syntactically valid な値として読めた場合だけ required です。
`checker.version` は optional metadata であり、raw `checker_version` が valid string の場合だけ記録します。
`process.launched = true` で `checker.id` または `checker.build_hash` を確定できず、
`error.kind` が checker infrastructure failure でもない場合は `policy_failure` です。
`status = checked` では `certificate_hash`、`export_hash`、`axiom_report_hash` が required で、
`error` は forbidden です。
`status = failed` では `error` が required です。
`certificate_hash`、`export_hash`、`axiom_report_hash` の failed 時の required / optional は
次の error kind ごとの規則に従います。
`diagnostics`、`axioms_used`、`declarations_checked` は optional metadata です。
`diagnostics` の型と empty 表現は 3.3 の共通規則に従い、診断がない場合は omit します。

MVP の `MachineCheckResult.runner` schema：

```text
required:
  - id
  - version
  - build_hash

unknown field:
  forbidden
```

`runner.id` と `runner.version` は non-empty string です。
`runner.build_hash` は runner build identity の `sha256:<lower-hex>` hash です。

MVP の `MachineCheckResult.checker` schema：

```text
required:
  - profile

required when checker process is launched:
  - binary_id
  - binary_hash

independently optional checker raw identity copy fields:
  - id
  - build_hash

optional:
  - version

unknown field:
  forbidden
```

`checker.profile` は request の `checker_profile` を常に写します。
`checker.binary_hash` は runner が実行した binary bytes の sha256 です。
checker を起動していない result では `profile` 以外の checker identity field を omit します。
checker process を起動した result では、raw identity の採用可否に関係なく runner が起動した
executable の `binary_id` / `binary_hash` を記録します。
raw JSON 自体を decode / parse できない場合は、checker が自己申告する
`id` / `version` / `build_hash` をすべて omit します。
raw JSON を parse でき、`checker_id` または `checker_build_hash` の syntactic value を読める場合は、
allowlist / profile と一致しなくても読めた actual value だけを
`checker.id` / `checker.build_hash` に field ごとに記録します。
missing / wrong type / null / invalid hash format の identity field は、その field だけ omit します。
この場合でも allowlist と一致しなかった checker raw verdict は正本 verdict として写しません。
`checker.id` と `checker.build_hash` は schema 上は独立 optional field です。
片方だけ存在する `MachineCheckResult` も schema valid ですが、accepted checker verdict にはなりません。
`status = checked`、または checker raw verdict をそのまま採用した non-policy `status = failed` では、
`checker.id` と `checker.build_hash` の両方が存在し、`SelectedCheckerPolicy` と一致していなければなりません。
片方または両方が missing / malformed / mismatch の launched result は
`error.kind = policy_failure`、`error.reason_code = checker_identity_missing`、
`checker_identity_mismatch`、または `checker_build_hash_mismatch` とし、
読めた actual identity field だけを `checker` object に残します。

`attempt` は positive integer で、同じ `(request_hash, checker.profile)` の
runner execution attempt ごとに 1 から単調増加します。
retry しない通常実行では `attempt = 1` です。
MVP の `npa-check run` と `/machine/check/certificate` は result store を更新しない
stateless execution なので、caller が attempt を指定しない場合は常に `attempt = 1` を書きます。
store-aware orchestrator が複数 attempt を同じ append-only result store に保存する場合だけ、
orchestrator が次の positive integer を決め、CLI では `--attempt <n>`、
API では wrapper object の top-level `attempt` で runner に渡します。
runner は result store を scan して採番してはいけません。
`attempt` の単調増加は result store ingestion の invariant であり、
stateless `npa-check run` 単体の validation ではありません。
`attempt` は `result_hash` から除外し、`run_artifact_hash` には含めます。

MVP の `MachineCheckResult.process` schema：

```text
required:
  - launched

required when launched = true and an exit status is available:
  - exit_code

required when launched = true and exit_code is omitted:
  - termination_reason

forbidden when launched = true and exit_code is present:
  - termination_reason

forbidden when launched = false:
  - exit_code
  - termination_reason

unknown field:
  forbidden
```

`process.launched` は boolean です。
`process.exit_code` は 0 以上 255 以下の integer です。
`termination_reason` は `timeout`、`resource_exhausted`、`killed_without_exit_status` の
いずれかです。
`termination_reason` は OS / runner が exit status を得られなかった場合だけ使い、
`exit_code` と同時に出してはいけません。
post-launch timeout / resource exhaustion で checker process が exit code を返さない場合は、
`exit_code` を omit し、対応する `termination_reason` を入れます。
`termination_reason = killed_without_exit_status` は
`error.kind = checker_internal_error`、`error.reason_code = process_exit_failure`、
`error.field = "process.termination_reason"`、`error.actual_value = "killed_without_exit_status"`
として記録します。
この場合 `process.launched = true`、`process.exit_code` は omit します。

MVP の `MachineCheckResult.resource_usage` schema：

```text
required:
  - steps
  - memory_peak_mb
  - elapsed_ms

unknown field:
  forbidden
```

`steps`、`memory_peak_mb`、`elapsed_ms` は non-negative integer です。
`memory_peak_mb` は MiB を切り上げた整数です。
`elapsed_ms` は runner が観測した wall-clock elapsed milliseconds を切り上げた整数です。
checker が deterministic step count を報告できない場合、runner は `steps = 0` を記録します。
checker を起動していない result では `steps = 0`、`memory_peak_mb = 0`、`elapsed_ms = 0` です。
これらの resource metadata は `result_hash` から除外し、`run_artifact_hash` には含めます。

MVP の error kind ごとの field requirement：

```text
certificate_decode_error:
  group: decode / schema / noncanonical failure
  required error fields: kind
  optional error fields: section, offset
  certificate_hash: optional

noncanonical_encoding:
  group: decode / schema / noncanonical failure
  required error fields: kind
  optional error fields: section, offset
  certificate_hash: optional

unsupported_schema_version:
  group: decode / schema / noncanonical failure
  required error fields: kind
  certificate_hash: optional

import_not_found:
  group: ordinary status = failed
  required error fields: kind
  optional error fields: expected_hash
  certificate_hash: required

import_hash_mismatch:
  group: ordinary status = failed
  required error fields: kind, expected_hash, actual_hash
  certificate_hash: required

certificate_hash_mismatch:
  group: ordinary status = failed
  required error fields: kind, expected_hash, actual_hash
  certificate_hash: required
  invariant: error.expected_hash = request.certificate.expected_certificate_hash
             error.actual_hash = MachineCheckResult.certificate_hash

axiom_report_mismatch:
  group: ordinary status = failed
  required error fields: kind, expected_hash, actual_hash
  certificate_hash: required

export_hash_mismatch:
  group: ordinary status = failed
  required error fields: kind, expected_hash, actual_hash
  certificate_hash: required

type_mismatch / conversion_failure / universe_inconsistency / inductive_invalid /
positivity_failure / declaration_hash_mismatch / dependency_hash_mismatch /
forbidden_axiom:
  group: ordinary status = failed
  required error fields: kind
  optional error fields: declaration, core_path, expected_hash, actual_hash
  certificate_hash: required

policy_failure:
  group: runner policy / identity failure
  required error fields: kind, reason_code
  optional error fields: field, expected_hash, actual_hash, expected_value, actual_value
  certificate_hash: omitted

checker_internal_error / resource_exhausted / timeout:
  group: runner or checker infrastructure failure
  required error fields: kind, reason_code
  certificate_hash: optional
```

MVP の `MachineCheckResult.error.reason_code` は closed enum です。
この文書に列挙されていない reason code を producer が追加してはいけません。

```text
policy_failure:
  runner_policy_reference_invalid
  runner_policy_file_unreadable
  runner_policy_hash_mismatch
  runner_policy_invalid
  request_policy_hash_mismatch
  request_trust_mode_mismatch
  request_checker_profile_not_allowed
  request_axiom_policy_mismatch
  request_axiom_policy_file_unreadable
  request_axiom_policy_hash_mismatch
  request_budget_mismatch
  request_import_mode_mismatch
  request_import_manifest_file_unreadable
  request_import_manifest_hash_mismatch
  request_import_manifest_invalid
  request_certificate_file_unreadable
  request_certificate_file_hash_mismatch
  checker_binary_file_unreadable
  checker_binary_hash_mismatch
  checker_identity_manifest_file_unreadable
  checker_identity_manifest_hash_mismatch
  checker_identity_manifest_invalid
  checker_build_hash_mismatch
  checker_identity_mismatch
  checker_identity_missing

checker_internal_error:
  checker_reported_internal_error
  malformed_success_output
  success_exit_status_mismatch
  missing_rejection_error
  malformed_rejection_output
  malformed_internal_error_output
  checker_module_mismatch
  process_exit_failure

timeout:
  launch_timeout
  checker_timeout

resource_exhausted:
  launch_resource_exhausted
  checker_resource_exhausted
```

これ以外の deterministic checker rejection では、reason code を使わず `error.kind`
と hash / value field だけで分類します。

checker raw output の異常時処理：

```text
exit 0 + invalid JSON or raw schema failure:
  status = failed, error.kind = checker_internal_error,
  error.reason_code = malformed_success_output

exit 0 + status != checked:
  status = failed, error.kind = checker_internal_error,
  error.reason_code = success_exit_status_mismatch

exit 1 + missing structured error:
  status = failed, error.kind = checker_internal_error,
  error.reason_code = missing_rejection_error

exit 1 + invalid JSON or raw schema failure:
  status = failed, error.kind = checker_internal_error,
  error.reason_code = malformed_rejection_output

exit 1 + status != failed:
  status = failed, error.kind = checker_internal_error,
  error.reason_code = missing_rejection_error

exit 1 + status = failed + error.kind = checker_internal_error:
  status = failed, error.kind = checker_internal_error,
  error.reason_code = malformed_rejection_output

exit 2 + invalid JSON or raw schema failure:
  status = failed, error.kind = checker_internal_error,
  error.reason_code = malformed_internal_error_output

exit 2 + error.kind != checker_internal_error:
  status = failed, error.kind = checker_internal_error,
  error.reason_code = malformed_internal_error_output

exit >= 3:
  status = failed, error.kind = checker_internal_error,
  error.reason_code = process_exit_failure.
  The runner must not copy stdout or stderr excerpts into diagnostics for exit >= 3.
  For this case the MachineCheckResult.diagnostics field must be omitted, not an empty
  array and not a fixed literal entry.

raw module mismatch:
  status = failed, error.kind = checker_internal_error,
  error.reason_code = checker_module_mismatch,
  error.field = "module",
  error.expected_value = MachineCheckRequest.module,
  error.actual_value = CheckerRawResult.module

stderr:
  raw bytes are never copied into MachineCheckResult.diagnostics in the MVP.
  Use fixed token checker_process:stderr_present only when stderr is non-empty and
  the checker process returned exit code 0, 1, or 2.
  This includes checked / failed raw verdicts, malformed raw output, status mismatch,
  missing structured error, raw module mismatch, and checker-reported internal error.
  Do not use checker_process:stderr_present for pre-launch failures, termination
  without exit status, or exit >= 3; for exit >= 3 diagnostics is omitted.
  never used as verdict

stdout:
  raw bytes are parsed only as CheckerRawResult JSON and are never copied into
  MachineCheckResult.diagnostics in the MVP.
  Use fixed token checker_process:stdout_present only when stdout is non-empty and
  the exit-code rule is one of:
    exit 0 + invalid JSON or raw schema failure,
    exit 1 + invalid JSON or raw schema failure,
    exit 2 + invalid JSON or raw schema failure.
  Do not use checker_process:stdout_present for status mismatch, missing structured
  error, raw module mismatch, or exit >= 3; for exit >= 3 diagnostics is omitted.
```

raw JSON を parse できない場合、runner は `CheckerRawResult` の required field を
満たせないため、checker raw identity を信用しません。
この場合の `MachineCheckResult.checker` には request の `checker_profile` と、
runner が起動した executable の `binary_id` / `binary_hash` だけを記録します。
`checker.id`、`checker.version`、`checker.build_hash` は omit します。

checker process の exit code convention：

```text
0:
  deterministic acceptance, MachineCheckResult.status = checked

1:
  deterministic certificate rejection with structured checker error,
  MachineCheckResult.status = failed

2:
  checker internal error with structured checker error,
  MachineCheckResult.status = failed, error.kind = checker_internal_error,
  error.reason_code = checker_reported_internal_error

3 or larger:
  process-level failure; runner records checker_internal_error with
  error.reason_code = process_exit_failure unless a more specific runner error kind applies

not launched:
  runner policy failure; MachineCheckResult.status = failed,
  error.kind = policy_failure

launch timeout:
  runner timeout before checker process launch; MachineCheckResult.status = failed,
  error.kind = timeout, error.reason_code = launch_timeout,
  process.launched = false

launch resource exhausted:
  runner resource failure before checker process launch; MachineCheckResult.status = failed,
  error.kind = resource_exhausted, error.reason_code = launch_resource_exhausted,
  process.launched = false

post-launch identity mismatch:
  runner policy failure; MachineCheckResult.status = failed,
  error.kind = policy_failure, process.launched = true,
  error.reason_code = checker_build_hash_mismatch or checker_identity_mismatch
                      or checker_identity_missing

timeout:
  runner-enforced wall-clock timeout after checker process launch;
  MachineCheckResult.status = failed,
  error.kind = timeout, error.reason_code = checker_timeout,
  process.launched = true

resource exhausted:
  runner-enforced resource exhaustion after checker process launch;
  MachineCheckResult.status = failed,
  error.kind = resource_exhausted,
  error.reason_code = checker_resource_exhausted,
  process.launched = true
```

成功時：

```json
{
  "schema": "npa.phase8.machine_check_result.v1",
  "request_id": "mchkreq_001",
  "request_hash": "sha256:...",
  "result_id": "mchkres_001",
  "result_hash": "sha256:...",
  "run_artifact_hash": "sha256:...",
  "policy": {
    "id": "phase8-pr",
    "version": 1,
    "hash": "sha256:..."
  },
  "runner": {
    "id": "npa-check-runner",
    "version": "0.8.0",
    "build_hash": "sha256:..."
  },
  "checker": {
    "id": "npa-checker-ref",
    "version": "0.8.0",
    "binary_id": "npa-checker-ref-macos-aarch64",
    "binary_hash": "sha256:...",
    "build_hash": "sha256:...",
    "profile": "reference"
  },
  "attempt": 1,
  "status": "checked",
  "module": "Std.Nat",
  "certificate_hash": "sha256:...",
  "export_hash": "sha256:...",
  "axiom_report_hash": "sha256:...",
  "axioms_used": [],
  "declarations_checked": 128,
  "process": {
    "launched": true,
    "exit_code": 0
  },
  "resource_usage": {
    "steps": 123456,
    "memory_peak_mb": 256,
    "elapsed_ms": 1732
  }
}
```

失敗時：

```json
{
  "schema": "npa.phase8.machine_check_result.v1",
  "request_id": "mchkreq_002",
  "request_hash": "sha256:...",
  "result_id": "mchkres_002",
  "result_hash": "sha256:...",
  "run_artifact_hash": "sha256:...",
  "policy": {
    "id": "phase8-pr",
    "version": 1,
    "hash": "sha256:..."
  },
  "runner": {
    "id": "npa-check-runner",
    "version": "0.8.0",
    "build_hash": "sha256:..."
  },
  "checker": {
    "id": "npa-checker-ref",
    "version": "0.8.0",
    "binary_id": "npa-checker-ref-macos-aarch64",
    "binary_hash": "sha256:...",
    "build_hash": "sha256:...",
    "profile": "reference"
  },
  "attempt": 1,
  "status": "failed",
  "module": "Std.Nat",
  "certificate_hash": "sha256:...",
  "error": {
    "kind": "type_mismatch",
    "declaration": "Nat.add_zero",
    "core_path": ["declarations", 17, "body"],
    "expected_hash": "sha256:...",
    "actual_hash": "sha256:..."
  },
  "process": {
    "launched": true,
    "exit_code": 1
  },
  "resource_usage": {
    "steps": 123456,
    "memory_peak_mb": 256,
    "elapsed_ms": 1732
  }
}
```

`request_hash` は 3.3 で定義した通り、`request_id` と `request_hash` field を除いた
`MachineCheckRequest` の canonical serialization hash です。
`result_hash` と `run_artifact_hash` の hash 対象は 3.3 の規則に従います。
runner は checker stdout をそのまま信用せず、次を照合してから result を保存します。

```text
- request_hash が実行した request と一致する
- policy.hash が runner に読み込まれた policy と一致する
- checker id / binary_hash / build_hash が policy allowlist と一致する
- checker profile が policy の required / allowed profile と一致する
- certificate_hash が request.certificate.expected_certificate_hash と一致する、
  または error.kind = certificate_hash_mismatch の invariant を満たす
- process exit status と checker status が矛盾しない
```

`certificate_hash` の照合を skip できるのは、checker が canonical certificate hash を
再計算できない段階で失敗した場合だけです。

MVP の skip 対象：

```text
- policy_failure
- certificate_decode_error
- noncanonical_encoding
- unsupported_schema_version
- checker_internal_error
- resource_exhausted
- timeout
```

それ以外の `status = checked` または `status = failed` では、runner は
`MachineCheckResult.certificate_hash` の存在を要求します。
`error.kind = certificate_hash_mismatch` 以外では、
`MachineCheckResult.certificate_hash == request.certificate.expected_certificate_hash` を要求します。
`error.kind = certificate_hash_mismatch` では、
`MachineCheckResult.certificate_hash != request.certificate.expected_certificate_hash` と、
上記の error field invariant を要求します。

`MachineCheckResult.certificate_hash` は checker が再計算した canonical certificate hash です。
`.npcert` file bytes の hash ではなく、certificate 内に格納された claimed hash でもありません。
file bytes hash が必要な場合は request / audit bundle / challenge manifest 側の `file_hash` に記録します。
request の `certificate.expected_certificate_hash` と recomputed hash が違う場合、runner は
`error.kind = certificate_hash_mismatch` を返します。
このとき `MachineCheckResult.certificate_hash` は recomputed hash、
`error.expected_hash` は request の expected hash、
`error.actual_hash` は recomputed hash です。

偽の `MachineCheckResult` を避けるため、CI / release では result artifact を
runner が append-only storage に書き込みます。
外部から持ち込まれた result file は、`result_hash`、`request_hash`、policy hash、
`run_artifact_hash`、runner identity、checker allowlist 照合が通らない限り正本として扱いません。

policy failure の例：

```json
{
  "schema": "npa.phase8.machine_check_result.v1",
  "request_id": "mchkreq_bad_policy_001",
  "request_hash": "sha256:...",
  "result_id": "mchkres_policy_failure_001",
  "result_hash": "sha256:...",
  "run_artifact_hash": "sha256:...",
  "policy": {
    "id": "phase8-pr",
    "version": 1,
    "hash": "sha256:..."
  },
  "runner": {
    "id": "npa-check-runner",
    "version": "0.8.0",
    "build_hash": "sha256:..."
  },
  "checker": {
    "profile": "reference"
  },
  "attempt": 1,
  "status": "failed",
  "module": "Std.Nat",
  "error": {
    "kind": "policy_failure",
    "reason_code": "request_budget_mismatch",
    "field": "budget.timeout_ms",
    "expected_value": 60000,
    "actual_value": 120000
  },
  "process": {
    "launched": false
  },
  "resource_usage": {
    "steps": 0,
    "memory_peak_mb": 0,
    "elapsed_ms": 0
  }
}
```

`process.launched = false` の result では、`checker.id`、`checker.binary_id`、
`checker.binary_hash`、`checker.build_hash` は存在しないことがあります。
ただし `checker.profile` は request で指定された profile を必ず記録します。
同様に、checker が起動していない場合や decode 前に失敗した場合は、
`certificate_hash` が存在しないことがあります。
post-launch identity mismatch の `policy_failure` では `process.launched = true` にし、
runner が起動した executable の `binary_id` / `binary_hash` と request の `checker.profile` を記録します。
checker が syntactically valid な `checker_id` / `checker_build_hash` を報告した場合は、
allowlist と一致しない場合でも actual 値を `checker.id` / `checker.build_hash` に
field ごとに記録します。
missing / malformed な identity field は、その field だけ omit します。
allowlist の expected 値は `error.expected_hash`、actual 値は `error.actual_hash` に記録します。
`checker.id` mismatch では `error.expected_value` / `error.actual_value` を使い、
hash field には入れません。
`checker.id` mismatch の場合でも、`checker.id` には checker が報告した actual id を記録します。
checker が id を報告できなかった場合は `checker.id` を omit し、
`error.reason_code = checker_identity_missing` とします。
ただし allowlist と一致しなかった checker raw verdict は正本 verdict として写しません。

AI 用に特に重要なのは、`error.kind` を structured enum にすることです。
AI が自然言語ログから失敗理由を推測しなくてよいようにします。

MVP の error kind：

```text
- certificate_decode_error
- noncanonical_encoding
- unsupported_schema_version
- import_not_found
- import_hash_mismatch
- declaration_hash_mismatch
- dependency_hash_mismatch
- type_mismatch
- conversion_failure
- universe_inconsistency
- inductive_invalid
- positivity_failure
- axiom_report_mismatch
- forbidden_axiom
- export_hash_mismatch
- certificate_hash_mismatch
- policy_failure
- checker_internal_error
- resource_exhausted
- timeout
```

この list は `MachineCheckResult.error.kind` 専用です。
`MachineCheckRequestErrorResult`、`NormalizeErrorResult`、`CompareValidationResult`、
`AuditSidecarValidationResult`、`AuxiliaryResult`、`ChallengeReplayResult` は
それぞれ別の artifact / response schema であり、別の `error.kind` enum を持ちます。
別 schema で同じ文字列を使う場合も、それは checker verdict ではなくその schema の失敗分類です。

---

# 6. NormalizedCheckResult

複数 checker の出力は、実装言語やエラー表現が異なります。
AI Profile では比較のために正規化します。

```json
{
  "schema": "npa.phase8.normalized_check_result.v1",
  "normalized_result_id": "norm_Std.Nat_001",
  "normalized_result_hash": "sha256:...",
  "artifact": {
    "module": "Std.Nat",
    "input_file_hash": "sha256:...",
    "expected_certificate_hash": "sha256:...",
    "import_lock_hash": "sha256:...",
    "axiom_policy_hash": "sha256:..."
  },
  "artifact_hash": "sha256:...",
  "policy": {
    "id": "phase8-release",
    "version": 1,
    "hash": "sha256:..."
  },
  "results": [
    {
      "result_id": "mchkres_fast_001",
      "result_hash": "sha256:...",
      "request_hash": "sha256:...",
      "policy_hash": "sha256:...",
      "artifact_hash": "sha256:...",
      "checker_id": "npa-fast-kernel",
      "checker_binary_hash": "sha256:...",
      "checker_build_hash": "sha256:...",
      "checker_profile": "fast-kernel",
      "process_launched": true,
      "status": "checked",
      "certificate_hash": "sha256:...",
      "export_hash": "sha256:...",
      "axiom_report_hash": "sha256:..."
    },
    {
      "result_id": "mchkres_ref_001",
      "result_hash": "sha256:...",
      "request_hash": "sha256:...",
      "policy_hash": "sha256:...",
      "artifact_hash": "sha256:...",
      "checker_id": "npa-checker-ref",
      "checker_binary_hash": "sha256:...",
      "checker_build_hash": "sha256:...",
      "checker_profile": "reference",
      "process_launched": true,
      "status": "checked",
      "certificate_hash": "sha256:...",
      "export_hash": "sha256:...",
      "axiom_report_hash": "sha256:..."
    },
    {
      "result_id": "mchkres_ext_001",
      "result_hash": "sha256:...",
      "request_hash": "sha256:...",
      "policy_hash": "sha256:...",
      "artifact_hash": "sha256:...",
      "checker_id": "npa-checker-ext",
      "checker_binary_hash": "sha256:...",
      "checker_build_hash": "sha256:...",
      "checker_profile": "external",
      "process_launched": true,
      "status": "checked",
      "certificate_hash": "sha256:...",
      "export_hash": "sha256:...",
      "axiom_report_hash": "sha256:..."
    }
  ],
  "comparison": {
    "status": "all_agree_checked",
    "matching_fields": [
      "certificate_hash",
      "export_hash",
      "axiom_report_hash"
    ],
    "missing_checker_profiles": [],
    "disagreements": [],
    "status_reasons": []
  }
}
```

MVP の `NormalizedCheckResult` top-level required field は次です。

```text
- schema
- normalized_result_id
- normalized_result_hash
- artifact
- artifact_hash
- policy
- results
- comparison
```

MVP の `NormalizedCheckResult.results[]` entry schema：

```text
always required:
  - result_id
  - result_hash
  - request_hash
  - policy_hash
  - artifact_hash
  - checker_profile
  - process_launched
  - status

required when process_launched = true:
  - checker_binary_hash

optional when process_launched = true and the corresponding raw identity field was syntactically valid:
  - checker_id
  - checker_build_hash

required when status = checked:
  - certificate_hash
  - export_hash
  - axiom_report_hash

required when status = failed:
  - error
  - failure_key

forbidden when process_launched = false:
  - checker_binary_hash
  - checker_id
  - checker_build_hash

forbidden when status = checked:
  - error
  - failure_key

optional when status = failed and present in the raw MachineCheckResult:
  - certificate_hash
  - export_hash
  - axiom_report_hash

unknown field:
  forbidden
```

`NormalizedCheckResult.results[]` entry の conditional validation は、always required、
`process_launched` conditional required / forbidden、`status` conditional required / forbidden、
allowed optional copy fields、unknown field の順で評価します。
conditional forbidden field は値の shape を検査せず、`actual_value = "forbidden_field"` とします。

`checker_binary_hash` は raw checker identity を trusted verdict として採用できない
launched failure でも、runner-owned executable identity として
`MachineCheckResult.checker.binary_hash` から必ず写します。
`checker_id` と `checker_build_hash` は `NormalizedCheckResult.results[]` schema 上は optional です。
normalizer は対応する `MachineCheckResult.checker.id` /
`MachineCheckResult.checker.build_hash` が存在する場合だけ同じ値を写し、存在しない field を
補完してはいけません。
`NormalizedCheckResult` 単体の schema validator はこれらの欠落だけで invalid にしてはいけません。
raw `MachineCheckResult` を入力に持つ normalizer / cross-artifact validator は、
raw field が存在するのに normalized entry へ写されていない場合、または値が一致しない場合を
normalization / cross-artifact mismatch として拒否します。
`status = failed` の `certificate_hash`、`export_hash`、`axiom_report_hash` は raw result に
存在する場合だけ写します。
`failure_key` は独立した trusted verdict ではなく、同じ entry の `error` から
7 の normalized failure key rule で導出した deterministic cache です。
`status = failed` の `NormalizedCheckResult` schema / domain validator は、
まず `error` を validation し、次に保存済み `failure_key` object 自体を
closed-world object として schema validation し、その後に expected `failure_key` object を再計算し、
保存済み `failure_key` と canonical object equality で一致することを検査します。
`failure_key` object 自体の unknown field、duplicate key、wrong type、explicit null、
invalid hash format、invalid name format、order violation は通常の
`normalized_result_schema_invalid` として報告します。
この object schema validation を通過した後の derived object との不一致は schema / domain failure であり、
`field = "results[].failure_key"`、
`expected_value = "derived_from_error"`、
`actual_value = "failure_key_mismatch"` にします。
たとえば `failure_key.declaration` が name grammar 違反なら `invalid_name_format` であり、
valid name だが sibling `error` から導出されない場合は `failure_key_mismatch` です。
`failure_key` に schema-valid だが導出されない field がある場合、または
`error` から導出される field が欠落している場合も同じ `failure_key_mismatch` です。

`artifact_hash` は top-level `artifact` object の canonical hash です。
normalizer は `artifact` を構築した直後に `artifact_hash` を計算し、
その後 `normalized_result_hash` を計算します。
`normalized_result_hash` は `artifact_hash` を hash 対象に含めます。
`artifact_hash` field と `artifact` object の再計算 hash が一致しない
`NormalizedCheckResult` は invalid です。
`npa-check compare`、release audit bundle validator、normalized result store validator は
`normalized_result_hash` を信頼する前に必ず `artifact_hash` を再計算して照合します。

MVP の `comparison` object は次を required にします。

```text
- status
- matching_fields
- missing_checker_profiles
- disagreements
- status_reasons
```

`matching_fields` は deterministic summary であり、次の固定順序だけを使います。

```text
checked comparison order:
  certificate_hash, export_hash, axiom_report_hash

failed comparison order:
  failure_key
```

`status = all_agree_checked` では `matching_fields` は
`["certificate_hash", "export_hash", "axiom_report_hash"]` です。
`status = all_agree_failed` では `matching_fields` は `["failure_key"]` です。
それ以外の status では `matching_fields` は `[]` です。

`missing_checker_profiles` は `RunnerPolicy.required_checker_profiles` のうち
入力 result に存在しない profile の配列です。
`status = missing_checker_result` の場合だけ non-empty にし、
それ以外の status では `[]` にします。
並び順は `RunnerPolicy.required_checker_profiles` に現れる順序です。
missing optional profile はここに入れません。

`disagreements` entry は次の形に固定します。

```json
{
  "field": "export_hash",
  "baseline_checker_profile": "fast-kernel",
  "baseline_hash": "sha256:...",
  "checker_profile": "external",
  "actual_hash": "sha256:..."
}
```

`field` は次に限定します。

```text
- artifact_hash
- certificate_hash
- export_hash
- axiom_report_hash
- failure_key
- status
```

hash field と `failure_key` の不一致では `baseline_hash` / `actual_hash` を使います。
`failure_key` の hash は failure_key object の canonical serialization hash です。
`status` の不一致では `baseline_value` / `actual_value` を使います。
`failure_key` object 自体を `baseline_value` / `actual_value` に入れてはいけません。
baseline は `RunnerPolicy.required_checker_profiles[0]` の result です。
ただし `field = artifact_hash` では baseline は top-level `NormalizedCheckResult.artifact_hash` であり、
`baseline_checker_profile` は omit します。
`disagreements[]` entry は closed-world object で、unknown field と duplicate key を禁止します。
field 種別ごとの required / forbidden member は次で固定します。

```text
field = artifact_hash:
  required: field, baseline_hash, checker_profile, actual_hash
  forbidden: baseline_checker_profile, baseline_value, actual_value

field = certificate_hash / export_hash / axiom_report_hash / failure_key:
  required: field, baseline_checker_profile, baseline_hash, checker_profile, actual_hash
  forbidden: baseline_value, actual_value

field = status:
  required: field, baseline_checker_profile, baseline_value, checker_profile, actual_value
  forbidden: baseline_hash, actual_hash
```

`disagreements[]` entry の schema validation priority は次で固定します。

```text
1. entry object type
2. duplicate key in the entry
3. field member presence / null / type / enum
4. field-specific required member presence / null / type / value shape, in the required order above
5. field-specific forbidden member presence, in the forbidden order above
6. unknown field, bytewise field name order
7. duplicate (field, checker_profile) pair and array order violation
```

entry object type failure の field は `comparison.disagreements[<i>]` です。
entry 内 member failure の field は `comparison.disagreements[<i>].<member>` です。
duplicate key の tie-break は `field`、`baseline_checker_profile`、`baseline_hash`、
`baseline_value`、`checker_profile`、`actual_hash`、`actual_value`、その後 unknown field の
bytewise field name order です。
required member と forbidden member が同時に不正な場合は required member failure を先に返します。
forbidden member は値の shape を検査せず、`actual_value = "forbidden_field"` とします。
duplicate `(field, checker_profile)` pair では、同じ pair がすでに出現している最小の後続 entry
index `<i>` を報告し、`field = "comparison.disagreements[<i>]"`、
`expected_value = "unique_field_checker_profile_pair"`、`actual_value = "duplicate_entry"` とします。
array order violation では、最初に `(field, checker_profile)` が直前 entry より bytewise 小さくなる
後続 entry index `<i>` を報告し、`field = "comparison.disagreements[<i>]"`、
`expected_value = "field_checker_profile_bytewise_ascending"`、`actual_value = "order_violation"` とします。
duplicate pair と array order violation が同時に成立する場合は duplicate pair を先に返します。

`checker_profile` は baseline と一致しない result の profile です。
`baseline_checker_profile` は baseline result の profile であり、
`field = artifact_hash` 以外では `RunnerPolicy.required_checker_profiles[0]` です。
`baseline_value` と `actual_value` は `NormalizedCheckResult.results[].status` の enum string です。
`disagreements` は `(field, checker_profile)` の組で重複を許さず、
`field`、次に `checker_profile` の bytewise lexicographic order で昇順に並べます。
`status = disagreement` では `disagreements` は non-empty でなければなりません。
それ以外の status では `disagreements` は `[]` です。

disagreement entry の生成規則は次で固定します。

```text
artifact_hash mismatch:
  participating result ごとに results[*].artifact_hash を
  NormalizedCheckResult.artifact_hash と比較する。
  一致しない result ごとに field = artifact_hash の entry を1件出す。
  baseline_hash = NormalizedCheckResult.artifact_hash,
  actual_hash = results[*].artifact_hash。

checked hash mismatch:
  participating checker がすべて checked の場合、
  certificate_hash, export_hash, axiom_report_hash をこの順序で比較する。
  baseline は最初の required profile の result。
  baseline と一致しない checker / field ごとに entry を1件出す。

failed failure_key mismatch:
  participating checker がすべて failed の場合、
  failure_key object の canonical hash を比較する。
  baseline と一致しない checker ごとに field = failure_key の entry を1件出す。

status mismatch:
  checked / failed が混在する場合、
  baseline status と一致しない checker ごとに field = status の entry を1件出す。
```

`status_reasons` は `policy_failure` と `inconclusive` の詳細専用です。
`status = policy_failure` または `status = inconclusive` の場合だけ non-empty にし、
それ以外の status では `[]` にします。
`disagreements` は checker 間の deterministic mismatch 専用なので、
`policy_failure`、`missing_checker_result`、`inconclusive` では `[]` にします。

MVP の `status_reasons` entry：

```json
{
  "kind": "policy_failure",
  "checker_profile": "reference",
  "result_hash": "sha256:...",
  "error_kind": "policy_failure",
  "reason_code": "checker_build_hash_mismatch",
  "field": "checker.build_hash",
  "expected_hash": "sha256:...",
  "actual_hash": "sha256:..."
}
```

`kind` は `policy_failure` または `inconclusive` です。
`checker_profile` は原因 result が特定できる場合 required です。
`result_hash` は原因 result が valid hash を持つ場合 required です。
`error_kind` は原因 result がある場合は原因 result の `error.kind` を写し、
comparison 自体が生成した理由では comparison-local な値を使います。
`reason_code` は `NormalizedComparisonReasonCode` です。
MVP の `NormalizedComparisonReasonCode` は、
5 の `MachineCheckResult.error.reason_code` closed enum と、
comparison が生成する comparison-generated reason code の union です。
原因 result が `error.reason_code` を持つ場合はその文字列をそのまま写します。
comparison 自体が生成する synthetic reason も同じ field に入ります。
この field には、この union に含まれない文字列を入れてはいけません。
hash / value mismatch がある場合は `field` と expected / actual を写します。
`status_reasons` は `kind`、次に `checker_profile`、次に `field` の
bytewise lexicographic order で昇順に並べます。
`checker_profile` または `field` が omit された entry の sort key では、該当 component を
empty string として扱います。
top-level `policy.hash` mismatch のように原因 result が特定できない場合、
`checker_profile` と `result_hash` は omit し、`error_kind = policy_failure`、
`reason_code = policy_hash_mismatch`、`field = "policy.hash"` を使います。
この場合、`expected_hash` は compare step に渡された `RunnerPolicy` の canonical hash、
`actual_hash` は `NormalizedCheckResult.policy.hash` です。
result-specific な policy mismatch では原因 result の `checker_profile` と `result_hash` を入れ、
`reason_code = result_policy_hash_mismatch`、`field = "results[].policy_hash"` を使います。
この場合、`expected_hash` は `NormalizedCheckResult.policy.hash`、
`actual_hash` は該当 result の `policy_hash` です。
MVP の comparison-generated `NormalizedComparisonReasonCode` は次に限定します。

```text
- policy_hash_mismatch
- result_policy_hash_mismatch
- checker_profile_not_allowed
- checker_identity_missing
- checker_identity_mismatch
- checker_binary_hash_mismatch
- checker_build_hash_mismatch
- malformed_process_state
```

comparison-generated reason の `error_kind` は次で固定します。

```text
policy_failure:
  - policy_hash_mismatch
  - result_policy_hash_mismatch
  - checker_profile_not_allowed
  - checker_identity_missing
  - checker_identity_mismatch
  - checker_binary_hash_mismatch
  - checker_build_hash_mismatch

checker_internal_error:
  - malformed_process_state
```

policy に含まれない profile では `reason_code = checker_profile_not_allowed`、
`field = "results[].checker_profile"`、
`expected_value = "required_or_optional_checker_profile"`、
`actual_value` に該当 profile を入れます。
comparison-generated identity reason の field shape は次で固定します。

```text
checker_binary_hash_mismatch:
  field = "results[].checker_binary_hash"
  expected_hash = SelectedCheckerPolicy.binary_hash
  actual_hash = results[*].checker_binary_hash

checker_identity_mismatch:
  field = "results[].checker_id"
  expected_value = SelectedCheckerPolicy.checker_id
  actual_value = results[*].checker_id

checker_build_hash_mismatch:
  field = "results[].checker_build_hash"
  expected_hash = SelectedCheckerPolicy.build_hash
  actual_hash = results[*].checker_build_hash

checker_identity_missing:
  field = "results[].checker_id" or "results[].checker_build_hash"
  expected_value = "required_for_launched_non_inconclusive_result"
  actual_value = "missing"

malformed_process_state:
  field = "results[].process_launched"
  expected_value = "process_state_consistent_with_error_kind"
  actual_value = "malformed_process_state"
```

`checker_identity_missing` で `checker_id` と `checker_build_hash` が両方不足する場合は、
field ごとに2件の `status_reasons` entry を生成します。
上記の result-local reason では原因 result の `checker_profile` と `result_hash` を入れます。
原因 result の `MachineCheckResult.error.reason_code` を写す場合は、
`error_kind` も同じ原因 result の `error.kind` を写します。
comparison-generated reason では `error_kind` は上の table に従います。

failed result entry の例：

```json
{
  "result_id": "mchkres_ref_002",
  "result_hash": "sha256:...",
  "request_hash": "sha256:...",
  "policy_hash": "sha256:...",
  "artifact_hash": "sha256:...",
  "checker_id": "npa-checker-ref",
  "checker_binary_hash": "sha256:...",
  "checker_build_hash": "sha256:...",
  "checker_profile": "reference",
  "process_launched": true,
  "status": "failed",
  "certificate_hash": "sha256:...",
  "error": {
    "kind": "type_mismatch",
    "declaration": "Nat.add_zero",
    "core_path": ["declarations", 17, "body"],
    "expected_hash": "sha256:...",
    "actual_hash": "sha256:..."
  },
  "failure_key": {
    "kind": "type_mismatch",
    "declaration": "Nat.add_zero",
    "core_path": ["declarations", 17, "body"],
    "expected_hash": "sha256:...",
    "actual_hash": "sha256:..."
  }
}
```

pre-check / internal failure entry の例：

```json
{
  "result_id": "mchkres_policy_failure_001",
  "result_hash": "sha256:...",
  "request_hash": "sha256:...",
  "policy_hash": "sha256:...",
  "artifact_hash": "sha256:...",
  "checker_profile": "reference",
  "process_launched": false,
  "status": "failed",
  "error": {
    "kind": "policy_failure",
    "reason_code": "request_budget_mismatch"
  },
  "failure_key": {
    "kind": "policy_failure",
    "reason_code": "request_budget_mismatch"
  }
}
```

この entry では checker が起動していないため、`checker_id`、`checker_binary_hash`、
`checker_build_hash`、`certificate_hash` は省略されます。
`checker_internal_error`、`timeout`、`certificate_decode_error` なども、
raw result に存在しない field は normalized entry でも省略します。

checker 起動後に raw output を parse できなかった failure entry の例：

```json
{
  "result_id": "mchkres_internal_001",
  "result_hash": "sha256:...",
  "request_hash": "sha256:...",
  "policy_hash": "sha256:...",
  "artifact_hash": "sha256:...",
  "checker_profile": "reference",
  "checker_binary_hash": "sha256:...",
  "process_launched": true,
  "status": "failed",
  "error": {
    "kind": "checker_internal_error",
    "reason_code": "malformed_rejection_output"
  },
  "failure_key": {
    "kind": "checker_internal_error",
    "reason_code": "malformed_rejection_output"
  }
}
```

この entry では checker が起動していますが、identity field を trusted verdict として
採用できないため `checker_id` と `checker_build_hash` は省略されます。
一方で runner-owned executable identity は記録済みなので、launched result では
`checker_binary_hash` を `MachineCheckResult.checker.binary_hash` から写します。

正規化器は、AI ではなく deterministic code として実装します。
AI はこの結果を読んで説明を書くだけです。
MVP では `NormalizedCheckResult.comparison` は required です。
`npa-check normalize-results` は、normalization と comparison を1つの deterministic step として実行し、
`comparison` field を埋めた `NormalizedCheckResult` を出力します。
これは normalize step がすでに `RunnerPolicy` を入力に取るためです。
`npa-check compare` は保存済み `NormalizedCheckResult` の `comparison` を再計算して検証する
idempotent command であり、別の正本 artifact を作りません。
再計算した comparison が保存済み field と一致しない場合、command は失敗します。
`normalized_result_hash` は次を除いた `NormalizedCheckResult` の canonical serialization hash です。

```text
- normalized_result_id
- normalized_result_hash
- results[*].result_id
```

`results[*].result_id` は人間向け provenance として残しますが、
再実行で変わり得るため normalized hash identity には含めません。
`results[*]` は comparison に必要な field を raw `MachineCheckResult` から写すため、
comparison は raw result files を再度開かなくても実行できます。
raw result file を開く必要があるのは、`result_hash` の再検証や監査 bundle 生成のときだけです。
`results[*].policy_hash` は raw `MachineCheckResult.policy.hash` から写します。
`results[*].artifact_hash` は、その result の `request_hash` から解決した
`MachineCheckRequest` で再構成した artifact object の canonical hash です。
`results[*].process_launched` は raw `MachineCheckResult.process.launched` から写します。
normalizer は入力 result ごとに `request_hash` から `MachineCheckRequest` を解決し、
request から artifact object を再構成します。
request から再構成した artifact hash を `results[*].artifact_hash` として記録します。
top-level `artifact_hash` は top-level `artifact` object の canonical hash です。
comparison は `results[*].artifact_hash` を top-level `artifact_hash` と比較します。
normalizer の入力は `MachineCheckResult` list、`RunnerPolicyReference`、
request store、および optional artifact selector です。
normalizer input には、同じ `checker_profile` の `MachineCheckResult` を2件以上入れてはいけません。
retry result が複数ある場合、caller は normalize 前に採用する attempt を1件に絞ります。
同じ `checker_profile` が重複する場合、normalizer は `NormalizedCheckResult` を作らず
`NormalizeErrorResult.error.reason_code = duplicate_checker_profile_result` を返します。
`results` array の順序は deterministic に固定します。
まず `RunnerPolicy.required_checker_profiles` の順序で required profile の entry を並べ、
次に入力に存在する optional profile の entry を
`RunnerPolicy.optional_checker_profiles` の順序で並べます。
policy に含まれない profile の entry は最後に `checker_profile` の bytewise lexicographic order で並べ、
`NormalizedCheckResult` は作成しますが、comparison で `policy_failure` にします。
policy に含まれない profile を `participating result` に入れてはいけません。
入力 list の順序を `NormalizedCheckResult.results` の順序に使ってはいけません。

artifact object の field は次から作ります。

```text
artifact.module:
  MachineCheckRequest.module

artifact.input_file_hash:
  MachineCheckRequest.certificate.file_hash

artifact.expected_certificate_hash:
  MachineCheckRequest.certificate.expected_certificate_hash

artifact.import_lock_hash:
  MachineCheckRequest.imports.manifest_hash

artifact.axiom_policy_hash:
  RunnerPolicy.axiom_policy.hash
```

`artifact.axiom_policy_hash` は request 単体からは作りません。
normalizer が RunnerPolicy を解決できない場合、`NormalizedCheckResult` を作らず
`NormalizeErrorResult` を返します。
top-level `artifact` は、normalizer input の explicit artifact selector から作ります。
MVP の selector は `module` と `request_hash` です。

```json
{
  "module": "Std.Nat",
  "request_hash": "sha256:..."
}
```

normalizer は selector の `request_hash` から基準 `MachineCheckRequest` を解決し、
top-level `artifact` を構築します。
selector の `module` は、解決した `MachineCheckRequest.module` と一致しなければなりません。
一致しない場合、normalizer は `NormalizeErrorResult` を返します。
selector が omit された場合は、single-artifact convenience mode として扱います。
この mode では `RunnerPolicy.required_checker_profiles[0]` と同じ `checker_profile` を持つ
result が入力 list にちょうど1件だけ存在しなければなりません。
0件または2件以上の場合、normalizer は `NormalizeErrorResult` を返します。
request store から selector または result の `request_hash` を解決できない場合、normalizer は
`NormalizedCheckResult` を返さず、`NormalizeErrorResult` を返します。

MVP の `NormalizeErrorResult`：

```json
{
  "schema": "npa.phase8.normalize_error_result.v1",
  "result_id": "normerr_Std.Nat_001",
  "result_hash": "sha256:...",
  "policy_hash": "sha256:...",
  "status": "failed",
  "error": {
    "kind": "normalize_failure",
    "reason_code": "request_hash_not_found",
    "field": "request_hash",
    "actual_hash": "sha256:..."
  }
}
```

`NormalizeErrorResult.result_hash` は `result_id` と `result_hash` field を除いた
canonical hash です。
`policy_hash` は、入力 `RunnerPolicyReference.hash` を軽量に decode して valid hash として読めた場合だけ
required で、その値を写します。
この軽量 decode は `NormalizeErrorResult` を構築するための envelope 処理であり、
endpoint-specific validation order ではありません。
したがって policy validation step に到達していない `selector_schema_invalid` でも、
`RunnerPolicyReference.hash` が valid hash として読めたなら `policy_hash` を入れます。
policy file が読めない場合や policy object を parse できない場合でも、
reference hash 自体が valid hash なら omit しません。
`RunnerPolicyReference.hash` 自体が missing、wrong type、explicit null、
または invalid hash format の場合は、reason_code が `policy_reference_invalid` 以外であっても
`policy_hash` を omit します。
API では wrapper validation が先に走るため、`policy.hash` の missing / wrong type / explicit null /
invalid hash format は `ApiError` であり、`NormalizeErrorResult` は作りません。
`NormalizeErrorResult.error.kind` は常に `normalize_failure` です。
これは checker verdict でも `MachineCheckResult` の `policy_failure` でもありません。
policy hash 解決失敗など policy に関係する理由でも、normalize pipeline artifact としては
`normalize_failure` に分類します。
MVP の `NormalizeErrorResult.error.reason_code` は次に限定します。

```text
- machine_result_file_unreadable
- machine_result_json_invalid
- machine_result_wrong_schema
- machine_result_schema_invalid
- machine_result_hash_mismatch
- machine_result_run_artifact_hash_mismatch
- machine_result_request_hash_mismatch
- request_hash_not_found
- request_file_unreadable
- request_json_invalid
- request_schema_invalid
- request_hash_missing
- request_file_hash_mismatch
- request_hash_mismatch
- request_store_reference_invalid
- request_store_manifest_hash_mismatch
- request_store_manifest_invalid
- output_path_conflict
- output_write_failure
- normalized_store_entry_file_hash_mismatch
- normalized_store_manifest_invalid
- normalized_store_entry_conflict
- normalized_store_write_failure
- policy_reference_invalid
- policy_file_unreadable
- policy_hash_mismatch
- duplicate_checker_profile_result
- selector_schema_invalid
- selector_module_mismatch
- selector_ambiguous
```

normalizer の validation は、request store entry を読まない intrinsic validation と、
request store を使う cross-artifact validation に分けます。
request store entry を読む前の順序は、`MachineCheckResult` intrinsic validation、
`artifact_selector` schema validation、`checker_profile` uniqueness validation、
`RunnerPolicyReference` schema validation、`request_store` reference schema / path validation、
policy file resolution / policy hash validation の順で固定します。
先の step で失敗した場合、後続 step の error は報告しません。
したがって malformed `artifact_selector` と unreadable policy file が同時に存在する場合は、
`selector_schema_invalid` を返し、policy file は読みません。
malformed `request_store` reference と unreadable policy file が同時に存在する場合は、
`request_store_reference_invalid` を返し、policy file は読みません。
CLI で file path から読む場合は file bytes を読めること、JSON として parse できること、
top-level schema が `npa.phase8.machine_check_result.v1` であることを検査します。
API で object として受け取る場合も、top-level schema と field schema を同じ順序で検査します。
`MachineCheckRequestErrorResult`、`NormalizeErrorResult`、`CompareValidationResult` など
`MachineCheckResult` 以外の schema が混入した場合は、checker verdict として扱わず
`NormalizeErrorResult.error.reason_code = machine_result_wrong_schema` を返します。
top-level JSON value が object でない場合も `machine_result_wrong_schema` です。
top-level JSON value が explicit null の場合は `actual_value = "null_not_allowed"`、
null 以外の non-object では `actual_value = "wrong_type"` とします。
`machine_result_wrong_schema` は top-level `schema` member 自体の問題だけに使います。
この場合 `error.field = "schema"`、
`expected_value = "npa.phase8.machine_check_result.v1"` を入れます。
`schema` field が存在しない場合は `actual_value = "missing"`、
explicit null の場合は `actual_value = "null_not_allowed"`、
string 以外の場合は `actual_value = "wrong_type"`、
unknown string の場合は `actual_value` に入力 artifact の `schema` 文字列を入れます。
`schema` が一意な string として存在し、値が `npa.phase8.machine_check_result.v1` と一致した後の
field schema violation は `machine_result_schema_invalid` です。
duplicate object key は、duplicate key が `schema` であっても `machine_result_schema_invalid` とし、
`expected_value = "unique_object_keys"`、`actual_value = "duplicate_field"` を返します。

`machine_result_file_unreadable` では `error.field = "machine_results[].path"`、
`actual_value = "unreadable"` にします。
`machine_result_json_invalid` では `error.field = "machine_results[].path"`、
`actual_value = "invalid_json"` にします。
`machine_result_schema_invalid` では `error.field` に invalid result field の JSON path、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、
`invalid_enum`、`invalid_hash_format`、`invalid_name_format`、`null_not_allowed`、`order_violation`、
`duplicate_field` のいずれかを入れます。
`MachineCheckResult.error.declaration` の name grammar violation は
`machine_result_schema_invalid` とし、
`field = "machine_results[].error.declaration"`、
`actual_value = "invalid_name_format"` にします。
`machine_result_hash_mismatch` では `error.field = "machine_results[].result_hash"`、
`expected_hash` に 3.3 の規則で再計算した `result_hash`、
`actual_hash` に input artifact の `result_hash` を入れます。
`machine_result_run_artifact_hash_mismatch` では
`error.field = "machine_results[].run_artifact_hash"`、
`expected_hash` に 3.3 の規則で再計算した `run_artifact_hash`、
`actual_hash` に input artifact の `run_artifact_hash` を入れます。
`machine_result_request_hash_mismatch` では `error.field = "machine_results[].request_hash"`、
`expected_hash` に解決した `MachineCheckRequest` から再計算した `request_hash`、
`actual_hash` に input `MachineCheckResult.request_hash` を入れます。
MachineCheckResult intrinsic validation order は次で固定します。

```text
1. file readable / JSON parse, if input is a file
2. top-level schema
3. MachineCheckResult schema
4. result_hash recomputation
5. run_artifact_hash recomputation
```

先の step で失敗した場合、後続 step の error は報告しません。

`artifact_selector` は optional です。
省略された場合は schema violation ではありません。
存在する場合は closed JSON object であり、`module` と `request_hash` は required です。
`artifact_selector` object 自体、または member の schema / domain violation は
`selector_schema_invalid` です。
`selector_schema_invalid` では、object 自体が wrong type / explicit null の場合
`field = "artifact_selector"`、
`expected_value = "ArtifactSelector"`、
`actual_value` に `wrong_type` または `null_not_allowed` を入れます。
member violation では `field` に `artifact_selector.module`、
`artifact_selector.request_hash`、または `artifact_selector.<unknown_field_name>` を入れ、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_hash_format`、
`invalid_name_format`、`null_not_allowed`、`order_violation`、`duplicate_field` のいずれかを入れます。
`artifact_selector.module` の name grammar violation は
`selector_module_mismatch` ではなく `selector_schema_invalid / invalid_name_format` です。
`artifact_selector.request_hash` の invalid hash format は
`request_hash_not_found` ではなく `selector_schema_invalid / invalid_hash_format` です。
`artifact_selector` schema validation に失敗した場合、normalizer は request store entry を読まず、
`request_hash_not_found`、`request_schema_invalid`、`selector_module_mismatch` は報告しません。

policy validation step を通過した後、request store を使う cross-artifact validation order は次で固定します。

```text
1. request store manifest file が readable であることを検査する
2. request store manifest file bytes hash を caller supplied request_store.manifest_hash と照合する
3. request store manifest file の JSON parse / schema / order / duplicate を検査する
4. artifact_selector が省略された場合、single-artifact convenience mode の baseline result を一意に選ぶ
5. explicit artifact_selector.request_hash、または step 4 で選んだ baseline result の request_hash を request store で解決する
6. step 5 で解決した baseline MachineCheckRequest file を readable / JSON / schema / self-hash / manifest entry hash まで検証する
7. explicit artifact_selector.module と baseline MachineCheckRequest.module を照合する
8. 入力 MachineCheckResult の request_hash を normalized result order で request store から解決し、
   各 MachineCheckRequest file を readable / JSON / schema / self-hash / manifest entry hash まで検証する
```

先の step で失敗した場合、後続 step の error は報告しません。
たとえば selector omitted かつ request store manifest が壊れている場合は
`request_store_manifest_invalid` を返し、`selector_ambiguous` は報告しません。
explicit selector の request hash が存在しない場合は step 5 の `request_hash_not_found` を返し、
`selector_module_mismatch` は報告しません。
step 6 で baseline request file が invalid な場合も `selector_module_mismatch` は報告しません。

step 5 と step 8 の request hash 解決順は次で固定します。
`artifact_selector` が指定された場合は、まず `artifact_selector.request_hash` を解決します。
見つからない場合は `request_hash_not_found`、
`field = "artifact_selector.request_hash"`、
`actual_hash = artifact_selector.request_hash` にします。
その後、入力 `MachineCheckResult` を `NormalizedCheckResult.results` に書き込む順序で解決します。
つまり required profile order、optional profile order、policy 外 profile の
`checker_profile` bytewise lexicographic order です。
`artifact_selector` が省略された場合も、single-artifact convenience mode で選ばれた
required baseline result はこの `MachineCheckResult` 解決順の中で扱い、
`artifact_selector.request_hash` では報告しません。
入力 `MachineCheckResult.request_hash` が request store にない場合は
`request_hash_not_found`、`field = "machine_results[].request_hash"`、
`actual_hash = MachineCheckResult.request_hash`、
`actual_value = MachineCheckResult.checker.profile` にします。
複数の入力 result の request hash が見つからない場合は、この解決順で最初の1件だけを報告します。
normalizer が request store から解決した request file を読めない、parse できない、
schema validation できない、または self hash validation できない場合、
`MachineCheckRequestErrorResult` ではなく `NormalizeErrorResult` を返します。
これは error が `npa-check run` ではなく normalize pipeline に属するためです。
`request_file_unreadable` では `field = "request_store.requests[].path"`、
`actual_value = "unreadable"` にします。
`request_json_invalid` では `field = "request_store.requests[].path"`、
`actual_value = "invalid_json"` にします。
`request_schema_invalid` では `field` に invalid request field の JSON path、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、
`invalid_enum`、`invalid_path`、`invalid_hash_format`、`invalid_name_format`、`null_not_allowed`、
`duplicate_field` のいずれかを入れます。
この field は artifact-local JSON path に `request_store.requests[]` prefix を付けた
wildcard-prefixed artifact path とします。
たとえば request artifact local `module` は `request_store.requests[].module` として報告します。
request store entry の top-level `schema` が
`npa.phase8.machine_check_request.v1` でない場合も `request_schema_invalid` です。
この場合は `field = "request_store.requests[].schema"`、
`expected_value = "npa.phase8.machine_check_request.v1"`、
`actual_value = "missing"`、`"null_not_allowed"`、`"wrong_type"`、
または入力 request artifact の `schema` 文字列を入れます。
この `request_schema_invalid` でも `actual_value = "wrong_schema"` は使いません。
`request_hash_missing` では `field = "request_hash"`、
`expected_value = "sha256:<lower-hex>"`、`actual_value = "missing"` にします。
これは request store から解決した `MachineCheckRequest` file 内の
artifact-local `request_hash` field が missing の場合だけに使います。
request store manifest entry の `request_hash` が missing の場合は
`request_store_manifest_invalid`、入力 `MachineCheckResult.request_hash` が missing の場合は
`machine_result_schema_invalid`、`artifact_selector.request_hash` が missing の場合は
`selector_schema_invalid` です。
`policy_reference_invalid` では、reference object 自体が missing / wrong type / explicit null の場合
`field = "policy"`、`expected_value = "RunnerPolicyReference"`、
`actual_value` に `missing`、`wrong_type`、または `null_not_allowed` を入れます。
reference object が存在し、その member が不正な場合は
`field` に invalid member の JSON path を入れます。
既知 member では `policy.kind`、`policy.path`、`policy.hash` のいずれか、
unknown field では `policy.<unknown_field_name>` です。
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_enum`、
`invalid_hash_format`、`invalid_path`、`null_not_allowed`、`order_violation`、
`duplicate_field` のいずれかを入れます。
policy file が読めるが JSON parse または `RunnerPolicy` schema / domain validation に失敗した場合も
`policy_reference_invalid` を使います。
JSON parse failure では `field = "policy.path"`、`actual_value = "invalid_json"`、
schema / domain validation failure では `field` に `policy.<RunnerPolicy JSON path>` を入れます。
root-level failure では `field = "policy"` とし、top-level `schema` failure では
`field = "policy.schema"` とします。
schema / domain validation failure の `expected_value` / `actual_value` は
4.1 の RunnerPolicy schema / domain validation field shape に従います。
`policy_file_unreadable` では `field = "policy.path"`、`actual_value = "unreadable"` を入れます。
この reason では `expected_hash` と `actual_hash` は omit します。
`policy_hash_mismatch` では `field = "policy.hash"`、
`expected_hash` に `RunnerPolicyReference.hash`、
`actual_hash` に読み込んだ `RunnerPolicy` の canonical hash を入れます。
`policy_reference_invalid` は request store reference validation より前の
`RunnerPolicyReference` schema validation step に到達した場合に返します。
`policy_file_unreadable` と `policy_hash_mismatch` は、request store reference validation を通過して
policy file resolution step に到達した場合だけ返します。
`request_store_reference_invalid` は Rust library boundary の raw normalizer input envelope が
`RequestStoreReference` JSON-like object を直接受け取った場合、または CLI `normalize-results` が
両方指定済みの `--request-store` / `--request-store-hash` pair を
normalizer input に変換した後だけ返します。
ここでいう raw normalizer input envelope は machine HTTP API ではなく、テストや CLI 実装から呼ぶ
in-process helper boundary です。
implementation はこの boundary で duplicate-aware decode、schema validation、path validation を実行し、
validation を通過した値だけを typed `RequestStoreReference` として normalizer core に渡します。
typed `RequestStoreReference` 構築後の normalizer core では missing / wrong type / explicit null /
unknown field / duplicate field / invalid path は到達不能で、manifest hash mismatch、
store file IO / JSON / schema / domain failure、artifact hash mismatch だけが残ります。
public machine API の `/machine/check/normalize` では `request_store` が endpoint wrapper field なので、
object shape / hash format / workspace path validation failure は `ApiError` で返し、
`NormalizeErrorResult` は作りません。
CLI `normalize-results` で `--request-store` / `--request-store-hash` pair が欠けている、
または片側だけ指定された場合も CLI argument validation error であり、
`request_store_reference_invalid` は作りません。
`request_store_reference_invalid` では、reference object 自体が missing / wrong type / explicit null の場合
`field = "request_store"`、`expected_value = "RequestStoreReference"`、
`actual_value` に `missing`、`wrong_type`、または `null_not_allowed` を入れます。
reference object が存在し、その member が不正な場合は
`field` に invalid member の JSON path を入れます。
既知 member では `request_store.kind`、`request_store.path`、`request_store.manifest_hash` のいずれか、
unknown field では `request_store.<unknown_field_name>` です。
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_enum`、
`invalid_hash_format`、`invalid_path`、`null_not_allowed`、`order_violation`、
`duplicate_field` のいずれかを入れます。
`request_store.path` の path schema violation では
`expected_value = "workspace_relative_path"`、`actual_value = "invalid_path"` にします。
`request_store.manifest_hash` の invalid hash format では
`expected_value = "sha256:<lower-hex>"`、`actual_value = "invalid_hash_format"` にします。
`request_store_manifest_invalid` では、request store manifest file を読めない場合
`field = "request_store.path"`、`actual_value = "unreadable"` にします。
JSON として壊れている場合は同じ field で `actual_value = "invalid_json"` にします。
schema / order / duplicate 違反では `field` に invalid request store manifest field の JSON path、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_hash_format`、
`invalid_path`、`null_not_allowed`、`order_violation`、`duplicate_field`、`duplicate_request_hash`、
`duplicate_path` のいずれかを入れます。
この field は caller-prefixed manifest path とし、manifest-local `requests[<i>].path` は
`request_store.requests[<i>].path` として報告します。
manifest schema / domain error の field は concrete index を含む caller-prefixed path に固定し、
entry file IO / JSON / artifact schema / hash validation error の field は下の dedicated reason code にある
`request_store.requests[]` wildcard path に固定します。
manifest entry `path` が workspace-relative path schema に違反する場合は
`request_store_manifest_invalid` としてここで止め、request file は読みに行きません。
`duplicate_checker_profile_result` では `field = "checker_profile"`、
`expected_value = "at_most_one_result_per_profile"` を入れ、`actual_value` に重複した profile 名を入れます。
`request_file_hash_mismatch` では `field = "request_store.requests[].file_hash"`、
`expected_hash` に request store manifest entry の `file_hash`、
`actual_hash` に request file bytes の sha256 を入れます。
`request_hash_mismatch` では次のどちらかの field を使います。

```text
field = request_hash:
  request file 内の self hash が再計算値と一致しない。
  expected_hash = recomputed request hash
  actual_hash = request file 内の request_hash

field = request_store.requests[].request_hash:
  request file の valid self hash が manifest entry と一致しない。
  expected_hash = manifest entry request_hash
  actual_hash = parsed request.request_hash
```

`request_store_manifest_hash_mismatch` では `field = "request_store.manifest_hash"`、
`expected_hash` に caller supplied manifest hash、`actual_hash` に request store manifest file bytes sha256 を入れます。
`selector_module_mismatch` では `field = "artifact_selector.module"`、
`expected_value` に解決した `MachineCheckRequest.module`、`actual_value` に selector の `module` を入れます。
`selector_ambiguous` では `field = "artifact_selector"`、
`expected_value = "exactly_one_required_profile_result"` を入れ、
`actual_value` には `zero_results` または `multiple_results` を入れます。
normalize error は comparison に渡してはいけません。

`artifact` は comparison の対象 identity です。
`request_hash` は profile / budget / checker selection を含む provenance であり、
required checker 間で一致することを要求してはいけません。
comparison は `policy.hash` だけでは実行できません。
required profiles、optional profiles、checker allowlist、MVP fixed policy field を読むため、
compare step は `RunnerPolicyReference` を入力に取り、そこから canonical `RunnerPolicy`
object を解決します。

比較 status：

```text
- all_agree_checked
- all_agree_failed
- disagreement
- missing_checker_result
- policy_failure
- inconclusive
```

`disagreement` は常に failure として扱います。
AI が多数決や説明で上書きしてはいけません。

comparison に入力できる result profile は `RunnerPolicy.required_checker_profiles` または
`RunnerPolicy.optional_checker_profiles` に含まれるものだけです。
それ以外の profile が `NormalizedCheckResult.results` に含まれる場合、
comparison は `NormalizedCheckResult` を invalid artifact として拒否せず、
`status_reasons` に `checker_profile_not_allowed` を入れて `policy_failure` を返します。
participating result は required profile の result と、実際に入力された optional profile の result です。
missing optional profile は `missing_checker_result` にしません。
ただし入力された optional result には、policy hash、checker identity、artifact hash の検査を
required result と同じように適用します。
入力された optional result が timeout / resource_exhausted / checker_internal_error で比較不能な場合は
`inconclusive` です。
入力された optional result が required result と deterministic に矛盾する場合は `disagreement` です。

比較規則は deterministic code で次の優先順位に従います。

```text
1. top-level policy.hash が input RunnerPolicy hash と一致しない、
   top-level policy.hash と results[*].policy_hash が一致しない、
   results[*].policy_hash 同士が一致しない、
   または results[*].checker_profile が policy の required / optional profile に含まれない
   -> status_reasons に policy_failure entry を入れる
   -> policy_failure

2. 下の malformed process state 条件に該当しない result で、
   error.kind = policy_failure の result がある
   -> status_reasons に policy_failure entry を入れる
   -> policy_failure

3. process_launched = true で、checker_binary_hash、checker_id、checker_build_hash のうち
   存在する identity field が policy allowlist と一致しない
   -> status_reasons に policy_failure entry を入れる
   -> policy_failure

4. process_launched = true かつ checker_id または checker_build_hash が不足している result で、
   error.kind が absent、または checker_internal_error / resource_exhausted / timeout 以外
   -> status_reasons に policy_failure entry を入れる
   -> policy_failure

5. schema-valid な normalized entry だが、process_launched / status / error.kind /
   identity field の組み合わせが下の malformed process state 条件に該当する
   -> status_reasons に malformed_process_state entry を入れる
   -> inconclusive

6. policy.required_checker_profiles の result が不足している
   -> missing_checker_profiles に不足 profile を policy order で入れる
   -> missing_checker_result

7. results[*].artifact_hash が NormalizedCheckResult.artifact_hash と一致しない、
   または results[*].artifact_hash 同士が一致しない
   -> disagreements に artifact_hash entry を入れる
   -> disagreement

8. process_launched = false かつ error.kind = timeout / resource_exhausted の
   launch 前 runner failure、または process_launched = true かつ
   resource_exhausted / checker_internal_error / timeout などで checker result が比較不能
   -> status_reasons に inconclusive entry を入れる
   -> inconclusive

9. participating checker の status がすべて checked
   かつ certificate_hash / export_hash / axiom_report_hash がすべて一致する
   -> all_agree_checked

10. participating checker の status がすべて failed
   かつ normalized failure key がすべて一致する
   -> all_agree_failed

11. 上記以外
   -> disagreements に status / checked hash / failure_key mismatch entry を入れる
   -> disagreement
```

checker allowlist 照合は `process_launched = true` の result にだけ適用します。
`checker_id` と `checker_build_hash` はそれぞれ独立に不足判定します。
片方だけ存在する launched result では、存在する field を step 3 で照合し、
不足 field を step 4 の `checker_identity_missing` として扱います。
同じ result で両方の step が成立する場合は両方の policy_failure reason を生成し、
`status_reasons` の並びは定義済み sort rule に従います。
step 4 の exempt 判定で `error` または `error.kind` が存在しない場合は non-exempt です。
したがって `status = checked` の launched result で `checker_id` または
`checker_build_hash` が不足する場合、comparison は `policy_failure` を返します。
`error.kind = policy_failure` が存在しても、同じ result が malformed process state 条件に
該当する場合は step 2 の対象外とし、step 5 で `malformed_process_state` として扱います。
malformed process state 条件は次に限定します。

```text
- process_launched = false なのに status = checked
- process_launched = false なのに checker_binary_hash / checker_id / checker_build_hash のいずれかが存在する
- process_launched = false かつ error.kind が policy_failure / timeout / resource_exhausted 以外
- process_launched = true かつ error.reason_code が launch_timeout または launch_resource_exhausted
- process_launched = false かつ error.reason_code が checker_timeout / checker_resource_exhausted /
  process_exit_failure のいずれか
```

これらは `NormalizedCheckResult` schema-only validation ではなく comparison-generated
`inconclusive` として扱い、`reason_code = malformed_process_state` を出します。
`process_launched = false` で checker identity が省略された result は、
`policy_failure` または `inconclusive` の判定規則で扱います。
malformed output などで checker が起動済みでも identity を得られない場合は、
`checker_internal_error` として `inconclusive` に分類します。
policy mismatch と artifact mismatch が同時に存在する場合は policy mismatch を優先し、
comparison status は `policy_failure` にします。
これは異なる policy の result を同一 artifact の disagreement として扱わないためです。
artifact mismatch は normalizer では拒否しません。
normalizer は result entry を保存し、comparison が deterministic に `disagreement` を返します。
`process_launched = false` で malformed process state 条件に該当しない場合に
許可される `error.kind` は `policy_failure`、`timeout`、`resource_exhausted` だけです。
このうち `policy_failure` は malformed process state 条件に該当しない場合だけ
上の step 2 で処理し、step 8 の `inconclusive` 対象には含めません。
launch 前 timeout は `error.kind = timeout`、`error.reason_code = launch_timeout`、
launch 前 resource exhaustion は `error.kind = resource_exhausted`、
`error.reason_code = launch_resource_exhausted` とします。
checker 起動後の timeout / resource exhaustion では `process_launched = true` にし、
`error.reason_code = checker_timeout` または `checker_resource_exhausted` とします。
それ以外の schema-valid な `process_launched = false` result は step 5 の
`malformed_process_state` として `inconclusive` にします。

artifact identity は次です。

```text
artifact_identity:
  - artifact.module
  - artifact.input_file_hash
  - artifact.expected_certificate_hash
  - artifact.import_lock_hash
  - artifact.axiom_policy_hash
```

`request_hash` は比較対象の同一性には使いません。

`all_agree_failed` の normalized failure key は次です。

```text
failure_key:
  - error.kind
  - error.reason_code, if present
  - error.field, if present
  - error.declaration, if present
  - error.core_path, if present
  - error.section, if present
  - error.offset, if present
  - error.expected_hash, if present
  - error.actual_hash, if present
  - error.expected_value, if present
  - error.actual_value, if present
```

normalizer はこの規則で `failure_key` を生成し、
validator / compare は保存済み `failure_key` を信頼する前に必ず再導出して照合します。
comparison の `failure_key` mismatch は、validation 済み `failure_key` object の canonical hash で比較します。
同じ `error.kind` でも field、declaration、hash、value が違う場合は `disagreement` です。
ただし `certificate_decode_error` のように declaration が存在しない error では、
`error.kind` と decode offset / section id が一致すれば同じ failure と見なします。

`inconclusive` は success ではありません。
CI / release policy では failure として扱い、人間が再実行または budget 調整を判断します。

---

# 7. AiAuditSidecar

AI が生成する artifact は sidecar です。
checker result と同じ directory に置いてもよいですが、hash chain には入れません。

```json
{
  "schema": "npa.phase8.ai_audit_sidecar.v1",
  "source": {
    "kind": "machine_result",
    "result_id": "mchkres_002",
    "result_hash": "sha256:...",
    "request_hash": "sha256:...",
    "run_artifact_hash": "sha256:...",
    "normalized_result_id": "norm_Std.Nat_001",
    "normalized_result_hash": "sha256:..."
  },
  "input_policy": {
    "id": "phase8-ai-triage-default",
    "version": 1,
    "hash": "sha256:...",
    "included_fields": [
      "certificate_hash",
      "checker_id",
      "checker_version",
      "error.core_path",
      "error.declaration",
      "error.kind",
      "module",
      "status"
    ],
    "redaction": "default"
  },
  "ai": {
    "agent": "npa-audit-assistant",
    "model": "example-model",
    "prompt_hash": "sha256:..."
  },
  "status": "triaged",
  "classification": {
    "category": "certificate_generator_bug",
    "confidence": "medium",
    "checker_error_kind": "type_mismatch"
  },
  "summary": "The checker rejected Nat.add_zero with type_mismatch at declarations[17].body.",
  "suggested_next_actions": [
    "Re-run certificate generation for Std.Nat with type tracing enabled.",
    "Compare the expected and actual core terms for declaration index 17."
  ]
}
```

MVP の `AiAuditSidecar` top-level required field：

```text
- schema
- source
- input_policy
- ai
- status
- summary
```

`classification` は `status = triaged`、`suggested_fix`、`suggested_challenge` の場合 required です。
`status = summarized` または `inconclusive` では optional です。
`classification` object が存在する場合、`category` と `confidence` は required です。
cross-artifact validation では、`classification.checker_error_kind` は
`classification` が存在し、かつ `source.kind = machine_result` で
参照先 `MachineCheckResult.status = failed` の場合だけ required です。
その場合は参照先 `MachineCheckResult.error.kind` と完全一致しなければなりません。
schema-only validation では参照先 `MachineCheckResult.status` を読まないため、
`source.kind = machine_result` の `classification.checker_error_kind` は optional です。
存在する場合は MVP の `MachineCheckResult.error.kind` enum の文字列として schema validation しますが、
required 判定や参照先 `error.kind` との一致判定は行いません。
`classification` が optional で omit された `summarized` / `inconclusive` sidecar では、
`classification.checker_error_kind` の照合は行いません。
`source.kind = machine_result` で参照先 `MachineCheckResult.status = checked` の場合と、
`source.kind = normalized_comparison` の場合は、`classification` が存在していても
`classification.checker_error_kind` は forbidden です。
`source.kind = normalized_comparison` の forbidden rule は source artifact を読まなくても判定できるため、
schema-only validation でも適用します。
`source.kind = normalized_comparison` で `classification.checker_error_kind` が存在する場合は
`forbidden_sidecar_field` を使い、`error.field = "classification.checker_error_kind"`、
`actual_value = "present"` とします。
この static forbidden presence は enum validation より優先し、値の内容は検査しません。
`classification.checker_error_kind` が enum として不正な場合は
`sidecar_schema_invalid` を使い、`error.field = "classification.checker_error_kind"`、
`expected_value = "MachineCheckResult.error.kind"`、
`actual_value = "invalid_enum"` とします。
cross-artifact validation で参照先 `MachineCheckResult.status = failed` かつ
`classification.checker_error_kind` が missing または参照先 `MachineCheckResult.error.kind` と
一致しない場合は `referenced_artifact_value_mismatch` を使います。
ここでの missing は、`classification` object が存在するが
`classification.checker_error_kind` member がない場合だけを指します。
`classification` が optional な `summarized` / `inconclusive` sidecar で
`classification` object 自体が omit された場合は、ここでは mismatch にしません。
status により `classification` object 自体が required なのに omit された場合は
step 3 の status-dependent required field violation として `sidecar_schema_invalid` を返します。
この場合は `error.field = "classification"`、
`expected_value = "required_for_status:<status>"`、
`actual_value = "missing"` です。
`classification.checker_error_kind` missing / mismatch の場合の
`error.field = "classification.checker_error_kind"`、
`expected_value` は参照先 `MachineCheckResult.error.kind`、
`actual_value` は missing なら `missing`、存在するなら sidecar の
`classification.checker_error_kind` です。
cross-artifact validation で参照先 `MachineCheckResult.status = checked` にもかかわらず
`classification.checker_error_kind` が存在する場合も
`referenced_artifact_value_mismatch` を使い、
`error.field = "classification.checker_error_kind"`、
`expected_value = "absent"`、`actual_value` は sidecar の
`classification.checker_error_kind` とします。
`suggested_next_actions` は `status = suggested_fix` または `suggested_challenge` の場合
non-empty array として required です。
それ以外の status では optional で、存在する場合は array でなければなりません。

MVP の `AiAuditSidecar.source` required field：

```text
all source:
  - kind

kind = machine_result:
  - result_hash
  - request_hash
  - run_artifact_hash

kind = normalized_comparison:
  - normalized_result_hash
```

`result_id` と `normalized_result_id` は optional human reference です。
`kind = machine_result` で `normalized_result_hash` が存在する場合、
`normalized_result_id` は optional です。
`kind = machine_result` で `normalized_result_id` が存在し、
`normalized_result_hash` が存在しない場合は step 3 の source shape violation として
`sidecar_schema_invalid` を返します。
この場合は `error.field = "source.normalized_result_hash"`、
`expected_value = "required_with_source.normalized_result_id"`、
`actual_value = "missing"` です。
`kind = normalized_comparison` では `result_hash`、`request_hash`、
`run_artifact_hash`、`result_id` を omit します。
`kind = normalized_comparison` でこれら forbidden source member が存在する場合は
step 3 の source shape violation として `forbidden_sidecar_field` を返します。
`error.field` は存在した member の JSON path、
`actual_value = "present"` です。
複数存在する場合は `source.result_hash`、`source.request_hash`、
`source.run_artifact_hash`、`source.result_id` の順で最初の field を返します。
step 3 内で複数の sidecar schema / source shape / static forbidden failure が同時に成立する場合は、
次の順で最初の1件だけを返します。

```text
1. duplicate object key
2. top-level schema missing / null / wrong type / mismatch
3. source object shape: source.kind missing / null / wrong type / invalid enum
4. source.kind ごとの required source member missing / null / wrong type / invalid hash format
5. source.kind ごとの forbidden source member presence
6. status enum / status-dependent required field
7. static forbidden sidecar field presence
8. classification enum
9. policy-gated field path violation
10. unknown field
11. その他の field schema failure
```

この order では source を解決せずに判定できる shape / schema failure だけを扱います。
`classification.checker_error_kind` の source artifact 依存 required / mismatch / checked-result presence は
この step 3 priority では扱わず、audit-sidecar validation order step 10 の
cross-artifact validation で扱います。

MVP の `AiAuditSidecar.input_policy` required field：

```text
- id
- version
- hash
- included_fields
- redaction
```

MVP の `AiAuditSidecar.ai` required field：

```text
- agent
- model
- prompt_hash
```

MVP の `classification.category` は次に限定します。

```text
- certificate_encoding_bug
- import_resolution_bug
- certificate_generator_bug
- kernel_checker_disagreement
- axiom_policy_violation
- source_to_certificate_staleness
- unsupported_feature
- checker_resource_limit
- checker_internal_bug
- unknown
```

MVP の `classification.confidence` は `low`、`medium`、`high`、`unknown` のいずれかです。
`classification.checker_error_kind` は source result から機械的に照合される補助 field であり、
AI が独自に選ぶ verdict ではありません。

MVP の `AiAuditInputPolicy` file schema：

```json
{
  "schema": "npa.phase8.ai_audit_input_policy.v1",
  "id": "phase8-ai-triage-default",
  "version": 1,
  "included_fields": [
    "certificate_hash",
    "checker_id",
    "checker_version",
    "error.core_path",
    "error.declaration",
    "error.kind",
    "module",
    "status"
  ],
  "redaction": "default",
  "allow_source_text": false,
  "allow_tactic_trace": false
}
```

input policy file は self-hash field を持ちません。
`AiAuditSidecar.input_policy.hash` は、この policy file の canonical hash です。
MVP の `AiAuditInputPolicy` top-level required field は `schema`、`id`、`version`、
`included_fields`、`redaction`、`allow_source_text`、`allow_tactic_trace` です。
`id` は non-empty string です。
empty string は `input_policy_schema_invalid` の domain failure とし、
`expected_value = "non_empty_string"`、`actual_value = "empty_string"` を使います。
`version` は JSON integer で、`1 <= version <= 9223372036854775807` でなければなりません。
wrong type / explicit null は `input_policy_schema_invalid` の schema failure、0 以下は
`actual_value = "non_positive_integer"`、範囲超過は
`actual_value = "integer_out_of_range"` です。
`AiAuditInputPolicy` schema / domain failure を `AuditSidecarValidationResult.error` に入れる場合、
`error.field` は input policy file artifact root に `input_policy` prefix を付けた JSON path です。
artifact root 自体の failure は `error.field = "input_policy"` にします。

```text
top-level JSON value is not object:
  field = "input_policy"
  expected_value = "object"
  actual_value = wrong_type | null_not_allowed

top-level schema missing / null / wrong type / mismatch:
  field = "input_policy.schema"
  expected_value = "npa.phase8.ai_audit_input_policy.v1"
  actual_value = missing | null_not_allowed | wrong_type | invalid_enum

generic field schema failure:
  field = "input_policy.<AiAuditInputPolicy JSON path>"
  expected_value = <schema requirement name>
  actual_value = missing | wrong_type | unknown_field | invalid_enum |
                 invalid_hash_format | null_not_allowed | order_violation |
                 duplicate_field

id empty:
  field = "input_policy.id"
  expected_value = "non_empty_string"
  actual_value = "empty_string"

version domain violation:
  field = "input_policy.version"
  expected_value = "positive_i64"
  actual_value = "non_positive_integer" | "integer_out_of_range"
```

`included_fields` は bytewise lexicographic order で昇順に並べます。
`included_fields` は重複を許しません。
MVP の `included_fields` に入れられる field path は次に限定します。

```text
- module
- input_file_hash
- expected_certificate_hash
- certificate_hash
- checker_id
- checker_version
- checker_profile
- checker_build_hash
- status
- error.kind
- error.reason_code
- error.declaration
- error.core_path
- error.expected_hash
- error.actual_hash
- policy.id
- policy.version
- policy.hash
```

この list にない field path を含む policy は `sidecar_schema_invalid` ではなく、
`AuditSidecarValidationResult.error.reason_code = input_policy_schema_invalid` として扱います。
sidecar copied metadata 側の `included_fields` に未知 field、重複、順序違反がある場合は
`input_policy_field_mismatch` ではなく `sidecar_schema_invalid` にします。
この場合の `actual_value` はそれぞれ `unknown_field`、`duplicate_field`、
`order_violation` です。
`expected_value` はそれぞれ `allowed_input_policy_field`、`unique_included_fields`、
`field_path_bytewise_ascending` です。
`error.field` は `input_policy.included_fields[<i>]` とし、`<i>` は未知 field の index、
重複 field の最小の後続 index、または最初に field path が直前 element より小さくなる
後続 index です。
`redaction` は `default`、`strict`、`release` のいずれかです。
`AiAuditInputPolicy` object は closed-world object で、unknown field と duplicate key を禁止します。
AiAuditInputPolicy validation は schema failure を domain failure より先に報告します。
ここでの schema failure は object / field shape、required field、explicit null、wrong type、
invalid enum / hash format、unknown field、duplicate object key、および
`included_fields[]` element の wrong type / explicit null です。
`included_fields[]` unsupported field path、`included_fields` order violation、
`included_fields` duplicate field は `input_policy_schema_invalid` reason code で報告しますが、
validation priority 上は local domain failure です。
したがって、たとえば `redaction` wrong type と `included_fields[]` unsupported field path が
同時に存在する場合は `redaction` wrong type を先に報告します。
複数の schema failure が同時に存在する場合は、top-level non-object、`schema`、
`id`、`version`、`included_fields` array、`included_fields[]` element by smaller index、
`redaction`、`allow_source_text`、`allow_tactic_trace`、
その後 unknown field の bytewise field name order で最初の1件だけを返します。
known field の duplicate object key は、その field の schema order 位置で
`expected_value = "unique_object_keys"`、`actual_value = "duplicate_field"` として報告します。
unknown field の duplicate object key は unknown field の位置で
`expected_value = "unique_object_keys"`、`actual_value = "duplicate_field"` を返し、
`error.field` は重複した後続 unknown field の JSON path にします。
複数の domain failure が同時に存在する場合は、`id` empty、`version` domain violation、
`included_fields[]` unsupported field path by smaller index、`included_fields` order violation、
`included_fields` duplicate field の順で最初の1件だけを返します。
duplicate field は、同じ field path がすでに出現している最小の後続 index を報告対象にします。
`included_fields[]` unsupported field path の `error.field` は
`input_policy.included_fields[<i>]`、`expected_value = "allowed_input_policy_field"`、
`actual_value = "unknown_field"` です。
`included_fields` order violation の `error.field` は `input_policy.included_fields[<i>]`、
`expected_value = "field_path_bytewise_ascending"` で、
`actual_value = "order_violation"`、
`<i>` は最初に field path が直前 element より小さくなる後続 index です。
`included_fields` duplicate field の `error.field` は `input_policy.included_fields[<i>]`、
`expected_value = "unique_included_fields"` で、
`actual_value = "duplicate_field"`、
`<i>` は同じ field path がすでに出現している最小の後続 index です。
cross-artifact validation では、validation reference の `input_policy.hash`、
`AiAuditSidecar.input_policy.hash`、policy file の canonical hash の3つが
完全一致することを検査します。
CLI の `--input-policy-hash` は validation reference の `input_policy.hash` です。
さらに `id`、`version`、`included_fields`、`redaction` は
policy file から sidecar に copied metadata として完全一致しなければなりません。
`allow_source_text` と `allow_tactic_trace` は validator が禁止 source text / tactic trace を
判定するために使い、sidecar には copied metadata として入れません。

MVP の sidecar schema は closed-world です。
各 object でこの文書に定義されていない field は
`AuditSidecarValidationResult.error.reason_code = sidecar_schema_invalid`、
`actual_value = "unknown_field"` として扱います。
ただし `allow_source_text` / `allow_tactic_trace` に依存する policy-gated field name は
generic unknown field として扱いません。
これらは sidecar schema が知っている policy-gated extension field name であり、
step 3 では field name の存在だけで `unknown_field` を返さず、値の shape だけを検査します。
policy-gated field name は top-level `AiAuditSidecar` object の optional field としてだけ許可します。
たとえば top-level `source_text` は policy-gated field として扱いますが、
`classification.source_text` や `ai.tactic_trace` は path が不正なので、
input policy の許可値に関係なく `forbidden_sidecar_field`、`actual_value = "present"` です。
MVP の policy-gated field value は string または string array に限定し、
それ以外は `sidecar_schema_invalid`、`actual_value = "wrong_type"` です。
cross-artifact validation では input policy を読んだ後の step 7 で、
対応する `allow_source_text = false` または `allow_tactic_trace = false` なら
`forbidden_sidecar_field`、`actual_value = "present"` を返します。
対応する input policy flag が `true` の場合、top-level の該当 policy-gated field は許可されます。
schema-only validation では input policy を読まないため、policy-gated field の禁止判定は行いません。
ただし schema-only validation でも、policy-gated field name が top-level 以外に出た場合は
path 不正として `forbidden_sidecar_field` を返します。
この例外は下の policy-gated field name だけに適用し、それ以外の未定義 field は
常に generic unknown field です。
次の reserved field name は unknown field ではなく `forbidden_sidecar_field` です。
sidecar の任意の object path に出現してはいけません。

```text
- verdict
- accepted
- checked
- verified
- checker_status
- certificate_valid
- proof_valid
- generated_certificate
- generated_certificate_bytes
- certificate_bytes
- proof_term
- raw_certificate
- raw_proof
```

`allow_source_text = false` の input policy では、次の field name も forbidden です。

```text
- source_text
- source_excerpt
- theorem_statement
- proof_script
```

`allow_tactic_trace = false` の input policy では、次の field name も forbidden です。

```text
- tactic_trace
- tactic_script
- elaboration_trace
- ai_search_trace
```

secret token の hard validation は field name に限定します。
次の field name が sidecar の任意の object path に出現した場合、
validator は `forbidden_sidecar_field` を返します。

```text
- secret
- token
- access_token
- refresh_token
- api_key
- password
- authorization
- private_key
```

自然言語の `summary` や `suggested_next_actions` 本文に対する token / source text 推定は
MVP では hard validation ではなく lint warning です。
hard validation は JSON field name と schema shape だけで deterministic に行います。

sidecar の禁止事項：

```text
- checker result と同じ `status` enum を使う
- sidecar 自身の status として `checked` / `accepted` / `verified` を使う
- checker output を書き換えたように見える field 名を使う
- certificate hash を AI が再計算した値として主張する
- source / tactic が正しいので certificate も正しい、と主張する
```

AI summary 本文では、checker result の引用として `status = checked` や
`status = failed` と書いてよいです。
ただしそれは必ず `source.result_hash` または `source.normalized_result_hash` に紐づく引用であり、
sidecar 自身の verdict ではありません。

`source.kind` は次のどちらかです。

```text
machine_result:
  one MachineCheckResult に対する summary / triage。
  source.result_hash is required.
  source.request_hash is required.
  source.run_artifact_hash is required.
  source.normalized_result_hash is optional semantic membership metadata.
  source.result_id is optional human reference.
  source.normalized_result_id is optional human reference.

normalized_comparison:
  NormalizedCheckResult.comparison に対する summary / triage。
  source.normalized_result_hash is required.
  source.result_hash must be omitted.
  source.request_hash must be omitted.
  source.run_artifact_hash must be omitted.
  source.normalized_result_id is optional human reference.
  source.result_id must be omitted.
```

`source.result_id` と `source.normalized_result_id` は hash identity には使いません。
validator は `source.kind = machine_result` では `source.run_artifact_hash`、
`source.kind = normalized_comparison` では `source.normalized_result_hash` で対象 artifact を解決します。
`source.kind = machine_result` で `source.normalized_result_hash` が存在する場合、
通常の cross-artifact validator は `NormalizedCheckResult.results[*].result_hash` による
semantic membership だけを検査します。
`NormalizedCheckResult.results[*]` は `run_artifact_hash` を持たないため、
この検査は source raw run が normalizer input に選ばれた exact run であることを証明しません。
exact selected raw result との一致は release audit bundle validator の closed-set rule だけが検査します。
`source.kind = machine_result` で `source.normalized_result_id` を書く場合は、
`source.normalized_result_hash` も必須です。
`source.normalized_result_hash` がない machine_result sidecar に
`source.normalized_result_id` を書いてはいけません。
release audit bundle で reproducibility repeated raw result を説明する sidecar では、
その raw result は normalizer input ではないため `source.normalized_result_hash` を omit します。
id field が存在する場合は、hash で解決した artifact 内の id field と一致しなければなりません。
一致しない場合、cross-artifact validation は failed です。

MVP の sidecar validator は次を検査します。

```text
- source.kind = machine_result の場合、source.run_artifact_hash で実在する MachineCheckResult を一意に解決できる
- source.kind = machine_result の場合、source.result_hash と source.request_hash が同じ MachineCheckResult と一致する
- source.kind = machine_result かつ source.result_id が存在する場合、同じ MachineCheckResult の result_id と一致する
- source.kind = machine_result かつ source.normalized_result_hash が存在する場合、その NormalizedCheckResult.results に同じ source.result_hash の entry が存在する。これは semantic membership check であり exact run membership check ではない
- source.kind = machine_result かつ source.normalized_result_id が存在する場合、source.normalized_result_hash も存在する
- source.kind = machine_result かつ source.normalized_result_id が存在する場合、同じ NormalizedCheckResult の normalized_result_id と一致する
- source.kind = normalized_comparison の場合、source.normalized_result_hash が実在する NormalizedCheckResult の normalized_result_hash と一致する
- source.kind = normalized_comparison かつ source.normalized_result_id が存在する場合、同じ NormalizedCheckResult の normalized_result_id と一致する
- source.kind = normalized_comparison の場合、source.result_hash / source.request_hash / source.run_artifact_hash は存在しない
- input_policy.hash が policy file の canonical hash と一致する
- input_policy.id / version / included_fields / redaction が policy file の copied metadata と一致する
- status が summarized / triaged / suggested_fix / suggested_challenge / inconclusive のいずれか
- status ごとの required field と classification enum が MVP schema に一致する
- source.kind = machine_result かつ source result が failed で classification が存在する場合、classification.checker_error_kind が source result の error.kind と一致する
- source.kind = machine_result かつ source result が checked の場合、classification.checker_error_kind は omit する
- source.kind = normalized_comparison の場合、classification.checker_error_kind は omit する
- sidecar が structured verdict field を持たない
- sidecar に certificate bytes / generated certificate bytes が含まれない
- sidecar に secret token や policy で禁止された source text が含まれない
```

validator input は sidecar file だけでは足りません。
MVP の cross-artifact validation は次を入力に取ります。

```text
- AiAuditSidecar file
- MachineCheckResult store
- NormalizedCheckResult store, source.normalized_result_hash を使う場合
- input_policy file
```

CLI / API が sidecar file だけを受け取る mode は `schema-only` validation です。
schema-only mode は JSON schema、禁止 field、status enum、
status ごとの required field、classification enum、hash 文字列の構文だけを検査し、
source hash や input_policy hash の実在確認は行いません。
schema-only mode では `source.kind = machine_result` の
`classification.checker_error_kind` を required にせず、存在する場合は enum 構文だけを検査します。
`source.kind = normalized_comparison` の `classification.checker_error_kind` は
schema-only mode でも forbidden です。
参照先 artifact の状態に依存する sidecar status 許可条件も schema-only mode では検査しません。
schema-only mode は、sidecar 内に cross-artifact claim が含まれていても、
それを検証済みとして扱ってはいけません。
schema-only result は `mode = schema_only` にし、`source_kind`、
`source_result_hash`、`source_normalized_result_hash` を返してはいけません。
schema-only mode では次を検査してはいけません。

```text
- source.run_artifact_hash / source.normalized_result_hash が実在するか
- source.result_hash / source.request_hash が参照先と一致するか
- source.result_id / source.normalized_result_id が参照先と一致するか
- sidecar status が参照先 artifact の状態に対して許可されるか
- source.kind = machine_result の classification.checker_error_kind が参照先 error.kind と一致するか
- input_policy.hash が policy file と一致するか
- policy で禁止された source text が sidecar に含まれるか
```

これらは cross-artifact validation 専用です。
CI / release で有効な sidecar として扱うには、必ず cross-artifact validation を通します。

sidecar status の許可条件は source artifact の状態で固定します。
`source.kind = machine_result` で参照先 `MachineCheckResult.status = checked` の場合、
sidecar は `status = summarized` のみ許可します。
checked result の sidecar は triage / fix suggestion ではなく、release audit summary 用です。
`source.kind = machine_result` で参照先 `MachineCheckResult.status = failed` の場合、
`summarized`、`triaged`、`suggested_fix`、`suggested_challenge`、`inconclusive` を許可します。
`source.kind = normalized_comparison` の場合、参照先
`NormalizedCheckResult.comparison.status` で許可 status を決めます。
`all_agree_checked` または `all_agree_failed` では `summarized` のみ許可します。
`disagreement`、`missing_checker_result`、`policy_failure`、`inconclusive` では
`summarized`、`triaged`、`suggested_fix`、`suggested_challenge`、`inconclusive` を許可します。
この4つの comparison status だけが release required `normalized_comparison` sidecar target になります。

validator は自然言語の完全な真偽判定をしません。
summary / suggested_next_actions の自然言語本文に対する「checker verdict を上書きしているか」の判定は、
hard validation ではなく lint warning に限定します。
hard validation は schema、禁止 field、status enum、source hash、入力 policy の一致だけを
deterministic に検査します。

---

# 8. AI triage

AI triage は、失敗を人間が直しやすい単位に分類する作業です。
正しさの判定ではありません。

MVP の分類：

```text
- certificate_encoding_bug
- import_resolution_bug
- certificate_generator_bug
- kernel_checker_disagreement
- axiom_policy_violation
- source_to_certificate_staleness
- unsupported_feature
- checker_resource_limit
- checker_internal_bug
- unknown
```

対応例：

```text
checker error kind: noncanonical_encoding
AI category      : certificate_encoding_bug
next action      : inspect serializer canonical ordering

checker error kind: import_hash_mismatch
AI category      : import_resolution_bug
next action      : compare import lock with certificate header

checker error kind: forbidden_axiom
AI category      : axiom_policy_violation
next action      : inspect axiom report and release policy

checker error kind: conversion_failure
AI category      : certificate_generator_bug or kernel_checker_disagreement
next action      : run differential checker with reduced declaration slice
```

AI は `confidence` を出してもよいですが、CI status に使ってはいけません。

---

# 9. Disagreement triage

checker disagreement は Phase 8 の重要な検出対象です。
AI の役割は、どの artifact を比較すればよいかを示すことです。

disagreement の例：

```text
- fast kernel は checked、reference checker は failed
- reference checker と external checker の export_hash が違う
- 同じ checker を2回走らせて certificate_hash が違う
- checker は checked だが axiom_report_hash が policy result と違う
```

AI triage は次の順で情報を集めます。

```text
1. certificate_hash が一致しているか
2. checker build hash が policy allowlist と一致しているか
3. import lock が同一か
4. failure declaration が同一か
5. error kind が同一か
6. export_hash / axiom_report_hash のどちらが違うか
7. resource_exhausted や timeout が混ざっていないか
```

AI がしてはいけないこと：

```text
- fast kernel の成功を理由に reference checker の失敗を無視する
- external checker の failure を flaky と決めつける
- checker version 差分を理由に checked 扱いにする
- source が変わっていないことを理由に certificate mismatch を無視する
```

disagreement report は必ず `failure` として CI に返します。

---

# 10. Challenge generation

AI は adversarial challenge を作れます。
これは checker を強くするためのテスト入力であり、証明 artifact ではありません。

challenge の基本形式：

```json
{
  "schema": "npa.phase8.challenge_manifest.v1",
  "challenge_id": "pch_001",
  "policy_hash": "sha256:...",
  "module": "Std.Nat",
  "imports": {
    "mode": "locked_store",
    "manifest": "build/certs/import-lock.json",
    "manifest_hash": "sha256:..."
  },
  "base_certificate": {
    "path": "build/certs/Std/Nat.npcert",
    "file_hash": "sha256:...",
    "claimed_certificate_hash": "sha256:..."
  },
  "mutated_certificate": {
    "path": "build/challenges/pch_001/Std.Nat.mutated.npcert",
    "file_hash": "sha256:...",
    "claimed_certificate_hash": "sha256:..."
  },
  "mutation": {
    "kind": "drop_axiom_report_entry",
    "target": "Nat.add_zero",
    "seed": "sha256:..."
  },
  "outcome_hint": {
    "status": "should_fail",
    "error_kinds": [
      "axiom_report_mismatch",
      "certificate_hash_mismatch"
    ]
  },
  "replay": {
    "generator": "npa-check challenge generate",
    "generator_version": "0.8.0",
    "generator_build_hash": "sha256:...",
    "args_hash": "sha256:..."
  },
  "generated_by": {
    "kind": "ai",
    "prompt_hash": "sha256:..."
  }
}
```

MVP の `ChallengeGenerationRequest` schema：

```json
{
  "schema": "npa.phase8.challenge_generation_request.v1",
  "request_id": "chgen_001",
  "request_hash": "sha256:...",
  "challenge_id": "pch_001",
  "policy_hash": "sha256:...",
  "module": "Std.Nat",
  "imports": {
    "mode": "locked_store",
    "manifest": "build/certs/import-lock.json",
    "manifest_hash": "sha256:..."
  },
  "base_certificate": {
    "path": "build/certs/Std/Nat.npcert",
    "file_hash": "sha256:...",
    "claimed_certificate_hash": "sha256:..."
  },
  "mutation": {
    "kind": "drop_axiom_report_entry",
    "target": "Nat.add_zero",
    "seed": "sha256:..."
  },
  "output": {
    "store_manifest_path": "build/challenges/manifest.json",
    "manifest_path": "build/challenges/pch_001/manifest.json",
    "mutated_certificate_path": "build/challenges/pch_001/Std.Nat.mutated.npcert"
  },
  "generated_by": {
    "kind": "ai",
    "prompt_hash": "sha256:..."
  }
}
```

`ChallengeGenerationRequest` は transient request ですが、deterministic replay のため
`request_hash` を持ちます。
`request_hash` は `request_id` と `request_hash` field を除いた
`ChallengeGenerationRequest` の canonical hash です。
ここでの generator は、完全な `ChallengeGenerationRequest` object を受け取る
generator core を指します。
generator core と API は policy / import / base certificate を読む前、かつどの output path にも
書き込む前に `request_hash` を validation しなければなりません。
CLI の flag front-end は generator core を呼ぶ前、かつ `--from` を読む前に、
`ChallengeGenerationRequest` に入る全 CLI flag の schema / domain validation を行います。
この validation は `base_certificate.path`、`imports.manifest`、`output.store_manifest_path`、
`output.manifest_path`、`output.mutated_certificate_path` の path schema validation と、
`mutation.kind` closed enum、`mutation.target` schema/domain validation、`mutation.seed` hash format、
`generated_by` conditional shape を含みます。
複数 failure がある場合は、この後の full `ChallengeGenerationRequest` schema / domain validation と
同じ field order で最初の failure を `generation_request_schema_invalid` として返し、
file IO と output 作成を行ってはいけません。
この CLI construction schema / domain validation が通った後でだけ、CLI front-end は `--from` を読んで
`base_certificate.file_hash` と `base_certificate.claimed_certificate_hash` を埋めた
`ChallengeGenerationRequest` を構築し、その後 `request_hash` を計算します。
この request construction phase は output path を作成・更新してはいけません。
construction phase で base certificate を読めない、または claimed hash を decode できない場合は、
`ChallengeGenerationRequest` を作らず、対応する generation `CommandError` を返します。
generator core は CLI front-end が埋めた base certificate hash を信用せず、
request hash validation 後に base certificate を再読込して再検証します。
`request_hash` が存在しない場合は `generation_request_hash_missing`、
`request_hash` が present だが wrong type / explicit null / invalid hash format の場合は
`generation_request_schema_invalid` です。
generator core と API は request self-hash 再計算の前に、
`request_hash` field を含む full `ChallengeGenerationRequest` schema / domain validation を行います。
複数の request schema / domain failure が同時に存在する場合は、schema 定義の field 出現順を
深さ優先でたどり、object member は schema に書かれた順で最初の failure を返します。
この schema 定義の top-level field order は `schema`、`request_id`、`request_hash`、
`challenge_id`、`policy_hash`、`module`、`imports`、`base_certificate`、`mutation`、
`output`、`generated_by` の順です。
nested object の field order は、`imports.mode`、`imports.manifest`、`imports.manifest_hash`、
`base_certificate.path`、`base_certificate.file_hash`、`base_certificate.claimed_certificate_hash`、
`mutation.kind`、`mutation.target`、`mutation.seed`、
`output.store_manifest_path`、`output.manifest_path`、`output.mutated_certificate_path`、
`generated_by.kind`、`generated_by.prompt_hash` の順です。
unknown / forbidden field は同じ object の定義済み field をすべて検査した後に評価し、
複数ある場合は JSON source 内の member 出現順で最初の failure を返します。
duplicate field は、その field の schema order 位置で `duplicate_field` として扱います。
この schema / domain validation が通った後でだけ 3.3 の規則で request hash を再計算し、
一致しない場合は `generation_request_hash_mismatch` です。
この検査に失敗した場合、`ChallengeManifest`、mutated certificate、
challenge output store manifest を作成または更新してはいけません。
`challenge_id` は generated `ChallengeManifest.challenge_id` にそのまま写します。
`policy_hash` は generated `ChallengeManifest.policy_hash` にそのまま写します。
generator と API は `ChallengeGenerationRequest.policy_hash`、`RunnerPolicyReference.hash`、
読み込んだ `RunnerPolicy` の canonical hash がすべて一致することを検査します。
`module` と `imports` は generated `ChallengeManifest` にそのまま写し、
challenge replay request materialization の正本入力として使います。
`imports.manifest` と `imports.manifest_hash` は `MachineCheckRequest.imports` と同じ意味です。
`imports.mode` も required で、`RunnerPolicy.import_policy.mode` と一致しなければなりません。
generator は `imports.manifest_hash` を import lock file bytes の sha256 と照合しなければなりません。
generator は `base_certificate.path` の file bytes sha256 を計算し、
`base_certificate.file_hash` と照合しなければなりません。
さらに base certificate から claimed certificate hash を decode し、
`base_certificate.claimed_certificate_hash` と照合しなければなりません。
この検査は CLI と API の両方で必須です。
API request body の `base_certificate.file_hash` と `base_certificate.claimed_certificate_hash` は
期待値であり、generator が再計算せずに信用してはいけません。
ここでの base certificate claimed hash decode は、full `ModuleCertBytes` decode ではなく、
raw claimed-hash extractor で `ModuleHashes.certificate_hash` field byte range / value bytes を exactly one として
識別する処理です。
raw claimed-hash extractor は byte-level mutation の raw framing scanner と同じ top-level
section scanning を使います。
Header と file end までの top-level section sequence を読み、unknown section kind、
Header 後の識別不能 byte、trailing garbage、duplicate top-level section、
out-of-order section、overlap、file size を越える length を失敗にします。
ただし `ModuleCertBytes` の full decode や canonical re-encode は行いません。
extractor は `ModuleHashes` section payload だけを local decode し、
Phase 2 の `ModuleHashes = export_hash, axiom_report_hash, certificate_hash` の固定 field sequence が
payload 末尾までちょうど読めることを要求します。
`ModuleHashes` payload 内の missing field、field order violation、extra field、
trailing payload byte、invalid hash encoding は extractor failure です。
成功条件は exactly one `ModuleHashes` section と exactly one
`ModuleHashes.certificate_hash` field byte range / value byte range を識別できることです。
この base certificate claimed hash decode は mutation-specific validation より先です。
decode 失敗は、後続の byte-level source format check や raw layout failure より先に
`base_certificate_claimed_hash_decode_failed` として返します。
したがって byte-level mutation でも、top-level raw framing failure と `ModuleHashes` local decode failure は
`mutation_target_invalid` ではなく `base_certificate_claimed_hash_decode_failed` です。
claimed hash decode が通った後で、byte-level mutation 共通の
`Header.format != NPA-CERT-0.1` を `mutation_target_invalid` として評価します。
base certificate が読めない、file hash が一致しない、claimed hash を decode できない、
または claimed hash が一致しない場合は generation validation failure です。
この場合、`ChallengeManifest`、mutated certificate、challenge output store manifest を
作成または更新してはいけません。
`challenge_id` は non-empty ASCII string で、`[a-z][a-z0-9_]*` に一致しなければなりません。
generator は `request_id`、output path、mutation seed から `challenge_id` を推測してはいけません。
MVP の challenge output store は `output.store_manifest_path` が指す manifest file で定義します。
directory scan や `manifest_path` の親 directory から store 境界を推測してはいけません。
`output.store_manifest_path` は required です。
同じ challenge output store manifest 内で同じ `challenge_id` を再利用してはいけません。
ただし retry 時に既存 entry の `challenge_id`、`manifest_path`、`manifest_hash` が
今回生成する entry と完全一致する場合は idempotent success として扱います。
同じ `challenge_id` が異なる entry に結びつく場合は generation failure です。
`ChallengeManifest.replay.args_hash` はこの `request_hash` と同じ値にします。
`ChallengeManifest.policy_hash` は challenge generation / replay に使う
`RunnerPolicy` の canonical hash です。
release / nightly pipeline では、この値が `ReleasePolicy.challenge_runner_policy_hash` と
一致しなければなりません。
MVP では `challenge_id`、`policy_hash`、`module`、`imports`、`mutation.kind`、`mutation.target`、
`mutation.seed` はすべて required です。
`ChallengeManifest.mutation.kind` は `^[a-z][a-z0-9_]{0,63}$` の string です。
この節の MVP challenge 種別に含まれる kind だけを rejection-required と分類し、
それ以外の grammar-valid kind は informational と分類します。
grammar violation、wrong type、explicit null、missing は `ChallengeManifest` schema / domain invalid です。
informational `ChallengeManifest.mutation.kind` の `mutation.target` は generator が解釈しない opaque target label です。
manifest-local validation では、informational target は non-empty string で、
各 character が visible ASCII `U+0021` から `U+007E` の範囲、長さが1..=255 byte でなければなりません。
空白、control character、non-ASCII、explicit null、wrong type、missing は
`ChallengeManifest` schema / domain invalid です。
target text format violation では `actual_value = "invalid_name_format"` を使います。
informational target は existing declaration / import / axiom lookup や `$whole_certificate` 照合を行わず、
`mutation_target_invalid` にも使いません。
`ChallengeGenerationRequest.mutation.kind` と CLI `npa-check challenge generate --kind` は
MVP challenge 種別の closed enum だけを受け付け、informational kind を生成してはいけません。
`ChallengeGenerationRequest.mutation.kind` がこの closed enum 外なら
`generation_request_schema_invalid` / `invalid_enum` であり、`mutation_target_invalid` ではありません。
`mutation.seed` は `sha256:<lower-hex>` 形式で、generator が mutation point を選ぶ唯一の乱択入力です。
mutation selection に使う seed bytes は、`sha256:` prefix の後ろの lower-hex を decode した32 byteです。
`sha256:` 付き文字列の UTF-8 bytes を seed bytes として使ってはいけません。
同じ request bytes、同じ generator binary、同じ base certificate bytes からは
同じ mutated certificate bytes と同じ manifest bytes が生成されなければなりません。

MVP の `ChallengeOutputStoreManifest` schema：

```json
{
  "schema": "npa.phase8.challenge_output_store_manifest.v1",
  "entries": [
    {
      "challenge_id": "pch_001",
      "manifest_path": "build/challenges/pch_001/manifest.json",
      "manifest_hash": "sha256:..."
    }
  ]
}
```

`entries` は `challenge_id` の bytewise lexicographic order で昇順に並べます。
`challenge_id` と `manifest_path` はそれぞれ unique です。
`manifest_hash` は保存された `ChallengeManifest` file bytes の sha256 です。
challenge generation は store manifest を読み、duplicate check を行ってから出力を書きます。
store manifest が存在しない場合は empty store として扱います。
store manifest が存在するが読めない、JSON として壊れている、schema 違反、
entry の `manifest_path` が読めない、参照先 `ChallengeManifest` が manifest-local
JSON / schema / domain validation を通らない、または entry の `manifest_hash` と参照先
`ChallengeManifest` file bytes が一致しない場合は generation failure です。
store manifest entry の `manifest_path` schema violation は store manifest schema / domain failure です。
schema-valid `manifest_path` が symlink escape などで owning root 外へ解決される場合は
entry manifest unreadable と同じ generation failure にします。
既存 store entry の referenced `ChallengeManifest` validation は manifest-local です。
`challenge_id` format、required / forbidden / unknown / null / duplicate field、hash format、
path format、`mutation.kind` の分類、`mutation.target` の kind 別 target grammar、
base / mutated certificate metadata の field shape を検査しますが、
`base_certificate.path`、`mutated_certificate.path`、`imports.manifest`、policy file、
import lock file などの外部 file は読みません。
したがって manifest-local validation では `mutated_certificate.claimed_certificate_hash` を optional field として扱い、
存在する場合は hash format だけを検査します。
manifest-local validator は mutated certificate file を読んで raw claimed-hash extractor を再実行してはならず、
extractor 成功時にこの field が required だったかどうかも判定しません。
`mutated_certificate.claimed_certificate_hash` を extractor 結果に応じて書く責務は challenge generator にだけあります。
`ChallengeOutputStoreManifest` file 自体の expected hash は generation request では受け取りません。
そのため MVP の challenge generation は、同じ `output.store_manifest_path` に対して
single writer / externally serialized execution を前提にします。
複数 generator を同じ store manifest に並行して書き込んではいけません。
並行生成が必要な pipeline は、challenge output store を shard するか、store manifest read から
atomic replace commit までを外部 lock で直列化します。
MVP generator は compare-and-swap 用の previous store hash を受け取らず、lost update の検出を
trust boundary に含めません。
generation 成功時は、生成した `ChallengeManifest` file bytes hash を使って新しい entry を追加し、
sort 済みの store manifest を `output.store_manifest_path` に書き戻します。
store manifest の書き戻しに失敗した場合は generation failure です。
この書き戻しは challenge output store manifest の更新であり、
既存 artifact file の上書き禁止規則の例外です。
challenge output store manifest が generation の commit point です。
実装は mutated certificate、`ChallengeManifest`、challenge output store manifest を
temporary file として作り、final mutated certificate path と final manifest path を配置してから
store manifest を atomic replace します。
store manifest が generated entry を参照して初めて generation 成功です。
store manifest commit 前に failure した場合、store manifest を更新してはいけません。
temporary file は best-effort で削除します。
store manifest に参照されない orphan challenge file は challenge output store reader が無視します。
retry 時に `output.manifest_path` または `output.mutated_certificate_path` が既に存在し、
その file bytes が今回生成する bytes と完全一致する場合は、上書きではなく既存 file の採用として扱います。
既存 output file の bytes が異なる場合は path conflict です。
既存 store manifest に同じ `challenge_id`、`manifest_path`、`manifest_hash` の entry が既にある場合は
idempotent success として扱います。
同じ `manifest_path` が異なる `challenge_id` または異なる `manifest_hash` に結びつく場合は
`challenge_output_store_entry_conflict` です。
exact-match adoption や idempotent retry の成功時も `status = written` を返し、
別の `adopted` status は作りません。

`npa-check challenge generate --json` の成功時 stdout と `/machine/check/challenge` の成功 response は
`ChallengeGenerationResult` です。
API は mutated certificate bytes を response body に埋め込まず、保存先 path と hash だけを返します。

MVP の `ChallengeGenerationResult`：

```json
{
  "schema": "npa.phase8.challenge_generation_result.v1",
  "status": "written",
  "challenge_id": "pch_001",
  "request_hash": "sha256:...",
  "policy_hash": "sha256:...",
  "challenge_manifest": {
    "path": "build/challenges/pch_001/manifest.json",
    "manifest_hash": "sha256:..."
  },
  "mutated_certificate": {
    "path": "build/challenges/pch_001/Std.Nat.mutated.npcert",
    "file_hash": "sha256:...",
    "claimed_certificate_hash": "sha256:..."
  },
  "challenge_store": {
    "kind": "manifest",
    "path": "build/challenges/manifest.json",
    "manifest_hash": "sha256:..."
  }
}
```

`challenge_manifest.manifest_hash` は保存した `ChallengeManifest` file bytes の sha256 です。
`mutated_certificate.file_hash` は保存した mutated certificate file bytes の sha256 です。
`ChallengeGenerationResult.mutated_certificate.claimed_certificate_hash` は raw claimed-hash extractor が
mutated certificate 内の `ModuleHashes.certificate_hash` field byte range / value bytes を exactly one として
識別できる場合だけ required です。
この extractor は full `ModuleCertBytes` decode ではなく、raw header / section framing から
`ModuleHashes` payload を local decode し、`ModuleHashes.certificate_hash` field byte range /
value bytes を exactly one として識別する処理です。
したがって `insert_unsupported_schema_version` で Header.format が `NPA-CERT-9.9` になっても、
`ModuleHashes.certificate_hash` field / value byte range を一意に識別できる場合は
`mutated_certificate.claimed_certificate_hash` を必ず書きます。
raw claimed-hash extractor が一意に識別できない mutation では omit します。
`challenge_store.manifest_hash` は generation 後の `ChallengeOutputStoreManifest` file bytes の sha256 です。
`policy_hash` は input `RunnerPolicyReference.hash` です。
`request_hash` は input `ChallengeGenerationRequest.request_hash` です。
`ChallengeGenerationResult` は transient response であり、`result_hash` を持ちません。

MVP の `mutation.target` は次の deterministic selector です。

```text
declaration target:
  change_declaration_body_without_hash
  change_declaration_hash_without_body
  drop_axiom_report_entry
  alter_de_bruijn_index
  replace_nat_zero_with_noncanonical_placeholder
  remove_dependency_entry
  target = declaration full name, e.g. Nat.add_zero

import target:
  replace_import_export_hash
  remove_dependency_entry
  target = imported module full name

axiom target:
  add_forbidden_axiom
  target = fresh axiom full name to introduce

whole certificate target:
  flip_canonical_encoding_byte
  reorder_declarations
  insert_unsupported_schema_version
  truncate_certificate_section
  target = "$whole_certificate"
```

declaration / import / axiom target の `mutation.target` は 3.3 の Phase 8 name JSON representation に従います。
name grammar violation は request / manifest schema failure の `invalid_name_format` であり、
`mutation_target_invalid` ではありません。
MVP known mutation kind の whole certificate target は bytewise に `"$whole_certificate"` でなければなりません。
whole certificate target が別 string の場合は request / manifest schema failure で
`actual_value = "invalid_enum"`、wrong type、explicit null、missing の場合はそれぞれ
`wrong_type`、`null_not_allowed`、`missing` を使います。
informational `ChallengeManifest.mutation.kind` の target は上記の opaque informational target label grammar だけを検査し、
この MVP target class table では分類しません。
`mutation_target_invalid` は name grammar が valid な existing target が対象 certificate に存在しない場合、
mutation kind と target class が合わない場合、または fresh target が既に存在する場合に使います。
`remove_dependency_entry` は declaration target と import target の両方を受け付けます。
ただし同じ `mutation.target` が current module の declaration name と imported module name の両方に
一致する場合は ambiguous target class として `mutation_target_invalid` です。
generator は declaration target と import target のどちらかを優先して選んではいけません。
import target は base certificate の `Imports` 内で `ImportEntry.module_name` が target と一致する
entry が exactly one の場合だけ解決します。
0件なら target lookup failure、2件以上なら ambiguous import target として `mutation_target_invalid` です。
module name が同じ複数 import のうち export_hash や certificate_hash で1件を選び直してはいけません。
`add_forbidden_axiom` だけは existing axiom を探す mutation ではありません。
その `mutation.target` は追加する fresh axiom declaration name であり、base certificate の current module
declaration / export block に同名 declaration が存在してはいけません。

MVP の mutation execution rule：

```text
mutation kind classification:
  structured mutation:
    change_declaration_body_without_hash
    change_declaration_hash_without_body
    drop_axiom_report_entry
    alter_de_bruijn_index
    replace_nat_zero_with_noncanonical_placeholder
    replace_import_export_hash
    remove_dependency_entry
    reorder_declarations
    add_forbidden_axiom

  byte-level mutation:
    flip_canonical_encoding_byte
    insert_unsupported_schema_version
    truncate_certificate_section

common:
  - generator は base certificate file bytes を読み、request の file_hash と照合する。
  - generator は claimed certificate hash を decode し、request の claimed_certificate_hash と照合する。
  - structured mutation は base certificate を `NPA-CERT-0.1` `ModuleCertBytes` として decode してから行う。
  - structured mutation で base certificate を decode できない場合は base_certificate_decode_failed。
    ここでの decode は Phase 2 canonical binary decode です。
    `Header.format` / `Header.core_spec`、minimal ULEB128、UTF-8、top-level section order / completeness、
    table topological order、duplicate table entry、table id reference、`GlobalRef::LocalGenerated` の
    declaration kind / generated name 整合、`AxiomReport.per_declaration` order など、
    Phase 2 decoder が `ModuleCert` object を構成するために必要な canonical structural invariant を検査します。
    decode 後に Phase 2 canonical encoding で再 encode した bytes が base certificate file bytes と
    bytewise に一致しない入力も `base_certificate_decode_failed` です。
    structured mutation の decode は full verifier / checker ではありません。
    import certificate loading、import export hash 照合、declaration hash 再計算、axiom report hash 再計算、
    certificate hash 再計算、kernel type check、axiom policy evaluation は行いません。
    これら semantic verifier failure は、mutation target / candidate collection に必要な構造が decode できる限り
    generation failure にせず、replay checker result に委ねます。
  - structured mutation は decode 済み payload を Phase 2 canonical encoding で再 encode した
    `canonical_base_bytes` を mutation 入力 bytes として使う。
    candidate の `canonical byte offset` は、この `canonical_base_bytes` 先頭からの 0-based byte offset です。
    structured mutation は original base certificate file bytes に対して patch してはいけません。
    encoded-byte patch mutation は `canonical_base_bytes` に直接 patch し、patch 後に再 decode / 再 encode しません。
    object mutation は decode 済み payload object を変更してから Phase 2 canonical encoding で再 encode します。
    `add_forbidden_axiom` だけは後述の通り canonical table / declaration hash / module hash pipeline も再実行します。
    それ以外の object mutation は必要な length prefix / table bytes は canonical encoding で更新しますが、
    明示された stored hash fields は再計算せず、変更前の hash bytes を保持します。
    MVP の encoded-byte patch mutation は `change_declaration_body_without_hash`、
    `change_declaration_hash_without_body`、`replace_nat_zero_with_noncanonical_placeholder` です。
    MVP の object mutation は `drop_axiom_report_entry`、`alter_de_bruijn_index`、
    `replace_import_export_hash`、`remove_dependency_entry`、`reorder_declarations`、`add_forbidden_axiom` です。
  - object / field candidate の offset は、その object / field の encoded bytes の first byte offset です。
    Term graph candidate は reachable `TermTable` node ごとに1 candidate とし、参照 edge ごとには数えない。
    同じ `TermTable` node が複数箇所から参照される場合も1 candidate です。
    term node candidate の offset はその `TermNode` tag byte の offset です。
    `alter_de_bruijn_index` と `replace_nat_zero_with_noncanonical_placeholder` の term graph root は
    target declaration の `DeclPayload` 種別ごとに固定します。
    `AxiomDecl` は `type`、`DefDecl` は `type` と `value`、`TheoremDecl` は `type` と `proof`、
    `InductiveDecl` は `params[].type`、`indices[].type`、`constructors[].type`、
    および `recursor.type` が present の場合のその `type` です。
    これら root から reachable な `TermTable` node だけを候補探索対象にし、
    dependency entry、axiom report、export block、import certificate、elaborator side data から
    追加 root を導出してはいけません。
  - byte-level mutation は full `ModuleCertBytes` decode を要求せず、`canonical_base_bytes` を使わない。
    base certificate claimed hash decode を先に行い、そこで検証済み raw section map と
    `ModuleHashes.certificate_hash` field / value byte range を得る。
    ここで失敗した場合は `base_certificate_claimed_hash_decode_failed` であり、
    `mutation_target_invalid` ではありません。
    その後で共通の raw Header.format source format check を行い、さらにその mutation が必要とする
    raw header / section framing だけを読む。
    MVP のすべての byte-level mutation は、base certificate の Header.format string field byte range を
    一意に特定でき、かつ value が `NPA-CERT-0.1` であることを要求します。
    Header.format byte range を一意に特定できない場合、または value が `NPA-CERT-0.1` でない場合は
    `mutation_target_invalid` です。
    Header.format byte range を一意に特定できない場合は `actual_value = "missing_header_format"`、
    一意に特定できるが value が `NPA-CERT-0.1` でない場合は
    `actual_value = "unexpected_header_format:<actual_header_format>"` です。
    この source format failure は `flip_canonical_encoding_byte` / `truncate_certificate_section` の
    candidate collection より先に評価します。
    byte-level mutation の byte offset は original base certificate file bytes の 0-based offset です。
    byte-level mutation は base certificate claimed hash decode と異なる raw section map を作ってはいけません。
    unknown section kind、Header 後の識別不能 byte、trailing garbage、duplicate section、
    out-of-order section、overlap、file size を越える length は、すべて先行する
    `base_certificate_claimed_hash_decode_failed` として扱います。
    claimed hash decode 成功後の `mutation_target_invalid` は、その mutation が要求する
    Header field / section / candidate byte range が検証済み raw section map から得られない場合だけです。
    section kind の欠落が `mutation_target_invalid` か candidate absence かは mutation kind ごとの rule で決めます。
    `flip_canonical_encoding_byte` は trusted payload 範囲と `ModuleHashes.certificate_hash` field byte range を
    検証済み raw section map と `ModuleHashes` local decode result から特定できなければならない。
    `ModuleHashes.certificate_hash` value byte range が exactly one でない入力は、ここに到達する前に
    `base_certificate_claimed_hash_decode_failed` です。
    `insert_unsupported_schema_version` は Header.format string field byte range だけを
    mutation-specific raw target として要求する。
    `truncate_certificate_section` は Header 以外の識別済み top-level section のうち、
    raw framing から start / end byte range を一意に特定でき、かつ non-empty な section だけを candidate にする。
    byte-level mutation が必要な raw layout または candidate を見つけられない場合は
    `base_certificate_decode_failed` ではなく `mutation_target_invalid` です。
  - existing declaration / import target が存在しない場合は mutation_target_invalid。
  - fresh axiom target for `add_forbidden_axiom` が既に current module に存在する場合は mutation_target_invalid。
  - candidate を持つ mutation の selection algorithm は structured / byte-level 共通です。
    candidate set を mutation kind ごとの deterministic order に並べ、
    hex decode 済み seed bytes の先頭8 byteを unsigned big-endian integer として解釈し、
    index = value mod candidate_count で1件を選ぶ。
    追加の PRNG、再 hash、rejection sampling、`sha256:` 付き文字列の UTF-8 bytes は使いません。
    structured mutation の candidate order は、各 rule が別順序を明記しない限り canonical byte offset 昇順です。
    byte-level mutation の candidate order は各 rule で定義し、同一 section 内の tie-break は
    original base certificate file bytes の 0-based offset 昇順です。
  - candidate_count = 0 の場合は mutation_target_invalid。
  - `add_forbidden_axiom` 以外の mutation は、特に明記しない限り既存の stored hash field を
    recompute せず、改変前の hash bytes を残す。

change_declaration_body_without_hash:
  target declaration の body / proof / value term の canonical encoded bytes から
  mutable payload byte candidates を canonical byte offset 昇順で集め、seed で1 byte を選ぶ。
  MVP での body / proof / value term root は `DefDecl.value` と `TheoremDecl.proof` だけです。
  `AxiomDecl` と `InductiveDecl` にはこの mutation の body / proof / value root がないため、
  candidate set empty として `mutation_target_invalid` です。
  candidate 探索対象はその root から reachable な `TermTable` node の encoded bytes です。
  その `TermTable` node が他 declaration、type root、export block からも参照されている場合でも
  candidate から除外せず、generator は shared node を clone / localize してはいけません。
  mutable payload byte は term tag、length prefix、section framing、stored hash field に属さない
  term payload byte だけです。
  ここで stored hash field とは、Phase 2 encoding 内の hash-typed field 全般です。
  `TermNode::Const(GlobalRef::Imported(_, _, decl_interface_hash), _)` の
  `decl_interface_hash` bytes も stored hash field とみなし、candidate に含めません。
  selected byte を xor 0x01 で変更する。
  mutable payload byte が存在しない場合は mutation_target_invalid。
  resulting bytes が decode 不能になる場合も generation failure ではなく checker result を oracle にする。
  stored declaration hash、export_hash、axiom_report_hash、certificate_hash は更新しない。

change_declaration_hash_without_body:
  target declaration の stored `DeclHashes.decl_interface_hash` first byte と
  `DeclHashes.decl_certificate_hash` first byte を candidate として canonical byte offset 昇順に並べ、
  seed で1つ選び xor 0x01 で変更する。
  body / type / proof bytes は変更しない。
  module-level hash field は更新しない。

drop_axiom_report_entry:
  target declaration の `DeclAxiomReport` entry を削除する。
  `AxiomReport` 内の sort order は残った entry の既存順を保ち、axiom_report_hash と
  certificate_hash は更新しない。
  target declaration は存在するが対応する `DeclAxiomReport` entry が存在しない場合は
  target lookup failure ではなく candidate set empty として `mutation_target_invalid` です。

alter_de_bruijn_index:
  target declaration の term graph 内の `BVar` candidates を canonical byte offset 昇順で集め、
  seed で1つ選び、index を index + 1 に変更する。
  stored declaration / module hash は更新しない。

replace_nat_zero_with_noncanonical_placeholder:
  target declaration の term graph 内で reachable `TermTable` node が canonical `Const Nat.zero` である
  node candidates を
  canonical byte offset 昇順で集め、seed で1つ選ぶ。
  ここで reference candidate とは `Const Nat.zero` を表す reachable node のことであり、
  同じ node への参照 edge が複数あっても candidate は1件です。
  `canonical Const Nat.zero` は `TermNode::Const(global_ref, levels)` で、
  `levels` が empty、かつ `global_ref` から encoded certificate 内だけで解決できる name が
  decoded Phase 2 `Name(["Nat", "zero"])` と一致する場合だけです。
  `GlobalRef::Imported(_, name, _)` と `GlobalRef::LocalGenerated(_, name)` ではその `name` を使い、
  `GlobalRef::Local(decl_index)` では current module の `Declarations[decl_index].decl.name` を使います。
  `GlobalRef::LocalGenerated(decl_index, name)` は、`decl_index` が current module declarations の範囲内で、
  その declaration が `InductiveDecl` であり、`constructors[].name` または present な `recursor.name` に
  同じ name が存在する場合だけ解決済み candidate とします。
  `decl_index` が current module declarations の範囲外、または参照先 declaration kind / generated name が
  整合しない場合は candidate にしません。
  import certificate や export block を開いて別名解決してはいけません。
  selected term node tag を reserved invalid tag 0xff に置き換える。
  core calculus に placeholder term を追加するわけではない。
  checker はこの bytes を `noncanonical_encoding` または `certificate_decode_error` として拒否する。

replace_import_export_hash:
  unique に解決した target import entry の stored `export_hash` first byte を xor 0x01 で変更する。
  referenced import certificate file は変更せず、module-level certificate_hash も更新しない。

remove_dependency_entry:
  declaration target の場合は、target declaration の `DeclCert.dependencies` 内の entry を
  canonical byte offset 昇順で candidate にし、seed で1件削除する。
  import target の場合は、unique に解決した target import entry の import index を使い、
  current module の全 `DeclCert.dependencies` から、`DependencyEntry.global_ref` が
  `GlobalRef::Imported(import_index, _, _)` である entry を candidate にする。
  import target candidate は owner `DeclCert` の canonical byte offset、次に dependency entry の
  canonical byte offset の昇順で並べ、seed で1件削除する。
  dependency entry が存在しない場合は mutation_target_invalid。
  stored declaration / module hash は更新しない。

add_forbidden_axiom:
  target axiom name の fresh `AxiomDecl` を current module の declarations の末尾に追加する。
  mutated certificate の stored declaration order は、base certificate の declaration order を保ったまま、
  末尾にこの axiom を1件追加した順序に固定する。
  name order で挿入位置を選び直してはいけません。
  target name は Phase 8 name grammar を満たせばよく、current module name を prefix として持つことは要求しない。
  fresh check は current module の `Declarations` と `ExportBlock` に対する Phase 2 name equality だけで行う。
  target name が base certificate の `Declarations` または `ExportBlock` のどちらかに存在する場合は
  `mutation_target_invalid` です。
  base certificate の `Declarations` と `ExportBlock` が mutation 前から互いに不整合でも、
  generator はそれを修復せず、それだけを理由に generation failure にもしません。
  target name がどちらにも存在しない場合、generator は axiom を追加してよく、
  pre-existing inconsistency の最終判定は replay checker result に委ねます。
  generator は import certificate を開いて imported export name との衝突を調べてはいけません。
  imported name collision が checker / kernel で問題になる場合は replay checker result に委ねる。
  追加する `AxiomDecl.universe_params` は empty vector に固定し、その type は
  `Sort Level::Zero`、つまり universe level zero の sort に固定する。
  ここでの zero は `Nat.zero` ではない。
  追加する `DeclCert.dependencies` は empty vector です。
  追加する `DeclCert.axiom_dependencies` は、追加 axiom 自身を指す `AxiomRef` だけを含める。
  この self `AxiomRef.global_ref` は `GlobalRef::Local(new_decl_index)` で、
  `new_decl_index` は末尾追加後の axiom declaration index です。
  `AxiomRef.name` は target axiom name です。
  `AxiomRef.decl_interface_hash` は追加 axiom の `decl_interface_hash` 再計算結果を使う。
  generator は table id を手で選ばず、既存 declarations と末尾追加 axiom から
  Phase 2 の canonical table / declaration hash / module hash pipeline を再実行する。
  declaration order、export block、axiom report、decl hash、export_hash、axiom_report_hash、
  certificate_hash は canonical rules で再計算する。
  この mutation は policy violation を検査するため、hash mismatch を意図的に先に起こさない。
  generator は axiom policy allowlist を oracle として信用せず、target 名が実際に forbidden かどうかの
  最終判定は replay の `MachineCheckResult.error.kind = forbidden_axiom` または他の checker result に委ねる。

flip_canonical_encoding_byte:
  trusted payload bytes から `ModuleHashes.certificate_hash` value bytes だけを除いた candidate byte offset を
  Imports, NameTable, LevelTable, TermTable, Declarations, ExportBlock, AxiomReport, ModuleHashes
  の順で集め、seed で1 byte を選び xor 0x01 する。
  ここで trusted payload bytes は Header を除く top-level section payload bytes です。
  `ModuleHashes` は先行する raw claimed-hash extractor の成功によって exactly once が保証済みです。
  `ModuleHashes` が欠落、duplicate、malformed の入力はここに到達する前に
  `base_certificate_claimed_hash_decode_failed` です。
  Imports, NameTable, LevelTable, TermTable, Declarations, ExportBlock, AxiomReport の7 section は
  `flip_canonical_encoding_byte` の required section であり、欠落した場合は
  `mutation_target_invalid` です。
  この順序は raw file の section 出現順ではなく、raw framing validation 後の Phase 2 top-level section order です。
  top-level section tag / section id / section length prefix などの section framing bytes は candidate に含めない。
  section payload 内に含まれる object tag、object length、table length、term tag、stored hash bytes は candidate に含める。
  ただし `ModuleHashes.certificate_hash` value bytes は candidate から除外し、その field の tag / length bytes が
  section payload 内に存在する場合は candidate に含める。
  resulting bytes がたまたま valid certificate になる場合も outcome_hint ではなく checker result を oracle にする。

reorder_declarations:
  adjacent declaration pair candidates を declaration order 昇順で集め、seed で1組を選んで swap する。
  declaration order は decoded `Declarations` vector の index order です。
  `declarations.len() < 2` の場合は candidate set empty として `mutation_target_invalid` です。
  pair candidate は `(decl_index, decl_index + 1)` で、candidate order は `decl_index` 昇順です。
  selected pair は `Declarations` vector 内の2つの `DeclCert` object だけを入れ替えます。
  generator は `GlobalRef::Local` / `LocalGenerated` index、`AxiomReport.per_declaration[].decl_index`、
  `ExportBlock` entry order、dependency / axiom refs、term tables、name tables、stored declaration hash、
  export_hash、axiom_report_hash、certificate_hash を追従更新してはいけません。
  mutation 後 bytes は object mutation として Phase 2 canonical encoding で再 encode しますが、
  各 `DeclCert` の encoded payload bytes と stored hashes は swap 前の object 内容をそのまま保持します。

insert_unsupported_schema_version:
  Header.format を same-length ASCII string `NPA-CERT-9.9` に置き換える。
  MVP の source format `NPA-CERT-0.1` と replacement `NPA-CERT-9.9` は byte length が同じなので、
  string length field は変更しない。
  この mutation は共通 byte-level source format check が成功した後にだけ置換を行います。
  Header.format が `NPA-CERT-0.1` 以外、または Header.format byte range を raw framing から
  一意に特定できない場合は `mutation_target_invalid` です。
  Header.format byte range を一意に特定できるすべての mismatch は、値が UTF-8 として読めるかどうかに
  かかわらず、`actual_value` に下の `unexpected_header_format:<actual_header_format>` 正規化ルールを使います。
  module hashes は更新しない。

truncate_certificate_section:
  Header 以外の top-level section candidates を
  Imports, NameTable, LevelTable, TermTable, Declarations, ExportBlock, AxiomReport, ModuleHashes
  の順で集め、seed で1つ選ぶ。
  欠落した section kind は candidate から除外し、それだけを理由に failure にしません。
  candidate section は raw framing から start / end byte range を一意に特定できる section だけです。
  `section_len = section_end - section_start` とし、`section_len > 0` の section だけを candidate にする。
  `section_len / 2` は integer floor division です。
  selected section の encoded bytes の後半を deterministic half-open range
  `[section_start + section_len / 2, section_end)` で削除する。
```

MVP の `npa-check challenge generate` と `/machine/check/challenge` は、
schema / domain validation を通過した後で、上記の target rule に合わない request を
`mutation_target_invalid` の generation validation failure として拒否し、
`ChallengeGenerationResult` と `ChallengeManifest` を返してはいけません。
`mutation.target` の name grammar violation、whole certificate target の `"$whole_certificate"` 以外の string、
wrong type、explicit null、missing、informational kind の closed enum violation など、
上で schema / domain failure と定義した target failure は `generation_request_schema_invalid` であり、
`mutation_target_invalid` ではありません。
`generated_by.kind` は `ci` または `ai` です。
`generated_by.kind = ai` の場合は `prompt_hash` が required で、
`generated_by.kind = ci` の場合は `prompt_hash` を omit します。
`output.store_manifest_path`、`output.manifest_path`、`output.mutated_certificate_path` は
workspace-relative path です。
API の `/machine/check/challenge` では、これらの path と
`base_certificate.path`、`imports.manifest` は inline `ChallengeGenerationRequest` 内の artifact field です。
したがって wrapper path validation の `ApiError` ではなく、request schema / domain validation として扱い、
path schema violation は `CommandError.reason_code = generation_request_schema_invalid`、
`field = "output.<path_field>"`、`"base_certificate.path"`、または `"imports.manifest"`、
`actual_value = "invalid_path"` で返します。
schema-valid path が symlink escape などで owning root 外へ解決される場合は path schema violation ではなく
filesystem safety failure です。
`base_certificate.path` と `imports.manifest` では、それぞれ対応する unreadable / hash mismatch 前の
file access failure として扱い、`base_certificate_file_unreadable` または
`import_manifest_file_unreadable` を返します。
`output.store_manifest_path` で既存 store manifest を読む段階なら `challenge_output_store_file_unreadable`、
commit / write 段階なら `challenge_output_store_write_failure` を返します。
`output.manifest_path` と `output.mutated_certificate_path` では対応する `*_write_failure` を返し、
`actual_value = "write_failed"` にします。
generator は指定された path 以外に正本 artifact を書いてはいけません。
既存 `output.manifest_path` と `output.mutated_certificate_path` の上書きは MVP では禁止し、
既存 file がある場合は、今回生成する bytes と完全一致する場合だけ exact-match adoption として扱います。
bytes が異なる既存 file は generation failure です。
既存 `output.store_manifest_path` は duplicate check 後に更新する唯一の例外です。
generation failure では `ChallengeGenerationResult` を返してはいけません。
CLI の `--json` では exit code 1、stdout empty、stderr に `CommandError` JSON を1個だけ出します。
API では wrapper validation 通過後の domain validation error body として
同じ `CommandError` object を返します。
この error body は release audit bundle の artifact kind には含めません。
challenge 系 command、つまり `challenge generate`、`challenge materialize-requests`、
`challenge replay` の `RunnerPolicyReference` error boundary は共通です。
CLI で required な `--policy` または `--policy-hash` が欠落した場合は CLI argument error であり、
`CommandError` body を返しません。
両方の flag が存在した後の malformed policy reference は
`CommandError.reason_code = policy_reference_invalid` として返します。
API では policy reference object の missing / wrong type / explicit null / unknown field /
invalid kind / invalid hash format / duplicate key は wrapper schema validation failure なので
`ApiError.reason_code = api_request_schema_invalid` を返し、`CommandError` body を返しません。
API の `policy.path` が workspace path validation に失敗した場合は
`ApiError.reason_code = api_path_outside_workspace` を返します。
wrapper validation 通過後に policy file が読めない場合は
`CommandError.reason_code = policy_file_unreadable`、
policy file が JSON parse または `RunnerPolicy` schema / domain validation に失敗した場合は
`policy_reference_invalid`、読み込んだ policy の canonical hash が
`RunnerPolicyReference.hash` と一致しない場合は `policy_hash_mismatch` にします。
challenge command で `policy_hash_mismatch` が複数成立しうる場合は、
まず `RunnerPolicyReference.hash` と読み込んだ `RunnerPolicy` canonical hash の mismatch を検査し、
これが成立するなら `field = "policy.hash"` を返します。
この検査が通った後でだけ、command artifact 側の `policy_hash`
（generation request の `policy_hash`、または `ChallengeManifest.policy_hash`）と
`RunnerPolicyReference.hash` の mismatch を検査します。
`policy_reference_invalid` の field shape は次で固定します。
reference object 自体が missing / wrong type / explicit null の場合は
`field = "policy"`、`expected_value = "RunnerPolicyReference"`、
`actual_value` に `missing`、`wrong_type`、または `null_not_allowed` を入れます。
reference object が存在し、その member が不正な場合は
`field` に `policy.kind`、`policy.path`、`policy.hash`、または
`policy.<unknown_field_name>` を入れ、`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_enum`、
`invalid_path`、`invalid_hash_format`、`null_not_allowed`、`order_violation`、
`duplicate_field` のいずれかを入れます。
policy file の JSON parse failure では `field = "policy.path"`、
`actual_value = "invalid_json"`、policy schema / domain validation failure では
`field` に `policy.<RunnerPolicy JSON path>` を入れます。
root-level failure では `field = "policy"` とし、top-level `schema` failure では
`field = "policy.schema"` とします。
`expected_value` / `actual_value` は
4.1 の RunnerPolicy schema / domain validation field shape に従います。
MVP の generation `CommandError.reason_code` は次に限定します。

```text
- generation_request_schema_invalid
- generation_request_hash_missing
- generation_request_hash_mismatch
- policy_reference_invalid
- policy_file_unreadable
- policy_hash_mismatch
- import_mode_mismatch
- import_manifest_file_unreadable
- import_manifest_hash_mismatch
- base_certificate_file_unreadable
- base_certificate_file_hash_mismatch
- base_certificate_claimed_hash_decode_failed
- base_certificate_claimed_hash_mismatch
- base_certificate_decode_failed
- mutation_target_invalid
- challenge_output_store_file_unreadable
- challenge_output_store_json_invalid
- challenge_output_store_manifest_invalid
- challenge_output_store_entry_manifest_unreadable
- challenge_output_store_entry_manifest_invalid
- challenge_output_store_entry_manifest_hash_mismatch
- challenge_id_conflict
- challenge_output_store_entry_conflict
- challenge_manifest_output_path_conflict
- mutated_certificate_output_path_conflict
- mutated_certificate_write_failure
- challenge_manifest_write_failure
- challenge_output_store_write_failure
```

generation `CommandError` の field は固定します。
`generation_request_schema_invalid` では `field` に invalid request field の JSON path、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_enum`、
`invalid_hash_format`、`invalid_name_format`、`invalid_path`、`null_not_allowed`、`order_violation`、
`duplicate_field` のいずれかを入れます。
`generation_request_hash_missing` では `field = "request_hash"`、
`expected_value = "sha256:<lower-hex>"`、`actual_value = "missing"` にします。
`generation_request_hash_mismatch` では `field = "request_hash"`、
`expected_hash` に `ChallengeGenerationRequest` から再計算した hash、
`actual_hash` に request の `request_hash` を入れます。
`policy_reference_invalid` では challenge 系 command 共通の policy reference field shape に従います。
`policy_file_unreadable` では `field = "policy.path"`、`actual_value = "unreadable"` にします。
`policy_hash_mismatch` では `field = "policy.hash"`、
`expected_hash` に caller 指定 hash、`actual_hash` に読み込んだ `RunnerPolicy` の canonical hash を入れます。
`ChallengeGenerationRequest.policy_hash` が `RunnerPolicyReference.hash` と一致しない場合は
同じ `policy_hash_mismatch` を使い、`field = "policy_hash"`、
`expected_hash` に `RunnerPolicyReference.hash`、
`actual_hash` に request の `policy_hash` を入れます。
`import_mode_mismatch` では `field = "imports.mode"`、
`expected_value` に `RunnerPolicy.import_policy.mode`、`actual_value` に request の `imports.mode` を入れます。
`import_manifest_file_unreadable` では `field = "imports.manifest"`、`actual_value = "unreadable"` にします。
`import_manifest_hash_mismatch` では `field = "imports.manifest_hash"`、
`expected_hash` に request の `imports.manifest_hash`、`actual_hash` に import lock file bytes hash を入れます。
`base_certificate_file_unreadable` では `field = "base_certificate.path"`、`actual_value = "unreadable"` にします。
`base_certificate_file_hash_mismatch` では `field = "base_certificate.file_hash"`、
`expected_hash` に request の `base_certificate.file_hash`、
`actual_hash` に base certificate file bytes hash を入れます。
`base_certificate_claimed_hash_decode_failed` では
`field = "base_certificate.claimed_certificate_hash"`、`actual_value = "decode_failed"` にします。
`base_certificate_claimed_hash_mismatch` では
`field = "base_certificate.claimed_certificate_hash"`、
`expected_hash` に request の `base_certificate.claimed_certificate_hash`、
`actual_hash` に decoded claimed certificate hash を入れます。
`base_certificate_decode_failed` では `field = "base_certificate.path"`、
`actual_value = "decode_failed"` にします。
この reason は claimed certificate hash は decode できたが、structured mutation に必要な
`NPA-CERT-0.1` `ModuleCertBytes` として decode できない場合にだけ使います。
`mutation_target_invalid` の field shape は subcase ごとに固定します。
target lookup / target class / fresh target failure では `field = "mutation.target"`、
`expected_value = "target_rule:<mutation.kind>"`、
`actual_value` に request の target を入れます。
mutation-specific raw layout failure では `field = "mutation.raw_layout"`、
`expected_value = "raw_layout_for:<mutation.kind>"` とします。
`actual_value` は次のいずれかに固定します。

```text
- missing_header_format
- unexpected_header_format:<actual_header_format>
- missing_required_section:<section_kind>
- missing_candidate_range
```

`unexpected_header_format:<actual_header_format>` の `<actual_header_format>` は次で正規化します。
Header.format value bytes が ASCII token `^[A-Za-z0-9._-]{1,64}$` として decode できる場合だけ、
その string をそのまま使います。
それ以外の場合は `<actual_header_format> = invalid_string` に固定します。
raw byte の hex dump、escaped string、truncated preview は deterministic diagnostics に入れてはいけません。
`missing_required_section:<section_kind>` は Phase 2 top-level section order で最初に欠落した
required section kind を使います。
candidate set が空の場合は `field = "mutation.candidates"`、
`expected_value = "non_empty_candidate_set:<mutation.kind>"`、
`actual_value = "empty"` にします。
`challenge_output_store_file_unreadable` では `field = "output.store_manifest_path"`、
`actual_value = "unreadable"` にします。
`challenge_output_store_json_invalid` では `field = "output.store_manifest_path"`、
`actual_value = "invalid_json"` にします。
`challenge_output_store_manifest_invalid` では
`field` に invalid store manifest field の JSON path、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、
`invalid_hash_format`、`invalid_path`、`null_not_allowed`、`order_violation`、
`duplicate_challenge_id`、`duplicate_manifest_path`、`duplicate_field` のいずれかを入れます。
`challenge_output_store_entry_manifest_unreadable` では
`field = "challenge_output_store.entries[].manifest_path"`、
`actual_value = "unreadable"` にします。
store entry `manifest_path` が schema-valid だが symlink escape で owning root 外へ解決される場合も
この reason code を使います。
`challenge_output_store_entry_manifest_invalid` では、referenced `ChallengeManifest` が
JSON parse または manifest-local schema / domain validation に失敗したことを表します。
JSON parse failure では `field = "challenge_output_store.entries[].manifest_path"`、
`expected_value = "ChallengeManifest JSON"`、`actual_value = "invalid_json"` にします。
schema / domain failure では `field` に
`challenge_output_store.entries[].manifest.<manifest-local JSON path>` を入れ、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_enum`、
`invalid_hash_format`、`invalid_name_format`、`invalid_path`、`null_not_allowed`、
`order_violation`、`duplicate_field` のいずれかを入れます。
`challenge_output_store_entry_manifest_hash_mismatch` では
`field = "challenge_output_store.entries[].manifest_hash"`、
`expected_hash` に store entry の `manifest_hash`、
`actual_hash` に参照先 `ChallengeManifest` file bytes hash を入れます。
`challenge_id_conflict` では `field = "challenge_id"`、
`actual_value` に重複した `challenge_id` を入れます。
`challenge_output_store_entry_conflict` では `field = "challenge_output_store.entries[]"`、
`expected_value` に追加予定 entry の canonical JSON string、
`actual_value` に衝突した既存 entry の canonical JSON string を入れます。
`challenge_manifest_output_path_conflict` では `field = "output.manifest_path"`、
`mutated_certificate_output_path_conflict` では `field = "output.mutated_certificate_path"` とし、
どちらも `expected_hash` に今回生成する file bytes hash、
`actual_hash` に既存 file bytes hash を入れます。
`mutated_certificate_write_failure` では `field = "output.mutated_certificate_path"`、
`challenge_manifest_write_failure` では `field = "output.manifest_path"`、
`challenge_output_store_write_failure` では `field = "output.store_manifest_path"` とし、
いずれも `actual_value = "write_failed"` にします。
複数の失敗条件が同時にある場合は、この一覧の順序で最初に該当した
`reason_code` を返します。
ただし schema-valid な既存 `ChallengeOutputStoreManifest.entries[]` の entry-scoped validation では、
entries を保存済み order、つまり `challenge_id` の bytewise lexicographic order で走査し、
最初に失敗した entry だけを報告します。
同じ entry 内では `manifest_path` unreadable / symlink escape、referenced `ChallengeManifest`
JSON / schema / domain invalid、`manifest_hash` mismatch の順で検査します。
後続 entry に別の entry-scoped failure があっても報告しません。
`actual_hash` / `actual_value` はこの最初に失敗した entry から取ります。

`outcome_hint` は oracle ではありません。
テスト判定に使うのは、変異後 certificate に対する checker result だけです。
名前も `expected_checker_status` ではなく `outcome_hint.status` に固定します。
MVP の `npa-check challenge generate` が作る mutation kind はすべて rejection-required corpus です。
`unexpected checker acceptance` とは、MVP mutation kind の replay で required checker の
いずれかが `MachineCheckResult.status = checked` を返すこと、または
replay 用 `NormalizedCheckResult.comparison.status = all_agree_checked` になることです。
この判定に `outcome_hint` は使いません。
MVP 一覧にない third-party challenge は CI pass condition の
`unexpected checker acceptance` 判定には使わず、informational replay として扱います。
この区別は `mutation.kind` だけで行います。
後述の MVP challenge 種別に含まれる `mutation.kind` は rejection-required、
それ以外の `mutation.kind` は informational です。
Phase 8 MVP の nightly coverage summary と `ReleaseAuditBundleManifest` に含める
`ChallengeOutputStoreManifest` は rejection-required challenge だけを coverage universe にします。
informational challenge manifest と informational `ChallengeReplayResult` は
Phase 8 MVP の `ReleaseAuditBundleManifest` には含めません。
informational artifact は bundle 外の diagnostic store、または将来の postmortem manifest で扱います。
informational challenge は `ChallengeCoverageSummary.total_challenges`、`replayed_challenges`、
`unexpected_acceptances`、nightly / release pass condition には含めません。
`ChallengeCoverageSummary.entries` が informational challenge を参照している場合、
coverage summary は invalid です。

challenge manifest 内の hash 名は次の意味です。

```text
file_hash:
  .npcert file bytes の sha256。

claimed_certificate_hash:
  raw claimed-hash extractor が `ModuleHashes.certificate_hash` value bytes から読んだ certificate_hash。
  certificate header / trailer や full `ModuleCertBytes` decode から読む値ではない。
  extractor は Header と file end までの raw top-level section sequence を検査し、
  unknown section kind、識別不能 byte、trailing garbage、duplicate / out-of-order /
  overlapping / out-of-file section を許可しない。
  extractor は `ModuleHashes` section payload を local decode し、`export_hash`、
  `axiom_report_hash`、`certificate_hash` の固定 field sequence が payload 末尾まで
  ちょうど読めることも要求する。
  `ModuleHashes` payload 内の missing / extra / out-of-order field、trailing payload byte、
  invalid hash encoding は extractor failure として扱う。
  base_certificate では required。
  mutated_certificate では manifest-local schema 上 optional。
  challenge generator は raw claimed-hash extractor が
  `ModuleHashes.certificate_hash` field byte range / value byte range を exactly one として識別できる場合は必ず書く。
  unsupported schema version でも extractor が成功する場合は omit しない。
  extractor が一意に識別できない場合だけ omit する。
  manifest-local validator はこの条件を再評価しない。

recomputed_certificate_hash:
  checker が canonical bytes から再計算した certificate_hash。
  manifest には書かず、MachineCheckResult 側にだけ記録する。
```

challenge replay 用の `MachineCheckRequest.certificate.expected_certificate_hash` は
`ChallengeManifest` だけから次の規則で作ります。
materialize / replay aggregate はこの field のために mutated certificate file を読まず、
raw claimed-hash extractor も再実行しません。
generator が manifest 作成時に raw claimed-hash extractor を実行し、
`mutated_certificate.claimed_certificate_hash` を書くか omit するかを決めます。

```text
ChallengeManifest.mutated_certificate.claimed_certificate_hash が存在する場合:
  expected_certificate_hash = mutated_certificate.claimed_certificate_hash

ChallengeManifest.mutated_certificate.claimed_certificate_hash が omit されている場合:
  expected_certificate_hash = base_certificate.claimed_certificate_hash
```

mutated certificate の claimed hash が manifest 上 absent の challenge で使う
`expected_certificate_hash` は request identity を安定させるための deterministic placeholder です。
この placeholder は replay 用 `NormalizedCheckResult.artifact.expected_certificate_hash` にも入ります。
つまり claimed-hash absent challenge の artifact identity には、mutated certificate の
recomputed hash ではなく base certificate の claimed hash が入ります。
実際の mutated file identity は `MachineCheckRequest.certificate.file_hash`、
challenge manifest の `mutated_certificate.file_hash`、
および challenge replay result の `mutated_file_hash` で追跡します。
checker が `certificate_decode_error` / `noncanonical_encoding` / `unsupported_schema_version` を返す場合、
runner は通常どおり certificate hash 照合を skip します。
もし claimed-hash absent の challenge が checker 側では decode されて canonical hash を再計算できた場合は、
通常の `certificate_hash` 照合または `certificate_hash_mismatch` invariant に従います。

challenge replay result は別 artifact として保存します。

```json
{
  "schema": "npa.phase8.challenge_replay_result.v1",
  "result_id": "chreplay_pch_001",
  "result_hash": "sha256:...",
  "challenge_id": "pch_001",
  "manifest_hash": "sha256:...",
  "mutated_file_hash": "sha256:...",
  "mutated_claimed_certificate_hash": "sha256:...",
  "checker_results": [
    {
      "result_id": "mchkres_challenge_ref_001",
      "result_hash": "sha256:...",
      "run_artifact_hash": "sha256:...",
      "checker_profile": "reference"
    }
  ],
  "missing_checker_profiles": [],
  "normalized_result_hash": "sha256:...",
  "policy_hash": "sha256:...",
  "artifact_hash": "sha256:...",
  "comparison_status": "all_agree_failed",
  "observed_error_kinds": [
    "axiom_report_mismatch",
    "certificate_hash_mismatch"
  ]
}
```

`mutated_claimed_certificate_hash` は `ChallengeManifest.mutated_certificate.claimed_certificate_hash`
が存在する場合だけ required で、その値を bytewise に copy します。
manifest 側で omit された場合は replay result 側も omit します。
challenge replay aggregate は mutated certificate file を読んで raw claimed-hash extractor を再実行してはいけません。
`manifest_hash` は保存された `ChallengeManifest` file bytes の sha256 です。
canonical object hash ではありません。
将来 challenge manifest 自体に canonical object hash が必要になった場合は、
別 field `manifest_object_hash` を追加します。
`result_hash` は `result_id`、`result_hash`、`checker_results[*].result_id` field を除いた
`ChallengeReplayResult` の canonical hash です。
`ChallengeReplayResult` は保存 artifact なので、release audit bundle では
`result_hash` と保存 file bytes の `file_hash` を検証します。
`ChallengeReplayResult.result_hash` は challenge replay summary の同一性であり、
checker verdict の代替ではありません。
checker の oracle は常に `checker_results[*].result_hash` で参照される `MachineCheckResult` です。

MVP の challenge replay store manifest：

```json
{
  "schema": "npa.phase8.challenge_replay_store_manifest.v1",
  "results": [
    {
      "challenge_id": "pch_001",
      "manifest_hash": "sha256:...",
      "result_hash": "sha256:...",
      "artifact_hash": "sha256:...",
      "path": "build/challenge-replays/pch_001.json",
      "file_hash": "sha256:..."
    }
  ]
}
```

`path` は workspace-relative path です。
`file_hash` は saved `ChallengeReplayResult` file bytes の SHA-256 です。
`challenge_id` は parsed `ChallengeReplayResult.challenge_id` と一致しなければなりません。
`manifest_hash` は referenced `ChallengeManifest` file bytes の SHA-256 であり、
parsed `ChallengeReplayResult.manifest_hash` と一致しなければなりません。
`result_hash` と `artifact_hash` は parsed `ChallengeReplayResult.result_hash` /
`artifact_hash` と一致しなければなりません。
entries は `result_hash` の bytewise lexicographic order で昇順に並べます。
`result_hash`、`path`、および `(challenge_id, manifest_hash)` は unique です。
同じ `(challenge_id, manifest_hash)` に複数の replay result を登録したい場合は、
coverage summary に渡す前に採用する replay result だけを含む filtered replay store manifest を作ります。
`checker_results[*].result_hash` は required です。
`checker_results[*].run_artifact_hash` は required で、exact saved
`MachineCheckResult` artifact identity として使います。
`checker_results[*].run_artifact_hash` は `ChallengeReplayResult.result_hash` の hash 対象に含めます。
`checker_results[*].result_id` は人間向け参照であり、監査時の同一性判定には使いません。
`checker_results` は `checker_profile` を unique key にします。
entries は `RunnerPolicy.required_checker_profiles` の順序、次に
`RunnerPolicy.optional_checker_profiles` の順序で並べます。
MVP では policy に含まれない `checker_profile` を `ChallengeReplayResult` に入れてはいけません。
`missing_checker_profiles` は required で、missing になった required profile だけを
`RunnerPolicy.required_checker_profiles` の順序で入れます。
missing optional profile は `missing_checker_profiles` に入れません。
`missing_checker_profiles` は `ChallengeReplayResult.result_hash` の hash 対象に含めます。
これにより、informational replay で `normalized_result_hash` と `comparison_status` を omit しても、
missing required result を replay summary 単体で表現できます。
`npa-check challenge replay` は `NormalizedCheckResult` を生成しません。
`normalized_result_hash` は challenge replay aggregate が `--normalized-store` / normalized result store
reference から対応する `NormalizedCheckResult` を解決できた場合だけ required です。
`normalized_result_hash` の omit は informational replay の artifact schema としては valid ですが、
nightly / release coverage に参照される replay result では Step 8 の release bundle validation で
`challenge_replay_result` class 5 source-key failure です。
この場合は `CommandError.reason_code = release_bundle_generation_failed`、
`field = "challenge_replay_result[<i>].artifact.normalized_result_hash"`、
`expected_value = "required_for_release_coverage"`、`actual_value = "missing"` とします。
`normalized_result_hash` が present だが wrong type、explicit null、invalid hash format の場合は
artifact schema / domain validation failure であり、release bundle Step 6 の `input_schema_invalid` です。
この場合は `expected_value = "sha256:<lower-hex>"` とし、
`actual_value` には `wrong_type`、`null_not_allowed`、または `invalid_hash_format` を入れます。
release bundle の Step 6 field は `challenge_replay_result[<i>].artifact.normalized_result_hash`、
challenge replay store entry validation field は `replay_store.results[].normalized_result_hash` です。
`ChallengeReplayResult` schema / domain validation では `normalized_result_hash` の shape validation を
`comparison_status` の conditional required / forbidden validation より先に行います。
`normalized_result_hash` が wrong type、explicit null、または invalid hash format の場合は、
上記の `normalized_result_hash` failure を返し、`comparison_status` の required / forbidden validation は
評価しません。
`comparison_status` の conditional rule でいう `normalized_result_hash` が存在する場合とは、
`normalized_result_hash` が valid `sha256:<lower-hex>` として読めた場合だけです。
`normalized_result_hash` が field として absent の場合だけ、下の forbidden `comparison_status` presence を評価します。
`comparison_status` は `normalized_result_hash` が存在する場合だけ required で、
`NormalizedCheckResult.comparison.status` と同じ closed enum を写します。
`normalized_result_hash` が存在する replay result で `comparison_status` が missing、wrong type、
explicit null、invalid enum の場合は `ChallengeReplayResult` schema / domain validation failure です。
この場合は `expected_value = "NormalizedCheckResult.comparison.status"` とし、
`actual_value` には `missing`、`wrong_type`、`null_not_allowed`、または `invalid_enum` を入れます。
release bundle の Step 6 では `input_schema_invalid` として扱い、
field は `challenge_replay_result[<i>].artifact.comparison_status` にします。
challenge replay store entry validation では既存の
`replay_store.results[].comparison_status` field shape を使います。
`normalized_result_hash` が omit された replay result では `comparison_status` も omit します。
この場合に `comparison_status` が存在すれば `ChallengeReplayResult` schema / domain validation failure です。
この forbidden presence では `expected_value = "absent_without_normalized_result_hash"`、
`actual_value = "present"` とします。
release bundle の Step 6 field は `challenge_replay_result[<i>].artifact.comparison_status`、
challenge replay store entry validation field は `replay_store.results[].comparison_status` です。
nightly / release coverage に使う replay result では `normalized_result_hash` と
`comparison_status` が required なので、pipeline は challenge replay 前に challenge result 用
`NormalizedCheckResult` を生成し、aggregate replay に normalized result store を渡さなければなりません。
対応する `NormalizedCheckResult` を解決できない場合、nightly / release 用 aggregate replay は
`ChallengeReplayResult` を保存せず challenge replay pipeline failure にします。
`observed_error_kinds` は checker results の `error.kind` を bytewise lexicographic order で
重複排除した配列です。
checker result がすべて `checked` の場合は `[]` にします。
`policy_hash` と `artifact_hash` は replay がどの policy / artifact identity で行われたかを
result 単体から検証するために required です。
`policy_hash` は replay に使った `RunnerPolicy` の canonical hash です。
`artifact_hash` は次の規則で決めます。

```text
normalized_result_hash が存在する場合:
  artifact_hash = referenced NormalizedCheckResult.artifact_hash

normalized_result_hash が omit された場合:
  RunnerPolicy.required_checker_profiles[0] の replay MachineCheckRequest から
  NormalizedCheckResult と同じ artifact object を構築し、その canonical hash を使う。
```

replay に含まれるすべての `checker_results[*]` は、machine result store から
`run_artifact_hash` で exact saved artifact を解決します。
解決した `MachineCheckResult.result_hash` と `MachineCheckResult.checker.profile` は
entry の `result_hash` と `checker_profile` に一致しなければなりません。
さらに各参照先 `MachineCheckResult.request_hash` から replay `MachineCheckRequest` を解決し、
それぞれ artifact hash を再計算します。
aggregate は
`RunnerPolicy.required_checker_profiles[0]` の replay `MachineCheckRequest` から
candidate replay artifact hash を計算します。
各 replay request から再計算した artifact hash は、この candidate replay artifact hash と
一致しなければなりません。
一致しない場合は `ChallengeReplayResult` を保存せず、challenge replay pipeline failure として扱います。
この計算は `normalized_result_hash` が omit された場合の `artifact_hash` 計算と同じです。
normalized result store が指定された場合、aggregate は
`artifact_hash = candidate replay artifact hash`、
`policy.hash = ChallengeReplayResult.policy_hash`、
`results[*].result_hash` の集合が `checker_results[*].result_hash` の集合と一致する
`NormalizedCheckResult` を探します。
一致する entry がちょうど1件ならその `normalized_result_hash` と `comparison.status` を写し、
`ChallengeReplayResult.artifact_hash` にはその `NormalizedCheckResult.artifact_hash` を入れます。
この `NormalizedCheckResult.artifact_hash` は candidate replay artifact hash と一致しなければなりません。
一致しない場合は `ChallengeReplayResult` を保存せず、challenge replay pipeline failure として扱います。
coverage-required mode は CLI の `--coverage-required`、または API request body の
`coverage_required = true` でだけ有効です。
0件または2件以上の場合、coverage-required mode では pipeline failure、
coverage-required でない mode では `normalized_result_hash` と `comparison_status` を omit します。
MVP では challenge replay pipeline failure 専用の保存 artifact は作りません。
CLI の `--json` では exit code 1、stdout empty、stderr に `CommandError` JSON を1個だけ出します。
API は wrapper validation 通過後の domain validation error body として
同じ `CommandError` object を返し、
`ChallengeReplayResult` body を返してはいけません。
この error body / transport error は release audit bundle の artifact kind には含めません。
MVP の challenge replay `CommandError.reason_code` は次に限定します。

```text
- challenge_manifest_file_unreadable
- challenge_manifest_hash_mismatch
- challenge_manifest_json_invalid
- challenge_manifest_schema_invalid
- policy_reference_invalid
- policy_file_unreadable
- policy_hash_mismatch
- input_reference_invalid
- request_store_manifest_hash_mismatch
- request_store_manifest_invalid
- request_store_entry_file_unreadable
- request_store_entry_json_invalid
- request_store_entry_schema_invalid
- request_store_entry_file_hash_mismatch
- request_store_entry_request_hash_mismatch
- result_store_manifest_hash_mismatch
- result_store_manifest_invalid
- result_store_entry_file_unreadable
- result_store_entry_json_invalid
- result_store_entry_schema_invalid
- result_store_entry_file_hash_mismatch
- result_store_entry_artifact_hash_mismatch
- result_store_entry_checker_profile_mismatch
- normalized_store_manifest_hash_mismatch
- normalized_store_manifest_invalid
- normalized_store_entry_file_unreadable
- normalized_store_entry_json_invalid
- normalized_store_entry_schema_invalid
- normalized_store_entry_file_hash_mismatch
- normalized_store_entry_artifact_hash_mismatch
- materialized_request_not_found
- materialized_request_hash_mismatch
- result_attempt_ambiguous
- replay_artifact_hash_mismatch
- normalized_result_not_found
- normalized_result_ambiguous
- normalized_result_artifact_hash_mismatch
- replay_store_manifest_invalid
- replay_store_entry_file_unreadable
- replay_store_entry_json_invalid
- replay_store_entry_schema_invalid
- replay_store_entry_file_hash_mismatch
- replay_store_entry_challenge_id_mismatch
- replay_store_entry_result_hash_mismatch
- replay_store_entry_manifest_hash_mismatch
- replay_store_entry_artifact_hash_mismatch
- replay_store_entry_conflict
- replay_output_path_conflict
- replay_output_write_failure
- replay_store_write_failure
```

challenge replay `CommandError` の field は固定します。
`challenge_manifest_file_unreadable` では `field = "challenge_manifest.path"`、
`actual_value = "unreadable"` にします。
`challenge_manifest_json_invalid` では `field = "challenge_manifest.path"`、
`actual_value = "invalid_json"` にします。
`challenge_manifest_schema_invalid` では `field` に invalid challenge manifest field の JSON path、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_enum`、
`invalid_hash_format`、`invalid_name_format`、`invalid_path`、`null_not_allowed`、`order_violation`、
`duplicate_field` のいずれかを入れます。
`policy_reference_invalid` では challenge 系 command 共通の policy reference field shape に従います。
`policy_file_unreadable` では `field = "policy.path"`、`actual_value = "unreadable"` にします。
`policy_hash_mismatch` では `field = "policy.hash"`、
`expected_hash` に caller 指定 hash、`actual_hash` に読み込んだ `RunnerPolicy` の canonical hash を入れます。
`ChallengeManifest.policy_hash` が `RunnerPolicyReference.hash` と一致しない場合は
同じ `policy_hash_mismatch` を使い、`field = "challenge_manifest.policy_hash"`、
`expected_hash` に `RunnerPolicyReference.hash`、
`actual_hash` に `ChallengeManifest.policy_hash` を入れます。
両方の mismatch が同時に成立する場合は、challenge 系 command 共通 rule に従い、
`RunnerPolicyReference.hash` と読み込んだ `RunnerPolicy` canonical hash の mismatch を先に報告し、
`field = "policy.hash"` を返します。
`input_reference_invalid` は CLI challenge replay の read-only input reference pair の片側指定、
path schema violation、または hash format violation に使います。
対象 pair は `--manifest` / `--manifest-hash`、`--request-store` / `--request-store-hash`、
`--result-store` / `--result-store-hash`、および active な
`--normalized-store` / `--normalized-store-hash` です。
field mapping は次で固定します。

```text
--manifest:
  path field = "challenge_manifest.path"
  hash field = "challenge_manifest.manifest_hash"
--request-store:
  path field = "request_store.path"
  hash field = "request_store.manifest_hash"
--result-store:
  path field = "result_store.path"
  hash field = "result_store.manifest_hash"
--normalized-store:
  path field = "normalized_store.path"
  hash field = "normalized_store.manifest_hash"
```

`--coverage-required` がない informational replay で normalized store pair が両方 omit された場合だけ valid です。
required pair が完全に欠けている場合、missing `--json`、duplicate singleton flag、
unsupported flag は CLI argument validation error であり、`CommandError` body を返しません。
片側指定では missing 側の field に `expected_value = "required"`、
`actual_value = "missing"` を入れます。
path schema violation では該当 `.path` field に
`expected_value = "workspace_relative_path"`、`actual_value = "invalid_path"` を入れます。
hash format violation では該当 `.manifest_hash` field に
`expected_value = "sha256:<lower-hex>"`、`actual_value = "invalid_hash_format"` を入れます。
複数の read-only input reference pair で `input_reference_invalid` が同時に成立する場合は、
`--manifest`、`--request-store`、`--result-store`、active な `--normalized-store` の順で報告します。
同じ pair 内では片側指定、path schema violation、hash format violation の順で最初の failure を返します。
同じ pair で path schema violation と hash format violation が同時に成立する場合は path schema violation を先に返します。
API `/machine/check/challenge/replay` では同じ malformed reference object / member は
wrapper schema validation または workspace path validation failure なので `ApiError` です。
API で `CommandError.reason_code = input_reference_invalid` は使いません。
API wrapper validation を通過した後に manifest / store file が unreadable、
hash mismatch、JSON invalid、または manifest schema / domain invalid だった場合だけ、
下の `*_manifest_*` reason code の `CommandError` を返します。
read-only input reference の manifest hash mismatch では
`challenge_manifest_hash_mismatch`、`request_store_manifest_hash_mismatch`、
`result_store_manifest_hash_mismatch`、または active な `normalized_store_manifest_hash_mismatch` を使い、
`field` に該当 reference の `*.manifest_hash` field path を入れ、
`expected_hash` に caller 指定 hash、`actual_hash` に manifest file bytes hash を入れます。
`--replay-store-out` / replay store output は caller supplied manifest hash を持たない write target なので、
`*_manifest_hash_mismatch` reason code は使いません。
`request_store_manifest_invalid`、`result_store_manifest_invalid`、
`normalized_store_manifest_invalid`、`replay_store_manifest_invalid` では、store manifest file を読めない場合は
`field` に該当 reference の `*.path` field path、`actual_value = "unreadable"` を入れます。
JSON として壊れている場合は同じ field で `actual_value = "invalid_json"` にします。
ただし `replay_store_manifest_invalid` は `--replay-store-out` / replay store output の既存 manifest に対する
write-stage validation error なので、manifest file unreadable / invalid JSON では
`field = "replay_store_output_path"` を使います。
schema / order / duplicate 違反では `field` に invalid store manifest field の JSON path、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_hash_format`、
`invalid_path`、`null_not_allowed`、`order_violation`、`duplicate_field`、
または manifest 種別ごとの unique key duplicate reason を入れます。
この field は caller-prefixed manifest path とします。
たとえば request store manifest の local `requests[<i>].path` は
`request_store.requests[<i>].path`、result store manifest の local `results[<i>].path` は
`result_store.results[<i>].path`、normalized store manifest の local `results[<i>].path` は
`normalized_store.results[<i>].path`、replay store manifest の local `results[<i>].path` は
`replay_store.results[<i>].path` として報告します。
manifest schema / domain error の field は concrete index を含む caller-prefixed path に固定します。
一方、entry file IO / JSON / artifact schema / hash validation error の field は下の dedicated reason code にある
`request_store.requests[]`、`result_store.results[]`、`normalized_store.results[]`、
`replay_store.results[]` の wildcard path に固定します。
store manifest entry `path` が workspace-relative path schema に違反する場合は
対応する `*_store_manifest_invalid` としてここで止め、entry file は読みに行きません。
`request_store_manifest_invalid` で許可する unique key duplicate reason は
`duplicate_request_hash` と `duplicate_path` だけです。
`result_store_manifest_invalid` で許可する unique key duplicate reason は
`duplicate_run_artifact_hash` と `duplicate_path` だけです。
`normalized_store_manifest_invalid` で許可する unique key duplicate reason は
`duplicate_normalized_result_hash` と `duplicate_path` だけです。
`replay_store_manifest_invalid` で許可する unique key duplicate reason は
`duplicate_result_hash`、`duplicate_path`、`duplicate_challenge_manifest` だけです。
他 manifest 種別の duplicate reason を使ってはいけません。
unique key duplicate reason の field は、重複 key の caller-prefixed manifest path に固定します。
`duplicate_request_hash` は `request_store.requests[<i>].request_hash`、
`duplicate_run_artifact_hash` は `result_store.results[<i>].run_artifact_hash`、
`duplicate_normalized_result_hash` は `normalized_store.results[<i>].normalized_result_hash`、
`duplicate_result_hash` は `replay_store.results[<i>].result_hash`、
`duplicate_path` は store 種別ごとの concrete manifest path に固定し、
`request_store.requests[<i>].path`、
`result_store.results[<i>].path`、
`normalized_store.results[<i>].path`、または
`replay_store.results[<i>].path` です。
`duplicate_challenge_manifest` は `(challenge_id, manifest_hash)` pair の duplicate であり、
`field = "replay_store.results[<i>].challenge_id"`、
`expected_value = "unique_challenge_id_manifest_hash_pair"`、
`actual_value = "duplicate_challenge_manifest"` とします。
store entry が参照する artifact file bytes や parsed artifact hash と一致しない場合は、
`*_store_manifest_invalid` ではなく次の dedicated reason code を使います。
`request_store_entry_file_unreadable`、`result_store_entry_file_unreadable`、
`normalized_store_entry_file_unreadable`、`replay_store_entry_file_unreadable` では
`field` に該当 entry の `path` field path、
`actual_value = "unreadable"` を入れます。
`request_store_entry_json_invalid`、`result_store_entry_json_invalid`、
`normalized_store_entry_json_invalid`、`replay_store_entry_json_invalid` では
`field` に該当 entry の `path` field path、
`actual_value = "invalid_json"` を入れます。
`request_store_entry_schema_invalid`、`result_store_entry_schema_invalid`、
`normalized_store_entry_schema_invalid`、`replay_store_entry_schema_invalid` では
`field` に invalid artifact field の JSON path、
`expected_value` に artifact schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_enum`、
`invalid_path`、`invalid_hash_format`、`invalid_name_format`、`null_not_allowed`、`order_violation`、
`duplicate_field` のいずれかを入れます。
この field は artifact-local JSON path に store root wildcard prefix を付けた
wildcard-prefixed artifact path とします。
たとえば request artifact local `module` は `request_store.requests[].module`、
machine result artifact local `checker.profile` は `result_store.results[].checker.profile`、
normalized result artifact local `comparison.status` は `normalized_store.results[].comparison.status`、
replay result artifact local `comparison_status` は `replay_store.results[].comparison_status` として報告します。
store entry artifact の top-level `schema` が期待値と一致しない場合も
対応する `*_store_entry_schema_invalid` です。
この場合は `field` に `request_store.requests[].schema`、
`result_store.results[].schema`、`normalized_store.results[].schema`、
または `replay_store.results[].schema` を入れ、
`expected_value` に期待する artifact schema string を入れます。
`actual_value` は `missing`、`null_not_allowed`、`wrong_type`、
または入力 artifact の `schema` 文字列です。
store entry artifact の top-level schema mismatch では
`actual_value = "wrong_schema"` を使いません。
`request_store_entry_file_hash_mismatch`、`result_store_entry_file_hash_mismatch`、
`normalized_store_entry_file_hash_mismatch`、`replay_store_entry_file_hash_mismatch` では
`field` に該当 entry の `file_hash` field path、
`expected_hash` に manifest entry の `file_hash`、
`actual_hash` に参照先 file bytes hash を入れます。
store entry artifact validation order は、store 種別にかかわらず次で固定します。

```text
1. entry file readable
2. entry JSON parse
3. entry artifact schema / top-level schema
4. entry file_hash vs referenced file bytes
5. parsed artifact self-hash
6. manifest entry fields vs parsed artifact fields
```

step 6 の manifest entry field comparison order は store 種別ごとに固定します。

```text
request_store_entry:
  - request_hash
result_store_entry:
  - result_hash
  - request_hash
  - run_artifact_hash
  - checker_profile
normalized_store_entry:
  - artifact_hash
  - normalized_result_hash
replay_store_entry:
  - challenge_id
  - manifest_hash
  - result_hash
  - artifact_hash
```

store entry artifact の self-hash は manifest entry との比較より先に再計算します。
複数の self-hash field がある artifact では、次の順序で検査します。

```text
self-hash validation order:
  request_store_entry:
    - request_hash
  result_store_entry:
    - result_hash
    - run_artifact_hash
  normalized_store_entry:
    - artifact_hash
    - normalized_result_hash
  replay_store_entry:
    - result_hash
```

self-hash mismatch の場合は、対応する `*_store_entry_*_hash_mismatch` reason を使い、
この順序で最初に見つかった mismatch field を `field` に入れます。
`expected_hash` に parsed artifact から再計算した hash、
`actual_hash` に parsed artifact 内の self-hash field を入れます。
self-hash が valid な場合だけ、manifest entry field と parsed artifact field を比較します。
hash field の mismatch では `expected_hash` に manifest entry の hash、
`actual_hash` に parsed artifact field の hash を入れます。
non-hash field の mismatch では `expected_value` に manifest entry の値、
`actual_value` に parsed artifact field の値を入れます。
`request_store_entry_request_hash_mismatch` では `field = "request_store.requests[].request_hash"`、
`expected_hash` に manifest entry の `request_hash`、
`actual_hash` に parsed `MachineCheckRequest.request_hash` を入れます。
`result_store_entry_artifact_hash_mismatch` では `field` に
`result_store.results[].result_hash`、`result_store.results[].request_hash`、
または `result_store.results[].run_artifact_hash` を入れ、
`expected_hash` に manifest entry の hash、`actual_hash` に parsed `MachineCheckResult` の同じ field を入れます。
`result_store_entry_checker_profile_mismatch` では `field = "result_store.results[].checker_profile"`、
`expected_value` に manifest entry の `checker_profile`、
`actual_value` に parsed `MachineCheckResult.checker.profile` を入れます。
`normalized_store_entry_artifact_hash_mismatch` では `field` に
`normalized_store.results[].artifact_hash` または `normalized_store.results[].normalized_result_hash` を入れ、
`expected_hash` に manifest entry の hash、`actual_hash` に parsed `NormalizedCheckResult` の同じ field を入れます。
`materialized_request_not_found` では `field = "request_store.requests[].request_hash"`、
`expected_hash` に再構成した replay request hash を入れます。
`materialized_request_hash_mismatch` では `field = "request_store.requests[].request_hash"`、
`expected_hash` に再構成した replay request hash、
`actual_hash` に materialized request self hash を入れます。
`result_attempt_ambiguous` では `field = "result_store.results[]"`、
`expected_value = "exactly_one_result_per_request_hash_and_checker_profile"`、
`actual_value = "multiple_results"` にします。
`replay_artifact_hash_mismatch` では `field = "artifact_hash"`、
`expected_hash` に candidate replay artifact hash、`actual_hash` に mismatch した replay request artifact hash を入れます。
`normalized_result_not_found` では `field = "normalized_store.results[]"`、
`expected_hash` に candidate replay artifact hash を入れます。
`normalized_result_ambiguous` では `field = "normalized_store.results[]"`、
`expected_value = "exactly_one_normalized_result"`、`actual_value = "multiple_results"` にします。
`normalized_result_artifact_hash_mismatch` では `field = "normalized_store.results[].artifact_hash"`、
`expected_hash` に candidate replay artifact hash、
`actual_hash` に解決した `NormalizedCheckResult.artifact_hash` を入れます。
`replay_store_entry_challenge_id_mismatch` では `field = "replay_store.results[].challenge_id"`、
`expected_value` に manifest entry の `challenge_id`、
`actual_value` に parsed `ChallengeReplayResult.challenge_id` を入れます。
`replay_store_entry_result_hash_mismatch` では `field = "replay_store.results[].result_hash"`、
manifest entry comparison では `expected_hash` に manifest entry の `result_hash`、
`actual_hash` に parsed `ChallengeReplayResult.result_hash` を入れます。
parsed `ChallengeReplayResult.result_hash` self-hash mismatch でも同じ reason code と field を使いますが、
その場合は `expected_hash` に parsed artifact から再計算した hash、
`actual_hash` に parsed `ChallengeReplayResult.result_hash` を入れます。
`replay_store_entry_manifest_hash_mismatch` では `field = "replay_store.results[].manifest_hash"`、
`expected_hash` に manifest entry の `manifest_hash`、
`actual_hash` に parsed `ChallengeReplayResult.manifest_hash` を入れます。
`replay_store_entry_artifact_hash_mismatch` では `field = "replay_store.results[].artifact_hash"`、
`expected_hash` に manifest entry の `artifact_hash`、
`actual_hash` に parsed `ChallengeReplayResult.artifact_hash` を入れます。
`replay_store_entry_conflict` では `field = "replay_store.results[]"`、
`expected_value` に追加予定 entry の canonical JSON string、
`actual_value` に衝突した既存 entry の canonical JSON string を入れます。
`replay_output_path_conflict` では `field = "replay_output_path"`、
`expected_hash` に今回書く replay result file hash、
`actual_hash` に既存 file bytes hash を入れます。
`replay_output_write_failure` では `field = "replay_output_path"`、
`replay_store_write_failure` では `field = "replay_store_output_path"` とし、
どちらも `actual_value = "write_failed"` にします。
その他の schema / manifest invalid では、該当する invalid field の JSON path を `field` に入れます。
複数の store entry artifact failure が同時にある場合は、まず
`request_store`、`result_store`、active な `normalized_store`、`replay_store` の順で store kind を選び、
同じ store manifest 内では entry array の小さい index を選びます。
同じ entry 内で複数 failure が成立する場合だけ、store entry artifact validation order を使います。
複数の write-stage failure が同時にある場合は、
`replay_store_manifest_invalid`、store entry artifact validation failure、
`replay_output_path_conflict`、`replay_store_entry_conflict`、
`replay_output_write_failure`、`replay_store_write_failure` の順で最初に該当した
`reason_code` を返します。

challenge manifest は checker input ではありません。
checker input は変異後の `.npcert` だけです。

MVP で作る challenge 種別：

```text
- flip_canonical_encoding_byte
- reorder_declarations
- replace_import_export_hash
- remove_dependency_entry
- change_declaration_body_without_hash
- change_declaration_hash_without_body
- drop_axiom_report_entry
- add_forbidden_axiom
- alter_de_bruijn_index
- replace_nat_zero_with_noncanonical_placeholder
- insert_unsupported_schema_version
- truncate_certificate_section
```

CLI の `npa-check challenge generate --kind` は上記の MVP challenge 種別の closed enum と
同じ文字列だけを受け取ります。
`ChallengeManifest.mutation.kind` schema 自体は grammar-valid informational kind を許しますが、
MVP generator は informational kind を生成しません。
Phase 2 `NPA-CERT-0.1` の serialized `DeclPayload` は declaration-local universe constraint field を
持たないため、`alter_universe_constraint` は MVP generator closed enum には含めません。
将来の certificate schema が encoded universe constraint field を追加する場合は、
その schema version と同時に `alter_universe_constraint` を新しい rejection-required kind として追加します。
`--target` は `ChallengeGenerationRequest.mutation.target`、
`--seed` は `ChallengeGenerationRequest.mutation.seed`、
`--challenge-id` は `ChallengeGenerationRequest.challenge_id`、
`--module` は `ChallengeGenerationRequest.module`、
`--imports` は `ChallengeGenerationRequest.imports.manifest`、
`--imports-hash` は `ChallengeGenerationRequest.imports.manifest_hash`、
`--from` は `ChallengeGenerationRequest.base_certificate.path`、
`--challenge-store` は `ChallengeGenerationRequest.output.store_manifest_path`、
`--manifest-out` と `--mutated-out` は `ChallengeGenerationRequest.output` に対応します。
`ChallengeGenerationRequest.imports.mode` は MVP では `locked_store` に固定し、
`RunnerPolicy.import_policy.mode` と一致しなければなりません。
CLI は `--from` の file bytes sha256 を `base_certificate.file_hash` として計算し、
base certificate から claimed certificate hash を decode して
`base_certificate.claimed_certificate_hash` に入れます。
この CLI request construction phase は `ChallengeGenerationRequest.request_hash` の
validation 前に行われますが、output artifact を作成・更新してはいけません。
base certificate の claimed hash を decode できない場合は generation validation failure です。
`--generated-by` は `ChallengeGenerationRequest.generated_by.kind` に対応し、
`ci` または `ai` のどちらかを required にします。
`--generated-by ai` では `--prompt-hash` が required、
`--generated-by ci` では `--prompt-hash` は forbidden です。
manifest の `mutation.kind`、`mutation.target`、`mutation.seed` は
`ChallengeGenerationRequest.mutation` と一致しなければなりません。
上記の mutation kind だけが Phase 8 MVP の rejection-required coverage 対象です。
上記以外の `mutation.kind` を持つ challenge manifest は informational としてだけ扱います。

AI が生成した challenge は、outcome_hint も含めて信用しません。
最終的なテスト oracle は checker result です。

---

# 11. Challenge minimization

checker failure が複雑な場合、AI は再現最小化を提案できます。
ただし最小化済み artifact も checker で再検査します。

minimization の方針：

```text
- failed declaration を含む最小 module slice を作る
- import set を policy の範囲で減らす
- source ではなく certificate declaration graph を基準に削る
- pretty output ではなく declaration hash / dependency hash で同一性を見る
- minimized certificate も noncanonical ならそのまま rejection を期待する
```

AI が出せるもの：

```text
- slice candidate
- suspected declaration list
- dependency path explanation
- next command suggestion
```

AI が出せないもの：

```text
- minimized artifact の受理判定
- checker failure の無効化
- missing dependency の自動補完による checked 扱い
```

---

# 12. CI integration

AI は CI で次の補助をします。

```text
- failed checker result の短い要約
- declaration 単位の failure grouping
- repeated failure の dedupe
- likely owner / phase の推定
- next debugging command の提案
- challenge coverage の不足指摘
```

CI status を決めるのは deterministic pipeline です。
pass condition は mode ごとに分けます。

release / nightly / high-trust の pass condition と audit bundle generation は、
`RunnerPolicy` とは別の `ReleasePolicy` で束ねます。
`RunnerPolicy` は checker 実行単位の policy であり、
`ReleasePolicy` はどの `RunnerPolicy`、challenge policy、AI sidecar requirement、
release audit requirement を組み合わせるかを決める deterministic config です。

MVP の `ReleasePolicy` schema：

```json
{
  "schema": "npa.phase8.release_policy.v1",
  "id": "phase8-release",
  "version": 1,
  "mode": "release",
  "runner_policy_hash": "sha256:...",
  "challenge_runner_policy_hash": "sha256:...",
  "ai_triage": {
    "enabled": true,
    "required": true,
    "input_policy_hash": "sha256:..."
  }
}
```

`mode` は `nightly`、`release`、`high-trust` のいずれかです。
PR mode は `ReleasePolicy` を使いません。
MVP の `ReleasePolicy` top-level required field は `schema`、`id`、`version`、`mode`、
`runner_policy_hash`、`challenge_runner_policy_hash`、`ai_triage` です。
`id` は non-empty string、`version` は JSON integer で
`1 <= version <= 9223372036854775807`、hash field は `sha256:<lower-hex>` です。
zero は「未設定」や「latest」を表しません。
`ReleasePolicy` と `ai_triage` object は closed-world object で、unknown field と
duplicate key を禁止します。
ReleasePolicy schema failure の field shape は次で固定します。
この節の `field` は ReleasePolicy artifact root からの JSON path です。
ReleasePolicy file/reference validation の `CommandError.field` として返す場合は、
`field = "$"` を `release_policy`、それ以外を `release_policy.<field>` に変換します。
他 artifact から ReleasePolicy を参照して diagnostic field を出す場合も同じ prefix rule を使います。

```text
top-level schema missing / null / wrong type / mismatch:
  field = "schema"
  expected_value = "npa.phase8.release_policy.v1"
  actual_value = missing | null_not_allowed | wrong_type | invalid_enum

top-level JSON value is not object:
  field = "$"
  expected_value = "object"
  actual_value = wrong_type | null_not_allowed

generic field schema failure:
  field = <ReleasePolicy JSON path>
  expected_value = <schema requirement name>
  actual_value = missing | wrong_type | unknown_field | invalid_enum |
                 invalid_hash_format | null_not_allowed | order_violation |
                 duplicate_field
```

ReleasePolicy domain failure の field shape は次で固定します。

```text
id empty:
  field = "id"
  expected_value = "non_empty_string"
  actual_value = "empty_string"

version domain violation:
  field = "version"
  expected_value = "positive_i64"
  actual_value = "non_positive_integer" | "integer_out_of_range"

ai_triage.input_policy_hash missing when enabled:
  field = "ai_triage.input_policy_hash"
  expected_value = "sha256:<lower-hex>"
  actual_value = "missing"

ai_triage.input_policy_hash present when disabled:
  field = "ai_triage.input_policy_hash"
  expected_value = "absent"
  actual_value = "present"

ai_triage.required true when disabled:
  field = "ai_triage.required"
  expected_value = "false_when_ai_triage_disabled"
  actual_value = "true"
```

`ai_triage.enabled` と `ai_triage.required` の missing / null / wrong type は generic field schema failure として
`field = "ai_triage.enabled"` または `field = "ai_triage.required"`、
`expected_value = "boolean"` で報告します。
`ai_triage.input_policy_hash` の wrong type / null / invalid hash format も generic field schema failure として
`expected_value = "sha256:<lower-hex>"` で報告します。
PR mode の optional AI sidecar は `ReleasePolicy.ai_triage.input_policy_hash` を参照できないため、
cross-artifact validation では `npa-check audit-sidecar validate` に明示された
`AiAuditInputPolicy` file / hash を唯一の input policy source とします。
PR mode に implicit default input policy lookup はありません。
PR mode の optional AI sidecar を CI diagnostic artifact として保存する場合、
`AiAuditSidecar.input_policy.hash` と `AuditSidecarValidationResult.input_policy_hash` は
その明示 input policy hash と一致し、validation status は `valid` でなければなりません。
input policy を与えない場合に実行できるのは schema-only validation だけであり、
その sidecar は cross-artifact validated PR diagnostic artifact ではありません。
PR pass condition は AI sidecar の有無、schema-only validation 結果、
cross-artifact validation 結果のいずれにも依存しません。
`runner_policy_hash` は通常の certificate check / normalization に使う `RunnerPolicy` の canonical hash です。
`challenge_runner_policy_hash` は challenge replay に使う `RunnerPolicy` の canonical hash です。
challenge replay を実行しない将来 mode 以外では required で、MVP の `nightly` / `release` /
`high-trust` では常に required です。
`ReleasePolicy.mode` と、解決した `RunnerPolicy.trust_mode` の対応は次で固定します。

```text
ReleasePolicy.mode = nightly:
  runner_policy_hash resolves to RunnerPolicy.trust_mode = nightly
  challenge_runner_policy_hash resolves to RunnerPolicy.trust_mode = nightly

ReleasePolicy.mode = release:
  runner_policy_hash resolves to RunnerPolicy.trust_mode = release
  challenge_runner_policy_hash resolves to RunnerPolicy.trust_mode = release

ReleasePolicy.mode = high-trust:
  runner_policy_hash resolves to RunnerPolicy.trust_mode = high-trust
  challenge_runner_policy_hash resolves to RunnerPolicy.trust_mode = high-trust
```

この対応が崩れる `ReleasePolicy` は invalid です。
`ReleasePolicy` schema / domain validation failure は deterministic config failure であり、
CI は release / nightly / high-trust pass 判定と release audit bundle generation を開始してはいけません。
mode / trust mismatch は、`ReleasePolicy` を resolver 付きで検証する段階と
release audit bundle validator の両方で同じ field shape を使います。

```text
runner_policy_hash trust_mode mismatch:
  field = "runner_policy_hash"
  expected_value = "RunnerPolicy.trust_mode:<ReleasePolicy.mode>"
  actual_value = "RunnerPolicy.trust_mode:<resolved runner trust_mode>"

challenge_runner_policy_hash trust_mode mismatch:
  field = "challenge_runner_policy_hash"
  expected_value = "RunnerPolicy.trust_mode:<ReleasePolicy.mode>"
  actual_value = "RunnerPolicy.trust_mode:<resolved challenge runner trust_mode>"
```

ReleasePolicy validation は schema failure、local domain failure、resolver 付き trust_mode mismatch の順で報告します。
resolver 付き trust_mode mismatch は、ReleasePolicy artifact 単体の schema / local domain validation が
通った後でだけ評価します。
複数の schema failure が同時に存在する場合は、top-level non-object、`schema`、
`id`、`version`、`mode`、`runner_policy_hash`、`challenge_runner_policy_hash`、
`ai_triage`、`ai_triage.enabled`、`ai_triage.required`、`ai_triage.input_policy_hash`、
その後 unknown field の順で最初の1件だけを返します。
known field の duplicate object key は、その field の schema order 位置で
`expected_value = "unique_object_keys"`、`actual_value = "duplicate_field"` として報告します。
unknown field の duplicate object key は unknown field の位置で
`expected_value = "unique_object_keys"`、`actual_value = "duplicate_field"` を返し、
`field` は重複した後続 unknown field の JSON path にします。
unknown field が複数ある場合は、top-level object、`ai_triage` object の順で object を選び、
同じ object 内では field name の bytewise lexicographic order で最初の field を返します。
複数の local domain failure が同時に存在する場合は、上の domain failure table の順で
最初の1件だけを返します。
両方が不一致の場合は `runner_policy_hash`、次に `challenge_runner_policy_hash` の順で
最初の1件を報告します。
`high-trust pass conditions` が `release pass conditions` を含む場合、
`ReleasePolicy.mode = release` の policy を再利用するという意味ではありません。
同じ predicate 群を `ReleasePolicy.mode = high-trust` と
`trust_mode = high-trust` の runner / challenge policy で評価するという意味です。
`ai_triage.enabled` と `ai_triage.required` は required で、default value はありません。
`ai_triage.input_policy_hash` は `ai_triage.enabled = true` の場合 required、
`ai_triage.enabled = false` の場合 forbidden です。
`ai_triage.input_policy_hash` は `AiAuditInputPolicy` file の canonical hash であり、
AI sidecar の `input_policy.hash` と audit-sidecar validation reference の
`input_policy.hash` はこの値と一致しなければなりません。
`ai_triage.enabled = false` の場合、`ai_triage.required` は false でなければなりません。
`ai_triage.enabled = true` かつ `ai_triage.required = true` の場合、
該当 mode の failed / non-success target には required AI sidecar diagnostic と
cross-artifact validation response を保存しなければなりません。
これは失敗 target の診断完全性条件であり、checker failure を pass に変える条件ではありません。
`ai_triage.required = true` は「常に AI sidecar を生成する」という意味ではありません。
required AI sidecar diagnostic target が0件の場合、required AI sidecar artifact も0件です。
required diagnostic target が1件以上あるのに対応する sidecar / validation response がない場合、
AI sidecar diagnostic step は pass しませんが、その target の checker / comparison status は変えません。
materialized `ReleaseAuditBundleManifest` では required AI sidecar target set は空でなければなりません。
`ai_triage.enabled = true` かつ `required = false` の場合、AI sidecar は保存してよい optional artifact ですが、
pass condition には含めません。
ただし release audit bundle に `ai_audit_sidecar` entry として含める場合は、
optional source であっても対応する `AuditSidecarValidationResult.status = valid` が必要です。
invalid sidecar や validation response のない sidecar を release audit bundle に含めてはいけません。
optional sidecar の validation も `ReleasePolicy.ai_triage.input_policy_hash` と
included `ai_audit_input_policy` file を使い、bundle 内の store manifest から再実行できなければなりません。
`ai_triage.enabled = false` の場合、release audit bundle の `ai_audit_sidecar` と
`audit_sidecar_validation_response` entry は forbidden です。

MVP の required AI sidecar target は、release audit bundle 生成前の CI diagnostic 用に次で定義します。
これは失敗 target を説明するための規則であり、`ReleaseAuditBundleManifest` に含める
required artifact の定義ではありません。

```text
machine_result target:
  CI diagnostic target NormalizedCheckResult.results に含まれる
  MachineCheckResult のうち、status = failed のものすべて。

normalized_comparison target:
  CI diagnostic target NormalizedCheckResult のうち、comparison.status が
  disagreement / missing_checker_result / policy_failure / inconclusive のもの。
```

CI diagnostic target の `NormalizedCheckResult` とは、同じ `ReleasePolicy.runner_policy_hash` で
評価され、release / nightly / high-trust pass 判定対象になる would-be release target です。
release audit bundle 生成前は、この would-be target の `artifact_hash` が将来の
`ReleaseAuditBundleManifest.artifact_hash` になります。
release audit bundle validator では、top-level `artifact_hash` と同じ `artifact_hash` を持つ
唯一の `NormalizedCheckResult` が release target です。
required AI sidecar target resolution は `NormalizedCheckResult` が存在する場合だけ実行します。
normalization が失敗し、`MachineCheckRequestErrorResult`、`NormalizeErrorResult`、または
`CommandError` だけが存在する場合、Phase 8 MVP では required AI sidecar target を作りません。
その場合の required AI sidecar diagnostic target set は空です。
pipeline error artifact を AI sidecar source にすることも forbidden です。
その失敗 provenance は deterministic error artifact、diagnostic store、または将来の
postmortem manifest で扱います。
該当する `NormalizedCheckResult` が複数件ある場合は、CI diagnostic generation failure、
または bundle invalid です。
MVP の `ReleaseAuditBundleManifest` は release / high-trust の pass artifact であり、
release target は PR pass conditions、つまり required checker profiles checked かつ
`NormalizedCheckResult.comparison.status = all_agree_checked` を満たしていなければなりません。
そのため materialized `ReleaseAuditBundleManifest` 内では required AI sidecar target set は
必ず空です。
release target に failed `MachineCheckResult`、non-success comparison、または
required AI sidecar target が1件でも存在する場合、release audit bundle generation failure とし、
すでに作られた manifest を検証する場合は bundle invalid です。
そのような失敗 target の AI sidecar / validation response は CI diagnostic として bundle 外に保存するか、
将来の postmortem manifest で扱います。
challenge replay 用の `MachineCheckResult`、`ChallengeReplayResult`、および
challenge replay 用 `NormalizedCheckResult` は required AI sidecar target ではありません。
`comparison.status = all_agree_checked` と `all_agree_failed` は
required `normalized_comparison` sidecar target ではありません。
diagnostic target では `all_agree_failed` を構成する個々の failed `MachineCheckResult` も
machine_result target になりますが、この diagnostic rule は release audit bundle には適用しません。
release audit bundle に含めてよい AI sidecar source は optional source だけの closed set です。

```text
optional machine_result sidecar source:
  release target NormalizedCheckResult.results[*] から bundle validator が選択した raw result。
  または required reproducibility AuxiliaryResult.selector.repeated_run_artifact_hash が指す
  baseline profile の repeated raw result。

optional normalized_comparison sidecar source:
  release target NormalizedCheckResult の normalized_result_hash。
```

required AI sidecar source を `ReleaseAuditBundleManifest` に含めてはいけません。
`source.kind = machine_result` では、`source.run_artifact_hash` が bundle validator の選択 raw result、
または reproducibility repeated raw result の `run_artifact_hash` と完全一致しなければなりません。
`source.result_hash` と `source.request_hash` も同じ raw result と一致しなければなりません。
選択 raw result とは、normalizer input に採用された `MachineCheckResult` file です。
normalizer input は同じ `checker_profile` を重複させないため、release target の
`results[*]` entry は `checker_profile`、`result_hash`、`request_hash` の組で
bundle 内の raw result に一意に解決できなければなりません。
解決結果が0件または複数件の場合、release audit bundle generation failure、
または bundle invalid です。
CI diagnostic required `machine_result` sidecar source でも同じ raw result selection rule を使います。
ただし bundle-local store がまだ materialize されていない場合、normalizer に渡した
machine result store / selected input set から同じ `(checker_profile, result_hash, request_hash)` で
一意に解決します。
`source.normalized_result_hash` が存在する場合は、source は normalizer input に採用された
raw result でなければならず、reproducibility repeated raw result では forbidden です。
その値は release target `NormalizedCheckResult.normalized_result_hash` と一致し、
かつその `results[*].result_hash` に `source.result_hash` が含まれていなければなりません。
reproducibility repeated raw result を AI sidecar source にする場合、
`source.normalized_result_hash` は omit しなければなりません。
challenge replay 用 result、informational replay result、pipeline error provenance、
release target に採用されなかった retry result、または bundle の target scope 外 artifact を
AI sidecar source にする `ai_audit_sidecar` entry は release audit bundle では forbidden です。
optional sidecar は bundle に含める artifact の説明を追加するだけであり、
`machine_check_result`、`machine_check_request`、`import_lock`、`normalized_check_result`、
`challenge_replay_result` の allowed set を広げてはいけません。
CI diagnostic として required AI sidecar target が1件以上ある場合、各 target について
`AiAuditSidecar.source` がその target を参照し、対応する
`AuditSidecarValidationResult.status = valid` が保存されていなければ
AI sidecar diagnostic step は pass しません。
required AI sidecar diagnostic の `input_policy.hash` は
`ReleasePolicy.ai_triage.input_policy_hash` と一致しなければなりません。
対応する `AuditSidecarValidationResult.input_policy_hash` も同じ値でなければなりません。
ただし diagnostic step が pass しても、その target は release audit bundle には materialize しません。
release bundle validator は optional sidecar についても保存済み validation response だけを信用せず、
bundle 内の sidecar、store manifest、result file、normalized result file、
`ai_audit_input_policy` file から cross-artifact validation を再実行できなければなりません。

```text
pr pass conditions:
  - required checker profiles all returned checked
  - normalized comparison is all_agree_checked
  - axiom policy passed

nightly pass conditions:
  - pr pass conditions
  - nightly required checker profiles all returned checked
  - reproducibility check passed
  - rejection-required challenge replay coverage is complete against the explicit ChallengeOutputStoreManifest
  - every rejection-required replay comparison is all_agree_failed
  - no unexpected checker acceptance was observed

release pass conditions:
  - nightly pass conditions
  - release audit bundle was generated
  - any optional AI sidecar included in the release audit bundle has valid
    cross-artifact validation

high-trust pass conditions:
  - release pass conditions
  - high-trust-reference checker returned checked
  - import certificate_hash verification passed

AI sidecar conditions:
  - optional for PR mode
  - diagnostic-required only for failed / non-success targets when
    ReleasePolicy.ai_triage.enabled = true and ai_triage.required = true
  - not required when the diagnostic target set is empty
  - never sufficient for pass
```

ある mode の pass conditions に出てこない auxiliary result は、その mode の pass には要求しません。
CI pass condition に使う required `AuxiliaryResult` の解決は
`kind`、`policy_hash`、`artifact_hash`、および kind-specific `selector` key で一意に行います。
`selector` が forbidden の kind では、selector key は empty です。
required result が0件または複数件の場合、その mode の pass failure です。
required result は `status = passed` でなければならず、
`failed` / `inconclusive` は CI diagnostic として保存してよいだけで pass には使いません。
MVP の required auxiliary result は次で固定します。

```text
pr:
  - exactly one axiom_policy result for the target artifact

nightly:
  - pr requirements
  - exactly one reproducibility result for the target artifact

release:
  - nightly requirements

high-trust:
  - release requirements
  - exactly one import_certificate_hash result for each distinct import lock hash
    in the high-trust evaluation import set
```

bundle 外の high-trust pass 判定では、high-trust evaluation import set は
current target の `MachineCheckRequest.imports.manifest_hash` と
`NormalizedCheckResult.artifact.import_lock_hash` から作る distinct hash set です。
release audit bundle validation では、`ReleaseAuditBundleManifest` の closed-set rule で定義する
included artifact 全体の import lock hash 集合を使います。
required `axiom_policy` / `reproducibility` result の `policy_hash` は active `RunnerPolicy` の
canonical hash です。
required `import_certificate_hash` result の `policy_hash` は active `ReleasePolicy` の canonical hash です。

CI pass condition に使う補助 result は、MVP では同じ deterministic envelope を使います。

```json
{
  "schema": "npa.phase8.auxiliary_result.v1",
  "kind": "axiom_policy",
  "result_id": "aux_axiom_Std.Nat_001",
  "result_hash": "sha256:...",
  "policy_hash": "sha256:...",
  "artifact_hash": "sha256:...",
  "selector": {
    "normalized_result_hash": "sha256:...",
    "checker_profile": "reference",
    "result_hash": "sha256:...",
    "axiom_report_hash": "sha256:..."
  },
  "status": "passed"
}
```

MVP の `kind`：

```text
- axiom_policy
- reproducibility
- import_certificate_hash
- audit_bundle
```

MVP の `AuxiliaryResult` required field：

```text
- schema
- kind
- result_id
- result_hash
- policy_hash
- artifact_hash
- status
```

`policy_hash` はその auxiliary check を支配する `RunnerPolicy` または release policy の
canonical hash です。
`selector` は kind-specific input selector です。
`selector` は `kind = axiom_policy` と `kind = reproducibility` では required、
`kind = import_certificate_hash` と `kind = audit_bundle` では forbidden です。
`selector` は `result_hash` の hash 対象に含めます。
`artifact_hash` は kind ごとに次を入れます。

```text
axiom_policy:
  NormalizedCheckResult.artifact_hash

reproducibility:
  reproducibility 対象 artifact の artifact_hash。
  release / high-trust bundle では release target NormalizedCheckResult.artifact_hash。

import_certificate_hash:
  import verification 対象の import lock artifact hash。
  ReleaseAuditBundleManifest 内では import_lock entry の hashes.manifest_hash。

audit_bundle:
  validation 対象の ReleaseAuditBundleManifest.bundle_hash。
  これはこの AuxiliaryResult 自身を含む bundle ではなく、すでに materialize 済みの
  target bundle の bundle_hash でなければならない。
```

MVP の `AuxiliaryResult.selector` schema：

```text
kind = axiom_policy:
  required:
    - normalized_result_hash
    - checker_profile
    - result_hash
    - axiom_report_hash
  meaning:
    normalized_result_hash identifies the target NormalizedCheckResult.
    checker_profile identifies the MachineCheckResult entry whose axiom report is evaluated.
    result_hash and axiom_report_hash must match that normalized result entry.

kind = reproducibility:
  required:
    - request_hash
    - checker_profile
    - baseline_run_artifact_hash
    - repeated_run_artifact_hash
  meaning:
    request_hash and checker_profile identify the replayed execution class.
    baseline_run_artifact_hash and repeated_run_artifact_hash identify two distinct saved
    MachineCheckResult artifacts for that request/profile pair.
    The two run_artifact_hash values must be different.

kind = import_certificate_hash:
  selector forbidden. artifact_hash is the import lock manifest hash.

kind = audit_bundle:
  selector forbidden. artifact_hash is the validated bundle_hash.
```

MVP の `AxiomReportStoreManifest` schema：

```json
{
  "schema": "npa.phase8.axiom_report_store_manifest.v1",
  "reports": [
    {
      "axiom_report_hash": "sha256:...",
      "path": "build/axiom-reports/Std.Nat.json",
      "file_hash": "sha256:..."
    }
  ]
}
```

`AxiomReportStoreManifest` file の `manifest_hash` は manifest file bytes の SHA-256 です。
top-level required field は `schema`、`reports` です。
各 `reports[]` entry の required field は `axiom_report_hash`、`path`、`file_hash` です。
`path` は workspace-relative path、`file_hash` は referenced `AxiomReport` file bytes の SHA-256、
`axiom_report_hash` は parsed `AxiomReport.axiom_report_hash` です。
`reports` は `axiom_report_hash` の UTF-8 bytewise lexicographic order で昇順に並べます。
`axiom_report_hash` と `path` はそれぞれ unique です。
`schema` が `npa.phase8.axiom_report_store_manifest.v1` 以外の string の場合は
`actual_value = "invalid_enum"` の schema failure とします。
`AxiomReportStoreManifest` loader は duplicate key、unknown field、wrong type、
explicit null、invalid_enum、invalid hash format、invalid path、order violation、
duplicate axiom_report_hash、duplicate path を schema / domain failure として拒否します。
duplicate axiom_report_hash の `actual_value` は `duplicate_axiom_report_hash`、
duplicate path の `actual_value` は `duplicate_path` です。
AxiomReportStoreManifest validation は schema failure を domain failure より先に報告します。
複数の schema failure が同時に存在する場合は、top-level non-object、`schema`、
`reports` array、`reports[]` entry object by smaller index、
`reports[].axiom_report_hash` by smaller index、`reports[].path` by smaller index、
`reports[].file_hash` by smaller index、その後 unknown field の順で最初の1件だけを返します。
known field の duplicate object key は、その field の schema order 位置で
`expected_value = "unique_object_keys"`、`actual_value = "duplicate_field"` として報告します。
unknown field の duplicate object key は unknown field の位置で
`expected_value = "unique_object_keys"`、`actual_value = "duplicate_field"` を返し、
`field` は重複した後続 unknown field の manifest-local JSON path にします。
unknown field が複数ある場合は top-level object、次に `reports[]` entry の小さい index の順で
object を選び、同じ object 内では field name の bytewise lexicographic order で最初の field を返します。
複数の domain failure が同時に存在する場合は、`reports` order violation、
duplicate axiom_report_hash、duplicate path の順で最初の1件だけを返します。
`reports` order violation の field は `reports[<i>]`、
`expected_value = "axiom_report_hash_bytewise_ascending"`、
`actual_value = "order_violation"` とし、
`<i>` は最初に `reports[<i>].axiom_report_hash` が直前 entry より小さくなる後続 index です。
duplicate axiom_report_hash の field は `reports[<i>].axiom_report_hash`、
`expected_value = "unique_axiom_report_hashes"`、
`actual_value = "duplicate_axiom_report_hash"` とし、
duplicate path の field は `reports[<i>].path`、
`expected_value = "unique_paths"`、`actual_value = "duplicate_path"` とします。
どちらの `<i>` も同じ key がすでに出現している最小の後続 index です。
`reports[].axiom_report_hash` と referenced `AxiomReport.axiom_report_hash` の一致は
schema-only loader ではなく、artifact resolution / oracle evaluation の段階で検査します。

required `axiom_policy` result for a CI target uses
`RunnerPolicy.required_checker_profiles[0]` as `selector.checker_profile`.
After `NormalizedCheckResult.comparison.status = all_agree_checked`, all required checked
results have the same `axiom_report_hash`, so this baseline selector is deterministic.
required `reproducibility` result for a CI target also uses
`RunnerPolicy.required_checker_profiles[0]` as `selector.checker_profile`.
MVP does not require separate required `axiom_policy` or `reproducibility`
`AuxiliaryResult` entries for every required profile.

`status` は `passed` / `failed` / `inconclusive` のいずれかです。
`status = failed` または `status = inconclusive` では `error.kind` と
`error.reason_code` を required にします。
`status = passed` では `error` を omit します。
MVP の `AuxiliaryResult.error.kind` は `auxiliary_failure` だけです。
MVP の `AuxiliaryResult.error.reason_code` は次に限定します。

```text
- axiom_policy_failed
- axiom_policy_inconclusive
- reproducibility_mismatch
- reproducibility_inconclusive
- import_certificate_hash_mismatch
- import_certificate_hash_inconclusive
- audit_bundle_missing
- audit_bundle_invalid
```

`reason_code` は `kind` と次の表で整合しなければなりません。

```text
kind = axiom_policy:
  failed: axiom_policy_failed
  inconclusive: axiom_policy_inconclusive

kind = reproducibility:
  failed: reproducibility_mismatch
  inconclusive: reproducibility_inconclusive

kind = import_certificate_hash:
  failed: import_certificate_hash_mismatch
  inconclusive: import_certificate_hash_inconclusive

kind = audit_bundle:
  failed: audit_bundle_missing, audit_bundle_invalid
  inconclusive: not allowed in MVP
```

`status = failed` では `failed` / `mismatch` / `missing` / `invalid` 系の reason code だけを使います。
`status = inconclusive` では `_inconclusive` 系の reason code だけを使います。
hash mismatch では `error.field`、`error.expected_hash`、`error.actual_hash` を入れます。
`npa-check auxiliary *` は、oracle evaluation が完了して well-formed `AuxiliaryResult` を
出力できた場合、`AuxiliaryResult.status` が `passed`、`failed`、`inconclusive` のどれであっても
process exit code は 0 です。
`npa-check release validate-bundle` は MVP では `kind = audit_bundle` だけを出力するため、
well-formed `AuxiliaryResult` の status は `passed` または `failed` に限定し、
`inconclusive` は出力してはいけません。
`release validate-bundle` でも well-formed `AuxiliaryResult` を出力できた場合の process exit code は 0 です。
CI pass / fail は process exit code ではなく `AuxiliaryResult.status` と mode-specific pass condition で判定します。
CLI reference pair が揃った後に path / hash schema validation に失敗した、
top-level command input file を読めない、top-level command input hash が一致しない、
または output を atomic write できない場合は、oracle evaluation の前段で失敗したものとして
process exit code 1 と `CommandError` を返します。
required reference pair の欠落または片側指定の分類は command ごとに固定します。
各 command 節が CLI argument validation error と明記する場合だけ `CommandError` body を返しません。
各 command 節が `input_reference_invalid` などの `CommandError.reason_code` を明記する場合は、
reference pair の欠落または片側指定も structured `CommandError` として返します。
`auxiliary import-certificate-hash` だけは例外として、`--import-lock` file が readable で
`--import-lock-hash` と file bytes hash が一致した後、その import lock manifest の
JSON / schema / domain validation に失敗した場合を oracle evaluation 済みの
`AuxiliaryResult.status = inconclusive` として扱います。
`--import-lock` と `--import-lock-hash` が両方欠けている場合は
missing required flag の CLI argument validation error であり、`CommandError` body を返しません。
`--import-lock` unreadable、`--import-lock-hash` mismatch、または `--import-lock` /
`--import-lock-hash` pair の片側指定は
`AuxiliaryResult` ではなく `CommandError` です。
`--import-lock` path schema violation、`--import-lock-hash` invalid hash format、
および片側指定の `CommandError.reason_code` は `input_reference_invalid` です。
`--import-lock` だけが欠けている場合は `field = "import_lock.path"`、
`--import-lock-hash` だけが欠けている場合は `field = "import_lock.manifest_hash"`、
`expected_value = "required"`、`actual_value = "missing"` とします。
`--import-lock` path schema violation では `field = "import_lock.path"`、
`expected_value = "workspace_relative_path"`、`actual_value = "invalid_path"` とします。
`--import-lock-hash` invalid hash format では `field = "import_lock.manifest_hash"`、
`expected_value = "sha256:<lower-hex>"`、`actual_value = "invalid_hash_format"` とします。
`--import-lock` unreadable は `CommandError.reason_code = input_file_unreadable`、
`field = "import_lock.path"`、`expected_value = "readable_file"`、
`actual_value = "unreadable"` とします。
`--import-lock-hash` mismatch は `CommandError.reason_code = input_hash_mismatch`、
`field = "import_lock.manifest_hash"`、`expected_hash = caller supplied hash`、
`actual_hash = import lock file bytes sha256` とします。
MVP の AuxiliaryResult kind ごとの input と oracle は次です。

```text
kind = axiom_policy:
  input:
    - RunnerPolicy.axiom_policy.hash
    - NormalizedCheckResult resolved by selector.normalized_result_hash
    - selector.normalized_result_hash
    - selector.checker_profile
    - selector.result_hash
    - selector.axiom_report_hash
    - axiom report artifact resolved by axiom_report_hash
  oracle:
    deterministic axiom policy evaluator over the axiom report artifact.
    passed iff the selected normalized result entry matches selector.result_hash and
    selector.axiom_report_hash, the axiom report self-hash is valid, the report module /
    certificate_hash match the selected target, and every used axiom is allowed by the policy.

kind = reproducibility:
  input:
    - selector.request_hash
    - selector.checker_profile
    - baseline MachineCheckResult resolved by selector.baseline_run_artifact_hash
    - repeated MachineCheckResult resolved by selector.repeated_run_artifact_hash
    - same RunnerPolicy hash and checker binary identity
  oracle:
    deterministic equality of status, derived failure_key from MachineCheckResult.error, certificate_hash,
    export_hash, axiom_report_hash, and result_hash.
    result_id, attempt, process, resource_usage, and diagnostics are ignored.

kind = import_certificate_hash:
  input:
    - ReleasePolicy with mode = high-trust
    - import lock manifest
    - imported certificate files referenced by the lock
  oracle:
    each imported certificate canonical certificate_hash recomputed by the
    built-in deterministic canonical certificate hash oracle
    matches the certificate_hash recorded in the import lock.
    export_hash and full semantic checking are not evaluated by this oracle.

kind = audit_bundle:
  input:
    - release audit bundle manifest
    - files referenced by the bundle manifest
  oracle:
    run the complete release audit bundle validator defined in section 13,
    including mode/trust checks, closed artifact sets, store manifest checks,
    sidecar revalidation, challenge coverage validation, and all cross-artifact
    reference checks.
```

`axiom_policy` oracle は次の順序で検査し、最初の失敗だけを
`AuxiliaryResult.error` に記録します。
CLI の `npa-check auxiliary axiom-policy` では `--normalized-result-hash` と
validated `NormalizedCheckResult.normalized_result_hash` の mismatch は oracle 前の
`CommandError.reason_code = input_hash_mismatch` です。
したがって下の `selector.normalized_result_hash` mismatch は、明示 selector を受け取る
library/API oracle evaluation、または保存済み `AuxiliaryResult` envelope の selector validation 用です。
CLI が新規生成する `AuxiliaryResult.selector.normalized_result_hash` は validated hash を必ず写します。

```text
selector normalized result hash mismatch:
  status = inconclusive
  reason_code = axiom_policy_inconclusive
  field = "selector.normalized_result_hash"
  expected_hash = validated NormalizedCheckResult.normalized_result_hash
  actual_hash = selector.normalized_result_hash

selected normalized result entry missing / unusable:
  status = inconclusive
  reason_code = axiom_policy_inconclusive
  field = "selector.checker_profile"
  expected_value = "checked_normalized_result_entry"
  actual_value = missing | not_checked | missing_axiom_report_hash

selector result_hash mismatch:
  status = inconclusive
  reason_code = axiom_policy_inconclusive
  field = "selector.result_hash"
  expected_hash = selected NormalizedCheckResult.results[<j>].result_hash
  actual_hash = selector.result_hash

selector axiom_report_hash mismatch:
  status = inconclusive
  reason_code = axiom_policy_inconclusive
  field = "selector.axiom_report_hash"
  expected_hash = selected NormalizedCheckResult.results[<j>].axiom_report_hash
  actual_hash = selector.axiom_report_hash

axiom report artifact missing / unreadable:
  status = inconclusive
  reason_code = axiom_policy_inconclusive
  field = "selector.axiom_report_hash"
  expected_value = "resolvable_axiom_report"
  actual_value = missing | unreadable

axiom report JSON / schema / domain failure:
  status = inconclusive
  reason_code = axiom_policy_inconclusive
  field = "axiom_report" for root-level failure, otherwise "axiom_report.<JSON path>"
  expected_value = "valid_json" for JSON parse failure, otherwise <schema requirement name>
  actual_value = invalid_json | missing | wrong_type | unknown_field |
                 invalid_enum | invalid_hash_format | invalid_name_format |
                 null_not_allowed | order_violation | duplicate_field |
                 duplicate_axiom_name

axiom report store entry file bytes hash mismatch:
  status = inconclusive
  reason_code = axiom_policy_inconclusive
  field = "axiom_report_store.reports[<i>].file_hash"
  expected_hash = AxiomReportStoreManifest.reports[<i>].file_hash
  actual_hash = referenced AxiomReport file bytes sha256

axiom report self-hash mismatch:
  status = inconclusive
  reason_code = axiom_policy_inconclusive
  field = "axiom_report.axiom_report_hash"
  expected_hash = recomputed AxiomReport.axiom_report_hash
  actual_hash = parsed AxiomReport.axiom_report_hash

axiom report store entry axiom_report_hash mismatch:
  status = inconclusive
  reason_code = axiom_policy_inconclusive
  field = "axiom_report_store.reports[<i>].axiom_report_hash"
  expected_hash = AxiomReportStoreManifest.reports[<i>].axiom_report_hash
  actual_hash = parsed AxiomReport.axiom_report_hash

axiom report hash mismatch:
  status = failed
  reason_code = axiom_policy_failed
  field = "selector.axiom_report_hash"
  expected_hash = selector.axiom_report_hash
  actual_hash = parsed AxiomReport.axiom_report_hash

axiom report module mismatch:
  status = inconclusive
  reason_code = axiom_policy_inconclusive
  field = "axiom_report.module"
  expected_value = selected NormalizedCheckResult.artifact.module
  actual_value = parsed AxiomReport.module

axiom report certificate_hash mismatch:
  status = inconclusive
  reason_code = axiom_policy_inconclusive
  field = "axiom_report.certificate_hash"
  expected_hash = selected NormalizedCheckResult.results[<j>].certificate_hash
  actual_hash = parsed AxiomReport.certificate_hash

disallowed axiom:
  status = failed
  reason_code = axiom_policy_failed
  field = "axiom_report.axioms[<i>].name"
  expected_value = "allowed_axiom"
  actual_value = parsed AxiomName の dotted Phase 8 JSON representation
```

`axiom_report.axioms[<i>]` の `<i>` は axiom report の deterministic order で最初に
policy に違反した axiom entry の zero-based index です。
`axiom_report_store.reports[<i>]` の `<i>` は axiom report store manifest 内の
該当 entry の zero-based index です。
`selected NormalizedCheckResult.results[<j>]` の `<j>` は selector の `checker_profile` で
選ばれた normalized result entry の zero-based index です。
`axiom_policy` の axiom report store entry validation は、missing / unreadable、
invalid JSON、schema / domain failure、file_hash mismatch、artifact self-hash mismatch、
manifest-field mismatch の順で判定します。

`reproducibility` oracle は次の順序で検査し、最初の失敗だけを
`AuxiliaryResult.error` に記録します。
表の各 row は上から順に評価します。
row 名に `baseline / repeated` とある場合、同じ row 内では baseline を先に、
次に repeated を検査してから次の row に進みます。
したがって repeated の earlier-row failure は baseline の later-row failure より先に報告します。

```text
baseline / repeated MachineCheckResult missing / unreadable:
  status = inconclusive
  reason_code = reproducibility_inconclusive
  field = "selector.baseline_run_artifact_hash" or "selector.repeated_run_artifact_hash"
  expected_value = "resolvable_machine_check_result"
  actual_value = missing | unreadable

baseline / repeated MachineCheckResult JSON / schema / domain failure:
  status = inconclusive
  reason_code = reproducibility_inconclusive
  field = "baseline" / "repeated" for root-level failure,
          otherwise "baseline.<JSON path>" / "repeated.<JSON path>"
  expected_value = "valid_json" for JSON parse failure, otherwise <schema requirement name>
  actual_value = invalid_json | missing | wrong_type | unknown_field |
                 invalid_enum | invalid_hash_format | invalid_name_format | null_not_allowed |
                 order_violation | duplicate_field

baseline / repeated result store entry file bytes hash mismatch:
  status = inconclusive
  reason_code = reproducibility_inconclusive
  field = "result_store.results[<i>].file_hash"
  expected_hash = result store entry file_hash
  actual_hash = referenced MachineCheckResult file bytes sha256

baseline / repeated MachineCheckResult result_hash self-hash mismatch:
  status = inconclusive
  reason_code = reproducibility_inconclusive
  field = "baseline.result_hash" or "repeated.result_hash"
  expected_hash = recomputed MachineCheckResult.result_hash
  actual_hash = parsed MachineCheckResult.result_hash

baseline / repeated MachineCheckResult run_artifact_hash self-hash mismatch:
  status = inconclusive
  reason_code = reproducibility_inconclusive
  field = "baseline.run_artifact_hash" or "repeated.run_artifact_hash"
  expected_hash = recomputed MachineCheckResult.run_artifact_hash
  actual_hash = parsed MachineCheckResult.run_artifact_hash

baseline / repeated result store entry manifest-field mismatch:
  status = inconclusive
  reason_code = reproducibility_inconclusive
  field = first mismatching field in:
          result_store.results[<i>].result_hash,
          result_store.results[<i>].request_hash,
          result_store.results[<i>].run_artifact_hash,
          result_store.results[<i>].checker_profile
  result_hash / request_hash / run_artifact_hash:
    expected_hash = result store entry field value
    actual_hash = parsed MachineCheckResult same field value
  checker_profile:
    expected_value = result store entry checker_profile
    actual_value = parsed MachineCheckResult.checker.profile

baseline / repeated selector run_artifact_hash mismatch:
  status = inconclusive
  reason_code = reproducibility_inconclusive
  field = "selector.baseline_run_artifact_hash" or "selector.repeated_run_artifact_hash"
  expected_hash = corresponding selector run_artifact_hash
  actual_hash = parsed MachineCheckResult.run_artifact_hash

selector / comparability precondition mismatch:
  status = inconclusive
  reason_code = reproducibility_inconclusive
  field = first mismatching field in:
          baseline.request_hash, repeated.request_hash,
          baseline.checker.profile, repeated.checker.profile,
          baseline.policy.hash, repeated.policy.hash,
          baseline.checker.binary_id, repeated.checker.binary_id,
          baseline.checker.binary_hash, repeated.checker.binary_hash,
          baseline.checker.id, repeated.checker.id,
          baseline.checker.build_hash,
          repeated.checker.build_hash
  expected_value / expected_hash:
    - baseline.request_hash, repeated.request_hash:
        selector.request_hash
    - baseline.checker.profile, repeated.checker.profile:
        selector.checker_profile
    - baseline.policy.hash, repeated.policy.hash:
        active RunnerPolicy hash
    - baseline.checker.binary_id:
        SelectedCheckerPolicy.binary_id
    - baseline.checker.binary_hash:
        SelectedCheckerPolicy.binary_hash
    - baseline.checker.id:
        SelectedCheckerPolicy.checker_id
    - baseline.checker.build_hash:
        SelectedCheckerPolicy.build_hash
    - repeated checker identity fields:
        corresponding baseline checker identity field
  actual_value / actual_hash = loaded baseline or repeated value

deterministic reproducibility mismatch:
  status = failed
  reason_code = reproducibility_mismatch
  field = first mismatching field in:
          repeated.status, repeated.derived_failure_key,
          repeated.certificate_hash, repeated.export_hash,
          repeated.axiom_report_hash, repeated.result_hash
  expected_value / expected_hash = baseline value
  actual_value / actual_hash = repeated value
```

`reproducibility` の `certificate_hash`、`export_hash`、`axiom_report_hash` で
presence だけが異なる場合は、`expected_value = present | absent`、
`actual_value = missing | present` を使います。
両方 present で値が違う場合は `expected_hash` / `actual_hash` を使います。
`reproducibility` の comparability precondition で expected field が存在すべきなのに
actual field が missing の場合は `expected_value = "present"`、
`actual_value = "missing"` を使います。
両方 present で値が違う場合、hash field では `expected_hash` / `actual_hash`、
non-hash field では `expected_value` / `actual_value` を使います。
`derived_failure_key` は saved `MachineCheckResult` field ではなく、baseline / repeated の
`MachineCheckResult.error` から 7 の normalized failure key rule で導出した object です。
`derived_failure_key` は両方の result が `status = failed` の場合だけ比較します。
`status` が異なる場合は `repeated.status` mismatch を報告し、`derived_failure_key` は比較しません。
`derived_failure_key` は canonical hash した値で比較し、
`expected_hash` / `actual_hash` を使います。
`MachineCheckResult.run_artifact_hash` self-hash は、同じ artifact の `result_hash`
self-hash が一致した場合だけ正当な integrity hash として扱います。
`result_store.results[<i>]` の `<i>` は machine result store manifest 内の
該当 entry の zero-based index です。
`reproducibility` の machine result store entry validation は、missing / unreadable、
invalid JSON、schema / domain failure、file_hash mismatch、artifact self-hash mismatch、
manifest-field mismatch の順で判定します。

`import_certificate_hash` oracle は import lock manifest の `imports` order で entry を検査し、
最初の失敗だけを `AuxiliaryResult.error` に記録します。
下の field shape で `imports[<i>]` と書く場合、`<i>` は最初に失敗した entry の
zero-based index です。実際の `AuxiliaryResult.error.field` には `imports[]` のような
index なし表記を出してはいけません。
field shape は次で固定します。

```text
import lock manifest JSON parse failure after readable/hash-verified:
  status = inconclusive
  reason_code = import_certificate_hash_inconclusive
  field = "import_lock"
  expected_value = "valid_json"
  actual_value = "invalid_json"

import lock manifest schema / domain failure after readable/hash-verified:
  status = inconclusive
  reason_code = import_certificate_hash_inconclusive
  field = "import_lock" for root-level failure, otherwise "import_lock.<JSON path>"
  expected_value = <schema requirement name>
  actual_value = missing | wrong_type | unknown_field | invalid_enum |
                 invalid_hash_format | invalid_name_format | invalid_path | null_not_allowed |
                 order_violation | duplicate_field | duplicate_module |
                 duplicate_path

imported certificate file missing / unreadable:
  status = inconclusive
  reason_code = import_certificate_hash_inconclusive
  field = "import_lock.imports[<i>].certificate.path"
  expected_value = "readable_file"
  actual_value = "missing" | "unreadable"

imported certificate file bytes hash mismatch:
  status = failed
  reason_code = import_certificate_hash_mismatch
  field = "import_lock.imports[<i>].certificate.file_hash"
  expected_hash = import lock imports[<i>].certificate.file_hash
  actual_hash = imported certificate file bytes sha256

imported certificate canonical decode / serialization failure:
  status = failed
  reason_code = import_certificate_hash_mismatch
  field = "import_lock.imports[<i>].certificate.path"
  expected_value = "canonical_certificate"
  actual_value = "invalid_certificate_encoding"

imported certificate canonical certificate_hash mismatch:
  status = failed
  reason_code = import_certificate_hash_mismatch
  field = "import_lock.imports[<i>].certificate.certificate_hash"
  expected_hash = import lock imports[<i>].certificate.certificate_hash
  actual_hash = recomputed canonical certificate hash
```

`actual_value = "missing"` は path が schema-valid だが file が存在しない場合、
`actual_value = "unreadable"` は file が存在するが read できない場合に使います。
`import_certificate_hash` oracle は `imports[].export_hash` を再計算・照合しません。
`export_hash` と import certificate の full semantic validity は checker import validation の責務です。
If an oracle-internal input required for a deterministic oracle is missing after
top-level command input validation has succeeded, use the corresponding
`*_inconclusive` reason code when one exists.
For `audit_bundle`, missing or invalid referenced bundle artifacts are `failed`,
not `inconclusive`.
The top-level `--manifest` path/hash pair of `release validate-bundle` is command
input, so unreadable manifest file, manifest file hash mismatch, and malformed
path / hash values after both flags are present are `CommandError`, not
`audit_bundle_missing`.
Missing or one-sided `--manifest` / `--manifest-hash` pairs are classified by
the `release validate-bundle` CLI argument validation rule below, not by this
oracle failure rule.
If the manifest file is readable and hash-verified but cannot be parsed far
enough to obtain the minimum audit envelope (`bundle_hash`, `policy_hash`,
`artifact_hash`, and `artifacts` array), the command
returns `CommandError.reason_code = input_json_invalid` or `input_schema_invalid`.
`npa-check auxiliary import-certificate-hash` does not launch a checker binary selected from
`RunnerPolicy.checker_allowlist` and does not take `checker_profile` as input.
The command uses the built-in deterministic canonical certificate decoder / hash oracle,
which must implement the same canonical certificate hash rule as the independent checker.
`ReleasePolicy` is used for `policy_hash`, mode gating, and high-trust pass-context identity.
`--release-policy` / `--release-policy-hash` reference failures use the same policy-reference
reason families as other deterministic commands, but with `release_policy.*` field paths.
If both `--release-policy` and `--release-policy-hash` are absent, this is a missing
required flag CLI argument validation error and returns no `CommandError` body.
One-sided pair, path schema violation, hash format violation, JSON parse failure,
and schema / domain validation failure of the referenced file are
`CommandError.reason_code = policy_reference_invalid`.
One-sided pair failures use `field = "release_policy.path"` when only
`--release-policy-hash` is present, `field = "release_policy.hash"` when only
`--release-policy` is present, `expected_value = "required"`、`actual_value = "missing"`。
Path schema violation uses `field = "release_policy.path"`、
`expected_value = "workspace_relative_path"`、`actual_value = "invalid_path"`。
Hash format violation uses `field = "release_policy.hash"`、
`expected_value = "sha256:<lower-hex>"`、`actual_value = "invalid_hash_format"`。
JSON parse failure uses `field = "release_policy.path"`、
`expected_value = "valid_json"`、`actual_value = "invalid_json"`。
Unreadable `--release-policy` is `policy_file_unreadable` with `field = "release_policy.path"`、
`expected_value = "readable_file"`、`actual_value = "unreadable"`。
`--release-policy-hash` mismatch is `policy_hash_mismatch` with
`field = "release_policy.hash"`、`expected_hash = caller supplied hash`、
`actual_hash = parsed ReleasePolicy canonical hash`。
JSON / schema / domain validation failure follows `ReleasePolicy` field shapes, not
`RunnerPolicy` field shapes, after applying the `release_policy` prefix rule.
If the referenced `ReleasePolicy.mode` is not `high-trust`, the command returns
`CommandError.reason_code = policy_reference_invalid` with
`field = "release_policy.mode"`、`expected_value = "high-trust"`、
`actual_value = <ReleasePolicy.mode>` and emits no `AuxiliaryResult`.
`diagnostics` は optional で、3.3 の fixed diagnostics token rule に従います。
`result_hash` は `result_id`、`result_hash`、`diagnostics` field を除いた canonical hash です。
`error` に自然言語を入れてはいけません。
人間向け説明は fixed diagnostics token、AI sidecar、または artifact 外ログに分離します。

mode ごとの required artifacts：

```text
pr:
  - MachineCheckRequest
  - MachineCheckResult for required profiles
  - NormalizedCheckResult with embedded comparison
  - axiom policy result

nightly:
  - PR mode artifacts
  - external checker result
  - reproducibility result
  - challenge output store manifest that defines the coverage universe
  - rejection-required challenge manifests referenced by that store manifest
  - challenge replay results referenced by the challenge coverage summary
  - challenge coverage summary
  - AI audit sidecar and validation responses only for failed / non-success
    CI diagnostic targets when ReleasePolicy.ai_triage.enabled = true
    and ai_triage.required = true

release:
  - nightly mode artifacts
  - release policy file
  - RunnerPolicy files referenced by the release policy
  - release audit bundle
  - checker binary identity manifest files referenced by those RunnerPolicy files, when present
  - import lock files referenced by included requests / normalized results / challenges, when present
  - AI audit input policy when ReleasePolicy.ai_triage.enabled = true
  - optional AI audit sidecar with input_policy and prompt_hash only when
    explicitly included as valid optional audit metadata

high-trust:
  - release mode artifacts
  - high-trust-reference checker result
  - import certificate_hash verification
  - retained raw result artifacts in append-only storage
```

nightly pipeline may save informational `ChallengeReplayResult` artifacts in a diagnostic store,
but they are not nightly required artifacts and do not contribute to coverage or pass conditions.
Only replay results referenced by `ChallengeCoverageSummary.entries[*].replay_result_hash`
are required for nightly pass.

AI sidecar diagnostic が必須の target でも、sidecar 生成失敗は「説明 artifact の不足」です。
checker failure を success に変えることも、checker success を failure に変えることもありません。
required diagnostic target が空の pass pipeline では、required AI sidecar artifact はありません。

PR comment に出す AI summary は、必ず checker result への参照を持ちます。

```text
good:
  Reference checker failed with `declaration_hash_mismatch` in `Nat.add_zero`.
  See result `mchkres_002`.

bad:
  The proof is probably fine; this looks like a checker issue.
```

---

# 13. Release audit

release mode では AI sidecar をより厳しく扱います。
AI の説明自体は信用しませんが、監査しやすい形式で保存します。

release audit bundle に含めるもの：

```text
- release policy file
- RunnerPolicy files referenced by the release policy
- AI audit input policy file when ReleasePolicy.ai_triage.enabled = true
- MachineCheckRequest files
- request store manifest covering included MachineCheckRequest files
- raw MachineCheckResult files
- machine result store manifest covering included MachineCheckResult files
- release target NormalizedCheckResult
- challenge replay NormalizedCheckResult for each included ChallengeReplayResult.normalized_result_hash
- normalized result store manifest covering included NormalizedCheckResult files
- checker binary identity manifest files referenced by included RunnerPolicy files, when present
- import lock files referenced by included requests / normalized results / challenges, when present
- passed axiom_policy AuxiliaryResult for the release target
- passed reproducibility AuxiliaryResult for the release target
- passed import_certificate_hash AuxiliaryResult for each referenced import lock, required when ReleasePolicy.mode = high-trust
- challenge output store manifest that defines the coverage universe
- rejection-required challenge manifests referenced by the challenge output store manifest
- challenge replay results referenced by the challenge coverage summary
- optional AI audit sidecar and matching AuditSidecarValidationResult response, only when both are valid and ReleasePolicy.ai_triage.enabled = true
- optional CompareValidationResult response for included NormalizedCheckResult entries, only when valid
- challenge coverage summary
```

MVP の `ReleaseAuditBundleManifest` schema：

```json
{
  "schema": "npa.phase8.release_audit_bundle_manifest.v1",
  "bundle_id": "release_0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
  "bundle_hash": "sha256:...",
  "policy_hash": "sha256:...",
  "artifact_hash": "sha256:...",
  "artifacts": [
    {
      "kind": "machine_check_result",
      "path": "artifacts/machine_check_result/0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef.json",
      "file_hash": "sha256:...",
      "hashes": {
        "result_hash": "sha256:...",
        "run_artifact_hash": "sha256:..."
      }
    },
    {
      "kind": "normalized_check_result",
      "path": "artifacts/normalized_check_result/0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef.json",
      "file_hash": "sha256:...",
      "hashes": {
        "normalized_result_hash": "sha256:...",
        "artifact_hash": "sha256:..."
      }
    }
  ]
}
```

`bundle_hash` は `bundle_id` と `bundle_hash` field を除いた
`ReleaseAuditBundleManifest` の canonical hash です。
`bundle_id` は `release_` + `bundle_hash` の `sha256:` prefix を除いた lower-hex
64文字に固定します。
`npa-check release bundle` は `bundle_hash` を計算してから `bundle_id` を埋めます。
bundle validator は `bundle_id` が `bundle_hash` から導出した値と一致することを検査し、
不一致なら bundle invalid です。
`policy_hash` は `ReleasePolicy` の canonical hash です。
MVP の release audit bundle には `kind = release_policy` artifact entry がちょうど1件必要で、
その `hashes.policy_hash` は top-level `policy_hash` と一致しなければなりません。
bundle validator は release policy file を parse し、
`ReleasePolicy.runner_policy_hash` と `ReleasePolicy.challenge_runner_policy_hash` を解決します。
それぞれの hash について、同じ bundle 内に `kind = runner_policy` entry が
ちょうど1件存在しなければなりません。
両者が同じ hash の場合は1件の `runner_policy` entry で兼ねてよいです。
その場合でも `npa-check release bundle` invocation では
`--runner-policy` / `--runner-policy-hash` と
`--challenge-runner-policy` / `--challenge-runner-policy-hash` の両方を明示します。
両 hash が同じ場合、この2つの path は bytewise に同じ bundle-local path でなければならず、
bundle manifest には `runner_policy` entry を1件だけ出します。
同じ hash に対して異なる path が指定された場合は
`CommandError.reason_code = release_bundle_generation_failed`、
`field = "runner_policy"`、`expected_value = "single_path_for_shared_runner_policy_hash"`、
`actual_value = "multiple_paths"` です。
bundle validator は included `runner_policy` artifact を parse し、
各 `RunnerPolicy` の canonical hash が対応する `hashes.policy_hash` と一致することを検査します。
さらに included `ReleasePolicy.mode` と、`runner_policy_hash` /
`challenge_runner_policy_hash` から解決した `RunnerPolicy.trust_mode` が
12 の対応表どおり一致することを検査します。
一致しない場合は 12 の mode / trust mismatch field shape で bundle invalid です。
included `RunnerPolicy` が `checker_identity_manifest` を持つ場合、
その `checker_identity_manifest.manifest_hash` ごとに、同じ bundle 内に
`kind = checker_identity_manifest` entry がちょうど1件存在しなければなりません。
複数の included `RunnerPolicy` が同じ `checker_identity_manifest.manifest_hash` を参照する場合は、
1件の `checker_identity_manifest` entry で兼ねてよいです。
異なる manifest hash を参照する場合は、distinct hash ごとに1件ずつ必要です。
各 entry の `hashes.manifest_hash` は、参照元 `RunnerPolicy.checker_identity_manifest.manifest_hash`
および referenced manifest file bytes sha256 と一致しなければなりません。
included `RunnerPolicy` のどれからも参照されない `checker_identity_manifest` entry は forbidden です。
included `RunnerPolicy` が1つも `checker_identity_manifest` を持たない場合、
`checker_identity_manifest` entry を含めてはいけません。
bundle validator は included `checker_identity_manifest` file を parse し、
`CheckerIdentityManifest` schema / domain validation と file-byte `manifest_hash` 検査を行います。
release audit では単一 run の selected profile だけではなく、
included `RunnerPolicy.checker_allowlist` の全 entry を対象に manifest completeness を検査します。
各 allowlist entry について、同じ `profile`、`checker_id`、`binary_id`、`binary_hash`、
`build_hash` を持つ `CheckerIdentityManifest.checkers[]` entry が存在しなければなりません。
manifest が included RunnerPolicy で参照されない profile の entry を追加で持つことは許可します。
bundle validator は追加 entry を policy allowlist として扱ってはいけません。
included artifact から参照される import lock hash は、次の集合で決まります。

```text
- included MachineCheckRequest.imports.manifest_hash
- included NormalizedCheckResult.artifact.import_lock_hash
- included rejection-required ChallengeManifest.imports.manifest_hash
```

Phase 8 MVP の release audit bundle では informational challenge manifest は forbidden なので、
informational challenge manifest からの import lock reference はこの集合に入りません。
informational artifact の import lock verification は、bundle 外の diagnostic store、
または将来の postmortem manifest の責務です。

この集合の distinct hash ごとに、同じ bundle 内に `kind = import_lock` entry が
ちょうど1件存在しなければなりません。
参照元 artifact set が 13 の prerequisite gate で定義する prerequisite-clean でない場合、
bundle generator / validator は import lock hash 集合を導出してはいけません。
この場合は prerequisite-clean でない参照元 artifact kind の failure を先に返します。
複数 artifact が同じ import lock hash を参照する場合は1件の `import_lock` entry で兼ねてよいです。
異なる import lock hash を参照する場合は、distinct hash ごとに1件ずつ必要です。
各 entry の `hashes.manifest_hash` は、参照元 artifact の import lock hash
および referenced import lock file bytes sha256 と一致しなければなりません。
`import_lock` entry の `path` は release bundle-local artifact path です。
元の `MachineCheckRequest.imports.manifest` や `ChallengeManifest.imports.manifest` の
path と bytewise に一致する必要はありません。
bundle generator は import lock file を bundle 内へ deterministic に配置してよく、
bundle validator は original path ではなく `hashes.manifest_hash` と file bytes sha256 だけを
binding として扱います。
original path は invocation provenance であり、bundle 内の trusted reference identity ではありません。
上の集合に含まれない `import_lock` entry は forbidden です。
上の集合が空の場合、`import_lock` entry を含めてはいけません。
`ReleasePolicy.ai_triage.enabled = true` の場合、同じ bundle 内に
`kind = ai_audit_input_policy` entry がちょうど1件必要で、
その `hashes.input_policy_hash` は `ReleasePolicy.ai_triage.input_policy_hash` と
一致しなければなりません。
bundle validator は input policy file を parse し、canonical hash を再計算して
`hashes.input_policy_hash` と一致することを検査します。
`ReleasePolicy.ai_triage.enabled = false` の場合、`kind = ai_audit_input_policy` entry は
forbidden です。
MVP の release audit bundle には、`kind = request_store_manifest`、
`kind = machine_result_store_manifest`、`kind = normalized_result_store_manifest` entry が
それぞれちょうど1件必要です。
この3件は release bundle-local manifest です。
release bundle generation の前に、pipeline は deterministic pre-bundle staging を2 phase で実行します。

```text
store phase:
  request / machine result / normalized result artifacts と
  target-scoped ChallengeOutputStoreManifest を bundle-root 配下へ stage し、
  bundle-local request / machine result / normalized result store manifest を作る。
  release / high-trust 用 ChallengeCoverageSummary は、この phase が成功した後に生成する。

final phase:
  release policy、RunnerPolicy、checker identity manifest、import lock、
  challenge manifest、challenge replay result、coverage summary、auxiliary result、
  optional AI sidecar / validation response、optional compare validation response を stage する。
  store phase で作った bundle-local store manifest は書き換えない。
```

各 phase は `npa-check release stage-bundle-inputs` で実行します。
この command は `--phase store|final`、`--bundle-root <path>`、
`--plan <path>`、`--plan-hash <sha256:...>`、`--json` を受け取ります。
Phase 8 MVP ではこれら5つの flag はすべて required です。
欠落、duplicate singleton flag、unsupported flag、または `--phase` が `store|final` 以外の場合は
CLI argument validation error であり、`CommandError` body を返しません。
`--plan` は workspace-relative path schema の対象です。
absolute path、drive prefix、empty segment、`.` / `..` segment、control character は forbidden です。
`--plan` path は invocation cwd から1回だけ解決します。
`--plan-hash` は `--plan` で指定した file の exact bytes SHA-256 です。
`--plan-hash` が `sha256:<64 lower-hex>` 形式でない場合は
`CommandError.reason_code = input_reference_invalid`、`field = "plan_hash"`、
`expected_value = "sha256:<lower-hex>"`、`actual_value = "invalid_hash_format"` です。
plan を parse / normalize / reserialize した結果の hash ではありません。
`ReleaseBundleStagingPlan` は source artifact / source store manifest を explicit path + hash で列挙する
pipeline-local JSON です。
MVP ではこの plan schema は release staging command 専用であり、release audit bundle artifact には含めません。
command は directory scan、glob、bundle-root からの implicit discovery、policy hash lookup を行いません。
Phase 8 MVP の `release stage-bundle-inputs` は release / high-trust / local diagnostic のいずれでも
invocation cwd を repository root に固定します。
cwd override、process cwd の暗黙採用、symlink resolution、case folding、path normalization、
environment variable expansion は path identity に使ってはいけません。

MVP の `ReleaseBundleStagingPlan` の top-level required field：

```text
- schema = npa.phase8.release_bundle_staging_plan.v1
- phase = store | final
- bundle_root
- inputs[]
```

`ReleaseBundleStagingPlan` は strict JSON object です。
top-level object と `inputs[]` object の unknown field は schema invalid です。
plan 内の `phase` は CLI の `--phase` と bytewise に一致しなければなりません。
不一致は `input_schema_invalid` で、`field = "phase"`、
`expected_value = "--phase:<store|final>"`、`actual_value = "plan.phase:<store|final>"` とします。
`bundle_root` と `--bundle-root` はどちらも workspace-relative path schema の対象です。
absolute path、drive prefix、empty segment、`.` / `..` segment、control character は forbidden です。
`--bundle-root` の path schema violation は `input_reference_invalid` で、
`field = "bundle_root"`、`expected_value = "workspace_relative_path"`、
`actual_value = "invalid_path"` とします。
plan 内 `bundle_root` の path schema violation は `input_schema_invalid` で、
`field = "bundle_root"`、`expected_value = "workspace_relative_path"`、
`actual_value = "invalid_path"` とします。
`bundle_root` と `--bundle-root` は解決前の文字列として bytewise に一致しなければなりません。
不一致は `input_schema_invalid` で、`field = "bundle_root"`、
`expected_value = "--bundle-root:<value>"`、`actual_value = "plan.bundle_root:<value>"` とします。
staging command は一致を確認した後、`bundle_root` を invocation cwd から1回だけ解決します。
symlink resolution、case folding、path normalization、environment variable expansion は
`bundle_root` identity に使ってはいけません。
`inputs[]` は `(kind, path, file_hash)` の bytewise lexicographic order で昇順に並べます。
同じ `(kind, path)` を持つ entry は、`file_hash` が異なる場合でも schema invalid です。
同じ `path` を持つ entry は、`kind` が異なる場合でも schema invalid です。
同じ source file bytes が複数 path から到達できる場合でも、staging の deduplication 単位は
後述の `kind + file_hash` であり、plan の order / uniqueness rule は緩めません。
`inputs[].path` は source path であり、generated bundle-local path ではありません。
relative source path は staging command の invocation cwd から1回だけ解決します。
absolute path、empty segment、`.` / `..` segment、control character は source path でも forbidden です。
source path は bundle artifact path にコピーされず、release bundle artifact identity にも含めません。

`inputs[]` は `(kind, path, file_hash)` を required とします。
`file_hash` は常に source file bytes sha256 です。
`expected_hash` という曖昧な field は MVP schema では forbidden です。
`hashes` は artifact kind ごとの parsed hash object で、release bundle artifact kind では required です。
`hashes` の key は下記の artifact kind ごとの `hashes` 定義と同じです。
manifest kind では `hashes.manifest_hash` が required で、`file_hash` と同じ値でなければなりません。
manifest kind の `hashes.manifest_hash` が `file_hash` と異なる場合は、source file bytes と
`file_hash` の照合を先に行います。
`file_hash` mismatch も同時に成立する場合は direct source input file_hash mismatch を返します。
`file_hash` が source file bytes と一致し、`hashes.manifest_hash` だけが異なる場合は
direct source input parsed hash mismatch として `input_hash_mismatch` を返し、
`field = "inputs[<i>].hashes.manifest_hash"`、
`expected_hash = inputs[<i>].hashes.manifest_hash`、
`actual_hash = source file bytes sha256` とします。
artifact kind 定義で required hash が `none` の kind では `hashes` は required empty object です。
required hash が `none` の kind で `hashes` が non-empty object の場合は
`input_schema_invalid` を返し、`field = "inputs[<i>].hashes"`、
`expected_value = "empty_object"`、`actual_value = "non_empty"` とします。
上記の dedicated payload で固定していない `ReleaseBundleStagingPlan` schema / order /
duplicate / path violation も `input_schema_invalid` です。
同時に複数の plan-local violation が成立する場合は、次の順で最初の1件だけを返します。

```text
1. duplicate object key
2. top-level schema / required field / wrong type / explicit null / unknown field
3. phase enum / --phase mismatch
4. bundle_root path schema / --bundle-root mismatch
5. inputs missing / wrong type / explicit null
6. inputs[] element object type / non-hashes member required field / wrong type / explicit null / unknown field
7. inputs[].kind enum violation
8. inputs[].path schema violation
9. inputs[].file_hash format violation
10. inputs[] order violation
11. duplicate input (kind, path) pair
12. duplicate input path
13. phase-kind cardinality violation
14. hashes missing / wrong type / explicit null
15. required hashes field missing / wrong type / explicit null / unknown field / invalid hash format
16. required-hash-none kind with non-empty hashes
```

同じ priority 内で複数 field が失敗する場合は、top-level object、`inputs[]` entry の小さい index、
その entry の `hashes` object の順で containing object を選びます。
同じ object 内では schema order を使います。
top-level schema order は `schema`、`phase`、`bundle_root`、`inputs` です。
`inputs[]` entry schema order は `kind`、`path`、`file_hash`、`hashes` です。
known field の duplicate object key は、その field の schema order 位置で報告します。
unknown field の duplicate object key は unknown field の位置で報告し、
同じ object 内では field name の bytewise lexicographic order で最初の field を返します。
duplicate object key では `field` に duplicate した member の plan JSON path、
`expected_value = "unique_object_keys"`、`actual_value = "duplicate_field"` を入れます。
top-level JSON value が object でない場合は `field = "$"`、
`expected_value = "object"`、`actual_value = "wrong_type"` または `"null_not_allowed"` とします。
top-level `schema` missing / null / wrong type / mismatch では `field = "schema"`、
`expected_value = "npa.phase8.release_bundle_staging_plan.v1"`、
`actual_value` に `missing`、`null_not_allowed`、`wrong_type`、または実際の schema 文字列を入れます。
top-level schema mismatch では `actual_value = "wrong_schema"` を使いません。
missing / wrong type / explicit null / unknown field では `field` に invalid member の plan JSON path、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`null_not_allowed`、または `unknown_field` を入れます。
`inputs[]` array element 自体が object でない場合は `field = "inputs[<i>]"`、
`expected_value = "object"`、`actual_value = "wrong_type"` または `"null_not_allowed"` とします。
`inputs[]` object 内の `hashes` 以外の required member 欠落、wrong type、explicit null、
unknown field では、
`field` に `inputs[<i>].<member>` を入れます。
`hashes` member 自体の missing、wrong type、explicit null は priority 14 で扱い、
`field = "inputs[<i>].hashes"`、`expected_value = "object"`、
`actual_value` に `missing`、`wrong_type`、または `null_not_allowed` を入れます。
`phase` enum violation では `field = "phase"`、
`expected_value = "store|final"`、`actual_value = "invalid_enum"` とします。
priority 3 内では `phase` enum violation を `--phase` mismatch より先に返します。
`--phase` mismatch は plan の `phase` が schema-valid な `store` または `final` として読めた場合だけ
評価し、invalid enum から mismatch を合成してはいけません。
priority 4 内では plan `bundle_root` path schema violation を `--bundle-root` mismatch より先に返します。
`--bundle-root` mismatch は CLI `--bundle-root` と plan `bundle_root` の両方が
workspace-relative path schema を通った場合だけ評価します。
inputs[] order / duplicate validation は、すべての entry の `kind` / `path` / `file_hash` が
missing ではなく、explicit null ではなく、JSON string であり、かつそれぞれ kind enum /
workspace-relative path schema / hash format validation を通過した後にだけ行います。
`inputs[]` order violation では、最初に昇順を破った後続 element の index を使い
`field = "inputs[<i>]"`、`expected_value = "inputs_sorted_by_kind_path_file_hash"`、
`actual_value = "order_violation"` とします。
duplicate input `(kind, path)` pair では、後続 duplicate entry の index を使い
`field = "inputs[<i>]"`、`expected_value = "unique_kind_path"`、
`actual_value = "duplicate_entry"` とします。
この `duplicate_entry` は `(kind, path)` pair の重複を意味し、`file_hash` が一致する場合にも
異なる場合にも使います。
duplicate input path では、後続 duplicate path entry の index を使い
`field = "inputs[<i>].path"`、`expected_value = "unique_path"`、
`actual_value = "duplicate_path"` とします。
`inputs[].path` schema violation では `field = "inputs[<i>].path"`、
`expected_value = "workspace_relative_path"`、`actual_value = "invalid_path"` とします。
`inputs[].file_hash` format violation では `field = "inputs[<i>].file_hash"`、
`expected_value = "sha256:<lower-hex>"`、`actual_value = "invalid_hash_format"` とします。
`hashes` object 内の required hash 欠落、wrong type、explicit null、unknown field、
invalid hash format では
`field = "inputs[<i>].hashes.<hash_field>"` を使い、
`expected_value` に `hash_field_for_kind:<kind>` または `sha256:<lower-hex>`、
`actual_value` に `missing`、`wrong_type`、`null_not_allowed`、`unknown_field`、
または `invalid_hash_format` を入れます。
`hashes` object 内で複数 failure が同時に成立する場合は、
artifact kind ごとの required hash field order を先に評価し、次に unknown field を
bytewise lexicographic order で評価します。
artifact kind ごとの required hash field order は下の direct source input parsed hash mismatch table と同じです。
required hash が `none` の kind では required hash field order は空であり、
non-empty `hashes` object は priority 16 の `field = "inputs[<i>].hashes"` で報告します。
`kind` は Phase 8 MVP の release bundle artifact kind です。
release bundle artifact kind ではない文字列は phase-kind validation ではなく kind enum violation として扱い、
`field = "inputs[<i>].kind"`、`expected_value = "release_bundle_artifact_kind"`、
`actual_value = "invalid_enum"` とします。
kind enum violation が複数ある場合は `inputs[]` の小さい index を先に報告します。
phase-kind cardinality validation は、すべての `inputs[].kind` が release bundle artifact kind として valid な場合だけ行います。
さらに、すべての `inputs[]` entry が order / duplicate validation を通過した後にだけ行います。

phase ごとの allowed `inputs[].kind` は次で固定します。

```text
store phase:
  required, one or more each:
    - request_store_manifest
    - machine_result_store_manifest
    - normalized_result_store_manifest
  required, exactly one:
    - challenge_output_store_manifest
  forbidden:
    - all other artifact kinds

final phase:
  allowed:
    - release_policy
    - runner_policy
    - checker_identity_manifest
    - import_lock
    - challenge_manifest
    - challenge_replay_result
    - challenge_coverage_summary
    - auxiliary_result
    - ai_audit_input_policy
    - ai_audit_sidecar
    - audit_sidecar_validation_response
    - compare_validation_response
  forbidden:
    - request_store_manifest
    - machine_result_store_manifest
    - normalized_result_store_manifest
    - challenge_output_store_manifest
    - machine_check_request
    - machine_check_result
    - normalized_check_result
```

`machine_check_request_error_result` と `normalize_error_result` は release bundle artifact kind ではないため、
phase-specific forbidden kind ではなく上記の kind enum violation として拒否します。

store phase の kind cardinality violation は `input_schema_invalid` です。
required source store manifest kind が0件の場合は `field = "inputs[]"`、
`expected_value = "one_or_more:<kind>"`、`actual_value = "missing:<kind>"` とします。
`challenge_output_store_manifest` が0件の場合は `field = "inputs[]"`、
`expected_value = "exactly_one:challenge_output_store_manifest"`、
`actual_value = "missing:challenge_output_store_manifest"` とします。
`challenge_output_store_manifest` が2件以上の場合は `field = "inputs[]"`、
`expected_value = "exactly_one:challenge_output_store_manifest"`、
`actual_value = "count:<n>"` とします。
store phase または final phase で forbidden kind が出た場合は `field = "inputs[<i>].kind"`、
`expected_value = "allowed_kind_for_phase:<store|final>"`、`actual_value = "<kind>"` とします。
store phase で複数の phase-kind violation がある場合は、required source store manifest kind の不足を
allowed kind table の順に報告し、次に `challenge_output_store_manifest` の不足または重複、
最後に forbidden kind を `inputs[]` の小さい index 順で報告します。
final phase で複数の forbidden kind がある場合は、`inputs[]` の小さい index を先に報告します。
final phase の allowed kind table は explicit input ごとの allowlist です。
final phase staging command は各 explicit input の kind / path / file_hash / schema / parsed hash だけを検査し、
release bundle の closed set completeness、kind ごとの必須 cardinality、または余剰 artifact の有無は検査しません。
これらは `npa-check release bundle` と release bundle validator が、
explicit bundle-local store manifest と `ReleaseAuditBundleManifest` を使って検査します。

store phase では `request_store_manifest`、`machine_result_store_manifest`、
`normalized_result_store_manifest` の source store manifest input を1件以上受け取り、
その entries が参照する artifact files を検証して stage します。
store phase plan の source store manifest input では、`inputs[].path` は上流 pipeline が作った
source store manifest file を指します。
この input の `file_hash` と `hashes.manifest_hash` は source store manifest file bytes の SHA-256 であり、
store phase が生成する bundle-local store manifest の hash ではありません。
生成後の bundle-local store manifest path / hash は `ReleaseBundleStagingResult.store_manifests[]` だけに出します。
source store manifest entry の `path` は、その store manifest schema の workspace-relative path です。
staging command は source store manifest file の親 directory からではなく、invocation cwd から1回だけ
entry path を解決します。
store phase では release / high-trust coverage summary generation に使う
target-scoped `challenge_output_store_manifest` も stage します。
`release stage-bundle-inputs` は coverage target identity を入力に持たないため、
`challenge_output_store_manifest` が target-scoped であることを証明しません。
store phase では explicit input file の file hash、JSON / schema / order / unique key、
および entry の manifest hash だけを検査して stage します。
target scope の検査は `npa-check challenge coverage-summary` と release bundle validation の責務です。
store phase の plan に個々の request / machine result / normalized result source artifact を
重複して列挙してはいけません。これらは source store manifest entries からだけ導出します。
final phase では coverage summary を含む非 store explicit input artifacts を stage します。
final phase では `request_store_manifest`、`machine_result_store_manifest`、
`normalized_result_store_manifest` input は forbidden です。
`bundle_root` は `--bundle-root` と bytewise に一致しなければなりません。

staging command は、release bundle に含める artifact file を
`bundle-root/artifacts/<kind>/<file_hash_without_sha256_prefix>.json` に copy します。
staging command は target path の parent directories を必要に応じて作成します。
staged artifact target の parent directory 作成失敗は `output_write_failure`、
generated bundle-local store manifest target の parent directory 作成失敗は
`output_store_write_failure` です。
target path の途中 component が file の場合、または final target path が既存 directory の場合は
`output_path_conflict` です。
request / machine result / normalized result については、その bundle-root-relative path を
store manifest entry の `path` に書いた bundle-local store manifest も作ります。
bundle-local store manifest は manifest bytes を生成してから `manifest_hash = sha256(file bytes)` を計算し、
`bundle-root/artifacts/<manifest_kind>/<manifest_hash_without_sha256_prefix>.json` に atomic write します。
manifest kind はそれぞれ `request_store_manifest`、`machine_result_store_manifest`、
`normalized_result_store_manifest` です。
generated store manifest artifact entry の `file_hash` と `hashes.manifest_hash` は同じ
manifest file bytes sha256 です。
通常 check 用 store と challenge replay 用 store が pipeline 上で別 manifest として作られていた場合、
store phase で同じ kind ごとに1つの bundle-local manifest へ deterministic merge します。
staging command は各 store schema の sort order と unique key rule で書き出します。
merge input に同じ unique key の entry が複数ある場合、parsed artifact hash と source file bytes が
完全一致する entry に限り、merge failure にせず1 entry へ deduplicate しなければなりません。
同じ unique key で parsed artifact hash または source file bytes が1つでも異なる場合は
merge failure であり、その manifest を release bundle command に渡してはいけません。
`npa-check release bundle` は各 kind につき事前 staging / merge 済み manifest を1つだけ受け取り、
複数 store manifest 入力、directory scan、glob、bundle-root からの暗黙 merge は行いません。
validator は bundle-local manifest の entry を merge / deduplicate してはいけません。
bundle-local manifest 内に exact duplicate entry、同じ unique key の entry、または
同じ `path` の entry が複数ある場合は、store manifest schema / domain failure として bundle invalid です。
同じ bundle-local `path` が別 unique key に割り当てられている場合も同様に bundle generation failure /
bundle invalid とします。
上流 pipeline の manifest file をそのまま含めてよいのは、その manifest が上記 bundle-local path rule を
すでに満たし、同じ release audit bundle に含まれる対応 artifact file だけを完全に覆う場合だけです。
通常用 manifest と challenge 用 manifest を同じ artifact kind で2件含めてはいけません。
各 store manifest の entry は、同じ release audit bundle に含まれる対応 artifact file を
すべて含み、bundle 外の file を参照してはいけません。
staging command は staged artifact file と generated bundle-local store manifest を temporary file として作り、
final target path に atomic replace します。
temporary file は final target path と同じ directory に置き、同じ filesystem 上の rename / replace だけを
atomic commit として扱います。
temporary file path は response や manifest に記録せず、semantic identity ではありません。
target path が既に存在し、bytes が今回書く bytes と完全一致する場合は exact-match adoption として扱います。
target path が既に存在し、bytes が異なる場合は `CommandError.reason_code = output_path_conflict` です。
store phase の commit point は generated bundle-local store manifest files の配置完了です。
final phase の commit point は requested staged artifact files すべての配置完了です。
failure 前に残った orphan staged files は、後続の `release bundle` が manifest で参照するまで無視します。
`release stage-bundle-inputs` の `CommandError.reason_code` は次で固定します。

```text
plan path schema violation:
  reason_code = input_reference_invalid
  field = "plan.path"
  expected_value = "workspace_relative_path"
  actual_value = "invalid_path"

missing required CLI flag / duplicate singleton flag / unsupported flag / invalid --phase enum:
  CLI argument validation error
  no CommandError body

plan hash invalid format:
  reason_code = input_reference_invalid
  field = "plan_hash"
  expected_value = "sha256:<lower-hex>"
  actual_value = "invalid_hash_format"

plan file unreadable:
  reason_code = input_file_unreadable
  field = "plan.path"

plan hash mismatch:
  reason_code = input_hash_mismatch
  field = "plan_hash"

plan invalid JSON:
  reason_code = input_json_invalid
  field = "plan.path"

plan schema / order / duplicate / path violation:
  reason_code = input_schema_invalid
  field = invalid ReleaseBundleStagingPlan JSON path
  expected_value = fixed schema requirement from ReleaseBundleStagingPlan rules
  actual_value = missing | wrong_type | null_not_allowed | unknown_field |
                 invalid_enum | invalid_path | invalid_hash_format |
                 duplicate_field | order_violation | duplicate_entry |
                 duplicate_path | non_empty | command-specific actual value

input kind enum violation:
  reason_code = input_schema_invalid
  field = "inputs[<i>].kind"
  expected_value = "release_bundle_artifact_kind"
  actual_value = "invalid_enum"

--bundle-root path schema violation:
  reason_code = input_reference_invalid
  field = "bundle_root"
  expected_value = "workspace_relative_path"
  actual_value = "invalid_path"

plan bundle_root path schema violation:
  reason_code = input_schema_invalid
  field = "bundle_root"
  expected_value = "workspace_relative_path"
  actual_value = "invalid_path"

bundle_root / --bundle-root mismatch:
  reason_code = input_schema_invalid
  field = "bundle_root"
  expected_value = "--bundle-root:<value>"
  actual_value = "plan.bundle_root:<value>"

store phase missing required source store manifest kind:
  reason_code = input_schema_invalid
  field = "inputs[]"
  expected_value = "one_or_more:<kind>"
  actual_value = "missing:<kind>"

store phase missing challenge output store manifest:
  reason_code = input_schema_invalid
  field = "inputs[]"
  expected_value = "exactly_one:challenge_output_store_manifest"
  actual_value = "missing:challenge_output_store_manifest"

store phase duplicate challenge output store manifest:
  reason_code = input_schema_invalid
  field = "inputs[]"
  expected_value = "exactly_one:challenge_output_store_manifest"
  actual_value = "count:<n>"

forbidden input kind for phase:
  reason_code = input_schema_invalid
  field = "inputs[<i>].kind"
  expected_value = "allowed_kind_for_phase:<store|final>"
  actual_value = "<kind>"

direct source input file unreadable:
  reason_code = input_file_unreadable
  field = "inputs[<i>].path"

direct source input file_hash mismatch:
  reason_code = input_hash_mismatch
  field = "inputs[<i>].file_hash"

direct source input JSON invalid:
  reason_code = input_json_invalid
  field = "inputs[<i>].path"

direct source input schema violation:
  reason_code = input_schema_invalid
  field = "inputs[<i>].artifact.<json_path>"

direct source input parsed hash mismatch:
  reason_code = input_hash_mismatch
  field = "inputs[<i>].hashes.<hash_field>"
  expected_hash = inputs[<i>].hashes.<hash_field>
  actual_hash = recomputed parsed artifact hash or manifest file bytes sha256

source store manifest JSON invalid:
  reason_code = input_store_manifest_invalid
  field = "inputs[<i>].path"
  actual_value = "invalid_json"

source store manifest schema / order / duplicate violation:
  reason_code = input_store_manifest_invalid
  field = "inputs[<i>].store.<source_store_json_path>"
  expected_value = store manifest schema requirement
  actual_value = missing | wrong_type | unknown_field | invalid_hash_format |
                 invalid_path |
                 null_not_allowed | order_violation | duplicate_field |
                 duplicate_request_hash | duplicate_run_artifact_hash |
                 duplicate_normalized_result_hash | duplicate_path

source store manifest entry unreadable / invalid JSON / schema violation / file_hash mismatch / parsed hash mismatch:
  reason_code = input_store_entry_invalid
  field = one of the fixed input_store_entry_invalid field shapes below

bundle-local merge conflict:
  reason_code = release_bundle_generation_failed
  field = "inputs[]"
  actual_value = "release_bundle_generation_failed"

target path conflict:
  reason_code = output_path_conflict
  field = bundle-root-relative target path

staged artifact copy / write failure:
  reason_code = output_write_failure
  field = bundle-root-relative target path

generated bundle-local store manifest write failure:
  reason_code = output_store_write_failure
  field = bundle-root-relative manifest target path
```

`command-specific actual value` は literal string ではありません。
上の dedicated field shape が固定する実値であり、top-level `schema` mismatch の実際の schema 文字列、
`plan.phase:<store|final>`、`plan.bundle_root:<value>`、`missing:<kind>`、
`count:<n>`、または forbidden input kind の実際の `<kind>` などを指します。
`inputs[<i>]` は `ReleaseBundleStagingPlan.inputs[]` の 0-based index です。
`<source_store_json_path>` は source store kind prefix を含む manifest field path です。
source store kind ごとの prefix mapping は次で固定します。

```text
request_store_manifest:
  source_store_json_path = "request_store.<manifest_local_path>"
machine_result_store_manifest:
  source_store_json_path = "machine_result_store.<manifest_local_path>"
normalized_result_store_manifest:
  source_store_json_path = "normalized_result_store.<manifest_local_path>"
```

たとえば manifest-local path が `schema` の場合は
`request_store.schema`、`machine_result_store.schema`、または
`normalized_result_store.schema` です。
manifest-local path が `requests[<j>].path` / `results[<j>].path` の場合は、
`request_store.requests[<j>].path`、
`machine_result_store.results[<j>].path`、または
`normalized_result_store.results[<j>].path` です。
top-level object 自体の wrong type / null など root-level failure では、
`request_store`、`machine_result_store`、または `normalized_result_store` を使います。
top-level member の duplicate key では、duplicated member を prefix 付きで報告します。
たとえば duplicate `schema` は `request_store.schema`、
`machine_result_store.schema`、または `normalized_result_store.schema` です。
`<json_path>` は対象 JSON object の root からの path で、
array element は 0-based index で表します。
source store manifest file 自体の read failure は `input_file_unreadable` で
`field = "inputs[<i>].path"`、`file_hash` mismatch は `input_hash_mismatch` で
`field = "inputs[<i>].file_hash"`、JSON parse failure は `input_store_manifest_invalid` で
`field = "inputs[<i>].path"` とします。
direct source input の artifact schema violation では、`<json_path>` は source artifact JSON の root からの
path です。
artifact validator が command-local ではない virtual root prefix を持つ field shape を定義している場合、
`release stage-bundle-inputs` はその virtual root を `inputs[<i>].artifact` に置換します。
MVP の direct source input で置換する virtual root prefix は次で固定します。

```text
kind = ai_audit_input_policy:
  input_policy
    -> inputs[<i>].artifact
  input_policy.schema
    -> inputs[<i>].artifact.schema
  input_policy.included_fields[<j>]
    -> inputs[<i>].artifact.included_fields[<j>]

kind = checker_identity_manifest:
  checker_identity_manifest
  checker_identity_manifest.$
    -> inputs[<i>].artifact
  checker_identity_manifest.schema
    -> inputs[<i>].artifact.schema
  checker_identity_manifest.checkers[<j>].profile
    -> inputs[<i>].artifact.checkers[<j>].profile

kind = import_lock:
  imports.manifest
    -> inputs[<i>].artifact
  imports.manifest.imports[<j>].module
    -> inputs[<i>].artifact.imports[<j>].module
  imports.manifest.imports[<j>].certificate.path
    -> inputs[<i>].artifact.imports[<j>].certificate.path
```

この表にない artifact validator の field shape は、artifact-local JSON path をそのまま
`inputs[<i>].artifact.<json_path>` に入れます。
この prefix 置換は `field` だけに適用し、`expected_value` / `actual_value` は
元の artifact validator が定義する固定値をそのまま使います。
direct source input parsed hash mismatch の `<hash_field>` は、`inputs[<i>].kind` ごとに次の順で検査し、
最初に mismatch した field を返します。
ただし phase-kind validation は direct source input validation より先に行うため、
`machine_check_request`、`machine_check_result`、`normalized_check_result` の direct source input parsed hash validation は
Phase 8 MVP の `store` / `final` phase では到達不能です。
下の unreachable kind は artifact kind ごとの hash 定義との対応を示すだけで、
実装は phase-kind validation 通過後の allowed kind だけを direct source input として hash validation します。
`machine_check_request_error_result` と `normalize_error_result` は pipeline error artifact であり、
Phase 8 MVP の release bundle artifact kind ではないため、この hash field table には含めず、
direct source input parsed hash validation にも到達しません。

```text
release_policy:
  - policy_hash
machine_check_request:
  - request_hash
machine_check_result:
  - result_hash
  - run_artifact_hash
normalized_check_result:
  - artifact_hash
  - normalized_result_hash
auxiliary_result:
  - result_hash
challenge_manifest:
  - manifest_hash
challenge_output_store_manifest:
  - manifest_hash
challenge_replay_result:
  - result_hash
challenge_coverage_summary:
  - summary_hash
ai_audit_input_policy:
  - input_policy_hash
runner_policy:
  - policy_hash
checker_identity_manifest:
  - manifest_hash
import_lock:
  - manifest_hash
request_store_manifest:
  - manifest_hash
machine_result_store_manifest:
  - manifest_hash
normalized_result_store_manifest:
  - manifest_hash
```

`ai_audit_sidecar`、`compare_validation_response`、`audit_sidecar_validation_response` は
hash field を持たないため、`hashes` は required empty object です。
これらの kind で non-empty `hashes` が存在する場合は parsed hash mismatch ではなく
plan schema violation として `input_schema_invalid` を返します。
manifest kind の `actual_hash` は referenced manifest file bytes sha256 です。
それ以外の kind の `actual_hash` は parsed artifact から再計算した canonical hash です。

`input_store_entry_invalid` の field shape は次で固定します。

```text
source store entry file unreadable:
  field = "inputs[<i>].store.<store_entry_path>.path"
  actual_value = "unreadable"

source store entry invalid JSON:
  field = "inputs[<i>].store.<store_entry_path>.path"
  actual_value = "invalid_json"

source store entry schema violation:
  field = "inputs[<i>].store.<store_entry_path>.artifact.<json_path>"
  expected_value = artifact schema requirement
  actual_value = missing | wrong_type | unknown_field | invalid_enum |
                 invalid_path | invalid_hash_format | invalid_name_format | null_not_allowed |
                 order_violation | duplicate_field | failure_key_mismatch

source store entry file_hash mismatch:
  field = "inputs[<i>].store.<store_entry_path>.file_hash"
  expected_hash = store entry file_hash
  actual_hash = referenced file bytes sha256

source store entry artifact self-hash mismatch:
  field = "inputs[<i>].store.<store_entry_path>.artifact.<hash_field>"
  expected_hash = recomputed parsed artifact hash
  actual_hash = parsed artifact field value

source store entry manifest-field mismatch:
  field = "inputs[<i>].store.<store_entry_path>.<hash_or_profile_field>"
  expected_hash / expected_value = store entry value
  actual_hash / actual_value = parsed artifact value
```

`<store_entry_path>` は source store kind に応じて
`request_store.requests[<j>]`、`machine_result_store.results[<j>]`、
または `normalized_result_store.results[<j>]` です。
`<j>` は source store manifest 内 entry array の 0-based index です。
`input_store_manifest_invalid` で許可する unique key duplicate reason は source store kind ごとに固定します。

```text
request_store_manifest:
  - duplicate_request_hash
  - duplicate_path

machine_result_store_manifest:
  - duplicate_run_artifact_hash
  - duplicate_path

normalized_result_store_manifest:
  - duplicate_normalized_result_hash
  - duplicate_path
```

unique key duplicate reason の field は、重複 key の caller-prefixed manifest path に固定します。
`duplicate_request_hash` は
`inputs[<i>].store.request_store.requests[<j>].request_hash`、
`duplicate_run_artifact_hash` は
`inputs[<i>].store.machine_result_store.results[<j>].run_artifact_hash`、
`duplicate_normalized_result_hash` は
`inputs[<i>].store.normalized_result_store.results[<j>].normalized_result_hash` です。
`duplicate_path` は source store kind ごとの concrete manifest path に固定し、
`inputs[<i>].store.request_store.requests[<j>].path`、
`inputs[<i>].store.machine_result_store.results[<j>].path`、または
`inputs[<i>].store.normalized_result_store.results[<j>].path` です。

source store entry artifact self-hash の検査順は次で固定します。

```text
self-hash validation order:
  request_store.requests[]:
    - request_hash
  machine_result_store.results[]:
    - result_hash
    - run_artifact_hash
  normalized_result_store.results[]:
    - artifact_hash
    - normalized_result_hash
```

source store entry manifest-field comparison order は次で固定します。

```text
manifest-field comparison order:
  request_store.requests[]:
    - request_hash
  machine_result_store.results[]:
    - result_hash
    - request_hash
    - run_artifact_hash
    - checker_profile
  normalized_result_store.results[]:
    - artifact_hash
    - normalized_result_hash
```

artifact self-hash mismatch の `<hash_field>` は self-hash validation order の field 名だけを使い、
manifest-field mismatch の `<hash_or_profile_field>` は manifest-field comparison order の
field 名だけを使います。
複数の mismatch がある場合は、各 order で最初の field を返します。
manifest-field mismatch の hash field では `expected_hash` に store entry の hash、
`actual_hash` に parsed artifact の同じ field の hash を入れます。
`checker_profile` mismatch では `expected_value` に store entry の `checker_profile`、
`actual_value` に parsed `MachineCheckResult.checker.profile` を入れます。

entry validation の内部優先順位は unreadable、invalid JSON、schema violation、file_hash mismatch、
artifact self-hash mismatch、manifest-field mismatch です。
source store entry validation で複数 entry が同時に失敗する場合は、まず
`ReleaseBundleStagingPlan.inputs[]` の小さい index の source store manifest input を選び、
同じ source store manifest 内では entry array の小さい index を選びます。
同じ entry 内で複数 failure が成立する場合だけ、上記の内部優先順位を使います。
複数の失敗条件が同時にある場合は、CLI argument validation、
plan / `--bundle-root` / `--plan-hash` reference validation、plan file、plan hash mismatch、
plan JSON / schema、bundle_root mismatch、phase-kind validation、
direct source input validation、source store manifest validation、source store entry validation、
merge conflict、target path conflict、write failure の順で最初に該当した `reason_code` を返します。
plan / `--bundle-root` / `--plan-hash` reference validation 内で複数 failure が同時に成立する場合は、
`--plan` path schema violation、`--bundle-root` path schema violation、
`--plan-hash` invalid hash format の順で最初の failure を返します。
phase-kind validation で複数 input が同時に失敗する場合は、`inputs[]` の小さい index を選びます。
direct source input validation で複数 input が同時に失敗する場合も、`inputs[]` の小さい index を選びます。
同じ direct source input 内では unreadable、file_hash mismatch、invalid JSON、artifact schema violation、
parsed hash mismatch の順で最初の failure を返します。
source store manifest validation で複数 source store manifest input が同時に失敗する場合は、
`inputs[]` の小さい index を選びます。
同じ source store manifest input 内では unreadable、file_hash mismatch、invalid JSON、
schema / order / duplicate violation の順で最初の failure を返します。
merge conflict、target path conflict、write failure で複数 target が同時に失敗する場合は、
`ReleaseBundleStagingPlan.inputs[]` の小さい index から導出された target を先にし、
generated bundle-local store manifest target 同士では manifest kind の bytewise lexicographic order で
最初の failure を返します。
`--json` 成功時 stdout は `ReleaseBundleStagingResult` です。
MVP の `ReleaseBundleStagingResult` は `schema = npa.phase8.release_bundle_staging_result.v1`、
`phase`、`bundle_root`、`staged_artifacts[]`、`store_manifests[]` を持つ transient response で、
`result_hash` を持ちません。
`staged_artifacts[]` は `(kind, path, file_hash)`、`store_manifests[]` は
`(kind, path, manifest_hash)` を持ち、どちらの `path` も bundle-root-relative path です。
`store_manifests[].manifest_hash` は referenced manifest file bytes sha256 で、
別名の `file_hash` field は持たせません。
`staged_artifacts[]` は `(kind, path, file_hash)` の bytewise lexicographic order、
`store_manifests[]` は `(kind, path, manifest_hash)` の bytewise lexicographic order で昇順に並べます。
どちらの array でも、同じ `(kind, path)` を持つ entry は hash が同じ場合にも異なる場合にも
forbidden です。さらに、同じ `path` を持つ entry も forbidden です。
`staged_artifacts[]` は generated bundle-local store manifest files を含めません。
`store_manifests[]` には generated bundle-local store manifest だけを入れ、
store phase input の source store manifest は入れません。
store phase では `store_manifests[]` に generated `request_store_manifest`、
`machine_result_store_manifest`、`normalized_result_store_manifest` をちょうど1件ずつ入れます。
final phase では `store_manifests[]` は required empty array です。
`ReleaseBundleStagingResult` は transient response であり、release bundle validator の入力ではありません。
release bundle command / validator は、bundle に含まれる
`ReleaseAuditBundleManifest.artifacts` の `request_store_manifest`、
`machine_result_store_manifest`、`normalized_result_store_manifest` entry と、対応する explicit path/hash flag で
渡された bundle-local store manifest を使って normalize / challenge replay /
audit-sidecar validation を再実行します。
store manifest が不足、重複、order violation、hash mismatch、bundle 外参照を含む場合は
bundle invalid です。
通常 certificate check の `MachineCheckRequest.policy.hash`、
`MachineCheckResult.policy.hash`、`NormalizedCheckResult.policy.hash` は
`ReleasePolicy.runner_policy_hash` と一致しなければなりません。
challenge replay の `ChallengeReplayResult.policy_hash` と
`ChallengeCoverageSummary.policy_hash` は
`ReleasePolicy.challenge_runner_policy_hash` と一致しなければなりません。
included `ChallengeManifest.policy_hash` も
`ReleasePolicy.challenge_runner_policy_hash` と一致しなければなりません。
`ChallengeReplayResult.checker_results[*].run_artifact_hash` から解決される
underlying `MachineCheckResult.policy.hash` と、その result の
`MachineCheckRequest.policy.hash` も `ReleasePolicy.challenge_runner_policy_hash`
と一致しなければなりません。
`ChallengeReplayResult.normalized_result_hash` が存在する場合、
参照先 `NormalizedCheckResult.policy.hash` も
`ReleasePolicy.challenge_runner_policy_hash` と一致しなければなりません。
required checker profile の再計算は、この included `runner_policy` artifact を parse して行い、
bundle 外の policy store には fallback しません。
MVP の `artifact_hash` は単一 release target の artifact identity です。
通常の certificate release bundle では、同じ target module に対する
`NormalizedCheckResult.artifact_hash` と一致しなければなりません。
MVP の `ReleaseAuditBundleManifest` は release target を持つ bundle だけを表します。
そのため `artifacts` には、この top-level `artifact_hash` と一致する
`kind = normalized_check_result` entry がちょうど1件必要です。
この entry が release target の `NormalizedCheckResult` です。
さらに、included `ChallengeReplayResult.normalized_result_hash` が存在する場合は、
同じ bundle 内にその hash と一致する `kind = normalized_check_result` entry が
ちょうど1件必要です。
`ChallengeReplayResult.normalized_result_hash` は release target
`NormalizedCheckResult.normalized_result_hash` と bytewise distinct でなければなりません。
同じ hash を指す場合、release target entry と challenge replay entry を deduplicate してはいけません。
この collision は `challenge_replay_result` class 5 source-key failure として扱い、
challenge replay 用 `normalized_check_result` expected set を導出してはいけません。
一方、複数の included `ChallengeReplayResult` が同じ non-release
`normalized_result_hash` を指すことは許可します。
challenge replay 用 `normalized_check_result` expected set は distinct
`ChallengeReplayResult.normalized_result_hash` set から導出し、同じ hash を指す複数 replay result は
1件の `normalized_check_result` entry を共有してよいです。
この共有自体は duplicate failure ではありません。
ただし各 replay result の `comparison_status` は、その共有 normalized result の再計算済み
comparison.status と個別に一致しなければなりません。
ただし challenge replay 用 `normalized_check_result` expected set の導出前には、
13 の prerequisite gate の `normalized_check_result` requires order を適用します。
`challenge_coverage_summary for normalized_check_result` が prerequisite-clean でない場合は、
その `challenge_coverage_summary` failure を先に返し、`challenge_replay_result` set も
`ChallengeReplayResult.normalized_result_hash` も評価してはいけません。
`challenge_coverage_summary for normalized_check_result` が prerequisite-clean で、
`challenge_replay_result set for normalized_check_result` が prerequisite-clean でない場合だけ、
`challenge_replay_result` の failure を返します。
challenge replay 用 `NormalizedCheckResult` は top-level `artifact_hash` と
一致する release target entry ではなく、mutated certificate 側の artifact identity を表します。
challenge replay 用 entry の `policy.hash` は `ReleasePolicy.challenge_runner_policy_hash` と
一致しなければなりません。
release target の `normalized_check_result` 以外に、included `ChallengeReplayResult` から
参照されない `normalized_check_result` entry を含めてはいけません。
`MachineCheckRequestErrorResult` や `NormalizeErrorResult` だけからなる
failure-only bundle は Phase 8 MVP では `ReleaseAuditBundleManifest` として materialize しません。
`ReleaseAuditBundleManifest` は pass artifact なので、`kind = machine_check_request_error_result` と
`kind = normalize_error_result` entry は Phase 8 MVP では forbidden です。
そのような失敗 provenance は release bundle の外に保存するか、将来の postmortem manifest で扱います。
release bundle に複数 module / 複数 target artifact を入れる場合は、
MVP では target ごとに別の `ReleaseAuditBundleManifest` を作ります。
複数 target を1つの bundle hash identity にまとめる rollup manifest は Phase 8 MVP では定義しません。
`artifacts` は `(kind, path)` の bytewise lexicographic order で昇順に並べます。
`(kind, path)` と `path` はどちらも unique です。
`ReleaseAuditBundleManifest.artifacts[].path` は `bundle-root` relative path です。
absolute path、empty segment、`.` / `..` segment、control character は forbidden です。
pre-bundle staging step と `npa-check release bundle` が使う artifact path は
`artifacts/<kind>/<file_hash_without_sha256_prefix>.json` に固定します。
同じ `kind` かつ同じ `file_hash` の source artifact が複数回入力された場合、
staging command は1つの staged file に deduplicate し、release bundle でも1つの artifact entry で
すべての closed-set reference を満たしてよいです。
ただし caller が同じ source bytes に対して互換しない expected hash を指定した場合は
検出段階で reason を固定します。
個々の direct source input の `hashes`、または release bundle command の parsed-hash explicit flag が
parsed artifact hash と一致しない場合は `input_hash_mismatch` です。
release bundle command の file-bytes-hash explicit flag が referenced file bytes sha256 と一致しない場合も
`input_hash_mismatch` です。
すべての explicit input hash validation が通った後、closed-set / cross-artifact rule 上で
同じ staged artifact identity に互換しない parsed hash identity を要求していることが分かった場合だけ
`release_bundle_generation_failed` です。
validator は bundle manifest file が置かれた directory を `bundle-root` とみなし、
artifact path をその directory から相対解決します。
workspace root、current directory、original pipeline path、または environment variable へ fallback してはいけません。
`file_hash` は referenced file bytes の sha256 です。
`hashes` は artifact kind ごとの parsed artifact hash を入れます。
ただし manifest kind の `manifest_hash` は、その referenced manifest file bytes の sha256 です。
MVP の manifest schema は self hash field を持ちません。
artifact entry の top-level field は `kind`、`path`、`file_hash`、`hashes`、
および kind ごとに明示された追加 field だけです。
unknown field は bundle invalid です。
`target_artifact_hash` は Phase 8 MVP の `ReleaseAuditBundleManifest` では forbidden です。
将来の postmortem manifest が `machine_check_request_error_result` や
`normalize_error_result` を扱う場合だけ、同名 field を改めて定義します。

MVP の artifact kind ごとの `hashes`：

```text
release_policy:
  required: policy_hash

machine_check_request:
  required: request_hash

machine_check_result:
  required: result_hash, run_artifact_hash

normalized_check_result:
  required: normalized_result_hash, artifact_hash

auxiliary_result:
  required: result_hash

challenge_manifest:
  required: manifest_hash

challenge_output_store_manifest:
  required: manifest_hash

challenge_replay_result:
  required: result_hash

challenge_coverage_summary:
  required: summary_hash

ai_audit_input_policy:
  required: input_policy_hash

ai_audit_sidecar:
  required: none

compare_validation_response:
  required: none

audit_sidecar_validation_response:
  required: none

runner_policy:
  required: policy_hash

checker_identity_manifest:
  required: manifest_hash

import_lock:
  required: manifest_hash

request_store_manifest:
  required: manifest_hash

machine_result_store_manifest:
  required: manifest_hash

normalized_result_store_manifest:
  required: manifest_hash
```

`ai_audit_sidecar`、`compare_validation_response`、`audit_sidecar_validation_response` は
保存正本 artifact hash を持たないため、`file_hash` だけを検査します。
validation response は transient response なので、bundle に含める場合も
checker verdict identity や CI pass identity には使いません。
`auxiliary_result` は保存済み deterministic 補助結果として検査します。
`ReleaseAuditBundleManifest` に含める `auxiliary_result` entry は、
included `ReleasePolicy.mode` から決まる closed set です。
`ReleasePolicy.mode = release` では次が required です。

```text
- exactly one kind = axiom_policy entry
  - policy_hash = ReleasePolicy.runner_policy_hash
  - artifact_hash = release target NormalizedCheckResult.artifact_hash
  - selector.normalized_result_hash = release target NormalizedCheckResult.normalized_result_hash
  - selector.checker_profile = runner RunnerPolicy.required_checker_profiles[0]
  - selector.result_hash and selector.axiom_report_hash match that release target result entry
  - status = passed

- exactly one kind = reproducibility entry
  - policy_hash = ReleasePolicy.runner_policy_hash
  - artifact_hash = release target NormalizedCheckResult.artifact_hash
  - selector.request_hash and selector.checker_profile match the release target result entry
    for runner RunnerPolicy.required_checker_profiles[0]
  - selector.baseline_run_artifact_hash and selector.repeated_run_artifact_hash
    are syntactically valid, extractable, and bytewise distinct
  - status = passed
```

上の `reproducibility` required entry は auxiliary_result closed set / prerequisite-clean 判定の
identity requirement です。
`selector.baseline_run_artifact_hash` / `selector.repeated_run_artifact_hash` が参照する
`MachineCheckResult` の存在は、この段階では要求しません。
valid / extractable な selector run artifact hash は下流の expected `machine_check_result` identity になり、
missing は `machine_check_result` class 4 failure として報告します。
参照先 `MachineCheckResult` が存在した後にだけ、
request_hash / checker_profile / result_hash equality を Step 8 class 5 として検査します。

`ReleasePolicy.mode = high-trust` では上の release requirements に加えて、
この節で定義した import lock hash 集合の distinct hash ごとに次が required です。

```text
- exactly one kind = import_certificate_hash entry
  - policy_hash = ReleaseAuditBundleManifest.policy_hash
  - artifact_hash = matching import_lock entry hashes.manifest_hash
  - status = passed
```

import lock hash 集合が空の場合、`import_certificate_hash` entry は0件でなければなりません。
`import_lock` set が 13 の prerequisite gate で定義する prerequisite-clean でない場合、
bundle generator / validator は high-trust の `import_certificate_hash`
expected set を導出してはいけません。
この場合は `import_lock` の failure を先に返します。
`ReleasePolicy.mode = release` では `import_certificate_hash` entry は forbidden です。
MVP の `ReleaseAuditBundleManifest` は `ReleasePolicy.mode = nightly` では materialize しません。
nightly policy を含む `ReleaseAuditBundleManifest` は bundle invalid です。
上記 closed set 以外の `auxiliary_result` kind、重複 entry、missing entry は
`auxiliary_result` class 4 closed-set failure です。
required slot に entry がちょうど1件あり、その entry の `status != passed` の場合は
`auxiliary_result` class 5 status mismatch として bundle invalid です。
`selector` の required / forbidden rule、unknown field、hash format、profile value、
および required slot に対する selector identity mismatch も bundle invalid です。
`axiom_policy` では、selector.normalized_result_hash は release target
`NormalizedCheckResult.normalized_result_hash` と比較し、selector.result_hash /
selector.axiom_report_hash は runner `RunnerPolicy.required_checker_profiles[0]` の
baseline `NormalizedCheckResult.results[*]` entry と比較します。
これらの mismatch は `auxiliary_result` class 5 として扱います。
`reproducibility` では、selector.request_hash / selector.checker_profile、
valid / extractable な selector run_artifact_hash syntax、および
baseline_run_artifact_hash と repeated_run_artifact_hash の bytewise distinctness までを
`auxiliary_result` identity として扱い、selector run_artifact_hash が指す
`MachineCheckResult` の存在はここでは要求しません。
valid / extractable な reproducibility selector run_artifact_hash の missing target は
`machine_check_result` class 4、target が存在した後の request_hash /
checker_profile / result_hash mismatch は Step 8 class 5 として扱います。
`auxiliary_result` の required set が missing / duplicate / extra、
required entry の `status != passed`、
または required selector key field を抽出できない malformed selector で invalid な場合、
bundle generator / validator はその `auxiliary_result` failure を返し、
`reproducibility` selector に依存する下流の `machine_check_result` allowed run set や
optional AI sidecar source set を推測してはいけません。
failed / inconclusive auxiliary result は CI diagnostic として bundle 外に保存してよいですが、
release audit bundle の pass artifact には含めません。
`kind = audit_bundle` の `AuxiliaryResult` は、自分自身が検査する
`ReleaseAuditBundleManifest` の中には含めません。
bundle validator は required なすべての included `auxiliary_result` について、`result_hash`、
`policy_hash`、`artifact_hash`、`status`、`error.reason_code` と kind の整合性を検査します。
Phase 8 MVP の release bundle artifact kind は、kind-specific auxiliary oracle の
全 oracle input artifact を保存しません。
たとえば axiom report artifact と imported certificate files は bundle artifact kind に含めません。
これは `AxiomReport` を Phase 8 saved artifact として保存しないという意味ではなく、
release audit bundle の included artifact closed set から除外するという意味です。
そのため bundle validator は `axiom_policy`、`reproducibility`、
`import_certificate_hash` の oracle を再実行せず、保存済み `AuxiliaryResult` envelope と
参照 hash の整合性だけを検査します。
`axiom_policy` では `policy_hash` が `ReleasePolicy.runner_policy_hash` と一致し、
`artifact_hash` が release target `NormalizedCheckResult.artifact_hash` と一致しなければなりません。
`reproducibility` でも `policy_hash` が `ReleasePolicy.runner_policy_hash` と一致し、
`artifact_hash` が release target `NormalizedCheckResult.artifact_hash` と一致しなければなりません。
`import_certificate_hash` では `artifact_hash` が included `import_lock` entry の
`hashes.manifest_hash` とちょうど1件一致しなければなりません。
これらの auxiliary oracle は release bundle generation 前の deterministic CI step で実行し、
CI pass condition はその `AuxiliaryResult.status` を使います。
bundle validator は failed / inconclusive の `AuxiliaryResult` を passed に昇格してはいけません。
included `compare_validation_response` entry は optional ですが、含める場合は
`CompareValidationResult.status = valid` だけを許可します。
`status = failed` の compare validation response は CI diagnostic として bundle 外に保存してよく、
release audit bundle には含めません。
各 included `compare_validation_response` は parsed `normalized_result_hash` で、
同じ bundle 内の `kind = normalized_check_result` entry にちょうど1件解決できなければなりません。
同じ `normalized_result_hash` に対する `compare_validation_response` entry は最大1件です。
release target の normalized result に対応する response の `policy_hash` は
`ReleasePolicy.runner_policy_hash` と一致しなければなりません。
included `ChallengeReplayResult.normalized_result_hash` から参照される challenge replay normalized result に
対応する response の `policy_hash` は `ReleasePolicy.challenge_runner_policy_hash` と一致しなければなりません。
それ以外の normalized result に対応する response は forbidden です。
bundle validator は `compare_validation_response` の有無にかかわらず、`normalized_check_result`
closed-set exactness を通過して release target または challenge replay target として分類できた
すべての allowed included `normalized_check_result` entry について、included normalized result file と
target-specific な included `RunnerPolicy` file を使って embedded `NormalizedCheckResult.comparison` を
再計算しなければなりません。
closed-set 外の extra / forbidden `normalized_check_result` entry には target-specific policy を選ばず、
comparison 再計算を行う前に `normalized_check_result` class 4 closed-set failure を返します。
再計算した comparison object が embedded comparison と canonical serialization 上で一致しない場合、
その `normalized_check_result` entry は Step 8 class 5 recomputation mismatch です。
release target の `NormalizedCheckResult.comparison.status` は、再計算済み comparison で
`all_agree_checked` でなければならず、そうでない bundle は invalid です。
この release target comparison validation failure は `normalized_check_result` class 5 です。
release target `normalized_check_result` に依存して下流 expected set を導出する場合は、
この class 5 failure を prerequisite failure として扱い、下流の missing / forbidden / duplicate failure を
合成してはいけません。
optional `compare_validation_response` が含まれる場合は、上の必須 comparison 再計算に加えて、
保存済み `CompareValidationResult` object と同じ inputs から再実行で得た object が
canonical serialization 上で一致することも検査しなければなりません。
release target の normalized result では、`ReleasePolicy.runner_policy_hash` に対応する
included `RunnerPolicy` file を使います。
challenge replay normalized result では、`ReleasePolicy.challenge_runner_policy_hash` に対応する
included `RunnerPolicy` file を使います。
対象 normalized result がこの2種類のどちらにも分類できない場合、
その `compare_validation_response` entry は forbidden です。
すべての included `ai_audit_sidecar` entry について、対応する `audit_sidecar_validation_response` entry を
parsed `AuditSidecarValidationResult.sidecar_file_hash`、`input_policy_hash`、
`source_kind`、`source_result_hash`、`source_normalized_result_hash` で特定します。
一致する response が0件または複数件ある場合は bundle invalid です。
`ReleaseAuditBundleManifest` に含まれる `ai_audit_sidecar` entry はすべて optional sidecar です。
required AI sidecar target に対応する `ai_audit_sidecar` entry は forbidden です。
bundle validator は included `ai_audit_input_policy` と included stores を使って
audit-sidecar validation を再実行し、保存済み response の parsed object が
再実行で得た `AuditSidecarValidationResult` object と canonical serialization 上で
一致することを検査しなければなりません。
さらに、すべての included `ai_audit_sidecar` entry の `AiAuditSidecar.source` は
12 の release audit bundle AI sidecar source closed set に含まれていなければなりません。
closed set 外の optional sidecar は、cross-artifact validation が valid でも bundle invalid です。
included sidecar source は artifact closed set の入力ではなく、すでに許可された artifact への説明参照です。
そのため sidecar 参照だけを理由に `machine_check_result`、`machine_check_request`、
`normalized_check_result`、`challenge_replay_result`、`import_lock` entry を追加してはいけません。
`machine_check_request` entry の `hashes.request_hash` は parsed `MachineCheckRequest.request_hash`
および 3.3 の再計算値と一致しなければなりません。
`machine_check_result` entry の parsed `MachineCheckResult.request_hash` は、
同じ `ReleaseAuditBundleManifest` 内の `kind = machine_check_request` entry に
ちょうど1件解決できなければなりません。
解決した request entry の `hashes.request_hash` と parsed request の再計算 hash は、
`MachineCheckResult.request_hash` と完全一致しなければなりません。
release bundle validator は request store manifest や外部 database を fallback として使わず、
bundle 内に対応する `MachineCheckRequest` がない `MachineCheckResult` を bundle invalid にします。
`challenge_replay_result.checker_results[*].run_artifact_hash` が参照する
`MachineCheckResult` についても同じ規則を適用します。
MVP の release audit bundle に含める `machine_check_result` entry は closed set です。
allowed run set は次の和集合です。

```text
- release target NormalizedCheckResult.results[*] から選ばれる raw result
- included ChallengeReplayResult.checker_results[*].run_artifact_hash
- required reproducibility AuxiliaryResult.selector.baseline_run_artifact_hash
- required reproducibility AuxiliaryResult.selector.repeated_run_artifact_hash
```

この allowed run set は、release target `normalized_check_result` for `machine_check_result`、
`challenge_replay_result set for machine_check_result`、
required `reproducibility` auxiliary result set が prerequisite-clean で
一意に解決できた後にだけ確定します。
`challenge_replay_result set for machine_check_result` の内部では、
`challenge_coverage_summary for challenge_replay_result` を前提として使います。
ここでの prerequisite-clean は 13 の Step 8 prerequisite gate で定義する、
下流 expected set の key material を安全に抽出できる状態です。
valid / extractable な `reproducibility` selector の `baseline_run_artifact_hash` と
`repeated_run_artifact_hash` は、どちらも expected `machine_check_result` identity になります。
ただし2つの selector run_artifact_hash が bytewise equal の場合は、
参照先 `MachineCheckResult` の有無を見ずに `auxiliary_result` class 5 selector identity mismatch とします。
その run artifact hash を持つ `machine_check_result` entry がない場合は、
`machine_check_result` の class 4 missing failure です。
対応する `machine_check_result` entry が存在した後の
request / checker_profile / result_hash equality は Step 8 class 5 で扱います。
これらの前提が prerequisite-clean でない場合は、その前提 failure を先に返し、
不足した replay result や missing / malformed `reproducibility` selector から
`machine_check_result` の missing / forbidden failure を合成してはいけません。
release target `NormalizedCheckResult.results[*]` は `run_artifact_hash` を持たないため、
bundle validator は release target raw result を次で選びます。
runner `RunnerPolicy.required_checker_profiles[0]` の result entry では、
required reproducibility selector の `baseline_run_artifact_hash` が選択 raw result です。
この artifact は release target result entry の `result_hash`、`request_hash`、
`checker_profile`、`policy_hash` と完全一致しなければなりません。
その他の release target result entry では、同じ bundle 内に
`result_hash`、`request_hash`、`checker_profile`、`policy_hash` が一致する
`machine_check_result` entry がちょうど1件存在し、それを選択 raw result とします。
非 baseline profile で同じ tuple に一致する retry result が2件以上ある場合は bundle invalid です。
この非 baseline raw result selection ambiguity は `machine_check_result` の
Step 8 class 5 identity / source-key failure であり、allowed run set を一意に導出できない
prerequisite failure として扱います。
この場合、どちらかの retry result を採用して `machine_check_result` missing / forbidden failure を
合成してはいけません。
同じ tuple に一致する entry が0件の場合は、expected artifact identity key
`("release_target_result", checker_profile, request_hash, result_hash, policy_hash)` の
missing `machine_check_result` class 4 failure です。
baseline profile の同じ tuple に一致する追加 retry result は、
`reproducibility.selector.repeated_run_artifact_hash` として参照される1件だけ許可します。
AI sidecar reference は release target raw result selection の disambiguation に使ってはいけません。
release target result entry に対応する AI sidecar は、選択 raw result の `run_artifact_hash` を
参照しなければなりません。
baseline reproducibility の repeated run を説明する optional sidecar だけは、
`reproducibility.selector.repeated_run_artifact_hash` を参照してよいです。
AI sidecar reference は allowed run set に新しい run を追加しません。
上の allowed run set に入らない `machine_check_result` entry は forbidden です。

MVP の release audit bundle に含める `machine_check_request` entry も closed set です。
含めてよい request は、included `machine_check_result` entry の parsed
`MachineCheckResult.request_hash` の distinct set にちょうど対応する request だけです。
各 distinct request hash について `kind = machine_check_request` entry がちょうど1件必要で、
追加の `machine_check_request` entry は forbidden です。
`MachineCheckRequestErrorResult` は valid な `MachineCheckRequest` を表さないため、
この closed set に request hash を追加しません。
`machine_check_request_error_result` と `normalize_error_result` は pipeline error artifact であり、
Phase 8 MVP の `ReleaseAuditBundleManifest` では forbidden です。
これらを含む manifest は bundle invalid です。
失敗 provenance は release bundle 外の diagnostic store、または将来の postmortem manifest で扱います。
`challenge_manifest` の `manifest_hash` は `ChallengeReplayResult.manifest_hash` と同じく
保存された challenge manifest file bytes の sha256 です。
MVP の release audit bundle には `kind = challenge_output_store_manifest` entry が
ちょうど1件必要です。
その `hashes.manifest_hash` は referenced `ChallengeOutputStoreManifest` file bytes sha256 と
一致しなければなりません。
bundle validator は included `ChallengeOutputStoreManifest` を parse し、
schema、sort order、unique key、各 entry の `manifest_hash` を検査します。
MVP の coverage に使う `ChallengeOutputStoreManifest` は target-scoped です。
target scope は coverage target `NormalizedCheckResult.artifact.module`、
`artifact.input_file_hash`、`artifact.expected_certificate_hash` の組で決まります。
global / multi-target challenge store を nightly coverage summary generation や
release audit bundle generation に直接使ってはいけません。
pipeline は coverage summary generation 前に、target ごとの
`ChallengeOutputStoreManifest` へ deterministic に split / filter しなければなりません。
この split / filter は release audit bundle generation 前の pipeline step です。
release audit bundle validator は original pipeline path を読まず、この split / filter を再実行しません。
bundle validation では included `challenge_output_store_manifest` をすでに filtered 済みの
bundle-local coverage universe として扱います。
filtered `ChallengeOutputStoreManifest` は source store entry の `manifest_path` を bytewise に保持します。
bundle generator は `manifest_path` を bundle-local path に書き換えてはいけません。
したがって filtered store の `manifest_hash` は、entry の削除と sort によってだけ変わり、
path rewrite では変わりません。
split / filter の入力 store は、filter 前に全 entry を検証します。
各 entry について `manifest_path` の file を読み、file bytes sha256 が entry の
`manifest_hash` と一致し、参照先 `ChallengeManifest` が manifest-local JSON / schema / domain validation を
通らなければなりません。
ここでの manifest-local domain validation は `ChallengeManifest` object 自体に閉じます。
`challenge_id` format、required / forbidden / unknown / null / duplicate field、
hash format、path format、`mutation.kind` の分類、`mutation.target` の kind 別 target grammar、
base / mutated certificate metadata の
field shape を検査しますが、`base_certificate.path`、`mutated_certificate.path`、
`imports.manifest`、policy file、import lock file などの外部 file は読みません。
参照先 manifest が unreadable、invalid JSON、schema invalid、hash mismatch、または
`mutation.kind` missing / invalid の場合、その entry を skip してはいけません。
nightly では coverage summary generation failure、release / high-trust では
release audit bundle generation failure です。
この split / filter では target scope が一致する entry のうち、
`ChallengeManifest.mutation.kind` が Phase 8 MVP の rejection-required challenge 種別であるものだけを残します。
informational challenge manifest は coverage universe から除外し、release audit bundle に含めてはいけません。
multi-target store をそのまま含む release audit bundle は bundle invalid です。
`ChallengeOutputStoreManifest.entries[].manifest_path` は original pipeline path であり、
bundle 内の `challenge_manifest` entry の `path` と一致する必要はありません。
bundle validator は `manifest_hash` だけを binding として使い、
`manifest_path` を使って bundle 外の file を読んではいけません。
`challenge_output_store_manifest` が 13 の prerequisite gate で定義する
prerequisite-clean でない場合、bundle generator / validator は
`entries[].manifest_hash` から `challenge_manifest` expected set を導出してはいけません。
この場合は `challenge_output_store_manifest` の failure を先に返します。
bundle validation では、store entry の `manifest_hash` と同じ hash を持つ included
`kind = challenge_manifest` entry の file bytes を parse して検査します。
bundle validator は included `ChallengeManifest` に対して、split / filter と同じ
manifest-local JSON / schema / domain validation を再実行します。
この検査でも外部 file は読みません。
store manifest の各 entry について、同じ bundle 内に
`kind = challenge_manifest` entry がちょうど1件存在しなければなりません。
逆に store manifest の entry から参照されない `challenge_manifest` entry は forbidden です。
参照した `ChallengeManifest.challenge_id` は
`ChallengeOutputStoreManifest.entries[].challenge_id` と一致しなければなりません。
参照した `ChallengeManifest.module`、`base_certificate.file_hash`、
`base_certificate.claimed_certificate_hash` は、それぞれ coverage target の
`NormalizedCheckResult.artifact.module`、`artifact.input_file_hash`、
`artifact.expected_certificate_hash` と一致しなければなりません。
参照した `ChallengeManifest.mutation.kind` が informational の場合、その store entry と
challenge manifest は release audit bundle では forbidden です。
target scope が一致しない store entry または challenge manifest も bundle invalid です。

MVP の release audit bundle には `kind = challenge_coverage_summary` entry が
ちょうど1件必要です。
その `hashes.summary_hash` は parsed `ChallengeCoverageSummary.summary_hash` および
`summary_id` / `summary_hash` を除いて再計算した canonical hash と一致しなければなりません。
release bundle validator / `audit_bundle` でこの identity validation が失敗する場合の
field shape と優先順位は次で固定します。

```text
artifact entry hashes.summary_hash vs parsed summary_hash mismatch:
  field = "coverage_summary.artifact.summary_hash"
  expected_hash = ReleaseAuditBundleManifest artifact entry hashes.summary_hash
  actual_hash = parsed ChallengeCoverageSummary.summary_hash

parsed summary_hash self-hash mismatch:
  field = "coverage_summary.artifact.summary_hash"
  expected_hash = recomputed ChallengeCoverageSummary summary_hash
  actual_hash = parsed ChallengeCoverageSummary.summary_hash

derived summary_id mismatch:
  field = "coverage_summary.artifact.summary_id"
  expected_value = "chcov_" + parsed summary_hash lower-hex without "sha256:"
  actual_value = parsed ChallengeCoverageSummary.summary_id
```

複数が同時に成立する場合は、artifact entry hash mismatch、parsed summary_hash
self-hash mismatch、derived summary_id mismatch の順で最初の failure を返します。
`npa-check release bundle` generation では caller supplied `--coverage-summary-hash` vs
parsed `summary_hash` は Step 7 の `input_hash_mismatch` で既に検査済みなので、
Step 8 では parsed summary_hash self-hash mismatch と derived summary_id mismatch だけを評価します。
generation でこれらが失敗する場合も
`CommandError.reason_code = release_bundle_generation_failed` とし、上の fixed field shape を
そのまま使います。
generation での優先順位は parsed summary_hash self-hash mismatch、derived summary_id mismatch の順です。
`file_hash` は referenced summary file bytes sha256 と一致しなければなりません。
parsed `ChallengeCoverageSummary.policy_hash` は
`ReleasePolicy.challenge_runner_policy_hash` と一致しなければなりません。
parsed `ChallengeCoverageSummary.artifact_hash` は
top-level `ReleaseAuditBundleManifest.artifact_hash` と一致しなければなりません。
parsed `ChallengeCoverageSummary.target_normalized_result_hash` は release target の
`NormalizedCheckResult.normalized_result_hash` と一致しなければなりません。
parsed `ChallengeCoverageSummary.challenge_store_manifest_hash` は
included `challenge_output_store_manifest` entry の `hashes.manifest_hash` と
一致しなければなりません。
parsed `ChallengeCoverageSummary.result_store_manifest_hash` は included
`machine_result_store_manifest` entry の `hashes.manifest_hash` と一致しなければなりません。
missing、duplicate、または extra の `challenge_coverage_summary` entry は bundle invalid です。
MVP の release audit bundle に含める `challenge_replay_result` entry も closed set です。
含めてよい replay result は、included `ChallengeCoverageSummary.entries[*].replay_result_hash` の
distinct set にちょうど対応するものだけです。
ただし `challenge_coverage_summary for challenge_replay_result` が prerequisite-clean でない場合、
bundle generator / validator は summary の entries から replay result set を導出してはいけません。
この場合は `challenge_coverage_summary` の failure を先に返します。
各 distinct replay result hash について `kind = challenge_replay_result` entry が
ちょうど1件必要で、追加の `challenge_replay_result` entry は forbidden です。
各 entry の `hashes.result_hash` は parsed `ChallengeReplayResult.result_hash` および
result_hash 再計算値と一致しなければなりません。
informational replay result、coverage summary entry から参照されない replay result、
または target scope 外の replay result は release audit bundle では forbidden です。

MVP の `ChallengeCoverageSummary` schema：

```json
{
  "schema": "npa.phase8.challenge_coverage_summary.v1",
  "summary_id": "chcov_0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
  "summary_hash": "sha256:...",
  "policy_hash": "sha256:...",
  "artifact_hash": "sha256:...",
  "target_normalized_result_hash": "sha256:...",
  "challenge_store_manifest_hash": "sha256:...",
  "result_store_manifest_hash": "sha256:...",
  "total_challenges": 12,
  "replayed_challenges": 12,
  "unexpected_acceptances": 0,
  "entries": [
    {
      "challenge_id": "pch_001",
      "manifest_hash": "sha256:...",
      "replay_result_hash": "sha256:...",
      "comparison_status": "all_agree_failed"
    }
  ]
}
```

`summary_hash` は `summary_id` と `summary_hash` field を除いた
`ChallengeCoverageSummary` の canonical hash です。
`summary_id` は `chcov_` + `summary_hash` の `sha256:` prefix を除いた lower-hex
64文字に固定します。
`npa-check challenge coverage-summary` は `summary_hash` を計算してから `summary_id` を埋めます。
summary reader と release bundle validator は `summary_id` が `summary_hash` から導出した値と
一致することを検査し、不一致なら coverage summary generation failure または bundle invalid です。
MVP の `ChallengeCoverageSummary.entries[]` entry は `challenge_id`、`manifest_hash`、
`replay_result_hash`、`comparison_status` を required field とします。
`comparison_status` は `NormalizedCheckResult.comparison.status` と同じ closed enum です。
missing、wrong type、explicit null、invalid enum、unknown field、duplicate field は
`ChallengeCoverageSummary` schema / domain validation failure です。
release bundle の Step 6 では `input_schema_invalid` として扱い、field は
`coverage_summary.artifact.entries[].comparison_status` などの caller-prefixed artifact path にします。
`comparison_status` が schema-valid になった後だけ、Step 8 の status binding validation を行います。
`policy_hash` は challenge replay に使った `RunnerPolicy` の canonical hash です。
`artifact_hash` は coverage 対象 target の `NormalizedCheckResult.artifact_hash` で、
release bundle 内では top-level `artifact_hash` と一致しなければなりません。
`target_normalized_result_hash` は coverage 対象 target の
`NormalizedCheckResult.normalized_result_hash` です。
`npa-check challenge coverage-summary` は coverage target を
`--target-normalized-result <path>` と `--target-normalized-result-hash <sha256:...>` で明示的に受け取ります。
`--target-normalized-result-hash` は parsed `NormalizedCheckResult.normalized_result_hash` です。
command は target normalized result file を読み、`normalized_result_hash` を照合し、
その `artifact_hash` が `--artifact-hash` と一致することを検査してから summary を作ります。
`--artifact-hash` だけから target を探索してはいけません。
`challenge_store_manifest_hash` は coverage universe を定義する
`ChallengeOutputStoreManifest` file bytes sha256 です。
nightly pipeline では明示的に与えた `ChallengeOutputStoreManifest` file の hash と一致しなければなりません。
nightly pipeline でも、その store は nightly pass 判定対象の coverage target に対して
target-scoped でなければならず、global / multi-target store を直接使った summary generation は失敗です。
release / high-trust bundle validation では、included `challenge_output_store_manifest` entry の
`hashes.manifest_hash` と一致しなければなりません。
`result_store_manifest_hash` は `unexpected_acceptances` の再計算に使った
machine result store manifest file bytes sha256 です。
release / high-trust に昇格する coverage summary を生成する場合、`--result-store` は
pre-bundle staging step が作った bundle-local machine result store manifest でなければなりません。
同じ場合、`--challenge-store` も store phase で staged した bundle-root 配下の
target-scoped `ChallengeOutputStoreManifest` でなければなりません。
この staged `ChallengeOutputStoreManifest` は source store entry の `manifest_path` を保持するため、
`npa-check challenge coverage-summary` は coverage summary generation 時点ではその original
`manifest_path` から `ChallengeManifest` を読んでよいです。
ただし release bundle validation は original `manifest_path` を読まず、final phase で staged された
同じ `manifest_hash` の `challenge_manifest` artifact だけを使います。
nightly diagnostic だけに使う coverage summary では、target-scoped filtered result store を渡してよいです。
どちらの場合も明示的に与えた `--result-store` file の hash と一致しなければなりません。
release / high-trust bundle validation では、included `machine_result_store_manifest` entry の
`hashes.manifest_hash` と一致しなければなりません。
`entries` は `challenge_id`、次に `manifest_hash` の bytewise lexicographic order で昇順に並べ、
`(challenge_id, manifest_hash)` と `replay_result_hash` はそれぞれ unique です。
`replay_result_hash` duplicate を artifact schema / domain validation failure として報告する場合は、
artifact-local field を `entries[].replay_result_hash`、
`expected_value = "unique_replay_result_hash"`、
`actual_value = "duplicate_replay_result_hash"` に固定します。
`npa-check release bundle` の Step 6 では caller prefix を付けて
`field = "coverage_summary.artifact.entries[].replay_result_hash"` とします。
`replay_result_hash` は referenced `ChallengeReplayResult.result_hash` です。
MVP の coverage summary に含める `ChallengeReplayResult` は
`normalized_result_hash` と `comparison_status` を持たなければなりません。
`npa-check challenge coverage-summary` は、まず referenced replay result ごとに
replay store reference / artifact base validation を行います。
missing / ambiguous / entry file unreadable / entry JSON parse failure /
non-coverage-required entry schema or domain failure / entry hash mismatch /
parsed artifact mismatch は cross-artifact coverage semantics failure であり、
coverage-required field validation より先に返します。
ここで non-coverage-required entry schema or domain failure とは、
`normalized_result_hash` と `comparison_status` に関する下の 1、2、3 以外の
replay artifact schema / domain failure です。
同じ replay result に base validation failure と coverage-required field validation failure が
両方ある場合は base validation failure を先に返します。
複数の replay result で base validation failure が同時に成立する場合は、
生成予定の `ChallengeCoverageSummary.entries[]` order、つまり `challenge_id`、次に
`manifest_hash` の bytewise lexicographic order で最初の coverage entry candidate を選びます。
base validation を通過した referenced replay result だけに、
coverage-required field validation を次の順で行います。

1. `ChallengeReplayResult.normalized_result_hash` shape validation:
   wrong type、explicit null、invalid hash format は
   `CommandError.reason_code = coverage_summary_generation_failed`、
   `field = "replay_store.results[].normalized_result_hash"`、
   `expected_value = "sha256:<lower-hex>"`、
   `actual_value = wrong_type | null_not_allowed | invalid_hash_format` です。
   この場合は `comparison_status` の conditional required / forbidden validation を評価しません。
2. `ChallengeReplayResult.comparison_status` conditional schema / domain validation:
   valid な `normalized_result_hash` を持つ replay result で
   `comparison_status` が missing、wrong type、explicit null、invalid enum の場合は
   `field = "replay_store.results[].comparison_status"`、
   `expected_value = "NormalizedCheckResult.comparison.status"`、
   `actual_value = missing | wrong_type | null_not_allowed | invalid_enum` です。
   `normalized_result_hash` が absent なのに `comparison_status` が present の場合は
   `field = "replay_store.results[].comparison_status"`、
   `expected_value = "absent_without_normalized_result_hash"`、
   `actual_value = "present"` です。
3. coverage-required semantic requirement:
   replay artifact schema / domain validation を通過したうえで
   `normalized_result_hash` を持たない replay result は covered challenge として数えず、
   `CommandError.reason_code = coverage_summary_generation_failed`、
   `field = "replay_store.results[].normalized_result_hash"`、
   `expected_value = "required_for_coverage_summary"`、
   `actual_value = "missing"` として扱います。
   この場合は schema-valid informational replay として `comparison_status` も omit されているため、
   `comparison_status` missing failure を追加で合成してはいけません。

複数の referenced `ChallengeReplayResult` で coverage-required field validation failure が
同時に成立する場合は、生成予定の `ChallengeCoverageSummary.entries[]` order、
つまり `challenge_id`、次に `manifest_hash` の bytewise lexicographic order で最初の
coverage entry candidate を選びます。
その candidate の中で複数 failure が成立する場合だけ、上の 1、2、3 の順で最初の failure を返します。
そのため nightly / release pipeline は coverage summary generation 前に、
各 challenge replay result の `normalized_result_hash` が解決済みであることを要求します。
release / high-trust bundle validator は、各 included `ChallengeReplayResult.normalized_result_hash` が指す
included challenge replay `NormalizedCheckResult` の embedded comparison を再計算した後、
`ChallengeReplayResult.comparison_status` がその再計算済み comparison.status と一致することを検査します。
この mismatch は `challenge_replay_result` class 5 source-value mismatch です。
さらに `ChallengeCoverageSummary.entries[].comparison_status` は、同じ entry の
`replay_result_hash` が参照する `ChallengeReplayResult.comparison_status` と一致しなければなりません。
この mismatch は `challenge_coverage_summary` class 5 source-value mismatch です。
この status binding validation では、参照先 challenge replay `NormalizedCheckResult` の
comparison 再計算が失敗した場合、status binding mismatch を合成せず、
`normalized_check_result` class 5 recomputation failure を先に返します。
この status binding validation は `normalized_check_result set` の prerequisite-clean 条件に含めます。
binding mismatch がある場合は、その source kind の class 5 failure を返し、
`import_lock` など downstream expected set の missing / forbidden / duplicate failure を合成してはいけません。
coverage pass condition は、これらの status binding がすべて通った後の
`ChallengeCoverageSummary.entries[].comparison_status` だけを使います。
`total_challenges` は coverage universe の `ChallengeOutputStoreManifest.entries.length` です。
Phase 8 MVP の coverage universe は rejection-required entry だけに deterministic filter 済みなので、
informational entry が含まれる `ChallengeOutputStoreManifest` は nightly coverage summary generation failure、
または release audit bundle invalid です。
`total_challenges` を release bundle に含まれる `challenge_manifest` entry 数から直接計算してはいけません。
`replayed_challenges` は `entries.length` です。
nightly pipeline では各 entry の `manifest_hash` が coverage universe の manifest entry を参照し、
`replay_result_hash` が nightly pipeline の replay result set に含まれる
`ChallengeReplayResult` を参照しなければなりません。
release / high-trust bundle validation では各 entry の `manifest_hash` は同じ bundle 内の
`challenge_manifest` entry を参照し、`replay_result_hash` は同じ bundle 内の
`challenge_replay_result` entry を参照しなければなりません。
参照した `ChallengeManifest.challenge_id` は entry の `challenge_id` と一致しなければなりません。
参照した manifest の `mutation.kind` が informational の場合、その entry は invalid です。
参照した `ChallengeReplayResult.challenge_id`、`manifest_hash`、`policy_hash` は、
entry の `challenge_id`、entry の `manifest_hash`、summary の `policy_hash` と
それぞれ完全一致しなければなりません。
参照した `ChallengeReplayResult.mutated_file_hash` は
`ChallengeManifest.mutated_certificate.file_hash` と一致しなければなりません。
`ChallengeManifest.mutated_certificate.claimed_certificate_hash` が存在する場合は
`ChallengeReplayResult.mutated_claimed_certificate_hash` と一致しなければならず、
manifest 側で omit された場合は replay result 側も omit しなければなりません。
参照した `ChallengeManifest.base_certificate.file_hash` は、coverage target の
`NormalizedCheckResult.artifact.input_file_hash` と一致しなければなりません。
参照した `ChallengeManifest.base_certificate.claimed_certificate_hash` は、coverage target の
`NormalizedCheckResult.artifact.expected_certificate_hash` と一致しなければなりません。
nightly pipeline で使う coverage target は、nightly pass 判定対象の
`NormalizedCheckResult` です。
release / high-trust bundle validation で使う coverage target は、bundle 内で top-level
`ReleaseAuditBundleManifest.artifact_hash` と同じ `artifact_hash` を持つ唯一の
`kind = normalized_check_result` entry です。
`ChallengeReplayResult.artifact_hash` は mutated certificate 側の artifact identity なので、
`ChallengeCoverageSummary.artifact_hash` と一致することを要求してはいけません。
`replayed_challenges` は `total_challenges` 以下でなければなりません。
nightly / release pass condition では `replayed_challenges = total_challenges` を要求します。
さらに rejection-required challenge の各 entry は `comparison_status = all_agree_failed` でなければなりません。
`missing_checker_result`、`policy_failure`、`inconclusive`、`disagreement`、
`all_agree_checked` は coverage pass ではありません。
`unexpected_acceptances` は `comparison_status = all_agree_checked` または
required checker の checked acceptance が観測された entry 数です。
required checker の checked acceptance は、referenced `ChallengeReplayResult.checker_results`
から included `MachineCheckResult` を解決して再計算します。
release / nightly pass condition では `unexpected_acceptances = 0` を要求します。
`ReleaseAuditBundleManifest` は release / high-trust pass artifact なので、
release / high-trust bundle validation では次のいずれかを bundle invalid とします。

```text
- replayed_challenges != total_challenges
- any rejection-required entry has comparison_status != all_agree_failed
- unexpected_acceptances != 0
- recomputed unexpected_acceptances differs from the stored unexpected_acceptances
```

nightly pipeline では同じ条件を nightly pass failure として扱い、
release audit bundle は materialize しません。
coverage summary generation failure では `ChallengeCoverageSummary` を保存せず、
nightly pipeline failure、または release audit bundle generation failure として扱います。
`audit_bundle` kind の `AuxiliaryResult.artifact_hash` は、その AuxiliaryResult が検証した
target bundle の `bundle_hash` と一致しなければなりません。
同じ `ReleaseAuditBundleManifest` の `artifacts` に、その manifest 自身の `bundle_hash` を
`artifact_hash` に持つ `kind = audit_bundle` の `AuxiliaryResult` を含めてはいけません。
これは `bundle_hash` と `AuxiliaryResult.result_hash` の循環を避けるためです。
audit bundle validation result を保存する場合は、target bundle の外に置くか、
target bundle を参照する別の post-audit bundle に含めます。

MVP の `CheckerIdentityManifest` schema：

```json
{
  "schema": "npa.phase8.checker_identity_manifest.v1",
  "generated_by": {
    "runner_id": "npa-check-runner",
    "runner_version": "0.8.0",
    "runner_build_hash": "sha256:..."
  },
  "checkers": [
    {
      "profile": "reference",
      "checker_id": "npa-checker-ref",
      "checker_version": "0.8.0",
      "binary_id": "npa-checker-ref-macos-aarch64",
      "binary_hash": "sha256:...",
      "build_hash": "sha256:..."
    }
  ]
}
```

`checker_identity_manifest` の `manifest_hash` は、この JSON file の exact bytes の sha256 です。
manifest object 自体に `manifest_hash` field は持たせません。
MVP の top-level required field は `schema`、`generated_by`、`checkers` です。
top-level manifest object は closed-world object で、unknown field と duplicate key を禁止します。
`generated_by` は closed-world object で、required field は `runner_id`、`runner_version`、
`runner_build_hash` です。
`generated_by.runner_id` と `generated_by.runner_version` は non-empty string、
`generated_by.runner_build_hash` は `sha256:<lower-hex>` です。
`generated_by` は manifest provenance metadata であり、runner pre-check identity の照合対象ではありません。
runner が現在実行中の `runner.id` / `runner.version` / `runner.build_hash` と
`generated_by` が一致しないことだけを理由に checker を拒否してはいけません。
`checkers` は `profile` の bytewise lexicographic order で昇順に並べます。
`profile` と `binary_id` はそれぞれ unique です。
`checkers[]` entry object は closed-world object で、unknown field と duplicate key を禁止します。
`profile` は 4.1 の checker profile name grammar、
`checker_id` と `binary_id` は 4.1 の `checker_allowlist[].checker_id` /
`checker_allowlist[].binary_id` と同じ grammar を使います。
`checker_id`、`binary_id`、`binary_hash`、`build_hash` は required です。
`checker_version` は optional metadata で、存在する場合は string でなければなりません。
runner が起動前 build identity 照合にこの manifest を使う場合、
単一 `MachineCheckRequest` の runner pre-check では、
`MachineCheckRequest.checker_profile` に対応する `SelectedCheckerPolicy` だけを
manifest と照合します。
`SelectedCheckerPolicy` と同じ `profile`、`checker_id`、`binary_id`、
`binary_hash`、`build_hash` の entry が存在しなければなりません。
未選択 required / optional profile の manifest entry 欠落や mismatch は、
この run の `MachineCheckResult` では `policy_failure` にしません。
全 profile の manifest completeness を検査したい場合は、それぞれの profile の
request / replay か audit validation で扱います。
`checker_version` は audit / display 用 metadata であり、MVP の policy identity には含めません。
runner は `checker_version` mismatch だけを理由に checker を拒否してはいけません。
checker を起動して raw identity check が通った場合、`MachineCheckResult.checker.version` には
`CheckerRawResult.checker_version` を記録します。
`SelectedCheckerPolicy` と一致しない場合は checker を起動せず、
4 の runner pre-check field shape に従って、
missing entry は `checker_identity_missing`、`checker_id` / `binary_id` mismatch は
`checker_identity_mismatch`、`binary_hash` mismatch は `checker_binary_hash_mismatch`、
`build_hash` mismatch は `checker_build_hash_mismatch` の `policy_failure` にします。

CheckerIdentityManifest schema failure では `error.field` に
`checker_identity_manifest.` prefix 付きの invalid field JSON path を入れます。
top-level `schema` が `npa.phase8.checker_identity_manifest.v1` でない場合は、
`error.field = "checker_identity_manifest.schema"`、
`expected_value = "npa.phase8.checker_identity_manifest.v1"`、
`actual_value = "missing"`、`"null_not_allowed"`、`"wrong_type"`、
または入力 manifest の `schema` 文字列にします。
top-level schema mismatch では `actual_value = "wrong_schema"` を使いません。
top-level JSON value が object でない場合は
`error.field = "checker_identity_manifest.$"`、
`expected_value = "object"`、`actual_value = "wrong_type"` または `"null_not_allowed"` にします。
それ以外の schema failure では `expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_hash_format`、
`invalid_name_format`、`null_not_allowed`、`order_violation`、`duplicate_field` のいずれかを入れます。
domain failure では次の field shape を使います。

```text
generated_by.runner_id / runner_version が空文字列:
  field = "checker_identity_manifest.generated_by.runner_id"
       | "checker_identity_manifest.generated_by.runner_version"
  expected_value = "non_empty_string"
  actual_value = "empty_string"

checkers が profile 昇順でない:
  field = "checker_identity_manifest.checkers[<i>].profile"
  expected_value = "profile_bytewise_ascending"
  actual_value = "order_violation"

checkers[].profile grammar violation:
  field = "checker_identity_manifest.checkers[<i>].profile"
  expected_value = "checker_profile_name"
  actual_value = "invalid_name_format"

checkers[].checker_id grammar violation:
  field = "checker_identity_manifest.checkers[<i>].checker_id"
  expected_value = "checker_id"
  actual_value = "invalid_name_format"

checkers[].binary_id grammar violation:
  field = "checker_identity_manifest.checkers[<i>].binary_id"
  expected_value = "checker_binary_id"
  actual_value = "invalid_name_format"

checkers[].profile が重複する:
  field = "checker_identity_manifest.checkers[<i>].profile"
  expected_value = "unique_profiles"
  actual_value = "duplicate_profile"

checkers[].binary_id が重複する:
  field = "checker_identity_manifest.checkers[<i>].binary_id"
  expected_value = "unique_binary_ids"
  actual_value = "duplicate_binary_id"
```

CheckerIdentityManifest validation は schema failure を domain failure より先に報告します。
複数の schema failure が同時に存在する場合は、次の順で最初の1件だけを返します。

```text
1. top-level JSON value is not object
2. top-level schema
3. generated_by object
4. generated_by.runner_id
5. generated_by.runner_version
6. generated_by.runner_build_hash
7. checkers array
8. checkers[] entry object, by smaller array index
9. checkers[].profile, by smaller array index
10. checkers[].checker_id, by smaller array index
11. checkers[].checker_version, by smaller array index when present
12. checkers[].binary_id, by smaller array index
13. checkers[].binary_hash, by smaller array index
14. checkers[].build_hash, by smaller array index
15. unknown field, by the containing object order above and then bytewise field name
```

known field の duplicate object key は、その field の schema order 位置で
`expected_value = "unique_object_keys"`、`actual_value = "duplicate_field"` として報告します。
unknown field の duplicate object key は item 15 の位置で
`expected_value = "unique_object_keys"`、`actual_value = "duplicate_field"` を返し、
`error.field` は重複した後続 unknown field の JSON path にします。
複数の domain failure が同時に存在する場合は、上の domain failure table の順で
最初の1件だけを返します。
同じ table row 内で複数の `checkers[]` entry が失敗する場合は小さい array index を優先します。
`checkers` order violation の `<i>` は、最初に `profile` が直前 entry より小さくなる
後続 entry index です。
`profile` / `binary_id` duplicate は、同じ key がすでに出現している最小の後続 entry index を
報告対象にします。
checker identity manifest file の `manifest_hash` mismatch や schema / domain failure がある場合、
runner は `SelectedCheckerPolicy` との entry 照合を行わず、4.1 の checker executable /
identity validation order に従って manifest-level failure を先に返します。

AI sidecar には次を含めます。

```text
- source result hash or normalized comparison hash
- prompt hash
- model identity
- redaction policy
- input artifact list
- generated summary
- generated next actions
```

AI sidecar に含めないもの：

```text
- secret tokens
- private source not allowed by policy
- raw prompt with unrelated user data
- checker binary path selected outside policy
- generated certificate bytes
```

---

# 14. Prompt and data policy

AI に渡す情報は最小化します。
特に private repository / unreleased theorem / proprietary library を扱う場合、
checker result の structured field だけで triage できるようにします。

default prompt payload：

```text
- module name
- input_file_hash
- expected_certificate_hash
- recomputed certificate_hash, if present in MachineCheckResult
- checker ids and versions
- status and error kind
- failed declaration name
- dependency hash path
- policy mode
- relevant previous failures
```

default で渡さないもの：

```text
- full certificate bytes
- full source file
- full proof term
- full tactic trace
- private theorem statements
- local filesystem absolute paths outside workspace
```

必要な場合だけ、policy で明示して追加します。
追加した情報は `AiAuditSidecar.input_policy` に記録します。

---

# 15. Training data

Phase 8 AI Profile から training data を作る場合、
label は checker result からだけ作ります。

positive example：

```text
input:
  certificate metadata + checker profile

label:
  status = checked from MachineCheckResult
```

negative example：

```text
input:
  mutated certificate metadata + checker profile

label:
  status = failed
  error.kind = noncanonical_encoding
```

禁止事項：

```text
- AI triage confidence を正解ラベルにする
- human PR comment だけから checker success label を作る
- source diff だけから certificate validity label を作る
- failed checker result を AI explanation で checked に変更する
```

training identity は次を含めます。

```text
always:
  - artifact.input_file_hash
  - artifact.expected_certificate_hash
  - checker_profile
  - result_hash
  - policy.hash
  - policy.version

when present in MachineCheckResult:
  - MachineCheckResult.certificate_hash
  - MachineCheckResult.checker.id as checker_id
  - MachineCheckResult.checker.build_hash as checker_build_hash
```

ここでの `MachineCheckResult.certificate_hash` は checker が再計算した canonical certificate hash です。
expected hash や file bytes hash と混同してはいけません。
`checker_id` と `checker_build_hash` は accepted checker verdict では required ですが、
pre-check failure、launch failure、malformed raw output、または identity missing policy failure では
片方または両方が存在しない場合があります。
training export はこれらの result を skip したり export failure にしたりせず、
`MachineCheckResult.checker.id` / `checker.build_hash` に存在する field だけを
`TrainingExample.identity` に写し、存在しない field は omit します。
`result_id` は再実行で変わり得るため、training identity には含めません。

MVP の training export は checker / CI artifact ではなく、offline evaluation 用 dataset です。
training export の record は JSON Lines とし、各 line は次の closed-world object です。

```json
{
  "schema": "npa.phase8.training_example.v1",
  "example_id": "trn_0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
  "source": {
    "kind": "machine_result",
    "result_hash": "sha256:...",
    "request_hash": "sha256:...",
    "run_artifact_hash": "sha256:..."
  },
  "identity": {
    "artifact_input_file_hash": "sha256:...",
    "artifact_expected_certificate_hash": "sha256:...",
    "certificate_hash": "sha256:...",
    "checker_id": "npa-checker-ref",
    "checker_build_hash": "sha256:...",
    "checker_profile": "reference",
    "result_hash": "sha256:...",
    "policy_hash": "sha256:...",
    "policy_version": 1
  },
  "input": {
    "module": "Std.Nat",
    "checker_profile": "reference",
    "trust_mode": "pr"
  },
  "label": {
    "source": "machine_check_result",
    "status": "failed",
    "error_kind": "type_mismatch"
  }
}
```

`example_id` は `trn_` + `source.run_artifact_hash` の `sha256:` prefix を除いた lower-hex
64文字に固定します。
同じ `run_artifact_hash` から複数 record を作ってはいけません。
training export は `source.kind = machine_result` の record だけを MVP で生成します。
`status = checked` の label では `error_kind` と `reason_code` を omit します。
source `MachineCheckResult.error.reason_code` が存在する failed label では `reason_code` を入れ、
存在しない場合は field を omit します。explicit null は使いません。
`TrainingExample.identity.certificate_hash`、`checker_id`、`checker_build_hash` は optional copied metadata です。
存在しない場合は field を omit し、explicit null は使いません。
それ以外の `identity` field は required です。
`input` には `AiAuditInputPolicy.included_fields` と同じ allowlist discipline を適用し、
full certificate bytes、full proof term、source text、tactic trace、absolute path、secret は入れません。
`npa-check training export` は `MachineCheckResult` と `NormalizedCheckResult` store を検証し、
record の label を必ず `MachineCheckResult.status` / `error.kind` / `error.reason_code` から写します。
AI sidecar の `classification`、`confidence`、`summary`、PR comment は training label に使ってはいけません。
MVP の export 対象集合は normalized store manifest に含まれる
`NormalizedCheckResult.results[*]` が参照する `MachineCheckResult` だけです。
result store に存在していても、どの `NormalizedCheckResult` からも参照されない
`MachineCheckResult` は export しません。
各 normalized result entry の `(result_hash, request_hash, checker_profile, policy_hash)` は、
machine result store 内の saved `MachineCheckResult` にちょうど1件解決できなければなりません。
0件の場合、または retry / repeated run により2件以上が一致する場合は
`CommandError.reason_code = training_export_generation_failed` です。
caller は export 前に採用する retry だけを含む filtered result store manifest を渡します。
同じ `run_artifact_hash` が複数の normalized result から参照される場合は1 record に deduplicate します。

JSON Lines file を保存する場合は、別途 `TrainingExportManifest` を保存します。

```json
{
  "schema": "npa.phase8.training_export_manifest.v1",
  "export_id": "trn_export_0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
  "file_hash": "sha256:...",
  "record_count": 128,
  "sources": {
    "result_store_manifest_hash": "sha256:...",
    "normalized_store_manifest_hash": "sha256:..."
  }
}
```

`file_hash` は JSON Lines file bytes の SHA-256 です。
`export_id` は `trn_export_` + `file_hash` の `sha256:` prefix を除いた lower-hex
64文字に固定します。
`TrainingExportManifest` は CI pass condition、release audit bundle、checker verdict identity には含めません。
training export を再生成した場合、record order は
`source.run_artifact_hash` の bytewise lexicographic order で固定します。
Phase 8 MVP の `npa-check training export` は JSON Lines file を成果物として保存する command なので、
`--normalized-store` / `--normalized-store-hash`、`--result-store` / `--result-store-hash`、
`--out`、`--manifest-out` は required input reference です。
required input reference の欠落、path/hash pair の片方だけ、hash flag の invalid hash format、
または path schema violation は `CommandError.reason_code = input_reference_invalid` です。
store path/hash pair が完全に欠けている場合は `field = "normalized_store"` または
`field = "result_store"`、`expected_value = "required_reference_pair"`、
`actual_value = "missing"` とします。
path だけが欠けている場合は `field = "<store>.path"`、hash だけが欠けている場合は
`field = "<store>.manifest_hash"`、`expected_value = "required"`、
`actual_value = "missing"` とします。
`--out` または `--manifest-out` が欠けている場合は `field = "out.path"` または
`field = "manifest_out.path"`、`expected_value = "required"`、`actual_value = "missing"` とします。
path schema violation では `expected_value = "workspace_relative_path"`、
`actual_value = "invalid_path"` とし、hash format violation では
`expected_value = "sha256:<lower-hex>"`、`actual_value = "invalid_hash_format"` とします。
duplicate singleton flag、unsupported flag、missing `--json` は CLI argument validation error であり、
`CommandError` body を返しません。
store manifest file を読めない場合は `input_file_unreadable` で、
`field = "<store>.path"`、`actual_value = "unreadable"` とします。
store manifest file bytes と caller supplied hash が一致しない場合は
`input_hash_mismatch` で、`field = "<store>.manifest_hash"`、
`expected_hash` は caller supplied hash、`actual_hash` は file bytes sha256 です。
`normalized_store` / `result_store` の JSON parse / schema / order / duplicate failure は
`input_store_manifest_invalid` で、JSON parse failure では `field = "<store>.path"`、
`actual_value = "invalid_json"`、それ以外では `field = "<store>.<json_path>"` とします。
store entry の missing / ambiguous / entry file unreadable / entry JSON parse failure /
entry schema or domain failure / entry hash mismatch / parsed artifact mismatch、
および normalized result と machine result の cross-store mismatch は
`CommandError.reason_code = training_export_generation_failed`、
`field = "command"`、`actual_value = "training_export_generation_failed"` とします。
JSON Lines file と `TrainingExportManifest` は temporary file として作り、
final JSON Lines path を配置してから manifest を atomic replace します。
`TrainingExportManifest.file_hash` が final JSON Lines file bytes を参照して初めて export 成功です。
manifest commit 前に failure した場合、manifest を更新してはいけません。
retry 時に final JSON Lines path が既に存在し、その file bytes が今回生成する bytes と完全一致する場合は
上書きではなく既存 file の採用として扱います。
既存 final JSON Lines path または manifest path の bytes が異なる場合は
`CommandError.reason_code = output_path_conflict` です。
`field` は `out.path` または `manifest_out.path` とし、
`expected_hash` に今回生成する file bytes hash、`actual_hash` に既存 file bytes hash を入れます。
temporary write / atomic replace failure は `output_write_failure` で、
`field = "out.path"` または `field = "manifest_out.path"`、
`actual_value = "write_failed"` とします。
`--json` 成功時 stdout は保存された `TrainingExportManifest` です。
JSON Lines 本体を stdout に出す mode、inline `TrainingExportManifest` だけを返す no-output mode、
または manifest なしで JSON Lines だけを保存する mode は MVP では定義しません。

---

# 16. Security considerations

Phase 8 AI Profile で想定する攻撃：

```text
- prompt injection in source comments
- malicious theorem name or diagnostic text
- adversarial pretty printer output
- AI-selected stale checker binary
- AI-selected permissive checker flag
- poisoned previous failure summary
- challenge manifest that claims outcome_hint success
- fake MachineCheckResult created by non-checker process
```

対策：

```text
- checker runner は binary allowlist を使う
- checker runner は network を使わない
- checker result は build hash と result hash を持つ
- AI は raw log ではなく structured result を優先して読む
- pretty text は command / prompt instruction として扱わない
- challenge outcome_hint は oracle ではなく metadata として扱う
- result artifact は append-only storage に保存する
```

AI prompt には必ず次の system-level invariant を入れます。

```text
You are not a checker.
Do not declare any certificate valid.
Only summarize deterministic checker results.
If checker results disagree, report failure.
```

---

# 17. Machine commands

Phase 8 AI Profile の MVP で必要な command：

```sh
npa-check run --policy ci/phase8-pr-policy.json --policy-hash sha256:... --request build/check-requests/Std.Nat.reference.json --json
npa-check run --policy ci/phase8-pr-policy.json --policy-hash sha256:... --request build/check-requests/Std.Nat.external.json --json
npa-check normalize-results --policy ci/phase8-pr-policy.json --policy-hash sha256:... --request-store build/check-requests/manifest.json --request-store-hash sha256:... --selector-module Std.Nat --selector-request-hash sha256:... --out build/normalized/Std.Nat.json --normalized-store-out build/normalized/manifest.json --json build/check-results/*.json
npa-check compare --policy ci/phase8-pr-policy.json --policy-hash sha256:... --json build/normalized/Std.Nat.json
npa-check challenge generate --policy ci/phase8-nightly-policy.json --policy-hash sha256:... --challenge-store build/challenges/manifest.json --challenge-id pch_001 --module Std.Nat --imports build/certs/import-lock.json --imports-hash sha256:... --kind drop_axiom_report_entry --target Nat.add_zero --seed sha256:... --from build/certs/Std/Nat.npcert --generated-by ai --prompt-hash sha256:... --manifest-out build/challenges/pch_001/manifest.json --mutated-out build/challenges/pch_001/Std.Nat.mutated.npcert --json
npa-check challenge materialize-requests --manifest build/challenges/pch_001/manifest.json --manifest-hash sha256:... --policy ci/phase8-nightly-policy.json --policy-hash sha256:... --request-dir build/check-requests/challenges/pch_001 --request-store-out build/check-requests/challenge-manifest.json --json
npa-check normalize-results --policy ci/phase8-nightly-policy.json --policy-hash sha256:... --request-store build/check-requests/challenge-manifest.json --request-store-hash sha256:... --selector-module Std.Nat --selector-request-hash sha256:... --out build/normalized/challenges/pch_001/Std.Nat.json --normalized-store-out build/normalized/challenge-manifest.json --json build/check-results/challenges/pch_001/*.json
npa-check challenge replay --manifest build/challenges/pch_001/manifest.json --manifest-hash sha256:... --policy ci/phase8-nightly-policy.json --policy-hash sha256:... --request-store build/check-requests/challenge-manifest.json --request-store-hash sha256:... --result-store build/check-results/manifest.json --result-store-hash sha256:... --normalized-store build/normalized/challenge-manifest.json --normalized-store-hash sha256:... --coverage-required --out build/challenge-replays/pch_001.json --replay-store-out build/challenge-replays/manifest.json --json
npa-check release stage-bundle-inputs --phase store --bundle-root build/release-audit/Std.Nat --plan ci/release-stage-store.json --plan-hash sha256:... --json
npa-check challenge coverage-summary --policy ci/phase8-nightly-policy.json --policy-hash sha256:... --artifact-hash sha256:... --target-normalized-result build/release-audit/Std.Nat/artifacts/normalized_check_result/dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd.json --target-normalized-result-hash sha256:... --challenge-store build/release-audit/Std.Nat/artifacts/challenge_output_store_manifest/2222222222222222222222222222222222222222222222222222222222222222.json --challenge-store-hash sha256:... --replay-store build/challenge-replays/manifest.json --replay-store-hash sha256:... --result-store build/release-audit/Std.Nat/artifacts/machine_result_store_manifest/ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff.json --result-store-hash sha256:... --out build/challenges/coverage/Std.Nat.json --json
npa-check auxiliary axiom-policy --policy ci/phase8-pr-policy.json --policy-hash sha256:... --normalized-result build/normalized/Std.Nat.json --normalized-result-hash sha256:... --axiom-report-store build/axiom-reports/manifest.json --axiom-report-store-hash sha256:... --out build/aux/Std.Nat.axiom-policy.json --json
npa-check auxiliary reproducibility --policy ci/phase8-nightly-policy.json --policy-hash sha256:... --baseline-run-artifact-hash sha256:... --repeated-run-artifact-hash sha256:... --result-store build/check-results/manifest.json --result-store-hash sha256:... --out build/aux/Std.Nat.reproducibility.json --json
npa-check auxiliary import-certificate-hash --release-policy ci/phase8-release-policy.json --release-policy-hash sha256:... --import-lock build/certs/import-lock.json --import-lock-hash sha256:... --out build/aux/import-lock.import-certificate-hash.json --json
npa-check release stage-bundle-inputs --phase final --bundle-root build/release-audit/Std.Nat --plan ci/release-stage-final.json --plan-hash sha256:... --json
npa-check release bundle --release-policy build/release-audit/Std.Nat/artifacts/release_policy/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.json --release-policy-hash sha256:... --runner-policy build/release-audit/Std.Nat/artifacts/runner_policy/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.json --runner-policy-hash sha256:... --challenge-runner-policy build/release-audit/Std.Nat/artifacts/runner_policy/cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc.json --challenge-runner-policy-hash sha256:... --artifact-hash sha256:... --target-normalized-result build/release-audit/Std.Nat/artifacts/normalized_check_result/dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd.json --target-normalized-result-hash sha256:... --request-store build/release-audit/Std.Nat/artifacts/request_store_manifest/eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee.json --request-store-hash sha256:... --result-store build/release-audit/Std.Nat/artifacts/machine_result_store_manifest/ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff.json --result-store-hash sha256:... --normalized-store build/release-audit/Std.Nat/artifacts/normalized_result_store_manifest/1111111111111111111111111111111111111111111111111111111111111111.json --normalized-store-hash sha256:... --challenge-store build/release-audit/Std.Nat/artifacts/challenge_output_store_manifest/2222222222222222222222222222222222222222222222222222222222222222.json --challenge-store-hash sha256:... --challenge-replay-result build/release-audit/Std.Nat/artifacts/challenge_replay_result/7777777777777777777777777777777777777777777777777777777777777777.json --challenge-replay-result-hash sha256:... --coverage-summary build/release-audit/Std.Nat/artifacts/challenge_coverage_summary/3333333333333333333333333333333333333333333333333333333333333333.json --coverage-summary-hash sha256:... --auxiliary-result build/release-audit/Std.Nat/artifacts/auxiliary_result/4444444444444444444444444444444444444444444444444444444444444444.json --auxiliary-result-hash sha256:... --auxiliary-result build/release-audit/Std.Nat/artifacts/auxiliary_result/5555555555555555555555555555555555555555555555555555555555555555.json --auxiliary-result-hash sha256:... --ai-audit-input-policy build/release-audit/Std.Nat/artifacts/ai_audit_input_policy/6666666666666666666666666666666666666666666666666666666666666666.json --ai-audit-input-policy-hash sha256:... --bundle-root build/release-audit/Std.Nat --out build/release-audit/Std.Nat/manifest.json --json
npa-check release validate-bundle --manifest build/release-audit/Std.Nat/manifest.json --manifest-hash sha256:... --out build/aux/Std.Nat.audit-bundle.json --json
npa-check training export --normalized-store build/normalized/manifest.json --normalized-store-hash sha256:... --result-store build/check-results/manifest.json --result-store-hash sha256:... --out build/training/phase8-examples.jsonl --manifest-out build/training/phase8-examples.manifest.json --json
npa-check audit-sidecar validate --sidecar build/audit/Std.Nat.ai.json --result-store build/check-results/manifest.json --result-store-hash sha256:... --normalized-store build/normalized/manifest.json --normalized-store-hash sha256:... --input-policy ci/phase8-ai-triage-default.json --input-policy-hash sha256:...
```

AI agent はこれらの command を提案または runner 経由で起動できます。
`auxiliary`、`challenge coverage-summary`、`release stage-bundle-inputs`、
`release bundle`、`release validate-bundle`、`training export` は
AI command ではなく deterministic pipeline command です。
AI はこれらの結果を説明できますが、`AuxiliaryResult`、`ChallengeCoverageSummary`、
`ReleaseAuditBundleManifest`、training export label を作る oracle にはなりません。
MVP の machine API endpoint は 18 に列挙するものだけです。
上記の追加 deterministic pipeline command は、対応する endpoint を明示的に追加するまでは
CLI / file-backed pipeline command として扱い、`/machine/check/release` や
`/machine/check/training` などの endpoint を推定してはいけません。
`npa-check audit-sidecar validate --schema-only --sidecar ...` は sidecar schema だけを検査します。
`--schema-only` なしの validate は 7 の cross-artifact validation を行います。
MVP CLI の mode flag は `--schema-only` だけです。
`--schema-only` が存在すれば schema-only mode、存在しなければ cross-artifact mode に固定します。
`--schema-only` の重複、値付き形式 `--schema-only=<value>`、`--no-schema-only`、
`--cross-artifact` などの別 mode flag は CLI argument error とし、
`AuditSidecarValidationResult` body を返しません。
CLI の audit-sidecar validation では、`--sidecar`、`--result-store`、`--normalized-store`、
`--input-policy` の path 引数も workspace-relative path schema の対象です。
ただし `--schema-only` で forbidden になる `--result-store`、`--normalized-store`、
`--input-policy` は、path 引数の中身を検査せず forbidden reference presence として扱います。
対象 path が absolute path、drive prefix、empty segment、`.` / `..` segment、
control character、または workspace 外解決になる場合は、file read を試みず、
`AuditSidecarValidationResult.error.reason_code = validation_reference_schema_invalid`、
`actual_value = invalid_path` を返します。
cross-artifact validation では `--result-store` / `--result-store-hash` と
`--input-policy` / `--input-policy-hash` は required です。
`--normalized-store` / `--normalized-store-hash` は、sidecar が
`source.normalized_result_hash` を持つ場合 required で、それ以外では optional です。
cross-artifact validation の active reference pair が完全に欠けている場合は
CLI argument error ではなく `validation_reference_missing` として扱います。
path flag と hash flag の片方だけが指定された partial reference は
`validation_reference_schema_invalid` として扱います。
cross-artifact validation で optional `--normalized-store` が指定された場合も、
validator は manifest hash / schema / entry hashes を検証します。
`--schema-only` では `--result-store`、`--result-store-hash`、
`--normalized-store`、`--normalized-store-hash`、`--input-policy`、
`--input-policy-hash` はすべて forbidden です。
hash-only flag だけが指定された場合も forbidden reference presence です。
どちらの mode でも証明の受理判定は行いません。

`npa-check run` の正本入力は `--request` で渡す `MachineCheckRequest` と、
`--policy` で渡す `RunnerPolicy` file です。
`--policy` と `--policy-hash` は trusted invocation input であり、
request file 内の policy metadata と照合されます。
`npa-check run` は optional `--attempt <positive-int>` を受け取り、省略時は `attempt = 1` です。
`--attempt` は result store の採番や書き込みを行う flag ではなく、
返す `MachineCheckResult.attempt` に写す値です。
`npa-check run --json` の stdout は `MachineCheckResult` または
`MachineCheckRequestErrorResult` です。
request load validation を通った場合だけ `MachineCheckResult` を出力します。
challenge の単一 profile 実行も `npa-check run --policy ... --policy-hash ... --request ...` を使い、
その stdout は通常の `MachineCheckResult` です。
`npa-check normalize-results` の required input は `--policy` / `--policy-hash`、
`--request-store` / `--request-store-hash`、`--json`、および1件以上の
`MachineCheckResult` file path です。
missing required flag、duplicate singleton flag、unsupported flag、missing `--json`、
または result file path が0件の invocation は CLI argument validation error であり、
`NormalizeErrorResult` body を返しません。
`--request-store` と `--request-store-hash` は required pair なので、
両方欠けている場合も片側だけ指定された場合も CLI argument validation error です。
両方が指定された後の `--request-store` path schema violation または
`--request-store-hash` invalid hash format は
`NormalizeErrorResult.error.reason_code = request_store_reference_invalid` です。
この場合 `field` は `request_store.path` または `request_store.manifest_hash` とし、
field shape は 6 の `request_store_reference_invalid` に従います。
`--selector-module` と `--selector-request-hash` は optional pair です。
両方省略した場合は single-artifact convenience mode を使います。
片側だけが指定された場合、または selector module / request hash の schema validation に失敗した場合は、
CLI は partial `artifact_selector` を endpoint input として扱い、
`NormalizeErrorResult.error.reason_code = selector_schema_invalid` を返します。
`npa-check normalize-results --out <path>` は `NormalizedCheckResult` を指定 path に保存します。
`--normalized-store-out <path>` flag が指定された場合だけ、normalized store manifest を更新します。
指定された manifest path の file が既に存在する場合は、既存 manifest を検証してから
output entry を追加し、`normalized_result_hash` order で sort した manifest を
atomic replace で書き戻します。
指定された manifest path の file が存在しない場合は、empty store manifest から開始し、
新しい manifest file を作成します。
`--normalized-store-out` flag が省略された場合は normalized store manifest を読まず、作らず、
更新しません。
`--normalized-store-out` を使う場合、normalized store manifest が commit point です。
`--normalized-store-out` を使わない場合は、final output path の配置完了が commit point です。
実装は output artifact を temporary file として作ります。
store を使う場合は manifest も temporary file として作り、final output path を配置してから
manifest を atomic replace します。
manifest が final output path と `output_file_hash` を参照して初めて store 更新成功です。
manifest commit 前に failure した場合、manifest を更新してはいけません。
temporary file は best-effort で削除します。
manifest に参照されない orphan output file は store reader が無視します。
retry 時に final output path が既に存在し、その file bytes が今回書く
`NormalizedCheckResult` file bytes と完全一致する場合は、上書きではなく既存 file の採用として扱います。
既存 final output path の bytes が異なる場合は `output_path_conflict` です。
既存 manifest に同じ `normalized_result_hash`、`path`、`file_hash` の entry が既にある場合は
idempotent success として扱います。
既存 manifest 内に同じ `normalized_result_hash` または同じ `path` の entry があり、
追加予定 entry の `normalized_result_hash`、`path`、`file_hash` の組と完全一致しない場合は
`normalized_store_entry_conflict` です。
既存 normalized store manifest file を読めない場合、JSON として壊れている場合、
または manifest の schema / order / duplicate 検証に失敗した場合は
`normalized_store_manifest_invalid` です。
manifest entry の `file_hash` と参照先 file bytes hash が一致しない場合は
`normalized_store_entry_file_hash_mismatch` です。
output artifact の temporary write または rename 失敗は `output_write_failure` です。
normalized store manifest の temporary write、rename、atomic replace 失敗は
`normalized_store_write_failure` です。
write-stage `NormalizeErrorResult` の field は固定します。
`output_path_conflict` では `error.field = "output_path"`、
`expected_hash` に今回書く output file hash、`actual_hash` に既存 file bytes hash を入れます。
`normalized_store_entry_conflict` では `error.field = "normalized_store.results[]"`、
`expected_value` に追加予定 entry の canonical JSON string、
`actual_value` に衝突した既存 entry の canonical JSON string を入れます。
`normalized_store_manifest_invalid` では、既存 normalized store manifest file を読めない場合
`error.field = "normalized_store.path"`、`actual_value = "unreadable"` にします。
JSON として壊れている場合は同じ field で `actual_value = "invalid_json"` にします。
schema / order / duplicate 違反では `error.field` に invalid manifest field の JSON path、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_hash_format`、
`invalid_path`、`null_not_allowed`、`order_violation`、`duplicate_normalized_result_hash`、
`duplicate_path`、`duplicate_field` のいずれかを入れます。
この field は caller-prefixed manifest path とし、manifest-local `results[<i>].path` は
`normalized_store.results[<i>].path` として報告します。
manifest schema / domain error の field は concrete index を含む caller-prefixed path に固定し、
`normalized_store_entry_file_hash_mismatch` は下の wildcard path を使います。
manifest entry `path` が workspace-relative path schema に違反する場合は
`normalized_store_manifest_invalid` としてここで止め、entry file は読みに行きません。
`normalized_store_entry_file_hash_mismatch` では `error.field = "normalized_store.results[].file_hash"`、
`expected_hash` に manifest entry の `file_hash`、`actual_hash` に参照先 file bytes hash を入れます。
既存 normalized store manifest 内の複数 entry で `normalized_store_entry_file_hash_mismatch` が
同時に成立する場合は、`results[]` の小さい index を先に報告します。
`output_write_failure` では `error.field = "output_path"`、
`normalized_store_write_failure` では `error.field = "normalized_store.path"` とし、
どちらも `actual_value = "write_failed"` にします。
複数の write-stage 失敗条件が同時にある場合は、
`normalized_store_manifest_invalid`、`normalized_store_entry_file_hash_mismatch`、
`output_path_conflict`、`normalized_store_entry_conflict`、`output_write_failure`、
`normalized_store_write_failure` の順で最初に該当した
`reason_code` を返します。
`--normalized-store-out` を使う場合は `--out` も required です。
`--normalized-store-out` があり `--out` がない invocation は CLI argument validation error であり、
`NormalizeErrorResult` や `NormalizationWriteResult` body を返しません。
`--out` と `--json` を同時指定した成功時 stdout は `NormalizationWriteResult` です。
exact-match adoption や idempotent retry の成功時も `status = written` を返し、
別の `adopted` status は作りません。
`--out` を指定しない `--json` 成功時 stdout は従来どおり `NormalizedCheckResult` です。
normalize pipeline failure と write-stage failure では stdout は `NormalizeErrorResult` で、
`--out` と `--normalized-store-out` は上記の commit rule に従い、未完了の更新を
成功として扱ってはいけません。

MVP の `NormalizationWriteResult`：

```json
{
  "schema": "npa.phase8.normalization_write_result.v1",
  "status": "written",
  "normalized_result_hash": "sha256:...",
  "artifact_hash": "sha256:...",
  "output_path": "build/normalized/Std.Nat.json",
  "output_file_hash": "sha256:...",
  "normalized_store": {
    "kind": "manifest",
    "path": "build/normalized/manifest.json",
    "manifest_hash": "sha256:..."
  }
}
```

`normalized_store` は `--normalized-store-out` を指定した場合だけ required です。
`output_file_hash` は保存した `NormalizedCheckResult` file bytes の sha256 です。
`NormalizationWriteResult` は transient response であり、`result_hash` を持ちません。
`npa-check compare` は `NormalizedCheckResult` と同じ `RunnerPolicyReference` を必要とするため、
CLI では `--policy` と `--policy-hash` が required です。
`npa-check auxiliary axiom-policy`、`npa-check auxiliary reproducibility`、
`npa-check auxiliary import-certificate-hash` は 12 の `AuxiliaryResult` を生成する deterministic command です。
成功時 stdout は保存された、または `--out` なしなら inline の `AuxiliaryResult` です。
`--out` 指定時は output file bytes hash を含む write summary を返さず、
書き込んだ `AuxiliaryResult` 自体を stdout に返します。
write failure は CLI/API pipeline failure であり、`AuxiliaryResult.status = inconclusive` に変換してはいけません。
`AuxiliaryResult.status` は oracle の評価結果だけを表します。

`npa-check auxiliary axiom-policy` の required input は `--policy` / `--policy-hash`、
`--normalized-result` / `--normalized-result-hash`、
`--axiom-report-store` / `--axiom-report-store-hash`、および `--json` です。
`npa-check auxiliary reproducibility` の required input は `--policy` / `--policy-hash`、
`--baseline-run-artifact-hash`、`--repeated-run-artifact-hash`、
`--result-store` / `--result-store-hash`、および `--json` です。
missing required flag、duplicate singleton flag、unsupported flag、missing `--json` は
CLI argument validation error であり、`CommandError` body を返しません。
`--policy` / `--policy-hash` の validation は deterministic command 共通の
`policy_reference_invalid` / `policy_file_unreadable` / `policy_hash_mismatch` field shape を使います。
non-policy path/hash pair が片側指定、path schema violation、hash format violation の場合は
`CommandError.reason_code = input_reference_invalid` です。

```text
auxiliary axiom-policy non-policy reference fields:
  --normalized-result:
    path field = "normalized_result.path"
    hash field = "normalized_result.normalized_result_hash"
    hash meaning = validated NormalizedCheckResult.normalized_result_hash after
                   artifact_hash and normalized_result_hash self-hash checks
  --axiom-report-store:
    path field = "axiom_report_store.path"
    hash field = "axiom_report_store.manifest_hash"
    hash meaning = AxiomReportStoreManifest file bytes sha256

auxiliary reproducibility non-policy reference fields:
  --result-store:
    path field = "result_store.path"
    hash field = "result_store.manifest_hash"
    hash meaning = machine result store manifest file bytes sha256
  --baseline-run-artifact-hash:
    field = "selector.baseline_run_artifact_hash"
    expected_value = "sha256:<lower-hex>"
  --repeated-run-artifact-hash:
    field = "selector.repeated_run_artifact_hash"
    expected_value = "sha256:<lower-hex>"
```

片側指定では missing path field または missing hash field に
`expected_value = "required"`、`actual_value = "missing"` を入れます。
path schema violation では path field に `expected_value = "workspace_relative_path"`、
`actual_value = "invalid_path"` を入れます。
hash format violation では hash field に `expected_value = "sha256:<lower-hex>"`、
`actual_value = "invalid_hash_format"` を入れます。
`--baseline-run-artifact-hash` と `--repeated-run-artifact-hash` が同じ値の場合は
`CommandError.reason_code = input_reference_invalid`、
`field = "selector.repeated_run_artifact_hash"`、
`expected_value = "distinct_run_artifact_hash"`、`actual_value = "duplicate"` とします。

`--normalized-result` file unreadable は `input_file_unreadable`、
JSON parse failure は `input_json_invalid`、schema / domain failure は `input_schema_invalid`、
artifact_hash / normalized_result_hash self-hash mismatch と caller hash mismatch は
`input_hash_mismatch` です。
このとき field はそれぞれ `normalized_result.path`、
`normalized_result.path`、`normalized_result.<JSON path>`、該当 hash field です。
file unreadable では `expected_value = "readable_file"`、`actual_value = "unreadable"`、
JSON parse failure では `expected_value = "valid_json"`、`actual_value = "invalid_json"`、
schema / domain failure では `expected_value = <schema requirement name>`、
`actual_value = missing | wrong_type | unknown_field | invalid_enum |
invalid_hash_format | invalid_name_format | null_not_allowed |
order_violation | duplicate_field | failure_key_mismatch` のいずれかを入れます。
`artifact_hash` self-hash mismatch では `field = "normalized_result.artifact_hash"`、
`expected_hash` に再計算した `NormalizedCheckResult.artifact_hash`、
`actual_hash` に parsed `NormalizedCheckResult.artifact_hash` を入れます。
`normalized_result_hash` self-hash mismatch では
`field = "normalized_result.normalized_result_hash"`、
`expected_hash` に再計算した `NormalizedCheckResult.normalized_result_hash`、
`actual_hash` に parsed `NormalizedCheckResult.normalized_result_hash` を入れます。
caller supplied `--normalized-result-hash` との mismatch では
`field = "normalized_result.normalized_result_hash"`、
`expected_hash` に caller supplied hash、
`actual_hash` に validated `NormalizedCheckResult.normalized_result_hash` を入れます。
この top-level input mismatch は `CommandError` であり、CLI は `AuxiliaryResult` を生成しません。
CLI が生成する selector は validated `NormalizedCheckResult.normalized_result_hash` を写すため、
通常の CLI 実行では `selector.normalized_result_hash` mismatch の oracle failure は発生しません。

`--axiom-report-store` / `--result-store` manifest file unreadable は
`input_file_unreadable`、manifest file hash mismatch は `input_hash_mismatch`、
manifest JSON parse / schema / order / duplicate failure は `input_store_manifest_invalid` です。
file unreadable では store path field、hash mismatch では store hash field を使い、
file unreadable では `expected_value = "readable_file"`、`actual_value = "unreadable"`、
hash mismatch では `expected_hash` は caller supplied hash、
`actual_hash` は manifest file bytes sha256 です。
JSON parse failure では store path field に `expected_value = "valid_json"`、
`actual_value = "invalid_json"` を入れます。
schema / order / duplicate failure では `field = "<store>.<JSON path>"`、
`expected_value = store manifest schema requirement`、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_enum`、
`invalid_hash_format`、`invalid_path`、`null_not_allowed`、`order_violation`、
`duplicate_field`、`duplicate_path` のいずれかを入れます。
`axiom_report_store` では `duplicate_axiom_report_hash` も許可し、
`result_store` では `duplicate_run_artifact_hash` も許可します。
ここで `<store>` は `axiom_report_store` または `result_store` です。
valid store manifest から selector が指す artifact を解決できない場合や、
selector が指す artifact file を読めない場合は command input validation ではなく
上の oracle-specific `*_inconclusive` field shape で `AuxiliaryResult` に記録します。

`npa-check challenge coverage-summary` は filtered `ChallengeOutputStoreManifest`、
`ChallengeReplayResult` store、machine result store から `ChallengeCoverageSummary` を生成します。
coverage target は `--target-normalized-result` / `--target-normalized-result-hash` で明示します。
生成した summary は target normalized result hash と machine result store manifest hash を
`target_normalized_result_hash` / `result_store_manifest_hash` として保存します。
この command は target normalized result、challenge manifest、replay result、machine result、
coverage target の cross-artifact reference を release audit bundle の closed-set rule で検査し、
coverage summary generation failure では `ChallengeCoverageSummary` を出力しません。
`unexpected_acceptances` は、各 referenced `ChallengeReplayResult.checker_results[*].run_artifact_hash`
を machine result store から解決し、required checker profile の
`MachineCheckResult.status = checked` を再計算して数えます。
machine result store が missing / ambiguous / hash mismatch の場合は
`CommandError.reason_code = coverage_summary_generation_failed` で、保存済み
`ChallengeReplayResult.comparison_status` だけから `unexpected_acceptances` を計算してはいけません。
成功時 stdout は保存された、または `--out` なしなら inline の `ChallengeCoverageSummary` です。

coverage-summary command の required input は `--policy` / `--policy-hash`、
`--artifact-hash`、`--target-normalized-result` / `--target-normalized-result-hash`、
`--challenge-store` / `--challenge-store-hash`、
`--replay-store` / `--replay-store-hash`、
`--result-store` / `--result-store-hash`、および `--json` です。
missing required flag、duplicate singleton flag、unsupported flag は CLI argument validation error であり、
`CommandError` body を返しません。
path/hash pair が完全に欠けている場合は missing required flag として CLI argument validation error です。
path/hash pair の片方だけがある場合、または path schema violation は
`CommandError.reason_code = input_reference_invalid` です。
`field` は `target_normalized_result.path`、`challenge_store.path`、`replay_store.path`、
`result_store.path`、または該当 hash field とし、
path schema violation の `expected_value = "workspace_relative_path"`、
`actual_value = "invalid_path"` とします。
non-policy hash flag の invalid hash format も `input_reference_invalid` で、
`field` は `artifact_hash`、`target_normalized_result.normalized_result_hash`、
`challenge_store.manifest_hash`、`replay_store.manifest_hash`、
または `result_store.manifest_hash`、
`expected_value = "sha256:<lower-hex>"`、`actual_value = "invalid_hash_format"` とします。
policy reference failure は challenge 系 command と同じ `policy_reference_invalid` /
`policy_file_unreadable` / `policy_hash_mismatch` の field shape を使います。
top-level input file を読めない場合は `input_file_unreadable` で、
`field = "<input>.path"`、`actual_value = "unreadable"` とします。
target normalized result の JSON parse failure は `input_json_invalid`、
schema / domain validation failure は `input_schema_invalid` です。
`challenge_store`、`replay_store`、`result_store` の JSON parse / schema / order / duplicate failure は
`input_store_manifest_invalid` で、`field = "<store>.<json_path>"`、
JSON parse failure では `field = "<store>.path"`、`actual_value = "invalid_json"` とします。
top-level hash mismatch は `input_hash_mismatch` で、field は
`target_normalized_result.normalized_result_hash`、
`challenge_store.manifest_hash`、`replay_store.manifest_hash`、
`result_store.manifest_hash`、または `artifact_hash` です。
`expected_hash` は caller supplied hash、`actual_hash` は parsed hash または file bytes sha256 です。
`--artifact-hash` は parsed target `NormalizedCheckResult.artifact_hash` と比較します。
store entry が参照する challenge manifest、replay result、machine result の missing / ambiguous /
entry file unreadable / entry JSON parse failure / entry schema or domain failure /
entry hash mismatch / parsed artifact mismatch、および coverage target scope mismatch は
cross-artifact coverage semantics の失敗なので
`CommandError.reason_code = coverage_summary_generation_failed`、
`field = "command"`、`actual_value = "coverage_summary_generation_failed"` とします。
ただし coverage-required replay result の malformed / missing `normalized_result_hash`、
malformed / missing `comparison_status`、および `normalized_result_hash` absent 時の
forbidden `comparison_status` presence について上で個別 field shape を固定している場合は、
その `replay_store.results[].*` field shape を使います。
`--out` を指定した場合、path schema violation は `input_reference_invalid`、
既存 output path の bytes が異なる場合は `output_path_conflict`、
temporary write / atomic replace failure は `output_write_failure` です。
`--out` path schema violation では `field = "out.path"`、
`expected_value = "workspace_relative_path"`、`actual_value = "invalid_path"` とします。
`output_path_conflict` では `field = "out.path"`、
`expected_hash` に今回生成する `ChallengeCoverageSummary` file bytes hash、
`actual_hash` に既存 file bytes hash を入れます。
`output_write_failure` では `field = "out.path"`、`actual_value = "write_failed"` とします。
exact-match adoption は成功として扱い、別 status は作りません。

`npa-check release bundle` は `ReleasePolicy`、release target `artifact_hash`、
store manifest、required auxiliary result、challenge coverage summary、および optional sidecar / validation response を
release audit bundle の closed-set rule に従って検査し、`ReleaseAuditBundleManifest` を出力します。
`bundle-root` は output placement root であり、input discovery root ではありません。
`--out` は `<bundle-root>/manifest.json` に固定します。
`--bundle-root` と `--out` は workspace-relative path schema の対象です。
`--bundle-root` の path schema violation は `input_reference_invalid` で、
`field = "bundle_root.path"`、`expected_value = "workspace_relative_path"`、
`actual_value = "invalid_path"` とします。
`--out` の path schema violation、または `--out` が `bundle-root` の外、
または `manifest.json` 以外を指す場合は
`CommandError.reason_code = input_reference_invalid` です。
このとき `field = "out.path"`、`expected_value = "<bundle-root>/manifest.json"`、
`actual_value = "invalid_path" | "outside_bundle_root" | "not_manifest_json"` とします。
release bundle command は explicit input file を discovery しません。
input files は pre-bundle staging step によって `bundle-root/artifacts/...` の deterministic path に
配置済みでなければなりません。
command は `ReleaseAuditBundleManifest` を `--out` に atomic write しますが、
artifact file path の rewrite、implicit copy、store manifest merge は行いません。
`--out` が既に存在し、file bytes が今回生成する manifest bytes と完全一致する場合は
exact-match adoption として成功扱いします。
既存 `--out` の bytes が異なる場合は `output_path_conflict` で、
`field = "out.path"`、`expected_hash` に今回生成する manifest file bytes hash、
`actual_hash` に既存 file bytes hash を入れます。
temporary write / atomic replace failure は `output_write_failure` で、
`field = "out.path"`、`actual_value = "write_failed"` とします。
`--json` 成功時 stdout は保存された `ReleaseAuditBundleManifest` です。
各 explicit input path は `bundle-root/artifacts/<kind>/<file_hash_without_sha256_prefix>.json` の形で
`bundle-root` 配下に存在しなければなりません。
command は manifest には `bundle-root` からの相対 path だけを記録します。
explicit input path が `bundle-root` 外、kind と一致しない directory、または
`<64 lower-hex>.json` 以外の filename を指す場合は
`CommandError.reason_code = input_reference_invalid` です。
path string が bundle-local artifact path shape を満たしていても、`bundle-root` から1回解決した
IO path が symlink escape などで bundle root 外を指す場合は、file readable failure ではなく
Step 2 の `input_reference_invalid` です。
Step 2 の field shape は input flag field table の path field を使い、
`expected_value = "bundle_artifact_path:<kind>"` とします。
`actual_value` は、path string が `bundle-root` 外を指す場合、または解決後 IO path が
symlink escape などで bundle root 外を指す場合は `invalid_path`、
expected kind directory と一致しない場合は `kind_mismatch`、
filename が `<64 lower-hex>.json` でない場合は `invalid_filename` とします。
同じ explicit input path で Step 2 の複数 subcondition が同時に成立する場合は、
`invalid_path`、`kind_mismatch`、`invalid_filename` の順で最初の failure を返します。
filename が表す hash と referenced file bytes sha256 が一致しない場合は
`CommandError.reason_code = input_hash_mismatch` です。
release bundle command は、release policy、runner policy、challenge runner policy、target normalized result、
request / machine result / normalized store manifest、challenge output store manifest、
challenge coverage summary、challenge replay result、auxiliary result、checker identity manifest、import lock、
AI audit input policy、
optional AI sidecar / validation response、optional compare validation response を
すべて explicit path + expected hash flag で受け取ります。
`ReleasePolicy` を読まずに静的に non-optional と分かる singleton input pair は、
closed set 上で同じ artifact に deduplicate される場合でも
flag としては required です。
repeatable input flags such as `--auxiliary-result`、`--challenge-replay-result`、
`--checker-identity-manifest`、`--import-lock`、`--ai-sidecar`、`--audit-sidecar-validation`、
`--compare-validation-response` are paired with their corresponding `--*-hash` flag
by occurrence order.
path without hash、hash without path、or unequal pair counts are `CommandError.reason_code = input_reference_invalid`。
静的に required な flag の欠落、duplicate singleton flag、unsupported flag、missing `--json` は
CLI argument validation error であり、`CommandError` body を返しません。
ここでいう静的に required な flag は `ReleasePolicy` を読まなくても required と分かる flag だけです。
`--ai-audit-input-policy` / `--ai-audit-input-policy-hash` の conditional required / forbidden は
`ReleasePolicy.ai_triage.enabled` に依存するため CLI argument validation error ではなく、
Step 8 の closed set / cross-artifact validation で判定します。
明示された input set が release audit bundle の closed set と一致しない場合は
`CommandError.reason_code = release_bundle_generation_failed` で、partial bundle を成功扱いしてはいけません。
directory scan、glob、bundle-root からの暗黙発見、policy hash からの store lookup は forbidden です。

release bundle command の explicit input validation order は次で固定します。

```text
1. pair shape / provided path schema / provided hash schema:
   reason_code = input_reference_invalid
   field = input flag field table の path field / hash field / pair field
   expected_value = "workspace_relative_path" | "sha256:<lower-hex>" | "required_pair"
   actual_value = invalid_path | invalid_hash_format | missing_pair | unequal_pair_count

2. bundle-local artifact path shape:
   reason_code = input_reference_invalid
   field = input flag field table の path field
   expected_value = "bundle_artifact_path:<kind>"
   actual_value = invalid_path | kind_mismatch | invalid_filename

3. file readable:
   reason_code = input_file_unreadable
   field = input flag field table の path field
   actual_value = "unreadable"

4. filename hash vs file bytes:
   reason_code = input_hash_mismatch
   field = input flag field table の path field
   expected_hash = "sha256:<filename_without_json>"
   actual_hash = referenced file bytes sha256

5. JSON parse:
   reason_code = input_json_invalid
   field = input flag field table の path field
   actual_value = "invalid_json"
   store manifest JSON parse failure uses input_store_manifest_invalid
   with field = input flag field table の path field and actual_value = "invalid_json"

6. artifact schema / domain validation:
   reason_code = input_schema_invalid
   field = input flag field table の artifact field
   store manifest schema / order / duplicate failure uses input_store_manifest_invalid
   with field = input flag field table の artifact field

7. caller hash flag vs parsed artifact hash or file bytes hash:
   reason_code = input_hash_mismatch
   field = input flag field table の hash field
   expected_hash = caller supplied hash
   actual_hash = parsed artifact hash or referenced file bytes sha256

8. closed set / cross-artifact validation:
   reason_code = release_bundle_generation_failed
   field = "command"
   actual_value = "release_bundle_generation_failed"
```

Step 1 の内部優先順位は、pair shape、provided path schema、provided hash schema の順です。
同じ flag pair で path schema violation と hash format violation が同時に成立する場合は、
path schema violation を先に報告します。
path without hash では input flag field table の hash field、
hash without path では input flag field table の path field、
どちらも `expected_value = "required_pair"`、`actual_value = "missing_pair"` とします。
repeatable pair の path / hash count が一致しない場合は input flag field table の pair field、
`expected_value = "required_pair"`、`actual_value = "unequal_pair_count"` とします。
複数の input で Step 1 failure が同時に成立する場合は、下の input flag field table の順で
対応する input を選びます。
repeatable flag pair では同じ pair kind 内の occurrence order で最初の failure を返します。
standalone hash flag の `--artifact-hash` には pair shape と path schema validation は適用せず、
hash format violation だけを Step 1 で扱い、`field = "artifact_hash"` とします。
`--artifact-hash` の hash format violation と flag pair failure が同時に成立する場合も、
input flag field table の順で最初の failure を返します。
Step 2 から Step 7 でも、同じ Step の failure が複数 input で同時に成立する場合は
input flag field table の順で対応する input を選びます。
repeatable flag pair では同じ pair kind 内の occurrence order で最初の failure を返します。
Step order は input order より優先します。
したがって任意の input の Step 2 failure は、別 input の Step 3-7 failure より先に報告します。

Step 7 の actual hash は、下の input flag field table で「parsed」と書かれた flag では
parsed artifact canonical hash、「file bytes」と書かれた flag では referenced file bytes sha256 です。
Step 4 は常に deterministic artifact path の filename と file bytes sha256 の照合です。
したがって parsed hash flag が正しくても filename hash が file bytes と違う input は Step 4 で失敗します。

Step 8 の内部優先順位は次で固定します。
この順序は `release_bundle_generation_failed` 内の tie-break order であり、
Step 1-7 の input validation failure より後にだけ適用します。

```text
1. shared runner policy path rule:
   ReleasePolicy.runner_policy_hash と challenge_runner_policy_hash が同じなのに
   --runner-policy と --challenge-runner-policy が異なる bundle-local path を指す場合。
   field = "runner_policy"

2. ai audit input policy conditional gating:
   ReleasePolicy.ai_triage.enabled に対する --ai-audit-input-policy pair の required / forbidden。
   field = "ai_audit_input_policy"

3. optional response / sidecar status gating:
   included CompareValidationResult.status != valid、
   optional AiAuditSidecar に対応する AuditSidecarValidationResult がない / 複数 / status != valid。
   CompareValidationResult.status failure の artifact kind は compare_validation_response。
   optional AiAuditSidecar に対応する response が0件または複数件の場合の artifact kind は ai_audit_sidecar。
   対応する response が1件で AuditSidecarValidationResult.status != valid の場合の artifact kind は
   audit_sidecar_validation_response。
   ReleasePolicy.ai_triage.enabled = false なのに ai_audit_sidecar または
   audit_sidecar_validation_response が存在する場合は、この class ではなく class 4 の
   forbidden artifact kind として扱う。

4. release audit bundle closed-set cardinality:
   required artifact kind missing、duplicate、extra / forbidden artifact kind。

5. cross-artifact identity / policy / mode / status / recomputation mismatch:
   policy hash、runner trust mode、store entry reference、coverage summary、
   normalized comparison recomputation、auxiliary selector、required auxiliary_result status、
   compare response recomputation、audit-sidecar recomputation などの不一致。
```

Step 8 の同じ class 内で複数 failure が成立する場合は、
release bundle artifact kind order で最初の kind を返します。
この順序は explicit input flag の順序から導出せず、次で固定します。

```text
release bundle artifact kind order:
  1. release_policy
  2. runner_policy
  3. checker_identity_manifest
  4. import_lock
  5. request_store_manifest
  6. machine_result_store_manifest
  7. normalized_result_store_manifest
  8. challenge_output_store_manifest
  9. machine_check_request
  10. machine_check_result
  11. normalized_check_result
  12. challenge_manifest
  13. challenge_replay_result
  14. challenge_coverage_summary
  15. auxiliary_result
  16. ai_audit_input_policy
  17. ai_audit_sidecar
  18. audit_sidecar_validation_response
  19. compare_validation_response
```

ただし下流 artifact kind の expected set が別 artifact kind の内容から導出される場合、
前提 artifact が prerequisite-clean になるまで、その下流 kind の
missing / forbidden / duplicate failure を合成してはいけません。
この prerequisite gate は、依存する下流 kind の class 4 closed-set failure を作る場面だけに適用します。
Step 8 class 1-3 の failure、およびその下流 kind に依存しない他 artifact kind の failure は、
上の Step 8 class order と artifact kind order のまま判定します。
前提 artifact が prerequisite-clean でないため下流 failure を作れない場合は、前提 kind の failure を返します。
その failure は原因に応じて class 4 cardinality failure または class 5 identity / source-key failure です。
複数 prerequisite が同時に prerequisite-clean でない場合は、下の gate table の `requires` に書かれた順で評価し、
各 prerequisite について再帰的にその prerequisite 自身の prerequisite gate を適用します。
同じ prerequisite kind 内の複数 failure は、通常の Step 8 class order、
artifact kind order、expected artifact identity key order で最初の failure を選びます。
prerequisite-clean は「下流 expected set の key material を bundle input だけから一意に抽出できる状態」を指し、
すべての cross-artifact semantic validation が完了していることは意味しません。
MVP の prerequisite gate は次で固定します。

```text
challenge_manifest:
  requires prerequisite-clean challenge_output_store_manifest.
  challenge_output_store_manifest is prerequisite-clean for challenge_manifest iff:
    - exactly one challenge_output_store_manifest entry exists,
    - hashes.manifest_hash matches the referenced ChallengeOutputStoreManifest file bytes,
    - the referenced ChallengeOutputStoreManifest is schema-valid, sorted, and unique,
    - every entries[*].manifest_hash is syntactically valid and extractable.
  This challenge_output_store_manifest prerequisite does not require referenced
  ChallengeManifest files to be present or valid; those failures belong to
  challenge_manifest closed-set or cross-artifact validation.
  challenge_manifest set is prerequisite-clean iff:
    - challenge_output_store_manifest is prerequisite-clean for challenge_manifest,
    - the set exactly matches ChallengeOutputStoreManifest.entries[*].manifest_hash,
    - every included challenge_manifest artifact entry has hashes.manifest_hash
      matching its referenced ChallengeManifest file bytes,
    - every included ChallengeManifest is manifest-local JSON / schema / domain valid
      as defined in 10, including mutation.kind classification and mutation.target validation
      according to kind,
    - every included ChallengeManifest.mutation.kind is a Phase 8 MVP rejection-required kind,
    - every included ChallengeManifest.imports.manifest_hash is syntactically valid and extractable.

normalized_check_result:
  Its release target closed-set cardinality, exactly one normalized_check_result entry
  whose artifact_hash equals ReleaseAuditBundleManifest.artifact_hash, is evaluated by
  normal Step 8 class / artifact kind order and does not require challenge_replay_result.
  Challenge replay normalized result entries require prerequisites in this order:
    1. prerequisite-clean release target normalized_check_result for normalized_check_result,
    2. prerequisite-clean challenge_coverage_summary for normalized_check_result,
    3. prerequisite-clean challenge_replay_result set for normalized_check_result.
  Release target normalized_check_result is prerequisite-clean for normalized_check_result iff:
    - exactly one release target normalized_check_result entry exists,
    - its normalized_result_hash and artifact.import_lock_hash are syntactically valid and extractable,
    - embedded comparison recomputation succeeds and the recomputed comparison.status is all_agree_checked.
  challenge_coverage_summary is prerequisite-clean for normalized_check_result iff:
    - exactly one challenge_coverage_summary entry exists,
    - artifact entry hashes.summary_hash, parsed ChallengeCoverageSummary.summary_hash,
      recomputed ChallengeCoverageSummary.summary_hash, and derived summary_id all match,
    - entries are schema-valid and sorted by (challenge_id, manifest_hash),
    - entries are unique by both (challenge_id, manifest_hash) and replay_result_hash,
    - every entries[*].replay_result_hash is syntactically valid and extractable.
  challenge_replay_result set is prerequisite-clean for normalized_check_result iff:
    - challenge_coverage_summary is prerequisite-clean for normalized_check_result,
    - the set exactly matches ChallengeCoverageSummary.entries[*].replay_result_hash
      using the challenge_replay_result expected artifact identity key,
    - every included ChallengeReplayResult has result_hash self-consistency:
      artifact entry hashes.result_hash when validating an existing ReleaseAuditBundleManifest,
      parsed ChallengeReplayResult.result_hash, and recomputed ChallengeReplayResult.result_hash
      all match. During `npa-check release bundle` generation, caller supplied
      `--challenge-replay-result-hash` vs parsed result_hash has already been checked in Step 7,
      so this prerequisite only needs parsed vs recomputed result_hash self-consistency.
    - each ChallengeReplayResult is matched to the unique summary entry whose
      replay_result_hash equals the challenge_replay_result expected artifact identity key,
    - normalized_result_hash is schema-valid and extractable for every included
      ChallengeReplayResult,
    - comparison_status has passed Step 6 artifact schema / domain validation
      for every included ChallengeReplayResult whose normalized_result_hash is present,
    - every included ChallengeReplayResult.normalized_result_hash is bytewise distinct
      from the release target NormalizedCheckResult.normalized_result_hash.
    A matched included ChallengeReplayResult that omits normalized_result_hash after Step 6
    is not a Step 6 schema failure and is not treated as an informational forbidden artifact.
    It is a challenge_replay_result class 5 source-key failure with
    field = "challenge_replay_result[<i>].artifact.normalized_result_hash",
    expected_value = "required_for_release_coverage", and actual_value = "missing".
    Extra challenge_replay_result inputs outside ChallengeCoverageSummary.entries[] remain
    normal class 4 extra artifacts and are reported before checking their normalized_result_hash.
    Because challenge_coverage_summary prerequisite-clean already requires unique
    entries[*].replay_result_hash and the challenge_replay_result set exactness check is class 4,
    missing or non-unique summary bindings are reported as either
    challenge_coverage_summary schema / domain failures or challenge_replay_result class 4
    missing / extra / duplicate failures. Do not synthesize a separate
    challenge_replay_result class 5 coverage summary entry binding failure.
    Malformed or missing ChallengeReplayResult.comparison_status when
    normalized_result_hash is present is a Step 6 input_schema_invalid /
    replay_store_entry_schema_invalid failure, not a Step 8 status binding mismatch.
    A ChallengeReplayResult that points at the release target normalized_result_hash is
    a challenge_replay_result class 5 source-key failure; do not deduplicate it with
    the release target normalized_check_result entry.
    This collision failure uses
    field = "challenge_replay_result[<i>].artifact.normalized_result_hash",
    expected_value = "distinct_from_release_target_normalized_result_hash",
    and actual_value = "matches_release_target_normalized_result_hash".
    Within the same matched included ChallengeReplayResult, multiple
    challenge_replay_result class 5 prerequisite failures are reported in this order:
      1. result_hash self-consistency failure
         field = "challenge_replay_result[<i>].artifact.result_hash",
         if artifact entry hashes.result_hash exists and differs from parsed
         ChallengeReplayResult.result_hash, expected_hash = artifact entry hashes.result_hash
         and actual_hash = parsed ChallengeReplayResult.result_hash. Otherwise
         expected_hash = recomputed ChallengeReplayResult.result_hash and
         actual_hash = parsed ChallengeReplayResult.result_hash.
      2. normalized_result_hash missing for release coverage
         field = "challenge_replay_result[<i>].artifact.normalized_result_hash",
         expected_value = "required_for_release_coverage",
         actual_value = "missing"
      3. normalized_result_hash collision with the release target
         field = "challenge_replay_result[<i>].artifact.normalized_result_hash",
         expected_value = "distinct_from_release_target_normalized_result_hash",
         actual_value = "matches_release_target_normalized_result_hash"
    Replay-to-normalized comparison_status mismatch is evaluated later as part of
    normalized_check_result set prerequisite-clean validation and is not included in
    this challenge_replay_result source-key priority.
    Multiple ChallengeReplayResult entries may point at the same non-release
    normalized_result_hash. This is not a duplicate failure; the downstream
    normalized_check_result expected set uses the distinct normalized_result_hash set.
  normalized_check_result set is prerequisite-clean iff:
    - release target normalized_check_result is prerequisite-clean for normalized_check_result,
    - challenge_replay_result set is prerequisite-clean for normalized_check_result,
    - for every distinct included ChallengeReplayResult.normalized_result_hash there is
      exactly one matching normalized_check_result entry,
    - no normalized_check_result entry exists outside the release target entry and
      ChallengeReplayResult.normalized_result_hash set,
    - every included normalized_check_result entry has normalized_result_hash and
      artifact.import_lock_hash syntactically valid and extractable,
    - after the closed set above is exact, every allowed included normalized_check_result
      entry is classified as either release target or challenge replay target,
    - embedded comparison recomputation succeeds for every allowed included
      normalized_check_result entry using the target-specific RunnerPolicy,
    - every included ChallengeReplayResult.comparison_status matches the recomputed
      comparison.status of its matching challenge replay normalized_check_result,
    - every ChallengeCoverageSummary.entries[].comparison_status matches the
      ChallengeReplayResult.comparison_status selected by that entry's replay_result_hash.
    Extra / forbidden normalized_check_result entries are class 4 closed-set failures
    and are reported before target-specific policy selection or comparison recomputation
    for those entries. A challenge replay normalized_check_result recomputation failure
    makes the normalized_check_result set non-clean as a normalized_check_result class 5
    prerequisite failure before downstream expected sets such as import_lock are derived.
    The status binding failures above make the normalized_check_result set non-clean
    while preserving their source artifact kind: challenge_replay_result class 5 for
    replay-to-normalized status mismatch, and challenge_coverage_summary class 5 for
    summary-to-replay status mismatch.

challenge_coverage_summary:
  Its own closed-set cardinality, exactly one challenge_coverage_summary entry,
  is evaluated by normal Step 8 class / artifact kind order and does not require
  base prerequisites.
  Source-key validation requires prerequisite-clean release_policy,
  release target normalized_check_result for challenge_coverage_summary,
  challenge_output_store_manifest,
  and machine_result_store_manifest.
  Base prerequisites are prerequisite-clean iff:
    - release_policy:
      exactly one release_policy entry exists and its policy_hash matches
      ReleaseAuditBundleManifest.policy_hash.
    - release target normalized_check_result for challenge_coverage_summary:
      exactly one normalized_check_result entry exists with artifact_hash equal to
      ReleaseAuditBundleManifest.artifact_hash, its normalized_result_hash is extractable,
      embedded comparison recomputation succeeds, and the recomputed comparison.status is all_agree_checked.
    - challenge_output_store_manifest:
      exactly one challenge_output_store_manifest entry exists and its manifest_hash is extractable.
    - machine_result_store_manifest:
      exactly one machine_result_store_manifest entry exists and its manifest_hash is extractable.
  If any base prerequisite is not prerequisite-clean, report that base prerequisite failure
  before deriving challenge_coverage_summary source-key failures.

challenge_replay_result:
  requires prerequisite-clean challenge_coverage_summary for challenge_replay_result.
  challenge_coverage_summary is prerequisite-clean for challenge_replay_result iff:
    - exactly one challenge_coverage_summary entry exists,
    - hashes.summary_hash, parsed summary_hash, recomputed summary hash, and derived summary_id match,
    - policy_hash, artifact_hash, target_normalized_result_hash, challenge_store_manifest_hash,
      and result_store_manifest_hash match the included release policy, target normalized result,
      challenge_output_store_manifest, and machine_result_store_manifest,
    - entries are schema-valid, sorted by (challenge_id, manifest_hash),
      unique by both (challenge_id, manifest_hash) and replay_result_hash,
      and every replay_result_hash is syntactically valid.

machine_check_result:
  requires prerequisite-clean release target normalized_check_result for machine_check_result,
  prerequisite-clean challenge_replay_result set for machine_check_result, and
  prerequisite-clean reproducibility auxiliary_result set.
  release target normalized_check_result is prerequisite-clean for machine_check_result iff:
    - exactly one release target normalized_check_result entry exists,
    - its normalized_result_hash is syntactically valid and extractable,
    - embedded comparison recomputation succeeds and the recomputed comparison.status is all_agree_checked,
    - every results[*] entry has schema-valid and extractable result_hash,
      request_hash, policy_hash, and checker_profile,
    - exactly one results[*] entry has checker_profile equal to
      RunnerPolicy.required_checker_profiles[0].
    The baseline result entry is defined only after all results[*] key material above is
    extractable. Entries whose checker_profile is not equal to
    RunnerPolicy.required_checker_profiles[0] are non-baseline entries. Missing,
    wrong type, null, invalid hash/name format, or other artifact schema / domain
    violations in any results[*] key field are Step 6 input_schema_invalid failures,
    not Step 8 prerequisite failures. After Step 6 has accepted the artifact,
    zero or multiple entries with checker_profile equal to
    RunnerPolicy.required_checker_profiles[0] make this prerequisite non-clean as
    a normalized_check_result class 5 identity / source-key failure.
  challenge_replay_result set is prerequisite-clean for machine_check_result iff:
    - challenge_coverage_summary is prerequisite-clean for challenge_replay_result,
    - the set exactly matches ChallengeCoverageSummary.entries[*].replay_result_hash
      using the challenge_replay_result expected artifact identity key,
    - every included ChallengeReplayResult has result_hash self-consistency:
      artifact entry hashes.result_hash when validating an existing ReleaseAuditBundleManifest,
      parsed ChallengeReplayResult.result_hash, and recomputed ChallengeReplayResult.result_hash
      all match. During `npa-check release bundle` generation, caller supplied
      `--challenge-replay-result-hash` vs parsed result_hash has already been checked in Step 7,
      so this prerequisite only needs parsed vs recomputed result_hash self-consistency.
    - each ChallengeReplayResult is matched to the unique summary entry whose
      replay_result_hash equals the challenge_replay_result expected artifact identity key,
    - ChallengeReplayResult.challenge_id and ChallengeReplayResult.manifest_hash
      match that unique summary entry,
    - ChallengeReplayResult.policy_hash matches top-level ChallengeCoverageSummary.policy_hash,
    - checker_results[*].run_artifact_hash values are schema-valid and extractable.
    This machine_check_result prerequisite does not require
    ChallengeReplayResult.normalized_result_hash extraction; that field is only
    required by challenge_replay_result set for normalized_check_result.
  reproducibility auxiliary_result set is prerequisite-clean iff:
    - exactly one reproducibility auxiliary_result entry exists in the required slot
      keyed by
      kind = reproducibility,
      policy_hash = ReleasePolicy.runner_policy_hash,
      artifact_hash = release target NormalizedCheckResult.artifact_hash,
    - that entry has required selector presence and selector schema validity,
    - selector.request_hash and selector.checker_profile match the release target
      result entry for runner RunnerPolicy.required_checker_profiles[0],
    - selector.baseline_run_artifact_hash and selector.repeated_run_artifact_hash
      are syntactically valid, extractable, and bytewise distinct,
    - no other reproducibility auxiliary_result entry exists in that required slot,
    - no reproducibility auxiliary_result entry exists outside that required slot,
    - the required entry has status = passed,
    Selector target existence is not required for reproducibility auxiliary_result set
    prerequisite-clean. Missing machine_check_result entries for valid / extractable
    baseline_run_artifact_hash or repeated_run_artifact_hash are downstream
    machine_check_result class 4 missing failures. After referenced MachineCheckResult
    entries exist, selector target request_hash / checker_profile / result_hash equality
    remains class 5. If baseline_run_artifact_hash and repeated_run_artifact_hash are
    bytewise equal, report auxiliary_result class 5 selector identity mismatch before
    deriving the machine_check_result allowed run set.
  machine_check_result set is prerequisite-clean iff:
    - the set exactly matches the allowed run set,
    - release target raw result selection has no non-baseline ambiguity,
    - every included MachineCheckResult has result_hash and run_artifact_hash self-consistency:
      artifact entry hashes.result_hash, parsed MachineCheckResult.result_hash,
      and recomputed MachineCheckResult.result_hash all match, and artifact entry
      hashes.run_artifact_hash, parsed MachineCheckResult.run_artifact_hash,
      and recomputed MachineCheckResult.run_artifact_hash all match,
    - every parsed request_hash is schema-valid and extractable.

machine_check_request:
  requires prerequisite-clean machine_check_result set.
  machine_check_request set is prerequisite-clean iff:
    - machine_check_result set is prerequisite-clean,
    - for every distinct included MachineCheckResult.request_hash there is exactly one
      matching machine_check_request entry,
    - no machine_check_request entry exists outside the included MachineCheckResult.request_hash set,
    - every included MachineCheckRequest has request_hash self-consistency:
      artifact entry hashes.request_hash, parsed MachineCheckRequest.request_hash,
      and recomputed MachineCheckRequest.request_hash all match,
    - every included MachineCheckRequest.imports.manifest_hash is syntactically valid and extractable.

import_lock:
  requires prerequisite-clean machine_check_request set,
  prerequisite-clean normalized_check_result set, and prerequisite-clean challenge_manifest set.
  import_lock set is prerequisite-clean iff:
    - the import_lock expected set is derived from
      MachineCheckRequest.imports.manifest_hash,
      NormalizedCheckResult.artifact.import_lock_hash, and
      rejection-required ChallengeManifest.imports.manifest_hash,
    - every distinct expected import lock hash has exactly one import_lock entry,
    - no import_lock entry exists outside that expected hash set,
    - every included import_lock entry has manifest_hash extractable and matching file bytes.

auxiliary_result:
  high-trust import_certificate_hash expected identities require prerequisite-clean import_lock set.
  If ReleasePolicy.mode = high-trust and import_lock set is not prerequisite-clean,
  do not derive import_certificate_hash missing / forbidden / duplicate failures.
  Report the import_lock prerequisite failure first.
  axiom_policy and reproducibility identities do not require import_lock set.

ai_audit_sidecar:
  optional machine_result sidecar source checks require prerequisite-clean
  machine_check_result set and prerequisite-clean reproducibility auxiliary_result set.
  optional normalized_comparison sidecar source checks require only
  prerequisite-clean release target normalized_check_result for normalized_comparison sidecar.
  Release target normalized_check_result is prerequisite-clean for normalized_comparison sidecar iff:
    - exactly one release target normalized_check_result entry exists,
    - its normalized_result_hash is syntactically valid and extractable,
    - embedded comparison recomputation succeeds and the recomputed comparison.status is all_agree_checked.
  They do not require prerequisite-clean machine_check_result set or
  prerequisite-clean reproducibility auxiliary_result set.
```

同じ artifact kind 内の repeatable input では occurrence order、manifest / bundle entry では
manifest 内の deterministic order で最初の failure を返します。
同じ artifact kind の class 4 で extra / forbidden existing entry と
missing expected entry が同時に成立する場合は、extra / forbidden existing entry を先に返します。
複数の extra / forbidden existing entry がある場合は、上の occurrence / deterministic order に従います。
missing expected entry のように対応する existing input / manifest entry がない failure では、
同じ artifact kind に先に返すべき extra / forbidden existing entry がない場合だけ、
artifact kind ごとの expected artifact identity key を bytewise lexicographic order で並べ、
最初の key の failure を返します。
expected artifact identity key が複数 component を持つ場合は、component-wise bytewise lexicographic order で比較します。
先頭 component が同じ場合だけ次 component を比較し、全 component が同じ key は同一 expected artifact identity です。
この比較では区切り文字付き string へ連結してはいけません。
expected artifact identity key は次で固定します。

```text
release_policy: policy_hash
runner_policy: policy_hash
checker_identity_manifest: manifest_hash
import_lock: manifest_hash
request_store_manifest: manifest_hash
machine_result_store_manifest: manifest_hash
normalized_result_store_manifest: manifest_hash
challenge_output_store_manifest: manifest_hash
machine_check_request: request_hash
machine_check_result:
  run_artifact_hash reference:
    ("run_artifact_hash", run_artifact_hash)
  release target NormalizedCheckResult.results[*] selected raw result without run_artifact_hash:
    ("release_target_result", checker_profile, request_hash, result_hash, policy_hash)
normalized_check_result: normalized_result_hash
challenge_manifest: manifest_hash
challenge_replay_result:
  release bundle command generation:
    parsed ChallengeReplayResult.result_hash after Step 7 caller-hash validation
  materialized ReleaseAuditBundleManifest / audit_bundle validation:
    ReleaseAuditBundleManifest.artifacts[].hashes.result_hash
challenge_coverage_summary: summary_hash
auxiliary_result:
  missing axiom_policy:
    ("axiom_policy", ReleasePolicy.runner_policy_hash,
     ReleaseAuditBundleManifest.artifact_hash)
  missing reproducibility:
    ("reproducibility", ReleasePolicy.runner_policy_hash,
     ReleaseAuditBundleManifest.artifact_hash)
  missing import_certificate_hash:
    ("import_certificate_hash", ReleaseAuditBundleManifest.policy_hash,
     matching import_lock manifest_hash)
ai_audit_input_policy: input_policy_hash
```

`auxiliary_result` の missing-entry key は、存在しない `AuxiliaryResult.selector` や
release target baseline result entry を読まなくても release bundle inputs から導出できる
component だけで構成します。
`axiom_policy` と `reproducibility` は release / high-trust bundle 内でそれぞれ最大1件だけ required なので、
missing-entry key には selector component を含めません。
`auxiliary_result` の required slot key は `kind`、`policy_hash`、`artifact_hash` で構成し、
selector は class 4 missing / duplicate / extra key には含めません。
同じ required slot に entry が0件なら missing、2件以上なら duplicate です。
ただし同じ `auxiliary_result` kind の outside-slot entry と required slot missing が同時に成立する場合は、
同じ artifact kind の class 4 tie-break に従い、outside-slot extra を missing より先に返します。
slot cardinality と outside-slot extra の class 4 failure は、status / selector の class 5 failure より常に先です。
同じ required slot に entry がちょうど1件ある場合だけ、まずその entry の `status != passed` を
class 5 status mismatch として評価します。
`status = passed` の場合だけ、その entry の selector presence / schema / identity mismatch を
class 5 として評価します。
`reproducibility` selector の baseline_run_artifact_hash と repeated_run_artifact_hash の
bytewise distinctness はこの selector identity 評価に含めます。
`reproducibility` selector run_artifact_hash target existence はこの selector identity
評価に含めず、`machine_check_result` closed-set / class 5 rules で扱います。
missing が定義されない optional artifact kind
（`ai_audit_sidecar`、`audit_sidecar_validation_response`、`compare_validation_response`）では、
この missing-entry key rule は使いません。
command-specific field shape が上で明示されている failure はその field shape を使い、
field shape が未定義の Step 8 failure は
`field = "command"`、`actual_value = "release_bundle_generation_failed"` を使います。

release bundle command の input flag field と hash 意味は次で固定します。
この表の順序は Step 1 の複数 failure tie-break order にも使います。
`field` には CLI flag 名を入れず、この表の field 名だけを使います。
hash field は `CommandError.field` 用の command input field であり、
`ReleaseAuditBundleManifest.artifacts[].hashes` の member 名ではありません。
repeatable input の occurrence-specific failure では `<i>` を 0-based occurrence index に置き換え、
pair count mismatch では pair field を使います。
artifact field は path field の `.path` を `.artifact.<json_path>` に置き換えた field です。
`<json_path>` は、artifact validator の caller-specific virtual root prefix を取り除いた後の
artifact-local JSON path です。
root-level artifact failure では path field の `.path` を `.artifact` に置き換え、
末尾に `.$` を残してはいけません。
MVP の release bundle command / validator で置換する virtual root prefix は次で固定します。

```text
release_policy:
  release_policy / release_policy.$
    -> release_policy.artifact

ai_audit_input_policy:
  input_policy -> ai_audit_input_policy.artifact

checker_identity_manifest:
  checker_identity_manifest / checker_identity_manifest.$
    -> checker_identity_manifest[<i>].artifact

import_lock:
  imports.manifest -> import_lock[<i>].artifact
```

たとえば `release_policy.schema` は `release_policy.artifact.schema`、
`checker_identity_manifest.schema` は
`checker_identity_manifest[<i>].artifact.schema`、
`imports.manifest.imports[<j>].module` は
`import_lock[<i>].artifact.imports[<j>].module` として報告します。
ReleasePolicy validation section の file/reference prefix rule は release policy command input
単体の `CommandError.field` 用です。
release bundle command / validator が included `release_policy` artifact の schema /
domain failure を報告する場合は、この table を優先し、`release_policy.schema` ではなく
`release_policy.artifact.schema` を使います。
Step 8 の command-specific field shape で repeatable input の `<i>` を使う場合も、
`<i>` は Step 1 の pair association 後の original 0-based occurrence index です。
closed-set validation 用に exact duplicate input を deduplicate した後の set index で
renumber してはいけません。
deduplicate された logical entry に対する Step 8 failure では、その exact duplicate group の
最小 original occurrence index を `<i>` に使います。
`npa-check release validate-bundle` / `audit_bundle` oracle は release bundle command の
repeatable input occurrence を持たないため、同じ repeatable artifact field shape を
materialized `ReleaseAuditBundleManifest` validation で使う場合、`<i>` は manifest schema / order
validation 後の `ReleaseAuditBundleManifest.artifacts` deterministic `(kind, path)` order における、
同じ artifact kind だけの 0-based occurrence index です。
この rule は `challenge_replay_result[<i>]`、`auxiliary_result[<i>]`、
`checker_identity_manifest[<i>]`、`import_lock[<i>]`、`ai_sidecar[<i>]`、
`audit_sidecar_validation[<i>]`、`compare_validation_response[<i>]` の field shape に適用します。
manifest schema / order validation 前で kind-specific occurrence index を確定できない failure では、
この command-specific repeatable field shape を使わず、`manifest.artifacts[<j>]...` の
manifest schema field shape を使います。

```text
input                                pair field                    path field                              hash field                                  artifact kind                         Step 7 actual hash source
--release-policy                     release_policy                release_policy.path                     release_policy.hash                         release_policy                        parsed ReleasePolicy canonical hash
--runner-policy                      runner_policy                 runner_policy.path                      runner_policy.hash                          runner_policy                         parsed RunnerPolicy canonical hash
--challenge-runner-policy            challenge_runner_policy       challenge_runner_policy.path            challenge_runner_policy.hash                runner_policy                         parsed challenge RunnerPolicy canonical hash
--artifact-hash                      artifact_hash                 <none>                                  artifact_hash                               <none>                                parsed target NormalizedCheckResult.artifact_hash and top-level ReleaseAuditBundleManifest.artifact_hash
--target-normalized-result           target_normalized_result      target_normalized_result.path           target_normalized_result.normalized_result_hash  normalized_check_result           parsed NormalizedCheckResult.normalized_result_hash
--request-store                      request_store                 request_store.path                      request_store.manifest_hash                 request_store_manifest                request store manifest file bytes sha256
--result-store                       result_store                  result_store.path                       result_store.manifest_hash                  machine_result_store_manifest         machine result store manifest file bytes sha256
--normalized-store                   normalized_store              normalized_store.path                   normalized_store.manifest_hash              normalized_result_store_manifest      normalized result store manifest file bytes sha256
--challenge-store                    challenge_store               challenge_store.path                    challenge_store.manifest_hash               challenge_output_store_manifest       ChallengeOutputStoreManifest file bytes sha256
--challenge-replay-result            challenge_replay_result       challenge_replay_result[<i>].path       challenge_replay_result[<i>].result_hash    challenge_replay_result               parsed ChallengeReplayResult.result_hash
--coverage-summary                   coverage_summary              coverage_summary.path                   coverage_summary.summary_hash               challenge_coverage_summary            parsed ChallengeCoverageSummary.summary_hash
--auxiliary-result                   auxiliary_result              auxiliary_result[<i>].path              auxiliary_result[<i>].result_hash           auxiliary_result                      parsed AuxiliaryResult.result_hash
--checker-identity-manifest          checker_identity_manifest     checker_identity_manifest[<i>].path     checker_identity_manifest[<i>].manifest_hash  checker_identity_manifest          checker identity manifest file bytes sha256
--import-lock                        import_lock                   import_lock[<i>].path                   import_lock[<i>].manifest_hash              import_lock                           import lock manifest file bytes sha256
--ai-audit-input-policy              ai_audit_input_policy         ai_audit_input_policy.path              ai_audit_input_policy.input_policy_hash      ai_audit_input_policy                 parsed AiAuditInputPolicy canonical hash
--ai-sidecar                         ai_sidecar                    ai_sidecar[<i>].path                    ai_sidecar[<i>].file_hash                   ai_audit_sidecar                      AiAuditSidecar file bytes sha256
--audit-sidecar-validation           audit_sidecar_validation      audit_sidecar_validation[<i>].path      audit_sidecar_validation[<i>].file_hash     audit_sidecar_validation_response     AuditSidecarValidationResult file bytes sha256
--compare-validation-response        compare_validation_response   compare_validation_response[<i>].path   compare_validation_response[<i>].file_hash  compare_validation_response           CompareValidationResult file bytes sha256
```

release bundle command が `ai_audit_input_policy` artifact の schema /
domain validation failure を `CommandError` として報告する場合、8 の `AiAuditInputPolicy`
input-policy-prefixed field shape の `input_policy` prefix を
`ai_audit_input_policy.artifact` に置換します。
したがって `input_policy` は `ai_audit_input_policy.artifact`、
`input_policy.schema` は `ai_audit_input_policy.artifact.schema`、
`input_policy.included_fields[<i>]` は
`ai_audit_input_policy.artifact.included_fields[<i>]` として報告します。
この prefix 置換は `field` だけに適用し、`expected_value` / `actual_value` は
8 の `AiAuditInputPolicy` field shape の固定値をそのまま使います。
release bundle command では `CommandError.reason_code = input_schema_invalid` を使い、
`input_policy_schema_invalid` は使いません。
`npa-check release validate-bundle` / `audit_bundle` oracle が materialized
`ReleaseAuditBundleManifest` validation の一部として同じ artifact schema / domain failure を
報告する場合も、enclosing reason_code は各 command / oracle の規則に従ったまま、
同じ `ai_audit_input_policy.artifact...` field shape を使います。
release bundle command の JSON parse failure は artifact schema failure ではなく、path field
`ai_audit_input_policy.path` の `input_json_invalid` として報告します。

`--challenge-replay-result` / `--challenge-replay-result-hash` は CLI shape 上は optional repeatable pair です。
Step 8 では `challenge_coverage_summary for challenge_replay_result` が prerequisite-clean な場合だけ、
included `ChallengeCoverageSummary.entries[*].replay_result_hash` の distinct set と
ちょうど一致する `challenge_replay_result` input set を要求し、不足・重複・余剰は
release audit bundle closed-set cardinality failure とします。
`challenge_coverage_summary for challenge_replay_result` が prerequisite-clean でない場合は、
summary の prerequisite failure を先に返し、`challenge_replay_result` input set の
不足・重複・余剰 failure を合成してはいけません。
ここでの input set は、Step 1-7 を通過した explicit input を
`kind`、bundle-local path、file bytes hash、parsed `ChallengeReplayResult.result_hash` が完全一致するものだけ
deduplicate した後の set です。
同じ `--challenge-replay-result` pair を2回渡しただけの exact duplicate は closed-set duplicate failure ではありません。
異なる file bytes / path の `challenge_replay_result` artifact が同じ parsed
`ChallengeReplayResult.result_hash` に解決される場合は、deduplicate せず duplicate として扱います。
`ReleasePolicy.ai_triage.enabled = true` の場合、`--ai-audit-input-policy` と
`--ai-audit-input-policy-hash` は required です。
`ReleasePolicy.ai_triage.enabled = false` の場合、これらの flag は forbidden です。
この conditional required / forbidden は policy file を parse した後にだけ判定できるため、
CLI argument validation error ではありません。
`ReleasePolicy.ai_triage.enabled = true` で pair が両方 omit された場合は
`CommandError.reason_code = release_bundle_generation_failed`、
`field = "ai_audit_input_policy"`、
`expected_value = "required_when_ai_triage_enabled"`、
`actual_value = "missing"` とします。
`ReleasePolicy.ai_triage.enabled = false` で pair の片方または両方が存在する場合は
`CommandError.reason_code = release_bundle_generation_failed`、
`field = "ai_audit_input_policy"`、
`expected_value = "absent_when_ai_triage_disabled"`、
`actual_value = "present"` とします。
ただし pair の片側指定、path schema violation、hash format violation、file unreadable、
JSON / schema / hash mismatch がある場合は、policy-dependent gating より先に
Step 1-7 の該当 failure を返します。
optional AI sidecar を bundle に含める場合は、対応する `--ai-sidecar` /
`--ai-sidecar-hash` と `--audit-sidecar-validation` / `--audit-sidecar-validation-hash`
の pair を明示しなければなりません。
optional compare validation response を bundle に含める場合は、
`--compare-validation-response` / `--compare-validation-response-hash` の pair を明示しなければなりません。
含めた response は `compare_validation_response` rule に従い、
`CompareValidationResult.status = valid` でなければ `release_bundle_generation_failed` です。
`npa-check release validate-bundle` は bundle manifest と referenced bundle files を検査し、
成功時は `AuxiliaryResult.kind = audit_bundle`、`status = passed` を出力します。
`--out` 指定時は `AuxiliaryResult` を保存し、`--out` なしなら inline で返します。
`--json` 成功時 stdout は保存された、または inline の `AuxiliaryResult` です。
`--out` 指定時も write summary は返さず、保存した `AuxiliaryResult` 自体を stdout に返します。
`--manifest` / `--manifest-hash` / `--json` の欠落、duplicate singleton flag、
unsupported flag は CLI argument validation error であり、`CommandError` body を返しません。
`--manifest` と `--manifest-hash` は required pair なので、片方だけが指定された invocation も
missing required flag の CLI argument validation error です。
`--manifest` と `--manifest-hash` が両方存在する場合の `--manifest-hash` invalid hash format、
`--manifest` または `--out` の path schema violation は `input_reference_invalid` です。
`--manifest` path schema violation では `field = "manifest.path"`、
`expected_value = "workspace_relative_path"`、`actual_value = "invalid_path"` とします。
`--manifest-hash` invalid hash format では `field = "manifest.manifest_hash"`、
`expected_value = "sha256:<lower-hex>"`、`actual_value = "invalid_hash_format"` とします。
`--out` path schema violation では `field = "out.path"`、
`expected_value = "workspace_relative_path"`、`actual_value = "invalid_path"` とします。
bundle manifest を読めない場合は `input_file_unreadable` で、
`field = "manifest.path"`、`actual_value = "unreadable"` とします。
`--manifest-hash` と file bytes hash が一致しない場合は `input_hash_mismatch` で、
`field = "manifest.manifest_hash"`、`expected_hash` は caller supplied hash、
`actual_hash` は manifest file bytes sha256 です。
これらは top-level command input failure なので `CommandError` を返し、`AuxiliaryResult` は出力しません。
bundle manifest が readable かつ hash-verified でも、minimum audit envelope を parse できない場合は
`CommandError.reason_code = input_json_invalid` または `input_schema_invalid` です。
JSON parse failure では `input_json_invalid`、`field = "manifest.path"`、
`expected_value = "valid_json"`、`actual_value = "invalid_json"` とします。
minimum audit envelope は top-level `schema`、valid `bundle_hash`、valid `policy_hash`、
valid `artifact_hash`、および `artifacts` array の存在です。
top-level JSON value が object でない場合、
`schema` が `npa.phase8.release_audit_bundle_manifest.v1` でない場合、
`bundle_hash` / `policy_hash` / `artifact_hash` が missing / explicit null / wrong type /
invalid hash format の場合、または `artifacts` が missing / explicit null / wrong type の場合は
`CommandError` です。
minimum audit envelope member（`schema`、`bundle_hash`、`policy_hash`、`artifact_hash`、`artifacts`）
の duplicate object key は、minimum audit envelope を deterministic に読めない failure として
`CommandError.reason_code = input_schema_invalid` にします。
複数の minimum audit envelope member が重複している場合は、
`schema`、`bundle_hash`、`policy_hash`、`artifact_hash`、`artifacts` の順で
最初の member を報告します。
minimum audit envelope member 以外の top-level duplicate key、または `artifacts[]` entry 内の
duplicate key は、minimum audit envelope が valid な後の bundle schema invalid として
`audit_bundle_invalid` にします。
この場合の field shape は次で固定します。

```text
non-envelope top-level duplicate key after minimum envelope:
  error.reason_code = audit_bundle_invalid
  error.field = "manifest"
  error.expected_value = "unique_object_keys"
  error.actual_value = "duplicate_field"

artifacts[] entry duplicate key after minimum envelope:
  error.reason_code = audit_bundle_invalid
  error.field = "manifest.artifacts[<j>]"
  error.expected_value = "unique_object_keys"
  error.actual_value = "duplicate_field"
```

複数の non-envelope duplicate key がある場合は top-level object member order を先に評価し、
次に `artifacts[]` の array order、同じ entry 内では object member order で最初の duplicate を返します。
minimum audit envelope schema failure の field shape は次で固定します。

```text
top-level JSON value is not object:
  reason_code = input_schema_invalid
  field = "manifest"
  expected_value = "object"
  actual_value = wrong_type | null_not_allowed

duplicate minimum audit envelope member:
  reason_code = input_schema_invalid
  field = "manifest.<duplicated envelope member>"
  expected_value = "unique_object_keys"
  actual_value = "duplicate_field"

schema missing / null / wrong type / mismatch:
  reason_code = input_schema_invalid
  field = "manifest.schema"
  expected_value = "npa.phase8.release_audit_bundle_manifest.v1"
  actual_value = missing | null_not_allowed | wrong_type | invalid_enum

bundle_hash / policy_hash / artifact_hash missing / null / wrong type / invalid hash format:
  reason_code = input_schema_invalid
  field = "manifest.<bundle_hash|policy_hash|artifact_hash>"
  expected_value = "sha256:<lower-hex>"
  actual_value = missing | null_not_allowed | wrong_type | invalid_hash_format

artifacts missing / null / wrong type:
  reason_code = input_schema_invalid
  field = "manifest.artifacts"
  expected_value = "array"
  actual_value = missing | null_not_allowed | wrong_type
```
minimum audit envelope schema failure が複数同時に成立する場合は、
top-level JSON value is not object、duplicate minimum audit envelope member、
schema、bundle_hash、policy_hash、artifact_hash、artifacts の順で最初の failure を返します。
duplicate minimum audit envelope member 同士の優先順は上の dedicated rule に従います。
minimum audit envelope の top-level schema mismatch では `actual_value = "wrong_schema"` を使いません。
期待 schema とは異なる string は `actual_value = "invalid_enum"`、explicit null は
`actual_value = "null_not_allowed"` とします。
minimum audit envelope が valid な後の bundle schema / domain invalid は
`AuxiliaryResult.kind = audit_bundle`、`status = failed`、
`error.reason_code = audit_bundle_invalid` です。
たとえば derived `bundle_id` mismatch、artifact entry schema violation、order violation、
closed set violation、cross-artifact mismatch は `audit_bundle_invalid` です。
`ReleaseAuditBundleManifest.bundle_hash` self-hash mismatch では
`error.field = "manifest.bundle_hash"`、
`expected_hash` は `bundle_id` と `bundle_hash` を除いて再計算した
`ReleaseAuditBundleManifest` canonical hash、
`actual_hash` は parsed `ReleaseAuditBundleManifest.bundle_hash` です。
`bundle_id` mismatch では `error.field = "manifest.bundle_id"`、
`expected_value` は `release_` + parsed `bundle_hash` の `sha256:` prefix を除いた lower-hex、
`actual_value` は parsed `ReleaseAuditBundleManifest.bundle_id` です。
`bundle_hash` self-hash mismatch と `bundle_id` mismatch が同時に成立する場合は、
`bundle_hash` self-hash mismatch を先に返します。
manifest schema / order validation 後で、より具体的な artifact kind field shape が
定義されていない bundle-level `audit_bundle_invalid` では
`error.field = "manifest"`、`actual_value = "audit_bundle_invalid"` を使います。
artifact closed-set failure で具体的な entry field がない場合は
`error.field = "manifest.artifacts"`、`actual_value = "audit_bundle_invalid"` を使います。
bundle manifest が readable かつ hash-verified で、parsed `bundle_hash` を特定できた後の
bundle invalid は `AuxiliaryResult.kind = audit_bundle`、`status = failed`、
`error.reason_code = audit_bundle_invalid` を出力します。
referenced bundle file が読めない、または missing の場合は
`AuxiliaryResult.kind = audit_bundle`、`status = failed`、
`error.reason_code = audit_bundle_missing` を出力します。
この場合の `error.field` は release bundle command の input flag field table と同じ
path field shape を使い、repeatable artifact の `<i>` は materialized
`ReleaseAuditBundleManifest` の kind-specific occurrence index rule に従います。
`expected_value = "readable_file"`、
`actual_value = "missing"` または `"unreadable"` にします。
たとえば included `ai_audit_input_policy` file が存在しない場合は
`error.field = "ai_audit_input_policy.path"`、
`expected_value = "readable_file"`、`actual_value = "missing"` です。
referenced bundle artifact validation は、manifest schema / order validation、
`bundle_hash` self-hash validation、`bundle_id` validation を通過した後でだけ行います。
複数 artifact で referenced file failure が同時に成立する場合は、
materialized `ReleaseAuditBundleManifest.artifacts[]` の deterministic `(kind, path)` order で
最初の entry を選びます。
同じ entry 内では symlink escape、missing / unreadable、file bytes hash mismatch、
JSON parse failure、artifact schema / domain failure、parsed artifact hash mismatch の順で
最初の failure を返します。
`ReleaseAuditBundleManifest.artifacts[<j>].path` が schema-valid でも、
bundle root から1回解決した IO path が symlink escape などで bundle root 外を指す場合は
missing / unreadable ではなく `audit_bundle_invalid` です。
この場合は `error.field = "manifest.artifacts[<j>].path"`、
`expected_value = "bundle_local_path_inside_bundle_root"`、
`actual_value = "invalid_path"` とします。
referenced bundle file が読めた後の file bytes hash mismatch、JSON parse failure、
artifact schema / domain failure、parsed artifact hash mismatch は
`AuxiliaryResult.kind = audit_bundle`、`status = failed`、
`error.reason_code = audit_bundle_invalid` です。
この場合の `error.field` は release bundle command の input flag field table と同じ
path / hash / artifact field shape を使い、repeatable artifact の `<i>` は materialized
`ReleaseAuditBundleManifest` の kind-specific occurrence index rule に従います。
ただし referenced file bytes hash mismatch は command の filename-hash path field ではなく、
materialized manifest entry の `file_hash` field を使います。
この failure の `error.field` は `manifest.artifacts[<j>].file_hash`、
`expected_hash` は `ReleaseAuditBundleManifest.artifacts[<j>].file_hash`、
`actual_hash` は referenced file bytes sha256 です。
`<j>` は `ReleaseAuditBundleManifest.artifacts[]` の 0-based index です。
JSON parse failure では対応する path field を使い、
`expected_value = "valid_json"`、`actual_value = "invalid_json"` とします。
たとえば included `ai_audit_input_policy` file が JSON として壊れている場合は
`error.reason_code = audit_bundle_invalid`、
`error.field = "ai_audit_input_policy.path"`、
`expected_value = "valid_json"`、`actual_value = "invalid_json"` です。
included `ai_audit_input_policy` file の schema / domain failure では
`error.reason_code = audit_bundle_invalid` のまま、
`ai_audit_input_policy.artifact...` field shape を使います。
`--out` が既に存在し、file bytes が今回生成する `AuxiliaryResult` bytes と完全一致する場合は
exact-match adoption として成功扱いします。
既存 `--out` の bytes が異なる場合は `output_path_conflict` で、
`field = "out.path"`、`expected_hash` に今回生成する `AuxiliaryResult` file bytes hash、
`actual_hash` に既存 file bytes hash を入れます。
temporary write / atomic replace failure は `output_write_failure` で、
`field = "out.path"`、`actual_value = "write_failed"` とします。

`npa-check challenge materialize-requests` は `ChallengeManifest` と `RunnerPolicy` から
required / optional profile ごとの replay `MachineCheckRequest` を生成し、
`--request-dir` に request files、`--request-store-out` に request store manifest を保存します。
この command は checker を起動せず、machine result store と normalized result store を更新しません。
CLI の required input は `--manifest` / `--manifest-hash`、
`--policy` / `--policy-hash`、`--request-dir`、`--request-store-out`、および `--json` です。
missing required flag、duplicate singleton flag、unsupported flag、missing `--json` は
CLI argument validation error であり、`CommandError` body を返しません。
`--manifest` と `--manifest-hash` は required pair なので、両方欠けている場合は
missing required flag の CLI argument validation error です。
片側指定、`--manifest` path schema violation、`--manifest-hash` invalid hash format、
`--request-dir` path schema violation、または `--request-store-out` path schema violation は
`CommandError.reason_code = input_reference_invalid` です。
API `/machine/check/challenge/requests` では同じ malformed reference object / member や
output path validation failure は wrapper schema validation または workspace path validation failure なので
`ApiError` であり、`CommandError.reason_code = input_reference_invalid` は使いません。
生成する request の `request_hash` 規則は 3.3 と同じです。
生成する request の `request_id` は
`chreq:` + `ChallengeManifest.challenge_id` + `:` + `checker_profile` に固定します。
生成する request file path は `--request-dir/<checker_profile>.json` です。
`checker_profile` は RunnerPolicy profile name grammar を通過済みなので、
generated request file path の filename は slash や `..` segment を含みません。
生成予定の request file path のいずれかが `--request-store-out` と bytewise に一致する場合は
output layout conflict として `request_output_path_conflict` です。
各 request の `module` は `ChallengeManifest.module`、
`imports` は `ChallengeManifest.imports`、
`certificate.path` と `certificate.file_hash` は
`ChallengeManifest.mutated_certificate.path` / `file_hash` を使います。
`certificate.expected_certificate_hash` は 10 の manifest-based expected hash rule に従います。
materialize は mutated certificate file を読まず、raw claimed-hash extractor も再実行しません。
`ChallengeManifest.mutated_certificate.claimed_certificate_hash` が存在する場合は
`certificate.expected_certificate_hash` にその値を copy し、manifest 側で omit されている場合は
`ChallengeManifest.base_certificate.claimed_certificate_hash` を deterministic placeholder として copy します。
このため materialization `CommandError.reason_code` には mutated certificate unreadable / hash mismatch /
claimed-hash extraction failure を追加しません。
`trust_mode`、`axiom_policy`、`budget`、`policy` は `RunnerPolicy` の値から profile ごとに写します。
materialize は request 生成前に `ChallengeManifest.imports.mode` が
`RunnerPolicy.import_policy.mode` と一致することを検査します。
また `ChallengeManifest.policy_hash` が `RunnerPolicyReference.hash` と一致することを検査します。
`--request-store-out` が既に存在する場合は manifest を検証してから entry を追加し、
`request_hash` order で sort した manifest を atomic replace で書き戻します。
`--request-store-out` が存在しない場合は empty store として作成します。
既存 request store manifest の検証では、manifest schema / order / duplicate だけでなく、
各 entry の `file_hash` が参照先 file bytes の sha256 と一致すること、
各 entry の `request_hash` が parsed `MachineCheckRequest.request_hash` と一致することも検査します。
参照先 request file を読めない、JSON として壊れている、または
`MachineCheckRequest` schema として invalid な場合も manifest 検証 failure です。
request store manifest が materialization の commit point です。
実装は request files と request store manifest の temporary file を作り、
final request file path を配置してから manifest を atomic replace します。
manifest がすべての generated request path と file hash を参照して初めて materialization 成功です。
manifest commit 前に failure した場合、manifest を更新してはいけません。
temporary file は best-effort で削除します。
manifest に参照されない orphan request file は request store reader が無視します。
retry 時に final request path が既に存在し、その file bytes が今回生成する
`MachineCheckRequest` file bytes と完全一致する場合は、上書きではなく既存 file の採用として扱います。
既存 final request path の bytes が異なる場合は `request_output_path_conflict` です。
複数の generated request path で既存 final request path conflict が同時に成立する場合は、
`RunnerPolicy.required_checker_profiles` の順序、次に `RunnerPolicy.optional_checker_profiles` の順序で
最初の checker profile の request path を報告します。
既存 manifest に同じ `request_hash`、`path`、`file_hash` の entry が既にある場合は
idempotent success として扱います。
既存 manifest 内に同じ `request_hash` または同じ `path` の entry があり、
追加予定 entry の `request_hash`、`path`、`file_hash` の組と完全一致しない場合は
`request_store_entry_conflict` です。
request file の temporary write または rename 失敗は `request_output_write_failure` です。
request store manifest の temporary write、rename、atomic replace 失敗は `request_store_write_failure` です。
`--json` 成功時 stdout は `ChallengeRequestMaterializationResult` です。
exact-match adoption や idempotent retry の成功時も `status = written` を返し、
別の `adopted` status は作りません。
materialization failure では `ChallengeRequestMaterializationResult` を返してはいけません。
CLI の `--json` では exit code 1、stdout empty、stderr に `CommandError` JSON を1個だけ出します。
API では wrapper validation 通過後の domain validation error body として
同じ `CommandError` object を返します。
この error body は release audit bundle の artifact kind には含めません。
MVP の materialization `CommandError.reason_code` は次に限定します。

```text
- input_reference_invalid
- challenge_manifest_file_unreadable
- challenge_manifest_hash_mismatch
- challenge_manifest_json_invalid
- challenge_manifest_schema_invalid
- policy_reference_invalid
- policy_file_unreadable
- policy_hash_mismatch
- import_mode_mismatch
- request_store_manifest_invalid
- request_store_entry_file_unreadable
- request_store_entry_json_invalid
- request_store_entry_schema_invalid
- request_store_entry_file_hash_mismatch
- request_store_entry_request_hash_mismatch
- request_store_entry_conflict
- request_output_path_conflict
- request_output_write_failure
- request_store_write_failure
```

materialization validation / commit failure order は次で固定します。
この順序は field 説明順ではなく、同時 failure の reason_code tie-break order です。

```text
1. CLI input reference shape:
   input_reference_invalid

2. ChallengeManifest input:
   challenge_manifest_file_unreadable
   challenge_manifest_hash_mismatch
   challenge_manifest_json_invalid
   challenge_manifest_schema_invalid

3. RunnerPolicy reference:
   policy_reference_invalid
   policy_file_unreadable
   policy_hash_mismatch

4. ChallengeManifest vs RunnerPolicy:
   import_mode_mismatch
   policy_hash_mismatch for challenge_manifest.policy_hash

5. generated output path layout:
   request_output_path_conflict

6. existing request store manifest:
   request_store_manifest_invalid

7. existing request store entries:
   request_store_entry_file_unreadable
   request_store_entry_json_invalid
   request_store_entry_schema_invalid
   request_store_entry_file_hash_mismatch
   request_store_entry_request_hash_mismatch

8. generated entry vs valid existing store:
   request_store_entry_conflict

9. commit writes:
   request_output_write_failure
   request_store_write_failure
```

materialization `CommandError` の field は固定します。
`input_reference_invalid` の field shape は次で固定します。
`--manifest` だけが欠けている場合は `field = "challenge_manifest.path"`、
`--manifest-hash` だけが欠けている場合は `field = "challenge_manifest.manifest_hash"`、
どちらも `expected_value = "required"`、`actual_value = "missing"` とします。
`--manifest` path schema violation では `field = "challenge_manifest.path"`、
`expected_value = "workspace_relative_path"`、`actual_value = "invalid_path"` とします。
`--manifest-hash` invalid hash format では `field = "challenge_manifest.manifest_hash"`、
`expected_value = "sha256:<lower-hex>"`、`actual_value = "invalid_hash_format"` とします。
`--request-dir` path schema violation では `field = "request_output_dir"`、
`expected_value = "workspace_relative_path"`、`actual_value = "invalid_path"` とします。
`--request-store-out` path schema violation では `field = "request_store_output_path"`、
`expected_value = "workspace_relative_path"`、`actual_value = "invalid_path"` とします。
複数の `input_reference_invalid` が同時に成立する場合は、`--manifest`、
`--request-dir`、`--request-store-out` の順で報告します。
同じ `--manifest` pair 内では片側指定、path schema violation、hash format violation の順で
最初の failure を返します。
同じ pair で path schema violation と hash format violation が同時に成立する場合は
path schema violation を先に返します。
`challenge_manifest_file_unreadable` では `field = "challenge_manifest.path"`、
`actual_value = "unreadable"` にします。
`challenge_manifest_hash_mismatch` では `field = "challenge_manifest.manifest_hash"`、
`expected_hash` に caller 指定 hash、`actual_hash` に manifest file bytes hash を入れます。
`challenge_manifest_json_invalid` では `field = "challenge_manifest.path"`、
`actual_value = "invalid_json"` にします。
`challenge_manifest_schema_invalid` では `field` に invalid manifest field の JSON path、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_enum`、
`invalid_hash_format`、`invalid_name_format`、`invalid_path`、`null_not_allowed`、`order_violation`、
`duplicate_field` のいずれかを入れます。
`policy_reference_invalid` では challenge 系 command 共通の policy reference field shape に従います。
`policy_file_unreadable` では `field = "policy.path"`、`actual_value = "unreadable"` にします。
`policy_hash_mismatch` では `field = "policy.hash"`、
`expected_hash` に caller 指定 hash、`actual_hash` に読み込んだ `RunnerPolicy` の canonical hash を入れます。
`ChallengeManifest.policy_hash` が `RunnerPolicyReference.hash` と一致しない場合は
同じ `policy_hash_mismatch` を使い、`field = "challenge_manifest.policy_hash"`、
`expected_hash` に `RunnerPolicyReference.hash`、
`actual_hash` に `ChallengeManifest.policy_hash` を入れます。
`import_mode_mismatch` では `field = "challenge_manifest.imports.mode"`、
`expected_value` に `RunnerPolicy.import_policy.mode`、
`actual_value` に `ChallengeManifest.imports.mode` を入れます。
`request_output_path_conflict` では、`--request-store-out` が generated request path と一致する
output layout conflict の場合、`field = "request_store_output_path"`、
`expected_value = "distinct_request_and_store_paths"`、`actual_value = "overlap"` とします。
同じ `request_output_path_conflict` 内で output layout conflict と既存 final request path conflict が
同時に成立する場合は、output layout conflict を先に報告します。
既存 final request path の bytes が今回生成する request bytes と異なる場合は、
`field` に衝突した generated request path、
`expected_hash` に今回生成する request file hash、`actual_hash` に既存 file bytes hash を入れます。
`request_store_manifest_invalid` では、既存 request store manifest file を読めない場合
`field = "request_store_output_path"`、`actual_value = "unreadable"` にします。
JSON として壊れている場合は同じ field で `actual_value = "invalid_json"` にします。
schema / order / duplicate 違反では `field` に invalid request store manifest field の JSON path、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_hash_format`、
`invalid_path`、`null_not_allowed`、`order_violation`、`duplicate_request_hash`、`duplicate_path`、
`duplicate_field` のいずれかを入れます。
この field は caller-prefixed manifest path とし、manifest-local `requests[<i>].path` は
`request_store.requests[<i>].path` として報告します。
manifest schema / domain error の field は concrete index を含む caller-prefixed path に固定し、
entry file IO / JSON / artifact schema / hash validation error の field は下の dedicated reason code にある
`request_store.requests[]` wildcard path に固定します。
manifest entry `path` が workspace-relative path schema に違反する場合は
`request_store_manifest_invalid` としてここで止め、request file は読みに行きません。
`request_store_entry_file_unreadable` では `field = "request_store.requests[].path"`、
`actual_value = "unreadable"` にします。
`request_store_entry_json_invalid` では `field = "request_store.requests[].path"`、
`actual_value = "invalid_json"` にします。
`request_store_entry_schema_invalid` では `field` に invalid request field の JSON path、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_enum`、
`invalid_path`、`invalid_hash_format`、`invalid_name_format`、`null_not_allowed`、`duplicate_field` のいずれかを入れます。
この field は artifact-local JSON path に `request_store.requests[]` prefix を付けた
wildcard-prefixed artifact path とします。
たとえば request artifact local `module` は `request_store.requests[].module` として報告します。
request store entry の top-level `schema` が
`npa.phase8.machine_check_request.v1` でない場合も `request_store_entry_schema_invalid` です。
この場合は `field = "request_store.requests[].schema"`、
`expected_value = "npa.phase8.machine_check_request.v1"`、
`actual_value = "missing"`、`"null_not_allowed"`、`"wrong_type"`、
または入力 request artifact の `schema` 文字列を入れます。
この top-level schema mismatch では `actual_value = "wrong_schema"` を使いません。
`request_store_entry_file_hash_mismatch` では `field = "request_store.requests[].file_hash"`、
`expected_hash` に manifest entry の `file_hash`、
`actual_hash` に参照先 request file bytes hash を入れます。
request store entry の request self-hash は manifest entry との比較より先に再計算します。
self-hash mismatch の場合は `request_store_entry_request_hash_mismatch` を使い、
`expected_hash` に parsed request から再計算した request hash、
`actual_hash` に parsed `MachineCheckRequest.request_hash` を入れます。
self-hash が valid な場合だけ、manifest entry の `request_hash` と parsed request field を比較します。
`request_store_entry_request_hash_mismatch` では `field = "request_store.requests[].request_hash"`、
`expected_hash` に manifest entry の `request_hash`、
`actual_hash` に parsed `MachineCheckRequest.request_hash` を入れます。
既存 request store manifest 内の複数 entry で entry validation failure が同時に成立する場合は、
`requests[]` の小さい index を先に報告します。
同じ entry 内で複数 failure が成立する場合は、file unreadable、invalid JSON、schema violation、
file_hash mismatch、request self-hash mismatch、manifest-field request_hash mismatch の順で報告します。
`request_store_entry_conflict` では `field = "request_store.requests[]"`、
`expected_value` に追加予定 entry の canonical JSON string、
`actual_value` に衝突した既存 entry の canonical JSON string を入れます。
`request_output_write_failure` では `field` に request path、
`request_store_write_failure` では `field = "request_store_output_path"` とし、
どちらも `actual_value = "write_failed"` にします。
複数の generated request path で `request_output_write_failure` が同時に成立する場合は、
`RunnerPolicy.required_checker_profiles` の順序、次に `RunnerPolicy.optional_checker_profiles` の順序で
最初の checker profile の request path を報告します。
複数の失敗条件が同時にある場合は、上の materialization validation / commit failure order で最初に該当した
`reason_code` を返します。

MVP の `ChallengeRequestMaterializationResult`：

```json
{
  "schema": "npa.phase8.challenge_request_materialization_result.v1",
  "status": "written",
  "challenge_id": "pch_001",
  "manifest_hash": "sha256:...",
  "policy_hash": "sha256:...",
  "request_store": {
    "kind": "manifest",
    "path": "build/check-requests/challenge-manifest.json",
    "manifest_hash": "sha256:..."
  },
  "requests": [
    {
      "checker_profile": "reference",
      "request_hash": "sha256:...",
      "path": "build/check-requests/challenges/pch_001/reference.json",
      "file_hash": "sha256:..."
    }
  ]
}
```

`requests` は `RunnerPolicy.required_checker_profiles` の順序、次に
`RunnerPolicy.optional_checker_profiles` の順序で並べます。
top-level `manifest_hash` は input `ChallengeManifest` file bytes の sha256 です。
`request_store.manifest_hash` は materialize 後の request store manifest file bytes の sha256 です。
`policy_hash` は input `RunnerPolicyReference.hash` です。
`ChallengeRequestMaterializationResult` は transient response であり、`result_hash` を持ちません。

MVP の `CommandError`：

```json
{
  "schema": "npa.phase8.command_error.v1",
  "command": "challenge materialize-requests",
  "reason_code": "request_output_path_conflict",
  "field": "request_output_dir/reference.json",
  "expected_hash": "sha256:...",
  "actual_hash": "sha256:..."
}
```

`schema`、`command`、`reason_code` は required です。
`command` は次のいずれかに限定します。

```text
- challenge generate
- challenge materialize-requests
- challenge replay
- challenge coverage-summary
- auxiliary axiom-policy
- auxiliary reproducibility
- auxiliary import-certificate-hash
- release stage-bundle-inputs
- release bundle
- release validate-bundle
- training export
```

`normalize-results` は `CommandError` を返さず、`NormalizeErrorResult` を返します。
`compare` は `CommandError` を返さず、`CompareValidationResult` を返します。
`audit-sidecar validate` は `CommandError` を返さず、`AuditSidecarValidationResult` または
CLI/API argument validation error を返します。
`release validate-bundle` は bundle validation の意味論的失敗を
`AuxiliaryResult.kind = audit_bundle` として返します。
この段落の `--manifest` / `--manifest-hash` は `release validate-bundle` の top-level
bundle manifest pair だけを指します。
challenge command の manifest / store reference pair は各 command 節の
`input_reference_invalid` rule に従います。
`release validate-bundle` の `--manifest` / `--manifest-hash` required pair の欠落または片側指定は CLI argument validation error であり、
`CommandError` body を返しません。
両方が指定された後の manifest reference schema violation、manifest file unreadable、
manifest file hash mismatch、manifest JSON invalid、minimum audit envelope schema invalid、
および output write failure は command pipeline 自体が成立しないため `CommandError` です。
minimum audit envelope が valid で、検証対象 bundle_hash / policy_hash / artifact_hash を特定できた後の
referenced bundle file missing / unreadable は `AuxiliaryResult.status = failed`、
`error.reason_code = audit_bundle_missing` です。
referenced bundle file missing / unreadable、readable file の hash / JSON / schema / domain /
parsed hash failure の `field` は、release validate-bundle 節の
kind-specific path / file_hash / artifact field shape に従います。
bundle_hash self-hash mismatch、bundle_id mismatch、symlink escape、または
field shape 未定義の bundle-level / artifact closed-set `audit_bundle_invalid` も
release validate-bundle 節の fixed fallback field shape に従います。
`expected_hash`、`actual_hash`、`expected_value`、`actual_value`、`diagnostics` は
該当する場合だけ入れます。
`diagnostics` は optional array of string で、3.3 の fixed diagnostics token rule に従います。
`CommandError` の分類や retry / pass 判定に使ってはいけません。
`CommandError` は transient diagnostic であり、`result_hash` を持ちません。
`challenge generate`、`challenge materialize-requests`、`challenge replay` 以外の
deterministic pipeline command の `CommandError.reason_code` は次に限定します。

```text
- policy_reference_invalid
- policy_file_unreadable
- policy_hash_mismatch
- input_reference_invalid
- input_file_unreadable
- input_hash_mismatch
- input_json_invalid
- input_schema_invalid
- input_store_manifest_invalid
- input_store_entry_invalid
- output_path_conflict
- output_write_failure
- output_store_write_failure
- release_bundle_generation_failed
- coverage_summary_generation_failed
- training_export_generation_failed
```

write failure や output path conflict は oracle の `failed` / `inconclusive` に変換せず、
`CommandError` として返します。
`input_*` reason では `field` に該当 input reference field path を入れます。
hash mismatch では `expected_hash` に caller 指定 hash、`actual_hash` に読み込んだ file bytes hash
または parsed artifact hash を入れます。
schema invalid では `expected_value` に schema requirement 名、`actual_value` に
`missing`、`wrong_type`、`unknown_field`、`invalid_enum`、`invalid_hash_format`、
`invalid_name_format`、`invalid_path`、`null_not_allowed`、`order_violation`、
`duplicate_field`、`duplicate_entry`、`duplicate_path`、`non_empty`、
`forbidden_field`、`failure_key_mismatch` のいずれかを入れます。
command-specific fixed field shape が具体的な `actual_value` を明示する場合は、
その値をこの generic list より優先します。
たとえば `count:<n>`、`missing:<kind>`、`plan.phase:<store|final>`、
または実際の invalid schema string は command-specific fixed field shape として許可します。
`*_generation_failed` の default field shape は `field = "command"`、`actual_value` に
`coverage_summary_generation_failed`、`release_bundle_generation_failed`、または
`training_export_generation_failed` を入れ、詳細は fixed diagnostics token または artifact 外ログに分離します。
command-specific fixed field shape が `field`、`expected_value`、または `actual_value` を明示する場合は、
その shape を default より優先します。
CLI の `--json` では exit code 1、stdout empty、stderr に `CommandError` JSON を1個だけ出します。
`npa-check challenge replay` は aggregate command であり、required / optional profile の
事前に materialize され request store に保存された challenge replay request と
`MachineCheckResult` を policy order で集め、
`ChallengeReplayResult` を出力します。
aggregate replay command は request store、machine result store、normalized result store を
生成・更新してはいけません。
`--out <path>` がある場合は `ChallengeReplayResult` を指定 path に保存します。
`--replay-store-out <path>` がある場合だけ challenge replay store manifest を更新します。
`--replay-store-out` を使う場合は `--out` も required です。
`--out` なしの `--json` 成功時 stdout は inline の `ChallengeReplayResult` です。
`--out` ありの `--json` 成功時 stdout も、保存した `ChallengeReplayResult` 本体です。
`--replay-store-out` を使う場合、challenge replay store manifest が commit point です。
指定された replay store manifest path の file が既に存在する場合は、既存 manifest を検証してから
replay entry を追加し、`result_hash` order で sort した manifest を atomic replace で書き戻します。
指定された replay store manifest path の file が存在しない場合は、empty replay store manifest から開始し、
新しい manifest file を作成します。
実装は replay result と replay store manifest を temporary file として作り、
final replay result path を配置してから manifest を atomic replace します。
manifest が final replay result path と file hash を参照して初めて store 更新成功です。
store commit 前に failure した場合、manifest を更新してはいけません。
retry 時に final replay result path が既に存在し、その file bytes が今回書く
`ChallengeReplayResult` file bytes と完全一致する場合は、上書きではなく既存 file の採用として扱います。
既存 final replay result path の bytes が異なる場合は `replay_output_path_conflict` です。
既存 replay store manifest に追加予定 entry と完全一致する entry が既にある場合は
idempotent success として扱います。
ここでの完全一致は `challenge_id`、`manifest_hash`、`result_hash`、`artifact_hash`、`path`、
`file_hash` がすべて一致することです。
既存 replay store manifest 内に同じ `result_hash`、同じ `path`、または同じ
`(challenge_id, manifest_hash)` の entry があり、追加予定 entry と完全一致しない場合は
`replay_store_entry_conflict` です。
`ChallengeManifest` と `RunnerPolicy` から replay `MachineCheckRequest` を再構成するのは、
request store 内の既存 request を検証するためだけです。
aggregate replay は `ChallengeManifest.policy_hash` が `RunnerPolicyReference.hash` と
一致することを検査してから request を再構成します。
`--normalized-store` は challenge result 用に事前生成された `NormalizedCheckResult` を
解決するための read-only input です。
`--coverage-required` は nightly / release coverage 用 replay を選ぶ明示フラグです。
`--coverage-required` がある場合、`--normalized-store` は required であり、
対応する `NormalizedCheckResult` が一意に解決できなければ pipeline failure です。
`--coverage-required` がない informational replay では `--normalized-store` を omit でき、
omit した場合は `ChallengeReplayResult.normalized_result_hash` と `comparison_status` も omit します。
challenge replay 用 normalized result store は、aggregate replay 前に challenge checker results を
`npa-check normalize-results` で正規化し、その出力を manifest に登録して作ります。
read-only store input を受け取る CLI flag は、path と expected manifest hash を必ず組にします。
read-only `ChallengeManifest` input も同じ扱いで、`--manifest` には `--manifest-hash` が required です。
CLI は `--manifest-hash` を challenge manifest file bytes の sha256 と照合してから parse します。
`--request-store` には `--request-store-hash`、
`--result-store` には `--result-store-hash`、
`--normalized-store` には `--normalized-store-hash` が required です。
ただし `--coverage-required` がない informational replay で `--normalized-store` 自体を omit する場合、
`--normalized-store-hash` も omit します。
CLI は expected manifest hash を manifest file bytes の sha256 と照合してから store を使います。
`npa-check challenge replay --request ...` のような単一 request 形式は MVP では定義しません。
`--profile reference build/certs/Std/Nat.npcert` のような短縮形を将来追加する場合も、
CLI は内部で policy から request を生成し、その request と request_hash を保存してから checker を起動します。
短縮形が checker binary path、import policy、axiom policy、budget を直接上書きできてはいけません。

---

# 18. API shape

CLI と同じ意味を持つ machine API を用意する場合、endpoint は次に限定します。

```text
POST /machine/check/certificate
POST /machine/check/normalize
POST /machine/check/compare
POST /machine/check/challenge
POST /machine/check/challenge/requests
POST /machine/check/challenge/replay
POST /machine/check/audit-sidecar/validate
```

API の禁止事項：

```text
- /ai/check
- /ai/verify
- /ai/accept
- /machine/check/from_source
- /machine/check/from_tactic
```

MVP の machine API は CLI と同じ local workspace file-backed API です。
request body 内の workspace-relative path は API process が持つ server workspace で解決し、
remote caller の filesystem path として解釈してはいけません。
`/machine/check/challenge` と `/machine/check/challenge/requests` の書き込み副作用、
atomic replace、commit point、exact-match adoption、failure 時の `CommandError` は
同名 CLI command と同じ規則に従います。
wrapper object schema validation と workspace path validation は分けて行います。
wrapper object schema validation では、required / unknown field、JSON type、
enum、hash format、null 禁止だけを検査します。
API wrapper の endpoint reference / output path field は、この段階では JSON string であることだけを検査します。
workspace path validation はその後に行い、endpoint wrapper の reference / output path field が
non-empty であり、`/` だけを
separator として使い、absolute path / drive prefix / empty segment / `.` / `..`
segment / control character を持たず、server workspace 外へ解決されないことを検査します。
対象は次に限定します。

```text
- RunnerPolicyReference.path
- store reference path / manifest_hash pair の path
  (`request_store.path`、`result_store.path`、`normalized_store.path`)
- ChallengeManifest reference path
- challenge materialize request_output_dir / request_store_output_path
- audit-sidecar validation sidecar.path
- audit-sidecar validation input_policy.path
```

inline artifact として渡される `MachineCheckRequest`、`ChallengeGenerationRequest`、
`MachineCheckResult`、`NormalizedCheckResult` 内部の path field は API wrapper path validation の対象外です。
それらは完全 artifact object の schema / self-hash / domain validation として検査し、
失敗時は endpoint 固有の `MachineCheckRequestErrorResult`、`NormalizeErrorResult`、
`CompareValidationResult`、または `CommandError` で返します。
`/machine/check/normalize` の `artifact_selector` は endpoint-specific domain object です。
top-level `artifact_selector` field 名の duplicate は wrapper duplicate key として
`ApiError.reason_code = api_request_schema_invalid` ですが、存在する `artifact_selector` value の
wrong type、explicit null、member の missing / wrong type / unknown / duplicate、
`module` の name grammar violation、`request_hash` の hash format violation は
wrapper validation ではなく `NormalizeErrorResult.error.reason_code = selector_schema_invalid` です。
wrapper object schema validation では duplicate key 検出を mode-dependent field exclusion より先に行います。
`schema_only` のような mode discriminator field が duplicate、missing、wrong type、
または invalid enum の場合、mode を推測せず `api_request_schema_invalid` を返します。
この場合は forbidden reference の payload や path を検査しません。
mode-dependent に forbidden になる reference field は、wrapper schema validation で
mode と field presence だけを検出した時点で workspace path validation の対象から外し、
nested path / hash / kind を検査してはいけません。
この規則は workspace path validation の対象除外だけを定めるもので、
endpoint 固有 domain validation の報告順を上書きしません。
たとえば `/machine/check/audit-sidecar/validate` の `schema_only = true` で
`result_store`、`normalized_store`、または `input_policy` が存在する場合は、
それらの内部 path が不正でも `ApiError` にはしません。
ただし実際に `AuditSidecarValidationResult.error.reason_code = validation_reference_schema_invalid`
を返すかどうかは audit-sidecar validation order の step 2-4 に従います。
API では audit-sidecar validation の active かつ mode-forbidden ではない `sidecar.path` が
workspace path validation に失敗した場合、`AuditSidecarValidationResult` ではなく常に
`ApiError.reason_code = api_path_outside_workspace` を返します。
`result_store.path`、`normalized_store.path`、`input_policy.path` についても、active reference object が
duplicate-free で対象 path member がちょうど1つだけ JSON string として存在する場合に限り、
workspace path validation failure を `ApiError.reason_code = api_path_outside_workspace` として返します。
active reference object 内に duplicate key がある場合は、audit-sidecar validation order step 5 の
`validation_reference_schema_invalid` を返します。
そのため API body のこれらの path については
`sidecar.path` と duplicate-free active validation reference path の workspace path validation failure では
`validation_reference_schema_invalid` / `actual_value = invalid_path` を返しません。
endpoint wrapper の reference / output path field が workspace path validation に失敗した場合は、理由を細分化せず
`api_path_outside_workspace` にします。
wrapper validation を通った後に policy file、manifest file、store entry file、
input artifact file を読めない場合、または output artifact / manifest を書けない場合は、
`ApiError` ではなく endpoint 固有の domain error を返します。

API の wrapper object schema violation、HTTP method mismatch、unknown endpoint、
HTTP request body JSON parse failure、および workspace path validation failure は
endpoint 固有 artifact ではなく `ApiError` を返します。
`api_json_invalid`、`api_request_schema_invalid`、`api_path_outside_workspace` は HTTP `400 Bad Request`、
`api_endpoint_not_found` は HTTP `404 Not Found`、
`api_method_not_allowed` は HTTP `405 Method Not Allowed` に固定します。
dispatch validation order は endpoint path、method の順に固定します。
endpoint path が未定義の場合は method に関係なく `api_endpoint_not_found` を返します。
endpoint path が定義済みで method が `POST` でない場合だけ `api_method_not_allowed` を返します。
`ApiError` は release audit bundle の artifact kind には含めません。

MVP の `ApiError`：

```json
{
  "schema": "npa.phase8.api_error.v1",
  "endpoint": "/machine/check/challenge",
  "reason_code": "api_path_outside_workspace",
  "field": "policy.path",
  "expected_value": "workspace_relative_path",
  "actual_value": "api_path_outside_workspace"
}
```

MVP の `ApiError.reason_code` は次に限定します。

```text
- api_json_invalid
- api_request_schema_invalid
- api_path_outside_workspace
- api_endpoint_not_found
- api_method_not_allowed
```

すべての `ApiError` で `schema`、`endpoint`、`reason_code`、`field`、
`expected_value`、`actual_value` は required です。
`ApiError` は `expected_hash`、`actual_hash`、`result_hash` を持ちません。
HTTP request body を parse できない場合でも、`endpoint` には dispatch 済み request path を入れます。
`api_json_invalid` では `field = "body"`、
`expected_value = "valid_json"`、
`actual_value = "invalid_json"` にします。
API body parser は duplicate-aware JSON event parser または duplicate-aware canonical decoder でなければなりません。
object を map に変換して duplicate key を破棄する parser、last-write-wins parser、
first-write-wins parser は禁止です。
decoder は duplicate key の JSON path と、その duplicate が endpoint wrapper / reference 側か
inline artifact 側かを判定できる情報を保持しなければなりません。
syntax として不正な JSON は `api_json_invalid` ですが、
duplicate object key は JSON parse failure ではなく schema / canonical decode failure として扱います。
endpoint wrapper object、RunnerPolicyReference、store reference、ChallengeManifest reference、
または endpoint output path wrapper に duplicate key がある場合は
`api_request_schema_invalid` を返します。
ただし `/machine/check/audit-sidecar/validate` の `result_store`、`normalized_store`、
`input_policy` は mode-dependent validation reference なので、この一般 rule の例外です。
これら active reference object 内部の duplicate key は API wrapper error ではなく、
audit-sidecar validation order step 5 の `validation_reference_schema_invalid` として返します。
inline artifact として渡される完全 `MachineCheckRequest`、`ChallengeGenerationRequest`、
`MachineCheckResult`、`NormalizedCheckResult` 内部、および
`/machine/check/normalize` の `artifact_selector` object 内部の duplicate key は
API wrapper error ではなく、各 endpoint 固有の schema validation failure として返します。
mode-dependent forbidden reference field の payload 内部に duplicate key があっても、
nested payload は検査せず、duplicate key は `api_request_schema_invalid` にしません。
forbidden reference field 名そのものが親 object で duplicate している場合だけ、
wrapper object の duplicate key として `api_request_schema_invalid` にします。
`api_request_schema_invalid` では `field` に invalid wrapper field の JSON path、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_enum`、
`invalid_hash_format`、`null_not_allowed`、`duplicate_field` のいずれかを入れます。
duplicate key の場合は `field` に duplicated field の JSON path、
`expected_value = "unique_object_keys"`、
`actual_value = "duplicate_field"` を入れます。
`api_path_outside_workspace` では `field` に path field の JSON path、
`expected_value = "workspace_relative_path"`、
`actual_value = "api_path_outside_workspace"` を入れます。
`api_endpoint_not_found` では `endpoint` に request path をそのまま入れ、
`field = "endpoint"`、`expected_value = "one_of_supported_machine_api_endpoints"`、
`actual_value = "unknown_endpoint"` にします。
`api_method_not_allowed` では `endpoint` に定義済み endpoint path を入れ、
`field = "method"`、`expected_value = "POST"`、
`actual_value` に request method token を入れます。
wrapper validation を通った後の domain validation / pipeline failure は、
各 endpoint が定義する `MachineCheckRequestErrorResult`、`NormalizeErrorResult`、
`CompareValidationResult`、`CommandError`、または `AuditSidecarValidationResult` で返します。

API の store reference object は、すべて `kind = manifest`、`path`、
`manifest_hash` を持ちます。
`manifest_hash` は referenced manifest file bytes の sha256 であり required です。
path だけの store reference、HTTP URL、database id、in-memory map は MVP では forbidden です。
`coverage_required = false` の `/machine/check/challenge/replay` で normalized result store 自体を
omit する場合だけ、normalized store reference とその `manifest_hash` を omit できます。
API の `ChallengeManifest` reference は次に固定します。

```json
{
  "kind": "file",
  "path": "build/challenges/pch_001/manifest.json",
  "manifest_hash": "sha256:..."
}
```

`manifest_hash` は challenge manifest file bytes の sha256 であり required です。
API はこの hash を照合してから `ChallengeManifest` を parse します。

`/machine/check/certificate` は `.npcert` だけを検査します。
request body は次の wrapper object です。

```json
{
  "check_request": {
    "schema": "npa.phase8.machine_check_request.v1",
    "request_hash": "sha256:..."
  },
  "policy": {
    "kind": "file",
    "path": "ci/phase8-pr-policy.json",
    "hash": "sha256:..."
  },
  "attempt": 1
}
```

`check_request` は完全な `MachineCheckRequest` object です。
`policy` は 4.1 の `RunnerPolicyReference` です。
`attempt` は optional positive integer で、省略時は `1` です。
`attempt` は result store を更新する API field ではなく、返却する
`MachineCheckResult.attempt` に写す runner input です。
wrapper object 自体の schema violation は transport-level validation error とし、
`ApiError` を返し、`MachineCheckRequestErrorResult` body を返しません。
response body は、transport-level validation failure では `ApiError`、
request load validation failure では `MachineCheckRequestErrorResult`、
それ以外の check execution / policy failure では `MachineCheckResult` です。
request load validation を通った場合だけ `MachineCheckResult` を返し、
inline `check_request` object の schema または self hash validation に失敗した場合は
`MachineCheckRequestErrorResult` を返します。
HTTP request body 自体の JSON parse failure は `ApiError.reason_code = api_json_invalid` であり、
`MachineCheckRequestErrorResult` を返してはいけません。
`policy` wrapper field の missing / wrong type / explicit null、`policy.kind` の invalid enum、
`policy.hash` の invalid hash format、unknown field、duplicate field は
`ApiError.reason_code = api_request_schema_invalid` です。
`policy.path` の workspace path validation failure は
`ApiError.reason_code = api_path_outside_workspace` です。
`check_request` が valid で、`policy` reference の file unreadable / hash mismatch /
policy schema / domain invalid が起きた場合は、`MachineCheckResult.status = failed`、
`error.kind = policy_failure` として返します。
`/machine/check/normalize` は `MachineCheckResult` の list、
`RunnerPolicyReference`、request store reference、および artifact selector を受け取り、
`NormalizedCheckResult` または `NormalizeErrorResult` を返します。
request body は次の wrapper object です。

```json
{
  "policy": {
    "kind": "file",
    "path": "ci/phase8-pr-policy.json",
    "hash": "sha256:..."
  },
  "request_store": {
    "kind": "manifest",
    "path": "build/check-requests/manifest.json",
    "manifest_hash": "sha256:..."
  },
  "artifact_selector": {
    "module": "Std.Nat",
    "request_hash": "sha256:..."
  },
  "machine_results": [
    {
      "schema": "npa.phase8.machine_check_result.v1",
      "result_hash": "sha256:...",
      "run_artifact_hash": "sha256:..."
    }
  ]
}
```

API の `machine_results` は inline の完全な `MachineCheckResult` object だけを受け取ります。
MVP API では `machine_results[].path` や file reference は定義しません。
file から読む形式は CLI の `npa-check normalize-results` だけです。
wrapper object 自体の schema violation は transport-level validation error とし、
`ApiError` を返し、`NormalizeErrorResult` body を返しません。
API の `policy` reference object の missing / wrong type / explicit null / unknown field /
invalid kind / invalid hash format / duplicate key は wrapper schema validation failure なので
`ApiError.reason_code = api_request_schema_invalid` を返します。
`policy.path` が API workspace path validation に失敗した場合は
`ApiError.reason_code = api_path_outside_workspace` を返します。
API の `request_store` reference object も endpoint wrapper field です。
`request_store` の missing / wrong type / explicit null / unknown field / duplicate key、
`request_store.kind` の invalid enum、`request_store.manifest_hash` の missing / wrong type /
explicit null / invalid hash format は wrapper schema validation failure なので
`ApiError.reason_code = api_request_schema_invalid` を返します。
`request_store.path` の missing / wrong type / explicit null は
`api_request_schema_invalid`、workspace path validation failure は
`ApiError.reason_code = api_path_outside_workspace` です。
wrapper validation 通過後の `/machine/check/normalize` endpoint-specific validation order は
6 の normalizer validation order に従います。
つまり inline `machine_results` intrinsic validation、`artifact_selector` schema validation、
`checker_profile` uniqueness validation を通過し、wrapper validation 済みの
`request_store` reference を受け取った後に policy file を解決します。
policy validation step に到達し、policy file が読めない場合は
`NormalizeErrorResult.error.reason_code = policy_file_unreadable`、
policy file が JSON parse または `RunnerPolicy` schema / domain validation に失敗した場合は
`policy_reference_invalid`、読み込んだ policy の canonical hash が
`RunnerPolicyReference.hash` と一致しない場合は `policy_hash_mismatch` にします。
`machine_results` inline object、`artifact_selector` object / member、
request store manifest file、または request store entry の endpoint-specific validation failure も
`NormalizeErrorResult` にします。
API で `artifact_selector` が存在し、その object / member が schema validation に失敗した場合は
transport-level `ApiError` ではなく `NormalizeErrorResult.error.reason_code = selector_schema_invalid`
を返します。
`artifact_selector` は optional で、省略時は 6 の single-artifact convenience mode を使います。
API response は artifact object そのものであり、normalized result store manifest は暗黙更新しません。
file-backed pipeline で後続の challenge replay / audit に使う場合、caller は response を保存し、
保存 file bytes と parsed hash から normalized result store manifest を作らなければなりません。
`/machine/check/compare` は `NormalizedCheckResult` と `RunnerPolicyReference` を受け取り、
保存済み `comparison` を再計算して検証した結果を返します。
request body は次の wrapper object です。

```json
{
  "policy": {
    "kind": "file",
    "path": "ci/phase8-pr-policy.json",
    "hash": "sha256:..."
  },
  "normalized_result": {
    "schema": "npa.phase8.normalized_check_result.v1",
    "normalized_result_hash": "sha256:...",
    "artifact_hash": "sha256:..."
  }
}
```

API の `normalized_result` は inline の完全な `NormalizedCheckResult` object だけを受け取ります。
MVP API では `normalized_result.path` や file reference は定義しません。
file から読む形式は CLI の `npa-check compare` だけです。
wrapper object 自体の schema violation は transport-level validation error とし、
`ApiError` を返し、`CompareValidationResult` body を返しません。
再計算結果が `NormalizedCheckResult.comparison` と一致しない場合は validation failure です。

MVP の `CompareValidationResult`：

```json
{
  "schema": "npa.phase8.compare_validation_result.v1",
  "normalized_result_hash": "sha256:...",
  "policy_hash": "sha256:...",
  "status": "valid",
  "embedded_comparison_status": "all_agree_checked",
  "recomputed_comparison_status": "all_agree_checked"
}
```

`status` は `valid` または `failed` です。
`status = valid` は、再計算した comparison object の canonical serialization が
`NormalizedCheckResult.comparison` と bytewise に一致したことを意味します。
`CompareValidationResult` は transient validation response であり、保存正本 artifact ではありません。
そのため `result_hash` を持ちません。
監査ログとして保存する場合は response file bytes の `file_hash` を
audit bundle manifest の `kind = compare_validation_response` entry に記録できますが、
その hash は checker verdict identity には使いません。
`CompareValidationResult.status = valid` は integrity validation の成功だけを表し、
checker success や CI pass を意味しません。
CI pass 判定では、別途 `NormalizedCheckResult.comparison.status = all_agree_checked` を要求します。
一致しない場合は `status = failed`、`error.kind = comparison_mismatch`、
`embedded_comparison_status` と `recomputed_comparison_status` を入れます。
policy を解決できない場合は `status = failed`、`error.kind = policy_failure`、
`error.reason_code = policy_reference_invalid`、`policy_file_unreadable`、
または `policy_hash_mismatch` にします。

MVP の `CompareValidationResult.error.kind` は次に限定します。

```text
- normalized_result_file_unreadable
- normalized_result_json_invalid
- normalized_result_schema_invalid
- normalized_artifact_hash_mismatch
- comparison_mismatch
- normalized_result_hash_mismatch
- policy_failure
```

`status = failed` では `error` が required です。
`CompareValidationResult` の top-level required field は status / error kind で固定します。
すべての response で `schema` と `status` は required です。
`status = valid` では `normalized_result_hash`、`policy_hash`、
`embedded_comparison_status`、`recomputed_comparison_status` が required です。
`status = failed` では `error` が required です。
`normalized_result_file_unreadable` と `normalized_result_json_invalid` では
`normalized_result_hash`、`embedded_comparison_status`、
`recomputed_comparison_status` を omit します。
`normalized_result_schema_invalid` と `normalized_artifact_hash_mismatch` では、
入力 artifact から valid な `normalized_result_hash` を信頼できないため
top-level `normalized_result_hash` を omit します。
`normalized_result_hash_mismatch` では input の `normalized_result_hash` を
top-level `normalized_result_hash` に写します。
`comparison_mismatch` では `normalized_result_hash`、`policy_hash`、
`embedded_comparison_status`、`recomputed_comparison_status` が required です。
`policy_failure` では step 4 まで成功しているため `normalized_result_hash` と
`embedded_comparison_status` が required です。
`policy_failure` では comparison recomputation を行わないため
`recomputed_comparison_status` は omit します。
`policy_hash` は `RunnerPolicyReference.hash` が valid hash として読めた場合だけ required で、
その値を写します。
`RunnerPolicyReference.hash` 自体が missing、wrong type、explicit null、
または invalid hash format の `policy_reference_invalid` では `policy_hash` を omit します。
CLI の `npa-check compare` で `--policy` または `--policy-hash` が欠落した場合は
CLI argument error であり、`CompareValidationResult` body を返しません。
両方の flag が存在した後の malformed policy reference は
`CompareValidationResult.status = failed`、
`error.kind = policy_failure`、
`error.reason_code = policy_reference_invalid` として返します。
validation order は次で固定します。

```text
1. input が file の場合、file readable / JSON parse を検査する
2. NormalizedCheckResult schema を検査する
3. NormalizedCheckResult.artifact_hash を再計算する
4. NormalizedCheckResult.normalized_result_hash を再計算する
5. RunnerPolicy を解決し、policy hash を照合する
6. comparison object を再計算する
```

1 で file を読めない場合は `normalized_result_file_unreadable`、
JSON parse に失敗した場合は `normalized_result_json_invalid` を返します。
2 で失敗した場合は `normalized_result_schema_invalid` を返します。
3 で失敗した場合は `normalized_artifact_hash_mismatch` を返します。
4 で失敗した場合は `normalized_result_hash_mismatch` を返し、
comparison mismatch は報告しません。
5 で失敗した場合は `policy_failure` を返し、comparison recomputation は行いません。
6 で失敗した場合だけ `comparison_mismatch` を返します。
`normalized_result_file_unreadable` では `error.field = "normalized_result.path"`、
`actual_value = "unreadable"` にします。
`normalized_result_json_invalid` では `error.field = "normalized_result.path"`、
`actual_value = "invalid_json"` にします。
`normalized_result_schema_invalid` では `error.field` に invalid field の JSON path、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、
`invalid_enum`、`invalid_hash_format`、`invalid_name_format`、
`null_not_allowed`、`order_violation`、`duplicate_field`、`duplicate_entry`、
`forbidden_field`、`failure_key_mismatch` のいずれかを入れます。
`normalized_artifact_hash_mismatch` では `error.field = "artifact_hash"`、
`error.expected_hash` に `artifact` object から再計算した canonical hash、
`error.actual_hash` に input の `artifact_hash` field を入れます。
`comparison_mismatch` では `error.field = "comparison"`、
`error.expected_hash` に再計算した comparison object の canonical hash、
`error.actual_hash` に embedded comparison object の canonical hash を入れます。
この場合、`embedded_comparison_status` と `recomputed_comparison_status` も required です。
`normalized_result_hash_mismatch` では `error.field = "normalized_result_hash"`、
`error.expected_hash` に `NormalizedCheckResult` から再計算した hash、
`error.actual_hash` に input の `normalized_result_hash` field を入れます。
`policy_failure` では `error.reason_code` を required にし、
`policy_reference_invalid`、`policy_file_unreadable`、`policy_hash_mismatch` のいずれかにします。
`policy_reference_invalid` では、reference object 自体が missing / wrong type / explicit null の場合
`error.field = "policy"`、`expected_value = "RunnerPolicyReference"`、
`actual_value` に `missing`、`wrong_type`、または `null_not_allowed` を入れます。
reference object が存在し、その member が不正な場合は
`error.field` に invalid member の JSON path を入れます。
既知 member では `policy.kind`、`policy.path`、`policy.hash` のいずれか、
unknown field では `policy.<unknown_field_name>` です。
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_enum`、
`invalid_path`、`invalid_hash_format`、`null_not_allowed`、`order_violation`、
`duplicate_field` のいずれかを入れます。
API の `/machine/check/compare` では wrapper schema validation が先に走るため、
policy reference の shape / hash format failure は `ApiError.reason_code = api_request_schema_invalid`
として返し、`CompareValidationResult` body を返しません。
`policy.path` が API workspace path validation に失敗した場合も
`ApiError.reason_code = api_path_outside_workspace` として返し、
`CompareValidationResult` body を返しません。
API wrapper validation 通過後に policy file が読めるが JSON parse または `RunnerPolicy`
schema / domain validation に失敗した場合は、`ApiError` ではなく
`CompareValidationResult.status = failed`、`error.kind = policy_failure`、
`error.reason_code = policy_reference_invalid` として返します。
policy file の JSON parse failure では `error.field = "policy.path"`、
`actual_value = "invalid_json"` を入れます。
`RunnerPolicy` schema / domain validation failure では `error.field` に
`policy.<RunnerPolicy JSON path>` を入れます。
root-level failure では `error.field = "policy"` とし、top-level `schema` failure では
`error.field = "policy.schema"` とします。
`expected_value` / `actual_value` は 4.1 の
RunnerPolicy schema / domain validation field shape に従います。
`policy_file_unreadable` では `error.field = "policy.path"`、
`actual_value = "unreadable"` にします。
`policy_hash_mismatch` では `error.field = "policy.hash"`、
`error.expected_hash` に caller が指定した `RunnerPolicyReference.hash`、
`error.actual_hash` に読み込んだ `RunnerPolicy` の canonical hash を入れます。
`/machine/check/challenge` は `ChallengeGenerationRequest` と `RunnerPolicyReference` を受け取り、
`ChallengeManifest`、変異後 certificate、challenge output store manifest を書き込み、
`ChallengeGenerationResult` を返します。
request body は次の wrapper object です。

```json
{
  "generation_request": {
    "schema": "npa.phase8.challenge_generation_request.v1",
    "request_hash": "sha256:..."
  },
  "policy": {
    "kind": "file",
    "path": "ci/phase8-nightly-policy.json",
    "hash": "sha256:..."
  }
}
```

`generation_request` は完全な `ChallengeGenerationRequest` object です。
API は `generation_request.policy_hash`、`policy.hash`、読み込んだ `RunnerPolicy` の
canonical hash がすべて一致することを検査します。
API は `generation_request.base_certificate.path` を読み、
file bytes hash と decoded claimed certificate hash が
`generation_request.base_certificate` 内の期待値と一致することも検査します。
CLI が生成した request と同じ field であっても、API は request body の hash 値を
信頼済み入力として扱ってはいけません。
wrapper object 自体の schema violation または API path validation failure は `ApiError` にします。
ここでいう API path validation failure は wrapper field である `policy.path` に限ります。
`generation_request.base_certificate.path`、`generation_request.imports.manifest`、
`generation_request.output.store_manifest_path`、`generation_request.output.manifest_path`、
`generation_request.output.mutated_certificate_path` の path schema violation は
inline `ChallengeGenerationRequest` の validation failure なので `CommandError` です。
wrapper validation を通った後の policy reference validation failure、
generation request validation failure、または generation pipeline failure は
`CommandError` にします。
`/machine/check/challenge/requests` は `ChallengeManifest` reference、`RunnerPolicyReference`、
request output directory、request store output path を受け取り、policy order の replay
`MachineCheckRequest` files と request store manifest を生成します。
この endpoint は checker を起動せず、`MachineCheckResult` と `NormalizedCheckResult` を生成しません。
response は `ChallengeRequestMaterializationResult` です。
wrapper object 自体の schema violation または API path validation failure は `ApiError` にします。
wrapper validation を通った後の policy reference validation failure、
challenge manifest validation failure、または既存 request / manifest conflict は
`CommandError` にします。
request body は次の wrapper object です。

```json
{
  "challenge_manifest": {
    "kind": "file",
    "path": "build/challenges/pch_001/manifest.json",
    "manifest_hash": "sha256:..."
  },
  "policy": {
    "kind": "file",
    "path": "ci/phase8-nightly-policy.json",
    "hash": "sha256:..."
  },
  "request_output_dir": "build/check-requests/challenges/pch_001",
  "request_store_output_path": "build/check-requests/challenge-manifest.json"
}
```

`request_output_dir` と `request_store_output_path` は workspace-relative path です。
`/machine/check/challenge/replay` は `ChallengeManifest` reference、`RunnerPolicyReference`、
challenge request store、machine result store、optional normalized result store を受け取り、
`ChallengeReplayResult` を返します。
request body は required boolean の `coverage_required` を持ちます。
request body は次の wrapper object です。

```json
{
  "coverage_required": true,
  "challenge_manifest": {
    "kind": "file",
    "path": "build/challenges/pch_001/manifest.json",
    "manifest_hash": "sha256:..."
  },
  "policy": {
    "kind": "file",
    "path": "ci/phase8-nightly-policy.json",
    "hash": "sha256:..."
  },
  "request_store": {
    "kind": "manifest",
    "path": "build/check-requests/challenge-manifest.json",
    "manifest_hash": "sha256:..."
  },
  "result_store": {
    "kind": "manifest",
    "path": "build/check-results/manifest.json",
    "manifest_hash": "sha256:..."
  },
  "normalized_store": {
    "kind": "manifest",
    "path": "build/normalized/challenge-manifest.json",
    "manifest_hash": "sha256:..."
  }
}
```

`coverage_required = true` の replay request では normalized result store が required です。
`coverage_required = false` で normalized result store を omit した場合、
response の `normalized_result_hash` と `comparison_status` は omit します。
wrapper object 自体の schema violation、required reference object の missing、
provided / active reference object の wrong type / explicit null / unknown field / duplicate key /
invalid kind / invalid hash format、または API path validation failure は
`ApiError` にします。
`coverage_required = false` で `normalized_store` object を omit することは ApiError ではありません。
API では malformed manifest / store reference を `CommandError.reason_code = input_reference_invalid`
に変換しません。
wrapper validation を通った後の manifest / store file unreadable、manifest hash mismatch、
manifest JSON / schema / domain failure、policy reference validation failure、
または replay pipeline failure は `CommandError` にします。
単一 checker profile の challenge execution は `/machine/check/certificate` に
challenge 用 `MachineCheckRequest` を渡して行います。
`/machine/check/audit-sidecar/validate` は `AiAuditSidecar` と validation references を受け取り、
schema-only または cross-artifact validation result を返します。
request body は次の wrapper object です。

```json
{
  "schema_only": false,
  "sidecar": {
    "path": "build/audit/Std.Nat.ai.json"
  },
  "result_store": {
    "kind": "manifest",
    "path": "build/check-results/manifest.json",
    "manifest_hash": "sha256:..."
  },
  "normalized_store": {
    "kind": "manifest",
    "path": "build/normalized/manifest.json",
    "manifest_hash": "sha256:..."
  },
  "input_policy": {
    "path": "ci/phase8-ai-triage-default.json",
    "hash": "sha256:..."
  }
}
```

`schema_only` は required boolean で、省略時 default はありません。
`schema_only = true` の場合だけ schema-only validation を行い、
`schema_only = false` の場合は cross-artifact validation を行います。
`sidecar` と `sidecar.path` はどちらの mode でも required です。
wrapper object 自体の schema violation または API path validation failure は `ApiError` にし、
`AuditSidecarValidationResult` body を返しません。
API wrapper schema validation で required とする audit-sidecar field は
`schema_only`、`sidecar`、`sidecar.path` だけです。
`sidecar` または `sidecar.path` の missing、wrong type、explicit null は
step 1 の fixed wrapper schema failure であり、`ApiError.reason_code = api_request_schema_invalid`
を返します。
`sidecar.path` が JSON string だが workspace path validation に失敗した場合は
`ApiError.reason_code = api_path_outside_workspace` を返します。
`result_store`、`normalized_store`、`input_policy` は mode-dependent validation reference であり、
欠落、object type、required member、unknown field、duplicate key、hash format、explicit null は
wrapper schema violation ではなく step 4 または step 5 の
`AuditSidecarValidationResult` として返します。
`kind` enum を持つのは `result_store` と `normalized_store` だけで、
`input_policy` は `kind` field を持ちません。
wrapper validation を通った後の sidecar file unreadable / JSON parse failure、
sidecar schema failure、validation reference failure、または cross-artifact validation failure は
`AuditSidecarValidationResult` にします。

MVP の `AuditSidecarValidationResult`：

```json
{
  "schema": "npa.phase8.audit_sidecar_validation_result.v1",
  "mode": "cross_artifact",
  "sidecar_file_hash": "sha256:...",
  "input_policy_hash": "sha256:...",
  "status": "valid",
  "source_kind": "machine_result",
  "source_result_hash": "sha256:...",
  "source_normalized_result_hash": "sha256:..."
}
```

`mode` は `schema_only` または `cross_artifact` です。
`status` は `valid` または `failed` です。
`AuditSidecarValidationResult` は transient validation response であり、保存正本 artifact ではありません。
そのため `result_hash` を持ちません。
監査ログとして保存する場合は response file bytes の `file_hash` を
audit bundle manifest の `kind = audit_sidecar_validation_response` entry に記録できますが、
その hash は checker verdict identity には使いません。
`status = valid` は sidecar validation の成功だけを表し、checker success や CI pass を意味しません。
`status = failed` では `error.kind = audit_sidecar_validation_failure` と
`error.reason_code` を required にします。

`AuditSidecarValidationResult` の required field は mode、status、sidecar parse state、
および source kind の解決可否で固定します。
どちらの mode でも `schema`、`mode`、`status` は required です。
`sidecar_file_hash` は sidecar file bytes を読めた場合だけ required です。
sidecar file を読めない場合は `sidecar_file_hash` を omit します。
`mode = schema_only` では `source_kind`、`source_result_hash`、
`source_normalized_result_hash`、`input_policy_hash` をすべて omit し、
source artifact、policy、store manifest は検証しません。
`mode = cross_artifact` では、sidecar JSON を parse でき、`source.kind` が valid enum として
読めた場合だけ `source_kind` が required です。
`input_policy_hash` は validation order の step 5 以降に到達し、
validation reference の `input_policy.hash` が valid hash として読めた場合だけ
required で、その値を写します。
step 2-4 の失敗では validation references を読まず、`input_policy_hash` を omit します。
`validation_reference_missing`、`validation_reference_schema_invalid`、または
`input_policy.hash` 自体の invalid hash でも omit します。
`sidecar_file_unreadable`、`sidecar_json_invalid`、または `source.kind` 自体の
`sidecar_schema_invalid` では `source_kind` を omit します。
`source.kind` 以外の schema violation や cross-artifact mismatch では、
読めた `source.kind` を `source_kind` に写します。
`source_kind = machine_result` では、`source.result_hash` が valid hash として読めた場合だけ
`source_result_hash` が required です。
`source.result_hash` 自体の `sidecar_schema_invalid` では `source_result_hash` を omit します。
sidecar source に `normalized_result_hash` がある場合だけ
`source_normalized_result_hash` も required にし、ない場合は omit します。
ただし `source.normalized_result_hash` 自体が missing / invalid hash の場合は
`source_normalized_result_hash` を omit します。
`source_kind = normalized_comparison` では、`source.normalized_result_hash` が valid hash として
読めた場合だけ `source_normalized_result_hash` が required で、`source_result_hash` は omit します。

MVP の `AuditSidecarValidationResult.error.reason_code`：

```text
- sidecar_file_unreadable
- sidecar_json_invalid
- sidecar_schema_invalid
- forbidden_sidecar_field
- validation_reference_missing
- validation_reference_schema_invalid
- input_policy_file_unreadable
- input_policy_json_invalid
- input_policy_schema_invalid
- input_policy_hash_mismatch
- input_policy_field_mismatch
- result_store_manifest_hash_mismatch
- result_store_manifest_invalid
- normalized_store_manifest_hash_mismatch
- normalized_store_manifest_invalid
- referenced_file_hash_mismatch
- referenced_artifact_hash_mismatch
- referenced_artifact_value_mismatch
- source_result_not_found
- source_normalized_result_not_found
- source_hash_mismatch
- source_id_mismatch
- normalized_result_missing_source
```

audit-sidecar validation order は次で固定します。
先の step で失敗した場合、後続 step の error は報告しません。

```text
1. validation request body の schema と mode を検査する
2. sidecar reference path schema、sidecar file readable / JSON parse を検査する
3. sidecar の closed-world schema、source shape、常時 forbidden field を検査する
4. schema_only = true の場合は cross-artifact validation references が存在しないことを検査し、
   なければここで終了する
5. cross-artifact validation references の required field と reference path schema を検査する
6. input_policy file readable / JSON / schema、reference / sidecar / file hash の3者一致、
   copied metadata を検査する
7. input_policy 依存の forbidden field を検査する
8. result_store manifest hash / schema / entry hashes を検査する
9. normalized_store が required または provided の場合、manifest hash / schema / entry hashes を検査する
10. sidecar source を store から解決し、source hash / id / normalized membership を検査する
```

API では validation request body 自体の JSON parse failure は
`ApiError.reason_code = api_json_invalid` とし、`AuditSidecarValidationResult` body を返しません。
API では 1 の validation request wrapper schema / mode failure は
`ApiError.reason_code = api_request_schema_invalid` とし、`AuditSidecarValidationResult` body を返しません。
API でも cross-artifact validation reference の欠落や partial reference は
`ApiError` ではなく step 5 の validation failure として返します。
API の `schema_only = false` で `result_store` または `input_policy` object が欠けている場合は
`validation_reference_missing` です。
API の active reference object が存在する場合、その required member の欠落、
wrong type、explicit null、invalid enum、invalid hash format は
`validation_reference_schema_invalid` です。
store reference の required member は `kind`、`path`、`manifest_hash` です。
`input_policy` reference の required member は `path`、`hash` で、`kind` は forbidden unknown field です。
CLI では `--sidecar` 欠落、`--schema-only` の重複、値付き形式
`--schema-only=<value>`、`--no-schema-only`、`--cross-artifact` など
MVP で定義しない mode flag だけを CLI argument error とし、
`AuditSidecarValidationResult` body を返しません。
cross-artifact validation reference の欠落や partial reference は、
validation request を構成したうえで step 5 の validation failure として返します。
2 以降の失敗だけ `AuditSidecarValidationResult.status = failed` として返します。
CLI または非 API caller で `sidecar.path` が workspace-relative path schema に失敗した場合は
step 2 の `validation_reference_schema_invalid` とし、file read は試みません。
CLI または非 API caller で cross-artifact validation reference の path が
workspace-relative path schema に失敗した場合は step 5 の
`validation_reference_schema_invalid` とし、manifest / policy file read は試みません。
API request body の validation reference path schema failure は、この order に入る前に
`ApiError.reason_code = api_path_outside_workspace` として返します。
ただしこれは `result_store`、`normalized_store`、`input_policy` の active reference object と
path member が JSON string として存在する場合だけです。
API の workspace path validation は duplicate-aware decode 後に行います。
active reference object 内に duplicate key がある場合は、path member の値が不正でも
workspace path validation を行わず、step 5 の `validation_reference_schema_invalid` を返します。
`api_path_outside_workspace` は、active reference object が duplicate-free で、
対象 path member がちょうど1つだけ JSON string として存在する場合にだけ返します。
この条件を満たす active reference path が workspace path validation に失敗した場合は、
同じ reference object 内の invalid hash format、invalid enum、unknown field、
または non-path required member missing よりも `api_path_outside_workspace` を優先します。
path member 自体が missing、explicit null、wrong type、または duplicate key の影響下にある場合は
workspace path validation を行わず、step 5 の validation reference failure を返します。
required reference object 自体の欠落は step 5 の
`validation_reference_missing` として返します。
存在する reference object 内の required member 欠落、wrong type、または explicit null は
step 5 の `validation_reference_schema_invalid` として返します。
`sidecar.path` は validation reference ではなく fixed wrapper field です。
API の `sidecar` / `sidecar.path` の missing、wrong type、explicit null は step 1 の `ApiError`、
JSON string として存在する `sidecar.path` の workspace path validation failure は
`api_path_outside_workspace` です。
`schema_only = true` で forbidden validation reference が存在していても、
sidecar file を読めない場合は step 2 の `sidecar_file_unreadable` を返します。
sidecar JSON parse に失敗した場合も step 2 の `sidecar_json_invalid` を返します。
sidecar schema / 常時 forbidden field に失敗した場合は step 3 の error を返します。
forbidden validation reference の `validation_reference_schema_invalid` は、
step 2 と step 3 を通過した後の step 4 でだけ返します。
3 の常時 forbidden field は reserved verdict field、certificate bytes field、secret token field です。
`allow_source_text` / `allow_tactic_trace` に依存する policy-gated field は、
field name の存在だけでは step 3 の unknown field にせず、top-level path と値の shape が valid な場合は 7 で検査します。
top-level 以外の path にある policy-gated field は step 3 の `forbidden_sidecar_field` として返します。
4 で `result_store`、`normalized_store`、`input_policy` のいずれかが存在する場合は
`error.reason_code = validation_reference_schema_invalid`、
`error.field` に存在してはいけない reference の JSON path、
`expected_value = "absent"`、`actual_value = "present"` を入れます。
forbidden reference field の値が explicit null、wrong type、または malformed object でも、
step 4 では nested schema を検査せず、field presence を優先して `actual_value = "present"` にします。
CLI では forbidden reference presence を、対応する path flag または hash flag のどちらかが
存在することとして判定します。
hash-only flag だけが存在する場合も nested hash の schema は検査せず、
`expected_value = "absent"`、`actual_value = "present"` を返します。
複数の forbidden CLI flag が同時に存在する場合は
`result_store.path`、`result_store.manifest_hash`、`normalized_store.path`、
`normalized_store.manifest_hash`、`input_policy.path`、`input_policy.hash` の順で
最初の field を `error.field` に入れます。
5 で `result_store`、`input_policy`、または required な `normalized_store` が不足する場合は
`input_policy_schema_invalid` を返さず、`error.reason_code = validation_reference_missing`、
`error.field` に不足 reference の JSON path、`actual_value = "missing"` を入れます。
CLI で required reference pair が完全に欠けている場合は不足 reference object の JSON path を入れます。
たとえば `--result-store` と `--result-store-hash` が両方ない場合は `error.field = "result_store"` です。
path flag と hash flag の片方だけが存在する場合は不足 member の JSON path を入れ、
`validation_reference_schema_invalid`、`expected_value = "required"`、
`actual_value = "missing"` を返します。
API の reference object が存在するが required member を欠く場合も同じく
`validation_reference_schema_invalid`、`expected_value = "required"`、
`actual_value = "missing"` を返します。
5 でその他の validation reference の schema が不正な場合は
`error.reason_code = validation_reference_schema_invalid`、
`error.field` に invalid reference field の JSON path を入れます。
step 5 内で複数の validation reference failure が同時に成立する場合は、次の順で最初の1件だけを返します。

```text
1. required reference object missing:
   result_store, input_policy, normalized_store
2. active reference object explicit null / wrong type:
   result_store, input_policy, normalized_store
3. partial reference missing required member:
   result_store.path, result_store.manifest_hash,
   input_policy.path, input_policy.hash,
   normalized_store.path, normalized_store.manifest_hash
4. reference member explicit null / wrong type / invalid enum / invalid hash format / duplicate key / unknown field:
   result_store.kind, result_store.path, result_store.manifest_hash,
   input_policy.path, input_policy.hash, input_policy.kind,
   normalized_store.kind, normalized_store.path, normalized_store.manifest_hash,
   その後 duplicate key / generic unknown field: result_store, input_policy, normalized_store
5. CLI / non API caller の workspace-relative path schema failure:
   result_store.path, input_policy.path, normalized_store.path
```

active reference object explicit null / wrong type では、`error.field` に reference object の JSON path、
`expected_value = "object"`、`actual_value = "null_not_allowed"` または `"wrong_type"` を入れます。
active reference object 内の generic unknown field は step 5 の
`validation_reference_schema_invalid` で返し、API wrapper schema error にはしません。
active reference object 内の duplicate key も step 5 の `validation_reference_schema_invalid` で返し、
API wrapper schema error にはしません。
duplicate key では `expected_value = "unique_object_keys"`、
`actual_value = "duplicate_field"` とします。
duplicate key が複数ある場合は `result_store`、`input_policy`、`normalized_store` の順で
reference object を選び、同じ object 内では duplicated field name の bytewise lexicographic order で
最初の field を返します。
generic unknown field が複数ある場合は `result_store`、`input_policy`、`normalized_store` の順で
reference object を選び、同じ object 内では unknown field name の bytewise lexicographic order で
最初の field を返します。
`input_policy.kind` は known forbidden unknown field として上の fixed order に含め、
generic unknown field より先に返します。
`normalized_store` は sidecar が `source.normalized_result_hash` を持つ場合だけ required reference object として扱います。
sidecar が `source.normalized_result_hash` を持たない場合に `normalized_store` が完全に omit されても
step 5 の failure ではありません。
audit-sidecar validation order step 2 で CLI / 非 API caller の `sidecar.path` が invalid path の場合も
`validation_reference_schema_invalid` を使い、`error.field = "sidecar.path"`、
`expected_value = "workspace_relative_path"`、`actual_value = "invalid_path"` にします。

`sidecar_file_unreadable` では `error.field = "sidecar.path"`、
`actual_value = "unreadable"` にし、`sidecar_file_hash` は omit します。
`sidecar_json_invalid` では `error.field = "sidecar.path"`、
`actual_value = "invalid_json"` にし、`sidecar_file_hash` は required です。
manifest hash mismatch では `error.field` に `result_store.manifest_hash` または
`normalized_store.manifest_hash` を入れ、`expected_hash` と `actual_hash` を入れます。
`sidecar_schema_invalid` では `error.field` に invalid field の JSON path を入れ、
`expected_value` に schema requirement 名、`actual_value` に `missing`、`wrong_type`、
`unknown_field`、`invalid_enum`、`invalid_hash_format`、`order_violation`、
`null_not_allowed`、`duplicate_field` のいずれかを入れます。
`forbidden_sidecar_field` では `error.field` に forbidden field の JSON path を入れ、
`actual_value = "present"` にします。
`validation_reference_missing` では `error.field` に `result_store`、`normalized_store`、
`input_policy` のいずれかを入れ、`actual_value = "missing"` にします。
`validation_reference_schema_invalid` では `error.field` に invalid validation reference field の JSON path、
`expected_value` に schema requirement 名、`actual_value` に `missing`、`wrong_type`、
`unknown_field`、`invalid_enum`、`invalid_path`、`invalid_hash_format`、
`null_not_allowed`、`duplicate_field`、`present` のいずれかを入れます。
`actual_value = invalid_path` の場合、`expected_value` は `workspace_relative_path` に固定します。
validation reference object またはその member が explicit null の場合は
`actual_value = null_not_allowed` にします。
`input_policy_file_unreadable` では `error.field = "input_policy.path"`、
`actual_value = "unreadable"` にします。
input policy file が JSON として壊れている場合は `input_policy_json_invalid` として扱い、
`error.field = "input_policy.path"`、`expected_value = "valid_json"`、
`actual_value = "invalid_json"` にします。
`input_policy_schema_invalid` では `error.field` に 8 の
`AiAuditInputPolicy` input-policy-prefixed field shape を使い、
`expected_value` / `actual_value` も 8 の `AiAuditInputPolicy` field shape が
定義する固定値をそのまま使います。
たとえば `included_fields[]` unsupported field path では
`expected_value = "allowed_input_policy_field"`、
`included_fields` order violation では
`expected_value = "field_path_bytewise_ascending"`、
duplicate では `expected_value = "unique_included_fields"` です。
これらを generic な schema requirement 名へ置き換えてはいけません。
`input_policy_hash_mismatch` では `error.field = "input_policy.hash"` を入れます。
step 6 の内部優先順位は次で固定します。
先の item で失敗した場合、後続 item の error は報告しません。

```text
1. input policy file readable
2. input policy file JSON parse
3. input policy file AiAuditInputPolicy schema / domain validation
4. validation reference / sidecar copied metadata / input policy file canonical hash mismatch
5. sidecar copied input_policy metadata mismatch
```

step 6 で複数の input policy hash mismatch が同時に成立する場合は、
validation reference と sidecar copied metadata の不一致を最優先します。
次に validation reference と input policy file の不一致を返し、
最後に sidecar copied metadata と input policy file の不一致を返します。
validation reference と sidecar copied metadata が不一致の場合、
`expected_hash` に validation reference の `input_policy.hash`、
`actual_hash` に `AiAuditSidecar.input_policy.hash` を入れます。
validation reference と input policy file が不一致の場合、
`expected_hash` に validation reference の `input_policy.hash`、
`actual_hash` に input policy file から再計算した canonical hash を入れます。
sidecar copied metadata と input policy file の不一致だけが残る場合も、
`expected_hash` に `AiAuditSidecar.input_policy.hash`、
`actual_hash` に input policy file から再計算した canonical hash を入れます。
`input_policy_field_mismatch` では `error.field` に
`input_policy.id`、`input_policy.version`、`input_policy.included_fields`、
`input_policy.redaction` のいずれかを入れ、
`expected_value` に policy file の値、`actual_value` に sidecar の値を入れます。
複数の copied metadata mismatch が同時に成立する場合は
`input_policy.id`、`input_policy.version`、`input_policy.included_fields`、
`input_policy.redaction` の順で最初の field を返します。
`result_store_manifest_invalid` と `normalized_store_manifest_invalid` では、
manifest file を読めない場合は `error.field = "result_store.path"` または
`error.field = "normalized_store.path"`、`actual_value = "unreadable"` にします。
JSON として壊れている場合は同じ field で `actual_value = "invalid_json"` にします。
schema / order / duplicate 違反では `error.field` に invalid manifest field の JSON path を入れ、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_hash_format`、
`invalid_path`、`null_not_allowed`、`order_violation`、`duplicate_field`、または manifest 種別ごとの
unique key duplicate reason を入れます。
manifest schema / domain error の field は concrete index を含む caller-prefixed path に固定します。
たとえば manifest-local `results[<i>].path` は
`result_store.results[<i>].path` または `normalized_store.results[<i>].path` として報告します。
manifest entry `path` が workspace-relative path schema に違反する場合は
対応する `*_store_manifest_invalid` としてここで止め、entry file は読みに行きません。
参照 file bytes や parsed artifact の hash validation error は下の `referenced_*` reason code にある
`result_store.results[]` / `normalized_store.results[]` wildcard path に固定します。
`result_store_manifest_invalid` で許可する unique key duplicate reason は
`duplicate_run_artifact_hash` と `duplicate_path` だけです。
`normalized_store_manifest_invalid` で許可する unique key duplicate reason は
`duplicate_normalized_result_hash` と `duplicate_path` だけです。
`duplicate_run_artifact_hash` を normalized store manifest に使ってはいけません。
`duplicate_normalized_result_hash` を result store manifest に使ってはいけません。
unique key duplicate reason の `error.field` は、重複 key の caller-prefixed manifest path に固定します。
`duplicate_run_artifact_hash` は `result_store.results[<i>].run_artifact_hash`、
`duplicate_normalized_result_hash` は `normalized_store.results[<i>].normalized_result_hash` です。
`duplicate_path` は store 種別ごとの concrete manifest path に固定し、
`result_store.results[<i>].path` または
`normalized_store.results[<i>].path` です。
`referenced_file_hash_mismatch` では `error.field` に
`result_store.results[].file_hash` または `normalized_store.results[].file_hash` を入れ、
`expected_hash` には manifest entry の file hash、
`actual_hash` には参照 file bytes から再計算した hash を入れます。
`referenced_artifact_hash_mismatch` では `error.field` に
`result_store.results[].result_hash`、`result_store.results[].request_hash`、
`result_store.results[].run_artifact_hash`、または
`normalized_store.results[].artifact_hash`、
`normalized_store.results[].normalized_result_hash` を入れます。
store entry artifact の self-hash mismatch では、`expected_hash` には parsed artifact から
再計算した hash、`actual_hash` には parsed artifact 内の self-hash field を入れます。
複数の self-hash field がある artifact の検査順は、challenge replay の
store entry validation と同じ順序にします。
同じ store manifest 内に複数の referenced file / artifact failure がある場合は、
entry array の小さい index を先に報告します。
同じ entry 内で複数 failure が成立する場合だけ、challenge replay の
store entry artifact validation order と同じ順序を使います。
self-hash が valid な artifact と manifest entry の hash field mismatch では、
`expected_hash` には manifest entry の hash、
`actual_hash` には parsed artifact field の hash を入れます。
この hash mismatch payload は、上記 `referenced_artifact_hash_mismatch` の
hash field にだけ使い、`checker_profile` や `status` には使いません。
`checker_profile` や `status` のような non-hash field mismatch では
`referenced_artifact_value_mismatch` を使い、`error.field` には
`result_store.results[].checker_profile`、`status`、または
`classification.checker_error_kind` を入れます。
store entry checker profile mismatch では `expected_value` に manifest entry の `checker_profile`、
`actual_value` に parsed `MachineCheckResult.checker.profile` を入れます。
source artifact 状態に対して sidecar status が許可されない場合は
`error.field = "status"`、`expected_value` に許可 status set 名、
`actual_value` に `AiAuditSidecar.status` を入れます。
classification checker error kind mismatch では、参照先 `MachineCheckResult.status` と
`MachineCheckResult.error.kind` に基づいて、AiAuditSidecar 節で定義した
`classification.checker_error_kind` field shape を使います。
`source_result_not_found` では `error.field = "source.run_artifact_hash"`、
`expected_hash` に sidecar が参照した `source.run_artifact_hash` を入れます。
`actual_hash` は該当 entry が存在しないことを表すため omit します。
`source_normalized_result_not_found` では `error.field = "source.normalized_result_hash"`、
`expected_hash` に sidecar が参照した `source.normalized_result_hash` を入れます。
`actual_hash` は該当 entry が存在しないことを表すため omit します。
`source_hash_mismatch` では `error.field` に `source.result_hash` または
`source.request_hash` を入れ、`expected_hash` には sidecar source の hash、
`actual_hash` には参照先 `MachineCheckResult` の同じ field の hash を入れます。
`source_id_mismatch` では `error.field` に `source.result_id` または `source.normalized_result_id` を入れ、
`expected_value` には sidecar source の id、
`actual_value` には参照先 artifact の同じ id field を入れます。
step 10 の `source.kind = machine_result` は、まず `source.run_artifact_hash` を
machine result store の unique key として lookup します。
該当 entry がなければ `source_result_not_found` を返します。
lookup 成功後は `source.result_hash`、`source.request_hash` の順に照合し、
最初の mismatch だけを `source_hash_mismatch` として返します。
`source.run_artifact_hash` は lookup key なので、lookup 成功後の `source_hash_mismatch.field` には使いません。
machine result hash が一致した後、`source.result_id` が存在する場合は
参照先 `MachineCheckResult.result_id` と照合し、mismatch なら `source_id_mismatch` を返します。
`source.kind = machine_result` で `source.normalized_result_id` が存在し、
`source.normalized_result_hash` が存在しない sidecar は step 3 の
`sidecar_schema_invalid` で拒否済みなので、step 10 には到達しません。
`source.normalized_result_hash` が存在する場合は、次に normalized store から
その `NormalizedCheckResult` を解決します。
解決できなければ `source_normalized_result_not_found` を返します。
解決できた `NormalizedCheckResult.results` に `source.result_hash` が含まれない場合は
`normalized_result_missing_source` を返します。
この check は `result_hash` による semantic membership です。
一般の audit-sidecar cross-artifact validator は `NormalizedCheckResult` から
source の exact `run_artifact_hash` を復元してはいけません。
release audit bundle validator だけが、bundle 内 artifact closed set と reproducibility selector を使って
exact selected raw result rule を追加で検査します。
`source.normalized_result_id` は normalized membership check が通った後だけ照合し、
mismatch なら `source_id_mismatch` を返します。
source id / normalized membership check が通った後、sidecar status 許可条件より先に
`classification.checker_error_kind` の cross-artifact required / mismatch / checked-result presence を検査します。
classification mismatch と sidecar status 許可違反が同時に成立する場合は、
`classification.checker_error_kind` の `referenced_artifact_value_mismatch` を返します。
classification check が通った後で sidecar status 許可条件を検査します。
許可されない場合は `referenced_artifact_value_mismatch` を返します。
`normalized_result_missing_source` では
`error.field = "normalized_result.results[].result_hash"`、
`expected_hash` に sidecar が参照した `source.result_hash` を入れます。
`actual_hash` は該当 entry が存在しないことを表すため omit します。
step 10 の `source.kind = normalized_comparison` は、まず `source.normalized_result_hash` を
normalized result store の unique key として lookup します。
該当 entry がなければ `source_normalized_result_not_found` を返します。
lookup 成功後、`source.normalized_result_id` が存在する場合は
参照先 `NormalizedCheckResult.normalized_result_id` と照合し、mismatch なら
`source_id_mismatch` を返します。
その後、参照先 `NormalizedCheckResult.comparison.status` に対する sidecar status 許可条件を検査します。
許可されない場合は `referenced_artifact_value_mismatch` を返します。

MVP の audit-sidecar validation request body field rules：

```json
{
  "schema_only": false,
  "sidecar": {
    "path": "build/audit/Std.Nat.ai.json"
  },
  "result_store": {
    "kind": "manifest",
    "path": "build/check-results/manifest.json",
    "manifest_hash": "sha256:..."
  },
  "normalized_store": {
    "kind": "manifest",
    "path": "build/normalized/manifest.json",
    "manifest_hash": "sha256:..."
  },
  "input_policy": {
    "path": "ci/phase8-ai-triage-default.json",
    "hash": "sha256:..."
  }
}
```

`schema_only = true` の場合、`result_store`、`normalized_store`、`input_policy` は
すべて forbidden です。
指定された場合は `validation_reference_schema_invalid` にします。
CLI では `--result-store-hash`、`--normalized-store-hash`、`--input-policy-hash` だけが
指定された場合も、それぞれ `result_store.manifest_hash`、`normalized_store.manifest_hash`、
`input_policy.hash` の forbidden presence として扱います。
`schema_only = false` の場合、`result_store` と `input_policy` は required です。
`input_policy.path` と `input_policy.hash` はどちらも required で、
CLI では `--input-policy` と `--input-policy-hash` に対応します。
`input_policy` reference は `kind` を持ちません。
API body で `input_policy.kind` が存在する場合は
`validation_reference_schema_invalid`、`error.field = "input_policy.kind"`、
`actual_value = "unknown_field"` を返します。
validator は `input_policy.hash` が省略された場合に file hash を暗黙採用してはいけません。
`result_store.path` と `result_store.manifest_hash` もどちらも required で、
CLI では `--result-store` と `--result-store-hash` に対応します。
`result_store.kind` と `normalized_store.kind` は required で、MVP では `manifest` だけを許可します。
`normalized_store` は sidecar が `source.normalized_result_hash` を持つ場合 required です。
sidecar が `source.normalized_result_hash` を持たない場合、`normalized_store` は optional です。
`normalized_store` を使う場合は `normalized_store.path` と `normalized_store.manifest_hash` の
両方が required で、CLI では `--normalized-store` と `--normalized-store-hash` に対応します。
optional `normalized_store` が指定された場合も、validator は manifest hash / schema / entry hashes を検証します。
ただし sidecar source 解決では `normalized_store` を使わず、
`source_normalized_result_hash`、`source_normalized_result_not_found`、
`normalized_result_missing_source` は報告しません。
MVP では `kind = manifest` だけを許可し、directory scan で hash から file を探してはいけません。

MVP の result store manifest：

```json
{
  "schema": "npa.phase8.machine_result_store_manifest.v1",
  "results": [
    {
      "result_hash": "sha256:...",
      "request_hash": "sha256:...",
      "run_artifact_hash": "sha256:...",
      "checker_profile": "reference",
      "path": "build/check-results/Std.Nat.reference.json",
      "file_hash": "sha256:..."
    }
  ]
}
```

MVP の normalized store manifest：

```json
{
  "schema": "npa.phase8.normalized_result_store_manifest.v1",
  "results": [
    {
      "normalized_result_hash": "sha256:...",
      "artifact_hash": "sha256:...",
      "path": "build/normalized/Std.Nat.json",
      "file_hash": "sha256:..."
    }
  ]
}
```

store manifest の `path` は workspace-relative path です。
`file_hash` は保存 file bytes の sha256 です。
validator は manifest file bytes hash を `manifest_hash` と照合し、各 entry の file を読み、
file bytes hash と parsed artifact hash を再計算します。
machine result store では、entry の hash は次の parsed `MachineCheckResult` field と一致しなければなりません。

```text
- entry.result_hash = MachineCheckResult.result_hash
- entry.request_hash = MachineCheckResult.request_hash
- entry.run_artifact_hash = MachineCheckResult.run_artifact_hash
- entry.checker_profile = MachineCheckResult.checker.profile
- entry.file_hash = result file bytes sha256
```

normalized result store では、entry の hash は次の parsed `NormalizedCheckResult` field と一致しなければなりません。

```text
- entry.normalized_result_hash = NormalizedCheckResult.normalized_result_hash
- entry.artifact_hash = NormalizedCheckResult.artifact_hash
- entry.file_hash = normalized result file bytes sha256
```

machine result store manifest entries は `run_artifact_hash` の bytewise lexicographic order で昇順に並べます。
machine result store では `run_artifact_hash` と `path` が unique key です。
同じ `result_hash` は retry で再利用されることがあるため、machine result store の unique key ではありません。
同じ `request_hash` も retry / multi-profile result で再利用されることがあるため、
machine result store の unique key ではありません。
同じ deterministic verdict を retry して `result_hash` が同じになっても、
`attempt`、`process`、`resource_usage`、`diagnostics` などを含む `run_artifact_hash` が異なれば
別の saved artifact として同じ store manifest に入れられます。
machine result store lookup では `run_artifact_hash` が canonical unique key です。
sidecar validator は `source.kind = machine_result` の参照を
`source.run_artifact_hash` で解決し、解決した artifact の `result_hash` と `request_hash` が
sidecar source と一致することを検査します。
`result_hash` だけで machine result store を検索してはいけません。
同じ `result_hash` を持つ retry result が複数あっても `run_artifact_hash` によって一意に解決します。

challenge replay aggregate は、`ChallengeManifest` と `RunnerPolicy` から required / optional profile ごとの
replay `MachineCheckRequest` を deterministic に再構成し、それぞれの `request_hash` を計算します。
この再構成は `npa-check challenge materialize-requests` と同じ field derivation を使います。
`certificate.expected_certificate_hash` は `ChallengeManifest.mutated_certificate.claimed_certificate_hash`
が存在する場合はその値、omit されている場合は
`ChallengeManifest.base_certificate.claimed_certificate_hash` を copy します。
aggregate はこの field のために mutated certificate file を読まず、raw claimed-hash extractor も再実行しません。
この再構成 request は in-memory validation object であり、aggregate command は request store を
書き換えてはいけません。
request store には同じ `request_hash` を持つ materialized request file が既に存在しなければなりません。
存在しない場合、または request store 内の request self hash が再構成 request の
`request_hash` と一致しない場合は、
`ChallengeReplayResult` を作らず challenge replay pipeline failure にします。
照合では `request_id` は無視します。
request file bytes は request store manifest の `file_hash` と照合しますが、
再構成 request との semantic 一致判定には使いません。
再構成 request と materialized request の `request_hash` 対象 field が1つでも異なる場合、
再計算した `request_hash` が一致しないため pipeline failure になります。
machine result store からは `(request_hash, checker_profile)` が一致する entry を探します。
0件ならその profile は missing result であり、`checker_results` には entry を作りません。
missing required profile は `ChallengeReplayResult.missing_checker_profiles` に入れます。
対応する `NormalizedCheckResult` を解決できた場合は、その
`NormalizedCheckResult.comparison.status = missing_checker_result` も併せて写されます。
`normalized_result_hash` を omit する informational replay では、
missing required profile は `missing_checker_profiles` だけで表現します。
missing optional profile は comparison の missing にも `missing_checker_profiles` にも含めません。
coverage-required replay でも、missing result 自体は aggregate pipeline failure ではありません。
ただし coverage-required replay では missing profile を反映した `NormalizedCheckResult` を
一意に解決できなければ pipeline failure であり、解決できた場合も
nightly / release pass condition では `comparison_status = all_agree_failed` ではないため fail です。
2件以上ある場合、retry attempt の選択が曖昧なので `ChallengeReplayResult` を作らず
challenge replay pipeline failure にします。
caller は aggregate replay 前に採用する attempt だけを含む filtered result store manifest を渡します。
coverage-required replay では、caller は aggregate replay 前に challenge checker results を
`npa-check normalize-results` で正規化し、その `NormalizedCheckResult` を含む
normalized result store manifest を渡します。
aggregate command は normalized result store を読み取り専用で使い、
`NormalizedCheckResult` を生成・保存・更新してはいけません。
normalized result store から対応する entry を探す規則は `# 10. Challenge generation` の
`ChallengeReplayResult.normalized_result_hash` 規則に従います。

normalized result store manifest entries は `normalized_result_hash` の bytewise lexicographic order で昇順に並べます。
normalized result store では `normalized_result_hash` と `path` が unique key です。

manifest schema に存在しない hash key は、その manifest 種別の unique key ではありません。
順序違反または unique key 重複は `*_store_manifest_invalid` です。

MVP の artifact selector：

```json
{
  "module": "Std.Nat",
  "request_hash": "sha256:..."
}
```

CLI では `--selector-module` と `--selector-request-hash` に対応します。
API では `/machine/check/normalize` request body の `artifact_selector` field に入れます。
selector を omit した場合の fallback は `# 6. NormalizedCheckResult` の
single-artifact convenience mode と同じです。

MVP の request store reference：

```json
{
  "kind": "manifest",
  "path": "build/check-requests/manifest.json",
  "manifest_hash": "sha256:..."
}
```

`kind = manifest` だけを MVP で許可します。
request store reference は closed-world object であり、`kind`、`path`、`manifest_hash` は required です。
`path` は workspace-relative path、`manifest_hash` は manifest file bytes の sha256 です。
CLI では `--request-store` と `--request-store-hash` の required pair から作ります。
API では endpoint wrapper field の `request_store` object から作ります。
API wrapper schema / path validation で検出できる reference object の missing、wrong type、
explicit null、unknown field、duplicate field、invalid kind、invalid hash format、
invalid path は `ApiError` で返し、`NormalizeErrorResult` を作りません。
CLI の `normalize-results` では、required pair が両方指定された後の path schema violation と
hash format violation は `NormalizeErrorResult.error.reason_code = request_store_reference_invalid` です。
manifest は `request_hash` から `MachineCheckRequest` file path と request file bytes hash への
map です。
API implementation は manifest file bytes hash を `manifest_hash` と照合してから request を解決します。
インメモリ map、database id、HTTP URL は MVP では使いません。

MVP の request store manifest schema：

```json
{
  "schema": "npa.phase8.request_store_manifest.v1",
  "requests": [
    {
      "request_hash": "sha256:...",
      "path": "build/check-requests/Std.Nat.reference.json",
      "file_hash": "sha256:..."
    }
  ]
}
```

`path` は workspace-relative path です。
`file_hash` は `MachineCheckRequest` file bytes の sha256 です。
manifest 内の `request_hash` は、その file を parse して 3.3 の規則で再計算した
`request_hash` と一致しなければなりません。
manifest entry の `path` が workspace-relative path schema に違反する場合は、
caller-specific な `request_store_manifest_invalid` または `input_store_manifest_invalid` として拒否し、
manifest-local invalid field path は `requests[<i>].path`、`actual_value = "invalid_path"` にします。
caller error body の `field` は caller-prefixed path を使います。
normalize / challenge replay / challenge materialize では
`request_store.requests[<i>].path`、release staging の source store では
`request_store.requests[<j>].path` のように、その caller が定義する store root prefix を付けます。
manifest schema / domain error は concrete index を含む field path で報告し、
manifest entry から解決した request file の IO / JSON / artifact schema / hash validation error は
`request_store.requests[]` wildcard path で報告します。
path schema violation の場合、entry file を読みに行ってはいけません。
`requests` は `request_hash` の bytewise lexicographic order で昇順に並べます。
同じ `request_hash` が2回以上出る manifest は invalid です。
同じ `path` が2回以上出る manifest も invalid です。
manifest generator はこの順序で書き出し、loader は順序違反と重複を
manifest file bytes の hash mismatch と区別して、caller-specific な
`request_store_manifest_invalid` reason として拒否します。
caller ごとの mapping は次に固定します。

```text
normalize-results:
  NormalizeErrorResult.error.reason_code = request_store_manifest_invalid

challenge replay:
  CommandError.reason_code = request_store_manifest_invalid

challenge materialize-requests の既存 request store:
  CommandError.reason_code = request_store_manifest_invalid
```

`request_store_manifest_hash_mismatch` は caller が指定した manifest file bytes hash と
実ファイル bytes hash が一致しない場合だけに使います。

これらの API は checker source / tactic / AI trace を verification input として受け取りません。
source や tactic を渡す API は Phase 8 の trust boundary を壊すため追加しません。

---

# 19. Implementation plan

Phase 8 AI Profile の実装順序：

```text
1. Define RunnerPolicy schema and canonical hash
2. Define import lock, MachineCheckRequest / MachineCheckResult schema
3. Implement checker runner with checker binary registry, allowlist, and fixed dynamic args
4. Store raw checker result before AI processing
5. Implement NormalizedCheckResult generator
6. Implement deterministic checker comparison
7. Implement AiAuditSidecar schema and validator
8. Add deterministic AuxiliaryResult commands for CI pass conditions
9. Add adversarial challenge manifest and deterministic mutation generator
10. Add challenge replay result store and coverage summary in nightly CI
11. Add release audit bundle generation / validation with explicit input references and AI sidecar metadata
12. Add training data exporter based only on checker labels with deterministic export manifest
```

AI integration は最後に入れます。
最初に必要なのは deterministic runner と result schema です。

---

# 20. Tests

MVP で必要なテスト：

```text
- AI sidecar cannot mark a certificate as checked
- missing raw MachineCheckResult makes sidecar invalid
- sidecar source hash mismatch is rejected
- checker result normalization is deterministic
- checker disagreement always fails comparison
- source-only evidence cannot produce MachineCheckResult
- tactic-only evidence cannot produce MachineCheckResult
- noncanonical certificate challenge is rejected by checker
- forbidden axiom challenge is rejected by policy
- challenge outcome_hint success cannot override checker failure
- prompt injection in theorem name is treated as data
- checker binary outside allowlist is rejected
- checker build_hash mismatch is rejected
- post-launch checker_build_hash mismatch becomes policy_failure
- MachineCheckResult checker.id and checker.build_hash are independent optional copy fields for identity failures
- post-launch identity failure records valid checker_id and checker_build_hash fields independently and omits only malformed identity fields
- launched checker results record runner-owned binary_id and binary_hash even when raw identity is not established
- malformed raw output normalized entries keep checker_binary_hash for launched results but omit checker_id and checker_build_hash
- explicit null is rejected as null_not_allowed unless a schema marks the field nullable
- top-level explicit null reports null_not_allowed, not wrong_type, for strict JSON object artifacts and envelopes
- RunnerPolicy schema failures use the fixed top-level, nested field, known duplicate, and unknown field priority before domain failures
- RunnerPolicy schema/domain field shapes distinguish RunnerPolicy-local paths from caller `policy.` prefixes and map local `$` to `policy`
- RunnerPolicy array/key domain failures report concrete 0-based indexes or concrete budget keys for invalid, duplicate, order, and unexpected entries
- MachineCheckRequest request_hash is required and distinct from request file_hash
- MachineCheckRequest top-level schema mismatch returns request_schema_invalid with fixed schema field shape
- MachineCheckRequest.module invalid name grammar returns request_schema_invalid / invalid_name_format
- MachineCheckRequest.policy.id / policy.version domain failures return request_schema_invalid before runner policy mismatch checks
- MachineCheckRequest schema / domain validation failures are reported before request_hash_mismatch
- MachineCheckResult.module invalid name grammar returns machine_result_schema_invalid / invalid_name_format in normalizer input validation
- MachineCheckResult.error.declaration invalid name grammar returns machine_result_schema_invalid / invalid_name_format in normalizer input validation
- structured error fields enforce fixed value shapes for declaration, core_path, section, offset, expected_hash, actual_hash, expected_value, and actual_value
- structured error expected_value / actual_value accept only deterministic JSON scalars, and use canonical JSON string only where explicitly specified
- diagnostics uses fixed source:code tokens, sorted bytewise, and never embeds raw stderr, OS text, paths, or free-form human text
- top-level schema mismatch actual_value follows each artifact's explicit field shape and is not inferred globally
- npa-check run requires an explicit RunnerPolicyReference and does not resolve policy from MachineCheckRequest alone
- npa-check run and /machine/check/certificate default attempt to 1 and copy explicit positive attempt without scanning result stores
- CLI commands requiring RunnerPolicyReference reject missing --policy-hash
- npa-check run malformed RunnerPolicyReference reports runner_policy_reference_invalid with the same member-level field shape as non-run policy_reference_invalid
- RunnerPolicyReference hash must match parsed RunnerPolicy canonical hash and MachineCheckRequest.policy.hash
- unreadable or hash-mismatched RunnerPolicyReference returns the dedicated runner policy reason_code
- unreadable checker executable returns checker_binary_file_unreadable before process launch
- checker identity manifest unreadable / hash mismatch / invalid schema returns the dedicated policy_failure reason_code before checker launch
- checker identity manifest top-level schema mismatch reports checker_identity_manifest.schema and never uses wrong_schema
- checker identity manifest requires generated_by and checkers, validates generated_by shape, and treats generated_by as provenance metadata rather than current runner identity
- checker identity manifest checkers use the same profile / checker_id / binary_id grammars as RunnerPolicy
- checker identity manifest pre-launch policy matching checks only MachineCheckRequest.checker_profile for that run
- raw checker_id / checker_build_hash missing is checker_identity_missing, not malformed raw output
- checker_version missing or mismatch does not reject an otherwise valid checker result
- checker.version is omitted from result_hash but included in run_artifact_hash
- CheckerRawResult requires schema and rejects unknown / duplicate / null fields as raw schema failures
- CheckerRawResult raw JSON/schema failures populate MachineCheckResult.error.field with checker_raw-prefixed fields and fixed expected/actual values
- CheckerRawResult raw schema failure priority reports missing or malformed error object before nested error.kind / error.reason_code
- CheckerRawResult status=failed reads schema-valid error.kind before deciding whether certificate_hash is required
- CheckerRawResult rejects raw policy_failure/resource_exhausted/timeout error.kind as invalid_enum
- CheckerRawResult permits missing module for decode/schema/noncanonical failure and checker_internal_error, but requires module before certificate_hash for ordinary failures
- CheckerRawResult ordinary failure kinds always require module and certificate_hash in MVP
- CheckerRawResult kind-specific required error hash fields are raw schema failures when missing or malformed
- CheckerRawResult checker_internal_error accepts only checker_reported_internal_error as raw error.reason_code
- CheckerRawResult error members are allowed or forbidden by error kind group, and forbidden nested fields return absent_for_error_kind before value validation
- CheckerRawResult permits certificate_hash on raw checker_internal_error only as an optional schema-valid top-level hash field
- CheckerRawResult forbidden status-dependent fields return forbidden_field before validating the forbidden field value
- CheckerRawResult.module invalid name grammar reports checker_raw.module / module_name / invalid_name_format
- CheckerRawResult.module invalid name grammar is raw schema failure, while valid but mismatching module is checker_module_mismatch
- raw module mismatch takes precedence over adopting exit 2 checker_reported_internal_error when module is schema-valid
- exit 1 with status checked is missing_rejection_error, exit 1 with checker_internal_error is malformed_rejection_output, and exit 2 with non-internal error is malformed_internal_error_output
- exit >= 3 never copies stdout / stderr into diagnostics and omits the diagnostics field
- checker_process:stderr_present is emitted only for non-empty stderr when the checker process returns exit 0/1/2
- checker_process:stdout_present is emitted only for non-empty malformed stdout in exit 0/1/2 invalid JSON or raw schema failure cases
- MachineCheckResult canonical output omits diagnostics when no diagnostic string exists
- malformed checker_version records only fixed diagnostics tokens and does not affect raw verdict adoption
- malformed RunnerPolicy fixed-value/domain failures map to runner_policy_invalid for run and policy_reference_invalid for non-run commands
- RunnerPolicy top-level schema mismatch reports schema and never uses wrong_schema
- invalid RunnerPolicy JSON in npa-check run reports runner_policy_invalid with field policy.path and actual_value invalid_json
- runner_policy_* MachineCheckResult failures copy MachineCheckRequest.policy into the required policy field
- API /machine/check/certificate malformed policy wrapper shape/hash/path failures return ApiError, not MachineCheckResult
- request pre-check policy_failure results use loaded RunnerPolicy in MachineCheckResult.policy and fixed reason field shapes
- malformed MachineCheckRequest returns MachineCheckRequestErrorResult, not MachineCheckResult
- /machine/check/certificate returns MachineCheckRequestErrorResult for request load failures
- inline /machine/check/certificate request load failures omit request_path and request_file_hash and use artifact-local error.field paths
- MachineCheckResult with wrong result_hash is rejected
- MachineCheckResult with wrong request_hash is rejected
- MachineCheckResult runner / checker / process / resource_usage nested schemas reject unknown fields
- MachineCheckResult process.launched=false forbids exit_code and termination_reason
- post-launch timeout without exit code requires process.termination_reason = timeout
- MachineCheckResult process forbids termination_reason when exit_code is present
- killed_without_exit_status maps to checker_internal_error/process_exit_failure
- MachineCheckResult resource_usage uses non-negative integer fields with zero values for not-launched runs
- RunnerPolicy required_checker_profiles must match the trust_mode table
- RunnerPolicy domain validation uses fixed field / expected_value / actual_value shapes
- RunnerPolicy id/version/checker_id/binary_id/budget domains reject invalid names, non-positive integers, and integer overflow
- high-trust required_checker_profiles includes release profiles plus high-trust-reference
- RunnerPolicy checker profile names are path-safe and reject slash, dot segments, uppercase, whitespace, and control characters
- RunnerPolicy checker_allowlist is sorted by profile and rejects duplicate profile or binary_id
- RunnerPolicy checker_allowlist and budgets profile sets exactly match required plus optional profiles
- runner command construction appends import, trust, axiom policy, and budget dynamic args after allowed_args and before certificate path
- RunnerPolicy checker_allowlist.allowed_args accepts only single-argv visible ASCII `--flag` or `--flag=value` static options and rejects positional args, `--`, and runner-owned dynamic flags
- runner command construction clears inherited environment and passes only LC_ALL, LANG, and TZ fixed values
- checker executable resolution uses runner-owned CheckerBinaryRegistry and validates final target bytes against binary_hash
- checker executable symlink escape outside the selected registry root is rejected before binary hash validation and launch
- runner rejects axiom policy file hash mismatch before checker launch
- sidecar input_policy hash mismatch is rejected
- request certificate file_hash mismatch is rejected before checker launch
- import manifest_hash mismatch is rejected before checker launch
- import lock manifest entries are sorted, path-bound, and verified by file bytes hash before checker import use
- import lock manifest v1 rejects entries missing certificate.certificate_hash
- npa-check run maps invalid import lock manifest JSON / schema / domain to request_import_manifest_invalid before checker launch
- import lock imports[].module invalid name grammar returns request_import_manifest_invalid / invalid_name_format
- ImportLockManifest schema/domain first failure uses the fixed field order, array index order, and duplicate module/path tie-breakers
- ImportLockManifest order validation treats equal adjacent sort keys as duplicate candidates, not order_violation, and reports the first decreasing concrete index
- malformed CheckerRawResult becomes checker_internal_error
- CheckerRawResult module mismatch becomes checker_internal_error
- policy_failure uses reason_code and does not hash human text
- MachineCheckResult infrastructure reason_code is closed enum
- checked-result sidecar omits classification.checker_error_kind
- NormalizedCheckResult failed entry includes failure_key
- NormalizedCheckResult failed entry rejects failure_key that is not derived from sibling error
- NormalizedCheckResult failed entry reports failure_key object schema/name errors before failure_key_mismatch
- NormalizedCheckResult artifact identity ignores request_hash
- NormalizedCheckResult has top-level artifact_hash matching the artifact object hash
- NormalizedCheckResult results[] conditional forbidden fields use forbidden_field without validating the forbidden value shape
- NormalizedCheckResult results are ordered by RunnerPolicy profile order
- NormalizedCheckResult comparison disagreements require hash members for hash/failure_key fields, value members for status, and forbid the opposite member family
- NormalizedCheckResult comparison disagreements schema validation reports required member failures before forbidden member presence with fixed member order
- NormalizedCheckResult comparison disagreements duplicate field/checker_profile pairs use duplicate_entry and fixed comparison.disagreements[<i>] fields
- network import resolution is rejected in Phase 8 runner
- `npa-check run` short form cannot override policy budget or checker path
- all_agree_failed requires matching failure_key, not only matching error.kind
- all_agree_failed compares validated failure_key canonical hashes, not unchecked embedded values
- optional checker result conflicts become disagreement, while missing optional result is ignored
- checker profile outside RunnerPolicy produces comparison policy_failure with checker_profile_not_allowed, not NormalizeErrorResult
- missing required checker profiles are recorded in comparison.missing_checker_profiles
- process_launched=false policy_failure result without malformed process state is comparison policy_failure, not inconclusive
- comparison policy_failure and inconclusive details are recorded in status_reasons
- comparison-generated status_reasons reason_code is separate from MachineCheckResult reason_code
- NormalizedComparisonReasonCode accepts copied MachineCheckResult reason_code values plus comparison-generated values only
- comparison-generated reason codes map to fixed error_kind values
- comparison-generated checker identity reasons use fixed field / expected / actual shapes
- launched checked results missing checker_id or checker_build_hash become comparison policy_failure
- malformed process state conditions produce comparison-generated inconclusive with malformed_process_state
- comparison disagreement entries are emitted for every deterministic mismatch
- comparison status_reasons sort omitted checker_profile and field deterministically
- failure_key disagreement uses canonical failure_key hash, not embedded object values
- resource_exhausted comparison is inconclusive and fails CI
- same certificate checked twice produces same normalized result
- normalized_result_hash ignores nested results[*].result_id
- compare rejects NormalizedCheckResult whose artifact_hash does not match artifact object
- NormalizedCheckResult comparison disagreements are sorted and schema-stable
- claimed-hash absent challenge request uses deterministic expected_certificate_hash placeholder
- challenge generate --kind accepts only the MVP rejection-required mutation kind enum, while stored ChallengeManifest.mutation.kind may be any grammar-valid informational kind
- informational ChallengeManifest.mutation.target is an opaque visible-ASCII target label and is not resolved through MVP target classes
- ChallengeGenerationRequest requires policy_hash, module, imports, base_certificate, mutation kind/target/seed, and output paths
- ChallengeGenerationRequest.challenge_id is copied exactly to ChallengeManifest.challenge_id
- ChallengeGenerationRequest.policy_hash is copied exactly to ChallengeManifest.policy_hash
- ChallengeGenerationRequest module and imports are copied exactly to ChallengeManifest
- ChallengeGenerationRequest imports.mode is required and must match RunnerPolicy.import_policy.mode
- challenge generate maps unreadable or symlink-escaped import manifests to import_manifest_file_unreadable
- challenge commands report RunnerPolicyReference.hash vs parsed policy hash mismatch before request or ChallengeManifest policy_hash mismatch
- ChallengeGenerationRequest request_hash is required and must match the canonical self hash before generation reads inputs or writes outputs
- ChallengeGenerationRequest schema / domain validation failures are reported before generation_request_hash_mismatch
- ChallengeGenerationRequest schema / domain first failure uses the fixed top-level and nested field order, including unknown / duplicate fields
- CLI challenge generate validates all request-field schema / domain failures, including output path schema failures, before reading --from
- CLI challenge generate may read --from only after construction schema validation and before request_hash validation, and that phase performs no output writes
- ChallengeGenerationRequest.module and name-target mutation.target invalid grammar return generation_request_schema_invalid / invalid_name_format
- ChallengeManifest.module and name-target mutation.target invalid grammar return challenge_manifest_schema_invalid / invalid_name_format
- ChallengeGenerationRequest whole-certificate mutation.target values other than $whole_certificate return generation_request_schema_invalid / invalid_enum, not mutation_target_invalid
- challenge generate computes base_certificate file_hash and claimed_certificate_hash from --from
- challenge generation API revalidates base_certificate file_hash and claimed_certificate_hash from file bytes
- challenge generate --json and /machine/check/challenge success return ChallengeGenerationResult without certificate bytes
- challenge generate failure returns CommandError on stderr/API body and no ChallengeGenerationResult
- challenge generate requires --generated-by and enforces --prompt-hash only for generated_by = ai
- conflicting duplicate ChallengeGenerationRequest.challenge_id in an output store is generation failure, while exact entry retry is idempotent success
- challenge generate requires --challenge-store and checks duplicate challenge_id only against that store manifest
- challenge generate writes a sorted ChallengeOutputStoreManifest entry on success
- challenge generate reports ChallengeOutputStoreManifest schema/order/duplicate failures with fixed challenge_output_store_manifest_invalid fields
- challenge generate reports unreadable or symlink-escaped existing ChallengeOutputStoreManifest entry manifests with challenge_output_store_entry_manifest_unreadable
- challenge generate validates referenced existing ChallengeOutputStoreManifest entry manifests with manifest-local ChallengeManifest JSON / schema / domain rules and reports challenge_output_store_entry_manifest_invalid
- challenge generate reports the first failing existing ChallengeOutputStoreManifest entry in stored order and applies unreadable, invalid, then hash-mismatch order within that entry
- challenge generate may atomically update ChallengeOutputStoreManifest but refuses to overwrite differing manifest-out and mutated-out artifacts
- challenge generate treats ChallengeOutputStoreManifest as commit point and can adopt exact-match orphan manifest / mutated certificate files on retry
- ChallengeGenerationRequest request_hash matches ChallengeManifest.replay.args_hash
- challenge generate rejects schema-valid kind-specific missing / mismatching / stale mutation.target with mutation_target_invalid before writing artifacts
- challenge mutation selection uses first 8 hex-decoded seed bytes as unsigned big-endian modulo candidate_count for both structured and byte-level candidate sets
- challenge mutation kinds have a fixed structured vs byte-level classification, and byte-level candidate/layout absence returns mutation_target_invalid instead of base_certificate_decode_failed
- byte-level challenge mutation offsets are original file byte offsets and, after base claimed-hash extraction, require only common source-format and mutation-specific raw header / section framing
- base certificate raw claimed-hash extractor rejects malformed raw top-level section framing and malformed ModuleHashes local payload before byte-level source format failure
- all byte-level challenge mutations require base Header.format = NPA-CERT-0.1 after claimed-hash extraction and before candidate collection
- insert_unsupported_schema_version writes mutated_certificate.claimed_certificate_hash when the raw claimed-hash extractor still finds exactly one ModuleHashes.certificate_hash field/value range after complete raw framing and ModuleHashes local validation
- ChallengeManifest manifest-local validation treats mutated_certificate.claimed_certificate_hash as optional and never re-runs the raw claimed-hash extractor
- byte-level source format failure normalizes non-token Header.format diagnostics to unexpected_header_format:invalid_string without copying raw bytes
- byte-level mutation maps top-level raw framing failures to base_certificate_claimed_hash_decode_failed, not mutation_target_invalid
- byte-level mutation reports duplicate, out-of-order, overlapping, or out-of-file top-level sections as base_certificate_claimed_hash_decode_failed; missing sections are fatal only when required by that mutation
- flip_canonical_encoding_byte candidates include top-level section payload bytes only, excluding section framing and ModuleHashes.certificate_hash value bytes
- flip_canonical_encoding_byte treats missing ModuleHashes as base_certificate_claimed_hash_decode_failed but missing other required sections as mutation_target_invalid
- flip_canonical_encoding_byte does not remap non-unique ModuleHashes.certificate_hash field/value ranges; those fail as base_certificate_claimed_hash_decode_failed
- mutation_target_invalid reports fixed field shapes for target rule failures, raw layout failures, and empty candidate sets
- truncate_certificate_section ignores missing section kinds as absent candidates and fails only when no non-empty candidate remains
- truncate_certificate_section uses floor division, excludes zero-length sections, and always deletes a non-empty suffix
- challenge structured mutation canonical byte offsets are measured in the verified Phase 2 canonical re-encoded ModuleCertBytes, and noncanonical original layouts are rejected before mutation
- challenge structured mutation rejects noncanonical Phase 2 binary decode / re-encode mismatch as base_certificate_decode_failed but does not run import, hash, kernel, or axiom-policy verification before generation
- challenge structured mutations have fixed encoded-byte-patch vs object-mutation execution classes, and object mutations re-encode canonical length/table bytes while preserving stale stored hashes unless specified
- reorder_declarations swaps only adjacent DeclCert objects in the Declarations vector and does not rewrite local indices, axiom report indices, export block, dependencies, tables, or stored hashes
- alter_universe_constraint is not accepted by the Phase 2 NPA-CERT-0.1 MVP challenge generator closed enum
- change_declaration_body_without_hash mutates only DefDecl.value / TheoremDecl.proof reachable TermTable payload bytes and does not clone shared TermTable nodes
- change_declaration_body_without_hash excludes all hash-typed bytes, including GlobalRef::Imported decl_interface_hash inside TermNode::Const, from mutable payload candidates
- change_declaration_hash_without_body chooses between decl_interface_hash and decl_certificate_hash by canonical byte offset and seed
- drop_axiom_report_entry reports missing per-declaration axiom report entry as empty candidate set, not target lookup failure
- term graph mutation candidates are reachable only from declaration-kind-specific DeclPayload term roots and never from export block, axiom report, imports, or side data
- import-target mutations require exactly one matching ImportEntry.module_name and reject zero or multiple matches without choosing by hash
- remove_dependency_entry supports declaration and import targets with deterministic candidate ordering and rejects ambiguous declaration/import target matches
- add_forbidden_axiom recomputes certificate hashes while other MVP mutations leave stored hashes stale unless specified
- add_forbidden_axiom appends an empty-universe AxiomDecl at declaration tail with Sort Level::Zero and self axiom dependency
- add_forbidden_axiom self dependency uses GlobalRef::Local(new_decl_index) and checks freshness only in current module declarations/export block
- add_forbidden_axiom does not repair pre-existing Declarations / ExportBlock inconsistency and rejects a target present in either structure
- replace_nat_zero_with_noncanonical_placeholder counts reachable TermTable nodes, not reference edges
- replace_nat_zero_with_noncanonical_placeholder matches only empty-level Const nodes whose GlobalRef name resolves inside the encoded certificate to Name(["Nat", "zero"])
- replace_nat_zero_with_noncanonical_placeholder requires LocalGenerated references to point at an InductiveDecl constructor or recursor with the embedded name before matching Nat.zero
- replace_nat_zero_with_noncanonical_placeholder writes reserved invalid term tag 0xff and does not add placeholder to core calculus
- challenge generate rejects existing manifest or mutated certificate paths only when file bytes differ from generated bytes
- MVP challenge mutation accepted by a required checker is unexpected checker acceptance
- challenge commands treat missing policy flags as CLI argument errors, malformed provided policy references as CommandError policy_reference_invalid, and API wrapper policy shape/path failures as ApiError
- ChallengeReplayResult manifest_hash is the ChallengeManifest file bytes hash
- ChallengeReplayResult mutated_claimed_certificate_hash is copied from ChallengeManifest and replay does not re-read the mutated certificate for extraction
- ChallengeReplayResult artifact_hash comes from NormalizedCheckResult or replay request artifact
- challenge materialize-requests creates policy-ordered replay MachineCheckRequest files and a request store manifest without running checkers
- challenge materialize-requests derives module/imports from ChallengeManifest and certificate fields from mutated_certificate
- challenge materialize-requests derives expected_certificate_hash from ChallengeManifest mutated/base claimed hashes without reading the mutated certificate file
- challenge replay aggregate reconstructs request_hash with the same manifest-based expected_certificate_hash rule as materialize-requests
- challenge materialize-requests rejects ChallengeManifest.policy_hash mismatch with RunnerPolicyReference.hash
- challenge materialize-requests rejects ChallengeManifest.imports.mode mismatch with RunnerPolicy.import_policy.mode
- challenge materialize-requests returns ChallengeRequestMaterializationResult with request store manifest_hash
- challenge materialize-requests treats request store manifest as commit point and can adopt exact-match orphan files on retry
- challenge materialize-requests failure returns CommandError on stderr/API body and no ChallengeRequestMaterializationResult
- challenge materialize-requests CommandError field shapes are fixed for manifest, policy, import, output, and store failures
- challenge materialize-requests malformed manifest path/hash pair and output paths return input_reference_invalid with fixed field shapes
- challenge materialize-requests rejects request_store_output_path overlapping a generated request file path as request_output_path_conflict
- challenge materialize-requests reports simultaneous generated request path conflicts and write failures by required profile order, then optional profile order
- challenge materialize-requests rejects existing request store entry file_hash and parsed request_hash mismatches with dedicated CommandError reason codes
- challenge materialize-requests rejects unreadable, invalid JSON, or schema-invalid existing request store entry files with dedicated CommandError reason codes
- challenge materialize-requests and challenge replay require --manifest-hash for read-only ChallengeManifest input
- challenge replay rejects ChallengeManifest.policy_hash mismatch with RunnerPolicyReference.hash
- challenge replay aggregate command consumes manifest, policy, request store, result store, and normalized store for coverage-required replay
- challenge replay aggregate treats request store as read-only and fails if materialized requests are missing
- challenge replay request-store comparison ignores request_id and validates request_hash semantics
- challenge replay artifact mismatch does not produce ChallengeReplayResult
- challenge replay with a missing required result records missing_checker_profiles and, when normalized result exists, missing_checker_result
- challenge replay fails when result store has multiple attempts for the same request_hash/profile
- challenge replay pipeline failure returns CommandError on stderr/API body and no ChallengeReplayResult
- challenge replay rejects request/result/normalized store entry file_hash and parsed artifact hash mismatches with dedicated CommandError reason codes
- challenge replay rejects unreadable, invalid JSON, or schema-invalid request/result/normalized store entry files with dedicated CommandError reason codes
- challenge replay --out / --replay-store-out writes ChallengeReplayResult and sorted ChallengeReplayStoreManifest atomically
- challenge replay --replay-store-out creates an empty replay store manifest when the specified manifest file is absent
- challenge replay --replay-store-out treats existing replay store entries as idempotent only when all entry fields match
- challenge replay store entry challenge_id mismatch uses replay_store_entry_challenge_id_mismatch
- challenge replay store entry manifest_hash and artifact_hash mismatches use dedicated replay_store_entry_*_hash_mismatch reason codes
- ChallengeReplayStoreManifest rejects duplicate result_hash, duplicate path, and duplicate challenge_id/manifest_hash pairs
- challenge replay store entry top-level schema mismatch reports the schema field and never uses wrong_schema
- store entry schema invalid fields use wildcard-prefixed artifact paths such as result_store.results[].checker.profile
- store entry artifact validation order is unreadable, JSON, schema, file_hash, self-hash, manifest-field mismatch
- challenge replay and sidecar store validation distinguish artifact self-hash mismatch from manifest-entry hash mismatch
- challenge replay and sidecar validate multi-hash store artifacts in the fixed self-hash order
- informational ChallengeReplayResult omits comparison_status when normalized_result_hash is omitted but still records missing_checker_profiles
- coverage-required ChallengeReplayResult fails if normalized_result_hash cannot be resolved from normalized store
- ChallengeReplayResult checker_results are profile-unique and policy-ordered
- ChallengeReplayResult checker_results include run_artifact_hash for exact saved result identity
- ChallengeReplayResult result_hash ignores nested checker_results[*].result_id
- normalize-results with --normalized-store-out writes a sorted normalized result store manifest entry atomically
- normalize-results with --normalized-store-out creates an empty store when the specified manifest file is absent
- normalize-results without --normalized-store-out does not read, create, or update a normalized store manifest
- normalize-results rejects --normalized-store-out without --out as a CLI argument error
- normalize-results treats normalized store manifest as commit point and can adopt exact-match orphan output on retry
- normalize-results --out --json returns NormalizationWriteResult with output_file_hash and normalized store manifest_hash
- normalize normalized store entry file_hash mismatch uses normalized_store_entry_file_hash_mismatch
- normalize existing normalized store unreadable or invalid JSON maps to normalized_store_manifest_invalid
- normalize request_store_manifest_invalid uses fixed field / expected_value / actual_value shapes including invalid_path for manifest entry paths
- normalize request_store_manifest_invalid reports manifest entry path errors with caller-prefixed fields such as request_store.requests[<i>].path
- normalize maps malformed CLI request_store reference path/hash to request_store_reference_invalid with fixed field shapes
- raw normalizer input envelope maps malformed RequestStoreReference JSON-like objects to request_store_reference_invalid for library-level tests
- normalize public API never returns request_store_reference_invalid for request_store wrapper shape/hash/path failures; those are ApiError
- normalize-results treats missing or one-sided --request-store / --request-store-hash as CLI argument errors without a NormalizeErrorResult body
- compare without resolvable RunnerPolicy is rejected
- compare CLI requires --policy and --policy-hash
- normalize maps broken request store files to NormalizeErrorResult
- normalize rejects non-MachineCheckResult inputs with machine_result_wrong_schema
- normalize separates machine_result_wrong_schema from machine_result_schema_invalid for schema null/type/unknown/duplicate cases
- normalize rejects MachineCheckResult result_hash / run_artifact_hash mismatch
- normalize rejects MachineCheckResult whose request_hash disagrees with request store
- normalize request store manifest request_hash mismatch reports expected_hash from manifest and actual_hash from parsed request
- normalize maps wrong-schema request store entries to request_schema_invalid with request_store.requests[].schema
- normalize maps missing request_hash inside a resolved MachineCheckRequest file to request_hash_missing, while missing manifest entry request_hash is request_store_manifest_invalid
- normalize maps malformed artifact_selector object/member to selector_schema_invalid with fixed field shapes
- normalize maps artifact_selector.module invalid name grammar to selector_schema_invalid / invalid_name_format
- normalize maps artifact_selector.request_hash invalid hash format to selector_schema_invalid / invalid_hash_format
- normalize validates artifact_selector schema before request store resolution and request_hash_not_found checks
- normalize validates artifact_selector schema and checker_profile uniqueness before policy file resolution
- normalize validates request_store reference schema/path before policy file resolution, but validates request store manifest after policy validation
- normalize validates policy before request store manifest and validates request store manifest before selector_ambiguous
- normalize / compare / challenge policy_reference_invalid prefixes policy file RunnerPolicy schema/domain fields with `policy.`
- normalize reports request_hash_not_found before selector_module_mismatch when explicit selector request_hash cannot be resolved
- normalize maps artifact_selector.module mismatch to selector_module_mismatch with field artifact_selector.module
- normalize without request store entry for request_hash is rejected
- normalize request_hash_not_found reports artifact_selector.request_hash or machine_results[].request_hash by source
- normalize request_hash_not_found checks all input MachineCheckResult entries in deterministic normalized order
- normalize write-stage conflicts use NormalizeErrorResult reason codes for output/store failures
- policy mismatch takes precedence over artifact disagreement
- AuxiliaryResult diagnostics do not affect result_hash
- machine result store uses run_artifact_hash, not result_hash, as the retry-safe unique key
- machine_result sidecar source requires result_hash, request_hash, and run_artifact_hash
- checker_id mismatch records expected_value and actual_value
- NormalizeErrorResult is returned instead of partial NormalizedCheckResult
- request store reference manifest hash mismatch is rejected
- CLI read-only inputs require matching --manifest-hash / --request-store-hash / --result-store-hash / --normalized-store-hash
- challenge replay malformed read-only manifest/store path/hash pairs return input_reference_invalid with fixed field shapes
- API challenge replay malformed manifest/store reference object, hash format, or path validation failures return ApiError, not input_reference_invalid
- challenge replay manifest hash mismatch reason codes apply only to read-only input references, not replay_store output writes
- API challenge replay allows normalized_store omission when coverage_required is false but rejects missing required references
- store manifest entry invalid workspace-relative path is a manifest invalid error and is not followed by entry file IO
- store manifest schema/domain errors use concrete indexed field paths, while entry file/artifact/hash errors use wildcard field paths
- API store references require manifest_hash and reject path-only store references
- API ChallengeManifest references require manifest_hash and reject path-only manifest references
- API invalid JSON request bodies return ApiError with api_json_invalid
- API wrapper schema and workspace path validation failures return ApiError, not endpoint artifacts or CommandError
- API duplicate-aware decoder rejects endpoint wrapper duplicate keys as api_request_schema_invalid with duplicate_field, except active audit-sidecar validation references
- API audit-sidecar active validation reference duplicate keys return AuditSidecarValidationResult validation_reference_schema_invalid, not ApiError
- API duplicate schema_only mode discriminator returns api_request_schema_invalid before mode-dependent reference validation
- API duplicate keys inside inline artifacts are routed to endpoint-specific schema validation failures
- API challenge generation output/base/import paths inside inline ChallengeGenerationRequest return CommandError generation_request_schema_invalid, not ApiError
- API duplicate keys inside mode-forbidden reference payloads do not override the forbidden-reference validation_reference_schema_invalid path
- API audit-sidecar duplicate-free validation reference path schema failures return api_path_outside_workspace, not validation_reference_schema_invalid
- API audit-sidecar duplicate-free invalid validation reference paths take precedence over invalid hash, unknown field, and non-path missing member failures
- API audit-sidecar active validation reference duplicate keys are checked before workspace path validation
- API audit-sidecar missing active validation references return AuditSidecarValidationResult validation_reference_missing, not ApiError
- API audit-sidecar partial validation references return AuditSidecarValidationResult validation_reference_schema_invalid only when duplicate-free invalid path does not take precedence
- API audit-sidecar existing reference objects with missing required members return validation_reference_schema_invalid, not validation_reference_missing
- API audit-sidecar input_policy.kind is rejected as validation_reference_schema_invalid unknown_field
- API domain file read/write failures use endpoint-specific error schemas after wrapper validation succeeds
- API normalize and compare wrappers accept inline artifacts only, while challenge replay uses manifest/store references
- normalize selector module mismatch returns NormalizeErrorResult
- omitted normalize selector is rejected when first required profile has zero or multiple results
- normalize uses RunnerPolicy.axiom_policy.hash for artifact.axiom_policy_hash
- policy_file_unreadable NormalizeErrorResult keeps policy_hash from RunnerPolicyReference
- NormalizeErrorResult omits policy_hash when RunnerPolicyReference.hash is missing, non-string, null, or invalid format
- NormalizeErrorResult policy_hash presence is decided by lightweight RunnerPolicyReference.hash decode even when an earlier endpoint validation failure wins
- NormalizeErrorResult policy_reference_invalid / policy_file_unreadable / policy_hash_mismatch use fixed field shapes
- API normalize policy wrapper schema/hash/path failures return ApiError, while policy file unreadable/invalid/hash mismatch returns NormalizeErrorResult
- API normalize request_store wrapper schema/hash/path failures return ApiError, while request store manifest unreadable/hash/JSON/schema failures return NormalizeErrorResult
- API normalize malformed artifact_selector value/member failures return NormalizeErrorResult selector_schema_invalid, while duplicate top-level artifact_selector field remains ApiError
- API normalize reports selector_schema_invalid before policy_file_unreadable when both endpoint-specific failures are present
- duplicate checker_profile in normalize input returns NormalizeErrorResult
- compare rejects a NormalizedCheckResult whose embedded comparison does not match recomputation
- CompareValidationResult policy_failure keeps normalized_result_hash and embedded_comparison_status, and omits recomputed_comparison_status
- CompareValidationResult policy_failure uses fixed policy / policy.path / policy.hash field shapes
- compare CLI missing policy flags is a CLI argument error, while malformed provided policy references return CompareValidationResult policy_reference_invalid
- API compare policy wrapper schema/hash/path failures return ApiError, while policy file unreadable/invalid/hash mismatch returns CompareValidationResult policy_failure
- normalize / compare / challenge replay APIs use RunnerPolicyReference, not policy hash store lookup
- machine API path reads and writes use the API process server workspace with CLI-equivalent side effects
- request store manifest order violation or duplicate entry is rejected
- request store manifest order/duplicate failures map to caller-specific request_store_manifest_invalid, not request_store_manifest_hash_mismatch
- challenge replay store manifest duplicate reason codes are manifest-kind-specific
- audit-sidecar step 3 sidecar schema/source shape/static forbidden failures use the fixed intra-step priority
- audit-sidecar step 5 validation reference failures use the fixed missing/partial/schema/path priority
- sidecar result store manifest hash mismatch is rejected
- sidecar result store duplicate run_artifact_hash or path is rejected
- sidecar normalized store duplicate normalized_result_hash or path is rejected
- sidecar result store entry hashes must match parsed result fields
- sidecar result store checker_profile must match parsed MachineCheckResult
- sidecar machine_result lookup uses run_artifact_hash, not result_hash alone
- sidecar machine_result lookup miss reports source_result_not_found on source.run_artifact_hash
- sidecar source hash mismatch checks result_hash before request_hash after run_artifact_hash lookup
- machine_result sidecar normalized_result_hash must contain source.result_hash
- machine_result sidecar normalized_result_hash is semantic membership only and does not prove exact selected raw run membership
- machine_result sidecar normalized_result_id without normalized_result_hash is rejected before normalized store lookup
- machine_result sidecar normalized membership failure takes precedence over normalized_result_id mismatch
- normalized_comparison sidecar forbidden source hash/id members return forbidden_sidecar_field with fixed priority
- sidecar source id fields are optional but must match referenced artifacts when present
- sidecar source_id_mismatch uses expected_value from sidecar source id and actual_value from referenced artifact id
- AiAuditSidecar status-dependent required fields are enforced
- AiAuditSidecar source-status rules allow normalized_comparison missing_checker_result and policy_failure required targets
- audit-sidecar schema-only validation does not enforce source-artifact-dependent sidecar status permissions
- audit-sidecar schema-only validation treats machine_result classification.checker_error_kind as optional enum-only metadata
- audit-sidecar normalized_comparison classification.checker_error_kind reports forbidden_sidecar_field before enum validation
- audit-sidecar cross-artifact classification.checker_error_kind missing, mismatch, or checked-result presence maps to referenced_artifact_value_mismatch with fixed field shape
- audit-sidecar classification.checker_error_kind mismatch takes precedence over sidecar status permission mismatch
- audit-sidecar cross-artifact validation rejects sidecar status not allowed by the referenced source artifact status
- audit-sidecar input_policy_field_mismatch reports copied metadata fields in id/version/included_fields/redaction order
- normalized_comparison sidecar lookup validates normalized_result_id before source status permission
- AiAuditSidecar optional classification omits checker_error_kind checks for summarized and inconclusive sidecars
- AiAuditSidecar source/input_policy/ai nested required fields are enforced
- AiAuditInputPolicy included_fields rejects duplicates, unknown fields, and order violations
- AiAuditInputPolicy schema invalid reports input_policy-prefixed error.field values, including top-level non-object and schema mismatch
- AiAuditInputPolicy schema/domain first failure uses schema field order before id/version/included_fields domain priority
- AiAuditInputPolicy treats included_fields unsupported/order/duplicate failures as local domain priority even though their reason_code is input_policy_schema_invalid
- AiAuditInputPolicy included_fields and sidecar copied included_fields report unknown, duplicate, and order failures with concrete element indexes
- AiAuditInputPolicy included_fields and sidecar copied included_fields use fixed expected_value strings for allowed field, uniqueness, and bytewise order failures
- sidecar copied input_policy.included_fields duplicate or order violation returns sidecar_schema_invalid
- policy-gated source/tactic sidecar fields are not generic unknown fields and are checked against input policy at step 7
- policy-gated source/tactic sidecar fields are only allowed at top-level paths; nested occurrences are forbidden_sidecar_field
- duplicate keys in sidecar/input-policy/store files are schema invalid with duplicate_field, not JSON invalid
- duplicate object keys take precedence over value schema failures for the same logical field
- invalid JSON AiAuditInputPolicy file returns input_policy_json_invalid
- schema/domain invalid AiAuditInputPolicy file returns input_policy_schema_invalid
- AiAuditInputPolicy copied metadata must match the policy file
- audit-sidecar step 6 reports unreadable input policy, invalid JSON/schema/domain, hash mismatch, and copied metadata mismatch in fixed priority order
- audit-sidecar cross-artifact validation requires validation reference input_policy.hash, sidecar input_policy.hash, and input policy file canonical hash to match
- audit-sidecar input_policy hash mismatch precedence reports reference-vs-sidecar before reference-vs-file before sidecar-vs-file
- audit-sidecar schema-only validation does not mark cross-artifact claims as validated
- audit-sidecar schema-only validation rejects cross-artifact validation references
- audit-sidecar cross-artifact validation requires result store and input_policy
- audit-sidecar cross-artifact validation maps missing active reference pairs to validation_reference_missing, not CLI argument error
- CLI/non-API audit-sidecar cross-artifact validation maps partial path/hash reference pairs to validation_reference_schema_invalid with actual_value missing
- API audit-sidecar partial active references return validation_reference_schema_invalid only when duplicate-free invalid path does not take precedence
- audit-sidecar validation reference explicit null returns validation_reference_schema_invalid with actual_value null_not_allowed
- audit-sidecar cross-artifact validation rejects missing --input-policy-hash as validation_reference_schema_invalid
- audit-sidecar schema-only forbidden validation references return validation_reference_schema_invalid with actual_value present
- audit-sidecar schema-only forbidden validation reference null still returns actual_value present
- audit-sidecar schema-only hash-only forbidden flags return validation_reference_schema_invalid with actual_value present
- audit-sidecar CLI mode is selected only by --schema-only presence; duplicate or unsupported mode flags are CLI argument errors
- audit-sidecar schema-only sidecar unreadable/json/schema failures take precedence over forbidden validation references
- CLI audit-sidecar invalid workspace-relative validation paths in active references return validation_reference_schema_invalid with actual_value invalid_path
- audit-sidecar store manifest unreadable, invalid JSON, schema duplicate field, and unique-key duplicate failures use fixed manifest invalid field shapes
- audit-sidecar store manifest duplicate_run_artifact_hash is result-store-only and duplicate_normalized_result_hash is normalized-store-only
- audit-sidecar cross-artifact validation validates provided optional normalized_store, but does not use it for source lookup unless source.normalized_result_hash exists
- audit-sidecar validation handles unreadable and invalid JSON sidecar files deterministically
- audit-sidecar validation order is fixed and reports only the first failing step
- audit-sidecar validation reference errors use validation_reference_* reason codes
- audit-sidecar fixed wrapper schema failures for schema_only/sidecar/sidecar.path missing, wrong type, or null return ApiError, not AuditSidecarValidationResult
- API audit-sidecar wrapper JSON parse and wrapper schema failures return api_json_invalid or api_request_schema_invalid before AuditSidecarValidationResult
- PR mode optional AI sidecar cross-artifact validation uses an explicit AiAuditInputPolicy file/hash and never falls back to ReleasePolicy
- PR mode AI sidecar without explicit input policy can only be schema-only validated and is not a cross-artifact validated CI diagnostic artifact
- AuditSidecarValidationResult omits source_kind when cross-artifact sidecar source.kind is unavailable
- AuditSidecarValidationResult omits source hash fields when the corresponding sidecar source hash is invalid
- AuditSidecarValidationResult field requirements depend on mode and source_kind
- AuditSidecarValidationResult omits input_policy_hash for failures before validation references are inspected
- AuditSidecarValidationResult records input_policy_hash for cross-artifact validation
- AuditSidecarValidationResult reason codes use fixed error.field and expected/actual shapes
- AuditSidecarValidationResult input_policy_schema_invalid preserves the section 8 AiAuditInputPolicy expected_value / actual_value strings and does not genericize included_fields failures
- AiAuditInputPolicy version uses the same positive i64 bounds and overflow diagnostics as other MVP policy versions
- post-launch timeout/resource exhaustion uses checker_timeout/checker_resource_exhausted
- run_artifact_hash changes when diagnostics changes, while result_hash does not
- ChallengeReplayResult result_hash is verified as a saved artifact hash in release audit
- ReleasePolicy schema has explicit ai_triage enabled/required fields with no defaults and conditional input_policy_hash
- ReleasePolicy version uses positive i64 bounds and reports non_positive_integer / integer_out_of_range deterministically
- ReleasePolicy schema/domain failures use fixed field / expected_value / actual_value shapes, including ai_triage conditions
- ReleasePolicy validation reports schema failures, local domain failures, and resolver trust_mode mismatches in fixed priority order
- ReleasePolicy known-field duplicate object keys are reported at that field's schema-order position, not after all known fields
- ReleasePolicy mode must match the trust_mode of both runner_policy_hash and challenge_runner_policy_hash
- ReleasePolicy mode/trust mismatch reports deterministic field, expected_value, and actual_value, and prevents bundle generation
- ReleaseAuditBundleManifest reports ReleasePolicy mode/trust mismatch with the same field shape and marks the bundle invalid
- ReleaseAuditBundleManifest includes exactly one ai_audit_input_policy artifact when ai_triage is enabled
- CI diagnostic required AI sidecars and their AuditSidecarValidationResult input_policy_hash match ReleasePolicy.ai_triage.input_policy_hash
- ReleaseAuditBundleManifest is a pass artifact and rejects any release target that still has required AI sidecar targets
- required AI sidecar targets are CI diagnostic targets and are not included in ReleaseAuditBundleManifest
- ReleaseAuditBundleManifest forbids required AI sidecar sources and allows only optional sidecar sources from the release bundle source closed set
- ai_triage.required produces no required AI sidecar artifacts when the CI diagnostic target set is empty
- MachineCheckRequestErrorResult, NormalizeErrorResult, and CommandError do not create required AI sidecar targets in Phase 8 MVP
- CI diagnostic required machine_result sidecar sources resolve through the selected raw MachineCheckResult run_artifact_hash using checker_profile, result_hash, and request_hash
- machine_result sidecar source.normalized_result_hash, when present, must be the release target normalized_result_hash and is forbidden for reproducibility repeated raw results
- ReleaseAuditBundleManifest rejects optional AI sidecars whose source is outside the release audit sidecar source closed set
- ReleaseAuditBundleManifest applies exact selected raw result rules in addition to general sidecar semantic normalized membership
- AI sidecar references do not expand the allowed machine_check_result, machine_check_request, normalized_check_result, challenge_replay_result, or import_lock sets
- nightly required challenge replay artifacts are only those referenced by ChallengeCoverageSummary.entries
- ReleaseAuditBundleManifest requires exactly one pre-merged bundle-local request, machine result, and normalized result store manifest covering only included artifacts
- pre-bundle staging copies explicit input artifacts to deterministic bundle-root artifact paths and writes bundle-local store manifests before release bundle generation
- release stage-bundle-inputs runs store phase before release coverage summary generation and final phase after coverage / auxiliary artifacts exist
- release stage-bundle-inputs uses atomic writes, exact-match adoption, and output_path_conflict for differing target bytes
- release stage-bundle-inputs creates target parent directories and treats directory creation failure as output_write_failure or output_store_write_failure
- release stage-bundle-inputs classifies target path file component and existing-directory conflicts as output_path_conflict
- release stage-bundle-inputs writes temporary files in the final target directory before atomic replace
- release stage-bundle-inputs missing required CLI flags, duplicate singleton flags, unsupported flags, and invalid --phase enum are CLI argument errors
- release stage-bundle-inputs plan hash is exact plan file bytes SHA-256, not parsed / reserialized JSON hash
- release stage-bundle-inputs rejects invalid --plan-hash format with input_reference_invalid and fixed plan_hash fields
- release stage-bundle-inputs rejects invalid --plan path schema and plan.phase / --phase mismatch with fixed fields
- release stage-bundle-inputs reports simultaneous --plan path, --bundle-root path, and --plan-hash reference failures in fixed order
- release stage-bundle-inputs reports ReleaseBundleStagingPlan top-level non-object and schema missing/null/wrong-type/mismatch with fixed fields
- release stage-bundle-inputs reports ReleaseBundleStagingPlan phase enum before --phase mismatch and bundle_root path schema before --bundle-root mismatch
- release stage-bundle-inputs plan separates file_hash from kind-specific parsed hashes, rejects expected_hash, unknown fields, duplicate inputs, and order violations
- release stage-bundle-inputs rejects invalid --bundle-root and plan bundle_root path schema and bundle_root / --bundle-root mismatch with fixed fields
- release stage-bundle-inputs enforces the phase-specific allowed input kind table
- release stage-bundle-inputs rejects pipeline error artifact kind strings as kind enum violations, not phase-forbidden kinds
- release stage-bundle-inputs store phase cardinality violations report fixed expected_value and actual_value fields
- release stage-bundle-inputs treats command-specific actual_value entries as fixed generated values, not placeholder strings
- release stage-bundle-inputs final phase does not validate release bundle closed-set cardinality
- release stage-bundle-inputs resolves source store manifest entry paths from invocation cwd, not from the manifest file parent directory
- release stage-bundle-inputs uses repository root as invocation cwd for every MVP mode
- release stage-bundle-inputs maps plan, source, source-store, merge, conflict, and write failures to the fixed CommandError reason codes
- release stage-bundle-inputs plan-local simultaneous failures use fixed priority, schema object order, input index order, and hashes field order
- release stage-bundle-inputs reports simultaneous phase-kind, direct source input, source store manifest, merge, conflict, and write failures by the fixed input index / per-input priority
- release stage-bundle-inputs input_store_entry_invalid reports deterministic actual_value / expected_hash / actual_hash fields and first failing entry validation step
- release stage-bundle-inputs source store manifest duplicate reasons and source artifact self-hash validation order are store-kind-specific
- release stage-bundle-inputs store merge deduplicates exact duplicate unique-key entries and rejects same-key hash or byte conflicts
- release stage-bundle-inputs source store manifest errors include the staging plan input index in field paths
- release stage-bundle-inputs maps AiAuditInputPolicy artifact-local input_policy fields to inputs[<i>].artifact fields while preserving expected_value / actual_value
- release stage-bundle-inputs maps CheckerIdentityManifest checker_identity_manifest and ImportLockManifest imports.manifest virtual roots to inputs[<i>].artifact fields
- release stage-bundle-inputs store phase stages target-scoped challenge_output_store_manifest before release coverage summary generation
- release stage-bundle-inputs does not validate challenge_output_store_manifest target scope; coverage-summary and bundle validation do
- ReleaseBundleStagingResult store_manifests records generated bundle-local manifests only, uses deterministic order, and records manifest_hash only, not a duplicate file_hash field
- ReleaseBundleStagingResult excludes generated store manifests from staged_artifacts and requires empty store_manifests in final phase
- ReleaseBundleStagingResult is not a release bundle validator input; validators use explicit bundle-local store manifest artifacts
- ReleaseAuditBundleManifest bundle-local manifest validation does not merge or deduplicate entries and rejects exact duplicate, same-key, or same-path conflicts
- optional AI sidecars included in release audit require valid AuditSidecarValidationResult and do not affect pass condition
- nightly AI sidecar diagnostic artifacts are required only for failed / non-success CI diagnostic targets when ReleasePolicy.ai_triage.enabled and ai_triage.required are both true
- CI diagnostic required AI sidecar targets are derived only from failed MachineCheckResult entries and non-success comparison, and remain outside ReleaseAuditBundleManifest
- ReleaseAuditBundleManifest includes exactly one release_policy artifact matching top-level policy_hash
- ReleaseAuditBundleManifest bundle_id is derived from bundle_hash and mismatch is bundle invalid
- ReleaseAuditBundleManifest artifact paths are deterministic bundle-root-relative paths based on kind and file_hash
- ReleaseAuditBundleManifest resolves normal and challenge RunnerPolicy files from ReleasePolicy hashes inside the bundle
- ReleaseAuditBundleManifest includes exactly one checker_identity_manifest artifact for each distinct manifest_hash referenced by included RunnerPolicy files and forbids unreferenced manifests
- ReleaseAuditBundleManifest validates checker_identity_manifest completeness against every included RunnerPolicy checker_allowlist entry
- ReleaseAuditBundleManifest includes exactly one import_lock artifact for each distinct import lock hash referenced by included requests, normalized results, or challenges and forbids unreferenced import locks
- release bundle does not derive import_lock expected set from non-prerequisite-clean machine_check_request, normalized_check_result, or challenge_manifest sets
- release bundle treats machine_check_request set as prerequisite-clean only after request closed-set exactness, request_hash self-consistency, and imports.manifest_hash extraction pass
- release bundle treats challenge_manifest set as prerequisite-clean only after store-referenced manifest exactness, manifest_hash self-consistency, rejection-required mutation kind, and imports.manifest_hash extraction pass
- release bundle scopes release target normalized_check_result prerequisite-clean conditions by downstream kind and applies the normalized_check_result prerequisites before challenge_replay_result prerequisites
- release audit challenge output store deterministic filtering excludes informational ChallengeManifest entries
- ReleaseAuditBundleManifest forbids informational ChallengeManifest and informational ChallengeReplayResult entries
- ReleaseAuditBundleManifest treats import_lock path as bundle-local and validates identity by manifest_hash and file bytes, not by original imports.manifest path
- ReleaseAuditBundleManifest requires exact AuxiliaryResult required slots for release and high-trust modes, classifies missing / duplicate / extra as auxiliary_result class 4, and classifies failed or inconclusive single-slot required entries as auxiliary_result class 5 status mismatch
- AuxiliaryResult selector is required for axiom_policy and reproducibility, forbidden for import_certificate_hash and audit_bundle, and is included in result_hash
- ReleaseAuditBundleManifest validates axiom_policy selector.normalized_result_hash against the release target NormalizedCheckResult and selector.result_hash / selector.axiom_report_hash against the baseline results[*] entry
- ReleaseAuditBundleManifest validates reproducibility AuxiliaryResult selector identity against the release target baseline request_hash / checker_profile and valid, distinct baseline / repeated run_artifact_hash syntax without requiring target MachineCheckResult existence
- release bundle uses kind / policy_hash / artifact_hash as the auxiliary_result required slot key, treats multiple entries in the same slot as duplicate class 4, treats status != passed on a single slot entry as class 5 before selector checks, and treats selector mismatch on a passed single slot entry as class 5
- release bundle treats reproducibility auxiliary_result entries outside the required slot as auxiliary_result class 4 extra before same-kind required-slot missing and before deriving machine_check_result allowed run set
- ReleaseAuditBundleManifest validates AuxiliaryResult envelopes and reference hashes without rerunning axiom_policy, reproducibility, or import_certificate_hash oracles in the MVP bundle
- release bundle treats reproducibility selector run_artifact_hash extraction as auxiliary_result prerequisite-clean, but reports referenced MachineCheckResult missing as machine_check_result class 4
- release bundle reports equal reproducibility baseline_run_artifact_hash and repeated_run_artifact_hash as auxiliary_result class 5 before deriving machine_check_result allowed run set
- release bundle validates reproducibility selector target request_hash / checker_profile / result_hash equality only after referenced MachineCheckResult entries exist
- release target NormalizedCheckResult policy.hash must match ReleasePolicy.runner_policy_hash in release audit
- ChallengeCoverageSummary policy_hash must match ReleasePolicy.challenge_runner_policy_hash in release audit
- ChallengeReplayResult underlying MachineCheckRequest, MachineCheckResult, and challenge replay NormalizedCheckResult policies match ReleasePolicy.challenge_runner_policy_hash
- coverage-required challenge replay requires normalized store and exactly one matching challenge replay NormalizedCheckResult
- ReleaseAuditBundleManifest includes challenge replay NormalizedCheckResult entries for each included ChallengeReplayResult.normalized_result_hash
- release bundle does not derive challenge replay normalized_check_result expected set from a challenge_replay_result set that is non-prerequisite-clean for normalized_check_result
- release bundle reports challenge_coverage_summary prerequisite failure before challenge_replay_result failure when deriving challenge replay normalized_check_result expected set
- challenge_coverage_summary must be prerequisite-clean for normalized_check_result before deriving the challenge replay normalized_check_result expected set
- npa-check challenge coverage-summary performs replay store reference / artifact base validation before coverage-required field validation, validates ChallengeReplayResult.normalized_result_hash shape before comparison_status conditional validation, rejects malformed normalized_result_hash with field replay_store.results[].normalized_result_hash, rejects forbidden comparison_status presence without normalized_result_hash with field replay_store.results[].comparison_status, rejects schema-valid informational replay missing normalized_result_hash for coverage without additionally synthesizing comparison_status missing, and reports multiple replay failures by the generated ChallengeCoverageSummary.entries[] order
- challenge_replay_result set prerequisite-clean for normalized_check_result requires normalized_result_hash to be extractable for every included coverage replay
- release bundle treats an included coverage ChallengeReplayResult missing normalized_result_hash as challenge_replay_result class 5 source-key failure with field challenge_replay_result[<i>].artifact.normalized_result_hash, while extra replay inputs outside coverage remain class 4 extra before normalized_result_hash extraction
- release bundle treats wrong-type, null, or invalid-hash-format ChallengeReplayResult.normalized_result_hash as Step 6 input_schema_invalid with field challenge_replay_result[<i>].artifact.normalized_result_hash before Step 8 prerequisite validation
- ChallengeReplayResult schema validation reports malformed normalized_result_hash before evaluating comparison_status conditional required or forbidden presence
- release bundle Step 8 challenge_replay_result[<i>] fields use the original repeatable input occurrence index, and exact duplicate deduped entries use the smallest original occurrence index
- challenge_replay_result closed-set identity key is parsed result_hash after Step 7 during release bundle generation and artifact entry hashes.result_hash during materialized ReleaseAuditBundleManifest validation
- challenge_replay_result result_hash self-consistency compares artifact entry hashes.result_hash, parsed result_hash, and recomputed result_hash when validating an existing ReleaseAuditBundleManifest
- challenge_replay_result class 5 source-key failures within the same included replay use fixed priority: result_hash self-consistency, normalized_result_hash missing, release target normalized_result_hash collision, while missing or non-unique summary bindings are reported as challenge_coverage_summary schema/domain failures or challenge_replay_result class 4 exact-set failures
- challenge_replay_result set prerequisite-clean for normalized_check_result rejects normalized_result_hash values equal to the release target normalized_result_hash as challenge_replay_result class 5 with expected_value distinct_from_release_target_normalized_result_hash and never deduplicates them with the release target entry
- challenge_replay_result set prerequisite-clean for normalized_check_result permits multiple challenge replay results to share the same non-release normalized_result_hash and derives one normalized_check_result expected entry per distinct normalized_result_hash
- release bundle derives challenge replay normalized_check_result expected set without requiring ChallengeReplayResult.checker_results[*].run_artifact_hash extraction
- challenge_replay_result set prerequisite-clean for machine_check_result requires challenge_coverage_summary prerequisite-clean for challenge_replay_result
- challenge_replay_result set prerequisite-clean for machine_check_result compares replay challenge_id and manifest_hash to the summary entry, and replay policy_hash to top-level ChallengeCoverageSummary.policy_hash
- challenge_replay_result set prerequisite-clean for machine_check_result requires checker_results[*].run_artifact_hash extraction
- challenge_replay_result set prerequisite-clean for machine_check_result does not require ChallengeReplayResult.normalized_result_hash extraction
- ReleaseAuditBundleManifest requires each MachineCheckResult request_hash to resolve to an included MachineCheckRequest
- ReleaseAuditBundleManifest rejects challenge_replay_result entries outside ChallengeCoverageSummary.entries[*].replay_result_hash
- ReleaseAuditBundleManifest rejects informational ChallengeReplayResult entries in the MVP
- ReleaseAuditBundleManifest rejects machine_check_result entries outside the closed allowed run set
- release bundle does not derive machine_check_result allowed run set from non-prerequisite-clean release target normalized_check_result for machine_check_result, challenge_replay_result set for machine_check_result, or reproducibility auxiliary_result sets
- release bundle uses the same prerequisite-clean machine_check_result set definition for machine_check_request and optional machine_result ai_audit_sidecar source checks
- release bundle reports missing MachineCheckResult entries for valid reproducibility selector run_artifact_hash values as machine_check_result class 4 missing failures
- release bundle reports reproducibility selector target equality mismatches after referenced MachineCheckResult entries exist as class 5 failures
- release bundle treats non-baseline release target raw result selection ambiguity as machine_check_result prerequisite class 5 failure, not as a synthesized missing or forbidden entry
- release bundle release target normalized_check_result prerequisite for machine_check_result requires the baseline result entry and all non-baseline result entries to expose result_hash, request_hash, policy_hash, and checker_profile
- release bundle classifies baseline and non-baseline release target results only after every results[*] key field is extractable and requires exactly one RunnerPolicy.required_checker_profiles[0] entry
- release bundle reports malformed or missing release target results[*] key fields as Step 6 input_schema_invalid, while zero or multiple baseline checker_profile entries after Step 6 are normalized_check_result class 5 prerequisite failures
- release bundle prerequisite self-consistency checks compare artifact entry hashes, parsed artifact hashes, and recomputed hashes for request, machine result, and challenge replay artifacts
- release bundle recomputes embedded NormalizedCheckResult.comparison for every allowed included normalized_check_result entry after normalized_check_result closed-set exactness, regardless of optional compare_validation_response presence, and requires the recomputed release target comparison status to be all_agree_checked
- release bundle treats release target normalized comparison recomputation failure or non-all_agree_checked status as normalized_check_result class 5 prerequisite failure before deriving downstream expected sets
- release bundle reports extra / forbidden normalized_check_result entries as class 4 before selecting a target-specific comparison policy for those entries, and treats challenge replay normalized comparison recomputation failure as normalized_check_result class 5 before deriving downstream expected sets
- release bundle validates ChallengeReplayResult.comparison_status against the recomputed challenge replay NormalizedCheckResult comparison status and validates ChallengeCoverageSummary.entries[].comparison_status against the matched ChallengeReplayResult as normalized_check_result set prerequisite-clean conditions before applying coverage pass conditions or deriving downstream expected sets
- release bundle treats missing, null, wrong-type, or invalid-enum ChallengeReplayResult.comparison_status when normalized_result_hash is present as Step 6 input_schema_invalid before Step 8 status binding validation
- release bundle treats present ChallengeReplayResult.comparison_status when normalized_result_hash is omitted as Step 6 input_schema_invalid with expected_value absent_without_normalized_result_hash before Step 8 status binding validation
- release bundle treats missing, null, wrong-type, or invalid-enum ChallengeCoverageSummary.entries[].comparison_status as Step 6 input_schema_invalid before Step 8 status binding validation
- release bundle validates optional normalized_comparison ai_audit_sidecar sources without requiring prerequisite-clean machine_check_result, reproducibility auxiliary_result, or release target artifact.import_lock_hash extraction
- ReleaseAuditBundleManifest selects the release target baseline raw result by reproducibility.selector.baseline_run_artifact_hash
- ReleaseAuditBundleManifest rejects non-baseline duplicate retry results that cannot be selected unambiguously
- ReleaseAuditBundleManifest rejects machine_check_request entries outside the distinct request_hash set of included MachineCheckResult artifacts
- ReleaseAuditBundleManifest validates ChallengeCoverageSummary.summary_hash / summary_id with fixed field shape and priority, and validates unexpected_acceptances
- release bundle generation reports ChallengeCoverageSummary summary_hash self-hash and summary_id mismatches as release_bundle_generation_failed with the same field shape and generation-specific priority
- ChallengeCoverageSummary challenge_store_manifest_hash binds coverage to an explicit ChallengeOutputStoreManifest
- ChallengeCoverageSummary summary_id is derived from summary_hash and mismatch is coverage generation failure or bundle invalid
- ChallengeCoverageSummary target_normalized_result_hash and result_store_manifest_hash bind coverage to the explicit target and machine result store used for recomputing unexpected_acceptances
- ChallengeCoverageSummary rejects duplicate replay_result_hash entries independently from duplicate challenge_id/manifest_hash entries
- ChallengeCoverageSummary duplicate replay_result_hash uses fixed field shape entries[].replay_result_hash and release bundle prefixes it as coverage_summary.artifact.entries[].replay_result_hash
- release coverage summary generation uses staged challenge_output_store_manifest and staged machine result store manifest
- ChallengeOutputStoreManifest used for coverage is target-scoped and rejects global or multi-target stores
- ChallengeOutputStoreManifest split/filter validates every referenced ChallengeManifest, including mutation.target grammar, before filtering and fails instead of skipping unreadable, invalid, hash-mismatched, or mutation-kind-invalid entries
- ChallengeOutputStoreManifest split/filter is a pre-bundle pipeline step and release audit bundle validation never reads original manifest_path
- ChallengeOutputStoreManifest split/filter uses manifest-local ChallengeManifest validation only and does not read base certificates, mutated certificates, import locks, or policy files
- ChallengeOutputStoreManifest entries and referenced ChallengeManifest base certificate fields must match the coverage target
- ReleaseAuditBundleManifest includes exactly one challenge_output_store_manifest and rejects challenge_manifest entries not referenced by it
- ReleaseAuditBundleManifest validates included ChallengeManifest mutation.target grammar before rejecting informational kind from the release set
- ReleaseAuditBundleManifest includes exactly one challenge_coverage_summary matching the included challenge_output_store_manifest and challenge runner policy
- ChallengeCoverageSummary total_challenges is derived from ChallengeOutputStoreManifest entries, not from the subset of challenge manifests in a bundle
- ChallengeCoverageSummary generation rejects coverage stores containing informational non-rejection challenges
- ChallengeCoverageSummary rejects replay results without comparison_status in MVP
- ChallengeCoverageSummary nightly/release pass requires every rejection-required entry comparison_status to be all_agree_failed
- ReleaseAuditBundleManifest rejects incomplete coverage, non-failing rejection-required comparison_status, or nonzero unexpected_acceptances
- ChallengeCoverageSummary rejects replay entries whose manifest / replay / policy / base certificate references do not match
- challenge coverage-summary command generates ChallengeCoverageSummary from filtered challenge store, replay store, result store, and explicit target normalized result only
- challenge coverage-summary requires explicit target NormalizedCheckResult and rejects artifact_hash mismatch
- challenge coverage-summary rejects malformed non-policy hash flags with input_reference_invalid
- challenge coverage-summary maps top-level reference, read, hash, JSON, schema, and output write failures to fixed CommandError reason codes
- challenge coverage-summary maps store entry unreadable / JSON / schema failures to coverage_summary_generation_failed
- challenge coverage-summary maps cross-artifact coverage semantic failures to coverage_summary_generation_failed
- challenge coverage-summary resolves MachineCheckResult artifacts from result store to recompute unexpected_acceptances
- release validate-bundle command returns AuxiliaryResult kind audit_bundle and reruns the complete bundle validator
- release validate-bundle requires a minimum audit envelope before it can emit audit_bundle AuxiliaryResult
- release validate-bundle treats missing --manifest / --manifest-hash / --json, including one-sided manifest pairs, as CLI argument errors
- release validate-bundle minimum audit envelope CommandError field shapes are fixed for top-level non-object, duplicate envelope members, schema, bundle_hash, policy_hash, artifact_hash, and artifacts
- release validate-bundle minimum audit envelope reports simultaneous schema failures in the fixed top-level / duplicate / schema / hash / artifacts priority
- release validate-bundle reports duplicate non-envelope top-level keys and artifacts[] entry keys as audit_bundle_invalid with fixed field shape and deterministic tie-break
- release validate-bundle returns CommandError for top-level manifest unreadable/hash mismatch and audit_bundle_missing for missing referenced bundle files after minimum envelope is known
- release validate-bundle returns audit_bundle_invalid for readable included artifact JSON/schema/hash failures and uses command-equivalent path/hash/artifact field shapes
- release validate-bundle validates referenced artifacts in materialized artifacts[] order and uses the fixed per-entry symlink / readable / hash / JSON / schema / parsed-hash priority
- release validate-bundle reports referenced bundle file missing/unreadable with audit_bundle_missing path fields and reports referenced file bytes hash mismatch on manifest.artifacts[<j>].file_hash
- release validate-bundle reports bundle_hash self-hash mismatch, bundle_id mismatch, symlink escape, and fallback bundle-level audit_bundle_invalid with fixed fields
- release validate-bundle uses the materialized ReleaseAuditBundleManifest deterministic kind-specific artifact entry occurrence index for every repeatable artifact field shape
- release validate-bundle never emits inconclusive audit_bundle AuxiliaryResult in the MVP
- release validate-bundle --out --json returns the saved AuxiliaryResult and uses exact-match adoption / output_path_conflict for existing outputs
- release bundle requires --out to be exactly bundle-root/manifest.json
- release bundle --json returns the saved ReleaseAuditBundleManifest on success
- release bundle missing required flags, duplicate singleton flags, unsupported flags, and missing --json are CLI argument errors
- release bundle rejects malformed --bundle-root / --out paths and malformed input hash flags with input_reference_invalid
- release bundle treats existing manifest.json with identical bytes as exact-match adoption and differing bytes as output_path_conflict
- release bundle command rejects implicit bundle-root scanning and requires explicit path/hash input references
- release bundle command validates each input hash flag according to its fixed parsed-hash or file-bytes-hash meaning
- release bundle --artifact-hash is the parsed target NormalizedCheckResult.artifact_hash and top-level bundle artifact_hash
- release bundle command validates bundle-local path shape, filename file_hash, JSON/schema, store manifest JSON/schema/order/duplicate failure, and parsed/file hash flags in fixed order
- release bundle command maps Step 2 explicit input path failures to invalid_path, kind_mismatch, or invalid_filename by the fixed path-shape rule
- release bundle command reports simultaneous Step 2-7 failures by Step order first, then input flag field table order, then repeatable occurrence order
- release bundle command treats symlink-escaped explicit input artifact paths as Step 2 input_reference_invalid with actual_value invalid_path before file readable checks
- release bundle release_bundle_generation_failed uses fixed Step 8 priority, prerequisite gating, and complete release bundle artifact kind order before falling back to command-level field shape
- release bundle prerequisite gate only blocks dependent downstream closed-set failures and does not preempt Step 8 class 1-3 or unrelated artifact failures
- release bundle evaluates multiple non-clean prerequisites in the requires-list order, recursively applying prerequisite gates
- release bundle optional response / sidecar Step 8 failures anchor artifact kind to compare_validation_response, ai_audit_sidecar, or audit_sidecar_validation_response by fixed rule
- release bundle treats ai_audit_sidecar and audit_sidecar_validation_response as forbidden artifacts, not optional response/status gating, when ReleasePolicy.ai_triage.enabled is false
- release bundle missing expected entries within the same artifact kind are reported by the fixed expected artifact identity key order using component-wise bytewise comparison
- release bundle machine_check_result missing-entry keys distinguish run_artifact_hash references from release target selected raw result tuples
- release bundle prerequisite gate reports prerequisite identity/source-key failures before downstream closed-set failures that depend on them
- release bundle does not derive challenge_manifest expected set from a non-prerequisite-clean ChallengeOutputStoreManifest
- release bundle does not derive challenge_replay_result expected set from a non-prerequisite-clean ChallengeCoverageSummary
- release bundle high-trust import_certificate_hash auxiliary expected set requires prerequisite-clean import_lock set
- release bundle checks challenge_coverage_summary source-key prerequisites before deriving replay_result_hash expected sets, while its own missing/duplicate entry cardinality follows normal Step 8 order
- release bundle auxiliary_result missing-entry keys use kind-specific derivable components and never require reading an absent selector object or release target baseline result
- release bundle command requires both runner policy flag pairs even when runner_policy_hash equals challenge_runner_policy_hash and emits one runner_policy entry for the shared path
- release bundle command accepts challenge_replay_result only through explicit path/hash pairs and, after challenge_coverage_summary for challenge_replay_result is prerequisite-clean, requires the set to match ChallengeCoverageSummary.entries[*].replay_result_hash
- release bundle command reports challenge_coverage_summary for challenge_replay_result prerequisite failure before challenge_replay_result closed-set cardinality when summary is non-prerequisite-clean
- release bundle exact duplicate challenge_replay_result input pairs are deduplicated before closed-set cardinality, while distinct files with the same result_hash are duplicate failures
- release bundle command requires ai audit input policy when ReleasePolicy.ai_triage.enabled is true
- release bundle ai audit input policy conditional required/forbidden failures are release_bundle_generation_failed, not CLI argument errors
- release bundle command maps AiAuditInputPolicy artifact-local input_policy fields to ai_audit_input_policy.artifact fields and uses input_schema_invalid, while validate-bundle/audit_bundle uses audit_bundle_invalid for the same included artifact schema/domain failures
- release bundle command / validator maps ReleasePolicy artifact-local fields to release_policy.artifact fields, overriding the standalone ReleasePolicy file/reference prefix rule
- release bundle command / validator strips CheckerIdentityManifest checker_identity_manifest and ImportLockManifest imports.manifest virtual roots before constructing artifact fields
- release bundle command accepts optional compare_validation_response only through explicit path/hash pairs
- ReleaseAuditBundleManifest forbids MachineCheckRequestErrorResult and NormalizeErrorResult in Phase 8 MVP
- target_artifact_hash is forbidden in ReleaseAuditBundleManifest MVP
- ReleaseAuditBundleManifest is not materialized for failure-only bundles without a NormalizedCheckResult target
- ReleaseAuditBundleManifest can include rejection-required ChallengeManifest entries by file-byte manifest_hash
- ReleaseAuditBundleManifest artifact_hash is a single target NormalizedCheckResult.artifact_hash in MVP
- compare/audit-sidecar validation responses can be recorded in release audit by file_hash only
- ReleaseAuditBundleManifest accepts compare_validation_response only when it is valid, references exactly one included NormalizedCheckResult, has the correct runner policy hash, and matches recomputation canonically
- ReleaseAuditBundleManifest recomputes compare_validation_response with runner_policy_hash for release target and challenge_runner_policy_hash for challenge replay normalized results
- audit_bundle AuxiliaryResult is not included in the same bundle it validates
- audit_bundle AuxiliaryResult oracle runs the complete ReleaseAuditBundleManifest validator, not only file/hash presence checks
- CheckerIdentityManifest has deterministic order, unique keys, and file-byte manifest_hash
- CheckerIdentityManifest schema/domain first failure uses fixed field order, array index order, and duplicate profile/binary_id tie-breakers
- CheckerIdentityManifest checkers order/grammar/duplicate domain failures report concrete checkers[<i>] fields, and equal adjacent profiles are duplicate_profile rather than order_violation
- CompareValidationResult valid does not imply all_agree_checked
- CompareValidationResult and AuditSidecarValidationResult are transient responses without result_hash
- ChallengeGenerationResult, ChallengeRequestMaterializationResult, ReleaseBundleStagingResult, NormalizationWriteResult, CommandError, and ApiError are transient responses without result_hash
- CompareValidationResult rejects unreadable, invalid JSON, schema-invalid, and artifact_hash-invalid normalized results
- CompareValidationResult failed responses omit unavailable top-level hashes and comparison statuses
- CompareValidationResult validates normalized_result_hash before policy and comparison
- CompareValidationResult failure errors use fixed kind and expected/actual hash fields
- NormalizeErrorResult uses error.kind = normalize_failure
- AxiomReport axiom_report_hash excludes axiom_report_hash itself and rejects duplicate / unsorted axioms deterministically
- AxiomReport rejects invalid ModuleName / AxiomName grammar with invalid_name_format
- AxiomReport schema/domain first failure uses fixed schema field order, duplicate key rules, axiom order violation index, and duplicate axiom name index
- axiom-policy disallowed axiom actual_value uses dotted Phase 8 JSON representation of parsed AxiomName
- AxiomReport self-hash mismatch is reported separately from selector axiom_report_hash mismatch
- axiom-policy rejects AxiomReport module and certificate_hash mismatch with separate field-specific shapes
- axiom-policy rejects invalid NormalizedCheckResult artifact.module grammar with input_schema_invalid / invalid_name_format
- AxiomReportStoreManifest has deterministic order, unique axiom_report_hash/path keys, and file-byte manifest_hash
- AxiomReportStoreManifest schema/domain first failure uses fixed schema field order and reports order, duplicate axiom_report_hash, and duplicate path deterministically
- axiom-policy validates NormalizedCheckResult artifact_hash and normalized_result_hash before using selector fields
- axiom-policy CLI returns CommandError for --normalized-result-hash mismatch and only uses selector.normalized_result_hash mismatch for explicit selector validation
- axiom-policy reports selector.normalized_result_hash, selector.result_hash, and selector.axiom_report_hash mismatches with field-specific shapes
- axiom-policy reports axiom report store entry hash mismatch with dedicated fields
- AuxiliaryResult kind-specific oracle inputs are deterministic
- auxiliary commands generate AuxiliaryResult from deterministic oracles and write failures are not converted to oracle inconclusive
- auxiliary axiom-policy and reproducibility commands map malformed non-policy references and store manifests to fixed CommandError reason codes
- auxiliary axiom-policy and reproducibility oracles emit first-failure AuxiliaryResult errors with fixed field / expected / actual shapes
- auxiliary axiom-policy and reproducibility store entry validation order is unreadable, JSON, schema, file_hash, self-hash, manifest-field mismatch
- reproducibility reports machine result store entry result_hash / request_hash / run_artifact_hash / checker_profile manifest-field mismatch with fixed fields
- reproducibility manifest-field mismatch uses expected_hash/actual_hash for hash fields and expected_value/actual_value only for checker_profile
- reproducibility checks each baseline / repeated row in baseline-then-repeated order before moving to the next row
- reproducibility compares derived failure_key from MachineCheckResult.error and never reads a saved MachineCheckResult failure_key field
- reproducibility validates MachineCheckResult result_hash and run_artifact_hash self-hashes before selector or deterministic equality checks
- auxiliary import-certificate-hash maps readable hash-verified invalid import lock manifests to inconclusive, but unreadable or hash-mismatched top-level inputs to CommandError
- auxiliary import-certificate-hash rejects one-sided import lock path/hash pairs with input_reference_invalid
- auxiliary import-certificate-hash uses the built-in deterministic canonical certificate hash oracle and rejects non-high-trust ReleasePolicy
- auxiliary import-certificate-hash reports missing/unreadable imported certificates as inconclusive with distinct actual_value and hash/decode/certificate_hash mismatches as failed
- auxiliary import-certificate-hash does not validate import export_hash or full semantic certificate validity
- auxiliary commands exit 0 when they successfully emit failed or inconclusive AuxiliaryResult
- release validate-bundle exits 0 when it successfully emits passed or failed audit_bundle AuxiliaryResult
- training export records labels only from MachineCheckResult status / error and never from AI sidecar text
- training export omits absent certificate_hash / checker_id / checker_build_hash copied metadata without skipping or failing the record
- training export includes only MachineCheckResult artifacts selected by normalized store entries and rejects ambiguous retries
- training export maps store input reference, unreadable file, hash mismatch, store manifest invalid, cross-store mismatch, and output failures to fixed CommandError reason codes
- training export maps store entry unreadable / JSON / schema failures to training_export_generation_failed
- training export manifest records JSON Lines file_hash and is not a CI or release audit artifact
- training export example_id is derived from source.run_artifact_hash, export_id is derived from JSON Lines file_hash, and --out / --manifest-out are always required in the MVP
- training export has no inline manifest-only or JSON Lines stdout mode in the MVP
- ReleaseAuditBundleManifest entries are sorted and keyed by kind/path
- AuxiliaryResult reason_code must match its auxiliary kind
- AiAuditSidecar forbidden verdict/source/secret fields are rejected by deterministic field-name rules
```

特に重要なのは、AI がどのような sidecar を出しても、
checker result と `NormalizedCheckResult.comparison` を上書きできないことです。

---

# 21. Non-goals

Phase 8 AI Profile でまだ入れないもの：

```text
- LLM-based proof checker
- natural language proof acceptance
- source re-elaboration as independent verification
- tactic replay as independent verification
- AI majority vote over checker disagreement
- AI-selected trusted checker binary
- remote import resolution
- self-modifying checker config
- accepting noncanonical certificates for compatibility
- using AI confidence as CI pass condition
```

将来、AI が checker 実装のバグ候補を発見することはあります。
それでも、修正後の checker binary と deterministic result が trust boundary です。

---

# 22. Completion criteria

Phase 8 AI Profile が完了したと言える条件：

```text
- MachineCheckRequest / MachineCheckResult schema が固定されている
- import lock schema と checker runner dynamic args が固定されている
- checker runner が policy allowlist と runner-owned binary registry だけを使う
- raw checker result が AI 処理前に保存される
- NormalizedCheckResult が deterministic に生成される
- disagreement が常に failure になる
- AiAuditSidecar が verdict を持てない schema になっている
- AI summary が checker result hash または normalized comparison hash に紐づく
- challenge generator が deterministic mutation で outcome-hint reject corpus を作れる
- challenge result は checker result を oracle にしている
- challenge replay result store と coverage target が明示 input / output として固定されている
- AuxiliaryResult / ChallengeCoverageSummary / ReleaseAuditBundleManifest が deterministic command で生成される
- ReleaseBundleStagingResult と pre-bundle staging の2 phase が deterministic に固定されている
- ReleaseBundleStagingPlan の file hash / parsed hash の意味が固定されている
- release bundle generation が pre-bundle staged artifact と explicit path/hash input だけで動く
- release bundle artifact path、bundle_id、summary_id が deterministic に導出される
- training export label が checker result だけから作られる
- CI が AI sidecar なしでも pass/fail を決められる
- release audit bundle に AI sidecar の入力方針と prompt hash が残る
```

---

# 23. 一文でまとめると

Phase 8 AI Profile は、**AI を independent checker の前後に置く監査補助として使い、
checker の代替にも trust boundary の一部にもしないための設計**です。
