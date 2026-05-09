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
- theorem search / simp-lite / rw 用 metadata を certificate hash に固定する
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
unknown field は artifact ごとの validation error として拒否します。

MVP JSON root shape は次に固定します。
top-level array は使いません。
各 root object に含まれる hash field は、その artifact の canonical bytes から再計算した値と一致しなければなりません。

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

MVP JSON strings for scalar enum fields are fixed as follows.
Tagged-object enums, such as `MachineStdGlobalRefView`, define their own JSON object shape in their section.
Any other spelling, casing, alias, numeric enum tag, or object wrapper is invalid.

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
`certificate_bytes_hash` は raw Phase 2 certificate bytes の `sha256` です。
`expected_certificate_hash` は Phase 2 certificate hash であり、`certificate_bytes_hash` と同じとは限りません。
両方を混同してはいけません。
`public_export_count` は Phase 2 verifier output の public `ExportEntry` 総数です。
`theorem_index_entry_count` はその module の public `TheoremDecl` / `AxiomDecl` entry 数です。
`simp_rule_count` は全 `MachineStdSimpProfile` を検証した後、その module の theorem entry に解決される unique
`SimpRuleRef` 数です。
同じ `SimpRuleKey` が複数 profile に出ても 1 件として数えます。

Manifest validation は各 module certificate を Phase 2 verifier で high-trust mode として検査し、
recomputed `export_hash` / `certificate_hash` / `axiom_report_hash` が manifest と一致することを確認します。
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

MVP bundle root imports and closures:

```text
std.logic.mvp:
  root_imports = [Std.Logic]
  import_closure = [Std.Logic]

std.nat.mvp:
  root_imports = [Std.Nat]
  import_closure = [Std.Logic, Std.Nat]

std.list.mvp:
  root_imports = [Std.List]
  import_closure = [Std.Logic, Std.Nat, Std.List]

std.algebra-basic.mvp:
  root_imports = [Std.Algebra.Basic, Std.Logic]
  import_closure = [Std.Logic, Std.Algebra.Basic]

std.all.mvp:
  root_imports = [Std.Algebra.Basic, Std.List, Std.Logic, Std.Nat]
  import_closure = [Std.Algebra.Basic, Std.List, Std.Logic, Std.Nat]
```

The lists above are semantic sets.
The emitted `root_imports` and `import_closure` arrays are still sorted by `(module, export_hash, certificate_hash)` canonical order.
`Core` is not emitted as a Phase 6 verified module certificate.
It is part of the kernel/core profile, not a standard-library import bundle artifact.
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
Changing this mapping changes `machine_std_import_bundle_hash`.

`root_imports` は Phase 5 `/machine/sessions.imports` に渡す direct import key list です。
`import_closure` は Phase 5 `/machine/sessions.import_closure` に渡す complete certificate payload set です。
`import_closure` は `root_imports` から certificate `ImportEntry` をたどって到達する最小 transitive closure と完全一致しなければなりません。
extra certificate、欠落 dependency、duplicate closure key は invalid bundle です。
Every `VerifiedModuleCertificateRequest` in `import_closure` must carry the same canonical certificate bytes as the
corresponding module artifact in `Std/*.npcert`.
The raw bytes hash must equal `MachineStdModuleArtifact.certificate_bytes_hash`, and the request
`expected_export_hash` / `expected_certificate_hash` must equal the same module artifact's
`expected_export_hash` / `expected_certificate_hash`.
An import bundle must not contain a byte-different re-encoding, alternate certificate, or stale copy for a release module.

`allow_axioms` は MVP 標準ライブラリでは必ず `[]` です。
将来 `Std.Classical` を追加する場合は別 bundle id にし、`allow_axioms` に入る axiom を明示します。
`allow_axioms` entries use Phase 5 `MachineAxiomRefWire` JSON, but Phase 6 import bundles may contain only the
`kind = "imported"` variant.
`kind = "current_module"` or any `source_index`-based axiom coordinate is invalid in a Phase 6 import bundle because the bundle
is release-global and has no checked-current declaration context.
Every non-empty future `allow_axioms` entry must resolve, by Phase 5 axiom option validation, to a unique `AxiomDecl` in the
bundle's verified import closure.
constructive MVP bundle が classical axiom を transitively import してはいけません。
`recommended_tactic_options` must validate as Phase 5 `MachineTacticOptionsRequest` against exactly this bundle's
`root_imports` / `import_closure` and an empty checked-current-declaration list.
This Phase 5 validation uses the recipe payload after dropping `recipe_id`; `recipe_id` is Phase 6 metadata and is not a
Phase 5 `MachineTacticOptionsRequest` field.
Every `SimpRuleRef`, `EqFamilyRef`, and `NatFamilyRef` must resolve within the bundle's direct import scope using Phase 5 option
validation rules.
Unknown, stale, or ambiguous recipe references make the bundle `InvalidStdImportBundle`.

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

MVP emitted recipe contents:

```text
std.logic-basic:
  kernel_check_profile = "npa.kernel.v0.1.builtin-eq-nat"
  simp_rules = rules from MachineStdSimpProfile "std.logic.simp"
  eq_family = null
  nat_family = null
  limits = MVP recommended limits

std.nat-simp:
  kernel_check_profile = "npa.kernel.v0.1.builtin-eq-nat"
  simp_rules = rules from MachineStdSimpProfile "std.nat.simp"
  eq_family = null
  nat_family = null
  limits = MVP recommended limits

std.list-simp:
  kernel_check_profile = "npa.kernel.v0.1.builtin-eq-nat"
  simp_rules = rules from MachineStdSimpProfile "std.list.simp"
  eq_family = null
  nat_family = null
  limits = MVP recommended limits

std.all-simp:
  kernel_check_profile = "npa.kernel.v0.1.builtin-eq-nat"
  simp_rules = rules from MachineStdSimpProfile "std.all.simp"
  eq_family = null
  nat_family = null
  limits = MVP recommended limits
```

The referenced simp profile must be present in the same release.
The recipe embeds the profile's canonicalized `rules` list, not the profile hash.
If the recipe rules differ from the referenced profile rules after Phase 4 `SimpRuleKey` sort/dedup,
the containing bundle is `InvalidStdImportBundle`.

These are semantic tactic options, not per-run deterministic budget.
Phase 7 can override them by choosing a different Phase 5 session, but the resulting `session_root_hash` changes.

`simp_rules` は Phase 4 `SimpRuleKey canonical order` で sort/dedup します。
duplicate count と input order は recipe identity に入りません。
`eq_family = null` は Phase 5 request の `eq_family: null` と同じ意味です。
つまり `kernel_check_profile = "npa.kernel.v0.1.builtin-eq-nat"` なら Phase 4 は kernel builtin Eq を default Eq family として解決でき、
`kernel_check_profile = "npa.kernel.v0.1.builtin-none"` なら resolved Eq family は `None` になります。
`EqFamilyRef` / `NatFamilyRef` を `Some` にする場合は Phase 5 wire field order と Phase 4 canonical bytes をそのまま使います。

```text
MachineStdTacticOptionsRecipe canonical bytes:
  - tag "npa.phase6.std-tactic-options-recipe.v1"
  - recipe_id
  - kernel_check_profile canonical bytes as Phase 5 KernelCheckProfileId
  - simp_rules in Phase 4 SimpRuleKey canonical order
  - eq_family option as Phase 4 EqFamilyRef canonical bytes
  - nat_family option as Phase 4 NatFamilyRef canonical bytes
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
TheoremDecl
AxiomDecl
```

`DefDecl`, `InductiveDecl`, constructors, recursors, and private dependencies are not theorem index entries in MVP.
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

MVP theorem index is complete for the release modules.
For every module listed in `MachineStdLibraryRelease.modules`, the validator recomputes the set of public `ExportEntry`
items whose declaration kind is `TheoremDecl` or `AxiomDecl`.
`MachineStdTheoremIndex.entries` must contain exactly that set, no more and no less.
Missing public theorem/axiom entries, extra private entries, generated constructor/recursor entries, or stale entries are
`InvalidStdTheoremIndex`.
`MachineStdModuleArtifact.theorem_index_entry_count` must equal the recomputed per-module count.

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

`entries` は `MachineStdGlobalRef canonical order` で sort し、同じ
`module / name / export_hash / certificate_hash / decl_interface_hash` を重複して持ってはいけません。

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
Invalid `MachineUniverseParamName` is `InvalidStdTheoremIndex`.
`axiom_dependencies` is the exact sorted/dedup projection of the same public `ExportEntry.axiom_dependencies` field to
`MachineStdAxiomRef`.
It is not derived from prompt metadata, theorem attributes, source text, proof pretty-printing, or
`AxiomReport.per_declaration`.
If the Phase 2 verifier exposes both `ExportEntry.axiom_dependencies` and a corresponding per-declaration transitive axiom
report, an implementation may cross-check them, but the theorem index field is sourced from `ExportEntry.axiom_dependencies`.
`attributes`, `rewrite_descriptors`, and `proof_term_size` are non-trusted metadata.
MVP must set `proof_term_size = None` for every entry.
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

`GlobalRef::Imported` は owner certificate の import table から `(module, export_hash, certificate_hash)` を解決し、
その imported module の verifier output で `name / decl_interface_hash` を引きます。
`GlobalRef::Local` は owner module の verifier output で引き、`public_export` はその declaration が owner module の
public `ExportEntry` に出る場合だけ true です。
constructor / recursor のような generated declaration は `Generated` に正規化します。
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
`MachineStdAxiomRef canonical bytes` は Phase 5 `MachineAxiomRefWire::Imported` canonical bytes と byte-for-byte 同じです。
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
For MVP std profiles, `eq_family = null` and `kernel_check_profile = "npa.kernel.v0.1.builtin-eq-nat"` select the Phase 4 builtin Eq family.
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

`lhs_core_hash`, `rhs_core_hash`, and `rule_telescope_hash` must be derived by the same Phase 4 simp / rw rule validation that builds `ResolvedSimpRule`.
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
  sha256(Phase 1 Expr canonical bytes of ResolvedSimpRule.theorem_lhs)

rhs_core_hash:
  sha256(Phase 1 Expr canonical bytes of ResolvedSimpRule.theorem_rhs)

MachineStdRuleTelescope canonical bytes:
  - tag "npa.phase6.std-rule-telescope.v1"
  - universe_params in ResolvedSimpRule.universe_params order:
      MachineUniverseParamName as String
  - rule_telescope in ResolvedSimpRule.rule_telescope order:
      param position as minimal unsigned LEB128 u64
      param ty hash = sha256(Phase 1 Expr canonical bytes of ResolvedRuleParam.ty)

rule_telescope_hash:
  sha256(MachineStdRuleTelescope canonical bytes)
```

`lhs_core_hash` and `rhs_core_hash` are the theorem's original equality sides.
They are not direction-adjusted `from_pattern` / `to_pattern`.
`direction` is represented only by the descriptor's `direction` field.
`ResolvedRuleParam.name` is display/debug metadata and is not included in `rule_telescope_hash`.

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
  descriptors = std.nat.rw descriptors + std.list.rw descriptors
```

`descriptors` は `source` の `MachineStdGlobalRef canonical order`、`direction`、`safety`、
`lhs_core_hash`、`rhs_core_hash`、`rule_telescope_hash` の順で sort し、完全重複を許しません。
`kernel_check_profile` と `eq_family` は descriptor validation に使う Eq selection です。
MVP std rewrite profiles は `kernel_check_profile = "npa.kernel.v0.1.builtin-eq-nat"` かつ `eq_family = null` です。
これは Phase 4 の builtin Eq default を選ぶという意味であり、「Eq family なし」ではありません。
Every MVP rewrite descriptor listed above has `direction = Forward`.
MVP rewrite profiles do not emit `Backward` descriptors.
AI clients that want a backward rewrite must request a Phase 5 `rw` tactic with `direction = "backward"` and pass Phase 5 validation;
the Phase 6 rewrite profile does not pre-authorize it as a separate descriptor.
Every descriptor source must resolve to a public theorem in the direct import scope of `required_import_bundle_id`.
For example, `std.list.rw` may reference `Std.List` theorem exports but must not include `Nat.add_zero`, because `Std.Nat`
is only transitive closure for `std.list.mvp`, not a direct root import.
If the exact membership above names a theorem that is absent from the checked standard-library certificates, the release is
`InvalidStdRewriteProfile`.
`profile_hash` は次で計算します。

```text
MachineStdRewriteProfile canonical bytes:
  - tag "npa.phase6.std-rewrite-profile.v1"
  - profile_id
  - required_import_bundle_id
  - kernel_check_profile canonical bytes as Phase 5 KernelCheckProfileId
  - eq_family option as Phase 4 EqFamilyRef canonical bytes
  - descriptors in canonical order:
      MachineStdRewriteDescriptor canonical bytes

profile_hash:
  sha256(MachineStdRewriteProfile canonical bytes with profile_hash omitted)
```

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
  - traverse the RHS head symbol set and normalize every new head to `MachineStdGlobalRefView`
    using the source theorem's owner module certificate context
  - resolve each allowed head label in the release module table:
      Nat.add  -> the unique `Std.Nat` public export named `Nat.add`
      Nat.zero -> the unique `Std.Nat` public export or generated export named `Nat.zero`
      Nat.succ -> the unique `Std.Nat` public export or generated export named `Nat.succ`
  - compare allowed heads to introduced heads by `MachineStdGlobalRefView canonical bytes`
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
They use Phase 1 Expr canonical bytes for expression equality and flatten only syntactic `App` spines.
No WHNF, unfolding, eta, conversion, associativity normalization, or pretty-name matching is allowed.

```text
flatten_app(e):
  repeatedly peel App(fn, arg) from the outside
  return (head, args) where args are restored in left-to-right application order

same_expr(a, b):
  Phase 1 Expr canonical bytes of a and b are byte-for-byte equal

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
  rules = std.nat.simp rules + std.list.simp rules
```

Rules are emitted in Phase 4 `SimpRuleKey` canonical order.
A `MachineStdSimpProfile.rules` artifact must not contain duplicate `SimpRuleKey` values.
The release generator may sort/dedup an internal candidate list before writing the artifact, but the artifact validator rejects
duplicates instead of silently repairing them.
A profile that includes the same `name / decl_interface_hash / direction` twice is `InvalidStdSimpProfile`.
A profile must not include a theorem whose `axiom_dependencies` are outside the bundle `allow_axioms`.
For MVP constructive std profiles, every rule must have `axiom_dependencies = []`.
MVP std simp profiles use `kernel_check_profile = "npa.kernel.v0.1.builtin-eq-nat"` and `eq_family = null`,
which selects Phase 4 builtin Eq default during validation.
Every MVP `SimpRuleRef` listed above has `direction = "forward"`.
MVP simp profiles do not emit backward simp rules.
Every `SimpRuleRef` must resolve to a public theorem in the direct import scope of `required_import_bundle_id`.
If the exact membership above names a theorem that is absent from the checked standard-library certificates, the release is
`InvalidStdSimpProfile`.

```text
MachineStdSimpProfile canonical bytes:
  - tag "npa.phase6.std-simp-profile.v1"
  - profile_id
  - required_import_bundle_id
  - kernel_check_profile canonical bytes as Phase 5 KernelCheckProfileId
  - eq_family option as Phase 4 EqFamilyRef canonical bytes
  - rules in Phase 4 SimpRuleKey canonical order

profile_hash:
  sha256(MachineStdSimpProfile canonical bytes with profile_hash omitted)
```

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
- metadata must bind to exact module/name/export_hash/certificate_hash/decl_interface_hash
- metadata must not be used to construct proof terms
- tags are lower_ascii strings from a fixed vocabulary
- examples are display-only and must not be copied into suggested_candidates without Phase 5 validation
- metadata is omitted from theorem_index_hash unless a separate prompt_metadata_hash is explicitly used
```

`Std.machine-prompt-metadata.json` is optional in MVP.
It is not referenced by `MachineStdLibraryRelease` and is not included in `std_library_release_hash`.
If emitted, it must validate against the theorem index entry set and carry its own `prompt_metadata_hash`.
Absent prompt metadata is valid.
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

`MachineStdPromptMetadata canonical bytes` include `global_ref`, `short_doc` option, `examples` in source order,
and `tags` sorted lexicographically by ASCII byte value.
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
  sha256(Phase 1 Expr canonical bytes of the closed example goal target Expr)
```

The example goal target is elaborated under `imports_bundle_id` with an empty local context.
Therefore an MVP prompt example target must be closed.
`goal_core_hash` excludes local context, pretty goal text, `display`, `candidate_kind`, `imports_bundle_id`, source span, and prompt text.
Because the prompt metadata artifact does not carry the original goal source or local context, the release validator checks only that
`goal_core_hash` is a valid `HashString` and that `imports_bundle_id` is known.
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
`examples` preserve JSON array order for display only; duplicate examples are allowed because they do not affect trusted proof state.

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
MVP no-custom-axiom validation requires both `module_axioms` and `transitive_axioms` to be `[]` for every release module.
Any mismatch with verifier-derived projection is `InvalidStdAxiomPolicy`.
`module_axioms` is not recomputed from public exports, owned `AxiomDecl` declarations, theorem-index entries, or prompt metadata.
It is derived from the verifier's module-level axiom report payload so imported/private axiom dependencies used by the module
cannot be hidden by export filtering.

`sorry`, `admit`, generated placeholder axioms, imported classical axioms, or private axiom dependencies are release blockers.
They must appear in Phase 2 axiom reports and cause `InvalidStdLibraryRelease`.
Phase 6 axiom reports must never contain Phase 5 `CurrentModule` axiom refs.
All axiom dependencies are projected to `MachineStdAxiomRef` using the owner module and owner `export_hash`.
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

`MachineStdLibraryRelease.axiom_report_hash` はこの `axiom_report_hash` と完全一致しなければなりません。

---

# 12. Validation Order

Release validation order is fixed so build failures are deterministic.

```text
MachineStdRelease validation:
  1. Parse artifact JSON with duplicate-key detection.
     Unknown field, missing required field, null where not allowed, invalid integer/hash/name:
       InvalidStdArtifactShape

  2. Verify every certificate payload with Phase 2 verifier in high-trust mode.
     Recomputed export_hash / certificate_hash / axiom_report_hash mismatch:
       InvalidStdLibraryRelease

  3. Build module context table from verifier output.
     Duplicate module, non-canonical module order, unsupported core spec, ordinary Core/prelude ImportEntry,
     release module ImportEntry that does not resolve to one of the release module artifacts:
       InvalidStdLibraryRelease

  4. Validate no-custom-axiom policy.
     Disallowed axiom, axiom report module-set mismatch, module/transitive axiom projection mismatch:
       InvalidStdAxiomPolicy

  5. Validate import bundle closure and recipe ids.
     Missing dependency, extra closure certificate, root import not in closure, invalid bundle-to-recipe mapping:
       InvalidStdImportBundle

  6. Validate theorem index identity and certificate-derived fields against public ExportEntry set.
     Stale hash, missing export, missing required entry, extra entry, invalid renderable name/universe param:
       InvalidStdTheoremIndex

  7. Validate theorem entry attribute shape.
     Non-canonical order, duplicate attribute, MVP-reserved attribute:
       InvalidStdTheoremIndex

  8. Validate rewrite profiles.
     Unknown descriptor, duplicate descriptor, unsafe profile membership:
       InvalidStdRewriteProfile

  9. Validate simp profiles.
     Unknown rule, duplicate rule, unsafe rule in SimpSafe profile, axiom mismatch:
       InvalidStdSimpProfile

  10. Validate import bundle recommended tactic options against validated simp profiles.
      Recipe/profile rule mismatch, Phase 5 option validation failure:
        InvalidStdImportBundle

  11. Validate theorem index derived metadata against validated profiles.
      modes mismatch, attributes/modes mismatch, rewrite_descriptors mismatch, rewrite shape failure:
        InvalidStdTheoremIndex

  12. Compute all artifact hashes and compare manifest fields.
     Also compare sidecar library_profile_id values, public_export_count, theorem_index_entry_count, and simp_rule_count
     against recomputed values.
     Any mismatch:
       InvalidStdLibraryRelease

  13. If optional prompt metadata is present, validate it against the theorem index and import bundle set.
      Stale global_ref target, duplicate tag, unknown tag, unknown imports_bundle_id, invalid hash:
        InvalidStdPromptMetadata
      The validator checks goal_core_hash shape only; it does not recompute or reject semantically stale goal_core_hash values
      in the MVP profile.
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
    "kernel_check_profile": "npa.kernel.v0.1.builtin-eq-nat",
    "allow_axioms": [],
    "tactic_options": {
      "simp_rules": [],
      "eq_family": null,
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
- release manifest hashes match certificate bytes and sidecar hashes
- theorem index entries point only to public exports
- every decl_interface_hash matches certificate verifier output
- every axiom dependency in index equals the public ExportEntry.axiom_dependencies projection
- axiom report sidecar module_axioms equals verifier-derived AxiomReport.module_axioms projection
- every simp profile rule resolves to a theorem with matching decl_interface_hash
- constructive bundles have empty module_axioms
- import bundles are minimal transitive closures
```

If an index entry is wrong but certificates verify, the proof system is still sound.
The artifact is rejected as bad metadata, not as a kernel failure.

---

# 16. Tests

Phase 6 AI MVP should include these tests.

```text
release determinism:
  same certificate bytes and sidecars produce same std_library_release_hash
  reordered JSON object fields do not change std_library_release_hash
  sidecar top-level arrays are rejected; each sidecar must use the fixed root object schema
  canonical enum bytes use the fixed one-byte tags, not JSON strings
  JSON enum strings use only the fixed wire spellings
  MachineStdGlobalRefView JSON requires kind = decl/generated and rejects mixed variant fields
  MachineStdAxiomRef JSON rejects kind/source_index/certificate_hash fields
  reordered module list is rejected unless it is ModuleName canonical order

recipe determinism:
  duplicate/reordered simp_rules in a recipe canonicalize to the same recipe bytes
  changing kernel_check_profile changes the import bundle hash
  bundle recipe_id that differs from the MVP mapping is rejected
  recipe rules that differ from the referenced simp profile are rejected
  recipe_id is dropped before Phase 5 MachineTacticOptionsRequest validation
  std.none is not emitted by MVP bundles

certificate binding:
  changing one theorem proof changes certificate_hash and invalidates stale manifest

export binding:
  changing a theorem type changes decl_interface_hash and invalidates theorem index entry

import bundle closure:
  missing dependency, extra dependency, duplicate closure key are rejected
  ordinary Core/prelude ImportEntry values are rejected rather than excluded from MachineStdImportBundle.import_closure
  verifier-internal prelude dependencies typed outside Phase 2 ImportEntry are not emitted as bundle certificates
  std.list.mvp has root_imports [Std.List] and closure [Std.Logic, Std.Nat, Std.List]
  import_closure certificate bytes match the corresponding Std/*.npcert module artifact byte-for-byte

no axiom:
  any axiom dependency in Std.Logic / Std.Nat / Std.List / Std.Algebra.Basic rejects release
  axiom report modules exactly match release modules
  module_axioms and transitive_axioms are both empty for every MVP module
  transitive_axioms mismatch with verifier-derived import closure rejects release

simp profile:
  Nat.add_zero is accepted as SimpSafe
  Nat.add_comm is rejected from SimpSafe and allowed only as RwOnly
  Nat.add_assoc is rejected from SimpSafe by the fixed associativity recognizer
  Nat.mul_succ is accepted as SimpSafe only through the fixed size/head-introduction exceptions
  List.length_nil and List.length_cons are accepted as SimpSafe only through fixed head-introduction exceptions
  head-introduction exceptions compare MachineStdGlobalRefView canonical bytes, not display names
  std.nat.simp membership exactly matches the MVP list
  std.list.simp does not include Nat rules because Nat is not a direct root import of std.list.mvp
  duplicate SimpRuleKey inside a MachineStdSimpProfile is rejected

Phase 5 handoff:
  std.nat.mvp import bundle can be copied into /machine/sessions
  recommended std.nat.simp recipe validates as Phase 5 MachineTacticOptions

theorem index:
  Nat.add_zero entry has exact/rw/simp modes
  List.append_assoc has rw but not simp mode
  all entries carry module/name/export_hash/certificate_hash/decl_interface_hash
  theorem entry kind is derived from public ExportEntry.kind
  statement_core_hash equals the Phase 2 ExportEntry.type_hash
  axiom_dependencies equals the projection of public ExportEntry.axiom_dependencies
  statement_head peels leading Pi from ExportEntry.type without WHNF/reduction
  constants include global refs in binder domains and conclusion, then sort/dedup by MachineStdGlobalRefView bytes
  proof_term_size is None for every MVP entry
  duplicate or non-canonical modes/attributes reject the theorem index
  Simp/Rw/Apply attributes agree exactly with derived modes; Intro/Elim/Refl/Trans/Congr are absent in MVP
  rw/simp modes are derived from the union of validated profiles
  theorem entry rewrite_descriptors equals the union of matching rewrite profile descriptors
  omitting one public theorem export rejects the theorem index
  adding a generated constructor as a theorem index entry rejects the theorem index

rewrite descriptor:
  lhs_core_hash/rhs_core_hash use ResolvedSimpRule.theorem_lhs/theorem_rhs, not from_pattern/to_pattern
  descriptors with same source/direction/safety/lhs/rhs but different rule_telescope_hash sort deterministically
  rule_telescope_hash ignores ResolvedRuleParam.name and uses param position plus type hash
  MVP rewrite profiles emit only Forward descriptors

axiom refs:
  Phase 6 axiom reports contain MachineStdAxiomRef, not CurrentModule/source_index refs

prompt metadata:
  absent prompt metadata is valid
  duplicate prompt metadata global_ref is rejected
  stale prompt metadata global_ref target is rejected
  prompt example with unknown imports_bundle_id is rejected
  prompt example goal_core_hash is a closed target Expr hash, not a hash of display text
  malformed goal_core_hash is rejected, but semantically stale goal_core_hash cannot be detected by the MVP release validator
  unknown prompt tag is rejected
```

---

# 17. 実装順序

Recommended order:

```text
1. Rename human library spec
   doc/phase6-human.md を標準ライブラリの人間向け設計として固定する

2. Certificate release loader
   Std.*.npcert を Phase 2 verifier で high-trust 検査し、module artifact を作る

3. Import bundle generator
   std.logic.mvp / std.nat.mvp / std.list.mvp / std.algebra-basic.mvp / std.all.mvp を作る

4. Theorem index generator
   public theorem / axiom ExportEntry から MachineStdTheoremEntry を作る

5. Theorem attribute derivation validator
   Simp / Rw / Apply attributes を validated modes から導出し、MVP で未使用の attributes を拒否する

6. Rewrite descriptor validator
   Phase 4 ResolvedSimpRule と同じ rule validation で lhs/rhs/telescope hash を導出する

7. Rewrite and simp profile generators
   std.logic.rw / std.nat.rw / std.list.rw / std.all.rw を作る
   std.logic.simp / std.nat.simp / std.list.simp / std.all.simp を作る

8. Phase 5 recipe integration tests
   import bundle + tactic options recipe が /machine/sessions で検証されることを確認する

9. Phase 7 retrieval fixtures
   basic Nat/List goals で exact/rw/simp candidate source を再現できることを確認する

10. Phase 8 audit checks
   sidecar と certificate verifier output の一致を独立に検査する
```

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
