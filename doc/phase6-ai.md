# Phase 6 AI Profile: Machine Standard Library

この文書は、NPA の **AI 向け Phase 6** の設計です。

`doc/phase6-human.md` は、人間が読み書きする標準ライブラリの構成、命名、定理セット、属性方針を扱います。
一方、AI 証明探索では、標準ライブラリを「便利なソース集」としてではなく、verified certificate に紐づいた
決定的な machine artifact として扱います。

Phase 6 AI Profile は、Phase 5 AI の Machine IDE/API と Phase 7 の AI 探索が使うための、
標準ライブラリ release manifest、import bundle、theorem / simp / rewrite metadata の wire contract を定義します。

---

# 1. 目的

Phase 6 AI Profile の目的は次です。

```text
- 標準ライブラリ certificate set を AI 探索器が決定的に import できるようにする
- theorem search / simp-lite / rw 用 metadata を certificate hash / export hash に bind し、
  metadata 自体は certificate hash の入力にしない
- 属性、ranking、prompt 表示、embedding を trusted payload に入れない
- Phase 5 AI session create に渡せる import_closure / imports / tactic_options recipe を提供する
- Phase 7 が premise retrieval と candidate generation を再現可能に行える artifact を定義する
```

最重要の原則はこれです。

```text
信頼しない:
  library source text
  notation / pretty statement
  attribute sidecar
  theorem ranking
  embedding vector
  usage statistics
  generated prompt text
  AI-generated proof hints

信頼する:
  Phase 2 canonical certificate bytes
  export_hash / certificate_hash / decl_interface_hash
  Phase 2 verifier output
  Phase 1 kernel check
  Phase 8 independent checker
```

標準ライブラリが「使える」ことと「信頼できる」ことを分けます。
AI が使う metadata は探索効率のための非信頼情報です。
証明として採用できるのは、最終的に generated proof certificate が kernel / certificate checker / independent checker に
通った後だけです。

---

# 2. Human Profile との差分

人間向け Phase 6 は、次のような内容を中心にします。

```text
- Std.Logic / Std.Nat / Std.List / Std.Algebra.Basic の定義
- 定理名
- どの定理を simp に入れるか
- どの theorem を後回しにするか
- 人間向け notation
```

AI 向け Phase 6 は、次を中心にします。

```text
- MachineStdLibraryRelease
- MachineStdModuleArtifact
- MachineStdImportBundle
- MachineStdTheoremIndex
- MachineStdSimpProfile
- MachineStdRewriteProfile
- MachineStdPromptMetadata
- artifact hash / validation order / deterministic ordering
```

AI Profile MVP では、library source text、tactic script、pretty statement、natural language description を
canonical artifact の正本にしません。
それらは response / prompt / audit view に含めてもよいですが、certificate hash、import closure、theorem identity、
simp rule identity には使いません。

---

# 3. 成果物

Phase 6 AI MVP の標準ライブラリ release は、次の artifact set で構成します。

```text
Std/
  Logic.npcert
  Nat.npcert
  List.npcert
  Algebra/Basic.npcert

Std.machine-release.json
Std.machine-import-bundles.json
Std.machine-theorem-index.json
Std.machine-simp-profiles.json
Std.machine-rewrite-profiles.json
Std.machine-axiom-report.json
Std.machine-prompt-metadata.json  optional
```

`npa.stdlib.mvp.v1` の certificate artifact path は次の table に固定します。
この table にない release module や別 path に置かれた同名 module は MVP release artifact として invalid です。
filesystem path は packaging locator であり、canonical bytes には入りません。

```text
Std.Logic          -> Std/Logic.npcert
Std.Nat            -> Std/Nat.npcert
Std.List           -> Std/List.npcert
Std.Algebra.Basic  -> Std/Algebra/Basic.npcert
```

MVP release module membership is exact.
`MachineStdLibraryRelease.modules` must contain exactly these four module names in `ModuleName` canonical order, and the package
locator must provide each module at the path shown above.
Missing modules, extra modules, duplicate modules, non-canonical module order, or a path mismatch for one of these fixed module names
are `InvalidStdLibraryRelease`.

The package locator is a validation input, not trusted payload.
For the MVP filesystem package, the validator receives one package root directory and resolves only the fixed POSIX relative paths
shown above.
Locator paths must use `/` separators, must not be absolute, and must not contain an empty component, `.`, `..`, `\`, a trailing
slash, or duplicate slashes.
If a local filesystem implementation follows symlinks, the resolved target must remain inside the package root; otherwise the
release is `InvalidStdLibraryRelease`.
Archive-based package readers must present the same normalized relative path table to the validator.
Case folding, Unicode normalization, platform path aliases, search paths, environment variables, and package-manager module
resolution are not part of the MVP locator rule.
The path table only decides which raw bytes are read for each fixed module name; it is never included in any Phase 6 canonical
bytes or hash.

The concrete MVP `ModuleName` canonical order is:

```text
Std.Nat
Std.List
Std.Logic
Std.Algebra.Basic
```

This order follows Phase 5 `Phase5Name canonical bytes`, not human dotted-name lexicographic order.
Any emitted `MachineStdLibraryRelease.modules` array using a different order is `InvalidStdLibraryRelease`.

`Std/*.npcert` files contain raw Phase 2 `ModuleCertBytes`.
They are not JSON and not lowercase hex text.
When a Phase 6 import bundle embeds the same certificate in a Phase 5 `VerifiedModuleCertificateRequest`, the raw file bytes are
lowercase-hex encoded into `certificate.bytes` with `certificate.encoding = "npa.certificate.canonical.v0.1.hex"`.
`certificate_bytes_hash` is always `sha256` of the raw Phase 2 certificate bytes before any hex encoding.
Hashing the hex text, path string, or JSON wrapper is invalid.

JSON は wire / storage 形式の一例です。
artifact identity は JSON byte string ではなく、この文書で定義する canonical bytes から計算します。
JSON object field order、whitespace、escape spelling、source file order、HashMap iteration order は hash 入力に使いません。
すべての JSON object は duplicate key を許しません。
unknown field は常に `InvalidStdArtifactShape` として拒否します。
Artifact-specific error categories are used only after the JSON object has the exact schema shape required by this document.

MVP JSON root shape は次に固定します。
top-level array は使いません。
各 root object に含まれる hash field は、その artifact の canonical bytes から再計算した値と一致しなければなりません。
`MachineStdLibraryRelease` is the only MVP root object that does not carry its own hash field.
Its `std_library_release_hash` is computed by validators, package indexes, and caches from the release canonical bytes, but is not a
field in `Std.machine-release.json`.
Sidecar root self-hash validation is separate from release-manifest hash binding.
Validators must first recompute each sidecar root hash from the parsed canonical artifact and compare it with that sidecar's own
hash field; only after the sidecar is valid do they compare the validated sidecar hash with the matching
`MachineStdLibraryRelease` manifest field.
Manifest hash equality must never mask a stale sidecar root hash field.

```text
Sidecar root own hash field mismatch:
  MachineStdImportBundleSet.import_bundles_hash:
    InvalidStdImportBundle
  MachineStdTheoremIndex.index_hash:
    InvalidStdTheoremIndex
  MachineStdSimpProfileSet.simp_profiles_hash:
    InvalidStdSimpProfile
  MachineStdRewriteProfileSet.rewrite_profiles_hash:
    InvalidStdRewriteProfile
  MachineStdAxiomReport.axiom_report_hash:
    InvalidStdAxiomPolicy
  MachineStdPromptMetadataSet.prompt_metadata_hash:
    InvalidStdPromptMetadata

Release manifest hash field mismatch after sidecar validation:
  any MachineStdLibraryRelease.*_hash field that differs from the validated sidecar hash:
    InvalidStdLibraryRelease
```

```text
Std.machine-release.json:
  MachineStdLibraryRelease

Std.machine-import-bundles.json:
  MachineStdImportBundleSet

Std.machine-theorem-index.json:
  MachineStdTheoremIndex

Std.machine-simp-profiles.json:
  MachineStdSimpProfileSet

Std.machine-rewrite-profiles.json:
  MachineStdRewriteProfileSet

Std.machine-axiom-report.json:
  MachineStdAxiomReport

Std.machine-prompt-metadata.json:
  MachineStdPromptMetadataSet
```

Set artifact root objects carry `library_profile_id` and reject duplicate item ids.
The item arrays are emitted in the canonical order defined by each set.

MVP の canonical source of truth は `*.npcert` です。
それ以外の machine artifact は certificate bytes と Phase 2 verifier output に対する sidecar です。

この文書の canonical bytes では、共通 primitive を次のように扱います。

```text
String:
  Phase 5 UTF-8 string primitive bytes
  In this Phase 6 document that means minimal unsigned LEB128 u64 byte length followed by the exact UTF-8 bytes after JSON decode.

ModuleName / FullyQualifiedName:
  Phase 5 Phase5Name canonical bytes

HashString:
  "sha256:" prefix を除いた 32-byte digest bytes

u64:
  minimal unsigned LEB128

Option<T>:
  0x00 for none
  0x01 followed by T canonical bytes for some

Vec<T>:
  length as minimal unsigned LEB128 u64, followed by item canonical bytes in specified order

Enum:
  one byte tag fixed by the enum's definition in this document
  wire JSON strings are not hash input unless a section explicitly says it reuses another phase's string bytes
```

All domain separator tags written as `tag "..."` in this document are encoded as `String` canonical bytes, not as raw ASCII
bytes and not as enum tags.

When this document refers to core expression canonical bytes, it means the Phase 2 `TermHashPayload(expr)` bytes from
`doc/phase2.md`, using the owner certificate context that produced the expression for `GlobalRef` encoding.
It does not mean Rust arena allocation bytes, `ExprId` / `TermId` table indices, pretty text, JSON, or Phase 5 `MachineExprView`
bytes.
The corresponding core expression hash is the Phase 2 `term_hash`:

```text
phase6_core_expr_hash(expr):
  sha256("NPA-TERM-0.1" || TermHashPayload(expr))
```

If the expression is a Phase 2 `ExportEntry.type`, use the verifier-provided `ExportEntry.type_hash`.
If the expression is a Phase 4 `ResolvedSimpRule` fragment, recompute the same Phase 2 `TermHashPayload` structurally from that
resolved core expression under the source theorem's certificate context.
For `ResolvedSimpRule.theorem_lhs`, `ResolvedSimpRule.theorem_rhs`, `ResolvedSimpRule.from_pattern`,
and `ResolvedSimpRule.to_pattern`, the expression is an open core expression under the full
`ResolvedSimpRule.rule_telescope`.
Its `BVar` indices are interpreted relative to that full telescope in Phase 4 order: `BVar 0` refers to the last term parameter in
the telescope, `BVar 1` to the previous one, and so on, following the core de Bruijn convention.
For `ResolvedRuleParam.ty`, the expression is hashed in the binder prefix context for that parameter, not under the full telescope.
If the parameter has position `i`, its type may refer only to parameters with positions `< i`; `BVar 0` refers to position `i - 1`,
`BVar 1` to position `i - 2`, and so on.
For `i = 0`, the type is hashed in an empty term-parameter context and any term `BVar` is invalid.
Validators must reject a `ResolvedRuleParam.ty` whose free `BVar` reaches the parameter itself or any later parameter.
Universe parameters are interpreted in `ResolvedSimpRule.universe_params` order.
The telescope is not prepended as synthetic `Pi` nodes before hashing these expression fragments or binder types; binder types are
represented separately in `MachineStdRuleTelescope`.
Implementations must not close these fragments by adding lambdas, lift binder types into the full telescope, reindex by display
names, or hash the whole theorem type as a substitute for the fragment hash.

JSON 上の `HashString` は Phase 5 `HashString` wire format をそのまま使います。
つまり、値は `"sha256:"` に続く 64 文字の lowercase hex digest でなければなりません。
`sha256:` prefix なしの bare hex、uppercase hex、別 algorithm prefix、base64、numeric JSON value は invalid です。
canonical bytes には prefix と hex text ではなく、decode 済みの 32-byte digest だけを入れます。

JSON 上の `u64` は Phase 5 unsigned integer token grammar をそのまま使います。
値は `0` または `[1-9][0-9]*` の JSON number token で、範囲は `0 <= value <= 2^64 - 1` です。
leading zero、`+` / `-` sign、fraction、exponent、string-encoded number は invalid です。

JSON field order、object key spelling order、pretty text、source span、filesystem path は canonical bytes に入りません。
All JSON fields shown in the Rust-like struct definitions in this document are required unless a section explicitly marks the
field optional.
`Option<T>` is encoded as JSON `null` for `None` and as the JSON encoding of `T` for `Some(T)`.
Omitting an `Option<T>` field is invalid.
When this document says "dictionary order" for identifier strings such as `bundle_id`, `profile_id`, `recipe_id`, or
`metadata_profile_id`, validators compare the decoded UTF-8 byte sequences lexicographically.
They do not use locale collation, Unicode normalization, JSON escape spelling, or the length-prefixed `String` canonical bytes.
This rule does not override `ModuleName`, `FullyQualifiedName`, `MachineStdGlobalRef`, or other places that explicitly sort by
canonical bytes.

MVP JSON strings for scalar enum fields are fixed as follows.
Tagged-object enums, such as `MachineStdGlobalRefView`, define their own JSON object shape in their section.
Any other spelling, casing, alias, numeric enum tag, or object wrapper is invalid.
Unknown scalar enum JSON strings are wire-shape errors and are reported as `InvalidStdArtifactShape` in Step 1.
After an enum value has been parsed successfully, semantic disagreements with verifier output or profile-derived contents are
reported by the owning artifact validation step.

```text
MachineStdTheoremKind:
  "theorem"  -> Theorem
  "axiom"    -> Axiom

MachineStdAttribute:
  "simp"   -> Simp
  "rw"     -> Rw
  "intro"  -> Intro
  "elim"   -> Elim
  "apply"  -> Apply
  "refl"   -> Refl
  "trans"  -> Trans
  "congr"  -> Congr

MachineTheoremMode:
  "exact"  -> exact
  "apply"  -> apply
  "rw"     -> rw
  "simp"   -> simp

RewriteDirection:
  "forward"   -> Forward
  "backward"  -> Backward

RewriteSafety:
  "simp_safe"              -> SimpSafe
  "rw_only"                -> RwOnly
  "unsafe_for_automation"  -> UnsafeForAutomation
```

---

# 4. Release Manifest

## 4.1 MachineStdLibraryRelease

標準ライブラリ release は、certificate set と machine sidecar の root manifest です。

```rust
struct MachineStdLibraryRelease {
    protocol_version: String,
    library_profile_id: String,
    core_spec_id: String,
    kernel_semantics_profile_id: String,
    modules: Vec<MachineStdModuleArtifact>,
    import_bundles_hash: HashString,
    theorem_index_hash: HashString,
    simp_profiles_hash: HashString,
    rewrite_profiles_hash: HashString,
    axiom_report_hash: HashString,
}
```

MVP values:

```text
protocol_version = "npa.stdlib-machine.v1"
library_profile_id = "npa.stdlib.mvp.v1"
core_spec_id = "core-spec-v0.1"
kernel_semantics_profile_id = "npa-kernel.phase1.v0.1"
```

All four manifest scalar fields above are fixed by the MVP release profile.
Any mismatch in `protocol_version`, `library_profile_id`, `core_spec_id`, or `kernel_semantics_profile_id` is
`InvalidStdLibraryRelease`, even if all manifest hashes are otherwise self-consistent.

`modules` は `ModuleName` canonical bytes の辞書順です。
request order、filesystem order、build tool output order は使いません。

## 4.2 MachineStdModuleArtifact

```rust
struct MachineStdModuleArtifact {
    module: ModuleName,
    expected_export_hash: HashString,
    expected_certificate_hash: HashString,
    certificate_encoding: String,
    certificate_bytes_hash: HashString,
    axiom_report_hash: HashString,
    public_export_count: u64,
    theorem_index_entry_count: u64,
    simp_rule_count: u64,
}
```

`certificate_encoding` は `"npa.certificate.canonical.v0.1.hex"` だけを許します。
This field describes the Phase 5 wire encoding used when the same raw `.npcert` bytes are embedded in import bundles; it does
not mean the on-disk `Std/*.npcert` file is hex text.
Any other `certificate_encoding` spelling is `InvalidStdLibraryRelease`, even if the on-disk certificate bytes and hashes are valid.
`certificate_bytes_hash` は raw Phase 2 certificate bytes の `sha256` です。
`expected_certificate_hash` は Phase 2 certificate hash であり、`certificate_bytes_hash` と同じとは限りません。
両方を混同してはいけません。
`public_export_count` は Phase 2 verifier output の public `ExportEntry` 総数です。
`theorem_index_entry_count` はその module の public `ExportKind::Theorem` / `ExportKind::Axiom` entry 数です。
`simp_rule_count` は全 `MachineStdSimpProfile.rules` を検証し、各 `SimpRuleRef` をその profile の
`required_import_bundle_id` で `MachineStdGlobalRef` に semantic resolution した後、その module の theorem entry に解決される
unique rule target 数です。
Count key は raw `SimpRuleKey` ではなく `(resolved MachineStdGlobalRef canonical bytes, direction)` です。
同じ resolved rule target が複数 profile に出ても 1 件として数えます。
実装は `SimpRuleRef` / `SimpRuleKey` だけから module を推測してはいけません。
These count fields are release module-artifact summary fields.
Count mismatches are reported as `InvalidStdLibraryRelease`, not as theorem-index or simp-profile errors.
The theorem index and simp profiles are still validated on their own contents before these summary counts are compared.

Manifest validation は各 module certificate を Phase 2 verifier で high-trust mode として検査し、
recomputed `export_hash` / `certificate_hash` / module-level Phase 2 `axiom_report_hash` が manifest と一致することを確認します。
一致しない場合は `InvalidStdLibraryRelease` です。

```text
MachineStdModuleArtifact canonical bytes:
  - tag "npa.phase6.std-module-artifact.v1"
  - module canonical bytes
  - expected_export_hash digest bytes
  - expected_certificate_hash digest bytes
  - certificate_encoding as String
  - certificate_bytes_hash digest bytes
  - axiom_report_hash digest bytes
  - public_export_count as u64
  - theorem_index_entry_count as u64
  - simp_rule_count as u64
```

## 4.3 Release Hash

```text
MachineStdLibraryRelease canonical bytes:
  - tag "npa.phase6.std-library-release.v1"
  - protocol_version
  - library_profile_id
  - core_spec_id
  - kernel_semantics_profile_id
  - modules in ModuleName canonical order:
      MachineStdModuleArtifact canonical bytes
  - import_bundles_hash
  - theorem_index_hash
  - simp_profiles_hash
  - rewrite_profiles_hash
  - axiom_report_hash

std_library_release_hash:
  sha256(MachineStdLibraryRelease canonical bytes)
```

`std_library_release_hash` は AI / Phase 7 cache key に使えます。
これは証明の trust root ではありません。
証明の trust root は certificate bytes と checker です。

Manifest-bound sidecars are exactly the sidecar root objects whose hashes are fields of
`MachineStdLibraryRelease`:
`MachineStdImportBundleSet`, `MachineStdTheoremIndex`, `MachineStdSimpProfileSet`,
`MachineStdRewriteProfileSet`, and `MachineStdAxiomReport`.
Optional prompt metadata, embeddings, usage statistics, ranking data, and other non-manifest sidecars are not manifest-bound and
must not affect `std_library_release_hash` unless a future release profile adds an explicit hash field to
`MachineStdLibraryRelease`.

---

# 5. Import Bundles

Phase 5 AI session create は、server が filesystem / network から import を補完しない設計です。
そのため Phase 6 AI は、client がそのまま `/machine/sessions` に渡せる import bundle を提供します。

## 5.1 MachineStdImportBundle

```rust
struct MachineStdImportBundle {
    bundle_id: String,
    root_imports: Vec<VerifiedImportRequest>,
    import_closure: Vec<VerifiedModuleCertificateRequest>,
    allow_axioms: Vec<MachineAxiomRefWire>,
    recommended_tactic_options: MachineStdTacticOptionsRecipe,
}

struct MachineStdImportBundleSet {
    library_profile_id: String,
    bundles: Vec<MachineStdImportBundle>,
    import_bundles_hash: HashString,
}
```

MVP bundle ids:

```text
std.logic.mvp
std.nat.mvp
std.list.mvp
std.algebra-basic.mvp
std.all.mvp
```

`MachineStdImportBundleSet.bundles` in the MVP profile must contain exactly these five `bundle_id` values.
Missing bundle ids, extra bundle ids, duplicate bundle ids, non-canonical `bundle_id` order, or an id that differs only by spelling
are `InvalidStdImportBundle`.
The concrete emitted `bundle_id` dictionary order is:

```text
std.algebra-basic.mvp
std.all.mvp
std.list.mvp
std.logic.mvp
std.nat.mvp
```

MVP bundle root import and closure memberships:

```text
std.logic.mvp:
  root_imports = {Std.Logic}
  import_closure = {Std.Logic}

std.nat.mvp:
  root_imports = {Std.Logic, Std.Nat}
  import_closure = {Std.Logic, Std.Nat}

std.list.mvp:
  root_imports = {Std.Logic, Std.List}
  import_closure = {Std.Logic, Std.Nat, Std.List}

std.algebra-basic.mvp:
  root_imports = {Std.Algebra.Basic, Std.Logic}
  import_closure = {Std.Logic, Std.Algebra.Basic}

std.all.mvp:
  root_imports = {Std.Algebra.Basic, Std.List, Std.Logic, Std.Nat}
  import_closure = {Std.Algebra.Basic, Std.List, Std.Logic, Std.Nat}
```

The memberships above are semantic sets and intentionally do not assert emitted JSON array order.
The emitted `root_imports` and `import_closure` arrays must be sorted by the actual
`(module, export_hash, certificate_hash)` canonical order of each request record.
`Std.Logic` is a direct root of `std.nat.mvp` and `std.list.mvp` so the recommended Eq family can resolve through Phase 5
direct-import option validation.
`Std.Nat` remains only a transitive closure module for `std.list.mvp`; therefore `std.list.simp` / `std.list.rw` may use
`Std.List` theorem sources and the `Std.Logic` Eq family, but must not include `Std.Nat` rewrite or simp rule sources.
`Core` is not emitted as a Phase 6 verified module certificate.
It is part of the kernel/core profile, not a standard-library import bundle artifact.
In this document, any phrase like "Std.Logic depends on Core" means a verifier-internal dependency on the kernel/core profile named
by `core_spec_id` and `kernel_semantics_profile_id`; it does not mean a Phase 2 `ImportEntry` named `Core`.
For the MVP Phase 2 certificate format, no `ImportEntry` is treated as `Core` / prelude.
Every entry in the certificate `Imports` vector is an ordinary module import and must resolve to a release module certificate in
the bundle closure.
The only dependency that may be skipped is a verifier-internal kernel/core prelude dependency that is typed separately from
Phase 2 `ImportEntry` and is validated against `core_spec_id` / `kernel_semantics_profile_id`.
`import_closure` is the minimal transitive closure over standard-library module imports only.
If a certificate has any `ImportEntry` that does not resolve to one of the release module artifacts, including an entry named
`Core` or intended as a prelude import, the release is `InvalidStdLibraryRelease`.
This is a release module certificate error, not an import bundle closure error; validators report it before validating individual
bundle membership.

MVP bundle to recipe mapping:

```text
std.logic.mvp          -> std.logic-basic
std.nat.mvp            -> std.nat-simp
std.list.mvp           -> std.list-simp
std.algebra-basic.mvp  -> std.logic-basic
std.all.mvp            -> std.all-simp
```

`recommended_tactic_options.recipe_id` must match this table exactly.
Changing this mapping is `InvalidStdImportBundle`, not merely a different `machine_std_import_bundle_hash`.

`root_imports` は Phase 5 `/machine/sessions.imports` に渡す direct import key list です。
`import_closure` は Phase 5 `/machine/sessions.import_closure` に渡す complete certificate payload set です。
`import_closure` は `root_imports` から certificate `ImportEntry` をたどって到達する最小 transitive closure と完全一致しなければなりません。
extra certificate、欠落 dependency、duplicate root import key、duplicate closure key は invalid bundle です。
The emitted `root_imports` and `import_closure` arrays must already be in the canonical order defined in section 5.2.
Validators must reject non-canonical order as `InvalidStdImportBundle`; they must not sort or deduplicate a malformed bundle before
computing `MachineStdImportBundle` canonical bytes.
Every `VerifiedModuleCertificateRequest` in `import_closure` must carry the same canonical certificate bytes as the
corresponding module artifact in `Std/*.npcert`.
The raw bytes hash must equal `MachineStdModuleArtifact.certificate_bytes_hash`, and the request
`expected_export_hash` / `expected_certificate_hash` must equal the same module artifact's
`expected_export_hash` / `expected_certificate_hash`.
An import bundle must not contain a byte-different re-encoding, alternate certificate, or stale copy for a release module.
After JSON shape validation has accepted the `certificate` wrapper fields, `certificate.encoding` must be exactly
`"npa.certificate.canonical.v0.1.hex"` and `certificate.bytes` must be lowercase even-length hex for the same raw bytes as the fixed
release module artifact.
Wrong encoding strings, malformed hex, decoded byte mismatch, stale `expected_export_hash` / `expected_certificate_hash`, or
decoded raw-byte hash mismatch with `MachineStdModuleArtifact.certificate_bytes_hash` are `InvalidStdImportBundle`.

`allow_axioms` は MVP 標準ライブラリでは必ず `[]` です。
将来 `Std.Classical` を追加する場合は別 bundle id にし、`allow_axioms` に入る axiom を明示します。
An MVP import bundle with a non-empty `allow_axioms` array is `InvalidStdImportBundle`.
`allow_axioms` entries use Phase 5 `MachineAxiomRefWire` JSON, but Phase 6 import bundles may contain only the
`kind = "imported"` variant.
`kind = "current_module"` or any `source_index`-based axiom coordinate is invalid in a Phase 6 import bundle because the bundle
is release-global and has no checked-current declaration context.
Every non-empty future `allow_axioms` entry must resolve, by Phase 5 axiom option validation, to a unique `AxiomDecl` in the
bundle's verified import closure.
This uses the Phase 5 rule literally: an imported axiom ref may point to a public or private `AxiomDecl` as long as
`module / export_hash / name / decl_interface_hash` resolves uniquely in the verified module.
Phase 6 does not add a public-export requirement for axiom refs.
`allow_axioms` arrays are emitted in Phase 5 `MachineAxiomRefWire` canonical order after resolving to the unique imported
axiom identity, and duplicate resolved identities are rejected.
Validators must reject non-canonical order, duplicate entries, invalid variants, malformed axiom refs, or unresolved future
entries as `InvalidStdImportBundle`; they must not sort, deduplicate, or repair a malformed bundle before hashing.
This explicit `allow_axioms` validation is distinct from interpreting a Phase 2 `GlobalRef::Imported` inside a certificate term;
the latter is still constrained by the Phase 2 `ExportBlock` rule.
constructive MVP bundle が classical axiom を transitively import してはいけません。
`recommended_tactic_options` must validate as Phase 5 `MachineTacticOptionsRequest` against exactly this bundle's
`root_imports` / `import_closure` and an empty checked-current-declaration list.
This Phase 5 validation uses the recipe payload after dropping `recipe_id`; `recipe_id` is Phase 6 metadata and is not a
Phase 5 `MachineTacticOptionsRequest` field.
Every `SimpRuleRef`, `EqFamilyRef`, and `NatFamilyRef` must resolve within the bundle's direct import scope using Phase 5 option
validation rules.
Unknown, stale, or ambiguous recipe references make the bundle `InvalidStdImportBundle`.
In the fixed release validation order, this semantic Phase 5 option validation is performed in Step 10 after simp profiles have
already been validated.
Step 5 checks bundle membership, closure, `allow_axioms`, recipe id mapping, and emitted recipe payload canonical shape only; it
must not decide unknown, stale, or ambiguous rule/family targets.

## 5.2 Bundle Order

`import_closure` と `root_imports` の response order は `(module, export_hash, certificate_hash)` canonical order です。
Phase 5 session create は request order に依存しませんが、Phase 6 artifact は diff と cache を安定させるため canonical order で出力します。

```text
MachineStdImportBundle canonical bytes:
  - tag "npa.phase6.std-import-bundle.v1"
  - bundle_id
  - root_imports in canonical order:
      module / expected_export_hash / expected_certificate_hash
  - import_closure in canonical order:
      module / expected_export_hash / expected_certificate_hash / certificate bytes hash
  - allow_axioms in MachineAxiomRefWire canonical order
  - recommended_tactic_options recipe canonical bytes

machine_std_import_bundle_hash:
  sha256(MachineStdImportBundle canonical bytes)
```

`machine_std_import_bundle_hash` is an internal derived digest, not a JSON field of `MachineStdImportBundle`.
Validators recompute it from the parsed bundle contents after bundle-shape validation and before computing
`MachineStdImportBundleSet` canonical bytes.
If an emitted bundle object contains a `machine_std_import_bundle_hash` field, that field is unknown and the artifact is
`InvalidStdArtifactShape`.
`MachineStdImportBundleSet` canonical bytes use the recomputed digest for each bundle, never a value copied from JSON.

The canonical bytes include `certificate bytes hash`, not the full certificate bytes, so the bundle hash is small.
The full wire artifact still includes certificate bytes.
Clients must not reconstruct certificate bytes from this hash.
The full wire payload is validated against the release module artifact table before the bundle hash is accepted; hash equality
alone is not treated as permission to substitute a different certificate payload.

`MachineStdLibraryRelease.import_bundles_hash` は、全 bundle を `bundle_id` 辞書順に並べた canonical list の hash です。
`MachineStdImportBundleSet.library_profile_id` は `MachineStdLibraryRelease.library_profile_id` と一致しなければなりません。
`bundles` は `bundle_id` 辞書順で、同じ `bundle_id` を重複して持ってはいけません。

```text
MachineStdImportBundleSet canonical bytes:
  - tag "npa.phase6.std-import-bundle-set.v1"
  - library_profile_id
  - bundles in bundle_id order:
      machine_std_import_bundle_hash digest bytes

import_bundles_hash:
  sha256(MachineStdImportBundleSet canonical bytes with import_bundles_hash omitted)
```

---

# 6. Tactic Options Recipes

Phase 6 AI does not execute tactics.
It provides deterministic recipes that a client can copy into Phase 5 AI `options.tactic_options`.

```rust
struct MachineStdTacticOptionsRecipe {
    recipe_id: String,
    kernel_check_profile: KernelCheckProfileId,
    simp_rules: Vec<SimpRuleRef>,
    eq_family: Option<EqFamilyRef>,
    nat_family: Option<NatFamilyRef>,
    max_simp_rewrite_steps: u64,
    max_open_goals: u64,
    max_metas: u64,
}
```

MVP emitted recommended recipe ids:

```text
std.logic-basic
std.nat-simp
std.list-simp
std.all-simp
```

`std.none` is reserved for future clients that want a no-recommendation recipe.
It is not emitted by any MVP import bundle and therefore does not contribute to `import_bundles_hash`.

Recipes are convenience artifacts.
Phase 5 must still validate every `SimpRuleRef`, `EqFamilyRef`, and `NatFamilyRef` against the session imports and checked current declarations.
If a recipe is stale, Phase 5 returns `InvalidMachineApiOptions`; it must not silently repair or downgrade the recipe.

MVP recommended limits:

```text
max_simp_rewrite_steps = 100
max_open_goals = 32
max_metas = 64
```

MVP standard family references:

```text
std.logic.eq-family:
  EqFamilyRef whose heads resolve in the direct import scope to public Std.Logic exports:
    eq_name = "Eq"
    refl_name = "Eq.refl"
    rec_name = "Eq.rec"
  each *_interface_hash is the matching public ExportEntry.decl_interface_hash
  the exports must be generated from one checked Std.Logic InductiveDecl:
    Eq      has ExportKind::Inductive
    Eq.refl has ExportKind::Constructor
    Eq.rec  has ExportKind::Recursor

std.nat.family:
  NatFamilyRef whose heads resolve in the direct import scope to public Std.Nat exports:
    nat_name = "Nat"
    zero_name = "Nat.zero"
    succ_name = "Nat.succ"
    rec_name = "Nat.rec"
  each *_interface_hash is the matching public ExportEntry.decl_interface_hash
  the exports must be generated from one checked Std.Nat InductiveDecl:
    Nat      has ExportKind::Inductive
    Nat.zero has ExportKind::Constructor
    Nat.succ has ExportKind::Constructor
    Nat.rec  has ExportKind::Recursor
```

For the MVP AI standard-library artifacts, `Eq`, `Eq.refl`, `Eq.rec`, `Nat`, `Nat.zero`, `Nat.succ`, and `Nat.rec` are
certificate-bound standard-library exports.
They are not modeled as Phase 6 builtin metadata.
`Eq` is introduced by the checked `Std.Logic` certificate as a public `ExportKind::Inductive` entry backed by a checked
`InductiveDecl`, with generated constructor and recursor exports.
`Nat` is introduced by the checked `Std.Nat` certificate as a public `ExportKind::Inductive` entry backed by a checked
`InductiveDecl`, with generated constructor and recursor exports.
Even if an implementation bootstraps the kernel with initial inductive support for `Eq` or `Nat`, the MVP standard-library
artifacts must expose and reference these names through the verified standard-library certificates, not through a special Phase 6
builtin reference.
The MVP recipes use `kernel_check_profile = "npa.kernel.v0.1.builtin-none"` and an explicit imported Eq family for this reason.
These family exports are release-shape requirements, not optional tactic hints.
Missing, private, stale, wrong-kind, or cross-parent `Eq` / `Nat` family exports reject the release as `InvalidStdLibraryRelease`.
`std.logic.eq-family` must resolve to the `Std.Logic` direct import by Phase 5 option head resolution, including the public generated
constructor / recursor exports.
The generated `Eq.refl` / `Eq.rec` exports are family heads; they are not theorem-index entries.
In the MVP AI profile, the public export name `Eq.refl` is reserved for the generated constructor of the `Eq` inductive.
No ordinary theorem may be exported with the same fully-qualified name.
Human-facing documentation and pretty-printers may display this generated constructor as `Eq.refl`, but the certificate export is the
generated constructor entry, not a separate theorem entry.
The label `std.logic.eq-family` is specification shorthand only.
JSON artifacts emit the actual Phase 5 `EqFamilyRef` object with the six `*_name` / `*_interface_hash` fields, never the label string.
`std.nat.family` is defined for clients that want `induction-nat`, but the MVP emitted recipes below do not enable induction and
therefore set `nat_family = null`.
If a future profile emits `std.nat.family`, it must likewise emit the actual Phase 5 `NatFamilyRef` object, not the label string.

MVP emitted recipe contents:

```text
std.logic-basic:
  kernel_check_profile = "npa.kernel.v0.1.builtin-none"
  simp_rules = rules from MachineStdSimpProfile "std.logic.simp"
  eq_family = std.logic.eq-family
  nat_family = null
  limits = MVP recommended limits

std.nat-simp:
  kernel_check_profile = "npa.kernel.v0.1.builtin-none"
  simp_rules = rules from MachineStdSimpProfile "std.nat.simp"
  eq_family = std.logic.eq-family
  nat_family = null
  limits = MVP recommended limits

std.list-simp:
  kernel_check_profile = "npa.kernel.v0.1.builtin-none"
  simp_rules = rules from MachineStdSimpProfile "std.list.simp"
  eq_family = std.logic.eq-family
  nat_family = null
  limits = MVP recommended limits

std.all-simp:
  kernel_check_profile = "npa.kernel.v0.1.builtin-none"
  simp_rules = rules from MachineStdSimpProfile "std.all.simp"
  eq_family = std.logic.eq-family
  nat_family = null
  limits = MVP recommended limits
```

The table above is normative for MVP import bundle validation.
For each bundle, `recommended_tactic_options` must match the recipe selected by the bundle-to-recipe mapping exactly:
`recipe_id`, `kernel_check_profile`, emitted `simp_rules`, `eq_family`, `nat_family`, and all numeric limits are checked.
A payload that would be valid Phase 5 `MachineTacticOptionsRequest` but differs from the MVP row is still
`InvalidStdImportBundle`.
The referenced simp profile must be present in the same release.
The recipe embeds the profile's canonicalized `rules` list, not the profile hash.
The release generator may sort/dedup an internal candidate rule list before writing the artifact, but the emitted recipe
`simp_rules` array itself must already be in Phase 4 `SimpRuleKey` canonical order and contain no duplicates.
If the emitted recipe rule array is non-canonical, contains duplicates, or differs from the referenced profile's emitted canonical
rules, the containing bundle is `InvalidStdImportBundle`.
Unlike Phase 5 request validation, the Phase 6 artifact validator must not sort or deduplicate emitted recipe `simp_rules` as a
repair step before hashing.
It first checks that the emitted JSON array is already in Phase 4 `SimpRuleKey` canonical order and contains no duplicates, and
then uses that exact array order for `MachineStdTacticOptionsRecipe` canonical bytes.
Phase 5 sort/dedup behavior is used only for the later semantic revalidation of the recipe payload after `recipe_id` has been
dropped; it is not a Phase 6 artifact normalization rule.

These are semantic tactic options, not per-run deterministic budget.
Phase 7 can override them by choosing a different Phase 5 session, but the resulting `session_root_hash` changes.

`simp_rules` canonical bytes use Phase 4 `SimpRuleKey canonical order`.
For artifact validation, duplicate count and input order are not silently normalized away; the JSON artifact must already match the
canonical rule array.
For generator-internal recipe identity before emission, duplicate count and input order may be normalized to the same canonical
candidate list.
Every emitted `SimpRuleRef.name` must satisfy the Phase 5 `MachineSurfaceRenderableName` rule before semantic resolution.
This is checked as `InvalidStdImportBundle` for recipe payloads and as `InvalidStdSimpProfile` for simp profile payloads.
MVP emitted recipes do not use `eq_family = null`.
They carry an explicit `std.logic.eq-family` object so simp/rw validation is bound to the verified `Std.Logic` certificate, not to
the kernel builtin Eq default.
`nat_family = null` is intentional for these recipes: they are simp/rw recipes and do not enable `induction-nat`.
`EqFamilyRef` / `NatFamilyRef` を `Some` にする場合、Phase 6 の `Option<T>` wrapper が `0x01` some tag を付けます。
The `T` payload is the Phase 4 `MachineTacticOptions` family fragment bytes below.
Do not include the Phase 4 `MachineTacticOptions` tag, a standalone family tag, JSON field names, or Phase 4 resolved-family bytes.
Every emitted `EqFamilyRef` / `NatFamilyRef` name must satisfy the Phase 5 `MachineSurfaceRenderableName` rule before semantic
family validation.

```text
EqFamilyRef canonical bytes:
  - eq_name as FullyQualifiedName canonical bytes
  - eq_interface_hash digest bytes
  - refl_name as FullyQualifiedName canonical bytes
  - refl_interface_hash digest bytes
  - rec_name as FullyQualifiedName canonical bytes
  - rec_interface_hash digest bytes

NatFamilyRef canonical bytes:
  - nat_name as FullyQualifiedName canonical bytes
  - nat_interface_hash digest bytes
  - zero_name as FullyQualifiedName canonical bytes
  - zero_interface_hash digest bytes
  - succ_name as FullyQualifiedName canonical bytes
  - succ_interface_hash digest bytes
  - rec_name as FullyQualifiedName canonical bytes
  - rec_interface_hash digest bytes
```

```text
MachineStdTacticOptionsRecipe canonical bytes:
  - tag "npa.phase6.std-tactic-options-recipe.v1"
  - recipe_id
  - kernel_check_profile canonical bytes as Phase 5 KernelCheckProfileId
  - simp_rules in Phase 4 SimpRuleKey canonical order
  - eq_family as Phase 6 Option<EqFamilyRef> using EqFamilyRef canonical bytes
  - nat_family as Phase 6 Option<NatFamilyRef> using NatFamilyRef canonical bytes
  - max_simp_rewrite_steps as u64
  - max_open_goals as u64
  - max_metas as u64
```

---

# 7. Machine Theorem Index

The Phase 6 AI theorem index is a certificate-bound premise catalog.
It is richer than Phase 5 AI MVP theorem search, but still non-trusted.
Phase 5 MVP may ignore this artifact and derive a minimal theorem index directly from direct imports.
Phase 7 can use this artifact for retrieval as long as every candidate is revalidated through Phase 5.

## 7.1 Entry Set

MVP index entries are public `ExportEntry` declarations whose kind is one of:

```text
ExportKind::Theorem
ExportKind::Axiom
```

`ExportKind::Def`, `ExportKind::Inductive`, `ExportKind::Constructor`, `ExportKind::Recursor`, and private dependencies are not
theorem index entries in MVP.
They may appear in `constants` / `head` metadata of theorem entries.

Every entry is bound by:

```text
module
name
export_hash
certificate_hash
decl_interface_hash
```

`name` is the Phase 2 `ExportEntry.name` itself.
Do not synthesize `module + "." + name`.
In the MVP theorem index, every theorem-index entry `global_ref.name` must also satisfy the Phase 5
`MachineSurfaceRenderableName` rule because Phase 7 may turn it into Phase 5 `exact` / `apply` / `rw` / `simp` candidate input.
This renderability requirement applies to public `ExportKind::Theorem` / `ExportKind::Axiom` entry names and to emitted `SimpRuleRef`,
`EqFamilyRef`, and `NatFamilyRef` names.
It does not apply to `MachineStdGlobalRefView` names that appear only inside `statement_head` / `constants`; those are
certificate-bound identity metadata and must satisfy `FullyQualifiedName` plus the normalization rules in section 7.3.
If a future profile emits Machine Surface statement text or candidate text from those views, that profile must add a separate
Phase 5 renderer preflight.

MVP theorem index is complete for the release modules.
For every module listed in `MachineStdLibraryRelease.modules`, the validator recomputes the set of public `ExportEntry`
items whose `ExportEntry.kind` is `ExportKind::Theorem` or `ExportKind::Axiom`.
`MachineStdTheoremIndex.entries` must contain exactly that set, no more and no less.
Missing public theorem/axiom entries, extra private entries, generated constructor/recursor entries, or stale entries are
`InvalidStdTheoremIndex`.
`MachineStdModuleArtifact.theorem_index_entry_count` must equal the recomputed per-module count; because this field belongs to the
release module artifact, its mismatch is classified as `InvalidStdLibraryRelease`.

## 7.2 MachineStdTheoremIndex

```rust
struct MachineStdTheoremIndex {
    index_profile_id: String,
    library_profile_id: String,
    entries: Vec<MachineStdTheoremEntry>,
    index_hash: HashString,
}
```

MVP values:

```text
index_profile_id = "npa.stdlib.theorem-index.mvp.v1"
library_profile_id = "npa.stdlib.mvp.v1"
```

`index_profile_id` must match the MVP value above.
A mismatch is `InvalidStdTheoremIndex`.
`library_profile_id` must match `MachineStdLibraryRelease.library_profile_id`; that sidecar/manifest mismatch is reported as
`InvalidStdLibraryRelease`.

`entries` は `MachineStdGlobalRef canonical order` で sort し、同じ
`module / name / export_hash / certificate_hash / decl_interface_hash` を重複して持ってはいけません。
Validators must reject non-canonical `entries` order or duplicate `global_ref` entries as `InvalidStdTheoremIndex`.
They must not sort or deduplicate malformed theorem-index entries before recomputing `index_hash`.

```text
MachineStdGlobalRef canonical order:
  module canonical bytes
  name canonical bytes
  export_hash digest bytes
  certificate_hash digest bytes
  decl_interface_hash digest bytes

MachineStdGlobalRef canonical bytes:
  - tag "npa.phase6.std-global-ref.v1"
  - module canonical bytes
  - name canonical bytes
  - export_hash digest bytes
  - certificate_hash digest bytes
  - decl_interface_hash digest bytes

MachineStdTheoremIndex canonical bytes:
  - tag "npa.phase6.std-theorem-index.v1"
  - index_profile_id
  - library_profile_id
  - entries in MachineStdGlobalRef canonical order:
      MachineStdTheoremEntry canonical bytes

theorem_index_hash:
  sha256(MachineStdTheoremIndex canonical bytes with index_hash omitted)
```

`MachineStdLibraryRelease.theorem_index_hash` はこの `theorem_index_hash` と完全一致しなければなりません。
theorem index 自体には `std_library_release_hash` を入れません。
release hash は `theorem_index_hash` を含むため、index 側に release hash を戻すと循環依存になります。
Phase 5 / Phase 7 の search response へ投影する場合は、`certificate_hash` を各 `global_ref` に重複して持たせず、
それを包む `session_root_hash` / `theorem_index_fingerprint` / `query_fingerprint` で direct import certificate を束縛してよいです。
その projection は session-local locator であり、この Phase 6 `MachineStdGlobalRef` の release artifact identity とは別物です。

## 7.3 MachineStdTheoremEntry

```rust
struct MachineStdTheoremEntry {
    global_ref: MachineStdGlobalRef,
    kind: MachineStdTheoremKind,
    universe_params: Vec<String>,
    statement_core_hash: HashString,
    statement_head: Option<MachineStdGlobalRefView>,
    constants: Vec<MachineStdGlobalRefView>,
    modes: Vec<MachineTheoremMode>,
    attributes: Vec<MachineStdAttribute>,
    rewrite_descriptors: Vec<MachineStdRewriteDescriptor>,
    axiom_dependencies: Vec<MachineStdAxiomRef>,
    proof_term_size: Option<u64>,
}

struct MachineStdGlobalRef {
    module: ModuleName,
    name: FullyQualifiedName,
    export_hash: HashString,
    certificate_hash: HashString,
    decl_interface_hash: HashString,
}

enum MachineStdTheoremKind {
    Theorem,
    Axiom,
}

enum MachineStdAttribute {
    Simp,
    Rw,
    Intro,
    Elim,
    Apply,
    Refl,
    Trans,
    Congr,
}

struct MachineStdAxiomRef {
    module: ModuleName,
    name: FullyQualifiedName,
    export_hash: HashString,
    decl_interface_hash: HashString,
}

enum MachineStdGlobalRefView {
    Decl {
        module: ModuleName,
        name: FullyQualifiedName,
        export_hash: HashString,
        certificate_hash: HashString,
        decl_interface_hash: HashString,
        public_export: bool,
    },
    Generated {
        module: ModuleName,
        parent_name: FullyQualifiedName,
        name: FullyQualifiedName,
        export_hash: HashString,
        certificate_hash: HashString,
        parent_decl_interface_hash: HashString,
        decl_interface_hash: HashString,
        public_export: bool,
    },
}
```

`statement_core_hash`, `statement_head`, `constants`, and `axiom_dependencies` are derived from Phase 2 verifier output.
`kind` is derived from the same public Phase 2 `ExportEntry.kind` that admits the theorem-index entry:
`ExportKind::Theorem` maps to `MachineStdTheoremKind::Theorem`, and `ExportKind::Axiom` maps to
`MachineStdTheoremKind::Axiom`.
Any other export kind, or a sidecar `kind` that disagrees with the public `ExportEntry`, is `InvalidStdTheoremIndex`.
`statement_core_hash` is the Phase 2 `ExportEntry.type_hash` for the theorem/axiom declaration.
It is not a hash of pretty text, Machine Surface text, or `MachineExprView`.
If an implementation recomputes it, it must use the Phase 2 declaration interface/type hash rule that produced `ExportEntry.type_hash`.
`universe_params` are Phase 5 `MachineUniverseParamName` strings decoded from Phase 2 `ExportEntry.universe_params` in ExportEntry order.
They must not be sorted, renamed, or deduplicated.
If the verifier-derived public `ExportEntry.universe_params` for a theorem-index-visible export contains a duplicate decoded name or
a name that is not a valid `MachineUniverseParamName`, the release is not compatible with the MVP Phase 6 AI profile and validation
fails as `InvalidStdLibraryRelease` in Step 3.
Given valid verifier output, an emitted theorem-index sidecar whose `universe_params` are invalid, reordered, missing, extra, or
duplicated is `InvalidStdTheoremIndex`.
For every theorem entry, `kind`, `universe_params`, `statement_core_hash`, `statement_head`, `constants`, and
`axiom_dependencies` must equal the values derived from the matching public `ExportEntry` and verifier output.
Any sidecar mismatch in these certificate-derived fields is `InvalidStdTheoremIndex`.
`axiom_dependencies` is the exact sorted/dedup projection of the same public `ExportEntry.axiom_dependencies` field to
`MachineStdAxiomRef`.
It is not derived from prompt metadata, theorem attributes, source text, proof pretty-printing, or
`AxiomReport.per_declaration`.
If any verifier-derived `ExportEntry.axiom_dependencies` item cannot be projected to `MachineStdAxiomRef` by the section 7.3
rules, validation fails as `InvalidStdAxiomPolicy` in Step 4 before theorem-index sidecar comparison.
Given successful projection, a theorem-index sidecar whose `axiom_dependencies` differ from that projected list is
`InvalidStdTheoremIndex` in Step 6.
If the Phase 2 verifier exposes both `ExportEntry.axiom_dependencies` and a corresponding per-declaration transitive axiom
report, an implementation may cross-check them, but the theorem index field is sourced from `ExportEntry.axiom_dependencies`.
`attributes`, `rewrite_descriptors`, and `proof_term_size` are non-trusted metadata.
MVP must set `proof_term_size = None` for every entry.
Any non-null `proof_term_size` in the MVP theorem index is `InvalidStdTheoremIndex`.
Non-MVP profiles may add deterministic proof-size metrics, but they must use a new theorem index profile id.

`MachineStdTheoremKind` canonical order is `Theorem`, then `Axiom`.
`MachineStdAttribute` canonical order is `Simp`, `Rw`, `Intro`, `Elim`, `Apply`, `Refl`, `Trans`, `Congr`.
`MachineTheoremMode` canonical order is the Phase 5 order `exact`, `apply`, `rw`, `simp`.
The enum bytes are fixed as follows:

```text
MachineStdTheoremKind:
  0x00 Theorem
  0x01 Axiom

MachineStdAttribute:
  0x00 Simp
  0x01 Rw
  0x02 Intro
  0x03 Elim
  0x04 Apply
  0x05 Refl
  0x06 Trans
  0x07 Congr

MachineTheoremMode:
  0x00 exact
  0x01 apply
  0x02 rw
  0x03 simp
```

`modes` and `attributes` are semantic sets.
The emitted JSON arrays must be sorted in the canonical order above and must not contain duplicates.
Non-canonical order, duplicate mode, or duplicate attribute is `InvalidStdTheoremIndex`.
The same rule applies to `constants`, `rewrite_descriptors`, and `axiom_dependencies`: the artifact must contain the derived
canonical sorted/deduplicated list, not an input-order list.
Validators must preserve the emitted arrays while validating this condition.
They must not sort or deduplicate malformed theorem-entry arrays as a repair step before recomputing `index_hash`.
For fields whose final expected contents depend on validated profiles, such as `modes`, `attributes`, and `rewrite_descriptors`,
validators may defer the content equality check to Step 11, but duplicate and non-canonical emitted order are still theorem-index
shape errors.

```text
MachineStdTheoremEntry canonical bytes:
  - global_ref canonical bytes
  - kind enum byte
  - universe_params in certificate order
  - statement_core_hash digest bytes
  - statement_head option as MachineStdGlobalRefView canonical bytes
  - constants in MachineStdGlobalRefView canonical order
  - modes in MachineTheoremMode canonical order
  - attributes in MachineStdAttribute canonical order
  - rewrite_descriptors in MachineStdRewriteDescriptor canonical order
  - axiom_dependencies in MachineStdAxiomRef canonical order
  - proof_term_size option
```

`statement_head` と `constants` は release artifact 用の `MachineStdGlobalRefView` で表します。
Phase 5 `MachineGlobalRefView` は session direct import scope や `tactic_head_visible` を含むため、そのまま使いません。
Phase 6 では owner module の certificate verifier output と release manifest の module table だけを owner context にします。
MVP release certificates must not expose theorem-index-visible types that require a builtin/core global-reference variant outside
`MachineStdGlobalRefView`.
`Eq` and `Nat` references in standard-library theorem types must be ordinary release declarations or generated declarations reachable
through `Std.Logic` / `Std.Nat` certificate exports.
If extraction reaches a global ref that cannot be normalized to `Decl` or `Generated` by the release module table and verified
certificate closure, validation fails with `InvalidStdTheoremIndex`.
`statement_head` and `constants` are extracted from the canonical Phase 2 `ExportEntry.type` expression only.
The theorem proof body, declaration body, pretty statement, Machine Surface rendering, and source text are not inspected.
No WHNF, beta/delta/iota/zeta reduction, conversion, unification, or tactic execution is allowed during extraction.

Extraction rules:

```text
statement_head:
  - take ExportEntry.type
  - peel leading syntactic Pi nodes from the outer spine
  - inspect the resulting conclusion expression
  - peel zero or more syntactic App nodes by repeatedly moving to the function position
  - if the reached head node is a global ref, normalize it to MachineStdGlobalRefView
  - otherwise statement_head = None

constants:
  - syntactically traverse the whole ExportEntry.type expression
  - include global refs appearing in binder domains, binder codomains, let types, let values, let bodies, and the conclusion
  - normalize every global ref to MachineStdGlobalRefView
  - sort by MachineStdGlobalRefView canonical order and dedup exact canonical-byte duplicates
```

`GlobalRef::Imported` は owner certificate の import table から imported module の
`(module, export_hash, certificate_hash)` を解決し、その imported module の verifier output で public
`ExportEntry.name / decl_interface_hash` を引きます。
If the referenced imported entry cannot be found as a public `ExportEntry` with the same `name` and `decl_interface_hash` in the
imported module whose artifact hashes match the `ImportEntry`, normalization fails with `InvalidStdTheoremIndex`.
Imported refs branch by that public `ExportEntry.kind` before constructing `MachineStdGlobalRefView`:
`ExportKind::Constructor` / `ExportKind::Recursor` normalizes to `Generated`; every other importable export kind normalizes to
`Decl`.
For imported refs normalized to `Decl`, `Decl.public_export` is always true after successful normalization.
`Decl.module`, `Decl.export_hash`, and `Decl.certificate_hash` are the imported module artifact values, and
`Decl.name` / `Decl.decl_interface_hash` are the public `ExportEntry` values.
`GlobalRef::Local` は owner module の verifier output で引き、`public_export` はその declaration が owner module の
public `ExportEntry` に出る場合だけ true です。
For `GlobalRef::Local`, `Decl.module`, `Decl.export_hash`, and `Decl.certificate_hash` are the owner module's
`module`, `expected_export_hash`, and `expected_certificate_hash` from `MachineStdLibraryRelease.modules`.
`Decl.name` and `Decl.decl_interface_hash` are taken from the owner verifier declaration table for the referenced local
declaration index, not from pretty text or source metadata.
If the local declaration index is out of range, lacks a deterministic fully-qualified name, or lacks a verifier-derived
`decl_interface_hash`, normalization fails with `InvalidStdTheoremIndex`.
If the same `name / decl_interface_hash` appears in the owner module public `ExportEntry` set, `public_export = true`; otherwise
the declaration is treated as a private dependency and `public_export = false`.
constructor / recursor のような generated declaration は `Generated` に正規化します。
For a generated ref, `Generated.module`, `Generated.export_hash`, and `Generated.certificate_hash` are the owner module artifact
values for `GlobalRef::LocalGenerated`, or the imported module artifact values for an imported constructor/recursor `ExportEntry`.
`Generated.name` and `Generated.decl_interface_hash` are taken from the verifier-reconstructed generated constructor/recursor
interface, not from pretty text or source metadata.
For an imported constructor/recursor `ExportEntry`, these values must also match the public `ExportEntry.name` and
`ExportEntry.decl_interface_hash`.
If the generated interface lacks a deterministic fully-qualified name or verifier-derived `decl_interface_hash`, normalization fails
with `InvalidStdTheoremIndex`.
`Generated.parent_name` and `Generated.parent_decl_interface_hash` are derived from the unique checked `InductiveDecl` that generated
the constructor or recursor.
For `GlobalRef::LocalGenerated`, this is the referenced local inductive declaration.
For an imported generated `ExportEntry`, the validator must find the unique inductive export in the imported module verifier output
whose generated constructor/recursor interface has the same generated `name` and `decl_interface_hash`.
If no such parent exists, or more than one parent matches, normalization fails with `InvalidStdTheoremIndex`.
`Generated.public_export` is true iff the owner module verifier output contains a public `ExportEntry` for that generated
constructor/recursor with the same `name` and `decl_interface_hash`, and the module artifact hashes match
`Generated.export_hash` / `Generated.certificate_hash`.
For `GlobalRef::LocalGenerated`, check the owner module public export block; if the generated artifact is found only as an internal
reconstructed declaration and has no matching public export, `public_export` is false.
For an imported generated ref, normalization has already reached a public constructor/recursor `ExportEntry` through the owner
certificate `ImportEntry`, so `Generated.public_export` must be true.
An imported generated artifact that is not a public `ExportEntry` is not a valid `GlobalRef::Imported` target in Phase 6 and fails
normalization instead of being emitted with `public_export = false`.
The public `ExportEntry.kind` must be `ExportKind::Constructor` for a constructor and `ExportKind::Recursor` for a recursor.
If the artifact claims `public_export = true` without the matching public export entry, validation fails with
`InvalidStdTheoremIndex`.
lookup が一意でない、または owner module certificate から `ImportEntry` をたどって検証できる certificate closure 内に
存在しない場合は `InvalidStdTheoremIndex` です。

```text
MachineStdGlobalRefView canonical bytes:
  - tag "npa.phase6.std-global-ref-view.v1"
  - variant tag:
      0x00 Decl
      0x01 Generated
  - Decl:
      module canonical bytes
      name canonical bytes
      export_hash digest bytes
      certificate_hash digest bytes
      decl_interface_hash digest bytes
      public_export as 0x00/0x01
  - Generated:
      module canonical bytes
      parent_name canonical bytes
      name canonical bytes
      export_hash digest bytes
      certificate_hash digest bytes
      parent_decl_interface_hash digest bytes
      decl_interface_hash digest bytes
      public_export as 0x00/0x01
```

`MachineStdGlobalRefView canonical order` is lexicographic order of these canonical bytes.
`MachineStdGlobalRefView` JSON is a tagged object.
The `kind` discriminator is required, and fields from the other variant are forbidden.
Unknown fields, missing fields, `null` for a non-optional field, non-boolean `public_export`, or any other discriminator string is
`InvalidStdArtifactShape`.

```text
MachineStdGlobalRefView JSON:
  Decl:
    {
      "kind": "decl",
      "module": ModuleName,
      "name": FullyQualifiedName,
      "export_hash": HashString,
      "certificate_hash": HashString,
      "decl_interface_hash": HashString,
      "public_export": true | false
    }

  Generated:
    {
      "kind": "generated",
      "module": ModuleName,
      "parent_name": FullyQualifiedName,
      "name": FullyQualifiedName,
      "export_hash": HashString,
      "certificate_hash": HashString,
      "parent_decl_interface_hash": HashString,
      "decl_interface_hash": HashString,
      "public_export": true | false
    }
```
The JSON `kind` string is not hash input; canonical bytes use the `0x00` / `0x01` variant tags above.

`MachineStdAxiomRef` は release-global な axiom ref です。
Phase 5 `MachineAxiomRefWire::Imported` と同じ payload だけを持ち、`CurrentModule` / `source_index` 座標を持ちません。
Phase 2 verifier output で module-local axiom が `GlobalRef::Local` として現れる場合でも、Phase 6 artifact では
owner module の `module / export_hash` と axiom declaration の `name / decl_interface_hash` に正規化します。
Imported axiom refs are not projected to the owner module.
Projection from a Phase 2 `AxiomRef` in an owner module verifier context is fixed as follows:

```text
GlobalRef::Local(decl_index):
  - target must resolve uniquely in the owner module verifier output
  - target declaration kind must be AxiomDecl
  - MachineStdAxiomRef.module = owner module
  - MachineStdAxiomRef.export_hash = owner module expected_export_hash
  - name / decl_interface_hash come from the owner axiom declaration

GlobalRef::Imported(import_index, name, decl_interface_hash):
  - import_index must resolve through the owner certificate ImportEntry table to a release module artifact
  - target must resolve uniquely as a public ExportEntry in that imported module's ExportBlock
  - target ExportEntry.kind must be ExportKind::Axiom with matching name / decl_interface_hash
  - MachineStdAxiomRef.module = imported module
  - MachineStdAxiomRef.export_hash = imported module expected_export_hash
  - name / decl_interface_hash come from the imported public axiom export

GlobalRef::LocalGenerated:
  - invalid as an axiom ref
```

The `GlobalRef::Imported` branch above follows the Phase 2 rule that imported certificate terms can only name exported declarations.
Private axiom dependencies from an imported module are represented differently: they are projected while validating that imported
module's own `AxiomReport`, where the private axiom appears as `GlobalRef::Local(decl_index)` in that module's verifier context.
The resulting `MachineStdAxiomRef` uses the imported module's `module / expected_export_hash` and the private axiom declaration's
`name / decl_interface_hash`, and transitive axiom union then carries that release-global ref forward.
Validators must not reinterpret an owner-module `GlobalRef::Imported` as a private declaration-table lookup.

If any lookup is missing, stale, ambiguous, wrong-kind, or outside the verified release closure, projection fails with
`InvalidStdAxiomPolicy`.
`MachineStdAxiomRef canonical bytes` intentionally reuse Phase 5 `MachineAxiomRefWire canonical bytes` for the `Imported`
variant.
The byte sequence includes the Phase 5 tag `"npa.phase5.axiom-ref-wire.v1"`, the imported variant tag `0x00`, and then the
imported payload fields `module`, `name`, `export_hash`, and `decl_interface_hash` in the Phase 5 order.
Implementations must not define a Phase 6-specific four-field canonical byte encoding for this object.
`MachineStdAxiomRef canonical order` is lexicographic order of these canonical bytes.
Phase 5 request へ渡す必要がある場合だけ、同じ payload から `MachineAxiomRefWire { kind: "imported", ... }` を作ります。
`MachineStdAxiomRef` JSON in Phase 6 artifacts is not the Phase 5 `MachineAxiomRefWire` JSON object.
It has no `kind`, `certificate_hash`, or `source_index` field.
Those fields are forbidden in Phase 6 `MachineStdAxiomRef` artifacts.

```text
MachineStdAxiomRef JSON:
  {
    "module": ModuleName,
    "name": FullyQualifiedName,
    "export_hash": HashString,
    "decl_interface_hash": HashString
  }
```
When a client copies a Phase 6 axiom ref into a Phase 5 `allow_axioms` request, it must add `kind = "imported"` and use the
Phase 5 `MachineAxiomRefWire` field set.

## 7.4 Attributes

MVP recognized attribute enum values:

```text
simp
rw
intro
elim
apply
refl
trans
congr
```

MVP does not emit a separate attribute sidecar file.
`MachineStdTheoremEntry.attributes` is a derived field inside `Std.machine-theorem-index.json`.

MVP attribute derivation:

```text
Simp:
  present exactly when `simp` mode is present

Rw:
  present exactly when `rw` mode is present

Apply:
  present exactly when `apply` mode is present

Intro / Elim / Refl / Trans / Congr:
  not emitted by `npa.stdlib.theorem-index.mvp.v1`
```

If `Simp`, `Rw`, or `Apply` disagrees with the derived mode, or if `Intro` / `Elim` / `Refl` / `Trans` / `Congr` appears in
the MVP theorem index, validation fails with `InvalidStdTheoremIndex`.
Because `rw` and `simp` modes depend on validated profile membership, validators must split this check:
early theorem-index validation checks only attribute JSON shape, canonical order, duplicates, and MVP-forbidden attribute values;
the `Simp` / `Rw` / `Apply` equality with derived modes is checked after rewrite and simp profiles have been validated.

Future profiles may add a hash-bound `MachineStdAttributeSet` sidecar.
Such a profile must use a new theorem index profile id and must bind every sidecar entry to an existing theorem entry by exact
`module / name / export_hash / certificate_hash / decl_interface_hash`.
A sidecar entry that names a missing theorem, a stale hash, a private declaration, or a non-theorem public export must be
`InvalidStdTheoremIndex`.

Attributes do not make a theorem trusted.
They only affect retrieval and recipe generation.

## 7.5 Modes

MVP `modes` use the same names as Phase 5 theorem search:

```text
exact
apply
rw
simp
```

Derivation:

```text
exact:
  every theorem / axiom entry

apply:
  statement type has at least one leading syntactic Pi

rw:
  rewrite_descriptors is non-empty

simp:
  entry is the source of at least one rule in any validated MachineStdSimpProfile
```

Mode derivation must not use natural language statement text or theorem names alone.
`MachineStdTheoremIndex` is a release-wide artifact, not a bundle-specific or profile-specific index.
`rw` and `simp` modes are derived from the union of all validated rewrite/simp profiles in the same release.
If no validated profile marks an entry usable for `rw` or `simp`, that mode is omitted for that entry.
If a rewrite profile marks a theorem as `rw` but the certificate-derived statement is not an equality theorem under that profile's Eq selection,
the rewrite descriptor is invalid.
For MVP std profiles, `eq_family = std.logic.eq-family` and
`kernel_check_profile = "npa.kernel.v0.1.builtin-none"` select the imported `Std.Logic` Eq family.
The release-wide `rw` / `simp` mode derivation must not use the Phase 4 builtin Eq default.
This is a Phase 6 release-artifact mode.
Phase 5 MVP may still omit `rw` mode from its session-local theorem index when it cannot resolve a public imported Eq head, as described in
`doc/phase5-ai.md`.

---

# 8. Rewrite Metadata

Rewrite metadata is useful for AI search, but it is not trusted.
The trusted part is the theorem statement and proof certificate.

## 8.1 MachineStdRewriteDescriptor

```rust
struct MachineStdRewriteDescriptor {
    source: MachineStdGlobalRef,
    direction: RewriteDirection,
    safety: RewriteSafety,
    lhs_core_hash: HashString,
    rhs_core_hash: HashString,
    rule_telescope_hash: HashString,
}

enum RewriteDirection {
    Forward,
    Backward,
}

enum RewriteSafety {
    SimpSafe,
    RwOnly,
    UnsafeForAutomation,
}
```

`lhs_core_hash`, `rhs_core_hash`, and `rule_telescope_hash` must be derived from the `ResolvedSimpRule` produced by the same
Phase 4 simp / rw rule validation.
Phase 6 does not reuse Phase 4's SimpRegistry `telescope hash`; it computes the Phase 6-specific
`MachineStdRuleTelescope` hash below from the validated `ResolvedSimpRule`.
Pretty `lhs` / `rhs` strings may be emitted in debug views, but they are not descriptor identity.
`RewriteDirection` canonical order is `Forward`, then `Backward`.
`RewriteSafety` canonical order is `SimpSafe`, `RwOnly`, then `UnsafeForAutomation`.
The enum bytes are fixed as follows:

```text
RewriteDirection:
  0x00 Forward
  0x01 Backward

RewriteSafety:
  0x00 SimpSafe
  0x01 RwOnly
  0x02 UnsafeForAutomation
```

Descriptor hashes are fixed as follows:

```text
lhs_core_hash:
  phase6_core_expr_hash(ResolvedSimpRule.theorem_lhs)

rhs_core_hash:
  phase6_core_expr_hash(ResolvedSimpRule.theorem_rhs)

MachineStdRuleTelescope canonical bytes:
  - tag "npa.phase6.std-rule-telescope.v1"
  - universe_params in ResolvedSimpRule.universe_params order:
      MachineUniverseParamName as String
  - rule_telescope in ResolvedSimpRule.rule_telescope order:
      param position as minimal unsigned LEB128 u64
      param ty hash = phase6_core_expr_hash(ResolvedRuleParam.ty)

rule_telescope_hash:
  sha256(MachineStdRuleTelescope canonical bytes)
```

`lhs_core_hash` and `rhs_core_hash` are the theorem's original equality sides.
They are not direction-adjusted `from_pattern` / `to_pattern`.
`direction` is represented only by the descriptor's `direction` field.
`ResolvedRuleParam.name` is display/debug metadata and is not included in `rule_telescope_hash`.
`param position` is the zero-based index of the parameter in the Phase 4 validated
`ResolvedSimpRule.rule_telescope` vector.
The first `ResolvedRuleParam` has position `0`, positions are contiguous, and universe parameters do not consume positions in this
term-parameter index space.
Every string in `ResolvedSimpRule.universe_params` must satisfy the Phase 5 `MachineUniverseParamName` rule before it is encoded
into `MachineStdRuleTelescope` canonical bytes.
Invalid universe parameter names in a rewrite descriptor are `InvalidStdRewriteProfile`.

```text
MachineStdRewriteDescriptor canonical bytes:
  - source MachineStdGlobalRef canonical bytes
  - direction enum byte
  - safety enum byte
  - lhs_core_hash digest bytes
  - rhs_core_hash digest bytes
  - rule_telescope_hash digest bytes
```

MVP `SimpSafe` rules are exactly the following small terminating rules from `doc/phase6-human.md`:

```text
Nat.add_zero
Nat.add_succ
Nat.zero_add
Nat.mul_zero
Nat.mul_succ
Nat.zero_mul
Nat.pred_zero
Nat.pred_succ
List.nil_append
List.cons_append
List.append_nil
List.length_nil
List.length_cons
List.map_nil
List.map_cons
List.map_id
List.foldr_nil
List.foldr_cons
```

MVP `RwOnly` rules are exactly:

```text
Nat.add_comm
Nat.add_assoc
List.append_assoc
List.length_append
```

This exact `RwOnly` set is normative for `npa.stdlib.mvp.v1`.
Human-profile notes that a theorem is "not simp", "慎重にする", or may later move to a dedicated tactic do not exclude it from
Phase 6 AI `RwOnly` unless this exact set omits it.
Conversely, theorems not listed here, such as `Nat.mul_comm`, `Nat.mul_assoc`, `Nat.left_distrib`,
`Nat.right_distrib`, and `List.map_comp`, are not emitted in MVP rewrite profiles.

`SimpSafe` must not be assigned by name pattern alone.
The descriptor must pass Phase 4 rule validation and the Phase 6 simp lint described below.

## 8.2 MachineStdRewriteProfile

```rust
struct MachineStdRewriteProfile {
    profile_id: String,
    required_import_bundle_id: String,
    kernel_check_profile: KernelCheckProfileId,
    eq_family: Option<EqFamilyRef>,
    descriptors: Vec<MachineStdRewriteDescriptor>,
    profile_hash: HashString,
}

struct MachineStdRewriteProfileSet {
    library_profile_id: String,
    profiles: Vec<MachineStdRewriteProfile>,
    rewrite_profiles_hash: HashString,
}
```

MVP profiles:

```text
std.logic.rw
std.nat.rw
std.list.rw
std.all.rw
```

`MachineStdRewriteProfileSet.profiles` in the MVP profile must contain exactly these four `profile_id` values.
Missing profile ids, extra profile ids, duplicate profile ids, non-canonical `profile_id` order, or an id that differs only by
spelling are `InvalidStdRewriteProfile`.
The concrete emitted rewrite `profile_id` dictionary order is:

```text
std.all.rw
std.list.rw
std.logic.rw
std.nat.rw
```

MVP rewrite profile membership is exact:

```text
std.logic.rw:
  descriptors = []
  required_import_bundle_id = std.logic.mvp

std.nat.rw:
  required_import_bundle_id = std.nat.mvp
  SimpSafe:
    Nat.add_zero
    Nat.add_succ
    Nat.zero_add
    Nat.mul_zero
    Nat.mul_succ
    Nat.zero_mul
    Nat.pred_zero
    Nat.pred_succ
  RwOnly:
    Nat.add_comm
    Nat.add_assoc

std.list.rw:
  required_import_bundle_id = std.list.mvp
  SimpSafe:
    List.nil_append
    List.cons_append
    List.append_nil
    List.length_nil
    List.length_cons
    List.map_nil
    List.map_cons
    List.map_id
    List.foldr_nil
    List.foldr_cons
  RwOnly:
    List.append_assoc
    List.length_append

std.all.rw:
  required_import_bundle_id = std.all.mvp
  descriptors = semantic union of std.nat.rw descriptors and std.list.rw descriptors
```

The `std.all.rw` row above is not list concatenation.
Validators first validate `std.nat.rw` and `std.list.rw`, then build the expected set keyed by full
`MachineStdRewriteDescriptor` canonical bytes.
If the same canonical bytes appear in both source profiles, they are one set member and are accepted once.
The emitted `std.all.rw.descriptors` array must contain that set exactly once per member, in `MachineStdRewriteDescriptor`
canonical order.
Duplicate descriptors inside the emitted `std.all.rw` array remain `InvalidStdRewriteProfile`.
After building the expected union, validators must still validate the emitted `std.all.rw` profile under `std.all.mvp`.
For every union member, the descriptor source must resolve uniquely in the `std.all.mvp` direct import scope to the same
`MachineStdGlobalRef`, and Phase 4 rule validation with `std.all.rw`'s `kernel_check_profile` / `eq_family` must reproduce the
same `MachineStdRewriteDescriptor` canonical bytes.
The descriptor hashes remain tied to the resolved source theorem's certificate context; validators must not recompute them from a
synthetic `std.all` owner context.
Any source mismatch, ambiguity, or descriptor-byte mismatch is `InvalidStdRewriteProfile`.

`descriptors` は `source` の `MachineStdGlobalRef canonical order`、`direction`、`safety`、
`lhs_core_hash`、`rhs_core_hash`、`rule_telescope_hash` の順で sort し、完全重複を許しません。
The release generator may sort an internal candidate descriptor list before writing the artifact, but the emitted
`descriptors` array itself must already be in this canonical order.
The MVP descriptor set for each profile must match the exact membership above after resolving names and running Phase 4 rule
validation.
The `required_import_bundle_id`, `kernel_check_profile`, and `eq_family` fields must also match the row and MVP Eq selection above.
Any mismatch is `InvalidStdRewriteProfile`.
The names in the MVP membership table are release-spec labels.
Each table name must resolve uniquely to one public theorem export in the direct import scope of `required_import_bundle_id`.
Zero matches or multiple matches are `InvalidStdRewriteProfile` before comparing descriptors.
The exact-membership comparison uses the derived descriptor canonical bytes, not display names.
Non-canonical order, duplicate descriptors, missing descriptors, extra descriptors, wrong safety, or wrong direction are
`InvalidStdRewriteProfile`.
`kernel_check_profile` と `eq_family` は descriptor validation に使う Eq selection です。
MVP std rewrite profiles は `kernel_check_profile = "npa.kernel.v0.1.builtin-none"` かつ
`eq_family = std.logic.eq-family` です。
これは verified `Std.Logic` export の Eq / Eq.refl / Eq.rec を選ぶという意味であり、Phase 4 builtin Eq default ではありません。
Every MVP rewrite descriptor listed above has `direction = Forward`.
MVP rewrite profiles do not emit `Backward` descriptors.
AI clients that want a backward rewrite must request a Phase 5 `rw` tactic with `direction = "backward"` and pass Phase 5 validation;
the Phase 6 rewrite profile does not pre-authorize it as a separate descriptor.
Every descriptor source must resolve to a public theorem in the direct import scope of `required_import_bundle_id`.
The profile `eq_family` heads must also resolve in that same direct import scope.
For example, `std.list.rw` may reference `Std.List` theorem exports and the `Std.Logic` Eq family, but must not include
`Nat.add_zero`, because `Std.Nat` is only transitive closure for `std.list.mvp`, not a direct root import.
If the exact membership above names a theorem that is absent from the checked standard-library certificates, the release is
`InvalidStdRewriteProfile`.
A rewrite profile must not include a descriptor whose source theorem has `axiom_dependencies` outside the `allow_axioms` of the
profile's `required_import_bundle_id`.
This applies to `SimpSafe`, `RwOnly`, and `UnsafeForAutomation` descriptors.
For MVP constructive std profiles, every rewrite descriptor source must have `axiom_dependencies = []`.
Any mismatch is `InvalidStdRewriteProfile`.
`profile_hash` は次で計算します。

```text
MachineStdRewriteProfile canonical bytes:
  - tag "npa.phase6.std-rewrite-profile.v1"
  - profile_id
  - required_import_bundle_id
  - kernel_check_profile canonical bytes as Phase 5 KernelCheckProfileId
  - eq_family as Phase 6 Option<EqFamilyRef> using EqFamilyRef canonical bytes
  - descriptors in canonical order:
      MachineStdRewriteDescriptor canonical bytes

profile_hash:
  sha256(MachineStdRewriteProfile canonical bytes with profile_hash omitted)
```

`profile_hash` is a required JSON field and must equal the digest recomputed from that profile's canonical bytes.
Validators check each rewrite profile's `profile_hash` before computing `MachineStdRewriteProfileSet` canonical bytes.
The set canonical bytes use these recomputed profile digests after validation; they must not trust the raw JSON field value as
the set-hash input.
An individual rewrite profile `profile_hash` mismatch is `InvalidStdRewriteProfile`.

`MachineStdLibraryRelease.rewrite_profiles_hash` は、全 rewrite profile を `profile_id` 辞書順に並べた
canonical list の hash です。
`MachineStdRewriteProfileSet.library_profile_id` は `MachineStdLibraryRelease.library_profile_id` と一致しなければなりません。
`profiles` は `profile_id` 辞書順で、同じ `profile_id` を重複して持ってはいけません。

```text
MachineStdRewriteProfileSet canonical bytes:
  - tag "npa.phase6.std-rewrite-profile-set.v1"
  - library_profile_id
  - profiles in profile_id order:
      profile_hash digest bytes

rewrite_profiles_hash:
  sha256(MachineStdRewriteProfileSet canonical bytes with rewrite_profiles_hash omitted)
```

The theorem index and rewrite profiles must agree exactly.
For each `MachineStdTheoremEntry`, `rewrite_descriptors` is the sorted union of all descriptors in all validated
`MachineStdRewriteProfile.descriptors` whose `source` equals `entry.global_ref`.
If an entry carries a descriptor that is absent from every rewrite profile, or a rewrite profile descriptor is missing from the
corresponding theorem entry, validation fails with `InvalidStdTheoremIndex`.
`rw` mode is present exactly when `rewrite_descriptors` is non-empty.

## 8.3 Simp Lint

Phase 6 AI MVP includes a deterministic lint for `SimpSafe` sidecar entries.
This lint is a quality gate for the standard library release, not a trusted proof check.

```text
SimpSafe lint:
  - source theorem resolves to a valid Phase 4 ResolvedSimpRule
  - direction is forward
  - lhs_core_hash != rhs_core_hash
  - lhs syntactic size is greater than or equal to rhs syntactic size, unless source is in the fixed size exception list
  - rule is not commutativity
  - rule is not associativity
  - rule does not rewrite a variable-only lhs
  - rule does not introduce a head symbol absent from lhs, unless the fixed head-introduction exception list allows that head
```

MVP exception lists are fixed by this profile and are not user-extensible.
They do not bypass Phase 4 rule validation, the forward-direction requirement, the non-commutativity/non-associativity checks,
or the variable-only-lhs rejection.

```text
MVP size exception list:
  Nat.mul_succ

MVP head-introduction exception list:
  Nat.mul_succ:
    may introduce Nat.add
  List.length_nil:
    may introduce Nat.zero
  List.length_cons:
    may introduce Nat.succ
```

Exception entries are matched after resolving the rule source through the exact MVP profile membership.
The implementation compares the resolved `MachineStdGlobalRef` for the source theorem, not pretty text alone.
If another theorem has the same display name but a different hash, the exception does not apply.
Allowed introduced heads are also hash-bound.
The names in the table above are labels for these canonical checks:

```text
Head-introduction exception matching:
  - resolve the source theorem to `MachineStdGlobalRef`
  - traverse the lhs and rhs head symbol sets and normalize every head to `MachineStdGlobalRefView`
    using the source theorem's owner module certificate context
  - compute `introduced_heads = rhs_normalized_head_set - lhs_normalized_head_set`
    using `MachineStdGlobalRefView canonical bytes` set difference
  - resolve each allowed head label in the release module table:
      Nat.add  -> the unique `Std.Nat` public export named `Nat.add`
      Nat.zero -> the unique `Std.Nat` public export or generated export named `Nat.zero`
      Nat.succ -> the unique `Std.Nat` public export or generated export named `Nat.succ`
  - require `introduced_heads` to be a subset of the allowed heads by `MachineStdGlobalRefView canonical bytes`
```

If an allowed head label cannot be resolved uniquely to a release declaration/generation artifact with matching
`module / name / export_hash / certificate_hash / decl_interface_hash`, the release is `InvalidStdSimpProfile`.
If a future rule needs another exception, it must stay `RwOnly` until a new profile id extends this list.

The lint uses the Phase 4 `ResolvedSimpRule` produced by the profile validation Eq selection.
For a descriptor with `direction = Forward`, lint `lhs` means `ResolvedSimpRule.from_pattern` and lint `rhs` means
`ResolvedSimpRule.to_pattern`.
For any other direction, `SimpSafe` is rejected before size/head checks.

```text
syntactic size:
  the Phase 5 MachineExprView.size occurrence count for the core Expr
  no WHNF, reduction, unfolding, conversion, or pretty rendering is used

head symbol set:
  every syntactic global application/constant head in the core Expr, normalized to MachineStdGlobalRefView canonical bytes
  bound variables, local variables, sorts, and metadata are ignored
  GlobalRef normalization uses the certificate context of the resolved source theorem that produced the ResolvedSimpRule
  Imported / Local / generated refs use the same MachineStdGlobalRefView normalization rules as statement_head/constants

variable-only lhs:
  let (head, _) = flatten_app(lhs)
  true iff head is a Phase 1 BVar node
  local variables in a rule telescope are represented by BVar, so this rejects both a bare variable lhs and a variable-headed app
  no WHNF, beta/eta reduction, unfolding, or binder-name inspection is used
```

`rule is commutativity` and `rule is associativity` are structural recognizers over `from_pattern` / `to_pattern`, not name checks.
The MVP recognizers are exactly the following checks.
They use Phase 6 core expression canonical bytes for expression equality and flatten only syntactic `App` spines.
No WHNF, unfolding, eta, conversion, associativity normalization, or pretty-name matching is allowed.

```text
flatten_app(e):
  repeatedly peel App(fn, arg) from the outside
  return (head, args) where args are restored in left-to-right application order

same_expr(a, b):
  Phase 6 core expression canonical bytes of a and b are byte-for-byte equal

same_head_and_prefix(e1, e2):
  let (h1, args1) = flatten_app(e1)
  let (h2, args2) = flatten_app(e2)
  require same_expr(h1, h2)
  require args1.len == args2.len and args1.len >= 2
  let prefix_len = args1.len - 2
  require same_expr(args1[i], args2[i]) for every i < prefix_len
```

Commutativity recognizer:

```text
commutativity(from_pattern, to_pattern):
  require same_head_and_prefix(from_pattern, to_pattern)
  let (_, from_args) = flatten_app(from_pattern)
  let (_, to_args) = flatten_app(to_pattern)
  let prefix_len = from_args.len - 2
  require same_expr(from_args[prefix_len],     to_args[prefix_len + 1])
  require same_expr(from_args[prefix_len + 1], to_args[prefix_len])
```

Associativity recognizer:

```text
left_assoc_shape(e, head, prefix):
  let (outer_head, outer_args) = flatten_app(e)
  require same_expr(outer_head, head)
  require outer_args.len == prefix.len + 2
  require same_expr(outer_args[i], prefix[i]) for every i < prefix.len
  let inner = outer_args[prefix.len]
  let z = outer_args[prefix.len + 1]
  let (inner_head, inner_args) = flatten_app(inner)
  require same_expr(inner_head, head)
  require inner_args.len == prefix.len + 2
  require same_expr(inner_args[i], prefix[i]) for every i < prefix.len
  let x = inner_args[prefix.len]
  let y = inner_args[prefix.len + 1]
  return Some(x, y, z)

right_assoc_shape(e, head, prefix):
  let (outer_head, outer_args) = flatten_app(e)
  require same_expr(outer_head, head)
  require outer_args.len == prefix.len + 2
  require same_expr(outer_args[i], prefix[i]) for every i < prefix.len
  let x = outer_args[prefix.len]
  let inner = outer_args[prefix.len + 1]
  let (inner_head, inner_args) = flatten_app(inner)
  require same_expr(inner_head, head)
  require inner_args.len == prefix.len + 2
  require same_expr(inner_args[i], prefix[i]) for every i < prefix.len
  let y = inner_args[prefix.len]
  let z = inner_args[prefix.len + 1]
  return Some(x, y, z)

associativity(from_pattern, to_pattern):
  require same_head_and_prefix(from_pattern, to_pattern)
  let (head, outer_args) = flatten_app(from_pattern)
  let prefix = outer_args[0 .. outer_args.len - 2]
  return true iff either:
    - left_assoc_shape(from_pattern, head, prefix) returns Some(x1, y1, z1)
      and right_assoc_shape(to_pattern, head, prefix) returns Some(x2, y2, z2)
      and same_expr(x1, x2), same_expr(y1, y2), same_expr(z1, z2)
    - right_assoc_shape(from_pattern, head, prefix) returns Some(x1, y1, z1)
      and left_assoc_shape(to_pattern, head, prefix) returns Some(x2, y2, z2)
      and same_expr(x1, x2), same_expr(y1, y2), same_expr(z1, z2)
```

If either recognizer returns true, `SimpSafe` is rejected and the rule may only appear as `RwOnly`.
If both recognizers return false, the commutativity/associativity lint checks pass.
Implementations must not add extra commutativity or associativity patterns without a new profile id.

---

# 9. Simp Profiles

`MachineStdSimpProfile` is a deterministic list of `SimpRuleRef` values.
It can be used to populate Phase 5 `MachineTacticOptionsRequest.simp_rules`.

```rust
struct MachineStdSimpProfile {
    profile_id: String,
    required_import_bundle_id: String,
    kernel_check_profile: KernelCheckProfileId,
    eq_family: Option<EqFamilyRef>,
    rules: Vec<SimpRuleRef>,
    profile_hash: HashString,
}

struct MachineStdSimpProfileSet {
    library_profile_id: String,
    profiles: Vec<MachineStdSimpProfile>,
    simp_profiles_hash: HashString,
}
```

MVP profiles:

```text
std.logic.simp
std.nat.simp
std.list.simp
std.all.simp
```

`MachineStdSimpProfileSet.profiles` in the MVP profile must contain exactly these four `profile_id` values.
Missing profile ids, extra profile ids, duplicate profile ids, non-canonical `profile_id` order, or an id that differs only by
spelling are `InvalidStdSimpProfile`.
The concrete emitted simp `profile_id` dictionary order is:

```text
std.all.simp
std.list.simp
std.logic.simp
std.nat.simp
```

MVP simp profile membership is exact:

```text
std.logic.simp:
  required_import_bundle_id = std.logic.mvp
  rules = []

std.nat.simp:
  required_import_bundle_id = std.nat.mvp
  rules:
    Nat.add_zero
    Nat.add_succ
    Nat.zero_add
    Nat.mul_zero
    Nat.mul_succ
    Nat.zero_mul
    Nat.pred_zero
    Nat.pred_succ

std.list.simp:
  required_import_bundle_id = std.list.mvp
  rules:
    List.nil_append
    List.cons_append
    List.append_nil
    List.length_nil
    List.length_cons
    List.map_nil
    List.map_cons
    List.map_id
    List.foldr_nil
    List.foldr_cons

std.all.simp:
  required_import_bundle_id = std.all.mvp
  rules = semantic union of std.nat.simp rules and std.list.simp rules
```

The `std.all.simp` row above is not list concatenation.
Validators first validate `std.nat.simp` and `std.list.simp`, then build the expected set keyed by full Phase 4 `SimpRuleKey`
canonical bytes.
If the same canonical bytes appear in both source profiles, they are one set member and are accepted once.
The emitted `std.all.simp.rules` array must contain that set exactly once per member, in Phase 4 `SimpRuleKey` canonical order.
Duplicate rules inside the emitted `std.all.simp` array remain `InvalidStdSimpProfile`.
Before deduplicating that union, validators retain the resolved `MachineStdGlobalRef` target of each source-profile rule.
If identical `SimpRuleKey` canonical bytes from source profiles resolve to different theorem targets, the union is ambiguous and
the release is `InvalidStdSimpProfile`.
After building the expected union, validators must still validate the emitted `std.all.simp` profile under `std.all.mvp`.
Every emitted union rule must resolve uniquely in the `std.all.mvp` direct import scope to the same `MachineStdGlobalRef` target
and direction as the source-profile rule, and the paired `std.all.rw` profile must contain the matching `SimpSafe` descriptor.
Any ambiguity, source mismatch, or missing paired descriptor is `InvalidStdSimpProfile`.

`std.logic.simp` is intentionally empty in the MVP AI profile.
The generated `Eq.refl` constructor may be used through the explicit Eq family, but it is not a theorem-index entry and is not a
`SimpRuleRef`.
Human-facing notes that mention an "Eq.refl closure" are descriptive library guidance only and do not add a Phase 6 AI simp rule
unless a future profile id defines one.

Rules are emitted in Phase 4 `SimpRuleKey` canonical order.
A `MachineStdSimpProfile.rules` artifact must not contain duplicate `SimpRuleKey` values.
The release generator may sort/dedup an internal candidate list before writing the artifact, but the artifact validator rejects
non-canonical order and duplicates instead of silently repairing them.
A profile that includes the same `name / decl_interface_hash / direction` twice is `InvalidStdSimpProfile`.
The MVP rule set for each profile must match the exact membership above after resolving names.
The `required_import_bundle_id`, `kernel_check_profile`, and `eq_family` fields must also match the row and MVP Eq selection above.
Missing rules, extra rules, wrong direction, wrong profile field value, non-canonical order, or duplicate rules are
`InvalidStdSimpProfile`.
MVP simp profiles are validated against the already validated rewrite profiles.
The fixed simp-to-rewrite profile pairing is:

```text
std.logic.simp -> std.logic.rw
std.nat.simp   -> std.nat.rw
std.list.simp  -> std.list.rw
std.all.simp   -> std.all.rw
```

Every non-empty `MachineStdSimpProfile.rules` entry must resolve to a theorem whose `MachineStdGlobalRef` and direction match one
descriptor in the paired rewrite profile, and that descriptor's `safety` must be `SimpSafe`.
A rule whose only matching descriptor is `RwOnly` or `UnsafeForAutomation`, or whose paired rewrite profile has no matching
descriptor, is `InvalidStdSimpProfile`.
A profile must not include a theorem whose `axiom_dependencies` are outside the bundle `allow_axioms`.
For MVP constructive std profiles, every rule must have `axiom_dependencies = []`.
MVP std simp profiles use `kernel_check_profile = "npa.kernel.v0.1.builtin-none"` and `eq_family = std.logic.eq-family`,
which selects the imported `Std.Logic` Eq family during validation.
Every MVP `SimpRuleRef` listed above has `direction = "forward"`.
MVP simp profiles do not emit backward simp rules.
Every `SimpRuleRef` must resolve uniquely to one public theorem in the direct import scope of `required_import_bundle_id`.
Zero matches or multiple matches for `name / decl_interface_hash / direction` are `InvalidStdSimpProfile`; validators must
not choose an arbitrary matching theorem.
The profile `eq_family` heads must also resolve in that same direct import scope.
If the exact membership above names a theorem that is absent from the checked standard-library certificates, the release is
`InvalidStdSimpProfile`.

```text
MachineStdSimpProfile canonical bytes:
  - tag "npa.phase6.std-simp-profile.v1"
  - profile_id
  - required_import_bundle_id
  - kernel_check_profile canonical bytes as Phase 5 KernelCheckProfileId
  - eq_family as Phase 6 Option<EqFamilyRef> using EqFamilyRef canonical bytes
  - rules in Phase 4 SimpRuleKey canonical order

profile_hash:
  sha256(MachineStdSimpProfile canonical bytes with profile_hash omitted)
```

`profile_hash` is a required JSON field and must equal the digest recomputed from that profile's canonical bytes.
Validators check each simp profile's `profile_hash` before computing `MachineStdSimpProfileSet` canonical bytes.
The set canonical bytes use these recomputed profile digests after validation; they must not trust the raw JSON field value as
the set-hash input.
An individual simp profile `profile_hash` mismatch is `InvalidStdSimpProfile`.

`MachineStdLibraryRelease.simp_profiles_hash` は、全 simp profile を `profile_id` 辞書順に並べた
canonical list の hash です。
`MachineStdSimpProfileSet.library_profile_id` は `MachineStdLibraryRelease.library_profile_id` と一致しなければなりません。
`profiles` は `profile_id` 辞書順で、同じ `profile_id` を重複して持ってはいけません。

```text
MachineStdSimpProfileSet canonical bytes:
  - tag "npa.phase6.std-simp-profile-set.v1"
  - library_profile_id
  - profiles in profile_id order:
      profile_hash digest bytes

simp_profiles_hash:
  sha256(MachineStdSimpProfileSet canonical bytes with simp_profiles_hash omitted)
```

The theorem index and simp profiles must agree on `simp` mode.
For each theorem entry, `simp` mode is present exactly when at least one validated `SimpRuleRef` in any
`MachineStdSimpProfile.rules` resolves to that entry's `global_ref`.
If `simp` mode is present without such a rule, or a matching rule exists but `simp` mode is absent, validation fails with
`InvalidStdTheoremIndex`.

---

# 10. Prompt Metadata

Phase 6 AI may provide prompt-friendly descriptions of declarations, but these are non-trusted.

```rust
struct MachineStdPromptMetadata {
    global_ref: MachineStdGlobalRef,
    short_doc: Option<String>,
    examples: Vec<MachineStdPromptExample>,
    tags: Vec<String>,
}

struct MachineStdPromptMetadataSet {
    metadata_profile_id: String,
    library_profile_id: String,
    entries: Vec<MachineStdPromptMetadata>,
    prompt_metadata_hash: HashString,
}

struct MachineStdPromptExample {
    goal_core_hash: HashString,
    imports_bundle_id: String,
    candidate_kind: String,
    display: String,
}
```

MVP values when `Std.machine-prompt-metadata.json` is emitted:

```text
metadata_profile_id = "npa.stdlib.prompt-metadata.mvp.v1"
library_profile_id = "npa.stdlib.mvp.v1"
```

MVP prompt metadata rules:

```text
- metadata must carry an explicit binding to exact module/name/export_hash/certificate_hash/decl_interface_hash
- metadata bytes must not be an input to certificate_hash, export_hash, theorem_index_hash, or simp_rule_identity
- metadata must not be used to construct proof terms
- tags are lower_ascii strings from a fixed vocabulary
- examples are display-only and must not be copied into suggested_candidates without Phase 5 validation
- metadata is always omitted from theorem_index_hash
- if metadata is emitted, it carries its own prompt_metadata_hash
```

`Std.machine-prompt-metadata.json` is optional in MVP.
It is not referenced by `MachineStdLibraryRelease` and is not included in `std_library_release_hash`.
Changing only prompt metadata changes `prompt_metadata_hash`, but must not change `std_library_release_hash`.
If emitted, it must validate against the theorem index entry set and carry its own `prompt_metadata_hash`.
`metadata_profile_id`, `library_profile_id`, and recomputed `prompt_metadata_hash` must match the MVP values and the artifact field.
A mismatch is `InvalidStdPromptMetadata`.
Absent prompt metadata is valid.
When prompt metadata is emitted, `entries` may be empty or any subset of `MachineStdTheoremIndex.entries`.
MVP prompt metadata has no completeness requirement, and validators must not compare prompt metadata entry count against theorem
index entry count.
Every emitted prompt metadata entry must target an existing theorem-index `global_ref`; a missing metadata entry for a theorem is
valid, but an entry outside the theorem index is `InvalidStdPromptMetadata`.
`entries` are sorted by `MachineStdGlobalRef canonical order`.
Two prompt metadata entries with the same `global_ref` are rejected as `InvalidStdPromptMetadata`.

```text
MachineStdPromptMetadataSet canonical bytes:
  - tag "npa.phase6.std-prompt-metadata-set.v1"
  - metadata_profile_id
  - library_profile_id
  - entries in MachineStdGlobalRef canonical order:
      MachineStdPromptMetadata canonical bytes

prompt_metadata_hash:
  sha256(MachineStdPromptMetadataSet canonical bytes with prompt_metadata_hash omitted)
```

`MachineStdPromptMetadata canonical bytes` include `global_ref`, `short_doc` option, `examples` in emitted JSON array order,
and `tags` sorted lexicographically by ASCII byte value.
Example order is part of prompt metadata artifact identity.
There is no separate source-order rule for examples in the MVP profile.
Generators must choose a deterministic JSON array order, and validators preserve that order when hashing; validators must not sort,
deduplicate, or otherwise repair `examples`.
Duplicate tags are rejected.
The emitted JSON `tags` array must already be in this ASCII byte lexicographic order.
Non-canonical tag order is `InvalidStdPromptMetadata`; validators must not silently sort a malformed artifact before accepting it.

```text
MachineStdPromptMetadata canonical bytes:
  - tag "npa.phase6.std-prompt-metadata.v1"
  - global_ref canonical bytes
  - short_doc option as String
  - examples in JSON array order:
      MachineStdPromptExample canonical bytes
  - tags in ASCII byte lexicographic order

MachineStdPromptExample canonical bytes:
  - tag "npa.phase6.std-prompt-example.v1"
  - goal_core_hash digest bytes
  - imports_bundle_id
  - candidate_kind
  - display
```

`MachineStdPromptExample.goal_core_hash` is a routing key for the example goal target.
For MVP prompt metadata it is:

```text
goal_core_hash:
  phase6_core_expr_hash(the closed example goal target Expr)
```

The example goal target is elaborated under `imports_bundle_id` with an empty local context.
Therefore an MVP prompt example target must be closed.
Because a prompt example target is not owned by a certificate declaration, its `GlobalRef` context is the synthetic Phase 5
current-module expression context whose import vector is the bundle's direct imports.
The generator must build the same `root_imports` / `import_closure` request as the corresponding `MachineStdImportBundle`, create the
Phase 5 verified session import context, elaborate the closed goal target in that session, and hash the resulting core expression
using the Phase 2 `TermHashPayload` with the session's direct import table.
For this prompt-goal expression context, `GlobalRef::Imported.import_index` indexes the Phase 5 session `imports` vector: the
deduped direct imports from `root_imports`, sorted by `(module, export_hash, certificate_hash)` canonical order.
`import_closure` certificates are verified dependencies only; closure-only modules do not consume `GlobalRef::Imported` indices and
are not directly addressable from the prompt-goal expression.
A generator must not emit an MVP prompt example whose checked closed goal target requires a direct reference to a closure-only
module.
This closure-only check is a generator and fixture-test obligation, not an MVP release-validator semantic check.
The MVP prompt metadata artifact carries only `goal_core_hash`, not the source goal target, so a release validator cannot recompute
whether the hash came from a direct-import-only expression.
The generator must not use release-module order, filesystem path order, prompt metadata entry order, import-closure order, or
release-global `MachineStdGlobalRef` bytes as a substitute for this expression hash context.
`goal_core_hash` excludes local context, pretty goal text, `display`, `candidate_kind`, `imports_bundle_id`, source span, and prompt text.
Because the prompt metadata artifact does not carry the original goal source or local context, the release validator checks only that
`imports_bundle_id` is known after the schema pass has already accepted `goal_core_hash` as a well-formed `HashString`.
The MVP release validator must not reject a prompt example as a stale `goal_core_hash` target, because it has no source payload from
which to recompute that hash.
The release generator, fixture tests, or any tool that emits prompt metadata must compute this hash from the checked closed goal
target and must not hash the `display` string.
Prompt examples that need local hypotheses or open de Bruijn variables require a future prompt metadata profile id with an explicit
context field.

Fixed MVP tag vocabulary:

```text
eq
logic
nat
list
algebra
simp
rw
apply
intro
elim
induction
```

Unknown tags are `InvalidStdPromptMetadata`.
`MachineStdPromptExample.candidate_kind` must be one of `exact`, `apply`, `rw`, `simp`, or `note`.
It is display/routing metadata only and must not be treated as a validated tactic kind.
`MachineStdPromptExample.imports_bundle_id` must equal one `MachineStdImportBundle.bundle_id` in the same release.
An unknown bundle id is `InvalidStdPromptMetadata`.
`examples` preserve JSON array order for display and artifact identity.
Duplicate examples are allowed; they change `prompt_metadata_hash` but do not affect trusted proof state.

---

# 11. Axiom Policy

MVP standard library modules are constructive and no-custom-axiom.

```rust
struct MachineStdAxiomReport {
    library_profile_id: String,
    modules: Vec<MachineStdModuleAxiomReport>,
    axiom_report_hash: HashString,
}

struct MachineStdModuleAxiomReport {
    module: ModuleName,
    export_hash: HashString,
    certificate_hash: HashString,
    module_axioms: Vec<MachineStdAxiomRef>,
    transitive_axioms: Vec<MachineStdAxiomRef>,
}
```

```text
Std.Logic:
  module_axioms = []
  transitive_axioms = []
Std.Nat:
  module_axioms = []
  transitive_axioms = []
Std.List:
  module_axioms = []
  transitive_axioms = []
Std.Algebra.Basic:
  module_axioms = []
  transitive_axioms = []
```

Any `AxiomDecl` in these modules is invalid unless the module is explicitly listed as an axiom module in the release profile.
MVP has no axiom modules.

`MachineStdAxiomReport.modules` must contain exactly the modules in `MachineStdLibraryRelease.modules`, in `ModuleName`
canonical order.
Duplicate modules or non-canonical module order are `InvalidStdAxiomPolicy`.
Each entry's `export_hash` and `certificate_hash` must match the corresponding `MachineStdModuleArtifact`.
For each module:

```text
module_axioms:
  the exact sorted/dedup projection of the module's Phase 2 verifier AxiomReport.module_axioms field

transitive_axioms:
  the exact sorted/dedup union of:
    - module_axioms
    - every imported module's transitive_axioms reachable through the certificate ImportEntry closure
```

Both lists use `MachineStdAxiomRef canonical order`.
Duplicate or non-canonical `module_axioms` / `transitive_axioms` arrays are `InvalidStdAxiomPolicy`.
Validators must not sort or deduplicate a malformed axiom report before recomputing `axiom_report_hash`.
MVP no-custom-axiom validation requires both `module_axioms` and `transitive_axioms` to be `[]` for every release module.
Any mismatch with verifier-derived projection is `InvalidStdAxiomPolicy`.
`module_axioms` is not recomputed from public exports, owned `AxiomDecl` declarations, theorem-index entries, or prompt metadata.
It is derived from the verifier's module-level axiom report payload so imported/private axiom dependencies used by the module
cannot be hidden by export filtering.

In the MVP constructive profile, `sorry`, `admit`, generated placeholder axioms, imported classical axioms, or private axiom
dependencies are release blockers.
They must appear in Phase 2 axiom reports and cause `InvalidStdAxiomPolicy`.
Phase 6 axiom reports must never contain Phase 5 `CurrentModule` axiom refs.
All axiom dependencies are projected to `MachineStdAxiomRef` using the Local/Imported projection rules in section 7.3.
If that projection cannot find a unique axiom declaration name and `decl_interface_hash` in verifier output,
the artifact is `InvalidStdAxiomPolicy`.

Future classical modules must be separated:

```text
Std.Classical
Std.Logic.Propext
```

They must use distinct import bundle ids and non-empty `allow_axioms`.
Constructive bundles must not import them transitively.

```text
MachineStdAxiomReport canonical bytes:
  - tag "npa.phase6.std-axiom-report.v1"
  - library_profile_id
  - modules in ModuleName canonical order:
      module
      export_hash digest bytes
      certificate_hash digest bytes
      module_axioms in MachineStdAxiomRef canonical order
      transitive_axioms in MachineStdAxiomRef canonical order

axiom_report_hash:
  sha256(MachineStdAxiomReport canonical bytes with axiom_report_hash omitted)
```

`MachineStdLibraryRelease.axiom_report_hash` は、この release-wide `MachineStdAxiomReport.axiom_report_hash` と完全一致しなければなりません。
This is distinct from each `MachineStdModuleArtifact.axiom_report_hash`, which is the module-level Phase 2 certificate
`axiom_report_hash` recomputed while validating one `.npcert` payload.

---

# 12. Validation Order

Release validation order is fixed so build failures are deterministic.

```text
MachineStdRelease validation:
  1. Parse artifact JSON with duplicate-key detection.
     Unknown field, missing required field, null where not allowed, wire-level invalid integer/hash/name grammar:
       InvalidStdArtifactShape
     This step covers JSON shape and scalar wire grammar only. Later semantic restrictions on otherwise well-formed scalars
     (for example MachineSurfaceRenderableName, MachineUniverseParamName, stale HashString targets, and hash self-mismatches)
     are reported by the owning validation step below.

  2. Build the release module payload table and verify certificates in dependency order.
     Before reading any certificate bytes, validate the release manifest scalar fields and module artifact table enough to build a
     bijective `ModuleName -> MachineStdModuleArtifact` map.
     This pre-pass checks the MVP `protocol_version`, `library_profile_id`, `core_spec_id`, `kernel_semantics_profile_id`, exact
     four-module membership, duplicate modules, ModuleName canonical module order, fixed module path mapping, package locator
     normalization, and fixed `certificate_encoding` spelling.
     If a module artifact cannot be identified uniquely from the manifest before certificate decoding, the release is
     `InvalidStdLibraryRelease`.
     Read exactly the fixed locator paths for the four manifest modules. Use the Phase 2 canonical certificate decoder, not debug
     JSON, sidecars, source files, or a lenient ad hoc parser, to decode the certificate header and ImportEntry list.
     A decoder failure while reading this prefix, trailing bytes, non-canonical encoding, or bytes that cannot later be fully decoded
     as the same Phase 2 certificate are `InvalidStdLibraryRelease`.
     Identify the decoded module name and Phase 2 ImportEntry list, and match each payload to the corresponding
     MachineStdModuleArtifact.
     Compute the standard-library import graph from those ImportEntry lists. Every ImportEntry must resolve to one release module
     artifact by module/export_hash/certificate_hash; an ordinary Core/prelude ImportEntry is invalid.
     Verify modules with the Phase 2 verifier in high-trust mode in topological dependency order, using ModuleName canonical order as
     the deterministic tie-breaker for independent modules.
     Cyclic imports, unresolved imports, module name/path mismatch, duplicate module payloads, or recomputed module export_hash /
     module certificate_hash / module-level Phase 2 axiom_report_hash mismatch with MachineStdModuleArtifact:
       InvalidStdLibraryRelease

  3. Build module context table from verifier output.
     Unsupported verifier output for the MVP core spec, duplicate or non-renderable theorem-index-visible ExportEntry universe params,
     missing/private/stale/wrong-kind/cross-parent MVP Eq/Nat family export:
       InvalidStdLibraryRelease

  4. Validate no-custom-axiom policy.
     Disallowed axiom, axiom ref projection failure, axiom report module-set mismatch,
     ExportEntry.axiom_dependencies projection failure,
     duplicate/non-canonical axiom report module order, duplicate/non-canonical module_axioms or transitive_axioms,
     module/transitive axiom projection mismatch,
     MachineStdAxiomReport.axiom_report_hash self-mismatch:
       InvalidStdAxiomPolicy

  5. Validate import bundle closure, `allow_axioms`, and recipe ids.
     Missing/extra bundle id, duplicate/non-canonical bundle order,
     duplicate root_imports or import_closure keys, non-canonical root_imports/import_closure order,
     missing dependency, extra closure certificate,
     embedded certificate encoding/hex failure, embedded certificate byte/hash mismatch,
     root import not in closure, invalid bundle-to-recipe mapping, non-renderable recipe name,
     non-canonical or duplicate recipe simp_rules,
     non-empty MVP allow_axioms, invalid/non-imported allow_axioms variant, malformed/unresolved allow_axioms entry,
     non-canonical or duplicate allow_axioms,
     MachineStdImportBundleSet.import_bundles_hash self-mismatch:
       InvalidStdImportBundle
     This step does not resolve recipe `SimpRuleRef`, `EqFamilyRef`, or `NatFamilyRef` against the bundle direct import scope.
     Stale, unknown, or ambiguous recipe references are checked in Step 10.

  6. Validate theorem index identity and certificate-derived fields against public ExportEntry set.
     index_profile_id mismatch, duplicate/non-canonical entry order, entry stale export/interface hash,
     missing export, missing required entry, extra entry,
     kind mismatch, sidecar universe_params mismatch/duplicate/invalid, statement_core_hash mismatch, statement_head/constants mismatch,
     axiom_dependencies sidecar mismatch, invalid renderable name/universe param,
     non-canonical or duplicate modes/constants/rewrite_descriptors/axiom_dependencies,
     non-null proof_term_size, MachineStdTheoremIndex.index_hash self-mismatch:
       InvalidStdTheoremIndex
     For `modes` and `rewrite_descriptors`, this step checks only emitted order/duplicate shape, not final profile-derived content.

  7. Validate theorem entry attribute shape only.
     Non-canonical order, duplicate attribute, MVP-reserved attribute:
       InvalidStdTheoremIndex
     Do not compare Simp/Rw/Apply attributes with derived modes yet; that depends on validated profiles.

  8. Validate rewrite profiles.
     Missing/extra rewrite profile id, duplicate/non-canonical profile order, wrong required_import_bundle_id/kernel_check_profile/eq_family,
     unknown descriptor source, non-renderable family name, ambiguous membership-table name, duplicate descriptor, non-canonical descriptor order,
     extra/missing descriptor, unsafe profile membership, invalid rule universe parameter, axiom mismatch,
     eq_family unknown/stale/ambiguous/coherence failure,
     per-profile profile_hash mismatch, MachineStdRewriteProfileSet.rewrite_profiles_hash self-mismatch:
       InvalidStdRewriteProfile

  9. Validate simp profiles.
     Missing/extra simp profile id, duplicate/non-canonical profile order, wrong required_import_bundle_id/kernel_check_profile/eq_family,
     non-renderable rule/family name, unknown or ambiguous rule, duplicate rule, non-canonical rule order, extra/missing rule,
     unsafe rule in SimpSafe profile,
     missing paired SimpSafe rewrite descriptor, axiom mismatch, eq_family unknown/stale/ambiguous/coherence failure, per-profile profile_hash mismatch,
     MachineStdSimpProfileSet.simp_profiles_hash self-mismatch:
       InvalidStdSimpProfile

  10. Validate import bundle recommended tactic options against validated simp profiles.
      Recipe/profile rule mismatch, recipe kernel/family/limit mismatch, stale/unknown/ambiguous recipe rule or family reference,
      Phase 5 option validation failure:
        InvalidStdImportBundle

  11. Validate theorem index derived metadata against validated profiles.
      modes mismatch, attributes/modes mismatch, rewrite_descriptors mismatch, rewrite shape failure:
        InvalidStdTheoremIndex
      Simp/Rw/Apply attributes must match the modes derived from the validated rewrite/simp profiles at this step.
      This step compares final expected contents only; non-canonical emitted order and duplicates were already rejected in Steps 6 and 7.

  12. Compare validated sidecar hashes with MachineStdLibraryRelease manifest hash fields.
     Also compare manifest-bound sidecar library_profile_id values and module-artifact summary counts
     (`public_export_count`, `theorem_index_entry_count`, `simp_rule_count`) against recomputed values.
     Optional prompt metadata is not manifest-bound and is excluded from this step; if present, its `library_profile_id` is checked
     in Step 13 and mismatches are `InvalidStdPromptMetadata`.
     The count checks happen after the theorem index and simp profiles have passed their own content validation; a count mismatch is
     a release/module-artifact summary mismatch.
     Manifest hash mismatch, sidecar/manifest library_profile_id mismatch, module count mismatch, summary count mismatch:
       InvalidStdLibraryRelease

  13. If optional prompt metadata is present, validate it against the theorem index and import bundle set.
      metadata_profile_id mismatch, library_profile_id mismatch, prompt_metadata_hash self-mismatch,
      non-canonical entry order, duplicate global_ref, stale global_ref target,
      non-canonical tag order, duplicate tag, unknown tag, invalid candidate_kind, unknown imports_bundle_id:
        InvalidStdPromptMetadata
      Prompt metadata may be empty or a strict subset of theorem-index entries; missing metadata entries are valid.
      At this step, the validator does not recompute or reject semantically stale goal_core_hash values in the MVP profile.
```

These error names are build-time artifact validation categories.
They are not Phase 5 `MachineApiErrorKind` unless an endpoint later exposes standard-library artifact loading as an API.

---

# 13. Phase 5 Integration

To start an AI proof session using the standard library, the client does not ask the Phase 5 server to resolve modules by name.
It expands a Phase 6 import bundle into a Phase 5 request.

```json
{
  "import_closure": [
    {
      "module": "Std.Logic",
      "expected_export_hash": "sha256:...",
      "expected_certificate_hash": "sha256:...",
      "certificate": {
        "encoding": "npa.certificate.canonical.v0.1.hex",
        "bytes": "..."
      }
    }
  ],
  "imports": [
    {
      "module": "Std.Logic",
      "expected_export_hash": "sha256:...",
      "expected_certificate_hash": "sha256:..."
    }
  ],
  "options": {
    "kernel_check_profile": "npa.kernel.v0.1.builtin-none",
    "allow_axioms": [],
    "tactic_options": {
      "simp_rules": [],
      "eq_family": {
        "eq_name": "Eq",
        "eq_interface_hash": "sha256:...",
        "refl_name": "Eq.refl",
        "refl_interface_hash": "sha256:...",
        "rec_name": "Eq.rec",
        "rec_interface_hash": "sha256:..."
      },
      "nat_family": null,
      "max_simp_rewrite_steps": 100,
      "max_open_goals": 32,
      "max_metas": 64
    }
  }
}
```

Phase 5 is still authoritative for:

```text
- import closure verification
- current session direct import scope
- MachineSurfaceCallableInterfaceTable construction
- simp registry validation
- theorem search result generation in MVP
- tactic candidate validation / execution
```

Phase 6 recipes are never accepted as already validated Phase 5 state.

---

# 14. Phase 7 Integration

Phase 7 can use Phase 6 AI artifacts for:

```text
- premise retrieval
- candidate template selection
- choosing a standard import bundle
- choosing a simp profile
- avoiding known unsafe rewrite rules
- filtering axiom-dependent declarations
- prompt payload enrichment
```

Phase 7 must not:

```text
- treat theorem index entries as proof
- trust attribute sidecars without Phase 5 validation
- copy prompt examples directly into accepted proof steps
- use embeddings or usage stats in certificate payload
- import modules not present in the verified import closure
- bypass Phase 5 run/batch/replay/verify
```

Every generated candidate is re-entered through Phase 5 `/machine/tactics/run` or `/machine/tactics/batch`.
Every successful proof chain is replayed and then verified.

---

# 15. Phase 8 Audit Hooks

Phase 8 independent checker can audit Phase 6 AI artifacts without trusting them.

Audit checks:

```text
- every module certificate verifies independently
- release manifest hashes match certificate bytes and manifest-bound sidecar hashes
- optional prompt metadata and other non-manifest sidecars are excluded from std_library_release_hash
- theorem index entries point only to public exports
- every decl_interface_hash matches certificate verifier output
- every axiom dependency in index equals the public ExportEntry.axiom_dependencies projection
- axiom report sidecar module_axioms equals verifier-derived AxiomReport.module_axioms projection
- axiom report sidecar transitive_axioms equals the deterministic union over certificate ImportEntry closure
- every simp profile rule resolves to a theorem with matching decl_interface_hash
- every rewrite profile descriptor resolves to a theorem with matching decl_interface_hash and recomputed descriptor hashes
- theorem index rewrite_descriptors equals the union of matching validated rewrite profile descriptors
- SimpSafe descriptors satisfy the fixed Phase 6 simp lint and RwOnly descriptors are excluded from simp profiles
- std.all.rw and std.all.simp revalidate under std.all.mvp and match their source-profile targets
- constructive bundles have empty allow_axioms
- every module in a constructive bundle closure has empty module_axioms and transitive_axioms
- import bundles are minimal transitive closures
```

If an index entry is wrong but certificates verify, the proof system is still sound.
The artifact is rejected as bad metadata, not as a kernel failure.

---

# 16. Tests

Phase 6 AI MVP should include these tests.

```text
release determinism:
  protocol_version/library_profile_id/core_spec_id/kernel_semantics_profile_id mismatch is rejected
  missing/extra MVP release modules or fixed module path mismatches are rejected
  locator paths with absolute paths, `..`, `.`, backslashes, duplicate slashes, trailing slash, or symlink escape are rejected
  same certificate bytes and manifest-bound sidecars produce same std_library_release_hash
  reordered JSON object fields do not change std_library_release_hash
  changing only optional prompt metadata changes prompt_metadata_hash but not std_library_release_hash
  stale sidecar root hash fields are rejected by the owning sidecar validation before manifest hash comparison
  manifest hash mismatch against an otherwise valid sidecar is rejected as InvalidStdLibraryRelease
  module artifact simp_rule_count is computed from resolved MachineStdGlobalRef/direction rule targets, not raw SimpRuleKey module guesses
  malformed HashString, integer, ModuleName, or FullyQualifiedName wire grammar is rejected as InvalidStdArtifactShape
  well-formed but semantically invalid names or stale hash targets are rejected by the owning sidecar validation step
  Std.machine-release.json that contains std_library_release_hash as a field is rejected as an unknown field
  sidecar top-level arrays are rejected; each sidecar must use the fixed root object schema
  canonical enum bytes use the fixed one-byte tags, not JSON strings
  JSON enum strings use only the fixed wire spellings
  unknown scalar enum JSON strings are rejected as InvalidStdArtifactShape, not as owning semantic mismatches
  MachineStdGlobalRefView JSON requires kind = decl/generated and rejects mixed variant fields
  MachineStdAxiomRef JSON rejects kind/source_index/certificate_hash fields
  reordered module list is rejected unless it is the concrete MVP ModuleName canonical order
  duplicate/missing manifest modules are rejected before certificate payloads are matched to module artifacts
  MachineStdModuleArtifact.certificate_encoding other than npa.certificate.canonical.v0.1.hex is rejected

recipe determinism:
  generated candidate simp_rules canonicalize before emission, but emitted duplicate/reordered recipe simp_rules are rejected
  emitted recipe simp_rules are not repaired by Phase 5-style sort/dedup before import bundle hashing
  non-renderable SimpRuleRef/EqFamilyRef/NatFamilyRef names are rejected in emitted recipes
  changing kernel_check_profile changes the candidate import bundle hash and is rejected by MVP validation
  bundle recipe_id that differs from the MVP mapping is rejected
  recipe kernel_check_profile/eq_family/nat_family/limit values that differ from the MVP row are rejected
  recipe rules that differ from the referenced simp profile are rejected
  recipe_id is dropped before Phase 5 MachineTacticOptionsRequest validation
  std.none is not emitted by MVP bundles
  EqFamilyRef/NatFamilyRef options use the Phase 6 Option tag plus family payload bytes, not full MachineTacticOptions bytes
  emitted MVP recipes use explicit std.logic.eq-family, not eq_family = null
  emitted MVP artifacts expose Eq/Nat through verified Std.Logic/Std.Nat exports, not Phase 6 builtin metadata
  std.logic.eq-family resolves Eq / Eq.refl / Eq.rec through direct public Std.Logic exports
  missing or non-public Eq.rec rejects the release before recipe validation
  stale or cross-parent Eq/Nat family exports reject the release before recipe validation
  an ordinary public theorem export named Eq.refl rejects the release because Eq.refl is the generated constructor export
  emitted MVP recipes keep nat_family = null and therefore do not enable induction-nat
  duplicate or non-renderable universe params in theorem-index-visible public ExportEntry verifier output reject the release

certificate binding:
  changing one theorem proof changes certificate_hash and invalidates stale manifest
  release import graph construction uses the Phase 2 canonical certificate decoder, not sidecars, debug JSON, or source files
  release certificates are verified in topological ImportEntry dependency order with ModuleName canonical tie-breaks
  cyclic release module imports are rejected as InvalidStdLibraryRelease

export binding:
  changing a theorem type changes decl_interface_hash and invalidates theorem index entry

import bundle closure:
  missing or extra MVP bundle ids are rejected
  MVP bundles are emitted in the concrete bundle_id dictionary order
  MachineStdImportBundle rejects an emitted machine_std_import_bundle_hash field as unknown
  import_bundles_hash is computed from verifier-recomputed per-bundle digests, not from a stored bundle hash field
  missing dependency, extra dependency, duplicate root import key, duplicate closure key are rejected
  reordered root_imports or import_closure arrays are rejected instead of being sorted before bundle hashing
  embedded certificate.encoding mismatch and malformed certificate.bytes hex are rejected as InvalidStdImportBundle
  embedded import_closure certificate byte/hash mismatches are rejected as InvalidStdImportBundle
  MVP bundles with non-empty allow_axioms are rejected as InvalidStdImportBundle
  allow_axioms rejects current_module/source_index variants and unresolved imported refs
  allow_axioms rejects duplicate resolved axiom identities and non-canonical order
  ordinary Core/prelude ImportEntry values are rejected rather than excluded from MachineStdImportBundle.import_closure
  verifier-internal prelude dependencies typed outside Phase 2 ImportEntry are not emitted as bundle certificates
  std.list.mvp has root_import membership {Std.Logic, Std.List} and closure membership {Std.Logic, Std.Nat, Std.List}
  emitted root_imports and import_closure arrays are sorted by each request record's canonical tuple
  import_closure certificate bytes match the corresponding Std/*.npcert module artifact byte-for-byte
  Std.Logic is a direct root whenever an emitted recipe/profile must resolve std.logic.eq-family

no axiom:
  any axiom dependency in Std.Logic / Std.Nat / Std.List / Std.Algebra.Basic rejects release
  MachineStdModuleArtifact.axiom_report_hash is the module-level Phase 2 hash and mismatches reject as InvalidStdLibraryRelease
  MachineStdAxiomReport.axiom_report_hash is the release-wide sidecar hash and self-mismatches reject as InvalidStdAxiomPolicy
  imported axiom dependencies project to the imported module/export_hash, not the dependent owner module
  axiom report modules exactly match release modules
  reordered or duplicate axiom report modules are rejected as InvalidStdAxiomPolicy
  duplicate or non-canonical module_axioms/transitive_axioms arrays are rejected as InvalidStdAxiomPolicy
  module_axioms and transitive_axioms are both empty for every MVP module
  transitive_axioms mismatch with verifier-derived import closure rejects release

simp profile:
  std.logic.simp is empty; Eq.refl is not emitted as a SimpRuleRef in MVP
  MVP simp profiles are emitted in the concrete profile_id dictionary order
  Nat.add_zero is accepted as SimpSafe
  Nat.add_comm is rejected from SimpSafe and allowed only as RwOnly
  Nat.add_assoc is rejected from SimpSafe by the fixed associativity recognizer
  Nat.mul_succ is accepted as SimpSafe only through the fixed size/head-introduction exceptions
  List.length_nil and List.length_cons are accepted as SimpSafe only through fixed head-introduction exceptions
  List.length_append is accepted as RwOnly but rejected from simp profiles
  head-introduction exceptions compare introduced head set difference by MachineStdGlobalRefView canonical bytes, not display names
  std.nat.simp membership exactly matches the MVP list
  std.list.simp does not include Nat rules because Nat is not a direct root import of std.list.mvp
  std.all.simp is the semantic union of validated std.nat.simp and std.list.simp, emitted in SimpRuleKey canonical order
  std.all.simp rules re-resolve in std.all.mvp to the same MachineStdGlobalRef targets as their source profiles
  identical SimpRuleKey bytes that resolve to different source-profile targets are rejected instead of being deduped
  a simp profile rule without a matching paired SimpSafe rewrite descriptor is rejected
  a rule with only a paired RwOnly descriptor is rejected from simp profiles
  non-renderable SimpRuleRef or EqFamilyRef names are rejected in simp profiles
  missing or extra MVP simp profile ids are rejected
  ambiguous SimpRuleRef resolution is rejected instead of choosing one matching theorem
  duplicate SimpRuleKey inside a MachineStdSimpProfile is rejected
  non-canonical SimpRuleKey order and extra/missing profile rules are rejected
  stale MachineStdSimpProfile.profile_hash is rejected before computing simp_profiles_hash
  simp_profiles_hash is computed from recomputed per-profile digests, not from trusted JSON profile_hash values

Phase 5 handoff:
  std.nat.mvp import bundle can be copied into /machine/sessions
  recommended std.nat.simp recipe validates as Phase 5 MachineTacticOptions

theorem index:
  theorem index_profile_id mismatch is rejected
  theorem index entries in non-canonical global_ref order are rejected
  duplicate theorem index global_ref entries are rejected
  Nat.add_zero entry has exact/rw/simp modes
  List.append_assoc has rw but not simp mode
  all entries carry module/name/export_hash/certificate_hash/decl_interface_hash
  non-renderable theorem-index entry global_ref.name rejects the theorem index
  theorem entry kind is derived from public ExportEntry.kind
  theorem entry kind mismatch with public ExportEntry.kind rejects the theorem index
  sidecar universe_params mismatch, reorder, duplicate, or invalid MachineUniverseParamName rejects the theorem index
  statement_core_hash equals the Phase 2 ExportEntry.type_hash
  statement_core_hash mismatch rejects the theorem index
  axiom_dependencies equals the projection of public ExportEntry.axiom_dependencies
  unprojectable verifier-derived ExportEntry.axiom_dependencies rejects as InvalidStdAxiomPolicy before theorem index comparison
  axiom_dependencies mismatch rejects the theorem index
  statement_head peels leading Pi from ExportEntry.type without WHNF/reduction
  statement_head mismatch rejects the theorem index
  constants include global refs in binder domains and conclusion, then sort/dedup by MachineStdGlobalRefView bytes
  constants mismatch rejects the theorem index
  non-canonical constants/rewrite_descriptors/axiom_dependencies arrays are rejected instead of being sorted before index_hash
  proof_term_size is None for every MVP entry
  non-null proof_term_size is rejected
  duplicate or non-canonical modes/attributes reject the theorem index
  Simp/Rw/Apply attributes agree exactly with derived modes; Intro/Elim/Refl/Trans/Congr are absent in MVP
  rw/simp modes are derived from the union of validated profiles
  theorem entry rewrite_descriptors equals the union of matching rewrite profile descriptors
  omitting one public theorem export rejects the theorem index
  adding a generated constructor as a theorem index entry rejects the theorem index
  imported generated constructor/recursor refs normalize to Generated, not Decl
  imported generated constructor/recursor refs always have public_export = true or reject normalization
  private local declaration references in theorem types normalize with owner module hashes and public_export = false
  private local generated refs normalize with owner module hashes, verifier-reconstructed generated interface hash, and public_export = false
  generated constructor/recursor public_export is true only when a matching public generated ExportEntry exists
  generated constructor/recursor parent_name and parent_decl_interface_hash are derived from the unique parent InductiveDecl
  theorem types containing a global ref not normalizable to MachineStdGlobalRefView::Decl/Generated reject the theorem index

rewrite descriptor:
  MVP rewrite profiles are emitted in the concrete profile_id dictionary order
  lhs_core_hash/rhs_core_hash/param type hashes use Phase 2 term_hash payloads, not Rust ExprId, TermId, JSON, or MachineExprView bytes
  open ResolvedSimpRule fragments are hashed with BVars relative to ResolvedSimpRule.rule_telescope order
  ResolvedRuleParam.ty is hashed in its binder prefix context and rejects BVars reaching itself or later parameters
  lhs_core_hash/rhs_core_hash use ResolvedSimpRule.theorem_lhs/theorem_rhs, not from_pattern/to_pattern
  descriptors with same source/direction/safety/lhs/rhs but different rule_telescope_hash sort deterministically
  rule_telescope_hash ignores ResolvedRuleParam.name and uses zero-based rule_telescope position plus type hash
  rule_telescope_hash is Phase 6-specific and does not reuse the Phase 4 SimpRegistry telescope hash
  invalid MachineUniverseParamName in ResolvedSimpRule.universe_params rejects the rewrite profile
  missing or extra MVP rewrite profile ids are rejected
  ambiguous MVP rewrite membership table names are rejected before descriptor comparison
  non-canonical rewrite descriptor order and extra/missing profile descriptors are rejected
  MVP RwOnly exact set is normative; non-listed not-simp theorems such as Nat.mul_comm or List.map_comp are rejected from MVP rewrite profiles
  MVP rewrite profiles emit only Forward descriptors
  rewrite descriptor sources whose axiom_dependencies are outside the profile bundle allow_axioms are rejected
  every MVP rewrite descriptor source has axiom_dependencies = []
  std.all.rw is the semantic union of validated std.nat.rw and std.list.rw, emitted in descriptor canonical order
  std.all.rw descriptors revalidate under std.all.mvp and reproduce the same descriptor canonical bytes
  non-renderable EqFamilyRef names are rejected in rewrite profiles
  stale MachineStdRewriteProfile.profile_hash is rejected before computing rewrite_profiles_hash
  rewrite_profiles_hash is computed from recomputed per-profile digests, not from trusted JSON profile_hash values

axiom refs:
  Phase 6 axiom reports contain MachineStdAxiomRef, not CurrentModule/source_index refs
  owner-module GlobalRef::Imported axiom refs resolve only through the imported module ExportBlock, not private declaration tables
  private imported axiom dependencies are projected from that imported module's own GlobalRef::Local axiom report entries

prompt metadata:
  absent prompt metadata is valid
  metadata_profile_id/library_profile_id/prompt_metadata_hash mismatches are rejected
  prompt metadata may be empty or a strict subset of theorem index entries
  missing prompt metadata for a theorem index entry is valid
  prompt metadata entries in non-canonical global_ref order are rejected
  duplicate prompt metadata global_ref is rejected
  stale prompt metadata global_ref target is rejected
  prompt metadata library_profile_id mismatch is InvalidStdPromptMetadata, not InvalidStdLibraryRelease
  prompt example goal_core_hash is generated in the canonical Phase 5 direct-import context for imports_bundle_id
  prompt example GlobalRef::Imported indices are assigned from root_imports, not import_closure
  prompt examples that require direct references to closure-only modules are rejected by the generator, not recomputed by the release validator
  prompt examples are hashed in emitted JSON array order, and validators do not sort or dedup examples
  prompt example with unknown imports_bundle_id is rejected
  prompt example with invalid candidate_kind is rejected
  prompt example goal_core_hash is a closed target Expr hash, not a hash of display text
  malformed goal_core_hash is rejected, but semantically stale goal_core_hash cannot be detected by the MVP release validator
  prompt tags in non-canonical ASCII order are rejected
  unknown prompt tag is rejected
```

---

# 17. マイルストーン

Phase 6 AI Profile は、標準ライブラリを一度に「賢く」する工程ではありません。
certificate-bound な最小 release を先に固定し、その上に import bundle、検索 index、simp/rw metadata、
Phase 5 / Phase 7 / Phase 8 連携を順に載せます。
各マイルストーンは、前段の canonical artifact と hash を壊さずに次段へ渡せることを完了条件にします。

```text
M0. Human / AI profile boundary fixed
M1. Certificate release loader
M2. Release manifest and axiom policy
M3. Import bundle closure generator
M4. Theorem index base generator
M5. Rewrite and simp profile generator
M6. Theorem index metadata finalizer
M7. Phase 5 session handoff
M8. Phase 7 retrieval fixtures
M9. Phase 8 audit hooks
```

## M0. Human / AI profile boundary fixed

目的:

```text
- doc/phase6-human.md を人間向け標準ライブラリ設計として固定する
- doc/phase6-ai.md は machine artifact / wire contract / validation order の正本にする
- source text、pretty statement、attribute sidecar、ranking、prompt metadata が trusted payload ではないことを明確にする
```

成果物:

```text
- Phase 6 Human Profile と AI Profile の責務分離
- MVP module set:
    Std.Logic
    Std.Nat
    Std.List
    Std.Algebra.Basic
- fixed package locator path table
```

完了条件:

```text
- MVP module membership と canonical module order が文書上固定されている
- optional prompt metadata が std_library_release_hash に入らないことが文書上固定されている
- trusted / untrusted boundary が Phase 2 / Phase 5 / Phase 7 / Phase 8 と矛盾しない
```

## M1. Certificate release loader

目的:

```text
- Std/*.npcert を唯一の canonical source of truth として読み込む
- fixed locator path から raw Phase 2 certificate bytes を取得する
- Phase 2 verifier を high-trust mode で呼び、module artifact に必要な verifier output を得る
```

成果物:

```text
- package root + fixed POSIX relative path の locator validator
- raw .npcert reader
- Phase 2 canonical certificate decoder integration
- high-trust verifier integration
- ModuleName -> verified certificate context table
```

完了条件:

```text
- missing / extra module、path mismatch、non-canonical module order を InvalidStdLibraryRelease として拒否できる
- absolute path、..、.、backslash、duplicate slash、trailing slash、symlink escape を拒否できる
- each certificate の export_hash / certificate_hash / module-level axiom_report_hash を再計算できる
- import graph を certificate ImportEntry だけから構成し、Core/prelude ImportEntry を ordinary import として拒否できる
- topological dependency order + ModuleName canonical tie-break で検証できる
```

## M2. Release manifest and axiom policy

目的:

```text
- MachineStdLibraryRelease と MachineStdModuleArtifact の canonical bytes / hash を実装する
- release-wide no-custom-axiom policy を verifier output から検査する
- sidecar self-hash と manifest-bound hash comparison を分離する
```

成果物:

```text
- Std.machine-release.json parser / validator
- MachineStdLibraryRelease canonical bytes
- std_library_release_hash computation
- MachineStdAxiomReport parser / validator
- release-wide axiom_report_hash computation
```

完了条件:

```text
- protocol_version / library_profile_id / core_spec_id / kernel_semantics_profile_id mismatch を拒否できる
- MachineStdLibraryRelease が std_library_release_hash field を持つ場合に unknown field として拒否できる
- module_axioms と transitive_axioms が MVP では全 module で空であることを検査できる
- stale MachineStdAxiomReport.axiom_report_hash を InvalidStdAxiomPolicy として manifest comparison 前に拒否できる
- manifest-bound sidecar hash mismatch を InvalidStdLibraryRelease として分類できる
```

## M3. Import bundle closure generator

目的:

```text
- Phase 5 /machine/sessions にそのまま渡せる import bundle を生成する
- direct roots と transitive closure を certificate-bound identity で固定する
- allow_axioms と recipe_id mapping を MVP profile に合わせて固定する
```

成果物:

```text
- MachineStdImportBundleSet generator / validator
- std.logic.mvp
- std.nat.mvp
- std.list.mvp
- std.algebra-basic.mvp
- std.all.mvp
- import_bundles_hash computation
```

完了条件:

```text
- missing / extra / duplicate bundle id と non-canonical bundle order を拒否できる
- root_imports / import_closure が canonical tuple order であることを検査できる
- import_closure certificate bytes が Std/*.npcert と byte-for-byte 一致することを検査できる
- extra dependency、missing dependency、duplicate root / closure key を拒否できる
- MVP allow_axioms が [] であることを検査できる
- bundle-to-recipe mapping が固定表と一致することを検査できる
```

## M4. Theorem index base generator

目的:

```text
- public theorem / axiom ExportEntry から certificate-derived theorem index を作る
- theorem identity を module / name / export_hash / certificate_hash / decl_interface_hash に固定する
- profile-derived metadata なしで確定できる fields だけを先に生成する
```

成果物:

```text
- MachineStdTheoremIndex base generator
- MachineStdGlobalRef canonical bytes / order
- MachineStdTheoremEntry certificate-derived fields:
    kind
    universe_params
    statement_core_hash
    statement_head
    constants
    axiom_dependencies
    proof_term_size = None
- exact / apply base mode derivation
```

完了条件:

```text
- theorem index entries が release module の public theorem / axiom ExportEntry と exact match する
- generated constructor / recursor、private entry、extra entry、missing entry を拒否できる
- statement_core_hash が Phase 2 ExportEntry.type_hash と一致する
- statement_head / constants / axiom_dependencies を verifier output から決定的に再構成できる
- rw/simp modes、Simp/Rw attributes、rewrite_descriptors をこの段階では final にしない
- final theorem_index_hash は M6 まで emit しない
```

## M5. Rewrite and simp profile generator

目的:

```text
- Phase 4 ResolvedSimpRule を Phase 6 の rewrite / simp sidecar に投影する
- lhs/rhs core hash と MachineStdRuleTelescope hash を certificate context で決定的に計算する
- unsafe な rewrite と simp-safe な rule を profile id ごとに分離する
```

成果物:

```text
- MachineStdRewriteProfileSet generator / validator
- MachineStdSimpProfileSet generator / validator
- std.logic.rw / std.nat.rw / std.list.rw / std.all.rw
- std.logic.simp / std.nat.simp / std.list.simp / std.all.simp
- rewrite_profiles_hash computation
- simp_profiles_hash computation
```

完了条件:

```text
- MVP profile ids と required_import_bundle_id / kernel_check_profile / eq_family が固定表と一致する
- lhs_core_hash / rhs_core_hash が Phase 2 term_hash payload から計算される
- rule_telescope_hash が Phase 6-specific rule_telescope bytes から計算される
- SimpSafe / RwOnly membership が MVP の固定集合と一致する
- std.all.rw と std.all.simp が source profiles の semantic union として再検証できる
- stale per-profile profile_hash を set hash 計算前に拒否できる
```

## M6. Theorem index metadata finalizer

目的:

```text
- validated rewrite/simp profiles から theorem index の derived metadata を確定する
- rw/simp modes、Simp/Rw/Apply attributes、rewrite_descriptors を theorem entry に反映する
- MVP で未使用の attributes を拒否する
```

成果物:

```text
- theorem index finalizer
- finalized modes
- finalized attributes
- finalized rewrite_descriptors
- theorem_index_hash computation
```

完了条件:

```text
- modes / attributes / rewrite_descriptors が validated profiles の結果と一致する
- duplicate / non-canonical modes、attributes、rewrite_descriptors を拒否できる
- Intro / Elim / Refl / Trans / Congr attributes が MVP artifact に出た場合に拒否できる
- theorem_index_hash self-mismatch を InvalidStdTheoremIndex として拒否できる
- MachineStdModuleArtifact.theorem_index_entry_count と simp_rule_count を final sidecar から検査できる
```

## M7. Phase 5 session handoff

目的:

```text
- import bundle と recommended_tactic_options recipe を Phase 5 API で再検証する
- Phase 6 recipe を trusted state としてではなく Phase 5 request payload として扱う
- stale recipe reference を Phase 5 option validation で検出する
```

成果物:

```text
- import bundle recipe finalizer
- MachineStdTacticOptionsRecipe canonical bytes
- Phase 5 MachineTacticOptionsRequest projection
- /machine/sessions integration tests
```

完了条件:

```text
- recipe_id を drop した payload が Phase 5 MachineTacticOptionsRequest として検証される
- SimpRuleRef / EqFamilyRef / NatFamilyRef が bundle direct import scope で解決される
- emitted recipe simp_rules が referenced simp profile の canonical rules と一致する
- std.logic.eq-family が verified Std.Logic exports に bind される
- MVP recipes が nat_family = null を保ち、induction-nat を有効化しない
```

## M8. Phase 7 retrieval fixtures

目的:

```text
- Phase 7 が Phase 6 metadata を使って premise retrieval / candidate generation を再現できることを確認する
- AI が生成した候補を必ず Phase 5 run/batch/replay/verify に戻す境界を固定する
```

成果物:

```text
- Nat / List basic goal fixtures
- exact candidate fixtures
- rw candidate fixtures
- simp candidate fixtures
- query_fingerprint / theorem_index_fingerprint regression tests
```

完了条件:

```text
- 同一 release artifact から同一 candidate source set を再現できる
- theorem ranking や prompt text を certificate hash / std_library_release_hash に入れない
- Phase 7 candidate が Phase 5 /machine/tactics/run または /machine/tactics/batch なしに採用されない
- stale global_ref / decl_interface_hash を持つ candidate が Phase 5 validation で拒否される
```

## M9. Phase 8 audit hooks

目的:

```text
- independent checker / audit layer が Phase 6 sidecar と verifier output の一致を再検査できるようにする
- trusted base を広げずに machine artifact の再現性を監査する
```

成果物:

```text
- Phase 8 audit checklist implementation
- sidecar vs verifier output comparison
- manifest-bound sidecar hash audit
- optional prompt metadata exclusion audit
```

完了条件:

```text
- release manifest hashes が certificate bytes と validated sidecar hashes に一致することを監査できる
- optional prompt metadata が std_library_release_hash から除外されていることを監査できる
- every decl_interface_hash / export_hash / certificate_hash が verifier output と一致することを監査できる
- every simp / rewrite profile target が matching decl_interface_hash に解決されることを監査できる
- import bundles が minimal transitive closure であり、constructive bundle の allow_axioms が空であることを監査できる
```

## Milestone dependency graph

```text
M0
  ↓
M1
  ↓
M2
  ├── M3
  └── M4
      ↓
M3 + M4
  ↓
 M5
  ↓
 M6
  ↓
M3 + M6
  ↓
 M7
  ↓
 M8
  ↓
 M9
```

M3 は M2 の manifest / module artifact table に依存します。
M4 は M2 の verified module context table に依存します。
M5 は M3 の import bundle identity、M4 の theorem identity、Phase 4 `ResolvedSimpRule` に依存します。
M7 は M3 の import bundle と M6 の finalized theorem / simp metadata を Phase 5 に引き渡します。
M8 以降は Phase 5 で再検証できる candidate source を前提に、Phase 7 / Phase 8 の利用面を固定します。

---

# 18. 入れないもの

MVP では次を入れません。

```text
- semantic embedding vectors as canonical artifact
- usage_count based ranking as canonical artifact
- server-side package download / module resolution
- source text based theorem identity
- attribute-driven trusted proof acceptance
- global transitive theorem search across unimported modules
- automatic import insertion by AI server
- classical axioms in constructive std bundle
- full simp, ring, omega, linarith metadata
- theorem minimization hints as trusted data
```

Embeddingや usage statistics は Phase 7 / Phase 9 の非信頼 ranking sidecar として追加できます。
その場合も `std_library_release_hash` とは別の artifact hash / profile id を持たせます。

---

# 19. 完了条件

Phase 6 AI Profile が完了したと言える条件はこれです。

```text
- standard library certificates を high-trust mode で検査できる
- release manifest が certificate bytes / export_hash / certificate_hash に固定されている
- Phase 5 に渡せる import bundle を生成できる
- no-custom-axiom policy を axiom report から検査できる
- theorem index entries が decl_interface_hash と export_hash に固定されている
- simp / rw metadata が Phase 4 rule validation と一致している
- recommended tactic options recipe が Phase 5 で再検証される
- Phase 7 が metadata を使って候補を作っても、Phase 5 run/batch なしには採用されない
- Phase 8 が sidecar と certificate verifier output の一致を監査できる
```

---

# 20. 一文でまとめると

Phase 6 AI Profile は、**標準ライブラリを AI 探索が安全に使える certificate-bound machine artifact set として公開するための非信頼 metadata 層**です。

中核は次です。

```text
verified release manifest:
  Std modules の certificate hash と artifact hash を固定する

import bundles:
  Phase 5 session create に渡せる closure payload を決定的に作る

machine theorem index:
  premise を module/name/export_hash/certificate_hash/decl_interface_hash に固定する

simp / rewrite profiles:
  AI が安全な候補を作りやすい metadata を提供し、最終的には Phase 5 が再検証する
```
