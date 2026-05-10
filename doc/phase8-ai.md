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

Phase 8 AI Profile の出力は3層に分けます。

```text
1. MachineCheckResult
   checker が生成した正本。

2. NormalizedCheckResult
   複数 checker の結果を比較しやすくする正規化表現。
   verdict は checker result から機械的に写すだけ。

3. AiAuditSidecar
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

---

# 4. MachineCheckRequest

AI agent や CI bot は checker を直接自由に呼び出すのではなく、
policy で固定された runner に request を渡します。

```json
{
  "schema": "npa.phase8.machine_check_request.v1",
  "request_id": "mchkreq_001",
  "module": "Std.Nat",
  "certificate": {
    "kind": "path",
    "path": "build/certs/Std/Nat.npcert",
    "expected_certificate_hash": "sha256:..."
  },
  "imports": {
    "mode": "locked_store",
    "manifest": "build/certs/import-lock.json"
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

---

# 5. MachineCheckResult

checker が返す結果は、AI が読む前に保存します。
AI が結果本文を書き換えた場合は別 artifact として扱い、正本にはしません。

成功時：

```json
{
  "schema": "npa.phase8.machine_check_result.v1",
  "request_id": "mchkreq_001",
  "result_id": "mchkres_001",
  "checker": {
    "id": "npa-checker-ref",
    "version": "0.8.0",
    "build_hash": "sha256:...",
    "profile": "reference"
  },
  "status": "checked",
  "module": "Std.Nat",
  "certificate_hash": "sha256:...",
  "export_hash": "sha256:...",
  "axiom_report_hash": "sha256:...",
  "axioms_used": [],
  "declarations_checked": 128,
  "diagnostics": []
}
```

失敗時：

```json
{
  "schema": "npa.phase8.machine_check_result.v1",
  "request_id": "mchkreq_002",
  "result_id": "mchkres_002",
  "checker": {
    "id": "npa-checker-ref",
    "version": "0.8.0",
    "build_hash": "sha256:...",
    "profile": "reference"
  },
  "status": "failed",
  "module": "Std.Nat",
  "error": {
    "kind": "type_mismatch",
    "declaration": "Nat.add_zero",
    "core_path": ["declarations", 17, "body"],
    "expected_hash": "sha256:...",
    "actual_hash": "sha256:..."
  }
}
```

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
- checker_internal_error
- resource_exhausted
```

---

# 6. NormalizedCheckResult

複数 checker の出力は、実装言語やエラー表現が異なります。
AI Profile では比較のために正規化します。

```json
{
  "schema": "npa.phase8.normalized_check_result.v1",
  "artifact": {
    "module": "Std.Nat",
    "certificate_hash": "sha256:..."
  },
  "results": [
    {
      "checker_id": "npa-fast-kernel",
      "checker_build_hash": "sha256:...",
      "status": "checked",
      "export_hash": "sha256:...",
      "axiom_report_hash": "sha256:..."
    },
    {
      "checker_id": "npa-checker-ref",
      "checker_build_hash": "sha256:...",
      "status": "checked",
      "export_hash": "sha256:...",
      "axiom_report_hash": "sha256:..."
    },
    {
      "checker_id": "npa-checker-ext",
      "checker_build_hash": "sha256:...",
      "status": "checked",
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

正規化器は、AI ではなく deterministic code として実装します。
AI はこの結果を読んで説明を書くだけです。

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

---

# 7. AiAuditSidecar

AI が生成する artifact は sidecar です。
checker result と同じ directory に置いてもよいですが、hash chain には入れません。

```json
{
  "schema": "npa.phase8.ai_audit_sidecar.v1",
  "source_result": "mchkres_002",
  "source_result_hash": "sha256:...",
  "ai": {
    "agent": "npa-audit-assistant",
    "model": "example-model",
    "prompt_hash": "sha256:..."
  },
  "status": "triaged",
  "classification": {
    "category": "certificate_generator_bug",
    "confidence": "medium",
    "checker_error_kind": "declaration_hash_mismatch"
  },
  "summary": "The checker rejected Nat.add_zero because the declaration hash stored in the certificate does not match the canonical body hash recomputed by the checker.",
  "suggested_next_actions": [
    "Re-run certificate generation for Std.Nat with hash tracing enabled.",
    "Compare the canonical body bytes for declaration index 17."
  ]
}
```

sidecar の禁止事項：

```text
- checker result と同じ `status` enum を使う
- `checked` / `accepted` / `verified` と書く
- checker output を書き換えたように見える field 名を使う
- certificate hash を AI が再計算した値として主張する
- source / tactic が正しいので certificate も正しい、と主張する
```

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
  "base_certificate_hash": "sha256:...",
  "mutation": {
    "kind": "drop_axiom_report_entry",
    "target": "Nat.add_zero"
  },
  "expected_checker_status": "failed",
  "expected_error_kinds": [
    "axiom_report_mismatch",
    "certificate_hash_mismatch"
  ],
  "generated_by": {
    "kind": "ai",
    "prompt_hash": "sha256:..."
  }
}
```

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

AI が生成した challenge は、expected result も含めて信用しません。
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
- AI audit sidecar, if generated
- challenge coverage summary
```

AI sidecar には次を含めます。

```text
- source result hash
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
- certificate hash
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
- certificate_hash
- checker_id
- checker_build_hash
- checker_profile
- result_id
- result_hash
- policy version
```

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
- challenge manifest that claims expected success
- fake MachineCheckResult created by non-checker process
```

対策：

```text
- checker runner は binary allowlist を使う
- checker runner は network を使わない
- checker result は build hash と result hash を持つ
- AI は raw log ではなく structured result を優先して読む
- pretty text は command / prompt instruction として扱わない
- challenge expected result は oracle ではなく metadata として扱う
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
npa-check run --profile reference --json build/certs/Std/Nat.npcert
npa-check run --profile external --json build/certs/Std/Nat.npcert
npa-check normalize-results --json build/check-results/*.json
npa-check compare --json build/normalized/Std.Nat.json
npa-check challenge generate --kind hash-mutation --from build/certs/Std/Nat.npcert
npa-check audit-sidecar validate build/audit/Std.Nat.ai.json
```

AI agent はこれらの command を提案または runner 経由で起動できます。
ただし `npa-check audit-sidecar validate` は sidecar schema の検査だけを行い、
証明の受理判定は行いません。

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
source や tactic を渡す API は Phase 8 の trust boundary を壊すため追加しません。

---

# 19. Implementation plan

Phase 8 AI Profile の実装順序：

```text
1. Rename human checker design to doc/phase8-human.md
2. Define MachineCheckRequest / MachineCheckResult schema
3. Implement checker runner with checker binary allowlist
4. Store raw checker result before AI processing
5. Implement NormalizedCheckResult generator
6. Implement deterministic checker comparison
7. Implement AiAuditSidecar schema and validator
8. Add CI summary generation using structured checker results
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
- sidecar source_result_hash mismatch is rejected
- checker result normalization is deterministic
- checker disagreement always fails comparison
- source-only evidence cannot produce MachineCheckResult
- tactic-only evidence cannot produce MachineCheckResult
- noncanonical certificate challenge is rejected by checker
- forbidden axiom challenge is rejected by policy
- challenge expected success cannot override checker failure
- prompt injection in theorem name is treated as data
- checker binary outside allowlist is rejected
- network import resolution is rejected in Phase 8 runner
- same certificate checked twice produces same normalized result
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
- AI summary が checker result hash に紐づく
- challenge generator が expected reject corpus を作れる
- challenge result は checker result を oracle にしている
- CI が AI sidecar なしでも pass/fail を決められる
- release audit bundle に AI sidecar の入力方針と prompt hash が残る
```

---

# 23. 一文でまとめると

Phase 8 AI Profile は、**AI を independent checker の前後に置く監査補助として使い、
checker の代替にも trust boundary の一部にもしないための設計**です。
