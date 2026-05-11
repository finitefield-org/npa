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

Unicode string values are treated as UTF-8 bytes after parsing.
Hashing code must not apply locale-dependent case folding or path normalization.
If a schema needs human text normalization, that schema must say so explicitly.
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
たとえば audit sidecar file 内の duplicate key は `sidecar_json_invalid` ではなく
`sidecar_schema_invalid`、input policy file 内なら `input_policy_schema_invalid`、
result / normalized store manifest 内なら各 `*_manifest_invalid` です。
API request body の duplicate key は machine API の `ApiError` 規則に従います。

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
`module` と `certificate_hash` は metadata ではなく、`axiom_policy` oracle が
selector で選んだ `NormalizedCheckResult` / result entry と照合する binding field です。
`axioms` は used axiom set であり、重複を禁止します。
各 entry は closed-world object で、MVP では `name` だけを required にします。
`axioms` は `name` の UTF-8 bytewise lexicographic order で昇順に並べます。
`AxiomReport` loader は duplicate key、unknown field、wrong type、explicit null、
invalid hash format、order violation、duplicate axiom name を schema / domain failure として拒否します。
duplicate axiom name の `actual_value` は `duplicate_axiom_name` です。
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
自然言語の説明、OS error text、stderr excerpt、human-facing hint は `diagnostics` に入れます。
`diagnostics` は `result_hash` から除外されるため、文言変更で verdict identity が変わりません。

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
`imports` は `module`、次に `export_hash`、次に `certificate.certificate_hash` の
bytewise lexicographic order で昇順に並べます。
MVP v1 ではすべての entry で `certificate.certificate_hash` を required とし、
欠落した entry は import lock manifest schema invalid です。
したがって sort key に absent value は存在しません。
`module` と `certificate.path` はそれぞれ unique です。
`certificate.kind = path` だけを MVP で許可します。
`schema` が `npa.phase8.import_lock_manifest.v1` 以外の string の場合、
および `certificate.kind` が `path` 以外の string の場合は
`actual_value = "invalid_enum"` の schema / domain failure とします。
`certificate.path` は workspace-relative path で、runner / checker は HTTP URL、
directory scan、database lookup、network import resolution に fallback してはいけません。
`certificate.file_hash` は referenced `.npcert` file bytes の SHA-256 です。
`certificate.certificate_hash` は referenced certificate の claimed certificate hash ではなく、
checker がその import certificate を検査して再計算すべき canonical certificate hash です。

通常 trust mode でも import entry の `export_hash` と `certificate.certificate_hash` は required です。
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
`invalid_hash_format`、`invalid_path`、`null_not_allowed`、`order_violation`、
`duplicate_field`、`duplicate_module`、`duplicate_path` のいずれかを入れます。
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
- request_hash field が存在し、sha256:<lower-hex> 形式
- request.request_hash が 3.3 の規則で再計算した hash と一致する
```

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
`invalid_enum`、`invalid_path`、`invalid_hash_format`、`null_not_allowed`、
`duplicate_field` のいずれかを入れます。
top-level `schema` が `npa.phase8.machine_check_request.v1` でない場合も
`request_schema_invalid` です。
この場合は `error.field = "schema"`、
`expected_value = "npa.phase8.machine_check_request.v1"`、
`actual_value = "missing"`、`"null_not_allowed"`、`"wrong_type"`、
または入力 artifact の `schema` 文字列を入れます。
top-level schema mismatch では `actual_value = "wrong_schema"` を使いません。
top-level JSON value が object でない場合は `error.field = "$"`、
`expected_value = "object"`、`actual_value = "wrong_type"` にします。
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
    field = invalid RunnerPolicy field path
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
                   invalid_hash_format | invalid_path | null_not_allowed |
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
`error.field` に invalid policy field の JSON path を入れます。
top-level `schema` が `npa.phase8.runner_policy.v1` でない場合は、
`error.field = "schema"`、
`expected_value = "npa.phase8.runner_policy.v1"`、
`actual_value = "missing"`、`"null_not_allowed"`、`"wrong_type"`、
または入力 policy の `schema` 文字列にします。
top-level schema mismatch では `actual_value = "wrong_schema"` を使いません。
top-level JSON value が object でない場合は `error.field = "$"`、
`expected_value = "object"`、`actual_value = "wrong_type"` にします。
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
`optional_checker_profiles` は required profile を含んではいけません。
`checker_allowlist` と `budgets` の profile 集合は、required / optional profile の和集合と
完全一致しなければなりません。
`optional_checker_profiles` は重複を許さず、配列順は semantic order です。
MVP では generator が bytewise lexicographic order で書き出すことを推奨しますが、
comparison / replay の optional profile order は policy file に保存された配列順を使います。
`checker_allowlist` は `profile` の bytewise lexicographic order で昇順に並べます。
`checker_allowlist.profile` と `checker_allowlist.binary_id` はそれぞれ unique です。
`allowed_args` の配列順は checker command identity の一部なので、sort してはいけません。
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

RunnerPolicy schema / domain validation failure の field shape は全 command で共通です。
`npa-check run` / `/machine/check/certificate` では `runner_policy_invalid`、
normalize / compare / challenge 系 command では `policy_reference_invalid` に入れます。
top-level `schema` mismatch と top-level non-object は上記 `runner_policy_invalid` と
同じ `field`、`expected_value`、`actual_value` を使います。
それ以外の schema failure では invalid field の JSON path、schema requirement 名、
上記の field schema failure `actual_value` を使います。
domain failure では次の table の `field`、`expected_value`、`actual_value` を使います。

```text
required_checker_profiles が trust_mode 表と一致しない:
  field = "required_checker_profiles"
  expected_value = "profiles_for_trust_mode:<trust_mode>"
  actual_value = "profile_set_mismatch"

required_checker_profiles の順序だけが trust_mode 表と一致しない:
  field = "required_checker_profiles"
  expected_value = "profiles_for_trust_mode:<trust_mode>"
  actual_value = "profile_order_mismatch"

optional_checker_profiles が required profile を含む:
  field = "optional_checker_profiles[]"
  expected_value = "exclude_required_checker_profiles"
  actual_value = "required_profile_in_optional"

optional_checker_profiles が重複 profile を含む:
  field = "optional_checker_profiles[]"
  expected_value = "unique_profiles"
  actual_value = "duplicate_profile"

checker_allowlist に required / optional profile の entry がない:
  field = "checker_allowlist"
  expected_value = "entry_for_each_required_and_optional_profile"
  actual_value = "missing_checker_allowlist_entry"

checker_allowlist に required / optional profile 以外の entry がある:
  field = "checker_allowlist"
  expected_value = "only_required_and_optional_profiles"
  actual_value = "unexpected_checker_allowlist_entry"

checker_allowlist が profile 昇順でない:
  field = "checker_allowlist"
  expected_value = "profile_bytewise_ascending"
  actual_value = "order_violation"

checker_allowlist.profile が重複する:
  field = "checker_allowlist[].profile"
  expected_value = "unique_profiles"
  actual_value = "duplicate_profile"

checker_allowlist.binary_id が重複する:
  field = "checker_allowlist[].binary_id"
  expected_value = "unique_binary_ids"
  actual_value = "duplicate_binary_id"

budgets に required / optional profile の entry がない:
  field = "budgets"
  expected_value = "budget_for_each_required_and_optional_profile"
  actual_value = "missing_budget_entry"

budgets に required / optional profile 以外の entry がある:
  field = "budgets"
  expected_value = "only_required_and_optional_profiles"
  actual_value = "unexpected_budget_entry"

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

RunnerPolicy validation は schema failure を domain failure より先に報告します。
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
- stderr は diagnostics 用で、verdict には使わない
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

MVP の checker executable resolution は runner-owned `CheckerBinaryRegistry` で行います。
registry は AI、request、sidecar から指定できず、`binary_id` から runner-controlled executable path を
一意に返します。
registry entry の path は workspace-relative または runner install root relative のどちらかですが、
どちらの場合も shell expansion、`PATH` search、current directory search は使いません。
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
ただし `checker_id`、`checker_build_hash`、`checker_version` は下の identity / metadata
例外規則を優先します。
duplicate object key と unknown field はこの例外対象ではなく、常に raw schema failure です。
raw schema failure は checker exit code に応じて `malformed_success_output`、
`malformed_rejection_output`、または `malformed_internal_error_output` に分類します。
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
    - certificate_hash, unless failure is before canonical hash recomputation
    - error.kind
  identity-checked:
    - checker_id
    - checker_build_hash
  optional:
    - checker_version
    - export_hash
    - axiom_report_hash
    - error.declaration
    - error.core_path
    - error.expected_hash
    - error.actual_hash

decode / schema / noncanonical failure:
  required:
    - schema
    - status = failed
    - module, if decodable
    - error.kind
  identity-checked:
    - checker_id
    - checker_build_hash
  optional:
    - checker_version
    - certificate_hash
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
wrong type / null の場合は `diagnostics` にだけ記録して raw verdict 採用の可否に使いません。

`MachineCheckResult.module` はすべての result で required です。
runner が常に `MachineCheckRequest.module` から埋め、checker raw output の `module` を
正本 source として使ってはいけません。
`CheckerRawResult.module` が存在し、`MachineCheckRequest.module` と一致しない場合、
runner は raw output を正本 verdict として写さず、
`status = failed`、`error.kind = checker_internal_error`、
`error.reason_code = checker_module_mismatch` の `MachineCheckResult` を保存します。
このとき `error.field = "module"`、
`error.expected_value = MachineCheckRequest.module`、
`error.actual_value = CheckerRawResult.module` にします。
checker identity が allowlist と一致する場合は `checker.id`、`checker.build_hash`、
`checker.binary_id`、`checker.binary_hash`、`checker.profile` を記録します。
raw `checker_version` が valid string の場合だけ `checker.version` も記録します。
checker identity が allowlist と一致しない場合は identity mismatch の `policy_failure` を優先します。
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

exit 2 + invalid JSON or raw schema failure:
  status = failed, error.kind = checker_internal_error,
  error.reason_code = malformed_internal_error_output

exit 2 + error.kind != checker_internal_error:
  status = failed, error.kind = checker_internal_error,
  error.reason_code = malformed_internal_error_output

exit >= 3:
  status = failed, error.kind = checker_internal_error,
  error.reason_code = process_exit_failure.
  If stdout contains valid CheckerRawResult JSON, runner may copy it into diagnostics after redaction,
  but must not copy its verdict, certificate hashes, checker id, or build hash into MachineCheckResult.

raw module mismatch:
  status = failed, error.kind = checker_internal_error,
  error.reason_code = checker_module_mismatch,
  error.field = "module",
  error.expected_value = MachineCheckRequest.module,
  error.actual_value = CheckerRawResult.module

stderr:
  copied only into diagnostics after redaction; never used as verdict
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
  MachineCheckResult.status = failed, error.kind = checker_internal_error

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
  },
  "diagnostics": []
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

optional checker identity copy fields:
  - checker_id
  - checker_build_hash

required when status = checked:
  - certificate_hash
  - export_hash
  - axiom_report_hash

required when status = failed:
  - error
  - failure_key

unknown field:
  forbidden
```

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
`policy_hash` は、入力 `RunnerPolicyReference.hash` が valid hash として読めた場合だけ required で、
その値を写します。
policy file が読めない場合や policy object を parse できない場合でも、
reference hash 自体が valid hash なら omit しません。
`RunnerPolicyReference.hash` 自体が missing、wrong type、explicit null、
または invalid hash format の `policy_reference_invalid` では `policy_hash` を omit します。
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
- selector_module_mismatch
- selector_ambiguous
```

normalizer は request store を解決する前に、入力 `MachineCheckResult` を validation します。
CLI で file path から読む場合は file bytes を読めること、JSON として parse できること、
top-level schema が `npa.phase8.machine_check_result.v1` であることを検査します。
API で object として受け取る場合も、top-level schema と field schema を同じ順序で検査します。
`MachineCheckRequestErrorResult`、`NormalizeErrorResult`、`CompareValidationResult` など
`MachineCheckResult` 以外の schema が混入した場合は、checker verdict として扱わず
`NormalizeErrorResult.error.reason_code = machine_result_wrong_schema` を返します。
top-level JSON value が object でない場合も `machine_result_wrong_schema` です。
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
`invalid_enum`、`invalid_hash_format`、`null_not_allowed`、`order_violation`、
`duplicate_field` のいずれかを入れます。
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
MachineCheckResult validation order は次で固定します。

```text
1. file readable / JSON parse, if input is a file
2. top-level schema
3. MachineCheckResult schema
4. result_hash recomputation
5. run_artifact_hash recomputation
6. request_hash resolution and request hash match
```

先の step で失敗した場合、後続 step の error は報告しません。

step 6 では request hash 解決順を固定します。
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
`invalid_enum`、`invalid_path`、`invalid_hash_format`、`null_not_allowed`、
`duplicate_field` のいずれかを入れます。
request store entry の top-level `schema` が
`npa.phase8.machine_check_request.v1` でない場合も `request_schema_invalid` です。
この場合は `field = "request_store.requests[].schema"`、
`expected_value = "npa.phase8.machine_check_request.v1"`、
`actual_value = "missing"`、`"null_not_allowed"`、`"wrong_type"`、
または入力 request artifact の `schema` 文字列を入れます。
この `request_schema_invalid` でも `actual_value = "wrong_schema"` は使いません。
`request_hash_missing` では `field = "request_hash"`、
`expected_value = "sha256:<lower-hex>"`、`actual_value = "missing"` にします。
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
schema / domain validation failure では `field` に invalid policy field の JSON path を入れます。
schema / domain validation failure の `expected_value` / `actual_value` は
4.1 の RunnerPolicy schema / domain validation field shape に従います。
`policy_file_unreadable` では `field = "policy.path"`、`actual_value = "unreadable"` を入れます。
この reason では `expected_hash` と `actual_hash` は omit します。
`policy_hash_mismatch` では `field = "policy.hash"`、
`expected_hash` に `RunnerPolicyReference.hash`、
`actual_hash` に読み込んだ `RunnerPolicy` の canonical hash を入れます。
`request_store_manifest_invalid` では、request store manifest file を読めない場合
`field = "request_store.path"`、`actual_value = "unreadable"` にします。
JSON として壊れている場合は同じ field で `actual_value = "invalid_json"` にします。
schema / order / duplicate 違反では `field` に invalid request store manifest field の JSON path、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_hash_format`、
`null_not_allowed`、`order_violation`、`duplicate_field`、`duplicate_request_hash`、
`duplicate_path` のいずれかを入れます。
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

`request_store_manifest_hash_mismatch` と
`selector_module_mismatch` では `field`、`expected_hash` または `expected_value`、
`actual_hash` または `actual_value` を入れます。
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
`kind = normalized_comparison` では `result_hash`、`request_hash`、
`run_artifact_hash`、`result_id` を omit します。

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
`redaction` は `default`、`strict`、`release` のいずれかです。
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
CLI の flag front-end は generator core を呼ぶ前に `--from` を読んで
`base_certificate.file_hash` と `base_certificate.claimed_certificate_hash` を埋めた
`ChallengeGenerationRequest` を構築し、その後 `request_hash` を計算します。
この request construction phase は output path を作成・更新してはいけません。
construction phase で base certificate を読めない、または claimed hash を decode できない場合は、
`ChallengeGenerationRequest` を作らず、対応する generation `CommandError` を返します。
generator core は CLI front-end が埋めた base certificate hash を信用せず、
request hash validation 後に base certificate を再読込して再検証します。
`request_hash` が存在しない場合は `generation_request_hash_missing`、
3.3 の規則で再計算した hash と一致しない場合は
`generation_request_hash_mismatch` です。
`request_hash` の invalid hash format は `generation_request_schema_invalid` です。
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
`mutation.seed` は `sha256:<lower-hex>` 形式で、generator が mutation point を選ぶ唯一の乱択入力です。
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
または entry の `manifest_hash` と参照先 `ChallengeManifest` file bytes が一致しない場合は
generation failure です。
`ChallengeOutputStoreManifest` file 自体の expected hash は generation request では受け取りません。
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
`mutated_certificate.claimed_certificate_hash` は mutated certificate から claimed hash を
decode できた場合だけ required です。
decode 不能 mutation では omit します。
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
  alter_universe_constraint
  alter_de_bruijn_index
  replace_nat_zero_with_noncanonical_placeholder
  target = declaration full name, e.g. Nat.add_zero

import target:
  replace_import_export_hash
  remove_dependency_entry
  target = imported module full name

axiom target:
  add_forbidden_axiom
  target = axiom name

whole certificate target:
  flip_canonical_encoding_byte
  reorder_declarations
  insert_unsupported_schema_version
  truncate_certificate_section
  target = "$whole_certificate"
```

MVP の mutation execution rule：

```text
common:
  - generator は base certificate file bytes を読み、request の file_hash と照合する。
  - generator は claimed certificate hash を decode し、request の claimed_certificate_hash と照合する。
  - structured mutation は base certificate を `NPA-CERT-0.1` `ModuleCertBytes` として decode してから行う。
  - structured mutation で base certificate を decode できない場合は base_certificate_decode_failed。
  - target declaration / import / axiom が存在しない場合は mutation_target_invalid。
  - candidate が複数ある mutation は、candidate を canonical byte offset 昇順で並べ、
    seed の hash bytes 先頭8 byteを unsigned big-endian integer として解釈し、
    index = value mod candidate_count で1件を選ぶ。
  - candidate_count = 0 の場合は mutation_target_invalid。
  - `add_forbidden_axiom` 以外の mutation は、特に明記しない限り既存の stored hash field を
    recompute せず、改変前の hash bytes を残す。

change_declaration_body_without_hash:
  target declaration の body / proof / value term の canonical encoded bytes から
  mutable payload byte candidates を canonical byte offset 昇順で集め、seed で1 byte を選ぶ。
  mutable payload byte は term tag、length prefix、section framing、stored hash field に属さない
  term payload byte だけです。
  selected byte を xor 0x01 で変更する。
  mutable payload byte が存在しない場合は mutation_target_invalid。
  resulting bytes が decode 不能になる場合も generation failure ではなく checker result を oracle にする。
  stored declaration hash、export_hash、axiom_report_hash、certificate_hash は更新しない。

change_declaration_hash_without_body:
  target declaration の stored `decl_certificate_hash` または `decl_interface_hash` の
  first byte を xor 0x01 で変更し、body / type / proof bytes は変更しない。
  module-level hash field は更新しない。

drop_axiom_report_entry:
  target declaration の `DeclAxiomReport` entry を削除する。
  `AxiomReport` 内の sort order は残った entry の既存順を保ち、axiom_report_hash と
  certificate_hash は更新しない。

alter_universe_constraint:
  target declaration の universe constraint candidates を canonical byte offset 昇順で集め、
  seed で1つ選ぶ。
  `<=` は `=` に、`=` は `<=` に変更する。
  constraint がない場合は mutation_target_invalid。
  stored declaration / module hash は更新しない。

alter_de_bruijn_index:
  target declaration の term graph 内の `BVar` candidates を canonical byte offset 昇順で集め、
  seed で1つ選び、index を index + 1 に変更する。
  stored declaration / module hash は更新しない。

replace_nat_zero_with_noncanonical_placeholder:
  target declaration の term graph 内の canonical `Const Nat.zero` reference candidates を
  canonical byte offset 昇順で集め、seed で1つ選ぶ。
  selected term node tag を reserved invalid tag 0xff に置き換える。
  core calculus に placeholder term を追加するわけではない。
  checker はこの bytes を `noncanonical_encoding` または `certificate_decode_error` として拒否する。

replace_import_export_hash:
  target import entry の stored `export_hash` first byte を xor 0x01 で変更する。
  referenced import certificate file は変更せず、module-level certificate_hash も更新しない。

remove_dependency_entry:
  target imported module または target declaration に対応する declaration dependency entry を
  seed で1件削除する。
  dependency entry が存在しない場合は mutation_target_invalid。
  stored declaration / module hash は更新しない。

add_forbidden_axiom:
  target axiom name の `AxiomDecl` を追加し、その type は `Sort zero` に固定する。
  declaration order、export block、axiom report、decl hash、export_hash、axiom_report_hash、
  certificate_hash は canonical rules で再計算する。
  この mutation は policy violation を検査するため、hash mismatch を意図的に先に起こさない。

flip_canonical_encoding_byte:
  trusted payload bytes から `ModuleHashes.certificate_hash` field bytes を除いた candidate byte offset を
  canonical section order で集め、seed で1 byte を選び xor 0x01 する。
  resulting bytes がたまたま valid certificate になる場合も outcome_hint ではなく checker result を oracle にする。

reorder_declarations:
  adjacent declaration pair candidates を declaration order 昇順で集め、seed で1組を選んで swap する。
  declaration bytes と stored hashes は更新しない。

insert_unsupported_schema_version:
  Header.format を same-length ASCII string `NPA-CERT-9.9` に置き換える。
  same-length replacement が不可能な encoding version では generation failure ではなく、
  canonical string length field を含む Header encoding を更新し、module hashes は更新しない。

truncate_certificate_section:
  Header 以外の top-level section candidates を
  Imports, NameTable, LevelTable, TermTable, Declarations, ExportBlock, AxiomReport, ModuleHashes
  の順で集め、seed で1つ選ぶ。
  selected section の encoded bytes の後半を deterministic half-open range
  `[section_start + section_len / 2, section_end)` で削除する。
```

MVP の `npa-check challenge generate` と `/machine/check/challenge` は、
上記の target rule に合わない request を `mutation_target_invalid` の generation validation failure として拒否し、
`ChallengeGenerationResult` と `ChallengeManifest` を返してはいけません。
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
`field` に invalid policy field の JSON path を入れ、`expected_value` / `actual_value` は
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
`invalid_hash_format`、`invalid_path`、`null_not_allowed`、`order_violation`、
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
`mutation_target_invalid` では `field = "mutation.target"`、
`expected_value` に mutation kind の target rule 名、`actual_value` に request の target を入れます。
`challenge_output_store_file_unreadable` では `field = "output.store_manifest_path"`、
`actual_value = "unreadable"` にします。
`challenge_output_store_json_invalid` では `field = "output.store_manifest_path"`、
`actual_value = "invalid_json"` にします。
`challenge_output_store_manifest_invalid` では
`field` に invalid store manifest field の JSON path、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、
`invalid_hash_format`、`null_not_allowed`、`order_violation`、
`duplicate_challenge_id`、`duplicate_manifest_path`、`duplicate_field` のいずれかを入れます。
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
  certificate header / trailer に格納された certificate_hash。
  base_certificate では required。
  mutated_certificate では、変異によって certificate が decode 不能になった場合だけ omit する。

recomputed_certificate_hash:
  checker が canonical bytes から再計算した certificate_hash。
  manifest には書かず、MachineCheckResult 側にだけ記録する。
```

challenge replay 用の `MachineCheckRequest.certificate.expected_certificate_hash` は次の規則で作ります。

```text
mutated certificate から claimed_certificate_hash を decode できる場合:
  expected_certificate_hash = mutated_certificate.claimed_certificate_hash

mutated certificate が decode 不能の場合:
  expected_certificate_hash = base_certificate.claimed_certificate_hash
```

decode 不能 mutation の `expected_certificate_hash` は request identity を安定させるための
deterministic placeholder です。
この placeholder は replay 用 `NormalizedCheckResult.artifact.expected_certificate_hash` にも入ります。
つまり decode 不能 challenge の artifact identity には、mutated certificate の recomputed hash ではなく
base certificate の claimed hash が入ります。
実際の mutated file identity は `MachineCheckRequest.certificate.file_hash`、
challenge manifest の `mutated_certificate.file_hash`、
および challenge replay result の `mutated_file_hash` で追跡します。
checker が `certificate_decode_error` / `noncanonical_encoding` / `unsupported_schema_version` を返す場合、
runner は通常どおり certificate hash 照合を skip します。
もし decode 不能のはずの challenge が decode されて checker が canonical hash を再計算できた場合は、
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

`mutated_claimed_certificate_hash` は、mutated certificate から claimed certificate hash を
decode できた場合だけ required です。
decode 不能 mutation では omit します。
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
`comparison_status` は `normalized_result_hash` が存在する場合だけ required で、
`NormalizedCheckResult.comparison.status` を写します。
`normalized_result_hash` が omit された replay result では `comparison_status` も omit します。
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
- replay_store_entry_result_hash_mismatch
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
`invalid_hash_format`、`invalid_path`、`null_not_allowed`、`order_violation`、
`duplicate_field` のいずれかを入れます。
`policy_reference_invalid` では challenge 系 command 共通の policy reference field shape に従います。
`policy_file_unreadable` では `field = "policy.path"`、`actual_value = "unreadable"` にします。
`policy_hash_mismatch` では `field = "policy.hash"`、
`expected_hash` に caller 指定 hash、`actual_hash` に読み込んだ `RunnerPolicy` の canonical hash を入れます。
`ChallengeManifest.policy_hash` が `RunnerPolicyReference.hash` と一致しない場合は
同じ `policy_hash_mismatch` を使い、`field = "challenge_manifest.policy_hash"`、
`expected_hash` に `RunnerPolicyReference.hash`、
`actual_hash` に `ChallengeManifest.policy_hash` を入れます。
manifest hash mismatch では `field` に該当 reference の `*.manifest_hash` field path を入れ、
`expected_hash` に caller 指定 hash、`actual_hash` に manifest file bytes hash を入れます。
`request_store_manifest_invalid`、`result_store_manifest_invalid`、
`normalized_store_manifest_invalid`、`replay_store_manifest_invalid` では、store manifest file を読めない場合は
`field` に該当 reference の `*.path` field path、`actual_value = "unreadable"` を入れます。
JSON として壊れている場合は同じ field で `actual_value = "invalid_json"` にします。
schema / order / duplicate 違反では `field` に invalid store manifest field の JSON path、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_hash_format`、
`null_not_allowed`、`order_violation`、`duplicate_field`、
または manifest 種別ごとの unique key duplicate reason を入れます。
`request_store_manifest_invalid` で許可する unique key duplicate reason は
`duplicate_request_hash` と `duplicate_path` だけです。
`result_store_manifest_invalid` で許可する unique key duplicate reason は
`duplicate_run_artifact_hash` と `duplicate_path` だけです。
`normalized_store_manifest_invalid` で許可する unique key duplicate reason は
`duplicate_normalized_result_hash` と `duplicate_path` だけです。
`replay_store_manifest_invalid` で許可する unique key duplicate reason は
`duplicate_result_hash`、`duplicate_path`、`duplicate_challenge_manifest` だけです。
他 manifest 種別の duplicate reason を使ってはいけません。
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
`invalid_path`、`invalid_hash_format`、`null_not_allowed`、`order_violation`、
`duplicate_field` のいずれかを入れます。
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
self-hash が valid な場合だけ、manifest entry の hash と parsed artifact field を比較します。
manifest entry と parsed artifact field の mismatch では、
`expected_hash` に manifest entry の hash、`actual_hash` に parsed artifact field の hash を入れます。
`request_store_entry_request_hash_mismatch` では `field = "request_store.requests[].request_hash"`、
`expected_hash` に manifest entry の `request_hash`、
`actual_hash` に parsed `MachineCheckRequest.request_hash` を入れます。
`result_store_entry_artifact_hash_mismatch` では `field` に
`result_store.results[].result_hash`、`request_hash`、または `run_artifact_hash` を入れ、
`expected_hash` に manifest entry の hash、`actual_hash` に parsed `MachineCheckResult` の同じ field を入れます。
`result_store_entry_checker_profile_mismatch` では `field = "result_store.results[].checker_profile"`、
`expected_value` に manifest entry の `checker_profile`、
`actual_value` に parsed `MachineCheckResult.checker.profile` を入れます。
`normalized_store_entry_artifact_hash_mismatch` では `field` に
`normalized_store.results[].normalized_result_hash` または `artifact_hash` を入れ、
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
`replay_store_entry_result_hash_mismatch` では `field = "replay_store.results[].result_hash"`、
`expected_hash` に manifest entry の `result_hash`、
`actual_hash` に parsed `ChallengeReplayResult.result_hash` を入れます。
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
複数の失敗条件が同時にある場合は、この一覧の順序で最初に該当した
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
- alter_universe_constraint
- alter_de_bruijn_index
- replace_nat_zero_with_noncanonical_placeholder
- insert_unsupported_schema_version
- truncate_certificate_section
```

CLI の `npa-check challenge generate --kind` は上記の `mutation.kind` enum と同じ文字列を受け取ります。
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
`id` は non-empty string、`version` は positive integer、hash field は
`sha256:<lower-hex>` です。
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
  actual_value = "wrong_type"

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

version non-positive:
  field = "version"
  expected_value = "positive_integer"
  actual_value = "non_positive"

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
  "status": "passed",
  "diagnostics": []
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
    deterministic equality of status, error failure_key, certificate_hash,
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

```text
selected normalized result entry missing / unusable:
  status = inconclusive
  reason_code = axiom_policy_inconclusive
  field = "selector.checker_profile"
  expected_value = "checked_normalized_result_entry"
  actual_value = missing | not_checked | missing_axiom_report_hash

selector / normalized result target mismatch:
  status = inconclusive
  reason_code = axiom_policy_inconclusive
  field = "selector.normalized_result_hash" or "selector.result_hash" or "selector.axiom_report_hash"
  expected_hash = validated NormalizedCheckResult.normalized_result_hash or selected result entry hash
  actual_hash = selector field value

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
                 invalid_enum | invalid_hash_format | null_not_allowed |
                 order_violation | duplicate_field | duplicate_axiom_name

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

axiom report target mismatch:
  status = inconclusive
  reason_code = axiom_policy_inconclusive
  field = "axiom_report.module" or "axiom_report.certificate_hash"
  expected_value / expected_hash = selected NormalizedCheckResult.artifact.module or selected result certificate_hash
  actual_value / actual_hash = parsed AxiomReport.module or parsed AxiomReport.certificate_hash

disallowed axiom:
  status = failed
  reason_code = axiom_policy_failed
  field = "axiom_report.axioms[<i>].name"
  expected_value = "allowed_axiom"
  actual_value = canonical axiom name
```

`axiom_report.axioms[<i>]` の `<i>` は axiom report の deterministic order で最初に
policy に違反した axiom entry の zero-based index です。
`axiom_report_store.reports[<i>]` の `<i>` は axiom report store manifest 内の
該当 entry の zero-based index です。
`axiom_policy` の axiom report store entry validation は、missing / unreadable、
invalid JSON、schema / domain failure、file_hash mismatch、artifact self-hash mismatch、
manifest-field mismatch の順で判定します。

`reproducibility` oracle は次の順序で検査し、最初の失敗だけを
`AuxiliaryResult.error` に記録します。

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
                 invalid_enum | invalid_hash_format | null_not_allowed |
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
          repeated.status, repeated.error.failure_key,
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
`error.failure_key` は 7 の normalized failure key を canonical hash した値で比較し、
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
                 invalid_hash_format | invalid_path | null_not_allowed |
                 order_violation | duplicate_field | duplicate_module |
                 duplicate_path

imported certificate file missing / unreadable:
  status = inconclusive
  reason_code = import_certificate_hash_inconclusive
  field = "import_lock.imports[<i>].certificate.path"
  expected_value = "readable_file"
  actual_value = "unreadable"

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
`diagnostics` は optional で、自然言語、stderr excerpt、human-facing hint を入れます。
`result_hash` は `result_id`、`result_hash`、`diagnostics` field を除いた canonical hash です。
`error` に自然言語を入れてはいけません。
人間向け説明は diagnostics または AI sidecar に分離します。

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
exact duplicate entry と同じ `(kind, path)` を持つ entry は schema invalid です。
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
artifact kind 定義で required hash が `none` の kind では `hashes` は required empty object です。
`kind` は Phase 8 MVP の release bundle artifact kind です。

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
    - machine_check_request_error_result
    - normalize_error_result
```

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
完全一致する exact duplicate だけを deduplicate してよいです。
同じ unique key で parsed artifact hash または source file bytes が1つでも異なる場合は
merge failure であり、その manifest を release bundle command に渡してはいけません。
`npa-check release bundle` は各 kind につき事前 staging / merge 済み manifest を1つだけ受け取り、
複数 store manifest 入力、directory scan、glob、bundle-root からの暗黙 merge は行いません。
validator が bundle-local manifest 内で同じ conflict を検出した場合は bundle invalid です。
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

source store manifest JSON invalid:
  reason_code = input_store_manifest_invalid
  field = "inputs[<i>].path"
  actual_value = "invalid_json"

source store manifest schema / order / duplicate violation:
  reason_code = input_store_manifest_invalid
  field = "inputs[<i>].store.<json_path>"
  expected_value = store manifest schema requirement
  actual_value = missing | wrong_type | unknown_field | invalid_hash_format |
                 null_not_allowed | order_violation | duplicate_field |
                 duplicate_request_hash | duplicate_run_artifact_hash |
                 duplicate_normalized_result_hash | duplicate_path

source store manifest entry unreadable / invalid JSON / schema violation / file_hash mismatch / parsed hash mismatch:
  reason_code = input_store_entry_invalid
  field = "inputs[<i>].store.<entry_json_path>"

bundle-local merge conflict:
  reason_code = release_bundle_generation_failed
  field = "inputs[]"

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

`inputs[<i>]` は `ReleaseBundleStagingPlan.inputs[]` の 0-based index です。
`<json_path>` と `<entry_json_path>` は対象 JSON object の root からの path で、
array element は 0-based index で表します。
source store manifest file 自体の read failure は `input_file_unreadable` で
`field = "inputs[<i>].path"`、`file_hash` mismatch は `input_hash_mismatch` で
`field = "inputs[<i>].file_hash"`、JSON parse failure は `input_store_manifest_invalid` で
`field = "inputs[<i>].path"` とします。
direct source input の artifact schema violation では、`<json_path>` は source artifact JSON の root からの
path です。

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
                 invalid_path | invalid_hash_format | null_not_allowed |
                 order_violation | duplicate_field

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

entry validation の内部優先順位は unreadable、invalid JSON、schema violation、file_hash mismatch、
artifact self-hash mismatch、manifest-field mismatch です。
複数の失敗条件が同時にある場合は、CLI argument validation、
plan / `--bundle-root` / `--plan-hash` reference validation、plan file、plan hash mismatch、
plan JSON / schema、bundle_root mismatch、phase-kind validation、
direct source input validation、source store manifest validation、source store entry validation、
merge conflict、target path conflict、write failure の順で最初に該当した `reason_code` を返します。
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
どちらの array でも exact duplicate entry、同じ `(kind, path)`、同じ `path` は forbidden です。
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
ただし caller が同じ source bytes に対して異なる expected parsed hash を指定した場合は
`input_hash_mismatch` または `release_bundle_generation_failed` です。
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
  - selector.baseline_run_artifact_hash resolves to an included MachineCheckResult whose
    result_hash matches that release target result entry
  - selector.repeated_run_artifact_hash resolves to a distinct included MachineCheckResult
    with the same request_hash and checker_profile
  - baseline and repeated MachineCheckResult parsed result_hash values are equal
  - status = passed
```

`ReleasePolicy.mode = high-trust` では上の release requirements に加えて、
この節で定義した import lock hash 集合の distinct hash ごとに次が required です。

```text
- exactly one kind = import_certificate_hash entry
  - policy_hash = ReleaseAuditBundleManifest.policy_hash
  - artifact_hash = matching import_lock entry hashes.manifest_hash
  - status = passed
```

import lock hash 集合が空の場合、`import_certificate_hash` entry は0件でなければなりません。
`ReleasePolicy.mode = release` では `import_certificate_hash` entry は forbidden です。
MVP の `ReleaseAuditBundleManifest` は `ReleasePolicy.mode = nightly` では materialize しません。
nightly policy を含む `ReleaseAuditBundleManifest` は bundle invalid です。
上記 closed set 以外の `auxiliary_result` kind、重複 entry、missing entry、
または `status != passed` の required entry は bundle invalid です。
`selector` の required / forbidden rule、unknown field、hash format、profile value、
および selector が指す included artifact との不一致も bundle invalid です。
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
bundle validator は included normalized result file と target-specific な included `RunnerPolicy` file を使って
compare validation を再実行し、保存済み `CompareValidationResult` object と
再実行で得た object が canonical serialization 上で一致することを検査しなければなりません。
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

release target `NormalizedCheckResult.results[*]` は `run_artifact_hash` を持たないため、
bundle validator は release target raw result を次で選びます。
runner `RunnerPolicy.required_checker_profiles[0]` の result entry では、
required reproducibility selector の `baseline_run_artifact_hash` が選択 raw result です。
この artifact は release target result entry の `result_hash`、`request_hash`、
`checker_profile`、`policy_hash` と完全一致しなければなりません。
その他の release target result entry では、同じ bundle 内に
`result_hash`、`request_hash`、`checker.profile`、`policy.hash` が一致する
`machine_check_result` entry がちょうど1件存在し、それを選択 raw result とします。
非 baseline profile で同じ tuple に一致する retry result が2件以上ある場合は bundle invalid です。
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
hash format、path format、`mutation.kind` の分類、base / mutated certificate metadata の
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
`(challenge_id, manifest_hash)` は unique です。
`replay_result_hash` は referenced `ChallengeReplayResult.result_hash` です。
MVP の coverage summary に含める `ChallengeReplayResult` は
`normalized_result_hash` と `comparison_status` を持たなければなりません。
`comparison_status` を持たない replay result は covered challenge として数えず、
coverage summary generation failure として扱います。
そのため nightly / release pipeline は coverage summary generation 前に、
各 challenge replay result の `normalized_result_hash` が解決済みであることを要求します。
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
`checkers` は `profile` の bytewise lexicographic order で昇順に並べます。
`profile` と `binary_id` はそれぞれ unique です。
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
`expected_value = "object"`、`actual_value = "wrong_type"` にします。
それ以外の schema failure では `expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_hash_format`、
`null_not_allowed`、`order_violation`、`duplicate_field` のいずれかを入れます。
domain failure では次の field shape を使います。

```text
checkers が profile 昇順でない:
  field = "checker_identity_manifest.checkers"
  expected_value = "profile_bytewise_ascending"
  actual_value = "order_violation"

checkers[].profile が重複する:
  field = "checker_identity_manifest.checkers[].profile"
  expected_value = "unique_profiles"
  actual_value = "duplicate_profile"

checkers[].binary_id が重複する:
  field = "checker_identity_manifest.checkers[].binary_id"
  expected_value = "unique_binary_ids"
  actual_value = "duplicate_binary_id"
```

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
npa-check release bundle --release-policy build/release-audit/Std.Nat/artifacts/release_policy/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.json --release-policy-hash sha256:... --runner-policy build/release-audit/Std.Nat/artifacts/runner_policy/bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb.json --runner-policy-hash sha256:... --challenge-runner-policy build/release-audit/Std.Nat/artifacts/runner_policy/cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc.json --challenge-runner-policy-hash sha256:... --artifact-hash sha256:... --target-normalized-result build/release-audit/Std.Nat/artifacts/normalized_check_result/dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd.json --target-normalized-result-hash sha256:... --request-store build/release-audit/Std.Nat/artifacts/request_store_manifest/eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee.json --request-store-hash sha256:... --result-store build/release-audit/Std.Nat/artifacts/machine_result_store_manifest/ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff.json --result-store-hash sha256:... --normalized-store build/release-audit/Std.Nat/artifacts/normalized_result_store_manifest/1111111111111111111111111111111111111111111111111111111111111111.json --normalized-store-hash sha256:... --challenge-store build/release-audit/Std.Nat/artifacts/challenge_output_store_manifest/2222222222222222222222222222222222222222222222222222222222222222.json --challenge-store-hash sha256:... --coverage-summary build/release-audit/Std.Nat/artifacts/challenge_coverage_summary/3333333333333333333333333333333333333333333333333333333333333333.json --coverage-summary-hash sha256:... --auxiliary-result build/release-audit/Std.Nat/artifacts/auxiliary_result/4444444444444444444444444444444444444444444444444444444444444444.json --auxiliary-result-hash sha256:... --auxiliary-result build/release-audit/Std.Nat/artifacts/auxiliary_result/5555555555555555555555555555555555555555555555555555555555555555.json --auxiliary-result-hash sha256:... --ai-audit-input-policy build/release-audit/Std.Nat/artifacts/ai_audit_input_policy/6666666666666666666666666666666666666666666666666666666666666666.json --ai-audit-input-policy-hash sha256:... --bundle-root build/release-audit/Std.Nat --out build/release-audit/Std.Nat/manifest.json --json
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
`null_not_allowed`、`order_violation`、`duplicate_normalized_result_hash`、
`duplicate_path`、`duplicate_field` のいずれかを入れます。
`normalized_store_entry_file_hash_mismatch` では `error.field = "normalized_store.results[].file_hash"`、
`expected_hash` に manifest entry の `file_hash`、`actual_hash` に参照先 file bytes hash を入れます。
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
invalid_hash_format | null_not_allowed | order_violation | duplicate_field` のいずれかを入れます。
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
filename が表す hash と referenced file bytes sha256 が一致しない場合は
`CommandError.reason_code = input_hash_mismatch` です。
release bundle command は、release policy、runner policy、challenge runner policy、target normalized result、
request / machine result / normalized store manifest、challenge output store manifest、
challenge coverage summary、auxiliary result、checker identity manifest、import lock、
AI audit input policy、
optional AI sidecar / validation response、optional compare validation response を
すべて explicit path + expected hash flag で受け取ります。
non-optional singleton input pair は、closed set 上で同じ artifact に deduplicate される場合でも
flag としては required です。
repeatable input flags such as `--auxiliary-result`、`--checker-identity-manifest`、
`--import-lock`、`--ai-sidecar`、`--audit-sidecar-validation`、
`--compare-validation-response` are paired with their corresponding `--*-hash` flag
by occurrence order.
path without hash、hash without path、or unequal pair counts are `CommandError.reason_code = input_reference_invalid`。
missing required flag、duplicate singleton flag、unsupported flag、missing `--json` は
CLI argument validation error であり、`CommandError` body を返しません。
明示された input set が release audit bundle の closed set と一致しない場合は
`CommandError.reason_code = release_bundle_generation_failed` で、partial bundle を成功扱いしてはいけません。
directory scan、glob、bundle-root からの暗黙発見、policy hash からの store lookup は forbidden です。

release bundle command の explicit input validation order は次で固定します。

```text
1. pair shape / provided path schema / provided hash schema:
   reason_code = input_reference_invalid
   field = "<flag>" or "<flag>.path"
   expected_value = "workspace_relative_path" | "sha256:<lower-hex>" | "required_pair"
   actual_value = invalid_path | invalid_hash_format | missing_pair | unequal_pair_count

2. bundle-local artifact path shape:
   reason_code = input_reference_invalid
   field = "<flag>.path"
   expected_value = "bundle_artifact_path:<kind>"
   actual_value = invalid_path | kind_mismatch | invalid_filename

3. file readable:
   reason_code = input_file_unreadable
   field = "<flag>.path"
   actual_value = "unreadable"

4. filename hash vs file bytes:
   reason_code = input_hash_mismatch
   field = "<flag>.path"
   expected_hash = "sha256:<filename_without_json>"
   actual_hash = referenced file bytes sha256

5. JSON parse:
   reason_code = input_json_invalid
   field = "<flag>.path"
   actual_value = "invalid_json"
   store manifest JSON parse failure uses input_store_manifest_invalid
   with field = "<flag>.path" and actual_value = "invalid_json"

6. artifact schema / domain validation:
   reason_code = input_schema_invalid
   field = "<flag>.artifact.<json_path>"
   store manifest schema / order / duplicate failure uses input_store_manifest_invalid
   with field = "<flag>.artifact.<json_path>"

7. caller hash flag vs parsed artifact hash or file bytes hash:
   reason_code = input_hash_mismatch
   field = "<hash_flag>"
   expected_hash = caller supplied hash
   actual_hash = parsed artifact hash or referenced file bytes sha256

8. closed set / cross-artifact validation:
   reason_code = release_bundle_generation_failed
   field = "command"
   actual_value = "release_bundle_generation_failed"
```

Step 7 の actual hash は、下の input hash flag table で「parsed」と書かれた flag では
parsed artifact canonical hash、「file bytes」と書かれた flag では referenced file bytes sha256 です。
Step 4 は常に deterministic artifact path の filename と file bytes sha256 の照合です。
したがって parsed hash flag が正しくても filename hash が file bytes と違う input は Step 4 で失敗します。

release bundle command の input hash flag の意味は次で固定します。

```text
--release-policy-hash:
  parsed ReleasePolicy canonical hash
--runner-policy-hash:
  parsed RunnerPolicy canonical hash
--challenge-runner-policy-hash:
  parsed challenge RunnerPolicy canonical hash
--artifact-hash:
  parsed target NormalizedCheckResult.artifact_hash and top-level ReleaseAuditBundleManifest.artifact_hash
--target-normalized-result-hash:
  parsed NormalizedCheckResult.normalized_result_hash
--request-store-hash:
  request store manifest file bytes sha256
--result-store-hash:
  machine result store manifest file bytes sha256
--normalized-store-hash:
  normalized result store manifest file bytes sha256
--challenge-store-hash:
  ChallengeOutputStoreManifest file bytes sha256
--coverage-summary-hash:
  parsed ChallengeCoverageSummary.summary_hash
--auxiliary-result-hash:
  parsed AuxiliaryResult.result_hash
--checker-identity-manifest-hash:
  checker identity manifest file bytes sha256
--import-lock-hash:
  import lock manifest file bytes sha256
--ai-audit-input-policy-hash:
  parsed AiAuditInputPolicy canonical hash
--ai-sidecar-hash:
  AiAuditSidecar file bytes sha256
--audit-sidecar-validation-hash:
  AuditSidecarValidationResult file bytes sha256
--compare-validation-response-hash:
  CompareValidationResult file bytes sha256
```

`ReleasePolicy.ai_triage.enabled = true` の場合、`--ai-audit-input-policy` と
`--ai-audit-input-policy-hash` は required です。
`ReleasePolicy.ai_triage.enabled = false` の場合、これらの flag は forbidden です。
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
`schema` が `npa.phase8.release_audit_bundle_manifest.v1` でない場合、
`bundle_hash` / `policy_hash` / `artifact_hash` が missing / explicit null / wrong type /
invalid hash format の場合、または `artifacts` が missing / explicit null / wrong type の場合は
`CommandError` です。
minimum audit envelope schema failure の field shape は次で固定します。

```text
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
minimum audit envelope の top-level schema mismatch では `actual_value = "wrong_schema"` を使いません。
期待 schema とは異なる string は `actual_value = "invalid_enum"`、explicit null は
`actual_value = "null_not_allowed"` とします。
minimum audit envelope が valid な後の bundle schema / domain invalid は
`AuxiliaryResult.kind = audit_bundle`、`status = failed`、
`error.reason_code = audit_bundle_invalid` です。
たとえば derived `bundle_id` mismatch、artifact entry schema violation、order violation、
closed set violation、cross-artifact mismatch は `audit_bundle_invalid` です。
bundle manifest が readable かつ hash-verified で、parsed `bundle_hash` を特定できた後の
bundle invalid は `AuxiliaryResult.kind = audit_bundle`、`status = failed`、
`error.reason_code = audit_bundle_invalid` を出力します。
referenced bundle file が読めない、または missing の場合は
`AuxiliaryResult.kind = audit_bundle`、`status = failed`、
`error.reason_code = audit_bundle_missing` を出力します。
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
生成する request の `request_hash` 規則は 3.3 と同じです。
生成する request の `request_id` は
`chreq:` + `ChallengeManifest.challenge_id` + `:` + `checker_profile` に固定します。
生成する request file path は `--request-dir/<checker_profile>.json` です。
各 request の `module` は `ChallengeManifest.module`、
`imports` は `ChallengeManifest.imports`、
`certificate.path` と `certificate.file_hash` は
`ChallengeManifest.mutated_certificate.path` / `file_hash` を使います。
`certificate.expected_certificate_hash` は 10 の decode 不能 placeholder 規則に従います。
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

materialization `CommandError` の field は固定します。
`challenge_manifest_file_unreadable` では `field = "challenge_manifest.path"`、
`actual_value = "unreadable"` にします。
`challenge_manifest_hash_mismatch` では `field = "challenge_manifest.manifest_hash"`、
`expected_hash` に caller 指定 hash、`actual_hash` に manifest file bytes hash を入れます。
`challenge_manifest_json_invalid` では `field = "challenge_manifest.path"`、
`actual_value = "invalid_json"` にします。
`challenge_manifest_schema_invalid` では `field` に invalid manifest field の JSON path、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_enum`、
`invalid_hash_format`、`invalid_path`、`null_not_allowed`、`order_violation`、
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
`request_output_path_conflict` では `field` に衝突した generated request path、
`expected_hash` に今回生成する request file hash、`actual_hash` に既存 file bytes hash を入れます。
`request_store_manifest_invalid` では、既存 request store manifest file を読めない場合
`field = "request_store_output_path"`、`actual_value = "unreadable"` にします。
JSON として壊れている場合は同じ field で `actual_value = "invalid_json"` にします。
schema / order / duplicate 違反では `field` に invalid request store manifest field の JSON path、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_hash_format`、
`null_not_allowed`、`order_violation`、`duplicate_request_hash`、`duplicate_path`、
`duplicate_field` のいずれかを入れます。
`request_store_entry_file_unreadable` では `field = "request_store.requests[].path"`、
`actual_value = "unreadable"` にします。
`request_store_entry_json_invalid` では `field = "request_store.requests[].path"`、
`actual_value = "invalid_json"` にします。
`request_store_entry_schema_invalid` では `field` に invalid request field の JSON path、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_enum`、
`invalid_path`、`invalid_hash_format`、`null_not_allowed`、`duplicate_field` のいずれかを入れます。
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
`request_store_entry_conflict` では `field = "request_store.requests[]"`、
`expected_value` に追加予定 entry の canonical JSON string、
`actual_value` に衝突した既存 entry の canonical JSON string を入れます。
`request_output_write_failure` では `field` に request path、
`request_store_write_failure` では `field = "request_store_output_path"` とし、
どちらも `actual_value = "write_failed"` にします。
複数の失敗条件が同時にある場合は、この一覧の順序で最初に該当した
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
`--manifest` / `--manifest-hash` required pair の欠落または片側指定は CLI argument validation error であり、
`CommandError` body を返しません。
両方が指定された後の manifest reference schema violation、manifest file unreadable、
manifest file hash mismatch、manifest JSON invalid、minimum audit envelope schema invalid、
および output write failure は command pipeline 自体が成立しないため `CommandError` です。
minimum audit envelope が valid で、検証対象 bundle_hash / policy_hash / artifact_hash を特定できた後の
referenced bundle file missing / unreadable は `AuxiliaryResult.status = failed`、
`error.reason_code = audit_bundle_missing` です。
`field` は原因になった CLI flag、API field、または workspace-relative path を指します。
`expected_hash`、`actual_hash`、`expected_value`、`actual_value`、`diagnostics` は
該当する場合だけ入れます。
`diagnostics` は optional array of string で、人間向け説明専用です。
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
`invalid_path`、`null_not_allowed`、`order_violation`、`duplicate_field` のいずれかを入れます。
`*_generation_failed` では `field = "command"`、`actual_value` に
`coverage_summary_generation_failed`、`release_bundle_generation_failed`、または
`training_export_generation_failed` を入れ、詳細は `diagnostics` にだけ置きます。
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
実装は replay result と replay store manifest を temporary file として作り、
final replay result path を配置してから manifest を atomic replace します。
manifest が final replay result path と file hash を参照して初めて store 更新成功です。
store commit 前に failure した場合、manifest を更新してはいけません。
retry 時に final replay result path が既に存在し、その file bytes が今回書く
`ChallengeReplayResult` file bytes と完全一致する場合は、上書きではなく既存 file の採用として扱います。
既存 final replay result path の bytes が異なる場合は `replay_output_path_conflict` です。
既存 replay store manifest に同じ `result_hash`、`path`、`file_hash` の entry が既にある場合は
idempotent success として扱います。
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
API では audit-sidecar validation の active かつ mode-forbidden ではない `sidecar.path`、
`result_store.path`、`normalized_store.path`、`input_policy.path` が workspace path validation に失敗した場合、
`AuditSidecarValidationResult` ではなく常に `ApiError.reason_code = api_path_outside_workspace` を返します。
そのため API body のこれらの path については
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
inline artifact として渡される完全 `MachineCheckRequest`、`ChallengeGenerationRequest`、
`MachineCheckResult`、`NormalizedCheckResult` 内部の duplicate key は API wrapper error ではなく、
各 endpoint 固有の schema validation failure として返します。
mode-dependent forbidden reference field の payload 内部に duplicate key があっても、
nested payload は検査せず、duplicate key は `api_request_schema_invalid` にしません。
forbidden reference field 名そのものが親 object で duplicate している場合だけ、
wrapper object の duplicate key として `api_request_schema_invalid` にします。
`api_request_schema_invalid` では `field` に invalid wrapper field の JSON path、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_enum`、
`invalid_path`、`invalid_hash_format`、`null_not_allowed`、`order_violation`、
`duplicate_field` のいずれかを入れます。
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
wrapper validation 通過後に policy file が読めない場合は
`NormalizeErrorResult.error.reason_code = policy_file_unreadable`、
policy file が JSON parse または `RunnerPolicy` schema / domain validation に失敗した場合は
`policy_reference_invalid`、読み込んだ policy の canonical hash が
`RunnerPolicyReference.hash` と一致しない場合は `policy_hash_mismatch` にします。
`machine_results` inline object、request store reference、または request store entry の
endpoint-specific validation failure も `NormalizeErrorResult` にします。
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
`invalid_enum`、`invalid_hash_format`、`null_not_allowed`、`order_violation`、
`duplicate_field` のいずれかを入れます。
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
`RunnerPolicy` schema / domain validation failure では `error.field` に invalid policy field の
JSON path を入れ、`expected_value` / `actual_value` は 4.1 の
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
wrapper object 自体の schema violation または API path validation failure は `ApiError` にします。
wrapper validation を通った後の manifest / store reference validation failure、
policy reference validation failure、または replay pipeline failure は `CommandError` にします。
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
欠落、object type、required member、hash format、explicit null は wrapper schema violation ではなく
step 4 または step 5 の `AuditSidecarValidationResult` として返します。
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
6. input_policy reference / sidecar / file hash の3者一致、file readable / schema、
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
2 で CLI / 非 API caller の `sidecar.path` が invalid path の場合も
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
`null_not_allowed`、`present` のいずれかを入れます。
`actual_value = invalid_path` の場合、`expected_value` は `workspace_relative_path` に固定します。
validation reference object またはその member が explicit null の場合は
`actual_value = null_not_allowed` にします。
`input_policy_file_unreadable` では `error.field = "input_policy.path"`、
`actual_value = "unreadable"` にします。
`input_policy_schema_invalid` では `error.field` に invalid input policy field の JSON path を入れ、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、
`invalid_enum`、`invalid_hash_format`、`null_not_allowed`、`order_violation`、
`duplicate_field` のいずれかを入れます。
`input_policy_hash_mismatch` では `error.field = "input_policy.hash"` を入れます。
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
`result_store_manifest_invalid` と `normalized_store_manifest_invalid` では、
manifest file を読めない場合は `error.field = "result_store.path"` または
`error.field = "normalized_store.path"`、`actual_value = "unreadable"` にします。
JSON として壊れている場合は同じ field で `actual_value = "invalid_json"` にします。
schema / order / duplicate 違反では `error.field` に invalid manifest field の JSON path を入れ、
`expected_value` に schema requirement 名、
`actual_value` に `missing`、`wrong_type`、`unknown_field`、`invalid_hash_format`、
`invalid_path`、`null_not_allowed`、`order_violation`、`duplicate_field`、または manifest 種別ごとの
unique key duplicate reason を入れます。
`result_store_manifest_invalid` で許可する unique key duplicate reason は
`duplicate_run_artifact_hash` と `duplicate_path` だけです。
`normalized_store_manifest_invalid` で許可する unique key duplicate reason は
`duplicate_normalized_result_hash` と `duplicate_path` だけです。
`duplicate_run_artifact_hash` を normalized store manifest に使ってはいけません。
`duplicate_normalized_result_hash` を result store manifest に使ってはいけません。
`referenced_file_hash_mismatch` では `error.field` に
`result_store.results[].file_hash` または `normalized_store.results[].file_hash` を入れ、
`expected_hash` には manifest entry の file hash、
`actual_hash` には参照 file bytes から再計算した hash を入れます。
`referenced_artifact_hash_mismatch` では `error.field` に
`result_store.results[].result_hash`、`result_store.results[].request_hash`、
`result_store.results[].run_artifact_hash`、または
`normalized_store.results[].normalized_result_hash`、
`normalized_store.results[].artifact_hash` を入れます。
store entry artifact の self-hash mismatch では、`expected_hash` には parsed artifact から
再計算した hash、`actual_hash` には parsed artifact 内の self-hash field を入れます。
複数の self-hash field がある artifact の検査順は、challenge replay の
store entry validation と同じ順序にします。
self-hash が valid な artifact と manifest entry の mismatch では、
`expected_hash` には manifest entry の hash、
`actual_hash` には parsed artifact field の hash を入れます。
`referenced_artifact_value_mismatch` では `error.field` に
`result_store.results[].checker_profile` または `status` を入れます。
store entry checker profile mismatch では `expected_value` に manifest entry の `checker_profile`、
`actual_value` に parsed `MachineCheckResult.checker.profile` を入れます。
source artifact 状態に対して sidecar status が許可されない場合は
`error.field = "status"`、`expected_value` に許可 status set 名、
`actual_value` に `AiAuditSidecar.status` を入れます。
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
`expected_value` と `actual_value` を入れます。
step 10 の `source.kind = machine_result` は、まず `source.run_artifact_hash` を
machine result store の unique key として lookup します。
該当 entry がなければ `source_result_not_found` を返します。
lookup 成功後は `source.result_hash`、`source.request_hash` の順に照合し、
最初の mismatch だけを `source_hash_mismatch` として返します。
`source.run_artifact_hash` は lookup key なので、lookup 成功後の `source_hash_mismatch.field` には使いません。
machine result hash が一致した後、`source.result_id` が存在する場合は
参照先 `MachineCheckResult.result_id` と照合し、mismatch なら `source_id_mismatch` を返します。
`source.kind = machine_result` で `source.normalized_result_id` が存在し、
`source.normalized_result_hash` が存在しない場合は、normalized store lookup を行わず
`sidecar_schema_invalid` を返します。
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
source id / normalized membership check が通った後で sidecar status 許可条件を検査します。
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
- MachineCheckRequest request_hash is required and distinct from request file_hash
- MachineCheckRequest top-level schema mismatch returns request_schema_invalid with fixed schema field shape
- npa-check run requires an explicit RunnerPolicyReference and does not resolve policy from MachineCheckRequest alone
- npa-check run and /machine/check/certificate default attempt to 1 and copy explicit positive attempt without scanning result stores
- CLI commands requiring RunnerPolicyReference reject missing --policy-hash
- npa-check run malformed RunnerPolicyReference reports runner_policy_reference_invalid with the same member-level field shape as non-run policy_reference_invalid
- RunnerPolicyReference hash must match parsed RunnerPolicy canonical hash and MachineCheckRequest.policy.hash
- unreadable or hash-mismatched RunnerPolicyReference returns the dedicated runner policy reason_code
- unreadable checker executable returns checker_binary_file_unreadable before process launch
- checker identity manifest unreadable / hash mismatch / invalid schema returns the dedicated policy_failure reason_code before checker launch
- checker identity manifest top-level schema mismatch reports checker_identity_manifest.schema and never uses wrong_schema
- checker identity manifest pre-launch policy matching checks only MachineCheckRequest.checker_profile for that run
- raw checker_id / checker_build_hash missing is checker_identity_missing, not malformed raw output
- checker_version missing or mismatch does not reject an otherwise valid checker result
- checker.version is omitted from result_hash but included in run_artifact_hash
- CheckerRawResult requires schema and rejects unknown / duplicate / null fields as raw schema failures
- exit 1 with status checked is missing_rejection_error, while exit 2 with non-internal error is malformed_internal_error_output
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
- high-trust required_checker_profiles includes release profiles plus high-trust-reference
- RunnerPolicy checker_allowlist is sorted by profile and rejects duplicate profile or binary_id
- RunnerPolicy checker_allowlist and budgets profile sets exactly match required plus optional profiles
- runner command construction appends import, trust, axiom policy, and budget dynamic args after allowed_args and before certificate path
- checker executable resolution uses runner-owned CheckerBinaryRegistry and validates final target bytes against binary_hash
- runner rejects axiom policy file hash mismatch before checker launch
- sidecar input_policy hash mismatch is rejected
- request certificate file_hash mismatch is rejected before checker launch
- import manifest_hash mismatch is rejected before checker launch
- import lock manifest entries are sorted, path-bound, and verified by file bytes hash before checker import use
- import lock manifest v1 rejects entries missing certificate.certificate_hash
- npa-check run maps invalid import lock manifest JSON / schema / domain to request_import_manifest_invalid before checker launch
- malformed CheckerRawResult becomes checker_internal_error
- CheckerRawResult module mismatch becomes checker_internal_error
- policy_failure uses reason_code and does not hash human text
- MachineCheckResult infrastructure reason_code is closed enum
- checked-result sidecar omits classification.checker_error_kind
- NormalizedCheckResult failed entry includes failure_key
- NormalizedCheckResult artifact identity ignores request_hash
- NormalizedCheckResult has top-level artifact_hash matching the artifact object hash
- NormalizedCheckResult results are ordered by RunnerPolicy profile order
- network import resolution is rejected in Phase 8 runner
- `npa-check run` short form cannot override policy budget or checker path
- all_agree_failed requires matching failure_key, not only matching error.kind
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
- decode-failure challenge request uses deterministic expected_certificate_hash placeholder
- challenge generate --kind accepts the same enum as ChallengeManifest.mutation.kind
- ChallengeGenerationRequest requires policy_hash, module, imports, base_certificate, mutation kind/target/seed, and output paths
- ChallengeGenerationRequest.challenge_id is copied exactly to ChallengeManifest.challenge_id
- ChallengeGenerationRequest.policy_hash is copied exactly to ChallengeManifest.policy_hash
- ChallengeGenerationRequest module and imports are copied exactly to ChallengeManifest
- ChallengeGenerationRequest imports.mode is required and must match RunnerPolicy.import_policy.mode
- ChallengeGenerationRequest request_hash is required and must match the canonical self hash before generation reads inputs or writes outputs
- CLI challenge generate may read --from only during request construction before request_hash validation, and that phase performs no output writes
- challenge generate computes base_certificate file_hash and claimed_certificate_hash from --from
- challenge generation API revalidates base_certificate file_hash and claimed_certificate_hash from file bytes
- challenge generate --json and /machine/check/challenge success return ChallengeGenerationResult without certificate bytes
- challenge generate failure returns CommandError on stderr/API body and no ChallengeGenerationResult
- challenge generate requires --generated-by and enforces --prompt-hash only for generated_by = ai
- conflicting duplicate ChallengeGenerationRequest.challenge_id in an output store is generation failure, while exact entry retry is idempotent success
- challenge generate requires --challenge-store and checks duplicate challenge_id only against that store manifest
- challenge generate writes a sorted ChallengeOutputStoreManifest entry on success
- challenge generate reports ChallengeOutputStoreManifest schema/order/duplicate failures with fixed challenge_output_store_manifest_invalid fields
- challenge generate may atomically update ChallengeOutputStoreManifest but refuses to overwrite differing manifest-out and mutated-out artifacts
- challenge generate treats ChallengeOutputStoreManifest as commit point and can adopt exact-match orphan manifest / mutated certificate files on retry
- ChallengeGenerationRequest request_hash matches ChallengeManifest.replay.args_hash
- challenge generate rejects kind-specific invalid mutation.target before writing artifacts
- challenge mutation selection is deterministic from canonical byte-offset candidates and seed
- add_forbidden_axiom recomputes certificate hashes while other MVP mutations leave stored hashes stale unless specified
- replace_nat_zero_with_noncanonical_placeholder writes reserved invalid term tag 0xff and does not add placeholder to core calculus
- challenge generate rejects existing manifest or mutated certificate paths only when file bytes differ from generated bytes
- MVP challenge mutation accepted by a required checker is unexpected checker acceptance
- challenge commands treat missing policy flags as CLI argument errors, malformed provided policy references as CommandError policy_reference_invalid, and API wrapper policy shape/path failures as ApiError
- ChallengeReplayResult manifest_hash is the ChallengeManifest file bytes hash
- ChallengeReplayResult artifact_hash comes from NormalizedCheckResult or replay request artifact
- challenge materialize-requests creates policy-ordered replay MachineCheckRequest files and a request store manifest without running checkers
- challenge materialize-requests derives module/imports from ChallengeManifest and certificate fields from mutated_certificate
- challenge materialize-requests rejects ChallengeManifest.policy_hash mismatch with RunnerPolicyReference.hash
- challenge materialize-requests rejects ChallengeManifest.imports.mode mismatch with RunnerPolicy.import_policy.mode
- challenge materialize-requests returns ChallengeRequestMaterializationResult with request store manifest_hash
- challenge materialize-requests treats request store manifest as commit point and can adopt exact-match orphan files on retry
- challenge materialize-requests failure returns CommandError on stderr/API body and no ChallengeRequestMaterializationResult
- challenge materialize-requests CommandError field shapes are fixed for manifest, policy, import, output, and store failures
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
- ChallengeReplayStoreManifest rejects duplicate result_hash, duplicate path, and duplicate challenge_id/manifest_hash pairs
- challenge replay store entry top-level schema mismatch reports the schema field and never uses wrong_schema
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
- normalize request_store_manifest_invalid uses fixed field / expected_value / actual_value shapes
- compare without resolvable RunnerPolicy is rejected
- compare CLI requires --policy and --policy-hash
- normalize maps broken request store files to NormalizeErrorResult
- normalize rejects non-MachineCheckResult inputs with machine_result_wrong_schema
- normalize separates machine_result_wrong_schema from machine_result_schema_invalid for schema null/type/unknown/duplicate cases
- normalize rejects MachineCheckResult result_hash / run_artifact_hash mismatch
- normalize rejects MachineCheckResult whose request_hash disagrees with request store
- normalize request store manifest request_hash mismatch reports expected_hash from manifest and actual_hash from parsed request
- normalize maps wrong-schema request store entries to request_schema_invalid with request_store.requests[].schema
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
- API store references require manifest_hash and reject path-only store references
- API ChallengeManifest references require manifest_hash and reject path-only manifest references
- API invalid JSON request bodies return ApiError with api_json_invalid
- API wrapper schema and workspace path validation failures return ApiError, not endpoint artifacts or CommandError
- API duplicate-aware decoder rejects endpoint wrapper duplicate keys as api_request_schema_invalid with duplicate_field
- API duplicate schema_only mode discriminator returns api_request_schema_invalid before mode-dependent reference validation
- API duplicate keys inside inline artifacts are routed to endpoint-specific schema validation failures
- API challenge generation output/base/import paths inside inline ChallengeGenerationRequest return CommandError generation_request_schema_invalid, not ApiError
- API duplicate keys inside mode-forbidden reference payloads do not override the forbidden-reference validation_reference_schema_invalid path
- API audit-sidecar validation path schema failures return api_path_outside_workspace, not validation_reference_schema_invalid
- API audit-sidecar missing active validation references return AuditSidecarValidationResult validation_reference_missing, not ApiError
- API audit-sidecar partial validation references return AuditSidecarValidationResult validation_reference_schema_invalid, not ApiError
- API audit-sidecar existing reference objects with missing required members return validation_reference_schema_invalid, not validation_reference_missing
- API audit-sidecar input_policy.kind is rejected as validation_reference_schema_invalid unknown_field
- API domain file read/write failures use endpoint-specific error schemas after wrapper validation succeeds
- API normalize and compare wrappers accept inline artifacts only, while challenge replay uses manifest/store references
- normalize selector module mismatch returns NormalizeErrorResult
- omitted normalize selector is rejected when first required profile has zero or multiple results
- normalize uses RunnerPolicy.axiom_policy.hash for artifact.axiom_policy_hash
- policy_file_unreadable NormalizeErrorResult keeps policy_hash from RunnerPolicyReference
- NormalizeErrorResult omits policy_hash when RunnerPolicyReference.hash is missing, non-string, null, or invalid format
- NormalizeErrorResult policy_reference_invalid / policy_file_unreadable / policy_hash_mismatch use fixed field shapes
- API normalize policy wrapper schema/hash/path failures return ApiError, while policy file unreadable/invalid/hash mismatch returns NormalizeErrorResult
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
- sidecar source id fields are optional but must match referenced artifacts when present
- AiAuditSidecar status-dependent required fields are enforced
- AiAuditSidecar source-status rules allow normalized_comparison missing_checker_result and policy_failure required targets
- audit-sidecar schema-only validation does not enforce source-artifact-dependent sidecar status permissions
- audit-sidecar schema-only validation treats machine_result classification.checker_error_kind as optional enum-only metadata
- audit-sidecar cross-artifact validation rejects sidecar status not allowed by the referenced source artifact status
- normalized_comparison sidecar lookup validates normalized_result_id before source status permission
- AiAuditSidecar optional classification omits checker_error_kind checks for summarized and inconclusive sidecars
- AiAuditSidecar source/input_policy/ai nested required fields are enforced
- AiAuditInputPolicy included_fields rejects duplicates, unknown fields, and order violations
- sidecar copied input_policy.included_fields duplicate or order violation returns sidecar_schema_invalid
- policy-gated source/tactic sidecar fields are not generic unknown fields and are checked against input policy at step 7
- policy-gated source/tactic sidecar fields are only allowed at top-level paths; nested occurrences are forbidden_sidecar_field
- duplicate keys in sidecar/input-policy/store files are schema invalid with duplicate_field, not JSON invalid
- invalid AiAuditInputPolicy file returns input_policy_schema_invalid
- AiAuditInputPolicy copied metadata must match the policy file
- audit-sidecar cross-artifact validation requires validation reference input_policy.hash, sidecar input_policy.hash, and input policy file canonical hash to match
- audit-sidecar input_policy hash mismatch precedence reports reference-vs-sidecar before reference-vs-file before sidecar-vs-file
- audit-sidecar schema-only validation does not mark cross-artifact claims as validated
- audit-sidecar schema-only validation rejects cross-artifact validation references
- audit-sidecar cross-artifact validation requires result store and input_policy
- audit-sidecar cross-artifact validation maps missing active reference pairs to validation_reference_missing, not CLI argument error
- audit-sidecar cross-artifact validation maps partial path/hash reference pairs to validation_reference_schema_invalid with actual_value missing
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
- post-launch timeout/resource exhaustion uses checker_timeout/checker_resource_exhausted
- run_artifact_hash changes when diagnostics changes, while result_hash does not
- ChallengeReplayResult result_hash is verified as a saved artifact hash in release audit
- ReleasePolicy schema has explicit ai_triage enabled/required fields with no defaults and conditional input_policy_hash
- ReleasePolicy schema/domain failures use fixed field / expected_value / actual_value shapes, including ai_triage conditions
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
- release stage-bundle-inputs plan separates file_hash from kind-specific parsed hashes, rejects expected_hash, unknown fields, duplicate inputs, and order violations
- release stage-bundle-inputs rejects invalid --bundle-root and plan bundle_root path schema and bundle_root / --bundle-root mismatch with fixed fields
- release stage-bundle-inputs enforces the phase-specific allowed input kind table
- release stage-bundle-inputs store phase cardinality violations report fixed expected_value and actual_value fields
- release stage-bundle-inputs final phase does not validate release bundle closed-set cardinality
- release stage-bundle-inputs resolves source store manifest entry paths from invocation cwd, not from the manifest file parent directory
- release stage-bundle-inputs uses repository root as invocation cwd for every MVP mode
- release stage-bundle-inputs maps plan, source, source-store, merge, conflict, and write failures to the fixed CommandError reason codes
- release stage-bundle-inputs input_store_entry_invalid reports deterministic actual_value / expected_hash / actual_hash fields and first failing entry validation step
- release stage-bundle-inputs source store manifest duplicate reasons and source artifact self-hash validation order are store-kind-specific
- release stage-bundle-inputs source store manifest errors include the staging plan input index in field paths
- release stage-bundle-inputs store phase stages target-scoped challenge_output_store_manifest before release coverage summary generation
- release stage-bundle-inputs does not validate challenge_output_store_manifest target scope; coverage-summary and bundle validation do
- ReleaseBundleStagingResult store_manifests records generated bundle-local manifests only, uses deterministic order, and records manifest_hash only, not a duplicate file_hash field
- ReleaseBundleStagingResult excludes generated store manifests from staged_artifacts and requires empty store_manifests in final phase
- ReleaseBundleStagingResult is not a release bundle validator input; validators use explicit bundle-local store manifest artifacts
- ReleaseAuditBundleManifest bundle-local manifest merge deduplicates only exact duplicate artifacts and rejects same-key or same-path conflicts
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
- release audit challenge output store deterministic filtering excludes informational ChallengeManifest entries
- ReleaseAuditBundleManifest forbids informational ChallengeManifest and informational ChallengeReplayResult entries
- ReleaseAuditBundleManifest treats import_lock path as bundle-local and validates identity by manifest_hash and file bytes, not by original imports.manifest path
- ReleaseAuditBundleManifest requires the closed set of passed AuxiliaryResult entries for release and high-trust modes and rejects missing, duplicate, extra, failed, or inconclusive entries
- AuxiliaryResult selector is required for axiom_policy and reproducibility, forbidden for import_certificate_hash and audit_bundle, and is included in result_hash
- ReleaseAuditBundleManifest validates axiom_policy and reproducibility AuxiliaryResult selectors against the release target baseline profile and included MachineCheckResult artifacts
- ReleaseAuditBundleManifest validates AuxiliaryResult envelopes and reference hashes without rerunning axiom_policy, reproducibility, or import_certificate_hash oracles in the MVP bundle
- release target NormalizedCheckResult policy.hash must match ReleasePolicy.runner_policy_hash in release audit
- ChallengeCoverageSummary policy_hash must match ReleasePolicy.challenge_runner_policy_hash in release audit
- ChallengeReplayResult underlying MachineCheckRequest, MachineCheckResult, and challenge replay NormalizedCheckResult policies match ReleasePolicy.challenge_runner_policy_hash
- coverage-required challenge replay requires normalized store and exactly one matching challenge replay NormalizedCheckResult
- ReleaseAuditBundleManifest includes challenge replay NormalizedCheckResult entries for each included ChallengeReplayResult.normalized_result_hash
- ReleaseAuditBundleManifest requires each MachineCheckResult request_hash to resolve to an included MachineCheckRequest
- ReleaseAuditBundleManifest rejects challenge_replay_result entries outside ChallengeCoverageSummary.entries[*].replay_result_hash
- ReleaseAuditBundleManifest rejects informational ChallengeReplayResult entries in the MVP
- ReleaseAuditBundleManifest rejects machine_check_result entries outside the closed allowed run set
- ReleaseAuditBundleManifest selects the release target baseline raw result by reproducibility.selector.baseline_run_artifact_hash
- ReleaseAuditBundleManifest rejects non-baseline duplicate retry results that cannot be selected unambiguously
- ReleaseAuditBundleManifest rejects machine_check_request entries outside the distinct request_hash set of included MachineCheckResult artifacts
- ReleaseAuditBundleManifest validates ChallengeCoverageSummary.summary_hash and unexpected_acceptances
- ChallengeCoverageSummary challenge_store_manifest_hash binds coverage to an explicit ChallengeOutputStoreManifest
- ChallengeCoverageSummary summary_id is derived from summary_hash and mismatch is coverage generation failure or bundle invalid
- ChallengeCoverageSummary target_normalized_result_hash and result_store_manifest_hash bind coverage to the explicit target and machine result store used for recomputing unexpected_acceptances
- release coverage summary generation uses staged challenge_output_store_manifest and staged machine result store manifest
- ChallengeOutputStoreManifest used for coverage is target-scoped and rejects global or multi-target stores
- ChallengeOutputStoreManifest split/filter validates every referenced ChallengeManifest before filtering and fails instead of skipping unreadable, invalid, hash-mismatched, or mutation-kind-invalid entries
- ChallengeOutputStoreManifest split/filter is a pre-bundle pipeline step and release audit bundle validation never reads original manifest_path
- ChallengeOutputStoreManifest split/filter uses manifest-local ChallengeManifest validation only and does not read base certificates, mutated certificates, import locks, or policy files
- ChallengeOutputStoreManifest entries and referenced ChallengeManifest base certificate fields must match the coverage target
- ReleaseAuditBundleManifest includes exactly one challenge_output_store_manifest and rejects challenge_manifest entries not referenced by it
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
- release validate-bundle minimum audit envelope CommandError field shapes are fixed for schema, bundle_hash, policy_hash, artifact_hash, and artifacts
- release validate-bundle returns CommandError for top-level manifest unreadable/hash mismatch and audit_bundle_missing for missing referenced bundle files after minimum envelope is known
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
- release bundle command requires both runner policy flag pairs even when runner_policy_hash equals challenge_runner_policy_hash and emits one runner_policy entry for the shared path
- release bundle command requires ai audit input policy when ReleasePolicy.ai_triage.enabled is true
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
- CompareValidationResult valid does not imply all_agree_checked
- CompareValidationResult and AuditSidecarValidationResult are transient responses without result_hash
- ChallengeGenerationResult, ChallengeRequestMaterializationResult, ReleaseBundleStagingResult, NormalizationWriteResult, CommandError, and ApiError are transient responses without result_hash
- CompareValidationResult rejects unreadable, invalid JSON, schema-invalid, and artifact_hash-invalid normalized results
- CompareValidationResult failed responses omit unavailable top-level hashes and comparison statuses
- CompareValidationResult validates normalized_result_hash before policy and comparison
- CompareValidationResult failure errors use fixed kind and expected/actual hash fields
- NormalizeErrorResult uses error.kind = normalize_failure
- AxiomReport axiom_report_hash excludes axiom_report_hash itself and rejects duplicate / unsorted axioms deterministically
- AxiomReport self-hash mismatch is reported separately from selector axiom_report_hash mismatch
- axiom-policy rejects AxiomReport module / certificate_hash that do not match the selected normalized result entry
- AxiomReportStoreManifest has deterministic order, unique axiom_report_hash/path keys, and file-byte manifest_hash
- axiom-policy validates NormalizedCheckResult artifact_hash and normalized_result_hash before using selector fields
- axiom-policy reports selector / normalized result entry mismatch and axiom report store entry hash mismatch with dedicated fields
- AuxiliaryResult kind-specific oracle inputs are deterministic
- auxiliary commands generate AuxiliaryResult from deterministic oracles and write failures are not converted to oracle inconclusive
- auxiliary axiom-policy and reproducibility commands map malformed non-policy references and store manifests to fixed CommandError reason codes
- auxiliary axiom-policy and reproducibility oracles emit first-failure AuxiliaryResult errors with fixed field / expected / actual shapes
- auxiliary axiom-policy and reproducibility store entry validation order is unreadable, JSON, schema, file_hash, self-hash, manifest-field mismatch
- reproducibility validates MachineCheckResult result_hash and run_artifact_hash self-hashes before selector or deterministic equality checks
- auxiliary import-certificate-hash maps readable hash-verified invalid import lock manifests to inconclusive, but unreadable or hash-mismatched top-level inputs to CommandError
- auxiliary import-certificate-hash rejects one-sided import lock path/hash pairs with input_reference_invalid
- auxiliary import-certificate-hash uses the built-in deterministic canonical certificate hash oracle and rejects non-high-trust ReleasePolicy
- auxiliary import-certificate-hash reports unreadable imported certificates as inconclusive and hash/decode/certificate_hash mismatches as failed
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
