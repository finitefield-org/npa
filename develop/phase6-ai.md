# Phase 6 AI Profile: Machine Standard Library

This document is the design for **Phase 6 for AI** in NPA.

`develop/phase6-human.md` covers the human-readable standard library structure,
names, theorem set, and attribute policy. AI proof search treats the standard
library not as a convenient source collection, but as deterministic machine
artifacts bound to verified certificates.

Phase 6 AI defines the wire contracts for standard-library release manifests,
import bundles, theorem metadata, simp metadata, rewrite metadata, and prompt
metadata used by the Phase 5 Machine IDE/API and Phase 7 AI search.

---

# 1. Purpose

Phase 6 AI provides:

```text
- deterministic imports of the standard-library certificate set for AI search
- theorem search / simp-lite / rw metadata bound to certificate_hash, export_hash, and decl_interface_hash
- non-trusted attributes, ranking, prompts, and embeddings outside certificate hashes
- import_closure / imports / tactic_options recipes usable by Phase 5 session create
- reproducible artifacts for Phase 7 premise retrieval and candidate generation
```

Trust rule:

```text
Not trusted:
  library source text
  notation / pretty statement
  attribute sidecar
  theorem ranking
  embedding vector
  usage statistics
  generated prompt text
  AI-generated proof hints

Trusted:
  Phase 2 canonical certificate bytes
  export_hash / certificate_hash / decl_interface_hash
  Phase 2 verifier output
  Phase 1 kernel check
  Phase 8 independent checker
```

Metadata may make the library useful for search, but proofs are accepted only
when generated proof certificates pass the kernel, certificate checker, and
required independent checker profiles.

---

# 2. Difference from Human Profile

Human Phase 6 centers:

```text
- definitions in Std.Logic / Std.Nat / Std.List / Std.Algebra.Basic
- theorem names
- simp theorem selection
- theorem roadmap
- human notation
```

AI Phase 6 centers:

```text
- MachineStdLibraryRelease
- MachineStdModuleArtifact
- MachineStdImportBundle
- MachineStdTheoremIndex
- MachineStdSimpProfile
- MachineStdRewriteProfile
- MachineStdPromptMetadata
- artifact hashes, validation order, and deterministic ordering
```

Library source text, tactic scripts, pretty statements, and natural-language
descriptions are not canonical artifact sources of truth. They may appear in
responses, prompts, or audit views, but not in certificate identity, import
closure identity, theorem identity, or simp rule identity.

---

# 3. Artifacts

The Phase 6 AI MVP release consists of:

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

## 3.1 Generated Artifact Policy

`Std/*.npcert` and `Std.machine-*.json` are release/build artifacts. In this
repository they are not committed as ordinary development deltas. Source
packages, Rust generators/validators, regression tests, and docs are the source
of truth. Tests regenerate raw `.npcert` and release sidecars in temporary
packages.

Distributed release packages may include generated artifacts, but artifact
identity is determined only by canonical bytes and validation order, not file
timestamps or JSON formatting.

Human debug JSON, source comments, pretty statements, and prompt metadata are
not inputs to `std_library_release_hash`.

Fixed MVP certificate locator:

```text
Std.Logic          -> Std/Logic.npcert
Std.Nat            -> Std/Nat.npcert
Std.List           -> Std/List.npcert
Std.Algebra.Basic  -> Std/Algebra/Basic.npcert
```

The MVP release module set is exact. `MachineStdLibraryRelease.modules` must
contain exactly these four names in `ModuleName` canonical order:

```text
Std.Nat
Std.List
Std.Logic
Std.Algebra.Basic
```

Missing modules, extra modules, duplicate modules, noncanonical order, or path
mismatch are `InvalidStdLibraryRelease`.

Legacy fixture names such as `Std.Nat.Basic` and `Std.Logic.Eq` may remain in
parser/resolver compatibility tests but are not MVP release modules and must be
rejected if they appear as release members, import bundle roots, or theorem
index owners.

The package locator is validation input, not trusted payload. Paths use POSIX
relative `/` separators and must not be absolute or contain empty components,
`.`, `..`, backslashes, trailing slashes, or duplicate slashes. If symlinks are
followed, the resolved target must remain inside the package root.

`Std/*.npcert` files contain raw Phase 2 `ModuleCertBytes`. When embedded into a
Phase 5 `VerifiedModuleCertificateRequest`, raw bytes are lowercase-hex encoded
with:

```text
certificate.encoding = "npa.certificate.canonical.v0.1.hex"
```

`certificate_bytes_hash` is sha256 of the raw Phase 2 certificate bytes, not the
hex text, path, or JSON wrapper.

JSON is a wire/storage form. Artifact identity comes from canonical bytes.
Object field order, whitespace, escape spelling, source file order, and
HashMap iteration order are not hash input. Duplicate keys and unknown fields
are rejected as `InvalidStdArtifactShape`.

Root objects:

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

`MachineStdLibraryRelease` is the only MVP root object without its own hash
field. Validators compute `std_library_release_hash` from release canonical
bytes. Sidecar roots carry their own hash fields and must validate before the
release manifest compares them.

Sidecar own-hash mismatches map to:

```text
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
```

Release manifest hash mismatches after sidecar validation are
`InvalidStdLibraryRelease`.

---

# 4. Release Manifest

## 4.1 MachineStdLibraryRelease

The release manifest records:

```text
- profile_version
- module artifacts
- import bundle hash
- theorem index hash
- simp profile hash
- rewrite profile hash
- axiom report hash
- optional prompt metadata hash
```

It binds machine sidecars to certificate bytes without making sidecars trusted
proof data.

## 4.2 MachineStdModuleArtifact

Each module artifact records:

```text
- module name
- fixed certificate locator path
- certificate_bytes_hash
- export_hash
- certificate_hash
- axiom_report_hash
- declaration count
- verifier profile
```

All hashes are recomputed from Phase 2 verifier output. A module artifact is
invalid if any field differs from verifier output.

## 4.3 Release Hash

`std_library_release_hash` is computed from canonical release bytes including
module artifact records and validated sidecar hashes. Optional prompt metadata
may be recorded but is excluded from the release hash unless a future profile
explicitly changes that rule.

---

# 5. Import Bundles

## 5.1 MachineStdImportBundle

An import bundle is a deterministic payload for Phase 5 session create.

It contains:

```text
- bundle name
- direct imports
- import_closure certificate payloads
- allow_axioms policy
- tactic option recipe reference
- bundle hash
```

Direct imports are the visible root imports. `import_closure` contains the
minimal transitive certificate closure needed by the Phase 2 verifier and Phase
4 kernel environment reconstruction.

Constructive bundles must not allow custom axioms. Classical bundles must list
allowed axioms explicitly. A bundle never authorizes a proof by itself; Phase 5
and Phase 2 revalidate the closure.

## 5.2 Bundle Order

Bundles, direct imports, and closure items use canonical order. The order must
not depend on filesystem order, JSON order, or hash-map iteration.

---

# 6. Tactic Options Recipes

Phase 6 provides recommended deterministic options for Phase 5 / Phase 4.

Recipes include:

```text
- Machine tactic profile
- deterministic budget defaults
- simp profile name
- rewrite profile name
- allowed tactic set
- maximum generated candidate size
```

Recipes are non-trusted convenience metadata. Phase 5 recomputes and validates
the options fingerprint when creating sessions.

---

# 7. Machine Theorem Index

## 7.1 Entry Set

The theorem index contains only declarations from verified MVP release modules.
Each entry is bound to:

```text
- module name
- export_hash
- certificate_hash
- declaration name
- decl_interface_hash
- statement core hash
- axiom dependencies
```

No entry is identified by pretty name or source text alone.

## 7.2 MachineStdTheoremIndex

The index records:

```text
- index profile version
- release hash binding
- entries in canonical order
- attribute sidecar hash
- index_hash
```

`index_hash` is recomputed from canonical index bytes. Ranking fields are
metadata; they are not proof evidence.

## 7.3 MachineStdTheoremEntry

Each entry contains:

```text
- global_ref
- module
- export_hash
- certificate_hash
- name
- decl_interface_hash
- type_hash / statement_core_hash
- axiom dependency summary
- theorem kind
- optional attributes
- optional prompt text
```

The entry is valid only if the referenced declaration exists in the module
export block with the same `decl_interface_hash`.

## 7.4 Attributes

Attributes include:

```text
simp
rewrite
intro
elim
congr
safe
unsafe_for_simp
domain tags
```

Attributes guide search and automation but are not trusted. Wrong attributes can
make search worse; they cannot prove a theorem.

## 7.5 Modes

Theorem index modes include:

```text
- premise retrieval
- rewrite retrieval
- simp registry construction
- prompt rendering
```

Every mode uses the same verified theorem identity.

---

# 8. Rewrite Metadata

## 8.1 MachineStdRewriteDescriptor

Rewrite descriptors bind a theorem to a rewrite view.

Fields:

```text
- theorem global_ref
- decl_interface_hash
- lhs core hash
- rhs core hash
- direction
- required Eq family
- side conditions, if any
- safety flag
```

The descriptor is valid only if Phase 4 rewrite validation can reconstruct the
same lhs/rhs from the theorem type and Eq family. Pretty rewrite patterns are
not identity.

## 8.2 MachineStdRewriteProfile

A rewrite profile is an ordered rule set for `rw` and `simp-lite`.

Ordering is deterministic:

```text
1. explicit profile priority
2. theorem name canonical order
3. decl_interface_hash bytes
```

Profiles carry their own hash and are bound from the release manifest.

## 8.3 Simp Lint

Simp lint validates:

```text
- rule theorem exists and hashes match
- rewrite descriptor matches theorem type
- no unsafe rule appears in a safe profile
- deterministic orientation is present
- obvious nontermination patterns are rejected or marked unsafe
```

Lint is a quality gate, not trusted proof evidence.

---

# 9. Simp Profiles

`MachineStdSimpProfile` defines rule sets for deterministic `simp-lite`.

It records:

```text
- profile name
- rule list
- priority
- safe/unsafe classification
- max default steps
- profile hash
```

Phase 4 revalidates rules before use. Phase 6 metadata only helps construct the
candidate rule set.

MVP safe `simp-lite` rules are fixed to the human Phase 6 release names:

```text
Std.Nat:
  Nat.add_zero
  Nat.add_succ
  Nat.zero_add
  Nat.mul_zero
  Nat.mul_succ
  Nat.zero_mul
  Nat.pred_zero
  Nat.pred_succ

Std.List:
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

MVP rewrite-only rules are exposed through rewrite profiles, not the safe
default simp profile:

```text
Std.Nat:
  Nat.add_comm
  Nat.add_assoc

Std.List:
  List.append_assoc
  List.length_append
```

---

# 10. Prompt Metadata

Prompt metadata may include:

```text
- short display names
- pretty statements
- doc snippets
- domain tags
- examples
- informal hints
```

Prompt metadata is optional and excluded from trusted release identity unless a
future profile explicitly includes a separate prompt metadata artifact hash. It
must still be bound to verified theorem identity by `decl_interface_hash`.

---

# 11. Axiom Policy

The standard release includes a machine axiom report derived from Phase 2
verifier output.

Policy views include:

```text
- module axioms
- per-declaration axiom dependencies
- custom axioms
- standard axioms
- contains_sorry
- safe_for_high_trust
```

Stored booleans are not trusted. Validators recompute policy decisions from the
canonical axiom report and configured allowlist.

The MVP kernel may represent the standard equality eliminator as the exact
`Eq.rec` family head imported from `Std.Logic`. In that case the generated axiom
report records only this standard exception:

```text
standard_axiom_exceptions:
  imported Std.Logic Eq.rec

module_axioms:
  Eq.rec

transitive_axioms:
  Eq.rec
```

---

# 12. Validation Order

Validation order:

```text
1. validate JSON schema shape and reject unknown fields / duplicate keys
2. load fixed module certificate bytes from fixed locator paths
3. verify each certificate with Phase 2 verifier
4. build module artifact table from verifier output
5. validate sidecar own hashes from canonical sidecar bytes
6. validate sidecar contents against module artifacts
7. compare validated sidecar hashes with release manifest fields
8. compute std_library_release_hash
9. expose import bundles / theorem index / profiles only after all checks pass
```

No sidecar may override certificate verifier output.

---

# 13. Phase 5 Integration

Phase 6 import bundles feed Phase 5 session create.

The handoff provides:

```text
- import_closure certificate payloads
- direct imports list
- verified theorem index fingerprint
- tactic options recipe
- simp / rewrite profile references
```

Phase 5 revalidates closure and options fingerprints. It does not trust a Phase
6 sidecar as a proof.

---

# 14. Phase 7 Integration

Phase 7 uses the theorem index, rewrite profiles, simp profiles, and prompt
metadata for retrieval and candidate generation.

Generated candidates still go through:

```text
Phase 5 run/batch
  ↓
Phase 4 Machine Tactic
  ↓
kernel check
  ↓
certificate verification
```

Metadata never authorizes acceptance.

---

# 15. Phase 8 Audit Hooks

Purpose:

```text
- allow the independent checker / audit layer to compare Phase 6 sidecars with verifier output
- audit reproducibility of machine artifacts without expanding the trusted base
```

Artifacts:

```text
- Phase 8 audit checklist implementation
- sidecar vs verifier output comparison
- manifest-bound sidecar hash audit
- optional prompt metadata exclusion audit
```

Completion checks:

```text
- release manifest hashes match certificate bytes and validated sidecar hashes
- optional prompt metadata is excluded from std_library_release_hash
- every decl_interface_hash / export_hash / certificate_hash matches verifier output
- every simp / rewrite profile target resolves to matching decl_interface_hash
- import bundles are minimal transitive closures
- constructive bundle allow_axioms contains no custom axioms
```

---

# 16. Tests

Tests must cover:

```text
- fixed MVP module set and canonical order
- fixed locator paths and path normalization rejection
- raw certificate byte hashing vs hex wrapper hashing
- sidecar own-hash mismatch errors
- release manifest sidecar hash mismatch errors
- theorem index entries bound to verifier output
- invalid theorem owner module rejection
- import bundle minimal closure
- constructive axiom policy
- rewrite descriptor validation against theorem type
- simp profile deterministic ordering
- prompt metadata exclusion from release hash
- Phase 5 session create from generated bundle
- Phase 7 retrieval fixture determinism
- Phase 8 audit comparison hooks
```

---

# 17. Milestones

## M0. Human / AI profile boundary fixed

Define generated artifacts as non-trusted metadata bound to certificates.

## M1. Certificate release loader

Load the exact four MVP module certificates from fixed locators and verify them
with the Phase 2 verifier.

## M2. Release manifest and axiom policy

Generate `MachineStdLibraryRelease`, module artifact records, and machine axiom
report. Validate no-custom-axiom policy.

## M3. Import bundle closure generator

Generate deterministic direct import bundles and minimal transitive closures for
Phase 5 session create.

## M4. Theorem index base generator

Generate theorem entries bound to module/export/certificate/declaration hashes.

## M5. Rewrite and simp profile generator

Generate validated rewrite descriptors, rewrite profiles, and simp profiles.

## M6. Theorem index metadata finalizer

Attach non-trusted attributes, prompt metadata references, and deterministic
profile identifiers.

## M7. Phase 5 session handoff

Verify Phase 5 can create sessions from generated import bundles and tactic
option recipes.

## M8. Phase 7 retrieval fixtures

Verify Phase 7 retrieval and candidate generation fixtures are deterministic.

## M9. Phase 8 audit hooks

Verify sidecars match certificate verifier output and release manifest bindings.

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

M3 depends on the M2 manifest/module table. M4 depends on the M2 verified module
context table. M5 depends on M3 import bundle identity, M4 theorem identity, and
Phase 4 `ResolvedSimpRule`. M7 hands M3/M6 artifacts to Phase 5. M8 and later
fix Phase 7 / Phase 8 usage against Phase 5-verifiable candidate sources.

---

# 18. Excluded

The MVP excludes:

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

Embeddings and usage statistics may be added later as untrusted Phase 7 / Phase
9 ranking sidecars with separate artifact hashes and profile ids.

---

# 19. Completion Criteria

Phase 6 AI is complete when:

```text
- standard library certificates can be checked in high-trust mode
- release manifest is fixed to certificate bytes / export_hash / certificate_hash
- import bundles usable by Phase 5 can be generated
- no-custom-axiom policy can be checked from axiom reports
- theorem index entries are fixed to decl_interface_hash and export_hash
- simp / rw metadata matches Phase 4 rule validation
- recommended tactic options recipe is revalidated by Phase 5
- Phase 7 metadata-derived candidates are not accepted without Phase 5 run/batch
- Phase 8 can audit sidecars against certificate verifier output
```

---

# 20. One-Sentence Summary

Phase 6 AI exposes the standard library as a certificate-bound machine artifact
set for AI search: verified release manifest, deterministic import bundles,
hash-bound theorem index, and non-trusted simp/rewrite metadata that Phase 5 and
later checkers revalidate before any proof is accepted.
