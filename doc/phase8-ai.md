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

Phase 8 AI Profile の出力は4種類に分けます。

```text
1. MachineCheckResult
   runner が checker raw result と process / policy metadata から生成する正本 envelope。

2. NormalizedCheckResult
   複数 checker の結果を比較しやすくする正規化表現。
   verdict は checker result から機械的に写すだけ。

3. NormalizeErrorResult
   request store や policy 解決に失敗し、NormalizedCheckResult を作れない場合の error artifact。
   checker verdict ではなく pipeline error として扱う。

4. AiAuditSidecar
   AI が生成する説明・分類・修正候補。
   verdict として扱ってはいけない。
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
- producers should omit unknown optional fields instead of writing null
- paths are workspace-relative, use `/`, and contain no `.` / `..` segment
- hashes over files use exact file bytes, not parsed JSON values
```

Unicode string values are treated as UTF-8 bytes after parsing.
Hashing code must not apply locale-dependent case folding or path normalization.
If a schema needs human text normalization, that schema must say so explicitly.

`request_hash` は `request_id` と `request_hash` field を除いた
`MachineCheckRequest` の canonical hash です。
`result_hash` は deterministic runner-envelope verdict の hash です。
checker が出した raw verdict だけでなく、runner が policy と照合した後に正本として保存する
`policy`、`runner`、`checker` identity、`status`、`error`、`certificate_hash`、
`export_hash`、`axiom_report_hash` などを hash 対象に含めます。
同じ checker raw result でも、policy hash、runner build hash、checker binary hash が違えば
別の `result_hash` になります。

`result_hash` から除外する field は次です。

```text
- request_id
- result_id
- request_hash
- result_hash
- run_artifact_hash
- attempt
- process
- resource_usage
- diagnostics
```

`run_artifact_hash` は `run_artifact_hash` field 自身を除いた full artifact hash です。
`run_artifact_hash` は canonicalized object hash であり、保存ファイル bytes の hash ではありません。
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

runner は実行前に次を検査します。

```text
- request.policy.hash が読み込んだ RunnerPolicy の canonical hash と一致する
- request.trust_mode が RunnerPolicy.trust_mode と一致する
- request.checker_profile が RunnerPolicy.required_checker_profiles または
  explicitly allowed optional profiles に含まれる
- request.axiom_policy が RunnerPolicy.axiom_policy.path と一致する
- request.budget が RunnerPolicy.budgets[checker_profile] と一致する
- request.imports.mode が RunnerPolicy.import_policy.mode と一致する
- request.imports.manifest_hash が import lock file bytes の hash と一致する
- request.certificate.file_hash が input certificate file bytes の hash と一致する
```

一致しない場合、runner は checker を起動せず `policy_failure` result を保存します。
「policy を優先して request を黙って修正する」動作は禁止です。

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
別値が入っている policy は `policy_failure` として扱います。

`trust_mode` ごとの MVP 必須 profile：

```text
pr:
  required_checker_profiles = [reference]

nightly:
  required_checker_profiles = [reference, external]

release:
  required_checker_profiles = [fast-kernel, reference, external]

high-trust:
  required_checker_profiles = [reference, external, high-trust-reference]
```

`binary_hash` は実行ファイル bytes の hash です。
`build_hash` は checker build identity です。
runner は起動前に `binary_hash` を allowlist と照合し、一致しなければ
checker を起動せず `policy_failure` result を保存します。
`build_hash` は、repository / CI が管理する checker identity manifest で起動前に照合できる場合は
起動前に照合します。
manifest がない場合でも、runner は checker 起動後に `CheckerRawResult.checker_build_hash` を
同じ allowlist entry と照合します。
起動後の build hash mismatch は checker verdict として扱わず、
`error.kind = policy_failure`、`error.reason_code = checker_build_hash_mismatch` の
`MachineCheckResult` として保存します。
この result の `checker.build_hash` には checker が実際に報告した actual build hash を記録します。
allowlist 側の expected build hash は `error.expected_hash`、actual build hash は
`error.actual_hash` に記録し、`error.field = "checker.build_hash"` とします。
checker が build hash を報告できなかった場合は `checker.build_hash` を omit し、
`error.reason_code = checker_identity_missing` とします。

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
runner は `checker_id` / `checker_build_hash` を同じ profile の allowlist entry と照合し、
process status と矛盾しない場合だけ `MachineCheckResult` に写します。
`checker_id` mismatch は `policy_failure` であり、
`error.reason_code = checker_identity_mismatch`、
`error.field = "checker.id"` として保存します。
allowlist 側の expected id は `error.expected_value`、checker が報告した actual id は
`error.actual_value` に記録します。

`CheckerRawResult` の required / optional field：

```text
status = checked:
  required:
    - checker_id
    - checker_version
    - checker_build_hash
    - status
    - module
    - certificate_hash
    - export_hash
    - axiom_report_hash
  forbidden:
    - error

ordinary status = failed:
  required:
    - checker_id
    - checker_version
    - checker_build_hash
    - status
    - module
    - certificate_hash, unless failure is before canonical hash recomputation
    - error.kind
  optional:
    - export_hash
    - axiom_report_hash
    - error.declaration
    - error.core_path
    - error.expected_hash
    - error.actual_hash

decode / schema / noncanonical failure:
  required:
    - checker_id
    - checker_version
    - checker_build_hash
    - status = failed
    - module, if decodable
    - error.kind
  optional:
    - certificate_hash
    - error.section
    - error.offset

checker internal error:
  required:
    - checker_id
    - checker_version
    - checker_build_hash
    - status = failed
    - error.kind = checker_internal_error
    - error.reason_code
  optional:
    - module
```

MVP の error kind ごとの field requirement：

```text
certificate_decode_error:
  group: decode / schema / noncanonical failure
  required error fields: kind, section or offset when available
  certificate_hash: optional

noncanonical_encoding:
  group: decode / schema / noncanonical failure
  required error fields: kind, section or offset when available
  certificate_hash: optional

unsupported_schema_version:
  group: decode / schema / noncanonical failure
  required error fields: kind
  certificate_hash: optional

import_not_found:
  group: ordinary status = failed
  required error fields: kind, expected_hash when available
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
  required error fields: kind, reason_code when available
  certificate_hash: optional
```

checker raw output の異常時処理：

```text
exit 0 + invalid JSON:
  status = failed, error.kind = checker_internal_error,
  error.reason_code = malformed_success_output

exit 0 + status != checked:
  status = failed, error.kind = checker_internal_error,
  error.reason_code = success_exit_status_mismatch

exit 1 + missing structured error:
  status = failed, error.kind = checker_internal_error,
  error.reason_code = missing_rejection_error

exit 1 + invalid JSON:
  status = failed, error.kind = checker_internal_error,
  error.reason_code = malformed_rejection_output

exit 2 + invalid JSON:
  status = failed, error.kind = checker_internal_error,
  error.reason_code = malformed_internal_error_output

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
  process-level failure; runner records checker_internal_error unless a more
  specific runner error kind applies

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
    "reason_code": "request_budget_mismatch"
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
checker が `checker_id` / `checker_build_hash` を報告した場合は actual 値を
`checker.id` / `checker.build_hash` に記録します。
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
  "policy": {
    "id": "phase8-pr",
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
    "disagreements": []
  }
}
```

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
採用できないため `checker_id`、`checker_binary_hash`、`checker_build_hash` は省略されます。

正規化器は、AI ではなく deterministic code として実装します。
AI はこの結果を読んで説明を書くだけです。
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
normalizer の入力は `MachineCheckResult` list、`RunnerPolicy` または resolvable policy hash、
request store、および optional artifact selector です。
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
selector の `module` は、解決した `MachineCheckRequest.certificate.module` と一致しなければなりません。
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
    "kind": "policy_failure",
    "reason_code": "request_hash_not_found",
    "field": "request_hash",
    "actual_hash": "sha256:..."
  }
}
```

`NormalizeErrorResult.result_hash` は `result_id` と `result_hash` field を除いた
canonical hash です。
`policy_hash` は、入力 policy hash が指定されている場合、または policy object から canonical hash を
計算できる場合だけ入れます。policy file が読めず hash も指定されていない場合は omit します。
MVP の `NormalizeErrorResult.error.reason_code` は次に限定します。

```text
- request_hash_not_found
- request_file_hash_mismatch
- request_hash_mismatch
- request_store_manifest_hash_mismatch
- request_store_manifest_invalid
- policy_hash_not_found
- policy_hash_mismatch
- selector_module_mismatch
- selector_ambiguous
```

`request_hash_not_found` と `policy_hash_not_found` では
`field` と `actual_hash` を入れます。
`request_store_manifest_invalid` では `field` を入れ、hash / value fields は omit してよいです。
`request_file_hash_mismatch`、`request_hash_mismatch`、
`request_store_manifest_hash_mismatch`、`policy_hash_mismatch`、
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
compare step は canonical `RunnerPolicy` object、または同じ内容を policy store から解決できる
`policy.hash` を入力に取ります。

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

比較規則は deterministic code で次の優先順位に従います。

```text
1. top-level policy.hash と results[*].policy_hash が一致しない、
   または results[*].policy_hash 同士が一致しない
   -> policy_failure

2. error.kind = policy_failure の result がある
   -> policy_failure

3. process_launched = true かつ checker identity field が存在する result で、
   checker id / binary hash / build hash が policy allowlist と一致しない
   -> policy_failure

4. process_launched = true かつ checker identity field が不足している result で、
   error.kind が checker_internal_error / resource_exhausted / timeout 以外
   -> policy_failure

5. policy.required_checker_profiles の result が不足している
   -> missing_checker_result

6. results[*].artifact_hash が top-level artifact hash と一致しない、
   または results[*].artifact_hash 同士が一致しない
   -> disagreement

7. process_launched = false の許可済み runner failure、
   または resource_exhausted / checker_internal_error / timeout などで checker result が比較不能
   -> inconclusive

8. required checker の status がすべて checked
   かつ export_hash / axiom_report_hash がすべて一致する
   -> all_agree_checked

9. required checker の status がすべて failed
   かつ normalized failure key がすべて一致する
   -> all_agree_failed

10. 上記以外
   -> disagreement
```

checker allowlist 照合は `process_launched = true` の result にだけ適用します。
`process_launched = false` で checker identity が省略された result は、
`policy_failure` または `inconclusive` の判定規則で扱います。
malformed output などで checker が起動済みでも identity を得られない場合は、
`checker_internal_error` として `inconclusive` に分類します。
policy mismatch と artifact mismatch が同時に存在する場合は policy mismatch を優先し、
comparison status は `policy_failure` にします。
これは異なる policy の result を同一 artifact の disagreement として扱わないためです。
artifact mismatch は normalizer では拒否しません。
normalizer は result entry を保存し、comparison が deterministic に `disagreement` を返します。
`process_launched = false` で許可される `error.kind` は `policy_failure`、
`timeout`、`resource_exhausted` だけです。
launch 前 timeout は `error.kind = timeout`、`error.reason_code = launch_timeout`、
launch 前 resource exhaustion は `error.kind = resource_exhausted`、
`error.reason_code = launch_resource_exhausted` とします。
checker 起動後の timeout / resource exhaustion では `process_launched = true` にし、
`error.reason_code = checker_timeout` または `checker_resource_exhausted` とします。
それ以外の `process_launched = false` result は malformed result として `policy_failure` にします。

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
      "module",
      "certificate_hash",
      "checker_id",
      "checker_version",
      "status",
      "error.kind",
      "error.declaration",
      "error.core_path"
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
  source.normalized_result_hash is optional but recommended.

normalized_comparison:
  NormalizedCheckResult.comparison に対する disagreement / inconclusive summary。
  source.normalized_result_hash is required.
  source.result_hash must be omitted.
  source.request_hash must be omitted.
  source.run_artifact_hash must be omitted.
```

MVP の sidecar validator は次を検査します。

```text
- source.kind = machine_result の場合、source.result_hash が実在する MachineCheckResult の result_hash と一致する
- source.kind = machine_result の場合、source.request_hash と source.run_artifact_hash が同じ MachineCheckResult と一致する
- source.kind = machine_result かつ source.normalized_result_hash が存在する場合、その NormalizedCheckResult.results に同じ source.result_hash の entry が存在する
- source.kind = normalized_comparison の場合、source.normalized_result_hash が実在する NormalizedCheckResult の normalized_result_hash と一致する
- source.kind = normalized_comparison の場合、source.result_hash / source.request_hash / source.run_artifact_hash は存在しない
- input_policy.hash が policy file の canonical hash と一致する
- status が summarized / triaged / suggested_fix / suggested_challenge / inconclusive のいずれか
- source.kind = machine_result かつ source result が failed の場合、classification.checker_error_kind が source result の error.kind と一致する
- source.kind = machine_result かつ source result が checked の場合、classification.checker_error_kind は omit する
- source.kind = normalized_comparison の場合、classification.checker_error_kind は omit する
- sidecar が structured verdict field を持たない
- sidecar に certificate bytes / generated certificate bytes が含まれない
- sidecar に secret token や policy で禁止された source text が含まれない
```

checked result に対する sidecar は `status = summarized` のみ許可します。
checked result の sidecar は triage / fix suggestion ではなく、release audit summary 用です。
failed machine result、または `normalized_comparison` source の disagreement / inconclusive に対してだけ
`triaged`、`suggested_fix`、`suggested_challenge` を使えます。

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

`outcome_hint` は oracle ではありません。
テスト判定に使うのは、変異後 certificate に対する checker result だけです。
名前も `expected_checker_status` ではなく `outcome_hint.status` に固定します。

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
  "challenge_id": "pch_001",
  "manifest_hash": "sha256:...",
  "mutated_file_hash": "sha256:...",
  "mutated_claimed_certificate_hash": "sha256:...",
  "checker_results": [
    {
      "result_id": "mchkres_challenge_ref_001",
      "result_hash": "sha256:...",
      "checker_profile": "reference"
    }
  ],
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
`checker_results[*].result_hash` は required です。
`checker_results[*].result_id` は人間向け参照であり、監査時の同一性判定には使いません。
`normalized_result_hash` は comparison artifact が生成された場合だけ required です。
`policy_hash` と `artifact_hash` は replay がどの policy / artifact identity で行われたかを
result 単体から検証するために required です。

challenge manifest は checker input ではありません。
checker input は変異後の `.npcert` だけです。

MVP で作る challenge 種別：

```text
- flip one canonical encoding byte
- reorder declarations
- replace import export_hash
- remove one dependency entry
- change declaration body without changing declaration hash
- change declaration hash without changing body
- drop one axiom report entry
- add forbidden axiom
- alter universe constraint
- alter de Bruijn index
- replace Nat.zero Const with a noncanonical placeholder
- insert unsupported schema version
- truncate certificate section
```

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

```text
CI pass conditions:
  - required checker profiles all returned checked
  - normalized comparison is all_agree_checked
  - axiom policy passed
  - reproducibility check passed
  - required audit bundle was generated

AI sidecar conditions:
  - optional for PR mode
  - required only as explanatory artifact in nightly / release modes
  - never sufficient for pass
```

CI pass condition に使う補助 result は、MVP では同じ deterministic envelope を使います。

```json
{
  "schema": "npa.phase8.auxiliary_result.v1",
  "kind": "axiom_policy",
  "result_id": "aux_axiom_Std.Nat_001",
  "result_hash": "sha256:...",
  "policy_hash": "sha256:...",
  "artifact_hash": "sha256:...",
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

`status` は `passed` / `failed` / `inconclusive` のいずれかです。
`status = failed` または `status = inconclusive` では `error.kind` と
`error.reason_code` を required にします。
`status = passed` では `error` を omit します。
`diagnostics` は optional で、自然言語、stderr excerpt、human-facing hint を入れます。
`result_hash` は `result_id`、`result_hash`、`diagnostics` field を除いた canonical hash です。
`error` に自然言語を入れてはいけません。
人間向け説明は diagnostics または AI sidecar に分離します。

mode ごとの required artifacts：

```text
pr:
  - MachineCheckRequest
  - MachineCheckResult for required profiles
  - NormalizedCheckResult
  - comparison result
  - axiom policy result

nightly:
  - PR mode artifacts
  - external checker result
  - reproducibility result
  - challenge replay results
  - AI audit sidecar for failures and disagreements

release:
  - nightly mode artifacts
  - release audit bundle
  - checker binary identity manifest
  - import lock hash
  - AI audit sidecar with input_policy and prompt_hash

high-trust:
  - release mode artifacts
  - high-trust-reference checker result
  - import certificate_hash verification
  - retained raw result artifacts in append-only storage
```

AI sidecar が必須の mode でも、sidecar 生成失敗は「説明 artifact の不足」です。
checker failure を success に変えることも、checker success を failure に変えることもありません。

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
- raw MachineCheckResult files
- NormalizedCheckResult
- checker binary identity manifest
- import lock
- axiom policy result
- reproducibility result
- AI audit sidecar, required when release policy enables AI triage
- challenge coverage summary
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
- artifact.input_file_hash
- artifact.expected_certificate_hash
- MachineCheckResult.certificate_hash when present
- checker_id
- checker_build_hash
- checker_profile
- result_hash
- policy.hash
- policy.version
```

ここでの `MachineCheckResult.certificate_hash` は checker が再計算した canonical certificate hash です。
expected hash や file bytes hash と混同してはいけません。
`result_id` は再実行で変わり得るため、training identity には含めません。

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
npa-check run --request build/check-requests/Std.Nat.reference.json --json
npa-check run --request build/check-requests/Std.Nat.external.json --json
npa-check normalize-results --policy ci/phase8-pr-policy.json --request-store build/check-requests/manifest.json --selector-module Std.Nat --selector-request-hash sha256:... --json build/check-results/*.json
npa-check compare --policy ci/phase8-pr-policy.json --json build/normalized/Std.Nat.json
npa-check challenge generate --policy ci/phase8-nightly-policy.json --kind hash-mutation --from build/certs/Std/Nat.npcert
npa-check challenge replay --request build/check-requests/challenges/pch_001.reference.json --json
npa-check audit-sidecar validate build/audit/Std.Nat.ai.json
```

AI agent はこれらの command を提案または runner 経由で起動できます。
ただし `npa-check audit-sidecar validate` は sidecar schema の検査だけを行い、
証明の受理判定は行いません。

`npa-check run` の正本入力は常に `--request` で渡す `MachineCheckRequest` です。
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

`/machine/check/certificate` は `.npcert` だけを検査します。
request body は `MachineCheckRequest`、response body は `MachineCheckResult` です。
`/machine/check/normalize` は `MachineCheckResult` の list、
`RunnerPolicy` または resolvable policy hash、request store reference、
および artifact selector を受け取り、
`NormalizedCheckResult` または `NormalizeErrorResult` を返します。
`/machine/check/compare` は `NormalizedCheckResult` と `RunnerPolicy` または resolvable policy hash を受け取り、
comparison result を返します。
`/machine/check/challenge` は challenge generation request を受け取り、
`ChallengeManifest` と変異後 certificate を返します。
`/machine/check/audit-sidecar/validate` は `AiAuditSidecar` を検査し、
sidecar schema validation result だけを返します。

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
`NormalizeErrorResult.error.reason_code = request_store_manifest_hash_mismatch`
ではなく `request_store_manifest_invalid` として拒否します。

これらの API は checker source / tactic / AI trace を verification input として受け取りません。
source や tactic を渡す API は Phase 8 の trust boundary を壊すため追加しません。

---

# 19. Implementation plan

Phase 8 AI Profile の実装順序：

```text
1. Define RunnerPolicy schema and canonical hash
2. Define MachineCheckRequest / MachineCheckResult schema
3. Implement checker runner with checker binary allowlist
4. Store raw checker result before AI processing
5. Implement NormalizedCheckResult generator
6. Implement deterministic checker comparison
7. Implement AiAuditSidecar schema and validator
8. Add CI mode artifact requirements
9. Add adversarial challenge manifest and generator
10. Add challenge replay in nightly CI
11. Add release audit bundle with AI sidecar metadata
12. Add training data exporter based only on checker labels
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
- MachineCheckResult with wrong result_hash is rejected
- MachineCheckResult with wrong request_hash is rejected
- sidecar input_policy hash mismatch is rejected
- request certificate file_hash mismatch is rejected before checker launch
- import manifest_hash mismatch is rejected before checker launch
- malformed CheckerRawResult becomes checker_internal_error
- policy_failure uses reason_code and does not hash human text
- checked-result sidecar omits classification.checker_error_kind
- NormalizedCheckResult failed entry includes failure_key
- comparison artifact identity ignores request_hash
- network import resolution is rejected in Phase 8 runner
- `npa-check run` short form cannot override policy budget or checker path
- all_agree_failed requires matching failure_key, not only matching error.kind
- resource_exhausted comparison is inconclusive and fails CI
- same certificate checked twice produces same normalized result
- normalized_result_hash ignores nested results[*].result_id
- decode-failure challenge request uses deterministic expected_certificate_hash placeholder
- compare without resolvable RunnerPolicy is rejected
- normalize without request store entry for request_hash is rejected
- policy mismatch takes precedence over artifact disagreement
- AuxiliaryResult diagnostics do not affect result_hash
- machine_result sidecar source requires result_hash, request_hash, and run_artifact_hash
- checker_id mismatch records expected_value and actual_value
- NormalizeErrorResult is returned instead of partial NormalizedCheckResult
- request store reference manifest hash mismatch is rejected
- normalize selector module mismatch returns NormalizeErrorResult
- omitted normalize selector is rejected when first required profile has zero or multiple results
- request store manifest order violation or duplicate entry is rejected
- machine_result sidecar normalized_result_hash must contain source.result_hash
- post-launch timeout/resource exhaustion uses checker_timeout/checker_resource_exhausted
```

特に重要なのは、AI がどのような sidecar を出しても、
checker result と comparison result を上書きできないことです。

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
- checker runner が policy allowlist だけを使う
- raw checker result が AI 処理前に保存される
- NormalizedCheckResult が deterministic に生成される
- disagreement が常に failure になる
- AiAuditSidecar が verdict を持てない schema になっている
- AI summary が checker result hash または normalized comparison hash に紐づく
- challenge generator が outcome-hint reject corpus を作れる
- challenge result は checker result を oracle にしている
- CI が AI sidecar なしでも pass/fail を決められる
- release audit bundle に AI sidecar の入力方針と prompt hash が残る
```

---

# 23. 一文でまとめると

Phase 8 AI Profile は、**AI を independent checker の前後に置く監査補助として使い、
checker の代替にも trust boundary の一部にもしないための設計**です。
